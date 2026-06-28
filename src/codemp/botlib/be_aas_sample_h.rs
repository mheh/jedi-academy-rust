
/*****************************************************************************
 * name:		be_aas_sample.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_sample.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::c_int;

// Forward declare AAS types
pub enum aas_face_t {}
pub enum aas_plane_t {}
pub enum aas_link_t {}
pub enum aas_trace_t {}
pub enum aas_areainfo_t {}

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

#[cfg(feature = "AASINTERN")]
extern "C" {
    pub fn AAS_InitAASLinkHeap();
    pub fn AAS_InitAASLinkedEntities();
    pub fn AAS_FreeAASLinkHeap();
    pub fn AAS_FreeAASLinkedEntities();
    pub fn AAS_AreaGroundFace(areanum: c_int, point: vec3_t) -> *mut aas_face_t;
    pub fn AAS_TraceEndFace(trace: *mut aas_trace_t) -> *mut aas_face_t;
    pub fn AAS_PlaneFromNum(planenum: c_int) -> *mut aas_plane_t;
    pub fn AAS_AASLinkEntity(absmins: vec3_t, absmaxs: vec3_t, entnum: c_int) -> *mut aas_link_t;
    pub fn AAS_LinkEntityClientBBox(absmins: vec3_t, absmaxs: vec3_t, entnum: c_int, presencetype: c_int) -> *mut aas_link_t;
    pub fn AAS_PointInsideFace(facenum: c_int, point: vec3_t, epsilon: f32) -> qboolean;
    pub fn AAS_InsideFace(face: *mut aas_face_t, pnormal: vec3_t, point: vec3_t, epsilon: f32) -> qboolean;
    pub fn AAS_UnlinkFromAreas(areas: *mut aas_link_t);
}

// returns the mins and maxs of the bounding box for the given presence type
pub extern "C" fn AAS_PresenceTypeBoundingBox(presencetype: c_int, mins: *mut vec3_t, maxs: *mut vec3_t);
// returns the cluster the area is in (negative portal number if the area is a portal)
pub extern "C" fn AAS_AreaCluster(areanum: c_int) -> c_int;
// returns the presence type(s) of the area
pub extern "C" fn AAS_AreaPresenceType(areanum: c_int) -> c_int;
// returns the presence type(s) at the given point
pub extern "C" fn AAS_PointPresenceType(point: vec3_t) -> c_int;
// returns the result of the trace of a client bbox
pub extern "C" fn AAS_TraceClientBBox(start: vec3_t, end: vec3_t, presencetype: c_int, passent: c_int) -> aas_trace_t;
// stores the areas the trace went through and returns the number of passed areas
pub extern "C" fn AAS_TraceAreas(start: vec3_t, end: vec3_t, areas: *mut c_int, points: *mut vec3_t, maxareas: c_int) -> c_int;
// returns the areas the bounding box is in
pub extern "C" fn AAS_BBoxAreas(absmins: vec3_t, absmaxs: vec3_t, areas: *mut c_int, maxareas: c_int) -> c_int;
// return area information
pub extern "C" fn AAS_AreaInfo(areanum: c_int, info: *mut aas_areainfo_t) -> c_int;
// returns the area the point is in
pub extern "C" fn AAS_PointAreaNum(point: vec3_t) -> c_int;
//
pub extern "C" fn AAS_PointReachabilityAreaIndex(point: vec3_t) -> c_int;
// returns the plane the given face is in
pub extern "C" fn AAS_FacePlane(facenum: c_int, normal: *mut vec3_t, dist: *mut f32);
