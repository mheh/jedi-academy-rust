// Filename:-	small_header.h
//
// This file is just used so I can isolate a few small structs from various horrible MP3 header files without
//	externalising code protos etc. This can now be included by both main game sound code (through sfx_t) and MP3 C code.
//

#[repr(C)]
pub union SAMPLE {
    pub s: core::ffi::c_int,
    pub x: f32,
}

#[repr(C)]
pub struct IN_OUT {
    pub in_bytes: core::ffi::c_int,
    pub out_bytes: core::ffi::c_int,
}

// Damn linux gcc isn't detecting byte as defined
#[cfg(windows)]
pub type byte = u8;

/////////////// eof ////////////
