// rww - shared trap call system

use core::ffi::{c_int, c_char, c_void};

// Type declarations for function signatures
// These correspond to C types used in trap functions
pub type qboolean = c_int;
pub type qhandle_t = c_int;
pub type vec3_t = [f32; 3];

// Opaque types for structures defined in headers
#[repr(C)]
pub struct mdxaBone_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct sharedRagDollParams_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct sharedRagDollUpdateParams_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct sharedSetBoneIKStateParams_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct sharedIKMoveParams_t {
    _opaque: [u8; 0],
}

// Extern trap functions
extern "C" {
    fn trap_G2API_GetBoltMatrix(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: vec3_t,
    ) -> qboolean;

    fn trap_G2API_GetBoltMatrix_NoReconstruct(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: vec3_t,
    ) -> qboolean;

    fn trap_G2API_GetBoltMatrix_NoRecNoRot(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: vec3_t,
    ) -> qboolean;

    fn trap_G2API_SetBoneAngles(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boneName: *const c_char,
        angles: *const vec3_t,
        flags: c_int,
        up: c_int,
        right: c_int,
        forward: c_int,
        modelList: *mut qhandle_t,
        blendTime: c_int,
        currentTime: c_int,
    ) -> qboolean;

    fn trap_G2API_SetBoneAnim(
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

    fn trap_G2API_GetBoneAnim(
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

    fn trap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut sharedRagDollParams_t);

    fn trap_G2API_AnimateG2Models(
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedRagDollUpdateParams_t,
    );

    fn trap_G2API_SetBoneIKState(
        ghoul2: *mut c_void,
        time: c_int,
        boneName: *const c_char,
        ikState: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean;

    fn trap_G2API_IKMove(
        ghoul2: *mut c_void,
        time: c_int,
        params: *mut sharedIKMoveParams_t,
    ) -> qboolean;

    fn trap_TrueMalloc(ptr: *mut *mut c_void, size: c_int);
    fn trap_TrueFree(ptr: *mut *mut c_void);
}

pub fn strap_G2API_GetBoltMatrix(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: *mut mdxaBone_t,
    angles: *const vec3_t,
    position: *const vec3_t,
    frameNum: c_int,
    modelList: *mut qhandle_t,
    scale: vec3_t,
) -> qboolean {
    unsafe {
        trap_G2API_GetBoltMatrix(
            ghoul2,
            modelIndex,
            boltIndex,
            matrix,
            angles,
            position,
            frameNum,
            modelList,
            scale,
        )
    }
}

pub fn strap_G2API_GetBoltMatrix_NoReconstruct(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: *mut mdxaBone_t,
    angles: *const vec3_t,
    position: *const vec3_t,
    frameNum: c_int,
    modelList: *mut qhandle_t,
    scale: vec3_t,
) -> qboolean {
    unsafe {
        trap_G2API_GetBoltMatrix_NoReconstruct(
            ghoul2,
            modelIndex,
            boltIndex,
            matrix,
            angles,
            position,
            frameNum,
            modelList,
            scale,
        )
    }
}

pub fn strap_G2API_GetBoltMatrix_NoRecNoRot(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: *mut mdxaBone_t,
    angles: *const vec3_t,
    position: *const vec3_t,
    frameNum: c_int,
    modelList: *mut qhandle_t,
    scale: vec3_t,
) -> qboolean {
    unsafe {
        trap_G2API_GetBoltMatrix_NoRecNoRot(
            ghoul2,
            modelIndex,
            boltIndex,
            matrix,
            angles,
            position,
            frameNum,
            modelList,
            scale,
        )
    }
}

pub fn strap_G2API_SetBoneAngles(
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: *const c_char,
    angles: *const vec3_t,
    flags: c_int,
    up: c_int,
    right: c_int,
    forward: c_int,
    modelList: *mut qhandle_t,
    blendTime: c_int,
    currentTime: c_int,
) -> qboolean {
    unsafe {
        trap_G2API_SetBoneAngles(
            ghoul2,
            modelIndex,
            boneName,
            angles,
            flags,
            up,
            right,
            forward,
            modelList,
            blendTime,
            currentTime,
        )
    }
}

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
) -> qboolean {
    unsafe {
        trap_G2API_SetBoneAnim(
            ghoul2,
            modelIndex,
            boneName,
            startFrame,
            endFrame,
            flags,
            animSpeed,
            currentTime,
            setFrame,
            blendTime,
        )
    }
}

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
) -> qboolean {
    unsafe {
        trap_G2API_GetBoneAnim(
            ghoul2,
            boneName,
            currentTime,
            currentFrame,
            startFrame,
            endFrame,
            flags,
            animSpeed,
            modelList,
            modelIndex,
        )
    }
}

pub fn strap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut sharedRagDollParams_t) {
    unsafe {
        trap_G2API_SetRagDoll(ghoul2, params);
    }
}

pub fn strap_G2API_AnimateG2Models(
    ghoul2: *mut c_void,
    time: c_int,
    params: *mut sharedRagDollUpdateParams_t,
) {
    unsafe {
        trap_G2API_AnimateG2Models(ghoul2, time, params);
    }
}

pub fn strap_G2API_SetBoneIKState(
    ghoul2: *mut c_void,
    time: c_int,
    boneName: *const c_char,
    ikState: c_int,
    params: *mut sharedSetBoneIKStateParams_t,
) -> qboolean {
    unsafe { trap_G2API_SetBoneIKState(ghoul2, time, boneName, ikState, params) }
}

pub fn strap_G2API_IKMove(
    ghoul2: *mut c_void,
    time: c_int,
    params: *mut sharedIKMoveParams_t,
) -> qboolean {
    unsafe { trap_G2API_IKMove(ghoul2, time, params) }
}

pub fn strap_TrueMalloc(ptr: *mut *mut c_void, size: c_int) {
    unsafe {
        trap_TrueMalloc(ptr, size);
    }
}

pub fn strap_TrueFree(ptr: *mut *mut c_void) {
    unsafe {
        trap_TrueFree(ptr);
    }
}
