//! Mechanical port of `codemp/qcommon/cm_test.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{
    cplane_t, qboolean, vec3_t, PLANE_NON_AXIAL, ERR_DROP,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_TERRAIN;
use crate::codemp::qcommon::cm_local_h::{
    leafList_t, clipMap_t, cNode_t, cLeaf_t, cbrush_t, cmodel_t, clipHandle_t,
    cArea_t, cmg, c_pointcontents, cm_noAreas, BOX_MODEL_HANDLE,
};
use crate::ffi::types::{QTRUE, QFALSE};
use core::ffi::c_int;
use core::ptr::addr_of_mut;

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "cm_local.h"
// #ifdef _XBOX
// #include "../renderer/tr_local.h"
// #endif

extern "C" {
    // extern functions from q_math.c
    pub fn DotProduct(v1: &vec3_t, v2: &vec3_t) -> f32;
    pub fn VectorCopy(src: &vec3_t, dst: &mut vec3_t);
    pub fn VectorSubtract(veca: &vec3_t, vecb: &vec3_t, out: &mut vec3_t);
    pub fn BoxOnPlaneSide(emins: &vec3_t, emaxs: &vec3_t, p: &cplane_t) -> c_int;
    pub fn AngleVectors(angles: &vec3_t, forward: &mut vec3_t, right: &mut vec3_t, up: &mut vec3_t);

    // extern functions from common.c
    pub fn Com_Error(code: c_int, fmt: *const core::ffi::c_char, ...);
    pub fn Com_Memset(dst: *mut core::ffi::c_void, c: c_int, count: usize);
}

/*
==================
CM_PointLeafnum_r

==================
*/
pub unsafe fn CM_PointLeafnum_r(p: &vec3_t, mut num: c_int, local: &mut clipMap_t) -> c_int {
    let mut d: f32;
    let node: *mut cNode_t;
    let plane: *mut cplane_t;

    while num >= 0 {
        node = (*local).nodes.add(num as usize);

        #[cfg(feature = "xbox")]
        {
            plane = (*cmg).planes.add((*node).planeNum as usize);
        }
        #[cfg(not(feature = "xbox"))]
        {
            plane = (*node).plane;
        }

        if (*plane).r#type < PLANE_NON_AXIAL as u8 {
            d = p[(*plane).r#type as usize] - (*plane).dist;
        } else {
            d = DotProduct(p, &(*plane).normal) - (*plane).dist;
        }
        if d < 0.0 {
            num = (*node).children[1];
        } else {
            num = (*node).children[0];
        }
    }

    c_pointcontents += 1; // optimize counter

    -1 - num
}

pub unsafe fn CM_PointLeafnum(p: &vec3_t) -> c_int {
    if (*cmg).numNodes == 0 {
        // map not loaded
        return 0;
    }
    CM_PointLeafnum_r(p, 0, &mut *cmg)
}

/*
======================================================================

LEAF LISTING

======================================================================
*/

pub unsafe fn CM_StoreLeafs(ll: &mut leafList_t, nodenum: c_int) {
    let leafNum: c_int;

    leafNum = -1 - nodenum;

    // store the lastLeaf even if the list is overflowed
    if (*(*cmg).leafs.add(leafNum as usize)).cluster != -1 {
        ll.lastLeaf = leafNum;
    }

    if ll.count >= ll.maxcount {
        ll.overflowed = QTRUE;
        return;
    }
    *(ll.list.add(ll.count as usize)) = leafNum;
    ll.count += 1;
}

