//! Data types from `g_public.h` — the game↔engine shared header.
//!
//! `g_public.h` defines **two** things: the ABI command/syscall enums (`GAME_*`
//! `vmMain` commands and `G_*` syscall trap numbers) and the shared **data** types
//! the engine and game both read. The ABI enums live in the VM scaffold
//! (`src/ffi/game_export.rs` / `game_import.rs`); **this module is the data half** —
//! `entityShared_t` (embedded by value as `gentity_t::r`), the ICARUS `taskID_t` /
//! `bSet_t` enums that size `gentity_t`'s `taskID`/`behaviorSet` arrays, `parms_t`,
//! and the `Vehicle_t` re-export (g_public.h forward-declares it; the real struct
//! lives in `bg_vehicles_h`). Mirrors upstream `codemp/game/g_public.h`.
//!
//! All structs here are pointer-free, so identical layout on 32- and 64-bit;
//! oracle-verified in `oracle/g_public_h_oracle.c`.

#![allow(non_camel_case_types)]

use crate::codemp::game::q_shared_h::{qboolean, vec3_t};
use core::ffi::{c_char, c_int};

/// `Q3_INFINITE` (g_public.h).
pub const Q3_INFINITE: c_int = 16777216;

/// `GAME_API_VERSION` (g_public.h) — the game-module ABI version handshake value.
pub const GAME_API_VERSION: c_int = 8;

// ---------------------------------------------------------------------------
// entity->svFlags. The server does not know how to interpret most of the values
// in entityStates (level eType), so the game must explicitly flag special
// server behaviors.
// ---------------------------------------------------------------------------

pub const SVF_NOCLIENT: c_int = 0x00000001; // don't send entity to clients, even if it has effects
pub const SVF_BOT: c_int = 0x00000008; // set if the entity is a bot
pub const SVF_PLAYER_USABLE: c_int = 0x00000010; // player can use this with the use button
pub const SVF_BROADCAST: c_int = 0x00000020; // send to all connected clients
pub const SVF_PORTAL: c_int = 0x00000040; // merge a second pvs at origin2 into snapshots
pub const SVF_USE_CURRENT_ORIGIN: c_int = 0x00000080; // entity->r.currentOrigin instead of entity->s.origin
                                                      // for link position (missiles and movers)
pub const SVF_SINGLECLIENT: c_int = 0x00000100; // only send to a single client (entityShared_t->singleClient)
pub const SVF_NOSERVERINFO: c_int = 0x00000200; // don't send CS_SERVERINFO updates to this client
                                                // so that it can be updated for ping tools without
                                                // lagging clients
pub const SVF_CAPSULE: c_int = 0x00000400; // use capsule for collision detection instead of bbox
pub const SVF_NOTSINGLECLIENT: c_int = 0x00000800; // send entity to everyone but one client
                                                   // (entityShared_t->singleClient)
pub const SVF_OWNERNOTSHARED: c_int = 0x00001000; // If it's owned by something and another thing owned by that something
                                                  // hits it, it will still touch
pub const SVF_ICARUS_FREEZE: c_int = 0x00008000; // NPCs are frozen, ents don't execute ICARUS commands
pub const SVF_GLASS_BRUSH: c_int = 0x08000000; // Ent is a glass brush

//rww - ghoul2 trace flags
pub const G2TRFLAG_DOGHOULTRACE: c_int = 0x00000001; //do the ghoul2 trace
pub const G2TRFLAG_HITCORPSES: c_int = 0x00000002; //will try g2 collision on the ent even if it's EF_DEAD
pub const G2TRFLAG_GETSURFINDEX: c_int = 0x00000004; //will replace surfaceFlags with the ghoul2 surface index that was hit, if any.
pub const G2TRFLAG_THICK: c_int = 0x00000008; //assures that the trace radius will be significantly large regardless of the trace box size.

/// `failedEdge_t` (g_public.h) — shared by gameside and in-engine NPC nav routines.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct failedEdge_t {
    pub startID: c_int,
    pub endID: c_int,
    pub checkTime: c_int,
    pub entID: c_int,
}
const _: () = assert!(core::mem::size_of::<failedEdge_t>() == 16);
const _: () = assert!(core::mem::align_of::<failedEdge_t>() == 4);

