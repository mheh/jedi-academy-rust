//! `bg_local.h` — local definitions for the bg (both games) files.
//!
//! Per the C header comment: "local definitions for the bg (both games) files".
//! Holds the pmove movement tunables (`#define`s), the `pml_t` per-move scratch
//! struct ("all of the locals will be zeroed before each pmove, just to make damn
//! sure we don't have any differences when running on client or server"), and the
//! `extern` declarations of the movement-parameter globals + the `PM_*`/`BG_*`
//! function prototypes.
//!
//! The movement-parameter globals (`pm_stopspeed`, `pm_accelerate`, …, `c_pmove`,
//! `forcePowerNeeded[][]`) and the `pml` global are *defined* in `bg_pmove.c`, and
//! the `PM_*`/`BG_*` functions are *defined* across `bg_pmove.c`/`bg_panimate.c`/
//! `bg_slidemove.c`/`bg_saber.c`; so only their **declarations** live here. They
//! are carried below as not-yet-ported source-order comments and will be wired in as
//! those `.c` files land.
//!
//! `pml_t` is pointer-free (vec3_t arrays / floats / ints / an embedded pointer-free
//! `trace_t`), so its layout is identical on 32- and 64-bit; oracle-verified.

#![allow(non_camel_case_types, non_snake_case)]

use crate::codemp::game::q_shared_h::{qboolean, trace_t, vec3_t};
use core::ffi::c_int;

/// `MIN_WALK_NORMAL` (bg_local.h) — can't walk on very steep slopes.
pub const MIN_WALK_NORMAL: f32 = 0.7;

pub const TIMER_LAND: c_int = 130;
pub const TIMER_GESTURE: c_int = 34 * 66 + 50;

pub const OVERCLIP: f32 = 1.001;

/// `pml_t` (bg_local.h) — the pmove local scratch state.
///
/// "all of the locals will be zeroed before each pmove, just to make damn sure we
/// don't have any differences when running on client or server."
///
/// Pointer-free (embeds a pointer-free `trace_t` by value); identical layout on
/// 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct pml_t {
    pub forward: vec3_t,
    pub right: vec3_t,
    pub up: vec3_t,
    pub frametime: f32,

    pub msec: c_int,

    pub walking: qboolean,
    pub groundPlane: qboolean,
    pub groundTrace: trace_t,

    pub impactSpeed: f32,

    pub previous_origin: vec3_t,
    pub previous_velocity: vec3_t,
    pub previous_waterlevel: c_int,
}

const _: () = assert!(core::mem::size_of::<pml_t>() == 132);
const _: () = assert!(core::mem::align_of::<pml_t>() == 4);
const _: () = assert!(core::mem::offset_of!(pml_t, frametime) == 36);
const _: () = assert!(core::mem::offset_of!(pml_t, groundTrace) == 52);
const _: () = assert!(core::mem::offset_of!(pml_t, impactSpeed) == 100);
const _: () = assert!(core::mem::offset_of!(pml_t, previous_waterlevel) == 128);

// The following are bg_pmove.c's file-scope globals, declared `extern` in bg_local.h.
// `pml` and the `pm_*`/`c_pmove`/`forcePowerNeeded` data are NOW PORTED in
// bg_pmove.rs (the data-layer slice); `pml` itself stays unported there (it lands
// with the movement logic). Carried as source-order comments for reference:
//
// extern pml_t pml;
//
// movement parameters
// extern float pm_stopspeed;
// extern float pm_duckScale;
// extern float pm_swimScale;
// extern float pm_wadeScale;
//
// extern float pm_accelerate;
// extern float pm_airaccelerate;
// extern float pm_wateraccelerate;
// extern float pm_flyaccelerate;
//
// extern float pm_friction;
// extern float pm_waterfriction;
// extern float pm_flightfriction;
//
// extern int   c_pmove;
//
// extern int forcePowerNeeded[NUM_FORCE_POWER_LEVELS][NUM_FORCE_POWERS];
// -- defined in bg_pmove.c -> bg_pmove.rs.

// Had to add these here because there was no file access within the BG right now.
// int  trap_FS_FOpenFile( const char *qpath, fileHandle_t *f, fsMode_t mode );
// void trap_FS_Read( void *buffer, int len, fileHandle_t f );
// void trap_FS_Write( const void *buffer, int len, fileHandle_t f );
// void trap_FS_FCloseFile( fileHandle_t f );
// -- the trap_FS_* wrappers live in src/trap/mod.rs.

