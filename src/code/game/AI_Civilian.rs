// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// (C header: g_headers.h)
// (C header: Q3_Interface.h)

use core::ffi::{c_int, c_void};

// Weapon constants
const WP_NONE: c_int = 0;

// Behavior state constants
const BS_HUNT_AND_KILL: c_int = 0;
const BS_FLEE: c_int = 1;

// Alert event level constants
const AEL_DANGER_GREAT: c_int = 4;

// Animation constants
const BOTH_COWER1: c_int = 0;

// Task ID constants
const TID_MOVE_NAV: c_int = 0;

// Stub type declarations for external game types
#[repr(C)]
pub struct gentity_t {
    _stub: [u8; 0],
}

// External function declarations
extern "C" {
    pub fn NPC_CheckSurrender() -> c_int;
    pub fn NPC_BehaviorSet_Default(bState: c_int);
    pub fn Q3_TaskIDPending(ent: *mut c_void, taskID: c_int) -> c_int;
    pub fn NPC_BSFlee() -> c_int;
    pub fn DistanceSquared(p1: *const c_void, p2: *const c_void) -> f32;
    pub fn NPC_StartFlee(
        enemy: *mut c_void,
        goalpos: *const c_void,
        alertLevel: c_int,
        minDist: c_int,
        maxDist: c_int,
    );
    pub fn VectorCompare(v1: *const c_void, v2: *const c_void) -> c_int;

    // Global variables
    pub static mut NPC: *mut c_void;
    pub static mut NPCInfo: *mut c_void;
}

#[allow(non_snake_case)]
pub fn NPC_BSCivilian_Default(bState: c_int) {
    unsafe {
        if !NPC.is_null() {
            // Mechanical translation of control flow; struct field access requires full gentity_t layout
            // if ( NPC->enemy
            // 	&& NPC->s.weapon == WP_NONE
            // 	&& NPC_CheckSurrender() )
            // {//surrendering, do nothing
            // }
            // else if ( NPC->enemy
            // 	&& NPC->s.weapon == WP_NONE
            // 	&& bState != BS_HUNT_AND_KILL
            // 	&& !Q3_TaskIDPending( NPC, TID_MOVE_NAV ) )
            // {//if in battle and have no weapon, run away, fixme: when in BS_HUNT_AND_KILL, they just stand there
            // 	if ( !NPCInfo->goalEntity
            // 		|| bState != BS_FLEE //not fleeing
            // 		|| ( NPC_BSFlee()//have reached our flee goal
            // 			&& NPC->enemy//still have enemy (NPC_BSFlee checks enemy and can clear it)
            // 			&& DistanceSquared( NPC->currentOrigin, NPC->enemy->currentOrigin ) < 16384 )//enemy within 128
            // 		)
            // 	{//run away!
            // 		NPC_StartFlee( NPC->enemy, NPC->enemy->currentOrigin, AEL_DANGER_GREAT, 5000, 10000 );
            // 	}
            // }
            // else
            // {//not surrendering
            // 	//FIXME: if unarmed and a jawa/ugnuaght, constantly look for enemies/players to run away from?
            // 	//FIXME: if we have a weapon and an enemy, set out playerTeam to the opposite of our enemy..???
            // 	NPC_BehaviorSet_Default(bState);
            // }
            // if ( !VectorCompare( NPC->client->ps.moveDir, vec3_origin ) )
            // {//moving
            // 	if ( NPC->client->ps.legsAnim == BOTH_COWER1 )
            // 	{//stop cowering anim on legs
            // 		NPC->client->ps.legsAnimTimer = 0;
            // 	}
            // }
        }
    }
}
