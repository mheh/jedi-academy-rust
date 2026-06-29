// name:		be_aas_move.c
//
// desc:		AAS
//
// $Archive: /MissionPack/code/botlib/be_aas_move.c $
// $Author: Ttimo $
// $Revision: 9 $
// $Modtime: 4/13/01 4:45p $
// $Date: 4/13/01 4:45p $

use core::ffi::{c_int, c_float};

// Placeholders for external dependencies to be wired later
extern "C" {
    fn AAS_Trace(origin: *const [c_float; 3], mins: *const [c_float; 3], maxs: *const [c_float; 3], end: *const [c_float; 3], ent: c_int, contents: c_int) -> aas_trace_t;
    fn AAS_PointAreaNum(point: *const [c_float; 3]) -> c_int;
    fn AAS_TraceClientBBox(start: *const [c_float; 3], end: *const [c_float; 3], presencetype: c_int, passent: c_int) -> aas_trace_t;
    fn AAS_PlaneFromNum(planenum: c_int) -> *const aas_plane_t;
    fn AAS_PointContents(point: *const [c_float; 3]) -> c_int;
    fn AAS_PointPresenceType(point: *const [c_float; 3]) -> c_int;
    fn AAS_PresenceTypeBoundingBox(presencetype: c_int, mins: *mut [c_float; 3], maxs: *mut [c_float; 3]);
    fn AAS_TraceAreas(start: *const [c_float; 3], end: *const [c_float; 3], areas: *mut c_int, points: *mut [c_float; 3], maxareas: c_int) -> c_int;
    fn AAS_PointInsideFace(facenum: c_int, point: *const [c_float; 3], epsilon: c_float) -> c_int;
    fn AAS_DebugLine(start: *const [c_float; 3], end: *const [c_float; 3], color: c_int);
    fn AAS_ClearShownDebugLines();
    fn LibVarValue(varname: *const u8, defaultvalue: *const u8) -> c_float;
    fn Com_Memset(dest: *mut core::ffi::c_void, fill: c_int, size: usize);
    fn VectorNormalize(vec: *mut [c_float; 3]) -> c_float;
    fn VectorCopy(src: *const [c_float; 3], dst: *mut [c_float; 3]);
    fn VectorClear(vec: *mut [c_float; 3]);
    fn VectorSubtract(a: *const [c_float; 3], b: *const [c_float; 3], out: *mut [c_float; 3]);
    fn VectorAdd(a: *const [c_float; 3], b: *const [c_float; 3], out: *mut [c_float; 3]);
    fn VectorScale(src: *const [c_float; 3], scale: c_float, dst: *mut [c_float; 3]);
    fn VectorMA(src: *const [c_float; 3], scale: c_float, dir: *const [c_float; 3], out: *mut [c_float; 3]);
    fn VectorLength(vec: *const [c_float; 3]) -> c_float;
    fn DotProduct(a: *const [c_float; 3], b: *const [c_float; 3]) -> c_float;
    fn VectorCompare(a: *const [c_float; 3], b: *const [c_float; 3]) -> c_int;
    fn AngleVectors(angles: *const [c_float; 3], forward: *mut [c_float; 3], right: *mut [c_float; 3], up: *mut [c_float; 3]);
    fn abs(x: c_int) -> c_int;
}

extern "C" {
    static botimport: botlib_import_t;
    static mut aasworld: aas_world_state_t;
}

#[repr(C)]
pub struct botlib_import_t {
    pub Print: unsafe extern "C" fn(c_int, *const u8),
}

#[repr(C)]
pub struct aas_settings_t {
    pub phys_gravitydirection: [c_float; 3],
    pub phys_friction: c_float,
    pub phys_stopspeed: c_float,
    pub phys_gravity: c_float,
    pub phys_waterfriction: c_float,
    pub phys_watergravity: c_float,
    pub phys_maxvelocity: c_float,
    pub phys_maxwalkvelocity: c_float,
    pub phys_maxcrouchvelocity: c_float,
    pub phys_maxswimvelocity: c_float,
    pub phys_walkaccelerate: c_float,
    pub phys_airaccelerate: c_float,
    pub phys_swimaccelerate: c_float,
    pub phys_maxstep: c_float,
    pub phys_maxsteepness: c_float,
    pub phys_maxwaterjump: c_float,
    pub phys_maxbarrier: c_float,
    pub phys_jumpvel: c_float,
    pub phys_falldelta5: c_float,
    pub phys_falldelta10: c_float,
    pub rs_waterjump: c_float,
    pub rs_teleport: c_float,
    pub rs_barrierjump: c_float,
    pub rs_startcrouch: c_float,
    pub rs_startgrapple: c_float,
    pub rs_startwalkoffledge: c_float,
    pub rs_startjump: c_float,
    pub rs_rocketjump: c_float,
    pub rs_bfgjump: c_float,
    pub rs_jumppad: c_float,
    pub rs_aircontrolledjumppad: c_float,
    pub rs_funcbob: c_float,
    pub rs_startelevator: c_float,
    pub rs_falldamage5: c_float,
    pub rs_falldamage10: c_float,
    pub rs_maxfallheight: c_float,
    pub rs_maxjumpfallheight: c_float,
}

#[repr(C)]
pub struct aas_trace_t {
    pub startsolid: c_int,
    pub fraction: c_float,
    pub endpos: [c_float; 3],
    pub ent: c_int,
    pub planenum: c_int,
    pub area: c_int,
    pub lastarea: c_int,
}

#[repr(C)]
pub struct aas_plane_t {
    pub normal: [c_float; 3],
    pub dist: c_float,
}

#[repr(C)]
pub struct aas_face_t {
    pub planenum: c_int,
    pub faceflags: c_int,
}

#[repr(C)]
pub struct aas_area_t {
    pub numfaces: c_int,
    pub firstface: c_int,
}

#[repr(C)]
pub struct aas_areasettings_t {
    pub areaflags: c_int,
    pub presencetype: c_int,
    pub contents: c_int,
}

#[repr(C)]
pub struct aas_world_state_t {
    pub areas: *const aas_area_t,
    pub areasettings: *const aas_areasettings_t,
    pub faceindex: *const c_int,
    pub faces: *const aas_face_t,
    pub planes: *const aas_plane_t,
}

#[repr(C)]
pub struct aas_clientmove_t {
    pub endpos: [c_float; 3],
    pub velocity: [c_float; 3],
    pub trace: aas_trace_t,
    pub stopevent: c_int,
    pub endarea: c_int,
    pub presencetype: c_int,
    pub endcontents: c_int,
    pub time: c_float,
    pub frames: c_int,
}

#[repr(C)]
pub struct aas_reachability_t {
    pub start: [c_float; 3],
    pub end: [c_float; 3],
}

