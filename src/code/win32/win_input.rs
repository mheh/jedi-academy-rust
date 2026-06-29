// win_input.c -- win32 mouse and joystick code
// 02/21/97 JCB Added extended DirectInput code to support external controllers.

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"
// #include "../client/client.h"
// #ifndef _IMMERSION
// #include "../client/fffx.h"
// #endif // _IMMERSION
// #include "win_local.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint};
use core::ptr::{addr_of, addr_of_mut, null_mut};

// Type aliases for qboolean and Windows types
type qboolean = c_int;
type LONG = c_int;
type UINT = c_uint;
type DWORD = u32;
type BYTE = u8;
type HRESULT = c_int;
type HINSTANCE = *mut c_void;
type HWND = *mut c_void;
type HMIDIIN = *mut c_void;

const qfalse: c_int = 0;
const qtrue: c_int = 1;

const VER_PLATFORM_WIN32_NT: c_int = 2;

// Key codes from keycodes.h
const A_MOUSE1: c_int = 152;
const A_MOUSE2: c_int = 153;
const A_MOUSE3: c_int = 171;
const A_MOUSE4: c_int = 172;
const A_MOUSE5: c_int = 173;
const A_CURSOR_UP: c_int = 175;
const A_CURSOR_DOWN: c_int = 176;
const A_CURSOR_LEFT: c_int = 177;
const A_CURSOR_RIGHT: c_int = 178;
const A_JOY0: c_int = 264;
const A_JOY16: c_int = 280;
const A_JOY17: c_int = 281;
const A_JOY18: c_int = 282;
const A_JOY19: c_int = 283;
const A_JOY20: c_int = 284;
const A_JOY21: c_int = 285;
const A_JOY22: c_int = 286;
const A_JOY23: c_int = 287;
const A_JOY24: c_int = 288;
const A_JOY25: c_int = 289;
const A_JOY26: c_int = 290;
const A_JOY27: c_int = 291;
const A_AUX0: c_int = 305;

// Windows API type definitions
#[repr(C)]
pub struct POINT {
	pub x: LONG,
	pub y: LONG,
}

#[repr(C)]
pub struct RECT {
	pub left: LONG,
	pub top: LONG,
	pub right: LONG,
	pub bottom: LONG,
}

#[repr(C)]
pub struct GUID {
	pub Data1: u32,
	pub Data2: u16,
	pub Data3: u16,
	pub Data4: [BYTE; 8],
}

// DirectInput structures and types
#[repr(C)]
pub struct DIOBJECTDATAFORMAT {
	pub pguid: *const GUID,
	pub dwOfs: DWORD,
	pub dwType: DWORD,
	pub dwFlags: DWORD,
}

#[repr(C)]
pub struct DIDATAFORMAT {
	pub dwSize: DWORD,
	pub dwObjSize: DWORD,
	pub dwFlags: DWORD,
	pub dwDataSize: DWORD,
	pub dwNumObjs: DWORD,
	pub rgodf: *mut DIOBJECTDATAFORMAT,
}

#[repr(C)]
pub struct DIDEVICEOBJECTDATA {
	pub dwOfs: DWORD,
	pub dwData: DWORD,
	pub dwTimeStamp: DWORD,
	pub dwSequence: DWORD,
}

#[repr(C)]
pub struct DIMOUSESTATE {
	pub lX: LONG,
	pub lY: LONG,
	pub lZ: LONG,
}

#[repr(C)]
pub struct DIPROPHEADER {
	pub dwSize: DWORD,
	pub dwHeaderSize: DWORD,
	pub dwObj: DWORD,
	pub dwHow: DWORD,
}

#[repr(C)]
pub struct DIPROPDWORD {
	pub diph: DIPROPHEADER,
	pub dwData: DWORD,
}

#[repr(C)]
pub struct MIDIINCAPS {
	pub wMid: u16,
	pub wPid: u16,
	pub vDriverVersion: u32,
	pub szPname: [c_char; 32],
	pub dwSupport: DWORD,
}

#[repr(C)]
pub struct JOYCAPS {
	pub wMid: u16,
	pub wPid: u16,
	pub szPname: [c_char; 32],
	pub wXmin: u32,
	pub wXmax: u32,
	pub wYmin: u32,
	pub wYmax: u32,
	pub wZmin: u32,
	pub wZmax: u32,
	pub wNumButtons: u32,
	pub wPeriodMin: u32,
	pub wPeriodMax: u32,
	pub wRmin: u32,
	pub wRmax: u32,
	pub wUmin: u32,
	pub wUmax: u32,
	pub wVmin: u32,
	pub wVmax: u32,
	pub wCaps: u32,
	pub wMaxAxes: u32,
	pub wNumAxes: u32,
	pub wMaxButtons: u32,
	pub szRegKey: [c_char; 260],
	pub szOEMVxD: [c_char; 260],
}

#[repr(C)]
pub struct JOYINFOEX {
	pub dwSize: u32,
	pub dwFlags: u32,
	pub dwXpos: u32,
	pub dwYpos: u32,
	pub dwZpos: u32,
	pub dwRpos: u32,
	pub dwUpos: u32,
	pub dwVpos: u32,
	pub dwButtons: u32,
	pub dwButtonNumber: u32,
	pub dwPOV: u32,
	pub dwReserved1: u32,
	pub dwReserved2: u32,
}

#[repr(C)]
struct WinMouseVars_t {
	oldButtonState: c_int,
	mouseActive: qboolean,
	mouseInitialized: qboolean,
}

static mut s_wmv: WinMouseVars_t = WinMouseVars_t {
	oldButtonState: 0,
	mouseActive: qfalse,
	mouseInitialized: qfalse,
};

static mut window_center_x: c_int = 0;
static mut window_center_y: c_int = 0;

//
// MIDI definitions
//
const MAX_MIDIIN_DEVICES: usize = 8;

#[repr(C)]
struct MidiInfo_t {
	numDevices: c_int,
	caps: [MIDIINCAPS; MAX_MIDIIN_DEVICES],
	hMidiIn: HMIDIIN,
}

static mut s_midiInfo: MidiInfo_t = MidiInfo_t {
	numDevices: 0,
	caps: [
		MIDIINCAPS {
			wMid: 0, wPid: 0, vDriverVersion: 0, szPname: [0; 32], dwSupport: 0
		}; MAX_MIDIIN_DEVICES
	],
	hMidiIn: null_mut(),
};

//
// Joystick definitions
//
const JOY_MAX_AXES: usize = 6; // X, Y, Z, R, U, V

