//! `bg_strap.h` — shared trap-call declarations.

#![allow(non_snake_case)]

use crate::codemp::game::q_shared_h::{
    mdxaBone_t, qboolean, qhandle_t, sharedIKMoveParams_t, sharedRagDollParams_t,
    sharedRagDollUpdateParams_t, sharedSetBoneIKStateParams_t, vec3_t,
};
use core::ffi::{c_char, c_int, c_void};

unsafe extern "C" {
    pub fn strap_G2API_GetBoltMatrix(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: vec3_t,
        position: vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: vec3_t,
    ) -> qboolean;

    pub fn strap_G2API_GetBoltMatrix_NoReconstruct(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: vec3_t,
        position: vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: vec3_t,
    ) -> qboolean;

    pub fn strap_G2API_GetBoltMatrix_NoRecNoRot(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: vec3_t,
        position: vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: vec3_t,
    ) -> qboolean;

    pub fn strap_G2API_SetBoneAngles(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boneName: *const c_char,
        angles: vec3_t,
        flags: c_int,
        up: c_int,
        right: c_int,
        forward: c_int,
        modelList: *mut qhandle_t,
        blendTime: c_int,
        currentTime: c_int,
    ) -> qboolean;

    pub fn strap_G2API_SetBoneAnim(
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
    ) -> qboolean;

    pub fn strap_G2API_GetBoneAnim(
        ghoul2: *mut c_void,
        boneName: *const c_char,
        currentTime: c_int,
        currentFrame: *mut f32,
        startFrame: *mut c_int,
        endFrame: *mut c_int,
        flags: *mut c_int,
        animSpeed: *mut f32,
        modelList: *mut c_int,
        modelIndex: c_int,
    ) -> qboolean;

    pub fn strap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut sharedRagDollParams_t);

    pub fn strap_G2API_AnimateG2Models(
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    );

    pub fn strap_G2API_SetBoneIKState(
        ghoul2: *mut c_void,
        time: c_int,
        boneName: *const c_char,
        ikState: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean;

    pub fn strap_G2API_IKMove(
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedIKMoveParams_t,
    ) -> qboolean;

    pub fn strap_TrueMalloc(ptr: *mut *mut c_void, size: c_int);

    pub fn strap_TrueFree(ptr: *mut *mut c_void);
}
