// Filename: tr_stl.h
//
//  I had to make this new file, because if I put the STL "map" include inside tr_local.h then one of the other header
//  files got compile errors because of using "map" in the function protos as a GLEnum, this way seemed simpler...

use core::ffi::{c_char, c_int};

// REM this out if you want to compile without using STL (but slower of course)
//

#[cfg(feature = "use_stl_for_shader_lookups")]
extern "C" {
    pub fn ShaderEntryPtrs_Clear();
    pub fn ShaderEntryPtrs_Size() -> c_int;
    pub fn ShaderEntryPtrs_Lookup(psShaderName: *const c_char) -> *const c_char;
    pub fn ShaderEntryPtrs_Insert(token: *const c_char, p: *const c_char);
}

#[cfg(not(feature = "use_stl_for_shader_lookups"))]
#[inline(always)]
pub fn ShaderEntryPtrs_Clear() {
    // Macro expansion: empty
}
