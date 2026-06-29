// world.c -- world query functions

// leave this as first line for PCH reasons...
//

use core::ffi::c_int;
use core::mem;
use core::ptr;

// Ghoul2 Insert Start

// For CGhoul2Info_v
// For G2 types
// For miniheap

// Ghoul2 Insert End

// For memory debugging features in _DEBUG

// #ifdef _DEBUG
//   #include <float.h>
// #endif //_DEBUG

// Ghoul2 Insert End

// #if MEM_DEBUG
// #include "..\smartheap\heapagnt.h"
// #define SV_TRACE_PROFILE (0)
// #endif

// #if 0 //G2_SUPERSIZEDBBOX is not being used
// static const float superSizedAdd=64.0f;
// #endif

// External types and functions we depend on (stubs for linking)
// These would normally be imported from other modules

pub type vec3_t = [f32; 3];
pub type clipHandle_t = c_int;
pub type qboolean = c_int;

// Local stub definitions for types crossing ABI boundary
// These are documented stubs matching Quake engine layouts

// Ghoul2 Insert Start
#[repr(C)]
pub struct G2CollisionRecord {
    pub mEntityNum: c_int,
    pub mCollisionNormal: [f32; 3],
    pub mPadding: c_int,
}
// Ghoul2 Insert End

#[repr(C)]
pub struct cplane_s {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: c_int,
    pub signbits: c_int,
    pub pad: [c_int; 1],
}

#[repr(C)]
pub struct trace_s {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: f32,
    pub endpos: [f32; 3],
    pub plane: cplane_s,
    pub entityNum: c_int,
    pub G2CollisionMap: [G2CollisionRecord; 32],  // MAX_G2_COLLISIONS, each entry tracks entity collision
}

pub type trace_t = trace_s;

// External gentity_t and svEntity_t defined elsewhere
extern "C" {
    pub type gentity_t;
    pub type svEntity_t;

    pub fn SV_SvEntityForGentity(ent: *const gentity_t) -> *mut svEntity_t;
    pub fn SV_GEntityForSvEntity(ent: *const svEntity_t) -> *mut gentity_t;
    pub fn SV_GentityNum(num: c_int) -> *mut gentity_t;
    pub fn CM_InlineModel(index: c_int) -> clipHandle_t;
    pub fn CM_TempBoxModel(mins: *const f32, maxs: *const f32) -> clipHandle_t;
    pub fn CM_ModelBounds(cmg: *mut c_int, model: clipHandle_t, mins: *mut f32, maxs: *mut f32);
    pub fn CM_BoxLeafnums(mins: *const f32, maxs: *const f32, leafs: *mut c_int, maxleafs: c_int, lastleaf: *mut c_int) -> c_int;
    pub fn CM_LeafArea(leaf: c_int) -> c_int;
    pub fn CM_LeafCluster(leaf: c_int) -> c_int;
    pub fn CM_BoxTrace(trace: *mut trace_t, start: *const f32, end: *const f32, mins: *const f32, maxs: *const f32, model: clipHandle_t, mask: c_int);
    pub fn CM_TransformedBoxTrace(trace: *mut trace_t, start: *const f32, end: *const f32, mins: *const f32, maxs: *const f32, model: clipHandle_t, mask: c_int, origin: *const f32, angles: *const f32);
    pub fn CM_PointContents(p: *const f32, model: clipHandle_t) -> c_int;
    pub fn CM_TransformedPointContents(p: *const f32, model: clipHandle_t, origin: *const f32, angles: *const f32) -> c_int;
    pub fn Com_Printf(fmt: *const u8, ...);
    pub fn Com_DPrintf(fmt: *const u8, ...);
    pub fn RadiusFromBounds(mins: *const f32, maxs: *const f32) -> f32;

    // Ghoul2 Insert Start
    pub static mut cmg: c_int;
    pub static mut sv: ServerState;

    pub type EG2_Collision;
    pub fn G2API_CollisionDetect(g2_map: *mut G2CollisionRecord, ghoul2: *mut c_int, angles: *const f32, position: *const f32, time: c_int, entity_num: c_int, start: *const f32, end: *const f32, model_scale: f32, vert_space: c_int, trace_type: EG2_Collision, use_lod: c_int, radius: f32);
    // Ghoul2 Insert End
}

// Ghoul2 Insert Start
#[repr(C)]
pub struct ServerState {
    pub state: c_int,
    pub time: c_int,
}

const G2VertSpaceServer: c_int = 1;
const SS_LOADING: c_int = 0;
// Ghoul2 Insert End

const CONTENTS_SOLID: c_int = 1;
const CONTENTS_BODY: c_int = 2;
const CONTENTS_LIGHTSABER: c_int = 4;
const ENTITYNUM_NONE: c_int = -1;
const ENTITYNUM_WORLD: c_int = -2;

