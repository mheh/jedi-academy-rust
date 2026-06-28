#![allow(non_snake_case)]

use core::ffi::{c_int, c_ulong};

// ============================================================================
// LOCAL STUBS FOR ZLIB DEPENDENCIES
// ============================================================================
// The following types and constants are referenced by deflate.h but defined
// in other zlib headers (e.g., zlib.h, zconf.h). They are stubbed here with
// typical zlib values to allow this file to compile and be structurally valid.
// When other zlib headers are ported, these stubs may be replaced with imports.

pub const WINDOW_SIZE: usize = 32768;  // Typical: 1 << 15
pub const MIN_MATCH: usize = 3;
pub const MAX_MATCH: usize = 258;
pub const MAX_WBITS: usize = 15;
pub const D_CODES: usize = 30;

// Enum for flush modes - from zlib.h
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum EFlush {
    Z_NO_FLUSH = 0,
    Z_PARTIAL_FLUSH = 1,
    Z_SYNC_FLUSH = 2,
    Z_FINISH = 4,
}

// Enum for compression levels - from zlib.h
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ELevel {
    Z_NO_COMPRESSION = 0,
    Z_BEST_SPEED = 1,
    Z_DEFAULT_COMPRESSION = 6,
    Z_BEST_COMPRESSION = 9,
}

// Opaque struct for z_stream - full definition in zlib.h
#[repr(C)]
pub struct z_stream {
    _unused: [u8; 0],
}

// ============================================================================
// END STUBS
// ============================================================================

// Stream status
pub const INIT_STATE: c_ulong = 42;
pub const BUSY_STATE: c_ulong = 113;
pub const FINISH_STATE: c_ulong = 666;

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
pub const TOO_FAR: c_int = 32767;

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
pub const END_BLOCK: u8 = 256;

// Repeat previous bit length 3-6 times (2 bits of repeat count)
pub const REP_3_6: u8 = 16;

// Repeat a zero length 3-10 times  (3 bits of repeat count)
pub const REPZ_3_10: u8 = 17;

// Repeat a zero length 11-138 times  (7 bits of repeat count)
pub const REPZ_11_138: u8 = 18;

// Number of bits used within bi_buf. (bi_buf might be implemented on
// more than 16 bits on some systems.)
pub const BUF_SIZE: usize = 8 * 2;

// Minimum amount of lookahead, except at the end of the input file.
// See deflate.c for comments about the MIN_MATCH+1.
pub const MIN_LOOKAHEAD: usize = MAX_MATCH + MIN_MATCH + 1;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum block_state
{
    NEED_MORE = 0,                                    // block not completed, need more input or more output
    BLOCK_DONE = 1,                                  // block flush performed
    FINISH_STARTED = 2,                              // finish started, need only more output at next deflate
    FINISH_DONE = 3                                  // finish done, accept no more input or output
}

// Data structure describing a single value and its code string.
#[repr(C)]
pub union ct_data_fc
{
    pub freq: u16,								// frequency count
    pub code: u16,								// bit string
}

#[repr(C)]
pub union ct_data_dl
{
    pub dad: u16,								// father node in Huffman tree
    pub len: u16,								// length of bit string
}

#[repr(C)]
pub struct ct_data_s
{
    pub fc: ct_data_fc,
    pub dl: ct_data_dl,
}

pub type ct_data = ct_data_s;

#[repr(C)]
pub struct static_tree_desc_s
{
    pub static_tree: *const ct_data_s,				// static tree or NULL
    pub extra_bits: *const c_ulong,  				// extra bits for each code or NULL
    pub extra_base: c_ulong,	  				// base index for extra_bits
    pub elems: c_ulong,		  				// max number of elements in the tree
    pub max_length: c_ulong,	  				// max bit length for the codes
}

#[repr(C)]
pub struct tree_desc_s
{
    pub dyn_tree: *mut ct_data_s,				// the dynamic tree
    pub max_code: c_ulong,				// largest code with non zero frequency
    pub stat_desc: *mut static_tree_desc_s,				// the corresponding static tree
}

// Main structure which the deflate algorithm works from
#[repr(C)]
pub struct deflate_state_s
{
    pub z: *mut z_stream,								// pointer back to this zlib stream
    pub status: c_ulong,							// as the name implies

    pub last_flush: EFlush,						// value of flush param for previous deflate call
    pub noheader: c_int,						// suppress zlib header and adler32

    pub pending_buf: [u8; MAX_BLOCK_SIZE + 5],// output still pending
    pub pending_out: *mut u8,					// next pending byte to output to the stream
    pub pending: c_ulong,						// nb of bytes in the pending buffer

