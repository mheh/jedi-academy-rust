//! `g_main.c` — the module's `vmMain` dispatch plus the `Com_*` error/print seam.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_main.c`. Two responsibilities live
//! here, as in the C:
//!
//! * the `vmMain` `switch` dispatch ([`vm_main`]), driven from
//!   [`crate::ffi::exports`];
//! * `Com_Error`/`Com_Printf`, the varargs forwarders to the `G_Error`/`G_Printf`
//!   traps. The C bodies are trivial:
//!   ```c
//!   void QDECL Com_Error ( int level, const char *error, ... ) { ...vsprintf...; G_Error( "%s", text ); }
//!   void QDECL Com_Printf( const char *msg, ... )             { ...vsprintf...; G_Printf( "%s", text ); }
//!   ```
//!   The C varargs + `vsprintf` become a pre-rendered `&str` (callers format with
//!   Rust's `format!`/`format_args!`) — the stage-1 deviation for the
//!   printf-family functions. The foundation code in
//!   [`crate::codemp::game::q_shared`] calls these; Rust permits the mutual `use`
//!   between sibling modules.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)] // C global names (`level`, `g_gametype`, ...) kept verbatim
#![allow(non_camel_case_types)] // C type names (`cvarTable_t`) kept verbatim

use core::ffi::{c_char, c_int, c_uint, c_void, CStr};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::bg_lib;
use crate::codemp::game::bg_panimate::{BG_ClearAnimsets, BG_InitAnimsets};
use crate::codemp::game::bg_public::{
    CS_CLIENT_DUELHEALTHS, CS_CLIENT_DUELISTS, CS_CLIENT_DUELWINNER, CS_INTERMISSION, CS_SCORES1,
    CS_SCORES2, CS_TEAMVOTE_TIME, CS_VOTE_TIME, CS_WARMUP, DUELTEAM_DOUBLE, DUELTEAM_FREE,
    DUELTEAM_LONE, ET_GENERAL, EV_GLOBAL_DUEL, EV_PLAYER_TELEPORT_IN, GT_CTF, GT_CTY, GT_DUEL,
    GT_HOLOCRON, GT_JEDIMASTER, GT_MAX_GAME_TYPE, GT_POWERDUEL, GT_SIEGE, GT_SINGLE_PLAYER,
    GT_TEAM, MOD_SUICIDE, PERS_RANK, PERS_SCORE, PMF_FOLLOW, PM_INTERMISSION, RANK_TIED_FLAG,
    SCORE_NOT_PRESENT, STAT_ARMOR, STAT_CLIENTS_READY, STAT_HEALTH, TEAM_BLUE, TEAM_FREE,
    TEAM_NUM_TEAMS, TEAM_RED, TEAM_SPECTATOR, VOTE_TIME,
};
use crate::codemp::game::g_client::{
    ClientBegin, ClientConnect, ClientDisconnect, ClientSpawn, ClientUserinfoChanged, SelectSpawnPoint, TeamCount, g2SaberInstance, respawn
};
use crate::codemp::game::g_cmds::{ClientCommand, DeathmatchScoreboardMessage, SetTeam, StopFollowing};
use crate::codemp::game::g_combat::player_die;
use crate::codemp::game::g_local::{
    gclient_t, gentity_t, level_locals_t, CON_CONNECTED, CON_CONNECTING, CON_DISCONNECTED,
    FL_TEAMSLAVE, GAMEVERSION, INTERMISSION_DELAY_TIME, MAX_G_SHARED_BUFFER_SIZE, SPECTATOR_FOLLOW,
    SPECTATOR_SCOREBOARD,
};
use crate::codemp::game::g_mem::G_InitMemory;
use crate::codemp::game::g_log::{G_LogWeaponInit, G_LogWeaponOutput};
use crate::codemp::game::g_misc::{gEscapeTime, gEscaping, TAG_Init};
use crate::codemp::game::g_public_h::{SVF_BOT, SVF_BROADCAST};
use crate::codemp::game::g_saga::{g_siegePersistant, SiegeDoTeamAssign};
use crate::codemp::game::g_session::{G_InitWorldSession, G_WriteSessionData};
use crate::codemp::game::g_spawn::{precachedKyle, G_ParseSpawnVars, G_SpawnGEntityFromSpawnVars};
use crate::codemp::game::g_svcmds::{ConsoleCommand, G_LoadIPBans, G_SaveBanIP};
use crate::codemp::game::g_utils::{
    G_CleanAllFakeClients, G_Find, G_PickTarget, G_ROFF_NotetrackCallback, G_SoundIndex,
    G_TempEntity, G_UseTargets2,
};
use crate::codemp::game::q_math::{vec3_origin, vectoangles, VectorCopy, VectorSubtract};
use crate::codemp::game::q_shared::{va, Com_sprintf, Q_CleanStr, Q_stricmp, Q_strncmp, Sz};
use crate::codemp::game::q_shared_h::{
    CVAR_ARCHIVE, CVAR_CHEAT, CVAR_INTERNAL, CVAR_LATCH, CVAR_NORESTART, CVAR_ROM, CVAR_SERVERINFO,
    CVAR_SYSTEMINFO, CVAR_USERINFO, FS_APPEND, FS_APPEND_SYNC, MAX_CLIENTS, MAX_GENTITIES,
    MAX_INFO_STRING, MAX_SABERS,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_TRIGGER;
use crate::ffi::types::{qboolean, vmCvar_t, QFALSE, QTRUE};
use crate::ffi::GameExport;
use crate::trap;

use crate::codemp::game::ai_main::{BotAILoadMap, BotAISetup, BotAIShutdown, BotAIStartFrame};
use crate::codemp::game::ai_util::{B_CleanupAlloc, B_InitAlloc};
use crate::codemp::game::bg_vehicleLoad::BG_VehicleLoadParms;
use crate::codemp::game::bg_saberLoad::WP_SaberLoadParms;
use crate::codemp::game::npc::NPC_InitGame;
use crate::codemp::game::g_timer::TIMER_Clear;
use crate::codemp::game::g_client::InitBodyQue;
use crate::codemp::game::g_items::{ClearRegisteredItems, G_CheckTeamItems, SaveRegisteredItems};
use crate::codemp::game::g_saga::InitSiegeMode;
use crate::codemp::game::g_spawn::G_SpawnEntitiesFromString;
use crate::codemp::game::g_bot::G_InitBots;
use crate::codemp::game::g_nav::navCalculatePaths;
use crate::codemp::game::g_local::{SP_PODIUM_MODEL, START_TIME_NAV_CALC};
use crate::codemp::game::bg_misc::bg_customSiegeSoundNames;
use crate::codemp::game::bg_public::{CS_CLIENT_JEDIMASTER, MAX_CUSTOM_SIEGE_SOUNDS};
use crate::codemp::game::g_utils::G_ModelIndex;
use crate::codemp::game::g_bot::G_RefreshNextMap;
use crate::codemp::game::g_nav::{fatalErrors, NAV_CalculatePaths};

// G_RunFrame callees + constants (per-frame tick).
use crate::codemp::game::anims::BOTH_CONSOLE1;
use crate::codemp::game::bg_public::{
    EF_SOUNDTRACKER, ET_ITEM, ET_MISSILE, ET_MOVER, ET_NPC, EVENT_VALID_MSEC, HANDEXTEND_CHOKE,
    PW_CLOAKED, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, SETANIM_TORSO,
};
use crate::codemp::game::bg_saga::bgSiegeClasses;
use crate::codemp::game::bg_saga_h::{CFL_STATVIEWER, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2};
use crate::codemp::game::g_active::{ClientEndFrame, ClientThink, G_CheckClientTimeouts, G_RunClient};
use crate::codemp::game::g_combat::G_Damage;
use crate::codemp::game::g_items::{G_RunItem, Jetpack_Off};
use crate::codemp::game::g_local::DAMAGE_NO_ARMOR;
use crate::codemp::game::g_missile::G_RunMissile;
use crate::codemp::game::g_mover::G_RunMover;
use crate::codemp::game::bg_misc::BG_GetItemIndexByTag;
use crate::codemp::game::g_ICARUScb::{
    Q3_GetFloat, Q3_GetString, Q3_GetTag, Q3_GetVector, Q3_Kill, Q3_Lerp2Angles, Q3_Lerp2End,
    Q3_Lerp2Origin, Q3_Lerp2Pos, Q3_Lerp2Start, Q3_Play, Q3_PlaySound, Q3_Remove, Q3_Set,
    Q3_Use, setTable,
};
use crate::codemp::game::q_shared::GetIDForString;
use crate::codemp::game::q_shared_h::stringID_table_t;
use crate::codemp::game::g_mover::{G_EntIsBreakable, G_EntIsDoor, G_EntIsRemovableUsable, G_EntIsUnlockedDoor};
use crate::codemp::game::g_nav::{NAV_ClearPathToPoint, NAV_FindPlayerWaypoint, WAYPOINT_NONE};
use crate::codemp::game::g_navnew::{NAV_CheckNodeFailedForEnt, NAVNEW_ClearPathBetweenPoints};
use crate::codemp::game::npc_combat::CP_FindCombatPointWaypoints;
use crate::codemp::game::npc_utils::NPC_ClearLOS2;
use crate::codemp::game::g_saga::{
    gSiegeRoundEnded, gSiegeRoundWinningTeam, G_SiegeClientExData, SiegeCheckTimers,
};
use crate::codemp::game::g_team::CheckTeamStatus;
use crate::codemp::game::g_utils::{
    G_EntitySound, G_FreeEntity, G_PointInBounds, G_SendG2KillQueue, G_SetAnim,
};
use crate::codemp::game::npc::ClearNPCGlobals;
use crate::codemp::game::npc_ai_jedi::Jedi_Decloak;
use crate::codemp::game::npc_ai_utils::AI_UpdateGroups;
use crate::codemp::game::npc_senses::ClearPlayerAlertEvents;
use crate::codemp::game::q_math::VectorLength;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    vec3_t, BUTTON_USE, CHAN_VOICE, ENTITYNUM_NONE, MAX_POWERUPS,
};
use crate::codemp::game::w_force::WP_ForcePowersUpdate;
use crate::codemp::game::w_saber::{WP_SaberPositionUpdate, WP_SaberStartMissileBlockCheck};

/// `q_shared.h` — `#define S_COLOR_RED "^1"` (color escape for console text).
const S_COLOR_RED: &str = "^1";

// `G_InitGame` seeds the RNG with `srand(randomSeed)`. In the retail
// dedicated-server build (`Q3_VM` undefined) this resolves to the C library
// `srand`; bg_lib.c's own `srand`/`rand` are the `#ifdef Q3_VM` (the `vm`
// feature) path. Declared the same way the q_shared.c port pulls in libc
// ctype/atoi helpers, so the native build links the platform `srand`.
extern "C" {
    fn srand(seed: c_uint);
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn atof(s: *const c_char) -> f64;
}

/// `level_locals_t level;` (g_main.c) — the single global holding the current
/// map's game state ("this structure is cleared as each map is entered"). The C
/// global lives in zeroed BSS and `G_InitGame` `memset`s it to zero on each map
/// load; mirrored here as a zero-initialised mutable static. Always take a
/// pointer with [`core::ptr::addr_of_mut!`] / [`core::ptr::addr_of!`] — never a
/// `&`/`&mut` to a `static mut` (the `static_mut_refs` lint, and UB if aliased).
pub static mut level: level_locals_t = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

/// Process-wide test-only mutex guarding the shared mutable `level` (and the cvar globals
/// `g_gametype`/`g_gravity`/`g_knockback`/…) that oracle tests across modules mutate. The
/// parallel test runner would otherwise let a test in one module clobber `level.time` while
/// a test in another reads it back; every test that writes those statics must take this one
/// shared lock so they serialize. (Previously each module kept its own private mutex, which
/// only serialized within a module — a latent cross-module race.)
#[cfg(test)]
pub(crate) fn level_lock() -> std::sync::MutexGuard<'static, ()> {
    static LEVEL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LEVEL_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// `gentity_t g_entities[MAX_GENTITIES]` (g_main.c:27) — the entity array. A true
/// static (BSS) array, matching PC's literal form, zero-initialised like [`g_clients`]
/// and re-`memset` each map load by [`G_InitGame`]. External linkage in C (the rest of
/// the game indexes `g_entities[i]`), so `pub`.
pub static mut g_entities: [gentity_t; MAX_GENTITIES] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

/// `gclient_t g_clients[MAX_CLIENTS]` (g_main.c) — the client array. Unlike the
/// heap-allocated `g_entities`, this is a static (BSS) array; zero-initialised
/// like [`level`], and re-`memset` each map load by [`G_InitGame`].
pub static mut g_clients: [gclient_t; MAX_CLIENTS] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

/// `char gSharedBuffer[MAX_G_SHARED_BUFFER_SIZE]` (g_main.c) — the module's
/// shared-memory scratch buffer. Registered with the engine via
/// [`trap::SV_RegisterSharedMemory`] in [`G_InitGame`]; the engine fills it for
/// certain callbacks (the ICARUS bridge) for the module to read back.
pub static mut gSharedBuffer: [c_char; MAX_G_SHARED_BUFFER_SIZE] = [0; MAX_G_SHARED_BUFFER_SIZE];

// Shared-memory argument structs for ICARUS vmMain callbacks (g_public.h:796-918).
// The engine fills `gSharedBuffer` with one of these before calling vmMain.
#[repr(C)] pub struct T_G_ICARUS_PLAYSOUND    { pub taskID: c_int, pub entID: c_int, pub name: [c_char; 2048], pub channel: [c_char; 2048] }
#[repr(C)] pub struct T_G_ICARUS_SET          { pub taskID: c_int, pub entID: c_int, pub type_name: [c_char; 2048], pub data: [c_char; 2048] }
#[repr(C)] pub struct T_G_ICARUS_LERP2POS     { pub taskID: c_int, pub entID: c_int, pub origin: vec3_t, pub angles: vec3_t, pub duration: f32, pub nullAngles: qboolean }
#[repr(C)] pub struct T_G_ICARUS_LERP2ORIGIN  { pub taskID: c_int, pub entID: c_int, pub origin: vec3_t, pub duration: f32 }
#[repr(C)] pub struct T_G_ICARUS_LERP2ANGLES  { pub taskID: c_int, pub entID: c_int, pub angles: vec3_t, pub duration: f32 }
#[repr(C)] pub struct T_G_ICARUS_GETTAG       { pub entID: c_int, pub name: [c_char; 2048], pub lookup: c_int, pub info: vec3_t }
#[repr(C)] pub struct T_G_ICARUS_LERP2START   { pub entID: c_int, pub taskID: c_int, pub duration: f32 }
#[repr(C)] pub struct T_G_ICARUS_LERP2END     { pub entID: c_int, pub taskID: c_int, pub duration: f32 }
#[repr(C)] pub struct T_G_ICARUS_USE          { pub entID: c_int, pub target: [c_char; 2048] }
#[repr(C)] pub struct T_G_ICARUS_KILL         { pub entID: c_int, pub name: [c_char; 2048] }
#[repr(C)] pub struct T_G_ICARUS_REMOVE       { pub entID: c_int, pub name: [c_char; 2048] }
#[repr(C)] pub struct T_G_ICARUS_PLAY         { pub taskID: c_int, pub entID: c_int, pub type_: [c_char; 2048], pub name: [c_char; 2048] }
#[repr(C)] pub struct T_G_ICARUS_GETFLOAT     { pub entID: c_int, pub type_: c_int, pub name: [c_char; 2048], pub value: f32 }
#[repr(C)] pub struct T_G_ICARUS_GETVECTOR    { pub entID: c_int, pub type_: c_int, pub name: [c_char; 2048], pub value: vec3_t }
#[repr(C)] pub struct T_G_ICARUS_GETSTRING    { pub entID: c_int, pub type_: c_int, pub name: [c_char; 2048], pub value: [c_char; 2048] }
#[repr(C)] pub struct T_G_ICARUS_SOUNDINDEX   { pub filename: [c_char; 2048] }
#[repr(C)] pub struct T_G_ICARUS_GETSETIDFORSTRING { pub string: [c_char; 2048] }

/// `qboolean gQueueScoreMessage` / `int gQueueScoreMessageTime` (g_main.c:1714) —
/// rww's deferred-scoreboard "queue": rather than broadcasting the scoreboard the
/// instant the score changes (which overflowed reliable-command buffers),
/// [`CalculateRanks`] sets these and the per-frame `G_RunFrame` tail flushes the
/// scoreboard once `gQueueScoreMessageTime` is reached. Module-internal file-scope
/// globals in C; mirrored as zero-initialised mutable statics. The consumer
/// (`G_RunFrame`, g_main.c:4189) is not yet ported.
pub static mut gQueueScoreMessage: qboolean = QFALSE;
pub static mut gQueueScoreMessageTime: c_int = 0;

/// `qboolean gDuelExit = qfalse;` (g_main.c:39) — set true when a duel ends on the
/// win-limit (so the winner's name is printed) and false when it ends on a kill/score
/// limit; consumed by the duel intermission / scoreboard code. First read+written in
/// [`CheckExitRules`]. A single-threaded game-module global; always via
/// `addr_of!`/`addr_of_mut!`.
pub static mut gDuelExit: qboolean = QFALSE;

/// `qboolean g_endPDuel = qfalse;` (g_main.c:2580) — latched by the power-duel logic
/// to request that the current power-duel round end on the next [`CheckExitRules`]
/// pass (cleared there when it fires `LogExit("Powerduel ended.")`).
pub static mut g_endPDuel: qboolean = QFALSE;

/// `qboolean gDoSlowMoDuel = qfalse;` (g_main.c:3594) — true while a duel-end
/// slow-motion sequence is playing; [`CheckExitRules`] bails out (never advances to
/// intermission) while it is set so the slow-mo finishes first.
pub static mut gDoSlowMoDuel: qboolean = QFALSE;

/// `int gSlowMoDuelTime = 0;` (g_main.c:3595) — `level.time` at which the duel-end
/// slow-motion sequence began; latched by `G_Damage` (g_combat) and consumed by the
/// slow-mo timescale ramp in [`G_RunFrame`]. (Homed here per its C definition; was
/// temporarily held in g_combat.rs before G_RunFrame landed.)
pub static mut gSlowMoDuelTime: c_int = 0;

/// `int g_LastFrameTime = 0;` (g_main.c:3591) — `level.time` of the previous frame,
/// stamped at the end of every [`G_RunFrame`]; read by w_force (`g_TimeSinceLastFrame`
/// derivation + the first-frame init). (Homed here per its C definition; w_force.c only
/// `extern`s it.)
pub static mut g_LastFrameTime: c_int = 0;

/// `int g_TimeSinceLastFrame = 0;` (g_main.c:3592) — per-frame delta
/// (`level.time - g_LastFrameTime`) computed once each [`G_RunFrame`]; read by w_force's
/// saber-danger timing. (Homed here per its C definition; w_force.c only `extern`s it.)
pub static mut g_TimeSinceLastFrame: c_int = 0;

/// `int g_siegeRespawnCheck = 0;` (g_main.c:3657) — the level-time stamp at which the
/// next siege respawn wave fires; written by the (not-yet-ported) siege-respawn logic
/// and read by `respawn` (g_client.c) to stamp the `EV_SIEGESPEC` temp-entity's
/// respawn-countdown time. A single-threaded game-module global; always via
/// `addr_of!`/`addr_of_mut!`.
pub static mut g_siegeRespawnCheck: c_int = 0;

/// `static int navCalcPathTime = 0;` (g_main.c:19) — when non-zero and past, the deferred
/// one-time nav-path calculation fires in [`NAV_CheckCalcPaths`]. Set by `G_InitGame`
/// (not yet ported), so it stays 0 here until that lands (faithful: the calc just doesn't fire).
pub static mut navCalcPathTime: c_int = 0;

/// `cvarTable_t` (g_main.c) — one row of the module's cvar registration table
/// [`gameCvarTable`]. File-local in C and never crossing the engine ABI, so the
/// layout need only be self-consistent. The C `char *` name/default fields point
/// at immutable string literals, represented here as `*const c_char` (the table
/// is read-only data apart from `modificationCount`, which `G_RegisterCvars`
/// updates — hence the table is a `static mut`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct cvarTable_t {
    pub vmCvar: *mut vmCvar_t,
    pub cvarName: *const c_char,
    pub defaultString: *const c_char,
    pub cvarFlags: c_int,
    pub modificationCount: c_int, // for tracking changes
    pub trackChange: qboolean,    // track this variable, and announce if changed
    pub teamShader: qboolean,     // track and if changed, update shader state
}

// Pin the host 64-bit layout (3 pointers + 4 ints == 40 bytes) to catch any
// field-order slip, even though this struct is internal-only.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<cvarTable_t>() == 40);
    assert!(core::mem::offset_of!(cvarTable_t, vmCvar) == 0);
    assert!(core::mem::offset_of!(cvarTable_t, cvarFlags) == 24);
    assert!(core::mem::offset_of!(cvarTable_t, teamShader) == 36);
};

// ===========================================================================
// The cvar mirrors backing `gameCvarTable`. In C these are non-static globals
// (extern-declared in g_local.h) that the rest of the game reads; mirrored as
// zero-initialised mutable statics that the engine fills in via
// `trap_Cvar_Register`. Transcribed verbatim in source order for the retail
// dedicated-server build: `_DEBUG`-only (`g_disableServerG2`) and
// `DEBUG_SABER_BOX`-only (`g_saberDebugBox`) cvars are omitted; the
// non-`FINAL_BUILD` `g_debugDamage` is kept (see DEVIATIONS.md "g_main.c build
// config").
// ===========================================================================

pub static mut g_trueJedi: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_gametype: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_MaxHolocronCarry: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_ff_objectives: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_autoMapCycle: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_dmflags: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_maxForceRank: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_forceBasedTeams: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_privateDuel: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_allowNPC: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_armBreakage: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_saberLocking: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_saberLockFactor: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_saberTraceSaberFirst: vmCvar_t = vmCvar_t::zeroed();

pub static mut d_saberKickTweak: vmCvar_t = vmCvar_t::zeroed();

pub static mut d_powerDuelPrint: vmCvar_t = vmCvar_t::zeroed();

pub static mut d_saberGhoul2Collision: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_saberBladeFaces: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_saberAlwaysBoxTrace: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_saberBoxTraceSize: vmCvar_t = vmCvar_t::zeroed();

pub static mut d_siegeSeekerNPC: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_debugMelee: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_stepSlideFix: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_noSpecMove: vmCvar_t = vmCvar_t::zeroed();

// `g_disableServerG2` is `#ifdef _DEBUG` only — omitted in the retail build.

pub static mut d_perPlayerGhoul2: vmCvar_t = vmCvar_t::zeroed();

pub static mut d_projectileGhoul2Collision: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_g2TraceLod: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_optvehtrace: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_locationBasedDamage: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_allowHighPingDuelist: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_logClientInfo: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_slowmoDuelEnd: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_saberDamageScale: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_useWhileThrowing: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_RMG: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_svfps: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_forceRegenTime: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_spawnInvulnerability: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_forcePowerDisable: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_weaponDisable: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_duelWeaponDisable: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_allowDuelSuicide: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_fraglimitVoteCorrection: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_fraglimit: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_duel_fraglimit: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_timelimit: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_capturelimit: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_saberInterpolate: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_friendlyFire: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_friendlySaber: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_password: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_needpass: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_maxclients: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_maxGameClients: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_dedicated: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_developer: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_speed: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_gravity: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_cheats: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_knockback: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_quadfactor: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_forcerespawn: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_siegeRespawn: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_inactivity: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_debugMove: vmCvar_t = vmCvar_t::zeroed();
// `g_debugDamage` is `#ifndef FINAL_BUILD` — present in this (non-final) build.
pub static mut g_debugDamage: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_debugAlloc: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_debugServerSkel: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_weaponRespawn: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_weaponTeamRespawn: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_adaptRespawn: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_motd: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_synchronousClients: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_warmup: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_doWarmup: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_restarted: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_log: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_logSync: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_statLog: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_statLogFile: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_blood: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_podiumDist: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_podiumDrop: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_allowVote: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_allowTeamVote: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_teamAutoJoin: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_teamForceBalance: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_banIPs: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_filterBan: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_debugForward: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_debugRight: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_debugUp: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_smoothClients: vmCvar_t = vmCvar_t::zeroed();

// In C these two live in the bg (`#include "../namespace_begin.h"`) namespace.
pub static mut pmove_fixed: vmCvar_t = vmCvar_t::zeroed();
pub static mut pmove_msec: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_listEntity: vmCvar_t = vmCvar_t::zeroed();
// `g_redteam`/`g_blueteam` are commented out in the PC source — omitted.
pub static mut g_singlePlayer: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_enableBreath: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_dismember: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_forceDodge: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_timeouttospec: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_saberDmgVelocityScale: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_saberDmgDelay_Idle: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_saberDmgDelay_Wound: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_saberDebugPrint: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_siegeTeamSwitch: vmCvar_t = vmCvar_t::zeroed();

pub static mut bg_fighterAltControl: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_vehAutoAimLead: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_autoKickKillSpammers: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_autoBanKillSpammers: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_autoKickTKSpammers: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_autoBanTKSpammers: vmCvar_t = vmCvar_t::zeroed();

// `g_saberDebugBox` is `#ifdef DEBUG_SABER_BOX` only — omitted in the retail build.

//NPC nav debug
pub static mut d_altRoutes: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_patched: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_saberRealisticCombat: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_saberRestrictForce: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_saberSPStyleDamage: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_debugSaberLocks: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_saberLockRandomNess: vmCvar_t = vmCvar_t::zeroed();
// nmckenzie: SABER_DAMAGE_WALLS
pub static mut g_saberWallDamageScale: vmCvar_t = vmCvar_t::zeroed();

pub static mut d_saberStanceDebug: vmCvar_t = vmCvar_t::zeroed();
// ai debug cvars
pub static mut debugNPCAI: vmCvar_t = vmCvar_t::zeroed(); // used to print out debug info about the bot AI
pub static mut debugNPCFreeze: vmCvar_t = vmCvar_t::zeroed(); // set to disable bot ai and temporarily freeze them in place
pub static mut debugNPCAimingBeam: vmCvar_t = vmCvar_t::zeroed();
pub static mut debugBreak: vmCvar_t = vmCvar_t::zeroed();
pub static mut debugNoRoam: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_saberCombat: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_JediAI: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_noGroupAI: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_asynchronousGroupAI: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_slowmodeath: vmCvar_t = vmCvar_t::zeroed();
pub static mut d_noIntermissionWait: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_spskill: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_siegeTeam1: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_siegeTeam2: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_austrian: vmCvar_t = vmCvar_t::zeroed();

pub static mut g_powerDuelStartHealth: vmCvar_t = vmCvar_t::zeroed();
pub static mut g_powerDuelEndHealth: vmCvar_t = vmCvar_t::zeroed();

// nmckenzie: temporary way to show player healths in duels. DUEL_HEALTH
pub static mut g_showDuelHealths: vmCvar_t = vmCvar_t::zeroed();

/// Build one [`cvarTable_t`] row. Mirrors a C aggregate initializer
/// `{ &vmCvar, "name", "default", flags, mod, track, team }` positionally; C's
/// omitted trailing fields (which it zero-fills) are written here as their
/// explicit zero values (`0` / [`QFALSE`]). `name`/`def` take `&CStr` literals.
const fn cv(
    vmCvar: *mut vmCvar_t,
    cvarName: &'static CStr,
    defaultString: &'static CStr,
    cvarFlags: c_int,
    modificationCount: c_int,
    trackChange: qboolean,
    teamShader: qboolean,
) -> cvarTable_t {
    cvarTable_t {
        vmCvar,
        cvarName: cvarName.as_ptr(),
        defaultString: defaultString.as_ptr(),
        cvarFlags,
        modificationCount,
        trackChange,
        teamShader,
    }
}

/// Number of rows in [`gameCvarTable`] (C: `sizeof(gameCvarTable)/sizeof([0])`).
/// A miscount here is a compile error (array length vs initializer count); the
/// oracle test cross-checks it against the real C table.
pub const gameCvarTableSize: usize = 139;

