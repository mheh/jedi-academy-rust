//! Mechanical port of `codemp/zlib32/deflate.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_int;

use super::zip_h::{
    byte, ulong, word, z_stream, D_CODES, MAX_MATCH, MAX_WBITS, MIN_MATCH, WINDOW_SIZE, EFlush,
    ELevel,
};

// Stream status
pub const INIT_STATE: ulong = 42;
pub const BUSY_STATE: ulong = 113;
pub const FINISH_STATE: ulong = 666;

pub const HASH_BITS: usize = 15;
pub const HASH_SIZE: usize = 1 << HASH_BITS;
pub const HASH_MASK: usize = HASH_SIZE - 1;

// Size of match buffer for literals/lengths.  There are 4 reasons for
// limiting lit_bufsize to 64K:
//   - frequencies can be kept in 16 bit counters
//   - if compression is not successful for the first block, all input
//     data is still in the window so we can still emit a stored block even
//     when input comes from standard input.  (This can also be done for
//     all blocks if lit_bufsize is not greater than 32K.)
//   - if compression is not successful for a file smaller than 64K, we can
//     even emit a stored file instead of a stored block (saving 5 bytes).
//     This is applicable only for zip (not gzip or zlib).
//   - creating new Huffman trees less frequently may not provide fast
//     adaptation to changes in the input data statistics. (Take for
//     example a binary file with poorly compressible code followed by
//     a highly compressible string table.) Smaller buffer sizes give
//     fast adaptation but have of course the overhead of transmitting
//     trees more frequently.
//   - I can't count above 4
pub const LIT_BUFSIZE: usize = 1 << 14;

pub const MAX_BLOCK_SIZE: usize = 0xffff;

// Number of bits by which ins_h must be shifted at each input
// step. It must be such that after MIN_MATCH steps, the oldest
// byte no longer takes part in the hash key.
pub const HASH_SHIFT: usize = (HASH_BITS + MIN_MATCH - 1) / MIN_MATCH;

// Matches of length 3 are discarded if their distance exceeds TOO_FAR
pub const TOO_FAR: ulong = 32767;

// Number of length codes, not counting the special END_BLOCK code
pub const LENGTH_CODES: usize = 29;

// Number of codes used to transfer the bit lengths
pub const BL_CODES: usize = 19;

// Number of literal bytes 0..255
pub const LITERALS: usize = 256;

// Number of Literal or Length codes, including the END_BLOCK code
pub const L_CODES: usize = LITERALS + 1 + LENGTH_CODES;

// See definition of array dist_code below
pub const DIST_CODE_LEN: usize = 512;

// Maximum heap size
pub const HEAP_SIZE: usize = 2 * L_CODES + 1;

// Index within the heap array of least frequent node in the Huffman tree
pub const SMALLEST: usize = 1;

// Bit length codes must not exceed MAX_BL_BITS bits
pub const MAX_BL_BITS: usize = 7;

// End of block literal code
pub const END_BLOCK: usize = 256;

// Repeat previous bit length 3-6 times (2 bits of repeat count)
pub const REP_3_6: usize = 16;

// Repeat a zero length 3-10 times  (3 bits of repeat count)
pub const REPZ_3_10: usize = 17;

// Repeat a zero length 11-138 times  (7 bits of repeat count)
pub const REPZ_11_138: usize = 18;

// Number of bits used within bi_buf. (bi_buf might be implemented on
// more than 16 bits on some systems.)
pub const BUF_SIZE: usize = 8 * 2;

// Minimum amount of lookahead, except at the end of the input file.
// See deflate.c for comments about the MIN_MATCH+1.
pub const MIN_LOOKAHEAD: usize = MAX_MATCH + MIN_MATCH + 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum block_state {
    NEED_MORE,      // block not completed, need more input or more output
    BLOCK_DONE,     // block flush performed
    FINISH_STARTED, // finish started, need only more output at next deflate
    FINISH_DONE,    // finish done, accept no more input or output
}

