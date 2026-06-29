
/*****************************************************************************
 * name:		be_aas_sample.c
 *
 * desc:		AAS environment sampling
 *
 * $Archive: /MissionPack/code/botlib/be_aas_sample.c $
 * $Author: Ttimo $
 * $Revision: 13 $
 * $Modtime: 4/13/01 4:45p $
 * $Date: 4/13/01 4:45p $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_char, c_void, c_float};

// Stub imports from external modules
// These would be defined in their respective header translations
extern "C" {
    static mut botimport: botlib_import_t;
    static mut aasworld: aas_world_t;
    #[cfg(not(feature = "vm"))]
    static mut bot_developer: c_int;

    fn LibVarValue(var: *const c_char, default_val: *const c_char) -> c_float;
    fn GetHunkMemory(size: usize) -> *mut c_void;
    fn GetClearedHunkMemory(size: usize) -> *mut c_void;
    fn FreeMemory(ptr: *mut c_void);
    fn Com_Memset(dest: *mut c_void, val: c_int, count: usize);

    fn AAS_EntityCollision(
        entnum: c_int,
        start: *const [c_float; 3],
        mins: *const [c_float; 3],
        maxs: *const [c_float; 3],
        end: *const [c_float; 3],
        contents: c_int,
        bsptrace: *mut bsp_trace_t,
    ) -> c_int;
    fn AAS_AreaReachability(areanum: c_int) -> c_int;
}

// Type definitions
pub type vec_t = c_float;
pub type vec3_t = [c_float; 3];
pub type qboolean = c_int;

// Presence type constants
const PRESENCE_NORMAL: c_int = 1;
const PRESENCE_CROUCH: c_int = 2;
const PRESENCE_NONE: c_int = 0;

// Flags
const FACE_GROUND: c_int = 1;

// Print levels
const PRT_MESSAGE: c_int = 0;
const PRT_ERROR: c_int = 1;
const PRT_FATAL: c_int = 2;

// Contents flags
const CONTENTS_SOLID: c_int = 1;
const CONTENTS_PLAYERCLIP: c_int = 2;

// //#define AAS_SAMPLE_DEBUG

#[allow(non_upper_case_globals)]
const BBOX_NORMAL_EPSILON: c_float = 0.001;

#[allow(non_upper_case_globals)]
const ON_EPSILON: c_float = 0.0; //0.0005

#[allow(non_upper_case_globals)]
const TRACEPLANE_EPSILON: c_float = 0.125;

// Stub struct types for external definitions
#[repr(C)]
pub struct botlib_import_t {
    pub Print: extern "C" fn(level: c_int, fmt: *const c_char, ...) -> (),
    // other fields not used in this file
}

#[repr(C)]
pub struct aas_world_t {
    pub loaded: c_int,
    pub initialized: c_int,
    pub linkheapsize: c_int,
    pub linkheap: *mut aas_link_t,
    pub freelinks: *mut aas_link_t,
    pub arealinkedentities: *mut *mut aas_link_t,
    pub numareas: c_int,
    pub numareasettings: c_int,
    pub numnodes: c_int,
    pub numplanes: c_int,
    pub numclusters: c_int,
    pub nodes: *mut aas_node_t,
    pub planes: *mut aas_plane_t,
    pub areas: *mut aas_area_t,
    pub areasettings: *mut aas_areasettings_t,
    pub clusters: *mut aas_cluster_t,
    pub portals: *mut aas_portal_t,
    pub faceindex: *mut c_int,
    pub edgeindex: *mut c_int,
    pub edges: *mut aas_edge_t,
    pub faces: *mut aas_face_t,
    pub vertexes: *mut *mut c_float,
}

#[repr(C)]
pub struct aas_link_t {
    pub prev_ent: *mut aas_link_t,
    pub next_ent: *mut aas_link_t,
    pub prev_area: *mut aas_link_t,
    pub next_area: *mut aas_link_t,
    pub entnum: c_int,
    pub areanum: c_int,
}

#[repr(C)]
pub struct aas_node_t {
    pub planenum: c_int,
    pub children: [c_int; 2],
}

#[repr(C)]
pub struct aas_plane_t {
    pub normal: vec3_t,
    pub dist: c_float,
    pub type_: c_int,
}

#[repr(C)]
pub struct aas_area_t {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub center: vec3_t,
    pub numfaces: c_int,
    pub firstface: c_int,
}

#[repr(C)]
pub struct aas_areasettings_t {
    pub contents: c_int,
    pub areaflags: c_int,
    pub presencetype: c_int,
    pub cluster: c_int,
    pub clusterareanum: c_int,
}

#[repr(C)]
pub struct aas_edge_t {
    pub v: [c_int; 2],
}

#[repr(C)]
pub struct aas_face_t {
    pub planenum: c_int,
    pub numedges: c_int,
    pub firstedge: c_int,
    pub faceflags: c_int,
}

#[repr(C)]
pub struct bsp_trace_t {
    pub fraction: c_float,
    pub startsolid: c_int,
    pub ent: c_int,
    pub endpos: vec3_t,
}

#[repr(C)]
pub struct aas_trace_t {
    pub fraction: c_float,
    pub startsolid: c_int,
    pub ent: c_int,
    pub area: c_int,
    pub planenum: c_int,
    pub endpos: vec3_t,
    pub lastarea: c_int,
}

#[repr(C)]
pub struct aas_cluster_t {
    pub numreachabilityareas: c_int,
}

#[repr(C)]
pub struct aas_portal_t {
    pub frontcluster: c_int,
    pub clusterareanum: [c_int; 2],
}

#[repr(C)]
pub struct aas_areainfo_t {
    pub cluster: c_int,
    pub contents: c_int,
    pub flags: c_int,
    pub presencetype: c_int,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub center: vec3_t,
}

#[repr(C)]
struct aas_tracestack_t {
    start: vec3_t,         //start point of the piece of line to trace
    end: vec3_t,           //end point of the piece of line to trace
    planenum: c_int,       //last plane used as splitter
    nodenum: c_int,        //node found after splitting with planenum
}

#[repr(C)]
struct aas_linkstack_t {
    nodenum: c_int,        //node found after splitting
}

pub static mut numaaslinks: c_int = 0;

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_PresenceTypeBoundingBox(presencetype: c_int, mins: &mut vec3_t, maxs: &mut vec3_t)
{
    let mut index: c_int;
    //bounding box size for each presence type
    let boxmins: [[c_float; 3]; 3] = [[0.0, 0.0, 0.0], [-15.0, -15.0, -24.0], [-15.0, -15.0, -24.0]];
    let boxmaxs: [[c_float; 3]; 3] = [[0.0, 0.0, 0.0], [15.0, 15.0, 32.0], [15.0, 15.0, 8.0]];

    if presencetype == PRESENCE_NORMAL {
        index = 1;
    } else if presencetype == PRESENCE_CROUCH {
        index = 2;
    } else {
        unsafe {
            (botimport.Print)(PRT_FATAL, b"AAS_PresenceTypeBoundingBox: unknown presence type\n\0".as_ptr() as *const c_char);
        }
        index = 2;
    } //end if
    mins[0] = boxmins[index as usize][0];
    mins[1] = boxmins[index as usize][1];
    mins[2] = boxmins[index as usize][2];
    maxs[0] = boxmaxs[index as usize][0];
    maxs[1] = boxmaxs[index as usize][1];
    maxs[2] = boxmaxs[index as usize][2];
} //end of the function AAS_PresenceTypeBoundingBox
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_InitAASLinkHeap()
{
    let mut i: c_int;
    let mut max_aaslinks: c_int;

    unsafe {
        max_aaslinks = aasworld.linkheapsize;
        //if there's no link heap present
        if aasworld.linkheap.is_null() {
            #[cfg(feature = "vm")]
            {
                max_aaslinks = 6144;
            }
            #[cfg(not(feature = "vm"))]
            {
                max_aaslinks = LibVarValue(b"max_aaslinks\0".as_ptr() as *const c_char, b"6144\0".as_ptr() as *const c_char) as c_int;
            }
            if max_aaslinks < 0 { max_aaslinks = 0; }
            aasworld.linkheapsize = max_aaslinks;
            aasworld.linkheap = GetHunkMemory((max_aaslinks as usize) * core::mem::size_of::<aas_link_t>()) as *mut aas_link_t;
        } //end if
        //link the links on the heap
        (*aasworld.linkheap.offset(0)).prev_ent = core::ptr::null_mut();
        (*aasworld.linkheap.offset(0)).next_ent = aasworld.linkheap.offset(1);
        i = 1;
        while i < max_aaslinks-1 {
            (*aasworld.linkheap.offset(i as isize)).prev_ent = aasworld.linkheap.offset((i - 1) as isize);
            (*aasworld.linkheap.offset(i as isize)).next_ent = aasworld.linkheap.offset((i + 1) as isize);
            i += 1;
        } //end for
        (*aasworld.linkheap.offset((max_aaslinks-1) as isize)).prev_ent = aasworld.linkheap.offset((max_aaslinks-2) as isize);
        (*aasworld.linkheap.offset((max_aaslinks-1) as isize)).next_ent = core::ptr::null_mut();
        //pointer to the first free link
        aasworld.freelinks = aasworld.linkheap.offset(0);
        //
        numaaslinks = max_aaslinks;
    }
} //end of the function AAS_InitAASLinkHeap
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_FreeAASLinkHeap()
{
    unsafe {
        if !aasworld.linkheap.is_null() {
            FreeMemory(aasworld.linkheap as *mut c_void);
        }
        aasworld.linkheap = core::ptr::null_mut();
        aasworld.linkheapsize = 0;
    }
} //end of the function AAS_FreeAASLinkHeap
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_AllocAASLink() -> *mut aas_link_t
{
    let link: *mut aas_link_t;

    unsafe {
        link = aasworld.freelinks;
        if link.is_null() {
            #[cfg(not(feature = "vm"))]
            if bot_developer == 0 {
                // Don't print if bot_developer is false
            } else {
                (botimport.Print)(PRT_FATAL, b"empty aas link heap\n\0".as_ptr() as *const c_char);
            }
            #[cfg(feature = "vm")]
            {
                (botimport.Print)(PRT_FATAL, b"empty aas link heap\n\0".as_ptr() as *const c_char);
            }
            return core::ptr::null_mut();
        } //end if
        if !aasworld.freelinks.is_null() {
            aasworld.freelinks = (*aasworld.freelinks).next_ent;
        }
        if !aasworld.freelinks.is_null() {
            (*aasworld.freelinks).prev_ent = core::ptr::null_mut();
        }
        numaaslinks -= 1;
        link
    }
} //end of the function AAS_AllocAASLink
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_DeAllocAASLink(link: *mut aas_link_t)
{
    unsafe {
        if !aasworld.freelinks.is_null() {
            (*aasworld.freelinks).prev_ent = link;
        }
        (*link).prev_ent = core::ptr::null_mut();
        (*link).next_ent = aasworld.freelinks;
        (*link).prev_area = core::ptr::null_mut();
        (*link).next_area = core::ptr::null_mut();
        aasworld.freelinks = link;
        numaaslinks += 1;
    }
} //end of the function AAS_DeAllocAASLink
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_InitAASLinkedEntities()
{
    unsafe {
        if aasworld.loaded == 0 { return; }
        if !aasworld.arealinkedentities.is_null() {
            FreeMemory(aasworld.arealinkedentities as *mut c_void);
        }
        aasworld.arealinkedentities = GetClearedHunkMemory(
                        (aasworld.numareas as usize) * core::mem::size_of::<*mut aas_link_t>()) as *mut *mut aas_link_t;
    }
} //end of the function AAS_InitAASLinkedEntities
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_FreeAASLinkedEntities()
{
    unsafe {
        if !aasworld.arealinkedentities.is_null() {
            FreeMemory(aasworld.arealinkedentities as *mut c_void);
        }
        aasworld.arealinkedentities = core::ptr::null_mut();
    }
} //end of the function AAS_InitAASLinkedEntities
//===========================================================================
// returns the AAS area the point is in
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_PointAreaNum(point: &vec3_t) -> c_int
{
    let mut nodenum: c_int;
    let mut dist: c_float;
    let aasnode: *mut aas_node_t;
    let plane: *mut aas_plane_t;

    unsafe {
        if aasworld.loaded == 0 {
            (botimport.Print)(PRT_ERROR, b"AAS_PointAreaNum: aas not loaded\n\0".as_ptr() as *const c_char);
            return 0;
        } //end if

        //start with node 1 because node zero is a dummy used for solid leafs
        nodenum = 1;
        while nodenum > 0 {
    //		botimport.Print(PRT_MESSAGE, "[%d]", nodenum);
    #[cfg(feature = "aas_sample_debug")]
            {
                if nodenum >= aasworld.numnodes {
                    (botimport.Print)(PRT_ERROR, b"nodenum = %d >= aasworld.numnodes = %d\n\0".as_ptr() as *const c_char, nodenum, aasworld.numnodes);
                    return 0;
                } //end if
            }
            aasnode = aasworld.nodes.offset(nodenum as isize);
    #[cfg(feature = "aas_sample_debug")]
            {
                if (*aasnode).planenum < 0 || (*aasnode).planenum >= aasworld.numplanes {
                    (botimport.Print)(PRT_ERROR, b"node->planenum = %d >= aasworld.numplanes = %d\n\0".as_ptr() as *const c_char, (*aasnode).planenum, aasworld.numplanes);
                    return 0;
                } //end if
            }
            plane = aasworld.planes.offset((*aasnode).planenum as isize);
            dist = point[0] * (*plane).normal[0] + point[1] * (*plane).normal[1] + point[2] * (*plane).normal[2] - (*plane).dist;
            if dist > 0.0 { nodenum = (*aasnode).children[0]; }
            else { nodenum = (*aasnode).children[1]; }
        } //end while
        if nodenum == 0 {
    #[cfg(feature = "aas_sample_debug")]
            (botimport.Print)(PRT_MESSAGE, b"in solid\n\0".as_ptr() as *const c_char);
            return 0;
        } //end if
        -nodenum
    }
} //end of the function AAS_PointAreaNum
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn AAS_PointReachabilityAreaIndex( origin: *const vec3_t ) -> c_int
{
    let mut areanum: c_int;
    let mut cluster: c_int;
    let mut i: c_int;
    let mut index: c_int;

    unsafe {
        if aasworld.initialized == 0 {
            return 0;
        }

        if origin.is_null() {
            index = 0;
            i = 0;
            while i < aasworld.numclusters {
                index += (*aasworld.clusters.offset(i as isize)).numreachabilityareas;
                i += 1;
            } //end for
            return index;
        } //end if

        areanum = AAS_PointAreaNum(&*origin);
        if areanum == 0 || AAS_AreaReachability(areanum) == 0 {
            return 0;
        }
        cluster = (*aasworld.areasettings.offset(areanum as isize)).cluster;
        areanum = (*aasworld.areasettings.offset(areanum as isize)).clusterareanum;
        if cluster < 0 {
            cluster = (*aasworld.portals.offset((-cluster) as isize)).frontcluster;
            areanum = (*aasworld.portals.offset((-cluster) as isize)).clusterareanum[0];
        } //end if

        index = 0;
        i = 0;
        while i < cluster {
            index += (*aasworld.clusters.offset(i as isize)).numreachabilityareas;
            i += 1;
        } //end for
        index += areanum;
        index
    }
} //end of the function AAS_PointReachabilityAreaIndex
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_AreaCluster(areanum: c_int) -> c_int
{
    unsafe {
        if areanum <= 0 || areanum >= aasworld.numareas {
            (botimport.Print)(PRT_ERROR, b"AAS_AreaCluster: invalid area number\n\0".as_ptr() as *const c_char);
            return 0;
        } //end if
        (*aasworld.areasettings.offset(areanum as isize)).cluster
    }
} //end of the function AAS_AreaCluster
//===========================================================================
// returns the presence types of the given area
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_AreaPresenceType(areanum: c_int) -> c_int
{
    unsafe {
        if aasworld.loaded == 0 { return 0; }
        if areanum <= 0 || areanum >= aasworld.numareas {
            (botimport.Print)(PRT_ERROR, b"AAS_AreaPresenceType: invalid area number\n\0".as_ptr() as *const c_char);
            return 0;
        } //end if
        (*aasworld.areasettings.offset(areanum as isize)).presencetype
    }
} //end of the function AAS_AreaPresenceType
//===========================================================================
// returns the presence type at the given point
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_PointPresenceType(point: &vec3_t) -> c_int
{
    let areanum: c_int;

    unsafe {
        if aasworld.loaded == 0 { return 0; }

        areanum = AAS_PointAreaNum(point);
        if areanum == 0 { return PRESENCE_NONE; }
        (*aasworld.areasettings.offset(areanum as isize)).presencetype
    }
} //end of the function AAS_PointPresenceType
//===========================================================================
// calculates the minimum distance between the origin of the box and the
// given plane when both will collide on the given side of the plane
//
// normal	=	normal vector of plane to calculate distance from
// mins		=	minimums of box relative to origin
// maxs		=	maximums of box relative to origin
// side		=	side of the plane we want to calculate the distance from
//				0 normal vector side
//				1 not normal vector side
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_BoxOriginDistanceFromPlane(normal: &vec3_t, mins: &vec3_t, maxs: &vec3_t, side: c_int) -> c_float
{
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];
    let mut i: c_int;

    //swap maxs and mins when on the other side of the plane
    if side != 0 {
        //get a point of the box that would be one of the first
        //to collide with the plane
        i = 0;
        while i < 3 {
            if normal[i as usize] > BBOX_NORMAL_EPSILON { v1[i as usize] = maxs[i as usize]; }
            else if normal[i as usize] < -BBOX_NORMAL_EPSILON { v1[i as usize] = mins[i as usize]; }
            else { v1[i as usize] = 0.0; }
            i += 1;
        } //end for
    } //end if
    else {
        //get a point of the box that would be one of the first
        //to collide with the plane
        i = 0;
        while i < 3 {
            if normal[i as usize] > BBOX_NORMAL_EPSILON { v1[i as usize] = mins[i as usize]; }
            else if normal[i as usize] < -BBOX_NORMAL_EPSILON { v1[i as usize] = maxs[i as usize]; }
            else { v1[i as usize] = 0.0; }
            i += 1;
        } //end for
    } //end else
    //
    v2[0] = normal[0];
    v2[1] = normal[1];
    v2[2] = normal[2];
    v2[0] = -v2[0];
    v2[1] = -v2[1];
    v2[2] = -v2[2];
//	VectorNegate(normal, v2);
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
} //end of the function AAS_BoxOriginDistanceFromPlane
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_AreaEntityCollision(areanum: c_int, start: &vec3_t, end: &vec3_t,
                                    presencetype: c_int, passent: c_int, trace: &mut aas_trace_t) -> c_int
{
    let mut collision: c_int;
    let mut boxmins: vec3_t = [0.0; 3];
    let mut boxmaxs: vec3_t = [0.0; 3];
    let mut link: *mut aas_link_t;
    let mut bsptrace: bsp_trace_t = unsafe { core::mem::zeroed() };

    AAS_PresenceTypeBoundingBox(presencetype, &mut boxmins, &mut boxmaxs);

    unsafe {
        Com_Memset(&mut bsptrace as *mut _ as *mut c_void, 0, core::mem::size_of::<bsp_trace_t>()); //make compiler happy
    }
    //assume no collision
    bsptrace.fraction = 1.0;
    collision = 0;
    unsafe {
        link = *aasworld.arealinkedentities.offset(areanum as isize);
        while !link.is_null() {
            //ignore the pass entity
            if (*link).entnum == passent {
                link = (*link).next_ent;
                continue;
            }
            //
            if AAS_EntityCollision((*link).entnum, start, &boxmins, &boxmaxs, end,
                                        CONTENTS_SOLID|CONTENTS_PLAYERCLIP, &mut bsptrace) != 0 {
                collision = 1;
            } //end if
            link = (*link).next_ent;
        } //end for
    }
    if collision != 0 {
        trace.startsolid = bsptrace.startsolid;
        trace.ent = bsptrace.ent;
        trace.endpos[0] = bsptrace.endpos[0];
        trace.endpos[1] = bsptrace.endpos[1];
        trace.endpos[2] = bsptrace.endpos[2];
        trace.area = 0;
        trace.planenum = 0;
        return 1;
    } //end if
    0
} //end of the function AAS_AreaEntityCollision
//===========================================================================
// recursive subdivision of the line by the BSP tree.
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_TraceClientBBox(start: &vec3_t, end: &vec3_t, presencetype: c_int,
                                                passent: c_int) -> aas_trace_t
{
    let mut side: c_int;
    let mut nodenum: c_int;
    let mut tmpplanenum: c_int;
    let mut front: c_float;
    let mut back: c_float;
    let mut frac: c_float;
    let mut cur_start: vec3_t = [0.0; 3];
    let mut cur_end: vec3_t = [0.0; 3];
    let mut cur_mid: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];
    let mut tracestack: [aas_tracestack_t; 127] = unsafe { core::mem::zeroed() };
    let mut tstack_p: *mut aas_tracestack_t;
    let aasnode: *mut aas_node_t;
    let plane: *mut aas_plane_t;
    let mut trace: aas_trace_t = unsafe { core::mem::zeroed() };

    //clear the trace structure
    unsafe {
        Com_Memset(&mut trace as *mut _ as *mut c_void, 0, core::mem::size_of::<aas_trace_t>());
    }

    unsafe {
        if aasworld.loaded == 0 { return trace; }

        tstack_p = tracestack.as_mut_ptr();
        //we start with the whole line on the stack
        (*tstack_p).start[0] = start[0];
        (*tstack_p).start[1] = start[1];
        (*tstack_p).start[2] = start[2];
        (*tstack_p).end[0] = end[0];
        (*tstack_p).end[1] = end[1];
        (*tstack_p).end[2] = end[2];
        (*tstack_p).planenum = 0;
        //start with node 1 because node zero is a dummy for a solid leaf
        (*tstack_p).nodenum = 1;		//starting at the root of the tree
        tstack_p = tstack_p.offset(1);

        loop {
            //pop up the stack
            tstack_p = tstack_p.offset(-1);
            //if the trace stack is empty (ended up with a piece of the
            //line to be traced in an area)
            if tstack_p < tracestack.as_mut_ptr() {
                tstack_p = tstack_p.offset(1);
                //nothing was hit
                trace.startsolid = 0;
                trace.fraction = 1.0;
                //endpos is the end of the line
                trace.endpos[0] = end[0];
                trace.endpos[1] = end[1];
                trace.endpos[2] = end[2];
                //nothing hit
                trace.ent = 0;
                trace.area = 0;
                trace.planenum = 0;
                return trace;
            } //end if
            //number of the current node to test the line against
            nodenum = (*tstack_p).nodenum;
            //if it is an area
            if nodenum < 0 {
    #[cfg(feature = "aas_sample_debug")]
                {
                    if -nodenum > aasworld.numareasettings {
                        (botimport.Print)(PRT_ERROR, b"AAS_TraceBoundingBox: -nodenum out of range\n\0".as_ptr() as *const c_char);
                        return trace;
                    } //end if
                }
                //botimport.Print(PRT_MESSAGE, "areanum = %d, must be %d\n", -nodenum, AAS_PointAreaNum(start));
                //if can't enter the area because it hasn't got the right presence type
                if ((*aasworld.areasettings.offset((-nodenum) as isize)).presencetype & presencetype) == 0 {
                    //if the start point is still the initial start point
                    //NOTE: no need for epsilons because the points will be
                    //exactly the same when they're both the start point
                    if (*tstack_p).start[0] == start[0] &&
                            (*tstack_p).start[1] == start[1] &&
                            (*tstack_p).start[2] == start[2] {
                        trace.startsolid = 1;
                        trace.fraction = 0.0;
                        v1[0] = 0.0;
                        v1[1] = 0.0;
                        v1[2] = 0.0;
                    } //end if
                    else {
                        trace.startsolid = 0;
                        v1[0] = end[0] - start[0];
                        v1[1] = end[1] - start[1];
                        v1[2] = end[2] - start[2];
                        v2[0] = (*tstack_p).start[0] - start[0];
                        v2[1] = (*tstack_p).start[1] - start[1];
                        v2[2] = (*tstack_p).start[2] - start[2];
                        let len_v2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
                        let v1_orig = v1.clone();
                        let len_v1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
                        let norm = len_v1;
                        if norm > 0.0 {
                            v1[0] /= norm;
                            v1[1] /= norm;
                            v1[2] /= norm;
                        }
                        trace.fraction = len_v2 / norm;
                        (*tstack_p).start[0] = (*tstack_p).start[0] + (-0.125) * v1_orig[0];
                        (*tstack_p).start[1] = (*tstack_p).start[1] + (-0.125) * v1_orig[1];
                        (*tstack_p).start[2] = (*tstack_p).start[2] + (-0.125) * v1_orig[2];
                    } //end else
                    trace.endpos[0] = (*tstack_p).start[0];
                    trace.endpos[1] = (*tstack_p).start[1];
                    trace.endpos[2] = (*tstack_p).start[2];
                    trace.ent = 0;
                    trace.area = -nodenum;
    //				VectorSubtract(end, start, v1);
                    trace.planenum = (*tstack_p).planenum;
                    //always take the plane with normal facing towards the trace start
                    plane = aasworld.planes.offset(trace.planenum as isize);
                    v1[0] = end[0] - start[0];
                    v1[1] = end[1] - start[1];
                    v1[2] = end[2] - start[2];
                    if (v1[0] * (*plane).normal[0] + v1[1] * (*plane).normal[1] + v1[2] * (*plane).normal[2]) > 0.0 {
                        trace.planenum ^= 1;
                    }
                    return trace;
                } //end if
                else {
                    if passent >= 0 {
                        if AAS_AreaEntityCollision(-nodenum, &(*tstack_p).start,
                                        &(*tstack_p).end, presencetype, passent,
                                        &mut trace) != 0 {
                            if trace.startsolid == 0 {
                                v1[0] = end[0] - start[0];
                                v1[1] = end[1] - start[1];
                                v1[2] = end[2] - start[2];
                                v2[0] = trace.endpos[0] - start[0];
                                v2[1] = trace.endpos[1] - start[1];
                                v2[2] = trace.endpos[2] - start[2];
                                let len_v2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
                                let len_v1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
                                trace.fraction = len_v2 / len_v1;
                            } //end if
                            return trace;
                        } //end if
                    } //end if
                } //end else
                trace.lastarea = -nodenum;
                continue;
            } //end if
            //if it is a solid leaf
            if nodenum == 0 {
                //if the start point is still the initial start point
                //NOTE: no need for epsilons because the points will be
                //exactly the same when they're both the start point
                if (*tstack_p).start[0] == start[0] &&
                        (*tstack_p).start[1] == start[1] &&
                        (*tstack_p).start[2] == start[2] {
                    trace.startsolid = 1;
                    trace.fraction = 0.0;
                    v1[0] = 0.0;
                    v1[1] = 0.0;
                    v1[2] = 0.0;
                } //end if
                else {
                    trace.startsolid = 0;
                    v1[0] = end[0] - start[0];
                    v1[1] = end[1] - start[1];
                    v1[2] = end[2] - start[2];
                    v2[0] = (*tstack_p).start[0] - start[0];
                    v2[1] = (*tstack_p).start[1] - start[1];
                    v2[2] = (*tstack_p).start[2] - start[2];
                    let len_v2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
                    let v1_orig = v1.clone();
                    let len_v1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
                    let norm = len_v1;
                    if norm > 0.0 {
                        v1[0] /= norm;
                        v1[1] /= norm;
                        v1[2] /= norm;
                    }
                    trace.fraction = len_v2 / norm;
                    (*tstack_p).start[0] = (*tstack_p).start[0] + (-0.125) * v1_orig[0];
                    (*tstack_p).start[1] = (*tstack_p).start[1] + (-0.125) * v1_orig[1];
                    (*tstack_p).start[2] = (*tstack_p).start[2] + (-0.125) * v1_orig[2];
                } //end else
                trace.endpos[0] = (*tstack_p).start[0];
                trace.endpos[1] = (*tstack_p).start[1];
                trace.endpos[2] = (*tstack_p).start[2];
                trace.ent = 0;
                trace.area = 0;	//hit solid leaf
    //			VectorSubtract(end, start, v1);
                trace.planenum = (*tstack_p).planenum;
                //always take the plane with normal facing towards the trace start
                plane = aasworld.planes.offset(trace.planenum as isize);
                v1[0] = end[0] - start[0];
                v1[1] = end[1] - start[1];
                v1[2] = end[2] - start[2];
                if (v1[0] * (*plane).normal[0] + v1[1] * (*plane).normal[1] + v1[2] * (*plane).normal[2]) > 0.0 {
                    trace.planenum ^= 1;
                }
                return trace;
            } //end if
    #[cfg(feature = "aas_sample_debug")]
            {
                if nodenum > aasworld.numnodes {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceBoundingBox: nodenum out of range\n\0".as_ptr() as *const c_char);
                    return trace;
                } //end if
            }
            //the node to test against
            aasnode = aasworld.nodes.offset(nodenum as isize);
            //start point of current line to test against node
            cur_start[0] = (*tstack_p).start[0];
            cur_start[1] = (*tstack_p).start[1];
            cur_start[2] = (*tstack_p).start[2];
            //end point of the current line to test against node
            cur_end[0] = (*tstack_p).end[0];
            cur_end[1] = (*tstack_p).end[1];
            cur_end[2] = (*tstack_p).end[2];
            //the current node plane
            plane = aasworld.planes.offset((*aasnode).planenum as isize);

    //		switch(plane->type)
            {/*FIXME: wtf doesn't this work? obviously the axial node planes aren't always facing positive!!!
                //check for axial planes
                case PLANE_X:
                {
                    front = cur_start[0] - plane->dist;
                    back = cur_end[0] - plane->dist;
                    break;
                } //end case
                case PLANE_Y:
                {
                    front = cur_start[1] - plane->dist;
                    back = cur_end[1] - plane->dist;
                    break;
                } //end case
                case PLANE_Z:
                {
                    front = cur_start[2] - plane->dist;
                    back = cur_end[2] - plane->dist;
                    break;
                } //end case*/
    //			default: //gee it's not an axial plane
                {
                    front = cur_start[0] * (*plane).normal[0] + cur_start[1] * (*plane).normal[1] + cur_start[2] * (*plane).normal[2] - (*plane).dist;
                    back = cur_end[0] * (*plane).normal[0] + cur_end[1] * (*plane).normal[1] + cur_end[2] * (*plane).normal[2] - (*plane).dist;
    //				break;
                } //end default
            } //end switch
            // bk010221 - old location of FPE hack and divide by zero expression
            //if the whole to be traced line is totally at the front of this node
            //only go down the tree with the front child
            if (front >= -ON_EPSILON && back >= -ON_EPSILON) {
                //keep the current start and end point on the stack
                //and go down the tree with the front child
                (*tstack_p).nodenum = (*aasnode).children[0];
                tstack_p = tstack_p.offset(1);
                if tstack_p >= tracestack.as_mut_ptr().offset(127) {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceBoundingBox: stack overflow\n\0".as_ptr() as *const c_char);
                    return trace;
                } //end if
            } //end if
            //if the whole to be traced line is totally at the back of this node
            //only go down the tree with the back child
            else if (front < ON_EPSILON && back < ON_EPSILON) {
                //keep the current start and end point on the stack
                //and go down the tree with the back child
                (*tstack_p).nodenum = (*aasnode).children[1];
                tstack_p = tstack_p.offset(1);
                if tstack_p >= tracestack.as_mut_ptr().offset(127) {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceBoundingBox: stack overflow\n\0".as_ptr() as *const c_char);
                    return trace;
                } //end if
            } //end if
            //go down the tree both at the front and back of the node
            else {
                tmpplanenum = (*tstack_p).planenum;
                // bk010221 - new location of divide by zero (see above)
                if front == back { front -= 0.001; } // bk0101022 - hack/FPE
                            //calculate the hitpoint with the node (split point of the line)
                //put the crosspoint TRACEPLANE_EPSILON pixels on the near side
                if front < 0.0 { frac = (front + TRACEPLANE_EPSILON)/(front-back); }
                else { frac = (front - TRACEPLANE_EPSILON)/(front-back); } // bk010221
                //
                if frac < 0.0 {
                    frac = 0.001; //0
                } else if frac > 1.0 {
                    frac = 0.999; //1
                }
                //frac = front / (front-back);
                //
                cur_mid[0] = cur_start[0] + (cur_end[0] - cur_start[0]) * frac;
                cur_mid[1] = cur_start[1] + (cur_end[1] - cur_start[1]) * frac;
                cur_mid[2] = cur_start[2] + (cur_end[2] - cur_start[2]) * frac;

    //			AAS_DrawPlaneCross(cur_mid, plane->normal, plane->dist, plane->type, LINECOLOR_RED);
                //side the front part of the line is on
                side = if front < 0.0 { 1 } else { 0 };
                //first put the end part of the line on the stack (back side)
                (*tstack_p).start[0] = cur_mid[0];
                (*tstack_p).start[1] = cur_mid[1];
                (*tstack_p).start[2] = cur_mid[2];
                //not necesary to store because still on stack
                //VectorCopy(cur_end, tstack_p->end);
                (*tstack_p).planenum = (*aasnode).planenum;
                (*tstack_p).nodenum = (*aasnode).children[if side != 0 { 0 } else { 1 }];
                tstack_p = tstack_p.offset(1);
                if tstack_p >= tracestack.as_mut_ptr().offset(127) {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceBoundingBox: stack overflow\n\0".as_ptr() as *const c_char);
                    return trace;
                } //end if
                //now put the part near the start of the line on the stack so we will
                //continue with thats part first. This way we'll find the first
                //hit of the bbox
                (*tstack_p).start[0] = cur_start[0];
                (*tstack_p).start[1] = cur_start[1];
                (*tstack_p).start[2] = cur_start[2];
                (*tstack_p).end[0] = cur_mid[0];
                (*tstack_p).end[1] = cur_mid[1];
                (*tstack_p).end[2] = cur_mid[2];
                (*tstack_p).planenum = tmpplanenum;
                (*tstack_p).nodenum = (*aasnode).children[side as usize];
                tstack_p = tstack_p.offset(1);
                if tstack_p >= tracestack.as_mut_ptr().offset(127) {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceBoundingBox: stack overflow\n\0".as_ptr() as *const c_char);
                    return trace;
                } //end if
            } //end else
        } //end while
    }