/// `static cvarTable_t gameCvarTable[]` (g_main.c). The cvar registration table,
/// transcribed verbatim in source order for the retail dedicated-server build.
/// `static mut` because `G_RegisterCvars` writes each row's `modificationCount`.
#[rustfmt::skip]
pub static mut gameCvarTable: [cvarTable_t; gameCvarTableSize] = [
    // don't override the cheat state set by the system
    cv(addr_of_mut!(g_cheats), c"sv_cheats", c"", 0, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_debugMelee), c"g_debugMelee", c"0", CVAR_SERVERINFO, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_stepSlideFix), c"g_stepSlideFix", c"1", CVAR_SERVERINFO, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_noSpecMove), c"g_noSpecMove", c"0", CVAR_SERVERINFO, 0, QTRUE, QFALSE),

    // noset vars
    cv(null_mut(), c"gamename", c"basejka" /* GAMEVERSION */, CVAR_SERVERINFO | CVAR_ROM, 0, QFALSE, QFALSE),
    // __DATE__: the C compiler's build date; not reproduced (see DEVIATIONS.md), so empty here.
    cv(null_mut(), c"gamedate", c"", CVAR_ROM, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_restarted), c"g_restarted", c"0", CVAR_ROM, 0, QFALSE, QFALSE),
    cv(null_mut(), c"sv_mapname", c"", CVAR_SERVERINFO | CVAR_ROM, 0, QFALSE, QFALSE),

    // latched vars
    cv(addr_of_mut!(g_gametype), c"g_gametype", c"0", CVAR_SERVERINFO | CVAR_LATCH, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_MaxHolocronCarry), c"g_MaxHolocronCarry", c"3", CVAR_SERVERINFO | CVAR_LATCH, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_maxclients), c"sv_maxclients", c"8", CVAR_SERVERINFO | CVAR_LATCH | CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_maxGameClients), c"g_maxGameClients", c"0", CVAR_SERVERINFO | CVAR_LATCH | CVAR_ARCHIVE, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_trueJedi), c"g_jediVmerc", c"0", CVAR_SERVERINFO | CVAR_LATCH | CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    // change anytime vars
    cv(addr_of_mut!(g_ff_objectives), c"g_ff_objectives", c"0", /*CVAR_SERVERINFO |*/ CVAR_CHEAT | CVAR_NORESTART, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_autoMapCycle), c"g_autoMapCycle", c"0", CVAR_ARCHIVE | CVAR_NORESTART, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_dmflags), c"dmflags", c"0", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_maxForceRank), c"g_maxForceRank", c"6", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_forceBasedTeams), c"g_forceBasedTeams", c"0", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_privateDuel), c"g_privateDuel", c"1", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_allowNPC), c"g_allowNPC", c"1", CVAR_SERVERINFO | CVAR_CHEAT, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_armBreakage), c"g_armBreakage", c"0", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_saberLocking), c"g_saberLocking", c"1", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_saberLockFactor), c"g_saberLockFactor", c"2", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_saberTraceSaberFirst), c"g_saberTraceSaberFirst", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(d_saberKickTweak), c"d_saberKickTweak", c"1", 0, 0, QTRUE, QFALSE),

    // Faithful quirk: the C source has only 5 initializers, so `qtrue` lands in
    // the modificationCount slot (not trackChange); trackChange is left 0.
    cv(addr_of_mut!(d_powerDuelPrint), c"d_powerDuelPrint", c"0", 0, QTRUE, QFALSE, QFALSE),

    cv(addr_of_mut!(d_saberGhoul2Collision), c"d_saberGhoul2Collision", c"1", CVAR_CHEAT, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_saberBladeFaces), c"g_saberBladeFaces", c"1", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(d_saberAlwaysBoxTrace), c"d_saberAlwaysBoxTrace", c"0", CVAR_CHEAT, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(d_saberBoxTraceSize), c"d_saberBoxTraceSize", c"0", CVAR_CHEAT, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(d_siegeSeekerNPC), c"d_siegeSeekerNPC", c"0", CVAR_CHEAT, 0, QTRUE, QFALSE),

    // `g_disableServerG2` row is `#ifdef _DEBUG` only — omitted in the retail build.

    cv(addr_of_mut!(d_perPlayerGhoul2), c"d_perPlayerGhoul2", c"0", CVAR_CHEAT, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(d_projectileGhoul2Collision), c"d_projectileGhoul2Collision", c"1", CVAR_CHEAT, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_g2TraceLod), c"g_g2TraceLod", c"3", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_optvehtrace), c"com_optvehtrace", c"0", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_locationBasedDamage), c"g_locationBasedDamage", c"1", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_allowHighPingDuelist), c"g_allowHighPingDuelist", c"1", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_logClientInfo), c"g_logClientInfo", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_slowmoDuelEnd), c"g_slowmoDuelEnd", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_saberDamageScale), c"g_saberDamageScale", c"1", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_useWhileThrowing), c"g_useWhileThrowing", c"1", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_RMG), c"RMG", c"0", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_svfps), c"sv_fps", c"20", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_forceRegenTime), c"g_forceRegenTime", c"200", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_spawnInvulnerability), c"g_spawnInvulnerability", c"3000", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_forcePowerDisable), c"g_forcePowerDisable", c"0", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_weaponDisable), c"g_weaponDisable", c"0", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_duelWeaponDisable), c"g_duelWeaponDisable", c"1", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_LATCH, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_allowDuelSuicide), c"g_allowDuelSuicide", c"1", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_fraglimitVoteCorrection), c"g_fraglimitVoteCorrection", c"1", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_fraglimit), c"fraglimit", c"20", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_duel_fraglimit), c"duel_fraglimit", c"10", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_timelimit), c"timelimit", c"0", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_capturelimit), c"capturelimit", c"8", CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_synchronousClients), c"g_synchronousClients", c"0", CVAR_SYSTEMINFO, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(d_saberInterpolate), c"d_saberInterpolate", c"0", CVAR_CHEAT, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_friendlyFire), c"g_friendlyFire", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_friendlySaber), c"g_friendlySaber", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_teamAutoJoin), c"g_teamAutoJoin", c"0", CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_teamForceBalance), c"g_teamForceBalance", c"0", CVAR_ARCHIVE, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_warmup), c"g_warmup", c"20", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_doWarmup), c"g_doWarmup", c"0", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_log), c"g_log", c"games.log", CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_logSync), c"g_logSync", c"0", CVAR_ARCHIVE, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_statLog), c"g_statLog", c"0", CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_statLogFile), c"g_statLogFile", c"statlog.log", CVAR_ARCHIVE, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_password), c"g_password", c"", CVAR_USERINFO, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_banIPs), c"g_banIPs", c"", CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_filterBan), c"g_filterBan", c"1", CVAR_ARCHIVE, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_needpass), c"g_needpass", c"0", CVAR_SERVERINFO | CVAR_ROM, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_dedicated), c"dedicated", c"0", 0, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_developer), c"developer", c"0", 0, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_speed), c"g_speed", c"250", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_gravity), c"g_gravity", c"800", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_knockback), c"g_knockback", c"1000", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_quadfactor), c"g_quadfactor", c"3", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_weaponRespawn), c"g_weaponrespawn", c"5", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_weaponTeamRespawn), c"g_weaponTeamRespawn", c"5", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_adaptRespawn), c"g_adaptrespawn", c"1", 0, 0, QTRUE, QFALSE),  // Make weapons respawn faster with a lot of players.
    cv(addr_of_mut!(g_forcerespawn), c"g_forcerespawn", c"60", 0, 0, QTRUE, QFALSE),  // One minute force respawn.  Give a player enough time to reallocate force.
    cv(addr_of_mut!(g_siegeRespawn), c"g_siegeRespawn", c"20", CVAR_SERVERINFO | CVAR_ARCHIVE, 0, QTRUE, QFALSE),  // siege respawn wave time
    cv(addr_of_mut!(g_inactivity), c"g_inactivity", c"0", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_debugMove), c"g_debugMove", c"0", 0, 0, QFALSE, QFALSE),
    // `g_debugDamage` is `#ifndef FINAL_BUILD` — present in this (non-final) build.
    cv(addr_of_mut!(g_debugDamage), c"g_debugDamage", c"0", 0, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_debugAlloc), c"g_debugAlloc", c"0", 0, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_debugServerSkel), c"g_debugServerSkel", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_motd), c"g_motd", c"", 0, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_blood), c"com_blood", c"1", 0, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_podiumDist), c"g_podiumDist", c"80", 0, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_podiumDrop), c"g_podiumDrop", c"70", 0, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_allowVote), c"g_allowVote", c"1", CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_allowTeamVote), c"g_allowTeamVote", c"1", CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_listEntity), c"g_listEntity", c"0", 0, 0, QFALSE, QFALSE),

    // The g_debugForward/Right/Up rows are `#if 0` in C — never compiled in.
    // The g_redteam/g_blueteam rows are commented out in the PC source — omitted.

    cv(addr_of_mut!(g_singlePlayer), c"ui_singlePlayerActive", c"", 0, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_enableBreath), c"g_enableBreath", c"0", 0, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_smoothClients), c"g_smoothClients", c"1", 0, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(pmove_fixed), c"pmove_fixed", c"0", CVAR_SYSTEMINFO, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(pmove_msec), c"pmove_msec", c"8", CVAR_SYSTEMINFO, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_dismember), c"g_dismember", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_forceDodge), c"g_forceDodge", c"1", 0, 0, QTRUE, QFALSE),

    cv(addr_of_mut!(g_timeouttospec), c"g_timeouttospec", c"70", CVAR_ARCHIVE, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_saberDmgVelocityScale), c"g_saberDmgVelocityScale", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_saberDmgDelay_Idle), c"g_saberDmgDelay_Idle", c"350", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_saberDmgDelay_Wound), c"g_saberDmgDelay_Wound", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    // `g_saberDebugPrint` is `#ifndef FINAL_BUILD` — present in this (non-final) build.
    cv(addr_of_mut!(g_saberDebugPrint), c"g_saberDebugPrint", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_debugSaberLocks), c"g_debugSaberLocks", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_saberLockRandomNess), c"g_saberLockRandomNess", c"2", CVAR_CHEAT, 0, QFALSE, QFALSE),
    // nmckenzie: SABER_DAMAGE_WALLS
    cv(addr_of_mut!(g_saberWallDamageScale), c"g_saberWallDamageScale", c"0.4", CVAR_SERVERINFO, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(d_saberStanceDebug), c"d_saberStanceDebug", c"0", 0, 0, QFALSE, QFALSE),

    // Faithful quirk: only 5 initializers in C, so `qfalse` (== 0) lands in the
    // modificationCount slot; trackChange/teamShader stay 0.
    cv(addr_of_mut!(g_siegeTeamSwitch), c"g_siegeTeamSwitch", c"1", CVAR_SERVERINFO | CVAR_ARCHIVE, QFALSE, QFALSE, QFALSE),

    cv(addr_of_mut!(bg_fighterAltControl), c"bg_fighterAltControl", c"0", CVAR_SERVERINFO, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_vehAutoAimLead), c"g_vehAutoAimLead", c"0", CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_autoKickKillSpammers), c"g_autoKickKillSpammers", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_autoBanKillSpammers), c"g_autoBanKillSpammers", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_autoKickTKSpammers), c"g_autoKickTKSpammers", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_autoBanTKSpammers), c"g_autoBanTKSpammers", c"0", CVAR_ARCHIVE, 0, QTRUE, QFALSE),

    // `g_saberDebugBox` row is `#ifdef DEBUG_SABER_BOX` only — omitted in the retail build.

    cv(addr_of_mut!(d_altRoutes), c"d_altRoutes", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(d_patched), c"d_patched", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_saberRealisticCombat), c"g_saberRealisticCombat", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_saberRestrictForce), c"g_saberRestrictForce", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(d_saberSPStyleDamage), c"d_saberSPStyleDamage", c"1", CVAR_CHEAT, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(debugNoRoam), c"d_noroam", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(debugNPCAimingBeam), c"d_npcaiming", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(debugBreak), c"d_break", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(debugNPCAI), c"d_npcai", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(debugNPCFreeze), c"d_npcfreeze", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(d_JediAI), c"d_JediAI", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(d_noGroupAI), c"d_noGroupAI", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(d_asynchronousGroupAI), c"d_asynchronousGroupAI", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),

    //0 = never (BORING)
    //1 = kyle only
    //2 = kyle and last enemy jedi
    //3 = kyle and any enemy jedi
    //4 = kyle and last enemy in a group
    //5 = kyle and any enemy
    //6 = also when kyle takes pain or enemy jedi dodges player saber swing or does an acrobatic evasion
    cv(addr_of_mut!(d_slowmodeath), c"d_slowmodeath", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(d_saberCombat), c"d_saberCombat", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_spskill), c"g_npcspskill", c"0", CVAR_ARCHIVE | CVAR_INTERNAL, 0, QFALSE, QFALSE),

    //for overriding the level defaults
    cv(addr_of_mut!(g_siegeTeam1), c"g_siegeTeam1", c"none", CVAR_ARCHIVE | CVAR_SERVERINFO, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_siegeTeam2), c"g_siegeTeam2", c"none", CVAR_ARCHIVE | CVAR_SERVERINFO, 0, QFALSE, QFALSE),

    //mainly for debugging with bots while I'm not around (want the server to
    //cycle through levels naturally)
    cv(addr_of_mut!(d_noIntermissionWait), c"d_noIntermissionWait", c"0", CVAR_CHEAT, 0, QFALSE, QFALSE),

    cv(addr_of_mut!(g_austrian), c"g_austrian", c"0", CVAR_ARCHIVE, 0, QFALSE, QFALSE),
    // nmckenzie: DUEL_HEALTH
    cv(addr_of_mut!(g_showDuelHealths), c"g_showDuelHealths", c"0", CVAR_SERVERINFO, 0, QFALSE, QFALSE),
    cv(addr_of_mut!(g_powerDuelStartHealth), c"g_powerDuelStartHealth", c"150", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
    cv(addr_of_mut!(g_powerDuelEndHealth), c"g_powerDuelEndHealth", c"90", CVAR_ARCHIVE, 0, QTRUE, QFALSE),
];

/// `void G_FindTeams( void )` (g_main.c:732).
///
/// Chain together all entities with a matching team field.
/// Entity teams are used for item groups and multi-entity mover groups.
///
/// All but the first will have the `FL_TEAMSLAVE` flag set and `teammaster` field set
/// All but the last will have the `teamchain` field set to the next one
///
/// No-oracle: pure pointer/field linking over the `g_entities` global array
/// (`teammaster`/`teamchain`/`team`/`targetname`/`flags`/`r.contents`), the same
/// `level`/`g_entities`-walker precedent as the other entity-graph mutators in this
/// file. Faithful 1:1; `strcmp` keeps the libc shape to match the C byte compare.
///
/// SAFETY: `g_entities` is a valid module static of `level.num_entities` in-use
/// entries once `G_InitGame`/spawn has run; `team`/`targetname` are NUL-terminated
/// C strings or null (guarded before deref).
pub fn G_FindTeams() {
    unsafe {
        // `c`/`c2` count teams and entities only for the commented-out debug print.
        let mut _c: c_int = 0;
        let mut _c2: c_int = 0;

        let num = (*addr_of!(level)).num_entities;
        let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();

        let mut i: c_int = 1;
        while i < num {
            let e = base.offset(i as isize);
            if (*e).inuse == QFALSE {
                i += 1;
                continue;
            }
            if (*e).team.is_null() {
                i += 1;
                continue;
            }
            if (*e).flags & FL_TEAMSLAVE != 0 {
                i += 1;
                continue;
            }
            if (*e).r.contents == CONTENTS_TRIGGER {
                i += 1;
                continue; // triggers NEVER link up in teams!
            }
            (*e).teammaster = e;
            _c += 1;
            _c2 += 1;

            let mut j: c_int = i + 1;
            while j < num {
                let e2 = base.offset(j as isize);
                if (*e2).inuse == QFALSE {
                    j += 1;
                    continue;
                }
                if (*e2).team.is_null() {
                    j += 1;
                    continue;
                }
                if (*e2).flags & FL_TEAMSLAVE != 0 {
                    j += 1;
                    continue;
                }
                if strcmp((*e).team, (*e2).team) == 0 {
                    _c2 += 1;
                    (*e2).teamchain = (*e).teamchain;
                    (*e).teamchain = e2;
                    (*e2).teammaster = e;
                    (*e2).flags |= FL_TEAMSLAVE;

                    // make sure that targets only point at the master
                    if !(*e2).targetname.is_null() {
                        (*e).targetname = (*e2).targetname;
                        (*e2).targetname = null_mut();
                    }
                }
                j += 1;
            }
            i += 1;
        }

        // G_Printf ("%i teams with %i entities\n", c, c2);
    }
}

/// `void G_RemapTeamShaders( void )`.
///
/// The entire C body is wrapped in `#if 0`, so this is a no-op. The disabled
/// code (kept here for reference) rebuilt the CTF team-icon shader remaps from
/// `g_redteam`/`g_blueteam` and pushed them via the `CS_SHADERSTATE` configstring:
/// ```c
/// char string[1024];
/// float f = level.time * 0.001;
/// Com_sprintf( string, sizeof(string), "team_icon/%s_red", g_redteam.string );
/// AddRemap("textures/ctf2/redteam01", string, f);
/// AddRemap("textures/ctf2/redteam02", string, f);
/// Com_sprintf( string, sizeof(string), "team_icon/%s_blue", g_blueteam.string );
/// AddRemap("textures/ctf2/blueteam01", string, f);
/// AddRemap("textures/ctf2/blueteam02", string, f);
/// trap_SetConfigstring(CS_SHADERSTATE, BuildShaderStateConfig());
/// ```
pub fn G_RemapTeamShaders() {}

/// `void G_RegisterCvars( void )` — register every cvar in [`gameCvarTable`] with
/// the engine, snapshot each modification count, remap team shaders if any tracked
/// one is a team shader, then range-check `g_gametype`.
pub fn G_RegisterCvars() {
    let mut remapped: qboolean = QFALSE;

    // for ( i = 0, cv = gameCvarTable ; i < gameCvarTableSize ; i++, cv++ )
    let tbl = addr_of_mut!(gameCvarTable);
    for i in 0..gameCvarTableSize {
        // SAFETY: `i < gameCvarTableSize` bounds the index; each row's `vmCvar`
        // points at a valid module static (or is null). `as_mut()` yields the
        // engine-writable mirror without forming a `&mut` to a `static mut` by
        // path (so it stays clear of the `static_mut_refs` lint). The C string
        // pointers come from this module's own `c"..."` literals.
        unsafe {
            let cv = addr_of_mut!((*tbl)[i]);
            let name = CStr::from_ptr((*cv).cvarName).to_str().unwrap_or_default();
            let value = CStr::from_ptr((*cv).defaultString)
                .to_str()
                .unwrap_or_default();
            trap::Cvar_Register((*cv).vmCvar.as_mut(), name, value, (*cv).cvarFlags);
            if !(*cv).vmCvar.is_null() {
                (*cv).modificationCount = (*(*cv).vmCvar).modificationCount;
            }
            if (*cv).teamShader != QFALSE {
                remapped = QTRUE;
            }
        }
    }

    if remapped != QFALSE {
        G_RemapTeamShaders();
    }

    // check some things
    // SAFETY: `g_gametype` is a valid module static. The engine has not refreshed
    // the mirror since registration, so the value is read once (C re-reads the
    // same unchanging field in each branch).
    let gametype = unsafe { (*addr_of!(g_gametype)).integer };
    if gametype < 0 || gametype >= GT_MAX_GAME_TYPE {
        G_Printf(&format!(
            "g_gametype {gametype} is out of range, defaulting to 0\n"
        ));
        trap::Cvar_Set("g_gametype", "0");
    } else if gametype == GT_HOLOCRON {
        G_Printf("This gametype is not supported.\n");
        trap::Cvar_Set("g_gametype", "0");
    } else if gametype == GT_JEDIMASTER {
        G_Printf("This gametype is not supported.\n");
        trap::Cvar_Set("g_gametype", "0");
    } else if gametype == GT_CTY {
        G_Printf("This gametype is not supported.\n");
        trap::Cvar_Set("g_gametype", "0");
    }

    // SAFETY: `level` and `g_warmup` are valid module statics.
    unsafe {
        (*addr_of_mut!(level)).warmupModificationCount = (*addr_of!(g_warmup)).modificationCount;
    }
}

/// `void G_UpdateCvars( void )` — refresh each cvar mirror; on a changed tracked
/// cvar, announce it to all clients, and flag a team-shader remap if needed.
pub fn G_UpdateCvars() {
    let mut remapped: qboolean = QFALSE;

    let tbl = addr_of_mut!(gameCvarTable);
    for i in 0..gameCvarTableSize {
        // SAFETY: `i` bounds the index; the table and the mirrors it points at are
        // valid module statics. `as_mut()` gives `None` for the NULL-mirror rows,
        // matching the C `if ( cv->vmCvar )` guard.
        unsafe {
            let cv = addr_of_mut!((*tbl)[i]);
            if let Some(vmcvar) = (*cv).vmCvar.as_mut() {
                trap::Cvar_Update(vmcvar);

                if (*cv).modificationCount != vmcvar.modificationCount {
                    (*cv).modificationCount = vmcvar.modificationCount;

                    if (*cv).trackChange != QFALSE {
                        let name = CStr::from_ptr((*cv).cvarName).to_str().unwrap_or_default();
                        let value = CStr::from_ptr(vmcvar.string.as_ptr()).to_string_lossy();
                        trap::SendServerCommand(
                            -1,
                            &format!("print \"Server: {name} changed to {value}\n\""),
                        );
                    }

                    if (*cv).teamShader != QFALSE {
                        remapped = QTRUE;
                    }
                }
            }
        }
    }

    if remapped != QFALSE {
        G_RemapTeamShaders();
    }
}

/// `void QDECL Com_Error( int level, const char *error, ... )`.
///
/// The error `level` is accepted for signature fidelity but unused: the C body
/// forwards only the formatted text via `G_Error("%s", text)`, dropping the
/// level. Spelled `_level` here both to mark it unused and because a parameter
/// may not shadow the `level` static (Rust E0530, unlike C). Never returns (the
/// engine `G_ERROR` syscall does not return).
pub fn Com_Error(_level: c_int, error: &str) -> ! {
    trap::Error(error)
}

/// `void QDECL Com_Printf( const char *msg, ... )`.
pub fn Com_Printf(msg: &str) {
    trap::Printf(msg);
}

/// `void QDECL G_Printf( const char *fmt, ... )`.
///
/// The C body is wrapped in `#ifndef FINAL_BUILD`, i.e. it is compiled to a
/// no-op in a `FINAL_BUILD` of the game and otherwise `vsprintf`s into a 1024
/// scratch buffer and calls `trap_Printf`. This crate is the non-`FINAL_BUILD`
/// (developer) build, so it forwards. As with the rest of the printf family the
/// C varargs + `vsprintf` collapse to a pre-rendered `&str` (callers format with
/// Rust's `format!`).
pub fn G_Printf(fmt: &str) {
    trap::Printf(fmt);
}

/// `void QDECL G_Error( const char *fmt, ... )`.
///
/// Renders the message and hands it to the `G_ERROR` trap, which aborts the game
/// and does not return. Distinct from [`Com_Error`], which takes (and drops) an
/// error `level`; `G_Error` takes only the format string.
pub fn G_Error(fmt: &str) -> ! {
    trap::Error(fmt)
}

/// `const char *G_GetStringEdString( char *refSection, char *refName )` (g_main.c:4217).
/// Build a *stringed-reference token* for the client to resolve in its own language.
///
/// The C has the "proper" `trap_SP_GetStringTextString` path commented out (it would
/// mix languages depending on the server's locale); instead it marks the reference
/// with a leading `@@@` and ships `refName` to the client, which scans for the `@@@`
/// indicator and resolves the string itself. `refSection` is therefore ignored here —
/// only `refName` is emitted. The result lives in a 1024-byte `static char text[1024]`
/// and survives until the next call (callers consume it immediately, e.g. inside `va`).
///
/// SAFETY: returns a pointer into module-static storage; valid until the next call.
/// `ref_name` must be a valid NUL-terminated C string.
pub unsafe fn G_GetStringEdString(
    _ref_section: *const c_char,
    ref_name: *const c_char,
) -> *const c_char {
    // static char text[1024]={0};
    static mut TEXT: [c_char; 1024] = [0; 1024];

    // Com_sprintf(text, sizeof(text), "@@@%s", refName);
    Com_sprintf(
        addr_of_mut!(TEXT) as *mut c_char,
        1024,
        format_args!("@@@{}", Sz(ref_name)),
    );
    addr_of!(TEXT) as *const c_char
}

/// `void QDECL G_LogPrintf( const char *fmt, ... )` — write a timestamped line to
/// the game log file, echoing it to the console on a dedicated server.
///
/// The C builds a 7-char `"%3i:%i%i "` time prefix (minutes : tens-of-seconds,
/// seconds) into a 1024 scratch buffer with `Com_sprintf`, then `vsprintf`s the
/// varargs message at `string + 7` (right after the prefix). Here the message is
/// pre-rendered (the printf-family varargs deviation — callers format with
/// `format!`), so the prefix and message concatenate directly; the C `string + 7`
/// (the message without the time prefix) is just `msg`. The console echo prints
/// only the message; the file write gets the whole prefixed line.
///
/// For the normal sub-1000-minute case the prefix is exactly 7 chars, so this is
/// byte-identical to the C; a 4-digit minute count would overrun the C's `+ 7`
/// offset, an edge the varargs deviation already abstracts away. See
/// DEVIATIONS.md.
pub fn G_LogPrintf(msg: &str) {
    // SAFETY: `level` and `g_dedicated` are valid module statics, read via
    // addr_of! (never a `&` to a `static mut` by path).
    unsafe {
        let mut sec = (*addr_of!(level)).time / 1000;

        let min = sec / 60;
        sec -= min * 60;
        let tens = sec / 10;
        sec -= tens * 10;

        // C: Com_sprintf( string, sizeof(string), "%3i:%i%i ", min, tens, sec )
        // then the message at string+7. `%3i` == width-3, space-padded.
        let string = format!("{min:3}:{tens}{sec} {msg}");

        if (*addr_of!(g_dedicated)).integer != 0 {
            G_Printf(msg); // C: G_Printf( "%s", string + 7 )
        }

        if (*addr_of!(level)).logFile == 0 {
            return;
        }

        // C: trap_FS_Write( string, strlen( string ), level.logFile )
        trap::FS_Write(string.as_bytes(), (*addr_of!(level)).logFile);
    }
}

/// `void G_RunThink (gentity_t *ent)` (g_main.c:3566). Run an entity's scheduled `think`
/// callback if its `nextthink` time has arrived (`0 < nextthink <= level.time`), clearing
/// `nextthink` first; a null `think` is tolerated. Either way, while the entity is still in
/// use its ICARUS task manager is pumped one step. No oracle (engine `trap_ICARUS_*` plus the
/// entity's own `think` callback drive every branch).
///
/// The C uses three `goto runicarus` early-exits that all skip the think and fall through to
/// the ICARUS pump; the combined guard `0 < thinktime <= level.time` expresses them directly.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe fn G_RunThink(ent: *mut gentity_t) {
    // C reads `nextthink` (an int) into a float `thinktime`.
    let thinktime = (*ent).nextthink as f32;

    if thinktime > 0.0 && thinktime <= (*addr_of!(level)).time as f32 {
        (*ent).nextthink = 0;
        // C: `if (!ent->think) goto runicarus;` — a null think here is tolerated, not an error.
        if let Some(think) = (*ent).think {
            think(ent);
        }
    }

    if (*ent).inuse != QFALSE {
        trap::ICARUS_MaintainTaskManager((*ent).s.number);
    }
}

