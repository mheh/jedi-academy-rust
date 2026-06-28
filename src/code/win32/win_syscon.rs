// win_syscon.h

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

// #include "../client/client.h"
// #include "win_local.h"
// #include "resource.h"
// #include <errno.h>
// #include <float.h>
// #include <fcntl.h>
// #include <stdio.h>
// #include <direct.h>
// #include <io.h>
// #include <conio.h>

use core::ffi::{c_char, c_int, c_void};
use core::mem;
use core::ptr::{self, addr_of, addr_of_mut};

// Windows type declarations
#[allow(non_camel_case_types)]
pub type HWND = *mut c_void;
#[allow(non_camel_case_types)]
pub type HBITMAP = *mut c_void;
#[allow(non_camel_case_types)]
pub type HBRUSH = *mut c_void;
#[allow(non_camel_case_types)]
pub type HFONT = *mut c_void;
#[allow(non_camel_case_types)]
pub type HDC = *mut c_void;
#[allow(non_camel_case_types)]
pub type HINSTANCE = *mut c_void;
#[allow(non_camel_case_types)]
pub type UINT = c_int;
#[allow(non_camel_case_types)]
pub type WPARAM = usize;
#[allow(non_camel_case_types)]
pub type LPARAM = isize;
#[allow(non_camel_case_types)]
pub type WNDPROC = extern "C" fn(HWND, UINT, WPARAM, LPARAM) -> i32;

// qboolean from engine
#[allow(non_camel_case_types)]
pub type qboolean = c_int;
const qfalse: c_int = 0;
const qtrue: c_int = 1;

const COPY_ID: c_int = 1;
const QUIT_ID: c_int = 2;
const CLEAR_ID: c_int = 3;

const ERRORBOX_ID: c_int = 10;
const ERRORTEXT_ID: c_int = 11;

const EDIT_ID: c_int = 100;
const INPUT_ID: c_int = 101;

#[repr(C)]
#[allow(non_snake_case)]
struct WinConData {
	hWnd: HWND,
	hwndBuffer: HWND,

	hwndButtonClear: HWND,
	hwndButtonCopy: HWND,
	hwndButtonQuit: HWND,

	hwndErrorBox: HWND,
	hwndErrorText: HWND,

	hbmLogo: HBITMAP,
	hbmClearBitmap: HBITMAP,

	hbrEditBackground: HBRUSH,
	hbrErrorBackground: HBRUSH,

	hfBufferFont: HFONT,
	hfButtonFont: HFONT,

	hwndInputLine: HWND,

	errorString: [c_char; 80],

	consoleText: [c_char; 512],
	returnedText: [c_char; 512],
	visLevel: c_int,
	quitOnClose: qboolean,
	windowWidth: c_int,
	windowHeight: c_int,

	SysInputLineWndProc: WNDPROC,
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
	SysInputLineWndProc: dummy_wndproc,
};

extern "C" fn dummy_wndproc(_hWnd: HWND, _uMsg: UINT, _wParam: WPARAM, _lParam: LPARAM) -> i32 {
	0
}

