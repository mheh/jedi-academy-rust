//! Port of `NPC_spawn.c` — the NPC spawning file. The keystone `NPC_Spawn_Do`
//! chain plus the ~50 `SP_NPC_*` classname spawners. Three branches are guarded
//! REVISIT stubs pending their not-yet-ported subsystems (vehicle create, ICARUS parms,
//! and the `NPC_Begin` think-fn assignment which lives in the NPC-AI core); see the
//! `extern "C"` forward-decl blocks and the per-call comments below.

#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)] // C names kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::anims::BOTH_STAND1;
use crate::codemp::game::b_public_h::{
    gNPC_t, BS_CINEMATIC, BS_DEFAULT, BS_WAIT, NPCAI_CUSTOM_GRAVITY, NPCAI_MATCHPLAYERWEAPON,
    RANK_COMMANDER, RANK_LT, SCF_ALT_FIRE, SCF_CHASE_ENEMIES, SCF_DONT_FIRE, SCF_IGNORE_ALERTS,
    SCF_LOOK_FOR_ENEMIES, SCF_NO_FORCE, SCF_NO_GROUPS,
};
use crate::codemp::game::bg_misc::BG_Alloc;
use crate::codemp::game::bg_panimate::BG_ParseAnimationFile;
use crate::codemp::game::bg_public::{
    bgEntity_t, EF2_FLYING, EF_NODRAW, ET_NPC, GT_SIEGE, MASK_NPCSOLID, MASK_SOLID, MOD_UNKNOWN, PERS_SCORE,
    PERS_SPAWN_COUNT, PERS_TEAM, PMF_RESPAWNED, PMF_TIME_KNOCKBACK, SETANIM_BOTH,
    SETANIM_FLAG_NORMAL, STAT_HEALTH, STAT_MAX_HEALTH, STAT_WEAPONS, TEAM_FREE, TEAM_NUM_TEAMS,
    WEAPON_IDLE,
};
use crate::codemp::game::bg_saga_h::{SIEGETEAM_TEAM1, SIEGETEAM_TEAM2};
use crate::codemp::game::bg_vehicleLoad::{g_vehicleInfo, BG_VehicleGetIndex};
use crate::codemp::game::bg_vehicles_h::{
    VEHICLE_NONE, VH_ANIMAL, VH_FIGHTER, VH_SPEEDER, VH_WALKER,
};
use crate::codemp::game::animalnpc::G_CreateAnimalNPC;
use crate::codemp::game::speedernpc::G_CreateSpeederNPC;
use crate::codemp::game::fighternpc::G_CreateFighterNPC;
use crate::codemp::game::walkernpc::G_CreateWalkerNPC;
use crate::codemp::game::bg_weapons::weaponData;
use crate::codemp::game::bg_weapons_h::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_DEMP2, WP_DISRUPTOR, WP_FLECHETTE, WP_NONE,
    WP_NUM_WEAPONS, WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON, WP_THERMAL,
};
use crate::codemp::game::g_local::{
    gclient_t, gentity_t, FL_DMG_BY_HEAVY_WEAP_ONLY, FL_NOTARGET, FL_NO_KNOCKBACK, FL_SHIELDED,
    FL_UNDYING, FRAMETIME,
    START_TIME_REMOVE_ENTS,
};
use crate::codemp::game::g_main::{
    g_allowNPC, g_entities, g_gametype, g_gravity, g_inactivity, g_spskill, level, Com_Printf,
};
use crate::codemp::game::g_public_h::{
    BSET_FIRST, BSET_SPAWN, NUM_BSETS, SVF_NOCLIENT, SVF_PLAYER_USABLE,
};
use crate::codemp::game::npc_utils::G_ActivateBehavior;
use crate::codemp::game::g_spawn::{G_NewString, G_SpawnFloat, G_SpawnInt};
use crate::codemp::game::g_utils::{
    G_CheckInSolid, G_CreateFakeClient, G_Find, G_FreeEntity, G_KillBox, G_ModelIndex,
    G_ScaleNetHealth, G_SetAngles, G_SetOrigin, G_Spawn, G_UseTargets, G_UseTargets2,
};
use crate::codemp::game::g_vehicles::G_VehicleSpawn;
use crate::codemp::game::g_active::ClientThink;
use crate::codemp::game::g_client::SetClientViewAngle;
use crate::codemp::game::g_combat::player_die;
use crate::codemp::game::g_ICARUScb::{G_DebugPrint, WL_DEBUG};
use crate::codemp::game::g_nav::WAYPOINT_NONE;
use crate::codemp::game::npc::{NPCInfo, NPC_SetAnim, NPC_Think, SetNPCGlobals};
use crate::codemp::game::npc_combat::{ChangeWeapon, NPC_ChangeWeapon};
use crate::codemp::game::npc_goal::NPC_ClearGoal;
use crate::codemp::game::npc_reactions::NPC_Use;
use crate::codemp::game::surfaceflags_h::CONTENTS_BODY;
use crate::codemp::game::npc_senses::InFOV;
use crate::codemp::game::npc_stats::{
    NPC_ParseParms, NPC_Precache, NPC_PrecacheAnimationCFG, TeamTable,
};
use crate::codemp::game::q_math::{
    AngleVectors, DistanceSquared, VectorAdd, VectorClear, VectorCopy, VectorMA, VectorNormalize,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared::{
    va, GetIDForString, Q_stricmp, Q_strlwr, Q_strncmp, Q_strrchr, Sz,
};
use crate::codemp::game::q_shared_h::{
    stringID_table_t, trace_t, usercmd_t, vec3_t, ENTITYNUM_MAX_NORMAL, ENTITYNUM_NONE,
    ENTITYNUM_WORLD, FORCE_LEVEL_3, FP_LEVITATION,
    MAX_GENTITIES, MIN_WORLD_COORD, PITCH, ROLL, TR_INTERPOLATE, YAW,
};
use crate::codemp::game::teams_h::{
    class_t, npcteam_t, CLASS_ATST, CLASS_BOBAFETT, CLASS_DESANN, CLASS_GALAKMECH, CLASS_GONK,
    CLASS_HOWLER, CLASS_IMPWORKER, CLASS_INTERROGATOR, CLASS_JEDI, CLASS_KYLE, CLASS_LUKE,
    CLASS_MARK1, CLASS_MARK2,
    CLASS_MINEMONSTER, CLASS_MOUSE, CLASS_PROBE, CLASS_PROTOCOL, CLASS_R2D2, CLASS_R5D2,
    CLASS_RANCOR, CLASS_REBORN, CLASS_REMOTE, CLASS_SEEKER, CLASS_SENTRY, CLASS_SHADOWTROOPER,
    CLASS_STORMTROOPER, CLASS_SWAMPTROOPER, CLASS_TAVION, CLASS_VEHICLE, CLASS_WAMPA,
    NPCTEAM_ENEMY, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER,
};
use crate::codemp::game::w_saber::WP_SaberInitBladeData;
use crate::codemp::game::w_saber_h::JSF_AMBUSH;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// NPC-AI pain handlers — installed by `NPC_PainFunc` as the spawned NPC's `pain`
// fn-ptr. Each lives in its per-class AI file; all are ported.
use crate::codemp::game::npc_ai_droid::{
    NPC_Droid_Pain, NPC_Gonk_Precache, NPC_Mouse_Precache, NPC_Protocol_Precache,
    NPC_R2D2_Precache, NPC_R5D2_Precache,
};
use crate::codemp::game::npc_ai_atst::{NPC_ATST_Pain, NPC_ATST_Precache};
use crate::codemp::game::npc_ai_mark1::{NPC_Mark1_Pain, NPC_Mark1_Precache};
use crate::codemp::game::npc_ai_mark2::{NPC_Mark2_Pain, NPC_Mark2_Precache};
use crate::codemp::game::npc_ai_galakmech::{NPC_GalakMech_Init, NPC_GalakMech_Precache, NPC_GM_Pain};
use crate::codemp::game::npc_ai_interrogator::NPC_Interrogator_Precache;
use crate::codemp::game::npc_ai_howler::{NPC_Howler_Pain, NPC_Howler_Precache};
use crate::codemp::game::npc_ai_imperialprobe::{NPC_Probe_Pain, NPC_Probe_Precache};
use crate::codemp::game::npc_ai_jedi::{
    Boba_Precache, Jedi_ClearTimers, Jedi_Cloak, NPC_Jedi_Pain, NPC_ShadowTrooper_Precache,
};
use crate::codemp::game::npc_ai_minemonster::{NPC_MineMonster_Pain, NPC_MineMonster_Precache};
use crate::codemp::game::npc_ai_rancor::{NPC_Rancor_Pain, Rancor_SetBolts};
use crate::codemp::game::npc_ai_remote::{NPC_Remote_Pain, NPC_Remote_Precache};
use crate::codemp::game::npc_ai_seeker::{NPC_Seeker_Pain, NPC_Seeker_Precache};
use crate::codemp::game::npc_ai_sentry::{NPC_Sentry_Pain, NPC_Sentry_Precache};
use crate::codemp::game::npc_ai_stormtrooper::{NPC_ST_Pain, ST_ClearTimers};
use crate::codemp::game::npc_ai_wampa::{NPC_Wampa_Pain, NPC_Wampa_Precache, Wampa_SetBolts};
use crate::codemp::game::npc_reactions::{NPC_Pain, NPC_Touch};
use crate::codemp::game::w_force::{WP_InitForcePowers, WP_SpawnInitForcePowers};

// b_local.h spawnflag #defines used by NPC_WeaponsForTeam / NPC_SetMiscDefaultData.
/// `b_local.h:139` — `#define SFB_RIFLEMAN 2`: spawnflag selecting a repeater.
const SFB_RIFLEMAN: c_int = 2;
/// `b_local.h:141` — `#define SFB_PHASER 4`: spawnflag selecting a blaster.
const SFB_PHASER: c_int = 4;
//#define SFB_TRICORDER 8 (b_local.h:143) — only used in the commented-out TRICORDER branch.
/// `b_local.h:147` — `#define SFB_CINEMATIC 32`: spawn with no default AI (BS_CINEMATIC).
const SFB_CINEMATIC: c_int = 32;
/// `b_local.h:148` — `#define SFB_NOTSOLID 64`: spawn non-solid.
const SFB_NOTSOLID: c_int = 64;
/// `b_local.h:149` — `#define SFB_STARTINSOLID 128`: okay to start in solid.
const SFB_STARTINSOLID: c_int = 128;
/// `NPC_spawn.c:59` — `#define NSF_DROP_TO_FLOOR 16`.
const NSF_DROP_TO_FLOOR: c_int = 16;
/// `NPC_spawn.c:1818` — `#define SHY_THINK_TIME 1000`.
const SHY_THINK_TIME: c_int = 1000;
/// `NPC_spawn.c:1819-1820` — shy-spawn proximity threshold (squared).
const SHY_SPAWN_DISTANCE: f32 = 128.0;
const SHY_SPAWN_DISTANCE_SQR: f32 = SHY_SPAWN_DISTANCE * SHY_SPAWN_DISTANCE;

/// `S_COLOR_RED` (q_shared.h:1065) — console color escape.
const S_COLOR_RED: &str = "^1";
/// `S_COLOR_GREEN` (q_shared.h:1066) — console color escape.
const S_COLOR_GREEN: &str = "^2";

// g_public.h:39-41 — svFlags suppressing the NPC's auto sound sets. Defined privately
// in npc_stats.rs (not exported); mirrored here for NPC_Spawn_Do / SP_NPC_spawner.
/// `g_public.h:39` — `#define SVF_NO_BASIC_SOUNDS 0x10000000` (No basic sounds).
const SVF_NO_BASIC_SOUNDS: c_int = 0x10000000;
/// `g_public.h:40` — `#define SVF_NO_COMBAT_SOUNDS 0x20000000` (No combat sounds).
const SVF_NO_COMBAT_SOUNDS: c_int = 0x20000000;
/// `g_public.h:41` — `#define SVF_NO_EXTRA_SOUNDS 0x40000000` (No extra or jedi sounds).
const SVF_NO_EXTRA_SOUNDS: c_int = 0x40000000;

/// `char *TeamNames[TEAM_NUM_TEAMS]` (NPC_stats.c:138) — the (mostly commented-out) team
/// name strings, sized to `bg_public.h`'s `TEAM_NUM_TEAMS` (4). Only used here in the
/// `npc kill team` error-prints. Mirrors the C array (NUL-able entry 0 = `""`); added
/// locally since it is not yet in the Rust `npc_stats` port.
static TeamNames: [&str; 4] = [
    "",
    //	"starfleet",
    //	"borg",
    //	"parasite",
    //	"scavengers",
    //	"klingon",
    //	"malon",
    //	"hirogen",
    //	"imperial",
    //	"stasis",
    //	"species8472",
    //	"dreadnought",
    //	"forge",
    //	"disguise",
    //	"player (not valid)"
    "player",
    "enemy",
    "neutral",
];

/// `qboolean showBBoxes = qfalse;` (NPC_spawn.c:4211) — debug toggle for drawing exact NPC
/// bounding boxes; flipped by `npc showbounds` and read by the NPC render code (NPC.c /
/// NPC_behavior.c via `extern`).
pub static mut showBBoxes: qboolean = QFALSE as qboolean;

// The NPC-AI / force-power helpers that NPC_SetMiscDefaultData installs and calls
// (Boba_Precache, Wampa_SetBolts, Rancor_SetBolts, Jedi_ClearTimers, ST_ClearTimers,
// Jedi_Cloak, WP_InitForcePowers, WP_SpawnInitForcePowers) all have real Rust bodies
// now; they are pulled in via `use` from their home modules above rather than through
// phantom `extern "C"` forward-decls (which broke the Windows DLL link — see
// EXTERN_C_AUDIT.md §A). NPC_Begin's think-fn install remains a REVISIT (NPC-AI core).

/// `int WP_SetSaberModel( gclient_t *client, class_t npcClass )` (NPC_spawn.c:92).
/// Body is a stub in the original ("rwwFIXMEFIXME: Do something here, need to let
/// the client know.") that just returns 1. Ported verbatim. No oracle (trivial).
pub unsafe fn WP_SetSaberModel(_client: *mut gclient_t, _npcClass: class_t) -> c_int {
    //rwwFIXMEFIXME: Do something here, need to let the client know.
    1
}

/// `typedef void (PAIN_FUNC) (gentity_t *self, gentity_t *attacker, int damage);`
/// (NPC_spawn.c:103) — the pain-handler fn-ptr type. The per-class AI pain handlers
/// (`NPC_Jedi_Pain`, `NPC_ST_Pain`, …) are plain `unsafe fn`, so this mirrors C's
/// plain (non-`extern "C"`) function-pointer type.
pub type PAIN_FUNC =
    unsafe extern "C" fn(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int);

/// `typedef void (TOUCH_FUNC) (gentity_t *self, gentity_t *other, trace_t *trace);`
/// (NPC_spawn.c:203) — the touch-handler fn-ptr type.
pub type TOUCH_FUNC =
    unsafe extern "C" fn(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);

/*
-------------------------
NPC_PainFunc
-------------------------
*/

/// `PAIN_FUNC *NPC_PainFunc( gentity_t *ent )` (NPC_spawn.c:105). Picks the right
/// pain-handler fn-ptr for the NPC: saber users get `NPC_Jedi_Pain`, otherwise it
/// switches on `NPC_class` (troopers/seekers/droids/etc. each get their specialized
/// handler, everyone else gets `NPC_Pain`). The commented-out legacy `playerTeam`
/// switch is carried verbatim. MARK1/MARK2/ATST/GALAKMECH install their now-ported
/// `NPC_Mark1_Pain`/`NPC_Mark2_Pain`/`NPC_ATST_Pain`/`NPC_GM_Pain` handlers. No oracle
/// (returns a fn-ptr keyed off entity state).
///
/// # Safety
/// `ent` and `ent->client` must be valid.
pub unsafe fn NPC_PainFunc(ent: *mut gentity_t) -> PAIN_FUNC {
    let func: PAIN_FUNC;

    if (*(*ent).client).ps.weapon == WP_SABER {
        func = NPC_Jedi_Pain;
    } else {
        // team no longer indicates race/species, use NPC_class to determine different npc types
        /*
        switch ( ent->client->playerTeam )
        {
        default:
            func = painF_NPC_Pain;
            break;
        }
        */
        match (*(*ent).client).NPC_class {
            // troopers get special pain
            CLASS_STORMTROOPER | CLASS_SWAMPTROOPER => {
                func = NPC_ST_Pain;
            }
            CLASS_SEEKER => {
                func = NPC_Seeker_Pain;
            }
            CLASS_REMOTE => {
                func = NPC_Remote_Pain;
            }
            CLASS_MINEMONSTER => {
                func = NPC_MineMonster_Pain;
            }
            CLASS_HOWLER => {
                func = NPC_Howler_Pain;
            }
            // all other droids, did I miss any?
            CLASS_GONK | CLASS_R2D2 | CLASS_R5D2 | CLASS_MOUSE | CLASS_PROTOCOL
            | CLASS_INTERROGATOR => {
                func = NPC_Droid_Pain;
            }
            CLASS_PROBE => {
                func = NPC_Probe_Pain;
            }
            CLASS_SENTRY => {
                func = NPC_Sentry_Pain;
            }
            CLASS_MARK1 => {
                func = NPC_Mark1_Pain;
            }
            CLASS_MARK2 => {
                func = NPC_Mark2_Pain;
            }
            CLASS_ATST => {
                func = NPC_ATST_Pain;
            }
            CLASS_GALAKMECH => {
                func = NPC_GM_Pain;
            }
            CLASS_RANCOR => {
                func = NPC_Rancor_Pain;
            }
            CLASS_WAMPA => {
                func = NPC_Wampa_Pain;
            }
            // everyone else gets the normal pain func
            _ => {
                func = NPC_Pain;
            }
        }
    }

    func
}

/*
-------------------------
NPC_TouchFunc
-------------------------
*/

/// `TOUCH_FUNC *NPC_TouchFunc( gentity_t *ent )` (NPC_spawn.c:205). Always returns
/// `NPC_Touch`. No oracle (returns a fn-ptr).
///
/// # Safety
/// `ent` must be valid (unused).
pub unsafe fn NPC_TouchFunc(_ent: *mut gentity_t) -> TOUCH_FUNC {
    NPC_Touch
}

/// `void NPC_SetMiscDefaultData( gentity_t *ent )` (NPC_spawn.c:221). Applies a
/// large pile of per-class / per-team defaults onto a freshly spawned NPC:
/// behavior state, Boba/Wampa/Rancor/Yoda/emperor special-cases, saber init,
/// force-power init, team-specific weapon tweaks, and the siege team assignment.
/// The vehicle-walker `NPC_ATST_Pain` install, the disabled `galak_mech` init,
/// and the NPC-AI precache/timer helpers (Boba/Wampa/Rancor/Jedi/ST/Cloak) are
/// REVISIT forward-decls (see the extern block). No oracle (entity-state).
///
/// # Safety
/// `ent`, `ent->client`, and `ent->NPC` must be valid.
pub unsafe fn NPC_SetMiscDefaultData(ent: *mut gentity_t) {
    if (*ent).spawnflags & SFB_CINEMATIC != 0 {
        //if a cinematic guy, default us to wait bState
        (*(*ent).NPC).behaviorState = BS_CINEMATIC;
    }
    if (*(*ent).client).NPC_class == CLASS_BOBAFETT {
        //set some stuff, precache
        Boba_Precache();
        (*(*ent).client).ps.fd.forcePowersKnown |= 1 << FP_LEVITATION;
        (*(*ent).client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] = FORCE_LEVEL_3;
        (*(*ent).client).ps.fd.forcePower = 100;
        (*(*ent).NPC).scriptFlags |= SCF_ALT_FIRE | SCF_NO_GROUPS;
    }
    //if ( !Q_stricmp( "atst_vehicle", ent->NPC_type ) )//FIXME: has to be a better, easier way to tell this, no?
    if (*ent).s.NPC_class == CLASS_VEHICLE && !(*ent).m_pVehicle.is_null() {
        (*ent).s.g2radius = 255; //MAX for this value, was (ent->r.maxs[2]-ent->r.mins[2]), which is 272 or something

        // REVISIT (vehicle subsystem): the VH_WALKER walker-mass/shield/NPC_ATST_Pain
        // branch (`ent->m_pVehicle->m_pVehicleInfo->type == VH_WALKER`) is stubbed
        // out pending vehicle data + NPC_ATST_Pain.

        //turn the damn hatch cover on and LEAVE it on
        trap::G2API_SetSurfaceOnOff(
            (*ent).ghoul2,
            c"head_hatchcover".as_ptr(),
            0, /*TURN_ON*/
        );
    }
    if Q_stricmp(c"wampa".as_ptr(), (*ent).NPC_type) == 0 {
        //FIXME: extern this into NPC.cfg?
        Wampa_SetBolts(ent);
        (*ent).s.g2radius = 80; //???
        (*ent).mass = 300.0; //???
        (*ent).flags |= FL_NO_KNOCKBACK;
        // REVISIT (NPC-AI): ent->pain = NPC_Wampa_Pain;
    }
    if (*(*ent).client).NPC_class == CLASS_RANCOR {
        Rancor_SetBolts(ent);
        (*ent).s.g2radius = 255; //MAX for this value, was (ent->r.maxs[2]-ent->r.mins[2]), which is 272 or something
        (*ent).mass = 1000.0; //???
        (*ent).flags |= FL_NO_KNOCKBACK;
        // REVISIT (NPC-AI): ent->pain = NPC_Rancor_Pain;
        (*ent).health *= 4;
    }
    if Q_stricmp(c"Yoda".as_ptr(), (*ent).NPC_type) == 0 {
        //FIXME: extern this into NPC.cfg?
        (*(*ent).NPC).scriptFlags |= SCF_NO_FORCE; //force powers don't work on him
    }
    if Q_stricmp(c"emperor".as_ptr(), (*ent).NPC_type) == 0 {
        //FIXME: extern this into NPC.cfg?
        (*(*ent).NPC).scriptFlags |= SCF_DONT_FIRE; //so he uses only force powers
    }
    //==================
    //	if ( ent->client->ps.saber[0].type != SABER_NONE )
    if (*(*ent).client).ps.weapon == WP_SABER {
        //rwwFIXMEFIXME: is this going to work?
        //if I'm equipped with a saber, initialize it (them)
        //	ent->client->ps.SaberDeactivate();
        //	ent->client->ps.SetSaberLength( 0 );
        WP_SaberInitBladeData(ent);
        (*(*ent).client).ps.saberHolstered = 2;
        //	G_CreateG2AttachedWeaponModel( ent, ent->client->ps.saber[0].model, ent->handRBolt, 0 );
        //	if ( ent->client->ps.dualSabers )
        //	{
        //		G_CreateG2AttachedWeaponModel( ent, ent->client->ps.saber[1].model, ent->handLBolt, 1 );
        //	}
        Jedi_ClearTimers(ent);
    }
    if (*(*ent).client).ps.fd.forcePowersKnown != 0 {
        WP_InitForcePowers(ent);
        WP_SpawnInitForcePowers(ent); //rww
    }
    if (*(*ent).client).NPC_class == CLASS_SEEKER {
        (*(*ent).NPC).defaultBehavior = BS_DEFAULT;
        (*(*ent).client).ps.gravity = 0;
        (*(*ent).NPC).aiFlags |= NPCAI_CUSTOM_GRAVITY;
        (*(*ent).client).ps.eFlags2 |= EF2_FLYING;
        (*ent).count = 30; // SEEKER shot ammo count
    }
    //***I'm not sure whether I should leave this as a TEAM_ switch, I think NPC_class may be more appropriate - dmv
    match (*(*ent).client).playerTeam {
        NPCTEAM_PLAYER => {
            //ent->flags |= FL_NO_KNOCKBACK;
            if (*(*ent).client).NPC_class == CLASS_JEDI || (*(*ent).client).NPC_class == CLASS_LUKE
            {
                //good jedi
                (*(*ent).client).enemyTeam = NPCTEAM_ENEMY;
                if (*ent).spawnflags & JSF_AMBUSH != 0 {
                    //ambusher
                    (*(*ent).NPC).scriptFlags |= SCF_IGNORE_ALERTS;
                    (*(*ent).client).noclip = 1; //hang
                }
            } else {
                // (G_CreateG2AttachedWeaponModel branch disabled in C)
                match (*(*ent).client).ps.weapon {
                    // WP_BRYAR_PISTOL/DISRUPTOR/BOWCASTER/REPEATER/DEMP2/FLECHETTE/
                    // ROCKET_LAUNCHER/default: nothing
                    _ if (*(*ent).client).ps.weapon == WP_THERMAL
                        || (*(*ent).client).ps.weapon == WP_BLASTER =>
                    {
                        //FIXME: health in NPCs.cfg, and not all blaster users are stormtroopers
                        //ent->health = 25;
                        //FIXME: not necc. a ST
                        ST_ClearTimers(ent);
                        if (*(*ent).NPC).rank >= RANK_LT || (*(*ent).client).ps.weapon == WP_THERMAL
                        {
                            //officers, grenade-throwers use alt-fire
                            //ent->health = 50;
                            //ent->NPC->scriptFlags |= SCF_ALT_FIRE;
                        }
                    }
                    _ => {}
                }
            }
            if (*(*ent).client).NPC_class == CLASS_KYLE
                || (*(*ent).client).NPC_class == CLASS_VEHICLE
                || ((*ent).spawnflags & SFB_CINEMATIC) != 0
            {
                (*(*ent).NPC).defaultBehavior = BS_CINEMATIC;
            } else {
                /*
                ent->NPC->defaultBehavior = BS_FOLLOW_LEADER;
                ent->client->leader = &g_entities[0];
                */
            }
        }

        NPCTEAM_NEUTRAL => {
            if Q_stricmp((*ent).NPC_type, c"gonk".as_ptr()) == 0 {
                // I guess we generically make them player usable
                (*ent).r.svFlags |= SVF_PLAYER_USABLE;
                //rwwFIXMEFIXME: Make use of this (battery charge by skill).
            }
        }

        NPCTEAM_ENEMY => {
            (*(*ent).NPC).defaultBehavior = BS_DEFAULT;
            if (*(*ent).client).NPC_class == CLASS_SHADOWTROOPER {
                //FIXME: a spawnflag?
                Jedi_Cloak(ent);
            }
            if (*(*ent).client).NPC_class == CLASS_TAVION
                || (*(*ent).client).NPC_class == CLASS_REBORN
                || (*(*ent).client).NPC_class == CLASS_DESANN
                || (*(*ent).client).NPC_class == CLASS_SHADOWTROOPER
            {
                (*(*ent).client).enemyTeam = NPCTEAM_PLAYER;
                if (*ent).spawnflags & JSF_AMBUSH != 0 {
                    //ambusher
                    (*(*ent).NPC).scriptFlags |= SCF_IGNORE_ALERTS;
                    (*(*ent).client).noclip = 1; //hang
                }
            } else if (*(*ent).client).NPC_class == CLASS_PROBE
                || (*(*ent).client).NPC_class == CLASS_REMOTE
                || (*(*ent).client).NPC_class == CLASS_INTERROGATOR
                || (*(*ent).client).NPC_class == CLASS_SENTRY
            {
                (*(*ent).NPC).defaultBehavior = BS_DEFAULT;
                (*(*ent).client).ps.gravity = 0;
                (*(*ent).NPC).aiFlags |= NPCAI_CUSTOM_GRAVITY;
                (*(*ent).client).ps.eFlags2 |= EF2_FLYING;
            } else {
                //		G_CreateG2AttachedWeaponModel( weaponData[ent->client->ps.weapon].weaponMdl, ent->handRBolt, 0 );
                match (*(*ent).client).ps.weapon {
                    // These each fall to their own empty `break` in the C switch:
                    // sniper/machine-gunner/shotgunner/etc. just-comment SCF_ALT_FIRE tweaks.
                    WP_BRYAR_PISTOL | WP_DISRUPTOR | WP_BOWCASTER | WP_REPEATER | WP_DEMP2
                    | WP_FLECHETTE | WP_ROCKET_LAUNCHER | WP_THERMAL | WP_STUN_BATON => {
                        // WP_DISRUPTOR: Sniper — //ent->NPC->scriptFlags |= SCF_ALT_FIRE;
                        // WP_REPEATER: machine-gunner.  WP_FLECHETTE: shotgunner
                        //   (stofficeralt SCF_ALT_FIRE disabled).  WP_THERMAL: Gran, bouncy.
                    }
                    // `default:` and `case WP_BLASTER:` share the body.
                    _ => {
                        //FIXME: health in NPCs.cfg, and not all blaster users are stormtroopers
                        //FIXME: not necc. a ST
                        ST_ClearTimers(ent);
                        if (*(*ent).NPC).rank >= RANK_COMMANDER {
                            //commanders use alt-fire (disabled)
                        }
                        if Q_stricmp(c"rodian2".as_ptr(), (*ent).NPC_type) == 0 {
                            //ent->NPC->scriptFlags |= SCF_ALT_FIRE;
                        }
                    }
                }
                if Q_stricmp(c"galak_mech".as_ptr(), (*ent).NPC_type) == 0 {
                    //starts with armor
                    NPC_GalakMech_Init(ent);
                }
            }
        }

        _ => {}
    }

    if (*(*ent).client).NPC_class == CLASS_SEEKER && !(*ent).activator.is_null() {
        //assume my teams are already set correctly
    } else {
        //for siege, want "bad" npc's allied with the "bad" team
        if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*ent).s.NPC_class != CLASS_VEHICLE {
            if (*(*ent).client).enemyTeam == NPCTEAM_PLAYER {
                (*(*ent).client).sess.sessionTeam = SIEGETEAM_TEAM1;
            } else if (*(*ent).client).enemyTeam == NPCTEAM_ENEMY {
                (*(*ent).client).sess.sessionTeam = SIEGETEAM_TEAM2;
            } else {
                (*(*ent).client).sess.sessionTeam = TEAM_FREE;
            }
        }
    }

    if (*(*ent).client).NPC_class == CLASS_ATST || (*(*ent).client).NPC_class == CLASS_MARK1 {
        // chris/steve/kevin requested that the mark1 be shielded also
        (*ent).flags |= FL_SHIELDED | FL_NO_KNOCKBACK;
    }
}

