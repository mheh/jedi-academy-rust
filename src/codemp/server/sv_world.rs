//! Faithful port of `codemp/server/sv_world.cpp`.
//!
//! world.c -- world query functions

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_float};
use crate::codemp::game::q_shared_h::{
    vec3_t, vec_t, qboolean, trace_t, byte,
    QTRUE, QFALSE, MAX_G2_COLLISIONS, G2Trace_t, CollisionRecord_t,
    PITCH, ROLL,
};

// External declarations (from server.h, qcommon headers, etc.)
extern "C" {
    // Stub for CMiniHeap (Ghoul2 type)
    pub static mut G2VertSpaceServer: *mut CMiniHeap;
}

// Local type stubs for this file

/// Stub for CMiniHeap (Ghoul2 heap type).
#[repr(C)]
pub struct CMiniHeap {
    _opaque: [u8; 0],
}

/// `clipHandle_t` — handle for collision models.
pub type clipHandle_t = c_int;

/// `sharedEntity_t` — shared entity state (stub; full definition in game headers).
#[repr(C)]
pub struct sharedEntity_t {
    pub r: entityShared_t,
    pub s: entityState_t,
    pub ghoul2: *mut c_int, // CGhoul2Info_v* stub
}

/// `entityShared_t` — shared entity reference frame.
#[repr(C)]
pub struct entityShared_t {
    pub linked: qboolean,
    pub linkcount: c_int,
    pub absmin: vec3_t,
    pub absmax: vec3_t,
    pub currentOrigin: vec3_t,
    pub currentAngles: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub contents: c_int,
    pub bmodel: qboolean,
    pub svFlags: c_int,
    pub ownerNum: c_int,
}

/// `entityState_t` — entity state for networking (stub).
#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub pos: vec3_t,
    pub angles: vec3_t,
    pub apos: trajectory_t,
    pub trBase: vec3_t,
    pub modelindex: c_int,
    pub solid: c_int,
    pub origin: vec3_t,
    pub NPC_class: c_int,
    pub ghoul2: *mut c_int,
    pub modelScale: f32,
    pub number_reserved: [c_int; 20], // Reserve space for additional fields
}

/// `trajectory_t` — trajectory tracking (stub).
#[repr(C)]
pub struct trajectory_t {
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
}

/// `svEntity_t` — server-side entity state.
#[repr(C)]
pub struct svEntity_t {
    pub worldSector: *mut worldSector_t,
    pub nextEntityInWorldSector: *mut svEntity_t,
    pub clusternums: [c_int; MAX_ENT_CLUSTERS],
    pub numClusters: c_int,
    pub lastCluster: c_int,
    pub areanum: c_int,
    pub areanum2: c_int,
}

pub const MAX_ENT_CLUSTERS: usize = 16;

/// `worldSector_t` — spatial partitioning node for entity queries.
#[repr(C)]
pub struct worldSector_t {
    pub axis: c_int,       // -1 = leaf node
    pub dist: c_float,
    pub children: [*mut worldSector_t; 2],
    pub entities: *mut svEntity_t,
}

pub const AREA_DEPTH: c_int = 4;
pub const AREA_NODES: usize = 64;

/// Global world sectors array.
pub static mut sv_worldSectors: [worldSector_t; AREA_NODES] = [worldSector_t {
    axis: 0,
    dist: 0.0,
    children: [core::ptr::null_mut(); 2],
    entities: core::ptr::null_mut(),
}; AREA_NODES];

/// Number of world sectors in use.
pub static mut sv_numworldSectors: c_int = 0;

/// Server state constants.
pub const SS_LOADING: c_int = 0;

/// Entity number constants.
pub const ENTITYNUM_NONE: c_int = -1;
pub const ENTITYNUM_WORLD: c_int = 0;
pub const MAX_GENTITIES: c_int = 4096;

/// Entity type constants.
pub const ET_MISSILE: c_int = 4;
pub const ET_NPC: c_int = 6;

/// Entity flag constants.
pub const EF_DEAD: c_int = 0x00000001;

/// Solid type constants.
pub const SOLID_BMODEL: c_int = 0xffffff;

/// Content mask constants.
pub const CONTENTS_SOLID: c_int = 0x00000001;
pub const CONTENTS_BODY: c_int = 0x00000004;
pub const CONTENTS_LIGHTSABER: c_int = 0x40000000;
pub const CONTENTS_NOSHOT: c_int = 0x00040000;

pub const MASK_SHOT: c_int = CONTENTS_SOLID | CONTENTS_BODY;

/// SVF (Shared Flags) constants.
pub const SVF_CAPSULE: c_int = 0x00000001;
pub const SVF_OWNERNOTSHARED: c_int = 0x00000002;

/// Trace flags (Ghoul2).
pub const G2TRFLAG_DOGHOULTRACE: c_int = 0x00000001;
pub const G2TRFLAG_THICK: c_int = 0x00000002;
pub const G2TRFLAG_GETSURFINDEX: c_int = 0x00000008;
pub const G2TRFLAG_HITCORPSES: c_int = 0x00000010;

/// NPC class constants.
pub const CLASS_VEHICLE: c_int = 9;

