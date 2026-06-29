// inflate.cpp faithful port to Rust
#![allow(non_snake_case, dead_code, non_upper_case_globals)]

use core::ffi::{c_int, c_char};
use core::ptr;
use core::mem;

// If you use the zlib library in a product, an acknowledgment is welcome
// in the documentation of your product. If for some reason you cannot
// include such an acknowledgment, I would appreciate that you keep this
// copyright string in the executable of your product.
pub const INFLATE_COPYRIGHT: &[u8] = b"Inflate 1.1.3 Copyright 1995-1998 Mark Adler ";

static mut inflate_error: *const c_char = b"OK\0".as_ptr() as *const c_char;

// Maximum size of dynamic tree.  The maximum found in a long but non-
// exhaustive search was 1004 huft structures (850 for length/literals
// and 154 for distances, the latter actually the result of an
// exhaustive search).  The actual maximum is not known, but the
// value below is more than safe.
const MANY: usize = 1440;

// maximum bit length of any code (if BMAX needs to be larger than 16, then h and x[] should be ulong.)
const BMAX: usize = 15;

const MAX_WBITS: u32 = 15;
const WINDOW_SIZE: usize = 1 << MAX_WBITS;

// The three kinds of block type
const STORED_BLOCK: u32 = 0;
const STATIC_TREES: u32 = 1;
const DYN_TREES: u32 = 2;
const MODE_ILLEGAL: u32 = 3;

// The minimum and maximum match lengths
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;

// number of distance codes
const D_CODES: usize = 30;

const TAG_INFLATE: c_int = 43;
const ZF_DEFLATED: u8 = 8;

