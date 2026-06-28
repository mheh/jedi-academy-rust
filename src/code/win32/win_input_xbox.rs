// win_input.c -- win32 mouse and joystick code
// 02/21/97 JCB Added extended DirectInput code to support external controllers.

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

use core::ffi::c_int;

// Declarations for Xbox API types (from <xtl.h> and related headers)
// These are opaque FFI types representing Xbox SDK structures

#[repr(C)]
pub struct XINPUT_GAMEPAD {
    pub wButtons: u16,
    pub bAnalogButtons: [u8; 8],
    pub sThumbLX: i16,
    pub sThumbLY: i16,
    pub sThumbRX: i16,
    pub sThumbRY: i16,
}

#[repr(C)]
pub struct XINPUT_STATE {
    pub dwPacketNumber: u32,
    pub Gamepad: XINPUT_GAMEPAD,
}

#[repr(C)]
pub struct XINPUT_RUMBLE {
    pub wLeftMotorSpeed: u16,
    pub wRightMotorSpeed: u16,
}

#[repr(C)]
pub struct XINPUT_FEEDBACK_HEADER {
    pub dwStatus: u32,
}

#[repr(C)]
pub struct XINPUT_FEEDBACK {
    pub Header: XINPUT_FEEDBACK_HEADER,
    pub Rumble: XINPUT_RUMBLE,
}

// Forward declaration of opaque HANDLE type for Xbox API
pub type HANDLE = *mut core::ffi::c_void;

// External Xbox API constants and types
pub const XDEVICE_TYPE_GAMEPAD: u32 = 0;
pub const XDEVICE_NO_SLOT: u32 = 0;
pub const ERROR_IO_PENDING: u32 = 997;

#[repr(C)]
pub struct XDEVICE_PREALLOC_TYPE {
    pub DeviceType: u32,
    pub PreallocCount: u32,
}

#[repr(C)]
pub struct controller_t {
    pub handle: HANDLE,
    pub state: XINPUT_STATE,
    pub feedback: XINPUT_FEEDBACK,
}

#[repr(C)]
pub struct inputstate_t {
    pub controllers: [controller_t; 4],
}

pub static mut in_state: *mut inputstate_t = core::ptr::null_mut();

extern "C" {
    fn IN_UIEmptyQueue();
    fn IN_PadUnplugged(port: c_int);
    fn IN_PadPlugged(port: c_int);
    fn IN_CommonJoyPress(port: c_int, key: i32, pressed: bool);
    fn IN_CommonUpdate();
    fn IN_RumbleInit();
    fn IN_RumbleShutdown();
    fn IN_RumbleFrame();

    // Xbox API functions
    fn XInputClose(handle: HANDLE) -> u32;
    fn XInputOpen(device_type: u32, port: c_int, slot: u32, reserved: *mut core::ffi::c_void) -> HANDLE;
    fn XGetDevices(device_type: u32) -> u32;
    fn XGetDeviceChanges(device_type: u32, insert: *mut u32, remove: *mut u32) -> c_int;
    fn XInputGetState(handle: HANDLE, state: *mut XINPUT_STATE) -> u32;
    fn XInputSetState(handle: HANDLE, feedback: *mut XINPUT_FEEDBACK) -> u32;
    fn XInitDevices(count: u32, prealloc: *const XDEVICE_PREALLOC_TYPE) -> u32;

    // External game state
    static mut noControllersConnected: bool;
    static wasPlugged: [bool; 4];

    // Joystick info structure
    static mut _padInfo: PadInfo;
}

#[repr(C)]
pub struct JoyInfo {
    pub x: f32,
    pub y: f32,
    pub valid: bool,
}

#[repr(C)]
pub struct PadInfo {
    pub joyInfo: [JoyInfo; 2],
    pub padId: c_int,
}

const IN_MAX_CONTROLLERS: c_int = 4;

/*
=========================================================================

JOYSTICK

=========================================================================
*/
// Process all the insertions and removals, updating handles and such
fn IN_ProcessChanges(dwInsert: u32, dwRemove: u32) {
    for port in 0..IN_MAX_CONTROLLERS {
        // Close removals.
        if ((1 << port) & dwRemove) != 0
        {
            unsafe {
                if !(*in_state).controllers[port as usize].handle.is_null() {
                    XInputClose((*in_state).controllers[port as usize].handle);
                    (*in_state).controllers[port as usize].handle = core::ptr::null_mut();
                    IN_PadUnplugged(port);
                }
            }
        }

        // Open insertions.
        if (1 << port) & dwInsert != 0
        {
            unsafe {
                (*in_state).controllers[port as usize].handle = XInputOpen(XDEVICE_TYPE_GAMEPAD, port, XDEVICE_NO_SLOT, core::ptr::null_mut());
                IN_PadPlugged(port);
            }
        }
    }

    // Rust: no explicit return needed for void-like functions
}

