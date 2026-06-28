//! `be_ai_gen.h` — genetic selection botlib declaration.

#![allow(non_snake_case)]

use core::ffi::c_int;

unsafe extern "C" {
    pub fn GeneticParentsAndChildSelection(
        numranks: c_int,
        ranks: *mut f32,
        parent1: *mut c_int,
        parent2: *mut c_int,
        child: *mut c_int,
    ) -> c_int;
}
