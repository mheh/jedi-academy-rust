#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void};
use std::ffi::CStr;

// Include local stubs and extern declarations
mod _stubs {
    use core::ffi::{c_char, c_int, c_void};

    // From mac_local.h - local stubs for Mac-specific APIs
    // These are platform-specific and handled by the header

    // Q3 engine/game API types
    pub type qboolean = c_int;
    pub const CPUID_GENERIC: c_int = 0;
    pub const MAX_OSPATH: usize = 256;
    pub const MAX_MSGLEN: usize = 16384;
    pub const MAX_FOUND_FILES: usize = 0x1000;

    // sysEventType_t
    #[derive(Clone, Copy)]
    pub enum sysEventType_t {
        SE_CONSOLE = 0,
        SE_PACKET = 1,
    }

    // sysEvent_t structure
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct sysEvent_t {
        pub evTime: c_int,
        pub evType: c_int,
        pub evValue: c_int,
        pub evValue2: c_int,
        pub evPtrLength: c_int,
        pub evPtr: *mut c_void,
    }

    // msg_t structure
    #[repr(C)]
    pub struct msg_t {
        pub data: *mut u8,
        pub cursize: c_int,
        pub maxsize: c_int,
    }

    // netadr_t structure
    #[repr(C)]
    pub struct netadr_t {
        pub type_: c_int,
        pub ip: [u8; 4],
        pub port: u16,
    }

    // cvar_t structure
    #[repr(C)]
    pub struct cvar_t {
        pub name: *const c_char,
        pub string: *const c_char,
        pub resetString: *const c_char,
        pub latched_string: *const c_char,
        pub flags: c_int,
        pub modified: c_int,
        pub modificationCount: c_int,
        pub value: f32,
        pub integer: c_int,
        pub next: *mut cvar_t,
        pub prev: *mut cvar_t,
    }

    // fileHandle_t
    pub type fileHandle_t = c_int;
}

use self::_stubs::*;

// Extern C declarations
extern "C" {
    fn Com_sprintf(buf: *mut c_char, bufsize: usize, fmt: *const c_char, ...) -> c_int;
    fn mkdir(path: *const c_char, mode: c_int) -> c_int;
    fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn sprintf(buf: *mut c_char, fmt: *const c_char, ...) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut std::ffi::c_void;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut std::ffi::c_void) -> usize;
    fn fclose(stream: *mut std::ffi::c_void) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    // Q3 engine functions
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
    fn Com_Init(commandLine: *const c_char);
    fn Com_Frame();
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_strcat(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn CopyString(string: *const c_char) -> *mut c_char;
    fn Z_Malloc(size: usize) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn FS_filelength(f: *mut std::ffi::c_void) -> usize;
    fn FS_Read(buffer: *mut c_void, len: usize, count: usize, f: fileHandle_t) -> c_int;
    fn FS_Seek(f: fileHandle_t, offset: c_int, origin: c_int);
    fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: usize);

    // Platform functions
    fn Sys_InitNetworking();
    fn Sys_InitInput();
    fn Sys_EndProfiling();
    fn Sys_ShutdownInput();
    fn Sys_ShutdownNetworking();
    fn Sys_SendKeyEvents();
    fn Sys_ConsoleInput() -> *mut c_char;
    fn Sys_Input();
    fn Sys_InitConsole();
    fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean;

    // Mac-specific functions (from DriverServices.h, etc.)
    fn MaxApplZone();
    fn MoreMasters();
    fn InitGraf(port: *mut c_void);
    fn InitFonts();
    fn FlushEvents(eventMask: c_int, stopMask: c_int);
    fn SetEventMask(eventMask: c_int);
    fn InitWindows();
    fn InitMenus();
    fn TEInit();
    fn InitDialogs(filterProc: *mut c_void);
    fn InitCursor();
    fn GetNewMBar(resID: c_int) -> *mut c_void;
    fn SetMenuBar(menubar: *mut c_void);
    fn DisposeHandle(h: *mut c_void);
    fn GetMenuHandle(menuID: c_int) -> *mut c_void;
    fn AppendResMenu(menu: *mut c_void, resType: c_int);
    fn DrawMenuBar();
    fn WaitNextEvent(eventMask: c_int, event: *mut c_void, sleep: c_int, mouseRgn: *mut c_void) -> qboolean;
    fn ccommand(argv: *mut *mut *mut c_char) -> c_int;
    fn ProfilerInit(collectType: c_int, timeBase: c_int, maxProfileLines: c_int, maxSymbols: c_int);
    fn ProfilerDump(filename: *const u8);
    fn ProfilerTerm();
    fn UpTime() -> c_void;
    fn AbsoluteToNanoseconds(t: c_void) -> c_void;
    fn PBGetCatInfoSync(paramBlock: *mut c_void) -> c_int;
    fn FSMakeFSSpec(vRefNum: c_int, dirID: c_int, fileName: *const u8, spec: *mut c_void) -> c_int;
    fn StopAlert(alertID: c_int, filterProc: *mut c_void) -> c_int;
    fn exit(code: c_int);
}

