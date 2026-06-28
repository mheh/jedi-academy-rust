#![allow(non_snake_case)]

/*****************************************************************************
 * name:		be_aas_file.c
 *
 * desc:		AAS file loading/writing
 *
 * $Archive: /MissionPack/code/botlib/be_aas_file.c $
 * $Author: Zaphod $
 * $Revision: 5 $
 * $Modtime: 5/16/01 2:36p $
 * $Date: 5/16/01 2:41p $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_char, c_void, c_ushort, c_ulong};
use core::ptr::{self, addr_of_mut};
use core::mem;

//#define AASFILEDEBUG

// Type definitions for AAS structures from aasfile.h
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

#[repr(C)]
pub struct aas_bbox_s {
    pub presencetype: c_int,
    pub flags: c_int,
    pub mins: vec3_t,
    pub maxs: vec3_t,
}
pub type aas_bbox_t = aas_bbox_s;

#[repr(C)]
pub struct aas_reachability_s {
    pub areanum: c_int,              //number of the reachable area
    pub facenum: c_int,              //number of the face towards the other area
    pub edgenum: c_int,              //number of the edge towards the other area
    pub start: vec3_t,               //start point of inter area movement
    pub end: vec3_t,                 //end point of inter area movement
    pub traveltype: c_int,           //type of travel required to get to the area
    pub traveltime: c_ushort,        //travel time of the inter area movement
}
pub type aas_reachability_t = aas_reachability_s;

#[repr(C)]
pub struct aas_areasettings_s {
    //could also add all kind of statistic fields
    pub contents: c_int,             //contents of the area
    pub areaflags: c_int,            //several area flags
    pub presencetype: c_int,         //how a bot can be present in this area
    pub cluster: c_int,              //cluster the area belongs to, if negative it's a portal
    pub clusterareanum: c_int,       //number of the area in the cluster
    pub numreachableareas: c_int,    //number of reachable areas from this one
    pub firstreachablearea: c_int,   //first reachable area in the reachable area index
}
pub type aas_areasettings_t = aas_areasettings_s;

#[repr(C)]
pub struct aas_portal_s {
    pub areanum: c_int,              //area that is the actual portal
    pub frontcluster: c_int,         //cluster at front of portal
    pub backcluster: c_int,          //cluster at back of portal
    pub clusterareanum: [c_int; 2],  //number of the area in the front and back cluster
}
pub type aas_portal_t = aas_portal_s;

pub type aas_portalindex_t = c_int;

#[repr(C)]
pub struct aas_cluster_s {
    pub numareas: c_int,             //number of areas in the cluster
    pub numreachabilityareas: c_int, //number of areas with reachabilities
    pub numportals: c_int,           //number of cluster portals
    pub firstportal: c_int,          //first cluster portal in the index
}
pub type aas_cluster_t = aas_cluster_s;

pub type aas_vertex_t = vec3_t;

#[repr(C)]
pub struct aas_plane_s {
    pub normal: vec3_t,              //normal vector of the plane
    pub dist: f32,                   //distance of the plane (normal vector * distance = point in plane)
    pub r#type: c_int,
}
pub type aas_plane_t = aas_plane_s;

#[repr(C)]
pub struct aas_edge_s {
    pub v: [c_int; 2],               //numbers of the vertexes of this edge
}
pub type aas_edge_t = aas_edge_s;

pub type aas_edgeindex_t = c_int;

#[repr(C)]
pub struct aas_face_s {
    pub planenum: c_int,             //number of the plane this face is in
    pub faceflags: c_int,            //face flags (no use to create face settings for just this field)
    pub numedges: c_int,             //number of edges in the boundary of the face
    pub firstedge: c_int,            //first edge in the edge index
    pub frontarea: c_int,            //area at the front of this face
    pub backarea: c_int,             //area at the back of this face
}
pub type aas_face_t = aas_face_s;

pub type aas_faceindex_t = c_int;

#[repr(C)]
pub struct aas_area_s {
    pub areanum: c_int,              //number of this area
    //3d definition
    pub numfaces: c_int,             //number of faces used for the boundary of the area
    pub firstface: c_int,            //first face in the face index used for the boundary of the area
    pub mins: vec3_t,                //mins of the area
    pub maxs: vec3_t,                //maxs of the area
    pub center: vec3_t,              //'center' of the area
}
pub type aas_area_t = aas_area_s;

#[repr(C)]
pub struct aas_node_s {
    pub planenum: c_int,
    pub children: [c_int; 2],        //child nodes of this node, or areas as leaves when negative
                                     //when a child is zero it's a solid leaf
}
pub type aas_node_t = aas_node_s;

//header lump
#[repr(C)]
pub struct aas_lump_s {
    pub fileofs: c_int,
    pub filelen: c_int,
}
pub type aas_lump_t = aas_lump_s;

//aas file header
#[repr(C)]
pub struct aas_header_s {
    pub ident: c_int,
    pub version: c_int,
    pub bspchecksum: c_int,
    //data entries
    pub lumps: [aas_lump_t; 14],  // AAS_LUMPS = 14
}
pub type aas_header_t = aas_header_s;

// Constants from aasfile.h
const AASID: c_int = ((('S' as c_int) << 24) + (('A' as c_int) << 16) + (('A' as c_int) << 8) + ('E' as c_int));
const AASVERSION_OLD: c_int = 4;
const AASVERSION: c_int = 5;

// Lump indices
const AASLUMP_BBOXES: usize = 0;
const AASLUMP_VERTEXES: usize = 1;
const AASLUMP_PLANES: usize = 2;
const AASLUMP_EDGES: usize = 3;
const AASLUMP_EDGEINDEX: usize = 4;
const AASLUMP_FACES: usize = 5;
const AASLUMP_FACEINDEX: usize = 6;
const AASLUMP_AREAS: usize = 7;
const AASLUMP_AREASETTINGS: usize = 8;
const AASLUMP_REACHABILITY: usize = 9;
const AASLUMP_NODES: usize = 10;
const AASLUMP_PORTALS: usize = 11;
const AASLUMP_PORTALINDEX: usize = 12;
const AASLUMP_CLUSTERS: usize = 13;

// Error codes from botlib.h (approximate)
const BLERR_NOERROR: c_int = 0;
const BLERR_CANNOTOPENAASFILE: c_int = 4;
const BLERR_WRONGAASFILEID: c_int = 5;
const BLERR_WRONGAASFILEVERSION: c_int = 6;
const BLERR_CANNOTREADAASLUMP: c_int = 7;

// Area flags from aasfile.h
const AREA_GROUNDED: c_int = 1;

// Print types from botlib.h
const PRT_MESSAGE: c_int = 0;
const PRT_WARNING: c_int = 1;
const PRT_ERROR: c_int = 2;

// File seek constants
const FS_SEEK_SET: c_int = 0;

// File open modes
const FS_READ: c_int = 0;
const FS_WRITE: c_int = 1;

// Type aliases for file handle and booleans
pub type fileHandle_t = c_int;

// Stub struct for botlib_import_t with Print and file functions
#[repr(C)]
pub struct botlib_import_t {
    pub Print: unsafe extern "C" fn(c_int, *const c_char, ...),
    pub FS_FOpenFile: unsafe extern "C" fn(*const c_char, *mut fileHandle_t, c_int) -> c_int,
    pub FS_Read: unsafe extern "C" fn(*mut c_void, c_int, fileHandle_t),
    pub FS_Write: unsafe extern "C" fn(*const c_void, c_int, fileHandle_t),
    pub FS_Seek: unsafe extern "C" fn(fileHandle_t, c_int, c_int) -> c_int,
    pub FS_FCloseFile: unsafe extern "C" fn(fileHandle_t),
}

// Stub aas_t structure
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
    pub bboxes: *mut aas_bbox_t,
    pub numvertexes: c_int,
    pub vertexes: *mut aas_vertex_t,
    pub numplanes: c_int,
    pub planes: *mut aas_plane_t,
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
    pub areasettings: *mut aas_areasettings_t,
    pub reachabilitysize: c_int,
    pub reachability: *mut aas_reachability_t,
    pub numnodes: c_int,
    pub nodes: *mut aas_node_t,
    pub numportals: c_int,
    pub portals: *mut aas_portal_t,
    pub portalindexsize: c_int,
    pub portalindex: *mut aas_portalindex_t,
    pub numclusters: c_int,
    pub clusters: *mut aas_cluster_t,
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
    pub fn LittleLong(l: c_int) -> c_int;
    pub fn LittleFloat(f: f32) -> f32;
    pub fn LittleShort(s: c_ushort) -> c_ushort;
    pub fn FreeMemory(ptr: *mut c_void);
    pub fn GetClearedHunkMemory(size: c_ulong) -> *mut c_void;
    pub fn AAS_Error(msg: *const c_char, ...);
    pub fn atoi(s: *const c_char) -> c_int;
    pub fn LibVarGetString(var_name: *const c_char) -> *mut c_char;
    pub fn Com_Memset(dest: *mut c_void, c: c_int, count: c_int);
}

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_SwapAASData()
{
    let mut i: c_int;
    let mut j: c_int;
    //bounding boxes
    i = 0;
    unsafe {
        while i < (*addr_of_mut!(aasworld)).numbboxes
        {
            (*(*addr_of_mut!(aasworld)).bboxes.add(i as usize)).presencetype = LittleLong((*(*addr_of_mut!(aasworld)).bboxes.add(i as usize)).presencetype);
            (*(*addr_of_mut!(aasworld)).bboxes.add(i as usize)).flags = LittleLong((*(*addr_of_mut!(aasworld)).bboxes.add(i as usize)).flags);
            j = 0;
            while j < 3
            {
                (*(*addr_of_mut!(aasworld)).bboxes.add(i as usize)).mins[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).bboxes.add(i as usize)).mins[j as usize]);
                (*(*addr_of_mut!(aasworld)).bboxes.add(i as usize)).maxs[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).bboxes.add(i as usize)).maxs[j as usize]);
                j += 1;
            } //end for
            i += 1;
        } //end for
        //vertexes
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numvertexes
        {
            j = 0;
            while j < 3
            {
                (*(*addr_of_mut!(aasworld)).vertexes.add(i as usize))[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).vertexes.add(i as usize))[j as usize]);
                j += 1;
            }
            i += 1;
        } //end for
        //planes
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numplanes
        {
            j = 0;
            while j < 3
            {
                (*(*addr_of_mut!(aasworld)).planes.add(i as usize)).normal[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).planes.add(i as usize)).normal[j as usize]);
                j += 1;
            }
            (*(*addr_of_mut!(aasworld)).planes.add(i as usize)).dist = LittleFloat((*(*addr_of_mut!(aasworld)).planes.add(i as usize)).dist);
            (*(*addr_of_mut!(aasworld)).planes.add(i as usize)).r#type = LittleLong((*(*addr_of_mut!(aasworld)).planes.add(i as usize)).r#type);
            i += 1;
        } //end for
        //edges
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numedges
        {
            (*(*addr_of_mut!(aasworld)).edges.add(i as usize)).v[0] = LittleLong((*(*addr_of_mut!(aasworld)).edges.add(i as usize)).v[0]);
            (*(*addr_of_mut!(aasworld)).edges.add(i as usize)).v[1] = LittleLong((*(*addr_of_mut!(aasworld)).edges.add(i as usize)).v[1]);
            i += 1;
        } //end for
        //edgeindex
        i = 0;
        while i < (*addr_of_mut!(aasworld)).edgeindexsize
        {
            *(*addr_of_mut!(aasworld)).edgeindex.add(i as usize) = LittleLong(*(*addr_of_mut!(aasworld)).edgeindex.add(i as usize));
            i += 1;
        } //end for
        //faces
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numfaces
        {
            (*(*addr_of_mut!(aasworld)).faces.add(i as usize)).planenum = LittleLong((*(*addr_of_mut!(aasworld)).faces.add(i as usize)).planenum);
            (*(*addr_of_mut!(aasworld)).faces.add(i as usize)).faceflags = LittleLong((*(*addr_of_mut!(aasworld)).faces.add(i as usize)).faceflags);
            (*(*addr_of_mut!(aasworld)).faces.add(i as usize)).numedges = LittleLong((*(*addr_of_mut!(aasworld)).faces.add(i as usize)).numedges);
            (*(*addr_of_mut!(aasworld)).faces.add(i as usize)).firstedge = LittleLong((*(*addr_of_mut!(aasworld)).faces.add(i as usize)).firstedge);
            (*(*addr_of_mut!(aasworld)).faces.add(i as usize)).frontarea = LittleLong((*(*addr_of_mut!(aasworld)).faces.add(i as usize)).frontarea);
            (*(*addr_of_mut!(aasworld)).faces.add(i as usize)).backarea = LittleLong((*(*addr_of_mut!(aasworld)).faces.add(i as usize)).backarea);
            i += 1;
        } //end for
        //face index
        i = 0;
        while i < (*addr_of_mut!(aasworld)).faceindexsize
        {
            *(*addr_of_mut!(aasworld)).faceindex.add(i as usize) = LittleLong(*(*addr_of_mut!(aasworld)).faceindex.add(i as usize));
            i += 1;
        } //end for
        //convex areas
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numareas
        {
            (*(*addr_of_mut!(aasworld)).areas.add(i as usize)).areanum = LittleLong((*(*addr_of_mut!(aasworld)).areas.add(i as usize)).areanum);
            (*(*addr_of_mut!(aasworld)).areas.add(i as usize)).numfaces = LittleLong((*(*addr_of_mut!(aasworld)).areas.add(i as usize)).numfaces);
            (*(*addr_of_mut!(aasworld)).areas.add(i as usize)).firstface = LittleLong((*(*addr_of_mut!(aasworld)).areas.add(i as usize)).firstface);
            j = 0;
            while j < 3
            {
                (*(*addr_of_mut!(aasworld)).areas.add(i as usize)).mins[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).areas.add(i as usize)).mins[j as usize]);
                (*(*addr_of_mut!(aasworld)).areas.add(i as usize)).maxs[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).areas.add(i as usize)).maxs[j as usize]);
                (*(*addr_of_mut!(aasworld)).areas.add(i as usize)).center[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).areas.add(i as usize)).center[j as usize]);
                j += 1;
            } //end for
            i += 1;
        } //end for
        //area settings
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numareasettings
        {
            (*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).contents = LittleLong((*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).contents);
            (*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).areaflags = LittleLong((*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).areaflags);
            (*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).presencetype = LittleLong((*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).presencetype);
            (*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).cluster = LittleLong((*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).cluster);
            (*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).clusterareanum = LittleLong((*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).clusterareanum);
            (*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).numreachableareas = LittleLong((*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).numreachableareas);
            (*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).firstreachablearea = LittleLong((*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).firstreachablearea);
            i += 1;
        } //end for
        //area reachability
        i = 0;
        while i < (*addr_of_mut!(aasworld)).reachabilitysize
        {
            (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).areanum = LittleLong((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).areanum);
            (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).facenum = LittleLong((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).facenum);
            (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).edgenum = LittleLong((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).edgenum);
            j = 0;
            while j < 3
            {
                (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).start[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).start[j as usize]);
                (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).end[j as usize] = LittleFloat((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).end[j as usize]);
                j += 1;
            } //end for
            (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).traveltype = LittleLong((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).traveltype);
            (*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).traveltime = LittleShort((*(*addr_of_mut!(aasworld)).reachability.add(i as usize)).traveltime);
            i += 1;
        } //end for
        //nodes
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numnodes
        {
            (*(*addr_of_mut!(aasworld)).nodes.add(i as usize)).planenum = LittleLong((*(*addr_of_mut!(aasworld)).nodes.add(i as usize)).planenum);
            (*(*addr_of_mut!(aasworld)).nodes.add(i as usize)).children[0] = LittleLong((*(*addr_of_mut!(aasworld)).nodes.add(i as usize)).children[0]);
            (*(*addr_of_mut!(aasworld)).nodes.add(i as usize)).children[1] = LittleLong((*(*addr_of_mut!(aasworld)).nodes.add(i as usize)).children[1]);
            i += 1;
        } //end for
        //cluster portals
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numportals
        {
            (*(*addr_of_mut!(aasworld)).portals.add(i as usize)).areanum = LittleLong((*(*addr_of_mut!(aasworld)).portals.add(i as usize)).areanum);
            (*(*addr_of_mut!(aasworld)).portals.add(i as usize)).frontcluster = LittleLong((*(*addr_of_mut!(aasworld)).portals.add(i as usize)).frontcluster);
            (*(*addr_of_mut!(aasworld)).portals.add(i as usize)).backcluster = LittleLong((*(*addr_of_mut!(aasworld)).portals.add(i as usize)).backcluster);
            (*(*addr_of_mut!(aasworld)).portals.add(i as usize)).clusterareanum[0] = LittleLong((*(*addr_of_mut!(aasworld)).portals.add(i as usize)).clusterareanum[0]);
            (*(*addr_of_mut!(aasworld)).portals.add(i as usize)).clusterareanum[1] = LittleLong((*(*addr_of_mut!(aasworld)).portals.add(i as usize)).clusterareanum[1]);
            i += 1;
        } //end for
        //cluster portal index
        i = 0;
        while i < (*addr_of_mut!(aasworld)).portalindexsize
        {
            *(*addr_of_mut!(aasworld)).portalindex.add(i as usize) = LittleLong(*(*addr_of_mut!(aasworld)).portalindex.add(i as usize));
            i += 1;
        } //end for
        //cluster
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numclusters
        {
            (*(*addr_of_mut!(aasworld)).clusters.add(i as usize)).numareas = LittleLong((*(*addr_of_mut!(aasworld)).clusters.add(i as usize)).numareas);
            (*(*addr_of_mut!(aasworld)).clusters.add(i as usize)).numreachabilityareas = LittleLong((*(*addr_of_mut!(aasworld)).clusters.add(i as usize)).numreachabilityareas);
            (*(*addr_of_mut!(aasworld)).clusters.add(i as usize)).numportals = LittleLong((*(*addr_of_mut!(aasworld)).clusters.add(i as usize)).numportals);
            (*(*addr_of_mut!(aasworld)).clusters.add(i as usize)).firstportal = LittleLong((*(*addr_of_mut!(aasworld)).clusters.add(i as usize)).firstportal);
            i += 1;
        } //end for
    }
} //end of the function AAS_SwapAASData

//===========================================================================
// dump the current loaded aas file
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_DumpAASData()
{
    unsafe {
        (*addr_of_mut!(aasworld)).numbboxes = 0;
        if !(*addr_of_mut!(aasworld)).bboxes.is_null() { FreeMemory((*addr_of_mut!(aasworld)).bboxes as *mut c_void); }
        (*addr_of_mut!(aasworld)).bboxes = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numvertexes = 0;
        if !(*addr_of_mut!(aasworld)).vertexes.is_null() { FreeMemory((*addr_of_mut!(aasworld)).vertexes as *mut c_void); }
        (*addr_of_mut!(aasworld)).vertexes = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numplanes = 0;
        if !(*addr_of_mut!(aasworld)).planes.is_null() { FreeMemory((*addr_of_mut!(aasworld)).planes as *mut c_void); }
        (*addr_of_mut!(aasworld)).planes = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numedges = 0;
        if !(*addr_of_mut!(aasworld)).edges.is_null() { FreeMemory((*addr_of_mut!(aasworld)).edges as *mut c_void); }
        (*addr_of_mut!(aasworld)).edges = ptr::null_mut();
        (*addr_of_mut!(aasworld)).edgeindexsize = 0;
        if !(*addr_of_mut!(aasworld)).edgeindex.is_null() { FreeMemory((*addr_of_mut!(aasworld)).edgeindex as *mut c_void); }
        (*addr_of_mut!(aasworld)).edgeindex = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numfaces = 0;
        if !(*addr_of_mut!(aasworld)).faces.is_null() { FreeMemory((*addr_of_mut!(aasworld)).faces as *mut c_void); }
        (*addr_of_mut!(aasworld)).faces = ptr::null_mut();
        (*addr_of_mut!(aasworld)).faceindexsize = 0;
        if !(*addr_of_mut!(aasworld)).faceindex.is_null() { FreeMemory((*addr_of_mut!(aasworld)).faceindex as *mut c_void); }
        (*addr_of_mut!(aasworld)).faceindex = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numareas = 0;
        if !(*addr_of_mut!(aasworld)).areas.is_null() { FreeMemory((*addr_of_mut!(aasworld)).areas as *mut c_void); }
        (*addr_of_mut!(aasworld)).areas = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numareasettings = 0;
        if !(*addr_of_mut!(aasworld)).areasettings.is_null() { FreeMemory((*addr_of_mut!(aasworld)).areasettings as *mut c_void); }
        (*addr_of_mut!(aasworld)).areasettings = ptr::null_mut();
        (*addr_of_mut!(aasworld)).reachabilitysize = 0;
        if !(*addr_of_mut!(aasworld)).reachability.is_null() { FreeMemory((*addr_of_mut!(aasworld)).reachability as *mut c_void); }
        (*addr_of_mut!(aasworld)).reachability = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numnodes = 0;
        if !(*addr_of_mut!(aasworld)).nodes.is_null() { FreeMemory((*addr_of_mut!(aasworld)).nodes as *mut c_void); }
        (*addr_of_mut!(aasworld)).nodes = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numportals = 0;
        if !(*addr_of_mut!(aasworld)).portals.is_null() { FreeMemory((*addr_of_mut!(aasworld)).portals as *mut c_void); }
        (*addr_of_mut!(aasworld)).portals = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numportals = 0;
        if !(*addr_of_mut!(aasworld)).portalindex.is_null() { FreeMemory((*addr_of_mut!(aasworld)).portalindex as *mut c_void); }
        (*addr_of_mut!(aasworld)).portalindex = ptr::null_mut();
        (*addr_of_mut!(aasworld)).portalindexsize = 0;
        if !(*addr_of_mut!(aasworld)).clusters.is_null() { FreeMemory((*addr_of_mut!(aasworld)).clusters as *mut c_void); }
        (*addr_of_mut!(aasworld)).clusters = ptr::null_mut();
        (*addr_of_mut!(aasworld)).numclusters = 0;
        //
        (*addr_of_mut!(aasworld)).loaded = 0;  // qfalse
        (*addr_of_mut!(aasworld)).initialized = 0;  // qfalse
        (*addr_of_mut!(aasworld)).savefile = 0;  // qfalse
    }
} //end of the function AAS_DumpAASData

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
#[cfg(feature = "AASFILEDEBUG")]
pub fn AAS_FileInfo()
{
    let mut i: c_int;
    let mut n: c_int;
    let mut optimized: c_int;

    unsafe {
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"version = %d\n\0".as_ptr() as *const c_char, AASVERSION);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numvertexes = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numvertexes);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numplanes = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numplanes);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numedges = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numedges);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"edgeindexsize = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).edgeindexsize);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numfaces = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numfaces);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"faceindexsize = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).faceindexsize);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numareas = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numareas);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numareasettings = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numareasettings);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"reachabilitysize = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).reachabilitysize);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numnodes = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numnodes);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numportals = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numportals);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"portalindexsize = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).portalindexsize);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"numclusters = %d\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numclusters);
        //
        n = 0;
        i = 0;
        while i < (*addr_of_mut!(aasworld)).numareasettings
        {
            if ((*(*addr_of_mut!(aasworld)).areasettings.add(i as usize)).areaflags & AREA_GROUNDED) != 0 { n += 1; }
            i += 1;
        } //end for
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"num grounded areas = %d\n\0".as_ptr() as *const c_char, n);
        //
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"planes size %d bytes\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numplanes * mem::size_of::<aas_plane_t>() as c_int);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"areas size %d bytes\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numareas * mem::size_of::<aas_area_t>() as c_int);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"areasettings size %d bytes\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numareasettings * mem::size_of::<aas_areasettings_t>() as c_int);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"nodes size %d bytes\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numnodes * mem::size_of::<aas_node_t>() as c_int);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"reachability size %d bytes\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).reachabilitysize * mem::size_of::<aas_reachability_t>() as c_int);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"portals size %d bytes\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numportals * mem::size_of::<aas_portal_t>() as c_int);
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"clusters size %d bytes\n\0".as_ptr() as *const c_char, (*addr_of_mut!(aasworld)).numclusters * mem::size_of::<aas_cluster_t>() as c_int);

        optimized = (*addr_of_mut!(aasworld)).numplanes * mem::size_of::<aas_plane_t>() as c_int +
                        (*addr_of_mut!(aasworld)).numareas * mem::size_of::<aas_area_t>() as c_int +
                        (*addr_of_mut!(aasworld)).numareasettings * mem::size_of::<aas_areasettings_t>() as c_int +
                        (*addr_of_mut!(aasworld)).numnodes * mem::size_of::<aas_node_t>() as c_int +
                        (*addr_of_mut!(aasworld)).reachabilitysize * mem::size_of::<aas_reachability_t>() as c_int +
                        (*addr_of_mut!(aasworld)).numportals * mem::size_of::<aas_portal_t>() as c_int +
                        (*addr_of_mut!(aasworld)).numclusters * mem::size_of::<aas_cluster_t>() as c_int;
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"optimzed size %d KB\n\0".as_ptr() as *const c_char, optimized >> 10);
    }
} //end of the function AAS_FileInfo

//===========================================================================
// allocate memory and read a lump of a AAS file
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_LoadAASLump(fp: fileHandle_t, offset: c_int, length: c_int, lastoffset: *mut c_int, size: c_int) -> *mut c_char
{
    let mut buf: *mut c_char;
    //
    unsafe {
        if length == 0
        {
            //just alloc a dummy
            return GetClearedHunkMemory((size + 1) as c_ulong) as *mut c_char;
        } //end if
        //seek to the data
        if offset != *lastoffset
        {
            ((*addr_of_mut!(botimport)).Print)(PRT_WARNING, b"AAS file not sequentially read\n\0".as_ptr() as *const c_char);
            if ((*addr_of_mut!(botimport)).FS_Seek)(fp, offset, FS_SEEK_SET) != 0
            {
                AAS_Error(b"can't seek to aas lump\n\0".as_ptr() as *const c_char);
                AAS_DumpAASData();
                ((*addr_of_mut!(botimport)).FS_FCloseFile)(fp);
                return ptr::null_mut();
            } //end if
        } //end if
        //allocate memory
        buf = GetClearedHunkMemory((length + 1) as c_ulong) as *mut c_char;
        //read the data
        if length != 0
        {
            ((*addr_of_mut!(botimport)).FS_Read)(buf as *mut c_void, length, fp);
            *lastoffset += length;
        } //end if
        return buf;
    }
} //end of the function AAS_LoadAASLump

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn AAS_DData(data: *mut u8, size: c_int)
{
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < size
        {
            *data.add(i as usize) ^= (i as u8).wrapping_mul(119);
            i += 1;
        } //end for
    }
} //end of the function AAS_DData

//===========================================================================
// load an aas file
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn AAS_LoadAASFile(filename: *mut c_char) -> c_int
{
    let mut fp: fileHandle_t;
    let mut header: aas_header_t;
    let mut offset: c_int;
    let mut length: c_int;
    let mut lastoffset: c_int;

    unsafe {
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"trying to load %s\n\0".as_ptr() as *const c_char, filename);
        //dump current loaded aas file
        AAS_DumpAASData();
        //open the file
        ((*addr_of_mut!(botimport)).FS_FOpenFile)(filename, &mut fp, FS_READ);
        if fp == 0
        {
            AAS_Error(b"can't open %s\n\0".as_ptr() as *const c_char, filename);
            return BLERR_CANNOTOPENAASFILE;
        } //end if
        //read the header
        ((*addr_of_mut!(botimport)).FS_Read)(&mut header as *mut aas_header_t as *mut c_void, mem::size_of::<aas_header_t>() as c_int, fp);
        lastoffset = mem::size_of::<aas_header_t>() as c_int;
        //check header identification
        header.ident = LittleLong(header.ident);
        if header.ident != AASID
        {
            AAS_Error(b"%s is not an AAS file\n\0".as_ptr() as *const c_char, filename);
            ((*addr_of_mut!(botimport)).FS_FCloseFile)(fp);
            return BLERR_WRONGAASFILEID;
        } //end if
        //check the version
        header.version = LittleLong(header.version);
        //
        if header.version != AASVERSION_OLD && header.version != AASVERSION
        {
            AAS_Error(b"aas file %s is version %i, not %i\n\0".as_ptr() as *const c_char, filename, header.version, AASVERSION);
            ((*addr_of_mut!(botimport)).FS_FCloseFile)(fp);
            return BLERR_WRONGAASFILEVERSION;
        } //end if
        //
        if header.version == AASVERSION
        {
            AAS_DData((&mut header as *mut aas_header_t as *mut u8).add(8), mem::size_of::<aas_header_t>() as c_int - 8);
        } //end if
        //
        (*addr_of_mut!(aasworld)).bspchecksum = atoi(LibVarGetString(b"sv_mapChecksum\0".as_ptr() as *const c_char));
        if LittleLong(header.bspchecksum) != (*addr_of_mut!(aasworld)).bspchecksum
        {
            AAS_Error(b"aas file %s is out of date\n\0".as_ptr() as *const c_char, filename);
            ((*addr_of_mut!(botimport)).FS_FCloseFile)(fp);
            return BLERR_WRONGAASFILEVERSION;
        } //end if
        //load the lumps:
        //bounding boxes
        offset = LittleLong(header.lumps[AASLUMP_BBOXES].fileofs);
        length = LittleLong(header.lumps[AASLUMP_BBOXES].filelen);
        (*addr_of_mut!(aasworld)).bboxes = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_bbox_t>() as c_int) as *mut aas_bbox_t;
        (*addr_of_mut!(aasworld)).numbboxes = length / mem::size_of::<aas_bbox_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numbboxes != 0 && (*addr_of_mut!(aasworld)).bboxes.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //vertexes
        offset = LittleLong(header.lumps[AASLUMP_VERTEXES].fileofs);
        length = LittleLong(header.lumps[AASLUMP_VERTEXES].filelen);
        (*addr_of_mut!(aasworld)).vertexes = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_vertex_t>() as c_int) as *mut aas_vertex_t;
        (*addr_of_mut!(aasworld)).numvertexes = length / mem::size_of::<aas_vertex_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numvertexes != 0 && (*addr_of_mut!(aasworld)).vertexes.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //planes
        offset = LittleLong(header.lumps[AASLUMP_PLANES].fileofs);
        length = LittleLong(header.lumps[AASLUMP_PLANES].filelen);
        (*addr_of_mut!(aasworld)).planes = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_plane_t>() as c_int) as *mut aas_plane_t;
        (*addr_of_mut!(aasworld)).numplanes = length / mem::size_of::<aas_plane_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numplanes != 0 && (*addr_of_mut!(aasworld)).planes.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //edges
        offset = LittleLong(header.lumps[AASLUMP_EDGES].fileofs);
        length = LittleLong(header.lumps[AASLUMP_EDGES].filelen);
        (*addr_of_mut!(aasworld)).edges = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_edge_t>() as c_int) as *mut aas_edge_t;
        (*addr_of_mut!(aasworld)).numedges = length / mem::size_of::<aas_edge_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numedges != 0 && (*addr_of_mut!(aasworld)).edges.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //edgeindex
        offset = LittleLong(header.lumps[AASLUMP_EDGEINDEX].fileofs);
        length = LittleLong(header.lumps[AASLUMP_EDGEINDEX].filelen);
        (*addr_of_mut!(aasworld)).edgeindex = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_edgeindex_t>() as c_int) as *mut aas_edgeindex_t;
        (*addr_of_mut!(aasworld)).edgeindexsize = length / mem::size_of::<aas_edgeindex_t>() as c_int;
        if (*addr_of_mut!(aasworld)).edgeindexsize != 0 && (*addr_of_mut!(aasworld)).edgeindex.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //faces
        offset = LittleLong(header.lumps[AASLUMP_FACES].fileofs);
        length = LittleLong(header.lumps[AASLUMP_FACES].filelen);
        (*addr_of_mut!(aasworld)).faces = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_face_t>() as c_int) as *mut aas_face_t;
        (*addr_of_mut!(aasworld)).numfaces = length / mem::size_of::<aas_face_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numfaces != 0 && (*addr_of_mut!(aasworld)).faces.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //faceindex
        offset = LittleLong(header.lumps[AASLUMP_FACEINDEX].fileofs);
        length = LittleLong(header.lumps[AASLUMP_FACEINDEX].filelen);
        (*addr_of_mut!(aasworld)).faceindex = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_faceindex_t>() as c_int) as *mut aas_faceindex_t;
        (*addr_of_mut!(aasworld)).faceindexsize = length / mem::size_of::<aas_faceindex_t>() as c_int;
        if (*addr_of_mut!(aasworld)).faceindexsize != 0 && (*addr_of_mut!(aasworld)).faceindex.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //convex areas
        offset = LittleLong(header.lumps[AASLUMP_AREAS].fileofs);
        length = LittleLong(header.lumps[AASLUMP_AREAS].filelen);
        (*addr_of_mut!(aasworld)).areas = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_area_t>() as c_int) as *mut aas_area_t;
        (*addr_of_mut!(aasworld)).numareas = length / mem::size_of::<aas_area_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numareas != 0 && (*addr_of_mut!(aasworld)).areas.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //area settings
        offset = LittleLong(header.lumps[AASLUMP_AREASETTINGS].fileofs);
        length = LittleLong(header.lumps[AASLUMP_AREASETTINGS].filelen);
        (*addr_of_mut!(aasworld)).areasettings = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_areasettings_t>() as c_int) as *mut aas_areasettings_t;
        (*addr_of_mut!(aasworld)).numareasettings = length / mem::size_of::<aas_areasettings_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numareasettings != 0 && (*addr_of_mut!(aasworld)).areasettings.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //reachability list
        offset = LittleLong(header.lumps[AASLUMP_REACHABILITY].fileofs);
        length = LittleLong(header.lumps[AASLUMP_REACHABILITY].filelen);
        (*addr_of_mut!(aasworld)).reachability = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_reachability_t>() as c_int) as *mut aas_reachability_t;
        (*addr_of_mut!(aasworld)).reachabilitysize = length / mem::size_of::<aas_reachability_t>() as c_int;
        if (*addr_of_mut!(aasworld)).reachabilitysize != 0 && (*addr_of_mut!(aasworld)).reachability.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //nodes
        offset = LittleLong(header.lumps[AASLUMP_NODES].fileofs);
        length = LittleLong(header.lumps[AASLUMP_NODES].filelen);
        (*addr_of_mut!(aasworld)).nodes = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_node_t>() as c_int) as *mut aas_node_t;
        (*addr_of_mut!(aasworld)).numnodes = length / mem::size_of::<aas_node_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numnodes != 0 && (*addr_of_mut!(aasworld)).nodes.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //cluster portals
        offset = LittleLong(header.lumps[AASLUMP_PORTALS].fileofs);
        length = LittleLong(header.lumps[AASLUMP_PORTALS].filelen);
        (*addr_of_mut!(aasworld)).portals = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_portal_t>() as c_int) as *mut aas_portal_t;
        (*addr_of_mut!(aasworld)).numportals = length / mem::size_of::<aas_portal_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numportals != 0 && (*addr_of_mut!(aasworld)).portals.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //cluster portal index
        offset = LittleLong(header.lumps[AASLUMP_PORTALINDEX].fileofs);
        length = LittleLong(header.lumps[AASLUMP_PORTALINDEX].filelen);
        (*addr_of_mut!(aasworld)).portalindex = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_portalindex_t>() as c_int) as *mut aas_portalindex_t;
        (*addr_of_mut!(aasworld)).portalindexsize = length / mem::size_of::<aas_portalindex_t>() as c_int;
        if (*addr_of_mut!(aasworld)).portalindexsize != 0 && (*addr_of_mut!(aasworld)).portalindex.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //clusters
        offset = LittleLong(header.lumps[AASLUMP_CLUSTERS].fileofs);
        length = LittleLong(header.lumps[AASLUMP_CLUSTERS].filelen);
        (*addr_of_mut!(aasworld)).clusters = AAS_LoadAASLump(fp, offset, length, &mut lastoffset, mem::size_of::<aas_cluster_t>() as c_int) as *mut aas_cluster_t;
        (*addr_of_mut!(aasworld)).numclusters = length / mem::size_of::<aas_cluster_t>() as c_int;
        if (*addr_of_mut!(aasworld)).numclusters != 0 && (*addr_of_mut!(aasworld)).clusters.is_null() { return BLERR_CANNOTREADAASLUMP; }
        //swap everything
        AAS_SwapAASData();
        //aas file is loaded
        (*addr_of_mut!(aasworld)).loaded = 1;  // qtrue
        //close the file
        ((*addr_of_mut!(botimport)).FS_FCloseFile)(fp);
        //
        #[cfg(feature = "AASFILEDEBUG")]
        AAS_FileInfo();
        //
        return BLERR_NOERROR;
    }
} //end of the function AAS_LoadAASFile

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
static mut AAS_WriteAASLump_offset: c_int = 0;

pub fn AAS_WriteAASLump(fp: fileHandle_t, h: *mut aas_header_t, lumpnum: usize, data: *mut c_void, length: c_int) -> c_int
{
    let mut lump: *mut aas_lump_t;

    unsafe {
        lump = &mut (*h).lumps[lumpnum];

        (*lump).fileofs = LittleLong(AAS_WriteAASLump_offset);	//LittleLong(ftell(fp));
        (*lump).filelen = LittleLong(length);

        if length > 0
        {
            ((*addr_of_mut!(botimport)).FS_Write)(data, length, fp);
        } //end if

        AAS_WriteAASLump_offset += length;

        return 1;  // qtrue
    }
} //end of the function AAS_WriteAASLump

//===========================================================================
// aas data is useless after writing to file because it is byte swapped
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_WriteAASFile(filename: *mut c_char) -> qboolean
{
    let mut header: aas_header_t;
    let mut fp: fileHandle_t;

    unsafe {
        ((*addr_of_mut!(botimport)).Print)(PRT_MESSAGE, b"writing %s\n\0".as_ptr() as *const c_char, filename);
        //swap the aas data
        AAS_SwapAASData();
        //initialize the file header
        Com_Memset(&mut header as *mut aas_header_t as *mut c_void, 0, mem::size_of::<aas_header_t>() as c_int);
        header.ident = LittleLong(AASID);
        header.version = LittleLong(AASVERSION);
        header.bspchecksum = LittleLong((*addr_of_mut!(aasworld)).bspchecksum);
        //open a new file
        ((*addr_of_mut!(botimport)).FS_FOpenFile)(filename, &mut fp, FS_WRITE);
        if fp == 0
        {
            ((*addr_of_mut!(botimport)).Print)(PRT_ERROR, b"error opening %s\n\0".as_ptr() as *const c_char, filename);
            return 0;  // qfalse
        } //end if
        //write the header
        ((*addr_of_mut!(botimport)).FS_Write)(&header as *const aas_header_t as *const c_void, mem::size_of::<aas_header_t>() as c_int, fp);
        AAS_WriteAASLump_offset = mem::size_of::<aas_header_t>() as c_int;
        //add the data lumps to the file
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_BBOXES, (*addr_of_mut!(aasworld)).bboxes as *mut c_void,
            (*addr_of_mut!(aasworld)).numbboxes * mem::size_of::<aas_bbox_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_VERTEXES, (*addr_of_mut!(aasworld)).vertexes as *mut c_void,
            (*addr_of_mut!(aasworld)).numvertexes * mem::size_of::<aas_vertex_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_PLANES, (*addr_of_mut!(aasworld)).planes as *mut c_void,
            (*addr_of_mut!(aasworld)).numplanes * mem::size_of::<aas_plane_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_EDGES, (*addr_of_mut!(aasworld)).edges as *mut c_void,
            (*addr_of_mut!(aasworld)).numedges * mem::size_of::<aas_edge_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_EDGEINDEX, (*addr_of_mut!(aasworld)).edgeindex as *mut c_void,
            (*addr_of_mut!(aasworld)).edgeindexsize * mem::size_of::<aas_edgeindex_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_FACES, (*addr_of_mut!(aasworld)).faces as *mut c_void,
            (*addr_of_mut!(aasworld)).numfaces * mem::size_of::<aas_face_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_FACEINDEX, (*addr_of_mut!(aasworld)).faceindex as *mut c_void,
            (*addr_of_mut!(aasworld)).faceindexsize * mem::size_of::<aas_faceindex_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_AREAS, (*addr_of_mut!(aasworld)).areas as *mut c_void,
            (*addr_of_mut!(aasworld)).numareas * mem::size_of::<aas_area_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_AREASETTINGS, (*addr_of_mut!(aasworld)).areasettings as *mut c_void,
            (*addr_of_mut!(aasworld)).numareasettings * mem::size_of::<aas_areasettings_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_REACHABILITY, (*addr_of_mut!(aasworld)).reachability as *mut c_void,
            (*addr_of_mut!(aasworld)).reachabilitysize * mem::size_of::<aas_reachability_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_NODES, (*addr_of_mut!(aasworld)).nodes as *mut c_void,
            (*addr_of_mut!(aasworld)).numnodes * mem::size_of::<aas_node_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_PORTALS, (*addr_of_mut!(aasworld)).portals as *mut c_void,
            (*addr_of_mut!(aasworld)).numportals * mem::size_of::<aas_portal_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_PORTALINDEX, (*addr_of_mut!(aasworld)).portalindex as *mut c_void,
            (*addr_of_mut!(aasworld)).portalindexsize * mem::size_of::<aas_portalindex_t>() as c_int) == 0 { return 0; }
        if AAS_WriteAASLump(fp, &mut header, AASLUMP_CLUSTERS, (*addr_of_mut!(aasworld)).clusters as *mut c_void,
            (*addr_of_mut!(aasworld)).numclusters * mem::size_of::<aas_cluster_t>() as c_int) == 0 { return 0; }
        //rewrite the header with the added lumps
        ((*addr_of_mut!(botimport)).FS_Seek)(fp, 0, FS_SEEK_SET);
        AAS_DData((&mut header as *mut aas_header_t as *mut u8).add(8), mem::size_of::<aas_header_t>() as c_int - 8);
        ((*addr_of_mut!(botimport)).FS_Write)(&header as *const aas_header_t as *const c_void, mem::size_of::<aas_header_t>() as c_int, fp);
        //close the file
        ((*addr_of_mut!(botimport)).FS_FCloseFile)(fp);
        return 1;  // qtrue
    }
} //end of the function AAS_WriteAASFile
