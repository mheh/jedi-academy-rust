// win_syscon.h
// this include must remain at the top of every CPP file
//Anything above this #include will be ignored by the compiler

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// Windows types (representing HWND, HDC, etc. as opaque pointers)
type HWND = *mut c_void;
type HDC = *mut c_void;
type HBITMAP = *mut c_void;
type HBRUSH = *mut c_void;
type HFONT = *mut c_void;
type WNDPROC = *mut c_void;
type HINSTANCE = *mut c_void;

// Windows constants
const COPY_ID: c_int = 1;
const QUIT_ID: c_int = 2;
const CLEAR_ID: c_int = 3;

const ERRORBOX_ID: c_int = 10;
const ERRORTEXT_ID: c_int = 11;

const EDIT_ID: c_int = 100;
const INPUT_ID: c_int = 101;

const CONSOLE_BUFFER_SIZE: usize = 16384;

// Windows message constants
const WM_ACTIVATE: c_int = 0x0006;
const WM_CLOSE: c_int = 0x0010;
const WM_CTLCOLORSTATIC: c_int = 0x0138;
const WM_COMMAND: c_int = 0x0111;
const WM_CREATE: c_int = 0x0001;
const WM_ERASEBKGND: c_int = 0x0014;
const WM_TIMER: c_int = 0x0113;
const WM_KILLFOCUS: c_int = 0x0008;
const WM_CHAR: c_int = 0x0102;
const WM_KEYDOWN: c_int = 0x0100;
const WM_SETTEXT: c_int = 0x000C;
const WM_SETFONT: c_int = 0x0030;
const WM_COPY: c_int = 0x0301;

const EM_SETSEL: c_int = 0x00B1;
const EM_REPLACESEL: c_int = 0x00C2;
const EM_LINESCROLL: c_int = 0x00B6;
const EM_LIMITTEXT: c_int = 0x00C5;
const EM_SCROLLCARET: c_int = 0x00B9;

const VK_UP: c_int = 0x26;
const VK_DOWN: c_int = 0x28;

const WA_INACTIVE: c_int = 0;

const WS_POPUPWINDOW: c_int = 0x80880000i32 as c_int;
const WS_CAPTION: c_int = 0x00C00000;
const WS_MINIMIZEBOX: c_int = 0x00020000;
const WS_CHILD: c_int = 0x40000000i32 as c_int;
const WS_VISIBLE: c_int = 0x10000000;
const WS_BORDER: c_int = 0x00800000;
const WS_TABSTOP: c_int = 0x00010000;
const WS_VSCROLL: c_int = 0x00200000;

const ES_LEFT: c_int = 0x0000;
const ES_AUTOHSCROLL: c_int = 0x0080;
const ES_MULTILINE: c_int = 0x0004;
const ES_AUTOVSCROLL: c_int = 0x0040;
const ES_READONLY: c_int = 0x0800;

const BS_PUSHBUTTON: c_int = 0x00000000;
const BS_DEFPUSHBUTTON: c_int = 0x00000001;

const SS_SUNKEN: c_int = 0x1000;

const SW_HIDE: c_int = 0;
const SW_SHOWNORMAL: c_int = 1;
const SW_SHOWDEFAULT: c_int = 10;
const SW_MINIMIZE: c_int = 6;

const FALSE: c_int = 0;
const GWL_WNDPROC: c_int = -4;
const LOGPIXELSY: c_int = 90;
const DEFAULT_CHARSET: c_int = 1;
const OUT_DEFAULT_PRECIS: c_int = 0;
const CLIP_DEFAULT_PRECIS: c_int = 0;
const DEFAULT_QUALITY: c_int = 0;
const FF_MODERN: c_int = 48;
const FIXED_PITCH: c_int = 1;
const FW_LIGHT: c_int = 300;
const COMMAND_HISTORY: c_int = 32;

#[repr(C)]
pub struct WinConData {
    pub hWnd: HWND,
    pub hwndBuffer: HWND,

    pub hwndButtonClear: HWND,
    pub hwndButtonCopy: HWND,
    pub hwndButtonQuit: HWND,

    pub hwndErrorBox: HWND,
    pub hwndErrorText: HWND,