// External C function declarations
extern "C" {
	fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
	fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
	fn strlen(s: *const c_char) -> usize;
	fn strncat(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
	fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;

	// Engine functions
	pub static mut com_viewlog: *mut crate::qcommon::cvar_t;
	fn Cvar_Set(name: *const c_char, value: *const c_char);
	fn Sys_QueEvent(time: c_int, event_type: c_int, arg0: c_int, arg1: c_int, arg_length: c_int, arg: *mut c_void);
	fn Sys_Print(msg: *const c_char);
	fn Sys_Error(msg: *const c_char, ...);
	fn CopyString(in_: *const c_char) -> *mut c_char;
	fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
	fn Q_IsColorString(p: *const c_char) -> c_int;
	fn va(fmt: *const c_char, ...) -> *const c_char;

	// Windows API
	fn SetFocus(hWnd: HWND) -> HWND;
	fn RegisterClass(lpWndClass: *const WNDCLASS) -> u16;
	fn CreateWindowEx(dwExStyle: u32, lpClassName: *const c_char, lpWindowName: *const c_char,
		dwStyle: u32, x: c_int, y: c_int, nWidth: c_int, nHeight: c_int,
		hWndParent: HWND, hMenu: *mut c_void, hInstance: HINSTANCE, lpParam: *mut c_void) -> HWND;
	fn CreateWindow(lpClassName: *const c_char, lpWindowName: *const c_char, dwStyle: u32,
		x: c_int, y: c_int, nWidth: c_int, nHeight: c_int,
		hWndParent: HWND, hMenu: *mut c_void, hInstance: HINSTANCE, lpParam: *mut c_void) -> HWND;
	fn GetDC(hWnd: HWND) -> HDC;
	fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
	fn GetDesktopWindow() -> HWND;
	fn GetDeviceCaps(hdc: HDC, nIndex: c_int) -> c_int;
	fn AdjustWindowRect(lpRect: *mut RECT, dwStyle: u32, bMenu: i32) -> i32;
	fn CreateFont(cHeight: c_int, cWidth: c_int, cEscapement: c_int, cOrientation: c_int,
		cWeight: c_int, bItalic: c_int, bUnderline: c_int, bStrikeOut: c_int,
		iCharSet: c_int, iOutPrecision: c_int, iClipPrecision: c_int, iQuality: c_int,
		iPitchAndFamily: c_int, pszFaceName: *const c_char) -> HFONT;
	fn CreateSolidBrush(crColor: u32) -> HBRUSH;
	fn SetTimer(hWnd: HWND, nIDEvent: usize, uElapse: c_int, lpTimerFunc: *mut c_void) -> usize;
	fn SendMessage(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> usize;
	fn DefWindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> isize;
	fn GetWindowText(hWnd: HWND, lpString: *mut c_char, nMaxCount: c_int) -> c_int;
	fn SetWindowText(hWnd: HWND, lpString: *const c_char) -> i32;
	fn UpdateWindow(hWnd: HWND) -> i32;
	fn InvalidateRect(hWnd: HWND, lpRect: *mut c_void, bErase: i32) -> i32;
	fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> i32;
	fn LoadIcon(hInstance: HINSTANCE, lpIconName: *const c_char) -> *mut c_void;
	fn LoadCursor(hInstance: HWND, lpCursorName: *const c_char) -> *mut c_void;
	fn SetForegroundWindow(hWnd: HWND) -> i32;
	fn CloseWindow(hWnd: HWND) -> i32;
	fn DestroyWindow(hWnd: HWND) -> i32;
	fn DeleteObject(hObject: *mut c_void) -> i32;
	fn CallWindowProc(lpPrevWndFunc: WNDPROC, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> isize;
	fn SetWindowLong(hWnd: HWND, nIndex: c_int, dwNewLong: i32) -> i32;
	fn PostQuitMessage(nExitCode: c_int);
	fn MulDiv(nNumber: c_int, nNumerator: c_int, nDenominator: c_int) -> c_int;
	fn SetBkColor(hdc: HDC, color: u32) -> u32;
	fn SetTextColor(hdc: HDC, color: u32) -> u32;
	fn RGB(r: c_int, g: c_int, b: c_int) -> u32;
	fn MAKEINTRESOURCE(i: usize) -> *const c_char;

	// Engine globals
	pub static mut g_wv: WinVars;
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct WinVars {
	hInstance: HINSTANCE,
	// other fields...
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct WNDCLASS {
	style: u32,
	lpfnWndProc: WNDPROC,
	cbClsExtra: c_int,
	cbWndExtra: c_int,
	hInstance: HINSTANCE,
	hIcon: *mut c_void,
	hCursor: *mut c_void,
	hbrBackground: *mut c_void,
	lpszMenuName: *const c_char,
	lpszClassName: *const c_char,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct RECT {
	left: c_int,
	top: c_int,
	right: c_int,
	bottom: c_int,
}

// Windows constants
const WM_ACTIVATE: UINT = 0x0006;
const WM_CLOSE: UINT = 0x0010;
const WM_CTLCOLORSTATIC: UINT = 0x0138;
const WM_COMMAND: UINT = 0x0111;
const WM_CREATE: UINT = 0x0001;
const WM_ERASEBKGND: UINT = 0x0014;
const WM_TIMER: UINT = 0x0113;
const WM_KILLFOCUS: UINT = 0x0008;
const WM_CHAR: UINT = 0x0102;
const WM_SETFONT: UINT = 0x0030;
const WM_SETTEXT: UINT = 0x000c;
const WM_COPY: UINT = 0x0301;

const WA_INACTIVE: WPARAM = 0;
const HIWORD_MASK: usize = 0xffff0000;
const LOWORD_MASK: usize = 0x0000ffff;

const WS_POPUPWINDOW: u32 = 0x80880000;
const WS_CAPTION: u32 = 0x00c00000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_BORDER: u32 = 0x00800000;
const WS_VSCROLL: u32 = 0x00200000;

const BS_PUSHBUTTON: u32 = 0;
const BS_DEFPUSHBUTTON: u32 = 1;

const SS_SUNKEN: u32 = 0x1000;

const ES_LEFT: u32 = 0;
const ES_AUTOHSCROLL: u32 = 0x0080;
const ES_MULTILINE: u32 = 0x0004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_READONLY: u32 = 0x0800;

const EM_SETSEL: UINT = 0x00b1;
const EM_REPLACESEL: UINT = 0x00c2;
const EM_LINESCROLL: UINT = 0x00b6;
const EM_SCROLLCARET: UINT = 0x00b9;
const EM_LIMITTEXT: UINT = 0x00c5;

const SW_SHOWDEFAULT: c_int = 10;
const SW_HIDE: c_int = 0;
const SW_SHOWNORMAL: c_int = 1;
const SW_MINIMIZE: c_int = 6;

const GWL_WNDPROC: c_int = -4;

const HORZRES: c_int = 8;
const VERTRES: c_int = 10;
const LOGPIXELSY: c_int = 90;

const DEFAULT_CHARSET: c_int = 1;
const OUT_DEFAULT_PRECIS: c_int = 0;
const CLIP_DEFAULT_PRECIS: c_int = 0;
const DEFAULT_QUALITY: c_int = 0;
const FF_MODERN: c_int = 48;
const FIXED_PITCH: c_int = 1;
const FW_LIGHT: c_int = 300;

const IDI_ICON1: usize = 101;
const IDC_ARROW: usize = 32512;
const COLOR_INACTIVEBORDER: usize = 2;

const FALSE: i32 = 0;
const TRUE: i32 = 1;

const SE_CONSOLE: c_int = 4;

fn LOWORD(l: WPARAM) -> WPARAM {
	l & LOWORD_MASK
}

fn HIWORD(l: WPARAM) -> WPARAM {
	(l & HIWORD_MASK) >> 16
}

extern "C" fn ConWndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> i32 {
	unsafe {
		let mut cmdString: *mut c_char;
		static mut s_timePolarity: qboolean = 0;

		match uMsg {
	WM_ACTIVATE => {
		if LOWORD(wParam) != WA_INACTIVE {
			SetFocus((*addr_of_mut!(s_wcd)).hwndInputLine);
		}

		if !addr_of_mut!(com_viewlog).is_null() {
			// if the viewlog is open, check to see if it's being minimized
			if (*addr_of_mut!(com_viewlog)).integer == 1 {
				if HIWORD(wParam) != 0 {		// minimized flag
					Cvar_Set(b"viewlog\0".as_ptr() as *const c_char, b"2\0".as_ptr() as *const c_char);
				}
			}
			else if (*addr_of_mut!(com_viewlog)).integer == 2 {
				if HIWORD(wParam) == 0 {		// minimized flag
					Cvar_Set(b"viewlog\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);
				}
			}
		}
		0
	}

	WM_CLOSE => {
		//cmdString = CopyString( "quit" );
		//Sys_QueEvent( 0, SE_CONSOLE, 0, 0, strlen( cmdString ) + 1, cmdString );
		if (*addr_of_mut!(s_wcd)).quitOnClose != 0 {
			PostQuitMessage(0);
		}
		else {
			Sys_ShowConsole(0, qfalse);
			Cvar_Set(b"viewlog\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
		}
		0
	}
	WM_CTLCOLORSTATIC => {
		if (lParam as HWND) == (*addr_of_mut!(s_wcd)).hwndBuffer {
			SetBkColor((wParam as HDC), RGB(0, 0, 0));
			SetTextColor((wParam as HDC), RGB(249, 249, 000));
			(*addr_of_mut!(s_wcd)).hbrEditBackground as i32
		}
		else if (lParam as HWND) == (*addr_of_mut!(s_wcd)).hwndErrorBox {
			if s_timePolarity & 1 != 0 {
				SetBkColor((wParam as HDC), RGB(0x80, 0x80, 0x80));
				SetTextColor((wParam as HDC), RGB(0xff, 0x00, 0x00));
			}
			else {
				SetBkColor((wParam as HDC), RGB(0x80, 0x80, 0x80));
				SetTextColor((wParam as HDC), RGB(0x00, 0x00, 0x00));
			}
			(*addr_of_mut!(s_wcd)).hbrErrorBackground as i32
		}
		else {
			FALSE
		}
	}

	WM_COMMAND => {
		if wParam == COPY_ID as usize {
			SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_SETSEL, 0, -1isize as usize);
			SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, WM_COPY, 0, 0);
		}
		else if wParam == QUIT_ID as usize {
			if (*addr_of_mut!(s_wcd)).quitOnClose != 0 {
				PostQuitMessage(0);
			}
			else {
				cmdString = CopyString(b"quit\0".as_ptr() as *const c_char);
				Sys_QueEvent(0, SE_CONSOLE, 0, 0, (strlen(cmdString) + 1) as c_int, cmdString as *mut c_void);
			}
		}
		else if wParam == CLEAR_ID as usize {
			SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_SETSEL, 0, -1isize as usize);
			SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_REPLACESEL, FALSE as usize, b"\0".as_ptr() as isize);
			UpdateWindow((*addr_of_mut!(s_wcd)).hwndBuffer);
		}
		0
	}
	WM_CREATE => {
		(*addr_of_mut!(s_wcd)).hbrEditBackground = CreateSolidBrush(RGB(0x00, 0x00, 0x00));
		(*addr_of_mut!(s_wcd)).hbrErrorBackground = CreateSolidBrush(RGB(0x80, 0x80, 0x80));
		SetTimer(hWnd, 1, 1000, ptr::null_mut());
		0
	}
	WM_ERASEBKGND => {
	    DefWindowProc(hWnd, uMsg, wParam, lParam) as i32
	}
	WM_TIMER => {
		if wParam == 1 {
			s_timePolarity = !s_timePolarity;
			if (*addr_of_mut!(s_wcd)).hwndErrorBox != ptr::null_mut() {
				InvalidateRect((*addr_of_mut!(s_wcd)).hwndErrorBox, ptr::null_mut(), FALSE);
			}
		}
		0
	}
    _ => {
        DefWindowProc(hWnd, uMsg, wParam, lParam) as i32
    }
		}
	}
}

extern "C" fn InputLineWndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> i32 {
	unsafe {
		let mut inputBuffer: [c_char; 1024] = [0; 1024];

		match uMsg {
		WM_KILLFOCUS => {
			if (wParam as HWND) == (*addr_of_mut!(s_wcd)).hWnd ||
				 (wParam as HWND) == (*addr_of_mut!(s_wcd)).hwndErrorBox {
				SetFocus(hWnd);
				return 0;
			}
			0
		}

		WM_CHAR => {
			if wParam == 13 {
				GetWindowText((*addr_of_mut!(s_wcd)).hwndInputLine, inputBuffer.as_mut_ptr(), 1024);
				strncat((*addr_of_mut!(s_wcd)).consoleText.as_mut_ptr(), inputBuffer.as_ptr(),
					((*addr_of_mut!(s_wcd)).consoleText.len() - strlen((*addr_of_mut!(s_wcd)).consoleText.as_ptr()) - 5) as usize);
				strcat((*addr_of_mut!(s_wcd)).consoleText.as_mut_ptr(), b"\n\0".as_ptr() as *const c_char);
				SetWindowText((*addr_of_mut!(s_wcd)).hwndInputLine, b"\0".as_ptr() as *const c_char);

				Sys_Print(va(b"]%s\n\0".as_ptr() as *const c_char, inputBuffer.as_ptr()));

				return 0;
			}
			0
		}
	    _ => {
	        CallWindowProc((*addr_of_mut!(s_wcd)).SysInputLineWndProc, hWnd, uMsg, wParam, lParam) as i32
	    }
		}
	}
}

/*
** Sys_CreateConsole
*/
pub unsafe fn Sys_CreateConsole() {
	let mut hDC: HDC;
	let mut wc: WNDCLASS;
	let mut rect: RECT;
	let DEDCLASS: *const c_char = b"JK2MP WinConsole\0".as_ptr() as *const c_char;
	let mut nHeight: c_int;
	let mut swidth: c_int;
	let mut sheight: c_int;
	let DEDSTYLE: u32 = WS_POPUPWINDOW | WS_CAPTION | WS_MINIMIZEBOX;

	memset(addr_of_mut!(wc) as *mut c_void, 0, mem::size_of::<WNDCLASS>());

	wc.style         = 0;
	wc.lpfnWndProc   = ConWndProc;
	wc.cbClsExtra    = 0;
	wc.cbWndExtra    = 0;
	wc.hInstance     = (*addr_of!(g_wv)).hInstance;
	wc.hIcon         = LoadIcon((*addr_of!(g_wv)).hInstance, MAKEINTRESOURCE(IDI_ICON1));
	wc.hCursor       = LoadCursor(ptr::null_mut(), MAKEINTRESOURCE(IDC_ARROW));
	wc.hbrBackground = COLOR_INACTIVEBORDER as *mut c_void;//(HBRUSH__ *)COLOR_WINDOW;
	wc.lpszMenuName  = ptr::null();
	wc.lpszClassName = DEDCLASS;

	if RegisterClass(addr_of!(wc)) == 0 {
		return;
	}

	rect.left = 0;
	rect.right = 600;
	rect.top = 0;
	rect.bottom = 450;
	AdjustWindowRect(addr_of_mut!(rect), DEDSTYLE, FALSE);

	hDC = GetDC(GetDesktopWindow());
	swidth = GetDeviceCaps(hDC, HORZRES);
	sheight = GetDeviceCaps(hDC, VERTRES);
	ReleaseDC(GetDesktopWindow(), hDC);

	(*addr_of_mut!(s_wcd)).windowWidth = rect.right - rect.left + 1;
	(*addr_of_mut!(s_wcd)).windowHeight = rect.bottom - rect.top + 1;

	(*addr_of_mut!(s_wcd)).hWnd = CreateWindowEx(0,
							   DEDCLASS,
							   b"Jedi Knight\xc2\xa0: Jedi Academy SP Console\0".as_ptr() as *const c_char,
							   DEDSTYLE,
							   (swidth - 600) / 2, (sheight - 450) / 2 , rect.right - rect.left + 1, rect.bottom - rect.top + 1,
							   ptr::null_mut(),
							   ptr::null_mut(),
							   (*addr_of!(g_wv)).hInstance,
							   ptr::null_mut());

	if (*addr_of_mut!(s_wcd)).hWnd == ptr::null_mut() {
		return;
	}

	//
	// create fonts
	//
	hDC = GetDC((*addr_of_mut!(s_wcd)).hWnd);
	nHeight = -MulDiv(8, GetDeviceCaps(hDC, LOGPIXELSY), 72);

	(*addr_of_mut!(s_wcd)).hfBufferFont = CreateFont(nHeight,
									  0,
									  0,
									  0,
									  FW_LIGHT,
									  0,
									  0,
									  0,
									  DEFAULT_CHARSET,
									  OUT_DEFAULT_PRECIS,
									  CLIP_DEFAULT_PRECIS,
									  DEFAULT_QUALITY,
									  FF_MODERN | FIXED_PITCH,
									  b"Courier New\0".as_ptr() as *const c_char);

	ReleaseDC((*addr_of_mut!(s_wcd)).hWnd, hDC);

	//
	// create the input line
	//
	(*addr_of_mut!(s_wcd)).hwndInputLine = CreateWindow(b"edit\0".as_ptr() as *const c_char, ptr::null(), WS_CHILD | WS_VISIBLE | WS_BORDER |
												ES_LEFT | ES_AUTOHSCROLL,
												6, 400, (*addr_of_mut!(s_wcd)).windowWidth-20, 20,
												(*addr_of_mut!(s_wcd)).hWnd,
												INPUT_ID as *mut c_void,	// child window ID
												(*addr_of!(g_wv)).hInstance, ptr::null_mut());

	//
	// create the buttons
	//
	(*addr_of_mut!(s_wcd)).hwndButtonCopy = CreateWindow(b"button\0".as_ptr() as *const c_char, ptr::null(), BS_PUSHBUTTON | WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON,
												5, 425, 72, 24,
												(*addr_of_mut!(s_wcd)).hWnd,
												COPY_ID as *mut c_void,	// child window ID
												(*addr_of!(g_wv)).hInstance, ptr::null_mut());
	SendMessage((*addr_of_mut!(s_wcd)).hwndButtonCopy, WM_SETTEXT, 0, b"copy\0".as_ptr() as isize);

	(*addr_of_mut!(s_wcd)).hwndButtonClear = CreateWindow(b"button\0".as_ptr() as *const c_char, ptr::null(), BS_PUSHBUTTON | WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON,
												82, 425, 72, 24,
												(*addr_of_mut!(s_wcd)).hWnd,
												CLEAR_ID as *mut c_void,	// child window ID
												(*addr_of!(g_wv)).hInstance, ptr::null_mut());
	SendMessage((*addr_of_mut!(s_wcd)).hwndButtonClear, WM_SETTEXT, 0, b"clear\0".as_ptr() as isize);

	(*addr_of_mut!(s_wcd)).hwndButtonQuit = CreateWindow(b"button\0".as_ptr() as *const c_char, ptr::null(), BS_PUSHBUTTON | WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON,
												(*addr_of_mut!(s_wcd)).windowWidth-92, 425, 72, 24,
												(*addr_of_mut!(s_wcd)).hWnd,
												QUIT_ID as *mut c_void,	// child window ID
												(*addr_of!(g_wv)).hInstance, ptr::null_mut());
	SendMessage((*addr_of_mut!(s_wcd)).hwndButtonQuit, WM_SETTEXT, 0, b"quit\0".as_ptr() as isize);


	//
	// create the scrollbuffer
	//
	(*addr_of_mut!(s_wcd)).hwndBuffer = CreateWindow(b"edit\0".as_ptr() as *const c_char, ptr::null(), WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER |
												ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
												6, 40, (*addr_of_mut!(s_wcd)).windowWidth-20, 354,
												(*addr_of_mut!(s_wcd)).hWnd,
												EDIT_ID as *mut c_void,	// child window ID
												(*addr_of!(g_wv)).hInstance, ptr::null_mut());
	SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, WM_SETFONT, (*addr_of_mut!(s_wcd)).hfBufferFont as usize, 0);

	(*addr_of_mut!(s_wcd)).SysInputLineWndProc = mem::transmute::<i32, WNDPROC>(
		SetWindowLong((*addr_of_mut!(s_wcd)).hwndInputLine, GWL_WNDPROC, mem::transmute::<WNDPROC, i32>(InputLineWndProc))
	);
	SendMessage((*addr_of_mut!(s_wcd)).hwndInputLine, WM_SETFONT, (*addr_of_mut!(s_wcd)).hfBufferFont as usize, 0);
	SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_LIMITTEXT, 0x7fff as usize, 0);

	ShowWindow((*addr_of_mut!(s_wcd)).hWnd, SW_SHOWDEFAULT);
	UpdateWindow((*addr_of_mut!(s_wcd)).hWnd);
	SetForegroundWindow((*addr_of_mut!(s_wcd)).hWnd);
	SetFocus((*addr_of_mut!(s_wcd)).hwndInputLine);

	(*addr_of_mut!(s_wcd)).visLevel = 1;
}

/*
** Sys_DestroyConsole
*/
pub unsafe fn Sys_DestroyConsole() {
	if (*addr_of_mut!(s_wcd)).hWnd != ptr::null_mut() {
		DeleteObject((*addr_of_mut!(s_wcd)).hbrEditBackground);
		DeleteObject((*addr_of_mut!(s_wcd)).hbrErrorBackground);
		DeleteObject((*addr_of_mut!(s_wcd)).hfBufferFont);
		ShowWindow((*addr_of_mut!(s_wcd)).hWnd, SW_HIDE);
		CloseWindow((*addr_of_mut!(s_wcd)).hWnd);
		DestroyWindow((*addr_of_mut!(s_wcd)).hWnd);
		(*addr_of_mut!(s_wcd)).hWnd = ptr::null_mut();
	}
}

/*
** Sys_ShowConsole
*/
pub unsafe fn Sys_ShowConsole(visLevel: c_int, quitOnClose: qboolean) {
	(*addr_of_mut!(s_wcd)).quitOnClose = quitOnClose;

	if visLevel == (*addr_of_mut!(s_wcd)).visLevel {
		if quitOnClose != 0 {//attempt to bring it to the front on error exit
			SetForegroundWindow((*addr_of_mut!(s_wcd)).hWnd);
			SetFocus((*addr_of_mut!(s_wcd)).hwndInputLine);
		}
		return;
	}

	(*addr_of_mut!(s_wcd)).visLevel = visLevel;

	if (*addr_of_mut!(s_wcd)).hWnd == ptr::null_mut() {
		return;
	}

	match visLevel {
	0 => {
		ShowWindow((*addr_of_mut!(s_wcd)).hWnd, SW_HIDE);
	}
	1 => {
		ShowWindow((*addr_of_mut!(s_wcd)).hWnd, SW_SHOWNORMAL);
		SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_LINESCROLL, 0, 0xffff as usize);
		if quitOnClose != 0 {//attempt to bring it to the front on error exit
			SetForegroundWindow((*addr_of_mut!(s_wcd)).hWnd);
			SetFocus((*addr_of_mut!(s_wcd)).hwndInputLine);
		}
	}
	2 => {
		ShowWindow((*addr_of_mut!(s_wcd)).hWnd, SW_MINIMIZE);
	}
	_ => {
		Sys_Error(b"Invalid visLevel %d sent to Sys_ShowConsole\n\0".as_ptr() as *const c_char, visLevel);
	}
	}
}

/*
** Sys_ConsoleInput
*/
pub unsafe fn Sys_ConsoleInput() -> *mut c_char {
	if (*addr_of_mut!(s_wcd)).consoleText[0] == 0 {
		return ptr::null_mut();
	}

	strcpy((*addr_of_mut!(s_wcd)).returnedText.as_mut_ptr(), (*addr_of_mut!(s_wcd)).consoleText.as_ptr());
	(*addr_of_mut!(s_wcd)).consoleText[0] = 0;

	return (*addr_of_mut!(s_wcd)).returnedText.as_mut_ptr();
}

/*
** Conbuf_AppendText
*/
pub unsafe fn Conbuf_AppendText(pMsg: *const c_char) {
	const CONSOLE_BUFFER_SIZE: usize = 16384;
	if (*addr_of_mut!(s_wcd)).hWnd == ptr::null_mut() {
		return;
	}
	let mut buffer: [c_char; CONSOLE_BUFFER_SIZE*4] = [0; CONSOLE_BUFFER_SIZE*4];
	let mut b: *mut c_char = buffer.as_mut_ptr();
	let mut msg: *const c_char;
	let mut bufLen: c_int;
	let mut i: c_int = 0;
	static mut s_totalChars: u32 = 0;

	//
	// if the message is REALLY long, use just the last portion of it
	//
	if strlen(pMsg) > CONSOLE_BUFFER_SIZE - 1 {
		msg = pMsg.add(strlen(pMsg) - CONSOLE_BUFFER_SIZE + 1);
	}
	else {
		msg = pMsg;
	}

	//
	// copy into an intermediate buffer
	//
	while *msg.add(i as usize) != 0 && ((b as usize - buffer.as_ptr() as usize) < buffer.len() - 1) {
		if *msg.add(i as usize) == b'\n' as c_char && *msg.add((i+1) as usize) == b'\r' as c_char {
			*b = b'\r' as c_char;
			*b.add(1) = b'\n' as c_char;
			b = b.add(2);
			i += 1;
		}
		else if *msg.add(i as usize) == b'\r' as c_char {
			*b = b'\r' as c_char;
			*b.add(1) = b'\n' as c_char;
			b = b.add(2);
		}
		else if *msg.add(i as usize) == b'\n' as c_char {
			*b = b'\r' as c_char;
			*b.add(1) = b'\n' as c_char;
			b = b.add(2);
		}
		else if Q_IsColorString(msg.add(i as usize)) != 0 {
			i += 1;
		}
		else {
			*b = *msg.add(i as usize);
			b = b.add(1);
		}
		i += 1;
	}
	*b = 0;
	bufLen = (b as usize - buffer.as_ptr() as usize) as c_int;

	s_totalChars = (s_totalChars as c_int + bufLen) as u32;

	//
	// replace selection instead of appending if we're overflowing
	//
	if s_totalChars > 0x7fff {
		SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_SETSEL, 0, -1isize as usize);
		s_totalChars = bufLen as u32;
	}

	//
	// put this text into the windows console
	//
	SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_LINESCROLL, 0, 0xffff as usize);
	SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_SCROLLCARET, 0, 0);
	SendMessage((*addr_of_mut!(s_wcd)).hwndBuffer, EM_REPLACESEL, 0, buffer.as_ptr() as isize);
}

