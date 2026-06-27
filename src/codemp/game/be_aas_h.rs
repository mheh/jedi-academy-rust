//! Area Awareness System types exported to the AI, from `be_aas.h`.
//!
//! These structs are written by the engine-side AAS routines and handed back to the
//! game module through the `trap_AAS_*` syscalls (out-params crossing the VM ABI by
//! pointer), so their layout is load-bearing. Faithful 1:1 with the original-JKA
//! `refs/raven-jediacademy/codemp/game/be_aas.h`. (`bsp_trace_t`/`bsp_surface_t` are
//! `#if 0`-commented in the header and unreferenced by any trap, so they are omitted.)

#![allow(non_camel_case_types)]

use super::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;
use core::ffi::c_int;

/// `solid_t` (be_aas.h) — solidity class for AAS world interaction.
pub type solid_t = c_int;
/// no interaction with other objects
pub const SOLID_NOT: solid_t = 0;
/// only touch when inside, after moving
pub const SOLID_TRIGGER: solid_t = 1;
/// touch on edge
pub const SOLID_BBOX: solid_t = 2;
/// bsp clip, touch on edge
pub const SOLID_BSP: solid_t = 3;

/// `aas_trace_t` (be_aas.h) — returned when a box is swept through the AAS world.
/// Embedded by value in [`aas_clientmove_t`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct aas_trace_t {
    /// if true, the initial point was in a solid area
    pub startsolid: qboolean,
    /// time completed, 1.0 = didn't hit anything
    pub fraction: f32,
    /// final position
    pub endpos: vec3_t,
    /// entity blocking the trace
    pub ent: c_int,
    /// last area the trace was in (zero if none)
    pub lastarea: c_int,
    /// area blocking the trace (zero if none)
    pub area: c_int,
    /// number of the plane that was hit
    pub planenum: c_int,
}
// qboolean (4) + float (4) + vec3_t (12) + 4×int (16) = 36, align 4.
const _: () = assert!(core::mem::size_of::<aas_trace_t>() == 36);
const _: () = assert!(core::mem::align_of::<aas_trace_t>() == 4);

/// `aas_entityinfo_t` (be_aas.h) — per-entity snapshot filled by `trap_AAS_EntityInfo`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct aas_entityinfo_t {
    /// true if updated this frame
    pub valid: c_int,
    /// entity type
    pub r#type: c_int,
    /// entity flags
    pub flags: c_int,
    /// local time
    pub ltime: f32,
    /// time between last and current update
    pub update_time: f32,
    /// number of the entity
    pub number: c_int,
    /// origin of the entity
    pub origin: vec3_t,
    /// angles of the model
    pub angles: vec3_t,
    /// for lerping
    pub old_origin: vec3_t,
    /// last visible origin
    pub lastvisorigin: vec3_t,
    /// bounding box minimums
    pub mins: vec3_t,
    /// bounding box maximums
    pub maxs: vec3_t,
    /// ground entity
    pub groundent: c_int,
    /// solid type
    pub solid: c_int,
    /// model used
    pub modelindex: c_int,
    /// weapons, CTF flags, etc
    pub modelindex2: c_int,
    /// model frame number
    pub frame: c_int,
    /// impulse events -- muzzle flashes, footsteps, etc
    pub event: c_int,
    /// event parameter
    pub eventParm: c_int,
    /// bit flags
    pub powerups: c_int,
    /// determines weapon and flash model, etc
    pub weapon: c_int,
    /// current legs anim
    pub legsAnim: c_int,
    /// current torso anim
    pub torsoAnim: c_int,
}

/// `aas_areainfo_t` (be_aas.h) — area description filled by `trap_AAS_AreaInfo`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct aas_areainfo_t {
    pub contents: c_int,
    pub flags: c_int,
    pub presencetype: c_int,
    pub cluster: c_int,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub center: vec3_t,
}

/// `aas_clientmove_t` (be_aas.h) — result of `trap_AAS_PredictClientMovement`.
/// Embeds [`aas_trace_t`] by value.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct aas_clientmove_t {
    /// position at the end of movement prediction
    pub endpos: vec3_t,
    /// area at end of movement prediction
    pub endarea: c_int,
    /// velocity at the end of movement prediction
    pub velocity: vec3_t,
    /// last trace
    pub trace: aas_trace_t,
    /// presence type at end of movement prediction
    pub presencetype: c_int,
    /// event that made the prediction stop
    pub stopevent: c_int,
    /// contents at the end of movement prediction
    pub endcontents: c_int,
    /// time predicted ahead
    pub time: f32,
    /// number of frames predicted ahead
    pub frames: c_int,
}

/// `aas_altroutegoal_t` (be_aas.h) — one alternate-route goal returned by
/// `trap_AAS_AlternativeRouteGoals` (filled into a caller array).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct aas_altroutegoal_t {
    pub origin: vec3_t,
    pub areanum: c_int,
    pub starttraveltime: u16,
    pub goaltraveltime: u16,
    pub extratraveltime: u16,
}

/// `aas_predictroute_t` (be_aas.h) — result of `trap_AAS_PredictRoute`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct aas_predictroute_t {
    /// position at the end of movement prediction
    pub endpos: vec3_t,
    /// area at end of movement prediction
    pub endarea: c_int,
    /// event that made the prediction stop
    pub stopevent: c_int,
    /// contents at the end of movement prediction
    pub endcontents: c_int,
    /// end travel flags
    pub endtravelflags: c_int,
    /// number of areas predicted ahead
    pub numareas: c_int,
    /// time predicted ahead (in hundredth of a sec)
    pub time: c_int,
}
