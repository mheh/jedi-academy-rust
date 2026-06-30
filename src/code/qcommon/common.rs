// common.c -- misc functions used in client and server

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types,
         dead_code, unused_variables, unused_mut, clippy::all)]

use crate::code::game::q_shared_h::*;
use crate::code::qcommon::qcommon_h::*;
use crate::code::qcommon::sstring_h::*;
use crate::code::qcommon::stv_version_h::*;

// #ifdef _XBOX
//   #include "../win32/win_file.h"  (Xbox feature only, not ported here)
//   #include "../ui/ui_splash.h"    (Xbox feature only, not ported here)
// #endif

// #ifndef FINAL_BUILD
//   #include "platform.h" -> crate::code::qcommon::platform_h (provides OutputDebugString)
// #endif
#[cfg(all(not(feature = "xbox"), not(feature = "final_build")))]
use crate::code::qcommon::platform_h::*;

use core::ffi::{c_char, c_double, c_float, c_int, c_uint, c_void};
use core::ptr::{addr_of, addr_of_mut};

// #define MAXPRINTMSG 4096
pub const MAXPRINTMSG: usize = 4096;

// #define MAX_NUM_ARGVS 50
pub const MAX_NUM_ARGVS: usize = 50;

pub static mut com_argc: c_int = 0;
pub static mut com_argv: [*mut c_char; MAX_NUM_ARGVS + 1] =
    [core::ptr::null_mut(); MAX_NUM_ARGVS + 1];

#[cfg(not(feature = "xbox"))]
static mut logfile: fileHandle_t = 0;
#[cfg(not(feature = "xbox"))]
static mut speedslog: fileHandle_t = 0;
#[cfg(not(feature = "xbox"))]
static mut camerafile: fileHandle_t = 0;
#[cfg(not(feature = "xbox"))]
pub static mut com_journalFile: fileHandle_t = 0;
#[cfg(not(feature = "xbox"))]
pub static mut com_journalDataFile: fileHandle_t = 0; // config files are written here

pub static mut com_viewlog: *mut cvar_t = core::ptr::null_mut();
pub static mut com_speeds: *mut cvar_t = core::ptr::null_mut();
pub static mut com_developer: *mut cvar_t = core::ptr::null_mut();
pub static mut com_timescale: *mut cvar_t = core::ptr::null_mut();
pub static mut com_fixedtime: *mut cvar_t = core::ptr::null_mut();
pub static mut com_maxfps: *mut cvar_t = core::ptr::null_mut();
pub static mut com_sv_running: *mut cvar_t = core::ptr::null_mut();
pub static mut com_cl_running: *mut cvar_t = core::ptr::null_mut();
pub static mut com_logfile: *mut cvar_t = core::ptr::null_mut(); // 1 = buffer log, 2 = flush after each print
pub static mut com_showtrace: *mut cvar_t = core::ptr::null_mut();
pub static mut com_terrainPhysics: *mut cvar_t = core::ptr::null_mut();
pub static mut com_version: *mut cvar_t = core::ptr::null_mut();
pub static mut com_buildScript: *mut cvar_t = core::ptr::null_mut(); // for automated data building scripts
pub static mut cl_paused: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_paused: *mut cvar_t = core::ptr::null_mut();
pub static mut com_skippingcin: *mut cvar_t = core::ptr::null_mut();
pub static mut com_speedslog: *mut cvar_t = core::ptr::null_mut(); // 1 = buffer log, 2 = flush after each print

#[cfg(feature = "g2_performance_analysis")]
pub static mut com_G2Report: *mut cvar_t = core::ptr::null_mut();

// com_speeds times
pub static mut time_game: c_int = 0;
pub static mut time_frontend: c_int = 0; // renderer frontend time
pub static mut time_backend: c_int = 0;  // renderer backend time

pub static mut timeInTrace: c_int = 0;
pub static mut timeInPVSCheck: c_int = 0;
pub static mut numTraces: c_int = 0;

pub static mut com_frameTime: c_int = 0;
pub static mut com_frameMsec: c_int = 0;
pub static mut com_frameNumber: c_int = 0;

pub static mut com_errorEntered: qboolean = qfalse;
pub static mut com_fullyInitialized: qboolean = qfalse;

pub static mut com_errorMessage: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];

// void Com_WriteConfig_f( void );  // forward declaration (not needed in Rust)
//JLF
//void G_DemoFrame();

//============================================================================

// External declarations for functions forward-declared at C file scope
// or inline in function bodies (not provided by included headers).
extern "C" {
    // pretty sucky, but that's how SoF did it...<g>
    fn SG_WipeSavegame(name: *const c_char);
    fn SG_Shutdown();

    // declared extern before Com_Init body
    fn Com_InitZoneMemory();
    fn R_InitWorldEffects();

    // declared extern before Com_Shutdown
    fn CM_FreeShaderText();

    // declared extern inside Hunk_Clear body
    fn CIN_CloseAllVideos();
    fn R_ClearStuffToStopGhoul2CrashingThings();

    // declared extern inside Com_Shutdown body
    fn Netchan_Shutdown();

    // declared extern inside Com_WriteCam / Com_FlushCamFile bodies
    static mut sv_mapname: *mut cvar_t;

    // declared extern inside Com_Frame body
    static mut c_traces: c_int;
    static mut c_brush_traces: c_int;
    static mut c_patch_traces: c_int;
    static mut c_pointcontents: c_int;

    // C standard library functions used in this file
    // (come through system headers in C but require explicit extern in Rust)
    // va_list is opaque; *mut c_void is used as a blind-port placeholder for va_list
    fn vsprintf(s: *mut c_char, format: *const c_char, ap: *mut c_void) -> c_int;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn toupper(c: c_int) -> c_int;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> c_double;
}

// Xbox-only extern declarations
#[cfg(feature = "xbox")]
extern "C" {
    fn WF_Init();
    fn CL_InitRef();
    fn R_Register();
    fn GLimp_Init();
    fn SP_DoLicense();
    fn Sys_InitFileCodes();
    fn Sys_FilecodeScan_f();
    fn Sys_StreamInit();
    fn CL_StartSound();
    fn SE_CheckForLanguageUpdates();
    fn TestDemoTimer() -> bool;
    fn PlayDemo();
    fn Sys_StreamShutdown();
    fn Sys_ShutdownFileCodes();
}

// G2 performance analysis declarations
#[cfg(feature = "g2_performance_analysis")]
extern "C" {
    fn G2Time_ResetTimers();
    fn G2Time_ReportTimers();
}

//============================================================================

#[cfg(not(feature = "xbox"))]
static mut rd_buffer: *mut c_char = core::ptr::null_mut();
#[cfg(not(feature = "xbox"))]
static mut rd_buffersize: c_int = 0;
#[cfg(not(feature = "xbox"))]
static mut rd_flush: Option<unsafe extern "C" fn(*mut c_char)> = None;

// #ifndef _XBOX
// #ifndef FINAL_BUILD
// #define OUTPUT_TO_BUILD_WINDOW
// #endif
// #endif  //not xbox
// (OUTPUT_TO_BUILD_WINDOW = cfg(all(not(feature="xbox"), not(feature="final_build"))))