/*
-------------------------
NPC_WeaponsForTeam
-------------------------
*/

/// `int NPC_WeaponsForTeam( team_t team, int spawnflags, const char *NPC_type )`
/// (NPC_spawn.c:516) — a pure `switch(team)` of `Q_stricmp`/`Q_strncmp` string compares
/// returning a `(1 << WP_*)` starting-weapon bitmask for the given NPC type. `team_t`
/// is `npcteam_t` (teams.h:14, `typedef int npcteam_t`). The commented-out legacy
/// `TEAM_*` (Borg/Hirogen/Klingon/Imperial/Scavengers/Malon/Forge/Stasis/Parasite/8472)
/// case branches are carried verbatim as comments. Pure leaf: zero entity/global state.
///
/// Oracle: `npc_spawn_oracle.c::jka_NPC_WeaponsForTeam` (pure; reuses `q_shared_oracle`'s
/// `Q_stricmp`/`Q_strncmp`).
pub unsafe fn NPC_WeaponsForTeam(
    team: npcteam_t,
    spawnflags: c_int,
    NPC_type: *const c_char,
) -> c_int {
    //*** not sure how to handle this, should I pass in class instead of team and go from there? - dmv
    match team {
        // no longer exists
        //	case TEAM_BORG:
        //		break;

        //	case TEAM_HIROGEN:
        //		if( Q_stricmp( "hirogenalpha", NPC_type ) == 0 )
        //			return ( 1 << WP_BLASTER);
        //Falls through

        //	case TEAM_KLINGON:

        //NOTENOTE: Falls through

        //	case TEAM_IMPERIAL:
        NPCTEAM_ENEMY => {
            if Q_stricmp(c"tavion".as_ptr(), NPC_type) == 0
                || Q_strncmp(c"reborn".as_ptr(), NPC_type, 6) == 0
                || Q_stricmp(c"desann".as_ptr(), NPC_type) == 0
                || Q_strncmp(c"shadowtrooper".as_ptr(), NPC_type, 13) == 0
            {
                return 1 << WP_SABER;
                //			return ( 1 << WP_IMPERIAL_BLADE);
            }
            //NOTENOTE: Falls through if not a knife user

            //	case TEAM_SCAVENGERS:
            //	case TEAM_MALON:
            //FIXME: default weapon in npc config?
            if Q_strncmp(c"stofficer".as_ptr(), NPC_type, 9) == 0 {
                return 1 << WP_FLECHETTE;
            }
            if Q_stricmp(c"stcommander".as_ptr(), NPC_type) == 0 {
                return 1 << WP_REPEATER;
            }
            if Q_stricmp(c"swamptrooper".as_ptr(), NPC_type) == 0 {
                return 1 << WP_FLECHETTE;
            }
            if Q_stricmp(c"swamptrooper2".as_ptr(), NPC_type) == 0 {
                return 1 << WP_REPEATER;
            }
            if Q_stricmp(c"rockettrooper".as_ptr(), NPC_type) == 0 {
                return 1 << WP_ROCKET_LAUNCHER;
            }
            if Q_strncmp(c"shadowtrooper".as_ptr(), NPC_type, 13) == 0 {
                return 1 << WP_SABER; //|( 1 << WP_RAPID_CONCUSSION)?
            }
            if Q_stricmp(c"imperial".as_ptr(), NPC_type) == 0 {
                //return ( 1 << WP_BLASTER_PISTOL);
                return 1 << WP_BLASTER;
            }
            if Q_strncmp(c"impworker".as_ptr(), NPC_type, 9) == 0 {
                //return ( 1 << WP_BLASTER_PISTOL);
                return 1 << WP_BLASTER;
            }
            if Q_stricmp(c"stormpilot".as_ptr(), NPC_type) == 0 {
                //return ( 1 << WP_BLASTER_PISTOL);
                return 1 << WP_BLASTER;
            }
            if Q_stricmp(c"galak".as_ptr(), NPC_type) == 0 {
                return 1 << WP_BLASTER;
            }
            if Q_stricmp(c"galak_mech".as_ptr(), NPC_type) == 0 {
                return 1 << WP_REPEATER;
            }
            if Q_strncmp(c"ugnaught".as_ptr(), NPC_type, 8) == 0 {
                return WP_NONE;
            }
            if Q_stricmp(c"granshooter".as_ptr(), NPC_type) == 0 {
                return 1 << WP_BLASTER;
            }
            if Q_stricmp(c"granboxer".as_ptr(), NPC_type) == 0 {
                return 1 << WP_STUN_BATON;
            }
            if Q_strncmp(c"gran".as_ptr(), NPC_type, 4) == 0 {
                return (1 << WP_THERMAL) | (1 << WP_STUN_BATON);
            }
            if Q_stricmp(c"rodian".as_ptr(), NPC_type) == 0 {
                return 1 << WP_DISRUPTOR;
            }
            if Q_stricmp(c"rodian2".as_ptr(), NPC_type) == 0 {
                return 1 << WP_BLASTER;
            }

            if Q_stricmp(c"interrogator".as_ptr(), NPC_type) == 0
                || Q_stricmp(c"sentry".as_ptr(), NPC_type) == 0
                || Q_strncmp(c"protocol".as_ptr(), NPC_type, 8) == 0
            {
                return WP_NONE;
            }

            if Q_strncmp(c"weequay".as_ptr(), NPC_type, 7) == 0 {
                return 1 << WP_BOWCASTER; //|( 1 << WP_STAFF )(FIXME: new weap?)
            }
            if Q_stricmp(c"impofficer".as_ptr(), NPC_type) == 0 {
                return 1 << WP_BLASTER;
            }
            if Q_stricmp(c"impcommander".as_ptr(), NPC_type) == 0 {
                return 1 << WP_BLASTER;
            }
            if Q_stricmp(c"probe".as_ptr(), NPC_type) == 0
                || Q_stricmp(c"seeker".as_ptr(), NPC_type) == 0
            {
                //return ( 1 << WP_BOT_LASER);
                return 0;
            }
            if Q_stricmp(c"remote".as_ptr(), NPC_type) == 0 {
                //return ( 1 << WP_BOT_LASER );
                return 0;
            }
            if Q_stricmp(c"trandoshan".as_ptr(), NPC_type) == 0 {
                return 1 << WP_REPEATER;
            }
            if Q_stricmp(c"atst".as_ptr(), NPC_type) == 0 {
                //return (( 1 << WP_ATST_MAIN)|( 1 << WP_ATST_SIDE));
                return 0;
            }
            if Q_stricmp(c"mark1".as_ptr(), NPC_type) == 0 {
                //return ( 1 << WP_BOT_LASER);
                return 0;
            }
            if Q_stricmp(c"mark2".as_ptr(), NPC_type) == 0 {
                //return ( 1 << WP_BOT_LASER);
                return 0;
            }
            if Q_stricmp(c"minemonster".as_ptr(), NPC_type) == 0 {
                return 1 << WP_STUN_BATON;
            }
            if Q_stricmp(c"howler".as_ptr(), NPC_type) == 0 {
                return 1 << WP_STUN_BATON;
            }
            //Stormtroopers, etc.
            1 << WP_BLASTER
        }

        NPCTEAM_PLAYER => {
            //		if(spawnflags & SFB_TRICORDER)
            //			return ( 1 << WP_TRICORDER);

            if spawnflags & SFB_RIFLEMAN != 0 {
                return 1 << WP_REPEATER;
            }

            if spawnflags & SFB_PHASER != 0 {
                //return ( 1 << WP_BLASTER_PISTOL);
                return 1 << WP_BLASTER;
            }

            if Q_strncmp(c"jedi".as_ptr(), NPC_type, 4) == 0
                || Q_stricmp(c"luke".as_ptr(), NPC_type) == 0
            {
                return 1 << WP_SABER;
            }

            if Q_strncmp(c"prisoner".as_ptr(), NPC_type, 8) == 0 {
                return WP_NONE;
            }
            if Q_strncmp(c"bespincop".as_ptr(), NPC_type, 9) == 0 {
                //return ( 1 << WP_BLASTER_PISTOL);
                return 1 << WP_BLASTER;
            }

            if Q_stricmp(c"MonMothma".as_ptr(), NPC_type) == 0 {
                return WP_NONE;
            }

            //rebel
            1 << WP_BLASTER
        }

        NPCTEAM_NEUTRAL => {
            if Q_stricmp(c"mark1".as_ptr(), NPC_type) == 0 {
                return WP_NONE;
            }
            if Q_stricmp(c"mark2".as_ptr(), NPC_type) == 0 {
                return WP_NONE;
            }
            if Q_strncmp(c"ugnaught".as_ptr(), NPC_type, 8) == 0 {
                return WP_NONE;
            }
            if Q_stricmp(c"bartender".as_ptr(), NPC_type) == 0 {
                return WP_NONE;
            }
            if Q_stricmp(c"morgankatarn".as_ptr(), NPC_type) == 0 {
                return WP_NONE;
            }

            WP_NONE
        }

        // these no longer exist
        //	case TEAM_FORGE:
        //		return ( 1 << WP_STUN_BATON);
        //		break;

        //	case TEAM_STASIS:
        //		return ( 1 << WP_STUN_BATON);
        //		break;

        //	case TEAM_PARASITE:
        //		break;

        //	case TEAM_8472:
        //		break;
        _ => WP_NONE,
    }
}

