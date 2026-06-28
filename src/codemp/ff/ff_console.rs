/*
 * Stubs to allow linking with FF_ fnuctions declared.
 * Brian Osman
 */

//JLFRUMBLE includes modified to avoid typename collision field_t

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};
use core::ptr::addr_of_mut;

// Stub: imported from ../client/fffx.h
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum ffFX_e {
	fffx_AircraftCarrierTakeOff,
	fffx_BasketballDribble,
	fffx_CarEngineIdle,
	fffx_ChainsawIdle,
	fffx_ChainsawInAction,
	fffx_DieselEngineIdle,
	fffx_Jump,
	fffx_Land,
	fffx_MachineGun,
	fffx_Punched,
	fffx_RocketLaunch,
	fffx_SecretDoor,
	fffx_SwitchClick,
	fffx_WindGust,
	fffx_WindShear,
	fffx_Pistol,
	fffx_Shotgun,
	fffx_Laser1,
	fffx_Laser2,
	fffx_Laser3,
	fffx_Laser4,
	fffx_Laser5,
	fffx_Laser6,
	fffx_OutOfAmmo,
	fffx_LightningGun,
	fffx_Missile,
	fffx_GatlingGun,
	fffx_ShortPlasma,
	fffx_PlasmaCannon1,
	fffx_PlasmaCannon2,
	fffx_Cannon,
	fffx_FallingShort,
	fffx_FallingMedium,
	fffx_FallingFar,
	fffx_StartConst,
	fffx_StopConst,
}

extern "C" {
	fn Com_Printf(fmt: *const c_char, ...) -> c_int;
	fn IN_CreateRumbleScript(controller: c_int, num_states: c_int, clear: bool) -> c_int;
	fn IN_GetMainController() -> c_int;
	fn IN_AddRumbleState(script_id: c_int, left_motor: c_int, right_motor: c_int, duration: c_int) -> ();
	fn IN_ExecuteRumbleScript(script_id: c_int) -> ();
	fn IN_AddEffectFade4(script_id: c_int, arg2: c_int, arg3: c_int, arg4: c_int, arg5: c_int, arg6: c_int) -> ();
	fn IN_KillRumbleScript(script_id: c_int) -> ();
}

static mut const_rumble: [c_int; 2] = [-1, -1]; // script id for constant rumble

pub fn FF_StopAll() {
	unsafe {
		Com_Printf(b"FF_StopAll: Please implement.\n\0".as_ptr() as *const c_char);
	}
	// Do nothing
}

pub fn FF_Stop(effect: ffFX_e) {
	unsafe {
		Com_Printf(b"FF_Stop: Please implement fffx_id = %i\n\0".as_ptr() as *const c_char, effect as c_int);
	}
	// Do nothing
}

pub fn FF_EnsurePlaying(effect: ffFX_e) {
	unsafe {
		Com_Printf(b"FF_EnsurePlaying: Please implement fffx_id = %i\n\0".as_ptr() as *const c_char, effect as c_int);
	}
	// Do nothing
}

