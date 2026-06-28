// common.c -- misc functions used in client and server

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::mem;
use core::ptr::{addr_of, addr_of_mut, null_mut};
use std::ffi::{CStr, CString};

// Include-equivalent declarations
// #include "../game/q_shared.h"
// #include "qcommon.h"
// #include "../qcommon/sstring.h"
// #include "stv_version.h"

const MAXPRINTMSG: usize = 4096;
const MAX_NUM_ARGVS: usize = 50;

pub static mut com_argc: c_int = 0;
pub static mut com_argv: [*mut c_char; MAX_NUM_ARGVS + 1] = [null_mut(); MAX_NUM_ARGVS + 1];

#[cfg(not(target_os = "windows"))]
mod file_handles {
    use core::ffi::c_void;
    use core::ptr::null_mut;

    // Platform-specific file handle type (stub)
    pub type fileHandle_t = *mut c_void;

    pub static mut logfile: fileHandle_t = null_mut();
    pub static mut speedslog: fileHandle_t = null_mut();
    pub static mut camerafile: fileHandle_t = null_mut();
}

#[cfg(not(target_os = "windows"))]
pub use file_handles::*;

pub static mut com_journalFile: *mut c_void = null_mut();
pub static mut com_journalDataFile: *mut c_void = null_mut();

// cvar_t pointer declarations (stubs)
#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub string: *const c_char,
    pub resetString: *const c_char,
    pub latched_string: *const c_char,
    pub flags: c_int,
    pub modified: bool,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
}

pub static mut com_viewlog: *mut cvar_t = null_mut();
pub static mut com_speeds: *mut cvar_t = null_mut();
pub static mut com_developer: *mut cvar_t = null_mut();
pub static mut com_timescale: *mut cvar_t = null_mut();
pub static mut com_fixedtime: *mut cvar_t = null_mut();
pub static mut com_maxfps: *mut cvar_t = null_mut();
pub static mut com_sv_running: *mut cvar_t = null_mut();
pub static mut com_cl_running: *mut cvar_t = null_mut();
pub static mut com_logfile: *mut cvar_t = null_mut();
pub static mut com_showtrace: *mut cvar_t = null_mut();
pub static mut com_terrainPhysics: *mut cvar_t = null_mut();
pub static mut com_version: *mut cvar_t = null_mut();
pub static mut com_buildScript: *mut cvar_t = null_mut();
pub static mut cl_paused: *mut cvar_t = null_mut();
pub static mut sv_paused: *mut cvar_t = null_mut();
pub static mut com_skippingcin: *mut cvar_t = null_mut();
pub static mut com_speedslog: *mut cvar_t = null_mut();

#[cfg(feature = "G2_PERFORMANCE_ANALYSIS")]
pub static mut com_G2Report: *mut cvar_t = null_mut();

// com_speeds times
pub static mut time_game: c_int = 0;
pub static mut time_frontend: c_int = 0;
pub static mut time_backend: c_int = 0;

pub static mut timeInTrace: c_int = 0;
pub static mut timeInPVSCheck: c_int = 0;
pub static mut numTraces: c_int = 0;

pub static mut com_frameTime: c_int = 0;
pub static mut com_frameMsec: c_int = 0;
pub static mut com_frameNumber: c_int = 0;

pub static mut com_errorEntered: bool = false;
pub static mut com_fullyInitialized: bool = false;

pub static mut com_errorMessage: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];

extern "C" {
    fn Com_WriteConfig_f();
}

//============================================================================

#[cfg(not(target_os = "windows"))]
mod redirect {
    use core::ffi::c_char;
    use core::ptr::null_mut;

    pub static mut rd_buffer: *mut c_char = null_mut();
    pub static mut rd_buffersize: i32 = 0;
    pub static mut rd_flush: Option<extern "C" fn(*mut c_char)> = None;
}

#[cfg(not(target_os = "windows"))]
use redirect::*;

pub fn Com_BeginRedirect(buffer: *mut c_char, buffersize: i32, flush: Option<extern "C" fn(*mut c_char)>) {
    #[cfg(not(target_os = "windows"))]
    {
        if buffer.is_null() || buffersize == 0 || flush.is_none() {
            return;
        }
        unsafe {
            rd_buffer = buffer;
            rd_buffersize = buffersize;
            rd_flush = flush;
            *buffer = 0;
        }
    }
}

pub fn Com_EndRedirect() {
    #[cfg(not(target_os = "windows"))]
    {
        unsafe {
            if let Some(flush_fn) = rd_flush {
                flush_fn(rd_buffer);
            }
            rd_buffer = null_mut();
            rd_buffersize = 0;
            rd_flush = None;
        }
    }
}

extern "C" {
    fn CL_ConsolePrint(msg: *const c_char);
    fn Sys_Print(msg: *const c_char);
    fn FS_FOpenFileWrite(filename: *const c_char) -> *mut c_void;
    fn FS_Write(data: *const c_void, len: usize, handle: *mut c_void) -> usize;
    fn FS_ForceFlush(handle: *mut c_void);
    fn FS_FCloseFile(handle: *mut c_void);
}

/*
=============
Com_Printf

Both client and server can use this, and it will output
to the apropriate place.

A raw string should NEVER be passed as fmt, because of "%f" type crashers.
=============
*/
#[no_mangle]
pub extern "C" fn Com_Printf(fmt: *const c_char, mut args: ...) {
    let mut msg: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];

    unsafe {
        // This is a simplified version - in Rust we cannot directly use varargs
        // This would require a more sophisticated approach with libc::va_list
        // For now, we preserve the function signature for C ABI compatibility

        #[cfg(not(target_os = "windows"))]
        {
            let rd_buf = addr_of_mut!(rd_buffer);
            if !(*rd_buf).is_null() {
                // Note: In a real implementation, string concatenation would happen here
                // but Rust varargs handling is complex; this is a structural placeholder
                return;
            }
        }

        CL_ConsolePrint(msg.as_ptr());
        Sys_Print(msg.as_ptr());

        #[cfg(not(target_os = "windows"))]
        {
            let com_logfile_ptr = addr_of_mut!(com_logfile);
            if !(*com_logfile_ptr).is_null() && (*(*com_logfile_ptr)).integer > 0 {
                let logfile_ptr = addr_of_mut!(logfile);
                if (*logfile_ptr).is_null() {
                    *logfile_ptr = FS_FOpenFileWrite(b"qconsole.log\0".as_ptr() as *const c_char);
                    if (*(*com_logfile_ptr)).integer > 1 {
                        FS_ForceFlush(*logfile_ptr);
                    }
                }
                if !(*logfile_ptr).is_null() {
                    FS_Write(msg.as_ptr() as *const c_void, msg.len(), *logfile_ptr);
                }
            }
        }
    }
}

/*
================
Com_DPrintf

A Com_Printf that only shows up if the "developer" cvar is set
================
*/
#[no_mangle]
pub extern "C" fn Com_DPrintf(fmt: *const c_char, mut args: ...) {
    let mut msg: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];

    unsafe {
        let com_developer_ptr = addr_of_mut!(com_developer);
        if (*com_developer_ptr).is_null() || (*(*com_developer_ptr)).integer == 0 {
            return;
        }

        // In a real implementation, vsprintf would be called here
        Com_Printf(b"%s\0".as_ptr() as *const c_char, msg.as_ptr());
    }
}

