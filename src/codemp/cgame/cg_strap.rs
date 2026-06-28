//rww - shared trap call system
use core::ffi::{c_int, c_char, c_void};
use super::cg_syscalls::{
    trap_G2API_GetBoltMatrix, trap_G2API_GetBoltMatrix_NoReconstruct, trap_G2API_GetBoltMatrix_NoRecNoRot,
    trap_G2API_SetBoneAngles, trap_G2API_SetBoneAnim, trap_G2API_GetBoneAnim,
    trap_G2API_SetRagDoll, trap_G2API_AnimateG2Models,
    trap_G2API_SetBoneIKState, trap_G2API_IKMove,
    trap_TrueMalloc, trap_TrueFree,
};

pub fn strap_G2API_GetBoltMatrix(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
                                  const_angles: *const [f32; 3], const_position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int
{
    trap_G2API_GetBoltMatrix(ghoul2, modelIndex, boltIndex, matrix, const_angles, const_position, frameNum, modelList, scale)
}

pub fn strap_G2API_GetBoltMatrix_NoReconstruct(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
                                  const_angles: *const [f32; 3], const_position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int
{
    trap_G2API_GetBoltMatrix_NoReconstruct(ghoul2, modelIndex, boltIndex, matrix, const_angles, const_position, frameNum, modelList, scale)
}

pub fn strap_G2API_GetBoltMatrix_NoRecNoRot(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut c_void,
                                  const_angles: *const [f32; 3], const_position: *const [f32; 3], frameNum: c_int, modelList: *mut c_int, scale: *const [f32; 3]) -> c_int
{
    trap_G2API_GetBoltMatrix_NoRecNoRot(ghoul2, modelIndex, boltIndex, matrix, const_angles, const_position, frameNum, modelList, scale)
}

pub fn strap_G2API_SetBoneAngles(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, const_angles: *const [f32; 3], flags: c_int,
                                  const_up: c_int, const_right: c_int, const_forward: c_int, modelList: *mut c_int,
                                  blendTime: c_int, currentTime: c_int) -> c_int
{
    trap_G2API_SetBoneAngles(ghoul2, modelIndex, boneName, const_angles, flags, const_up, const_right, const_forward, modelList, blendTime, currentTime)
}

pub fn strap_G2API_SetBoneAnim(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, startFrame: c_int, endFrame: c_int,
                              flags: c_int, animSpeed: f32, currentTime: c_int, setFrame: f32, blendTime: c_int) -> c_int
{
    trap_G2API_SetBoneAnim(ghoul2, modelIndex, boneName, startFrame, endFrame, flags, animSpeed, currentTime, setFrame, blendTime)
}

pub fn strap_G2API_GetBoneAnim(ghoul2: *mut c_void, boneName: *const c_char, currentTime: c_int, currentFrame: *mut f32,
                           startFrame: *mut c_int, endFrame: *mut c_int, flags: *mut c_int, animSpeed: *mut f32, modelList: *mut c_int, modelIndex: c_int) -> c_int
{
    trap_G2API_GetBoneAnim(ghoul2, boneName, currentTime, currentFrame, startFrame, endFrame, flags, animSpeed, modelList, modelIndex)
}

pub fn strap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut c_void)
{
    trap_G2API_SetRagDoll(ghoul2, params);
}

pub fn strap_G2API_AnimateG2Models(ghoul2: *mut c_void, time: c_int, params: *mut c_void)
{
    trap_G2API_AnimateG2Models(ghoul2, time, params);
}

pub fn strap_G2API_SetBoneIKState(ghoul2: *mut c_void, time: c_int, boneName: *const c_char, ikState: c_int, params: *mut c_void) -> c_int
{
    trap_G2API_SetBoneIKState(ghoul2, time, boneName, ikState, params)
}

pub fn strap_G2API_IKMove(ghoul2: *mut c_void, time: c_int, params: *mut c_void) -> c_int
{
    trap_G2API_IKMove(ghoul2, time, params)
}

pub fn strap_TrueMalloc(ptr: *mut *mut c_void, size: c_int)
{
    trap_TrueMalloc(ptr, size);
}

pub fn strap_TrueFree(ptr: *mut *mut c_void)
{
    trap_TrueFree(ptr);
}
