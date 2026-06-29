#![allow(non_camel_case_types)]

pub const MAX_INSTANCE_TRIES: i32 = 5;

// on a symmetric map which corner is the first node
#[repr(C)]
pub enum symmetry_t {
    SYMMETRY_NONE,
    SYMMETRY_TOPLEFT,
    SYMMETRY_BOTTOMRIGHT,
}
