// win_main.h

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::mem;
use core::ptr::{addr_of, addr_of_mut, null, null_mut};

// Simplified Win32 type aliases for structures we need
type HANDLE = *mut c_void;
type HINSTANCE = *mut c_void;
type HWND = *mut c_void;
type DWORD = u32;
type LONG = i32;
type LPCSTR = *const c_char;
type LPSTR = *mut c_char;
type BOOL = i32;
type BYTE = u8;
type LPTSTR = *mut c_char;

const INVALID_HANDLE_VALUE: HANDLE = -1isize as *mut c_void;

// The following macros set and clear, respectively, given bits
// of the C runtime library debug flag, as specified by a bitmask.

#[cfg(debug_assertions)]
macro_rules! SET_CRT_DEBUG_FIELD {
    ($a:expr) => {
        unsafe { _CrtSetDbgFlag(($a) | _CrtSetDbgFlag(_CRTDBG_REPORT_FLAG)) }
    };
}

#[cfg(not(debug_assertions))]
macro_rules! SET_CRT_DEBUG_FIELD {
    ($a:expr) => {
        ()
    };
}

#[cfg(debug_assertions)]
macro_rules! CLEAR_CRT_DEBUG_FIELD {
    ($a:expr) => {
        unsafe { _CrtSetDbgFlag(!($a) & _CrtSetDbgFlag(_CRTDBG_REPORT_FLAG)) }
    };
}

#[cfg(not(debug_assertions))]
macro_rules! CLEAR_CRT_DEBUG_FIELD {
    ($a:expr) => {
        ()
    };
}

const CD_BASEDIR: &str = "gamedata\\gamedata";
const CD_EXE: &str = "jasp.exe";
const CD_VOLUME: &str = "JEDIACAD";

const MEM_THRESHOLD: i64 = 128 * 1024 * 1024;

static mut sys_cmdline: [c_char; 4096] = [0; 4096]; // MAX_STRING_CHARS is likely 4096

// Stub types for Windows structures
#[repr(C)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

#[repr(C)]
pub struct MEMORYSTATUS {
    pub dwLength: DWORD,
    pub dwMemoryLoad: DWORD,
    pub dwTotalPhys: DWORD,
    pub dwAvailPhys: DWORD,
    pub dwTotalPageFile: DWORD,
    pub dwAvailPageFile: DWORD,
    pub dwTotalVirtual: DWORD,
    pub dwAvailVirtual: DWORD,
}

#[repr(C)]
pub struct _finddata_t {
    pub attrib: c_int,
    pub time_write: i64,
    pub size: i64,
    pub name: [c_char; 260],
}

#[repr(C)]
pub struct SYSTEMTIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDayOfWeek: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}

