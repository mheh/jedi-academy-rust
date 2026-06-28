#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void, c_uint};
use std::ptr::{addr_of, addr_of_mut, null_mut};
use std::mem::zeroed;

// Opaque type stubs for game engine types
#[repr(C)]
pub struct cvar_t {
    // Quake 3 cvar_t has layout: name*, string*, resetString*, latchedString*, flags, modified, value, ...
    // We only care about the value field which is typically a float, placed after several pointers and ints
    _pads: [*const c_char; 4],  // name, string, resetString, latchedString pointers
    _flags: c_int,
    _modified: c_int,
    pub value: f32,  // The field we access in this code
}

#[repr(C)]
pub struct refexport_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct sysEvent_t {
    pub evTime: c_int,
    pub evType: c_int,
    pub evValue: c_int,
    pub evValue2: c_int,
    pub evPtrLength: c_int,
    pub evPtr: *mut c_void,
}

#[repr(C)]
pub struct fileHandle_t {
    _opaque: c_int,
}

#[repr(C)]
pub struct netadr_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct msg_t {
    _opaque: [u8; 0],
}

pub type qboolean = c_int;
pub type byte = u8;
pub type uid_t = u32;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

const MAX_OSPATH: usize = 256;
const MAX_MSGLEN: usize = 16384;
const TAG_FILESYS: c_int = 0;
const TAG_EVENT: c_int = 1;

const ERR_FATAL: c_int = 1;
const SE_CONSOLE: c_int = 1;
const SE_PACKET: c_int = 2;

// Structure containing functions exported from refresh DLL
pub static mut re: refexport_t = unsafe { zeroed() };

pub static mut sys_frame_time: c_uint = 0;

pub static mut saved_euid: uid_t = 0;
pub static mut stdin_active: qboolean = qtrue;

pub static mut nostdout: *mut cvar_t = null_mut();

// =======================================================================
// General routines
// =======================================================================

// bk001207
const MEM_THRESHOLD: usize = 96 * 1024 * 1024;

/*
==================
Sys_LowPhysicalMemory()
==================
*/
pub fn Sys_LowPhysicalMemory() -> qboolean {
    // MEMORYSTATUS stat;
    // GlobalMemoryStatus (&stat);
    // return (stat.dwTotalPhys <= MEM_THRESHOLD) ? qtrue : qfalse;
    qfalse  // bk001207 - FIXME
}

/*
==================
Sys_FunctionCmp
==================
*/
pub fn Sys_FunctionCmp(f1: *mut c_void, f2: *mut c_void) -> c_int {
    qtrue
}

/*
==================
Sys_FunctionCheckSum
==================
*/
pub fn Sys_FunctionCheckSum(f1: *mut c_void) -> c_int {
    0
}

/*
==================
Sys_MonkeyShouldBeSpanked
==================
*/
pub fn Sys_MonkeyShouldBeSpanked() -> c_int {
    0
}

pub fn Sys_BeginProfiling() {
}

/*
=================
Sys_In_Restart_f

Restart the input subsystem
=================
*/
pub fn Sys_In_Restart_f() {
    IN_Shutdown();
    IN_Init();
}

pub fn Sys_ConsoleOutput(string: *const c_char) {
    unsafe {
        if !nostdout.is_null() && (*nostdout).value != 0 {
            return;
        }
    }

    unsafe {
        libc::fputs(string, libc::stdout);
    }
}

pub extern "C" fn Sys_Printf(fmt: *const c_char, ...) {
    // Rust cannot safely implement variadic functions.
    // This is declared for C callers but the body is stubbed.
    // Calls from Rust should use Com_Printf directly instead.
}

// bk010104 - added for abstraction
pub fn Sys_Exit(ex: c_int) {
    #[cfg(not(debug_assertions))]
    {
        // We can't do this
        // as long as GL DLL's keep installing with atexit...
        // exit(ex);
        unsafe { libc::_exit(ex); }
    }

    #[cfg(debug_assertions)]
    {
        // Give me a backtrace on error exits.
        debug_assert_eq!(ex, 0);
        unsafe { libc::exit(ex); }
    }
}

pub fn Sys_Quit() {
    CL_Shutdown();
    unsafe {
        libc::fcntl(0, libc::F_SETFL, libc::fcntl(0, libc::F_GETFL, 0) & !libc::O_NONBLOCK);
    }
    Sys_Exit(0);
}

