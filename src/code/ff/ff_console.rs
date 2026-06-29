#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_float};
use crate::code::client::fffx_h::ffFX_e;
use crate::code::client::fffx_h::{
    fffx_AircraftCarrierTakeOff, fffx_BasketballDribble, fffx_CarEngineIdle,
    fffx_ChainsawIdle, fffx_ChainsawInAction, fffx_DieselEngineIdle, fffx_Jump,
    fffx_Land, fffx_MachineGun, fffx_Punched, fffx_RocketLaunch, fffx_SecretDoor,
    fffx_SwitchClick, fffx_WindGust, fffx_WindShear, fffx_Pistol, fffx_Shotgun,
    fffx_Laser1, fffx_Laser2, fffx_Laser3, fffx_Laser4, fffx_Laser5, fffx_Laser6,
    fffx_OutOfAmmo, fffx_LightningGun, fffx_Missile, fffx_GatlingGun,
    fffx_ShortPlasma, fffx_PlasmaCannon1, fffx_PlasmaCannon2, fffx_Cannon,
};

// Conditional imports for Xbox-specific effects
#[cfg(feature = "xbox")]
use crate::code::client::fffx_h::{
    fffx_FallingShort, fffx_FallingMedium, fffx_FallingFar,
    fffx_StartConst, fffx_StopConst,
};

/*
 * Stubs to allow linking with FF_ fnuctions declared.
 * Brian Osman
 */

// JLFRUMBLE includes modified to avoid typename collision field_t MPSKIPPED

extern "C" {
    fn Com_Printf(fmt: *const c_char, ...) -> ();
    fn IN_CreateRumbleScript(controller: c_int, count: c_int, looping: c_int) -> c_int;
    fn IN_GetMainController() -> c_int;
    fn IN_AddRumbleState(script: c_int, leftMotor: c_int, rightMotor: c_int, duration: c_int) -> ();
    fn IN_ExecuteRumbleScript(script: c_int) -> ();
    fn IN_KillRumbleScript(script: c_int) -> ();
    fn IN_AddEffectFade4(script: c_int, leftMotor: c_int, rightMotor: c_int, leftFade: c_int, rightFade: c_int, duration: c_int) -> ();
}

pub extern "C" fn FF_StopAll() {
    unsafe {
        Com_Printf(c"FF_StopAll: Please implement.\n".as_ptr());
    }
    // Do nothing
}

pub extern "C" fn FF_Stop(effect: ffFX_e) {
    unsafe {
        Com_Printf(c"FF_Stop: Please implement fffx_id = %i\n".as_ptr(), effect as c_int);
    }
    // Do nothing
}

pub extern "C" fn FF_EnsurePlaying(effect: ffFX_e) {
    unsafe {
        Com_Printf(c"FF_EnsurePlaying: Please implement fffx_id = %i\n".as_ptr(), effect as c_int);
    }
    // Do nothing
}

