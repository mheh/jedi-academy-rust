// win_main.c
// Anything above this #include will be ignored by the compiler

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use std::ptr;
use std::mem;

// Extern declarations for engine and game functions
extern "C" {
    fn Cvar_Get(varName: *const c_char, varValue: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cvar_SetValue(var_name: *const c_char, value: c_int);
    fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
    fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;

    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_Quit_f();
    fn Com_Init(commandLine: *const c_char);
    fn Com_Frame();
    fn Com_BlockChecksum(buffer: *const c_void, length: c_int) -> c_int;
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn Com_FilterPath(filter: *const c_char, name: *const c_char, folders: c_int) -> c_int;
    fn Com_ShutdownZoneMemory();
    fn Com_ShutdownHunkMemory();

    fn Cmd_AddCommand(cmd_name: *const c_char, function: extern "C" fn());

    fn Conbuf_AppendText(msg: *const c_char);
    fn Sys_SetErrorText(text: *const c_char);
    fn Sys_ShowConsole(show: c_int, qactive: c_int);
    fn Sys_CreateConsole();
    fn Sys_DestroyConsole();
    fn Sys_ConsoleInput() -> *mut c_char;
    fn Sys_Milliseconds() -> c_int;
    fn Sys_GetPacket(from: *mut netadr_t, msg: *mut msg_t) -> c_int;
    fn Sys_GetProcessorId() -> c_int;
    fn Sys_GetCurrentUser() -> *const c_char;
    fn Sys_GetCPUSpeed() -> c_int;
    fn Sys_GetPhysicalMemory() -> c_int;

    fn IN_Init();
    fn IN_Shutdown();
    fn IN_Frame();

    fn NET_Init();
    fn NET_Restart();

    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn FS_FileIsInPAK(qpath: *const c_char, checksum: *mut c_int) -> c_int;
    fn FS_FOpenFileWrite(filename: *const c_char) -> c_int;
    fn FS_Write(buffer: *const c_void, len: c_int, f: c_int) -> c_int;
    fn FS_FCloseFile(f: c_int);
    fn FS_BuildOSPath(base: *const c_char, game: *const c_char, qpath: *const c_char) -> *mut c_char;
    fn FS_FileExists(qpath: *const c_char) -> c_int;
    fn FS_Read(buffer: *mut c_void, len: c_int, f: c_int) -> c_int;
    fn FS_Seek(f: c_int, offset: c_int, origin: c_int);

    fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);

    fn Z_Malloc(size: usize, tag: c_int) -> *mut c_void;
    fn Z_Malloc_qtrue(size: usize, tag: c_int, qtrue_flag: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    fn CopyString(in_str: *const c_char) -> *mut c_char;

    fn Q_stricmp(s0: *const c_char, s1: *const c_char) -> c_int;
    fn Q_strncmp(s0: *const c_char, s1: *const c_char, n: usize) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);

    fn MainWndProc(hwnd: *mut c_void, uMsg: u32, wParam: usize, lParam: isize) -> isize;

    fn va(format: *const c_char, ...) -> *const c_char;
    fn Language_IsAsian() -> c_int;
    fn SE_GetString(token: *const c_char) -> *const c_char;
}

// Windows API declarations
extern "C" {
    fn GlobalMemoryStatus(lpBuffer: *mut MEMORYSTATUS);
    fn GetModuleFileName(hModule: *mut c_void, lpFilename: *mut c_char, nSize: u32) -> u32;
    fn GetTempPath(nBufferLength: u32, lpBuffer: *mut c_char) -> u32;
    fn GetTempFileName(lpPathName: *const c_char, lpPrefixString: *const c_char, uUnique: u32, lpTempFileName: *mut c_char) -> u32;
    fn CopyFile(lpExistingFileName: *const c_char, lpNewFileName: *const c_char, bFailIfExists: i32) -> i32;
    fn CreateFile(lpFileName: *const c_char, dwDesiredAccess: u32, dwShareMode: u32, lpSecurityAttributes: *mut c_void, dwCreationDisposition: u32, dwFlagsAndAttributes: u32, hTemplateFile: *mut c_void) -> *mut c_void;
    fn GetDriveType(lpRootPathName: *const c_char) -> u32;
    fn GetVolumeInformation(lpRootPathName: *const c_char, lpVolumeNameBuffer: *mut c_char, nVolumeNameSize: u32, lpVolumeSerialNumber: *mut u32, lpMaximumComponentLength: *mut u32, lpFileSystemFlags: *mut u32, lpFileSystemNameBuffer: *mut c_char, nFileSystemNameSize: u32) -> i32;
    fn OpenClipboard(hWndNewOwner: *mut c_void) -> i32;
    fn GetClipboardData(uFormat: u32) -> *mut c_void;
    fn GlobalLock(hMem: *mut c_void) -> *mut c_void;
    fn GlobalSize(hMem: *mut c_void) -> usize;
    fn GlobalUnlock(hMem: *mut c_void) -> i32;
    fn CloseClipboard() -> i32;
    fn FreeLibrary(hLibModule: *mut c_void) -> i32;
    fn LoadLibrary(lpLibFileName: *const c_char) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const c_char) -> *mut c_void;
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut c_void;
    fn GetCurrentProcessId() -> u32;
    fn WaitForSingleObject(hHandle: *mut c_void, dwMilliseconds: u32) -> u32;
    fn CloseHandle(hObject: *mut c_void) -> i32;
    fn CreateProcess(lpApplicationName: *const c_char, lpCommandLine: *mut c_char, lpProcessAttributes: *mut c_void, lpThreadAttributes: *mut c_void, bInheritHandles: i32, dwCreationFlags: u32, lpEnvironment: *mut c_void, lpCurrentDirectory: *const c_char, lpStartupInfo: *mut STARTUPINFO, lpProcessInformation: *mut PROCESS_INFORMATION) -> i32;
    fn ZeroMemory(dest: *mut c_void, length: usize);
    fn wsprintf(Output: *mut c_char, Format: *const c_char, ...) -> i32;
    fn GetMessage(lpMsg: *mut MSG, hWnd: *mut c_void, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn PeekMessage(lpMsg: *mut MSG, hWnd: *mut c_void, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessage(lpMsg: *const MSG) -> isize;
    fn timeBeginPeriod(uPeriod: u32) -> u32;
    fn timeEndPeriod(uPeriod: u32) -> u32;
    fn timeGetTime() -> u32;
    fn GetVersionEx(lpVersionInformation: *mut OSVERSIONINFO) -> i32;
    fn SetErrorMode(uMode: u32) -> u32;
    fn IsBadReadPtr(lp: *const c_void, ucb: usize) -> i32;
    fn _mkdir(dirname: *const c_char) -> c_int;
    fn _getcwd(buffer: *mut c_char, maxlen: c_int) -> *mut c_char;
    fn _findfirst(filespec: *const c_char, fileinfo: *mut finddata_t) -> isize;
    fn _findnext(handle: isize, fileinfo: *mut finddata_t) -> c_int;
    fn _findclose(handle: isize) -> c_int;
    fn _A_SUBDIR() -> u16;
    fn strlen(s: *const c_char) -> usize;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE;
    fn fseek(stream: *mut FILE, offset: c_int, whence: c_int) -> c_int;
    fn ftell(stream: *mut FILE) -> c_int;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut FILE) -> usize;
    fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut FILE) -> usize;
    fn fclose(stream: *mut FILE) -> c_int;
    fn exit(status: c_int) -> !;
    fn atoi(nptr: *const c_char) -> c_int;
    fn strtok(s: *mut c_char, delim: *const c_char) -> *mut c_char;
    fn strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn MessageBoxEx(hWnd: *mut c_void, lpText: *const c_char, lpCaption: *const c_char, uType: u32, wLanguageId: u16) -> i32;
    fn MessageBox(hWnd: *mut c_void, lpText: *const c_char, lpCaption: *const c_char, uType: u32) -> i32;
}

