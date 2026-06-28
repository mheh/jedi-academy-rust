//! `cm_public.h` — collision model public declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, qboolean, trace_t, vec3_t, vec3pair_t};
use core::ffi::{c_char, c_int};

// Header-local stubs for q_shared.h symbols not yet ported in `q_shared_h.rs`.
pub type clipHandle_t = c_int;

// markfragments are returned by CM_MarkFragments()
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct markFragment_t {
    pub firstPoint: c_int,
    pub numPoints: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct orientation_t {
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
}

// Opaque stubs for declarations from unported qcommon collision headers.
#[repr(C)]
pub struct traceWork_s {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CCMPatch {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn CM_LoadMap(name: *const c_char, clientload: qboolean, checksum: *mut c_int);

    pub fn CM_ClearMap();
    pub fn CM_InlineModel(index: c_int) -> clipHandle_t; // 0 = world, 1 + are bmodels
    pub fn CM_TempBoxModel(mins: *const vec3_t, maxs: *const vec3_t, capsule: c_int) -> clipHandle_t;

    pub fn CM_ModelBounds(model: clipHandle_t, mins: *mut vec3_t, maxs: *mut vec3_t);

    pub fn CM_NumClusters() -> c_int;
    pub fn CM_NumInlineModels() -> c_int;
    pub fn CM_EntityString() -> *mut c_char;

    // returns an ORed contents mask
    pub fn CM_PointContents(p: *const vec3_t, model: clipHandle_t) -> c_int;
    pub fn CM_TransformedPointContents(
        p: *const vec3_t,
        model: clipHandle_t,
        origin: *const vec3_t,
        angles: *const vec3_t,
    ) -> c_int;

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

    pub fn CM_ClusterPVS(cluster: c_int) -> *mut byte;

    pub fn CM_PointLeafnum(p: *const vec3_t) -> c_int;

    // only returns non-solid leafs
    // overflow if return listsize and if *lastLeaf != list[listsize-1]
    pub fn CM_BoxLeafnums(
        mins: *const vec3_t,
        maxs: *const vec3_t,
        boxList: *mut c_int,
        listsize: c_int,
        lastLeaf: *mut c_int,
    ) -> c_int;
    //rwwRMG - changed to boxList to not conflict with list type

    pub fn CM_LeafCluster(leafnum: c_int) -> c_int;
    pub fn CM_LeafArea(leafnum: c_int) -> c_int;

    pub fn CM_AdjustAreaPortalState(area1: c_int, area2: c_int, open: qboolean);
    pub fn CM_AreasConnected(area1: c_int, area2: c_int) -> qboolean;

    pub fn CM_WriteAreaBits(buffer: *mut byte, area: c_int) -> c_int;

    //rwwRMG - added:
    pub fn CM_GenericBoxCollide(abounds: *const vec3pair_t, bbounds: *const vec3pair_t) -> bool;
    pub fn CM_HandlePatchCollision(
        tw: *mut traceWork_s,
        trace: *mut trace_t,
        tStart: *const vec3_t,
        tEnd: *const vec3_t,
        patch: *mut CCMPatch,
        checkcount: c_int,
    );
    pub fn CM_CalcExtents(
        start: *const vec3_t,
        end: *const vec3_t,
        tw: *const traceWork_s,
        bounds: *mut vec3pair_t,
    );

    // cm_tag.c
    pub fn CM_LerpTag(
        tag: *mut orientation_t,
        model: clipHandle_t,
        startFrame: c_int,
        endFrame: c_int,
        frac: f32,
        tagName: *const c_char,
    ) -> c_int;

    // cm_marks.c
    pub fn CM_MarkFragments(
        numPoints: c_int,
        points: *const vec3_t,
        projection: *const vec3_t,
        maxPoints: c_int,
        pointBuffer: *mut vec3_t,
        maxFragments: c_int,
        fragmentBuffer: *mut markFragment_t,
    ) -> c_int;

    // cm_patch.c
    pub fn CM_DrawDebugSurface(drawPoly: Option<unsafe extern "C" fn(c_int, c_int, *mut f32)>);

    // cm_shader.cpp
    pub fn CM_GetShaderText(key: *const c_char) -> *const c_char;
    pub fn CM_FreeShaderText();
    pub fn CM_LoadShaderText(forceReload: qboolean);
}