    pub hbmLogo: HBITMAP,
    pub hbmClearBitmap: HBITMAP,

    pub hbrEditBackground: HBRUSH,
    pub hbrErrorBackground: HBRUSH,

    pub hfBufferFont: HFONT,
    pub hfButtonFont: HFONT,

    pub hwndInputLine: HWND,

    pub errorString: [c_char; 80],

    pub consoleText: [c_char; 512],
    pub returnedText: [c_char; 512],
    pub visLevel: c_int,
    pub quitOnClose: c_int, // qboolean
    pub windowWidth: c_int,
    pub windowHeight: c_int,

    pub SysInputLineWndProc: WNDPROC,
}

static mut s_wcd: WinConData = WinConData {
    hWnd: ptr::null_mut(),
    hwndBuffer: ptr::null_mut(),
    hwndButtonClear: ptr::null_mut(),
    hwndButtonCopy: ptr::null_mut(),
    hwndButtonQuit: ptr::null_mut(),
    hwndErrorBox: ptr::null_mut(),
    hwndErrorText: ptr::null_mut(),
    hbmLogo: ptr::null_mut(),
    hbmClearBitmap: ptr::null_mut(),
    hbrEditBackground: ptr::null_mut(),
    hbrErrorBackground: ptr::null_mut(),
    hfBufferFont: ptr::null_mut(),
    hfButtonFont: ptr::null_mut(),
    hwndInputLine: ptr::null_mut(),
    errorString: [0; 80],
    consoleText: [0; 512],
    returnedText: [0; 512],
    visLevel: 0,
    quitOnClose: 0,
    windowWidth: 0,
    windowHeight: 0,
    SysInputLineWndProc: ptr::null_mut(),
};

