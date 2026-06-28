//! Harvested-leaf stub of `NPC.c` — the NPC-core file is not yet ported, but two of its
//! file-scope globals are needed by consumers that *are* portable now.
//!
//! `NPC` / `NPCInfo` (NPC.c:34-35) are the "current NPC being thought" pointers the
//! whole NPC track reads through (`#define`-free in C — plain externs declared in
//! b_local.h:63-64). They are set per-think by the not-yet-ported NPC core; until that
//! lands they sit null, exactly as the C zero-inits them at load. Combat-point
//! helpers (`NPC_FreeCombatPoint`/`NPC_SetCombatPoint`) are the first consumers.
//!
//! When NPC.c proper is ported, these definitions move there (and this stub is
//! retired) — do not redefine them.

#![allow(non_upper_case_globals)] // `NPCInfo` kept verbatim from C
#![allow(non_snake_case)] // C names kept verbatim

use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut};

use crate::trap;

use crate::codemp::game::anims::{TORSO_WEAPONIDLE3, TORSO_WEAPONREADY1, TORSO_WEAPONREADY3};
use crate::codemp::game::b_public_h::RANK_LT_JG;
use crate::codemp::game::b_public_h::{
    gNPC_t, BS_ADVANCE_FIGHT, BS_CINEMATIC, BS_DEFAULT, BS_FLEE, BS_FOLLOW_LEADER,
    BS_HUNT_AND_KILL, BS_INVESTIGATE, BS_JUMP, BS_NOCLIP, BS_PATROL, BS_REMOVE, BS_SEARCH,
    BS_SLEEP, BS_STAND_AND_SHOOT, BS_STAND_GUARD, BS_WAIT, BS_WANDER, NPCAI_LOST, SCF_ALT_FIRE,
    SCF_CROUCHED, SCF_FORCED_MARCH, SCF_LEAN_LEFT, SCF_LEAN_RIGHT, SCF_RUNNING, SCF_WALKING,
};
use crate::codemp::game::bg_public::{
    EF2_HELD_BY_MONSTER, EF_DISINTEGRATION, EF_NODRAW, ET_NPC, EV_VICTORY1, EV_VICTORY3,
    MASK_SOLID, PMF_FOLLOW, SETANIM_FLAG_NORMAL, SETANIM_TORSO, TEAM_SPECTATOR, WEAPON_IDLE,
    WEAPON_READY,
};
use crate::codemp::game::bg_weapons_h::{
    WP_BRYAR_PISTOL, WP_DISRUPTOR, WP_EMPLACED_GUN, WP_NONE, WP_SABER, WP_STUN_BATON, WP_THERMAL,
};
use crate::codemp::game::g_active::ClientThink;
use crate::codemp::game::g_local::{
    gclient_t, gentity_t, AEL_DANGER_GREAT, AEL_DISCOVERED, FL_DONT_SHOOT, FRAMETIME,
};
use crate::codemp::game::g_main::{
    d_patched, debugNPCFreeze, g_dismember, g_entities, g_saberRealisticCombat, g_spskill, level,
};
use crate::codemp::game::g_nav::G_Cube;
use crate::codemp::game::g_public_h::{BSET_ATTACK, BSET_DELAYED, SVF_ICARUS_FREEZE, TID_MOVE_NAV};
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_Find, G_FreeEntity, G_SetAnim, G_SetOrigin, G_SoundOnEnt};
use crate::codemp::game::npc_ai_atst::NPC_BSATST_Default;
use crate::codemp::game::npc_ai_droid::NPC_BSDroid_Default;
use crate::codemp::game::npc_ai_galakmech::{GM_Dying, NPC_BSGM_Default};
use crate::codemp::game::npc_ai_grenadier::NPC_BSGrenadier_Default;
use crate::codemp::game::npc_ai_howler::NPC_BSHowler_Default;
use crate::codemp::game::npc_ai_imperialprobe::NPC_BSImperialProbe_Default;
use crate::codemp::game::npc_ai_interrogator::NPC_BSInterrogator_Default;
use crate::codemp::game::npc_ai_jedi::{NPC_BSJedi_Default, NPC_BSJedi_FollowLeader};
use crate::codemp::game::npc_ai_mark1::{Mark1_dying, NPC_BSMark1_Default};
use crate::codemp::game::npc_ai_mark2::NPC_BSMark2_Default;
use crate::codemp::game::npc_ai_minemonster::NPC_BSMineMonster_Default;
use crate::codemp::game::npc_ai_rancor::NPC_BSRancor_Default;
use crate::codemp::game::npc_ai_remote::NPC_BSRemote_Default;
use crate::codemp::game::npc_ai_seeker::NPC_BSSeeker_Default;
use crate::codemp::game::npc_ai_sentry::NPC_BSSentry_Default;
use crate::codemp::game::npc_ai_sniper::NPC_BSSniper_Default;
use crate::codemp::game::npc_ai_stormtrooper::{
    NPC_BSST_Default, NPC_BSST_Investigate, NPC_BSST_Sleep,
};
use crate::codemp::game::npc_ai_wampa::NPC_BSWampa_Default;
use crate::codemp::game::npc_behavior::{
    NPC_BSAdvanceFight, NPC_BSCinematic, NPC_BSEmplaced, NPC_BSFlee, NPC_BSFollowLeader,
    NPC_BSRemove, NPC_BSSearch, NPC_BSSleep, NPC_BSWait, NPC_BSWander, NPC_CheckSurrender,
    NPC_StartFlee,
};
use crate::codemp::game::npc_combat::{G_AddVoiceEvent, G_ClearEnemy, NPC_MaxDistSquaredForWeapon};
use crate::codemp::game::npc_move::NPC_ApplyRoff;
use crate::codemp::game::npc_reactions::{NPC_CheckAllClear, NPC_CheckPlayerAim};
use crate::codemp::game::npc_senses::{eventClearTime, AddSightEvent};
use crate::codemp::game::npc_stats::NPC_LoadParms;
use crate::codemp::game::npc_utils::{
    G_ActivateBehavior, NPC_CheckCharmed, NPC_CheckLookTarget, NPC_SetLookTarget, NPC_UpdateAngles,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleVectors, DotProduct, Q_fabs, VectorAdd, VectorCompare,
    VectorCopy, VectorLengthSquared, VectorSubtract,
};
use crate::codemp::game::q_shared_h::{
    qboolean, trace_t, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_CLIENTS, QFALSE, QTRUE,
};
use crate::codemp::game::q_shared_h::{
    usercmd_t, vec3_t, ANGLE2SHORT, BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_USE, BUTTON_WALKING,
    CHAN_AUTO, PITCH, ROLL, YAW,
};
use crate::codemp::game::surfaceflags_h::{CONTENTS_CORPSE, CONTENTS_NODROP, CONTENTS_TRIGGER};
use crate::codemp::game::teams_h::{
    class_t, CLASS_ATST, CLASS_BOBAFETT, CLASS_GALAKMECH, CLASS_GONK, CLASS_HOWLER,
    CLASS_INTERROGATOR, CLASS_JAWA, CLASS_MARK1, CLASS_MARK2, CLASS_MINEMONSTER, CLASS_MOUSE,
    CLASS_PROBE, CLASS_PROTOCOL, CLASS_R2D2, CLASS_R5D2, CLASS_RANCOR, CLASS_REMOTE, CLASS_SEEKER,
    CLASS_SENTRY, CLASS_UGNAUGHT, CLASS_VEHICLE, CLASS_WAMPA, NPCTEAM_ENEMY, NPCTEAM_NEUTRAL,
};

/// `gentity_t *NPC;` (NPC.c:34) — the entity currently running NPC AI this think.
pub static mut NPC: *mut gentity_t = core::ptr::null_mut();

/// `gNPC_t *NPCInfo;` (NPC.c:35) — `NPC->NPC`, cached for the duration of the think.
pub static mut NPCInfo: *mut gNPC_t = core::ptr::null_mut();

/// `gclient_t *client;` (file-local in NPC.c, declared in b_local.h:65) — `NPC->client`,
/// cached for the duration of the think.
pub static mut client: *mut gclient_t = core::ptr::null_mut();

/// `usercmd_t ucmd;` (file-local in NPC.c, declared in b_local.h) — the movement command
/// built up for the current NPC each think; zeroed at the start of every think.
pub static mut ucmd: usercmd_t = usercmd_t {
    serverTime: 0,
    angles: [0; 3],
    buttons: 0,
    weapon: 0,
    forcesel: 0,
    invensel: 0,
    generic_cmd: 0,
    forwardmove: 0,
    rightmove: 0,
    upmove: 0,
};

/// `#define ALERT_CLEAR_TIME 200` (b_local.h:164) — kept verbatim as a file-local
/// const (matching the C macro shared across the NPC track).
const ALERT_CLEAR_TIME: c_int = 200;

