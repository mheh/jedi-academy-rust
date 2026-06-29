#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use crate::codemp::ui::keycodes_h::*;

// Anything above this include will be ignored by the compiler
// include ../qcommon/exe_headers.h
// include ../client/client.h
// include win_local.h

// Forward declaration - extern types/functions that need to be defined elsewhere
extern "C" {
	pub static mut g_wv: WinVars_t;

	// Functions from other modules
	pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
	pub fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
	pub fn Com_DPrintf(fmt: *const c_char, ...);
	pub fn Key_ClearStates();
	pub fn IN_Activate(activate: u32);
	pub fn GetKeyState(nVirtKey: c_int) -> c_int;
	pub fn MapVirtualKey(uCode: u32, uMapType: u32) -> u32;
	pub fn GetWindowLong(hWnd: *mut c_void, nIndex: c_int) -> c_int;
	pub fn AdjustWindowRect(lpRect: *mut RECT, dwStyle: u32, bMenu: u32) -> u32;
	pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
	pub fn Cvar_SetValue(var_name: *const c_char, value: f32);
	pub fn RegisterHotKey(hWnd: *mut c_void, id: c_int, fsModifiers: u32, vk: u32) -> u32;
	pub fn UnregisterHotKey(hWnd: *mut c_void, id: c_int) -> u32;
	pub fn SystemParametersInfo(uiAction: u32, uiParam: u32, pvParam: *mut c_void, fWinIni: u32) -> u32;
	pub fn Sys_QueEvent(time: c_int, type_: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void);
	pub fn DefWindowProc(hWnd: *mut c_void, uMsg: u32, wParam: usize, lParam: isize) -> isize;
	pub fn RegisterWindowMessage(lpString: *const c_char) -> u32;
	pub fn Cbuf_AddText(text: *const c_char);
	pub fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
	pub fn SNDDMA_Activate(activate: u32);
	pub fn IN_MouseEvent(buttons: c_int);
}

// Stub types that need to be defined elsewhere
#[repr(C)]
pub struct WinVars_t {
	// Placeholder - actual structure defined elsewhere
	pub hWnd: *mut c_void,
	pub sysMsgTime: c_int,
	pub isMinimized: u32,
	pub activeApp: u32,
	// ... other fields
}

#[repr(C)]
pub struct cvar_t {
	pub name: *const c_char,
	pub string: *const c_char,
	pub latched_string: *const c_char,
	pub integer: c_int,
	pub value: f32,
	pub modified: u32,
	pub latchedInteger: c_int,
	pub latchedValue: f32,
	pub flags: c_int,
	pub next: *mut cvar_t,
	pub prev: *mut cvar_t,
}

#[repr(C)]
pub struct RECT {
	pub left: c_int,
	pub top: c_int,
	pub right: c_int,
	pub bottom: c_int,
}

#[repr(C)]
pub struct CLASS {
	pub state: c_int,
	// Placeholder - actual structure defined elsewhere
}

// The only directly referenced keycode - the console key (which gives different ascii codes depending on locale)
const CONSOLE_SCAN_CODE: u32 = 0x29;

const WM_MOUSEWHEEL: u32 = 0x020A;

// note: the following is a workaround for platforms that don't have this message
// #ifndef WM_MOUSEWHEEL
// #define WM_MOUSEWHEEL (WM_MOUSELAST+1)  // message that will be supported by the OS
// #endif

static mut MSH_MOUSEWHEEL: u32 = 0;

// Console variables that we need to access from this module
static mut vid_xpos: *mut cvar_t = core::ptr::null_mut();            // X coordinate of window position
static mut vid_ypos: *mut cvar_t = core::ptr::null_mut();            // Y coordinate of window position
static mut r_fullscreen: *mut cvar_t = core::ptr::null_mut();

// #define VID_NUM_MODES ( sizeof( vid_modes ) / sizeof( vid_modes[0] ) )

static mut s_alttab_disabled: u32 = 0;