/*********
IN_CheckForNoControllers()
If there are no controllers plugged in, the UI
is notified so it can display an appropriate
message.
*********/
fn IN_CheckForNoControllers() {
    unsafe {
        if !noControllersConnected {
            if !wasPlugged[0]
                && !wasPlugged[1]
                && !wasPlugged[2]
                && !wasPlugged[3]
            {
                // Tell the UI that there are no controllers connected
                // VM_Call( uivm, UI_CONTROLLER_UNPLUGGED, true, -1);
                noControllersConnected = true;
            }
        }
    }
}

/*
=========================================================================

  RUMBLE SUPPORT

=========================================================================
*/

fn IN_RumbleAdjust(controller: c_int, left: c_int, right: c_int) -> bool {
    assert!(controller >= 0 && controller < IN_MAX_CONTROLLERS);

    // Get a device handle for the controller.  This may fail.
    let handle = unsafe { (*in_state).controllers[controller as usize].handle };

    if handle.is_null() {
        return false;
    }

    unsafe {
        let fb = &mut (*in_state).controllers[controller as usize].feedback;

        // If a prior rumble update is still pending, go away
        if fb.Header.dwStatus == ERROR_IO_PENDING {
            return false;
        }

        fb.Rumble.wLeftMotorSpeed = left as u16;
        fb.Rumble.wRightMotorSpeed = right as u16;

        XInputSetState(handle, fb) == ERROR_IO_PENDING
    }
}


/*
=========================================================================

=========================================================================
*/

/*
igBool IN_WindowClose(igWindow *window)
{
	SV_Shutdown ("Server quit\n");
	CL_Shutdown ();
	Com_Shutdown ();
	Sys_Quit ();
	return true;
}
*/

/*
===========
IN_Shutdown
===========
*/
fn IN_Shutdown() {
    IN_RumbleShutdown();

    unsafe {
        if !in_state.is_null() {
            let _ = Box::from_raw(in_state);
            in_state = core::ptr::null_mut();
        }
    }
}


/*
===========
IN_Init
===========
*/
fn IN_Init() {
    unsafe {
        // Initialize support for 4 gamepads
        let mut xdpt: [XDEVICE_PREALLOC_TYPE; 1] = [XDEVICE_PREALLOC_TYPE {
            DeviceType: XDEVICE_TYPE_GAMEPAD,
            PreallocCount: 4,
        }];

        // Initialize the peripherals. We can only ever
        // call XInitDevices once, no matter what.
        static mut bInputInitialized: bool = false;
        if !bInputInitialized {
            XInitDevices(1, &xdpt[0]);
        }
        bInputInitialized = true;

        in_state = Box::into_raw(Box::new(inputstate_t {
            controllers: core::mem::zeroed(),
        }));

        // Zero all of our data, including handles
        core::ptr::write_bytes(
            (*in_state).controllers.as_mut_ptr(),
            0,
            IN_MAX_CONTROLLERS as usize,
        );

        // Find out the status of all gamepad ports, then open them
        IN_ProcessChanges(XGetDevices(XDEVICE_TYPE_GAMEPAD), 0);

        IN_RumbleInit();
    }
}

#[inline]
fn _joyAxisConvert(x: i16) -> f32 {
    // Change scale
    let mut y = x as f32 / 32767.0;

    // Cheesy deadzone
    if y.abs() < 0.25 {
        y = 0.0;
    }

    y
}

// How many controls on the xbox gamepad?
const IN_NUM_DIGITAL_BUTTONS: usize = 8;
const IN_NUM_ANALOG_BUTTONS: usize = 8;
// Cutoff where the analog buttons are considered to be "pressed"
// This should be smarter.
const IN_ANALOG_BUTTON_THRESHOLD: u8 = 64;