pub static mut aassettings: aas_settings_t = aas_settings_t {
    phys_gravitydirection: [0.0; 3],
    phys_friction: 0.0,
    phys_stopspeed: 0.0,
    phys_gravity: 0.0,
    phys_waterfriction: 0.0,
    phys_watergravity: 0.0,
    phys_maxvelocity: 0.0,
    phys_maxwalkvelocity: 0.0,
    phys_maxcrouchvelocity: 0.0,
    phys_maxswimvelocity: 0.0,
    phys_walkaccelerate: 0.0,
    phys_airaccelerate: 0.0,
    phys_swimaccelerate: 0.0,
    phys_maxstep: 0.0,
    phys_maxsteepness: 0.0,
    phys_maxwaterjump: 0.0,
    phys_maxbarrier: 0.0,
    phys_jumpvel: 0.0,
    phys_falldelta5: 0.0,
    phys_falldelta10: 0.0,
    rs_waterjump: 0.0,
    rs_teleport: 0.0,
    rs_barrierjump: 0.0,
    rs_startcrouch: 0.0,
    rs_startgrapple: 0.0,
    rs_startwalkoffledge: 0.0,
    rs_startjump: 0.0,
    rs_rocketjump: 0.0,
    rs_bfgjump: 0.0,
    rs_jumppad: 0.0,
    rs_aircontrolledjumppad: 0.0,
    rs_funcbob: 0.0,
    rs_startelevator: 0.0,
    rs_falldamage5: 0.0,
    rs_falldamage10: 0.0,
    rs_maxfallheight: 0.0,
    rs_maxjumpfallheight: 0.0,
};

// #define AAS_MOVE_DEBUG

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_DropToFloor(origin: *mut [c_float; 3], mins: *const [c_float; 3], maxs: *const [c_float; 3]) -> c_int
{
    let mut end: [c_float; 3] = [0.0; 3];
    let mut trace: aas_trace_t;

    VectorCopy(origin, &mut end);
    end[2] -= 100.0;
    trace = AAS_Trace(origin, mins, maxs, &end, 0, 0x0002); // CONTENTS_SOLID
    if trace.startsolid != 0 { return 0; } // qfalse
    VectorCopy(&trace.endpos, origin);
    return 1; // qtrue
} //end of the function AAS_DropToFloor

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_InitSettings()
{
    addr_of_mut!(aassettings).write(aas_settings_t {
        phys_gravitydirection: [0.0, 0.0, -1.0],
        phys_friction: LibVarValue(b"phys_friction\0".as_ptr(), b"6\0".as_ptr()),
        phys_stopspeed: LibVarValue(b"phys_stopspeed\0".as_ptr(), b"100\0".as_ptr()),
        phys_gravity: LibVarValue(b"phys_gravity\0".as_ptr(), b"800\0".as_ptr()),
        phys_waterfriction: LibVarValue(b"phys_waterfriction\0".as_ptr(), b"1\0".as_ptr()),
        phys_watergravity: LibVarValue(b"phys_watergravity\0".as_ptr(), b"400\0".as_ptr()),
        phys_maxvelocity: LibVarValue(b"phys_maxvelocity\0".as_ptr(), b"320\0".as_ptr()),
        phys_maxwalkvelocity: LibVarValue(b"phys_maxwalkvelocity\0".as_ptr(), b"320\0".as_ptr()),
        phys_maxcrouchvelocity: LibVarValue(b"phys_maxcrouchvelocity\0".as_ptr(), b"100\0".as_ptr()),
        phys_maxswimvelocity: LibVarValue(b"phys_maxswimvelocity\0".as_ptr(), b"150\0".as_ptr()),
        phys_walkaccelerate: LibVarValue(b"phys_walkaccelerate\0".as_ptr(), b"10\0".as_ptr()),
        phys_airaccelerate: LibVarValue(b"phys_airaccelerate\0".as_ptr(), b"1\0".as_ptr()),
        phys_swimaccelerate: LibVarValue(b"phys_swimaccelerate\0".as_ptr(), b"4\0".as_ptr()),
        phys_maxstep: LibVarValue(b"phys_maxstep\0".as_ptr(), b"19\0".as_ptr()),
        phys_maxsteepness: LibVarValue(b"phys_maxsteepness\0".as_ptr(), b"0.7\0".as_ptr()),
        phys_maxwaterjump: LibVarValue(b"phys_maxwaterjump\0".as_ptr(), b"18\0".as_ptr()),
        phys_maxbarrier: LibVarValue(b"phys_maxbarrier\0".as_ptr(), b"33\0".as_ptr()),
        phys_jumpvel: LibVarValue(b"phys_jumpvel\0".as_ptr(), b"270\0".as_ptr()),
        phys_falldelta5: LibVarValue(b"phys_falldelta5\0".as_ptr(), b"40\0".as_ptr()),
        phys_falldelta10: LibVarValue(b"phys_falldelta10\0".as_ptr(), b"60\0".as_ptr()),
        rs_waterjump: LibVarValue(b"rs_waterjump\0".as_ptr(), b"400\0".as_ptr()),
        rs_teleport: LibVarValue(b"rs_teleport\0".as_ptr(), b"50\0".as_ptr()),
        rs_barrierjump: LibVarValue(b"rs_barrierjump\0".as_ptr(), b"100\0".as_ptr()),
        rs_startcrouch: LibVarValue(b"rs_startcrouch\0".as_ptr(), b"300\0".as_ptr()),
        rs_startgrapple: LibVarValue(b"rs_startgrapple\0".as_ptr(), b"500\0".as_ptr()),
        rs_startwalkoffledge: LibVarValue(b"rs_startwalkoffledge\0".as_ptr(), b"70\0".as_ptr()),
        rs_startjump: LibVarValue(b"rs_startjump\0".as_ptr(), b"300\0".as_ptr()),
        rs_rocketjump: LibVarValue(b"rs_rocketjump\0".as_ptr(), b"500\0".as_ptr()),
        rs_bfgjump: LibVarValue(b"rs_bfgjump\0".as_ptr(), b"500\0".as_ptr()),
        rs_jumppad: LibVarValue(b"rs_jumppad\0".as_ptr(), b"250\0".as_ptr()),
        rs_aircontrolledjumppad: LibVarValue(b"rs_aircontrolledjumppad\0".as_ptr(), b"300\0".as_ptr()),
        rs_funcbob: LibVarValue(b"rs_funcbob\0".as_ptr(), b"300\0".as_ptr()),
        rs_startelevator: LibVarValue(b"rs_startelevator\0".as_ptr(), b"50\0".as_ptr()),
        rs_falldamage5: LibVarValue(b"rs_falldamage5\0".as_ptr(), b"300\0".as_ptr()),
        rs_falldamage10: LibVarValue(b"rs_falldamage10\0".as_ptr(), b"500\0".as_ptr()),
        rs_maxfallheight: LibVarValue(b"rs_maxfallheight\0".as_ptr(), b"0\0".as_ptr()),
        rs_maxjumpfallheight: LibVarValue(b"rs_maxjumpfallheight\0".as_ptr(), b"450\0".as_ptr()),
    });
} //end of the function AAS_InitSettings