pub fn Sys_Init() {
    Cmd_AddCommand(b"in_restart\0".as_ptr() as *const c_char, Sys_In_Restart_f);

    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86")]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"linux i386\0".as_ptr() as *const c_char);
        }

        #[cfg(target_arch = "alpha")]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"linux alpha\0".as_ptr() as *const c_char);
        }

        #[cfg(target_arch = "sparc")]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"linux sparc\0".as_ptr() as *const c_char);
        }

        #[cfg(target_os = "freebsd")]
        {
            #[cfg(target_arch = "x86")]
            {
                Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"freebsd i386\0".as_ptr() as *const c_char);
            }

            #[cfg(target_arch = "alpha")]
            {
                Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"freebsd alpha\0".as_ptr() as *const c_char);
            }

            #[cfg(not(any(target_arch = "x86", target_arch = "alpha")))]
            {
                Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"freebsd unknown\0".as_ptr() as *const c_char);
            }
        }

        #[cfg(not(target_os = "freebsd"))]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"linux unknown\0".as_ptr() as *const c_char);
        }
    }

    #[cfg(target_os = "solaris")]
    {
        #[cfg(target_arch = "x86")]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"solaris x86\0".as_ptr() as *const c_char);
        }

        #[cfg(target_arch = "sparc")]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"solaris sparc\0".as_ptr() as *const c_char);
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "sparc")))]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"solaris unknown\0".as_ptr() as *const c_char);
        }
    }

    #[cfg(target_os = "irix")]
    {
        #[cfg(target_arch = "mips")]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"sgi mips\0".as_ptr() as *const c_char);
        }

        #[cfg(not(target_arch = "mips"))]
        {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"sgi unknown\0".as_ptr() as *const c_char);
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "solaris", target_os = "irix")))]
    {
        Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"unknown\0".as_ptr() as *const c_char);
    }

    Cvar_Set(b"username\0".as_ptr() as *const c_char, Sys_GetCurrentUser());

    IN_Init();
}

pub extern "C" fn Sys_Error(error: *const c_char, ...) {
    // change stdin to non blocking
    unsafe {
        libc::fcntl(0, libc::F_SETFL, libc::fcntl(0, libc::F_GETFL, 0) & !libc::O_NONBLOCK);
    }

    CL_Shutdown();

    unsafe {
        libc::fprintf(libc::stderr, b"Sys_Error: %s\n\0".as_ptr() as *const c_char, error);
    }

    Sys_Exit(1); // bk010104 - use single exit point.
}

pub extern "C" fn Sys_Warn(warning: *const c_char, ...) {
    unsafe {
        libc::fprintf(libc::stderr, b"Warning: %s\0".as_ptr() as *const c_char, warning);
    }
}

/*
============
Sys_FileTime

returns -1 if not present
============
*/
pub fn Sys_FileTime(path: *const c_char) -> c_int {
    let mut buf: libc::stat = unsafe { zeroed() };

    unsafe {
        if libc::stat(path, &mut buf) == -1 {
            return -1;
        }

        buf.st_mtime as c_int
    }
}

pub extern "C" fn floating_point_exception_handler(_whatever: c_int) {
    unsafe {
        libc::signal(libc::SIGFPE, floating_point_exception_handler as libc::sighandler_t);
    }
}

pub fn Sys_ConsoleInput() -> *mut c_char {
    static mut text: [c_char; 256] = [0; 256];
    let mut len: c_int;
    let mut fdset: libc::fd_set = unsafe { zeroed() };
    let mut timeout: libc::timeval = unsafe { zeroed() };

    unsafe {
        if com_dedicated.is_null() || (*com_dedicated).value == 0 {
            return null_mut();
        }

        if stdin_active == 0 {
            return null_mut();
        }

        libc::FD_ZERO(&mut fdset);
        libc::FD_SET(0, &mut fdset); // stdin
        timeout.tv_sec = 0;
        timeout.tv_usec = 0;
        if libc::select(1, &mut fdset, null_mut(), null_mut(), &mut timeout) == -1
            || libc::FD_ISSET(0, &fdset) == 0
        {
            return null_mut();
        }

        len = libc::read(0, text.as_mut_ptr() as *mut c_void, std::mem::size_of_val(&text)) as c_int;
        if len == 0 {
            // eof!
            stdin_active = qfalse;
            return null_mut();
        }

        if len < 1 {
            return null_mut();
        }
        text[(len - 1) as usize] = 0; // rip off the /n and terminate

        text.as_mut_ptr()
    }
}

/*****************************************************************************/