fn IN_UpdateGamepad(port: c_int) {
    // Lookup table to convert the digital buttons to fakeAscii_t, in mask order
    let digitalXlat: [i32; IN_NUM_DIGITAL_BUTTONS] = unsafe {
        let mut arr: [i32; IN_NUM_DIGITAL_BUTTONS] = core::mem::zeroed();
        arr[0] = A_JOY5; // DPAD_UP
        arr[1] = A_JOY7; // DPAD_DOWN
        arr[2] = A_JOY8; // DPAD_LEFT
        arr[3] = A_JOY6; // DPAD_LEFT
        arr[4] = A_JOY4; // Start
        arr[5] = A_JOY1; // Back
        arr[6] = A_JOY2; // Left stick
        arr[7] = A_JOY3;  // Right stick
        arr
    };

    // Lookup table to convet the analog buttons to fakeAscii_t, in DX order
    let analogXlat: [i32; IN_NUM_ANALOG_BUTTONS] = unsafe {
        let mut arr: [i32; IN_NUM_ANALOG_BUTTONS] = core::mem::zeroed();
        arr[0] = A_JOY15; // A
        arr[1] = A_JOY14; // B
        arr[2] = A_JOY16; // X
        arr[3] = A_JOY13; // Y
        arr[4] = A_JOY10; // Black
        arr[5] = A_JOY9;  // White
        arr[6] = A_JOY11; // Left trigger
        arr[7] = A_JOY12;  // Right trigger
        arr
    };

    // Get new state
    let mut newState: XINPUT_STATE = unsafe { core::mem::zeroed() };
    unsafe {
        XInputGetState((*in_state).controllers[port as usize].handle, &mut newState);
    }

    // Get old state
    let oldState = unsafe { &mut (*in_state).controllers[port as usize].state };

    let mut buttonIdx: usize;
    let mut oldPressed: bool;
    let mut newPressed: bool;

    // Check all digital buttons first
    for buttonIdx in 0..IN_NUM_DIGITAL_BUTTONS {
        oldPressed = (oldState.Gamepad.wButtons & (1 << buttonIdx)) != 0;
        newPressed = (newState.Gamepad.wButtons & (1 << buttonIdx)) != 0;

        if oldPressed != newPressed {
            IN_CommonJoyPress(port, digitalXlat[buttonIdx], newPressed);
        }
    }

    // Now check all analog buttons
    for buttonIdx in 0..IN_NUM_ANALOG_BUTTONS {
        oldPressed = oldState.Gamepad.bAnalogButtons[buttonIdx] > IN_ANALOG_BUTTON_THRESHOLD;
        newPressed = newState.Gamepad.bAnalogButtons[buttonIdx] > IN_ANALOG_BUTTON_THRESHOLD;

        if oldPressed != newPressed {
            IN_CommonJoyPress(port, analogXlat[buttonIdx], newPressed);
        }
    }

    // Update joysticks
    unsafe {
        _padInfo.joyInfo[0].x = _joyAxisConvert(newState.Gamepad.sThumbLX);
        _padInfo.joyInfo[0].y = _joyAxisConvert(newState.Gamepad.sThumbLY);
        _padInfo.joyInfo[1].x = _joyAxisConvert(newState.Gamepad.sThumbRX);
        _padInfo.joyInfo[1].y = _joyAxisConvert(newState.Gamepad.sThumbRY);
        _padInfo.joyInfo[0].valid = true;
        _padInfo.joyInfo[1].valid = true;
        _padInfo.padId = port;
    }

    // Copy state back
    *oldState = newState;

    // Update game
    IN_CommonUpdate();
}

/*
==================
IN_Frame

Called every frame, even if not generating commands
==================
*/
// extern int ignoreInputTime;
fn IN_Frame() {
    unsafe {
        if !in_state.is_null() {
            // First, check for changes in device status (removed/inserted pads)
            let mut dwInsert: u32 = 0;
            let mut dwRemove: u32 = 0;
            if XGetDeviceChanges(XDEVICE_TYPE_GAMEPAD, &mut dwInsert, &mut dwRemove) != 0
            {
                IN_ProcessChanges(dwInsert, dwRemove);
            }
            else
            {
                IN_CheckForNoControllers();
            }

            // Generate callbacks for each controller that's plugged in
            for port in 0..IN_MAX_CONTROLLERS {
                if !(*in_state).controllers[port as usize].handle.is_null() {
                    IN_UpdateGamepad(port);
                }
            }

            IN_UIEmptyQueue();
            IN_RumbleFrame();
        }
    }
}

// External key code constants (from keycodes.h)
extern "C" {
    static A_JOY1: i32;
    static A_JOY2: i32;
    static A_JOY3: i32;
    static A_JOY4: i32;
    static A_JOY5: i32;
    static A_JOY6: i32;
    static A_JOY7: i32;
    static A_JOY8: i32;
    static A_JOY9: i32;
    static A_JOY10: i32;
    static A_JOY11: i32;
    static A_JOY12: i32;
    static A_JOY13: i32;
    static A_JOY14: i32;
    static A_JOY15: i32;
    static A_JOY16: i32;
}