pub unsafe fn CorpsePhysics(self_: *mut gentity_t) {
    // run the bot through the server like it was a real client
    ucmd = usercmd_t::default(); // memset( &ucmd, 0, sizeof( ucmd ) );
    ClientThink((*self_).s.number, addr_of_mut!(ucmd));
    //VectorCopy( self->s.origin, self->s.origin2 );
    //rww - don't get why this is happening.

    if (*(*self_).client).NPC_class == CLASS_GALAKMECH {
        GM_Dying(self_);
    }
    //FIXME: match my pitch and roll for the slope of my groundPlane
    if (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE
        && (*self_).s.eFlags & EF_DISINTEGRATION == 0
    {
        //on the ground
        //FIXME: check 4 corners
        pitch_roll_for_slope(self_, core::ptr::null());
    }

    if eventClearTime == (*addr_of!(level)).time + ALERT_CLEAR_TIME {
        //events were just cleared out so add me again
        if (*(*self_).client).ps.eFlags & EF_NODRAW == 0 {
            AddSightEvent(
                (*self_).enemy,
                &(*self_).r.currentOrigin,
                384.0,
                AEL_DISCOVERED,
                0.0,
            );
        }
    }

    if (*addr_of!(level)).time - (*self_).s.time > 3000 {
        //been dead for 3 seconds
        if (*addr_of!(g_dismember)).integer < 11381138
            && (*addr_of!(g_saberRealisticCombat)).integer == 0
        {
            //can't be dismembered once dead
            if (*(*self_).client).NPC_class != CLASS_PROTOCOL {
                //	self->client->dismembered = qtrue;
            }
        }
    }

    //if ( level.time - self->s.time > 500 )
    if (*(*self_).client).respawnTime < (*addr_of!(level)).time + 500 {
        //don't turn "nonsolid" until about 1 second after actual death

        if (*(*self_).client).ps.eFlags & EF_DISINTEGRATION != 0 {
            (*self_).r.contents = 0;
        } else if (*(*self_).client).NPC_class != CLASS_MARK1
            && (*(*self_).client).NPC_class != CLASS_INTERROGATOR
        {
            // The Mark1 & Interrogator stays solid.
            (*self_).r.contents = CONTENTS_CORPSE;
            //self->r.maxs[2] = -8;
        }

        if !(*self_).message.is_null() {
            (*self_).r.contents |= CONTENTS_TRIGGER;
        }
    }
}

/*
----------------------------------------
NPC_RemoveBody

Determines when it's ok to ditch the corpse
----------------------------------------
*/

pub unsafe fn BodyRemovalPadTime(ent: *mut gentity_t) -> c_int {
    let time: c_int;

    if ent.is_null() || (*ent).client.is_null() {
        return 0;
    }
    /*
        switch ( ent->client->playerTeam )
        {
        case NPCTEAM_KLINGON:	// no effect, we just remove them when the player isn't looking
        case NPCTEAM_SCAVENGERS:
        case NPCTEAM_HIROGEN:
        case NPCTEAM_MALON:
        case NPCTEAM_IMPERIAL:
        case NPCTEAM_STARFLEET:
            time = 10000; // 15 secs.
            break;

        case NPCTEAM_BORG:
            time = 2000;
            break;

        case NPCTEAM_STASIS:
            return qtrue;
            break;

        case NPCTEAM_FORGE:
            time = 1000;
            break;

        case NPCTEAM_BOTS:
    //		if (!Q_stricmp( ent->NPC_type, "mouse" ))
    //		{
                time = 0;
    //		}
    //		else
    //		{
    //			time = 10000;
    //		}
            break;

        case NPCTEAM_8472:
            time = 2000;
            break;

        default:
            // never go away
            time = Q3_INFINITE;
            break;
        }
    */
    // team no longer indicates species/race, so in this case we'd use NPC_class, but
    match (*(*ent).client).NPC_class {
        CLASS_MOUSE
        | CLASS_GONK
        | CLASS_R2D2
        | CLASS_R5D2
        // case CLASS_PROTOCOL:
        | CLASS_MARK1
        | CLASS_MARK2
        | CLASS_PROBE
        | CLASS_SEEKER
        | CLASS_REMOTE
        | CLASS_SENTRY
        | CLASS_INTERROGATOR => {
            time = 0;
        }
        _ => {
            // never go away
            //	time = Q3_INFINITE;
            // for now I'm making default 10000
            time = 10000;
        }
    }

    time
}

/*
----------------------------------------
NPC_RemoveBodyEffect

Effect to be applied when ditching the corpse
----------------------------------------
*/

unsafe fn NPC_RemoveBodyEffect() {
    //	vec3_t		org;
    //	gentity_t	*tent;

    if NPC.is_null() || (*NPC).client.is_null() || ((*NPC).s.eFlags & EF_NODRAW) != 0 {
        return;
    }
    /*
        switch(NPC->client->playerTeam)
        {
        case NPCTEAM_STARFLEET:
            //FIXME: Starfleet beam out
            break;

        case NPCTEAM_BOTS:
    //		VectorCopy( NPC->r.currentOrigin, org );
    //		org[2] -= 16;
    //		tent = G_TempEntity( org, EV_BOT_EXPLODE );
    //		tent->owner = NPC;

            break;

        default:
            break;
        }
    */

    // team no longer indicates species/race, so in this case we'd use NPC_class, but

    // stub code
    match (*(*NPC).client).NPC_class {
        CLASS_PROBE
        | CLASS_SEEKER
        | CLASS_REMOTE
        | CLASS_SENTRY
        | CLASS_GONK
        | CLASS_R2D2
        | CLASS_R5D2
        // case CLASS_PROTOCOL:
        | CLASS_MARK1
        | CLASS_MARK2
        | CLASS_INTERROGATOR
        | CLASS_ATST => {
            // yeah, this is a little weird, but for now I'm listing all droids
            //	VectorCopy( NPC->r.currentOrigin, org );
            //	org[2] -= 16;
            //	tent = G_TempEntity( org, EV_BOT_EXPLODE );
            //	tent->owner = NPC;
        }
        _ => {}
    }
}

pub unsafe fn NPC_AvoidWallsAndCliffs() {
    //...
}

pub unsafe fn NPC_InitAI() {
    /*
    trap_Cvar_Register(&g_saberRealisticCombat, "g_saberRealisticCombat", "0", CVAR_CHEAT);

    trap_Cvar_Register(&debugNoRoam, "d_noroam", "0", CVAR_CHEAT);
    trap_Cvar_Register(&debugNPCAimingBeam, "d_npcaiming", "0", CVAR_CHEAT);
    trap_Cvar_Register(&debugBreak, "d_break", "0", CVAR_CHEAT);
    trap_Cvar_Register(&debugNPCAI, "d_npcai", "0", CVAR_CHEAT);
    trap_Cvar_Register(&debugNPCFreeze, "d_npcfreeze", "0", CVAR_CHEAT);
    trap_Cvar_Register(&d_JediAI, "d_JediAI", "0", CVAR_CHEAT);
    trap_Cvar_Register(&d_noGroupAI, "d_noGroupAI", "0", CVAR_CHEAT);
    trap_Cvar_Register(&d_asynchronousGroupAI, "d_asynchronousGroupAI", "0", CVAR_CHEAT);

    //0 = never (BORING)
    //1 = kyle only
    //2 = kyle and last enemy jedi
    //3 = kyle and any enemy jedi
    //4 = kyle and last enemy in a group
    //5 = kyle and any enemy
    //6 = also when kyle takes pain or enemy jedi dodges player saber swing or does an acrobatic evasion

    trap_Cvar_Register(&d_slowmodeath, "d_slowmodeath", "0", CVAR_CHEAT);

    trap_Cvar_Register(&d_saberCombat, "d_saberCombat", "0", CVAR_CHEAT);

    trap_Cvar_Register(&g_spskill, "g_npcspskill", "0", CVAR_ARCHIVE | CVAR_USERINFO);
    */
}

/*
===============
SetNPCGlobals

local function to set globals used throughout the AI code
===============
*/
pub unsafe fn SetNPCGlobals(ent: *mut gentity_t) {
    NPC = ent;
    NPCInfo = (*ent).NPC;
    client = (*ent).client;
    ucmd = usercmd_t::default(); // memset( &ucmd, 0, sizeof( usercmd_t ) );
}

static mut _saved_NPC: *mut gentity_t = core::ptr::null_mut();
static mut _saved_NPCInfo: *mut gNPC_t = core::ptr::null_mut();
static mut _saved_client: *mut gclient_t = core::ptr::null_mut();
static mut _saved_ucmd: usercmd_t = usercmd_t {
    serverTime: 0,
    angles: [0; 3],
    buttons: 0,
    weapon: 0,
    forcesel: 0,
    invensel: 0,
    generic_cmd: 0,
    forwardmove: 0,
    rightmove: 0,
    upmove: 0,
};

pub unsafe fn SaveNPCGlobals() {
    _saved_NPC = NPC;
    _saved_NPCInfo = NPCInfo;
    _saved_client = client;
    _saved_ucmd = ucmd; // memcpy( &_saved_ucmd, &ucmd, sizeof( usercmd_t ) );
}

pub unsafe fn RestoreNPCGlobals() {
    NPC = _saved_NPC;
    NPCInfo = _saved_NPCInfo;
    client = _saved_client;
    ucmd = _saved_ucmd; // memcpy( &ucmd, &_saved_ucmd, sizeof( usercmd_t ) );
}

//We MUST do this, other funcs were using NPC illegally when "self" wasn't the global NPC
pub unsafe fn ClearNPCGlobals() {
    NPC = core::ptr::null_mut();
    NPCInfo = core::ptr::null_mut();
    client = core::ptr::null_mut();
}
//===============

/*
-------------------------
void NPC_KeepCurrentFacing(void)

Fills in a default ucmd to keep current angles facing
*/
pub unsafe fn NPC_KeepCurrentFacing() {
    if ucmd.angles[YAW] == 0 {
        ucmd.angles[YAW] =
            ANGLE2SHORT((*client).ps.viewangles[YAW]) - (*client).ps.delta_angles[YAW];
    }

    if ucmd.angles[PITCH] == 0 {
        ucmd.angles[PITCH] =
            ANGLE2SHORT((*client).ps.viewangles[PITCH]) - (*client).ps.delta_angles[PITCH];
    }
}

/*
====================================================================
void pitch_roll_for_slope( gentity_t *forwhom, vec3_t pass_slope )

MG

This will adjust the pitch and roll of a monster to match
a given slope - if a non-'0 0 0' slope is passed, it will
use that value, otherwise it will use the ground underneath
the monster.  If it doesn't find a surface, it does nothinh\g
and returns.
====================================================================
*/

pub unsafe fn pitch_roll_for_slope(forwhom: *mut gentity_t, pass_slope: *const vec3_t) {
    let mut slope: vec3_t = [0.0; 3];
    let mut nvf: vec3_t = [0.0; 3];
    let mut ovf: vec3_t = [0.0; 3];
    let mut ovr: vec3_t = [0.0; 3];
    let mut startspot: vec3_t = [0.0; 3];
    let mut endspot: vec3_t;
    let mut new_angles: vec3_t = [0.0, 0.0, 0.0];
    let pitch: f32;
    let mod_: f32;
    let dot: f32;

    //if we don't have a slope, get one
    if pass_slope.is_null() || VectorCompare(&vec3_origin, &*pass_slope) != 0 {
        VectorCopy(&(*forwhom).r.currentOrigin, &mut startspot);
        startspot[2] += (*forwhom).r.mins[2] + 4.0;
        endspot = startspot;
        endspot[2] -= 300.0;
        let trace = trap::Trace(
            &(*forwhom).r.currentOrigin,
            &vec3_origin,
            &vec3_origin,
            &endspot,
            (*forwhom).s.number,
            MASK_SOLID,
        );
        //		if(trace_fraction>0.05&&forwhom.movetype==MOVETYPE_STEP)
        //			forwhom.flags(-)FL_ONGROUND;

        if trace.fraction >= 1.0 {
            return;
        }

        // `if( !( &trace.plane ) )` — the C tests the address of a struct member,
        // which is never null, so this guard can never fire. Preserved for fidelity.

        if VectorCompare(&vec3_origin, &trace.plane.normal) != 0 {
            return;
        }

        VectorCopy(&trace.plane.normal, &mut slope);
    } else {
        VectorCopy(&*pass_slope, &mut slope);
    }

    AngleVectors(
        &(*forwhom).r.currentAngles,
        Some(&mut ovf),
        Some(&mut ovr),
        None,
    );

    vectoangles(&slope, &mut new_angles);
    pitch = new_angles[PITCH] + 90.0;
    new_angles[ROLL] = 0.0;
    new_angles[PITCH] = 0.0;

    AngleVectors(&new_angles, Some(&mut nvf), None, None);

    mod_ = DotProduct(&nvf, &ovr);

    let mod_ = if mod_ < 0.0 { -1.0 } else { 1.0 };

    dot = DotProduct(&nvf, &ovf);

    if !(*forwhom).client.is_null() {
        let oldmins2: f32;

        (*(*forwhom).client).ps.viewangles[PITCH] = dot * pitch;
        (*(*forwhom).client).ps.viewangles[ROLL] = (1.0 - Q_fabs(dot)) * pitch * mod_;
        oldmins2 = (*forwhom).r.mins[2];
        (*forwhom).r.mins[2] = (-24.0
            + 12.0 * ((*(*forwhom).client).ps.viewangles[PITCH] as f64).abs() / 180.0_f32 as f64)
            as f32;
        //FIXME: if it gets bigger, move up
        if oldmins2 > (*forwhom).r.mins[2] {
            //our mins is now lower, need to move up
            //FIXME: trace?
            (*(*forwhom).client).ps.origin[2] += oldmins2 - (*forwhom).r.mins[2];
            (*forwhom).r.currentOrigin[2] = (*(*forwhom).client).ps.origin[2];
            trap::LinkEntity(forwhom);
        }
    } else {
        (*forwhom).r.currentAngles[PITCH] = dot * pitch;
        (*forwhom).r.currentAngles[ROLL] = (1.0 - Q_fabs(dot)) * pitch * mod_;
    }
}

//===============

// `extern qboolean showBBoxes;` (NPC.c:657; defined NPC_spawn.c:4211). Until NPC_spawn's
// debug-bbox toggle lands, carried here as a local file-scope static so the only consumer
// (NPC_ShowDebugInfo) compiles; it sits `qfalse`, exactly as the C zero-inits it.
static mut showBBoxes: qboolean = QFALSE;

// `vec3_t NPCDEBUG_RED = {1.0, 0.0, 0.0};` (NPC.c:658). Only RED is used in MP NPC.c
// (NPC_ShowDebugInfo); the GREEN/BLUE/LIGHT_BLUE siblings are unused here and omitted.
static mut NPCDEBUG_RED: vec3_t = [1.0, 0.0, 0.0];

pub unsafe fn NPC_ShowDebugInfo() {
    if showBBoxes != QFALSE {
        let mut found: *mut gentity_t = core::ptr::null_mut();
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];

        loop {
            found = G_Find(
                found,
                offset_of!(gentity_t, classname),
                c"NPC".as_ptr() as *const c_char,
            );
            if found.is_null() {
                break;
            }

            if trap::InPVS(
                &(*found).r.currentOrigin,
                &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(0))
                    .r
                    .currentOrigin,
            ) != QFALSE
            {
                VectorAdd(&(*found).r.currentOrigin, &(*found).r.mins, &mut mins);
                VectorAdd(&(*found).r.currentOrigin, &(*found).r.maxs, &mut maxs);
                G_Cube(&mins, &maxs, &*addr_of!(NPCDEBUG_RED), 0.25);
            }
        }
    }
}