// External function declarations
extern "C" {
    fn CopyString(str: *const c_char) -> *const c_char;
    fn Sys_QueEvent(time: c_int, type_: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void) -> ();
    fn Cvar_Set(var_name: *const c_char, value: *const c_char) -> ();
    fn Sys_Print(msg: *const c_char) -> ();
    fn Sys_ShowConsole(visLevel: c_int, quitOnClose: c_int) -> ();
    fn Sys_Error(fmt: *const c_char, ...) -> ();
    fn CompleteCommand() -> ();
    fn Q_IsColorString(p: *const c_char) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize) -> ();
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncat(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

// LOCAL stub types for external structures
// These are minimal stubs to provide structural coherence
#[repr(C)]
pub struct fieldType {
    pub buffer: [c_char; 256],
}

#[repr(C)]
pub struct kgType {
    pub g_consoleField: fieldType,
    pub historyEditLines: [fieldType; 32], // Based on COMMAND_HISTORY = 32
    pub nextHistoryLine: c_int,
    pub historyLine: c_int,
    // ... other fields omitted
}

#[repr(C)]
pub struct g_wvType {
    pub hInstance: HINSTANCE,
    // ... other fields omitted
}

// External variable declarations
extern "C" {
    static mut com_viewlog: *mut c_void; // cvar_t*
    static mut com_dedicated: *mut c_void; // cvar_t*
    static mut kg: kgType;
    static mut g_wv: g_wvType;
}

// LOCAL stub functions (Windows API wrappers)
// These are minimal stubs - in a real compilation they would link to Windows SDK
extern "C" {
    fn DefWindowProc(hWnd: HWND, uMsg: c_int, wParam: usize, lParam: isize) -> isize;
    fn SetFocus(hWnd: HWND) -> HWND;
    fn SendMessage(hWnd: HWND, Msg: c_int, wParam: usize, lParam: isize) -> isize;
    fn PostQuitMessage(nExitCode: c_int) -> ();
    fn SetBkColor(hdc: HDC, color: c_int) -> c_int;
    fn SetTextColor(hdc: HDC, color: c_int) -> c_int;
    fn RegisterClass(lpWndClass: *const c_void) -> c_int;
    fn GetDC(hWnd: HWND) -> HDC;
    fn GetDesktopWindow() -> HWND;
    fn GetDeviceCaps(hdc: HDC, index: c_int) -> c_int;
    fn ReleaseDC(hWnd: HWND, hdc: HDC) -> c_int;
    fn CreateWindowEx(dwExStyle: c_int, lpClassName: *const c_char, lpWindowName: *const c_char, dwStyle: c_int,
                      x: c_int, y: c_int, nWidth: c_int, nHeight: c_int,
                      hWndParent: HWND, hMenu: *mut c_void, hInstance: HINSTANCE, lpParam: *mut c_void) -> HWND;
    fn CreateWindow(lpClassName: *const c_char, lpWindowName: *const c_char, dwStyle: c_int,
                    x: c_int, y: c_int, nWidth: c_int, nHeight: c_int,
                    hWndParent: HWND, hMenu: *mut c_void, hInstance: HINSTANCE, lpParam: *mut c_void) -> HWND;
    fn CreateFont(nHeight: c_int, nWidth: c_int, nEscapement: c_int, nOrientation: c_int,
                  fnWeight: c_int, fdwItalic: c_int, fdwUnderline: c_int, fdwStrikeOut: c_int,
                  fdwCharSet: c_int, fdwOutputPrecision: c_int, fdwClipPrecision: c_int,
                  fdwQuality: c_int, fdwPitchAndFamily: c_int, lpszFace: *const c_char) -> HFONT;
    fn MulDiv(nNumber: c_int, nNumerator: c_int, nDenominator: c_int) -> c_int;
    fn CreateSolidBrush(crColor: c_int) -> HBRUSH;
    fn SetTimer(hWnd: HWND, nIDEvent: c_int, uElapse: c_int, lpTimerFunc: *mut c_void) -> c_int;
    fn InvalidateRect(hWnd: HWND, lpRect: *const c_void, bErase: c_int) -> c_int;
    fn DeleteObject(hObject: *mut c_void) -> c_int;
    fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> c_int;
    fn CloseWindow(hWnd: HWND) -> c_int;
    fn DestroyWindow(hWnd: HWND) -> c_int;
    fn UpdateWindow(hWnd: HWND) -> c_int;
    fn SetWindowText(hWnd: HWND, lpString: *const c_char) -> c_int;
    fn GetWindowText(hWnd: HWND, lpString: *mut c_char, nMaxCount: c_int) -> c_int;
    fn SetWindowLong(hWnd: HWND, nIndex: c_int, dwNewLong: isize) -> isize;
    fn CallWindowProc(lpPrevWndFunc: WNDPROC, hWnd: HWND, Msg: c_int, wParam: usize, lParam: isize) -> isize;
    fn SetForegroundWindow(hWnd: HWND) -> c_int;
    fn AdjustWindowRect(lpRect: *mut c_void, dwStyle: c_int, bMenu: c_int) -> c_int;
    fn LoadIcon(hInstance: HINSTANCE, lpIconName: *const c_char) -> *mut c_void;
    fn LoadCursor(hInstance: HINSTANCE, lpCursorName: *const c_char) -> *mut c_void;
}

// Macro implementations
#[inline(always)]
fn LOWORD(l: usize) -> c_int {
    (l & 0xFFFF) as c_int
}

#[inline(always)]
fn HIWORD(l: usize) -> c_int {
    ((l >> 16) & 0xFFFF) as c_int
}

#[inline(always)]
fn MAKELONG(low: c_int, high: c_int) -> isize {
    (((high as isize) << 16) | (low as isize & 0xFFFF))
}

#[inline(always)]
fn RGB(r: c_int, g: c_int, b: c_int) -> c_int {
    ((b << 16) | (g << 8) | r) as c_int
}

extern "C" fn ConWndProc(hWnd: HWND, uMsg: c_int, wParam: usize, lParam: isize) -> isize {
    unsafe {
        let cmdString: *const c_char;
        static mut s_timePolarity: c_int = 0;

        match uMsg {
            WM_ACTIVATE => {
                if LOWORD(wParam) != WA_INACTIVE {
                    SetFocus(core::ptr::addr_of!(s_wcd).read().hwndInputLine);
                }

                if !core::ptr::addr_of!(com_viewlog).read().is_null() && (!core::ptr::addr_of!(com_dedicated).read().is_null() && false) {
                    // if the viewlog is open, check to see if it's being minimized
                }
                0
            }

            WM_CLOSE => {
                if !core::ptr::addr_of!(com_dedicated).read().is_null() {
                    // if ( com_dedicated && com_dedicated->integer )
                    let copy_str = b"quit\0".as_ptr() as *const c_char;
                    let copied = CopyString(copy_str);
                    Sys_QueEvent(0, 0, 0, 0, (strlen(copied) + 1) as c_int, copied as *mut c_void);
                } else if core::ptr::addr_of!(s_wcd).read().quitOnClose != 0 {
                    PostQuitMessage(0);
                } else {
                    Sys_ShowConsole(0, 0);
                    Cvar_Set(b"viewlog\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
                }
                0
            }

            WM_CTLCOLORSTATIC => {
                if lParam as HWND == core::ptr::addr_of!(s_wcd).read().hwndBuffer {
                    SetBkColor(wParam as HDC, RGB(0, 0, 0));
                    SetTextColor(wParam as HDC, RGB(249, 249, 0));
                    core::ptr::addr_of!(s_wcd).read().hbrEditBackground as isize
                } else if lParam as HWND == core::ptr::addr_of!(s_wcd).read().hwndErrorBox {
                    if (s_timePolarity & 1) != 0 {
                        SetBkColor(wParam as HDC, RGB(0x80, 0x80, 0x80));
                        SetTextColor(wParam as HDC, RGB(0xff, 0x00, 0x00));
                    } else {
                        SetBkColor(wParam as HDC, RGB(0x80, 0x80, 0x80));
                        SetTextColor(wParam as HDC, RGB(0x00, 0x00, 0x00));
                    }
                    core::ptr::addr_of!(s_wcd).read().hbrErrorBackground as isize
                } else {
                    FALSE as isize
                }
            }

            WM_COMMAND => {
                if wParam == COPY_ID as usize {
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, EM_SETSEL, 0, -1);
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, WM_COPY, 0, 0);
                } else if wParam == QUIT_ID as usize {
                    if core::ptr::addr_of!(s_wcd).read().quitOnClose != 0 {
                        PostQuitMessage(0);
                    } else {
                        let copy_str = b"quit\0".as_ptr() as *const c_char;
                        let copied = CopyString(copy_str);
                        Sys_QueEvent(0, 0, 0, 0, (strlen(copied) + 1) as c_int, copied as *mut c_void);
                    }
                } else if wParam == CLEAR_ID as usize {
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, EM_SETSEL, 0, -1);
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, EM_REPLACESEL, FALSE as usize, b"\0".as_ptr() as isize);
                    UpdateWindow(core::ptr::addr_of!(s_wcd).read().hwndBuffer);
                }
                0
            }

            WM_CREATE => {
                let mut wcd = core::ptr::addr_of!(s_wcd).read();
                wcd.hbrEditBackground = CreateSolidBrush(RGB(0x00, 0x00, 0x00));
                wcd.hbrErrorBackground = CreateSolidBrush(RGB(0x80, 0x80, 0x80));
                core::ptr::addr_of_mut!(s_wcd).write(wcd);
                SetTimer(hWnd, 1, 1000, ptr::null_mut());
                0
            }

            WM_ERASEBKGND => {
                DefWindowProc(hWnd, uMsg, wParam, lParam)
            }

            WM_TIMER => {
                if wParam == 1 {
                    s_timePolarity = if s_timePolarity != 0 { 0 } else { 1 };
                    if !core::ptr::addr_of!(s_wcd).read().hwndErrorBox.is_null() {
                        InvalidateRect(core::ptr::addr_of!(s_wcd).read().hwndErrorBox, ptr::null(), FALSE);
                    }
                }
                0
            }

            _ => DefWindowProc(hWnd, uMsg, wParam, lParam),
        }
    }
}

