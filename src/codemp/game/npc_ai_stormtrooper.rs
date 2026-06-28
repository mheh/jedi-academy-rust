//! Slice of `NPC_AI_Stormtrooper.c` — the stormtrooper NPC's behavior state.
//! Opened bottom-up at the leaf seam: the aggression/timer helpers, the speech
//! dispatch, the pain handler, the movement-speech store/say pair, the
//! mark-to-cover reaction and the timer-transfer helper are genuinely portable
//! today, while the deep `BS*_*` / `ST_Move` / `ST_Commander` /
//! `ST_CheckFireState` chain still routes through the NPC-AI core and NAV
//! (`NPC_MoveToGoal`, `UpdateGoal`, `NPC_GetMoveDirection`), the flee subsystem
//! (`G_StartFlee`), ICARUS (`trap_ICARUS_TaskIDPending`) and squad-group AI —
//! not yet ported.
//!
//! Ported here so far: `ST_AggressionAdjust` (NPC_AI_Stormtrooper.c:60),
//! `ST_ClearTimers` (:88), `ST_Speech` (:124), `ST_MarkToCover` (:227),
//! `NPC_ST_Pain` (:260), `NPC_ST_SayMovementSpeech` (:304),
//! `NPC_ST_StoreMovementSpeech` (:327), `ST_TransferTimers` (:1579).
//!
//! Still blocked (flee / NAV / ICARUS / squad-group AI): `ST_StartFlee`,
//! `ST_HoldPosition`, `ST_Move`, `ST_TransferMoveGoal`, `NPC_ST_SleepShuffle`,
//! `NPC_BSST_Sleep`, `NPC_CheckEnemyStealth`, `NPC_CheckPlayerTeamStealth`,
//! `NPC_ST_InvestigateEvent`, `ST_OffsetLook`, `ST_LookAround`,
//! `NPC_BSST_Investigate`, `NPC_BSST_Patrol`, `NPC_BSST_Idle`,
//! `ST_CheckMoveState`, `ST_ResolveBlockedShot`, `ST_CheckFireState`,
//! `ST_TrackEnemy`, `ST_ApproachEnemy`, `ST_HuntEnemy`, `ST_GetCPFlags`,
//! `ST_Commander`, `NPC_BSST_Attack`, `NPC_BSST_Default`.

#![allow(non_snake_case)] // C function names (`ST_AggressionAdjust`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define`/enum constants kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::ai_h::{
    AIGroupInfo_t, SQUAD_COVER, SQUAD_IDLE, SQUAD_POINT, SQUAD_RETREAT, SQUAD_SCOUT,
    SQUAD_STAND_AND_SHOOT, SQUAD_TRANSITION,
};
use crate::codemp::game::anims::BOTH_STAND4;
use crate::codemp::game::b_local_h::{
    CPF_DUCK, CP_ANY, CP_APPROACH_ENEMY, CP_AVOID, CP_AVOID_ENEMY, CP_CLEAR, CP_CLOSEST, CP_COVER,
    CP_DUCK, CP_FLANK, CP_FLEE, CP_HAS_ROUTE, CP_INVESTIGATE, CP_NEAREST, CP_RETREAT, CP_SAFE,
    CP_SQUAD, MIN_ROCKET_DIST_SQUARED,
};
use crate::codemp::game::b_public_h::{
    BS_DEFAULT, BS_INVESTIGATE, BS_SEARCH, NPCAI_BLOCKED, RANK_ENSIGN, RANK_LT, SCF_ALT_FIRE,
    SCF_CHASE_ENEMIES, SCF_DONT_FIRE, SCF_FIRE_WEAPON, SCF_IGNORE_ALERTS, SCF_LOOK_FOR_ENEMIES,
    SCF_RUNNING, SCF_USE_CP_NEAREST, SPOT_HEAD,
};
use crate::codemp::game::bg_public::{
    ET_ITEM, EV_ANGER1, EV_ANGER3, EV_CHASE1, EV_CHASE3, EV_CONFUSE1, EV_CONFUSE3, EV_COVER1,
    EV_COVER5, EV_DETECTED1, EV_DETECTED5, EV_ESCAPING1, EV_ESCAPING3, EV_GIVEUP1, EV_GIVEUP4,
    EV_LOOK1, EV_LOOK2, EV_LOST1, EV_OUTFLANK1, EV_OUTFLANK2, EV_PUSHED1, EV_PUSHED3, EV_SIGHT1,
    EV_SIGHT3, EV_SOUND1, EV_SOUND3, EV_SUSPICIOUS1, EV_SUSPICIOUS5, MASK_SHOT, PMF_DUCKED,
    SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, SETANIM_TORSO, TEAM_NUM_TEAMS,
    WEAPON_READY,
};
use crate::codemp::game::bg_weapons_h::{
    WP_DET_PACK, WP_DISRUPTOR, WP_EMPLACED_GUN, WP_FLECHETTE, WP_NONE, WP_REPEATER,
    WP_ROCKET_LAUNCHER, WP_SABER, WP_THERMAL, WP_TRIP_MINE,
};
use crate::codemp::game::g_local::{
    gentity_t, AEL_DANGER, AEL_DANGER_GREAT, AEL_DISCOVERED, AEL_MINOR, AET_SIGHT, AET_SOUND,
    FL_NAVGOAL, FL_NOTARGET,
};
use crate::codemp::game::g_main::{d_asynchronousGroupAI, g_entities, g_spskill, level};
use crate::codemp::game::g_nav::{
    navInfo_t, FlyingCreature, NAV_FindClosestWaypointForEnt, NAV_HitNavGoal, NPC_SetMoveGoal,
    NIF_COLLISION, WAYPOINT_NONE,
};
use crate::codemp::game::g_public_h::{BSET_AWAKE, Q3_INFINITE, SVF_GLASS_BRUSH, TID_MOVE_NAV};
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Get, TIMER_Set};
use crate::codemp::game::g_utils::{G_ExpandPointToBBox, GetAnglesForDirection};
use crate::codemp::game::npc::{
    ucmd, NPCInfo, NPC_SetAnim, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals, NPC,
};
use crate::codemp::game::npc_ai_default::NPC_BSPatrol;
use crate::codemp::game::npc_ai_utils::{
    AI_GetGroup, AI_GroupContainsEntNum, AI_GroupUpdateClearShotTime, AI_GroupUpdateEnemyLastSeen,
    AI_GroupUpdateSquadstates,
};
use crate::codemp::game::npc_behavior::{G_StartFlee, NPC_BSSearchStart, NPC_StartFlee};
use crate::codemp::game::npc_combat::{
    ChangeWeapon, G_AddVoiceEvent, G_ClearEnemy, G_SetEnemy, NPC_AimAdjust, NPC_ChangeWeapon,
    NPC_CheckGetNewWeapon, NPC_FindCombatPoint, NPC_FreeCombatPoint, NPC_SetCombatPoint,
    NPC_ShotEntity, WeaponThink,
};
use crate::codemp::game::npc_goal::{NPC_ReachedGoal, UpdateGoal};
use crate::codemp::game::npc_move::{NAV_GetLastMove, NPC_MoveToGoal};
use crate::codemp::game::npc_reactions::{NPC_Pain, NPC_TempLookTarget};
use crate::codemp::game::npc_senses::{
    InFOV, NPC_CheckAlertEvents, NPC_CheckForDanger, NPC_GetHFOVPercentage, NPC_GetVFOVPercentage,
};
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, G_ActivateBehavior, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy,
    NPC_FacePosition, NPC_UpdateAngles, NPC_ValidEnemy,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleVectors, Distance, DistanceSquared, DotProduct, VectorClear,
    VectorCompare, VectorCopy, VectorLength, VectorMA, VectorNormalize, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::random;
use crate::codemp::game::q_shared_h::{
    trace_t, vec3_t, BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_WALKING, ENTITYNUM_NONE,
    ENTITYNUM_WORLD, PITCH, YAW,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_BOTCLIP, CONTENTS_FOG, CONTENTS_WATER,
};
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_IMPERIAL, CLASS_IMPWORKER, CLASS_SWAMPTROOPER, NPCTEAM_PLAYER,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// used to stop several group AI from speaking all at once
static mut groupSpeechDebounceTime: [c_int; TEAM_NUM_TEAMS as usize] = [0; TEAM_NUM_TEAMS as usize];

// File-scope shared state for the NPC_BSST_Attack AI loop (set by the orchestrators,
// read by ST_CheckMoveState / ST_CheckFireState / etc.).
static mut enemyLOS: qboolean = QFALSE;
static mut enemyCS: qboolean = QFALSE;
static mut enemyInFOV: qboolean = QFALSE;
static mut hitAlly: qboolean = QFALSE;
static mut faceEnemy: qboolean = QFALSE;
static mut r#move: qboolean = QFALSE;
static mut shoot: qboolean = QFALSE;
static mut enemyDist: f32 = 0.0;
static mut impactPos: vec3_t = [0.0; 3];

// Local state enums
#[allow(dead_code)]
const LSTATE_NONE: c_int = 0;
const LSTATE_UNDERFIRE: c_int = LSTATE_NONE + 1;
#[allow(dead_code)]
const LSTATE_INVESTIGATE: c_int = LSTATE_UNDERFIRE + 1;

const MAX_VIEW_DIST: c_int = 1024;
const MAX_VIEW_SPEED: c_int = 250;
const MAX_LIGHT_INTENSITY: c_int = 255;
const MIN_LIGHT_THRESHOLD: f32 = 0.1;
const ST_MIN_LIGHT_THRESHOLD: c_int = 30;
const ST_MAX_LIGHT_THRESHOLD: c_int = 180;
const DISTANCE_THRESHOLD: f32 = 0.075f32;

const DISTANCE_SCALE: f32 = 0.35f32; //These first three get your base detection rating, ideally add up to 1
const FOV_SCALE: f32 = 0.40f32; //
const LIGHT_SCALE: f32 = 0.25f32; //

const SPEED_SCALE: f32 = 0.25f32; //These next two are bonuses
const TURNING_SCALE: f32 = 0.25f32; //

const REALIZE_THRESHOLD: f32 = 0.6f32;
const CAUTIOUS_THRESHOLD: f32 = REALIZE_THRESHOLD * 0.75; //

pub unsafe fn ST_AggressionAdjust(self_: *mut gentity_t, change: c_int) {
    let upper_threshold: c_int;
    let lower_threshold: c_int;

    (*(*self_).NPC).stats.aggression += change;

    //FIXME: base this on initial NPC stats
    if (*(*self_).client).playerTeam == NPCTEAM_PLAYER {
        //good guys are less aggressive
        upper_threshold = 7;
        lower_threshold = 1;
    } else {
        //bad guys are more aggressive
        upper_threshold = 10;
        lower_threshold = 3;
    }

    if (*(*self_).NPC).stats.aggression > upper_threshold {
        (*(*self_).NPC).stats.aggression = upper_threshold;
    } else if (*(*self_).NPC).stats.aggression < lower_threshold {
        (*(*self_).NPC).stats.aggression = lower_threshold;
    }
}

pub unsafe fn ST_ClearTimers(ent: *mut gentity_t) {
    TIMER_Set(ent, c"chatter".as_ptr(), 0);
    TIMER_Set(ent, c"duck".as_ptr(), 0);
    TIMER_Set(ent, c"stand".as_ptr(), 0);
    TIMER_Set(ent, c"shuffleTime".as_ptr(), 0);
    TIMER_Set(ent, c"sleepTime".as_ptr(), 0);
    TIMER_Set(ent, c"enemyLastVisible".as_ptr(), 0);
    TIMER_Set(ent, c"roamTime".as_ptr(), 0);
    TIMER_Set(ent, c"hideTime".as_ptr(), 0);
    TIMER_Set(ent, c"attackDelay".as_ptr(), 0); //FIXME: Slant for difficulty levels
    TIMER_Set(ent, c"stick".as_ptr(), 0);
    TIMER_Set(ent, c"scoutTime".as_ptr(), 0);
    TIMER_Set(ent, c"flee".as_ptr(), 0);
    TIMER_Set(ent, c"interrogating".as_ptr(), 0);
    TIMER_Set(ent, c"verifyCP".as_ptr(), 0);
}

const SPEECH_CHASE: c_int = 0;
const SPEECH_CONFUSED: c_int = SPEECH_CHASE + 1;
const SPEECH_COVER: c_int = SPEECH_CONFUSED + 1;
const SPEECH_DETECTED: c_int = SPEECH_COVER + 1;
const SPEECH_GIVEUP: c_int = SPEECH_DETECTED + 1;
const SPEECH_LOOK: c_int = SPEECH_GIVEUP + 1;
const SPEECH_LOST: c_int = SPEECH_LOOK + 1;
const SPEECH_OUTFLANK: c_int = SPEECH_LOST + 1;
const SPEECH_ESCAPING: c_int = SPEECH_OUTFLANK + 1;
const SPEECH_SIGHT: c_int = SPEECH_ESCAPING + 1;
const SPEECH_SOUND: c_int = SPEECH_SIGHT + 1;
const SPEECH_SUSPICIOUS: c_int = SPEECH_SOUND + 1;
const SPEECH_YELL: c_int = SPEECH_SUSPICIOUS + 1;
const SPEECH_PUSHED: c_int = SPEECH_YELL + 1;