pub unsafe fn NPC_ApplyScriptFlags() {
    if (*NPCInfo).scriptFlags & SCF_CROUCHED != 0 {
        if (*NPCInfo).charmedTime > (*addr_of!(level)).time
            && (ucmd.forwardmove != 0 || ucmd.rightmove != 0)
        {
            //ugh, if charmed and moving, ignore the crouched command
        } else {
            ucmd.upmove = -127;
        }
    }

    if (*NPCInfo).scriptFlags & SCF_RUNNING != 0 {
        ucmd.buttons &= !BUTTON_WALKING;
    } else if (*NPCInfo).scriptFlags & SCF_WALKING != 0 {
        if (*NPCInfo).charmedTime > (*addr_of!(level)).time
            && (ucmd.forwardmove != 0 || ucmd.rightmove != 0)
        {
            //ugh, if charmed and moving, ignore the walking command
        } else {
            ucmd.buttons |= BUTTON_WALKING;
        }
    }
    /*
        if(NPCInfo->scriptFlags & SCF_CAREFUL)
        {
            ucmd.buttons |= BUTTON_CAREFUL;
        }
    */
    if (*NPCInfo).scriptFlags & SCF_LEAN_RIGHT != 0 {
        ucmd.buttons |= BUTTON_USE;
        ucmd.rightmove = 127;
        ucmd.forwardmove = 0;
        ucmd.upmove = 0;
    } else if (*NPCInfo).scriptFlags & SCF_LEAN_LEFT != 0 {
        ucmd.buttons |= BUTTON_USE;
        ucmd.rightmove = -127;
        ucmd.forwardmove = 0;
        ucmd.upmove = 0;
    }

    if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 && ucmd.buttons & BUTTON_ATTACK != 0 {
        //Use altfire instead
        ucmd.buttons |= BUTTON_ALT_ATTACK;
    }
}

