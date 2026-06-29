#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use crate::codemp::game::q_shared_h::{vec3_t, PITCH, YAW, ROLL};
use crate::codemp::game::q_math::{VectorCopy, VectorSubtract, VectorNormalize, vectoangles, vec3_origin};
use crate::codemp::game::bg_public::BG_GiveMeVectorFromMatrix;
use crate::codemp::ghoul2::g2_h::{BONE_ANGLES_POSTMULT, BONE_ANGLES_REPLACE, BONE_ANIM_OVERRIDE_FREEZE};
use crate::codemp::game::q_shared_h::ORIGIN;
use crate::ffi::types::qboolean;

// ============================================================================
// Stubs for cgame types not yet fully ported
// ============================================================================

/// Stub for entityState_t: client entity state from the server.
/// Only the fields needed for cg_turret are included.
#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub modelindex: c_int,
    pub fireflag: c_int,
    pub bolt2: c_int,
    pub pos: trajectory_t,
    // ... rest of fields omitted
}

/// Stub for trajectory_t: position/velocity trajectory.
#[repr(C)]
pub struct trajectory_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
}

/// Stub for centity_t: client entity with position and rendering state.
#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub nextState: entityState_t,
    pub interpolate: qboolean,
    pub currentValid: qboolean,
    pub lerpOrigin: vec3_t,
    pub lerpAngles: vec3_t,
    pub torsoBolt: c_int,
    pub turAngles: vec3_t,
    pub bolt4: c_int,
    pub dustTrailTime: c_int,
    pub frame_minus1_refreshed: c_int,
    pub ghoul2: *mut c_void,
    pub modelScale: vec3_t,
    // ... rest of fields omitted
}

/// Stub for weaponInfo_s: weapon metadata.
#[repr(C)]
pub struct weaponInfo_s {
    pub registered: qboolean,
    // ... rest of fields omitted
}

pub type weaponInfo_t = weaponInfo_s;

/// Stub for mdxaBone_t: Ghoul2 bone matrix.
#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

// ============================================================================
// External functions - trap calls and engine functions
// ============================================================================

