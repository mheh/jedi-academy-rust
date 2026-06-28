//! Slice of `b_public.h` (the NPC/AI public header).
//!
//! Carries the real per-NPC runtime state struct `gNPC_t` (`gentity_t::NPC`
//! points at one, allocated lazily when an entity becomes an NPC) and the value
//! types it embeds — `gNPCstats_t`, plus the `visibility_t`/`spot_t`/
//! `jumpState_t`/`rank_t`/`bState_t` enums and the `usercmd_t`/`vec3_t` it carries
//! by value. Pointer members (`AIGroupInfo_t*`, `gentity_t*`) stay raw pointers to
//! forward types, so `gNPC_t`'s layout is **arch-dependent** (64-bit asserts gated
//! + host-64-bit oracle, like the g_local.h masters). Mirrors upstream
//! `codemp/game/b_public.h`.

#![allow(non_camel_case_types)]

use crate::codemp::game::ai_h::AIGroupInfo_t;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::q_shared_h::{qboolean, usercmd_t, vec3_t};
use core::ffi::c_int;

// NPC AI flags (b_public.h) — stored in `gNPC_t::aiFlags` (a plain `int`).
pub const NPCAI_CHECK_WEAPON: c_int = 0x00000001;
pub const NPCAI_BURST_WEAPON: c_int = 0x00000002;
pub const NPCAI_MOVING: c_int = 0x00000004;
pub const NPCAI_TOUCHED_GOAL: c_int = 0x00000008;
pub const NPCAI_PUSHED: c_int = 0x00000010;
pub const NPCAI_NO_COLL_AVOID: c_int = 0x00000020;
pub const NPCAI_BLOCKED: c_int = 0x00000040;
pub const NPCAI_OFF_PATH: c_int = 0x00000100;
pub const NPCAI_IN_SQUADPOINT: c_int = 0x00000200;
pub const NPCAI_STRAIGHT_TO_DESTPOS: c_int = 0x00000400;
pub const NPCAI_NO_SLOWDOWN: c_int = 0x00001000;
pub const NPCAI_LOST: c_int = 0x00002000; // Can't nav to his goal
pub const NPCAI_SHIELDS: c_int = 0x00004000; // Has shields, borg can adapt
pub const NPCAI_GREET_ALLIES: c_int = 0x00008000; // Say hi to nearby allies
pub const NPCAI_FORM_TELE_NAV: c_int = 0x00010000; // Tells formation people to use nav info to get to
pub const NPCAI_ENROUTE_TO_HOMEWP: c_int = 0x00020000; // Lets us know to run our lostenemyscript when we get to homeWp
pub const NPCAI_MATCHPLAYERWEAPON: c_int = 0x00040000; // Match the player's weapon except when it changes during cinematics
pub const NPCAI_DIE_ON_IMPACT: c_int = 0x00100000; // Next time you crashland, die!
pub const NPCAI_CUSTOM_GRAVITY: c_int = 0x00200000; // Don't use g_gravity, I fly!