//===========================================================================
// returns qtrue if the bot is against a ladder
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_AgainstLadder(origin: *const [c_float; 3]) -> c_int
{
    let mut areanum: c_int;
    let mut i: c_int;
    let mut facenum: c_int;
    let mut side: c_int;
    let mut org: [c_float; 3] = [0.0; 3];
    let mut plane: *const aas_plane_t;
    let mut face: *const aas_face_t;
    let mut area: *const aas_area_t;

    VectorCopy(origin, &mut org);
    areanum = AAS_PointAreaNum(&org);
    if areanum == 0
    {
        org[0] += 1.0;
        areanum = AAS_PointAreaNum(&org);
        if areanum == 0
        {
            org[1] += 1.0;
            areanum = AAS_PointAreaNum(&org);
            if areanum == 0
            {
                org[0] -= 2.0;
                areanum = AAS_PointAreaNum(&org);
                if areanum == 0
                {
                    org[1] -= 2.0;
                    areanum = AAS_PointAreaNum(&org);
                } //end if
            } //end if
        } //end if
    } //end if
    //if in solid... wrrr shouldn't happen
    if areanum == 0 { return 0; } // qfalse
    //if not in a ladder area
    if ((*aasworld.areasettings.offset(areanum as isize)).areaflags & 0x8) == 0 { return 0; } // AREA_LADDER, qfalse
    //if a crouch only area
    if ((*aasworld.areasettings.offset(areanum as isize)).presencetype & 0x1) == 0 { return 0; } // PRESENCE_NORMAL, qfalse
    //
    area = &(*aasworld.areas.offset(areanum as isize));
    i = 0;
    while i < (*area).numfaces
    {
        facenum = *aasworld.faceindex.offset(((*area).firstface + i) as isize);
        side = if facenum < 0 { 1 } else { 0 };
        face = aasworld.faces.offset(abs(facenum) as isize);
        //if the face isn't a ladder face
        if ((*face).faceflags & 0x4) == 0 {
            i += 1;
            continue;
        } // FACE_LADDER
        //get the plane the face is in
        plane = aasworld.planes.offset(((*face).planenum ^ side) as isize);
        //if the origin is pretty close to the plane
        if (DotProduct(addr_of!((*plane).normal), origin) - (*plane).dist).abs() as c_int < 3
        {
            if AAS_PointInsideFace(abs(facenum), origin, 0.1) != 0 { return 1; } // qtrue
        } //end if
        i += 1;
    } //end for
    return 0; // qfalse
} //end of the function AAS_AgainstLadder

//===========================================================================
// returns qtrue if the bot is on the ground
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_OnGround(origin: *const [c_float; 3], presencetype: c_int, passent: c_int) -> c_int
{
    let mut trace: aas_trace_t;
    let mut end: [c_float; 3] = [0.0; 3];
    let up: [c_float; 3] = [0.0, 0.0, 1.0];
    let mut plane: *const aas_plane_t;

    VectorCopy(origin, &mut end);
    end[2] -= 10.0;

    trace = AAS_TraceClientBBox(origin, &end, presencetype, passent);

    //if in solid
    if trace.startsolid != 0 { return 0; } // qfalse
    //if nothing hit at all
    if trace.fraction >= 1.0 { return 0; } // qfalse
    //if too far from the hit plane
    if (*origin)[2] - trace.endpos[2] > 10.0 { return 0; } // qfalse
    //check if the plane isn't too steep
    plane = AAS_PlaneFromNum(trace.planenum);
    if DotProduct(addr_of!((*plane).normal), &up) < addr_of!(aassettings).phys_maxsteepness { return 0; } // qfalse
    //the bot is on the ground
    return 1; // qtrue
} //end of the function AAS_OnGround

//===========================================================================
// returns qtrue if a bot at the given position is swimming
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_Swimming(origin: *const [c_float; 3]) -> c_int
{
    let mut testorg: [c_float; 3] = [0.0; 3];

    VectorCopy(origin, &mut testorg);
    testorg[2] -= 2.0;
    if (AAS_PointContents(&testorg) & (0x0008 | 0x0010 | 0x0020)) != 0 { return 1; } // CONTENTS_LAVA, CONTENTS_SLIME, CONTENTS_WATER, qtrue
    return 0; // qfalse
} //end of the function AAS_Swimming

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
static VEC_UP: [c_float; 3] = [0.0, -1.0,  0.0];
static MOVEDIR_UP: [c_float; 3] = [0.0,  0.0,  1.0];
static VEC_DOWN: [c_float; 3] = [0.0, -2.0,  0.0];
static MOVEDIR_DOWN: [c_float; 3] = [0.0,  0.0, -1.0];

pub unsafe extern "C" fn AAS_SetMovedir(angles: *const [c_float; 3], movedir: *mut [c_float; 3])
{
    if VectorCompare(angles, &VEC_UP) != 0
    {
        VectorCopy(&MOVEDIR_UP, movedir);
    } //end if
    else if VectorCompare(angles, &VEC_DOWN) != 0
    {
        VectorCopy(&MOVEDIR_DOWN, movedir);
    } //end else if
    else
    {
        AngleVectors(angles, movedir, core::ptr::null_mut(), core::ptr::null_mut());
    } //end else
} //end of the function AAS_SetMovedir

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_JumpReachRunStart(reach: *const aas_reachability_t, runstart: *mut [c_float; 3])
{
    let mut hordir: [c_float; 3] = [0.0; 3];
    let mut start: [c_float; 3] = [0.0; 3];
    let mut cmdmove: [c_float; 3] = [0.0; 3];
    let mut move_: aas_clientmove_t = core::mem::zeroed();

    //
    hordir[0] = (*reach).start[0] - (*reach).end[0];
    hordir[1] = (*reach).start[1] - (*reach).end[1];
    hordir[2] = 0.0;
    VectorNormalize(&mut hordir);
    //start point
    VectorCopy(&(*reach).start, &mut start);
    start[2] += 1.0;
    //get command movement
    VectorScale(&hordir, 400.0, &mut cmdmove);
    //
    AAS_PredictClientMovement(&mut move_, -1, &start, 0x1, 1, // PRESENCE_NORMAL
                                &[0.0, 0.0, 0.0], &cmdmove, 1, 2, 0.1,
                                0x00000001 | 0x00000002 | 0x00000004 | 0x00000020 | 0x00010000, // SE_ENTERWATER|SE_ENTERSLIME|SE_ENTERLAVA|SE_HITGROUNDDAMAGE|SE_GAP
                                0, 0);
    VectorCopy(&move_.endpos, runstart);
    //don't enter slime or lava and don't fall from too high
    if (move_.stopevent & (0x00000002 | 0x00000004 | 0x00010000)) != 0 // SE_ENTERSLIME|SE_ENTERLAVA|SE_HITGROUNDDAMAGE
    {
        VectorCopy(&start, runstart);
    } //end if
} //end of the function AAS_JumpReachRunStart

