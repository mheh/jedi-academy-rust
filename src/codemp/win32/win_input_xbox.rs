// win_input.c -- win32 mouse and joystick code
// 02/21/97 JCB Added extended DirectInput code to support external controllers.

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_float, c_void};

// Redefine types from win_input_h.rs for extern linkage
#[repr(C)]
pub struct JoystickInfo {
    pub valid: bool,
    pub x: c_float,
    pub y: c_float,
}

#[repr(C)]
pub struct PadInfo {
    pub joyInfo: [JoystickInfo; 2],
    pub padId: c_int,
}

// Xbox API type definitions (from xtl.h and Xbox headers)
#[repr(C)]
pub struct XINPUT_STATE {
    pub dwPacketNumber: u32,
    pub Gamepad: XINPUT_GAMEPAD,
}

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
pub struct XINPUT_FEEDBACK {
    pub Header: XINPUT_FEEDBACK_HEADER,
    pub Rumble: XINPUT_RUMBLE,
}

#[repr(C)]
pub struct XINPUT_FEEDBACK_HEADER {
    pub dwStatus: u32,
}

#[repr(C)]
pub struct XINPUT_RUMBLE {
    pub wLeftMotorSpeed: u16,
    pub wRightMotorSpeed: u16,
}

#[repr(C)]
pub struct XDEVICE_PREALLOC_TYPE {
    pub dwDeviceType: u32,
    pub dwPreallocCount: u32,
}

// From keycodes.h (external stub)
pub type fakeAscii_t = c_int;

// Xbox device type constants
const XDEVICE_TYPE_GAMEPAD: u32 = 0;
const XDEVICE_TYPE_MEMORY_UNIT: u32 = 1;
const XDEVICE_TYPE_VOICE_MICROPHONE: u32 = 2;
const XDEVICE_TYPE_VOICE_HEADPHONE: u32 = 3;
const XDEVICE_NO_SLOT: u32 = 0xFFFFFFFF;
const ERROR_IO_PENDING: u32 = 997;

// fakeAscii_t button codes (from keycodes.h)
const A_JOY1: fakeAscii_t = 256 + 1;
const A_JOY2: fakeAscii_t = 256 + 2;
const A_JOY3: fakeAscii_t = 256 + 3;
const A_JOY4: fakeAscii_t = 256 + 4;
const A_JOY5: fakeAscii_t = 256 + 5;
const A_JOY6: fakeAscii_t = 256 + 6;
const A_JOY7: fakeAscii_t = 256 + 7;
const A_JOY8: fakeAscii_t = 256 + 8;
const A_JOY9: fakeAscii_t = 256 + 9;
const A_JOY10: fakeAscii_t = 256 + 10;
const A_JOY11: fakeAscii_t = 256 + 11;
const A_JOY12: fakeAscii_t = 256 + 12;
const A_JOY13: fakeAscii_t = 256 + 13;
const A_JOY14: fakeAscii_t = 256 + 14;
const A_JOY15: fakeAscii_t = 256 + 15;
const A_JOY16: fakeAscii_t = 256 + 16;