const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;

const SOLID_BMODEL: c_int = 0;

const MAX_TOTAL_ENT_LEAFS: usize = 128;
const AREA_DEPTH: c_int = 8;
const AREA_NODES: usize = 1024;
const MAX_ENT_CLUSTERS: usize = 32;
const MAX_GENTITIES: usize = 1024;
const MAX_G2_COLLISIONS: usize = 32;

pub const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

/*
===============================================================================

ENTITY CHECKING

To avoid linearly searching through lists of entities during environment testing,
the world is carved up with an evenly spaced, axially aligned bsp tree.  Entities
are kept in chains either at the final leafs, or at the first node that splits
them, which prevents having to deal with multiple fragments of a single entity.

===============================================================================
*/

#[repr(C)]
pub struct worldSector_s {
    pub axis: c_int,          // -1 = leaf node
    pub dist: f32,
    pub children: [*mut worldSector_s; 2],
    pub entities: *mut svEntity_t,
}

pub type worldSector_t = worldSector_s;

pub static mut sv_worldSectors: [worldSector_t; AREA_NODES] = [worldSector_t {
    axis: 0,
    dist: 0.0,
    children: [ptr::null_mut(); 2],
    entities: ptr::null_mut(),
}; AREA_NODES];

pub static mut sv_numworldSectors: c_int = 0;

/*
================
SV_ClipHandleForEntity

Returns a headnode that can be used for testing or clipping to a
given entity.  If the entity is a bsp model, the headnode will
be returned, otherwise a custom box tree will be constructed.
================
*/
pub unsafe fn SV_ClipHandleForEntity(ent: *const gentity_t) -> clipHandle_t {
    if (*ent).bmodel != 0 {
        // explicit hulls in the BSP model
        return CM_InlineModel((*ent).s.modelindex);
    }

    // create a temp tree from bounding box sizes
    CM_TempBoxModel(
        addr_of!((*ent).mins) as *const f32,
        addr_of!((*ent).maxs) as *const f32,
    ) //,ent->contents );
}

/*
===============
SV_CreateworldSector

Builds a uniformly subdivided tree for the given world size
===============
*/
pub unsafe fn SV_CreateworldSector(depth: c_int, mins: *const f32, maxs: *const f32) -> *mut worldSector_t {
    let anode: *mut worldSector_t;
    let mut size: vec3_t = [0.0; 3];
    let mut mins1: vec3_t = [0.0; 3];
    let mut maxs1: vec3_t = [0.0; 3];
    let mut mins2: vec3_t = [0.0; 3];
    let mut maxs2: vec3_t = [0.0; 3];

    anode = addr_of_mut!(sv_worldSectors[sv_numworldSectors as usize]);
    sv_numworldSectors += 1;

    if depth == AREA_DEPTH {
        (*anode).axis = -1;
        (*anode).children[0] = ptr::null_mut();
        (*anode).children[1] = ptr::null_mut();
        return anode;
    }

    // VectorSubtract(maxs, mins, size)
    for i in 0..3 {
        size[i] = *maxs.add(i) - *mins.add(i);
    }

    if size[0] > size[1] {
        (*anode).axis = 0;
    } else {
        (*anode).axis = 1;
    }

    (*anode).dist = 0.5 * (*maxs.add((*anode).axis as usize) + *mins.add((*anode).axis as usize));

    // VectorCopy(mins, mins1)
    for i in 0..3 {
        mins1[i] = *mins.add(i);
    }
    // VectorCopy(mins, mins2)
    for i in 0..3 {
        mins2[i] = *mins.add(i);
    }
    // VectorCopy(maxs, maxs1)
    for i in 0..3 {
        maxs1[i] = *maxs.add(i);
    }
    // VectorCopy(maxs, maxs2)
    for i in 0..3 {
        maxs2[i] = *maxs.add(i);
    }

    maxs1[(*anode).axis as usize] = (*anode).dist;
    mins2[(*anode).axis as usize] = (*anode).dist;

    (*anode).children[0] = SV_CreateworldSector(depth + 1, mins2.as_ptr(), maxs2.as_ptr());
    (*anode).children[1] = SV_CreateworldSector(depth + 1, mins1.as_ptr(), maxs1.as_ptr());

    anode
}

/*
===============
SV_ClearWorld

===============
*/
pub unsafe fn SV_ClearWorld() {
    let h: clipHandle_t;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    ptr::write_bytes(
        addr_of_mut!(sv_worldSectors) as *mut u8,
        0,
        mem::size_of_val(&sv_worldSectors),
    );
    sv_numworldSectors = 0;

    // get world map bounds
    h = CM_InlineModel(0);
    CM_ModelBounds(addr_of_mut!(cmg), h, mins.as_mut_ptr(), maxs.as_mut_ptr());
    SV_CreateworldSector(0, mins.as_ptr(), maxs.as_ptr());
}

