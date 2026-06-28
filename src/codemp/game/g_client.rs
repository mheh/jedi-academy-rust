//! Port of `g_client.c` — client connect / spawn / view-angle setup. Landed
//! incrementally: only the functions whose full dep-set is already ported. Currently
//! carries the view-angle setter, the full spawn-point selector family (DM / furthest /
//! nearest / random / duel / initial), the `SP_info_player_*` spawn-class registrars, the
//! body-queue infra (`BodySink`/`InitBodyQue`/`CopyToBodyQue`/`MaintainBodyQueue`), the
//! `SelectSpectatorSpawnPoint` intermission placer, the team-count helpers, the `PickTeam`
//! balancer, the `ClientCleanName` name sanitiser, the `ClientUserinfoChanged`/`G_SaberModelSetup`
//! info-string + saber-model setup, and the `G_BreakArm` limb-break helper (a `G_Damage` callee).
//! The remaining bulk (client connect/spawn infra, the JM-saber entity) is not yet ported, pending its
//! supporting subsystems (the JM-saber think/touch, `ClientSpawn` and friends).

#![allow(non_snake_case)] // C function names (`SetClientViewAngle`, …) kept verbatim
#![allow(non_upper_case_globals)] // file-local statics keep their C names (playerMins, playerMaxs)

use crate::codemp::game::ai_main::BotAIShutdownClient;
use crate::codemp::game::anims::{BOTH_PAIN2, BOTH_PAIN3};
use crate::codemp::game::anims::{BOTH_STAND1TO2, TORSO_RAISEWEAP1};
use crate::codemp::game::bg_lib;
use crate::codemp::game::bg_misc::{
    BG_IsValidCharacterModel, BG_PlayerStateToEntityState, BG_ValidateSkinForTeam, WeaponReadyAnim,
};
use crate::codemp::game::bg_panimate::{
    bgAllAnims, bgHumanoidAnimations, BGPAFtextLoaded, BG_ParseAnimationFile,
    BG_SaberStartTransAnim,
};
use crate::codemp::game::bg_public::{
    bgEntity_t, team_t, BROKENLIMB_LARM, BROKENLIMB_RARM, CROUCH_MAXS_2, CS_CLIENT_JEDIMASTER,
    CS_PLAYERS, DEFAULT_MAXS_2, DEFAULT_MINS_2, DUELTEAM_DOUBLE, DUELTEAM_FREE, DUELTEAM_LONE,
    DUELTEAM_SINGLE, EF2_SHIP_DEATH, EF_DEAD, EF_DISINTEGRATION, EF_DOUBLE_AMMO, EF_INVULNERABLE,
    EF_NODRAW, EF_TELEPORT_BIT, ET_BODY, ET_GENERAL, ET_MISSILE, EV_BECOME_JEDIMASTER, EV_BODYFADE,
    EV_CLIENTJOIN, EV_PLAYER_TELEPORT_IN, EV_PLAYER_TELEPORT_OUT, EV_SIEGESPEC, GIB_HEALTH, GT_CTF,
    GT_CTY, GT_DUEL, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM, MASK_PLAYERSOLID,
    MASK_SOLID, PERS_SCORE, PERS_SPAWN_COUNT, PERS_TEAM, PMF_RESPAWNED, PMF_TIME_KNOCKBACK,
    PW_NUM_POWERUPS, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_HOLDLESS, SETANIM_FLAG_OVERRIDE,
    SETANIM_TORSO, STAT_ARMOR, STAT_HEALTH, STAT_HOLDABLE_ITEM, STAT_HOLDABLE_ITEMS,
    STAT_MAX_HEALTH, STAT_WEAPONS, TEAM_BLUE, TEAM_FREE, TEAM_NUM_TEAMS, TEAM_RED, TEAM_SPECTATOR,
    WEAPON_RAISING,
};
use crate::codemp::game::bg_saberLoad::{WP_SaberStyleValidForSaber, WP_UseFirstValidSaberStyle};
use crate::codemp::game::bg_saga::{
    bgSiegeClasses, BG_SiegeCheckClassLegality, BG_SiegeFindClassIndexByName,
};
use crate::codemp::game::bg_saga_h::siegeClass_t;
use crate::codemp::game::bg_saga_h::{CFL_EXTRA_AMMO, CFL_SINGLE_ROCKET, SIEGETEAM_TEAM1};
use crate::codemp::game::bg_vehicleLoad::BG_GetVehicleModelName;
use crate::codemp::game::bg_vehicles_h::{
    MAX_VEHICLE_EXHAUSTS, MAX_VEHICLE_MUZZLES, MAX_VEHICLE_TURRET_MUZZLES,
};
use crate::codemp::game::bg_weapons::{ammoData, weaponData};
use crate::codemp::game::bg_weapons_h::{
    AMMO_BLASTER, AMMO_POWERCELL, WP_BLASTER, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_MELEE, WP_NONE,
    WP_NUM_WEAPONS, WP_ROCKET_LAUNCHER, WP_SABER,
};
use crate::codemp::game::g_active::{ClientEndFrame, ClientThink};
use crate::codemp::game::g_bot::{G_BotConnect, G_RemoveQueuedBotBegin};
use crate::codemp::game::g_cmds::{
    BroadcastTeamChange, G_SetSaber, SetTeam, SetTeamQuick, StopFollowing,
};
use crate::codemp::game::g_combat::body_die;
use crate::codemp::game::g_combat::player_die;
use crate::codemp::game::g_combat::TossClientItems;
use crate::codemp::game::g_local::{
    clientPersistant_t, clientSession_t, gclient_t, gentity_t, BODY_QUEUE_SIZE, CON_CONNECTED,
    CON_CONNECTING, CON_DISCONNECTED, FL_BOUNCE_HALF, FL_NO_BOTS, FL_NO_HUMANS, HL_MAX,
    PSG_TEAMVOTED, PSG_VOTED, SPECTATOR_FOLLOW, SPECTATOR_FREE, SPECTATOR_SCOREBOARD, TEAM_ACTIVE,
    TEAM_BEGIN,
};
use crate::codemp::game::g_log::G_ClearClientLog;
use crate::codemp::game::g_main::{
    d_perPlayerGhoul2, g_duelWeaponDisable, g_duel_fraglimit, g_entities, g_forcePowerDisable,
    g_gametype, g_inactivity, g_logClientInfo, g_needpass, g_password, g_powerDuelEndHealth,
    g_powerDuelStartHealth, g_siegeRespawn, g_siegeRespawnCheck, g_spawnInvulnerability,
    g_trueJedi, g_weaponDisable, level, CalculateRanks, Com_Error, Com_Printf,
    FindIntermissionPoint, G_Error, G_GetStringEdString, G_LogPrintf, MoveClientToIntermission,
};
use crate::codemp::game::g_misc::gEscaping;
use crate::codemp::game::g_object::G_RunObject;
use crate::codemp::game::g_public_h::Q3_INFINITE;
use crate::codemp::game::g_public_h::{SVF_BOT, SVF_BROADCAST};
use crate::codemp::game::g_saga::{
    gSiegeRoundBegun, gSiegeRoundEnded, G_ValidateSiegeClassForTeam, SiegeRespawn,
};
use crate::codemp::game::g_session::{
    G_InitSessionData, G_ReadSessionData, G_WriteClientSessionData,
};
use crate::codemp::game::g_spawn::{precachedKyle, G_SpawnInt};
use crate::codemp::game::g_svcmds::G_FilterPacket;
use crate::codemp::game::g_team::{SelectCTFSpawnPoint, SelectSiegeSpawnPoint, TeamName};
use crate::codemp::game::g_utils::{
    G_AddEvent, G_EntitySound, G_Find, G_FreeEntity, G_InitGentity, G_KillBox, G_KillG2Queue,
    G_ModelIndex, G_MuteSound, G_PlayerHasCustomSkeleton, G_SetAnim, G_SetOrigin, G_Sound,
    G_SoundIndex, G_Spawn, G_TempEntity, G_UseTargets,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    vec3_origin, VectorAdd, VectorCopy, VectorLength, VectorNormalize, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{
    random, va, Com_sprintf, Info_SetValueForKey, Info_Validate, Info_ValueForKey, Q_stricmp,
    Q_strncpyz, Q_strrchr, Sz,
};
use crate::codemp::game::q_shared_h::ANGLE2SHORT;
use crate::codemp::game::q_shared_h::MAX_SABERS;
use crate::codemp::game::q_shared_h::SFL_BOLT_TO_WRIST;
use crate::codemp::game::q_shared_h::SFL_TWO_HANDED;
use crate::codemp::game::q_shared_h::{
    forcedata_t, saberInfo_t, trace_t, usercmd_t, vec3_t, CHAN_AUTO, CHAN_VOICE, ENTITYNUM_NONE,
    ERR_DROP, FORCE_LEVEL_3, FORCE_LIGHTSIDE, FP_SABER_OFFENSE, MAX_CLIENTS, MAX_GENTITIES,
    MAX_INFO_STRING, MAX_INFO_VALUE, MAX_PERSISTANT, MAX_QPATH, MAX_STRING_CHARS, NEGATIVE_Y,
    NEGATIVE_Z, NUM_FORCE_POWERS, NUM_TRACK_CHANNELS, PITCH, POSITIVE_X, POSITIVE_Z,
    Q_COLOR_ESCAPE, ROLL, SS_DUAL, SS_FAST, SS_STAFF, SS_STRONG, TRACK_CHANNEL_1, TR_GRAVITY,
    TR_STATIONARY,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_CORPSE, CONTENTS_NODROP, CONTENTS_PLAYERCLIP, CONTENTS_SOLID,
    CONTENTS_TRIGGER,
};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::codemp::game::teams_h::{NPCTEAM_ENEMY, NPCTEAM_PLAYER};
use crate::codemp::game::w_force::{
    WP_ForcePowerStop, WP_HasForcePowers, WP_InitForcePowers, WP_SpawnInitForcePowers,
};
use crate::codemp::game::w_saber::{HasSetSaberOnly, WP_SaberAddG2Model, WP_SaberInitBladeData};
use crate::codemp::ghoul2::g2_h::{
    BONE_ANGLES_POSTMULT, BONE_ANIM_BLEND, BONE_ANIM_OVERRIDE_FREEZE, BONE_ANIM_OVERRIDE_LOOP,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::c_uint;
use core::ffi::c_void;
use core::ffi::CStr;
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

extern "C" {
    /// libc `int atoi( const char * )` — the retail (non-`Q3_VM`) build links the C
    /// library's `atoi` (the `g_session.rs` / `g_cmds.rs` precedent).
    fn atoi(s: *const c_char) -> c_int;
    /// libc `char *strcpy( char *, const char * )`.
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    /// libc `int strcmp( const char *, const char * )`.
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    /// libc `char *strcat( char *, const char * )`.
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    /// libc `char *strchr( const char *, int )`.
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    /// libc `char *strstr( const char *, const char * )`.
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
}

// g_client.c:16-17 — file-local player bounding box used by the spawn-spot telefrag test.
static playerMins: vec3_t = [-15.0, -15.0, DEFAULT_MINS_2 as f32];
static playerMaxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2 as f32];

// g_client.c:861 — body-queue corpse sink delay.
const BODY_SINK_TIME: c_int = 30000; //45000

/*QUAKED info_player_duel (1 0 1) (-16 -16 -24) (16 16 32) initial
potential spawning position for duelists in duel.
*/
/// `void SP_info_player_duel( gentity_t *ent )` (g_client.c:32). Reads the optional `nobots`
/// / `nohumans` spawn keys and sets the matching `FL_NO_BOTS` / `FL_NO_HUMANS` flags so the
/// spawn-spot selectors can exclude this point. No oracle (spawn-key reads + entity flags).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_duel(ent: *mut gentity_t) {
    let mut i: c_int = 0;

    G_SpawnInt(c"nobots".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        (*ent).flags |= FL_NO_BOTS;
    }
    G_SpawnInt(c"nohumans".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        (*ent).flags |= FL_NO_HUMANS;
    }
}

/*QUAKED info_player_duel1 (1 0 1) (-16 -16 -24) (16 16 32) initial
potential spawning position for lone duelists in powerduel.
*/
/// `void SP_info_player_duel1( gentity_t *ent )` (g_client.c:52). Power-duel lone-duelist
/// start; identical body to [`SP_info_player_duel`] (carried verbatim, not aliased, to match
/// the C). No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_duel1(ent: *mut gentity_t) {
    let mut i: c_int = 0;

    G_SpawnInt(c"nobots".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        (*ent).flags |= FL_NO_BOTS;
    }
    G_SpawnInt(c"nohumans".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        (*ent).flags |= FL_NO_HUMANS;
    }
}

/*QUAKED info_player_duel2 (1 0 1) (-16 -16 -24) (16 16 32) initial
potential spawning position for paired duelists in powerduel.
*/
/// `void SP_info_player_duel2( gentity_t *ent )` (g_client.c:72). Power-duel paired-duelist
/// start; identical body to [`SP_info_player_duel`] (carried verbatim, not aliased, to match
/// the C). No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_duel2(ent: *mut gentity_t) {
    let mut i: c_int = 0;

    G_SpawnInt(c"nobots".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        (*ent).flags |= FL_NO_BOTS;
    }
    G_SpawnInt(c"nohumans".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        (*ent).flags |= FL_NO_HUMANS;
    }
}

/*QUAKED info_player_deathmatch (1 0 1) (-16 -16 -24) (16 16 32) initial
potential spawning position for deathmatch games.
*/
/// `void SP_info_player_deathmatch( gentity_t *ent )` (g_client.c:93). DM spawn point; reads
/// the `nobots` / `nohumans` keys and sets the matching flags. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_deathmatch(ent: *mut gentity_t) {
    let mut i: c_int = 0;

    G_SpawnInt(c"nobots".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        (*ent).flags |= FL_NO_BOTS;
    }
    G_SpawnInt(c"nohumans".as_ptr(), c"0".as_ptr(), &mut i);
    if i != 0 {
        (*ent).flags |= FL_NO_HUMANS;
    }
}

/*QUAKED info_player_start (1 0 0) (-16 -16 -24) (16 16 32)
equivelant to info_player_deathmatch
*/
/// `void SP_info_player_start( gentity_t *ent )` (g_client.c:110). Aliases the entity to the
/// `info_player_deathmatch` class and delegates to [`SP_info_player_deathmatch`]. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_start(ent: *mut gentity_t) {
    (*ent).classname = c"info_player_deathmatch".as_ptr() as *mut _;
    SP_info_player_deathmatch(ent);
}

/*QUAKED info_player_start_red (1 0 0) (-16 -16 -24) (16 16 32) INITIAL
For Red Team DM starts
equivalent to info_player_deathmatch
*/
/// `void SP_info_player_start_red( gentity_t *ent )` (g_client.c:121). Red-team DM start;
/// delegates straight to [`SP_info_player_deathmatch`] (no classname alias, matching the C).
/// No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_start_red(ent: *mut gentity_t) {
    SP_info_player_deathmatch(ent);
}

/*QUAKED info_player_start_blue (1 0 0) (-16 -16 -24) (16 16 32) INITIAL
For Blue Team DM starts
equivalent to info_player_deathmatch
*/
/// `void SP_info_player_start_blue( gentity_t *ent )` (g_client.c:136). Blue-team DM start;
/// delegates straight to [`SP_info_player_deathmatch`] (no classname alias, matching the C).
/// No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_start_blue(ent: *mut gentity_t) {
    SP_info_player_deathmatch(ent);
}

/// `void SiegePointUse( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_client.c:114). Toggles a siege spawn point on/off via `genericValue1`. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SiegePointUse(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    //Toggle the point on/off
    if (*self_).genericValue1 != 0 {
        (*self_).genericValue1 = 0;
    } else {
        (*self_).genericValue1 = 1;
    }
}

/*QUAKED info_player_siegeteam1 (1 0 0) (-16 -16 -24) (16 16 32)
siege start point - team1.
startoff - if non-0 spawn point will be disabled until used.
*/
/// `void SP_info_player_siegeteam1( gentity_t *ent )` (g_client.c:139). Outside Siege this
/// degrades to a DM spawn; in Siege it reads `startoff` to seed `genericValue1` (enabled
/// state) and installs [`SiegePointUse`] as its `use` callback. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_siegeteam1(ent: *mut gentity_t) {
    let mut soff: c_int = 0;

    if (*addr_of!(g_gametype)).integer != GT_SIEGE {
        //turn into a DM spawn if not in siege game mode
        (*ent).classname = c"info_player_deathmatch".as_ptr() as *mut _;
        SP_info_player_deathmatch(ent);

        return;
    }

    G_SpawnInt(c"startoff".as_ptr(), c"0".as_ptr(), &mut soff);

    if soff != 0 {
        //start disabled
        (*ent).genericValue1 = 0;
    } else {
        (*ent).genericValue1 = 1;
    }

    (*ent).r#use = Some(SiegePointUse);
}

/*QUAKED info_player_siegeteam2 (0 0 1) (-16 -16 -24) (16 16 32)
siege start point - team2.
startoff - if non-0 spawn point will be disabled until used.
*/
/// `void SP_info_player_siegeteam2( gentity_t *ent )` (g_client.c:175). Team2 sibling of
/// [`SP_info_player_siegeteam1`] (carried verbatim, not aliased, to match the C). No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_siegeteam2(ent: *mut gentity_t) {
    let mut soff: c_int = 0;

    if (*addr_of!(g_gametype)).integer != GT_SIEGE {
        //turn into a DM spawn if not in siege game mode
        (*ent).classname = c"info_player_deathmatch".as_ptr() as *mut _;
        SP_info_player_deathmatch(ent);

        return;
    }

    G_SpawnInt(c"startoff".as_ptr(), c"0".as_ptr(), &mut soff);

    if soff != 0 {
        //start disabled
        (*ent).genericValue1 = 0;
    } else {
        (*ent).genericValue1 = 1;
    }

    (*ent).r#use = Some(SiegePointUse);
}

/*QUAKED info_player_intermission (1 0 1) (-16 -16 -24) (16 16 32)
The intermission will be viewed from this point.
*/
/// `void SP_info_player_intermission( gentity_t *ent )` (g_client.c:203). Empty in the C —
/// the entity is only a positional marker the intermission camera reads. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_intermission(_ent: *mut gentity_t) {}

/*QUAKED info_player_intermission_red (1 0 1) (-16 -16 -24) (16 16 32)
In a Siege game, the intermission will happen here if the Red (attacking) team wins.
*/
/// `void SP_info_player_intermission_red( gentity_t *ent )` (g_client.c:241). Empty in the C —
/// a positional marker the intermission camera reads when Red wins. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_intermission_red(_ent: *mut gentity_t) {}

/*QUAKED info_player_intermission_blue (1 0 1) (-16 -16 -24) (16 16 32)
In a Siege game, the intermission will happen here if the Blue (defending) team wins.
*/
/// `void SP_info_player_intermission_blue( gentity_t *ent )` (g_client.c:252). Empty in the C —
/// a positional marker the intermission camera reads when Blue wins. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_info_player_intermission_blue(_ent: *mut gentity_t) {}

/// `void SetClientViewAngle( gentity_t *ent, vec3_t angle )` (g_client.c:1025). Sets the
/// player's view to `angle`: derives the per-axis delta angle (the offset the client adds
/// to its command angles to reach the server's view) and mirrors `angle` into both
/// `s.angles` and `ps.viewangles`. Oracle-tested bit-exact via [`crate::oracle::jka_SetClientViewAngle`].
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-NULL.
pub unsafe fn SetClientViewAngle(ent: *mut gentity_t, angle: &vec3_t) {
    // set the delta angle
    for i in 0..3 {
        let cmdAngle = ANGLE2SHORT(angle[i]);
        (*(*ent).client).ps.delta_angles[i] = cmdAngle - (*(*ent).client).pers.cmd.angles[i];
    }
    VectorCopy(angle, &mut (*ent).s.angles);
    VectorCopy(&(*ent).s.angles, &mut (*(*ent).client).ps.viewangles);
}