unsafe fn WIN_DisableAltTab() {
	if s_alttab_disabled != 0 {
		return;
	}

	if Q_stricmp(Cvar_VariableString(b"arch\0".as_ptr() as *const c_char), b"winnt\0".as_ptr() as *const c_char) == 0 {
		RegisterHotKey(core::ptr::null_mut(), 0, 0x0001, 0x09); // MOD_ALT, VK_TAB
	}
	else {
		let mut old: u32 = 0;

		SystemParametersInfo(0x0011, 1, &mut old as *mut _ as *mut c_void, 0); // SPI_SCREENSAVERRUNNING
	}
	s_alttab_disabled = 1; // qtrue
}

unsafe fn WIN_EnableAltTab() {
	if s_alttab_disabled != 0 {
		if Q_stricmp(Cvar_VariableString(b"arch\0".as_ptr() as *const c_char), b"winnt\0".as_ptr() as *const c_char) == 0 {
			UnregisterHotKey(core::ptr::null_mut(), 0);
		}
		else {
			let mut old: u32 = 0;

			SystemParametersInfo(0x0011, 0, &mut old as *mut _ as *mut c_void, 0); // SPI_SCREENSAVERRUNNING
		}

		s_alttab_disabled = 0; // qfalse
	}
}

/*
==================
VID_AppActivate
==================
*/
unsafe fn VID_AppActivate(fActive: u32, minimize: u32) {
	(*core::ptr::addr_of_mut!(g_wv)).isMinimized = minimize;

	Com_DPrintf(b"VID_AppActivate: %i\n\0".as_ptr() as *const c_char, fActive);

	Key_ClearStates();	// FIXME!!!

	// we don't want to act like we're active if we're minimized
	if fActive != 0 && (*core::ptr::addr_of_mut!(g_wv)).isMinimized == 0 {
		(*core::ptr::addr_of_mut!(g_wv)).activeApp = 1; // qtrue
	}
	else {
		(*core::ptr::addr_of_mut!(g_wv)).activeApp = 0; // qfalse
	}

	// minimize/restore mouse-capture on demand
	if (*core::ptr::addr_of_mut!(g_wv)).activeApp == 0 {
		IN_Activate(0); // qfalse
	}
	else {
		IN_Activate(1); // qtrue
	}
}

//==========================================================================

