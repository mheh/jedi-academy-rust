//! Extra Ghoul2 syscall wrappers not in `mod.rs` — `trap_G2_*` / `trap_G2API_*`
//! (list/size/dup/attach/rag/IK extras). 1:1 with
//! `refs/raven-jediacademy/codemp/game/g_syscalls.c`; bodies land in Phase B.
//! Types: q_shared_h::sharedRagDollParams_t (and existing mdxaBone_t/CollisionRecord_t etc).

use core::ffi::c_void;

use super::cstr;
use crate::codemp::game::q_shared_h::{mdxaBone_t, qhandle_t, sharedRagDollParams_t, vec3_t};
use crate::ffi::syscalls::pass_float;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport::*;

// CG Specific API calls

/// `trap_G2_ListModelBones` — dump the bone list of the ghoul2 instance to the console.
pub fn G2_ListModelBones(ghl_info: *mut c_void, frame: i32) {
    unsafe {
        syscall!(G_G2_LISTBONES, ghl_info, frame);
    }
}

/// `trap_G2_ListModelSurfaces` — dump the surface list of the ghoul2 instance to the console.
pub fn G2_ListModelSurfaces(ghl_info: *mut c_void) {
    unsafe {
        syscall!(G_G2_LISTSURFACES, ghl_info);
    }
}

/// `trap_G2_SetGhoul2ModelIndexes` — set the model/skin handle lists on the ghoul2 instance.
pub fn G2_SetGhoul2ModelIndexes(
    ghoul2: *mut c_void,
    model_list: *mut qhandle_t,
    skin_list: *mut qhandle_t,
) {
    unsafe {
        syscall!(G_G2_SETMODELS, ghoul2, model_list, skin_list);
    }
}

/// `trap_G2API_Ghoul2Size` — number of ghoul2 models in the instance.
pub fn G2API_Ghoul2Size(ghl_info: *mut c_void) -> i32 {
    unsafe { syscall!(G_G2_SIZE, ghl_info) as i32 }
}

/// `trap_G2API_GetGLAName` — copy the instance's GLA (animation) file name into `fill_buf`.
pub fn G2API_GetGLAName(ghoul2: *mut c_void, model_index: i32, fill_buf: *mut core::ffi::c_char) {
    unsafe {
        syscall!(G_G2_GETGLANAME, ghoul2, model_index, fill_buf);
    }
}

/// `trap_G2API_CopyGhoul2Instance` — copy `model_index` from one ghoul2 instance to another.
pub fn G2API_CopyGhoul2Instance(g2_from: *mut c_void, g2_to: *mut c_void, model_index: i32) -> i32 {
    unsafe { syscall!(G_G2_COPYGHOUL2INSTANCE, g2_from, g2_to, model_index) as i32 }
}

/// `trap_G2API_DuplicateGhoul2Instance` — deep-copy a ghoul2 instance into `*g2_to`.
pub fn G2API_DuplicateGhoul2Instance(g2_from: *mut c_void, g2_to: *mut *mut c_void) {
    unsafe {
        syscall!(G_G2_DUPLICATEGHOUL2INSTANCE, g2_from, g2_to);
    }
}

/// `trap_G2API_HasGhoul2ModelOnIndex` — does the instance have a model at `model_index`?
pub fn G2API_HasGhoul2ModelOnIndex(ghl_info: *mut c_void, model_index: i32) -> qboolean {
    unsafe { syscall!(G_G2_HASGHOUL2MODELONINDEX, ghl_info, model_index) as qboolean }
}

/// `trap_G2API_RemoveGhoul2Model` — remove the model at `model_index` from the instance.
pub fn G2API_RemoveGhoul2Model(ghl_info: *mut c_void, model_index: i32) -> qboolean {
    unsafe { syscall!(G_G2_REMOVEGHOUL2MODEL, ghl_info, model_index) as qboolean }
}

/// `trap_G2API_SetRootSurface` — set `surface_name` as the model's root surface.
pub fn G2API_SetRootSurface(ghoul2: *mut c_void, model_index: i32, surface_name: &str) -> qboolean {
    let surface = cstr(surface_name);
    unsafe { syscall!(G_G2_SETROOTSURFACE, ghoul2, model_index, surface.as_ptr()) as qboolean }
}

/// `trap_G2API_SetNewOrigin` — re-origin the instance to the bolt at `bolt_index`.
pub fn G2API_SetNewOrigin(ghoul2: *mut c_void, bolt_index: i32) -> qboolean {
    unsafe { syscall!(G_G2_SETNEWORIGIN, ghoul2, bolt_index) as qboolean }
}

/// `trap_G2API_GetBoltMatrix_NoReconstruct` — like `G2API_GetBoltMatrix` but force it to not
/// reconstruct the skeleton before getting the bolt position. `model_list` may be null.
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoltMatrix_NoReconstruct(
    ghoul2: *mut c_void,
    model_index: i32,
    bolt_index: i32,
    matrix: &mut mdxaBone_t,
    angles: &vec3_t,
    position: &vec3_t,
    frame_num: i32,
    model_list: *mut qhandle_t,
    scale: &vec3_t,
) -> qboolean {
    unsafe {
        syscall!(
            G_G2_GETBOLT_NOREC,
            ghoul2,
            model_index,
            bolt_index,
            matrix as *mut mdxaBone_t,
            angles.as_ptr(),
            position.as_ptr(),
            frame_num,
            model_list,
            scale.as_ptr()
        ) as qboolean
    }
}