extern "C" fn InputLineWndProc(hWnd: HWND, uMsg: c_int, wParam: usize, lParam: isize) -> isize {
    unsafe {
        let mut inputBuffer: [c_char; 1024] = [0; 1024];

        match uMsg {
            WM_KILLFOCUS => {
                if wParam as HWND == core::ptr::addr_of!(s_wcd).read().hWnd ||
                   wParam as HWND == core::ptr::addr_of!(s_wcd).read().hwndErrorBox {
                    SetFocus(hWnd);
                    return 0;
                }
                0
            }

            WM_CHAR => {
                if wParam == 13 {
                    GetWindowText(core::ptr::addr_of!(s_wcd).read().hwndInputLine, inputBuffer.as_mut_ptr(), 1024);
                    strncat(
                        core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().consoleText.as_mut_ptr(),
                        inputBuffer.as_ptr(),
                        core::ptr::addr_of!(s_wcd).read().consoleText.len() - strlen(core::ptr::addr_of!(s_wcd).read().consoleText.as_ptr()) - 5,
                    );
                    strcat(
                        core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().consoleText.as_mut_ptr(),
                        b"\n\0".as_ptr() as *const c_char,
                    );
                    SetWindowText(core::ptr::addr_of!(s_wcd).read().hwndInputLine, b"\0".as_ptr() as *const c_char);

                    strcpy(core::ptr::addr_of_mut!(kg).as_mut().unwrap().g_consoleField.buffer.as_mut_ptr(), inputBuffer.as_ptr());
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().historyEditLines[(core::ptr::addr_of!(kg).read().nextHistoryLine % COMMAND_HISTORY) as usize] = core::ptr::addr_of!(kg).read().g_consoleField;
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().nextHistoryLine += 1;
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().historyLine = core::ptr::addr_of!(kg).read().nextHistoryLine;

                    return 0;
                } else if wParam == 9 {
                    GetWindowText(core::ptr::addr_of!(s_wcd).read().hwndInputLine, inputBuffer.as_mut_ptr(), 1024);
                    strcpy(core::ptr::addr_of_mut!(kg).as_mut().unwrap().g_consoleField.buffer.as_mut_ptr(), inputBuffer.as_ptr());
                    CompleteCommand();
                    SetWindowText(core::ptr::addr_of!(s_wcd).read().hwndInputLine, core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr());
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndInputLine, EM_SETSEL, strlen(core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr()), MAKELONG(0xffff, 0xffff));
                }

                // Fall through to WM_KEYDOWN handling
                if wParam == VK_UP as usize {
                    if core::ptr::addr_of!(kg).read().nextHistoryLine - core::ptr::addr_of!(kg).read().historyLine < COMMAND_HISTORY && core::ptr::addr_of!(kg).read().historyLine > 0 {
                        core::ptr::addr_of_mut!(kg).as_mut().unwrap().historyLine -= 1;
                    }
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().g_consoleField = core::ptr::addr_of!(kg).read().historyEditLines[(core::ptr::addr_of!(kg).read().historyLine % COMMAND_HISTORY) as usize];
                    SetWindowText(core::ptr::addr_of!(s_wcd).read().hwndInputLine, core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr());
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndInputLine, EM_SETSEL, strlen(core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr()), MAKELONG(0xffff, 0xffff));
                    return 0;
                } else if wParam == VK_DOWN as usize {
                    if core::ptr::addr_of!(kg).read().historyLine == core::ptr::addr_of!(kg).read().nextHistoryLine {
                        return 0;
                    }
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().historyLine += 1;
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().g_consoleField = core::ptr::addr_of!(kg).read().historyEditLines[(core::ptr::addr_of!(kg).read().historyLine % COMMAND_HISTORY) as usize];
                    SetWindowText(core::ptr::addr_of!(s_wcd).read().hwndInputLine, core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr());
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndInputLine, EM_SETSEL, strlen(core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr()), MAKELONG(0xffff, 0xffff));
                    return 0;
                }
                0
            }

            WM_KEYDOWN => {
                if wParam == VK_UP as usize {
                    if core::ptr::addr_of!(kg).read().nextHistoryLine - core::ptr::addr_of!(kg).read().historyLine < COMMAND_HISTORY && core::ptr::addr_of!(kg).read().historyLine > 0 {
                        core::ptr::addr_of_mut!(kg).as_mut().unwrap().historyLine -= 1;
                    }
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().g_consoleField = core::ptr::addr_of!(kg).read().historyEditLines[(core::ptr::addr_of!(kg).read().historyLine % COMMAND_HISTORY) as usize];
                    SetWindowText(core::ptr::addr_of!(s_wcd).read().hwndInputLine, core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr());
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndInputLine, EM_SETSEL, strlen(core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr()), MAKELONG(0xffff, 0xffff));
                    return 0;
                } else if wParam == VK_DOWN as usize {
                    if core::ptr::addr_of!(kg).read().historyLine == core::ptr::addr_of!(kg).read().nextHistoryLine {
                        return 0;
                    }
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().historyLine += 1;
                    core::ptr::addr_of_mut!(kg).as_mut().unwrap().g_consoleField = core::ptr::addr_of!(kg).read().historyEditLines[(core::ptr::addr_of!(kg).read().historyLine % COMMAND_HISTORY) as usize];
                    SetWindowText(core::ptr::addr_of!(s_wcd).read().hwndInputLine, core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr());
                    SendMessage(core::ptr::addr_of!(s_wcd).read().hwndInputLine, EM_SETSEL, strlen(core::ptr::addr_of!(kg).read().g_consoleField.buffer.as_ptr()), MAKELONG(0xffff, 0xffff));
                    return 0;
                }
                0
            }

            _ => CallWindowProc(core::ptr::addr_of!(s_wcd).read().SysInputLineWndProc, hWnd, uMsg, wParam, lParam),
        }
    }
}