/*
** Sys_SetErrorText
*/
pub unsafe fn Sys_SetErrorText(buf: *const c_char) {
	Q_strncpyz((*addr_of_mut!(s_wcd)).errorString.as_mut_ptr(), buf, (*addr_of_mut!(s_wcd)).errorString.len());

	if (*addr_of_mut!(s_wcd)).hwndErrorBox == ptr::null_mut() {
		(*addr_of_mut!(s_wcd)).hwndErrorBox = CreateWindow(b"static\0".as_ptr() as *const c_char, ptr::null(), WS_CHILD | WS_VISIBLE | SS_SUNKEN,
													6, 5, (*addr_of_mut!(s_wcd)).windowWidth-20, 30,
													(*addr_of_mut!(s_wcd)).hWnd,
													ERRORBOX_ID as *mut c_void,	// child window ID
													(*addr_of!(g_wv)).hInstance, ptr::null_mut());
		SendMessage((*addr_of_mut!(s_wcd)).hwndErrorBox, WM_SETFONT, (*addr_of_mut!(s_wcd)).hfBufferFont as usize, 0);
		SetWindowText((*addr_of_mut!(s_wcd)).hwndErrorBox, (*addr_of_mut!(s_wcd)).errorString.as_ptr());

		DestroyWindow((*addr_of_mut!(s_wcd)).hwndInputLine);
		(*addr_of_mut!(s_wcd)).hwndInputLine = ptr::null_mut();
	}
}