pub extern "C" fn FF_Play(effect: ffFX_e) {
    let mut s: c_int; // script id
    static mut const_rumble: [c_int; 2] = [-1, 0]; // script id for constant rumble
    let mut client: c_int;

    // super huge switch for rumble effects
    match effect {
        fffx_AircraftCarrierTakeOff | fffx_BasketballDribble | fffx_CarEngineIdle
        | fffx_ChainsawIdle | fffx_ChainsawInAction | fffx_DieselEngineIdle | fffx_Jump => {
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 2, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 50000, 10000, 200);
                    IN_AddRumbleState(s, 0, 0, 10);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        fffx_Land => {
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 2, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 50000, 10000, 200);
                    IN_AddRumbleState(s, 0, 0, 10);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        fffx_MachineGun => {
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 2, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 56000, 20000, 230);
                    IN_AddRumbleState(s, 0, 0, 10);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        fffx_Punched | fffx_RocketLaunch
        | fffx_SecretDoor | fffx_SwitchClick => {  // used by saber
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 1, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 30000, 10000, 120);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        fffx_WindGust | fffx_WindShear | fffx_Pistol => {
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 2, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 50000, 10000, 200);
                    IN_AddRumbleState(s, 0, 0, 10);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        fffx_Shotgun | fffx_Laser1 | fffx_Laser2 | fffx_Laser3 | fffx_Laser4
        | fffx_Laser5 | fffx_Laser6 | fffx_OutOfAmmo | fffx_LightningGun
        | fffx_Missile | fffx_GatlingGun => {
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 2, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 39000, 0, 220);
                    IN_AddRumbleState(s, 0, 0, 10);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        fffx_ShortPlasma | fffx_PlasmaCannon1 | fffx_PlasmaCannon2
        | fffx_Cannon => {
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 1, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 25000, 10000, 230);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        #[cfg(feature = "xbox")]
        fffx_FallingShort | fffx_FallingMedium => {
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 1, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 25000, 10000, 230);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        #[cfg(feature = "xbox")]
        fffx_FallingFar => {
            unsafe {
                s = IN_CreateRumbleScript(IN_GetMainController(), 1, 1);
                if s != -1 {
                    IN_AddRumbleState(s, 32000, 10000, 230);
                    IN_ExecuteRumbleScript(s);
                }
            }
        }
        #[cfg(feature = "xbox")]
        fffx_StartConst => {
            unsafe {
                client = IN_GetMainController();
                if const_rumble[client as usize] == -1 {
                    const_rumble[client as usize] = IN_CreateRumbleScript(IN_GetMainController(), 9, 1);
                    if const_rumble[client as usize] != -1 {
                        IN_AddEffectFade4(const_rumble[client as usize], 0, 0, 50000, 0, 2000);
                        IN_AddRumbleState(const_rumble[client as usize], 50000, 0, 300);
                        IN_AddEffectFade4(const_rumble[client as usize], 50000, 50000, 0, 0, 1000);
                        IN_ExecuteRumbleScript(const_rumble[client as usize]);
                    }
                }
            }
        }
        #[cfg(feature = "xbox")]
        fffx_StopConst => {
            unsafe {
                client = IN_GetMainController();
                if const_rumble[client as usize] == -1 {
                    return;
                }
                IN_KillRumbleScript(const_rumble[client as usize]);
                const_rumble[client as usize] = -1;
            }
        }
        _ => {
            unsafe {
                Com_Printf(c"No rumble script is defined for fffx_id = %i\n".as_ptr(), effect as c_int);
            }
        }
    }
}

/*********
FF_XboxShake

intensity	- speed of rumble
duration	- length of rumble
*********/
const FF_SH_MIN_MOTOR_SPEED: c_int = 20000;
const FF_SH_MOTOR_SPEED_MODIFIER: c_int = 65535 - FF_SH_MIN_MOTOR_SPEED;

#[cfg(feature = "xbox")]
pub extern "C" fn FF_XboxShake(intensity: c_float, duration: c_int) {
    let mut s: c_int;
    unsafe {
        s = IN_CreateRumbleScript(IN_GetMainController(), 1, 1);
        if s != -1 {
            let mut speed: c_int;
            // figure out the speed
            speed = (FF_SH_MIN_MOTOR_SPEED) + ((FF_SH_MOTOR_SPEED_MODIFIER as f32 * intensity) as c_int);

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

#[cfg(feature = "xbox")]
pub extern "C" fn FF_XboxDamage(damage: c_int, xpos: c_float) {
    let mut s: c_int;
    unsafe {
        s = IN_CreateRumbleScript(IN_GetMainController(), 1, 1);
        if s != -1 {
            let mut leftMotorSpeed: c_int;
            let mut rightMotorSpeed: c_int;
            let mut duration: c_int;
            let mut per: c_float;

            duration = 175;

            // how much damage?
            if damage > 100 {
                per = 1.0;
            } else {
                per = (damage / 100) as c_float;
            }

            if xpos >= -0.2 && xpos <= 0.2 {	// damge to center
                leftMotorSpeed = rightMotorSpeed = (FF_DA_MIN_MOTOR_SPEED) + ((FF_DA_MOTOR_SPEED_MODIFIER as f32 * per) as c_int);
            } else if xpos > 0.2 {	// damage to right
                rightMotorSpeed = (FF_DA_MIN_MOTOR_SPEED) + ((FF_DA_MOTOR_SPEED_MODIFIER as f32 * per) as c_int);
                leftMotorSpeed = 0;
            } else {	// damage to left
                leftMotorSpeed = (FF_DA_MIN_MOTOR_SPEED) + ((FF_DA_MOTOR_SPEED_MODIFIER as f32 * per) as c_int);
                rightMotorSpeed = 0;
            }

            // Add the state and execute
            IN_AddRumbleState(s, leftMotorSpeed, rightMotorSpeed, duration);
            IN_ExecuteRumbleScript(s);
        }
    }
}
