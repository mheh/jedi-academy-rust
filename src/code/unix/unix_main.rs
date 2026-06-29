use core::ffi::{c_int, c_char, c_void};
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;
use std::os::unix::ffi::OsStrExt;
use libc::{
    uid_t, geteuid, getuid, seteuid, fcntl, signal, getpagesize,
    select, read, write, fseek, fread, dlopen, dlsym, dlclose,
    FILE, timeval, fd_set, FD_ZERO, FD_SET, FD_ISSET, FNDELAY, F_SETFL, F_GETFL,
    RTLD_LAZY, SIGFPE, getcwd, stat, SEEK_SET, malloc, free, memcpy, memset,
};
use std::ptr::{addr_of, addr_of_mut};

// ===========================================================================
// Forward declarations for external functions
// ===========================================================================

extern "C" {
    pub static mut nostdout: *mut core::ffi::c_void;  // cvar_t*
    pub static mut com_dedicated: *mut core::ffi::c_void;  // cvar_t*

    fn IN_Shutdown();
    fn IN_Init();
    fn IN_Frame();
    fn Cmd_AddCommand(cmd: *const c_char, func: extern "C" fn());
    fn Sys_SetFPCW();
    fn Cvar_Set(name: *const c_char, value: *const c_char);
    fn CL_Shutdown();
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_sprintf(buffer: *mut c_char, bufsize: libc::size_t, fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_Init(cmdline: *const c_char);
    fn NET_Init();
    fn Com_Frame();
    fn Sys_Milliseconds() -> c_int;
    fn Sys_SendKeyEvents();
    fn Sys_GetPacket(adr: *mut libc::c_void, msg: *mut libc::c_void) -> c_int;
    fn Sys_LowFPPrecision();
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut core::ffi::c_void;  // cvar_t*
    fn Z_Malloc(size: c_int) -> *mut libc::c_void;
    fn Z_Free(ptr: *mut libc::c_void);
    fn MSG_Init(msg: *mut libc::c_void, buffer: *mut u8, size: c_int);
    fn GetRefAPI(version: c_int, parms: *mut libc::c_void) -> *mut libc::c_void;
}

// Structure containing functions exported from refresh DLL
pub static mut re: core::ffi::c_void = unsafe { mem::zeroed() };

pub static mut sys_frame_time: c_int = 0;

pub static mut saved_euid: uid_t = 0;
pub static mut stdin_active: c_int = 1;  // qtrue

// =======================================================================
// General routines
// =======================================================================

#[allow(non_snake_case)]
pub extern "C" fn Sys_BeginProfiling() {
}

/*
=================
Sys_In_Restart_f

Restart the input subsystem
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_In_Restart_f() {
    unsafe {
        IN_Shutdown();
        IN_Init();
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_ConsoleOutput(string: *mut c_char) {
    unsafe {
        if !nostdout.is_null() {
            // Check nostdout->value
            let nostdout_cvar = nostdout as *const libc::c_void;
            if nostdout_cvar as *const c_int as usize != 0 {
                let value = *(nostdout_cvar as *const c_int);
                if value != 0 {
                    return;
                }
            }
        }

        let cstr = CStr::from_ptr(string);
        print!("{}", cstr.to_string_lossy());
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_Printf(fmt: *const c_char, mut args: ...) {
    unsafe {
        let mut text: [c_char; 1024] = [0; 1024];
        let mut argptr = args.as_va_list();
        libc::vsprintf(text.as_mut_ptr(), fmt, argptr);

        let text_len = libc::strlen(text.as_ptr());
        if text_len > 1024 {
            Com_Error(3, b"memory overwrite in Sys_Printf\0".as_ptr() as *const c_char);
        }

        // Check nostdout
        if !nostdout.is_null() {
            let nostdout_cvar = nostdout as *const libc::c_void;
            if nostdout_cvar as *const c_int as usize != 0 {
                let value = *(nostdout_cvar as *const c_int);
                if value != 0 {
                    return;
                }
            }
        }

        let mut p = text.as_mut_ptr() as *mut u8;
        while *p != 0 {
            *p &= 0x7f;
            if (*p > 128 || *p < 32) && *p != 10 && *p != 13 && *p != 9 {
                libc::printf(b"[%02x]\0".as_ptr() as *const c_char, *p);
            } else {
                libc::putc(*p as c_int, libc::stdout);
            }
            p = p.add(1);
        }
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_Quit() {
    unsafe {
        CL_Shutdown();
        fcntl(0, F_SETFL, fcntl(0, F_GETFL, 0) & !FNDELAY);
        libc::_exit(0);
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_Init() {
    unsafe {
        Cmd_AddCommand(b"in_restart\0".as_ptr() as *const c_char, Sys_In_Restart_f);

        #[cfg(target_arch = "x86")]
        Sys_SetFPCW();

        #[cfg(target_os = "linux")]
        {
            #[cfg(target_arch = "x86")]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"linux i386\0".as_ptr() as *const c_char);
            #[cfg(target_arch = "x86_64")]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"linux alpha\0".as_ptr() as *const c_char);
            #[cfg(target_arch = "sparc64")]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"linux sparc\0".as_ptr() as *const c_char);
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "sparc64")))]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"linux unknown\0".as_ptr() as *const c_char);
        }

        #[cfg(target_os = "solaris")]
        {
            #[cfg(target_arch = "x86")]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"solaris x86\0".as_ptr() as *const c_char);
            #[cfg(target_arch = "sparc64")]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"solaris sparc\0".as_ptr() as *const c_char);
            #[cfg(not(any(target_arch = "x86", target_arch = "sparc64")))]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"solaris unknown\0".as_ptr() as *const c_char);
        }

        #[cfg(target_os = "irix")]
        {
            #[cfg(target_arch = "mips")]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"sgi mips\0".as_ptr() as *const c_char);
            #[cfg(not(target_arch = "mips"))]
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"sgi unknown\0".as_ptr() as *const c_char);
        }

        #[cfg(not(any(target_os = "linux", target_os = "solaris", target_os = "irix")))]
        Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"unknown\0".as_ptr() as *const c_char);

        IN_Init();
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_Error(error: *const c_char, mut args: ...) {
    unsafe {
        // change stdin to non blocking
        fcntl(0, F_SETFL, fcntl(0, F_GETFL, 0) & !FNDELAY);

        CL_Shutdown();

        let mut string: [c_char; 1024] = [0; 1024];
        let mut argptr = args.as_va_list();
        libc::vsprintf(string.as_mut_ptr(), error, argptr);

        libc::fprintf(libc::stderr, b"Error: %s\n\0".as_ptr() as *const c_char, string.as_ptr());

        libc::_exit(1);
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_Warn(warning: *const c_char, mut args: ...) {
    unsafe {
        let mut string: [c_char; 1024] = [0; 1024];
        let mut argptr = args.as_va_list();
        libc::vsprintf(string.as_mut_ptr(), warning, argptr);

        libc::fprintf(libc::stderr, b"Warning: %s\0".as_ptr() as *const c_char, string.as_ptr());
    }
}

/*
============
Sys_FileTime

returns -1 if not present
============
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_FileTime(path: *mut c_char) -> c_int {
    unsafe {
        let mut buf: libc::stat = mem::zeroed();

        if stat(path, &mut buf) == -1 {
            return -1;
        }

        buf.st_mtime as c_int
    }
}

#[allow(non_snake_case)]
pub extern "C" fn floating_point_exception_handler(_whatever: c_int) {
    // Sys_Warn("floating point exception\n");
    unsafe {
        signal(SIGFPE, floating_point_exception_handler as libc::sighandler_t);
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_ConsoleInput() -> *mut c_char {
    unsafe {
        static mut text: [c_char; 256] = [0; 256];
        let mut fdset: fd_set = mem::zeroed();
        let mut timeout: timeval = mem::zeroed();
        let mut len: c_int;

        if com_dedicated.is_null() {
            let com_ded = com_dedicated as *const c_int;
            if *com_ded == 0 {
                return ptr::null_mut();
            }
        } else {
            return ptr::null_mut();
        }

        if stdin_active == 0 {
            return ptr::null_mut();
        }

        FD_ZERO(&mut fdset);
        FD_SET(0, &mut fdset);  // stdin
        timeout.tv_sec = 0;
        timeout.tv_usec = 0;

        if select(1, &mut fdset, ptr::null_mut(), ptr::null_mut(), &mut timeout) == -1 ||
           !FD_ISSET(0, &fdset) {
            return ptr::null_mut();
        }

        len = read(0, text.as_mut_ptr() as *mut libc::c_void, text.len()) as c_int;
        if len == 0 {  // eof!
            stdin_active = 0;  // qfalse
            return ptr::null_mut();
        }

        if len < 1 {
            return ptr::null_mut();
        }

        text[(len - 1) as usize] = 0;  // rip off the /n and terminate

        text.as_mut_ptr()
    }
}

/*****************************************************************************/

static mut game_library: *mut c_void = ptr::null_mut();

#[cfg(target_arch = "x86")]
const GAMENAME: &[u8] = b"qagamei386.so\0";
#[cfg(target_arch = "x86_64")]
const GAMENAME: &[u8] = b"qagameaxp.so\0";
#[cfg(target_arch = "mips")]
const GAMENAME: &[u8] = b"qagamemips.so\0";

/*
=================
Sys_UnloadGame
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_UnloadGame() {
    unsafe {
        Com_Printf(b"------ Unloading %s ------\n\0".as_ptr() as *const c_char, GAMENAME.as_ptr());
        if !game_library.is_null() {
            dlclose(game_library);
        }
        game_library = ptr::null_mut();
    }
}

/*
=================
Sys_GetGameAPI

Loads the game dll
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_GetGameAPI(parms: *mut c_void) -> *mut c_void {
    unsafe {
        let mut name: [c_char; 256] = [0; 256];  // MAX_OSPATH
        let mut curpath: [c_char; 256] = [0; 256];  // MAX_OSPATH

        if !game_library.is_null() {
            Com_Error(3, b"Sys_GetGameAPI without Sys_UnloadingGame\0".as_ptr() as *const c_char);
        }

        // check the current debug directory first for development purposes
        getcwd(curpath.as_mut_ptr(), curpath.len());

        Com_Printf(b"------- Loading %s -------\n\0".as_ptr() as *const c_char, GAMENAME.as_ptr());
        Com_sprintf(name.as_mut_ptr(), name.len(), b"%s/%s\0".as_ptr() as *const c_char,
                    curpath.as_ptr(), GAMENAME.as_ptr());

        game_library = dlopen(name.as_ptr(), RTLD_LAZY);
        if !game_library.is_null() {
            Com_DPrintf(b"LoadLibrary (%s)\n\0".as_ptr() as *const c_char, name.as_ptr());
        } else {
            Com_Printf(b"LoadLibrary(\"%s\") failed\n\0".as_ptr() as *const c_char, name.as_ptr());
            Com_Printf(b"...reason: '%s'\n\0".as_ptr() as *const c_char, dlerror());
            Com_Error(3, b"Couldn't load game\0".as_ptr() as *const c_char);
        }

        let GetGameAPI = dlsym(game_library, b"GetGameAPI\0".as_ptr() as *const c_char);
        if GetGameAPI.is_null() {
            Sys_UnloadGame();
            return ptr::null_mut();
        }

        let func: extern "C" fn(*mut c_void) -> *mut c_void = mem::transmute(GetGameAPI);
        func(parms)
    }
}

/*****************************************************************************/

static mut cgame_library: *mut c_void = ptr::null_mut();

#[cfg(target_arch = "x86")]
const CGAMENAME: &[u8] = b"cgamei386.so\0";
#[cfg(target_arch = "x86_64")]
const CGAMENAME: &[u8] = b"cgameaxp.so\0";
#[cfg(target_arch = "mips")]
const CGAMENAME: &[u8] = b"cgamemips.so\0";

/*
=================
Sys_UnloadGame
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_UnloadCGame() {
    unsafe {
        if !cgame_library.is_null() {
            dlclose(cgame_library);
        }
        cgame_library = ptr::null_mut();
    }
}

/*
=================
Sys_GetGameAPI

Loads the game dll
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_GetCGameAPI() -> *mut c_void {
    unsafe {
        let mut name: [c_char; 256] = [0; 256];  // MAX_OSPATH
        let mut curpath: [c_char; 256] = [0; 256];  // MAX_OSPATH

        Sys_UnloadCGame();

        getcwd(curpath.as_mut_ptr(), curpath.len());

        Com_Printf(b"------- Loading %s -------\n\0".as_ptr() as *const c_char, CGAMENAME.as_ptr());

        libc::sprintf(name.as_mut_ptr(), b"%s/%s\0".as_ptr() as *const c_char,
                      curpath.as_ptr(), CGAMENAME.as_ptr());
        cgame_library = dlopen(name.as_ptr(), RTLD_LAZY);
        if cgame_library.is_null() {
            Com_Printf(b"LoadLibrary (%s)\n\0".as_ptr() as *const c_char, name.as_ptr());
            Com_Error(3, b"Couldn't load cgame: %s\0".as_ptr() as *const c_char, dlerror());
        }

        let api = dlsym(cgame_library, b"GetCGameAPI\0".as_ptr() as *const c_char);
        if api.is_null() {
            Com_Error(3, b"dlsym() failed on GetCGameAPI\0".as_ptr() as *const c_char);
        }

        let func: extern "C" fn() -> *mut c_void = mem::transmute(api);
        func()
    }
}

/*****************************************************************************/

static mut ui_library: *mut c_void = ptr::null_mut();

#[cfg(target_arch = "x86")]
const UINAME: &[u8] = b"uii386.so\0";
#[cfg(target_arch = "x86_64")]
const UINAME: &[u8] = b"uiaxp.so\0";
#[cfg(target_arch = "mips")]
const UINAME: &[u8] = b"uimips.so\0";

/*
=================
Sys_UnloadUI
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_UnloadUI() {
    unsafe {
        if !ui_library.is_null() {
            dlclose(ui_library);
        }
        ui_library = ptr::null_mut();
    }
}

/*
=================
Sys_GetUIAPI

Loads the ui dll
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_GetUIAPI() -> *mut c_void {
    unsafe {
        let mut name: [c_char; 256] = [0; 256];  // MAX_OSPATH
        let mut curpath: [c_char; 256] = [0; 256];  // MAX_OSPATH

        Sys_UnloadUI();

        getcwd(curpath.as_mut_ptr(), curpath.len());

        Com_Printf(b"------- Loading %s -------\n\0".as_ptr() as *const c_char, UINAME.as_ptr());

        libc::sprintf(name.as_mut_ptr(), b"%s/%s\0".as_ptr() as *const c_char,
                      curpath.as_ptr(), UINAME.as_ptr());
        ui_library = dlopen(name.as_ptr(), RTLD_LAZY);
        if ui_library.is_null() {
            Com_Printf(b"LoadLibrary (%s)\n\0".as_ptr() as *const c_char, name.as_ptr());
            Com_Error(3, b"Couldn't load ui: %s\0".as_ptr() as *const c_char, dlerror());
        }

        let api = dlsym(ui_library, b"GetUIAPI\0".as_ptr() as *const c_char);
        if api.is_null() {
            Com_Error(3, b"dlsym() failed on GetUIAPI\0".as_ptr() as *const c_char);
        }

        api
    }
}

/*****************************************************************************/

#[allow(non_snake_case)]
pub extern "C" fn Sys_GetRefAPI(parms: *mut c_void) -> *mut c_void {
    unsafe {
        let api_version = 8;  // REF_API_VERSION
        GetRefAPI(api_version, parms)
    }
}

/*
========================================================================

BACKGROUND FILE STREAMING

========================================================================
*/

#[repr(C)]
struct streamState_t {
    file: *mut FILE,
    buffer: *mut u8,
    eof: c_int,
    bufferSize: c_int,
    streamPosition: c_int,  // next byte to be returned by Sys_StreamRead
    threadPosition: c_int,  // next byte to be read from file
}

static mut stream: streamState_t = streamState_t {
    file: ptr::null_mut(),
    buffer: ptr::null_mut(),
    eof: 0,
    bufferSize: 0,
    streamPosition: 0,
    threadPosition: 0,
};

/*
===============
Sys_StreamThread

A thread will be sitting in this loop forever
================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_StreamThread() {
    unsafe {
        let mut buffer: c_int;
        let mut count: c_int;
        let mut readCount: c_int;
        let mut bufferPoint: c_int;
        let mut r: c_int;

        //Loop here
        //  EnterCriticalSection (&stream.crit);

        // if there is any space left in the buffer, fill it up
        if stream.eof == 0 {
            count = stream.bufferSize - (stream.threadPosition - stream.streamPosition);
            if count != 0 {
                bufferPoint = stream.threadPosition % stream.bufferSize;
                buffer = stream.bufferSize - bufferPoint;
                readCount = if buffer < count { buffer } else { count };
                r = fread(stream.buffer.add(bufferPoint as usize), 1, readCount as usize, stream.file) as c_int;
                stream.threadPosition += r;

                if r != readCount {
                    stream.eof = 1;  // qtrue
                }
            }
        }

        //  LeaveCriticalSection (&stream.crit);
    }
}

/*
===============
Sys_InitStreamThread

================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_InitStreamThread() {
}

/*
===============
Sys_ShutdownStreamThread

================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_ShutdownStreamThread() {
}


/*
===============
Sys_BeginStreamedFile

================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_BeginStreamedFile(f: *mut FILE, readAhead: c_int) {
    unsafe {
        if !stream.file.is_null() {
            Com_Error(3, b"Sys_BeginStreamedFile: unclosed stream\0".as_ptr() as *const c_char);
        }

        stream.file = f;
        stream.buffer = Z_Malloc(readAhead) as *mut u8;
        stream.bufferSize = readAhead;
        stream.streamPosition = 0;
        stream.threadPosition = 0;
        stream.eof = 0;  // qfalse

        // let the thread start running
        //  LeaveCriticalSection( &stream.crit );
    }
}

/*
===============
Sys_EndStreamedFile

================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_EndStreamedFile(f: *mut FILE) {
    unsafe {
        if f != stream.file {
            Com_Error(3, b"Sys_EndStreamedFile: wrong file\0".as_ptr() as *const c_char);
        }
        // don't leave critical section until another stream is started
        //  EnterCriticalSection( &stream.crit );

        stream.file = ptr::null_mut();
        Z_Free(stream.buffer as *mut c_void);
    }
}


/*
===============
Sys_StreamedRead

================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: *mut FILE) -> c_int {
    unsafe {
        let mut available: c_int;
        let mut remaining: c_int;
        let mut sleepCount: c_int;
        let mut copy: c_int;
        let mut bufferCount: c_int;
        let mut bufferPoint: c_int;
        let mut dest: *mut u8 = buffer as *mut u8;

        remaining = size * count;

        if remaining <= 0 {
            Com_Error(3, b"Streamed read with non-positive size\0".as_ptr() as *const c_char);
        }

        sleepCount = 0;
        while remaining > 0 {
            available = stream.threadPosition - stream.streamPosition;
            if available == 0 {
                if stream.eof != 0 {
                    break;
                }
                Sys_StreamThread();
                continue;
            }

            bufferPoint = stream.streamPosition % stream.bufferSize;
            bufferCount = stream.bufferSize - bufferPoint;

            copy = if available < bufferCount { available } else { bufferCount };
            if copy > remaining {
                copy = remaining;
            }
            memcpy(dest as *mut c_void, stream.buffer.add(bufferPoint as usize) as *const c_void, copy as usize);
            stream.streamPosition += copy;
            dest = dest.add(copy as usize);
            remaining -= copy;
        }

        (count * size - remaining) / size
    }
}

/*
===============
Sys_StreamSeek

================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_StreamSeek(f: *mut FILE, offset: c_int, origin: c_int) {
    unsafe {
        // halt the thread
        //  EnterCriticalSection( &stream.crit );

        // clear to that point
        fseek(f, offset as i64, origin);
        stream.streamPosition = 0;
        stream.threadPosition = 0;
        stream.eof = 0;  // qfalse

        // let the thread start running at the new position
        //  LeaveCriticalSection( &stream.crit );
    }
}


/*
========================================================================

EVENT LOOP

========================================================================
*/

const MAX_QUED_EVENTS: usize = 64;
const MASK_QUED_EVENTS: usize = MAX_QUED_EVENTS - 1;
const MAX_MSGLEN: usize = 16384;

#[repr(C)]
struct sysEvent_t {
    evTime: c_int,
    evType: c_int,
    evValue: c_int,
    evValue2: c_int,
    evPtrLength: c_int,
    evPtr: *mut c_void,
}

static mut eventQue: [sysEvent_t; MAX_QUED_EVENTS] = unsafe { [mem::zeroed(); MAX_QUED_EVENTS] };
static mut eventHead: c_int = 0;
static mut eventTail: c_int = 0;
static mut sys_packetReceived: [u8; MAX_MSGLEN] = [0; MAX_MSGLEN];

/*
================
Sys_QueEvent

A time of 0 will get the current time
Ptr should either be null, or point to a block of data that can
be freed by the game later.
================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_QueEvent(time: c_int, event_type: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void) {
    unsafe {
        let idx = (eventHead as usize) & MASK_QUED_EVENTS;
        let ev = addr_of_mut!(eventQue[idx]);
        eventHead += 1;

        let time_val = if time == 0 {
            Sys_Milliseconds()
        } else {
            time
        };

        (*ev).evTime = time_val;
        (*ev).evType = event_type;
        (*ev).evValue = value;
        (*ev).evValue2 = value2;
        (*ev).evPtrLength = ptrLength;
        (*ev).evPtr = ptr;
    }
}

/*
================
Sys_GetEvent

================
*/
#[allow(non_snake_case)]
pub extern "C" fn Sys_GetEvent() -> sysEvent_t {
    unsafe {
        let mut ev: sysEvent_t = mem::zeroed();

        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) as usize) & MASK_QUED_EVENTS];
        }

        // pump the message loop
        // in vga this calls KBD_Update, under X, it calls GetEvent
        Sys_SendKeyEvents();

        // check for console commands
        let s = Sys_ConsoleInput();
        if !s.is_null() {
            let len = (libc::strlen(s) + 1) as c_int;
            let b = malloc(len as usize) as *mut c_char;
            libc::strcpy(b, s);
            Sys_QueEvent(0, 1, 0, 0, len, b as *mut c_void);  // SE_CONSOLE = 1
        }

        // check for other input devices
        IN_Frame();

        // check for network packets
        MSG_Init(addr_of_mut!(eventQue) as *mut c_void, addr_of_mut!(sys_packetReceived), MAX_MSGLEN as c_int);

        let mut adr: [u8; 20] = mem::zeroed();  // netadr_t placeholder
        if Sys_GetPacket(addr_of_mut!(adr) as *mut c_void, addr_of_mut!(sys_packetReceived) as *mut c_void) != 0 {
            let len_val = (mem::size_of_val(&adr) + MAX_MSGLEN) as c_int;
            let buf = malloc(len_val as usize) as *mut u8;
            memcpy(buf as *mut c_void, addr_of!(adr) as *const c_void, mem::size_of_val(&adr));
            memcpy(buf.add(mem::size_of_val(&adr)) as *mut c_void, sys_packetReceived.as_ptr() as *const c_void, MAX_MSGLEN);
            Sys_QueEvent(0, 2, 0, 0, len_val, buf as *mut c_void);  // SE_PACKET = 2
        }

        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) as usize) & MASK_QUED_EVENTS];
        }

        // create an empty event to return
        memset(addr_of_mut!(ev) as *mut c_void, 0, mem::size_of_val(&ev));
        ev.evTime = Sys_Milliseconds();

        ev
    }
}

