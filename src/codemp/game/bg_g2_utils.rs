//! `bg_g2_utils.c` — "both games" ghoul2 bolt/surface helpers, *"all completely
//! stateless"* (the upstream file comment). Two functions, both **engine-trap-facing**
//! (they call `trap_G2API_*` syscalls), so — like the rest of the ghoul2-trap surface
//! (`BG_G2*Angles`, `BG_IK_MoveArm`) — they are faithful ports with **no C oracle**.
//!
//! - [`BG_AttachToRancor`] — compute the world transform of a bolt (the Rancor's mouth
//!   `"jaw_bone"` or right hand `"*r_hand"`) for something it is carrying, writing the
//!   chosen origin/axis/angles out through the (nullable) `out_*` pointers. Pure matrix
//!   math over [`G2API_GetBoltMatrix`](crate::trap::G2API_GetBoltMatrix)'s output via
//!   [`BG_GiveMeVectorFromMatrix`].
//! - [`BG_GetRootSurfNameWithVariant`] — pick the on-screen variant (`<name>`, then
//!   `<name>a`..`<name>h`) of a root surface that is currently rendered.
//!
//! Neither has a live caller yet (the Rancor NPC and the model-variant logic land in
//! later stages); they are ported now because the whole file is portable — its only
//! prerequisites were the two `trap_G2API_AddBolt` / `trap_G2API_GetSurfaceRenderStatus`
//! wrappers (mechanical ABI additions, the ghoul2-cluster precedent). This completes
//! `bg_g2_utils.c` for the server build.

#![allow(non_upper_case_globals, non_snake_case)]

use crate::codemp::game::bg_public::BG_GiveMeVectorFromMatrix;
use crate::codemp::game::q_math::{vectoangles, VectorSet};
use crate::codemp::game::q_shared::{Com_sprintf, Q_strncpyz, Sz};
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, qboolean, qhandle_t, vec3_t, NEGATIVE_X, NEGATIVE_Y, ORIGIN, PITCH, POSITIVE_X,
    POSITIVE_Z, QFALSE, QTRUE, ROLL,
};
use crate::trap;
use core::ffi::{c_char, c_int, c_void};

/// Bolt-attach helper — the C `void BG_AttachToRancor(...)`. `out_origin`, `out_angles`
/// and `out_axis` mirror the C `vec3_t` out-params and may each be NULL (the C guards
/// every write with `if ( out_x )`), so they are raw pointers here; `out_axis` is the
/// three-vector array `vec3_t out_axis[3]`.
#[allow(clippy::too_many_arguments)]
pub unsafe fn BG_AttachToRancor(
    ghoul2: *mut c_void,
    rancYaw: f32,
    rancOrigin: &vec3_t,
    time: c_int,
    modelList: *mut qhandle_t,
    modelScale: &vec3_t,
    inMouth: qboolean,
    out_origin: *mut vec3_t,
    out_angles: *mut vec3_t,
    out_axis: *mut [vec3_t; 3],
) {
    let mut boltMatrix = mdxaBone_t::default();
    let boltIndex: c_int;
    let mut rancAngles: vec3_t = [0.0; 3];
    let mut temp_angles: vec3_t = [0.0; 3];
    // Getting the bolt here
    if inMouth != QFALSE {
        //in mouth
        boltIndex = trap::G2API_AddBolt(ghoul2, 0, "jaw_bone");
    } else {
        //in right hand
        boltIndex = trap::G2API_AddBolt(ghoul2, 0, "*r_hand");
    }
    VectorSet(&mut rancAngles, 0.0, rancYaw, 0.0);
    trap::G2API_GetBoltMatrix(
        ghoul2,
        0,
        boltIndex,
        &mut boltMatrix,
        &rancAngles,
        rancOrigin,
        time,
        modelList,
        modelScale,
    );
    // Storing ent position, bolt position, and bolt axis
    if !out_origin.is_null() {
        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut *out_origin);
    }
    if !out_axis.is_null() {
        let axis = &mut *out_axis;
        if inMouth != QFALSE {
            //in mouth
            BG_GiveMeVectorFromMatrix(&boltMatrix, POSITIVE_Z, &mut axis[0]);
            BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut axis[1]);
            BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_X, &mut axis[2]);
        } else {
            //in hand
            BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut axis[0]);
            BG_GiveMeVectorFromMatrix(&boltMatrix, POSITIVE_X, &mut axis[1]);
            BG_GiveMeVectorFromMatrix(&boltMatrix, POSITIVE_Z, &mut axis[2]);
        }
        //FIXME: this is messing up our axis and turning us inside-out?
        if !out_angles.is_null() {
            vectoangles(&axis[0], &mut *out_angles);
            vectoangles(&axis[2], &mut temp_angles);
            (*out_angles)[ROLL] = -temp_angles[PITCH];
        }
    } else if !out_angles.is_null() {
        let mut temp_axis: [vec3_t; 3] = [[0.0; 3]; 3];
        if inMouth != QFALSE {
            //in mouth
            BG_GiveMeVectorFromMatrix(&boltMatrix, POSITIVE_Z, &mut temp_axis[0]);
            BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_X, &mut temp_axis[2]);
        } else {
            //in hand
            BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut temp_axis[0]);
            BG_GiveMeVectorFromMatrix(&boltMatrix, POSITIVE_Z, &mut temp_axis[2]);
        }
        //FIXME: this is messing up our axis and turning us inside-out?
        vectoangles(&temp_axis[0], &mut *out_angles);
        vectoangles(&temp_axis[2], &mut temp_angles);
        (*out_angles)[ROLL] = -temp_angles[PITCH];
    }
}

const MAX_VARIANTS: c_int = 8;

/// Root-surface variant picker — the C `qboolean BG_GetRootSurfNameWithVariant(...)`.
/// Returns the rendered variant of `rootSurfName` (the plain name if it is on, else the
/// first of `<name>a`..`<name>h` that is rendered) in `returnSurfName`; returns `qtrue`
/// if a rendered name was found, `qfalse` otherwise (falling back to the plain name).
pub unsafe fn BG_GetRootSurfNameWithVariant(
    ghoul2: *mut c_void,
    rootSurfName: *const c_char,
    returnSurfName: *mut c_char,
    returnSize: c_int,
) -> qboolean {
    if ghoul2.is_null() || trap::G2API_GetSurfaceRenderStatus(ghoul2, 0, rootSurfName) == 0 {
        //see if the basic name without variants is on
        Q_strncpyz(returnSurfName, rootSurfName, returnSize);
        return QTRUE;
    } else {
        //check variants
        for i in 0..MAX_VARIANTS {
            Com_sprintf(
                returnSurfName,
                returnSize,
                format_args!("{}{}", Sz(rootSurfName), (b'a' + i as u8) as char),
            );
            if trap::G2API_GetSurfaceRenderStatus(ghoul2, 0, returnSurfName) == 0 {
                return QTRUE;
            }
        }
    }
    Q_strncpyz(returnSurfName, rootSurfName, returnSize);
    QFALSE
}