#[repr(C)]
pub struct OSVERSIONINFO {
    pub dwOSVersionInfoSize: DWORD,
    pub dwMajorVersion: DWORD,
    pub dwMinorVersion: DWORD,
    pub dwBuildNumber: DWORD,
    pub dwPlatformId: DWORD,
    pub szCSDVersion: [c_char; 128],
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: usize,
    pub lParam: isize,
    pub time: DWORD,
    pub pt: (i32, i32),
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
pub struct msg_t {
    pub allowoverflow: bool,
    pub overflowed: bool,
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
}

#[repr(C)]
pub struct streamState_t {
    pub threadHandle: HANDLE,
    pub threadId: c_int,
    pub crit: [u8; 40], // CRITICAL_SECTION is opaque, size varies
    pub file: u32, // fileHandle_t
    pub buffer: *mut u8,
    pub eof: c_int, // qboolean
    pub bufferSize: c_int,
    pub streamPosition: c_int,
    pub threadPosition: c_int,
}

// Windows API extern declarations
extern "C" {
    fn CreateFile(
        lpFileName: LPCSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *const c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;

    fn CloseHandle(hObject: HANDLE) -> BOOL;

    fn GetFileTime(
        hFile: HANDLE,
        lpCreationTime: *mut FILETIME,
        lpLastAccessTime: *mut FILETIME,
        lpLastWriteTime: *mut FILETIME,
    ) -> BOOL;

    fn CopyFile(lpExistingFileName: LPCSTR, lpNewFileName: LPCSTR, bFailIfExists: BOOL) -> BOOL;

    fn GetFileAttributes(lpFileName: LPCSTR) -> DWORD;

    fn SetFileAttributes(lpFileName: LPCSTR, dwFileAttributes: DWORD) -> BOOL;

    fn GlobalMemoryStatus(lpBuffer: *mut MEMORYSTATUS);

    fn GetDriveType(lpRootPathName: LPCSTR) -> DWORD;

    fn GetVolumeInformation(
        lpRootPathName: LPCSTR,
        lpVolumeNameBuffer: LPSTR,
        nVolumeNameSize: DWORD,
        lpVolumeSerialNumber: *mut DWORD,
        lpMaximumComponentLength: *mut DWORD,
        lpFileSystemFlags: *mut DWORD,
        lpFileSystemNameBuffer: LPSTR,
        nFileSystemNameSize: DWORD,
    ) -> BOOL;

    fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;

    fn GetClipboardData(uFormat: DWORD) -> HANDLE;

    fn GlobalLock(hMem: HANDLE) -> *mut c_void;

    fn GlobalUnlock(hMem: HANDLE) -> BOOL;

    fn GlobalSize(hMem: HANDLE) -> usize;

    fn CloseClipboard() -> BOOL;

    fn LoadLibrary(lpLibFileName: LPCSTR) -> HINSTANCE;

    fn FreeLibrary(hLibModule: HINSTANCE) -> BOOL;

    fn GetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> *mut c_void;

    fn CreateThread(
        lpThreadAttributes: *const c_void,
        dwStackSize: usize,
        lpStartAddress: extern "C" fn(*mut c_void) -> DWORD,
        lpParameter: *mut c_void,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;

    fn InitializeCriticalSection(lpCriticalSection: *mut c_void);

    fn EnterCriticalSection(lpCriticalSection: *mut c_void);

    fn LeaveCriticalSection(lpCriticalSection: *mut c_void);

    fn DeleteCriticalSection(lpCriticalSection: *mut c_void);

    fn PeekMessage(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
        wRemoveMsg: u32,
    ) -> BOOL;

    fn GetMessage(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> BOOL;

    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;

    fn DispatchMessage(lpMsg: *const MSG) -> isize;

    fn Sleep(dwMilliseconds: DWORD);

    fn timeBeginPeriod(uPeriod: u32) -> u32;

    fn timeEndPeriod(uPeriod: u32) -> u32;

    fn timeGetTime() -> DWORD;

    fn GetVersionEx(lpVersionInfo: *mut OSVERSIONINFO) -> BOOL;

    fn GetLastError() -> DWORD;

    fn FormatMessage(
        dwFlags: DWORD,
        lpSource: *const c_void,
        dwMessageId: DWORD,
        dwLanguageId: DWORD,
        lpBuffer: *mut LPSTR,
        nSize: DWORD,
        Arguments: *const c_void,
    ) -> DWORD;

    fn SetErrorMode(uMode: u32) -> u32;

    fn _findfirst(filespec: LPCSTR, fileinfo: *mut _finddata_t) -> isize;

    fn _findnext(handle: isize, fileinfo: *mut _finddata_t) -> c_int;

    fn _findclose(handle: isize) -> c_int;

    fn _mkdir(path: LPCSTR) -> c_int;

    fn _getcwd(buffer: LPSTR, maxlen: c_int) -> LPSTR;

    fn _CrtSetDbgFlag(flags: c_int) -> c_int;

    fn strcpy(dest: LPSTR, src: LPCSTR) -> LPSTR;

    fn strlen(s: LPCSTR) -> usize;

    fn memcpy(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;

    fn memset(dest: *mut c_void, c: c_int, count: usize) -> *mut c_void;

    fn strtok(str: LPSTR, delim: LPCSTR) -> LPSTR;

    fn MessageBox(hWnd: HWND, lpText: LPCSTR, lpCaption: LPCSTR, uType: u32) -> c_int;

    fn fopen(filename: LPCSTR, mode: LPCSTR) -> *mut c_void;

    fn fclose(stream: *mut c_void) -> c_int;

    fn sprintf(buffer: LPSTR, format: LPCSTR, ...) -> c_int;

    fn vsprintf(buffer: LPSTR, format: LPCSTR, arglist: *const c_void) -> c_int;

    fn malloc(size: usize) -> *mut c_void;

    fn free(ptr: *mut c_void);

    fn abs(x: c_int) -> c_int;
}

// External engine functions
extern "C" {
    fn Conbuf_AppendText(text: LPCSTR);
    fn Sys_SetErrorText(text: LPCSTR);
    fn Sys_ShowConsole(show: c_int, qbool: c_int);
    fn IN_Shutdown();
    fn Com_Quit_f();
    fn Sys_DestroyConsole();
    fn Com_ShutdownZoneMemory();
    fn Com_ShutdownHunkMemory();
    fn Com_Printf(fmt: LPCSTR, ...);
    fn Sys_ConsoleInput() -> LPSTR;
    fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);
    fn FS_Read(buffer: *mut c_void, len: c_int, f: u32) -> c_int;
    fn FS_Seek(f: u32, offset: c_int, origin: c_int);
    fn CopyString(in_: LPCSTR) -> LPSTR;
    fn Z_Malloc(size: usize, tag: c_int, qbool: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Com_Error(level: c_int, fmt: LPCSTR, ...);
    fn Com_DPrintf(fmt: LPCSTR, ...);
    fn Cvar_Get(name: LPCSTR, value: LPCSTR, flags: c_int) -> *mut c_void;
    fn Cvar_Set(name: LPCSTR, value: LPCSTR);
    fn Cvar_SetValue(name: LPCSTR, value: f32);
    fn Cvar_VariableString(name: LPCSTR) -> LPCSTR;
    fn Cmd_AddCommand(name: LPCSTR, f: extern "C" fn());
    fn Sys_Milliseconds() -> c_int;
    fn Sys_GetProcessorId() -> c_int;
    fn Sys_GetCurrentUser() -> LPSTR;
    fn Com_Init(cmdline: LPCSTR);
    fn Com_Frame();
    fn Sys_CreateConsole();
    fn Q_stricmp(s1: LPCSTR, s2: LPCSTR) -> c_int;
    fn Q_strncpyz(dest: LPSTR, src: LPCSTR, size: usize);
    fn va(format: LPCSTR, ...) -> LPSTR;
    fn MainWndProc(hwnd: HWND, uMsg: u32, wParam: usize, lParam: isize) -> isize;
    fn Language_IsAsian() -> c_int;
    fn SE_GetString(label: LPCSTR) -> LPCSTR;
}

// Global structures
pub struct GWv {
    pub hInstance: HINSTANCE,
    pub osversion: OSVERSIONINFO,
    pub sysMsgTime: DWORD,
    pub isMinimized: c_int,
    pub activeApp: c_int,
}

static mut g_wv: GWv = GWv {
    hInstance: null_mut(),
    osversion: OSVERSIONINFO {
        dwOSVersionInfoSize: 0,
        dwMajorVersion: 0,
        dwMinorVersion: 0,
        dwBuildNumber: 0,
        dwPlatformId: 0,
        szCSDVersion: [0; 128],
    },
    sysMsgTime: 0,
    isMinimized: 0,
    activeApp: 0,
};

static mut com_developer: *mut c_void = null_mut();
static mut com_viewlog: *mut c_void = null_mut();

static mut game_library: HINSTANCE = null_mut();

static mut eventQue: [sysEvent_t; 256] = [
    sysEvent_t {
        evTime: 0,
        evType: 0,
        evValue: 0,
        evValue2: 0,
        evPtrLength: 0,
        evPtr: null_mut(),
    }; 256
];
static mut eventHead: c_int = 0;
static mut eventTail: c_int = 0;
static mut sys_packetReceived: [u8; 1400] = [0; 1400]; // MAX_MSGLEN is likely 1400

static mut stream: streamState_t = streamState_t {
    threadHandle: null_mut(),
    threadId: 0,
    crit: [0; 40],
    file: 0,
    buffer: null_mut(),
    eof: 0,
    bufferSize: 0,
    streamPosition: 0,
    threadPosition: 0,
};

// Constants
const MAX_OSPATH: usize = 256;
const MAX_STRING_CHARS: usize = 4096;
const MAX_MSGLEN: usize = 1400;
const MAX_FOUND_FILES: usize = 0x1000;
const MASK_QUED_EVENTS: c_int = 255; // MAX_QUED_EVENTS - 1
const MAX_QUED_EVENTS: c_int = 256;

const GENERIC_READ: DWORD = 0x80000000;
const FILE_SHARE_READ: DWORD = 0x00000001;
const OPEN_EXISTING: DWORD = 3;
const FILE_FLAG_NO_BUFFERING: DWORD = 0x20000000;
const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;

const CF_TEXT: DWORD = 1;

const DRIVE_CDROM: DWORD = 5;

const _A_SUBDIR: c_int = 0x10;

const _CRTDBG_REPORT_FLAG: c_int = 0x0F;
const _CRTDBG_LEAK_CHECK_DF: c_int = 0x20;

const SEM_FAILCRITICALERRORS: u32 = 0x0001;

const VER_PLATFORM_WIN32s: DWORD = 0;
const VER_PLATFORM_WIN32_WINDOWS: DWORD = 1;
const VER_PLATFORM_WIN32_NT: DWORD = 2;

const CPUID_GENERIC: c_int = 0;
const CPUID_INTEL_UNSUPPORTED: c_int = 1;
const CPUID_INTEL_PENTIUM: c_int = 2;
const CPUID_INTEL_MMX: c_int = 3;
const CPUID_INTEL_KATMAI: c_int = 4;
const CPUID_INTEL_WILLIAMETTE: c_int = 5;
const CPUID_AMD_3DNOW: c_int = 6;
const CPUID_AXP: c_int = 7;

const TAG_LISTFILES: c_int = 3;
const TAG_CLIPBOARD: c_int = 4;
const TAG_EVENT: c_int = 5;
const CVAR_ROM: c_int = 0x40;

const ERR_FATAL: c_int = 2;

const PM_NOREMOVE: u32 = 0;

const SE_CONSOLE: c_int = 0;

const MB_YESNO: u32 = 0x00000004;
const MB_ICONWARNING: u32 = 0x00000030;
const MB_TASKMODAL: u32 = 0x00002000;
const IDYES: c_int = 6;

const FORMAT_MESSAGE_ALLOCATE_BUFFER: DWORD = 0x00000100;
const FORMAT_MESSAGE_FROM_SYSTEM: DWORD = 0x00001000;

const LOWORD: fn(x: DWORD) -> DWORD = |x| x & 0xFFFF;

macro_rules! MAKELANGID {
    ($p:expr, $s:expr) => {
        (($s as DWORD) << 10 | ($p as DWORD))
    };
}

/*
==================
Sys_GetFileTime()
==================
*/
fn Sys_GetFileTime(psFileName: LPCSTR, ft: &mut FILETIME) -> bool {
    let mut bSuccess: bool = false;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;

    hFile = unsafe {
        CreateFile(
            psFileName, // LPCTSTR lpFileName,          // pointer to name of the file
            GENERIC_READ, // DWORD dwDesiredAccess,       // access (read-write) mode
            FILE_SHARE_READ, // DWORD dwShareMode,           // share mode
            null(), // LPSECURITY_ATTRIBUTES lpSecurityAttributes,	// pointer to security attributes
            OPEN_EXISTING, // DWORD dwCreationDisposition,  // how to create
            FILE_FLAG_NO_BUFFERING, // DWORD dwFlagsAndAttributes,   // file attributes
            null_mut(), // HANDLE hTemplateFile          // handle to file with attributes to
        )
    };

    if hFile != INVALID_HANDLE_VALUE {
        if unsafe {
            GetFileTime(
                hFile, // handle to file
                null_mut(), // LPFILETIME lpCreationTime
                null_mut(), // LPFILETIME lpLastAccessTime
                ft, // LPFILETIME lpLastWriteTime
            )
        } != 0
        {
            bSuccess = true;
        }

        unsafe {
            CloseHandle(hFile);
        }
    }

    return bSuccess;
}

/*
==================
Sys_FileOutOfDate()
==================
*/
fn Sys_FileOutOfDate(psFinalFileName: LPCSTR, psDataFileName: LPCSTR) -> c_int {
    let mut ftFinalFile: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut ftDataFile: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };

    if Sys_GetFileTime(psFinalFileName, &mut ftFinalFile)
        && Sys_GetFileTime(psDataFileName, &mut ftDataFile)
    {
        // timer res only accurate to within 2 seconds on FAT, so can't do exact compare...
        //
        //LONG l = CompareFileTime( &ftFinalFile, &ftDataFile );
        if (unsafe { abs((ftFinalFile.dwLowDateTime as c_int) - (ftDataFile.dwLowDateTime as c_int)) }
            as u32) <= 20000000
            && ftFinalFile.dwHighDateTime == ftDataFile.dwHighDateTime
        {
            return 0; // file not out of date, ie use it.
        }
        return 1; // flag return code to copy over a replacement version of this file
    }

    // extra error check, report as suspicious if you find a file locally but not out on the net.,.
    //
    unsafe {
        let com_developer_ptr = &com_developer as *const *mut c_void;
        if !com_developer_ptr.is_null() {
            let com_dev_struct = *com_developer_ptr as *mut c_int;
            if !com_dev_struct.is_null() && *com_dev_struct != 0 {
                if !Sys_GetFileTime(psDataFileName, &mut ftDataFile) {
                    Com_Printf(
                        b"Sys_FileOutOfDate: reading %s but it's not on the net!\n\0".as_ptr()
                            as LPCSTR,
                        psFinalFileName,
                    );
                }
            }
        }
    }

    return 0;
}

/*
==================
Sys_LowPhysicalMemory()
==================
*/
fn Sys_CopyFile(lpExistingFileName: LPCSTR, lpNewFileName: LPCSTR, bOverWrite: c_int) -> c_int {
    let mut bOk: c_int = 1; // qtrue
    if unsafe { CopyFile(lpExistingFileName, lpNewFileName, (bOverWrite == 0) as BOOL) } == 0
        && bOverWrite != 0
    {
        let dwAttrs: DWORD = unsafe { GetFileAttributes(lpNewFileName) };
        unsafe {
            SetFileAttributes(lpNewFileName, dwAttrs & !FILE_ATTRIBUTE_READONLY);
            bOk = CopyFile(lpExistingFileName, lpNewFileName, 0) as c_int;
        }
    }
    return bOk;
}

/*
==================
Sys_LowPhysicalMemory()
==================
*/
fn Sys_LowPhysicalMemory() -> c_int {
    static mut stat: MEMORYSTATUS = MEMORYSTATUS {
        dwLength: 0,
        dwMemoryLoad: 0,
        dwTotalPhys: 0,
        dwAvailPhys: 0,
        dwTotalPageFile: 0,
        dwAvailPageFile: 0,
        dwTotalVirtual: 0,
        dwAvailVirtual: 0,
    };
    static mut bAsked: c_int = 0; // qfalse
    let mut sys_lowmem: *mut c_void = null_mut();

    unsafe {
        if bAsked == 0 {
            // just in case it takes a little time for GlobalMemoryStatus() to gather stats on
            //	stuff we don't care about such as virtual mem etc.
            bAsked = 1; // qtrue
            GlobalMemoryStatus(&mut stat);
        }
        sys_lowmem = Cvar_Get(b"sys_lowmem\0".as_ptr() as LPCSTR, b"0\0".as_ptr() as LPCSTR, 0);

        let sys_lowmem_int_ptr = (sys_lowmem as *mut c_int).offset(2); // offset to integer field
        if !sys_lowmem_int_ptr.is_null() && *sys_lowmem_int_ptr != 0 {
            return 1; // qtrue
        }
        return if stat.dwTotalPhys <= MEM_THRESHOLD as u32 {
            1
        } else {
            0
        };
    }
}

/*
==================
Sys_BeginProfiling
==================
*/
fn Sys_BeginProfiling() {
    // this is just used on the mac build
}

/*
=============
Sys_Error

Show the early console as an error dialog
=============
*/
extern "C" fn Sys_Error(error: LPCSTR) {
    let mut argptr: *const c_void = std::ptr::null();
    let mut text: [c_char; 4096] = [0; 4096];
    let mut msg: MSG = unsafe { mem::zeroed() };

    // Note: This would require va_start/va_end which are difficult in Rust
    // Simplified version that doesn't use variadic args properly
    unsafe {
        let text_ptr = &mut text[0] as LPSTR;
        vsprintf(text_ptr, error, argptr);

        Conbuf_AppendText(&text[0] as LPCSTR);
        Conbuf_AppendText(b"\n\0".as_ptr() as LPCSTR);

        Sys_SetErrorText(&text[0] as LPCSTR);
        Sys_ShowConsole(1, 1); // qtrue

        timeEndPeriod(1);

        IN_Shutdown();

        // wait for the user to quit
        loop {
            if GetMessage(&mut msg, null_mut(), 0, 0) == 0 {
                Com_Quit_f();
            }
            TranslateMessage(&msg);
            DispatchMessage(&msg);
        }

        Sys_DestroyConsole();
        Com_ShutdownZoneMemory();
        Com_ShutdownHunkMemory();

        std::process::exit(1);
    }
}

/*
==============
Sys_Quit
==============
*/
fn Sys_Quit() {
    unsafe {
        timeEndPeriod(1);
        IN_Shutdown();
        Sys_DestroyConsole();
        Com_ShutdownZoneMemory();
        Com_ShutdownHunkMemory();

        std::process::exit(0);
    }
}

/*
==============
Sys_Print
==============
*/
fn Sys_Print(msg: LPCSTR) {
    unsafe {
        Conbuf_AppendText(msg);
    }
}

/*
==============
Sys_Mkdir
==============
*/
fn Sys_Mkdir(path: LPCSTR) {
    unsafe {
        _mkdir(path);
    }
}

/*
==============
Sys_Cwd
==============
*/
fn Sys_Cwd() -> LPSTR {
    static mut cwd: [c_char; 256] = [0; 256];

    unsafe {
        _getcwd(&mut cwd[0] as LPSTR, (mem::size_of_val(&cwd) - 1) as c_int);
        cwd[MAX_OSPATH - 1] = 0;

        return &mut cwd[0] as LPSTR;
    }
}

/*
==============
Sys_DefaultCDPath
==============
*/
fn Sys_DefaultCDPath() -> LPSTR {
    return b"\0".as_ptr() as LPSTR;
}

/*
==============
Sys_DefaultBasePath
==============
*/
fn Sys_DefaultBasePath() -> LPSTR {
    return Sys_Cwd();
}

/*
==============================================================

DIRECTORY SCANNING

==============================================================
*/

fn Sys_ListFiles(
    directory: LPCSTR,
    extension: LPCSTR,
    numfiles: &mut c_int,
    wantsubs: c_int,
) -> *mut *mut c_char {
    let mut search: [c_char; 256] = [0; 256];
    let mut nfiles: c_int = 0;
    let mut listCopy: *mut *mut c_char = null_mut();
    let mut list: [*mut c_char; 0x1000] = [null_mut(); 0x1000];
    let mut findinfo: _finddata_t = unsafe { mem::zeroed() };
    let mut findhandle: isize = 0;
    let mut flag: c_int = 0;
    let mut i: c_int = 0;

    let mut extension_ref = extension;
    if extension.is_null() {
        extension_ref = b"\0".as_ptr() as LPCSTR;
    }

    // passing a slash as extension will find directories
    unsafe {
        if *extension_ref == b'/' as c_char && *extension_ref.offset(1) == 0 {
            extension_ref = b"\0".as_ptr() as LPCSTR;
            flag = 0;
        } else {
            flag = _A_SUBDIR;
        }
    }

    unsafe {
        sprintf(
            &mut search[0] as LPSTR,
            b"%s\\*%s\0".as_ptr() as LPCSTR,
            directory,
            extension_ref,
        );
    }

    // search
    nfiles = 0;

    findhandle = unsafe { _findfirst(&search[0] as LPCSTR, &mut findinfo as *mut _finddata_t) };
    if findhandle == -1 {
        *numfiles = 0;
        return null_mut();
    }

    unsafe {
        loop {
            if (wantsubs == 0 && flag ^ (findinfo.attrib & _A_SUBDIR))
                || (wantsubs != 0 && (findinfo.attrib & _A_SUBDIR) != 0)
            {
                if nfiles == (MAX_FOUND_FILES - 1) as c_int {
                    break;
                }
                list[nfiles as usize] = CopyString(&findinfo.name[0] as LPCSTR);
                nfiles += 1;
            }
            if _findnext(findhandle, &mut findinfo as *mut _finddata_t) == -1 {
                break;
            }
        }

        list[nfiles as usize] = null_mut();

        _findclose(findhandle);

        // return a copy of the list
        *numfiles = nfiles;

        if nfiles == 0 {
            return null_mut();
        }

        listCopy = Z_Malloc(
            ((nfiles + 1) as usize) * mem::size_of::<*mut c_char>(),
            TAG_LISTFILES,
            0,
        ) as *mut *mut c_char;
        i = 0;
        while i < nfiles {
            *listCopy.offset(i as isize) = list[i as usize];
            i += 1;
        }
        *listCopy.offset(i as isize) = null_mut();

        return listCopy;
    }
}

fn Sys_FreeFileList(filelist: *mut *mut c_char) {
    let mut i: c_int = 0;

    if filelist.is_null() {
        return;
    }

    unsafe {
        while !(*filelist.offset(i as isize)).is_null() {
            Z_Free(*filelist.offset(i as isize) as *mut c_void);
            i += 1;
        }

        Z_Free(filelist as *mut c_void);
    }
}

//========================================================

/*
================
Sys_ScanForCD

Search all the drives to see if there is a valid CD to grab
the cddir from
================
*/
#[cfg(feature = "FINAL_BUILD")]
fn Sys_ScanForCD() -> c_int {
    let mut drive: [c_char; 4] = [0; 4];
    let mut f: *mut c_void = null_mut();
    let mut test: [c_char; 256] = [0; 256];

    drive[0] = b'c' as c_char;
    drive[1] = b':' as c_char;
    drive[2] = b'\\' as c_char;
    drive[3] = 0;

    // scan the drives
    unsafe {
        while drive[0] as u8 <= b'z' {
            if GetDriveType(&drive[0] as LPCSTR) == DRIVE_CDROM {
                let mut VolumeName: [c_char; 260] = [0; 260];
                let mut FileSystemName: [c_char; 260] = [0; 260];
                let mut VolumeSerialNumber: DWORD = 0;
                let mut MaximumComponentLength: DWORD = 0;
                let mut FileSystemFlags: DWORD = 0;

                let Result: BOOL = GetVolumeInformation(
                    &drive[0] as LPCSTR,
                    &mut VolumeName[0] as LPSTR,
                    mem::size_of_val(&VolumeName) as DWORD,
                    &mut VolumeSerialNumber,
                    &mut MaximumComponentLength,
                    &mut FileSystemFlags,
                    &mut FileSystemName[0] as LPSTR,
                    mem::size_of_val(&FileSystemName) as DWORD,
                );

                if Result != 0 && strnicmp(&VolumeName[0] as LPCSTR, CD_VOLUME.as_ptr() as LPCSTR, 8) == 0
                {
                    sprintf(
                        &mut test[0] as LPSTR,
                        b"%s%s\\%s\0".as_ptr() as LPCSTR,
                        &drive[0] as LPCSTR,
                        CD_BASEDIR.as_ptr() as LPCSTR,
                        CD_EXE.as_ptr() as LPCSTR,
                    );
                    f = fopen(&test[0] as LPCSTR, b"r\0".as_ptr() as LPCSTR);
                    if !f.is_null() {
                        fclose(f);
                        return Result;
                    }
                }
            }
            drive[0] = (drive[0] as u8 + 1) as c_char;
        }
    }

    return 0; // qfalse
}

extern "C" fn strnicmp(s1: LPCSTR, s2: LPCSTR, n: c_int) -> c_int {
    let mut i: c_int = 0;
    while i < n {
        unsafe {
            let c1 = (*s1.offset(i as isize) as u8).to_ascii_lowercase();
            let c2 = (*s2.offset(i as isize) as u8).to_ascii_lowercase();
            if c1 != c2 {
                return (c1 as i32) - (c2 as i32);
            }
        }
        i += 1;
    }
    return 0;
}

/*
================
Sys_CheckCD

Return true if the proper CD is in the drive
================
*/
fn Sys_CheckCD() -> c_int {
    #[cfg(feature = "FINAL_BUILD")]
    {
        return Sys_ScanForCD();
    }
    #[cfg(not(feature = "FINAL_BUILD"))]
    {
        return 1; // qtrue
    }
}

/*
================
Sys_GetClipboardData

================
*/
fn Sys_GetClipboardData() -> *mut c_char {
    let mut data: *mut c_char = null_mut();
    let mut cliptext: *mut c_char = null_mut();

    if unsafe { OpenClipboard(null_mut()) } != 0 {
        let mut hClipboardData: HANDLE = null_mut();

        if unsafe {
            hClipboardData = GetClipboardData(CF_TEXT);
            !hClipboardData.is_null()
        } {
            if unsafe {
                cliptext = GlobalLock(hClipboardData) as *mut c_char;
                !cliptext.is_null()
            } {
                unsafe {
                    data = Z_Malloc(
                        GlobalSize(hClipboardData) + 1,
                        TAG_CLIPBOARD,
                        0,
                    ) as *mut c_char;
                    strcpy(data, cliptext);
                    GlobalUnlock(hClipboardData);

                    strtok(data, b"\n\r\x08\0".as_ptr() as LPCSTR);
                }
            }
        }
        unsafe {
            CloseClipboard();
        }
    }
    return data;
}

/*
========================================================================

GAME DLL

========================================================================
*/

/*
=================
Sys_UnloadGame
=================
*/
fn Sys_UnloadGame() {
    unsafe {
        if game_library.is_null() {
            return;
        }
        if FreeLibrary(game_library) == 0 {
            Com_Error(
                ERR_FATAL,
                b"FreeLibrary failed for game library\0".as_ptr() as LPCSTR,
            );
        }
        game_library = null_mut();
    }
}

/*
=================
Sys_GetGameAPI

Loads the game dll
=================
*/
fn Sys_GetGameAPI(parms: *mut c_void) -> *mut c_void {
    let mut GetGameAPI: Option<extern "C" fn(*mut c_void) -> *mut c_void> = None;
    let mut name: [c_char; 256] = [0; 256];
    let mut cwd: [c_char; 256] = [0; 256];

    #[cfg(target_arch = "x86")]
    {
        let gamename: &str = "jagamex86.dll";

        #[cfg(not(debug_assertions))]
        {
            let debugdir: &str = "release";
        }

        #[cfg(all(debug_assertions, not(feature = "MEM_DEBUG")))]
        {
            let debugdir: &str = "debug";
        }

        #[cfg(feature = "MEM_DEBUG")]
        {
            let debugdir: &str = "shdebug";
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        let gamename: &str = "jagamex86.dll"; // Same for 64-bit in this port
        let debugdir: &str = "release";
    }

    unsafe {
        if !game_library.is_null() {
            Com_Error(
                ERR_FATAL,
                b"Sys_GetGameAPI without Sys_UnloadingGame\0".as_ptr() as LPCSTR,
            );
        }

        // check the current debug directory first for development purposes
        _getcwd(&mut cwd[0] as LPSTR, mem::size_of_val(&cwd) as c_int);
        sprintf(
            &mut name[0] as LPSTR,
            b"%s/%s/%s\0".as_ptr() as LPCSTR,
            &cwd[0] as LPCSTR,
            #[cfg(target_arch = "x86")]
            {
                #[cfg(not(debug_assertions))]
                {
                    b"release\0".as_ptr() as LPCSTR
                }
                #[cfg(all(debug_assertions, not(feature = "MEM_DEBUG")))]
                {
                    b"debug\0".as_ptr() as LPCSTR
                }
                #[cfg(feature = "MEM_DEBUG")]
                {
                    b"shdebug\0".as_ptr() as LPCSTR
                }
            }
            #[cfg(target_arch = "x86_64")]
            {
                b"release\0".as_ptr() as LPCSTR
            },
            b"jagamex86.dll\0".as_ptr() as LPCSTR,
        );

        game_library = LoadLibrary(&name[0] as LPCSTR);
        if !game_library.is_null() {
            Com_DPrintf(
                b"LoadLibrary (%s)\n\0".as_ptr() as LPCSTR,
                &name[0] as LPCSTR,
            );
        } else {
            // check the current directory for other development purposes
            sprintf(
                &mut name[0] as LPSTR,
                b"%s/%s\0".as_ptr() as LPCSTR,
                &cwd[0] as LPCSTR,
                b"jagamex86.dll\0".as_ptr() as LPCSTR,
            );
            game_library = LoadLibrary(&name[0] as LPCSTR);
            if !game_library.is_null() {
                Com_DPrintf(
                    b"LoadLibrary (%s)\n\0".as_ptr() as LPCSTR,
                    &name[0] as LPCSTR,
                );
            } else {
                let mut buf: LPSTR = null_mut();

                FormatMessage(
                    FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
                    null(),
                    GetLastError(),
                    MAKELANGID!(0, 0),
                    &mut buf,
                    0,
                    null(),
                );

                Com_Printf(
                    b"LoadLibrary(\"%s\") failed\n\0".as_ptr() as LPCSTR,
                    &name[0] as LPCSTR,
                );
                Com_Printf(
                    b"...reason: '%s'\n\0".as_ptr() as LPCSTR,
                    buf,
                );
                Com_Error(
                    ERR_FATAL,
                    b"Couldn't load game\0".as_ptr() as LPCSTR,
                );
            }
        }

        GetGameAPI =
            std::mem::transmute(GetProcAddress(game_library, b"GetGameAPI\0".as_ptr() as LPCSTR));
        if GetGameAPI.is_none() {
            Sys_UnloadGame();
            return null_mut();
        }
        return GetGameAPI.unwrap()(parms);
    }
}

/*
=================
Sys_LoadCgame

Used to hook up a development dll
=================
*/
fn Sys_LoadCgame(
    entryPoint: &mut *mut c_void,
    systemcalls: extern "C" fn(c_int) -> c_int,
) -> *mut c_void {
    let mut dllEntry: Option<extern "C" fn(extern "C" fn(c_int) -> c_int)> = None;

    unsafe {
        dllEntry = std::mem::transmute(GetProcAddress(
            game_library,
            b"dllEntry\0".as_ptr() as LPCSTR,
        ));
        *entryPoint = GetProcAddress(game_library, b"vmMain\0".as_ptr() as LPCSTR);
        if entryPoint.is_null() || dllEntry.is_none() {
            FreeLibrary(game_library);
            return null_mut();
        }

        dllEntry.unwrap()(systemcalls);
        return game_library;
    }
}

/*
========================================================================

BACKGROUND FILE STREAMING

========================================================================
*/

#[cfg(feature = "STREAMING_DISABLED")]
{
    fn Sys_InitStreamThread() {}

    fn Sys_ShutdownStreamThread() {}

    fn Sys_BeginStreamedFile(f: u32, readAhead: c_int) {}

    fn Sys_EndStreamedFile(f: u32) {}

    fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: u32) -> c_int {
        return unsafe { FS_Read(buffer, size * count, f) };
    }

    fn Sys_StreamSeek(f: u32, offset: c_int, origin: c_int) {
        unsafe {
            FS_Seek(f, offset, origin);
        }
    }
}

#[cfg(not(feature = "STREAMING_DISABLED"))]
{
    fn Sys_StreamThread() {
        let mut buffer: c_int = 0;
        let mut count: c_int = 0;
        let mut readCount: c_int = 0;
        let mut bufferPoint: c_int = 0;
        let mut r: c_int = 0;

        unsafe {
            loop {
                Sleep(10);
                EnterCriticalSection(&mut stream.crit[0] as *mut c_void);

                // if there is any space left in the buffer, fill it up
                while stream.eof == 0 {
                    count = stream.bufferSize - (stream.threadPosition - stream.streamPosition);
                    if count == 0 {
                        break;
                    }

                    bufferPoint = stream.threadPosition % stream.bufferSize;
                    buffer = stream.bufferSize - bufferPoint;
                    readCount = if buffer < count { buffer } else { count };

                    r = FS_Read(
                        stream.buffer.offset(bufferPoint as isize) as *mut c_void,
                        readCount,
                        stream.file,
                    );
                    stream.threadPosition += r;

                    if r != readCount {
                        stream.eof = 1; // qtrue
                        break;
                    }
                }

                LeaveCriticalSection(&mut stream.crit[0] as *mut c_void);
            }
        }
    }

    /*
    ===============
    Sys_InitStreamThread

    ================
    */
    fn Sys_InitStreamThread() {
        unsafe {
            InitializeCriticalSection(&mut stream.crit[0] as *mut c_void);

            // don't leave the critical section until there is a
            // valid file to stream, which will cause the StreamThread
            // to sleep without any overhead
            EnterCriticalSection(&mut stream.crit[0] as *mut c_void);

            stream.threadHandle = CreateThread(
                null(), // LPSECURITY_ATTRIBUTES lpsa,
                0,      // DWORD cbStack,
                Sys_StreamThread, // LPTHREAD_START_ROUTINE lpStartAddr,
                null_mut(), // LPVOID lpvThreadParm,
                0,          //   DWORD fdwCreate,
                &mut stream.threadId as *mut c_int as *mut DWORD,
            );
        }
    }

    /*
    ===============
    Sys_ShutdownStreamThread

    ================
    */
    fn Sys_ShutdownStreamThread() {}

    /*
    ===============
    Sys_BeginStreamedFile

    ================
    */
    fn Sys_BeginStreamedFile(f: u32, readAhead: c_int) {
        unsafe {
            if stream.file != 0 {
                Sys_EndStreamedFile(stream.file);
            }

            stream.file = f;
            stream.buffer = Z_Malloc(readAhead as usize, 0, 0) as *mut u8;
            stream.bufferSize = readAhead;
            stream.streamPosition = 0;
            stream.threadPosition = 0;
            stream.eof = 0; // qfalse

            // let the thread start running
            LeaveCriticalSection(&mut stream.crit[0] as *mut c_void);
        }
    }

    /*
    ===============
    Sys_EndStreamedFile

    ================
    */
    fn Sys_EndStreamedFile(f: u32) {
        unsafe {
            if f != stream.file {
                Com_Error(
                    ERR_FATAL,
                    b"Sys_EndStreamedFile: wrong file\0".as_ptr() as LPCSTR,
                );
            }
            // don't leave critical section until another stream is started
            EnterCriticalSection(&mut stream.crit[0] as *mut c_void);

            stream.file = 0;
            Z_Free(stream.buffer as *mut c_void);
        }
    }

    /*
    ===============
    Sys_StreamedRead

    ================
    */
    fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: u32) -> c_int {
        let mut available: c_int = 0;
        let mut remaining: c_int = 0;
        let mut sleepCount: c_int = 0;
        let mut copy: c_int = 0;
        let mut bufferCount: c_int = 0;
        let mut bufferPoint: c_int = 0;
        let mut dest: *mut u8 = buffer as *mut u8;

        remaining = size * count;

        if remaining <= 0 {
            unsafe {
                Com_Error(
                    ERR_FATAL,
                    b"Streamed read with non-positive size\0".as_ptr() as LPCSTR,
                );
            }
        }

        sleepCount = 0;
        unsafe {
            while remaining > 0 {
                available = stream.threadPosition - stream.streamPosition;
                if available == 0 {
                    if stream.eof != 0 {
                        break;
                    }
                    if sleepCount == 1 {
                        Com_DPrintf(b"Sys_StreamedRead: waiting\n\0".as_ptr() as LPCSTR);
                    }
                    sleepCount += 1;
                    if sleepCount > 100 {
                        Com_Error(
                            ERR_FATAL,
                            b"Sys_StreamedRead: thread has died\0".as_ptr() as LPCSTR,
                        );
                    }
                    Sleep(10);
                    continue;
                }

                bufferPoint = stream.streamPosition % stream.bufferSize;
                bufferCount = stream.bufferSize - bufferPoint;

                copy = if available < bufferCount {
                    available
                } else {
                    bufferCount
                };
                if copy > remaining {
                    copy = remaining;
                }
                memcpy(
                    dest as *mut c_void,
                    stream.buffer.offset(bufferPoint as isize) as *const c_void,
                    copy as usize,
                );
                stream.streamPosition += copy;
                dest = dest.offset(copy as isize);
                remaining -= copy;
            }
        }

        return (count * size - remaining) / size;
    }

    /*
    ===============
    Sys_StreamSeek

    ================
    */
    fn Sys_StreamSeek(f: u32, offset: c_int, origin: c_int) {
        unsafe {
            // halt the thread
            EnterCriticalSection(&mut stream.crit[0] as *mut c_void);

            // clear to that point
            FS_Seek(f, offset, origin);
            stream.streamPosition = 0;
            stream.threadPosition = 0;
            stream.eof = 0; // qfalse

            // let the thread start running at the new position
            LeaveCriticalSection(&mut stream.crit[0] as *mut c_void);
        }
    }
}

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
fn Sys_QueEvent(
    time: c_int,
    typ: c_int,
    value: c_int,
    value2: c_int,
    ptrLength: c_int,
    ptr: *mut c_void,
) {
    let mut ev: &mut sysEvent_t;
    let mut time_mut = time;

    unsafe {
        ev = &mut eventQue[(eventHead & MASK_QUED_EVENTS) as usize];
        if eventHead - eventTail >= MAX_QUED_EVENTS {
            Com_Printf(b"Sys_QueEvent: overflow\n\0".as_ptr() as LPCSTR);
            // we are discarding an event, but don't leak memory
            if !ev.evPtr.is_null() {
                Z_Free(ev.evPtr);
            }
            eventTail += 1;
        }

        eventHead += 1;

        if time_mut == 0 {
            time_mut = Sys_Milliseconds();
        }

        ev.evTime = time_mut;
        ev.evType = typ;
        ev.evValue = value;
        ev.evValue2 = value2;
        ev.evPtrLength = ptrLength;
        ev.evPtr = ptr;
    }
}

/*
================
Sys_GetEvent

================
*/
fn Sys_GetEvent() -> sysEvent_t {
    let mut msg: MSG = unsafe { mem::zeroed() };
    let mut ev: sysEvent_t = unsafe { mem::zeroed() };
    let mut s: LPSTR = null_mut();
    let mut netmsg: msg_t = unsafe { mem::zeroed() };

    unsafe {
        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) & MASK_QUED_EVENTS) as usize];
        }