extern "C" {
    pub fn trap_G2API_InitGhoul2Model(
        ghoul2Ptr: *mut *mut c_void,
        fileName: *const c_char,
        modelIndex: c_int,
        customSkin: c_int,
        customShader: c_int,
        modelFlags: c_int,
        lodBias: c_int,
    );

    pub fn CG_ConfigString(index: c_int) -> *const c_char;

    pub fn trap_G2API_AddBolt(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int;

    pub fn trap_G2API_SetBoneAngles(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boneName: *const c_char,
        angles: *const vec3_t,
        flags: c_int,
        up: c_int,
        right: c_int,
        forward: c_int,
        modelList: *mut c_int,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn trap_G2API_SetBoneAnim(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boneName: *const c_char,
        startFrame: c_int,
        endFrame: c_int,
        flags: c_int,
        animSpeed: f32,
        currentTime: c_int,
        setFrame: f32,
        blendTime: c_int,
    ) -> c_int;

    pub fn CG_RegisterWeapon(weaponNum: c_int);

    pub fn trap_G2API_GetBoltMatrix(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        boltMatrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        origin: *const vec3_t,
        frameNum: c_int,
        modelList: *mut c_int,
        scale: *const vec3_t,
    ) -> c_int;

    pub fn trap_FX_PlayEffectID(
        fxHandle: c_int,
        origin: *const vec3_t,
        forward: *const vec3_t,
        dontKill: c_int,
        unk: c_int,
    );
}

// ============================================================================
// Global state
// ============================================================================

extern "C" {
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static mut cg_entities: [centity_t; 2048];
    pub static mut cg_weapons: [weaponInfo_t; 32];
}

/// Stub for cg_t: client game state.
#[repr(C)]
pub struct cg_t {
    pub time: c_int,
    // ... rest of fields omitted
}

/// Stub for cgs_t: client game static state.
#[repr(C)]
pub struct cgs_t {
    pub gameModels: [c_int; 256],
    pub effects: cgEffects_t,
    // ... rest of fields omitted
}

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub mTurretMuzzleFlash: c_int,
    // ... rest of fields omitted
}

// ============================================================================
// Constants
// ============================================================================

pub const WP_TURRET: c_int = 22;
pub const POSITIVE_Y: c_int = 0;
pub const POSITIVE_X: c_int = 0; // Unused but kept for completeness
pub const POSITIVE_Z: c_int = 0; // Unused but kept for completeness
pub const NEGATIVE_Y: c_int = 1;
pub const NEGATIVE_Z: c_int = 2;
pub const NEGATIVE_X: c_int = 3;

// ============================================================================
// Functions
// ============================================================================

//rww - The turret is heavily dependant on bone angles. We can't happily set that on the server, so it is done client-only.

pub fn CreepToPosition(ideal: &mut vec3_t, current: &mut vec3_t) {
    let max_degree_switch: f32 = 90.0;
    let mut degrees_negative: i32 = 0;
    let mut degrees_positive: i32 = 0;
    let mut doNegative: i32 = 0;

    let mut angle_ideal: i32;
    let mut angle_current: i32;

    angle_ideal = ideal[YAW] as i32;
    angle_current = current[YAW] as i32;

    if angle_ideal <= angle_current {
        degrees_negative = angle_current - angle_ideal;

        degrees_positive = (360 - angle_current) + angle_ideal;
    } else {
        degrees_negative = angle_current + (360 - angle_ideal);

        degrees_positive = angle_ideal - angle_current;
    }

    if degrees_negative < degrees_positive {
        doNegative = 1;
    }

    if doNegative != 0 {
        current[YAW] -= max_degree_switch;

        if current[YAW] < ideal[YAW] && (current[YAW] + (max_degree_switch * 2.0)) >= ideal[YAW] {
            current[YAW] = ideal[YAW];
        }

        if current[YAW] < 0.0 {
            current[YAW] += 361.0;
        }
    } else {
        current[YAW] += max_degree_switch;

        if current[YAW] > ideal[YAW] && (current[YAW] - (max_degree_switch * 2.0)) <= ideal[YAW] {
            current[YAW] = ideal[YAW];
        }

        if current[YAW] > 360.0 {
            current[YAW] -= 361.0;
        }
    }

    if ideal[PITCH] < 0.0 {
        ideal[PITCH] += 360.0;
    }

    angle_ideal = ideal[PITCH] as i32;
    angle_current = current[PITCH] as i32;

    doNegative = 0;

    if angle_ideal <= angle_current {
        degrees_negative = angle_current - angle_ideal;

        degrees_positive = (360 - angle_current) + angle_ideal;
    } else {
        degrees_negative = angle_current + (360 - angle_ideal);

        degrees_positive = angle_ideal - angle_current;
    }

    if degrees_negative < degrees_positive {
        doNegative = 1;
    }

    if doNegative != 0 {
        current[PITCH] -= max_degree_switch;

        if current[PITCH] < ideal[PITCH] && (current[PITCH] + (max_degree_switch * 2.0)) >= ideal[PITCH] {
            current[PITCH] = ideal[PITCH];
        }

        if current[PITCH] < 0.0 {
            current[PITCH] += 361.0;
        }
    } else {
        current[PITCH] += max_degree_switch;

        if current[PITCH] > ideal[PITCH] && (current[PITCH] - (max_degree_switch * 2.0)) <= ideal[PITCH] {
            current[PITCH] = ideal[PITCH];
        }

        if current[PITCH] > 360.0 {
            current[PITCH] -= 361.0;
        }
    }
}

pub fn TurretClientRun(ent: *mut centity_t) {
    unsafe {
        if (*ent).ghoul2.is_null() {
            let weaponInfo: *mut weaponInfo_t;

            trap_G2API_InitGhoul2Model(
                &mut (*ent).ghoul2,
                CG_ConfigString(20 + (*ent).currentState.modelindex), // CS_MODELS + ent->currentState.modelindex
                0,
                0,
                0,
                0,
                0,
            );

            if (*ent).ghoul2.is_null() {
                //bad
                return;
            }

            (*ent).torsoBolt = trap_G2API_AddBolt((*ent).ghoul2, 0, b"*flash02\0".as_ptr() as *const c_char);

            trap_G2API_SetBoneAngles(
                (*ent).ghoul2,
                0,
                b"bone_hinge\0".as_ptr() as *const c_char,
                &vec3_origin,
                BONE_ANGLES_POSTMULT,
                POSITIVE_Y,
                POSITIVE_Z,
                POSITIVE_X,
                core::ptr::null_mut(),
                100,
                cg.time,
            );
            trap_G2API_SetBoneAngles(
                (*ent).ghoul2,
                0,
                b"bone_gback\0".as_ptr() as *const c_char,
                &vec3_origin,
                BONE_ANGLES_POSTMULT,
                POSITIVE_Y,
                POSITIVE_Z,
                POSITIVE_X,
                core::ptr::null_mut(),
                100,
                cg.time,
            );
            trap_G2API_SetBoneAngles(
                (*ent).ghoul2,
                0,
                b"bone_barrel\0".as_ptr() as *const c_char,
                &vec3_origin,
                BONE_ANGLES_POSTMULT,
                POSITIVE_Y,
                POSITIVE_Z,
                POSITIVE_X,
                core::ptr::null_mut(),
                100,
                cg.time,
            );

            trap_G2API_SetBoneAnim(
                (*ent).ghoul2,
                0,
                b"model_root\0".as_ptr() as *const c_char,
                0,
                11,
                BONE_ANIM_OVERRIDE_FREEZE,
                0.8,
                cg.time,
                0.0,
                0,
            );

            (*ent).turAngles[ROLL] = 0.0;
            (*ent).turAngles[PITCH] = 90.0;
            (*ent).turAngles[YAW] = 0.0;

            weaponInfo = &mut cg_weapons[WP_TURRET as usize];

            if !(*weaponInfo).registered {
                CG_RegisterWeapon(WP_TURRET);
            }
        }

        if (*ent).currentState.fireflag == 2 {
            //I'm about to blow
            // In C, ent->turAngles is an embedded array, so the pointer is always valid.
            // The original C condition is always true.
            trap_G2API_SetBoneAngles(
                (*ent).ghoul2,
                0,
                b"bone_hinge\0".as_ptr() as *const c_char,
                &(*ent).turAngles,
                BONE_ANGLES_REPLACE,
                NEGATIVE_Y,
                NEGATIVE_Z,
                NEGATIVE_X,
                core::ptr::null_mut(),
                100,
                cg.time,
            );
            return;
        } else if (*ent).currentState.fireflag != 0 && (*ent).bolt4 != (*ent).currentState.fireflag {
            let mut muzzleOrg: vec3_t = [0.0; 3];
            let mut muzzleDir: vec3_t = [0.0; 3];
            let mut boltMatrix: mdxaBone_t = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };

            trap_G2API_GetBoltMatrix(
                (*ent).ghoul2,
                0,
                (*ent).torsoBolt,
                &mut boltMatrix,
                /*ent->lerpAngles*/&vec3_origin,
                &(*ent).lerpOrigin,
                cg.time,
                &mut cgs.gameModels[0],
                &(*ent).modelScale,
            );
            BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzleOrg);
            BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_X, &mut muzzleDir);

            trap_FX_PlayEffectID(cgs.effects.mTurretMuzzleFlash, &muzzleOrg, &muzzleDir, -1, -1);

            (*ent).bolt4 = (*ent).currentState.fireflag;
        } else if (*ent).currentState.fireflag == 0 {
            (*ent).bolt4 = 0;
        }

        if (*ent).currentState.bolt2 != 2048 {
            //turn toward the enemy
            let enemy: *mut centity_t = &mut cg_entities[(*ent).currentState.bolt2 as usize];

            if !enemy.is_null() {
                let mut enAng: vec3_t = [0.0; 3];
                let mut enPos: vec3_t = [0.0; 3];

                VectorCopy(&(*enemy).currentState.pos.trBase, &mut enPos);

                VectorSubtract(&enPos, &(*ent).lerpOrigin, &mut enAng);
                VectorNormalize(&mut enAng);
                vectoangles(&enAng, &mut enAng);
                enAng[ROLL] = 0.0;
                enAng[PITCH] += 90.0;

                CreepToPosition(&mut enAng, &mut (*ent).turAngles);
            }
        } else {
            let mut idleAng: vec3_t = [0.0; 3];
            let mut turnAmount: f32;

            if (*ent).turAngles[YAW] > 360.0 {
                (*ent).turAngles[YAW] -= 361.0;
            }

            if (*ent).dustTrailTime == 0 {
                (*ent).dustTrailTime = cg.time;
            }

            turnAmount = ((cg.time - (*ent).dustTrailTime) as f32) * 0.03;

            if turnAmount > 360.0 {
                turnAmount = 360.0;
            }

            idleAng[PITCH] = 90.0;
            idleAng[ROLL] = 0.0;
            idleAng[YAW] = (*ent).turAngles[YAW] + turnAmount;
            (*ent).dustTrailTime = cg.time;

            CreepToPosition(&mut idleAng, &mut (*ent).turAngles);
        }

        if cg.time < (*ent).frame_minus1_refreshed {
            (*ent).frame_minus1_refreshed = cg.time;
            return;
        }

        (*ent).frame_minus1_refreshed = cg.time;
        trap_G2API_SetBoneAngles(
            (*ent).ghoul2,
            0,
            b"bone_hinge\0".as_ptr() as *const c_char,
            &(*ent).turAngles,
            BONE_ANGLES_REPLACE,
            NEGATIVE_Y,
            NEGATIVE_Z,
            NEGATIVE_X,
            core::ptr::null_mut(),
            100,
            cg.time,
        );
    }
}