/*
===============
SV_UnlinkEntity

===============
*/
pub unsafe fn SV_UnlinkEntity(gEnt: *mut gentity_t) {
    let ent: *mut svEntity_t;
    let mut scan: *mut svEntity_t;
    let ws: *mut worldSector_t;

    // this should never be called with a freed entity
    if (*gEnt).inuse == 0 {
        return;
    }

    ent = SV_SvEntityForGentity(gEnt);

    (*gEnt).linked = 0; // qfalse

    ws = (*ent).worldSector;
    if ws.is_null() {
        return; // not linked in anywhere
    }
    (*ent).worldSector = ptr::null_mut();

    if (*ws).entities == ent {
        (*ws).entities = (*ent).nextEntityInWorldSector;
        return;
    }

    scan = (*ws).entities;
    while !scan.is_null() {
        if (*scan).nextEntityInWorldSector == ent {
            (*scan).nextEntityInWorldSector = (*ent).nextEntityInWorldSector;
            return;
        }
        scan = (*scan).nextEntityInWorldSector;
    }

    Com_Printf(b"WARNING: SV_UnlinkEntity: not found in worldSector\n\0" as *const u8);
}

/*
===============
SV_LinkEntity

===============
*/
pub unsafe fn SV_LinkEntity(gEnt: *mut gentity_t) {
    let mut node: *mut worldSector_t;
    let mut leafs: [c_int; MAX_TOTAL_ENT_LEAFS] = [0; MAX_TOTAL_ENT_LEAFS];
    let mut cluster: c_int;
    let mut num_leafs: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut area: c_int = 0;
    let ent: *mut svEntity_t;

    // this should never be called with a freed entity
    if (*gEnt).inuse == 0 {
        return;
    }

    ent = SV_SvEntityForGentity(gEnt);

    if !(*ent).worldSector.is_null() {
        SV_UnlinkEntity(gEnt); // unlink from old position
    }

    // encode the size into the entityState_t for client prediction
    if (*gEnt).bmodel != 0 {
        (*gEnt).s.solid = SOLID_BMODEL; // a solid_box will never create this value
    } else if ((*gEnt).contents & (CONTENTS_SOLID | CONTENTS_BODY)) != 0 {
        // assume that x/y are equal and symetric
        i = (*gEnt).maxs[0] as c_int;
        if i < 1 {
            i = 1;
        }
        if i > 255 {
            i = 255;
        }

        // z is not symetric
        j = (-(*gEnt).mins[2]) as c_int;
        if j < 1 {
            j = 1;
        }
        if j > 255 {
            j = 255;
        }

        // and z maxs can be negative...
        k = ((*gEnt).maxs[2] + 32.0) as c_int;
        if k < 1 {
            k = 1;
        }
        if k > 255 {
            k = 255;
        }

        (*gEnt).s.solid = ((k << 16) | (j << 8) | i) as c_int;
    } else {
        (*gEnt).s.solid = 0;
    }

    // get the position
    let origin: *const f32 = addr_of!((*gEnt).currentOrigin) as *const f32;
    let angles: *const f32 = addr_of!((*gEnt).currentAngles) as *const f32;

    // set the abs box
    if ((*gEnt).bmodel != 0) && ((*angles.add(0)) != 0.0 || (*angles.add(1)) != 0.0 || (*angles.add(2)) != 0.0)
    {
        // expand for rotation
        let mut max: f32;
        let mut ii: c_int;

        max = RadiusFromBounds(
            addr_of!((*gEnt).mins) as *const f32,
            addr_of!((*gEnt).maxs) as *const f32,
        );
        ii = 0;
        while ii < 3 {
            (*gEnt).absmin[ii as usize] = *origin.add(ii as usize) - max;
            (*gEnt).absmax[ii as usize] = *origin.add(ii as usize) + max;
            ii += 1;
        }
    } else {
        // normal
        // VectorAdd(origin, gEnt->mins, gEnt->absmin)
        for ii in 0..3 {
            (*gEnt).absmin[ii] = *origin.add(ii) + (*gEnt).mins[ii];
        }
        // VectorAdd(origin, gEnt->maxs, gEnt->absmax)
        for ii in 0..3 {
            (*gEnt).absmax[ii] = *origin.add(ii) + (*gEnt).maxs[ii];
        }
    }

    // because movement is clipped an epsilon away from an actual edge,
    // we must fully check even when bounding boxes don't quite touch
    (*gEnt).absmin[0] -= 1.0;
    (*gEnt).absmin[1] -= 1.0;
    (*gEnt).absmin[2] -= 1.0;
    (*gEnt).absmax[0] += 1.0;
    (*gEnt).absmax[1] += 1.0;
    (*gEnt).absmax[2] += 1.0;

    // link to PVS leafs
    (*ent).numClusters = 0;
    (*ent).lastCluster = 0;
    (*ent).areanum = -1;
    (*ent).areanum2 = -1;

    //get all leafs, including solids
    let mut lastLeaf: c_int = 0;
    num_leafs = CM_BoxLeafnums(
        (*gEnt).absmin.as_ptr(),
        (*gEnt).absmax.as_ptr(),
        leafs.as_mut_ptr(),
        MAX_TOTAL_ENT_LEAFS as c_int,
        addr_of_mut!(lastLeaf),
    );

    // if none of the leafs were inside the map, the
    // entity is outside the world and can be considered unlinked
    if num_leafs == 0 {
        return;
    }

    // set areas, even from clusters that don't fit in the entity array
    i = 0;
    while i < num_leafs {
        area = CM_LeafArea(leafs[i as usize]);
        if area != -1 {
            // doors may legally straggle two areas,
            // but nothing should evern need more than that
            if (*ent).areanum != -1 && (*ent).areanum != area {
                if (*ent).areanum2 != -1 && (*ent).areanum2 != area && unsafe { sv.state } == SS_LOADING {
                    Com_DPrintf(
                        b"Object %i touching 3 areas at %f %f %f\n\0" as *const u8,
                        (*gEnt).s.number,
                        (*gEnt).absmin[0],
                        (*gEnt).absmin[1],
                        (*gEnt).absmin[2],
                    );
                }
                (*ent).areanum2 = area;
            } else {
                (*ent).areanum = area;
            }
        }
        i += 1;
    }

    // store as many explicit clusters as we can
    (*ent).numClusters = 0;
    i = 0;
    while i < num_leafs {
        cluster = CM_LeafCluster(leafs[i as usize]);
        if cluster != -1 {
            (*ent).clusternums[(*ent).numClusters as usize] = cluster;
            (*ent).numClusters += 1;
            if (*ent).numClusters == MAX_ENT_CLUSTERS as c_int {
                break;
            }
        }
        i += 1;
    }

    // store off a last cluster if we need to
    if i != num_leafs {
        (*ent).lastCluster = CM_LeafCluster(lastLeaf);
    }

    // find the first world sector node that the ent's box crosses
    node = addr_of_mut!(sv_worldSectors[0]);
    loop {
        if (*node).axis == -1 {
            break;
        }
        if (*gEnt).absmin[(*node).axis as usize] > (*node).dist {
            node = (*node).children[0];
        } else if (*gEnt).absmax[(*node).axis as usize] < (*node).dist {
            node = (*node).children[1];
        } else {
            break; // crosses the node
        }
    }

    // link it in
    (*ent).worldSector = node;
    (*ent).nextEntityInWorldSector = (*node).entities;
    (*node).entities = ent;

    (*gEnt).linked = 1; // qtrue
}