// Global variables from the C source
pub static mut sys_ticBase: c_int = 0;
pub static mut sys_msecBase: c_int = 0;
pub static mut sys_lastEventTic: c_int = 0;

pub static mut sys_profile: *mut cvar_t = std::ptr::null_mut();
pub static mut sys_waitNextEvent: *mut cvar_t = std::ptr::null_mut();

// Event queue globals
static mut eventQue: [sysEvent_t; 256] = [sysEvent_t {
    evTime: 0,
    evType: 0,
    evValue: 0,
    evValue2: 0,
    evPtrLength: 0,
    evPtr: std::ptr::null_mut(),
}; 256];
static mut eventHead: c_int = 0;
static mut eventTail: c_int = 0;
static mut sys_packetReceived: [u8; 16384] = [0; 16384];

static mut sys_profiling: qboolean = 0;

const MAX_QUED_EVENTS: usize = 256;
const MASK_QUED_EVENTS: c_int = 255; // MAX_QUED_EVENTS - 1

const ERR_FATAL: c_int = 3;

//===========================================================================

pub fn putenv(buffer: *mut c_char) {
    // the mac doesn't seem to have the concept of environment vars, so nop this
}

pub fn Sys_UnloadGame() {}

pub fn Sys_GetGameAPI(parms: *mut c_void) -> *mut c_void {
    extern "C" {
        fn GetGameAPI(import: *mut c_void) -> *mut c_void;
    }
    // we are hard-linked in, so no need to load anything
    unsafe { GetGameAPI(parms) }
}

pub fn Sys_UnloadUI() {}

pub fn Sys_GetUIAPI() -> *mut c_void {
    extern "C" {
        fn GetUIAPI() -> *mut c_void;
    }
    // we are hard-linked in, so no need to load anything
    unsafe { GetUIAPI() }
}

pub fn Sys_UnloadBotLib() {}

pub fn Sys_GetBotLibAPI(parms: *mut c_void) -> *mut c_void {
    std::ptr::null_mut()
}

pub fn Sys_GetBotAIAPI(parms: *mut c_void) -> *mut c_void {
    std::ptr::null_mut()
}

extern "C" {
    fn dllEntry(syscallptr: extern "C" fn(c_int, ...) -> c_int);
    fn vmMain(command: c_int, ...) -> c_int;
}

pub fn Sys_LoadDll(
    name: *const c_char,
    entryPoint: *mut extern "C" fn(c_int, ...) -> c_int,
    systemCalls: extern "C" fn(c_int, ...) -> c_int,
) -> *mut c_void {
    unsafe {
        dllEntry(systemCalls);
        *entryPoint = vmMain;
    }
    1 as *mut c_void
}

pub fn Sys_UnloadDll(_dllHandle: *mut c_void) {}

//===========================================================================

pub fn Sys_GetClipboardData() -> *mut c_char {
    // FIXME
    std::ptr::null_mut()
}

pub fn Sys_GetProcessorId() -> c_int {
    CPUID_GENERIC
}

pub fn Sys_Mkdir(path: *const c_char) {
    let mut ospath: [c_char; 256] = [0; 256];
    let err: c_int;

    unsafe {
        Com_sprintf(
            ospath.as_mut_ptr(),
            std::mem::size_of_val(&ospath),
            b"%s:\0".as_ptr() as *const c_char,
            path,
        );

        err = mkdir(ospath.as_ptr(), 0o777);
    }
}

pub fn Sys_Cwd() -> *mut c_char {
    static mut dir: [c_char; 256] = [0; 256];
    let l: usize;

    unsafe {
        getcwd(dir.as_mut_ptr(), std::mem::size_of_val(&dir));
        dir[255] = 0; // MAX_OSPATH-1 = 0

        // strip off the last colon
        l = strlen(dir.as_ptr());
        if l > 0 {
            dir[l - 1] = 0;
        }
        dir.as_mut_ptr()
    }
}