// Script flags (b_public.h) — stored in `gNPC_t::scriptFlags` (a plain `int`).
pub const SCF_CROUCHED: c_int = 0x00000001; // Force ucmd.upmove to be -127
pub const SCF_WALKING: c_int = 0x00000002; // Force BUTTON_WALKING to be pressed
pub const SCF_MORELIGHT: c_int = 0x00000004; // NPC will have a minlight of 96
pub const SCF_LEAN_RIGHT: c_int = 0x00000008; // Force rightmove+BUTTON_USE
pub const SCF_LEAN_LEFT: c_int = 0x00000010; // Force leftmove+BUTTON_USE
pub const SCF_RUNNING: c_int = 0x00000020; // Takes off walking button, overrides SCF_WALKING
pub const SCF_ALT_FIRE: c_int = 0x00000040; // Force to use alt-fire when firing
pub const SCF_NO_RESPONSE: c_int = 0x00000080; // NPC will not do generic responses to being used
pub const SCF_FFDEATH: c_int = 0x00000100; // Just tells player_die to run the friendly fire deathscript
pub const SCF_NO_COMBAT_TALK: c_int = 0x00000200; // NPC will not use their generic combat chatter stuff
pub const SCF_CHASE_ENEMIES: c_int = 0x00000400; // NPC chase enemies
pub const SCF_LOOK_FOR_ENEMIES: c_int = 0x00000800; // NPC be on the lookout for enemies
pub const SCF_FACE_MOVE_DIR: c_int = 0x00001000; // NPC face direction it's moving
pub const SCF_IGNORE_ALERTS: c_int = 0x00002000; // NPC ignore alert events
pub const SCF_DONT_FIRE: c_int = 0x00004000; // NPC won't shoot
pub const SCF_DONT_FLEE: c_int = 0x00008000; // NPC never flees
pub const SCF_FORCED_MARCH: c_int = 0x00010000; // NPC that the player must aim at to make him walk
pub const SCF_NO_GROUPS: c_int = 0x00020000; // NPC cannot alert groups or be part of a group
pub const SCF_FIRE_WEAPON: c_int = 0x00040000; // NPC will fire his (her) weapon
pub const SCF_NO_MIND_TRICK: c_int = 0x00080000; // Not succeptible to mind tricks
pub const SCF_USE_CP_NEAREST: c_int = 0x00100000; // Will use combat point close to it
pub const SCF_NO_FORCE: c_int = 0x00200000; // Not succeptible to force powers
pub const SCF_NO_FALLTODEATH: c_int = 0x00400000; // NPC will not scream and tumble and fall to hit death over large drops
pub const SCF_NO_ACROBATICS: c_int = 0x00800000; // Jedi won't jump, roll or cartwheel
pub const SCF_USE_SUBTITLES: c_int = 0x01000000; // This NPC will display subtitles when it speaks lines
pub const SCF_NO_ALERT_TALK: c_int = 0x02000000; // Will not say alert sounds, but still can be woken up by alerts

// Debug alert-info flags (b_public.h).
pub const AID_IDLE: c_int = 0x00000000; // Nothing is happening
pub const AID_ACQUIRED: c_int = 0x00000001; // A target has been found
pub const AID_LOST: c_int = 0x00000002; // Alert, but no target is in sight
pub const AID_CONFUSED: c_int = 0x00000004; // Is unable to come up with a course of action
pub const AID_LOSTPATH: c_int = 0x00000008; // Cannot make a valid movement due to lack of connections

/// `visibility_t` (b_public.h) — embedded by value in `gNPC_t::enemyLastVisibility`.
pub type visibility_t = c_int;
pub const VIS_UNKNOWN: visibility_t = 0;
pub const VIS_NOT: visibility_t = 1;
pub const VIS_PVS: visibility_t = 2;
pub const VIS_360: visibility_t = 3;
pub const VIS_FOV: visibility_t = 4;
pub const VIS_SHOOT: visibility_t = 5;

/// `spot_t` (b_public.h) — NPC aim-spot selector. (Not embedded in `gNPC_t`, but
/// part of the same self-contained header section.)
pub type spot_t = c_int;
pub const SPOT_ORIGIN: spot_t = 0;
pub const SPOT_CHEST: spot_t = 1;
pub const SPOT_HEAD: spot_t = 2;
pub const SPOT_HEAD_LEAN: spot_t = 3;
pub const SPOT_WEAPON: spot_t = 4;
pub const SPOT_LEGS: spot_t = 5;
pub const SPOT_GROUND: spot_t = 6;

/// `lookMode_t` (b_public.h `//# lookMode_e`) — how an NPC chooses its look target.
/// Embedded by value in `renderInfo_t::lookMode`.
pub type lookMode_t = c_int;
pub const LM_ENT: lookMode_t = 0;
pub const LM_INTEREST: lookMode_t = 1;

/// `jumpState_t` (b_public.h `//# jumpState_e`) — embedded by value in
/// `gNPC_t::jumpState`.
pub type jumpState_t = c_int;
pub const JS_WAITING: jumpState_t = 0;
pub const JS_FACING: jumpState_t = 1;
pub const JS_CROUCHING: jumpState_t = 2;
pub const JS_JUMPING: jumpState_t = 3;
pub const JS_LANDING: jumpState_t = 4;