//	return trace;
} //end of the function AAS_TraceClientBBox
//===========================================================================
// recursive subdivision of the line by the BSP tree.
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_TraceAreas(start: &vec3_t, end: &vec3_t, areas: *mut c_int, points: *mut vec3_t, maxareas: c_int) -> c_int
{
    let mut side: c_int;
    let mut nodenum: c_int;
    let mut tmpplanenum: c_int;
    let mut numareas: c_int;
    let mut front: c_float;
    let mut back: c_float;
    let mut frac: c_float;
    let mut cur_start: vec3_t = [0.0; 3];
    let mut cur_end: vec3_t = [0.0; 3];
    let mut cur_mid: vec3_t = [0.0; 3];
    let mut tracestack: [aas_tracestack_t; 127] = unsafe { core::mem::zeroed() };
    let mut tstack_p: *mut aas_tracestack_t;
    let aasnode: *mut aas_node_t;
    let plane: *mut aas_plane_t;

    numareas = 0;
    unsafe {
        *areas.offset(0) = 0;
        if aasworld.loaded == 0 { return numareas; }

        tstack_p = tracestack.as_mut_ptr();
        //we start with the whole line on the stack
        (*tstack_p).start[0] = start[0];
        (*tstack_p).start[1] = start[1];
        (*tstack_p).start[2] = start[2];
        (*tstack_p).end[0] = end[0];
        (*tstack_p).end[1] = end[1];
        (*tstack_p).end[2] = end[2];
        (*tstack_p).planenum = 0;
        //start with node 1 because node zero is a dummy for a solid leaf
        (*tstack_p).nodenum = 1;		//starting at the root of the tree
        tstack_p = tstack_p.offset(1);

        loop {
            //pop up the stack
            tstack_p = tstack_p.offset(-1);
            //if the trace stack is empty (ended up with a piece of the
            //line to be traced in an area)
            if tstack_p < tracestack.as_mut_ptr() {
                return numareas;
            } //end if
            //number of the current node to test the line against
            nodenum = (*tstack_p).nodenum;
            //if it is an area
            if nodenum < 0 {
    #[cfg(feature = "aas_sample_debug")]
                {
                    if -nodenum > aasworld.numareasettings {
                        (botimport.Print)(PRT_ERROR, b"AAS_TraceAreas: -nodenum = %d out of range\n\0".as_ptr() as *const c_char, -nodenum);
                        return numareas;
                    } //end if
                }
                //botimport.Print(PRT_MESSAGE, "areanum = %d, must be %d\n", -nodenum, AAS_PointAreaNum(start));
                *areas.offset(numareas as isize) = -nodenum;
                if !points.is_null() {
                    (*points.offset(numareas as isize))[0] = (*tstack_p).start[0];
                    (*points.offset(numareas as isize))[1] = (*tstack_p).start[1];
                    (*points.offset(numareas as isize))[2] = (*tstack_p).start[2];
                }
                numareas += 1;
                if numareas >= maxareas { return numareas; }
                continue;
            } //end if
            //if it is a solid leaf
            if nodenum == 0 {
                continue;
            } //end if
    #[cfg(feature = "aas_sample_debug")]
            {
                if nodenum > aasworld.numnodes {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceAreas: nodenum out of range\n\0".as_ptr() as *const c_char);
                    return numareas;
                } //end if
            }
            //the node to test against
            aasnode = aasworld.nodes.offset(nodenum as isize);
            //start point of current line to test against node
            cur_start[0] = (*tstack_p).start[0];
            cur_start[1] = (*tstack_p).start[1];
            cur_start[2] = (*tstack_p).start[2];
            //end point of the current line to test against node
            cur_end[0] = (*tstack_p).end[0];
            cur_end[1] = (*tstack_p).end[1];
            cur_end[2] = (*tstack_p).end[2];
            //the current node plane
            plane = aasworld.planes.offset((*aasnode).planenum as isize);

    //		switch(plane->type)
            {/*FIXME: wtf doesn't this work? obviously the node planes aren't always facing positive!!!
                //check for axial planes
                case PLANE_X:
                {
                    front = cur_start[0] - plane->dist;
                    back = cur_end[0] - plane->dist;
                    break;
                } //end case
                case PLANE_Y:
                {
                    front = cur_start[1] - plane->dist;
                    back = cur_end[1] - plane->dist;
                    break;
                } //end case
                case PLANE_Z:
                {
                    front = cur_start[2] - plane->dist;
                    back = cur_end[2] - plane->dist;
                    break;
                } //end case*/
    //			default: //gee it's not an axial plane
                {
                    front = cur_start[0] * (*plane).normal[0] + cur_start[1] * (*plane).normal[1] + cur_start[2] * (*plane).normal[2] - (*plane).dist;
                    back = cur_end[0] * (*plane).normal[0] + cur_end[1] * (*plane).normal[1] + cur_end[2] * (*plane).normal[2] - (*plane).dist;
    //				break;
                } //end default
            } //end switch

            //if the whole to be traced line is totally at the front of this node
            //only go down the tree with the front child
            if front > 0.0 && back > 0.0 {
                //keep the current start and end point on the stack
                //and go down the tree with the front child
                (*tstack_p).nodenum = (*aasnode).children[0];
                tstack_p = tstack_p.offset(1);
                if tstack_p >= tracestack.as_mut_ptr().offset(127) {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceAreas: stack overflow\n\0".as_ptr() as *const c_char);
                    return numareas;
                } //end if
            } //end if
            //if the whole to be traced line is totally at the back of this node
            //only go down the tree with the back child
            else if front <= 0.0 && back <= 0.0 {
                //keep the current start and end point on the stack
                //and go down the tree with the back child
                (*tstack_p).nodenum = (*aasnode).children[1];
                tstack_p = tstack_p.offset(1);
                if tstack_p >= tracestack.as_mut_ptr().offset(127) {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceAreas: stack overflow\n\0".as_ptr() as *const c_char);
                    return numareas;
                } //end if
            } //end if
            //go down the tree both at the front and back of the node
            else {
                tmpplanenum = (*tstack_p).planenum;
                //calculate the hitpoint with the node (split point of the line)
                //put the crosspoint TRACEPLANE_EPSILON pixels on the near side
                if front < 0.0 { frac = front/(front-back); }
                else { frac = front/(front-back); }
                if frac < 0.0 { frac = 0.0; }
                else if frac > 1.0 { frac = 1.0; }
                //frac = front / (front-back);
                //
                cur_mid[0] = cur_start[0] + (cur_end[0] - cur_start[0]) * frac;
                cur_mid[1] = cur_start[1] + (cur_end[1] - cur_start[1]) * frac;
                cur_mid[2] = cur_start[2] + (cur_end[2] - cur_start[2]) * frac;

    //			AAS_DrawPlaneCross(cur_mid, plane->normal, plane->dist, plane->type, LINECOLOR_RED);
                //side the front part of the line is on
                side = if front < 0.0 { 1 } else { 0 };
                //first put the end part of the line on the stack (back side)
                (*tstack_p).start[0] = cur_mid[0];
                (*tstack_p).start[1] = cur_mid[1];
                (*tstack_p).start[2] = cur_mid[2];
                //not necesary to store because still on stack
                //VectorCopy(cur_end, tstack_p->end);
                (*tstack_p).planenum = (*aasnode).planenum;
                (*tstack_p).nodenum = (*aasnode).children[if side != 0 { 0 } else { 1 }];
                tstack_p = tstack_p.offset(1);
                if tstack_p >= tracestack.as_mut_ptr().offset(127) {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceAreas: stack overflow\n\0".as_ptr() as *const c_char);
                    return numareas;
                } //end if
                //now put the part near the start of the line on the stack so we will
                //continue with thats part first. This way we'll find the first
                //hit of the bbox
                (*tstack_p).start[0] = cur_start[0];
                (*tstack_p).start[1] = cur_start[1];
                (*tstack_p).start[2] = cur_start[2];
                (*tstack_p).end[0] = cur_mid[0];
                (*tstack_p).end[1] = cur_mid[1];
                (*tstack_p).end[2] = cur_mid[2];
                (*tstack_p).planenum = tmpplanenum;
                (*tstack_p).nodenum = (*aasnode).children[side as usize];
                tstack_p = tstack_p.offset(1);
                if tstack_p >= tracestack.as_mut_ptr().offset(127) {
                    (botimport.Print)(PRT_ERROR, b"AAS_TraceAreas: stack overflow\n\0".as_ptr() as *const c_char);
                    return numareas;
                } //end if
            } //end else
        } //end while
    }
//	return numareas;
} //end of the function AAS_TraceAreas
//===========================================================================
// a simple cross product
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
// void AAS_OrthogonalToVectors(vec3_t v1, vec3_t v2, vec3_t res)
#[inline]
pub fn AAS_OrthogonalToVectors(v1: &vec3_t, v2: &vec3_t, res: &mut vec3_t) {
    res[0] = ((v1[1] * v2[2]) - (v1[2] * v2[1]));
    res[1] = ((v1[2] * v2[0]) - (v1[0] * v2[2]));
    res[2] = ((v1[0] * v2[1]) - (v1[1] * v2[0]));
}
//===========================================================================
// tests if the given point is within the face boundaries
//
// Parameter:				face		: face to test if the point is in it
//								pnormal	: normal of the plane to use for the face
//								point		: point to test if inside face boundaries
// Returns:					qtrue if the point is within the face boundaries
// Changes Globals:		-
//===========================================================================
pub fn AAS_InsideFace(face: *mut aas_face_t, pnormal: &vec3_t, point: &vec3_t, epsilon: c_float) -> c_int
{
    let mut i: c_int;
    let mut firstvertex: c_int;
    let mut edgenum: c_int;
    let mut v0: vec3_t = [0.0; 3];
    let mut edgevec: vec3_t = [0.0; 3];
    let mut pointvec: vec3_t = [0.0; 3];
    let mut sepnormal: vec3_t = [0.0; 3];
    let edge: *mut aas_edge_t;
    #[cfg(feature = "aas_sample_debug")]
    let mut lastvertex: c_int = 0;

    unsafe {
        if aasworld.loaded == 0 { return 0; }

        i = 0;
        while i < (*face).numedges {
            edgenum = *aasworld.edgeindex.offset(((*face).firstedge + i) as isize);
            edge = aasworld.edges.offset(edgenum.abs() as isize);
            //get the first vertex of the edge
            firstvertex = if edgenum < 0 { 1 } else { 0 };
            v0[0] = *aasworld.vertexes.offset(((*edge).v[firstvertex as usize]) as isize).offset(0);
            v0[1] = *aasworld.vertexes.offset(((*edge).v[firstvertex as usize]) as isize).offset(1);
            v0[2] = *aasworld.vertexes.offset(((*edge).v[firstvertex as usize]) as isize).offset(2);
            //edge vector
            let v_notfirst = aasworld.vertexes.offset(((*edge).v[(1-firstvertex) as usize]) as isize);
            edgevec[0] = *v_notfirst.offset(0) - v0[0];
            edgevec[1] = *v_notfirst.offset(1) - v0[1];
            edgevec[2] = *v_notfirst.offset(2) - v0[2];
            //
    #[cfg(feature = "aas_sample_debug")]
            {
                if lastvertex != 0 && lastvertex != (*edge).v[firstvertex as usize] {
                    (botimport.Print)(PRT_MESSAGE, b"winding not counter clockwise\n\0".as_ptr() as *const c_char);
                } //end if
                lastvertex = (*edge).v[(1-firstvertex) as usize];
            }
            //vector from first edge point to point possible in face
            pointvec[0] = point[0] - v0[0];
            pointvec[1] = point[1] - v0[1];
            pointvec[2] = point[2] - v0[2];
            //get a vector pointing inside the face orthogonal to both the
            //edge vector and the normal vector of the plane the face is in
            //this vector defines a plane through the origin (first vertex of
            //edge) and through both the edge vector and the normal vector
            //of the plane
            AAS_OrthogonalToVectors(&edgevec, pnormal, &mut sepnormal);
            //check on wich side of the above plane the point is
            //this is done by checking the sign of the dot product of the
            //vector orthogonal vector from above and the vector from the
            //origin (first vertex of edge) to the point
            //if the dotproduct is smaller than zero the point is outside the face
            if (pointvec[0] * sepnormal[0] + pointvec[1] * sepnormal[1] + pointvec[2] * sepnormal[2]) < -epsilon {
                return 0;
            }
            i += 1;
        } //end for
    }
    1
} //end of the function AAS_InsideFace
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_PointInsideFace(facenum: c_int, point: &vec3_t, epsilon: c_float) -> c_int
{
    let mut i: c_int;
    let mut firstvertex: c_int;
    let mut edgenum: c_int;
    let v1: *mut c_float;
    let v2: *mut c_float;
    let mut edgevec: vec3_t = [0.0; 3];
    let mut pointvec: vec3_t = [0.0; 3];
    let mut sepnormal: vec3_t = [0.0; 3];
    let edge: *mut aas_edge_t;
    let plane: *mut aas_plane_t;
    let face: *mut aas_face_t;

    unsafe {
        if aasworld.loaded == 0 { return 0; }

        face = aasworld.faces.offset(facenum as isize);
        plane = aasworld.planes.offset((*face).planenum as isize);
        //
        i = 0;
        while i < (*face).numedges {
            edgenum = *aasworld.edgeindex.offset(((*face).firstedge + i) as isize);
            edge = aasworld.edges.offset(edgenum.abs() as isize);
            //get the first vertex of the edge
            firstvertex = if edgenum < 0 { 1 } else { 0 };
            v1 = aasworld.vertexes.offset(((*edge).v[firstvertex as usize]) as isize);
            v2 = aasworld.vertexes.offset(((*edge).v[(1-firstvertex) as usize]) as isize);
            //edge vector
            edgevec[0] = *v2.offset(0) - *v1.offset(0);
            edgevec[1] = *v2.offset(1) - *v1.offset(1);
            edgevec[2] = *v2.offset(2) - *v1.offset(2);
            //vector from first edge point to point possible in face
            pointvec[0] = point[0] - *v1.offset(0);
            pointvec[1] = point[1] - *v1.offset(1);
            pointvec[2] = point[2] - *v1.offset(2);
            //
            sepnormal[0] = edgevec[1] * (*plane).normal[2] - edgevec[2] * (*plane).normal[1];
            sepnormal[1] = edgevec[2] * (*plane).normal[0] - edgevec[0] * (*plane).normal[2];
            sepnormal[2] = edgevec[0] * (*plane).normal[1] - edgevec[1] * (*plane).normal[0];
            //
            if (pointvec[0] * sepnormal[0] + pointvec[1] * sepnormal[1] + pointvec[2] * sepnormal[2]) < -epsilon {
                return 0;
            }
            i += 1;
        } //end for
    }
    1
} //end of the function AAS_PointInsideFace
//===========================================================================
// returns the ground face the given point is above in the given area
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_AreaGroundFace(areanum: c_int, point: &vec3_t) -> *mut aas_face_t
{
    let mut i: c_int;
    let mut facenum: c_int;
    let up: vec3_t = [0.0, 0.0, 1.0];
    let mut normal: vec3_t = [0.0; 3];
    let area: *mut aas_area_t;
    let face: *mut aas_face_t;

    unsafe {
        if aasworld.loaded == 0 { return core::ptr::null_mut(); }

        area = aasworld.areas.offset(areanum as isize);
        i = 0;
        while i < (*area).numfaces {
            facenum = *aasworld.faceindex.offset(((*area).firstface + i) as isize);
            face = aasworld.faces.offset(facenum.abs() as isize);
            //if this is a ground face
            if ((*face).faceflags & FACE_GROUND) != 0 {
                //get the up or down normal
                if (*aasworld.planes.offset((*face).planenum as isize)).normal[2] < 0.0 {
                    normal[0] = -up[0];
                    normal[1] = -up[1];
                    normal[2] = -up[2];
                } else {
                    normal[0] = up[0];
                    normal[1] = up[1];
                    normal[2] = up[2];
                }
                //check if the point is in the face
                if AAS_InsideFace(face, &normal, point, 0.01) != 0 { return face; }
            } //end if
            i += 1;
        } //end for
    }
    core::ptr::null_mut()
} //end of the function AAS_AreaGroundFace
//===========================================================================
// returns the face the trace end position is situated in
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_FacePlane(facenum: c_int, normal: &mut vec3_t, dist: &mut c_float)
{
    let plane: *mut aas_plane_t;

    unsafe {
        plane = aasworld.planes.offset((*aasworld.faces.offset(facenum as isize)).planenum as isize);
        normal[0] = (*plane).normal[0];
        normal[1] = (*plane).normal[1];
        normal[2] = (*plane).normal[2];
        *dist = (*plane).dist;
    }
} //end of the function AAS_FacePlane
//===========================================================================
// returns the face the trace end position is situated in
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_TraceEndFace(trace: *mut aas_trace_t) -> *mut aas_face_t
{
    let mut i: c_int;
    let mut facenum: c_int;
    let area: *mut aas_area_t;
    let face: *mut aas_face_t;
    let mut firstface: *mut aas_face_t = core::ptr::null_mut();

    unsafe {
        if aasworld.loaded == 0 { return core::ptr::null_mut(); }

        //if started in solid no face was hit
        if (*trace).startsolid != 0 { return core::ptr::null_mut(); }
        //trace->lastarea is the last area the trace was in
        area = aasworld.areas.offset((*trace).lastarea as isize);
        //check which face the trace.endpos was in
        i = 0;
        while i < (*area).numfaces {
            facenum = *aasworld.faceindex.offset(((*area).firstface + i) as isize);
            face = aasworld.faces.offset(facenum.abs() as isize);
            //if the face is in the same plane as the trace end point
            if ((*face).planenum & !1) == ((*trace).planenum & !1) {
                //firstface is used for optimization, if theres only one
                //face in the plane then it has to be the good one
                //if there are more faces in the same plane then always
                //check the one with the fewest edges first
/*			if (firstface)
            {
                if (firstface->numedges < face->numedges)
                {
                    if (AAS_InsideFace(firstface,
                        aasworld.planes[face->planenum].normal, trace->endpos))
                    {
                        return firstface;
                    } //end if
                    firstface = face;
                } //end if
                else
                {
                    if (AAS_InsideFace(face,
                        aasworld.planes[face->planenum].normal, trace->endpos))
                    {
                        return face;
                    } //end if
                } //end else
            } //end if
            else
            {
                firstface = face;
            } //end else*/
                if AAS_InsideFace(face,
                            &(*aasworld.planes.offset((*face).planenum as isize)).normal, &(*trace).endpos, 0.01) != 0 {
                    return face;
                }
            } //end if
            i += 1;
        } //end for
    }
    firstface
} //end of the function AAS_TraceEndFace
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_BoxOnPlaneSide2(absmins: &vec3_t, absmaxs: &vec3_t, p: *mut aas_plane_t) -> c_int
{
    let mut i: c_int;
    let mut sides: c_int;
    let mut dist1: c_float;
    let mut dist2: c_float;
    let mut corners: [[c_float; 3]; 2] = [[0.0; 3]; 2];

    unsafe {
        i = 0;
        while i < 3 {
            if (*p).normal[i as usize] < 0.0 {
                corners[0][i as usize] = absmins[i as usize];
                corners[1][i as usize] = absmaxs[i as usize];
            } //end if
            else {
                corners[1][i as usize] = absmins[i as usize];
                corners[0][i as usize] = absmaxs[i as usize];
            } //end else
            i += 1;
        } //end for
    }
    dist1 = corners[0][0] * unsafe { (*p).normal[0] } + corners[0][1] * unsafe { (*p).normal[1] } + corners[0][2] * unsafe { (*p).normal[2] } - unsafe { (*p).dist };
    dist2 = corners[1][0] * unsafe { (*p).normal[0] } + corners[1][1] * unsafe { (*p).normal[1] } + corners[1][2] * unsafe { (*p).normal[2] } - unsafe { (*p).dist };
    sides = 0;
    if dist1 >= 0.0 { sides = 1; }
    if dist2 < 0.0 { sides |= 2; }

    sides
} //end of the function AAS_BoxOnPlaneSide2
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
//int AAS_BoxOnPlaneSide(vec3_t absmins, vec3_t absmaxs, aas_plane_t *p)
#[inline]
pub fn AAS_BoxOnPlaneSide(absmins: &vec3_t, absmaxs: &vec3_t, p: *mut aas_plane_t) -> c_int {
    unsafe {
        if (*p).type_ < 3 {
            if (*p).dist <= absmins[(*p).type_ as usize] {
                1
            } else if (*p).dist >= absmaxs[(*p).type_ as usize] {
                2
            } else {
                3
            }
        } else {
            AAS_BoxOnPlaneSide2(absmins, absmaxs, p)
        }
    }
} //end of the function AAS_BoxOnPlaneSide
//===========================================================================
// remove the links to this entity from all areas
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_UnlinkFromAreas(areas: *mut aas_link_t)
{
    let mut link: *mut aas_link_t;
    let mut nextlink: *mut aas_link_t;

    unsafe {
        link = areas;
        while !link.is_null() {
            //next area the entity is linked in
            nextlink = (*link).next_area;
            //remove the entity from the linked list of this area
            if !(*link).prev_ent.is_null() {
                (*(*link).prev_ent).next_ent = (*link).next_ent;
            } else {
                *aasworld.arealinkedentities.offset((*link).areanum as isize) = (*link).next_ent;
            }
            if !(*link).next_ent.is_null() {
                (*(*link).next_ent).prev_ent = (*link).prev_ent;
            }
            //deallocate the link structure
            AAS_DeAllocAASLink(link);
            link = nextlink;
        } //end for
    }
} //end of the function AAS_UnlinkFromAreas
//===========================================================================
// link the entity to the areas the bounding box is totally or partly
// situated in. This is done with recursion down the tree using the
// bounding box to test for plane sides
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================

