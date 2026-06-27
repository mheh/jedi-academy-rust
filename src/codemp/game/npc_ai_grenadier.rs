//! Faithful port of `NPC_AI_Grenadier.c` — the thermal-detonator "grenadier"
//! NPC's behavior state. The whole file is now landed: the timer-clear helper,
//! the confusion sound reaction, the pain handler, the hold-position helper, the
//! move helper, the check-fire/check-move state helpers, the shot evaluator, and
//! the patrol / attack / default behavior-state entry points.
//!
//! Ported: `Grenadier_ClearTimers` (NPC_AI_Grenadier.c:49),
//! `NPC_Grenadier_PlayConfusionSound` (:65), `NPC_Grenadier_Pain` (:90),
//! `Grenadier_HoldPosition` (:111), `Grenadier_Move` (:129),
//! `NPC_BSGrenadier_Patrol` (:190), `Grenadier_CheckMoveState` (:307),
//! `Grenadier_CheckFireState` (:399), `Grenadier_EvaluateShot` (:441),
//! `NPC_BSGrenadier_Attack` (:461), `NPC_BSGrenadier_Default` (:664).
//! (`NPC_BSGrenadier_Idle` is disabled/commented-out in the C source.)

#![allow(non_snake_case)] // C function names (`Grenadier_ClearTimers`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define`/enum constants kept verbatim

use core::ffi::c_int;
use core::ptr::addr_of;

use crate::codemp::game::ai_h::{
    SQUAD_COVER, SQUAD_IDLE, SQUAD_RETREAT, SQUAD_SCOUT, SQUAD_STAND_AND_SHOOT, SQUAD_TRANSITION,
};
use crate::codemp::game::b_local_h::{
    CP_APPROACH_ENEMY, CP_CLEAR, CP_CLOSEST, CP_FLANK, CP_HAS_ROUTE, CP_HORZ_DIST_COLL, CP_NEAREST,
};
use crate::codemp::game::b_public_h::{
    BS_DEFAULT, SCF_CHASE_ENEMIES, SCF_DONT_FIRE, SCF_FIRE_WEAPON, SCF_IGNORE_ALERTS,
    SCF_LOOK_FOR_ENEMIES, SCF_USE_CP_NEAREST,
};
use crate::codemp::game::bg_public::{
    EV_CONFUSE1, EV_CONFUSE3, EV_PUSHED1, EV_PUSHED3, STAT_WEAPONS,
};
use crate::codemp::game::bg_weapons_h::{WP_SABER, WP_STUN_BATON, WP_THERMAL};
use crate::codemp::game::g_local::{gentity_t, AEL_DANGER, AEL_DISCOVERED, AEL_SUSPICIOUS};
use crate::codemp::game::g_main::{g_entities, level};
use crate::codemp::game::g_nav::{
    navInfo_t, FlyingCreature, NAV_HitNavGoal, NPC_SetMoveGoal, NIF_COLLISION,
};
use crate::codemp::game::g_public_h::SVF_GLASS_BRUSH;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::npc::{ucmd, NPC, NPCInfo};
use crate::codemp::game::npc_combat::{
    G_AddVoiceEvent, G_ClearEnemy, G_SetEnemy, NPC_AimAdjust, NPC_ChangeWeapon,
    NPC_FindCombatPoint, NPC_FreeCombatPoint, NPC_SetCombatPoint, NPC_ShotEntity, WeaponThink,
};
use crate::codemp::game::npc_goal::{NPC_ReachedGoal, UpdateGoal};
use crate::codemp::game::npc_move::{NAV_GetLastMove, NPC_MoveToGoal};
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::npc_senses::{InFOV3, NPC_CheckAlertEvents, NPC_CheckForDanger};
use crate::codemp::game::npc_utils::{
    NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles,
};
use crate::codemp::game::bg_pmove::BG_SabersOff;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, DistanceHorizontalSquared, DistanceSquared, VectorCompare,
    VectorCopy, VectorSubtract,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{trace_t, vec3_t, BUTTON_WALKING, PITCH, YAW};
use crate::trap;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};

// `NPC_CheckPlayerTeamStealth` and `trap` need full namespace; declared below.
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;