pub fn Sys_DefaultCDPath() -> *mut c_char {
    b"\0".as_ptr() as *mut c_char
}

pub fn Sys_DefaultBasePath() -> *mut c_char {
    Sys_Cwd()
}

/*
 =================================================================================

 FILE FINDING

 =================================================================================
*/

pub fn PStringToCString(s: *mut c_char) -> c_int {
    let mut l: c_int;
    let mut i: c_int;

    unsafe {
        l = (*(s as *mut u8)) as c_int;
        i = 0;
        while i < l {
            *(s.offset(i as isize)) = *(s.offset((i + 1) as isize));
            i += 1;
        }
        *(s.offset(l as isize)) = 0;
    }
    l
}

pub fn CStringToPString(s: *mut c_char) -> c_int {
    let l: c_int;
    let mut i: c_int;

    unsafe {
        l = strlen(s) as c_int;
        i = 0;
        while i < l {
            *(s.offset((l - i) as isize)) = *(s.offset((l - i - 1) as isize));
            i += 1;
        }
        *(s as *mut u8) = l as u8;
    }
    l
}

pub fn Sys_ListFiles(
    directory: *const c_char,
    extension: *const c_char,
    numfiles: *mut c_int,
    wantsubs: qboolean,
) -> *mut *mut c_char {
    let mut nfiles: c_int;
    let mut listCopy: *mut *mut c_char;
    let mut pdirectory: [c_char; 256] = [0; 256];
    let mut list: [*mut c_char; 4096] = [std::ptr::null_mut(); 4096];
    let mut findhandle: c_int;
    let mut directoryFlag: c_int;
    let mut i: c_int;
    let mut extensionLength: c_int;
    let mut VRefNum: c_int;
    let mut DrDirId: c_int;
    let mut index: c_int;
    let mut fsspec: [u8; 80] = [0; 80]; // FSSpec is a Mac struct; stub size

    unsafe {
        // get the volume and directory numbers
        // there has to be a better way than this...
        {
            let mut paramBlock: [u8; 512] = [0; 512]; // CInfoPBRec stub

            Q_strncpyz(pdirectory.as_mut_ptr(), directory, std::mem::size_of_val(&pdirectory));
            CStringToPString(pdirectory.as_mut_ptr());
            FSMakeFSSpec(0, 0, pdirectory.as_ptr() as *const u8, fsspec.as_mut_ptr());

            // Access stub fields (Mac-specific, stubbed)
            VRefNum = 0; // Would be fsspec.vRefNum
            DrDirId = 0; // Would be DrDirId from paramBlock

            memset(paramBlock.as_mut_ptr() as *mut c_void, 0, std::mem::size_of_val(&paramBlock));
            // Would set paramBlock.hFileInfo.ioNamePtr = pdirectory
            // Would call PBGetCatInfoSync(&paramBlock)
        }

        if extension.is_null() {
            extension = b"\0".as_ptr() as *const c_char;
        }
        extensionLength = strlen(extension) as c_int;

        if wantsubs != 0 || (*extension == b'/' as c_char && *(extension.offset(1)) == 0) {
            directoryFlag = 16;
        } else {
            directoryFlag = 0;
        }

        nfiles = 0;

        index = 1;
        loop {
            let mut paramBlock: [u8; 512] = [0; 512]; // CInfoPBRec stub
            let mut fileName: [c_char; 256] = [0; 256];
            let mut length: c_int;
            let mut err: c_int;

            memset(paramBlock.as_mut_ptr() as *mut c_void, 0, std::mem::size_of_val(&paramBlock));
            // Would set paramBlock fields

            err = PBGetCatInfoSync(paramBlock.as_mut_ptr() as *mut c_void) as c_int;

            if err != 0 {
                // noErr = 0
                break;
            }

            // Check directoryFlag (stubbed Mac behavior)
            if directoryFlag != 0 {
                continue; // simplified stub
            }

            // convert filename to C string
            length = PStringToCString(fileName.as_mut_ptr());

            // check the extension
            if directoryFlag == 0 {
                if length < extensionLength {
                    continue;
                }
                if Q_stricmp(
                    fileName.as_ptr().offset((length - extensionLength) as isize),
                    extension,
                ) != 0
                {
                    continue;
                }
            }

            // add this file
            if nfiles == 4095 {
                // MAX_FOUND_FILES - 1
                break;
            }
            list[nfiles as usize] = CopyString(fileName.as_ptr());
            nfiles += 1;

            index += 1;
        }

        list[nfiles as usize] = std::ptr::null_mut();

        // return a copy of the list
        *numfiles = nfiles;

        if nfiles == 0 {
            return std::ptr::null_mut();
        }

        listCopy = Z_Malloc(((nfiles + 1) * std::mem::size_of::<*mut c_char>()) as usize)
            as *mut *mut c_char;
        i = 0;
        while i < nfiles {
            *(listCopy.offset(i as isize)) = list[i as usize];
            i += 1;
        }
        *(listCopy.offset(i as isize)) = std::ptr::null_mut();

        listCopy
    }
}

