// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// (C header: g_headers.h)

// b_local.h
// g_nav.h
// anims.h
// g_navigator.h

use core::ffi::{c_int, c_char, c_void};

#![allow(non_snake_case)]

// Stub type declarations for external game types
#[repr(C)]
pub struct gentity_t {
    _stub: [u8; 0],
}

#[repr(C)]
pub struct gentity_s {
    _stub: [u8; 0],
}

#[repr(C)]
pub struct gclient_t {
    _stub: [u8; 0],
}

#[repr(C)]
pub struct gNPC_t {
    _stub: [u8; 0],
}

#[repr(C)]
pub struct trace_t {
    _stub: [u8; 0],
}

// External function declarations
extern "C" {
    pub fn CG_DrawAlert(origin: *const [c_int; 3], rating: f32);
    pub fn G_AddVoiceEvent(self_: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    pub fn NPC_TempLookTarget(self_: *mut gentity_t, lookEntNum: c_int, minLookTime: c_int, maxLookTime: c_int);
    pub fn G_ExpandPointToBBox(point: *mut [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], ignore: c_int, clipmask: c_int) -> c_int;
    pub fn FlyingCreature(ent: *mut gentity_t) -> c_int;
    pub fn Saboteur_Cloak(self_: *mut gentity_t);
    pub fn Saboteur_Decloak(self_: *mut gentity_t);

    pub fn NPC_CheckPlayerTeamStealth() -> c_int;
    pub fn TIMER_Set(ent: *mut gentity_t, name: *const c_char, duration: c_int);
    pub fn TIMER_Done(ent: *const gentity_t, name: *const c_char) -> c_int;
    pub fn TIMER_Get(ent: *const gentity_t, name: *const c_char) -> c_int;
    pub fn NPC_CheckAlertEvents(check: c_int, checkOther: c_int, ignoreAlert: c_int, mustHaveOwner: c_int, minAlert: c_int) -> c_int;
    pub fn NPC_CheckForDanger(alertEvent: c_int) -> c_int;
    pub fn NPC_UpdateAngles(doPitch: c_int, doYaw: c_int);
    pub fn G_SetEnemy(ent: *mut gentity_t, enemy: *mut gentity_t);
    pub fn G_ClearEnemy(ent: *mut gentity_t);
    pub fn NPC_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: c_int, mod_: c_int);
    pub fn NPC_FreeCombatPoint(cp: c_int, qtrue: c_int);
    pub fn UpdateGoal() -> c_int;
    pub fn NPC_MoveToGoal(allowVJump: c_int) -> c_int;
    pub fn NPC_FindCombatPoint(origin1: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], cpFlags: c_int, radius: c_int) -> c_int;
    pub fn NPC_SetCombatPoint(cp: c_int);
    pub fn NPC_SetMoveGoal(ent: *mut gentity_t, goal: *const [f32; 3], radius: c_int, block: c_int, cp: c_int);
    pub fn NPC_CheckEnemyExt() -> c_int;
    pub fn NPC_ClearLOS(ent: *const gentity_t) -> c_int;
    pub fn NPC_MaxDistSquaredForWeapon() -> f32;
    pub fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    pub fn CalcMuzzlePoint(ent: *mut gentity_t, forward: *const [f32; 3], right: *const [f32; 3], up: *const [f32; 3], muzzle: *mut [f32; 3], barrel_num: c_int);
    pub fn CalcEntitySpot(ent: *mut gentity_t, spot: c_int, point: *mut [f32; 3]);
    pub fn VectorMA(veca: *const [f32; 3], scale: f32, vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    pub fn Q_flrand(min: f32, max: f32) -> f32;
    pub fn GetAnglesForDirection(from: *const [f32; 3], to: *const [f32; 3], angles: *mut [f32; 3]);
    pub fn AngleNormalize360(angle: f32) -> f32;
    pub fn NPC_ChangeWeapon(wp: c_int);
    pub fn DistanceSquared(p1: *const [f32; 3], p2: *const [f32; 3]) -> f32;
    pub fn VectorCompare(v1: *const [f32; 3], v2: *const [f32; 3]) -> c_int;
    pub fn NPC_Tusken_Taunt();
    pub fn WeaponThink(make_attack: c_int);
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundPath: *const c_char);
    pub fn NPC_ReachedGoal();
    pub fn VectorCopy(in_: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorSubtract(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn vectoangles(value: *const [f32; 3], angles: *mut [f32; 3]);
    pub fn Q_irand(min: c_int, max: c_int) -> c_int;
    pub fn NPC_BSSniper_Patrol();

    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut gNPC_t;
    pub static mut level: c_void;
    pub static mut g_entities: [gentity_t; 1];
    pub static mut ucmd: c_void;
    pub static g_spskill: *const c_void;
}

// Namespace-like functions (STEER::Reached)
extern "C" {
    pub fn STEER_Reached(NPC: *mut gentity_t, ent: *mut gentity_t, radius: c_int, flying: c_int) -> c_int;
}

// Type aliases for qboolean and vec3_t
pub type qboolean = c_int;
pub const QTRUE: c_int = 1;
pub const QFALSE: c_int = 0;
pub type vec3_t = [f32; 3];

// Macro definitions translated
#[inline]
fn SPF_NO_HIDE() -> c_int { 2 }

const MAX_VIEW_DIST: c_int = 1024;
const MAX_VIEW_SPEED: c_int = 250;
const MAX_LIGHT_INTENSITY: c_int = 255;
const MIN_LIGHT_THRESHOLD: f32 = 0.1;

const DISTANCE_SCALE: f32 = 0.25;
const DISTANCE_THRESHOLD: f32 = 0.075;
const SPEED_SCALE: f32 = 0.25;
const FOV_SCALE: f32 = 0.5;
const LIGHT_SCALE: f32 = 0.25;

const REALIZE_THRESHOLD: f32 = 0.6;
const CAUTIOUS_THRESHOLD: f32 = REALIZE_THRESHOLD * 0.75;

// Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_UNDERFIRE: c_int = 1;
const LSTATE_INVESTIGATE: c_int = 2;

// Squad state constants
const SQUAD_IDLE: c_int = 0;
const SQUAD_STAND_AND_SHOOT: c_int = 1;
const SQUAD_RETREAT: c_int = 2;
const SQUAD_TRANSITION: c_int = 3;
const SQUAD_SCOUT: c_int = 4;
const SQUAD_COVER: c_int = 5;

// Alert event level constants
const AEL_SUSPICIOUS: c_int = 1;
const AEL_DISCOVERED: c_int = 2;
const AEL_DANGER: c_int = 3;

// Script flag constants
const SCF_LOOK_FOR_ENEMIES: c_int = 1;
const SCF_IGNORE_ALERTS: c_int = 2;
const SCF_CHASE_ENEMIES: c_int = 4;
const SCF_DONT_FIRE: c_int = 8;
const SCF_ALT_FIRE: c_int = 16;
const SCF_USE_CP_NEAREST: c_int = 32;

// Combat point constants
const CP_CLEAR: c_int = 1;
const CP_HAS_ROUTE: c_int = 2;
const CP_FLANK: c_int = 4;
const CP_APPROACH_ENEMY: c_int = 8;
const CP_CLOSEST: c_int = 16;
const CP_NEAREST: c_int = 32;
const CP_HORZ_DIST_COLL: c_int = 64;

// Button constants
const BUTTON_WALKING: c_int = 1;
const BUTTON_ATTACK: c_int = 2;
const BUTTON_ALT_ATTACK: c_int = 4;

// Weapon constants
const WP_DISRUPTOR: c_int = 1;
const WP_TUSKEN_RIFLE: c_int = 2;
const WP_EMPLACED_GUN: c_int = 3;

// Entity flags
const SVF_GLASS_BRUSH: c_int = 1;

// Angle constants
const YAW: c_int = 0;
const PITCH: c_int = 1;

// Spot constants
const SPOT_WEAPON: c_int = 0;
const SPOT_ORIGIN: c_int = 1;
const SPOT_HEAD_LEAN: c_int = 2;

// Mask constants
const MASK_SHOT: c_int = 1;

// Class constants
const CLASS_SABOTEUR: c_int = 1;

// Enemy position lag constants
const ENEMY_POS_LAG_STEPS: c_int = 10;
const ENEMY_POS_LAG_INTERVAL: c_int = 1;
const MAX_ENEMY_POS_LAG: c_int = 10;

// Static variables
static mut enemyLOS: qboolean = QFALSE;
static mut enemyCS: qboolean = QFALSE;
static mut faceEnemy: qboolean = QFALSE;
static mut move_val: qboolean = QFALSE;
static mut shoot: qboolean = QFALSE;
static mut enemyDist: f32 = 0.0;

#[allow(non_snake_case)]
pub fn Sniper_ClearTimers(ent: *mut gentity_t) {
    unsafe {
        TIMER_Set(ent, b"chatter\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"duck\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"stand\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"shuffleTime\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"sleepTime\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"enemyLastVisible\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"roamTime\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"hideTime\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"attackDelay\0".as_ptr() as *const c_char, 0);	//FIXME: Slant for difficulty levels
        TIMER_Set(ent, b"stick\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"scoutTime\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"flee\0".as_ptr() as *const c_char, 0);
        TIMER_Set(ent, b"taunting\0".as_ptr() as *const c_char, 0);
    }
}

#[allow(non_snake_case)]
pub fn NPC_Sniper_PlayConfusionSound(self_: *mut gentity_t) {
    //FIXME: make this a custom sound in sound set
    unsafe {
        if (*self_).health > 0 {
            G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000);
        }
        //reset him to be totally unaware again
        TIMER_Set(self_, b"enemyLastVisible\0".as_ptr() as *const c_char, 0);
        TIMER_Set(self_, b"flee\0".as_ptr() as *const c_char, 0);
        (*NPCInfo).squadState = SQUAD_IDLE;
        (*NPCInfo).tempBehavior = BS_DEFAULT;

        //self->NPC->behaviorState = BS_PATROL;
        G_ClearEnemy(self_);//FIXME: or just self->enemy = NULL;?

        (*NPCInfo).investigateCount = 0;
    }
}


