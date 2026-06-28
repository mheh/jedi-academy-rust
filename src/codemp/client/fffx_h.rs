//! Mechanical port of `codemp/client/fffx.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

#[cfg(not(feature = "cgame_only"))]
use crate::codemp::game::q_shared_h::qboolean;
#[cfg(not(feature = "cgame_only"))]
use core::ffi::{c_float, c_long};
use core::ffi::c_int;

// this part can be seen by the CGAME as well...

// These enums match the generic ones built into the effects ROM in the MS SideWinder FF Joystick,
//	so blame MS for anything you don't like (like that aircraft carrier one - jeez!)...
//
// (Judging from the names of most of these, the MS FF guys appear to be rather fond of ID-type games...)
//

/// `ffFX_e` (`fffx.h`) — C `typedef enum`, stored/passed as `int`.
pub type ffFX_e = c_int;

pub const fffx_RandomNoise: ffFX_e = 0;
pub const fffx_AircraftCarrierTakeOff: ffFX_e = 1; // this one is pointless / dumb
pub const fffx_BasketballDribble: ffFX_e = 2;
pub const fffx_CarEngineIdle: ffFX_e = 3;
pub const fffx_ChainsawIdle: ffFX_e = 4;
pub const fffx_ChainsawInAction: ffFX_e = 5;
pub const fffx_DieselEngineIdle: ffFX_e = 6;
pub const fffx_Jump: ffFX_e = 7;
pub const fffx_Land: ffFX_e = 8;
pub const fffx_MachineGun: ffFX_e = 9;
pub const fffx_Punched: ffFX_e = 10;
pub const fffx_RocketLaunch: ffFX_e = 11;
pub const fffx_SecretDoor: ffFX_e = 12;
pub const fffx_SwitchClick: ffFX_e = 13;
pub const fffx_WindGust: ffFX_e = 14;
pub const fffx_WindShear: ffFX_e = 15; // also pretty crap
pub const fffx_Pistol: ffFX_e = 16;
pub const fffx_Shotgun: ffFX_e = 17;
pub const fffx_Laser1: ffFX_e = 18;
pub const fffx_Laser2: ffFX_e = 19;
pub const fffx_Laser3: ffFX_e = 20;
pub const fffx_Laser4: ffFX_e = 21;
pub const fffx_Laser5: ffFX_e = 22;
pub const fffx_Laser6: ffFX_e = 23;
pub const fffx_OutOfAmmo: ffFX_e = 24;
pub const fffx_LightningGun: ffFX_e = 25;
pub const fffx_Missile: ffFX_e = 26;
pub const fffx_GatlingGun: ffFX_e = 27;
pub const fffx_ShortPlasma: ffFX_e = 28;
pub const fffx_PlasmaCannon1: ffFX_e = 29;
pub const fffx_PlasmaCannon2: ffFX_e = 30;
pub const fffx_Cannon: ffFX_e = 31;
#[cfg(feature = "xbox")]
pub const fffx_FallingShort: ffFX_e = 32;
#[cfg(feature = "xbox")]
pub const fffx_FallingMedium: ffFX_e = 33;
#[cfg(feature = "xbox")]
pub const fffx_FallingFar: ffFX_e = 34;
#[cfg(feature = "xbox")]
pub const fffx_StartConst: ffFX_e = 35;
#[cfg(feature = "xbox")]
pub const fffx_StopConst: ffFX_e = 36;
//
#[cfg(not(feature = "xbox"))]
pub const fffx_NUMBEROF: ffFX_e = 32;
#[cfg(feature = "xbox")]
pub const fffx_NUMBEROF: ffFX_e = 37;
pub const fffx_NULL: ffFX_e = fffx_NUMBEROF + 1; // special use, ignore during array mallocs etc, use fffx_NUMBEROF instead

// Once the game is running you should *only* access FF functions through these 4 macros. Savvy?
//
// Usage note: In practice, you can have about 4 FF FX running concurrently, though I defy anyone to make sense
//	of that amount of white-noise vibration. MS guidelines say you shouldn't leave vibration on for long periods
//	of time (eg engine rumble) because of various nerve-damage/lawsuit issues, so for this reason there is no API
//	support for setting durations etc. All FF FX stuff here is designed for things like firing, hit damage, driving
//	over bumps, etc that can be played as one-off events, though you *can* do things like FFFX_ENSURE(fffx_ChainsawIdle)
//	if you really want to.
//
// Combining small numbers of effects such as having a laser firing (MS photons have higher mass apparently <g>),
//	and then firing a machine gun in bursts as well are no problem, and easily felt, hence the ability to stop playing
//	individual FF FX.
//

// Rust has no C preprocessor object/function macros at item scope; keep the public
// macro names as thin wrappers over the original targets.
#[cfg(not(feature = "cgame_only"))]
#[inline]
pub unsafe fn FFFX_START(f: ffFX_e) {
    unsafe { FF_Play(f) }
}

#[cfg(not(feature = "cgame_only"))]
#[inline]
pub unsafe fn FFFX_ENSURE(f: ffFX_e) {
    unsafe { FF_EnsurePlaying(f) }
}

#[cfg(not(feature = "cgame_only"))]
#[inline]
pub unsafe fn FFFX_STOP(f: ffFX_e) {
    unsafe { FF_Stop(f) }
}

#[cfg(not(feature = "cgame_only"))]
#[inline]
pub unsafe fn FFFX_STOPALL() {
    unsafe { FF_StopAll() }
}

#[cfg(not(feature = "cgame_only"))]
unsafe extern "C" {
    //
    // These 2 are called at app start/stop, but you can call FF_Init to change FF devices anytime (takes a couple of seconds)
    //
    // C++ default argument `bTryMouseFirst=true` is not represented in Rust FFI.
    pub fn FF_Init(bTryMouseFirst: qboolean);
    pub fn FF_Shutdown();
    //
    // other stuff you may want to call but don't have to...
    //
    pub fn FF_IsAvailable() -> qboolean;
    pub fn FF_IsMouse() -> qboolean;
    pub fn FF_SetTension(iTension: c_int) -> qboolean; // tension setting 0..3 (0=none)
    pub fn FF_SetSpring(lSpring: c_long) -> qboolean; // precision version of above, 0..n..10000
                                                       // (only provided for command line fiddling with
                                                       //	weird hardware. FF_SetTension(1) = default
                                                       // = FF_SetSpring(2000) (internal lookup table))

    //
    //
    //

    // do *not* call this functions directly (or else!), use the macros above
    //
    pub fn FF_Play(fffx: ffFX_e);
    pub fn FF_EnsurePlaying(fffx: ffFX_e);
    pub fn FF_Stop(fffx: ffFX_e);
    pub fn FF_StopAll();

    #[cfg(feature = "xbox")]
    pub fn FF_XboxShake(intensity: c_float, duration: c_int);
    #[cfg(feature = "xbox")]
    pub fn FF_XboxDamage(damage: c_int, xpos: c_float);
}

#[cfg(not(feature = "cgame_only"))]
pub const MAX_CONCURRENT_FFFXs: c_int = 4; // only for my code to use/read, do NOT alter!

//////////////////////// eof //////////////////////