/*
=================
Sys_UnloadDll

=================
*/
pub fn Sys_UnloadDll(dllHandle: *mut c_void) {
    // bk001206 - verbose error reporting
    let err: *const c_char; // rb010123 - now const
    if dllHandle.is_null() {
        Com_Printf(b"Sys_UnloadDll(NULL)\n\0".as_ptr() as *const c_char);
        return;
    }
    unsafe {
        libc::dlclose(dllHandle);
        err = libc::dlerror();
        if !err.is_null() {
            Com_Printf("Sys_UnloadGame failed on dlclose: \"%s\b"!\n\0".as_ptr() as *const c_char, err);
        }
    }
}

/*
=================
Sys_LoadDll

Used to load a development dll instead of a virtual machine
=================
*/
extern "C" {
    pub fn FS_BuildOSPath(base: *const c_char, game: *const c_char, qpath: *const c_char) -> *const c_char;
}

pub fn Sys_LoadDll(
    name: *const c_char,
    entryPoint: *mut *mut c_int,
    systemcalls: *mut c_int,
) -> *mut c_void {
    let mut libHandle: *mut c_void;
    let mut dllEntry: unsafe extern "C" fn(*mut c_int) = unsafe { zeroed() };
    let mut curpath: [c_char; MAX_OSPATH] = unsafe { zeroed() };
    let mut fname: [c_char; MAX_OSPATH] = unsafe { zeroed() };
    // char	loadname[MAX_OSPATH];
    let basepath: *const c_char;
    let cdpath: *const c_char;
    let gamedir: *const c_char;
    let mut fn_: *const c_char;
    let mut err: *const c_char = null_mut(); // bk001206 // rb0101023 - now const

    // bk001206 - let's have some paranoia
    debug_assert!(!name.is_null());

    unsafe {
        libc::getcwd(curpath.as_mut_ptr(), std::mem::size_of_val(&curpath));

        #[cfg(target_arch = "x86")]
        {
            #[cfg(debug_assertions)]
            {
                libc::snprintf(
                    fname.as_mut_ptr(),
                    std::mem::size_of_val(&fname),
                    b"%si386-debug.so\0".as_ptr() as *const c_char,
                    name,
                ); // bk010205 - different DLL name
            }

            #[cfg(not(debug_assertions))]
            {
                libc::snprintf(
                    fname.as_mut_ptr(),
                    std::mem::size_of_val(&fname),
                    b"%si386.so\0".as_ptr() as *const c_char,
                    name,
                );
            }
        }

        #[cfg(target_arch = "powerpc")]
        {
            libc::snprintf(
                fname.as_mut_ptr(),
                std::mem::size_of_val(&fname),
                b"%sppc.so\0".as_ptr() as *const c_char,
                name,
            ); // rcg010207 - PPC support.
        }

        #[cfg(target_arch = "alpha")]
        {
            libc::snprintf(
                fname.as_mut_ptr(),
                std::mem::size_of_val(&fname),
                b"%saxp.so\0".as_ptr() as *const c_char,
                name,
            );
        }

        #[cfg(target_arch = "mips")]
        {
            libc::snprintf(
                fname.as_mut_ptr(),
                std::mem::size_of_val(&fname),
                b"%smips.so\0".as_ptr() as *const c_char,
                name,
            );
        }

        // bk001129 - was RTLD_LAZY
        const Q_RTLD: c_int = libc::RTLD_NOW;

        // if 0 // bk010205 - was NDEBUG // bk001129 - FIXME: what is this good for?
        //   // bk001206 - do not have different behavior in builds
        //   Q_strncpyz(loadname, curpath, sizeof(loadname));
        //   // bk001129 - from cvs1.17 (mkv)
        //   Q_strcat(loadname, sizeof(loadname), "/");
        //
        //   Q_strcat(loadname, sizeof(loadname), fname);
        //   Com_Printf( "Sys_LoadDll(%s)... \n", loadname );
        //   libHandle = dlopen( loadname, Q_RTLD );
        //   //if ( !libHandle ) {
        //   // bk001206 - report any problem
        //   //Com_Printf( "Sys_LoadDll(%s) failed: \"%s\"\n", loadname, dlerror() );
        // #endif // bk010205 - do not load from installdir

        basepath = Cvar_VariableString(b"fs_basepath\0".as_ptr() as *const c_char);
        cdpath = Cvar_VariableString(b"fs_cdpath\0".as_ptr() as *const c_char);
        gamedir = Cvar_VariableString(b"fs_game\0".as_ptr() as *const c_char);

        fn_ = FS_BuildOSPath(basepath, gamedir, fname.as_ptr());
        // bk001206 - verbose
        Com_Printf(b"Sys_LoadDll(%s)... \n\0".as_ptr() as *const c_char, fn_);

        // bk001129 - from cvs1.17 (mkv), was fname not fn
        libHandle = libc::dlopen(fn_, Q_RTLD);

        #[cfg(not(debug_assertions))]
        {
            if libHandle.is_null() {
                Com_Printf(b"Failed to open DLL\n\0".as_ptr() as *const c_char);
            }
        }

        if libHandle.is_null() {
            let cdpath_ptr = cdpath as *const u8;
            if !cdpath_ptr.is_null() && *cdpath_ptr != 0 {
                // bk001206 - report any problem
                Com_Printf("Sys_LoadDll(%s) failed: \"%s\b"\n\0".as_ptr() as *const c_char, fn_, libc::dlerror());

                fn_ = FS_BuildOSPath(cdpath, gamedir, fname.as_ptr());
                libHandle = libc::dlopen(fn_, Q_RTLD);
                if libHandle.is_null() {
                    // bk001206 - report any problem
                    Com_Printf("Sys_LoadDll(%s) failed: \"%s\b"\n\0".as_ptr() as *const c_char, fn_, libc::dlerror());
                } else {
                    Com_Printf(b"Sys_LoadDll(%s): succeeded ...\n\0".as_ptr() as *const c_char, fn_);
                }
            } else {
                Com_Printf(b"Sys_LoadDll(%s): succeeded ...\n\0".as_ptr() as *const c_char, fn_);
            }

            if libHandle.is_null() {
                #[cfg(not(debug_assertions))]
                {
                    Com_Error(ERR_FATAL, b"Sys_LoadDll(%s) failed dlopen() completely!\n\0".as_ptr() as *const c_char, name);
                }

                #[cfg(debug_assertions)]
                {
                    Com_Printf(b"Sys_LoadDll(%s) failed dlopen() completely!\n\0".as_ptr() as *const c_char, name);
                }

                return null_mut();
            }
        }
        // bk001206 - no different behavior
        // #ifndef NDEBUG }
        // else Com_Printf ( "Sys_LoadDll(%s): succeeded ...\n", loadname );
        // #endif

        dllEntry = std::mem::transmute(libc::dlsym(libHandle, b"dllEntry\0".as_ptr() as *const c_char));
        if dllEntry as *const c_void == null_mut() {
            err = libc::dlerror();
            Com_Printf("Sys_LoadDLL(%s) failed dlsym(dllEntry): \"%s\b" ! \n\0".as_ptr() as *const c_char, name, err);
        }
        // int vmMain( int command, int arg0, int arg1, int arg2, int arg3, int arg4, int arg5, int arg6, int arg7, int arg8, int arg9, int arg10, int arg11  )
        *entryPoint = std::mem::transmute(libc::dlsym(libHandle, b"vmMain\0".as_ptr() as *const c_char));
        if (*entryPoint).is_null() {
            err = libc::dlerror();
        }
        if (*entryPoint).is_null() || dllEntry as *const c_void == null_mut() {
            #[cfg(not(debug_assertions))]
            {
                Com_Error(ERR_FATAL, "Sys_LoadDll(%s) failed dlsym(vmMain): \"%s\b" !\n\0".as_ptr() as *const c_char, name, err);
            }

            #[cfg(debug_assertions)]
            {
                Com_Printf("Sys_LoadDll(%s) failed dlsym(vmMain): \"%s\b" !\n\0".as_ptr() as *const c_char, name, err);
            }

            libc::dlclose(libHandle);
            err = libc::dlerror();
            if !err.is_null() {
                Com_Printf("Sys_LoadDll(%s) failed dlcose: \"%s\b"\n\0".as_ptr() as *const c_char, name, err);
            }
            return null_mut();
        }
        Com_Printf(b"Sys_LoadDll(%s) found **vmMain** at  %p  \n\0".as_ptr() as *const c_char, name, *entryPoint); // bk001212
        dllEntry(systemcalls as *mut c_int);
        Com_Printf(b"Sys_LoadDll(%s) succeeded!\n\0".as_ptr() as *const c_char, name);
        libHandle
    }
}

// if 0 // bk010215 - scheduled for full deletion
// /*****************************************************************************/
//
// static void *game_library;
//
// #ifdef __i386__
//     const char *gamename = "qagamei386.so";
// #elif defined __alpha__
//     const char *gamename = "qagameaxp.so";
// #elif defined __mips__
//     const char *gamename = "qagamemips.so";
// #else
// #error Unknown arch
// #endif
//
// /*
// =================
// Sys_UnloadGame
// =================
// */
// void Sys_UnloadGame (void) {
//   // bk001206 - this code is never used
//   assert(0);
//
//   Com_Printf("------ Unloading %s ------\n", gamename);
//   if (game_library) {
//     dlclose (game_library);
//     game_library = NULL;
//   }
// }
//
// /*
// =================
// Sys_GetGameAPI
//
// Loads the game dll
// =================
// */
// void *Sys_GetGameAPI (void *parms)
// {
//     void    *(*GetGameAPI) (void *);
//
//     char    name[MAX_OSPATH];
//     char    curpath[MAX_OSPATH];
//     //char    *path; // bk001204 - unused
//
//   // bk001206 - this code is never used
//   assert(0);
//
//     if (game_library)
//         Com_Error (ERR_FATAL, "Sys_GetGameAPI without Sys_UnloadingGame");
//
//     // check the current debug directory first for development purposes
//     getcwd(curpath, sizeof(curpath));
//
//     Com_Printf("------- Loading %s -------\n", gamename);
//     Com_sprintf (name, sizeof(name), "%s/%s", curpath, gamename);
//
//     game_library = dlopen (name, RTLD_LAZY );
//     if (game_library)
//         Com_DPrintf ("LoadLibrary (%s)\n",name);
//     else {
//         Com_Printf( "LoadLibrary(\"%s\") failed\n", name);
//         Com_Printf( "...reason: '%s'\n", dlerror() );
//         Com_Error( ERR_FATAL, "Couldn't load game" );
//     }
//
//     GetGameAPI = (void *)dlsym (game_library, "GetGameAPI");
//     if (!GetGameAPI)
//     {
//         Sys_UnloadGame ();
//         return NULL;
//     }
//
//     return GetGameAPI (parms);
// }
//
// /*****************************************************************************/
//
// static void *cgame_library;
//
// /*
// =================
// Sys_UnloadGame
// =================
// */
// void Sys_UnloadCGame (void)
// {
//   // bk001206 - this code is never used
//   assert(0);
//     if (cgame_library)
//         dlclose (cgame_library);
//     cgame_library = NULL;
// }
//
// /*
// =================
// Sys_GetGameAPI
//
// Loads the game dll
// =================
// */
// void *Sys_GetCGameAPI (void)
// {
//     void    *(*api) (void);
//
//     char    name[MAX_OSPATH];
//     char    curpath[MAX_OSPATH];
// #ifdef __i386__
//     const char *cgamename = "cgamei386.so";
// #elif defined __alpha__
//     const char *cgamename = "cgameaxp.so";
// #elif defined __mips__
//     const char *cgamename = "cgamemips.so";
// #else
// #error Unknown arch
// #endif
//
//   // bk001206 - this code is never used
//   assert(0);
//
//     Sys_UnloadCGame();
//
//     getcwd(curpath, sizeof(curpath));
//
//     Com_Printf("------- Loading %s -------\n", cgamename);
//
//     sprintf (name, "%s/%s", curpath, cgamename);
//     cgame_library = dlopen (name, RTLD_LAZY );
//     if (!cgame_library)
//     {
//         Com_Printf ("LoadLibrary (%s)\n",name);
//         Com_Error( ERR_FATAL, "Couldn't load cgame: %s", dlerror() );
//     }
//
//     api = (void *)dlsym (cgame_library, "GetCGameAPI");
//     if (!api)
//     {
//         Com_Error( ERR_FATAL, "dlsym() failed on GetCGameAPI" );
//     }
//
//     return api();
// }
//
// /*****************************************************************************/
//
// static void *ui_library;
//
// /*
// =================
// Sys_UnloadUI
// =================
// */
// void Sys_UnloadUI(void)
// {
//   // bk001206 - this code is never used
//   assert(0);
//     if (ui_library)
//         dlclose (ui_library);
//     ui_library = NULL;
// }
//
// /*
// =================
// Sys_GetUIAPI
//
// Loads the ui dll
// =================
// */
// void *Sys_GetUIAPI (void)
// {
//     void    *(*api)(void);
//
//     char    name[MAX_OSPATH];
//     char    curpath[MAX_OSPATH];
// #ifdef __i386__
//     const char *uiname = "uii386.so";
// #elif defined __alpha__
//     const char *uiname = "uiaxp.so";
// #elif defined __mips__
//     const char *uiname = "uimips.so";
// #else
// #error Unknown arch
// #endif
//
//     // bk001206 - this code is never used
//     assert(0);
//     Sys_UnloadUI();
//
//     getcwd(curpath, sizeof(curpath));
//
//     Com_Printf("------- Loading %s -------\n", uiname);
//
//     sprintf (name, "%s/%s", curpath, uiname);
//     ui_library = dlopen (name, RTLD_LAZY );
//     if (!ui_library)
//     {
//         Com_Printf ("LoadLibrary (%s)\n",name);
//         Com_Error( ERR_FATAL, "Couldn't load ui: %s", dlerror() );
//     }
//
//     api = (void *(*)(void))dlsym (ui_library, "GetUIAPI");
//     if (!api)
//     {
//         Com_Error( ERR_FATAL, "dlsym() failed on GetUIAPI" );
//     }
//
//     return api();
// }
//
// /*****************************************************************************/
//
// static void *botlib_library;
//
// /*
// =================
// Sys_UnloadGame
// =================
// */
// void Sys_UnloadBotLib (void)
// {
//   // bk001206 - this code is never used
//   assert(0);
//     if (botlib_library)
//         dlclose (botlib_library);
//     botlib_library = NULL;
// }
//
// /*
// =================
// Sys_GetGameAPI
//
// Loads the game dll
// =================
// */
// void *Sys_GetBotLibAPI (void *parms )
// {
//     void    *(*GetBotLibAPI) (void *);
//     char    name[MAX_OSPATH];
//     char    curpath[MAX_OSPATH];
// #ifdef __i386__
//     const char *botlibname = "qaboti386.so";
// #elif defined __alpha__
//     const char *botlibname = "qabotaxp.so";
// #elif defined __mips__
//     const char *botlibname = "qabotmips.so";
// #else
// #error Unknown arch
// #endif
//     // bk001129 - this code is never used
//     assert(0);
//
//     Sys_UnloadBotLib();
//
//     getcwd(curpath, sizeof(curpath));
//
//     Com_Printf("------- Loading %s -------\n", botlibname);
//
//     sprintf (name, "%s/%s", curpath, botlibname);
//     // bk001129 - was  RTLD_LAZY
//     botlib_library = dlopen (name, RTLD_NOW );
//     if (!botlib_library)
//     {
//         Com_Printf ("LoadLibrary (%s)\n",name);
//         Com_Error( ERR_FATAL, "Couldn't load botlib: %s", dlerror() );
//     }
//
//     GetBotLibAPI = (void *)dlsym (botlib_library, "GetBotLibAPI");
//     if (!GetBotLibAPI)
//     {
//         Sys_UnloadBotLib ();
//         Com_Error( ERR_FATAL, "dlsym() failed on GetBotLibAPI" );
//     }
//
//     // bk001129 - this is a signature mismatch
//     return GetBotLibAPI (parms);
// }
//
// void *Sys_GetBotAIAPI (void *parms ) {
//     return NULL;
// }
//
// /*****************************************************************************/
// #endif // bk010215

/*
========================================================================

BACKGROUND FILE STREAMING

========================================================================
*/

// if 1
pub fn Sys_InitStreamThread() {
}

pub fn Sys_ShutdownStreamThread() {
}

pub fn Sys_BeginStreamedFile(f: c_int, _readAhead: c_int) {
}

pub fn Sys_EndStreamedFile(f: c_int) {
}

pub fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: c_int) -> c_int {
    FS_Read(buffer, size * count, f)
}

pub fn Sys_StreamSeek(f: c_int, offset: c_int, origin: c_int) {
    FS_Seek(f, offset, origin);
}

// else
// typedef struct {
//     fileHandle_t file;
//     byte    *buffer;
//     qboolean    eof;
//     int     bufferSize;
//     int     streamPosition;   // next byte to be returned by Sys_StreamRead
//     int     threadPosition;   // next byte to be read from file
// } streamState_t;
//
// streamState_t   stream;
//
// /*
// ===============
// Sys_StreamThread
//
// A thread will be sitting in this loop forever
// ================
// */
// void Sys_StreamThread( void )
// {
//     int     buffer;
//     int     count;
//     int     readCount;
//     int     bufferPoint;
//     int     r;
//
//     // if there is any space left in the buffer, fill it up
//     if ( !stream.eof ) {
//         count = stream.bufferSize - (stream.threadPosition - stream.streamPosition);
//         if ( count ) {
//             bufferPoint = stream.threadPosition % stream.bufferSize;
//             buffer = stream.bufferSize - bufferPoint;
//             readCount = buffer < count ? buffer : count;
//             r = FS_Read ( stream.buffer + bufferPoint, readCount, stream.file );
//             stream.threadPosition += r;
//
//             if ( r != readCount )
//                 stream.eof = qtrue;
//         }
//     }
// }
//
// /*
// ===============
// Sys_InitStreamThread
//
// ================
// */
// void Sys_InitStreamThread( void )
// {
// }
//
// /*
// ===============
// Sys_ShutdownStreamThread
//
// ================
// */
// void Sys_ShutdownStreamThread( void )
// {
// }
//
//
// /*
// ===============
// Sys_BeginStreamedFile
//
// ================
// */
// void Sys_BeginStreamedFile( fileHandle_t f, int readAhead )
// {
//     if ( stream.file ) {
//         Com_Error( ERR_FATAL, "Sys_BeginStreamedFile: unclosed stream");
//     }
//
//     stream.file = f;
//     stream.buffer = Z_Malloc( readAhead,TAG_FILESYS,qfalse );
//     stream.bufferSize = readAhead;
//     stream.streamPosition = 0;
//     stream.threadPosition = 0;
//     stream.eof = qfalse;
// }
//
// /*
// ===============
// Sys_EndStreamedFile
//
// ================
// */
// void Sys_EndStreamedFile( fileHandle_t f )
// {
//     if ( f != stream.file ) {
//         Com_Error( ERR_FATAL, "Sys_EndStreamedFile: wrong file");
//     }
//
//     stream.file = 0;
//     Z_Free( stream.buffer );
// }
//
//
// /*
// ===============
// Sys_StreamedRead
//
// ================
// */
// int Sys_StreamedRead( void *buffer, int size, int count, fileHandle_t f )
// {
//     int     available;
//     int     remaining;
//     int     sleepCount;
//     int     copy;
//     int     bufferCount;
//     int     bufferPoint;
//     byte    *dest;
//
//     dest = (byte *)buffer;
//     remaining = size * count;
//
//     if ( remaining <= 0 ) {
//         Com_Error( ERR_FATAL, "Streamed read with non-positive size" );
//     }
//
//     sleepCount = 0;
//     while ( remaining > 0 ) {
//         available = stream.threadPosition - stream.streamPosition;
//         if ( !available ) {
//             if (stream.eof)
//                 break;
//             Sys_StreamThread();
//             continue;
//         }
//
//         bufferPoint = stream.streamPosition % stream.bufferSize;
//         bufferCount = stream.bufferSize - bufferPoint;
//
//         copy = available < bufferCount ? available : bufferCount;
//         if ( copy > remaining ) {
//             copy = remaining;
//         }
//         memcpy( dest, stream.buffer + bufferPoint, copy );
//         stream.streamPosition += copy;
//         dest += copy;
//         remaining -= copy;
//     }
//
//     return (count * size - remaining) / size;
// }
//
// /*
// ===============
// Sys_StreamSeek
//
// ================
// */
// void Sys_StreamSeek( fileHandle_t f, int offset, int origin ) {
//     // clear to that point
//     FS_Seek( f, offset, origin );
//     stream.streamPosition = 0;
//     stream.threadPosition = 0;
//     stream.eof = qfalse;
// }
//
// #endif

/*
========================================================================

EVENT LOOP

========================================================================
*/

// bk000306: upped this from 64
const MAX_QUED_EVENTS: usize = 256;
const MASK_QUED_EVENTS: usize = MAX_QUED_EVENTS - 1;

// bk000306: initialize
pub static mut eventHead: c_int = 0;
pub static mut eventTail: c_int = 0;
pub static mut sys_packetReceived: [byte; MAX_MSGLEN] = [0; MAX_MSGLEN];

// eventQue needs special initialization - sysEvent_t can't be const zero-initialized in stable Rust
// We'll initialize it lazily or leave it as uninitialized
pub static mut eventQue: [sysEvent_t; MAX_QUED_EVENTS] = [sysEvent_t {
    evTime: 0,
    evType: 0,
    evValue: 0,
    evValue2: 0,
    evPtrLength: 0,
    evPtr: null_mut(),
}; MAX_QUED_EVENTS];

/*
================
Sys_QueEvent

A time of 0 will get the current time
Ptr should either be null, or point to a block of data that can
be freed by the game later.
================
*/
pub fn Sys_QueEvent(time: c_int, event_type: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void) {
    let ev: *mut sysEvent_t;
    let mut time_val: c_int = time;

    unsafe {
        ev = &mut eventQue[(eventHead as usize) & MASK_QUED_EVENTS];

        // bk000305 - was missing
        if eventHead - eventTail >= MAX_QUED_EVENTS as c_int {
            Com_Printf(b"Sys_QueEvent: overflow\n\0".as_ptr() as *const c_char);
            // we are discarding an event, but don't leak memory
            if !(*ev).evPtr.is_null() {
                Z_Free((*ev).evPtr);
            }
            eventTail += 1;
        }

        eventHead += 1;

        if time_val == 0 {
            time_val = Sys_Milliseconds();
        }

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
pub fn Sys_GetEvent() -> sysEvent_t {
    let mut ev: sysEvent_t = unsafe { zeroed() };
    let s: *mut c_char;
    let mut netmsg: msg_t = unsafe { zeroed() };
    let mut adr: netadr_t = unsafe { zeroed() };

    unsafe {
        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) as usize) & MASK_QUED_EVENTS];
        }

        // pump the message loop
        // in vga this calls KBD_Update, under X, it calls GetEvent
        Sys_SendKeyEvents();

        // check for console commands
        s = Sys_ConsoleInput();
        if !s.is_null() {
            let mut b: *mut c_char;
            let mut len: c_int;

            len = libc::strlen(s) as c_int + 1;
            b = Z_Malloc(len as usize, TAG_FILESYS, qfalse) as *mut c_char;
            libc::strcpy(b, s);
            Sys_QueEvent(0, 1, 0, 0, len, b as *mut c_void); // SE_CONSOLE = 1
        }

        // check for other input devices
        IN_Frame();

        // check for network packets
        MSG_Init(&mut netmsg, sys_packetReceived.as_mut_ptr(), std::mem::size_of_val(&sys_packetReceived) as c_int);
        if Sys_GetPacket(&mut adr, &mut netmsg) != 0 {
            let buf: *mut netadr_t;
            let len: c_int;

            // copy out to a seperate buffer for qeueing
            len = (std::mem::size_of::<netadr_t>() as c_int) + 0; // netmsg.cursize - needs accessor
            buf = Z_Malloc(len as usize, TAG_EVENT, qfalse) as *mut netadr_t;
            *buf = adr;
            // memcpy( buf+1, netmsg.data, netmsg.cursize );
            Sys_QueEvent(0, 2, 0, 0, len, buf as *mut c_void); // SE_PACKET = 2
        }

        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) as usize) & MASK_QUED_EVENTS];
        }

        // create an empty event to return

        libc::memset(&mut ev as *mut _ as *mut c_void, 0, std::mem::size_of::<sysEvent_t>());
        ev.evTime = Sys_Milliseconds();

        ev
    }
}