/*
-------------------------
NPC_ST_Pain
-------------------------
*/

#[allow(non_snake_case)]
pub fn NPC_Sniper_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: c_int, mod_: c_int) {
    unsafe {
        (*NPCInfo).localState = LSTATE_UNDERFIRE;

        if (*(*self_).client).NPC_class == CLASS_SABOTEUR {
            Saboteur_Decloak(self_);
        }
        TIMER_Set(self_, b"duck\0".as_ptr() as *const c_char, -1);
        TIMER_Set(self_, b"stand\0".as_ptr() as *const c_char, 2000);

        NPC_Pain(self_, inflictor, other, point, damage, mod_);

        if damage == 0 && (*self_).health > 0 {
            //FIXME: better way to know I was pushed
            G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
        }
    }
}

/*
-------------------------
ST_HoldPosition
-------------------------
*/

#[allow(non_snake_case)]
unsafe fn Sniper_HoldPosition() {
    NPC_FreeCombatPoint((*NPCInfo).combatPoint, QTRUE);
    (*NPCInfo).goalEntity = core::ptr::null_mut();

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

#[allow(non_snake_case)]
unsafe fn Sniper_Move() -> qboolean {
    (*NPCInfo).combatMove = QTRUE;//always move straight toward our goal

    let moved = NPC_MoveToGoal(QTRUE);
//	navInfo_t	info;

    //Get the move info
//	NAV_GetLastMove( info );

    //FIXME: if we bump into another one of our guys and can't get around him, just stop!
    //If we hit our target, then stop and fire!
//	if ( info.flags & NIF_COLLISION )
//	{
//		if ( info.blocker == NPC->enemy )
//		{
//			Sniper_HoldPosition();
//		}
//	}

    //If our move failed, then reset
    if moved == QFALSE {
        //couldn't get to enemy
        if ((*NPCInfo).scriptFlags&SCF_CHASE_ENEMIES) != 0 && !(*NPCInfo).goalEntity.is_null() && (*NPCInfo).goalEntity == (*NPC).enemy {
            //we were running after enemy
            //Try to find a combat point that can hit the enemy
            let mut cpFlags = (CP_CLEAR|CP_HAS_ROUTE);
            if ((*NPCInfo).scriptFlags&SCF_USE_CP_NEAREST) != 0 {
                cpFlags &= !(CP_FLANK|CP_APPROACH_ENEMY|CP_CLOSEST);
                cpFlags |= CP_NEAREST;
            }
            let mut cp = NPC_FindCombatPoint((*NPC).currentOrigin, (*NPC).currentOrigin, (*NPC).currentOrigin, cpFlags, 32);
            if cp == -1 && ((*NPCInfo).scriptFlags&SCF_USE_CP_NEAREST) == 0 {
                //okay, try one by the enemy
                cp = NPC_FindCombatPoint((*NPC).currentOrigin, (*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin, CP_CLEAR|CP_HAS_ROUTE|CP_HORZ_DIST_COLL, 32);
            }
            //NOTE: there may be a perfectly valid one, just not one within CP_COLLECT_RADIUS of either me or him...
            if cp != -1 {
                //found a combat point that has a clear shot to enemy
                NPC_SetCombatPoint(cp);
                // level.combatPoints[cp].origin needs to be accessed - stub for now
                NPC_SetMoveGoal(NPC, core::ptr::null(), 8, QTRUE, cp);
                return moved;
            }
        }
        //just hang here
        Sniper_HoldPosition();
    }

    moved
}

/*
-------------------------
NPC_BSSniper_Patrol
-------------------------
*/

#[allow(non_snake_case)]
pub fn NPC_BSSniper_Patrol_impl() {
    //FIXME: pick up on bodies of dead buddies?
    unsafe {
        (*NPC).count = 0;

        if (*NPCInfo).confusionTime < *(level as *mut c_int) {
            //Look for any enemies
            if ((*NPCInfo).scriptFlags&SCF_LOOK_FOR_ENEMIES) != 0 {
                if NPC_CheckPlayerTeamStealth() != 0 {
                    //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//Should be auto now
                    //NPC_AngerSound();
                    NPC_UpdateAngles(QTRUE, QTRUE);
                    return;
                }
            }

            if ((*NPCInfo).scriptFlags&SCF_IGNORE_ALERTS) == 0 {
                //Is there danger nearby
                let alertEvent = NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_SUSPICIOUS);
                if NPC_CheckForDanger(alertEvent) != 0 {
                    NPC_UpdateAngles(QTRUE, QTRUE);
                    return;
                }
                else {
                    //check for other alert events
                    //There is an event to look at
                    if alertEvent >= 0 {
                        //&& level.alertEvents[alertEvent].ID != NPCInfo->lastAlertID )
                        //NPCInfo->lastAlertID = level.alertEvents[alertEvent].ID;
                        // Need level.alertEvents access - stubbing
                    }
                }

                // if ( NPCInfo->investigateDebounceTime > level.time )
                // Investigation code stubbed
            }
        }

        //If we have somewhere to go, then do that
        if UpdateGoal() != 0 {
            // ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(QTRUE);
        }

        NPC_UpdateAngles(QTRUE, QTRUE);
    }
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

#[allow(non_snake_case)]
unsafe fn Sniper_CheckMoveState() {
    //See if we're a scout
    if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
        //NPCInfo->behaviorState == BS_STAND_AND_SHOOT )
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            move_val = QFALSE;
            return;
        }
    }
    //See if we're running away
    else if (*NPCInfo).squadState == SQUAD_RETREAT {
        if TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) != 0 {
            (*NPCInfo).squadState = SQUAD_IDLE;
        }
        else {
            faceEnemy = QFALSE;
        }
    }
    else if (*NPCInfo).squadState == SQUAD_IDLE {
        if (*NPCInfo).goalEntity.is_null() {
            move_val = QFALSE;
            return;
        }
    }

    if TIMER_Done(NPC, b"taunting\0".as_ptr() as *const c_char) == 0 {
        //no move while taunting
        move_val = QFALSE;
        return;
    }

    //See if we're moving towards a goal, not the enemy
    if ((*NPCInfo).goalEntity != (*NPC).enemy) && !(*NPCInfo).goalEntity.is_null() {
        //Did we make it?
        if STEER_Reached(NPC, (*NPCInfo).goalEntity, 16, if FlyingCreature(NPC) != 0 { 1 } else { 0 }) != 0
            || ((*NPCInfo).squadState == SQUAD_SCOUT && enemyLOS != 0 && enemyDist <= 10000.0) {
            let mut newSquadState = SQUAD_STAND_AND_SHOOT;
            //we got where we wanted to go, set timers based on why we were running
            match (*NPCInfo).squadState {
                SQUAD_RETREAT => {
                    //was running away
                    if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
                        Saboteur_Cloak(NPC);
                    }
                    TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, ((*NPC).max_health - (*NPC).health) * 100);
                    TIMER_Set(NPC, b"hideTime\0".as_ptr() as *const c_char, Q_irand(3000, 7000));
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
            NPC_ReachedGoal();
            //don't attack right away
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand((6-(*NPCInfo).stats.aim)*50, (6-(*NPCInfo).stats.aim)*100));	//FIXME: Slant for difficulty levels, too?
            //don't do something else just yet
            TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const c_char, Q_irand(1000, 4000));
            //stop fleeing
            if (*NPCInfo).squadState == SQUAD_RETREAT {
                TIMER_Set(NPC, b"flee\0".as_ptr() as *const c_char, -*(level as *mut c_int));
                (*NPCInfo).squadState = SQUAD_IDLE;
            }
            return;
        }

        //keep going, hold of roamTimer until we get there
        TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const c_char, Q_irand(4000, 8000));
    }
}

