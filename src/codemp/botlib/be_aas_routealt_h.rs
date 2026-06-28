/*****************************************************************************
 * name:		be_aas_routealt.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_routealt.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::c_int;

// LOCAL STUB: vec3_t (from q_shared.h)
pub type vec3_t = [f32; 3];

// LOCAL STUB: aas_altroutegoal_t (struct defined elsewhere)
#[repr(C)]
pub struct aas_altroutegoal_t {
    // Stub structure - actual definition provided by codemp/botlib module
}

#[cfg(feature = "aas_intern")]
extern "C" {
    pub fn AAS_InitAlternativeRouting();
    pub fn AAS_ShutdownAlternativeRouting();
}

extern "C" {
    pub fn AAS_AlternativeRouteGoals(
        start: *const vec3_t,
        startareanum: c_int,
        goal: *const vec3_t,
        goalareanum: c_int,
        travelflags: c_int,
        altroutegoals: *mut aas_altroutegoal_t,
        maxaltroutegoals: c_int,
        r#type: c_int,
    ) -> c_int;
}