/*****************************************************************************/

pub fn Sys_CheckCD() -> qboolean {
    qtrue
}

pub fn Sys_AppActivate() {
}

pub fn Sys_GetClipboardData() -> *mut c_char {
    null_mut()
}

pub fn Sys_Print(msg: *const c_char) {
    unsafe {
        libc::fputs(msg, libc::stderr);
    }
}

pub fn Sys_ConfigureFPU() {
    // bk001213 - divide by zero
    #[cfg(all(target_os = "linux", target_arch = "x86"))]
    {
        #[cfg(debug_assertions)]
        {
            // bk0101022 - enable FPE's in debug mode
            // static int fpu_word = _FPU_DEFAULT & ~(_FPU_MASK_ZM | _FPU_MASK_IM);
            // int current = 0;
            // _FPU_GETCW(current);
            // if ( current!=fpu_word) {
            // #if 0
            //   Com_Printf("FPU Control 0x%x (was 0x%x)\n", fpu_word, current );
            //   _FPU_SETCW( fpu_word );
            //   _FPU_GETCW( current );
            //   assert(fpu_word==current);
            // #endif
            // }
        }

        #[cfg(not(debug_assertions))]
        {
            // static int fpu_word = _FPU_DEFAULT;
            // _FPU_SETCW( fpu_word );
        }
    }
}