/// `int QDECL SortRanks( const void *a, const void *b )` (g_main.c:1647) — the
/// `qsort` comparator that orders `level.sortedClients` for the scoreboard. `a`
/// and `b` point to client-index ints; both index `level.clients[]`. The ordering
/// (highest-ranked first) is: lone power-duelists, then by-score active players,
/// with spectators / connecting / scoreboard clients pushed last.
///
/// No-oracle: the body reads two large mutable globals (`level.clients[]` and
/// `g_gametype`), so a bit-exact harness would need the full client-array
/// scaffolding — the "disproportionate scaffolding" precedent (cf. `SortClients`,
/// which *is* oracle-tested because it is a pure int comparator). Faithful 1:1.
///
/// SAFETY: `a`/`b` are the `const int *` element pointers `qsort`/`trap_*` hand
/// the comparator; the indices they hold are in-range for `level.clients`
/// (populated alongside `level.sortedClients` in `CalculateRanks`).
pub extern "C" fn SortRanks(a: *const c_void, b: *const c_void) -> c_int {
    unsafe {
        let clients = (*addr_of!(level)).clients;
        let ca: *const gclient_t = clients.add(*(a as *const c_int) as usize);
        let cb: *const gclient_t = clients.add(*(b as *const c_int) as usize);

        if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
            // sort single duelists first
            if (*ca).sess.duelTeam == DUELTEAM_LONE && (*ca).sess.sessionTeam != TEAM_SPECTATOR {
                return -1;
            }
            if (*cb).sess.duelTeam == DUELTEAM_LONE && (*cb).sess.sessionTeam != TEAM_SPECTATOR {
                return 1;
            }
            // others will be auto-sorted below but above spectators.
        }

        // sort special clients last
        if (*ca).sess.spectatorState == SPECTATOR_SCOREBOARD || (*ca).sess.spectatorClient < 0 {
            return 1;
        }
        if (*cb).sess.spectatorState == SPECTATOR_SCOREBOARD || (*cb).sess.spectatorClient < 0 {
            return -1;
        }

        // then connecting clients
        if (*ca).pers.connected == CON_CONNECTING {
            return 1;
        }
        if (*cb).pers.connected == CON_CONNECTING {
            return -1;
        }

        // then spectators
        if (*ca).sess.sessionTeam == TEAM_SPECTATOR && (*cb).sess.sessionTeam == TEAM_SPECTATOR {
            if (*ca).sess.spectatorTime < (*cb).sess.spectatorTime {
                return -1;
            }
            if (*ca).sess.spectatorTime > (*cb).sess.spectatorTime {
                return 1;
            }
            return 0;
        }
        if (*ca).sess.sessionTeam == TEAM_SPECTATOR {
            return 1;
        }
        if (*cb).sess.sessionTeam == TEAM_SPECTATOR {
            return -1;
        }

        // then sort by score
        if (*ca).ps.persistant[PERS_SCORE as usize] > (*cb).ps.persistant[PERS_SCORE as usize] {
            return -1;
        }
        if (*ca).ps.persistant[PERS_SCORE as usize] < (*cb).ps.persistant[PERS_SCORE as usize] {
            return 1;
        }
        0
    }
}

/// `void CalculateRanks( void )` (g_main.c:1774) — the scoring keystone. Rebuilds
/// `level.sortedClients` (every non-disconnected client index, sorted by
/// [`SortRanks`]), recomputes the connected/non-spectator/playing/voting client
/// counts (and per-team voting counts, excluding bots), picks the two auto-follow
/// targets, then assigns every connected client's `ps.persistant[PERS_RANK]`
/// (team-order in team games; score-order with the `RANK_TIED_FLAG` tie marker
/// otherwise) and publishes the `CS_SCORES1/2` (and `CS_CLIENT_DUELWINNER`)
/// configstrings. Finally checks the level-end rules and, at intermission / in
/// (power)duel, queues a deferred scoreboard broadcast.
///
/// No-oracle: like [`SortRanks`]/[`LogExit`] the whole body mutates the large
/// `level` global (and reads `g_gametype`/`g_entities`) and ends in trap
/// configstring side effects — the "disproportionate scaffolding" precedent. The
/// deterministic sort/rank core is the qsort over `level.sortedClients` (driven by
/// the already-oracle-validated [`bg_lib::qsort`] machinery) plus the rank
/// assignment; both are exercised through the larger client-array fixtures used by
/// the other `level`-mutating ports. Faithful 1:1.
///
/// `CheckExitRules` (g_main.c:2581) is the `void` level-end gate, now ported (see
/// [`CheckExitRules`]) and invoked directly here.
/// `SendScoreboardMessageToAllClients()` is commented out in the C — the queue
/// (`gQueueScoreMessage`/`Time`) replaces it.
///
/// SAFETY: `level`/`g_gametype`/`g_entities` are valid module statics;
/// `numConnectedClients`/`numPlayingClients` (each <= `maxclients` <= `MAX_CLIENTS`)
/// bound every `sortedClients`/`clients` index, and `g_entities[i]` is in range for
/// `i < maxclients`.
pub fn CalculateRanks() {
    unsafe {
        let lvl = addr_of_mut!(level);

        // int rank, score, newScore; gclient_t *cl; — declared at their use sites
        // below (the score-ranking branch) to satisfy Rust's definite-init.
        //int		nonSpecIndex = -1;

        // int preNumSpec = level.numNonSpectatorClients; — read only by the
        // commented-out GT_DUEL "versus" message block below, so kept as a binding
        // that documents the original but is otherwise unused.
        let _preNumSpec: c_int = (*lvl).numNonSpectatorClients;

        (*lvl).follow1 = -1;
        (*lvl).follow2 = -1;
        (*lvl).numConnectedClients = 0;
        (*lvl).numNonSpectatorClients = 0;
        (*lvl).numPlayingClients = 0;
        (*lvl).numVotingClients = 0; // don't count bots
        for i in 0..TEAM_NUM_TEAMS {
            // numteamVotingClients is sized [2]; only TEAM_RED/TEAM_BLUE are ever
            // incremented below, matching the C (which zeroes [TEAM_NUM_TEAMS] but
            // the array only has the two used slots).
            if (i as usize) < (*lvl).numteamVotingClients.len() {
                (*lvl).numteamVotingClients[i as usize] = 0;
            }
        }
        let clients = (*lvl).clients;
        for i in 0..(*lvl).maxclients {
            if (*clients.add(i as usize)).pers.connected != CON_DISCONNECTED {
                (*lvl).sortedClients[(*lvl).numConnectedClients as usize] = i;
                (*lvl).numConnectedClients += 1;

                if (*clients.add(i as usize)).sess.sessionTeam != TEAM_SPECTATOR
                    || (*addr_of!(g_gametype)).integer == GT_DUEL
                    || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
                {
                    if (*clients.add(i as usize)).sess.sessionTeam != TEAM_SPECTATOR {
                        (*lvl).numNonSpectatorClients += 1;
                        //nonSpecIndex = i;
                    }

                    // decide if this should be auto-followed
                    if (*clients.add(i as usize)).pers.connected == CON_CONNECTED {
                        if (*clients.add(i as usize)).sess.sessionTeam != TEAM_SPECTATOR
                            || (*clients.add(i as usize)).iAmALoser != QFALSE
                        {
                            (*lvl).numPlayingClients += 1;
                        }
                        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize)).r.svFlags & SVF_BOT == 0 {
                            (*lvl).numVotingClients += 1;
                            if (*clients.add(i as usize)).sess.sessionTeam == TEAM_RED {
                                (*lvl).numteamVotingClients[0] += 1;
                            } else if (*clients.add(i as usize)).sess.sessionTeam == TEAM_BLUE {
                                (*lvl).numteamVotingClients[1] += 1;
                            }
                        }
                        if (*lvl).follow1 == -1 {
                            (*lvl).follow1 = i;
                        } else if (*lvl).follow2 == -1 {
                            (*lvl).follow2 = i;
                        }
                    }
                }
            }
        }

        //if (!g_warmup.integer)
        if true {
            (*lvl).warmupTime = 0;
        }

        /*
        if (level.numNonSpectatorClients == 2 && preNumSpec < 2 && nonSpecIndex != -1 && g_gametype.integer == GT_DUEL && !level.warmupTime)
        {
            gentity_t *currentWinner = G_GetDuelWinner(&level.clients[nonSpecIndex]);

            if (currentWinner && currentWinner->client)
            {
                trap_SendServerCommand( -1, va("cp \"%s" S_COLOR_WHITE " %s %s\n\"",
                currentWinner->client->pers.netname, G_GetStringEdString("MP_SVGAME", "VERSUS"), level.clients[nonSpecIndex].pers.netname));
            }
        }
        */
        //NOTE: for now not doing this either. May use later if appropriate.

        bg_lib::qsort(
            (*lvl).sortedClients.as_mut_ptr() as *mut c_void,
            (*lvl).numConnectedClients as usize,
            core::mem::size_of_val(&(*lvl).sortedClients[0]),
            SortRanks,
        );

        // set the rank value for all clients that are connected and not spectators
        if (*addr_of!(g_gametype)).integer >= GT_TEAM {
            // in team games, rank is just the order of the teams, 0=red, 1=blue, 2=tied
            for i in 0..(*lvl).numConnectedClients {
                let cl: *mut gclient_t = clients.add((*lvl).sortedClients[i as usize] as usize);
                if (*lvl).teamScores[TEAM_RED as usize] == (*lvl).teamScores[TEAM_BLUE as usize] {
                    (*cl).ps.persistant[PERS_RANK as usize] = 2;
                } else if (*lvl).teamScores[TEAM_RED as usize]
                    > (*lvl).teamScores[TEAM_BLUE as usize]
                {
                    (*cl).ps.persistant[PERS_RANK as usize] = 0;
                } else {
                    (*cl).ps.persistant[PERS_RANK as usize] = 1;
                }
            }
        } else {
            let mut rank: c_int = -1;
            let mut score: c_int = 0;
            for i in 0..(*lvl).numPlayingClients {
                let cl: *mut gclient_t = clients.add((*lvl).sortedClients[i as usize] as usize);
                let newScore: c_int = (*cl).ps.persistant[PERS_SCORE as usize];
                if i == 0 || newScore != score {
                    rank = i;
                    // assume we aren't tied until the next client is checked
                    (*clients.add((*lvl).sortedClients[i as usize] as usize))
                        .ps
                        .persistant[PERS_RANK as usize] = rank;
                } else {
                    // we are tied with the previous client
                    (*clients.add((*lvl).sortedClients[(i - 1) as usize] as usize))
                        .ps
                        .persistant[PERS_RANK as usize] = rank | RANK_TIED_FLAG;
                    (*clients.add((*lvl).sortedClients[i as usize] as usize))
                        .ps
                        .persistant[PERS_RANK as usize] = rank | RANK_TIED_FLAG;
                }
                score = newScore;
                if (*addr_of!(g_gametype)).integer == GT_SINGLE_PLAYER
                    && (*lvl).numPlayingClients == 1
                {
                    (*clients.add((*lvl).sortedClients[i as usize] as usize))
                        .ps
                        .persistant[PERS_RANK as usize] = rank | RANK_TIED_FLAG;
                }
            }
        }

        // set the CS_SCORES1/2 configstrings, which will be visible to everyone
        // (C uses `va("%i", x)` to render the int; the ported `trap::SetConfigstring`
        // takes a `&str`, so the `va` render is the idiomatic `&format!` here.)
        if (*addr_of!(g_gametype)).integer >= GT_TEAM {
            trap::SetConfigstring(
                CS_SCORES1,
                &format!("{}", (*lvl).teamScores[TEAM_RED as usize]),
            );
            trap::SetConfigstring(
                CS_SCORES2,
                &format!("{}", (*lvl).teamScores[TEAM_BLUE as usize]),
            );
        } else {
            if (*lvl).numConnectedClients == 0 {
                trap::SetConfigstring(CS_SCORES1, &format!("{}", SCORE_NOT_PRESENT));
                trap::SetConfigstring(CS_SCORES2, &format!("{}", SCORE_NOT_PRESENT));
            } else if (*lvl).numConnectedClients == 1 {
                trap::SetConfigstring(
                    CS_SCORES1,
                    &format!(
                        "{}",
                        (*clients.add((*lvl).sortedClients[0] as usize))
                            .ps
                            .persistant[PERS_SCORE as usize]
                    ),
                );
                trap::SetConfigstring(CS_SCORES2, &format!("{}", SCORE_NOT_PRESENT));
            } else {
                trap::SetConfigstring(
                    CS_SCORES1,
                    &format!(
                        "{}",
                        (*clients.add((*lvl).sortedClients[0] as usize))
                            .ps
                            .persistant[PERS_SCORE as usize]
                    ),
                );
                trap::SetConfigstring(
                    CS_SCORES2,
                    &format!(
                        "{}",
                        (*clients.add((*lvl).sortedClients[1] as usize))
                            .ps
                            .persistant[PERS_SCORE as usize]
                    ),
                );
            }

            if (*addr_of!(g_gametype)).integer != GT_DUEL
                || (*addr_of!(g_gametype)).integer != GT_POWERDUEL
            {
                //when not in duel, use this configstring to pass the index of the player currently in first place
                if (*lvl).numConnectedClients >= 1 {
                    trap::SetConfigstring(
                        CS_CLIENT_DUELWINNER,
                        &format!("{}", (*lvl).sortedClients[0]),
                    );
                } else {
                    trap::SetConfigstring(CS_CLIENT_DUELWINNER, "-1");
                }
            }
        }

        // see if it is time to end the level
        CheckExitRules();

        // if we are at the intermission or in multi-frag Duel game mode, send the new info to everyone
        if (*lvl).intermissiontime != 0
            || (*addr_of!(g_gametype)).integer == GT_DUEL
            || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        {
            gQueueScoreMessage = QTRUE;
            gQueueScoreMessageTime = (*lvl).time + 500;
            //SendScoreboardMessageToAllClients();
            //rww - Made this operate on a "queue" system because it was causing large overflows
        }
    }
}

/// `void LogExit( const char *string )` (g_main.c:2244) — queue the intermission
/// and dump the final scoreboard to the game log. Logs the exit reason, stamps
/// `level.intermissionQueued`, raises the `CS_INTERMISSION` configstring (so
/// clients stop playing soon-to-be-cut voice sounds), then logs up to 32 sorted
/// clients' scores/pings (and the team totals in team gametypes), skipping
/// spectators and still-connecting clients. The single-player `spWin`/`spLose`
/// console-command tail is commented out in the C and omitted here too.
///
/// No-oracle: pure logging/configstring side effects over the large mutable
/// `level` global + `g_gametype`; nothing computable to compare bit-exact (same
/// trap/global-read precedent as the rest of this chain). Faithful 1:1.
///
/// SAFETY: `level` and `g_gametype` are valid module statics; `numConnectedClients`
/// (capped at 32) bounds the `sortedClients`/`clients` indexing, populated together
/// in `CalculateRanks`.
pub fn LogExit(string: &str) {
    unsafe {
        let lvl = addr_of_mut!(level);

        G_LogPrintf(&format!("Exit: {string}\n"));

        (*lvl).intermissionQueued = (*lvl).time;

        // this will keep the clients from playing any voice sounds
        // that will get cut off when the queued intermission starts
        trap::SetConfigstring(CS_INTERMISSION, "1");

        // don't send more than 32 scores (FIXME?)
        let mut num_sorted = (*lvl).numConnectedClients;
        if num_sorted > 32 {
            num_sorted = 32;
        }

        if (*addr_of!(g_gametype)).integer >= GT_TEAM {
            G_LogPrintf(&format!(
                "red:{}  blue:{}\n",
                (*lvl).teamScores[TEAM_RED as usize],
                (*lvl).teamScores[TEAM_BLUE as usize]
            ));
        }

        for i in 0..num_sorted {
            let cl = (*lvl)
                .clients
                .add((*lvl).sortedClients[i as usize] as usize);

            if (*cl).sess.sessionTeam == TEAM_SPECTATOR {
                continue;
            }
            if (*cl).pers.connected == CON_CONNECTING {
                continue;
            }

            let ping = if (*cl).ps.ping < 999 {
                (*cl).ps.ping
            } else {
                999
            };

            G_LogPrintf(&format!(
                "score: {}  ping: {}  client: {} {}\n",
                (*cl).ps.persistant[PERS_SCORE as usize],
                ping,
                (*lvl).sortedClients[i as usize],
                Sz((*cl).pers.netname.as_ptr())
            ));
        }

        // The single-player spWin/spLose console-command tail is commented out in C.
    }
}

/// `void MoveClientToIntermission( gentity_t *ent )` (g_main.c:1967).
///
/// When the intermission starts, this will be called for all players.
/// If a new client connects, this will be called after the spawn function.
///
/// No-oracle: takes the client out of follow mode via the entity-mutating
/// `StopFollowing`, snaps the entity/player state to `level.intermission_origin`,
/// and zeroes the powerup/event/render fields — pure side-effecting state mutation
/// over a `gentity_t`/`gclient_t`, no computable return, same precedent as the
/// other intermission helpers in this file. Faithful 1:1.
///
/// SAFETY: `ent` is a valid client entity (caller passes a connected client whose
/// `client` pointer is wired by `G_InitGame`); `level` is a valid module static.
pub unsafe fn MoveClientToIntermission(ent: *mut gentity_t) {
    // take out of follow mode if needed
    if (*(*ent).client).sess.spectatorState == SPECTATOR_FOLLOW {
        StopFollowing(ent);
    }

    // move to the spot
    VectorCopy(
        &(*addr_of!(level)).intermission_origin,
        &mut (*ent).s.origin,
    );
    VectorCopy(
        &(*addr_of!(level)).intermission_origin,
        &mut (*(*ent).client).ps.origin,
    );
    VectorCopy(
        &(*addr_of!(level)).intermission_angle,
        &mut (*(*ent).client).ps.viewangles,
    );
    (*(*ent).client).ps.pm_type = PM_INTERMISSION;

    // clean up powerup info
    (*(*ent).client).ps.powerups = [0; crate::codemp::game::q_shared_h::MAX_POWERUPS];

    (*(*ent).client).ps.eFlags = 0;
    (*ent).s.eFlags = 0;
    (*ent).s.eType = ET_GENERAL;
    (*ent).s.modelindex = 0;
    (*ent).s.loopSound = 0;
    (*ent).s.loopIsSoundset = QFALSE;
    (*ent).s.event = 0;
    (*ent).r.contents = 0;
}

/// `void FindIntermissionPoint( void )` (g_main.c:2000) — locate the camera spot
/// the scoreboard/intermission view (and spectator spawns) use. In Siege, once the
/// round has ended it first looks for the winning team's
/// `info_player_intermission_red`/`_blue` and fires its `target2`. Otherwise (or as
/// fallback) it prefers an
/// `info_player_intermission` entity: copies its origin/angles into
/// `level.intermission_origin`/`intermission_angle`, and if it has a `target`,
/// re-aims the angles toward that target. If the mapper left no intermission
/// entity, falls back to a regular spawn point via `SelectSpawnPoint`.
///
/// No-oracle: walks the `g_entities` array and the `level` global through the
/// engine-backed `G_Find`/`G_PickTarget`/`SelectSpawnPoint` traps. Faithful 1:1.
///
/// SAFETY: `level` and `g_entities` are valid module statics; the field offsets
/// and pointers handed to `G_Find`/`SelectSpawnPoint` follow the same patterns the
/// rest of the spawn code uses.
pub fn FindIntermissionPoint() {
    unsafe {
        let lvl = addr_of_mut!(level);

        let mut ent: *mut gentity_t = null_mut();

        // find the intermission spot
        if (*addr_of!(g_gametype)).integer == GT_SIEGE
            && (*lvl).intermissiontime != 0
            && (*lvl).intermissiontime <= (*lvl).time
            && *addr_of!(gSiegeRoundEnded) != QFALSE
        {
            if *addr_of!(gSiegeRoundWinningTeam) == SIEGETEAM_TEAM1 {
                ent = G_Find(
                    null_mut(),
                    core::mem::offset_of!(gentity_t, classname),
                    c"info_player_intermission_red".as_ptr(),
                );
                if !ent.is_null() && !(*ent).target2.is_null() {
                    G_UseTargets2(ent, ent, (*ent).target2);
                }
            } else if *addr_of!(gSiegeRoundWinningTeam) == SIEGETEAM_TEAM2 {
                ent = G_Find(
                    null_mut(),
                    core::mem::offset_of!(gentity_t, classname),
                    c"info_player_intermission_blue".as_ptr(),
                );
                if !ent.is_null() && !(*ent).target2.is_null() {
                    G_UseTargets2(ent, ent, (*ent).target2);
                }
            }
        }
        if ent.is_null() {
            ent = G_Find(
                null_mut(),
                core::mem::offset_of!(gentity_t, classname),
                c"info_player_intermission".as_ptr(),
            );
        }
        if ent.is_null() {
            // the map creator forgot to put in an intermission point...
            SelectSpawnPoint(
                &vec3_origin,
                &mut (*lvl).intermission_origin,
                &mut (*lvl).intermission_angle,
                TEAM_SPECTATOR,
            );
        } else {
            VectorCopy(&(*ent).s.origin, &mut (*lvl).intermission_origin);
            VectorCopy(&(*ent).s.angles, &mut (*lvl).intermission_angle);
            // if it has a target, look towards it
            if !(*ent).target.is_null() {
                let target = G_PickTarget((*ent).target);
                if !target.is_null() {
                    let mut dir = vec3_origin;
                    VectorSubtract(&(*target).s.origin, &(*lvl).intermission_origin, &mut dir);
                    vectoangles(&dir, &mut (*lvl).intermission_angle);
                }
            }
        }
    }
}

/// `void BeginIntermission( void )` (g_main.c:2030) — start the end-of-level intermission.
/// No-op if already active. In duel/powerduel it resets the duel-winner configstring,
/// applies [`AdjustTournamentScores`] (non-powerduel only) and latches `gDuelExit` from
/// [`DuelLimitHit`]. Then it stamps `level.intermissiontime`, picks the intermission spot
/// ([`FindIntermissionPoint`]), respawns dead clients ([`respawn`]) and moves every in-use
/// client to the spot ([`MoveClientToIntermission`]), and finally pushes the scoreboard to
/// everyone ([`SendScoreboardMessageToAllClients`]). The commented-out single-player
/// `UpdateTournamentInfo` block is preserved verbatim, matching the retail "I don't want
/// this to happen" note.
///
/// No-oracle: mutates the large `level`/`g_entities` globals + emits configstrings/scoreboard
/// side effects. Faithful 1:1.
///
/// SAFETY: `level`, `g_entities`, `g_gametype` are valid module statics; `client` indexes
/// `g_entities` within `level.maxclients`.
pub fn BeginIntermission() {
    unsafe {
        let lvl = addr_of_mut!(level);

        if (*lvl).intermissiontime != 0 {
            return; // already active
        }

        // if in tournement mode, change the wins / losses
        if (*addr_of!(g_gametype)).integer == GT_DUEL
            || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        {
            trap::SetConfigstring(CS_CLIENT_DUELWINNER, "-1");

            if (*addr_of!(g_gametype)).integer != GT_POWERDUEL {
                AdjustTournamentScores();
            }
            if DuelLimitHit() == QTRUE {
                gDuelExit = QTRUE;
            } else {
                gDuelExit = QFALSE;
            }
        }

        (*lvl).intermissiontime = (*lvl).time;
        FindIntermissionPoint();

        /*
        if (g_singlePlayer.integer) {
            trap_Cvar_Set("ui_singlePlayerActive", "0");
            UpdateTournamentInfo();
        }
        */
        //what the? Well, I don't want this to happen.

        // move all clients to the intermission point
        for i in 0..(*lvl).maxclients {
            let client = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
            if (*client).inuse != QTRUE {
                continue;
            }
            // respawn if dead
            if (*client).health <= 0 {
                if (*addr_of!(g_gametype)).integer != GT_POWERDUEL
                    || (*client).client.is_null()
                    || (*(*client).client).sess.sessionTeam != TEAM_SPECTATOR
                {
                    //don't respawn spectators in powerduel or it will mess the line order all up
                    respawn(client);
                }
            }
            MoveClientToIntermission(client);
        }

        // send the current scoring to all clients
        SendScoreboardMessageToAllClients();
    }
}

/// `qboolean DuelLimitHit( void )` (g_main.c:2089) — has any connected client
/// reached the duel frag limit? Scans `level.clients[0..g_maxclients]`, skipping
/// the unconnected; returns true the moment one has `sess.wins >=
/// g_duel_fraglimit` (when that cvar is non-zero). Gates the duel exit / map
/// restart logic in `BeginIntermission`/`ExitLevel`/`CheckIntermissionExit`.
///
/// No-oracle: reads the large mutable `level` global plus the `g_maxclients` /
/// `g_duel_fraglimit` cvars. Faithful 1:1.
///
/// SAFETY: `level`, `g_maxclients`, `g_duel_fraglimit` are valid module statics;
/// `clients` is the engine-allocated array indexed within `g_maxclients`.
pub fn DuelLimitHit() -> qboolean {
    unsafe {
        let lvl = addr_of!(level);
        let maxclients = (*addr_of!(g_maxclients)).integer;
        let fraglimit = (*addr_of!(g_duel_fraglimit)).integer;

        for i in 0..maxclients {
            let cl = (*lvl).clients.add(i as usize);
            if (*cl).pers.connected != CON_CONNECTED {
                continue;
            }

            if fraglimit != 0 && (*cl).sess.wins >= fraglimit {
                return QTRUE;
            }
        }

        QFALSE
    }
}

/// `void DuelResetWinsLosses( void )` (g_main.c:2109) — zero the `sess.wins` /
/// `sess.losses` of every connected client. Called by `ExitLevel` once the duel
/// frag limit has been hit and the map is about to change.
///
/// No-oracle: mutates the large `level` global. Faithful 1:1.
///
/// SAFETY: `level` and `g_maxclients` are valid module statics; `clients` is the
/// engine-allocated array indexed within `g_maxclients`.
pub fn DuelResetWinsLosses() {
    unsafe {
        let lvl = addr_of_mut!(level);
        let maxclients = (*addr_of!(g_maxclients)).integer;

        for i in 0..maxclients {
            let cl = (*lvl).clients.add(i as usize);
            if (*cl).pers.connected != CON_CONNECTED {
                continue;
            }

            (*cl).sess.wins = 0;
            (*cl).sess.losses = 0;
        }
    }
}

/// `qboolean g_noPDuelCheck = qfalse;` (g_main.c:1740) — file-global flag set true while
/// [`G_ResetDuelists`] re-suicides the three power-duelists, so the power-duel bookkeeping in
/// `player_die` is suppressed for those scripted deaths (it would otherwise treat them as a real
/// round loss). Module-internal file-scope global; mirrored as a zero-initialised mutable static.
pub static mut g_noPDuelCheck: qboolean = QFALSE;

/// `void G_ResetDuelists( void )` (g_main.c:1741) — respawn the top three `sortedClients`
/// power-duelists in place: suicide each (with [`g_noPDuelCheck`] raised so `player_die` skips the
/// power-duel round bookkeeping), unlink, [`ClientSpawn`] it, then emit a teleport-in effect entity
/// at the respawned origin. Called once [`G_CanResetDuelists`] confirms all three are respawnable.
///
/// No-oracle: mutates the `g_entities` / `level` globals, calls `player_die`/`ClientSpawn` and the
/// `trap_UnlinkEntity` / `G_TempEntity` engine seams. Faithful 1:1.
///
/// SAFETY: `level` and `g_entities` are valid module statics; the top three `sortedClients` indices
/// are populated by `CalculateRanks` and verified live by `G_CanResetDuelists` before this runs, so
/// indexing `g_entities[sortedClients[0..3]]` and the `ent->client` deref are in range.
pub fn G_ResetDuelists() {
    unsafe {
        let lvl = addr_of!(level);

        let mut i: usize = 0;
        while i < 3 {
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*lvl).sortedClients[i] as usize);

            *addr_of_mut!(g_noPDuelCheck) = QTRUE;
            player_die(ent, ent, ent, 999, MOD_SUICIDE);
            *addr_of_mut!(g_noPDuelCheck) = QFALSE;
            trap::UnlinkEntity(ent);
            ClientSpawn(ent);

            // add a teleportation effect
            let tent = G_TempEntity(&(*(*ent).client).ps.origin, EV_PLAYER_TELEPORT_IN);
            (*tent).s.clientNum = (*ent).s.clientNum;
            i += 1;
        }
    }
}

