#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// Opaque type stubs for types defined in qfiles.h
// These will be properly defined once qfiles_h.rs is ported
#[repr(C)]
pub struct trace_t(c_void);

#[repr(C)]
pub struct markFragment_t(c_void);

// Type aliases from qfiles.h equivalents
// In C: typedef int qboolean; typedef float vec3_t[3]; typedef int clipHandle_t; typedef unsigned char byte;
type qboolean = c_int;
type clipHandle_t = c_int;
type vec3_t = [f32; 3];
type byte = u8;

// Platform-independent declarations
extern "C" {
    // qboolean CM_DeleteCachedMap(qboolean bGuaranteedOkToDelete);
    pub fn CM_DeleteCachedMap(bGuaranteedOkToDelete: qboolean) -> qboolean;

    // void CM_ClearMap( void );
    pub fn CM_ClearMap();

    // int CM_TotalMapContents();
    pub fn CM_TotalMapContents() -> c_int;

    // clipHandle_t CM_InlineModel( int index );
    // 0 = world, 1 + are bmodels
    pub fn CM_InlineModel(index: c_int) -> clipHandle_t;

    // clipHandle_t CM_TempBoxModel( const vec3_t mins, const vec3_t maxs );
    // , const int contents );
    pub fn CM_TempBoxModel(mins: *const vec3_t, maxs: *const vec3_t) -> clipHandle_t;

    // int CM_ModelContents( clipHandle_t model, int subBSPIndex );
    pub fn CM_ModelContents(model: clipHandle_t, subBSPIndex: c_int) -> c_int;

    // int CM_NumClusters (void);
    pub fn CM_NumClusters() -> c_int;

    // int CM_NumInlineModels( void );
    pub fn CM_NumInlineModels() -> c_int;

    // char *CM_EntityString (void);
    pub fn CM_EntityString() -> *mut c_char;

    // char *CM_SubBSPEntityString (int index);
    pub fn CM_SubBSPEntityString(index: c_int) -> *mut c_char;

    // int CM_LoadSubBSP(const char *name, qboolean clientload);
    pub fn CM_LoadSubBSP(name: *const c_char, clientload: qboolean) -> c_int;

    // int CM_FindSubBSP(int modelIndex);
    pub fn CM_FindSubBSP(modelIndex: c_int) -> c_int;

    // returns an ORed contents mask
    // int CM_PointContents( const vec3_t p, clipHandle_t model );
    pub fn CM_PointContents(p: *const vec3_t, model: clipHandle_t) -> c_int;

    // int CM_TransformedPointContents( const vec3_t p, clipHandle_t model, const vec3_t origin, const vec3_t angles );
    pub fn CM_TransformedPointContents(
        p: *const vec3_t,
        model: clipHandle_t,
        origin: *const vec3_t,
        angles: *const vec3_t,
    ) -> c_int;

    // void CM_BoxTrace ( trace_t *results, const vec3_t start, const vec3_t end,
    //                    const vec3_t mins, const vec3_t maxs,
    //                    clipHandle_t model, int brushmask);
    pub fn CM_BoxTrace(
        results: *mut trace_t,
        start: *const vec3_t,
        end: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        model: clipHandle_t,
        brushmask: c_int,
    );

    // void CM_TransformedBoxTrace( trace_t *results, const vec3_t start, const vec3_t end,
    //                              const vec3_t mins, const vec3_t maxs,
    //                              clipHandle_t model, int brushmask,
    //                              const vec3_t origin, const vec3_t angles);
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
    );

    // int CM_PointLeafnum( const vec3_t p );
    pub fn CM_PointLeafnum(p: *const vec3_t) -> c_int;

    // only returns non-solid leafs
    // overflow if return listsize and if *lastLeaf != list[listsize-1]
    // int CM_BoxLeafnums( const vec3_t mins, const vec3_t maxs, int *boxList,
    //                      int listsize, int *lastLeaf );
    pub fn CM_BoxLeafnums(
        mins: *const vec3_t,
        maxs: *const vec3_t,
        boxList: *mut c_int,
        listsize: c_int,
        lastLeaf: *mut c_int,
    ) -> c_int;

    // int CM_LeafCluster (int leafnum);
    pub fn CM_LeafCluster(leafnum: c_int) -> c_int;

    // int CM_LeafArea (int leafnum);
    pub fn CM_LeafArea(leafnum: c_int) -> c_int;

    // void CM_AdjustAreaPortalState( int area1, int area2, qboolean open );
    pub fn CM_AdjustAreaPortalState(area1: c_int, area2: c_int, open: qboolean);

    // qboolean CM_AreasConnected( int area1, int area2 );
    pub fn CM_AreasConnected(area1: c_int, area2: c_int) -> qboolean;

    // int CM_WriteAreaBits( byte *buffer, int area );
    pub fn CM_WriteAreaBits(buffer: *mut byte, area: c_int) -> c_int;

    // for savegames
    // void CM_WritePortalState ();
    pub fn CM_WritePortalState();

    // void CM_ReadPortalState ();
    pub fn CM_ReadPortalState();

    // cm_marks.c
    // int CM_MarkFragments( int numPoints, const vec3_t *points, const vec3_t projection,
    //                      int maxPoints, vec3_t pointBuffer, int maxFragments, markFragment_t *fragmentBuffer );
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
    // void CM_DrawDebugSurface( void (*drawPoly)(int color, int numPoints, float *points) );
    pub fn CM_DrawDebugSurface(drawPoly: Option<extern "C" fn(c_int, c_int, *mut f32)>);
}

// Platform-specific: Xbox variant
#[cfg(target_os = "xbox")]
extern "C" {
    // void CM_LoadMap( const char *name, qboolean clientload, int *checksum);
    pub fn CM_LoadMap(name: *const c_char, clientload: qboolean, checksum: *mut c_int);

    // const byte *CM_ClusterPVS (int cluster);
    pub fn CM_ClusterPVS(cluster: c_int) -> *const byte;
}

// Platform-specific: Non-Xbox variant (default)
#[cfg(not(target_os = "xbox"))]
extern "C" {
    // void CM_LoadMap( const char *name, qboolean clientload, int *checksum, qboolean subBSP);
    pub fn CM_LoadMap(name: *const c_char, clientload: qboolean, checksum: *mut c_int, subBSP: qboolean);

    // byte *CM_ClusterPVS (int cluster);
    pub fn CM_ClusterPVS(cluster: c_int) -> *mut byte;
}