/*
============================================================================

AREA QUERY

Fills in a list of all entities who's absmin / absmax intersects the given
bounds.  This does NOT mean that they actually touch in the case of bmodels.
============================================================================
*/

#[repr(C)]
pub struct areaParms_t {
    pub mins: *const f32,
    pub maxs: *const f32,
    pub list: *mut *mut gentity_t,
    pub count: c_int,
    pub maxcount: c_int,
}

/*
====================
SV_AreaEntities_r

====================
*/
pub unsafe fn SV_AreaEntities_r(node: *mut worldSector_t, ap: *mut areaParms_t) {
    let mut check: *mut svEntity_t;
    let next: *mut svEntity_t;
    let gcheck: *mut gentity_t;
    let mut count: c_int;

    count = 0;

    check = (*node).entities;
    while !check.is_null() {
        next = (*check).nextEntityInWorldSector;

        gcheck = SV_GEntityForSvEntity(check);

        if (*gcheck).absmin[0] > *(*ap).maxs.add(0)
            || (*gcheck).absmin[1] > *(*ap).maxs.add(1)
            || (*gcheck).absmin[2] > *(*ap).maxs.add(2)
            || (*gcheck).absmax[0] < *(*ap).mins.add(0)
            || (*gcheck).absmax[1] < *(*ap).mins.add(1)
            || (*gcheck).absmax[2] < *(*ap).mins.add(2)
        {
            check = next;
            continue;
        }

        if (*ap).count == (*ap).maxcount {
            Com_DPrintf(
                b"SV_AreaEntities: reached maxcount (%d)\n\0" as *const u8,
                (*ap).maxcount,
            );
            return;
        }

        *(*ap).list.add((*ap).count as usize) = gcheck;
        (*ap).count += 1;
        check = next;
    }

    if (*node).axis == -1 {
        return; // terminal node
    }

    // recurse down both sides
    if *(*ap).maxs.add((*node).axis as usize) > (*node).dist {
        SV_AreaEntities_r((*node).children[0], ap);
    }
    if *(*ap).mins.add((*node).axis as usize) < (*node).dist {
        SV_AreaEntities_r((*node).children[1], ap);
    }
}