#[allow(non_snake_case)]
unsafe fn Sniper_ResolveBlockedShot() {
    if TIMER_Done(NPC, b"duck\0".as_ptr() as *const c_char) != 0 {
        //we're not ducking
        if TIMER_Done(NPC, b"roamTime\0".as_ptr() as *const c_char) != 0 {
            //not roaming
            //FIXME: try to find another spot from which to hit the enemy
            if ((*NPCInfo).scriptFlags&SCF_CHASE_ENEMIES) != 0 && ((*NPCInfo).goalEntity.is_null() || (*NPCInfo).goalEntity == (*NPC).enemy) {
                //we were running after enemy
                //Try to find a combat point that can hit the enemy
                let mut cpFlags = (CP_CLEAR|CP_HAS_ROUTE);
                if ((*NPCInfo).scriptFlags&SCF_USE_CP_NEAREST) != 0 {
                    cpFlags &= !(CP_FLANK|CP_APPROACH_ENEMY|CP_CLOSEST);
                    cpFlags |= CP_NEAREST;
                }
                let mut cp = NPC_FindCombatPoint((*NPC).currentOrigin, (*NPC).currentOrigin, (*NPC).currentOrigin, cpFlags, 32);
                if cp == -1 && ((*NPCInfo).scriptFlags&SCF_USE_CP_NEAREST) == 0 {
                    //okay, try one by the enemy
                    cp = NPC_FindCombatPoint((*NPC).currentOrigin, (*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin, CP_CLEAR|CP_HAS_ROUTE|CP_HORZ_DIST_COLL, 32);
                }
                //NOTE: there may be a perfectly valid one, just not one within CP_COLLECT_RADIUS of either me or him...
                if cp != -1 {
                    //found a combat point that has a clear shot to enemy
                    NPC_SetCombatPoint(cp);
                    NPC_SetMoveGoal(NPC, core::ptr::null(), 8, QTRUE, cp);
                    TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, -1);
                    if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
                        Saboteur_Decloak(NPC);
                    }
                    TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(1000, 3000));
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

#[allow(non_snake_case)]
unsafe fn Sniper_CheckFireState() {
    if enemyCS != 0 {
        //if have a clear shot, always try
        return;
    }

    if (*NPCInfo).squadState == SQUAD_RETREAT || (*NPCInfo).squadState == SQUAD_TRANSITION || (*NPCInfo).squadState == SQUAD_SCOUT {
        //runners never try to fire at the last pos
        return;
    }

    let vec3_origin = [0.0, 0.0, 0.0];
    if VectorCompare((*(*NPC).client).ps.velocity, &vec3_origin) == 0 {
        //if moving at all, don't do this
        return;
    }

    if TIMER_Done(NPC, b"taunting\0".as_ptr() as *const c_char) == 0 {
        //no shoot while taunting
        return;
    }

    //continue to fire on their last position
    if (Q_irand(0, 1) == 0)
        && (*NPCInfo).enemyLastSeenTime != 0
        && *(level as *mut c_int) - (*NPCInfo).enemyLastSeenTime < ((5-(*NPCInfo).stats.aim)*1000) {
        //FIXME: incorporate skill too?
        if VectorCompare(&vec3_origin, (*NPCInfo).enemyLastSeenLocation) == 0 {
            //Fire on the last known position
            let mut muzzle = [0.0; 3];
            let mut dir = [0.0; 3];
            let mut angles = [0.0; 3];

            CalcEntitySpot(NPC, SPOT_WEAPON, &mut muzzle);
            VectorSubtract((*NPCInfo).enemyLastSeenLocation, &muzzle, &mut dir);

            VectorNormalize(&mut dir);

            vectoangles(&dir, &mut angles);

            (*NPCInfo).desiredYaw = angles[YAW as usize];
            (*NPCInfo).desiredPitch = angles[PITCH as usize];

            shoot = QTRUE;
            //faceEnemy = qfalse;
        }
        return;
    }
    else if *(level as *mut c_int) - (*NPCInfo).enemyLastSeenTime > 10000 {
        //next time we see him, we'll miss few times first
        (*NPC).count = 0;
    }
}

#[allow(non_snake_case)]
pub fn Sniper_EvaluateShot(hit: c_int) -> qboolean {
    unsafe {
        if (*NPC).enemy.is_null() {
            return QFALSE;
        }

        let hitEnt = &mut g_entities[hit as usize];
        if hit == (*(*NPC).enemy).s.number
            || (!hitEnt.is_null() && !(*hitEnt as *mut c_void as *const c_void).is_null() && !(*(*hitEnt).client).is_null() && (*(*(*hitEnt).client).playerTeam) == (*(*(*NPC).client).enemyTeam))
            || (!hitEnt.is_null() && (*hitEnt).takedamage != 0 && (((*hitEnt).svFlags&SVF_GLASS_BRUSH) != 0 || (*hitEnt).health < 40 || (*NPC).s.weapon == WP_EMPLACED_GUN))
            || (!hitEnt.is_null() && ((*hitEnt).svFlags&SVF_GLASS_BRUSH) != 0) {
            //can hit enemy or will hit glass, so shoot anyway
            return QTRUE;
        }
        QFALSE
    }
}

#[allow(non_snake_case)]
pub fn Sniper_FaceEnemy() {
    unsafe {
        //FIXME: the ones behind kill holes are facing some arbitrary direction and not firing
        //FIXME: If actually trying to hit enemy, don't fire unless enemy is at least in front of me?
        //FIXME: need to give designers option to make them not miss first few shots
        if !(*NPC).enemy.is_null() {
            let mut muzzle = [0.0; 3];
            let mut target = [0.0; 3];
            let mut angles = [0.0; 3];
            let mut forward = [0.0; 3];
            let mut right = [0.0; 3];
            let mut up = [0.0; 3];
            //Get the positions
            AngleVectors((*(*NPC).client).ps.viewangles, &mut forward, &mut right, &mut up);
            CalcMuzzlePoint(NPC, &forward, &right, &up, &mut muzzle, 0);
            //CalcEntitySpot( NPC, SPOT_WEAPON, muzzle );
            CalcEntitySpot((*NPC).enemy, SPOT_ORIGIN, &mut target);

            if enemyDist > 65536.0 && (*NPCInfo).stats.aim < 5 {
                //is 256 squared, was 16384 (128*128)
                if (*NPC).count < (5-(*NPCInfo).stats.aim) {
                    //miss a few times first
                    if shoot != 0 && TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != 0 && *(level as *mut c_int) >= (*NPCInfo).shotTime {
                        //ready to fire again
                        let mut aimError = QFALSE;
                        let mut hit = QTRUE;
                        let mut tryMissCount = 0;
                        let mut trace = core::mem::zeroed::<trace_t>();

                        GetAnglesForDirection(&muzzle, &target, &mut angles);
                        AngleVectors(&angles, &mut forward, &mut right, &mut up);

                        while hit != 0 && tryMissCount < 10 {
                            tryMissCount += 1;
                            if Q_irand(0, 1) == 0 {
                                aimError = QTRUE;
                                if Q_irand(0, 1) == 0 {
                                    VectorMA(&target, (*(*NPC).enemy).maxs[2]*Q_flrand(1.5, 4.0), &right, &mut target);
                                }
                                else {
                                    VectorMA(&target, (*(*NPC).enemy).mins[2]*Q_flrand(1.5, 4.0), &right, &mut target);
                                }
                            }
                            if aimError == 0 || Q_irand(0, 1) == 0 {
                                if Q_irand(0, 1) == 0 {
                                    VectorMA(&target, (*(*NPC).enemy).maxs[2]*Q_flrand(1.5, 4.0), &up, &mut target);
                                }
                                else {
                                    VectorMA(&target, (*(*NPC).enemy).mins[2]*Q_flrand(1.5, 4.0), &up, &mut target);
                                }
                            }
                            // gi.trace stubbed - requires engine access
                            // gi.trace(&mut trace, &muzzle, vec3_origin, vec3_origin, &target, (*NPC).s.number, MASK_SHOT);
                            // hit = Sniper_EvaluateShot(trace.entityNum);
                            hit = 0; // Stub
                        }
                        (*NPC).count += 1;
                    }
                    else {
                        if enemyLOS == 0 {
                            NPC_UpdateAngles(QTRUE, QTRUE);
                            return;
                        }
                    }
                }
                else {
                    //based on distance, aim value, difficulty and enemy movement, miss
                    //FIXME: incorporate distance as a factor?
                    let mut missFactor = 8-((*NPCInfo).stats.aim+*(g_spskill as *mut c_int)) * 3;
                    if missFactor > ENEMY_POS_LAG_STEPS {
                        missFactor = ENEMY_POS_LAG_STEPS;
                    }
                    else if missFactor < 0 {
                        //???
                        missFactor = 0;
                    }
                    // Need NPCInfo->enemyLaggedPos access - stubbing
                    // VectorCopy((*NPCInfo).enemyLaggedPos[missFactor as usize], &mut target);
                }
                GetAnglesForDirection(&muzzle, &target, &mut angles);
            }
            else {
                target[2] += Q_flrand(0.0, (*(*NPC).enemy).maxs[2]);
                //CalcEntitySpot( NPC->enemy, SPOT_HEAD_LEAN, target );
                GetAnglesForDirection(&muzzle, &target, &mut angles);
            }

            (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW as usize]);
            (*NPCInfo).desiredPitch = AngleNormalize360(angles[PITCH as usize]);
        }
        NPC_UpdateAngles(QTRUE, QTRUE);
    }
}

#[allow(non_snake_case)]
pub fn Sniper_UpdateEnemyPos() {
    unsafe {
        let mut index;
        let mut i = MAX_ENEMY_POS_LAG - ENEMY_POS_LAG_INTERVAL;
        while i >= 0 {
            index = i / ENEMY_POS_LAG_INTERVAL;
            if index == 0 {
                // CalcEntitySpot((*NPC).enemy, SPOT_HEAD_LEAN, (*NPCInfo).enemyLaggedPos[index as usize]);
                // (*NPCInfo).enemyLaggedPos[index as usize][2] -= Q_flrand(2.0, 16.0);
            }
            else {
                // VectorCopy((*NPCInfo).enemyLaggedPos[(index-1) as usize], (*NPCInfo).enemyLaggedPos[index as usize]);
            }
            if i == 0 { break; }
            i -= ENEMY_POS_LAG_INTERVAL;
        }
    }
}

/*
-------------------------
NPC_BSSniper_Attack
-------------------------
*/

#[allow(non_snake_case)]
pub fn Sniper_StartHide() {
    unsafe {
        let duckTime = Q_irand(2000, 5000);

        TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, duckTime);
        if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
            Saboteur_Cloak(NPC);
        }
        TIMER_Set(NPC, b"watch\0".as_ptr() as *const c_char, 500);
        TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, duckTime + Q_irand(500, 2000));
    }
}