pub unsafe fn CM_StoreBrushes(ll: &mut leafList_t, nodenum: c_int) {
    let mut i: c_int;
    let mut k: c_int;
    let mut leafnum: c_int;
    let mut brushnum: c_int;
    let leaf: *mut cLeaf_t;
    let b: *mut cbrush_t;

    leafnum = -1 - nodenum;

    leaf = (*cmg).leafs.add(leafnum as usize);

    k = 0;
    while k < (*leaf).numLeafBrushes {
        brushnum = *((*cmg).leafbrushes.add(((*leaf).firstLeafBrush + k) as usize));
        b = (*cmg).brushes.add(brushnum as usize);
        if (*b).checkcount == (*cmg).checkcount as u16 {
            k += 1;
            continue; // already checked this brush in another leaf
        }
        (*b).checkcount = (*cmg).checkcount as u16;
        i = 0;
        while i < 3 {
            if (*b).bounds[0][i as usize] >= ll.bounds[1][i as usize]
                || (*b).bounds[1][i as usize] <= ll.bounds[0][i as usize]
            {
                break;
            }
            i += 1;
        }
        if i != 3 {
            k += 1;
            continue;
        }
        if ll.count >= ll.maxcount {
            ll.overflowed = QTRUE;
            return;
        }
        let list_as_brush_ptr_ptr = ll.list as *mut *mut cbrush_t;
        *list_as_brush_ptr_ptr.add(ll.count as usize) = b;
        ll.count += 1;
        k += 1;
    }
    // #if 0
    // // store patches?
    // for ( k = 0 ; k < leaf->numLeafSurfaces ; k++ ) {
    //     patch = cm.surfaces[ cm.leafsurfaces[ leaf->firstleafsurface + k ] ];
    //     if ( !patch ) {
    //         continue;
    //     }
    // }
    // #endif
}

/*
=============
CM_BoxLeafnums

Fills in a list of all the leafs touched
=============
*/
pub unsafe fn CM_BoxLeafnums_r(ll: *mut leafList_t, mut nodenum: c_int) {
    let plane: *mut cplane_t;
    let node: *mut cNode_t;
    let mut s: c_int;

    loop {
        if nodenum < 0 {
            if let Some(storeLeafs) = (*ll).storeLeafs {
                storeLeafs(ll, nodenum);
            }
            return;
        }

        node = (*cmg).nodes.add(nodenum as usize);

        #[cfg(feature = "xbox")]
        {
            plane = (*cmg).planes.add((*node).planeNum as usize);
        }
        #[cfg(not(feature = "xbox"))]
        {
            plane = (*node).plane;
        }

        s = BoxOnPlaneSide(&(*ll).bounds[0], &(*ll).bounds[1], &*plane);
        if s == 1 {
            nodenum = (*node).children[0];
        } else if s == 2 {
            nodenum = (*node).children[1];
        } else {
            // go down both
            CM_BoxLeafnums_r(ll, (*node).children[0]);
            nodenum = (*node).children[1];
        }
    }
}

/*
==================
CM_BoxLeafnums
==================
*/
pub unsafe fn CM_BoxLeafnums(
    mins: &vec3_t,
    maxs: &vec3_t,
    boxList: *mut c_int,
    listsize: c_int,
    lastLeaf: *mut c_int,
) -> c_int {
    //rwwRMG - changed to boxList to not conflict with list type
    let mut ll: leafList_t = core::mem::zeroed();

    (*cmg).checkcount += 1;

    VectorCopy(mins, &mut ll.bounds[0]);
    VectorCopy(maxs, &mut ll.bounds[1]);
    ll.count = 0;
    ll.maxcount = listsize;
    ll.list = boxList;
    ll.storeLeafs = Some(CM_StoreLeafs);
    ll.lastLeaf = 0;
    ll.overflowed = QFALSE;

    CM_BoxLeafnums_r(&mut ll as *mut _, 0);

    *lastLeaf = ll.lastLeaf;
    ll.count
}

/*
==================
CM_BoxBrushes
==================
*/
pub unsafe fn CM_BoxBrushes(
    mins: &vec3_t,
    maxs: &vec3_t,
    boxList: *mut *mut cbrush_t,
    listsize: c_int,
) -> c_int {
    //rwwRMG - changed to boxList to not conflict with list type
    let mut ll: leafList_t = core::mem::zeroed();

    (*cmg).checkcount += 1;

    VectorCopy(mins, &mut ll.bounds[0]);
    VectorCopy(maxs, &mut ll.bounds[1]);
    ll.count = 0;
    ll.maxcount = listsize;
    ll.list = boxList as *mut c_int;
    ll.storeLeafs = Some(CM_StoreBrushes);
    ll.lastLeaf = 0;
    ll.overflowed = QFALSE;

    CM_BoxLeafnums_r(&mut ll as *mut _, 0);

    ll.count
}