pub fn Sys_FreeFileList(list: *mut *mut c_char) {
    let mut i: c_int = 0;

    unsafe {
        if list.is_null() {
            return;
        }

        loop {
            if (*list.offset(i as isize)).is_null() {
                break;
            }
            Z_Free(*list.offset(i as isize) as *mut c_void);
            i += 1;
        }

        Z_Free(list as *mut c_void);
    }
}

//===================================================================

/*
================
Sys_Init

The cvar and file system has been setup, so configurations are loaded
================
*/
pub fn Sys_Init() {
    unsafe {
        Sys_InitNetworking();
        Sys_InitInput();
    }
}

/*
=================
Sys_Shutdown
=================
*/
pub fn Sys_Shutdown() {
    unsafe {
        Sys_EndProfiling();
        Sys_ShutdownInput();
        Sys_ShutdownNetworking();
    }
}

/*
=================
Sys_BeginProfiling
=================
*/
pub fn Sys_BeginProfiling() {
    unsafe {
        if (*sys_profile).integer == 0 {
            return;
        }
        ProfilerInit(1, 0, 16384, 64); // collectDetailed, bestTimeBase
        sys_profiling = 1; // qtrue
    }
}

/*
=================
Sys_EndProfiling
=================
*/
pub fn Sys_EndProfiling() {
    let mut pstring: [u8; 1024] = [0; 1024];

    unsafe {
        if sys_profiling == 0 {
            return;
        }
        sys_profiling = 0; // qfalse

        let basepath = Cvar_VariableString(b"fs_basepath\0".as_ptr() as *const c_char);
        sprintf(
            (pstring.as_mut_ptr() as *mut c_char).offset(1),
            b"%s:profile.txt\0".as_ptr() as *const c_char,
            basepath,
        );
        pstring[0] = strlen((pstring.as_mut_ptr() as *const c_char).offset(1)) as u8;
        ProfilerDump(pstring.as_ptr());
        ProfilerTerm();
    }
}

//================================================================================

/*
================
Sys_Milliseconds
================
*/
pub fn Sys_Milliseconds() -> c_int {
    // Simplified stub - Mac UpTime/AbsoluteTime APIs are platform-specific
    // This would need actual Mac implementation with UpTime() and AbsoluteToNanoseconds()
    // For now, return 0 as a stub
    unsafe {
        // The original code uses:
        // t = UpTime();
        // nano = AbsoluteToNanoseconds( t );
        // doub = (((double) nano.hi) * kTwoPower32) + nano.lo;
        // return doub * 0.000001;
        // These are Mac-specific calls that require proper Mac SDK
    }
    0
}

/*
================
Sys_Error
================
*/
pub fn Sys_Error(error: *const c_char) {
    let mut argptr: std::ffi::VaList;
    let mut string: [c_char; 1024] = [0; 1024];
    let mut string2: [c_char; 1024] = [0; 1024];

    unsafe {
        Sys_Shutdown();

        // Note: vsprintf requires va_list, which is complex to handle from Rust
        // Stub implementation - would need proper varargs handling
        sprintf(
            (string2.as_mut_ptr() as *mut c_char).offset(1),
            b"Error (varargs not implemented in Rust stub)\0".as_ptr() as *const c_char,
        );
        string2[0] = strlen((string2.as_mut_ptr() as *const c_char).offset(1)) as c_char;

        strcpy(
            (string.as_mut_ptr() as *mut c_char).offset(1),
            b"Quake 3 Error:\0".as_ptr() as *const c_char,
        );
        string[0] = strlen((string.as_mut_ptr() as *const c_char).offset(1)) as c_char;

        // set the dialog box strings
        // ParamText( (unsigned char *)string, (unsigned char *)string2,
        // (unsigned char *)string2, (unsigned char *)string2 );

        // run a dialog
        StopAlert(128, std::ptr::null_mut());

        exit(0);
    }
}