/// `qboolean SpotWouldTelefrag( gentity_t *spot )` (g_client.c:483). Tests whether spawning
/// at `spot` would overlap a player: builds the `playerMins`/`playerMaxs` box around the
/// spot origin, queries the engine for entities in it, and returns `qtrue` the moment any
/// returned entity has a non-NULL `client`. (The original health check is commented out in
/// the C — a *connected* client at the spot is enough to reject it.) No oracle (queries the
/// engine via `trap_EntitiesInBox` and walks the global `g_entities` array).
///
/// # Safety
/// `spot` must point to a valid `gentity_t`; the `g_entities` global must be initialised.
pub unsafe fn SpotWouldTelefrag(spot: *mut gentity_t) -> qboolean {
    let mut touch: [i32; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    VectorAdd(&(*spot).s.origin, &playerMins, &mut mins);
    VectorAdd(&(*spot).s.origin, &playerMaxs, &mut maxs);
    let num = trap::EntitiesInBox(&mins, &maxs, &mut touch);

    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    for i in 0..num as usize {
        let hit = base.add(touch[i] as usize);
        //if ( hit->client && hit->client->ps.stats[STAT_HEALTH] > 0 ) {
        if !(*hit).client.is_null() {
            return QTRUE;
        }
    }

    QFALSE
}

/// `qboolean SpotWouldTelefrag2( gentity_t *mover, vec3_t dest )` (g_client.c:505). Tests
/// whether moving `mover` to `dest` would overlap a solid: builds `mover`'s bounds box at
/// `dest`, queries the engine, and returns `qtrue` for the first non-`mover` hit whose
/// `r.contents` shares a bit with `mover`'s. No oracle (engine query + `g_entities` walk).
///
/// # Safety
/// `mover` must point to a valid `gentity_t`; the `g_entities` global must be initialised.
pub unsafe fn SpotWouldTelefrag2(mover: *mut gentity_t, dest: &vec3_t) -> qboolean {
    let mut touch: [i32; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    VectorAdd(dest, &(*mover).r.mins, &mut mins);
    VectorAdd(dest, &(*mover).r.maxs, &mut maxs);
    let num = trap::EntitiesInBox(&mins, &maxs, &mut touch);

    let base = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    for i in 0..num as usize {
        let hit = base.add(touch[i] as usize);
        if hit == mover {
            continue;
        }

        if (*hit).r.contents & (*mover).r.contents != 0 {
            return QTRUE;
        }
    }

    QFALSE
}

/// `gentity_t *SelectNearestDeathmatchSpawnPoint( vec3_t from )` (g_client.c:541). Walks all
/// `info_player_deathmatch` spots and returns the one nearest `from` (smallest origin
/// distance), or NULL if there are none. No oracle (walks `g_entities` via `G_Find`).
///
/// # Safety
/// The `g_entities`/`level` globals must be initialised.
pub unsafe fn SelectNearestDeathmatchSpawnPoint(from: &vec3_t) -> *mut gentity_t {
    let mut delta: vec3_t = [0.0; 3];
    let mut nearestDist: f32 = 999999.0;
    let mut nearestSpot: *mut gentity_t = null_mut();
    let mut spot: *mut gentity_t = null_mut();

    loop {
        spot = G_Find(
            spot,
            offset_of!(gentity_t, classname),
            c"info_player_deathmatch".as_ptr(),
        );
        if spot.is_null() {
            break;
        }

        VectorSubtract(&(*spot).s.origin, from, &mut delta);
        let dist = VectorLength(&delta);
        if dist < nearestDist {
            nearestDist = dist;
            nearestSpot = spot;
        }
    }

    nearestSpot
}

/// `gentity_t *SelectRandomDeathmatchSpawnPoint( void )` (g_client.c:573). Collects up to
/// `MAX_SPAWN_POINTS` non-telefragging DM spots and returns a uniformly random one; if none
/// are clear it falls back to the very first DM spot. No oracle (entity walk + `rand()`).
///
/// # Safety
/// The `g_entities`/`level` globals must be initialised.
pub unsafe fn SelectRandomDeathmatchSpawnPoint() -> *mut gentity_t {
    const MAX_SPAWN_POINTS: usize = 128;
    let mut count: c_int = 0;
    let mut spot: *mut gentity_t = null_mut();
    let mut spots: [*mut gentity_t; MAX_SPAWN_POINTS] = [null_mut(); MAX_SPAWN_POINTS];

    loop {
        spot = G_Find(
            spot,
            offset_of!(gentity_t, classname),
            c"info_player_deathmatch".as_ptr(),
        );
        if spot.is_null() {
            break;
        }
        if SpotWouldTelefrag(spot) == QTRUE {
            continue;
        }
        spots[count as usize] = spot;
        count += 1;
    }

    if count == 0 {
        // no spots that won't telefrag
        return G_Find(
            null_mut(),
            offset_of!(gentity_t, classname),
            c"info_player_deathmatch".as_ptr(),
        );
    }

    let selection = bg_lib::rand() % count;
    spots[selection as usize]
}

/// `gentity_t *SelectRandomFurthestSpawnPoint ( vec3_t avoidPoint, vec3_t origin, vec3_t angles, team_t team )`
/// (g_client.c:654). Chooses a spawn far from `avoidPoint`. In Team DM (`g_gametype == GT_TEAM`,
/// `team` a real team) it first searches the team-start spots (`info_player_start_red`/`_blue`)
/// and only falls back to `info_player_deathmatch` if none were found; either way it collects up
/// to 64 non-telefragging spots ordered by descending distance (an insertion sort into
/// `list_dist`/`list_spot`), then picks one at random from the furthest half
/// (`random() * (numSpots / 2)`), writing its origin (raised 9 units) and angles into
/// `origin`/`angles` and returning it. If no clear spot exists it falls back to the first DM
/// spot, erroring out if there is none. No oracle (walks `g_entities`, calls
/// `SpotWouldTelefrag`/`G_Find`, and pulls a `random()` draw).
///
/// # Safety
/// The `g_entities`/`level` globals must be initialised; `origin`/`angles` must be valid.
pub unsafe fn SelectRandomFurthestSpawnPoint(
    avoidPoint: &vec3_t,
    origin: &mut vec3_t,
    angles: &mut vec3_t,
    team: team_t,
) -> *mut gentity_t {
    let mut delta: vec3_t = [0.0; 3];
    let mut list_dist: [f32; 64] = [0.0; 64];
    let mut list_spot: [*mut gentity_t; 64] = [null_mut(); 64];
    let mut numSpots: i32 = 0;
    let mut i: i32;

    let mut spot: *mut gentity_t = null_mut();

    //in Team DM, look for a team start spot first, if any
    if (*addr_of!(g_gametype)).integer == GT_TEAM && team != TEAM_FREE && team != TEAM_SPECTATOR {
        let classname: *const core::ffi::c_char = if team == TEAM_RED {
            c"info_player_start_red".as_ptr()
        } else {
            c"info_player_start_blue".as_ptr()
        };
        loop {
            spot = G_Find(spot, offset_of!(gentity_t, classname), classname);
            if spot.is_null() {
                break;
            }
            if SpotWouldTelefrag(spot) == QTRUE {
                continue;
            }
            VectorSubtract(&(*spot).s.origin, avoidPoint, &mut delta);
            let dist = VectorLength(&delta);
            i = 0;
            while i < numSpots {
                if dist > list_dist[i as usize] {
                    if numSpots >= 64 {
                        numSpots = 64 - 1;
                    }
                    let mut j = numSpots;
                    while j > i {
                        list_dist[j as usize] = list_dist[(j - 1) as usize];
                        list_spot[j as usize] = list_spot[(j - 1) as usize];
                        j -= 1;
                    }
                    list_dist[i as usize] = dist;
                    list_spot[i as usize] = spot;
                    numSpots += 1;
                    if numSpots > 64 {
                        numSpots = 64;
                    }
                    break;
                }
                i += 1;
            }
            if i >= numSpots && numSpots < 64 {
                list_dist[numSpots as usize] = dist;
                list_spot[numSpots as usize] = spot;
                numSpots += 1;
            }
        }
    }

    if numSpots == 0 {
        //couldn't find any of the above
        loop {
            spot = G_Find(
                spot,
                offset_of!(gentity_t, classname),
                c"info_player_deathmatch".as_ptr(),
            );
            if spot.is_null() {
                break;
            }
            if SpotWouldTelefrag(spot) == QTRUE {
                continue;
            }
            VectorSubtract(&(*spot).s.origin, avoidPoint, &mut delta);
            let dist = VectorLength(&delta);
            i = 0;
            while i < numSpots {
                if dist > list_dist[i as usize] {
                    if numSpots >= 64 {
                        numSpots = 64 - 1;
                    }
                    let mut j = numSpots;
                    while j > i {
                        list_dist[j as usize] = list_dist[(j - 1) as usize];
                        list_spot[j as usize] = list_spot[(j - 1) as usize];
                        j -= 1;
                    }
                    list_dist[i as usize] = dist;
                    list_spot[i as usize] = spot;
                    numSpots += 1;
                    if numSpots > 64 {
                        numSpots = 64;
                    }
                    break;
                }
                i += 1;
            }
            if i >= numSpots && numSpots < 64 {
                list_dist[numSpots as usize] = dist;
                list_spot[numSpots as usize] = spot;
                numSpots += 1;
            }
        }
        if numSpots == 0 {
            spot = G_Find(
                null_mut(),
                offset_of!(gentity_t, classname),
                c"info_player_deathmatch".as_ptr(),
            );
            if spot.is_null() {
                G_Error("Couldn't find a spawn point");
            }
            VectorCopy(&(*spot).s.origin, origin);
            origin[2] += 9.0;
            VectorCopy(&(*spot).s.angles, angles);
            return spot;
        }
    }

    // select a random spot from the spawn points furthest away
    let rnd = (random() * (numSpots / 2) as f32) as usize;

    VectorCopy(&(*list_spot[rnd]).s.origin, origin);
    origin[2] += 9.0;
    VectorCopy(&(*list_spot[rnd]).s.angles, angles);

    list_spot[rnd]
}

/// `gentity_t *SelectSpawnPoint ( vec3_t avoidPoint, vec3_t origin, vec3_t angles, team_t team )`
/// (g_client.c:854). Chooses a player/deathmatch start. The body is a single delegation to
/// [`SelectRandomFurthestSpawnPoint`], forwarding `team`; the original nearest-vs-random
/// selection is commented out in the C. No oracle (delegates to an entity-walking selector).
///
/// # Safety
/// The `g_entities`/`level` globals must be initialised; `origin`/`angles` must be valid.
pub unsafe fn SelectSpawnPoint(
    avoidPoint: &vec3_t,
    origin: &mut vec3_t,
    angles: &mut vec3_t,
    team: team_t,
) -> *mut gentity_t {
    SelectRandomFurthestSpawnPoint(avoidPoint, origin, angles, team)

    /*
    gentity_t	*spot;
    gentity_t	*nearestSpot;

    nearestSpot = SelectNearestDeathmatchSpawnPoint( avoidPoint );

    spot = SelectRandomDeathmatchSpawnPoint ( );
    if ( spot == nearestSpot ) {
        // roll again if it would be real close to point of death
        spot = SelectRandomDeathmatchSpawnPoint ( );
        if ( spot == nearestSpot ) {
            // last try
            spot = SelectRandomDeathmatchSpawnPoint ( );
        }
    }

    // find a single player start spot
    if (!spot) {
        G_Error( "Couldn't find a spawn point" );
    }

    VectorCopy (spot->s.origin, origin);
    origin[2] += 9;
    VectorCopy (spot->s.angles, angles);

    return spot;
    */
}

/// `gentity_t *SelectDuelSpawnPoint( int team, vec3_t avoidPoint, vec3_t origin, vec3_t angles )`
/// (g_client.c:664). Picks a (power)duel start: chooses the classname by `team`
/// (`info_player_duel1`/`2`/`info_player_duel`, else DM), collects up to 64 non-telefragging
/// spots furthest-first, and randomly picks from the far half. If a duel class yields nothing
/// it retries (`goto tryAgain`) with `info_player_deathmatch`; if even that is empty it falls
/// back to the first DM spot or `G_Error`s. Writes origin (+9)/angles. No oracle (entity walk
/// + `random()`).
///
/// # Safety
/// The `g_entities`/`level` globals must be initialised; `origin`/`angles` must be valid.
pub unsafe fn SelectDuelSpawnPoint(
    team: c_int,
    avoidPoint: &vec3_t,
    origin: &mut vec3_t,
    angles: &mut vec3_t,
) -> *mut gentity_t {
    let mut delta: vec3_t = [0.0; 3];
    let mut list_dist: [f32; 64] = [0.0; 64];
    let mut list_spot: [*mut gentity_t; 64] = [null_mut(); 64];
    let mut numSpots: i32;
    let mut i: i32;

    let mut spotName: *const core::ffi::c_char = if team == DUELTEAM_LONE {
        c"info_player_duel1".as_ptr()
    } else if team == DUELTEAM_DOUBLE {
        c"info_player_duel2".as_ptr()
    } else if team == DUELTEAM_SINGLE {
        c"info_player_duel".as_ptr()
    } else {
        c"info_player_deathmatch".as_ptr()
    };

    // tryAgain:
    loop {
        numSpots = 0;
        let mut spot: *mut gentity_t = null_mut();

        loop {
            spot = G_Find(spot, offset_of!(gentity_t, classname), spotName);
            if spot.is_null() {
                break;
            }
            if SpotWouldTelefrag(spot) == QTRUE {
                continue;
            }
            VectorSubtract(&(*spot).s.origin, avoidPoint, &mut delta);
            let dist = VectorLength(&delta);
            i = 0;
            while i < numSpots {
                if dist > list_dist[i as usize] {
                    if numSpots >= 64 {
                        numSpots = 64 - 1;
                    }
                    let mut j = numSpots;
                    while j > i {
                        list_dist[j as usize] = list_dist[(j - 1) as usize];
                        list_spot[j as usize] = list_spot[(j - 1) as usize];
                        j -= 1;
                    }
                    list_dist[i as usize] = dist;
                    list_spot[i as usize] = spot;
                    numSpots += 1;
                    if numSpots > 64 {
                        numSpots = 64;
                    }
                    break;
                }
                i += 1;
            }
            if i >= numSpots && numSpots < 64 {
                list_dist[numSpots as usize] = dist;
                list_spot[numSpots as usize] = spot;
                numSpots += 1;
            }
        }
        if numSpots == 0 {
            if Q_stricmp(spotName, c"info_player_deathmatch".as_ptr()) != 0 {
                //try the loop again with info_player_deathmatch as the target if we couldn't find a duel spot
                spotName = c"info_player_deathmatch".as_ptr();
                continue; // goto tryAgain
            }

            //If we got here we found no free duel or DM spots, just try the first DM spot
            let spot0 = G_Find(
                null_mut(),
                offset_of!(gentity_t, classname),
                c"info_player_deathmatch".as_ptr(),
            );
            if spot0.is_null() {
                G_Error("Couldn't find a spawn point");
            }
            VectorCopy(&(*spot0).s.origin, origin);
            origin[2] += 9.0;
            VectorCopy(&(*spot0).s.angles, angles);
            return spot0;
        }

        // select a random spot from the spawn points furthest away
        let rnd = (random() * (numSpots / 2) as f32) as usize;

        VectorCopy(&(*list_spot[rnd]).s.origin, origin);
        origin[2] += 9.0;
        VectorCopy(&(*list_spot[rnd]).s.angles, angles);

        return list_spot[rnd];
    }
}

/// `gentity_t *SelectInitialSpawnPoint( vec3_t origin, vec3_t angles, team_t team )`
/// (g_client.c:894). Tries to find a DM spot flagged 'initial' (`spawnflags & 1`); if none
/// exists or it would telefrag, defers to [`SelectSpawnPoint`] from the world origin,
/// forwarding `team`. Writes origin (+9)/angles. No oracle (entity walk + selector delegation).
///
/// # Safety
/// The `g_entities`/`level` globals must be initialised; `origin`/`angles` must be valid.
pub unsafe fn SelectInitialSpawnPoint(
    origin: &mut vec3_t,
    angles: &mut vec3_t,
    team: team_t,
) -> *mut gentity_t {
    let mut spot: *mut gentity_t = null_mut();

    loop {
        spot = G_Find(
            spot,
            offset_of!(gentity_t, classname),
            c"info_player_deathmatch".as_ptr(),
        );
        if spot.is_null() {
            break;
        }
        if (*spot).spawnflags & 1 != 0 {
            break;
        }
    }

    if spot.is_null() || SpotWouldTelefrag(spot) == QTRUE {
        return SelectSpawnPoint(&vec3_origin, origin, angles, team);
    }

    VectorCopy(&(*spot).s.origin, origin);
    origin[2] += 9.0;
    VectorCopy(&(*spot).s.angles, angles);

    spot
}

/// `gentity_t *SelectSpectatorSpawnPoint( vec3_t origin, vec3_t angles )`
/// (g_client.c) — place a spectator at the intermission camera. Delegates to
/// [`FindIntermissionPoint`] (g_main) to populate `level.intermission_origin` /
/// `intermission_angle`, copies those out, and always returns `NULL` (no entity).
///
/// No-oracle: writes the `level` global through `FindIntermissionPoint`. Faithful
/// 1:1. Unblocked now that `FindIntermissionPoint` has landed.
///
/// SAFETY: `level` is a valid module static; the out-params are caller-owned.
pub unsafe fn SelectSpectatorSpawnPoint(
    origin: &mut vec3_t,
    angles: &mut vec3_t,
) -> *mut gentity_t {
    FindIntermissionPoint();

    VectorCopy(&(*addr_of!(level)).intermission_origin, origin);
    VectorCopy(&(*addr_of!(level)).intermission_angle, angles);

    null_mut()
}

/// `void BodySink( gentity_t *ent )` (g_client.c:877). Body-queue corpse think: after the
/// sink delay it unlinks the (never-freed) body and clears its physics flag; otherwise it
/// fires the `EV_BODYFADE` event, reschedules far out, and disables further damage. No oracle
/// (engine syscalls + global `level`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn BodySink(ent: *mut gentity_t) {
    if (*addr_of!(level)).time - (*ent).timestamp > BODY_SINK_TIME + 2500 {
        // the body ques are never actually freed, they are just unlinked
        trap::UnlinkEntity(ent);
        (*ent).physicsObject = QFALSE;
        return;
    }
    //	ent->nextthink = level.time + 100;
    //	ent->s.pos.trBase[2] -= 1;

    G_AddEvent(ent, EV_BODYFADE, 0);
    (*ent).nextthink = (*addr_of!(level)).time + 18000;
    (*ent).takedamage = QFALSE;
}

/// `void InitBodyQue (void)` (g_client.c:857). Spawns the `BODY_QUEUE_SIZE` never-freed
/// "bodyque" corpse placeholders and resets the rotating index. No oracle (spawns entities
/// via `G_Spawn` and mutates the global `level`).
///
/// # Safety
/// The `level`/`g_entities` globals must be initialised.
pub unsafe fn InitBodyQue() {
    (*addr_of_mut!(level)).bodyQueIndex = 0;
    for i in 0..BODY_QUEUE_SIZE {
        let ent = G_Spawn();
        (*ent).classname = c"bodyque".as_ptr() as *mut c_char;
        (*ent).neverFree = QTRUE;
        (*addr_of_mut!(level)).bodyQue[i] = ent;
    }
}

/// `static qboolean CopyToBodyQue( gentity_t *ent )` (g_client.c:900). A player is respawning,
/// so make a corpse entity that looks like the existing one to leave behind. Bails out during
/// intermission, in a nodrop area, or when disintegrated; otherwise grabs the next body-queue
/// slot, copies the entity state, sets up corpse physics/contents/think, broadcasts the `ircg`
/// reliable command, and links it. No oracle (engine syscalls + global `level`/`g_entities`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-null; the `level`/`g_entities`
/// globals must be initialised.
unsafe fn CopyToBodyQue(ent: *mut gentity_t) -> qboolean {
    let mut islight = 0;

    if (*addr_of!(level)).intermissiontime != 0 {
        return QFALSE;
    }

    trap::UnlinkEntity(ent);

    // if client is in a nodrop area, don't leave the body
    let contents = trap::PointContents(&(*ent).s.origin, -1);
    if contents & CONTENTS_NODROP != 0 {
        return QFALSE;
    }

    if !(*ent).client.is_null() && (*(*ent).client).ps.eFlags & EF_DISINTEGRATION != 0 {
        //for now, just don't spawn a body if you got disint'd
        return QFALSE;
    }

    // grab a body que and cycle to the next one
    let body = (*addr_of!(level)).bodyQue[(*addr_of!(level)).bodyQueIndex as usize];
    (*addr_of_mut!(level)).bodyQueIndex =
        ((*addr_of!(level)).bodyQueIndex + 1) % BODY_QUEUE_SIZE as c_int;

    trap::UnlinkEntity(body);
    (*body).s = (*ent).s;

    //avoid oddly angled corpses floating around
    (*body).s.angles[PITCH] = 0.0;
    (*body).s.angles[ROLL] = 0.0;
    (*body).s.apos.trBase[PITCH] = 0.0;
    (*body).s.apos.trBase[ROLL] = 0.0;

    (*body).s.g2radius = 100;

    (*body).s.eType = ET_BODY;
    (*body).s.eFlags = EF_DEAD; // clear EF_TALK, etc

    if !(*ent).client.is_null() && (*(*ent).client).ps.eFlags & EF_DISINTEGRATION != 0 {
        (*body).s.eFlags |= EF_DISINTEGRATION;
    }

    VectorCopy(&(*(*ent).client).ps.lastHitLoc, &mut (*body).s.origin2);

    (*body).s.powerups = 0; // clear powerups
    (*body).s.loopSound = 0; // clear lava burning
    (*body).s.loopIsSoundset = QFALSE;
    (*body).s.number =
        body.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;
    (*body).timestamp = (*addr_of!(level)).time;
    (*body).physicsObject = QTRUE;
    (*body).physicsBounce = 0.0; // don't bounce
    if (*body).s.groundEntityNum == ENTITYNUM_NONE {
        (*body).s.pos.trType = TR_GRAVITY;
        (*body).s.pos.trTime = (*addr_of!(level)).time;
        VectorCopy(&(*(*ent).client).ps.velocity, &mut (*body).s.pos.trDelta);
    } else {
        (*body).s.pos.trType = TR_STATIONARY;
    }
    (*body).s.event = 0;

    (*body).s.weapon = (*ent).s.bolt2;

    if (*body).s.weapon == WP_SABER && (*(*ent).client).ps.saberInFlight != 0 {
        (*body).s.weapon = WP_BLASTER; //lie to keep from putting a saber on the corpse, because it was thrown at death
    }

    //G_AddEvent(body, EV_BODY_QUEUE_COPY, ent->s.clientNum);
    //Now doing this through a modified version of the rcg reliable command.
    if !(*ent).client.is_null() && (*(*ent).client).ps.fd.forceSide == FORCE_LIGHTSIDE {
        islight = 1;
    }
    trap::SendServerCommand(
        -1,
        &format!(
            "ircg {} {} {} {}",
            (*ent).s.number,
            (*body).s.number,
            (*body).s.weapon,
            islight
        ),
    );

    (*body).r.svFlags = (*ent).r.svFlags | SVF_BROADCAST;
    VectorCopy(&(*ent).r.mins, &mut (*body).r.mins);
    VectorCopy(&(*ent).r.maxs, &mut (*body).r.maxs);
    VectorCopy(&(*ent).r.absmin, &mut (*body).r.absmin);
    VectorCopy(&(*ent).r.absmax, &mut (*body).r.absmax);

    (*body).s.torsoAnim = (*(*ent).client).ps.legsAnim;
    (*body).s.legsAnim = (*(*ent).client).ps.legsAnim;

    (*body).s.customRGBA[0] = (*(*ent).client).ps.customRGBA[0];
    (*body).s.customRGBA[1] = (*(*ent).client).ps.customRGBA[1];
    (*body).s.customRGBA[2] = (*(*ent).client).ps.customRGBA[2];
    (*body).s.customRGBA[3] = (*(*ent).client).ps.customRGBA[3];

    (*body).clipmask = CONTENTS_SOLID | CONTENTS_PLAYERCLIP;
    (*body).r.contents = CONTENTS_CORPSE;
    (*body).r.ownerNum = (*ent).s.number;

    (*body).nextthink = (*addr_of!(level)).time + BODY_SINK_TIME;
    (*body).think = Some(BodySink);

    (*body).die = Some(body_die);

    // don't take more damage if already gibbed
    if (*ent).health <= GIB_HEALTH {
        (*body).takedamage = QFALSE;
    } else {
        (*body).takedamage = QTRUE;
    }

    VectorCopy(&(*body).s.pos.trBase, &mut (*body).r.currentOrigin);
    trap::LinkEntity(body);

    QTRUE
}

/// `void MaintainBodyQueue(gentity_t *ent)` (g_client.c:1039). On respawn, decide whether to
/// leave a corpse — taking ragdoll/dismemberment (`noCorpse`, `fallingToDeath`, ship-death,
/// temp-spectate) into account — by calling [`CopyToBodyQue`]; if no body was produced, fire
/// the `rcg` reliable command so the client fixes up limb/ragdoll state. No oracle (engine
/// syscalls + global `level`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-null; the `level` global must
/// be initialised.
pub unsafe fn MaintainBodyQueue(ent: *mut gentity_t) {
    //do whatever should be done taking ragdoll and dismemberment states into account.
    let mut doRCG = QFALSE;

    debug_assert!(!ent.is_null() && !(*ent).client.is_null());
    if (*(*ent).client).tempSpectate > (*addr_of!(level)).time
        || (*(*ent).client).ps.eFlags2 & EF2_SHIP_DEATH != 0
    {
        (*(*ent).client).noCorpse = QTRUE;
    }

    if (*(*ent).client).noCorpse == QFALSE && (*(*ent).client).ps.fallingToDeath == QFALSE {
        if CopyToBodyQue(ent) == QFALSE {
            doRCG = QTRUE;
        }
    } else {
        (*(*ent).client).noCorpse = QFALSE; //clear it for next time
        (*(*ent).client).ps.fallingToDeath = QFALSE;
        doRCG = QTRUE;
    }

    if doRCG == QTRUE {
        //bodyque func didn't manage to call ircg so call this to assure our limbs and ragdoll states are proper on the client.
        trap::SendServerCommand(-1, &format!("rcg {}", (*ent).s.clientNum));
    }
}

/// `team_t TeamCount( int ignoreClientNum, int team )` (g_client.c:1146). Counts connected
/// clients on `team` (skipping `ignoreClientNum`), also counting Siege players whose
/// `siegeDesiredTeam` matches. No oracle (walks the global `level.clients`).
///
/// # Safety
/// The `level` global must be initialised.
pub unsafe fn TeamCount(ignoreClientNum: c_int, team: c_int) -> team_t {
    let mut count: team_t = 0;

    let clients = (*addr_of!(level)).clients;
    let maxclients = (*addr_of!(level)).maxclients;
    for i in 0..maxclients {
        if i == ignoreClientNum {
            continue;
        }
        let cl = clients.offset(i as isize);
        if (*cl).pers.connected == CON_DISCONNECTED {
            continue;
        }
        if (*cl).sess.sessionTeam == team {
            count += 1;
        } else if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*cl).sess.siegeDesiredTeam == team
        {
            count += 1;
        }
    }

    count
}