// File-scope statics mirroring the C decls — the per-frame attack working set,
// shared between `NPC_BSGrenadier_Attack` and its `Grenadier_CheckMoveState` /
// `Grenadier_CheckFireState` helpers.
static mut enemyLOS3: c_int = 0;
static mut enemyCS3: c_int = 0;
static mut faceEnemy3: c_int = 0;
static mut move3: c_int = 0;
static mut shoot3: c_int = 0;
static mut enemyDist3: f32 = 0.0;

// Local state enums
#[allow(dead_code)]
const LSTATE_NONE: c_int = 0;
const LSTATE_UNDERFIRE: c_int = LSTATE_NONE + 1;
#[allow(dead_code)]
const LSTATE_INVESTIGATE: c_int = LSTATE_UNDERFIRE + 1;

pub unsafe fn Grenadier_ClearTimers(ent: *mut gentity_t) {
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
}

pub unsafe fn NPC_Grenadier_PlayConfusionSound(self_: *mut gentity_t) {
    //FIXME: make this a custom sound in sound set
    if (*self_).health > 0 {
        G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000);
    }
    //reset him to be totally unaware again
    TIMER_Set(self_, c"enemyLastVisible".as_ptr(), 0);
    TIMER_Set(self_, c"flee".as_ptr(), 0);
    (*(*self_).NPC).squadState = SQUAD_IDLE;
    (*(*self_).NPC).tempBehavior = BS_DEFAULT;

    //self->NPC->behaviorState = BS_PATROL;
    G_ClearEnemy(self_); //FIXME: or just self->enemy = NULL;?

    (*(*self_).NPC).investigateCount = 0;
}

/*
-------------------------
NPC_ST_Pain
-------------------------
*/

pub unsafe extern "C" fn NPC_Grenadier_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    (*(*self_).NPC).localState = LSTATE_UNDERFIRE;

    TIMER_Set(self_, c"duck".as_ptr(), -1);
    TIMER_Set(self_, c"stand".as_ptr(), 2000);

    NPC_Pain(self_, attacker, damage);

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

unsafe fn Grenadier_HoldPosition() {
    NPC_FreeCombatPoint((*(*addr_of!(NPCInfo))).combatPoint, QTRUE);
    (*(*addr_of!(NPCInfo))).goalEntity = core::ptr::null_mut();

    /*if ( TIMER_Done( NPC, "stand" ) )
    {//FIXME: what if can't shoot from this pos?
        TIMER_Set( NPC, "duck", Q_irand( 2000, 4000 ) );
    }
    */
}

/*
-------------------------
ST_Move
-------------------------
*/

