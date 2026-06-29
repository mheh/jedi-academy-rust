// Bryar Pistol Weapon Effects

use core::ffi::{c_int, c_char};

// ============================================================================
// Type definitions
// ============================================================================

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

// ============================================================================
// External struct types - stubs for FFI
// ============================================================================

#[repr(C)]
pub struct trajectory_t {
    pub trDelta: vec3_t,
    // Additional fields omitted - defined in cg_local.h
}

#[repr(C)]
pub struct entityState_t {
    pub pos: trajectory_t,
    pub generic1: c_int,
    // Additional fields omitted - defined in cg_local.h
}

#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub lerpOrigin: vec3_t,
    // Additional fields omitted - defined in cg_local.h
}

#[repr(C)]
pub struct weaponInfo_s {
    // Opaque - not dereferenced in this file
}

#[repr(C)]
pub struct cgs_effects_t {
    pub bryarShotEffect: c_int,
    pub bryarWallImpactEffect: c_int,
    pub bryarFleshImpactEffect: c_int,
    pub bryarDroidImpactEffect: c_int,
    pub bryarPowerupShotEffect: c_int,
    pub bryarWallImpactEffect2: c_int,
    pub bryarWallImpactEffect3: c_int,
    pub turretShotEffect: c_int,
    pub concussionImpactEffect: c_int,
    pub concussionShotEffect: c_int,
    // Additional fields omitted - defined in fx_local.h
}

#[repr(C)]
pub struct cgs_t {
    pub effects: cgs_effects_t,
    // Additional fields omitted - defined in cg_local.h
}

// ============================================================================
// External declarations
// ============================================================================

extern "C" {
    pub static mut cgs: cgs_t;

    fn VectorNormalize2(from: *const vec3_t, to: *mut vec3_t) -> f32;
    fn trap_FX_PlayEffectID(
        fx_id: c_int,
        origin: *const vec3_t,
        normal: *const vec3_t,
        arg3: c_int,
        arg4: c_int,
    );
    fn trap_FX_AddLine(
        start: *const vec3_t,
        end: *const vec3_t,
        param1: f32,
        param2: f32,
        param3: f32,
        param4: f32,
        param5: f32,
        param6: f32,
        color1: *const vec3_t,
        color2: *const vec3_t,
        param9: f32,
        flags: c_int,
        shader: c_int,
        fx_flags: c_int,
    );
    fn trap_R_RegisterShader(name: *const c_char) -> c_int;
}

// FX flags
const FX_SIZE_LINEAR: c_int = 0x0001;
const FX_ALPHA_LINEAR: c_int = 0x0002;

// ============================================================================
// Static data
// ============================================================================

static WHITE: vec3_t = [1.0f32, 1.0f32, 1.0f32];
static BRIGHT: vec3_t = [0.75f32, 0.5f32, 1.0f32];

// ============================================================================
// Function definitions
// ============================================================================

