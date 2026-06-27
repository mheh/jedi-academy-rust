//! Slice of `NPC_AI_Sniper.c` — the ranged "sniper" NPC's behavior state.
//! Opened bottom-up at the leaf seam and drained as the NPC-AI / NAV core landed:
//! the timer/sound/pain helpers, the hold/move/patrol/check-move/resolve-blocked/
//! fire-state/evaluate-shot/update-enemy-pos/hide chain are now ported.
//!
//! Ported here: `Sniper_ClearTimers` (NPC_AI_Sniper.c:44),
//! `NPC_Sniper_PlayConfusionSound` (:60), `NPC_Sniper_Pain` (:85),
//! `Sniper_HoldPosition` (:106), `Sniper_Move` (:124),
//! `NPC_BSSniper_Patrol` (:185), `Sniper_CheckMoveState` (:308),
//! `Sniper_ResolveBlockedShot` (:383), `Sniper_CheckFireState` (:442),
//! `Sniper_EvaluateShot` (:488), `Sniper_UpdateEnemyPos` (:605),
//! `Sniper_StartHide` (:631), `Sniper_FaceEnemy` (:508),
//! `NPC_BSSniper_Attack` (:640), `NPC_BSSniper_Default` (:854).

#![allow(non_snake_case)] // C function names (`Sniper_ClearTimers`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define`/enum constants kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::ai_h::{
    SQUAD_COVER, SQUAD_IDLE, SQUAD_RETREAT, SQUAD_SCOUT, SQUAD_STAND_AND_SHOOT, SQUAD_TRANSITION,
};
use crate::codemp::game::b_local_h::{
    CP_APPROACH_ENEMY, CP_CLEAR, CP_CLOSEST, CP_FLANK, CP_HAS_ROUTE, CP_HORZ_DIST_COLL, CP_NEAREST,
};
use crate::codemp::game::b_public_h::{
    SCF_CHASE_ENEMIES, SCF_IGNORE_ALERTS, SCF_LOOK_FOR_ENEMIES, SCF_USE_CP_NEAREST,
};
use crate::codemp::game::g_nav::{
    navInfo_t, FlyingCreature, NAV_HitNavGoal, NIF_COLLISION, NPC_SetMoveGoal,
};
use crate::codemp::game::g_timer::TIMER_Done;
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC};
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::codemp::game::npc_combat::{
    G_SetEnemy, NPC_FindCombatPoint, NPC_FreeCombatPoint, NPC_SetCombatPoint,
};
use crate::codemp::game::npc_goal::{NPC_ReachedGoal, UpdateGoal};
use crate::codemp::game::npc_move::{NAV_GetLastMove, NPC_MoveToGoal};
use crate::codemp::game::npc_senses::{NPC_CheckAlertEvents, NPC_CheckForDanger};
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_UpdateAngles,
};
use crate::codemp::game::npc_combat::{
    NPC_ChangeWeapon, NPC_MaxDistSquaredForWeapon, WeaponThink,
};
use crate::codemp::game::g_utils::G_SoundOnEnt;
use crate::codemp::game::g_local::AEL_DANGER;
use crate::codemp::game::g_timer::TIMER_Get;
use crate::codemp::game::b_public_h::{SCF_ALT_FIRE, SCF_DONT_FIRE};
use crate::codemp::game::bg_weapons_h::WP_DISRUPTOR;
use crate::codemp::game::q_math::DistanceSquared;
use crate::codemp::game::q_shared_h::{BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_WALKING, CHAN_WEAPON};
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleNormalize360, AngleVectors, VectorCompare, VectorCopy,
    VectorMA, VectorNormalize, VectorSubtract,
};
use crate::codemp::game::g_utils::GetAnglesForDirection;
use crate::codemp::game::g_weapon::CalcMuzzlePoint;
use crate::codemp::game::b_public_h::{ENEMY_POS_LAG_STEPS, SPOT_ORIGIN};
use crate::codemp::game::bg_public::MASK_SHOT;
use crate::trap;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::codemp::game::b_public_h::{
    BS_DEFAULT, ENEMY_POS_LAG_INTERVAL, MAX_ENEMY_POS_LAG, SPOT_HEAD_LEAN, SPOT_WEAPON,
};
use crate::codemp::game::bg_public::{EV_CONFUSE1, EV_CONFUSE3};
use crate::codemp::game::bg_weapons_h::WP_EMPLACED_GUN;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_local::{AEL_DISCOVERED, AEL_SUSPICIOUS};
use crate::codemp::game::g_main::{g_entities, g_spskill, level};
use crate::codemp::game::g_public_h::SVF_GLASS_BRUSH;
use crate::codemp::game::g_timer::TIMER_Set;
use crate::codemp::game::bg_public::{EV_PUSHED1, EV_PUSHED3};
use crate::codemp::game::npc_combat::{G_AddVoiceEvent, G_ClearEnemy};
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{vec3_t, PITCH, YAW};