#[cfg(not(feature = "xbox"))]
pub unsafe fn Com_BeginRedirect(
    buffer: *mut c_char,
    buffersize: c_int,
    flush: Option<unsafe extern "C" fn(*mut c_char)>,
) {
    if buffer.is_null() || buffersize == 0 || flush.is_none() {
        return;
    }
    rd_buffer = buffer;
    rd_buffersize = buffersize;
    rd_flush = flush;

    *rd_buffer = 0;
}

#[cfg(not(feature = "xbox"))]
pub unsafe fn Com_EndRedirect() {
    if let Some(flush_fn) = rd_flush {
        flush_fn(rd_buffer);
    }

    rd_buffer = core::ptr::null_mut();
    rd_buffersize = 0;
    rd_flush = None;
}

/*
=============
Com_Printf

Both client and server can use this, and it will output
to the apropriate place.

A raw string should NEVER be passed as fmt, because of "%f" type crashers.
=============
*/
// Requires #![feature(c_variadic)] in crate root for variadic Rust fn definitions.
// va_list passed as *mut c_void to the vsprintf extern declaration (blind-port placeholder).
pub unsafe extern "C" fn Com_Printf(fmt: *const c_char, mut args: ...) {
    let mut argptr = args.as_va_list();
    let mut msg: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];

    // va_start(argptr, fmt);
    vsprintf(msg.as_mut_ptr(), fmt, &mut argptr as *mut _ as *mut c_void);
    // va_end(argptr);

    #[cfg(not(feature = "xbox"))]
    {
        if !rd_buffer.is_null() {
            if (strlen(msg.as_ptr()) + strlen(rd_buffer)) > (rd_buffersize as usize - 1) {
                if let Some(flush_fn) = rd_flush {
                    flush_fn(rd_buffer);
                }
                *rd_buffer = 0;
            }
            strcat(rd_buffer, msg.as_ptr());
            return;
        }
    }

    CL_ConsolePrint(msg.as_ptr());

    // echo to dedicated console and early console
    Sys_Print(msg.as_ptr());

    // #ifdef OUTPUT_TO_BUILD_WINDOW
    #[cfg(all(not(feature = "xbox"), not(feature = "final_build")))]
    OutputDebugString(msg.as_ptr());
    // #endif

    #[cfg(not(feature = "xbox"))]
    {
        // logfile
        if !com_logfile.is_null() && (*com_logfile).integer != 0 {
            if logfile == 0 {
                logfile = FS_FOpenFileWrite(b"qconsole.log\0".as_ptr() as *const c_char);
                if (*com_logfile).integer > 1 {
                    // force it to not buffer so we get valid
                    // data even if we are crashing
                    FS_ForceFlush(logfile);
                }
            }
            if logfile != 0 {
                FS_Write(msg.as_ptr() as *const c_void, strlen(msg.as_ptr()), logfile);
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
pub unsafe extern "C" fn Com_DPrintf(fmt: *const c_char, mut args: ...) {
    let mut argptr = args.as_va_list();
    let mut msg: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];

    if com_developer.is_null() || (*com_developer).integer == 0 {
        return; // don't confuse non-developers with techie stuff...
    }

    // va_start(argptr, fmt);
    vsprintf(msg.as_mut_ptr(), fmt, &mut argptr as *mut _ as *mut c_void);
    // va_end(argptr);

    Com_Printf(b"%s\0".as_ptr() as *const c_char, msg.as_ptr());
}

pub unsafe fn Com_WriteCam(text: *const c_char) {
    #[cfg(not(feature = "xbox"))]
    {
        static mut mapname: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        // camerafile
        if camerafile == 0 {
            // extern cvar_t *sv_mapname; (declared at module level above)

            //NOTE: always saves in working dir if using one...
            sprintf(
                mapname.as_mut_ptr(),
                b"maps/%s_cam.map\0".as_ptr() as *const c_char,
                (*sv_mapname).string,
            );
            camerafile = FS_FOpenFileWrite(mapname.as_ptr());
        }

        if camerafile != 0 {
            FS_Printf(camerafile, b"%s\0".as_ptr() as *const c_char, text);
        }

        Com_Printf(b"%s\n\0".as_ptr() as *const c_char, mapname.as_ptr());
    }
}

pub unsafe fn Com_FlushCamFile() {
    #[cfg(not(feature = "xbox"))]
    {
        if camerafile == 0 {
            // nothing to flush, right?
            Com_Printf(b"No cam file available\n\0".as_ptr() as *const c_char);
            return;
        }
        FS_ForceFlush(camerafile);
        FS_FCloseFile(camerafile);
        camerafile = 0;

        static mut flushedMapname: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        // extern cvar_t *sv_mapname; (declared at module level above)
        sprintf(
            flushedMapname.as_mut_ptr(),
            b"maps/%s_cam.map\0".as_ptr() as *const c_char,
            (*sv_mapname).string,
        );
        Com_Printf(
            b"flushed all cams to %s\n\0".as_ptr() as *const c_char,
            flushedMapname.as_ptr(),
        );
    }
}

/*
=============
Com_Error

Both client and server can use this, and it will
do the apropriate things.
=============
*/

// void SG_WipeSavegame(const char *name);  // pretty sucky, but that's how SoF did it...<g>
// void SG_Shutdown();
// (declared at module-level extern block above)
//void SCR_UnprecacheScreenshot();

pub unsafe extern "C" fn Com_Error(code: c_int, fmt: *const c_char, mut args: ...) {
    let mut argptr = args.as_va_list();
    // C function parameter is mutable; shadow it so `code = ERR_FATAL` below takes effect.
    let mut code = code;

    // #if defined(_WIN32) && defined(_DEBUG)
    #[cfg(all(target_os = "windows", debug_assertions))]
    {
        if code != ERR_DISCONNECT && code != ERR_NEED_CD {
            //		if (com_noErrorInterrupt && !com_noErrorInterrupt->integer)
            {
                // __asm { int 0x03 }
                #[cfg(target_arch = "x86")]
                core::arch::asm!("int 0x03");
                #[cfg(target_arch = "x86_64")]
                core::arch::asm!("int3");
            }
        }
    }
    // #endif

    // when we are running automated scripts, make sure we
    // know if anything failed
    if !com_buildScript.is_null() && (*com_buildScript).integer != 0 {
        code = ERR_FATAL;
    }

    if com_errorEntered != qfalse {
        Sys_Error(
            b"recursive error after: %s\0".as_ptr() as *const c_char,
            com_errorMessage.as_ptr(),
        );
    }

    com_errorEntered = qtrue;

    //reset some game stuff here
    //	SCR_UnprecacheScreenshot();

    // va_start(argptr, fmt);
    vsprintf(
        com_errorMessage.as_mut_ptr(),
        fmt,
        &mut argptr as *mut _ as *mut c_void,
    );
    // va_end(argptr);

    if code != ERR_DISCONNECT {
        Cvar_Get(
            b"com_errorMessage\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_ROM,
        ); //give com_errorMessage a default so it won't come back to life after a resetDefaults
        Cvar_Set(
            b"com_errorMessage\0".as_ptr() as *const c_char,
            com_errorMessage.as_ptr(),
        );
    }

    SG_Shutdown(); // close any file pointers
    if code == ERR_DISCONNECT {
        SV_Shutdown(b"Disconnect\0".as_ptr() as *const c_char);
        CL_Disconnect();
        CL_FlushMemory();
        CL_StartHunkUsers();
        com_errorEntered = qfalse;
        // C++: throw ("DISCONNECTED\n");
        panic!("DISCONNECTED\n");
    } else if code == ERR_DROP {
        // If loading/saving caused the crash/error - delete the temp file
        SG_WipeSavegame(b"current\0".as_ptr() as *const c_char); // delete file

        SV_Shutdown(va(
            b"Server crashed: %s\n\0".as_ptr() as *const c_char,
            com_errorMessage.as_ptr(),
        ));
        CL_Disconnect();
        CL_FlushMemory();
        CL_StartHunkUsers();
        // S_COLOR_RED = "^1", S_COLOR_MAGENTA = "^5" from q_shared.h adjacent-string concat
        Com_Printf(
            b"^1********************\n^5ERROR: %s\n^1********************\n\0".as_ptr()
                as *const c_char,
            com_errorMessage.as_ptr(),
        );
        com_errorEntered = qfalse;
        // C++: throw ("DROPPED\n");
        panic!("DROPPED\n");
    } else if code == ERR_NEED_CD {
        SV_Shutdown(b"Server didn't have CD\n\0".as_ptr() as *const c_char);
        if !com_cl_running.is_null() && (*com_cl_running).integer != 0 {
            CL_Disconnect();
            CL_FlushMemory();
            CL_StartHunkUsers();
            com_errorEntered = qfalse;
        } else {
            Com_Printf(b"Server didn't have CD\n\0".as_ptr() as *const c_char);
        }
        // C++: throw ("NEED CD\n");
        panic!("NEED CD\n");
    } else {
        CL_Shutdown();
        SV_Shutdown(va(
            // S_COLOR_RED = "^1"
            b"^1Server fatal crashed: %s\n\0".as_ptr() as *const c_char,
            com_errorMessage.as_ptr(),
        ));
    }

    Com_Shutdown();

    Sys_Error(b"%s\0".as_ptr() as *const c_char, com_errorMessage.as_ptr());
}


/*
=============
Com_Quit_f

Both client and server can use this, and it will
do the apropriate things.
=============
*/
pub unsafe fn Com_Quit_f() {
    // don't try to shutdown if we are in a recursive error
    if com_errorEntered == qfalse {
        SV_Shutdown(b"Server quit\n\0".as_ptr() as *const c_char);
        CL_Shutdown();
        Com_Shutdown();
    }
    Sys_Quit();
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

// #define MAX_CONSOLE_LINES 32
const MAX_CONSOLE_LINES: usize = 32;
pub static mut com_numConsoleLines: c_int = 0;
pub static mut com_consoleLines: [*mut c_char; MAX_CONSOLE_LINES] =
    [core::ptr::null_mut(); MAX_CONSOLE_LINES];

/*
==================
Com_ParseCommandLine

Break it up into multiple console lines
==================
*/
pub unsafe fn Com_ParseCommandLine(mut commandLine: *mut c_char) {
    com_consoleLines[0] = commandLine;
    com_numConsoleLines = 1;

    while *commandLine != 0 {
        // look for a + seperating character
        // if commandLine came from a file, we might have real line seperators
        if *commandLine == b'+' as c_char || *commandLine == b'\n' as c_char {
            if com_numConsoleLines == MAX_CONSOLE_LINES as c_int {
                return;
            }
            com_consoleLines[com_numConsoleLines as usize] = commandLine.add(1);
            com_numConsoleLines += 1;
            *commandLine = 0;
        }
        commandLine = commandLine.add(1);
    }
}


/*
===================
Com_SafeMode

Check for "safe" on the command line, which will
skip loading of jaconfig.cfg
===================
*/
pub unsafe fn Com_SafeMode() -> qboolean {
    let mut i: c_int;

    i = 0;
    while i < com_numConsoleLines {
        Cmd_TokenizeString(com_consoleLines[i as usize]);
        if Q_stricmp(Cmd_Argv(0), b"safe\0".as_ptr() as *const c_char) == 0
            || Q_stricmp(Cmd_Argv(0), b"cvar_restart\0".as_ptr() as *const c_char) == 0
        {
            // C: com_consoleLines[i][0] = 0;  -- zeroes first byte of the pointed-to string
            *com_consoleLines[i as usize] = 0;
            return qtrue;
        }
        i += 1;
    }
    qfalse
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
pub unsafe fn Com_StartupVariable(match_: *const c_char) {
    let mut i: c_int;
    let mut s: *mut c_char;
    let mut cv: *mut cvar_t;

    i = 0;
    while i < com_numConsoleLines {
        Cmd_TokenizeString(com_consoleLines[i as usize]);
        if strcmp(Cmd_Argv(0), b"set\0".as_ptr() as *const c_char) != 0 {
            i += 1;
            continue;
        }

        s = Cmd_Argv(1);
        if match_.is_null() || stricmp(s, match_) == 0 {
            Cvar_Set(s, Cmd_Argv(2));
            cv = Cvar_Get(s, b"\0".as_ptr() as *const c_char, 0);
            (*cv).flags |= CVAR_USER_CREATED;
            //			com_consoleLines[i] = 0;
        }
        i += 1;
    }
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
pub unsafe fn Com_AddStartupCommands() -> qboolean {
    let mut i: c_int;
    let mut added: qboolean;

    added = qfalse;
    // quote every token, so args with semicolons can work
    i = 0;
    while i < com_numConsoleLines {
        if com_consoleLines[i as usize].is_null()
            || *com_consoleLines[i as usize] == 0
        {
            i += 1;
            continue;
        }

        // set commands won't override menu startup
        if Q_stricmpn(com_consoleLines[i as usize], b"set\0".as_ptr() as *const c_char, 3) != 0 {
            added = qtrue;
        }
        Cbuf_AddText(com_consoleLines[i as usize]);
        Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
        i += 1;
    }

    added
}


//============================================================================


pub unsafe fn Info_Print(mut s: *const c_char) {
    let mut key: [c_char; 512] = [0; 512];
    let mut value: [c_char; 512] = [0; 512];
    let mut o: *mut c_char;
    let mut l: c_int;

    if *s == b'\\' as c_char {
        s = s.add(1);
    }
    while *s != 0 {
        o = key.as_mut_ptr();
        while *s != 0 && *s != b'\\' as c_char {
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }

        l = o.offset_from(key.as_ptr()) as c_int;
        if l < 20 {
            memset(o as *mut c_void, b' ' as c_int, (20 - l) as usize);
            key[20] = 0;
        } else {
            *o = 0;
        }
        Com_Printf(b"%s\0".as_ptr() as *const c_char, key.as_ptr());

        if *s == 0 {
            Com_Printf(b"MISSING VALUE\n\0".as_ptr() as *const c_char);
            return;
        }

        o = value.as_mut_ptr();
        s = s.add(1);
        while *s != 0 && *s != b'\\' as c_char {
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;

        if *s != 0 {
            s = s.add(1);
        }
        Com_Printf(b"%s\n\0".as_ptr() as *const c_char, value.as_ptr());
    }
}

/*
============
Com_StringContains
============
*/
pub unsafe fn Com_StringContains(
    mut str1: *mut c_char,
    str2: *mut c_char,
    casesensitive: c_int,
) -> *mut c_char {
    let len: c_int;
    let mut i: c_int;
    let mut j: c_int;

    len = strlen(str1) as c_int - strlen(str2) as c_int;
    i = 0;
    while i <= len {
        j = 0;
        loop {
            if *str2.add(j as usize) == 0 {
                break;
            }
            if casesensitive != 0 {
                if *str1.add(j as usize) != *str2.add(j as usize) {
                    break;
                }
            } else {
                if toupper(*str1.add(j as usize) as c_int)
                    != toupper(*str2.add(j as usize) as c_int)
                {
                    break;
                }
            }
            j += 1;
        }
        if *str2.add(j as usize) == 0 {
            return str1;
        }
        i += 1;
        str1 = str1.add(1);
    }
    core::ptr::null_mut()
}

/*
============
Com_Filter
============
*/
pub unsafe fn Com_Filter(
    mut filter: *mut c_char,
    mut name: *mut c_char,
    casesensitive: c_int,
) -> c_int {
    let mut buf: [c_char; MAX_TOKEN_CHARS as usize] = [0; MAX_TOKEN_CHARS as usize];
    let mut ptr: *mut c_char;
    let mut i: c_int;

    while *filter != 0 {
        if *filter == b'*' as c_char {
            filter = filter.add(1);
            i = 0;
            while *filter != 0 {
                if *filter == b'*' as c_char || *filter == b'?' as c_char {
                    break;
                }
                buf[i as usize] = *filter;
                filter = filter.add(1);
                i += 1;
            }
            buf[i as usize] = b'\0' as c_char;
            if strlen(buf.as_ptr()) != 0 {
                ptr = Com_StringContains(name, buf.as_mut_ptr(), casesensitive);
                if ptr.is_null() {
                    return qfalse;
                }
                name = ptr.add(strlen(buf.as_ptr()));
            }
        } else if *filter == b'?' as c_char {
            filter = filter.add(1);
            name = name.add(1);
        } else {
            if casesensitive != 0 {
                if *filter != *name {
                    return qfalse;
                }
            } else {
                if toupper(*filter as c_int) != toupper(*name as c_int) {
                    return qfalse;
                }
            }
            filter = filter.add(1);
            name = name.add(1);
        }
    }
    qtrue
}



/*
=================
Com_InitHunkMemory
=================
*/
pub unsafe fn Com_InitHunkMemory() {
    Hunk_Clear();

    //	Cmd_AddCommand( "meminfo", Z_Details_f );
}

// I'm leaving this in just in case we ever need to remember where's a good place to hook something like this in.
//
pub unsafe fn Com_ShutdownHunkMemory() {
}


/*
===================
Hunk_SetMark

The server calls this after the level and game VM have been loaded
===================
*/
pub unsafe fn Hunk_SetMark() {
}



/*
=================
Hunk_ClearToMark

The client calls this before starting a vid_restart or snd_restart
=================
*/
pub unsafe fn Hunk_ClearToMark() {
    Z_TagFree(TAG_HUNKALLOC);
    Z_TagFree(TAG_HUNKMISCMODELS);
}



/*
=================
Hunk_Clear

The server calls this before shutting down or loading a new map
=================
*/
pub unsafe fn Hunk_Clear() {
    Z_TagFree(TAG_HUNKALLOC);
    Z_TagFree(TAG_HUNKMISCMODELS);

    // extern void CIN_CloseAllVideos(); (declared at module level above)
    CIN_CloseAllVideos();

    // extern void R_ClearStuffToStopGhoul2CrashingThings(void); (declared at module level above)
    R_ClearStuffToStopGhoul2CrashingThings();
}



/*
===================================================================

EVENTS AND JOURNALING

In addition to these events, .cfg files are also copied to the
journaled file
===================================================================
*/

// #define MAX_PUSHED_EVENTS 64
const MAX_PUSHED_EVENTS: usize = 64;
pub static mut com_pushedEventsHead: c_int = 0;
pub static mut com_pushedEventsTail: c_int = 0;
pub static mut com_pushedEvents: [sysEvent_t; MAX_PUSHED_EVENTS] =
    unsafe { core::mem::zeroed() };

/*
=================
Com_GetRealEvent
=================
*/
pub unsafe fn Com_GetRealEvent() -> sysEvent_t {
    let ev: sysEvent_t;

    // get an event from the system
    ev = Sys_GetEvent();

    ev
}

/*
=================
Com_PushEvent
=================
*/
pub unsafe fn Com_PushEvent(event: *mut sysEvent_t) {
    let ev: *mut sysEvent_t;
    static mut printedWarning: c_int = 0;

    ev = &mut com_pushedEvents
        [(com_pushedEventsHead as usize) & (MAX_PUSHED_EVENTS - 1)];

    if com_pushedEventsHead - com_pushedEventsTail >= MAX_PUSHED_EVENTS as c_int {

        // don't print the warning constantly, or it can give time for more...
        if printedWarning == 0 {
            printedWarning = qtrue;
            Com_Printf(b"WARNING: Com_PushEvent overflow\n\0".as_ptr() as *const c_char);
        }

        if !(*ev).evPtr.is_null() {
            Z_Free((*ev).evPtr);
        }
        com_pushedEventsTail += 1;
    } else {
        printedWarning = qfalse;
    }

    *ev = *event;
    com_pushedEventsHead += 1;
}

/*
=================
Com_GetEvent
=================
*/
pub unsafe fn Com_GetEvent() -> sysEvent_t {
    if com_pushedEventsHead > com_pushedEventsTail {
        com_pushedEventsTail += 1;
        return com_pushedEvents
            [((com_pushedEventsTail - 1) as usize) & (MAX_PUSHED_EVENTS - 1)];
    }
    Com_GetRealEvent()
}

/*
=================
Com_RunAndTimeServerPacket
=================
*/
pub unsafe fn Com_RunAndTimeServerPacket(evFrom: *mut netadr_t, buf: *mut msg_t) {
    let mut t1: c_int = 0;
    let mut t2: c_int;
    let msec: c_int;

    t1 = 0;

    if (*com_speeds).integer != 0 {
        t1 = Sys_Milliseconds();
    }

    SV_PacketEvent(*evFrom, buf);

    if (*com_speeds).integer != 0 {
        t2 = Sys_Milliseconds();
        msec = t2 - t1;
        if (*com_speeds).integer == 3 {
            Com_Printf(
                b"SV_PacketEvent time: %i\n\0".as_ptr() as *const c_char,
                msec,
            );
        }
    }
}

/*
=================
Com_EventLoop

Returns last event time
=================
*/
pub unsafe fn Com_EventLoop() -> c_int {
    let mut ev: sysEvent_t;
    let mut evFrom: netadr_t = core::mem::zeroed();
    let mut bufData: [byte; MAX_MSGLEN as usize] = [0; MAX_MSGLEN as usize];
    let mut buf: msg_t = core::mem::zeroed();

    MSG_Init(&mut buf, bufData.as_mut_ptr(), core::mem::size_of_val(&bufData) as c_int);

    loop {
        ev = Com_GetEvent();

        // if no more events are available
        if ev.evType == SE_NONE {
            // manually send packet events for the loopback channel
            while NET_GetLoopPacket(NS_CLIENT, &mut evFrom, &mut buf) != 0 {
                CL_PacketEvent(evFrom, &mut buf);
            }

            while NET_GetLoopPacket(NS_SERVER, &mut evFrom, &mut buf) != 0 {
                // if the server just shut down, flush the events
                if (*com_sv_running).integer != 0 {
                    Com_RunAndTimeServerPacket(&mut evFrom, &mut buf);
                }
            }

            return ev.evTime;
        }


        match ev.evType {
            x if x == SE_NONE => {
                // SE_NONE handled above and as default; also listed here per C switch
            }
            x if x == SE_KEY => {
                CL_KeyEvent(ev.evValue, ev.evValue2, ev.evTime);
            }
            x if x == SE_CHAR => {
                CL_CharEvent(ev.evValue);
            }
            x if x == SE_MOUSE => {
                CL_MouseEvent(ev.evValue, ev.evValue2, ev.evTime);
            }
            x if x == SE_JOYSTICK_AXIS => {
                CL_JoystickEvent(ev.evValue, ev.evValue2, ev.evTime);
            }
            x if x == SE_CONSOLE => {
                Cbuf_AddText(ev.evPtr as *mut c_char);
                Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
            }
            x if x == SE_PACKET => {
                evFrom = *(ev.evPtr as *mut netadr_t);
                buf.cursize = ev.evPtrLength
                    - core::mem::size_of::<netadr_t>() as c_int;

                // we must copy the contents of the message out, because
                // the event buffers are only large enough to hold the
                // exact payload, but channel messages need to be large
                // enough to hold fragment reassembly
                if buf.cursize as c_uint > buf.maxsize as c_uint {
                    Com_Printf(
                        b"Com_EventLoop: oversize packet\n\0".as_ptr() as *const c_char,
                    );
                    // C: continue -- skips Z_Free(evPtr) at the bottom of the loop (faithful)
                    continue;
                }
                memcpy(
                    buf.data as *mut c_void,
                    (ev.evPtr as *mut netadr_t).add(1) as *const c_void,
                    buf.cursize as usize,
                );
                if (*com_sv_running).integer != 0 {
                    Com_RunAndTimeServerPacket(&mut evFrom, &mut buf);
                } else {
                    CL_PacketEvent(evFrom, &mut buf);
                }
            }
            _ => {
                Com_Error(
                    ERR_FATAL,
                    b"Com_EventLoop: bad event type %i\0".as_ptr() as *const c_char,
                    ev.evTime,
                );
            }
        }

        // free any block data
        if !ev.evPtr.is_null() {
            Z_Free(ev.evPtr);
        }
    }
}

/*
================
Com_Milliseconds

Can be used for profiling, but will be journaled accurately
================
*/
pub unsafe fn Com_Milliseconds() -> c_int {
    let mut ev: sysEvent_t;

    // get events and push them until we get a null event with the current time
    loop {
        ev = Com_GetRealEvent();
        if ev.evType != SE_NONE {
            Com_PushEvent(&mut ev);
        }
        if ev.evType == SE_NONE {
            break;
        }
    }

    ev.evTime
}

//============================================================================

/*
=============
Com_Error_f

Just throw a fatal error to
test error shutdown procedures
=============
*/
// C: static void Com_Error_f (void)  -- file-local static function
unsafe fn Com_Error_f() {
    if Cmd_Argc() > 1 {
        Com_Error(ERR_DROP, b"Testing drop error\0".as_ptr() as *const c_char);
    } else {
        Com_Error(ERR_FATAL, b"Testing fatal error\0".as_ptr() as *const c_char);
    }
}


/*
=============
Com_Freeze_f

Just freeze in place for a given number of seconds to test
error recovery
=============
*/
// C: static void Com_Freeze_f (void)
unsafe fn Com_Freeze_f() {
    let s: c_float;
    let start: c_int;
    let mut now: c_int;

    if Cmd_Argc() != 2 {
        Com_Printf(b"freeze <seconds>\n\0".as_ptr() as *const c_char);
        return;
    }
    s = atof(Cmd_Argv(1)) as c_float;

    start = Com_Milliseconds();

    loop {
        now = Com_Milliseconds();
        if ((now - start) as c_float) * 0.001_f32 > s {
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
// C: static void Com_Crash_f( void )
unsafe fn Com_Crash_f() {
    *(0 as *mut c_int) = 0x12345678;
}

/*
=================
Com_Init
=================
*/
// extern void Com_InitZoneMemory();  (declared at module-level extern block)
// extern void R_InitWorldEffects();  (declared at module-level extern block)
pub unsafe fn Com_Init(commandLine: *mut c_char) {
    let mut s: *mut c_char;

    Com_Printf(
        b"%s %s %s\n\0".as_ptr() as *const c_char,
        Q3_VERSION,
        CPUSTRING,
        b"__DATE__\0".as_ptr() as *const c_char, // C: __DATE__ preprocessor macro
    );

    // C++: try {
    // (C++ exceptions are not translated to Rust catch_unwind; throw -> panic! in callee)
    {
        // prepare enough of the subsystems to handle
        // cvar and command buffer management
        Com_ParseCommandLine(commandLine);

        Swap_Init();
        Cbuf_Init();

        Com_InitZoneMemory();

        // #ifdef _XBOX
        #[cfg(feature = "xbox")]
        {
            WF_Init();
            // set up ri
            // extern void CL_InitRef( void );
            CL_InitRef();

            // register renderer cvars
            // extern void R_Register(void);
            R_Register();

            // start the gl render layer
            // extern void GLimp_Init(void);
            GLimp_Init();

            // put up the license screen
            SP_DoLicense();
        }
        // #endif

        Cmd_Init();
        Cvar_Init();

        // get the commandline cvars set
        Com_StartupVariable(core::ptr::null());

        // done early so bind command exists
        CL_InitKeyCommands();

        // #ifdef _XBOX
        #[cfg(feature = "xbox")]
        {
            // extern void Sys_FilecodeScan_f();
            Sys_InitFileCodes();
            Cmd_AddCommand(
                b"filecodes\0".as_ptr() as *const c_char,
                Some(Sys_FilecodeScan_f),
            );

            // extern void Sys_StreamInit();
            Sys_StreamInit();
        }
        // #endif

        FS_InitFilesystem(); //uses z_malloc
        R_InitWorldEffects(); // this doesn't do much but I want to be sure certain variables are intialized.

        Cbuf_AddText(b"exec default.cfg\n\0".as_ptr() as *const c_char);

        // skip the jaconfig.cfg if "safe" is on the command line
        if Com_SafeMode() == qfalse {
            Cbuf_AddText(b"exec jaconfig.cfg\n\0".as_ptr() as *const c_char);
        }

        Cbuf_AddText(b"exec autoexec.cfg\n\0".as_ptr() as *const c_char);

        Cbuf_Execute();

        // override anything from the config files with command line args
        Com_StartupVariable(core::ptr::null());

        // allocate the stack based hunk allocator
        Com_InitHunkMemory();

        // if any archived cvars are modified after this, we will trigger a writing
        // of the config file
        cvar_modifiedFlags &= !CVAR_ARCHIVE;

        //
        // init commands and vars
        //
        Cmd_AddCommand(b"quit\0".as_ptr() as *const c_char, Some(Com_Quit_f as unsafe fn()));
        Cmd_AddCommand(
            b"writeconfig\0".as_ptr() as *const c_char,
            Some(Com_WriteConfig_f as unsafe fn()),
        );

        com_maxfps = Cvar_Get(
            b"com_maxfps\0".as_ptr() as *const c_char,
            b"85\0".as_ptr() as *const c_char,
            CVAR_ARCHIVE,
        );

        com_developer = Cvar_Get(
            b"developer\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_TEMP,
        );
        com_logfile = Cvar_Get(
            b"logfile\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_TEMP,
        );
        com_speedslog = Cvar_Get(
            b"speedslog\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_TEMP,
        );

        com_timescale = Cvar_Get(
            b"timescale\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        );
        com_fixedtime = Cvar_Get(
            b"fixedtime\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        );
        com_showtrace = Cvar_Get(
            b"com_showtrace\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        );
        com_terrainPhysics = Cvar_Get(
            b"com_terrainPhysics\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        );
        com_viewlog = Cvar_Get(
            b"viewlog\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_TEMP,
        );
        com_speeds = Cvar_Get(
            b"com_speeds\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        );

        // #ifdef G2_PERFORMANCE_ANALYSIS
        #[cfg(feature = "g2_performance_analysis")]
        {
            com_G2Report = Cvar_Get(
                b"com_G2Report\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
                0,
            );
        }
        // #endif

        cl_paused = Cvar_Get(
            b"cl_paused\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_ROM,
        );
        sv_paused = Cvar_Get(
            b"sv_paused\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_ROM,
        );
        com_sv_running = Cvar_Get(
            b"sv_running\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_ROM,
        );
        com_cl_running = Cvar_Get(
            b"cl_running\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_ROM,
        );
        com_skippingcin = Cvar_Get(
            b"skippingCinematic\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_ROM,
        );
        com_buildScript = Cvar_Get(
            b"com_buildScript\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        );

        if !com_developer.is_null() && (*com_developer).integer != 0 {
            Cmd_AddCommand(
                b"error\0".as_ptr() as *const c_char,
                Some(Com_Error_f as unsafe fn()),
            );
            Cmd_AddCommand(
                b"crash\0".as_ptr() as *const c_char,
                Some(Com_Crash_f as unsafe fn()),
            );
            Cmd_AddCommand(
                b"freeze\0".as_ptr() as *const c_char,
                Some(Com_Freeze_f as unsafe fn()),
            );
        }

        s = va(
            b"%s %s %s\0".as_ptr() as *const c_char,
            Q3_VERSION,
            CPUSTRING,
            b"__DATE__\0".as_ptr() as *const c_char, // C: __DATE__ preprocessor macro
        );
        com_version = Cvar_Get(
            b"version\0".as_ptr() as *const c_char,
            s,
            CVAR_ROM | CVAR_SERVERINFO,
        );

        SE_Init(); // Initialize StringEd

        Sys_Init(); // this also detects CPU type, so I can now do this CPU check below...

        Netchan_Init(Com_Milliseconds() & 0xffff); // pick a port value that should be nice and random
        //	VM_Init();
        SV_Init();

        CL_Init();

        // #ifdef _XBOX
        // Experiment. Sound memory never gets freed, move it earlier. This
        // will also let us play movies sooner, if we need to.
        #[cfg(feature = "xbox")]
        {
            // extern void CL_StartSound(void);
            CL_StartSound();
        }
        // #endif

        Sys_ShowConsole((*com_viewlog).integer, qfalse);

        // set com_frameTime so that if a map is started on the
        // command line it will still be able to count on com_frameTime
        // being random enough for a serverid
        com_frameTime = Com_Milliseconds();

        // add + commands from command line
        // #ifndef _XBOX
        #[cfg(not(feature = "xbox"))]
        {
            if Com_AddStartupCommands() == qfalse {
                // #ifdef NDEBUG
                #[cfg(not(debug_assertions))]
                {
                    // if the user didn't give any commands, run default action
                    //			if ( !com_dedicated->integer )
                    {
                        Cbuf_AddText(b"cinematic openinglogos\n\0".as_ptr() as *const c_char);
                        //				if( !com_introPlayed->integer ) {
                        //					Cvar_Set( com_introPlayed->name, "1" );
                        //					Cvar_Set( "nextmap", "cinematic intro" );
                        //				}
                    }
                }
                // #endif
            }
        }
        // #endif
        com_fullyInitialized = qtrue;
        Com_Printf(b"--- Common Initialization Complete ---\n\0".as_ptr() as *const c_char);

        //HACKERY FOR THE DEUTSCH
        //if ( (Cvar_VariableIntegerValue("ui_iscensored") == 1) 	//if this was on before, set it again so it gets its flags
        //	)
        //{
        //	Cvar_Get( "ui_iscensored",   "1", CVAR_ARCHIVE|CVAR_ROM|CVAR_INIT|CVAR_CHEAT|CVAR_NORESTART);
        //	Cvar_Set( "ui_iscensored",   "1");	//just in case it was archived
        //	// NOTE : I also create this in UI_Init()
        //	Cvar_Get( "g_dismemberment", "0", CVAR_ARCHIVE|CVAR_ROM|CVAR_INIT|CVAR_CHEAT);
        //	Cvar_Set( "g_dismemberment", "0");	//just in case it was archived
        //}
    }
    // C++: catch (const char* reason) {
    //     Sys_Error ("Error during initialization %s", reason);
    // }
    // (C++ exception catch not translated; throw -> panic! in callees)

    // #ifdef _XBOX
    //Load these early to keep them at the beginning of memory.  Perhaps
    //here is too early though.  After the license screen would be better.
    #[cfg(feature = "xbox")]
    {
        // extern void SE_CheckForLanguageUpdates(void);
        SE_CheckForLanguageUpdates();
    }
    // #endif
}

//==================================================================

pub unsafe fn Com_WriteConfigToFile(filename: *const c_char) {
    // #ifndef _XBOX
    #[cfg(not(feature = "xbox"))]
    {
        let f: fileHandle_t;

        f = FS_FOpenFileWrite(filename);
        if f == 0 {
            Com_Printf(
                b"Couldn't write %s.\n\0".as_ptr() as *const c_char,
                filename,
            );
            return;
        }

        FS_Printf(
            f,
            b"// generated by Star Wars Jedi Academy, do not modify\n\0".as_ptr()
                as *const c_char,
        );
        Key_WriteBindings(f);
        Cvar_WriteVariables(f);
        FS_FCloseFile(f);
    }
    // #endif
}


/*
===============
Com_WriteConfiguration

Writes key bindings and archived cvars to config file if modified
===============
*/
pub unsafe fn Com_WriteConfiguration() {
    // if we are quiting without fully initializing, make sure
    // we don't write out anything
    if com_fullyInitialized == qfalse {
        return;
    }

    if (cvar_modifiedFlags & CVAR_ARCHIVE) == 0 {
        return;
    }
    cvar_modifiedFlags &= !CVAR_ARCHIVE;

    Com_WriteConfigToFile(b"jaconfig.cfg\0".as_ptr() as *const c_char);
}


/*
===============
Com_WriteConfig_f

Write the config file to a specific name
===============
*/
pub unsafe fn Com_WriteConfig_f() {
    let mut filename: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];

    if Cmd_Argc() != 2 {
        Com_Printf(b"Usage: writeconfig <filename>\n\0".as_ptr() as *const c_char);
        return;
    }

    Q_strncpyz(
        filename.as_mut_ptr(),
        Cmd_Argv(1),
        core::mem::size_of_val(&filename) as c_int,
    );
    COM_DefaultExtension(
        filename.as_mut_ptr(),
        core::mem::size_of_val(&filename) as c_int,
        b".cfg\0".as_ptr() as *const c_char,
    );
    Com_Printf(b"Writing %s.\n\0".as_ptr() as *const c_char, filename.as_ptr());
    Com_WriteConfigToFile(filename.as_ptr());
}

/*
================
Com_ModifyMsec
================
*/

pub unsafe fn Com_ModifyMsec(mut msec: c_int, fraction: &mut c_float) -> c_int {
    let clampTime: c_int;

    *fraction = 0.0_f32;

    //
    // modify time for debugging values
    //
    if (*com_fixedtime).integer != 0 {
        msec = (*com_fixedtime).integer;
    } else if (*com_timescale).value != 0.0_f32 {
        *fraction = msec as c_float;
        *fraction *= (*com_timescale).value;
        // C: msec=(int)floor(fraction) -- floor takes double, fraction is float (promotion)
        msec = (*fraction as f64).floor() as c_int;
        *fraction -= msec as c_float;
    }

    // don't let it scale below 1 msec
    if msec < 1 {
        msec = 1;
        *fraction = 0.0_f32;
    }

    if (*com_skippingcin).integer != 0 {
        // we're skipping ahead so let it go a bit faster
        clampTime = 500;
    } else {
        // for local single player gaming
        // we may want to clamp the time to prevent players from
        // flying off edges when something hitches.
        clampTime = 200;
    }

    if msec > clampTime {
        msec = clampTime;
        *fraction = 0.0_f32;
    }

    msec
}

/*
=================
Com_Frame
=================
*/
static mut corg: vec3_t = [0.0_f32; 3];
static mut cangles: vec3_t = [0.0_f32; 3];
static mut bComma: bool = false;

pub unsafe fn Com_SetOrgAngles(org: vec3_t, angles: vec3_t) {
    VectorCopy(org, corg);
    VectorCopy(angles, cangles);
}

// #ifdef G2_PERFORMANCE_ANALYSIS
// void G2Time_ResetTimers(void);   (declared at module-level cfg block)
// void G2Time_ReportTimers(void);  (declared at module-level cfg block)
// #endif

// #pragma warning (disable: 4701)  //local may have been used without init (timing info vars)
pub unsafe fn Com_Frame() {
// try
{
    let mut timeBeforeFirstEvents: c_int = 0;
    let mut timeBeforeServer: c_int = 0;
    let mut timeBeforeEvents: c_int = 0;
    let mut timeBeforeClient: c_int = 0;
    let mut timeAfter: c_int = 0;
    let mut msec: c_int;
    let minMsec: c_int;
    static mut lastTime: c_int = 0;

    // write config file if anything changed
    // #ifndef _XBOX
    #[cfg(not(feature = "xbox"))]
    {
        Com_WriteConfiguration();

        // if "viewlog" has been modified, show or hide the log console
        if (*com_viewlog).modified != qfalse {
            Sys_ShowConsole((*com_viewlog).integer, qfalse);
            (*com_viewlog).modified = qfalse;
        }
    }
    // #endif

    //
    // main event loop
    //
    if (*com_speeds).integer != 0 {
        timeBeforeFirstEvents = Sys_Milliseconds();
    }

    // we may want to spin here if things are going too fast
    if (*com_maxfps).integer > 0 {
        minMsec = 1000 / (*com_maxfps).integer;
    } else {
        minMsec = 1;
    }
    loop {
        com_frameTime = Com_EventLoop();
        if lastTime > com_frameTime {
            lastTime = com_frameTime; // possible on first frame
        }
        msec = com_frameTime - lastTime;
        if msec >= minMsec {
            break;
        }
    }
    Cbuf_Execute();

    lastTime = com_frameTime;

    // mess with msec if needed
    com_frameMsec = msec;
    let mut fractionMsec: c_float = 0.0_f32;
    msec = Com_ModifyMsec(msec, &mut fractionMsec);

    //
    // server side
    //
    if (*com_speeds).integer != 0 {
        timeBeforeServer = Sys_Milliseconds();
    }

    SV_Frame(msec, fractionMsec);


    //
    // client system
    //
    // #ifdef _XBOX
    //	extern void G_DemoFrame();
    //
    //	G_DemoFrame();
    #[cfg(feature = "xbox")]
    {
        // extern bool TestDemoTimer();  (declared at module-level cfg block)
        // extern void PlayDemo();       (declared at module-level cfg block)
        if TestDemoTimer() {
            PlayDemo();
        }
    }
    // #endif

    //	if ( !com_dedicated->integer )
    {
        //
        // run event loop a second time to get server to client packets
        // without a frame of latency
        //
        if (*com_speeds).integer != 0 {
            timeBeforeEvents = Sys_Milliseconds();
        }
        Com_EventLoop();
        Cbuf_Execute();


        //
        // client side
        //
        if (*com_speeds).integer != 0 {
            timeBeforeClient = Sys_Milliseconds();
        }

        CL_Frame(msec, fractionMsec);

        if (*com_speeds).integer != 0 {
            timeAfter = Sys_Milliseconds();
        }
    }


    //
    // report timing information
    //
    if (*com_speeds).integer != 0 {
        let all: c_int;
        let sv: c_int;
        let ev: c_int;
        let cl: c_int;

        let all = timeAfter - timeBeforeServer;
        let sv = timeBeforeEvents - timeBeforeServer;
        let ev = timeBeforeServer - timeBeforeFirstEvents
            + timeBeforeClient - timeBeforeEvents;
        let cl = timeAfter - timeBeforeClient;
        let sv = sv - time_game;
        let cl = cl - time_frontend - time_backend;

        Com_Printf(
            b"fr:%i all:%3i sv:%3i ev:%3i cl:%3i gm:%3i tr:%3i pvs:%3i rf:%3i bk:%3i\n\0"
                .as_ptr() as *const c_char,
            com_frameNumber,
            all,
            sv,
            ev,
            cl,
            time_game,
            timeInTrace,
            timeInPVSCheck,
            time_frontend,
            time_backend,
        );

        // #ifndef _XBOX
        #[cfg(not(feature = "xbox"))]
        {
            // speedslog
            if !com_speedslog.is_null() && (*com_speedslog).integer != 0 {
                if speedslog == 0 {
                    speedslog = FS_FOpenFileWrite(
                        b"speeds.log\0".as_ptr() as *const c_char,
                    );
                    FS_Write(
                        b"data={\n\0".as_ptr() as *const c_void,
                        strlen(b"data={\n\0".as_ptr() as *const c_char),
                        speedslog,
                    );
                    bComma = false;
                    if (*com_speedslog).integer > 1 {
                        // force it to not buffer so we get valid
                        // data even if we are crashing
                        FS_ForceFlush(logfile);
                    }
                }
                if speedslog != 0 {
                    let mut msg: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];

                    if bComma {
                        FS_Write(
                            b",\n\0".as_ptr() as *const c_void,
                            strlen(b",\n\0".as_ptr() as *const c_char),
                            speedslog,
                        );
                        bComma = false;
                    }
                    FS_Write(
                        b"{\0".as_ptr() as *const c_void,
                        strlen(b"{\0".as_ptr() as *const c_char),
                        speedslog,
                    );
                    Com_sprintf(
                        msg.as_mut_ptr(),
                        core::mem::size_of_val(&msg) as c_int,
                        b"%8.4f,%8.4f,%8.4f,%8.4f,%8.4f,%8.4f,\0".as_ptr() as *const c_char,
                        corg[0] as f64,
                        corg[1] as f64,
                        corg[2] as f64,
                        cangles[0] as f64,
                        cangles[1] as f64,
                        cangles[2] as f64,
                    );
                    FS_Write(msg.as_ptr() as *const c_void, strlen(msg.as_ptr()), speedslog);
                    Com_sprintf(
                        msg.as_mut_ptr(),
                        core::mem::size_of_val(&msg) as c_int,
                        b"%i,%3i,%3i,%3i,%3i,%3i,%3i,%3i,%3i,%3i}\0".as_ptr() as *const c_char,
                        com_frameNumber,
                        all,
                        sv,
                        ev,
                        cl,
                        time_game,
                        timeInTrace,
                        timeInPVSCheck,
                        time_frontend,
                        time_backend,
                    );
                    FS_Write(msg.as_ptr() as *const c_void, strlen(msg.as_ptr()), speedslog);
                    bComma = true;
                }
            }
        }
        // #endif

        timeInTrace = 0;
        timeInPVSCheck = 0;
    }

    //
    // trace optimization tracking
    //
    if (*com_showtrace).integer != 0 {
        // extern int c_traces, c_brush_traces, c_patch_traces;  (declared at module level)
        // extern int c_pointcontents;                            (declared at module level)

        /*
        Com_Printf( "%4i non-sv_traces, %4i sv_traces, %4i ms, ave %4.2f ms\n", c_traces - numTraces, numTraces, timeInTrace, (float)timeInTrace/(float)numTraces );
        timeInTrace = numTraces = 0;
        c_traces = 0;
        */

        Com_Printf(
            b"%4i traces  (%ib %ip) %4i points\n\0".as_ptr() as *const c_char,
            c_traces,
            c_brush_traces,
            c_patch_traces,
            c_pointcontents,
        );
        c_traces = 0;
        c_brush_traces = 0;
        c_patch_traces = 0;
        c_pointcontents = 0;
    }

    com_frameNumber += 1;
}//try
    // catch (const char* reason) {
    //     Com_Printf(reason);
    //     return;  // an ERR_DROP was thrown
    // }
    // (C++ exception catch not translated; throw -> panic! in callees)

    // #ifdef G2_PERFORMANCE_ANALYSIS
    #[cfg(feature = "g2_performance_analysis")]
    {
        if !com_G2Report.is_null() && (*com_G2Report).integer != 0 {
            G2Time_ReportTimers();
        }

        G2Time_ResetTimers();
    }
    // #endif
}

// #pragma warning (default: 4701)  //local may have been used without init

/*
=================
Com_Shutdown
=================
*/
// extern void CM_FreeShaderText(void);  (declared at module-level extern block)
pub unsafe fn Com_Shutdown() {
    CM_ClearMap();

    // #ifndef _XBOX
    #[cfg(not(feature = "xbox"))]
    {
        CM_FreeShaderText();

        if logfile != 0 {
            FS_FCloseFile(logfile);
            logfile = 0;
        }

        if speedslog != 0 {
            FS_Write(
                b"\n};\0".as_ptr() as *const c_void,
                strlen(b"\n};\0".as_ptr() as *const c_char),
                speedslog,
            );
            FS_FCloseFile(speedslog);
            speedslog = 0;
        }

        if camerafile != 0 {
            FS_FCloseFile(camerafile);
            camerafile = 0;
        }

        if com_journalFile != 0 {
            FS_FCloseFile(com_journalFile);
            com_journalFile = 0;
        }
    }
    // #endif

    // #ifdef _XBOX
    #[cfg(feature = "xbox")]
    {
        // extern void Sys_StreamShutdown();  (declared at module-level cfg block)
        Sys_StreamShutdown();
        Sys_ShutdownFileCodes();
    }
    // #endif

    SE_ShutDown(); //close the string packages

    // extern void Netchan_Shutdown();  (declared at module-level extern block)
    Netchan_Shutdown();
}

/*
============
ParseTextFile
============
*/

// C++ overload 1: bool Com_ParseTextFile(const char *file, class CGenericParser2 &parser, bool cleanFirst)
// C++ overload 2: CGenericParser2 *Com_ParseTextFile(const char *file, bool cleanFirst, bool writeable)
// Rust does not support function overloading; overload 2 is renamed Com_ParseTextFile_create.
// (dedup note: two C++ overloads with same name, different signatures — renamed second in Rust)

pub unsafe fn Com_ParseTextFile(
    file: *const c_char,
    parser: &mut CGenericParser2,
    cleanFirst: bool,
) -> bool {
    let mut f: fileHandle_t = 0;
    let mut length: c_int = 0;
    let mut buf: *mut c_char = core::ptr::null_mut();
    let mut bufParse: *mut c_char = core::ptr::null_mut();

    length = FS_FOpenFileByMode(file, &mut f, FS_READ);
    if f == 0 || length == 0 {
        return false;
    }

    // C++: buf = new char [length + 1];
    // Faithful: allocate a zeroed buffer via Vec, leak into raw pointer for C-style ownership
    let mut buf_vec: alloc::vec::Vec<c_char> =
        alloc::vec![0; (length + 1) as usize];
    buf = buf_vec.as_mut_ptr();
    FS_Read(buf as *mut c_void, length, f);
    *buf.add(length as usize) = 0;

    bufParse = buf;
    parser.Parse(&mut bufParse, cleanFirst);
    // C++: delete buf;  (buf_vec drops here, freeing the allocation)
    drop(buf_vec);

    FS_FCloseFile(f);

    true
}

pub unsafe fn Com_ParseTextFileDestroy(parser: &mut CGenericParser2) {
    parser.Clean();
}

// C++ overload 2 (renamed): CGenericParser2 *Com_ParseTextFile(const char *file, bool cleanFirst, bool writeable)
pub unsafe fn Com_ParseTextFile_create(
    file: *const c_char,
    cleanFirst: bool,
    writeable: bool,
) -> *mut CGenericParser2 {
    let mut f: fileHandle_t = 0;
    let mut length: c_int = 0;
    let mut buf: *mut c_char = core::ptr::null_mut();
    let mut bufParse: *mut c_char = core::ptr::null_mut();
    let mut parse: *mut CGenericParser2;

    length = FS_FOpenFileByMode(file, &mut f, FS_READ);
    if f == 0 || length == 0 {
        return core::ptr::null_mut();
    }

    // C++: buf = new char [length + 1];
    let mut buf_vec: alloc::vec::Vec<c_char> =
        alloc::vec![0; (length + 1) as usize];
    buf = buf_vec.as_mut_ptr();
    FS_Read(buf as *mut c_void, length, f);
    FS_FCloseFile(f);
    *buf.add(length as usize) = 0;

    bufParse = buf;

    // C++: parse = new CGenericParser2;
    let mut parse_box = alloc::boxed::Box::new(CGenericParser2::new());
    parse = alloc::boxed::Box::into_raw(parse_box);
    if !(*parse).Parse(&mut bufParse, cleanFirst, writeable) {
        // C++: delete parse;
        let _ = alloc::boxed::Box::from_raw(parse);
        parse = core::ptr::null_mut();
    }

    // C++: delete buf;  (buf_vec drops here)
    drop(buf_vec);

    parse
}