        // pump the message loop
        while PeekMessage(&mut msg, null_mut(), 0, 0, PM_NOREMOVE) != 0 {
            if GetMessage(&mut msg, null_mut(), 0, 0) == 0 {
                Com_Quit_f();
            }

            // save the msg time, because wndprocs don't have access to the timestamp
            g_wv.sysMsgTime = msg.time;

            TranslateMessage(&msg);
            DispatchMessage(&msg);
        }

        // check for console commands
        s = Sys_ConsoleInput();
        if !s.is_null() {
            let mut b: LPSTR;
            let mut len: c_int;

            len = strlen(s) as c_int + 1;
            b = Z_Malloc(len as usize, TAG_EVENT, 0) as *mut c_char;
            strcpy(b, s);
            Sys_QueEvent(0, SE_CONSOLE, 0, 0, len, b as *mut c_void);
        }

        // check for network packets
        MSG_Init(
            &mut netmsg,
            &mut sys_packetReceived[0],
            MAX_MSGLEN as c_int,
        );

        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) & MASK_QUED_EVENTS) as usize];
        }

        // create an empty event to return

        memset(&mut ev as *mut sysEvent_t as *mut c_void, 0, mem::size_of::<sysEvent_t>());
        ev.evTime = timeGetTime() as c_int;

        return ev;
    }
}

