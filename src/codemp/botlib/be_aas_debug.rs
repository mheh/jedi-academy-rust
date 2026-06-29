/*****************************************************************************
 * name:		be_aas_debug.c
 *
 * desc:		AAS debug code
 *
 * $Archive: /MissionPack/code/botlib/be_aas_debug.c $
 * $Author: Ttimo $
 * $Revision: 8 $
 * $Modtime: 4/22/01 8:52a $
 * $Date: 4/22/01 8:52a $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_float};
use libc;

// Import types and macros from other ported modules
// These are stubs - the actual types should be defined in their respective modules
type vec3_t = [c_float; 3];
const qfalse: c_int = 0;
const qtrue: c_int = 1;

// Stub macros from q_shared.h - implemented mechanically
macro_rules! VectorCopy {
    ($src:expr, $dst:expr) => {
        {
            $dst[0] = $src[0];
            $dst[1] = $src[1];
            $dst[2] = $src[2];
        }
    };
}

macro_rules! VectorMA {
    ($src:expr, $scale:expr, $add:expr, $dst:expr) => {
        {
            $dst[0] = $src[0] + $scale * $add[0];
            $dst[1] = $src[1] + $scale * $add[1];
            $dst[2] = $src[2] + $scale * $add[2];
        }
    };
}

macro_rules! VectorScale {
    ($src:expr, $scale:expr, $dst:expr) => {
        {
            $dst[0] = $src[0] * $scale;
            $dst[1] = $src[1] * $scale;
            $dst[2] = $src[2] * $scale;
        }
    };
}

macro_rules! VectorNormalize {
    ($v:expr) => {
        {
            let len = ($v[0] * $v[0] + $v[1] * $v[1] + $v[2] * $v[2]).sqrt();
            if len != 0.0 {
                $v[0] /= len;
                $v[1] /= len;
                $v[2] /= len;
            }
        }
    };
}

macro_rules! VectorSubtract {
    ($a:expr, $b:expr, $c:expr) => {
        {
            $c[0] = $a[0] - $b[0];
            $c[1] = $a[1] - $b[1];
            $c[2] = $a[2] - $b[2];
        }
    };
}

macro_rules! VectorClear {
    ($v:expr) => {
        {
            $v[0] = 0.0;
            $v[1] = 0.0;
            $v[2] = 0.0;
        }
    };
}

macro_rules! VectorSet {
    ($v:expr, $x:expr, $y:expr, $z:expr) => {
        {
            $v[0] = $x;
            $v[1] = $y;
            $v[2] = $z;
        }
    };
}

macro_rules! DotProduct {
    ($a:expr, $b:expr) => {
        ($a[0] * $b[0] + $a[1] * $b[1] + $a[2] * $b[2])
    };
}

macro_rules! CrossProduct {
    ($a:expr, $b:expr, $c:expr) => {
        {
            $c[0] = $a[1] * $b[2] - $a[2] * $b[1];
            $c[1] = $a[2] * $b[0] - $a[0] * $b[2];
            $c[2] = $a[0] * $b[1] - $a[1] * $b[0];
        }
    };
}

// Constants from be_aas_def.h
const LINECOLOR_RED: c_int = 1;
const LINECOLOR_GREEN: c_int = 2;
const LINECOLOR_BLUE: c_int = 3;
const LINECOLOR_YELLOW: c_int = 4;
const LINECOLOR_NONE: c_int = 0;

const PRT_ERROR: c_int = 1;
const PRT_MESSAGE: c_int = 2;

// Constants from be_aas.h
const TRAVELTYPE_MASK: c_int = 0xFF;
const TRAVEL_INVALID: c_int = 0;
const TRAVEL_WALK: c_int = 1;
const TRAVEL_CROUCH: c_int = 2;
const TRAVEL_BARRIERJUMP: c_int = 3;
const TRAVEL_JUMP: c_int = 4;
const TRAVEL_LADDER: c_int = 5;
const TRAVEL_WALKOFFLEDGE: c_int = 6;
const TRAVEL_SWIM: c_int = 7;
const TRAVEL_WATERJUMP: c_int = 8;
const TRAVEL_TELEPORT: c_int = 9;
const TRAVEL_ELEVATOR: c_int = 10;
const TRAVEL_ROCKETJUMP: c_int = 11;
const TRAVEL_BFGJUMP: c_int = 12;
const TRAVEL_GRAPPLEHOOK: c_int = 13;
const TRAVEL_JUMPPAD: c_int = 14;
const TRAVEL_FUNCBOB: c_int = 15;

const FACE_GROUND: c_int = 1;
const FACE_LADDER: c_int = 2;

const AREACONTENTS_VIEWPORTAL: c_int = 0x80;
const PRESENCE_NORMAL: c_int = 1;

const SE_HITGROUND: c_int = 1;
const SE_ENTERWATER: c_int = 2;
const SE_ENTERSLIME: c_int = 4;
const SE_ENTERLAVA: c_int = 8;
const SE_HITGROUNDDAMAGE: c_int = 16;
const SE_TOUCHJUMPPAD: c_int = 32;
const SE_HITGROUNDAREA: c_int = 64;

// Stub types for AAS structures
#[repr(C)]
pub struct aas_edge_t {
    pub v: [c_int; 2],
}

#[repr(C)]
pub struct aas_plane_t {
    pub normal: vec3_t,
    pub dist: c_float,
}

#[repr(C)]
pub struct aas_face_t {
    pub planenum: c_int,
    pub numedges: c_int,
    pub firstedge: c_int,
    pub faceflags: c_int,
    pub frontarea: c_int,
    pub backarea: c_int,
}

#[repr(C)]
pub struct aas_area_t {
    pub numfaces: c_int,
    pub firstface: c_int,
}

#[repr(C)]
pub struct aas_areasettings_t {
    pub contents: c_int,
    pub numreachableareas: c_int,
    pub firstreachablearea: c_int,
}

#[repr(C)]
pub struct aas_reachability_t {
    pub areanum: c_int,
    pub edgenum: c_int,
    pub facenum: c_int,
    pub traveltype: c_int,
    pub start: vec3_t,
    pub end: vec3_t,
}

#[repr(C)]
pub struct aas_clientmove_t {
    pub presence: c_int,
}

// Stub for botimport and aasworld
#[repr(C)]
pub struct BotImport {
    pub DebugPolygonCreate: unsafe extern "C" fn(c_int, c_int, *mut vec3_t) -> c_int,
    pub DebugPolygonDelete: unsafe extern "C" fn(c_int) -> (),
    pub DebugLineCreate: unsafe extern "C" fn() -> c_int,
    pub DebugLineDelete: unsafe extern "C" fn(c_int) -> (),
    pub DebugLineShow: unsafe extern "C" fn(c_int, vec3_t, vec3_t, c_int) -> (),
    pub Print: unsafe extern "C" fn(c_int, *const u8, ...) -> (),
}

#[repr(C)]
pub struct aasworld_t {
    pub numfaces: c_int,
    pub faces: *mut aas_face_t,
    pub numedges: c_int,
    pub edges: *mut aas_edge_t,
    pub edgeindex: *mut c_int,
    pub vertexes: *mut vec3_t,
    pub planes: *mut aas_plane_t,
    pub numareas: c_int,
    pub areas: *mut aas_area_t,
    pub faceindex: *mut c_int,
    pub areasettings: *mut aas_areasettings_t,
    pub reachability: *mut aas_reachability_t,
}

#[repr(C)]
pub struct aassettings_t {
    pub phys_jumpvel: c_float,
}

// Extern stubs
extern "C" {
    pub static mut botimport: BotImport;
    pub static mut aasworld: aasworld_t;
    pub static mut aassettings: aassettings_t;
    pub fn AAS_HorizontalVelocityForJump(vel: c_float, start: vec3_t, end: vec3_t, speed: *mut c_float);
    pub fn AAS_RocketJumpZVelocity(start: vec3_t) -> c_float;
    pub fn AAS_PredictClientMovement(move_: *mut aas_clientmove_t, entnum: c_int, start: vec3_t,
                                     presencetype: c_int, onground: c_int, velocity: vec3_t,
                                     cmdmove: vec3_t, numframes: c_int, frametime: c_int,
                                     framedelta: c_float, stopevent: c_int, stoparea: c_int, visualize: c_int);
    pub fn AAS_JumpReachRunStart(reach: *mut aas_reachability_t, dir: vec3_t);
    pub fn AAS_PointAreaNum(p: vec3_t) -> c_int;
    pub fn AAS_AreaCluster(areanum: c_int) -> c_int;
    pub fn AAS_Time() -> c_float;
}

const MAX_DEBUGLINES: usize = 1024;
const MAX_DEBUGPOLYGONS: usize = 8192;

pub static mut debuglines: [c_int; MAX_DEBUGLINES] = [0; MAX_DEBUGLINES];
pub static mut debuglinevisible: [c_int; MAX_DEBUGLINES] = [0; MAX_DEBUGLINES];
pub static mut numdebuglines: c_int = 0;

static mut debugpolygons: [c_int; MAX_DEBUGPOLYGONS] = [0; MAX_DEBUGPOLYGONS];

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ClearShownPolygons() {
	unsafe {
		//*
		for i in 0..MAX_DEBUGPOLYGONS {
			if debugpolygons[i] != 0 {
				botimport.DebugPolygonDelete(debugpolygons[i]);
			}
			debugpolygons[i] = 0;
		} //end for
		//*/
		/*
			for i in 0..MAX_DEBUGPOLYGONS {
				botimport.DebugPolygonDelete(i as c_int);
				debugpolygons[i] = 0;
			} //end for
		*/
	}
} //end of the function AAS_ClearShownPolygons
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ShowPolygon(color: c_int, numpoints: c_int, points: *mut vec3_t) {
	unsafe {
		for i in 0..MAX_DEBUGPOLYGONS {
			if debugpolygons[i] == 0 {
				debugpolygons[i] = botimport.DebugPolygonCreate(color, numpoints, points);
				break;
			} //end if
		} //end for
	}
} //end of the function AAS_ShowPolygon
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ClearShownDebugLines() {
	unsafe {
		//make all lines invisible
		for i in 0..MAX_DEBUGLINES {
			if debuglines[i] != 0 {
				//botimport.DebugLineShow(debuglines[i], NULL, NULL, LINECOLOR_NONE);
				botimport.DebugLineDelete(debuglines[i]);
				debuglines[i] = 0;
				debuglinevisible[i] = qfalse;
			} //end if
		} //end for
	}
} //end of the function AAS_ClearShownDebugLines
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_DebugLine(start: vec3_t, end: vec3_t, color: c_int) {
	unsafe {
		for line in 0..MAX_DEBUGLINES {
			if debuglines[line] == 0 {
				debuglines[line] = botimport.DebugLineCreate();
				debuglinevisible[line] = qfalse;
				numdebuglines += 1;
			} //end if
			if debuglinevisible[line] == 0 {
				botimport.DebugLineShow(debuglines[line], start, end, color);
				debuglinevisible[line] = qtrue;
				return;
			} //end else
		} //end for
	}
} //end of the function AAS_DebugLine
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn AAS_PermanentLine(start: vec3_t, end: vec3_t, color: c_int) {
	unsafe {
		let line = botimport.DebugLineCreate();
		botimport.DebugLineShow(line, start, end, color);
	}
} //end of the function AAS_PermenentLine
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn AAS_DrawPermanentCross(origin: vec3_t, size: c_float, color: c_int) {
	unsafe {
		let mut start: vec3_t = [0.0; 3];
		let mut end: vec3_t = [0.0; 3];

		for i in 0..3 {
			VectorCopy!(origin, start);
			start[i] += size;
			VectorCopy!(origin, end);
			end[i] -= size;
			AAS_DebugLine(start, end, color);
			let debugline = botimport.DebugLineCreate();
			botimport.DebugLineShow(debugline, start, end, color);
		} //end for
	}
} //end of the function AAS_DrawPermanentCross
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_DrawPlaneCross(point: vec3_t, normal: vec3_t, dist: c_float, type_: c_int, color: c_int) {
	unsafe {
		let mut start1: vec3_t = [0.0; 3];
		let mut end1: vec3_t = [0.0; 3];
		let mut start2: vec3_t = [0.0; 3];
		let mut end2: vec3_t = [0.0; 3];
		let mut lines: [c_int; 2] = [0; 2];

		//make a cross in the hit plane at the hit point
		VectorCopy!(point, start1);
		VectorCopy!(point, end1);
		VectorCopy!(point, start2);
		VectorCopy!(point, end2);

		let n0 = type_ % 3;
		let n1 = (type_ + 1) % 3;
		let n2 = (type_ + 2) % 3;
		start1[n1 as usize] -= 6.0;
		start1[n2 as usize] -= 6.0;
		end1[n1 as usize] += 6.0;
		end1[n2 as usize] += 6.0;
		start2[n1 as usize] += 6.0;
		start2[n2 as usize] -= 6.0;
		end2[n1 as usize] -= 6.0;
		end2[n2 as usize] += 6.0;

		start1[n0 as usize] = (dist - (start1[n1 as usize] * normal[n1 as usize] +
					start1[n2 as usize] * normal[n2 as usize])) / normal[n0 as usize];
		end1[n0 as usize] = (dist - (end1[n1 as usize] * normal[n1 as usize] +
					end1[n2 as usize] * normal[n2 as usize])) / normal[n0 as usize];
		start2[n0 as usize] = (dist - (start2[n1 as usize] * normal[n1 as usize] +
					start2[n2 as usize] * normal[n2 as usize])) / normal[n0 as usize];
		end2[n0 as usize] = (dist - (end2[n1 as usize] * normal[n1 as usize] +
					end2[n2 as usize] * normal[n2 as usize])) / normal[n0 as usize];

		let mut j = 0;
		let mut line = 0;
		while j < 2 && line < MAX_DEBUGLINES as c_int {
			if debuglines[line as usize] == 0 {
				debuglines[line as usize] = botimport.DebugLineCreate();
				lines[j as usize] = debuglines[line as usize];
				debuglinevisible[line as usize] = qtrue;
				numdebuglines += 1;
			} //end if
			else if debuglinevisible[line as usize] == 0 {
				lines[j as usize] = debuglines[line as usize];
				debuglinevisible[line as usize] = qtrue;
			} //end else
			line += 1;
		} //end for
		botimport.DebugLineShow(lines[0], start1, end1, color);
		botimport.DebugLineShow(lines[1], start2, end2, color);
	}
} //end of the function AAS_DrawPlaneCross
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ShowBoundingBox(origin: vec3_t, mins: vec3_t, maxs: vec3_t) {
	unsafe {
		let mut bboxcorners: [vec3_t; 8] = [[0.0; 3]; 8];
		let mut lines: [c_int; 3] = [0; 3];

		//upper corners
		bboxcorners[0][0] = origin[0] + maxs[0];
		bboxcorners[0][1] = origin[1] + maxs[1];
		bboxcorners[0][2] = origin[2] + maxs[2];
		//
		bboxcorners[1][0] = origin[0] + mins[0];
		bboxcorners[1][1] = origin[1] + maxs[1];
		bboxcorners[1][2] = origin[2] + maxs[2];
		//
		bboxcorners[2][0] = origin[0] + mins[0];
		bboxcorners[2][1] = origin[1] + mins[1];
		bboxcorners[2][2] = origin[2] + maxs[2];
		//
		bboxcorners[3][0] = origin[0] + maxs[0];
		bboxcorners[3][1] = origin[1] + mins[1];
		bboxcorners[3][2] = origin[2] + maxs[2];
		//lower corners
		// Com_Memcpy(bboxcorners[4], bboxcorners[0], sizeof(vec3_t) * 4);
		core::ptr::copy_nonoverlapping(
			&bboxcorners[0] as *const vec3_t,
			&mut bboxcorners[4] as *mut vec3_t,
			4,
		);
		for i in 0..4 {
			bboxcorners[4 + i][2] = origin[2] + mins[2];
		}
		//draw bounding box
		for i in 0..4 {
			let mut j = 0;
			let mut line = 0;
			while j < 3 && line < MAX_DEBUGLINES as c_int {
				if debuglines[line as usize] == 0 {
					debuglines[line as usize] = botimport.DebugLineCreate();
					lines[j as usize] = debuglines[line as usize];
					debuglinevisible[line as usize] = qtrue;
					numdebuglines += 1;
				} //end if
				else if debuglinevisible[line as usize] == 0 {
					lines[j as usize] = debuglines[line as usize];
					debuglinevisible[line as usize] = qtrue;
				} //end else
				line += 1;
			} //end for
			//top plane
			botimport.DebugLineShow(lines[0], bboxcorners[i],
										bboxcorners[((i+1)&3)], LINECOLOR_RED);
			//bottom plane
			botimport.DebugLineShow(lines[1], bboxcorners[4+i],
										bboxcorners[(4+((i+1)&3))], LINECOLOR_RED);
			//vertical lines
			botimport.DebugLineShow(lines[2], bboxcorners[i],
										bboxcorners[4+i], LINECOLOR_RED);
		} //end for
	}
} //end of the function AAS_ShowBoundingBox
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ShowFace(facenum: c_int) {
	unsafe {
		let mut color = LINECOLOR_YELLOW;
		let mut start: vec3_t = [0.0; 3];
		let mut end: vec3_t = [0.0; 3];

		//check if face number is in range
		if facenum >= aasworld.numfaces {
			botimport.Print(PRT_ERROR, b"facenum %d out of range\n" as *const _ as *const u8, facenum);
		} //end if
		let face = &aasworld.faces[facenum as usize];
		//walk through the edges of the face
		for i in 0..face.numedges {
			//edge number
			let edgenum = (*aasworld.edgeindex.add((face.firstedge + i) as usize)).abs();
			//check if edge number is in range
			if edgenum >= aasworld.numedges {
				botimport.Print(PRT_ERROR, b"edgenum %d out of range\n" as *const _ as *const u8, edgenum);
			} //end if
			let edge = &aasworld.edges[edgenum as usize];
			if color == LINECOLOR_RED { color = LINECOLOR_GREEN; }
			else if color == LINECOLOR_GREEN { color = LINECOLOR_BLUE; }
			else if color == LINECOLOR_BLUE { color = LINECOLOR_YELLOW; }
			else { color = LINECOLOR_RED; }
			AAS_DebugLine(*aasworld.vertexes.add(edge.v[0] as usize),
										*aasworld.vertexes.add(edge.v[1] as usize),
										color);
		} //end for
		let plane = &aasworld.planes[face.planenum as usize];
		let edgenum = (*aasworld.edgeindex.add(face.firstedge as usize)).abs();
		let edge = &aasworld.edges[edgenum as usize];
		VectorCopy!(*aasworld.vertexes.add(edge.v[0] as usize), start);
		VectorMA!(start, 20.0, plane.normal, end);
		AAS_DebugLine(start, end, LINECOLOR_RED);
	}
} //end of the function AAS_ShowFace
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ShowFacePolygon(facenum: c_int, color: c_int, flip: c_int) {
	unsafe {
		let mut points: [vec3_t; 128] = [[0.0; 3]; 128];
		let mut numpoints = 0;

		//check if face number is in range
		if facenum >= aasworld.numfaces {
			botimport.Print(PRT_ERROR, b"facenum %d out of range\n" as *const _ as *const u8, facenum);
		} //end if
		let face = &aasworld.faces[facenum as usize];
		//walk through the edges of the face
		if flip != 0 {
			let mut i = face.numedges - 1;
			while i >= 0 {
				//edge number
				let edgenum = *aasworld.edgeindex.add((face.firstedge + i) as usize);
				let edge = &aasworld.edges[(edgenum.abs()) as usize];
				VectorCopy!(*aasworld.vertexes.add(edge.v[(if edgenum < 0 { 1 } else { 0 }) as usize] as usize), points[numpoints as usize]);
				numpoints += 1;
				i -= 1;
			} //end for
		} //end if
		else {
			for i in 0..face.numedges {
				//edge number
				let edgenum = *aasworld.edgeindex.add((face.firstedge + i) as usize);
				let edge = &aasworld.edges[(edgenum.abs()) as usize];
				VectorCopy!(*aasworld.vertexes.add(edge.v[(if edgenum < 0 { 1 } else { 0 }) as usize] as usize), points[numpoints as usize]);
				numpoints += 1;
			} //end for
		} //end else
		AAS_ShowPolygon(color, numpoints, &mut points[0] as *mut vec3_t);
	}
} //end of the function AAS_ShowFacePolygon
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ShowArea(areanum: c_int, groundfacesonly: c_int) {
	unsafe {
		let mut areaedges: [c_int; MAX_DEBUGLINES] = [0; MAX_DEBUGLINES];
		let mut numareaedges = 0;
		let mut color = 0;

		//
		if areanum < 0 || areanum >= aasworld.numareas {
			botimport.Print(PRT_ERROR, b"area %d out of range [0, %d]\n" as *const _ as *const u8,
								areanum, aasworld.numareas);
			return;
		} //end if
		//pointer to the convex area
		let area = &aasworld.areas[areanum as usize];
		//walk through the faces of the area
		for i in 0..area.numfaces {
			let facenum = (*aasworld.faceindex.add((area.firstface + i) as usize)).abs();
			//check if face number is in range
			if facenum >= aasworld.numfaces {
				botimport.Print(PRT_ERROR, b"facenum %d out of range\n" as *const _ as *const u8, facenum);
			} //end if
			let face = &aasworld.faces[facenum as usize];
			//ground faces only
			if groundfacesonly != 0 {
				if (face.faceflags & (FACE_GROUND | FACE_LADDER)) == 0 {
					continue;
				}
			} //end if
			//walk through the edges of the face
			for j in 0..face.numedges {
				//edge number
				let edgenum = (*aasworld.edgeindex.add((face.firstedge + j) as usize)).abs();
				//check if edge number is in range
				if edgenum >= aasworld.numedges {
					botimport.Print(PRT_ERROR, b"edgenum %d out of range\n" as *const _ as *const u8, edgenum);
				} //end if
				//check if the edge is stored already
				let mut n = 0;
				while n < numareaedges {
					if areaedges[n as usize] == edgenum {
						break;
					}
					n += 1;
				} //end for
				if n == numareaedges && numareaedges < MAX_DEBUGLINES as c_int {
					areaedges[numareaedges as usize] = edgenum;
					numareaedges += 1;
				} //end if
			} //end for
			//AAS_ShowFace(facenum);
		} //end for
		//draw all the edges
		for n in 0..numareaedges {
			let mut line = 0;
			while line < MAX_DEBUGLINES as c_int {
				if debuglines[line as usize] == 0 {
					debuglines[line as usize] = botimport.DebugLineCreate();
					debuglinevisible[line as usize] = qfalse;
					numdebuglines += 1;
				} //end if
				if debuglinevisible[line as usize] == 0 {
					break;
				} //end else
				line += 1;
			} //end for
			if line >= MAX_DEBUGLINES as c_int {
				return;
			}
			let edge = &aasworld.edges[areaedges[n as usize] as usize];
			if color == LINECOLOR_RED { color = LINECOLOR_BLUE; }
			else if color == LINECOLOR_BLUE { color = LINECOLOR_GREEN; }
			else if color == LINECOLOR_GREEN { color = LINECOLOR_YELLOW; }
			else { color = LINECOLOR_RED; }
			botimport.DebugLineShow(debuglines[line as usize],
									*aasworld.vertexes.add(edge.v[0] as usize),
									*aasworld.vertexes.add(edge.v[1] as usize),
									color);
			debuglinevisible[line as usize] = qtrue;
		} //end for*/
	}
} //end of the function AAS_ShowArea
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ShowAreaPolygons(areanum: c_int, color: c_int, groundfacesonly: c_int) {
	unsafe {
		//
		if areanum < 0 || areanum >= aasworld.numareas {
			botimport.Print(PRT_ERROR, b"area %d out of range [0, %d]\n" as *const _ as *const u8,
								areanum, aasworld.numareas);
			return;
		} //end if
		//pointer to the convex area
		let area = &aasworld.areas[areanum as usize];
		//walk through the faces of the area
		for i in 0..area.numfaces {
			let facenum = (*aasworld.faceindex.add((area.firstface + i) as usize)).abs();
			//check if face number is in range
			if facenum >= aasworld.numfaces {
				botimport.Print(PRT_ERROR, b"facenum %d out of range\n" as *const _ as *const u8, facenum);
			} //end if
			let face = &aasworld.faces[facenum as usize];
			//ground faces only
			if groundfacesonly != 0 {
				if (face.faceflags & (FACE_GROUND | FACE_LADDER)) == 0 {
					continue;
				}
			} //end if
			AAS_ShowFacePolygon(facenum, color, if face.frontarea != areanum { 1 } else { 0 });
		} //end for
	}
} //end of the function AAS_ShowAreaPolygons
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_DrawCross(origin: vec3_t, size: c_float, color: c_int) {
	unsafe {
		let mut start: vec3_t = [0.0; 3];
		let mut end: vec3_t = [0.0; 3];

		for i in 0..3 {
			VectorCopy!(origin, start);
			start[i] += size;
			VectorCopy!(origin, end);
			end[i] -= size;
			AAS_DebugLine(start, end, color);
		} //end for
	}
} //end of the function AAS_DrawCross
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
#[cfg(debug_assertions)]
pub fn AAS_PrintTravelType(traveltype: c_int) {
	unsafe {
		let str_: *const u8 = match traveltype & TRAVELTYPE_MASK {
			TRAVEL_INVALID => b"TRAVEL_INVALID\0" as *const u8,
			TRAVEL_WALK => b"TRAVEL_WALK\0" as *const u8,
			TRAVEL_CROUCH => b"TRAVEL_CROUCH\0" as *const u8,
			TRAVEL_BARRIERJUMP => b"TRAVEL_BARRIERJUMP\0" as *const u8,
			TRAVEL_JUMP => b"TRAVEL_JUMP\0" as *const u8,
			TRAVEL_LADDER => b"TRAVEL_LADDER\0" as *const u8,
			TRAVEL_WALKOFFLEDGE => b"TRAVEL_WALKOFFLEDGE\0" as *const u8,
			TRAVEL_SWIM => b"TRAVEL_SWIM\0" as *const u8,
			TRAVEL_WATERJUMP => b"TRAVEL_WATERJUMP\0" as *const u8,
			TRAVEL_TELEPORT => b"TRAVEL_TELEPORT\0" as *const u8,
			TRAVEL_ELEVATOR => b"TRAVEL_ELEVATOR\0" as *const u8,
			TRAVEL_ROCKETJUMP => b"TRAVEL_ROCKETJUMP\0" as *const u8,
			TRAVEL_BFGJUMP => b"TRAVEL_BFGJUMP\0" as *const u8,
			TRAVEL_GRAPPLEHOOK => b"TRAVEL_GRAPPLEHOOK\0" as *const u8,
			TRAVEL_JUMPPAD => b"TRAVEL_JUMPPAD\0" as *const u8,
			TRAVEL_FUNCBOB => b"TRAVEL_FUNCBOB\0" as *const u8,
			_ => b"UNKNOWN TRAVEL TYPE\0" as *const u8,
		}; //end match
		botimport.Print(PRT_MESSAGE, b"%s" as *const u8, str_);
	}
}
#[cfg(not(debug_assertions))]
pub fn AAS_PrintTravelType(_traveltype: c_int) {
}
//end of the function AAS_PrintTravelType
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_DrawArrow(start: vec3_t, end: vec3_t, linecolor: c_int, arrowcolor: c_int) {
	unsafe {
		let mut dir: vec3_t = [0.0; 3];
		let mut cross: vec3_t = [0.0; 3];
		let mut p1: vec3_t = [0.0; 3];
		let mut p2: vec3_t = [0.0; 3];
		let up: vec3_t = [0.0, 0.0, 1.0];

		VectorSubtract!(end, start, dir);
		VectorNormalize!(dir);
		let dot = DotProduct!(dir, up);
		if dot > 0.99 || dot < -0.99 {
			VectorSet!(cross, 1.0, 0.0, 0.0);
		} else {
			CrossProduct!(dir, up, cross);
		}

		VectorMA!(end, -6.0, dir, p1);
		VectorCopy!(p1, p2);
		VectorMA!(p1, 6.0, cross, p1);
		VectorMA!(p2, -6.0, cross, p2);

		AAS_DebugLine(start, end, linecolor);
		AAS_DebugLine(p1, end, arrowcolor);
		AAS_DebugLine(p2, end, arrowcolor);
	}
} //end of the function AAS_DrawArrow
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ShowReachability(reach: *mut aas_reachability_t) {
	unsafe {
		let mut dir: vec3_t = [0.0; 3];
		let mut cmdmove: vec3_t = [0.0; 3];
		let mut velocity: vec3_t = [0.0; 3];
		let mut speed = 0.0;
		let mut zvel: c_float;
		let mut move_: aas_clientmove_t = core::mem::zeroed();

		AAS_ShowAreaPolygons((*reach).areanum, 5, qtrue);
		//AAS_ShowArea(reach->areanum, qtrue);
		AAS_DrawArrow((*reach).start, (*reach).end, LINECOLOR_BLUE, LINECOLOR_YELLOW);
		//
		if ((*reach).traveltype & TRAVELTYPE_MASK) == TRAVEL_JUMP ||
			((*reach).traveltype & TRAVELTYPE_MASK) == TRAVEL_WALKOFFLEDGE
		{
			AAS_HorizontalVelocityForJump(aassettings.phys_jumpvel, (*reach).start, (*reach).end, &mut speed);
			//
			VectorSubtract!((*reach).end, (*reach).start, dir);
			dir[2] = 0.0;
			VectorNormalize!(dir);
			//set the velocity
			VectorScale!(dir, speed, velocity);
			//set the command movement
			VectorClear!(cmdmove);
			cmdmove[2] = aassettings.phys_jumpvel;
			//
			AAS_PredictClientMovement(&mut move_, -1, (*reach).start, PRESENCE_NORMAL, qtrue,
									velocity, cmdmove, 3, 30, 0.1,
									SE_HITGROUND|SE_ENTERWATER|SE_ENTERSLIME|
									SE_ENTERLAVA|SE_HITGROUNDDAMAGE, 0, qtrue);
			//
			if ((*reach).traveltype & TRAVELTYPE_MASK) == TRAVEL_JUMP {
				AAS_JumpReachRunStart(reach, dir);
				AAS_DrawCross(dir, 4.0, LINECOLOR_BLUE);
			} //end if
		} //end if
		else if ((*reach).traveltype & TRAVELTYPE_MASK) == TRAVEL_ROCKETJUMP {
			zvel = AAS_RocketJumpZVelocity((*reach).start);
			AAS_HorizontalVelocityForJump(zvel, (*reach).start, (*reach).end, &mut speed);
			//
			VectorSubtract!((*reach).end, (*reach).start, dir);
			dir[2] = 0.0;
			VectorNormalize!(dir);
			//get command movement
			VectorScale!(dir, speed, cmdmove);
			VectorSet!(velocity, 0.0, 0.0, zvel);
			//
			AAS_PredictClientMovement(&mut move_, -1, (*reach).start, PRESENCE_NORMAL, qtrue,
									velocity, cmdmove, 30, 30, 0.1,
									SE_ENTERWATER|SE_ENTERSLIME|
									SE_ENTERLAVA|SE_HITGROUNDDAMAGE|
									SE_TOUCHJUMPPAD|SE_HITGROUNDAREA, (*reach).areanum, qtrue);
		} //end else if
		else if ((*reach).traveltype & TRAVELTYPE_MASK) == TRAVEL_JUMPPAD {
			VectorSet!(cmdmove, 0.0, 0.0, 0.0);
			//
			VectorSubtract!((*reach).end, (*reach).start, dir);
			dir[2] = 0.0;
			VectorNormalize!(dir);
			//set the velocity
			//NOTE: the edgenum is the horizontal velocity
			VectorScale!(dir, (*reach).edgenum as c_float, velocity);
			//NOTE: the facenum is the Z velocity
			velocity[2] = (*reach).facenum as c_float;
			//
			AAS_PredictClientMovement(&mut move_, -1, (*reach).start, PRESENCE_NORMAL, qtrue,
									velocity, cmdmove, 30, 30, 0.1,
									SE_ENTERWATER|SE_ENTERSLIME|
									SE_ENTERLAVA|SE_HITGROUNDDAMAGE|
									SE_TOUCHJUMPPAD|SE_HITGROUNDAREA, (*reach).areanum, qtrue);
		} //end else if
	}
} //end of the function AAS_ShowReachability
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_ShowReachableAreas(areanum: c_int) {
	unsafe {
		static mut reach: aas_reachability_t = unsafe { core::mem::zeroed() };
		static mut index: c_int = 0;
		static mut lastareanum: c_int = 0;
		static mut lasttime: c_float = 0.0;

		if areanum != lastareanum {
			index = 0;
			lastareanum = areanum;
		} //end if
		let settings = &aasworld.areasettings[areanum as usize];
		//
		if settings.numreachableareas == 0 {
			return;
		}
		//
		if index >= settings.numreachableareas {
			index = 0;
		}
		//
		if AAS_Time() - lasttime > 1.5 {
			core::ptr::copy_nonoverlapping(
				&*aasworld.reachability.add((settings.firstreachablearea + index) as usize) as *const aas_reachability_t,
				&mut reach as *mut aas_reachability_t,
				1,
			);
			index += 1;
			lasttime = AAS_Time();
			AAS_PrintTravelType(reach.traveltype & TRAVELTYPE_MASK);
			botimport.Print(PRT_MESSAGE, b"\n" as *const u8);
		} //end if
		AAS_ShowReachability(&mut reach);
	}
} //end of the function ShowReachableAreas