/*
-------------------------
NPC_SetWeapons
-------------------------
*/

/// `void NPC_SetWeapons( gentity_t *ent )` (NPC_spawn.c:766). Builds the NPC's
/// `STAT_WEAPONS` bitmask from [`NPC_WeaponsForTeam`], gives 100 ammo per owned
/// weapon, and picks `bestWeap` (saber always wins; melee only if nothing better).
/// The `RegisterItem`/precache call is a `rwwFIXMEFIXME` no-op in the original.
/// No oracle (entity-state).
///
/// # Safety
/// `ent`, `ent->client`, and `ent->NPC` must be valid.
pub unsafe fn NPC_SetWeapons(ent: *mut gentity_t) {
    let mut bestWeap: c_int = WP_NONE;
    let weapons = NPC_WeaponsForTeam(
        (*(*ent).client).playerTeam,
        (*ent).spawnflags,
        (*ent).NPC_type,
    );

    (*(*ent).client).ps.stats[STAT_WEAPONS as usize] = 0;
    let mut curWeap = WP_SABER;
    while curWeap < WP_NUM_WEAPONS {
        if weapons & (1 << curWeap) != 0 {
            (*(*ent).client).ps.stats[STAT_WEAPONS as usize] |= 1 << curWeap;
            //			RegisterItem( FindItemForWeapon( (weapon_t)(curWeap) ) );	//precache the weapon
            //rwwFIXMEFIXME: Precache
            let ammoIdx = weaponData[curWeap as usize].ammoIndex as usize;
            (*(*ent).client).ps.ammo[ammoIdx] = 100; //FIXME: max ammo
            (*(*ent).NPC).currentAmmo = 100;

            if bestWeap == WP_SABER {
                // still want to register other weapons -- force saber to be best weap
                curWeap += 1;
                continue;
            }

            if curWeap == WP_STUN_BATON {
                if bestWeap == WP_NONE {
                    // We'll only consider giving Melee since we haven't found anything better yet.
                    bestWeap = curWeap;
                }
            } else if curWeap > bestWeap || bestWeap == WP_STUN_BATON {
                // This will never override saber as best weap.  Also will override WP_STUN_BATON if something better comes later in the list
                bestWeap = curWeap;
            }
        }
        curWeap += 1;
    }

    (*(*ent).client).ps.weapon = bestWeap;
}

/*
-------------------------
NPC_SpawnEffect

  NOTE:  Make sure any effects called here have their models, tga's and sounds precached in
            CG_RegisterNPCEffects in cg_player.cpp
-------------------------
*/

/// `void NPC_SpawnEffect (gentity_t *ent)` (NPC_spawn.c:815). Empty in the original.
/// Ported verbatim. No oracle (no-op).
///
/// # Safety
/// `ent` must be a valid `gentity_t` (unused).
pub unsafe fn NPC_SpawnEffect(_ent: *mut gentity_t) {}

//--------------------------------------------------------------
// NPC_SetFX_SpawnStates
//
// Set up any special parms for spawn effects
//--------------------------------------------------------------

/// `void NPC_SetFX_SpawnStates( gentity_t *ent )` (NPC_spawn.c:824). Unless the NPC
/// has `NPCAI_CUSTOM_GRAVITY`, copies the global `g_gravity` cvar into its
/// `ps.gravity`. No oracle (cvar/entity-state).
///
/// # Safety
/// `ent`, `ent->client`, and `ent->NPC` must be valid.
pub unsafe fn NPC_SetFX_SpawnStates(ent: *mut gentity_t) {
    if (*(*ent).NPC).aiFlags & NPCAI_CUSTOM_GRAVITY == 0 {
        (*(*ent).client).ps.gravity = (*addr_of!(g_gravity)).value as c_int;
    }
}

/*
================
NPC_SpotWouldTelefrag

================
*/

/// `qboolean NPC_SpotWouldTelefrag( gentity_t *npc )` (NPC_spawn.c:838). Returns
/// `qtrue` if any *other* live, NPC-solid client overlaps the box at `npc`'s
/// current origin (ignoring `npc` itself and its owner). Broad-phase
/// `trap_EntitiesInBox` over `g_entities`. No oracle (trap + global entities).
///
/// # Safety
/// `npc` must be valid; `g_entities` must be initialised.
pub unsafe fn NPC_SpotWouldTelefrag(npc: *mut gentity_t) -> qboolean {
    let mut touch: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    VectorAdd(&(*npc).r.currentOrigin, &(*npc).r.mins, &mut mins);
    VectorAdd(&(*npc).r.currentOrigin, &(*npc).r.maxs, &mut maxs);
    let num = trap::EntitiesInBox(&mins, &maxs, &mut touch);

    for &t in touch.iter().take(num as usize) {
        let hit = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(t as isize);
        //if ( hit->client && hit->client->ps.stats[STAT_HEALTH] > 0 ) {
        if (*hit).inuse != QFALSE
            && !(*hit).client.is_null()
            && (*hit).s.number != (*npc).s.number
            && (*hit).r.contents & MASK_NPCSOLID != 0
            && (*hit).s.number != (*npc).r.ownerNum
            && (*hit).r.ownerNum != (*npc).s.number
        {
            return QTRUE as qboolean;
        }
    }

    QFALSE as qboolean
}

/// `gNPC_t *gNPCPtrs[MAX_GENTITIES];` (NPC_spawn.c:1277) — per-entity NPC-state
/// pointer table, lazily allocated by [`New_NPC_t`].
static mut gNPCPtrs: [*mut gNPC_t; MAX_GENTITIES as usize] = [null_mut(); MAX_GENTITIES as usize];

/// `gNPC_t *New_NPC_t(int entNum)` (NPC_spawn.c:1279). Lazily `BG_Alloc`s the
/// `gNPC_t` slot for `entNum` (once), zeroes it, and returns it. No oracle (opaque
/// allocation / static table).
///
/// # Safety
/// `entNum` must be a valid entity index (`0..MAX_GENTITIES`).
pub unsafe fn New_NPC_t(entNum: c_int) -> *mut gNPC_t {
    if gNPCPtrs[entNum as usize].is_null() {
        gNPCPtrs[entNum as usize] =
            BG_Alloc(core::mem::size_of::<gNPC_t>() as c_int) as *mut gNPC_t;
    }

    let ptr = gNPCPtrs[entNum as usize];

    if !ptr.is_null() {
        // clear it...
        //
        core::ptr::write_bytes(ptr, 0, 1);
    }

    ptr
}

/// `void NPC_NPCPtrsClear(void)` (NPC_spawn.c:1301, `#ifdef _XBOX`). Resets the
/// whole [`gNPCPtrs`] table to NULL. The original is Xbox-only; ported faithfully
/// for completeness. No oracle (static table).
///
/// # Safety
/// Call only when no live NPC dereferences its `gNPC_t`.
pub unsafe fn NPC_NPCPtrsClear() {
    for i in 0..MAX_GENTITIES as usize {
        gNPCPtrs[i] = null_mut();
    }
}

/// `void NPC_DefaultScriptFlags( gentity_t *ent )` (NPC_spawn.c:1357). Sets the
/// NPC's default script flags (`SCF_CHASE_ENEMIES|SCF_LOOK_FOR_ENEMIES`); bails if
/// `ent` or `ent->NPC` is NULL. No oracle (entity-state).
///
/// # Safety
/// `ent` may be NULL; if non-NULL, `ent->NPC` is read.
pub unsafe fn NPC_DefaultScriptFlags(ent: *mut gentity_t) {
    if ent.is_null() || (*ent).NPC.is_null() {
        return;
    }
    //Set up default script flags
    (*(*ent).NPC).scriptFlags = SCF_CHASE_ENEMIES | SCF_LOOK_FOR_ENEMIES;
}

/*
-------------------------
NPC_Spawn_Go
-------------------------
*/

