#![allow(non_snake_case)]

/*****************************************************************************
 * name:		be_aas_debug.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_debug.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::c_int;

// Stub type for external structure used in this header
// Defined in aasfile.h
pub type vec3_t = [f32; 3];
pub struct aas_reachability_s;

pub extern "C" {
    //clear the shown debug lines
    pub fn AAS_ClearShownDebugLines();
    //
    pub fn AAS_ClearShownPolygons();
    //show a debug line
    pub fn AAS_DebugLine(start: vec3_t, end: vec3_t, color: c_int);
    //show a permenent line
    pub fn AAS_PermanentLine(start: vec3_t, end: vec3_t, color: c_int);
    //show a permanent cross
    pub fn AAS_DrawPermanentCross(origin: vec3_t, size: f32, color: c_int);
    //draw a cross in the plane
    pub fn AAS_DrawPlaneCross(point: vec3_t, normal: vec3_t, dist: f32, type_: c_int, color: c_int);
    //show a bounding box
    pub fn AAS_ShowBoundingBox(origin: vec3_t, mins: vec3_t, maxs: vec3_t);
    //show a face
    pub fn AAS_ShowFace(facenum: c_int);
    //show an area
    pub fn AAS_ShowArea(areanum: c_int, groundfacesonly: c_int);
    //
    pub fn AAS_ShowAreaPolygons(areanum: c_int, color: c_int, groundfacesonly: c_int);
    //draw a cros
    pub fn AAS_DrawCross(origin: vec3_t, size: f32, color: c_int);
    //print the travel type
    pub fn AAS_PrintTravelType(traveltype: c_int);
    //draw an arrow
    pub fn AAS_DrawArrow(start: vec3_t, end: vec3_t, linecolor: c_int, arrowcolor: c_int);
    //visualize the given reachability
    pub fn AAS_ShowReachability(reach: *mut aas_reachability_s);
    //show the reachable areas from the given area
    pub fn AAS_ShowReachableAreas(areanum: c_int);
}