pub fn Sys_PrintBinVersion(name: *const c_char) {
    let date: *const c_char = bb"__DATE__\0".as_ptr() as *const c_char;
    let time: *const c_char = bb"__TIME__\0".as_ptr() as *const c_char;
    let sep: *const c_char = bb"==============================================================\0".as_ptr() as *const c_char;
    unsafe {
        libc::fprintf(libc::stdout, bb"\n\n%s\n\0".as_ptr() as *const c_char, sep);

        #[cfg(feature = "dedicated")]
        {
            libc::fprintf(libc::stdout, bb"Linux Quake3 Dedicated Server [%s %s]\n\0".as_ptr() as *const c_char, date, time);
        }

        #[cfg(not(feature = "dedicated"))]
        {
            libc::fprintf(libc::stdout, bb"Linux Quake3 Full Executable  [%s %s]\n\0".as_ptr() as *const c_char, date, time);
        }

        libc::fprintf(libc::stdout, bb" local install: %s\n\0".as_ptr() as *const c_char, name);
        libc::fprintf(libc::stdout, bb"%s\n\n\0".as_ptr() as *const c_char, sep);
    }
}

pub fn Sys_ParseArgs(argc: c_int, argv: *mut *mut c_char) {
    if argc == 2 {
        unsafe {
            if libc::strcmp(*argv.add(1), b"--version\0".as_ptr() as *const c_char) == 0
                || libc::strcmp(*argv.add(1), b"-v\0".as_ptr() as *const c_char) == 0
            {
                Sys_PrintBinVersion(*argv);
                Sys_Exit(0);
            }
        }
    }
}