/// `gentity_t *NPC_Spawn_Do( gentity_t *ent )` (NPC_spawn.c:1378). The
/// keystone: spawns and fully wires up a new NPC entity from a spawner `ent` —
/// allocates the entity/NPC/fake-client, copies origin/angles/team/targets/parms,
/// parses the NPC config, and installs the `NPC_Begin` think. Returns the new
/// entity (or NULL on any allocation/parse failure). PC always `G_Spawn()`s (the
/// Xbox `vehicle`-selected `G_SpawnVehicle` is gone).
/// The `NPC_Vehicle`-classname create path and the ICARUS `ent->parms` loop are
/// guarded REVISIT stubs (see comments). No oracle (entity-state + traps).
///
/// # Safety
/// `ent` must be a valid spawner `gentity_t`.
pub unsafe fn NPC_Spawn_Do(ent: *mut gentity_t) -> *mut gentity_t {
    let mut newent: *mut gentity_t;
    let mut saveOrg: vec3_t = [0.0; 3];

    // C uses a `finish:` label that restores the spawner origin (if it dropped to
    // floor) then returns `newent`. The `goto finish;` paths (alloc failures)
    // `break 'spawn` to it; the early hard `return NULL` paths (G_Spawn fail,
    // vehicle, parse fail) return directly without touching saveOrg.
    newent = 'spawn: {
        //Test for drop to floor
        if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
            let mut bottom: vec3_t = [0.0; 3];

            VectorCopy(&(*ent).r.currentOrigin, &mut saveOrg);
            VectorCopy(&(*ent).r.currentOrigin, &mut bottom);
            bottom[2] = MIN_WORLD_COORD as f32;
            let tr = trap::Trace(
                &(*ent).r.currentOrigin,
                &(*ent).r.mins,
                &(*ent).r.maxs,
                &bottom,
                (*ent).s.number,
                MASK_NPCSOLID,
            );
            if tr.allsolid == 0 && tr.startsolid == 0 && tr.fraction < 1.0 {
                G_SetOrigin(ent, &tr.endpos);
            }
        }

        //Check the spawner's count
        if (*ent).count != -1 {
            (*ent).count -= 1;

            if (*ent).count <= 0 {
                (*ent).r#use = None; //never again
                                     //FIXME: why not remove me...?  Because of all the string pointers?  Just do G_NewStrings?
            }
        }

        // PC NPC_Spawn_Do always G_Spawn()s (Xbox's `vehicle`-selected G_SpawnVehicle
        // does not exist in the PC tree).
        newent = G_Spawn();

        if newent.is_null() {
            Com_Printf("\x01ERROR: NPC G_Spawn failed\n");
            return null_mut();
        }

        (*newent).fullName = (*ent).fullName;

        (*newent).NPC = New_NPC_t((*newent).s.number);
        if (*newent).NPC.is_null() {
            Com_Printf("\x01ERROR: NPC G_Alloc NPC failed\n");
            break 'spawn newent;
        }

        //newent->client = (gclient_s *)G_Alloc (sizeof(gclient_s));
        G_CreateFakeClient((*newent).s.number, &mut (*newent).client);

        (*(*newent).NPC).tempGoal = G_Spawn();

        if (*(*newent).NPC).tempGoal.is_null() {
            (*newent).NPC = null_mut();
            break 'spawn newent;
        }

        (*(*(*newent).NPC).tempGoal).classname = c"NPC_goal".as_ptr() as *mut c_char;
        (*(*(*newent).NPC).tempGoal).parent = newent;
        (*(*(*newent).NPC).tempGoal).r.svFlags |= SVF_NOCLIENT;

        if (*newent).client.is_null() {
            Com_Printf("\x01ERROR: NPC BG_Alloc client failed\n");
            break 'spawn newent;
        }

        core::ptr::write_bytes((*newent).client, 0, 1);

        //Assign the pointer for bg entity access
        (*newent).playerState = &mut (*(*newent).client).ps;

        //==NPC_Connect( newent, net_name );===================================

        if (*ent).NPC_type.is_null() {
            (*ent).NPC_type = c"random".as_ptr() as *mut c_char;
        } else {
            (*ent).NPC_type = Q_strlwr(G_NewString((*ent).NPC_type));
        }

        if (*ent).r.svFlags & SVF_NO_BASIC_SOUNDS != 0 {
            (*newent).r.svFlags |= SVF_NO_BASIC_SOUNDS;
        }
        if (*ent).r.svFlags & SVF_NO_COMBAT_SOUNDS != 0 {
            (*newent).r.svFlags |= SVF_NO_COMBAT_SOUNDS;
        }
        if (*ent).r.svFlags & SVF_NO_EXTRA_SOUNDS != 0 {
            (*newent).r.svFlags |= SVF_NO_EXTRA_SOUNDS;
        }

        if !(*ent).message.is_null() {
            //has a key
            (*newent).message = (*ent).message; //transfer the key name
            (*newent).flags |= FL_NO_KNOCKBACK; //don't fall off ledges
        }

        // If this is a vehicle we need to see what kind it is so we properlly allocate it.
        if Q_stricmp((*ent).classname, c"NPC_Vehicle".as_ptr()) == 0 {
            // Get the vehicle entry index.
            let iVehIndex = BG_VehicleGetIndex((*ent).NPC_type);

            if iVehIndex == VEHICLE_NONE {
                G_FreeEntity(newent);
                //get rid of the spawner, too, I guess
                G_FreeEntity(ent);
                return null_mut();
            }
            // NOTE: If you change/add any of these, update NPC_Spawn_f for the new vehicle you
            // want to be able to spawn in manually.

            // See what kind of vehicle this is and allocate it properly.
            match g_vehicleInfo[iVehIndex as usize].r#type {
                VH_ANIMAL => {
                    // Create the animal (making sure all it's data is initialized).
                    G_CreateAnimalNPC(&mut (*newent).m_pVehicle, (*ent).NPC_type);
                }
                VH_SPEEDER => {
                    // Create the speeder (making sure all it's data is initialized).
                    G_CreateSpeederNPC(&mut (*newent).m_pVehicle, (*ent).NPC_type);
                }
                VH_FIGHTER => {
                    // Create the fighter (making sure all it's data is initialized).
                    G_CreateFighterNPC(&mut (*newent).m_pVehicle, (*ent).NPC_type);
                }
                VH_WALKER => {
                    // Create the walker (making sure all it's data is initialized).
                    G_CreateWalkerNPC(&mut (*newent).m_pVehicle, (*ent).NPC_type);
                }
                _ => {
                    Com_Printf(&format!("{S_COLOR_RED}ERROR: Couldn't spawn NPC\n"));
                    G_FreeEntity(newent);
                    //get rid of the spawner, too, I guess
                    G_FreeEntity(ent);
                    return null_mut();
                }
            }

            debug_assert!(
                !(*newent).m_pVehicle.is_null()
                    && !(*(*newent).m_pVehicle).m_pVehicleInfo.is_null()
                    && (*(*(*newent).m_pVehicle).m_pVehicleInfo)
                        .Initialize
                        .is_some()
            );

            //set up my happy prediction hack
            (*(*newent).m_pVehicle).m_vOrientation =
                addr_of_mut!((*(*newent).client).ps.vehOrientation[0]);

            // Setup the vehicle.
            (*(*newent).m_pVehicle).m_pParentEntity = newent as *mut bgEntity_t;
            ((*(*(*newent).m_pVehicle).m_pVehicleInfo).Initialize.unwrap())((*newent).m_pVehicle);

            //cache all the assets
            ((*(*(*newent).m_pVehicle).m_pVehicleInfo)
                .RegisterAssets
                .unwrap())((*newent).m_pVehicle);
            //set the class
            (*(*newent).client).NPC_class = CLASS_VEHICLE;
            if g_vehicleInfo[iVehIndex as usize].r#type == VH_FIGHTER {
                //FIXME: EXTERN!!!
                //don't get pushed around, blasters bounce off, only damage from heavy weaps
                (*newent).flags |= FL_NO_KNOCKBACK | FL_SHIELDED | FL_DMG_BY_HEAVY_WEAP_ONLY;
            }
            //WTF?!!! Ships spawning in pointing straight down!
            //set them up to start landed
            *(*(*newent).m_pVehicle).m_vOrientation.add(YAW) = (*ent).s.angles[YAW];
            *(*(*newent).m_pVehicle).m_vOrientation.add(PITCH) = 0.0;
            *(*(*newent).m_pVehicle).m_vOrientation.add(ROLL) = 0.0;
            G_SetAngles(
                newent,
                &*((*(*newent).m_pVehicle).m_vOrientation as *const vec3_t),
            );
            SetClientViewAngle(
                newent,
                &*((*(*newent).m_pVehicle).m_vOrientation as *const vec3_t),
            );

            //newent->m_pVehicle->m_ulFlags |= VEH_GEARSOPEN;
            //why? this would just make it so the initial anim never got played... -rww
            //There was no initial anim, it would just open the gear even though it's already on the ground (fixed now, made an initial anim)

            //For SUSPEND spawnflag, the amount of time to drop like a rock after SUSPEND turns off
            (*newent).fly_sound_debounce_time = (*ent).fly_sound_debounce_time;

            //for no-pilot-death delay
            (*newent).damage = (*ent).damage;

            //no-pilot-death distance
            (*newent).speed = (*ent).speed;

            //for veh transfer all healy stuff
            (*newent).healingclass = (*ent).healingclass;
            (*newent).healingsound = (*ent).healingsound;
            (*newent).healingrate = (*ent).healingrate;
            (*newent).model2 = (*ent).model2; //for droidNPC
        } else {
            (*(*newent).client).ps.weapon = WP_NONE; //init for later check in NPC_Begin
        }

        VectorCopy(&(*ent).s.origin, &mut (*newent).s.origin);
        VectorCopy(&(*ent).s.origin, &mut (*(*newent).client).ps.origin);
        VectorCopy(&(*ent).s.origin, &mut (*newent).r.currentOrigin);
        G_SetOrigin(newent, &(*ent).s.origin); //just to be sure!
                                               //NOTE: on vehicles, anything in the .npc file will STOMP data on the NPC that's set by the vehicle
        if NPC_ParseParms((*ent).NPC_type, newent) == 0 {
            Com_Printf("\x01ERROR: Couldn't spawn NPC\n");
            G_FreeEntity(newent);
            //get rid of the spawner, too, I guess
            G_FreeEntity(ent);
            return null_mut();
        }

        if !(*ent).NPC_type.is_null() {
            if Q_stricmp((*ent).NPC_type, c"kyle".as_ptr()) == 0 {
                //FIXME: "player", not Kyle?  Or check NPC_type against player's NPC_type?
                (*(*newent).NPC).aiFlags |= NPCAI_MATCHPLAYERWEAPON;
            } else if Q_stricmp((*ent).NPC_type, c"test".as_ptr()) == 0 {
                for n in 0..1isize {
                    let ge = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(n);
                    if (*ge).s.eType != ET_NPC && !(*ge).client.is_null() {
                        VectorCopy(&(*ge).s.origin, &mut (*newent).s.origin);
                        (*(*newent).client).playerTeam = (*(*ge).client).playerTeam;
                        (*newent).s.teamowner = (*(*ge).client).playerTeam;
                        break;
                    }
                }
                (*(*newent).NPC).behaviorState = BS_WAIT;
                (*(*newent).NPC).defaultBehavior = BS_WAIT;
                (*newent).classname = c"NPC".as_ptr() as *mut c_char;
                //		newent->svFlags |= SVF_NOPUSH;
            }
        }
        //=====================================================================
        //set the info we want
        if (*newent).health == 0 {
            (*newent).health = (*ent).health;
        }
        (*newent).script_targetname = (*ent).NPC_targetname;
        (*newent).targetname = (*ent).NPC_targetname;
        (*newent).target = (*ent).NPC_target; //death
        (*newent).target2 = (*ent).target2; //knocked out death
        (*newent).target3 = (*ent).target3; //???
        (*newent).target4 = (*ent).target4; //ffire death
        (*newent).wait = (*ent).wait;

        for index in (BSET_FIRST as usize)..(NUM_BSETS as usize) {
            if !(*ent).behaviorSet[index].is_null() {
                (*newent).behaviorSet[index] = (*ent).behaviorSet[index];
            }
        }

        (*newent).classname = c"NPC".as_ptr() as *mut c_char;
        (*newent).NPC_type = (*ent).NPC_type;
        trap::UnlinkEntity(newent);

        VectorCopy(&(*ent).s.angles, &mut (*newent).s.angles);
        VectorCopy(&(*ent).s.angles, &mut (*newent).r.currentAngles);
        VectorCopy(&(*ent).s.angles, &mut (*(*newent).client).ps.viewangles);
        (*(*newent).NPC).desiredYaw = (*ent).s.angles[YAW as usize];

        trap::LinkEntity(newent);
        (*newent).spawnflags = (*ent).spawnflags;

        if !(*ent).paintarget.is_null() {
            //safe to point at owner's string since memory is never freed during game
            (*newent).paintarget = (*ent).paintarget;
        }
        if !(*ent).opentarget.is_null() {
            (*newent).opentarget = (*ent).opentarget;
        }

        //==New stuff=====================================================================
        (*newent).s.eType = ET_NPC; //ET_PLAYER;

        //FIXME: Call CopyParms
        if !(*ent).parms.is_null() {
            // REVISIT (ICARUS): the per-parm `Q3_SetParm( newent->s.number, parmNum,
            // ent->parms->parm[parmNum] )` copy loop is guarded out pending the ICARUS
            // bridge. The `if (ent->parms)` test is preserved as a no-op body.
        }
        //FIXME: copy cameraGroup, store mine in message or other string field

        //set origin
        (*newent).s.pos.trType = TR_INTERPOLATE;
        (*newent).s.pos.trTime = level.time;
        VectorCopy(&(*newent).r.currentOrigin, &mut (*newent).s.pos.trBase);
        VectorClear(&mut (*newent).s.pos.trDelta);
        (*newent).s.pos.trDuration = 0;
        //set angles
        (*newent).s.apos.trType = TR_INTERPOLATE;
        (*newent).s.apos.trTime = level.time;
        //VectorCopy( newent->r.currentOrigin, newent->s.apos.trBase );
        //Why was the origin being used as angles? Typo I'm assuming -rww
        VectorCopy(&(*newent).s.angles, &mut (*newent).s.apos.trBase);

        VectorClear(&mut (*newent).s.apos.trDelta);
        (*newent).s.apos.trDuration = 0;

        (*(*newent).NPC).combatPoint = -1;

        (*newent).flags |= FL_NOTARGET; //So he's ignored until he's fully spawned
        (*newent).s.eFlags |= EF_NODRAW; //So he's ignored until he's fully spawned

        (*newent).think = Some(NPC_Begin);
        (*newent).nextthink = level.time + FRAMETIME;
        NPC_DefaultScriptFlags(newent);

        //copy over team variables, too
        (*newent).s.shouldtarget = (*ent).s.shouldtarget;
        (*newent).s.teamowner = (*ent).s.teamowner;
        (*newent).alliedTeam = (*ent).alliedTeam;
        (*newent).teamnodmg = (*ent).teamnodmg;
        if !(*ent).team.is_null() && *(*ent).team != 0 {
            //specified team directly?
            (*(*newent).client).sess.sessionTeam = atoi_team((*ent).team);
        } else if (*newent).s.teamowner != TEAM_FREE {
            (*(*newent).client).sess.sessionTeam = (*newent).s.teamowner;
        } else if (*newent).alliedTeam != TEAM_FREE {
            (*(*newent).client).sess.sessionTeam = (*newent).alliedTeam;
        } else if (*newent).teamnodmg != TEAM_FREE {
            (*(*newent).client).sess.sessionTeam = (*newent).teamnodmg;
        } else {
            (*(*newent).client).sess.sessionTeam = TEAM_FREE;
        }
        (*(*newent).client).ps.persistant[PERS_TEAM as usize] =
            (*(*newent).client).sess.sessionTeam;

        trap::LinkEntity(newent);

        if (*ent).r#use.is_none() {
            if !(*ent).target.is_null() {
                //use any target we're pointed at
                G_UseTargets(ent, ent);
            }
            if !(*ent).closetarget.is_null() {
                //last guy should fire this target when he dies
                (*newent).target = (*ent).closetarget;
            }
            (*ent).targetname = null_mut();
            //why not remove me...?  Because of all the string pointers?  Just do G_NewStrings?
            G_FreeEntity(ent); //bye!
        }

        newent
    }; // end 'spawn

    // finish:
    if (*ent).spawnflags & NSF_DROP_TO_FLOOR != 0 {
        G_SetOrigin(ent, &saveOrg);
    }

    newent
}

// The native game build resolves `atoi` to libc (the in-module bg_lib copy is
// `Q3_VM`-only); mirror g_spawn.rs's local binding (see its `extern "C"` block).
extern "C" {
    fn atoi(s: *const c_char) -> c_int;
}

/// `atoi`-on-team helper: C does `atoi(ent->team)` to a `team_t`.
unsafe fn atoi_team(s: *const c_char) -> c_int {
    atoi(s)
}

