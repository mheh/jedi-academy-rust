#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::os::raw::{c_uint, c_long};

// Stubs for unported dependencies from q_shared.h, qcommon.h, client.h, win_local.h

pub const MAX_OSPATH: usize = 256;
pub const MAX_QPATH: usize = 64;
pub const MAX_MSGLEN: usize = 16384;
pub const MAX_QUED_EVENTS: usize = 256;
pub const MASK_QUED_EVENTS: usize = MAX_QUED_EVENTS - 1;

pub const TAG_NEWDEL: c_int = 1;
pub const TAG_LISTFILES: c_int = 2;

pub const ERR_FATAL: c_int = 3;

pub type qboolean = c_int;
pub const qfalse: qboolean = 0;
pub const qtrue: qboolean = 1;

#[repr(C)]
pub struct sysEvent_t {
    pub evTime: c_int,
    pub evType: c_int,
    pub evValue: c_int,
    pub evValue2: c_int,
    pub evPtr: *mut c_void,
}

// External globals
extern "C" {
    pub static mut eventHead: c_int;
    pub static mut eventTail: c_int;
    pub static mut eventQue: [sysEvent_t; MAX_QUED_EVENTS];
    pub static mut sys_packetReceived: [u8; MAX_MSGLEN];
}

// External functions - memory allocation
extern "C" {
    pub fn Z_Malloc(size: usize, tag: c_int, qallocate: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
}

// External functions - command system
extern "C" {
    pub fn Cmd_AddCommand(name: *const c_char, func: extern "C" fn());
}

// External functions - timing
extern "C" {
    pub fn Sys_Milliseconds() -> c_int;
}

// External functions - message handling
#[repr(C)]
pub struct msg_t {
    // stub - structure details not needed for this file
    _data: [u8; 0],
}

extern "C" {
    pub fn MSG_Init(msg: *mut msg_t, data: *mut u8, length: c_int);
}

// External functions - printing and errors
extern "C" {
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Com_Init(cmdline: *const c_char);
    pub fn Com_Frame();
    pub fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    pub fn Com_ShutdownZoneMemory();
    pub fn Com_ShutdownHunkMemory();
}

// External functions - sound and streaming
extern "C" {
    pub fn S_Shutdown();
    pub fn Sys_StreamShutdown();
}

// External functions - input and UI
extern "C" {
    pub fn IN_Frame();
}

// External functions - file system
extern "C" {
    pub fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    pub fn FS_FreeFile(buffer: *mut c_void);
}

// External functions - game API
extern "C" {
    pub fn GetGameAPI(import: *mut c_void) -> *mut c_void;
    pub fn CG_PreInit();
    pub fn cg_dllEntry(syscallptr: extern "C" fn(c_int, ...) -> c_int);
    pub fn vmMain(
        command: c_int,
        arg0: c_int,
        arg1: c_int,
        arg2: c_int,
        arg3: c_int,
        arg4: c_int,
        arg5: c_int,
        arg6: c_int,
        arg7: c_int,
    ) -> c_int;
}

// External functions - cvar
extern "C" {
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

#[cfg(target_os = "windows")]
extern "system" {
    pub fn OutputDebugStringA(lpOutputString: *const c_char);
}

// Platform-specific type stubs
#[cfg(target_os = "windows")]
pub type HANDLE = *mut c_void;
#[cfg(target_os = "windows")]
pub type LPCSTR = *const c_char;
#[cfg(target_os = "windows")]
pub const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

// External functions for game loading
extern "C" {
    pub fn Sys_In_Restart_f();
    pub fn Sys_Net_Restart_f();
    pub fn DebugConsoleHandleCommands();
}

// Operator new overloads equivalent - Rust translation note: C++ operator new/delete
// mapped to direct Z_Malloc/Z_Free calls for functional equivalence

// Sys_Init
//
// Called after the common systems (cvars, files, etc)
// are initialized
pub extern "C" fn Sys_Init() {
    unsafe {
        Cmd_AddCommand(b"in_restart\0".as_ptr() as *const c_char, Sys_In_Restart_f);
        Cmd_AddCommand(b"net_restart\0".as_ptr() as *const c_char, Sys_Net_Restart_f);
    }
}

pub extern "C" fn Sys_Cwd() -> *mut c_char {
    static mut cwd: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

    unsafe {
        #[cfg(target_os = "windows")]
        {
            // strcpy(cwd, "d:");
            cwd[0] = b'd' as c_char;
            cwd[1] = b':' as c_char;
            cwd[2] = 0;
        }

        #[cfg(target_os = "macos")]
        {
            // strcpy(cwd, ".");
            cwd[0] = b'.' as c_char;
            cwd[1] = 0;
        }

        core::ptr::addr_of_mut!(cwd) as *mut c_char
    }
}

//
// Sys_In_Restart_f
//
// Restart the input subsystem
//
pub extern "C" fn Sys_In_Restart_f() {}

//
// Sys_Error
//
// Show the early console as an error dialog
//
pub extern "C" fn Sys_Error(error: *const c_char, mut args: ...) {
    let mut text: [c_char; 256] = [0; 256];

    unsafe {
        // Porting note: va_list handling - simplified stub
        // In actual implementation, would need proper varargs handling
        libc::vsprintf(text.as_mut_ptr(), error, args.as_va_list());

        #[cfg(target_os = "macos")]
        {
            libc::printf(text.as_ptr());
        }

        #[cfg(target_os = "windows")]
        {
            OutputDebugStringA(text.as_ptr());
        }

        // Com_ShutdownZoneMemory();
        // Com_ShutdownHunkMemory();

        libc::exit(1);
    }
}

//
// Sys_GetEvent
//
//
pub extern "C" fn Sys_GetEvent() -> sysEvent_t {
    unsafe {
        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) & MASK_QUED_EVENTS as c_int) as usize];
        }

        // check for network packets
        let mut netmsg: msg_t = core::mem::zeroed();
        MSG_Init(
            &mut netmsg as *mut msg_t,
            sys_packetReceived.as_mut_ptr(),
            MAX_MSGLEN as c_int,
        );

        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) & MASK_QUED_EVENTS as c_int) as usize];
        }

        // create an empty event to return
        let mut ev: sysEvent_t = core::mem::zeroed();
        ev.evTime = Sys_Milliseconds();

        ev
    }
}

