#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

//-----------------------------------------------------------------------------
// File: dbg_console_xbox.cpp
//
// Desc: Listens for string commands sent from a debug console on a
//       remote dev machine, and forwards them to the Q3 engine.
//
//       Commands are sent from the remote debug console through the debug
//       channel to the debug monitor on the Xbox machine.  The Xbox machine
//       receives the commands on a separate thread through a
//       registered command processor callback function. The callback
//       function will store commands in a buffer, and the app should
//       poll this buffer once per frame and then decipher and handle
//       the commands.
//
// Hist: 02.05.01 - Initial creation for March XDK release
//       08.21.02 - Revision and code cleanup
//       04.10.02 - Buthcered by BTO for use in JK3:JA
//
// Copyright (c) Microsoft Corporation. All rights reserved.
//-----------------------------------------------------------------------------

// Windows API and libc function declarations
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn lstrcpynA(lpString1: *mut c_char, lpString2: *const c_char, iMaxLength: c_int) -> *mut c_char;
    fn lstrcpyA(lpString1: *mut c_char, lpString2: *const c_char) -> *mut c_char;
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn DmRegisterCommandProcessor(
        lpName: *const c_char,
        pdmcc: extern "C" fn(*const c_char, *mut c_char, c_int, *mut c_void) -> c_int,
    ) -> c_int;
    fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
    fn va(format: *const c_char, ...) -> *const c_char;
}

// Windows CRITICAL_SECTION type
#[repr(C)]
pub struct CRITICAL_SECTION {
    DebugInfo: *mut c_void,
    LockCount: i32,
    RecursionCount: i32,
    OwningThread: *mut c_void,
    LockSemaphore: *mut c_void,
    SpinCount: usize,
}

const MAXRCMDLENGTH: c_int = 256;
const XBDM_NOERR: c_int = 0;

// Command prefix for things sent across the dubg channel
static g_strDebugConsoleCommandPrefix: &[u8] = b"XCMD";


// Global buffer to receive remote commands from the debug console. Note that
// since this data is accessed by the app's main thread, and the debug monitor
// thread, we need to protect access with a critical section
static mut g_strRemoteBuf: [c_char; 256] = [0; 256];


// The critical section used to protect data that is shared between threads
static mut g_CriticalSection: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: core::ptr::null_mut(),
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};


// Temporary replacement for CRT string funcs, since
// we can't call CRT functions on the debug monitor
// thread right now.


//-----------------------------------------------------------------------------
// Name: dbgtolower()
// Desc: Returns lowercase of char
//-----------------------------------------------------------------------------
#[inline]
fn dbgtolower(ch: c_char) -> c_char {
    if ch >= b'A' as c_char && ch <= b'Z' as c_char {
        ch - (b'A' - b'a') as c_char
    } else {
        ch
    }
}


//-----------------------------------------------------------------------------
// Name: dbgstrnicmp()
// Desc: Critical section safe string compare.
//-----------------------------------------------------------------------------
fn dbgstrnicmp(str1: *const c_char, str2: *const c_char, mut n: c_int) -> bool {
    unsafe {
        while dbgtolower(*str1) == dbgtolower(*str2) && *str1 != 0 && n > 0 {
            n -= 1;
            str1 = str1.add(1);
            str2 = str2.add(1);
        }

        n == 0 || dbgtolower(*str1) == dbgtolower(*str2)
    }
}


//-----------------------------------------------------------------------------
// Name: dbgstrcpy()
// Desc: Critical section safe string copy
//-----------------------------------------------------------------------------
fn dbgstrcpy(mut strDest: *mut c_char, mut strSrc: *const c_char) {
    unsafe {
        loop {
            let c = *strSrc;
            *strDest = c;
            if c == 0 {
                break;
            }
            strDest = strDest.add(1);
            strSrc = strSrc.add(1);
        }
    }
}


//-----------------------------------------------------------------------------
// Name: DebugConsoleCmdProcessor()
// Desc: Command notification proc that is called by the Xbox debug monitor to
//       have us process a command.  What we'll actually attempt to do is tell
//       it to make calls to us on a separate thread, so that we can just block
//       until we're able to process a command.
//
// Note: Do NOT include newlines in the response string! To do so will confuse
//       the internal WinSock networking code used by the debug monitor API.
//-----------------------------------------------------------------------------
extern "C" fn DebugConsoleCmdProcessor(
    strCommand: *const c_char,
    strResponse: *mut c_char,
    dwResponseLen: c_int,
    _pdmcc: *mut c_void,
) -> c_int {
    unsafe {
        // Skip over the command prefix and the exclamation mark
        let strCommand = strCommand.add(strlen(g_strDebugConsoleCommandPrefix.as_ptr() as *const c_char) + 1);

        // Check if this is the initial connect signal
        if dbgstrnicmp(strCommand, b"__connect__\0".as_ptr() as *const c_char, 11) {
            // If so, respond that we're connected
            lstrcpynA(strResponse, b"Connected.\0".as_ptr() as *const c_char, dwResponseLen);
            return XBDM_NOERR;
        }

        // g_strRemoteBuf needs to be protected by the critical section
        EnterCriticalSection(&mut g_CriticalSection);
        if g_strRemoteBuf[0] != 0 {
            // This means the application has probably stopped polling for debug commands
            dbgstrcpy(strResponse, b"Cannot execute - previous command still pending\0".as_ptr() as *const c_char);
        } else {
            dbgstrcpy(g_strRemoteBuf.as_mut_ptr(), strCommand);
        }
        LeaveCriticalSection(&mut g_CriticalSection);

        XBDM_NOERR
    }
}


//-----------------------------------------------------------------------------
// Name: DebugConsoleHandleCommands()
// Desc: Poll routine called periodically (typically every frame) by the Xbox
//       app to see if there is a command waiting to be executed, and if so,
//       execute it.
//-----------------------------------------------------------------------------
pub extern "C" fn DebugConsoleHandleCommands() -> c_int {
    static mut bInitialized: bool = false;
    let mut strLocalBuf: [c_char; 257] = [0; 257]; // local copy of command

    unsafe {
        // Initialize ourselves when we're first called.
        if !bInitialized {
            // Register our command handler with the debug monitor
            let hr = DmRegisterCommandProcessor(
                g_strDebugConsoleCommandPrefix.as_ptr() as *const c_char,
                DebugConsoleCmdProcessor,
            );
            if hr != XBDM_NOERR {
                return 0; // FALSE
            }

            // We'll also need a critical section to protect access to g_strRemoteBuf
            InitializeCriticalSection(&mut g_CriticalSection);

            bInitialized = true;
        }

        // If there's nothing waiting, return.
        if g_strRemoteBuf[0] == 0 {
            return 0; // FALSE
        }

        // Grab a local copy of the command received in the remote buffer
        EnterCriticalSection(&mut g_CriticalSection);

        lstrcpyA(strLocalBuf.as_mut_ptr(), g_strRemoteBuf.as_ptr());
        g_strRemoteBuf[0] = 0;

        LeaveCriticalSection(&mut g_CriticalSection);

        let fmt_str = va(b"%s\n\0".as_ptr() as *const c_char, strLocalBuf.as_ptr());
        Cbuf_ExecuteText(1, fmt_str); // EXEC_APPEND = 1

        1 // TRUE
    }
}