// External function declarations
extern "C" {
    pub fn CM_InlineModel(index: c_int) -> clipHandle_t;
    pub fn CM_TempBoxModel(mins: *const vec3_t, maxs: *const vec3_t, capsule: qboolean) -> clipHandle_t;
    pub fn CM_ModelBounds(model: clipHandle_t, mins: *mut vec3_t, maxs: *mut vec3_t);
    pub fn CM_BoxLeafnums(
        mins: *const vec3_t,
        maxs: *const vec3_t,
        leafnums: *mut c_int,
        maxcount: c_int,
        lastleaf: *mut c_int,
    ) -> c_int;
    pub fn CM_LeafArea(leafnum: c_int) -> c_int;
    pub fn CM_LeafCluster(leafnum: c_int) -> c_int;
    pub fn CM_TransformedBoxTrace(
        results: *mut trace_t,
        start: *const vec3_t,
        end: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        model: clipHandle_t,
        brushmask: c_int,
        origin: *const vec3_t,
        angles: *const vec3_t,
        capsule: c_int,
    );
    pub fn CM_BoxTrace(
        results: *mut trace_t,
        start: *const vec3_t,
        end: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        model: clipHandle_t,
        brushmask: c_int,
        capsule: c_int,
    );
    pub fn CM_TransformedPointContents(
        p: *const vec3_t,
        model: clipHandle_t,
        origin: *const vec3_t,
        angles: *const vec3_t,
    ) -> c_int;
    pub fn CM_PointContents(p: *const vec3_t, model: clipHandle_t) -> c_int;

    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Memset(p: *mut c_int, c: c_int, n: usize);

    pub fn SV_SvEntityForGentity(gent: *const sharedEntity_t) -> *mut svEntity_t;
    pub fn SV_GEntityForSvEntity(sent: *mut svEntity_t) -> *mut sharedEntity_t;
    pub fn SV_GentityNum(num: c_int) -> *mut sharedEntity_t;

    pub fn RadiusFromBounds(mins: *const vec3_t, maxs: *const vec3_t) -> vec_t;

    pub fn G2API_CollisionDetect(
        collisionMap: *mut CollisionRecord_t,
        ghoul2: *mut c_int,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNumber: c_int,
        entityNum: c_int,
        rayStart: *const vec3_t,
        rayEnd: *const vec3_t,
        scale: f32,
        G2VertSpace: *mut CMiniHeap,
        traceFlags: c_int,
        useLOD: c_int,
        radius: f32,
    );

    pub fn G2API_CollisionDetectCache(
        collisionMap: *mut CollisionRecord_t,
        ghoul2: *mut c_int,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNumber: c_int,
        entityNum: c_int,
        rayStart: *const vec3_t,
        rayEnd: *const vec3_t,
        scale: f32,
        G2VertSpace: *mut CMiniHeap,
        traceFlags: c_int,
        useLOD: c_int,
    );
}

// Vector math stubs (from q_math.c / mathlib.h)
#[inline]
unsafe fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t) {
    (*dst)[0] = (*src)[0];
    (*dst)[1] = (*src)[1];
    (*dst)[2] = (*src)[2];
}

#[inline]
unsafe fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t) {
    (*c)[0] = (*a)[0] - (*b)[0];
    (*c)[1] = (*a)[1] - (*b)[1];
    (*c)[2] = (*a)[2] - (*b)[2];
}

#[inline]
unsafe fn VectorAdd(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t) {
    (*c)[0] = (*a)[0] + (*b)[0];
    (*c)[1] = (*a)[1] + (*b)[1];
    (*c)[2] = (*a)[2] + (*b)[2];
}

#[inline]
unsafe fn VectorLength(v: *const vec3_t) -> vec_t {
    let x = (*v)[0];
    let y = (*v)[1];
    let z = (*v)[2];
    (x * x + y * y + z * z).sqrt()
}

/// Origin vector (0, 0, 0).
pub static vec3_origin: vec3_t = [0.0, 0.0, 0.0];

// Server state and entity globals (stubs).
#[repr(C)]
pub struct server_t {
    pub state: c_int,
}

#[repr(C)]
pub struct serverStatic_t {
    pub time: c_int,
}

extern "C" {
    pub static mut sv: server_t;
    pub static mut svs: serverStatic_t;
    pub static mut com_optvehtrace: *const cvar_t;
    pub static mut sv_showghoultraces: *const cvar_t;
}

/// `cvar_t` — console variable (stub).
#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub integer: c_int,
}

/*
================
SV_ClipHandleForEntity

Returns a headnode that can be used for testing or clipping to a
given entity.  If the entity is a bsp model, the headnode will
be returned, otherwise a custom box tree will be constructed.
================
*/
#[inline]
pub unsafe fn SV_ClipHandleForEntity(ent: *const sharedEntity_t) -> clipHandle_t {
    if (*ent).r.bmodel != 0 {
        // explicit hulls in the BSP model
        return CM_InlineModel((*ent).s.modelindex);
    }
    if ((*ent).r.svFlags & SVF_CAPSULE) != 0 {
        // create a temp capsule from bounding box sizes
        return CM_TempBoxModel(
            core::ptr::addr_of!((*ent).r.mins),
            core::ptr::addr_of!((*ent).r.maxs),
            QTRUE,
        );
    }

    // create a temp tree from bounding box sizes
    CM_TempBoxModel(
        core::ptr::addr_of!((*ent).r.mins),
        core::ptr::addr_of!((*ent).r.maxs),
        QFALSE,
    )
}