//================================================================

/*
=================
Sys_In_Restart_f

Restart the input subsystem
=================
*/
extern "C" fn Sys_In_Restart_f() {
    unsafe {
        IN_Shutdown();
        IN_Init();
    }
}

fn Sys_IsExpired() -> bool {
    // #if 0 block - always returns false
    //								sec min Hr Day Mon Yr
    //    struct tm t_valid_start	= { 0, 0, 8, 23, 6, 103 };	//zero based months!
    //								sec min Hr Day Mon Yr
    //    struct tm t_valid_end	= { 0, 0, 20, 30, 6, 103 };
    //    struct tm t_valid_end	= t_valid_start;
    //	t_valid_end.tm_mday += 8;
    //	time_t startTime  = mktime( &t_valid_start);
    //	time_t expireTime = mktime( &t_valid_end);
    //	time_t now;
    //	time(&now);
    //	if((now < startTime) || (now> expireTime))
    //	{
    //		return true;
    //	}
    return false;
}

/*
================
Sys_Init

Called after the common systems (cvars, files, etc)
are initialized
================
*/
const OSR2_BUILD_NUMBER: DWORD = 1111;
const WIN98_BUILD_NUMBER: DWORD = 1998;

#[cfg(feature = "MEM_DEBUG")]
extern "C" {
    fn SH_Register();
}