// Enums for inflate modes

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum inflate_block_mode {
    TYPE = 0,      // get type bits (3, including end bit)
    LENS = 1,      // get lengths for stored
    STORED = 2,    // processing stored block
    TABLE = 3,     // get table lengths
    BTREE = 4,     // get bit lengths tree for a dynamic block
    DTREE = 5,     // get length, distance trees for a dynamic block
    CODES = 6,     // processing fixed or dynamic block
    DRY = 7,       // output remaining window bytes
    DONE = 8,      // finished last block, done
    BAD = 9,       // got a data error--stuck here
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum inflate_codes_mode {
    START = 0,     // x: set up for LEN
    LEN = 1,       // i: get length/literal/eob next
    LENEXT = 2,    // i: getting length extra (have base)
    DIST = 3,      // i: get distance next
    DISTEXT = 4,   // i: getting distance extra
    COPY = 5,      // o: copying bytes in window, waiting for space
    LIT = 6,       // o: got literal, waiting for output space
    WASH = 7,      // o: got eob, possibly still output waiting
    END = 8,       // x: got eob and all data flushed
    BADCODE = 9,   // x: got error
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum inflate_mode {
    imMETHOD = 0,  // waiting for method byte
    imFLAG = 1,    // waiting for flag byte
    imBLOCKS = 2,  // decompressing blocks
    imCHECK4 = 3,  // four check bytes to go
    imCHECK3 = 4,  // three check bytes to go
    imCHECK2 = 5,  // two check bytes to go
    imCHECK1 = 6,  // one check byte to go
    imDONE = 7,    // finished check, done
    imBAD = 8,     // got an error--stay here
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EStatus {
    Z_STREAM_ERROR = -3i32 as u32,  // Basic error from failed sanity checks
    Z_BUF_ERROR = -2i32 as u32,     // Not enough input or output
    Z_DATA_ERROR = -1i32 as u32,    // Invalid data in the stream
    Z_OK = 0,
    Z_STREAM_END = 1,               // End of stream
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct inflate_huft_t {
    pub Exop: u8,          // number of extra bits or operation
    pub Bits: u8,          // number of bits in this code or subcode
    pub base: usize,       // literal, length base, distance base, or table offset
}

// inflate codes private state
#[repr(C)]
pub struct inflate_codes_state_t {
    pub mode: inflate_codes_mode,    // current inflate_codes mode
    pub len: usize,
    pub code_tree: *mut inflate_huft_t,
    pub code_need: usize,
    pub lit: usize,
    pub copy_get: usize,
    pub copy_dist: usize,
    pub lbits: u8,         // ltree bits decoded per branch
    pub dbits: u8,         // dtree bits decoder per branch
    pub ltree: *mut inflate_huft_t,  // literal/length/eob tree
    pub dtree: *mut inflate_huft_t,  // distance tree
}

// inflate blocks semi-private state
#[repr(C)]
pub struct inflate_blocks_state_t {
    pub mode: inflate_block_mode,    // current inflate_block mode
    pub left: usize,                 // if STORED, bytes left to copy
    pub table: usize,                // table lengths (14 bits)
    pub index: usize,                // index into blens (or border)
    pub blens: *mut usize,           // bit lengths of codes
    pub bb: usize,                   // bit length tree depth
    pub tb: *mut inflate_huft_t,     // bit length decoding tree
    pub last: u32,                   // true if this block is the last block
    pub bitk: usize,                 // bits in bit buffer
    pub bitb: usize,                 // bit buffer
    pub hufts: *mut inflate_huft_t,  // single malloc for tree space
    pub window: [u8; WINDOW_SIZE],   // sliding window
    pub end: *mut u8,                // one byte after sliding window
    pub read: *mut u8,               // window read pointer
    pub write: *mut u8,              // window write pointer
    pub check: usize,                // check on output
    pub decode_codes: *mut inflate_codes_state_t,
}

// inflate private state
#[repr(C)]
pub struct inflate_state {
    pub mode: inflate_mode,
    pub method: usize,
    pub nowrap: c_int,
    pub wbits: usize,
    pub blocks: *mut inflate_blocks_state_t,
    pub adler: usize,
    pub calcadler: usize,
}

#[repr(C)]
pub struct z_stream_s {
    pub next_in: *mut u8,
    pub avail_in: usize,
    pub total_in: usize,
    pub next_out: *mut u8,
    pub avail_out: usize,
    pub total_out: usize,
    pub status: EStatus,
    pub error: EStatus,
    pub istate: *mut inflate_state,
    pub dstate: *mut core::ffi::c_void,
    pub quality: usize,
}

// And'ing with mask[n] masks the lower n bits
static INFLATE_MASK: [usize; 17] = [
    0x0000,
    0x0001, 0x0003, 0x0007, 0x000f, 0x001f, 0x003f, 0x007f, 0x00ff,
    0x01ff, 0x03ff, 0x07ff, 0x0fff, 0x1fff, 0x3fff, 0x7fff, 0xffff
];

// Order of the bit length code lengths
static BORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
];

// Copy lengths for literal codes 257..285 (see note #13 above about 258)
static CPLENS: [usize; 31] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31,
    35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258, 0, 0
];

// Extra bits for literal codes 257..285 (112 == invalid)
static CPLEXT: [usize; 31] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
    3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 112, 112
];

// Copy offsets for distance codes 0..29
static CPDIST: [usize; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193,
    257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145,
    8193, 12289, 16385, 24577
];

// Extra bits for distance codes, from zip.h (extern const ulong extra_dbits[D_CODES])
static EXTRA_DBITS: [usize; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6,
    7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13
];

static mut FIXED_BL: usize = 9;
static mut FIXED_BD: usize = 5;

static FIXED_TL: [inflate_huft_t; 512] = [
    inflate_huft_t { Exop: 96, Bits: 7, base: 256 }, inflate_huft_t { Exop: 0, Bits: 8, base: 80  }, inflate_huft_t { Exop: 0, Bits: 8, base: 16 }, inflate_huft_t { Exop: 84,  Bits: 8, base: 115 },
    inflate_huft_t { Exop: 82, Bits: 7, base: 31  }, inflate_huft_t { Exop: 0, Bits: 8, base: 112 }, inflate_huft_t { Exop: 0, Bits: 8, base: 48 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 192 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 10  }, inflate_huft_t { Exop: 0, Bits: 8, base: 96  }, inflate_huft_t { Exop: 0, Bits: 8, base: 32 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 160 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 0   }, inflate_huft_t { Exop: 0, Bits: 8, base: 128 }, inflate_huft_t { Exop: 0, Bits: 8, base: 64 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 224 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 6   }, inflate_huft_t { Exop: 0, Bits: 8, base: 88  }, inflate_huft_t { Exop: 0, Bits: 8, base: 24 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 144 },
    inflate_huft_t { Exop: 83, Bits: 7, base: 59  }, inflate_huft_t { Exop: 0, Bits: 8, base: 120 }, inflate_huft_t { Exop: 0, Bits: 8, base: 56 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 208 },
    inflate_huft_t { Exop: 81, Bits: 7, base: 17  }, inflate_huft_t { Exop: 0, Bits: 8, base: 104 }, inflate_huft_t { Exop: 0, Bits: 8, base: 40 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 176 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 8   }, inflate_huft_t { Exop: 0, Bits: 8, base: 136 }, inflate_huft_t { Exop: 0, Bits: 8, base: 72 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 240 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 4   }, inflate_huft_t { Exop: 0, Bits: 8, base: 84  }, inflate_huft_t { Exop: 0, Bits: 8, base: 20 }, inflate_huft_t { Exop: 85,  Bits: 8, base: 227 },
    inflate_huft_t { Exop: 83, Bits: 7, base: 43  }, inflate_huft_t { Exop: 0, Bits: 8, base: 116 }, inflate_huft_t { Exop: 0, Bits: 8, base: 52 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 200 },
    inflate_huft_t { Exop: 81, Bits: 7, base: 13  }, inflate_huft_t { Exop: 0, Bits: 8, base: 100 }, inflate_huft_t { Exop: 0, Bits: 8, base: 36 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 168 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 4   }, inflate_huft_t { Exop: 0, Bits: 8, base: 132 }, inflate_huft_t { Exop: 0, Bits: 8, base: 68 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 232 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 8   }, inflate_huft_t { Exop: 0, Bits: 8, base: 92  }, inflate_huft_t { Exop: 0, Bits: 8, base: 28 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 152 },
    inflate_huft_t { Exop: 84, Bits: 7, base: 83  }, inflate_huft_t { Exop: 0, Bits: 8, base: 124 }, inflate_huft_t { Exop: 0, Bits: 8, base: 60 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 216 },
    inflate_huft_t { Exop: 82, Bits: 7, base: 23  }, inflate_huft_t { Exop: 0, Bits: 8, base: 108 }, inflate_huft_t { Exop: 0, Bits: 8, base: 44 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 184 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 12  }, inflate_huft_t { Exop: 0, Bits: 8, base: 140 }, inflate_huft_t { Exop: 0, Bits: 8, base: 76 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 248 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 3   }, inflate_huft_t { Exop: 0, Bits: 8, base: 82  }, inflate_huft_t { Exop: 0, Bits: 8, base: 18 }, inflate_huft_t { Exop: 85,  Bits: 8, base: 163 },
    inflate_huft_t { Exop: 83, Bits: 7, base: 35  }, inflate_huft_t { Exop: 0, Bits: 8, base: 114 }, inflate_huft_t { Exop: 0, Bits: 8, base: 50 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 196 },
    inflate_huft_t { Exop: 81, Bits: 7, base: 11  }, inflate_huft_t { Exop: 0, Bits: 8, base: 98  }, inflate_huft_t { Exop: 0, Bits: 8, base: 34 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 164 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 2   }, inflate_huft_t { Exop: 0, Bits: 8, base: 130 }, inflate_huft_t { Exop: 0, Bits: 8, base: 66 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 228 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 7   }, inflate_huft_t { Exop: 0, Bits: 8, base: 90  }, inflate_huft_t { Exop: 0, Bits: 8, base: 26 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 148 },
    inflate_huft_t { Exop: 84, Bits: 7, base: 67  }, inflate_huft_t { Exop: 0, Bits: 8, base: 122 }, inflate_huft_t { Exop: 0, Bits: 8, base: 58 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 212 },
    inflate_huft_t { Exop: 82, Bits: 7, base: 19  }, inflate_huft_t { Exop: 0, Bits: 8, base: 106 }, inflate_huft_t { Exop: 0, Bits: 8, base: 42 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 180 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 10  }, inflate_huft_t { Exop: 0, Bits: 8, base: 138 }, inflate_huft_t { Exop: 0, Bits: 8, base: 74 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 244 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 5   }, inflate_huft_t { Exop: 0, Bits: 8, base: 86  }, inflate_huft_t { Exop: 0, Bits: 8, base: 22 }, inflate_huft_t { Exop: 192, Bits: 8, base: 0   },
    inflate_huft_t { Exop: 83, Bits: 7, base: 51  }, inflate_huft_t { Exop: 0, Bits: 8, base: 118 }, inflate_huft_t { Exop: 0, Bits: 8, base: 54 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 204 },
    inflate_huft_t { Exop: 81, Bits: 7, base: 15  }, inflate_huft_t { Exop: 0, Bits: 8, base: 102 }, inflate_huft_t { Exop: 0, Bits: 8, base: 38 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 172 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 6   }, inflate_huft_t { Exop: 0, Bits: 8, base: 134 }, inflate_huft_t { Exop: 0, Bits: 8, base: 70 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 236 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 9   }, inflate_huft_t { Exop: 0, Bits: 8, base: 94  }, inflate_huft_t { Exop: 0, Bits: 8, base: 30 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 156 },
    inflate_huft_t { Exop: 84, Bits: 7, base: 99  }, inflate_huft_t { Exop: 0, Bits: 8, base: 126 }, inflate_huft_t { Exop: 0, Bits: 8, base: 62 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 220 },
    inflate_huft_t { Exop: 82, Bits: 7, base: 27  }, inflate_huft_t { Exop: 0, Bits: 8, base: 110 }, inflate_huft_t { Exop: 0, Bits: 8, base: 46 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 188 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 14  }, inflate_huft_t { Exop: 0, Bits: 8, base: 142 }, inflate_huft_t { Exop: 0, Bits: 8, base: 78 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 252 },
    inflate_huft_t { Exop: 96, Bits: 7, base: 256 }, inflate_huft_t { Exop: 0, Bits: 8, base: 81  }, inflate_huft_t { Exop: 0, Bits: 8, base: 17 }, inflate_huft_t { Exop: 85,  Bits: 8, base: 131 },
    inflate_huft_t { Exop: 82, Bits: 7, base: 31  }, inflate_huft_t { Exop: 0, Bits: 8, base: 113 }, inflate_huft_t { Exop: 0, Bits: 8, base: 49 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 194 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 10  }, inflate_huft_t { Exop: 0, Bits: 8, base: 97  }, inflate_huft_t { Exop: 0, Bits: 8, base: 33 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 162 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 1   }, inflate_huft_t { Exop: 0, Bits: 8, base: 129 }, inflate_huft_t { Exop: 0, Bits: 8, base: 65 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 226 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 6   }, inflate_huft_t { Exop: 0, Bits: 8, base: 89  }, inflate_huft_t { Exop: 0, Bits: 8, base: 25 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 146 },
    inflate_huft_t { Exop: 83, Bits: 7, base: 59  }, inflate_huft_t { Exop: 0, Bits: 8, base: 121 }, inflate_huft_t { Exop: 0, Bits: 8, base: 57 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 210 },
    inflate_huft_t { Exop: 81, Bits: 7, base: 17  }, inflate_huft_t { Exop: 0, Bits: 8, base: 105 }, inflate_huft_t { Exop: 0, Bits: 8, base: 41 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 178 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 9   }, inflate_huft_t { Exop: 0, Bits: 8, base: 137 }, inflate_huft_t { Exop: 0, Bits: 8, base: 73 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 242 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 4   }, inflate_huft_t { Exop: 0, Bits: 8, base: 85  }, inflate_huft_t { Exop: 0, Bits: 8, base: 21 }, inflate_huft_t { Exop: 80,  Bits: 8, base: 258 },
    inflate_huft_t { Exop: 83, Bits: 7, base: 43  }, inflate_huft_t { Exop: 0, Bits: 8, base: 117 }, inflate_huft_t { Exop: 0, Bits: 8, base: 53 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 202 },
    inflate_huft_t { Exop: 81, Bits: 7, base: 13  }, inflate_huft_t { Exop: 0, Bits: 8, base: 101 }, inflate_huft_t { Exop: 0, Bits: 8, base: 37 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 170 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 5   }, inflate_huft_t { Exop: 0, Bits: 8, base: 133 }, inflate_huft_t { Exop: 0, Bits: 8, base: 69 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 234 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 8   }, inflate_huft_t { Exop: 0, Bits: 8, base: 93  }, inflate_huft_t { Exop: 0, Bits: 8, base: 29 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 154 },
    inflate_huft_t { Exop: 84, Bits: 7, base: 83  }, inflate_huft_t { Exop: 0, Bits: 8, base: 125 }, inflate_huft_t { Exop: 0, Bits: 8, base: 61 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 218 },
    inflate_huft_t { Exop: 82, Bits: 7, base: 23  }, inflate_huft_t { Exop: 0, Bits: 8, base: 109 }, inflate_huft_t { Exop: 0, Bits: 8, base: 45 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 186 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 13  }, inflate_huft_t { Exop: 0, Bits: 8, base: 141 }, inflate_huft_t { Exop: 0, Bits: 8, base: 77 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 250 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 3   }, inflate_huft_t { Exop: 0, Bits: 8, base: 83  }, inflate_huft_t { Exop: 0, Bits: 8, base: 19 }, inflate_huft_t { Exop: 85,  Bits: 8, base: 195 },
    inflate_huft_t { Exop: 83, Bits: 7, base: 35  }, inflate_huft_t { Exop: 0, Bits: 8, base: 115 }, inflate_huft_t { Exop: 0, Bits: 8, base: 51 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 198 },
    inflate_huft_t { Exop: 81, Bits: 7, base: 11  }, inflate_huft_t { Exop: 0, Bits: 8, base: 99  }, inflate_huft_t { Exop: 0, Bits: 8, base: 35 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 166 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 3   }, inflate_huft_t { Exop: 0, Bits: 8, base: 131 }, inflate_huft_t { Exop: 0, Bits: 8, base: 67 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 230 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 7   }, inflate_huft_t { Exop: 0, Bits: 8, base: 91  }, inflate_huft_t { Exop: 0, Bits: 8, base: 27 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 150 },
    inflate_huft_t { Exop: 84, Bits: 7, base: 67  }, inflate_huft_t { Exop: 0, Bits: 8, base: 123 }, inflate_huft_t { Exop: 0, Bits: 8, base: 59 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 214 },
    inflate_huft_t { Exop: 82, Bits: 7, base: 19  }, inflate_huft_t { Exop: 0, Bits: 8, base: 107 }, inflate_huft_t { Exop: 0, Bits: 8, base: 43 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 182 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 11  }, inflate_huft_t { Exop: 0, Bits: 8, base: 139 }, inflate_huft_t { Exop: 0, Bits: 8, base: 75 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 246 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 5   }, inflate_huft_t { Exop: 0, Bits: 8, base: 87  }, inflate_huft_t { Exop: 0, Bits: 8, base: 23 }, inflate_huft_t { Exop: 192, Bits: 8, base: 0   },
    inflate_huft_t { Exop: 83, Bits: 7, base: 51  }, inflate_huft_t { Exop: 0, Bits: 8, base: 119 }, inflate_huft_t { Exop: 0, Bits: 8, base: 55 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 206 },
    inflate_huft_t { Exop: 81, Bits: 7, base: 15  }, inflate_huft_t { Exop: 0, Bits: 8, base: 103 }, inflate_huft_t { Exop: 0, Bits: 8, base: 39 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 174 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 7   }, inflate_huft_t { Exop: 0, Bits: 8, base: 135 }, inflate_huft_t { Exop: 0, Bits: 8, base: 71 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 238 },
    inflate_huft_t { Exop: 80, Bits: 7, base: 9   }, inflate_huft_t { Exop: 0, Bits: 8, base: 95  }, inflate_huft_t { Exop: 0, Bits: 8, base: 31 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 158 },
    inflate_huft_t { Exop: 84, Bits: 7, base: 99  }, inflate_huft_t { Exop: 0, Bits: 8, base: 127 }, inflate_huft_t { Exop: 0, Bits: 8, base: 63 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 222 },
    inflate_huft_t { Exop: 82, Bits: 7, base: 27  }, inflate_huft_t { Exop: 0, Bits: 8, base: 111 }, inflate_huft_t { Exop: 0, Bits: 8, base: 47 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 190 },
    inflate_huft_t { Exop: 0,  Bits: 8, base: 15  }, inflate_huft_t { Exop: 0, Bits: 8, base: 143 }, inflate_huft_t { Exop: 0, Bits: 8, base: 79 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 254 },
];

static FIXED_TD: [inflate_huft_t; 32] = [
    inflate_huft_t { Exop: 80, Bits: 5, base: 1  }, inflate_huft_t { Exop: 87, Bits: 5, base: 257  }, inflate_huft_t { Exop: 83, Bits: 5, base: 17  }, inflate_huft_t { Exop: 91,  Bits: 5, base: 4097  },
    inflate_huft_t { Exop: 81, Bits: 5, base: 5  }, inflate_huft_t { Exop: 89, Bits: 5, base: 1025 }, inflate_huft_t { Exop: 85, Bits: 5, base: 65  }, inflate_huft_t { Exop: 93,  Bits: 5, base: 16385 },
    inflate_huft_t { Exop: 80, Bits: 5, base: 3  }, inflate_huft_t { Exop: 88, Bits: 5, base: 513  }, inflate_huft_t { Exop: 84, Bits: 5, base: 33  }, inflate_huft_t { Exop: 92,  Bits: 5, base: 8193  },
    inflate_huft_t { Exop: 82, Bits: 5, base: 9  }, inflate_huft_t { Exop: 90, Bits: 5, base: 2049 }, inflate_huft_t { Exop: 86, Bits: 5, base: 129 }, inflate_huft_t { Exop: 192, Bits: 5, base: 24577 },
    inflate_huft_t { Exop: 80, Bits: 5, base: 2  }, inflate_huft_t { Exop: 87, Bits: 5, base: 385  }, inflate_huft_t { Exop: 83, Bits: 5, base: 25  }, inflate_huft_t { Exop: 91,  Bits: 5, base: 6145  },
    inflate_huft_t { Exop: 81, Bits: 5, base: 7  }, inflate_huft_t { Exop: 89, Bits: 5, base: 1537 }, inflate_huft_t { Exop: 85, Bits: 5, base: 97  }, inflate_huft_t { Exop: 93,  Bits: 5, base: 24577 },
    inflate_huft_t { Exop: 80, Bits: 5, base: 4  }, inflate_huft_t { Exop: 88, Bits: 5, base: 769  }, inflate_huft_t { Exop: 84, Bits: 5, base: 49  }, inflate_huft_t { Exop: 92,  Bits: 5, base: 12289 },
    inflate_huft_t { Exop: 82, Bits: 5, base: 13 }, inflate_huft_t { Exop: 90, Bits: 5, base: 3073 }, inflate_huft_t { Exop: 86, Bits: 5, base: 193 }, inflate_huft_t { Exop: 192, Bits: 5, base: 24577 },
];

// External functions (from C)
extern "C" {
    fn Z_Malloc(size: c_int, tag: c_int, clear: c_int) -> *mut core::ffi::c_void;
    fn Z_Free(ptr: *mut core::ffi::c_void) -> c_int;
    fn adler32(adler: usize, buf: *const u8, len: usize) -> usize;
    fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
}

// ===============================================================================
// ===============================================================================

unsafe fn inflate_blocks_reset(z: *mut z_stream_s, s: *mut inflate_blocks_state_t) {
    let s_ref = &mut *s;
    let z_ref = &mut *z;

    if (s_ref.mode as u32 == inflate_block_mode::BTREE as u32) || (s_ref.mode as u32 == inflate_block_mode::DTREE as u32) {
        Z_Free(s_ref.blens as *mut core::ffi::c_void);
    }
    if s_ref.mode as u32 == inflate_block_mode::CODES as u32 {
        Z_Free(s_ref.decode_codes as *mut core::ffi::c_void);
    }
    s_ref.mode = inflate_block_mode::TYPE;
    s_ref.bitk = 0;
    s_ref.bitb = 0;
    s_ref.write = s_ref.window.as_mut_ptr();
    s_ref.read = s_ref.window.as_mut_ptr();
    (*z_ref.istate).adler = 1;
}

// ===============================================================================
// ===============================================================================

unsafe fn inflate_blocks_free(z: *mut z_stream_s, s: *mut inflate_blocks_state_t) -> EStatus {
    inflate_blocks_reset(z, s);
    Z_Free((*s).hufts as *mut core::ffi::c_void);
    (*s).hufts = ptr::null_mut();
    Z_Free(s as *mut core::ffi::c_void);
    EStatus::Z_OK
}

// ===============================================================================
// ===============================================================================

unsafe fn inflate_blocks_new(z: *mut z_stream_s, _check: *const core::ffi::c_void) -> *mut inflate_blocks_state_t {
    let s = Z_Malloc(mem::size_of::<inflate_blocks_state_t>() as c_int, TAG_INFLATE, 1) as *mut inflate_blocks_state_t;
    (*s).hufts = Z_Malloc((mem::size_of::<inflate_huft_t>() * MANY) as c_int, TAG_INFLATE, 1) as *mut inflate_huft_t;
    (*s).end = (*s).window.as_mut_ptr().add(WINDOW_SIZE);
    (*s).mode = inflate_block_mode::TYPE;
    inflate_blocks_reset(z, s);

    s
}

// ===============================================================================
// copy as much as possible from the sliding window to the output area
// ===============================================================================

unsafe fn inflate_flush_copy(z: *mut z_stream_s, s: *mut inflate_blocks_state_t, mut count: usize) {
    let z_ref = &mut *z;
    let s_ref = &mut *s;

    if count > z_ref.avail_out {
        count = z_ref.avail_out;
    }
    if count > 0 && (z_ref.error as u32 == EStatus::Z_BUF_ERROR as u32) {
        z_ref.error = EStatus::Z_OK;
    }

    // Calculate the checksum if required
    if (*(*z).istate).nowrap == 0 {
        (*(*z).istate).adler = adler32((*(*z).istate).adler, s_ref.read, count);
    }

    // copy as as end of window
    memcpy(z_ref.next_out, s_ref.read, count);

    // update counters
    z_ref.avail_out -= count;
    z_ref.total_out += count;
    z_ref.next_out = z_ref.next_out.add(count);
    s_ref.read = s_ref.read.add(count);
}

// ===============================================================================
// ===============================================================================

unsafe fn inflate_flush(z: *mut z_stream_s, s: *mut inflate_blocks_state_t) {
    let z_ref = &mut *z;
    let s_ref = &mut *s;

    // compute number of bytes to copy as as end of window
    let count = if s_ref.read <= s_ref.write {
        s_ref.write as usize - s_ref.read as usize
    } else {
        s_ref.end as usize - s_ref.read as usize
    };

    inflate_flush_copy(z, s, count);

    // see if more to copy at beginning of window
    if s_ref.read == s_ref.end {
        // wrap pointers
        s_ref.read = s_ref.window.as_mut_ptr();
        if s_ref.write == s_ref.end {
            s_ref.write = s_ref.window.as_mut_ptr();
        }
        // compute bytes to copy
        let count = s_ref.write as usize - s_ref.read as usize;
        inflate_flush_copy(z, s, count);
    }
}

// ===============================================================================
// get bytes and bits
// ===============================================================================

unsafe fn getbits(z: *mut z_stream_s, s: *mut inflate_blocks_state_t, bits: usize) -> bool {
    let z_ref = &mut *z;
    let s_ref = &mut *s;

    while s_ref.bitk < bits {
        if z_ref.avail_in > 0 {
            z_ref.error = EStatus::Z_OK;
        } else {
            inflate_flush(z, s);
            return false;
        }
        z_ref.avail_in -= 1;
        z_ref.total_in += 1;
        let byte_val = *z_ref.next_in as usize;
        s_ref.bitb |= byte_val << s_ref.bitk;
        z_ref.next_in = z_ref.next_in.add(1);
        s_ref.bitk += 8;
    }
    true
}

// ===============================================================================
// output bytes
// ===============================================================================

unsafe fn needout(z: *mut z_stream_s, s: *mut inflate_blocks_state_t, mut bytesToEnd: usize) -> usize {
    let s_ref = &mut *s;
    let z_ref = &mut *z;

    if bytesToEnd == 0 {
        if (s_ref.write == s_ref.end) && (s_ref.read != s_ref.window.as_mut_ptr()) {
            s_ref.write = s_ref.window.as_mut_ptr();
            bytesToEnd = if s_ref.write < s_ref.read {
                s_ref.read as usize - s_ref.write as usize - 1
            } else {
                s_ref.end as usize - s_ref.write as usize
            };
        }
        if bytesToEnd == 0 {
            inflate_flush(z, s);
            bytesToEnd = if s_ref.write < s_ref.read {
                s_ref.read as usize - s_ref.write as usize - 1
            } else {
                s_ref.end as usize - s_ref.write as usize
            };
            if (s_ref.write == s_ref.end) && (s_ref.read != s_ref.window.as_mut_ptr()) {
                s_ref.write = s_ref.window.as_mut_ptr();
                bytesToEnd = if s_ref.write < s_ref.read {
                    s_ref.read as usize - s_ref.write as usize - 1
                } else {
                    s_ref.end as usize - s_ref.write as usize
                };
            }
            if bytesToEnd == 0 {
                inflate_flush(z, s);
                return bytesToEnd;
            }
        }
    }
    z_ref.error = EStatus::Z_OK;
    bytesToEnd
}

// ===============================================================================
// Called with number of bytes left to write in window at least 258
//	 (the maximum string length) and number of input bytes available
//	 at least ten.	The ten bytes are six bytes for the longest length/
//	 distance pair plus four bytes for overloading the bit buffer.
// ===============================================================================

unsafe fn qcopy(mut dst: *mut u8, mut src: *const u8, mut count: usize) -> *mut u8 {
    while count > 0 {
        *dst = *src;
        dst = dst.add(1);
        src = src.add(1);
        count -= 1;
    }
    dst
}

unsafe fn get_remaining(s: *mut inflate_blocks_state_t) -> usize {
    let s_ref = &*s;
    if s_ref.write < s_ref.read {
        s_ref.read as usize - s_ref.write as usize - 1
    } else {
        s_ref.end as usize - s_ref.write as usize
    }
}

unsafe fn inflate_fast(lengthMask: usize, distMask: usize, lengthTree: *mut inflate_huft_t, distTree: *mut inflate_huft_t, s: *mut inflate_blocks_state_t, z: *mut z_stream_s) -> EStatus {
    let mut data = (*z).next_in;
    let mut dst = (*s).write;
    let mut availin = (*z).avail_in;
    let mut bitb = (*s).bitb;
    let mut bitk = (*s).bitk;

    let mut bytesToEnd = get_remaining(s);

    // do until not enough input or output space for fast loop
    // assume called with bytesToEnd >= 258 && availIn >= 10
    while (bytesToEnd >= 258) && (availin >= 10) {
        // get literal/length code
        while bitk < 20 {
            bitb |= (*data as usize) << bitk;
            data = data.add(1);
            bitk += 8;
            availin -= 1;
        }

        let mut huft = lengthTree.add(bitb & lengthMask);
        if (*huft).Exop == 0 {
            bitb >>= (*huft).Bits;
            bitk -= (*huft).Bits as usize;
            *dst = (*huft).base as u8;
            dst = dst.add(1);
            bytesToEnd -= 1;
        } else {
            let mut extraBits = (*huft).Exop as usize;
            let mut morebits = true;
            loop {
                bitb >>= (*huft).Bits;
                bitk -= (*huft).Bits as usize;
                if (extraBits & 16) != 0 {
                    // get extra bits for length
                    let extra_mask = extraBits & 15;
                    let count = (*huft).base + (bitb & INFLATE_MASK[extra_mask]);
                    bitb >>= extra_mask;
                    bitk -= extra_mask;
                    // decode distance base of block to copy
                    while bitk < 15 {
                        bitb |= (*data as usize) << bitk;
                        data = data.add(1);
                        bitk += 8;
                        availin -= 1;
                    }
                    huft = distTree.add(bitb & distMask);
                    extraBits = (*huft).Exop as usize;
                    let mut copymore = true;
                    loop {
                        bitb >>= (*huft).Bits;
                        bitk -= (*huft).Bits as usize;
                        if (extraBits & 16) != 0 {
                            // get extra bits to add to distance base
                            let extra_dist = extraBits & 15;
                            while bitk < extra_dist {
                                bitb |= (*data as usize) << bitk;
                                data = data.add(1);
                                bitk += 8;
                                availin -= 1;
                            }
                            let dist = (*huft).base + (bitb & INFLATE_MASK[extra_dist]);
                            bitb >>= extra_dist;
                            bitk -= extra_dist;

                            // do the copy
                            bytesToEnd -= count;
                            // offset before dest
                            let src = if (dst as usize - (*s).window.as_ptr() as usize) >= dist {
                                // just copy
                                (dst as usize - dist) as *const u8
                            } else {
                                // else offset after destination
                                // bytes from offset to end
                                let extra_bits_copy = dist - (dst as usize - (*s).window.as_ptr() as usize);
                                // pointer to offset
                                let src_base = ((*s).end as usize - extra_bits_copy) as *const u8;
                                // if source crosses,
                                if count > extra_bits_copy {
                                    // copy to end of window
                                    dst = qcopy(dst, src_base, extra_bits_copy) as *mut u8;
                                    // copy rest from start of window
                                    let count_rest = count - extra_bits_copy;
                                    // copy all or what's left
                                    dst = qcopy(dst, (*s).window.as_ptr(), count_rest) as *mut u8;
                                    copymore = false;
                                    (*s).write = dst;
                                    break;
                                }
                                src_base
                            };
                            // copy all or what's left
                            dst = qcopy(dst, src, count) as *mut u8;
                            copymore = false;
                        } else {
                            if (extraBits & 64) == 0 {
                                huft = huft.add((*huft).base).add(bitb & INFLATE_MASK[extraBits]);
                                extraBits = (*huft).Exop as usize;
                            } else {
                                inflate_error = b"Inflate data: Invalid distance code\0".as_ptr() as *const c_char;
                                (*z).avail_in = availin;
                                (*z).total_in += data as usize - (*z).next_in as usize;
                                (*z).next_in = data;
                                (*s).write = dst;
                                let count_delta = data as usize - (*z).next_in as usize;
                                let bitb_count = (bitk >> 3) < count_delta ? bitk >> 3 : count_delta;
                                (*s).bitb = bitb;
                                (*s).bitk = bitk - (bitb_count << 3);
                                (*z).avail_in += bitb_count;
                                (*z).total_in -= bitb_count;
                                (*z).next_in = ((*z).next_in as usize - bitb_count) as *mut u8;
                                return EStatus::Z_DATA_ERROR;
                            }
                        }
                        if !copymore {
                            break;
                        }
                    }

                    morebits = false;
                } else {
                    if (extraBits & 64) == 0 {
                        huft = huft.add((*huft).base).add(bitb & INFLATE_MASK[extraBits]);
                        extraBits = (*huft).Exop as usize;
                        if extraBits == 0 {
                            bitb >>= (*huft).Bits;
                            bitk -= (*huft).Bits as usize;
                            *dst = (*huft).base as u8;
                            dst = dst.add(1);
                            bytesToEnd -= 1;
                            morebits = false;
                        }
                    } else if (extraBits & 32) != 0 {
                        let count = data as usize - (*z).next_in as usize;

                        (*z).avail_in = availin;
                        (*z).total_in += count;
                        (*z).next_in = data;

                        (*s).write = dst;

                        let count_check = (bitk >> 3) < count ? bitk >> 3 : count;

                        (*s).bitb = bitb;
                        (*s).bitk = bitk - (count_check << 3);
                        (*z).avail_in += count_check;
                        (*z).total_in -= count_check;
                        (*z).next_in = ((*z).next_in as usize - count_check) as *mut u8;
                        return EStatus::Z_STREAM_END;
                    } else {
                        inflate_error = b"Inflate data: Invalid literal/length code\0".as_ptr() as *const c_char;
                        (*z).avail_in = availin;
                        (*z).total_in += data as usize - (*z).next_in as usize;
                        (*z).next_in = data;
                        (*s).write = dst;
                        let count_delta = data as usize - (*z).next_in as usize;
                        let bitb_count = (bitk >> 3) < count_delta ? bitk >> 3 : count_delta;
                        (*s).bitb = bitb;
                        (*s).bitk = bitk - (bitb_count << 3);
                        (*z).avail_in += bitb_count;
                        (*z).total_in -= bitb_count;
                        (*z).next_in = ((*z).next_in as usize - bitb_count) as *mut u8;
                        return EStatus::Z_DATA_ERROR;
                    }
                }
                if !morebits {
                    break;
                }
            }
        }
    }

    // not enough input or output--restore pointers and return
    let count = data as usize - (*z).next_in as usize;

    (*z).avail_in = availin;
    (*z).total_in += count;
    (*z).next_in = data;

    (*s).write = dst;

    let count_bitb = (bitk >> 3) < count ? bitk >> 3 : count;
    (*s).bitb = bitb;
    (*s).bitk = bitk - (count_bitb << 3);
    (*z).avail_in += count_bitb;
    (*z).total_in -= count_bitb;
    (*z).next_in = ((*z).next_in as usize - count_bitb) as *mut u8;

    EStatus::Z_OK
}

// ===============================================================================
// ===============================================================================

unsafe fn inflate_codes(z: *mut z_stream_s, s: *mut inflate_blocks_state_t) {
    let mut infCodes = (*s).decode_codes;
    let mut bytesToEnd = get_remaining(s);

    // process input and output based on current state
    loop {
        // waiting for "i:"=input, "o:"=output, "x:"=nothing
        match (*infCodes).mode {
            inflate_codes_mode::START => {
                if (bytesToEnd >= 258) && ((*z).avail_in >= 10) {
                    (*z).error = inflate_fast(INFLATE_MASK[(*infCodes).code_need], INFLATE_MASK[(*infCodes).dbits as usize], (*infCodes).ltree, (*infCodes).dtree, s, z);
                    bytesToEnd = get_remaining(s);
                    if (*z).error as u32 != EStatus::Z_OK as u32 {
                        (*infCodes).mode = if (*z).error as u32 == EStatus::Z_STREAM_END as u32 {
                            inflate_codes_mode::WASH
                        } else {
                            inflate_codes_mode::BADCODE
                        };
                        break;
                    }
                }
                (*infCodes).code_need = (*infCodes).lbits as usize;
                (*infCodes).code_tree = (*infCodes).ltree;
                (*infCodes).mode = inflate_codes_mode::LEN;
            }
            inflate_codes_mode::LEN => {
                if !getbits(z, s, (*infCodes).code_need) {
                    if (*z).status as u32 == EStatus::Z_BUF_ERROR as u32 {
                        (*z).error = EStatus::Z_STREAM_END;
                    }
                    return;
                }
                let huft = (*infCodes).code_tree.add((*s).bitb & INFLATE_MASK[(*infCodes).code_need]);
                (*s).bitb >>= (*huft).Bits;
                (*s).bitk -= (*huft).Bits as usize;
                let extraBits = (*huft).Exop as usize;
                if extraBits == 0 {
                    (*infCodes).lit = (*huft).base;
                    (*infCodes).mode = inflate_codes_mode::LIT;
                    break;
                }
                if (extraBits & 16) != 0 {
                    (*infCodes).copy_get = extraBits & 15;
                    (*infCodes).len = (*huft).base;
                    (*infCodes).mode = inflate_codes_mode::LENEXT;
                    break;
                }
                if (extraBits & 64) == 0 {
                    (*infCodes).code_need = extraBits;
                    (*infCodes).code_tree = huft.add((*huft).base);
                    break;
                }
                if (extraBits & 32) != 0 {
                    (*infCodes).mode = inflate_codes_mode::WASH;
                    break;
                }
                (*infCodes).mode = inflate_codes_mode::BADCODE;
                inflate_error = b"Inflate data: Invalid literal/length code\0".as_ptr() as *const c_char;
                (*z).error = EStatus::Z_DATA_ERROR;
                inflate_flush(z, s);
                return;
            }
            inflate_codes_mode::LENEXT => {
                if !getbits(z, s, (*infCodes).copy_get) {
                    return;
                }
                (*infCodes).len += (*s).bitb & INFLATE_MASK[(*infCodes).copy_get];
                (*s).bitb >>= (*infCodes).copy_get;
                (*s).bitk -= (*infCodes).copy_get;
                (*infCodes).code_need = (*infCodes).dbits as usize;
                (*infCodes).code_tree = (*infCodes).dtree;
                (*infCodes).mode = inflate_codes_mode::DIST;
            }
            inflate_codes_mode::DIST => {
                if !getbits(z, s, (*infCodes).code_need) {
                    return;
                }
                let huft = (*infCodes).code_tree.add((*s).bitb & INFLATE_MASK[(*infCodes).code_need]);
                (*s).bitb >>= (*huft).Bits;
                (*s).bitk -= (*huft).Bits as usize;
                let extraBits = (*huft).Exop as usize;
                if (extraBits & 16) != 0 {
                    (*infCodes).copy_get = extraBits & 15;
                    (*infCodes).copy_dist = (*huft).base;
                    (*infCodes).mode = inflate_codes_mode::DISTEXT;
                    break;
                }
                if (extraBits & 64) == 0 {
                    (*infCodes).code_need = extraBits;
                    (*infCodes).code_tree = huft.add((*huft).base);
                    break;
                }
                (*infCodes).mode = inflate_codes_mode::BADCODE;
                inflate_error = b"Inflate data: Invalid distance code\0".as_ptr() as *const c_char;
                (*z).error = EStatus::Z_DATA_ERROR;
                inflate_flush(z, s);
                return;
            }
            inflate_codes_mode::DISTEXT => {
                if !getbits(z, s, (*infCodes).copy_get) {
                    return;
                }
                (*infCodes).copy_dist += (*s).bitb & INFLATE_MASK[(*infCodes).copy_get];
                (*s).bitb >>= (*infCodes).copy_get;
                (*s).bitk -= (*infCodes).copy_get;
                (*infCodes).mode = inflate_codes_mode::COPY;
            }
            inflate_codes_mode::COPY => {
                let src = if ((*s).write as usize - (*s).window.as_ptr() as usize) < (*infCodes).copy_dist {
                    ((*s).end as usize - ((*infCodes).copy_dist - ((*s).write as usize - (*s).window.as_ptr() as usize))) as *const u8
                } else {
                    ((*s).write as usize - (*infCodes).copy_dist) as *const u8
                };
                let mut src_mut = src as *mut u8;
                while (*infCodes).len > 0 {
                    bytesToEnd = needout(z, s, bytesToEnd);
                    if bytesToEnd == 0 {
                        return;
                    }
                    *(*s).write = *src_mut;
                    (*s).write = (*s).write.add(1);
                    src_mut = src_mut.add(1);
                    bytesToEnd -= 1;
                    if src_mut == (*s).end {
                        src_mut = (*s).window.as_mut_ptr();
                    }
                    (*infCodes).len -= 1;
                }
                (*infCodes).mode = inflate_codes_mode::START;
                break;
            }
            inflate_codes_mode::LIT => {
                bytesToEnd = needout(z, s, bytesToEnd);
                if bytesToEnd == 0 {
                    return;
                }
                *(*s).write = (*infCodes).lit as u8;
                (*s).write = (*s).write.add(1);
                bytesToEnd -= 1;
                (*infCodes).mode = inflate_codes_mode::START;
                break;
            }
            inflate_codes_mode::WASH => {
                if (*s).bitk > 7 {
                    (*s).bitk -= 8;
                    (*z).avail_in += 1;
                    (*z).total_in -= 1;
                    (*z).next_in = (*z).next_in.sub(1);
                }
                inflate_flush(z, s);
                bytesToEnd = get_remaining(s);
                if (*s).read != (*s).write {
                    inflate_error = b"Inflate data: read != write while in WASH\0".as_ptr() as *const c_char;
                    inflate_flush(z, s);
                    return;
                }
                (*infCodes).mode = inflate_codes_mode::END;
            }
            inflate_codes_mode::END => {
                (*z).error = EStatus::Z_STREAM_END;
                inflate_flush(z, s);
                return;
            }
            inflate_codes_mode::BADCODE => {
                (*z).error = EStatus::Z_DATA_ERROR;
                inflate_flush(z, s);
                return;
            }
        }
    }
}

// ===============================================================================
// ===============================================================================

unsafe fn inflate_codes_new(z: *mut z_stream_s, bl: usize, bd: usize, lengthTree: *mut inflate_huft_t, distTree: *mut inflate_huft_t) -> *mut inflate_codes_state_t {
    let c = Z_Malloc(mem::size_of::<inflate_codes_state_t>() as c_int, TAG_INFLATE, 1) as *mut inflate_codes_state_t;
    (*c).mode = inflate_codes_mode::START;
    (*c).lbits = bl as u8;
    (*c).dbits = bd as u8;
    (*c).ltree = lengthTree;
    (*c).dtree = distTree;

    c
}

// ===============================================================================
// Generate Huffman trees for efficient decoding
// ===============================================================================

unsafe fn huft_build(b: *mut usize, numCodes: usize, s: usize, d: *const usize, e: *const usize, t: *mut *mut inflate_huft_t, m: *mut usize, hp: *mut inflate_huft_t, hn: *mut usize, workspace: *mut usize) -> EStatus {
    let mut codeCounter: usize;
    let mut bitLengths: [usize; BMAX + 1] = [0; BMAX + 1];
    let mut bitOffsets: [usize; BMAX + 1] = [0; BMAX + 1];
    let mut f: usize;
    let mut maxCodeLen: i32;
    let mut tableLevel: i32;
    let mut i: usize;
    let mut j: usize;
    let mut bitsPerCode: i32;
    let mut bitsPerTable: usize;
    let mut bitsBeforeTable: i32;
    let mut p: *mut usize;
    let mut q: *mut inflate_huft_t;
    let mut r: inflate_huft_t;
    let mut tableStack: [*mut inflate_huft_t; BMAX] = [ptr::null_mut(); BMAX];
    let mut xp: *mut usize;
    let mut dummyCodes: i32;
    let mut entryCount: usize;

    // Generate counts for each bit length
    p = b;
    i = numCodes;
    loop {
        bitLengths[*p] += 1;
        p = p.add(1);
        i -= 1;
        if i == 0 {
            break;
        }
    }

    // null input--all zero length codes
    if bitLengths[0] == numCodes {
        *t = ptr::null_mut();
        *m = 0;
        return EStatus::Z_OK;
    }

    // Find minimum and maximum length, bound *m by those
    bitsPerTable = *m;
    j = 1;
    loop {
        if j > BMAX {
            break;
        }
        if bitLengths[j] != 0 {
            break;
        }
        j += 1;
    }
    bitsPerCode = j as i32;

    if (bitsPerTable as i32) < bitsPerCode {
        bitsPerTable = bitsPerCode as usize;
    }
    i = BMAX;
    loop {
        if i == 0 {
            break;
        }
        if bitLengths[i] != 0 {
            break;
        }
        i -= 1;
    }
    maxCodeLen = i as i32;

    if (bitsPerTable as i32) > maxCodeLen {
        bitsPerTable = maxCodeLen as usize;
    }
    *m = bitsPerTable;

    // Adjust last length count to fill out codes, if needed
    dummyCodes = (1 << bitsPerCode) as i32;
    let mut j_loop = bitsPerCode as usize;
    loop {
        if j_loop >= maxCodeLen as usize {
            break;
        }
        dummyCodes -= bitLengths[j_loop] as i32;
        if dummyCodes < 0 {
            return EStatus::Z_DATA_ERROR;
        }
        dummyCodes = (dummyCodes as i32) << 1;
        j_loop += 1;
    }
    dummyCodes -= bitLengths[maxCodeLen as usize] as i32;
    if dummyCodes < 0 {
        return EStatus::Z_DATA_ERROR;
    }
    bitLengths[maxCodeLen as usize] += dummyCodes as usize;

    // Generate starting offsets into the value table for each length
    bitOffsets[1] = 0;
    j = 0;
    p = bitLengths.as_mut_ptr().add(1);
    xp = bitOffsets.as_mut_ptr().add(2);
    let mut i_loop = (maxCodeLen - 1) as usize;
    loop {
        if i_loop == 0 {
            break;
        }
        j += *p;
        *xp = j;
        p = p.add(1);
        xp = xp.add(1);
        i_loop -= 1;
    }

    // Make a table of values in order of bit lengths
    p = b;
    i = 0;
    loop {
        j = *p;
        if j != 0 {
            *workspace.add(bitOffsets[j]) = i;
            bitOffsets[j] += 1;
        }
        p = p.add(1);
        i += 1;
        if i >= numCodes {
            break;
        }
    }

    // set numCodes to length of workspace
    let numCodes_len = bitOffsets[maxCodeLen as usize];

    // Generate the Huffman codes and for each, make the table entries
    bitOffsets[0] = 0;
    i = 0;
    p = workspace;
    tableLevel = -1;
    bitsBeforeTable = bitsPerTable as i32;
    bitsBeforeTable = -bitsBeforeTable;
    tableStack[0] = ptr::null_mut();
    q = ptr::null_mut();
    entryCount = 0;

    let mut bitsPerCode_loop = bitsPerCode;
    loop {
        if bitsPerCode_loop > maxCodeLen {
            break;
        }
        codeCounter = bitLengths[bitsPerCode_loop as usize];
        loop {
            if codeCounter == 0 {
                break;
            }
            codeCounter -= 1;

            while bitsPerCode_loop > bitsBeforeTable + bitsPerTable as i32 {
                tableLevel += 1;
                bitsBeforeTable += bitsPerTable as i32;

                entryCount = (maxCodeLen - bitsBeforeTable) as usize;
                entryCount = if entryCount > bitsPerTable { bitsPerTable } else { entryCount };
                j = (bitsPerCode_loop - bitsBeforeTable) as usize;
                f = 1 << j;
                if f > codeCounter + 1 {
                    f -= codeCounter + 1;
                    xp = bitLengths.as_mut_ptr().add(bitsPerCode_loop as usize);
                    if (j as i32) < (entryCount as i32) {
                        j += 1;
                        loop {
                            if j >= entryCount {
                                break;
                            }
                            f <<= 1;
                            if f <= *xp.add(1) {
                                break;
                            }
                            f -= *xp.add(1);
                            xp = xp.add(1);
                            j += 1;
                        }
                    }
                }
                entryCount = 1 << j;

                if *hn + entryCount > MANY {
                    return EStatus::Z_DATA_ERROR;
                }
                q = hp.add(*hn);
                tableStack[tableLevel as usize] = q;
                *hn += entryCount;

                if tableLevel != 0 {
                    bitOffsets[tableLevel as usize] = i;
                    r.Bits = bitsPerTable as u8;
                    r.Exop = j as u8;
                    j = i >> (bitsBeforeTable - bitsPerTable as i32) as usize;
                    r.base = (q as usize - tableStack[(tableLevel - 1) as usize] as usize - j) as usize;
                    *tableStack[(tableLevel - 1) as usize].add(j) = r;
                } else {
                    *t = q;
                }
            }

            // set up table entry in r
            r.Bits = (bitsPerCode_loop - bitsBeforeTable) as u8;
            if p >= workspace.add(numCodes_len) {
                r.Exop = 128 + 64;
            } else if *p < s {
                r.Exop = if *p < 256 { 0 } else { 32 + 64 };
                r.base = *p;
                p = p.add(1);
            } else {
                r.Exop = (*e.add(*p - s) + 16 + 64) as u8;
                r.base = *d.add(*p - s);
                p = p.add(1);
            }

            // fill code-like entries with r
            f = 1 << (bitsPerCode_loop - bitsBeforeTable) as usize;
            j = i >> bitsBeforeTable as usize;
            loop {
                if j >= entryCount {
                    break;
                }
                *q.add(j) = r;
                j += f;
            }

            // backwards increment the bitsPerCode-bit code i
            j = 1 << (bitsPerCode_loop - 1) as usize;
            loop {
                if (i & j) == 0 {
                    break;
                }
                i ^= j;
                j >>= 1;
            }
            i ^= j;

            // backup over finished tables
            loop {
                if (i & (((1 << bitsBeforeTable) - 1) as usize)) == bitOffsets[tableLevel as usize] {
                    break;
                }
                tableLevel -= 1;
                bitsBeforeTable -= bitsPerTable as i32;
            }
        }
        bitsPerCode_loop += 1;
    }

    // Return Z_BUF_ERROR if we were given an incomplete table
    if (dummyCodes != 0) && (maxCodeLen != 1) {
        return EStatus::Z_BUF_ERROR;
    }
    EStatus::Z_OK
}

// ===============================================================================

unsafe fn inflate_trees_bits(z: *mut z_stream_s, c: *mut usize, bb: *mut usize, tb: *mut *mut inflate_huft_t, hp: *mut inflate_huft_t) {
    let mut hn: usize = 0;
    let mut workspace: [usize; 19] = [0; 19];

    (*z).error = huft_build(c, 19, 19, ptr::null(), ptr::null(), tb, bb, hp, &mut hn, workspace.as_mut_ptr());
    if (*z).error as u32 == EStatus::Z_DATA_ERROR as u32 {
        inflate_error = b"Inflate data: Oversubscribed dynamic bit lengths tree\0".as_ptr() as *const c_char;
    } else if ((*z).error as u32 == EStatus::Z_BUF_ERROR as u32) || (*bb == 0) {
        inflate_error = b"Inflate data: Incomplete dynamic bit lengths tree\0".as_ptr() as *const c_char;
        (*z).error = EStatus::Z_DATA_ERROR;
    }
}

// ===============================================================================

unsafe fn inflate_trees_dynamic(z: *mut z_stream_s, numLiteral: usize, numDist: usize, c: *mut usize, bl: *mut usize, bd: *mut usize, tl: *mut *mut inflate_huft_t, td: *mut *mut inflate_huft_t, hp: *mut inflate_huft_t) {
    let mut hn: usize = 0;
    let mut workspace: [usize; 288] = [0; 288];

    // build literal/length tree
    (*z).error = huft_build(c, numLiteral, 257, CPLENS.as_ptr(), CPLEXT.as_ptr(), tl, bl, hp, &mut hn, workspace.as_mut_ptr());
    if ((*z).error as u32 != EStatus::Z_OK as u32) || (*bl == 0) {
        inflate_error = b"Inflate data: Erroneous literal/length tree\0".as_ptr() as *const c_char;
        (*z).error = EStatus::Z_DATA_ERROR;
        return;
    }
    // build distance tree
    (*z).error = huft_build(c.add(numLiteral), numDist, 0, CPDIST.as_ptr(), EXTRA_DBITS.as_ptr(), td, bd, hp, &mut hn, workspace.as_mut_ptr());
    if ((*z).error as u32 != EStatus::Z_OK as u32) || ((*bd == 0) && (numLiteral > 257)) {
        inflate_error = b"Inflate data: Erroneous distance tree\0".as_ptr() as *const c_char;
        (*z).error = EStatus::Z_DATA_ERROR;
        return;
    }
}

// ===============================================================================

unsafe fn inflate_trees_fixed(z: *mut z_stream_s, bl: *mut usize, bd: *mut usize, tl: *mut *mut inflate_huft_t, td: *mut *mut inflate_huft_t) {
    *bl = FIXED_BL;
    *bd = FIXED_BD;
    *tl = FIXED_TL.as_ptr() as *mut inflate_huft_t;
    *td = FIXED_TD.as_ptr() as *mut inflate_huft_t;
    (*z).error = EStatus::Z_OK;
}

// ===============================================================================
// ===============================================================================

unsafe fn inflate_blocks(s: *mut inflate_blocks_state_t, z: *mut z_stream_s) {
    let mut t: usize;
    let mut bytesToEnd: usize;
    let mut bl: usize;
    let mut bd: usize;
    let mut lengthTree: *mut inflate_huft_t = ptr::null_mut();
    let mut distTree: *mut inflate_huft_t = ptr::null_mut();
    let mut c: *mut inflate_codes_state_t;

    // copy input/output information to locals (UPDATE macro restores)
    bytesToEnd = if (*s).write < (*s).read {
        (*s).read as usize - (*s).write as usize - 1
    } else {
        (*s).end as usize - (*s).write as usize
    };

    // process input based on current state
    loop {
        match (*s).mode {
            inflate_block_mode::TYPE => {
                if !getbits(z, s, 3) {
                    return;
                }
                t = (*s).bitb & 7;
                (*s).last = if (t & 1) != 0 { 1 } else { 0 };

                match t >> 1 {
                    STORED_BLOCK => {
                        (*s).bitb >>= 3;
                        (*s).bitk -= 3;
                        t = (*s).bitk & 7;
                        (*s).bitb >>= t;
                        (*s).bitk -= t;
                        (*s).mode = inflate_block_mode::LENS;
                    }
                    STATIC_TREES => {
                        inflate_trees_fixed(z, &mut bl, &mut bd, &mut lengthTree, &mut distTree);
                        (*s).decode_codes = inflate_codes_new(z, bl, bd, lengthTree, distTree);
                        (*s).bitb >>= 3;
                        (*s).bitk -= 3;
                        (*s).mode = inflate_block_mode::CODES;
                    }
                    DYN_TREES => {
                        (*s).bitb >>= 3;
                        (*s).bitk -= 3;
                        (*s).mode = inflate_block_mode::TABLE;
                    }
                    MODE_ILLEGAL => {
                        (*s).bitb >>= 3;
                        (*s).bitk -= 3;
                        (*s).mode = inflate_block_mode::BAD;
                        inflate_error = b"Inflate data: Invalid block type\0".as_ptr() as *const c_char;
                        (*z).error = EStatus::Z_DATA_ERROR;
                        inflate_flush(z, s);
                        return;
                    }
                    _ => {}
                }
            }
            inflate_block_mode::LENS => {
                if !getbits(z, s, 32) {
                    return;
                }
                if ((!(*s).bitb) >> 16) != ((*s).bitb & 0xffff) {
                    (*s).mode = inflate_block_mode::BAD;
                    inflate_error = b"Inflate data: Invalid stored block lengths\0".as_ptr() as *const c_char;
                    (*z).error = EStatus::Z_DATA_ERROR;
                    inflate_flush(z, s);
                    return;
                }
                (*s).left = (*s).bitb & 0xffff;
                (*s).bitb = 0;
                (*s).bitk = 0;
                (*s).mode = if (*s).left != 0 {
                    inflate_block_mode::STORED
                } else if (*s).last != 0 {
                    inflate_block_mode::DRY
                } else {
                    inflate_block_mode::TYPE
                };
            }
            inflate_block_mode::STORED => {
                if (*z).avail_in == 0 {
                    inflate_flush(z, s);
                    return;
                }
                bytesToEnd = needout(z, s, bytesToEnd);
                if bytesToEnd == 0 {
                    return;
                }
                t = (*s).left;
                if t > (*z).avail_in {
                    t = (*z).avail_in;
                }
                if t > bytesToEnd {
                    t = bytesToEnd;
                }
                memcpy((*s).write, (*z).next_in, t);
                (*z).next_in = (*z).next_in.add(t);
                (*z).avail_in -= t;
                (*z).total_in += t;
                (*s).write = (*s).write.add(t);
                bytesToEnd -= t;
                (*s).left -= t;
                if (*s).left != 0 {
                    break;
                }
                (*s).mode = if (*s).last != 0 {
                    inflate_block_mode::DRY
                } else {
                    inflate_block_mode::TYPE
                };
            }
            inflate_block_mode::TABLE => {
                if !getbits(z, s, 14) {
                    return;
                }
                t = (*s).bitb & 0x3fff;
                (*s).table = t;
                if ((t & 0x1f) > 29) || (((t >> 5) & 0x1f) > 29) {
                    (*s).mode = inflate_block_mode::BAD;
                    inflate_error = b"Inflate data: Too many length or distance symbols\0".as_ptr() as *const c_char;
                    (*z).error = EStatus::Z_DATA_ERROR;
                    inflate_flush(z, s);
                    return;
                }
                t = 258 + (t & 0x1f) + ((t >> 5) & 0x1f);
                (*s).blens = Z_Malloc((t * mem::size_of::<usize>()) as c_int, TAG_INFLATE, 0) as *mut usize;
                (*s).bitb >>= 14;
                (*s).bitk -= 14;
                (*s).index = 0;
                (*s).mode = inflate_block_mode::BTREE;
            }
            inflate_block_mode::BTREE => {
                while (*s).index < 4 + ((*s).table >> 10) {
                    if !getbits(z, s, 3) {
                        return;
                    }
                    *(*s).blens.add(BORDER[(*s).index]) = (*s).bitb & 7;
                    (*s).index += 1;
                    (*s).bitb >>= 3;
                    (*s).bitk -= 3;
                }
                while (*s).index < 19 {
                    *(*s).blens.add(BORDER[(*s).index]) = 0;
                    (*s).index += 1;
                }
                (*s).bb = 7;
                inflate_trees_bits(z, (*s).blens, &mut (*s).bb, &mut (*s).tb, (*s).hufts);
                if (*z).error as u32 != EStatus::Z_OK as u32 {
                    Z_Free((*s).blens as *mut core::ffi::c_void);
                    (*s).mode = inflate_block_mode::BAD;
                    inflate_flush(z, s);
                    return;
                }
                (*s).index = 0;
                (*s).mode = inflate_block_mode::DTREE;
            }
            inflate_block_mode::DTREE => {
                loop {
                    t = (*s).table;
                    if !((*s).index < 258 + (t & 0x1f) + ((t >> 5) & 0x1f)) {
                        break;
                    }

                    t = (*s).bb;
                    if !getbits(z, s, t) {
                        return;
                    }
                    let h = (*s).tb.add((*s).bitb & INFLATE_MASK[t]);
                    t = (*h).Bits as usize;
                    let c_val = (*h).base;
                    if c_val < 16 {
                        (*s).bitb >>= t;
                        (*s).bitk -= t;
                        *(*s).blens.add((*s).index) = c_val;
                        (*s).index += 1;
                    } else {
                        let i = if c_val == 18 { 7 } else { c_val - 14 };
                        let mut j = if c_val == 18 { 11 } else { 3 };
                        if !getbits(z, s, t + i) {
                            return;
                        }
                        (*s).bitb >>= t;
                        (*s).bitk -= t;
                        j += (*s).bitb & INFLATE_MASK[i];
                        (*s).bitb >>= i;
                        (*s).bitk -= i;
                        i = (*s).index;
                        t = (*s).table;
                        if (i + j > 258 + (t & 0x1f) + ((t >> 5) & 0x1f)) || ((c_val == 16) && (i < 1)) {
                            Z_Free((*s).blens as *mut core::ffi::c_void);
                            (*s).mode = inflate_block_mode::BAD;
                            inflate_error = b"Inflate data: Invalid bit length repeat\0".as_ptr() as *const c_char;
                            (*z).error = EStatus::Z_DATA_ERROR;
                            inflate_flush(z, s);
                            return;
                        }
                        let c_copy = if c_val == 16 {
                            *(*s).blens.add(i - 1)
                        } else {
                            0
                        };
                        let mut i_copy = i;
                        loop {
                            if j == 0 {
                                break;
                            }
                            *(*s).blens.add(i_copy) = c_copy;
                            i_copy += 1;
                            j -= 1;
                        }
                        (*s).index = i_copy;
                    }
                }
                (*s).tb = ptr::null_mut();

                bl = 9;
                bd = 6;
                t = (*s).table;
                inflate_trees_dynamic(z, 257 + (t & 0x1f), 1 + ((t >> 5) & 0x1f), (*s).blens, &mut bl, &mut bd, &mut lengthTree, &mut distTree, (*s).hufts);
                Z_Free((*s).blens as *mut core::ffi::c_void);
                if (*z).error as u32 != EStatus::Z_OK as u32 {
                    (*s).mode = inflate_block_mode::BAD;
                    inflate_flush(z, s);
                    return;
                }
                c = inflate_codes_new(z, bl, bd, lengthTree, distTree);
                (*s).decode_codes = c;
                (*s).mode = inflate_block_mode::CODES;
            }
            inflate_block_mode::CODES => {
                inflate_codes(z, s);
                if (*z).error as u32 != EStatus::Z_STREAM_END as u32 {
                    inflate_flush(z, s);
                    return;
                }
                (*z).error = EStatus::Z_OK;
                Z_Free((*s).decode_codes as *mut core::ffi::c_void);
                bytesToEnd = if (*s).write < (*s).read {
                    (*s).read as usize - (*s).write as usize - 1
                } else {
                    (*s).end as usize - (*s).write as usize
                };
                if (*s).last == 0 {
                    (*s).mode = inflate_block_mode::TYPE;
                    break;
                }
                (*s).mode = inflate_block_mode::DRY;
            }
            inflate_block_mode::DRY => {
                inflate_flush(z, s);
                bytesToEnd = if (*s).write < (*s).read {
                    (*s).read as usize - (*s).write as usize - 1
                } else {
                    (*s).end as usize - (*s).write as usize
                };
                if (*s).read != (*s).write {
                    inflate_error = b"Inflate data: read != write in DRY\0".as_ptr() as *const c_char;
                    inflate_flush(z, s);
                    return;
                }
                (*s).mode = inflate_block_mode::DONE;
            }
            inflate_block_mode::DONE => {
                (*z).error = EStatus::Z_STREAM_END;
                inflate_flush(z, s);
                return;
            }
            inflate_block_mode::BAD => {
                (*z).error = EStatus::Z_DATA_ERROR;
                inflate_flush(z, s);
                return;
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Controlling routines
// -------------------------------------------------------------------------------------------------

pub unsafe fn inflateEnd(z: *mut z_stream_s) -> EStatus {
    assert!(!z.is_null());

    if !(*z).istate.is_null() && !(*(*z).istate).blocks.is_null() {
        inflate_blocks_free(z, (*(*z).istate).blocks);
        (*(*z).istate).blocks = ptr::null_mut();
    }
    if !(*z).istate.is_null() {
        Z_Free((*z).istate as *mut core::ffi::c_void);
        (*z).istate = ptr::null_mut();
    }
    EStatus::Z_OK
}

// ===============================================================================
// ===============================================================================

pub unsafe fn inflateInit(z: *mut z_stream_s, _flush: i32, noWrap: i32) -> EStatus {
    // initialize state
    assert!(!z.is_null());

    inflate_error = b"OK\0".as_ptr() as *const c_char;

    (*z).istate = Z_Malloc(mem::size_of::<inflate_state>() as c_int, TAG_INFLATE, 1) as *mut inflate_state;
    (*(*z).istate).blocks = ptr::null_mut();

    // handle nowrap option (no zlib header or check)
    (*(*z).istate).nowrap = noWrap;
    (*(*z).istate).wbits = MAX_WBITS as usize;

    // create inflate_blocks state
    (*(*z).istate).blocks = inflate_blocks_new(z, ptr::null());

    (*z).status = EStatus::Z_OK;
    // if(flush == Z_FINISH)
    // {
    //     z->status = Z_BUF_ERROR;
    // }

    // reset state
    (*(*z).istate).mode = inflate_mode::imMETHOD;
    if (*(*z).istate).nowrap != 0 {
        (*(*z).istate).mode = inflate_mode::imBLOCKS;
    }
    inflate_blocks_reset(z, (*(*z).istate).blocks);
    EStatus::Z_OK
}

// ===============================================================================
// ===============================================================================

pub unsafe fn inflate(z: *mut z_stream_s) -> EStatus {
    let mut b: usize;

    // Sanity check data
    assert!(!z.is_null());
    assert!(!(*z).istate.is_null());

    loop {
        match (*(*z).istate).mode {
            inflate_mode::imMETHOD => {
                if (*z).avail_in == 0 {
                    return (*z).status;
                }
                (*(*z).istate).method = *(*z).next_in as usize;
                (*z).next_in = (*z).next_in.add(1);
                (*z).avail_in -= 1;
                (*z).total_in += 1;
                if ((*(*z).istate).method & 0xf) != ZF_DEFLATED as usize {
                    (*(*z).istate).mode = inflate_mode::imBAD;
                    inflate_error = b"Inflate data: Unknown compression method\0".as_ptr() as *const c_char;
                    return EStatus::Z_DATA_ERROR;
                }
                if ((*(*z).istate).method >> 4) + 8 > (*(*z).istate).wbits {
                    (*(*z).istate).mode = inflate_mode::imBAD;
                    inflate_error = b"Inflate data: Invalid window size\0".as_ptr() as *const c_char;
                    return EStatus::Z_DATA_ERROR;
                }
                (*(*z).istate).mode = inflate_mode::imFLAG;
            }
            inflate_mode::imFLAG => {
                if (*z).avail_in == 0 {
                    return (*z).status;
                }
                b = *(*z).next_in as usize;
                (*z).next_in = (*z).next_in.add(1);
                (*z).avail_in -= 1;
                (*z).total_in += 1;
                if (((*(*z).istate).method << 8) + b) % 31 != 0 {
                    (*(*z).istate).mode = inflate_mode::imBAD;
                    inflate_error = b"Inflate data: Incorrect header check\0".as_ptr() as *const c_char;
                    return EStatus::Z_DATA_ERROR;
                }
                (*(*z).istate).mode = inflate_mode::imBLOCKS;
            }
            inflate_mode::imBLOCKS => {
                inflate_blocks((*(*z).istate).blocks, z);

                // Make sure everything processed ok
                if (*z).error as u32 == EStatus::Z_DATA_ERROR as u32 {
                    (*(*z).istate).mode = inflate_mode::imBAD;
                    return EStatus::Z_DATA_ERROR;
                }

                if (*z).error as u32 != EStatus::Z_STREAM_END as u32 {
                    return (*z).status;
                }
                (*(*z).istate).calcadler = (*(*z).istate).adler;
                inflate_blocks_reset(z, (*(*z).istate).blocks);
                if (*(*z).istate).nowrap != 0 {
                    (*(*z).istate).mode = inflate_mode::imDONE;
                    break;
                }
                (*(*z).istate).mode = inflate_mode::imCHECK4;
            }
            inflate_mode::imCHECK4 => {
                if (*z).avail_in == 0 {
                    return (*z).status;
                }
                (*(*z).istate).adler = (*(*z).next_in as usize) << 24;
                (*z).next_in = (*z).next_in.add(1);
                (*z).avail_in -= 1;
                (*z).total_in += 1;
                (*(*z).istate).mode = inflate_mode::imCHECK3;
            }
            inflate_mode::imCHECK3 => {
                if (*z).avail_in == 0 {
                    return (*z).status;
                }
                (*(*z).istate).adler += (*(*z).next_in as usize) << 16;
                (*z).next_in = (*z).next_in.add(1);
                (*z).avail_in -= 1;
                (*z).total_in += 1;
                (*(*z).istate).mode = inflate_mode::imCHECK2;
            }
            inflate_mode::imCHECK2 => {
                if (*z).avail_in == 0 {
                    return (*z).status;
                }
                (*(*z).istate).adler += (*(*z).next_in as usize) << 8;
                (*z).next_in = (*z).next_in.add(1);
                (*z).avail_in -= 1;
                (*z).total_in += 1;
                (*(*z).istate).mode = inflate_mode::imCHECK1;
            }
            inflate_mode::imCHECK1 => {
                if (*z).avail_in == 0 {
                    return (*z).status;
                }
                (*(*z).istate).adler += *(*z).next_in as usize;
                (*z).next_in = (*z).next_in.add(1);
                (*z).avail_in -= 1;
                (*z).total_in += 1;

                if (*(*z).istate).calcadler != (*(*z).istate).adler {
                    inflate_error = b"Inflate data: Failed Adler checksum\0".as_ptr() as *const c_char;
                    (*(*z).istate).mode = inflate_mode::imBAD;
                    break;
                }
                (*(*z).istate).mode = inflate_mode::imDONE;
            }
            inflate_mode::imDONE => {
                return EStatus::Z_STREAM_END;
            }
            inflate_mode::imBAD => {
                return EStatus::Z_DATA_ERROR;
            }
        }
    }
    EStatus::Z_STREAM_END
}

// ===============================================================================
// ===============================================================================

pub unsafe fn inflateError() -> *const c_char {
    inflate_error
}

// ===============================================================================
// External calls
// ===============================================================================

pub unsafe fn InflateFile(src: *mut u8, compressedSize: usize, dst: *mut u8, uncompressedSize: usize, noWrap: i32) -> u32 {
    let mut z: z_stream_s = mem::zeroed();

    inflateInit(&mut z, 0, noWrap);

    z.next_in = src;
    z.avail_in = compressedSize;
    z.next_out = dst;
    z.avail_out = uncompressedSize;

    if inflate(&mut z) as u32 != EStatus::Z_STREAM_END as u32 {
        inflate_error = b"Inflate data: Stream did not end\0".as_ptr() as *const c_char;
        inflateEnd(&mut z);
        return 0;
    }

    if z.avail_in != 0 {
        inflate_error = b"Inflate data: Remaining input data at stream end\0".as_ptr() as *const c_char;
        inflateEnd(&mut z);
        return 0;
    }
    if z.avail_out != 0 {
        inflate_error = b"Inflate data: Remaining output space at stream end\0".as_ptr() as *const c_char;
        inflateEnd(&mut z);
        return 0;
    }
    if z.total_in != compressedSize {
        inflate_error = b"Inflate data: Number of processed bytes != compressed size\0".as_ptr() as *const c_char;
        inflateEnd(&mut z);
        return 0;
    }
    if z.total_out != uncompressedSize {
        inflate_error = b"Inflate data: Number of bytes output != uncompressed size\0".as_ptr() as *const c_char;
        inflateEnd(&mut z);
        return 0;
    }
    inflateEnd(&mut z);
    1
}

// end