static VIRTUALKEY_CONVERT: [[u8; 2]; 0x92] = [
	[0,				0				],
	[141,			141		], // VK_LBUTTON 01 Left mouse button  (A_MOUSE1)
	[142,			142		], // VK_RBUTTON 02 Right mouse button  (A_MOUSE2)
	[0,				0				], // VK_CANCEL 03 Control-break processing
	[166,			166		], // VK_MBUTTON 04 Middle mouse button (three-button mouse)  (A_MOUSE3)
	[167,			167		], // VK_XBUTTON1 05 Windows 2000/XP: X1 mouse button  (A_MOUSE4)
	[168,			168		], // VK_XBUTTON2 06 Windows 2000/XP: X2 mouse button  (A_MOUSE5)
	[0,				0				], // 07 Undefined
	[8,			8		], // VK_BACK 08 BACKSPACE key  (A_BACKSPACE)
	[9,			9			], // VK_TAB 09 TAB key  (A_TAB)
	[0,				0				], // 0A Reserved
	[0,				0				], // 0B Reserved
	[21,			0				], // VK_CLEAR 0C CLEAR key  (A_KP_5)
	[10, 			13 		], // VK_RETURN 0D ENTER key  (A_ENTER, A_KP_ENTER)
	[0,				0				], // 0E Undefined
	[0,				0				], // 0F Undefined
	[1,			1			], // VK_SHIFT 10 SHIFT key  (A_SHIFT)
	[2,			2			], // VK_CONTROL 11 CTRL key  (A_CTRL)
	[3,			3			], // VK_MENU 12 ALT key  (A_ALT)
	[7,			7			], // VK_PAUSE 13 PAUSE key  (A_PAUSE)
	[4,		4		], // VK_CAPITAL 14 CAPS LOCK key  (A_CAPSLOCK)
	[0,				0				], // VK_KANA 15 IME Kana mode
	[0,				0				], // 16 Undefined
	[0,				0				], // VK_JUNJA 17 IME Junja mode
	[0,				0				], // VK_FINAL 18 IME final mode
	[0,				0				], // VK_KANJI 19 IME Kanji mode
	[0,				0				], // 1A Undefined
	[27,			27		], // VK_ESCAPE 1B ESC key  (A_ESCAPE)
	[0,				0				], // VK_CONVERT 1C IME convert
	[0,				0				], // VK_NONCONVERT 1D IME nonconvert
	[0,				0				], // VK_ACCEPT 1E IME accept
	[0,				0				], // VK_MODECHANGE 1F IME mode change request
	[32,			32		], // VK_SPACE 20 SPACEBAR  (A_SPACE)
	[25,			145		], // VK_PRIOR 21 PAGE UP key  (A_KP_9, A_PAGE_UP)
	[19,			158		], // VK_NEXT 22 PAGE DOWN key  (A_KP_3, A_PAGE_DOWN)
	[17,			157		], // VK_END 23 END key  (A_KP_1, A_END)
	[23,			144		], // VK_HOME 24 HOME key  (A_KP_7, A_HOME)
	[20,			172	], // VK_LEFT 25 LEFT ARROW key  (A_KP_4, A_CURSOR_LEFT)
	[24,			170   	], // VK_UP 26 UP ARROW key  (A_KP_8, A_CURSOR_UP)
	[22,			173	], // VK_RIGHT 27 RIGHT ARROW key  (A_KP_6, A_CURSOR_RIGHT)
	[18,			171	], // VK_DOWN 28 DOWN ARROW key  (A_KP_2, A_CURSOR_DOWN)
	[0,				0				], // VK_SELECT 29 SELECT key
	[0,				0				], // VK_PRINT 2A PRINT key
	[0,				0				], // VK_EXECUTE 2B EXECUTE key
	[15,	15	], // VK_SNAPSHOT 2C PRINT SCREEN key  (A_PRINTSCREEN)
	[16,			143		], // VK_INSERT 2D INS key  (A_KP_0, A_INSERT)
	[14,		127		], // VK_DELETE 2E DEL key  (A_KP_PERIOD, A_DELETE)
	[0,				0				], // VK_HELP 2F HELP key
	[48,			48				], // 30 0 key  (A_0)
	[49,			49				], // 31 1 key  (A_1)
	[50,			50				], // 32 2 key  (A_2)
	[51,			51				], // 33 3 key  (A_3)
	[52,			52				], // 34 4 key  (A_4)
	[53,			53				], // 35 5 key  (A_5)
	[54,			54				], // 36 6 key  (A_6)
	[55,			55				], // 37 7 key  (A_7)
	[56,			56				], // 38 8 key  (A_8)
	[57,			57				], // 39 9 key  (A_9)
	[0,				0				], // 3A Undefined
	[0,				0				], // 3B Undefined
	[0,				0				], // 3C Undefined
	[0,				0				], // 3D Undefined
	[0,				0				], // 3E Undefined
	[0,				0				], // 3F Undefined
	[0,				0				], // 40 Undefined
	[65,			65		], // 41 A key  (A_CAP_A)
	[66,			66		], // 42 B key  (A_CAP_B)
	[67,			67		], // 43 C key  (A_CAP_C)
	[68,			68		], // 44 D key  (A_CAP_D)
	[69,			69		], // 45 E key  (A_CAP_E)
	[70,			70		], // 46 F key  (A_CAP_F)
	[71,			71		], // 47 G key  (A_CAP_G)
	[72,			72		], // 48 H key  (A_CAP_H)
	[73,			73		], // 49 I key  (A_CAP_I)
	[74,			74		], // 4A J key  (A_CAP_J)
	[75,			75		], // 4B K key  (A_CAP_K)
	[76,			76		], // 4C L key  (A_CAP_L)
	[77,			77		], // 4D M key  (A_CAP_M)
	[78,			78		], // 4E N key  (A_CAP_N)
	[79,			79		], // 4F O key  (A_CAP_O)
	[80,			80		], // 50 P key  (A_CAP_P)
	[81,			81		], // 51 Q key  (A_CAP_Q)
	[82,			82		], // 52 R key  (A_CAP_R)
	[83,			83		], // 53 S key  (A_CAP_S)
	[84,			84		], // 54 T key  (A_CAP_T)
	[85,			85		], // 55 U key  (A_CAP_U)
	[86,			86		], // 56 V key  (A_CAP_V)
	[87,			87		], // 57 W key  (A_CAP_W)
	[88,			88		], // 58 X key  (A_CAP_X)
	[89,			89		], // 59 Y key  (A_CAP_Y)
	[90,			90		], // 5A Z key  (A_CAP_Z)
	[0,				0				], // VK_LWIN 5B Left Windows key (Microsoft Natural keyboard)
	[0,				0				], // VK_RWIN 5C Right Windows key (Natural keyboard)
	[0,				0				], // VK_APPS 5D Applications key (Natural keyboard)
	[0,				0				], // 5E Reserved
	[0,				0				], // VK_SLEEP 5F Computer Sleep key
	[16,			16		], // VK_NUMPAD0 60 Numeric keypad 0 key  (A_KP_0)
	[17,			17		], // VK_NUMPAD1 61 Numeric keypad 1 key  (A_KP_1)
	[18,			18		], // VK_NUMPAD2 62 Numeric keypad 2 key  (A_KP_2)
	[19,			19		], // VK_NUMPAD3 63 Numeric keypad 3 key  (A_KP_3)
	[20,			20		], // VK_NUMPAD4 64 Numeric keypad 4 key  (A_KP_4)
	[21,			21		], // VK_NUMPAD5 65 Numeric keypad 5 key  (A_KP_5)
	[22,			22		], // VK_NUMPAD6 66 Numeric keypad 6 key  (A_KP_6)
	[23,			23		], // VK_NUMPAD7 67 Numeric keypad 7 key  (A_KP_7)
	[24,			24		], // VK_NUMPAD8 68 Numeric keypad 8 key  (A_KP_8)
	[25,			25		], // VK_NUMPAD9 69 Numeric keypad 9 key  (A_KP_9)
	[215,		215		], // VK_MULTIPLY 6A Multiply key  (A_MULTIPLY)
	[11, 			11 		], // VK_ADD 6B Add key  (A_KP_PLUS)
	[0,				0				], // VK_SEPARATOR 6C Separator key
	[12,			12		], // VK_SUBTRACT 6D Subtract key  (A_KP_MINUS)
	[14,			14		], // VK_DECIMAL 6E Decimal key  (A_KP_PERIOD)
	[247,			247		], // VK_DIVIDE 6F Divide key  (A_DIVIDE)
	[28,			28		], // VK_F1 70 F1 key  (A_F1)
	[29,			29		], // VK_F2 71 F2 key  (A_F2)
	[30,			30		], // VK_F3 72 F3 key  (A_F3)
	[31,			31		], // VK_F4 73 F4 key  (A_F4)
	[132,			132		], // VK_F5 74 F5 key  (A_F5)
	[133,			133		], // VK_F6 75 F6 key  (A_F6)
	[134,			134		], // VK_F7 76 F7 key  (A_F7)
	[135,			135		], // VK_F8 77 F8 key  (A_F8)
	[149,			149		], // VK_F9 78 F9 key  (A_F9)
	[150,			150		], // VK_F10 79 F10 key  (A_F10)
	[151,			151		], // VK_F11 7A F11 key  (A_F11)
	[152,			152		], // VK_F12 7B F12 key  (A_F12)
	[0,				0				], // VK_F13 7C F13 key
	[0,				0				], // VK_F14 7D F14 key
	[0,				0				], // VK_F15 7E F15 key
	[0,				0				], // VK_F16 7F F16 key
	[0,				0				], // VK_F17 80H F17 key
	[0,				0				], // VK_F18 81H F18 key
	[0,				0				], // VK_F19 82H F19 key
	[0,				0				], // VK_F20 83H F20 key
	[0,				0				], // VK_F21 84H F21 key
	[0,				0				], // VK_F22 85H F22 key
	[0,				0				], // VK_F23 86H F23 key
	[0,				0				], // VK_F24 87H F24 key
	[0,				0				], // 88 Unassigned
	[0,				0				], // 89 Unassigned
	[0,				0				], // 8A Unassigned
	[0,				0				], // 8B Unassigned
	[0,				0				], // 8C Unassigned
	[0,				0				], // 8D Unassigned
	[0,				0				], // 8E Unassigned
	[0,				0				], // 8F Unassigned
	[5,		5		], // VK_NUMLOCK 90 NUM LOCK key  (A_NUMLOCK)
	[6,			6	]  // VK_SCROLL 91  (A_SCROLLLOCK)
];