fn Sys_Init() {
    let mut cpuid: c_int = 0;

    unsafe {
        // make sure the timer is high precision, otherwise
        // NT gets 18ms resolution
        timeBeginPeriod(1);

        Cmd_AddCommand(b"in_restart\0".as_ptr() as LPCSTR, Sys_In_Restart_f);
        #[cfg(feature = "MEM_DEBUG")]
        {
            SH_Register();
        }

        g_wv.osversion.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFO>() as DWORD;

        if GetVersionEx(&mut g_wv.osversion) == 0 {
            Com_Error(
                ERR_FATAL,
                b"Couldn't get OS info\0".as_ptr() as LPCSTR,
            );
        }
        if Sys_IsExpired() {
            g_wv.osversion.dwPlatformId = VER_PLATFORM_WIN32s; //sneaky: hide the expire with this error
        }

        if g_wv.osversion.dwMajorVersion < 4 {
            Com_Error(
                ERR_FATAL,
                b"This game requires Windows version 4 or greater\0".as_ptr() as LPCSTR,
            );
        }
        if g_wv.osversion.dwPlatformId == VER_PLATFORM_WIN32s {
            Com_Error(
                ERR_FATAL,
                b"This game doesn't run on Win32s\0".as_ptr() as LPCSTR,
            );
        }

        if g_wv.osversion.dwPlatformId == VER_PLATFORM_WIN32_NT {
            Cvar_Set(b"arch\0".as_ptr() as LPCSTR, b"winnt\0".as_ptr() as LPCSTR);
        } else if g_wv.osversion.dwPlatformId == VER_PLATFORM_WIN32_WINDOWS {
            if LOWORD(g_wv.osversion.dwBuildNumber) >= WIN98_BUILD_NUMBER {
                Cvar_Set(b"arch\0".as_ptr() as LPCSTR, b"win98\0".as_ptr() as LPCSTR);
            } else if LOWORD(g_wv.osversion.dwBuildNumber) >= OSR2_BUILD_NUMBER {
                Cvar_Set(
                    b"arch\0".as_ptr() as LPCSTR,
                    b"win95 osr2.x\0".as_ptr() as LPCSTR,
                );
            } else {
                Cvar_Set(b"arch\0".as_ptr() as LPCSTR, b"win95\0".as_ptr() as LPCSTR);
            }
        } else {
            Cvar_Set(
                b"arch\0".as_ptr() as LPCSTR,
                b"unknown Windows variant\0".as_ptr() as LPCSTR,
            );
        }

        // save out a couple things in rom cvars for the renderer to access
        Cvar_Get(
            b"win_hinstance\0".as_ptr() as LPCSTR,
            va(
                b"%i\0".as_ptr() as LPCSTR,
                g_wv.hInstance as usize as c_int,
            ),
            CVAR_ROM,
        );
        Cvar_Get(
            b"win_wndproc\0".as_ptr() as LPCSTR,
            va(b"%i\0".as_ptr() as LPCSTR, MainWndProc as usize as c_int),
            CVAR_ROM,
        );

        //
        // figure out our CPU
        //
        Cvar_Get(
            b"sys_cpustring\0".as_ptr() as LPCSTR,
            b"detect\0".as_ptr() as LPCSTR,
            CVAR_ROM,
        );
        if Q_stricmp(
            Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
            b"detect\0".as_ptr() as LPCSTR,
        ) == 0
        {
            Com_Printf(b"...detecting CPU, found \0".as_ptr() as LPCSTR);

            cpuid = Sys_GetProcessorId();

            match cpuid {
                CPUID_GENERIC => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as LPCSTR, b"generic\0".as_ptr() as LPCSTR);
                }
                CPUID_INTEL_UNSUPPORTED => {
                    Cvar_Set(
                        b"sys_cpustring\0".as_ptr() as LPCSTR,
                        b"x86 (pre-Pentium)\0".as_ptr() as LPCSTR,
                    );
                }
                CPUID_INTEL_PENTIUM => {
                    Cvar_Set(
                        b"sys_cpustring\0".as_ptr() as LPCSTR,
                        b"x86 (P5/PPro, non-MMX)\0".as_ptr() as LPCSTR,
                    );
                }
                CPUID_INTEL_MMX => {
                    Cvar_Set(
                        b"sys_cpustring\0".as_ptr() as LPCSTR,
                        b"x86 (P5/Pentium2, MMX)\0".as_ptr() as LPCSTR,
                    );
                }
                CPUID_INTEL_KATMAI => {
                    Cvar_Set(
                        b"sys_cpustring\0".as_ptr() as LPCSTR,
                        b"Intel Pentium III\0".as_ptr() as LPCSTR,
                    );
                }
                CPUID_INTEL_WILLIAMETTE => {
                    Cvar_Set(
                        b"sys_cpustring\0".as_ptr() as LPCSTR,
                        b"Intel Pentium IV\0".as_ptr() as LPCSTR,
                    );
                }
                CPUID_AMD_3DNOW => {
                    Cvar_Set(
                        b"sys_cpustring\0".as_ptr() as LPCSTR,
                        b"AMD w/ 3DNow!\0".as_ptr() as LPCSTR,
                    );
                }
                CPUID_AXP => {
                    Cvar_Set(
                        b"sys_cpustring\0".as_ptr() as LPCSTR,
                        b"Alpha AXP\0".as_ptr() as LPCSTR,
                    );
                }
                _ => {
                    Com_Error(
                        ERR_FATAL,
                        b"Unknown cpu type %d\n\0".as_ptr() as LPCSTR,
                        cpuid,
                    );
                }
            }
        } else {
            Com_Printf(b"...forcing CPU type to \0".as_ptr() as LPCSTR);
            if Q_stricmp(
                Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
                b"generic\0".as_ptr() as LPCSTR,
            ) == 0
            {
                cpuid = CPUID_GENERIC;
            } else if Q_stricmp(
                Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
                b"x87\0".as_ptr() as LPCSTR,
            ) == 0
            {
                cpuid = CPUID_INTEL_PENTIUM;
            } else if Q_stricmp(
                Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
                b"mmx\0".as_ptr() as LPCSTR,
            ) == 0
            {
                cpuid = CPUID_INTEL_MMX;
            } else if Q_stricmp(
                Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
                b"3dnow\0".as_ptr() as LPCSTR,
            ) == 0
            {
                cpuid = CPUID_AMD_3DNOW;
            } else if Q_stricmp(
                Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
                b"PentiumIII\0".as_ptr() as LPCSTR,
            ) == 0
            {
                cpuid = CPUID_INTEL_KATMAI;
            } else if Q_stricmp(
                Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
                b"PentiumIV\0".as_ptr() as LPCSTR,
            ) == 0
            {
                cpuid = CPUID_INTEL_WILLIAMETTE;
            } else if Q_stricmp(
                Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
                b"axp\0".as_ptr() as LPCSTR,
            ) == 0
            {
                cpuid = CPUID_AXP;
            } else {
                Com_Printf(
                    b"WARNING: unknown sys_cpustring '%s'\n\0".as_ptr() as LPCSTR,
                    Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
                );
                cpuid = CPUID_GENERIC;
            }
        }
        Cvar_SetValue(b"sys_cpuid\0".as_ptr() as LPCSTR, cpuid as f32);
        Com_Printf(
            b"%s\n\0".as_ptr() as LPCSTR,
            Cvar_VariableString(b"sys_cpustring\0".as_ptr() as LPCSTR),
        );

        Cvar_Set(b"username\0".as_ptr() as LPCSTR, Sys_GetCurrentUser());

        IN_Init(); // FIXME: not in dedicated?
    }
}