// Constants
const CD_BASEDIR: &[u8] = b"gamedata\\gamedata";
const CD_EXE: &[u8] = b"jamp.exe";
const CD_BASEDIR_LINUX: &[u8] = b"bin\\x86\\glibc-2.1";
const CD_EXE_LINUX: &[u8] = b"jamp";
const CD_VOLUME: &[u8] = b"JEDIACAD";
const MEM_THRESHOLD: u32 = 128 * 1024 * 1024;
const MAX_FOUND_FILES: usize = 0x1000;
const MAX_QUED_EVENTS: usize = 256;
const MASK_QUED_EVENTS: usize = MAX_QUED_EVENTS - 1;
const MAX_OSPATH: usize = 256;
const MAX_STRING_CHARS: usize = 4096;
const MAX_QPATH: usize = 64;
const MAX_MSGLEN: usize = 16384;
const MAX_FILE_HANDLES: usize = 64;
const MAX_PATH: usize = 260;
const _MAX_PATH: usize = 260;

const OSR2_BUILD_NUMBER: u32 = 1111;
const WIN98_BUILD_NUMBER: u32 = 1998;

const CPUID_GENERIC: c_int = 0;
const CPUID_INTEL_UNSUPPORTED: c_int = 1;
const CPUID_INTEL_PENTIUM: c_int = 2;
const CPUID_INTEL_MMX: c_int = 3;
const CPUID_INTEL_KATMAI: c_int = 4;
const CPUID_INTEL_WILLIAMETTE: c_int = 5;
const CPUID_AMD_3DNOW: c_int = 6;
const CPUID_AXP: c_int = 7;

const VER_PLATFORM_WIN32s: u32 = 0;
const VER_PLATFORM_WIN32_WINDOWS: u32 = 1;
const VER_PLATFORM_WIN32_NT: u32 = 2;

const TAG_FILESYS: c_int = 1;
const TAG_CLIPBOARD: c_int = 2;
const TAG_EVENT: c_int = 3;
const CVAR_ROM: c_int = 4;

const SE_CONSOLE: c_int = 1;
const SE_PACKET: c_int = 2;

const ERR_FATAL: c_int = 1;

const FILE_SHARE_READ: u32 = 1;
const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x04000000;
const OPEN_EXISTING: u32 = 3;
const SYNCHRONIZE: u32 = 0x00100000;

const CF_TEXT: u32 = 1;

const DRIVE_CDROM: u32 = 5;

const PM_NOREMOVE: u32 = 0;

const SEM_FAILCRITICALERRORS: u32 = 0x0001;

const MB_OKCANCEL: u32 = 1;
const MB_ICONEXCLAMATION: u32 = 0x00000030;
const MB_DEFBUTTON2: u32 = 0x00000100;
const MB_TOPMOST: u32 = 0x00040000;
const MB_SETFOREGROUND: u32 = 0x00010000;
const MB_YESNO: u32 = 4;
const MB_ICONWARNING: u32 = 0x00000030;
const MB_TASKMODAL: u32 = 0x00002000;

const IDOK: i32 = 1;
const IDYES: i32 = 6;

const SEEK_END: c_int = 2;
const SEEK_SET: c_int = 0;

const INFINITE: u32 = 0xFFFFFFFF;

const LANG_NEUTRAL: u32 = 0;
const SUBLANG_DEFAULT: u32 = 0;

const IMAGE_NT_SIGNATURE: u32 = 0x4550;

// Macro for MAKELANGID
#[inline]
const fn MAKELANGID(pri: u32, sub: u32) -> u16 {
    ((sub << 10) | pri) as u16
}

// Type definitions
#[repr(C)]
pub struct MEMORYSTATUS {
    dwLength: u32,
    dwMemoryLoad: u32,
    dwTotalPhys: u32,
    dwAvailPhys: u32,
    dwTotalPageFile: u32,
    dwAvailPageFile: u32,
    dwTotalVirtual: u32,
    dwAvailVirtual: u32,
}

#[repr(C)]
pub struct cvar_t {
    hash: c_int,
    name: *const c_char,
    string: *const c_char,
    resetString: *const c_char,
    integer: c_int,
    value: f32,
    flags: c_int,
    modified: c_int,
    modificationCount: c_int,
    next: *mut cvar_t,
    prev: *mut cvar_t,
}

#[repr(C)]
pub struct msg_t {
    data: *mut u8,
    cursize: c_int,
    maxsize: c_int,
    readcount: c_int,
    bit: c_int,
}

#[repr(C)]
pub struct netadr_t {
    type_: c_int,
    ip: [u8; 4],
    port: u16,
}

#[repr(C)]
pub struct sysEvent_t {
    evTime: u32,
    evType: c_int,
    evValue: c_int,
    evValue2: c_int,
    evPtrLength: c_int,
    evPtr: *mut c_void,
}

#[repr(C)]
pub struct STARTUPINFO {
    cb: u32,
    lpReserved: *mut c_char,
    lpDesktop: *mut c_char,
    lpTitle: *mut c_char,
    dwX: u32,
    dwY: u32,
    dwXSize: u32,
    dwYSize: u32,
    dwXCountChars: u32,
    dwYCountChars: u32,
    dwFillAttribute: u32,
    dwFlags: u32,
    wShowWindow: u16,
    cbReserved2: u16,
    lpReserved2: *mut u8,
    hStdInput: *mut c_void,
    hStdOutput: *mut c_void,
    hStdError: *mut c_void,
}

#[repr(C)]
pub struct PROCESS_INFORMATION {
    hProcess: *mut c_void,
    hThread: *mut c_void,
    dwProcessId: u32,
    dwThreadId: u32,
}

#[repr(C)]
pub struct MSG_T {
    wParam: usize,
    lParam: isize,
    time: u32,
}

#[repr(C)]
pub struct OSVERSIONINFO {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,
    szCSDVersion: [c_char; 128],
}

#[repr(C)]
pub struct IMAGE_DOS_HEADER {
    e_magic: u16,
    e_cblp: u16,
    e_cp: u16,
    e_crlf: u16,
    e_cparhdr: u16,
    e_minalloc: u16,
    e_maxalloc: u16,
    e_ss: u16,
    e_sp: u16,
    e_csum: u16,
    e_ip: u16,
    e_cs: u16,
    e_lfarlc: u16,
    e_ovno: u16,
    e_res: [u16; 4],
    e_oemid: u16,
    e_oeminfo: u16,
    e_res2: [u16; 10],
    e_lfanew: i32,
}

#[repr(C)]
pub struct IMAGE_FILE_HEADER {
    Machine: u16,
    NumberOfSections: u16,
    TimeDateStamp: u32,
    PointerToSymbolTable: u32,
    NumberOfSymbols: u32,
    SizeOfOptionalHeader: u16,
    Characteristics: u16,
}

#[repr(C)]
pub struct IMAGE_OPTIONAL_HEADER {
    Magic: u16,
    // Many more fields...
    MajorLinkerVersion: u8,
    MinorLinkerVersion: u8,
    SizeOfCode: u32,
    SizeOfInitializedData: u32,
    SizeOfUninitializedData: u32,
    AddressOfEntryPoint: u32,
    BaseOfCode: u32,
}

#[repr(C)]
pub struct IMAGE_NT_HEADERS {
    Signature: u32,
    FileHeader: IMAGE_FILE_HEADER,
    OptionalHeader: IMAGE_OPTIONAL_HEADER,
}

#[repr(C)]
pub struct IMAGE_SECTION_HEADER {
    Name: [c_char; 8],
    VirtualSize: u32,
    VirtualAddress: u32,
    SizeOfRawData: u32,
    PointerToRawData: u32,
}

#[repr(C)]
pub struct finddata_t {
    attrib: u16,
    time_create: u64,
    time_access: u64,
    time_write: u64,
    size: u32,
    name: [c_char; 260],
}

#[repr(C)]
pub struct streamsIO_t {
    file: c_int,
    buffer: *mut u8,
    eof: c_int,
    active: c_int,
    bufferSize: c_int,
    streamPosition: c_int,
    threadPosition: c_int,
}

#[repr(C)]
pub struct streamState_t {
    threadHandle: *mut c_void,
    threadId: u32,
    crit: CRITICAL_SECTION,
    sIO: [streamsIO_t; MAX_FILE_HANDLES],
}

pub enum FILE {}

#[repr(C)]
pub struct CRITICAL_SECTION {
    DebugInfo: *mut c_void,
    LockCount: i32,
    RecursionCount: i32,
    OwningThread: *mut c_void,
    LockSemaphore: *mut c_void,
    SpinCount: usize,
}

#[repr(C)]
pub struct g_wv_t {
    hInstance: *mut c_void,
    sysMsgTime: u32,
    osversion: OSVERSIONINFO,
    isMinimized: c_int,
    activeApp: c_int,
}

