#![allow(non_snake_case)]

use core::ffi::c_int;

// //#include "../ff/ff_public.h"

// LOCAL STUB: ffHandle_t type from ff_public.h
pub type ffHandle_t = c_int;

extern "C" {
    pub fn FF_AddForce(ff: ffHandle_t); // /*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
    pub fn FF_AddLoopingForce(ff: ffHandle_t); // /*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
    //pub fn FF_Respatialize( int entNum, const vec3_t origin );
    pub fn FF_Update();
}