/*
================
SV_AreaEntities
================
*/
pub unsafe fn SV_AreaEntities(
    mins: *const f32,
    maxs: *const f32,
    elist: *mut *mut gentity_t,
    maxcount: c_int,
) -> c_int {
    let mut ap: areaParms_t;

    ap.mins = mins;
    ap.maxs = maxs;
    ap.list = elist;
    ap.count = 0;
    ap.maxcount = maxcount;

    // #if SV_TRACE_PROFILE
    // #if MEM_DEBUG
    //   {
    //       int old=dbgMemSetCheckpoint(2003);
    //       malloc(1);
    //       dbgMemSetCheckpoint(old);
    //   }
    // #endif
    // #endif

    SV_AreaEntities_r(addr_of_mut!(sv_worldSectors[0]), addr_of_mut!(ap));

    ap.count
}

/*
===============
SV_SectorList_f
===============
*/

pub unsafe fn SV_SectorList_f() {
    let mut i: c_int;
    let mut c: c_int;
    let sec: *mut worldSector_t;
    let mut ent: *mut svEntity_t;

    i = 0;
    while (i as usize) < AREA_NODES {
        sec = addr_of_mut!(sv_worldSectors[i as usize]);

        c = 0;
        ent = (*sec).entities;
        while !ent.is_null() {
            c += 1;
            ent = (*ent).nextEntityInWorldSector;
        }
        Com_Printf(b"sector %i: %i entities\n\0" as *const u8, i, c);
        i += 1;
    }
}

// Note: The C++ STL-based implementation (#else block) is omitted
// as it requires C++ features not suitable for faithful Rust translation.
// Using the #if 1 version above instead.

//===========================================================================

#[repr(C)]
pub struct moveclip_t {
    pub boxmins: vec3_t,
    pub boxmaxs: vec3_t,
    // enclose the test object along entire move
    pub mins: *const f32,
    pub maxs: *const f32,
    // size of the moving object
    // Ghoul2 Insert Start
    pub start: vec3_t,
    // Ghoul2 Insert End
    pub end: vec3_t,
    pub passEntityNum: c_int,
    pub contentmask: c_int,
    // Ghoul2 Insert Start
    pub eG2TraceType: c_int, // EG2_Collision
    pub useLod: c_int,
    pub trace: trace_t, // make sure nothing goes under here for Ghoul2 collision purposes
    // Ghoul2 Insert End
}