extern "C" {
    static mut g_wv: g_wv_t;
    static mut com_dedicated: *mut cvar_t;
    static mut com_viewlog: *mut cvar_t;
}

// Stubs for unported constants that need to reference constants
#[inline]
fn _A_SUBDIR_VALUE() -> u16 {
    0x10
}

// Macro definitions
const fn MakePtr(ptr: usize, addValue: usize) -> usize {
    ptr + addValue
}

// Static globals
static mut sys_cmdline: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

// enable this for executable checksumming
// #ifdef FINAL_BUILD
// #define SPANK_MONKEYS
// #endif
static mut sys_monkeySpank: c_int = 0;
static mut sys_checksum: c_int = 0;

/*
==================
Sys_LowPhysicalMemory()
==================
*/
pub extern "C" fn Sys_LowPhysicalMemory() -> c_int {
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
    static mut bAsked: c_int = 0;
    static mut sys_lowmem: *mut cvar_t = ptr::null_mut();

    unsafe {
        if sys_lowmem.is_null() {
            sys_lowmem = Cvar_Get(b"sys_lowmem\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        if bAsked == 0 {
            // just in case it takes a little time for GlobalMemoryStatus() to gather stats on
            // stuff we don't care about such as virtual mem etc.
            bAsked = 1;
            GlobalMemoryStatus(&mut stat);
        }
        if (*sys_lowmem).integer != 0 {
            return 1;
        }
        if stat.dwTotalPhys <= MEM_THRESHOLD {
            return 1;
        } else {
            return 0;
        }
    }
}

/*
==================
Sys_FunctionCmp
==================
*/
pub extern "C" fn Sys_FunctionCmp(f1: *mut c_void, f2: *mut c_void) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut l: c_int;
    let mut func_end: [u8; 32] = [0xC3, 0x90, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut ptr: *mut u8;
    let mut ptr2: *mut u8;
    let mut f1_ptr: *mut u8;
    let mut f2_ptr: *mut u8;

    unsafe {
        ptr = f1 as *mut u8;
        if *(ptr as *const u8) == 0xE9 {
            //Com_Printf("f1 %p1 jmp %d\n", (int *) f1, *(int*)(ptr+1));
            f1_ptr = (f1 as *mut u8).add((*(ptr.add(1) as *const i32)) as usize).add(5);
        } else {
            f1_ptr = ptr;
        }
        //Com_Printf("f1 ptr %p\n", f1_ptr);

        ptr = f2 as *mut u8;
        if *(ptr as *const u8) == 0xE9 {
            //Com_Printf("f2 %p jmp %d\n", (int *) f2, *(int*)(ptr+1));
            f2_ptr = (f2 as *mut u8).add((*(ptr.add(1) as *const i32)) as usize).add(5);
        } else {
            f2_ptr = ptr;
        }
        //Com_Printf("f2 ptr %p\n", f2_ptr);

        #[cfg(debug_assertions)]
        {
            // sprintf((char *)func_end, "%c%c%c%c%c%c%c", 0x5F, 0x5E, 0x5B, 0x8B, 0xE5, 0x5D, 0xC3);
            func_end[0] = 0x5F;
            func_end[1] = 0x5E;
            func_end[2] = 0x5B;
            func_end[3] = 0x8B;
            func_end[4] = 0xE5;
            func_end[5] = 0x5D;
            func_end[6] = 0xC3;
        }

        i = 0;
        while i < 1024 {
            j = 0;
            while func_end[j as usize] != 0 {
                if *f1_ptr.add((i + j) as usize) != func_end[j as usize] {
                    break;
                }
                j += 1;
            }
            if func_end[j as usize] == 0 {
                break;
            }
            i += 1;
        }

        #[cfg(debug_assertions)]
        {
            l = i + 7;
        }
        #[cfg(not(debug_assertions))]
        {
            l = i + 2;
        }
        //Com_Printf("function length = %d\n", l);

        i = 0;
        while i < l {
            // check for a potential function call
            if *f1_ptr.add(i as usize) == 0xE8 {
                // get the function pointers in case this really is a function call
                ptr = f1_ptr.add(i as usize).add((*(f1_ptr.add((i + 1) as usize) as *const i32)) as usize).add(5);
                ptr2 = f2_ptr.add(i as usize).add((*(f2_ptr.add((i + 1) as usize) as *const i32)) as usize).add(5);
                // if it was a function call and both f1 and f2 call the same function
                if ptr == ptr2 {
                    i += 4;
                    i += 1;
                    continue;
                }
            }
            if *f1_ptr.add(i as usize) != *f2_ptr.add(i as usize) {
                return 0;
            }
            i += 1;
        }
        return 1;
    }
}

/*
==================
Sys_FunctionCheckSum
==================
*/
pub extern "C" fn Sys_FunctionCheckSum(f1: *mut c_void) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut l: c_int;
    let mut func_end: [u8; 32] = [0xC3, 0x90, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut ptr: *mut u8;
    let mut f1_ptr: *mut u8;

    unsafe {
        ptr = f1 as *mut u8;
        if *(ptr as *const u8) == 0xE9 {
            //Com_Printf("f1 %p1 jmp %d\n", (int *) f1, *(int*)(ptr+1));
            f1_ptr = (f1 as *mut u8).add((*(ptr.add(1) as *const i32)) as usize).add(5);
        } else {
            f1_ptr = ptr;
        }
        //Com_Printf("f1 ptr %p\n", f1_ptr);

        #[cfg(debug_assertions)]
        {
            // sprintf((char *)func_end, "%c%c%c%c%c%c%c", 0x5F, 0x5E, 0x5B, 0x8B, 0xE5, 0x5D, 0xC3);
            func_end[0] = 0x5F;
            func_end[1] = 0x5E;
            func_end[2] = 0x5B;
            func_end[3] = 0x8B;
            func_end[4] = 0xE5;
            func_end[5] = 0x5D;
            func_end[6] = 0xC3;
        }

        i = 0;
        while i < 1024 {
            j = 0;
            while func_end[j as usize] != 0 {
                if *f1_ptr.add((i + j) as usize) != func_end[j as usize] {
                    break;
                }
                j += 1;
            }
            if func_end[j as usize] == 0 {
                break;
            }
            i += 1;
        }

        #[cfg(debug_assertions)]
        {
            l = i + 7;
        }
        #[cfg(not(debug_assertions))]
        {
            l = i + 2;
        }
        //Com_Printf("function length = %d\n", l);
        return Com_BlockChecksum(f1_ptr as *const c_void, l);
    }
}

/*
==================
Sys_MonkeyShouldBeSpanked
==================
*/
pub extern "C" fn Sys_MonkeyShouldBeSpanked() -> c_int {
    unsafe {
        return sys_monkeySpank;
    }
}

/*
==================
Sys_CodeInMemoryChecksum
==================
*/
fn MakePtr_cast(cast_type: usize, ptr: usize, addValue: usize) -> usize {
    ptr + addValue
}

pub extern "C" fn Sys_CodeInMemoryChecksum(codeBase: *mut c_void) -> c_int {
    let mut dosHeader: *mut IMAGE_DOS_HEADER;
    let mut pNTHeader: *mut IMAGE_NT_HEADERS;
    let mut section: *mut IMAGE_SECTION_HEADER;

    unsafe {
        dosHeader = codeBase as *mut IMAGE_DOS_HEADER;
        pNTHeader = (codeBase as usize + (*dosHeader).e_lfanew as usize) as *mut IMAGE_NT_HEADERS;

        // First, verify that the e_lfanew field gave us a reasonable
        // pointer, then verify the PE signature.
        if IsBadReadPtr(pNTHeader as *const c_void, mem::size_of::<IMAGE_NT_HEADERS>()) != 0 ||
           (*pNTHeader).Signature != IMAGE_NT_SIGNATURE
        {
            //printf("Unhandled EXE type, or invalid .EXE\n");
            return 0;
        }
        // first section oughta be the code section
        section = (pNTHeader as usize + mem::size_of::<IMAGE_NT_HEADERS>()) as *mut IMAGE_SECTION_HEADER;
        /*
        // the name of the code section should be .text
        if ( Q_stricmp( section->Name, ".text" ) ) {
            return 0;
        }
        */

        return Com_BlockChecksum(
            (codeBase as usize + (*section).VirtualAddress as usize) as *const c_void,
            (*section).SizeOfRawData as c_int
        );
    }
}

/*
==================
Sys_ChecksumExe
==================
*/

// make sure this string is unique in the executable
//					   01234567890123
static mut exeChecksumId: *mut u8 = b"q3monkeyid\0\0\0\0".as_ptr() as *mut u8;

pub extern "C" fn Sys_ChecksumExe(codeBase: *mut c_void) {
    let mut szPathOrig: [c_char; _MAX_PATH] = [0; _MAX_PATH];
    let mut szPathClone: [c_char; _MAX_PATH] = [0; _MAX_PATH];
    let mut si: STARTUPINFO;
    let mut szCmdLine: [c_char; 512] = [0; 512];
    let mut hfile: *mut c_void;
    let mut hProcessOrig: *mut c_void;
    let mut pi: PROCESS_INFORMATION;
    let mut l: c_int;
    let mut i: c_int;
    let mut n: c_int;
    let mut f: *mut FILE;
    let mut buf: *mut u8;
    let mut ptr: *mut u8;

    unsafe {
        extern "C" {
            static __argv: *mut *mut c_char;
        }

        // Is this the original EXE or the clone EXE?
        if Q_stricmp(*__argv.add(1), b"monkey\0".as_ptr() as *const c_char) != 0 {
            // Original EXE: Spawn clone EXE to delete this EXE

            GetModuleFileName(ptr::null_mut(), szPathOrig.as_mut_ptr(), _MAX_PATH as u32);
            GetTempPath(_MAX_PATH as u32, szPathClone.as_mut_ptr());
            GetTempFileName(szPathClone.as_ptr(), b"Del\0".as_ptr() as *const c_char, 0, szPathClone.as_mut_ptr());
            CopyFile(szPathOrig.as_ptr(), szPathClone.as_ptr(), 0);

            // Open the clone EXE using FILE_FLAG_DELETE_ON_CLOSE
            hfile = CreateFile(
                szPathClone.as_ptr(),
                0,
                FILE_SHARE_READ,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_FLAG_DELETE_ON_CLOSE,
                ptr::null_mut()
            );
            // Spawn the clone EXE passing it our EXE's process handle
            // and the full path name to the original EXE file.
            hProcessOrig = OpenProcess(SYNCHRONIZE, 1, GetCurrentProcessId());
            wsprintf(
                szCmdLine.as_mut_ptr(),
                b"%s monkey %d %d \"%s\"\0".as_ptr() as *const c_char,
                szPathClone.as_ptr(),
                sys_checksum,
                hProcessOrig as i32,
                szPathOrig.as_ptr()
            );
            ZeroMemory(&mut si as *mut _ as *mut c_void, mem::size_of::<STARTUPINFO>());
            si.cb = mem::size_of::<STARTUPINFO>() as u32;
            CreateProcess(ptr::null(), szCmdLine.as_mut_ptr(), ptr::null_mut(), ptr::null_mut(), 1, 0, ptr::null_mut(), ptr::null(), &mut si, &mut pi);
            CloseHandle(hProcessOrig);
            CloseHandle(hfile);
        } else {
            // Clone EXE: When original EXE terminates, overwrite it with a new one
            sys_checksum = atoi(*__argv.add(2));
            hProcessOrig = atoi(*__argv.add(3)) as *mut c_void;
            WaitForSingleObject(hProcessOrig, INFINITE);
            CloseHandle(hProcessOrig);
            // open the original executable
            f = fopen(*__argv.add(4), b"rb\0".as_ptr() as *const c_char);
            if f.is_null() {
                return;
            }
            fseek(f, 0, SEEK_END);
            l = ftell(f);
            fseek(f, 0, SEEK_SET);
            buf = malloc(l as usize) as *mut u8;
            if fread(buf as *mut c_void, l as usize, 1, f) != 1 {
                return;
            }
            fclose(f);
            // search for the exe name string, nice brute force
            n = strlen(exeChecksumId as *const c_char) as c_int;
            i = 0;
            while i < l {
                if Q_strncmp(buf.add(i as usize) as *const c_char, exeChecksumId as *const c_char, n as usize) == 0 {
                    break;
                }
                i += 1;
            }
            if i >= l {
                return;
            }
            ptr = buf.add(i as usize);
            // write checksum into exe memory image
            *ptr.add(0) = ((sys_checksum >> 24) & 0xFF) as u8;
            *ptr.add(1) = ((sys_checksum >> 16) & 0xFF) as u8;
            *ptr.add(2) = ((sys_checksum >> 8) & 0xFF) as u8;
            *ptr.add(3) = ((sys_checksum >> 0) & 0xFF) as u8;
            *ptr.add(4) = 0;
            *ptr.add(5) = 0;
            *ptr.add(6) = 0;
            *ptr.add(7) = 0;
            *ptr.add(8) = 0;
            *ptr.add(9) = 0;
            // write out new exe with checksum
            f = fopen(*__argv.add(4), b"wb\0".as_ptr() as *const c_char);
            if f.is_null() {
                return;
            }
            if fwrite(buf as *const c_void, l as usize, 1, f) != 1 {
                return;
            }
            fclose(f);
            free(buf as *mut c_void);
            // The system will delete the clone EXE automatically
            // because it was opened with FILE_FLAG_DELETE_ON_CLOSE
        }
        //
        exit(0);
    }
}

/*
==================
Sys_VerifyCodeChecksum
==================
*/
pub extern "C" fn Sys_VerifyCodeChecksum(codeBase: *mut c_void) {
    // NOTE: should not checksum code in debug mode because the memory image changes
    //		 as soon as you set a break point!
    #[cfg(all(feature = "spank_monkeys", not(debug_assertions)))]
    {
        let mut exeChecksum: c_int;

        unsafe {
            // if the checksum is not yet stored in the executable
            if *exeChecksumId.add(4) != 0 {
                // spawn another process that will replace this executable with one that has a checksum
                Sys_ChecksumExe(codeBase);
                return;
            }

            exeChecksum = ((*exeChecksumId.add(0) as c_int) << 24) |
                          ((*exeChecksumId.add(1) as c_int) << 16) |
                          ((*exeChecksumId.add(2) as c_int) << 8) |
                          (*exeChecksumId.add(3) as c_int);
            if exeChecksum != sys_checksum {
                sys_monkeySpank = 1;
            }
        }
    }
}

/*
==================
Sys_BeginProfiling
==================
*/
pub extern "C" fn Sys_BeginProfiling() {
    // this is just used on the mac build
}

/*
=============
Sys_Error

Show the early console as an error dialog
=============
*/
pub extern "C" fn Sys_Error(error: *const c_char, _args: ...) {
    let mut text: [c_char; 4096] = [0; 4096];
    let mut msg: MSG_T;

    unsafe {
        // We can't really use va_list in Rust easily without variadic support,
        // so this is a simplified stub. In production, would need to handle varargs properly
        // For now, just call Com_Error which should handle the formatting
        Com_Error(ERR_FATAL, error);

        Conbuf_AppendText(error);
        Conbuf_AppendText(b"\n\0".as_ptr() as *const c_char);

        Sys_SetErrorText(error);
        Sys_ShowConsole(1, 1);

        timeEndPeriod(1);

        IN_Shutdown();

        // wait for the user to quit
        loop {
            if GetMessage(&mut msg as *mut _ as *mut MSG, ptr::null_mut(), 0, 0) == 0 {
                Com_Quit_f();
            }
            TranslateMessage(&msg as *const _ as *const MSG);
            DispatchMessage(&msg as *const _ as *const MSG);
        }

        // Sys_DestroyConsole();
        // Com_ShutdownZoneMemory();
        // Com_ShutdownHunkMemory();

        // exit(1);
    }
}

/*
==============
Sys_Quit
==============
*/
pub extern "C" fn Sys_Quit() {
    unsafe {
        timeEndPeriod(1);
        IN_Shutdown();
        Sys_DestroyConsole();
        Com_ShutdownZoneMemory();
        Com_ShutdownHunkMemory();

        exit(0);
    }
}

/*
==============
Sys_Print
==============
*/
pub extern "C" fn Sys_Print(msg: *const c_char) {
    unsafe {
        Conbuf_AppendText(msg);
    }
}


/*
==============
Sys_Mkdir
==============
*/
pub extern "C" fn Sys_Mkdir(path: *const c_char) {
    unsafe {
        _mkdir(path);
    }
}

/*
==============
Sys_Cwd
==============
*/
pub extern "C" fn Sys_Cwd() -> *mut c_char {
    static mut cwd: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

    unsafe {
        _getcwd(cwd.as_mut_ptr(), (MAX_OSPATH - 1) as c_int);
        cwd[MAX_OSPATH - 1] = 0;

        return cwd.as_mut_ptr();
    }
}

/*
==============
Sys_DefaultCDPath
==============
*/
pub extern "C" fn Sys_DefaultCDPath() -> *mut c_char {
    return b"\0".as_ptr() as *mut c_char;
}

/*
==============
Sys_DefaultBasePath
==============
*/
pub extern "C" fn Sys_DefaultBasePath() -> *mut c_char {
    return Sys_Cwd();
}

/*
==============================================================

DIRECTORY SCANNING

==============================================================
*/

pub extern "C" fn Sys_ListFilteredFiles(basedir: *const c_char, subdirs: *mut c_char, filter: *mut c_char, psList: *mut *mut c_char, numfiles: *mut c_int) {
    let mut search: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut newsubdirs: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut filename: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut findhandle: isize;
    let mut findinfo: finddata_t;

    unsafe {
        if *numfiles >= MAX_FOUND_FILES as c_int - 1 {
            return;
        }

        if strlen(subdirs) > 0 {
            Com_sprintf(search.as_mut_ptr(), MAX_OSPATH, b"%s\\%s\\*\0".as_ptr() as *const c_char, basedir, subdirs);
        } else {
            Com_sprintf(search.as_mut_ptr(), MAX_OSPATH, b"%s\\*\0".as_ptr() as *const c_char, basedir);
        }

        findhandle = _findfirst(search.as_ptr(), &mut findinfo);
        if findhandle == -1 {
            return;
        }

        loop {
            if (findinfo.attrib & _A_SUBDIR_VALUE()) != 0 {
                if Q_stricmp(findinfo.name.as_ptr(), b".\0".as_ptr() as *const c_char) != 0 &&
                   Q_stricmp(findinfo.name.as_ptr(), b"..\0".as_ptr() as *const c_char) != 0 {
                    if strlen(subdirs) > 0 {
                        Com_sprintf(newsubdirs.as_mut_ptr(), MAX_OSPATH, b"%s\\%s\0".as_ptr() as *const c_char, subdirs, findinfo.name.as_ptr());
                    } else {
                        Com_sprintf(newsubdirs.as_mut_ptr(), MAX_OSPATH, b"%s\0".as_ptr() as *const c_char, findinfo.name.as_ptr());
                    }
                    Sys_ListFilteredFiles(basedir, newsubdirs.as_mut_ptr(), filter, psList, numfiles);
                }
            }
            if *numfiles >= MAX_FOUND_FILES as c_int - 1 {
                break;
            }
            Com_sprintf(filename.as_mut_ptr(), MAX_OSPATH, b"%s\\%s\0".as_ptr() as *const c_char, subdirs, findinfo.name.as_ptr());
            if Com_FilterPath(filter, filename.as_ptr(), 0) == 0 {
                if _findnext(findhandle, &mut findinfo) == -1 {
                    break;
                }
                continue;
            }
            *psList.add(*numfiles as usize) = CopyString(filename.as_ptr());
            *numfiles += 1;

            if _findnext(findhandle, &mut findinfo) == -1 {
                break;
            }
        }

        _findclose(findhandle);
    }
}

static fn strgtr(s0: *const c_char, s1: *const c_char) -> c_int {
    let mut l0: c_int;
    let mut l1: c_int;
    let mut i: c_int;

    unsafe {
        l0 = strlen(s0) as c_int;
        l1 = strlen(s1) as c_int;

        if l1 < l0 {
            l0 = l1;
        }

        i = 0;
        while i < l0 {
            if *s1.add(i as usize) as u8 > *s0.add(i as usize) as u8 {
                return 1;
            }
            if (*s1.add(i as usize) as u8) < (*s0.add(i as usize) as u8) {
                return 0;
            }
            i += 1;
        }
        return 0;
    }
}

pub extern "C" fn Sys_ListFiles(directory: *const c_char, extension: *const c_char, filter: *mut c_char, numfiles: *mut c_int, wantsubs: c_int) -> *mut *mut c_char {
    let mut search: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut nfiles: c_int;
    let mut listCopy: *mut *mut c_char;
    let mut list: [*mut c_char; MAX_FOUND_FILES] = [ptr::null_mut(); MAX_FOUND_FILES];
    let mut findinfo: finddata_t;
    let mut findhandle: isize;
    let mut flag: c_int;
    let mut i: c_int;

    unsafe {
        if !filter.is_null() {

            nfiles = 0;
            Sys_ListFilteredFiles(directory, b"\0".as_ptr() as *mut c_char, filter, list.as_mut_ptr(), &mut nfiles);

            list[nfiles as usize] = ptr::null_mut();
            *numfiles = nfiles;

            if nfiles == 0 {
                return ptr::null_mut();
            }

            listCopy = Z_Malloc(((nfiles + 1) * mem::size_of::<*mut c_char>() as c_int) as usize, TAG_FILESYS) as *mut *mut c_char;
            i = 0;
            while i < nfiles {
                *listCopy.add(i as usize) = list[i as usize];
                i += 1;
            }
            *listCopy.add(i as usize) = ptr::null_mut();

            return listCopy;
        }

        let mut ext: *const c_char = extension;
        if ext.is_null() {
            ext = b"\0".as_ptr() as *const c_char;
        }

        // passing a slash as extension will find directories
        if *ext.add(0) as u8 == b'/' && *ext.add(1) as u8 == 0 {
            ext = b"\0".as_ptr() as *const c_char;
            flag = 0;
        } else {
            flag = _A_SUBDIR_VALUE() as c_int;
        }

        Com_sprintf(search.as_mut_ptr(), MAX_OSPATH, b"%s\\*%s\0".as_ptr() as *const c_char, directory, ext);

        // search
        nfiles = 0;

        findhandle = _findfirst(search.as_ptr(), &mut findinfo);
        if findhandle == -1 {
            *numfiles = 0;
            return ptr::null_mut();
        }

        loop {
            if (wantsubs == 0 && (flag ^ ((findinfo.attrib & _A_SUBDIR_VALUE()) as c_int)) != 0) ||
               (wantsubs != 0 && (findinfo.attrib & _A_SUBDIR_VALUE()) != 0) {
                if nfiles == MAX_FOUND_FILES as c_int - 1 {
                    break;
                }
                list[nfiles as usize] = CopyString(findinfo.name.as_ptr());
                nfiles += 1;
            }
            if _findnext(findhandle, &mut findinfo) == -1 {
                break;
            }
        }

        list[nfiles as usize] = ptr::null_mut();

        _findclose(findhandle);

        // return a copy of the list
        *numfiles = nfiles;

        if nfiles == 0 {
            return ptr::null_mut();
        }

        listCopy = Z_Malloc(((nfiles + 1) * mem::size_of::<*mut c_char>() as c_int) as usize, TAG_FILESYS) as *mut *mut c_char;
        i = 0;
        while i < nfiles {
            *listCopy.add(i as usize) = list[i as usize];
            i += 1;
        }
        *listCopy.add(i as usize) = ptr::null_mut();

        loop {
            flag = 0;
            i = 1;
            while i < nfiles {
                if strgtr(*listCopy.add((i - 1) as usize), *listCopy.add(i as usize)) != 0 {
                    let mut temp = *listCopy.add(i as usize);
                    *listCopy.add(i as usize) = *listCopy.add((i - 1) as usize);
                    *listCopy.add((i - 1) as usize) = temp;
                    flag = 1;
                }
                i += 1;
            }
            if flag == 0 {
                break;
            }
        }

        return listCopy;
    }
}

pub extern "C" fn Sys_FreeFileList(psList: *mut *mut c_char) {
    let mut i: c_int;

    unsafe {
        if psList.is_null() {
            return;
        }

        i = 0;
        while !(*psList.add(i as usize)).is_null() {
            Z_Free(*psList.add(i as usize) as *mut c_void);
            i += 1;
        }

        Z_Free(psList as *mut c_void);
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
#[cfg(feature = "final_build")]
fn Sys_ScanForCD() -> c_int {
    let mut drive: [c_char; 4] = [0; 4];
    let mut f: *mut FILE;
    let mut test: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

    unsafe {
        drive[0] = b'c' as c_char;
        drive[1] = b':' as c_char;
        drive[2] = b'\\' as c_char;
        drive[3] = 0;

        // scan the drives
        let mut c: u8 = b'c';
        while c <= b'z' {
            drive[0] = c as c_char;
            if GetDriveType(drive.as_ptr()) == DRIVE_CDROM {
                let mut Result: i32;
                let mut VolumeName: [c_char; MAX_PATH] = [0; MAX_PATH];
                let mut FileSystemName: [c_char; MAX_PATH] = [0; MAX_PATH];
                let mut VolumeSerialNumber: u32 = 0;
                let mut MaximumComponentLength: u32 = 0;
                let mut FileSystemFlags: u32 = 0;

                Result = GetVolumeInformation(
                    drive.as_ptr(),
                    VolumeName.as_mut_ptr(),
                    MAX_PATH as u32,
                    &mut VolumeSerialNumber,
                    &mut MaximumComponentLength,
                    &mut FileSystemFlags,
                    FileSystemName.as_mut_ptr(),
                    MAX_PATH as u32
                );

                if Result != 0 && strnicmp(VolumeName.as_ptr(), CD_VOLUME.as_ptr() as *const c_char, 8) == 0 {
                    sprintf(test.as_mut_ptr(), b"%s%s\\%s\0".as_ptr() as *const c_char, drive.as_ptr(), CD_BASEDIR.as_ptr() as *const c_char, CD_EXE.as_ptr() as *const c_char);
                    f = fopen(test.as_ptr(), b"r\0".as_ptr() as *const c_char);
                    if !f.is_null() {
                        fclose(f);
                        return 1;
                    } else {
                        sprintf(test.as_mut_ptr(), b"%s%s\\%s\0".as_ptr() as *const c_char, drive.as_ptr(), CD_BASEDIR_LINUX.as_ptr() as *const c_char, CD_EXE_LINUX.as_ptr() as *const c_char);
                        f = fopen(test.as_ptr(), b"r\0".as_ptr() as *const c_char);
                        if !f.is_null() {
                            fclose(f);
                            return 1;
                        }
                    }
                }
            }
            c += 1;
        }

        return 0;
    }
}

/*
================
Sys_CheckCD

Return true if the proper CD is in the drive
================
*/
pub extern "C" fn Sys_CheckCD() -> c_int {
    #[cfg(feature = "final_build")]
    {
        return Sys_ScanForCD();
    }
    #[cfg(not(feature = "final_build"))]
    {
        return 1;
    }
}


/*
================
Sys_GetClipboardData

================
*/
pub extern "C" fn Sys_GetClipboardData() -> *mut c_char {
    let mut data: *mut c_char = ptr::null_mut();
    let mut cliptext: *mut c_char;

    unsafe {
        if OpenClipboard(ptr::null_mut()) != 0 {
            let mut hClipboardData: *mut c_void;

            hClipboardData = GetClipboardData(CF_TEXT);
            if !hClipboardData.is_null() {
                cliptext = GlobalLock(hClipboardData) as *mut c_char;
                if !cliptext.is_null() {
                    data = Z_Malloc(GlobalSize(hClipboardData) + 1, TAG_CLIPBOARD) as *mut c_char;
                    Q_strncpyz(data, cliptext, GlobalSize(hClipboardData) + 1);
                    GlobalUnlock(hClipboardData);

                    strtok(data, b"\n\r\x08\0".as_ptr() as *const c_char);
                }
            }
            CloseClipboard();
        }
        return data;
    }
}


/*
========================================================================

LOAD/UNLOAD DLL

========================================================================
*/

/*
=================
Sys_UnloadDll

=================
*/
pub extern "C" fn Sys_UnloadDll(dllHandle: *mut c_void) {
    unsafe {
        if dllHandle.is_null() {
            return;
        }
        if FreeLibrary(dllHandle) == 0 {
            Com_Error(ERR_FATAL, b"Sys_UnloadDll FreeLibrary failed\0".as_ptr() as *const c_char);
        }
    }
}

// make sure the dll can be opened by the file system, then write the
// file back out again so it can be loaded is a library. If the read
// fails then the dll is probably not in the pk3 and we are running
// a pure server -rww
pub extern "C" fn Sys_UnpackDLL(name: *const c_char) -> c_int {
    let mut data: *mut c_void;
    let mut f: c_int;
    let mut len: c_int = FS_ReadFile(name, &mut data);
    let mut ck: c_int;

    unsafe {
        if len < 1 {
            // failed to read the file (out of the pk3 if pure)
            return 0;
        }

        if FS_FileIsInPAK(name, &mut ck) == -1 {
            // alright, it isn't in a pk3 anyway, so we don't need to write it.
            // this is allowable when running non-pure.
            FS_FreeFile(data);
            return 1;
        }

        f = FS_FOpenFileWrite(name);
        if f == 0 {
            // can't open for writing? Might be in use.
            // This is possibly a malicious user attempt to circumvent dll
            // replacement so we won't allow it.
            FS_FreeFile(data);
            return 0;
        }

        if FS_Write(data, len, f) < len {
            // Failed to write the full length. Full disk maybe?
            FS_FreeFile(data);
            return 0;
        }

        FS_FCloseFile(f);
        FS_FreeFile(data);

        return 1;
    }
}

/*
=================
Sys_LoadDll

Used to load a development dll instead of a virtual machine
=================
*/

pub extern "C" fn Sys_LoadDll(name: *const c_char, entryPoint: *mut extern "C" fn(c_int, ...) -> c_int,
                  systemcalls: extern "C" fn(c_int, ...) -> c_int) -> *mut c_void {
    static mut lastWarning: c_int = 0;
    let mut libHandle: *mut c_void;
    let mut dllEntry: extern "C" fn(extern "C" fn(c_int, ...) -> c_int);
    let mut basepath: *const c_char;
    let mut cdpath: *const c_char;
    let mut gamedir: *const c_char;
    let mut fn_: *mut c_char;
    #[cfg(not(debug_assertions))]
    let mut timestamp: c_int;
    #[cfg(not(debug_assertions))]
    let mut ret: c_int;
    let mut filename: [c_char; MAX_QPATH] = [0; MAX_QPATH];

    unsafe {
        Com_sprintf(filename.as_mut_ptr(), MAX_QPATH, b"%sx86.dll\0".as_ptr() as *const c_char, name);

        #[cfg(not(debug_assertions))]
        {
            timestamp = Sys_Milliseconds();
            //	if( ((timestamp - lastWarning) > (5 * 60000)) && !Cvar_VariableIntegerValue( "dedicated" )
            //		&& !Cvar_VariableIntegerValue( "com_blindlyLoadDLLs" ) ) {
            if 0 != 0 {
                if FS_FileExists(filename.as_ptr()) != 0 {
                    lastWarning = timestamp;
                    ret = MessageBoxEx(
                        ptr::null_mut(),
                        b"You are about to load a .DLL executable that\nhas not been verified for use with Quake III Arena.\nThis type of file can compromise the security of\nyour computer.\n\nSelect 'OK' if you choose to load it anyway.\0".as_ptr() as *const c_char,
                        b"Security Warning\0".as_ptr() as *const c_char,
                        MB_OKCANCEL | MB_ICONEXCLAMATION | MB_DEFBUTTON2 | MB_TOPMOST | MB_SETFOREGROUND,
                        MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT)
                    );
                    if ret != IDOK {
                        return ptr::null_mut();
                    }
                }
            }
        }

        if Sys_UnpackDLL(filename.as_ptr()) == 0 {
            return ptr::null_mut();
        }

        // rjr disable for final release #ifndef NDEBUG
        libHandle = LoadLibrary(filename.as_ptr());
        if libHandle.is_null() {
            //#endif
            basepath = Cvar_VariableString(b"fs_basepath\0".as_ptr() as *const c_char);
            cdpath = Cvar_VariableString(b"fs_cdpath\0".as_ptr() as *const c_char);
            gamedir = Cvar_VariableString(b"fs_game\0".as_ptr() as *const c_char);

            fn_ = FS_BuildOSPath(basepath, gamedir, filename.as_ptr());
            libHandle = LoadLibrary(fn_);

            if libHandle.is_null() {
                if *cdpath.add(0) as u8 != 0 {
                    fn_ = FS_BuildOSPath(cdpath, gamedir, filename.as_ptr());
                    libHandle = LoadLibrary(fn_);
                }

                if libHandle.is_null() {
                    return ptr::null_mut();
                }
            }
            //#ifndef NDEBUG
        }
        //#endif

        dllEntry = mem::transmute(GetProcAddress(libHandle, b"dllEntry\0".as_ptr() as *const c_char));
        *entryPoint = mem::transmute(GetProcAddress(libHandle, b"vmMain\0".as_ptr() as *const c_char));
        if entryPoint.is_null() || dllEntry as *const c_void == ptr::null() {
            FreeLibrary(libHandle);
            return ptr::null_mut();
        }
        dllEntry(systemcalls);

        return libHandle;
    }
}


/*
========================================================================

BACKGROUND FILE STREAMING

========================================================================
*/

// Simple stubs for streaming (feature is disabled by default via #if 1 in original)

pub extern "C" fn Sys_InitStreamThread() {
}

pub extern "C" fn Sys_ShutdownStreamThread() {
}

pub extern "C" fn Sys_BeginStreamedFile(f: c_int, readAhead: c_int) {
}

pub extern "C" fn Sys_EndStreamedFile(f: c_int) {
}

pub extern "C" fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: c_int) -> c_int {
    unsafe {
        return FS_Read(buffer, size * count, f);
    }
}

pub extern "C" fn Sys_StreamSeek(f: c_int, offset: c_int, origin: c_int) {
    unsafe {
        FS_Seek(f, offset, origin);
    }
}

/*
========================================================================

EVENT LOOP

========================================================================
*/

static mut eventQue: [sysEvent_t; MAX_QUED_EVENTS] = [sysEvent_t {
    evTime: 0,
    evType: 0,
    evValue: 0,
    evValue2: 0,
    evPtrLength: 0,
    evPtr: ptr::null_mut(),
}; MAX_QUED_EVENTS];
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
pub extern "C" fn Sys_QueEvent(time: c_int, type_: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void) {
    let mut ev: *mut sysEvent_t;
    let mut t = time;

    unsafe {
        ev = &mut eventQue[(eventHead & (MASK_QUED_EVENTS as c_int)) as usize];
        if eventHead - eventTail >= MAX_QUED_EVENTS as c_int {
            Com_Printf(b"Sys_QueEvent: overflow\n\0".as_ptr() as *const c_char);
            // we are discarding an event, but don't leak memory
            if !(*ev).evPtr.is_null() {
                Z_Free((*ev).evPtr);
            }
            eventTail += 1;
        }

        eventHead += 1;

        if t == 0 {
            t = Sys_Milliseconds();
        }

        (*ev).evTime = t as u32;
        (*ev).evType = type_;
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
pub extern "C" fn Sys_GetEvent() -> sysEvent_t {
    let mut msg: MSG_T;
    let mut ev: sysEvent_t;
    let mut s: *mut c_char;
    let mut netmsg: msg_t;
    let mut adr: netadr_t;

    unsafe {
        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) & (MASK_QUED_EVENTS as c_int)) as usize];
        }

        // pump the message loop
        while PeekMessage(&mut msg as *mut _ as *mut MSG, ptr::null_mut(), 0, 0, PM_NOREMOVE) != 0 {
            if GetMessage(&mut msg as *mut _ as *mut MSG, ptr::null_mut(), 0, 0) == 0 {
                Com_Quit_f();
            }

            // save the msg time, because wndprocs don't have access to the timestamp
            g_wv.sysMsgTime = msg.time;

            TranslateMessage(&msg as *const _ as *const MSG);
            DispatchMessage(&msg as *const _ as *const MSG);
        }

        // check for console commands
        s = Sys_ConsoleInput();
        if !s.is_null() {
            let mut b: *mut c_char;
            let mut len: c_int;

            len = strlen(s) as c_int + 1;
            b = Z_Malloc(len as usize, TAG_EVENT) as *mut c_char;
            Q_strncpyz(b, s, len as usize);
            Sys_QueEvent(0, SE_CONSOLE, 0, 0, len, b as *mut c_void);
        }

        // check for network packets
        MSG_Init(&mut netmsg, sys_packetReceived.as_mut_ptr(), MAX_MSGLEN as c_int);
        if Sys_GetPacket(&mut adr, &mut netmsg) != 0 {
            let mut buf: *mut netadr_t;
            let mut len: c_int;

            // copy out to a seperate buffer for qeueing
            // the readcount stepahead is for SOCKS support
            len = (mem::size_of::<netadr_t>() as c_int) + netmsg.cursize - netmsg.readcount;
            buf = Z_Malloc_qtrue(len as usize, TAG_EVENT, 1) as *mut netadr_t;
            *buf = adr;
            memcpy(
                buf.add(1) as *mut c_void,
                netmsg.data.add(netmsg.readcount as usize) as *const c_void,
                (netmsg.cursize - netmsg.readcount) as usize
            );
            Sys_QueEvent(0, SE_PACKET, 0, 0, len, buf as *mut c_void);
        }

        // return if we have data
        if eventHead > eventTail {
            eventTail += 1;
            return eventQue[((eventTail - 1) & (MASK_QUED_EVENTS as c_int)) as usize];
        }

        // create an empty event to return

        memset(&mut ev as *mut _ as *mut c_void, 0, mem::size_of::<sysEvent_t>());
        ev.evTime = timeGetTime();

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
pub extern "C" fn Sys_In_Restart_f() {
    IN_Shutdown();
    IN_Init();
}


/*
=================
Sys_Net_Restart_f

Restart the network subsystem
=================
*/
pub extern "C" fn Sys_Net_Restart_f() {
    NET_Restart();
}

fn Sys_IsExpired() -> c_int {
    #[allow(unreachable_code)]
    {
        // if 0
        return 0;

        // #if 0
        // //								sec min Hr Day Mon Yr
        //     struct tm t_valid_start	= { 0, 0, 8, 23, 6, 103 };	//zero based months!
        // //								sec min Hr Day Mon Yr
        //     struct tm t_valid_end	= { 0, 0, 20, 30, 6, 103 };
        // //    struct tm t_valid_end	= t_valid_start;
        // //	t_valid_end.tm_mday += 8;
        // 	time_t startTime  = mktime( &t_valid_start);
        // 	time_t expireTime = mktime( &t_valid_end);
        // 	time_t now;
        // 	time(&now);
        // 	if((now < startTime) || (now> expireTime))
        // 	{
        // 		return true;
        // 	}
        // #endif
    }
}

/*
================
Sys_Init

Called after the common systems (cvars, files, etc)
are initialized
================
*/

pub extern "C" fn Sys_Init() {
    let mut cpuid: c_int;

    unsafe {
        // make sure the timer is high precision, otherwise
        // NT gets 18ms resolution
        timeBeginPeriod(1);

        Cmd_AddCommand(b"in_restart\0".as_ptr() as *const c_char, Sys_In_Restart_f);
        Cmd_AddCommand(b"net_restart\0".as_ptr() as *const c_char, Sys_Net_Restart_f);

        g_wv.osversion.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFO>() as u32;

        if GetVersionEx(&mut g_wv.osversion) == 0 {
            Sys_Error(b"Couldn't get OS info\0".as_ptr() as *const c_char);
        }
        if Sys_IsExpired() != 0 {
            g_wv.osversion.dwPlatformId = VER_PLATFORM_WIN32s;	//sneaky: hide the expire with this error
        }

        if g_wv.osversion.dwMajorVersion < 4 {
            Sys_Error(b"This game requires Windows version 4 or greater\0".as_ptr() as *const c_char);
        }
        if g_wv.osversion.dwPlatformId == VER_PLATFORM_WIN32s {
            Sys_Error(b"This game doesn't run on Win32s\0".as_ptr() as *const c_char);
        }

        if g_wv.osversion.dwPlatformId == VER_PLATFORM_WIN32_NT {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"winnt\0".as_ptr() as *const c_char);
        } else if g_wv.osversion.dwPlatformId == VER_PLATFORM_WIN32_WINDOWS {
            if (g_wv.osversion.dwBuildNumber & 0xFFFF) >= WIN98_BUILD_NUMBER {
                Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"win98\0".as_ptr() as *const c_char);
            } else if (g_wv.osversion.dwBuildNumber & 0xFFFF) >= OSR2_BUILD_NUMBER {
                Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"win95 osr2.x\0".as_ptr() as *const c_char);
            } else {
                Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"win95\0".as_ptr() as *const c_char);
            }
        } else {
            Cvar_Set(b"arch\0".as_ptr() as *const c_char, b"unknown Windows variant\0".as_ptr() as *const c_char);
        }

        // save out a couple things in rom cvars for the renderer to access
        Cvar_Get(b"win_hinstance\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, g_wv.hInstance as i32), CVAR_ROM);
        Cvar_Get(b"win_wndproc\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, MainWndProc as *const c_void as i32), CVAR_ROM);

        //
        // figure out our CPU
        //
        Cvar_Get(b"sys_cpustring\0".as_ptr() as *const c_char, b"detect\0".as_ptr() as *const c_char, CVAR_ROM);
        if Q_stricmp(Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char), b"detect\0".as_ptr() as *const c_char) == 0 {
            Com_Printf(b"...detecting CPU, found \0".as_ptr() as *const c_char);

            cpuid = Sys_GetProcessorId();

            match cpuid {
                CPUID_GENERIC => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as *const c_char, b"generic\0".as_ptr() as *const c_char);
                }
                CPUID_INTEL_UNSUPPORTED => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as *const c_char, b"x86 (pre-Pentium)\0".as_ptr() as *const c_char);
                }
                CPUID_INTEL_PENTIUM => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as *const c_char, b"x86 (P5/PPro, non-MMX)\0".as_ptr() as *const c_char);
                }
                CPUID_INTEL_MMX => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as *const c_char, b"x86 (P5/Pentium2, MMX)\0".as_ptr() as *const c_char);
                }
                CPUID_INTEL_KATMAI => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as *const c_char, b"Intel Pentium III\0".as_ptr() as *const c_char);
                }
                CPUID_INTEL_WILLIAMETTE => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as *const c_char, b"Intel Pentium IV\0".as_ptr() as *const c_char);
                }
                CPUID_AMD_3DNOW => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as *const c_char, b"AMD w/ 3DNow!\0".as_ptr() as *const c_char);
                }
                CPUID_AXP => {
                    Cvar_Set(b"sys_cpustring\0".as_ptr() as *const c_char, b"Alpha AXP\0".as_ptr() as *const c_char);
                }
                _ => {
                    Com_Error(ERR_FATAL, b"Unknown cpu type %d\n\0".as_ptr() as *const c_char, cpuid);
                }
            }
        } else {
            Com_Printf(b"...forcing CPU type to \0".as_ptr() as *const c_char);
            if Q_stricmp(Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char), b"generic\0".as_ptr() as *const c_char) == 0 {
                cpuid = CPUID_GENERIC;
            } else if Q_stricmp(Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char), b"x87\0".as_ptr() as *const c_char) == 0 {
                cpuid = CPUID_INTEL_PENTIUM;
            } else if Q_stricmp(Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char), b"mmx\0".as_ptr() as *const c_char) == 0 {
                cpuid = CPUID_INTEL_MMX;
            } else if Q_stricmp(Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char), b"3dnow\0".as_ptr() as *const c_char) == 0 {
                cpuid = CPUID_AMD_3DNOW;
            } else if Q_stricmp(Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char), b"PentiumIII\0".as_ptr() as *const c_char) == 0 {
                cpuid = CPUID_INTEL_KATMAI;
            } else if Q_stricmp(Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char), b"PentiumIV\0".as_ptr() as *const c_char) == 0 {
                cpuid = CPUID_INTEL_WILLIAMETTE;
            } else if Q_stricmp(Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char), b"axp\0".as_ptr() as *const c_char) == 0 {
                cpuid = CPUID_AXP;
            } else {
                Com_Printf(b"WARNING: unknown sys_cpustring '%s'\n\0".as_ptr() as *const c_char, Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char));
                cpuid = CPUID_GENERIC;
            }
        }
        Cvar_SetValue(b"sys_cpuid\0".as_ptr() as *const c_char, cpuid);
        Com_Printf(b"%s\n\0".as_ptr() as *const c_char, Cvar_VariableString(b"sys_cpustring\0".as_ptr() as *const c_char));

        Cvar_Set(b"username\0".as_ptr() as *const c_char, Sys_GetCurrentUser());
        Cvar_SetValue(b"sys_cpuspeed\0".as_ptr() as *const c_char, Sys_GetCPUSpeed());
        Cvar_SetValue(b"sys_memory\0".as_ptr() as *const c_char, Sys_GetPhysicalMemory());

        IN_Init();		// FIXME: not in dedicated?
    }
}

