// win_input.c -- win32 mouse and joystick code
// 02/21/97 JCB Added extended DirectInput code to support external controllers.
// Anything above this #include will be ignored by the compiler

use core::ffi::{c_int, c_uint, c_char};
use core::mem;

// Stub declarations - these would normally come from included headers
// #include "../qcommon/exe_headers.h"
// #include "../client/client.h"
// #include "win_local.h"

// Local type stubs for structural coherence
type qboolean = bool;
type cvar_t = u8;  // Stub

#[repr(C)]
struct WinMouseVars_t {
    oldButtonState: c_int,
    mouseActive: qboolean,
    mouseInitialized: qboolean,
}

static mut s_wmv: WinMouseVars_t = WinMouseVars_t {
    oldButtonState: 0,
    mouseActive: false,
    mouseInitialized: false,
};

static mut window_center_x: c_int = 0;
static mut window_center_y: c_int = 0;

//
// MIDI definitions
//
static fn IN_StartupMIDI();
static fn IN_ShutdownMIDI();

const MAX_MIDIIN_DEVICES: usize = 8;

#[repr(C)]
struct MidiInfo_t {
    numDevices: c_int,
    caps: [u8; MAX_MIDIIN_DEVICES],  // MIDIINCAPS - stub
    hMidiIn: *mut core::ffi::c_void,  // HMIDIIN
}

static mut s_midiInfo: MidiInfo_t = MidiInfo_t {
    numDevices: 0,
    caps: [0u8; MAX_MIDIIN_DEVICES],
    hMidiIn: core::ptr::null_mut(),
};

//
// Joystick definitions
//
const JOY_MAX_AXES: usize = 6;  // X, Y, Z, R, U, V

#[repr(C)]
struct joystickInfo_t {
    avail: qboolean,
    id: c_int,  // joystick number
    jc: [u8; 1],  // JOYCAPS - stub
    oldbuttonstate: c_int,
    oldpovstate: c_int,
    ji: [u8; 1],  // JOYINFOEX - stub
}

static mut joy: joystickInfo_t = joystickInfo_t {
    avail: false,
    id: 0,
    jc: [0u8; 1],
    oldbuttonstate: 0,
    oldpovstate: 0,
    ji: [0u8; 1],
};

static mut in_midi: *mut cvar_t = core::ptr::null_mut();
static mut in_midiport: *mut cvar_t = core::ptr::null_mut();
static mut in_midichannel: *mut cvar_t = core::ptr::null_mut();
static mut in_mididevice: *mut cvar_t = core::ptr::null_mut();

static mut in_mouse: *mut cvar_t = core::ptr::null_mut();
static mut in_joystick: *mut cvar_t = core::ptr::null_mut();
static mut in_joyBallScale: *mut cvar_t = core::ptr::null_mut();
static mut in_debugJoystick: *mut cvar_t = core::ptr::null_mut();
static mut joy_threshold: *mut cvar_t = core::ptr::null_mut();
static mut joy_xbutton: *mut cvar_t = core::ptr::null_mut();
static mut joy_ybutton: *mut cvar_t = core::ptr::null_mut();

static mut in_appactive: qboolean = false;