/// `int TeamLeader( int team )` (g_client.c:1177). Returns the client number of `team`'s
/// leader, or -1 if none. No oracle (walks the global `level.clients`).
///
/// # Safety
/// The `level` global must be initialised.
pub unsafe fn TeamLeader(team: c_int) -> c_int {
    let clients = (*addr_of!(level)).clients;
    let maxclients = (*addr_of!(level)).maxclients;
    for i in 0..maxclients {
        let cl = clients.offset(i as isize);
        if (*cl).pers.connected == CON_DISCONNECTED {
            continue;
        }
        if (*cl).sess.sessionTeam == team && (*cl).sess.teamLeader != QFALSE {
            return i;
        }
    }

    -1
}

/// `static qboolean AllForceDisabled(int force)` (g_client.c:2577). Returns `qtrue` only when
/// `force` is non-zero and every one of the `NUM_FORCE_POWERS` low bits is set (i.e. all force
/// powers disabled). No oracle (pure bitwise).
pub fn AllForceDisabled(force: c_int) -> qboolean {
    if force != 0 {
        for i in 0..NUM_FORCE_POWERS {
            if force & (1 << i) == 0 {
                return QFALSE;
            }
        }

        return QTRUE;
    }

    QFALSE
}

/// `team_t PickTeam( int ignoreClientNum )` (g_client.c:1200). Returns the team a new player
/// should join: the smaller of `TeamCount(TEAM_RED)`/`TeamCount(TEAM_BLUE)`, breaking a tie
/// toward the team with the lower `level.teamScores`. No oracle (reads the global `level`).
///
/// # Safety
/// The `level` global must be initialised (via `TeamCount` and `level.teamScores`).
pub unsafe fn PickTeam(ignoreClientNum: c_int) -> team_t {
    let mut counts: [team_t; TEAM_NUM_TEAMS as usize] = [0; TEAM_NUM_TEAMS as usize];

    counts[TEAM_BLUE as usize] = TeamCount(ignoreClientNum, TEAM_BLUE);
    counts[TEAM_RED as usize] = TeamCount(ignoreClientNum, TEAM_RED);

    if counts[TEAM_BLUE as usize] > counts[TEAM_RED as usize] {
        return TEAM_RED;
    }
    if counts[TEAM_RED as usize] > counts[TEAM_BLUE as usize] {
        return TEAM_BLUE;
    }
    // equal team count, so join the team with the lowest score
    if (*addr_of!(level)).teamScores[TEAM_BLUE as usize]
        > (*addr_of!(level)).teamScores[TEAM_RED as usize]
    {
        return TEAM_RED;
    }
    TEAM_BLUE
}

/// `static void ClientCleanName( const char *in, char *out, int outSize )` (g_client.c:1244).
/// Sanitises a player name into `out`: strips leading spaces, caps consecutive spaces at 3,
/// drops black (`^0`) color codes, keeps other `^N` codes, and substitutes `"Padawan"` for an
/// empty/colorless result. `ColorIndex(c)` is the inlined `((c) - '0') & 7` macro.
///
/// # Safety
/// `in` must be a valid NUL-terminated C string; `out` must point to at least `outSize`
/// writable bytes (`outSize >= 1`).
pub unsafe fn ClientCleanName(mut in_: *const c_char, out: *mut c_char, outSize: c_int) {
    let mut len: c_int;
    let mut ch: c_char;
    let mut spaces: c_int;

    // save room for trailing null byte
    let out_size = outSize - 1;

    len = 0;
    let mut colorless_len: c_int = 0;
    let p = out;
    let mut out = out;
    *p = 0;
    spaces = 0;

    loop {
        ch = *in_;
        in_ = in_.add(1);
        if ch == 0 {
            break;
        }

        // don't allow leading spaces
        if *p == 0 && ch == b' ' as c_char {
            continue;
        }

        // check colors
        if ch == Q_COLOR_ESCAPE {
            // solo trailing carat is not a color prefix
            if *in_ == 0 {
                break;
            }

            // don't allow black in a name, period
            if ((*in_ - b'0' as c_char) & 7) == 0 {
                in_ = in_.add(1);
                continue;
            }

            // make sure room in dest for both chars
            if len > out_size - 2 {
                break;
            }

            *out = ch;
            out = out.add(1);
            *out = *in_;
            out = out.add(1);
            in_ = in_.add(1);
            len += 2;
            continue;
        }

        // don't allow too many consecutive spaces
        if ch == b' ' as c_char {
            spaces += 1;
            if spaces > 3 {
                continue;
            }
        } else {
            spaces = 0;
        }

        if len > out_size - 1 {
            break;
        }

        *out = ch;
        out = out.add(1);
        colorless_len += 1;
        len += 1;
    }
    *out = 0;

    // don't allow empty names
    if *p == 0 || colorless_len == 0 {
        Q_strncpyz(p, c"Padawan".as_ptr(), out_size);
    }
}

/// `void G_DebugWrite(const char *path, const char *text)` (g_client.c:1322).
///
/// A `#ifdef _DEBUG`-only helper that appends `text` to the file at `path` via the
/// engine filesystem traps. Gated under `#[cfg(debug_assertions)]` to mirror the C
/// `#ifdef _DEBUG` guard — it is absent from the retail (non-debug) build, as in C.
///
/// **No oracle** — pure engine-syscall I/O side effect (`trap_FS_FOpenFile`/
/// `trap_FS_Write`/`trap_FS_FCloseFile`), nothing to assert bit-exact.
///
/// The `trap` wrappers are Rust-idiomatic: `FS_FOpenFile` takes a `&str` path and
/// returns `(length, handle)` (the length is discarded here, as the C return is too),
/// and `FS_Write` takes a byte slice whose length supplies the C `len`. The C
/// `strlen(text)` byte count is exactly `CStr::to_bytes()` (NUL-terminator excluded).
#[cfg(debug_assertions)]
pub unsafe fn G_DebugWrite(path: *const c_char, text: *const c_char) {
    use crate::codemp::game::q_shared_h::FS_APPEND;
    let (_len, f) = trap::FS_FOpenFile(&CStr::from_ptr(path).to_string_lossy(), FS_APPEND);
    trap::FS_Write(CStr::from_ptr(text).to_bytes(), f);
    trap::FS_FCloseFile(f);
}

/// `void ClientUserinfoChanged( int clientNum )` (g_client.c:1787).
///
/// Called on connect and whenever a client's userinfo changes. Re-reads the engine
/// userinfo, sanitizes the name, applies the player skin RGB, resolves the Siege class
/// + sabers, computes max-health, and finally publishes the `CS_PLAYERS+clientNum`
/// configstring subset (name/team/model/colors/wins/losses/saber names/duel team) that
/// other clients use to draw scoreboards and models.
///
/// **No oracle** — engine syscalls (`trap_GetUserinfo`/`trap_SetConfigstring`) +
/// global-state mutation (the configstring-family precedent).
///
/// Deviations from original JKA (all precedented):
/// - The commented-out name-change/`CON_CONNECTED` block (g_client.c:1842-1861) is kept
///   as a source-order comment (compiled out in C too).
/// - The Xbox `if (com_dedicated && com_dedicated->integer) XBL_PL_UpdatePlayerName(...)`
///   block (g_client.c:2189-2192) is dropped: `XBL_PL_UpdatePlayerName` is `_XBOX`-only
///   and OpenJK has no such block — this is not the Xbox build (`com_dedicated` is not in
///   the module).
/// - The `if (modelChanged) { SetupGameGhoul2Model(...); ... }` block (g_client.c:2194) is
///   now ported (its `SetupGameGhoul2Model` blocker has landed). It only fires for allowable
///   server-side custom-skeleton cases (`modelChanged` becomes true only via the
///   `d_perPlayerGhoul2` default-0 cheat cvar or a custom skeleton).
///
/// # Safety
/// `clientNum` must be a valid client index; `g_entities`/`level`/the cvar globals must
/// be initialised.
pub unsafe fn ClientUserinfoChanged(clientNum: c_int) {
    let mut model_buf = [0 as c_char; MAX_QPATH];
    let model = model_buf.as_mut_ptr();
    let mut forcePowers = [0 as c_char; MAX_QPATH];
    let mut oldname = [0 as c_char; MAX_STRING_CHARS];
    let mut c1 = [0 as c_char; MAX_INFO_STRING];
    let mut c2 = [0 as c_char; MAX_INFO_STRING];
    //	char	redTeam[MAX_INFO_STRING];
    //	char	blueTeam[MAX_INFO_STRING];
    let mut userinfo = [0 as c_char; MAX_INFO_STRING];
    let mut className = [0 as c_char; MAX_QPATH]; // name of class type to use in siege
    let mut saberName = [0 as c_char; MAX_QPATH];
    let mut saber2Name = [0 as c_char; MAX_QPATH];
    let mut modelChanged: qboolean = QFALSE;

    let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(clientNum as usize);
    let client = (*ent).client;

    // trap_GetUserinfo( clientNum, userinfo, sizeof( userinfo ) );
    {
        let info = trap::GetUserinfo(clientNum);
        let bytes = info.as_bytes();
        let n = bytes.len().min(MAX_INFO_STRING - 1);
        for k in 0..n {
            userinfo[k] = bytes[k] as c_char;
        }
        userinfo[n] = 0;
    }

    // check for malformed or illegal info strings
    if Info_Validate(userinfo.as_ptr()) == QFALSE {
        strcpy(userinfo.as_mut_ptr(), c"\\name\\badinfo".as_ptr());
    }

    // check for local client
    let mut s = Info_ValueForKey(userinfo.as_ptr(), c"ip".as_ptr());
    if strcmp(s, c"localhost".as_ptr()) == 0 {
        (*client).pers.localClient = QTRUE;
    }

    // check the item prediction
    s = Info_ValueForKey(userinfo.as_ptr(), c"cg_predictItems".as_ptr());
    if atoi(s) == 0 {
        (*client).pers.predictItemPickup = QFALSE;
    } else {
        (*client).pers.predictItemPickup = QTRUE;
    }

    // set name
    Q_strncpyz(
        oldname.as_mut_ptr(),
        (*client).pers.netname.as_ptr(),
        oldname.len() as c_int,
    );
    s = Info_ValueForKey(userinfo.as_ptr(), c"name".as_ptr());
    ClientCleanName(
        s,
        (*client).pers.netname.as_mut_ptr(),
        (*client).pers.netname.len() as c_int,
    );

    if (*client).sess.sessionTeam == TEAM_SPECTATOR {
        if (*client).sess.spectatorState == SPECTATOR_SCOREBOARD {
            Q_strncpyz(
                (*client).pers.netname.as_mut_ptr(),
                c"scoreboard".as_ptr(),
                (*client).pers.netname.len() as c_int,
            );
        }
    }
    if (*client).pers.connected == CON_CONNECTED {
        if strcmp(oldname.as_ptr(), (*client).pers.netname.as_ptr()) != 0 {
            if (*client).pers.netnameTime > (*addr_of!(level)).time {
                trap::SendServerCommand(
                    clientNum,
                    &format!(
                        "print \"{}\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr() as *mut c_char,
                            c"NONAMECHANGE".as_ptr() as *mut c_char,
                        )),
                    ),
                );

                Info_SetValueForKey(userinfo.as_mut_ptr(), c"name".as_ptr(), oldname.as_ptr());
                trap::SetUserinfo(
                    clientNum,
                    &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy(),
                );
                strcpy((*client).pers.netname.as_mut_ptr(), oldname.as_ptr());
            } else {
                trap::SendServerCommand(
                    -1,
                    &format!(
                        "print \"{}^7 {} {}\n\"",
                        Sz(oldname.as_ptr()),
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr() as *mut c_char,
                            c"PLRENAME".as_ptr() as *mut c_char,
                        )),
                        Sz((*client).pers.netname.as_ptr()),
                    ),
                );
                (*client).pers.netnameTime = (*addr_of!(level)).time + 5000;
            }
        }
    }

    // set model
    Q_strncpyz(
        model,
        Info_ValueForKey(userinfo.as_ptr(), c"model".as_ptr()),
        model_buf.len() as c_int,
    );

    if (*addr_of!(d_perPlayerGhoul2)).integer != 0 {
        if Q_stricmp(model, (*client).modelname.as_ptr()) != 0 {
            strcpy((*client).modelname.as_mut_ptr(), model);
            modelChanged = QTRUE;
        }
    }

    // Get the skin RGB based on his userinfo
    let mut value = Info_ValueForKey(userinfo.as_ptr(), c"char_color_red".as_ptr());
    if !value.is_null() {
        (*client).ps.customRGBA[0] = atoi(value);
    } else {
        (*client).ps.customRGBA[0] = 255;
    }

    value = Info_ValueForKey(userinfo.as_ptr(), c"char_color_green".as_ptr());
    if !value.is_null() {
        (*client).ps.customRGBA[1] = atoi(value);
    } else {
        (*client).ps.customRGBA[1] = 255;
    }

    value = Info_ValueForKey(userinfo.as_ptr(), c"char_color_blue".as_ptr());
    if !value.is_null() {
        (*client).ps.customRGBA[2] = atoi(value);
    } else {
        (*client).ps.customRGBA[2] = 255;
    }

    if ((*client).ps.customRGBA[0] + (*client).ps.customRGBA[1] + (*client).ps.customRGBA[2]) < 100
    {
        // hmm, too dark!
        (*client).ps.customRGBA[0] = 255;
        (*client).ps.customRGBA[1] = 255;
        (*client).ps.customRGBA[2] = 255;
    }

    (*client).ps.customRGBA[3] = 255;

    Q_strncpyz(
        forcePowers.as_mut_ptr(),
        Info_ValueForKey(userinfo.as_ptr(), c"forcepowers".as_ptr()),
        forcePowers.len() as c_int,
    );
    let _ = forcePowers;

    // bots set their team a few frames later
    let team: c_int;
    if (*addr_of!(g_gametype)).integer >= GT_TEAM
        && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(clientNum as usize))
            .r
            .svFlags
            & SVF_BOT
            != 0
    {
        s = Info_ValueForKey(userinfo.as_ptr(), c"team".as_ptr());
        if Q_stricmp(s, c"red".as_ptr()) == 0 || Q_stricmp(s, c"r".as_ptr()) == 0 {
            team = TEAM_RED;
        } else if Q_stricmp(s, c"blue".as_ptr()) == 0 || Q_stricmp(s, c"b".as_ptr()) == 0 {
            team = TEAM_BLUE;
        } else {
            // pick the team with the least number of players
            team = PickTeam(clientNum);
        }
    } else {
        team = (*client).sess.sessionTeam;
    }

    // Set the siege class
    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        strcpy(className.as_mut_ptr(), (*client).sess.siegeClass.as_ptr());

        // This function will see if the given class is legal for the given team.
        // If not className will be filled in with the first legal class for this team.
        /*		if (!BG_SiegeCheckClassLegality(team, className) &&
            Q_stricmp(client->sess.siegeClass, "none"))
        { //if it isn't legal pop up the class menu
            trap_SendServerCommand(ent-g_entities, "scl");
        }
        */
        // Now that the team is legal for sure, we'll go ahead and get an index for it.
        (*client).siegeClass = BG_SiegeFindClassIndexByName(className.as_ptr());
        if (*client).siegeClass == -1 {
            // ok, get the first valid class for the team you're on then, I guess.
            BG_SiegeCheckClassLegality(team, className.as_mut_ptr());
            strcpy((*client).sess.siegeClass.as_mut_ptr(), className.as_ptr());
            (*client).siegeClass = BG_SiegeFindClassIndexByName(className.as_ptr());
        } else {
            // otherwise, make sure the class we are using is legal.
            G_ValidateSiegeClassForTeam(ent, team);
            strcpy(className.as_mut_ptr(), (*client).sess.siegeClass.as_ptr());
        }

        // Set the sabers if the class dictates
        if (*client).siegeClass != -1 {
            let scl = (addr_of_mut!(bgSiegeClasses) as *mut siegeClass_t)
                .add((*client).siegeClass as usize);

            if (*scl).saber1[0] != 0 {
                G_SetSaber(ent, 0, (*scl).saber1.as_mut_ptr(), QTRUE);
            } else {
                // default I guess
                G_SetSaber(ent, 0, c"Kyle".as_ptr() as *mut c_char, QTRUE);
            }
            if (*scl).saber2[0] != 0 {
                G_SetSaber(ent, 1, (*scl).saber2.as_mut_ptr(), QTRUE);
            } else {
                // no second saber then
                G_SetSaber(ent, 1, c"none".as_ptr() as *mut c_char, QTRUE);
            }

            // make sure the saber models are updated
            G_SaberModelSetup(ent);

            if (*scl).forcedModel[0] != 0 {
                // be sure to override the model we actually use
                strcpy(model, (*scl).forcedModel.as_ptr());
                if (*addr_of!(d_perPlayerGhoul2)).integer != 0 {
                    if Q_stricmp(model, (*client).modelname.as_ptr()) != 0 {
                        strcpy((*client).modelname.as_mut_ptr(), model);
                        modelChanged = QTRUE;
                    }
                }
            }

            // force them to use their class model on the server, if the class dictates
            if G_PlayerHasCustomSkeleton(ent) == QTRUE {
                if Q_stricmp(model, (*client).modelname.as_ptr()) != 0 || (*ent).localAnimIndex == 0
                {
                    strcpy((*client).modelname.as_mut_ptr(), model);
                    modelChanged = QTRUE;
                }
            }
        }
    } else {
        strcpy(className.as_mut_ptr(), c"none".as_ptr());
    }

    // Set the saber name
    strcpy(saberName.as_mut_ptr(), (*client).sess.saberType.as_ptr());
    strcpy(saber2Name.as_mut_ptr(), (*client).sess.saber2Type.as_ptr());

    // set max health
    let maxHealth: c_int;
    let health: c_int;
    if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).siegeClass != -1 {
        let scl =
            (addr_of_mut!(bgSiegeClasses) as *mut siegeClass_t).add((*client).siegeClass as usize);
        let mut mh = 100;

        if (*scl).maxhealth != 0 {
            mh = (*scl).maxhealth;
        }

        maxHealth = mh;
        health = mh;
    } else {
        maxHealth = 100;
        health = 100; // atoi( Info_ValueForKey( userinfo, "handicap" ) );
    }
    (*client).pers.maxHealth = health;
    if (*client).pers.maxHealth < 1 || (*client).pers.maxHealth > maxHealth {
        (*client).pers.maxHealth = 100;
    }
    (*client).ps.stats[STAT_MAX_HEALTH as usize] = (*client).pers.maxHealth;

    /*	NOTE: all client side now

        // team
        switch( team ) {
        case TEAM_RED:
            ForceClientSkin(client, model, "red");
    //		ForceClientSkin(client, headModel, "red");
            break;
        case TEAM_BLUE:
            ForceClientSkin(client, model, "blue");
    //		ForceClientSkin(client, headModel, "blue");
            break;
        }
        // don't ever use a default skin in teamplay, it would just waste memory
        // however bots will always join a team but they spawn in as spectator
        if ( g_gametype.integer >= GT_TEAM && team == TEAM_SPECTATOR) {
            ForceClientSkin(client, model, "red");
    //		ForceClientSkin(client, headModel, "red");
        }
        */

    if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        (*client).pers.teamInfo = QTRUE;
    } else {
        s = Info_ValueForKey(userinfo.as_ptr(), c"teamoverlay".as_ptr());
        if *s == 0 || atoi(s) != 0 {
            (*client).pers.teamInfo = QTRUE;
        } else {
            (*client).pers.teamInfo = QFALSE;
        }
    }
    /*
    s = Info_ValueForKey( userinfo, "cg_pmove_fixed" );
    if ( !*s || atoi( s ) == 0 ) {
        client->pers.pmoveFixed = qfalse;
    }
    else {
        client->pers.pmoveFixed = qtrue;
    }
    */

    // team task (0 = none, 1 = offence, 2 = defence)
    let teamTask = atoi(Info_ValueForKey(userinfo.as_ptr(), c"teamtask".as_ptr()));
    // team Leader (1 = leader, 0 is normal player)
    let teamLeader = (*client).sess.teamLeader;

    // colors
    strcpy(
        c1.as_mut_ptr(),
        Info_ValueForKey(userinfo.as_ptr(), c"color1".as_ptr()),
    );
    strcpy(
        c2.as_mut_ptr(),
        Info_ValueForKey(userinfo.as_ptr(), c"color2".as_ptr()),
    );

    //	strcpy(redTeam, Info_ValueForKey( userinfo, "g_redteam" ));
    //	strcpy(blueTeam, Info_ValueForKey( userinfo, "g_blueteam" ));

    // send over a subset of the userinfo keys so other clients can
    // print scoreboards, display models, and play custom sounds
    let s: *mut c_char = if (*ent).r.svFlags & SVF_BOT != 0 {
        va(format_args!(
            "n\\{}\\t\\{}\\model\\{}\\c1\\{}\\c2\\{}\\hc\\{}\\w\\{}\\l\\{}\\skill\\{}\\tt\\{}\\tl\\{}\\siegeclass\\{}\\st\\{}\\st2\\{}\\dt\\{}\\sdt\\{}",
            Sz((*client).pers.netname.as_ptr()),
            team,
            Sz(model),
            Sz(c1.as_ptr()),
            Sz(c2.as_ptr()),
            (*client).pers.maxHealth,
            (*client).sess.wins,
            (*client).sess.losses,
            Sz(Info_ValueForKey(userinfo.as_ptr(), c"skill".as_ptr())),
            teamTask,
            teamLeader,
            Sz(className.as_ptr()),
            Sz(saberName.as_ptr()),
            Sz(saber2Name.as_ptr()),
            (*client).sess.duelTeam,
            (*client).sess.siegeDesiredTeam
        ))
    } else if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        // more crap to send
        va(format_args!(
            "n\\{}\\t\\{}\\model\\{}\\c1\\{}\\c2\\{}\\hc\\{}\\w\\{}\\l\\{}\\tt\\{}\\tl\\{}\\siegeclass\\{}\\st\\{}\\st2\\{}\\dt\\{}\\sdt\\{}",
            Sz((*client).pers.netname.as_ptr()),
            (*client).sess.sessionTeam,
            Sz(model),
            Sz(c1.as_ptr()),
            Sz(c2.as_ptr()),
            (*client).pers.maxHealth,
            (*client).sess.wins,
            (*client).sess.losses,
            teamTask,
            teamLeader,
            Sz(className.as_ptr()),
            Sz(saberName.as_ptr()),
            Sz(saber2Name.as_ptr()),
            (*client).sess.duelTeam,
            (*client).sess.siegeDesiredTeam
        ))
    } else {
        va(format_args!(
            "n\\{}\\t\\{}\\model\\{}\\c1\\{}\\c2\\{}\\hc\\{}\\w\\{}\\l\\{}\\tt\\{}\\tl\\{}\\st\\{}\\st2\\{}\\dt\\{}",
            Sz((*client).pers.netname.as_ptr()),
            (*client).sess.sessionTeam,
            Sz(model),
            Sz(c1.as_ptr()),
            Sz(c2.as_ptr()),
            (*client).pers.maxHealth,
            (*client).sess.wins,
            (*client).sess.losses,
            teamTask,
            teamLeader,
            Sz(saberName.as_ptr()),
            Sz(saber2Name.as_ptr()),
            (*client).sess.duelTeam
        ))
    };

    trap::SetConfigstring(CS_PLAYERS + clientNum, &CStr::from_ptr(s).to_string_lossy());

    // The Xbox `if (com_dedicated && com_dedicated->integer) XBL_PL_UpdatePlayerName(...)`
    // block (g_client.c:2189) is dropped — `_XBOX`-only, not this build (see fn doc).

    if modelChanged == QTRUE
    //only going to be true for allowable server-side custom skeleton cases
    {
        //update the server g2 instance if appropriate
        let modelname = Info_ValueForKey(userinfo.as_ptr(), c"model".as_ptr());
        SetupGameGhoul2Model(ent, modelname, null_mut());

        if !(*ent).ghoul2.is_null() && !(*ent).client.is_null() {
            (*(*ent).client).renderInfo.lastG2 = null_mut(); //update the renderinfo bolts next update.
        }

        (*client).torsoAnimExecute = -1;
        (*client).legsAnimExecute = -1;
        (*client).torsoLastFlip = QFALSE;
        (*client).legsLastFlip = QFALSE;
    }

    if (*addr_of!(g_logClientInfo)).integer != 0 {
        G_LogPrintf(&format!(
            "ClientUserinfoChanged: {} {}\n",
            clientNum,
            CStr::from_ptr(s).to_string_lossy()
        ));
    }
}