unsafe fn Grenadier_Move() -> qboolean {
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
            Grenadier_HoldPosition();
        }
    }

    //If our move failed, then reset
    if moved == QFALSE {
        //couldn't get to enemy
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0
            && (*(*NPC).client).ps.weapon == WP_THERMAL
            && !(*NPCInfo).goalEntity.is_null()
            && (*NPCInfo).goalEntity == (*NPC).enemy
        {
            //we were running after enemy
            //Try to find a combat point that can hit the enemy
            let mut cpFlags: c_int = CP_CLEAR | CP_HAS_ROUTE;
            let mut cp: c_int;

            if (*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST != 0 {
                cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                cpFlags |= CP_NEAREST;
            }
            cp = NPC_FindCombatPoint(
                &(*NPC).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                cpFlags,
                32.0,
                -1,
            );
            if cp == -1 && (*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST == 0 {
                //okay, try one by the enemy
                cp = NPC_FindCombatPoint(
                    &(*NPC).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &(*(*NPC).enemy).r.currentOrigin,
                    CP_CLEAR | CP_HAS_ROUTE | CP_HORZ_DIST_COLL,
                    32.0,
                    -1,
                );
            }
            //NOTE: there may be a perfectly valid one, just not one within CP_COLLECT_RADIUS of either me or him...
            if cp != -1 {
                //found a combat point that has a clear shot to enemy
                NPC_SetCombatPoint(cp);
                NPC_SetMoveGoal(
                    NPC,
                    &(*addr_of!(level)).combatPoints[cp as usize].origin,
                    8,
                    QTRUE,
                    cp,
                    core::ptr::null_mut(),
                );
                return moved;
            }
        }
        //just hang here
        Grenadier_HoldPosition();
    }

    moved
}

/*
-------------------------
ST_CheckFireState
-------------------------
*/

unsafe fn Grenadier_CheckFireState() {
    if enemyCS3 != 0 {
        //if have a clear shot, always try
        return;
    }

    if (*(*addr_of!(NPCInfo))).squadState == SQUAD_RETREAT
        || (*(*addr_of!(NPCInfo))).squadState == SQUAD_TRANSITION
        || (*(*addr_of!(NPCInfo))).squadState == SQUAD_SCOUT
    {
        //runners never try to fire at the last pos
        return;
    }

    if VectorCompare(&(*(*(*addr_of!(NPC))).client).ps.velocity, &vec3_origin) == 0 {
        //if moving at all, don't do this
        return;
    }

    //continue to fire on their last position
    /*
    if ( !Q_irand( 0, 1 ) && NPCInfo->enemyLastSeenTime && level.time - NPCInfo->enemyLastSeenTime < 4000 )
    {
        //Fire on the last known position
        vec3_t	muzzle, dir, angles;

        CalcEntitySpot( NPC, SPOT_WEAPON, muzzle );
        VectorSubtract( NPCInfo->enemyLastSeenLocation, muzzle, dir );

        VectorNormalize( dir );

        vectoangles( dir, angles );

        NPCInfo->desiredYaw		= angles[YAW];
        NPCInfo->desiredPitch	= angles[PITCH];
        //FIXME: they always throw toward enemy, so this will be very odd...
        shoot3 = qtrue;
        faceEnemy3 = qfalse;

        return;
    }
    */
}

pub unsafe fn Grenadier_EvaluateShot(hit: c_int) -> qboolean {
    if (*(*addr_of!(NPC))).enemy.is_null() {
        return QFALSE;
    }

    if hit == (*(*(*addr_of!(NPC))).enemy).s.number
        || (!core::ptr::addr_of!(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(hit as isize)).is_null()
            && ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(hit as isize)).r.svFlags & SVF_GLASS_BRUSH) != 0)
    {
        //can hit enemy or will hit glass, so shoot anyway
        return QTRUE;
    }
    QFALSE
}

/*
-------------------------
NPC_BSGrenadier_Patrol
-------------------------
*/

