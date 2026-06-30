// Porting note: the previous blind-port stub fabricated struct definitions and opaque
// placeholders for external types; all of those have been removed.  External types are
// imported from their canonical modules per the triage instructions.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// #include "../game/q_shared.h"  (triage-resolved to qcommon mirror)
use crate::codemp::qcommon::q_shared_h::*;
// #include "qfiles.h"
use crate::codemp::qcommon::qfiles_h::*;
// traceWork_s lives in cm_local.h (referenced but not #included in the C header)
use crate::codemp::qcommon::cm_local_h::*;
// CCMPatch lives in cm_landscape.h (referenced but not #included in the C header)
use crate::codemp::qcommon::cm_landscape_h::*;
use core::ffi::{c_char, c_int};

// CM_LoadMap has identical signatures in the #ifdef _XBOX and #else arms;
// declared once here (duplicate extern items do not parse in Rust).

// CM_ClusterPVS differs in const-ness of the return pointer between the
// #ifdef _XBOX arm (const byte *) and the #else arm (byte *); gated below.
#[cfg(feature = "xbox")]
unsafe extern "C" {
    pub fn CM_ClusterPVS(cluster: c_int) -> *const byte;
}

#[cfg(not(feature = "xbox"))]
unsafe extern "C" {
    pub fn CM_ClusterPVS(cluster: c_int) -> *mut byte;
}

unsafe extern "C" {
    pub fn CM_LoadMap(name: *const c_char, clientload: qboolean, checksum: *mut c_int);

    pub fn CM_ClearMap();
    pub fn CM_InlineModel(index: c_int) -> clipHandle_t; // 0 = world, 1 + are bmodels
    pub fn CM_TempBoxModel(
        mins: *const vec3_t,
        maxs: *const vec3_t,
        capsule: c_int,
    ) -> clipHandle_t;

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
    // Porting note: `trace_t &trace` is a C++ reference; translated as *mut trace_t.
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
    // Porting note: `const vec3_t projection` and `vec3_t pointBuffer` are array
    // parameters; in C they decay to pointers, translated as *const/*mut vec3_t.
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