/// `rank_t` (ai.h `//# rank_e`) — for pips; embedded by value in `gNPC_t::rank`.
/// (Lives in ai.h but lands here as a `gNPC_t`-embedded value type.)
pub type rank_t = c_int;
pub const RANK_CIVILIAN: rank_t = 0;
pub const RANK_CREWMAN: rank_t = 1;
pub const RANK_ENSIGN: rank_t = 2;
pub const RANK_LT_JG: rank_t = 3;
pub const RANK_LT: rank_t = 4;
pub const RANK_LT_COMM: rank_t = 5;
pub const RANK_COMMANDER: rank_t = 6;
pub const RANK_CAPTAIN: rank_t = 7;

/// `bState_t` (g_public.h `//# bState_e`) — NPC behavior-state machine; embedded by
/// value in `gNPC_t::behaviorState`/`defaultBehavior`/`tempBehavior`. "These take
/// over only if script allows them to be autonomous." (Lives in g_public.h but
/// lands here as a `gNPC_t`-embedded value type.)
pub type bState_t = c_int;
pub const BS_DEFAULT: bState_t = 0; // default behavior for that NPC
pub const BS_ADVANCE_FIGHT: bState_t = 1; // Advance to captureGoal and shoot enemies if you can
pub const BS_SLEEP: bState_t = 2; // Play awake script when startled by sound
pub const BS_FOLLOW_LEADER: bState_t = 3; // Follow your leader and shoot any enemies you come across
pub const BS_JUMP: bState_t = 4; // Face navgoal and jump to it.
pub const BS_SEARCH: bState_t = 5; // search the immediate branches of waypoints for enemies
pub const BS_WANDER: bState_t = 6; // Wander down random waypoint paths
pub const BS_NOCLIP: bState_t = 7; // Moves through walls, etc.
pub const BS_REMOVE: bState_t = 8; // Waits for player to leave PVS then removes itself
pub const BS_CINEMATIC: bState_t = 9; // Does nothing but face it's angles and move to a goal if it has one
                                      // internal bStates only
pub const BS_WAIT: bState_t = 10; // Does nothing but face it's angles
pub const BS_STAND_GUARD: bState_t = 11;
pub const BS_PATROL: bState_t = 12;
pub const BS_INVESTIGATE: bState_t = 13; // head towards temp goal and look for enemies and listen for sounds
pub const BS_STAND_AND_SHOOT: bState_t = 14;
pub const BS_HUNT_AND_KILL: bState_t = 15;
pub const BS_FLEE: bState_t = 16; // Run away!
pub const NUM_BSTATES: bState_t = 17;

/// `gNPCstats_t` (b_public.h `gNPCstats_e`) — loaded-in NPC stats that scripts can
/// set. Embedded by value in `gNPC_t::stats`. Pointer-free; identical on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct gNPCstats_t {
    // AI
    pub aggression: c_int,
    pub aim: c_int,
    pub earshot: f32,
    pub evasion: c_int,
    pub hfov: c_int, // horizontal field of view
    pub intelligence: c_int,
    pub move_: c_int,       // `move` (Rust keyword)
    pub reactions: c_int,   // 1-5, higher is better
    pub shootDistance: f32, // Maximum range- overrides range set for weapon if nonzero
    pub vfov: c_int,        // vertical field of view
    pub vigilance: f32,
    pub visrange: f32,
    // Movement
    pub runSpeed: c_int,
    pub walkSpeed: c_int,
    pub yawSpeed: f32, // 1 - whatever, default is 50
    pub health: c_int,
    pub acceleration: c_int,
}
const _: () = assert!(core::mem::size_of::<gNPCstats_t>() == 68);

/// `MAX_ENEMY_POS_LAG` (b_public.h).
pub const MAX_ENEMY_POS_LAG: usize = 2400;
/// `ENEMY_POS_LAG_INTERVAL` (b_public.h).
pub const ENEMY_POS_LAG_INTERVAL: usize = 100;
/// `ENEMY_POS_LAG_STEPS` (b_public.h) — sizes `gNPC_t::enemyLaggedPos`.
pub const ENEMY_POS_LAG_STEPS: usize = MAX_ENEMY_POS_LAG / ENEMY_POS_LAG_INTERVAL;