unsafe fn ST_Speech(self_: *mut gentity_t, speechType: c_int, failChance: f32) {
    if random() < failChance {
        return;
    }

    if failChance >= 0.0 {
        //a negative failChance makes it always talk
        if !(*(*self_).NPC).group.is_null() {
            //group AI speech debounce timer
            if (*(*(*self_).NPC).group).speechDebounceTime > (*addr_of!(level)).time {
                return;
            }
            /*
            else if ( !self->NPC->group->enemy )
            {
                if ( groupSpeechDebounceTime[self->client->playerTeam] > level.time )
                {
                    return;
                }
            }
            */
        } else if TIMER_Done(self_, c"chatter".as_ptr()) == 0 {
            //personal timer
            return;
        } else if groupSpeechDebounceTime[(*(*self_).client).playerTeam as usize]
            > (*addr_of!(level)).time
        {
            //for those not in group AI
            //FIXME: let certain speech types interrupt others?  Let closer NPCs interrupt farther away ones?
            return;
        }
    }

    if !(*(*self_).NPC).group.is_null() {
        //So they don't all speak at once...
        //FIXME: if they're not yet mad, they have no group, so distracting a group of them makes them all speak!
        (*(*(*self_).NPC).group).speechDebounceTime = (*addr_of!(level)).time + Q_irand(2000, 4000);
    } else {
        TIMER_Set(self_, c"chatter".as_ptr(), Q_irand(2000, 4000));
    }
    groupSpeechDebounceTime[(*(*self_).client).playerTeam as usize] =
        (*addr_of!(level)).time + Q_irand(2000, 4000);

    if (*(*self_).NPC).blockedSpeechDebounceTime > (*addr_of!(level)).time {
        return;
    }

    match speechType {
        SPEECH_CHASE => {
            G_AddVoiceEvent(self_, Q_irand(EV_CHASE1, EV_CHASE3), 2000);
        }
        SPEECH_CONFUSED => {
            G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000);
        }
        SPEECH_COVER => {
            G_AddVoiceEvent(self_, Q_irand(EV_COVER1, EV_COVER5), 2000);
        }
        SPEECH_DETECTED => {
            G_AddVoiceEvent(self_, Q_irand(EV_DETECTED1, EV_DETECTED5), 2000);
        }
        SPEECH_GIVEUP => {
            G_AddVoiceEvent(self_, Q_irand(EV_GIVEUP1, EV_GIVEUP4), 2000);
        }
        SPEECH_LOOK => {
            G_AddVoiceEvent(self_, Q_irand(EV_LOOK1, EV_LOOK2), 2000);
        }
        SPEECH_LOST => {
            G_AddVoiceEvent(self_, EV_LOST1, 2000);
        }
        SPEECH_OUTFLANK => {
            G_AddVoiceEvent(self_, Q_irand(EV_OUTFLANK1, EV_OUTFLANK2), 2000);
        }
        SPEECH_ESCAPING => {
            G_AddVoiceEvent(self_, Q_irand(EV_ESCAPING1, EV_ESCAPING3), 2000);
        }
        SPEECH_SIGHT => {
            G_AddVoiceEvent(self_, Q_irand(EV_SIGHT1, EV_SIGHT3), 2000);
        }
        SPEECH_SOUND => {
            G_AddVoiceEvent(self_, Q_irand(EV_SOUND1, EV_SOUND3), 2000);
        }
        SPEECH_SUSPICIOUS => {
            G_AddVoiceEvent(self_, Q_irand(EV_SUSPICIOUS1, EV_SUSPICIOUS5), 2000);
        }
        SPEECH_YELL => {
            G_AddVoiceEvent(self_, Q_irand(EV_ANGER1, EV_ANGER3), 2000);
        }
        SPEECH_PUSHED => {
            G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
        }
        _ => {}
    }

    (*(*self_).NPC).blockedSpeechDebounceTime = (*addr_of!(level)).time + 2000;
}

pub unsafe fn ST_MarkToCover(self_: *mut gentity_t) {
    if self_.is_null() || (*self_).NPC.is_null() {
        return;
    }
    (*(*self_).NPC).localState = LSTATE_UNDERFIRE;
    TIMER_Set(self_, c"attackDelay".as_ptr(), Q_irand(500, 2500));
    ST_AggressionAdjust(self_, -3);
    if !(*(*self_).NPC).group.is_null() && (*(*(*self_).NPC).group).numGroup > 1 {
        ST_Speech(self_, SPEECH_COVER, 0.0); //FIXME: flee sound?
    }
}

pub unsafe fn ST_StartFlee(
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
    dangerPoint: &vec3_t,
    dangerLevel: c_int,
    minTime: c_int,
    maxTime: c_int,
) {
    if self_.is_null() || (*self_).NPC.is_null() {
        return;
    }
    G_StartFlee(self_, enemy, dangerPoint, dangerLevel, minTime, maxTime);
    if !(*(*self_).NPC).group.is_null() && (*(*(*self_).NPC).group).numGroup > 1 {
        ST_Speech(self_, SPEECH_COVER, 0.0); //FIXME: flee sound?
    }
}

#[allow(dead_code)] // static helper; only caller (ST_Move) still blocked on NPC_MoveToGoal / NAV_GetLastMove
unsafe fn ST_HoldPosition() {
    if (*NPCInfo).squadState == SQUAD_RETREAT {
        TIMER_Set(NPC, c"flee".as_ptr(), -(*addr_of!(level)).time);
    }
    TIMER_Set(NPC, c"verifyCP".as_ptr(), Q_irand(1000, 3000)); //don't look for another one for a few seconds
    NPC_FreeCombatPoint((*NPCInfo).combatPoint, QTRUE);
    //NPCInfo->combatPoint = -1;//???
    if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE {
        //don't have a script waiting for me to get to my point, okay to stop trying and stand
        AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, SQUAD_STAND_AND_SHOOT);
        (*NPCInfo).goalEntity = core::ptr::null_mut();
    }

    /*if ( TIMER_Done( NPC, "stand" ) )
    {//FIXME: what if can't shoot from this pos?
        TIMER_Set( NPC, "duck", Q_irand( 2000, 4000 ) );
    }
    */
}

/*
-------------------------
NPC_ST_Pain
-------------------------
*/

pub unsafe extern "C" fn NPC_ST_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    (*(*self_).NPC).localState = LSTATE_UNDERFIRE;

    TIMER_Set(self_, c"duck".as_ptr(), -1);
    TIMER_Set(self_, c"hideTime".as_ptr(), -1);
    TIMER_Set(self_, c"stand".as_ptr(), 2000);

    NPC_Pain(self_, attacker, damage);

    if damage == 0 && (*self_).health > 0 {
        //FIXME: better way to know I was pushed
        G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
    }
}

pub unsafe fn NPC_ST_SayMovementSpeech() {
    if (*NPCInfo).movementSpeech == 0 {
        return;
    }
    if !(*NPCInfo).group.is_null()
        && !(*(*NPCInfo).group).commander.is_null()
        && !(*(*(*NPCInfo).group).commander).client.is_null()
        && (*(*(*(*NPCInfo).group).commander).client).NPC_class == CLASS_IMPERIAL
        && Q_irand(0, 3) == 0
    {
        //imperial (commander) gives the order
        ST_Speech(
            (*(*NPCInfo).group).commander,
            (*NPCInfo).movementSpeech,
            (*NPCInfo).movementSpeechChance,
        );
    } else {
        //really don't want to say this unless we can actually get there...
        ST_Speech(
            NPC,
            (*NPCInfo).movementSpeech,
            (*NPCInfo).movementSpeechChance,
        );
    }

    (*NPCInfo).movementSpeech = 0;
    (*NPCInfo).movementSpeechChance = 0.0f32;
}

pub unsafe fn NPC_ST_StoreMovementSpeech(speech: c_int, chance: f32) {
    (*NPCInfo).movementSpeech = speech;
    (*NPCInfo).movementSpeechChance = chance;
}