/// `void ExitLevel( void )` (g_main.c:2136) — when the intermission has been exited, kill or
/// advance the server based on the `nextmap` cvar. In duel/powerduel, if the frag limit was NOT
/// hit it kicks off a `map_restart 0` and bails (the loser is rotated to spectator so the next
/// challenger restarts the round); otherwise it clears the wins/losses. For a Siege map with the
/// team-switch cvar set and a beaten time, it restarts the same map (else `vstr nextmap`), then —
/// when team-switch is on — reassigns teams immediately via [`SiegeDoTeamAssign`]. Finally it zeroes
/// the team scores and every connected client's `PERS_SCORE`, persists the session data with
/// [`G_WriteSessionData`], and flips every still-`CON_CONNECTED` client to `CON_CONNECTING` so early
/// arrivers into the next level know the others are still reconnecting.
///
/// No-oracle: reads/writes the large `level` global, fires the `trap_SendConsoleCommand` engine
/// seam and writes session data. Faithful 1:1; the Siege field reads (`g_siegePersistant.beatingTime`)
/// are reads on an already-ported global, not the deferred Siege bulk.
///
/// SAFETY: `level`, `g_gametype`, `g_maxclients`, `g_siegeTeamSwitch` and `g_siegePersistant` are
/// valid module statics; `level.clients + i` is in range for `i < g_maxclients`.
pub fn ExitLevel() {
    unsafe {
        let lvl = addr_of_mut!(level);
        let gametype = (*addr_of!(g_gametype)).integer;

        // if we are running a tournement map, kick the loser to spectator status,
        // which will automatically grab the next spectator and restart
        if gametype == GT_DUEL || gametype == GT_POWERDUEL {
            if DuelLimitHit() == QFALSE {
                if (*lvl).restarted == QFALSE {
                    trap::SendConsoleCommand(EXEC_APPEND, "map_restart 0\n");
                    (*lvl).restarted = QTRUE;
                    (*lvl).changemap = null_mut();
                    (*lvl).intermissiontime = 0;
                }
                return;
            }

            DuelResetWinsLosses();
        }

        if gametype == GT_SIEGE
            && (*addr_of!(g_siegeTeamSwitch)).integer != 0
            && (*addr_of!(g_siegePersistant)).beatingTime != QFALSE
        {
            //restart same map...
            trap::SendConsoleCommand(EXEC_APPEND, "map_restart 0\n");
        } else {
            trap::SendConsoleCommand(EXEC_APPEND, "vstr nextmap\n");
        }
        (*lvl).changemap = null_mut();
        (*lvl).intermissiontime = 0;

        if gametype == GT_SIEGE && (*addr_of!(g_siegeTeamSwitch)).integer != 0 {
            //switch out now
            SiegeDoTeamAssign();
        }

        // reset all the scores so we don't enter the intermission again
        (*lvl).teamScores[TEAM_RED as usize] = 0;
        (*lvl).teamScores[TEAM_BLUE as usize] = 0;
        let maxclients = (*addr_of!(g_maxclients)).integer;
        for i in 0..maxclients {
            let cl = (*lvl).clients.add(i as usize);
            if (*cl).pers.connected != CON_CONNECTED {
                continue;
            }
            (*cl).ps.persistant[PERS_SCORE as usize] = 0;
        }

        // we need to do this here before chaning to CON_CONNECTING
        G_WriteSessionData();

        // change all client states to connecting, so the early players into the
        // next level will know the others aren't done reconnecting
        for i in 0..maxclients {
            let cl = (*lvl).clients.add(i as usize);
            if (*cl).pers.connected == CON_CONNECTED {
                (*cl).pers.connected = CON_CONNECTING;
            }
        }
    }
}

/// `void CheckIntermissionExit( void )` (g_main.c:2310) — once intermission is active, decide
/// when to leave the level. Counts ready / not-ready non-bot connected clients (building a
/// `readyMask` of the first 16). In duel/powerduel it then does the one-shot post-round
/// bookkeeping (`gDidDuelStuff`, 2s after intermission starts): austrian-mode result logging
/// and — unless the duel frag limit was hit — rotating the loser out / next challenger in and
/// republishing the duelist/winner configstrings. Outside the duel between-round path it copies
/// `readyMask` into every client's `STAT_CLIENTS_READY` and exits the level once the timing /
/// ready-vote rules are satisfied.
///
/// The tournament-removal cluster (`RemovePowerDuelLosers` / `AddPowerDuelPlayers` /
/// `RemoveDuelDrawLoser` / `RemoveTournamentLoser` / `AddTournamentPlayer`) is now LIVE — those
/// callees landed once the `SetTeam` keystone (h187) was ported, so the duel-rotation control
/// flow runs in full.
///
/// [`ExitLevel`] (g_main.c:2136) is now LIVE — its `SiegeDoTeamAssign` / `G_WriteSessionData`
/// callees landed, so the four exit paths advance/restart the level in full.
///
/// No-oracle: reads/writes the large `level` / `g_entities` globals + emits log / configstring
/// side effects. Faithful 1:1; the austrian-mode `G_LogPrintf` lines are preserved verbatim.
///
/// SAFETY: `level`, `g_entities`, `g_gametype` and the cvars are valid module statics; the
/// `sortedClients` indices are populated alongside `numPlayingClients` in `CalculateRanks`.
pub fn CheckIntermissionExit() {
    unsafe {
        let lvl = addr_of_mut!(level);
        let maxclients = (*addr_of!(g_maxclients)).integer;
        let gametype = (*addr_of!(g_gametype)).integer;

        // see which players are ready
        let mut ready: c_int = 0;
        let mut not_ready: c_int = 0;
        let mut ready_mask: c_int = 0;
        for i in 0..maxclients {
            let cl = (*lvl).clients.add(i as usize);
            if (*cl).pers.connected != CON_CONNECTED {
                continue;
            }
            if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*cl).ps.clientNum as usize))
                .r
                .svFlags
                & SVF_BOT
                != 0
            {
                continue;
            }

            if (*cl).readyToExit == QTRUE {
                ready += 1;
                if i < 16 {
                    ready_mask |= 1 << i;
                }
            } else {
                not_ready += 1;
            }
        }

        if (gametype == GT_DUEL || gametype == GT_POWERDUEL)
            && gDidDuelStuff == QFALSE
            && (*lvl).time > (*lvl).intermissiontime + 2000
        {
            gDidDuelStuff = QTRUE;

            if (*addr_of!(g_austrian)).integer != 0 && gametype != GT_POWERDUEL {
                let c0 = (*lvl).clients.add((*lvl).sortedClients[0] as usize);
                let c1 = (*lvl).clients.add((*lvl).sortedClients[1] as usize);
                G_LogPrintf("Duel Results:\n");
                //G_LogPrintf("Duel Time: %d\n", level.time );
                G_LogPrintf(&format!(
                    "winner: {}, score: {}, wins/losses: {}/{}\n",
                    Sz((*c0).pers.netname.as_ptr()),
                    (*c0).ps.persistant[PERS_SCORE as usize],
                    (*c0).sess.wins,
                    (*c0).sess.losses
                ));
                G_LogPrintf(&format!(
                    "loser: {}, score: {}, wins/losses: {}/{}\n",
                    Sz((*c1).pers.netname.as_ptr()),
                    (*c1).ps.persistant[PERS_SCORE as usize],
                    (*c1).sess.wins,
                    (*c1).sess.losses
                ));
            }
            // if we are running a tournement map, kick the loser to spectator status,
            // which will automatically grab the next spectator and restart
            if DuelLimitHit() == QFALSE {
                if gametype == GT_POWERDUEL {
                    RemovePowerDuelLosers();
                    AddPowerDuelPlayers();
                } else {
                    let c0 = (*lvl).clients.add((*lvl).sortedClients[0] as usize);
                    let c1 = (*lvl).clients.add((*lvl).sortedClients[1] as usize);
                    if (*c0).ps.persistant[PERS_SCORE as usize]
                        == (*c1).ps.persistant[PERS_SCORE as usize]
                        && (*c0).pers.connected == CON_CONNECTED
                        && (*c1).pers.connected == CON_CONNECTED
                    {
                        RemoveDuelDrawLoser();
                    } else {
                        RemoveTournamentLoser();
                    }
                    AddTournamentPlayer();
                }

                if (*addr_of!(g_austrian)).integer != 0 {
                    if gametype == GT_POWERDUEL {
                        let c0 = (*lvl).clients.add((*lvl).sortedClients[0] as usize);
                        let c1 = (*lvl).clients.add((*lvl).sortedClients[1] as usize);
                        let c2 = (*lvl).clients.add((*lvl).sortedClients[2] as usize);
                        G_LogPrintf(&format!(
                            "Power Duel Initiated: {} {}/{} vs {} {}/{} and {} {}/{}, kill limit: {}\n",
                            Sz((*c0).pers.netname.as_ptr()),
                            (*c0).sess.wins,
                            (*c0).sess.losses,
                            Sz((*c1).pers.netname.as_ptr()),
                            (*c1).sess.wins,
                            (*c1).sess.losses,
                            Sz((*c2).pers.netname.as_ptr()),
                            (*c2).sess.wins,
                            (*c2).sess.losses,
                            (*addr_of!(g_fraglimit)).integer
                        ));
                    } else {
                        let c0 = (*lvl).clients.add((*lvl).sortedClients[0] as usize);
                        let c1 = (*lvl).clients.add((*lvl).sortedClients[1] as usize);
                        G_LogPrintf(&format!(
                            "Duel Initiated: {} {}/{} vs {} {}/{}, kill limit: {}\n",
                            Sz((*c0).pers.netname.as_ptr()),
                            (*c0).sess.wins,
                            (*c0).sess.losses,
                            Sz((*c1).pers.netname.as_ptr()),
                            (*c1).sess.wins,
                            (*c1).sess.losses,
                            (*addr_of!(g_fraglimit)).integer
                        ));
                    }
                }

                if gametype == GT_POWERDUEL {
                    if (*lvl).numPlayingClients >= 3 && (*lvl).numNonSpectatorClients >= 3 {
                        trap::SetConfigstring(
                            CS_CLIENT_DUELISTS,
                            &format!(
                                "{}|{}|{}",
                                (*lvl).sortedClients[0],
                                (*lvl).sortedClients[1],
                                (*lvl).sortedClients[2]
                            ),
                        );
                        trap::SetConfigstring(CS_CLIENT_DUELWINNER, "-1");
                    }
                } else {
                    if (*lvl).numPlayingClients >= 2 {
                        trap::SetConfigstring(
                            CS_CLIENT_DUELISTS,
                            &format!("{}|{}", (*lvl).sortedClients[0], (*lvl).sortedClients[1]),
                        );
                        trap::SetConfigstring(CS_CLIENT_DUELWINNER, "-1");
                    }
                }

                return;
            }

            if (*addr_of!(g_austrian)).integer != 0 && gametype != GT_POWERDUEL {
                let c0 = (*lvl).clients.add((*lvl).sortedClients[0] as usize);
                G_LogPrintf(&format!(
                    "Duel Tournament Winner: {} wins/losses: {}/{}\n",
                    Sz((*c0).pers.netname.as_ptr()),
                    (*c0).sess.wins,
                    (*c0).sess.losses
                ));
            }

            if gametype == GT_POWERDUEL {
                RemovePowerDuelLosers();
                AddPowerDuelPlayers();

                if (*lvl).numPlayingClients >= 3 && (*lvl).numNonSpectatorClients >= 3 {
                    trap::SetConfigstring(
                        CS_CLIENT_DUELISTS,
                        &format!(
                            "{}|{}|{}",
                            (*lvl).sortedClients[0],
                            (*lvl).sortedClients[1],
                            (*lvl).sortedClients[2]
                        ),
                    );
                    trap::SetConfigstring(CS_CLIENT_DUELWINNER, "-1");
                }
            } else {
                //this means we hit the duel limit so reset the wins/losses
                //but still push the loser to the back of the line, and retain the order for
                //the map change
                let c0 = (*lvl).clients.add((*lvl).sortedClients[0] as usize);
                let c1 = (*lvl).clients.add((*lvl).sortedClients[1] as usize);
                if (*c0).ps.persistant[PERS_SCORE as usize]
                    == (*c1).ps.persistant[PERS_SCORE as usize]
                    && (*c0).pers.connected == CON_CONNECTED
                    && (*c1).pers.connected == CON_CONNECTED
                {
                    RemoveDuelDrawLoser();
                } else {
                    RemoveTournamentLoser();
                }

                AddTournamentPlayer();

                if (*lvl).numPlayingClients >= 2 {
                    trap::SetConfigstring(
                        CS_CLIENT_DUELISTS,
                        &format!("{}|{}", (*lvl).sortedClients[0], (*lvl).sortedClients[1]),
                    );
                    trap::SetConfigstring(CS_CLIENT_DUELWINNER, "-1");
                }
            }
        }

        if (gametype == GT_DUEL || gametype == GT_POWERDUEL) && gDuelExit == QFALSE {
            //in duel, we have different behaviour for between-round intermissions
            if (*lvl).time > (*lvl).intermissiontime + 4000 {
                //automatically go to next after 4 seconds
                ExitLevel();
                return;
            }

            for i in 0..maxclients {
                //being in a "ready" state is not necessary here, so clear it for everyone
                //yes, I also thinking holding this in a ps value uniquely for each player
                //is bad and wrong, but it wasn't my idea.
                let cl = (*lvl).clients.add(i as usize);
                if (*cl).pers.connected != CON_CONNECTED {
                    continue;
                }
                (*cl).ps.stats[STAT_CLIENTS_READY as usize] = 0;
            }
            return;
        }

        // copy the readyMask to each player's stats so
        // it can be displayed on the scoreboard
        for i in 0..maxclients {
            let cl = (*lvl).clients.add(i as usize);
            if (*cl).pers.connected != CON_CONNECTED {
                continue;
            }
            (*cl).ps.stats[STAT_CLIENTS_READY as usize] = ready_mask;
        }

        // never exit in less than five seconds
        if (*lvl).time < (*lvl).intermissiontime + 5000 {
            return;
        }

        if (*addr_of!(d_noIntermissionWait)).integer != 0 {
            //don't care who wants to go, just go.
            ExitLevel();
            return;
        }

        // if nobody wants to go, clear timer
        if ready == 0 {
            (*lvl).readyToExit = QFALSE;
            return;
        }

        // if everyone wants to go, go now
        if not_ready == 0 {
            ExitLevel();
            return;
        }

        // the first person to ready starts the ten second timeout
        if (*lvl).readyToExit == QFALSE {
            (*lvl).readyToExit = QTRUE;
            (*lvl).exitTime = (*lvl).time;
        }

        // if we have waited ten seconds since at least one player
        // wanted to exit, go ahead
        if (*lvl).time < (*lvl).exitTime + 10000 {
            return;
        }

        ExitLevel();
    }
}

/// `qboolean ScoreIsTied( void )` (g_main.c:2554) — is the top of the scoreboard
/// a tie? Fewer than two playing clients is never a tie; in team gametypes the red
/// and blue team scores decide it; otherwise the top two `sortedClients` scores
/// are compared.
///
/// No-oracle: reads the large mutable `level` global (`numPlayingClients`,
/// `teamScores`, `sortedClients`, `clients[].ps`) plus `g_gametype` — same
/// disproportionate-scaffolding precedent as `SortRanks`. Faithful 1:1.
///
/// SAFETY: `level` and `g_gametype` are valid module statics; the `sortedClients`
/// indices are populated alongside `numPlayingClients` in `CalculateRanks`, so
/// indices `[0]`/`[1]` are valid whenever `numPlayingClients >= 2`.
pub fn ScoreIsTied() -> qboolean {
    unsafe {
        let lvl = addr_of!(level);

        if (*lvl).numPlayingClients < 2 {
            return QFALSE;
        }

        if (*addr_of!(g_gametype)).integer >= GT_TEAM {
            return ((*lvl).teamScores[TEAM_RED as usize] == (*lvl).teamScores[TEAM_BLUE as usize])
                as qboolean;
        }

        let clients = (*lvl).clients;
        let a = (*clients.add((*lvl).sortedClients[0] as usize))
            .ps
            .persistant[PERS_SCORE as usize];
        let b = (*clients.add((*lvl).sortedClients[1] as usize))
            .ps
            .persistant[PERS_SCORE as usize];

        (a == b) as qboolean
    }
}

/// `void CheckExitRules( void )` (g_main.c:2581) — the per-frame level-end gate.
/// "There will be a delay between the time the exit is qualified for and the time
/// everyone is moved to the intermission spot, so you can see the last frag."
///
/// Faithful 1:1 of the C control flow: if already at intermission, defer to
/// `CheckIntermissionExit`; bail while a slow-mo duel is playing; handle the
/// `gEscaping` "escape" round end-conditions (timer elapsed / everyone dead); fire
/// the queued intermission once `INTERMISSION_DELAY_TIME` has passed; then check the
/// sudden-death tie, the timelimit, the power-duel end latch (`g_endPDuel`), and the
/// frag/duel-win/capture limits, calling [`LogExit`] for whichever fires first.
///
/// Two large `/* ... */` commented-out blocks in the C — the early `GT_POWERDUEL`
/// "Duel forfeit" win-condition (g_main.c:2640-2673, which still carries the raw
/// `<<<<<<<`/`=======`/`>>>>>>>` CVS merge markers) and the "completely insane"
/// per-duelist win/loss bookkeeping (g_main.c:2715-2816) — are preserved verbatim as
/// Rust block comments, exactly as the rest of this file carries over dead C.
///
/// The two intermission callees [`CheckIntermissionExit`] (at the top) and
/// [`BeginIntermission`] (in the `intermissionQueued` block) are wired to their
/// in-file ports; neither returns a value that steers control flow here.
///
/// No oracle: `void` + side-effecting via [`LogExit`]/[`trap::SendServerCommand`] over
/// the large mutable `level` global and the duel/escape statics — the same
/// disproportionate-scaffolding precedent as the other `level`-mutating ports.
///
/// SAFETY: `level`, `g_entities`, `g_gametype` and the cvar/duel/escape statics are
/// valid module statics; `g_entities[i]`/`level.clients[i]` are in range for
/// `i < MAX_CLIENTS` / `i < g_maxclients`, and every in-use client entity's `client`
/// back-pointer is set by `G_InitGame`.
pub fn CheckExitRules() {
    unsafe {
        let mut printLimit: qboolean = QTRUE;
        // if at the intermission, wait for all non-bots to
        // signal ready, then go to next level
        if (*addr_of!(level)).intermissiontime != 0 {
            CheckIntermissionExit();
            return;
        }

        if gDoSlowMoDuel != QFALSE {
            //don't go to intermission while in slow motion
            return;
        }

        if gEscaping != QFALSE {
            let mut i: c_int = 0;
            let mut numLiveClients: c_int = 0;

            while i < MAX_CLIENTS as c_int {
                let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
                if (*ent).inuse != QFALSE && !(*ent).client.is_null() && (*ent).health > 0 {
                    let cl = (*ent).client;
                    if (*cl).sess.sessionTeam != TEAM_SPECTATOR
                        && ((*cl).ps.pm_flags & PMF_FOLLOW) == 0
                    {
                        numLiveClients += 1;
                    }
                }

                i += 1;
            }
            if gEscapeTime < (*addr_of!(level)).time {
                gEscaping = QFALSE;
                LogExit("Escape time ended.");
                return;
            }
            if numLiveClients == 0 {
                gEscaping = QFALSE;
                LogExit("Everyone failed to escape.");
                return;
            }
        }

        if (*addr_of!(level)).intermissionQueued != 0 {
            //int time = (g_singlePlayer.integer) ? SP_INTERMISSION_DELAY_TIME : INTERMISSION_DELAY_TIME;
            let time: c_int = INTERMISSION_DELAY_TIME;
            if (*addr_of!(level)).time - (*addr_of!(level)).intermissionQueued >= time {
                (*addr_of_mut!(level)).intermissionQueued = 0;
                BeginIntermission();
            }
            return;
        }

        /*
        if (g_gametype.integer == GT_POWERDUEL)
        {
            if (level.numPlayingClients < 3)
            {
        <<<<<<< g_main.c
                if ((gDuelists[0] == gDuelists[1] && gDuelists[0] != -1) ||
                    (gDuelists[0] == gDuelists[2] && gDuelists[0] != -1) ||
                    (gDuelists[1] == gDuelists[2] && gDuelists[1] != -1))
                {
                    LogExit("Duel forfeit.");
                    return;
                }
                else
        =======
                if (!level.intermissiontime)
        >>>>>>> 1.16
                {
        <<<<<<< g_main.c
                    int x = 0;
                    gentity_t *duelist;
                    while (x < 3)
        =======
                    if (d_powerDuelPrint.integer)
        >>>>>>> 1.16
                    {
                        Com_Printf("POWERDUEL WIN CONDITION: Duel forfeit (1)\n");
                    }
                    LogExit("Duel forfeit.");
                    return;
                }
            }
        }
        */

        // check for sudden death
        if (*addr_of!(g_gametype)).integer != GT_SIEGE {
            if ScoreIsTied() != QFALSE {
                // always wait for sudden death
                if ((*addr_of!(g_gametype)).integer != GT_DUEL)
                    || (*addr_of!(g_timelimit)).integer == 0
                {
                    if (*addr_of!(g_gametype)).integer != GT_POWERDUEL {
                        return;
                    }
                }
            }
        }

        if (*addr_of!(g_gametype)).integer != GT_SIEGE {
            if (*addr_of!(g_timelimit)).integer != 0 && (*addr_of!(level)).warmupTime == 0 {
                if (*addr_of!(level)).time - (*addr_of!(level)).startTime
                    >= (*addr_of!(g_timelimit)).integer * 60000
                {
                    //				trap_SendServerCommand( -1, "print \"Timelimit hit.\n\"");
                    trap::SendServerCommand(
                        -1,
                        &CStr::from_ptr(va(format_args!(
                            "print \"{}.\n\"",
                            Sz(G_GetStringEdString(
                                c"MP_SVGAME".as_ptr(),
                                c"TIMELIMIT_HIT".as_ptr()
                            ))
                        )))
                        .to_string_lossy(),
                    );
                    if (*addr_of!(d_powerDuelPrint)).integer != 0 {
                        Com_Printf("POWERDUEL WIN CONDITION: Timelimit hit (1)\n");
                    }
                    LogExit("Timelimit hit.");
                    return;
                }
            }
        }

        if (*addr_of!(g_gametype)).integer == GT_POWERDUEL
            && (*addr_of!(level)).numPlayingClients >= 3
        {
            if g_endPDuel != QFALSE {
                g_endPDuel = QFALSE;
                LogExit("Powerduel ended.");
            }

            //yeah, this stuff was completely insane.
            /*
            int duelists[3];
            duelists[0] = level.sortedClients[0];
            duelists[1] = level.sortedClients[1];
            duelists[2] = level.sortedClients[2];

            if (duelists[0] != -1 &&
                duelists[1] != -1 &&
                duelists[2] != -1)
            {
                if (!g_entities[duelists[0]].inuse ||
                    !g_entities[duelists[0]].client ||
                    g_entities[duelists[0]].client->ps.stats[STAT_HEALTH] <= 0 ||
                    g_entities[duelists[0]].client->sess.sessionTeam != TEAM_FREE)
                { //The lone duelist lost, give the other two wins (if applicable) and him a loss
                    if (g_entities[duelists[0]].inuse &&
                        g_entities[duelists[0]].client)
                    {
                        g_entities[duelists[0]].client->sess.losses++;
                        ClientUserinfoChanged(duelists[0]);
                    }
                    if (g_entities[duelists[1]].inuse &&
                        g_entities[duelists[1]].client)
                    {
                        if (g_entities[duelists[1]].client->ps.stats[STAT_HEALTH] > 0 &&
                            g_entities[duelists[1]].client->sess.sessionTeam == TEAM_FREE)
                        {
                            g_entities[duelists[1]].client->sess.wins++;
                        }
                        else
                        {
                            g_entities[duelists[1]].client->sess.losses++;
                        }
                        ClientUserinfoChanged(duelists[1]);
                    }
                    if (g_entities[duelists[2]].inuse &&
                        g_entities[duelists[2]].client)
                    {
                        if (g_entities[duelists[2]].client->ps.stats[STAT_HEALTH] > 0 &&
                            g_entities[duelists[2]].client->sess.sessionTeam == TEAM_FREE)
                        {
                            g_entities[duelists[2]].client->sess.wins++;
                        }
                        else
                        {
                            g_entities[duelists[2]].client->sess.losses++;
                        }
                        ClientUserinfoChanged(duelists[2]);
                    }

                    //Will want to parse indecies for two out at some point probably
                    trap_SetConfigstring ( CS_CLIENT_DUELWINNER, va("%i", duelists[1] ) );

                    if (d_powerDuelPrint.integer)
                    {
                        Com_Printf("POWERDUEL WIN CONDITION: Coupled duelists won (1)\n");
                    }
                    LogExit( "Coupled duelists won." );
                    gDuelExit = qfalse;
                }
                else if ((!g_entities[duelists[1]].inuse ||
                    !g_entities[duelists[1]].client ||
                    g_entities[duelists[1]].client->sess.sessionTeam != TEAM_FREE ||
                    g_entities[duelists[1]].client->ps.stats[STAT_HEALTH] <= 0) &&
                    (!g_entities[duelists[2]].inuse ||
                    !g_entities[duelists[2]].client ||
                    g_entities[duelists[2]].client->sess.sessionTeam != TEAM_FREE ||
                    g_entities[duelists[2]].client->ps.stats[STAT_HEALTH] <= 0))
                { //the coupled duelists lost, give the lone duelist a win (if applicable) and the couple both losses
                    if (g_entities[duelists[1]].inuse &&
                        g_entities[duelists[1]].client)
                    {
                        g_entities[duelists[1]].client->sess.losses++;
                        ClientUserinfoChanged(duelists[1]);
                    }
                    if (g_entities[duelists[2]].inuse &&
                        g_entities[duelists[2]].client)
                    {
                        g_entities[duelists[2]].client->sess.losses++;
                        ClientUserinfoChanged(duelists[2]);
                    }

                    if (g_entities[duelists[0]].inuse &&
                        g_entities[duelists[0]].client &&
                        g_entities[duelists[0]].client->ps.stats[STAT_HEALTH] > 0 &&
                        g_entities[duelists[0]].client->sess.sessionTeam == TEAM_FREE)
                    {
                        g_entities[duelists[0]].client->sess.wins++;
                        ClientUserinfoChanged(duelists[0]);
                    }

                    trap_SetConfigstring ( CS_CLIENT_DUELWINNER, va("%i", duelists[0] ) );

                    if (d_powerDuelPrint.integer)
                    {
                        Com_Printf("POWERDUEL WIN CONDITION: Lone duelist won (1)\n");
                    }
                    LogExit( "Lone duelist won." );
                    gDuelExit = qfalse;
                }
            }
            */
            return;
        }

        if (*addr_of!(level)).numPlayingClients < 2 {
            return;
        }

        let sKillLimit: &str;
        if (*addr_of!(g_gametype)).integer == GT_DUEL
            || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        {
            if (*addr_of!(g_fraglimit)).integer > 1 {
                sKillLimit = "Kill limit hit.";
            } else {
                sKillLimit = "";
                printLimit = QFALSE;
            }
        } else {
            sKillLimit = "Kill limit hit.";
        }
        if (*addr_of!(g_gametype)).integer < GT_SIEGE && (*addr_of!(g_fraglimit)).integer != 0 {
            if (*addr_of!(level)).teamScores[TEAM_RED as usize] >= (*addr_of!(g_fraglimit)).integer
            {
                trap::SendServerCommand(
                    -1,
                    &CStr::from_ptr(va(format_args!(
                        "print \"Red {}\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr(),
                            c"HIT_THE_KILL_LIMIT".as_ptr()
                        ))
                    )))
                    .to_string_lossy(),
                );
                if (*addr_of!(d_powerDuelPrint)).integer != 0 {
                    Com_Printf("POWERDUEL WIN CONDITION: Kill limit (1)\n");
                }
                LogExit(sKillLimit);
                return;
            }

            if (*addr_of!(level)).teamScores[TEAM_BLUE as usize] >= (*addr_of!(g_fraglimit)).integer
            {
                trap::SendServerCommand(
                    -1,
                    &CStr::from_ptr(va(format_args!(
                        "print \"Blue {}\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr(),
                            c"HIT_THE_KILL_LIMIT".as_ptr()
                        ))
                    )))
                    .to_string_lossy(),
                );
                if (*addr_of!(d_powerDuelPrint)).integer != 0 {
                    Com_Printf("POWERDUEL WIN CONDITION: Kill limit (2)\n");
                }
                LogExit(sKillLimit);
                return;
            }

            for i in 0..(*addr_of!(g_maxclients)).integer {
                let cl = (*addr_of!(level)).clients.add(i as usize);
                if (*cl).pers.connected != CON_CONNECTED {
                    continue;
                }
                if (*cl).sess.sessionTeam != TEAM_FREE {
                    continue;
                }

                if ((*addr_of!(g_gametype)).integer == GT_DUEL
                    || (*addr_of!(g_gametype)).integer == GT_POWERDUEL)
                    && (*addr_of!(g_duel_fraglimit)).integer != 0
                    && (*cl).sess.wins >= (*addr_of!(g_duel_fraglimit)).integer
                {
                    if (*addr_of!(d_powerDuelPrint)).integer != 0 {
                        Com_Printf("POWERDUEL WIN CONDITION: Duel limit hit (1)\n");
                    }
                    LogExit("Duel limit hit.");
                    gDuelExit = QTRUE;
                    trap::SendServerCommand(
                        -1,
                        &CStr::from_ptr(va(format_args!(
                            "print \"{}^7 hit the win limit.\n\"",
                            Sz((*cl).pers.netname.as_ptr())
                        )))
                        .to_string_lossy(),
                    );
                    return;
                }

                if (*cl).ps.persistant[PERS_SCORE as usize] >= (*addr_of!(g_fraglimit)).integer {
                    if (*addr_of!(d_powerDuelPrint)).integer != 0 {
                        Com_Printf("POWERDUEL WIN CONDITION: Kill limit (3)\n");
                    }
                    LogExit(sKillLimit);
                    gDuelExit = QFALSE;
                    if printLimit != QFALSE {
                        trap::SendServerCommand(
                            -1,
                            &CStr::from_ptr(va(format_args!(
                                "print \"{}^7 {}.\n\"",
                                Sz((*cl).pers.netname.as_ptr()),
                                Sz(G_GetStringEdString(
                                    c"MP_SVGAME".as_ptr(),
                                    c"HIT_THE_KILL_LIMIT".as_ptr()
                                ))
                            )))
                            .to_string_lossy(),
                        );
                    }
                    return;
                }
            }
        }

        if (*addr_of!(g_gametype)).integer >= GT_CTF && (*addr_of!(g_capturelimit)).integer != 0 {
            if (*addr_of!(level)).teamScores[TEAM_RED as usize]
                >= (*addr_of!(g_capturelimit)).integer
            {
                trap::SendServerCommand(
                    -1,
                    &CStr::from_ptr(va(format_args!(
                        "print \"{} \"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr(),
                            c"PRINTREDTEAM".as_ptr()
                        ))
                    )))
                    .to_string_lossy(),
                );
                trap::SendServerCommand(
                    -1,
                    &CStr::from_ptr(va(format_args!(
                        "print \"{}.\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr(),
                            c"HIT_CAPTURE_LIMIT".as_ptr()
                        ))
                    )))
                    .to_string_lossy(),
                );
                LogExit("Capturelimit hit.");
                return;
            }

            if (*addr_of!(level)).teamScores[TEAM_BLUE as usize]
                >= (*addr_of!(g_capturelimit)).integer
            {
                trap::SendServerCommand(
                    -1,
                    &CStr::from_ptr(va(format_args!(
                        "print \"{} \"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr(),
                            c"PRINTBLUETEAM".as_ptr()
                        ))
                    )))
                    .to_string_lossy(),
                );
                trap::SendServerCommand(
                    -1,
                    &CStr::from_ptr(va(format_args!(
                        "print \"{}.\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr(),
                            c"HIT_CAPTURE_LIMIT".as_ptr()
                        ))
                    )))
                    .to_string_lossy(),
                );
                LogExit("Capturelimit hit.");
                return;
            }
        }
    }
}