// do a quick mem test to check for any potential future mem problems...
//
fn QuickMemTest() {
    //	if (!Sys_LowPhysicalMemory())
    {
        let iMemTestMegs: c_int = 128; // useful search label
        // special test,
        let pvData: *mut c_void = unsafe { malloc((iMemTestMegs * 1024 * 1024) as usize) };
        if !pvData.is_null() {
            unsafe {
                free(pvData);
            }
        } else {
            // err...
            //
            unsafe {
                let Language_IsAsian_result = Language_IsAsian();
                let psContinue: LPCSTR = if Language_IsAsian_result != 0 {
                    b"Your machine failed to allocate %dMB in a memory test, which may mean you'll have problems running this game all the way through.\n\nContinue anyway?\0".as_ptr() as LPCSTR
                } else {
                    SE_GetString(b"CON_TEXT_FAILED_MEMTEST\0".as_ptr() as LPCSTR)
                };
                // ( since it's too much hassle doing MBCS code pages and decodings etc for MessageBox command )

                if MessageBox(
                    null_mut(),
                    va(psContinue, iMemTestMegs),
                    b"Query\0".as_ptr() as LPCSTR,
                    MB_YESNO | MB_ICONWARNING | MB_TASKMODAL,
                ) != IDYES
                {
                    let psNoMem: LPCSTR = if Language_IsAsian() != 0 {
                        b"Insufficient memory to run this game!\n\0".as_ptr() as LPCSTR
                    } else {
                        SE_GetString(b"CON_TEXT_INSUFFICIENT_MEMORY\0".as_ptr() as LPCSTR)
                    };
                    // ( since it's too much hassle doing MBCS code pages and decodings etc for MessageBox command )

                    Com_Error(ERR_FATAL, psNoMem);
                }
            }
        }
    }
}