/// `gNPC_t` (b_public.h) — the per-NPC runtime state struct. `gentity_t::NPC`
/// holds only a **pointer** to one (allocated lazily when an entity becomes an
/// NPC). Carries `gentity_t*`/`AIGroupInfo_t*` pointers => layout is
/// arch-dependent (64-bit layout asserted + host-64-bit oracle).
///
/// NOTE!!! If you add any ptr fields into this structure could you please tell me
/// so I can update the load/save code? -slc
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gNPC_t {
    // FIXME: Put in playerInfo or something
    pub timeOfDeath: c_int, // FIXME do we really need both of these
    pub touchedByPlayer: *mut gentity_t,

    pub enemyLastVisibility: visibility_t,

    pub aimTime: c_int,
    pub desiredYaw: f32,
    pub desiredPitch: f32,
    pub lockedDesiredYaw: f32,
    pub lockedDesiredPitch: f32,
    pub aimingBeam: *mut gentity_t, // debugging aid

    pub enemyLastSeenLocation: vec3_t,
    pub enemyLastSeenTime: c_int,
    pub enemyLastHeardLocation: vec3_t,
    pub enemyLastHeardTime: c_int,
    pub lastAlertID: c_int, // unique ID

    pub eFlags: c_int,
    pub aiFlags: c_int,

    pub currentAmmo: c_int, // this sucks, need to find a better way
    pub shotTime: c_int,
    pub burstCount: c_int,
    pub burstMin: c_int,
    pub burstMean: c_int,
    pub burstMax: c_int,
    pub burstSpacing: c_int,
    pub attackHold: c_int,
    pub attackHoldTime: c_int,
    pub shootAngles: vec3_t, // Angles to where bot is shooting

    // extra character info
    pub rank: rank_t, // for pips

    // Behavior state info
    pub behaviorState: bState_t, // determines what actions he should be doing
    pub defaultBehavior: bState_t, // State bot will default to if none other set
    pub tempBehavior: bState_t,  // While valid, overrides other behavior

    pub ignorePain: qboolean, // only play pain scripts when take pain

    pub duckDebounceTime: c_int, // Keeps them ducked for a certain time
    pub walkDebounceTime: c_int,
    pub enemyCheckDebounceTime: c_int,
    pub investigateDebounceTime: c_int,
    pub investigateCount: c_int,
    pub investigateGoal: vec3_t,
    pub investigateSoundDebounceTime: c_int,
    pub greetingDebounceTime: c_int, // when we can greet someone next
    pub eventOwner: *mut gentity_t,

    // bState-specific fields
    pub coverTarg: *mut gentity_t,
    pub jumpState: jumpState_t,
    pub followDist: f32,

    // goal, navigation & pathfinding
    pub tempGoal: *mut gentity_t, // used for locational goals (player's last seen/heard position)
    pub goalEntity: *mut gentity_t,
    pub lastGoalEntity: *mut gentity_t,
    pub eventualGoal: *mut gentity_t,
    pub captureGoal: *mut gentity_t, // Where we should try to capture
    pub defendEnt: *mut gentity_t,   // Who we're trying to protect
    pub greetEnt: *mut gentity_t,    // Who we're greeting
    pub goalTime: c_int,             // FIXME: This is never actually used
    pub straightToGoal: qboolean,    // move straight at navgoals
    pub distToGoal: f32,
    pub navTime: c_int,
    pub blockingEntNum: c_int,
    pub blockedSpeechDebounceTime: c_int,
    pub lastSideStepSide: c_int,
    pub sideStepHoldTime: c_int,
    pub homeWp: c_int,
    pub group: *mut AIGroupInfo_t,

    pub lastPathAngles: vec3_t, // So we know which way to face generally when we stop

    // stats
    pub stats: gNPCstats_t,
    pub aimErrorDebounceTime: c_int,
    pub lastAimErrorYaw: f32,
    pub lastAimErrorPitch: f32,
    pub aimOfs: vec3_t,
    pub currentAim: c_int,
    pub currentAggression: c_int,

    // scriptflags
    pub scriptFlags: c_int, // in b_local.h

    // moveInfo
    pub desiredSpeed: c_int,
    pub currentSpeed: c_int,
    pub last_forwardmove: core::ffi::c_char,
    pub last_rightmove: core::ffi::c_char,
    pub lastClearOrigin: vec3_t,
    pub consecutiveBlockedMoves: c_int,
    pub blockedDebounceTime: c_int,
    pub shoveCount: c_int,
    pub blockedDest: vec3_t,

    pub combatPoint: c_int, // NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
    pub lastFailedCombatPoint: c_int, // NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
    pub movementSpeech: c_int,        // what to say when you first successfully move
    pub movementSpeechChance: f32,    // how likely you are to say it

    // Testing physics at 20fps
    pub nextBStateThink: c_int,
    pub last_ucmd: usercmd_t,

    // JWEIER ADDITIONS START
    pub combatMove: qboolean,
    pub goalRadius: c_int,

    // FIXME: These may be redundant
    pub pauseTime: c_int, // Time to stand still
    pub standTime: c_int,

    pub localState: c_int, // Tracking information local to entity
    pub squadState: c_int, // Tracking information for team level interaction
    // JWEIER ADDITIONS END
    pub confusionTime: c_int, // Doesn't respond to alerts or pick up enemies (unless shot) until this time is up
    pub charmedTime: c_int,   // charmed to enemy team
    pub controlledTime: c_int, // controlled by player
    pub surrenderTime: c_int, // Hands up

    // Lagging enemy position - FIXME: seems awful wasteful...
    pub enemyLaggedPos: [vec3_t; ENEMY_POS_LAG_STEPS],

    pub watchTarget: *mut gentity_t, // for BS_CINEMATIC, keeps facing this ent

    pub ffireCount: c_int, // need 3 int fields...
    pub ffireDebounce: c_int,
    pub ffireFadeDebounce: c_int,
}