/// `void AddTournamentPlayer( void )` (g_main.c:1261) — if there are fewer than two
/// tournament (duel) players, pull the longest-waiting eligible spectator into the
/// game and restart. Walks `level.clients`, skips clients that are not connected
/// spectators, are lagging out (when `g_allowHighPingDuelist` is off), or are the
/// dedicated follow/scoreboard clients, then `SetTeam`s the
/// earliest-`spectatorTime` candidate to free-for-all.
///
/// No-oracle: walks the mutable `level`/`g_entities` globals and side-effects via
/// `SetTeam` — same precedent as the other `level`-walking duel orchestration in
/// this file. Faithful 1:1.
///
/// SAFETY: `level` is a valid module static; indices range within `level.maxclients`
/// into the engine-allocated `level.clients`, and `nextInLine - level.clients`
/// recovers the same client's `g_entities` slot.
pub unsafe fn AddTournamentPlayer() {
    let lvl = addr_of_mut!(level);

    if (*lvl).numPlayingClients >= 2 {
        return;
    }

    // never change during intermission
    //	if ( level.intermissiontime ) {
    //		return;
    //	}

    let mut nextInLine: *mut gclient_t = null_mut();

    for i in 0..(*lvl).maxclients {
        let client: *mut gclient_t = (*lvl).clients.offset(i as isize);
        if (*client).pers.connected != CON_CONNECTED {
            continue;
        }
        if g_allowHighPingDuelist.integer == 0 && (*client).ps.ping >= 999 {
            //don't add people who are lagging out if cvar is not set to allow it.
            continue;
        }
        if (*client).sess.sessionTeam != TEAM_SPECTATOR {
            continue;
        }
        // never select the dedicated follow or scoreboard clients
        if (*client).sess.spectatorState == SPECTATOR_SCOREBOARD
            || (*client).sess.spectatorClient < 0
        {
            continue;
        }

        if nextInLine.is_null() || (*client).sess.spectatorTime < (*nextInLine).sess.spectatorTime {
            nextInLine = client;
        }
    }

    if nextInLine.is_null() {
        return;
    }

    (*lvl).warmupTime = -1;

    // set them to free-for-all team
    let idx = nextInLine.offset_from((*lvl).clients);
    SetTeam(
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(idx),
        c"f".as_ptr() as *mut c_char,
    );
}

/// `void RemoveTournamentLoser( void )` (g_main.c:1327) — make the loser a spectator
/// at the back of the line. Only acts when exactly two clients are playing; takes the
/// second-ranked `sortedClients` entry and, if still connected, `SetTeam`s them to
/// spectator.
///
/// No-oracle: reads the mutable `level` global and side-effects via `SetTeam`. Faithful 1:1.
///
/// SAFETY: `level` is a valid module static; `sortedClients[1]` is populated by
/// `CalculateRanks` and indexes within range when two clients are playing.
pub unsafe fn RemoveTournamentLoser() {
    let lvl = addr_of_mut!(level);

    if (*lvl).numPlayingClients != 2 {
        return;
    }

    let clientNum = (*lvl).sortedClients[1];

    if (*(*lvl).clients.offset(clientNum as isize)).pers.connected != CON_CONNECTED {
        return;
    }

    // make them a spectator
    SetTeam(
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(clientNum as isize),
        c"s".as_ptr() as *mut c_char,
    );
}

/// `void RemoveTournamentWinner( void )` (g_main.c:1542) — make the tournament
/// winner a spectator. Only acts when exactly two clients are playing; takes the
/// top-ranked `sortedClients` entry and, if still connected, `SetTeam`s them to
/// spectator.
///
/// No-oracle: reads the mutable `level` global and side-effects via `SetTeam`. Faithful 1:1.
///
/// SAFETY: `level` is a valid module static; `sortedClients[0]` is populated by
/// `CalculateRanks` and indexes within range when two clients are playing.
pub unsafe fn RemoveTournamentWinner() {
    let lvl = addr_of_mut!(level);

    if (*lvl).numPlayingClients != 2 {
        return;
    }

    let clientNum = (*lvl).sortedClients[0];

    if (*(*lvl).clients.offset(clientNum as isize)).pers.connected != CON_CONNECTED {
        return;
    }

    // make them a spectator
    SetTeam(
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(clientNum as isize),
        c"s".as_ptr() as *mut c_char,
    );
}

/// `void G_PowerDuelCount( int *loners, int *doubles, qboolean countSpec )`
/// (g_main.c:1344) — tally how many connected clients are on each power-duel side.
/// Walks `g_entities[0..MAX_CLIENTS]`, and for every in-use client entity (skipping
/// spectators unless `countSpec`) increments `*loners` for `DUELTEAM_LONE` or
/// `*doubles` for `DUELTEAM_DOUBLE`.
///
/// No-oracle: reads the `g_entities` global array (engine-allocated `gentity_t`s)
/// through their `client` back-pointers — the same disproportionate-scaffolding
/// precedent as the other `level`/`g_entities` walkers in this file. Faithful 1:1.
///
/// SAFETY: `g_entities` is a valid module static of at least `MAX_CLIENTS` entries
/// once `G_InitGame` has run; each in-use client entity's `client` pointer is set
/// by `G_InitGame` to `level.clients + i`. The two out-params are caller-owned.
pub fn G_PowerDuelCount(loners: &mut c_int, doubles: &mut c_int, count_spec: qboolean) {
    unsafe {
        let mut i: usize = 0;

        while i < MAX_CLIENTS {
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i);
            let cl = (*ent).client;

            if (*ent).inuse != QFALSE
                && !cl.is_null()
                && (count_spec != QFALSE || (*cl).sess.sessionTeam != TEAM_SPECTATOR)
            {
                if (*cl).sess.duelTeam == DUELTEAM_LONE {
                    *loners += 1;
                } else if (*cl).sess.duelTeam == DUELTEAM_DOUBLE {
                    *doubles += 1;
                }
            }
            i += 1;
        }
    }
}

/// `qboolean g_duelAssigning` (g_main.c:1368) — guards re-entrant power-duel team
/// assignment; declared just before `AddPowerDuelPlayers` in the C.
pub static mut g_duelAssigning: qboolean = QFALSE;

/// `void AddPowerDuelPlayers( void )` (g_main.c:1369) — fill out a power-duel (one
/// "lone" duelist vs. a "double" pair) by pulling eligible spectators in. Bails if
/// three are already playing, if the in-game balance is already satisfied, or if the
/// spectator pool can't even form a balanced set; otherwise `SetTeam`s the
/// longest-waiting eligible spectator into the game and recurses until everyone is in.
///
/// No-oracle: walks the mutable `level`/`g_entities` globals and side-effects via
/// `G_PowerDuelCount`/`SetTeam`; recursive, like the C. Faithful 1:1.
///
/// SAFETY: `level` is a valid module static; indices range within `level.maxclients`
/// into the engine-allocated `level.clients`, and `nextInLine - level.clients`
/// recovers the same client's `g_entities` slot.
pub unsafe fn AddPowerDuelPlayers() {
    let lvl = addr_of_mut!(level);

    let mut loners: c_int = 0;
    let mut doubles: c_int = 0;
    let mut nonspecLoners: c_int = 0;
    let mut nonspecDoubles: c_int = 0;
    let mut nextInLine: *mut gclient_t;

    if (*lvl).numPlayingClients >= 3 {
        return;
    }

    nextInLine = null_mut();

    G_PowerDuelCount(&mut nonspecLoners, &mut nonspecDoubles, QFALSE);
    if nonspecLoners >= 1 && nonspecDoubles >= 2 {
        //we have enough people, stop
        return;
    }

    //Could be written faster, but it's not enough to care I suppose.
    G_PowerDuelCount(&mut loners, &mut doubles, QTRUE);

    if loners < 1 || doubles < 2 {
        //don't bother trying to spawn anyone yet if the balance is not even set up between spectators
        return;
    }

    //Count again, with only in-game clients in mind.
    loners = nonspecLoners;
    doubles = nonspecDoubles;
    //	G_PowerDuelCount(&loners, &doubles, qfalse);

    for i in 0..(*lvl).maxclients {
        let client: *mut gclient_t = (*lvl).clients.offset(i as isize);
        if (*client).pers.connected != CON_CONNECTED {
            continue;
        }
        if (*client).sess.sessionTeam != TEAM_SPECTATOR {
            continue;
        }
        if (*client).sess.duelTeam == DUELTEAM_FREE {
            continue;
        }
        if (*client).sess.duelTeam == DUELTEAM_LONE && loners >= 1 {
            continue;
        }
        if (*client).sess.duelTeam == DUELTEAM_DOUBLE && doubles >= 2 {
            continue;
        }

        // never select the dedicated follow or scoreboard clients
        if (*client).sess.spectatorState == SPECTATOR_SCOREBOARD
            || (*client).sess.spectatorClient < 0
        {
            continue;
        }

        if nextInLine.is_null() || (*client).sess.spectatorTime < (*nextInLine).sess.spectatorTime {
            nextInLine = client;
        }
    }

    if nextInLine.is_null() {
        return;
    }

    (*lvl).warmupTime = -1;

    // set them to free-for-all team
    let idx = nextInLine.offset_from((*lvl).clients);
    SetTeam(
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(idx),
        c"f".as_ptr() as *mut c_char,
    );

    //Call recursively until everyone is in
    AddPowerDuelPlayers();
}

/// `qboolean g_dontFrickinCheck` (g_main.c:1450) — suppresses an end-of-round check
/// while losers are being removed; declared just before `RemovePowerDuelLosers` in the C.
pub static mut g_dontFrickinCheck: qboolean = QFALSE;

/// `void RemovePowerDuelLosers( void )` (g_main.c:1452) — kick up to three dead/loser
/// duelists out to spectator after a power-duel round, then recompute ranks. Scans
/// `level.clients` for connected clients that are dead or flagged `iAmALoser`; if none
/// qualified, falls back to removing the top `sortedClients` entry. `SetTeam`s each to
/// spectator, clears `g_dontFrickinCheck`, and calls `CalculateRanks`.
///
/// No-oracle: walks the mutable `level` global and side-effects via `SetTeam`/
/// `CalculateRanks`. Faithful 1:1.
///
/// SAFETY: `level` is a valid module static; the scan stays within `MAX_CLIENTS`, and
/// `remClients` indices come from connected clients / `sortedClients[0]`, all valid
/// `g_entities` slots.
pub unsafe fn RemovePowerDuelLosers() {
    let lvl = addr_of_mut!(level);

    let mut remClients: [c_int; 3] = [0; 3];
    let mut remNum: usize = 0;
    let mut i: usize = 0;

    while i < MAX_CLIENTS && remNum < 3 {
        //cl = &level.clients[level.sortedClients[i]];
        let cl: *mut gclient_t = (*lvl).clients.add(i);

        if (*cl).pers.connected == CON_CONNECTED
            && ((*cl).ps.stats[STAT_HEALTH as usize] <= 0 || (*cl).iAmALoser != QFALSE)
            && ((*cl).sess.sessionTeam != TEAM_SPECTATOR || (*cl).iAmALoser != QFALSE)
        {
            //he was dead or he was spectating as a loser
            remClients[remNum] = (*cl).ps.clientNum;
            remNum += 1;
        }

        i += 1;
    }

    if remNum == 0 {
        //Time ran out or something? Oh well, just remove the main guy.
        remClients[remNum] = (*lvl).sortedClients[0];
        remNum += 1;
    }

    i = 0;
    while i < remNum {
        //set them all to spectator
        SetTeam(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(remClients[i] as isize),
            c"s".as_ptr() as *mut c_char,
        );
        i += 1;
    }

    g_dontFrickinCheck = QFALSE;

    //recalculate stuff now that we have reset teams.
    CalculateRanks();
}

/// `void RemoveDuelDrawLoser( void )` (g_main.c:1496) — when a duel ends in a draw,
/// demote the worse-off of the two duelists (higher remaining health+armor wins; an
/// exact tie demotes the second-ranked client). Bails if either of the top two
/// `sortedClients` has disconnected.
///
/// No-oracle: reads the mutable `level` global and side-effects via `SetTeam`. Faithful 1:1.
///
/// SAFETY: `level` is a valid module static; `sortedClients[0]`/`[1]` are populated by
/// `CalculateRanks` and index valid `clients`/`g_entities` slots in a two-player duel.
pub unsafe fn RemoveDuelDrawLoser() {
    let lvl = addr_of_mut!(level);

    // clFirst/clSec/clFailure mirror the C's three locals; clFailure (C `= 0`) is
    // unconditionally assigned by the tie-break below before its only read.
    let clFailure: c_int;

    if (*(*lvl).clients.offset((*lvl).sortedClients[0] as isize))
        .pers
        .connected
        != CON_CONNECTED
    {
        return;
    }
    if (*(*lvl).clients.offset((*lvl).sortedClients[1] as isize))
        .pers
        .connected
        != CON_CONNECTED
    {
        return;
    }

    let clFirst = (*(*lvl).clients.offset((*lvl).sortedClients[0] as isize))
        .ps
        .stats[STAT_HEALTH as usize]
        + (*(*lvl).clients.offset((*lvl).sortedClients[0] as isize))
            .ps
            .stats[STAT_ARMOR as usize];
    let clSec = (*(*lvl).clients.offset((*lvl).sortedClients[1] as isize))
        .ps
        .stats[STAT_HEALTH as usize]
        + (*(*lvl).clients.offset((*lvl).sortedClients[1] as isize))
            .ps
            .stats[STAT_ARMOR as usize];

    if clFirst > clSec {
        clFailure = 1;
    } else if clSec > clFirst {
        clFailure = 0;
    } else {
        clFailure = 2;
    }

    if clFailure != 2 {
        SetTeam(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset((*lvl).sortedClients[clFailure as usize] as isize),
            c"s".as_ptr() as *mut c_char,
        );
    } else {
        //we could be more elegant about this, but oh well.
        SetTeam(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset((*lvl).sortedClients[1] as isize),
            c"s".as_ptr() as *mut c_char,
        );
    }
}

// ========================================================================
//
// FUNCTIONS CALLED EVERY FRAME
//
// ========================================================================

/// `void G_RemoveDuelist(int team)` (g_main.c:2932) — bench every duelist on
/// `team`: walk all `MAX_CLIENTS` entity slots and `SetTeam(ent, "s")` each
/// in-use client whose `duelTeam` matches `team` and who isn't already a
/// spectator. Mirrors the C `while`-loop exactly.
///
/// No-oracle: walks the `g_entities`/`g_clients` globals and side-effects via
/// `SetTeam`, same precedent as the other `level`-walking duel orchestration in
/// this file.
///
/// SAFETY: `g_entities` is the engine-allocated entity array; indices `0..MAX_CLIENTS`
/// are valid client slots and `(*ent).client` is checked non-null before deref.
pub unsafe fn G_RemoveDuelist(team: c_int) {
    let mut i: c_int = 0;
    while i < MAX_CLIENTS as c_int {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (*ent).inuse != QFALSE
            && !(*ent).client.is_null()
            && (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR
            && (*(*ent).client).sess.duelTeam == team
        {
            SetTeam(ent, c"s".as_ptr() as *mut c_char);
        }
        i += 1;
    }
}

/// `void PrintTeam( int team, char *message )` (g_main.c:3410) — send an
/// already-formatted server command `message` to every client on `team`. Walks
/// `level.clients[0..level.maxclients]`, skipping clients not on `team`.
///
/// No-oracle: walks the global `level.clients` array and side-effects via
/// `trap_SendServerCommand`, same precedent as `CheckTeamLeader`/`TeamCount`.
/// Faithful 1:1; `message` keeps its C `char *` shape and is converted to `&str`
/// at the trap boundary.
///
/// SAFETY: `level` is a valid module static; `clients` is the engine-allocated
/// array indexed within `level.maxclients`. `message` is a valid C string.
pub unsafe fn PrintTeam(team: c_int, message: *const c_char) {
    let clients = (*addr_of!(level)).clients;
    let maxclients = (*addr_of!(level)).maxclients;

    for i in 0..maxclients {
        let cl = clients.offset(i as isize);
        if (*cl).sess.sessionTeam != team {
            continue;
        }
        trap::SendServerCommand(i, &CStr::from_ptr(message).to_string_lossy());
    }
}

/// `void SetLeader(int team, int client)` (g_main.c:3425) — make `client` the leader
/// of `team`. Bails (with a `PrintTeam` notice) if the client has disconnected or has
/// left the team; otherwise demotes any current leader on the team, promotes `client`,
/// refreshes the affected userinfos, and announces the new leader to the team.
///
/// No-oracle: walks the global `level.clients` array, mutates `sess.teamLeader`, and
/// fires the `PrintTeam`/`ClientUserinfoChanged` trap side-effects — same precedent as
/// `CheckTeamLeader`/`PrintTeam`. Faithful 1:1.
///
/// SAFETY: `level` is a valid module static; `client`/`i` index within
/// `level.maxclients` into the engine-allocated `level.clients` array.
pub unsafe fn SetLeader(team: c_int, client: c_int) {
    let clients = (*addr_of!(level)).clients;
    let maxclients = (*addr_of!(level)).maxclients;

    if (*clients.offset(client as isize)).pers.connected == CON_DISCONNECTED {
        PrintTeam(
            team,
            va(format_args!(
                "print \"{} is not connected\n\"",
                Sz((*clients.offset(client as isize)).pers.netname.as_ptr())
            )),
        );
        return;
    }
    if (*clients.offset(client as isize)).sess.sessionTeam != team {
        PrintTeam(
            team,
            va(format_args!(
                "print \"{} is not on the team anymore\n\"",
                Sz((*clients.offset(client as isize)).pers.netname.as_ptr())
            )),
        );
        return;
    }
    for i in 0..maxclients {
        let cl = clients.offset(i as isize);
        if (*cl).sess.sessionTeam != team {
            continue;
        }
        if (*cl).sess.teamLeader != QFALSE {
            (*cl).sess.teamLeader = QFALSE;
            ClientUserinfoChanged(i);
        }
    }
    (*clients.offset(client as isize)).sess.teamLeader = QTRUE;
    ClientUserinfoChanged(client);
    PrintTeam(
        team,
        va(format_args!(
            "print \"{} {}\n\"",
            Sz((*clients.offset(client as isize)).pers.netname.as_ptr()),
            Sz(G_GetStringEdString(
                c"MP_SVGAME".as_ptr(),
                c"NEWTEAMLEADER".as_ptr()
            ))
        )),
    );
}

/// `void CheckTeamLeader( int team )` (g_main.c:3454) — make sure `team` has a leader.
/// Scans `level.clients` for an existing leader on `team`; if none, promotes the first
/// non-bot client on the team (or, failing that, the first client of any kind).
///
/// No-oracle: walks the global `level.clients`/`g_entities` arrays, same precedent as
/// `TeamLeader`/`TeamCount`. Faithful 1:1; the C `i >= level.maxclients` post-loop test is
/// reproduced via an explicit "found" flag (Rust `for` consumes its index).
///
/// SAFETY: `level` and `g_entities` are valid module statics for `0..level.maxclients`.
pub unsafe fn CheckTeamLeader(team: c_int) {
    let clients = (*addr_of!(level)).clients;
    let maxclients = (*addr_of!(level)).maxclients;

    let mut found = false;
    for i in 0..maxclients {
        let cl = clients.offset(i as isize);
        if (*cl).sess.sessionTeam != team {
            continue;
        }
        if (*cl).sess.teamLeader != QFALSE {
            found = true;
            break;
        }
    }
    if !found {
        for i in 0..maxclients {
            let cl = clients.offset(i as isize);
            if (*cl).sess.sessionTeam != team {
                continue;
            }
            if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize)).r.svFlags & SVF_BOT == 0 {
                (*cl).sess.teamLeader = QTRUE;
                break;
            }
        }
        for i in 0..maxclients {
            let cl = clients.offset(i as isize);
            if (*cl).sess.sessionTeam != team {
                continue;
            }
            (*cl).sess.teamLeader = QTRUE;
            break;
        }
    }
}

/// `void SendScoreboardMessageToAllClients( void )` (g_main.c:1949) — push a fresh
/// scoreboard to every connected client. "Do this at BeginIntermission time and
/// whenever ranks are recalculated due to enters/exits/forced team changes."
///
/// No-oracle: loops over `level.clients[0..level.maxclients]` and fires the
/// `DeathmatchScoreboardMessage` per-client trap side-effect — pure global-walk +
/// I/O, no computable return, the same precedent as the other `level`/`g_entities`
/// walkers in this file. Faithful 1:1.
///
/// SAFETY: `level` and `g_entities` are valid module statics; `level.clients` is
/// the engine-allocated array indexed within `level.maxclients`, and `g_entities + i`
/// is the matching client entity (`G_InitGame` wires `g_entities[i].client`).
pub unsafe fn SendScoreboardMessageToAllClients() {
    for i in 0..(*addr_of!(level)).maxclients {
        if (*(*addr_of!(level)).clients.offset(i as isize))
            .pers
            .connected
            == CON_CONNECTED
        {
            DeathmatchScoreboardMessage((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize));
        }
    }
}

/// `void AdjustTournamentScores( void )` (g_main.c:1564) — award the win/loss after a
/// tournament (duel) round. When the top two `sortedClients` are tied on score and both
/// connected, the round is decided on remaining health+armor: the higher total wins (a
/// tie on that too is broken in favour of `sortedClients[0]`). Otherwise the score
/// already separates them and `sortedClients[0]` simply takes the win. Each result bumps
/// the winner's `sess.wins` / loser's `sess.losses`, refreshes both userinfos, and
/// publishes the winner's client number to `CS_CLIENT_DUELWINNER`.
///
/// No-oracle: mutates the global `level.clients` `sess` state and fires the
/// `ClientUserinfoChanged` / `trap_SetConfigstring` side-effects — the same
/// entity-state-mutation + trap precedent as the other duel functions in this file;
/// nothing here returns a computable value. Faithful 1:1.
///
/// SAFETY: `level` is a valid module static; `level.sortedClients[0]`/`[1]` are
/// populated by `CalculateRanks` before `CheckTournament` calls this, so the
/// `level.clients` indices are in range.
pub unsafe fn AdjustTournamentScores() {
    let lvl = addr_of_mut!(level);
    let clients = (*lvl).clients;

    if (*clients.add((*lvl).sortedClients[0] as usize))
        .ps
        .persistant[PERS_SCORE as usize]
        == (*clients.add((*lvl).sortedClients[1] as usize))
            .ps
            .persistant[PERS_SCORE as usize]
        && (*clients.add((*lvl).sortedClients[0] as usize))
            .pers
            .connected
            == CON_CONNECTED
        && (*clients.add((*lvl).sortedClients[1] as usize))
            .pers
            .connected
            == CON_CONNECTED
    {
        let clFirst = (*clients.add((*lvl).sortedClients[0] as usize)).ps.stats
            [STAT_HEALTH as usize]
            + (*clients.add((*lvl).sortedClients[0] as usize)).ps.stats[STAT_ARMOR as usize];
        let clSec = (*clients.add((*lvl).sortedClients[1] as usize)).ps.stats[STAT_HEALTH as usize]
            + (*clients.add((*lvl).sortedClients[1] as usize)).ps.stats[STAT_ARMOR as usize];
        let mut clFailure;
        let mut clSuccess;

        if clFirst > clSec {
            clFailure = 1;
            clSuccess = 0;
        } else if clSec > clFirst {
            clFailure = 0;
            clSuccess = 1;
        } else {
            clFailure = 2;
            clSuccess = 2;
        }

        if clFailure != 2 {
            let clientNum = (*lvl).sortedClients[clSuccess as usize];

            (*clients.add(clientNum as usize)).sess.wins += 1;
            ClientUserinfoChanged(clientNum);
            trap::SetConfigstring(
                CS_CLIENT_DUELWINNER,
                &CStr::from_ptr(va(format_args!("{}", clientNum))).to_string_lossy(),
            );

            let clientNum = (*lvl).sortedClients[clFailure as usize];

            (*clients.add(clientNum as usize)).sess.losses += 1;
            ClientUserinfoChanged(clientNum);
        } else {
            clSuccess = 0;
            clFailure = 1;

            let clientNum = (*lvl).sortedClients[clSuccess as usize];

            (*clients.add(clientNum as usize)).sess.wins += 1;
            ClientUserinfoChanged(clientNum);
            trap::SetConfigstring(
                CS_CLIENT_DUELWINNER,
                &CStr::from_ptr(va(format_args!("{}", clientNum))).to_string_lossy(),
            );

            let clientNum = (*lvl).sortedClients[clFailure as usize];

            (*clients.add(clientNum as usize)).sess.losses += 1;
            ClientUserinfoChanged(clientNum);
        }
    } else {
        let clientNum = (*lvl).sortedClients[0];
        if (*clients.add(clientNum as usize)).pers.connected == CON_CONNECTED {
            (*clients.add(clientNum as usize)).sess.wins += 1;
            ClientUserinfoChanged(clientNum);

            trap::SetConfigstring(
                CS_CLIENT_DUELWINNER,
                &CStr::from_ptr(va(format_args!("{}", clientNum))).to_string_lossy(),
            );
        }

        let clientNum = (*lvl).sortedClients[1];
        if (*clients.add(clientNum as usize)).pers.connected == CON_CONNECTED {
            (*clients.add(clientNum as usize)).sess.losses += 1;
            ClientUserinfoChanged(clientNum);
        }
    }
}

