#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// fakeAscii_t is defined in the input system module
type fakeAscii_t = c_int;

extern "C" {
	pub fn IN_ControllersChanged(inserted: *mut c_int, removed: *mut c_int) -> bool;
}

// #if defined (_XBOX ) || defined (_GAMECUBE)
// #define _USE_RUMBLE
// #endif

extern "C" {
	pub fn IN_AnyButtonPressed() -> bool;

	pub fn IN_enableRumble();
	pub fn IN_disableRumble();
	pub fn IN_usingRumble() -> bool;

	pub fn IN_CreateRumbleScript(controller: c_int, numStates: c_int, deleteWhenFinished: bool) -> c_int;
	pub fn IN_DeleteRumbleScript(whichScript: c_int);
	pub fn IN_KillRumbleScript(whichScript: c_int);
	pub fn IN_ExecuteRumbleScript(whichScript: c_int);

	pub fn IN_AdvanceToNextState(whichScript: c_int) -> bool;

	pub fn IN_KillRumbleScripts(controller: c_int);
	// Overload: IN_KillRumbleScripts(void) - kill all scripts
	pub fn IN_KillRumbleScripts_void();
}

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

#[cfg(feature = "xbox")]
extern "C" {
	pub fn IN_AddRumbleState(whichScript: c_int, leftSpeed: c_int, rightSpeed: c_int, timeInMs: c_int) -> c_int;
	pub fn IN_AddEffectFade4(
		whichScript: c_int,
		startLeft: c_int,
		startRight: c_int,
		endLeft: c_int,
		endRight: c_int,
		timeInMs: c_int,
	) -> c_int;
	pub fn IN_AddEffectFadeExp6(
		whichScript: c_int,
		startLeft: c_int,
		startRight: c_int,
		endLeft: c_int,
		endRight: c_int,
		factor: c_char,
		timeInMs: c_int,
	) -> c_int;
}

#[cfg(feature = "gamecube")]
pub const IN_GCACTION_START: c_int = 1;
#[cfg(feature = "gamecube")]
pub const IN_GCACTION_STOP: c_int = 2;
#[cfg(feature = "gamecube")]
pub const IN_GCACTION_STOPHARD: c_int = 3;

#[cfg(feature = "gamecube")]
extern "C" {
	// Note: Original C++ version had default parameter: int arg = 0
	pub fn IN_AddRumbleState(whichScript: c_int, action: c_int, timeInMs: c_int, arg: c_int) -> c_int;
}

extern "C" {
	pub fn IN_AddRumbleStateSpecial(whichScript: c_int, action: c_int, arg1: c_int, arg2: c_int) -> c_int;
	pub fn IN_KillRumbleState(whichScript: c_int, index: c_int);

	pub fn IN_PauseRumbling(controller: c_int);
	// Overload: IN_PauseRumbling(void)
	pub fn IN_PauseRumbling_void();

	pub fn IN_UnPauseRumbling(controller: c_int);
	// Overload: IN_UnPauseRumbling(void)
	pub fn IN_UnPauseRumbling_void();

	pub fn IN_TogglePauseRumbling(controller: c_int);
	// Overload: IN_TogglePauseRumbling(void)
	pub fn IN_TogglePauseRumbling_void();

	pub fn IN_GetMainController() -> c_int;
	pub fn IN_SetMainController(id: c_int);

	pub fn IN_PadUnplugged(controller: c_int);
	pub fn IN_PadPlugged(controller: c_int);

	pub fn IN_CommonJoyPress(controller: c_int, button: fakeAscii_t, pressed: bool);
	pub fn IN_CommonUpdate();

	pub fn IN_RumbleAdjust(controller: c_int, left: c_int, right: c_int) -> bool;
	pub fn IN_RumbleInit();
	pub fn IN_RumbleShutdown();
	pub fn IN_RumbleFrame();
}

pub const IN_MAX_JOYSTICKS: c_int = 2;

// Stores gamepad joystick info
#[repr(C)]
pub struct JoystickInfo {
	pub valid: bool,
	pub x: f32,
	pub y: f32,
}

// Stores gamepad id and joysick info
#[repr(C)]
pub struct PadInfo {
	pub joyInfo: [JoystickInfo; 2],
	pub padId: c_int,
}

// Buffer for gamepad info
extern "C" {
	pub static mut _padInfo: PadInfo;
}