/*
=======
MapKey

Map from windows to quake keynums
=======
*/
unsafe fn MapKey(key: u32, mut wParam: u16) -> c_int {
	let mut result: u32 = 0;
	let scan: u32;
	let extended: u32;

	// Check for the console key (hard code to the key you would expect)
	scan = (key >> 16) & 0xff;
	if scan == CONSOLE_SCAN_CODE {
		return 26 as c_int; // A_CONSOLE
	}

	// Try to convert the virtual key directly
	result = 0;
	extended = (key >> 24) & 1;
	if wParam as u32 > 0 && wParam as u32 <= 0x91 { // VK_SCROLL
		// yeuch, but oh well...
		//
		if wParam as u32 >= 0x60 && wParam as u32 <= 0x69 { // VK_NUMPAD0 .. VK_NUMPAD9
			let bNumlockOn = (GetKeyState(0x90) & 1) != 0; // VK_NUMLOCK
			if bNumlockOn {
				wParam = (0x30 + (wParam as u32 - 0x60)) as u16;	// convert to standard 0..9
			}
		}
		if extended as usize < 2 && wParam as usize < VIRTUALKEY_CONVERT.len() {
			result = VIRTUALKEY_CONVERT[wParam as usize][extended as usize] as u32;
		}
	}
	// Get the unshifted ascii code (if any)
	if result == 0 {
		result = MapVirtualKey(wParam as u32, 2) & 0xff;
	}
	// Output any debug prints
	//	if(in_debug && in_debug->integer & 1)
	//	{
	//		Com_Printf("WM_KEY: %x : %x : %x\n", key, wParam, result);
	//	}
	result as c_int
}


