
/*****************************************************************************
 * name:		be_aas_optimize.c
 *
 * desc:		decreases the .aas file size after the reachabilities have
 *				been calculated, just dumps all the faces, edges and vertexes
 *
 * $Archive: /MissionPack/code/botlib/be_aas_optimize.c $
 * $Author: Zaphod $
 * $Revision: 5 $
 * $Modtime: 11/22/00 8:50a $
 * $Date: 11/22/00 8:55a $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{self, addr_of_mut};
use core::mem;

// PORTING: Import external types and functions
use core::ffi::c_ulong;

// Stub type definitions for AAS structures
// These mirror the C definitions from aasfile.h
pub type vec3_t = [f32; 3];
pub type aas_vertex_t = vec3_t;
pub type aas_edgeindex_t = c_int;
pub type aas_faceindex_t = c_int;

#[repr(C)]
pub struct aas_edge_t {
    pub v: [c_int; 2],  //numbers of the vertexes of this edge
}

#[repr(C)]
pub struct aas_face_t {
    pub planenum: c_int,        //number of the plane this face is in
    pub faceflags: c_int,       //face flags (no use to create face settings for just this field)
    pub numedges: c_int,        //number of edges in the boundary of the face
    pub firstedge: c_int,       //first edge in the edge index
    pub frontarea: c_int,       //area at the front of this face
    pub backarea: c_int,        //area at the back of this face
}

#[repr(C)]
pub struct aas_area_t {
    pub areanum: c_int,         //number of this area
    //3d definition
    pub numfaces: c_int,        //number of faces used for the boundary of the area
    pub firstface: c_int,       //first face in the face index used for the boundary of the area
    pub mins: vec3_t,           //mins of the area
    pub maxs: vec3_t,           //maxs of the area
    pub center: vec3_t,         //'center' of the area
}

#[repr(C)]
pub struct optimized_s
{
    //vertexes
    pub numvertexes: c_int,
    pub vertexes: *mut aas_vertex_t,
    //edges
    pub numedges: c_int,
    pub edges: *mut aas_edge_t,
    //edge index
    pub edgeindexsize: c_int,
    pub edgeindex: *mut aas_edgeindex_t,
    //faces
    pub numfaces: c_int,
    pub faces: *mut aas_face_t,
    //face index
    pub faceindexsize: c_int,
    pub faceindex: *mut aas_faceindex_t,
    //convex areas
    pub numareas: c_int,
    pub areas: *mut aas_area_t,
    //
    pub vertexoptimizeindex: *mut c_int,
    pub edgeoptimizeindex: *mut c_int,
    pub faceoptimizeindex: *mut c_int,
}

pub type optimized_t = optimized_s;

// External stubs for botlib_import_t and aasworld
#[repr(C)]
pub struct botlib_import_t {
    pub Print: unsafe extern "C" fn(c_int, *const c_char, ...),
}

#[repr(C)]
pub struct aas_reachability_t {
    pub areanum: c_int,         //number of the reachable area
    pub facenum: c_int,         //number of the face towards the other area
    pub edgenum: c_int,         //number of the edge towards the other area
    pub start: vec3_t,          //start point of inter area movement
    pub end: vec3_t,            //end point of inter area movement
    pub traveltype: c_int,      //type of travel required to get to the area
    pub traveltime: u16,        //travel time of the inter area movement
}

#[repr(C)]
pub struct aas_t {
    pub loaded: c_int,
    pub initialized: c_int,
    pub savefile: c_int,
    pub bspchecksum: c_int,
    pub time: f32,
    pub numframes: c_int,
    pub filename: [c_char; 64],
    pub mapname: [c_char; 64],
    pub numbboxes: c_int,
    pub bboxes: *mut c_void,
    pub numvertexes: c_int,
    pub vertexes: *mut aas_vertex_t,
    pub numplanes: c_int,
    pub planes: *mut c_void,
    pub numedges: c_int,
    pub edges: *mut aas_edge_t,
    pub edgeindexsize: c_int,
    pub edgeindex: *mut aas_edgeindex_t,
    pub numfaces: c_int,
    pub faces: *mut aas_face_t,
    pub faceindexsize: c_int,
    pub faceindex: *mut aas_faceindex_t,
    pub numareas: c_int,
    pub areas: *mut aas_area_t,
    pub numareasettings: c_int,
    pub areasettings: *mut c_void,
    pub reachabilitysize: c_int,
    pub reachability: *mut aas_reachability_t,
    pub numnodes: c_int,
    pub nodes: *mut c_void,
    pub numportals: c_int,
    pub portals: *mut c_void,
    pub portalindexsize: c_int,
    pub portalindex: *mut c_void,
    pub numclusters: c_int,
    pub clusters: *mut c_void,
    pub numreachabilityareas: c_int,
    pub reachabilitytime: f32,
    pub linkheap: *mut c_void,
    pub linkheapsize: c_int,
    pub freelinks: *mut c_void,
    pub arealinkedentities: *mut *mut c_void,
    pub maxentities: c_int,
    pub maxclients: c_int,
    pub entities: *mut c_void,
    pub configstrings: [*mut c_char; 512],
    pub indexessetup: c_int,
    pub travelflagfortype: [c_int; 32],
    pub areacontentstravelflags: *mut c_int,
    pub areaupdate: *mut c_void,
    pub portalupdate: *mut c_void,
    pub frameroutingupdates: c_int,
    pub reversedreachability: *mut c_void,
    pub areatraveltimes: *mut *mut *mut c_void,
    pub clusterareacache: *mut *mut *mut c_void,
    pub portalcache: *mut *mut c_void,
    pub oldestcache: *mut c_void,
    pub newestcache: *mut c_void,
    pub portalmaxtraveltimes: *mut c_int,
    pub reachabilityareaindex: *mut c_int,
    pub reachabilityareas: *mut c_void,
}

extern "C" {
    pub static mut aasworld: aas_t;
    pub static mut botimport: botlib_import_t;

    //PORTING: External C functions
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize);
    pub fn GetClearedMemory(size: c_ulong) -> *mut c_void;
}

const TRAVELTYPE_MASK: c_int = 0xFFFFFF;
const TRAVEL_ELEVATOR: c_int = 11;
const TRAVEL_JUMPPAD: c_int = 18;
const TRAVEL_FUNCBOB: c_int = 19;
const FACE_LADDER: c_int = 2;
const PRT_MESSAGE: c_int = 2;

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn AAS_KeepEdge(edge: *mut aas_edge_t) -> c_int
{
    return 1;
} //end of the function AAS_KeepFace
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn AAS_OptimizeEdge(optimized: *mut optimized_t, edgenum: c_int) -> c_int
{
    let mut i: c_int;
    let mut optedgenum: c_int;
    let mut edge: *mut aas_edge_t;
    let mut optedge: *mut aas_edge_t;

    unsafe {
        edge = &mut (*addr_of_mut!(aasworld)).edges[abs(edgenum) as usize];
        if AAS_KeepEdge(edge) == 0 { return 0; }

        optedgenum = (*optimized).edgeoptimizeindex[abs(edgenum) as usize];
        if optedgenum != 0
        {
            //keep the edge reversed sign
            if edgenum > 0 { return optedgenum; }
            else { return -optedgenum; }
        } //end if

        optedge = &mut (*optimized).edges[(*optimized).numedges as usize];

        i = 0;
        while i < 2
        {
            if (*optimized).vertexoptimizeindex[(*edge).v[i as usize] as usize] != 0
            {
                (*optedge).v[i as usize] = (*optimized).vertexoptimizeindex[(*edge).v[i as usize] as usize];
            } //end if
            else
            {
                VectorCopy(
                    &(*addr_of_mut!(aasworld)).vertexes[(*edge).v[i as usize] as usize][0] as *const f32,
                    &mut (*optimized).vertexes[(*optimized).numvertexes as usize][0] as *mut f32
                );
                (*optedge).v[i as usize] = (*optimized).numvertexes;
                (*optimized).vertexoptimizeindex[(*edge).v[i as usize] as usize] = (*optimized).numvertexes;
                (*optimized).numvertexes += 1;
            } //end else
            i += 1;
        } //end for
        (*optimized).edgeoptimizeindex[abs(edgenum) as usize] = (*optimized).numedges;
        optedgenum = (*optimized).numedges;
        (*optimized).numedges += 1;
        //keep the edge reversed sign
        if edgenum > 0 { return optedgenum; }
        else { return -optedgenum; }
    }
} //end of the function AAS_OptimizeEdge
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn AAS_KeepFace(face: *mut aas_face_t) -> c_int
{
    unsafe {
        if ((*face).faceflags & FACE_LADDER) == 0 { return 0; }
        else { return 1; }
    }
} //end of the function AAS_KeepFace
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn AAS_OptimizeFace(optimized: *mut optimized_t, facenum: c_int) -> c_int
{
    let mut i: c_int;
    let mut edgenum: c_int;
    let mut optedgenum: c_int;
    let mut optfacenum: c_int;
    let mut face: *mut aas_face_t;
    let mut optface: *mut aas_face_t;

    unsafe {
        face = &mut (*addr_of_mut!(aasworld)).faces[abs(facenum) as usize];
        if AAS_KeepFace(face) == 0 { return 0; }

        optfacenum = (*optimized).faceoptimizeindex[abs(facenum) as usize];
        if optfacenum != 0
        {
            //keep the face side sign
            if facenum > 0 { return optfacenum; }
            else { return -optfacenum; }
        } //end if

        optface = &mut (*optimized).faces[(*optimized).numfaces as usize];
        Com_Memcpy(
            optface as *mut c_void,
            face as *const c_void,
            mem::size_of::<aas_face_t>()
        );

        (*optface).numedges = 0;
        (*optface).firstedge = (*optimized).edgeindexsize;
        i = 0;
        while i < (*face).numedges
        {
            edgenum = (*addr_of_mut!(aasworld)).edgeindex[((*face).firstedge + i) as usize];
            optedgenum = AAS_OptimizeEdge(optimized, edgenum);
            if optedgenum != 0
            {
                (*optimized).edgeindex[((*optface).firstedge + (*optface).numedges) as usize] = optedgenum;
                (*optface).numedges += 1;
                (*optimized).edgeindexsize += 1;
            } //end if
            i += 1;
        } //end for
        (*optimized).faceoptimizeindex[abs(facenum) as usize] = (*optimized).numfaces;
        optfacenum = (*optimized).numfaces;
        (*optimized).numfaces += 1;
        //keep the face side sign
        if facenum > 0 { return optfacenum; }
        else { return -optfacenum; }
    }
} //end of the function AAS_OptimizeFace
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn AAS_OptimizeArea(optimized: *mut optimized_t, areanum: c_int)
{
    let mut i: c_int;
    let mut facenum: c_int;
    let mut optfacenum: c_int;
    let mut area: *mut aas_area_t;
    let mut optarea: *mut aas_area_t;

    unsafe {
        area = &mut (*addr_of_mut!(aasworld)).areas[areanum as usize];
        optarea = &mut (*optimized).areas[areanum as usize];
        Com_Memcpy(
            optarea as *mut c_void,
            area as *const c_void,
            mem::size_of::<aas_area_t>()
        );

        (*optarea).numfaces = 0;
        (*optarea).firstface = (*optimized).faceindexsize;
        i = 0;
        while i < (*area).numfaces
        {
            facenum = (*addr_of_mut!(aasworld)).faceindex[((*area).firstface + i) as usize];
            optfacenum = AAS_OptimizeFace(optimized, facenum);
            if optfacenum != 0
            {
                (*optimized).faceindex[((*optarea).firstface + (*optarea).numfaces) as usize] = optfacenum;
                (*optarea).numfaces += 1;
                (*optimized).faceindexsize += 1;
            } //end if
            i += 1;
        } //end for
    }
} //end of the function AAS_OptimizeArea
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn AAS_OptimizeAlloc(optimized: *mut optimized_t)
{
    unsafe {
        (*optimized).vertexes = GetClearedMemory((((*addr_of_mut!(aasworld)).numvertexes as usize) * mem::size_of::<aas_vertex_t>()) as c_ulong) as *mut aas_vertex_t;
        (*optimized).numvertexes = 0;
        (*optimized).edges = GetClearedMemory((((*addr_of_mut!(aasworld)).numedges as usize) * mem::size_of::<aas_edge_t>()) as c_ulong) as *mut aas_edge_t;
        (*optimized).numedges = 1; //edge zero is a dummy
        (*optimized).edgeindex = GetClearedMemory((((*addr_of_mut!(aasworld)).edgeindexsize as usize) * mem::size_of::<aas_edgeindex_t>()) as c_ulong) as *mut aas_edgeindex_t;
        (*optimized).edgeindexsize = 0;
        (*optimized).faces = GetClearedMemory((((*addr_of_mut!(aasworld)).numfaces as usize) * mem::size_of::<aas_face_t>()) as c_ulong) as *mut aas_face_t;
        (*optimized).numfaces = 1; //face zero is a dummy
        (*optimized).faceindex = GetClearedMemory((((*addr_of_mut!(aasworld)).faceindexsize as usize) * mem::size_of::<aas_faceindex_t>()) as c_ulong) as *mut aas_faceindex_t;
        (*optimized).faceindexsize = 0;
        (*optimized).areas = GetClearedMemory((((*addr_of_mut!(aasworld)).numareas as usize) * mem::size_of::<aas_area_t>()) as c_ulong) as *mut aas_area_t;
        (*optimized).numareas = (*addr_of_mut!(aasworld)).numareas;
        //
        (*optimized).vertexoptimizeindex = GetClearedMemory((((*addr_of_mut!(aasworld)).numvertexes as usize) * mem::size_of::<c_int>()) as c_ulong) as *mut c_int;
        (*optimized).edgeoptimizeindex = GetClearedMemory((((*addr_of_mut!(aasworld)).numedges as usize) * mem::size_of::<c_int>()) as c_ulong) as *mut c_int;
        (*optimized).faceoptimizeindex = GetClearedMemory((((*addr_of_mut!(aasworld)).numfaces as usize) * mem::size_of::<c_int>()) as c_ulong) as *mut c_int;
    }
} //end of the function AAS_OptimizeAlloc
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn AAS_OptimizeStore(optimized: *mut optimized_t)
{
    unsafe {
        //store the optimized vertexes
        if !(*addr_of_mut!(aasworld)).vertexes.is_null() { FreeMemory((*addr_of_mut!(aasworld)).vertexes as *mut c_void); }
        (*addr_of_mut!(aasworld)).vertexes = (*optimized).vertexes;
        (*addr_of_mut!(aasworld)).numvertexes = (*optimized).numvertexes;
        //store the optimized edges
        if !(*addr_of_mut!(aasworld)).edges.is_null() { FreeMemory((*addr_of_mut!(aasworld)).edges as *mut c_void); }
        (*addr_of_mut!(aasworld)).edges = (*optimized).edges;
        (*addr_of_mut!(aasworld)).numedges = (*optimized).numedges;
        //store the optimized edge index
        if !(*addr_of_mut!(aasworld)).edgeindex.is_null() { FreeMemory((*addr_of_mut!(aasworld)).edgeindex as *mut c_void); }
        (*addr_of_mut!(aasworld)).edgeindex = (*optimized).edgeindex;
        (*addr_of_mut!(aasworld)).edgeindexsize = (*optimized).edgeindexsize;
        //store the optimized faces
        if !(*addr_of_mut!(aasworld)).faces.is_null() { FreeMemory((*addr_of_mut!(aasworld)).faces as *mut c_void); }
        (*addr_of_mut!(aasworld)).faces = (*optimized).faces;
        (*addr_of_mut!(aasworld)).numfaces = (*optimized).numfaces;
        //store the optimized face index
        if !(*addr_of_mut!(aasworld)).faceindex.is_null() { FreeMemory((*addr_of_mut!(aasworld)).faceindex as *mut c_void); }
        (*addr_of_mut!(aasworld)).faceindex = (*optimized).faceindex;
        (*addr_of_mut!(aasworld)).faceindexsize = (*optimized).faceindexsize;
        //store the optimized areas
        if !(*addr_of_mut!(aasworld)).areas.is_null() { FreeMemory((*addr_of_mut!(aasworld)).areas as *mut c_void); }
        (*addr_of_mut!(aasworld)).areas = (*optimized).areas;
        (*addr_of_mut!(aasworld)).numareas = (*optimized).numareas;
        //free optimize indexes
        FreeMemory((*optimized).vertexoptimizeindex as *mut c_void);
        FreeMemory((*optimized).edgeoptimizeindex as *mut c_void);
        FreeMemory((*optimized).faceoptimizeindex as *mut c_void);
    }
} //end of the function AAS_OptimizeStore
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_Optimize()
{
    let mut i: c_int;
    let mut sign: c_int;
    let mut optimized: optimized_t = unsafe { mem::zeroed() };

    unsafe {

        AAS_OptimizeAlloc(&mut optimized);
        i = 1;
        while i < (*addr_of_mut!(aasworld)).numareas
        {
            AAS_OptimizeArea(&mut optimized, i);
            i += 1;
        } //end for
        //reset the reachability face pointers
        i = 0;
        while i < (*addr_of_mut!(aasworld)).reachabilitysize
        {
            //NOTE: for TRAVEL_ELEVATOR the facenum is the model number of
            //		the elevator
            if (((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).traveltype) & TRAVELTYPE_MASK) == TRAVEL_ELEVATOR {
                i += 1;
                continue;
            }
            //NOTE: for TRAVEL_JUMPPAD the facenum is the Z velocity and the edgenum is the hor velocity
            if (((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).traveltype) & TRAVELTYPE_MASK) == TRAVEL_JUMPPAD {
                i += 1;
                continue;
            }
            //NOTE: for TRAVEL_FUNCBOB the facenum and edgenum contain other coded information
            if (((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).traveltype) & TRAVELTYPE_MASK) == TRAVEL_FUNCBOB {
                i += 1;
                continue;
            }
            //
            sign = (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).facenum;
            (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).facenum = optimized.faceoptimizeindex[abs((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).facenum) as usize];
            if sign < 0 { (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).facenum = -(*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).facenum; }
            sign = (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).edgenum;
            (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).edgenum = optimized.edgeoptimizeindex[abs((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).edgenum) as usize];
            if sign < 0 { (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).edgenum = -(*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).edgenum; }
            i += 1;
        } //end for
        //store the optimized AAS data into aasworld
        AAS_OptimizeStore(&mut optimized);
        //print some nice stuff :)
        (botimport.Print)(PRT_MESSAGE, "AAS data optimized.\n\0".as_ptr() as *const c_char);
    }
} //end of the function AAS_Optimize