/*****************************************************************************/

#[allow(non_snake_case)]
pub extern "C" fn Sys_AppActivate() {
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_GetClipboardData() -> *mut c_char {
    ptr::null_mut()
}

#[allow(non_snake_case)]
pub extern "C" fn Sys_Print(msg: *const c_char) {
    unsafe {
        libc::fputs(msg, libc::stderr);
    }
}

extern "C" {
    fn SetProgramPath(path: *const c_char);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    unsafe {
        let mut oldtime: c_int;
        let mut newtime: c_int;
        let mut len: c_int = 0;
        let mut i: c_int;

        // go back to real user for config loads
        saved_euid = geteuid();
        seteuid(getuid());

        SetProgramPath(*argv);

        // merge the command line, this is kinda silly
        len = 1;
        i = 1;
        while i < argc {
            len += libc::strlen(*argv.add(i as usize)) as c_int + 1;
            i += 1;
        }
        let cmdline = malloc(len as usize) as *mut c_char;
        *cmdline = 0;
        i = 1;
        while i < argc {
            if i > 1 {
                libc::strcat(cmdline, b" \0".as_ptr() as *const c_char);
            }
            libc::strcat(cmdline, *argv.add(i as usize));
            i += 1;
        }
        Com_Init(cmdline);
        NET_Init();

        fcntl(0, F_SETFL, fcntl(0, F_GETFL, 0) | FNDELAY);

        nostdout = Cvar_Get(b"nostdout\0".as_ptr() as *const c_char,
                           b"0\0".as_ptr() as *const c_char, 0);
        if !nostdout.is_null() {
            let nostdout_cvar = nostdout as *const c_int;
            if *nostdout_cvar == 0 {
                fcntl(0, F_SETFL, fcntl(0, F_GETFL, 0) | FNDELAY);
                //printf ("Linux Quake -- Version %0.3f\n", LINUX_VERSION);
            }
        }

        loop {
            // set low precision every frame, because some system calls
            // reset it arbitrarily
            Sys_LowFPPrecision();

            Com_Frame();
        }
    }
}

#[doc(hidden)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub extern "C" fn dlerror() -> *const c_char {
    // Stub - would normally call libc::dlerror()
    b"dlopen error\0".as_ptr() as *const c_char
}

// Stub for architecture-specific DLL names
#[allow(non_snake_case)]
fn _unused_code_example() {
    /*
    ================
    Sys_MakeCodeWriteable
    ================
    */
    // void Sys_MakeCodeWriteable (unsigned long startaddr, unsigned long length)
    // {
    //  int r;
    //  unsigned long addr;
    //  int psize = getpagesize();
    //
    //  addr = (startaddr & ~(psize-1)) - psize;
    //
    //  //fprintf(stderr, "writable code %lx(%lx)-%lx, length=%lx\n", startaddr,
    //  //		addr, startaddr+length, length);
    //
    //  r = mprotect((char*)addr, length + startaddr - addr + psize, 7);
    //
    //  if (r < 0)
    //    		Sys_Error("Protection change failed\n");
    // }
}