// External Xbox API functions
extern "C" {
    fn XInputOpen(
        dwDeviceType: u32,
        dwPort: u32,
        dwSlot: u32,
        pUnknown: *mut c_void,
    ) -> *mut c_void;

    fn XInputClose(handle: *mut c_void) -> u32;

    fn XInputGetState(handle: *mut c_void, pState: *mut XINPUT_STATE) -> u32;

    fn XInputSetState(handle: *mut c_void, pFeedback: *mut XINPUT_FEEDBACK) -> u32;

    fn XGetDevices(dwDeviceType: u32) -> u32;

    fn XGetDeviceChanges(
        dwDeviceType: u32,
        pdwInsertions: *mut u32,
        pdwRemovals: *mut u32,
    ) -> c_int;

    fn XInitDevices(dwPreallocCount: usize, pPreallocTypes: *const XDEVICE_PREALLOC_TYPE) -> c_int;

    // External game callbacks (from client/keycodes.h or related)
    fn IN_UIEmptyQueue();
    fn IN_PadUnplugged(port: c_int);
    fn IN_PadPlugged(port: c_int);
    fn IN_CommonJoyPress(port: c_int, key: fakeAscii_t, pressed: bool);
    fn IN_CommonUpdate();
    fn IN_RumbleInit();
    fn IN_RumbleShutdown();
    fn IN_RumbleFrame();

    // External globals (from other modules)
    static mut noControllersConnected: bool;
    static mut wasPlugged: [bool; 4];
    static mut _padInfo: PadInfo;

    // From libc
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

#[repr(C)]
struct controller_t {
    handle: *mut c_void,
    state: XINPUT_STATE,
    feedback: XINPUT_FEEDBACK,
}

#[repr(C)]
struct inputstate_t {
    controllers: [controller_t; IN_MAX_CONTROLLERS],
}

const IN_MAX_CONTROLLERS: usize = 4;

static mut in_state: *mut inputstate_t = core::ptr::null_mut();

/*
=========================================================================

JOYSTICK

=========================================================================
*/
// Process all the insertions and removals, updating handles and such
fn IN_ProcessChanges(dwInsert: u32, dwRemove: u32) {
    for port in 0..IN_MAX_CONTROLLERS {
        // Close removals.
        if ((1 << port) & dwRemove) != 0 {
            unsafe {
                if !(*in_state).controllers[port].handle.is_null() {
                    XInputClose((*in_state).controllers[port].handle);
                    (*in_state).controllers[port].handle = core::ptr::null_mut();
                    IN_PadUnplugged(port as c_int);
                }
            }
        }

        // Open insertions.
        if (1 << port) & dwInsert != 0 {
            unsafe {
                (*in_state).controllers[port].handle = XInputOpen(
                    XDEVICE_TYPE_GAMEPAD,
                    port as u32,
                    XDEVICE_NO_SLOT,
                    core::ptr::null_mut(),
                );
                IN_PadPlugged(port as c_int);
            }
        }
    }

    return;
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
            if !wasPlugged[0] && !wasPlugged[1] && !wasPlugged[2] && !wasPlugged[3] {
                // Tell the UI that there are no controllers connected
                //	VM_Call( uivm, UI_CONTROLLER_UNPLUGGED, true, -1);
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

pub fn IN_RumbleAdjust(controller: c_int, left: c_int, right: c_int) -> bool {
    assert!(controller >= 0 && controller < IN_MAX_CONTROLLERS as c_int);

    unsafe {
        // Get a device handle for the controller.  This may fail.
        let handle = (*in_state).controllers[controller as usize].handle;

        if handle.is_null() {
            return false;
        }

        let fb = &mut (*in_state).controllers[controller as usize].feedback;

        // If a prior rumble update is still pending, go away
        if fb.Header.dwStatus == ERROR_IO_PENDING {
            return false;
        }

        fb.Rumble.wLeftMotorSpeed = left as u16;
        fb.Rumble.wRightMotorSpeed = right as u16;

        return ERROR_IO_PENDING == XInputSetState(handle, fb);
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
pub fn IN_Shutdown() {
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
pub fn IN_Init() {
    unsafe {
        in_state = Box::into_raw(Box::new(inputstate_t {
            controllers: core::mem::zeroed(),
        }));

        // Initialize support for 4 gamepads
        let xdpt: [XDEVICE_PREALLOC_TYPE; 4] = [
            XDEVICE_PREALLOC_TYPE {
                dwDeviceType: XDEVICE_TYPE_GAMEPAD,
                dwPreallocCount: 4,
            },
            XDEVICE_PREALLOC_TYPE {
                dwDeviceType: XDEVICE_TYPE_MEMORY_UNIT,
                dwPreallocCount: 1,
            },
            XDEVICE_PREALLOC_TYPE {
                dwDeviceType: XDEVICE_TYPE_VOICE_MICROPHONE,
                dwPreallocCount: 1,
            },
            XDEVICE_PREALLOC_TYPE {
                dwDeviceType: XDEVICE_TYPE_VOICE_HEADPHONE,
                dwPreallocCount: 1,
            },
        ];

        // Initialize the peripherals. We can only ever
        // call XInitDevices once, no matter what.
        static mut bInputInitialized: bool = false;
        if !bInputInitialized {
            XInitDevices(xdpt.len(), xdpt.as_ptr());
        }
        bInputInitialized = true;

        // Zero all of our data, including handles
        memset(
            (*in_state).controllers.as_mut_ptr() as *mut c_void,
            0,
            core::mem::size_of_val(&(*in_state).controllers),
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
    if y.abs() < 0.25f32 {
        y = 0.0f32;
    }

    y
}

// How many controls on the xbox gamepad?
#[allow(non_upper_case_globals)]
const IN_NUM_DIGITAL_BUTTONS: usize = 8;
#[allow(non_upper_case_globals)]
const IN_NUM_ANALOG_BUTTONS: usize = 8;
// Cutoff where the analog buttons are considered to be "pressed"
// This should be smarter.
#[allow(non_upper_case_globals)]
const IN_ANALOG_BUTTON_THRESHOLD: u8 = 64;

fn IN_UpdateGamepad(port: c_int) {
    unsafe {
        // Lookup table to convert the digital buttons to fakeAscii_t, in mask order
        let digitalXlat: [fakeAscii_t; IN_NUM_DIGITAL_BUTTONS] = [
            A_JOY5, // DPAD_UP
            A_JOY7, // DPAD_DOWN
            A_JOY8, // DPAD_LEFT
            A_JOY6, // DPAD_LEFT
            A_JOY4, // Start
            A_JOY1, // Back
            A_JOY2, // Left stick
            A_JOY3, // Right stick
        ];

        // Lookup table to convet the analog buttons to fakeAscii_t, in DX order
        let analogXlat: [fakeAscii_t; IN_NUM_ANALOG_BUTTONS] = [
            A_JOY15, // A
            A_JOY14, // B
            A_JOY16, // X
            A_JOY13, // Y
            A_JOY10, // Black
            A_JOY9,  // White
            A_JOY11, // Left trigger
            A_JOY12, // Right trigger
        ];

        // Get new state
        let mut newState: XINPUT_STATE = core::mem::zeroed();
        XInputGetState((*in_state).controllers[port as usize].handle, &mut newState);

        // Get old state
        let oldState = &mut (*in_state).controllers[port as usize].state;

        // Check all digital buttons first
        for buttonIdx in 0..IN_NUM_DIGITAL_BUTTONS {
            let oldPressed = (oldState.Gamepad.wButtons & (1 << buttonIdx)) != 0;
            let newPressed = (newState.Gamepad.wButtons & (1 << buttonIdx)) != 0;

            if oldPressed != newPressed {
                IN_CommonJoyPress(port, digitalXlat[buttonIdx], newPressed);
            }
        }

        // Now check all analog buttons
        for buttonIdx in 0..IN_NUM_ANALOG_BUTTONS {
            let oldPressed = oldState.Gamepad.bAnalogButtons[buttonIdx] > IN_ANALOG_BUTTON_THRESHOLD;
            let newPressed = newState.Gamepad.bAnalogButtons[buttonIdx] > IN_ANALOG_BUTTON_THRESHOLD;

            if oldPressed != newPressed {
                IN_CommonJoyPress(port, analogXlat[buttonIdx], newPressed);
            }
        }

        // Update joysticks
        _padInfo.joyInfo[0].x = _joyAxisConvert(newState.Gamepad.sThumbLX);
        _padInfo.joyInfo[0].y = _joyAxisConvert(newState.Gamepad.sThumbLY);
        _padInfo.joyInfo[1].x = _joyAxisConvert(newState.Gamepad.sThumbRX);
        _padInfo.joyInfo[1].y = _joyAxisConvert(newState.Gamepad.sThumbRY);
        _padInfo.joyInfo[0].valid = true;
        _padInfo.joyInfo[1].valid = true;
        _padInfo.padId = port;

        // Copy state back
        *oldState = newState;

        // Update game
        IN_CommonUpdate();
    }
}

/*
==================
IN_Frame

Called every frame, even if not generating commands
==================
*/
//extern int ignoreInputTime;
pub fn IN_Frame() {
    unsafe {
        if !in_state.is_null() {
            // First, check for changes in device status (removed/inserted pads)
            let mut dwInsert: u32 = 0;
            let mut dwRemove: u32 = 0;
            if XGetDeviceChanges(XDEVICE_TYPE_GAMEPAD, &mut dwInsert, &mut dwRemove) != 0 {
                IN_ProcessChanges(dwInsert, dwRemove);
            } else {
                IN_CheckForNoControllers();
            }

            // Generate callbacks for each controller that's plugged in
            for port in 0..IN_MAX_CONTROLLERS {
                if !(*in_state).controllers[port].handle.is_null() {
                    IN_UpdateGamepad(port as c_int);
                }
            }

            IN_UIEmptyQueue();
            IN_RumbleFrame();
        }
    }
}
