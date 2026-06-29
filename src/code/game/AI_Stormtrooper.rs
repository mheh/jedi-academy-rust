#![allow(
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_mut,
    unused_assignments,
    unused_imports,
    non_camel_case_types,
    unused_unsafe,
)]
// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
use crate::code::game::g_headers_h::*;

use crate::code::game::b_local_h::*;
use crate::code::game::g_nav_h::*;
use crate::code::game::anims_h::*;
use crate::code::game::g_navigator_h::*;

use core::ffi::{c_int, c_float, c_char};
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    fn CG_DrawAlert(origin: vec3_t, rating: f32);
    fn G_AddVoiceEvent(self_: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    fn AI_GroupUpdateSquadstates(group: *mut AIGroupInfo_t, member: *mut gentity_t, newSquadState: c_int);
    fn AI_GroupContainsEntNum(group: *mut AIGroupInfo_t, entNum: c_int) -> qboolean;
    fn AI_GroupUpdateEnemyLastSeen(group: *mut AIGroupInfo_t, spot: vec3_t);
    fn AI_GroupUpdateClearShotTime(group: *mut AIGroupInfo_t);
    fn NPC_TempLookTarget(self_: *mut gentity_t, lookEntNum: c_int, minLookTime: c_int, maxLookTime: c_int);
    fn G_ExpandPointToBBox(point: vec3_t, mins: *const f32, maxs: *const f32, ignore: c_int, clipmask: c_int) -> qboolean;
    fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int);
    fn NPC_CheckGetNewWeapon();
    fn Q3_TaskIDPending(ent: *mut gentity_t, taskType: taskID_t) -> qboolean;
    fn GetTime(lastTime: c_int) -> c_int;
    fn NPC_AimAdjust(change: c_int);
    fn FlyingCreature(ent: *mut gentity_t) -> qboolean;
    fn NPC_EvasionSaber();
    fn RT_Flying(self_: *mut gentity_t) -> qboolean;
    //extern	CNavigator	navigator;
    static mut d_asynchronousGroupAI: *mut cvar_t;
}

const MAX_VIEW_DIST: f32 = 1024.0;
const MAX_VIEW_SPEED: f32 = 250.0;
const MAX_LIGHT_INTENSITY: f32 = 255.0;
// 0.1 is a double literal in C
const MIN_LIGHT_THRESHOLD: f32 = 0.1;
const ST_MIN_LIGHT_THRESHOLD: c_int = 30;
const ST_MAX_LIGHT_THRESHOLD: c_int = 180;
const DISTANCE_THRESHOLD: f32 = 0.075;
// (100 squared) don't stop running backwards if your goal is less than 100 away
const MIN_TURN_AROUND_DIST_SQ: c_int = 10000;
const SABER_AVOID_DIST: f32 = 128.0; //256.0f
const SABER_AVOID_DIST_SQ: f32 = SABER_AVOID_DIST * SABER_AVOID_DIST;

//These first three get your base detection rating, ideally add up to 1
const DISTANCE_SCALE: f32 = 0.35; //
const FOV_SCALE: f32 = 0.40; //
const LIGHT_SCALE: f32 = 0.25; //

//These next two are bonuses
const SPEED_SCALE: f32 = 0.25; //
const TURNING_SCALE: f32 = 0.25; //

const REALIZE_THRESHOLD: f32 = 0.6;
// CAUTIOUS_THRESHOLD in C: ( REALIZE_THRESHOLD * 0.75 ) — 0.75 is a double literal,
// promoting to double; preserved as f32 here (no meaningful precision difference)
const CAUTIOUS_THRESHOLD: f32 = REALIZE_THRESHOLD * 0.75;

// Forward declaration present in C; not needed in Rust
// qboolean NPC_CheckPlayerTeamStealth( void );

// File-local static globals (C static = BSS zero-initialized)
// Porting note: C variable 'move' renamed to 'move_' because 'move' is a Rust keyword.
static mut enemyLOS: qboolean = qfalse;
static mut enemyCS: qboolean = qfalse;
static mut enemyInFOV: qboolean = qfalse;
static mut hitAlly: qboolean = qfalse;
static mut faceEnemy: qboolean = qfalse;
static mut move_: qboolean = qfalse;
static mut shoot: qboolean = qfalse;
static mut enemyDist: f32 = 0.0_f32;
static mut impactPos: vec3_t = [0.0_f32; 3];

pub static mut groupSpeechDebounceTime: [c_int; TEAM_NUM_TEAMS as usize] =
    [0; TEAM_NUM_TEAMS as usize]; //used to stop several group AI from speaking all at once

pub unsafe fn NPC_Saboteur_Precache() {
    G_SoundIndex(b"sound/chars/shadowtrooper/cloak.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/shadowtrooper/decloak.wav\0".as_ptr() as *const c_char);
}

// Porting note: called in this file with one arg (lines 97, 2629 in C++) implying a
// header default for uncloakTime.  We define it with two params; one-arg call sites
// use 0 as the default.
pub unsafe fn Saboteur_Decloak(self_: *mut gentity_t, uncloakTime: c_int) {
    if !self_.is_null() && !(*self_).client.is_null() {
        if (*(*self_).client).ps.powerups[PW_CLOAKED as usize] != 0
            && TIMER_Done(self_, b"decloakwait\0".as_ptr() as *const c_char) != 0
        {//Uncloak
            (*(*self_).client).ps.powerups[PW_CLOAKED as usize] = 0;
            (*(*self_).client).ps.powerups[PW_UNCLOAKING as usize] = (*level).time + 2000;
            //FIXME: temp sound
            G_SoundOnEnt(self_, CHAN_ITEM, b"sound/chars/shadowtrooper/decloak.wav\0".as_ptr() as *const c_char);
            TIMER_Set(self_, b"nocloak\0".as_ptr() as *const c_char, uncloakTime);

            // Can't Recloak
            //self->NPC->aiFlags	&= ~NPCAI_SHIELDS;
        }
    }
}

pub unsafe fn Saboteur_Cloak(self_: *mut gentity_t) {
    if !self_.is_null() && !(*self_).client.is_null() && !(*self_).NPC.is_null() {
        //FIXME: need to have this timer set once first?
        if TIMER_Done(self_, b"nocloak\0".as_ptr() as *const c_char) != 0 {
            //not sitting around waiting to cloak again
            if ((*(*self_).NPC).aiFlags & NPCAI_SHIELDS) == 0 {
                //not allowed to cloak, actually
                Saboteur_Decloak(self_, 0);
            } else if (*(*self_).client).ps.powerups[PW_CLOAKED as usize] == 0 {
                //cloak
                (*(*self_).client).ps.powerups[PW_CLOAKED as usize] = Q3_INFINITE;
                (*(*self_).client).ps.powerups[PW_UNCLOAKING as usize] = (*level).time + 2000;
                //FIXME: debounce attacks?
                //FIXME: temp sound
                G_SoundOnEnt(self_, CHAN_ITEM, b"sound/chars/shadowtrooper/cloak.wav\0".as_ptr() as *const c_char);
            }
        }
    }
}

//Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_UNDERFIRE: c_int = 1;
const LSTATE_INVESTIGATE: c_int = 2;

