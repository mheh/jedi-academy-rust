// leave this as first line for PCH reasons...
//
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint};

// Import type definitions from sibling modules
// Note: These would normally be in local headers included via #include,
// but we declare them via extern "C" for the faithful port.

#[repr(C)]
pub struct RECT {
    pub left: c_int,
    pub top: c_int,
    pub right: c_int,
    pub bottom: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: c_int, // qboolean
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
}

pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;

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
pub struct WinVars_t {
    pub hWnd: HWND,
    pub hInstance: HINSTANCE,
    pub activeApp: c_int, // qboolean
    pub isMinimized: c_int, // qboolean
    pub osversion: OSVERSIONINFO,
    pub sysMsgTime: c_uint,
}

// Extern declarations for globals and functions
extern "C" {
    pub static mut g_wv: WinVars_t;
    pub static mut ge: *mut c_void;
    pub static cls: WindowClass;

    fn Cvar_VariableString(name: *const c_char) -> *const c_char;
    fn RegisterHotKey(hWnd: HWND, id: c_int, fsModifiers: u32, vk: u32) -> c_int;
    fn UnregisterHotKey(hWnd: HWND, id: c_int) -> c_int;
    fn SystemParametersInfo(uiAction: u32, uiParam: u32, pvParam: *mut c_void, fWinIni: u32) -> c_int;
    fn Key_ClearStates();
    fn IN_Activate(fActive: c_int);
    fn MapVirtualKey(uCode: u32, uMapType: u32) -> u32;
    fn Sys_QueEvent(time: c_int, type_: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void);
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_SetValue(var_name: *const c_char, value: f32);
    fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
    fn Cbuf_AddText(text: *const c_char);
    fn SNDDMA_Activate(bAppActive: c_int);
    fn GetWindowLong(hWnd: HWND, nIndex: c_int) -> c_int;
    fn AdjustWindowRect(lpRect: *mut RECT, dwStyle: u32, bMenu: c_int) -> c_int;
    fn DefWindowProc(hWnd: HWND, uMsg: u32, wParam: usize, lParam: isize) -> isize;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

#[repr(C)]
pub struct WindowClass {
    pub keyCatchers: c_int,
}

// The only directly referenced keycode - the console key (which gives different ascii codes depending on locale)
const CONSOLE_SCAN_CODE: u32 = 0x29;

#[allow(non_upper_case_globals)]
static mut MSH_MOUSEWHEEL: u32 = 0;

// Console variables that we need to access from this module
#[allow(non_upper_case_globals)]
static mut vid_xpos: *mut cvar_t = core::ptr::null_mut(); // X coordinate of window position
#[allow(non_upper_case_globals)]
static mut vid_ypos: *mut cvar_t = core::ptr::null_mut(); // Y coordinate of window position
#[allow(non_upper_case_globals)]
static mut sr_fullscreen: *mut cvar_t = core::ptr::null_mut();

#[allow(non_upper_case_globals)]
static mut s_alttab_disabled: c_int = 0; // qboolean

unsafe fn WIN_DisableAltTab() {
    if s_alttab_disabled != 0 {
        return;
    }

    let arch_str = Cvar_VariableString(b"arch\0".as_ptr() as *const c_char);
    if Q_stricmp(arch_str, b"winnt\0".as_ptr() as *const c_char) == 0 {
        RegisterHotKey(core::ptr::null_mut(), 0, 1, 0x09); // MOD_ALT = 1, VK_TAB = 0x09
    } else {
        let mut old: c_int = 0;

        SystemParametersInfo(0x0011, 1, core::ptr::addr_of_mut!(old) as *mut c_void, 0); // SPI_SCREENSAVERRUNNING = 0x0011
    }
    s_alttab_disabled = 1; // qtrue
}

unsafe fn WIN_EnableAltTab() {
    if s_alttab_disabled != 0 {
        let arch_str = Cvar_VariableString(b"arch\0".as_ptr() as *const c_char);
        if Q_stricmp(arch_str, b"winnt\0".as_ptr() as *const c_char) == 0 {
            UnregisterHotKey(core::ptr::null_mut(), 0);
        } else {
            let mut old: c_int = 0;

            SystemParametersInfo(0x0011, 0, core::ptr::addr_of_mut!(old) as *mut c_void, 0); // SPI_SCREENSAVERRUNNING = 0x0011
        }

        s_alttab_disabled = 0; // qfalse
    }
}

/*
==================
VID_AppActivate
==================
*/
unsafe fn VID_AppActivate(fActive: c_int, minimize: c_int) {
    g_wv.isMinimized = minimize;

    Key_ClearStates(); // FIXME!!!

    // we don't want to act like we're active if we're minimized
    if fActive != 0 && g_wv.isMinimized == 0 {
        g_wv.activeApp = 1; // qtrue
    } else {
        g_wv.activeApp = 0; // qfalse
    }

    // minimize/restore mouse-capture on demand
    if g_wv.activeApp == 0 {
        IN_Activate(0); // qfalse
    } else {
        IN_Activate(1); // qtrue
    }
}

static virtualKeyConvert: [[u8; 2]; 0x92] = [
    [0, 0], // 00
    [1, 1], // VK_LBUTTON 01 Left mouse button
    [2, 2], // VK_RBUTTON 02 Right mouse button
    [0, 0], // VK_CANCEL 03 Control-break processing
    [3, 3], // VK_MBUTTON 04 Middle mouse button (three-button mouse)
    [4, 4], // VK_XBUTTON1 05 Windows 2000/XP: X1 mouse button
    [5, 5], // VK_XBUTTON2 06 Windows 2000/XP: X2 mouse button
    [0, 0], // 07 Undefined
    [8, 8], // VK_BACK 08 BACKSPACE key
    [9, 9], // VK_TAB 09 TAB key
    [0, 0], // 0A Reserved
    [0, 0], // 0B Reserved
    [12, 0], // VK_CLEAR 0C CLEAR key
    [13, 14], // VK_RETURN 0D ENTER key
    [0, 0], // 0E Undefined
    [0, 0], // 0F Undefined
    [16, 16], // VK_SHIFT 10 SHIFT key
    [17, 17], // VK_CONTROL 11 CTRL key
    [18, 18], // VK_MENU 12 ALT key
    [19, 19], // VK_PAUSE 13 PAUSE key
    [20, 20], // VK_CAPITAL 14 CAPS LOCK key
    [0, 0], // VK_KANA 15 IME Kana mode
    [0, 0], // 16 Undefined
    [0, 0], // VK_JUNJA 17 IME Junja mode
    [0, 0], // VK_FINAL 18 IME final mode
    [0, 0], // VK_KANJI 19 IME Kanji mode
    [0, 0], // 1A Undefined
    [27, 27], // VK_ESCAPE 1B ESC key
    [0, 0], // VK_CONVERT 1C IME convert
    [0, 0], // VK_NONCONVERT 1D IME nonconvert
    [0, 0], // VK_ACCEPT 1E IME accept
    [0, 0], // VK_MODECHANGE 1F IME mode change request
    [32, 32], // VK_SPACE 20 SPACEBAR
    [33, 34], // VK_PRIOR 21 PAGE UP key
    [35, 36], // VK_NEXT 22 PAGE DOWN key
    [37, 38], // VK_END 23 END key
    [39, 40], // VK_HOME 24 HOME key
    [41, 42], // VK_LEFT 25 LEFT ARROW key
    [43, 44], // VK_UP 26 UP ARROW key
    [45, 46], // VK_RIGHT 27 RIGHT ARROW key
    [47, 48], // VK_DOWN 28 DOWN ARROW key
    [0, 0], // VK_SELECT 29 SELECT key
    [0, 0], // VK_PRINT 2A PRINT key
    [0, 0], // VK_EXECUTE 2B EXECUTE key
    [49, 49], // VK_SNAPSHOT 2C PRINT SCREEN key
    [50, 51], // VK_INSERT 2D INS key
    [52, 53], // VK_DELETE 2E DEL key
    [0, 0], // VK_HELP 2F HELP key
    [48, 48], // 30 0 key
    [49, 49], // 31 1 key
    [50, 50], // 32 2 key
    [51, 51], // 33 3 key
    [52, 52], // 34 4 key
    [53, 53], // 35 5 key
    [54, 54], // 36 6 key
    [55, 55], // 37 7 key
    [56, 56], // 38 8 key
    [57, 57], // 39 9 key
    [0, 0], // 3A Undefined
    [0, 0], // 3B Undefined
    [0, 0], // 3C Undefined
    [0, 0], // 3D Undefined
    [0, 0], // 3E Undefined
    [0, 0], // 3F Undefined
    [0, 0], // 40 Undefined
    [65, 65], // 41 A key
    [66, 66], // 42 B key
    [67, 67], // 43 C key
    [68, 68], // 44 D key
    [69, 69], // 45 E key
    [70, 70], // 46 F key
    [71, 71], // 47 G key
    [72, 72], // 48 H key
    [73, 73], // 49 I key
    [74, 74], // 4A J key
    [75, 75], // 4B K key
    [76, 76], // 4C L key
    [77, 77], // 4D M key
    [78, 78], // 4E N key
    [79, 79], // 4F O key
    [80, 80], // 50 P key
    [81, 81], // 51 Q key
    [82, 82], // 52 R key
    [83, 83], // 53 S key
    [84, 84], // 54 T key
    [85, 85], // 55 U key
    [86, 86], // 56 V key
    [87, 87], // 57 W key
    [88, 88], // 58 X key
    [89, 89], // 59 Y key
    [90, 90], // 5A Z key
    [0, 0], // VK_LWIN 5B Left Windows key (Microsoft® Natural® keyboard)
    [0, 0], // VK_RWIN 5C Right Windows key (Natural keyboard)
    [0, 0], // VK_APPS 5D Applications key (Natural keyboard)
    [0, 0], // 5E Reserved
    [0, 0], // VK_SLEEP 5F Computer Sleep key
    [48, 48], // VK_NUMPAD0 60 Numeric keypad 0 key
    [49, 49], // VK_NUMPAD1 61 Numeric keypad 1 key
    [50, 50], // VK_NUMPAD2 62 Numeric keypad 2 key
    [51, 51], // VK_NUMPAD3 63 Numeric keypad 3 key
    [52, 52], // VK_NUMPAD4 64 Numeric keypad 4 key
    [53, 53], // VK_NUMPAD5 65 Numeric keypad 5 key
    [54, 54], // VK_NUMPAD6 66 Numeric keypad 6 key
    [55, 55], // VK_NUMPAD7 67 Numeric keypad 7 key
    [56, 56], // VK_NUMPAD8 68 Numeric keypad 8 key
    [57, 57], // VK_NUMPAD9 69 Numeric keypad 9 key
    [42, 42], // VK_MULTIPLY 6A Multiply key
    [43, 43], // VK_ADD 6B Add key
    [0, 0], // VK_SEPARATOR 6C Separator key
    [45, 45], // VK_SUBTRACT 6D Subtract key
    [46, 46], // VK_DECIMAL 6E Decimal key
    [47, 47], // VK_DIVIDE 6F Divide key
    [112, 112], // VK_F1 70 F1 key
    [113, 113], // VK_F2 71 F2 key
    [114, 114], // VK_F3 72 F3 key
    [115, 115], // VK_F4 73 F4 key
    [116, 116], // VK_F5 74 F5 key
    [117, 117], // VK_F6 75 F6 key
    [118, 118], // VK_F7 76 F7 key
    [119, 119], // VK_F8 77 F8 key
    [120, 120], // VK_F9 78 F9 key
    [121, 121], // VK_F10 79 F10 key
    [122, 122], // VK_F11 7A F11 key
    [123, 123], // VK_F12 7B F12 key
    [0, 0], // VK_F13 7C F13 key
    [0, 0], // VK_F14 7D F14 key
    [0, 0], // VK_F15 7E F15 key
    [0, 0], // VK_F16 7F F16 key
    [0, 0], // VK_F17 80H F17 key
    [0, 0], // VK_F18 81H F18 key
    [0, 0], // VK_F19 82H F19 key
    [0, 0], // VK_F20 83H F20 key
    [0, 0], // VK_F21 84H F21 key
    [0, 0], // VK_F22 85H F22 key
    [0, 0], // VK_F23 86H F23 key
    [0, 0], // VK_F24 87H F24 key
    [0, 0], // 88 Unassigned
    [0, 0], // 89 Unassigned
    [0, 0], // 8A Unassigned
    [0, 0], // 8B Unassigned
    [0, 0], // 8C Unassigned
    [0, 0], // 8D Unassigned
    [0, 0], // 8E Unassigned
    [0, 0], // 8F Unassigned
    [144, 144], // VK_NUMLOCK 90 NUM LOCK key
    [145, 145], // VK_SCROLL 91
];

/*
=======
MapKey

Map from windows to quake keynums
=======
*/
unsafe fn MapKey(key: u32, wParam: u32) -> c_int {
    let scan: u32;
    let extended: u32;

    // Check for the console key (hard code to the key you would expect)
    scan = (key >> 16) & 0xff;
    if scan == CONSOLE_SCAN_CODE {
        return 0x60; // A_CONSOLE
    }

    // Try to convert the virtual key directly
    let mut result: u32 = 0;
    extended = (key >> 24) & 1;
    if wParam > 0 && wParam <= 0x91 {
        // VK_SCROLL = 0x91
        result = virtualKeyConvert[wParam as usize][extended as usize] as u32;
    }
    // Get the unshifted ascii code (if any)
    if result == 0 {
        result = MapVirtualKey(wParam, 2) & 0xff;
    }
    // Output any debug prints
    // if(in_debug && in_debug->integer & 1)
    // {
    // Com_Printf("WM_KEY: %x : %x : %x\n", key, wParam, result);
    // }
    result as c_int
}

/*
====================
MainWndProc

main window procedure
====================
*/

#[no_mangle]
pub unsafe extern "C" fn MainWndProc(
    hWnd: HWND,
    uMsg: u32,
    wParam: usize,
    lParam: isize,
) -> isize {
    let code: c_int;

    // Handle custom mousewheel message
    if uMsg == MSH_MOUSEWHEEL {
        if (wParam as i32) > 0 {
            Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, 0x7a, 1, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELUP, qtrue
            Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, 0x7a, 0, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELUP, qfalse
        } else {
            Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, 0x7b, 1, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELDOWN, qtrue
            Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, 0x7b, 0, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELDOWN, qfalse
        }
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }

    match uMsg {
        0x020a => {
            // WM_MOUSEWHEEL
            //
            // this chunk of code theoretically only works under NT4 and Win98
            // since this message doesn't exist under Win95
            //
            if ((wParam >> 16) as i16) > 0 {
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, 0x7a, 1, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELUP, qtrue
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, 0x7a, 0, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELUP, qfalse
            } else {
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, 0x7b, 1, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELDOWN, qtrue
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, 0x7b, 0, 0, core::ptr::null_mut()); // SE_KEY, A_MWHEELDOWN, qfalse
            }
        }

        0x0001 => {
            // WM_CREATE

            g_wv.hWnd = hWnd;

            vid_xpos = Cvar_Get(b"vid_xpos\0".as_ptr() as *const c_char, b"3\0".as_ptr() as *const c_char, 1); // CVAR_ARCHIVE = 1
            vid_ypos = Cvar_Get(b"vid_ypos\0".as_ptr() as *const c_char, b"22\0".as_ptr() as *const c_char, 1);
            sr_fullscreen = Cvar_Get(b"r_fullscreen\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 1 | 32); // CVAR_ARCHIVE | CVAR_LATCH

            MSH_MOUSEWHEEL = RegisterWindowMessage(b"MSWHEEL_ROLLMSG\0".as_ptr());
            if (*sr_fullscreen).integer != 0 {
                WIN_DisableAltTab();
            } else {
                WIN_EnableAltTab();
            }
        }
        0x0002 => {
            // WM_DESTROY
            // let sound and input know about this?
            g_wv.hWnd = core::ptr::null_mut();
            if (*sr_fullscreen).integer != 0 {
                WIN_EnableAltTab();
            }
        }

        0x0010 => {
            // WM_CLOSE
            Cbuf_ExecuteText(0, b"quit\0".as_ptr() as *const c_char); // EXEC_APPEND = 0
        }

        0x0086 => {
            // WM_ACTIVATE
            let fActive = (wParam & 0xffff) as c_int;
            let fMinimized = ((wParam >> 16) & 0xffff) as c_int;

            VID_AppActivate(if fActive != 0 { 1 } else { 0 }, fMinimized); // WA_INACTIVE = 0
            SNDDMA_Activate(if fActive != 0 && fMinimized == 0 { 1 } else { 0 });
        }

        0x0003 => {
            // WM_MOVE
            let xPos: c_int = ((lParam & 0xffff) as i16) as c_int; // horizontal position
            let yPos: c_int = (((lParam >> 16) & 0xffff) as i16) as c_int; // vertical position

            if (*sr_fullscreen).integer == 0 {
                let mut r = RECT {
                    left: 0,
                    top: 0,
                    right: 1,
                    bottom: 1,
                };

                let style = GetWindowLong(hWnd, -16); // GWL_STYLE = -16

                AdjustWindowRect(core::ptr::addr_of_mut!(r), style as u32, 0);

                Cvar_SetValue(b"vid_xpos\0".as_ptr() as *const c_char, (xPos + r.left) as f32);
                Cvar_SetValue(b"vid_ypos\0".as_ptr() as *const c_char, (yPos + r.top) as f32);
                (*vid_xpos).modified = 0; // qfalse
                (*vid_ypos).modified = 0; // qfalse

                if g_wv.activeApp != 0 {
                    IN_Activate(1); // qtrue
                }
            }
        }

        // this is complicated because Win32 seems to pack multiple mouse events into
        // one update sometimes, so we always check all states and look for events
        0x0201 | 0x0202 | 0x0204 | 0x0205 | 0x0207 | 0x0208 | 0x0200 | 0x020b | 0x020c => {
            // WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MOUSEMOVE | WM_BUTTON4DOWN | WM_BUTTON4UP
            let mut temp: c_int = 0;

            if (wParam & 0x0001) != 0 {
                temp |= 1; // MK_LBUTTON
            }

            if (wParam & 0x0002) != 0 {
                temp |= 2; // MK_RBUTTON
            }

            if (wParam & 0x0010) != 0 {
                temp |= 4; // MK_MBUTTON
            }

            if (wParam & 0x0020) != 0 {
                temp |= 8; // MK_BUTTON4L
            }

            if (wParam & 0x0040) != 0 {
                temp |= 16; // MK_BUTTON4R
            }

            IN_MouseEvent(temp);
        }

        0x0112 => {
            // WM_SYSCOMMAND
            if ((wParam & 0xFFF0) == 0xF140) || ((wParam & 0xFFF0) == 0xF170) {
                // SC_SCREENSAVE | SC_MONITORPOWER
                return 0;
            }
        }

        0x0104 => {
            // WM_SYSKEYDOWN
            if wParam == 0x0d {
                // VK_RETURN - alt-enter
                if !sr_fullscreen.is_null() {
                    let allow_save = if ge.is_null() {
                        true
                    } else {
                        // ge->GameAllowedToSaveHere() && !(cls.keyCatchers&KEYCATCH_UI)
                        // For now, assume safe. This requires proper function binding
                        true
                    };
                    if allow_save {
                        // okay, don't switch if the game is running while in a cinematic or in the menu
                        Cvar_SetValue(b"r_fullscreen\0".as_ptr() as *const c_char, if (*sr_fullscreen).integer != 0 { 0.0 } else { 1.0 });
                        Cbuf_AddText(b"vid_restart\n\0".as_ptr() as *const c_char);
                    }
                }
                return 0;
            }
            // fall through to WM_KEYDOWN
            code = MapKey(lParam as u32, wParam as u32);
            if code != 0 {
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, code, 1, 0, core::ptr::null_mut()); // SE_KEY, qtrue
            }
        }

        0x0100 => {
            // WM_KEYDOWN
            code = MapKey(lParam as u32, wParam as u32);
            if code != 0 {
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, code, 1, 0, core::ptr::null_mut()); // SE_KEY, qtrue
            }
        }

        0x0105 => {
            // WM_SYSKEYUP
            code = MapKey(lParam as u32, wParam as u32);
            if code != 0 {
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, code, 0, 0, core::ptr::null_mut()); // SE_KEY, qfalse
            }
        }

        0x0101 => {
            // WM_KEYUP
            code = MapKey(lParam as u32, wParam as u32);
            if code != 0 {
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 0, code, 0, 0, core::ptr::null_mut()); // SE_KEY, qfalse
            }
        }

        0x0102 => {
            // WM_CHAR
            if ((lParam >> 16) & 0xff) != CONSOLE_SCAN_CODE {
                Sys_QueEvent(g_wv.sysMsgTime as c_int, 1, wParam as c_int, 0, 0, core::ptr::null_mut()); // SE_CHAR
            }
            // Output any debug prints
            // if(in_debug && in_debug->integer & 2)
            // {
            // Com_Printf("WM_CHAR: %x\n", wParam);
            // }
        }

        0x0218 => {
            // WM_POWERBROADCAST
            if wParam == 0x0004 {
                // PBT_APMQUERYSUSPEND
                #[cfg(not(feature = "FINAL_BUILD"))]
                {
                    // Com_Printf("Cannot go into hibernate / standby mode while game is running!\n");
                }
                return 0x00000000; // BROADCAST_QUERY_DENY
            }
        }

        _ => {}
    }

    DefWindowProc(hWnd, uMsg, wParam, lParam)
}

// Stub for RegisterWindowMessage
unsafe fn RegisterWindowMessage(lpString: *const u8) -> u32 {
    // This is a Windows API stub. In a real implementation, this would call the actual Windows API.
    // For now, we just return a placeholder value.
    1024
}
