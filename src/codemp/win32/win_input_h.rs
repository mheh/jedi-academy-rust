#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_float};

// fakeAscii_t is defined in another module; declared here for function signatures.
// (To be imported from the appropriate module when available)
pub type fakeAscii_t = c_int; // Stub - adjust import as needed

#[cfg(any(feature = "xbox", feature = "gamecube"))]
pub const _USE_RUMBLE: () = ();

pub const IN_CMD_GOTO_XTIMES: c_int = -5;
pub const IN_CMD_GOTO: c_int = -6;

pub const IN_CMD_DEC_ARG2: c_int = -7;
pub const IN_CMD_INC_ARG2: c_int = -8;
pub const IN_CMD_DEC_ARG1: c_int = -9;
pub const IN_CMD_INC_ARG1: c_int = -10;

#[cfg(feature = "xbox")]
pub const IN_CMD_DEC_LEFT: c_int = -70;
#[cfg(feature = "xbox")]
pub const IN_CMD_DEC_RIGHT: c_int = -71;
#[cfg(feature = "xbox")]
pub const IN_CMD_INC_LEFT: c_int = -72;
#[cfg(feature = "xbox")]
pub const IN_CMD_INC_RIGHT: c_int = -73;

#[cfg(feature = "gamecube")]
pub const IN_GCACTION_START: c_int = 1;
#[cfg(feature = "gamecube")]
pub const IN_GCACTION_STOP: c_int = 2;
#[cfg(feature = "gamecube")]
pub const IN_GCACTION_STOPHARD: c_int = 3;

pub const IN_MAX_JOYSTICKS: c_int = 2;

// Stores gamepad joystick info
#[repr(C)]
pub struct JoystickInfo {
    pub valid: bool,
    pub x: c_float,
    pub y: c_float,
}

// Stores gamepad id and joysick info
#[repr(C)]
pub struct PadInfo {
    pub joyInfo: [JoystickInfo; 2],
    pub padId: c_int,
}

// Buffer for gamepad info
pub static mut _padInfo: PadInfo = PadInfo {
    joyInfo: [JoystickInfo { valid: false, x: 0.0, y: 0.0 }; 2],
    padId: 0,
};

extern "C" {
    pub fn IN_ControllersChanged(inserted: *mut c_int, removed: *mut c_int) -> bool;
    pub fn IN_AnyButtonPressed() -> bool;
    pub fn IN_enableRumble();
    pub fn IN_disableRumble();
    pub fn IN_usingRumble() -> bool;
    pub fn IN_CreateRumbleScript(controller: c_int, numStates: c_int, deleteWhenFinished: bool) -> c_int;
    pub fn IN_DeleteRumbleScript(whichScript: c_int);
    pub fn IN_KillRumbleScript(whichScript: c_int);
    pub fn IN_ExecuteRumbleScript(whichScript: c_int);
    pub fn IN_AdvanceToNextState(whichScript: c_int) -> bool;
    // Rust does not support C-style overloading; split IN_KillRumbleScripts into two with different names
    pub fn IN_KillRumbleScripts_controller(controller: c_int);
    pub fn IN_KillRumbleScripts_void();
    // Rust does not support C-style overloading; split IN_PauseRumbling into two with different names
    pub fn IN_PauseRumbling_controller(controller: c_int);
    pub fn IN_PauseRumbling_void();
    // Rust does not support C-style overloading; split IN_UnPauseRumbling into two with different names
    pub fn IN_UnPauseRumbling_controller(controller: c_int);
    pub fn IN_UnPauseRumbling_void();
    // Rust does not support C-style overloading; split IN_TogglePauseRumbling into two with different names
    pub fn IN_TogglePauseRumbling_controller(controller: c_int);
    pub fn IN_TogglePauseRumbling_void();
    pub fn IN_GetMainController() -> c_int;
    pub fn IN_SetMainController(id: c_int);
    pub fn IN_PadUnplugged(controller: c_int);
    pub fn IN_PadPlugged(controller: c_int);
    pub fn IN_CommonJoyPress(controller: c_int, button: fakeAscii_t, pressed: bool);
    pub fn IN_CommonUpdate();
    pub fn IN_AddRumbleStateSpecial(whichScript: c_int, action: c_int, arg1: c_int, arg2: c_int) -> c_int;
    pub fn IN_KillRumbleState(whichScript: c_int, index: c_int);
    pub fn IN_RumbleAdjust(controller: c_int, left: c_int, right: c_int) -> bool;
    pub fn IN_RumbleInit();
    pub fn IN_RumbleShutdown();
    pub fn IN_RumbleFrame();
}

#[cfg(feature = "xbox")]
extern "C" {
    pub fn IN_AddRumbleState(whichScript: c_int, leftSpeed: c_int, rightSpeed: c_int, timeInMs: c_int) -> c_int;
    pub fn IN_AddEffectFade4(whichScript: c_int, startLeft: c_int, startRight: c_int, endLeft: c_int, endRight: c_int, timeInMs: c_int) -> c_int;
    pub fn IN_AddEffectFadeExp6(whichScript: c_int, startLeft: c_int, startRight: c_int, endLeft: c_int, endRight: c_int, factor: c_char, timeInMs: c_int) -> c_int;
}

#[cfg(feature = "gamecube")]
extern "C" {
    // Note: Original C had default argument `int arg = 0`, which Rust does not support.
    // Callers must pass all arguments explicitly.
    pub fn IN_AddRumbleState(whichScript: c_int, action: c_int, timeInMs: c_int, arg: c_int) -> c_int;
}