/// `G_SaberModelSetup` (g_client.c:1332) — (re)load each of the client's saber hilt models
/// into its per-saber ghoul2 instances (`weaponGhoul2[]`), apply the custom skin, set the
/// hand bolt-info (right hand for saber 0, left for 1), register each blade's bolt point
/// (`*blade1`, `*blade2`, …, with the `*flash` 0ldsk3wl fallback), then copy each loaded
/// model into the entity's main ghoul2 instance at slot `i+1`. Returns `qtrue` when *no*
/// custom saber model loaded (so the caller should keep the default), `qfalse` once any
/// custom blade bolted. `ent` must have a valid `client`.
pub unsafe fn G_SaberModelSetup(ent: *mut gentity_t) -> qboolean {
    let mut i = 0;
    let mut fallbackForSaber = QTRUE;

    let client = (*ent).client;

    while i < MAX_SABERS {
        if (*client).saber[i].model[0] != 0 {
            //first kill it off if we've already got it
            if !(*client).weaponGhoul2[i].is_null() {
                trap::G2API_CleanGhoul2Models(&mut (*client).weaponGhoul2[i]);
            }
            trap::G2API_InitGhoul2Model(
                &mut (*client).weaponGhoul2[i],
                (*client).saber[i].model.as_ptr(),
                0,
                0,
                -20,
                0,
                0,
            );

            if !(*client).weaponGhoul2[i].is_null() {
                let mut j: c_int = 0;
                let mut tagBolt: c_int;

                if (*client).saber[i].skin != 0 {
                    trap::G2API_SetSkin(
                        (*client).weaponGhoul2[i],
                        0,
                        (*client).saber[i].skin,
                        (*client).saber[i].skin,
                    );
                }

                if (*client).saber[i].saberFlags & SFL_BOLT_TO_WRIST != 0 {
                    trap::G2API_SetBoltInfo((*client).weaponGhoul2[i], 0, 3 + i as c_int);
                } else {
                    // bolt to right hand for 0, or left hand for 1
                    trap::G2API_SetBoltInfo((*client).weaponGhoul2[i], 0, i as c_int);
                }

                //Add all the bolt points
                while j < (*client).saber[i].numBlades {
                    let tagName =
                        CStr::from_ptr(va(format_args!("*blade{}", j + 1))).to_string_lossy();
                    tagBolt = trap::G2API_AddBolt((*client).weaponGhoul2[i], 0, &tagName);

                    if tagBolt == -1 {
                        if j == 0 {
                            //guess this is an 0ldsk3wl saber
                            tagBolt = trap::G2API_AddBolt((*client).weaponGhoul2[i], 0, "*flash");
                            let _ = tagBolt;
                            fallbackForSaber = QFALSE;
                            break;
                        }

                        if tagBolt == -1 {
                            debug_assert!(false);
                            break;
                        }
                    }
                    j += 1;

                    fallbackForSaber = QFALSE; //got at least one custom saber so don't need default
                }

                //Copy it into the main instance
                trap::G2API_CopySpecificGhoul2Model(
                    (*client).weaponGhoul2[i],
                    0,
                    (*ent).ghoul2,
                    i as c_int + 1,
                );
            }
        } else {
            break;
        }

        i += 1;
    }

    fallbackForSaber
}

pub unsafe fn G_BreakArm(ent: *mut gentity_t, arm: c_int) {
    let mut anim: c_int = -1;

    debug_assert!(!ent.is_null() && !(*ent).client.is_null());

    if (*ent).s.NPC_class == CLASS_VEHICLE || (*ent).localAnimIndex > 1 {
        //no broken limbs for vehicles and non-humanoids
        return;
    }

    if arm == 0 {
        //repair him
        (*(*ent).client).ps.brokenLimbs = 0;
        return;
    }

    if (*(*ent).client).ps.fd.saberAnimLevel == SS_STAFF {
        //I'm too lazy to deal with this as well for now.
        return;
    }

    if arm == BROKENLIMB_LARM {
        if (*(*ent).client).saber[1].model[0] != 0
            && (*(*ent).client).ps.weapon == WP_SABER as c_int
            && (*(*ent).client).ps.saberHolstered == 0
            && (*(*ent).client).saber[1].soundOff != 0
        {
            //the left arm shuts off its saber upon being broken
            G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[1].soundOff);
        }
    }

    (*(*ent).client).ps.brokenLimbs = 0; //make sure it's cleared out
    (*(*ent).client).ps.brokenLimbs |= 1 << arm; //this arm is now marked as broken

    //Do a pain anim based on the side. Since getting your arm broken does tend to hurt.
    if arm == BROKENLIMB_LARM {
        anim = BOTH_PAIN2;
    } else if arm == BROKENLIMB_RARM {
        anim = BOTH_PAIN3;
    }

    if anim == -1 {
        return;
    }

    G_SetAnim(
        ent,
        &mut (*(*ent).client).pers.cmd,
        SETANIM_BOTH,
        anim,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        0,
    );

    //This could be combined into a single event. But I guess limbs don't break often enough to
    //worry about it.
    G_EntitySound(ent, CHAN_VOICE, G_SoundIndex("*pain25.wav"));
    //FIXME: A nice bone snapping sound instead if possible
    G_Sound(
        ent,
        CHAN_AUTO,
        G_SoundIndex(
            &CStr::from_ptr(va(format_args!(
                "sound/player/bodyfall_human{}.wav",
                Q_irand(1, 3)
            )))
            .to_string_lossy(),
        ),
    );
}

/// `G_UpdateClientAnims` (g_client.c:2663) — drive the server-side ghoul2 skeleton's leg/torso
/// (and humanoid "Motion") bone animations to match `self->client->ps` so server-side bolt/bone
/// lookups (saber positions, hit detection) line up with what the client renders. Mirrors what
/// cgame does: pulls the current `legsAnim`/`torsoAnim`, looks up the frame range and lerp speed
/// from `bgAllAnims[localAnimIndex]`, and pushes them onto the bones via `trap_G2API_SetBoneAnim`.
/// In a saber lock it freezes all three core bones on `saberLockFrame`. Non-humanoids
/// (`localAnimIndex > 1`) only set whatever bones exist; vehicles set only the root bone.
///
/// The original's broken-limb arm posing (the `#if 0 //disabled for now` block, ~2786-2907,
/// the sole `trap_G2API_RemoveBone` call site) is compiled out in JKA and is dropped here too.
pub unsafe fn G_UpdateClientAnims(self_: *mut gentity_t, mut anim_speed_scale: f32) {
    let f: c_int;
    let torso_anim: c_int;
    let legs_anim: c_int;
    let mut first_frame: c_int = 0;
    let mut last_frame: c_int = 0;
    let mut a_flags: c_int = 0;
    let mut anim_speed: f32;
    let mut l_anim_speed_scale: f32 = 0.0;
    let mut set_torso = QFALSE;

    let client = (*self_).client;
    let local_anim_index = (*self_).localAnimIndex;

    torso_anim = (*client).ps.torsoAnim;
    legs_anim = (*client).ps.legsAnim;

    if (*client).ps.saberLockFrame != 0 {
        let lock = (*client).ps.saberLockFrame;
        trap::G2API_SetBoneAnim(
            (*self_).ghoul2,
            0,
            "model_root",
            lock,
            lock + 1,
            BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
            anim_speed_scale,
            (*addr_of!(level)).time,
            -1.0,
            150,
        );
        trap::G2API_SetBoneAnim(
            (*self_).ghoul2,
            0,
            "lower_lumbar",
            lock,
            lock + 1,
            BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
            anim_speed_scale,
            (*addr_of!(level)).time,
            -1.0,
            150,
        );
        trap::G2API_SetBoneAnim(
            (*self_).ghoul2,
            0,
            "Motion",
            lock,
            lock + 1,
            BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
            anim_speed_scale,
            (*addr_of!(level)).time,
            -1.0,
            150,
        );
        return;
    }

    let legs_dead = local_anim_index > 1 && {
        let a = *(*addr_of!(bgAllAnims))[local_anim_index as usize]
            .anims
            .add(legs_anim as usize);
        a.firstFrame == 0 && a.numFrames == 0
    };

    if !legs_dead {
        // We'll allow non-humanoids to skip the legs anim and fall through to the torso.
        if (*client).legsAnimExecute != legs_anim || (*client).legsLastFlip != (*client).ps.legsFlip
        {
            let a = *(*addr_of!(bgAllAnims))[local_anim_index as usize]
                .anims
                .add(legs_anim as usize);
            anim_speed = 50.0f32 / a.frameLerp as c_int as f32;
            anim_speed *= anim_speed_scale;
            l_anim_speed_scale = anim_speed;

            if a.loopFrames as c_int != -1 {
                a_flags = BONE_ANIM_OVERRIDE_LOOP;
            } else {
                a_flags = BONE_ANIM_OVERRIDE_FREEZE;
            }

            if anim_speed < 0.0 {
                last_frame = a.firstFrame as c_int;
                first_frame = a.firstFrame as c_int + a.numFrames as c_int;
            } else {
                first_frame = a.firstFrame as c_int;
                last_frame = a.firstFrame as c_int + a.numFrames as c_int;
            }

            a_flags |= BONE_ANIM_BLEND; //since client defaults to blend.

            trap::G2API_SetBoneAnim(
                (*self_).ghoul2,
                0,
                "model_root",
                first_frame,
                last_frame,
                a_flags,
                l_anim_speed_scale,
                (*addr_of!(level)).time,
                -1.0,
                150,
            );
            (*client).legsAnimExecute = legs_anim;
            (*client).legsLastFlip = (*client).ps.legsFlip;
        }
    }

    // tryTorso:
    if local_anim_index > 1 && {
        let a = *(*addr_of!(bgAllAnims))[local_anim_index as usize]
            .anims
            .add(torso_anim as usize);
        a.firstFrame == 0 && a.numFrames == 0
    } {
        //If this fails as well just return.
        return;
    } else if (*self_).s.number >= MAX_CLIENTS as c_int && (*self_).s.NPC_class == CLASS_VEHICLE {
        //we only want to set the root bone for vehicles
        return;
    }

    if ((*client).torsoAnimExecute != torso_anim
        || (*client).torsoLastFlip != (*client).ps.torsoFlip)
        && (*self_).noLumbar == QFALSE
    {
        // C resets `aFlags = 0; animSpeed = 0;` here; both are dead stores (re-set below
        // before any read) — kept implicit since `a_flags` is already 0 at declaration.
        f = torso_anim;

        BG_SaberStartTransAnim(
            (*self_).s.number,
            (*client).ps.fd.saberAnimLevel,
            (*client).ps.weapon,
            f,
            &mut anim_speed_scale,
            (*client).ps.brokenLimbs,
        );

        let a = *(*addr_of!(bgAllAnims))[local_anim_index as usize]
            .anims
            .add(f as usize);
        anim_speed = 50.0f32 / a.frameLerp as c_int as f32;
        anim_speed *= anim_speed_scale;
        l_anim_speed_scale = anim_speed;

        if a.loopFrames as c_int != -1 {
            a_flags = BONE_ANIM_OVERRIDE_LOOP;
        } else {
            a_flags = BONE_ANIM_OVERRIDE_FREEZE;
        }

        a_flags |= BONE_ANIM_BLEND; //since client defaults to blend.

        if anim_speed < 0.0 {
            last_frame = a.firstFrame as c_int;
            first_frame = a.firstFrame as c_int + a.numFrames as c_int;
        } else {
            first_frame = a.firstFrame as c_int;
            last_frame = a.firstFrame as c_int + a.numFrames as c_int;
        }

        trap::G2API_SetBoneAnim(
            (*self_).ghoul2,
            0,
            "lower_lumbar",
            first_frame,
            last_frame,
            a_flags,
            l_anim_speed_scale,
            (*addr_of!(level)).time,
            /*firstFrame why was it this before?*/ -1.0,
            150,
        );

        (*client).torsoAnimExecute = torso_anim;
        (*client).torsoLastFlip = (*client).ps.torsoFlip;

        set_torso = QTRUE;
    }

    if set_torso == QTRUE && local_anim_index <= 1 {
        //only set the motion bone for humanoids.
        trap::G2API_SetBoneAnim(
            (*self_).ghoul2,
            0,
            "Motion",
            first_frame,
            last_frame,
            a_flags,
            l_anim_speed_scale,
            (*addr_of!(level)).time,
            -1.0,
            150,
        );
    }

    // The broken-limb arm posing block (#if 0 //disabled for now) is compiled out in JKA;
    // it is the only trap_G2API_RemoveBone call site and is intentionally omitted here.
}

// in case it gets stuck somewhere no one can reach
const JMSABER_RESPAWN_TIME: c_int = 20000;

pub unsafe fn ThrowSaberToAttacker(self_: *mut gentity_t, attacker: *mut gentity_t) {
    let mut ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .add((*(*self_).client).ps.saberIndex as usize);
    let mut a: vec3_t = [0.0; 3];
    let mut alt_velocity: c_int = 0;

    if ent.is_null() || (*ent).enemy != self_ {
        // something has gone very wrong (this should never happen)
        // but in case it does.. find the saber manually
        #[cfg(debug_assertions)]
        Com_Printf("Lost the saber! Attempting to use global pointer..\n");
        ent = gJMSaberEnt;

        if ent.is_null() {
            #[cfg(debug_assertions)]
            Com_Printf("The global pointer was NULL. This is a bad thing.\n");
            return;
        }

        #[cfg(debug_assertions)]
        Com_Printf(&format!(
            "Got it ({}). Setting enemy to client {}.\n",
            (*ent).s.number,
            (*self_).s.number
        ));

        (*ent).enemy = self_;
        (*(*self_).client).ps.saberIndex = (*ent).s.number;
    }

    trap::SetConfigstring(CS_CLIENT_JEDIMASTER, "-1");

    if !attacker.is_null()
        && !(*attacker).client.is_null()
        && (*(*self_).client).ps.saberInFlight == QTRUE
    {
        // someone killed us and we had the saber thrown, so actually move this saber to the saber location
        // if we killed ourselves with saber thrown, however, same suicide rules of respawning at spawn spot still
        // apply.
        let flyingsaber: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*(*self_).client).ps.saberEntityNum as usize);

        if !flyingsaber.is_null() && (*flyingsaber).inuse == QTRUE {
            VectorCopy(&(*flyingsaber).s.pos.trBase, &mut (*ent).s.pos.trBase);
            VectorCopy(&(*flyingsaber).s.pos.trDelta, &mut (*ent).s.pos.trDelta);
            VectorCopy(&(*flyingsaber).s.apos.trBase, &mut (*ent).s.apos.trBase);
            VectorCopy(&(*flyingsaber).s.apos.trDelta, &mut (*ent).s.apos.trDelta);

            VectorCopy(&(*flyingsaber).r.currentOrigin, &mut (*ent).r.currentOrigin);
            VectorCopy(&(*flyingsaber).r.currentAngles, &mut (*ent).r.currentAngles);
            alt_velocity = 1;
        }
    }

    (*(*self_).client).ps.saberInFlight = QTRUE; //say he threw it anyway in order to properly remove from dead body

    WP_SaberAddG2Model(
        ent,
        (*(*self_).client).saber[0].model.as_ptr(),
        (*(*self_).client).saber[0].skin,
    );

    (*ent).s.eFlags &= !(EF_NODRAW);
    (*ent).s.modelGhoul2 = 1;
    (*ent).s.eType = ET_MISSILE;
    (*ent).enemy = null_mut();

    if attacker.is_null() || (*attacker).client.is_null() {
        VectorCopy(&(*ent).s.origin2, &mut (*ent).s.pos.trBase);
        VectorCopy(&(*ent).s.origin2, &mut (*ent).s.origin);
        VectorCopy(&(*ent).s.origin2, &mut (*ent).r.currentOrigin);
        (*ent).pos2[0] = 0.0;
        trap::LinkEntity(ent);
        return;
    }

    if alt_velocity == 0 {
        VectorCopy(&(*self_).s.pos.trBase, &mut (*ent).s.pos.trBase);
        VectorCopy(&(*self_).s.pos.trBase, &mut (*ent).s.origin);
        VectorCopy(&(*self_).s.pos.trBase, &mut (*ent).r.currentOrigin);

        VectorSubtract(
            &(*(*attacker).client).ps.origin,
            &(*ent).s.pos.trBase,
            &mut a,
        );

        VectorNormalize(&mut a);

        (*ent).s.pos.trDelta[0] = a[0] * 256.0;
        (*ent).s.pos.trDelta[1] = a[1] * 256.0;
        (*ent).s.pos.trDelta[2] = 256.0;
    }

    trap::LinkEntity(ent);
}

