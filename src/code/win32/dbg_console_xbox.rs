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

use core::ffi::{c_char, c_int, c_void, c_ulong};

const MAXRCMDLENGTH: usize = 256; // Size of the remote cmd buffer

// Command prefix for things sent across the dubg channel
static g_strDebugConsoleCommandPrefix: &[u8] = b"XCMD";

// Global buffer to receive remote commands from the debug console. Note that
// since this data is accessed by the app's main thread, and the debug monitor
// thread, we need to protect access with a critical section
static mut g_strRemoteBuf: [c_char; MAXRCMDLENGTH] = [0; MAXRCMDLENGTH];

// The critical section used to protect data that is shared between threads
static mut g_CriticalSection: CRITICAL_SECTION = CRITICAL_SECTION { _opaque: [0; 40] };

// Temporary replacement for CRT string funcs, since
// we can't call CRT functions on the debug monitor
// thread right now.

//-----------------------------------------------------------------------------
// Name: dbgtolower()
// Desc: Returns lowercase of char
//-----------------------------------------------------------------------------
#[inline]
unsafe fn dbgtolower(ch: c_char) -> c_char {
    if ch >= b'A' as c_char && ch <= b'Z' as c_char {
        ch - (b'A' as c_char - b'a' as c_char)
    } else {
        ch
    }
}

//-----------------------------------------------------------------------------
// Name: dbgstrnicmp()
// Desc: Critical section safe string compare.
//-----------------------------------------------------------------------------
unsafe fn dbgstrnicmp(str1: *const c_char, str2: *const c_char, mut n: c_int) -> c_int {
    let mut s1 = str1;
    let mut s2 = str2;

    while dbgtolower(*s1) == dbgtolower(*s2) && *s1 != 0 && n > 0 {
        n -= 1;
        s1 = s1.add(1);
        s2 = s2.add(1);
    }

    if n == 0 || dbgtolower(*s1) == dbgtolower(*s2) {
        1
    } else {
        0
    }
}

//-----------------------------------------------------------------------------
// Name: dbgstrcpy()
// Desc: Critical section safe string copy
//-----------------------------------------------------------------------------
unsafe fn dbgstrcpy(strDest: *mut c_char, strSrc: *const c_char) {
    let mut dest = strDest;
    let mut src = strSrc;

    loop {
        *dest = *src;
        if *src == 0 {
            break;
        }
        dest = dest.add(1);
        src = src.add(1);
    }
}

// Windows/Xbox API types and stubs
#[repr(C)]
struct CRITICAL_SECTION {
    _opaque: [u8; 40], // Opaque critical section structure
}

type HRESULT = c_int;
type DWORD = c_ulong;
type BOOL = c_int;
type PDM_CMDCONT = *mut c_void;

const XBDM_NOERR: HRESULT = 0;
const EXEC_APPEND: c_int = 0;

// External C functions from Xbox XDK and game engine
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn lstrcpynA(lpString1: *mut c_char, lpString2: *const c_char, iMaxLength: c_int)
        -> *mut c_char;
    fn lstrcpyA(lpString1: *mut c_char, lpString2: *const c_char) -> *mut c_char;
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn DmRegisterCommandProcessor(
        strPrefix: *const c_char,
        pdmccCallback: extern "C" fn(*const c_char, *mut c_char, DWORD, PDM_CMDCONT) -> HRESULT,
    ) -> HRESULT;
    fn Cbuf_ExecuteText(exec_level: c_int, text: *const c_char);
    fn va(format: *const c_char, ...) -> *const c_char;
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
    dwResponseLen: DWORD,
    pdmcc: PDM_CMDCONT,
) -> HRESULT {
    unsafe {
        // Skip over the command prefix and the exclamation mark
        let strCommand = strCommand.add(strlen(g_strDebugConsoleCommandPrefix.as_ptr() as *const c_char) + 1);

        // Check if this is the initial connect signal
        if dbgstrnicmp(strCommand, b"__connect__\0".as_ptr() as *const c_char, 11) != 0 {
            // If so, respond that we're connected
            lstrcpynA(
                strResponse,
                b"Connected.\0".as_ptr() as *const c_char,
                dwResponseLen as c_int,
            );
            return XBDM_NOERR;
        }

        // g_strRemoteBuf needs to be protected by the critical section
        EnterCriticalSection(&mut g_CriticalSection);
        if g_strRemoteBuf[0] != 0 {
            // This means the application has probably stopped polling for debug commands
            dbgstrcpy(
                strResponse,
                b"Cannot execute - previous command still pending\0".as_ptr() as *const c_char,
            );
        } else {
            dbgstrcpy(&mut g_strRemoteBuf[0], strCommand);
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
pub fn DebugConsoleHandleCommands() -> BOOL {
    static mut bInitialized: BOOL = 0;
    let mut strLocalBuf: [c_char; MAXRCMDLENGTH + 1] = [0; MAXRCMDLENGTH + 1];

    unsafe {
        // Initialize ourselves when we're first called.
        if bInitialized == 0 {
            // Register our command handler with the debug monitor
            let hr = DmRegisterCommandProcessor(
                g_strDebugConsoleCommandPrefix.as_ptr() as *const c_char,
                DebugConsoleCmdProcessor,
            );
            if hr != 0 {
                return 0; // FALSE
            }

            // We'll also need a critical section to protect access to g_strRemoteBuf
            InitializeCriticalSection(&mut g_CriticalSection);

            bInitialized = 1; // TRUE
        }

        // If there's nothing waiting, return.
        if g_strRemoteBuf[0] == 0 {
            return 0; // FALSE
        }

        // Grab a local copy of the command received in the remote buffer
        EnterCriticalSection(&mut g_CriticalSection);

        lstrcpyA(&mut strLocalBuf[0], &g_strRemoteBuf[0]);
        g_strRemoteBuf[0] = 0;

        LeaveCriticalSection(&mut g_CriticalSection);

        Cbuf_ExecuteText(
            EXEC_APPEND,
            va(b"%s\n\0".as_ptr() as *const c_char, &strLocalBuf[0]),
        );

        1 // TRUE
    }
}