#[allow(non_snake_case)]
pub fn NPC_BSSniper_Attack() {
    unsafe {
        //Don't do anything if we're hurt
        if (*NPC).painDebounceTime > *(level as *mut c_int) {
            NPC_UpdateAngles(QTRUE, QTRUE);
            return;
        }

        //NPC_CheckEnemy( qtrue, qfalse );
        //If we don't have an enemy, just idle
        if NPC_CheckEnemyExt() == QFALSE {
            //!NPC->enemy )//
            NPC_BSSniper_Patrol();//FIXME: or patrol?
            return;
        }

        if TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) != 0 && NPC_CheckForDanger(NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_DANGER)) != 0 {
            //going to run
            NPC_UpdateAngles(QTRUE, QTRUE);
            return;
        }

        if (*NPC).enemy.is_null() {
            //WTF?  somehow we lost our enemy?
            NPC_BSSniper_Patrol();//FIXME: or patrol?
            return;
        }

        enemyLOS = QFALSE;
        enemyCS = QFALSE;
        move_val = QTRUE;
        faceEnemy = QFALSE;
        shoot = QFALSE;
        enemyDist = DistanceSquared((*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin);
        if enemyDist < 16384.0 {
            //128 squared
            //too close, so switch to primary fire
            if (*(*NPC).client).ps.weapon == WP_DISRUPTOR
                || (*(*NPC).client).ps.weapon == WP_TUSKEN_RIFLE {
                //sniping... should be assumed
                if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0 {
                    //use primary fire
                    let mut trace = core::mem::zeroed::<trace_t>();
                    // gi.trace stubbed
                    // gi.trace(&mut trace, (*(*NPC).enemy).currentOrigin, (*(*NPC).enemy).mins, (*(*NPC).enemy).maxs, (*NPC).currentOrigin, (*(*NPC).enemy).s.number, (*(*NPC).enemy).clipmask);
                    // if !trace.allsolid && !trace.startsolid && (trace.fraction == 1.0 || trace.entityNum == (*NPC).s.number) {
                    //    //he can get right to me
                    //    (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
                    //    //reset fire-timing variables
                    //    NPC_ChangeWeapon((*(*NPC).client).ps.weapon);
                    //    NPC_UpdateAngles(QTRUE, QTRUE);
                    //    return;
                    // }
                }
                //FIXME: switch back if he gets far away again?
            }
        }
        else if enemyDist > 65536.0 {
            //256 squared
            if (*(*NPC).client).ps.weapon == WP_DISRUPTOR
                || (*(*NPC).client).ps.weapon == WP_TUSKEN_RIFLE {
                //sniping... should be assumed
                if ((*NPCInfo).scriptFlags&SCF_ALT_FIRE) == 0 {
                    //use primary fire
                    (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
                    //reset fire-timing variables
                    NPC_ChangeWeapon((*(*NPC).client).ps.weapon);
                    NPC_UpdateAngles(QTRUE, QTRUE);
                    return;
                }
            }
        }

        Sniper_UpdateEnemyPos();
        //can we see our target?
        if NPC_ClearLOS((*NPC).enemy) != 0 {
            //|| (NPCInfo->stats.aim >= 5 && gi.inPVS( NPC->client->renderInfo.eyePoint, NPC->enemy->currentOrigin )) )
            (*NPCInfo).enemyLastSeenTime = *(level as *mut c_int);
            VectorCopy((*(*NPC).enemy).currentOrigin, (*NPCInfo).enemyLastSeenLocation);
            enemyLOS = QTRUE;
            let maxShootDist = NPC_MaxDistSquaredForWeapon();
            if enemyDist < maxShootDist {
                let mut fwd = [0.0; 3];
                let mut right = [0.0; 3];
                let mut up = [0.0; 3];
                let mut muzzle = [0.0; 3];
                let mut end = [0.0; 3];
                let mut tr = core::mem::zeroed::<trace_t>();
                AngleVectors((*(*NPC).client).ps.viewangles, &mut fwd, &mut right, &mut up);
                CalcMuzzlePoint(NPC, &fwd, &right, &up, &mut muzzle, 0);
                VectorMA(&muzzle, 8192.0, &fwd, &mut end);
                // gi.trace stubbed
                // gi.trace(&mut tr, &muzzle, NULL, NULL, &end, (*NPC).s.number, MASK_SHOT, G2_RETURNONHIT, 0);

                let hit = 0; // tr.entityNum stubbed
                //can we shoot our target?
                if Sniper_EvaluateShot(hit) != 0 {
                    enemyCS = QTRUE;
                }
            }
        }
        /*
        else if ( gi.inPVS( NPC->enemy->currentOrigin, NPC->currentOrigin ) )
        {
            NPCInfo->enemyLastSeenTime = level.time;
            faceEnemy = qtrue;
        }
        */

        if enemyLOS != 0 {
            //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
            faceEnemy = QTRUE;
        }

        if TIMER_Done(NPC, b"taunting\0".as_ptr() as *const c_char) == 0 {
            move_val = QFALSE;
            shoot = QFALSE;
        }
        else if enemyCS != 0 {
            shoot = QTRUE;
        }
        else if *(level as *mut c_int) - (*NPCInfo).enemyLastSeenTime > 3000 {
            //Hmm, have to get around this bastard... FIXME: this NPCInfo->enemyLastSeenTime builds up when ducked seems to make them want to run when they uncrouch
            Sniper_ResolveBlockedShot();
        }
        else if (*(*NPC).client).ps.weapon == WP_TUSKEN_RIFLE && Q_irand(0, 100) == 0 {
            //start a taunt
            NPC_Tusken_Taunt();
            TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, -1);
            move_val = QFALSE;
        }

        //Check for movement to take care of
        Sniper_CheckMoveState();

        //See if we should override shooting decision with any special considerations
        Sniper_CheckFireState();

        if move_val != 0 {
            //move toward goal
            if !(*NPCInfo).goalEntity.is_null() {
                //&& ( NPCInfo->goalEntity != NPC->enemy || enemyDist > 10000 ) )//100 squared
                move_val = Sniper_Move();
            }
            else {
                move_val = QFALSE;
            }
        }

        if move_val == 0 {
            if TIMER_Done(NPC, b"duck\0".as_ptr() as *const c_char) == 0 {
                if TIMER_Done(NPC, b"watch\0".as_ptr() as *const c_char) != 0 {
                    //not while watching
                    // ucmd.upmove = -127;
                    if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
                        Saboteur_Cloak(NPC);
                    }
                }
            }
            //FIXME: what about leaning?
            //FIXME: also, when stop ducking, start looking, if enemy can see me, chance of ducking back down again
        }
        else {
            //stop ducking!
            TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, -1);
            if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
                Saboteur_Decloak(NPC);
            }
        }

        if TIMER_Done(NPC, b"duck\0".as_ptr() as *const c_char) != 0
            && TIMER_Done(NPC, b"watch\0".as_ptr() as *const c_char) != 0
            && (TIMER_Get(NPC, b"attackDelay\0".as_ptr() as *const c_char) - *(level as *mut c_int)) > 1000
            && (*NPC).attackDebounceTime < *(level as *mut c_int) {
            if enemyLOS != 0 && ((*NPCInfo).scriptFlags&SCF_ALT_FIRE) != 0 {
                if (*NPC).fly_sound_debounce_time < *(level as *mut c_int) {
                    (*NPC).fly_sound_debounce_time = *(level as *mut c_int) + 2000;
                }
            }
        }

        if faceEnemy == 0 {
            //we want to face in the dir we're running
            if move_val != 0 {
                //don't run away and shoot
                (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW as usize];
                (*NPCInfo).desiredPitch = 0.0;
                shoot = QFALSE;
            }
            NPC_UpdateAngles(QTRUE, QTRUE);
        }
        else {
            // if ( faceEnemy )
            //face the enemy
            Sniper_FaceEnemy();
        }

        if ((*NPCInfo).scriptFlags&SCF_DONT_FIRE) != 0 {
            shoot = QFALSE;
        }

        //FIXME: don't shoot right away!
        if shoot != 0 {
            //try to shoot if it's time
            if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != 0 {
                WeaponThink(QTRUE);
                // if (ucmd.buttons&(BUTTON_ATTACK|BUTTON_ALT_ATTACK)) {
                //     G_SoundOnEnt(NPC, CHAN_WEAPON, "sound/null.wav");
                // }

                //took a shot, now hide
                if ((*NPC).spawnflags&SPF_NO_HIDE()) == 0 && Q_irand(0, 1) == 0 {
                    //FIXME: do this if in combat point and combat point has duck-type cover... also handle lean-type cover
                    Sniper_StartHide();
                }
                else {
                    TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, (*NPCInfo).shotTime - *(level as *mut c_int));
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn NPC_BSSniper_Default() {
    unsafe {
        if (*NPC).enemy.is_null() {
            //don't have an enemy, look for one
            NPC_BSSniper_Patrol();
        }
        else {
            //if ( NPC->enemy )
            //have an enemy
            NPC_BSSniper_Attack();
        }
    }
}

// Constants for constants that were computed - stub values for now
const EV_CONFUSE1: c_int = 1;
const EV_CONFUSE3: c_int = 3;
const EV_PUSHED1: c_int = 4;
const EV_PUSHED3: c_int = 6;
const BS_DEFAULT: c_int = 0;
const CHAN_WEAPON: c_int = 1;