/*
====================
MainWndProc

main window procedure
====================
*/

const WM_BUTTON4DOWN: u32 = 0x020B; // (WM_MOUSELAST+2)
const WM_BUTTON4UP: u32 = 0x020C; // (WM_MOUSELAST+3)
const MK_BUTTON4L: u32 = 0x0020;
const MK_BUTTON4R: u32 = 0x0040;

pub extern "C" fn MainWndProc(
    hWnd: *mut c_void,
    uMsg: u32,
    wParam: usize,
    lParam: isize) -> isize
{
	unsafe {
		let mut code: u8;

		if uMsg == MSH_MOUSEWHEEL {
			if (wParam as c_int) > 0 {
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, 137, 1, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELUP, qtrue
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, 137, 0, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELUP, qfalse
			}
			else {
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, 139, 1, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELDOWN, qtrue
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, 139, 0, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELDOWN, qfalse
			}
	        return DefWindowProc(hWnd, uMsg, wParam, lParam as usize);
		}

		match uMsg {
		0x020A => {  // WM_MOUSEWHEEL
			//
			//
			// this chunk of code theoretically only works under NT4 and Win98
			// since this message doesn't exist under Win95
			//
			if ((wParam >> 16) as i16) > 0 {
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, 137, 1, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELUP, qtrue
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, 137, 0, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELUP, qfalse
			}
			else {
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, 139, 1, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELDOWN, qtrue
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, 139, 0, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELDOWN, qfalse
			}
		},

		0x0001 => {  // WM_CREATE

			(*core::ptr::addr_of_mut!(g_wv)).hWnd = hWnd;

			vid_xpos = Cvar_Get(b"vid_xpos\0".as_ptr() as *const c_char, b"3\0".as_ptr() as *const c_char, 0x0010); // CVAR_ARCHIVE
			vid_ypos = Cvar_Get(b"vid_ypos\0".as_ptr() as *const c_char, b"22\0".as_ptr() as *const c_char, 0x0010); // CVAR_ARCHIVE
			r_fullscreen = Cvar_Get(b"r_fullscreen\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0x0010 | 0x0020); // CVAR_ARCHIVE | CVAR_LATCH

			MSH_MOUSEWHEEL = RegisterWindowMessage(b"MSWHEEL_ROLLMSG\0".as_ptr() as *const c_char);
			if !r_fullscreen.is_null() && (*r_fullscreen).integer != 0 {
				WIN_DisableAltTab();
			}
			else {
				WIN_EnableAltTab();
			}

		},
		// #if 0
		// case WM_DISPLAYCHANGE:
		// 	Com_DPrintf( "WM_DISPLAYCHANGE\n" );
		// 	// we need to force a vid_restart if the user has changed
		// 	// their desktop resolution while the game is running,
		// 	// but don't do anything if the message is a result of
		// 	// our own calling of ChangeDisplaySettings
		// 	if ( com_insideVidInit ) {
		// 		break;		// we did this on purpose
		// 	}
		// 	// something else forced a mode change, so restart all our gl stuff
		// 	Cbuf_AddText( "vid_restart\n" );
		// 	break;
		// #endif
		0x0002 => {  // WM_DESTROY
			// let sound and input know about this?
			(*core::ptr::addr_of_mut!(g_wv)).hWnd = core::ptr::null_mut();
			if !r_fullscreen.is_null() && (*r_fullscreen).integer != 0 {
				WIN_EnableAltTab();
			}
		},

		0x0010 => {  // WM_CLOSE
			Cbuf_ExecuteText(4, b"quit\0".as_ptr() as *const c_char); // EXEC_APPEND
		},

		0x0006 => {  // WM_ACTIVATE
			{
				let fActive: c_int = (wParam & 0xFFFF) as c_int;
				let fMinimized: c_int = ((wParam >> 16) & 0xFFFF) as c_int;

				VID_AppActivate(
					if fActive != 0 { 1 } else { 0 },
					fMinimized as u32
				);
				SNDDMA_Activate(if fActive != 0 && fMinimized == 0 { 1 } else { 0 });
			}
		},

		0x0003 => {  // WM_MOVE
			{
				let xPos: c_int = (lParam as u16) as i16 as c_int;    // horizontal position
				let yPos: c_int = ((lParam >> 16) as u16) as i16 as c_int;    // vertical position
				let mut r: RECT = RECT {
					left: 0,
					top: 0,
					right: 1,
					bottom: 1,
				};
				let style: c_int;

				if r_fullscreen.is_null() || (*r_fullscreen).integer == 0 {
					style = GetWindowLong(hWnd, -16); // GWL_STYLE
					AdjustWindowRect(&mut r, style as u32, 0);

					Cvar_SetValue(b"vid_xpos\0".as_ptr() as *const c_char, (xPos + r.left) as f32);
					Cvar_SetValue(b"vid_ypos\0".as_ptr() as *const c_char, (yPos + r.top) as f32);
					if !vid_xpos.is_null() {
						(*vid_xpos).modified = 0; // qfalse
					}
					if !vid_ypos.is_null() {
						(*vid_ypos).modified = 0; // qfalse
					}
					if (*core::ptr::addr_of_mut!(g_wv)).activeApp != 0 {
						IN_Activate(1); // qtrue
					}
				}
			}
		},

	// this is complicated because Win32 seems to pack multiple mouse events into
	// one update sometimes, so we always check all states and look for events
		0x0201 | 0x0202 | 0x0204 | 0x0205 | 0x0207 | 0x0208 | 0x0200 | 0x020B | 0x020C => {
			// WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MOUSEMOVE | WM_BUTTON4DOWN | WM_BUTTON4UP
			{
				let mut temp: c_int = 0;

				if (wParam & 0x0001) != 0 { // MK_LBUTTON
					temp |= 1;
				}

				if (wParam & 0x0002) != 0 { // MK_RBUTTON
					temp |= 2;
				}

				if (wParam & 0x0010) != 0 { // MK_MBUTTON
					temp |= 4;
				}

			 	if (wParam & MK_BUTTON4L) != 0 {
					temp |= 8;
				}

				if (wParam & MK_BUTTON4R) != 0 {
					temp |= 16;
				}

				IN_MouseEvent(temp);
			}
		},

		0x0112 => {  // WM_SYSCOMMAND
			if ((wParam & 0xFFF0) == 0xF140) || ((wParam & 0xFFF0) == 0xF170) { // SC_SCREENSAVE | SC_MONITORPOWER
				return 0;
			}
		},

		0x0104 => {  // WM_SYSKEYDOWN
			if wParam == 0x0D { // VK_RETURN
				if !r_fullscreen.is_null() && !cl_allowAltEnter.is_null() {
					// Check cls.state - we need a proper definition
					// For now, just check the conditions structurally
					if (*cl_allowAltEnter).integer != 0 {
						Cvar_SetValue(b"r_fullscreen\0".as_ptr() as *const c_char, if (*r_fullscreen).integer != 0 { 0.0 } else { 1.0 });
						Cbuf_AddText(b"vid_restart\n\0".as_ptr() as *const c_char);
					}
				}
				return 0;
			}
			// fall through
		},
		0x0100 => {  // WM_KEYDOWN
			code = MapKey(lParam as u32, wParam as u16) as u8;
			if code != 0 {
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, code as c_int, 1, 0, core::ptr::null_mut()); // SE_KEY, qtrue
			}
		},

		0x0105 => {  // WM_SYSKEYUP
		},
		0x0101 => {  // WM_KEYUP
			code = MapKey(lParam as u32, wParam as u16) as u8;
			if code != 0 {
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 1, code as c_int, 0, 0, core::ptr::null_mut()); // SE_KEY, qfalse
			}
		},

		0x0102 => {  // WM_CHAR
			if ((lParam >> 16) & 0xff) != CONSOLE_SCAN_CODE {
				Sys_QueEvent((*core::ptr::addr_of_mut!(g_wv)).sysMsgTime, 2, wParam as c_int, 0, 0, core::ptr::null_mut()); // SE_CHAR
			}
			// Output any debug prints
	//		if(in_debug && in_debug->integer & 2)
	//		{
	//			Com_Printf("WM_CHAR: %x\n", wParam);
	//		}
		},

		0x0218 => {  // WM_POWERBROADCAST
			if wParam == 0x0004 { // PBT_APMQUERYSUSPEND
				#[cfg(not(feature = "FINAL_BUILD"))]
				Com_DPrintf(b"Cannot go into hibernate / standby mode while game is running!\n\0".as_ptr() as *const c_char);
				return 0x425A4158; // BROADCAST_QUERY_DENY
			}
		},
		_ => {},
	   }

	    DefWindowProc(hWnd, uMsg, wParam, lParam as usize)
	}
}

// Placeholder for external symbols that need to be provided
extern "C" {
	pub static mut cl_allowAltEnter: *mut cvar_t;
	pub static mut cls: CLASS;
}