/// `trap_G2API_DoesBoneExist` — check if a bone exists on the skeleton without actually adding
/// it to the bone list. -rww
pub fn G2API_DoesBoneExist(ghoul2: *mut c_void, model_index: i32, bone_name: &str) -> qboolean {
    let bone = cstr(bone_name);
    unsafe { syscall!(G_G2_DOESBONEEXIST, ghoul2, model_index, bone.as_ptr()) as qboolean }
}

/// `trap_G2API_AbsurdSmoothing` — hack for smoothing during ugly situations. forgive me.
pub fn G2API_AbsurdSmoothing(ghoul2: *mut c_void, status: qboolean) {
    unsafe {
        syscall!(G_G2_ABSURDSMOOTHING, ghoul2, status);
    }
}

//rww - RAGDOLL_BEGIN

/// `trap_G2API_SetRagDoll` — kick the instance into ragdoll using `params`.
pub fn G2API_SetRagDoll(ghoul2: *mut c_void, params: &mut sharedRagDollParams_t) {
    unsafe {
        syscall!(
            G_G2_SETRAGDOLL,
            ghoul2,
            params as *mut sharedRagDollParams_t
        );
    }
}
//rww - RAGDOLL_END

//additional ragdoll options -rww

/// `trap_G2API_RagPCJConstraint` — override default pcj bone constraints.
pub fn G2API_RagPCJConstraint(
    ghoul2: *mut c_void,
    bone_name: &str,
    min: &vec3_t,
    max: &vec3_t,
) -> qboolean {
    let bone = cstr(bone_name);
    unsafe {
        syscall!(
            G_G2_RAGPCJCONSTRAINT,
            ghoul2,
            bone.as_ptr(),
            min.as_ptr(),
            max.as_ptr()
        ) as qboolean
    }
}

/// `trap_G2API_RagPCJGradientSpeed` — override the default gradient movespeed for a pcj bone.
pub fn G2API_RagPCJGradientSpeed(ghoul2: *mut c_void, bone_name: &str, speed: f32) -> qboolean {
    let bone = cstr(bone_name);
    unsafe {
        syscall!(
            G_G2_RAGPCJGRADIENTSPEED,
            ghoul2,
            bone.as_ptr(),
            pass_float(speed)
        ) as qboolean
    }
}

/// `trap_G2API_RagEffectorGoal` — override an effector bone's goal position (world coordinates).
pub fn G2API_RagEffectorGoal(ghoul2: *mut c_void, bone_name: &str, pos: &vec3_t) -> qboolean {
    let bone = cstr(bone_name);
    unsafe { syscall!(G_G2_RAGEFFECTORGOAL, ghoul2, bone.as_ptr(), pos.as_ptr()) as qboolean }
}

/// `trap_G2API_GetRagBonePos` — current position of said bone is put into `pos` (world coordinates).
pub fn G2API_GetRagBonePos(
    ghoul2: *mut c_void,
    bone_name: &str,
    pos: &vec3_t,
    ent_angles: &vec3_t,
    ent_pos: &vec3_t,
    ent_scale: &vec3_t,
) -> qboolean {
    let bone = cstr(bone_name);
    unsafe {
        syscall!(
            G_G2_GETRAGBONEPOS,
            ghoul2,
            bone.as_ptr(),
            pos.as_ptr(),
            ent_angles.as_ptr(),
            ent_pos.as_ptr(),
            ent_scale.as_ptr()
        ) as qboolean
    }
}

/// `trap_G2API_RagEffectorKick` — add velocity to a rag bone.
pub fn G2API_RagEffectorKick(ghoul2: *mut c_void, bone_name: &str, velocity: &vec3_t) -> qboolean {
    let bone = cstr(bone_name);
    unsafe {
        syscall!(
            G_G2_RAGEFFECTORKICK,
            ghoul2,
            bone.as_ptr(),
            velocity.as_ptr()
        ) as qboolean
    }
}

/// `trap_G2API_RagForceSolve` — make sure we are actively performing solve/settle routines, if desired.
pub fn G2API_RagForceSolve(ghoul2: *mut c_void, force: qboolean) -> qboolean {
    unsafe { syscall!(G_G2_RAGFORCESOLVE, ghoul2, force) as qboolean }
}

//rww - Stuff to allow association of ghoul2 instances to entity numbers.
//This way, on listen servers when both the client and server are doing
//ghoul2 operations, we can copy relevant data off the client instance
//directly onto the server instance and slash the transforms and whatnot
//right in half.

/// `trap_G2API_AttachInstanceToEntNum` — associate the ghoul2 instance with `entity_num`.
pub fn G2API_AttachInstanceToEntNum(ghoul2: *mut c_void, entity_num: i32, server: qboolean) {
    unsafe {
        syscall!(G_G2_ATTACHINSTANCETOENTNUM, ghoul2, entity_num, server);
    }
}

/// `trap_G2API_ClearAttachedInstance` — drop the ghoul2 instance association for `entity_num`.
pub fn G2API_ClearAttachedInstance(entity_num: i32) {
    unsafe {
        syscall!(G_G2_CLEARATTACHEDINSTANCE, entity_num);
    }
}