pub fn FF_Play(effect: ffFX_e) {
	let mut s: c_int;	// script id
	let mut client: c_int;

	// super huge switch for rumble effects
	match effect {
	ffFX_e::fffx_AircraftCarrierTakeOff |
	ffFX_e::fffx_BasketballDribble |
	ffFX_e::fffx_CarEngineIdle |
	ffFX_e::fffx_ChainsawIdle |
	ffFX_e::fffx_ChainsawInAction |
	ffFX_e::fffx_DieselEngineIdle |
	ffFX_e::fffx_Jump => {
		unsafe {
			s = IN_CreateRumbleScript(IN_GetMainController(), 2, true);
			if s != -1 {
				IN_AddRumbleState(s, 50000, 10000, 200);
				IN_AddRumbleState(s, 0, 0, 10);
				IN_ExecuteRumbleScript(s);
			}
		}
	},
	ffFX_e::fffx_Land => {
		unsafe {
			s = IN_CreateRumbleScript(IN_GetMainController(), 2, true);
			if s != -1 {
				IN_AddRumbleState(s, 50000, 10000, 200);
				IN_AddRumbleState(s, 0, 0, 10);
				IN_ExecuteRumbleScript(s);
			}
		}
	},
	ffFX_e::fffx_MachineGun => {
		unsafe {
			s = IN_CreateRumbleScript(IN_GetMainController(), 2, true);
			if s != -1 {
				IN_AddRumbleState(s, 56000, 20000, 230);
				IN_AddRumbleState(s, 0, 0, 10);
				IN_ExecuteRumbleScript(s);
			}
		}
	},
	ffFX_e::fffx_Punched |
	ffFX_e::fffx_RocketLaunch |
	ffFX_e::fffx_SecretDoor |
	ffFX_e::fffx_SwitchClick => {	// used by saber
		unsafe {
			s = IN_CreateRumbleScript(IN_GetMainController(), 1, true);
			if s != -1 {
				IN_AddRumbleState(s, 30000, 10000, 120);
				IN_ExecuteRumbleScript(s);
			}
		}
	},
	ffFX_e::fffx_WindGust |
	ffFX_e::fffx_WindShear |
	ffFX_e::fffx_Pistol => {
		unsafe {
			s = IN_CreateRumbleScript(IN_GetMainController(), 2, true);
			if s != -1 {
				IN_AddRumbleState(s, 50000, 10000, 200);
				IN_AddRumbleState(s, 0, 0, 10);
				IN_ExecuteRumbleScript(s);
			}
		}
	},
	ffFX_e::fffx_Shotgun |
	ffFX_e::fffx_Laser1 |
	ffFX_e::fffx_Laser2 |
	ffFX_e::fffx_Laser3 |
	ffFX_e::fffx_Laser4 |
	ffFX_e::fffx_Laser5 |
	ffFX_e::fffx_Laser6 |
	ffFX_e::fffx_OutOfAmmo |
	ffFX_e::fffx_LightningGun |
	ffFX_e::fffx_Missile |
	ffFX_e::fffx_GatlingGun => {
		unsafe {
			s = IN_CreateRumbleScript(IN_GetMainController(), 2, true);
			if s != -1 {
				IN_AddRumbleState(s, 39000, 0, 220);
				IN_AddRumbleState(s, 0, 0, 10);
				IN_ExecuteRumbleScript(s);
			}
		}
	},
	ffFX_e::fffx_ShortPlasma |
	ffFX_e::fffx_PlasmaCannon1 |
	ffFX_e::fffx_PlasmaCannon2 |
	ffFX_e::fffx_Cannon |
	ffFX_e::fffx_FallingShort |
	ffFX_e::fffx_FallingMedium => {
		unsafe {
			s = IN_CreateRumbleScript(IN_GetMainController(), 1, true);
			if s != -1 {
				IN_AddRumbleState(s, 25000, 10000, 230);
				IN_ExecuteRumbleScript(s);
			}
		}
	},
	ffFX_e::fffx_FallingFar => {
		unsafe {
			s = IN_CreateRumbleScript(IN_GetMainController(), 1, true);
			if s != -1 {
				IN_AddRumbleState(s, 32000, 10000, 230);
				IN_ExecuteRumbleScript(s);
			}
		}
	},
	ffFX_e::fffx_StartConst => {
		unsafe {
			client = IN_GetMainController();
			let ptr = addr_of_mut!(const_rumble);
			if (*ptr)[client as usize] == -1 {
				(*ptr)[client as usize] = IN_CreateRumbleScript(IN_GetMainController(), 9, true);
				if (*ptr)[client as usize] != -1 {
					IN_AddEffectFade4((*ptr)[client as usize], 0, 0, 50000, 0, 2000);
					IN_AddRumbleState((*ptr)[client as usize], 50000, 0, 300);
					IN_AddEffectFade4((*ptr)[client as usize], 50000, 50000, 0, 0, 1000);
					IN_ExecuteRumbleScript((*ptr)[client as usize]);
				}
			}
		}
	},
	ffFX_e::fffx_StopConst => {
		unsafe {
			client = IN_GetMainController();
			let ptr = addr_of_mut!(const_rumble);
			if (*ptr)[client as usize] == -1 {
				return;
			}
			IN_KillRumbleScript((*ptr)[client as usize]);
			(*ptr)[client as usize] = -1;
		}
	},
	_ => {
		unsafe {
			Com_Printf(b"No rumble script is defined for fffx_id = %i\n\0".as_ptr() as *const c_char, effect as c_int);
		}
	},
	}
}

/*********
FF_XboxShake

intensity	- speed of rumble
duration	- length of rumble
*********/
const FF_SH_MIN_MOTOR_SPEED: c_int = 20000;
const FF_SH_MOTOR_SPEED_MODIFIER: c_int = 65535 - FF_SH_MIN_MOTOR_SPEED;

pub fn FF_XboxShake(intensity: f32, duration: c_int) {
	let mut s: c_int;
	unsafe {
		s = IN_CreateRumbleScript(IN_GetMainController(), 1, true);
		if s != -1 {
			let mut speed: c_int;
			// figure out the speed
			speed = FF_SH_MIN_MOTOR_SPEED + (FF_SH_MOTOR_SPEED_MODIFIER as f32 * intensity) as c_int;

			// Add the state and execute
			IN_AddRumbleState(s, speed, speed, duration);
			IN_ExecuteRumbleScript(s);
		}
	}
}

/*********
FF_XboxDamage

damage	- Amount of damage
xpos	- x position for the damage ( -1.0 - 1.0 )

The following function various the rumble based upon
the amount of damage and the position of the damage.
*********/
const FF_DA_MIN_MOTOR_SPEED: c_int = 20000;	// use this to vary the minimum intensity
const FF_DA_MOTOR_SPEED_MODIFIER: c_int = 65535 - FF_DA_MIN_MOTOR_SPEED;

pub fn FF_XboxDamage(damage: c_int, xpos: f32) {
	let mut s: c_int;
	unsafe {
		s = IN_CreateRumbleScript(IN_GetMainController(), 1, true);
		if s != -1 {
			let mut leftMotorSpeed: c_int;
			let mut rightMotorSpeed: c_int;
			let mut duration: c_int;
			let mut per: f32;

			duration = 175;

			// how much damage?
			if damage > 100 {
				per = 1.0;
			}
			else {
				per = damage as f32 / 100.0;
			}

			if xpos >= -0.2 && xpos <= 0.2 {	// damge to center
				leftMotorSpeed = (FF_DA_MIN_MOTOR_SPEED) + ((FF_DA_MOTOR_SPEED_MODIFIER as f32 * per) as c_int);
				rightMotorSpeed = leftMotorSpeed;
			}
			else if xpos > 0.2 {	// damage to right
				rightMotorSpeed = (FF_DA_MIN_MOTOR_SPEED) + ((FF_DA_MOTOR_SPEED_MODIFIER as f32 * per) as c_int);
				leftMotorSpeed = 0;
			}
			else {	// damage to left
				leftMotorSpeed = (FF_DA_MIN_MOTOR_SPEED) + ((FF_DA_MOTOR_SPEED_MODIFIER as f32 * per) as c_int);
				rightMotorSpeed = 0;
			}

			// Add the state and execute
			IN_AddRumbleState(s, leftMotorSpeed, rightMotorSpeed, duration);
			IN_ExecuteRumbleScript(s);
		}
	}
}
