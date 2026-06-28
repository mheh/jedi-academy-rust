#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    improper_ctypes,
    static_mut_refs
)]

use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut, null, null_mut};

use crate::codemp::game::q_shared_h::{byte, qboolean, MAX_QPATH, MAX_STRING_CHARS, QFALSE, QTRUE};
use crate::ffi::types::fileHandle_t;

const CD_BASEDIR: &[u8] = b"gamedata\\gamedata\0";
const CD_EXE: &[u8] = b"jamp.exe\0";
const CD_BASEDIR_LINUX: &[u8] = b"bin\\x86\\glibc-2.1\0";
const CD_EXE_LINUX: &[u8] = b"jamp\0";
const CD_VOLUME: &[u8] = b"JEDIACAD\0";
const MEM_THRESHOLD: c_ulong = 128 * 1024 * 1024;

const MAX_OSPATH: usize = 256;
const MAX_FOUND_FILES: usize = 0x1000;
const MAX_MSGLEN: usize = 16384;
const MAX_QUED_EVENTS: usize = 256;
const MASK_QUED_EVENTS: usize = MAX_QUED_EVENTS - 1;

const _A_SUBDIR: c_uint = 0x10;
const CF_TEXT: c_uint = 1;
const ERR_FATAL: c_int = 0;
const PM_NOREMOVE: c_uint = 0;
const SEM_FAILCRITICALERRORS: c_uint = 0x0001;
const TAG_CLIPBOARD: c_int = 8;
const TAG_EVENT: c_int = 9;
const TAG_FILESYS: c_int = 10;

const CPUID_GENERIC: c_int = 0;
const CPUID_INTEL_UNSUPPORTED: c_int = 1;
const CPUID_INTEL_PENTIUM: c_int = 2;
const CPUID_INTEL_MMX: c_int = 3;
const CPUID_INTEL_KATMAI: c_int = 4;
const CPUID_INTEL_WILLIAMETTE: c_int = 5;
const CPUID_AMD_3DNOW: c_int = 6;
const CPUID_AXP: c_int = 7;

pub type BOOL = c_int;
pub type DWORD = c_ulong;
pub type HANDLE = *mut c_void;
pub type HINSTANCE = *mut HINSTANCE__;
pub type UINT = c_uint;