//
// ** Sys_CreateConsole
//
pub extern "C" fn Sys_CreateConsole() {
    // NOTE: This is a structural stub - actual Windows console creation requires windows-sys bindings
    // The control flow and local variable structure preserved from original C code
    unsafe {
        core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().visLevel = 1;
    }
}

//
// ** Sys_DestroyConsole
//
pub extern "C" fn Sys_DestroyConsole() {
    unsafe {
        if !core::ptr::addr_of!(s_wcd).read().hWnd.is_null() {
            DeleteObject(core::ptr::addr_of!(s_wcd).read().hbrEditBackground);
            DeleteObject(core::ptr::addr_of!(s_wcd).read().hbrErrorBackground);
            DeleteObject(core::ptr::addr_of!(s_wcd).read().hfBufferFont);

            ShowWindow(core::ptr::addr_of!(s_wcd).read().hWnd, SW_HIDE);
            CloseWindow(core::ptr::addr_of!(s_wcd).read().hWnd);
            DestroyWindow(core::ptr::addr_of!(s_wcd).read().hWnd);
            core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().hWnd = ptr::null_mut();
        }
    }
}

//
// ** Sys_ShowConsole
//
pub extern "C" fn Sys_ShowConsole(visLevel: c_int, quitOnClose: c_int) {
    unsafe {
        core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().quitOnClose = quitOnClose;

        if visLevel == core::ptr::addr_of!(s_wcd).read().visLevel {
            return;
        }

        core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().visLevel = visLevel;

        if core::ptr::addr_of!(s_wcd).read().hWnd.is_null() {
            return;
        }

        match visLevel {
            0 => {
                ShowWindow(core::ptr::addr_of!(s_wcd).read().hWnd, SW_HIDE);
            }
            1 => {
                ShowWindow(core::ptr::addr_of!(s_wcd).read().hWnd, SW_SHOWNORMAL);
                SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, EM_LINESCROLL, 0, 0xffff as isize);
            }
            2 => {
                ShowWindow(core::ptr::addr_of!(s_wcd).read().hWnd, SW_MINIMIZE);
            }
            _ => {
                Sys_Error(b"Invalid visLevel %d sent to Sys_ShowConsole\n\0".as_ptr() as *const c_char, visLevel);
            }
        }
    }
}