//=======================================================================
//int	totalMsec, countMsec;

/*
==================
WinMain

==================
*/
#[no_mangle]
pub extern "C" fn WinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPSTR,
    nCmdShow: c_int,
) -> c_int {
    let mut cwd: [c_char; 256] = [0; 256];
    //	int			startTime, endTime;

    SET_CRT_DEBUG_FIELD!(_CRTDBG_LEAK_CHECK_DF);
    //   _CrtSetBreakAlloc(34804);

    unsafe {
        // should never get a previous instance in Win32
        if !hPrevInstance.is_null() {
            return 0;
        }

        g_wv.hInstance = hInstance;
        Q_strncpyz(&mut sys_cmdline[0] as LPSTR, lpCmdLine, mem::size_of_val(&sys_cmdline));

        // done before Com/Sys_Init since we need this for error output
        Sys_CreateConsole();

        // no abort/retry/fail errors
        SetErrorMode(SEM_FAILCRITICALERRORS);

        // get the initial time base
        Sys_Milliseconds();

        //#if 0
        // if we find the CD, add a +set cddir xxx command line
        // Sys_ScanForCD();
        //#endif

        Sys_InitStreamThread();

        Com_Init(&sys_cmdline[0] as LPCSTR);

        QuickMemTest();

        _getcwd(&mut cwd[0] as LPSTR, mem::size_of_val(&cwd) as c_int);
        Com_Printf(
            b"Working directory: %s\n\0".as_ptr() as LPCSTR,
            &cwd[0] as LPCSTR,
        );

        // hide the early console since we've reached the point where we
        // have a working graphics subsystems
        let com_viewlog_ptr = &com_viewlog as *const *mut c_void;
        if !com_viewlog_ptr.is_null() {
            let com_view_struct = *com_viewlog_ptr as *mut c_int;
            if !com_view_struct.is_null() && *com_view_struct == 0 {
                Sys_ShowConsole(0, 0); // qfalse
            }
        }

        // main game loop
        loop {
            // if not running as a game client, sleep a bit
            if g_wv.isMinimized != 0 {
                Sleep(5);
            }
            #[cfg(debug_assertions)]
            {
                if g_wv.activeApp == 0 {
                    Sleep(50);
                }
            }

            // set low precision every frame, because some system calls
            // reset it arbitrarily
            //		_controlfp( _PC_24, _MCW_PC );

            //		startTime = Sys_Milliseconds();

            // make sure mouse and joystick are only called once a frame
            IN_Frame();

            // run the game
            Com_Frame();

            //		endTime = Sys_Milliseconds();
            //		totalMsec += endTime - startTime;
            //		countMsec++;
        }

        // never gets here
    }
}