extern "C" {
    fn FS_Printf(handle: *mut c_void, fmt: *const c_char, ...);
    static mut sv_mapname: *mut cvar_t;
}

pub fn Com_WriteCam(text: *const c_char) {
    #[cfg(not(target_os = "windows"))]
    {
        let mut mapname: [c_char; 260] = [0; 260]; // MAX_QPATH

        unsafe {
            let camerafile_ptr = addr_of_mut!(camerafile);
            if (*camerafile_ptr).is_null() {
                // sprintf(mapname, "maps/%s_cam.map", sv_mapname->string);
                // This is a placeholder for the string formatting
                *camerafile_ptr = FS_FOpenFileWrite(mapname.as_ptr());
            }

            if !(*camerafile_ptr).is_null() {
                FS_Printf(*camerafile_ptr, b"%s\0".as_ptr() as *const c_char, text);
            }

            Com_Printf(b"%s\n\0".as_ptr() as *const c_char, mapname.as_ptr());
        }
    }
}

pub fn Com_FlushCamFile() {
    #[cfg(not(target_os = "windows"))]
    {
        unsafe {
            let camerafile_ptr = addr_of_mut!(camerafile);
            if (*camerafile_ptr).is_null() {
                Com_Printf(b"No cam file available\n\0".as_ptr() as *const c_char);
                return;
            }
            FS_ForceFlush(*camerafile_ptr);
            FS_FCloseFile(*camerafile_ptr);
            *camerafile_ptr = null_mut();

            let mut flushedMapname: [c_char; 260] = [0; 260];
            // sprintf(flushedMapname, "maps/%s_cam.map", sv_mapname->string);
            Com_Printf(b"flushed all cams to %s\n\0".as_ptr() as *const c_char, flushedMapname.as_ptr());
        }
    }
}

/*
=============
Com_Error

Both client and server can use this, and it will
do the apropriate things.
=============
*/

extern "C" {
    fn SG_WipeSavegame(name: *const c_char);
    fn SG_Shutdown();
    fn Sys_Error(fmt: *const c_char, ...);
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_Set(name: *const c_char, value: *const c_char);
    fn SV_Shutdown(message: *const c_char);
    fn CL_Disconnect();
    fn CL_FlushMemory();
    fn CL_StartHunkUsers();
    fn CL_Shutdown();
    fn Com_Shutdown();
}