/*
-------------------------

	MAIN FIRE

-------------------------
FX_BryarProjectileThink
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_BryarProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0f32; 3];

    unsafe {
        if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
            forward[2] = 1.0f32;
        }

        trap_FX_PlayEffectID(
            cgs.effects.bryarShotEffect,
            &(*cent).lerpOrigin,
            &forward,
            -1,
            -1,
        );
    }
}

/*
-------------------------
FX_BryarHitWall
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_BryarHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    unsafe {
        trap_FX_PlayEffectID(cgs.effects.bryarWallImpactEffect, origin, normal, -1, -1);
    }
}

/*
-------------------------
FX_BryarHitPlayer
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_BryarHitPlayer(origin: *const vec3_t, normal: *const vec3_t, humanoid: qboolean) {
    unsafe {
        if humanoid != 0 {
            trap_FX_PlayEffectID(cgs.effects.bryarFleshImpactEffect, origin, normal, -1, -1);
        } else {
            trap_FX_PlayEffectID(cgs.effects.bryarDroidImpactEffect, origin, normal, -1, -1);
        }
    }
}


/*
-------------------------

	ALT FIRE

-------------------------
FX_BryarAltProjectileThink
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_BryarAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0f32; 3];

    unsafe {
        if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
            forward[2] = 1.0f32;
        }

        // see if we have some sort of extra charge going on
        let mut t = 1;
        while t < (*cent).currentState.generic1 {
            // just add ourselves over, and over, and over when we are charged
            trap_FX_PlayEffectID(cgs.effects.bryarPowerupShotEffect, &(*cent).lerpOrigin, &forward, -1, -1);
            t = t + 1;
        }

        //	for ( int t = 1; t < cent->gent->count; t++ )	// The single player stores the charge in count, which isn't accessible on the client

        trap_FX_PlayEffectID(cgs.effects.bryarShotEffect, &(*cent).lerpOrigin, &forward, -1, -1);
    }
}

/*
-------------------------
FX_BryarAltHitWall
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_BryarAltHitWall(origin: *const vec3_t, normal: *const vec3_t, power: c_int) {
    unsafe {
        match power {
            4 | 5 => {
                trap_FX_PlayEffectID(cgs.effects.bryarWallImpactEffect3, origin, normal, -1, -1);
            }

            2 | 3 => {
                trap_FX_PlayEffectID(cgs.effects.bryarWallImpactEffect2, origin, normal, -1, -1);
            }

            _ => {
                trap_FX_PlayEffectID(cgs.effects.bryarWallImpactEffect, origin, normal, -1, -1);
            }
        }
    }
}

/*
-------------------------
FX_BryarAltHitPlayer
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_BryarAltHitPlayer(origin: *const vec3_t, normal: *const vec3_t, humanoid: qboolean) {
    unsafe {
        if humanoid != 0 {
            trap_FX_PlayEffectID(cgs.effects.bryarFleshImpactEffect, origin, normal, -1, -1);
        } else {
            trap_FX_PlayEffectID(cgs.effects.bryarDroidImpactEffect, origin, normal, -1, -1);
        }
    }
}


//TURRET
/*
-------------------------
FX_TurretProjectileThink
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_TurretProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0f32; 3];

    unsafe {
        if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
            forward[2] = 1.0f32;
        }

        trap_FX_PlayEffectID(cgs.effects.turretShotEffect, &(*cent).lerpOrigin, &forward, -1, -1);
    }
}

/*
-------------------------
FX_TurretHitWall
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_TurretHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    unsafe {
        trap_FX_PlayEffectID(cgs.effects.bryarWallImpactEffect, origin, normal, -1, -1);
    }
}

/*
-------------------------
FX_TurretHitPlayer
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_TurretHitPlayer(origin: *const vec3_t, normal: *const vec3_t, humanoid: qboolean) {
    unsafe {
        if humanoid != 0 {
            trap_FX_PlayEffectID(cgs.effects.bryarFleshImpactEffect, origin, normal, -1, -1);
        } else {
            trap_FX_PlayEffectID(cgs.effects.bryarDroidImpactEffect, origin, normal, -1, -1);
        }
    }
}



//CONCUSSION (yeah, should probably make a new file for this.. or maybe just move all these stupid semi-redundant fx_ functions into one file)
/*
-------------------------
FX_ConcussionHitWall
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_ConcussionHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    unsafe {
        trap_FX_PlayEffectID(cgs.effects.concussionImpactEffect, origin, normal, -1, -1);
    }
}

/*
-------------------------
FX_ConcussionHitPlayer
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_ConcussionHitPlayer(origin: *const vec3_t, normal: *const vec3_t, humanoid: qboolean) {
    unsafe {
        trap_FX_PlayEffectID(cgs.effects.concussionImpactEffect, origin, normal, -1, -1);
    }
}

/*
-------------------------
FX_ConcussionProjectileThink
-------------------------
*/
#[allow(non_snake_case)]
pub fn FX_ConcussionProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0f32; 3];

    unsafe {
        if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
            forward[2] = 1.0f32;
        }

        trap_FX_PlayEffectID(cgs.effects.concussionShotEffect, &(*cent).lerpOrigin, &forward, -1, -1);
    }
}

/*
---------------------------
FX_ConcAltShot
---------------------------
*/
#[allow(non_snake_case)]
pub fn FX_ConcAltShot(start: *const vec3_t, end: *const vec3_t) {
    unsafe {
        //"concussion/beam"
        trap_FX_AddLine(
            start,
            end,
            0.1f32,
            10.0f32,
            0.0f32,
            1.0f32,
            0.0f32,
            0.0f32,
            &WHITE,
            &WHITE,
            0.0f32,
            175,
            trap_R_RegisterShader("gfx/effects/blueLine\0".as_ptr() as *const c_char),
            FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        );

        // add some beef
        trap_FX_AddLine(
            start,
            end,
            0.1f32,
            7.0f32,
            0.0f32,
            1.0f32,
            0.0f32,
            0.0f32,
            &BRIGHT,
            &BRIGHT,
            0.0f32,
            150,
            trap_R_RegisterShader("gfx/misc/whiteline2\0".as_ptr() as *const c_char),
            FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        );
    }
}