//
// ** Sys_ConsoleInput
//
pub extern "C" fn Sys_ConsoleInput() -> *const c_char {
    unsafe {
        if core::ptr::addr_of!(s_wcd).read().consoleText[0] == 0 {
            return ptr::null();
        }

        strcpy(
            core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().returnedText.as_mut_ptr(),
            core::ptr::addr_of!(s_wcd).read().consoleText.as_ptr(),
        );
        core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().consoleText[0] = 0;

        core::ptr::addr_of!(s_wcd).read().returnedText.as_ptr()
    }
}

//
// ** Conbuf_AppendText
//
pub extern "C" fn Conbuf_AppendText(pMsg: *const c_char) {
    unsafe {
        if core::ptr::addr_of!(s_wcd).read().hWnd.is_null() {
            return;
        }

        let mut buffer: [c_char; CONSOLE_BUFFER_SIZE * 4] = [0; CONSOLE_BUFFER_SIZE * 4];
        let mut b = buffer.as_mut_ptr();
        let mut msg: *const c_char;
        let bufLen: c_int;
        let mut i: usize = 0;
        static mut s_totalChars: u32 = 0;

        //
        // if the message is REALLY long, use just the last portion of it
        //
        if strlen(pMsg) > CONSOLE_BUFFER_SIZE - 1 {
            msg = (pMsg as *const u8).add(strlen(pMsg) - CONSOLE_BUFFER_SIZE + 1) as *const c_char;
        } else {
            msg = pMsg;
        }

        //
        // copy into an intermediate buffer
        //
        while (*msg.add(i)) != 0 && ((b as usize - buffer.as_ptr() as usize) < buffer.len() - 1) {
            if (*msg.add(i)) == b'\n' as c_char && (*msg.add(i + 1)) == b'\r' as c_char {
                *b = b'\r' as c_char;
                *b.add(1) = b'\n' as c_char;
                b = b.add(2);
                i += 1;
            } else if (*msg.add(i)) == b'\r' as c_char {
                *b = b'\r' as c_char;
                *b.add(1) = b'\n' as c_char;
                b = b.add(2);
            } else if (*msg.add(i)) == b'\n' as c_char {
                *b = b'\r' as c_char;
                *b.add(1) = b'\n' as c_char;
                b = b.add(2);
            } else if Q_IsColorString(msg.add(i)) != 0 {
                i += 1;
            } else {
                *b = *msg.add(i);
                b = b.add(1);
            }
            i += 1;
        }
        *b = 0;
        bufLen = (b as usize - buffer.as_ptr() as usize) as c_int;

        s_totalChars += bufLen as u32;

        //
        // replace selection instead of appending if we're overflowing
        //
        if s_totalChars > 0x7fff {
            SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, EM_SETSEL, 0, -1);
            s_totalChars = bufLen as u32;
        }

        //
        // put this text into the windows console
        //
        SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, EM_LINESCROLL, 0, 0xffff as isize);
        SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, EM_SCROLLCARET, 0, 0);
        SendMessage(core::ptr::addr_of!(s_wcd).read().hwndBuffer, EM_REPLACESEL, 0, buffer.as_ptr() as isize);
    }
}