pub unsafe fn ST_TransferTimers(self_: *mut gentity_t, other: *mut gentity_t) {
    TIMER_Set(
        other,
        c"attackDelay".as_ptr(),
        TIMER_Get(self_, c"attackDelay".as_ptr()) - (*addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"duck".as_ptr(),
        TIMER_Get(self_, c"duck".as_ptr()) - (*addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"stick".as_ptr(),
        TIMER_Get(self_, c"stick".as_ptr()) - (*addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"scoutTime".as_ptr(),
        TIMER_Get(self_, c"scout".as_ptr()) - (*addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"roamTime".as_ptr(),
        TIMER_Get(self_, c"roamTime".as_ptr()) - (*addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"stand".as_ptr(),
        TIMER_Get(self_, c"stand".as_ptr()) - (*addr_of!(level)).time,
    );
    TIMER_Set(self_, c"attackDelay".as_ptr(), -1);
    TIMER_Set(self_, c"duck".as_ptr(), -1);
    TIMER_Set(self_, c"stick".as_ptr(), -1);
    TIMER_Set(self_, c"scoutTime".as_ptr(), -1);
    TIMER_Set(self_, c"roamTime".as_ptr(), -1);
    TIMER_Set(self_, c"stand".as_ptr(), -1);
}

#[allow(dead_code)] // static helper; only callers (NPC_BSST_* states) still blocked
unsafe fn ST_Move() -> qboolean {
    let moved: qboolean;
    let mut info: navInfo_t = core::mem::zeroed();

    (*NPCInfo).combatMove = QTRUE; //always move straight toward our goal

    moved = NPC_MoveToGoal(QTRUE);

    //Get the move info
    NAV_GetLastMove(&mut info);

    //FIXME: if we bump into another one of our guys and can't get around him, just stop!
    //If we hit our target, then stop and fire!
    if info.flags & NIF_COLLISION != 0 {
        if info.blocker == (*NPC).enemy {
            ST_HoldPosition();
        }
    }

    //If our move failed, then reset
    if moved == QFALSE {
        //FIXME: if we're going to a combat point, need to pick a different one
        if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE {
            //can't transfer movegoal or stop when a script we're running is waiting to complete
            if !info.blocker.is_null()
                && !(*info.blocker).NPC.is_null()
                && !(*NPCInfo).group.is_null()
                && (*(*info.blocker).NPC).group == (*NPCInfo).group
            //(NPCInfo->aiFlags&NPCAI_BLOCKED) && NPCInfo->group != NULL )
            {
                //dammit, something is in our way
                //see if it's one of ours
                let mut j: c_int = 0;

                while j < (*(*NPCInfo).group).numGroup {
                    if (*(*NPCInfo).group).member[j as usize].number == (*NPCInfo).blockingEntNum {
                        //we're being blocked by one of our own, pass our goal onto them and I'll stand still
                        ST_TransferMoveGoal(
                            NPC,
                            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                .add((*(*NPCInfo).group).member[j as usize].number as usize),
                        );
                        break;
                    }
                    j += 1;
                }
            }

            ST_HoldPosition();
        }
    } else {
        //First time you successfully move, say what it is you're doing
        NPC_ST_SayMovementSpeech();
    }

    moved
}

unsafe fn NPC_ST_SleepShuffle() {
    //Play an awake script if we have one
    if G_ActivateBehavior(NPC, BSET_AWAKE) != QFALSE {
        return;
    }

    //Automate some movement and noise
    if TIMER_Done(NPC, c"shuffleTime".as_ptr()) != QFALSE {
        //TODO: Play sleeping shuffle animation

        //int	soundIndex = Q_irand( 0, 1 );

        /*
        switch ( soundIndex )
        {
        case 0:
            G_Sound( NPC, G_SoundIndex("sound/chars/imperialsleeper1/scav4/hunh.mp3") );
            break;

        case 1:
            G_Sound( NPC, G_SoundIndex("sound/chars/imperialsleeper3/scav4/tryingtosleep.wav") );
            break;
        }
        */

        TIMER_Set(NPC, c"shuffleTime".as_ptr(), 4000);
        TIMER_Set(NPC, c"sleepTime".as_ptr(), 2000);
        return;
    }

    //They made another noise while we were stirring, see if we can see them
    if TIMER_Done(NPC, c"sleepTime".as_ptr()) != QFALSE {
        NPC_CheckPlayerTeamStealth();
        TIMER_Set(NPC, c"sleepTime".as_ptr(), 2000);
    }
}

pub unsafe fn NPC_BSST_Sleep() {
    let alertEvent: c_int = NPC_CheckAlertEvents(QFALSE, QTRUE, -1, QFALSE, AEL_MINOR); //only check sounds since we're alseep!

    //There is an event we heard
    if alertEvent >= 0 {
        //See if it was enough to wake us up
        if (*addr_of!(level)).alertEvents[alertEvent as usize].level == AEL_DISCOVERED
            && (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0
        {
            //rwwFIXMEFIXME: Care about all clients not just 0
            // (the C `&g_entities[0]` null-check can never hold — the array is static.)
            if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0)).health > 0 {
                G_SetEnemy(
                    NPC,
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0),
                );
                return;
            }
        }

        //Otherwise just stir a bit
        NPC_ST_SleepShuffle();
        return;
    }
}

pub unsafe fn NPC_CheckEnemyStealth(target: *mut gentity_t) -> qboolean {
    let mut minDist: f32 = 40.0; //any closer than 40 and we definitely notice
    let maxViewDist: f32;
    let clearLOS: qboolean;

    //In case we aquired one some other way
    if !(*NPC).enemy.is_null() {
        return QTRUE;
    }

    //Ignore notarget
    if (*target).flags & FL_NOTARGET != 0 {
        return QFALSE;
    }

    if (*target).health <= 0 {
        return QFALSE;
    }

    if (*(*target).client).ps.weapon == WP_SABER
        && (*(*target).client).ps.saberHolstered == 0
        && (*(*target).client).ps.saberInFlight == QFALSE
    {
        //if target has saber in hand and activated, we wake up even sooner even if not facing him
        minDist = 100.0;
    }

    let mut target_dist: f32 = DistanceSquared(&(*target).r.currentOrigin, &(*NPC).r.currentOrigin);

    //If the target is this close, then wake up regardless
    if (*(*target).client).ps.pm_flags & PMF_DUCKED == 0
        && (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0
        && target_dist < (minDist * minDist)
    {
        G_SetEnemy(NPC, target);
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
        return QTRUE;
    }

    maxViewDist = MAX_VIEW_DIST as f32;

    let maxViewDist = if (*NPCInfo).stats.visrange > maxViewDist {
        //FIXME: should we always just set maxViewDist to this?
        (*NPCInfo).stats.visrange
    } else {
        maxViewDist
    };

    if target_dist > (maxViewDist * maxViewDist) {
        //out of possible visRange
        return QFALSE;
    }

    //Check FOV first
    if InFOV(target, NPC, (*NPCInfo).stats.hfov, (*NPCInfo).stats.vfov) == QFALSE {
        return QFALSE;
    }

    //clearLOS = ( target->client->ps.leanofs ) ? NPC_ClearLOS5( target->client->renderInfo.eyePoint ) : NPC_ClearLOS4( target );
    clearLOS = NPC_ClearLOS4(target);

    //Now check for clear line of vision
    if clearLOS != QFALSE {
        let mut targ_org: vec3_t = [0.0; 3];
        let hAngle_perc: f32;
        let vAngle_perc: f32;
        let target_speed: f32;
        let target_crouching: c_int;
        let dist_rating: f32;
        let mut speed_rating: f32;
        let turning_rating: f32;
        let light_level: f32;
        let FOV_perc: f32;
        let mut vis_rating: f32;
        let dist_influence: f32;
        let fov_influence: f32;
        let light_influence: f32;
        let mut target_rating: f32;
        let contents: c_int;
        let realize: f32;
        let cautious: f32;

        if (*(*target).client).NPC_class == CLASS_ATST {
            //can't miss 'em!
            G_SetEnemy(NPC, target);
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
            return QTRUE;
        }
        VectorSet(
            &mut targ_org,
            (*target).r.currentOrigin[0],
            (*target).r.currentOrigin[1],
            (*target).r.currentOrigin[2] + (*target).r.maxs[2] - 4.0,
        );
        let mut hAngle_perc_v = NPC_GetHFOVPercentage(
            &targ_org,
            &(*(*NPC).client).renderInfo.eyePoint,
            &(*(*NPC).client).renderInfo.eyeAngles,
            (*NPCInfo).stats.hfov as f32,
        );
        let mut vAngle_perc_v = NPC_GetVFOVPercentage(
            &targ_org,
            &(*(*NPC).client).renderInfo.eyePoint,
            &(*(*NPC).client).renderInfo.eyeAngles,
            (*NPCInfo).stats.vfov as f32,
        );

        //Scale them vertically some, and horizontally pretty harshly
        vAngle_perc_v *= vAngle_perc_v; //( vAngle_perc * vAngle_perc );
        hAngle_perc_v *= hAngle_perc_v * hAngle_perc_v;
        hAngle_perc = hAngle_perc_v;
        vAngle_perc = vAngle_perc_v;

        //Cap our vertical vision severely
        //if ( vAngle_perc <= 0.3f ) // was 0.5f
        //	return qfalse;

        //Assess the player's current status
        target_dist = Distance(&(*target).r.currentOrigin, &(*NPC).r.currentOrigin);

        target_speed = VectorLength(&(*(*target).client).ps.velocity);
        target_crouching = ((*(*target).client).pers.cmd.upmove < 0) as c_int;
        dist_rating = target_dist / maxViewDist;
        speed_rating = target_speed / MAX_VIEW_SPEED as f32;
        turning_rating = 5.0f32; //AngleDelta( target->client->ps.viewangles[PITCH], target->lastAngles[PITCH] )/180.0f + AngleDelta( target->client->ps.viewangles[YAW], target->lastAngles[YAW] )/180.0f;
        light_level = (255 / MAX_LIGHT_INTENSITY) as f32; //( target->lightLevel / MAX_LIGHT_INTENSITY );
        FOV_perc = 1.0f32 - (hAngle_perc + vAngle_perc) * 0.5f32; //FIXME: Dunno about the average...
        vis_rating = 0.0f32;

        //Too dark
        if light_level < MIN_LIGHT_THRESHOLD {
            return QFALSE;
        }

        //Too close?
        if dist_rating < DISTANCE_THRESHOLD {
            G_SetEnemy(NPC, target);
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
            return QTRUE;
        }

        //Out of range
        if dist_rating > 1.0f32 {
            return QFALSE;
        }

        //Cap our speed checks
        if speed_rating > 1.0f32 {
            speed_rating = 1.0f32;
        }

        //Calculate the distance, fov and light influences
        //...Visibilty linearly wanes over distance
        dist_influence = DISTANCE_SCALE * (1.0f32 - dist_rating);
        //...As the percentage out of the FOV increases, straight perception suffers on an exponential scale
        fov_influence = FOV_SCALE * (1.0f32 - FOV_perc);
        //...Lack of light hides, abundance of light exposes
        light_influence = (light_level - 0.5f32) * LIGHT_SCALE;

        //Calculate our base rating
        target_rating = dist_influence + fov_influence + light_influence;

        //Now award any final bonuses to this number
        contents = crate::trap::PointContents(&targ_org, (*target).s.number);
        if contents & CONTENTS_WATER != 0 {
            let myContents =
                crate::trap::PointContents(&(*(*NPC).client).renderInfo.eyePoint, (*NPC).s.number);
            if myContents & CONTENTS_WATER == 0 {
                //I'm not in water
                if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {
                    //these guys can see in in/through water pretty well
                    vis_rating = 0.10f32; //10% bonus
                } else {
                    vis_rating = 0.35f32; //35% bonus
                }
            } else {
                //else, if we're both in water
                if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {
                    //I can see him just fine
                } else {
                    vis_rating = 0.15f32; //15% bonus
                }
            }
        } else {
            //not in water
            if contents & CONTENTS_FOG != 0 {
                vis_rating = 0.15f32; //15% bonus
            }
        }

        target_rating *= 1.0f32 - vis_rating;

        //...Motion draws the eye quickly
        target_rating += speed_rating * SPEED_SCALE;
        target_rating += turning_rating * TURNING_SCALE;
        //FIXME: check to see if they're animating, too?  But can we do something as simple as frame != oldframe?

        //...Smaller targets are harder to indentify
        if target_crouching != 0 {
            target_rating *= 0.9f32; //10% bonus
        }

        //If he's violated the threshold, then realize him
        //float difficulty_scale = 1.0f + (2.0f-g_spskill.value);//if playing on easy, 20% harder to be seen...?
        if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {
            //swamptroopers can see much better
            realize = CAUTIOUS_THRESHOLD; /*difficulty_scale*/
            cautious = CAUTIOUS_THRESHOLD * 0.75f32; /*difficulty_scale*/
        } else {
            realize = REALIZE_THRESHOLD; /*difficulty_scale*/
            cautious = CAUTIOUS_THRESHOLD * 0.75f32; /*difficulty_scale*/
        }

        if target_rating > realize && (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            G_SetEnemy(NPC, target);
            (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
            return QTRUE;
        }

        //If he's above the caution threshold, then realize him in a few seconds unless he moves to cover
        if target_rating > cautious && (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
            //FIXME: ambushing guys should never talk
            if TIMER_Done(NPC, c"enemyLastVisible".as_ptr()) != QFALSE {
                //If we haven't already, start the counter
                let lookTime: c_int = Q_irand(4500, 8500);
                //NPCInfo->timeEnemyLastVisible = level.time + 2000;
                TIMER_Set(NPC, c"enemyLastVisible".as_ptr(), lookTime);
                //TODO: Play a sound along the lines of, "Huh?  What was that?"
                ST_Speech(NPC, SPEECH_SIGHT, 0.0);
                NPC_TempLookTarget(NPC, (*target).s.number, lookTime, lookTime);
                //FIXME: set desired yaw and pitch towards this guy?
            } else if TIMER_Get(NPC, c"enemyLastVisible".as_ptr()) <= (*addr_of!(level)).time + 500
                && (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0
            //FIXME: Is this reliable?
            {
                if (*NPCInfo).rank < RANK_LT && Q_irand(0, 2) == 0 {
                    let interrogateTime: c_int = Q_irand(2000, 4000);
                    ST_Speech(NPC, SPEECH_SUSPICIOUS, 0.0);
                    TIMER_Set(NPC, c"interrogating".as_ptr(), interrogateTime);
                    G_SetEnemy(NPC, target);
                    (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
                    TIMER_Set(NPC, c"attackDelay".as_ptr(), interrogateTime);
                    TIMER_Set(NPC, c"stand".as_ptr(), interrogateTime);
                } else {
                    G_SetEnemy(NPC, target);
                    (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
                    //FIXME: ambush guys (like those popping out of water) shouldn't delay...
                    TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
                    TIMER_Set(NPC, c"stand".as_ptr(), Q_irand(500, 2500));
                }
                return QTRUE;
            }

            return QFALSE;
        }
    }

    QFALSE
}

pub unsafe fn NPC_CheckPlayerTeamStealth() -> qboolean {
    /*
    //NOTENOTE: For now, all stealh checks go against the player, since
    //			he is the main focus.  Squad members and rivals do not
    //			fall into this category and will be ignored.

    NPC_CheckEnemyStealth( &g_entities[0] );	//Change this pointer to assess other entities
    */
    let mut i: c_int;

    i = 0;
    while i < ENTITYNUM_WORLD {
        let enemy: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (*enemy).inuse == QFALSE {
            i += 1;
            continue;
        }

        if !enemy.is_null()
            && !(*enemy).client.is_null()
            && NPC_ValidEnemy(enemy) != QFALSE
            && (*(*enemy).client).playerTeam == (*(*NPC).client).enemyTeam
        {
            if NPC_CheckEnemyStealth(enemy) != QFALSE {
                //Change this pointer to assess other entities
                return QTRUE;
            }
        }
        i += 1;
    }
    QFALSE
}

#[allow(dead_code)] // static helper; only caller (NPC_BSST_* states) still blocked
unsafe fn NPC_ST_InvestigateEvent(eventID: c_int, extraSuspicious: qboolean) -> qboolean {
    //If they've given themselves away, just take them as an enemy
    if (*NPCInfo).confusionTime < (*addr_of!(level)).time {
        if (*addr_of!(level)).alertEvents[eventID as usize].level == AEL_DISCOVERED
            && (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0
        {
            (*NPCInfo).lastAlertID = (*addr_of!(level)).alertEvents[eventID as usize].ID;
            if (*addr_of!(level)).alertEvents[eventID as usize]
                .owner
                .is_null()
                || (*(*addr_of!(level)).alertEvents[eventID as usize].owner)
                    .client
                    .is_null()
                || (*(*addr_of!(level)).alertEvents[eventID as usize].owner).health <= 0
                || (*(*(*addr_of!(level)).alertEvents[eventID as usize].owner).client).playerTeam
                    != (*(*NPC).client).enemyTeam
            {
                //not an enemy
                return QFALSE;
            }
            //FIXME: what if can't actually see enemy, don't know where he is... should we make them just become very alert and start looking for him?  Or just let combat AI handle this... (act as if you lost him)
            //ST_Speech( NPC, SPEECH_CHARGE, 0 );
            G_SetEnemy(NPC, (*addr_of!(level)).alertEvents[eventID as usize].owner);
            (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
            if (*addr_of!(level)).alertEvents[eventID as usize].r#type == AET_SOUND {
                //heard him, didn't see him, stick for a bit
                TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(500, 2500));
            }
            return QTRUE;
        }
    }

    //don't look at the same alert twice
    if (*addr_of!(level)).alertEvents[eventID as usize].ID == (*NPCInfo).lastAlertID {
        return QFALSE;
    }
    (*NPCInfo).lastAlertID = (*addr_of!(level)).alertEvents[eventID as usize].ID;

    //Must be ready to take another sound event
    /*
    if ( NPCInfo->investigateSoundDebounceTime > level.time )
    {
        return qfalse;
    }
    */

    if (*addr_of!(level)).alertEvents[eventID as usize].r#type == AET_SIGHT {
        //sight alert, check the light level
        if (*addr_of!(level)).alertEvents[eventID as usize].light
            < Q_irand(ST_MIN_LIGHT_THRESHOLD, ST_MAX_LIGHT_THRESHOLD) as f32
        {
            //below my threshhold of potentially seeing
            return QFALSE;
        }
    }

    //Save the position for movement (if necessary)
    VectorCopy(
        &(*addr_of!(level)).alertEvents[eventID as usize].position,
        &mut (*NPCInfo).investigateGoal,
    );

    //First awareness of it
    (*NPCInfo).investigateCount += if extraSuspicious != QFALSE { 2 } else { 1 };

    //Clamp the value
    if (*NPCInfo).investigateCount > 4 {
        (*NPCInfo).investigateCount = 4;
    }

    //See if we should walk over and investigate
    if (*addr_of!(level)).alertEvents[eventID as usize].level > AEL_MINOR
        && (*NPCInfo).investigateCount > 1
        && (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0
    {
        //make it so they can walk right to this point and look at it rather than having to use combatPoints
        if G_ExpandPointToBBox(
            &mut (*NPCInfo).investigateGoal,
            &(*NPC).r.mins,
            &(*NPC).r.maxs,
            (*NPC).s.number,
            ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
        ) != QFALSE
        {
            //we were able to move the investigateGoal to a point in which our bbox would fit
            //drop the goal to the ground so we can get at it
            let mut end: vec3_t = [0.0; 3];
            VectorCopy(&(*NPCInfo).investigateGoal, &mut end);
            end[2] -= 512.0; //FIXME: not always right?  What if it's even higher, somehow?
            let trace: trace_t = trap::Trace(
                &(*NPCInfo).investigateGoal,
                &(*NPC).r.mins,
                &(*NPC).r.maxs,
                &end,
                ENTITYNUM_NONE,
                ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
            );
            if trace.fraction >= 1.0f32 {
                //too high to even bother
                //FIXME: look at them???
            } else {
                VectorCopy(&trace.endpos, &mut (*NPCInfo).investigateGoal);
                NPC_SetMoveGoal(
                    NPC,
                    &(*NPCInfo).investigateGoal,
                    16,
                    QTRUE,
                    -1,
                    core::ptr::null_mut(),
                );
                (*NPCInfo).localState = LSTATE_INVESTIGATE;
            }
        } else {
            let id: c_int = NPC_FindCombatPoint(
                &(*NPCInfo).investigateGoal,
                &(*NPCInfo).investigateGoal,
                &(*NPCInfo).investigateGoal,
                CP_INVESTIGATE | CP_HAS_ROUTE,
                0.0,
                -1,
            );

            if id != -1 {
                NPC_SetMoveGoal(
                    NPC,
                    &(*addr_of!(level)).combatPoints[id as usize].origin,
                    16,
                    QTRUE,
                    id,
                    core::ptr::null_mut(),
                );
                (*NPCInfo).localState = LSTATE_INVESTIGATE;
            }
        }
        //Say something
        //FIXME: only if have others in group... these should be responses?
        if (*NPCInfo).investigateDebounceTime + (*NPCInfo).pauseTime > (*addr_of!(level)).time {
            //was already investigating
            if !(*NPCInfo).group.is_null()
                && !(*(*NPCInfo).group).commander.is_null()
                && !(*(*(*NPCInfo).group).commander).client.is_null()
                && (*(*(*(*NPCInfo).group).commander).client).NPC_class == CLASS_IMPERIAL
                && Q_irand(0, 3) == 0
            {
                ST_Speech((*(*NPCInfo).group).commander, SPEECH_LOOK, 0.0); //FIXME: "I'll go check it out" type sounds
            } else {
                ST_Speech(NPC, SPEECH_LOOK, 0.0); //FIXME: "I'll go check it out" type sounds
            }
        } else {
            if (*addr_of!(level)).alertEvents[eventID as usize].r#type == AET_SIGHT {
                ST_Speech(NPC, SPEECH_SIGHT, 0.0);
            } else if (*addr_of!(level)).alertEvents[eventID as usize].r#type == AET_SOUND {
                ST_Speech(NPC, SPEECH_SOUND, 0.0);
            }
        }
        //Setup the debounce info
        (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 5000;
        (*NPCInfo).investigateSoundDebounceTime = (*addr_of!(level)).time + 2000;
        (*NPCInfo).pauseTime = (*addr_of!(level)).time;
    } else {
        //just look?
        //Say something
        if (*addr_of!(level)).alertEvents[eventID as usize].r#type == AET_SIGHT {
            ST_Speech(NPC, SPEECH_SIGHT, 0.0);
        } else if (*addr_of!(level)).alertEvents[eventID as usize].r#type == AET_SOUND {
            ST_Speech(NPC, SPEECH_SOUND, 0.0);
        }
        //Setup the debounce info
        (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 1000;
        (*NPCInfo).investigateSoundDebounceTime = (*addr_of!(level)).time + 1000;
        (*NPCInfo).pauseTime = (*addr_of!(level)).time;
        VectorCopy(
            &(*addr_of!(level)).alertEvents[eventID as usize].position,
            &mut (*NPCInfo).investigateGoal,
        );
    }

    if (*addr_of!(level)).alertEvents[eventID as usize].level >= AEL_DANGER {
        (*NPCInfo).investigateDebounceTime = Q_irand(500, 2500);
    }

    //Start investigating
    (*NPCInfo).tempBehavior = BS_INVESTIGATE;
    QTRUE
}

#[allow(dead_code)] // static helper; only caller (ST_LookAround) still blocked on group-AI / NPC_UpdateAngles
unsafe fn ST_OffsetLook(offset: f32, out: &mut vec3_t) {
    let mut angles: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut temp: vec3_t = [0.0; 3];

    GetAnglesForDirection(
        &(*NPC).r.currentOrigin,
        &(*NPCInfo).investigateGoal,
        &mut angles,
    );
    angles[YAW] += offset;
    AngleVectors(&angles, Some(&mut forward), None, None);
    VectorMA(&(*NPC).r.currentOrigin, 64.0, &forward, out);

    CalcEntitySpot(NPC, SPOT_HEAD, &mut temp as *mut vec3_t);
    out[2] = temp[2];
}

unsafe fn ST_LookAround() {
    let mut lookPos: vec3_t = [0.0; 3];
    let perc: f32 = ((*addr_of!(level)).time - (*NPCInfo).pauseTime) as f32
        / (*NPCInfo).investigateDebounceTime as f32;

    //Keep looking at the spot
    if perc < 0.25 {
        VectorCopy(&(*NPCInfo).investigateGoal, &mut lookPos);
    } else if perc < 0.5f32
    //Look up but straight ahead
    {
        ST_OffsetLook(0.0f32, &mut lookPos);
    } else if perc < 0.75f32
    //Look right
    {
        ST_OffsetLook(45.0f32, &mut lookPos);
    } else
    //Look left
    {
        ST_OffsetLook(-45.0f32, &mut lookPos);
    }

    NPC_FacePosition(&mut lookPos as *mut vec3_t, QTRUE);
}

pub unsafe fn NPC_BSST_Investigate() {
    //get group- mainly for group speech debouncing, but may use for group scouting/investigating AI, too
    AI_GetGroup(NPC);

    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(QTRUE);
    }

    if (*NPCInfo).confusionTime < (*addr_of!(level)).time {
        if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            //Look for an enemy
            if NPC_CheckPlayerTeamStealth() != QFALSE {
                //NPCInfo->behaviorState	= BS_HUNT_AND_KILL;//should be auto now
                ST_Speech(NPC, SPEECH_DETECTED, 0.0);
                (*NPCInfo).tempBehavior = BS_DEFAULT;
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }
    }

    if (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
        let alertEvent: c_int =
            NPC_CheckAlertEvents(QTRUE, QTRUE, (*NPCInfo).lastAlertID, QFALSE, AEL_MINOR);

        //There is an event to look at
        if alertEvent >= 0 {
            if (*NPCInfo).confusionTime < (*addr_of!(level)).time {
                if NPC_CheckForDanger(alertEvent) != QFALSE {
                    //running like hell
                    ST_Speech(NPC, SPEECH_COVER, 0.0); //FIXME: flee sound?
                    return;
                }
            }

            if (*addr_of!(level)).alertEvents[alertEvent as usize].ID != (*NPCInfo).lastAlertID {
                NPC_ST_InvestigateEvent(alertEvent, QTRUE);
            }
        }
    }

    //If we're done looking, then just return to what we were doing
    if ((*NPCInfo).investigateDebounceTime + (*NPCInfo).pauseTime) < (*addr_of!(level)).time {
        (*NPCInfo).tempBehavior = BS_DEFAULT;
        (*NPCInfo).goalEntity = UpdateGoal();

        NPC_UpdateAngles(QTRUE, QTRUE);
        //Say something
        ST_Speech(NPC, SPEECH_GIVEUP, 0.0);
        return;
    }

    //FIXME: else, look for new alerts

    //See if we're searching for the noise's origin
    if (*NPCInfo).localState == LSTATE_INVESTIGATE && (!(*NPCInfo).goalEntity.is_null()) {
        //See if we're there
        if NAV_HitNavGoal(
            &(*NPC).r.currentOrigin,
            &(*NPC).r.mins,
            &(*NPC).r.maxs,
            &(*(*NPCInfo).goalEntity).r.currentOrigin,
            32,
            FlyingCreature(NPC),
        ) == QFALSE
        {
            ucmd.buttons |= BUTTON_WALKING;

            //Try and move there
            if NPC_MoveToGoal(QTRUE) != QFALSE {
                //Bump our times
                (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 5000;
                (*NPCInfo).pauseTime = (*addr_of!(level)).time;

                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }

        //Otherwise we're done or have given up
        //Say something
        //ST_Speech( NPC, SPEECH_LOOK, 0.33f );
        (*NPCInfo).localState = LSTATE_NONE;
    }

    //Look around
    ST_LookAround();
}

pub unsafe fn NPC_BSST_Patrol() {
    //FIXME: pick up on bodies of dead buddies?

    //get group- mainly for group speech debouncing, but may use for group scouting/investigating AI, too
    AI_GetGroup(NPC);

    if (*NPCInfo).confusionTime < (*addr_of!(level)).time {
        //Look for any enemies
        if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            if NPC_CheckPlayerTeamStealth() != QFALSE {
                //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                //NPC_AngerSound();
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }
    }

    if (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
        let alertEvent: c_int = NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_MINOR);

        //There is an event to look at
        if alertEvent >= 0 {
            if NPC_ST_InvestigateEvent(alertEvent, QFALSE) != QFALSE {
                //actually going to investigate it
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }
    }

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons |= BUTTON_WALKING;
        //ST_Move( NPCInfo->goalEntity );
        NPC_MoveToGoal(QTRUE);
    } else
    // if ( !(NPCInfo->scriptFlags&SCF_IGNORE_ALERTS) )
    {
        if (*(*NPC).client).NPC_class != CLASS_IMPERIAL
            && (*(*NPC).client).NPC_class != CLASS_IMPWORKER
        {
            //imperials do not look around
            if TIMER_Done(NPC, c"enemyLastVisible".as_ptr()) != QFALSE {
                //nothing suspicious, look around
                if Q_irand(0, 30) == 0 {
                    (*NPCInfo).desiredYaw = (*NPC).s.angles[1] + Q_irand(-90, 90) as f32;
                }
                if Q_irand(0, 30) == 0 {
                    (*NPCInfo).desiredPitch = Q_irand(-20, 20) as f32;
                }
            }
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
    //TEMP hack for Imperial stand anim
    if (*(*NPC).client).NPC_class == CLASS_IMPERIAL || (*(*NPC).client).NPC_class == CLASS_IMPWORKER
    {
        //hack
        if ucmd.forwardmove != 0 || ucmd.rightmove != 0 || ucmd.upmove != 0 {
            //moving

            if ((*(*NPC).client).ps.torsoTimer <= 0)
                || ((*(*NPC).client).ps.torsoAnim == BOTH_STAND4)
            {
                if (ucmd.buttons & BUTTON_WALKING) != 0 && (*NPCInfo).scriptFlags & SCF_RUNNING == 0
                {
                    //not running, only set upper anim
                    //  No longer overrides scripted anims
                    NPC_SetAnim(
                        NPC,
                        SETANIM_TORSO,
                        BOTH_STAND4,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    (*(*NPC).client).ps.torsoTimer = 200;
                }
            }
        } else {
            //standing still, set both torso and legs anim
            //  No longer overrides scripted anims
            if ((*(*NPC).client).ps.torsoTimer <= 0
                || ((*(*NPC).client).ps.torsoAnim == BOTH_STAND4))
                && ((*(*NPC).client).ps.legsTimer <= 0
                    || ((*(*NPC).client).ps.legsAnim == BOTH_STAND4))
            {
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_STAND4,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                (*(*NPC).client).ps.legsTimer = 200;
                (*(*NPC).client).ps.torsoTimer = (*(*NPC).client).ps.legsTimer;
            }
        }
        //FIXME: this is a disgusting hack that is supposed to make the Imperials start with their weapon holstered- need a better way
        if (*(*NPC).client).ps.weapon != WP_NONE {
            ChangeWeapon(NPC, WP_NONE);
            (*(*NPC).client).ps.weapon = WP_NONE;
            (*(*NPC).client).ps.weaponstate = WEAPON_READY;
            /*
            if ( NPC->weaponModel[0] > 0 )
            {
                gi.G2API_RemoveGhoul2Model( NPC->ghoul2, NPC->weaponModel[0] );
                NPC->weaponModel[0] = -1;
            }
            */
            //rwwFIXMEFIXME: Do this?
        }
    }
}

pub unsafe fn ST_TrackEnemy(self_: *mut gentity_t, enemyPos: &vec3_t) {
    //clear timers
    TIMER_Set(self_, c"attackDelay".as_ptr(), Q_irand(1000, 2000));
    //TIMER_Set( self, "duck", -1 );
    TIMER_Set(self_, c"stick".as_ptr(), Q_irand(500, 1500));
    TIMER_Set(self_, c"stand".as_ptr(), -1);
    TIMER_Set(
        self_,
        c"scoutTime".as_ptr(),
        TIMER_Get(self_, c"stick".as_ptr()) - (*addr_of!(level)).time + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*(*self_).NPC).combatPoint, QFALSE);
    //go after his last seen pos
    NPC_SetMoveGoal(self_, enemyPos, 16, QFALSE, -1, core::ptr::null_mut());
}

pub unsafe fn ST_ApproachEnemy(self_: *mut gentity_t) -> c_int {
    TIMER_Set(self_, c"attackDelay".as_ptr(), Q_irand(250, 500));
    //TIMER_Set( self, "duck", -1 );
    TIMER_Set(self_, c"stick".as_ptr(), Q_irand(1000, 2000));
    TIMER_Set(self_, c"stand".as_ptr(), -1);
    TIMER_Set(
        self_,
        c"scoutTime".as_ptr(),
        TIMER_Get(self_, c"stick".as_ptr()) - (*addr_of!(level)).time + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*(*self_).NPC).combatPoint, QFALSE);
    //return the relevant combat point flags
    CP_CLEAR | CP_CLOSEST
}

pub unsafe fn ST_HuntEnemy(self_: *mut gentity_t) {
    //TIMER_Set( NPC, "attackDelay", Q_irand( 250, 500 ) );//Disabled this for now, guys who couldn't hunt would never attack
    //TIMER_Set( NPC, "duck", -1 );
    TIMER_Set(NPC, c"stick".as_ptr(), Q_irand(250, 1000));
    TIMER_Set(NPC, c"stand".as_ptr(), -1);
    TIMER_Set(
        NPC,
        c"scoutTime".as_ptr(),
        TIMER_Get(NPC, c"stick".as_ptr()) - (*addr_of!(level)).time + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*NPCInfo).combatPoint, QFALSE);
    //go directly after the enemy
    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        (*(*self_).NPC).goalEntity = (*NPC).enemy;
    }
}

pub unsafe fn ST_GetCPFlags() -> c_int {
    let mut cpFlags: c_int = 0;
    if !NPC.is_null() && !(*NPCInfo).group.is_null() {
        if NPC == (*(*NPCInfo).group).commander && (*(*NPC).client).NPC_class == CLASS_IMPERIAL {
            //imperials hang back and give orders
            if (*(*NPCInfo).group).numGroup > 1 && Q_irand(-3, (*(*NPCInfo).group).numGroup) > 1 {
                //FIXME: make sure he;s giving orders with these lines
                if Q_irand(0, 1) != 0 {
                    ST_Speech(NPC, SPEECH_CHASE, 0.5);
                } else {
                    ST_Speech(NPC, SPEECH_YELL, 0.5);
                }
            }
            cpFlags = CP_CLEAR | CP_COVER | CP_AVOID | CP_SAFE | CP_RETREAT;
        } else if (*(*NPCInfo).group).morale < 0 {
            //hide
            cpFlags = CP_COVER | CP_AVOID | CP_SAFE | CP_RETREAT;
        } else if (*(*NPCInfo).group).morale < (*(*NPCInfo).group).numGroup {
            //morale is low for our size
            let moraleDrop: c_int = (*(*NPCInfo).group).numGroup - (*(*NPCInfo).group).morale;
            if moraleDrop < -6 {
                //flee (no clear shot needed)
                cpFlags = CP_FLEE | CP_RETREAT | CP_COVER | CP_AVOID | CP_SAFE;
            } else if moraleDrop < -3 {
                //retreat (no clear shot needed)
                cpFlags = CP_RETREAT | CP_COVER | CP_AVOID | CP_SAFE;
            } else if moraleDrop < 0 {
                //cover (no clear shot needed)
                cpFlags = CP_COVER | CP_AVOID | CP_SAFE;
            }
        } else {
            let moraleBoost: c_int = (*(*NPCInfo).group).morale - (*(*NPCInfo).group).numGroup;
            if moraleBoost > 20 {
                //charge to any one and outflank (no cover needed)
                cpFlags = CP_CLEAR | CP_FLANK | CP_APPROACH_ENEMY;
            } else if moraleBoost > 15 {
                //charge to closest one (no cover needed)
                cpFlags = CP_CLEAR | CP_CLOSEST | CP_APPROACH_ENEMY;
            } else if moraleBoost > 10 {
                //charge closer (no cover needed)
                cpFlags = CP_CLEAR | CP_APPROACH_ENEMY;
            }
        }
    }
    if cpFlags == 0 {
        //at some medium level of morale
        match Q_irand(0, 3) {
            0 => {
                //just take the nearest one
                cpFlags = CP_CLEAR | CP_COVER | CP_NEAREST;
            }
            1 => {
                //take one closer to the enemy
                cpFlags = CP_CLEAR | CP_COVER | CP_APPROACH_ENEMY;
            }
            2 => {
                //take the one closest to the enemy
                cpFlags = CP_CLEAR | CP_COVER | CP_CLOSEST | CP_APPROACH_ENEMY;
            }
            3 => {
                //take the one on the other side of the enemy
                cpFlags = CP_CLEAR | CP_COVER | CP_FLANK | CP_APPROACH_ENEMY;
            }
            _ => {}
        }
    }
    if !NPC.is_null() && (*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST != 0 {
        cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
        cpFlags |= CP_NEAREST;
    }
    cpFlags
}

pub unsafe fn ST_ResolveBlockedShot(hit: c_int) {
    let stuckTime: c_int;
    //figure out how long we intend to stand here, max
    if TIMER_Get(NPC, c"roamTime".as_ptr()) > TIMER_Get(NPC, c"stick".as_ptr()) {
        stuckTime = TIMER_Get(NPC, c"roamTime".as_ptr()) - (*addr_of!(level)).time;
    } else {
        stuckTime = TIMER_Get(NPC, c"stick".as_ptr()) - (*addr_of!(level)).time;
    }

    if TIMER_Done(NPC, c"duck".as_ptr()) != QFALSE {
        //we're not ducking
        if AI_GroupContainsEntNum((*NPCInfo).group, hit) != QFALSE {
            let member: *mut gentity_t =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(hit as usize);
            if TIMER_Done(member, c"duck".as_ptr()) != QFALSE {
                //they aren't ducking
                if TIMER_Done(member, c"stand".as_ptr()) != QFALSE {
                    //they're not being forced to stand
                    //tell them to duck at least as long as I'm not moving
                    TIMER_Set(member, c"duck".as_ptr(), stuckTime);
                    return;
                }
            }
        }
    } else {
        //maybe we should stand
        if TIMER_Done(NPC, c"stand".as_ptr()) != QFALSE {
            //stand for as long as we'll be here
            TIMER_Set(NPC, c"stand".as_ptr(), stuckTime);
            return;
        }
    }
    //Hmm, can't resolve this by telling them to duck or telling me to stand
    //We need to move!
    TIMER_Set(NPC, c"roamTime".as_ptr(), -1);
    TIMER_Set(NPC, c"stick".as_ptr(), -1);
    TIMER_Set(NPC, c"duck".as_ptr(), -1);
    TIMER_Set(NPC, c"attakDelay".as_ptr(), Q_irand(1000, 3000));
}

pub unsafe fn ST_TransferMoveGoal(self_: *mut gentity_t, other: *mut gentity_t) {
    if trap::ICARUS_TaskIDPending(self_, TID_MOVE_NAV) != QFALSE {
        //can't transfer movegoal when a script we're running is waiting to complete
        return;
    }
    if (*(*self_).NPC).combatPoint != -1 {
        //I've got a combatPoint I'm going to, give it to him
        (*(*other).NPC).combatPoint = (*(*self_).NPC).combatPoint;
        (*(*self_).NPC).lastFailedCombatPoint = (*(*self_).NPC).combatPoint;
        (*(*self_).NPC).combatPoint = -1;
    } else {
        //I must be going for a goal, give that to him instead
        if (*(*self_).NPC).goalEntity == (*(*self_).NPC).tempGoal {
            NPC_SetMoveGoal(
                other,
                &(*(*(*self_).NPC).tempGoal).r.currentOrigin,
                (*(*self_).NPC).goalRadius,
                if (*(*(*self_).NPC).tempGoal).flags & FL_NAVGOAL != 0 {
                    QTRUE
                } else {
                    QFALSE
                },
                -1,
                core::ptr::null_mut(),
            );
        } else {
            (*(*other).NPC).goalEntity = (*(*self_).NPC).goalEntity;
        }
    }
    //give him my squadstate
    AI_GroupUpdateSquadstates((*(*self_).NPC).group, other, (*NPCInfo).squadState);

    //give him my timers and clear mine
    ST_TransferTimers(self_, other);

    //now make me stand around for a second or two at least
    AI_GroupUpdateSquadstates((*(*self_).NPC).group, self_, SQUAD_STAND_AND_SHOOT);
    TIMER_Set(self_, c"stand".as_ptr(), Q_irand(1000, 3000));
}

#[allow(dead_code)] // static helper; only caller (NPC_BSST_Attack) still blocked
unsafe fn ST_CheckMoveState() {
    if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) != QFALSE {
        //moving toward a goal that a script is waiting on, so don't stop for anything!
        r#move = QTRUE;
    }
    //See if we're a scout
    else if (*NPCInfo).squadState == SQUAD_SCOUT {
        //If we're supposed to stay put, then stand there and fire
        if TIMER_Done(NPC, c"stick".as_ptr()) == QFALSE {
            r#move = QFALSE;
            return;
        }

        //Otherwise, if we can see our target, just shoot
        if enemyLOS != QFALSE {
            if enemyCS != QFALSE {
                //if we're going after our enemy, we can stop now
                if (*NPCInfo).goalEntity == (*NPC).enemy {
                    AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, SQUAD_STAND_AND_SHOOT);
                    r#move = QFALSE;
                    return;
                }
            }
        } else {
            //Move to find our target
            faceEnemy = QFALSE;
        }

        /*
        if ( TIMER_Done( NPC, "scoutTime" ) )
        {//we can't scout to him, someone else give it a try
            AI_GroupUpdateSquadstates( NPCInfo->group, NPC, SQUAD_STAND_AND_SHOOT );
            TIMER_Set( NPC, "roamTime", Q_irand( 1000, 2000 ) );
            move = qfalse;
            return;
        }
        */

        //ucmd.buttons |= BUTTON_CAREFUL;
    }
    //See if we're running away
    else if (*NPCInfo).squadState == SQUAD_RETREAT {
        if !(*NPCInfo).goalEntity.is_null() {
            faceEnemy = QFALSE;
        } else {
            //um, lost our goal?  Just stand and shoot, then
            (*NPCInfo).squadState = SQUAD_STAND_AND_SHOOT;
        }
    }
    //see if we're heading to some other combatPoint
    else if (*NPCInfo).squadState == SQUAD_TRANSITION {
        //ucmd.buttons |= BUTTON_CAREFUL;
        if (*NPCInfo).goalEntity.is_null() {
            //um, lost our goal?  Just stand and shoot, then
            (*NPCInfo).squadState = SQUAD_STAND_AND_SHOOT;
        }
    }
    //see if we're at point, duck and fire
    else if (*NPCInfo).squadState == SQUAD_POINT {
        if TIMER_Done(NPC, c"stick".as_ptr()) != QFALSE {
            AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, SQUAD_STAND_AND_SHOOT);
            return;
        }

        r#move = QFALSE;
        return;
    }
    //see if we're just standing around
    else if (*NPCInfo).squadState == SQUAD_STAND_AND_SHOOT {
        //from this squadState we can transition to others?
        r#move = QFALSE;
        return;
    }
    //see if we're hiding
    else if (*NPCInfo).squadState == SQUAD_COVER {
        //Should we duck?
        r#move = QFALSE;
        return;
    }
    //see if we're just standing around
    else if (*NPCInfo).squadState == SQUAD_IDLE {
        if (*NPCInfo).goalEntity.is_null() {
            r#move = QFALSE;
            return;
        }
    }
    //??
    else {
        //invalid squadState!
    }

    //See if we're moving towards a goal, not the enemy
    if ((*NPCInfo).goalEntity != (*NPC).enemy) && (!(*NPCInfo).goalEntity.is_null()) {
        //Did we make it?
        if NAV_HitNavGoal(
            &(*NPC).r.currentOrigin,
            &(*NPC).r.mins,
            &(*NPC).r.maxs,
            &(*(*NPCInfo).goalEntity).r.currentOrigin,
            16,
            FlyingCreature(NPC),
        ) != QFALSE
            || (trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE
                && (*NPCInfo).squadState == SQUAD_SCOUT
                && enemyLOS != QFALSE
                && enemyDist <= 10000.0)
        {
            //either hit our navgoal or our navgoal was not a crucial (scripted) one (maybe a combat point) and we're scouting and found our enemy
            let mut newSquadState: c_int = SQUAD_STAND_AND_SHOOT;
            //we got where we wanted to go, set timers based on why we were running
            match (*NPCInfo).squadState {
                //was running away
                SQUAD_RETREAT => {
                    //done fleeing, obviously
                    TIMER_Set(
                        NPC,
                        c"duck".as_ptr(),
                        ((*(*NPC).client).pers.maxHealth - (*NPC).health) * 100,
                    );
                    TIMER_Set(NPC, c"hideTime".as_ptr(), Q_irand(3000, 7000));
                    TIMER_Set(NPC, c"flee".as_ptr(), -(*addr_of!(level)).time);
                    newSquadState = SQUAD_COVER;
                }
                //was heading for a combat point
                SQUAD_TRANSITION => {
                    TIMER_Set(NPC, c"hideTime".as_ptr(), Q_irand(2000, 4000));
                }
                //was running after player
                SQUAD_SCOUT => {}
                _ => {}
            }
            AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, newSquadState);
            NPC_ReachedGoal();
            //don't attack right away
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(250, 500)); //FIXME: Slant for difficulty levels
                                                                        //don't do something else just yet
            TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(1000, 4000));
            return;
        }

        //keep going, hold of roamTimer until we get there
        TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(4000, 8000));
    }
}