// Opaque client types
#[repr(C)]
pub struct clientStatic_t {
    _opaque: [u8; 0],
}

pub static mut cls: clientStatic_t = unsafe { zeroed() };

pub extern "C" fn main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    // int     oldtime, newtime; // bk001204 - unused
    let mut len: c_int;
    let mut i: c_int;
    let mut cmdline: *mut c_char;

    unsafe {
        // go back to real user for config loads
        saved_euid = libc::geteuid();
        libc::seteuid(libc::getuid());

        Sys_ParseArgs(argc, argv); // bk010104 - added this for support

        Sys_SetDefaultCDPath(*argv);

        // merge the command line, this is kinda silly
        len = 1;
        i = 1;
        while i < argc {
            len += libc::strlen(*argv.add(i as usize)) as c_int + 1;
            i += 1;
        }

        cmdline = libc::malloc(len as usize) as *mut c_char;
        *cmdline = 0;

        i = 1;
        while i < argc {
            if i > 1 {
                libc::strcat(cmdline, b" \0".as_ptr() as *const c_char);
            }
            libc::strcat(cmdline, *argv.add(i as usize));
            i += 1;
        }

        // bk000306 - clear queues
        libc::memset(eventQue.as_mut_ptr() as *mut c_void, 0, MAX_QUED_EVENTS * std::mem::size_of::<sysEvent_t>());
        libc::memset(sys_packetReceived.as_mut_ptr() as *mut c_void, 0, MAX_MSGLEN * std::mem::size_of::<byte>());

        Com_Init(cmdline);
        NET_Init();

        libc::fcntl(0, libc::F_SETFL, libc::fcntl(0, libc::F_GETFL, 0) | libc::O_NONBLOCK);

        nostdout = Cvar_Get(b"nostdout\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        if !nostdout.is_null() && (*nostdout).value == 0 {
            libc::fcntl(0, libc::F_SETFL, libc::fcntl(0, libc::F_GETFL, 0) | libc::O_NONBLOCK);
        }

        loop {
            #[cfg(target_os = "linux")]
            {
                Sys_ConfigureFPU();
            }
            Com_Frame();
        }
    }
}