/// Render a possibly-null C string for `G_DebugPrint` `%s` formatting (C `printf` prints
/// `(null)` for a NULL `char*` on glibc; mirror that for the no-oracle debug lines).
unsafe fn cstr_or(p: *const c_char) -> String {
    if p.is_null() {
        String::from("(null)")
    } else {
        CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}

/// `void NPC_Begin (gentity_t *ent)` (NPC_spawn.c:869) — the second half of NPC spawning:
/// runs after the NPC's client/stats are set up and brings the entity fully live. Telefrag-checks
/// the spot (retry/giveup), plays the spawn effect, seeds health (map > NPC.cfg > 100, with the
/// difficulty scaling + per-class aim/yawSpeed tweaks), sets the solid/contents/clip + render
/// state, picks & changes the weapon, snaps the view angle, kills its box & links, installs the
/// `die`/`pain`/`touch`/`use`/`think` callbacks, runs the spawn ICARUS script, then does one
/// `ClientThink` to drop to the floor. Finally, if it's a vehicle with a droid-unit tag, spawns
/// and bolts on the droid. No oracle (entity/client/NPC state + traps + ICARUS).
///
/// # Safety
/// `ent` must be a valid, client-attached spawner `gentity_t` mid-spawn (`ent->NPC` set).
pub unsafe extern "C" fn NPC_Begin(ent: *mut gentity_t) {
    let spawn_origin: vec3_t;
    let mut spawn_angles: vec3_t;
    let client: *mut gclient_t;
    let mut ucmd: usercmd_t = core::mem::zeroed();
    let spawnPoint: *mut gentity_t = null_mut();

    if (*ent).spawnflags & SFB_NOTSOLID == 0 {
        //No NPCs should telefrag
        if NPC_SpotWouldTelefrag(ent) != QFALSE {
            if (*ent).wait < 0.0 {
                //remove yourself
                G_DebugPrint(
                    WL_DEBUG,
                    &format!(
                        "NPC {} could not spawn, firing target3 ({}) and removing self\n",
                        cstr_or(( *ent).targetname),
                        cstr_or((*ent).target3)
                    ),
                );
                //Fire off our target3
                G_UseTargets2(ent, ent, (*ent).target3);

                //Kill us
                (*ent).think = Some(G_FreeEntity);
                (*ent).nextthink = level.time + 100;
            } else {
                G_DebugPrint(
                    WL_DEBUG,
                    &format!(
                        "NPC {} could not spawn, waiting {:.2} secs to try again\n",
                        cstr_or((*ent).targetname),
                        (*ent).wait / 1000.0
                    ),
                );
                (*ent).think = Some(NPC_Begin);
                (*ent).nextthink = level.time + (*ent).wait as c_int; //try again in half a second
            }
            return;
        }
    }
    //Spawn effect
    NPC_SpawnEffect(ent);

    spawn_origin = (*(*ent).client).ps.origin;
    spawn_angles = (*ent).s.angles;
    spawn_angles[YAW] = (*(*ent).NPC).desiredYaw;

    client = (*ent).client;

    // increment the spawncount so the client will detect the respawn
    (*client).ps.persistant[PERS_SPAWN_COUNT as usize] += 1;

    (*client).airOutTime = level.time + 12000;

    (*client).ps.clientNum = (*ent).s.number;
    // clear entity values

    if (*ent).health != 0 {
        // Was health supplied in map
        (*client).ps.stats[STAT_MAX_HEALTH as usize] = (*ent).health;
        (*client).pers.maxHealth = (*ent).health;
    } else if (*(*ent).NPC).stats.health != 0 {
        // Was health supplied in NPC.cfg?
        if (*(*ent).client).NPC_class != CLASS_REBORN
            && (*(*ent).client).NPC_class != CLASS_SHADOWTROOPER
            //&& ent->client->NPC_class != CLASS_TAVION
            //&& ent->client->NPC_class != CLASS_DESANN
            && (*(*ent).client).NPC_class != CLASS_JEDI
        {
            // up everyone except jedi
            // 100% on easy, 125% on medium, 150% on hard
            (*(*ent).NPC).stats.health +=
                (*(*ent).NPC).stats.health / 4 * (*addr_of!(g_spskill)).integer;
        }

        (*client).ps.stats[STAT_MAX_HEALTH as usize] = (*(*ent).NPC).stats.health;
        (*client).pers.maxHealth = (*(*ent).NPC).stats.health;
    } else {
        (*client).ps.stats[STAT_MAX_HEALTH as usize] = 100;
        (*client).pers.maxHealth = 100;
    }

    if Q_stricmp(c"rodian".as_ptr(), (*ent).NPC_type) == 0 {
        //sniper
        //NOTE: this will get overridden by any aim settings in their spawnscripts
        match (*addr_of!(g_spskill)).integer {
            0 => {
                (*(*ent).NPC).stats.aim = 1;
            }
            1 => {
                (*(*ent).NPC).stats.aim = Q_irand(2, 3);
            }
            2 => {
                (*(*ent).NPC).stats.aim = Q_irand(3, 4);
            }
            _ => {}
        }
    } else if (*(*ent).client).NPC_class == CLASS_STORMTROOPER
        || (*(*ent).client).NPC_class == CLASS_SWAMPTROOPER
        || (*(*ent).client).NPC_class == CLASS_IMPWORKER
        || Q_stricmp(c"rodian2".as_ptr(), (*ent).NPC_type) == 0
    {
        //tweak yawspeed for these NPCs based on difficulty
        match (*addr_of!(g_spskill)).integer {
            0 => {
                (*(*ent).NPC).stats.yawSpeed *= 0.75;
                if (*(*ent).client).NPC_class == CLASS_IMPWORKER {
                    (*(*ent).NPC).stats.aim -= Q_irand(3, 6);
                }
            }
            1 => {
                if (*(*ent).client).NPC_class == CLASS_IMPWORKER {
                    (*(*ent).NPC).stats.aim -= Q_irand(2, 4);
                }
            }
            2 => {
                (*(*ent).NPC).stats.yawSpeed *= 1.5;
                if (*(*ent).client).NPC_class == CLASS_IMPWORKER {
                    (*(*ent).NPC).stats.aim -= Q_irand(0, 2);
                }
            }
            _ => {}
        }
    } else if (*(*ent).client).NPC_class == CLASS_REBORN
        || (*(*ent).client).NPC_class == CLASS_SHADOWTROOPER
    {
        match (*addr_of!(g_spskill)).integer {
            1 => {
                (*(*ent).NPC).stats.yawSpeed *= 1.25;
            }
            2 => {
                (*(*ent).NPC).stats.yawSpeed *= 1.5;
            }
            _ => {}
        }
    }

    (*ent).s.groundEntityNum = ENTITYNUM_NONE;
    (*ent).mass = 10.0; // `mass` is f32 in the Rust gentity_t (C: int 10)
    (*ent).takedamage = QTRUE as c_int;
    (*ent).inuse = QTRUE as qboolean;
    (*ent).classname = c"NPC".as_ptr() as *mut c_char;
    //	if ( ent->client->race == RACE_HOLOGRAM ) {...} else  (SP-only, commented in C)
    if (*ent).spawnflags & SFB_NOTSOLID == 0 {
        (*ent).r.contents = CONTENTS_BODY;
        (*ent).clipmask = MASK_NPCSOLID;
    } else {
        (*ent).r.contents = 0;
        (*ent).clipmask = MASK_NPCSOLID & !CONTENTS_BODY;
    }
    //if(!ent->client->moveType) ... rwwFIXMEFIXME: movetype support

    (*ent).die = Some(player_die);
    (*ent).waterlevel = 0;
    (*ent).watertype = 0;
    (*(*ent).client).ps.rocketLockIndex = ENTITYNUM_NONE;
    (*(*ent).client).ps.rocketLockTime = 0.0; // `rocketLockTime` is f32 in the Rust playerState_t

    //visible to player and NPCs
    if (*(*ent).client).NPC_class != CLASS_R2D2
        && (*(*ent).client).NPC_class != CLASS_R5D2
        && (*(*ent).client).NPC_class != CLASS_MOUSE
        && (*(*ent).client).NPC_class != CLASS_GONK
        && (*(*ent).client).NPC_class != CLASS_PROTOCOL
    {
        (*ent).flags &= !FL_NOTARGET;
    }
    (*ent).s.eFlags &= !EF_NODRAW;

    NPC_SetFX_SpawnStates(ent);

    //client->ps.friction = 6;  rwwFIXMEFIXME: per ent friction?

    if (*(*ent).client).ps.weapon == WP_NONE as c_int {
        //not set by the NPCs.cfg
        NPC_SetWeapons(ent);
    }
    //select the weapon
    (*(*ent).NPC).currentAmmo = (*(*ent).client).ps.ammo
        [weaponData[(*(*ent).client).ps.weapon as usize].ammoIndex as usize];
    (*(*ent).client).ps.weaponstate = WEAPON_IDLE;
    ChangeWeapon(ent, (*(*ent).client).ps.weapon);

    VectorCopy(&spawn_origin, &mut (*client).ps.origin);

    // the respawned flag will be cleared after the attack and jump keys come up
    (*client).ps.pm_flags |= PMF_RESPAWNED;

    // clear entity state values
    (*ent).s.eType = ET_NPC as c_int;

    VectorCopy(&spawn_origin, &mut (*ent).s.origin);

    SetClientViewAngle(ent, &spawn_angles);
    (*(*ent).client).renderInfo.lookTarget = ENTITYNUM_NONE;

    if (*ent).spawnflags & 64 == 0 {
        G_KillBox(ent);
        trap::LinkEntity(ent);
    }

    // don't allow full run speed for a bit
    (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
    (*client).ps.pm_time = 100;

    (*client).respawnTime = level.time;
    (*client).inactivityTime = level.time + ((*addr_of!(g_inactivity)).value * 1000.0) as c_int;
    (*client).latched_buttons = 0;
    if (*ent).s.m_iVehicleNum != 0 {
        //I'm an NPC in a vehicle (or a vehicle), I already have owner set
    } else if (*client).NPC_class == CLASS_SEEKER && !(*ent).activator.is_null() {
        //somebody else "owns" me
        (*ent).s.owner = (*(*ent).activator).s.number;
        (*ent).r.ownerNum = (*(*ent).activator).s.number;
    } else {
        (*ent).s.owner = ENTITYNUM_NONE;
    }

    // set default animations
    if (*(*ent).client).NPC_class != CLASS_VEHICLE {
        NPC_SetAnim(ent, SETANIM_BOTH as c_int, BOTH_STAND1, SETANIM_FLAG_NORMAL as c_int);
    }

    if !spawnPoint.is_null() {
        // fire the targets of the spawn point
        G_UseTargets(spawnPoint, ent);
    }

    //ICARUS include
    trap::ICARUS_InitEnt(ent);

    //==NPC initialization
    SetNPCGlobals(ent);

    (*ent).enemy = null_mut();
    (*NPCInfo).timeOfDeath = 0;
    (*NPCInfo).shotTime = 0;
    NPC_ClearGoal();
    NPC_ChangeWeapon((*(*ent).client).ps.weapon);

    //==Final NPC initialization
    (*ent).pain = Some(NPC_PainFunc(ent)); //painF_NPC_Pain;
    (*ent).touch = Some(NPC_TouchFunc(ent)); //touchF_NPC_Touch;

    (*(*ent).client).ps.ping = (*(*ent).NPC).stats.reactions * 50;

    //MCG - Begin: NPC hacks
    //FIXME: Set the team correctly
    if (*ent).s.NPC_class != CLASS_VEHICLE || (*addr_of!(g_gametype)).integer != GT_SIEGE as c_int {
        (*(*ent).client).ps.persistant[PERS_TEAM as usize] = (*(*ent).client).playerTeam;
    }

    (*ent).r#use = Some(NPC_Use);
    (*ent).think = Some(NPC_Think);
    (*ent).nextthink = level.time + FRAMETIME + Q_irand(0, 100);

    NPC_SetMiscDefaultData(ent);
    if (*ent).health <= 0 {
        //ORIGINAL ID: health will count down towards max_health
        (*ent).health = (*(*ent).client).pers.maxHealth;
        (*client).ps.stats[STAT_HEALTH as usize] = (*(*ent).client).pers.maxHealth;
    } else {
        (*client).ps.stats[STAT_HEALTH as usize] = (*ent).health;
    }

    if (*ent).s.shouldtarget != 0 {
        (*ent).maxHealth = (*ent).health;
        G_ScaleNetHealth(ent);
    }

    ChangeWeapon(ent, (*(*ent).client).ps.weapon); //yes, again... sigh

    if (*ent).spawnflags & SFB_STARTINSOLID == 0 {
        //Not okay to start in solid
        G_CheckInSolid(ent, QTRUE as qboolean);
    }
    VectorClear(&mut (*(*ent).NPC).lastClearOrigin);

    //Run a script if you have one assigned to you
    if G_ActivateBehavior(ent, BSET_SPAWN) != QFALSE {
        trap::ICARUS_MaintainTaskManager((*ent).s.number);
    }

    VectorCopy(
        &(*ent).r.currentOrigin,
        &mut (*(*ent).client).renderInfo.eyePoint,
    );

    // run a client frame to drop exactly to the floor,
    // initialize animations and other things
    // C re-`memset`s ucmd here; it is already zeroed at declaration and unread since, so the
    // re-zero is omitted (behavior-identical).
    // C: VectorCopy(client->pers.cmd.angles, ucmd.angles) — usercmd_t angles are int[3],
    // so this is a plain 3-element copy, not the f32 VectorCopy.
    ucmd.angles = (*client).pers.cmd.angles;

    (*(*ent).client).ps.groundEntityNum = ENTITYNUM_NONE;

    if (*(*ent).NPC).aiFlags & NPCAI_MATCHPLAYERWEAPON != 0 {
        //G_MatchPlayerWeapon( ent );
        //rwwFIXMEFIXME: Use this? Probably doesn't really matter for MP.
    }

    ClientThink((*ent).s.number, &mut ucmd);

    trap::LinkEntity(ent);

    if (*(*ent).client).playerTeam == NPCTEAM_ENEMY {
        //valid enemy spawned
        if (*ent).spawnflags & SFB_CINEMATIC == 0 && (*(*ent).NPC).behaviorState != BS_CINEMATIC {
            //not a cinematic enemy
            if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())).client.is_null() {
                //missionStats.enemiesSpawned++  rwwFIXMEFIXME (SP-only)
            }
        }
    }
    (*ent).waypoint = WAYPOINT_NONE;
    (*(*ent).NPC).homeWp = WAYPOINT_NONE;

    if !(*ent).m_pVehicle.is_null() {
        //a vehicle
        //check for droidunit
        if (*(*ent).m_pVehicle).m_iDroidUnitTag != -1 {
            let mut droidNPCType: *mut c_char = null_mut();
            let droidEnt: *mut gentity_t;
            if !(*ent).model2.is_null() && *(*ent).model2 != 0 {
                //specified on the NPC_Vehicle spawner ent
                droidNPCType = (*ent).model2;
            } else if !(*(*(*ent).m_pVehicle).m_pVehicleInfo).droidNPC.is_null()
                && *(*(*(*ent).m_pVehicle).m_pVehicleInfo).droidNPC != 0
            {
                //specified in the vehicle's .veh file
                droidNPCType = (*(*(*ent).m_pVehicle).m_pVehicleInfo).droidNPC;
            }

            if !droidNPCType.is_null() {
                if Q_stricmp(c"random".as_ptr(), droidNPCType) == 0
                    || Q_stricmp(c"default".as_ptr(), droidNPCType) == 0
                {
                    //use default - R2D2 or R5D2
                    if Q_irand(0, 1) != 0 {
                        droidNPCType = c"r2d2".as_ptr() as *mut c_char;
                    } else {
                        droidNPCType = c"r5d2".as_ptr() as *mut c_char;
                    }
                }
                droidEnt = NPC_SpawnType(ent, droidNPCType, null_mut(), QFALSE as qboolean);
                if !droidEnt.is_null() {
                    if !(*droidEnt).client.is_null() {
                        (*droidEnt).s.m_iVehicleNum = (*ent).s.number;
                        (*(*droidEnt).client).ps.m_iVehicleNum = (*ent).s.number;
                        (*droidEnt).s.owner = (*ent).s.number;
                        (*droidEnt).r.ownerNum = (*ent).s.number;
                        (*(*ent).m_pVehicle).m_pDroidUnit = droidEnt as *mut bgEntity_t;
                        //set team
                        (*droidEnt).alliedTeam = (*ent).alliedTeam;
                        (*droidEnt).teamnodmg = (*ent).teamnodmg;
                        (*(*droidEnt).client).sess.sessionTeam = (*(*ent).client).sess.sessionTeam;
                        (*(*droidEnt).client).ps.persistant[PERS_TEAM as usize] =
                            (*(*ent).client).ps.persistant[PERS_TEAM as usize];
                        //position
                        VectorCopy(&(*ent).r.currentOrigin, &mut (*droidEnt).s.origin);
                        VectorCopy(&(*ent).r.currentOrigin, &mut (*(*droidEnt).client).ps.origin);
                        G_SetOrigin(droidEnt, &(*droidEnt).s.origin);
                        trap::LinkEntity(droidEnt);
                        VectorCopy(&(*ent).r.currentAngles, &mut (*droidEnt).s.angles);
                        G_SetAngles(droidEnt, &(*droidEnt).s.angles);
                        if !(*droidEnt).NPC.is_null() {
                            (*(*droidEnt).NPC).desiredYaw = (*droidEnt).s.angles[YAW];
                            (*(*droidEnt).NPC).desiredPitch = (*droidEnt).s.angles[PITCH];
                        }
                        (*droidEnt).flags |= FL_UNDYING;
                    } else {
                        //wtf?
                        G_FreeEntity(droidEnt);
                    }
                }
            }
        }
    }
}

/// `void NPC_Spawn_Go(gentity_t *ent)` (NPC_spawn.c:1773). Thin wrapper:
/// `NPC_Spawn_Do(ent)`. No oracle (entity-state).
///
/// # Safety
/// `ent` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn NPC_Spawn_Go(ent: *mut gentity_t) {
    NPC_Spawn_Do(ent);
}

/*
-------------------------
NPC_ShySpawn
-------------------------
*/

/// `void NPC_ShySpawn( gentity_t *ent )` (NPC_spawn.c:1822). Re-thinks every
/// `SHY_THINK_TIME` ms until player 0 is close enough and either out of FOV or with
/// no clear LOS, then spawns. The `NPC_ClearLOS2` LOS check is unported (NPC-AI), so
/// — REVISIT — we conservatively skip only that inner guard while preserving the
/// proximity + FOV gate. No oracle (entity-state + traps).
///
/// # Safety
/// `ent` must be a valid spawner `gentity_t`; `g_entities` must be initialised.
pub unsafe extern "C" fn NPC_ShySpawn(ent: *mut gentity_t) {
    (*ent).nextthink = level.time + SHY_THINK_TIME;
    (*ent).think = Some(NPC_ShySpawn);

    //rwwFIXMEFIXME: Care about other clients not just 0?
    let player0 = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    if DistanceSquared(&(*player0).r.currentOrigin, &(*ent).r.currentOrigin)
        <= SHY_SPAWN_DISTANCE_SQR
    {
        return;
    }

    if InFOV(ent, player0, 80, 64) != 0 {
        // FIXME: hardcoded fov
        // REVISIT (NPC-AI): the `NPC_ClearLOS2( &g_entities[0], ent->r.currentOrigin )`
        // inner guard is unported; matching the C semantics requires it to return
        // before spawning, so we keep waiting (return) when in-FOV.
        return;
    }

    (*ent).think = None;
    (*ent).nextthink = 0;

    NPC_Spawn_Go(ent);
}

/*
-------------------------
NPC_Spawn
-------------------------
*/

/// `void NPC_Spawn ( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (NPC_spawn.c:1847). The spawner's `use` callback: schedules a delayed
/// (shy or normal) think, or spawns immediately. No oracle (entity-state).
///
/// # Safety
/// `ent` must be a valid spawner `gentity_t`; `other`/`activator` may be anything.
pub unsafe extern "C" fn NPC_Spawn(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    //delay before spawning NPC
    if (*ent).delay != 0 {
        if (*ent).spawnflags & 2048 != 0 {
            // SHY
            (*ent).think = Some(NPC_ShySpawn);
        } else {
            (*ent).think = Some(NPC_Spawn_Go);
        }

        (*ent).nextthink = level.time + (*ent).delay;
    } else if (*ent).spawnflags & 2048 != 0 {
        // SHY
        NPC_ShySpawn(ent);
    } else {
        NPC_Spawn_Do(ent);
    }
}

/// `void NPC_PrecacheType( char *NPC_type )` (NPC_spawn.c:1965). Spawns a throwaway
/// fake spawner, runs [`NPC_Precache`] on it, and frees it. No oracle (entity-state).
///
/// # Safety
/// `NPC_type` must be a valid C string pointer (or NULL).
pub unsafe fn NPC_PrecacheType(NPC_type: *mut c_char) {
    let fakespawner = G_Spawn();
    if !fakespawner.is_null() {
        (*fakespawner).NPC_type = NPC_type;
        NPC_Precache(fakespawner);
        //NOTE: does the spawner have to stay around to send any precached info to the clients...?
        G_FreeEntity(fakespawner);
    }
}

