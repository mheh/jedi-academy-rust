//! Mechanical port of `codemp/png/png.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_ulong};

// Known chunk types

#[inline]
const fn png_chunk_tag(tag: [u8; 4]) -> c_int {
    ((tag[0] as c_int) << 24)
        | ((tag[1] as c_int) << 16)
        | ((tag[2] as c_int) << 8)
        | (tag[3] as c_int)
}

pub const PNG_IHDR: c_int = png_chunk_tag(*b"IHDR");
pub const PNG_IDAT: c_int = png_chunk_tag(*b"IDAT");
pub const PNG_IEND: c_int = png_chunk_tag(*b"IEND");
pub const PNG_tEXt: c_int = png_chunk_tag(*b"tEXt");

pub const PNG_PLTE: c_int = png_chunk_tag(*b"PLTE");
pub const PNG_bKGD: c_int = png_chunk_tag(*b"bKGD");
pub const PNG_cHRM: c_int = png_chunk_tag(*b"cHRM");
pub const PNG_gAMA: c_int = png_chunk_tag(*b"gAMA");
pub const PNG_hIST: c_int = png_chunk_tag(*b"hIST");
pub const PNG_iCCP: c_int = png_chunk_tag(*b"iCCP");
pub const PNG_iTXt: c_int = png_chunk_tag(*b"iTXt");
pub const PNG_oFFs: c_int = png_chunk_tag(*b"oFFs");
pub const PNG_pCAL: c_int = png_chunk_tag(*b"pCAL");
pub const PNG_sCAL: c_int = png_chunk_tag(*b"sCAL");
pub const PNG_pHYs: c_int = png_chunk_tag(*b"pHYs");
pub const PNG_sBIT: c_int = png_chunk_tag(*b"sBIT");
pub const PNG_sPLT: c_int = png_chunk_tag(*b"sPLT");
pub const PNG_sRGB: c_int = png_chunk_tag(*b"sRGB");
pub const PNG_tIME: c_int = png_chunk_tag(*b"tIME");
pub const PNG_tRNS: c_int = png_chunk_tag(*b"tRNS");
pub const PNG_zTXt: c_int = png_chunk_tag(*b"zTXt");

// Filter values

pub const PNG_FILTER_VALUE_NONE: c_int = 0;
pub const PNG_FILTER_VALUE_SUB: c_int = 1;
pub const PNG_FILTER_VALUE_UP: c_int = 2;
pub const PNG_FILTER_VALUE_AVG: c_int = 3;
pub const PNG_FILTER_VALUE_PAETH: c_int = 4;
pub const PNG_FILTER_NUM: c_int = 5;

// Common defines and typedefs

pub const MAX_PNG_WIDTH: c_int = 4096;
pub const MAX_PNG_DEPTH: c_int = 4;

pub type byte = u8;
pub type word = u16;
pub type ulong = c_ulong;

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
pub struct png_ihdr_t {
    pub width: ulong,
    pub height: ulong,
    pub bitdepth: byte,    // Bits per sample (not per pixel)
    pub colortype: byte,   // bit 0 - palette; bit 1 - RGB; bit 2 - alpha channel
    pub compression: byte, // 0 for zip - error otherwise
    pub filter: byte,      // 0 for adaptive with the 5 basic types - error otherwise
    pub interlace: byte,   // 0 for no interlace - 1 for Adam7 interlace
}

const _: () =
    assert!(core::mem::size_of::<png_ihdr_t>() == core::mem::size_of::<ulong>() * 2 + 5);
const _: () = assert!(core::mem::align_of::<png_ihdr_t>() == 1);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct png_image_t {
    pub data: *mut byte,
    pub width: ulong,
    pub height: ulong,
    pub bytedepth: ulong,
    pub isimage: bool,
}

extern "C" {
    pub fn LoadPNG32(
        name: *mut c_char,
        pixels: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
        bytedepth: *mut c_int,
    ) -> bool;
    pub fn LoadPNG8(
        name: *mut c_char,
        pixels: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
    ) -> bool;
    pub fn PNG_Save(
        name: *const c_char,
        data: *mut byte,
        width: c_int,
        height: c_int,
        bytedepth: c_int,
    ) -> bool;
}

// end