/*
===============================================================================

ENTITY CHECKING

To avoid linearly searching through lists of entities during environment testing,
the world is carved up with an evenly spaced, axially aligned bsp tree.  Entities
are kept in chains either at the final leafs, or at the first node that splits
them, which prevents having to deal with multiple fragments of a single entity.

===============================================================================
*/

/*
===============
SV_SectorList_f
===============
*/
pub unsafe extern "C" fn SV_SectorList_f() {
    let mut i: c_int;
    let mut c: c_int;
    let mut sec: *mut worldSector_t;
    let mut ent: *mut svEntity_t;

    i = 0;
    while i < AREA_NODES as c_int {
        sec = core::ptr::addr_of_mut!(sv_worldSectors[i as usize]);

        c = 0;
        ent = (*sec).entities;
        while !ent.is_null() {
            c += 1;
            ent = (*ent).nextEntityInWorldSector;
        }
        Com_Printf("sector %i: %i entities\n\0".as_ptr() as *const c_char, i, c);
        i += 1;
    }
}

/*
===============
SV_CreateworldSector

Builds a uniformly subdivided tree for the given world size
===============
*/
pub unsafe fn SV_CreateworldSector(
    depth: c_int,
    mins: *const vec3_t,
    maxs: *const vec3_t,
) -> *mut worldSector_t {
    let mut anode: *mut worldSector_t;
    let mut size: vec3_t;
    let mut mins1: vec3_t;
    let mut maxs1: vec3_t;
    let mut mins2: vec3_t;
    let mut maxs2: vec3_t;

    anode = core::ptr::addr_of_mut!(sv_worldSectors[sv_numworldSectors as usize]);
    sv_numworldSectors += 1;

    if depth == AREA_DEPTH {
        (*anode).axis = -1;
        (*anode).children[0] = core::ptr::null_mut();
        (*anode).children[1] = core::ptr::null_mut();
        return anode;
    }

    VectorSubtract(maxs, mins, core::ptr::addr_of_mut!(size));
    if size[0] > size[1] {
        (*anode).axis = 0;
    } else {
        (*anode).axis = 1;
    }

    (*anode).dist = 0.5 * ((*maxs)[(*anode).axis as usize] + (*mins)[(*anode).axis as usize]);
    VectorCopy(mins, core::ptr::addr_of_mut!(mins1));
    VectorCopy(mins, core::ptr::addr_of_mut!(mins2));
    VectorCopy(maxs, core::ptr::addr_of_mut!(maxs1));
    VectorCopy(maxs, core::ptr::addr_of_mut!(maxs2));

    (*maxs1)[(*anode).axis as usize] = (*anode).dist;
    (*mins2)[(*anode).axis as usize] = (*anode).dist;

    (*anode).children[0] = SV_CreateworldSector(depth + 1, core::ptr::addr_of!(mins2), core::ptr::addr_of!(maxs2));
    (*anode).children[1] = SV_CreateworldSector(depth + 1, core::ptr::addr_of!(mins1), core::ptr::addr_of!(maxs1));

    anode
}

/*
===============
SV_ClearWorld

===============
*/
pub unsafe fn SV_ClearWorld() {
    let mut h: clipHandle_t;
    let mut mins: vec3_t;
    let mut maxs: vec3_t;

    Com_Memset(
        core::ptr::addr_of_mut!(sv_worldSectors) as *mut c_int,
        0,
        core::mem::size_of_val(&sv_worldSectors),
    );
    sv_numworldSectors = 0;

    // get world map bounds
    h = CM_InlineModel(0);
    CM_ModelBounds(h, core::ptr::addr_of_mut!(mins), core::ptr::addr_of_mut!(maxs));
    SV_CreateworldSector(0, core::ptr::addr_of!(mins), core::ptr::addr_of!(maxs));
}

/*
===============
SV_UnlinkEntity

===============
*/
pub unsafe fn SV_UnlinkEntity(gEnt: *mut sharedEntity_t) {
    let mut ent: *mut svEntity_t;
    let mut scan: *mut svEntity_t;
    let mut ws: *mut worldSector_t;

    ent = SV_SvEntityForGentity(gEnt);

    (*gEnt).r.linked = QFALSE;

    ws = (*ent).worldSector;
    if ws.is_null() {
        return; // not linked in anywhere
    }
    (*ent).worldSector = core::ptr::null_mut();

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

    Com_Printf("WARNING: SV_UnlinkEntity: not found in worldSector\n\0".as_ptr() as *const c_char);
}

/*
===============
SV_LinkEntity

===============
*/
const MAX_TOTAL_ENT_LEAFS: usize = 128;