/// `void SP_NPC_spawner( gentity_t *self)` (NPC_spawn.c:1977). The shared NPC
/// spawn-point setup that every `SP_NPC_*` classname spawner funnels into: bails
/// if NPCs are disabled, sets `fullName`/`count`/`wait`/`delay` defaults, precaches
/// the anim cfg + NPC, then either waits for triggering (`use = NPC_Spawn`) or
/// schedules a deferred `NPC_Spawn_Go`. No oracle (entity-state + traps).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_spawner(self_: *mut gentity_t) {
    if (*addr_of!(g_allowNPC)).integer == 0 {
        (*self_).think = Some(G_FreeEntity);
        (*self_).nextthink = level.time;
        return;
    }
    if (*self_).fullName.is_null() || *(*self_).fullName == 0 {
        //FIXME: make an index into an external string table for localization
        (*self_).fullName = c"Humanoid Lifeform".as_ptr() as *mut c_char;
    }

    //register/precache the models needed for this NPC, not anymore
    //self->classname = "NPC_spawner";

    if (*self_).count == 0 {
        (*self_).count = 1;
    }

    {
        //Stop loading of certain extra sounds
        // C uses a function-`static int garbage` scratch slot for the G_SpawnInt
        // out-param; a local is behavior-identical (only the return bool is read).
        let mut garbage: c_int = 0;

        if G_SpawnInt(c"noBasicSounds".as_ptr(), c"0".as_ptr(), &mut garbage) != QFALSE {
            (*self_).r.svFlags |= SVF_NO_BASIC_SOUNDS;
        }
        if G_SpawnInt(c"noCombatSounds".as_ptr(), c"0".as_ptr(), &mut garbage) != QFALSE {
            (*self_).r.svFlags |= SVF_NO_COMBAT_SOUNDS;
        }
        if G_SpawnInt(c"noExtraSounds".as_ptr(), c"0".as_ptr(), &mut garbage) != QFALSE {
            (*self_).r.svFlags |= SVF_NO_EXTRA_SOUNDS;
        }
    }

    if (*self_).wait == 0.0 {
        (*self_).wait = 500.0;
    } else {
        (*self_).wait *= 1000.0; //1 = 1 msec, 1000 = 1 sec
    }

    (*self_).delay *= 1000; //1 = 1 msec, 1000 = 1 sec

    let mut t: c_int = 0;
    G_SpawnInt(c"showhealth".as_ptr(), c"0".as_ptr(), &mut t);
    if t != 0 {
        (*self_).s.shouldtarget = 1;
    }
    //rwwFIXMEFIXME: support for this flag?

    //We have to load the animation.cfg now because spawnscripts are going to want to set anims and we need to know their length and if they're valid
    NPC_PrecacheAnimationCFG((*self_).NPC_type);

    //rww - can't cheat and do this on the client like in SP, so I'm doing this.
    NPC_Precache(self_);

    if !(*self_).targetname.is_null() {
        //Wait for triggering
        (*self_).r#use = Some(NPC_Spawn);
    //	self->svFlags |= SVF_NPC_PRECACHE;//FIXME: precache my weapons somehow?
    } else {
        //NOTE: auto-spawners never check for shy spawning
        //if ( spawning )
        //just gonna always do this I suppose.
        //in entity spawn stage - map starting up
        (*self_).think = Some(NPC_Spawn_Go);
        (*self_).nextthink = level.time + START_TIME_REMOVE_ENTS + 50;
    }

    //FIXME: store cameraGroup somewhere else and apply to spawned NPCs' cameraGroup
    //Or just don't include NPC_spawners in cameraGroupings
}

// ===========================================================================
// SP_NPC_* classname spawners (NPC_spawn.c:2347+). Each is the spawn function the
// engine calls for an `NPC_<X>` map entity: it picks the `NPC_type` config name
// (sometimes by spawnflag or `Q_irand`), optionally calls `WP_SetSaberModel`, then
// funnels into `SP_NPC_spawner`. All no-oracle (entity-state). String-literal
// `self->NPC_type = "X"` assignments become `c"X".as_ptr() as *mut c_char`.
//
// `SP_NPC_Vehicle` + its `use` callback `NPC_VehicleSpawnUse` landed cycle 106 (once
// `G_VehicleSpawn` became `extern "C"` so it installs as a `think`/`use` fn-ptr); see
// `NPC_VehiclePrecache` above. The Galak / Droid_ATST/Mark1/Mark2/Interrogator spawners
// carry their real PC bodies (set NPC_type, run SP_NPC_spawner, precache) now that their
// per-class precache fns are ported.
// ===========================================================================