pub unsafe fn ST_AggressionAdjust(self_: *mut gentity_t, change: c_int) {
    let upper_threshold: c_int;
    let lower_threshold: c_int;

    (*(*self_).NPC).stats.aggression += change;

    //FIXME: base this on initial NPC stats
    if (*(*self_).client).playerTeam == TEAM_PLAYER {
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
    TIMER_Set(ent, b"chatter\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"duck\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"stand\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"shuffleTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"sleepTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"enemyLastVisible\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"roamTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"hideTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"attackDelay\0".as_ptr() as *const c_char, 0); //FIXME: Slant for difficulty levels
    TIMER_Set(ent, b"stick\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"scoutTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"flee\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"interrogating\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"verifyCP\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"strafeRight\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"strafeLeft\0".as_ptr() as *const c_char, 0);
}

const SPEECH_CHASE: c_int = 0;
const SPEECH_CONFUSED: c_int = 1;
const SPEECH_COVER: c_int = 2;
const SPEECH_DETECTED: c_int = 3;
const SPEECH_GIVEUP: c_int = 4;
const SPEECH_LOOK: c_int = 5;
const SPEECH_LOST: c_int = 6;
const SPEECH_OUTFLANK: c_int = 7;
const SPEECH_ESCAPING: c_int = 8;
const SPEECH_SIGHT: c_int = 9;
const SPEECH_SOUND: c_int = 10;
const SPEECH_SUSPICIOUS: c_int = 11;
const SPEECH_YELL: c_int = 12;
const SPEECH_PUSHED: c_int = 13;

unsafe fn ST_Speech(self_: *mut gentity_t, speechType: c_int, failChance: f32) {
    if random() < failChance {
        return;
    }

    if failChance >= 0.0 {
        //a negative failChance makes it always talk
        if !(*(*self_).NPC).group.is_null() {
            //group AI speech debounce timer
            if (*(*(*self_).NPC).group).speechDebounceTime > (*level).time {
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
        } else if TIMER_Done(self_, b"chatter\0".as_ptr() as *const c_char) == 0 {
            //personal timer
            return;
        } else if groupSpeechDebounceTime[(*(*self_).client).playerTeam as usize] > (*level).time {
            //for those not in group AI
            //FIXME: let certain speech types interrupt others?  Let closer NPCs interrupt farther away ones?
            return;
        }
    }

    if !(*(*self_).NPC).group.is_null() {
        //So they don't all speak at once...
        //FIXME: if they're not yet mad, they have no group, so distracting a group of them makes them all speak!
        (*(*(*self_).NPC).group).speechDebounceTime = (*level).time + Q_irand(2000, 4000);
    } else {
        TIMER_Set(self_, b"chatter\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
    }
    groupSpeechDebounceTime[(*(*self_).client).playerTeam as usize] =
        (*level).time + Q_irand(2000, 4000);

    if (*(*self_).NPC).blockedSpeechDebounceTime > (*level).time {
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

    (*(*self_).NPC).blockedSpeechDebounceTime = (*level).time + 2000;
}

pub unsafe fn ST_MarkToCover(self_: *mut gentity_t) {
    if self_.is_null() || (*self_).NPC.is_null() {
        return;
    }
    (*(*self_).NPC).localState = LSTATE_UNDERFIRE;
    TIMER_Set(self_, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
    ST_AggressionAdjust(self_, -3);
    if !(*(*self_).NPC).group.is_null() && (*(*(*self_).NPC).group).numGroup > 1 {
        ST_Speech(self_, SPEECH_COVER, 0.0); //FIXME: flee sound?
    }
}

pub unsafe fn ST_StartFlee(
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
    dangerPoint: vec3_t,
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
/*
-------------------------
NPC_ST_Pain
-------------------------
*/

pub unsafe fn NPC_ST_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    (*(*self_).NPC).localState = LSTATE_UNDERFIRE;

    TIMER_Set(self_, b"duck\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"hideTime\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"stand\0".as_ptr() as *const c_char, 2000);

    NPC_Pain(self_, inflictor, other, point, damage, mod_, hitLoc);

    if damage == 0 && (*self_).health > 0 {
        //FIXME: better way to know I was pushed
        G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
    }
}

/*
-------------------------
ST_HoldPosition
-------------------------
*/

unsafe fn ST_HoldPosition() {
    if (*NPCInfo).squadState == SQUAD_RETREAT {
        TIMER_Set(NPC, b"flee\0".as_ptr() as *const c_char, -(*level).time);
    }
    TIMER_Set(NPC, b"verifyCP\0".as_ptr() as *const c_char, Q_irand(1000, 3000)); //don't look for another one for a few seconds
    NPC_FreeCombatPoint((*NPCInfo).combatPoint, qtrue);
    //NPCInfo->combatPoint = -1;//???
    if Q3_TaskIDPending(NPC, TID_MOVE_NAV) == 0 {
        //don't have a script waiting for me to get to my point, okay to stop trying and stand
        AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, SQUAD_STAND_AND_SHOOT);
        (*NPCInfo).goalEntity = core::ptr::null_mut();
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
    {//imperial (commander) gives the order
        ST_Speech(
            (*(*NPCInfo).group).commander,
            (*NPCInfo).movementSpeech,
            (*NPCInfo).movementSpeechChance,
        );
    } else {
        //really don't want to say this unless we can actually get there...
        ST_Speech(NPC, (*NPCInfo).movementSpeech, (*NPCInfo).movementSpeechChance);
    }

    (*NPCInfo).movementSpeech = 0;
    (*NPCInfo).movementSpeechChance = 0.0_f32;
}

pub unsafe fn NPC_ST_StoreMovementSpeech(speech: c_int, chance: f32) {
    (*NPCInfo).movementSpeech = speech;
    (*NPCInfo).movementSpeechChance = chance;
}
/*
-------------------------
ST_Move
-------------------------
*/
// Forward declaration void ST_TransferMoveGoal( gentity_t *self, gentity_t *other ) not
// needed in Rust.
unsafe fn ST_Move() -> qboolean {
    (*NPCInfo).combatMove = qtrue; //always move straight toward our goal

    let moved: qboolean = NPC_MoveToGoal(qtrue);
    if moved == qfalse {
        ST_HoldPosition();
    }

    NPC_ST_SayMovementSpeech();

    moved
}

/*
-------------------------
NPC_ST_SleepShuffle
-------------------------
*/

unsafe fn NPC_ST_SleepShuffle() {
    //Play an awake script if we have one
    if G_ActivateBehavior(NPC, BSET_AWAKE) != 0 {
        return;
    }

    //Automate some movement and noise
    if TIMER_Done(NPC, b"shuffleTime\0".as_ptr() as *const c_char) != 0 {
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

        TIMER_Set(NPC, b"shuffleTime\0".as_ptr() as *const c_char, 4000);
        TIMER_Set(NPC, b"sleepTime\0".as_ptr() as *const c_char, 2000);
        return;
    }

    //They made another noise while we were stirring, see if we can see them
    if TIMER_Done(NPC, b"sleepTime\0".as_ptr() as *const c_char) != 0 {
        NPC_CheckPlayerTeamStealth();
        TIMER_Set(NPC, b"sleepTime\0".as_ptr() as *const c_char, 2000);
    }
}

/*
-------------------------
NPC_BSST_Sleep
-------------------------
*/

pub unsafe fn NPC_BSST_Sleep() {
    let alertEvent: c_int = NPC_CheckAlertEvents(qfalse, qtrue, -1, qfalse, AEL_MINOR); //only check sounds since we're alseep!

    //There is an event we heard
    if alertEvent >= 0 {
        //See if it was enough to wake us up
        if (*level).alertEvents[alertEvent as usize].level == AEL_DISCOVERED
            && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
        {
            if !g_entities.is_null() && (*g_entities.add(0)).health > 0 {
                G_SetEnemy(NPC, g_entities.add(0));
                return;
            }
        }

        //Otherwise just stir a bit
        NPC_ST_SleepShuffle();
        return;
    }
}

/*
-------------------------
NPC_CheckEnemyStealth
-------------------------
*/

pub unsafe fn NPC_CheckEnemyStealth(target: *mut gentity_t) -> qboolean {
    let mut target_dist: f32;
    let mut minDist: f32 = 40.0; //any closer than 40 and we definitely notice

    //In case we aquired one some other way
    if !(*NPC).enemy.is_null() {
        return qtrue;
    }

    //Ignore notarget
    if ((*target).flags & FL_NOTARGET) != 0 {
        return qfalse;
    }

    if (*target).health <= 0 {
        return qfalse;
    }

    if (*(*target).client).ps.weapon == WP_SABER
        && (*(*target).client).ps.SaberActive() != 0
        && (*(*target).client).ps.saberInFlight == 0
    {//if target has saber in hand and activated, we wake up even sooner even if not facing him
        minDist = 100.0;
    }

    target_dist = DistanceSquared((*target).currentOrigin, (*NPC).currentOrigin);
    //If the target is this close, then wake up regardless
    if ((*(*target).client).ps.pm_flags & PMF_DUCKED) == 0 //not ducking
        && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 //looking for enemies
        && target_dist < (minDist * minDist) //closer than minDist
    {
        G_SetEnemy(NPC, target);
        (*NPCInfo).enemyLastSeenTime = (*level).time;
        TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
        return qtrue;
    }

    let mut maxViewDist: f32 = MAX_VIEW_DIST;

    //	if ( NPCInfo->stats.visrange > maxViewDist )
    {//FIXME: should we always just set maxViewDist to this?
        maxViewDist = (*NPCInfo).stats.visrange;
    }

    if target_dist > (maxViewDist * maxViewDist) {
        //out of possible visRange
        return qfalse;
    }

    //Check FOV first
    if InFOV2(target, NPC, (*NPCInfo).stats.hfov, (*NPCInfo).stats.vfov) == qfalse {
        return qfalse;
    }

    let clearLOS: qboolean = if (*(*target).client).ps.leanofs != 0 {
        NPC_ClearLOS4((*(*target).client).renderInfo.eyePoint)
    } else {
        NPC_ClearLOS2(target)
    };

    //Now check for clear line of vision
    if clearLOS != 0 {
        if (*(*target).client).NPC_class == CLASS_ATST {
            //can't miss 'em!
            G_SetEnemy(NPC, target);
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
            return qtrue;
        }
        let targ_org: vec3_t = [
            (*target).currentOrigin[0],
            (*target).currentOrigin[1],
            (*target).currentOrigin[2] + (*target).maxs[2] - 4.0,
        ];
        let hAngle_perc: f32 = NPC_GetHFOVPercentage(
            targ_org,
            (*(*NPC).client).renderInfo.eyePoint,
            (*(*NPC).client).renderInfo.eyeAngles,
            (*NPCInfo).stats.hfov,
        );
        let vAngle_perc: f32 = NPC_GetVFOVPercentage(
            targ_org,
            (*(*NPC).client).renderInfo.eyePoint,
            (*(*NPC).client).renderInfo.eyeAngles,
            (*NPCInfo).stats.vfov,
        );

        //Scale them vertically some, and horizontally pretty harshly
        let vAngle_perc: f32 = vAngle_perc * vAngle_perc; //( vAngle_perc * vAngle_perc );
        let hAngle_perc: f32 = hAngle_perc * (hAngle_perc * hAngle_perc);

        //Cap our vertical vision severely
        //if ( vAngle_perc <= 0.3f ) // was 0.5f
        //	return qfalse;

        //Assess the player's current status
        target_dist = Distance((*target).currentOrigin, (*NPC).currentOrigin);

        let target_speed: f32 = VectorLength((*(*target).client).ps.velocity);
        let target_crouching: c_int = ((*(*target).client).usercmd.upmove < 0) as c_int;
        let dist_rating: f32 = target_dist / maxViewDist;
        let mut speed_rating: f32 = target_speed / MAX_VIEW_SPEED;
        let turning_rating: f32 = AngleDelta(
            (*(*target).client).ps.viewangles[PITCH as usize],
            (*target).lastAngles[PITCH as usize],
        ) / 180.0
            + AngleDelta(
                (*(*target).client).ps.viewangles[YAW as usize],
                (*target).lastAngles[YAW as usize],
            ) / 180.0;
        let light_level: f32 = (*target).lightLevel / MAX_LIGHT_INTENSITY;
        let FOV_perc: f32 = 1.0 - (hAngle_perc + vAngle_perc) * 0.5; //FIXME: Dunno about the average...
        let mut vis_rating: f32 = 0.0;

        //Too dark
        if light_level < MIN_LIGHT_THRESHOLD {
            return qfalse;
        }

        //Too close?
        if dist_rating < DISTANCE_THRESHOLD {
            G_SetEnemy(NPC, target);
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
            return qtrue;
        }

        //Out of range
        if dist_rating > 1.0 {
            return qfalse;
        }

        //Cap our speed checks
        if speed_rating > 1.0 {
            speed_rating = 1.0;
        }

        //Calculate the distance, fov and light influences
        //...Visibilty linearly wanes over distance
        let dist_influence: f32 = DISTANCE_SCALE * (1.0 - dist_rating);
        //...As the percentage out of the FOV increases, straight perception suffers on an exponential scale
        let fov_influence: f32 = FOV_SCALE * (1.0 - FOV_perc);
        //...Lack of light hides, abundance of light exposes
        let light_influence: f32 = (light_level - 0.5) * LIGHT_SCALE;

        //Calculate our base rating
        let mut target_rating: f32 = dist_influence + fov_influence + light_influence;

        //Now award any final bonuses to this number
        let contents: c_int = gi.pointcontents(targ_org, (*target).s.number);
        if (contents & CONTENTS_WATER) != 0 {
            let myContents: c_int =
                gi.pointcontents((*(*NPC).client).renderInfo.eyePoint, (*NPC).s.number);
            if (myContents & CONTENTS_WATER) == 0 {
                //I'm not in water
                if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {
                    //these guys can see in in/through water pretty well
                    vis_rating = 0.10; //10% bonus
                } else {
                    vis_rating = 0.35; //35% bonus
                }
            } else {
                //else, if we're both in water
                if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {
                    //I can see him just fine
                } else {
                    vis_rating = 0.15; //15% bonus
                }
            }
        } else {
            //not in water
            if (contents & CONTENTS_FOG) != 0 {
                vis_rating = 0.15; //15% bonus
            }
        }

        target_rating *= 1.0 - vis_rating;

        //...Motion draws the eye quickly
        target_rating += speed_rating * SPEED_SCALE;
        target_rating += turning_rating * TURNING_SCALE;
        //FIXME: check to see if they're animating, too?  But can we do something as simple as frame != oldframe?

        //...Smaller targets are harder to indentify
        if target_crouching != 0 {
            target_rating *= 0.9; //10% bonus
        }

        //If he's violated the threshold, then realize him
        //float difficulty_scale = 1.0f + (2.0f-g_spskill->value);//if playing on easy, 20% harder to be seen...?
        let realize: f32;
        let cautious: f32;
        if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {
            //swamptroopers can see much better
            realize = CAUTIOUS_THRESHOLD as f32; /* *difficulty_scale */
            cautious = CAUTIOUS_THRESHOLD as f32 * 0.75; /* *difficulty_scale */
        } else {
            realize = REALIZE_THRESHOLD as f32; /* *difficulty_scale */
            cautious = CAUTIOUS_THRESHOLD as f32 * 0.75; /* *difficulty_scale */
        }

        if target_rating > realize && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            G_SetEnemy(NPC, target);
            (*NPCInfo).enemyLastSeenTime = (*level).time;
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
            return qtrue;
        }

        //If he's above the caution threshold, then realize him in a few seconds unless he moves to cover
        if target_rating > cautious && ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
            //FIXME: ambushing guys should never talk
            if TIMER_Done(NPC, b"enemyLastVisible\0".as_ptr() as *const c_char) != 0 {
                //If we haven't already, start the counter
                let lookTime: c_int = Q_irand(4500, 8500);
                //NPCInfo->timeEnemyLastVisible = level.time + 2000;
                TIMER_Set(NPC, b"enemyLastVisible\0".as_ptr() as *const c_char, lookTime);
                //TODO: Play a sound along the lines of, "Huh?  What was that?"
                ST_Speech(NPC, SPEECH_SIGHT, 0.0);
                NPC_TempLookTarget(NPC, (*target).s.number, lookTime, lookTime);
                //FIXME: set desired yaw and pitch towards this guy?
            } else if TIMER_Get(NPC, b"enemyLastVisible\0".as_ptr() as *const c_char)
                <= (*level).time + 500
                && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
            //FIXME: Is this reliable?
            {
                if (*NPCInfo).rank < RANK_LT && Q_irand(0, 2) == 0 {
                    let interrogateTime: c_int = Q_irand(2000, 4000);
                    ST_Speech(NPC, SPEECH_SUSPICIOUS, 0.0);
                    TIMER_Set(NPC, b"interrogating\0".as_ptr() as *const c_char, interrogateTime);
                    G_SetEnemy(NPC, target);
                    (*NPCInfo).enemyLastSeenTime = (*level).time;
                    TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, interrogateTime);
                    TIMER_Set(NPC, b"stand\0".as_ptr() as *const c_char, interrogateTime);
                } else {
                    G_SetEnemy(NPC, target);
                    (*NPCInfo).enemyLastSeenTime = (*level).time;
                    //FIXME: ambush guys (like those popping out of water) shouldn't delay...
                    TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
                    TIMER_Set(NPC, b"stand\0".as_ptr() as *const c_char, Q_irand(500, 2500));
                }
                return qtrue;
            }

            return qfalse;
        }
    }

    qfalse
}

pub unsafe fn NPC_CheckPlayerTeamStealth() -> qboolean {
    /*
    //NOTENOTE: For now, all stealh checks go against the player, since
    //			he is the main focus.  Squad members and rivals do not
    //			fall into this category and will be ignored.

    NPC_CheckEnemyStealth( &g_entities[0] );	//Change this pointer to assess other entities
    */
    let mut enemy: *mut gentity_t;
    let mut i: c_int = 0;
    while i < ENTITYNUM_WORLD {
        if PInUse(i) == 0 {
            i += 1;
            continue;
        }
        enemy = g_entities.add(i as usize);
        if !enemy.is_null()
            && !(*enemy).client.is_null()
            && NPC_ValidEnemy(enemy) != 0
        {
            if NPC_CheckEnemyStealth(enemy) != 0 //Change this pointer to assess other entities
            {
                return qtrue;
            }
        }
        i += 1;
    }
    qfalse
}

pub unsafe fn NPC_CheckEnemiesInSpotlight() -> qboolean {
    let mut entityList: [*mut gentity_t; MAX_GENTITIES as usize] =
        [core::ptr::null_mut(); MAX_GENTITIES as usize];
    let mut enemy: *mut gentity_t;
    let mut suspect: *mut gentity_t = core::ptr::null_mut();
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    let mut i: c_int = 0;
    while i < 3 {
        mins[i as usize] = (*(*NPC).client).renderInfo.eyePoint[i as usize] - (*NPC).speed;
        maxs[i as usize] = (*(*NPC).client).renderInfo.eyePoint[i as usize] + (*NPC).speed;
        i += 1;
    }

    let numListedEntities: c_int =
        gi.EntitiesInBox(mins, maxs, entityList.as_mut_ptr(), MAX_GENTITIES);

    let mut i: c_int = 0;
    while i < numListedEntities {
        if PInUse(i) == 0 {
            i += 1;
            continue;
        }

        enemy = entityList[i as usize];

        if !enemy.is_null()
            && !(*enemy).client.is_null()
            && NPC_ValidEnemy(enemy) != 0
            && (*(*enemy).client).playerTeam == (*(*NPC).client).enemyTeam
        {//valid ent & client, valid enemy, on the target team
            //check to see if they're in my FOV
            if InFOV3(
                (*enemy).currentOrigin,
                (*(*NPC).client).renderInfo.eyePoint,
                (*(*NPC).client).renderInfo.eyeAngles,
                (*NPCInfo).stats.hfov,
                (*NPCInfo).stats.vfov,
            ) != 0
            {//in my cone
                //check to see that they're close enough
                if DistanceSquared((*(*NPC).client).renderInfo.eyePoint, (*enemy).currentOrigin)
                    - 256.0 /*fudge factor: 16 squared*/
                    <= (*NPC).speed * (*NPC).speed
                {//within range
                    //check to see if we have a clear trace to them
                    if G_ClearLOS(NPC, enemy) != 0 {
                        //clear LOS
                        //make sure their light level is at least my beam's brightness
                        //FIXME: HOW?
                        //enemy->lightLevel / MAX_LIGHT_INTENSITY

                        //good enough, take him!
                        //FIXME: pick closest one?
                        //FIXME: have the graduated noticing like other NPCs? (based on distance, FOV dot, etc...)
                        G_SetEnemy(NPC, enemy);
                        TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
                        return qtrue;
                    }
                }
            }
            if InFOV3(
                (*enemy).currentOrigin,
                (*(*NPC).client).renderInfo.eyePoint,
                (*(*NPC).client).renderInfo.eyeAngles,
                90,
                (*NPCInfo).stats.vfov * 3,
            ) != 0
            {//one to look at if we don't get an enemy
                if G_ClearLOS(NPC, enemy) != 0 {
                    //clear LOS
                    if suspect.is_null()
                        || DistanceSquared(
                            (*(*NPC).client).renderInfo.eyePoint,
                            (*enemy).currentOrigin,
                        ) < DistanceSquared(
                            (*(*NPC).client).renderInfo.eyePoint,
                            (*suspect).currentOrigin,
                        )
                    {//remember him
                        suspect = enemy;
                    }
                }
            }
        }
        i += 1;
    }
    if !suspect.is_null()
        && Q_flrand(
            0.0,
            (*NPCInfo).stats.visrange * (*NPCInfo).stats.visrange,
        ) > DistanceSquared(
            (*(*NPC).client).renderInfo.eyePoint,
            (*suspect).currentOrigin,
        )
    {//hey!  who's that?
        if TIMER_Done(NPC, b"enemyLastVisible\0".as_ptr() as *const c_char) != 0 {
            //If we haven't already, start the counter
            let lookTime: c_int = Q_irand(4500, 8500);
            //NPCInfo->timeEnemyLastVisible = level.time + 2000;
            TIMER_Set(NPC, b"enemyLastVisible\0".as_ptr() as *const c_char, lookTime);
            //TODO: Play a sound along the lines of, "Huh?  What was that?"
            ST_Speech(NPC, SPEECH_SIGHT, 0.0);
            //set desired yaw and pitch towards this guy?
            //FIXME: this is permanent, they will never look away... *sigh*
            NPC_FacePosition((*suspect).currentOrigin, qtrue);
            //FIXME: they still need some sort of eye/head tag/bone that can turn?
            //NPC_TempLookTarget( NPC, suspect->s.number, lookTime, lookTime );
        } else if TIMER_Get(NPC, b"enemyLastVisible\0".as_ptr() as *const c_char)
            <= (*level).time + 500
            && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
        //FIXME: Is this reliable?
        {
            if Q_irand(0, 2) == 0 {
                let interrogateTime: c_int = Q_irand(2000, 4000);
                ST_Speech(NPC, SPEECH_SUSPICIOUS, 0.0);
                TIMER_Set(NPC, b"interrogating\0".as_ptr() as *const c_char, interrogateTime);
                //G_SetEnemy( NPC, target );
                //NPCInfo->enemyLastSeenTime = level.time;
                //TIMER_Set( NPC, "attackDelay", interrogateTime );
                //TIMER_Set( NPC, "stand", interrogateTime );
                //set desired yaw and pitch towards this guy?
                //FIXME: this is permanent, they will never look away... *sigh*
                NPC_FacePosition((*suspect).currentOrigin, qtrue);
                //FIXME: they still need some sort of eye/head tag/bone that can turn?
                //NPC_TempLookTarget( NPC, suspect->s.number, interrogateTime, interrogateTime );
            }
        }
    }
    qfalse
}

/*
-------------------------
NPC_ST_InvestigateEvent
-------------------------
*/

const MAX_CHECK_THRESHOLD: c_int = 1;

// Porting note: original C++ uses 'bool extraSuspicious'; called with qboolean values
// (qtrue/qfalse).  Parameter kept as qboolean for ABI consistency at call sites.
unsafe fn NPC_ST_InvestigateEvent(eventID: c_int, extraSuspicious: qboolean) -> qboolean {
    //If they've given themselves away, just take them as an enemy
    if (*NPCInfo).confusionTime < (*level).time {
        if (*level).alertEvents[eventID as usize].level == AEL_DISCOVERED
            && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
        {
            //NPCInfo->lastAlertID = level.alertEvents[eventID].ID;
            if (*level).alertEvents[eventID as usize].owner.is_null()
                || (*(*level).alertEvents[eventID as usize].owner).client.is_null()
                || (*(*level).alertEvents[eventID as usize].owner).health <= 0
                || (*(*(*level).alertEvents[eventID as usize].owner).client).playerTeam
                    != (*(*NPC).client).enemyTeam
            {//not an enemy
                return qfalse;
            }
            //FIXME: what if can't actually see enemy, don't know where he is... should we make them just become very alert and start looking for him?  Or just let combat AI handle this... (act as if you lost him)
            //ST_Speech( NPC, SPEECH_CHARGE, 0 );
            G_SetEnemy(NPC, (*level).alertEvents[eventID as usize].owner);
            (*NPCInfo).enemyLastSeenTime = (*level).time;
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
            if (*level).alertEvents[eventID as usize].kind == AET_SOUND {
                //heard him, didn't see him, stick for a bit
                TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const c_char, Q_irand(500, 2500));
            }
            return qtrue;
        }
    }

    //don't look at the same alert twice
    /*
    if ( level.alertEvents[eventID].ID == NPCInfo->lastAlertID )
    {
        return qfalse;
    }
    NPCInfo->lastAlertID = level.alertEvents[eventID].ID;
    */

    //Must be ready to take another sound event
    /*
    if ( NPCInfo->investigateSoundDebounceTime > level.time )
    {
        return qfalse;
    }
    */

    if (*level).alertEvents[eventID as usize].kind == AET_SIGHT {
        //sight alert, check the light level
        if (*level).alertEvents[eventID as usize].light
            < Q_irand(ST_MIN_LIGHT_THRESHOLD, ST_MAX_LIGHT_THRESHOLD)
        {//below my threshhold of potentially seeing
            return qfalse;
        }
    }

    //Save the position for movement (if necessary)
    VectorCopy(
        (*level).alertEvents[eventID as usize].position,
        (*NPCInfo).investigateGoal,
    );

    //First awareness of it
    (*NPCInfo).investigateCount += if extraSuspicious != 0 { 2 } else { 1 };

    //Clamp the value
    if (*NPCInfo).investigateCount > 4 {
        (*NPCInfo).investigateCount = 4;
    }

    //See if we should walk over and investigate
    if (*level).alertEvents[eventID as usize].level > AEL_MINOR
        && (*NPCInfo).investigateCount > 1
        && ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0
    {
        //make it so they can walk right to this point and look at it rather than having to use combatPoints
        if G_ExpandPointToBBox(
            (*NPCInfo).investigateGoal,
            (*NPC).mins.as_ptr(),
            (*NPC).maxs.as_ptr(),
            (*NPC).s.number,
            ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
        ) != 0
        {//we were able to move the investigateGoal to a point in which our bbox would fit
            //drop the goal to the ground so we can get at it
            let mut end: vec3_t = [0.0; 3];
            let mut trace: trace_t = core::mem::zeroed();
            VectorCopy((*NPCInfo).investigateGoal, end);
            end[2] -= 512.0; //FIXME: not always right?  What if it's even higher, somehow?
            gi.trace(
                &mut trace,
                (*NPCInfo).investigateGoal,
                (*NPC).mins,
                (*NPC).maxs,
                end,
                ENTITYNUM_NONE,
                ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
            );
            if trace.fraction >= 1.0 {
                //too high to even bother
                //FIXME: look at them???
            } else {
                VectorCopy(trace.endpos, (*NPCInfo).investigateGoal);
                NPC_SetMoveGoal(NPC, (*NPCInfo).investigateGoal, 16, qtrue, -1, core::ptr::null_mut());
                (*NPCInfo).localState = LSTATE_INVESTIGATE;
            }
        } else {
            let id: c_int = NPC_FindCombatPoint(
                (*NPCInfo).investigateGoal,
                (*NPCInfo).investigateGoal,
                (*NPCInfo).investigateGoal,
                CP_INVESTIGATE | CP_HAS_ROUTE,
                0,
                -1,
            );

            if id != -1 {
                NPC_SetMoveGoal(
                    NPC,
                    (*level).combatPoints[id as usize].origin,
                    16,
                    qtrue,
                    id,
                    core::ptr::null_mut(),
                );
                (*NPCInfo).localState = LSTATE_INVESTIGATE;
            }
        }
        //Say something
        //FIXME: only if have others in group... these should be responses?
        if (*NPCInfo).investigateDebounceTime + (*NPCInfo).pauseTime > (*level).time {
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
            if (*level).alertEvents[eventID as usize].kind == AET_SIGHT {
                ST_Speech(NPC, SPEECH_SIGHT, 0.0);
            } else if (*level).alertEvents[eventID as usize].kind == AET_SOUND {
                ST_Speech(NPC, SPEECH_SOUND, 0.0);
            }
        }
        //Setup the debounce info
        (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 5000;
        (*NPCInfo).investigateSoundDebounceTime = (*level).time + 2000;
        (*NPCInfo).pauseTime = (*level).time;
    } else {
        //just look?
        //Say something
        if (*level).alertEvents[eventID as usize].kind == AET_SIGHT {
            ST_Speech(NPC, SPEECH_SIGHT, 0.0);
        } else if (*level).alertEvents[eventID as usize].kind == AET_SOUND {
            ST_Speech(NPC, SPEECH_SOUND, 0.0);
        }
        //Setup the debounce info
        (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 1000;
        (*NPCInfo).investigateSoundDebounceTime = (*level).time + 1000;
        (*NPCInfo).pauseTime = (*level).time;
        VectorCopy(
            (*level).alertEvents[eventID as usize].position,
            (*NPCInfo).investigateGoal,
        );
        if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER && RT_Flying(NPC) == 0 {
            //if ( !Q_irand( 0, 2 ) )
            {//look around
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_GUARD_LOOKAROUND1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }
        }
    }

    if (*level).alertEvents[eventID as usize].level >= AEL_DANGER {
        (*NPCInfo).investigateDebounceTime = Q_irand(500, 2500);
    }

    //Start investigating
    (*NPCInfo).tempBehavior = BS_INVESTIGATE;
    qtrue
}

/*
-------------------------
ST_OffsetLook
-------------------------
*/

unsafe fn ST_OffsetLook(offset: f32, out: *mut vec3_t) {
    let mut angles: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut temp: vec3_t = [0.0; 3];

    GetAnglesForDirection((*NPC).currentOrigin, (*NPCInfo).investigateGoal, angles);
    (*out)[YAW as usize] += offset;
    AngleVectors(angles, forward.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut());
    VectorMA((*NPC).currentOrigin, 64.0, forward, *out);

    CalcEntitySpot(NPC, SPOT_HEAD, temp);
    (*out)[2] = temp[2];
}

/*
-------------------------
ST_LookAround
-------------------------
*/

unsafe fn ST_LookAround() {
    let mut lookPos: vec3_t = [0.0; 3];
    let perc: f32 = ((*level).time - (*NPCInfo).pauseTime) as f32
        / (*NPCInfo).investigateDebounceTime as f32;

    //Keep looking at the spot
    if perc < 0.25 {
        VectorCopy((*NPCInfo).investigateGoal, lookPos);
    } else if perc < 0.5 {
        //Look up but straight ahead
        ST_OffsetLook(0.0, &mut lookPos);
    } else if perc < 0.75 {
        //Look right
        ST_OffsetLook(45.0, &mut lookPos);
    } else {
        //Look left
        ST_OffsetLook(-45.0, &mut lookPos);
    }

    NPC_FacePosition(lookPos, qfalse);
}

/*
-------------------------
NPC_BSST_Investigate
-------------------------
*/

pub unsafe fn NPC_BSST_Investigate() {
    //get group- mainly for group speech debouncing, but may use for group scouting/investigating AI, too
    AI_GetGroup(NPC);

    if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) != 0 {
        WeaponThink(qtrue);
    }

    if (*NPCInfo).confusionTime < (*level).time {
        if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            //Look for an enemy
            if NPC_CheckPlayerTeamStealth() != 0 {
                //NPCInfo->behaviorState	= BS_HUNT_AND_KILL;//should be auto now
                ST_Speech(NPC, SPEECH_DETECTED, 0.0);
                (*NPCInfo).tempBehavior = BS_DEFAULT;
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }
    }

    if ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
        let alertEvent: c_int = NPC_CheckAlertEvents(qtrue, qtrue, (*NPCInfo).lastAlertID, qfalse, AEL_MINOR);

        //There is an event to look at
        if alertEvent >= 0 {
            if (*NPCInfo).confusionTime < (*level).time {
                if NPC_CheckForDanger(alertEvent) != 0 {
                    //running like hell
                    ST_Speech(NPC, SPEECH_COVER, 0.0); //FIXME: flee sound?
                    return;
                }
            }

            //if ( level.alertEvents[alertEvent].ID != NPCInfo->lastAlertID )
            {
                NPC_ST_InvestigateEvent(alertEvent, qtrue);
            }
        }
    }

    //If we're done looking, then just return to what we were doing
    if ((*NPCInfo).investigateDebounceTime + (*NPCInfo).pauseTime) < (*level).time {
        (*NPCInfo).tempBehavior = BS_DEFAULT;
        (*NPCInfo).goalEntity = UpdateGoal();

        NPC_UpdateAngles(qtrue, qtrue);
        //Say something
        ST_Speech(NPC, SPEECH_GIVEUP, 0.0);
        return;
    }

    //FIXME: else, look for new alerts

    //See if we're searching for the noise's origin
    if (*NPCInfo).localState == LSTATE_INVESTIGATE && !(*NPCInfo).goalEntity.is_null() {
        //See if we're there
        if STEER::Reached(NPC, (*NPCInfo).goalEntity, 32, (FlyingCreature(NPC) != 0) as qboolean)
            == 0
        {
            (*ucmd).buttons |= BUTTON_WALKING;

            //Try and move there
            if NPC_MoveToGoal(qtrue) != 0 {
                //Bump our times
                (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 5000;
                (*NPCInfo).pauseTime = (*level).time;

                NPC_UpdateAngles(qtrue, qtrue);
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

/*
-------------------------
NPC_BSST_Patrol
-------------------------
*/

pub unsafe fn NPC_BSST_Patrol() {
    //FIXME: pick up on bodies of dead buddies?

    //Not a scriptflag, but...
    if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
        && ((*(*NPC).client).ps.eFlags & EF_SPOTLIGHT) != 0
    {//using spotlight search mode
        let mut eyeFwd: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mins: vec3_t = [-2.0, -2.0, -2.0];
        let maxs: vec3_t = [2.0, 2.0, 2.0];
        let mut trace: trace_t = core::mem::zeroed();
        AngleVectors(
            (*(*NPC).client).renderInfo.eyeAngles,
            eyeFwd.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        VectorMA(
            (*(*NPC).client).renderInfo.eyePoint,
            (*NPCInfo).stats.visrange,
            eyeFwd,
            end,
        );
        //get server-side trace impact point
        gi.trace(
            &mut trace,
            (*(*NPC).client).renderInfo.eyePoint,
            mins,
            maxs,
            end,
            (*NPC).s.number,
            MASK_OPAQUE | CONTENTS_BODY | CONTENTS_CORPSE,
        );
        (*NPC).speed = trace.fraction * (*NPCInfo).stats.visrange;
        if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            //FIXME: do a FOV cone check, then a trace
            if trace.entityNum < ENTITYNUM_WORLD {
                //hit something
                //try cheap check first
                let enemy: *mut gentity_t = g_entities.add(trace.entityNum as usize);
                if !enemy.is_null()
                    && !(*enemy).client.is_null()
                    && NPC_ValidEnemy(enemy) != 0
                    && (*(*enemy).client).playerTeam == (*(*NPC).client).enemyTeam
                {
                    G_SetEnemy(NPC, enemy);
                    TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
                    //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                    //NPC_AngerSound();
                    NPC_UpdateAngles(qtrue, qtrue);
                    return;
                }
            }
            //FIXME: maybe do a quick check of ents within the spotlight's radius?
            //hmmm, look around
            if NPC_CheckEnemiesInSpotlight() != 0 {
                //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                //NPC_AngerSound();
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }
    } else {
        //get group- mainly for group speech debouncing, but may use for group scouting/investigating AI, too
        AI_GetGroup(NPC);

        if (*NPCInfo).confusionTime < (*level).time {
            //Look for any enemies
            if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                if NPC_CheckPlayerTeamStealth() != 0 {
                    //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                    //NPC_AngerSound();
                    NPC_UpdateAngles(qtrue, qtrue);
                    return;
                }
            }
        }
    }

    if ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
        let alertEvent: c_int = NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_MINOR);

        //There is an event to look at
        if alertEvent >= 0 {
            if NPC_CheckForDanger(alertEvent) != 0 {
                //going to run?
                ST_Speech(NPC, SPEECH_COVER, 0.0);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                //NPCInfo->lastAlertID = level.alertEvents[eventID].ID;
                if (*level).alertEvents[alertEvent as usize].owner.is_null()
                    || (*(*level).alertEvents[alertEvent as usize].owner).client.is_null()
                    || (*(*level).alertEvents[alertEvent as usize].owner).health <= 0
                    || (*(*(*level).alertEvents[alertEvent as usize].owner).client).playerTeam
                        != (*(*NPC).client).enemyTeam
                {//not an enemy
                    return;
                }
                //FIXME: what if can't actually see enemy, don't know where he is... should we make them just become very alert and start looking for him?  Or just let combat AI handle this... (act as if you lost him)
                //ST_Speech( NPC, SPEECH_CHARGE, 0 );
                G_SetEnemy(NPC, (*level).alertEvents[alertEvent as usize].owner);
                (*NPCInfo).enemyLastSeenTime = (*level).time;
                TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2500));
                return;
            } else if NPC_ST_InvestigateEvent(alertEvent, qfalse) != 0 {
                //actually going to investigate it
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }
    }

    //If we have somewhere to go, then do that
    if UpdateGoal() != 0 {
        (*ucmd).buttons |= BUTTON_WALKING;
        //ST_Move( NPCInfo->goalEntity );
        NPC_MoveToGoal(qtrue);
    } else {
        // else if ( !(NPCInfo->scriptFlags&SCF_IGNORE_ALERTS) )
        if (*(*NPC).client).NPC_class != CLASS_IMPERIAL
            && (*(*NPC).client).NPC_class != CLASS_IMPWORKER
        {//imperials do not look around
            if TIMER_Done(NPC, b"enemyLastVisible\0".as_ptr() as *const c_char) != 0 {
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

    NPC_UpdateAngles(qtrue, qtrue);
    //TEMP hack for Imperial stand anim
    if (*(*NPC).client).NPC_class == CLASS_IMPERIAL
        || (*(*NPC).client).NPC_class == CLASS_IMPWORKER
    {//hack
        if (*(*NPC).client).ps.weapon != WP_CONCUSSION {
            //not Rax
            if (*ucmd).forwardmove != 0 || (*ucmd).rightmove != 0 || (*ucmd).upmove != 0 {
                //moving

                if (*(*NPC).client).ps.torsoAnimTimer == 0
                    || (*(*NPC).client).ps.torsoAnim == BOTH_STAND4
                {
                    if ((*ucmd).buttons & BUTTON_WALKING) != 0
                        && ((*NPCInfo).scriptFlags & SCF_RUNNING) == 0
                    {//not running, only set upper anim
                        //  No longer overrides scripted anims
                        NPC_SetAnim(
                            NPC,
                            SETANIM_TORSO,
                            BOTH_STAND4,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                        (*(*NPC).client).ps.torsoAnimTimer = 200;
                    }
                }
            } else {
                //standing still, set both torso and legs anim
                //  No longer overrides scripted anims
                if ((*(*NPC).client).ps.torsoAnimTimer == 0
                    || (*(*NPC).client).ps.torsoAnim == BOTH_STAND4)
                    && ((*(*NPC).client).ps.legsAnimTimer == 0
                        || (*(*NPC).client).ps.legsAnim == BOTH_STAND4)
                {
                    NPC_SetAnim(
                        NPC,
                        SETANIM_BOTH,
                        BOTH_STAND4,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    (*(*NPC).client).ps.torsoAnimTimer = 200;
                    (*(*NPC).client).ps.legsAnimTimer = 200;
                }
                //FIXME: this is a disgusting hack that is supposed to make the Imperials start with their weapon holstered- need a better way
                if (*(*NPC).client).ps.weapon != WP_NONE {
                    ChangeWeapon(NPC, WP_NONE);
                    (*(*NPC).client).ps.weapon = WP_NONE;
                    (*(*NPC).client).ps.weaponstate = WEAPON_READY;
                    G_RemoveWeaponModels(NPC);
                }
            }
        }
    }
}

/*
-------------------------
NPC_BSST_Idle
-------------------------
*/
/*
void NPC_BSST_Idle( void )
{
	int alertEvent = NPC_CheckAlertEvents( qtrue, qtrue );

	//There is an event to look at
	if ( alertEvent >= 0 )
	{
		NPC_ST_InvestigateEvent( alertEvent, qfalse );
		NPC_UpdateAngles( qtrue, qtrue );
		return;
	}

	TIMER_Set( NPC, "roamTime", 2000 + Q_irand( 1000, 2000 ) );

	NPC_UpdateAngles( qtrue, qtrue );
}
*/
/*
-------------------------
ST_CheckMoveState
-------------------------
*/

unsafe fn ST_CheckMoveState() {
    if Q3_TaskIDPending(NPC, TID_MOVE_NAV) != 0 {
        //moving toward a goal that a script is waiting on, so don't stop for anything!
        move_ = qtrue;
    } else if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
        && (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE
    {//no squad stuff
        return;
    }
    //	else if ( NPC->NPC->scriptFlags&SCF_NO_GROUPS )
    {
        move_ = qtrue;
    }
    //See if we're a scout

    //See if we're moving towards a goal, not the enemy
    if (*NPCInfo).goalEntity != (*NPC).enemy && !(*NPCInfo).goalEntity.is_null() {
        //Did we make it?
        if STEER::Reached(NPC, (*NPCInfo).goalEntity, 16, (FlyingCreature(NPC) != 0) as qboolean)
            != 0
            || (enemyLOS != 0
                && ((*NPCInfo).aiFlags & NPCAI_STOP_AT_LOS) != 0
                && Q3_TaskIDPending(NPC, TID_MOVE_NAV) == 0)
        {//either hit our navgoal or our navgoal was not a crucial (scripted) one (maybe a combat point) and we're scouting and found our enemy
            let mut newSquadState: c_int = SQUAD_STAND_AND_SHOOT;
            //we got where we wanted to go, set timers based on why we were running
            match (*NPCInfo).squadState {
                SQUAD_RETREAT => {
                    //was running away
                    //done fleeing, obviously
                    TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, ((*NPC).max_health - (*NPC).health) * 100);
                    TIMER_Set(NPC, b"hideTime\0".as_ptr() as *const c_char, Q_irand(3000, 7000));
                    TIMER_Set(NPC, b"flee\0".as_ptr() as *const c_char, -(*level).time);
                    newSquadState = SQUAD_COVER;
                }
                SQUAD_TRANSITION => {
                    //was heading for a combat point
                    TIMER_Set(NPC, b"hideTime\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
                }
                SQUAD_SCOUT => {
                    //was running after player
                }
                _ => {}
            }
            AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, newSquadState);
            NPC_ReachedGoal();
            //don't attack right away
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(250, 500)); //FIXME: Slant for difficulty levels
            //don't do something else just yet

            // THIS IS THE ONE TRUE PLACE WHERE ROAM TIME IS SET
            TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const c_char, Q_irand(8000, 15000)); //Q_irand( 1000, 4000 ) );
            if Q_irand(0, 3) == 0 {
                TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, Q_irand(5000, 10000)); // just reached our goal, chance of ducking now
            }
            return;
        }

        //keep going, hold of roamTimer until we get there
        TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const c_char, Q_irand(8000, 9000));
    }
}

pub unsafe fn ST_ResolveBlockedShot(hit: c_int) {
    let stuckTime: c_int;
    //figure out how long we intend to stand here, max
    if TIMER_Get(NPC, b"roamTime\0".as_ptr() as *const c_char)
        > TIMER_Get(NPC, b"stick\0".as_ptr() as *const c_char)
    {
        stuckTime = TIMER_Get(NPC, b"roamTime\0".as_ptr() as *const c_char) - (*level).time;
    } else {
        stuckTime = TIMER_Get(NPC, b"stick\0".as_ptr() as *const c_char) - (*level).time;
    }

    if TIMER_Done(NPC, b"duck\0".as_ptr() as *const c_char) != 0 {
        //we're not ducking
        if AI_GroupContainsEntNum((*NPCInfo).group, hit) != 0 {
            let member: *mut gentity_t = g_entities.add(hit as usize);
            if TIMER_Done(member, b"duck\0".as_ptr() as *const c_char) != 0 {
                //they aren't ducking
                if TIMER_Done(member, b"stand\0".as_ptr() as *const c_char) != 0 {
                    //they're not being forced to stand
                    //tell them to duck at least as long as I'm not moving
                    TIMER_Set(member, b"duck\0".as_ptr() as *const c_char, stuckTime); // tell my friend to duck so I can shoot over his head
                    return;
                }
            }
        }
    } else {
        //maybe we should stand
        if TIMER_Done(NPC, b"stand\0".as_ptr() as *const c_char) != 0 {
            //stand for as long as we'll be here
            TIMER_Set(NPC, b"stand\0".as_ptr() as *const c_char, stuckTime);
            return;
        }
    }
    //Hmm, can't resolve this by telling them to duck or telling me to stand
    //We need to move!
    TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const c_char, -1);
    TIMER_Set(NPC, b"stick\0".as_ptr() as *const c_char, -1);
    TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, -1);
    TIMER_Set(NPC, b"attakDelay\0".as_ptr() as *const c_char, Q_irand(1000, 3000));
}

/*
-------------------------
ST_CheckFireState
-------------------------
*/

unsafe fn ST_CheckFireState() {
    if enemyCS != 0 {
        //if have a clear shot, always try
        return;
    }

    if (*NPCInfo).squadState == SQUAD_RETREAT
        || (*NPCInfo).squadState == SQUAD_TRANSITION
        || (*NPCInfo).squadState == SQUAD_SCOUT
    {//runners never try to fire at the last pos
        return;
    }

    if !VectorCompare((*(*NPC).client).ps.velocity, vec3_origin) != 0 {
        //if moving at all, don't do this
        return;
    }

    //See if we should continue to fire on their last position
    // !TIMER_Done( NPC, "stick" ) ||
    if hitAlly == 0 //we're not going to hit an ally
        && enemyInFOV != 0 //enemy is in our FOV //FIXME: or we don't have a clear LOS?
        && (*NPCInfo).enemyLastSeenTime > 0 //we've seen the enemy
        && !(*NPCInfo).group.is_null() //have a group
        && ((*(*NPCInfo).group).numState[SQUAD_RETREAT as usize] > 0
            || (*(*NPCInfo).group).numState[SQUAD_TRANSITION as usize] > 0
            || (*(*NPCInfo).group).numState[SQUAD_SCOUT as usize] > 0)
    //laying down covering fire
    {
        if (*level).time - (*NPCInfo).enemyLastSeenTime < 10000 //we have seem the enemy in the last 10 seconds
            && ((*NPCInfo).group.is_null()
                || (*level).time - (*(*NPCInfo).group).lastSeenEnemyTime < 10000)
        //we are not in a group or the group has seen the enemy in the last 10 seconds
        {
            if Q_irand(0, 10) == 0 {
                //Fire on the last known position
                let mut muzzle: vec3_t = [0.0; 3];
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];
                let mut tooClose: qboolean = qfalse;
                let mut tooFar: qboolean = qfalse;

                CalcEntitySpot(NPC, SPOT_HEAD, muzzle);
                if VectorCompare(impactPos, vec3_origin) != 0 {
                    //never checked ShotEntity this frame, so must do a trace...
                    let mut tr: trace_t = core::mem::zeroed();
                    //vec3_t	mins = {-2,-2,-2}, maxs = {2,2,2};
                    let mut forward: vec3_t = [0.0; 3];
                    let mut end: vec3_t = [0.0; 3];
                    AngleVectors(
                        (*(*NPC).client).ps.viewangles,
                        forward.as_mut_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    VectorMA(muzzle, 8192.0, forward, end);
                    gi.trace(
                        &mut tr,
                        muzzle,
                        vec3_origin,
                        vec3_origin,
                        end,
                        (*NPC).s.number,
                        MASK_SHOT,
                    );
                    VectorCopy(tr.endpos, impactPos);
                }

                //see if impact would be too close to me
                let mut distThreshold: f32 = 16384.0; /*128*128*/ //default
                match (*NPC).s.weapon {
                    WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                    | WP_DET_PACK => {
                        distThreshold = 65536.0; /*256*256*/
                    }
                    WP_REPEATER => {
                        if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 {
                            distThreshold = 65536.0; /*256*256*/
                        }
                    }
                    WP_CONCUSSION => {
                        if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0 {
                            distThreshold = 65536.0; /*256*256*/
                        }
                    }
                    _ => {}
                }

                let dist: f32 = DistanceSquared(impactPos, muzzle);

                if dist < distThreshold {
                    //impact would be too close to me
                    tooClose = qtrue;
                } else if (*level).time - (*NPCInfo).enemyLastSeenTime > 5000
                    || (!(*NPCInfo).group.is_null()
                        && (*level).time - (*(*NPCInfo).group).lastSeenEnemyTime > 5000)
                {//we've haven't seen them in the last 5 seconds
                    //see if it's too far from where he is
                    distThreshold = 65536.0; /*256*256*/ //default
                    match (*NPC).s.weapon {
                        WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                        | WP_DET_PACK => {
                            distThreshold = 262144.0; /*512*512*/
                        }
                        WP_REPEATER => {
                            if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 {
                                distThreshold = 262144.0; /*512*512*/
                            }
                        }
                        WP_CONCUSSION => {
                            if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0 {
                                distThreshold = 262144.0; /*512*512*/
                            }
                        }
                        _ => {}
                    }
                    let dist2: f32 = DistanceSquared(impactPos, (*NPCInfo).enemyLastSeenLocation);
                    if dist2 > distThreshold {
                        //impact would be too far from enemy
                        tooFar = qtrue;
                    }
                }

                if tooClose == 0 && tooFar == 0 {
                    //okay too shoot at last pos
                    VectorSubtract((*NPCInfo).enemyLastSeenLocation, muzzle, dir);
                    VectorNormalize(dir);
                    vectoangles(dir, angles);

                    (*NPCInfo).desiredYaw = angles[YAW as usize];
                    (*NPCInfo).desiredPitch = angles[PITCH as usize];

                    shoot = qtrue;
                    faceEnemy = qfalse;
                    //AI_GroupUpdateSquadstates( NPCInfo->group, NPC, SQUAD_STAND_AND_SHOOT );
                    return;
                }
            }
        }
    }
}

pub unsafe fn ST_TrackEnemy(self_: *mut gentity_t, enemyPos: vec3_t) {
    //clear timers
    TIMER_Set(self_, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(1000, 2000));
    //TIMER_Set( self, "duck", -1 );
    TIMER_Set(self_, b"stick\0".as_ptr() as *const c_char, Q_irand(500, 1500));
    TIMER_Set(self_, b"stand\0".as_ptr() as *const c_char, -1);
    TIMER_Set(
        self_,
        b"scoutTime\0".as_ptr() as *const c_char,
        TIMER_Get(self_, b"stick\0".as_ptr() as *const c_char) - (*level).time + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*(*self_).NPC).combatPoint, qfalse);
    //go after his last seen pos
    NPC_SetMoveGoal(self_, enemyPos, 100, qfalse, -1, core::ptr::null_mut());
    if Q_irand(0, 3) == 0 {
        (*NPCInfo).aiFlags |= NPCAI_STOP_AT_LOS;
    }
}

pub unsafe fn ST_ApproachEnemy(self_: *mut gentity_t) -> c_int {
    TIMER_Set(self_, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(250, 500));
    //TIMER_Set( self, "duck", -1 );
    TIMER_Set(self_, b"stick\0".as_ptr() as *const c_char, Q_irand(1000, 2000));
    TIMER_Set(self_, b"stand\0".as_ptr() as *const c_char, -1);
    TIMER_Set(
        self_,
        b"scoutTime\0".as_ptr() as *const c_char,
        TIMER_Get(self_, b"stick\0".as_ptr() as *const c_char) - (*level).time + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*(*self_).NPC).combatPoint, qfalse);
    //return the relevant combat point flags
    CP_CLEAR | CP_CLOSEST
}