    // Sliding window. Input bytes are read into the second half of the window,
    // and move to the first half later to keep a dictionary of at least wSize
    // bytes. With this organization, matches are limited to a distance of
    // wSize-MAX_MATCH bytes, but this ensures that IO is always
    // performed with a length multiple of the block size. Also, it limits
    // the window size to 64K, which is quite useful on MSDOS.
    // To do: use the user input buffer as sliding window.
    pub window: [u8; WINDOW_SIZE * 2],

    // Link to older string with same hash index. To limit the size of this
    // array to 64K, this link is maintained only for the last 32K strings.
    // An index in this array is thus a window index modulo 32K.
    pub prev: [u16; WINDOW_SIZE],

    pub head: [u16; HASH_SIZE],				// Heads of the hash chains or NULL.

    pub ins_h: c_ulong,							// hash index of string to be inserted

    // Window position at the beginning of the current output block. Gets
    // negative when the window is moved backwards.
    pub block_start: c_int,

    pub match_length: c_ulong,					// length of best match
    pub prev_match: c_ulong,						// previous match
    pub match_available: c_ulong,				// set if previous match exists
    pub strstart: c_ulong,						// start of string to insert
    pub match_start: c_ulong,					// start of matching string
    pub lookahead: c_ulong,						// number of valid bytes ahead in window

    // Length of the best match at previous step. Matches not greater than this
    // are discarded. This is used in the lazy match evaluation.
    pub prev_length: c_ulong,

    // Attempt to find a better match only when the current match is strictly
    // smaller than this value. This mechanism is used only for compression	levels >= 4.
    pub max_lazy_match: c_ulong,

    pub good_match: c_ulong,						// Use a faster search when the previous match is longer than this
    pub nice_match: c_ulong,						// Stop searching when current match exceeds this

    // To speed up deflation, hash chains are never searched beyond this
    // length.  A higher limit improves compression ratio but degrades the speed.
    pub max_chain_length: c_ulong,

    pub level: ELevel,							// compression level (0..9)

    pub dyn_ltree: [ct_data_s; HEAP_SIZE],			// literal and length tree
    pub dyn_dtree: [ct_data_s; (2 * D_CODES) + 1], 	// distance tree
    pub bl_tree: [ct_data_s; (2 * BL_CODES) + 1],  	// Huffman tree for bit lengths

    pub l_desc: tree_desc_s,							// desc. for literal tree
    pub d_desc: tree_desc_s,							// desc. for distance tree
    pub bl_desc: tree_desc_s,						// desc. for bit length tree

    pub bl_count: [u16; MAX_WBITS + 1],		// number of codes at each bit length for an optimal tree

    // The sons of heap[n] are heap[2*n] and heap[2*n+1]. heap[0] is not used.
    // The same heap array is used to build all trees.
    pub heap: [c_ulong; (2 * L_CODES) + 1],		// heap used to build the Huffman trees
    pub heap_len: c_ulong,						// number of elements in the heap
    pub heap_max: c_ulong,						// element of largest frequency

    pub depth: [u8; (2 * L_CODES) + 1],		// Depth of each subtree used as tie breaker for trees of equal frequency

    pub l_buf: [u8; LIT_BUFSIZE],				// buffer for literals or lengths

    pub last_lit: c_ulong,						// running index in l_buf

    // Buffer for distances. To simplify the code, d_buf and l_buf have
    // the same number of elements. To use different lengths, an extra flag
    // array would be necessary.
    pub d_buf: [u16; LIT_BUFSIZE],

    pub opt_len: c_ulong,						// bit length of current block with optimal trees
    pub static_len: c_ulong,						// bit length of current block with static trees
    pub matches: c_ulong,						// number of string matches in current block
    pub last_eob_len: c_ulong,					// bit length of EOB code for last block

    pub bi_buf: u16,							// Output buffer. bits are inserted starting at the bottom (least significant bits).
    pub bi_valid: c_ulong,						// Number of valid bits in bi_buf.  All bits above the last valid bit are always zero.

    pub adler: c_ulong,
}

// Compression function. Returns the block state after the call.
pub type compress_func = extern "C" fn(*mut deflate_state_s, EFlush) -> block_state;

#[repr(C)]
pub struct config_s
{
   pub good_length: u16,				// reduce lazy search above this match length
   pub max_lazy: u16,					// do not perform lazy search above this match length
   pub nice_length: u16,				// quit search above this match length
   pub max_chain: u16,
   pub func: compress_func,
}

// end