// PM anim utility functions (defined in bg_pmove.c / bg_panimate.c / bg_saber.c /
// bg_slidemove.c -- declared here, ported when those .c files land):
// qboolean PM_SaberInParry( int move );
// qboolean PM_SaberInKnockaway( int move );
// qboolean PM_SaberInReflect( int move );
// qboolean PM_SaberInStart( int move );
// qboolean PM_InSaberAnim( int anim );
// qboolean PM_InKnockDown( playerState_t *ps );
// qboolean PM_PainAnim( int anim );
// qboolean PM_JumpingAnim( int anim );
// qboolean PM_LandingAnim( int anim );
// qboolean PM_SpinningAnim( int anim );
// qboolean PM_InOnGroundAnim ( int anim );
// qboolean PM_InRollComplete( playerState_t *ps, int anim );
//
// int PM_AnimLength( int index, animNumber_t anim );
//
// int PM_GetSaberStance(void);
// float PM_GroundDistance(void);
// qboolean PM_SomeoneInFront(trace_t *tr);
// saberMoveName_t PM_SaberFlipOverAttackMove(void);
// saberMoveName_t PM_SaberJumpAttackMove( void );
//
// void PM_ClipVelocity( vec3_t in, vec3_t normal, vec3_t out, float overbounce );
// void PM_AddTouchEnt( int entityNum );
// void PM_AddEvent( int newEvent );
//
// qboolean PM_SlideMove( qboolean gravity );
// void     PM_StepSlideMove( qboolean gravity );
//
// void PM_StartTorsoAnim( int anim );
// void PM_ContinueLegsAnim( int anim );
// void PM_ForceLegsAnim( int anim );
//
// void PM_BeginWeaponChange( int weapon );
// void PM_FinishWeaponChange( void );
//
// void PM_SetAnim(int setAnimParts,int anim,int setAnimFlags, int blendTime);
//
// void PM_WeaponLightsaber(void);
// void PM_SetSaberMove(short newMove);
//
// void PM_SetForceJumpZStart(float value);
//
// void BG_CycleInven(playerState_t *ps, int direction);

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;
    use core::mem::{align_of, size_of};

    /// Parity: the `pml_t` `sizeof`/`alignof` and every field `offsetof` match the
    /// authentic C (transcribed verbatim in the oracle TU). Pointer-free =>
    /// arch-independent.
    #[test]
    fn bg_local_pml_t_layout_matches_c() {
        unsafe {
            assert_eq!(size_of::<pml_t>(), jka_bl_sizeof_pml_t());
            assert_eq!(align_of::<pml_t>(), jka_bl_alignof_pml_t());
            assert_eq!(core::mem::offset_of!(pml_t, forward), jka_bl_off_forward());
            assert_eq!(
                core::mem::offset_of!(pml_t, frametime),
                jka_bl_off_frametime()
            );
            assert_eq!(core::mem::offset_of!(pml_t, msec), jka_bl_off_msec());
            assert_eq!(core::mem::offset_of!(pml_t, walking), jka_bl_off_walking());
            assert_eq!(
                core::mem::offset_of!(pml_t, groundPlane),
                jka_bl_off_groundPlane()
            );
            assert_eq!(
                core::mem::offset_of!(pml_t, groundTrace),
                jka_bl_off_groundTrace()
            );
            assert_eq!(
                core::mem::offset_of!(pml_t, impactSpeed),
                jka_bl_off_impactSpeed()
            );
            assert_eq!(
                core::mem::offset_of!(pml_t, previous_origin),
                jka_bl_off_previous_origin()
            );
            assert_eq!(
                core::mem::offset_of!(pml_t, previous_velocity),
                jka_bl_off_previous_velocity()
            );
            assert_eq!(
                core::mem::offset_of!(pml_t, previous_waterlevel),
                jka_bl_off_previous_waterlevel()
            );
        }
    }

    /// Parity: the `#define` movement tunables match the authentic C header.
    #[test]
    fn bg_local_consts_match_c() {
        unsafe {
            assert_eq!(MIN_WALK_NORMAL, jka_bl_MIN_WALK_NORMAL());
            assert_eq!(TIMER_LAND, jka_bl_TIMER_LAND());
            assert_eq!(TIMER_GESTURE, jka_bl_TIMER_GESTURE());
            assert_eq!(OVERCLIP, jka_bl_OVERCLIP());
        }
    }
}