#[repr(C)]
struct joystickInfo_t {
	avail: qboolean,
	id: c_int, // joystick number
	jc: JOYCAPS,
	oldbuttonstate: c_int,
	oldpovstate: c_int,
	ji: JOYINFOEX,
}

static mut joy: joystickInfo_t = joystickInfo_t {
	avail: qfalse,
	id: 0,
	jc: JOYCAPS {
		wMid: 0, wPid: 0, szPname: [0; 32], wXmin: 0, wXmax: 0, wYmin: 0, wYmax: 0,
		wZmin: 0, wZmax: 0, wNumButtons: 0, wPeriodMin: 0, wPeriodMax: 0,
		wRmin: 0, wRmax: 0, wUmin: 0, wUmax: 0, wVmin: 0, wVmax: 0, wCaps: 0,
		wMaxAxes: 0, wNumAxes: 0, wMaxButtons: 0, szRegKey: [0; 260], szOEMVxD: [0; 260],
	},
	oldbuttonstate: 0,
	oldpovstate: 0,
	ji: JOYINFOEX {
		dwSize: 0, dwFlags: 0, dwXpos: 0, dwYpos: 0, dwZpos: 0, dwRpos: 0,
		dwUpos: 0, dwVpos: 0, dwButtons: 0, dwButtonNumber: 0, dwPOV: 0,
		dwReserved1: 0, dwReserved2: 0,
	},
};

// cvar_t from engine
#[repr(C)]
pub struct cvar_t {
	pub integer: c_int,
	pub value: f32,
	pub modified: qboolean,
}

static mut in_midi: *mut cvar_t = null_mut();
static mut in_midiport: *mut cvar_t = null_mut();
static mut in_midichannel: *mut cvar_t = null_mut();
static mut in_mididevice: *mut cvar_t = null_mut();

static mut in_mouse: *mut cvar_t = null_mut();
static mut in_joystick: *mut cvar_t = null_mut();
static mut in_joyBallScale: *mut cvar_t = null_mut();
static mut in_debugJoystick: *mut cvar_t = null_mut();
static mut joy_threshold: *mut cvar_t = null_mut();
static mut js_ffmult: *mut cvar_t = null_mut();
static mut joy_xbutton: *mut cvar_t = null_mut();
static mut joy_ybutton: *mut cvar_t = null_mut();

static mut in_appactive: qboolean = qfalse;

// forward-referenced functions
extern "C" {
	pub fn IN_StartupJoystick();
	pub fn IN_JoyMove();
}

extern "C" {
	// Console/UI functions
	fn Com_Printf(fmt: *const c_char, ...);
	fn Cvar_Get(
		var_name: *const c_char,
		var_value: *const c_char,
		flags: c_int,
	) -> *mut cvar_t;
	fn Cvar_Set(var_name: *const c_char, value: *const c_char);
	fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;
	fn Cmd_AddCommand(cmd_name: *const c_char, function: *const c_void);
	fn Cmd_RemoveCommand(cmd_name: *const c_char);
	fn Sys_QueEvent(
		time: c_int,
		type_: c_int,
		value: c_int,
		value2: c_int,
		ptrLength: c_int,
		ptr: *mut c_void,
	);

	// Windows API functions
	fn GetSystemMetrics(nIndex: c_int) -> c_int;
	fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> c_int;
	fn SetCursorPos(X: c_int, Y: c_int) -> c_int;
	fn SetCapture(hWnd: HWND) -> HWND;
	fn ClipCursor(lpRect: *const RECT) -> c_int;
	fn ShowCursor(bShow: c_int) -> c_int;
	fn ReleaseCapture() -> c_int;
	fn GetCursorPos(lpPoint: *mut POINT) -> c_int;
	fn LoadLibrary(lpLibFileName: *const c_char) -> HINSTANCE;
	fn GetProcAddress(hModule: HINSTANCE, lpProcName: *const c_char) -> *mut c_void;
	fn FreeLibrary(hLibModule: HINSTANCE) -> c_int;
	fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

	// Global variables
	pub static mut g_wv: WinVars_t;
	pub static mut cls: ClientStatic_t;
}

#[repr(C)]
pub struct WinVars_t {
	pub hWnd: HWND,
	pub hInstance: HINSTANCE,
	pub activeApp: c_int,
	pub isMinimized: c_int,
	pub osversion: OSVERSIONINFO,
	pub sysMsgTime: c_uint,
}

#[repr(C)]
pub struct OSVERSIONINFO {
	pub dwOSVersionInfoSize: c_int,
	pub dwMajorVersion: c_int,
	pub dwMinorVersion: c_int,
	pub dwBuildNumber: c_int,
	pub dwPlatformId: c_int,
	pub szCSDVersion: [c_char; 128],
}

#[repr(C)]
pub struct ClientStatic_t {
	pub keyCatchers: c_int,
}

/*
============================================================

WIN32 MOUSE CONTROL

============================================================
*/

/*
================
IN_InitWin32Mouse
================
*/
extern "C" fn IN_InitWin32Mouse() {
}

/*
================
IN_ShutdownWin32Mouse
================
*/
extern "C" fn IN_ShutdownWin32Mouse() {
}

/*
================
IN_ActivateWin32Mouse
================
*/
extern "C" fn IN_ActivateWin32Mouse() {
	let mut width: c_int;
	let mut height: c_int;
	let mut window_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

	unsafe {
		width = GetSystemMetrics(0); // SM_CXSCREEN = 0
		height = GetSystemMetrics(1); // SM_CYSCREEN = 1

		GetWindowRect(g_wv.hWnd, &mut window_rect);
		if window_rect.left < 0 {
			window_rect.left = 0;
		}
		if window_rect.top < 0 {
			window_rect.top = 0;
		}
		if window_rect.right >= width {
			window_rect.right = width - 1;
		}
		if window_rect.bottom >= height - 1 {
			window_rect.bottom = height - 1;
		}
		window_center_x = (window_rect.right + window_rect.left) / 2;
		window_center_y = (window_rect.top + window_rect.bottom) / 2;

		SetCursorPos(window_center_x, window_center_y);

		SetCapture(g_wv.hWnd);
		ClipCursor(&window_rect);
		loop {
			if ShowCursor(0) < 0 {
				break;
			}
		}
	}
}

/*
================
IN_DeactivateWin32Mouse
================
*/
extern "C" fn IN_DeactivateWin32Mouse() {
	unsafe {
		ClipCursor(null_mut());
		ReleaseCapture();
		loop {
			if ShowCursor(1) >= 0 {
				break;
			}
		}
	}
}