//====================================================================

/*
==================
CM_PointContents

==================
*/
pub unsafe fn CM_PointContents(p: &vec3_t, model: clipHandle_t) -> c_int {
    let mut leafnum: c_int;
    let mut i: c_int;
    let mut k: c_int;
    let mut brushnum: c_int;
    let leaf: *mut cLeaf_t;
    let b: *mut cbrush_t;
    let mut contents: c_int;
    let mut d: f32;
    let clipm: *mut cmodel_t;
    let mut local: *mut clipMap_t;

    if (*cmg).numNodes == 0 {
        // map not loaded
        return 0;
    }

    if !model.is_null() {
        clipm = CM_ClipHandleToModel(model, &mut local);
        if (*clipm).firstNode != -1 {
            leafnum = CM_PointLeafnum_r(p, 0, &mut *local);
            leaf = (*local).leafs.add(leafnum as usize);
        } else {
            leaf = &mut (*clipm).leaf;
        }
    } else {
        local = addr_of_mut!(cmg);
        leafnum = CM_PointLeafnum_r(p, 0, &mut *cmg);
        leaf = (*local).leafs.add(leafnum as usize);
    }

    contents = 0;
    k = 0;
    while k < (*leaf).numLeafBrushes {
        brushnum = *((*local).leafbrushes.add(((*leaf).firstLeafBrush + k) as usize));
        b = (*local).brushes.add(brushnum as usize);

        // see if the point is in the brush
        i = 0;
        while i < (*b).numsides as c_int {
            #[cfg(feature = "xbox")]
            {
                d = DotProduct(p, &(*(*cmg).planes.add((*(*b).sides.add(i as usize)).planeNum as usize)).normal);
            }
            #[cfg(not(feature = "xbox"))]
            {
                d = DotProduct(p, &(*(*(*b).sides.add(i as usize)).plane).normal);
            }
            // FIXME test for Cash
            //			if ( d >= b->sides[i].plane->dist ) {
            #[cfg(feature = "xbox")]
            {
                if d > (*(*cmg).planes.add((*(*b).sides.add(i as usize)).planeNum as usize)).dist {
                    break;
                }
            }
            #[cfg(not(feature = "xbox"))]
            {
                if d > (*(*(*b).sides.add(i as usize)).plane).dist {
                    break;
                }
            }
            i += 1;
        }

        if i == (*b).numsides as c_int {
            contents |= (*b).contents;
            if !(*cmg).landScape.is_null() && (contents & CONTENTS_TERRAIN) != 0 {
                if p[2] < (*(*cmg).landScape).GetWaterHeight() {
                    contents |= (*(*cmg).landScape).GetWaterContents();
                }
            }
        }
        k += 1;
    }

    contents
}

/*
==================
CM_TransformedPointContents

Handles offseting and rotation of the end points for moving and
rotating entities
==================
*/
pub unsafe fn CM_TransformedPointContents(
    p: &vec3_t,
    model: clipHandle_t,
    origin: &vec3_t,
    angles: &vec3_t,
) -> c_int {
    let mut p_l: vec3_t = [0.0; 3];
    let mut temp: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    // subtract origin offset
    VectorSubtract(p, origin, &mut p_l);

    // rotate start and end into the models frame of reference
    if model != BOX_MODEL_HANDLE && (angles[0] != 0.0 || angles[1] != 0.0 || angles[2] != 0.0) {
        AngleVectors(angles, &mut forward, &mut right, &mut up);

        VectorCopy(&p_l, &mut temp);
        p_l[0] = DotProduct(&temp, &forward);
        p_l[1] = -DotProduct(&temp, &right);
        p_l[2] = DotProduct(&temp, &up);
    }

    CM_PointContents(&p_l, model)
}

/*
===============================================================================

PVS

===============================================================================
*/
#[cfg(feature = "xbox")]
extern "C" {
    pub static tr: trGlobals_t;
}

#[cfg(feature = "xbox")]
pub struct trGlobals_t {
    // stub - only used for Xbox
}

