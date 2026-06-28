#![allow(non_snake_case)]

pub const MAX_INSTANCE_TRIES: i32 = 5;

// on a symmetric map which corner is the first node
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum symmetry_t {
    SYMMETRY_NONE = 0,
    SYMMETRY_TOPLEFT = 1,
    SYMMETRY_BOTTOMRIGHT = 2,
}