/*
================
Sys_Quit
================
*/
pub fn Sys_Quit() {
    Sys_Shutdown();
    unsafe {
        exit(0);
    }
}

//===================================================================

pub fn Sys_BeginStreamedFile(_f: fileHandle_t, _readAhead: c_int) {}

pub fn Sys_EndStreamedFile(_f: fileHandle_t) {}

pub fn Sys_StreamedRead(
    buffer: *mut c_void,
    size: c_int,
    count: c_int,
    f: fileHandle_t,
) -> c_int {
    unsafe { FS_Read(buffer, size as usize, count as usize, f) }
}

pub fn Sys_StreamSeek(f: fileHandle_t, offset: c_int, origin: c_int) {
    unsafe {
        FS_Seek(f, offset, origin);
    }
}

//=================================================================================

/*
========================================================================

EVENT LOOP

========================================================================
*/

/*
================
Sys_QueEvent

A time of 0 will get the current time
Ptr should either be null, or point to a block of data that can
be freed by the game later.
================
*/
pub fn Sys_QueEvent(
    mut time: c_int,
    event_type: c_int,
    value: c_int,
    value2: c_int,
    ptrLength: c_int,
    ptr: *mut c_void,
) {
    let ev: *mut sysEvent_t;

    unsafe {
        ev = unsafe_addr_of_mut!(eventQue[(eventHead & MASK_QUED_EVENTS) as usize]);
        if eventHead - eventTail >= MAX_QUED_EVENTS as c_int {
            Com_Printf(b"Sys_QueEvent: overflow\n\0".as_ptr() as *const c_char);
            // we are discarding an event, but don't leak memory
            if !(*ev).evPtr.is_null() {
                free((*ev).evPtr);
            }
            eventTail += 1;
        }
        eventHead += 1;

        if time == 0 {
            time = Sys_Milliseconds();
        }

        (*ev).evTime = time;
        (*ev).evType = event_type;
        (*ev).evValue = value;
        (*ev).evValue2 = value2;
        (*ev).evPtrLength = ptrLength;
        (*ev).evPtr = ptr;
    }
}

// Helper macro for unsafe addr_of_mut (since we can't use it without nightly or std::ptr::addr_of_mut)
macro_rules! unsafe_addr_of_mut {
    ($e:expr) => {
        &mut $e as *mut _
    };
}

/*
=================
Sys_PumpEvents
=================
*/
pub fn Sys_PumpEvents() {
    let s: *mut c_char;
    let mut netmsg: msg_t = msg_t {
        data: std::ptr::null_mut(),
        cursize: 0,
        maxsize: 0,
    };
    let mut adr: netadr_t = netadr_t {
        type_: 0,
        ip: [0; 4],
        port: 0,
    };

    unsafe {
        // pump the message loop
        Sys_SendKeyEvents();

        // check for console commands
        s = Sys_ConsoleInput();
        if !s.is_null() {
            let b: *mut c_char;
            let len: usize;

            len = strlen(s) + 1;
            b = malloc(len) as *mut c_char;
            if b.is_null() {
                Com_Error(ERR_FATAL, b"malloc failed in Sys_PumpEvents\0".as_ptr() as *const c_char);
            }
            strcpy(b, s);
            Sys_QueEvent(0, SE_CONSOLE as c_int, 0, 0, len as c_int, b as *mut c_void);
        }

        // check for other input devices
        Sys_Input();

        // check for network packets
        MSG_Init(
            &mut netmsg,
            unsafe_addr_of_mut!(sys_packetReceived[0]),
            std::mem::size_of_val(&sys_packetReceived),
        );
        if Sys_GetPacket(&mut adr, &mut netmsg) != 0 {
            let buf: *mut netadr_t;
            let len: usize;

            // copy out to a seperate buffer for qeueing
            len = std::mem::size_of::<netadr_t>() + netmsg.cursize as usize;
            buf = malloc(len) as *mut netadr_t;
            if buf.is_null() {
                Com_Error(ERR_FATAL, b"malloc failed in Sys_PumpEvents\0".as_ptr() as *const c_char);
            }
            *buf = adr;
            memcpy(
                buf.offset(1) as *mut c_void,
                netmsg.data as *const c_void,
                netmsg.cursize as usize,
            );
            Sys_QueEvent(
                0,
                SE_PACKET as c_int,
                0,
                0,
                len as c_int,
                buf as *mut c_void,
            );
        }
    }
}