// `gNPC_t` carries `gentity_t*`/`AIGroupInfo_t*` pointers => arch-dependent layout,
// validated at the host 64-bit layout (like the g_local.h masters).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::align_of::<gNPC_t>() == 8);
// `enemyLastVisibility` is the first field after the leading `gentity_t*`, so its
// offset pins the pointer width (4 + ptr(8) + pad = 16 on 64-bit).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gNPC_t, enemyLastVisibility) == 16);

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;
    use core::mem::{offset_of, size_of};

    /// Parity: the `lookMode_t` enumerator values match the authentic C.
    #[test]
    fn lookmode_values_match_c() {
        unsafe {
            assert_eq!(LM_ENT, jka_bp_LM_ENT());
            assert_eq!(LM_INTEREST, jka_bp_LM_INTEREST());
        }
    }

    /// Parity: `gNPCstats_t` (pointer-free) and the pointer-bearing `gNPC_t` match
    /// the authentic C layout. `gNPC_t` carries `gentity_t*`/`AIGroupInfo_t*` so its
    /// size/offsets are validated at the host 64-bit layout.
    #[test]
    fn gnpc_layout_matches_c() {
        unsafe {
            assert_eq!(size_of::<gNPCstats_t>(), jka_bp_sizeof_gNPCstats_t());
            assert_eq!(
                offset_of!(gNPCstats_t, aggression),
                jka_bp_off_stats_aggression()
            );
            assert_eq!(
                offset_of!(gNPCstats_t, runSpeed),
                jka_bp_off_stats_runSpeed()
            );
            assert_eq!(
                offset_of!(gNPCstats_t, acceleration),
                jka_bp_off_stats_acceleration()
            );

            #[cfg(target_pointer_width = "64")]
            {
                assert_eq!(size_of::<gNPC_t>(), jka_bp_sizeof_gNPC_t());
                // first field
                assert_eq!(
                    offset_of!(gNPC_t, timeOfDeath),
                    jka_bp_off_npc_timeOfDeath()
                );
                // a middle field (embedded gNPCstats_t)
                assert_eq!(offset_of!(gNPC_t, stats), jka_bp_off_npc_stats());
                // the embedded usercmd_t
                assert_eq!(offset_of!(gNPC_t, last_ucmd), jka_bp_off_npc_last_ucmd());
                // last field
                assert_eq!(
                    offset_of!(gNPC_t, ffireFadeDebounce),
                    jka_bp_off_npc_ffireFadeDebounce()
                );
            }
        }
    }
}
