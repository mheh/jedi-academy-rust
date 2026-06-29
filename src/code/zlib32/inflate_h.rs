// Maximum size of dynamic tree.  The maximum found in a long but non-
// exhaustive search was 1004 huft structures (850 for length/literals
// and 154 for distances, the latter actually the result of an
// exhaustive search).  The actual maximum is not known, but the
// value below is more than safe.

pub const MANY: core::ffi::c_ulong = 1440;

// maximum bit length of any code (if BMAX needs to be larger than 16, then h and x[] should be ulong.)
pub const BMAX: core::ffi::c_ulong = 15;

pub type check_func = extern "C" fn(core::ffi::c_ulong, *const u8, core::ffi::c_ulong) -> core::ffi::c_ulong;

#[repr(C)]
pub enum inflate_block_mode {
    TYPE,       // get type bits (3, including end bit)
    LENS,       // get lengths for stored
    STORED,     // processing stored block
    TABLE,      // get table lengths
    BTREE,      // get bit lengths tree for a dynamic block
    DTREE,      // get length, distance trees for a dynamic block
    CODES,      // processing fixed or dynamic block
    DRY,        // output remaining window bytes
    DONE,       // finished last block, done
    BAD,        // got a data error--stuck here
}

// waiting for "i:"=input, "o:"=output, "x:"=nothing
#[repr(C)]
pub enum inflate_codes_mode {
    START,      // x: set up for LEN
    LEN,        // i: get length/literal/eob next
    LENEXT,     // i: getting length extra (have base)
    DIST,       // i: get distance next
    DISTEXT,    // i: getting distance extra
    COPY,       // o: copying bytes in window, waiting for space
    LIT,        // o: got literal, waiting for output space
    WASH,       // o: got eob, possibly still output waiting
    END,        // x: got eob and all data flushed
    BADCODE,    // x: got error
}

#[repr(C)]
pub enum inflate_mode {
    imMETHOD,   // waiting for method byte
    imFLAG,     // waiting for flag byte
    imBLOCKS,   // decompressing blocks
    imCHECK4,   // four check bytes to go
    imCHECK3,   // three check bytes to go
    imCHECK2,   // two check bytes to go
    imCHECK1,   // one check byte to go
    imDONE,     // finished check, done
    imBAD,      // got an error--stay here
}

#[repr(C)]
pub struct inflate_huft_t {
    pub Exop: u8,                   // number of extra bits or operation
    pub Bits: u8,                   // number of bits in this code or subcode
    pub base: core::ffi::c_ulong,   // literal, length base, distance base, or table offset
}

#[repr(C)]
pub struct inflate_codes_state_code_s {
    pub tree: *mut inflate_huft_t,  // pointer into tree
    pub need: core::ffi::c_ulong,   // bits needed
}

#[repr(C)]
pub struct inflate_codes_state_copy_s {
    pub get: core::ffi::c_ulong,    // bits to get for extra
    pub dist: core::ffi::c_ulong,   // distance back to copy from
}

#[repr(C)]
pub union inflate_codes_state_union_s {
    pub code: inflate_codes_state_code_s,   // if LEN or DIST, where in tree
    pub lit: core::ffi::c_ulong,            // if LIT, literal
    pub copy: inflate_codes_state_copy_s,   // if EXT or COPY, where and how much
}

// inflate codes private state
#[repr(C)]
pub struct inflate_codes_state_s {
    pub mode: inflate_codes_mode,               // current inflate_codes mode

    // mode dependent information
    pub len: core::ffi::c_ulong,
    pub submode: inflate_codes_state_union_s,   // submode

    // mode independent information
    pub lbits: u8,                      // ltree bits decoded per branch
    pub dbits: u8,                      // dtree bits decoder per branch
    pub ltree: *mut inflate_huft_t,     // literal/length/eob tree
    pub dtree: *mut inflate_huft_t,     // distance tree
}

pub type inflate_codes_state_t = inflate_codes_state_s;

#[repr(C)]
pub struct inflate_blocks_state_trees_s {
    pub table: core::ffi::c_ulong,              // table lengths (14 bits)
    pub index: core::ffi::c_ulong,              // index into blens (or border)
    pub blens: *mut core::ffi::c_ulong,         // bit lengths of codes
    pub bb: core::ffi::c_ulong,                 // bit length tree depth
    pub tb: *mut inflate_huft_t,                // bit length decoding tree
}

#[repr(C)]
pub struct inflate_blocks_state_decode_s {
    pub codes: *mut inflate_codes_state_t,
}

#[repr(C)]
pub union inflate_blocks_state_union_s {
    pub left: core::ffi::c_ulong,               // if STORED, bytes left to copy
    pub trees: inflate_blocks_state_trees_s,    // if DTREE, decoding info for trees
    pub decode: inflate_blocks_state_decode_s,  // if CODES, current state
}

// Window size from standard zlib (32KB)
const WINDOW_SIZE: usize = 32768;

// inflate blocks semi-private state
#[repr(C)]
pub struct inflate_blocks_state_s {
    // mode
    pub mode: inflate_block_mode,           // current inflate_block mode

    // mode dependent information
    pub submode: inflate_blocks_state_union_s,
    pub last: bool,                         // true if this block is the last block

    // mode independent information
    pub bitk: core::ffi::c_ulong,           // bits in bit buffer
    pub bitb: core::ffi::c_ulong,           // bit buffer
    pub hufts: *mut inflate_huft_t,         // single malloc for tree space
    pub window: [u8; WINDOW_SIZE],          // sliding window
    pub end: *mut u8,                       // one byte after sliding window
    pub read: *mut u8,                      // window read pointer
    pub write: *mut u8,                     // window write pointer
    pub check: core::ffi::c_ulong,          // check on output
}

pub type inflate_blocks_state_t = inflate_blocks_state_s;

// inflate private state
#[repr(C)]
pub struct inflate_state {
    pub mode: inflate_mode,                 // current inflate mode

    pub method: core::ffi::c_ulong,         // if FLAGS, method byte

    // mode independent information
    pub nowrap: core::ffi::c_int,           // flag for no wrapper
    pub wbits: core::ffi::c_ulong,          // log2(window size)  (8..15, defaults to 15)
    pub blocks: *mut inflate_blocks_state_t,   // current inflate_blocks state

    pub adler: core::ffi::c_ulong,
    pub calcadler: core::ffi::c_ulong,
}

// end