pub unsafe extern "C" fn JMSaberThink(ent: *mut gentity_t) {
    gJMSaberEnt = ent;

    if !(*ent).enemy.is_null() {
        if (*(*ent).enemy).client.is_null() || (*(*ent).enemy).inuse != QTRUE {
            // disconnected?
            VectorCopy(&(*(*ent).enemy).s.pos.trBase, &mut (*ent).s.pos.trBase);
            VectorCopy(&(*(*ent).enemy).s.pos.trBase, &mut (*ent).s.origin);
            VectorCopy(&(*(*ent).enemy).s.pos.trBase, &mut (*ent).r.currentOrigin);
            (*ent).s.modelindex = G_ModelIndex("models/weapons2/saber/saber_w.glm");
            (*ent).s.eFlags &= !(EF_NODRAW);
            (*ent).s.modelGhoul2 = 1;
            (*ent).s.eType = ET_MISSILE;
            (*ent).enemy = null_mut();

            (*ent).pos2[0] = 1.0;
            (*ent).pos2[1] = 0.0; //respawn next think
            trap::LinkEntity(ent);
        } else {
            (*ent).pos2[1] = ((*addr_of!(level)).time + JMSABER_RESPAWN_TIME) as f32;
        }
    } else if (*ent).pos2[0] != 0.0 && (*ent).pos2[1] < (*addr_of!(level)).time as f32 {
        VectorCopy(&(*ent).s.origin2, &mut (*ent).s.pos.trBase);
        VectorCopy(&(*ent).s.origin2, &mut (*ent).s.origin);
        VectorCopy(&(*ent).s.origin2, &mut (*ent).r.currentOrigin);
        (*ent).pos2[0] = 0.0;
        trap::LinkEntity(ent);
    }

    (*ent).nextthink = (*addr_of!(level)).time + 50;
    G_RunObject(ent);
}

pub unsafe extern "C" fn JMSaberTouch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    let mut i: c_int = 0;
    //	gentity_t *te;

    if other.is_null() || (*other).client.is_null() || (*other).health < 1 {
        return;
    }

    if !(*self_).enemy.is_null() {
        return;
    }

    if (*self_).s.modelindex == 0 {
        return;
    }

    if (*(*other).client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_SABER) != 0 {
        return;
    }
    if (*(*other).client).ps.isJediMaster != 0 {
        return;
    }

    (*self_).enemy = other;
    (*(*other).client).ps.stats[STAT_WEAPONS as usize] = 1 << WP_SABER;
    (*(*other).client).ps.weapon = WP_SABER;
    (*other).s.weapon = WP_SABER;
    G_AddEvent(other, EV_BECOME_JEDIMASTER, 0);

    // Track the jedi master
    trap::SetConfigstring(CS_CLIENT_JEDIMASTER, &format!("{}", (*other).s.number));

    if (*addr_of!(g_spawnInvulnerability)).integer != 0 {
        (*(*other).client).ps.eFlags |= EF_INVULNERABLE;
        (*(*other).client).invulnerableTimer =
            (*addr_of!(level)).time + (*addr_of!(g_spawnInvulnerability)).integer;
    }

    trap::SendServerCommand(
        -1,
        &format!(
            "cp \"{} {}\n\"",
            Sz((*(*other).client).pers.netname.as_ptr()),
            Sz(G_GetStringEdString(
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"BECOMEJM".as_ptr() as *mut c_char,
            )),
        ),
    );

    (*(*other).client).ps.isJediMaster = QTRUE;
    (*(*other).client).ps.saberIndex = (*self_).s.number;

    if (*other).health < 200 && (*other).health > 0 {
        //full health when you become the Jedi Master
        (*(*other).client).ps.stats[STAT_HEALTH as usize] = 200;
        (*other).health = 200;
    }

    if (*(*other).client).ps.fd.forcePower < 100 {
        (*(*other).client).ps.fd.forcePower = 100;
    }

    while i < NUM_FORCE_POWERS as c_int {
        (*(*other).client).ps.fd.forcePowersKnown |= 1 << i;
        (*(*other).client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;

        i += 1;
    }

    (*self_).pos2[0] = 1.0;
    (*self_).pos2[1] = ((*addr_of!(level)).time + JMSABER_RESPAWN_TIME) as f32;

    (*self_).s.modelindex = 0;
    (*self_).s.eFlags |= EF_NODRAW;
    (*self_).s.modelGhoul2 = 0;
    (*self_).s.eType = ET_GENERAL;

    /*
    te = G_TempEntity( vec3_origin, EV_DESTROY_GHOUL2_INSTANCE );
    te->r.svFlags |= SVF_BROADCAST;
    te->s.eventParm = self->s.number;
    */
    G_KillG2Queue((*self_).s.number);
}

pub static mut gJMSaberEnt: *mut gentity_t = null_mut();

/*QUAKED info_jedimaster_start (1 0 0) (-16 -16 -24) (16 16 32)
"jedi master" saber spawn point
*/
pub unsafe fn SP_info_jedimaster_start(ent: *mut gentity_t) {
    if (*addr_of!(g_gametype)).integer != GT_JEDIMASTER {
        gJMSaberEnt = null_mut();
        G_FreeEntity(ent);
        return;
    }

    (*ent).enemy = null_mut();

    (*ent).flags = FL_BOUNCE_HALF;

    (*ent).s.modelindex = G_ModelIndex("models/weapons2/saber/saber_w.glm");
    (*ent).s.modelGhoul2 = 1;
    (*ent).s.g2radius = 20;
    //ent->s.eType = ET_GENERAL;
    (*ent).s.eType = ET_MISSILE;
    (*ent).s.weapon = WP_SABER;
    (*ent).s.pos.trType = TR_GRAVITY;
    (*ent).s.pos.trTime = (*addr_of!(level)).time;
    VectorSet(&mut (*ent).r.maxs, 3.0, 3.0, 3.0);
    VectorSet(&mut (*ent).r.mins, -3.0, -3.0, -3.0);
    (*ent).r.contents = CONTENTS_TRIGGER;
    (*ent).clipmask = MASK_SOLID;

    (*ent).isSaberEntity = QTRUE;

    (*ent).bounceCount = -5;

    (*ent).physicsObject = QTRUE;

    VectorCopy(&(*ent).s.pos.trBase, &mut (*ent).s.origin2); //remember the spawn spot

    (*ent).touch = Some(JMSaberTouch);

    trap::LinkEntity(ent);

    (*ent).think = Some(JMSaberThink);
    (*ent).nextthink = (*addr_of!(level)).time + 50;
}

/// `void *g2SaberInstance = NULL;` (g_client.c:1414) — the server's shared template ghoul2
/// instance of the saber model, lazily created the first time [`SetupGameGhoul2Model`] runs
/// for a saber user and copied into each player's instance. Read by `w_saber.c`'s
/// `WP_SaberPositionUpdate` (it bails while this is null). The lazy-init lives in
/// [`SetupGameGhoul2Model`] (and the parallel `G_PrecacheGhoul2Models` in `g_spawn.rs`);
/// `g_main.c`'s shutdown also cleans it.
pub static mut g2SaberInstance: *mut c_void = null_mut();

/// `void SetupGameGhoul2Model( gentity_t *ent, char *modelname, char *skinName )`
/// (g_client.c:1422). Builds the server-side ghoul2 instance for `ent`: tears down any
/// existing instance, lazily precaches the shared "kyle" template ([`precachedKyle`]), then —
/// only when per-player models are enabled (`d_perPlayerGhoul2`), `ent` is an NPC, or it has a
/// custom skeleton — loads the player/vehicle model + skin and validates the GLA, falling back
/// to the kyle duplicate on any failure. Finally attaches the instance to the entity num,
/// loads the humanoid animation config, resolves `localAnimIndex` from the GLA, wires vehicle
/// bolts (droid/exhaust/muzzle/turret), and for saber users adds the hand/chest bolts, seeds
/// the lumbar/cranium bone angles, lazily builds the shared saber template ([`g2SaberInstance`])
/// and copies the per-client saber models in via [`G_SaberModelSetup`].
///
/// No oracle: pure side effects through the ghoul2 traps + entity/vehicle state mutation
/// (project precedent c111).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`; `modelname` is a valid
/// writable NUL-terminated buffer (it is mutated in place for vehicles via
/// [`BG_GetVehicleModelName`]); `skin_name` is NULL or a valid NUL-terminated C string.
pub(crate) unsafe fn SetupGameGhoul2Model(
    ent: *mut gentity_t,
    modelname: *mut c_char,
    skin_name: *mut c_char,
) {
    let handle: c_int;
    // char afilename[MAX_QPATH]; — only ever holds the kyle literal below, so we pass the
    // C-string directly rather than carry the scratch buffer.
    // #if 0
    //     char /**GLAName,*/ *slash;
    // #endif
    let mut GLAName = [0 as c_char; MAX_QPATH];
    let tempVec: vec3_t = [0.0, 0.0, 0.0];

    // First things first.  If this is a ghoul2 model, then let's make sure we demolish this first.
    if !(*ent).ghoul2.is_null() && trap::G2_HaveWeGhoul2Models((*ent).ghoul2) != QFALSE {
        trap::G2API_CleanGhoul2Models(&mut (*ent).ghoul2);
    }

    //rww - just load the "standard" model for the server"
    if precachedKyle.is_null() {
        let defSkin: c_int;

        handle = trap::G2API_InitGhoul2Model(
            addr_of_mut!(precachedKyle),
            c"models/players/kyle/model.glm".as_ptr(),
            0,
            0,
            -20,
            0,
            0,
        );

        if handle < 0 {
            return;
        }

        defSkin = trap::R_RegisterSkin("models/players/kyle/model_default.skin");
        trap::G2API_SetSkin(precachedKyle, 0, defSkin, defSkin);
    }

    if !precachedKyle.is_null() && trap::G2_HaveWeGhoul2Models(precachedKyle) != QFALSE {
        if (*addr_of!(d_perPlayerGhoul2)).integer != 0
            || (*ent).s.number >= MAX_CLIENTS as c_int
            || G_PlayerHasCustomSkeleton(ent) != QFALSE
        {
            //rww - allow option for perplayer models on server for collision and bolt stuff.
            let mut modelFullPath = [0 as c_char; MAX_QPATH];
            let mut truncModelName = [0 as c_char; MAX_QPATH];
            let mut skin = [0 as c_char; MAX_QPATH];
            let mut vehicleName = [0 as c_char; MAX_QPATH];
            let mut skinHandle: c_int = 0;
            let mut i: c_int = 0;
            let mut p: *mut c_char;

            // If this is a vehicle, get it's model name.
            if (*(*ent).client).NPC_class == CLASS_VEHICLE {
                strcpy(vehicleName.as_mut_ptr(), modelname);
                BG_GetVehicleModelName(modelname);
                strcpy(truncModelName.as_mut_ptr(), modelname);
                skin[0] = 0;
                if !(*ent).m_pVehicle.is_null()
                    && !(*(*ent).m_pVehicle).m_pVehicleInfo.is_null()
                    && !(*(*(*ent).m_pVehicle).m_pVehicleInfo).skin.is_null()
                    && *(*(*(*ent).m_pVehicle).m_pVehicleInfo).skin != 0
                {
                    skinHandle = trap::R_RegisterSkin(&format!(
                        "models/players/{}/model_{}.skin",
                        CStr::from_ptr(modelname).to_string_lossy(),
                        CStr::from_ptr((*(*(*ent).m_pVehicle).m_pVehicleInfo).skin)
                            .to_string_lossy()
                    ));
                } else {
                    skinHandle = trap::R_RegisterSkin(&format!(
                        "models/players/{}/model_default.skin",
                        CStr::from_ptr(modelname).to_string_lossy()
                    ));
                }
            } else {
                if !skin_name.is_null() && *skin_name != 0 {
                    strcpy(skin.as_mut_ptr(), skin_name);
                    strcpy(truncModelName.as_mut_ptr(), modelname);
                } else {
                    strcpy(skin.as_mut_ptr(), c"default".as_ptr());

                    strcpy(truncModelName.as_mut_ptr(), modelname);
                    p = Q_strrchr(truncModelName.as_mut_ptr(), '/' as c_int);

                    if !p.is_null() {
                        *p = 0;
                        p = p.add(1);

                        while !p.is_null() && *p != 0 {
                            skin[i as usize] = *p;
                            i += 1;
                            p = p.add(1);
                        }
                        skin[i as usize] = 0;
                        // i = 0; (vestigial in C — `i` is not read again in this block)
                    }

                    if BG_IsValidCharacterModel(truncModelName.as_ptr(), skin.as_ptr()) == QFALSE {
                        strcpy(truncModelName.as_mut_ptr(), c"kyle".as_ptr());
                        strcpy(skin.as_mut_ptr(), c"default".as_ptr());
                    }

                    if (*addr_of!(g_gametype)).integer >= GT_TEAM
                        && (*addr_of!(g_gametype)).integer != GT_SIEGE
                        && (*addr_of!(g_trueJedi)).integer == 0
                    {
                        BG_ValidateSkinForTeam(
                            truncModelName.as_ptr(),
                            skin.as_mut_ptr(),
                            (*(*ent).client).sess.sessionTeam,
                            null_mut(),
                        );
                    } else if (*addr_of!(g_gametype)).integer == GT_SIEGE {
                        //force skin for class if appropriate
                        if (*(*ent).client).siegeClass != -1 {
                            let scl = (addr_of_mut!(bgSiegeClasses) as *mut siegeClass_t)
                                .add((*(*ent).client).siegeClass as usize);
                            if (*scl).forcedSkin[0] != 0 {
                                strcpy(skin.as_mut_ptr(), (*scl).forcedSkin.as_ptr());
                            }
                        }
                    }
                }
            }

            if skin[0] != 0 {
                let useSkinName: String;

                if !strchr(skin.as_ptr(), '|' as c_int).is_null() {
                    //three part skin
                    useSkinName = format!(
                        "models/players/{}/|{}",
                        CStr::from_ptr(truncModelName.as_ptr()).to_string_lossy(),
                        CStr::from_ptr(skin.as_ptr()).to_string_lossy()
                    );
                } else {
                    useSkinName = format!(
                        "models/players/{}/model_{}.skin",
                        CStr::from_ptr(truncModelName.as_ptr()).to_string_lossy(),
                        CStr::from_ptr(skin.as_ptr()).to_string_lossy()
                    );
                }

                skinHandle = trap::R_RegisterSkin(&useSkinName);
            }

            Com_sprintf(
                modelFullPath.as_mut_ptr(),
                MAX_QPATH as c_int,
                format_args!(
                    "models/players/{}/model.glm",
                    CStr::from_ptr(truncModelName.as_ptr()).to_string_lossy()
                ),
            );
            let handle2 = trap::G2API_InitGhoul2Model(
                &mut (*ent).ghoul2,
                modelFullPath.as_ptr(),
                0,
                skinHandle,
                -20,
                0,
                0,
            );

            if handle2 < 0 {
                //Huh. Guess we don't have this model. Use the default.

                if !(*ent).ghoul2.is_null() && trap::G2_HaveWeGhoul2Models((*ent).ghoul2) != QFALSE
                {
                    trap::G2API_CleanGhoul2Models(&mut (*ent).ghoul2);
                }
                (*ent).ghoul2 = null_mut();
                trap::G2API_DuplicateGhoul2Instance(precachedKyle, &mut (*ent).ghoul2);
            } else {
                trap::G2API_SetSkin((*ent).ghoul2, 0, skinHandle, skinHandle);

                GLAName[0] = 0;
                trap::G2API_GetGLAName((*ent).ghoul2, 0, GLAName.as_mut_ptr());

                if GLAName[0] == 0
                    || (strstr(GLAName.as_ptr(), c"players/_humanoid/".as_ptr()).is_null()
                        && (*ent).s.number < MAX_CLIENTS as c_int
                        && G_PlayerHasCustomSkeleton(ent) == QFALSE)
                {
                    //a bad model
                    trap::G2API_CleanGhoul2Models(&mut (*ent).ghoul2);
                    (*ent).ghoul2 = null_mut();
                    trap::G2API_DuplicateGhoul2Instance(precachedKyle, &mut (*ent).ghoul2);
                }

                if (*ent).s.number >= MAX_CLIENTS as c_int {
                    (*ent).s.modelGhoul2 = 1; //so we know to free it on the client when we're removed.

                    if skin[0] != 0 {
                        //append it after a *
                        strcat(
                            modelFullPath.as_mut_ptr(),
                            va(format_args!(
                                "*{}",
                                CStr::from_ptr(skin.as_ptr()).to_string_lossy()
                            )),
                        );
                    }

                    if (*(*ent).client).NPC_class == CLASS_VEHICLE {
                        //vehicles are tricky and send over their vehicle names as the model (the model is then retrieved based on the vehicle name)
                        (*ent).s.modelindex =
                            G_ModelIndex(&CStr::from_ptr(vehicleName.as_ptr()).to_string_lossy());
                    } else {
                        (*ent).s.modelindex =
                            G_ModelIndex(&CStr::from_ptr(modelFullPath.as_ptr()).to_string_lossy());
                    }
                }
            }
        } else {
            trap::G2API_DuplicateGhoul2Instance(precachedKyle, &mut (*ent).ghoul2);
        }
    } else {
        return;
    }

    //Attach the instance to this entity num so we can make use of client-server
    //shared operations if possible.
    trap::G2API_AttachInstanceToEntNum((*ent).ghoul2, (*ent).s.number, QTRUE);

    // The model is now loaded.

    GLAName[0] = 0;

    if *addr_of!(BGPAFtextLoaded) == QFALSE {
        if BG_ParseAnimationFile(
            c"models/players/_humanoid/animation.cfg".as_ptr(),
            addr_of_mut!(bgHumanoidAnimations) as *mut _,
            QTRUE,
        ) == -1
        {
            Com_Printf("Failed to load humanoid animation file\n");
            return;
        }
    }

    if (*ent).s.number >= MAX_CLIENTS as c_int || G_PlayerHasCustomSkeleton(ent) != QFALSE {
        (*ent).localAnimIndex = -1;

        GLAName[0] = 0;
        trap::G2API_GetGLAName((*ent).ghoul2, 0, GLAName.as_mut_ptr());

        if GLAName[0] != 0 && strstr(GLAName.as_ptr(), c"players/_humanoid/".as_ptr()).is_null()
        /*&& !strstr(GLAName, "players/rockettrooper/")*/
        {
            //it doesn't use humanoid anims.
            let slash = Q_strrchr(GLAName.as_mut_ptr(), '/' as c_int);
            if !slash.is_null() {
                strcpy(slash, c"/animation.cfg".as_ptr());

                (*ent).localAnimIndex = BG_ParseAnimationFile(GLAName.as_ptr(), null_mut(), QFALSE);
            }
        } else {
            //humanoid index.
            if !strstr(GLAName.as_ptr(), c"players/rockettrooper/".as_ptr()).is_null() {
                (*ent).localAnimIndex = 1;
            } else {
                (*ent).localAnimIndex = 0;
            }
        }

        if (*ent).localAnimIndex == -1 {
            Com_Error(ERR_DROP, "NPC had an invalid GLA\n");
        }
    } else {
        GLAName[0] = 0;
        trap::G2API_GetGLAName((*ent).ghoul2, 0, GLAName.as_mut_ptr());

        if !strstr(GLAName.as_ptr(), c"players/rockettrooper/".as_ptr()).is_null() {
            //assert(!"Should not have gotten in here with rockettrooper skel");
            (*ent).localAnimIndex = 1;
        } else {
            (*ent).localAnimIndex = 0;
        }
    }

    if (*ent).s.NPC_class == CLASS_VEHICLE && !(*ent).m_pVehicle.is_null() {
        //do special vehicle stuff
        let mut i: c_int;

        // Setup the default first bolt
        i = trap::G2API_AddBolt((*ent).ghoul2, 0, "model_root");
        let _ = i;

        // Setup the droid unit.
        (*(*ent).m_pVehicle).m_iDroidUnitTag = trap::G2API_AddBolt((*ent).ghoul2, 0, "*droidunit");

        // Setup the Exhausts.
        i = 0;
        while (i as usize) < MAX_VEHICLE_EXHAUSTS {
            (*(*ent).m_pVehicle).m_iExhaustTag[i as usize] =
                trap::G2API_AddBolt((*ent).ghoul2, 0, &format!("*exhaust{}", i + 1));
            i += 1;
        }

        // Setup the Muzzles.
        i = 0;
        while (i as usize) < MAX_VEHICLE_MUZZLES {
            (*(*ent).m_pVehicle).m_iMuzzleTag[i as usize] =
                trap::G2API_AddBolt((*ent).ghoul2, 0, &format!("*muzzle{}", i + 1));
            if (*(*ent).m_pVehicle).m_iMuzzleTag[i as usize] == -1 {
                //ergh, try *flash?
                (*(*ent).m_pVehicle).m_iMuzzleTag[i as usize] =
                    trap::G2API_AddBolt((*ent).ghoul2, 0, &format!("*flash{}", i + 1));
            }
            i += 1;
        }

        // Setup the Turrets.
        i = 0;
        while (i as usize) < MAX_VEHICLE_TURRET_MUZZLES {
            if !(*(*(*ent).m_pVehicle).m_pVehicleInfo).turret[i as usize]
                .gunnerViewTag
                .is_null()
            {
                (*(*ent).m_pVehicle).m_iGunnerViewTag[i as usize] = trap::G2API_AddBolt(
                    (*ent).ghoul2,
                    0,
                    &CStr::from_ptr(
                        (*(*(*ent).m_pVehicle).m_pVehicleInfo).turret[i as usize].gunnerViewTag,
                    )
                    .to_string_lossy(),
                );
            } else {
                (*(*ent).m_pVehicle).m_iGunnerViewTag[i as usize] = -1;
            }
            i += 1;
        }
    }

    if (*(*ent).client).ps.weapon == WP_SABER as c_int || (*ent).s.number < MAX_CLIENTS as c_int {
        //a player or NPC saber user
        trap::G2API_AddBolt((*ent).ghoul2, 0, "*r_hand");
        trap::G2API_AddBolt((*ent).ghoul2, 0, "*l_hand");

        //rhand must always be first bolt. lhand always second. Whichever you want the
        //jetpack bolted to must always be third.
        trap::G2API_AddBolt((*ent).ghoul2, 0, "*chestg");

        //claw bolts
        trap::G2API_AddBolt((*ent).ghoul2, 0, "*r_hand_cap_r_arm");
        trap::G2API_AddBolt((*ent).ghoul2, 0, "*l_hand_cap_l_arm");

        trap::G2API_SetBoneAnim(
            (*ent).ghoul2,
            0,
            "model_root",
            0,
            12,
            BONE_ANIM_OVERRIDE_LOOP,
            1.0,
            (*addr_of!(level)).time,
            -1.0,
            -1,
        );
        trap::G2API_SetBoneAngles(
            (*ent).ghoul2,
            0,
            "upper_lumbar",
            &tempVec,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X,
            NEGATIVE_Y,
            NEGATIVE_Z,
            null_mut(),
            0,
            (*addr_of!(level)).time,
        );
        trap::G2API_SetBoneAngles(
            (*ent).ghoul2,
            0,
            "cranium",
            &tempVec,
            BONE_ANGLES_POSTMULT,
            POSITIVE_Z,
            NEGATIVE_Y,
            POSITIVE_X,
            null_mut(),
            0,
            (*addr_of!(level)).time,
        );

        if g2SaberInstance.is_null() {
            trap::G2API_InitGhoul2Model(
                addr_of_mut!(g2SaberInstance),
                c"models/weapons2/saber/saber_w.glm".as_ptr(),
                0,
                0,
                -20,
                0,
                0,
            );

            if !g2SaberInstance.is_null() {
                // indicate we will be bolted to model 0 (ie the player) on bolt 0 (always the right hand) when we get copied
                trap::G2API_SetBoltInfo(g2SaberInstance, 0, 0);
                // now set up the gun bolt on it
                trap::G2API_AddBolt(g2SaberInstance, 0, "*blade1");
            }
        }

        if G_SaberModelSetup(ent) != QFALSE {
            if !g2SaberInstance.is_null() {
                trap::G2API_CopySpecificGhoul2Model(g2SaberInstance, 0, (*ent).ghoul2, 1);
            }
        }
    }

    if (*ent).s.number >= MAX_CLIENTS as c_int {
        //some extra NPC stuff
        if trap::G2API_AddBolt((*ent).ghoul2, 0, "lower_lumbar") == -1 {
            //check now to see if we have this bone for setting anims and such
            (*ent).noLumbar = QTRUE;
        }
    }
}