// Data structure describing a single value and its code string.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ct_data_s_fc {
    pub freq: word, // frequency count
    pub code: word, // bit string
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ct_data_s_dl {
    pub dad: word, // father node in Huffman tree
    pub len: word, // length of bit string
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ct_data_s {
    pub fc: ct_data_s_fc,
    pub dl: ct_data_s_dl,
}

pub type ct_data = ct_data_s;

const _: () = assert!(core::mem::size_of::<ct_data>() == 4);
const _: () = assert!(core::mem::align_of::<ct_data>() == 2);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct static_tree_desc_s {
    pub static_tree: *const ct_data, // static tree or NULL
    pub extra_bits: *const ulong,    // extra bits for each code or NULL
    pub extra_base: ulong,          // base index for extra_bits
    pub elems: ulong,               // max number of elements in the tree
    pub max_length: ulong,          // max bit length for the codes
}

pub type static_tree_desc = static_tree_desc_s;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct tree_desc_s {
    pub dyn_tree: *mut ct_data,              // the dynamic tree
    pub max_code: ulong,                    // largest code with non zero frequency
    pub stat_desc: *mut static_tree_desc,   // the corresponding static tree
}

pub type tree_desc = tree_desc_s;

// Main structure which the deflate algorithm works from
#[repr(C)]
pub struct deflate_state_s {
    pub z: *mut z_stream, // pointer back to this zlib stream
    pub status: ulong,   // as the name implies

    pub last_flush: EFlush, // value of flush param for previous deflate call
    pub noheader: c_int,   // suppress zlib header and adler32

    pub pending_buf: [byte; MAX_BLOCK_SIZE + 5], // output still pending
    pub pending_out: *mut byte,                  // next pending byte to output to the stream
    pub pending: ulong,                          // nb of bytes in the pending buffer

    // Sliding window. Input bytes are read into the second half of the window,
    // and move to the first half later to keep a dictionary of at least wSize
    // bytes. With this organization, matches are limited to a distance of
    // wSize-MAX_MATCH bytes, but this ensures that IO is always
    // performed with a length multiple of the block size. Also, it limits
    // the window size to 64K, which is quite useful on MSDOS.
    // To do: use the user input buffer as sliding window.
    pub window: [byte; WINDOW_SIZE * 2],

    // Link to older string with same hash index. To limit the size of this
    // array to 64K, this link is maintained only for the last 32K strings.
    // An index in this array is thus a window index modulo 32K.
    pub prev: [word; WINDOW_SIZE],

    pub head: [word; HASH_SIZE], // Heads of the hash chains or NULL.

    pub ins_h: ulong, // hash index of string to be inserted

    // Window position at the beginning of the current output block. Gets
    // negative when the window is moved backwards.
    pub block_start: c_int,

    pub match_length: ulong,    // length of best match
    pub prev_match: ulong,      // previous match
    pub match_available: ulong, // set if previous match exists
    pub strstart: ulong,        // start of string to insert
    pub match_start: ulong,     // start of matching string
    pub lookahead: ulong,       // number of valid bytes ahead in window

    // Length of the best match at previous step. Matches not greater than this
    // are discarded. This is used in the lazy match evaluation.
    pub prev_length: ulong,

    // Attempt to find a better match only when the current match is strictly
    // smaller than this value. This mechanism is used only for compression	levels >= 4.
    pub max_lazy_match: ulong,

    pub good_match: ulong, // Use a faster search when the previous match is longer than this
    pub nice_match: ulong, // Stop searching when current match exceeds this

    // To speed up deflation, hash chains are never searched beyond this
    // length.  A higher limit improves compression ratio but degrades the speed.
    pub max_chain_length: ulong,

    pub level: ELevel, // compression level (0..9)

    pub dyn_ltree: [ct_data; HEAP_SIZE],        // literal and length tree
    pub dyn_dtree: [ct_data; (2 * D_CODES) + 1], // distance tree
    pub bl_tree: [ct_data; (2 * BL_CODES) + 1], // Huffman tree for bit lengths

    pub l_desc: tree_desc,  // desc. for literal tree
    pub d_desc: tree_desc,  // desc. for distance tree
    pub bl_desc: tree_desc, // desc. for bit length tree

    pub bl_count: [word; MAX_WBITS + 1], // number of codes at each bit length for an optimal tree

    // The sons of heap[n] are heap[2*n] and heap[2*n+1]. heap[0] is not used.
    // The same heap array is used to build all trees.
    pub heap: [ulong; (2 * L_CODES) + 1], // heap used to build the Huffman trees
    pub heap_len: ulong,                  // number of elements in the heap
    pub heap_max: ulong,                  // element of largest frequency

    pub depth: [byte; (2 * L_CODES) + 1], // Depth of each subtree used as tie breaker for trees of equal frequency

    pub l_buf: [byte; LIT_BUFSIZE], // buffer for literals or lengths

    pub last_lit: ulong, // running index in l_buf

    // Buffer for distances. To simplify the code, d_buf and l_buf have
    // the same number of elements. To use different lengths, an extra flag
    // array would be necessary.
    pub d_buf: [word; LIT_BUFSIZE],

    pub opt_len: ulong,      // bit length of current block with optimal trees
    pub static_len: ulong,   // bit length of current block with static trees
    pub matches: ulong,      // number of string matches in current block
    pub last_eob_len: ulong, // bit length of EOB code for last block

    pub bi_buf: word,     // Output buffer. bits are inserted starting at the bottom (least significant bits).
    pub bi_valid: ulong, // Number of valid bits in bi_buf.  All bits above the last valid bit are always zero.

    pub adler: ulong,
}

pub type deflate_state = deflate_state_s;

// Compression function. Returns the block state after the call.
pub type compress_func = Option<unsafe extern "C" fn(s: *mut deflate_state, flush: EFlush) -> block_state>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct config_s {
    pub good_length: word, // reduce lazy search above this match length
    pub max_lazy: word,    // do not perform lazy search above this match length
    pub nice_length: word, // quit search above this match length
    pub max_chain: word,
    pub func: compress_func,
}

pub type config = config_s;

// end