/// `qboolean G_CanResetDuelists( void )` (g_main.c:1718) — precheck before a new
/// power-duel respawns all three duelists: returns `qtrue` only if the top three
/// `sortedClients` are all live, in-use, non-spectator clients on an assigned duel
/// team (`duelTeam > DUELTEAM_FREE`). Any failure returns `qfalse` immediately.
///
/// No-oracle: reads `level.sortedClients` and the `g_entities` array (entity
/// `inuse`/`health` plus the client `sess`), same precedent as the other duel
/// walkers here. Faithful 1:1.
///
/// SAFETY: `level` and `g_entities` are valid module statics; the top three
/// `sortedClients` indices are populated by `CalculateRanks` before the power-duel
/// logic that calls this, so indexing `g_entities[sortedClients[0..3]]` is in range.
pub fn G_CanResetDuelists() -> qboolean {
    unsafe {
        let lvl = addr_of!(level);

        let mut i: usize = 0;
        while i < 3 {
            // precheck to make sure they are all respawnable
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*lvl).sortedClients[i] as usize);
            let cl = (*ent).client;

            if (*ent).inuse == QFALSE
                || cl.is_null()
                || (*ent).health <= 0
                || (*cl).sess.sessionTeam == TEAM_SPECTATOR
                || (*cl).sess.duelTeam <= DUELTEAM_FREE
            {
                return QFALSE;
            }
            i += 1;
        }

        QTRUE
    }
}

/// `int g_duelPrintTimer = 0;` (g_main.c:2957) — file-scope throttle for the
/// "need more duelists" centre-print in power-duel ([`CheckTournament`]); next print
/// is allowed once `level.time` passes it.
#[allow(non_upper_case_globals)]
pub static mut g_duelPrintTimer: c_int = 0;

/// `CheckTournament` (g_main.c:2958) — per-frame duel/tournament bookkeeping (called
/// from [`G_RunFrame`]): publish the `CS_CLIENT_DUELISTS`/`CS_CLIENT_DUELHEALTHS`
/// configstrings, pull spectators into GT_DUEL, balance/seed GT_POWERDUEL, and run the
/// team-game warmup countdown (`map_restart` when it elapses).
///
/// **Deviation (Xbox split-screen botmatch dropped):** the JKA source is the Xbox
/// codebase; the GT_POWERDUEL "need more players" branch contains a botmatch/splitscreen
/// block (g_main.c:3133-3165) and an `else` bot-removal clause (3169-3198) that call
/// **client-side** symbols absent from the dedicated MP module (`VM_Call(uivm,
/// UI_IS_FULLSCREEN)`, `cls.state`, `Cvar_VariableValue`/`Cvar_SetValue`,
/// `G_AddRandomBot`/`G_RemoveRandomBot`). Both are dropped — retail OpenJK omits them
/// entirely (g_main.c:2429-2443), leaving just the centre-print + `g_duelPrintTimer`
/// reset. Everything else is faithful to JKA, including JKA's `g_duelPrintTimer =
/// level.time + 3000` (OpenJK uses 10000). See DEVIATIONS "CheckTournament".
///
/// **No oracle** — pure `level`/cvar/trap/`g_entities` control flow (the G_InitGame
/// family). All non-Xbox callees are ported and traps wrapped.
pub fn CheckTournament() {
    // check because we run 3 game frames before calling Connect and/or ClientBegin
    // for clients on a map_restart
    //	if ( level.numPlayingClients == 0 && (g_gametype.integer != GT_POWERDUEL) ) {
    //		return;
    //	}

    // SAFETY: single-threaded frame; `level`/cvars/`g_entities` reached via
    // addr_of!/addr_of_mut! (never a &mut to a `static mut` by path); ported callees
    // are `unsafe`. `sortedClients[0..3]` are populated by CalculateRanks before the
    // duel logic that indexes them.
    unsafe {
        let lvl = addr_of_mut!(level);

        if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
            if (*lvl).numPlayingClients >= 3 && (*lvl).numNonSpectatorClients >= 3 {
                trap::SetConfigstring(
                    CS_CLIENT_DUELISTS,
                    &format!(
                        "{}|{}|{}",
                        (*lvl).sortedClients[0],
                        (*lvl).sortedClients[1],
                        (*lvl).sortedClients[2]
                    ),
                );
            }
        } else {
            if (*lvl).numPlayingClients >= 2 {
                trap::SetConfigstring(
                    CS_CLIENT_DUELISTS,
                    &format!("{}|{}", (*lvl).sortedClients[0], (*lvl).sortedClients[1]),
                );
            }
        }

        if (*addr_of!(g_gametype)).integer == GT_DUEL {
            // pull in a spectator if needed
            if (*lvl).numPlayingClients < 2
                && (*lvl).intermissiontime == 0
                && (*lvl).intermissionQueued == 0
            {
                AddTournamentPlayer();

                if (*lvl).numPlayingClients >= 2 {
                    trap::SetConfigstring(
                        CS_CLIENT_DUELISTS,
                        &format!("{}|{}", (*lvl).sortedClients[0], (*lvl).sortedClients[1]),
                    );
                }
            }

            if (*lvl).numPlayingClients >= 2 {
                // nmckenzie: DUEL_HEALTH
                if (*addr_of!(g_showDuelHealths)).integer >= 1 {
                    let ps1 = addr_of!((*(*lvl).clients.add((*lvl).sortedClients[0] as usize)).ps);
                    let ps2 = addr_of!((*(*lvl).clients.add((*lvl).sortedClients[1] as usize)).ps);
                    trap::SetConfigstring(
                        CS_CLIENT_DUELHEALTHS,
                        &format!(
                            "{}|{}|!",
                            (*ps1).stats[STAT_HEALTH as usize],
                            (*ps2).stats[STAT_HEALTH as usize]
                        ),
                    );
                }
            }

            //rww - It seems we have decided there will be no warmup in duel.
            //if (!g_warmup.integer)
            {
                //don't care about any of this stuff then, just add people and leave me alone
                (*lvl).warmupTime = 0;
                return;
            }
            // (the `#if 0` warmup-countdown block is omitted — compiled out in C.)
        } else if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
            if (*lvl).numPlayingClients < 2 {
                //hmm, ok, pull more in.
                *addr_of_mut!(g_dontFrickinCheck) = QFALSE;
            }

            if (*lvl).numPlayingClients > 3 {
                //umm..yes..lets take care of that then.
                let mut lone: c_int = 0;
                let mut dbl: c_int = 0;

                G_PowerDuelCount(&mut lone, &mut dbl, QFALSE);
                if lone > 1 {
                    G_RemoveDuelist(DUELTEAM_LONE);
                } else if dbl > 2 {
                    G_RemoveDuelist(DUELTEAM_DOUBLE);
                }
            } else if (*lvl).numPlayingClients < 3 {
                //hmm, someone disconnected or something and we need em
                let mut lone: c_int = 0;
                let mut dbl: c_int = 0;

                G_PowerDuelCount(&mut lone, &mut dbl, QFALSE);
                if lone < 1 {
                    *addr_of_mut!(g_dontFrickinCheck) = QFALSE;
                } else if dbl < 1 {
                    *addr_of_mut!(g_dontFrickinCheck) = QFALSE;
                }
            }

            // pull in a spectator if needed
            if (*lvl).numPlayingClients < 3 && *addr_of!(g_dontFrickinCheck) == QFALSE {
                AddPowerDuelPlayers();

                if (*lvl).numPlayingClients >= 3 && G_CanResetDuelists() != QFALSE {
                    let te = G_TempEntity(&vec3_origin, EV_GLOBAL_DUEL);
                    (*te).r.svFlags |= SVF_BROADCAST;
                    //this is really pretty nasty, but..
                    (*te).s.otherEntityNum = (*lvl).sortedClients[0];
                    (*te).s.otherEntityNum2 = (*lvl).sortedClients[1];
                    (*te).s.groundEntityNum = (*lvl).sortedClients[2];

                    trap::SetConfigstring(
                        CS_CLIENT_DUELISTS,
                        &format!(
                            "{}|{}|{}",
                            (*lvl).sortedClients[0],
                            (*lvl).sortedClients[1],
                            (*lvl).sortedClients[2]
                        ),
                    );
                    G_ResetDuelists();

                    *addr_of_mut!(g_dontFrickinCheck) = QTRUE;
                } else if (*lvl).numPlayingClients > 0 || (*lvl).numConnectedClients > 0 {
                    if *addr_of!(g_duelPrintTimer) < (*lvl).time {
                        //print once every 10 seconds
                        let mut lone: c_int = 0;
                        let mut dbl: c_int = 0;

                        G_PowerDuelCount(&mut lone, &mut dbl, QTRUE);

                        if lone < 1 {
                            trap::SendServerCommand(
                                -1,
                                &format!(
                                    "cp \"{}\n\"",
                                    Sz(G_GetStringEdString(
                                        c"MP_SVGAME".as_ptr(),
                                        c"DUELMORESINGLE".as_ptr()
                                    ))
                                ),
                            );
                        } else {
                            trap::SendServerCommand(
                                -1,
                                &format!(
                                    "cp \"{}\n\"",
                                    Sz(G_GetStringEdString(
                                        c"MP_SVGAME".as_ptr(),
                                        c"DUELMOREPAIRED".as_ptr()
                                    ))
                                ),
                            );
                        }

                        *addr_of_mut!(g_duelPrintTimer) = (*lvl).time + 10000;
                    }
                }

                if (*lvl).numPlayingClients >= 3 && (*lvl).numNonSpectatorClients >= 3 {
                    //pulled in a needed person
                    if G_CanResetDuelists() != QFALSE {
                        let te = G_TempEntity(&vec3_origin, EV_GLOBAL_DUEL);
                        (*te).r.svFlags |= SVF_BROADCAST;
                        //this is really pretty nasty, but..
                        (*te).s.otherEntityNum = (*lvl).sortedClients[0];
                        (*te).s.otherEntityNum2 = (*lvl).sortedClients[1];
                        (*te).s.groundEntityNum = (*lvl).sortedClients[2];

                        trap::SetConfigstring(
                            CS_CLIENT_DUELISTS,
                            &format!(
                                "{}|{}|{}",
                                (*lvl).sortedClients[0],
                                (*lvl).sortedClients[1],
                                (*lvl).sortedClients[2]
                            ),
                        );

                        if (*addr_of!(g_austrian)).integer != 0 {
                            let c0 = (*lvl).clients.add((*lvl).sortedClients[0] as usize);
                            let c1 = (*lvl).clients.add((*lvl).sortedClients[1] as usize);
                            let c2 = (*lvl).clients.add((*lvl).sortedClients[2] as usize);
                            G_LogPrintf(&format!(
                                "Duel Initiated: {} {}/{} vs {} {}/{} and {} {}/{}, kill limit: {}\n",
                                Sz((*c0).pers.netname.as_ptr()),
                                (*c0).sess.wins,
                                (*c0).sess.losses,
                                Sz((*c1).pers.netname.as_ptr()),
                                (*c1).sess.wins,
                                (*c1).sess.losses,
                                Sz((*c2).pers.netname.as_ptr()),
                                (*c2).sess.wins,
                                (*c2).sess.losses,
                                (*addr_of!(g_fraglimit)).integer
                            ));
                        }
                        //trap_SendConsoleCommand( EXEC_APPEND, "map_restart 0\n" );
                        //FIXME: This seems to cause problems. But we'd like to reset things whenever a new opponent is set.
                    }
                }
            } else {
                //if you have proper num of players then don't try to add again
                *addr_of_mut!(g_dontFrickinCheck) = QTRUE;
            }

            (*lvl).warmupTime = 0;
            return;
        } else if (*lvl).warmupTime != 0 {
            let mut counts = [0 as c_int; TEAM_NUM_TEAMS as usize];
            let mut not_enough = QFALSE;

            if (*addr_of!(g_gametype)).integer > GT_TEAM {
                counts[TEAM_BLUE as usize] = TeamCount(-1, TEAM_BLUE);
                counts[TEAM_RED as usize] = TeamCount(-1, TEAM_RED);

                if counts[TEAM_RED as usize] < 1 || counts[TEAM_BLUE as usize] < 1 {
                    not_enough = QTRUE;
                }
            } else if (*lvl).numPlayingClients < 2 {
                not_enough = QTRUE;
            }

            if not_enough != QFALSE {
                if (*lvl).warmupTime != -1 {
                    (*lvl).warmupTime = -1;
                    trap::SetConfigstring(CS_WARMUP, &format!("{}", (*lvl).warmupTime));
                    G_LogPrintf("Warmup:\n");
                }
                return; // still waiting for team members
            }

            if (*lvl).warmupTime == 0 {
                return;
            }

            // if the warmup is changed at the console, restart it
            /*
            if ( g_warmup.modificationCount != level.warmupModificationCount ) {
                level.warmupModificationCount = g_warmup.modificationCount;
                level.warmupTime = -1;
            }
            */

            // if all players have arrived, start the countdown
            if (*lvl).warmupTime < 0 {
                // fudge by -1 to account for extra delays
                (*lvl).warmupTime = (*lvl).time + ((*addr_of!(g_warmup)).integer - 1) * 1000;
                trap::SetConfigstring(CS_WARMUP, &format!("{}", (*lvl).warmupTime));
                return;
            }

            // if the warmup time has counted down, restart
            if (*lvl).time > (*lvl).warmupTime {
                (*lvl).warmupTime += 10000;
                trap::Cvar_Set("g_restarted", "1");
                trap::SendConsoleCommand(EXEC_APPEND, "map_restart 0\n");
                (*lvl).restarted = QTRUE;
                return;
            }
        }
    }
}

// =====================================================================================
// `int vmMain( int command, int arg0..arg11 )` — g_main.c:512
// -------------------------------------------------------------------------------------
// FALSE-POSITIVE (layout artifact). `ported_index.py` flags `vmMain` as missing because
// no `fn vmMain` lives in this file, but the live ABI dispatch was relocated into the
// VM-ABI scaffold: the `cdecl` engine entry-point `vmMain` is `crate/src/ffi/exports.rs`
// (`pub unsafe extern "C" fn vmMain`, ~exports.rs:26), which packs arg0..arg11 into a
// `[isize; 12]` and forwards to the actual `switch` dispatch `vm_main` just below in this
// file (`g_main.rs:vm_main`, ~3540). So this exact C `switch` IS ported — split across the
// `extern "C"` shim (in `ffi/`) and `vm_main` (here). Faithful translation of the C body,
// for 1:1-file self-documentation only:
//
// pub fn vmMain(command, arg0, arg1, .., arg11) -> isize {
//     match command {
//         GAME_INIT                => { G_InitGame(arg0, arg1, arg2); 0 }
//         GAME_SHUTDOWN            => { G_ShutdownGame(arg0); 0 }
//         GAME_CLIENT_CONNECT      => ClientConnect(arg0, arg1, arg2) as isize,
//         GAME_CLIENT_THINK        => { ClientThink(arg0, null_mut()); 0 }
//         GAME_CLIENT_USERINFO_CHANGED => { ClientUserinfoChanged(arg0); 0 }
//         GAME_CLIENT_DISCONNECT   => { ClientDisconnect(arg0); 0 }
//         GAME_CLIENT_BEGIN        => { ClientBegin(arg0, QTRUE); 0 }
//         GAME_CLIENT_COMMAND      => { ClientCommand(arg0); 0 }
//         GAME_RUN_FRAME           => { G_RunFrame(arg0); 0 }
//         GAME_CONSOLE_COMMAND     => ConsoleCommand() as isize,
//         BOTAI_START_FRAME        => BotAIStartFrame(arg0) as isize,
//         GAME_ROFF_NOTETRACK_CALLBACK => {
//             G_ROFF_NotetrackCallback(&mut g_entities[arg0], arg1 as *const c_char); 0
//         }
//         GAME_SPAWN_RMG_ENTITY    => {
//             if G_ParseSpawnVars(QFALSE) { G_SpawnGEntityFromSpawnVars(QFALSE); }
//             0
//         }
//
//         // rww - begin icarus callbacks (all read their args from the shared-memory
//         // buffer `gSharedBuffer`, cast to the matching `T_G_ICARUS_*` struct).
//         GAME_ICARUS_PLAYSOUND => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_PLAYSOUND;
//             Q3_PlaySound((*sm).taskID, (*sm).entID, (*sm).name, (*sm).channel) as isize
//         }
//         GAME_ICARUS_SET => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_SET;
//             Q3_Set((*sm).taskID, (*sm).entID, (*sm).type_name, (*sm).data) as isize
//         }
//         GAME_ICARUS_LERP2POS => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_LERP2POS;
//             if (*sm).nullAngles {
//                 Q3_Lerp2Pos((*sm).taskID, (*sm).entID, (*sm).origin, null_mut(), (*sm).duration);
//             } else {
//                 Q3_Lerp2Pos((*sm).taskID, (*sm).entID, (*sm).origin, (*sm).angles, (*sm).duration);
//             }
//             0
//         }
//         GAME_ICARUS_LERP2ORIGIN => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_LERP2ORIGIN;
//             Q3_Lerp2Origin((*sm).taskID, (*sm).entID, (*sm).origin, (*sm).duration); 0
//         }
//         GAME_ICARUS_LERP2ANGLES => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_LERP2ANGLES;
//             Q3_Lerp2Angles((*sm).taskID, (*sm).entID, (*sm).angles, (*sm).duration); 0
//         }
//         GAME_ICARUS_GETTAG => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_GETTAG;
//             Q3_GetTag((*sm).entID, (*sm).name, (*sm).lookup, (*sm).info) as isize
//         }
//         GAME_ICARUS_LERP2START => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_LERP2START;
//             Q3_Lerp2Start((*sm).entID, (*sm).taskID, (*sm).duration); 0
//         }
//         GAME_ICARUS_LERP2END => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_LERP2END;
//             Q3_Lerp2End((*sm).entID, (*sm).taskID, (*sm).duration); 0
//         }
//         GAME_ICARUS_USE => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_USE;
//             Q3_Use((*sm).entID, (*sm).target); 0
//         }
//         GAME_ICARUS_KILL => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_KILL;
//             Q3_Kill((*sm).entID, (*sm).name); 0
//         }
//         GAME_ICARUS_REMOVE => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_REMOVE;
//             Q3_Remove((*sm).entID, (*sm).name); 0
//         }
//         GAME_ICARUS_PLAY => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_PLAY;
//             Q3_Play((*sm).taskID, (*sm).entID, (*sm).type_, (*sm).name); 0
//         }
//         GAME_ICARUS_GETFLOAT => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_GETFLOAT;
//             Q3_GetFloat((*sm).entID, (*sm).type_, (*sm).name, &mut (*sm).value) as isize
//         }
//         GAME_ICARUS_GETVECTOR => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_GETVECTOR;
//             Q3_GetVector((*sm).entID, (*sm).type_, (*sm).name, (*sm).value) as isize
//         }
//         GAME_ICARUS_GETSTRING => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_GETSTRING;
//             let mut crap: *mut c_char = null_mut(); // "I am sorry for this -rww"
//             let morecrap = &mut crap;              // "and this"
//             let r = Q3_GetString((*sm).entID, (*sm).type_, (*sm).name, morecrap);
//             if !crap.is_null() { strcpy((*sm).value.as_mut_ptr(), crap); } // success!
//             r as isize
//         }
//         GAME_ICARUS_SOUNDINDEX => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_SOUNDINDEX;
//             G_SoundIndex((*sm).filename); 0
//         }
//         GAME_ICARUS_GETSETIDFORSTRING => {
//             let sm = gSharedBuffer as *mut T_G_ICARUS_GETSETIDFORSTRING;
//             GetIDForString(setTable, (*sm).string) as isize
//         }
//         // rww - end icarus callbacks
//
//         GAME_NAV_CLEARPATHTOPOINT => NAV_ClearPathToPoint(
//             &mut g_entities[arg0], arg1 as *mut f32, arg2 as *mut f32, arg3 as *mut f32, arg4, arg5,
//         ) as isize,
//         GAME_NAV_CLEARLOS => NPC_ClearLOS2(&mut g_entities[arg0], arg1 as *const f32) as isize,
//         GAME_NAV_CLEARPATHBETWEENPOINTS => NAVNEW_ClearPathBetweenPoints(
//             arg0 as *mut f32, arg1 as *mut f32, arg2 as *mut f32, arg3 as *mut f32, arg4, arg5,
//         ) as isize,
//         GAME_NAV_CHECKNODEFAILEDFORENT => NAV_CheckNodeFailedForEnt(&mut g_entities[arg0], arg1) as isize,
//         GAME_NAV_ENTISUNLOCKEDDOOR => G_EntIsUnlockedDoor(arg0) as isize,
//         GAME_NAV_ENTISDOOR => G_EntIsDoor(arg0) as isize,
//         GAME_NAV_ENTISBREAKABLE => G_EntIsBreakable(arg0) as isize,
//         GAME_NAV_ENTISREMOVABLEUSABLE => G_EntIsRemovableUsable(arg0) as isize,
//         GAME_NAV_FINDCOMBATPOINTWAYPOINTS => { CP_FindCombatPointWaypoints(); 0 }
//         GAME_GETITEMINDEXBYTAG => BG_GetItemIndexByTag(arg0, arg1) as isize,
//
//         _ => -1,
//     }
// }
// =====================================================================================

/// Dispatch a `gameExport_t` command from the engine. Returns the `intptr_t`
/// result the engine expects for that command (`0` for the void/unhandled cases).
/// Mirrors the big `switch ( command )` in `g_main.c`; handlers are stubs for now.
pub fn vm_main(command: c_int, args: &[isize; 12]) -> isize {
    match GameExport::from_raw(command) {
        Some(GameExport::GAME_INIT) => {
            // ( int levelTime, int randomSeed, int restart )
            G_InitGame(args[0] as c_int, args[1] as c_int, args[2] as c_int);
            0
        }
        Some(GameExport::GAME_SHUTDOWN) => {
            // ( int restart )
            G_ShutdownGame(args[0] as c_int);
            0
        }
        Some(GameExport::GAME_CLIENT_CONNECT) => {
            // ( int clientNum, qboolean firstTime, qboolean isBot )
            // Return NULL (0) to allow the connection; a non-null char* would be
            // a denial reason. Always-allow until the account/ban port lands.
            unsafe { ClientConnect(args[0] as c_int, args[1] as c_int, args[2] as c_int) as isize }
        }
        Some(GameExport::GAME_CLIENT_THINK) => {
            unsafe { ClientThink(args[0] as c_int, null_mut()); }
            return 0
        }
        Some(GameExport::GAME_CLIENT_USERINFO_CHANGED) => {
            unsafe { ClientUserinfoChanged(args[0] as c_int); }
            return 0
        }
        Some(GameExport::GAME_CLIENT_DISCONNECT) => {
            unsafe { ClientDisconnect(args[0] as c_int); }
            return 0
        }
        Some(GameExport::GAME_CLIENT_BEGIN) => {
            unsafe { ClientBegin(args[0] as c_int, QTRUE); }
            return 0
        }
        Some(GameExport::GAME_CLIENT_COMMAND) => {
            unsafe { ClientCommand(args[0] as c_int); }
            return 0
        }
        Some(GameExport::GAME_RUN_FRAME) => {
            // ( int levelTime )
            G_RunFrame(args[0] as c_int);
            0
        }
        Some(GameExport::GAME_CONSOLE_COMMAND) => {
            unsafe { ConsoleCommand() as isize }
        }
        Some(GameExport::BOTAI_START_FRAME) => {
            unsafe { BotAIStartFrame(args[0] as c_int) as isize }
        }
        Some(GameExport::GAME_ROFF_NOTETRACK_CALLBACK) => {
            unsafe { G_ROFF_NotetrackCallback((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(args[0] as usize), args[1] as *const c_char); }
            0
        }
        Some(GameExport::GAME_SPAWN_RMG_ENTITY) => {
            if unsafe { G_ParseSpawnVars(QFALSE) } != QFALSE {
                unsafe { G_SpawnGEntityFromSpawnVars(QFALSE); }
            }
            0
        }
        // rww - begin icarus callbacks
        Some(GameExport::GAME_ICARUS_PLAYSOUND) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_PLAYSOUND;
            Q3_PlaySound((*sm).taskID, (*sm).entID, (*sm).name.as_ptr(), (*sm).channel.as_ptr()) as isize
        }
        Some(GameExport::GAME_ICARUS_SET) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_SET;
            Q3_Set((*sm).taskID, (*sm).entID, (*sm).type_name.as_ptr(), (*sm).data.as_ptr()) as isize
        }
        Some(GameExport::GAME_ICARUS_LERP2POS) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_LERP2POS;
            let angles: *const vec3_t = if (*sm).nullAngles != QFALSE { null_mut() } else { &(*sm).angles };
            Q3_Lerp2Pos((*sm).taskID, (*sm).entID, &(*sm).origin, angles, (*sm).duration);
            0
        }
        Some(GameExport::GAME_ICARUS_LERP2ORIGIN) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_LERP2ORIGIN;
            Q3_Lerp2Origin((*sm).taskID, (*sm).entID, &(*sm).origin, (*sm).duration);
            0
        }
        Some(GameExport::GAME_ICARUS_LERP2ANGLES) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_LERP2ANGLES;
            Q3_Lerp2Angles((*sm).taskID, (*sm).entID, &(*sm).angles, (*sm).duration);
            0
        }
        Some(GameExport::GAME_ICARUS_GETTAG) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_GETTAG;
            Q3_GetTag((*sm).entID, (*sm).name.as_ptr(), (*sm).lookup, &mut (*sm).info) as isize
        }
        Some(GameExport::GAME_ICARUS_LERP2START) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_LERP2START;
            Q3_Lerp2Start((*sm).entID, (*sm).taskID, (*sm).duration);
            0
        }
        Some(GameExport::GAME_ICARUS_LERP2END) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_LERP2END;
            Q3_Lerp2End((*sm).entID, (*sm).taskID, (*sm).duration);
            0
        }
        Some(GameExport::GAME_ICARUS_USE) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_USE;
            Q3_Use((*sm).entID, (*sm).target.as_ptr());
            0
        }
        Some(GameExport::GAME_ICARUS_KILL) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_KILL;
            Q3_Kill((*sm).entID, (*sm).name.as_ptr());
            0
        }
        Some(GameExport::GAME_ICARUS_REMOVE) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_REMOVE;
            Q3_Remove((*sm).entID, (*sm).name.as_ptr());
            0
        }
        Some(GameExport::GAME_ICARUS_PLAY) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_PLAY;
            Q3_Play((*sm).taskID, (*sm).entID, (*sm).type_.as_ptr(), (*sm).name.as_ptr());
            0
        }
        Some(GameExport::GAME_ICARUS_GETFLOAT) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_GETFLOAT;
            Q3_GetFloat((*sm).entID, (*sm).type_, (*sm).name.as_ptr(), &mut (*sm).value) as isize
        }
        Some(GameExport::GAME_ICARUS_GETVECTOR) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_GETVECTOR;
            Q3_GetVector((*sm).entID, (*sm).type_, (*sm).name.as_ptr(), &mut (*sm).value) as isize
        }
        Some(GameExport::GAME_ICARUS_GETSTRING) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_GETSTRING;
            let mut crap: *mut c_char = null_mut(); // "I am sorry for this -rww"
            let r = Q3_GetString((*sm).entID, (*sm).type_, (*sm).name.as_ptr(), &mut crap);
            if !crap.is_null() {
                strcpy((*sm).value.as_mut_ptr(), crap);
            }
            r as isize
        }
        Some(GameExport::GAME_ICARUS_SOUNDINDEX) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_SOUNDINDEX;
            G_SoundIndex(CStr::from_ptr((*sm).filename.as_ptr()).to_str().unwrap_or(""));
            0
        }
        Some(GameExport::GAME_ICARUS_GETSETIDFORSTRING) => unsafe {
            let sm = addr_of_mut!(gSharedBuffer) as *mut T_G_ICARUS_GETSETIDFORSTRING;
            GetIDForString(addr_of!(setTable) as *const stringID_table_t, (*sm).string.as_ptr()) as isize
        }
        // rww - end icarus callbacks

        Some(GameExport::GAME_NAV_CLEARPATHTOPOINT) => {
            // NAV_ClearPathToPoint(&g_entities[arg0], (float*)arg1, (float*)arg2, (float*)arg3, arg4, arg5)
            unsafe {
                NAV_ClearPathToPoint(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(args[0] as usize),
                    &*(args[1] as *const vec3_t),
                    &*(args[2] as *const vec3_t),
                    &*(args[3] as *const vec3_t),
                    args[4] as c_int,
                    args[5] as c_int,
                ) as isize
            }
        }
        Some(GameExport::GAME_NAV_CLEARLOS) => {
            // NPC_ClearLOS2(&g_entities[arg0], (const float*)arg1)
            unsafe {
                NPC_ClearLOS2(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(args[0] as usize),
                    &*(args[1] as *const vec3_t),
                ) as isize
            }
        }
        Some(GameExport::GAME_NAV_CLEARPATHBETWEENPOINTS) => {
            // NAVNEW_ClearPathBetweenPoints((float*)arg0, (float*)arg1, (float*)arg2, (float*)arg3, arg4, arg5)
            unsafe {
                NAVNEW_ClearPathBetweenPoints(
                    &*(args[0] as *const vec3_t),
                    &*(args[1] as *const vec3_t),
                    &*(args[2] as *const vec3_t),
                    &*(args[3] as *const vec3_t),
                    args[4] as c_int,
                    args[5] as c_int,
                ) as isize
            }
        }
        Some(GameExport::GAME_NAV_CHECKNODEFAILEDFORENT) => {
            // NAV_CheckNodeFailedForEnt(&g_entities[arg0], arg1)
            unsafe {
                NAV_CheckNodeFailedForEnt(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(args[0] as usize),
                    args[1] as c_int,
                ) as isize
            }
        }
        Some(GameExport::GAME_NAV_ENTISUNLOCKEDDOOR) => {
            unsafe { G_EntIsUnlockedDoor(args[0] as c_int) as isize }
        }
        Some(GameExport::GAME_NAV_ENTISDOOR) => {
            unsafe { G_EntIsDoor(args[0] as c_int) as isize }
        }
        Some(GameExport::GAME_NAV_ENTISBREAKABLE) => {
            unsafe { G_EntIsBreakable(args[0] as c_int) as isize }
        }
        Some(GameExport::GAME_NAV_ENTISREMOVABLEUSABLE) => {
            unsafe { G_EntIsRemovableUsable(args[0] as c_int) as isize }
        }
        Some(GameExport::GAME_NAV_FINDCOMBATPOINTWAYPOINTS) => {
            unsafe { CP_FindCombatPointWaypoints(); }
            0
        }
        Some(GameExport::GAME_GETITEMINDEXBYTAG) => {
            BG_GetItemIndexByTag(args[0] as c_int, args[1] as c_int) as isize
        }

        // Unknown command — matches C's `return -1` fallthrough.
        _ => -1,
    }
}