/*
===========
ClientBegin

called when a client has finished connecting, and is ready
to be placed into the level.  This will happen every level load,
and on transition between teams, but doesn't happen on respawns
============
*/
/// `void ClientBegin( int clientNum, qboolean allowTeamReset )` (g_client.c:2374).
/// Places a finished-connecting client into the level: (re)assigns a bot's team,
/// resets the entity, stops active force powers + mutes kill sounds, wipes the
/// playerState, re-inits force/saber data + the Ghoul2 model, applies the team /
/// saber loadout, spawns the client, and sends the teleport-in event. No oracle
/// (mutates entity/client state, hits traps, uses the game RNG).
///
/// # Safety
/// `client_num` must be a valid client index; the `g_entities` / `level` globals must
/// be initialised and `g_entities[client_num].client` non-null.
pub unsafe fn ClientBegin(client_num: c_int, allow_team_reset: qboolean) {
    let ent: *mut gentity_t;
    let client: *mut gclient_t;
    let flags: c_int;
    let mut i: c_int;
    let mut userinfo = [0 as c_char; MAX_INFO_VALUE];
    let modelname: *mut c_char;

    ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(client_num as usize);

    if ((*ent).r.svFlags & SVF_BOT) != 0 && (*addr_of!(g_gametype)).integer >= GT_TEAM {
        if allow_team_reset != QFALSE {
            let pre_sess: c_int;

            //SetTeam(ent, "");
            (*(*ent).client).sess.sessionTeam = PickTeam(-1);
            // trap_GetUserinfo(clientNum, userinfo, MAX_INFO_STRING);
            {
                let info = trap::GetUserinfo(client_num);
                let bytes = info.as_bytes();
                let n = bytes.len().min(MAX_INFO_STRING - 1);
                for k in 0..n {
                    userinfo[k] = bytes[k] as c_char;
                }
                userinfo[n] = 0;
            }

            if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
                (*(*ent).client).sess.sessionTeam = TEAM_RED;
            }

            // C: const char *team = "Red"; — immediately overwritten below.
            let team: *const c_char = if (*(*ent).client).sess.sessionTeam == TEAM_RED {
                c"Red".as_ptr()
            } else {
                c"Blue".as_ptr()
            };

            Info_SetValueForKey(userinfo.as_mut_ptr(), c"team".as_ptr(), team);

            trap::SetUserinfo(
                client_num,
                &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy(),
            );

            (*(*ent).client).ps.persistant[PERS_TEAM as usize] = (*(*ent).client).sess.sessionTeam;

            pre_sess = (*(*ent).client).sess.sessionTeam;
            G_ReadSessionData((*ent).client);
            (*(*ent).client).sess.sessionTeam = pre_sess;
            G_WriteClientSessionData((*ent).client);
            ClientUserinfoChanged(client_num);
            ClientBegin(client_num, QFALSE);
            return;
        }
    }

    client = (*addr_of!(level)).clients.add(client_num as usize);

    if (*ent).r.linked != QFALSE {
        trap::UnlinkEntity(ent);
    }
    G_InitGentity(ent);
    (*ent).touch = None;
    (*ent).pain = None;
    (*ent).client = client;

    //assign the pointer for bg entity access
    (*ent).playerState = addr_of_mut!((*(*ent).client).ps);

    (*client).pers.connected = CON_CONNECTED;
    (*client).pers.enterTime = (*addr_of!(level)).time;
    (*client).pers.teamState.state = TEAM_BEGIN;

    // save eflags around this, because changing teams will
    // cause this to happen with a valid entity, and we
    // want to make sure the teleport bit is set right
    // so the viewpoint doesn't interpolate through the
    // world to the new position
    flags = (*client).ps.eFlags;

    i = 0;

    while i < NUM_FORCE_POWERS as c_int {
        if ((*(*ent).client).ps.fd.forcePowersActive & (1 << i)) != 0 {
            WP_ForcePowerStop(ent, i);
        }
        i += 1;
    }

    i = TRACK_CHANNEL_1;

    while i < NUM_TRACK_CHANNELS {
        let k = (i - 50) as usize;
        if (*(*ent).client).ps.fd.killSoundEntIndex[k] != 0
            && (*(*ent).client).ps.fd.killSoundEntIndex[k] < MAX_GENTITIES as c_int
            && (*(*ent).client).ps.fd.killSoundEntIndex[k] > 0
        {
            G_MuteSound((*(*ent).client).ps.fd.killSoundEntIndex[k], CHAN_VOICE);
        }
        i += 1;
    }
    // i = 0; (vestigial in C — `i` is not read again after this point)

    core::ptr::write_bytes(addr_of_mut!((*client).ps), 0, 1);
    (*client).ps.eFlags = flags;

    (*client).ps.hasDetPackPlanted = QFALSE;

    //first-time force power initialization
    WP_InitForcePowers(ent);

    //init saber ent
    WP_SaberInitBladeData(ent);

    // First time model setup for that player.
    // trap_GetUserinfo( clientNum, userinfo, sizeof(userinfo) );
    {
        let info = trap::GetUserinfo(client_num);
        let bytes = info.as_bytes();
        let n = bytes.len().min(MAX_INFO_VALUE - 1);
        for k in 0..n {
            userinfo[k] = bytes[k] as c_char;
        }
        userinfo[n] = 0;
    }
    modelname = Info_ValueForKey(userinfo.as_ptr(), c"model".as_ptr());
    SetupGameGhoul2Model(ent, modelname, null_mut());

    if !(*ent).ghoul2.is_null() && !(*ent).client.is_null() {
        (*(*ent).client).renderInfo.lastG2 = null_mut(); //update the renderinfo bolts next update.
    }

    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        && (*client).sess.sessionTeam != TEAM_SPECTATOR
        && (*client).sess.duelTeam == DUELTEAM_FREE
    {
        SetTeam(ent, c"s".as_ptr() as *mut c_char);
    } else {
        if (*addr_of!(g_gametype)).integer == GT_SIEGE
            && (*addr_of!(gSiegeRoundBegun) == QFALSE || *addr_of!(gSiegeRoundEnded) != QFALSE)
        {
            SetTeamQuick(ent, TEAM_SPECTATOR, QFALSE);
        }

        if ((*ent).r.svFlags & SVF_BOT) != 0 && (*addr_of!(g_gametype)).integer != GT_SIEGE {
            let saber_val: *mut c_char = Info_ValueForKey(userinfo.as_ptr(), c"saber1".as_ptr());
            let saber2_val: *mut c_char = Info_ValueForKey(userinfo.as_ptr(), c"saber2".as_ptr());

            if saber_val.is_null() || *saber_val == 0 {
                //blah, set em up with a random saber
                let r = bg_lib::rand() % 50;
                let mut sab1 = [0 as c_char; 1024];
                let mut sab2 = [0 as c_char; 1024];

                if r <= 17 {
                    strcpy(sab1.as_mut_ptr(), c"Katarn".as_ptr());
                    strcpy(sab2.as_mut_ptr(), c"none".as_ptr());
                } else if r <= 34 {
                    strcpy(sab1.as_mut_ptr(), c"Katarn".as_ptr());
                    strcpy(sab2.as_mut_ptr(), c"Katarn".as_ptr());
                } else {
                    strcpy(sab1.as_mut_ptr(), c"dual_1".as_ptr());
                    strcpy(sab2.as_mut_ptr(), c"none".as_ptr());
                }
                G_SetSaber(ent, 0, sab1.as_mut_ptr(), QFALSE);
                G_SetSaber(ent, 0, sab2.as_mut_ptr(), QFALSE);
                Info_SetValueForKey(userinfo.as_mut_ptr(), c"saber1".as_ptr(), sab1.as_ptr());
                Info_SetValueForKey(userinfo.as_mut_ptr(), c"saber2".as_ptr(), sab2.as_ptr());
                trap::SetUserinfo(
                    client_num,
                    &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy(),
                );
            } else {
                G_SetSaber(ent, 0, saber_val, QFALSE);
            }

            if !saber_val.is_null() && *saber_val != 0 && (saber2_val.is_null() || *saber2_val == 0)
            {
                G_SetSaber(ent, 0, c"none".as_ptr() as *mut c_char, QFALSE);
                Info_SetValueForKey(userinfo.as_mut_ptr(), c"saber2".as_ptr(), c"none".as_ptr());
                trap::SetUserinfo(
                    client_num,
                    &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy(),
                );
            } else {
                G_SetSaber(ent, 0, saber2_val, QFALSE);
            }
        }

        // locate ent at a spawn point
        ClientSpawn(ent);
    }

    if (*client).sess.sessionTeam != TEAM_SPECTATOR {
        // send event
        let tent = G_TempEntity(&(*(*ent).client).ps.origin, EV_PLAYER_TELEPORT_IN);
        (*tent).s.clientNum = (*ent).s.clientNum;

        if (*addr_of!(g_gametype)).integer != GT_DUEL
            || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        {
            trap::SendServerCommand(
                -1,
                &format!(
                    "print \"{}^7 {}\n\"",
                    Sz((*client).pers.netname.as_ptr()),
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr(),
                        c"PLENTER".as_ptr(),
                    )),
                ),
            );
        }
    }
    G_LogPrintf(&format!("ClientBegin: {client_num}\n"));

    // count current clients and rank for scoreboard
    CalculateRanks();

    G_ClearClientLog(client_num);
}