/*
================
Sys_GetEvent

================
*/
pub fn Sys_GetEvent() -> sysEvent_t {
    let mut ev: sysEvent_t = sysEvent_t {
        evTime: 0,
        evType: 0,
        evValue: 0,
        evValue2: 0,
        evPtrLength: 0,
        evPtr: std::ptr::null_mut(),
    };

    unsafe {
        if eventHead == eventTail {
            Sys_PumpEvents();
        }
        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) & MASK_QUED_EVENTS) as usize];
        }

        // create an empty event to return
        memset(&mut ev as *mut _ as *mut c_void, 0, std::mem::size_of_val(&ev));
        ev.evTime = Sys_Milliseconds();

        // track the mac event "when" to milliseconds rate
        sys_ticBase = sys_lastEventTic;
        sys_msecBase = ev.evTime;

        ev
    }
}

/*
=============
InitMacStuff
=============
*/
pub fn InitMacStuff() {
    let menuBar: *mut c_void;
    let _dir: [c_char; 256] = [0; 256];

    unsafe {
        // init toolbox
        MaxApplZone();
        MoreMasters();

        InitGraf(std::ptr::null_mut()); // &qd.thePort - stubbed
        InitFonts();
        FlushEvents(-1, 0); // everyEvent = -1
        SetEventMask(-1);
        InitWindows();
        InitMenus();
        TEInit();
        InitDialogs(std::ptr::null_mut());
        InitCursor();

        // init menu
        menuBar = GetNewMBar(128); // rMenuBar = 128
        if menuBar.is_null() {
            Com_Error(
                ERR_FATAL,
                b"MenuBar not found.\0".as_ptr() as *const c_char,
            );
        }

        SetMenuBar(menuBar);
        DisposeHandle(menuBar);
        AppendResMenu(GetMenuHandle(1), 68); // mApple = 1, 'DRVR' = 68
        DrawMenuBar();

        Sys_InitConsole();

        SetEventMask(-1);
    }
}

//==================================================================================

/*
=============
ReadCommandLineParms

Read startup options from a text file or dialog box
=============
*/
pub fn ReadCommandLineParms() -> *mut c_char {
    let f: *mut std::ffi::c_void;
    let len: usize;
    let buf: *mut c_char;
    let mut event: [u8; 256] = [0; 256]; // EventRecord stub

    unsafe {
        // flush out all the events and see if shift is held down
        // to bring up the args window
        loop {
            if WaitNextEvent(-1, event.as_mut_ptr() as *mut c_void, 0, std::ptr::null_mut()) == 0 {
                break;
            }
        }
        // Check for shift key (stubbed - would check event.modifiers & 512)

        // otherwise check for a parms file
        f = fopen(b"MacQuake3Parms.txt\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
        if f.is_null() {
            return b"\0".as_ptr() as *mut c_char;
        }
        len = FS_filelength(f);
        buf = malloc(len + 1) as *mut c_char;
        if buf.is_null() {
            exit(1);
        }
        *(buf.offset(len as isize)) = 0;
        fread(buf as *mut c_void, len, 1, f);
        fclose(f);

        buf
    }
}

/*
=============
main
=============
*/
pub fn main_mac() {
    let commandLine: *mut c_char;

    InitMacStuff();

    commandLine = ReadCommandLineParms();

    unsafe {
        Com_Init(commandLine);

        sys_profile = Cvar_Get(b"sys_profile\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        (*sys_profile).modified = 0; // qfalse

        sys_waitNextEvent = Cvar_Get(b"sys_waitNextEvent\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

        loop {
            // run the frame
            Com_Frame();

            if (*sys_profile).modified != 0 {
                (*sys_profile).modified = 0; // qfalse
                if (*sys_profile).integer != 0 {
                    Com_Printf(b"Beginning profile.\n\0".as_ptr() as *const c_char);
                    Sys_BeginProfiling();
                } else {
                    Com_Printf(b"Ending profile.\n\0".as_ptr() as *const c_char);
                    Sys_EndProfiling();
                }
            }
        }
    }
}