pub unsafe fn NPC_BSGrenadier_Patrol() {
    //FIXME: pick up on bodies of dead buddies?
    if (*NPCInfo).confusionTime < (*addr_of!(level)).time {
        //Look for any enemies
        if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            if NPC_CheckPlayerTeamStealth() != QFALSE {
                //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be automatic now
                //NPC_AngerSound();
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }

        if (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
            //Is there danger nearby
            let alertEvent: c_int = NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_SUSPICIOUS);
            if NPC_CheckForDanger(alertEvent) != QFALSE {
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            } else {
                //check for other alert events
                //There is an event to look at
                if alertEvent >= 0
                    && (*addr_of!(level)).alertEvents[alertEvent as usize].ID != (*NPCInfo).lastAlertID
                {
                    (*NPCInfo).lastAlertID = (*addr_of!(level)).alertEvents[alertEvent as usize].ID;
                    if (*addr_of!(level)).alertEvents[alertEvent as usize].level == AEL_DISCOVERED {
                        if !(*addr_of!(level)).alertEvents[alertEvent as usize].owner.is_null()
                            && !(*(*addr_of!(level)).alertEvents[alertEvent as usize].owner)
                                .client
                                .is_null()
                            && (*(*addr_of!(level)).alertEvents[alertEvent as usize].owner).health
                                >= 0
                            && (*(*(*addr_of!(level)).alertEvents[alertEvent as usize].owner).client)
                                .playerTeam
                                == (*(*NPC).client).enemyTeam
                        {
                            //an enemy
                            G_SetEnemy(NPC, (*addr_of!(level)).alertEvents[alertEvent as usize].owner);
                            //NPCInfo->enemyLastSeenTime = level.time;
                            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
                        }
                    } else {
                        //FIXME: get more suspicious over time?
                        //Save the position for movement (if necessary)
                        VectorCopy(
                            &(*addr_of!(level)).alertEvents[alertEvent as usize].position,
                            &mut (*NPCInfo).investigateGoal,
                        );
                        (*NPCInfo).investigateDebounceTime =
                            (*addr_of!(level)).time + Q_irand(500, 1000);
                        if (*addr_of!(level)).alertEvents[alertEvent as usize].level
                            == AEL_SUSPICIOUS
                        {
                            //suspicious looks longer
                            (*NPCInfo).investigateDebounceTime += Q_irand(500, 2500);
                        }
                    }
                }
            }

            if (*NPCInfo).investigateDebounceTime > (*addr_of!(level)).time {
                //FIXME: walk over to it, maybe?  Not if not chase enemies
                //NOTE: stops walking or doing anything else below
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];
                let o_yaw: f32;
                let o_pitch: f32;

                VectorSubtract(
                    &(*NPCInfo).investigateGoal,
                    &(*(*NPC).client).renderInfo.eyePoint,
                    &mut dir,
                );
                vectoangles(&dir, &mut angles);

                o_yaw = (*NPCInfo).desiredYaw;
                o_pitch = (*NPCInfo).desiredPitch;
                (*NPCInfo).desiredYaw = angles[YAW as usize];
                (*NPCInfo).desiredPitch = angles[PITCH as usize];

                NPC_UpdateAngles(QTRUE, QTRUE);

                (*NPCInfo).desiredYaw = o_yaw;
                (*NPCInfo).desiredPitch = o_pitch;
                return;
            }
        }
    }

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons |= BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
NPC_BSGrenadier_Idle
-------------------------
*/
/*
void NPC_BSGrenadier_Idle( void )
{
    //FIXME: check for other alert events?

    //Is there danger nearby?
    if ( NPC_CheckForDanger( NPC_CheckAlertEvents( qtrue, qtrue, -1, qfalse, AEL_DANGER ) ) )
    {
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

unsafe fn Grenadier_CheckMoveState() {
    //See if we're a scout
    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
        //behaviorState == BS_STAND_AND_SHOOT )
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            move3 = QFALSE;
            return;
        }
    }
    //See if we're running away
    else if (*NPCInfo).squadState == SQUAD_RETREAT {
        if TIMER_Done(NPC, c"flee".as_ptr()) != QFALSE {
            (*NPCInfo).squadState = SQUAD_IDLE;
        } else {
            faceEnemy3 = QFALSE;
        }
    }
    /*
    else if ( NPCInfo->squadState == SQUAD_IDLE )
    {
        if ( !NPCInfo->goalEntity )
        {
            move3 = qfalse;
            return;
        }
        //Should keep moving toward player when we're out of range... right?
    }
    */

    //See if we're moving towards a goal, not the enemy
    if (*NPCInfo).goalEntity != (*NPC).enemy && !(*NPCInfo).goalEntity.is_null() {
        //Did we make it?
        if NAV_HitNavGoal(
            &(*NPC).r.currentOrigin,
            &(*NPC).r.mins,
            &(*NPC).r.maxs,
            &(*(*NPCInfo).goalEntity).r.currentOrigin,
            16,
            FlyingCreature(NPC),
        ) != QFALSE
            || ((*NPCInfo).squadState == SQUAD_SCOUT && enemyLOS3 != 0 && enemyDist3 <= 10000.0)
        {
            let mut newSquadState: c_int = SQUAD_STAND_AND_SHOOT;
            //we got where we wanted to go, set timers based on why we were running
            match (*NPCInfo).squadState {
                SQUAD_RETREAT => {
                    //was running away
                    TIMER_Set(
                        NPC,
                        c"duck".as_ptr(),
                        ((*(*NPC).client).pers.maxHealth - (*NPC).health) * 100,
                    );
                    TIMER_Set(NPC, c"hideTime".as_ptr(), Q_irand(3000, 7000));
                    newSquadState = SQUAD_COVER;
                }
                SQUAD_TRANSITION => {
                    //was heading for a combat point
                    TIMER_Set(NPC, c"hideTime".as_ptr(), Q_irand(2000, 4000));
                }
                SQUAD_SCOUT => {
                    //was running after player
                }
                _ => {}
            }
            let _ = newSquadState;
            NPC_ReachedGoal();
            //don't attack right away
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(250, 500)); //FIXME: Slant for difficulty levels
            //don't do something else just yet
            TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(1000, 4000));
            //stop fleeing
            if (*NPCInfo).squadState == SQUAD_RETREAT {
                TIMER_Set(NPC, c"flee".as_ptr(), -(*addr_of!(level)).time);
                (*NPCInfo).squadState = SQUAD_IDLE;
            }
            return;
        }

        //keep going, hold of roamTimer until we get there
        TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(4000, 8000));
    }

    if (*NPCInfo).goalEntity.is_null() {
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            (*NPCInfo).goalEntity = (*NPC).enemy;
        }
    }
}

