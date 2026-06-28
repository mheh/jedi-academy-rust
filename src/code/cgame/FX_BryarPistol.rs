// Bryar Pistol Weapon Effects

// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"

// #include "cg_local.h"
// #include "cg_media.h"
// #include "FxScheduler.h"

use core::ffi::{c_int, c_char};

// Minimal struct definitions needed for this module.
// These stub definitions mirror the C layout for types used in this file.

#[repr(C)]
pub struct trajectory_t {
    pub trDelta: [f32; 3],
    pub trTime: c_int,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub pos: trajectory_t,
}

#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub owner: *mut gentity_t,
    pub count: c_int,
}

#[repr(C)]
pub struct centity_t {
    pub gent: *mut gentity_t,
    pub currentState: entityState_t,
    pub lerpOrigin: [f32; 3],
}

#[repr(C)]
pub struct cg_t {
    pub time: c_int,
    // ... other fields not used in this file
}

#[repr(C)]
pub struct cgs_effects_t {
    pub bryarShotEffect: *const c_char,
    pub bryarWallImpactEffect: *const c_char,
    pub bryarWallImpactEffect2: *const c_char,
    pub bryarWallImpactEffect3: *const c_char,
    pub bryarFleshImpactEffect: *const c_char,
    pub bryarPowerupShotEffect: *const c_char,
    // ... other fields not used in this file
}

#[repr(C)]
pub struct cgs_t {
    pub effects: cgs_effects_t,
    // ... other fields not used in this file
}

// External C function declarations
extern "C" {
    // Opaque type for weaponInfo_s - only passed as pointer, not dereferenced
    pub type weaponInfo_s;

    // Global variables
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static mut theFxScheduler: FxScheduler;

    // Vector functions
    fn VectorNormalize2(in_: *const [f32; 3], out: *mut [f32; 3]) -> f32;
    fn VectorScale(in_: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
}

#[repr(C)]
pub struct FxScheduler {
    _private: [u8; 0],
}

impl FxScheduler {
    pub unsafe fn PlayEffect(&self, effect: *const c_char, origin: *const [f32; 3], forward: *const [f32; 3]) {
        // stub - actual implementation from engine
        let _ = (effect, origin, forward, self);
    }
}

/*
-------------------------

	MAIN FIRE

-------------------------
FX_BryarProjectileThink
-------------------------
*/
pub unsafe fn FX_BryarProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s) {
    let mut forward = [0.0f32; 3];

    if VectorNormalize2(core::addr_of!((*cent).gent.s.pos.trDelta), core::addr_of_mut!(forward)) == 0.0f32 {
        if VectorNormalize2(core::addr_of!((*cent).currentState.pos.trDelta), core::addr_of_mut!(forward)) == 0.0f32 {
            forward[2] = 1.0f32;
        }
    }

    // hack the scale of the forward vector if we were just fired or bounced...this will shorten up the tail for a split second so tails don't clip so harshly
    let mut dif = cg.time - (*(*cent).gent).s.pos.trTime;

    if dif < 75 {
        if dif < 0 {
            dif = 0;
        }

        let scale = (dif as f32 / 75.0f32) * 0.95f32 + 0.05f32;

        VectorScale(core::addr_of!(forward), scale, core::addr_of_mut!(forward));
    }

    if !(*cent).gent.is_null() && !(*(*cent).gent).owner.is_null() && (*(*(*cent).gent).owner).s.number > 0 {
        theFxScheduler.PlayEffect(b"bryar/NPCshot\0".as_ptr() as *const c_char, core::addr_of!((*cent).lerpOrigin), core::addr_of!(forward));
    } else {
        theFxScheduler.PlayEffect(cgs.effects.bryarShotEffect, core::addr_of!((*cent).lerpOrigin), core::addr_of!(forward));
    }
}

/*
-------------------------
FX_BryarHitWall
-------------------------
*/
pub unsafe fn FX_BryarHitWall(origin: *mut [f32; 3], normal: *mut [f32; 3]) {
    theFxScheduler.PlayEffect(cgs.effects.bryarWallImpactEffect, origin as *const _, normal as *const _);
}

/*
-------------------------
FX_BryarHitPlayer
-------------------------
*/
pub unsafe fn FX_BryarHitPlayer(origin: *mut [f32; 3], normal: *mut [f32; 3], humanoid: c_int) {
    let _ = humanoid; // unused parameter
    theFxScheduler.PlayEffect(cgs.effects.bryarFleshImpactEffect, origin as *const _, normal as *const _);
}


/*
-------------------------

	ALT FIRE

-------------------------
FX_BryarAltProjectileThink
-------------------------
*/
pub unsafe fn FX_BryarAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s) {
    let mut forward = [0.0f32; 3];

    if VectorNormalize2(core::addr_of!((*cent).gent.s.pos.trDelta), core::addr_of_mut!(forward)) == 0.0f32 {
        if VectorNormalize2(core::addr_of!((*cent).currentState.pos.trDelta), core::addr_of_mut!(forward)) == 0.0f32 {
            forward[2] = 1.0f32;
        }
    }

    // hack the scale of the forward vector if we were just fired or bounced...this will shorten up the tail for a split second so tails don't clip so harshly
    let mut dif = cg.time - (*(*cent).gent).s.pos.trTime;

    if dif < 75 {
        if dif < 0 {
            dif = 0;
        }

        let scale = (dif as f32 / 75.0f32) * 0.95f32 + 0.05f32;

        VectorScale(core::addr_of!(forward), scale, core::addr_of_mut!(forward));
    }

    // see if we have some sort of extra charge going on
    for t in 1..(*(*cent).gent).count {
        // just add ourselves over, and over, and over when we are charged
        theFxScheduler.PlayEffect(cgs.effects.bryarPowerupShotEffect, core::addr_of!((*cent).lerpOrigin), core::addr_of!(forward));
    }

    theFxScheduler.PlayEffect(cgs.effects.bryarShotEffect, core::addr_of!((*cent).lerpOrigin), core::addr_of!(forward));
}

/*
-------------------------
FX_BryarAltHitWall
-------------------------
*/
pub unsafe fn FX_BryarAltHitWall(origin: *mut [f32; 3], normal: *mut [f32; 3], power: c_int) {
    match power {
        4 | 5 => {
            theFxScheduler.PlayEffect(cgs.effects.bryarWallImpactEffect3, origin as *const _, normal as *const _);
        }

        2 | 3 => {
            theFxScheduler.PlayEffect(cgs.effects.bryarWallImpactEffect2, origin as *const _, normal as *const _);
        }

        _ => {
            theFxScheduler.PlayEffect(cgs.effects.bryarWallImpactEffect, origin as *const _, normal as *const _);
        }
    }
}

/*
-------------------------
FX_BryarAltHitPlayer
-------------------------
*/
pub unsafe fn FX_BryarAltHitPlayer(origin: *mut [f32; 3], normal: *mut [f32; 3], humanoid: c_int) {
    let _ = humanoid; // unused parameter
    theFxScheduler.PlayEffect(cgs.effects.bryarFleshImpactEffect, origin as *const _, normal as *const _);
}