/// `void SP_NPC_Kyle( gentity_t *self)` (NPC_spawn.c:2347).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Kyle(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Kyle".as_ptr() as *mut c_char;
    WP_SetSaberModel(null_mut(), CLASS_KYLE);
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Lando( gentity_t *self)` (NPC_spawn.c:2363).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Lando(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Lando".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Jan( gentity_t *self)` (NPC_spawn.c:2377).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Jan(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Jan".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Luke( gentity_t *self)` (NPC_spawn.c:2391).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Luke(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Luke".as_ptr() as *mut c_char;
    WP_SetSaberModel(null_mut(), CLASS_LUKE);
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_MonMothma( gentity_t *self)` (NPC_spawn.c:2407).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_MonMothma(self_: *mut gentity_t) {
    (*self_).NPC_type = c"MonMothma".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Tavion( gentity_t *self)` (NPC_spawn.c:2421).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Tavion(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Tavion".as_ptr() as *mut c_char;
    WP_SetSaberModel(null_mut(), CLASS_TAVION);
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Tavion_New( gentity_t *self)` (NPC_spawn.c:2441).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Tavion_New(self_: *mut gentity_t) {
    if (*self_).spawnflags & 1 != 0 {
        (*self_).NPC_type = c"tavion_scepter".as_ptr() as *mut c_char;
    } else if (*self_).spawnflags & 2 != 0 {
        (*self_).NPC_type = c"tavion_sith_sword".as_ptr() as *mut c_char;
    } else {
        (*self_).NPC_type = c"tavion_new".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Alora( gentity_t *self)` (NPC_spawn.c:2469).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Alora(self_: *mut gentity_t) {
    if (*self_).spawnflags & 1 != 0 {
        (*self_).NPC_type = c"alora_dual".as_ptr() as *mut c_char;
    } else {
        (*self_).NPC_type = c"alora".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Reborn_New( gentity_t *self)` (NPC_spawn.c:2496).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Reborn_New(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 4 != 0 {
            //weaker guys
            if (*self_).spawnflags & 1 != 0 {
                (*self_).NPC_type = c"reborn_dual2".as_ptr() as *mut c_char;
            } else if (*self_).spawnflags & 2 != 0 {
                (*self_).NPC_type = c"reborn_staff2".as_ptr() as *mut c_char;
            } else {
                (*self_).NPC_type = c"reborn_new2".as_ptr() as *mut c_char;
            }
        } else if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"reborn_dual".as_ptr() as *mut c_char;
        } else if (*self_).spawnflags & 2 != 0 {
            (*self_).NPC_type = c"reborn_staff".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"reborn_new".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Cultist_Saber( gentity_t *self)` (NPC_spawn.c:2551).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Cultist_Saber(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            if (*self_).spawnflags & 8 != 0 {
                (*self_).NPC_type = c"cultist_saber_med_throw".as_ptr() as *mut c_char;
            } else {
                (*self_).NPC_type = c"cultist_saber_med".as_ptr() as *mut c_char;
            }
        } else if (*self_).spawnflags & 2 != 0 {
            if (*self_).spawnflags & 8 != 0 {
                (*self_).NPC_type = c"cultist_saber_strong_throw".as_ptr() as *mut c_char;
            } else {
                (*self_).NPC_type = c"cultist_saber_strong".as_ptr() as *mut c_char;
            }
        } else if (*self_).spawnflags & 2 != 0 {
            // NOTE: C duplicates the `spawnflags&2` test here (dead branch); kept verbatim.
            if (*self_).spawnflags & 8 != 0 {
                (*self_).NPC_type = c"cultist_saber_all_throw".as_ptr() as *mut c_char;
            } else {
                (*self_).NPC_type = c"cultist_saber_all".as_ptr() as *mut c_char;
            }
        } else if (*self_).spawnflags & 8 != 0 {
            (*self_).NPC_type = c"cultist_saber_throw".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"cultist_saber".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Cultist_Saber_Powers( gentity_t *self)` (NPC_spawn.c:2620).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Cultist_Saber_Powers(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            if (*self_).spawnflags & 8 != 0 {
                (*self_).NPC_type = c"cultist_saber_med_throw2".as_ptr() as *mut c_char;
            } else {
                (*self_).NPC_type = c"cultist_saber_med2".as_ptr() as *mut c_char;
            }
        } else if (*self_).spawnflags & 2 != 0 {
            if (*self_).spawnflags & 8 != 0 {
                (*self_).NPC_type = c"cultist_saber_strong_throw2".as_ptr() as *mut c_char;
            } else {
                (*self_).NPC_type = c"cultist_saber_strong2".as_ptr() as *mut c_char;
            }
        } else if (*self_).spawnflags & 2 != 0 {
            // NOTE: C duplicates the `spawnflags&2` test here (dead branch); kept verbatim.
            if (*self_).spawnflags & 8 != 0 {
                (*self_).NPC_type = c"cultist_saber_all_throw2".as_ptr() as *mut c_char;
            } else {
                (*self_).NPC_type = c"cultist_saber_all2".as_ptr() as *mut c_char;
            }
        } else if (*self_).spawnflags & 8 != 0 {
            (*self_).NPC_type = c"cultist_saber_throw".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"cultist_saber2".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Cultist( gentity_t *self)` (NPC_spawn.c:2688). For the saber-cultist
/// spawnflag, randomizes the style/throw spawnflags and tail-calls
/// [`SP_NPC_Cultist_Saber`] (sibling).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Cultist(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = null_mut();
            (*self_).spawnflags = 0; //fast, no throw
            match Q_irand(0, 2) {
                0 => (*self_).spawnflags |= 1, //medium
                1 => (*self_).spawnflags |= 2, //strong
                2 => (*self_).spawnflags |= 4, //all
                _ => {}
            }
            if Q_irand(0, 1) != 0 {
                //throw
                (*self_).spawnflags |= 8;
            }
            SP_NPC_Cultist_Saber(self_);
            return;
        } else if (*self_).spawnflags & 2 != 0 {
            (*self_).NPC_type = c"cultist_grip".as_ptr() as *mut c_char;
        } else if (*self_).spawnflags & 4 != 0 {
            (*self_).NPC_type = c"cultist_lightning".as_ptr() as *mut c_char;
        } else if (*self_).spawnflags & 8 != 0 {
            (*self_).NPC_type = c"cultist_drain".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"cultist".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Cultist_Commando( gentity_t *self)` (NPC_spawn.c:2746).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Cultist_Commando(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        (*self_).NPC_type = c"cultistcommando".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Cultist_Destroyer( gentity_t *self)` (NPC_spawn.c:2764).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Cultist_Destroyer(self_: *mut gentity_t) {
    (*self_).NPC_type = c"cultist".as_ptr() as *mut c_char; //"cultist_explode";
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Reelo( gentity_t *self)` (NPC_spawn.c:2777).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Reelo(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Reelo".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Galak( gentity_t *self)` (NPC_spawn.c:2784). Spawnflag 1 picks the
/// `Galak_Mech` variant (and precaches it), otherwise plain `Galak`, then hands off
/// to [`SP_NPC_spawner`].
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Galak(self_: *mut gentity_t) {
    if (*self_).spawnflags & 1 != 0 {
        (*self_).NPC_type = c"Galak_Mech".as_ptr() as *mut c_char;
        NPC_GalakMech_Precache();
    } else {
        (*self_).NPC_type = c"Galak".as_ptr() as *mut c_char;
    }

    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Desann( gentity_t *self)` (NPC_spawn.c:2818).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Desann(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Desann".as_ptr() as *mut c_char;
    WP_SetSaberModel(null_mut(), CLASS_DESANN);
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Bartender( gentity_t *self)` (NPC_spawn.c:2834).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Bartender(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Bartender".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_MorganKatarn( gentity_t *self)` (NPC_spawn.c:2848).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_MorganKatarn(self_: *mut gentity_t) {
    (*self_).NPC_type = c"MorganKatarn".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Jedi( gentity_t *self)` (NPC_spawn.c:2869).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Jedi(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"jeditrainer".as_ptr() as *mut c_char;
        } else if Q_irand(0, 1) != 0 {
            (*self_).NPC_type = c"Jedi".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"Jedi2".as_ptr() as *mut c_char;
        }
    }
    WP_SetSaberModel(null_mut(), CLASS_JEDI);
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Prisoner( gentity_t *self)` (NPC_spawn.c:2908).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Prisoner(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if Q_irand(0, 1) != 0 {
            (*self_).NPC_type = c"Prisoner".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"Prisoner2".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Rebel( gentity_t *self)` (NPC_spawn.c:2932).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Rebel(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if Q_irand(0, 1) != 0 {
            (*self_).NPC_type = c"Rebel".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"Rebel2".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Stormtrooper( gentity_t *self)` (NPC_spawn.c:2967).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Stormtrooper(self_: *mut gentity_t) {
    if (*self_).spawnflags & 8 != 0 {
        //rocketer
        (*self_).NPC_type = c"rockettrooper".as_ptr() as *mut c_char;
    } else if (*self_).spawnflags & 4 != 0 {
        //alt-officer
        (*self_).NPC_type = c"stofficeralt".as_ptr() as *mut c_char;
    } else if (*self_).spawnflags & 2 != 0 {
        //commander
        (*self_).NPC_type = c"stcommander".as_ptr() as *mut c_char;
    } else if (*self_).spawnflags & 1 != 0 {
        //officer
        (*self_).NPC_type = c"stofficer".as_ptr() as *mut c_char;
    } else {
        //regular trooper
        if Q_irand(0, 1) != 0 {
            (*self_).NPC_type = c"StormTrooper".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"StormTrooper2".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_StormtrooperOfficer( gentity_t *self)` (NPC_spawn.c:2999). Sets the
/// officer spawnflag and tail-calls [`SP_NPC_Stormtrooper`] (sibling).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_StormtrooperOfficer(self_: *mut gentity_t) {
    (*self_).spawnflags |= 1;
    SP_NPC_Stormtrooper(self_);
}

/// `void SP_NPC_Snowtrooper( gentity_t *self)` (NPC_spawn.c:3013).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Snowtrooper(self_: *mut gentity_t) {
    (*self_).NPC_type = c"snowtrooper".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Tie_Pilot( gentity_t *self)` (NPC_spawn.c:3028).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Tie_Pilot(self_: *mut gentity_t) {
    (*self_).NPC_type = c"stormpilot".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Ugnaught( gentity_t *self)` (NPC_spawn.c:3042).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Ugnaught(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if Q_irand(0, 1) != 0 {
            (*self_).NPC_type = c"Ugnaught".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"Ugnaught2".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Jawa( gentity_t *self)` (NPC_spawn.c:3068).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Jawa(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"jawa_armed".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"jawa".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Gran( gentity_t *self)` (NPC_spawn.c:3096).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Gran(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"granshooter".as_ptr() as *mut c_char;
        } else if (*self_).spawnflags & 2 != 0 {
            (*self_).NPC_type = c"granboxer".as_ptr() as *mut c_char;
        } else if Q_irand(0, 1) != 0 {
            (*self_).NPC_type = c"gran".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"gran2".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Rodian( gentity_t *self)` (NPC_spawn.c:3133).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Rodian(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"rodian2".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"rodian".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Weequay( gentity_t *self)` (NPC_spawn.c:3157).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Weequay(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        match Q_irand(0, 3) {
            0 => (*self_).NPC_type = c"Weequay".as_ptr() as *mut c_char,
            1 => (*self_).NPC_type = c"Weequay2".as_ptr() as *mut c_char,
            2 => (*self_).NPC_type = c"Weequay3".as_ptr() as *mut c_char,
            3 => (*self_).NPC_type = c"Weequay4".as_ptr() as *mut c_char,
            _ => {}
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Trandoshan( gentity_t *self)` (NPC_spawn.c:3188).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Trandoshan(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        (*self_).NPC_type = c"Trandoshan".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Tusken( gentity_t *self)` (NPC_spawn.c:3205).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Tusken(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"tuskensniper".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"tusken".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Noghri( gentity_t *self)` (NPC_spawn.c:3229).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Noghri(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        (*self_).NPC_type = c"noghri".as_ptr() as *mut c_char;
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_SwampTrooper( gentity_t *self)` (NPC_spawn.c:3247).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_SwampTrooper(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"SwampTrooper2".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"SwampTrooper".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Imperial( gentity_t *self)` (NPC_spawn.c:3279). (The goodie/security
/// key precache block is commented out in C — `rwwFIXMEFIXME: Allow goodie keys`.)
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Imperial(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"ImpOfficer".as_ptr() as *mut c_char;
        } else if (*self_).spawnflags & 2 != 0 {
            (*self_).NPC_type = c"ImpCommander".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"Imperial".as_ptr() as *mut c_char;
        }
    }
    //rwwFIXMEFIXME: Allow goodie keys
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_ImpWorker( gentity_t *self)` (NPC_spawn.c:3322).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_ImpWorker(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if Q_irand(0, 2) == 0 {
            (*self_).NPC_type = c"ImpWorker".as_ptr() as *mut c_char;
        } else if Q_irand(0, 1) != 0 {
            (*self_).NPC_type = c"ImpWorker2".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"ImpWorker3".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_BespinCop( gentity_t *self)` (NPC_spawn.c:3350).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_BespinCop(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if Q_irand(0, 1) == 0 {
            (*self_).NPC_type = c"BespinCop".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"BespinCop2".as_ptr() as *mut c_char;
        }
    }
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Reborn( gentity_t *self)` (NPC_spawn.c:3384).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Reborn(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if (*self_).spawnflags & 1 != 0 {
            (*self_).NPC_type = c"rebornforceuser".as_ptr() as *mut c_char;
        } else if (*self_).spawnflags & 2 != 0 {
            (*self_).NPC_type = c"rebornfencer".as_ptr() as *mut c_char;
        } else if (*self_).spawnflags & 4 != 0 {
            (*self_).NPC_type = c"rebornacrobat".as_ptr() as *mut c_char;
        } else if (*self_).spawnflags & 8 != 0 {
            (*self_).NPC_type = c"rebornboss".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"reborn".as_ptr() as *mut c_char;
        }
    }
    WP_SetSaberModel(null_mut(), CLASS_REBORN);
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Murjj( gentity_t *self)` (NPC_spawn.c:3451).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Murjj(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Murjj".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Swamp( gentity_t *self)` (NPC_spawn.c:3465).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Swamp(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Swamp".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Howler( gentity_t *self)` (NPC_spawn.c:3479).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Howler(self_: *mut gentity_t) {
    (*self_).NPC_type = c"howler".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Claw( gentity_t *self)` (NPC_spawn.c:3508).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Claw(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Claw".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Glider( gentity_t *self)` (NPC_spawn.c:3522).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Glider(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Glider".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Flier2( gentity_t *self)` (NPC_spawn.c:3536).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Flier2(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Flier2".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Lizard( gentity_t *self)` (NPC_spawn.c:3550).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Lizard(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Lizard".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Fish( gentity_t *self)` (NPC_spawn.c:3564).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Fish(self_: *mut gentity_t) {
    (*self_).NPC_type = c"Fish".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Monster_Rancor( gentity_t *self)` (NPC_spawn.c:3596).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Rancor(self_: *mut gentity_t) {
    (*self_).NPC_type = c"rancor".as_ptr() as *mut c_char;
    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Droid_Interrogator( gentity_t *self)` (NPC_spawn.c:3614).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Interrogator(self_: *mut gentity_t) {
    (*self_).NPC_type = c"interrogator".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    NPC_Interrogator_Precache(self_);
}

/// `void SP_NPC_Droid_Mark1( gentity_t *self)` (NPC_spawn.c:3654).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Mark1(self_: *mut gentity_t) {
    (*self_).NPC_type = c"mark1".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    NPC_Mark1_Precache();
}

/// `void SP_NPC_Droid_Mark2( gentity_t *self)` (NPC_spawn.c:3676).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Mark2(self_: *mut gentity_t) {
    (*self_).NPC_type = c"mark2".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    NPC_Mark2_Precache();
}

/// `void SP_NPC_Droid_ATST( gentity_t *self)` (NPC_spawn.c:3695).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_ATST(self_: *mut gentity_t) {
    if (*self_).spawnflags & 1 != 0 {
        (*self_).NPC_type = c"atst_vehicle".as_ptr() as *mut c_char;
    } else {
        (*self_).NPC_type = c"atst".as_ptr() as *mut c_char;
    }

    SP_NPC_spawner(self_);

    NPC_ATST_Precache();
}
/// `void SP_NPC_ShadowTrooper( gentity_t *self)` (NPC_spawn.c:3421).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_ShadowTrooper(self_: *mut gentity_t) {
    if (*self_).NPC_type.is_null() {
        if Q_irand(0, 1) == 0 {
            (*self_).NPC_type = c"ShadowTrooper".as_ptr() as *mut c_char;
        } else {
            (*self_).NPC_type = c"ShadowTrooper2".as_ptr() as *mut c_char;
        }
    }

    NPC_ShadowTrooper_Precache();
    WP_SetSaberModel(null_mut(), CLASS_SHADOWTROOPER);

    SP_NPC_spawner(self_);
}

/// `void SP_NPC_MineMonster( gentity_t *self)` (NPC_spawn.c:3493).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_MineMonster(self_: *mut gentity_t) {
    (*self_).NPC_type = c"minemonster".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);
    NPC_MineMonster_Precache();
}

/// `void SP_NPC_Monster_Wampa( gentity_t *self)` (NPC_spawn.c:3580).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Monster_Wampa(self_: *mut gentity_t) {
    (*self_).NPC_type = c"wampa".as_ptr() as *mut c_char;

    NPC_Wampa_Precache();

    SP_NPC_spawner(self_);
}

/// `void SP_NPC_Droid_Probe( gentity_t *self)` (NPC_spawn.c:3635).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Probe(self_: *mut gentity_t) {
    (*self_).NPC_type = c"probe".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    NPC_Probe_Precache();
}

/// `void SP_NPC_Droid_Remote( gentity_t *self)` (NPC_spawn.c:3723).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Remote(self_: *mut gentity_t) {
    (*self_).NPC_type = c"remote".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    NPC_Remote_Precache();
}

/// `void SP_NPC_Droid_Seeker( gentity_t *self)` (NPC_spawn.c:3741).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Seeker(self_: *mut gentity_t) {
    (*self_).NPC_type = c"seeker".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    NPC_Seeker_Precache();
}

/// `void SP_NPC_Droid_Sentry( gentity_t *self)` (NPC_spawn.c:3759).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Sentry(self_: *mut gentity_t) {
    (*self_).NPC_type = c"sentry".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    NPC_Sentry_Precache();
}

/// `void SP_NPC_Droid_Gonk( gentity_t *self)` (NPC_spawn.c:3779).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Gonk(self_: *mut gentity_t) {
    (*self_).NPC_type = c"gonk".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    //precache the Gonk sounds
    NPC_Gonk_Precache();
}

/// `void SP_NPC_Droid_Mouse( gentity_t *self)` (NPC_spawn.c:3800).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Mouse(self_: *mut gentity_t) {
    (*self_).NPC_type = c"mouse".as_ptr() as *mut c_char;

    SP_NPC_spawner(self_);

    //precache the Mouse sounds
    NPC_Mouse_Precache();
}

/// `void SP_NPC_Droid_R2D2( gentity_t *self)` (NPC_spawn.c:3822).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_R2D2(self_: *mut gentity_t) {
    if (*self_).spawnflags & 1 != 0 {
        //imperial skin
        (*self_).NPC_type = c"r2d2_imp".as_ptr() as *mut c_char;
    } else {
        (*self_).NPC_type = c"r2d2".as_ptr() as *mut c_char;
    }

    SP_NPC_spawner(self_);

    NPC_R2D2_Precache();
}

/// `void SP_NPC_Droid_R5D2( gentity_t *self)` (NPC_spawn.c:3850).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_R5D2(self_: *mut gentity_t) {
    if (*self_).spawnflags & 1 != 0 {
        //imperial skin
        (*self_).NPC_type = c"r5d2_imp".as_ptr() as *mut c_char;
    } else {
        (*self_).NPC_type = c"r5d2".as_ptr() as *mut c_char;
    }

    SP_NPC_spawner(self_);

    NPC_R5D2_Precache();
}

/// `void SP_NPC_Droid_Protocol( gentity_t *self)` (NPC_spawn.c:3875).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Droid_Protocol(self_: *mut gentity_t) {
    if (*self_).spawnflags & 1 != 0 {
        //imperial skin
        (*self_).NPC_type = c"protocol_imp".as_ptr() as *mut c_char;
    } else {
        (*self_).NPC_type = c"protocol".as_ptr() as *mut c_char;
    }

    SP_NPC_spawner(self_);
    NPC_Protocol_Precache();
}

/// `qboolean NPC_VehiclePrecache( gentity_t *spawner )` (NPC_spawn.c:2110). Precaches a
/// vehicle's model/skin/animation-config and any attached droid NPC. Looks up the
/// vehicle index ([`BG_VehicleGetIndex`]), precaches the `$NPC_type` model, instances a
/// throwaway ghoul2 model to pull the GLA name (deriving the `animation.cfg` path) and
/// parse the anim config, then precaches the droid NPC (defaulting to r2d2/r5d2). The
/// `ModelMem.SetNPCMode(...)` calls are `_XBOX`-only and omitted (see `NPC_stats`). No
/// oracle (engine asset I/O + traps).
///
/// # Safety
/// `spawner` must be a valid `gentity_t`.
pub unsafe fn NPC_VehiclePrecache(spawner: *mut gentity_t) -> qboolean {
    let mut droidNPCType: *mut c_char = null_mut();
    //This will precache the vehicle
    let iVehIndex = BG_VehicleGetIndex((*spawner).NPC_type);
    if iVehIndex == VEHICLE_NONE {
        //fixme: error msg?
        return QFALSE as qboolean;
    }

    G_ModelIndex(&format!(
        "${}",
        CStr::from_ptr((*spawner).NPC_type).to_string_lossy()
    )); //make sure the thing is frickin precached
        //now cache his model/skin/anim config
    let pVehInfo = addr_of_mut!(g_vehicleInfo[iVehIndex as usize]);
    if !(*pVehInfo).model.is_null() && *(*pVehInfo).model != 0 {
        let mut tempG2: *mut core::ffi::c_void = null_mut();
        let mut skin: c_int = 0;
        if !(*pVehInfo).skin.is_null() && *(*pVehInfo).skin != 0 {
            skin = trap::R_RegisterSkin(&format!(
                "models/players/{}/model_{}.skin",
                CStr::from_ptr((*pVehInfo).model).to_string_lossy(),
                CStr::from_ptr((*pVehInfo).skin).to_string_lossy()
            ));
        }
        // ModelMem.SetNPCMode(true);  // _XBOX-only
        trap::G2API_InitGhoul2Model(
            addr_of_mut!(tempG2),
            va(format_args!(
                "models/players/{}/model.glm",
                CStr::from_ptr((*pVehInfo).model).to_string_lossy()
            )),
            0,
            skin,
            0,
            0,
            0,
        );
        // ModelMem.SetNPCMode(false);  // _XBOX-only
        if !tempG2.is_null() {
            //now, cache the anim config.
            let mut GLAName: [c_char; 1024] = [0; 1024];

            GLAName[0] = 0;
            trap::G2API_GetGLAName(tempG2, 0, GLAName.as_mut_ptr());

            if GLAName[0] != 0 {
                let slash = Q_strrchr(GLAName.as_ptr(), b'/' as c_int);
                if !slash.is_null() {
                    // strcpy(slash, "/animation.cfg");
                    let repl = b"/animation.cfg\0";
                    for (k, &b) in repl.iter().enumerate() {
                        *slash.add(k) = b as c_char;
                    }

                    BG_ParseAnimationFile(GLAName.as_ptr(), null_mut(), QFALSE as qboolean);
                }
            }
            trap::G2API_CleanGhoul2Models(addr_of_mut!(tempG2));
        }
    }

    //also precache the droid NPC if there is one
    if !(*spawner).model2.is_null() && *(*spawner).model2 != 0 {
        droidNPCType = (*spawner).model2;
    } else if !(*pVehInfo).droidNPC.is_null() && *(*pVehInfo).droidNPC != 0 {
        droidNPCType = (*pVehInfo).droidNPC;
    }

    if !droidNPCType.is_null() {
        if Q_stricmp(c"random".as_ptr(), droidNPCType) == 0
            || Q_stricmp(c"default".as_ptr(), droidNPCType) == 0
        {
            //precache both r2 and r5, as defaults
            NPC_PrecacheType(c"r2d2".as_ptr() as *mut c_char);
            NPC_PrecacheType(c"r5d2".as_ptr() as *mut c_char);
        } else {
            NPC_PrecacheType(droidNPCType);
        }
    }
    QTRUE as qboolean
}

/// `void NPC_VehicleSpawnUse( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (NPC_spawn.c:2184) — the `use` callback for a targeted vehicle spawner: with a `delay`,
/// schedule [`G_VehicleSpawn`] as a `think` after `delay` msec; otherwise spawn the vehicle now.
/// No oracle (entity-state + `level.time`).
///
/// # Safety
/// `self`/`other`/`activator` must be valid `gentity_t*`.
pub unsafe extern "C" fn NPC_VehicleSpawnUse(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    if (*self_).delay != 0 {
        (*self_).think = Some(G_VehicleSpawn);
        (*self_).nextthink = level.time + (*self_).delay;
    } else {
        G_VehicleSpawn(self_);
    }
}

/// `void SP_NPC_Vehicle( gentity_t *self)` (NPC_spawn.c:2199) — the `NPC_Vehicle` map-entity
/// spawner: applies `NPC_type`/`classname`/`wait`/`delay` defaults (wait & delay are scaled
/// sec→msec), sets origin/angles, reads the `dropTime`/`showhealth` spawn keys, then either
/// arms a targeted `use`-trigger ([`NPC_VehicleSpawnUse`]) or, untargeted, spawns immediately
/// (or after `delay` via the [`G_VehicleSpawn`] `think`). Precache failure frees the entity.
/// No oracle (entity-state + traps).
///
/// # Safety
/// `self_` must be a valid spawner `gentity_t`.
pub unsafe extern "C" fn SP_NPC_Vehicle(self_: *mut gentity_t) {
    let mut dropTime: f32 = 0.0;
    let mut t: c_int = 0;

    if (*self_).NPC_type.is_null() {
        (*self_).NPC_type = c"swoop".as_ptr() as *mut c_char;
    }

    if (*self_).classname.is_null() {
        (*self_).classname = c"NPC_Vehicle".as_ptr() as *mut c_char;
    }

    if (*self_).wait == 0.0 {
        (*self_).wait = 500.0;
    } else {
        (*self_).wait *= 1000.0; //1 = 1 msec, 1000 = 1 sec
    }
    (*self_).delay *= 1000; //1 = 1 msec, 1000 = 1 sec

    G_SetOrigin(self_, &(*self_).s.origin);
    G_SetAngles(self_, &(*self_).s.angles);
    G_SpawnFloat(c"dropTime".as_ptr(), c"0".as_ptr(), &mut dropTime);
    if dropTime != 0.0 {
        (*self_).fly_sound_debounce_time = ((dropTime as f64) * 1000.0).ceil() as c_int;
    }

    G_SpawnInt(c"showhealth".as_ptr(), c"0".as_ptr(), &mut t);
    if t != 0 {
        (*self_).s.shouldtarget = 1;
    }
    //FIXME: PRECACHE!!!

    if !(*self_).targetname.is_null() {
        if NPC_VehiclePrecache(self_) == QFALSE as qboolean {
            //FIXME: err msg?
            G_FreeEntity(self_);
            return;
        }
        (*self_).r#use = Some(NPC_VehicleSpawnUse);
    } else if (*self_).delay != 0 {
        if NPC_VehiclePrecache(self_) == QFALSE as qboolean {
            //FIXME: err msg?
            G_FreeEntity(self_);
            return;
        }
        (*self_).think = Some(G_VehicleSpawn);
        (*self_).nextthink = level.time + (*self_).delay;
    } else {
        G_VehicleSpawn(self_);
    }
}

// NPC console commands
/*
NPC_Spawn_f
*/

/// `gentity_t *NPC_SpawnType( gentity_t *ent, char *npc_type, char *targetname, qboolean isVehicle )`
/// (NPC_spawn.c:3896). Console-spawns an NPC: allocates a throwaway `NPC_spawner` at a
/// point in front of the issuing player (trace forward + drop), copies the type/targetname,
/// runs the per-type precache, then hands off to [`NPC_Spawn_Do`]. The ATST/MARK1/MARK2/
/// interrogator/galak_mech precache branches are `assert(0)`-only in the original (their
/// precaches commented out). No oracle (entity-state + traps).
///
/// # Safety
/// `ent` may be NULL; if non-NULL, `ent->client` is read. `npc_type` may be NULL.
pub unsafe fn NPC_SpawnType(
    ent: *mut gentity_t,
    npc_type: *mut c_char,
    targetname: *mut c_char,
    isVehicle: qboolean,
) -> *mut gentity_t {
    let NPCspawner = G_Spawn();
    let mut forward: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    if NPCspawner.is_null() {
        Com_Printf("^1NPC_Spawn Error: Out of entities!\n");
        return null_mut();
    }

    (*NPCspawner).think = Some(G_FreeEntity);
    (*NPCspawner).nextthink = level.time + FRAMETIME;

    if npc_type.is_null() {
        return null_mut();
    }

    if *npc_type == 0 {
        Com_Printf("^1Error, expected one of:^7 NPC spawn [NPC type (from ext_data/NPCs)]\n NPC spawn vehicle [VEH type (from ext_data/vehicles)]\n");
        return null_mut();
    }

    if ent.is_null() || (*ent).client.is_null() {
        //screw you, go away
        return null_mut();
    }

    //rwwFIXMEFIXME: Care about who is issuing this command/other clients besides 0?
    //Spawn it at spot of first player
    //FIXME: will gib them!
    AngleVectors(
        &(*(*ent).client).ps.viewangles,
        Some(&mut forward),
        None,
        None,
    );
    VectorNormalize(&mut forward);
    VectorMA(&(*ent).r.currentOrigin, 64.0, &forward, &mut end);
    let mut trace = trap::Trace(
        &(*ent).r.currentOrigin,
        &[0.0; 3],
        &[0.0; 3],
        &end,
        0,
        MASK_SOLID,
    );
    VectorCopy(&trace.endpos, &mut end);
    end[2] -= 24.0;
    trace = trap::Trace(&trace.endpos, &[0.0; 3], &[0.0; 3], &end, 0, MASK_SOLID);
    VectorCopy(&trace.endpos, &mut end);
    end[2] += 24.0;
    G_SetOrigin(NPCspawner, &end);
    VectorCopy(&(*NPCspawner).r.currentOrigin, &mut (*NPCspawner).s.origin);
    //set the yaw so that they face away from player
    (*NPCspawner).s.angles[1] = (*(*ent).client).ps.viewangles[1];

    trap::LinkEntity(NPCspawner);

    (*NPCspawner).NPC_type = G_NewString(npc_type);

    if !targetname.is_null() {
        (*NPCspawner).NPC_targetname = G_NewString(targetname);
    }

    (*NPCspawner).count = 1;

    (*NPCspawner).delay = 0;

    //NPCspawner->spawnflags |= SFB_NOTSOLID;

    //NPCspawner->playerTeam = TEAM_FREE;
    //NPCspawner->behaviorSet[BSET_SPAWN] = "common/guard";

    if isVehicle != QFALSE as qboolean {
        (*NPCspawner).classname = c"NPC_Vehicle".as_ptr() as *mut c_char;
    }

    //call precache funcs for James' builds
    if Q_stricmp(c"gonk".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Gonk_Precache();
    } else if Q_stricmp(c"mouse".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Mouse_Precache();
    } else if Q_strncmp(c"r2d2".as_ptr(), (*NPCspawner).NPC_type, 4) == 0 {
        NPC_R2D2_Precache();
    } else if Q_stricmp(c"atst".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_ATST_Precache();
    } else if Q_strncmp(c"r5d2".as_ptr(), (*NPCspawner).NPC_type, 4) == 0 {
        NPC_R5D2_Precache();
    } else if Q_stricmp(c"mark1".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Mark1_Precache();
    } else if Q_stricmp(c"mark2".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Mark2_Precache();
    } else if Q_stricmp(c"interrogator".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Interrogator_Precache(null_mut());
    } else if Q_stricmp(c"probe".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Probe_Precache();
    } else if Q_stricmp(c"seeker".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Seeker_Precache();
    } else if Q_stricmp(c"remote".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Remote_Precache();
    } else if Q_strncmp(c"shadowtrooper".as_ptr(), (*NPCspawner).NPC_type, 13) == 0 {
        NPC_ShadowTrooper_Precache();
    } else if Q_stricmp(c"minemonster".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_MineMonster_Precache();
    } else if Q_stricmp(c"howler".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Howler_Precache();
    } else if Q_stricmp(c"sentry".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Sentry_Precache();
    } else if Q_stricmp(c"protocol".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Protocol_Precache();
    } else if Q_stricmp(c"galak_mech".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_GalakMech_Precache();
    } else if Q_stricmp(c"wampa".as_ptr(), (*NPCspawner).NPC_type) == 0 {
        NPC_Wampa_Precache();
    }

    NPC_Spawn_Do(NPCspawner)
}

/// `void NPC_Spawn_f( gentity_t *ent )` (NPC_spawn.c:4049). The `npc spawn` console
/// subcommand: reads the NPC type (and optional `vehicle` prefix / targetname) from the
/// command args and hands off to [`NPC_SpawnType`]. The C reads each `trap_Argv` into a
/// fixed `char[1024]`; [`trap::Argv`] returns an owned `String`, copied byte-for-byte into
/// the `[c_char; 1024]` token. No oracle (drives `trap_Argv` + entity-state).
///
/// # Safety
/// `ent` must be a valid `gentity_t`.
pub unsafe fn NPC_Spawn_f(ent: *mut gentity_t) {
    let mut npc_type: [c_char; 1024] = [0; 1024];
    let mut targetname: [c_char; 1024] = [0; 1024];
    let mut isVehicle = QFALSE as qboolean;

    // trap_Argv(2, npc_type, 1024);
    copy_argv(2, &mut npc_type);
    if Q_stricmp(c"vehicle".as_ptr(), npc_type.as_ptr()) == 0 {
        isVehicle = QTRUE as qboolean;
        copy_argv(3, &mut npc_type);
        copy_argv(4, &mut targetname);
    } else {
        copy_argv(3, &mut targetname);
    }

    NPC_SpawnType(
        ent,
        npc_type.as_mut_ptr(),
        targetname.as_mut_ptr(),
        isVehicle,
    );
}

/// `trap_Argv(n, buf, sizeof(buf))` idiom — copies command token `n` byte-for-byte into a
/// fixed C-string buffer (truncating, NUL-terminating). Mirrors the C buffer fill since
/// [`trap::Argv`] returns an owned `String` (see `g_cmds`/`g_svcmds`).
fn copy_argv(n: i32, buf: &mut [c_char; 1024]) {
    let arg = trap::Argv(n);
    let bytes = arg.as_bytes();
    let count = bytes.len().min(buf.len() - 1);
    for k in 0..count {
        buf[k] = bytes[k] as c_char;
    }
    buf[count] = 0;
}

/// `void NPC_Kill_f( void )` (NPC_spawn.c:4074). The `npc kill ...` console subcommand:
/// kills/removes NPCs by targetname, `all`, or `team [teamname]` (`nonally` = all but your
/// teammates). No oracle (console cmd: `trap_Argv`, `Com_Printf`, `g_entities` iteration,
/// `die` fn-ptr / `G_FreeEntity`).
///
/// # Safety
/// Reads/mutates `g_entities`; calls each victim's `die` fn-ptr. Caller must hold the game
/// frame invariants.
pub unsafe fn NPC_Kill_f() {
    // C: `int n;` — a shared loop counter; each Rust `for n in ...` rebinds it locally.
    let mut name: [c_char; 1024] = [0; 1024];
    let mut kill_team: c_int = TEAM_FREE; // team_t killTeam = TEAM_FREE;
    let mut kill_non_sf = QFALSE as qboolean;

    // trap_Argv(2, name, 1024);
    copy_argv(2, &mut name);

    if name[0] == 0 {
        Com_Printf(&format!("{S_COLOR_RED}Error, Expected:\n"));
        Com_Printf(&format!(
            "{S_COLOR_RED}NPC kill '[NPC targetname]' - kills NPCs with certain targetname\n"
        ));
        Com_Printf(&format!("{S_COLOR_RED}or\n"));
        Com_Printf(&format!("{S_COLOR_RED}NPC kill 'all' - kills all NPCs\n"));
        Com_Printf(&format!("{S_COLOR_RED}or\n"));
        Com_Printf(&format!(
            "{S_COLOR_RED}NPC team '[teamname]' - kills all NPCs of a certain team ('nonally' is all but your allies)\n"
        ));
        return;
    }

    if Q_stricmp(c"team".as_ptr(), name.as_ptr()) == 0 {
        // trap_Argv(3, name, 1024);
        copy_argv(3, &mut name);

        if name[0] == 0 {
            Com_Printf(&format!(
                "{S_COLOR_RED}NPC_Kill Error: 'npc kill team' requires a team name!\n"
            ));
            Com_Printf(&format!("{S_COLOR_RED}Valid team names are:\n"));
            for n in (TEAM_FREE + 1)..(TEAM_NUM_TEAMS) {
                Com_Printf(&format!("{S_COLOR_RED}{}\n", TeamNames[n as usize]));
            }
            Com_Printf(&format!(
                "{S_COLOR_RED}nonally - kills all but your teammates\n"
            ));
            return;
        }

        if Q_stricmp(c"nonally".as_ptr(), name.as_ptr()) == 0 {
            kill_non_sf = QTRUE as qboolean;
        } else {
            kill_team = GetIDForString(addr_of!(TeamTable) as *const stringID_table_t, name.as_ptr());

            if kill_team == TEAM_FREE {
                Com_Printf(&format!(
                    "{S_COLOR_RED}NPC_Kill Error: team '{}' not recognized\n",
                    Sz(name.as_ptr())
                ));
                Com_Printf(&format!("{S_COLOR_RED}Valid team names are:\n"));
                for n in (TEAM_FREE + 1)..(TEAM_NUM_TEAMS) {
                    Com_Printf(&format!("{S_COLOR_RED}{}\n", TeamNames[n as usize]));
                }
                Com_Printf(&format!(
                    "{S_COLOR_RED}nonally - kills all but your teammates\n"
                ));
                return;
            }
        }
    }

    for n in 1..ENTITYNUM_MAX_NORMAL {
        let player = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(n as isize);
        if (*player).inuse == QFALSE as qboolean {
            continue;
        }
        if kill_non_sf != QFALSE as qboolean {
            // if ( player ) — always non-NULL here (&g_entities[n]); mirror anyway.
            if !player.is_null() {
                if !(*player).client.is_null() {
                    if (*(*player).client).playerTeam != NPCTEAM_PLAYER {
                        Com_Printf(&format!(
                            "{S_COLOR_GREEN}Killing NPC {} named {}\n",
                            Sz((*player).NPC_type),
                            Sz((*player).targetname)
                        ));
                        (*player).health = 0;

                        if (*player).die.is_some() && !(*player).client.is_null() {
                            if let Some(die) = (*player).die {
                                die(
                                    player,
                                    player,
                                    player,
                                    (*(*player).client).pers.maxHealth,
                                    MOD_UNKNOWN,
                                );
                            }
                        }
                    }
                } else if !(*player).NPC_type.is_null()
                    && !(*player).classname.is_null()
                    && *(*player).classname != 0
                    && Q_stricmp(c"NPC_starfleet".as_ptr(), (*player).classname) != 0
                {
                    //A spawner, remove it
                    Com_Printf(&format!(
                        "{S_COLOR_GREEN}Removing NPC spawner {} with NPC named {}\n",
                        Sz((*player).NPC_type),
                        Sz((*player).NPC_targetname)
                    ));
                    G_FreeEntity(player);
                    //FIXME: G_UseTargets2(player, player, player->NPC_target & player->target);?
                }
            }
        } else if !player.is_null() && !(*player).NPC.is_null() && !(*player).client.is_null() {
            if kill_team != TEAM_FREE {
                if (*(*player).client).playerTeam == kill_team {
                    Com_Printf(&format!(
                        "{S_COLOR_GREEN}Killing NPC {} named {}\n",
                        Sz((*player).NPC_type),
                        Sz((*player).targetname)
                    ));
                    (*player).health = 0;
                    if let Some(die) = (*player).die {
                        die(
                            player,
                            player,
                            player,
                            (*(*player).client).pers.maxHealth,
                            MOD_UNKNOWN,
                        );
                    }
                }
            } else if (!(*player).targetname.is_null()
                && Q_stricmp(name.as_ptr(), (*player).targetname) == 0)
                || Q_stricmp(name.as_ptr(), c"all".as_ptr()) == 0
            {
                Com_Printf(&format!(
                    "{S_COLOR_GREEN}Killing NPC {} named {}\n",
                    Sz((*player).NPC_type),
                    Sz((*player).targetname)
                ));
                (*player).health = 0;
                (*(*player).client).ps.stats[STAT_HEALTH as usize] = 0;
                if let Some(die) = (*player).die {
                    die(player, player, player, 100, MOD_UNKNOWN);
                }
            }
        }
        /*
        else if ( player && (player->svFlags&SVF_NPC_PRECACHE) )
        {//a spawner
            Com_Printf( S_COLOR_GREEN"Removing NPC spawner %s named %s\n", player->NPC_type, player->targetname );
            G_FreeEntity( player );
        }
        */
        //rwwFIXMEFIXME: should really do something here.
    }
}

/// `void NPC_PrintScore( gentity_t *ent )` (NPC_spawn.c:4201). Prints the NPC's
/// targetname and `PERS_SCORE`. No oracle (Com_Printf + entity-state).
///
/// # Safety
/// `ent` and `ent->client` must be valid; `ent->targetname` may be NULL.
pub unsafe fn NPC_PrintScore(ent: *mut gentity_t) {
    Com_Printf(&format!(
        "{}: {}\n",
        Sz((*ent).targetname),
        (*(*ent).client).ps.persistant[PERS_SCORE as usize]
    ));
}

/*
Svcmd_NPC_f

parse and dispatch bot commands
*/
/// `void Cmd_NPC_f( gentity_t *ent )` (NPC_spawn.c:4212). Parses and dispatches the `npc`
/// console command: `spawn` / `kill` / `showbounds` (toggles [`showBBoxes`]) / `score`.
/// No oracle (console cmd: `trap_Argv`, `Com_Printf`, dispatch into entity-state subcommands).
///
/// # Safety
/// `ent` must be a valid `gentity_t`; `g_entities` must be initialised.
pub unsafe fn Cmd_NPC_f(ent: *mut gentity_t) {
    let mut cmd: [c_char; 1024] = [0; 1024];

    // trap_Argv( 1, cmd, 1024 );
    copy_argv(1, &mut cmd);

    if cmd[0] == 0 {
        Com_Printf("Valid NPC commands are:\n");
        Com_Printf(" spawn [NPC type (from NCPCs.cfg)]\n");
        Com_Printf(" kill [NPC targetname] or [all(kills all NPCs)] or 'team [teamname]'\n");
        Com_Printf(" showbounds (draws exact bounding boxes of NPCs)\n");
        Com_Printf(" score [NPC targetname] (prints number of kills per NPC)\n");
    } else if Q_stricmp(cmd.as_ptr(), c"spawn".as_ptr()) == 0 {
        NPC_Spawn_f(ent);
    } else if Q_stricmp(cmd.as_ptr(), c"kill".as_ptr()) == 0 {
        NPC_Kill_f();
    } else if Q_stricmp(cmd.as_ptr(), c"showbounds".as_ptr()) == 0 {
        //Toggle on and off
        showBBoxes = if showBBoxes != QFALSE as qboolean {
            QFALSE as qboolean
        } else {
            QTRUE as qboolean
        };
    } else if Q_stricmp(cmd.as_ptr(), c"score".as_ptr()) == 0 {
        let mut cmd2: [c_char; 1024] = [0; 1024];
        // C: `gentity_t *ent = NULL;` — both branches assign before any read, so the initial
        // NULL is dead; declare without an initializer (each branch's first store inits it).
        #[allow(unused_assignments)]
        let mut ent: *mut gentity_t = null_mut();

        // trap_Argv(2, cmd2, 1024);
        copy_argv(2, &mut cmd2);

        if cmd2[0] == 0 {
            //Show the score for all NPCs
            Com_Printf("SCORE LIST:\n");
            for i in 0..ENTITYNUM_WORLD {
                ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize);
                if ent.is_null() || (*ent).client.is_null() {
                    continue;
                }
                NPC_PrintScore(ent);
            }
        } else {
            ent = G_Find(
                null_mut(),
                core::mem::offset_of!(gentity_t, targetname),
                cmd2.as_ptr(),
            );
            if !ent.is_null() && !(*ent).client.is_null() {
                NPC_PrintScore(ent);
            } else {
                Com_Printf(&format!(
                    "ERROR: NPC score - no such NPC {}\n",
                    Sz(cmd2.as_ptr())
                ));
            }
        }
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::NPC_WeaponsForTeam;
    use crate::codemp::game::teams_h::{NPCTEAM_ENEMY, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER};
    use crate::oracle::jka_NPC_WeaponsForTeam;

    /// Parity: a representative spread of `NPC_type` strings across the three live teams
    /// (plus spawnflag-driven PLAYER cases and an out-of-range team) all return the same
    /// weapon bitmask as the authentic C.
    #[test]
    fn npc_weapons_for_team_matches_c() {
        unsafe {
            let cases: &[(i32, i32, &core::ffi::CStr)] = &[
                // ENEMY: saber users, prefix matches, blaster default, droids
                (NPCTEAM_ENEMY, 0, c"tavion"),
                (NPCTEAM_ENEMY, 0, c"reborn_new"),
                (NPCTEAM_ENEMY, 0, c"desann"),
                (NPCTEAM_ENEMY, 0, c"shadowtrooper"),
                (NPCTEAM_ENEMY, 0, c"stofficer2"),
                (NPCTEAM_ENEMY, 0, c"stcommander"),
                (NPCTEAM_ENEMY, 0, c"swamptrooper"),
                (NPCTEAM_ENEMY, 0, c"swamptrooper2"),
                (NPCTEAM_ENEMY, 0, c"rockettrooper"),
                (NPCTEAM_ENEMY, 0, c"galak_mech"),
                (NPCTEAM_ENEMY, 0, c"ugnaught"),
                (NPCTEAM_ENEMY, 0, c"granboxer"),
                (NPCTEAM_ENEMY, 0, c"gran"),
                (NPCTEAM_ENEMY, 0, c"rodian"),
                (NPCTEAM_ENEMY, 0, c"interrogator"),
                (NPCTEAM_ENEMY, 0, c"weequay"),
                (NPCTEAM_ENEMY, 0, c"probe"),
                (NPCTEAM_ENEMY, 0, c"trandoshan"),
                (NPCTEAM_ENEMY, 0, c"howler"),
                (NPCTEAM_ENEMY, 0, c"stormtrooper"),
                // PLAYER: spawnflags + names
                (NPCTEAM_PLAYER, 2, c"anything"),
                (NPCTEAM_PLAYER, 4, c"anything"),
                (NPCTEAM_PLAYER, 0, c"jedi_hero"),
                (NPCTEAM_PLAYER, 0, c"luke"),
                (NPCTEAM_PLAYER, 0, c"prisoner"),
                (NPCTEAM_PLAYER, 0, c"bespincop"),
                (NPCTEAM_PLAYER, 0, c"MonMothma"),
                (NPCTEAM_PLAYER, 0, c"rebel"),
                // NEUTRAL
                (NPCTEAM_NEUTRAL, 0, c"mark1"),
                (NPCTEAM_NEUTRAL, 0, c"mark2"),
                (NPCTEAM_NEUTRAL, 0, c"ugnaught"),
                (NPCTEAM_NEUTRAL, 0, c"bartender"),
                (NPCTEAM_NEUTRAL, 0, c"morgankatarn"),
                (NPCTEAM_NEUTRAL, 0, c"someguy"),
                // out-of-range team -> default
                (99, 0, c"whatever"),
            ];
            for &(team, sf, ty) in cases {
                assert_eq!(
                    NPC_WeaponsForTeam(team, sf, ty.as_ptr()),
                    jka_NPC_WeaponsForTeam(team, sf, ty.as_ptr()),
                    "team={team} sf={sf} type={ty:?}"
                );
            }
        }
    }
}