#[cfg(feature = "xbox")]
pub unsafe fn CM_ClusterPVS(cluster: c_int) -> *const u8 {
    if cluster < 0 || cluster >= (*cmg).numClusters || (*cmg).vised == QFALSE {
        return core::ptr::null();
    }

    // XBOX TODO: This would need to call Decompress on a C++ object
    // Stub implementation for now
    core::ptr::null()
}

#[cfg(not(feature = "xbox"))]
pub unsafe fn CM_ClusterPVS(cluster: c_int) -> *mut u8 {
    if cluster < 0 || cluster >= (*cmg).numClusters || (*cmg).vised == QFALSE {
        return (*cmg).visibility;
    }

    (*cmg).visibility.add((cluster * (*cmg).clusterBytes) as usize)
}

/*
===============================================================================

AREAPORTALS

===============================================================================
*/
#[cfg(feature = "xbox")]
pub unsafe fn CM_FloodArea_r(areaNum: c_int, floodnum: c_int) {
    let mut i: c_int;
    let area: *mut cArea_t;
    let con: *mut c_int;

    area = (*cmg).areas.add(areaNum as usize);

    if (*area).floodvalid == (*cmg).floodvalid {
        if (*area).floodnum == floodnum {
            return;
        }
        Com_Error(ERR_DROP, b"FloodArea_r: reflooded\0".as_ptr() as *const core::ffi::c_char);
    }

    (*area).floodnum = floodnum;
    (*area).floodvalid = (*cmg).floodvalid;
    con = (*cmg).areaPortals.add((areaNum * (*cmg).numAreas) as usize);
    i = 0;
    while i < (*cmg).numAreas {
        if *con.add(i as usize) > 0 {
            CM_FloodArea_r(i, floodnum);
        }
        i += 1;
    }
}

#[cfg(not(feature = "xbox"))]
pub unsafe fn CM_FloodArea_r(areaNum: c_int, floodnum: c_int, cm: &mut clipMap_t) {
    let mut i: c_int;
    let area: *mut cArea_t;
    let con: *mut c_int;

    area = cm.areas.add(areaNum as usize);

    if (*area).floodvalid == cm.floodvalid {
        if (*area).floodnum == floodnum {
            return;
        }
        Com_Error(ERR_DROP, b"FloodArea_r: reflooded\0".as_ptr() as *const core::ffi::c_char);
    }

    (*area).floodnum = floodnum;
    (*area).floodvalid = cm.floodvalid;
    con = cm.areaPortals.add((areaNum * cm.numAreas) as usize);
    i = 0;
    while i < cm.numAreas {
        if *con.add(i as usize) > 0 {
            CM_FloodArea_r(i, floodnum, cm);
        }
        i += 1;
    }
}

/*
====================
CM_FloodAreaConnections

====================
*/
#[cfg(feature = "xbox")]
pub unsafe fn CM_FloodAreaConnections() {
    let mut i: c_int;
    let area: *mut cArea_t;
    let mut floodnum: c_int;

    // all current floods are now invalid
    (*cmg).floodvalid += 1;
    floodnum = 0;

    i = 0;
    while i < (*cmg).numAreas {
        area = (*cmg).areas.add(i as usize);
        if (*area).floodvalid == (*cmg).floodvalid {
            i += 1;
            continue; // already flooded into
        }
        floodnum += 1;
        CM_FloodArea_r(i, floodnum);
        i += 1;
    }
}

#[cfg(not(feature = "xbox"))]
pub unsafe fn CM_FloodAreaConnections(cm: &mut clipMap_t) {
    let mut i: c_int;
    let area: *mut cArea_t;
    let mut floodnum: c_int;

    // all current floods are now invalid
    cm.floodvalid += 1;
    floodnum = 0;

    i = 0;
    while i < cm.numAreas {
        area = cm.areas.add(i as usize);
        if (*area).floodvalid == cm.floodvalid {
            i += 1;
            continue; // already flooded into
        }
        floodnum += 1;
        CM_FloodArea_r(i, floodnum, cm);
        i += 1;
    }
}