pub unsafe fn NPC_CheckAttackHold() {
    let mut vec: vec3_t = [0.0; 3];

    // If they don't have an enemy they shouldn't hold their attack anim.
    if (*NPC).enemy.is_null() {
        (*NPCInfo).attackHoldTime = 0;
        return;
    }

    /*	if ( ( NPC->client->ps.weapon == WP_BORG_ASSIMILATOR ) || ( NPC->client->ps.weapon == WP_BORG_DRILL ) )
    {//FIXME: don't keep holding this if can't hit enemy?

        // If they don't have shields ( been disabled) they shouldn't hold their attack anim.
        if ( !(NPC->NPC->aiFlags & NPCAI_SHIELDS) )
        {
            NPCInfo->attackHoldTime = 0;
            return;
        }

        VectorSubtract(NPC->enemy->r.currentOrigin, NPC->r.currentOrigin, vec);
        if( VectorLengthSquared(vec) > NPC_MaxDistSquaredForWeapon() )
        {
            NPCInfo->attackHoldTime = 0;
            PM_SetTorsoAnimTimer(NPC, &NPC->client->ps.torsoAnimTimer, 0);
        }
        else if( NPCInfo->attackHoldTime && NPCInfo->attackHoldTime > level.time )
        {
            ucmd.buttons |= BUTTON_ATTACK;
        }
        else if ( ( NPCInfo->attackHold ) && ( ucmd.buttons & BUTTON_ATTACK ) )
        {
            NPCInfo->attackHoldTime = level.time + NPCInfo->attackHold;
            PM_SetTorsoAnimTimer(NPC, &NPC->client->ps.torsoAnimTimer, NPCInfo->attackHold);
        }
        else
        {
            NPCInfo->attackHoldTime = 0;
            PM_SetTorsoAnimTimer(NPC, &NPC->client->ps.torsoAnimTimer, 0);
        }
    }
    else*/
    {
        //everyone else...?  FIXME: need to tie this into AI somehow?
        VectorSubtract(
            &(*(*NPC).enemy).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &mut vec,
        );
        if VectorLengthSquared(&vec) > NPC_MaxDistSquaredForWeapon() {
            (*NPCInfo).attackHoldTime = 0;
        } else if (*NPCInfo).attackHoldTime != 0
            && (*NPCInfo).attackHoldTime > (*addr_of!(level)).time
        {
            ucmd.buttons |= BUTTON_ATTACK;
        } else if (*NPCInfo).attackHold != 0 && ucmd.buttons & BUTTON_ATTACK != 0 {
            (*NPCInfo).attackHoldTime = (*addr_of!(level)).time + (*NPCInfo).attackHold;
        } else {
            (*NPCInfo).attackHoldTime = 0;
        }
    }
}

pub unsafe fn NPC_CheckAttackScript() {
    if ucmd.buttons & BUTTON_ATTACK == 0 {
        return;
    }

    G_ActivateBehavior(NPC, BSET_ATTACK);
}

pub unsafe fn NPC_CheckInSolid() {
    let mut point: vec3_t = [0.0; 3];
    VectorCopy(&(*NPC).r.currentOrigin, &mut point);
    point[2] -= 0.25;

    let trace = trap::Trace(
        &(*NPC).r.currentOrigin,
        &(*NPC).r.mins,
        &(*NPC).r.maxs,
        &point,
        (*NPC).s.number,
        (*NPC).clipmask,
    );
    if trace.startsolid == 0 && trace.allsolid == 0 {
        VectorCopy(&(*NPC).r.currentOrigin, &mut (*NPCInfo).lastClearOrigin);
    } else if VectorLengthSquared(&(*NPCInfo).lastClearOrigin) != 0.0 {
        //			Com_Printf("%s stuck in solid at %s: fixing...\n", NPC->script_targetname, vtos(NPC->r.currentOrigin));
        G_SetOrigin(NPC, &(*NPCInfo).lastClearOrigin);
        trap::LinkEntity(NPC);
    }
}