#[repr(C)]
pub struct HINSTANCE__ {
    unused: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct MSG {
    pub hwnd: HANDLE,
    pub message: UINT,
    pub wParam: usize,
    pub lParam: isize,
    pub time: DWORD,
    pub pt_x: c_int,
    pub pt_y: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _finddata_t {
    pub attrib: c_uint,
    pub time_create: isize,
    pub time_access: isize,
    pub time_write: isize,
    pub size: isize,
    pub name: [c_char; 260],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct clientStatic_t {
    _opaque: [byte; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct netadr_t {
    pub r#type: c_int,
    pub ip: [byte; 4],
    pub ipx: [byte; 10],
    pub port: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct msg_t {
    pub allowoverflow: qboolean,
    pub overflowed: qboolean,
    pub oob: qboolean,
    pub data: *mut byte,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sysEvent_t {
    pub evTime: c_int,
    pub evType: sysEventType_t,
    pub evValue: c_int,
    pub evValue2: c_int,
    pub evPtrLength: c_int,
    pub evPtr: *mut c_void,
}

pub type sysEventType_t = c_int;
pub const SE_NONE: sysEventType_t = 0;
pub const SE_KEY: sysEventType_t = 1;
pub const SE_CHAR: sysEventType_t = 2;
pub const SE_MOUSE: sysEventType_t = 3;
pub const SE_JOYSTICK_AXIS: sysEventType_t = 4;
pub const SE_CONSOLE: sysEventType_t = 5;
pub const SE_PACKET: sysEventType_t = 6;

#[no_mangle]
pub static mut sys_cmdline: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
#[no_mangle]
pub static mut cls: clientStatic_t = clientStatic_t { _opaque: [] };

static mut sys_monkeySpank: c_int = 0;
static mut sys_checksum: c_int = 0;

static mut g_consoleField1: [c_char; 256] = [0; 256];
static mut g_consoleField2: [c_char; 256] = [0; 256];

#[no_mangle]
pub static mut eventQue: [sysEvent_t; MAX_QUED_EVENTS] = [sysEvent_t {
    evTime: 0,
    evType: SE_NONE,
    evValue: 0,
    evValue2: 0,
    evPtrLength: 0,
    evPtr: null_mut(),
}; MAX_QUED_EVENTS];
static mut eventHead: c_int = 0;
static mut eventTail: c_int = 0;
#[no_mangle]
pub static mut sys_packetReceived: [byte; MAX_MSGLEN] = [0; MAX_MSGLEN];

static STRING_FORMAT: [c_char; 3] = [b'%' as c_char, b's' as c_char, 0];
static NEWLINE: [c_char; 2] = [b'\n' as c_char, 0];

unsafe extern "C" {
    fn printf(format: *const c_char, ...) -> c_int;
    fn sprintf(buffer: *mut c_char, format: *const c_char, ...) -> c_int;
    fn vsprintf(buffer: *mut c_char, format: *const c_char, arg: *mut c_void) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcmpi(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strtok(str: *mut c_char, delim: *const c_char) -> *mut c_char;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn malloc(size: usize) -> *mut c_void;
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn fclose(stream: *mut c_void) -> c_int;
    fn exit(status: c_int) -> !;

    fn _mkdir(path: *const c_char) -> c_int;
    fn _getcwd(buffer: *mut c_char, maxlen: c_int) -> *mut c_char;
    fn _findfirst(filespec: *const c_char, fileinfo: *mut _finddata_t) -> isize;
    fn _findnext(handle: isize, fileinfo: *mut _finddata_t) -> c_int;
    fn _findclose(handle: isize) -> c_int;
    fn kbhit() -> c_int;
    fn getch() -> c_int;

    fn GlobalMemoryStatus(lpBuffer: *mut MEMORYSTATUS);
    fn OpenClipboard(hWndNewOwner: HANDLE) -> BOOL;
    fn GetClipboardData(uFormat: UINT) -> HANDLE;
    fn GlobalLock(hMem: HANDLE) -> *mut c_void;
    fn GlobalSize(hMem: HANDLE) -> usize;
    fn GlobalUnlock(hMem: HANDLE) -> BOOL;
    fn CloseClipboard() -> BOOL;
    fn FreeLibrary(hLibModule: HINSTANCE) -> BOOL;
    fn LoadLibrary(lpLibFileName: *const c_char) -> HINSTANCE;
    fn GetProcAddress(hModule: HINSTANCE, lpProcName: *const c_char) -> *mut c_void;
    fn GetMessage(lpMsg: *mut MSG, hWnd: HANDLE, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    fn PeekMessage(
        lpMsg: *mut MSG,
        hWnd: HANDLE,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
        wRemoveMsg: UINT,
    ) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn DispatchMessage(lpMsg: *const MSG) -> isize;
    fn timeBeginPeriod(uPeriod: UINT) -> UINT;
    fn timeEndPeriod(uPeriod: UINT) -> UINT;
    fn timeGetTime() -> DWORD;
    fn SetErrorMode(uMode: UINT) -> UINT;
    fn Sleep(dwMilliseconds: DWORD);

    fn Q_CleanStr(string: *mut c_char) -> *mut c_char;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_BlockChecksum(buffer: *mut c_void, length: c_int) -> c_int;
    fn Com_Quit_f();
    fn Com_ShutdownZoneMemory();
    fn Com_ShutdownHunkMemory();
    fn Com_Init(commandLine: *mut c_char);
    fn Com_Frame();
    fn Com_FilterPath(filter: *const c_char, name: *const c_char, casesensitive: qboolean) -> qboolean;

    fn Cmd_CommandCompletion(callback: unsafe extern "C" fn(*const c_char));
    fn Cmd_AddCommand(cmd_name: *const c_char, function: unsafe extern "C" fn());
    fn Cvar_CommandCompletion(callback: unsafe extern "C" fn(*const c_char));
    fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cvar_SetValue(var_name: *const c_char, value: c_int);
    fn Cvar_VariableString(var_name: *const c_char) -> *mut c_char;

    fn CopyString(in_: *const c_char) -> *mut c_char;
    #[link_name = "Z_Malloc"]
    fn Z_Malloc(size: usize, tag: c_int) -> *mut c_void;
    #[link_name = "Z_Malloc"]
    fn Z_Malloc3(size: usize, tag: c_int, zeroit: qboolean) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    fn FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int;
    fn FS_Seek(f: fileHandle_t, offset: c_int, origin: c_int);
    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FileIsInPAK(filename: *const c_char, pChecksum: *mut c_int) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn FS_FOpenFileWrite(filename: *const c_char) -> fileHandle_t;
    fn FS_Write(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int;
    fn FS_FCloseFile(f: fileHandle_t);
    fn FS_BuildOSPath(base: *const c_char, game: *const c_char, qpath: *const c_char) -> *mut c_char;

    fn IN_Init();
    fn IN_Frame();
    fn IN_Shutdown();
    fn NET_Init();
    fn NET_Restart();
    fn MSG_Init(buf: *mut msg_t, data: *mut byte, length: c_int);
    fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean;
    fn Sys_Milliseconds() -> c_int;
    fn Sys_GetProcessorId() -> c_int;
    fn Sys_GetCurrentUser() -> *mut c_char;

    static mut com_dedicated: *mut cvar_t;
    static mut com_viewlog: *mut cvar_t;
}

#[no_mangle]
pub unsafe extern "C" fn Sys_GetBotAIAPI(parms: *mut c_void) -> *mut c_void {
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn Conbuf_AppendText(pMsg: *const c_char) {
    let mut msg: [c_char; 4096] = [0; 4096];
    strcpy(msg.as_mut_ptr(), pMsg);
    printf(Q_CleanStr(msg.as_mut_ptr()));
    printf(NEWLINE.as_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn Sys_LowPhysicalMemory() -> qboolean {
    let mut stat: MEMORYSTATUS = core::mem::zeroed();
    GlobalMemoryStatus(addr_of_mut!(stat));
    if stat.dwTotalPhys <= MEM_THRESHOLD {
        QTRUE
    } else {
        QFALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn Sys_FunctionCmp(f1: *mut c_void, f2: *mut c_void) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let l: c_int;
    let func_end: [byte; 32] = [0xC3, 0x90, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut ptr: *mut byte;
    let mut ptr2: *mut byte;
    let f1_ptr: *mut byte;
    let f2_ptr: *mut byte;

    ptr = f1 as *mut byte;
    if *ptr == 0xE9 {
        f1_ptr = (f1 as *mut byte).offset(*(ptr.add(1) as *mut c_int) as isize + 5);
    } else {
        f1_ptr = ptr;
    }

    ptr = f2 as *mut byte;
    if *ptr == 0xE9 {
        f2_ptr = (f2 as *mut byte).offset(*(ptr.add(1) as *mut c_int) as isize + 5);
    } else {
        f2_ptr = ptr;
    }

    i = 0;
    while i < 1024 {
        j = 0;
        while func_end[j as usize] != 0 {
            if *f1_ptr.offset((i + j) as isize) != func_end[j as usize] {
                break;
            }
            j += 1;
        }
        if func_end[j as usize] == 0 {
            break;
        }
        i += 1;
    }
    l = i + 2;

    i = 0;
    while i < l {
        if *f1_ptr.offset(i as isize) == 0xE8 {
            ptr = f1_ptr
                .offset(i as isize)
                .offset(*(f1_ptr.offset((i + 1) as isize) as *mut c_int) as isize + 5);
            ptr2 = f2_ptr
                .offset(i as isize)
                .offset(*(f2_ptr.offset((i + 1) as isize) as *mut c_int) as isize + 5);
            if ptr == ptr2 {
                i += 5;
                continue;
            }
        }
        if *f1_ptr.offset(i as isize) != *f2_ptr.offset(i as isize) {
            return QFALSE;
        }
        i += 1;
    }
    QTRUE
}

#[no_mangle]
pub unsafe extern "C" fn Sys_FunctionCheckSum(f1: *mut c_void) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let l: c_int;
    let func_end: [byte; 32] = [0xC3, 0x90, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let ptr: *mut byte = f1 as *mut byte;
    let f1_ptr: *mut byte;

    if *ptr == 0xE9 {
        f1_ptr = (f1 as *mut byte).offset(*(ptr.add(1) as *mut c_int) as isize + 5);
    } else {
        f1_ptr = ptr;
    }

    i = 0;
    while i < 1024 {
        j = 0;
        while func_end[j as usize] != 0 {
            if *f1_ptr.offset((i + j) as isize) != func_end[j as usize] {
                break;
            }
            j += 1;
        }
        if func_end[j as usize] == 0 {
            break;
        }
        i += 1;
    }
    l = i + 2;
    Com_BlockChecksum(f1_ptr as *mut c_void, l)
}

#[no_mangle]
pub unsafe extern "C" fn Sys_MonkeyShouldBeSpanked() -> c_int {
    sys_monkeySpank
}

#[no_mangle]
pub unsafe extern "C" fn Sys_VerifyCodeChecksum(codeBase: *mut c_void) {}

unsafe extern "C" fn PrintMatches(s: *const c_char) {
    if Q_stricmpn(
        s,
        addr_of!(g_consoleField1) as *const c_char,
        strlen(addr_of!(g_consoleField1) as *const c_char) as c_int,
    ) == 0
    {
        printf(c"    %s\n".as_ptr(), s);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Sys_ConsoleInput() -> *mut c_char {
    static mut len: c_int = 0;
    static mut bPendingExtended: bool = false;
    let ClearLine: [c_char; 82] = [
        b'\r' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char, b' ' as c_char,
        b' ' as c_char, b' ' as c_char, b' ' as c_char, b'\r' as c_char, 0, 0, 0,
    ];

    if kbhit() == 0 {
        return null_mut();
    }

    if len == 0 {
        memset(addr_of_mut!(g_consoleField1) as *mut c_void, 0, size_of::<[c_char; 256]>());
    }

    g_consoleField1[len as usize] = getch() as c_char;

    if bPendingExtended {
        match g_consoleField1[len as usize] as u8 as c_int {
            x if x == b'H' as c_int => {
                strcpy(addr_of_mut!(g_consoleField1) as *mut c_char, addr_of!(g_consoleField2) as *const c_char);
                printf(ClearLine.as_ptr());
                printf(STRING_FORMAT.as_ptr(), addr_of!(g_consoleField1) as *const c_char);
                len = strlen(addr_of!(g_consoleField1) as *const c_char) as c_int;
            }
            x if x == b'K' as c_int => {}
            x if x == b'M' as c_int => {}
            x if x == b'P' as c_int => {}
            _ => {}
        }
        g_consoleField1[len as usize] = 0;
        bPendingExtended = false;
    } else {
        match g_consoleField1[len as usize] as u8 as c_int {
            0x00 | 0xe0 => {
                g_consoleField1[len as usize] = 0;
                bPendingExtended = true;
            }
            8 => {
                printf(c"%c %c".as_ptr(), g_consoleField1[len as usize] as c_int, g_consoleField1[len as usize] as c_int);
                g_consoleField1[len as usize] = 0;
                if len > 0 {
                    len -= 1;
                }
                g_consoleField1[len as usize] = 0;
            }
            9 => {
                if len != 0 {
                    g_consoleField1[len as usize] = 0;
                    printf(NEWLINE.as_ptr());
                    Cmd_CommandCompletion(PrintMatches);
                    Cvar_CommandCompletion(PrintMatches);
                    printf(c"\n%s".as_ptr(), addr_of!(g_consoleField1) as *const c_char);
                }
            }
            27 => {
                printf(ClearLine.as_ptr());
                len = 0;
            }
            x if x == b'\r' as c_int => {
                g_consoleField1[len as usize] = 0;
                printf(NEWLINE.as_ptr());
                if len != 0 {
                    len = 0;
                    strcpy(addr_of_mut!(g_consoleField2) as *mut c_char, addr_of!(g_consoleField1) as *const c_char);
                    return addr_of_mut!(g_consoleField1) as *mut c_char;
                }
            }
            x if x == (b'v' - b'a' + 1) as c_int => {
                let mut cbd: *mut c_char;
                g_consoleField1[len as usize] = 0;
                cbd = Sys_GetClipboardData();
                if !cbd.is_null() {
                    strncpy(g_consoleField1.as_mut_ptr().add(len as usize), cbd, size_of::<[c_char; 256]>());
                    printf(STRING_FORMAT.as_ptr(), cbd);
                    len += strlen(cbd) as c_int;
                    Z_Free(cbd as *mut c_void);
                    if len == size_of::<[c_char; 256]>() as c_int {
                        len = 0;
                        return addr_of_mut!(g_consoleField1) as *mut c_char;
                    }
                }
            }
            _ => {
                printf(c"%c".as_ptr(), g_consoleField1[len as usize] as c_int);
                len += 1;
                if len == size_of::<[c_char; 256]>() as c_int {
                    len = 0;
                    return addr_of_mut!(g_consoleField1) as *mut c_char;
                }
            }
        }
    }

    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn Sys_BeginProfiling() {}

#[no_mangle]
pub unsafe extern "C" fn Sys_ShowConsole(visLevel: c_int, quitOnClose: qboolean) {}

#[no_mangle]
pub unsafe extern "C" fn Sys_Error(error: *const c_char) {
    // DEVIATION: stable Rust cannot define the original C variadic `Sys_Error`.
    let mut text: [c_char; 4096] = [0; 4096];
    let mut msg: MSG = core::mem::zeroed();

    strcpy(text.as_mut_ptr(), error);

    Conbuf_AppendText(text.as_ptr());
    Conbuf_AppendText(c"\n".as_ptr());

    Sys_ShowConsole(1, QTRUE);

    timeEndPeriod(1);

    IN_Shutdown();

    loop {
        if GetMessage(addr_of_mut!(msg), null_mut(), 0, 0) == 0 {
            Com_Quit_f();
        }
        TranslateMessage(addr_of!(msg));
        DispatchMessage(addr_of!(msg));
    }
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Quit() {
    timeEndPeriod(1);
    IN_Shutdown();
    Com_ShutdownZoneMemory();
    Com_ShutdownHunkMemory();
    exit(0);
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Print(msg: *const c_char) {
    Conbuf_AppendText(msg);
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Mkdir(path: *const c_char) {
    _mkdir(path);
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Cwd() -> *mut c_char {
    static mut cwd: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    _getcwd(addr_of_mut!(cwd) as *mut c_char, (size_of::<[c_char; MAX_OSPATH]>() - 1) as c_int);
    cwd[MAX_OSPATH - 1] = 0;
    addr_of_mut!(cwd) as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn Sys_DefaultCDPath() -> *mut c_char {
    c"".as_ptr() as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn Sys_DefaultBasePath() -> *mut c_char {
    Sys_Cwd()
}

#[no_mangle]
pub unsafe extern "C" fn Sys_ListFilteredFiles(
    basedir: *const c_char,
    subdirs: *mut c_char,
    filter: *mut c_char,
    psList: *mut *mut c_char,
    numfiles: *mut c_int,
) {
    let mut search: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut newsubdirs: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut filename: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut findinfo: _finddata_t = core::mem::zeroed();

    if *numfiles >= (MAX_FOUND_FILES - 1) as c_int {
        return;
    }

    if strlen(subdirs) != 0 {
        Com_sprintf(search.as_mut_ptr(), size_of::<[c_char; MAX_OSPATH]>() as c_int, c"%s\\%s\\*".as_ptr(), basedir, subdirs);
    } else {
        Com_sprintf(search.as_mut_ptr(), size_of::<[c_char; MAX_OSPATH]>() as c_int, c"%s\\*".as_ptr(), basedir);
    }

    let findhandle = _findfirst(search.as_ptr(), addr_of_mut!(findinfo));
    if findhandle == -1 {
        return;
    }

    loop {
        if (findinfo.attrib & _A_SUBDIR) != 0 {
            if Q_stricmp(findinfo.name.as_ptr(), c".".as_ptr()) != 0
                && Q_stricmp(findinfo.name.as_ptr(), c"..".as_ptr()) != 0
            {
                if strlen(subdirs) != 0 {
                    Com_sprintf(newsubdirs.as_mut_ptr(), size_of::<[c_char; MAX_OSPATH]>() as c_int, c"%s\\%s".as_ptr(), subdirs, findinfo.name.as_ptr());
                } else {
                    Com_sprintf(newsubdirs.as_mut_ptr(), size_of::<[c_char; MAX_OSPATH]>() as c_int, c"%s".as_ptr(), findinfo.name.as_ptr());
                }
                Sys_ListFilteredFiles(basedir, newsubdirs.as_mut_ptr(), filter, psList, numfiles);
            }
        }
        if *numfiles >= (MAX_FOUND_FILES - 1) as c_int {
            break;
        }
        Com_sprintf(filename.as_mut_ptr(), size_of::<[c_char; MAX_OSPATH]>() as c_int, c"%s\\%s".as_ptr(), subdirs, findinfo.name.as_ptr());
        if Com_FilterPath(filter, filename.as_ptr(), QFALSE) == 0 {
            if _findnext(findhandle, addr_of_mut!(findinfo)) == -1 {
                break;
            }
            continue;
        }
        *psList.offset(*numfiles as isize) = CopyString(filename.as_ptr());
        *numfiles += 1;

        if _findnext(findhandle, addr_of_mut!(findinfo)) == -1 {
            break;
        }
    }

    _findclose(findhandle);
}

unsafe fn strgtr(s0: *const c_char, s1: *const c_char) -> qboolean {
    let mut l0: c_int = strlen(s0) as c_int;
    let l1: c_int = strlen(s1) as c_int;
    let mut i: c_int;

    if l1 < l0 {
        l0 = l1;
    }

    i = 0;
    while i < l0 {
        if *s1.offset(i as isize) > *s0.offset(i as isize) {
            return QTRUE;
        }
        if *s1.offset(i as isize) < *s0.offset(i as isize) {
            return QFALSE;
        }
        i += 1;
    }
    QFALSE
}

#[no_mangle]
pub unsafe extern "C" fn Sys_ListFiles(
    directory: *const c_char,
    mut extension: *const c_char,
    filter: *mut c_char,
    numfiles: *mut c_int,
    wantsubs: qboolean,
) -> *mut *mut c_char {
    let mut search: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut nfiles: c_int;
    let mut listCopy: *mut *mut c_char;
    let mut list: [*mut c_char; MAX_FOUND_FILES] = [null_mut(); MAX_FOUND_FILES];
    let mut findinfo: _finddata_t = core::mem::zeroed();
    let mut flag: c_int;
    let mut i: c_int;

    if !filter.is_null() {
        nfiles = 0;
        Sys_ListFilteredFiles(directory, c"".as_ptr() as *mut c_char, filter, list.as_mut_ptr(), addr_of_mut!(nfiles));

        list[nfiles as usize] = null_mut();
        *numfiles = nfiles;

        if nfiles == 0 {
            return null_mut();
        }

        listCopy = Z_Malloc(((nfiles + 1) as usize) * size_of::<*mut c_char>(), TAG_FILESYS) as *mut *mut c_char;
        i = 0;
        while i < nfiles {
            *listCopy.offset(i as isize) = list[i as usize];
            i += 1;
        }
        *listCopy.offset(i as isize) = null_mut();

        return listCopy;
    }

    if extension.is_null() {
        extension = c"".as_ptr();
    }

    if *extension == b'/' as c_char && *extension.add(1) == 0 {
        extension = c"".as_ptr();
        flag = 0;
    } else {
        flag = _A_SUBDIR as c_int;
    }

    Com_sprintf(search.as_mut_ptr(), size_of::<[c_char; MAX_OSPATH]>() as c_int, c"%s\\*%s".as_ptr(), directory, extension);

    nfiles = 0;

    let findhandle = _findfirst(search.as_ptr(), addr_of_mut!(findinfo));
    if findhandle == -1 {
        *numfiles = 0;
        return null_mut();
    }

    loop {
        if ((wantsubs == 0 && (flag ^ (findinfo.attrib & _A_SUBDIR) as c_int) != 0)
            || (wantsubs != 0 && (findinfo.attrib & _A_SUBDIR) != 0))
        {
            if nfiles == (MAX_FOUND_FILES - 1) as c_int {
                break;
            }
            list[nfiles as usize] = CopyString(findinfo.name.as_ptr());
            nfiles += 1;
        }

        if _findnext(findhandle, addr_of_mut!(findinfo)) == -1 {
            break;
        }
    }

    list[nfiles as usize] = null_mut();

    _findclose(findhandle);

    *numfiles = nfiles;

    if nfiles == 0 {
        return null_mut();
    }

    listCopy = Z_Malloc(((nfiles + 1) as usize) * size_of::<*mut c_char>(), TAG_FILESYS) as *mut *mut c_char;
    i = 0;
    while i < nfiles {
        *listCopy.offset(i as isize) = list[i as usize];
        i += 1;
    }
    *listCopy.offset(i as isize) = null_mut();

    loop {
        flag = 0;
        i = 1;
        while i < nfiles {
            if strgtr(*listCopy.offset((i - 1) as isize), *listCopy.offset(i as isize)) != 0 {
                let temp = *listCopy.offset(i as isize);
                *listCopy.offset(i as isize) = *listCopy.offset((i - 1) as isize);
                *listCopy.offset((i - 1) as isize) = temp;
                flag = 1;
            }
            i += 1;
        }
        if flag == 0 {
            break;
        }
    }

    listCopy
}

#[no_mangle]
pub unsafe extern "C" fn Sys_FreeFileList(psList: *mut *mut c_char) {
    let mut i: c_int;

    if psList.is_null() {
        return;
    }

    i = 0;
    while !(*psList.offset(i as isize)).is_null() {
        Z_Free(*psList.offset(i as isize) as *mut c_void);
        i += 1;
    }

    Z_Free(psList as *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn Sys_CheckCD() -> qboolean {
    QTRUE
}

#[no_mangle]
pub unsafe extern "C" fn Sys_GetClipboardData() -> *mut c_char {
    let mut data: *mut c_char = null_mut();
    let mut cliptext: *mut c_char;

    if OpenClipboard(null_mut()) != 0 {
        let mut hClipboardData: HANDLE;

        hClipboardData = GetClipboardData(CF_TEXT);
        if !hClipboardData.is_null() {
            cliptext = GlobalLock(hClipboardData) as *mut c_char;
            if !cliptext.is_null() {
                data = Z_Malloc(GlobalSize(hClipboardData) + 1, TAG_CLIPBOARD) as *mut c_char;
                Q_strncpyz(data, cliptext, (GlobalSize(hClipboardData) + 1) as c_int);
                GlobalUnlock(hClipboardData);

                strtok(data, c"\n\r\x08".as_ptr());
            }
        }
        CloseClipboard();
    }
    data
}

#[no_mangle]
pub unsafe extern "C" fn Sys_UnloadDll(dllHandle: *mut c_void) {
    if dllHandle.is_null() {
        return;
    }
    if FreeLibrary(dllHandle as HINSTANCE) == 0 {
        Com_Error(ERR_FATAL, c"Sys_UnloadDll FreeLibrary failed".as_ptr());
    }
}

#[no_mangle]
pub unsafe extern "C" fn Sys_UnpackDLL(name: *const c_char) -> bool {
    let mut data: *mut c_void = null_mut();
    let f: fileHandle_t;
    let len: c_int = FS_ReadFile(name, addr_of_mut!(data));
    let mut ck: c_int = 0;

    if len < 1 {
        return false;
    }

    if FS_FileIsInPAK(name, addr_of_mut!(ck)) == -1 {
        FS_FreeFile(data);
        return true;
    }

    f = FS_FOpenFileWrite(name);
    if f == 0 {
        FS_FreeFile(data);
        return false;
    }

    if FS_Write(data, len, f) < len {
        FS_FreeFile(data);
        return false;
    }

    FS_FCloseFile(f);
    FS_FreeFile(data);

    true
}

pub type dllEntry_t = unsafe extern "C" fn(systemcalls: unsafe extern "C" fn(c_int, ...) -> c_int);
pub type vmMain_t = unsafe extern "C" fn(c_int, ...) -> c_int;
pub type syscall_t = unsafe extern "C" fn(c_int, ...) -> c_int;

#[no_mangle]
pub unsafe extern "C" fn Sys_LoadDll(
    name: *const c_char,
    entryPoint: *mut *mut c_void,
    systemcalls: *mut c_void,
) -> *mut c_void {
    static mut lastWarning: c_int = 0;
    let mut libHandle: HINSTANCE;
    let dllEntry: *mut c_void;
    let mut basepath: *mut c_char;
    let mut cdpath: *mut c_char;
    let mut gamedir: *mut c_char;
    let mut fn_: *mut c_char;
    let mut filename: [c_char; MAX_QPATH] = [0; MAX_QPATH];

    Com_sprintf(filename.as_mut_ptr(), size_of::<[c_char; MAX_QPATH]>() as c_int, c"%sx86.dll".as_ptr(), name);

    if !Sys_UnpackDLL(filename.as_ptr()) {
        return null_mut();
    }

    libHandle = LoadLibrary(filename.as_ptr());
    if libHandle.is_null() {
        basepath = Cvar_VariableString(c"fs_basepath".as_ptr());
        cdpath = Cvar_VariableString(c"fs_cdpath".as_ptr());
        gamedir = Cvar_VariableString(c"fs_game".as_ptr());

        fn_ = FS_BuildOSPath(basepath, gamedir, filename.as_ptr());
        libHandle = LoadLibrary(fn_);

        if libHandle.is_null() {
            if *cdpath != 0 {
                fn_ = FS_BuildOSPath(cdpath, gamedir, filename.as_ptr());
                libHandle = LoadLibrary(fn_);
            }

            if libHandle.is_null() {
                return null_mut();
            }
        }
    }

    dllEntry = GetProcAddress(libHandle, c"dllEntry".as_ptr());
    *entryPoint = GetProcAddress(libHandle, c"vmMain".as_ptr());
    if (*entryPoint).is_null() || dllEntry.is_null() {
        FreeLibrary(libHandle);
        return null_mut();
    }
    let dllEntry_fn: dllEntry_t = core::mem::transmute(dllEntry);
    let systemcalls_fn: syscall_t = core::mem::transmute(systemcalls);
    dllEntry_fn(systemcalls_fn);

    libHandle as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn Sys_InitStreamThread() {}

#[no_mangle]
pub unsafe extern "C" fn Sys_ShutdownStreamThread() {}

#[no_mangle]
pub unsafe extern "C" fn Sys_BeginStreamedFile(f: fileHandle_t, readAhead: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn Sys_EndStreamedFile(f: fileHandle_t) {}

#[no_mangle]
pub unsafe extern "C" fn Sys_StreamedRead(
    buffer: *mut c_void,
    size: c_int,
    count: c_int,
    f: fileHandle_t,
) -> c_int {
    FS_Read(buffer, size * count, f)
}

#[no_mangle]
pub unsafe extern "C" fn Sys_StreamSeek(f: fileHandle_t, offset: c_int, origin: c_int) {
    FS_Seek(f, offset, origin);
}

#[no_mangle]
pub unsafe extern "C" fn Sys_QueEvent(
    mut time: c_int,
    type_: sysEventType_t,
    value: c_int,
    value2: c_int,
    ptrLength: c_int,
    ptr: *mut c_void,
) {
    let ev: *mut sysEvent_t;

    ev = addr_of_mut!(eventQue[(eventHead as usize) & MASK_QUED_EVENTS]);
    if eventHead - eventTail >= MAX_QUED_EVENTS as c_int {
        Com_Printf(c"Sys_QueEvent: overflow\n".as_ptr());
        if !(*ev).evPtr.is_null() {
            Z_Free((*ev).evPtr);
        }
        eventTail += 1;
    }

    eventHead += 1;

    if time == 0 {
        time = Sys_Milliseconds();
    }

    (*ev).evTime = time;
    (*ev).evType = type_;
    (*ev).evValue = value;
    (*ev).evValue2 = value2;
    (*ev).evPtrLength = ptrLength;
    (*ev).evPtr = ptr;
}

#[no_mangle]
pub unsafe extern "C" fn Sys_GetEvent() -> sysEvent_t {
    let mut msg: MSG = core::mem::zeroed();
    let mut ev: sysEvent_t = core::mem::zeroed();
    let mut s: *mut c_char;
    let mut netmsg: msg_t = core::mem::zeroed();
    let mut adr: netadr_t = core::mem::zeroed();

    if eventHead > eventTail {
        eventTail += 1;
        return eventQue[((eventTail - 1) as usize) & MASK_QUED_EVENTS];
    }

    while PeekMessage(addr_of_mut!(msg), null_mut(), 0, 0, PM_NOREMOVE) != 0 {
        if GetMessage(addr_of_mut!(msg), null_mut(), 0, 0) == 0 {
            Com_Quit_f();
        }

        TranslateMessage(addr_of!(msg));
        DispatchMessage(addr_of!(msg));
    }

    s = Sys_ConsoleInput();
    if !s.is_null() {
        let mut b: *mut c_char;
        let len: c_int;

        len = strlen(s) as c_int + 1;
        b = Z_Malloc(len as usize, TAG_EVENT) as *mut c_char;
        Q_strncpyz(b, s, len);
        Sys_QueEvent(0, SE_CONSOLE, 0, 0, len, b as *mut c_void);
    }

    MSG_Init(
        addr_of_mut!(netmsg),
        addr_of_mut!(sys_packetReceived) as *mut byte,
        size_of::<[byte; MAX_MSGLEN]>() as c_int,
    );
    if Sys_GetPacket(addr_of_mut!(adr), addr_of_mut!(netmsg)) != 0 {
        let mut buf: *mut netadr_t;
        let len: c_int;

        len = size_of::<netadr_t>() as c_int + netmsg.cursize - netmsg.readcount;
        buf = Z_Malloc3(len as usize, TAG_EVENT, QTRUE) as *mut netadr_t;
        *buf = adr;
        memcpy(
            buf.add(1) as *mut c_void,
            netmsg.data.add(netmsg.readcount as usize) as *const c_void,
            (netmsg.cursize - netmsg.readcount) as usize,
        );
        Sys_QueEvent(0, SE_PACKET, 0, 0, len, buf as *mut c_void);
    }

    if eventHead > eventTail {
        eventTail += 1;
        return eventQue[((eventTail - 1) as usize) & MASK_QUED_EVENTS];
    }

    memset(addr_of_mut!(ev) as *mut c_void, 0, size_of::<sysEvent_t>());
    ev.evTime = timeGetTime() as c_int;

    ev
}

#[no_mangle]
pub unsafe extern "C" fn Sys_In_Restart_f() {
    IN_Shutdown();
    IN_Init();
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Net_Restart_f() {
    NET_Restart();
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Init() {
    let mut cpuid: c_int;

    timeBeginPeriod(1);

    Cmd_AddCommand(c"in_restart".as_ptr(), Sys_In_Restart_f);
    Cmd_AddCommand(c"net_restart".as_ptr(), Sys_Net_Restart_f);

    Cvar_Get(c"sys_cpustring".as_ptr(), c"detect".as_ptr(), 0);
    if Q_stricmp(Cvar_VariableString(c"sys_cpustring".as_ptr()), c"detect".as_ptr()) == 0 {
        Com_Printf(c"...detecting CPU, found ".as_ptr());

        cpuid = Sys_GetProcessorId();

        match cpuid {
            CPUID_GENERIC => Cvar_Set(c"sys_cpustring".as_ptr(), c"generic".as_ptr()),
            CPUID_INTEL_UNSUPPORTED => Cvar_Set(c"sys_cpustring".as_ptr(), c"x86 (pre-Pentium)".as_ptr()),
            CPUID_INTEL_PENTIUM => Cvar_Set(c"sys_cpustring".as_ptr(), c"x86 (P5/PPro, non-MMX)".as_ptr()),
            CPUID_INTEL_MMX => Cvar_Set(c"sys_cpustring".as_ptr(), c"x86 (P5/Pentium2, MMX)".as_ptr()),
            CPUID_INTEL_KATMAI => Cvar_Set(c"sys_cpustring".as_ptr(), c"Intel Pentium III".as_ptr()),
            CPUID_INTEL_WILLIAMETTE => Cvar_Set(c"sys_cpustring".as_ptr(), c"Intel Pentium IV".as_ptr()),
            CPUID_AMD_3DNOW => Cvar_Set(c"sys_cpustring".as_ptr(), c"AMD w/ 3DNow!".as_ptr()),
            CPUID_AXP => Cvar_Set(c"sys_cpustring".as_ptr(), c"Alpha AXP".as_ptr()),
            _ => {
                Com_Error(ERR_FATAL, c"Unknown cpu type %d\n".as_ptr(), cpuid);
            }
        }
    } else {
        Com_Printf(c"...forcing CPU type to ".as_ptr());
        if Q_stricmp(Cvar_VariableString(c"sys_cpustring".as_ptr()), c"generic".as_ptr()) == 0 {
            cpuid = CPUID_GENERIC;
        } else if Q_stricmp(Cvar_VariableString(c"sys_cpustring".as_ptr()), c"x87".as_ptr()) == 0 {
            cpuid = CPUID_INTEL_PENTIUM;
        } else if Q_stricmp(Cvar_VariableString(c"sys_cpustring".as_ptr()), c"mmx".as_ptr()) == 0 {
            cpuid = CPUID_INTEL_MMX;
        } else if Q_stricmp(Cvar_VariableString(c"sys_cpustring".as_ptr()), c"3dnow".as_ptr()) == 0 {
            cpuid = CPUID_AMD_3DNOW;
        } else if Q_stricmp(Cvar_VariableString(c"sys_cpustring".as_ptr()), c"PentiumIII".as_ptr()) == 0 {
            cpuid = CPUID_INTEL_KATMAI;
        } else if Q_stricmp(Cvar_VariableString(c"sys_cpustring".as_ptr()), c"PentiumIV".as_ptr()) == 0 {
            cpuid = CPUID_INTEL_WILLIAMETTE;
        } else if Q_stricmp(Cvar_VariableString(c"sys_cpustring".as_ptr()), c"axp".as_ptr()) == 0 {
            cpuid = CPUID_AXP;
        } else {
            Com_Printf(
                c"WARNING: unknown sys_cpustring '%s'\n".as_ptr(),
                Cvar_VariableString(c"sys_cpustring".as_ptr()),
            );
            cpuid = CPUID_GENERIC;
        }
    }
    Cvar_SetValue(c"sys_cpuid".as_ptr(), cpuid);
    Com_Printf(c"%s\n".as_ptr(), Cvar_VariableString(c"sys_cpustring".as_ptr()));

    Cvar_Set(c"username".as_ptr(), Sys_GetCurrentUser());

    IN_Init();
}

#[no_mangle]
pub unsafe extern "C" fn main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut cwd: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let cmdline: *mut c_char;
    let mut i: c_int;
    let mut len: c_int;

    len = 1;
    i = 1;
    while i < argc {
        len += strlen(*argv.offset(i as isize)) as c_int + 1;
        i += 1;
    }
    cmdline = malloc(len as usize) as *mut c_char;
    *cmdline = 0;
    i = 1;
    while i < argc {
        if i > 1 {
            strcat(cmdline, c" ".as_ptr());
        }
        strcat(cmdline, *argv.offset(i as isize));
        i += 1;
    }

    SetErrorMode(SEM_FAILCRITICALERRORS);

    Sys_Milliseconds();

    Sys_InitStreamThread();

    Com_Init(cmdline);

    NET_Init();

    _getcwd(cwd.as_mut_ptr(), size_of::<[c_char; MAX_OSPATH]>() as c_int);
    Com_Printf(c"Working directory: %s\n".as_ptr(), cwd.as_ptr());

    if (*com_dedicated).integer == 0 && (*com_viewlog).integer == 0 {
        Sys_ShowConsole(0, QFALSE);
    }

    loop {
        Sleep(5);

        IN_Frame();

        Com_Frame();
    }
}