/*
===========
ClientSpawn

Called every time a client is placed fresh in the world:
after the first ClientBegin, and after each respawn
Initializes all non-persistant parts of playerState
============
*/
/// `void ClientSpawn( gentity_t *ent )` (g_client.c:2920). Places a client fresh
/// in the world — re-reads/forces the saber loadout, selects a spawn point, clears
/// everything but the persistant data, sets up health/armor/weapons (incl. siege
/// class overrides and trueJedi/merc handling), links the entity, and runs a
/// settling client think/end-frame. No oracle (mutates entity/client state via
/// engine traps + global `level`/`g_entities`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-null; the
/// `level`/`g_entities`/cvar globals must be initialised.
pub unsafe fn ClientSpawn(ent: *mut gentity_t) {
    let index: c_int;
    let mut spawn_origin: vec3_t = [0.0; 3];
    let mut spawn_angles: vec3_t = [0.0; 3];
    let client: *mut gclient_t;
    let mut i: c_int;
    let saved: clientPersistant_t;
    let savedSess: clientSession_t;
    let mut persistant: [c_int; MAX_PERSISTANT] = [0; MAX_PERSISTANT];
    let mut spawnPoint: *mut gentity_t; // reassigned in the retry loop (do/while in C)
    let flags: c_int;
    let gameFlags: c_int;
    let savedPing: c_int;
    let accuracy_hits: c_int;
    let accuracy_shots: c_int;
    let eventSequence: c_int;
    let mut userinfo = [0 as c_char; MAX_INFO_STRING];
    let savedForce: forcedata_t;
    let saveSaberNum: c_int; // = ENTITYNUM_NONE;
    let wDisable: c_int; // = 0;
    let savedSiegeIndex: c_int; // = 0;
    let maxHealth: c_int;
    let mut saberSaved: [saberInfo_t; MAX_SABERS] = core::mem::zeroed();
    let mut l: c_int = 0;
    let mut g2WeaponPtrs: [*mut c_void; MAX_SABERS] = [null_mut(); MAX_SABERS];
    let mut value: *mut c_char;
    let mut saber: *mut c_char;
    let mut changedSaber: qboolean = QFALSE;
    let mut inSiegeWithClass: qboolean = QFALSE;

    index = ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;
    client = (*ent).client;

    //first we want the userinfo so we can see if we should update this client's saber -rww
    // trap_GetUserinfo( index, userinfo, sizeof(userinfo) );
    {
        let info = trap::GetUserinfo(index);
        let bytes = info.as_bytes();
        let n = bytes.len().min(MAX_INFO_STRING - 1);
        for k in 0..n {
            userinfo[k] = bytes[k] as c_char;
        }
        userinfo[n] = 0;
    }

    while l < MAX_SABERS as c_int {
        match l {
            0 => {
                saber = (*(*ent).client).sess.saberType.as_mut_ptr();
            }
            1 => {
                saber = (*(*ent).client).sess.saber2Type.as_mut_ptr();
            }
            _ => {
                saber = null_mut();
            }
        }

        value = Info_ValueForKey(userinfo.as_ptr(), va(format_args!("saber{}", l + 1)));
        if !saber.is_null()
            && !value.is_null()
            && (Q_stricmp(value, saber) != 0
                || *saber == 0
                || (*(*ent).client).saber[0].model[0] == 0)
        {
            //doesn't match up (or our session saber is BS), we want to try setting it
            if G_SetSaber(ent, l, value, QFALSE) != QFALSE {
                changedSaber = QTRUE;
            } else if *saber == 0 || (*(*ent).client).saber[0].model[0] == 0 {
                //Well, we still want to say they changed then (it means this is siege and we have some overrides)
                changedSaber = QTRUE;
            }
        }
        l += 1;
    }

    if changedSaber != QFALSE {
        //make sure our new info is sent out to all the other clients, and give us a valid stance
        ClientUserinfoChanged((*ent).s.number);

        //make sure the saber models are updated
        G_SaberModelSetup(ent);

        l = 0;
        while l < MAX_SABERS as c_int {
            //go through and make sure both sabers match the userinfo
            match l {
                0 => {
                    saber = (*(*ent).client).sess.saberType.as_mut_ptr();
                }
                1 => {
                    saber = (*(*ent).client).sess.saber2Type.as_mut_ptr();
                }
                _ => {
                    saber = null_mut();
                }
            }

            value = Info_ValueForKey(userinfo.as_ptr(), va(format_args!("saber{}", l + 1)));

            if Q_stricmp(value, saber) != 0 {
                //they don't match up, force the user info
                Info_SetValueForKey(
                    userinfo.as_mut_ptr(),
                    va(format_args!("saber{}", l + 1)),
                    saber,
                );
                // trap_SetUserinfo( ent->s.number, userinfo );
                trap::SetUserinfo(
                    (*ent).s.number,
                    &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy(),
                );
            }
            l += 1;
        }

        if (*(*ent).client).saber[0].model[0] != 0 && (*(*ent).client).saber[1].model[0] != 0 {
            //dual
            (*(*ent).client).ps.fd.saberAnimLevelBase = SS_DUAL;
            (*(*ent).client).ps.fd.saberAnimLevel = SS_DUAL;
            (*(*ent).client).ps.fd.saberDrawAnimLevel = SS_DUAL;
        } else if (*(*ent).client).saber[0].saberFlags & SFL_TWO_HANDED != 0 {
            //staff
            (*(*ent).client).ps.fd.saberAnimLevel = SS_STAFF;
            (*(*ent).client).ps.fd.saberDrawAnimLevel = SS_STAFF;
        } else {
            if (*(*ent).client).sess.saberLevel < SS_FAST {
                (*(*ent).client).sess.saberLevel = SS_FAST;
            } else if (*(*ent).client).sess.saberLevel > SS_STRONG {
                (*(*ent).client).sess.saberLevel = SS_STRONG;
            }
            (*(*ent).client).ps.fd.saberAnimLevelBase = (*(*ent).client).sess.saberLevel;
            (*(*ent).client).ps.fd.saberAnimLevel = (*(*ent).client).sess.saberLevel;
            (*(*ent).client).ps.fd.saberDrawAnimLevel = (*(*ent).client).sess.saberLevel;

            if (*addr_of!(g_gametype)).integer != GT_SIEGE
                && (*(*ent).client).ps.fd.saberAnimLevel
                    > (*(*ent).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]
            {
                let lvl = (*(*ent).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize];
                (*(*ent).client).ps.fd.saberAnimLevelBase = lvl;
                (*(*ent).client).ps.fd.saberAnimLevel = lvl;
                (*(*ent).client).ps.fd.saberDrawAnimLevel = lvl;
                (*(*ent).client).sess.saberLevel = lvl;
            }
        }
        if (*addr_of!(g_gametype)).integer != GT_SIEGE {
            //let's just make sure the styles we chose are cool
            if WP_SaberStyleValidForSaber(
                addr_of_mut!((*(*ent).client).saber[0]),
                addr_of_mut!((*(*ent).client).saber[1]),
                (*(*ent).client).ps.saberHolstered,
                (*(*ent).client).ps.fd.saberAnimLevel,
            ) == QFALSE
            {
                WP_UseFirstValidSaberStyle(
                    addr_of_mut!((*(*ent).client).saber[0]),
                    addr_of_mut!((*(*ent).client).saber[1]),
                    (*(*ent).client).ps.saberHolstered,
                    addr_of_mut!((*(*ent).client).ps.fd.saberAnimLevel),
                );
                // C: saberAnimLevelBase = saberCycleQueue = saberAnimLevel
                let lvl = (*(*ent).client).ps.fd.saberAnimLevel;
                (*(*ent).client).ps.fd.saberAnimLevelBase = lvl;
                (*(*ent).client).saberCycleQueue = lvl;
            }
        }
    }
    let _ = l; // C: `l = 0;` — reset is dead here (each following loop re-inits l)

    if (*client).ps.fd.forceDoInit != 0 {
        //force a reread of force powers
        WP_InitForcePowers(ent);
        (*client).ps.fd.forceDoInit = 0;
    }

    if (*(*ent).client).ps.fd.saberAnimLevel != SS_STAFF
        && (*(*ent).client).ps.fd.saberAnimLevel != SS_DUAL
        && (*(*ent).client).ps.fd.saberAnimLevel == (*(*ent).client).ps.fd.saberDrawAnimLevel
        && (*(*ent).client).ps.fd.saberAnimLevel == (*(*ent).client).sess.saberLevel
    {
        if (*(*ent).client).sess.saberLevel < SS_FAST {
            (*(*ent).client).sess.saberLevel = SS_FAST;
        } else if (*(*ent).client).sess.saberLevel > SS_STRONG {
            (*(*ent).client).sess.saberLevel = SS_STRONG;
        }
        (*(*ent).client).ps.fd.saberAnimLevel = (*(*ent).client).sess.saberLevel;
        (*(*ent).client).ps.fd.saberDrawAnimLevel = (*(*ent).client).sess.saberLevel;

        if (*addr_of!(g_gametype)).integer != GT_SIEGE
            && (*(*ent).client).ps.fd.saberAnimLevel
                > (*(*ent).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]
        {
            let lvl = (*(*ent).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize];
            (*(*ent).client).ps.fd.saberAnimLevel = lvl;
            (*(*ent).client).ps.fd.saberDrawAnimLevel = lvl;
            (*(*ent).client).sess.saberLevel = lvl;
        }
    }

    // find a spawn point
    // do it before setting health back up, so farthest
    // ranging doesn't count this client
    if (*client).sess.sessionTeam == TEAM_SPECTATOR {
        spawnPoint = SelectSpectatorSpawnPoint(&mut spawn_origin, &mut spawn_angles);
    } else if (*addr_of!(g_gametype)).integer == GT_CTF || (*addr_of!(g_gametype)).integer == GT_CTY
    {
        // all base oriented team games use the CTF spawn points
        spawnPoint = SelectCTFSpawnPoint(
            (*client).sess.sessionTeam,
            (*client).pers.teamState.state,
            &mut spawn_origin,
            &mut spawn_angles,
        );
    } else if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        spawnPoint = SelectSiegeSpawnPoint(
            (*client).siegeClass,
            (*client).sess.sessionTeam,
            (*client).pers.teamState.state,
            &mut spawn_origin,
            &mut spawn_angles,
        );
    } else {
        loop {
            if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
                spawnPoint = SelectDuelSpawnPoint(
                    (*client).sess.duelTeam,
                    &(*client).ps.origin,
                    &mut spawn_origin,
                    &mut spawn_angles,
                );
            } else if (*addr_of!(g_gametype)).integer == GT_DUEL {
                // duel
                spawnPoint = SelectDuelSpawnPoint(
                    DUELTEAM_SINGLE,
                    &(*client).ps.origin,
                    &mut spawn_origin,
                    &mut spawn_angles,
                );
            } else {
                // the first spawn should be at a good looking spot
                if (*client).pers.initialSpawn == QFALSE && (*client).pers.localClient != QFALSE {
                    (*client).pers.initialSpawn = QTRUE;
                    spawnPoint = SelectInitialSpawnPoint(
                        &mut spawn_origin,
                        &mut spawn_angles,
                        (*client).sess.sessionTeam,
                    );
                } else {
                    // don't spawn near existing origin if possible
                    spawnPoint = SelectSpawnPoint(
                        &(*client).ps.origin,
                        &mut spawn_origin,
                        &mut spawn_angles,
                        (*client).sess.sessionTeam,
                    );
                }
            }

            // Tim needs to prevent bots from spawning at the initial point
            // on q3dm0...
            if ((*spawnPoint).flags & FL_NO_BOTS) != 0 && ((*ent).r.svFlags & SVF_BOT) != 0 {
                continue; // try again
            }
            // just to be symetric, we have a nohumans option...
            if ((*spawnPoint).flags & FL_NO_HUMANS) != 0 && ((*ent).r.svFlags & SVF_BOT) == 0 {
                continue; // try again
            }

            break;
        }
    }
    (*client).pers.teamState.state = TEAM_ACTIVE;

    // toggle the teleport bit so the client knows to not lerp
    // and never clear the voted flag
    flags = (*(*ent).client).ps.eFlags & EF_TELEPORT_BIT;
    let flags = flags ^ EF_TELEPORT_BIT;
    gameFlags = ((*(*ent).client).mGameFlags & (PSG_VOTED | PSG_TEAMVOTED) as c_uint) as c_int;

    // clear everything but the persistant data

    saved = (*client).pers;
    savedSess = (*client).sess;
    savedPing = (*client).ps.ping;
    //	savedAreaBits = client->areabits;
    accuracy_hits = (*client).accuracy_hits;
    accuracy_shots = (*client).accuracy_shots;
    for i in 0..MAX_PERSISTANT {
        persistant[i] = (*client).ps.persistant[i];
    }
    eventSequence = (*client).ps.eventSequence;

    savedForce = (*client).ps.fd;

    saveSaberNum = (*client).ps.saberEntityNum;

    savedSiegeIndex = (*client).siegeClass;

    l = 0;
    while l < MAX_SABERS as c_int {
        saberSaved[l as usize] = (*client).saber[l as usize];
        g2WeaponPtrs[l as usize] = (*client).weaponGhoul2[l as usize];
        l += 1;
    }

    i = 0;
    while i < HL_MAX {
        (*ent).locationDamage[i as usize] = 0;
        i += 1;
    }

    core::ptr::write_bytes(client, 0, 1); // memset (client, 0, sizeof(*client)); // bk FIXME: Com_Memset?
    (*client).bodyGrabIndex = ENTITYNUM_NONE;

    //Get the skin RGB based on his userinfo
    value = Info_ValueForKey(userinfo.as_ptr(), c"char_color_red".as_ptr());
    if !value.is_null() {
        (*client).ps.customRGBA[0] = atoi(value);
    } else {
        (*client).ps.customRGBA[0] = 255;
    }

    value = Info_ValueForKey(userinfo.as_ptr(), c"char_color_green".as_ptr());
    if !value.is_null() {
        (*client).ps.customRGBA[1] = atoi(value);
    } else {
        (*client).ps.customRGBA[1] = 255;
    }

    value = Info_ValueForKey(userinfo.as_ptr(), c"char_color_blue".as_ptr());
    if !value.is_null() {
        (*client).ps.customRGBA[2] = atoi(value);
    } else {
        (*client).ps.customRGBA[2] = 255;
    }

    if ((*client).ps.customRGBA[0] + (*client).ps.customRGBA[1] + (*client).ps.customRGBA[2]) < 100
    {
        //hmm, too dark!
        (*client).ps.customRGBA[0] = 255;
        (*client).ps.customRGBA[1] = 255;
        (*client).ps.customRGBA[2] = 255;
    }

    (*client).ps.customRGBA[3] = 255;

    (*client).siegeClass = savedSiegeIndex;

    l = 0;
    while l < MAX_SABERS as c_int {
        (*client).saber[l as usize] = saberSaved[l as usize];
        (*client).weaponGhoul2[l as usize] = g2WeaponPtrs[l as usize];
        l += 1;
    }

    //or the saber ent num
    (*client).ps.saberEntityNum = saveSaberNum;
    (*client).saberStoredIndex = saveSaberNum;

    (*client).ps.fd = savedForce;

    (*client).ps.duelIndex = ENTITYNUM_NONE;

    //spawn with 100
    (*client).ps.jetpackFuel = 100;
    (*client).ps.cloakFuel = 100;

    (*client).pers = saved;
    (*client).sess = savedSess;
    (*client).ps.ping = savedPing;
    //	client->areabits = savedAreaBits;
    (*client).accuracy_hits = accuracy_hits;
    (*client).accuracy_shots = accuracy_shots;
    (*client).lastkilled_client = -1;

    for i in 0..MAX_PERSISTANT {
        (*client).ps.persistant[i] = persistant[i];
    }
    (*client).ps.eventSequence = eventSequence;
    // increment the spawncount so the client will detect the respawn
    (*client).ps.persistant[PERS_SPAWN_COUNT as usize] += 1;
    (*client).ps.persistant[PERS_TEAM as usize] = (*client).sess.sessionTeam;

    (*client).airOutTime = (*addr_of!(level)).time + 12000;

    // set max health
    if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).siegeClass != -1 {
        let scl: *const siegeClass_t = &(*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize];
        maxHealth = if (*scl).maxhealth != 0 {
            (*scl).maxhealth
        } else {
            100
        };
    } else {
        maxHealth = 100;
    }
    (*client).pers.maxHealth = maxHealth; //atoi( Info_ValueForKey( userinfo, "handicap" ) );
    if (*client).pers.maxHealth < 1 || (*client).pers.maxHealth > maxHealth {
        (*client).pers.maxHealth = 100;
    }
    // clear entity values
    (*client).ps.stats[STAT_MAX_HEALTH as usize] = (*client).pers.maxHealth;
    (*client).ps.eFlags = flags;
    (*client).mGameFlags = gameFlags as c_uint;

    (*ent).s.groundEntityNum = ENTITYNUM_NONE;
    (*ent).client = (*addr_of!(level)).clients.offset(index as isize);
    (*ent).playerState = &mut (*(*ent).client).ps;
    (*ent).takedamage = QTRUE;
    (*ent).inuse = QTRUE;
    (*ent).classname = c"player".as_ptr() as *mut c_char;
    (*ent).r.contents = CONTENTS_BODY;
    (*ent).clipmask = MASK_PLAYERSOLID;
    (*ent).die = Some(player_die);
    (*ent).waterlevel = 0;
    (*ent).watertype = 0;
    (*ent).flags = 0;

    VectorCopy(&playerMins, &mut (*ent).r.mins);
    VectorCopy(&playerMaxs, &mut (*ent).r.maxs);
    (*client).ps.crouchheight = CROUCH_MAXS_2;
    (*client).ps.standheight = DEFAULT_MAXS_2;

    (*client).ps.clientNum = index;
    //give default weapons
    (*client).ps.stats[STAT_WEAPONS as usize] = 1 << WP_NONE;

    if (*addr_of!(g_gametype)).integer == GT_DUEL || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        wDisable = (*addr_of!(g_duelWeaponDisable)).integer;
    } else {
        wDisable = (*addr_of!(g_weaponDisable)).integer;
    }

    if (*addr_of!(g_gametype)).integer != GT_HOLOCRON
        && (*addr_of!(g_gametype)).integer != GT_JEDIMASTER
        && HasSetSaberOnly() == QFALSE
        && AllForceDisabled((*addr_of!(g_forcePowerDisable)).integer) == QFALSE
        && (*addr_of!(g_trueJedi)).integer != 0
    {
        if (*addr_of!(g_gametype)).integer >= GT_TEAM
            && ((*client).sess.sessionTeam == TEAM_BLUE || (*client).sess.sessionTeam == TEAM_RED)
        {
            //In Team games, force one side to be merc and other to be jedi
            if (*addr_of!(level)).numPlayingClients > 0 {
                //already someone in the game
                let mut forceTeam: c_int = TEAM_SPECTATOR;
                let clients = (*addr_of!(level)).clients;
                let maxclients = (*addr_of!(level)).maxclients;
                let mut i: c_int = 0;
                while i < maxclients {
                    let cl = clients.offset(i as isize);
                    if (*cl).pers.connected == CON_DISCONNECTED {
                        i += 1;
                        continue;
                    }
                    if (*cl).sess.sessionTeam == TEAM_BLUE || (*cl).sess.sessionTeam == TEAM_RED {
                        //in-game
                        if WP_HasForcePowers(&(*cl).ps) != QFALSE {
                            //this side is using force
                            forceTeam = (*cl).sess.sessionTeam;
                        } else {
                            //other team is using force
                            if (*cl).sess.sessionTeam == TEAM_BLUE {
                                forceTeam = TEAM_RED;
                            } else {
                                forceTeam = TEAM_BLUE;
                            }
                        }
                        break;
                    }
                    i += 1;
                }
                if WP_HasForcePowers(&(*client).ps) != QFALSE
                    && (*client).sess.sessionTeam != forceTeam
                {
                    //using force but not on right team, switch him over
                    let teamName = TeamName(forceTeam);
                    //client->sess.sessionTeam = forceTeam;
                    SetTeam(ent, teamName as *mut c_char);
                    return;
                }
            }
        }

        if WP_HasForcePowers(&(*client).ps) != QFALSE {
            (*client).ps.trueNonJedi = QFALSE;
            (*client).ps.trueJedi = QTRUE;
            //make sure they only use the saber
            (*client).ps.weapon = WP_SABER;
            (*client).ps.stats[STAT_WEAPONS as usize] = 1 << WP_SABER;
        } else {
            //no force powers set
            (*client).ps.trueNonJedi = QTRUE;
            (*client).ps.trueJedi = QFALSE;
            if wDisable == 0 || (wDisable & (1 << WP_BRYAR_PISTOL)) == 0 {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BRYAR_PISTOL;
            }
            if wDisable == 0 || (wDisable & (1 << WP_BLASTER)) == 0 {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BLASTER;
            }
            if wDisable == 0 || (wDisable & (1 << WP_BOWCASTER)) == 0 {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BOWCASTER;
            }
            (*client).ps.stats[STAT_WEAPONS as usize] &= !(1 << WP_SABER);
            (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_MELEE;
            (*client).ps.ammo[AMMO_POWERCELL as usize] =
                (*addr_of!(ammoData))[AMMO_POWERCELL as usize].max;
            (*client).ps.weapon = WP_BRYAR_PISTOL;
        }
    } else {
        //jediVmerc is incompatible with this gametype, turn it off!
        // trap_Cvar_Set( "g_jediVmerc", "0" );
        trap::Cvar_Set("g_jediVmerc", "0");
        if (*addr_of!(g_gametype)).integer == GT_HOLOCRON {
            //always get free saber level 1 in holocron
            (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_SABER; //these are precached in g_items, ClearRegisteredItems()
        } else {
            if (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] != 0 {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_SABER; //these are precached in g_items, ClearRegisteredItems()
            } else {
                //if you don't have saber attack rank then you don't get a saber
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_MELEE;
            }
        }

        if (*addr_of!(g_gametype)).integer != GT_SIEGE {
            if wDisable == 0 || (wDisable & (1 << WP_BRYAR_PISTOL)) == 0 {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BRYAR_PISTOL;
            } else if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER {
                (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BRYAR_PISTOL;
            }
        }

        if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER {
            (*client).ps.stats[STAT_WEAPONS as usize] &= !(1 << WP_SABER);
            (*client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_MELEE;
        }

        if ((*client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_SABER)) != 0 {
            (*client).ps.weapon = WP_SABER;
        } else if ((*client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_BRYAR_PISTOL)) != 0 {
            (*client).ps.weapon = WP_BRYAR_PISTOL;
        } else {
            (*client).ps.weapon = WP_MELEE;
        }
    }

    /*
    client->ps.stats[STAT_HOLDABLE_ITEMS] |= ( 1 << HI_BINOCULARS );
    client->ps.stats[STAT_HOLDABLE_ITEM] = BG_GetItemIndexByTag(HI_BINOCULARS, IT_HOLDABLE);
    */

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*client).siegeClass != -1
        && (*client).sess.sessionTeam != TEAM_SPECTATOR
    {
        //well then, we will use a custom weaponset for our class
        let mut m: c_int = 0;

        (*client).ps.stats[STAT_WEAPONS as usize] =
            (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].weapons;

        if ((*client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_SABER)) != 0 {
            (*client).ps.weapon = WP_SABER;
        } else if ((*client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_BRYAR_PISTOL)) != 0 {
            (*client).ps.weapon = WP_BRYAR_PISTOL;
        } else {
            (*client).ps.weapon = WP_MELEE;
        }
        inSiegeWithClass = QTRUE;

        while m < WP_NUM_WEAPONS {
            if ((*client).ps.stats[STAT_WEAPONS as usize] & (1 << m)) != 0 {
                if (*client).ps.weapon != WP_SABER {
                    //try to find the highest ranking weapon we have
                    if m > (*client).ps.weapon {
                        (*client).ps.weapon = m;
                    }
                }

                if m >= WP_BRYAR_PISTOL {
                    //Max his ammo out for all the weapons he has.
                    if (*addr_of!(g_gametype)).integer == GT_SIEGE && m == WP_ROCKET_LAUNCHER {
                        //don't give full ammo!
                        //FIXME: extern this and check it when getting ammo from supplier, pickups or ammo stations!
                        if (*client).siegeClass != -1
                            && ((*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize]
                                .classflags
                                & (1 << CFL_SINGLE_ROCKET))
                                != 0
                        {
                            (*client).ps.ammo
                                [(*addr_of!(weaponData))[m as usize].ammoIndex as usize] = 1;
                        } else {
                            (*client).ps.ammo
                                [(*addr_of!(weaponData))[m as usize].ammoIndex as usize] = 10;
                        }
                    } else {
                        if (*addr_of!(g_gametype)).integer == GT_SIEGE
                            && (*client).siegeClass != -1
                            && ((*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize]
                                .classflags
                                & (1 << CFL_EXTRA_AMMO))
                                != 0
                        {
                            //double ammo
                            let ai = (*addr_of!(weaponData))[m as usize].ammoIndex;
                            (*client).ps.ammo[ai as usize] =
                                (*addr_of!(ammoData))[ai as usize].max * 2;
                            (*client).ps.eFlags |= EF_DOUBLE_AMMO;
                        } else {
                            let ai = (*addr_of!(weaponData))[m as usize].ammoIndex;
                            (*client).ps.ammo[ai as usize] = (*addr_of!(ammoData))[ai as usize].max;
                        }
                    }
                }
            }
            m += 1;
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*client).siegeClass != -1
        && (*client).sess.sessionTeam != TEAM_SPECTATOR
    {
        //use class-specified inventory
        (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] =
            (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].invenItems;
        (*client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
    } else {
        (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] = 0;
        (*client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*client).siegeClass != -1
        && (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].powerups != 0
        && (*client).sess.sessionTeam != TEAM_SPECTATOR
    {
        //this class has some start powerups
        i = 0;
        while i < PW_NUM_POWERUPS {
            if ((*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].powerups & (1 << i)) != 0
            {
                (*client).ps.powerups[i as usize] = Q3_INFINITE;
            }
            i += 1;
        }
    }

    if (*client).sess.sessionTeam == TEAM_SPECTATOR {
        (*client).ps.stats[STAT_WEAPONS as usize] = 0;
        (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] = 0;
        (*client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
    }

    // nmckenzie: DESERT_SIEGE... or well, siege generally.  This was over-writing the max value, which was NOT good for siege.
    if inSiegeWithClass == QFALSE {
        (*client).ps.ammo[AMMO_BLASTER as usize] = 100; //ammoData[AMMO_BLASTER].max; //100 seems fair.
    }
    //	client->ps.ammo[AMMO_POWERCELL] = ammoData[AMMO_POWERCELL].max;
    //	client->ps.ammo[AMMO_FORCE] = ammoData[AMMO_FORCE].max;
    //	client->ps.ammo[AMMO_METAL_BOLTS] = ammoData[AMMO_METAL_BOLTS].max;
    //	client->ps.ammo[AMMO_ROCKETS] = ammoData[AMMO_ROCKETS].max;
    /*
    client->ps.stats[STAT_WEAPONS] = ( 1 << WP_BRYAR_PISTOL);
    if ( g_gametype.integer == GT_TEAM ) {
        client->ps.ammo[WP_BRYAR_PISTOL] = 50;
    } else {
        client->ps.ammo[WP_BRYAR_PISTOL] = 100;
    }
    */
    (*client).ps.rocketLockIndex = ENTITYNUM_NONE;
    (*client).ps.rocketLockTime = 0.0;

    //rww - Set here to initialize the circling seeker drone to off.
    //A quick note about this so I don't forget how it works again:
    //ps.genericEnemyIndex is kept in sync between the server and client.
    //When it gets set then an entitystate value of the same name gets
    //set along with an entitystate flag in the shared bg code. Which
    //is why a value needs to be both on the player state and entity state.
    //(it doesn't seem to just carry over the entitystate value automatically
    //because entity state value is derived from player state data or some
    //such)
    (*client).ps.genericEnemyIndex = -1;

    (*client).ps.isJediMaster = QFALSE;

    if (*client).ps.fallingToDeath != 0 {
        (*client).ps.fallingToDeath = 0;
        (*client).noCorpse = QTRUE;
    }

    //Do per-spawn force power initialization
    WP_SpawnInitForcePowers(ent);

    // health will count down towards max_health
    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*client).siegeClass != -1
        && (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].starthealth != 0
    {
        //class specifies a start health, so use it
        (*client).ps.stats[STAT_HEALTH as usize] =
            (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].starthealth;
        (*ent).health = (*client).ps.stats[STAT_HEALTH as usize];
    } else if (*addr_of!(g_gametype)).integer == GT_DUEL
        || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        //only start with 100 health in Duel
        if (*addr_of!(g_gametype)).integer == GT_POWERDUEL
            && (*client).sess.duelTeam == DUELTEAM_LONE
        {
            if (*addr_of!(g_duel_fraglimit)).integer != 0 {
                let h = (*addr_of!(g_powerDuelStartHealth)).integer
                    - (((*addr_of!(g_powerDuelStartHealth)).integer
                        - (*addr_of!(g_powerDuelEndHealth)).integer) as f32
                        * (*client).sess.wins as f32
                        / (*addr_of!(g_duel_fraglimit)).integer as f32)
                        as c_int;
                (*client).ps.stats[STAT_MAX_HEALTH as usize] = h;
                (*client).ps.stats[STAT_HEALTH as usize] = h;
                (*ent).health = h;
            } else {
                (*client).ps.stats[STAT_MAX_HEALTH as usize] = 150;
                (*client).ps.stats[STAT_HEALTH as usize] = 150;
                (*ent).health = 150;
            }
        } else {
            (*client).ps.stats[STAT_MAX_HEALTH as usize] = 100;
            (*client).ps.stats[STAT_HEALTH as usize] = 100;
            (*ent).health = 100;
        }
    } else if (*client).ps.stats[STAT_MAX_HEALTH as usize] <= 100 {
        let h = ((*client).ps.stats[STAT_MAX_HEALTH as usize] as f32 * 1.25) as c_int;
        (*client).ps.stats[STAT_HEALTH as usize] = h;
        (*ent).health = h;
    } else if (*client).ps.stats[STAT_MAX_HEALTH as usize] < 125 {
        (*client).ps.stats[STAT_HEALTH as usize] = 125;
        (*ent).health = 125;
    } else {
        (*client).ps.stats[STAT_HEALTH as usize] = (*client).ps.stats[STAT_MAX_HEALTH as usize];
        (*ent).health = (*client).ps.stats[STAT_MAX_HEALTH as usize];
    }

    // Start with a small amount of armor as well.
    if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).siegeClass != -1
    /*&&
    bgSiegeClasses[client->siegeClass].startarmor*/
    {
        //class specifies a start armor amount, so use it
        (*client).ps.stats[STAT_ARMOR as usize] =
            (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].startarmor;
    } else if (*addr_of!(g_gametype)).integer == GT_DUEL
        || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        //no armor in duel
        (*client).ps.stats[STAT_ARMOR as usize] = 0;
    } else {
        (*client).ps.stats[STAT_ARMOR as usize] =
            ((*client).ps.stats[STAT_MAX_HEALTH as usize] as f32 * 0.25) as c_int;
    }

    G_SetOrigin(ent, &spawn_origin);
    VectorCopy(&spawn_origin, &mut (*client).ps.origin);

    // the respawned flag will be cleared after the attack and jump keys come up
    (*client).ps.pm_flags |= PMF_RESPAWNED;

    // trap_GetUsercmd( client - level.clients, &ent->client->pers.cmd );
    trap::GetUsercmd(
        client.offset_from((*addr_of!(level)).clients) as c_int,
        &mut (*(*ent).client).pers.cmd as *mut usercmd_t,
    );
    SetClientViewAngle(ent, &spawn_angles);

    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
    } else {
        G_KillBox(ent);
        trap::LinkEntity(ent);

        // force the base weapon up
        //client->ps.weapon = WP_BRYAR_PISTOL;
        //client->ps.weaponstate = FIRST_WEAPON;
        if (*client).ps.weapon <= WP_NONE {
            (*client).ps.weapon = WP_BRYAR_PISTOL;
        }

        (*client).ps.torsoTimer = 0;
        (*client).ps.legsTimer = 0;

        if (*client).ps.weapon == WP_SABER {
            G_SetAnim(
                ent,
                null_mut(),
                SETANIM_BOTH,
                BOTH_STAND1TO2,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS,
                0,
            );
        } else {
            G_SetAnim(
                ent,
                null_mut(),
                SETANIM_TORSO,
                TORSO_RAISEWEAP1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS,
                0,
            );
            (*client).ps.legsAnim = (*addr_of!(WeaponReadyAnim))[(*client).ps.weapon as usize];
        }
        (*client).ps.weaponstate = WEAPON_RAISING;
        (*client).ps.weaponTime = (*client).ps.torsoTimer;
    }

    // don't allow full run speed for a bit
    (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
    (*client).ps.pm_time = 100;

    (*client).respawnTime = (*addr_of!(level)).time;
    (*client).inactivityTime = (*addr_of!(level)).time + (*addr_of!(g_inactivity)).integer * 1000;
    (*client).latched_buttons = 0;

    if (*addr_of!(level)).intermissiontime != 0 {
        MoveClientToIntermission(ent);
    } else {
        // fire the targets of the spawn point
        G_UseTargets(spawnPoint, ent);

        // select the highest weapon number available, after any
        // spawn given items have fired
        /*
        client->ps.weapon = 1;
        for ( i = WP_NUM_WEAPONS - 1 ; i > 0 ; i-- ) {
            if ( client->ps.stats[STAT_WEAPONS] & ( 1 << i ) ) {
                client->ps.weapon = i;
                break;
            }
        }
        */
    }

    //set teams for NPCs to recognize
    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //Imperial (team1) team is allied with "enemy" NPCs in this mode
        if (*client).sess.sessionTeam == SIEGETEAM_TEAM1 {
            (*client).playerTeam = NPCTEAM_ENEMY;
            (*ent).s.teamowner = NPCTEAM_ENEMY as c_int;
            (*client).enemyTeam = NPCTEAM_PLAYER;
        } else {
            (*client).playerTeam = NPCTEAM_PLAYER;
            (*ent).s.teamowner = NPCTEAM_PLAYER as c_int;
            (*client).enemyTeam = NPCTEAM_ENEMY;
        }
    } else {
        (*client).playerTeam = NPCTEAM_PLAYER;
        (*ent).s.teamowner = NPCTEAM_PLAYER as c_int;
        (*client).enemyTeam = NPCTEAM_ENEMY;
    }

    /*
    //scaling for the power duel opponent
    if (g_gametype.integer == GT_POWERDUEL &&
        client->sess.duelTeam == DUELTEAM_LONE)
    {
        client->ps.iModelScale = 125;
        VectorSet(ent->modelScale, 1.25f, 1.25f, 1.25f);
    }
    */
    //Disabled. At least for now. Not sure if I'll want to do it or not eventually.

    // run a client frame to drop exactly to the floor,
    // initialize animations and other things
    (*client).ps.commandTime = (*addr_of!(level)).time - 100;
    (*(*ent).client).pers.cmd.serverTime = (*addr_of!(level)).time;
    ClientThink(
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        null_mut(),
    );

    // positively link the client, even if the command times are weird
    if (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR {
        BG_PlayerStateToEntityState(&mut (*client).ps, &mut (*ent).s, QTRUE);
        VectorCopy(&(*(*ent).client).ps.origin, &mut (*ent).r.currentOrigin);
        trap::LinkEntity(ent);
    }

    if (*addr_of!(g_spawnInvulnerability)).integer != 0 {
        (*(*ent).client).ps.eFlags |= EF_INVULNERABLE;
        (*(*ent).client).invulnerableTimer =
            (*addr_of!(level)).time + (*addr_of!(g_spawnInvulnerability)).integer;
    }

    // run the presend to set anything else
    ClientEndFrame(ent);

    // clear entity state values
    BG_PlayerStateToEntityState(&mut (*client).ps, &mut (*ent).s, QTRUE);

    //rww - make sure client has a valid icarus instance
    trap::ICARUS_FreeEnt(ent);
    trap::ICARUS_InitEnt(ent);
}

/*
================
respawn
================
*/
/// `void respawn( gentity_t *ent )` (g_client.c:1076). Respawns a client: keeps the
/// corpse in the body queue, then either forces escaping/power-duel losers into free
/// spectate (and flags them a loser), runs the GT_SIEGE respawn-wave gate (temp-spectate
/// + `EV_SIEGESPEC` countdown), or re-spawns the client in place with a teleport-in
/// effect. No oracle (mutates entity/client state via `ClientSpawn`/traps + reads the
/// `level`/cvar globals).
///
/// The GT_SIEGE non-wave branch calls the real [`SiegeRespawn`] port (g_saga.rs).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `client` is non-null; the
/// `level`/`g_entities`/cvar globals must be initialised.
pub unsafe fn respawn(ent: *mut gentity_t) {
    MaintainBodyQueue(ent);

    if *addr_of!(gEscaping) != QFALSE || (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
        (*(*ent).client).sess.sessionTeam = TEAM_SPECTATOR;
        (*(*ent).client).sess.spectatorState = SPECTATOR_FREE;
        (*(*ent).client).sess.spectatorClient = 0;

        (*(*ent).client).pers.teamState.state = TEAM_BEGIN;
        (*(*ent).client).sess.spectatorTime = (*addr_of!(level)).time;
        ClientSpawn(ent);
        (*(*ent).client).iAmALoser = QTRUE;
        return;
    }

    trap::UnlinkEntity(ent);

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        if (*addr_of!(g_siegeRespawn)).integer != 0 {
            if (*(*ent).client).tempSpectate <= (*addr_of!(level)).time {
                let mut min_del: c_int = (*addr_of!(g_siegeRespawn)).integer * 2000;
                if min_del < 20000 {
                    min_del = 20000;
                }
                (*(*ent).client).tempSpectate = (*addr_of!(level)).time + min_del;
                // C: ent->health = ent->client->ps.stats[STAT_HEALTH] = 1;
                (*(*ent).client).ps.stats[STAT_HEALTH as usize] = 1;
                (*ent).health = 1;
                (*(*ent).client).ps.weapon = WP_NONE;
                (*(*ent).client).ps.stats[STAT_WEAPONS as usize] = 0;
                (*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] = 0;
                (*(*ent).client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
                (*ent).takedamage = QFALSE;
                trap::LinkEntity(ent);

                // Respawn time.
                if (*ent).s.number < MAX_CLIENTS as c_int {
                    let te = G_TempEntity(&(*(*ent).client).ps.origin, EV_SIEGESPEC);
                    (*te).s.time = *addr_of!(g_siegeRespawnCheck);
                    (*te).s.owner = (*ent).s.number;
                }

                return;
            }
        }
        SiegeRespawn(ent);
    } else {
        let tent: *mut gentity_t;

        ClientSpawn(ent);

        // add a teleportation effect
        tent = G_TempEntity(&(*(*ent).client).ps.origin, EV_PLAYER_TELEPORT_IN);
        (*tent).s.clientNum = (*ent).s.clientNum;
    }
}

/*
===========
ClientConnect

Called when a player begins connecting to the server.
Called again for every map change or tournement restart.

The session information will be valid after exit.

Return NULL if the client should be allowed, otherwise return
a string with the reason for denial.

Otherwise, the client will be sent the current gamestate
and will eventually get to ClientBegin.

firstTime will be qtrue the very first time a client connects
to the server machine, but qfalse on map changes and tournement
restarts.
============
*/
/// `char *ClientConnect( int clientNum, qboolean firstTime, qboolean isBot )`
/// (g_client.c:2235). Admits a connecting client: ban-filters their IP, wires up the
/// `gclient_t`, reads/initialises session data, applies siege/power-duel spectator
/// defaults, runs bot-connect, broadcasts the join, recomputes ranks, and fires the
/// `EV_CLIENTJOIN` broadcast event. Returns NULL to allow the connection, or a static
/// denial-reason C string. No oracle (mutates client/entity state + globals via traps).
///
/// # Safety
/// `client_num` must be a valid client index; the `g_entities`/`level`/cvar globals must
/// be initialised.
pub unsafe fn ClientConnect(
    client_num: c_int,
    first_time: qboolean,
    is_bot: qboolean,
) -> *mut c_char {
    let mut value: *mut c_char;
    let client: *mut gclient_t;
    let mut userinfo = [0 as c_char; MAX_INFO_STRING];
    let mut IPstring = [0 as c_char; 32];
    let ent: *mut gentity_t;
    let te: *mut gentity_t;

    ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(client_num as usize);

    // trap_GetUserinfo( clientNum, userinfo, sizeof( userinfo ) );
    {
        let info = trap::GetUserinfo(client_num);
        let bytes = info.as_bytes();
        let n = bytes.len().min(MAX_INFO_STRING - 1);
        for k in 0..n {
            userinfo[k] = bytes[k] as c_char;
        }
        userinfo[n] = 0;
    }

    // check to see if they are on the banned IP list
    value = Info_ValueForKey(userinfo.as_ptr(), c"ip".as_ptr());
    Q_strncpyz(IPstring.as_mut_ptr(), value, IPstring.len() as c_int);

    if G_FilterPacket(value) != QFALSE {
        return c"Banned.".as_ptr() as *mut c_char;
    }

    if (*ent).r.svFlags & SVF_BOT == 0 && is_bot == QFALSE && (*addr_of!(g_needpass)).integer != 0 {
        // check for a password
        value = Info_ValueForKey(userinfo.as_ptr(), c"password".as_ptr());
        if (*addr_of!(g_password)).string[0] != 0
            && Q_stricmp((*addr_of!(g_password)).string.as_ptr(), c"none".as_ptr()) != 0
            && strcmp((*addr_of!(g_password)).string.as_ptr(), value) != 0
        {
            static mut S_TEMP: [c_char; 1024] = [0; 1024];
            Q_strncpyz(
                addr_of_mut!(S_TEMP) as *mut c_char,
                G_GetStringEdString(c"MP_SVGAME".as_ptr(), c"INVALID_ESCAPE_TO_MAIN".as_ptr()),
                core::mem::size_of_val(&*addr_of!(S_TEMP)) as c_int,
            );
            return addr_of_mut!(S_TEMP) as *mut c_char; // return "Invalid password";
        }
    }

    // they can connect
    (*ent).client = (*addr_of!(level)).clients.add(client_num as usize);
    client = (*ent).client;

    //assign the pointer for bg entity access
    (*ent).playerState = addr_of_mut!((*(*ent).client).ps);

    core::ptr::write_bytes(client, 0, 1);

    (*client).pers.connected = CON_CONNECTING;

    // read or initialize the session data
    if first_time != QFALSE || (*addr_of!(level)).newSession != QFALSE {
        G_InitSessionData(client, userinfo.as_mut_ptr(), is_bot);
    }
    G_ReadSessionData(client);

    (*client).sess.IPstring[0] = 0;
    Q_strncpyz(
        (*client).sess.IPstring.as_mut_ptr(),
        IPstring.as_ptr(),
        (*client).sess.IPstring.len() as c_int,
    );

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (first_time != QFALSE || (*addr_of!(level)).newSession != QFALSE)
    {
        //if this is the first time then auto-assign a desired siege team and show briefing for that team
        (*client).sess.siegeDesiredTeam = 0; //PickTeam(ent->s.number);
                                             //don't just show it - they'll see it if they switch to a team on purpose.
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).sess.sessionTeam != TEAM_SPECTATOR {
        if first_time != QFALSE || (*addr_of!(level)).newSession != QFALSE {
            //start as spec
            (*client).sess.siegeDesiredTeam = (*client).sess.sessionTeam;
            (*client).sess.sessionTeam = TEAM_SPECTATOR;
        }
    } else if (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        && (*client).sess.sessionTeam != TEAM_SPECTATOR
    {
        (*client).sess.sessionTeam = TEAM_SPECTATOR;
    }

    if is_bot != QFALSE {
        (*ent).r.svFlags |= SVF_BOT;
        (*ent).inuse = QTRUE;
        if G_BotConnect(
            client_num,
            if first_time != QFALSE { QFALSE } else { QTRUE },
        ) == QFALSE
        {
            return c"BotConnectfailed".as_ptr() as *mut c_char;
        }
    }

    // get and distribute relevent paramters
    G_LogPrintf(&format!("ClientConnect: {client_num}\n"));
    ClientUserinfoChanged(client_num);
    G_LogPrintf(&format!(
        "{} connected with IP: {}\n",
        Sz((*client).pers.netname.as_ptr()),
        Sz((*client).sess.IPstring.as_ptr())
    ));

    // don't do the "xxx connected" messages if they were caried over from previous level
    if first_time != QFALSE {
        trap::SendServerCommand(
            -1,
            &format!(
                "print \"{}^7 {}\n\"",
                Sz((*client).pers.netname.as_ptr()),
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr(),
                    c"PLCONNECT".as_ptr()
                )),
            ),
        );
    }

    if (*addr_of!(g_gametype)).integer >= GT_TEAM && (*client).sess.sessionTeam != TEAM_SPECTATOR {
        BroadcastTeamChange(client, -1);
    }

    // count current clients and rank for scoreboard
    CalculateRanks();

    te = G_TempEntity(&*addr_of!(vec3_origin), EV_CLIENTJOIN);
    (*te).r.svFlags |= SVF_BROADCAST;
    (*te).s.eventParm = client_num;

    null_mut() // return NULL
}

/*
===========
ClientDisconnect

Called when a player drops from the server.
Will not be called between levels.

This should NOT be called directly by any game logic,
call trap_DropClient(), which will call this and do
server system housekeeping.
============
*/
/// `void ClientDisconnect( int clientNum )` (g_client.c:3804). Drops a client: stops
/// active force powers, mutes kill sounds, ejects from any vehicle, stops following
/// spectators, fires a teleport-out + tosses their items, awards a tourney win to the
/// surviving duelist, cleans up Ghoul2 instances, unlinks/clears the entity, and
/// recomputes ranks. No oracle (mutates entity/client state + globals via traps).
///
/// Calls the real [`G_RemoveQueuedBotBegin`] (g_bot.rs) / [`BotAIShutdownClient`]
/// (ai_main.rs) bot-subsystem ports and `TossClientItems` (g_combat).
///
/// # Safety
/// `client_num` must be a valid client index; the `g_entities`/`level` globals must be
/// initialised.
pub unsafe fn ClientDisconnect(client_num: c_int) {
    let ent: *mut gentity_t;
    let tent: *mut gentity_t;
    let mut i: c_int;

    // cleanup if we are kicking a bot that
    // hasn't spawned yet
    G_RemoveQueuedBotBegin(client_num);

    ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(client_num as usize);
    if (*ent).client.is_null() {
        return;
    }

    i = 0;

    while i < NUM_FORCE_POWERS as c_int {
        if ((*(*ent).client).ps.fd.forcePowersActive & (1 << i)) != 0 {
            WP_ForcePowerStop(ent, i);
        }
        i += 1;
    }

    i = TRACK_CHANNEL_1;

    while i < NUM_TRACK_CHANNELS {
        let k = (i - 50) as usize;
        if (*(*ent).client).ps.fd.killSoundEntIndex[k] != 0
            && (*(*ent).client).ps.fd.killSoundEntIndex[k] < MAX_GENTITIES as c_int
            && (*(*ent).client).ps.fd.killSoundEntIndex[k] > 0
        {
            G_MuteSound((*(*ent).client).ps.fd.killSoundEntIndex[k], CHAN_VOICE);
        }
        i += 1;
    }
    // i = 0; (vestigial in C — `i` is re-initialised by the following-clients loop below)

    if (*(*ent).client).ps.m_iVehicleNum != 0 {
        //tell it I'm getting off
        let veh = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*(*ent).client).ps.m_iVehicleNum as usize);

        if (*veh).inuse != QFALSE && !(*veh).client.is_null() && !(*veh).m_pVehicle.is_null() {
            let p_con = (*(*ent).client).pers.connected;

            (*(*ent).client).pers.connected = 0; // C: literal 0 (CON_DISCONNECTED)
            ((*(*(*veh).m_pVehicle).m_pVehicleInfo).Eject.unwrap())(
                (*veh).m_pVehicle,
                ent as *mut bgEntity_t,
                QTRUE,
            );
            (*(*ent).client).pers.connected = p_con;
        }
    }

    // stop any following clients
    i = 0;
    while i < (*addr_of!(level)).maxclients {
        let ci = (*addr_of!(level)).clients.add(i as usize);
        if (*ci).sess.sessionTeam == TEAM_SPECTATOR
            && (*ci).sess.spectatorState == SPECTATOR_FOLLOW
            && (*ci).sess.spectatorClient == client_num
        {
            StopFollowing(
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize),
            );
        }
        i += 1;
    }

    // send effect if they were completely connected
    if (*(*ent).client).pers.connected == CON_CONNECTED
        && (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR
    {
        tent = G_TempEntity(&(*(*ent).client).ps.origin, EV_PLAYER_TELEPORT_OUT);
        (*tent).s.clientNum = (*ent).s.clientNum;

        // They don't get to take powerups with them!
        // Especially important for stuff like CTF flags
        TossClientItems(ent);
    }

    G_LogPrintf(&format!("ClientDisconnect: {client_num}\n"));
    G_LogPrintf(&format!(
        "{} disconnected with IP: {}\n",
        Sz((*(*ent).client).pers.netname.as_ptr()),
        Sz((*(*ent).client).sess.IPstring.as_ptr())
    ));

    // if we are playing in tourney mode, give a win to the other player and clear his frags for this round
    if (*addr_of!(g_gametype)).integer == GT_DUEL
        && (*addr_of!(level)).intermissiontime == 0
        && (*addr_of!(level)).warmupTime == 0
    {
        if (*addr_of!(level)).sortedClients[1] == client_num {
            let win = (*addr_of!(level))
                .clients
                .add((*addr_of!(level)).sortedClients[0] as usize);
            (*win).ps.persistant[PERS_SCORE as usize] = 0;
            (*win).sess.wins += 1;
            ClientUserinfoChanged((*addr_of!(level)).sortedClients[0]);
        } else if (*addr_of!(level)).sortedClients[0] == client_num {
            let win = (*addr_of!(level))
                .clients
                .add((*addr_of!(level)).sortedClients[1] as usize);
            (*win).ps.persistant[PERS_SCORE as usize] = 0;
            (*win).sess.wins += 1;
            ClientUserinfoChanged((*addr_of!(level)).sortedClients[1]);
        }
    }

    if !(*ent).ghoul2.is_null() && trap::G2_HaveWeGhoul2Models((*ent).ghoul2) != QFALSE {
        trap::G2API_CleanGhoul2Models(addr_of_mut!((*ent).ghoul2));
    }
    i = 0;
    while i < MAX_SABERS as c_int {
        if !(*(*ent).client).weaponGhoul2[i as usize].is_null()
            && trap::G2_HaveWeGhoul2Models((*(*ent).client).weaponGhoul2[i as usize]) != QFALSE
        {
            trap::G2API_CleanGhoul2Models(addr_of_mut!((*(*ent).client).weaponGhoul2[i as usize]));
        }
        i += 1;
    }

    trap::UnlinkEntity(ent);
    (*ent).s.modelindex = 0;
    (*ent).inuse = QFALSE;
    (*ent).classname = c"disconnected".as_ptr() as *mut c_char;
    (*(*ent).client).pers.connected = CON_DISCONNECTED;
    (*(*ent).client).ps.persistant[PERS_TEAM as usize] = TEAM_FREE;
    (*(*ent).client).sess.sessionTeam = TEAM_FREE;
    (*ent).r.contents = 0;

    trap::SetConfigstring(CS_PLAYERS + client_num, "");

    CalculateRanks();

    if ((*ent).r.svFlags & SVF_BOT) != 0 {
        BotAIShutdownClient(client_num, QFALSE);
    }

    G_ClearClientLog(client_num);
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::codemp::game::g_local::gclient_s;
    use crate::oracle::{jka_ClientCleanName, jka_SetClientViewAngle};
    use core::mem::MaybeUninit;

    /// `SetClientViewAngle` over a spread of view angles and pre-existing command angles —
    /// negatives, large magnitudes that exercise the `ANGLE2SHORT` truncate-and-mask, and
    /// fractional values. Checked bit-exact against the extracted C: all three written
    /// arrays (`ps.delta_angles`, `s.angles`, `ps.viewangles`).
    #[test]
    fn setclientviewangle_matches_oracle() {
        let cases: &[(vec3_t, [i32; 3])] = &[
            ([0.0, 0.0, 0.0], [0, 0, 0]),
            ([90.0, 180.0, 270.0], [0, 0, 0]),
            ([-45.0, 360.0, 45.5], [100, -200, 16384]),
            ([725.25, -725.25, 0.5], [-1, 32767, -32768]),
            ([12.5, -370.0, 1000.0], [5000, 5000, 5000]),
        ];

        for (i, (angle, cmd_angles)) in cases.iter().enumerate() {
            // Build a zeroed gentity_t/gclient_s and wire the client pointer.
            let mut ent: gentity_t = unsafe { MaybeUninit::zeroed().assume_init() };
            let mut client: gclient_s = unsafe { MaybeUninit::zeroed().assume_init() };
            client.pers.cmd.angles = *cmd_angles;
            ent.client = &mut client;

            unsafe { SetClientViewAngle(&mut ent, angle) };

            let mut w_delta = [0i32; 3];
            let mut w_s = [0f32; 3];
            let mut w_view = [0f32; 3];
            unsafe {
                jka_SetClientViewAngle(
                    angle.as_ptr(),
                    cmd_angles.as_ptr(),
                    w_delta.as_mut_ptr(),
                    w_s.as_mut_ptr(),
                    w_view.as_mut_ptr(),
                );
            }

            assert_eq!(client.ps.delta_angles, w_delta, "case {i}: delta_angles");
            assert_eq!(ent.s.angles, w_s, "case {i}: s.angles");
            assert_eq!(client.ps.viewangles, w_view, "case {i}: viewangles");
        }
    }

    /// `ClientCleanName` over names exercising every branch: leading spaces, runs of >3
    /// spaces, ^0-black drop, ^N keep, a solo trailing carat, a black-only name that falls
    /// back to "Padawan", the empty string, and a long name that hits the `outSize` cap.
    /// Checked byte-exact against the extracted C for several buffer sizes.
    #[test]
    fn clientcleanname_matches_oracle() {
        let cases: &[&[u8]] = &[
            b"\0",
            b"   \0",
            b"Padawan\0",
            b"  Kyle  \0",
            b"a    b\0",
            b"^1Red^7White^0Black\0",
            b"^0^0^0\0",
            b"trailing^\0",
            b"^\0",
            b"^9color^\0",
            b"AVeryLongNameThatExceedsTheBufferSizeForSureYesIndeed\0",
            b"^1^2^3^4^5^6^7^8\0",
            b" ^1 leading color\0",
        ];
        let sizes: &[c_int] = &[4, 8, 16, 32, 64];

        for (i, name) in cases.iter().enumerate() {
            for &size in sizes {
                let mut mine = vec![0u8; size as usize + 8];
                let mut theirs = vec![0u8; size as usize + 8];
                unsafe {
                    ClientCleanName(
                        name.as_ptr() as *const c_char,
                        mine.as_mut_ptr() as *mut c_char,
                        size,
                    );
                    jka_ClientCleanName(
                        name.as_ptr() as *const c_char,
                        theirs.as_mut_ptr() as *mut c_char,
                        size,
                    );
                }
                assert_eq!(mine, theirs, "case {i} (size {size})");
            }
        }
    }
}