// forward-referenced functions
extern "C" {
    fn IN_StartupJoystick();
    fn IN_JoyMove();
    fn Com_Printf(fmt: *const c_char, ...);
    fn GetSystemMetrics(nIndex: c_int) -> c_int;
    fn GetWindowRect(hWnd: *mut core::ffi::c_void, lpRect: *mut core::ffi::c_void) -> c_int;
    fn SetCursorPos(X: c_int, Y: c_int) -> c_int;
    fn SetCapture(hWnd: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn ClipCursor(lpRect: *const core::ffi::c_void) -> c_int;
    fn ShowCursor(bShow: c_int) -> c_int;
    fn GetCursorPos(lpPoint: *mut core::ffi::c_void) -> c_int;
    fn ReleaseCapture() -> c_int;
    fn Sys_QueEvent(time: c_uint, type_: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut core::ffi::c_void);
    fn LoadLibrary(lpLibFileName: *const c_char) -> *mut core::ffi::c_void;
    fn GetProcAddress(hModule: *mut core::ffi::c_void, lpProcName: *const c_char) -> *mut core::ffi::c_void;
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cmd_AddCommand(cmd_name: *const c_char, fn_: *const core::ffi::c_void);
    fn Cmd_RemoveCommand(cmd_name: *const c_char);
    fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    fn Com_Memset(ptr: *mut core::ffi::c_void, c: c_int, count: usize);
    fn joyGetNumDevs() -> c_uint;
    fn joyGetPosEx(uJoyID: c_uint, pji: *mut core::ffi::c_void) -> c_uint;
    fn joyGetDevCaps(uJoyID: c_uint, pjc: *mut core::ffi::c_void, cbjc: c_uint) -> c_uint;
    fn memset(ptr: *mut core::ffi::c_void, c: c_int, count: usize);
    fn midiInGetNumDevs() -> c_uint;
    fn midiInGetDevCaps(uDeviceID: c_uint, pMidiInCaps: *mut core::ffi::c_void, cbMidiInCaps: c_uint) -> c_uint;
    fn midiInOpen(phMidiIn: *mut *mut core::ffi::c_void, uDeviceID: c_uint, dwCallback: c_uint, dwInstance: c_uint, fdwOpen: c_uint) -> c_uint;
    fn midiInStart(hMidiIn: *mut core::ffi::c_void) -> c_uint;
    fn midiInClose(hMidiIn: *mut core::ffi::c_void) -> c_uint;
}

static fn MidiInfo_f();

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
pub extern "C" fn IN_InitWin32Mouse() {
}

/*
================
IN_ShutdownWin32Mouse
================
*/
pub extern "C" fn IN_ShutdownWin32Mouse() {
}

/*
================
IN_ActivateWin32Mouse
================
*/
pub extern "C" fn IN_ActivateWin32Mouse() {
    let mut width: c_int;
    let mut height: c_int;
    let mut window_rect: [u8; 1] = [0u8; 1];  // RECT - stub

    const SM_CXSCREEN: c_int = 0;
    const SM_CYSCREEN: c_int = 1;

    width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    unsafe {
        GetWindowRect(core::ptr::null_mut(), core::ptr::addr_of_mut!(window_rect) as *mut core::ffi::c_void);
        // if (window_rect.left < 0)
        //     window_rect.left = 0;
        // if (window_rect.top < 0)
        //     window_rect.top = 0;
        // if (window_rect.right >= width)
        //     window_rect.right = width-1;
        // if (window_rect.bottom >= height-1)
        //     window_rect.bottom = height-1;
        // window_center_x = (window_rect.right + window_rect.left)/2;
        // window_center_y = (window_rect.top + window_rect.bottom)/2;

        // SetCursorPos (window_center_x, window_center_y);

        SetCapture(core::ptr::null_mut());
        ClipCursor(core::ptr::addr_of!(window_rect) as *const core::ffi::c_void);
        while ShowCursor(0) >= 0 {
        }
    }
}

/*
================
IN_DeactivateWin32Mouse
================
*/
pub extern "C" fn IN_DeactivateWin32Mouse() {
    unsafe {
        ClipCursor(core::ptr::null());
        ReleaseCapture();
        while ShowCursor(1) < 0 {
        }
    }
}

/*
================
IN_Win32Mouse
================
*/
pub extern "C" fn IN_Win32Mouse(mx: *mut c_int, my: *mut c_int) {
    let mut current_pos: [u8; 1] = [0u8; 1];  // POINT - stub

    // find mouse movement
    unsafe {
        GetCursorPos(core::ptr::addr_of_mut!(current_pos) as *mut core::ffi::c_void);

        // force the mouse to the center, so there's room to move
        SetCursorPos(window_center_x, window_center_y);

        // *mx = current_pos.x - window_center_x;
        // *my = current_pos.y - window_center_y;
    }
}


/*
============================================================

DIRECT INPUT MOUSE CONTROL

============================================================
*/

// GUID definitions - preserved as stubs
const GUID_SysMouse: u128 = 0;
const GUID_XAxis: u128 = 1;
const GUID_YAxis: u128 = 2;
const GUID_ZAxis: u128 = 3;

const DINPUT_BUFFERSIZE: c_int = 16;

// Stub for DirectInputCreate function pointer
static mut pDirectInputCreate: Option<extern "C" fn(*mut core::ffi::c_void, c_uint, *mut *mut core::ffi::c_void, *mut core::ffi::c_void) -> c_uint> = None;

static mut hInstDI: *mut core::ffi::c_void = core::ptr::null_mut();

#[repr(C)]
struct MYDATA {
    lX: c_int,      // X axis goes here
    lY: c_int,      // Y axis goes here
    lZ: c_int,      // Z axis goes here
    bButtonA: u8,   // One button goes here
    bButtonB: u8,   // Another button goes here
    bButtonC: u8,   // Another button goes here
    bButtonD: u8,   // Another button goes here
}

// DIOBJECTDATAFORMAT stub
#[repr(C)]
struct DIOBJECTDATAFORMAT {
    guid: *const u128,
    dwOfs: c_uint,
    dwType: c_uint,
    dwFlags: c_uint,
}

static rgodf: [DIOBJECTDATAFORMAT; 7] = [
    DIOBJECTDATAFORMAT { guid: core::ptr::null(), dwOfs: 0, dwType: 0, dwFlags: 0 },
    DIOBJECTDATAFORMAT { guid: core::ptr::null(), dwOfs: 4, dwType: 0, dwFlags: 0 },
    DIOBJECTDATAFORMAT { guid: core::ptr::null(), dwOfs: 8, dwType: 0, dwFlags: 0x80000000 },
    DIOBJECTDATAFORMAT { guid: core::ptr::null(), dwOfs: 12, dwType: 0, dwFlags: 0 },
    DIOBJECTDATAFORMAT { guid: core::ptr::null(), dwOfs: 13, dwType: 0, dwFlags: 0 },
    DIOBJECTDATAFORMAT { guid: core::ptr::null(), dwOfs: 14, dwType: 0, dwFlags: 0x80000000 },
    DIOBJECTDATAFORMAT { guid: core::ptr::null(), dwOfs: 15, dwType: 0, dwFlags: 0x80000000 },
];

const NUM_OBJECTS: usize = 7;

#[repr(C)]
struct DIDATAFORMAT {
    dwSize: c_uint,
    dwObjSize: c_uint,
    dwFlags: c_uint,
    dwDataSize: c_uint,
    dwNumObjs: c_uint,
    rgodf: *const DIOBJECTDATAFORMAT,
}

static df: DIDATAFORMAT = DIDATAFORMAT {
    dwSize: 0,
    dwObjSize: 0,
    dwFlags: 0,
    dwDataSize: 0,
    dwNumObjs: NUM_OBJECTS as c_uint,
    rgodf: core::ptr::null(),
};

static mut g_pdi: *mut core::ffi::c_void = core::ptr::null_mut();
static mut g_pMouse: *mut core::ffi::c_void = core::ptr::null_mut();

extern "C" {
    fn IN_DIMouse(mx: *mut c_int, my: *mut c_int);
}

/*
========================
IN_InitDIMouse
========================
*/
pub extern "C" fn IN_InitDIMouse() -> qboolean {
    let mut hr: c_int;
    let mut x: c_int = 0;
    let mut y: c_int = 0;

    // DIPROPDWORD dipdw struct stub

    unsafe {
        Com_Printf(b"Initializing DirectInput...\n\0".as_ptr() as *const c_char);

        if hInstDI.is_null() {
            hInstDI = LoadLibrary(b"dinput.dll\0".as_ptr() as *const c_char);

            if hInstDI.is_null() {
                Com_Printf(b"Couldn't load dinput.dll\n\0".as_ptr() as *const c_char);
                return false;
            }
        }

        if pDirectInputCreate.is_none() {
            pDirectInputCreate = Some(core::mem::transmute(
                GetProcAddress(hInstDI, b"DirectInputCreateA\0".as_ptr() as *const c_char)
            ));

            if pDirectInputCreate.is_none() {
                Com_Printf(b"Couldn't get DI proc addr\n\0".as_ptr() as *const c_char);
                return false;
            }
        }

        // register with DirectInput and get an IDirectInput to play with.
        if let Some(create_fn) = pDirectInputCreate {
            hr = create_fn(core::ptr::null_mut(), 0x0700, core::ptr::addr_of_mut!(g_pdi), core::ptr::null_mut()) as c_int;
        } else {
            return false;
        }

        if hr != 0 {
            Com_Printf(b"iDirectInputCreate failed\n\0".as_ptr() as *const c_char);
            return false;
        }

        // obtain an interface to the system mouse device.
        // hr = IDirectInput_CreateDevice(g_pdi, GUID_SysMouse, &g_pMouse, NULL);

        // if (FAILED(hr)) {
        //     Com_Printf ("Couldn't open DI mouse device\n");
        //     return qfalse;
        // }

        // set the data format to "mouse format".
        // hr = IDirectInputDevice_SetDataFormat(g_pMouse, &df);

        // if (FAILED(hr)) 	{
        //     Com_Printf ("Couldn't set DI mouse format\n");
        //     return qfalse;
        // }

        // set the cooperativity level.
        // hr = IDirectInputDevice_SetCooperativeLevel(g_pMouse, g_wv.hWnd,
        //         DISCL_EXCLUSIVE | DISCL_FOREGROUND);

        // if (FAILED(hr)) {
        //     Com_Printf ("Couldn't set DI coop level\n");
        //     return qfalse;
        // }

        // set the buffer size to DINPUT_BUFFERSIZE elements.
        // the buffer size is a DWORD property associated with the device
        // hr = IDirectInputDevice_SetProperty(g_pMouse, DIPROP_BUFFERSIZE, &dipdw.diph);

        // if (FAILED(hr)) {
        //     Com_Printf ("Couldn't set DI buffersize\n");
        //     return qfalse;
        // }

        // clear any pending samples
        IN_DIMouse(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(y));
        IN_DIMouse(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(y));

        Com_Printf(b"DirectInput initialized.\n\0".as_ptr() as *const c_char);
        true
    }
}

/*
==========================
IN_ShutdownDIMouse
==========================
*/
pub extern "C" fn IN_ShutdownDIMouse() {
    unsafe {
        if !g_pMouse.is_null() {
            // IDirectInputDevice_Release(g_pMouse);
            g_pMouse = core::ptr::null_mut();
        }

        if !g_pdi.is_null() {
            // IDirectInput_Release(g_pdi);
            g_pdi = core::ptr::null_mut();
        }
    }
}

/*
==========================
IN_ActivateDIMouse
==========================
*/
pub extern "C" fn IN_ActivateDIMouse() {
    let mut hr: c_int;

    unsafe {
        if g_pMouse.is_null() {
            return;
        }

        // we may fail to reacquire if the window has been recreated
        // hr = IDirectInputDevice_Acquire( g_pMouse );
        if hr != 0 {
            if !IN_InitDIMouse() {
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
pub extern "C" fn IN_DeactivateDIMouse() {
    unsafe {
        if !g_pMouse.is_null() {
            return;
        }
        // IDirectInputDevice_Unacquire( g_pMouse );
    }
}


/*
===================
IN_DIMouse
===================
*/
pub extern "C" fn IN_DIMouse(mx: *mut c_int, my: *mut c_int) {
    // DIDEVICEOBJECTDATA od;
    // DIMOUSESTATE state;
    let mut dwElements: c_uint = 0;
    let mut hr: c_int = 0;
    static mut oldSysTime: f32 = 0.0;

    unsafe {
        if g_pMouse.is_null() {
            return;
        }

        // fetch new events
        loop {
            dwElements = 1;

            // hr = IDirectInputDevice_GetDeviceData(g_pMouse,
            //         sizeof(DIDEVICEOBJECTDATA), &od, &dwElements, 0);
            // if ((hr == DIERR_INPUTLOST) || (hr == DIERR_NOTACQUIRED)) {
            //     IDirectInputDevice_Acquire(g_pMouse);
            //     return;
            // }

            /* Unable to read data or no data available */
            if hr != 0 {
                break;
            }

            if dwElements == 0 {
                break;
            }

            // switch (od.dwOfs) {
            // case DIMOFS_BUTTON0:
            //     if (od.dwData & 0x80)
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE1, qtrue, 0, NULL );
            //     else
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE1, qfalse, 0, NULL );
            //     break;

            // case DIMOFS_BUTTON1:
            //     if (od.dwData & 0x80)
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE2, qtrue, 0, NULL );
            //     else
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE2, qfalse, 0, NULL );
            //     break;

            // case DIMOFS_BUTTON2:
            //     if (od.dwData & 0x80)
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE3, qtrue, 0, NULL );
            //     else
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE3, qfalse, 0, NULL );
            //     break;

            // case DIMOFS_BUTTON3:
            //     if (od.dwData & 0x80)
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE4, qtrue, 0, NULL );
            //     else
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE4, qfalse, 0, NULL );
            //     break;

            // // needs DIRECTINPUT_VERSION >= 0x0700 to compile, which we seem to have, so...
            // //
            // case DIMOFS_BUTTON4:
            //     if (od.dwData & 0x80)
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE5, qtrue, 0, NULL );
            //     else
            //         Sys_QueEvent( od.dwTimeStamp, SE_KEY, A_MOUSE5, qfalse, 0, NULL );
            //     break;
            // }
        }

        // read the raw delta counter and ignore
        // the individual sample time / values
        // hr = IDirectInputDevice_GetDeviceState(g_pMouse,
        //         sizeof(DIDEVICEOBJECTDATA), &state);
        if hr != 0 {
            *mx = 0;
            *my = 0;
            return;
        }
        // *mx = state.lX;
        // *my = state.lY;
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
pub extern "C" fn IN_ActivateMouse() {
    unsafe {
        if !s_wmv.mouseInitialized {
            return;
        }
        if *(core::ptr::addr_of!(in_mouse) as *const *const cvar_t) as *const u8 as usize == 0 {
            s_wmv.mouseActive = false;
            return;
        }
        if s_wmv.mouseActive {
            return;
        }

        s_wmv.mouseActive = true;

        if *(core::ptr::addr_of!(in_mouse) as *const *const cvar_t) as *const u8 as usize != !0 {
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
pub extern "C" fn IN_DeactivateMouse() {
    unsafe {
        if !s_wmv.mouseInitialized {
            return;
        }
        if !s_wmv.mouseActive {
            return;
        }
        s_wmv.mouseActive = false;

        IN_DeactivateDIMouse();
        IN_DeactivateWin32Mouse();
    }
}



/*
===========
IN_StartupMouse
===========
*/
pub extern "C" fn IN_StartupMouse() {
    unsafe {
        s_wmv.mouseInitialized = false;

        if *(core::ptr::addr_of!(in_mouse) as *const *const cvar_t) as *const u8 as usize == 0 {
            Com_Printf(b"Mouse control not active.\n\0".as_ptr() as *const c_char);
            return;
        }

        // nt4.0 direct input is screwed up
        // if ( ( g_wv.osversion.dwPlatformId == VER_PLATFORM_WIN32_NT ) &&
        //      ( g_wv.osversion.dwMajorVersion == 4 ) )
        // {
        //     Com_Printf ("Disallowing DirectInput on NT 4.0\n");
        //     Cvar_Set( "in_mouse", "-1" );
        // }

        s_wmv.mouseInitialized = true;

        if *(core::ptr::addr_of!(in_mouse) as *const *const cvar_t) as *const u8 as usize == !0 {
            Com_Printf(b"Skipping check for DirectInput\n\0".as_ptr() as *const c_char);
        } else {
            if IN_InitDIMouse() {
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
    0,  // A_MOUSE1
    1,  // A_MOUSE2
    2,  // A_MOUSE3
    3,  // A_MOUSE4
    4,  // A_MOUSE5
];

pub extern "C" fn IN_MouseEvent(mstate: c_int) {
    let mut i: c_int;

    unsafe {
        if !s_wmv.mouseInitialized {
            return;
        }

        // perform button actions
        i = 0;
        while i < MAX_MOUSE_BUTTONS as c_int {
            if ((mstate & (1 << i)) != 0) && ((s_wmv.oldButtonState & (1 << i)) == 0) {
                Sys_QueEvent(0, 0, mouseConvert[i as usize], 1, 0, core::ptr::null_mut());
            }
            if ((mstate & (1 << i)) == 0) && ((s_wmv.oldButtonState & (1 << i)) != 0) {
                Sys_QueEvent(0, 0, mouseConvert[i as usize], 0, 0, core::ptr::null_mut());
            }
            i += 1;
        }
        s_wmv.oldButtonState = mstate;
    }
}


/*
===========
IN_MouseMove
===========
*/
pub extern "C" fn IN_MouseMove() {
    let mut mx: c_int = 0;
    let mut my: c_int = 0;

    unsafe {
        if !g_pMouse.is_null() {
            IN_DIMouse(core::ptr::addr_of_mut!(mx), core::ptr::addr_of_mut!(my));
        } else {
            IN_Win32Mouse(core::ptr::addr_of_mut!(mx), core::ptr::addr_of_mut!(my));
        }

        if mx == 0 && my == 0 {
            return;
        }

        Sys_QueEvent(0, 0, mx, my, 0, core::ptr::null_mut());
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
pub extern "C" fn IN_Startup() {
    unsafe {
        Com_Printf(b"\n------- Input Initialization -------\n\0".as_ptr() as *const c_char);
        IN_StartupMouse();
        IN_StartupJoystick();
        IN_StartupMIDI();
        Com_Printf(b"------------------------------------\n\0".as_ptr() as *const c_char);

        // in_mouse->modified = qfalse;
        // in_joystick->modified = qfalse;
    }
}

/*
===========
IN_Shutdown
===========
*/
pub extern "C" fn IN_Shutdown() {
    unsafe {
        IN_DeactivateMouse();
        IN_ShutdownDIMouse();
        IN_ShutdownMIDI();
        Cmd_RemoveCommand(b"midiinfo\0".as_ptr() as *const c_char);
    }
}


/*
===========
IN_Init
===========
*/
pub extern "C" fn IN_Init() {
    unsafe {
        // MIDI input controler variables
        in_midi = Cvar_Get(b"in_midi\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        in_midiport = Cvar_Get(b"in_midiport\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
        in_midichannel = Cvar_Get(b"in_midichannel\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
        in_mididevice = Cvar_Get(b"in_mididevice\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

        Cmd_AddCommand(b"midiinfo\0".as_ptr() as *const c_char, MidiInfo_f as *const core::ffi::c_void);

        // mouse variables
        in_mouse = Cvar_Get(b"in_mouse\0".as_ptr() as *const c_char, b"-1\0".as_ptr() as *const c_char, 0);

        // joystick variables
        in_joystick = Cvar_Get(b"in_joystick\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        in_joyBallScale = Cvar_Get(b"in_joyBallScale\0".as_ptr() as *const c_char, b"0.02\0".as_ptr() as *const c_char, 0);
        in_debugJoystick = Cvar_Get(b"in_debugjoystick\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

        joy_threshold = Cvar_Get(b"joy_threshold\0".as_ptr() as *const c_char, b"0.15\0".as_ptr() as *const c_char, 0);

        joy_xbutton = Cvar_Get(b"joy_xbutton\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);  // treat axis as a button
        joy_ybutton = Cvar_Get(b"joy_ybutton\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);  // treat axis as a button

        IN_Startup();
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
pub extern "C" fn IN_Activate(active: qboolean) {
    unsafe {
        in_appactive = active;

        if !active {
            IN_DeactivateMouse();
        }
    }
}

extern "C" {
    static mut r_fullscreen: *mut cvar_t;
}

/*
==================
IN_Frame

Called every frame, even if not generating commands
==================
*/
pub extern "C" fn IN_Frame() {
    // post joystick events
    unsafe {
        IN_JoyMove();

        if !s_wmv.mouseInitialized {
            return;
        }

        // if ( cls.keyCatchers & KEYCATCH_CONSOLE ) {
        //     // temporarily deactivate if not in the game and
        //     // running on the desktop
        //     if (r_fullscreen && r_fullscreen->value == 0 )	{
        //         IN_DeactivateMouse ();
        //         return;
        //     }
        // }

        if !in_appactive {
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
pub extern "C" fn IN_ClearStates() {
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
pub extern "C" fn IN_StartupJoystick() {
    let mut numdevs: c_uint;
    let mut mmr: c_uint;

    unsafe {
        // assume no joystick
        joy.avail = false;

        if !(*(core::ptr::addr_of!(in_joystick) as *const *const cvar_t) as *const u8 as usize != 0) {
            Com_Printf(b"Joystick is not active.\n\0".as_ptr() as *const c_char);
            return;
        }

        // verify joystick driver is present
        if (numdevs = joyGetNumDevs()) == 0 {
            Com_Printf(b"joystick not found -- driver not present\n\0".as_ptr() as *const c_char);
            return;
        }

        // cycle through the joystick ids for the first valid one
        mmr = 0;
        joy.id = 0;
        while joy.id < numdevs as c_int {
            Com_Memset(core::ptr::addr_of_mut!(joy.ji) as *mut core::ffi::c_void, 0, mem::size_of_val(&joy.ji));
            // joy.ji.dwSize = sizeof(joy.ji);
            // joy.ji.dwFlags = JOY_RETURNCENTERED;

            if (mmr = joyGetPosEx(joy.id as c_uint, core::ptr::addr_of_mut!(joy.ji) as *mut core::ffi::c_void)) == 0 {
                break;
            }
            joy.id += 1;
        }

        // abort startup if we didn't find a valid joystick
        if mmr != 0 {
            Com_Printf(b"joystick not found -- no valid joysticks (%x)\n\0".as_ptr() as *const c_char, mmr);
            return;
        }

        // get the capabilities of the selected joystick
        // abort startup if command fails
        Com_Memset(core::ptr::addr_of_mut!(joy.jc) as *mut core::ffi::c_void, 0, mem::size_of_val(&joy.jc));
        if (mmr = joyGetDevCaps(joy.id as c_uint, core::ptr::addr_of_mut!(joy.jc) as *mut core::ffi::c_void, mem::size_of_val(&joy.jc) as c_uint)) != 0 {
            Com_Printf(b"joystick not found -- invalid joystick capabilities (%x)\n\0".as_ptr() as *const c_char, mmr);
            return;
        }

        Com_Printf(b"Joystick found.\n\0".as_ptr() as *const c_char);
        // Com_Printf( "Pname: %s\n", joy.jc.szPname );
        // Com_Printf( "OemVxD: %s\n", joy.jc.szOEMVxD );
        // Com_Printf( "RegKey: %s\n", joy.jc.szRegKey );

        // Com_Printf( "Numbuttons: %i / %i\n", joy.jc.wNumButtons, joy.jc.wMaxButtons );
        // Com_Printf( "Axis: %i / %i\n", joy.jc.wNumAxes, joy.jc.wMaxAxes );
        // Com_Printf( "Caps: 0x%x\n", joy.jc.wCaps );
        // if ( joy.jc.wCaps & JOYCAPS_HASPOV ) {
        //     Com_Printf( "HASPOV\n" );
        // } else {
        //     Com_Printf( "no POV\n" );
        // }

        // old button and POV states default to no buttons pressed
        joy.oldbuttonstate = 0;
        joy.oldpovstate = 0;

        // mark the joystick as available
        joy.avail = true;
    }
}

/*
===========
JoyToF
===========
*/
fn JoyToF(value: c_int) -> f32 {
    let mut fValue: f32;
    let mut val = value;

    // move centerpoint to zero
    val -= 32768;

    // convert range from -32768..32767 to -1..1
    fValue = val as f32 / 32768.0;

    if fValue < -1.0 {
        fValue = -1.0;
    }
    if fValue > 1.0 {
        fValue = 1.0;
    }
    fValue
}

fn JoyToI(value: c_int) -> c_int {
    // move centerpoint to zero
    value - 32768
}

static joyDirectionKeys: [c_int; 16] = [
    0,  // A_CURSOR_LEFT
    1,  // A_CURSOR_RIGHT
    2,  // A_CURSOR_UP
    3,  // A_CURSOR_DOWN
    4,  // A_JOY16
    5,  // A_JOY17
    6,  // A_JOY18
    7,  // A_JOY19
    8,  // A_JOY20
    9,  // A_JOY21
    10, // A_JOY22
    11, // A_JOY23
    12, // A_JOY24
    13, // A_JOY25
    14, // A_JOY26
    15, // A_JOY27
];

/*
===========
IN_JoyMove
===========
*/
pub extern "C" fn IN_JoyMove() {
    let mut fAxisValue: f32;
    let mut i: c_int;
    let mut buttonstate: c_uint;
    let mut povstate: c_uint;
    let mut x: c_int;
    let mut y: c_int;

    unsafe {
        // verify joystick is available and that the user wants to use it
        if !joy.avail {
            return;
        }

        // collect the joystick data, if possible
        memset(core::ptr::addr_of_mut!(joy.ji) as *mut core::ffi::c_void, 0, mem::size_of_val(&joy.ji));
        // joy.ji.dwSize = sizeof(joy.ji);
        // joy.ji.dwFlags = JOY_RETURNALL;

        if joyGetPosEx(joy.id as c_uint, core::ptr::addr_of_mut!(joy.ji) as *mut core::ffi::c_void) != 0 {
            // read error occurred
            // turning off the joystick seems too harsh for 1 read error,
            // but what should be done?
            // Com_Printf ("IN_ReadJoystick: no response\n");
            // joy.avail = false;
            return;
        }

        if *(core::ptr::addr_of!(in_debugJoystick) as *const *const cvar_t) as *const u8 as usize != 0 {
            // Com_Printf( "%8x %5i %5.2f %5.2f %5.2f %5.2f %6i %6i\n",
            //     joy.ji.dwButtons,
            //     joy.ji.dwPOV,
            //     JoyToF( joy.ji.dwXpos ), JoyToF( joy.ji.dwYpos ),
            //     JoyToF( joy.ji.dwZpos ), JoyToF( joy.ji.dwRpos ),
            //     JoyToI( joy.ji.dwUpos ), JoyToI( joy.ji.dwVpos ) );
        }

        // loop through the joystick buttons
        // key a joystick event or auxillary event for higher number buttons for each state change
        // buttonstate = joy.ji.dwButtons;
        buttonstate = 0;
        i = 0;
        // while i < joy.jc.wNumButtons {
        while i < 32 {
            if ((buttonstate & ((1 as c_uint) << i)) != 0) && ((joy.oldbuttonstate & (1 << i)) == 0) {
                Sys_QueEvent(0, 0, 0 + i, 1, 0, core::ptr::null_mut());  // A_JOY0 + i
            }
            if ((buttonstate & ((1 as c_uint) << i)) == 0) && ((joy.oldbuttonstate & (1 << i)) != 0) {
                Sys_QueEvent(0, 0, 0 + i, 0, 0, core::ptr::null_mut());  // A_JOY0 + i
            }
            i += 1;
        }
        joy.oldbuttonstate = buttonstate as c_int;

        povstate = 0;

        // convert main joystick motion into 6 direction button bits
        i = 0;
        // while i < joy.jc.wNumAxes && i < 4 {
        while i < 4 {
            // get the floating point zero-centered, potentially-inverted data for the current axis
            // fAxisValue = JoyToF( (&joy.ji.dwXpos)[i] );
            fAxisValue = 0.0;

            if i == 0 && !(*(core::ptr::addr_of!(joy_xbutton) as *const *const cvar_t) as *const u8 as usize != 0) {
                if (fAxisValue < -(*(core::ptr::addr_of!(joy_threshold) as *const *const cvar_t) as *const u8 as usize as f32)) || (fAxisValue > *(core::ptr::addr_of!(joy_threshold) as *const *const cvar_t) as *const u8 as usize as f32) {
                    Sys_QueEvent(0, 0, 0, (-(fAxisValue * 127.0)) as c_int, 0, core::ptr::null_mut());  // AXIS_SIDE
                } else {
                    Sys_QueEvent(0, 0, 0, 0, 0, core::ptr::null_mut());  // AXIS_SIDE
                }
                i += 1;
                continue;
            }

            if i == 1 && !(*(core::ptr::addr_of!(joy_ybutton) as *const *const cvar_t) as *const u8 as usize != 0) {
                if (fAxisValue < -(*(core::ptr::addr_of!(joy_threshold) as *const *const cvar_t) as *const u8 as usize as f32)) || (fAxisValue > *(core::ptr::addr_of!(joy_threshold) as *const *const cvar_t) as *const u8 as usize as f32) {
                    Sys_QueEvent(0, 0, 1, (-(fAxisValue * 127.0)) as c_int, 0, core::ptr::null_mut());  // AXIS_FORWARD
                } else {
                    Sys_QueEvent(0, 0, 1, 0, 0, core::ptr::null_mut());  // AXIS_FORWARD
                }
                i += 1;
                continue;
            }

            if fAxisValue < -(*(core::ptr::addr_of!(joy_threshold) as *const *const cvar_t) as *const u8 as usize as f32) {
                povstate |= 1 << (i * 2);
            } else if fAxisValue > *(core::ptr::addr_of!(joy_threshold) as *const *const cvar_t) as *const u8 as usize as f32 {
                povstate |= 1 << (i * 2 + 1);
            }
            i += 1;
        }

        // convert POV information from a direction into 4 button bits
        // if ( joy.jc.wCaps & JOYCAPS_HASPOV ) {
        //     if ( joy.ji.dwPOV != JOY_POVCENTERED ) {
        //         if (joy.ji.dwPOV == JOY_POVFORWARD)
        //             povstate |= 1<<12;
        //         if (joy.ji.dwPOV == JOY_POVBACKWARD)
        //             povstate |= 1<<13;
        //         if (joy.ji.dwPOV == JOY_POVRIGHT)
        //             povstate |= 1<<14;
        //         if (joy.ji.dwPOV == JOY_POVLEFT)
        //             povstate |= 1<<15;
        //     }
        // }

        // determine which bits have changed and key an auxillary event for each change
        i = 0;
        while i < 16 {
            if ((povstate & ((1 as c_uint) << i)) != 0) && ((joy.oldpovstate & (1 << i)) == 0) {
                Sys_QueEvent(0, 0, joyDirectionKeys[i as usize], 1, 0, core::ptr::null_mut());
            }

            if ((povstate & ((1 as c_uint) << i)) == 0) && ((joy.oldpovstate & (1 << i)) != 0) {
                Sys_QueEvent(0, 0, joyDirectionKeys[i as usize], 0, 0, core::ptr::null_mut());
            }
            i += 1;
        }
        joy.oldpovstate = povstate as c_int;

        // if there is a trackball like interface, simulate mouse moves
        // if ( joy.jc.wNumAxes >= 6 ) {
        //     x = JoyToI( joy.ji.dwUpos ) * in_joyBallScale->value;
        //     y = JoyToI( joy.ji.dwVpos ) * in_joyBallScale->value;
        //     if ( x || y ) {
        //         Sys_QueEvent( g_wv.sysMsgTime, SE_MOUSE, x, y, 0, NULL );
        //     }
        // }
    }
}

/*
=========================================================================

MIDI

=========================================================================
*/

unsafe extern "C" fn MIDI_NoteOff(note: c_int) {
    let mut qkey: c_int;

    qkey = note - 60;  // A_AUX0 offset

    if qkey < 0 {  // A_AUX0
        return;
    }
    Sys_QueEvent(0, 0, qkey, 0, 0, core::ptr::null_mut());
}

unsafe extern "C" fn MIDI_NoteOn(note: c_int, velocity: c_int) {
    let mut qkey: c_int;

    if velocity == 0 {
        MIDI_NoteOff(note);
    }

    qkey = note - 60;  // A_AUX0 offset

    if qkey < 0 {  // A_AUX0
        return;
    }
    Sys_QueEvent(0, 0, qkey, 1, 0, core::ptr::null_mut());
}

extern "C" fn MidiInProc(hMidiIn: *mut core::ffi::c_void, uMsg: c_uint, dwInstance: c_uint,
                         dwParam1: c_uint, dwParam2: c_uint) {
    let mut message: c_int;

    match uMsg {
        0 => {  // MIM_OPEN
        }
        1 => {  // MIM_CLOSE
        }
        0x3C0 => {  // MIM_DATA
            message = (dwParam1 & 0xff) as c_int;

            // note on
            if (message & 0xf0) == 0x90 {
                unsafe {
                    if (((message & 0x0f) + 1) as c_uint == *(core::ptr::addr_of!(in_midichannel) as *const *const cvar_t) as *const u8 as usize as c_uint) {
                        MIDI_NoteOn(((dwParam1 & 0xff00) >> 8) as c_int, ((dwParam1 & 0xff0000) >> 16) as c_int);
                    }
                }
            } else if (message & 0xf0) == 0x80 {
                unsafe {
                    if (((message & 0x0f) + 1) as c_uint == *(core::ptr::addr_of!(in_midichannel) as *const *const cvar_t) as *const u8 as usize as c_uint) {
                        MIDI_NoteOff(((dwParam1 & 0xff00) >> 8) as c_int);
                    }
                }
            }
        }
        0x3C1 => {  // MIM_LONGDATA
        }
        0x3C2 => {  // MIM_ERROR
        }
        0x3C3 => {  // MIM_LONGERROR
        }
        _ => {}
    }

    //  Sys_QueEvent( sys_msg_time, SE_KEY, wMsg, qtrue, 0, NULL );
}

unsafe extern "C" fn MidiInfo_f() {
    let mut i: c_int;

    let enableStrings: [*const c_char; 2] = [
        b"disabled\0".as_ptr() as *const c_char,
        b"enabled\0".as_ptr() as *const c_char,
    ];

    Com_Printf(b"\nMIDI control:       %s\n\0".as_ptr() as *const c_char, if *(core::ptr::addr_of!(in_midi) as *const *const cvar_t) as *const u8 as usize != 0 { 1 } else { 0 });
    Com_Printf(b"port:               %d\n\0".as_ptr() as *const c_char, *(core::ptr::addr_of!(in_midiport) as *const *const cvar_t) as *const u8 as usize);
    Com_Printf(b"channel:            %d\n\0".as_ptr() as *const c_char, *(core::ptr::addr_of!(in_midichannel) as *const *const cvar_t) as *const u8 as usize);
    Com_Printf(b"current device:     %d\n\0".as_ptr() as *const c_char, *(core::ptr::addr_of!(in_mididevice) as *const *const cvar_t) as *const u8 as usize);
    Com_Printf(b"number of devices:  %d\n\0".as_ptr() as *const c_char, s_midiInfo.numDevices);
    i = 0;
    while i < s_midiInfo.numDevices {
        if i == Cvar_VariableValue(b"in_mididevice\0".as_ptr() as *const c_char) as c_int {
            Com_Printf(b"***\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b"...\0".as_ptr() as *const c_char);
        }
        Com_Printf(b"device %2d:       %s\n\0".as_ptr() as *const c_char, i);
        Com_Printf(b"...manufacturer ID: 0x%hx\n\0".as_ptr() as *const c_char);
        Com_Printf(b"...product ID:      0x%hx\n\0".as_ptr() as *const c_char);

        Com_Printf(b"\n\0".as_ptr() as *const c_char);
        i += 1;
    }
}

unsafe extern "C" fn IN_StartupMIDI() {
    let mut i: c_int;

    if Cvar_VariableValue(b"in_midi\0".as_ptr() as *const c_char) == 0.0 {
        return;
    }

    //
    // enumerate MIDI IN devices
    //
    s_midiInfo.numDevices = midiInGetNumDevs() as c_int;

    i = 0;
    while i < s_midiInfo.numDevices {
        midiInGetDevCaps(i as c_uint, core::ptr::addr_of_mut!(s_midiInfo.caps[i as usize]) as *mut core::ffi::c_void, 1);
        i += 1;
    }

    //
    // open the MIDI IN port
    //
    if midiInOpen(core::ptr::addr_of_mut!(s_midiInfo.hMidiIn),
                  *(core::ptr::addr_of!(in_mididevice) as *const *const cvar_t) as *const u8 as usize as c_uint,
                  MidiInProc as c_uint,
                  0 as c_uint,
                  0x00000001) != 0 {  // CALLBACK_FUNCTION
        Com_Printf(b"WARNING: could not open MIDI device %d: '%s'\n\0".as_ptr() as *const c_char, *(core::ptr::addr_of!(in_mididevice) as *const *const cvar_t) as *const u8 as usize);
        return;
    }

    midiInStart(s_midiInfo.hMidiIn);
}

unsafe extern "C" fn IN_ShutdownMIDI() {
    if !s_midiInfo.hMidiIn.is_null() {
        midiInClose(s_midiInfo.hMidiIn);
    }
    Com_Memset(core::ptr::addr_of_mut!(s_midiInfo) as *mut core::ffi::c_void, 0, mem::size_of_val(&s_midiInfo));
}
