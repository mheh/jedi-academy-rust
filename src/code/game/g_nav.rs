// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// #include "b_local.h"
// #include "g_nav.h"
// #include "g_navigator.h"

use core::ffi::{c_int, c_char, c_uint};

//Global navigator
//CNavigator		navigator;

extern "C" {
    pub fn G_EntIsUnlockedDoor(entityNum: c_int) -> bool;
    pub fn G_EntIsDoor(entityNum: c_int) -> bool;
    pub fn G_EntIsRemovableUsable(entNum: c_int) -> bool;
    pub fn G_FindClosestPointOnLineSegment(start: *const [f32; 3], end: *const [f32; 3], from: *const [f32; 3], result: *mut [f32; 3]) -> bool;
    pub fn G_AddVoiceEvent(self_: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    //For debug graphics
    pub fn CG_Line(start: *mut [f32; 3], end: *mut [f32; 3], color: *mut [f32; 3], alpha: f32);
    pub fn CG_Cube(mins: *mut [f32; 3], maxs: *mut [f32; 3], color: *mut [f32; 3], alpha: f32);
    pub fn CG_CubeOutline(mins: *mut [f32; 3], maxs: *mut [f32; 3], time: c_int, color: c_uint, alpha: f32);
    pub fn FlyingCreature(ent: *mut gentity_t) -> bool;

    pub fn G_CheckInSolid(ent: *mut gentity_t, check_use: bool) -> bool;
    pub fn G_FreeEntity(ent: *mut gentity_t);
    pub fn TAG_Add(name: *const c_char, desc: *const c_char, origin: *const [f32; 3], angles: *const [f32; 3], radius: c_int, flags: c_int);
    pub fn vtos(v: *const [f32; 3]) -> *const c_char;

    pub static NPCDEBUG_RED: [f32; 3];
    pub static mut g_entities: *mut gentity_t;
    pub static mut level: level_locals_t;
}

// Stub types for external dependencies
#[repr(C)]
pub struct gentity_t {
    pub NPC: *mut npdata_t,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub currentOrigin: [f32; 3],
    pub s: entity_state_t,
    pub contents: c_int,
    pub clipmask: c_int,
    pub count: c_int,
    pub classname: *const c_char,
    pub spawnflags: c_int,
    pub targetname: *const c_char,
    pub radius: f32,
    pub target: *mut gentity_t,
    pub waypoint: c_int,
}

#[repr(C)]
pub struct npdata_t {
    pub tempGoal: *mut gentity_t,
    pub goalEntity: *mut gentity_t,
    pub goalRadius: f32,
    pub aiFlags: c_int,
}

#[repr(C)]
pub struct entity_state_t {
    pub number: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub radius: f32,
}

#[repr(C)]
pub struct trace_t {
    pub fraction: f32,
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
}

// Constants
const DEFAULT_MAXS_0: f32 = 20.0;
const DEFAULT_MAXS_1: f32 = 20.0;
const DEFAULT_MAXS_2: f32 = 45.0;
const DEFAULT_MINS_0: f32 = -20.0;
const DEFAULT_MINS_1: f32 = -20.0;
const DEFAULT_MINS_2: f32 = -24.0;
const STEPSIZE: f32 = 18.0;
const MAX_RADIUS_CHECK: f32 = 200.0;
const YAW_ITERATIONS: c_int = 8;
const CONTENTS_SOLID: c_int = 1;
const CONTENTS_MONSTERCLIP: c_int = 8;
const CONTENTS_BOTCLIP: c_int = 16384;
const CONTENTS_TRIGGER: c_int = 0x4000;
const MASK_DEADSOLID: c_int = 1 | 16384;
const CROUCH_MAXS_2: f32 = 32.0;
const WAYPOINT_NONE: c_int = -1;
const ENTITYNUM_NONE: c_int = 2047;
const ENTITYNUM_WORLD: c_int = 2047;
const SVF_NAVGOAL: c_int = 0x0020;
const NPCAI_STOP_AT_LOS: c_int = 0x00000200;
const RTF_NAVGOAL: c_int = 0x0020;

extern "C" {
    pub fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    pub fn VectorSet(v: *mut [f32; 3], x: f32, y: f32, z: f32);
    pub fn VectorMA(veca: *const [f32; 3], scale: f32, vecb: *const [f32; 3], vecc: *mut [f32; 3]);
    pub fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn gi_argv(arg: c_int) -> *const c_char;
    pub fn gi_trace(results: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passent: c_int, contentmask: c_int);
    pub fn gi_linkentity(ent: *mut gentity_t);
    pub fn G_Error(fmt: *const c_char, ...);
    pub fn atoi(nptr: *const c_char) -> c_int;
}

// Namespace-like module for NAV functions
pub mod NAV {
    use super::*;
    extern "C" {
        pub fn SpawnedPoint(ent: *mut gentity_t);
        pub fn TeleportTo(ent: *mut gentity_t, name: *const c_char);
        pub fn TeleportTo_int(ent: *mut gentity_t, num: c_int);
        pub fn ShowStats();
    }
}

// Static variable declaration to match C semantics
extern "C" {
    pub static mut delayedShutDown: c_int;
}

/*
-------------------------
NPC_SetMoveGoal
-------------------------
*/

pub unsafe fn NPC_SetMoveGoal(ent: *mut gentity_t, point: *const [f32; 3], radius: c_int, isNavGoal: bool, combatPoint: c_int, targetEnt: *mut gentity_t)
{
    //Must be an NPC
    if (*ent).NPC == core::ptr::null_mut()
    {
        return;
    }

    if (*(*ent).NPC).tempGoal == core::ptr::null_mut()
    {//must still have a goal
        return;
    }

    //Copy the origin
    //VectorCopy( point, ent->NPC->goalPoint );	//FIXME: Make it use this, and this alone!
    VectorCopy( point, &mut (*(*(*ent).NPC).tempGoal).currentOrigin as *mut [f32; 3] );

    //Copy the mins and maxs to the tempGoal
    VectorCopy( &(*ent).mins as *const [f32; 3], &mut (*(*(*ent).NPC).tempGoal).mins as *mut [f32; 3] );
    VectorCopy( &(*ent).mins as *const [f32; 3], &mut (*(*(*ent).NPC).tempGoal).maxs as *mut [f32; 3] );

    //FIXME: TESTING let's try making sure the tempGoal isn't stuck in the ground?
    if false
    {
        let mut trace: trace_t = core::mem::zeroed();
        let mut bottom: [f32; 3] = [
            (*(*(*ent).NPC).tempGoal).currentOrigin[0],
            (*(*(*ent).NPC).tempGoal).currentOrigin[1],
            (*(*(*ent).NPC).tempGoal).currentOrigin[2] + (*(*(*ent).NPC).tempGoal).mins[2]
        ];
        gi_trace( &mut trace, &(*(*(*ent).NPC).tempGoal).currentOrigin, &[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0], &bottom, (*ent).s.number, (*ent).clipmask );
        if trace.fraction < 1.0f32
        {//in the ground, raise it up
            (*(*(*ent).NPC).tempGoal).currentOrigin[2] -= (*(*(*ent).NPC).tempGoal).mins[2]*(1.0f32-trace.fraction)-0.125f32;
        }
    }

    (*(*(*ent).NPC).tempGoal).target = core::ptr::null_mut();
    (*(*(*ent).NPC).tempGoal).clipmask = (*ent).clipmask;
    (*(*(*ent).NPC).tempGoal).s.number &= !(SVF_NAVGOAL);
    if !targetEnt.is_null() && (*targetEnt).waypoint >= 0
    {
        (*(*(*ent).NPC).tempGoal).waypoint = (*targetEnt).waypoint;
    }
    else
    {
        (*(*(*ent).NPC).tempGoal).waypoint = WAYPOINT_NONE;
    }
    // Placeholder: ent->NPC->tempGoal->noWaypointTime = 0; (member not in stub)

    if isNavGoal
    {
        assert!(!(*(*ent).NPC).tempGoal.is_null());
        (*(*(*ent).NPC).tempGoal).s.number |= SVF_NAVGOAL;
    }

    // Placeholder: ent->NPC->tempGoal->combatPoint = combatPoint; (member not in stub)
    (*(*(*ent).NPC).tempGoal).target = targetEnt;

    (*(*ent).NPC).goalEntity = (*(*ent).NPC).tempGoal;
    (*(*ent).NPC).goalRadius = radius as f32;
    (*(*ent).NPC).aiFlags &= !(NPCAI_STOP_AT_LOS);

    gi_linkentity( (*(*ent).NPC).goalEntity );
}

/*
-------------------------
waypoint_testDirection
-------------------------
*/

unsafe fn waypoint_testDirection( origin: *const [f32; 3], yaw: f32, minDist: f32 ) -> f32
{
    let mut trace_dir: [f32; 3] = [0.0, 0.0, 0.0];
    let mut test_pos: [f32; 3] = [0.0, 0.0, 0.0];
    let mut maxs: [f32; 3] = [0.0, 0.0, 0.0];
    let mut mins: [f32; 3] = [0.0, 0.0, 0.0];
    let mut tr: trace_t = core::mem::zeroed();

    //Setup the mins and max
    VectorSet( &mut maxs, DEFAULT_MAXS_0, DEFAULT_MAXS_1, DEFAULT_MAXS_2 );
    VectorSet( &mut mins, DEFAULT_MINS_0, DEFAULT_MINS_1, DEFAULT_MINS_2 + STEPSIZE );

    //Get our test direction
    let angles: [f32; 3] = [ 0.0, yaw, 0.0 ];
    AngleVectors( &angles, &mut trace_dir, core::ptr::null_mut(), core::ptr::null_mut() );

    //Move ahead
    VectorMA( origin, minDist, &trace_dir, &mut test_pos );

    gi_trace( &mut tr, origin, &mins, &maxs, &test_pos, ENTITYNUM_NONE, ( CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP ) );

    return ( minDist * tr.fraction );	//return actual dist completed
}

/*
-------------------------
waypoint_getRadius
-------------------------
*/

unsafe fn waypoint_getRadius( ent: *mut gentity_t ) -> f32
{
    let mut minDist: f32 = MAX_RADIUS_CHECK + 1.0; // (unsigned int) -1;
    let mut dist: f32;

    for i in 0..YAW_ITERATIONS
    {
        dist = waypoint_testDirection( &(*ent).currentOrigin, ((360.0f32/YAW_ITERATIONS as f32) * i as f32), minDist );

        if dist < minDist
        {
            minDist = dist;
        }
    }

    return minDist + DEFAULT_MAXS_0;
}

/*QUAKED waypoint  (0.7 0.7 0) (-20 -20 -24) (20 20 45) SOLID_OK DROP_TO_FLOOR
a place to go.

SOLID_OK - only use if placing inside solid is unavoidable in map, but may be clear in-game (ie: at the bottom of a tall, solid lift that starts at the top position)
DROP_TO_FLOOR - will cause the point to auto drop to the floor

radius is automatically calculated in-world.
"targetJump" is a special edge that only guys who can jump will cross (so basically Jedi)
*/

pub unsafe fn SP_waypoint( ent: *mut gentity_t )
{
    VectorSet(&mut (*ent).mins, DEFAULT_MINS_0, DEFAULT_MINS_1, DEFAULT_MINS_2);
    VectorSet(&mut (*ent).maxs, DEFAULT_MAXS_0, DEFAULT_MAXS_1, DEFAULT_MAXS_2);

    (*ent).contents = CONTENTS_TRIGGER;
    (*ent).clipmask = MASK_DEADSOLID;

    gi_linkentity( ent );

    (*ent).count = -1;
    (*ent).classname = "waypoint\0".as_ptr() as *const c_char;

    if ((*ent).spawnflags&2) != 0
    {
        (*ent).currentOrigin[2] += 128.0f32;
    }

    if ((*ent).spawnflags&1) == 0 && G_CheckInSolid (ent, true)
    {//if not SOLID_OK, and in solid
        (*ent).maxs[2] = CROUCH_MAXS_2;
        if G_CheckInSolid (ent, true)
        {
            Com_Printf("[S_COLOR_RED]ERROR: Waypoint %s at %s in solid!\n\0".as_ptr() as *const c_char);
            assert!(false, "Waypoint in solid!");
//				if (!g_entities[ENTITYNUM_WORLD].s.radius){{	//not a region
//					G_Error("Waypoint %s at %s in solid!\n", ent->targetname, vtos(ent->currentOrigin));
//				}
            delayedShutDown = level.time + 100;
            G_FreeEntity(ent);
            return;
        }
    }

    //G_SpawnString("targetJump", "", &ent->targetJump);
    (*ent).radius = waypoint_getRadius( ent );
    NAV::SpawnedPoint(ent);

    G_FreeEntity(ent);
    return;
}

/*QUAKED waypoint_small  (0.7 0.7 0) (-2 -2 -24) (2 2 32) SOLID_OK
SOLID_OK - only use if placing inside solid is unavoidable in map, but may be clear in-game (ie: at the bottom of a tall, solid lift that starts at the top position)
DROP_TO_FLOOR - will cause the point to auto drop to the floor
*/

pub unsafe fn SP_waypoint_small (ent: *mut gentity_t)
{
    VectorSet(&mut (*ent).mins, -2.0, -2.0, DEFAULT_MINS_2);
    VectorSet(&mut (*ent).maxs, 2.0, 2.0, DEFAULT_MAXS_2);

    (*ent).contents = CONTENTS_TRIGGER;
    (*ent).clipmask = MASK_DEADSOLID;

    gi_linkentity( ent );

    (*ent).count = -1;
    (*ent).classname = "waypoint\0".as_ptr() as *const c_char;

    if ((*ent).spawnflags&1) == 0 && G_CheckInSolid( ent, true )
    {
        (*ent).maxs[2] = CROUCH_MAXS_2;
        if G_CheckInSolid( ent, true )
        {
            Com_Printf("[S_COLOR_RED]ERROR: Waypoint_small %s at %s in solid!\n\0".as_ptr() as *const c_char);
            assert!(false);
            // #ifndef FINAL_BUILD
            if (*g_entities.add(ENTITYNUM_WORLD as usize)).s.radius == 0 {	//not a region
                G_Error("Waypoint_small %s at %s in solid!\n\0".as_ptr() as *const c_char);
            }
            // #endif
            G_FreeEntity(ent);
            return;
        }
    }

    (*ent).radius = 2.0;	// radius
    NAV::SpawnedPoint(ent);

    G_FreeEntity(ent);
    return;
}


/*QUAKED waypoint_navgoal (0.3 1 0.3) (-20 -20 -24) (20 20 40) SOLID_OK DROP_TO_FLOOR NO_AUTO_CONNECT
A waypoint for script navgoals
Not included in navigation data

DROP_TO_FLOOR - will cause the point to auto drop to the floor
NO_AUTO_CONNECT - will not automatically connect to any other points, you must then connect it by hand


SOLID_OK - only use if placing inside solid is unavoidable in map, but may be clear in-game (ie: at the bottom of a tall, solid lift that starts at the top position)

targetname - name you would use in script when setting a navgoal (like so:)

  For example: if you give this waypoint a targetname of "console", make an NPC go to it in a script like so:

  set ("navgoal", "console");

radius - how far from the navgoal an ent can be before it thinks it reached it - default is "0" which means no radius check, just have to touch it

*/

pub unsafe fn SP_waypoint_navgoal( ent: *mut gentity_t )
{
    let radius: c_int = if (*ent).radius != 0.0 { (*ent).radius as c_int } else { 12 };

    VectorSet( &mut (*ent).mins, -16.0, -16.0, -24.0 );
    VectorSet( &mut (*ent).maxs, 16.0, 16.0, 32.0 );
    (*ent).s.origin[2] += 0.125;
    if ((*ent).spawnflags&1) == 0 && G_CheckInSolid( ent, false )
    {
        Com_Printf("[S_COLOR_RED]ERROR: Waypoint_navgoal %s at %s in solid!\n\0".as_ptr() as *const c_char);
        assert!(false);
        // #ifndef FINAL_BUILD
        if (*g_entities.add(ENTITYNUM_WORLD as usize)).s.radius == 0 {	//not a region
            G_Error("Waypoint_navgoal %s at %s in solid!\n\0".as_ptr() as *const c_char);
        }
        // #endif
    }
    TAG_Add( (*ent).targetname, core::ptr::null(), &(*ent).s.origin, &(*ent).s.angles, radius, RTF_NAVGOAL );

    (*ent).classname = "navgoal\0".as_ptr() as *const c_char;

    // Placeholder: would need PT_GOALNODE constant
    NAV::SpawnedPoint(ent);

    G_FreeEntity( ent );//can't do this, they need to be found later by some functions, though those could be fixed, maybe?
}

/*
-------------------------
Svcmd_Nav_f
-------------------------
*/

pub unsafe fn Svcmd_Nav_f()
{
    let cmd: *const c_char = gi_argv( 1 );

    if Q_stricmp( cmd, "show\0".as_ptr() as *const c_char ) == 0
    {
        let mut cmd = gi_argv( 2 );

        if Q_stricmp( cmd, "all\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showNodes = !NAVDEBUG_showNodes;

            //NOTENOTE: This causes the two states to sync up if they aren't already
            NAVDEBUG_showCollision = NAVDEBUG_showNavGoals =
            NAVDEBUG_showCombatPoints = NAVDEBUG_showEnemyPath =
            NAVDEBUG_showEdges = NAVDEBUG_showNearest = NAVDEBUG_showRadius = NAVDEBUG_showNodes;
        }
        else if Q_stricmp( cmd, "nodes\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showNodes = !NAVDEBUG_showNodes;
        }
        else if Q_stricmp( cmd, "radius\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showRadius = !NAVDEBUG_showRadius;
        }
        else if Q_stricmp( cmd, "edges\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showEdges = !NAVDEBUG_showEdges;
        }
        else if Q_stricmp( cmd, "testpath\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showTestPath = !NAVDEBUG_showTestPath;
        }
        else if Q_stricmp( cmd, "enemypath\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showEnemyPath = !NAVDEBUG_showEnemyPath;
        }
        else if Q_stricmp( cmd, "combatpoints\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showCombatPoints = !NAVDEBUG_showCombatPoints;
        }
        else if Q_stricmp( cmd, "navgoals\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showNavGoals = !NAVDEBUG_showNavGoals;
        }
        else if Q_stricmp( cmd, "collision\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showCollision = !NAVDEBUG_showCollision;
        }
        else if Q_stricmp( cmd, "grid\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showGrid = !NAVDEBUG_showGrid;
        }
        else if Q_stricmp( cmd, "nearest\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showNearest = !NAVDEBUG_showNearest;
        }
        else if Q_stricmp( cmd, "lines\0".as_ptr() as *const c_char ) == 0
        {
            NAVDEBUG_showPointLines = !NAVDEBUG_showPointLines;
        }
    }
    else if Q_stricmp( cmd, "set\0".as_ptr() as *const c_char ) == 0
    {
        let mut cmd = gi_argv( 2 );

        if Q_stricmp( cmd, "testgoal\0".as_ptr() as *const c_char ) == 0
        {
        //	NAVDEBUG_curGoal = navigator.GetNearestNode( &g_entities[0], g_entities[0].waypoint, NF_CLEAR_PATH, WAYPOINT_NONE );
        }
    }
    else if Q_stricmp( cmd, "goto\0".as_ptr() as *const c_char ) == 0
    {
        let mut cmd = gi_argv( 2 );
        NAV::TeleportTo(g_entities, cmd);
    }
    else if Q_stricmp( cmd, "gotonum\0".as_ptr() as *const c_char ) == 0
    {
        let mut cmd = gi_argv( 2 );
        NAV::TeleportTo_int(g_entities, atoi(cmd));
    }
    else if Q_stricmp( cmd, "totals\0".as_ptr() as *const c_char ) == 0
    {
        NAV::ShowStats();
    }
    else
    {
        //Print the available commands
        Com_Printf("nav - valid commands\n---\n\0".as_ptr() as *const c_char );
        Com_Printf("show\n - nodes\n - edges\n - testpath\n - enemypath\n - combatpoints\n - navgoals\n---\n\0".as_ptr() as *const c_char);
        Com_Printf("goto\n ---\n \0".as_ptr() as *const c_char );
        Com_Printf("gotonum\n ---\n \0".as_ptr() as *const c_char );
        Com_Printf("totals\n ---\n \0".as_ptr() as *const c_char );
        Com_Printf("set\n - testgoal\n---\n \0".as_ptr() as *const c_char );
    }
}

//
//JWEIER ADDITIONS START

pub static mut navCalculatePaths: bool = false;

pub static mut NAVDEBUG_showNodes: bool = false;
pub static mut NAVDEBUG_showRadius: bool = false;
pub static mut NAVDEBUG_showEdges: bool = false;
pub static mut NAVDEBUG_showTestPath: bool = false;
pub static mut NAVDEBUG_showEnemyPath: bool = false;
pub static mut NAVDEBUG_showCombatPoints: bool = false;
pub static mut NAVDEBUG_showNavGoals: bool = false;
pub static mut NAVDEBUG_showCollision: bool = false;
pub static mut NAVDEBUG_curGoal: c_int = 0;
pub static mut NAVDEBUG_showGrid: bool = false;
pub static mut NAVDEBUG_showNearest: bool = false;
pub static mut NAVDEBUG_showPointLines: bool = false;

//
//JWEIER ADDITIONS END