// File-scope statics mirroring the C decls (NPC_AI_Sniper.c:29-34); the unread ones
// feed the still-blocked NAV-bound siblings (`Sniper_Move`/`Sniper_CheckMoveState`/
// `Sniper_FaceEnemy`/`NPC_BSSniper_Attack`).
static mut enemyLOS2: c_int = 0;
static mut enemyCS2: c_int = 0;
static mut faceEnemy2: c_int = 0;
static mut move2: c_int = 0;
static mut shoot2: c_int = 0;
static mut enemyDist2: f32 = 0.0;

const SPF_NO_HIDE: c_int = 2;

// Local state enums
#[allow(dead_code)]
const LSTATE_NONE: c_int = 0;
const LSTATE_UNDERFIRE: c_int = LSTATE_NONE + 1;
#[allow(dead_code)]
const LSTATE_INVESTIGATE: c_int = LSTATE_UNDERFIRE + 1;

pub unsafe fn Sniper_ClearTimers(ent: *mut gentity_t) {
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

pub unsafe fn NPC_Sniper_PlayConfusionSound(self_: *mut gentity_t) {
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

pub unsafe extern "C" fn NPC_Sniper_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
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

unsafe fn Sniper_HoldPosition() {
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

unsafe fn Sniper_Move() -> qboolean {
    let moved: qboolean;
    let mut info: navInfo_t = core::mem::zeroed();

    (*(*addr_of_mut!(NPCInfo))).combatMove = QTRUE; //always move straight toward our goal

    moved = NPC_MoveToGoal(QTRUE);

    //Get the move info
    NAV_GetLastMove(&mut info);

    //FIXME: if we bump into another one of our guys and can't get around him, just stop!
    //If we hit our target, then stop and fire!
    if info.flags & NIF_COLLISION != 0 {
        if info.blocker == (*(*addr_of!(NPC))).enemy {
            Sniper_HoldPosition();
        }
    }

    //If our move failed, then reset
    if moved == QFALSE {
        //couldn't get to enemy
        if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_CHASE_ENEMIES != 0
            && !(*(*addr_of!(NPCInfo))).goalEntity.is_null()
            && (*(*addr_of!(NPCInfo))).goalEntity == (*(*addr_of!(NPC))).enemy
        {
            //we were running after enemy
            //Try to find a combat point that can hit the enemy
            let mut cpFlags = CP_CLEAR | CP_HAS_ROUTE;
            let mut cp: c_int;
            if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_USE_CP_NEAREST != 0 {
                cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                cpFlags |= CP_NEAREST;
            }
            cp = NPC_FindCombatPoint(
                &(*(*addr_of!(NPC))).r.currentOrigin,
                &(*(*addr_of!(NPC))).r.currentOrigin,
                &(*(*addr_of!(NPC))).r.currentOrigin,
                cpFlags,
                32.0,
                -1,
            );
            if cp == -1 && (*(*addr_of!(NPCInfo))).scriptFlags & SCF_USE_CP_NEAREST == 0 {
                //okay, try one by the enemy
                cp = NPC_FindCombatPoint(
                    &(*(*addr_of!(NPC))).r.currentOrigin,
                    &(*(*addr_of!(NPC))).r.currentOrigin,
                    &(*(*(*addr_of!(NPC))).enemy).r.currentOrigin,
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
                    *addr_of!(NPC),
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
        Sniper_HoldPosition();
    }

    moved
}

pub unsafe fn Sniper_EvaluateShot(hit: c_int) -> qboolean {
    let npc = *addr_of!(NPC);

    if (*npc).enemy.is_null() {
        return QFALSE;
    }

    let hitEnt: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(hit as isize);
    if hit == (*(*npc).enemy).s.number
        || (!hitEnt.is_null()
            && !(*hitEnt).client.is_null()
            && (*(*hitEnt).client).playerTeam == (*(*npc).client).enemyTeam)
        || (!hitEnt.is_null()
            && (*hitEnt).takedamage != QFALSE
            && (((*hitEnt).r.svFlags & SVF_GLASS_BRUSH) != 0
                || (*hitEnt).health < 40
                || (*npc).s.weapon == WP_EMPLACED_GUN as c_int))
        || (!hitEnt.is_null() && ((*hitEnt).r.svFlags & SVF_GLASS_BRUSH) != 0)
    {
        //can hit enemy or will hit glass, so shoot anyway
        return QTRUE;
    }
    QFALSE
}

/*
-------------------------
NPC_BSSniper_Attack
-------------------------
*/

pub unsafe fn Sniper_StartHide() {
    let duckTime = Q_irand(2000, 5000);

    TIMER_Set(*addr_of!(NPC), c"duck".as_ptr(), duckTime);
    TIMER_Set(*addr_of!(NPC), c"watch".as_ptr(), 500);
    TIMER_Set(
        *addr_of!(NPC),
        c"attackDelay".as_ptr(),
        duckTime + Q_irand(500, 2000),
    );
}

pub unsafe fn Sniper_UpdateEnemyPos() {
    let mut index: c_int;
    let mut i: c_int;

    i = (MAX_ENEMY_POS_LAG - ENEMY_POS_LAG_INTERVAL) as c_int;
    while i >= 0 {
        index = i / ENEMY_POS_LAG_INTERVAL as c_int;
        if index == 0 {
            CalcEntitySpot(
                (*(*addr_of!(NPC))).enemy,
                SPOT_HEAD_LEAN,
                addr_of_mut!((*(*addr_of_mut!(NPCInfo))).enemyLaggedPos[index as usize]),
            );
            (*(*addr_of_mut!(NPCInfo))).enemyLaggedPos[index as usize][2] -= flrand(2.0, 16.0);
        } else {
            let src = addr_of!((*(*addr_of!(NPCInfo))).enemyLaggedPos[(index - 1) as usize]);
            let dst = addr_of_mut!((*(*addr_of_mut!(NPCInfo))).enemyLaggedPos[index as usize]);
            VectorCopy(&*src, &mut *dst);
        }
        i -= ENEMY_POS_LAG_INTERVAL as c_int;
    }
}

/*
-------------------------
NPC_BSSniper_Patrol
-------------------------
*/

pub unsafe fn NPC_BSSniper_Patrol() {
    //FIXME: pick up on bodies of dead buddies?
    (*(*addr_of_mut!(NPC))).count = 0;

    if (*(*addr_of!(NPCInfo))).confusionTime < (*addr_of!(level)).time {
        //Look for any enemies
        if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            if NPC_CheckPlayerTeamStealth() != QFALSE {
                //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//Should be auto now
                //NPC_AngerSound();
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }

        if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_IGNORE_ALERTS == 0 {
            //Is there danger nearby
            let alertEvent = NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_SUSPICIOUS);
            if NPC_CheckForDanger(alertEvent) != QFALSE {
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            } else {
                //check for other alert events
                //There is an event to look at
                if alertEvent >= 0
                    && (*addr_of!(level)).alertEvents[alertEvent as usize].ID
                        != (*(*addr_of!(NPCInfo))).lastAlertID
                {
                    (*(*addr_of_mut!(NPCInfo))).lastAlertID =
                        (*addr_of!(level)).alertEvents[alertEvent as usize].ID;
                    if (*addr_of!(level)).alertEvents[alertEvent as usize].level == AEL_DISCOVERED {
                        if !(*addr_of!(level)).alertEvents[alertEvent as usize]
                            .owner
                            .is_null()
                            && !(*(*addr_of!(level)).alertEvents[alertEvent as usize].owner)
                                .client
                                .is_null()
                            && (*(*addr_of!(level)).alertEvents[alertEvent as usize].owner).health
                                >= 0
                            && (*(*(*addr_of!(level)).alertEvents[alertEvent as usize].owner).client)
                                .playerTeam
                                == (*(*(*addr_of!(NPC))).client).enemyTeam
                        {
                            //an enemy
                            G_SetEnemy(
                                *addr_of!(NPC),
                                (*addr_of!(level)).alertEvents[alertEvent as usize].owner,
                            );
                            //NPCInfo->enemyLastSeenTime = level.time;
                            TIMER_Set(
                                *addr_of!(NPC),
                                c"attackDelay".as_ptr(),
                                Q_irand(
                                    (6 - (*(*addr_of!(NPCInfo))).stats.aim) * 100,
                                    (6 - (*(*addr_of!(NPCInfo))).stats.aim) * 500,
                                ),
                            );
                        }
                    } else {
                        //FIXME: get more suspicious over time?
                        //Save the position for movement (if necessary)
                        //FIXME: sound?
                        VectorCopy(
                            &(*addr_of!(level)).alertEvents[alertEvent as usize].position,
                            &mut (*(*addr_of_mut!(NPCInfo))).investigateGoal,
                        );
                        (*(*addr_of_mut!(NPCInfo))).investigateDebounceTime =
                            (*addr_of!(level)).time + Q_irand(500, 1000);
                        if (*addr_of!(level)).alertEvents[alertEvent as usize].level == AEL_SUSPICIOUS
                        {
                            //suspicious looks longer
                            (*(*addr_of_mut!(NPCInfo))).investigateDebounceTime +=
                                Q_irand(500, 2500);
                        }
                    }
                }
            }

            if (*(*addr_of!(NPCInfo))).investigateDebounceTime > (*addr_of!(level)).time {
                //FIXME: walk over to it, maybe?  Not if not chase enemies flag
                //NOTE: stops walking or doing anything else below
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];
                let o_yaw: f32;
                let o_pitch: f32;

                VectorSubtract(
                    &(*(*addr_of!(NPCInfo))).investigateGoal,
                    &(*(*(*addr_of!(NPC))).client).renderInfo.eyePoint,
                    &mut dir,
                );
                vectoangles(&dir, &mut angles);

                o_yaw = (*(*addr_of!(NPCInfo))).desiredYaw;
                o_pitch = (*(*addr_of!(NPCInfo))).desiredPitch;
                (*(*addr_of_mut!(NPCInfo))).desiredYaw = angles[YAW];
                (*(*addr_of_mut!(NPCInfo))).desiredPitch = angles[PITCH];

                NPC_UpdateAngles(QTRUE, QTRUE);

                (*(*addr_of_mut!(NPCInfo))).desiredYaw = o_yaw;
                (*(*addr_of_mut!(NPCInfo))).desiredPitch = o_pitch;
                return;
            }
        }
    }

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        (*addr_of_mut!(ucmd)).buttons |= BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
NPC_BSSniper_Idle
-------------------------
*/
/*
void NPC_BSSniper_Idle( void )
{
    //reset our shotcount
    NPC->count = 0;

    //FIXME: check for other alert events?

    //Is there danger nearby?
    if ( NPC_CheckForDanger( NPC_CheckAlertEvents( qtrue, qtrue ) ) )
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

unsafe fn Sniper_CheckMoveState() {
    //See if we're a scout
    if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_CHASE_ENEMIES == 0
    //NPCInfo->behaviorState == BS_STAND_AND_SHOOT )
    {
        if (*(*addr_of!(NPCInfo))).goalEntity == (*(*addr_of!(NPC))).enemy {
            move2 = QFALSE;
            return;
        }
    }
    //See if we're running away
    else if (*(*addr_of!(NPCInfo))).squadState == SQUAD_RETREAT {
        if TIMER_Done(*addr_of!(NPC), c"flee".as_ptr()) != QFALSE {
            (*(*addr_of_mut!(NPCInfo))).squadState = SQUAD_IDLE;
        } else {
            faceEnemy2 = QFALSE;
        }
    } else if (*(*addr_of!(NPCInfo))).squadState == SQUAD_IDLE {
        if (*(*addr_of!(NPCInfo))).goalEntity.is_null() {
            move2 = QFALSE;
            return;
        }
    }

    //See if we're moving towards a goal, not the enemy
    if (*(*addr_of!(NPCInfo))).goalEntity != (*(*addr_of!(NPC))).enemy
        && !(*(*addr_of!(NPCInfo))).goalEntity.is_null()
    {
        //Did we make it?
        if NAV_HitNavGoal(
            &(*(*addr_of!(NPC))).r.currentOrigin,
            &(*(*addr_of!(NPC))).r.mins,
            &(*(*addr_of!(NPC))).r.maxs,
            &(*(*(*addr_of!(NPCInfo))).goalEntity).r.currentOrigin,
            16,
            FlyingCreature(*addr_of!(NPC)),
        ) != QFALSE
            || ((*(*addr_of!(NPCInfo))).squadState == SQUAD_SCOUT
                && enemyLOS2 != 0
                && enemyDist2 <= 10000.0)
        {
            #[allow(unused_assignments)]
            let mut newSquadState = SQUAD_STAND_AND_SHOOT;
            //we got where we wanted to go, set timers based on why we were running
            match (*(*addr_of!(NPCInfo))).squadState {
                x if x == SQUAD_RETREAT => {
                    //was running away
                    TIMER_Set(
                        *addr_of!(NPC),
                        c"duck".as_ptr(),
                        ((*(*(*addr_of!(NPC))).client).pers.maxHealth - (*(*addr_of!(NPC))).health)
                            * 100,
                    );
                    TIMER_Set(*addr_of!(NPC), c"hideTime".as_ptr(), Q_irand(3000, 7000));
                    newSquadState = SQUAD_COVER;
                }
                x if x == SQUAD_TRANSITION => {
                    //was heading for a combat point
                    TIMER_Set(*addr_of!(NPC), c"hideTime".as_ptr(), Q_irand(2000, 4000));
                }
                x if x == SQUAD_SCOUT => { //was running after player
                }
                _ => {}
            }
            let _ = newSquadState;
            NPC_ReachedGoal();
            //don't attack right away
            TIMER_Set(
                *addr_of!(NPC),
                c"attackDelay".as_ptr(),
                Q_irand(
                    (6 - (*(*addr_of!(NPCInfo))).stats.aim) * 50,
                    (6 - (*(*addr_of!(NPCInfo))).stats.aim) * 100,
                ),
            ); //FIXME: Slant for difficulty levels, too?
               //don't do something else just yet
            TIMER_Set(*addr_of!(NPC), c"roamTime".as_ptr(), Q_irand(1000, 4000));
            //stop fleeing
            if (*(*addr_of!(NPCInfo))).squadState == SQUAD_RETREAT {
                TIMER_Set(*addr_of!(NPC), c"flee".as_ptr(), -(*addr_of!(level)).time);
                (*(*addr_of_mut!(NPCInfo))).squadState = SQUAD_IDLE;
            }
            return;
        }

        //keep going, hold of roamTimer until we get there
        TIMER_Set(*addr_of!(NPC), c"roamTime".as_ptr(), Q_irand(4000, 8000));
    }
}

unsafe fn Sniper_ResolveBlockedShot() {
    if TIMER_Done(*addr_of!(NPC), c"duck".as_ptr()) != QFALSE {
        //we're not ducking
        if TIMER_Done(*addr_of!(NPC), c"roamTime".as_ptr()) != QFALSE {
            //not roaming
            //FIXME: try to find another spot from which to hit the enemy
            if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_CHASE_ENEMIES != 0
                && ((*(*addr_of!(NPCInfo))).goalEntity.is_null()
                    || (*(*addr_of!(NPCInfo))).goalEntity == (*(*addr_of!(NPC))).enemy)
            {
                //we were running after enemy
                //Try to find a combat point that can hit the enemy
                let mut cpFlags = CP_CLEAR | CP_HAS_ROUTE;
                let mut cp: c_int;

                if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_USE_CP_NEAREST != 0 {
                    cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                    cpFlags |= CP_NEAREST;
                }
                cp = NPC_FindCombatPoint(
                    &(*(*addr_of!(NPC))).r.currentOrigin,
                    &(*(*addr_of!(NPC))).r.currentOrigin,
                    &(*(*addr_of!(NPC))).r.currentOrigin,
                    cpFlags,
                    32.0,
                    -1,
                );
                if cp == -1 && (*(*addr_of!(NPCInfo))).scriptFlags & SCF_USE_CP_NEAREST == 0 {
                    //okay, try one by the enemy
                    cp = NPC_FindCombatPoint(
                        &(*(*addr_of!(NPC))).r.currentOrigin,
                        &(*(*addr_of!(NPC))).r.currentOrigin,
                        &(*(*(*addr_of!(NPC))).enemy).r.currentOrigin,
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
                        *addr_of!(NPC),
                        &(*addr_of!(level)).combatPoints[cp as usize].origin,
                        8,
                        QTRUE,
                        cp,
                        core::ptr::null_mut(),
                    );
                    TIMER_Set(*addr_of!(NPC), c"duck".as_ptr(), -1);
                    TIMER_Set(*addr_of!(NPC), c"attackDelay".as_ptr(), Q_irand(1000, 3000));
                    return;
                }
            }
        }
    }
    /*
    else
    {//maybe we should stand
        if ( TIMER_Done( NPC, "stand" ) )
        {//stand for as long as we'll be here
            TIMER_Set( NPC, "stand", Q_irand( 500, 2000 ) );
            return;
        }
    }
    //Hmm, can't resolve this by telling them to duck or telling me to stand
    //We need to move!
    TIMER_Set( NPC, "roamTime", -1 );
    TIMER_Set( NPC, "stick", -1 );
    TIMER_Set( NPC, "duck", -1 );
    TIMER_Set( NPC, "attackDelay", Q_irand( 1000, 3000 ) );
    */
}

/*
-------------------------
ST_CheckFireState
-------------------------
*/

unsafe fn Sniper_CheckFireState() {
    if enemyCS2 != 0 {
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
    if Q_irand(0, 1) == 0
        && (*(*addr_of!(NPCInfo))).enemyLastSeenTime != 0
        && (*addr_of!(level)).time - (*(*addr_of!(NPCInfo))).enemyLastSeenTime
            < ((5 - (*(*addr_of!(NPCInfo))).stats.aim) * 1000)
    //FIXME: incorporate skill too?
    {
        if VectorCompare(&vec3_origin, &(*(*addr_of!(NPCInfo))).enemyLastSeenLocation) == 0 {
            //Fire on the last known position
            let mut muzzle: vec3_t = [0.0; 3];
            let mut dir: vec3_t = [0.0; 3];
            let mut angles: vec3_t = [0.0; 3];

            CalcEntitySpot(*addr_of!(NPC), SPOT_WEAPON, addr_of_mut!(muzzle));
            VectorSubtract(
                &(*(*addr_of!(NPCInfo))).enemyLastSeenLocation,
                &muzzle,
                &mut dir,
            );

            VectorNormalize(&mut dir);

            vectoangles(&dir, &mut angles);

            (*(*addr_of_mut!(NPCInfo))).desiredYaw = angles[YAW];
            (*(*addr_of_mut!(NPCInfo))).desiredPitch = angles[PITCH];

            shoot2 = QTRUE;
            //faceEnemy2 = qfalse;
        }
        return;
    } else if (*addr_of!(level)).time - (*(*addr_of!(NPCInfo))).enemyLastSeenTime > 10000 {
        //next time we see him, we'll miss few times first
        (*(*addr_of_mut!(NPC))).count = 0;
    }
}

pub unsafe fn Sniper_FaceEnemy() {
    //FIXME: the ones behind kill holes are facing some arbitrary direction and not firing
    //FIXME: If actually trying to hit enemy, don't fire unless enemy is at least in front of me?
    //FIXME: need to give designers option to make them not miss first few shots
    if !(*(*addr_of!(NPC))).enemy.is_null() {
        let mut muzzle: vec3_t = [0.0; 3];
        let mut target: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0; 3];
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut up: vec3_t = [0.0; 3];
        //Get the positions
        AngleVectors(
            &(*(*(*addr_of!(NPC))).client).ps.viewangles,
            Some(&mut forward),
            Some(&mut right),
            Some(&mut up),
        );
        CalcMuzzlePoint(*addr_of!(NPC), &forward, &right, &up, &mut muzzle);
        //CalcEntitySpot( NPC, SPOT_WEAPON, muzzle );
        CalcEntitySpot((*(*addr_of!(NPC))).enemy, SPOT_ORIGIN, addr_of_mut!(target));

        if enemyDist2 > 65536.0 && (*(*addr_of!(NPCInfo))).stats.aim < 5 {
            //is 256 squared, was 16384 (128*128)
            if (*(*addr_of!(NPC))).count < (5 - (*(*addr_of!(NPCInfo))).stats.aim) {
                //miss a few times first
                if shoot2 != 0
                    && TIMER_Done(*addr_of!(NPC), c"attackDelay".as_ptr()) != QFALSE
                    && (*addr_of!(level)).time >= (*(*addr_of!(NPCInfo))).shotTime
                {
                    //ready to fire again
                    let mut aimError: qboolean = QFALSE;
                    let mut hit: qboolean = QTRUE;
                    let mut tryMissCount: c_int = 0;

                    GetAnglesForDirection(&muzzle, &target, &mut angles);
                    AngleVectors(&angles, Some(&mut forward), Some(&mut right), Some(&mut up));

                    while hit != QFALSE && tryMissCount < 10 {
                        tryMissCount += 1;
                        if Q_irand(0, 1) == 0 {
                            aimError = QTRUE;
                            if Q_irand(0, 1) == 0 {
                                let src = target;
                                VectorMA(
                                    &src,
                                    (*(*(*addr_of!(NPC))).enemy).r.maxs[2] * flrand(1.5, 4.0),
                                    &right,
                                    &mut target,
                                );
                            } else {
                                let src = target;
                                VectorMA(
                                    &src,
                                    (*(*(*addr_of!(NPC))).enemy).r.mins[2] * flrand(1.5, 4.0),
                                    &right,
                                    &mut target,
                                );
                            }
                        }
                        if aimError == QFALSE || Q_irand(0, 1) == 0 {
                            if Q_irand(0, 1) == 0 {
                                let src = target;
                                VectorMA(
                                    &src,
                                    (*(*(*addr_of!(NPC))).enemy).r.maxs[2] * flrand(1.5, 4.0),
                                    &up,
                                    &mut target,
                                );
                            } else {
                                let src = target;
                                VectorMA(
                                    &src,
                                    (*(*(*addr_of!(NPC))).enemy).r.mins[2] * flrand(1.5, 4.0),
                                    &up,
                                    &mut target,
                                );
                            }
                        }
                        let trace = trap::Trace(
                            &muzzle,
                            &vec3_origin,
                            &vec3_origin,
                            &target,
                            (*(*addr_of!(NPC))).s.number,
                            MASK_SHOT,
                        );
                        hit = Sniper_EvaluateShot(trace.entityNum as c_int);
                    }
                    (*(*addr_of_mut!(NPC))).count += 1;
                } else {
                    if enemyLOS2 == 0 {
                        NPC_UpdateAngles(QTRUE, QTRUE);
                        return;
                    }
                }
            } else {
                //based on distance, aim value, difficulty and enemy movement, miss
                //FIXME: incorporate distance as a factor?
                let mut missFactor = 8
                    - ((*(*addr_of!(NPCInfo))).stats.aim + (*addr_of!(g_spskill)).integer) * 3;
                if missFactor > ENEMY_POS_LAG_STEPS as c_int {
                    missFactor = ENEMY_POS_LAG_STEPS as c_int;
                } else if missFactor < 0 {
                    //???
                    missFactor = 0;
                }
                VectorCopy(
                    &(*(*addr_of!(NPCInfo))).enemyLaggedPos[missFactor as usize],
                    &mut target,
                );
            }
            GetAnglesForDirection(&muzzle, &target, &mut angles);
        } else {
            target[2] += flrand(0.0, (*(*(*addr_of!(NPC))).enemy).r.maxs[2]);
            //CalcEntitySpot( NPC->enemy, SPOT_HEAD_LEAN, target );
            GetAnglesForDirection(&muzzle, &target, &mut angles);
        }

        (*(*addr_of_mut!(NPCInfo))).desiredYaw = AngleNormalize360(angles[YAW]);
        (*(*addr_of_mut!(NPCInfo))).desiredPitch = AngleNormalize360(angles[PITCH]);
    }
    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
NPC_BSSniper_Attack
-------------------------
*/

pub unsafe fn NPC_BSSniper_Attack() {
    //Don't do anything if we're hurt
    if (*(*addr_of!(NPC))).painDebounceTime > (*addr_of!(level)).time {
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    //NPC_CheckEnemy( qtrue, qfalse );
    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE
    // !NPC->enemy )//
    {
        (*(*addr_of_mut!(NPC))).enemy = core::ptr::null_mut();
        NPC_BSSniper_Patrol(); //FIXME: or patrol?
        return;
    }

    if TIMER_Done(*addr_of!(NPC), c"flee".as_ptr()) != QFALSE
        && NPC_CheckForDanger(NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_DANGER)) != QFALSE
    {
        //going to run
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    if (*(*addr_of!(NPC))).enemy.is_null() {
        //WTF?  somehow we lost our enemy?
        NPC_BSSniper_Patrol(); //FIXME: or patrol?
        return;
    }

    enemyCS2 = QFALSE;
    enemyLOS2 = QFALSE;
    move2 = QTRUE;
    faceEnemy2 = QFALSE;
    shoot2 = QFALSE;
    enemyDist2 = DistanceSquared(
        &(*(*addr_of!(NPC))).r.currentOrigin,
        &(*(*(*addr_of!(NPC))).enemy).r.currentOrigin,
    );
    if enemyDist2 < 16384.0
    //128 squared
    {
        //too close, so switch to primary fire
        if (*(*(*addr_of!(NPC))).client).ps.weapon == WP_DISRUPTOR {
            //sniping... should be assumed
            if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_ALT_FIRE != 0 {
                //use primary fire
                let trace = trap::Trace(
                    &(*(*(*addr_of!(NPC))).enemy).r.currentOrigin,
                    &(*(*(*addr_of!(NPC))).enemy).r.mins,
                    &(*(*(*addr_of!(NPC))).enemy).r.maxs,
                    &(*(*addr_of!(NPC))).r.currentOrigin,
                    (*(*(*addr_of!(NPC))).enemy).s.number,
                    (*(*(*addr_of!(NPC))).enemy).clipmask,
                );
                if trace.allsolid == 0
                    && trace.startsolid == 0
                    && (trace.fraction == 1.0
                        || trace.entityNum as c_int == (*(*addr_of!(NPC))).s.number)
                {
                    //he can get right to me
                    (*(*addr_of_mut!(NPCInfo))).scriptFlags &= !SCF_ALT_FIRE;
                    //reset fire-timing variables
                    NPC_ChangeWeapon(WP_DISRUPTOR);
                    NPC_UpdateAngles(QTRUE, QTRUE);
                    return;
                }
            }
            //FIXME: switch back if he gets far away again?
        }
    } else if enemyDist2 > 65536.0
    //256 squared
    {
        if (*(*(*addr_of!(NPC))).client).ps.weapon == WP_DISRUPTOR {
            //sniping... should be assumed
            if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_ALT_FIRE == 0 {
                //use primary fire
                (*(*addr_of_mut!(NPCInfo))).scriptFlags |= SCF_ALT_FIRE;
                //reset fire-timing variables
                NPC_ChangeWeapon(WP_DISRUPTOR);
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }
    }

    Sniper_UpdateEnemyPos();
    //can we see our target?
    if NPC_ClearLOS4((*(*addr_of!(NPC))).enemy) != QFALSE
    //|| (NPCInfo->stats.aim >= 5 && gi.inPVS( NPC->client->renderInfo.eyePoint, NPC->enemy->currentOrigin )) )
    {
        let maxShootDist: f32;

        (*(*addr_of_mut!(NPCInfo))).enemyLastSeenTime = (*addr_of!(level)).time;
        VectorCopy(
            &(*(*(*addr_of!(NPC))).enemy).r.currentOrigin,
            &mut (*(*addr_of_mut!(NPCInfo))).enemyLastSeenLocation,
        );
        enemyLOS2 = QTRUE;
        maxShootDist = NPC_MaxDistSquaredForWeapon();
        if enemyDist2 < maxShootDist {
            let mut fwd: vec3_t = [0.0; 3];
            let mut right: vec3_t = [0.0; 3];
            let mut up: vec3_t = [0.0; 3];
            let mut muzzle: vec3_t = [0.0; 3];
            let mut end: vec3_t = [0.0; 3];
            let hit: c_int;

            AngleVectors(
                &(*(*(*addr_of!(NPC))).client).ps.viewangles,
                Some(&mut fwd),
                Some(&mut right),
                Some(&mut up),
            );
            CalcMuzzlePoint(*addr_of!(NPC), &fwd, &right, &up, &mut muzzle);
            VectorMA(&muzzle, 8192.0, &fwd, &mut end);
            let tr = trap::Trace(
                &muzzle,
                &vec3_origin,
                &vec3_origin,
                &end,
                (*(*addr_of!(NPC))).s.number,
                MASK_SHOT,
            );

            hit = tr.entityNum as c_int;
            //can we shoot our target?
            if Sniper_EvaluateShot(hit) != QFALSE {
                enemyCS2 = QTRUE;
            }
        }
    }
    /*
    else if ( gi.inPVS( NPC->enemy->currentOrigin, NPC->currentOrigin ) )
    {
        NPCInfo->enemyLastSeenTime = level.time;
        faceEnemy2 = qtrue;
    }
    */

    if enemyLOS2 != 0 {
        //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
        faceEnemy2 = QTRUE;
    }
    if enemyCS2 != 0 {
        shoot2 = QTRUE;
    } else if (*addr_of!(level)).time - (*(*addr_of!(NPCInfo))).enemyLastSeenTime > 3000 {
        //Hmm, have to get around this bastard... FIXME: this NPCInfo->enemyLastSeenTime builds up when ducked seems to make them want to run when they uncrouch
        Sniper_ResolveBlockedShot();
    }

    //Check for movement to take care of
    Sniper_CheckMoveState();

    //See if we should override shooting decision with any special considerations
    Sniper_CheckFireState();

    if move2 != 0 {
        //move toward goal
        if !(*(*addr_of!(NPCInfo))).goalEntity.is_null()
        //&& ( NPCInfo->goalEntity != NPC->enemy || enemyDist2 > 10000 ) )//100 squared
        {
            move2 = Sniper_Move();
        } else {
            move2 = QFALSE;
        }
    }

    if move2 == 0 {
        if TIMER_Done(*addr_of!(NPC), c"duck".as_ptr()) == QFALSE {
            if TIMER_Done(*addr_of!(NPC), c"watch".as_ptr()) != QFALSE {
                //not while watching
                (*addr_of_mut!(ucmd)).upmove = -127;
            }
        }
        //FIXME: what about leaning?
        //FIXME: also, when stop ducking, start looking, if enemy can see me, chance of ducking back down again
    } else {
        //stop ducking!
        TIMER_Set(*addr_of!(NPC), c"duck".as_ptr(), -1);
    }

    if TIMER_Done(*addr_of!(NPC), c"duck".as_ptr()) != QFALSE
        && TIMER_Done(*addr_of!(NPC), c"watch".as_ptr()) != QFALSE
        && (TIMER_Get(*addr_of!(NPC), c"attackDelay".as_ptr()) - (*addr_of!(level)).time) > 1000
        && (*(*addr_of!(NPC))).attackDebounceTime < (*addr_of!(level)).time
    {
        if enemyLOS2 != 0 && (*(*addr_of!(NPCInfo))).scriptFlags & SCF_ALT_FIRE != 0 {
            if (*(*addr_of!(NPC))).fly_sound_debounce_time < (*addr_of!(level)).time {
                (*(*addr_of_mut!(NPC))).fly_sound_debounce_time = (*addr_of!(level)).time + 2000;
            }
        }
    }

    if faceEnemy2 == 0 {
        //we want to face in the dir we're running
        if move2 != 0 {
            //don't run away and shoot
            (*(*addr_of_mut!(NPCInfo))).desiredYaw =
                (*(*addr_of!(NPCInfo))).lastPathAngles[YAW];
            (*(*addr_of_mut!(NPCInfo))).desiredPitch = 0.0;
            shoot2 = QFALSE;
        }
        NPC_UpdateAngles(QTRUE, QTRUE);
    } else
    // if ( faceEnemy2 )
    {
        //face the enemy
        Sniper_FaceEnemy();
    }

    if (*(*addr_of!(NPCInfo))).scriptFlags & SCF_DONT_FIRE != 0 {
        shoot2 = QFALSE;
    }

    //FIXME: don't shoot right away!
    if shoot2 != 0 {
        //try to shoot if it's time
        if TIMER_Done(*addr_of!(NPC), c"attackDelay".as_ptr()) != QFALSE {
            WeaponThink(QTRUE);
            if (*addr_of!(ucmd)).buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) != 0 {
                G_SoundOnEnt(*addr_of!(NPC), CHAN_WEAPON, "sound/null.wav");
            }

            //took a shot, now hide
            if (*(*addr_of!(NPC))).spawnflags & SPF_NO_HIDE == 0 && Q_irand(0, 1) == 0 {
                //FIXME: do this if in combat point and combat point has duck-type cover... also handle lean-type cover
                Sniper_StartHide();
            } else {
                TIMER_Set(
                    *addr_of!(NPC),
                    c"attackDelay".as_ptr(),
                    (*(*addr_of!(NPCInfo))).shotTime - (*addr_of!(level)).time,
                );
            }
        }
    }
}

pub unsafe fn NPC_BSSniper_Default() {
    if (*(*addr_of!(NPC))).enemy.is_null() {
        //don't have an enemy, look for one
        NPC_BSSniper_Patrol();
    } else
    //if ( NPC->enemy )
    {
        //have an enemy
        NPC_BSSniper_Attack();
    }
}