#[no_mangle]
pub extern "C" fn Com_Error(mut code: c_int, fmt: *const c_char, mut args: ...) {
    unsafe {
        #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
        {
            // Windows-specific debug trap (inline assembly stub)
            // if (code != ERR_DISCONNECT && code != ERR_NEED_CD) { __asm { int 0x03 } }
        }

        let com_buildScript_ptr = addr_of_mut!(com_buildScript);
        if !(*com_buildScript_ptr).is_null() && (*(*com_buildScript_ptr)).integer != 0 {
            code = 1; // ERR_FATAL
        }

        let com_errorEntered_ptr = addr_of_mut!(com_errorEntered);
        if *com_errorEntered_ptr {
            Sys_Error(b"recursive error after: %s\0".as_ptr() as *const c_char, addr_of!(com_errorMessage));
        }

        *com_errorEntered_ptr = true;

        // vsprintf(com_errorMessage, fmt, argptr);

        let error_code = code;
        if error_code != 0 { // ERR_DISCONNECT
            Cvar_Get(b"com_errorMessage\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char, 1); // CVAR_ROM
            Cvar_Set(b"com_errorMessage\0".as_ptr() as *const c_char, addr_of!(com_errorMessage));
        }

        SG_Shutdown();

        if error_code == 0 { // ERR_DISCONNECT
            CL_Disconnect();
            CL_FlushMemory();
            CL_StartHunkUsers();
            *com_errorEntered_ptr = false;
            // throw ("DISCONNECTED\n");
        } else if error_code == 2 { // ERR_DROP
            SG_WipeSavegame(b"current\0".as_ptr() as *const c_char);
            SV_Shutdown(b"Server crashed: %s\n\0".as_ptr() as *const c_char);
            CL_Disconnect();
            CL_FlushMemory();
            CL_StartHunkUsers();
            Com_Printf(b"********************\nERROR: %s\n********************\n\0".as_ptr() as *const c_char, addr_of!(com_errorMessage));
            *com_errorEntered_ptr = false;
            // throw ("DROPPED\n");
        } else if error_code == 3 { // ERR_NEED_CD
            SV_Shutdown(b"Server didn't have CD\n\0".as_ptr() as *const c_char);
            let com_cl_running_ptr = addr_of_mut!(com_cl_running);
            if !(*com_cl_running_ptr).is_null() && (*(*com_cl_running_ptr)).integer != 0 {
                CL_Disconnect();
                CL_FlushMemory();
                CL_StartHunkUsers();
                *com_errorEntered_ptr = false;
            } else {
                Com_Printf(b"Server didn't have CD\n\0".as_ptr() as *const c_char);
            }
            // throw ("NEED CD\n");
        } else {
            CL_Shutdown();
            SV_Shutdown(b"Server fatal crashed: %s\n\0".as_ptr() as *const c_char);
        }

        Com_Shutdown();
        Sys_Error(b"%s\0".as_ptr() as *const c_char, addr_of!(com_errorMessage));
    }
}

/*
=============
Com_Quit_f

Both client and server can use this, and it will
do the apropriate things.
=============
*/
#[no_mangle]
pub extern "C" fn Com_Quit_f() {
    unsafe {
        let com_errorEntered_ptr = addr_of_mut!(com_errorEntered);
        if !*com_errorEntered_ptr {
            SV_Shutdown(b"Server quit\n\0".as_ptr() as *const c_char);
            CL_Shutdown();
            Com_Shutdown();
        }
    }
    Sys_Quit();
}

extern "C" {
    fn Sys_Quit();
}

/*
============================================================================

COMMAND LINE FUNCTIONS

+ characters seperate the commandLine string into multiple console
command lines.

All of these are valid:

quake3 +set test blah +map test
quake3 set test blah+map test
quake3 set test blah + map test

============================================================================
*/

const MAX_CONSOLE_LINES: usize = 32;
pub static mut com_numConsoleLines: c_int = 0;
pub static mut com_consoleLines: [*mut c_char; MAX_CONSOLE_LINES] = [null_mut(); MAX_CONSOLE_LINES];

/*
==================
Com_ParseCommandLine

Break it up into multiple console lines
==================
*/
#[no_mangle]
pub extern "C" fn Com_ParseCommandLine(commandLine: *mut c_char) {
    unsafe {
        let com_consoleLines_ptr = addr_of_mut!(com_consoleLines);
        let com_numConsoleLines_ptr = addr_of_mut!(com_numConsoleLines);

        (*com_consoleLines_ptr)[0] = commandLine;
        *com_numConsoleLines_ptr = 1;

        let mut cmd_ptr = commandLine;
        loop {
            let ch = *cmd_ptr;
            if ch == 0 {
                break;
            }

            if ch == '+' as c_char || ch == '\n' as c_char {
                if *com_numConsoleLines_ptr as usize == MAX_CONSOLE_LINES {
                    return;
                }
                (*com_consoleLines_ptr)[*com_numConsoleLines_ptr as usize] = cmd_ptr.offset(1);
                *com_numConsoleLines_ptr += 1;
                *cmd_ptr = 0;
            }
            cmd_ptr = cmd_ptr.offset(1);
        }
    }
}

extern "C" {
    fn Cmd_TokenizeString(str_: *const c_char);
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

/*
===================
Com_SafeMode

Check for "safe" on the command line, which will
skip loading of jaconfig.cfg
===================
*/
#[no_mangle]
pub extern "C" fn Com_SafeMode() -> bool {
    unsafe {
        let com_numConsoleLines_ptr = addr_of_mut!(com_numConsoleLines);
        let com_consoleLines_ptr = addr_of_mut!(com_consoleLines);

        for i in 0..*com_numConsoleLines_ptr as usize {
            Cmd_TokenizeString((*com_consoleLines_ptr)[i] as *const c_char);
            if Q_stricmp(Cmd_Argv(0), b"safe\0".as_ptr() as *const c_char) == 0 ||
               Q_stricmp(Cmd_Argv(0), b"cvar_restart\0".as_ptr() as *const c_char) == 0 {
                (*com_consoleLines_ptr)[i] = 0 as *mut c_char;
                return true;
            }
        }
        false
    }
}

extern "C" {
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

/*
===============
Com_StartupVariable

Searches for command line parameters that are set commands.
If match is not NULL, only that cvar will be looked for.
That is necessary because cddir and basedir need to be set
before the filesystem is started, but all other sets should
be after execing the config and default.
===============
*/
#[no_mangle]
pub extern "C" fn Com_StartupVariable(match_: *const c_char) {
    unsafe {
        let com_numConsoleLines_ptr = addr_of_mut!(com_numConsoleLines);
        let com_consoleLines_ptr = addr_of_mut!(com_consoleLines);

        for i in 0..*com_numConsoleLines_ptr as usize {
            Cmd_TokenizeString((*com_consoleLines_ptr)[i] as *const c_char);
            if libc::strcmp(Cmd_Argv(0), b"set\0".as_ptr() as *const c_char) != 0 {
                continue;
            }

            let s = Cmd_Argv(1);
            if match_.is_null() || stricmp(s, match_) == 0 {
                Cvar_Set(s, Cmd_Argv(2));
                let cv = Cvar_Get(s, b"\0".as_ptr() as *const c_char, 0);
                if !cv.is_null() {
                    (*cv).flags |= 16; // CVAR_USER_CREATED
                }
            }
        }
    }
}

extern "C" {
    fn Cbuf_AddText(text: *const c_char);
}

/*
=================
Com_AddStartupCommands

Adds command line parameters as script statements
Commands are seperated by + signs

Returns qtrue if any late commands were added, which
will keep the demoloop from immediately starting
=================
*/
#[no_mangle]
pub extern "C" fn Com_AddStartupCommands() -> bool {
    unsafe {
        let com_numConsoleLines_ptr = addr_of_mut!(com_numConsoleLines);
        let com_consoleLines_ptr = addr_of_mut!(com_consoleLines);

        let mut added = false;
        for i in 0..*com_numConsoleLines_ptr as usize {
            if (*com_consoleLines_ptr)[i].is_null() || *(*com_consoleLines_ptr)[i] == 0 {
                continue;
            }

            if Q_stricmp((*com_consoleLines_ptr)[i] as *const c_char, b"set\0".as_ptr() as *const c_char) != 0 {
                added = true;
            }
            Cbuf_AddText((*com_consoleLines_ptr)[i] as *const c_char);
            Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
        }

        added
    }
}

//============================================================================

#[no_mangle]
pub extern "C" fn Info_Print(s: *const c_char) {
    unsafe {
        let mut key: [c_char; 512] = [0; 512];
        let mut value: [c_char; 512] = [0; 512];
        let mut s_ptr = s;

        if *s_ptr == '\\' as c_char {
            s_ptr = s_ptr.offset(1);
        }

        loop {
            if *s_ptr == 0 {
                break;
            }

            let mut o = key.as_mut_ptr();
            loop {
                if *s_ptr == 0 || *s_ptr == '\\' as c_char {
                    break;
                }
                *o = *s_ptr;
                o = o.offset(1);
                s_ptr = s_ptr.offset(1);
            }

            let l = o as usize - key.as_ptr() as usize;
            if l < 20 {
                libc::memset(o as *mut c_void, ' ' as i32, 20 - l);
                key[20] = 0;
            } else {
                *o = 0;
            }
            Com_Printf(b"%s\0".as_ptr() as *const c_char, key.as_ptr());

            if *s_ptr == 0 {
                Com_Printf(b"MISSING VALUE\n\0".as_ptr() as *const c_char);
                return;
            }

            o = value.as_mut_ptr();
            s_ptr = s_ptr.offset(1);
            loop {
                if *s_ptr == 0 || *s_ptr == '\\' as c_char {
                    break;
                }
                *o = *s_ptr;
                o = o.offset(1);
                s_ptr = s_ptr.offset(1);
            }
            *o = 0;

            if *s_ptr != 0 {
                s_ptr = s_ptr.offset(1);
            }
            Com_Printf(b"%s\n\0".as_ptr() as *const c_char, value.as_ptr());
        }
    }
}

/*
============
Com_StringContains
============
*/
#[no_mangle]
pub extern "C" fn Com_StringContains(str1: *mut c_char, str2: *mut c_char, casesensitive: c_int) -> *mut c_char {
    unsafe {
        let len1 = libc::strlen(str1);
        let len2 = libc::strlen(str2);
        let len = if len1 > len2 { len1 - len2 } else { 0 };

        let mut str1_ptr = str1;
        for i in 0..=len {
            let mut j = 0;
            loop {
                if *str2.offset(j as isize) == 0 {
                    break;
                }
                if casesensitive != 0 {
                    if *str1_ptr.offset(j as isize) != *str2.offset(j as isize) {
                        break;
                    }
                } else {
                    if libc::toupper(*str1_ptr.offset(j as isize) as u32) as c_char !=
                       libc::toupper(*str2.offset(j as isize) as u32) as c_char {
                        break;
                    }
                }
                j += 1;
            }
            if *str2.offset(j as isize) == 0 {
                return str1_ptr;
            }
            str1_ptr = str1_ptr.offset(1);
        }
        null_mut()
    }
}

/*
============
Com_Filter
============
*/
#[no_mangle]
pub extern "C" fn Com_Filter(filter: *mut c_char, name: *mut c_char, casesensitive: c_int) -> c_int {
    unsafe {
        let mut buf: [c_char; 256] = [0; 256]; // MAX_TOKEN_CHARS
        let mut filter_ptr = filter;
        let mut name_ptr = name;

        loop {
            if *filter_ptr == 0 {
                break;
            }

            if *filter_ptr == '*' as c_char {
                filter_ptr = filter_ptr.offset(1);
                let mut i = 0;
                loop {
                    if *filter_ptr == 0 {
                        break;
                    }
                    if *filter_ptr == '*' as c_char || *filter_ptr == '?' as c_char {
                        break;
                    }
                    buf[i] = *filter_ptr;
                    filter_ptr = filter_ptr.offset(1);
                    i += 1;
                }
                buf[i] = 0;
                if i > 0 {
                    let ptr = Com_StringContains(name_ptr, buf.as_mut_ptr(), casesensitive);
                    if ptr.is_null() {
                        return 0; // qfalse
                    }
                    name_ptr = ptr.offset(libc::strlen(buf.as_ptr()) as isize);
                }
            } else if *filter_ptr == '?' as c_char {
                filter_ptr = filter_ptr.offset(1);
                name_ptr = name_ptr.offset(1);
            } else {
                if casesensitive != 0 {
                    if *filter_ptr != *name_ptr {
                        return 0; // qfalse
                    }
                } else {
                    if libc::toupper(*filter_ptr as u32) as c_char !=
                       libc::toupper(*name_ptr as u32) as c_char {
                        return 0; // qfalse
                    }
                }
                filter_ptr = filter_ptr.offset(1);
                name_ptr = name_ptr.offset(1);
            }
        }
        1 // qtrue
    }
}

extern "C" {
    fn Hunk_Clear();
}

/*
=================
Com_InitHunkMemory
=================
*/
#[no_mangle]
pub extern "C" fn Com_InitHunkMemory() {
    Hunk_Clear();
}

// I'm leaving this in just in case we ever need to remember where's a good place to hook something like this in.
//
#[no_mangle]
pub extern "C" fn Com_ShutdownHunkMemory() {
}

extern "C" {
    fn Z_TagFree(tag: c_int);
}

/*
===================
Hunk_SetMark

The server calls this after the level and game VM have been loaded
===================
*/
#[no_mangle]
pub extern "C" fn Hunk_SetMark() {
}

/*
=================
Hunk_ClearToMark

The client calls this before starting a vid_restart or snd_restart
=================
*/
#[no_mangle]
pub extern "C" fn Hunk_ClearToMark() {
    Z_TagFree(1); // TAG_HUNKALLOC
    Z_TagFree(2); // TAG_HUNKMISCMODELS
}

/*
=================
Hunk_Clear

The server calls this before shutting down or loading a new map
=================
*/
#[no_mangle]
pub extern "C" fn Hunk_Clear() {
    Z_TagFree(1); // TAG_HUNKALLOC
    Z_TagFree(2); // TAG_HUNKMISCMODELS

    extern "C" {
        fn CIN_CloseAllVideos();
        fn R_ClearStuffToStopGhoul2CrashingThings();
    }

    CIN_CloseAllVideos();
    R_ClearStuffToStopGhoul2CrashingThings();
}

/*
===================================================================

EVENTS AND JOURNALING

In addition to these events, .cfg files are also copied to the
journaled file
===================================================================
*/

const MAX_PUSHED_EVENTS: usize = 64;

#[repr(C)]
pub struct sysEvent_t {
    pub evTime: c_int,
    pub evType: c_int,
    pub evValue: c_int,
    pub evValue2: c_int,
    pub evPtr: *mut c_void,
    pub evPtrLength: c_int,
}

pub static mut com_pushedEventsHead: c_int = 0;
pub static mut com_pushedEventsTail: c_int = 0;
pub static mut com_pushedEvents: [sysEvent_t; MAX_PUSHED_EVENTS] = unsafe { mem::zeroed() };

/*
=================
Com_GetRealEvent
=================
*/
#[no_mangle]
pub extern "C" fn Com_GetRealEvent() -> sysEvent_t {
    extern "C" {
        fn Sys_GetEvent() -> sysEvent_t;
    }
    unsafe { Sys_GetEvent() }
}

/*
=================
Com_PushEvent
=================
*/
#[no_mangle]
pub extern "C" fn Com_PushEvent(event: *const sysEvent_t) {
    unsafe {
        static mut printedWarning: c_int = 0;
        let com_pushedEventsHead_ptr = addr_of_mut!(com_pushedEventsHead);
        let com_pushedEventsTail_ptr = addr_of_mut!(com_pushedEventsTail);
        let com_pushedEvents_ptr = addr_of_mut!(com_pushedEvents);

        let ev = &mut (*com_pushedEvents_ptr)[((*com_pushedEventsHead_ptr) & (MAX_PUSHED_EVENTS as c_int - 1)) as usize];

        if *com_pushedEventsHead_ptr - *com_pushedEventsTail_ptr >= MAX_PUSHED_EVENTS as c_int {
            if printedWarning == 0 {
                printedWarning = 1;
                Com_Printf(b"WARNING: Com_PushEvent overflow\n\0".as_ptr() as *const c_char);
            }

            extern "C" {
                fn Z_Free(ptr: *mut c_void);
            }
            if !ev.evPtr.is_null() {
                Z_Free(ev.evPtr);
            }
            *com_pushedEventsTail_ptr += 1;
        } else {
            printedWarning = 0;
        }

        *ev = *event;
        *com_pushedEventsHead_ptr += 1;
    }
}

/*
=================
Com_GetEvent
=================
*/
#[no_mangle]
pub extern "C" fn Com_GetEvent() -> sysEvent_t {
    unsafe {
        let com_pushedEventsHead_ptr = addr_of_mut!(com_pushedEventsHead);
        let com_pushedEventsTail_ptr = addr_of_mut!(com_pushedEventsTail);
        let com_pushedEvents_ptr = addr_of_mut!(com_pushedEvents);

        if *com_pushedEventsHead_ptr > *com_pushedEventsTail_ptr {
            *com_pushedEventsTail_ptr += 1;
            return (*com_pushedEvents_ptr)[(((*com_pushedEventsTail_ptr) - 1) & (MAX_PUSHED_EVENTS as c_int - 1)) as usize];
        }
        Com_GetRealEvent()
    }
}

#[repr(C)]
pub struct netadr_t {
    pub type_: c_int,
    pub ip: [u8; 4],
    pub port: u16,
}

#[repr(C)]
pub struct msg_t {
    pub allowoverflow: bool,
    pub overflowed: bool,
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
}

extern "C" {
    fn MSG_Init(buf: *mut msg_t, data: *mut u8, len: usize);
    fn SV_PacketEvent(from: netadr_t, buf: *mut msg_t);
}

/*
=================
Com_RunAndTimeServerPacket
=================
*/
#[no_mangle]
pub extern "C" fn Com_RunAndTimeServerPacket(evFrom: *mut netadr_t, buf: *mut msg_t) {
    unsafe {
        let mut t1: c_int = 0;
        let mut t2: c_int = 0;
        let mut msec: c_int = 0;

        let com_speeds_ptr = addr_of_mut!(com_speeds);
        if !(*com_speeds_ptr).is_null() && (*(*com_speeds_ptr)).integer != 0 {
            extern "C" {
                fn Sys_Milliseconds() -> c_int;
            }
            t1 = Sys_Milliseconds();
        }

        SV_PacketEvent(*evFrom, buf);

        if !(*com_speeds_ptr).is_null() && (*(*com_speeds_ptr)).integer != 0 {
            extern "C" {
                fn Sys_Milliseconds() -> c_int;
            }
            t2 = Sys_Milliseconds();
            msec = t2 - t1;
            if (*(*com_speeds_ptr)).integer == 3 {
                Com_Printf(b"SV_PacketEvent time: %i\n\0".as_ptr() as *const c_char, msec);
            }
        }
    }
}

const SE_NONE: c_int = 0;
const SE_KEY: c_int = 1;
const SE_CHAR: c_int = 2;
const SE_MOUSE: c_int = 3;
const SE_JOYSTICK_AXIS: c_int = 4;
const SE_CONSOLE: c_int = 5;
const SE_PACKET: c_int = 6;
const MAX_MSGLEN: usize = 16384;

extern "C" {
    fn NET_GetLoopPacket(sock: c_int, from: *mut netadr_t, buf: *mut msg_t) -> bool;
    fn CL_PacketEvent(from: netadr_t, buf: *mut msg_t);
    fn CL_KeyEvent(key: c_int, down: c_int, time: c_int);
    fn CL_CharEvent(key: c_int);
    fn CL_MouseEvent(dx: c_int, dy: c_int, time: c_int);
    fn CL_JoystickEvent(axis: c_int, value: c_int, time: c_int);
}

const NS_CLIENT: c_int = 0;
const NS_SERVER: c_int = 1;

/*
=================
Com_EventLoop

Returns last event time
=================
*/
#[no_mangle]
pub extern "C" fn Com_EventLoop() -> c_int {
    unsafe {
        let mut ev: sysEvent_t = mem::zeroed();
        let mut evFrom: netadr_t = mem::zeroed();
        let mut bufData: [u8; MAX_MSGLEN] = [0; MAX_MSGLEN];
        let mut buf: msg_t = mem::zeroed();

        MSG_Init(&mut buf, bufData.as_mut_ptr(), bufData.len());

        loop {
            ev = Com_GetEvent();

            if ev.evType == SE_NONE {
                while NET_GetLoopPacket(NS_CLIENT, &mut evFrom, &mut buf) {
                    CL_PacketEvent(evFrom, &mut buf);
                }

                let com_sv_running_ptr = addr_of_mut!(com_sv_running);
                while NET_GetLoopPacket(NS_SERVER, &mut evFrom, &mut buf) {
                    if !(*com_sv_running_ptr).is_null() && (*(*com_sv_running_ptr)).integer != 0 {
                        Com_RunAndTimeServerPacket(&mut evFrom, &mut buf);
                    }
                }

                return ev.evTime;
            }

            match ev.evType {
                SE_NONE => {},
                SE_KEY => CL_KeyEvent(ev.evValue, ev.evValue2, ev.evTime),
                SE_CHAR => CL_CharEvent(ev.evValue),
                SE_MOUSE => CL_MouseEvent(ev.evValue, ev.evValue2, ev.evTime),
                SE_JOYSTICK_AXIS => CL_JoystickEvent(ev.evValue, ev.evValue2, ev.evTime),
                SE_CONSOLE => {
                    Cbuf_AddText(ev.evPtr as *const c_char);
                    Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
                },
                SE_PACKET => {
                    evFrom = *(ev.evPtr as *const netadr_t);
                    buf.cursize = ev.evPtrLength - mem::size_of::<netadr_t>() as c_int;

                    if (buf.cursize as usize) > buf.maxsize as usize {
                        Com_Printf(b"Com_EventLoop: oversize packet\n\0".as_ptr() as *const c_char);
                        continue;
                    }
                    libc::memcpy(buf.data as *mut c_void, (ev.evPtr as *const netadr_t).offset(1) as *const c_void, buf.cursize as usize);

                    let com_sv_running_ptr = addr_of_mut!(com_sv_running);
                    if !(*com_sv_running_ptr).is_null() && (*(*com_sv_running_ptr)).integer != 0 {
                        Com_RunAndTimeServerPacket(&mut evFrom, &mut buf);
                    } else {
                        CL_PacketEvent(evFrom, &mut buf);
                    }
                },
                _ => Com_Error(1, b"Com_EventLoop: bad event type %i\0".as_ptr() as *const c_char, ev.evTime),
            }

            extern "C" {
                fn Z_Free(ptr: *mut c_void);
            }
            if !ev.evPtr.is_null() {
                Z_Free(ev.evPtr);
            }
        }
    }
}

/*
================
Com_Milliseconds

Can be used for profiling, but will be journaled accurately
================
*/
#[no_mangle]
pub extern "C" fn Com_Milliseconds() -> c_int {
    unsafe {
        let mut ev: sysEvent_t;

        loop {
            ev = Com_GetRealEvent();
            if ev.evType != SE_NONE {
                Com_PushEvent(&ev);
            }
            if ev.evType == SE_NONE {
                break;
            }
        }

        ev.evTime
    }
}

//============================================================================

/*
=============
Com_Error_f

Just throw a fatal error to
test error shutdown procedures
=============
*/
extern "C" {
    fn Cmd_Argc() -> c_int;
}

#[no_mangle]
pub extern "C" fn Com_Error_f() {
    if Cmd_Argc() > 1 {
        Com_Error(2, b"Testing drop error\0".as_ptr() as *const c_char);
    } else {
        Com_Error(1, b"Testing fatal error\0".as_ptr() as *const c_char);
    }
}

/*
=============
Com_Freeze_f

Just freeze in place for a given number of seconds to test
error recovery
=============
*/
#[no_mangle]
pub extern "C" fn Com_Freeze_f() {
    extern "C" {
        fn atof(str_: *const c_char) -> f64;
        fn Sys_Milliseconds() -> c_int;
    }

    if Cmd_Argc() != 2 {
        Com_Printf(b"freeze <seconds>\n\0".as_ptr() as *const c_char);
        return;
    }

    let s: f64 = unsafe { atof(Cmd_Argv(1)) };
    let start = Com_Milliseconds();

    loop {
        let now = Com_Milliseconds();
        if (now - start) as f64 * 0.001 > s {
            break;
        }
    }
}

/*
=================
Com_Crash_f

A way to force a bus error for development reasons
=================
*/
#[no_mangle]
pub extern "C" fn Com_Crash_f() {
    unsafe {
        *(null_mut::<*mut c_int>()) = 0x12345678 as *mut c_int;
    }
}

/*
=================
Com_Init
=================
*/
extern "C" {
    fn Com_InitZoneMemory();
    fn R_InitWorldEffects();
    fn Swap_Init();
    fn Cbuf_Init();
    fn Cmd_Init();
    fn Cvar_Init();
    fn CL_InitKeyCommands();
    fn FS_InitFilesystem();
    fn Cbuf_Execute();
    fn Cmd_AddCommand(name: *const c_char, func: extern "C" fn());
    fn SE_Init();
    fn Sys_Init();
    fn Netchan_Init(time: c_int);
    fn SV_Init();
    fn CL_Init();
    fn Sys_ShowConsole(show: c_int, exclusive: bool);
}

#[no_mangle]
pub extern "C" fn Com_Init(commandLine: *const c_char) {
    unsafe {
        Com_Printf(b"%s %s %s\n\0".as_ptr() as *const c_char,
                   b"Jedi Academy\0".as_ptr() as *const c_char,
                   b"CPU\0".as_ptr() as *const c_char,
                   env!("CARGO_PKG_VERSION"));

        Com_ParseCommandLine(commandLine as *mut c_char);

        Swap_Init();
        Cbuf_Init();
        Com_InitZoneMemory();

        Cmd_Init();
        Cvar_Init();

        Com_StartupVariable(null_mut());
        CL_InitKeyCommands();

        FS_InitFilesystem();
        R_InitWorldEffects();

        Cbuf_AddText(b"exec default.cfg\n\0".as_ptr() as *const c_char);

        if !Com_SafeMode() {
            Cbuf_AddText(b"exec jaconfig.cfg\n\0".as_ptr() as *const c_char);
        }

        Cbuf_AddText(b"exec autoexec.cfg\n\0".as_ptr() as *const c_char);
        Cbuf_Execute();

        Com_StartupVariable(null_mut());

        Com_InitHunkMemory();

        extern "C" {
            static mut cvar_modifiedFlags: c_int;
        }
        cvar_modifiedFlags &= !1; // CVAR_ARCHIVE

        Cmd_AddCommand(b"quit\0".as_ptr() as *const c_char, Com_Quit_f);
        Cmd_AddCommand(b"writeconfig\0".as_ptr() as *const c_char, Com_WriteConfig_f);

        com_maxfps = Cvar_Get(b"com_maxfps\0".as_ptr() as *const c_char, b"85\0".as_ptr() as *const c_char, 1);
        com_developer = Cvar_Get(b"developer\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 2);
        com_logfile = Cvar_Get(b"logfile\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 2);
        com_speedslog = Cvar_Get(b"speedslog\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 2);

        com_timescale = Cvar_Get(b"timescale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 4);
        com_fixedtime = Cvar_Get(b"fixedtime\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 4);
        com_showtrace = Cvar_Get(b"com_showtrace\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 4);
        com_terrainPhysics = Cvar_Get(b"com_terrainPhysics\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 4);
        com_viewlog = Cvar_Get(b"viewlog\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 2);
        com_speeds = Cvar_Get(b"com_speeds\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

        #[cfg(feature = "G2_PERFORMANCE_ANALYSIS")]
        {
            com_G2Report = Cvar_Get(b"com_G2Report\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        cl_paused = Cvar_Get(b"cl_paused\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 1);
        sv_paused = Cvar_Get(b"sv_paused\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 1);
        com_sv_running = Cvar_Get(b"sv_running\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 1);
        com_cl_running = Cvar_Get(b"cl_running\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 1);
        com_skippingcin = Cvar_Get(b"skippingCinematic\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 1);
        com_buildScript = Cvar_Get(b"com_buildScript\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

        let com_developer_ptr = addr_of_mut!(com_developer);
        if !(*com_developer_ptr).is_null() && (*(*com_developer_ptr)).integer != 0 {
            Cmd_AddCommand(b"error\0".as_ptr() as *const c_char, Com_Error_f);
            Cmd_AddCommand(b"crash\0".as_ptr() as *const c_char, Com_Crash_f);
            Cmd_AddCommand(b"freeze\0".as_ptr() as *const c_char, Com_Freeze_f);
        }

        com_version = Cvar_Get(b"version\0".as_ptr() as *const c_char,
                               b"Jedi Academy\0".as_ptr() as *const c_char, 5);

        SE_Init();
        Sys_Init();

        Netchan_Init(Com_Milliseconds() & 0xffff);
        SV_Init();
        CL_Init();

        Sys_ShowConsole(if let Some(cvlog) = unsafe { Some(&*com_viewlog) } { (*cvlog).integer } else { 0 }, false);

        let com_frameTime_ptr = addr_of_mut!(com_frameTime);
        *com_frameTime_ptr = Com_Milliseconds();

        if !Com_AddStartupCommands() {
            #[cfg(not(debug_assertions))]
            {
                Cbuf_AddText(b"cinematic openinglogos\n\0".as_ptr() as *const c_char);
            }
        }

        let com_fullyInitialized_ptr = addr_of_mut!(com_fullyInitialized);
        *com_fullyInitialized_ptr = true;
        Com_Printf(b"--- Common Initialization Complete ---\n\0".as_ptr() as *const c_char);
    }
}

//==================================================================

#[no_mangle]
pub extern "C" fn Com_WriteConfigToFile(filename: *const c_char) {
    #[cfg(not(target_os = "windows"))]
    {
        unsafe {
            let f = FS_FOpenFileWrite(filename);
            if f.is_null() {
                Com_Printf(b"Couldn't write %s.\n\0".as_ptr() as *const c_char, filename);
                return;
            }

            extern "C" {
                fn Key_WriteBindings(f: *mut c_void);
                fn Cvar_WriteVariables(f: *mut c_void);
            }

            FS_Printf(f, b"// generated by Star Wars Jedi Academy, do not modify\n\0".as_ptr() as *const c_char);
            Key_WriteBindings(f);
            Cvar_WriteVariables(f);
            FS_FCloseFile(f);
        }
    }
}

/*
===============
Com_WriteConfiguration

Writes key bindings and archived cvars to config file if modified
===============
*/
#[no_mangle]
pub extern "C" fn Com_WriteConfiguration() {
    unsafe {
        let com_fullyInitialized_ptr = addr_of_mut!(com_fullyInitialized);
        if !*com_fullyInitialized_ptr {
            return;
        }

        extern "C" {
            static mut cvar_modifiedFlags: c_int;
        }

        if (cvar_modifiedFlags & 1) == 0 { // CVAR_ARCHIVE
            return;
        }
        cvar_modifiedFlags &= !1;

        Com_WriteConfigToFile(b"jaconfig.cfg\0".as_ptr() as *const c_char);
    }
}

/*
===============
Com_WriteConfig_f

Write the config file to a specific name
===============
*/
#[no_mangle]
pub extern "C" fn Com_WriteConfig_f() {
    let mut filename: [c_char; 260] = [0; 260];

    if Cmd_Argc() != 2 {
        Com_Printf(b"Usage: writeconfig <filename>\n\0".as_ptr() as *const c_char);
        return;
    }

    extern "C" {
        fn Q_strncpyz(dst: *mut c_char, src: *const c_char, dstsize: usize);
        fn COM_DefaultExtension(path: *mut c_char, maxsize: usize, extension: *const c_char);
    }

    unsafe {
        Q_strncpyz(filename.as_mut_ptr(), Cmd_Argv(1), filename.len());
        COM_DefaultExtension(filename.as_mut_ptr(), filename.len(), b".cfg\0".as_ptr() as *const c_char);
        Com_Printf(b"Writing %s.\n\0".as_ptr() as *const c_char, filename.as_ptr());
        Com_WriteConfigToFile(filename.as_ptr());
    }
}

/*
================
Com_ModifyMsec
================
*/

#[no_mangle]
pub extern "C" fn Com_ModifyMsec(mut msec: c_int, fraction: *mut f32) -> c_int {
    unsafe {
        *fraction = 0.0;

        let com_fixedtime_ptr = addr_of_mut!(com_fixedtime);
        if !(*com_fixedtime_ptr).is_null() && (*(*com_fixedtime_ptr)).integer != 0 {
            msec = (*(*com_fixedtime_ptr)).integer;
        } else {
            let com_timescale_ptr = addr_of_mut!(com_timescale);
            if !(*com_timescale_ptr).is_null() && (*(*com_timescale_ptr)).value != 0.0 {
                *fraction = msec as f32;
                *fraction *= (*(*com_timescale_ptr)).value;
                msec = fraction.floor() as c_int;
                *fraction -= msec as f32;
            }
        }

        if msec < 1 {
            msec = 1;
            *fraction = 0.0;
        }

        let com_skippingcin_ptr = addr_of_mut!(com_skippingcin);
        let clampTime = if !(*com_skippingcin_ptr).is_null() && (*(*com_skippingcin_ptr)).integer != 0 {
            500
        } else {
            200
        };

        if msec > clampTime {
            msec = clampTime;
            *fraction = 0.0;
        }

        msec
    }
}

/*
=================
Com_Frame
=================
*/
static mut corg: [f32; 3] = [0.0; 3];
static mut cangles: [f32; 3] = [0.0; 3];
static mut bComma: bool = false;

#[no_mangle]
pub extern "C" fn Com_SetOrgAngles(org: *const f32, angles: *const f32) {
    unsafe {
        for i in 0..3 {
            corg[i] = *org.offset(i as isize);
            cangles[i] = *angles.offset(i as isize);
        }
    }
}

#[cfg(feature = "G2_PERFORMANCE_ANALYSIS")]
extern "C" {
    fn G2Time_ResetTimers();
    fn G2Time_ReportTimers();
}

#[no_mangle]
pub extern "C" fn Com_Frame() {
    unsafe {
        let mut timeBeforeFirstEvents: c_int = 0;
        let mut timeBeforeServer: c_int = 0;
        let mut timeBeforeEvents: c_int = 0;
        let mut timeBeforeClient: c_int = 0;
        let mut timeAfter: c_int = 0;
        let mut msec: c_int = 0;
        let mut minMsec: c_int = 0;
        static mut lastTime: c_int = 0;

        #[cfg(not(target_os = "windows"))]
        Com_WriteConfiguration();

        #[cfg(not(target_os = "windows"))]
        {
            let com_viewlog_ptr = addr_of_mut!(com_viewlog);
            if !(*com_viewlog_ptr).is_null() && (*(*com_viewlog_ptr)).modified {
                Sys_ShowConsole((*(*com_viewlog_ptr)).integer, false);
                (*(*com_viewlog_ptr)).modified = false;
            }
        }

        let com_speeds_ptr = addr_of_mut!(com_speeds);
        if !(*com_speeds_ptr).is_null() && (*(*com_speeds_ptr)).integer != 0 {
            extern "C" {
                fn Sys_Milliseconds() -> c_int;
            }
            timeBeforeFirstEvents = Sys_Milliseconds();
        }

        let com_maxfps_ptr = addr_of_mut!(com_maxfps);
        if !(*com_maxfps_ptr).is_null() && (*(*com_maxfps_ptr)).integer > 0 {
            minMsec = 1000 / (*(*com_maxfps_ptr)).integer;
        } else {
            minMsec = 1;
        }

        loop {
            let com_frameTime_ptr = addr_of_mut!(com_frameTime);
            *com_frameTime_ptr = Com_EventLoop();
            if lastTime > *com_frameTime_ptr {
                lastTime = *com_frameTime_ptr;
            }
            msec = *com_frameTime_ptr - lastTime;
            if msec >= minMsec {
                break;
            }
        }

        Cbuf_Execute();
        lastTime = Com_EventLoop();

        let com_frameMsec_ptr = addr_of_mut!(com_frameMsec);
        *com_frameMsec_ptr = msec;
        let mut fractionMsec: f32 = 0.0;
        msec = Com_ModifyMsec(msec, &mut fractionMsec);

        if !(*com_speeds_ptr).is_null() && (*(*com_speeds_ptr)).integer != 0 {
            extern "C" {
                fn Sys_Milliseconds() -> c_int;
            }
            timeBeforeServer = Sys_Milliseconds();
        }

        extern "C" {
            fn SV_Frame(msec: c_int, fractionMsec: f32);
            fn CL_Frame(msec: c_int, fractionMsec: f32);
        }

        SV_Frame(msec, fractionMsec);

        {
            if !(*com_speeds_ptr).is_null() && (*(*com_speeds_ptr)).integer != 0 {
                extern "C" {
                    fn Sys_Milliseconds() -> c_int;
                }
                timeBeforeEvents = Sys_Milliseconds();
            }
            Com_EventLoop();
            Cbuf_Execute();

            if !(*com_speeds_ptr).is_null() && (*(*com_speeds_ptr)).integer != 0 {
                extern "C" {
                    fn Sys_Milliseconds() -> c_int;
                }
                timeBeforeClient = Sys_Milliseconds();
            }

            CL_Frame(msec, fractionMsec);

            if !(*com_speeds_ptr).is_null() && (*(*com_speeds_ptr)).integer != 0 {
                extern "C" {
                    fn Sys_Milliseconds() -> c_int;
                }
                timeAfter = Sys_Milliseconds();
            }
        }

        if !(*com_speeds_ptr).is_null() && (*(*com_speeds_ptr)).integer != 0 {
            let all = timeAfter - timeBeforeServer;
            let sv = timeBeforeEvents - timeBeforeServer;
            let ev = timeBeforeServer - timeBeforeFirstEvents + timeBeforeClient - timeBeforeEvents;
            let cl = timeAfter - timeBeforeClient;

            let time_game_ptr = addr_of_mut!(time_game);
            let time_frontend_ptr = addr_of_mut!(time_frontend);
            let time_backend_ptr = addr_of_mut!(time_backend);
            let timeInTrace_ptr = addr_of_mut!(timeInTrace);
            let timeInPVSCheck_ptr = addr_of_mut!(timeInPVSCheck);
            let com_frameNumber_ptr = addr_of_mut!(com_frameNumber);

            let sv_adjusted = sv - *time_game_ptr;
            let cl_adjusted = cl - (*time_frontend_ptr + *time_backend_ptr);

            Com_Printf(b"fr:%i all:%3i sv:%3i ev:%3i cl:%3i gm:%3i tr:%3i pvs:%3i rf:%3i bk:%3i\n\0".as_ptr() as *const c_char,
                      *com_frameNumber_ptr, all, sv_adjusted, ev, cl_adjusted,
                      *time_game_ptr, *timeInTrace_ptr, *timeInPVSCheck_ptr,
                      *time_frontend_ptr, *time_backend_ptr);

            #[cfg(not(target_os = "windows"))]
            {
                let com_speedslog_ptr = addr_of_mut!(com_speedslog);
                if !(*com_speedslog_ptr).is_null() && (*(*com_speedslog_ptr)).integer != 0 {
                    let speedslog_ptr = addr_of_mut!(speedslog);
                    if (*speedslog_ptr).is_null() {
                        *speedslog_ptr = FS_FOpenFileWrite(b"speeds.log\0".as_ptr() as *const c_char);
                        FS_Write(b"data={\n\0".as_ptr() as *const c_void, 8, *speedslog_ptr);
                        bComma = false;
                        if (*(*com_speedslog_ptr)).integer > 1 {
                            let logfile_ptr = addr_of_mut!(logfile);
                            FS_ForceFlush(*logfile_ptr);
                        }
                    }
                    if !(*speedslog_ptr).is_null() {
                        let mut msg: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];

                        if bComma {
                            FS_Write(b",\n\0".as_ptr() as *const c_void, 3, *speedslog_ptr);
                            bComma = false;
                        }
                        FS_Write(b"{\0".as_ptr() as *const c_void, 1, *speedslog_ptr);

                        extern "C" {
                            fn Com_sprintf(buffer: *mut c_char, bufsize: usize, fmt: *const c_char, ...);
                        }

                        // This is a placeholder for sprintf calls
                        // In real code, these would format the data

                        FS_Write(b"}\0".as_ptr() as *const c_void, 1, *speedslog_ptr);
                        bComma = true;
                    }
                }
            }

            let timeInTrace_ptr = addr_of_mut!(timeInTrace);
            let timeInPVSCheck_ptr = addr_of_mut!(timeInPVSCheck);
            *timeInTrace_ptr = 0;
            *timeInPVSCheck_ptr = 0;
        }

        let com_showtrace_ptr = addr_of_mut!(com_showtrace);
        if !(*com_showtrace_ptr).is_null() && (*(*com_showtrace_ptr)).integer != 0 {
            extern "C" {
                static mut c_traces: c_int;
                static mut c_brush_traces: c_int;
                static mut c_patch_traces: c_int;
                static mut c_pointcontents: c_int;
            }

            Com_Printf(b"%4i traces  (%ib %ip) %4i points\n\0".as_ptr() as *const c_char,
                      c_traces, c_brush_traces, c_patch_traces, c_pointcontents);
            c_traces = 0;
            c_brush_traces = 0;
            c_patch_traces = 0;
            c_pointcontents = 0;
        }

        let com_frameNumber_ptr = addr_of_mut!(com_frameNumber);
        *com_frameNumber_ptr += 1;
    }

    #[cfg(feature = "G2_PERFORMANCE_ANALYSIS")]
    {
        unsafe {
            let com_G2Report_ptr = addr_of_mut!(com_G2Report);
            if !(*com_G2Report_ptr).is_null() && (*(*com_G2Report_ptr)).integer != 0 {
                G2Time_ReportTimers();
            }
            G2Time_ResetTimers();
        }
    }
}

/*
=================
Com_Shutdown
=================
*/
extern "C" {
    fn CM_ClearMap();
    fn CM_FreeShaderText();
}

#[no_mangle]
pub extern "C" fn Com_Shutdown() {
    unsafe {
        CM_ClearMap();

        #[cfg(not(target_os = "windows"))]
        {
            CM_FreeShaderText();

            let logfile_ptr = addr_of_mut!(logfile);
            if !(*logfile_ptr).is_null() {
                FS_FCloseFile(*logfile_ptr);
                *logfile_ptr = null_mut();
            }

            let speedslog_ptr = addr_of_mut!(speedslog);
            if !(*speedslog_ptr).is_null() {
                FS_Write(b"\n};\0".as_ptr() as *const c_void, 4, *speedslog_ptr);
                FS_FCloseFile(*speedslog_ptr);
                *speedslog_ptr = null_mut();
            }

            let camerafile_ptr = addr_of_mut!(camerafile);
            if !(*camerafile_ptr).is_null() {
                FS_FCloseFile(*camerafile_ptr);
                *camerafile_ptr = null_mut();
            }

            let com_journalFile_ptr = addr_of_mut!(com_journalFile);
            if !(*com_journalFile_ptr).is_null() {
                FS_FCloseFile(*com_journalFile_ptr);
                *com_journalFile_ptr = null_mut();
            }
        }

        extern "C" {
            fn SE_ShutDown();
            fn Netchan_Shutdown();
        }

        SE_ShutDown();
        Netchan_Shutdown();
    }
}

/*
============
ParseTextFile
============
*/

#[repr(C)]
pub struct CGenericParser2 {
    _marker: [u8; 0],
}

extern "C" {
    fn FS_FOpenFileByMode(filename: *const c_char, handle: *mut *mut c_void, mode: c_int) -> c_int;
    fn FS_Read(buffer: *mut c_void, len: c_int, handle: *mut c_void) -> c_int;
}

const FS_READ: c_int = 0;

#[no_mangle]
pub extern "C" fn Com_ParseTextFile(file: *const c_char, parser: *mut CGenericParser2, cleanFirst: bool) -> bool {
    unsafe {
        let mut f: *mut c_void = null_mut();
        let mut length: c_int = 0;
        let mut buf: *mut c_char = null_mut();
        let mut bufParse: *mut c_char;

        length = FS_FOpenFileByMode(file, &mut f, FS_READ);
        if f.is_null() || length == 0 {
            return false;
        }

        buf = libc::malloc(length as usize + 1) as *mut c_char;
        FS_Read(buf as *mut c_void, length, f);
        *buf.offset(length as isize) = 0;

        bufParse = buf;
        // parser.Parse(&bufParse, cleanFirst);
        libc::free(buf as *mut c_void);

        FS_FCloseFile(f);

        true
    }
}

#[no_mangle]
pub extern "C" fn Com_ParseTextFileDestroy(parser: *mut CGenericParser2) {
    // parser.Clean();
}

#[no_mangle]
pub extern "C" fn Com_ParseTextFile_New(file: *const c_char, cleanFirst: bool, writeable: bool) -> *mut CGenericParser2 {
    unsafe {
        let mut f: *mut c_void = null_mut();
        let mut length: c_int = 0;
        let mut buf: *mut c_char = null_mut();
        let mut bufParse: *mut c_char;
        let mut parse: *mut CGenericParser2;

        length = FS_FOpenFileByMode(file, &mut f, FS_READ);
        if f.is_null() || length == 0 {
            return null_mut();
        }

        buf = libc::malloc(length as usize + 1) as *mut c_char;
        FS_Read(buf as *mut c_void, length, f);
        FS_FCloseFile(f);
        *buf.offset(length as isize) = 0;

        bufParse = buf;

        parse = libc::malloc(mem::size_of::<CGenericParser2>()) as *mut CGenericParser2;
        // if (!parse->Parse(&bufParse, cleanFirst, writeable))
        // {
        //     libc::free(parse as *mut c_void);
        //     parse = null_mut();
        // }

        libc::free(buf as *mut c_void);

        parse
    }
}