#[allow(dead_code)] // static helper; only caller (NPC_BSST_Attack) still blocked
unsafe fn ST_CheckFireState() {
    if enemyCS != QFALSE {
        //if have a clear shot, always try
        return;
    }

    if (*NPCInfo).squadState == SQUAD_RETREAT
        || (*NPCInfo).squadState == SQUAD_TRANSITION
        || (*NPCInfo).squadState == SQUAD_SCOUT
    {
        //runners never try to fire at the last pos
        return;
    }

    if VectorCompare(&(*(*NPC).client).ps.velocity, &vec3_origin) == 0 {
        //if moving at all, don't do this
        return;
    }

    //See if we should continue to fire on their last position
    // !TIMER_Done( NPC, "stick" ) ||
    if hitAlly == QFALSE //we're not going to hit an ally
        && enemyInFOV != QFALSE //enemy is in our FOV //FIXME: or we don't have a clear LOS?
        && (*NPCInfo).enemyLastSeenTime > 0 //we've seen the enemy
        && !(*NPCInfo).group.is_null() //have a group
        && ((*(*NPCInfo).group).numState[SQUAD_RETREAT as usize] > 0
            || (*(*NPCInfo).group).numState[SQUAD_TRANSITION as usize] > 0
            || (*(*NPCInfo).group).numState[SQUAD_SCOUT as usize] > 0)
    //laying down covering fire
    {
        if (*addr_of!(level)).time - (*NPCInfo).enemyLastSeenTime < 10000 //we have seem the enemy in the last 10 seconds
            && ((*NPCInfo).group.is_null()
                || (*addr_of!(level)).time - (*(*NPCInfo).group).lastSeenEnemyTime < 10000)
        //we are not in a group or the group has seen the enemy in the last 10 seconds
        {
            if Q_irand(0, 10) == 0 {
                //Fire on the last known position
                let mut muzzle: vec3_t = [0.0; 3];
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];
                let mut tooClose: qboolean = QFALSE;
                let mut tooFar: qboolean = QFALSE;
                let mut distThreshold: f32;
                let mut dist: f32;

                CalcEntitySpot(NPC, SPOT_HEAD, &mut muzzle as *mut vec3_t);
                if VectorCompare(&*addr_of!(impactPos), &vec3_origin) != 0 {
                    //never checked ShotEntity this frame, so must do a trace...
                    //vec3_t	mins = {-2,-2,-2}, maxs = {2,2,2};
                    let mut forward: vec3_t = [0.0; 3];
                    let mut end: vec3_t = [0.0; 3];
                    AngleVectors(
                        &(*(*NPC).client).ps.viewangles,
                        Some(&mut forward),
                        None,
                        None,
                    );
                    VectorMA(&muzzle, 8192.0, &forward, &mut end);
                    let tr: trace_t = trap::Trace(
                        &muzzle,
                        &vec3_origin,
                        &vec3_origin,
                        &end,
                        (*NPC).s.number,
                        MASK_SHOT,
                    );
                    VectorCopy(&tr.endpos, &mut *addr_of_mut!(impactPos));
                }

                //see if impact would be too close to me
                distThreshold = 16384.0 /*128*128*/; //default
                match (*NPC).s.weapon {
                    WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE | WP_DET_PACK => {
                        distThreshold = 65536.0 /*256*256*/;
                    }
                    WP_REPEATER => {
                        if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                            distThreshold = 65536.0 /*256*256*/;
                        }
                    }
                    _ => {}
                }

                dist = DistanceSquared(&*addr_of!(impactPos), &muzzle);

                if dist < distThreshold {
                    //impact would be too close to me
                    tooClose = QTRUE;
                } else if (*addr_of!(level)).time - (*NPCInfo).enemyLastSeenTime > 5000
                    || (!(*NPCInfo).group.is_null()
                        && (*addr_of!(level)).time - (*(*NPCInfo).group).lastSeenEnemyTime > 5000)
                {
                    //we've haven't seen them in the last 5 seconds
                    //see if it's too far from where he is
                    distThreshold = 65536.0 /*256*256*/; //default
                    match (*NPC).s.weapon {
                        WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                        | WP_DET_PACK => {
                            distThreshold = 262144.0 /*512*512*/;
                        }
                        WP_REPEATER => {
                            if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                                distThreshold = 262144.0 /*512*512*/;
                            }
                        }
                        _ => {}
                    }
                    dist =
                        DistanceSquared(&*addr_of!(impactPos), &(*NPCInfo).enemyLastSeenLocation);
                    if dist > distThreshold {
                        //impact would be too far from enemy
                        tooFar = QTRUE;
                    }
                }

                if tooClose == QFALSE && tooFar == QFALSE {
                    //okay too shoot at last pos
                    VectorSubtract(&(*NPCInfo).enemyLastSeenLocation, &muzzle, &mut dir);
                    VectorNormalize(&mut dir);
                    vectoangles(&dir, &mut angles);

                    (*NPCInfo).desiredYaw = angles[YAW as usize];
                    (*NPCInfo).desiredPitch = angles[PITCH as usize];

                    shoot = QTRUE;
                    faceEnemy = QFALSE;
                    //AI_GroupUpdateSquadstates( NPCInfo->group, NPC, SQUAD_STAND_AND_SHOOT );
                    return;
                }
            }
        }
    }
}