/// `entityShared_t` (g_public.h) — shared by both the server system and game,
/// embedded by value as `gentity_t::r`. Pointer-free; identical on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct entityShared_t {
    pub linked: qboolean, // qfalse if not in any good cluster
    pub linkcount: c_int,

    pub svFlags: c_int,      // SVF_NOCLIENT, SVF_BROADCAST, etc
    pub singleClient: c_int, // only send to this client when SVF_SINGLECLIENT is set

    pub bmodel: qboolean, // if false, assume an explicit mins / maxs bounding box
    // only set by trap_SetBrushModel
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub contents: c_int, // CONTENTS_TRIGGER, CONTENTS_SOLID, CONTENTS_BODY, etc
    // a non-solid entity should set to 0
    pub absmin: vec3_t, // derived from mins/maxs and origin + rotation
    pub absmax: vec3_t,

    // currentOrigin will be used for all collision detection and world linking.
    // it will not necessarily be the same as the trajectory evaluation for the current
    // time, because each entity must be moved one at a time after time is advanced
    // to avoid simultanious collision issues
    pub currentOrigin: vec3_t,
    pub currentAngles: vec3_t,
    pub mIsRoffing: qboolean, // set to qtrue when the entity is being roffed

    // when a trace call is made and passEntityNum != ENTITYNUM_NONE,
    // an ent will be excluded from testing if:
    // ent->s.number == passEntityNum	(don't interact with self)
    // ent->s.ownerNum = passEntityNum	(don't interact with your own missiles)
    // entity[ent->s.ownerNum].ownerNum = passEntityNum	(don't interact with other missiles from owner)
    pub ownerNum: c_int,

    // mask of clients that this entity should be broadcast too.  The first 32 clients
    // are represented by the first array index and the latter 32 clients are represented
    // by the second array index.
    pub broadcastClients: [c_int; 2],
}
const _: () = assert!(core::mem::size_of::<entityShared_t>() == 112);
const _: () = assert!(core::mem::align_of::<entityShared_t>() == 4);
const _: () = assert!(core::mem::offset_of!(entityShared_t, mins) == 20);
const _: () = assert!(core::mem::offset_of!(entityShared_t, currentOrigin) == 72);
const _: () = assert!(core::mem::offset_of!(entityShared_t, broadcastClients) == 104);

// ---------------------------------------------------------------------------
// ICARUS task/behavior-set enums (g_public.h). `NUM_TIDS`/`NUM_BSETS` size the
// `gentity_t::taskID`/`behaviorSet` arrays, so they must be exact.
// ---------------------------------------------------------------------------

/// `taskID_t` (g_public.h `//# taskID_e`).
pub type taskID_t = c_int;
pub const TID_CHAN_VOICE: taskID_t = 0; // Waiting for a voice sound to complete
pub const TID_ANIM_UPPER: taskID_t = 1; // Waiting to finish a lower anim holdtime
pub const TID_ANIM_LOWER: taskID_t = 2; // Waiting to finish a lower anim holdtime
pub const TID_ANIM_BOTH: taskID_t = 3; // Waiting to finish lower and upper anim holdtimes or normal md3 animating
pub const TID_MOVE_NAV: taskID_t = 4; // Trying to get to a navgoal or For ET_MOVERS
pub const TID_ANGLE_FACE: taskID_t = 5; // Turning to an angle or facing
pub const TID_BSTATE: taskID_t = 6; // Waiting for a certain bState to finish
pub const TID_LOCATION: taskID_t = 7; // Waiting for ent to enter a specific trigger_location
                                      //	TID_MISSIONSTATUS,	// Waiting for player to finish reading MISSION STATUS SCREEN
pub const TID_RESIZE: taskID_t = 8; // Waiting for clear bbox to inflate size
pub const TID_SHOOT: taskID_t = 9; // Waiting for fire event
/// `NUM_TIDS` — for def of taskID array (10).
pub const NUM_TIDS: usize = 10;

