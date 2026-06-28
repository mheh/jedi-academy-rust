//! Mechanical port of `codemp/mp3code/small_header.h`.
//!
//! This file is just used so I can isolate a few small structs from various horrible MP3 header files without
//! externalising code protos etc. This can now be included by both main game sound code (through sfx_t) and MP3 C code.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;

#[repr(C)]
#[derive(Clone, Copy)]
pub union SAMPLE {
    pub s: c_int,
    pub x: f32,
}

const _: () = assert!(core::mem::size_of::<SAMPLE>() == 4);
const _: () = assert!(core::mem::align_of::<SAMPLE>() == 4);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IN_OUT {
    pub in_bytes: c_int,
    pub out_bytes: c_int,
}

const _: () = assert!(core::mem::size_of::<IN_OUT>() == 8);
const _: () = assert!(core::mem::align_of::<IN_OUT>() == 4);

// WIN32-only typedef from the oracle header.
#[cfg(target_family = "windows")]
pub type byte = u8;

// eof