// do a quick mem test to check for any potential future mem problems...
//
fn QuickMemTest() {
    //	if (!Sys_LowPhysicalMemory())
    {
        let iMemTestMegs: c_int = 128;	// useful search label
        // special test,
        unsafe {
            let pvData: *mut c_void = malloc((iMemTestMegs * 1024 * 1024) as usize);
            if !pvData.is_null() {
                free(pvData);
            } else {
                // err...
                //
                let bAsian: c_int = Language_IsAsian();
                let psContinue: *const c_char = if bAsian != 0 {
                    b"Your machine failed to allocate %dMB in a memory test, which may mean you'll have problems running this game all the way through.\n\nContinue anyway?\0".as_ptr() as *const c_char
                } else {
                    SE_GetString(b"CON_TEXT_FAILED_MEMTEST\0".as_ptr() as *const c_char)
                };
                // ( since it's too much hassle doing MBCS code pages and decodings etc for MessageBox command )

                let result = MessageBox(
                    ptr::null_mut(),
                    va(psContinue, iMemTestMegs),
                    b"Query\0".as_ptr() as *const c_char,
                    MB_YESNO | MB_ICONWARNING | MB_TASKMODAL
                );

                if result != IDYES {
                    let psNoMem: *const c_char = if bAsian != 0 {
                        b"Insufficient memory to run this game!\n\0".as_ptr() as *const c_char
                    } else {
                        SE_GetString(b"CON_TEXT_INSUFFICIENT_MEMORY\0".as_ptr() as *const c_char)
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
pub extern "C" fn WinMain(hInstance: *mut c_void, hPrevInstance: *mut c_void, lpCmdLine: *mut c_char, nCmdShow: c_int) -> c_int {
    //	int			startTime, endTime;

    unsafe {
        // should never get a previous instance in Win32
        if !hPrevInstance.is_null() {
            return 0;
        }

        sys_checksum = Sys_CodeInMemoryChecksum(hInstance);
        Sys_VerifyCodeChecksum(hInstance);

        g_wv.hInstance = hInstance;
        Q_strncpyz(sys_cmdline.as_mut_ptr(), lpCmdLine, MAX_STRING_CHARS);

        // done before Com/Sys_Init since we need this for error output
        Sys_CreateConsole();

        // no abort/retry/fail errors
        SetErrorMode(SEM_FAILCRITICALERRORS);

        // get the initial time base
        Sys_Milliseconds();

        #[allow(unreachable_code)]
        {
            // if we find the CD, add a +set cddir xxx command line
            // Sys_ScanForCD();
        }


        Sys_InitStreamThread();

        Com_Init(sys_cmdline.as_ptr());

        #[cfg(not(feature = "dedicated"))]
        {
            QuickMemTest();
        }

        NET_Init();

        // hide the early console since we've reached the point where we
        // have a working graphics subsystems
        if (*com_dedicated).is_null() || (*com_dedicated).integer == 0 {
            if (*com_viewlog).is_null() || (*com_viewlog).integer == 0 {
                Sys_ShowConsole(0, 0);
            }
        }

        #[cfg(debug_assertions)]
        {
            if sys_monkeySpank != 0 {
                Cvar_Set(b"cl_trn\0".as_ptr() as *const c_char, b"666\0".as_ptr() as *const c_char);
            }
        }

        // main game loop
        loop {
            // if not running as a game client, sleep a bit
            if g_wv.isMinimized != 0 || (!(*com_dedicated).is_null() && (*com_dedicated).integer != 0) {
                // Sleep(5);
            }

            #[cfg(debug_assertions)]
            {
                if g_wv.activeApp == 0 {
                    // Sleep(50);
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