pub fn AAS_AASLinkEntity(absmins: &vec3_t, absmaxs: &vec3_t, entnum: c_int) -> *mut aas_link_t
{
    let mut side: c_int;
    let mut nodenum: c_int;
    let mut linkstack: [aas_linkstack_t; 128] = unsafe { core::mem::zeroed() };
    let mut lstack_p: *mut aas_linkstack_t;
    let aasnode: *mut aas_node_t;
    let plane: *mut aas_plane_t;
    let link: *mut aas_link_t;
    let mut areas: *mut aas_link_t = core::ptr::null_mut();

    unsafe {
        if aasworld.loaded == 0 {
            (botimport.Print)(PRT_ERROR, b"AAS_LinkEntity: aas not loaded\n\0".as_ptr() as *const c_char);
            return core::ptr::null_mut();
        } //end if

        areas = core::ptr::null_mut();
        //
        lstack_p = linkstack.as_mut_ptr();
        //we start with the whole line on the stack
        //start with node 1 because node zero is a dummy used for solid leafs
        (*lstack_p).nodenum = 1;		//starting at the root of the tree
        lstack_p = lstack_p.offset(1);

        loop {
            //pop up the stack
            lstack_p = lstack_p.offset(-1);
            //if the trace stack is empty (ended up with a piece of the
            //line to be traced in an area)
            if lstack_p < linkstack.as_mut_ptr() { break; }
            //number of the current node to test the line against
            nodenum = (*lstack_p).nodenum;
            //if it is an area
            if nodenum < 0 {
                //NOTE: the entity might have already been linked into this area
                // because several node children can point to the same area
                link = *aasworld.arealinkedentities.offset((-nodenum) as isize);
                loop {
                    if link.is_null() { break; }
                    if (*link).entnum == entnum { break; }
                    link = (*link).next_ent;
                } //end for
                if !link.is_null() { continue; }
                //
                link = AAS_AllocAASLink();
                if link.is_null() { return areas; }
                (*link).entnum = entnum;
                (*link).areanum = -nodenum;
                //put the link into the double linked area list of the entity
                (*link).prev_area = core::ptr::null_mut();
                (*link).next_area = areas;
                if !areas.is_null() { (*areas).prev_area = link; }
                areas = link;
                //put the link into the double linked entity list of the area
                (*link).prev_ent = core::ptr::null_mut();
                (*link).next_ent = *aasworld.arealinkedentities.offset((-nodenum) as isize);
                if !(*aasworld.arealinkedentities.offset((-nodenum) as isize)).is_null() {
                        (*(*aasworld.arealinkedentities.offset((-nodenum) as isize))).prev_ent = link;
                }
                *aasworld.arealinkedentities.offset((-nodenum) as isize) = link;
                //
                continue;
            } //end if
            //if solid leaf
            if nodenum == 0 { continue; }
            //the node to test against
            aasnode = aasworld.nodes.offset(nodenum as isize);
            //the current node plane
            plane = aasworld.planes.offset((*aasnode).planenum as isize);
            //get the side(s) the box is situated relative to the plane
            side = AAS_BoxOnPlaneSide2(absmins, absmaxs, plane);
            //if on the front side of the node
            if (side & 1) != 0 {
                (*lstack_p).nodenum = (*aasnode).children[0];
                lstack_p = lstack_p.offset(1);
            } //end if
            if lstack_p >= linkstack.as_mut_ptr().offset(127) {
                (botimport.Print)(PRT_ERROR, b"AAS_LinkEntity: stack overflow\n\0".as_ptr() as *const c_char);
                break;
            } //end if
            //if on the back side of the node
            if (side & 2) != 0 {
                (*lstack_p).nodenum = (*aasnode).children[1];
                lstack_p = lstack_p.offset(1);
            } //end if
            if lstack_p >= linkstack.as_mut_ptr().offset(127) {
                (botimport.Print)(PRT_ERROR, b"AAS_LinkEntity: stack overflow\n\0".as_ptr() as *const c_char);
                break;
            } //end if
        } //end while
    }
    areas
} //end of the function AAS_AASLinkEntity
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_LinkEntityClientBBox(absmins: &vec3_t, absmaxs: &vec3_t, entnum: c_int, presencetype: c_int) -> *mut aas_link_t
{
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut newabsmins: vec3_t = [0.0; 3];
    let mut newabsmaxs: vec3_t = [0.0; 3];

    AAS_PresenceTypeBoundingBox(presencetype, &mut mins, &mut maxs);
    newabsmins[0] = absmins[0] - maxs[0];
    newabsmins[1] = absmins[1] - maxs[1];
    newabsmins[2] = absmins[2] - maxs[2];
    newabsmaxs[0] = absmaxs[0] - mins[0];
    newabsmaxs[1] = absmaxs[1] - mins[1];
    newabsmaxs[2] = absmaxs[2] - mins[2];
    //relink the entity
    AAS_AASLinkEntity(&newabsmins, &newabsmaxs, entnum)
} //end of the function AAS_LinkEntityClientBBox
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_BBoxAreas(absmins: &vec3_t, absmaxs: &vec3_t, areas: *mut c_int, maxareas: c_int) -> c_int
{
    let linkedareas: *mut aas_link_t;
    let link: *mut aas_link_t;
    let mut num: c_int;

    linkedareas = AAS_AASLinkEntity(absmins, absmaxs, -1);
    num = 0;
    unsafe {
        link = linkedareas;
        while !link.is_null() {
            *areas.offset(num as isize) = (*link).areanum;
            num += 1;
            if num >= maxareas {
                break;
            }
            link = (*link).next_area;
        } //end for
    }
    AAS_UnlinkFromAreas(linkedareas);
    num
} //end of the function AAS_BBoxAreas
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_AreaInfo( areanum: c_int, info: *mut aas_areainfo_t ) -> c_int
{
    let settings: *mut aas_areasettings_t;
    unsafe {
        if info.is_null() {
            return 0;
        }
        if areanum <= 0 || areanum >= aasworld.numareas {
            (botimport.Print)(PRT_ERROR, b"AAS_AreaInfo: areanum %d out of range\n\0".as_ptr() as *const c_char, areanum);
            return 0;
        } //end if
        settings = aasworld.areasettings.offset(areanum as isize);
        (*info).cluster = (*settings).cluster;
        (*info).contents = (*settings).contents;
        (*info).flags = (*settings).areaflags;
        (*info).presencetype = (*settings).presencetype;
        let area_mins = &(*aasworld.areas.offset(areanum as isize)).mins;
        let area_maxs = &(*aasworld.areas.offset(areanum as isize)).maxs;
        let area_center = &(*aasworld.areas.offset(areanum as isize)).center;
        (*info).mins[0] = area_mins[0];
        (*info).mins[1] = area_mins[1];
        (*info).mins[2] = area_mins[2];
        (*info).maxs[0] = area_maxs[0];
        (*info).maxs[1] = area_maxs[1];
        (*info).maxs[2] = area_maxs[2];
        (*info).center[0] = area_center[0];
        (*info).center[1] = area_center[1];
        (*info).center[2] = area_center[2];
        core::mem::size_of::<aas_areainfo_t>() as c_int
    }
} //end of the function AAS_AreaInfo
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn AAS_PlaneFromNum(planenum: c_int) -> *mut aas_plane_t
{
    unsafe {
        if aasworld.loaded == 0 { return core::ptr::null_mut(); }

        aasworld.planes.offset(planenum as isize)
    }
} //end of the function AAS_PlaneFromNum