pub extern "C" fn Sys_Print(msg: *const c_char) {
    unsafe {
        #[cfg(target_os = "macos")]
        {
            libc::printf(msg);
        }

        #[cfg(target_os = "windows")]
        {
            OutputDebugStringA(msg);
        }
    }
}

//
// Sys_Log
//
pub extern "C" fn Sys_Log_short(file: *const c_char, msg: *const c_char) {
    unsafe {
        Sys_Log_full(
            file,
            msg as *const c_void,
            libc::strlen(msg) as c_int,
            if libc::strchr(msg, b'\n' as c_int) != std::ptr::null() {
                1
            } else {
                0
            },
        );
    }
}

//
// Sys_Log
//
pub extern "C" fn Sys_Log_full(
    file: *const c_char,
    buffer: *const c_void,
    size: c_int,
    flush: qboolean,
) {
    #[cfg(not(feature = "FINAL_BUILD"))]
    {
        use std::ffi::CStr;

        static mut unableToLog: bool = false;

        unsafe {
            // Once we've failed to write to the log files once, bail out.
            // This lets us put release builds on DVD without recompiling.
            if unableToLog {
                return;
            }

            #[repr(C)]
            struct FileInfo {
                name: [c_char; MAX_QPATH],
                handle: *mut libc::FILE,
            }

            const LOG_MAX_FILES: usize = 4;
            static mut files: [FileInfo; LOG_MAX_FILES] = [
                FileInfo {
                    name: [0; MAX_QPATH],
                    handle: std::ptr::null_mut(),
                };
                LOG_MAX_FILES
            ];
            static mut num_files: c_int = 0;

            let mut cur: *mut FileInfo = std::ptr::null_mut();
            for f in 0..num_files as usize {
                if libc::stricmp(file, files[f].name.as_ptr()) == 0 {
                    cur = &mut files[f] as *mut FileInfo;
                    break;
                }
            }

            if cur.is_null() {
                if num_files >= LOG_MAX_FILES as c_int {
                    Sys_Print(b"Too many log files!\n\0".as_ptr() as *const c_char);
                    return;
                }

                cur = &mut files[num_files as usize] as *mut FileInfo;
                num_files += 1;
                libc::strcpy((*cur).name.as_mut_ptr(), file);
                (*cur).handle = std::ptr::null_mut();
            }

            let mut fullname: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            libc::sprintf(
                fullname.as_mut_ptr(),
                b"d:\\%s\0".as_ptr() as *const c_char,
                file,
            );
            if (*cur).handle.is_null() {
                (*cur).handle = libc::fopen(fullname.as_ptr(), b"wb\0".as_ptr() as *const c_char);
                if (*cur).handle.is_null() {
                    Sys_Print(b"Unable to open log file!\n\0".as_ptr() as *const c_char);
                    unableToLog = true;
                    return;
                }
            }

            if size == 1 {
                libc::fputc(*(buffer as *const c_char) as u8 as c_int, (*cur).handle);
            } else {
                libc::fwrite(buffer, size as usize, 1, (*cur).handle);
            }

            if flush != 0 {
                libc::fflush((*cur).handle);
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub static mut Sys_FileStreamMutex: HANDLE = INVALID_HANDLE_VALUE;

pub extern "C" fn Win_Init() {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            // Sys_FileStreamMutex = CreateMutex(NULL, FALSE, NULL);
            // Porting note: CreateMutex stubbed - Windows API not directly available
        }
    }
}

//
// XBE SWITCHING SUPPORT
//

// Despite what you may think, this function actually just returns
// a value telling you if you *should* quick-boot -- ie skip intro
// cinematics and such. Only supposed to XGetLaunchInfo once per
// boot, so we cache the results.

pub extern "C" fn Sys_QuickStart() -> qboolean {
    static mut retVal: qboolean = 0;
    static mut initialized: qboolean = 0;

    unsafe {
        if initialized != 0 {
            return retVal;
        }

        initialized = 1;

        // Porting note: XGetLaunchInfo and related Xbox APIs stubbed - Xbox platform not supported
        // Functional equivalent would check launch data and set retVal accordingly

        0
    }
}

pub extern "C" fn Sys_Reboot(reason: *const c_char) {
    unsafe {
        // LAUNCH_DATA ld;
        // const char *path = NULL;

        // memset( &ld, 0, sizeof(ld) );

        if libc::strcmp(reason, b"multiplayer\0".as_ptr() as *const c_char) == 0 {
            // path = "d:\\jamp.xbe";
        } else {
            Com_Error(
                ERR_FATAL,
                b"Unknown reboot code %s\n\0".as_ptr() as *const c_char,
                reason,
            );
        }

        // Title should not be doing ANYTHING in the background.
        // Shutting down sound ensures that the sound thread is gone
        S_Shutdown();
        // Similarly, kill off the streaming thread
        Sys_StreamShutdown();

        // XLaunchNewImage(path, &ld);

        // This function should not return!
        Com_Error(
            ERR_FATAL,
            b"ERROR: XLaunchNewImage returned\n\0".as_ptr() as *const c_char,
        );
    }
}

//
// WinMain
//
#[cfg(target_os = "windows")]
pub extern "C" fn main() -> c_int {
    main_impl()
}

#[cfg(target_os = "macos")]
pub extern "C" fn main_mac(argc: c_int, argv: *mut *mut c_char) -> c_int {
    main_impl()
}

fn main_impl() -> c_int {
    unsafe {
        // 	Z_SetFreeOSMem();

        // I'm going to kill someone. This should not be necessary. No, really.
        // Direct3D_SetPushBufferSize(1024*1024, 128*1024);
        // Porting note: Direct3D stubbed

        // get the initial time base
        Sys_Milliseconds();

        Win_Init();
        Com_Init(b"\0".as_ptr() as *const c_char);

        // main game loop
        loop {
            IN_Frame();
            Com_Frame();

            // Poll debug console for new commands
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                DebugConsoleHandleCommands();
            }
        }

        // return 0; // unreachable
    }
}

pub extern "C" fn Sys_GetClipboardData() -> *mut c_char {
    std::ptr::null_mut()
}

pub extern "C" fn Sys_StartProcess(_path: *mut c_char, _wait: qboolean) {}

pub extern "C" fn Sys_OpenURL(_url: *mut c_char, _mode: c_int) {}

pub extern "C" fn Sys_Quit() {}

pub extern "C" fn Sys_ShowConsole(_level: c_int, _quitOnClose: c_int) {}

pub extern "C" fn Sys_Mkdir(_path: *const c_char) {}

pub extern "C" fn Sys_LowPhysicalMemory() -> c_int {
    0
}

pub extern "C" fn Sys_FreeFileList(filelist: *mut *mut c_char) {
    unsafe {
        // All strings in a file list are allocated at once, so we just need to
        // do two frees, one for strings, one for the pointers.
        if !filelist.is_null() {
            if !(*filelist).is_null() {
                Z_Free(*filelist as *mut c_void);
            }

            Z_Free(filelist as *mut c_void);
        }
    }
}

#[cfg(feature = "JK2MP")]
pub extern "C" fn Sys_ListFiles(
    directory: *const c_char,
    extension: *const c_char,
    filter: *mut c_char,
    numfiles: *mut c_int,
    wantsubs: qboolean,
) -> *mut *mut c_char {
    Sys_ListFiles_impl(directory, extension, filter as *mut c_char, numfiles, wantsubs)
}

#[cfg(not(feature = "JK2MP"))]
pub extern "C" fn Sys_ListFiles(
    directory: *const c_char,
    extension: *const c_char,
    numfiles: *mut c_int,
    wantsubs: qboolean,
) -> *mut *mut c_char {
    Sys_ListFiles_impl(directory, extension, std::ptr::null_mut(), numfiles, wantsubs)
}

fn Sys_ListFiles_impl(
    directory: *const c_char,
    extension: *const c_char,
    filter: *mut c_char,
    numfiles: *mut c_int,
    wantsubs: qboolean,
) -> *mut *mut c_char {
    unsafe {
        #[cfg(feature = "JK2MP")]
        {
            // MP has extra filter paramter. We don't support that.
            if !filter.is_null() {
                // assert(!"Sys_ListFiles doesn't support filter on console!");
                return std::ptr::null_mut();
            }
        }

        // Hax0red console version of Sys_ListFiles. We mangle our arguments to get a standard filename
        // That file should exist, and contain the list of files that meet this search criteria.
        let mut listFilename: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
        let mut listFile: *mut c_char = std::ptr::null_mut();
        let mut curFile: *mut c_char;
        let mut end: *mut c_char;
        let mut nfiles: c_int;
        let mut retList: *mut *mut c_char;

        let mut dir_ptr = directory;
        // S00per hack
        if !libc::strstr(directory, b"d:\\base\\\0".as_ptr() as *const c_char).is_null() {
            dir_ptr = directory.add(8);
        }

        let mut ext_ptr = extension;
        if ext_ptr.is_null() {
            ext_ptr = b"\0".as_ptr() as *const c_char;
        } else if *ext_ptr as u8 == b'/' && *ext_ptr.add(1) as u8 == 0 {
            // Passing a slash as extension will find directories
            ext_ptr = b"dir\0".as_ptr() as *const c_char;
        } else if *ext_ptr as u8 == b'.' {
            // Skip over leading .
            ext_ptr = ext_ptr.add(1);
        }

        // Build our filename
        Com_sprintf(
            listFilename.as_mut_ptr(),
            MAX_OSPATH,
            b"%s\\_console_%s_list_\0".as_ptr() as *const c_char,
            dir_ptr,
            ext_ptr,
        );

        let mut file_buffer: *mut c_void = std::ptr::null_mut();
        if FS_ReadFile(listFilename.as_ptr(), &mut file_buffer) <= 0 {
            listFile = file_buffer as *mut c_char;
            if !listFile.is_null() {
                FS_FreeFile(listFile as *mut c_void);
            }
            Com_Printf(
                b"WARNING: List file %s not found\n\0".as_ptr() as *const c_char,
                listFilename.as_ptr(),
            );
            if !numfiles.is_null() {
                *numfiles = 0;
            }
            return std::ptr::null_mut();
        }

        listFile = file_buffer as *mut c_char;

        // Do a first pass to count number of files in the list
        nfiles = 0;
        curFile = listFile;
        loop {
            // Find end of line
            end = libc::strchr(curFile, b'\r' as c_int);
            if !end.is_null() {
                // Should have a \n next -- skip them both
                end = end.add(2);
            } else {
                end = libc::strchr(curFile, b'\n' as c_int);
                if !end.is_null() {
                    end = end.add(1);
                } else {
                    end = curFile.add(libc::strlen(curFile));
                }
            }

            // Is the line empty?  If so, we're done.
            if curFile.is_null() || *curFile == 0 {
                break;
            }
            nfiles += 1;

            // Advance to next line
            curFile = end;
        }

        // Fill in caller's pointer for number of files found
        if !numfiles.is_null() {
            *numfiles = nfiles;
        }

        // Did we find any files at all?
        if nfiles == 0 {
            FS_FreeFile(listFile as *mut c_void);
            return std::ptr::null_mut();
        }

        // Allocate a file list, and quick string pool, but use LISTFILES
        retList = Z_Malloc(
            ((nfiles + 1) as usize) * std::mem::size_of::<*mut c_char>(),
            TAG_LISTFILES,
            qfalse,
        ) as *mut *mut c_char;
        // Our string pool is actually slightly too large, but it's temporary, and that's better
        // than slightly too small
        let stringPool = Z_Malloc(
            libc::strlen(listFile) + 1,
            TAG_LISTFILES,
            qfalse,
        ) as *mut c_char;

        // Now go through the list of files again, and fill in the list to be returned
        nfiles = 0;
        curFile = listFile;
        loop {
            // Find end of line
            end = libc::strchr(curFile, b'\r' as c_int);
            if !end.is_null() {
                // Should have a \n next -- skip them both
                *end = 0;
                end = end.add(1);
                *end = 0;
                end = end.add(1);
            } else {
                end = libc::strchr(curFile, b'\n' as c_int);
                if !end.is_null() {
                    *end = 0;
                    end = end.add(1);
                } else {
                    end = curFile.add(libc::strlen(curFile));
                }
            }

            // Is the line empty?  If so, we're done.
            let curStrSize = libc::strlen(curFile);
            if curStrSize < 1 {
                *retList.add(nfiles as usize) = std::ptr::null_mut();
                break;
            }

            // Alloc a small copy
            // retList[nfiles++] = CopyString( curFile );
            *retList.add(nfiles as usize) = stringPool.add((stringPool as *mut u8 as usize
                - stringPool as *mut u8 as usize)
                + nfiles as usize);
            libc::strcpy(
                *retList.add(nfiles as usize),
                curFile,
            );
            let new_pool = stringPool.add(curStrSize + 1);
            libc::strcpy(new_pool, curFile);

            nfiles += 1;

            // Advance to next line
            curFile = end;
        }

        // Free the special file's buffer
        FS_FreeFile(listFile as *mut c_void);

        retList
    }
}

//
// Sys_UnloadGame
//
pub extern "C" fn Sys_UnloadGame() {}

//
// Sys_GetGameAPI
//
// Loads the game dll
//
#[cfg(not(feature = "JK2MP"))]
pub extern "C" fn Sys_GetGameAPI(parms: *mut c_void) -> *mut c_void {
    unsafe { GetGameAPI(parms) }
}

//
// Sys_LoadCgame
//
// Used to hook up a development dll
//
#[cfg(not(feature = "JK2MP"))]
pub extern "C" fn Sys_LoadCgame(
    entryPoint: *mut extern "C" fn(c_int, ...) -> c_int,
    systemcalls: extern "C" fn(c_int, ...) -> c_int,
) -> *mut c_void {
    unsafe {
        cg_dllEntry(systemcalls);
        *entryPoint = vmMain as extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int) -> c_int
            as extern "C" fn(c_int, ...) -> c_int;
        // CG_PreInit();
    }
    std::ptr::null_mut()
}

// VVFIXME: More stubs
pub extern "C" fn Sys_FileOutOfDate(
    psFinalFileName: LPCSTR,
    psDataFileName: LPCSTR,
) -> qboolean {
    qfalse
}

pub extern "C" fn Sys_CopyFile(
    lpExistingFileName: LPCSTR,
    lpNewFileName: LPCSTR,
    bOverwrite: qboolean,
) -> qboolean {
    qfalse
}

pub extern "C" fn Sys_CheckCD() -> qboolean {
    qtrue
}