/*
====================
SV_ClipMoveToEntities

====================
*/
pub unsafe fn SV_ClipMoveToEntities(clip: *mut moveclip_t) {
    let mut i: c_int = 0;
    let mut num: c_int;
    let mut touchlist: [*mut gentity_t; 1024] = [ptr::null_mut(); 1024]; // MAX_GENTITIES

    num = SV_AreaEntities((*clip).boxmins.as_ptr(), (*clip).boxmaxs.as_ptr(), touchlist.as_mut_ptr(), MAX_GENTITIES as c_int);

    let owner: *mut gentity_t = if (*clip).passEntityNum != ENTITYNUM_NONE {
        (*SV_GentityNum((*clip).passEntityNum)).owner
    } else {
        ptr::null_mut()
    };

    while i < num {
        if (*clip).trace.allsolid != 0 {
            return;
        }
        let touch: *mut gentity_t = touchlist[i as usize];

        // see if we should ignore this entity
        if (*clip).passEntityNum != ENTITYNUM_NONE {
            if (*touch).s.number == (*clip).passEntityNum {
                i += 1;
                continue; // don't clip against the pass entity
            }
            if !(*touch).owner.is_null() && (*(*touch).owner).s.number == (*clip).passEntityNum {
                i += 1;
                continue; // don't clip against own missiles
            }
            if owner == touch {
                i += 1;
                continue; // don't clip against owner
            }
            if !owner.is_null() && (*touch).owner == owner {
                i += 1;
                continue; // don't clip against other missiles from our owner
            }
        }

        // if it doesn't have any brushes of a type we
        // are looking for, ignore it
        if ((*clip).contentmask & (*touch).contents) == 0 {
            i += 1;
            continue;
        }

        // might intersect, so do an exact clip
        let clipHandle: clipHandle_t = SV_ClipHandleForEntity(touch);

        let origin: *const f32 = addr_of!((*touch).currentOrigin) as *const f32;
        let angles: *const f32 = addr_of!((*touch).currentAngles) as *const f32;

        if (*touch).bmodel == 0 {
            angles = vec3_origin.as_ptr(); // boxes don't rotate
        }

        // #if 0 //G2_SUPERSIZEDBBOX is not being used
        //   bool shrinkBox=true;
        //
        //   if (clip->eG2TraceType != G2_SUPERSIZEDBBOX)
        //   {
        //       shrinkBox=false;
        //   }
        //   else if (trace.entityNum == touch->s.number&&touch->ghoul2.size()&&!(touch->contents & CONTENTS_LIGHTSABER))
        //   {
        //       shrinkBox=false;
        //   }
        //   if (shrinkBox)
        //   {
        //       vec3_t sh_mins;
        //       vec3_t sh_maxs;
        //       int j;
        //       for ( j=0 ; j<3 ; j++ )
        //       {
        //           sh_mins[j]=clip->mins[j]+superSizedAdd;
        //           sh_maxs[j]=clip->maxs[j]-superSizedAdd;
        //       }
        //       CM_TransformedBoxTrace ( &trace, clip->start, clip->end,
        //           sh_mins, sh_maxs, clipHandle,  clip->contentmask,
        //           origin, angles);
        //   }
        //   else
        // #endif
        {
            // #ifdef __MACOS__
            //   // compiler bug with const
            //   CM_TransformedBoxTrace ( &trace, (float *)clip->start, (float *)clip->end,
            //       (float *)clip->mins, (float *)clip->maxs, clipHandle,  clip->contentmask,
            //       origin, angles);
            // #else
            let mut trace: trace_t = mem::zeroed();
            CM_TransformedBoxTrace(
                addr_of_mut!(trace),
                (*clip).start.as_ptr(),
                (*clip).end.as_ptr(),
                (*clip).mins,
                (*clip).maxs,
                clipHandle,
                (*clip).contentmask,
                origin,
                angles,
            );
            // #endif
            //FIXME: when startsolid in another ent, doesn't return correct entityNum
            //ALSO: 2 players can be standing next to each other and this function will
            //think they're in each other!!!
            let oldTrace: trace_t = (*clip).trace;

            if trace.allsolid != 0 {
            if (*clip).trace.allsolid == 0 {
                //We didn't come in here all solid, so set the clip->trace's entityNum
                (*clip).trace.entityNum = (*touch).s.number;
            }
            (*clip).trace.allsolid = 1; // qtrue
            trace.entityNum = (*touch).s.number;
        } else if trace.startsolid != 0 {
            if (*clip).trace.startsolid == 0 {
                //We didn't come in here starting solid, so set the clip->trace's entityNum
                (*clip).trace.entityNum = (*touch).s.number;
            }
            (*clip).trace.startsolid = 1; // qtrue
            trace.entityNum = (*touch).s.number;
        }

        if trace.fraction < (*clip).trace.fraction {
            let oldStart: c_int;

            // make sure we keep a startsolid from a previous trace
            oldStart = (*clip).trace.startsolid;

            trace.entityNum = (*touch).s.number;
            (*clip).trace = trace;
            (*clip).trace.startsolid |= oldStart;
        }

        // Ghoul2 Insert Start

        // decide if we should do the ghoul2 collision detection right here
        if (trace.entityNum == (*touch).s.number) && ((*clip).eG2TraceType != 0) // G2_NOCOLLIDE = 0
        {
            // do we actually have a ghoul2 model here?
            if 0 != 0 && ((*touch).contents & CONTENTS_LIGHTSABER) == 0 // ghoul2.size() check omitted, stubbed with 0!=0
            {
                let mut oldTraceRecSize: c_int = 0;
                let mut newTraceRecSize: c_int = 0;
                let mut z: c_int;

                // we have to do this because sometimes you may hit a model's bounding box, but not actually penetrate the Ghoul2 Models polygons
                // this is, needless to say, not good. So we must check to see if we did actually hit the model, and if not, reset the trace stuff
                // to what it was to begin with

                // set our trace record size
                z = 0;
                while z < MAX_G2_COLLISIONS as c_int {
                    // if (clip->trace.G2CollisionMap[z].mEntityNum != -1)
                    // {
                    //     oldTraceRecSize++;
                    // }
                    z += 1;
                }

                // if we are looking at an entity then use the player state to get it's angles and origin from
                let mut radius: f32;
                // #if 0 //G2_SUPERSIZEDBBOX is not being used
                //   if (clip->eG2TraceType == G2_SUPERSIZEDBBOX)
                //   {
                //       radius=(clip->maxs[0]-clip->mins[0]-2.0f*superSizedAdd)/2.0f;
                //   }
                //   else
                // #endif
                {
                    radius = ((*clip).maxs[0] - (*clip).mins[0]) / 2.0;
                }
                if 0 != 0 // touch->client check
                {
                    // vec3_t world_angles;
                    //
                    // world_angles[PITCH] =  0;
                    // //legs do not *always* point toward the viewangles!
                    // //world_angles[YAW] =  touch->client->viewangles[YAW];
                    // world_angles[YAW] =  touch->client->legsYaw;
                    // world_angles[ROLL] =  0;
                    //
                    // G2API_CollisionDetect(clip->trace.G2CollisionMap, touch->ghoul2,
                    //     world_angles, touch->client->origin, sv.time, touch->s.number, clip->start, clip->end, touch->s.modelScale, G2VertSpaceServer, clip->eG2TraceType, clip->useLod,radius);
                } else {
                    //use the correct origin and angles!  is this right now?
                    // G2API_CollisionDetect(clip->trace.G2CollisionMap, touch->ghoul2,
                    //   touch->currentAngles, touch->currentOrigin, sv.time, touch->s.number, clip->start, clip->end, touch->s.modelScale, G2VertSpaceServer, clip->eG2TraceType, clip->useLod,radius);
                }

                // set our new trace record size

                z = 0;
                while z < MAX_G2_COLLISIONS as c_int {
                    // if (clip->trace.G2CollisionMap[z].mEntityNum != -1)
                    // {
                    //     newTraceRecSize++;
                    // }
                    z += 1;
                }

                // did we actually touch this model? If not, lets reset this ent as being hit..
                if newTraceRecSize == oldTraceRecSize {
                    (*clip).trace = oldTrace;
                } else {
                    //this trace was valid, so copy the best collision into quake trace place info
                    z = 0;
                    while z < MAX_G2_COLLISIONS as c_int {
                        // if (clip->trace.G2CollisionMap[z].mEntityNum==touch->s.number)
                        // {
                        //     clip->trace.plane.normal[0] = clip->trace.G2CollisionMap[z].mCollisionNormal[0];
                        //     clip->trace.plane.normal[1] = clip->trace.G2CollisionMap[z].mCollisionNormal[1];
                        //     clip->trace.plane.normal[2] = clip->trace.G2CollisionMap[z].mCollisionNormal[2];
                        //     break;
                        // }
                        z += 1;
                    }
                    // assert(z<MAX_G2_COLLISIONS); // hmm well ah, weird
                    // assert(VectorLength(clip->trace.plane.normal)>0.1f);
                }
            }
        }
        // Ghoul2 Insert End

        i += 1;
    }
}

/*
==================
SV_Trace

Moves the given mins/maxs volume through the world from start to end.
passEntityNum and entities owned by passEntityNum are explicitly not checked.
==================
*/
// Ghoul2 Insert Start
pub unsafe fn SV_Trace(
    results: *mut trace_t,
    start: *const f32,
    mins: *const f32,
    maxs: *const f32,
    end: *const f32,
    passEntityNum: c_int,
    contentmask: c_int,
    eG2TraceType: c_int, // EG2_Collision
    useLod: c_int,
) {
    // Ghoul2 Insert End
    // #ifdef _DEBUG
    //   assert( !_isnan(start[0])&&!_isnan(start[1])&&!_isnan(start[2])&&!_isnan(end[0])&&!_isnan(end[1])&&!_isnan(end[2]));
    // #endif// _DEBUG

    // #if SV_TRACE_PROFILE
    // #if MEM_DEBUG
    //   {
    //       int old=dbgMemSetCheckpoint(2002);
    //       malloc(1);
    //       dbgMemSetCheckpoint(old);
    //   }
    // #endif
    // #endif

    let mut clip: moveclip_t = mem::zeroed();
    let mut i: c_int = 0;
    //   int     startMS, endMS;
    let mut world_frac: f32 = 0.0;

    /*
    startMS = Sys_Milliseconds ();
    numTraces++;
    */
    let mins_ptr: *const f32 = if mins.is_null() {
        vec3_origin.as_ptr()
    } else {
        mins
    };
    let maxs_ptr: *const f32 = if maxs.is_null() {
        vec3_origin.as_ptr()
    } else {
        maxs
    };

    // clip to world
    //NOTE: this will stop not only on static architecture but also entity brushes such as
    //doors, etc.  This prevents us from being able to shorten the trace so that we can
    //ignore all ents past this endpoint... perhaps need to check the entityNum in this
    //BoxTrace or have it not clip against entity brushes here.
    CM_BoxTrace(addr_of_mut!(clip.trace), start, end, mins_ptr, maxs_ptr, 0, contentmask);
    clip.trace.entityNum = if clip.trace.fraction != 1.0 {
        ENTITYNUM_WORLD
    } else {
        ENTITYNUM_NONE
    };
    if clip.trace.fraction == 0.0 {
        // blocked immediately by the world
        *results = clip.trace;
        //   goto addtime;
        return;
    }

    clip.contentmask = contentmask;
    // Ghoul2 Insert Start
    // VectorCopy(start, clip.start)
    for ii in 0..3 {
        clip.start[ii] = *start.add(ii);
    }
    clip.eG2TraceType = eG2TraceType;
    clip.useLod = useLod;
    // Ghoul2 Insert End

    //Shorten the trace to the size of the trace until it hit the world
    // VectorCopy(clip.trace.endpos, clip.end)
    for ii in 0..3 {
        clip.end[ii] = clip.trace.endpos[ii]; // assuming endpos is vec3_t
    }
    //remember the current completion fraction
    world_frac = clip.trace.fraction;
    //set the fraction back to 1.0 for the trace vs. entities
    clip.trace.fraction = 1.0;

    //VectorCopy( end, clip.end );
    // create the bounding box of the entire move
    // we can limit it to the part of the move not
    // already clipped off by the world, which can be
    // a significant savings for line of sight and shot traces
    clip.passEntityNum = passEntityNum;

    // #if 0 //G2_SUPERSIZEDBBOX is not being used
    //   vec3_t superMin;
    //   vec3_t superMax;  // prison, in boscobel
    //
    //   if (eG2TraceType==G2_SUPERSIZEDBBOX)
    //   {
    //       for ( i=0 ; i<3 ; i++ )
    //       {
    //           superMin[i]=mins[i]-superSizedAdd;
    //           superMax[i]=maxs[i]+superSizedAdd;
    //       }
    //       clip.mins = superMin;
    //       clip.maxs = superMax;
    //   }
    //   else
    // #endif
    {
        clip.mins = mins_ptr;
        clip.maxs = maxs_ptr;
    }

    i = 0;
    while i < 3 {
        if *end.add(i as usize) > *start.add(i as usize) {
            clip.boxmins[i as usize] = clip.start[i as usize] + *clip.mins.add(i as usize) - 1.0;
            clip.boxmaxs[i as usize] = clip.end[i as usize] + *clip.maxs.add(i as usize) + 1.0;
        } else {
            clip.boxmins[i as usize] = clip.end[i as usize] + *clip.mins.add(i as usize) - 1.0;
            clip.boxmaxs[i as usize] = clip.start[i as usize] + *clip.maxs.add(i as usize) + 1.0;
        }
        i += 1;
    }

    // clip to other solid entities
    SV_ClipMoveToEntities(addr_of_mut!(clip));

    //scale the trace back down by the previous fraction
    clip.trace.fraction *= world_frac;
    *results = clip.trace;

    /*
    addtime:
        endMS = Sys_Milliseconds ();

        timeInTrace += endMS - startMS;
    */
}

/*
=============
SV_PointContents
=============
*/
pub unsafe fn SV_PointContents(p: *const f32, passEntityNum: c_int) -> c_int {
    let mut touch: [*mut gentity_t; 1024] = [ptr::null_mut(); 1024]; // MAX_GENTITIES
    let mut i: c_int = 0;
    let mut num: c_int;
    let mut contents: c_int;
    //   int     startMS, endMS;

    // #if MEM_DEBUG
    // #if SV_TRACE_PROFILE
    //   {
    //       int old=dbgMemSetCheckpoint(2001);
    //       malloc(1);
    //       dbgMemSetCheckpoint(old);
    //   }
    // #endif
    // #endif

    /*
    startMS = Sys_Milliseconds ();
    numTraces++;
    */

    // get base contents from world
    contents = CM_PointContents(p, 0);

    // or in contents from all the other entities
    num = SV_AreaEntities(p, p, touch.as_mut_ptr(), MAX_GENTITIES as c_int);

    while i < num {
        let hit: *mut gentity_t = touch[i as usize];
        if (*hit).s.number == passEntityNum {
            i += 1;
            continue;
        }
        // might intersect, so do an exact clip
        let clipHandle: clipHandle_t = SV_ClipHandleForEntity(hit);
        let mut angles: *const f32 = addr_of!((*hit).s.angles) as *const f32;
        if (*hit).bmodel == 0 {
            angles = vec3_origin.as_ptr(); // boxes don't rotate
        }

        let c2: c_int = CM_TransformedPointContents(p, clipHandle, addr_of!((*hit).s.origin) as *const f32, angles);

        contents |= c2;
        i += 1;
    }

    /*
    endMS = Sys_Milliseconds ();
    timeInTrace += endMS - startMS;
    */
    contents
}
