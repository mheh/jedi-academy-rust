use core::ffi::{c_int, c_char, c_ulong, c_bool};

// Known chunk types

pub const PNG_IHDR: u32 = u32::from_be_bytes([b'I', b'H', b'D', b'R']);
pub const PNG_IDAT: u32 = u32::from_be_bytes([b'I', b'D', b'A', b'T']);
pub const PNG_IEND: u32 = u32::from_be_bytes([b'I', b'E', b'N', b'D']);
pub const PNG_tEXt: u32 = u32::from_be_bytes([b't', b'E', b'X', b't']);

pub const PNG_PLTE: u32 = u32::from_be_bytes([b'P', b'L', b'T', b'E']);
pub const PNG_bKGD: u32 = u32::from_be_bytes([b'b', b'K', b'G', b'D']);
pub const PNG_cHRM: u32 = u32::from_be_bytes([b'c', b'H', b'R', b'M']);
pub const PNG_gAMA: u32 = u32::from_be_bytes([b'g', b'A', b'M', b'A']);
pub const PNG_hIST: u32 = u32::from_be_bytes([b'h', b'I', b'S', b'T']);
pub const PNG_iCCP: u32 = u32::from_be_bytes([b'i', b'C', b'C', b'P']);
pub const PNG_iTXt: u32 = u32::from_be_bytes([b'i', b'T', b'X', b't']);
pub const PNG_oFFs: u32 = u32::from_be_bytes([b'o', b'F', b'F', b's']);
pub const PNG_pCAL: u32 = u32::from_be_bytes([b'p', b'C', b'A', b'L']);
pub const PNG_sCAL: u32 = u32::from_be_bytes([b's', b'C', b'A', b'L']);
pub const PNG_pHYs: u32 = u32::from_be_bytes([b'p', b'H', b'Y', b's']);
pub const PNG_sBIT: u32 = u32::from_be_bytes([b's', b'B', b'I', b'T']);
pub const PNG_sPLT: u32 = u32::from_be_bytes([b's', b'P', b'L', b'T']);
pub const PNG_sRGB: u32 = u32::from_be_bytes([b's', b'R', b'G', b'B']);
pub const PNG_tIME: u32 = u32::from_be_bytes([b't', b'I', b'M', b'E']);
pub const PNG_tRNS: u32 = u32::from_be_bytes([b't', b'R', b'N', b'S']);
pub const PNG_zTXt: u32 = u32::from_be_bytes([b'z', b'T', b'X', b't']);

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
pub struct png_ihdr_s {
    pub width: ulong,
    pub height: ulong,
    pub bitdepth: byte,  // Bits per sample (not per pixel)
    pub colortype: byte,  // bit 0 - palette; bit 1 - RGB; bit 2 - alpha channel
    pub compression: byte,  // 0 for zip - error otherwise
    pub filter: byte,  // 0 for adaptive with the 5 basic types - error otherwise
    pub interlace: byte,  // 0 for no interlace - 1 for Adam7 interlace
}

pub type png_ihdr_t = png_ihdr_s;

#[repr(C)]
pub struct png_image_s {
    pub data: *mut byte,
    pub width: ulong,
    pub height: ulong,
    pub bytedepth: ulong,
    pub isimage: c_bool,
}

pub type png_image_t = png_image_s;

extern "C" {
    pub fn LoadPNG32(
        name: *mut c_char,
        pixels: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
        bytedepth: *mut c_int,
    ) -> c_bool;

    pub fn LoadPNG8(
        name: *mut c_char,
        pixels: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
    ) -> c_bool;

    pub fn PNG_Save(
        name: *const c_char,
        data: *mut byte,
        width: c_int,
        height: c_int,
        bytedepth: c_int,
    ) -> c_bool;
}

// end
