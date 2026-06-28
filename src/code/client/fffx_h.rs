// Filename:-	fffx.h		(Force Feedback FX)

// this part can be seen by the CGAME as well...

// These enums match the generic ones built into the effects ROM in the MS SideWinder FF Joystick,
//	so blame MS for anything you don't like (like that aircraft carrier one - jeez!)...
//
// (Judging from the names of most of these, the MS FF guys appear to be rather fond of ID-type games...)
//
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ffFX_e {
    fffx_RandomNoise = 0,
    fffx_AircraftCarrierTakeOff,    // this one is pointless / dumb
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
    fffx_WindShear,     // also pretty crap
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
    // #ifdef _XBOX (Original C had these Xbox-only variants)
    #[cfg(xbox)]
    fffx_FallingShort,
    #[cfg(xbox)]
    fffx_FallingMedium,
    #[cfg(xbox)]
    fffx_FallingFar,
    #[cfg(xbox)]
    fffx_StartConst,
    #[cfg(xbox)]
    fffx_StopConst,
    // #endif
    //
    fffx_NUMBEROF,
    fffx_NULL,      // special use, ignore during array mallocs etc, use fffx_NUMBEROF instead
}

/////////////////////////// START of functions to call /////////////////////////////////
//
//
//
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
/// Start a force feedback effect
#[inline]
pub fn FFFX_START(f: ffFX_e) {
    FF_Play(f);
}

/// Ensure a force feedback effect is playing
#[inline]
pub fn FFFX_ENSURE(f: ffFX_e) {
    FF_EnsurePlaying(f);
}

/// Stop a force feedback effect (some effects (eg. gatling, chainsaw), need this, or they play too long after trigger-off.)
#[inline]
pub fn FFFX_STOP(f: ffFX_e) {
    FF_Stop(f);
}

/// Stop all force feedback effects
#[inline]
pub fn FFFX_STOPALL() {
    FF_StopAll();
}

//
// These 2 are called at app start/stop, but you can call FF_Init to change FF devices anytime (takes a couple of seconds)
//
extern "C" {
    /// Initialize force feedback
    pub fn FF_Init(bTryMouseFirst: core::ffi::c_int);

    /// Shutdown force feedback
    pub fn FF_Shutdown();

    //
    // other stuff you may want to call but don't have to...
    //
    /// Check if force feedback is available
    pub fn FF_IsAvailable() -> core::ffi::c_int;

    /// Check if using force feedback mouse
    pub fn FF_IsMouse() -> core::ffi::c_int;

    /// Set tension setting 0..3 (0=none)
    pub fn FF_SetTension(iTension: core::ffi::c_int) -> core::ffi::c_int;

    /// Precision version of tension: 0..n..10000
    /// (only provided for command line fiddling with
    /// weird hardware. FF_SetTension(1) = default
    /// = FF_SetSpring(2000) (internal lookup table))
    pub fn FF_SetSpring(lSpring: core::ffi::c_long) -> core::ffi::c_int;

    //
    //
    //
    /////////////////////////// END of functions to call /////////////////////////////////

    // do *not* call this functions directly (or else!), use the macros above
    //
    pub fn FF_Play(fffx: ffFX_e);
    pub fn FF_EnsurePlaying(fffx: ffFX_e);
    pub fn FF_Stop(fffx: ffFX_e);
    pub fn FF_StopAll();

    // #ifdef _XBOX
    #[cfg(xbox)]
    pub fn FF_XboxShake(intensity: core::ffi::c_float, duration: core::ffi::c_int);
    #[cfg(xbox)]
    pub fn FF_XboxDamage(damage: core::ffi::c_int, xpos: core::ffi::c_float);
    // #endif
}

pub const MAX_CONCURRENT_FFFXs: core::ffi::c_int = 4;  // only for my code to use/read, do NOT alter!

//////////////////////// eof //////////////////////