pub fn AAS_FloodAreas_r(areanum: c_int, cluster: c_int, done: *mut c_int) {
	unsafe {
		AAS_ShowAreaPolygons(areanum, 1, qtrue);
		//pointer to the convex area
		let area = &aasworld.areas[areanum as usize];
		let settings = &aasworld.areasettings[areanum as usize];
		//walk through the faces of the area
		for i in 0..area.numfaces {
			let facenum = (*aasworld.faceindex.add((area.firstface + i) as usize)).abs();
			let face = &aasworld.faces[facenum as usize];
			let nextareanum = if face.frontarea == areanum {
				face.backarea
			} else {
				face.frontarea
			};
			if nextareanum == 0 {
				continue;
			}
			if *done.add(nextareanum as usize) != 0 {
				continue;
			}
			*done.add(nextareanum as usize) = qtrue;
			if (aasworld.areasettings[nextareanum as usize].contents & AREACONTENTS_VIEWPORTAL) != 0 {
				continue;
			}
			if AAS_AreaCluster(nextareanum) != cluster {
				continue;
			}
			AAS_FloodAreas_r(nextareanum, cluster, done);
		} //end for
		//
		for i in 0..settings.numreachableareas {
			let reach = &*aasworld.reachability.add((settings.firstreachablearea + i) as usize);
			let nextareanum = reach.areanum;
			if nextareanum == 0 {
				continue;
			}
			if *done.add(nextareanum as usize) != 0 {
				continue;
			}
			*done.add(nextareanum as usize) = qtrue;
			if (aasworld.areasettings[nextareanum as usize].contents & AREACONTENTS_VIEWPORTAL) != 0 {
				continue;
			}
			if AAS_AreaCluster(nextareanum) != cluster {
				continue;
			}
			/*
			if ((reach->traveltype & TRAVELTYPE_MASK) == TRAVEL_WALKOFFLEDGE)
			{
				AAS_DebugLine(reach->start, reach->end, 1);
			}
			*/
			AAS_FloodAreas_r(nextareanum, cluster, done);
		}
	}
}

pub fn AAS_FloodAreas(origin: vec3_t) {
	unsafe {
		// GetClearedMemory() needs to be defined; using allocation as a substitute
		let done = libc::malloc((aasworld.numareas as usize) * core::mem::size_of::<c_int>()) as *mut c_int;
		libc::memset(done as *mut libc::c_void, 0, (aasworld.numareas as usize) * core::mem::size_of::<c_int>());

		let areanum = AAS_PointAreaNum(origin);
		let cluster = AAS_AreaCluster(areanum);
		AAS_FloodAreas_r(areanum, cluster, done);
	}
}