pub unsafe fn G_DroidSounds(self_: *mut gentity_t) {
    if !(*self_).client.is_null() {
        //make the noises
        if TIMER_Done(self_, c"patrolNoise".as_ptr()) != 0 && Q_irand(0, 20) == 0 {
            match (*(*self_).client).NPC_class {
                CLASS_R2D2 =>
                // droid
                {
                    G_SoundOnEnt(
                        self_,
                        CHAN_AUTO,
                        &format!("sound/chars/r2d2/misc/r2d2talk0{}.wav", Q_irand(1, 3)),
                    );
                }
                CLASS_R5D2 =>
                // droid
                {
                    G_SoundOnEnt(
                        self_,
                        CHAN_AUTO,
                        &format!("sound/chars/r5d2/misc/r5talk{}.wav", Q_irand(1, 4)),
                    );
                }
                CLASS_PROBE =>
                // droid
                {
                    G_SoundOnEnt(
                        self_,
                        CHAN_AUTO,
                        &format!("sound/chars/probe/misc/probetalk{}.wav", Q_irand(1, 3)),
                    );
                }
                CLASS_MOUSE =>
                // droid
                {
                    G_SoundOnEnt(
                        self_,
                        CHAN_AUTO,
                        &format!("sound/chars/mouse/misc/mousego{}.wav", Q_irand(1, 3)),
                    );
                }
                CLASS_GONK =>
                // droid
                {
                    G_SoundOnEnt(
                        self_,
                        CHAN_AUTO,
                        &format!("sound/chars/gonk/misc/gonktalk{}.wav", Q_irand(1, 2)),
                    );
                }
                _ => {}
            }
            TIMER_Set(self_, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
        }
    }
}

pub unsafe fn NPC_SetAnim(
    ent: *mut gentity_t,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
) {
    // FIXME : once torsoAnim and legsAnim are in the same structure for NCP and Players
    // rename PM_SETAnimFinal to PM_SetAnim and have both NCP and Players call PM_SetAnim
    G_SetAnim(
        ent,
        core::ptr::null_mut(),
        setAnimParts,
        anim,
        setAnimFlags,
        0,
    );
    /*
    if(ent->client)
    {//Players, NPCs
        if (setAnimFlags&SETANIM_FLAG_OVERRIDE)
        {
            if (setAnimParts & SETANIM_TORSO)
            {
                if( (setAnimFlags & SETANIM_FLAG_RESTART) || ent->client->ps.torsoAnim != anim )
                {
                    PM_SetTorsoAnimTimer( ent, &ent->client->ps.torsoTimer, 0 );
                }
            }
            if (setAnimParts & SETANIM_LEGS)
            {
                if( (setAnimFlags & SETANIM_FLAG_RESTART) || ent->client->ps.legsAnim != anim )
                {
                    PM_SetLegsAnimTimer( ent, &ent->client->ps.legsAnimTimer, 0 );
                }
            }
        }

        PM_SetAnimFinal(&ent->client->ps.torsoAnim,&ent->client->ps.legsAnim,setAnimParts,anim,setAnimFlags,
            &ent->client->ps.torsoAnimTimer,&ent->client->ps.legsAnimTimer,ent);
    }
    else
    {//bodies, etc.
        if (setAnimFlags&SETANIM_FLAG_OVERRIDE)
        {
            if (setAnimParts & SETANIM_TORSO)
            {
                if( (setAnimFlags & SETANIM_FLAG_RESTART) || ent->s.torsoAnim != anim )
                {
                    PM_SetTorsoAnimTimer( ent, &ent->s.torsoAnimTimer, 0 );
                }
            }
            if (setAnimParts & SETANIM_LEGS)
            {
                if( (setAnimFlags & SETANIM_FLAG_RESTART) || ent->s.legsAnim != anim )
                {
                    PM_SetLegsAnimTimer( ent, &ent->s.legsAnimTimer, 0 );
                }
            }
        }

        PM_SetAnimFinal(&ent->s.torsoAnim,&ent->s.legsAnim,setAnimParts,anim,setAnimFlags,
            &ent->s.torsoAnimTimer,&ent->s.legsAnimTimer,ent);
    }
    */
}

/// `void NPC_RemoveBody( gentity_t *self )` (NPC.c:116) — the corpse's per-frame
/// `think` once the death sequence is done. Runs corpse physics, pumps the ICARUS
/// task manager, then decides when it is OK to ditch the body: exploding droids drop
/// their bbox and free immediately; Galak never disappears; enemies / protocol droids
/// (and their thrown saber) linger then free, unless still held by a Rancor.
///
/// `pub unsafe extern "C" fn` so it slots into the `gentity_t::think` fn-pointer ABI
/// (`DeadThink` assigns `self->think = NPC_RemoveBody`).
///
/// # Safety
/// `self_` must be a valid NPC entity with non-null `client`/`NPC`.
pub unsafe extern "C" fn NPC_RemoveBody(self_: *mut gentity_t) {
    CorpsePhysics(self_);

    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;

    if (*(*self_).NPC).nextBStateThink <= (*addr_of!(level)).time {
        trap::ICARUS_MaintainTaskManager((*self_).s.number);
    }
    (*(*self_).NPC).nextBStateThink = (*addr_of!(level)).time + FRAMETIME;

    if !(*self_).message.is_null() {
        // I still have a key
        return;
    }

    // I don't consider this a hack, it's creative coding . . .
    // I agree, very creative... need something like this for ATST and GALAKMECH too!
    if (*(*self_).client).NPC_class == CLASS_MARK1 {
        Mark1_dying(self_);
    }

    // Since these blow up, remove the bounding box.
    if (*(*self_).client).NPC_class == CLASS_REMOTE
        || (*(*self_).client).NPC_class == CLASS_SENTRY
        || (*(*self_).client).NPC_class == CLASS_PROBE
        || (*(*self_).client).NPC_class == CLASS_INTERROGATOR
        || (*(*self_).client).NPC_class == CLASS_PROBE
        || (*(*self_).client).NPC_class == CLASS_MARK2
    {
        //if ( !self->taskManager || !self->taskManager->IsRunning() )
        if trap::ICARUS_IsRunning((*self_).s.number) == QFALSE {
            if (*self_).activator.is_null()
                || (*(*self_).activator).client.is_null()
                || (*(*(*self_).activator).client).ps.eFlags2 & EF2_HELD_BY_MONSTER == 0
            {
                // not being held by a Rancor
                G_FreeEntity(self_);
            }
        }
        return;
    }

    //FIXME: don't ever inflate back up?
    (*self_).r.maxs[2] =
        (*(*self_).client).renderInfo.eyePoint[2] - (*self_).r.currentOrigin[2] + 4.0;
    if (*self_).r.maxs[2] < -8.0 {
        (*self_).r.maxs[2] = -8.0;
    }

    if (*(*self_).client).NPC_class == CLASS_GALAKMECH {
        // never disappears
        return;
    }
    if !(*self_).NPC.is_null() && (*(*self_).NPC).timeOfDeath <= (*addr_of!(level)).time {
        (*(*self_).NPC).timeOfDeath = (*addr_of!(level)).time + 1000;
        // Only do all of this nonsense for Scav boys ( and girls )
        // should I check NPC_class here instead of TEAM ? - dmv
        if (*(*self_).client).playerTeam == NPCTEAM_ENEMY
            || (*(*self_).client).NPC_class == CLASS_PROTOCOL
        {
            (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME; // try back in a second

            /*
            ... (FOV/LOS player-proximity hold is MP-disabled in the original) ...
            */
            //Don't care about this for MP I guess.
        }

        //FIXME: there are some conditions - such as heavy combat - in which we want
        //			to remove the bodies... but in other cases it's just weird, like
        //			when they're right behind you in a closed room and when they've been
        //			placed as dead NPCs by a designer...
        //			For now we just assume that a corpse with no enemy was
        //			placed in the map as a corpse
        if !(*self_).enemy.is_null() {
            //if ( !self->taskManager || !self->taskManager->IsRunning() )
            if trap::ICARUS_IsRunning((*self_).s.number) == QFALSE
                && ((*self_).activator.is_null()
                    || (*(*self_).activator).client.is_null()
                    || (*(*(*self_).activator).client).ps.eFlags2 & EF2_HELD_BY_MONSTER == 0)
            {
                // not being held by a Rancor
                if !(*self_).client.is_null()
                    && (*(*self_).client).ps.saberEntityNum > 0
                    && (*(*self_).client).ps.saberEntityNum < ENTITYNUM_WORLD
                {
                    let saberent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                        .cast::<gentity_t>())
                    .offset((*(*self_).client).ps.saberEntityNum as isize);
                    if !saberent.is_null() {
                        G_FreeEntity(saberent);
                    }
                }
                G_FreeEntity(self_);
            }
        }
    }
}

/// `static void DeadThink( void )` (NPC.c:480) — the per-think entry for a dead NPC.
/// Re-inflates the Ghoul2 collision bbox (clamped, only while at rest), then either
/// waits out `BodyRemovalPadTime` and hands off to `NPC_RemoveBody`/`G_FreeEntity`
/// (droids vanish via `EF_NODRAW`), or marks nodrop corpses nodraw, finally running
/// `CorpsePhysics`. Reads the file-global `NPC`/`NPCInfo` (set by `SetNPCGlobals`).
/// C `static`, reached only via `NPC_Think`.
///
/// # Safety
/// `NPC`/`NPCInfo` must point at the current think entity.
unsafe fn DeadThink() {
    let mut trace: trace_t;

    //HACKHACKHACKHACKHACK
    //We should really have a seperate G2 bounding box (seperate from the physics bbox) for G2 collisions only
    //FIXME: don't ever inflate back up?
    (*NPC).r.maxs[2] = (*(*NPC).client).renderInfo.eyePoint[2] - (*NPC).r.currentOrigin[2] + 4.0;
    if (*NPC).r.maxs[2] < -8.0 {
        (*NPC).r.maxs[2] = -8.0;
    }
    if VectorCompare(&(*(*NPC).client).ps.velocity, &vec3_origin) != QFALSE {
        //not flying through the air
        if (*NPC).r.mins[0] > -32.0 {
            (*NPC).r.mins[0] -= 1.0;
            trace = trap::Trace(
                &(*NPC).r.currentOrigin,
                &(*NPC).r.mins,
                &(*NPC).r.maxs,
                &(*NPC).r.currentOrigin,
                (*NPC).s.number,
                (*NPC).clipmask,
            );
            if trace.allsolid != 0 {
                (*NPC).r.mins[0] += 1.0;
            }
        }
        if (*NPC).r.maxs[0] < 32.0 {
            (*NPC).r.maxs[0] += 1.0;
            trace = trap::Trace(
                &(*NPC).r.currentOrigin,
                &(*NPC).r.mins,
                &(*NPC).r.maxs,
                &(*NPC).r.currentOrigin,
                (*NPC).s.number,
                (*NPC).clipmask,
            );
            if trace.allsolid != 0 {
                (*NPC).r.maxs[0] -= 1.0;
            }
        }
        if (*NPC).r.mins[1] > -32.0 {
            (*NPC).r.mins[1] -= 1.0;
            trace = trap::Trace(
                &(*NPC).r.currentOrigin,
                &(*NPC).r.mins,
                &(*NPC).r.maxs,
                &(*NPC).r.currentOrigin,
                (*NPC).s.number,
                (*NPC).clipmask,
            );
            if trace.allsolid != 0 {
                (*NPC).r.mins[1] += 1.0;
            }
        }
        if (*NPC).r.maxs[1] < 32.0 {
            (*NPC).r.maxs[1] += 1.0;
            trace = trap::Trace(
                &(*NPC).r.currentOrigin,
                &(*NPC).r.mins,
                &(*NPC).r.maxs,
                &(*NPC).r.currentOrigin,
                (*NPC).s.number,
                (*NPC).clipmask,
            );
            if trace.allsolid != 0 {
                (*NPC).r.maxs[1] -= 1.0;
            }
        }
    }
    //HACKHACKHACKHACKHACK

    //FIXME: tilt and fall off of ledges?
    //NPC_PostDeathThink();

    // (the timeOfDeath death-anim-completion block is commented out in the original)
    {
        //death anim done (or were given a specific amount of time to wait before removal), wait the requisite amount of time them remove
        if (*addr_of!(level)).time >= (*NPCInfo).timeOfDeath + BodyRemovalPadTime(NPC) {
            if (*(*NPC).client).ps.eFlags & EF_NODRAW != 0 {
                //if ( !NPC->taskManager || !NPC->taskManager->IsRunning() )
                if trap::ICARUS_IsRunning((*NPC).s.number) == QFALSE {
                    (*NPC).think = Some(G_FreeEntity);
                    (*NPC).nextthink = (*addr_of!(level)).time + FRAMETIME;
                }
            } else {
                // Start the body effect first, then delay 400ms before ditching the corpse
                NPC_RemoveBodyEffect();

                //FIXME: keep it running through physics somehow?
                (*NPC).think = Some(NPC_RemoveBody);
                (*NPC).nextthink = (*addr_of!(level)).time + FRAMETIME;
                let npc_class: class_t = (*(*NPC).client).NPC_class;
                // check for droids
                if npc_class == CLASS_SEEKER
                    || npc_class == CLASS_REMOTE
                    || npc_class == CLASS_PROBE
                    || npc_class == CLASS_MOUSE
                    || npc_class == CLASS_GONK
                    || npc_class == CLASS_R2D2
                    || npc_class == CLASS_R5D2
                    || npc_class == CLASS_MARK2
                    || npc_class == CLASS_SENTRY
                //npc_class == CLASS_PROTOCOL ||
                {
                    (*(*NPC).client).ps.eFlags |= EF_NODRAW;
                    (*NPCInfo).timeOfDeath = (*addr_of!(level)).time + FRAMETIME * 8;
                } else {
                    (*NPCInfo).timeOfDeath = (*addr_of!(level)).time + FRAMETIME * 4;
                }
            }
            return;
        }
    }

    // If the player is on the ground and the resting position contents haven't been set yet...(BounceCount tracks the contents)
    if (*NPC).bounceCount < 0 && (*NPC).s.groundEntityNum >= 0 {
        // if client is in a nodrop area, make him/her nodraw
        let contents: c_int = trap::PointContents(&(*NPC).r.currentOrigin, -1);
        (*NPC).bounceCount = contents;

        if contents & CONTENTS_NODROP != 0 {
            (*(*NPC).client).ps.eFlags |= EF_NODRAW;
        }
    }

    CorpsePhysics(NPC);
}

/// `void NPC_InitGame( void )` (NPC.c:2058) — game-load NPC setup: load the NPC parm
/// files and register AI cvars. The TagMalloc of `globals.NPCs`, the debug-name cvar,
/// and the team-counter reset are all `#if 0`/commented in the original. Called once
/// from `GAME_INIT` (wiring lands when g_main reaches it).
///
/// # Safety
/// Must run at game-load with the trap layer live (`NPC_LoadParms` reads files).
pub unsafe fn NPC_InitGame() {
    //	globals.NPCs = (gNPC_t *) gi.TagMalloc(game.maxclients * sizeof(game.bots[0]), TAG_GAME);
    //	trap_Cvar_Register(&debugNPCName, "d_npc", "0", CVAR_CHEAT);

    NPC_LoadParms();
    NPC_InitAI();
    //	NPC_InitAnimTable();
}

// ===========================================================================
// Guarded stubs for not-yet-ported NPC-AI callees (keystone-with-guarded-stubs).
// Each routes a dispatch call-site to a no-op/sentinel so the bState scaffold
// lands faithfully; the real body lives in a per-type NPC_AI_*.c file that has
// no .rs yet (or whose dispatch keystone is still a stub). REVISIT: delete each
// stub and repoint its caller when the real fn lands (see crate/DEVIATIONS.md).
// ===========================================================================

// --- Generic bState leaves (NPC_AI_Default.c / NPC_behavior.c). ---
// NPC_BSFollowLeader + NPC_BSSearch dispatch to the real bodies in npc_behavior.rs
// (imported above, c100); NPC_BSJump/NPC_BSNoClip likewise, and NPC_BSDefault from
// npc_ai_default.rs — all now wired to their real implementations.
use crate::codemp::game::npc_ai_default::NPC_BSDefault;
use crate::codemp::game::npc_behavior::{NPC_BSJump, NPC_BSNoClip};

// All per-type bState `_Default` leaves (Droid/Rancor/Sentry/ImperialProbe/Remote/
// Grenadier/MineMonster/Howler/ST + Seeker/Sniper/Jedi/Wampa) are ported and imported above.

// --- Helpers (NPC_AI_BobaFett.c, not yet ported). ---
/// `qboolean Boba_Flying( gentity_t *self )` (NPC_AI_BobaFett.c) — sentinel `qfalse`.
/// REVISIT: un-stub when the Boba Fett AI lands.
unsafe fn Boba_Flying(_self_: *mut gentity_t) -> qboolean {
    QFALSE
}

/// `void NPC_HandleAIFlags( void )` (NPC.c:740) — per-think AI-flag/timer housekeeping:
/// on `NPCAI_LOST` clears the flag and (if the lost goal is the enemy) hands off to
/// `NPC_LostEnemyDecideChase`; fires a delayed victory voice event; decays the
/// friendly-fire counter; and resets a long-stuck patch-nav block counter. The
/// `NPCAI_GREET_ALLIES` greeting block is commented out in the original (kept as a
/// comment). Reads the file-globals `NPC`/`NPCInfo`.
///
/// # Safety
/// `NPC`/`NPCInfo` must point at the current think entity.
pub unsafe fn NPC_HandleAIFlags() {
    //FIXME: make these flags checks a function call like NPC_CheckAIFlagsAndTimers
    if (*NPCInfo).aiFlags & NPCAI_LOST != 0 {
        // Print that you need help!
        //FIXME: shouldn't remove this just yet if cg_draw needs it
        (*NPCInfo).aiFlags &= !NPCAI_LOST;

        if !(*NPCInfo).goalEntity.is_null() && (*NPCInfo).goalEntity == (*NPC).enemy {
            // We can't nav to our enemy
            // Drop enemy and see if we should search for him
            crate::codemp::game::npc_ai_default::NPC_LostEnemyDecideChase();
        }
    }

    //MRJ Request:
    // (the NPCAI_GREET_ALLIES greeting block is commented out in the original)

    //been told to play a victory sound after a delay
    if (*NPCInfo).greetingDebounceTime != 0
        && (*NPCInfo).greetingDebounceTime < (*addr_of!(level)).time
    {
        G_AddVoiceEvent(NPC, Q_irand(EV_VICTORY1, EV_VICTORY3), Q_irand(2000, 4000));
        (*NPCInfo).greetingDebounceTime = 0;
    }

    if (*NPCInfo).ffireCount > 0 {
        if (*NPCInfo).ffireFadeDebounce < (*addr_of!(level)).time {
            (*NPCInfo).ffireCount -= 1;
            //Com_Printf( "drop: %d < %d\n", NPCInfo->ffireCount, 3+((2-g_spskill.integer)*2) );
            (*NPCInfo).ffireFadeDebounce = (*addr_of!(level)).time + 3000;
        }
    }
    if (*addr_of!(d_patched)).integer != 0 {
        // use patch-style navigation
        if (*NPCInfo).consecutiveBlockedMoves > 20 {
            // been stuck for a while, try again?
            (*NPCInfo).consecutiveBlockedMoves = 0;
        }
    }
}

// ===========================================================================
// bState behavior-set dispatchers (NPC.c:941-1386). Thin per-class routers that
// pick the leaf bState handler, all wired to their ported per-type `_Default` leaves.
// ===========================================================================

/// `void NPC_BehaviorSet_Charmed( int bState )` (NPC.c:941).
pub unsafe fn NPC_BehaviorSet_Charmed(b_state: c_int) {
    match b_state {
        BS_FOLLOW_LEADER => NPC_BSFollowLeader(),
        BS_REMOVE => NPC_BSRemove(),
        BS_SEARCH => NPC_BSSearch(),
        BS_WANDER => NPC_BSWander(),
        BS_FLEE => NPC_BSFlee(),
        // default + BS_DEFAULT
        _ => NPC_BSDefault(),
    }
}

/// `void NPC_BehaviorSet_Default( int bState )` (NPC.c:972).
pub unsafe fn NPC_BehaviorSet_Default(b_state: c_int) {
    match b_state {
        BS_ADVANCE_FIGHT => NPC_BSAdvanceFight(),
        BS_SLEEP => NPC_BSSleep(),
        BS_FOLLOW_LEADER => NPC_BSFollowLeader(),
        BS_JUMP => NPC_BSJump(),
        BS_REMOVE => NPC_BSRemove(),
        BS_SEARCH => NPC_BSSearch(),
        BS_NOCLIP => NPC_BSNoClip(),
        BS_WANDER => NPC_BSWander(),
        BS_FLEE => NPC_BSFlee(),
        BS_WAIT => NPC_BSWait(),
        BS_CINEMATIC => NPC_BSCinematic(),
        // default + BS_DEFAULT
        _ => NPC_BSDefault(),
    }
}

/// `void NPC_BehaviorSet_ImperialProbe( int bState )` (NPC.c:1049).
pub unsafe fn NPC_BehaviorSet_ImperialProbe(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSImperialProbe_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Seeker( int bState )` (NPC.c:1074).
pub unsafe fn NPC_BehaviorSet_Seeker(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSSeeker_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Remote( int bState )` (NPC.c:1098) — always the default handler.
pub unsafe fn NPC_BehaviorSet_Remote(_b_state: c_int) {
    NPC_BSRemote_Default();
}

/// `void NPC_BehaviorSet_Sentry( int bState )` (NPC.c:1110).
pub unsafe fn NPC_BehaviorSet_Sentry(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSSentry_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Grenadier( int bState )` (NPC.c:1132).
pub unsafe fn NPC_BehaviorSet_Grenadier(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSGrenadier_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Sniper( int bState )` (NPC.c:1154).
pub unsafe fn NPC_BehaviorSet_Sniper(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSSniper_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Stormtrooper( int bState )` (NPC.c:1177).
pub unsafe fn NPC_BehaviorSet_Stormtrooper(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSST_Default()
        }
        BS_INVESTIGATE => NPC_BSST_Investigate(),
        BS_SLEEP => NPC_BSST_Sleep(),
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Jedi( int bState )` (NPC.c:1209).
pub unsafe fn NPC_BehaviorSet_Jedi(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSJedi_Default()
        }
        BS_FOLLOW_LEADER => NPC_BSJedi_FollowLeader(),
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Droid( int bState )` (NPC.c:1236).
pub unsafe fn NPC_BehaviorSet_Droid(b_state: c_int) {
    match b_state {
        BS_DEFAULT | BS_STAND_GUARD | BS_PATROL => NPC_BSDroid_Default(),
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Mark1( int bState )` (NPC.c:1252).
pub unsafe fn NPC_BehaviorSet_Mark1(b_state: c_int) {
    match b_state {
        BS_DEFAULT | BS_STAND_GUARD | BS_PATROL => NPC_BSMark1_Default(),
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Mark2( int bState )` (NPC.c:1272).
pub unsafe fn NPC_BehaviorSet_Mark2(b_state: c_int) {
    match b_state {
        BS_DEFAULT | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL => NPC_BSMark2_Default(),
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_ATST( int bState )` (NPC.c:1293).
pub unsafe fn NPC_BehaviorSet_ATST(b_state: c_int) {
    match b_state {
        BS_DEFAULT | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL => NPC_BSATST_Default(),
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Interrogator( int bState )` (NPC.c:1019).
pub unsafe fn NPC_BehaviorSet_Interrogator(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSInterrogator_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_MineMonster( int bState )` (NPC.c:1327).
pub unsafe fn NPC_BehaviorSet_MineMonster(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSMineMonster_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Howler( int bState )` (NPC.c:1349).
pub unsafe fn NPC_BehaviorSet_Howler(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSHowler_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_BehaviorSet_Rancor( int bState )` (NPC.c:1371).
pub unsafe fn NPC_BehaviorSet_Rancor(b_state: c_int) {
    match b_state {
        BS_STAND_GUARD | BS_PATROL | BS_STAND_AND_SHOOT | BS_HUNT_AND_KILL | BS_DEFAULT => {
            NPC_BSRancor_Default()
        }
        _ => NPC_BehaviorSet_Default(b_state),
    }
}

/// `void NPC_RunBehavior( int team, int bState )` (NPC.c:1397) — the bState router:
/// picks the per-class behavior set from weapon / NPC_class / team and runs it.
/// Vehicles with a `m_pVehicle` skip AI entirely. `dontSetAim` is set in several
/// branches but never read in this MP build (vestigial in the original).
///
/// # Safety
/// `NPC`/`NPCInfo` must point at the current think entity (non-null `client`).
#[allow(unused_assignments)] // `dontSetAim` is write-only in the original
pub unsafe fn NPC_RunBehavior(team: c_int, b_state: c_int) {
    let mut _dont_set_aim: qboolean = QFALSE;

    if (*NPC).s.NPC_class == CLASS_VEHICLE && !(*NPC).m_pVehicle.is_null() {
        // vehicles don't do AI!
        return;
    }

    if b_state == BS_CINEMATIC {
        NPC_BSCinematic();
    } else if (*(*NPC).client).ps.weapon == WP_EMPLACED_GUN {
        NPC_BSEmplaced();
        NPC_CheckCharmed();
        return;
    } else if (*(*NPC).client).ps.weapon == WP_SABER {
        // jedi
        NPC_BehaviorSet_Jedi(b_state);
        _dont_set_aim = QTRUE;
    } else if (*(*NPC).client).NPC_class == CLASS_WAMPA {
        // wampa
        NPC_BSWampa_Default();
    } else if (*(*NPC).client).NPC_class == CLASS_RANCOR {
        // rancor
        NPC_BehaviorSet_Rancor(b_state);
    } else if (*(*NPC).client).NPC_class == CLASS_REMOTE {
        NPC_BehaviorSet_Remote(b_state);
    } else if (*(*NPC).client).NPC_class == CLASS_SEEKER {
        NPC_BehaviorSet_Seeker(b_state);
    } else if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
        // bounty hunter
        if Boba_Flying(NPC) != QFALSE {
            NPC_BehaviorSet_Seeker(b_state);
        } else {
            NPC_BehaviorSet_Jedi(b_state);
        }
        _dont_set_aim = QTRUE;
    } else if (*NPCInfo).scriptFlags & SCF_FORCED_MARCH != 0 {
        // being forced to march
        NPC_BSDefault();
    } else {
        match team {
            NPCTEAM_ENEMY => {
                // special cases for enemy droids
                match (*(*NPC).client).NPC_class {
                    CLASS_ATST => {
                        NPC_BehaviorSet_ATST(b_state);
                        return;
                    }
                    CLASS_PROBE => {
                        NPC_BehaviorSet_ImperialProbe(b_state);
                        return;
                    }
                    CLASS_REMOTE => {
                        NPC_BehaviorSet_Remote(b_state);
                        return;
                    }
                    CLASS_SENTRY => {
                        NPC_BehaviorSet_Sentry(b_state);
                        return;
                    }
                    CLASS_INTERROGATOR => {
                        NPC_BehaviorSet_Interrogator(b_state);
                        return;
                    }
                    CLASS_MINEMONSTER => {
                        NPC_BehaviorSet_MineMonster(b_state);
                        return;
                    }
                    CLASS_HOWLER => {
                        NPC_BehaviorSet_Howler(b_state);
                        return;
                    }
                    CLASS_MARK1 => {
                        NPC_BehaviorSet_Mark1(b_state);
                        return;
                    }
                    CLASS_MARK2 => {
                        NPC_BehaviorSet_Mark2(b_state);
                        return;
                    }
                    CLASS_GALAKMECH => {
                        NPC_BSGM_Default();
                        return;
                    }
                    _ => {}
                }

                if !(*NPC).enemy.is_null()
                    && (*NPC).s.weapon as c_int == WP_NONE
                    && b_state != BS_HUNT_AND_KILL
                    && trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE
                {
                    // if in battle and have no weapon, run away, fixme: when in BS_HUNT_AND_KILL, they just stand there
                    if b_state != BS_FLEE {
                        NPC_StartFlee(
                            (*NPC).enemy,
                            &(*(*NPC).enemy).r.currentOrigin,
                            AEL_DANGER_GREAT,
                            5000,
                            10000,
                        );
                    } else {
                        NPC_BSFlee();
                    }
                    return;
                }
                if (*(*NPC).client).ps.weapon == WP_SABER {
                    // special melee exception
                    NPC_BehaviorSet_Default(b_state);
                    return;
                }
                if (*(*NPC).client).ps.weapon == WP_DISRUPTOR
                    && (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0
                {
                    // a sniper
                    NPC_BehaviorSet_Sniper(b_state);
                    return;
                }
                if (*(*NPC).client).ps.weapon == WP_THERMAL
                    || (*(*NPC).client).ps.weapon == WP_STUN_BATON
                {
                    // a grenadier
                    NPC_BehaviorSet_Grenadier(b_state);
                    return;
                }
                if NPC_CheckSurrender() != QFALSE {
                    return;
                }
                NPC_BehaviorSet_Stormtrooper(b_state);
            }
            NPCTEAM_NEUTRAL => {
                // special cases for enemy droids
                if (*(*NPC).client).NPC_class == CLASS_PROTOCOL
                    || (*(*NPC).client).NPC_class == CLASS_UGNAUGHT
                    || (*(*NPC).client).NPC_class == CLASS_JAWA
                {
                    NPC_BehaviorSet_Default(b_state);
                } else if (*(*NPC).client).NPC_class == CLASS_VEHICLE {
                    // TODO: Add vehicle behaviors here.
                    NPC_UpdateAngles(QTRUE, QTRUE); // just face our spawn angles for now
                } else {
                    // Just one of the average droids
                    NPC_BehaviorSet_Droid(b_state);
                }
            }
            _ => {
                if (*(*NPC).client).NPC_class == CLASS_SEEKER {
                    NPC_BehaviorSet_Seeker(b_state);
                } else {
                    if (*NPCInfo).charmedTime > (*addr_of!(level)).time {
                        NPC_BehaviorSet_Charmed(b_state);
                    } else {
                        NPC_BehaviorSet_Default(b_state);
                    }
                    NPC_CheckCharmed();
                    _dont_set_aim = QTRUE;
                }
            }
        }
    }
}

/// `void NPC_ExecuteBState( gentity_t *self )` (NPC.c:1593) — runs one NPC behaviour-state
/// think: AI-flag housekeeping, delayed-script firing, bState selection (temp overrides
/// default), `NPC_RunBehavior`, then post-bState cleanup — drop a freed enemy, set the
/// look target, gate firing on `FL_DONT_SHOOT`/surrender, weapon-ready anim/weaponstate,
/// script flags, wall avoidance, save the ucmd and run a `ClientThink` (or apply a roff).
///
/// # Safety
/// `self_`/the file-globals `NPC`/`NPCInfo`/`client` must point at the current think
/// entity (`self_ == NPC`).
pub unsafe fn NPC_ExecuteBState(self_: *mut gentity_t) {
    let b_state: c_int;

    NPC_HandleAIFlags();

    //FIXME: these next three bits could be a function call, some sort of setup/cleanup func
    //Lookmode must be reset every think cycle
    if (*NPC).delayScriptTime != 0 && (*NPC).delayScriptTime <= (*addr_of!(level)).time {
        G_ActivateBehavior(NPC, BSET_DELAYED);
        (*NPC).delayScriptTime = 0;
    }

    //Clear this and let bState set it itself, so it automatically handles changing bStates... but we need a set bState wrapper func
    (*NPCInfo).combatMove = QFALSE;

    //Execute our bState
    if (*NPCInfo).tempBehavior != 0 {
        // Overrides normal behavior until cleared
        b_state = (*NPCInfo).tempBehavior;
    } else {
        if (*NPCInfo).behaviorState == 0 {
            (*NPCInfo).behaviorState = (*NPCInfo).defaultBehavior;
        }
        b_state = (*NPCInfo).behaviorState;
    }

    //Pick the proper bstate for us and run it
    NPC_RunBehavior((*(*self_).client).playerTeam, b_state);

    //Here we need to see what the scripted stuff told us to do
    //Back to normal?  All decisions made?

    if !(*NPC).enemy.is_null() {
        if (*(*NPC).enemy).inuse == QFALSE {
            // just in case bState doesn't catch this
            G_ClearEnemy(NPC);
        }
    }

    if (*client).ps.saberLockTime != 0 && (*client).ps.saberLockEnemy != ENTITYNUM_NONE {
        NPC_SetLookTarget(
            NPC,
            (*client).ps.saberLockEnemy,
            (*addr_of!(level)).time + 1000,
        );
    } else if NPC_CheckLookTarget(NPC) == QFALSE {
        if !(*NPC).enemy.is_null() {
            NPC_SetLookTarget(NPC, (*(*NPC).enemy).s.number, 0);
        }
    }

    if !(*NPC).enemy.is_null() {
        if (*(*NPC).enemy).flags & FL_DONT_SHOOT != 0 {
            ucmd.buttons &= !BUTTON_ATTACK;
            ucmd.buttons &= !BUTTON_ALT_ATTACK;
        } else if (*(*NPC).client).playerTeam != NPCTEAM_ENEMY
            && !(*(*NPC).enemy).NPC.is_null()
            && ((*(*(*NPC).enemy).NPC).surrenderTime > (*addr_of!(level)).time
                || (*(*(*NPC).enemy).NPC).scriptFlags & SCF_FORCED_MARCH != 0)
        {
            // don't shoot someone who's surrendering if you're a good guy
            ucmd.buttons &= !BUTTON_ATTACK;
            ucmd.buttons &= !BUTTON_ALT_ATTACK;
        }

        if (*client).ps.weaponstate == WEAPON_IDLE {
            (*client).ps.weaponstate = WEAPON_READY;
        }
    } else {
        if (*client).ps.weaponstate == WEAPON_READY {
            (*client).ps.weaponstate = WEAPON_IDLE;
        }
    }

    if ucmd.buttons & BUTTON_ATTACK == 0 && (*NPC).attackDebounceTime > (*addr_of!(level)).time {
        // We just shot but aren't still shooting, so hold the gun up for a while
        if (*client).ps.weapon == WP_SABER {
            // One-handed
            NPC_SetAnim(NPC, SETANIM_TORSO, TORSO_WEAPONREADY1, SETANIM_FLAG_NORMAL);
        } else if (*client).ps.weapon == WP_BRYAR_PISTOL {
            // Sniper pose
            NPC_SetAnim(NPC, SETANIM_TORSO, TORSO_WEAPONREADY3, SETANIM_FLAG_NORMAL);
        }
    } else if (*NPC).enemy.is_null() {
        // HACK!
        if (*NPC).s.torsoAnim == TORSO_WEAPONREADY1 || (*NPC).s.torsoAnim == TORSO_WEAPONREADY3 {
            // we look ready for action, using one of the first 2 weapon, let's rest our weapon on our shoulder
            NPC_SetAnim(NPC, SETANIM_TORSO, TORSO_WEAPONIDLE3, SETANIM_FLAG_NORMAL);
        }
    }

    NPC_CheckAttackHold();
    NPC_ApplyScriptFlags();

    //cliff and wall avoidance
    NPC_AvoidWallsAndCliffs();

    // run the bot through the server like it was a real client
    //=== Save the ucmd for the second no-think Pmove ============================
    ucmd.serverTime = (*addr_of!(level)).time - 50;
    (*NPCInfo).last_ucmd = ucmd; // memcpy( &NPCInfo->last_ucmd, &ucmd, sizeof( usercmd_t ) )
    if (*NPCInfo).attackHoldTime == 0 {
        // so we don't fire twice in one think
        (*NPCInfo).last_ucmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
    }
    //============================================================================
    NPC_CheckAttackScript();
    NPC_KeepCurrentFacing();

    if (*NPC).next_roff_time == 0 || (*NPC).next_roff_time < (*addr_of!(level)).time {
        // If we were following a roff, we don't do normal pmoves.
        ClientThink((*NPC).s.number, addr_of_mut!(ucmd));
    } else {
        NPC_ApplyRoff();
    }

    // end of thinking cleanup
    (*NPCInfo).touchedByPlayer = core::ptr::null_mut();

    NPC_CheckPlayerAim();
    NPC_CheckAllClear();
}

/// `void NPC_Think( gentity_t *self )` (NPC.c:1843) — the per-frame NPC AI entry (the
/// `gentity_t::think` for live NPCs). Sets the NPC globals, clears the move dir, routes
/// dead NPCs to `DeadThink`, honours the ICARUS freeze, throttles bState thinking
/// (`nextBStateThink`, faster for hard-skill Jedi), and otherwise runs `NPC_ExecuteBState`
/// — finally pumping the ICARUS task manager every frame and syncing `ps.origin`.
///
/// `pub unsafe extern "C" fn` for the `gentity_t::think` fn-pointer ABI.
///
/// # Safety
/// `self_` must be a valid NPC entity with non-null `client`/`NPC`.
pub unsafe extern "C" fn NPC_Think(self_: *mut gentity_t) {
    let mut old_move_dir: vec3_t = [0.0; 3];
    let mut i: c_int = 0;
    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;

    SetNPCGlobals(self_);

    ucmd = usercmd_t::default(); // memset( &ucmd, 0, sizeof( ucmd ) )

    VectorCopy(&(*(*self_).client).ps.moveDir, &mut old_move_dir);
    if (*self_).s.NPC_class != CLASS_VEHICLE {
        // YOU ARE BREAKING MY PREDICTION. Bad clear.
        (*(*self_).client).ps.moveDir = [0.0; 3]; // VectorClear
    }

    if self_.is_null() || (*self_).NPC.is_null() || (*self_).client.is_null() {
        return;
    }

    // dead NPCs have a special think, don't run scripts (for now)
    //FIXME: this breaks deathscripts
    if (*self_).health <= 0 {
        DeadThink();
        if (*NPCInfo).nextBStateThink <= (*addr_of!(level)).time {
            trap::ICARUS_MaintainTaskManager((*self_).s.number);
        }
        (*(*self_).client).ps.origin = (*self_).r.currentOrigin;
        return;
    }

    // see if NPC ai is frozen
    if (*addr_of!(debugNPCFreeze)).value != 0.0 || (*NPC).r.svFlags & SVF_ICARUS_FREEZE != 0 {
        NPC_UpdateAngles(QTRUE, QTRUE);
        ClientThink((*self_).s.number, addr_of_mut!(ucmd));
        (*(*self_).client).ps.origin = (*self_).r.currentOrigin;
        return;
    }

    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME / 2;

    while i < MAX_CLIENTS as c_int {
        let player: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize);

        if (*player).inuse != QFALSE
            && !(*player).client.is_null()
            && (*(*player).client).sess.sessionTeam != TEAM_SPECTATOR
            && (*(*player).client).ps.pm_flags & PMF_FOLLOW == 0
        {
            //if ( player->client->ps.viewEntity == self->s.number )
            if false {
                // rwwFIXMEFIXME: Allow controlling ents
                // being controlled by player
                G_DroidSounds(self_);
                (*NPCInfo).last_ucmd.serverTime = (*addr_of!(level)).time - 50;
                ClientThink((*NPC).s.number, addr_of_mut!(ucmd));
                (*(*self_).client).ps.origin = (*self_).r.currentOrigin;
                return;
            }
        }
        i += 1;
    }

    if (*(*self_).client).NPC_class == CLASS_VEHICLE {
        if (*(*self_).client).ps.m_iVehicleNum != 0 {
            // we don't think on our own
            // well, run scripts, though...
            trap::ICARUS_MaintainTaskManager((*self_).s.number);
            return;
        } else {
            (*(*self_).client).ps.moveDir = [0.0; 3]; // VectorClear
            (*(*self_).client).pers.cmd.forwardmove = 0;
            (*(*self_).client).pers.cmd.rightmove = 0;
            (*(*self_).client).pers.cmd.upmove = 0;
            (*(*self_).client).pers.cmd.buttons = 0;
            (*(*self_).m_pVehicle).m_ucmd = (*(*self_).client).pers.cmd;
        }
    } else if (*NPC).s.m_iVehicleNum != 0 {
        // droid in a vehicle?
        G_DroidSounds(self_);
    }

    if (*NPCInfo).nextBStateThink <= (*addr_of!(level)).time && (*NPC).s.m_iVehicleNum == 0 {
        // NPCs sitting in Vehicles do NOTHING
        if (*NPC).s.eType != ET_NPC {
            // Something drastic happened in our script
            return;
        }

        if (*NPC).s.weapon as c_int == WP_SABER
            && (*addr_of!(g_spskill)).integer >= 2
            && (*NPCInfo).rank > RANK_LT_JG
        {
            // Jedi think faster on hard difficulty, except low-rank (reborn)
            (*NPCInfo).nextBStateThink = (*addr_of!(level)).time + FRAMETIME / 2;
        } else {
            // Maybe even 200 ms?
            (*NPCInfo).nextBStateThink = (*addr_of!(level)).time + FRAMETIME;
        }

        //nextthink is set before this so something in here can override it
        if (*self_).s.NPC_class != CLASS_VEHICLE || (*self_).m_pVehicle.is_null() {
            // ok, let's not do this at all for vehicles.
            NPC_ExecuteBState(self_);
        }
    } else {
        VectorCopy(&old_move_dir, &mut (*(*self_).client).ps.moveDir);
        //or use client->pers.lastCommand?
        (*NPCInfo).last_ucmd.serverTime = (*addr_of!(level)).time - 50;
        if (*NPC).next_roff_time == 0 || (*NPC).next_roff_time < (*addr_of!(level)).time {
            // If we were following a roff, we don't do normal pmoves.
            //FIXME: firing angles (no aim offset) or regular angles?
            NPC_UpdateAngles(QTRUE, QTRUE);
            ucmd = (*NPCInfo).last_ucmd; // memcpy( &ucmd, &NPCInfo->last_ucmd, sizeof( usercmd_t ) )
            ClientThink((*NPC).s.number, addr_of_mut!(ucmd));
        } else {
            NPC_ApplyRoff();
        }
    }
    //must update icarus *every* frame because of certain animation completions in the pmove stuff that can leave a 50ms gap between ICARUS animation commands
    trap::ICARUS_MaintainTaskManager((*self_).s.number);
    (*(*self_).client).ps.origin = (*self_).r.currentOrigin;
}