/*
================
IN_Win32Mouse
================
*/
extern "C" fn IN_Win32Mouse(mx: *mut c_int, my: *mut c_int) {
	let mut current_pos: POINT = POINT { x: 0, y: 0 };

	unsafe {
		// find mouse movement
		GetCursorPos(&mut current_pos);

		// force the mouse to the center, so there's room to move
		SetCursorPos(window_center_x, window_center_y);

		*mx = current_pos.x - window_center_x;
		*my = current_pos.y - window_center_y;
	}
}


/*
============================================================

DIRECT INPUT MOUSE CONTROL

============================================================
*/

#[allow(non_upper_case_globals)]
const DEFINE_GUID_qGUID_SysMouse: GUID = GUID {
	Data1: 0x6F1D2B60,
	Data2: 0xD5A0,
	Data3: 0x11CF,
	Data4: [0xBF, 0xC7, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
const DEFINE_GUID_qGUID_XAxis: GUID = GUID {
	Data1: 0xA36D02E0,
	Data2: 0xC9F3,
	Data3: 0x11CF,
	Data4: [0xBF, 0xC7, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
const DEFINE_GUID_qGUID_YAxis: GUID = GUID {
	Data1: 0xA36D02E1,
	Data2: 0xC9F3,
	Data3: 0x11CF,
	Data4: [0xBF, 0xC7, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
const DEFINE_GUID_qGUID_ZAxis: GUID = GUID {
	Data1: 0xA36D02E2,
	Data2: 0xC9F3,
	Data3: 0x11CF,
	Data4: [0xBF, 0xC7, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00],
};

const DINPUT_BUFFERSIZE: DWORD = 16;

type pDirectInputCreate_fn = unsafe extern "C" fn(
	HINSTANCE,
	DWORD,
	*mut *mut c_void,
	*mut c_void,
) -> HRESULT;

static mut pDirectInputCreate: Option<pDirectInputCreate_fn> = None;

#[repr(C)]
struct MYDATA {
	lX: LONG,				// X axis goes here
	lY: LONG,				// Y axis goes here
	lZ: LONG,				// Z axis goes here
	bButtonA: BYTE,			// One button goes here
	bButtonB: BYTE,			// Another button goes here
	bButtonC: BYTE,			// Another button goes here
	bButtonD: BYTE,			// Another button goes here
}

static mut rgodf: [DIOBJECTDATAFORMAT; 7] = [
	DIOBJECTDATAFORMAT { pguid: &DEFINE_GUID_qGUID_XAxis, dwOfs: 0, dwType: 0x00000003 | 0x00FF0000, dwFlags: 0 },
	DIOBJECTDATAFORMAT { pguid: &DEFINE_GUID_qGUID_YAxis, dwOfs: 4, dwType: 0x00000003 | 0x00FF0000, dwFlags: 0 },
	DIOBJECTDATAFORMAT { pguid: &DEFINE_GUID_qGUID_ZAxis, dwOfs: 8, dwType: 0x80000000 | 0x00000003 | 0x00FF0000, dwFlags: 0 },
	DIOBJECTDATAFORMAT { pguid: null_mut(), dwOfs: 12, dwType: 0x00000004 | 0x00FF0000, dwFlags: 0 },
	DIOBJECTDATAFORMAT { pguid: null_mut(), dwOfs: 13, dwType: 0x00000004 | 0x00FF0000, dwFlags: 0 },
	DIOBJECTDATAFORMAT { pguid: null_mut(), dwOfs: 14, dwType: 0x80000000 | 0x00000004 | 0x00FF0000, dwFlags: 0 },
	DIOBJECTDATAFORMAT { pguid: null_mut(), dwOfs: 15, dwType: 0x80000000 | 0x00000004 | 0x00FF0000, dwFlags: 0 },
];

const NUM_OBJECTS: usize = 7;

static mut df: DIDATAFORMAT = DIDATAFORMAT {
	dwSize: 20,                 // sizeof(DIDATAFORMAT)
	dwObjSize: 16,              // sizeof(DIOBJECTDATAFORMAT)
	dwFlags: 0x00000001,        // DIDF_RELAXIS
	dwDataSize: 16,             // sizeof(MYDATA)
	dwNumObjs: 7,               // NUM_OBJECTS
	rgodf: unsafe { &mut rgodf as *mut _ },
};

static mut g_pdi: *mut c_void = null_mut();
static mut g_pMouse: *mut c_void = null_mut();
static mut hInstDI: HINSTANCE = null_mut();

/*
========================
IN_InitDIMouse
========================
*/
extern "C" fn IN_InitDIMouse() -> qboolean {
	let mut hr: HRESULT;
	let mut x: c_int = 0;
	let mut y: c_int = 0;
	let mut dipdw: DIPROPDWORD = DIPROPDWORD {
		diph: DIPROPHEADER {
			dwSize: 8,          // sizeof(DIPROPDWORD)
			dwHeaderSize: 4,    // sizeof(DIPROPHEADER)
			dwObj: 0,           // diph.dwObj
			dwHow: 1,           // DIPH_DEVICE
		},
		dwData: DINPUT_BUFFERSIZE, // dwData
	};

	unsafe {
		Com_Printf(b"Initializing DirectInput...\n\0".as_ptr() as *const c_char);

		if hInstDI == null_mut() {
			hInstDI = LoadLibrary(b"dinput.dll\0".as_ptr() as *const c_char);

			if hInstDI == null_mut() {
				Com_Printf(b"Couldn't load dinput.dll\n\0".as_ptr() as *const c_char);
				return qfalse;
			}
		}

		if pDirectInputCreate.is_none() {
			let proc_addr = GetProcAddress(hInstDI, b"DirectInputCreateA\0".as_ptr() as *const c_char);
			if proc_addr != null_mut() {
				pDirectInputCreate = Some(core::mem::transmute(proc_addr));
			}

			if pDirectInputCreate.is_none() {
				Com_Printf(b"Couldn't get DI proc addr\n\0".as_ptr() as *const c_char);
				return qfalse;
			}
		}

		// register with DirectInput and get an IDirectInput to play with.
		let iDirectInputCreate = pDirectInputCreate.unwrap();
		hr = iDirectInputCreate(g_wv.hInstance, 0x0800, &mut g_pdi, null_mut());

		if hr != 0 {
			Com_Printf(b"iDirectInputCreate failed\n\0".as_ptr() as *const c_char);
			return qfalse;
		}

		// obtain an interface to the system mouse device.
		// We call g_pdi->CreateDevice manually using offset-based vtable lookup
		let di_vtable = *(g_pdi as *const *const *const c_void);
		let create_device: extern "C" fn(*mut c_void, *const GUID, *mut *mut c_void, *mut c_void) -> HRESULT = core::mem::transmute(*di_vtable.add(3));
		hr = create_device(g_pdi, &DEFINE_GUID_qGUID_SysMouse, &mut g_pMouse, null_mut());

		if hr != 0 {
			Com_Printf(b"Couldn't open DI mouse device\n\0".as_ptr() as *const c_char);
			return qfalse;
		}

		// set the data format to "mouse format".
		let set_data_format: extern "C" fn(*mut c_void, *const DIDATAFORMAT) -> HRESULT = core::mem::transmute(*di_vtable.add(4));
		hr = set_data_format(g_pMouse, &df);

		if hr != 0 {
			Com_Printf(b"Couldn't set DI mouse format\n\0".as_ptr() as *const c_char);
			return qfalse;
		}

		// set the cooperativity level.
		let set_coop_level: extern "C" fn(*mut c_void, HWND, DWORD) -> HRESULT = core::mem::transmute(*di_vtable.add(5));
		hr = set_coop_level(g_pMouse, g_wv.hWnd, 0x00000010 | 0x00000008); // DISCL_EXCLUSIVE | DISCL_FOREGROUND

		if hr != 0 {
			Com_Printf(b"Couldn't set DI coop level\n\0".as_ptr() as *const c_char);
			return qfalse;
		}

		// set the buffer size to DINPUT_BUFFERSIZE elements.
		// the buffer size is a DWORD property associated with the device
		let set_property: extern "C" fn(*mut c_void, DWORD, *const DIPROPHEADER) -> HRESULT = core::mem::transmute(*di_vtable.add(6));
		hr = set_property(g_pMouse, 1u32, &dipdw.diph); // DIPROP_BUFFERSIZE

		if hr != 0 {
			Com_Printf(b"Couldn't set DI buffersize\n\0".as_ptr() as *const c_char);
			return qfalse;
		}

		// clear any pending samples
		IN_DIMouse(&mut x, &mut y);
		IN_DIMouse(&mut x, &mut y);

		Com_Printf(b"DirectInput initialized.\n\0".as_ptr() as *const c_char);
		return qtrue;
	}
}

/*
==========================
IN_ShutdownDIMouse
==========================
*/
extern "C" fn IN_ShutdownDIMouse() {
	unsafe {
		if g_pMouse != null_mut() {
			let mouse_vtable = *(g_pMouse as *const *const *const c_void);
			let release: extern "C" fn(*mut c_void) -> HRESULT = core::mem::transmute(*mouse_vtable.add(2));
			release(g_pMouse);
			g_pMouse = null_mut();
		}
		if g_pdi != null_mut() {
			let di_vtable = *(g_pdi as *const *const *const c_void);
			let release: extern "C" fn(*mut c_void) -> HRESULT = core::mem::transmute(*di_vtable.add(2));
			release(g_pdi);
			g_pdi = null_mut();
		}
		if hInstDI != null_mut() {
			FreeLibrary(hInstDI);
			hInstDI = null_mut();
		}
	}
}

/*
==========================
IN_ActivateDIMouse
==========================
*/
extern "C" fn IN_ActivateDIMouse() {
	let mut hr: HRESULT;

	unsafe {
		if g_pMouse == null_mut() {
			return;
		}

		// we may fail to reacquire if the window has been recreated
		let mouse_vtable = *(g_pMouse as *const *const *const c_void);
		let acquire: extern "C" fn(*mut c_void) -> HRESULT = core::mem::transmute(*mouse_vtable.add(7));
		hr = acquire(g_pMouse);

		if hr != 0 {
			if IN_InitDIMouse() == qfalse {
				Com_Printf(b"Falling back to Win32 mouse support...\n\0".as_ptr() as *const c_char);
				Cvar_Set(b"in_mouse\0".as_ptr() as *const c_char, b"-1\0".as_ptr() as *const c_char);
			}
		}
	}
}

/*
==========================
IN_DeactivateDIMouse
==========================
*/
extern "C" fn IN_DeactivateDIMouse() {
	unsafe {
		if g_pMouse == null_mut() {
			return;
		}
		let mouse_vtable = *(g_pMouse as *const *const *const c_void);
		let unacquire: extern "C" fn(*mut c_void) -> HRESULT = core::mem::transmute(*mouse_vtable.add(8));
		unacquire(g_pMouse);
	}
}


/*
===================
IN_DIMouse
===================
*/
extern "C" fn IN_DIMouse(mx: *mut c_int, my: *mut c_int) {
	let mut od: DIDEVICEOBJECTDATA;
	let mut state: DIMOUSESTATE = DIMOUSESTATE { lX: 0, lY: 0, lZ: 0 };
	let mut dwElements: DWORD;
	let mut hr: HRESULT;

	unsafe {
		if g_pMouse == null_mut() {
			return;
		}

		// fetch new events
		loop {
			dwElements = 1;

			let mouse_vtable = *(g_pMouse as *const *const *const c_void);
			let get_device_data: extern "C" fn(*mut c_void, usize, *mut DIDEVICEOBJECTDATA, *mut DWORD, u32) -> HRESULT = core::mem::transmute(*mouse_vtable.add(9));

			od = DIDEVICEOBJECTDATA { dwOfs: 0, dwData: 0, dwTimeStamp: 0, dwSequence: 0 };
			hr = get_device_data(g_pMouse, 16, &mut od, &mut dwElements, 0); // sizeof(DIDEVICEOBJECTDATA) = 16

			if hr == 0xDFFF0003i32 || hr == 0xDFFF0005i32 { // DIERR_INPUTLOST or DIERR_NOTACQUIRED
				let acquire: extern "C" fn(*mut c_void) -> HRESULT = core::mem::transmute(*mouse_vtable.add(7));
				acquire(g_pMouse);
				return;
			}

			/* Unable to read data or no data available */
			if hr != 0 {
				break;
			}

			if dwElements == 0 {
				break;
			}

			if od.dwOfs == 0 { // DIMOFS_BUTTON0
				if od.dwData & 0x80 != 0 {
					Sys_QueEvent(od.dwTimeStamp as c_int, 1, A_MOUSE1, 1, 0, null_mut()); // SE_KEY
				} else {
					Sys_QueEvent(od.dwTimeStamp as c_int, 1, A_MOUSE1, 0, 0, null_mut());
				}
			} else if od.dwOfs == 4 { // DIMOFS_BUTTON1
				if od.dwData & 0x80 != 0 {
					Sys_QueEvent(od.dwTimeStamp as c_int, 1, A_MOUSE2, 1, 0, null_mut()); // SE_KEY
				} else {
					Sys_QueEvent(od.dwTimeStamp as c_int, 1, A_MOUSE2, 0, 0, null_mut());
				}
			} else if od.dwOfs == 8 { // DIMOFS_BUTTON2
				if od.dwData & 0x80 != 0 {
					Sys_QueEvent(od.dwTimeStamp as c_int, 1, A_MOUSE3, 1, 0, null_mut()); // SE_KEY
				} else {
					Sys_QueEvent(od.dwTimeStamp as c_int, 1, A_MOUSE3, 0, 0, null_mut());
				}
			}
		}

		// read the raw delta counter and ignore
		// the individual sample time / values
		let mouse_vtable = *(g_pMouse as *const *const *const c_void);
		let get_device_state: extern "C" fn(*mut c_void, usize, *mut DIMOUSESTATE) -> HRESULT = core::mem::transmute(*mouse_vtable.add(10));
		hr = get_device_state(g_pMouse, 12, &mut state); // sizeof(DIMOUSESTATE) = 12

		if hr != 0 {
			*mx = 0;
			*my = 0;
			return;
		}
		*mx = state.lX;
		*my = state.lY;
	}
}

/*
============================================================

  MOUSE CONTROL

============================================================
*/

/*
===========
IN_ActivateMouse

Called when the window gains focus or changes in some way
===========
*/
extern "C" fn IN_ActivateMouse() {
	unsafe {
		if s_wmv.mouseInitialized == qfalse {
			return;
		}
		if (*in_mouse).integer == 0 {
			s_wmv.mouseActive = qfalse;
			return;
		}
		if s_wmv.mouseActive != qfalse {
			return;
		}

		s_wmv.mouseActive = qtrue;

		if (*in_mouse).integer != -1 {
			IN_ActivateDIMouse();
		}
		IN_ActivateWin32Mouse();
	}
}


/*
===========
IN_DeactivateMouse

Called when the window loses focus
===========
*/
extern "C" fn IN_DeactivateMouse() {
	unsafe {
		if s_wmv.mouseInitialized == qfalse {
			return;
		}
		if s_wmv.mouseActive == qfalse {
			return;
		}
		s_wmv.mouseActive = qfalse;

		IN_DeactivateDIMouse();
		IN_DeactivateWin32Mouse();
	}
}



/*
===========
IN_StartupMouse
===========
*/
extern "C" fn IN_StartupMouse() {
	unsafe {
		s_wmv.mouseInitialized = qfalse;

		if (*in_mouse).integer == 0 {
			Com_Printf(b"Mouse control not active.\n\0".as_ptr() as *const c_char);
			return;
		}

		// nt4.0 direct input is screwed up
		if (g_wv.osversion.dwPlatformId == VER_PLATFORM_WIN32_NT) &&
		   (g_wv.osversion.dwMajorVersion == 4)
		{
			Com_Printf(b"Disallowing DirectInput on NT 4.0\n\0".as_ptr() as *const c_char);
			Cvar_Set(b"in_mouse\0".as_ptr() as *const c_char, b"-1\0".as_ptr() as *const c_char);
		}

		s_wmv.mouseInitialized = qtrue;

		if (*in_mouse).integer == -1 {
			Com_Printf(b"Skipping check for DirectInput\n\0".as_ptr() as *const c_char);
		} else {
			if IN_InitDIMouse() != qfalse {
				return;
			}
			Com_Printf(b"Falling back to Win32 mouse support...\n\0".as_ptr() as *const c_char);
		}
		IN_InitWin32Mouse();
	}
}

/*
===========
IN_MouseEvent
===========
*/
const MAX_MOUSE_BUTTONS: usize = 5;

static mouseConvert: [c_int; MAX_MOUSE_BUTTONS] = [
	A_MOUSE1,
	A_MOUSE2,
	A_MOUSE3,
	A_MOUSE4,
	A_MOUSE5
];

extern "C" fn IN_MouseEvent(mstate: c_int) {
	let mut i: c_int;

	unsafe {
		if s_wmv.mouseInitialized == qfalse {
			return;
		}

		// perform button actions
		for i in 0..MAX_MOUSE_BUTTONS as c_int {
			if ((mstate & (1 << i)) != 0) && ((s_wmv.oldButtonState & (1 << i)) == 0) {
				Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, mouseConvert[i as usize], 1, 0, null_mut()); // SE_KEY
			}
			if ((mstate & (1 << i)) == 0) && ((s_wmv.oldButtonState & (1 << i)) != 0) {
				Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, mouseConvert[i as usize], 0, 0, null_mut());
			}
		}
		s_wmv.oldButtonState = mstate;
	}
}



/*
===========
IN_MouseMove
===========
*/
extern "C" fn IN_MouseMove() {
	let mut mx: c_int = 0;
	let mut my: c_int = 0;

	unsafe {
		if g_pMouse != null_mut() {
			IN_DIMouse(&mut mx, &mut my);
		} else {
			IN_Win32Mouse(&mut mx, &mut my);
		}

		if mx == 0 && my == 0 {
			return;
		}

		Sys_QueEvent(0, 2, mx, my, 0, null_mut()); // SE_MOUSE
	}
}


/*
=========================================================================

=========================================================================
*/

/*
===========
IN_Startup
===========
*/
extern "C" fn IN_Startup() {
	unsafe {
		Com_Printf(b"\n------- Input Initialization -------\n\0".as_ptr() as *const c_char);
		IN_StartupMouse();
		IN_StartupJoystick();
		IN_StartupMIDI();
		Com_Printf(b"------------------------------------\n\0".as_ptr() as *const c_char);

		(*in_mouse).modified = qfalse;
		(*in_joystick).modified = qfalse;
	}
}

/*
===========
IN_Shutdown
===========
*/
extern "C" fn IN_Shutdown() {
	unsafe {
		IN_DeactivateMouse();
		IN_ShutdownDIMouse();
		IN_ShutdownMIDI();
		Cmd_RemoveCommand(b"midiinfo\0".as_ptr() as *const c_char);
		// #ifndef _IMMERSION
		// FF_Shutdown();
		// #endif // _IMMERSION
	}
}


/*
===========
IN_Init
===========
*/
extern "C" fn IN_Init() {
	unsafe {
		// MIDI input controler variables
		in_midi = Cvar_Get(b"in_midi\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 16); // CVAR_ARCHIVE = 16
		in_midiport = Cvar_Get(b"in_midiport\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 16);
		in_midichannel = Cvar_Get(b"in_midichannel\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 16);
		in_mididevice = Cvar_Get(b"in_mididevice\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 16);

		Cmd_AddCommand(b"midiinfo\0".as_ptr() as *const c_char, MidiInfo_f as *const c_void);

		// mouse variables
		in_mouse = Cvar_Get(b"in_mouse\0".as_ptr() as *const c_char, b"-1\0".as_ptr() as *const c_char, 40); // CVAR_ARCHIVE | CVAR_LATCH = 16 | 64

		// joystick variables
		in_joystick = Cvar_Get(b"in_joystick\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 40);
		in_joyBallScale = Cvar_Get(b"in_joyBallScale\0".as_ptr() as *const c_char, b"0.02\0".as_ptr() as *const c_char, 16);
		in_debugJoystick = Cvar_Get(b"in_debugjoystick\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 8); // CVAR_TEMP = 8

		joy_threshold = Cvar_Get(b"joy_threshold\0".as_ptr() as *const c_char, b"0.15\0".as_ptr() as *const c_char, 16);

		js_ffmult = Cvar_Get(b"js_ffmult\0".as_ptr() as *const c_char, b"3.0\0".as_ptr() as *const c_char, 16); // force feedback

		joy_xbutton = Cvar_Get(b"joy_xbutton\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 16); // treat axis as a button
		joy_ybutton = Cvar_Get(b"joy_ybutton\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 16); // treat axis as a button

		IN_Startup();
		// #ifndef _IMMERSION
		// FF_Init();
		// #endif // _IMMERSION
	}
}


/*
===========
IN_Activate

Called when the main window gains or loses focus.
The window may have been destroyed and recreated
between a deactivate and an activate.
===========
*/
extern "C" fn IN_Activate(active: qboolean) {
	unsafe {
		in_appactive = active;

		if active == qfalse {
			IN_DeactivateMouse();
		}
	}
}


/*
==================
IN_Frame

Called every frame, even if not generating commands
==================
*/
extern "C" fn IN_Frame() {
	unsafe {
		// post joystick events
		IN_JoyMove();

		if s_wmv.mouseInitialized == qfalse {
			return;
		}

		if cls.keyCatchers & 0x00000001 != 0 { // KEYCATCH_CONSOLE
			// temporarily deactivate if not in the game and
			// running on the desktop
			if Cvar_VariableIntegerValue(b"r_fullscreen\0".as_ptr() as *const c_char) == 0 {
				IN_DeactivateMouse();
				return;
			}
		}

		if in_appactive == qfalse {
			IN_DeactivateMouse();
			return;
		}

		IN_ActivateMouse();

		// post events to the system que
		IN_MouseMove();
	}
}


/*
===================
IN_ClearStates
===================
*/
extern "C" fn IN_ClearStates() {
	unsafe {
		s_wmv.oldButtonState = 0;
	}
}


/*
=========================================================================

JOYSTICK

=========================================================================
*/

/*
===============
IN_StartupJoystick
===============
*/
extern "C" fn IN_StartupJoystick() {
	let mut numdevs: c_int;
	let mut mmr: DWORD;

	unsafe {
		// assume no joystick
		joy.avail = qfalse;

		if (*in_joystick).integer == 0 {
			Com_Printf(b"Joystick is not active.\n\0".as_ptr() as *const c_char);
			return;
		}

		// verify joystick driver is present
		numdevs = joyGetNumDevs() as c_int;
		if numdevs == 0 {
			Com_Printf(b"joystick not found -- driver not present\n\0".as_ptr() as *const c_char);
			return;
		}

		// cycle through the joystick ids for the first valid one
		mmr = 0;
		joy.id = 0;
		loop {
			if joy.id >= numdevs {
				break;
			}

			memset(addr_of_mut!(joy.ji) as *mut c_void, 0, core::mem::size_of::<JOYINFOEX>());
			joy.ji.dwSize = core::mem::size_of::<JOYINFOEX>() as u32;
			joy.ji.dwFlags = 0x00000001; // JOY_RETURNCENTERED

			mmr = joyGetPosEx(joy.id as u32, addr_of_mut!(joy.ji));
			if mmr == 0 { // JOYERR_NOERROR
				break;
			}
			joy.id += 1;
		}

		// abort startup if we didn't find a valid joystick
		if mmr != 0 { // JOYERR_NOERROR
			Com_Printf(b"joystick not found -- no valid joysticks (%x)\n\0".as_ptr() as *const c_char, mmr);
			return;
		}

		// get the capabilities of the selected joystick
		// abort startup if command fails
		memset(addr_of_mut!(joy.jc) as *mut c_void, 0, core::mem::size_of::<JOYCAPS>());
		mmr = joyGetDevCaps(joy.id as u32, addr_of_mut!(joy.jc), core::mem::size_of::<JOYCAPS>());
		if mmr != 0 { // JOYERR_NOERROR
			Com_Printf(b"joystick not found -- invalid joystick capabilities (%x)\n\0".as_ptr() as *const c_char, mmr);
			return;
		}

		Com_Printf(b"Joystick found.\n\0".as_ptr() as *const c_char);
		Com_Printf(b"Pname: %s\n\0".as_ptr() as *const c_char, joy.jc.szPname.as_ptr());
		Com_Printf(b"OemVxD: %s\n\0".as_ptr() as *const c_char, joy.jc.szOEMVxD.as_ptr());
		Com_Printf(b"RegKey: %s\n\0".as_ptr() as *const c_char, joy.jc.szRegKey.as_ptr());

		Com_Printf(b"Numbuttons: %i / %i\n\0".as_ptr() as *const c_char, joy.jc.wNumButtons, joy.jc.wMaxButtons);
		Com_Printf(b"Axis: %i / %i\n\0".as_ptr() as *const c_char, joy.jc.wNumAxes, joy.jc.wMaxAxes);
		Com_Printf(b"Caps: 0x%x\n\0".as_ptr() as *const c_char, joy.jc.wCaps);
		if joy.jc.wCaps & 0x00000004 != 0 { // JOYCAPS_HASPOV
			Com_Printf(b"HASPOV\n\0".as_ptr() as *const c_char);
		} else {
			Com_Printf(b"no POV\n\0".as_ptr() as *const c_char);
		}

		// old button and POV states default to no buttons pressed
		joy.oldbuttonstate = 0;
		joy.oldpovstate = 0;

		// mark the joystick as available
		joy.avail = qtrue;
	}
}

/*
===========
JoyToF
===========
*/
extern "C" fn JoyToF(value: c_int) -> f32 {
	let mut fValue: f32;

	// move centerpoint to zero
	let value = value - 32768;

	// convert range from -32768..32767 to -1..1
	fValue = (value as f32) / 32768.0;

	if fValue < -1.0 {
		fValue = -1.0;
	}
	if fValue > 1.0 {
		fValue = 1.0;
	}
	fValue
}

extern "C" fn JoyToI(value: c_int) -> c_int {
	// move centerpoint to zero
	let value = value - 32768;

	value
}

static joyDirectionKeys: [c_int; 16] = [
	A_CURSOR_LEFT,
	A_CURSOR_RIGHT,
	A_CURSOR_UP,
	A_CURSOR_DOWN,
	A_JOY16,
	A_JOY17,
	A_JOY18,
	A_JOY19,
	A_JOY20,
	A_JOY21,
	A_JOY22,
	A_JOY23,
	A_JOY24,
	A_JOY25,
	A_JOY26,
	A_JOY27,
];

/*
===========
IN_JoyMove
===========
*/
extern "C" fn IN_JoyMove() {
	let mut fAxisValue: f32;
	let mut i: c_int;
	let mut buttonstate: DWORD;
	let mut povstate: DWORD;
	let mut x: c_int;
	let mut y: c_int;

	unsafe {
		// verify joystick is available and that the user wants to use it
		if joy.avail == qfalse {
			return;
		}

		// collect the joystick data, if possible
		memset(addr_of_mut!(joy.ji) as *mut c_void, 0, core::mem::size_of::<JOYINFOEX>());
		joy.ji.dwSize = core::mem::size_of::<JOYINFOEX>() as u32;
		joy.ji.dwFlags = 0x000000FF; // JOY_RETURNALL

		if joyGetPosEx(joy.id as u32, addr_of_mut!(joy.ji)) as DWORD != 0 { // JOYERR_NOERROR
			// read error occurred
			// turning off the joystick seems too harsh for 1 read error,\
			// but what should be done?
			// Com_Printf ("IN_ReadJoystick: no response\n");
			// joy.avail = false;
			return;
		}

		if (*in_debugJoystick).integer != 0 {
			Com_Printf(b"%8x %5i %5.2f %5.2f %5.2f %5.2f %6i %6i\n\0".as_ptr() as *const c_char,
				joy.ji.dwButtons,
				joy.ji.dwPOV,
				JoyToF(joy.ji.dwXpos as c_int), JoyToF(joy.ji.dwYpos as c_int),
				JoyToF(joy.ji.dwZpos as c_int), JoyToF(joy.ji.dwRpos as c_int),
				JoyToI(joy.ji.dwUpos as c_int), JoyToI(joy.ji.dwVpos as c_int));
		}

		// loop through the joystick buttons
		// key a joystick event or auxillary event for higher number buttons for each state change
		buttonstate = joy.ji.dwButtons;
		i = 0;
		loop {
			if i >= joy.jc.wNumButtons as c_int {
				break;
			}
			if (buttonstate & (1 << i)) != 0 && (joy.oldbuttonstate & (1 << i)) == 0 {
				Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, A_JOY0 + i, qtrue, 0, null_mut()); // SE_KEY
			}
			if (buttonstate & (1 << i)) == 0 && (joy.oldbuttonstate & (1 << i)) != 0 {
				Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, A_JOY0 + i, qfalse, 0, null_mut());
			}
			i += 1;
		}
		joy.oldbuttonstate = buttonstate as c_int;

		povstate = 0;

		// convert main joystick motion into 6 direction button bits
		i = 0;
		loop {
			if i >= joy.jc.wNumAxes as c_int || i >= 4 {
				break;
			}
			// get the floating point zero-centered, potentially-inverted data for the current axis
			let axis_values = [joy.ji.dwXpos as c_int, joy.ji.dwYpos as c_int, joy.ji.dwZpos as c_int, joy.ji.dwRpos as c_int];
			fAxisValue = JoyToF(axis_values[i as usize]);

			if i == 0 && (*joy_xbutton).integer == 0 {
				if fAxisValue < -(*joy_threshold).value || fAxisValue > (*joy_threshold).value {
					Sys_QueEvent(g_wv.sysMsgTime as c_int, 8, 0, (-(fAxisValue * 127.0)) as c_int, 0, null_mut()); // SE_JOYSTICK_AXIS, AXIS_SIDE
				} else {
					Sys_QueEvent(g_wv.sysMsgTime as c_int, 8, 0, 0, 0, null_mut());
				}
				i += 1;
				continue;
			}

			if i == 1 && (*joy_ybutton).integer == 0 {
				if fAxisValue < -(*joy_threshold).value || fAxisValue > (*joy_threshold).value {
					Sys_QueEvent(g_wv.sysMsgTime as c_int, 8, 1, (-(fAxisValue * 127.0)) as c_int, 0, null_mut()); // SE_JOYSTICK_AXIS, AXIS_FORWARD
				} else {
					Sys_QueEvent(g_wv.sysMsgTime as c_int, 8, 1, 0, 0, null_mut());
				}
				i += 1;
				continue;
			}

			if fAxisValue < -(*joy_threshold).value {
				povstate |= 1 << (i * 2);
			} else if fAxisValue > (*joy_threshold).value {
				povstate |= 1 << (i * 2 + 1);
			}
			i += 1;
		}

		// convert POV information from a direction into 4 button bits
		if joy.jc.wCaps & 0x00000004 != 0 { // JOYCAPS_HASPOV
			if joy.ji.dwPOV != 0xFFFFu32 { // JOY_POVCENTERED
				if joy.ji.dwPOV == 0 { // JOY_POVFORWARD
					povstate |= 1 << 12;
				}
				if joy.ji.dwPOV == 18000 { // JOY_POVBACKWARD
					povstate |= 1 << 13;
				}
				if joy.ji.dwPOV == 9000 { // JOY_POVRIGHT
					povstate |= 1 << 14;
				}
				if joy.ji.dwPOV == 27000 { // JOY_POVLEFT
					povstate |= 1 << 15;
				}
			}
		}

		// determine which bits have changed and key an auxillary event for each change
		for i in 0..16 {
			if (povstate & (1 << i)) != 0 && (joy.oldpovstate & (1 << i)) == 0 {
				Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, joyDirectionKeys[i], qtrue, 0, null_mut()); // SE_KEY
			}

			if (povstate & (1 << i)) == 0 && (joy.oldpovstate & (1 << i)) != 0 {
				Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, joyDirectionKeys[i], qfalse, 0, null_mut());
			}
		}
		joy.oldpovstate = povstate as c_int;

		// if there is a trackball like interface, simulate mouse moves
		if joy.jc.wNumAxes >= 6 {
			x = (JoyToI(joy.ji.dwUpos as c_int) as f32 * (*in_joyBallScale).value) as c_int;
			y = (JoyToI(joy.ji.dwVpos as c_int) as f32 * (*in_joyBallScale).value) as c_int;
			if x != 0 || y != 0 {
				Sys_QueEvent(g_wv.sysMsgTime as c_int, 2, x, y, 0, null_mut()); // SE_MOUSE
			}
		}
	}
}

/*
=========================================================================

MIDI

=========================================================================
*/

extern "C" {
	pub fn joyGetNumDevs() -> u32;
	pub fn joyGetPosEx(uJoyID: u32, pji: *mut JOYINFOEX) -> u32;
	pub fn joyGetDevCaps(uJoyID: u32, pjc: *mut JOYCAPS, cbjc: usize) -> u32;
	pub fn midiInGetNumDevs() -> u32;
	pub fn midiInGetDevCaps(uDeviceID: u32, lpMidiInCaps: *mut MIDIINCAPS, cbMidiInCaps: usize) -> u32;
	pub fn midiInOpen(lphMidiIn: *mut HMIDIIN, uDeviceID: u32, dwCallback: u32, dwInstance: u32, fdwOpen: u32) -> u32;
	pub fn midiInStart(hMidiIn: HMIDIIN) -> u32;
	pub fn midiInClose(hMidiIn: HMIDIIN) -> u32;
}

fn MIDI_NoteOff(note: c_int) {
	let qkey: c_int;

	unsafe {
		qkey = note - 60 + A_AUX0;

		if qkey < A_AUX0 {
			return;
		}
		Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, qkey, qfalse, 0, null_mut()); // SE_KEY
	}
}

fn MIDI_NoteOn(note: c_int, velocity: c_int) {
	let qkey: c_int;

	unsafe {
		if velocity == 0 {
			MIDI_NoteOff(note);
		}

		qkey = note - 60 + A_AUX0;

		if qkey < A_AUX0 {
			return;
		}
		Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, qkey, qtrue, 0, null_mut()); // SE_KEY
	}
}

extern "C" fn MidiInProc(hMidiIn: HMIDIIN, uMsg: UINT, dwInstance: u32, dwParam1: u32, dwParam2: u32) {
	let message: c_int;

	unsafe {
		if uMsg == 0x3C1 { // MIM_OPEN
		} else if uMsg == 0x3C2 { // MIM_CLOSE
		} else if uMsg == 0x3C3 { // MIM_DATA
			message = (dwParam1 & 0xff) as c_int;

			// note on
			if (message & 0xf0) == 0x90 {
				if ((message & 0x0f) + 1) as c_int == (*in_midichannel).integer {
					MIDI_NoteOn(((dwParam1 & 0xff00) >> 8) as c_int, ((dwParam1 & 0xff0000) >> 16) as c_int);
				}
			} else if (message & 0xf0) == 0x80 {
				if ((message & 0x0f) + 1) as c_int == (*in_midichannel).integer {
					MIDI_NoteOff(((dwParam1 & 0xff00) >> 8) as c_int);
				}
			}
		} else if uMsg == 0x3C4 { // MIM_LONGDATA
		} else if uMsg == 0x3C5 { // MIM_ERROR
		} else if uMsg == 0x3C6 { // MIM_LONGERROR
		}

		//	Sys_QueEvent( sys_msg_time, SE_KEY, wMsg, qtrue, 0, NULL );
	}
}

fn MidiInfo_f() {
	let mut i: c_int;

	let enableStrings: [*const c_char; 2] = [
		b"disabled\0".as_ptr() as *const c_char,
		b"enabled\0".as_ptr() as *const c_char
	];

	unsafe {
		Com_Printf(b"\nMIDI control:       %s\n\0".as_ptr() as *const c_char, enableStrings[((*in_midi).integer != 0) as usize]);
		Com_Printf(b"port:               %d\n\0".as_ptr() as *const c_char, (*in_midiport).integer);
		Com_Printf(b"channel:            %d\n\0".as_ptr() as *const c_char, (*in_midichannel).integer);
		Com_Printf(b"current device:     %d\n\0".as_ptr() as *const c_char, (*in_mididevice).integer);
		Com_Printf(b"number of devices:  %d\n\0".as_ptr() as *const c_char, s_midiInfo.numDevices);
		for i in 0..s_midiInfo.numDevices {
			if i == Cvar_VariableIntegerValue(b"in_mididevice\0".as_ptr() as *const c_char) {
				Com_Printf(b"***\0".as_ptr() as *const c_char);
			} else {
				Com_Printf(b"...\0".as_ptr() as *const c_char);
			}
			Com_Printf(b"device %2d:       %s\n\0".as_ptr() as *const c_char, i, s_midiInfo.caps[i as usize].szPname.as_ptr());
			Com_Printf(b"...manufacturer ID: 0x%hx\n\0".as_ptr() as *const c_char, s_midiInfo.caps[i as usize].wMid);
			Com_Printf(b"...product ID:      0x%hx\n\0".as_ptr() as *const c_char, s_midiInfo.caps[i as usize].wPid);

			Com_Printf(b"\n\0".as_ptr() as *const c_char);
		}
	}
}

fn IN_StartupMIDI() {
	let mut i: c_int;

	unsafe {
		if Cvar_VariableIntegerValue(b"in_midi\0".as_ptr() as *const c_char) == 0 {
			return;
		}

		//
		// enumerate MIDI IN devices
		//
		s_midiInfo.numDevices = midiInGetNumDevs() as c_int;

		for i in 0..s_midiInfo.numDevices {
			midiInGetDevCaps(i as u32, addr_of_mut!(s_midiInfo.caps[i as usize]), core::mem::size_of::<MIDIINCAPS>());
		}

		//
		// open the MIDI IN port
		//
		if midiInOpen(
			addr_of_mut!(s_midiInfo.hMidiIn),
			(*in_mididevice).integer as u32,
			MidiInProc as u32,
			0,
			0x00000010, // CALLBACK_FUNCTION
		) != 0 { // MMSYSERR_NOERROR
			Com_Printf(
				b"WARNING: could not open MIDI device %d: '%s'\n\0".as_ptr() as *const c_char,
				(*in_mididevice).integer,
				s_midiInfo.caps[(*in_mididevice).value as usize].szPname.as_ptr()
			);
			return;
		}

		midiInStart(s_midiInfo.hMidiIn);
	}
}

fn IN_ShutdownMIDI() {
	unsafe {
		if s_midiInfo.hMidiIn != null_mut() {
			midiInClose(s_midiInfo.hMidiIn);
		}
		memset(addr_of_mut!(s_midiInfo) as *mut c_void, 0, core::mem::size_of::<MidiInfo_t>());
	}
}