/*
-------------------------
ST_Commander

  Make decisions about who should go where, etc.

FIXME: leader (group-decision-making) AI?
FIXME: need alternate routes!
FIXME: more group voice interaction
FIXME: work in pairs?

-------------------------
*/
pub unsafe fn ST_Commander() {
    let mut cp: c_int;
    let mut cpFlags_org: c_int;
    let mut cpFlags: c_int;
    let group: *mut AIGroupInfo_t = (*NPCInfo).group;
    let mut member: *mut gentity_t; //, *buddy;
    let mut runner: qboolean = QFALSE;
    let mut enemyLost: qboolean = QFALSE;
    let mut enemyProtected: qboolean = QFALSE;
    let mut scouting: qboolean;
    let mut squadState: c_int;
    let curMemberNum: c_int;
    let lastMemberNum: c_int;
    let mut avoidDist: f32;

    (*group).processed = QTRUE;

    if (*group).enemy.is_null() || (*(*group).enemy).client.is_null() {
        //hmm, no enemy...?!
        return;
    }

    //FIXME: have this group commander check the enemy group (if any) and see if they have
    //		superior numbers.  If they do, fall back rather than advance.  If you have
    //		superior numbers, advance on them.
    //FIXME: find the group commander and have him occasionally give orders when there is speech
    //FIXME: start fleeing when only a couple of you vs. a lightsaber, possibly give up if the only one left

    SaveNPCGlobals();

    if (*group).lastSeenEnemyTime < (*addr_of!(level)).time - 180000 {
        //dissolve the group
        ST_Speech(NPC, SPEECH_LOST, 0.0);
        (*(*group).enemy).waypoint = NAV_FindClosestWaypointForEnt((*group).enemy, WAYPOINT_NONE);
        for i in 0..(*group).numGroup {
            member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*group).member[i as usize].number as usize);
            SetNPCGlobals(member);
            if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) != QFALSE {
                //running somewhere that a script requires us to go, don't break from that
                continue;
            }
            if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
                //not allowed to move on my own
                continue;
            }
            //Lost enemy for three minutes?  go into search mode?
            G_ClearEnemy(NPC);
            (*NPC).waypoint = NAV_FindClosestWaypointForEnt(NPC, (*(*group).enemy).waypoint);
            if (*NPC).waypoint == WAYPOINT_NONE {
                (*NPCInfo).behaviorState = BS_DEFAULT; //BS_PATROL;
            } else if (*(*group).enemy).waypoint == WAYPOINT_NONE
                || (trap::Nav_GetPathCost((*NPC).waypoint, (*(*group).enemy).waypoint)
                    >= Q3_INFINITE)
            {
                NPC_BSSearchStart((*NPC).waypoint, BS_SEARCH);
            } else {
                NPC_BSSearchStart((*(*group).enemy).waypoint, BS_SEARCH);
            }
        }
        (*group).enemy = core::ptr::null_mut();
        RestoreNPCGlobals();
        return;
    }

    //See if anyone in our group is not alerted and alert them
    /*
    for ( i = 0; i < group->numGroup; i++ )
    {
        member = &g_entities[group->member[i].number];
        if ( !member->enemy )
        {//he's not mad, so get him mad
            //Have his buddy tell him to get mad
            if ( group->member[i].closestBuddy != ENTITYNUM_NONE )
            {
                buddy = &g_entities[group->member[i].closestBuddy];
                if ( buddy->enemy == group->enemy )
                {
                    SetNPCGlobals( buddy );
                    ST_Speech( NPC, SPEECH_CHARGE, 0.7f );
                }
            }
            SetNPCGlobals( member );
            G_SetEnemy( member, group->enemy );
        }
    }
    */
    //Okay, everyone is mad

    //see if anyone is running
    if (*group).numState[SQUAD_SCOUT as usize] > 0
        || (*group).numState[SQUAD_TRANSITION as usize] > 0
        || (*group).numState[SQUAD_RETREAT as usize] > 0
    {
        //someone is running
        runner = QTRUE;
    }

    if
    /* !runner &&*/
    (*group).lastSeenEnemyTime > (*addr_of!(level)).time - 32000
        && (*group).lastSeenEnemyTime < (*addr_of!(level)).time - 30000
    {
        //no-one has seen the enemy for 30 seconds// and no-one is running after him
        if !(*group).commander.is_null() && Q_irand(0, 1) == 0 {
            ST_Speech((*group).commander, SPEECH_ESCAPING, 0.0);
        } else {
            ST_Speech(NPC, SPEECH_ESCAPING, 0.0);
        }
        //don't say this again
        (*NPCInfo).blockedSpeechDebounceTime = (*addr_of!(level)).time + 3000;
    }

    if (*group).lastSeenEnemyTime < (*addr_of!(level)).time - 10000 {
        //no-one has seen the enemy for at least 10 seconds!  Should send a scout
        enemyLost = QTRUE;
    }

    if (*group).lastClearShotTime < (*addr_of!(level)).time - 5000 {
        //no-one has had a clear shot for 5 seconds!
        enemyProtected = QTRUE;
    }

    //Go through the list:

    //Everyone should try to get to a combat point if possible
    if (*addr_of!(d_asynchronousGroupAI)).integer != 0 {
        //do one member a turn
        (*group).activeMemberNum += 1;
        if (*group).activeMemberNum >= (*group).numGroup {
            (*group).activeMemberNum = 0;
        }
        curMemberNum = (*group).activeMemberNum;
        lastMemberNum = curMemberNum + 1;
    } else {
        curMemberNum = 0;
        lastMemberNum = (*group).numGroup;
    }
    for i in curMemberNum..lastMemberNum {
        //reset combat point flags
        cp = -1;
        cpFlags = 0;
        squadState = SQUAD_IDLE;
        avoidDist = 0.0;
        scouting = QFALSE;

        //get the next guy
        member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*group).member[i as usize].number as usize);
        if (*member).enemy.is_null() {
            //don't include guys that aren't angry
            continue;
        }
        SetNPCGlobals(member);

        if TIMER_Done(NPC, c"flee".as_ptr()) == QFALSE {
            //running away
            continue;
        }

        if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) != QFALSE {
            //running somewhere that a script requires us to go
            continue;
        }

        if (*NPC).s.weapon == WP_NONE
            && !(*NPCInfo).goalEntity.is_null()
            && (*NPCInfo).goalEntity == (*NPCInfo).tempGoal
            && !(*(*NPCInfo).goalEntity).enemy.is_null()
            && (*(*(*NPCInfo).goalEntity).enemy).s.eType == ET_ITEM
        {
            //running to pick up a gun, don't do other logic
            continue;
        }

        //see if this member should start running (only if have no officer... FIXME: should always run from AEL_DANGER_GREAT?)
        if (*group).commander.is_null() || (*(*(*group).commander).NPC).rank < RANK_ENSIGN {
            if NPC_CheckForDanger(NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_DANGER))
                != QFALSE
            {
                //going to run
                ST_Speech(NPC, SPEECH_COVER, 0.0);
                continue;
            }
        }

        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
            //not allowed to do combat-movement
            continue;
        }

        //check the local state
        if (*NPCInfo).squadState != SQUAD_RETREAT {
            //not already retreating
            if (*(*NPC).client).ps.weapon == WP_NONE {
                //weaponless, should be hiding
                if (*NPCInfo).goalEntity.is_null()
                    || (*(*NPCInfo).goalEntity).enemy.is_null()
                    || (*(*(*NPCInfo).goalEntity).enemy).s.eType != ET_ITEM
                {
                    //not running after a pickup
                    if TIMER_Done(NPC, c"hideTime".as_ptr()) != QFALSE
                        || (DistanceSquared(
                            &(*(*group).enemy).r.currentOrigin,
                            &(*NPC).r.currentOrigin,
                        ) < 65536.0
                            && NPC_ClearLOS4((*NPC).enemy) != QFALSE)
                    {
                        //done hiding or enemy near and can see us
                        //er, start another flee I guess?
                        NPC_StartFlee(
                            (*NPC).enemy,
                            &(*(*NPC).enemy).r.currentOrigin,
                            AEL_DANGER_GREAT,
                            5000,
                            10000,
                        );
                    } //else, just hang here
                }
                continue;
            }
            if TIMER_Done(NPC, c"roamTime".as_ptr()) != QFALSE
                && TIMER_Done(NPC, c"hideTime".as_ptr()) != QFALSE
                && (*NPC).health > 10
                && trap::InPVS(&(*(*group).enemy).r.currentOrigin, &(*NPC).r.currentOrigin)
                    == QFALSE
            {
                //cant even see enemy
                //better go after him
                cpFlags |= CP_CLEAR | CP_COVER;
            } else if (*NPCInfo).localState == LSTATE_UNDERFIRE {
                //we've been shot
                match (*(*(*group).enemy).client).ps.weapon {
                    WP_SABER => {
                        if DistanceSquared(
                            &(*(*group).enemy).r.currentOrigin,
                            &(*NPC).r.currentOrigin,
                        ) < 65536.0
                        //256 squared
                        {
                            cpFlags |= CP_AVOID_ENEMY | CP_COVER | CP_AVOID | CP_RETREAT;
                            if (*group).commander.is_null()
                                || (*(*(*group).commander).NPC).rank < RANK_ENSIGN
                            {
                                squadState = SQUAD_RETREAT;
                            }
                            avoidDist = 256.0;
                        }
                    }
                    //default and WP_BLASTER
                    _ => {
                        cpFlags |= CP_COVER;
                    }
                }
                if (*NPC).health <= 10 {
                    if (*group).commander.is_null()
                        || (*(*(*group).commander).NPC).rank < RANK_ENSIGN
                    {
                        cpFlags |= CP_FLEE | CP_AVOID | CP_RETREAT;
                        squadState = SQUAD_RETREAT;
                    }
                }
            } else {
                //not hit, see if there are other reasons we should run
                if trap::InPVS(&(*NPC).r.currentOrigin, &(*(*group).enemy).r.currentOrigin)
                    != QFALSE
                {
                    //in the same room as enemy
                    if (*(*NPC).client).ps.weapon == WP_ROCKET_LAUNCHER
                        && DistanceSquared(
                            &(*(*group).enemy).r.currentOrigin,
                            &(*NPC).r.currentOrigin,
                        ) < MIN_ROCKET_DIST_SQUARED
                        && (*NPCInfo).squadState != SQUAD_TRANSITION
                    {
                        //too close for me to fire my weapon and I'm not already on the move
                        cpFlags |= CP_AVOID_ENEMY | CP_CLEAR | CP_AVOID;
                        avoidDist = 256.0;
                    } else {
                        match (*(*(*group).enemy).client).ps.weapon {
                            WP_SABER => {
                                //if ( group->enemy->client->ps.SaberLength() > 0 )
                                if (*(*(*group).enemy).client).ps.saberHolstered == 0 {
                                    if DistanceSquared(
                                        &(*(*group).enemy).r.currentOrigin,
                                        &(*NPC).r.currentOrigin,
                                    ) < 65536.0
                                    {
                                        if TIMER_Done(NPC, c"hideTime".as_ptr()) != QFALSE {
                                            if (*NPCInfo).squadState != SQUAD_TRANSITION {
                                                //not already moving: FIXME: we need to see if where we're going is good now?
                                                cpFlags |= CP_AVOID_ENEMY | CP_CLEAR | CP_AVOID;
                                                avoidDist = 256.0;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if cpFlags == 0 {
            //okay, we have no new enemy-driven reason to run... let's use tactics now
            if runner != QFALSE && (*NPCInfo).combatPoint != -1 {
                //someone is running and we have a combat point already
                if (*NPCInfo).squadState != SQUAD_SCOUT
                    && (*NPCInfo).squadState != SQUAD_TRANSITION
                    && (*NPCInfo).squadState != SQUAD_RETREAT
                {
                    //it's not us
                    if TIMER_Done(NPC, c"verifyCP".as_ptr()) != QFALSE
                        && DistanceSquared(
                            &(*NPC).r.currentOrigin,
                            &(*addr_of!(level)).combatPoints[(*NPCInfo).combatPoint as usize]
                                .origin,
                        ) > 64.0 * 64.0
                    {
                        //1 - 3 seconds have passed since you chose a CP, see if you're there since, for some reason, you've stopped running...
                        //uh, WTF, we're not on our combat point?
                        //er, try again, I guess?
                        cp = (*NPCInfo).combatPoint;
                        cpFlags |= ST_GetCPFlags();
                    } else {
                        //cover them
                        //stop ducking
                        TIMER_Set(NPC, c"duck".as_ptr(), -1);
                        //start shooting
                        TIMER_Set(NPC, c"attackDelay".as_ptr(), -1);
                        //AI should take care of the rest - fire at enemy
                    }
                } else {
                    //we're running
                    //see if we're blocked
                    if (*NPCInfo).aiFlags & NPCAI_BLOCKED != 0 {
                        //dammit, something is in our way
                        //see if it's one of ours
                        for j in 0..(*group).numGroup {
                            if (*group).member[j as usize].number == (*NPCInfo).blockingEntNum {
                                //we're being blocked by one of our own, pass our goal onto them and I'll stand still
                                ST_TransferMoveGoal(
                                    NPC,
                                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                        .add((*group).member[j as usize].number as usize),
                                );
                                break;
                            }
                        }
                    }
                    //we don't need to do anything else
                    continue;
                }
            } else {
                //okay no-one is running, use some tactics
                if (*NPCInfo).combatPoint != -1 {
                    //we have a combat point we're supposed to be running to
                    if (*NPCInfo).squadState != SQUAD_SCOUT
                        && (*NPCInfo).squadState != SQUAD_TRANSITION
                        && (*NPCInfo).squadState != SQUAD_RETREAT
                    {
                        //but we're not running
                        if TIMER_Done(NPC, c"verifyCP".as_ptr()) != QFALSE {
                            //1 - 3 seconds have passed since you chose a CP, see if you're there since, for some reason, you've stopped running...
                            if DistanceSquared(
                                &(*NPC).r.currentOrigin,
                                &(*addr_of!(level)).combatPoints[(*NPCInfo).combatPoint as usize]
                                    .origin,
                            ) > 64.0 * 64.0
                            {
                                //uh, WTF, we're not on our combat point?
                                //er, try again, I guess?
                                cp = (*NPCInfo).combatPoint;
                                cpFlags |= ST_GetCPFlags();
                            }
                        }
                    }
                }
                if enemyLost != QFALSE {
                    //if no-one has seen the enemy for a while, send a scout
                    //ask where he went
                    if (*group).numState[SQUAD_SCOUT as usize] <= 0 {
                        scouting = QTRUE;
                        NPC_ST_StoreMovementSpeech(SPEECH_CHASE, 0.0);
                    }
                    //Since no-one else has done this, I should be the closest one, so go after him...
                    ST_TrackEnemy(NPC, &(*group).enemyLastSeenPos);
                    //set me into scout mode
                    AI_GroupUpdateSquadstates(group, NPC, SQUAD_SCOUT);
                    //we're not using a cp, so we need to set runner to true right here
                    runner = QTRUE;
                } else if enemyProtected != QFALSE {
                    //if no-one has a clear shot at the enemy, someone should go after him
                    //FIXME: if I'm in an area where no safe combat points have a clear shot at me, they don't come after me... they should anyway, though after some extra hesitation.
                    //ALSO: seem to give up when behind an area portal?
                    //since no-one else here has done this, I should be the closest one
                    if TIMER_Done(NPC, c"roamTime".as_ptr()) != QFALSE
                        && Q_irand(0, (*group).numGroup) == 0
                    {
                        //only do this if we're ready to move again and we feel like it
                        cpFlags |= ST_ApproachEnemy(NPC);
                        //set me into scout mode
                        AI_GroupUpdateSquadstates(group, NPC, SQUAD_SCOUT);
                    }
                } else {
                    //group can see and has been shooting at the enemy
                    //see if we should do something fancy?

                    {
                        //we're ready to move
                        if (*NPCInfo).combatPoint == -1 {
                            //we're not on a combat point
                            if true
                            /* !Q_irand( 0, 2 )*/
                            {
                                //we should go for a combat point
                                cpFlags |= ST_GetCPFlags();
                            }
                            // else {
                            //     TIMER_Set( NPC, "stick", Q_irand( 2000, 4000 ) );
                            //     TIMER_Set( NPC, "roamTime", Q_irand( 1000, 3000 ) );
                            // }
                        } else if TIMER_Done(NPC, c"roamTime".as_ptr()) != QFALSE {
                            //we are already on a combat point
                            if i == 0 {
                                //we're the closest
                                if (*group).morale - (*group).numGroup > 0 && Q_irand(0, 4) == 0 {
                                    //try to outflank him
                                    cpFlags |= CP_CLEAR | CP_COVER | CP_FLANK | CP_APPROACH_ENEMY;
                                } else if (*group).morale - (*group).numGroup < 0 {
                                    //better move!
                                    cpFlags |= ST_GetCPFlags();
                                } else {
                                    //If we're point, then get down
                                    TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(2000, 5000));
                                    TIMER_Set(NPC, c"stick".as_ptr(), Q_irand(2000, 5000));
                                    //FIXME: what if we can't shoot from a ducked pos?
                                    TIMER_Set(NPC, c"duck".as_ptr(), Q_irand(3000, 4000));
                                    AI_GroupUpdateSquadstates(group, NPC, SQUAD_POINT);
                                }
                            } else if i == (*group).numGroup - 1 {
                                //farthest from the enemy
                                if (*group).morale - (*group).numGroup < 0 {
                                    //low morale, just hang here
                                    TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(2000, 5000));
                                    TIMER_Set(NPC, c"stick".as_ptr(), Q_irand(2000, 5000));
                                } else if (*group).morale - (*group).numGroup > 0 {
                                    //try to move in on the enemy
                                    cpFlags |= ST_ApproachEnemy(NPC);
                                    //set me into scout mode
                                    AI_GroupUpdateSquadstates(group, NPC, SQUAD_SCOUT);
                                } else {
                                    //use normal decision making process
                                    cpFlags |= ST_GetCPFlags();
                                }
                            } else {
                                //someone in-between
                                if (*group).morale - (*group).numGroup < 0 || Q_irand(0, 4) == 0 {
                                    //do something
                                    cpFlags |= ST_GetCPFlags();
                                } else {
                                    TIMER_Set(NPC, c"stick".as_ptr(), Q_irand(2000, 4000));
                                    TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(2000, 4000));
                                }
                            }
                        }
                    }
                    if cpFlags == 0 {
                        //still not moving
                        //see if we should say something?
                        /*
                        if ( NPC->attackDebounceTime < level.time - 2000 )
                        {//we, personally, haven't shot for 2 seconds
                            //maybe yell at the enemy?
                            ST_Speech( NPC, SPEECH_CHARGE, 0.9f );
                        }
                        */

                        //see if we should do other fun stuff
                        //toy with ducking
                        if TIMER_Done(NPC, c"duck".as_ptr()) != QFALSE {
                            //not ducking
                            if TIMER_Done(NPC, c"stand".as_ptr()) != QFALSE {
                                //don't have to keep standing
                                if (*NPCInfo).combatPoint == -1
                                    || (*addr_of!(level)).combatPoints
                                        [(*NPCInfo).combatPoint as usize]
                                        .flags
                                        & CPF_DUCK
                                        != 0
                                {
                                    //okay to duck here
                                    if Q_irand(0, 3) == 0 {
                                        TIMER_Set(NPC, c"duck".as_ptr(), Q_irand(1000, 3000));
                                    }
                                }
                            }
                        }
                        //FIXME: what about CPF_LEAN?
                    }
                }
            }
        }

        //clear the local state
        (*NPCInfo).localState = LSTATE_NONE;

        if (*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST != 0 {
            cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
            cpFlags |= CP_NEAREST;
        }
        //Assign combat points
        if cpFlags != 0 {
            //we want to run to a combat point
            /*
            if ( NPCInfo->combatPoint != -1 )
            {//if we're on a combat point, we obviously don't want the one we're closest to
                cpFlags |= CP_AVOID;
            }
            */

            if (*(*(*group).enemy).client).ps.weapon == WP_SABER
                && /*group->enemy->client->ps.SaberLength() > 0*/
                (*(*(*group).enemy).client).ps.saberHolstered == 0
            {
                //we obviously want to avoid the enemy if he has a saber
                cpFlags |= CP_AVOID_ENEMY;
                avoidDist = 256.0;
            }

            //remember what we *wanted* to do...
            cpFlags_org = cpFlags;

            //now get a combat point
            if cp == -1 {
                //may have had sone set above
                cp = NPC_FindCombatPoint(
                    &(*NPC).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &(*(*group).enemy).r.currentOrigin,
                    cpFlags | CP_HAS_ROUTE,
                    avoidDist,
                    (*NPCInfo).lastFailedCombatPoint,
                );
            }
            while cp == -1 && cpFlags != CP_ANY {
                //start "OR"ing out certain flags to see if we can find *any* point
                if cpFlags & CP_INVESTIGATE != 0 {
                    //don't need to investigate
                    cpFlags &= !CP_INVESTIGATE;
                } else if cpFlags & CP_SQUAD != 0 {
                    //don't need to stick to squads
                    cpFlags &= !CP_SQUAD;
                } else if cpFlags & CP_DUCK != 0 {
                    //don't need to duck
                    cpFlags &= !CP_DUCK;
                } else if cpFlags & CP_NEAREST != 0 {
                    //don't need closest one to me
                    cpFlags &= !CP_NEAREST;
                } else if cpFlags & CP_FLANK != 0 {
                    //don't need to flank enemy
                    cpFlags &= !CP_FLANK;
                } else if cpFlags & CP_SAFE != 0 {
                    //don't need one that hasn't been shot at recently
                    cpFlags &= !CP_SAFE;
                } else if cpFlags & CP_CLOSEST != 0 {
                    //don't need to get closest to enemy
                    cpFlags &= !CP_CLOSEST;
                    //but let's try to approach at least
                    cpFlags |= CP_APPROACH_ENEMY;
                } else if cpFlags & CP_APPROACH_ENEMY != 0 {
                    //don't need to approach enemy
                    cpFlags &= !CP_APPROACH_ENEMY;
                } else if cpFlags & CP_COVER != 0 {
                    //don't need cover
                    cpFlags &= !CP_COVER;
                    //but let's pick one that makes us duck
                    cpFlags |= CP_DUCK;
                } else if cpFlags & CP_CLEAR != 0 {
                    //don't need a clear shot to enemy
                    cpFlags &= !CP_CLEAR;
                } else if cpFlags & CP_AVOID_ENEMY != 0 {
                    //don't need to avoid enemy
                    cpFlags &= !CP_AVOID_ENEMY;
                } else if cpFlags & CP_RETREAT != 0 {
                    //don't need to retreat
                    cpFlags &= !CP_RETREAT;
                } else if cpFlags & CP_FLEE != 0 {
                    //don't need to flee
                    cpFlags &= !CP_FLEE;
                    //but at least avoid enemy and pick one that gives cover
                    cpFlags |= CP_COVER | CP_AVOID_ENEMY;
                } else if cpFlags & CP_AVOID != 0 {
                    //okay, even pick one right by me
                    cpFlags &= !CP_AVOID;
                } else {
                    cpFlags = CP_ANY;
                }
                //now try again
                cp = NPC_FindCombatPoint(
                    &(*NPC).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &(*(*group).enemy).r.currentOrigin,
                    cpFlags | CP_HAS_ROUTE,
                    avoidDist,
                    -1,
                );
            }
            //see if we got a valid one
            if cp != -1 {
                //found a combat point
                //let others know that someone is now running
                runner = QTRUE;
                //don't change course again until we get to where we're going
                TIMER_Set(NPC, c"roamTime".as_ptr(), Q3_INFINITE);
                TIMER_Set(NPC, c"verifyCP".as_ptr(), Q_irand(1000, 3000)); //don't make sure you're in your CP for 1 - 3 seconds
                NPC_SetCombatPoint(cp);
                NPC_SetMoveGoal(
                    NPC,
                    &(*addr_of!(level)).combatPoints[cp as usize].origin,
                    8,
                    QTRUE,
                    cp,
                    core::ptr::null_mut(),
                );
                //okay, try a move right now to see if we can even get there

                //if ( ST_Move() )
                {
                    //we actually can get to it, so okay to say you're going there.
                    //FIXME: Hmm... any way we can store this move info so we don't have to do it again
                    //		when our turn to think comes up?

                    //set us up so others know we're on the move
                    if squadState != SQUAD_IDLE {
                        AI_GroupUpdateSquadstates(group, NPC, squadState);
                    } else if cpFlags & CP_FLEE != 0 {
                        //outright running for your life
                        AI_GroupUpdateSquadstates(group, NPC, SQUAD_RETREAT);
                    } else {
                        //any other kind of transition between combat points
                        AI_GroupUpdateSquadstates(group, NPC, SQUAD_TRANSITION);
                    }

                    //unless we're trying to flee, walk slowly
                    if cpFlags_org & CP_FLEE == 0 {
                        //ucmd.buttons |= BUTTON_CAREFUL;
                    }

                    /*
                    if ( scouting )
                    {//successfully chasing enemy
                        ST_Speech( NPC, SPEECH_CHASE, 0.0f );
                        //don't say this again
                        //group->speechDebounceTime = level.time + 5000;
                    }
                    //flanking:
                    else */
                    if cpFlags & CP_FLANK != 0 {
                        if (*group).numGroup > 1 {
                            NPC_ST_StoreMovementSpeech(SPEECH_OUTFLANK, -1.0);
                        }
                    } else {
                        //okay, let's cheat
                        if (*group).numGroup > 1 {
                            let mut dot: f32 = 1.0;
                            if Q_irand(0, 3) == 0 {
                                //25% of the time, see if we're flanking the enemy
                                let mut eDir2Me: vec3_t = [0.0; 3];
                                let mut eDir2CP: vec3_t = [0.0; 3];

                                VectorSubtract(
                                    &(*NPC).r.currentOrigin,
                                    &(*(*group).enemy).r.currentOrigin,
                                    &mut eDir2Me,
                                );
                                VectorNormalize(&mut eDir2Me);

                                VectorSubtract(
                                    &(*addr_of!(level)).combatPoints
                                        [(*NPCInfo).combatPoint as usize]
                                        .origin,
                                    &(*(*group).enemy).r.currentOrigin,
                                    &mut eDir2CP,
                                );
                                VectorNormalize(&mut eDir2CP);

                                dot = DotProduct(&eDir2Me, &eDir2CP);
                            }

                            if dot < 0.4 {
                                //flanking!
                                NPC_ST_StoreMovementSpeech(SPEECH_OUTFLANK, -1.0);
                            } else if Q_irand(0, 10) == 0 {
                                //regular movement
                                NPC_ST_StoreMovementSpeech(SPEECH_YELL, 0.2); //was SPEECH_COVER
                            }
                        }
                    }
                    /*
                    else if ( cpFlags & CP_CLOSEST || cpFlags & CP_APPROACH_ENEMY )
                    {
                        if ( group->numGroup > 1 )
                        {
                            NPC_ST_StoreMovementSpeech( SPEECH_CHASE, 0.4f );
                        }
                    }
                    */
                } //else: nothing, a failed move should clear the combatPoint and you can try again next frame
            } else if (*NPCInfo).squadState == SQUAD_SCOUT {
                //we couldn't find a combatPoint by the player, so just go after him directly
                ST_HuntEnemy(NPC);
                //set me into scout mode
                AI_GroupUpdateSquadstates(group, NPC, SQUAD_SCOUT);
                //AI should take care of rest
            }
        }
        let _ = scouting;
    }

    RestoreNPCGlobals();
}

/*
-------------------------
NPC_BSST_Attack
-------------------------
*/

pub unsafe fn NPC_BSST_Attack() {
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut shootDir: vec3_t = [0.0; 3];
    let dot: f32;

    //Don't do anything if we're hurt
    if (*NPC).painDebounceTime > (*addr_of!(level)).time {
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    //NPC_CheckEnemy( qtrue, qfalse );
    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE
    // !NPC->enemy )//
    {
        (*NPC).enemy = core::ptr::null_mut();
        if (*(*NPC).client).playerTeam == NPCTEAM_PLAYER {
            NPC_BSPatrol();
        } else {
            NPC_BSST_Patrol(); //FIXME: or patrol?
        }
        return;
    }

    //FIXME: put some sort of delay into the guys depending on how they saw you...?

    //Get our group info
    if TIMER_Done(NPC, c"interrogating".as_ptr()) != QFALSE {
        AI_GetGroup(NPC); //, 45, 512, NPC->enemy );
    } else {
        //FIXME: when done interrogating, I should send out a team alert!
    }

    if !(*NPCInfo).group.is_null() {
        //I belong to a squad of guys - we should *always* have a group
        if (*(*NPCInfo).group).processed == QFALSE {
            //I'm the first ent in my group, I'll make the command decisions
            // (#if AI_TIMERS debug timing block dropped — off-by-default profiling)
            ST_Commander();
        }
    } else if TIMER_Done(NPC, c"flee".as_ptr()) != QFALSE
        && NPC_CheckForDanger(NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_DANGER)) != QFALSE
    {
        //not already fleeing, and going to run
        ST_Speech(NPC, SPEECH_COVER, 0.0);
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    if (*NPC).enemy.is_null() {
        //WTF?  somehow we lost our enemy?
        NPC_BSST_Patrol(); //FIXME: or patrol?
        return;
    }

    enemyLOS = QFALSE;
    enemyCS = QFALSE;
    enemyInFOV = QFALSE;
    r#move = QTRUE;
    faceEnemy = QFALSE;
    shoot = QFALSE;
    hitAlly = QFALSE;
    VectorClear(&mut *addr_of_mut!(impactPos));
    enemyDist = DistanceSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin);

    VectorSubtract(
        &(*(*NPC).enemy).r.currentOrigin,
        &(*NPC).r.currentOrigin,
        &mut enemyDir,
    );
    VectorNormalize(&mut enemyDir);
    AngleVectors(
        &(*(*NPC).client).ps.viewangles,
        Some(&mut shootDir),
        None,
        None,
    );
    dot = DotProduct(&enemyDir, &shootDir);
    if dot > 0.5 || (enemyDist * (1.0 - dot)) < 10000.0 {
        //enemy is in front of me or they're very close and not behind me
        enemyInFOV = QTRUE;
    }

    if enemyDist < MIN_ROCKET_DIST_SQUARED
    //128
    {
        //enemy within 128
        if ((*(*NPC).client).ps.weapon == WP_FLECHETTE || (*(*NPC).client).ps.weapon == WP_REPEATER)
            && (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0
        {
            //shooting an explosive, but enemy too close, switch to primary fire
            (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
            //FIXME: we can never go back to alt-fire this way since, after this, we don't know if we were initially supposed to use alt-fire or not...
        }
    } else if enemyDist > 65536.0
    //256 squared
    {
        if (*(*NPC).client).ps.weapon == WP_DISRUPTOR {
            //sniping... should be assumed
            if (*NPCInfo).scriptFlags & SCF_ALT_FIRE == 0 {
                //use primary fire
                (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
                //reset fire-timing variables
                NPC_ChangeWeapon(WP_DISRUPTOR);
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }
    }

    //can we see our target?
    if NPC_ClearLOS4((*NPC).enemy) != QFALSE {
        AI_GroupUpdateEnemyLastSeen((*NPCInfo).group, &(*(*NPC).enemy).r.currentOrigin);
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
        enemyLOS = QTRUE;

        if (*(*NPC).client).ps.weapon == WP_NONE {
            enemyCS = QFALSE; //not true, but should stop us from firing
            NPC_AimAdjust(-1); //adjust aim worse longer we have no weapon
        } else {
            //can we shoot our target?
            if ((*(*NPC).client).ps.weapon == WP_ROCKET_LAUNCHER
                || ((*(*NPC).client).ps.weapon == WP_FLECHETTE
                    && (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0))
                && enemyDist < MIN_ROCKET_DIST_SQUARED
            //128*128
            {
                enemyCS = QFALSE; //not true, but should stop us from firing
                hitAlly = QTRUE; //us!
                                 //FIXME: if too close, run away!
            } else if enemyInFOV != QFALSE {
                //if enemy is FOV, go ahead and check for shooting
                let hit: c_int = NPC_ShotEntity((*NPC).enemy, addr_of_mut!(impactPos));
                let hitEnt: *mut gentity_t =
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(hit as usize);

                if hit == (*(*NPC).enemy).s.number
                    || (!hitEnt.is_null()
                        && !(*hitEnt).client.is_null()
                        && (*(*hitEnt).client).playerTeam == (*(*NPC).client).enemyTeam)
                    || (!hitEnt.is_null()
                        && (*hitEnt).takedamage != QFALSE
                        && ((*hitEnt).r.svFlags & SVF_GLASS_BRUSH != 0
                            || (*hitEnt).health < 40
                            || (*NPC).s.weapon == WP_EMPLACED_GUN))
                {
                    //can hit enemy or enemy ally or will hit glass or other minor breakable (or in emplaced gun), so shoot anyway
                    AI_GroupUpdateClearShotTime((*NPCInfo).group);
                    enemyCS = QTRUE;
                    NPC_AimAdjust(2); //adjust aim better longer we have clear shot at enemy
                    VectorCopy(
                        &(*(*NPC).enemy).r.currentOrigin,
                        &mut (*NPCInfo).enemyLastSeenLocation,
                    );
                } else {
                    //Hmm, have to get around this bastard
                    NPC_AimAdjust(1); //adjust aim better longer we can see enemy
                    ST_ResolveBlockedShot(hit);
                    if !hitEnt.is_null()
                        && !(*hitEnt).client.is_null()
                        && (*(*hitEnt).client).playerTeam == (*(*NPC).client).playerTeam
                    {
                        //would hit an ally, don't fire!!!
                        hitAlly = QTRUE;
                    } else {
                        //Check and see where our shot *would* hit... if it's not close to the enemy (within 256?), then don't fire
                    }
                }
            } else {
                enemyCS = QFALSE; //not true, but should stop us from firing
            }
        }
    } else if trap::InPVS(&(*(*NPC).enemy).r.currentOrigin, &(*NPC).r.currentOrigin) != QFALSE {
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
        faceEnemy = QTRUE;
        NPC_AimAdjust(-1); //adjust aim worse longer we cannot see enemy
    }

    if (*(*NPC).client).ps.weapon == WP_NONE {
        faceEnemy = QFALSE;
        shoot = QFALSE;
    } else {
        if enemyLOS != QFALSE {
            //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
            faceEnemy = QTRUE;
        }
        if enemyCS != QFALSE {
            shoot = QTRUE;
        }
    }

    //Check for movement to take care of
    ST_CheckMoveState();

    //See if we should override shooting decision with any special considerations
    ST_CheckFireState();

    if faceEnemy != QFALSE {
        //face the enemy
        NPC_FaceEnemy(QTRUE);
    }

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
        //not supposed to chase my enemies
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            //goal is my entity, so don't move
            r#move = QFALSE;
        }
    }

    if (*(*NPC).client).ps.weaponTime > 0 && (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
        r#move = QFALSE;
    }

    if r#move != QFALSE {
        //move toward goal
        if !(*NPCInfo).goalEntity.is_null()
        //&& ( NPCInfo->goalEntity != NPC->enemy || enemyDist > 10000 ) )//100 squared
        {
            r#move = ST_Move();
        } else {
            r#move = QFALSE;
        }
    }

    if r#move == QFALSE {
        if TIMER_Done(NPC, c"duck".as_ptr()) == QFALSE {
            ucmd.upmove = -127;
        }
        //FIXME: what about leaning?
    } else {
        //stop ducking!
        TIMER_Set(NPC, c"duck".as_ptr(), -1);
    }

    if TIMER_Done(NPC, c"flee".as_ptr()) == QFALSE {
        //running away
        faceEnemy = QFALSE;
    }

    //FIXME: check scf_face_move_dir here?

    if faceEnemy == QFALSE {
        //we want to face in the dir we're running
        if r#move == QFALSE {
            //if we haven't moved, we should look in the direction we last looked?
            VectorCopy(
                &(*(*NPC).client).ps.viewangles,
                &mut (*NPCInfo).lastPathAngles,
            );
        }
        (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW as usize];
        (*NPCInfo).desiredPitch = 0.0;
        NPC_UpdateAngles(QTRUE, QTRUE);
        if r#move != QFALSE {
            //don't run away and shoot
            shoot = QFALSE;
        }
    }

    if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
        shoot = QFALSE;
    }

    if !(*NPC).enemy.is_null() && !(*(*NPC).enemy).enemy.is_null() {
        if (*(*NPC).enemy).s.weapon == WP_SABER && (*(*(*NPC).enemy).enemy).s.weapon == WP_SABER {
            //don't shoot at an enemy jedi who is fighting another jedi, for fear of injuring one or causing rogue blaster deflections (a la Obi Wan/Vader duel at end of ANH)
            shoot = QFALSE;
        }
    }
    //FIXME: don't shoot right away!
    if (*(*NPC).client).ps.weaponTime > 0 {
        if (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
            if enemyLOS == QFALSE || enemyCS == QFALSE {
                //cancel it
                (*(*NPC).client).ps.weaponTime = 0;
            } else {
                //delay our next attempt
                TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(3000, 5000));
            }
        }
    } else if shoot != QFALSE {
        //try to shoot if it's time
        if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE {
            if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON == 0 {
                // we've already fired, no need to do it again here
                WeaponThink(QTRUE);
            }
            //NASTY
            if (*NPC).s.weapon == WP_ROCKET_LAUNCHER
                && (ucmd.buttons & BUTTON_ATTACK) != 0
                && r#move == QFALSE
                && (*addr_of!(g_spskill)).integer > 1
                && Q_irand(0, 3) == 0
            {
                //every now and then, shoot a homing rocket
                ucmd.buttons &= !BUTTON_ATTACK;
                ucmd.buttons |= BUTTON_ALT_ATTACK;
                (*(*NPC).client).ps.weaponTime = Q_irand(1000, 2500);
            }
        }
    }
}

pub unsafe fn NPC_BSST_Default() {
    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(QTRUE);
    }

    if (*NPC).enemy.is_null() {
        //don't have an enemy, look for one
        NPC_BSST_Patrol();
    } else
    //if ( NPC->enemy )
    {
        //have an enemy
        NPC_CheckGetNewWeapon();
        NPC_BSST_Attack();
    }
}