/// `qboolean gDidDuelStuff` (g_main.c) — "gets reset on game reinit". A file
/// global with external linkage in C (other game files reference it), mirrored
/// here as a `static mut`.
pub static mut gDidDuelStuff: qboolean = QFALSE;

/// `void G_InitGame( int levelTime, int randomSeed, int restart )` — the game
/// module's level-load entry point (engine `GAME_INIT`).
///
/// This is the **opening sequence** of the C `G_InitGame` (g_main.c lines
/// 894-1003): the `g_entities` heap allocation, the version banner, RNG seed
/// (`srand`), cvar registration ([`G_RegisterCvars`] — wired in here for the
/// first time), the `level` re-zero plus the level timers, the `level.snd_*`
/// sound-index registration (via [`G_SoundIndex`]), the `#ifndef _XBOX` logfile
/// block (open `g_log`, then write the `InitGame:` banner through
/// [`G_LogPrintf`]), and the **entity/client array setup + `trap_LocateGameData`**
/// (the pointer-prefix-critical handoff — the engine is handed the array bases
/// and the real `gentity_t`/`gclient_t` strides). All subsystem inits are wired to
/// their ports (`g_vehiclePoolInit`, `B_InitAlloc`, `BG_VehicleLoadParms`,
/// `G_InitWorldSession`, …).
/// **Full port** — the tail is now in place: `WP_SaberLoadParms`, `NPC_InitGame`,
/// `TIMER_Clear`, `trap_ICARUS_Init`, `InitBodyQue`, `ClearRegisteredItems`,
/// `InitSiegeMode`, the `mapname`/`sv_mapChecksum` cvars + `trap_Nav_Load`, the entity
/// spawner `G_SpawnEntitiesFromString`, `G_FindTeams`/`G_CheckTeamItems`, the duel
/// configstrings, `SaveRegisteredItems`, the SP build-script precache, bot setup
/// (`BotAISetup`/`BotAILoadMap`/`G_InitBots`), `G_RemapTeamShaders`, the nav
/// path-calc decision (`navCalcPathTime`/`CP_FindCombatPointWaypoints`), and the
/// GT_SIEGE custom-sound registration. See DEVIATIONS.md.
///
/// `restart` is part of the ABI signature but is first consumed by the not-yet-ported
/// tail (`BotAISetup(restart)` etc.), hence the fn-scoped `#[allow(unused_variables)]`
/// until that slice lands.
#[allow(unused_variables)]
pub fn G_InitGame(levelTime: c_int, randomSeed: c_int, restart: c_int) {
    // `gentity_t g_entities[MAX_GENTITIES]` is a true static (BSS) array — program
    // lifetime, no per-map (re)alloc. PC just `memset`s it each map load (done in the
    // entity-init step below), so there is no allocation to perform here.

    // (the #ifdef _XBOX restart re-init block is omitted — non-Xbox retail build.
    // PC G_InitGame does not reset g_vehiclePoolInit; it stays a file-local static in
    // g_utils.rs and is reset only by the one-shot first G_AllocateVehicleObject call.)

    // SAFETY: every access below is a raw-pointer deref of a module static via
    // addr_of_mut!, never a `&mut` to a `static mut` by path (static_mut_refs).
    // `srand` is the libc extern declared at the top of this module.
    unsafe {
        // Fix for yet another friggin global in the goddamn fucking DLLs.
        *addr_of_mut!(gDidDuelStuff) = QFALSE;

        //Init RMG to 0, it will be autoset to 1 if there is terrain on the level.
        trap::Cvar_Set("RMG", "0");
        (*addr_of_mut!(g_RMG)).integer = 0;

        // Clean up any client-server ghoul2 instance attachments that may still exist exe-side
        trap::G2API_CleanEntAttachments();
        BG_InitAnimsets(); //clear it out
        B_InitAlloc(); //make sure everything is clean
        trap::SV_RegisterSharedMemory(addr_of_mut!(gSharedBuffer) as *mut c_char);
        BG_VehicleLoadParms();

        G_Printf("------- Game Initialization -------\n");
        G_Printf(&format!("gamename: {GAMEVERSION}\n"));
        // C: G_Printf("gamedate: %s\n", __DATE__). The compiler build date is
        // non-reproducible and left empty (see DEVIATIONS.md, as for `gamedate`).
        G_Printf("gamedate: \n");

        // Rust-port identity marker. Additive console log only — it deliberately
        // does NOT touch the faithful `gamename`/`gamedate` strings above, which
        // stay bit-for-bit JKA for ABI/oracle parity. This line is how a live
        // server confirms the Rust module (vs. a stock .qvm/.so) loaded.
        G_Printf(&format!(
            "^3rust jampgame v{} loaded^7\n",
            env!("CARGO_PKG_VERSION")
        ));

        srand(randomSeed as c_uint);

        G_RegisterCvars();

        //G_ProcessIPBans();  -- the Xbox tree used this; the PC source loads from banip.txt.
        G_LoadIPBans();
        G_InitMemory();

        // set some level globals
        // memset re-zeroes the entire `level` static. Note this clears the
        // `warmupModificationCount` that G_RegisterCvars set moments ago —
        // faithful to the C order (register at line 937, memset at 944).
        core::ptr::write_bytes(
            addr_of_mut!(level) as *mut u8,
            0,
            core::mem::size_of::<level_locals_t>(),
        );
        (*addr_of_mut!(level)).time = levelTime;
        (*addr_of_mut!(level)).startTime = levelTime;

        (*addr_of_mut!(level)).snd_fry = G_SoundIndex("sound/player/fry.wav"); // FIXME standing in lava / slime

        (*addr_of_mut!(level)).snd_hack = G_SoundIndex("sound/player/hacking.wav");
        (*addr_of_mut!(level)).snd_medHealed = G_SoundIndex("sound/player/supp_healed.wav");
        (*addr_of_mut!(level)).snd_medSupplied = G_SoundIndex("sound/player/supp_supplied.wav");

        //trap_SP_RegisterServer("mp_svgame");

        // #ifndef _XBOX — the retail (non-Xbox) build keeps the logfile block.
        // C tests `g_log.string[0]`; the cvar mirror's `string` buffer becomes a
        // Rust String, so `g_log.string[0] != 0` is `!is_empty()`.
        let g_log_string = CStr::from_ptr((*addr_of!(g_log)).string.as_ptr()).to_string_lossy();
        if !g_log_string.is_empty() {
            let mode = if (*addr_of!(g_logSync)).integer != 0 {
                FS_APPEND_SYNC
            } else {
                FS_APPEND
            };
            // C passes &level.logFile as the out-param; the wrapper returns the
            // handle (the length return is discarded, as in C).
            let (_len, fh) = trap::FS_FOpenFile(&g_log_string, mode);
            (*addr_of_mut!(level)).logFile = fh;
            if (*addr_of!(level)).logFile == 0 {
                G_Printf(&format!("WARNING: Couldn't open logfile: {g_log_string}\n"));
            } else {
                let serverinfo = trap::GetServerinfo();
                G_LogPrintf("------------------------------------------------------------\n");
                G_LogPrintf(&format!("InitGame: {serverinfo}\n"));
            }
        } else {
            G_Printf("Not logging to disk.\n");
        }

        G_LogWeaponInit();

        G_InitWorldSession();

        // initialize all entities for this game
        // memset( g_entities, 0, MAX_GENTITIES * sizeof(g_entities[0]) ); — zero
        // the static entity array each map load.
        core::ptr::write_bytes(addr_of_mut!(g_entities) as *mut gentity_t, 0, MAX_GENTITIES);
        (*addr_of_mut!(level)).gentities = addr_of_mut!(g_entities) as *mut gentity_t;

        // initialize all clients for this game
        (*addr_of_mut!(level)).maxclients = (*addr_of!(g_maxclients)).integer;
        // memset( g_clients, 0, MAX_CLIENTS * sizeof(g_clients[0]) );
        core::ptr::write_bytes(addr_of_mut!(g_clients) as *mut gclient_t, 0, MAX_CLIENTS);
        (*addr_of_mut!(level)).clients = addr_of_mut!(g_clients) as *mut gclient_t;

        // set client fields on player ents
        // for ( i=0 ; i<level.maxclients ; i++ ) g_entities[i].client = level.clients + i;
        let maxclients = (*addr_of!(level)).maxclients;
        for i in 0..maxclients as isize {
            (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i)).client = (*addr_of!(level)).clients.offset(i);
        }

        // always leave room for the max number of clients, even if they aren't
        // all used, so numbers inside that range are NEVER anything but clients
        (*addr_of_mut!(level)).num_entities = MAX_CLIENTS as c_int;

        // let the server system know where the entites are
        // C: trap_LocateGameData( level.gentities, level.num_entities,
        //      sizeof(gentity_t), &level.clients[0].ps, sizeof(level.clients[0]) );
        // The 4th arg is `&clients[0].ps` (`ps` is at offset 0 of gclient_t, the
        // pointer-prefix anchor); the strides are the real struct sizes.
        trap::LocateGameData(
            (*addr_of!(level)).gentities,
            (*addr_of!(level)).num_entities,
            core::mem::size_of::<gentity_t>() as c_int,
            addr_of_mut!((*(*addr_of!(level)).clients).ps),
            core::mem::size_of::<gclient_t>() as c_int,
        );

        //Load sabers.cfg data
        WP_SaberLoadParms();

        NPC_InitGame();

        TIMER_Clear();
        //
        //ICARUS INIT START — initialise the engine-side ICARUS instance so the module's
        // `trap_ICARUS_FreeEnt` calls (G_InitGentity / G_FreeEntity / player_die / ClientSpawn)
        // find a live `iICARUS` instead of asserting.
        trap::ICARUS_Init();
        //ICARUS INIT END

        // reserve some spots for dead player bodies
        InitBodyQue();

        ClearRegisteredItems();

        //make sure saber data is loaded before this! (so we can precache the appropriate hilts)
        InitSiegeMode();

        // `mapname` is a function-local cvar in the C source (g_main.c:899), not
        // the module static the table once carried.
        let mut mapname: vmCvar_t = vmCvar_t::zeroed();
        trap::Cvar_Register(Some(&mut mapname), "mapname", "", CVAR_SERVERINFO | CVAR_ROM);
        let mut ck_sum: vmCvar_t = vmCvar_t::zeroed();
        trap::Cvar_Register(Some(&mut ck_sum), "sv_mapChecksum", "", CVAR_ROM);

        let mapname_str = CStr::from_ptr(mapname.string.as_ptr()).to_string_lossy();
        *addr_of_mut!(navCalculatePaths) =
            (trap::Nav_Load(&mapname_str, ck_sum.integer) == QFALSE) as qboolean;

        // parse the key/value pairs and spawn gentities
        G_SpawnEntitiesFromString(QFALSE);

        // general initialization
        G_FindTeams();

        // make sure we have flags for CTF, etc
        if (*addr_of!(g_gametype)).integer >= GT_TEAM {
            G_CheckTeamItems();
        } else if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER {
            trap::SetConfigstring(CS_CLIENT_JEDIMASTER, "-1");
        }

        if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
            trap::SetConfigstring(CS_CLIENT_DUELISTS, "-1|-1|-1");
        } else {
            trap::SetConfigstring(CS_CLIENT_DUELISTS, "-1|-1");
        }
        // nmckenzie: DUEL_HEALTH: Default.
        trap::SetConfigstring(CS_CLIENT_DUELHEALTHS, "-1|-1|!");
        trap::SetConfigstring(CS_CLIENT_DUELWINNER, "-1");

        SaveRegisteredItems();

        if (*addr_of!(g_gametype)).integer == GT_SINGLE_PLAYER
            || trap::Cvar_VariableIntegerValue("com_buildScript") != 0
        {
            G_ModelIndex(SP_PODIUM_MODEL);
            G_SoundIndex("sound/player/gurp1.wav");
            G_SoundIndex("sound/player/gurp2.wav");
        }

        if trap::Cvar_VariableIntegerValue("bot_enable") != 0 {
            BotAISetup(restart);
            BotAILoadMap(restart);
            G_InitBots(restart);
        }

        G_RemapTeamShaders();

        if (*addr_of!(g_gametype)).integer == GT_DUEL
            || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        {
            G_LogPrintf(&format!(
                "Duel Tournament Begun: kill limit {}, win limit: {}\n",
                (*addr_of!(g_fraglimit)).integer,
                (*addr_of!(g_duel_fraglimit)).integer
            ));
        }

        if *addr_of!(navCalculatePaths) != QFALSE {
            //not loaded - need to calc paths
            navCalcPathTime = (*addr_of!(level)).time + START_TIME_NAV_CALC; //make sure all ents are in and linked
        } else {
            //loaded
            trap::Nav_SetPathsCalculated(QTRUE);
            //need to do this, because combatpoint waypoints aren't saved out...?
            CP_FindCombatPointWaypoints();
            navCalcPathTime = 0;
            //No loading games in MP.
        }

        if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            //just get these configstrings registered now...
            let mut i = 0usize;
            while i < MAX_CUSTOM_SIEGE_SOUNDS {
                let snd = (*addr_of!(bg_customSiegeSoundNames))[i];
                if snd.is_null() {
                    break;
                }
                G_SoundIndex(&CStr::from_ptr(snd).to_string_lossy());
                i += 1;
            }
        }
    }
}

/// `G_ShutdownGame` (g_main.c:1132) — the `GAME_SHUTDOWN` half of the module
/// lifecycle: release every dynamic allocation the module made (fake clients,
/// animsets, per-entity + per-client + global ghoul2 instances, ICARUS). (The `B_Alloc`
/// pool cleanup is deferred with its `B_InitAlloc` partner — see the body.)
///
/// **`g_entities` ownership decision (Option B):** the array is a true static (BSS)
/// array `gentity_t g_entities[MAX_GENTITIES]` — PC's literal source form (g_main.c:27),
/// program lifetime, nothing to free. The old PC `delete[]`-on-non-restart applied only
/// to the heap-allocated variant; the static array never frees.
///
/// **No oracle** — pure control flow over module statics / `g_entities` / traps
/// (the `G_InitGame` precedent). All callees are ported and all ghoul2/ICARUS/ROFF/FS
/// traps wrapped; the loop, the `MAX_SABERS` weapon-ghoul2 sub-loop, the two global
/// ghoul2 templates (`g2SaberInstance`, `precachedKyle`), and the `bot_enable`-gated
/// `BotAIShutdown` are carried verbatim.
pub fn G_ShutdownGame(restart: c_int) {
    // SAFETY: single-threaded module shutdown; every static is reached through
    // addr_of!/addr_of_mut! (never a &mut to a `static mut` by path), and every
    // g_entities access is a raw-pointer deref of the (post-G_InitGame, non-null)
    // contiguous block.
    unsafe {
        //	G_Printf ("==== ShutdownGame ====\n");

        G_SaveBanIP();
        G_CleanAllFakeClients(); // get rid of dynamically allocated fake client structs.

        BG_ClearAnimsets(); // free all dynamic allocations made through the engine

        //	Com_Printf("... Gameside GHOUL2 Cleanup\n");
        let mut i = 0usize;
        while i < MAX_GENTITIES {
            // clean up all the ghoul2 instances
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i);

            if !(*ent).ghoul2.is_null() && trap::G2_HaveWeGhoul2Models((*ent).ghoul2) != QFALSE {
                trap::G2API_CleanGhoul2Models(addr_of_mut!((*ent).ghoul2));
                (*ent).ghoul2 = null_mut();
            }
            if !(*ent).client.is_null() {
                let mut j = 0usize;
                while j < MAX_SABERS {
                    if !(*(*ent).client).weaponGhoul2[j].is_null()
                        && trap::G2_HaveWeGhoul2Models((*(*ent).client).weaponGhoul2[j]) != QFALSE
                    {
                        trap::G2API_CleanGhoul2Models(addr_of_mut!(
                            (*(*ent).client).weaponGhoul2[j]
                        ));
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        if !(*addr_of!(g2SaberInstance)).is_null()
            && trap::G2_HaveWeGhoul2Models(*addr_of!(g2SaberInstance)) != QFALSE
        {
            trap::G2API_CleanGhoul2Models(addr_of_mut!(g2SaberInstance));
            *addr_of_mut!(g2SaberInstance) = null_mut();
        }
        if !(*addr_of!(precachedKyle)).is_null()
            && trap::G2_HaveWeGhoul2Models(*addr_of!(precachedKyle)) != QFALSE
        {
            trap::G2API_CleanGhoul2Models(addr_of_mut!(precachedKyle));
            *addr_of_mut!(precachedKyle) = null_mut();
        }

        //	Com_Printf ("... ICARUS_Shutdown\n");
        trap::ICARUS_Shutdown(); // Shut ICARUS down

        //	Com_Printf ("... Reference Tags Cleared\n");
        TAG_Init(); // Clear the reference tags

        G_LogWeaponOutput();

        if (*addr_of!(level)).logFile != 0 {
            G_LogPrintf("ShutdownGame:\n");
            G_LogPrintf("------------------------------------------------------------\n");
            trap::FS_FCloseFile((*addr_of!(level)).logFile);
        }

        // write all the client session data so we can get it back
        G_WriteSessionData();

        trap::ROFF_Clean();

        if trap::Cvar_VariableIntegerValue("bot_enable") != 0 {
            BotAIShutdown(restart);
        }

        B_CleanupAlloc(); //clean up all allocations made with B_Alloc

        // Cleanup g_entities array:
        // `g_entities` is now a true static (BSS) array with program lifetime — there
        // is nothing to free. PC's `delete[]`-on-non-restart had meaning only for the
        // heap-allocated variant; the static-array form mirrors PC's literal source
        // (`gentity_t g_entities[MAX_GENTITIES];`), which never frees.
    }
}

/// `G_RunFrame` (g_main.c:3659) — the `GAME_RUN_FRAME` per-server-frame tick: advance
/// level time, run the slow-mo duel timescale ramp, walk every in-use entity (events,
/// missiles/items/movers, per-client space-suffocation / hacking / jetpack / cloak /
/// siege upkeep, force-power + saber updates, NPCs, think), end-frame fixups on every
/// client, then the per-frame game checks (tournament/exit/team/vote/cvars) and the
/// queued ghoul2-kill / scoreboard flushes.
///
/// Omitted: the `_G_FRAME_PERFANAL` precision-timer scaffolding and the `_XBOX`
/// split-screen `ClientManager` blocks (non-retail), the `#if 0` body-drag branch, and
/// the dead local `msec` (computed-but-unread in C — the `G_Damage`/`save` precedent).
///
/// **No oracle** — pure per-frame `level`/cvar/`g_entities`/trap control flow (the
/// G_InitGame family). All callees ported and traps wrapped.
pub fn G_RunFrame(levelTime: c_int) {
    // SAFETY: single-threaded per-frame entry; `level`/cvars/`g_entities`/module statics
    // are reached via addr_of!/addr_of_mut! (never a &mut to a `static mut` by path);
    // ported callees are `unsafe`. Entity pointers come from the (post-G_InitGame,
    // non-null) contiguous `g_entities` block, indexed within `level.num_entities`.
    unsafe {
        let lvl = addr_of_mut!(level);

        if (*addr_of!(g_gametype)).integer == GT_SIEGE
            && (*addr_of!(g_siegeRespawn)).integer != 0
            && *addr_of!(g_siegeRespawnCheck) < (*lvl).time
        {
            //check for a respawn wave
            let mut i = 0usize;
            while i < MAX_CLIENTS {
                let cl_ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i);

                if (*cl_ent).inuse != QFALSE
                    && !(*cl_ent).client.is_null()
                    && (*(*cl_ent).client).tempSpectate > (*lvl).time
                    && (*(*cl_ent).client).sess.sessionTeam != TEAM_SPECTATOR
                {
                    respawn(cl_ent);
                    (*(*cl_ent).client).tempSpectate = 0;
                }
                i += 1;
            }

            *addr_of_mut!(g_siegeRespawnCheck) =
                (*lvl).time + (*addr_of!(g_siegeRespawn)).integer * 1000;
        }

        if *addr_of!(gDoSlowMoDuel) != QFALSE {
            if (*lvl).restarted != QFALSE {
                let mut buf = [0 as c_char; 128];
                trap::Cvar_VariableStringBuffer("timescale", &mut buf);
                let t_fval = atof(buf.as_ptr()) as f32;

                trap::Cvar_Set("timescale", "1");
                if t_fval == 1.0 {
                    *addr_of_mut!(gDoSlowMoDuel) = QFALSE;
                }
            } else {
                //difference in time between when the slow motion was initiated and now
                let time_dif = ((*lvl).time - *addr_of!(gSlowMoDuelTime)) as f32;

                if time_dif < 150.0 {
                    trap::Cvar_Set("timescale", "0.1f");
                } else if time_dif < 1150.0 {
                    let mut use_dif = time_dif / 1000.0; //scale from 0.1 up to 1
                    if use_dif < 0.1 {
                        use_dif = 0.1;
                    }
                    if use_dif > 1.0 {
                        use_dif = 1.0;
                    }
                    trap::Cvar_Set("timescale", &format!("{:.6}", use_dif));
                } else {
                    let mut buf = [0 as c_char; 128];
                    trap::Cvar_VariableStringBuffer("timescale", &mut buf);
                    let t_fval = atof(buf.as_ptr()) as f32;

                    trap::Cvar_Set("timescale", "1");
                    if time_dif > 1500.0 && t_fval == 1.0 {
                        *addr_of_mut!(gDoSlowMoDuel) = QFALSE;
                    }
                }
            }
        }

        // if we are waiting for the level to restart, do nothing
        if (*lvl).restarted != QFALSE {
            return;
        }

        (*lvl).framenum += 1;
        (*lvl).previousTime = (*lvl).time;
        (*lvl).time = levelTime;
        // (C computes `msec = level.time - level.previousTime` here but never reads it.)

        if (*addr_of!(g_allowNPC)).integer != 0 {
            NAV_CheckCalcPaths();
        }

        AI_UpdateGroups();

        if (*addr_of!(g_allowNPC)).integer != 0 {
            if (*addr_of!(d_altRoutes)).integer != 0 {
                trap::Nav_CheckAllFailedEdges();
            }
            trap::Nav_ClearCheckedNodes();

            //remember last waypoint, clear current one
            let mut i = 0;
            while i < (*lvl).num_entities {
                let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

                if (*ent).inuse == QFALSE {
                    i += 1;
                    continue;
                }

                if (*ent).waypoint != WAYPOINT_NONE && (*ent).noWaypointTime < (*lvl).time {
                    (*ent).lastWaypoint = (*ent).waypoint;
                    (*ent).waypoint = WAYPOINT_NONE;
                }
                if (*addr_of!(d_altRoutes)).integer != 0 {
                    trap::Nav_CheckFailedNodes(ent);
                }
                i += 1;
            }

            //Look to clear out old events
            ClearPlayerAlertEvents();
        }

        *addr_of_mut!(g_TimeSinceLastFrame) = (*lvl).time - *addr_of!(g_LastFrameTime);

        // get any cvar changes
        G_UpdateCvars();

        //
        // go through all allocated objects
        //
        let mut i = 0;
        while i < (*lvl).num_entities {
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
            if (*ent).inuse == QFALSE {
                i += 1;
                continue;
            }

            // clear events that are too old
            if (*lvl).time - (*ent).eventTime > EVENT_VALID_MSEC {
                if (*ent).s.event != 0 {
                    (*ent).s.event = 0; // &= EV_EVENT_BITS;
                    if !(*ent).client.is_null() {
                        (*(*ent).client).ps.externalEvent = 0;
                        // predicted events should never be set to zero
                    }
                }
                if (*ent).freeAfterEvent != QFALSE {
                    // tempEntities or dropped items completely go away after their event
                    if (*ent).s.eFlags & EF_SOUNDTRACKER != 0 {
                        //don't trigger the event again..
                        (*ent).s.event = 0;
                        (*ent).s.eventParm = 0;
                        (*ent).s.eType = 0;
                        (*ent).eventTime = 0;
                    } else {
                        G_FreeEntity(ent);
                        i += 1;
                        continue;
                    }
                } else if (*ent).unlinkAfterEvent != QFALSE {
                    // items that will respawn will hide themselves after their pickup event
                    (*ent).unlinkAfterEvent = QFALSE;
                    trap::UnlinkEntity(ent);
                }
            }

            // temporary entities don't think
            if (*ent).freeAfterEvent != QFALSE {
                i += 1;
                continue;
            }

            if (*ent).r.linked == QFALSE && (*ent).neverFree != QFALSE {
                i += 1;
                continue;
            }

            if (*ent).s.eType == ET_MISSILE {
                G_RunMissile(ent);
                i += 1;
                continue;
            }

            if (*ent).s.eType == ET_ITEM || (*ent).physicsObject != QFALSE {
                G_RunItem(ent);
                i += 1;
                continue;
            }

            if (*ent).s.eType == ET_MOVER {
                G_RunMover(ent);
                i += 1;
                continue;
            }

            if i < MAX_CLIENTS as c_int {
                G_CheckClientTimeouts(ent);

                if (*(*ent).client).inSpaceIndex != 0
                    && (*(*ent).client).inSpaceIndex != ENTITYNUM_NONE
                {
                    //we're in space, check for suffocating and for exiting
                    let spacetrigger =
                        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).inSpaceIndex as usize);

                    if (*spacetrigger).inuse == QFALSE
                        || G_PointInBounds(
                            &(*(*ent).client).ps.origin,
                            &(*spacetrigger).r.absmin,
                            &(*spacetrigger).r.absmax,
                        ) == QFALSE
                    {
                        //no longer in space then I suppose
                        (*(*ent).client).inSpaceIndex = 0;
                    } else {
                        //check for suffocation
                        if (*(*ent).client).inSpaceSuffocation < (*lvl).time {
                            //suffocate!
                            if (*ent).health > 0 && (*ent).takedamage != QFALSE {
                                //if they're still alive..
                                G_Damage(
                                    ent,
                                    spacetrigger,
                                    spacetrigger,
                                    null_mut(),
                                    addr_of_mut!((*(*ent).client).ps.origin),
                                    Q_irand(50, 70),
                                    DAMAGE_NO_ARMOR,
                                    MOD_SUICIDE,
                                );

                                if (*ent).health > 0 {
                                    //did that last one kill them?
                                    //play the choking sound
                                    G_EntitySound(
                                        ent,
                                        CHAN_VOICE,
                                        G_SoundIndex(&format!("*choke{}.wav", Q_irand(1, 3))),
                                    );

                                    //make them grasp their throat
                                    (*(*ent).client).ps.forceHandExtend = HANDEXTEND_CHOKE;
                                    (*(*ent).client).ps.forceHandExtendTime = (*lvl).time + 2000;
                                }
                            }

                            (*(*ent).client).inSpaceSuffocation = (*lvl).time + Q_irand(100, 200);
                        }
                    }
                }

                if (*(*ent).client).isHacking != 0 {
                    //hacking checks
                    let hacked = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).isHacking as usize);
                    let mut ang_dif: vec3_t = [0.0; 3];

                    VectorSubtract(
                        &(*(*ent).client).ps.viewangles,
                        &(*(*ent).client).hackingAngles,
                        &mut ang_dif,
                    );

                    //keep him in the "use" anim
                    if (*(*ent).client).ps.torsoAnim != BOTH_CONSOLE1 {
                        G_SetAnim(
                            ent,
                            null_mut(),
                            SETANIM_TORSO,
                            BOTH_CONSOLE1,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                    } else {
                        (*(*ent).client).ps.torsoTimer = 500;
                    }
                    (*(*ent).client).ps.weaponTime = (*(*ent).client).ps.torsoTimer;

                    if (*(*ent).client).pers.cmd.buttons & BUTTON_USE == 0 {
                        //have to keep holding use
                        (*(*ent).client).isHacking = 0;
                        (*(*ent).client).ps.hackingTime = 0;
                    } else if hacked.is_null() || (*hacked).inuse == QFALSE {
                        //shouldn't happen, but safety first
                        (*(*ent).client).isHacking = 0;
                        (*(*ent).client).ps.hackingTime = 0;
                    } else if G_PointInBounds(
                        &(*(*ent).client).ps.origin,
                        &(*hacked).r.absmin,
                        &(*hacked).r.absmax,
                    ) == QFALSE
                    {
                        //they stepped outside the thing they're hacking, so reset hacking time
                        (*(*ent).client).isHacking = 0;
                        (*(*ent).client).ps.hackingTime = 0;
                    } else if VectorLength(&ang_dif) > 10.0 {
                        //must remain facing generally the same angle as when we start
                        (*(*ent).client).isHacking = 0;
                        (*(*ent).client).ps.hackingTime = 0;
                    }
                }

                // JETPACK_DEFUEL_RATE 200 / JETPACK_REFUEL_RATE 150
                if (*(*ent).client).jetPackOn != QFALSE {
                    //using jetpack, drain fuel
                    if (*(*ent).client).jetPackDebReduce < (*lvl).time {
                        if (*(*ent).client).pers.cmd.upmove > 0 {
                            //take more if they're thrusting
                            (*(*ent).client).ps.jetpackFuel -= 2;
                        } else {
                            (*(*ent).client).ps.jetpackFuel -= 1;
                        }

                        if (*(*ent).client).ps.jetpackFuel <= 0 {
                            //turn it off
                            (*(*ent).client).ps.jetpackFuel = 0;
                            Jetpack_Off(ent);
                        }
                        (*(*ent).client).jetPackDebReduce = (*lvl).time + JETPACK_DEFUEL_RATE;
                    }
                } else if (*(*ent).client).ps.jetpackFuel < 100 {
                    //recharge jetpack
                    if (*(*ent).client).jetPackDebRecharge < (*lvl).time {
                        (*(*ent).client).ps.jetpackFuel += 1;
                        (*(*ent).client).jetPackDebRecharge = (*lvl).time + JETPACK_REFUEL_RATE;
                    }
                }

                // CLOAK_DEFUEL_RATE 200 / CLOAK_REFUEL_RATE 150
                if (*(*ent).client).ps.powerups[PW_CLOAKED as usize] != 0 {
                    //using cloak, drain battery
                    if (*(*ent).client).cloakDebReduce < (*lvl).time {
                        (*(*ent).client).ps.cloakFuel -= 1;

                        if (*(*ent).client).ps.cloakFuel <= 0 {
                            //turn it off
                            (*(*ent).client).ps.cloakFuel = 0;
                            Jedi_Decloak(ent);
                        }
                        (*(*ent).client).cloakDebReduce = (*lvl).time + CLOAK_DEFUEL_RATE;
                    }
                } else if (*(*ent).client).ps.cloakFuel < 100 {
                    //recharge cloak
                    if (*(*ent).client).cloakDebRecharge < (*lvl).time {
                        (*(*ent).client).ps.cloakFuel += 1;
                        (*(*ent).client).cloakDebRecharge = (*lvl).time + CLOAK_REFUEL_RATE;
                    }
                }

                if (*addr_of!(g_gametype)).integer == GT_SIEGE
                    && (*(*ent).client).siegeClass != -1
                    && (*addr_of!(bgSiegeClasses))[(*(*ent).client).siegeClass as usize].classflags
                        & (1 << CFL_STATVIEWER)
                        != 0
                {
                    //see if it's time to send this guy an update of extended info
                    if (*(*ent).client).siegeEDataSend < (*lvl).time {
                        G_SiegeClientExData(ent);
                        (*(*ent).client).siegeEDataSend = (*lvl).time + 1000; //once every sec seems ok
                    }
                }

                if (*lvl).intermissiontime == 0
                    && (*(*ent).client).ps.pm_flags & PMF_FOLLOW == 0
                    && (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR
                {
                    WP_ForcePowersUpdate(ent, addr_of_mut!((*(*ent).client).pers.cmd));
                    WP_SaberPositionUpdate(ent, addr_of_mut!((*(*ent).client).pers.cmd));
                    WP_SaberStartMissileBlockCheck(ent, addr_of_mut!((*(*ent).client).pers.cmd));
                }

                if (*addr_of!(g_allowNPC)).integer != 0 {
                    //This was originally intended to only be done for client 0.
                    NAV_FindPlayerWaypoint(i);
                }

                trap::ICARUS_MaintainTaskManager((*ent).s.number);

                G_RunClient(ent);
                i += 1;
                continue;
            } else if (*ent).s.eType == ET_NPC {
                // turn off any expired powerups
                let mut j = 0usize;
                while j < MAX_POWERUPS {
                    if (*(*ent).client).ps.powerups[j] < (*lvl).time {
                        (*(*ent).client).ps.powerups[j] = 0;
                    }
                    j += 1;
                }

                WP_ForcePowersUpdate(ent, addr_of_mut!((*(*ent).client).pers.cmd));
                WP_SaberPositionUpdate(ent, addr_of_mut!((*(*ent).client).pers.cmd));
                WP_SaberStartMissileBlockCheck(ent, addr_of_mut!((*(*ent).client).pers.cmd));
            }

            G_RunThink(ent);

            if (*addr_of!(g_allowNPC)).integer != 0 {
                ClearNPCGlobals();
            }

            i += 1;
        }

        SiegeCheckTimers();

        trap::ROFF_UpdateEntities();

        // perform final fixups on the players
        let mut i = 0;
        while i < (*lvl).maxclients {
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
            if (*ent).inuse != QFALSE {
                ClientEndFrame(ent);
            }
            i += 1;
        }

        // see if it is time to do a tournement restart
        CheckTournament();

        // see if it is time to end the level
        CheckExitRules();

        // update to team status?
        CheckTeamStatus();

        // cancel vote if timed out
        CheckVote();

        // check team votes
        CheckTeamVote(TEAM_RED);
        CheckTeamVote(TEAM_BLUE);

        // for tracking changes
        CheckCvars();

        if (*addr_of!(g_listEntity)).integer != 0 {
            let mut i = 0usize;
            while i < MAX_GENTITIES {
                let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i);
                G_Printf(&format!("{:4}: {}\n", i, Sz((*ent).classname)));
                i += 1;
            }
            trap::Cvar_Set("g_listEntity", "0");
        }

        //At the end of the frame, send out the ghoul2 kill queue, if there is one
        G_SendG2KillQueue();

        if *addr_of!(gQueueScoreMessage) != QFALSE {
            if *addr_of!(gQueueScoreMessageTime) < (*lvl).time {
                SendScoreboardMessageToAllClients();

                *addr_of_mut!(gQueueScoreMessageTime) = 0;
                *addr_of_mut!(gQueueScoreMessage) = QFALSE;
            }
        }

        *addr_of_mut!(g_LastFrameTime) = (*lvl).time;
    }
}