pub unsafe fn SV_LinkEntity(gEnt: *mut sharedEntity_t) {
    let mut node: *mut worldSector_t;
    let mut leafs: [c_int; MAX_TOTAL_ENT_LEAFS] = [0; MAX_TOTAL_ENT_LEAFS];
    let mut cluster: c_int;
    let mut num_leafs: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut area: c_int;
    let mut lastLeaf: c_int;
    let mut origin: *const vec3_t;
    let mut angles: *const vec3_t;
    let mut ent: *mut svEntity_t;

    ent = SV_SvEntityForGentity(gEnt);

    if !(*ent).worldSector.is_null() {
        SV_UnlinkEntity(gEnt); // unlink from old position
    }

    // encode the size into the entityState_t for client prediction
    if (*gEnt).r.bmodel != 0 {
        (*gEnt).s.solid = SOLID_BMODEL; // a solid_box will never create this value
    } else if ((*gEnt).r.contents & (CONTENTS_SOLID | CONTENTS_BODY)) != 0 {
        // assume that x/y are equal and symetric
        i = (*gEnt).r.maxs[0] as c_int;
        if i < 1 {
            i = 1;
        }
        if i > 255 {
            i = 255;
        }

        // z is not symetric
        j = (-(*gEnt).r.mins[2]) as c_int;
        if j < 1 {
            j = 1;
        }
        if j > 255 {
            j = 255;
        }

        // and z maxs can be negative...
        k = ((*gEnt).r.maxs[2] as c_int + 32);
        if k < 1 {
            k = 1;
        }
        if k > 255 {
            k = 255;
        }

        (*gEnt).s.solid = (k << 16) | (j << 8) | i;

        if (*gEnt).s.solid == SOLID_BMODEL {
            // yikes, this would make everything explode violently.
            (*gEnt).s.solid = (k << 16) | (j << 8) | (i - 1);
        }
    } else {
        (*gEnt).s.solid = 0;
    }

    // get the position
    origin = core::ptr::addr_of!((*gEnt).r.currentOrigin);
    angles = core::ptr::addr_of!((*gEnt).r.currentAngles);

    // set the abs box
    if (*gEnt).r.bmodel != 0
        && ((*angles)[0] != 0.0 || (*angles)[1] != 0.0 || (*angles)[2] != 0.0)
    {
        // expand for rotation
        let mut max: vec_t;
        let mut i_loop: c_int;

        max = RadiusFromBounds(core::ptr::addr_of!((*gEnt).r.mins), core::ptr::addr_of!((*gEnt).r.maxs));
        i_loop = 0;
        while i_loop < 3 {
            (*gEnt).r.absmin[i_loop as usize] = (*origin)[i_loop as usize] - max;
            (*gEnt).r.absmax[i_loop as usize] = (*origin)[i_loop as usize] + max;
            i_loop += 1;
        }
    } else {
        // normal
        VectorAdd(origin, core::ptr::addr_of!((*gEnt).r.mins), core::ptr::addr_of_mut!((*gEnt).r.absmin));
        VectorAdd(origin, core::ptr::addr_of!((*gEnt).r.maxs), core::ptr::addr_of_mut!((*gEnt).r.absmax));
    }

    // because movement is clipped an epsilon away from an actual edge,
    // we must fully check even when bounding boxes don't quite touch
    (*gEnt).r.absmin[0] -= 1.0;
    (*gEnt).r.absmin[1] -= 1.0;
    (*gEnt).r.absmin[2] -= 1.0;
    (*gEnt).r.absmax[0] += 1.0;
    (*gEnt).r.absmax[1] += 1.0;
    (*gEnt).r.absmax[2] += 1.0;

    // link to PVS leafs
    (*ent).numClusters = 0;
    (*ent).lastCluster = 0;
    (*ent).areanum = -1;
    (*ent).areanum2 = -1;

    // get all leafs, including solids
    num_leafs = CM_BoxLeafnums(
        core::ptr::addr_of!((*gEnt).r.absmin),
        core::ptr::addr_of!((*gEnt).r.absmax),
        leafs.as_mut_ptr(),
        MAX_TOTAL_ENT_LEAFS as c_int,
        core::ptr::addr_of_mut!(lastLeaf),
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
                if (*ent).areanum2 != -1 && (*ent).areanum2 != area && sv.state == SS_LOADING {
                    Com_DPrintf(
                        "Object %i touching 3 areas at %f %f %f\n\0".as_ptr() as *const c_char,
                        (*gEnt).s.number,
                        (*gEnt).r.absmin[0],
                        (*gEnt).r.absmin[1],
                        (*gEnt).r.absmin[2],
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

    (*gEnt).r.linkcount += 1;

    // find the first world sector node that the ent's box crosses
    node = core::ptr::addr_of_mut!(sv_worldSectors[0]);
    loop {
        if (*node).axis == -1 {
            break;
        }
        if (*gEnt).r.absmin[(*node).axis as usize] > (*node).dist {
            node = (*node).children[0];
        } else if (*gEnt).r.absmax[(*node).axis as usize] < (*node).dist {
            node = (*node).children[1];
        } else {
            break; // crosses the node
        }
    }

    // link it in
    (*ent).worldSector = node;
    (*ent).nextEntityInWorldSector = (*node).entities;
    (*node).entities = ent;

    (*gEnt).r.linked = QTRUE;
}

/*
============================================================================

AREA QUERY

Fills in a list of all entities who's absmin / absmax intersects the given
bounds.  This does NOT mean that they actually touch in the case of bmodels.
============================================================================
*/

#[repr(C)]
struct areaParms_t {
    mins: *const vec3_t,
    maxs: *const vec3_t,
    list: *mut c_int,
    count: c_int,
    maxcount: c_int,
}

/*
====================
SV_AreaEntities_r

====================
*/
unsafe fn SV_AreaEntities_r(node: *mut worldSector_t, ap: *mut areaParms_t) {
    let mut check: *mut svEntity_t;
    let mut next: *mut svEntity_t;
    let mut gcheck: *mut sharedEntity_t;
    let mut count: c_int;

    count = 0;

    check = (*node).entities;
    while !check.is_null() {
        next = (*check).nextEntityInWorldSector;

        gcheck = SV_GEntityForSvEntity(check);

        if (*gcheck).r.absmin[0] > (*(*ap).maxs)[0]
            || (*gcheck).r.absmin[1] > (*(*ap).maxs)[1]
            || (*gcheck).r.absmin[2] > (*(*ap).maxs)[2]
            || (*gcheck).r.absmax[0] < (*(*ap).mins)[0]
            || (*gcheck).r.absmax[1] < (*(*ap).mins)[1]
            || (*gcheck).r.absmax[2] < (*(*ap).mins)[2]
        {
            check = next;
            continue;
        }

        if (*ap).count == (*ap).maxcount {
            Com_DPrintf("SV_AreaEntities: MAXCOUNT\n\0".as_ptr() as *const c_char);
            return;
        }

        *(*ap).list.add((*ap).count as usize) = check as *const u8 as usize as c_int
            - core::ptr::addr_of!(sv) as *const u8 as usize as c_int;
        (*ap).count += 1;
        check = next;
    }

    if (*node).axis == -1 {
        return; // terminal node
    }

    // recurse down both sides
    if (*(*ap).maxs)[(*node).axis as usize] > (*node).dist {
        SV_AreaEntities_r((*node).children[0], ap);
    }
    if (*(*ap).mins)[(*node).axis as usize] < (*node).dist {
        SV_AreaEntities_r((*node).children[1], ap);
    }
}

/*
================
SV_AreaEntities
================
*/
pub unsafe fn SV_AreaEntities(
    mins: *const vec3_t,
    maxs: *const vec3_t,
    entityList: *mut c_int,
    maxcount: c_int,
) -> c_int {
    let mut ap: areaParms_t;

    ap.mins = mins;
    ap.maxs = maxs;
    ap.list = entityList;
    ap.count = 0;
    ap.maxcount = maxcount;

    SV_AreaEntities_r(core::ptr::addr_of_mut!(sv_worldSectors[0]), core::ptr::addr_of_mut!(ap));

    ap.count
}

/*
====================
SV_ClipToEntity

====================
*/
pub unsafe fn SV_ClipToEntity(
    trace: *mut trace_t,
    start: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    end: *const vec3_t,
    entityNum: c_int,
    contentmask: c_int,
    capsule: c_int,
) {
    let mut touch: *mut sharedEntity_t;
    let mut clipHandle: clipHandle_t;
    let mut origin: *const vec3_t;
    let mut angles: *const vec3_t;

    touch = SV_GentityNum(entityNum);

    Com_Memset(trace as *mut c_int, 0, core::mem::size_of::<trace_t>());

    // if it doesn't have any brushes of a type we
    // are looking for, ignore it
    if (contentmask & (*touch).r.contents) == 0 {
        (*trace).fraction = 1.0;
        return;
    }

    // might intersect, so do an exact clip
    clipHandle = SV_ClipHandleForEntity(touch);

    origin = core::ptr::addr_of!((*touch).r.currentOrigin);
    angles = core::ptr::addr_of!((*touch).r.currentAngles);

    if (*touch).r.bmodel == 0 {
        angles = core::ptr::addr_of!(vec3_origin); // boxes don't rotate
    }

    CM_TransformedBoxTrace(
        trace,
        start,
        end,
        mins,
        maxs,
        clipHandle,
        contentmask,
        origin,
        angles,
        capsule,
    );

    if (*trace).fraction < 1.0 {
        (*trace).entityNum = (*touch).s.number;
    }
}

/*
====================
SV_ClipMoveToEntities

====================
*/
#[cfg(not(feature = "FINAL_BUILD"))]
unsafe fn VectorDistance(p1: *const vec3_t, p2: *const vec3_t) -> vec_t {
    let mut dir: vec3_t;

    VectorSubtract(p2, p1, core::ptr::addr_of_mut!(dir));
    VectorLength(core::ptr::addr_of!(dir))
}

#[repr(C)]
struct moveclip_t {
    boxmins: vec3_t,
    boxmaxs: vec3_t, // enclose the test object along entire move
    mins: *const vec3_t,
    maxs: *const vec3_t, // size of the moving object
    // Ghoul2 Insert Start
    start: vec3_t,
    end: vec3_t,
    passEntityNum: c_int,
    contentmask: c_int,
    capsule: c_int,
    traceFlags: c_int,
    useLod: c_int,
    trace: trace_t, // make sure nothing goes under here for Ghoul2 collision purposes
    // Ghoul2 Insert End
}

unsafe fn SV_ClipMoveToEntities(clip: *mut moveclip_t) {
    static mut touchlist: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
    let mut i: c_int;
    let mut num: c_int;
    let mut touch: *mut sharedEntity_t;
    let mut passOwnerNum: c_int;
    let mut trace: trace_t;
    let mut oldTrace: trace_t;
    let mut clipHandle: clipHandle_t;
    let mut origin: *const vec3_t;
    let mut angles: *const vec3_t;
    let mut thisOwnerShared: c_int = 1;

    oldTrace = core::mem::zeroed();

    num = SV_AreaEntities(
        core::ptr::addr_of!((*clip).boxmins),
        core::ptr::addr_of!((*clip).boxmaxs),
        touchlist.as_mut_ptr(),
        MAX_GENTITIES,
    );

    if (*clip).passEntityNum != ENTITYNUM_NONE {
        passOwnerNum = (*SV_GentityNum((*clip).passEntityNum)).r.ownerNum;
        if passOwnerNum == ENTITYNUM_NONE {
            passOwnerNum = -1;
        }
    } else {
        passOwnerNum = -1;
    }

    if ((*SV_GentityNum((*clip).passEntityNum)).r.svFlags & SVF_OWNERNOTSHARED) != 0 {
        thisOwnerShared = 0;
    }

    i = 0;
    while i < num {
        if (*clip).trace.allsolid != 0 {
            return;
        }
        touch = SV_GentityNum(touchlist[i as usize]);

        // see if we should ignore this entity
        if (*clip).passEntityNum != ENTITYNUM_NONE {
            if touchlist[i as usize] == (*clip).passEntityNum {
                i += 1;
                continue; // don't clip against the pass entity
            }
            if (*touch).r.ownerNum == (*clip).passEntityNum {
                if ((*touch).r.svFlags & SVF_OWNERNOTSHARED) != 0 {
                    if (*clip).contentmask != (MASK_SHOT | CONTENTS_LIGHTSABER)
                        && (*clip).contentmask != MASK_SHOT
                    {
                        // it's not a laser hitting the other "missile", don't care then
                        i += 1;
                        continue;
                    }
                } else {
                    i += 1;
                    continue; // don't clip against own missiles
                }
            }
            if (*touch).r.ownerNum == passOwnerNum
                && ((*touch).r.svFlags & SVF_OWNERNOTSHARED) == 0
                && thisOwnerShared != 0
            {
                i += 1;
                continue; // don't clip against other missiles from our owner
            }

            if (*touch).s.eType == ET_MISSILE
                && ((*touch).r.svFlags & SVF_OWNERNOTSHARED) == 0
                && (*touch).r.ownerNum == passOwnerNum
            {
                // blah, hack
                i += 1;
                continue;
            }
        }

        // if it doesn't have any brushes of a type we
        // are looking for, ignore it
        if ((*clip).contentmask & (*touch).r.contents) == 0 {
            i += 1;
            continue;
        }

        if ((*clip).contentmask == (MASK_SHOT | CONTENTS_LIGHTSABER)
            || (*clip).contentmask == MASK_SHOT)
            && ((*touch).r.contents > 0 && ((*touch).r.contents & CONTENTS_NOSHOT) != 0)
        {
            i += 1;
            continue;
        }

        // might intersect, so do an exact clip
        clipHandle = SV_ClipHandleForEntity(touch);

        origin = core::ptr::addr_of!((*touch).r.currentOrigin);
        angles = core::ptr::addr_of!((*touch).r.currentAngles);

        if (*touch).r.bmodel == 0 {
            angles = core::ptr::addr_of!(vec3_origin); // boxes don't rotate
        }

        trace = core::mem::zeroed();
        CM_TransformedBoxTrace(
            core::ptr::addr_of_mut!(trace),
            core::ptr::addr_of!((*clip).start),
            core::ptr::addr_of!((*clip).end),
            (*clip).mins,
            (*clip).maxs,
            clipHandle,
            (*clip).contentmask,
            origin,
            angles,
            (*clip).capsule,
        );

        if ((*clip).traceFlags & G2TRFLAG_DOGHOULTRACE) != 0 {
            // keep these older variables around for a bit, incase we need to replace them in the Ghoul2 Collision check
            oldTrace = (*clip).trace;
        }

        if trace.allsolid != 0 {
            (*clip).trace.allsolid = QTRUE;
            trace.entityNum = (*touch).s.number;
        } else if trace.startsolid != 0 {
            (*clip).trace.startsolid = QTRUE;
            trace.entityNum = (*touch).s.number;

            // rww - added this because we want to get the number of an ent even if our trace starts inside it.
            (*clip).trace.entityNum = (*touch).s.number;
        }

        if trace.fraction < (*clip).trace.fraction {
            let oldStart: byte;

            // make sure we keep a startsolid from a previous trace
            oldStart = (*clip).trace.startsolid as byte;

            trace.entityNum = (*touch).s.number;
            (*clip).trace = trace;
            (*clip).trace.startsolid =
                ((((*clip).trace.startsolid as c_int) | (oldStart as c_int)) as qboolean);
        }
        /*
        Ghoul2 Insert Start
        */
        // #if 0 — This code is disabled in the original
        /*
        Ghoul2 Insert End
        */
        // #else
        // rww - since this is multiplayer and we don't have the luxury of violating networking rules in horrible ways,
        // this must be done somewhat differently.
        if ((*clip).traceFlags & G2TRFLAG_DOGHOULTRACE) != 0
            && trace.entityNum == (*touch).s.number
            && !(*touch).ghoul2.is_null()
            && (((*clip).traceFlags & G2TRFLAG_HITCORPSES) != 0
                || ((*touch).s.eFlags & EF_DEAD) == 0)
        {
            // standard behavior will be to ignore g2 col on dead ents, but if traceFlags is set to allow, then we'll try g2 col on EF_DEAD people too.
            static mut G2Trace: G2Trace_t = [CollisionRecord_t {
                mDistance: 0.0,
                mEntityNum: 0,
                mModelIndex: 0,
                mPolyIndex: 0,
                mSurfaceIndex: 0,
                mCollisionPosition: [0.0; 3],
                mCollisionNormal: [0.0; 3],
                mFlags: 0,
                mMaterial: 0,
                mLocation: 0,
                mBarycentricI: 0.0,
                mBarycentricJ: 0.0,
            }; MAX_G2_COLLISIONS];

            let mut angles: vec3_t;
            let mut fRadius: f32 = 0.0;
            let mut tN: c_int = 0;
            let mut bestTr: c_int = -1;

            if (*(*clip).mins)[0] != 0.0 || (*(*clip).maxs)[0] != 0.0 {
                fRadius = ((*(*clip).maxs)[0] - (*(*clip).mins)[0]) / 2.0;
            }

            if ((*clip).traceFlags & G2TRFLAG_THICK) != 0 {
                // if using this flag, make sure it's at least 1.0f
                if fRadius < 1.0 {
                    fRadius = 1.0;
                }
            }

            memset_g2trace(&mut G2Trace);

            if (*touch).s.number < MAX_CLIENTS {
                VectorCopy(core::ptr::addr_of!((*touch).s.apos.trBase), core::ptr::addr_of_mut!(angles));
            } else {
                VectorCopy(core::ptr::addr_of!((*touch).r.currentAngles), core::ptr::addr_of_mut!(angles));
            }
            angles[PITCH] = 0.0;
            angles[ROLL] = 0.0;

            // I would think that you could trace from trace.endpos instead of clip->start, but that causes it to miss sometimes.. Not sure what it's off, but if it could be done like that, it would probably
            // be faster.
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                if !sv_showghoultraces.is_null() && (*sv_showghoultraces).integer != 0 {
                    Com_Printf(
                        "Ghoul2 trace   lod=%1d   length=%6.0f   to %s\n\0".as_ptr() as *const c_char,
                        (*clip).useLod,
                        VectorDistance(core::ptr::addr_of!((*clip).start), core::ptr::addr_of!((*clip).end)),
                        (*(*touch).ghoul2).cast::<c_char>(), // Stub: cast ghoul2 to c_char for display
                    );
                }
            }

            if !com_optvehtrace.is_null()
                && (*com_optvehtrace).integer != 0
                && (*touch).s.eType == ET_NPC
                && (*touch).s.NPC_class == CLASS_VEHICLE
                && !(*touch).ghoul2.is_null() // stub check for m_pVehicle
            {
                // for vehicles cache the transform data.
                G2API_CollisionDetectCache(
                    G2Trace.as_mut_ptr(),
                    (*touch).ghoul2,
                    core::ptr::addr_of!(angles),
                    core::ptr::addr_of!((*touch).r.currentOrigin),
                    svs.time,
                    (*touch).s.number,
                    core::ptr::addr_of!((*clip).start),
                    core::ptr::addr_of!((*clip).end),
                    (*touch).s.modelScale,
                    G2VertSpaceServer,
                    0,
                    (*clip).useLod,
                );
            } else {
                G2API_CollisionDetect(
                    G2Trace.as_mut_ptr(),
                    (*touch).ghoul2,
                    core::ptr::addr_of!(angles),
                    core::ptr::addr_of!((*touch).r.currentOrigin),
                    svs.time,
                    (*touch).s.number,
                    core::ptr::addr_of!((*clip).start),
                    core::ptr::addr_of!((*clip).end),
                    (*touch).s.modelScale,
                    G2VertSpaceServer,
                    0,
                    (*clip).useLod,
                    fRadius,
                );
            }

            tN = 0;
            while tN < MAX_G2_COLLISIONS as c_int {
                if G2Trace[tN as usize].mEntityNum == (*touch).s.number {
                    // ok, valid
                    bestTr = tN;
                    break;
                } else if G2Trace[tN as usize].mEntityNum == -1 {
                    // there should not be any after the first -1
                    break;
                }
                tN += 1;
            }

            if bestTr == -1 {
                // Well then, put the trace back to the old one.
                (*clip).trace = oldTrace;
            } else {
                // Otherwise, set the endpos/normal/etc. to the model location hit instead of leaving it out in space.
                VectorCopy(
                    core::ptr::addr_of!(G2Trace[bestTr as usize].mCollisionPosition),
                    core::ptr::addr_of_mut!((*clip).trace.endpos),
                );
                VectorCopy(
                    core::ptr::addr_of!(G2Trace[bestTr as usize].mCollisionNormal),
                    core::ptr::addr_of_mut!((*clip).trace.plane.normal),
                );

                if ((*clip).traceFlags & G2TRFLAG_GETSURFINDEX) != 0 {
                    // we have requested that surfaceFlags be stomped over with the g2 hit surface index.
                    if (*clip).trace.entityNum == G2Trace[bestTr as usize].mEntityNum {
                        (*clip).trace.surfaceFlags = G2Trace[bestTr as usize].mSurfaceIndex;
                    }
                }
            }
        }
        // #endif
        /*
        Ghoul2 Insert End
        */
        i += 1;
    }
}

/// Helper to initialize G2Trace_t with all entries marked as -1.
#[inline]
unsafe fn memset_g2trace(trace: &mut G2Trace_t) {
    let mut i = 0;
    while i < MAX_G2_COLLISIONS {
        (*trace)[i].mEntityNum = -1;
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
pub unsafe fn SV_Trace(
    results: *mut trace_t,
    start: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    end: *const vec3_t,
    passEntityNum: c_int,
    contentmask: c_int,
    capsule: c_int,
    traceFlags: c_int,
    useLod: c_int,
) {
    let mut clip: moveclip_t;
    let mut i: c_int;

    if mins.is_null() {
        mins = core::ptr::addr_of!(vec3_origin);
    }
    if maxs.is_null() {
        maxs = core::ptr::addr_of!(vec3_origin);
    }

    clip = core::mem::zeroed();

    // clip to world
    CM_BoxTrace(
        core::ptr::addr_of_mut!(clip.trace),
        start,
        end,
        mins,
        maxs,
        0,
        contentmask,
        capsule,
    );
    clip.trace.entityNum = if clip.trace.fraction != 1.0 {
        ENTITYNUM_WORLD
    } else {
        ENTITYNUM_NONE
    };
    if clip.trace.fraction == 0.0 {
        *results = clip.trace;
        return; // blocked immediately by the world
    }

    clip.contentmask = contentmask;
    // Ghoul2 Insert Start
    VectorCopy(start, core::ptr::addr_of_mut!(clip.start));
    clip.traceFlags = traceFlags;
    clip.useLod = useLod;
    // Ghoul2 Insert End
    // VectorCopy( clip.trace.endpos, clip.end );
    VectorCopy(end, core::ptr::addr_of_mut!(clip.end));
    clip.mins = mins;
    clip.maxs = maxs;
    clip.passEntityNum = passEntityNum;
    clip.capsule = capsule;

    // create the bounding box of the entire move
    // we can limit it to the part of the move not
    // already clipped off by the world, which can be
    // a significant savings for line of sight and shot traces
    i = 0;
    while i < 3 {
        if (*end)[i as usize] > (*start)[i as usize] {
            clip.boxmins[i as usize] = clip.start[i as usize] + (*clip.mins)[i as usize] - 1.0;
            clip.boxmaxs[i as usize] = clip.end[i as usize] + (*clip.maxs)[i as usize] + 1.0;
        } else {
            clip.boxmins[i as usize] = clip.end[i as usize] + (*clip.mins)[i as usize] - 1.0;
            clip.boxmaxs[i as usize] = clip.start[i as usize] + (*clip.maxs)[i as usize] + 1.0;
        }
        i += 1;
    }

    // clip to other solid entities
    SV_ClipMoveToEntities(core::ptr::addr_of_mut!(clip));

    *results = clip.trace;
}

/*
=============
SV_PointContents
=============
*/
pub unsafe fn SV_PointContents(p: *const vec3_t, passEntityNum: c_int) -> c_int {
    let mut touch: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
    let mut hit: *mut sharedEntity_t;
    let mut i: c_int;
    let mut num: c_int;
    let mut contents: c_int;
    let mut c2: c_int;
    let mut clipHandle: clipHandle_t;
    let mut angles: *const vec3_t;

    // get base contents from world
    contents = CM_PointContents(p, 0);

    // or in contents from all the other entities
    num = SV_AreaEntities(p, p, touch.as_mut_ptr(), MAX_GENTITIES);

    i = 0;
    while i < num {
        if touch[i as usize] == passEntityNum {
            i += 1;
            continue;
        }
        hit = SV_GentityNum(touch[i as usize]);
        // might intersect, so do an exact clip
        clipHandle = SV_ClipHandleForEntity(hit);
        angles = core::ptr::addr_of!((*hit).s.angles);
        if (*hit).r.bmodel == 0 {
            angles = core::ptr::addr_of!(vec3_origin); // boxes don't rotate
        }

        c2 = CM_TransformedPointContents(p, clipHandle, core::ptr::addr_of!((*hit).s.origin), core::ptr::addr_of!((*hit).s.angles));

        contents |= c2;
        i += 1;
    }

    contents
}