/*
-------------------------
NPC_BSGrenadier_Attack
-------------------------
*/

pub unsafe fn NPC_BSGrenadier_Attack() {
    //Don't do anything if we're hurt
    if (*NPC).painDebounceTime > (*addr_of!(level)).time {
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    //NPC_CheckEnemy( qtrue, qfalse );
    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE {
        // !NPC->enemy )//
        (*NPC).enemy = core::ptr::null_mut();
        NPC_BSGrenadier_Patrol(); //FIXME: or patrol?
        return;
    }

    if TIMER_Done(NPC, c"flee".as_ptr()) != QFALSE
        && NPC_CheckForDanger(NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_DANGER)) != QFALSE
    {
        //going to run
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    if (*NPC).enemy.is_null() {
        //WTF?  somehow we lost our enemy?
        NPC_BSGrenadier_Patrol(); //FIXME: or patrol?
        return;
    }

    enemyCS3 = QFALSE;
    enemyLOS3 = enemyCS3;
    move3 = QTRUE;
    faceEnemy3 = QFALSE;
    shoot3 = QFALSE;
    enemyDist3 = DistanceSquared(&(*(*NPC).enemy).r.currentOrigin, &(*NPC).r.currentOrigin);

    //See if we should switch to melee attack
    if enemyDist3 < 16384.0 //128
        && ((*(*NPC).enemy).client.is_null()
            || (*(*(*NPC).enemy).client).ps.weapon != WP_SABER
            || BG_SabersOff(&mut (*(*(*NPC).enemy).client).ps) != QFALSE)
    {
        //enemy is close and not using saber
        if (*(*NPC).client).ps.weapon == WP_THERMAL {
            //grenadier
            let trace: trace_t = trap::Trace(
                &(*NPC).r.currentOrigin,
                &(*(*NPC).enemy).r.mins,
                &(*(*NPC).enemy).r.maxs,
                &(*(*NPC).enemy).r.currentOrigin,
                (*NPC).s.number,
                (*(*NPC).enemy).clipmask,
            );
            if trace.allsolid == 0
                && trace.startsolid == 0
                && (trace.fraction == 1.0
                    || trace.entityNum as c_int == (*(*NPC).enemy).s.number)
            {
                //I can get right to him
                //reset fire-timing variables
                NPC_ChangeWeapon(WP_STUN_BATON);
                if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
                    //NPCInfo->behaviorState == BS_STAND_AND_SHOOT )
                    //FIXME: should we be overriding scriptFlags?
                    (*NPCInfo).scriptFlags |= SCF_CHASE_ENEMIES; //NPCInfo->behaviorState = BS_HUNT_AND_KILL;
                }
            }
        }
    } else if enemyDist3 > 65536.0
        || (!(*(*NPC).enemy).client.is_null()
            && (*(*(*NPC).enemy).client).ps.weapon == WP_SABER
            && (*(*(*NPC).enemy).client).ps.saberHolstered == 0)
    {
        //256
        //enemy is far or using saber
        if (*(*NPC).client).ps.weapon == WP_STUN_BATON
            && (*(*NPC).client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_THERMAL) != 0
        {
            //fisticuffs, make switch to thermal if have it
            //reset fire-timing variables
            NPC_ChangeWeapon(WP_THERMAL);
        }
    }

    //can we see our target?
    if NPC_ClearLOS4((*NPC).enemy) != QFALSE {
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
        enemyLOS3 = QTRUE;

        if (*(*NPC).client).ps.weapon == WP_STUN_BATON {
            if enemyDist3 <= 4096.0
                && InFOV3(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &(*(*NPC).client).ps.viewangles,
                    90,
                    45,
                ) != QFALSE
            {
                //within 64 & infront
                VectorCopy(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &mut (*NPCInfo).enemyLastSeenLocation,
                );
                enemyCS3 = QTRUE;
            }
        } else if InFOV3(
            &(*(*NPC).enemy).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &(*(*NPC).client).ps.viewangles,
            45,
            90,
        ) != QFALSE
        {
            //in front of me
            //can we shoot our target?
            //FIXME: how accurate/necessary is this check?
            let hit: c_int = NPC_ShotEntity((*NPC).enemy, core::ptr::null_mut());
            let hitEnt: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(hit as isize);
            if hit == (*(*NPC).enemy).s.number
                || (!hitEnt.is_null()
                    && !(*hitEnt).client.is_null()
                    && (*(*hitEnt).client).playerTeam == (*(*NPC).client).enemyTeam)
            {
                let enemyHorzDist: f32;

                VectorCopy(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &mut (*NPCInfo).enemyLastSeenLocation,
                );
                enemyHorzDist = DistanceHorizontalSquared(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                );
                if enemyHorzDist < 1048576.0 {
                    //within 1024
                    enemyCS3 = QTRUE;
                    NPC_AimAdjust(2); //adjust aim better longer we have clear shot at enemy
                } else {
                    NPC_AimAdjust(1); //adjust aim better longer we can see enemy
                }
            }
        }
    } else {
        NPC_AimAdjust(-1); //adjust aim worse longer we cannot see enemy
    }
    /*
    else if ( trap_InPVS( NPC->enemy->r.currentOrigin, NPC->r.currentOrigin ) )
    {
        NPCInfo->enemyLastSeenTime = level.time;
        faceEnemy3 = qtrue;
    }
    */

    if enemyLOS3 != 0 {
        //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
        faceEnemy3 = QTRUE;
    }

    if enemyCS3 != 0 {
        shoot3 = QTRUE;
        if (*(*NPC).client).ps.weapon == WP_THERMAL {
            //don't chase and throw
            move3 = QFALSE;
        } else if (*(*NPC).client).ps.weapon == WP_STUN_BATON
            && enemyDist3
                < ((*NPC).r.maxs[0] + (*(*NPC).enemy).r.maxs[0] + 16.0)
                    * ((*NPC).r.maxs[0] + (*(*NPC).enemy).r.maxs[0] + 16.0)
        {
            //close enough
            move3 = QFALSE;
        }
    } //this should make him chase enemy when out of range...?

    //Check for movement to take care of
    Grenadier_CheckMoveState();

    //See if we should override shooting decision with any special considerations
    Grenadier_CheckFireState();

    if move3 != 0 {
        //move toward goal
        if !(*NPCInfo).goalEntity.is_null()
        //&& ( NPCInfo->goalEntity != NPC->enemy || enemyDist3 > 10000 ) )//100 squared
        {
            move3 = Grenadier_Move();
        } else {
            move3 = QFALSE;
        }
    }

    if move3 == 0 {
        if TIMER_Done(NPC, c"duck".as_ptr()) == QFALSE {
            ucmd.upmove = -127;
        }
        //FIXME: what about leaning?
    } else {
        //stop ducking!
        TIMER_Set(NPC, c"duck".as_ptr(), -1);
    }

    if faceEnemy3 == 0 {
        //we want to face in the dir we're running
        if move3 != 0 {
            //don't run away and shoot
            (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW as usize];
            (*NPCInfo).desiredPitch = 0.0;
            shoot3 = QFALSE;
        }
        NPC_UpdateAngles(QTRUE, QTRUE);
    } else {
        // if ( faceEnemy3 )
        //face the enemy
        NPC_FaceEnemy(QTRUE);
    }

    if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
        shoot3 = QFALSE;
    }

    //FIXME: don't shoot right away!
    if shoot3 != 0 {
        //try to shoot if it's time
        if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE {
            if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON == 0 {
                // we've already fired, no need to do it again here
                WeaponThink(QTRUE);
                TIMER_Set(
                    NPC,
                    c"attackDelay".as_ptr(),
                    (*NPCInfo).shotTime - (*addr_of!(level)).time,
                );
            }
        }
    }
}

pub unsafe fn NPC_BSGrenadier_Default() {
    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(QTRUE);
    }

    if (*NPC).enemy.is_null() {
        //don't have an enemy, look for one
        NPC_BSGrenadier_Patrol();
    } else {
        //if ( NPC->enemy )
        //have an enemy
        NPC_BSGrenadier_Attack();
    }
}