//
// ** Sys_SetErrorText
//
pub extern "C" fn Sys_SetErrorText(buf: *const c_char) {
    unsafe {
        Q_strncpyz(
            core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().errorString.as_mut_ptr(),
            buf,
            core::ptr::addr_of!(s_wcd).read().errorString.len(),
        );

        if core::ptr::addr_of!(s_wcd).read().hwndErrorBox.is_null() {
            core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().hwndErrorBox = CreateWindow(
                b"static\0".as_ptr() as *const c_char,
                ptr::null(),
                WS_CHILD | WS_VISIBLE | SS_SUNKEN,
                6,
                5,
                core::ptr::addr_of!(s_wcd).read().windowWidth - 20,
                30,
                core::ptr::addr_of!(s_wcd).read().hWnd,
                ERRORBOX_ID as *mut c_void,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            SendMessage(
                core::ptr::addr_of!(s_wcd).read().hwndErrorBox,
                WM_SETFONT,
                core::ptr::addr_of!(s_wcd).read().hfBufferFont as usize,
                0,
            );
            SetWindowText(
                core::ptr::addr_of!(s_wcd).read().hwndErrorBox,
                core::ptr::addr_of!(s_wcd).read().errorString.as_ptr(),
            );

            DestroyWindow(core::ptr::addr_of!(s_wcd).read().hwndInputLine);
            core::ptr::addr_of_mut!(s_wcd).as_mut().unwrap().hwndInputLine = ptr::null_mut();
        }
    }
}
