#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_float};
use core::ptr::{addr_of, addr_of_mut};

// Imports from qcommon.h, client.h, win_local.h, resource.h, g_public.h, xbox/XBLive.h
extern "C" {
    // From qcommon
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Malloc_tag_4(size: usize, tag: c_int, clear: c_int, align: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Cmd_AddCommand(name: *const c_char, func: extern "C" fn());
    fn Com_Init(cmdline: *const c_char);
    fn Com_Frame();
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_ShutdownZoneMemory();
    fn Com_ShutdownHunkMemory();
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // From client
    static mut cls: cls_t;

    // From win_local
    fn IN_Frame();
    fn Sys_GetPacket(adr: *mut netadr_t, msg: *mut msg_t) -> c_int;
    fn Sys_QueEvent(time: c_int, event_type: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void);
    fn Sys_Milliseconds() -> c_int;
    fn OutputDebugString(str: *const c_char);
    fn Direct3D_SetPushBufferSize(size1: c_int, size2: c_int);

    // From network
    fn NET_Init();
    fn MSG_Init(msg: *mut msg_t, data: *mut c_char, length: c_int);

    // From sound
    fn S_Shutdown();
    fn S_BeginRegistration();

    // From FS
    fn FS_ReadFile(name: *const c_char, buf: *mut *mut c_char) -> c_int;
    fn FS_FreeFile(buf: *mut c_void);

    // From game
    fn GetGameAPI(import: *mut game_import_t) -> *mut game_export_t;
    fn CG_PreInit();
    fn cg_dllEntry(syscallptr: extern "C" fn(c_int) -> c_int);
    fn vmMain(command: c_int, arg0: c_int, arg1: c_int, arg2: c_int, arg3: c_int, arg4: c_int, arg5: c_int, arg6: c_int, arg7: c_int) -> c_int;
    fn DebugConsoleHandleCommands();

    // From xbox
    fn XLaunchNewImage(path: *const c_char, ld: *mut LAUNCH_DATA);
    fn CreateMutex(lpMutexAttributes: *mut c_void, bInitialOwner: c_int, lpName: *const c_char) -> *mut c_void;

    // libc functions
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn printf(fmt: *const c_char, ...) -> c_int;
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn fclose(stream: *mut c_void) -> c_int;
    fn fputc(c: c_int, stream: *mut c_void) -> c_int;
    fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    fn fflush(stream: *mut c_void) -> c_int;
    fn vsprintf(str: *mut c_char, format: *const c_char, ap: core::ffi::VaList) -> c_int;
    fn va_start(ap: *mut core::ffi::VaList, ...);
    fn va_end(ap: *mut core::ffi::VaList);
    fn exit(status: c_int) -> !;
}

// Extern globals from other modules
extern "C" {
    pub static mut eventHead: c_int;
    pub static mut eventTail: c_int;
    pub static mut eventQue: [sysEvent_t; MAX_QUED_EVENTS];
    pub static mut sys_packetReceived: [c_char; MAX_MSGLEN];
}

// Constants
pub const MAX_OSPATH: usize = 256;
pub const MAX_QPATH: usize = 64;
pub const MAX_QUED_EVENTS: usize = 256;
pub const MAX_MSGLEN: usize = 16384;
pub const MASK_QUED_EVENTS: usize = MAX_QUED_EVENTS - 1;
pub const MAX_POLL_RATE: c_int = 15;

pub const TAG_NEWDEL: c_int = 0;
pub const TAG_EVENT: c_int = 1;
pub const TAG_LISTFILES: c_int = 2;

pub const ERR_FATAL: c_int = 3;

pub const SE_PACKET: c_int = 1;

pub const LOG_MAX_FILES: usize = 4;

pub const INVALID_HANDLE_VALUE: *mut c_void = core::ptr::null_mut();

// Flags for Sys_Log
const LAUNCH_MAGIC: &[u8; 4] = b"J3D1";

// Type definitions
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
pub struct netadr_t {
    // Placeholder - actual layout from qcommon
    pub type_: c_int,
}

#[repr(C)]
pub struct msg_t {
    pub allowoverflow: c_int,
    pub overflowed: c_int,
    pub data: *mut c_char,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
}

#[repr(C)]
pub struct cls_t {
    pub soundRegistered: c_int,
    // ... other fields omitted for brevity
}

#[repr(C)]
pub struct game_import_t {
    // Placeholder
}

#[repr(C)]
pub struct game_export_t {
    // Placeholder
}

#[repr(C)]
pub struct FileInfo {
    pub name: [c_char; MAX_QPATH],
    pub handle: *mut c_void,
}

#[repr(C)]
pub struct LAUNCH_DATA {
    pub Data: [c_char; 256],
}

#[repr(C)]
pub union PLD_LAUNCH_DASHBOARD {
    pub dwReason: c_int,
}

pub const XLD_LAUNCH_DASHBOARD_NEW_ACCOUNT_SIGNUP: c_int = 0;
pub const XLD_LAUNCH_DASHBOARD_NETWORK_CONFIGURATION: c_int = 1;
pub const XLD_LAUNCH_DASHBOARD_ACCOUNT_MANAGEMENT: c_int = 2;

#[cfg(target_os = "windows")]
extern "C" {
    fn qfalse() -> c_int;
    fn qtrue() -> c_int;
}

#[cfg(not(target_os = "windows"))]
pub const qfalse: c_int = 0;
#[cfg(not(target_os = "windows"))]
pub const qtrue: c_int = 1;

// C++ operator new and delete stubs
// Note: These are C++ specific and translated as extern functions
// In the original, these override global new/delete operators
pub extern "C" fn _Znwm(size: usize) -> *mut c_void {
    // operator new(size_t size)
    unsafe { Z_Malloc(size, TAG_NEWDEL, qfalse as c_int) }
}

pub extern "C" fn _Znam(size: usize) -> *mut c_void {
    // operator new[](size_t size)
    unsafe { Z_Malloc(size, TAG_NEWDEL, qfalse as c_int) }
}

pub extern "C" fn _ZdaPv(ptr: *mut c_void) {
    // operator delete[](void *ptr)
    if !ptr.is_null() {
        unsafe { Z_Free(ptr) };
    }
}

pub extern "C" fn _ZdlPv(ptr: *mut c_void) {
    // operator delete(void *ptr)
    if !ptr.is_null() {
        unsafe { Z_Free(ptr) };
    }
}

/*
================
Sys_Init

Called after the common systems (cvars, files, etc)
are initialized
================
*/
extern "C" {
    fn Sys_In_Restart_f();
    fn Sys_Net_Restart_f();
}

pub extern "C" fn Sys_Init() {
    unsafe {
        Cmd_AddCommand(b"in_restart\0".as_ptr() as *const c_char, Sys_In_Restart_f);
        Cmd_AddCommand(b"net_restart\0".as_ptr() as *const c_char, Sys_Net_Restart_f);
    }
}

pub extern "C" fn Sys_Cwd() -> *mut c_char {
    static mut cwd: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

    unsafe {
        #[cfg(all(feature = "xbox"))]
        {
            strcpy(addr_of_mut!(cwd[0]), b"d:\0".as_ptr() as *const c_char);
        }

        #[cfg(all(feature = "gamecube"))]
        {
            strcpy(addr_of_mut!(cwd[0]), b".\0".as_ptr() as *const c_char);
        }

        addr_of_mut!(cwd[0])
    }
}

/*
=================
Sys_In_Restart_f

Restart the input subsystem
=================
*/
pub extern "C" fn Sys_In_Restart_f() {
}

/*
=============
Sys_Error

Show the early console as an error dialog
=============
*/
pub extern "C" fn Sys_Error(error: *const c_char, mut args: ...) {
    let mut text: [c_char; 256] = [0; 256];

    unsafe {
        let mut argptr = args.clone();
        vsprintf(addr_of_mut!(text[0]), error, argptr);

        #[cfg(all(feature = "gamecube"))]
        {
            printf(addr_of!(text[0]));
        }

        #[cfg(not(all(feature = "gamecube")))]
        {
            OutputDebugString(addr_of!(text[0]));
        }

        // #if 0 // UN-PORT
        // Com_ShutdownZoneMemory();
        // Com_ShutdownHunkMemory();
        // #endif

        exit(1);
    }
}

/*
================
Sys_GetEvent

================
*/
pub extern "C" fn Sys_GetEvent() -> sysEvent_t {
    let mut ev: sysEvent_t;

    unsafe {
        // return if we have data
        if addr_of!(eventHead).read() > addr_of!(eventTail).read() {
            addr_of_mut!(eventTail).write(addr_of!(eventTail).read() + 1);
            return addr_of!(eventQue).read()[((addr_of!(eventTail).read() - 1) & MASK_QUED_EVENTS as c_int) as usize];
        }

        // check for network packets
        let mut netmsg: msg_t;
        let mut adr: netadr_t;

        for poll in 0..MAX_POLL_RATE {
            MSG_Init(addr_of_mut!(netmsg), addr_of_mut!(sys_packetReceived[0]), MAX_MSGLEN as c_int);
            if Sys_GetPacket(addr_of_mut!(adr), addr_of_mut!(netmsg)) != 0 {
                let mut buf: *mut netadr_t;
                let len: c_int;

                // copy out to a seperate buffer for qeueing
                // the readcount stepahead is for SOCKS support
                len = (core::mem::size_of::<netadr_t>() as c_int) + netmsg.cursize - netmsg.readcount;
                //buf = (netadr_t *)GG_Malloc( len, MemoryBlock::kEventTag, qtrue );
                buf = Z_Malloc_tag_4(len as usize, TAG_EVENT, qfalse as c_int, 4) as *mut netadr_t;
                (*buf) = adr;
                memcpy(
                    (buf as *mut c_char).add(1) as *mut c_void,
                    (netmsg.data as *const c_char).add(netmsg.readcount as usize) as *const c_void,
                    (netmsg.cursize - netmsg.readcount) as usize
                );
                Sys_QueEvent(0, SE_PACKET, 0, 0, len, buf as *mut c_void);
            } else {
                // Bail out if there's no more data
                break;
            }
        }

        // #if 0	// Removed as in SOF2
        // 	// return if we have data
        //     if ( eventHead > eventTail ) {
        //             eventTail++;
        //             return eventQue[ ( eventTail - 1 ) & MASK_QUED_EVENTS ];
        //     }
        // #endif

        // create an empty event to return
        memset(addr_of_mut!(ev) as *mut c_void, 0, core::mem::size_of::<sysEvent_t>());
        ev.evTime = Sys_Milliseconds();

        ev
    }
}

pub extern "C" fn Sys_Print(msg: *const c_char) {
    unsafe {
        #[cfg(all(feature = "gamecube"))]
        {
            printf(msg);
        }

        #[cfg(not(all(feature = "gamecube")))]
        {
            OutputDebugString(msg);
        }
    }
}

/*
==============
Sys_Log
==============
*/
pub extern "C" fn Sys_Log(file: *const c_char, msg: *const c_char) {
    unsafe {
        Sys_Log_full(file, msg as *const c_void, strlen(msg) as c_int, if !strchr(msg, b'\n' as c_int).is_null() { 1 } else { 0 });
    }
}

/*
==============
Sys_Log
==============
*/
pub extern "C" fn Sys_Log_full(file: *const c_char, buffer: *const c_void, size: c_int, flush: c_int) {
    #[cfg(not(feature = "final_build"))]
    {
        static mut unableToLog: c_int = 0;

        // Once we've failed to write to the log files once, bail out.
        // This lets us put release builds on DVD without recompiling.
        if addr_of!(unableToLog).read() != 0 {
            return;
        }

        static mut files: [FileInfo; LOG_MAX_FILES] = [FileInfo { name: [0; MAX_QPATH], handle: core::ptr::null_mut() }; LOG_MAX_FILES];
        static mut num_files: c_int = 0;

        let mut cur: Option<&mut FileInfo> = None;
        for f in 0..(addr_of!(num_files).read() as usize) {
            if stricmp(file, addr_of!(files[f].name[0])) == 0 {
                cur = Some(&mut addr_of_mut!(files[f]));
                break;
            }
        }

        if cur.is_none() {
            if addr_of!(num_files).read() >= LOG_MAX_FILES as c_int {
                Sys_Print(b"Too many log files!\n\0".as_ptr() as *const c_char);
                return;
            }

            let n = addr_of!(num_files).read();
            addr_of_mut!(num_files).write(n + 1);
            cur = Some(&mut addr_of_mut!(files[n as usize]));
            strcpy(addr_of_mut!(cur.unwrap().name[0]), file);
            cur.unwrap().handle = core::ptr::null_mut();
        }

        let cur_file = cur.unwrap();
        let mut fullname: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        Com_sprintf(addr_of_mut!(fullname[0]), MAX_QPATH as c_int, b"d:\\%s\0".as_ptr() as *const c_char, file);

        if cur_file.handle.is_null() {
            cur_file.handle = fopen(addr_of!(fullname[0]), b"wb\0".as_ptr() as *const c_char);
            if cur_file.handle.is_null() {
                Sys_Print(b"Unable to open log file!\n\0".as_ptr() as *const c_char);
                addr_of_mut!(unableToLog).write(1);
                return;
            }
        }

        if size == 1 {
            fputc(*(buffer as *const c_char) as c_int, cur_file.handle);
        } else {
            fwrite(buffer, size as usize, 1, cur_file.handle);
        }

        if flush != 0 {
            fflush(cur_file.handle);
        }
    }
}

#[cfg(feature = "xbox")]
pub static mut Sys_FileStreamMutex: *mut c_void = INVALID_HANDLE_VALUE;

pub extern "C" fn Win_Init() {
    #[cfg(feature = "xbox")]
    unsafe {
        addr_of_mut!(Sys_FileStreamMutex).write(CreateMutex(core::ptr::null_mut(), 0, core::ptr::null()));
    }
}

/*
=====================

XBE SWITCHING SUPPORT

=====================
*/
pub extern "C" fn Sys_Reboot(reason: *const c_char) {
    unsafe {
        let mut ld: LAUNCH_DATA = core::mem::zeroed();
        let mut path: *const c_char = core::ptr::null();

        memset(addr_of_mut!(ld) as *mut c_void, 0, core::mem::size_of::<LAUNCH_DATA>());

        if Q_stricmp(reason, b"new_account\0".as_ptr() as *const c_char) == 0 {
            let pDash = &mut ld.Data[0] as *mut c_char as *mut PLD_LAUNCH_DASHBOARD;
            (*pDash).dwReason = XLD_LAUNCH_DASHBOARD_NEW_ACCOUNT_SIGNUP;
            path = core::ptr::null();
        } else if Q_stricmp(reason, b"net_config\0".as_ptr() as *const c_char) == 0 {
            let pDash = &mut ld.Data[0] as *mut c_char as *mut PLD_LAUNCH_DASHBOARD;
            (*pDash).dwReason = XLD_LAUNCH_DASHBOARD_NETWORK_CONFIGURATION;
            path = core::ptr::null();
        } else if Q_stricmp(reason, b"manage_account\0".as_ptr() as *const c_char) == 0 {
            let pDash = &mut ld.Data[0] as *mut c_char as *mut PLD_LAUNCH_DASHBOARD;
            (*pDash).dwReason = XLD_LAUNCH_DASHBOARD_ACCOUNT_MANAGEMENT;
            path = core::ptr::null();
        } else if Q_stricmp(reason, b"singleplayer\0".as_ptr() as *const c_char) == 0 {
            path = b"d:\\default.xbe\0".as_ptr() as *const c_char;
            strcpy(addr_of_mut!(ld.Data[0]), LAUNCH_MAGIC.as_ptr() as *const c_char);
        } else {
            Com_Error(ERR_FATAL, b"Unknown reboot code %s\n\0".as_ptr() as *const c_char, reason);
        }

        // Title should not be doing ANYTHING in the background.
        // Shutting down sound ensures that the sound thread is gone
        S_Shutdown();
        // Similarly, kill off the streaming thread
        Sys_StreamShutdown();

        XLaunchNewImage(path, addr_of_mut!(ld));

        // This function should not return!
        Com_Error(ERR_FATAL, b"ERROR: XLaunchNewImage returned\n\0".as_ptr() as *const c_char);
    }
}

extern "C" {
    fn Sys_StreamShutdown();
}

/*
==================
WinMain

==================
*/
#[cfg(target_os = "windows")]
pub extern "C" fn main() -> c_int {
    unsafe {
        //	Z_SetFreeOSMem();

        // I'm going to kill someone. This should not be necessary. No, really.
        Direct3D_SetPushBufferSize(1024 * 1024, 128 * 1024);

        // get the initial time base
        Sys_Milliseconds();

        Win_Init();
        Com_Init(b"\0".as_ptr() as *const c_char);

        //Start sound early.  The STL inside will allocate memory and we don't
        //want that memory in the middle of the zone.
        if addr_of!(cls).read().soundRegistered == 0 {
            addr_of_mut!(cls).write({
                let mut c = addr_of!(cls).read();
                c.soundRegistered = qtrue as c_int;
                c
            });
            S_BeginRegistration();
        }

        NET_Init();

        // main game loop
        loop {
            IN_Frame();
            Com_Frame();

            // Do any XBL stuff
            //		XBL_Tick();

            // Poll debug console for new commands
            #[cfg(not(feature = "final_build"))]
            {
                DebugConsoleHandleCommands();
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub extern "C" fn main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    unsafe {
        // I'm going to kill someone. This should not be necessary. No, really.
        Direct3D_SetPushBufferSize(1024 * 1024, 128 * 1024);

        // get the initial time base
        Sys_Milliseconds();

        Win_Init();
        Com_Init(b"\0".as_ptr() as *const c_char);

        //Start sound early.  The STL inside will allocate memory and we don't
        //want that memory in the middle of the zone.
        if addr_of!(cls).read().soundRegistered == 0 {
            addr_of_mut!(cls).write({
                let mut c = addr_of!(cls).read();
                c.soundRegistered = qtrue as c_int;
                c
            });
            S_BeginRegistration();
        }

        NET_Init();

        // main game loop
        loop {
            IN_Frame();
            Com_Frame();

            // Do any XBL stuff
            //		XBL_Tick();

            // Poll debug console for new commands
            #[cfg(not(feature = "final_build"))]
            {
                DebugConsoleHandleCommands();
            }
        }
    }
}

pub extern "C" fn Sys_GetClipboardData() -> *mut c_char {
    core::ptr::null_mut()
}

pub extern "C" fn Sys_StartProcess(_path: *mut c_char, _background: c_int) {
}

pub extern "C" fn Sys_OpenURL(_url: *mut c_char, _mode: c_int) {
}

pub extern "C" fn Sys_Quit() {
}

pub extern "C" fn Sys_ShowConsole(_level: c_int, _quitOnClose: c_int) {
}

pub extern "C" fn Sys_Mkdir(_path: *const c_char) {
}

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

#[cfg(feature = "jk2mp")]
pub extern "C" fn Sys_ListFiles(directory: *const c_char, extension: *const c_char, filter: *mut c_char, numfiles: *mut c_int, wantsubs: c_int) -> *mut *mut c_char {
    unsafe {
        // MP has extra filter paramter. We don't support that.
        if !filter.is_null() {
            // assert(!"Sys_ListFiles doesn't support filter on console!");
            return core::ptr::null_mut();
        }
        Sys_ListFiles_impl(directory, extension, numfiles, wantsubs)
    }
}

#[cfg(not(feature = "jk2mp"))]
pub extern "C" fn Sys_ListFiles(directory: *const c_char, extension: *const c_char, numfiles: *mut c_int, wantsubs: c_int) -> *mut *mut c_char {
    Sys_ListFiles_impl(directory, extension, numfiles, wantsubs)
}

fn Sys_ListFiles_impl(mut directory: *const c_char, mut extension: *const c_char, numfiles: *mut c_int, _wantsubs: c_int) -> *mut *mut c_char {
    unsafe {
        // Hax0red console version of Sys_ListFiles. We mangle our arguments to get a standard filename
        // That file should exist, and contain the list of files that meet this search criteria.
        let mut listFilename: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
        let mut listFile: *mut c_char;
        let mut curFile: *mut c_char;
        let mut end: *mut c_char;
        let mut nfiles: c_int;
        let mut retList: *mut *mut c_char;

        // S00per hack
        if !strstr(directory, b"d:\\base\\\0".as_ptr() as *const c_char).is_null() {
            directory = directory.add(8);
        }

        if extension.is_null() {
            extension = b"\0".as_ptr() as *const c_char;
        } else if *extension as c_int == b'/' as c_int && *extension.add(1) as c_int == 0 {
            // Passing a slash as extension will find directories
            extension = b"dir\0".as_ptr() as *const c_char;
        } else if *extension as c_int == b'.' as c_int {
            // Skip over leading .
            extension = extension.add(1);
        }

        // Build our filename
        Com_sprintf(addr_of_mut!(listFilename[0]), MAX_OSPATH as c_int, b"%s\\_console_%s_list_\0".as_ptr() as *const c_char, directory, extension);

        if FS_ReadFile(addr_of!(listFilename[0]), &mut listFile) <= 0 {
            if !listFile.is_null() {
                FS_FreeFile(listFile as *mut c_void);
            }
            Com_Printf(b"WARNING: List file %s not found\n\0".as_ptr() as *const c_char, addr_of!(listFilename[0]));
            if !numfiles.is_null() {
                *numfiles = 0;
            }
            return core::ptr::null_mut();
        }

        // Do a first pass to count number of files in the list
        nfiles = 0;
        curFile = listFile;
        loop {
            // Find end of line
            end = strchr(curFile, b'\r' as c_int);
            if !end.is_null() {
                // Should have a \n next -- skip them both
                end = end.add(2);
            } else {
                end = strchr(curFile, b'\n' as c_int);
                if !end.is_null() {
                    end = end.add(1);
                } else {
                    end = curFile.add(strlen(curFile));
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
            return core::ptr::null_mut();
        }

        // Allocate a file list, and quick string pool, but use LISTFILES
        retList = Z_Malloc(((nfiles + 1) * core::mem::size_of::<*mut c_char>()) as usize, TAG_LISTFILES, qfalse as c_int) as *mut *mut c_char;
        // Our string pool is actually slightly too large, but it's temporary, and that's better
        // than slightly too small
        let stringPool: *mut c_char = Z_Malloc(strlen(listFile) + 1, TAG_LISTFILES, qfalse as c_int) as *mut c_char;

        // Now go through the list of files again, and fill in the list to be returned
        nfiles = 0;
        curFile = listFile;
        let mut stringPool_cursor = stringPool;
        loop {
            // Find end of line
            end = strchr(curFile, b'\r' as c_int);
            if !end.is_null() {
                // Should have a \n next -- skip them both
                *end = 0;
                end = end.add(1);
                *end = 0;
                end = end.add(1);
            } else {
                end = strchr(curFile, b'\n' as c_int);
                if !end.is_null() {
                    *end = 0;
                    end = end.add(1);
                } else {
                    end = curFile.add(strlen(curFile));
                }
            }

            // Is the line empty?  If so, we're done.
            let curStrSize = strlen(curFile);
            if curStrSize < 1 {
                *retList.add(nfiles as usize) = core::ptr::null_mut();
                break;
            }

            // Alloc a small copy
            //retList[nfiles++] = CopyString( curFile );
            *retList.add(nfiles as usize) = stringPool_cursor;
            strcpy(stringPool_cursor, curFile);
            stringPool_cursor = stringPool_cursor.add(curStrSize + 1);
            nfiles += 1;

            // Advance to next line
            curFile = end;
        }

        // Free the special file's buffer
        FS_FreeFile(listFile as *mut c_void);

        retList
    }
}

extern "C" {
    fn strstr(s1: *const c_char, s2: *const c_char) -> *mut c_char;
}

/*
=================
Sys_UnloadGame
=================
*/
pub extern "C" fn Sys_UnloadGame() {
}

/*
=================
Sys_GetGameAPI

Loads the game dll
=================
*/
#[cfg(not(feature = "jk2mp"))]
pub extern "C" fn Sys_GetGameAPI(parms: *mut c_void) -> *mut c_void {
    unsafe {
        GetGameAPI(parms as *mut game_import_t) as *mut c_void
    }
}

/*
=================
Sys_LoadCgame

Used to hook up a development dll
=================
*/
// void * Sys_LoadCgame( void )
#[cfg(not(feature = "jk2mp"))]
pub extern "C" fn Sys_LoadCgame(entryPoint: *mut extern "C" fn(c_int) -> c_int, systemcalls: extern "C" fn(c_int) -> c_int) -> *mut c_void {
    unsafe {
        CG_PreInit();
        cg_dllEntry(systemcalls);
        *entryPoint = vmMain as extern "C" fn(c_int) -> c_int;
        core::ptr::null_mut()
    }
}

/* VVFIXME: More stubs */
pub extern "C" fn Sys_FileOutOfDate(_psFinalFileName: *const c_char /* dest */, _psDataFileName: *const c_char /* src */) -> c_int {
    qfalse as c_int
}

pub extern "C" fn Sys_CopyFile(_lpExistingFileName: *const c_char, _lpNewFileName: *const c_char, _bOverwrite: c_int) -> c_int {
    qfalse as c_int
}

pub extern "C" fn Sys_CheckCD() -> c_int {
    qtrue as c_int
}