/// `bSet_t` (g_public.h `//# bSet_e`). This should check to matching a behavior
/// state name first, then look for a script.
pub type bSet_t = c_int;
pub const BSET_INVALID: bSet_t = -1;
pub const BSET_FIRST: bSet_t = 0;
pub const BSET_SPAWN: bSet_t = 0; //# script to use when first spawned
pub const BSET_USE: bSet_t = 1; //# script to use when used
pub const BSET_AWAKE: bSet_t = 2; //# script to use when awoken/startled
pub const BSET_ANGER: bSet_t = 3; //# script to use when aquire an enemy
pub const BSET_ATTACK: bSet_t = 4; //# script to run when you attack
pub const BSET_VICTORY: bSet_t = 5; //# script to run when you kill someone
pub const BSET_LOSTENEMY: bSet_t = 6; //# script to run when you can't find your enemy
pub const BSET_PAIN: bSet_t = 7; //# script to use when take pain
pub const BSET_FLEE: bSet_t = 8; //# script to use when take pain below 50% of health
pub const BSET_DEATH: bSet_t = 9; //# script to use when killed
pub const BSET_DELAYED: bSet_t = 10; //# script to run when self->delayScriptTime is reached
pub const BSET_BLOCKED: bSet_t = 11; //# script to run when blocked by a friendly NPC or player
pub const BSET_BUMPED: bSet_t = 12; //# script to run when bumped into a friendly NPC or player (can set bumpRadius)
pub const BSET_STUCK: bSet_t = 13; //# script to run when blocked by a wall
pub const BSET_FFIRE: bSet_t = 14; //# script to run when player shoots their own teammates
pub const BSET_FFDEATH: bSet_t = 15; //# script to run when player kills a teammate
pub const BSET_MINDTRICK: bSet_t = 16; //# script to run when player does a mind trick on this NPC
/// `NUM_BSETS` — count of behavior-set scripts (17).
pub const NUM_BSETS: usize = 17;

/// `MAX_PARMS` (g_public.h).
pub const MAX_PARMS: usize = 16;
/// `MAX_PARM_STRING_LENGTH` = `MAX_QPATH` ("was 16, had to lengthen it so they
/// could take a valid file path").
pub const MAX_PARM_STRING_LENGTH: usize = crate::codemp::game::q_shared_h::MAX_QPATH;

/// `parms_t` (g_public.h) — the ICARUS parameter block (`gentity_t::parms` points
/// at one). Pointer-free; identical on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct parms_t {
    pub parm: [[c_char; MAX_PARM_STRING_LENGTH]; MAX_PARMS],
}
const _: () = assert!(core::mem::size_of::<parms_t>() == 1024);
const _: () = assert!(core::mem::align_of::<parms_t>() == 1);

/// `MAX_FAILED_NODES` (g_public.h) — sizes `gentity_t::failedWaypoints`.
pub const MAX_FAILED_NODES: usize = 8;

/// `Vehicle_t` (g_public.h: `typedef struct Vehicle_s Vehicle_t;`) — g_public.h only
/// **forward-declares** it; the full struct lives in `bg_vehicles.h`, now ported as
/// [`bg_vehicles_h::Vehicle_t`]. Re-exported here so the header's
/// "knows-about-`Vehicle_t`" role and the existing import paths
/// (`g_local`'s `gentity_t::m_pVehicle`) stay stable while resolving to the one real
/// type. (Pointer-only at every use site, so this never affected any layout.)
///
/// [`bg_vehicles_h::Vehicle_t`]: crate::codemp::game::bg_vehicles_h::Vehicle_t
pub use crate::codemp::game::bg_vehicles_h::Vehicle_t;

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;
    use core::mem::{offset_of, size_of};

    /// Parity: the g_public.h data structs match the authentic C `sizeof` /
    /// `offsetof`, and the ICARUS enum terminals (`NUM_TIDS`/`NUM_BSETS`, which size
    /// `gentity_t` arrays) match. All pointer-free => arch-independent.
    #[test]
    fn g_public_layout_matches_c() {
        unsafe {
            assert_eq!(size_of::<failedEdge_t>(), jka_gp_sizeof_failedEdge_t());

            assert_eq!(size_of::<entityShared_t>(), jka_gp_sizeof_entityShared_t());
            assert_eq!(offset_of!(entityShared_t, mins), jka_gp_off_es_mins());
            assert_eq!(
                offset_of!(entityShared_t, currentOrigin),
                jka_gp_off_es_currentOrigin()
            );
            assert_eq!(
                offset_of!(entityShared_t, broadcastClients),
                jka_gp_off_es_broadcastClients()
            );

            assert_eq!(size_of::<parms_t>(), jka_gp_sizeof_parms_t());

            assert_eq!(NUM_TIDS as c_int, jka_gp_NUM_TIDS());
            assert_eq!(NUM_BSETS as c_int, jka_gp_NUM_BSETS());
            assert_eq!(BSET_INVALID, jka_gp_BSET_INVALID());
            assert_eq!(BSET_MINDTRICK, jka_gp_BSET_MINDTRICK());
        }
    }
}