// ============================================================================
// Extern function declarations for engine/game functions
// ============================================================================

extern "C" {
    pub fn CL_Shutdown();
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Init(commandLine: *const c_char);
    pub fn Com_Frame();
    pub fn Cvar_Get(varName: *const c_char, defaultValue: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Cvar_Set(varName: *const c_char, value: *const c_char);
    pub fn Cvar_VariableString(varName: *const c_char) -> *const c_char;
    pub fn Cmd_AddCommand(cmdName: *const c_char, function: unsafe extern "C" fn());
    pub fn IN_Init();
    pub fn IN_Shutdown();
    pub fn IN_Frame();
    pub fn Sys_GetCurrentUser() -> *const c_char;
    pub fn Sys_SetDefaultCDPath(path: *const c_char);
    pub fn NET_Init();
    pub fn Z_Malloc(size: usize, tag: c_int, zero: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn FS_Read(buffer: *mut c_void, len: c_int, f: c_int) -> c_int;
    pub fn FS_Seek(f: c_int, offset: c_int, origin: c_int);
    pub fn Sys_SendKeyEvents();
    pub fn Sys_GetPacket(adr: *mut netadr_t, msg: *mut msg_t) -> qboolean;
    pub fn MSG_Init(buf: *mut msg_t, data: *mut byte, length: c_int);
    pub fn Sys_Milliseconds() -> c_int;
    pub static mut com_dedicated: *mut cvar_t;
}
