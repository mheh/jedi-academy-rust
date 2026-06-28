//! Port of `refs/raven-jediacademy/codemp/ghoul2/g2.h` — the ghoul2 bone-flag and
//! bolt-packing constants shared between the game module and the engine. Pure
//! `#define`s in the original, carried here as `pub const`s.

use core::ffi::c_int;

pub const BONE_ANGLES_PREMULT: c_int = 0x0001;
pub const BONE_ANGLES_POSTMULT: c_int = 0x0002;
pub const BONE_ANGLES_REPLACE: c_int = 0x0004;

//rww - RAGDOLL_BEGIN
pub const BONE_ANGLES_RAGDOLL: c_int = 0x2000; // the rag flags give more details
                                               //rww - RAGDOLL_END
pub const BONE_ANGLES_IK: c_int = 0x4000; // the rag flags give more details

pub const BONE_ANGLES_TOTAL: c_int =
    BONE_ANGLES_PREMULT | BONE_ANGLES_POSTMULT | BONE_ANGLES_REPLACE;
pub const BONE_ANIM_OVERRIDE: c_int = 0x0008;
pub const BONE_ANIM_OVERRIDE_LOOP: c_int = 0x0010;
pub const BONE_ANIM_OVERRIDE_FREEZE: c_int = 0x0040 + BONE_ANIM_OVERRIDE;
pub const BONE_ANIM_BLEND: c_int = 0x0080;
pub const BONE_ANIM_TOTAL: c_int =
    BONE_ANIM_OVERRIDE | BONE_ANIM_OVERRIDE_LOOP | BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND;

// defines to setup the
pub const ENTITY_WIDTH: c_int = 12;
pub const MODEL_WIDTH: c_int = 10;
pub const BOLT_WIDTH: c_int = 10;

pub const MODEL_AND: c_int = (1 << MODEL_WIDTH) - 1;
pub const BOLT_AND: c_int = (1 << BOLT_WIDTH) - 1;
pub const ENTITY_AND: c_int = (1 << ENTITY_WIDTH) - 1;

pub const BOLT_SHIFT: c_int = 0;
pub const MODEL_SHIFT: c_int = BOLT_SHIFT + BOLT_WIDTH;
pub const ENTITY_SHIFT: c_int = MODEL_SHIFT + MODEL_WIDTH;