/*
====================
CM_AdjustAreaPortalState

====================
*/
pub unsafe fn CM_AdjustAreaPortalState(area1: c_int, area2: c_int, open: qboolean) {
    if area1 < 0 || area2 < 0 {
        return;
    }

    if area1 >= (*cmg).numAreas || area2 >= (*cmg).numAreas {
        Com_Error(
            ERR_DROP,
            b"CM_ChangeAreaPortalState: bad area number\0".as_ptr() as *const core::ffi::c_char,
        );
    }

    if open != QFALSE {
        *(*cmg)
            .areaPortals
            .add((area1 * (*cmg).numAreas + area2) as usize) += 1;
        *(*cmg)
            .areaPortals
            .add((area2 * (*cmg).numAreas + area1) as usize) += 1;
    } else {
        *(*cmg)
            .areaPortals
            .add((area1 * (*cmg).numAreas + area2) as usize) -= 1;
        *(*cmg)
            .areaPortals
            .add((area2 * (*cmg).numAreas + area1) as usize) -= 1;
        if *(*cmg).areaPortals.add((area2 * (*cmg).numAreas + area1) as usize) < 0 {
            Com_Error(
                ERR_DROP,
                b"CM_AdjustAreaPortalState: negative reference count\0".as_ptr()
                    as *const core::ffi::c_char,
            );
        }
    }

    #[cfg(feature = "xbox")]
    {
        CM_FloodAreaConnections();
    }
    #[cfg(not(feature = "xbox"))]
    {
        CM_FloodAreaConnections(&mut *cmg);
    }
}

/*
====================
CM_AreasConnected

====================
*/
pub unsafe fn CM_AreasConnected(area1: c_int, area2: c_int) -> qboolean {
    #[cfg(not(feature = "bspc"))]
    {
        if !cm_noAreas.is_null() && (*cm_noAreas).integer != 0 {
            return QTRUE;
        }
    }

    if area1 < 0 || area2 < 0 {
        return QFALSE;
    }

    if area1 >= (*cmg).numAreas || area2 >= (*cmg).numAreas {
        Com_Error(ERR_DROP, b"area >= cmg.numAreas\0".as_ptr() as *const core::ffi::c_char);
    }

    if (*cmg).areas[area1 as usize].floodnum == (*cmg).areas[area2 as usize].floodnum {
        return QTRUE;
    }
    QFALSE
}

/*
=================
CM_WriteAreaBits

Writes a bit vector of all the areas
that are in the same flood as the area parameter
Returns the number of bytes needed to hold all the bits.

The bits are OR'd in, so you can CM_WriteAreaBits from multiple
viewpoints and get the union of all visible areas.

This is used to cull non-visible entities from snapshots
=================
*/
pub unsafe fn CM_WriteAreaBits(buffer: *mut u8, area: c_int) -> c_int {
    let mut i: c_int;
    let mut floodnum: c_int;
    let mut bytes: c_int;

    bytes = ((*cmg).numAreas + 7) >> 3;

    #[cfg(not(feature = "bspc"))]
    {
        if (*cm_noAreas).integer != 0 || area == -1
        {
            // for debugging, send everything
            Com_Memset(buffer as *mut core::ffi::c_void, 255, bytes as usize);
        } else {
            floodnum = (*cmg).areas[area as usize].floodnum;
            i = 0;
            while i < (*cmg).numAreas {
                if (*cmg).areas[i as usize].floodnum == floodnum || area == -1 {
                    *buffer.add((i >> 3) as usize) |= (1 << (i & 7)) as u8;
                }
                i += 1;
            }
        }
    }

    #[cfg(feature = "bspc")]
    {
        if area == -1 {
            // for debugging, send everything
            Com_Memset(buffer as *mut core::ffi::c_void, 255, bytes as usize);
        } else {
            floodnum = (*cmg).areas[area as usize].floodnum;
            i = 0;
            while i < (*cmg).numAreas {
                if (*cmg).areas[i as usize].floodnum == floodnum || area == -1 {
                    *buffer.add((i >> 3) as usize) |= (1 << (i & 7)) as u8;
                }
                i += 1;
            }
        }
    }

    bytes
}