//===========================================================================
// returns the Z velocity when rocket jumping at the origin
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_WeaponJumpZVelocity(origin: *const [c_float; 3], radiusdamage: c_float) -> c_float
{
    let mut kvel: [c_float; 3] = [0.0; 3];
    let mut v: [c_float; 3] = [0.0; 3];
    let mut start: [c_float; 3] = [0.0; 3];
    let mut end: [c_float; 3] = [0.0; 3];
    let mut forward: [c_float; 3] = [0.0; 3];
    let mut right: [c_float; 3] = [0.0; 3];
    let mut viewangles: [c_float; 3] = [0.0; 3];
    let mut dir: [c_float; 3] = [0.0; 3];
    let mut mass: c_float;
    let mut knockback: c_float;
    let mut points: c_float;
    let rocketoffset: [c_float; 3] = [8.0, 8.0, -8.0];
    let botmins: [c_float; 3] = [-16.0, -16.0, -24.0];
    let botmaxs: [c_float; 3] = [16.0, 16.0, 32.0];
    let mut bsptrace: aas_trace_t;

    //look down (90 degrees)
    viewangles[0] = 90.0; // PITCH
    viewangles[1] = 0.0;  // YAW
    viewangles[2] = 0.0;  // ROLL
    //get the start point shooting from
    VectorCopy(origin, &mut start);
    start[2] += 8.0; //view offset Z
    AngleVectors(&viewangles, &mut forward, &mut right, core::ptr::null_mut());
    start[0] += forward[0] * rocketoffset[0] + right[0] * rocketoffset[1];
    start[1] += forward[1] * rocketoffset[0] + right[1] * rocketoffset[1];
    start[2] += forward[2] * rocketoffset[0] + right[2] * rocketoffset[1] + rocketoffset[2];
    //end point of the trace
    VectorMA(&start, 500.0, &forward, &mut end);
    //trace a line to get the impact point
    bsptrace = AAS_Trace(&start, core::ptr::null(), core::ptr::null(), &end, 1, 0x0002); // CONTENTS_SOLID
    //calculate the damage the bot will get from the rocket impact
    VectorAdd(&botmins, &botmaxs, &mut v);
    VectorMA(origin, 0.5, &v, &mut v);
    VectorSubtract(&bsptrace.endpos, &v, &mut v);
    //
    points = radiusdamage - 0.5 * VectorLength(&v);
    if points < 0.0 { points = 0.0; }
    //the owner of the rocket gets half the damage
    points *= 0.5;
    //mass of the bot (p_client.c: PutClientInServer)
    mass = 200.0;
    //knockback is the same as the damage points
    knockback = points;
    //direction of the damage (from trace.endpos to bot origin)
    VectorSubtract(origin, &bsptrace.endpos, &mut dir);
    VectorNormalize(&mut dir);
    //damage velocity
    VectorScale(&dir, 1600.0 * (knockback / mass), &mut kvel);	//the rocket jump hack...
    //rocket impact velocity + jump velocity
    return kvel[2] + addr_of!(aassettings).phys_jumpvel;
} //end of the function AAS_WeaponJumpZVelocity

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_RocketJumpZVelocity(origin: *const [c_float; 3]) -> c_float
{
    //rocket radius damage is 120 (p_weapon.c: Weapon_RocketLauncher_Fire)
    return AAS_WeaponJumpZVelocity(origin, 120.0);
} //end of the function AAS_RocketJumpZVelocity

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_BFGJumpZVelocity(origin: *const [c_float; 3]) -> c_float
{
    //bfg radius damage is 1000 (p_weapon.c: weapon_bfg_fire)
    return AAS_WeaponJumpZVelocity(origin, 120.0);
} //end of the function AAS_BFGJumpZVelocity

//===========================================================================
// applies ground friction to the given velocity
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_Accelerate(velocity: *mut [c_float; 3], frametime: c_float, wishdir: *const [c_float; 3], wishspeed: c_float, accel: c_float)
{
    // q2 style
    let mut i: c_int;
    let mut addspeed: c_float;
    let mut accelspeed: c_float;
    let mut currentspeed: c_float;

    currentspeed = DotProduct(velocity, wishdir);
    addspeed = wishspeed - currentspeed;
    if addspeed <= 0.0 {
        return;
    }
    accelspeed = accel * frametime * wishspeed;
    if accelspeed > addspeed {
        accelspeed = addspeed;
    }

    i = 0;
    while i < 3 {
        (*velocity)[i as usize] += accelspeed * (*wishdir)[i as usize];
        i += 1;
    }
} //end of the function AAS_Accelerate

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_AirControl(start: *const [c_float; 3], end: *const [c_float; 3], velocity: *mut [c_float; 3], cmdmove: *const [c_float; 3])
{
    let mut dir: [c_float; 3] = [0.0; 3];

    VectorSubtract(end, start, &mut dir);
} //end of the function AAS_AirControl

//===========================================================================
// applies ground friction to the given velocity
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ApplyFriction(vel: *mut [c_float; 3], friction: c_float, stopspeed: c_float,
                                            frametime: c_float)
{
    let mut speed: c_float;
    let mut control: c_float;
    let mut newspeed: c_float;

    //horizontal speed
    speed = ((*vel)[0] * (*vel)[0] + (*vel)[1] * (*vel)[1]).sqrt();
    if speed != 0.0
    {
        control = if speed < stopspeed { stopspeed } else { speed };
        newspeed = speed - frametime * control * friction;
        if newspeed < 0.0 { newspeed = 0.0; }
        newspeed /= speed;
        (*vel)[0] *= newspeed;
        (*vel)[1] *= newspeed;
    } //end if
} //end of the function AAS_ApplyFriction

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ClipToBBox(trace: *mut aas_trace_t, start: *const [c_float; 3], end: *const [c_float; 3], presencetype: c_int, mins: *const [c_float; 3], maxs: *const [c_float; 3]) -> c_int
{
    let mut i: c_int;
    let mut j: c_int;
    let mut side: c_int;
    let mut front: c_float;
    let mut back: c_float;
    let mut frac: c_float;
    let mut planedist: c_float;
    let mut bboxmins: [c_float; 3] = [0.0; 3];
    let mut bboxmaxs: [c_float; 3] = [0.0; 3];
    let mut absmins: [c_float; 3] = [0.0; 3];
    let mut absmaxs: [c_float; 3] = [0.0; 3];
    let mut dir: [c_float; 3] = [0.0; 3];
    let mut mid: [c_float; 3] = [0.0; 3];

    AAS_PresenceTypeBoundingBox(presencetype, &mut bboxmins, &mut bboxmaxs);
    VectorSubtract(mins, &bboxmaxs, &mut absmins);
    VectorSubtract(maxs, &bboxmins, &mut absmaxs);
    //
    VectorCopy(end, &mut (*trace).endpos);
    (*trace).fraction = 1.0;
    i = 0;
    while i < 3
    {
        if (*start)[i as usize] < absmins[i as usize] && (*end)[i as usize] < absmins[i as usize] { return 0; } // qfalse
        if (*start)[i as usize] > absmaxs[i as usize] && (*end)[i as usize] > absmaxs[i as usize] { return 0; } // qfalse
        i += 1;
    } //end for
    //check bounding box collision
    VectorSubtract(end, start, &mut dir);
    frac = 1.0;
    i = 0;
    while i < 3
    {
        //get plane to test collision with for the current axis direction
        if dir[i as usize] > 0.0 { planedist = absmins[i as usize]; }
        else { planedist = absmaxs[i as usize]; }
        //calculate collision fraction
        front = (*start)[i as usize] - planedist;
        back = (*end)[i as usize] - planedist;
        frac = front / (front - back);
        //check if between bounding planes of next axis
        side = i + 1;
        if side > 2 { side = 0; }
        mid[side as usize] = (*start)[side as usize] + dir[side as usize] * frac;
        if mid[side as usize] > absmins[side as usize] && mid[side as usize] < absmaxs[side as usize]
        {
            //check if between bounding planes of next axis
            side += 1;
            if side > 2 { side = 0; }
            mid[side as usize] = (*start)[side as usize] + dir[side as usize] * frac;
            if mid[side as usize] > absmins[side as usize] && mid[side as usize] < absmaxs[side as usize]
            {
                mid[i as usize] = planedist;
                break;
            } //end if
        } //end if
        i += 1;
    } //end for
    //if there was a collision
    if i != 3
    {
        (*trace).startsolid = 0; // qfalse
        (*trace).fraction = frac;
        (*trace).ent = 0;
        (*trace).planenum = 0;
        (*trace).area = 0;
        (*trace).lastarea = 0;
        //trace endpos
        j = 0;
        while j < 3 {
            (*trace).endpos[j as usize] = (*start)[j as usize] + dir[j as usize] * frac;
            j += 1;
        }
        return 1; // qtrue
    } //end if
    return 0; // qfalse
} //end of the function AAS_ClipToBBox

