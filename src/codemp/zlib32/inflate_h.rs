//! Mechanical port of `codemp/zlib32/inflate.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_int;

use super::zip_h::{byte, ulong, WINDOW_SIZE};

// Maximum size of dynamic tree.  The maximum found in a long but non-
// exhaustive search was 1004 huft structures (850 for length/literals
// and 154 for distances, the latter actually the result of an
// exhaustive search).  The actual maximum is not known, but the
// value below is more than safe.

pub const MANY: usize = 1440;

// maximum bit length of any code (if BMAX needs to be larger than 16, then h and x[] should be ulong.)
pub const BMAX: usize = 15;

pub type check_func = Option<unsafe extern "C" fn(check: ulong, buf: *const byte, len: ulong) -> ulong>;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum inflate_block_mode {
    TYPE,   // get type bits (3, including end bit)
    LENS,   // get lengths for stored
    STORED, // processing stored block
    TABLE,  // get table lengths
    BTREE,  // get bit lengths tree for a dynamic block
    DTREE,  // get length, distance trees for a dynamic block
    CODES,  // processing fixed or dynamic block
    DRY,    // output remaining window bytes
    DONE,   // finished last block, done
    BAD,    // got a data error--stuck here
}

// waiting for "i:"=input, "o:"=output, "x:"=nothing
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum inflate_codes_mode {
    START,   // x: set up for LEN
    LEN,     // i: get length/literal/eob next
    LENEXT,  // i: getting length extra (have base)
    DIST,    // i: get distance next
    DISTEXT, // i: getting distance extra
    COPY,    // o: copying bytes in window, waiting for space
    LIT,     // o: got literal, waiting for output space
    WASH,    // o: got eob, possibly still output waiting
    END,     // x: got eob and all data flushed
    BADCODE, // x: got error
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum inflate_mode {
    imMETHOD, // waiting for method byte
    imFLAG,   // waiting for flag byte
    imBLOCKS, // decompressing blocks
    imCHECK4, // four check bytes to go
    imCHECK3, // three check bytes to go
    imCHECK2, // two check bytes to go
    imCHECK1, // one check byte to go
    imDONE,   // finished check, done
    imBAD,    // got an error--stay here
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct inflate_huft_s {
    pub Exop: byte,  // number of extra bits or operation
    pub Bits: byte,  // number of bits in this code or subcode
    pub base: ulong, // literal, length base, distance base, or table offset
}

pub type inflate_huft_t = inflate_huft_s;

// inflate codes private state
#[repr(C)]
#[derive(Clone, Copy)]
pub struct inflate_codes_state_s_code {
    pub tree: *mut inflate_huft_t, // pointer into tree
    pub need: ulong,              // bits needed
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct inflate_codes_state_s_copy {
    pub get: ulong,  // bits to get for extra
    pub dist: ulong, // distance back to copy from
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union inflate_codes_state_s_submode {
    pub code: inflate_codes_state_s_code, // if LEN or DIST, where in tree
    pub lit: ulong,                       // if LIT, literal
    pub copy: inflate_codes_state_s_copy, // if EXT or COPY, where and how much
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct inflate_codes_state_s {
    pub mode: inflate_codes_mode, // current inflate_codes mode

    // mode dependent information
    pub len: ulong,
    pub submode: inflate_codes_state_s_submode,

    // mode independent information
    pub lbits: byte,                 // ltree bits decoded per branch
    pub dbits: byte,                 // dtree bits decoder per branch
    pub ltree: *mut inflate_huft_t,  // literal/length/eob tree
    pub dtree: *mut inflate_huft_t,  // distance tree
}

pub type inflate_codes_state_t = inflate_codes_state_s;

// inflate blocks semi-private state
#[repr(C)]
#[derive(Clone, Copy)]
pub struct inflate_blocks_state_s_trees {
    pub table: ulong,             // table lengths (14 bits)
    pub index: ulong,             // index into blens (or border)
    pub blens: *mut ulong,        // bit lengths of codes
    pub bb: ulong,                // bit length tree depth
    pub tb: *mut inflate_huft_t,  // bit length decoding tree
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct inflate_blocks_state_s_decode {
    pub codes: *mut inflate_codes_state_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union inflate_blocks_state_s_submode {
    pub left: ulong,                            // if STORED, bytes left to copy
    pub trees: inflate_blocks_state_s_trees,    // if DTREE, decoding info for trees
    pub decode: inflate_blocks_state_s_decode,  // if CODES, current state
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct inflate_blocks_state_s {
    // mode
    pub mode: inflate_block_mode, // current inflate_block mode

    // mode dependent information
    pub submode: inflate_blocks_state_s_submode,
    pub last: bool, // true if this block is the last block

    // mode independent information
    pub bitk: ulong,                  // bits in bit buffer
    pub bitb: ulong,                  // bit buffer
    pub hufts: *mut inflate_huft_t,   // single malloc for tree space
    pub window: [byte; WINDOW_SIZE],  // sliding window
    pub end: *mut byte,               // one byte after sliding window
    pub read: *mut byte,              // window read pointer
    pub write: *mut byte,             // window write pointer
    pub check: ulong,                 // check on output
}

pub type inflate_blocks_state_t = inflate_blocks_state_s;

// inflate private state
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct inflate_state_s {
    pub mode: inflate_mode, // current inflate mode

    pub method: ulong, // if FLAGS, method byte

    // mode independent information
    pub nowrap: c_int,                       // flag for no wrapper
    pub wbits: ulong,                        // log2(window size)  (8..15, defaults to 15)
    pub blocks: *mut inflate_blocks_state_t, // current inflate_blocks state

    pub adler: ulong,
    pub calcadler: ulong,
}

pub type inflate_state = inflate_state_s;

// end