pub unsafe fn ST_HuntEnemy(self_: *mut gentity_t) {
    //TIMER_Set( NPC, "attackDelay", Q_irand( 250, 500 ) );//Disabled this for now, guys who couldn't hunt would never attack
    //TIMER_Set( NPC, "duck", -1 );
    TIMER_Set(NPC, b"stick\0".as_ptr() as *const c_char, Q_irand(250, 1000));
    TIMER_Set(NPC, b"stand\0".as_ptr() as *const c_char, -1);
    TIMER_Set(
        NPC,
        b"scoutTime\0".as_ptr() as *const c_char,
        TIMER_Get(NPC, b"stick\0".as_ptr() as *const c_char) - (*level).time + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*NPCInfo).combatPoint, qfalse);
    //go directly after the enemy
    if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
        (*(*self_).NPC).goalEntity = (*NPC).enemy;
    }
}

pub unsafe fn ST_TransferTimers(self_: *mut gentity_t, other: *mut gentity_t) {
    TIMER_Set(
        other,
        b"attackDelay\0".as_ptr() as *const c_char,
        TIMER_Get(self_, b"attackDelay\0".as_ptr() as *const c_char) - (*level).time,
    );
    TIMER_Set(
        other,
        b"duck\0".as_ptr() as *const c_char,
        TIMER_Get(self_, b"duck\0".as_ptr() as *const c_char) - (*level).time,
    );
    TIMER_Set(
        other,
        b"stick\0".as_ptr() as *const c_char,
        TIMER_Get(self_, b"stick\0".as_ptr() as *const c_char) - (*level).time,
    );
    TIMER_Set(
        other,
        b"scoutTime\0".as_ptr() as *const c_char,
        TIMER_Get(self_, b"scoutTime\0".as_ptr() as *const c_char) - (*level).time,
    );
    TIMER_Set(
        other,
        b"roamTime\0".as_ptr() as *const c_char,
        TIMER_Get(self_, b"roamTime\0".as_ptr() as *const c_char) - (*level).time,
    );
    TIMER_Set(
        other,
        b"stand\0".as_ptr() as *const c_char,
        TIMER_Get(self_, b"stand\0".as_ptr() as *const c_char) - (*level).time,
    );
    TIMER_Set(self_, b"attackDelay\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"duck\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"stick\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"scoutTime\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"roamTime\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"stand\0".as_ptr() as *const c_char, -1);
}

pub unsafe fn ST_TransferMoveGoal(self_: *mut gentity_t, other: *mut gentity_t) {
    if Q3_TaskIDPending(self_, TID_MOVE_NAV) != 0 {
        //can't transfer movegoal when a script we're running is waiting to complete
        return;
    }
    if (*(*self_).NPC).combatPoint != -1 {
        //I've got a combatPoint I'm going to, give it to him
        (*(*self_).NPC).lastFailedCombatPoint = (*(*other).NPC).combatPoint;
        (*(*other).NPC).combatPoint = (*(*self_).NPC).combatPoint;
        (*(*self_).NPC).combatPoint = -1;
    } else {
        //I must be going for a goal, give that to him instead
        if (*(*self_).NPC).goalEntity == (*(*self_).NPC).tempGoal {
            NPC_SetMoveGoal(
                other,
                (*(*(*self_).NPC).tempGoal).currentOrigin,
                (*(*self_).NPC).goalRadius as c_int,
                if ((*(*(*self_).NPC).tempGoal).svFlags & SVF_NAVGOAL) != 0 { qtrue } else { qfalse },
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
    TIMER_Set(self_, b"stand\0".as_ptr() as *const c_char, Q_irand(1000, 3000));
}

pub unsafe fn ST_GetCPFlags() -> c_int {
    let mut cpFlags: c_int = 0;
    if !NPC.is_null() && !(*NPCInfo).group.is_null() {
        if NPC == (*(*NPCInfo).group).commander
            && (*(*NPC).client).NPC_class == CLASS_IMPERIAL
        {//imperials hang back and give orders
            if (*(*NPCInfo).group).numGroup > 1
                && Q_irand(-3, (*(*NPCInfo).group).numGroup) > 1
            {//FIXME: make sure he;s giving orders with these lines
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
            /*
            if ( NPC->client->NPC_class == CLASS_SABOTEUR && !Q_irand( 0, 3 ) )
            {
                Saboteur_Cloak( NPC );
            }
            */
        }
        /*		else if ( NPCInfo->group->morale < NPCInfo->group->numGroup )
        		{//morale is low for our size
        			int moraleDrop = NPCInfo->group->numGroup - NPCInfo->group->morale;
        			if ( moraleDrop < -6 )
        			{//flee (no clear shot needed)
        				cpFlags = (CP_FLEE|CP_RETREAT|CP_COVER|CP_AVOID|CP_SAFE);
        			}
        			else if ( moraleDrop < -3 )
        			{//retreat (no clear shot needed)
        				cpFlags = (CP_RETREAT|CP_COVER|CP_AVOID|CP_SAFE);
        			}
        			else if ( moraleDrop < 0 )
        			{//cover (no clear shot needed)
        				cpFlags = (CP_COVER|CP_AVOID|CP_SAFE);
        			}
        		}*/
        else {
            let moraleBoost: c_int = (*(*NPCInfo).group).morale - (*(*NPCInfo).group).numGroup;
            if moraleBoost > 20 {
                //charge to any one and outflank (no cover needed)
                cpFlags = CP_CLEAR | CP_FLANK | CP_APPROACH_ENEMY;
                //Saboteur_Decloak( NPC );
            } else if moraleBoost > 15 {
                //charge to closest one (no cover needed)
                cpFlags = CP_CLEAR | CP_CLOSEST | CP_APPROACH_ENEMY;
                /*
                if ( NPC->client->NPC_class == CLASS_SABOTEUR && !Q_irand( 0, 3 ) )
                {
                    Saboteur_Decloak( NPC );
                }
                */
            } else if moraleBoost > 10 {
                //charge closer (no cover needed)
                cpFlags = CP_CLEAR | CP_APPROACH_ENEMY;
                /*
                if ( NPC->client->NPC_class == CLASS_SABOTEUR && !Q_irand( 0, 6 ) )
                {
                    Saboteur_Decloak( NPC );
                }
                */
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
    if !NPC.is_null() && ((*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST) != 0 {
        cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
        cpFlags |= CP_NEAREST;
    }
    cpFlags
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
    let mut i: c_int; //, j;
    let mut cp: c_int;
    let mut cpFlags_org: c_int;
    let mut cpFlags: c_int;
    let group: *mut AIGroupInfo_t = (*NPCInfo).group;
    let mut member: *mut gentity_t; //, *buddy;
    let mut runner: qboolean = qfalse;
    let mut enemyLost: qboolean = qfalse;
    let mut scouting: qboolean = qfalse;
    let mut squadState: c_int;
    let mut avoidDist: f32;

    (*group).processed = qtrue;

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

    if (*group).lastSeenEnemyTime < (*level).time - 180000 {
        //dissolve the group
        ST_Speech(NPC, SPEECH_LOST, 0.0);
        (*(*group).enemy).waypoint = NAV::GetNearestNode((*group).enemy);
        i = 0;
        while i < (*group).numGroup {
            member = g_entities.add((*group).member[i as usize].number as usize);
            SetNPCGlobals(member);
            if Q3_TaskIDPending(NPC, TID_MOVE_NAV) != 0 {
                //running somewhere that a script requires us to go, don't break from that
                i += 1;
                continue;
            }
            if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
                //not allowed to move on my own
                i += 1;
                continue;
            }
            //Lost enemy for three minutes?  go into search mode?
            G_ClearEnemy(NPC);
            (*NPC).waypoint = NAV::GetNearestNode((*group).enemy);
            if (*NPC).waypoint == WAYPOINT_NONE {
                (*NPCInfo).behaviorState = BS_DEFAULT; //BS_PATROL;
            } else if (*(*group).enemy).waypoint == WAYPOINT_NONE
                || NAV::EstimateCostToGoal((*NPC).waypoint, (*(*group).enemy).waypoint)
                    >= Q3_INFINITE
            {
                NPC_BSSearchStart((*NPC).waypoint, BS_SEARCH);
            } else {
                NPC_BSSearchStart((*(*group).enemy).waypoint, BS_SEARCH);
            }
            i += 1;
        }
        (*group).enemy = core::ptr::null_mut();
        RestoreNPCGlobals();
        return;
    }

    //see if anyone is running
    if (*group).numState[SQUAD_SCOUT as usize] > 0
        || (*group).numState[SQUAD_TRANSITION as usize] > 0
        || (*group).numState[SQUAD_RETREAT as usize] > 0
    {//someone is running
        runner = qtrue;
    }

    if /* !runner && */ (*group).lastSeenEnemyTime > (*level).time - 32000
        && (*group).lastSeenEnemyTime < (*level).time - 30000
    {//no-one has seen the enemy for 30 seconds// and no-one is running after him
        if !(*group).commander.is_null() && Q_irand(0, 1) == 0 {
            ST_Speech((*group).commander, SPEECH_ESCAPING, 0.0);
        } else {
            ST_Speech(NPC, SPEECH_ESCAPING, 0.0);
        }
        //don't say this again
        (*NPCInfo).blockedSpeechDebounceTime = (*level).time + 3000;
    }

    if (*group).lastSeenEnemyTime < (*level).time - 7000 {
        //no-one has seen the enemy for at least 10 seconds!  Should send a scout
        enemyLost = qtrue;
    }

    //Go through the list:

    //Everyone should try to get to a combat point if possible
    let curMemberNum: c_int;
    let lastMemberNum: c_int;
    if (*d_asynchronousGroupAI).integer != 0 {
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
    i = curMemberNum;
    while i < lastMemberNum {
        //reset combat point flags
        cp = -1;
        cpFlags = 0;
        squadState = SQUAD_IDLE;
        avoidDist = 0.0;
        scouting = qfalse;

        //get the next guy
        member = g_entities.add((*group).member[i as usize].number as usize);
        if (*member).enemy.is_null() {
            //don't include guys that aren't angry
            i += 1;
            continue;
        }
        SetNPCGlobals(member);

        if TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) == 0 {
            //running away
            i += 1;
            continue;
        }

        if Q3_TaskIDPending(NPC, TID_MOVE_NAV) != 0 {
            //running somewhere that a script requires us to go
            i += 1;
            continue;
        }

        if (*NPC).s.weapon == WP_NONE
            && !(*NPCInfo).goalEntity.is_null()
            && (*NPCInfo).goalEntity == (*NPCInfo).tempGoal
            && (*(*NPCInfo).goalEntity).s.eType == ET_ITEM
        {//running to pick up a gun, don't do other logic
            i += 1;
            continue;
        }

        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
            //not allowed to do combat-movement
            i += 1;
            continue;
        }

        if (*(*NPC).client).ps.weapon == WP_NONE {
            //weaponless, should be hiding
            if (*NPCInfo).goalEntity.is_null()
                || (*(*NPCInfo).goalEntity).enemy.is_null()
                || (*(*(*NPCInfo).goalEntity).enemy).s.eType != ET_ITEM
            {//not running after a pickup
                if TIMER_Done(NPC, b"hideTime\0".as_ptr() as *const c_char) != 0
                    || (DistanceSquared(
                        (*(*group).enemy).currentOrigin,
                        (*NPC).currentOrigin,
                    ) < 65536.0
                        && NPC_ClearLOS2((*NPC).enemy) != 0)
                {//done hiding or enemy near and can see us
                    //er, start another flee I guess?
                    NPC_StartFlee(
                        (*NPC).enemy,
                        (*(*NPC).enemy).currentOrigin,
                        AEL_DANGER_GREAT,
                        5000,
                        10000,
                    );
                } //else, just hang here
            }
            i += 1;
            continue;
        }

        if enemyLost != 0 && NAV::InSameRegion(NPC, (*(*NPC).enemy).currentOrigin) != 0 {
            ST_TrackEnemy(NPC, (*(*NPC).enemy).currentOrigin);
            i += 1;
            continue;
        }

        if (*NPC).enemy.is_null() {
            i += 1;
            continue;
        }

        // Check To See We Have A Clear Shot To The Enemy Every Couple Seconds
        //---------------------------------------------------------------------
        if TIMER_Done(NPC, b"checkGrenadeTooCloseDebouncer\0".as_ptr() as *const c_char) != 0 {
            TIMER_Set(
                NPC,
                b"checkGrenadeTooCloseDebouncer\0".as_ptr() as *const c_char,
                Q_irand(300, 600),
            );

            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            let mut fled: bool = false;
            let mut ent: *mut gentity_t;

            let mut entityList: [*mut gentity_t; MAX_GENTITIES as usize] =
                [core::ptr::null_mut(); MAX_GENTITIES as usize];

            for inner_i in 0..3_i32 {
                mins[inner_i as usize] = (*NPC).currentOrigin[inner_i as usize] - 200.0;
                maxs[inner_i as usize] = (*NPC).currentOrigin[inner_i as usize] + 200.0;
            }

            let numListedEntities: c_int =
                gi.EntitiesInBox(mins, maxs, entityList.as_mut_ptr(), MAX_GENTITIES);

            let mut e: c_int = 0;
            while e < numListedEntities {
                ent = entityList[e as usize];

                if ent == NPC {
                    e += 1;
                    continue;
                }
                if (*ent).owner == NPC {
                    e += 1;
                    continue;
                }
                if !(*ent).inuse {
                    e += 1;
                    continue;
                }
                if (*ent).s.eType == ET_MISSILE {
                    if (*ent).s.weapon == WP_THERMAL {
                        //a thermal
                        if (*ent).has_bounced != 0
                            && ((*ent).owner.is_null() || OnSameTeam((*ent).owner, NPC) == 0)
                        {//bounced and an enemy thermal
                            ST_Speech(NPC, SPEECH_COVER, 0.0); //FIXME: flee sound?
                            NPC_StartFlee(
                                (*NPC).enemy,
                                (*ent).currentOrigin,
                                AEL_DANGER_GREAT,
                                1000,
                                2000,
                            );
                            fled = true;
                            //							cpFlags |= (CP_CLEAR|CP_COVER);	// NOPE, Can't See The Enemy, So Find A New Combat Point
                            TIMER_Set(
                                NPC,
                                b"checkGrenadeTooCloseDebouncer\0".as_ptr() as *const c_char,
                                Q_irand(2000, 4000),
                            );
                            break;
                        }
                    }
                }
                e += 1;
            }
            if fled {
                i += 1;
                continue;
            }
        }

        // Check To See We Have A Clear Shot To The Enemy Every Couple Seconds
        //---------------------------------------------------------------------
        if TIMER_Done(NPC, b"checkEnemyVisDebouncer\0".as_ptr() as *const c_char) != 0 {
            TIMER_Set(
                NPC,
                b"checkEnemyVisDebouncer\0".as_ptr() as *const c_char,
                Q_irand(3000, 7000),
            );
            if NPC_ClearLOS2((*NPC).enemy) == 0 {
                cpFlags |= CP_CLEAR | CP_COVER; // NOPE, Can't See The Enemy, So Find A New Combat Point
            }
        }

        // Check To See If The Enemy Is Too Close For Comfort
        //----------------------------------------------------
        if (*(*NPC).client).NPC_class != CLASS_ASSASSIN_DROID {
            if TIMER_Done(NPC, b"checkEnemyTooCloseDebouncer\0".as_ptr() as *const c_char) != 0 {
                TIMER_Set(
                    NPC,
                    b"checkEnemyTooCloseDebouncer\0".as_ptr() as *const c_char,
                    Q_irand(1000, 6000),
                );

                let mut distThreshold: f32 = 16384.0; /*128*128*/ //default
                match (*NPC).s.weapon {
                    WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                    | WP_DET_PACK => {
                        distThreshold = 65536.0; /*256*256*/
                    }
                    WP_REPEATER => {
                        if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 {
                            distThreshold = 65536.0; /*256*256*/
                        }
                    }
                    WP_CONCUSSION => {
                        if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0 {
                            distThreshold = 65536.0; /*256*256*/
                        }
                    }
                    _ => {}
                }

                if DistanceSquared((*(*group).enemy).currentOrigin, (*NPC).currentOrigin)
                    < distThreshold
                {
                    cpFlags |= CP_CLEAR | CP_COVER;
                }
            }
        }

        //clear the local state
        (*NPCInfo).localState = LSTATE_NONE;

        cpFlags &= !CP_NEAREST;
        //Assign combat points
        if cpFlags != 0 {
            //we want to run to a combat point
            //always avoid enemy when picking combat points, and we always want to be able to get there
            cpFlags |= CP_AVOID_ENEMY | CP_HAS_ROUTE | CP_TRYFAR;
            avoidDist = 200.0;
            cpFlags_org = cpFlags; //remember what we *wanted* to do...

            //now get a combat point
            if cp == -1 {
                //may have had sone set above
                cp = NPC_FindCombatPointRetry(
                    (*NPC).currentOrigin,
                    (*NPC).currentOrigin,
                    (*NPC).currentOrigin,
                    &mut cpFlags,
                    avoidDist,
                    (*NPCInfo).lastFailedCombatPoint,
                );
            }

            //see if we got a valid one
            if cp != -1 {
                //found a combat point
                //let others know that someone is now running
                runner = qtrue;
                //don't change course again until we get to where we're going
                TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const c_char, Q3_INFINITE);

                NPC_SetCombatPoint(cp);
                NPC_SetMoveGoal(
                    NPC,
                    (*level).combatPoints[cp as usize].origin,
                    8,
                    qtrue,
                    cp,
                    core::ptr::null_mut(),
                );

                // If Successfully
                if (cpFlags & CP_FLANK) != 0
                    || ((cpFlags & CP_COVER) != 0 && (cpFlags & CP_CLEAR) != 0)
                {
                } else if Q_irand(0, 3) == 0 {
                    (*NPCInfo).aiFlags |= NPCAI_STOP_AT_LOS;
                }

                //okay, try a move right now to see if we can even get there
                if (cpFlags & CP_FLANK) != 0 {
                    if (*group).numGroup > 1 {
                        NPC_ST_StoreMovementSpeech(SPEECH_OUTFLANK, -1.0);
                    }
                } else if (cpFlags & CP_COVER) != 0 && (cpFlags & CP_CLEAR) == 0 {
                    //going into hiding
                    NPC_ST_StoreMovementSpeech(SPEECH_COVER, -1.0);
                } else {
                    if Q_irand(0, 20) == 0 {
                        //hell, we're loading the sounds, use them every now and then!
                        if Q_irand(0, 1) != 0 {
                            NPC_ST_StoreMovementSpeech(SPEECH_OUTFLANK, -1.0);
                        } else {
                            NPC_ST_StoreMovementSpeech(SPEECH_ESCAPING, -1.0);
                        }
                    }
                }
            }
        }
        i += 1;
    }

    RestoreNPCGlobals();
}

extern "C" {
    fn G_Knockdown(
        self_: *mut gentity_t,
        attacker: *mut gentity_t,
        pushDir: *const vec3_t,
        strength: f32,
        breakSaberLock: qboolean,
    );
}
pub unsafe fn Noghri_StickTrace() {
    if (*NPC).ghoul2.len() == 0 || (*NPC).weaponModel[0] <= 0 {
        return;
    }

    let boltIndex: c_int =
        gi.G2API_AddBolt(&mut (*NPC).ghoul2[(*NPC).weaponModel[0] as usize], b"*weapon\0".as_ptr() as *const c_char);
    if boltIndex != -1 {
        let curTime: c_int = if cg.time != 0 { cg.time } else { (*level).time };
        let mut hit: qboolean = qfalse;
        let mut lastHit: c_int = ENTITYNUM_NONE;
        let mut time: c_int = curTime - 25;
        while time <= curTime + 25 && hit == 0 {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut tip: vec3_t = [0.0; 3];
            let mut dir: vec3_t = [0.0; 3];
            let mut base: vec3_t = [0.0; 3];
            let angles: vec3_t = [0.0, (*NPC).currentAngles[YAW as usize], 0.0];
            let mins: vec3_t = [-2.0, -2.0, -2.0];
            let maxs: vec3_t = [2.0, 2.0, 2.0];
            let mut trace: trace_t = core::mem::zeroed();

            gi.G2API_GetBoltMatrix(
                &mut (*NPC).ghoul2,
                (*NPC).weaponModel[0],
                boltIndex,
                &mut boltMatrix,
                angles,
                (*NPC).currentOrigin,
                time,
                core::ptr::null_mut(),
                (*NPC).s.modelScale,
            );
            gi.G2API_GiveMeVectorFromMatrix(boltMatrix, ORIGIN, base);
            gi.G2API_GiveMeVectorFromMatrix(boltMatrix, POSITIVE_Y, dir);
            VectorMA(base, 48.0, dir, tip);
            #[cfg(not(feature = "final_build"))]
            {
                if (*d_saberCombat).integer > 1 {
                    G_DebugLine(base, tip, FRAMETIME, 0x000000ff, qtrue);
                }
            }
            gi.trace(
                &mut trace,
                base,
                mins,
                maxs,
                tip,
                (*NPC).s.number,
                MASK_SHOT,
                G2_RETURNONHIT,
                10,
            );
            if trace.fraction < 1.0 && trace.entityNum != lastHit {
                //hit something
                let traceEnt: *mut gentity_t = g_entities.add(trace.entityNum as usize);
                if (*traceEnt).takedamage != 0
                    && ((*traceEnt).client.is_null()
                        || traceEnt == (*NPC).enemy
                        || (*(*traceEnt).client).NPC_class != (*(*NPC).client).NPC_class)
                {//smack
                    let dmg: c_int = Q_irand(12, 20); //FIXME: base on skill!
                    //FIXME: debounce?
                    G_Sound(
                        traceEnt,
                        G_SoundIndex(
                            va(
                                b"sound/weapons/tusken_staff/stickhit%d.wav\0".as_ptr() as *const c_char,
                                Q_irand(1, 4),
                            ),
                        ),
                    );
                    G_Damage(
                        traceEnt,
                        NPC,
                        NPC,
                        vec3_origin,
                        trace.endpos,
                        dmg,
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                    if (*traceEnt).health > 0 && dmg > 17 {
                        //do pain on enemy
                        G_Knockdown(traceEnt, NPC, &dir, 300.0, qtrue);
                    }
                    lastHit = trace.entityNum;
                    hit = qtrue;
                }
            }
            time += 25;
        }
    }
}
/*
-------------------------
NPC_BSST_Attack
-------------------------
*/

pub unsafe fn NPC_BSST_Attack() {
    //Don't do anything if we're hurt
    if (*NPC).painDebounceTime > (*level).time {
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    //NPC_CheckEnemy( qtrue, qfalse );
    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(qfalse) == qfalse {
        // !NPC->enemy )//
        if (*(*NPC).client).playerTeam == TEAM_PLAYER {
            NPC_BSPatrol();
        } else {
            NPC_BSST_Patrol(); //FIXME: or patrol?
        }
        return;
    }

    //FIXME: put some sort of delay into the guys depending on how they saw you...?

    //Get our group info
    if TIMER_Done(NPC, b"interrogating\0".as_ptr() as *const c_char) != 0 {
        AI_GetGroup(NPC); //, 45, 512, NPC->enemy );
    } else {
        //FIXME: when done interrogating, I should send out a team alert!
    }

    if !(*NPCInfo).group.is_null() {
        //I belong to a squad of guys - we should *always* have a group
        if (*(*NPCInfo).group).processed == 0 {
            //I'm the first ent in my group, I'll make the command decisions
            #[cfg(feature = "ai_timers")]
            let start_time = GetTime(0);
            ST_Commander();
            #[cfg(feature = "ai_timers")]
            {
                let commTime = GetTime(start_time);
                if commTime > 20 {
                    gi.Printf(
                        b"%sERROR: Commander time: %d\n\0".as_ptr() as *const c_char,
                        S_COLOR_RED,
                        commTime,
                    );
                } else if commTime > 10 {
                    gi.Printf(
                        b"%sWARNING: Commander time: %d\n\0".as_ptr() as *const c_char,
                        S_COLOR_YELLOW,
                        commTime,
                    );
                } else if commTime > 2 {
                    gi.Printf(
                        b"%sCommander time: %d\n\0".as_ptr() as *const c_char,
                        S_COLOR_GREEN,
                        commTime,
                    );
                }
            }
        }
    } else if TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) != 0
        && NPC_CheckForDanger(
            NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_DANGER),
        ) != 0
    {//not already fleeing, and going to run
        ST_Speech(NPC, SPEECH_COVER, 0.0);
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    if (*NPC).enemy.is_null() {
        //WTF?  somehow we lost our enemy?
        NPC_BSST_Patrol(); //FIXME: or patrol?
        return;
    }

    if !(*NPCInfo).goalEntity.is_null() && (*NPCInfo).goalEntity != (*NPC).enemy {
        (*NPCInfo).goalEntity = UpdateGoal();
    }

    enemyLOS = qfalse;
    enemyCS = qfalse;
    enemyInFOV = qfalse;
    move_ = qtrue;
    faceEnemy = qfalse;
    shoot = qfalse;
    hitAlly = qfalse;
    VectorClear(impactPos);
    enemyDist = DistanceSquared((*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin);

    let mut enemyDir: vec3_t = [0.0; 3];
    let mut shootDir: vec3_t = [0.0; 3];
    VectorSubtract((*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin, enemyDir);
    VectorNormalize(enemyDir);
    AngleVectors(
        (*(*NPC).client).ps.viewangles,
        shootDir.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    let dot: f32 = DotProduct(enemyDir, shootDir);
    if dot > 0.5 || (enemyDist * (1.0 - dot)) < 10000.0 {
        //enemy is in front of me or they're very close and not behind me
        enemyInFOV = qtrue;
    }

    if enemyDist < MIN_ROCKET_DIST_SQUARED {
        //enemy within 128
        if ((*(*NPC).client).ps.weapon == WP_FLECHETTE
            || (*(*NPC).client).ps.weapon == WP_REPEATER)
            && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0
        {//shooting an explosive, but enemy too close, switch to primary fire
            (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
            //FIXME: we can never go back to alt-fire this way since, after this, we don't know if we were initially supposed to use alt-fire or not...
        }
    } else if enemyDist > 65536.0 {
        //256 squared
        if (*(*NPC).client).ps.weapon == WP_DISRUPTOR {
            //sniping...
            if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0 {
                //use primary fire
                (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
                //reset fire-timing variables
                NPC_ChangeWeapon((*(*NPC).client).ps.weapon);
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }
    }

    //can we see our target?
    if NPC_ClearLOS2((*NPC).enemy) != 0 {
        AI_GroupUpdateEnemyLastSeen((*NPCInfo).group, (*(*NPC).enemy).currentOrigin);
        (*NPCInfo).enemyLastSeenTime = (*level).time;
        enemyLOS = qtrue;

        if (*(*NPC).client).ps.weapon == WP_NONE {
            enemyCS = qfalse; //not true, but should stop us from firing
            NPC_AimAdjust(-1); //adjust aim worse longer we have no weapon
        } else {
            //can we shoot our target?
            if (enemyDist < MIN_ROCKET_DIST_SQUARED)
                && (((*level).time - (*NPC).lastMoveTime) < 5000)
                && ((*(*NPC).client).ps.weapon == WP_ROCKET_LAUNCHER
                    || ((*(*NPC).client).ps.weapon == WP_CONCUSSION
                        && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0)
                    || ((*(*NPC).client).ps.weapon == WP_FLECHETTE
                        && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0))
            {
                enemyCS = qfalse; //not true, but should stop us from firing
                hitAlly = qtrue; //us!
                //FIXME: if too close, run away!
            } else if enemyInFOV != 0 {
                //if enemy is FOV, go ahead and check for shooting
                let hit: c_int = NPC_ShotEntity((*NPC).enemy, impactPos);
                let hitEnt: *mut gentity_t = g_entities.add(hit as usize);

                if hit == (*(*NPC).enemy).s.number
                    || (!hitEnt.is_null()
                        && !(*hitEnt).client.is_null()
                        && (*(*hitEnt).client).playerTeam == (*(*NPC).client).enemyTeam)
                    || (!hitEnt.is_null()
                        && (*hitEnt).takedamage != 0
                        && (((*hitEnt).svFlags & SVF_GLASS_BRUSH) != 0
                            || (*hitEnt).health < 40
                            || (*NPC).s.weapon == WP_EMPLACED_GUN))
                {//can hit enemy or enemy ally or will hit glass or other minor breakable (or in emplaced gun), so shoot anyway
                    AI_GroupUpdateClearShotTime((*NPCInfo).group);
                    enemyCS = qtrue;
                    NPC_AimAdjust(2); //adjust aim better longer we have clear shot at enemy
                    VectorCopy((*(*NPC).enemy).currentOrigin, (*NPCInfo).enemyLastSeenLocation);
                } else {
                    //Hmm, have to get around this bastard
                    NPC_AimAdjust(1); //adjust aim better longer we can see enemy
                    ST_ResolveBlockedShot(hit);
                    if !hitEnt.is_null()
                        && !(*hitEnt).client.is_null()
                        && (*(*hitEnt).client).playerTeam == (*(*NPC).client).playerTeam
                    {//would hit an ally, don't fire!!!
                        hitAlly = qtrue;
                    } else {
                        //Check and see where our shot *would* hit... if it's not close to the enemy (within 256?), then don't fire
                    }
                }
            } else {
                enemyCS = qfalse; //not true, but should stop us from firing
            }
        }
    } else if gi.inPVS((*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin) != 0 {
        (*NPCInfo).enemyLastSeenTime = (*level).time;
        faceEnemy = qtrue;
        NPC_AimAdjust(-1); //adjust aim worse longer we cannot see enemy
    }

    if (*(*NPC).client).ps.weapon == WP_NONE {
        faceEnemy = qfalse;
        shoot = qfalse;
    } else {
        if enemyLOS != 0 {
            //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
            faceEnemy = qtrue;
        }
        if enemyCS != 0 {
            shoot = qtrue;
        }
    }

    //Check for movement to take care of
    ST_CheckMoveState();

    //See if we should override shooting decision with any special considerations
    ST_CheckFireState();

    if faceEnemy != 0 {
        //face the enemy
        NPC_FaceEnemy(qtrue);
    }

    if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
        //not supposed to chase my enemies
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            //goal is my entity, so don't move
            move_ = qfalse;
        }
    } else if ((*(*NPC).NPC).scriptFlags & SCF_NO_GROUPS) != 0 {
        //	NPCInfo->goalEntity = UpdateGoal();

        (*NPCInfo).goalEntity = if enemyLOS != 0 {
            core::ptr::null_mut()
        } else {
            (*NPC).enemy
        };
    }

    if (*(*NPC).client).fireDelay != 0 && (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
        move_ = qfalse;
    }

    if (*ucmd).rightmove == 0 {
        //only if not already strafing for some strange reason...?
        //NOTE: these are never set here, but can be set in AI_Jedi.cpp for those NPCs who are sort of Stormtrooper/Jedi hybrids
        //NOTE: this stomps navigation movement entirely!
        //FIXME: if enemy behind me and turning to face enemy, don't strafe in that direction, too
        if TIMER_Done(NPC, b"strafeLeft\0".as_ptr() as *const c_char) == 0 {
            /*
            if ( NPCInfo->desiredYaw > NPC->client->ps.viewangles[YAW] + 60 )
            {//we want to turn left, don't apply the strafing
            }
            else
            */
            {//go ahead and strafe left
                (*ucmd).rightmove = -127;
                //re-check the duck as we might want to be rolling
                VectorClear((*(*NPC).client).ps.moveDir);
                move_ = qfalse;
            }
        } else if TIMER_Done(NPC, b"strafeRight\0".as_ptr() as *const c_char) == 0 {
            /*if ( NPCInfo->desiredYaw < NPC->client->ps.viewangles[YAW] - 60 )
            {//we want to turn right, don't apply the strafing
            }
            else
            */
            {//go ahead and strafe left
                (*ucmd).rightmove = 127;
                VectorClear((*(*NPC).client).ps.moveDir);
                move_ = qfalse;
            }
        }
    }

    if (*(*NPC).client).ps.legsAnim == BOTH_GUARD_LOOKAROUND1 {
        //don't move when doing silly look around thing
        move_ = qfalse;
    }
    if move_ != 0 {
        //move toward goal
        if !(*NPCInfo).goalEntity.is_null() {
            //&& ( NPCInfo->goalEntity != NPC->enemy || enemyDist > 10000 ) )//100 squared
            move_ = ST_Move();
            if ((*(*NPC).client).NPC_class != CLASS_ROCKETTROOPER
                || (*NPC).s.weapon != WP_ROCKET_LAUNCHER
                || enemyDist < MIN_ROCKET_DIST_SQUARED) //rockettroopers who use rocket launchers turn around and run if you get too close (closer than 128)
                && (*ucmd).forwardmove <= -32
            {//moving backwards at least 45 degrees
                if !(*NPCInfo).goalEntity.is_null()
                    && DistanceSquared(
                        (*(*NPCInfo).goalEntity).currentOrigin,
                        (*NPC).currentOrigin,
                    ) > MIN_TURN_AROUND_DIST_SQ as f32
                {//don't stop running backwards if your goal is less than 100 away
                    if TIMER_Done(NPC, b"runBackwardsDebounce\0".as_ptr() as *const c_char) != 0 {
                        //not already waiting for next run backwards
                        if TIMER_Exists(NPC, b"runningBackwards\0".as_ptr() as *const c_char) == 0 {
                            //start running backwards
                            TIMER_Set(
                                NPC,
                                b"runningBackwards\0".as_ptr() as *const c_char,
                                Q_irand(500, 1000),
                            ); //Q_irand( 2000, 3500 ) );
                        } else if TIMER_Done2(
                            NPC,
                            b"runningBackwards\0".as_ptr() as *const c_char,
                            qtrue,
                        ) != 0
                        {//done running backwards
                            TIMER_Set(
                                NPC,
                                b"runBackwardsDebounce\0".as_ptr() as *const c_char,
                                Q_irand(3000, 5000),
                            );
                        }
                    }
                }
            } else {
                //not running backwards
                //TIMER_Remove( NPC, "runningBackwards" );
            }
        } else {
            move_ = qfalse;
        }
    }

    if move_ == 0 {
        if (*(*NPC).client).NPC_class != CLASS_ASSASSIN_DROID {
            if TIMER_Done(NPC, b"duck\0".as_ptr() as *const c_char) == 0 {
                (*ucmd).upmove = -127;
            }
        }
        //FIXME: what about leaning?
    } else {
        //stop ducking!
        TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, -1);
    }

    if (*(*NPC).client).NPC_class == CLASS_REBORN //cultist using a gun
        && (*NPCInfo).rank >= RANK_LT_COMM //commando or better
        && (*(*(*NPC).enemy).s).weapon == WP_SABER
    //fighting a saber-user
    {//commando saboteur vs. jedi/reborn
        //see if we need to avoid their saber
        NPC_EvasionSaber();
    }

    if /*/!TIMER_Done( NPC, "flee" ) || */
        (move_ != 0 && TIMER_Done(NPC, b"runBackwardsDebounce\0".as_ptr() as *const c_char) == 0)
    {//running away
        faceEnemy = qfalse;
    }

    //FIXME: check scf_face_move_dir here?

    if faceEnemy == 0 {
        //we want to face in the dir we're running
        if move_ == 0 {
            //if we haven't moved, we should look in the direction we last looked?
            VectorCopy((*(*NPC).client).ps.viewangles, (*NPCInfo).lastPathAngles);
        }
        (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW as usize];
        (*NPCInfo).desiredPitch = 0.0;
        NPC_UpdateAngles(qtrue, qtrue);
        if move_ != 0 {
            //don't run away and shoot
            shoot = qfalse;
        }
    }

    if ((*NPCInfo).scriptFlags & SCF_DONT_FIRE) != 0 {
        shoot = qfalse;
    }

    if !(*NPC).enemy.is_null() && !(*(*NPC).enemy).enemy.is_null() {
        if (*(*NPC).enemy).s.weapon == WP_SABER
            && (*(*(*NPC).enemy).enemy).s.weapon == WP_SABER
        {//don't shoot at an enemy jedi who is fighting another jedi, for fear of injuring one or causing rogue blaster deflections (a la Obi Wan/Vader duel at end of ANH)
            shoot = qfalse;
        }
    }
    //FIXME: don't shoot right away!
    if (*(*NPC).client).fireDelay != 0 {
        if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
            Saboteur_Decloak(NPC, 0);
        }
        if (*NPC).s.weapon == WP_ROCKET_LAUNCHER
            || ((*NPC).s.weapon == WP_CONCUSSION
                && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0)
        {
            if enemyLOS == 0 || enemyCS == 0 {
                //cancel it
                (*(*NPC).client).fireDelay = 0;
            } else {
                //delay our next attempt
                TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(3000, 5000));
            }
        }
    } else if shoot != 0 {
        //try to shoot if it's time
        if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
            Saboteur_Decloak(NPC, 0);
        }
        if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != 0 {
            if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) == 0 {
                // we've already fired, no need to do it again here
                WeaponThink(qtrue);
            }
            //NASTY
            if (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
                if ((*ucmd).buttons & BUTTON_ATTACK) != 0
                    && move_ == 0
                    && (*g_spskill).integer > 1
                    && Q_irand(0, 3) == 0
                {//every now and then, shoot a homing rocket
                    (*ucmd).buttons &= !BUTTON_ATTACK;
                    (*ucmd).buttons |= BUTTON_ALT_ATTACK;
                    (*(*NPC).client).fireDelay = Q_irand(1000, 2500);
                }
            } else if (*NPC).s.weapon == WP_NOGHRI_STICK && enemyDist < (48 * 48) as f32
            //?
            {
                (*ucmd).buttons &= !BUTTON_ATTACK;
                (*ucmd).buttons |= BUTTON_ALT_ATTACK;
                (*(*NPC).client).fireDelay = Q_irand(1500, 2000);
            }
        }
    } else {
        if (*NPC).attackDebounceTime < (*level).time {
            if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
                Saboteur_Cloak(NPC);
            }
        }
    }
}

extern "C" {
    fn G_TuskenAttackAnimDamage(self_: *mut gentity_t) -> qboolean;
}
pub unsafe fn NPC_BSST_Default() {
    if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) != 0 {
        WeaponThink(qtrue);
    }

    if (*NPC).s.weapon == WP_NOGHRI_STICK {
        if G_TuskenAttackAnimDamage(NPC) != 0 {
            Noghri_StickTrace();
        }
    }

    if (*NPC).enemy.is_null() {
        //don't have an enemy, look for one
        NPC_BSST_Patrol();
    } else {
        //if ( NPC->enemy )
        //have an enemy
        if !(*(*NPC).enemy).client.is_null() //enemy is a client
            && ((*(*(*NPC).enemy).client).NPC_class == CLASS_UGNAUGHT
                || (*(*(*NPC).enemy).client).NPC_class == CLASS_JAWA) //enemy is a lowly jawa or ugnaught
            && (*(*NPC).enemy).enemy != NPC //enemy's enemy is not me
            && ((*(*NPC).enemy).enemy.is_null()
                || (*(*(*(*NPC).enemy).enemy).client).NPC_class != CLASS_RANCOR
                    && (*(*(*(*NPC).enemy).enemy).client).NPC_class != CLASS_WAMPA)
        //enemy's enemy is not a client or is not a wampa or rancor (which is scarier than me)
        {//they should be scared of ME and no-one else
            G_SetEnemy((*NPC).enemy, NPC);
        }
        NPC_CheckGetNewWeapon();
        NPC_BSST_Attack();
    }
}