//===========================================================================
// predicts the movement
// assumes regular bounding box sizes
// NOTE: out of water jumping is not included
// NOTE: grappling hook is not included
//
// Parameter:			origin			: origin to start with
//						presencetype	: presence type to start with
//						velocity		: velocity to start with
//						cmdmove			: client command movement
//						cmdframes		: number of frame cmdmove is valid
//						maxframes		: maximum number of predicted frames
//						frametime		: duration of one predicted frame
//						stopevent		: events that stop the prediction
//						stopareanum		: stop as soon as entered this area
// Returns:				aas_clientmove_t
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ClientMovementPrediction(move_: *mut aas_clientmove_t,
                                    entnum: c_int, origin: *const [c_float; 3],
                                    presencetype: c_int, onground: c_int,
                                    velocity: *const [c_float; 3], cmdmove: *const [c_float; 3],
                                    cmdframes: c_int,
                                    maxframes: c_int, frametime: c_float,
                                    stopevent: c_int, stopareanum: c_int,
                                    mins: *const [c_float; 3], maxs: *const [c_float; 3], visualize: c_int) -> c_int
{
    let mut phys_friction: c_float;
    let mut phys_stopspeed: c_float;
    let mut phys_gravity: c_float;
    let mut phys_waterfriction: c_float;
    let mut phys_watergravity: c_float;
    let mut phys_walkaccelerate: c_float;
    let mut phys_airaccelerate: c_float;
    let mut phys_swimaccelerate: c_float;
    let mut phys_maxwalkvelocity: c_float;
    let mut phys_maxcrouchvelocity: c_float;
    let mut phys_maxswimvelocity: c_float;
    let mut phys_maxstep: c_float;
    let mut phys_maxsteepness: c_float;
    let mut phys_jumpvel: c_float;
    let mut friction: c_float;
    let mut gravity: c_float;
    let mut delta: c_float;
    let mut maxvel: c_float;
    let mut wishspeed: c_float;
    let mut accelerate: c_float;
    //let mut velchange: c_float;
    //let mut newvel: c_float;
    let mut n: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut pc: c_int;
    let mut step: c_int;
    let mut swimming: c_int;
    let mut ax: c_int;
    let mut crouch: c_int;
    let mut event: c_int;
    let mut jump_frame: c_int;
    let mut areanum: c_int;
    let mut areas: [c_int; 20] = [0; 20];
    let mut numareas: c_int;
    let mut points: [[c_float; 3]; 20] = [[0.0; 3]; 20];
    let mut org: [c_float; 3] = [0.0; 3];
    let mut end: [c_float; 3] = [0.0; 3];
    let mut feet: [c_float; 3] = [0.0; 3];
    let mut start: [c_float; 3] = [0.0; 3];
    let mut stepend: [c_float; 3] = [0.0; 3];
    let mut lastorg: [c_float; 3] = [0.0; 3];
    let mut wishdir: [c_float; 3] = [0.0; 3];
    let mut frame_test_vel: [c_float; 3] = [0.0; 3];
    let mut old_frame_test_vel: [c_float; 3] = [0.0; 3];
    let mut left_test_vel: [c_float; 3] = [0.0; 3];
    let up: [c_float; 3] = [0.0, 0.0, 1.0];
    let mut plane: *const aas_plane_t;
    let mut plane2: *const aas_plane_t;
    let mut trace: aas_trace_t;
    let mut steptrace: aas_trace_t;
    let mut onground = onground;
    let mut presencetype = presencetype;

    if frametime <= 0.0 { frametime = 0.1; }
    //
    phys_friction = addr_of!(aassettings).phys_friction;
    phys_stopspeed = addr_of!(aassettings).phys_stopspeed;
    phys_gravity = addr_of!(aassettings).phys_gravity;
    phys_waterfriction = addr_of!(aassettings).phys_waterfriction;
    phys_watergravity = addr_of!(aassettings).phys_watergravity;
    phys_maxwalkvelocity = addr_of!(aassettings).phys_maxwalkvelocity;// * frametime;
    phys_maxcrouchvelocity = addr_of!(aassettings).phys_maxcrouchvelocity;// * frametime;
    phys_maxswimvelocity = addr_of!(aassettings).phys_maxswimvelocity;// * frametime;
    phys_walkaccelerate = addr_of!(aassettings).phys_walkaccelerate;
    phys_airaccelerate = addr_of!(aassettings).phys_airaccelerate;
    phys_swimaccelerate = addr_of!(aassettings).phys_swimaccelerate;
    phys_maxstep = addr_of!(aassettings).phys_maxstep;
    phys_maxsteepness = addr_of!(aassettings).phys_maxsteepness;
    phys_jumpvel = addr_of!(aassettings).phys_jumpvel * frametime;
    //
    Com_Memset(move_ as *mut core::ffi::c_void, 0, core::mem::size_of::<aas_clientmove_t>());
    Com_Memset(&mut trace as *mut aas_trace_t as *mut core::ffi::c_void, 0, core::mem::size_of::<aas_trace_t>());
    //start at the current origin
    VectorCopy(origin, &mut org);
    org[2] += 0.25;
    //velocity to test for the first frame
    VectorScale(velocity, frametime, &mut frame_test_vel);
    //
    jump_frame = -1;
    //predict a maximum of 'maxframes' ahead
    n = 0;
    while n < maxframes
    {
        swimming = AAS_Swimming(&org);
        //get gravity depending on swimming or not
        gravity = if swimming != 0 { phys_watergravity } else { phys_gravity };
        //apply gravity at the START of the frame
        frame_test_vel[2] = frame_test_vel[2] - (gravity * 0.1 * frametime);
        //if on the ground or swimming
        if onground != 0 || swimming != 0
        {
            friction = if swimming != 0 { phys_friction } else { phys_waterfriction };
            //apply friction
            VectorScale(&frame_test_vel, 1.0 / frametime, &mut frame_test_vel);
            AAS_ApplyFriction(&mut frame_test_vel, friction, phys_stopspeed, frametime);
            VectorScale(&frame_test_vel, frametime, &mut frame_test_vel);
        } //end if
        crouch = 0; // qfalse
        //apply command movement
        if n < cmdframes
        {
            ax = 0;
            maxvel = phys_maxwalkvelocity;
            accelerate = phys_airaccelerate;
            VectorCopy(cmdmove, &mut wishdir);
            if onground != 0
            {
                if (*cmdmove)[2] < -300.0
                {
                    crouch = 1; // qtrue
                    maxvel = phys_maxcrouchvelocity;
                } //end if
                //if not swimming and upmove is positive then jump
                if swimming == 0 && (*cmdmove)[2] > 1.0
                {
                    //jump velocity minus the gravity for one frame + 5 for safety
                    frame_test_vel[2] = phys_jumpvel - (gravity * 0.1 * frametime) + 5.0;
                    jump_frame = n;
                    //jumping so air accelerate
                    accelerate = phys_airaccelerate;
                } //end if
                else
                {
                    accelerate = phys_walkaccelerate;
                } //end else
                ax = 2;
            } //end if
            if swimming != 0
            {
                maxvel = phys_maxswimvelocity;
                accelerate = phys_swimaccelerate;
                ax = 3;
            } //end if
            else
            {
                wishdir[2] = 0.0;
            } //end else
            //
            wishspeed = VectorNormalize(&mut wishdir);
            if wishspeed > maxvel { wishspeed = maxvel; }
            VectorScale(&frame_test_vel, 1.0 / frametime, &mut frame_test_vel);
            AAS_Accelerate(&mut frame_test_vel, frametime, &wishdir, wishspeed, accelerate);
            VectorScale(&frame_test_vel, frametime, &mut frame_test_vel);
            /*
            i = 0;
            while i < ax
            {
                velchange = ((*cmdmove)[i as usize] * frametime) - frame_test_vel[i as usize];
                if velchange > phys_maxacceleration { velchange = phys_maxacceleration; }
                else if velchange < -phys_maxacceleration { velchange = -phys_maxacceleration; }
                newvel = frame_test_vel[i as usize] + velchange;
                //
                if frame_test_vel[i as usize] <= maxvel && newvel > maxvel { frame_test_vel[i as usize] = maxvel; }
                else if frame_test_vel[i as usize] >= -maxvel && newvel < -maxvel { frame_test_vel[i as usize] = -maxvel; }
                else { frame_test_vel[i as usize] = newvel; }
                i += 1;
            } //end for
            */
        } //end if
        if crouch != 0
        {
            presencetype = 0x2; // PRESENCE_CROUCH
        } //end if
        else if presencetype == 0x2 // PRESENCE_CROUCH
        {
            if (AAS_PointPresenceType(&org) & 0x1) != 0 // PRESENCE_NORMAL
            {
                presencetype = 0x1; // PRESENCE_NORMAL
            } //end if
        } //end else
        //save the current origin
        VectorCopy(&org, &mut lastorg);
        //move linear during one frame
        VectorCopy(&frame_test_vel, &mut left_test_vel);
        j = 0;
        loop
        {
            VectorAdd(&org, &left_test_vel, &mut end);
            //trace a bounding box
            trace = AAS_TraceClientBBox(&org, &end, presencetype, entnum);
            //
//#ifdef AAS_MOVE_DEBUG
            if visualize != 0
            {
                if trace.startsolid != 0 {
                    botimport.Print(1, b"PredictMovement: start solid\n\0".as_ptr()); // PRT_MESSAGE
                }
                AAS_DebugLine(&org, &trace.endpos, 1); // LINECOLOR_RED
            } //end if
//#endif //AAS_MOVE_DEBUG
            //
            if (stopevent & (0x00000008 | 0x00000100 | 0x00000200 | 0x00000400)) != 0 // SE_ENTERAREA|SE_TOUCHJUMPPAD|SE_TOUCHTELEPORTER|SE_TOUCHCLUSTERPORTAL
            {
                numareas = AAS_TraceAreas(&org, &trace.endpos, areas.as_mut_ptr(), points.as_mut_ptr(), 20);
                i = 0;
                while i < numareas
                {
                    if (stopevent & 0x00000008) != 0 // SE_ENTERAREA
                    {
                        if areas[i as usize] == stopareanum
                        {
                            VectorCopy(&points[i as usize], &mut (*move_).endpos);
                            VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                            (*move_).endarea = areas[i as usize];
                            (*move_).trace = trace;
                            (*move_).stopevent = 0x00000008; // SE_ENTERAREA
                            (*move_).presencetype = presencetype;
                            (*move_).endcontents = 0;
                            (*move_).time = (n as c_float) * frametime;
                            (*move_).frames = n;
                            return 1; // qtrue
                        } //end if
                    } //end if
                    //NOTE: if not the first frame
                    if (stopevent & 0x00000100) != 0 && n != 0 // SE_TOUCHJUMPPAD
                    {
                        if ((*aasworld.areasettings.offset(areas[i as usize] as isize)).contents & 0x0100) != 0 // AREACONTENTS_JUMPPAD
                        {
                            VectorCopy(&points[i as usize], &mut (*move_).endpos);
                            VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                            (*move_).endarea = areas[i as usize];
                            (*move_).trace = trace;
                            (*move_).stopevent = 0x00000100; // SE_TOUCHJUMPPAD
                            (*move_).presencetype = presencetype;
                            (*move_).endcontents = 0;
                            (*move_).time = (n as c_float) * frametime;
                            (*move_).frames = n;
                            return 1; // qtrue
                        } //end if
                    } //end if
                    if (stopevent & 0x00000200) != 0 // SE_TOUCHTELEPORTER
                    {
                        if ((*aasworld.areasettings.offset(areas[i as usize] as isize)).contents & 0x0200) != 0 // AREACONTENTS_TELEPORTER
                        {
                            VectorCopy(&points[i as usize], &mut (*move_).endpos);
                            (*move_).endarea = areas[i as usize];
                            VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                            (*move_).trace = trace;
                            (*move_).stopevent = 0x00000200; // SE_TOUCHTELEPORTER
                            (*move_).presencetype = presencetype;
                            (*move_).endcontents = 0;
                            (*move_).time = (n as c_float) * frametime;
                            (*move_).frames = n;
                            return 1; // qtrue
                        } //end if
                    } //end if
                    if (stopevent & 0x00000400) != 0 // SE_TOUCHCLUSTERPORTAL
                    {
                        if ((*aasworld.areasettings.offset(areas[i as usize] as isize)).contents & 0x0400) != 0 // AREACONTENTS_CLUSTERPORTAL
                        {
                            VectorCopy(&points[i as usize], &mut (*move_).endpos);
                            (*move_).endarea = areas[i as usize];
                            VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                            (*move_).trace = trace;
                            (*move_).stopevent = 0x00000400; // SE_TOUCHCLUSTERPORTAL
                            (*move_).presencetype = presencetype;
                            (*move_).endcontents = 0;
                            (*move_).time = (n as c_float) * frametime;
                            (*move_).frames = n;
                            return 1; // qtrue
                        } //end if
                    } //end if
                    i += 1;
                } //end for
            } //end if
            //
            if (stopevent & 0x00020000) != 0 // SE_HITBOUNDINGBOX
            {
                if AAS_ClipToBBox(&mut trace, &org, &trace.endpos, presencetype, mins, maxs) != 0
                {
                    VectorCopy(&trace.endpos, &mut (*move_).endpos);
                    (*move_).endarea = AAS_PointAreaNum(&(*move_).endpos);
                    VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                    (*move_).trace = trace;
                    (*move_).stopevent = 0x00020000; // SE_HITBOUNDINGBOX
                    (*move_).presencetype = presencetype;
                    (*move_).endcontents = 0;
                    (*move_).time = (n as c_float) * frametime;
                    (*move_).frames = n;
                    return 1; // qtrue
                } //end if
            } //end if
            //move the entity to the trace end point
            VectorCopy(&trace.endpos, &mut org);
            //if there was a collision
            if trace.fraction < 1.0
            {
                //get the plane the bounding box collided with
                plane = AAS_PlaneFromNum(trace.planenum);
                //
                if (stopevent & 0x00040000) != 0 // SE_HITGROUNDAREA
                {
                    if DotProduct(addr_of!((*plane).normal), &up) > phys_maxsteepness
                    {
                        VectorCopy(&org, &mut start);
                        start[2] += 0.5;
                        if AAS_PointAreaNum(&start) == stopareanum
                        {
                            VectorCopy(&start, &mut (*move_).endpos);
                            (*move_).endarea = stopareanum;
                            VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                            (*move_).trace = trace;
                            (*move_).stopevent = 0x00040000; // SE_HITGROUNDAREA
                            (*move_).presencetype = presencetype;
                            (*move_).endcontents = 0;
                            (*move_).time = (n as c_float) * frametime;
                            (*move_).frames = n;
                            return 1; // qtrue
                        } //end if
                    } //end if
                } //end if
                //assume there's no step
                step = 0; // qfalse
                //if it is a vertical plane and the bot didn't jump recently
                if (*plane).normal[2] == 0.0 && (jump_frame < 0 || n - jump_frame > 2)
                {
                    //check for a step
                    VectorMA(&org, -0.25, addr_of!((*plane).normal), &mut start);
                    VectorCopy(&start, &mut stepend);
                    start[2] += phys_maxstep;
                    steptrace = AAS_TraceClientBBox(&start, &stepend, presencetype, entnum);
                    //
                    if steptrace.startsolid == 0
                    {
                        plane2 = AAS_PlaneFromNum(steptrace.planenum);
                        if DotProduct(addr_of!((*plane2).normal), &up) > phys_maxsteepness
                        {
                            VectorSubtract(&end, &steptrace.endpos, &mut left_test_vel);
                            left_test_vel[2] = 0.0;
                            frame_test_vel[2] = 0.0;
//#ifdef AAS_MOVE_DEBUG
                            if visualize != 0
                            {
                                if steptrace.endpos[2] - org[2] > 0.125
                                {
                                    VectorCopy(&org, &mut start);
                                    start[2] = steptrace.endpos[2];
                                    AAS_DebugLine(&org, &start, 3); // LINECOLOR_BLUE
                                } //end if
                            } //end if
//#endif //AAS_MOVE_DEBUG
                            org[2] = steptrace.endpos[2];
                            step = 1; // qtrue
                        } //end if
                    } //end if
                } //end if
                //
                if step == 0 // qfalse
                {
                    //velocity left to test for this frame is the projection
                    //of the current test velocity into the hit plane
                    VectorMA(&left_test_vel, -DotProduct(&left_test_vel, addr_of!((*plane).normal)),
                                        addr_of!((*plane).normal), &mut left_test_vel);
                    //store the old velocity for landing check
                    VectorCopy(&frame_test_vel, &mut old_frame_test_vel);
                    //test velocity for the next frame is the projection
                    //of the velocity of the current frame into the hit plane
                    VectorMA(&frame_test_vel, -DotProduct(&frame_test_vel, addr_of!((*plane).normal)),
                                        addr_of!((*plane).normal), &mut frame_test_vel);
                    //check for a landing on an almost horizontal floor
                    if DotProduct(addr_of!((*plane).normal), &up) > phys_maxsteepness
                    {
                        onground = 1; // qtrue
                    } //end if
                    if (stopevent & 0x00000020) != 0 // SE_HITGROUNDDAMAGE
                    {
                        delta = 0.0;
                        if old_frame_test_vel[2] < 0.0 &&
                                frame_test_vel[2] > old_frame_test_vel[2] &&
                                onground == 0
                        {
                            delta = old_frame_test_vel[2];
                        } //end if
                        else if onground != 0
                        {
                            delta = frame_test_vel[2] - old_frame_test_vel[2];
                        } //end else
                        if delta != 0.0
                        {
                            delta = delta * 10.0;
                            delta = delta * delta * 0.0001;
                            if swimming != 0 { delta = 0.0; }
                            // never take falling damage if completely underwater
                            /*
                            if (ent->waterlevel == 3) return;
                            if (ent->waterlevel == 2) delta *= 0.25;
                            if (ent->waterlevel == 1) delta *= 0.5;
                            */
                            if delta > 40.0
                            {
                                VectorCopy(&org, &mut (*move_).endpos);
                                (*move_).endarea = AAS_PointAreaNum(&org);
                                VectorCopy(&frame_test_vel, &mut (*move_).velocity);
                                (*move_).trace = trace;
                                (*move_).stopevent = 0x00000020; // SE_HITGROUNDDAMAGE
                                (*move_).presencetype = presencetype;
                                (*move_).endcontents = 0;
                                (*move_).time = (n as c_float) * frametime;
                                (*move_).frames = n;
                                return 1; // qtrue
                            } //end if
                        } //end if
                    } //end if
                } //end if
            } //end if
            //extra check to prevent endless loop
            j += 1;
            if j > 20 { return 0; } // qfalse
        //while there is a plane hit
            if !(trace.fraction < 1.0) { break; }
        }
        //if going down
        if frame_test_vel[2] <= 10.0
        {
            //check for a liquid at the feet of the bot
            VectorCopy(&org, &mut feet);
            feet[2] -= 22.0;
            pc = AAS_PointContents(&feet);
            //get event from pc
            event = 0x0000; // SE_NONE
            if (pc & 0x0008) != 0 { event |= 0x00000004; } // CONTENTS_LAVA, SE_ENTERLAVA
            if (pc & 0x0010) != 0 { event |= 0x00000002; } // CONTENTS_SLIME, SE_ENTERSLIME
            if (pc & 0x0020) != 0 { event |= 0x00000001; } // CONTENTS_WATER, SE_ENTERWATER
            //
            areanum = AAS_PointAreaNum(&org);
            if ((*aasworld.areasettings.offset(areanum as isize)).contents & 0x0001) != 0 // AREACONTENTS_LAVA
            {
                event |= 0x00000004; // SE_ENTERLAVA
            }
            if ((*aasworld.areasettings.offset(areanum as isize)).contents & 0x0002) != 0 // AREACONTENTS_SLIME
            {
                event |= 0x00000002; // SE_ENTERSLIME
            }
            if ((*aasworld.areasettings.offset(areanum as isize)).contents & 0x0004) != 0 // AREACONTENTS_WATER
            {
                event |= 0x00000001; // SE_ENTERWATER
            }
            //if in lava or slime
            if (event & stopevent) != 0
            {
                VectorCopy(&org, &mut (*move_).endpos);
                (*move_).endarea = areanum;
                VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                (*move_).stopevent = event & stopevent;
                (*move_).presencetype = presencetype;
                (*move_).endcontents = pc;
                (*move_).time = (n as c_float) * frametime;
                (*move_).frames = n;
                return 1; // qtrue
            } //end if
        } //end if
        //
        onground = AAS_OnGround(&org, presencetype, entnum);
        //if onground and on the ground for at least one whole frame
        if onground != 0
        {
            if (stopevent & 0x00000010) != 0 // SE_HITGROUND
            {
                VectorCopy(&org, &mut (*move_).endpos);
                (*move_).endarea = AAS_PointAreaNum(&org);
                VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                (*move_).trace = trace;
                (*move_).stopevent = 0x00000010; // SE_HITGROUND
                (*move_).presencetype = presencetype;
                (*move_).endcontents = 0;
                (*move_).time = (n as c_float) * frametime;
                (*move_).frames = n;
                return 1; // qtrue
            } //end if
        } //end if
        else if (stopevent & 0x00008000) != 0 // SE_LEAVEGROUND
        {
            VectorCopy(&org, &mut (*move_).endpos);
            (*move_).endarea = AAS_PointAreaNum(&org);
            VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
            (*move_).trace = trace;
            (*move_).stopevent = 0x00008000; // SE_LEAVEGROUND
            (*move_).presencetype = presencetype;
            (*move_).endcontents = 0;
            (*move_).time = (n as c_float) * frametime;
            (*move_).frames = n;
            return 1; // qtrue
        } //end else if
        else if (stopevent & 0x00010000) != 0 // SE_GAP
        {
            let mut gaptrace: aas_trace_t;

            VectorCopy(&org, &mut start);
            VectorCopy(&start, &mut end);
            end[2] -= 48.0 + addr_of!(aassettings).phys_maxbarrier;
            gaptrace = AAS_TraceClientBBox(&start, &end, 0x2, -1); // PRESENCE_CROUCH
            //if solid is found the bot cannot walk any further and will not fall into a gap
            if gaptrace.startsolid == 0
            {
                //if it is a gap (lower than one step height)
                if gaptrace.endpos[2] < org[2] - addr_of!(aassettings).phys_maxstep - 1.0
                {
                    if (AAS_PointContents(&end) & 0x0020) == 0 // CONTENTS_WATER
                    {
                        VectorCopy(&lastorg, &mut (*move_).endpos);
                        (*move_).endarea = AAS_PointAreaNum(&lastorg);
                        VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
                        (*move_).trace = trace;
                        (*move_).stopevent = 0x00010000; // SE_GAP
                        (*move_).presencetype = presencetype;
                        (*move_).endcontents = 0;
                        (*move_).time = (n as c_float) * frametime;
                        (*move_).frames = n;
                        return 1; // qtrue
                    } //end if
                } //end if
            } //end if
        } //end else if
        n += 1;
    } //end for
    //
    VectorCopy(&org, &mut (*move_).endpos);
    (*move_).endarea = AAS_PointAreaNum(&(*move_).endpos);
    VectorScale(&frame_test_vel, 1.0 / frametime, &mut (*move_).velocity);
    (*move_).stopevent = 0x0000; // SE_NONE
    (*move_).presencetype = presencetype;
    (*move_).endcontents = 0;
    (*move_).time = (n as c_float) * frametime;
    (*move_).frames = n;
    //
    return 1; // qtrue
} //end of the function AAS_ClientMovementPrediction

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_PredictClientMovement(move_: *mut aas_clientmove_t,
                                entnum: c_int, origin: *const [c_float; 3],
                                presencetype: c_int, onground: c_int,
                                velocity: *const [c_float; 3], cmdmove: *const [c_float; 3],
                                cmdframes: c_int,
                                maxframes: c_int, frametime: c_float,
                                stopevent: c_int, stopareanum: c_int, visualize: c_int) -> c_int
{
    let mut mins: [c_float; 3] = [0.0; 3];
    let mut maxs: [c_float; 3] = [0.0; 3];
    return AAS_ClientMovementPrediction(move_, entnum, origin, presencetype, onground,
                                        velocity, cmdmove, cmdframes, maxframes,
                                        frametime, stopevent, stopareanum,
                                        &mins, &maxs, visualize);
} //end of the function AAS_PredictClientMovement

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ClientMovementHitBBox(move_: *mut aas_clientmove_t,
                                entnum: c_int, origin: *const [c_float; 3],
                                presencetype: c_int, onground: c_int,
                                velocity: *const [c_float; 3], cmdmove: *const [c_float; 3],
                                cmdframes: c_int,
                                maxframes: c_int, frametime: c_float,
                                mins: *const [c_float; 3], maxs: *const [c_float; 3], visualize: c_int) -> c_int
{
    return AAS_ClientMovementPrediction(move_, entnum, origin, presencetype, onground,
                                        velocity, cmdmove, cmdframes, maxframes,
                                        frametime, 0x00020000, 0, // SE_HITBOUNDINGBOX
                                        mins, maxs, visualize);
} //end of the function AAS_ClientMovementHitBBox

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_TestMovementPrediction(entnum: c_int, origin: *const [c_float; 3], dir: *mut [c_float; 3])
{
    let mut velocity: [c_float; 3] = [0.0; 3];
    let mut cmdmove: [c_float; 3] = [0.0; 3];
    let mut move_: aas_clientmove_t = core::mem::zeroed();

    VectorClear(&mut velocity);
    if AAS_Swimming(origin) == 0 { (*dir)[2] = 0.0; }
    VectorNormalize(dir);
    VectorScale(dir, 400.0, &mut cmdmove);
    cmdmove[2] = 224.0;
    AAS_ClearShownDebugLines();
    AAS_PredictClientMovement(&mut move_, entnum, origin, 0x1, 1, // PRESENCE_NORMAL, qtrue
                                    &velocity, &cmdmove, 13, 13, 0.1, 0x00000010, 0, 1); // SE_HITGROUND, qtrue
    if (move_.stopevent & 0x00008000) != 0 // SE_LEAVEGROUND
    {
        botimport.Print(1, b"leave ground\n\0".as_ptr()); // PRT_MESSAGE
    } //end if
} //end of the function TestMovementPrediction

