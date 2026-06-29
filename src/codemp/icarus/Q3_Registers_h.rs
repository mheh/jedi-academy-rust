#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};
use std::collections::BTreeMap;

// Variable type constants
pub const VTYPE_NONE: c_int = 0;
pub const VTYPE_FLOAT: c_int = 1;
pub const VTYPE_STRING: c_int = 2;
pub const VTYPE_VECTOR: c_int = 3;

pub const MAX_VARIABLES: usize = 32;

// Type aliases for C++ std::map equivalents
// Original: map<string, string>
pub type varString_m = BTreeMap<String, String>;
// Original: map<string, float>
pub type varFloat_m = BTreeMap<String, f32>;

// Global variable declarations
// SAFETY: Must be initialized by Q3_InitVariables before use
pub static mut varStrings: Option<varString_m> = None;
pub static mut varFloats: Option<varFloat_m> = None;
pub static mut varVectors: Option<varString_m> = None;

// 3D vector type
pub type vec3_t = [f32; 3];

extern "C" {
    pub fn Q3_InitVariables();
    pub fn Q3_DeclareVariable(r#type: c_int, name: *const c_char);
    pub fn Q3_FreeVariable(name: *const c_char);
    pub fn Q3_GetStringVariable(name: *const c_char, value: *mut *const c_char) -> c_int;
    pub fn Q3_GetFloatVariable(name: *const c_char, value: *mut f32) -> c_int;
    pub fn Q3_GetVectorVariable(name: *const c_char, value: vec3_t) -> c_int;
    pub fn Q3_VariableDeclared(name: *const c_char) -> c_int;
    pub fn Q3_SetFloatVariable(name: *const c_char, value: f32) -> c_int;
    pub fn Q3_SetStringVariable(name: *const c_char, value: *const c_char) -> c_int;
    pub fn Q3_SetVectorVariable(name: *const c_char, value: *const c_char) -> c_int;
}