/// `#define JETPACK_DEFUEL_RATE 200` (g_main.c:3980) — jetpack idle drain debounce (~20s
/// from a full tank).
const JETPACK_DEFUEL_RATE: c_int = 200;
/// `#define JETPACK_REFUEL_RATE 150` (g_main.c:3981).
const JETPACK_REFUEL_RATE: c_int = 150;
/// `#define CLOAK_DEFUEL_RATE 200` (g_main.c:4012).
const CLOAK_DEFUEL_RATE: c_int = 200;
/// `#define CLOAK_REFUEL_RATE 150` (g_main.c:4013).
const CLOAK_REFUEL_RATE: c_int = 150;

/// `EXEC_INSERT` (q_shared.h:412) — the `EXEC_*` enum's second value (`EXEC_NOW`
/// is 0), so `1`: "insert at current position, but don't run yet". Passed to
/// `trap_SendConsoleCommand`.
const EXEC_INSERT: c_int = 1;
/// `EXEC_APPEND` (q_shared.h:413) — the `EXEC_*` enum's third value, so `2`: "add
/// to end of the command buffer (normal case)".
const EXEC_APPEND: c_int = 2;

/// `void G_KickAllBots(void)` (g_main.c:3296) — queue a `kick` console command for
/// every connected bot. Walks `level.clients[0..g_maxclients]`; for each connected
/// client whose entity carries `SVF_BOT`, copies its netname, cleans the colour
/// codes out, and inserts `kick "<name>"` into the console command buffer.
///
/// No-oracle: reads the global `level`/`g_entities` arrays and fires the
/// `trap_SendConsoleCommand` side-effect — the same global-walk + trap precedent as
/// the other `level`/`g_entities` walkers in this file; nothing computable returns.
/// Faithful 1:1; the C `char netname[36]` + `strcpy` becomes a fixed `[c_char; 36]`
/// scratch buffer, and `Q_CleanStr` keeps its in-place C shape.
///
/// SAFETY: `g_entities`/`level` are valid module statics once `G_InitGame` has run;
/// `level.clients + i` is in range for `i < g_maxclients`, and `pers.netname` is a
/// NUL-terminated C string no longer than the 36-byte scratch buffer.
pub fn G_KickAllBots() {
    unsafe {
        // char netname[36];
        let mut netname: [c_char; 36] = [0; 36];

        let clients = (*addr_of!(level)).clients;

        let mut i: c_int = 0;
        while i < (*addr_of!(g_maxclients)).integer {
            let cl = clients.offset(i as isize);
            if (*cl).pers.connected != CON_CONNECTED {
                i += 1;
                continue;
            }
            if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset((*cl).ps.clientNum as isize))
                .r
                .svFlags
                & SVF_BOT
                == 0
            {
                i += 1;
                continue;
            }
            // strcpy(netname, cl->pers.netname);
            let mut k = 0usize;
            loop {
                let b = (*cl).pers.netname[k];
                netname[k] = b;
                if b == 0 {
                    break;
                }
                k += 1;
            }
            Q_CleanStr(netname.as_mut_ptr());
            trap::SendConsoleCommand(
                EXEC_INSERT,
                &CStr::from_ptr(va(format_args!("kick \"{}\"\n", Sz(netname.as_ptr()))))
                    .to_string_lossy(),
            );
            i += 1;
        }
    }
}

/// `void CheckCvars( void )` (g_main.c:3532) — react to a changed `g_password`. The
/// first time (and any time) `g_password`'s modification count changes, it sanitises
/// the password (turning `%` into `.` so it can't break the userinfo string),
/// re-sets `g_password`, then sets `g_needpass` to `1` when a non-empty password
/// other than "none" is in effect, else `0`.
///
/// No-oracle: reads the `g_password` cvar mirror and fires `trap_Cvar_Set`
/// side-effects, with a `static int lastMod` carried across calls — the same
/// cvar-mirror + trap precedent as `G_RegisterCvars`/`G_UpdateCvars`. Faithful 1:1;
/// the C `char password[MAX_INFO_STRING]` + `strcpy`/`while(*c)` `%`→`.` pass become
/// a fixed scratch buffer and an in-place byte walk, and `Q_stricmp` keeps its C shape.
///
/// SAFETY: `g_password` is a valid module static; its `string` is a NUL-terminated C
/// string shorter than `MAX_INFO_STRING`.
pub fn CheckCvars() {
    // static int lastMod = -1;
    static mut LAST_MOD: c_int = -1;

    unsafe {
        if (*addr_of!(g_password)).modificationCount != *addr_of!(LAST_MOD) {
            // char password[MAX_INFO_STRING];
            let mut password: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];
            *addr_of_mut!(LAST_MOD) = (*addr_of!(g_password)).modificationCount;

            // strcpy( password, g_password.string );
            let mut k = 0usize;
            loop {
                let b = (*addr_of!(g_password)).string[k];
                password[k] = b;
                if b == 0 {
                    break;
                }
                k += 1;
            }
            // char *c = password; while(*c) { if (*c == '%') *c = '.'; c++; }
            let mut c = 0usize;
            while password[c] != 0 {
                if password[c] == b'%' as c_char {
                    password[c] = b'.' as c_char;
                }
                c += 1;
            }
            trap::Cvar_Set(
                "g_password",
                &CStr::from_ptr(password.as_ptr()).to_string_lossy(),
            );

            if (*addr_of!(g_password)).string[0] != 0
                && Q_stricmp((*addr_of!(g_password)).string.as_ptr(), c"none".as_ptr()) != 0
            {
                trap::Cvar_Set("g_needpass", "1");
            } else {
                trap::Cvar_Set("g_needpass", "0");
            }
        }
    }
}

/// `void CheckVote( void )` (g_main.c:3324) — resolve a pending server-wide vote each frame.
/// When a passed vote's 3s execute delay elapses, runs `level.voteString`; for a gametype vote
/// it refreshes the next map (kicking bots if voting to Siege) and auto-corrects `fraglimit`
/// to/from duel. Separately, when the vote window (`VOTE_TIME`) expires or a yes/no majority of
/// `numVotingClients` is reached, announces pass/fail (`G_GetStringEdString`) and clears the
/// vote. No oracle (`level` vote state + cvar/console/server-command traps).
///
/// # Safety
/// `level` is a valid module static; the cvar/console/server-command trap pointers must be live.
pub unsafe fn CheckVote() {
    if (*addr_of!(level)).voteExecuteTime != 0
        && (*addr_of!(level)).voteExecuteTime < (*addr_of!(level)).time
    {
        (*addr_of_mut!(level)).voteExecuteTime = 0;
        trap::SendConsoleCommand(
            EXEC_APPEND,
            &format!(
                "{}\n",
                CStr::from_ptr((*addr_of!(level)).voteString.as_ptr()).to_string_lossy()
            ),
        );

        if (*addr_of!(level)).votingGametype != QFALSE {
            if trap::Cvar_VariableIntegerValue("g_gametype") != (*addr_of!(level)).votingGametypeTo
            {
                //If we're voting to a different game type, be sure to refresh all the map stuff
                let nextMap = G_RefreshNextMap((*addr_of!(level)).votingGametypeTo, QTRUE);

                if (*addr_of!(level)).votingGametypeTo == GT_SIEGE as c_int {
                    //ok, kick all the bots, cause the aren't supported!
                    G_KickAllBots();
                    //trap_Cvar_Set("bot_minplayers", "0");
                }

                if !nextMap.is_null() && *nextMap != 0 {
                    trap::SendConsoleCommand(
                        EXEC_APPEND,
                        &format!("map {}\n", CStr::from_ptr(nextMap).to_string_lossy()),
                    );
                }
            } else {
                //otherwise, just leave the map until a restart
                G_RefreshNextMap((*addr_of!(level)).votingGametypeTo, QFALSE);
            }

            if (*addr_of!(g_fraglimitVoteCorrection)).integer != 0 {
                //This means to auto-correct fraglimit when voting to and from duel.
                let currentGT = trap::Cvar_VariableIntegerValue("g_gametype");
                let currentFL = trap::Cvar_VariableIntegerValue("fraglimit");
                let currentTL = trap::Cvar_VariableIntegerValue("timelimit");

                let votingTo = (*addr_of!(level)).votingGametypeTo;
                if (votingTo == GT_DUEL as c_int || votingTo == GT_POWERDUEL as c_int)
                    && currentGT != GT_DUEL as c_int
                    && currentGT != GT_POWERDUEL as c_int
                {
                    if currentFL > 3 || currentFL == 0 {
                        //if voting to duel, and fraglimit is more than 3 (or unlimited), set it to 3
                        trap::SendConsoleCommand(EXEC_APPEND, "fraglimit 3\n");
                    }
                    if currentTL != 0 {
                        //if voting to duel, and timelimit is set, make it unlimited
                        trap::SendConsoleCommand(EXEC_APPEND, "timelimit 0\n");
                    }
                } else if votingTo != GT_DUEL as c_int
                    && votingTo != GT_POWERDUEL as c_int
                    && (currentGT == GT_DUEL as c_int || currentGT == GT_POWERDUEL as c_int)
                    && currentFL != 0
                    && currentFL < 20
                {
                    //if voting from duel, and fraglimit is less than 20, set it up to 20
                    trap::SendConsoleCommand(EXEC_APPEND, "fraglimit 20\n");
                }
            }

            (*addr_of_mut!(level)).votingGametype = QFALSE;
            (*addr_of_mut!(level)).votingGametypeTo = 0;
        }
    }
    if (*addr_of!(level)).voteTime == 0 {
        return;
    }
    if (*addr_of!(level)).time - (*addr_of!(level)).voteTime >= VOTE_TIME {
        trap::SendServerCommand(
            -1,
            &format!(
                "print \"{}\n\"",
                CStr::from_ptr(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr(),
                    c"VOTEFAILED".as_ptr()
                ))
                .to_string_lossy()
            ),
        );
    } else if (*addr_of!(level)).voteYes > (*addr_of!(level)).numVotingClients / 2 {
        // execute the command, then remove the vote
        trap::SendServerCommand(
            -1,
            &format!(
                "print \"{}\n\"",
                CStr::from_ptr(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr(),
                    c"VOTEPASSED".as_ptr()
                ))
                .to_string_lossy()
            ),
        );
        (*addr_of_mut!(level)).voteExecuteTime = (*addr_of!(level)).time + 3000;
    } else if (*addr_of!(level)).voteNo >= (*addr_of!(level)).numVotingClients / 2 {
        // same behavior as a timeout
        trap::SendServerCommand(
            -1,
            &format!(
                "print \"{}\n\"",
                CStr::from_ptr(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr(),
                    c"VOTEFAILED".as_ptr()
                ))
                .to_string_lossy()
            ),
        );
    } else {
        // still waiting for a majority
        return;
    }
    (*addr_of_mut!(level)).voteTime = 0;
    trap::SetConfigstring(CS_VOTE_TIME, "");
}

/// `void CheckTeamVote( int team )` (g_main.c:3486) — resolve a pending team
/// (red/blue) vote. Maps `team` to a configstring slot (`0` for `TEAM_RED`, `1` for
/// `TEAM_BLUE`, ignoring anything else); if a vote is in progress it either times
/// out, passes (when yes-votes exceed half the team's voting clients), or fails —
/// and on a resolution clears the vote time and the `CS_TEAMVOTE_TIME+offset`
/// configstring.
///
/// No-oracle: reads/writes the global `level` team-vote state and fires
/// `trap_SendServerCommand`/`trap_SendConsoleCommand`/`trap_SetConfigstring`
/// side-effects — the same `level`-walk + trap precedent as `CheckTeamLeader`.
/// Faithful 1:1; the `leader` branch keeps its commented-out `SetLeader` like the C.
///
/// SAFETY: `level` is a valid module static; `cs_offset` is `0`/`1`, in range for
/// the two-element `teamVote*` arrays, and `teamVoteString[cs_offset]` is a
/// NUL-terminated C string.
pub fn CheckTeamVote(team: c_int) {
    unsafe {
        let cs_offset: usize;

        if team == TEAM_RED {
            cs_offset = 0;
        } else if team == TEAM_BLUE {
            cs_offset = 1;
        } else {
            return;
        }

        if (*addr_of!(level)).teamVoteTime[cs_offset] == 0 {
            return;
        }
        if (*addr_of!(level)).time - (*addr_of!(level)).teamVoteTime[cs_offset] >= VOTE_TIME {
            trap::SendServerCommand(
                -1,
                &CStr::from_ptr(va(format_args!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr(),
                        c"TEAMVOTEFAILED".as_ptr()
                    ))
                )))
                .to_string_lossy(),
            );
        } else if (*addr_of!(level)).teamVoteYes[cs_offset]
            > (*addr_of!(level)).numteamVotingClients[cs_offset] / 2
        {
            // execute the command, then remove the vote
            trap::SendServerCommand(
                -1,
                &CStr::from_ptr(va(format_args!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr(),
                        c"TEAMVOTEPASSED".as_ptr()
                    ))
                )))
                .to_string_lossy(),
            );
            //
            if Q_strncmp(
                c"leader".as_ptr(),
                (*addr_of!(level)).teamVoteString[cs_offset].as_ptr(),
                6,
            ) == 0
            {
                //set the team leader
                //SetLeader(team, atoi(level.teamVoteString[cs_offset] + 7));
            } else {
                trap::SendConsoleCommand(
                    EXEC_APPEND,
                    &CStr::from_ptr(va(format_args!(
                        "{}\n",
                        Sz((*addr_of!(level)).teamVoteString[cs_offset].as_ptr())
                    )))
                    .to_string_lossy(),
                );
            }
        } else if (*addr_of!(level)).teamVoteNo[cs_offset]
            >= (*addr_of!(level)).numteamVotingClients[cs_offset] / 2
        {
            // same behavior as a timeout
            trap::SendServerCommand(
                -1,
                &CStr::from_ptr(va(format_args!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr(),
                        c"TEAMVOTEFAILED".as_ptr()
                    ))
                )))
                .to_string_lossy(),
            );
        } else {
            // still waiting for a majority
            return;
        }
        (*addr_of_mut!(level)).teamVoteTime[cs_offset] = 0;
        trap::SetConfigstring(CS_TEAMVOTE_TIME + cs_offset as c_int, "");
    }
}

/// `void NAV_CheckCalcPaths( void )` (g_main.c:3599) — the deferred one-time nav-path build.
/// Once [`navCalcPathTime`] elapses (set after map load so all ents are linked), registers the
/// `mapname`/`sv_mapChecksum` cvars, clears failed edges, runs the module-side
/// [`NAV_CalculatePaths`] + engine-side `trap_Nav_CalculatePaths`, then (unless `fatalErrors`)
/// saves the `.nav` file. No oracle (cvars + nav traps + file I/O).
///
/// # Safety
/// Reads/writes the module statics (`navCalcPathTime`, `level`, `fatalErrors`); the cvar/nav
/// trap pointers must be live.
pub unsafe fn NAV_CheckCalcPaths() {
    if navCalcPathTime != 0 && navCalcPathTime < (*addr_of!(level)).time {
        //first time we've ever loaded this map...
        // `mapname` is a function-local cvar in the C source (g_main.c:3526).
        let mut mapnameCv: vmCvar_t = vmCvar_t::zeroed();
        let mut ckSum: vmCvar_t = vmCvar_t::zeroed();

        trap::Cvar_Register(
            Some(&mut mapnameCv),
            "mapname",
            "",
            CVAR_SERVERINFO | CVAR_ROM,
        );
        trap::Cvar_Register(Some(&mut ckSum), "sv_mapChecksum", "", CVAR_ROM);

        //clear all the failed edges
        trap::Nav_ClearAllFailedEdges();

        //Calculate all paths
        NAV_CalculatePaths(mapnameCv.string.as_ptr(), ckSum.integer);

        trap::Nav_CalculatePaths(QFALSE);

        // #ifndef FINAL_BUILD
        if fatalErrors != 0 {
            Com_Printf(&format!(
                "{S_COLOR_RED}Not saving .nav file due to fatal nav errors\n"
            ));
        }
        // #endif  /  #ifndef _XBOX
        else if trap::Nav_Save(
            &CStr::from_ptr(mapnameCv.string.as_ptr()).to_string_lossy(),
            ckSum.integer,
        ) == QFALSE
        {
            Com_Printf(&format!(
                "Unable to save navigations data for map \"{}\" (checksum:{})\n",
                CStr::from_ptr(mapnameCv.string.as_ptr()).to_string_lossy(),
                ckSum.integer
            ));
        }
        navCalcPathTime = 0;
    }
}

//so shared code can get the local time depending on the side it's executed on
/// `int BG_GetTime(void)` (g_main.c:3636). On the game (server) side the local
/// time is simply `level.time`. In C the body is wrapped in
/// `namespace_begin.h`/`namespace_end.h` (the bg shared-code namespace); that is a
/// build-time linkage detail with no Rust analogue.
///
/// No-oracle: a one-line read of the mutable `level` global; there is no
/// self-contained C body to extract (it just returns a field of the game-state
/// global). Faithful 1:1.
///
/// SAFETY: `level` is a valid module static.
pub fn BG_GetTime() -> c_int {
    unsafe { (*addr_of!(level)).time }
}

#[cfg(test)]
mod tests {
    /// Parity: the hand-built [`gameCvarTable`] matches the authentic C table in
    /// `oracle/g_main_oracle.c` (verbatim from g_main.c) row-for-row — count plus
    /// every data field (name, default, flags, modificationCount, trackChange,
    /// teamShader). Catches a dropped/added row, a mis-OR'd flag, and the
    /// field-slot quirks (e.g. d_powerDuelPrint's `qtrue` in modificationCount).
    /// The `vmCvar` pointer column is intentionally not compared (Rust and C
    /// point at different globals).
    #[cfg(feature = "oracle")]
    #[test]
    fn cvar_table_matches_c() {
        use crate::oracle::*;
        use core::ffi::CStr;

        unsafe {
            let n = jka_gameCvarTableSize();
            assert_eq!(n as usize, super::gameCvarTableSize, "table length");

            let tbl = core::ptr::addr_of!(super::gameCvarTable);
            for i in 0..n {
                let row = (*tbl)[i as usize];
                let name = CStr::from_ptr(jka_cvar_name(i));
                assert_eq!(CStr::from_ptr(row.cvarName), name, "cvarName at row {i}");
                // gamedate's C default is __DATE__ (the compiler build date), which
                // the Rust port leaves empty (see DEVIATIONS.md); skip its default.
                if name != c"gamedate" {
                    assert_eq!(
                        CStr::from_ptr(row.defaultString),
                        CStr::from_ptr(jka_cvar_default(i)),
                        "defaultString at row {i}"
                    );
                }
                assert_eq!(row.cvarFlags, jka_cvar_flags(i), "cvarFlags at row {i}");
                assert_eq!(
                    row.modificationCount,
                    jka_cvar_modcount(i),
                    "modificationCount at row {i}"
                );
                assert_eq!(row.trackChange, jka_cvar_track(i), "trackChange at row {i}");
                assert_eq!(row.teamShader, jka_cvar_team(i), "teamShader at row {i}");
            }
        }
    }

    /// Parity: [`G_GetStringEdString`] emits the same `@@@<refName>` token as the
    /// real C (`oracle/g_main_oracle.c`). Covers a normal refName, the empty
    /// string (`@@@` alone), and confirms `refSection` is ignored. Both sides
    /// return a pointer into their own 1024-byte static, so the bytes are compared.
    #[cfg(feature = "oracle")]
    #[test]
    fn g_get_string_ed_string_matches_c() {
        use crate::oracle::jka_G_GetStringEdString;
        use core::ffi::CStr;

        unsafe {
            for (sect, name) in [
                (c"MP_SVGAME", c"JOINEDTHEREDTEAM"),
                (c"WHATEVER", c"JOINEDTHEBATTLE"),
                (c"X", c""),
            ] {
                let rust = super::G_GetStringEdString(sect.as_ptr(), name.as_ptr());
                let c = jka_G_GetStringEdString(
                    sect.as_ptr() as *mut core::ffi::c_char,
                    name.as_ptr() as *mut core::ffi::c_char,
                );
                assert_eq!(
                    CStr::from_ptr(rust),
                    CStr::from_ptr(c),
                    "G_GetStringEdString for {name:?}"
                );
            }
        }
    }
}