//===========================================================================
// calculates the horizontal velocity needed to perform a jump from start
// to end
//
// Parameter:			zvel	: z velocity for jump
//						start	: start position of jump
//						end		: end position of jump
//						*speed	: returned speed for jump
// Returns:				qfalse if too high or too far from start to end
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_HorizontalVelocityForJump(zvel: c_float, start: *const [c_float; 3], end: *const [c_float; 3], velocity: *mut c_float) -> c_int
{
    let mut phys_gravity: c_float;
    let mut phys_maxvelocity: c_float;
    let mut maxjump: c_float;
    let mut height2fall: c_float;
    let mut t: c_float;
    let mut top: c_float;
    let mut dir: [c_float; 3] = [0.0; 3];

    phys_gravity = addr_of!(aassettings).phys_gravity;
    phys_maxvelocity = addr_of!(aassettings).phys_maxvelocity;

    //maximum height a player can jump with the given initial z velocity
    maxjump = 0.5 * phys_gravity * (zvel / phys_gravity) * (zvel / phys_gravity);
    //top of the parabolic jump
    top = (*start)[2] + maxjump;
    //height the bot will fall from the top
    height2fall = top - (*end)[2];
    //if the goal is to high to jump to
    if height2fall < 0.0
    {
        *velocity = phys_maxvelocity;
        return 0;
    } //end if
    //time a player takes to fall the height
    t = (height2fall / (0.5 * phys_gravity)).sqrt();
  	//direction from start to end
    VectorSubtract(end, start, &mut dir);
    //
    if (t + zvel / phys_gravity) == 0.0 {
        *velocity = phys_maxvelocity;
        return 0;
    }
    //calculate horizontal speed
    *velocity = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt() / (t + zvel / phys_gravity);
    //the horizontal speed must be lower than the max speed
    if *velocity > phys_maxvelocity
    {
        *velocity = phys_maxvelocity;
        return 0;
    } //end if
    return 1;
} //end of the function AAS_HorizontalVelocityForJump
