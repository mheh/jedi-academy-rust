//! Group-AI types from `ai.h` needed by `g_local.h`'s `level_locals_t`.
//!
//! `level_locals_t` embeds `AIGroupInfo_t groups[MAX_FRAME_GROUPS]` by value, so
//! the `g_local.h` unit needs `AIGroupInfo_t` (and its `AIGroupMember_t` rows + the
//! squad-state count that sizes its `numState` array). `AIGroupInfo_t` carries
//! `gentity_t*` pointers, so its layout is **arch-dependent** (64-bit asserts gated
//! + host-64-bit oracle, like the g_local.h masters). Ported incrementally — the
//! rest of ai.h (the NPC behavior-state machine, ranks, etc.) lands with its own
//! port. Mirrors upstream `codemp/game/ai.h`.

#![allow(non_camel_case_types)]

use crate::codemp::game::bg_public::team_t;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::q_shared_h::{qboolean, vec3_t};
use core::ffi::c_int;

/// `MAX_FRAME_GROUPS` (ai.h) — sizes `level_locals_t::groups`.
pub const MAX_FRAME_GROUPS: usize = 32;

// Squad states (anonymous enum in ai.h, so plain `c_int` consts). `NUM_SQUAD_STATES`
// sizes `AIGroupInfo_t::numState`.
pub const SQUAD_IDLE: c_int = 0; // No target found, waiting
pub const SQUAD_STAND_AND_SHOOT: c_int = 1; // Standing in position and shoot (no cover)
pub const SQUAD_RETREAT: c_int = 2; // Running away from combat
pub const SQUAD_COVER: c_int = 3; // Under protective cover
pub const SQUAD_TRANSITION: c_int = 4; // Moving between points, not firing
pub const SQUAD_POINT: c_int = 5; // On point, laying down suppressive fire
pub const SQUAD_SCOUT: c_int = 6; // Poking out to draw enemy
pub const NUM_SQUAD_STATES: c_int = 7;

// Distance ratings (`distance_e`, ai.h). Named typedef enum, but kept as plain
// `c_int` consts to match the squad-state idiom above.
pub const DIST_MELEE: c_int = 0;
pub const DIST_LONG: c_int = 1;

/// `MAX_GROUP_MEMBERS` (ai.h) — sizes `AIGroupInfo_t::member`.
pub const MAX_GROUP_MEMBERS: usize = 32;

/// `AIGroupMember_t` (ai.h). "!!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!"
/// Pointer-free; identical on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AIGroupMember_t {
    pub number: c_int,
    pub waypoint: c_int,
    pub pathCostToEnemy: c_int,
    pub closestBuddy: c_int,
}
const _: () = assert!(core::mem::size_of::<AIGroupMember_t>() == 16);

/// `AIGroupInfo_t` (ai.h). "!!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!"
/// Carries `gentity_t*` pointers => arch-dependent (64-bit layout asserted).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AIGroupInfo_t {
    pub numGroup: c_int,
    pub processed: qboolean,
    pub team: team_t,
    pub enemy: *mut gentity_t,
    pub enemyWP: c_int,
    pub speechDebounceTime: c_int,
    pub lastClearShotTime: c_int,
    pub lastSeenEnemyTime: c_int,
    pub morale: c_int,
    pub moraleAdjust: c_int,
    pub moraleDebounce: c_int,
    pub memberValidateTime: c_int,
    pub activeMemberNum: c_int,
    pub commander: *mut gentity_t,
    pub enemyLastSeenPos: vec3_t,
    pub numState: [c_int; NUM_SQUAD_STATES as usize],
    pub member: [AIGroupMember_t; MAX_GROUP_MEMBERS],
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<AIGroupInfo_t>() == 624);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::align_of::<AIGroupInfo_t>() == 8);
const _: () = assert!(core::mem::offset_of!(AIGroupInfo_t, team) == 8); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(AIGroupInfo_t, enemy) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(AIGroupInfo_t, commander) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(AIGroupInfo_t, member) == 112);

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;
    use core::mem::{offset_of, size_of};

    /// Parity: `NUM_SQUAD_STATES` (sizes `AIGroupInfo_t::numState`) and the struct
    /// layouts match the authentic C. `AIGroupMember_t` is pointer-free; the
    /// pointer-bearing `AIGroupInfo_t` is validated at the host 64-bit layout.
    #[test]
    fn ai_layout_matches_c() {
        unsafe {
            assert_eq!(NUM_SQUAD_STATES, jka_ai_NUM_SQUAD_STATES());
            assert_eq!(size_of::<AIGroupMember_t>(), jka_ai_sizeof_AIGroupMember_t());

            #[cfg(target_pointer_width = "64")]
            {
                assert_eq!(size_of::<AIGroupInfo_t>(), jka_ai_sizeof_AIGroupInfo_t());
                assert_eq!(offset_of!(AIGroupInfo_t, enemy), jka_ai_off_enemy());
                assert_eq!(offset_of!(AIGroupInfo_t, commander), jka_ai_off_commander());
                assert_eq!(offset_of!(AIGroupInfo_t, numState), jka_ai_off_numState());
                assert_eq!(offset_of!(AIGroupInfo_t, member), jka_ai_off_member());
            }
        }
    }
}
