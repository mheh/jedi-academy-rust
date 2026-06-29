#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_variables, dead_code, clippy::all)]

use crate::code::game::q_shared_h::*;
use crate::code::qcommon::qcommon_h::*;
use crate::code::zlib32::zip_h::*;
use crate::code::zlib32::deflate_h::*;

use core::ffi::{c_char, c_int, c_ulong};
use core::ptr::{addr_of, addr_of_mut, null, null_mut};

#[cfg(feature = "timing")]
pub static mut totalDeflateTime: [c_int; Z_MAX_COMPRESSION as usize + 1] =
    [0; Z_MAX_COMPRESSION as usize + 1];
#[cfg(feature = "timing")]
pub static mut totalDeflateCount: [c_int; Z_MAX_COMPRESSION as usize + 1] =
    [0; Z_MAX_COMPRESSION as usize + 1];

// If you use the zlib library in a product, an acknowledgment is welcome
// in the documentation of your product. If for some reason you cannot
// include such an acknowledgment, I would appreciate that you keep this
// copyright string in the executable of your product.
pub static deflate_copyright: &[u8] =
    b"Deflate 1.1.3 Copyright 1995-1998 Jean-loup Gailly \0";

static mut deflate_error: *const c_char = b"OK\0".as_ptr() as *const c_char;

//  ALGORITHM
//
//  The "deflation" process depends on being able to identify portions
//  of the input text which are identical to earlier input (within a
//  sliding window trailing behind the input currently being processed).
//
//  The most straightforward technique turns out to be the fastest for
//  most input files: try all possible matches and select the longest.
//  The key feature of this algorithm is that insertions into the string
//  dictionary are very simple and thus fast, and deletions are avoided
//  completely. Insertions are performed at each input character, whereas
//  string matches are performed only when the previous match ends. So it
//  is preferable to spend more time in matches to allow very fast string
//  insertions and avoid deletions. The matching algorithm for small
//  strings is inspired from that of Rabin & Karp. A brute force approach
//  is used to find longer strings when a small match has been found.
//  A similar algorithm is used in comic (by Jan-Mark Wams) and freeze
//  (by Leonid Broukhis).
//
// ACKNOWLEDGEMENTS
//
//  The idea of lazy evaluation of matches is due to Jan-Mark Wams, and
//  I found it in 'freeze' written by Leonid Broukhis.
//  Thanks to many people for bug reports and testing.
//
// REFERENCES
//
//  Deutsch, L.P.,"DEFLATE Compressed Data Format Specification".
//  Available in ftp://ds.internic.net/rfc/rfc1951.txt
//
//  A description of the Rabin and Karp algorithm is given in the book
//     "Algorithms" by R. Sedgewick, Addison-Wesley, p252.
//
//  Fiala,E.R., and Greene,D.H.
//     Data Compression with Finite Windows, Comm.ACM, 32,4 (1989) 490-595

// ===============================================================================
//  A word is an index in the character window. We use short instead of int to
//  save space in the various tables. ulong is used only for parameter passing.

// Porting note: ct_data_s is initialized below using the union literal syntax.
// mk_ct(fc_val, dl_val) initializes fc.freq = fc_val, dl.dad = dl_val,
// which are the first fields of each union — matching C aggregate initialization.
const fn mk_ct(fc_val: u16, dl_val: u16) -> ct_data_s {
    ct_data_s {
        fc: ct_data_fc { freq: fc_val },
        dl: ct_data_dl { dad: dl_val },
    }
}

//  The static literal tree. Since the bit lengths are imposed, there is no
//  need for the L_CODES extra codes used during heap construction. However
//  The codes 286 and 287 are needed to build a canonical tree (see _tr_init
//  below).
static static_ltree: [ct_data_s; L_CODES + 2] = [
    mk_ct( 12,8), mk_ct(140,8), mk_ct( 76,8), mk_ct(204,8), mk_ct( 44,8),
    mk_ct(172,8), mk_ct(108,8), mk_ct(236,8), mk_ct( 28,8), mk_ct(156,8),
    mk_ct( 92,8), mk_ct(220,8), mk_ct( 60,8), mk_ct(188,8), mk_ct(124,8),
    mk_ct(252,8), mk_ct(  2,8), mk_ct(130,8), mk_ct( 66,8), mk_ct(194,8),
    mk_ct( 34,8), mk_ct(162,8), mk_ct( 98,8), mk_ct(226,8), mk_ct( 18,8),
    mk_ct(146,8), mk_ct( 82,8), mk_ct(210,8), mk_ct( 50,8), mk_ct(178,8),
    mk_ct(114,8), mk_ct(242,8), mk_ct( 10,8), mk_ct(138,8), mk_ct( 74,8),
    mk_ct(202,8), mk_ct( 42,8), mk_ct(170,8), mk_ct(106,8), mk_ct(234,8),
    mk_ct( 26,8), mk_ct(154,8), mk_ct( 90,8), mk_ct(218,8), mk_ct( 58,8),
    mk_ct(186,8), mk_ct(122,8), mk_ct(250,8), mk_ct(  6,8), mk_ct(134,8),
    mk_ct( 70,8), mk_ct(198,8), mk_ct( 38,8), mk_ct(166,8), mk_ct(102,8),
    mk_ct(230,8), mk_ct( 22,8), mk_ct(150,8), mk_ct( 86,8), mk_ct(214,8),
    mk_ct( 54,8), mk_ct(182,8), mk_ct(118,8), mk_ct(246,8), mk_ct( 14,8),
    mk_ct(142,8), mk_ct( 78,8), mk_ct(206,8), mk_ct( 46,8), mk_ct(174,8),
    mk_ct(110,8), mk_ct(238,8), mk_ct( 30,8), mk_ct(158,8), mk_ct( 94,8),
    mk_ct(222,8), mk_ct( 62,8), mk_ct(190,8), mk_ct(126,8), mk_ct(254,8),
    mk_ct(  1,8), mk_ct(129,8), mk_ct( 65,8), mk_ct(193,8), mk_ct( 33,8),
    mk_ct(161,8), mk_ct( 97,8), mk_ct(225,8), mk_ct( 17,8), mk_ct(145,8),
    mk_ct( 81,8), mk_ct(209,8), mk_ct( 49,8), mk_ct(177,8), mk_ct(113,8),
    mk_ct(241,8), mk_ct(  9,8), mk_ct(137,8), mk_ct( 73,8), mk_ct(201,8),
    mk_ct( 41,8), mk_ct(169,8), mk_ct(105,8), mk_ct(233,8), mk_ct( 25,8),
    mk_ct(153,8), mk_ct( 89,8), mk_ct(217,8), mk_ct( 57,8), mk_ct(185,8),
    mk_ct(121,8), mk_ct(249,8), mk_ct(  5,8), mk_ct(133,8), mk_ct( 69,8),
    mk_ct(197,8), mk_ct( 37,8), mk_ct(165,8), mk_ct(101,8), mk_ct(229,8),
    mk_ct( 21,8), mk_ct(149,8), mk_ct( 85,8), mk_ct(213,8), mk_ct( 53,8),
    mk_ct(181,8), mk_ct(117,8), mk_ct(245,8), mk_ct( 13,8), mk_ct(141,8),
    mk_ct( 77,8), mk_ct(205,8), mk_ct( 45,8), mk_ct(173,8), mk_ct(109,8),
    mk_ct(237,8), mk_ct( 29,8), mk_ct(157,8), mk_ct( 93,8), mk_ct(221,8),
    mk_ct( 61,8), mk_ct(189,8), mk_ct(125,8), mk_ct(253,8),
    mk_ct( 19,9), mk_ct(275,9), mk_ct(147,9), mk_ct(403,9), mk_ct( 83,9),
    mk_ct(339,9), mk_ct(211,9), mk_ct(467,9), mk_ct( 51,9), mk_ct(307,9),
    mk_ct(179,9), mk_ct(435,9), mk_ct(115,9), mk_ct(371,9), mk_ct(243,9),
    mk_ct(499,9), mk_ct( 11,9), mk_ct(267,9), mk_ct(139,9), mk_ct(395,9),
    mk_ct( 75,9), mk_ct(331,9), mk_ct(203,9), mk_ct(459,9), mk_ct( 43,9),
    mk_ct(299,9), mk_ct(171,9), mk_ct(427,9), mk_ct(107,9), mk_ct(363,9),
    mk_ct(235,9), mk_ct(491,9), mk_ct( 27,9), mk_ct(283,9), mk_ct(155,9),
    mk_ct(411,9), mk_ct( 91,9), mk_ct(347,9), mk_ct(219,9), mk_ct(475,9),
    mk_ct( 59,9), mk_ct(315,9), mk_ct(187,9), mk_ct(443,9), mk_ct(123,9),
    mk_ct(379,9), mk_ct(251,9), mk_ct(507,9), mk_ct(  7,9), mk_ct(263,9),
    mk_ct(135,9), mk_ct(391,9), mk_ct( 71,9), mk_ct(327,9), mk_ct(199,9),
    mk_ct(455,9), mk_ct( 39,9), mk_ct(295,9), mk_ct(167,9), mk_ct(423,9),
    mk_ct(103,9), mk_ct(359,9), mk_ct(231,9), mk_ct(487,9), mk_ct( 23,9),
    mk_ct(279,9), mk_ct(151,9), mk_ct(407,9), mk_ct( 87,9), mk_ct(343,9),
    mk_ct(215,9), mk_ct(471,9), mk_ct( 55,9), mk_ct(311,9), mk_ct(183,9),
    mk_ct(439,9), mk_ct(119,9), mk_ct(375,9), mk_ct(247,9), mk_ct(503,9),
    mk_ct( 15,9), mk_ct(271,9), mk_ct(143,9), mk_ct(399,9), mk_ct( 79,9),
    mk_ct(335,9), mk_ct(207,9), mk_ct(463,9), mk_ct( 47,9), mk_ct(303,9),
    mk_ct(175,9), mk_ct(431,9), mk_ct(111,9), mk_ct(367,9), mk_ct(239,9),
    mk_ct(495,9), mk_ct( 31,9), mk_ct(287,9), mk_ct(159,9), mk_ct(415,9),
    mk_ct( 95,9), mk_ct(351,9), mk_ct(223,9), mk_ct(479,9), mk_ct( 63,9),
    mk_ct(319,9), mk_ct(191,9), mk_ct(447,9), mk_ct(127,9), mk_ct(383,9),
    mk_ct(255,9), mk_ct(511,9),
    mk_ct(  0,7), mk_ct( 64,7), mk_ct( 32,7), mk_ct( 96,7), mk_ct( 16,7),
    mk_ct( 80,7), mk_ct( 48,7), mk_ct(112,7), mk_ct(  8,7), mk_ct( 72,7),
    mk_ct( 40,7), mk_ct(104,7), mk_ct( 24,7), mk_ct( 88,7), mk_ct( 56,7),
    mk_ct(120,7), mk_ct(  4,7), mk_ct( 68,7), mk_ct( 36,7), mk_ct(100,7),
    mk_ct( 20,7), mk_ct( 84,7), mk_ct( 52,7), mk_ct(116,7),
    mk_ct(  3,8), mk_ct(131,8), mk_ct( 67,8), mk_ct(195,8), mk_ct( 35,8),
    mk_ct(163,8), mk_ct( 99,8), mk_ct(227,8),
];

// The static distance tree. (Actually a trivial tree since all codes use 5 bits.)
static static_dtree: [ct_data_s; D_CODES] = [
    mk_ct( 0,5), mk_ct(16,5), mk_ct( 8,5), mk_ct(24,5), mk_ct( 4,5),
    mk_ct(20,5), mk_ct(12,5), mk_ct(28,5), mk_ct( 2,5), mk_ct(18,5),
    mk_ct(10,5), mk_ct(26,5), mk_ct( 6,5), mk_ct(22,5), mk_ct(14,5),
    mk_ct(30,5), mk_ct( 1,5), mk_ct(17,5), mk_ct( 9,5), mk_ct(25,5),
    mk_ct( 5,5), mk_ct(21,5), mk_ct(13,5), mk_ct(29,5), mk_ct( 3,5),
    mk_ct(19,5), mk_ct(11,5), mk_ct(27,5), mk_ct( 7,5), mk_ct(23,5),
];

// Distance codes. The first 256 values correspond to the distances
// 3 .. 258, the last 256 values correspond to the top 8 bits of
// the 15 bit distances.
static tr_dist_code: [u8; DIST_CODE_LEN] = [
     0,  1,  2,  3,  4,  4,  5,  5,  6,  6,  6,  6,  7,  7,  7,  7,  8,  8,  8,  8,
     8,  8,  8,  8,  9,  9,  9,  9,  9,  9,  9,  9, 10, 10, 10, 10, 10, 10, 10, 10,
    10, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11,
    11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 13, 13, 13, 13,
    13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13,
    13, 13, 13, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,
    14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,
    14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,
    14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,  0,  0, 16, 17,
    18, 18, 19, 19, 20, 20, 20, 20, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22, 22, 22,
    23, 23, 23, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
    24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25,
    26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26,
    26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 27, 27, 27,
    27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27,
    27, 27, 27, 27, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    28, 28, 28, 28, 28, 28, 28, 28, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29,
    29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29,
    29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29,
    29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29,
];

// length code for each normalized match length (0 == MIN_MATCH)
static tr_length_code: [u8; MAX_MATCH as usize - MIN_MATCH as usize + 1] = [
     0,  1,  2,  3,  4,  5,  6,  7,  8,  8,  9,  9, 10, 10, 11, 11, 12, 12, 12, 12,
    13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 16, 16, 16, 16,
    17, 17, 17, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19,
    19, 19, 19, 19, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20,
    21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22,
    22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 23, 23, 23,
    23, 23, 23, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
    25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25,
    25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 26, 26, 26,
    26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26,
    26, 26, 26, 26, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27,
    27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 28,
];

// First normalized length for each code (0 = MIN_MATCH)
static base_length: [c_int; LENGTH_CODES] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 24, 28, 32, 40, 48, 56,
    64, 80, 96, 112, 128, 160, 192, 224, 0,
];

// First normalized distance for each code (0 = distance of 1)
static base_dist: [c_int; D_CODES] = [
       0,     1,     2,     3,     4,     6,     8,    12,    16,    24,
      32,    48,    64,    96,   128,   192,   256,   384,   512,   768,
    1024,  1536,  2048,  3072,  4096,  6144,  8192, 12288, 16384, 24576,
];

// Note: the deflate() code requires max_lazy >= MIN_MATCH and max_chain >= 4
// For deflate_fast() (levels <= 3) good is ignored and lazy has a different
// meaning.

// Forward declarations (Rust does not require forward declarations; these are
// defined later in this file)
// deflate_stored, deflate_fast, deflate_slow defined below

// Values for max_lazy_match, good_match and max_chain_length, depending on
// the desired pack level (0..9). The values given below have been tuned to
// exclude worst case performance for pathological files. Better values may be
// found for specific files.

// Porting deviation: deflate_stored/fast/slow are defined as `unsafe extern "C" fn`
// (not plain `unsafe fn`) because they are stored as `compress_func` (extern "C" fn)
// function pointers in configuration_table. The unsafe transmute below removes
// the `unsafe` from the function pointer type to satisfy `compress_func`.
static configuration_table: [config_s; 10] = unsafe {
    // good lazy nice chain
    [
        config_s { good_length:  0, max_lazy:   0, nice_length:   0, max_chain:    0, func: core::mem::transmute(deflate_stored  as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) }, // store only
        config_s { good_length:  4, max_lazy:   4, nice_length:   8, max_chain:    4, func: core::mem::transmute(deflate_fast    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) }, // maximum speed, no lazy matches
        config_s { good_length:  4, max_lazy:   5, nice_length:  16, max_chain:    8, func: core::mem::transmute(deflate_fast    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) },
        config_s { good_length:  4, max_lazy:   6, nice_length:  32, max_chain:   32, func: core::mem::transmute(deflate_fast    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) },
        config_s { good_length:  4, max_lazy:   4, nice_length:  16, max_chain:   16, func: core::mem::transmute(deflate_slow    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) }, // lazy matches
        config_s { good_length:  8, max_lazy:  16, nice_length:  32, max_chain:   32, func: core::mem::transmute(deflate_slow    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) },
        config_s { good_length:  8, max_lazy:  16, nice_length: 128, max_chain:  128, func: core::mem::transmute(deflate_slow    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) },
        config_s { good_length:  8, max_lazy:  32, nice_length: 128, max_chain:  256, func: core::mem::transmute(deflate_slow    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) },
        config_s { good_length: 32, max_lazy: 128, nice_length: 258, max_chain: 1024, func: core::mem::transmute(deflate_slow    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) },
        config_s { good_length: 32, max_lazy: 258, nice_length: 258, max_chain: 4096, func: core::mem::transmute(deflate_slow    as unsafe extern "C" fn(*mut deflate_state_s, EFlush) -> block_state) }, // maximum compression
    ]
};

// extra bits for each length code
static mut extra_lbits: [c_ulong; LENGTH_CODES] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

// Extra bits for distance codes
#[no_mangle]
pub static mut extra_dbits: [c_ulong; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
];

// extra bits for each bit length code
static mut extra_blbits: [c_ulong; BL_CODES] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 7,
];

// The lengths of the bit length codes are sent in order of decreasing
// probability, to avoid transmitting the lengths for unused bit length codes.
static bl_order: [u8; BL_CODES] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

// Porting note: static_tree_desc_s contains raw pointer fields (*const ct_data_s,
// *const c_ulong), so these must be static mut. Pointer fields are initialized
// with addr_of! to the corresponding static arrays.
static mut static_l_desc: static_tree_desc_s = static_tree_desc_s {
    static_tree: addr_of!(static_ltree) as *const ct_data_s,
    extra_bits:  addr_of!(extra_lbits)  as *const c_ulong,
    extra_base:  LITERALS as c_ulong + 1,
    elems:       L_CODES as c_ulong,
    max_length:  15, // MAX_WBITS
};

static mut static_d_desc: static_tree_desc_s = static_tree_desc_s {
    static_tree: addr_of!(static_dtree) as *const ct_data_s,
    extra_bits:  addr_of!(extra_dbits)  as *const c_ulong,
    extra_base:  0,
    elems:       D_CODES as c_ulong,
    max_length:  15, // MAX_WBITS
};

static mut static_bl_desc: static_tree_desc_s = static_tree_desc_s {
    static_tree: null(),
    extra_bits:  addr_of!(extra_blbits) as *const c_ulong,
    extra_base:  0,
    elems:       BL_CODES as c_ulong,
    max_length:  MAX_BL_BITS as c_ulong,
};

// ===============================================================================
// Output bytes to the output stream. Inlined for speed
// ===============================================================================

#[inline]
unsafe fn put_byte(s: *mut deflate_state_s, c: u8) {
    let pending = (*s).pending as usize;
    (*s).pending_buf[pending] = c;
    (*s).pending += 1;
}

// Fixme: write as 1 short
#[inline]
unsafe fn put_short(s: *mut deflate_state_s, w: u16) {
    let pending = (*s).pending as usize;
    (*s).pending_buf[pending]     = (w & 0xff) as u8;
    (*s).pending_buf[pending + 1] = (w >> 8) as u8;
    (*s).pending += 2;
}

#[inline]
unsafe fn put_shortMSB(s: *mut deflate_state_s, w: u16) {
    let pending = (*s).pending as usize;
    (*s).pending_buf[pending]     = (w >> 8) as u8;
    (*s).pending_buf[pending + 1] = (w & 0xff) as u8;
    (*s).pending += 2;
}

#[inline]
unsafe fn put_longMSB(s: *mut deflate_state_s, l: c_ulong) {
    let pending = (*s).pending as usize;
    (*s).pending_buf[pending]     = (l >> 24) as u8;
    (*s).pending_buf[pending + 1] = (l >> 16) as u8;
    (*s).pending_buf[pending + 2] = (l >> 8)  as u8;
    (*s).pending_buf[pending + 3] = (l & 0xff) as u8;
    (*s).pending += 4;
}

// ===============================================================================
// Send a value on a given number of bits.
// IN assertion: length <= 16 and value fits in length bits.
// ===============================================================================

unsafe fn send_bits(s: *mut deflate_state_s, val: c_ulong, len: c_ulong) {
    assert!(len <= 16);
    assert!(val <= 65536);

    if (*s).bi_valid > (BUF_SIZE as c_ulong - len) {
        (*s).bi_buf |= (val << (*s).bi_valid) as u16;
        put_short(s, (*s).bi_buf);
        (*s).bi_buf = (val >> (BUF_SIZE as c_ulong - (*s).bi_valid)) as u16;
        (*s).bi_valid += len - BUF_SIZE as c_ulong;
    } else {
        (*s).bi_buf |= (val << (*s).bi_valid) as u16;
        (*s).bi_valid += len;
    }
}

// ===============================================================================
// Initialize a new block.
// ===============================================================================

unsafe fn init_block(s: *mut deflate_state_s) {
    let mut n: c_ulong; // iterates over tree elements

    // Initialize the trees.
    n = 0;
    while n < L_CODES as c_ulong {
        (*s).dyn_ltree[n as usize].fc.freq = 0;
        n += 1;
    }
    n = 0;
    while n < D_CODES as c_ulong {
        (*s).dyn_dtree[n as usize].fc.freq = 0;
        n += 1;
    }
    n = 0;
    while n < BL_CODES as c_ulong {
        (*s).bl_tree[n as usize].fc.freq = 0;
        n += 1;
    }
    (*s).dyn_ltree[END_BLOCK as usize].fc.freq = 1;
    (*s).opt_len    = 0;
    (*s).static_len = 0;
    (*s).last_lit   = 0;
    (*s).matches    = 0;
}

// ===============================================================================
// Initialize the tree data structures for a new zlib stream.
// ===============================================================================

unsafe fn tr_init(s: *mut deflate_state_s) {
    (*s).l_desc.dyn_tree  = (*s).dyn_ltree.as_mut_ptr();
    (*s).l_desc.stat_desc = addr_of_mut!(static_l_desc);

    (*s).d_desc.dyn_tree  = (*s).dyn_dtree.as_mut_ptr();
    (*s).d_desc.stat_desc = addr_of_mut!(static_d_desc);

    (*s).bl_desc.dyn_tree  = (*s).bl_tree.as_mut_ptr();
    (*s).bl_desc.stat_desc = addr_of_mut!(static_bl_desc);

    (*s).bi_buf   = 0;
    (*s).bi_valid = 0;
    // enough lookahead for inflate
    (*s).last_eob_len = 8;

    // Initialize the first block of the first file:
    init_block(s);
}

// ===============================================================================
// Compares to subtrees, using the tree depth as tie breaker when
// the subtrees have equal frequency. This minimizes the worst case length.
// ===============================================================================

unsafe fn smaller(tree: *const ct_data_s, son: c_ulong, daughter: c_ulong, depth: *const u8) -> bool {
    if (*tree.add(son as usize)).fc.freq < (*tree.add(daughter as usize)).fc.freq {
        return true;
    }
    if ((*tree.add(son as usize)).fc.freq == (*tree.add(daughter as usize)).fc.freq)
        && (*depth.add(son as usize) <= *depth.add(daughter as usize))
    {
        return true;
    }
    false
}

// ===============================================================================
// Restore the heap property by moving down the tree starting at node k,
// exchanging a node with the smallest of its two sons if necessary, stopping
// when the heap property is re-established (each father smaller than its
// two sons).
// ===============================================================================

unsafe fn pqdownheap(s: *mut deflate_state_s, tree: *mut ct_data_s, node: c_ulong) {
    let mut base: c_ulong;
    let mut sibling: c_ulong; // left son of node
    let mut node = node;

    base    = (*s).heap[node as usize];
    sibling = node << 1;

    while sibling <= (*s).heap_len {
        // Set sibling to the smallest of the two children
        if (sibling < (*s).heap_len)
            && smaller(tree, (*s).heap[sibling as usize + 1], (*s).heap[sibling as usize], (*s).depth.as_ptr())
        {
            sibling += 1;
        }
        // Exit if base is smaller than both sons
        if smaller(tree, base, (*s).heap[sibling as usize], (*s).depth.as_ptr()) {
            break;
        }
        // Exchange base with the smallest son
        (*s).heap[node as usize] = (*s).heap[sibling as usize];
        node = sibling;

        // And continue down the tree, setting sibling to the left son of base
        sibling <<= 1;
    }
    (*s).heap[node as usize] = base;
}

// ===============================================================================
// Compute the optimal bit lengths for a tree and update the total bit length
// for the current block.
// IN assertion: the fields freq and dad are set, heap[heap_max] and
//    above are the tree nodes sorted by increasing frequency.
// OUT assertions: the field len is set to the optimal bit length, the
//     array bl_count contains the frequencies for each bit length.
//     The length opt_len is updated; static_len is also updated if stree is
//     not null.
// ===============================================================================

unsafe fn gen_bitlen(s: *mut deflate_state_s, desc: *mut tree_desc_s) {
    let stree:      *const ct_data_s;
    let extra:      *const c_ulong;
    let base:       c_ulong;
    let max_length: c_ulong;
    let mut heapIdx: c_ulong; // heap index
    let mut n: c_ulong;
    let mut m: c_ulong;      // iterate over the tree elements
    let mut bits: c_ulong;   // bit length
    let mut xbits: c_ulong;  // extra bits
    let mut freq: u16;        // frequency
    let mut overflow: c_ulong; // number of elements with bit length too large

    stree      = (*(*desc).stat_desc).static_tree;
    extra      = (*(*desc).stat_desc).extra_bits;
    base       = (*(*desc).stat_desc).extra_base;
    max_length = (*(*desc).stat_desc).max_length;
    overflow   = 0;

    bits = 0;
    while bits <= 15 /* MAX_WBITS */ {
        (*s).bl_count[bits as usize] = 0;
        bits += 1;
    }

    // In a first pass, compute the optimal bit lengths (which may
    // overflow in the case of the bit length tree).
    // root of the heap
    (*(*desc).dyn_tree.add((*s).heap[(*s).heap_max as usize] as usize)).dl.len = 0;

    heapIdx = (*s).heap_max + 1;
    while heapIdx < HEAP_SIZE as c_ulong {
        n    = (*s).heap[heapIdx as usize];
        bits = (*(*desc).dyn_tree.add((*(*desc).dyn_tree.add(n as usize)).dl.dad as usize)).dl.len as c_ulong + 1;
        if bits > max_length {
            bits = max_length;
            overflow += 1;
        }
        // We overwrite tree[n].dl.dad which is no longer needed
        (*(*desc).dyn_tree.add(n as usize)).dl.len = bits as u16;

        // not a leaf node
        if n > (*desc).max_code {
            heapIdx += 1;
            continue;
        }

        (*s).bl_count[bits as usize] += 1;
        xbits = 0;
        if n >= base {
            xbits = *extra.add((n - base) as usize);
        }
        freq = (*(*desc).dyn_tree.add(n as usize)).fc.freq;
        (*s).opt_len += freq as c_ulong * (bits + xbits);
        if !stree.is_null() {
            (*s).static_len += freq as c_ulong * ((*stree.add(n as usize)).dl.len as c_ulong + xbits);
        }
        heapIdx += 1;
    }
    if overflow == 0 {
        return;
    }

    // Find the first bit length which could increase
    loop {
        bits = max_length - 1;
        while (*s).bl_count[bits as usize] == 0 {
            bits -= 1;
        }
        // move one leaf down the tree
        (*s).bl_count[bits as usize]     -= 1;
        // move one overflow item as its brother
        (*s).bl_count[bits as usize + 1] += 2;
        // The brother of the overflow item also moves one step up,
        // but this does not affect bl_count[max_length]
        (*s).bl_count[max_length as usize] -= 1;
        overflow = overflow.wrapping_sub(2);
        if overflow == 0 { break; }
    }

    // Now recompute all bit lengths, scanning in increasing frequency.
    // heapIdx is still equal to HEAP_SIZE. (It is simpler to reconstruct all
    // lengths instead of fixing only the wrong ones. This idea is taken
    // from 'ar' written by Haruhiko Okumura.)
    bits = max_length;
    while bits > 0 {
        n = (*s).bl_count[bits as usize] as c_ulong;
        while n > 0 {
            heapIdx -= 1;
            m = (*s).heap[heapIdx as usize];
            if m > (*desc).max_code {
                continue;
            }
            if (*(*desc).dyn_tree.add(m as usize)).dl.len != bits as u16 {
                (*s).opt_len += (bits - (*(*desc).dyn_tree.add(m as usize)).dl.len as c_ulong)
                    * (*(*desc).dyn_tree.add(m as usize)).fc.freq as c_ulong;
                (*(*desc).dyn_tree.add(m as usize)).dl.len = bits as u16;
            }
            n -= 1;
        }
        bits -= 1;
    }
}

// ===============================================================================
// Flush the bit buffer and align the output on a byte boundary
// ===============================================================================

unsafe fn bi_windup(s: *mut deflate_state_s) {
    if (*s).bi_valid > 8 {
        put_short(s, (*s).bi_buf);
    } else if (*s).bi_valid > 0 {
        put_byte(s, (*s).bi_buf as u8);
    }
    (*s).bi_buf   = 0;
    (*s).bi_valid = 0;
}

// ===============================================================================
// Reverse the first len bits of a code, using straightforward code (a faster
// method would use a table)
// ===============================================================================

unsafe fn bi_reverse(code: c_ulong, len: c_ulong) -> c_ulong {
    let mut res: c_ulong;
    let mut code = code;
    let mut len  = len;

    assert!(1 <= len);
    assert!(len <= 15);

    res = 0;
    loop {
        res |= code & 1;
        code >>= 1;
        res  <<= 1;
        len   -= 1;
        if len == 0 { break; }
    }

    res >> 1
}

// ===============================================================================
// Generate the codes for a given tree and bit counts (which need not be optimal).
// IN assertion: the array bl_count contains the bit length statistics for
// the given tree and the field len is set for all tree elements.
// OUT assertion: the field code is set for all tree elements of non zero code length.
// ===============================================================================

unsafe fn gen_codes(tree: *mut ct_data_s, max_code: c_ulong, bl_count: *mut u16) {
    let mut next_code: [u16; 15 + 1] = [0; 16]; // next code value for each bit length
    let mut code: u16;                            // running code value
    let mut bits: c_ulong;                        // bit index
    let mut codes: c_ulong;                       // code index
    let mut len: c_ulong;

    // The distribution counts are first used to generate the code values
    // without bit reversal.
    code = 0;
    bits = 1;
    while bits <= 15 /* MAX_WBITS */ {
        code = ((code as c_ulong + *bl_count.add(bits as usize - 1) as c_ulong) << 1) as u16;
        next_code[bits as usize] = code;
        bits += 1;
    }

    // Check that the bit counts in bl_count are consistent. The last code
    // must be all ones.
    codes = 0;
    while codes <= max_code {
        len = (*tree.add(codes as usize)).dl.len as c_ulong;

        if len == 0 {
            codes += 1;
            continue;
        }
        // Now reverse the bits
        (*tree.add(codes as usize)).fc.code = bi_reverse(next_code[len as usize] as c_ulong, len) as u16;
        next_code[len as usize] += 1;
        codes += 1;
    }
}

// ===============================================================================
// Construct one Huffman tree and assigns the code bit strings and lengths.
// Update the total bit length for the current block.
// IN assertion: the field freq is set for all tree elements.
// OUT assertions: the fields len and code are set to the optimal bit length
//     and corresponding code. The length opt_len is updated; static_len is
//     also updated if stree is not null. The field max_code is set.
// ===============================================================================

unsafe fn build_tree(s: *mut deflate_state_s, desc: *mut tree_desc_s) {
    let tree:  *mut ct_data_s;
    let stree: *const ct_data_s;
    let elems: c_ulong;
    let mut n: c_ulong;
    let mut m: c_ulong;        // iterate over heap elements
    let mut max_code: c_ulong; // largest code with non zero frequency
    let mut node: c_ulong;     // new node being created

    tree  = (*desc).dyn_tree;
    stree = (*(*desc).stat_desc).static_tree;
    elems = (*(*desc).stat_desc).elems;
    max_code = 0;

    // Construct the initial heap, with least frequent element in
    // heap[SMALLEST]. The sons of heap[n] are heap[2*n] and heap[2*n+1].
    // heap[0] is not used.
    (*s).heap_len = 0;
    (*s).heap_max = HEAP_SIZE as c_ulong;

    n = 0;
    while n < elems {
        if (*tree.add(n as usize)).fc.freq != 0 {
            max_code = n;
            (*s).heap_len += 1;
            (*s).heap[(*s).heap_len as usize] = n;
            (*s).depth[n as usize] = 0;
        } else {
            (*tree.add(n as usize)).dl.len = 0;
        }
        n += 1;
    }

    // The pkzip format requires that at least one distance code exists,
    // and that at least one bit should be sent even if there is only one
    // possible code. So to avoid special checks later on we force at least
    // two codes of non zero frequency.
    while (*s).heap_len < 2 {
        if max_code < 2 {
            max_code += 1;
            (*s).heap_len += 1;
            (*s).heap[(*s).heap_len as usize] = max_code;
        } else {
            (*s).heap_len += 1;
            (*s).heap[(*s).heap_len as usize] = 0;
        }
        node = (*s).heap[(*s).heap_len as usize];
        (*tree.add(node as usize)).fc.freq = 1;
        (*s).depth[node as usize] = 0;
        (*s).opt_len -= 1;
        if !stree.is_null() {
            (*s).static_len -= (*stree.add(node as usize)).dl.len as c_ulong;
        }
        // node is 0 or 1 so it does not have extra bits
    }
    (*desc).max_code = max_code;

    // The elements heap[heap_len/2+1 .. heap_len] are leaves of the tree,
    // establish sub-heaps of increasing lengths:
    n = (*s).heap_len >> 1;
    while n >= 1 {
        pqdownheap(s, tree, n);
        n -= 1;
    }

    // Construct the Huffman tree by repeatedly combining the least two
    // frequent nodes.

    // next internal node of the tree
    node = elems;
    loop {
        n = (*s).heap[SMALLEST];
        (*s).heap[SMALLEST] = (*s).heap[(*s).heap_len as usize];
        (*s).heap_len -= 1;
        pqdownheap(s, tree, SMALLEST as c_ulong);
        m = (*s).heap[SMALLEST]; // m = node of next least frequency

        (*s).heap_max -= 1;
        (*s).heap[(*s).heap_max as usize] = n; // keep the nodes sorted by frequency
        (*s).heap_max -= 1;
        (*s).heap[(*s).heap_max as usize] = m;

        // Create a new node father of n and m
        (*tree.add(node as usize)).fc.freq =
            ((*tree.add(n as usize)).fc.freq as c_ulong + (*tree.add(m as usize)).fc.freq as c_ulong) as u16;
        if (*s).depth[n as usize] > (*s).depth[m as usize] {
            (*s).depth[node as usize] = (*s).depth[n as usize];
        } else {
            (*s).depth[node as usize] = (*s).depth[m as usize];
            (*s).depth[node as usize] += 1;
        }
        (*tree.add(m as usize)).dl.dad = node as u16;
        (*tree.add(n as usize)).dl.dad = node as u16;

        // and insert the new node in the heap
        (*s).heap[SMALLEST] = node;
        node += 1;
        pqdownheap(s, tree, SMALLEST as c_ulong);

        if (*s).heap_len < 2 { break; }
    }

    (*s).heap_max -= 1;
    (*s).heap[(*s).heap_max as usize] = (*s).heap[SMALLEST];

    // At this point, the fields freq and dad are set. We can now
    // generate the bit lengths.
    gen_bitlen(s, desc);

    // The field len is now set, we can generate the bit codes
    gen_codes(tree, max_code, (*s).bl_count.as_mut_ptr());
}

// ===============================================================================
// Scan a literal or distance tree to determine the frequencies of the codes
// in the bit length tree.
// ===============================================================================

unsafe fn scan_tree(s: *mut deflate_state_s, tree: *mut ct_data_s, max_code: c_ulong) {
    let mut n: c_ulong;        // iterates over all tree elements
    let mut prevlen: c_ulong;  // last emitted length
    let mut curlen: c_ulong;   // length of current code
    let mut nextlen: c_ulong;  // length of next code
    let mut count: c_ulong;    // repeat count of the current code
    let mut max_count: c_ulong; // max repeat count
    let mut min_count: c_ulong; // min repeat count

    prevlen   = 0xffff;
    nextlen   = (*tree.add(0)).dl.len as c_ulong;
    count     = 0;
    max_count = 7;
    min_count = 4;

    if nextlen == 0 {
        max_count = 138;
        min_count = 3;
    }
    // guard
    (*tree.add(max_code as usize + 1)).dl.len = prevlen as u16;

    n = 0;
    while n <= max_code {
        curlen  = nextlen;
        nextlen = (*tree.add(n as usize + 1)).dl.len as c_ulong;
        count  += 1;
        if count < max_count && curlen == nextlen {
            n += 1;
            continue;
        } else if count < min_count {
            (*s).bl_tree[curlen as usize].fc.freq += count as u16;
        } else if curlen != 0 {
            if curlen != prevlen {
                (*s).bl_tree[curlen as usize].fc.freq += 1;
            }
            (*s).bl_tree[REP_3_6 as usize].fc.freq += 1;
        } else if count <= 10 {
            (*s).bl_tree[REPZ_3_10 as usize].fc.freq += 1;
        } else {
            (*s).bl_tree[REPZ_11_138 as usize].fc.freq += 1;
        }
        count   = 0;
        prevlen = curlen;
        if nextlen == 0 {
            max_count = 138;
            min_count = 3;
        } else if curlen == nextlen {
            max_count = 6;
            min_count = 3;
        } else {
            max_count = 7;
            min_count = 4;
        }
        n += 1;
    }
}

// ===============================================================================
// Send a literal or distance tree in compressed form, using the codes in bl_tree.
// ===============================================================================

unsafe fn send_tree(s: *mut deflate_state_s, tree: *mut ct_data_s, max_code: c_ulong) {
    let mut n: c_ulong;         // iterates over all tree elements
    let mut prevlen: c_ulong;   // last emitted length
    let mut curlen: c_ulong;    // length of current code
    let mut nextlen: c_ulong;   // length of next code
    let mut count: c_ulong;     // repeat count of the current code
    let mut max_count: c_ulong; // max repeat count
    let mut min_count: c_ulong; // min repeat count

    prevlen   = 0xffff;
    nextlen   = (*tree.add(0)).dl.len as c_ulong;
    count     = 0;
    max_count = 7;
    min_count = 4;

    if nextlen == 0 {
        max_count = 138;
        min_count = 3;
    }

    n = 0;
    while n <= max_code {
        curlen  = nextlen;
        nextlen = (*tree.add(n as usize + 1)).dl.len as c_ulong;
        count  += 1;
        if count < max_count && curlen == nextlen {
            n += 1;
            continue;
        } else if count < min_count {
            loop {
                send_bits(
                    s,
                    (*s).bl_tree[curlen as usize].fc.code as c_ulong,
                    (*s).bl_tree[curlen as usize].dl.len  as c_ulong,
                );
                count -= 1;
                if count == 0 { break; }
            }
        } else if curlen != 0 {
            if curlen != prevlen {
                send_bits(
                    s,
                    (*s).bl_tree[curlen as usize].fc.code as c_ulong,
                    (*s).bl_tree[curlen as usize].dl.len  as c_ulong,
                );
                count -= 1;
            }
            send_bits(
                s,
                (*s).bl_tree[REP_3_6 as usize].fc.code as c_ulong,
                (*s).bl_tree[REP_3_6 as usize].dl.len  as c_ulong,
            );
            send_bits(s, count - 3, 2);
        } else if count <= 10 {
            send_bits(
                s,
                (*s).bl_tree[REPZ_3_10 as usize].fc.code as c_ulong,
                (*s).bl_tree[REPZ_3_10 as usize].dl.len  as c_ulong,
            );
            send_bits(s, count - 3, 3);
        } else {
            send_bits(
                s,
                (*s).bl_tree[REPZ_11_138 as usize].fc.code as c_ulong,
                (*s).bl_tree[REPZ_11_138 as usize].dl.len  as c_ulong,
            );
            send_bits(s, count - 11, 7);
        }
        count   = 0;
        prevlen = curlen;
        if nextlen == 0 {
            max_count = 138;
            min_count = 3;
        } else if curlen == nextlen {
            max_count = 6;
            min_count = 3;
        } else {
            max_count = 7;
            min_count = 4;
        }
        n += 1;
    }
}

// ===============================================================================
// Construct the Huffman tree for the bit lengths and return the index in
// bl_order of the last bit length code to send.
// ===============================================================================

unsafe fn build_bl_tree(s: *mut deflate_state_s) -> c_ulong {
    let mut max_blindex: c_ulong; // index of last bit length code of non zero freq

    // Determine the bit length frequencies for literal and distance trees
    scan_tree(s, (*s).dyn_ltree.as_mut_ptr(), (*s).l_desc.max_code);
    scan_tree(s, (*s).dyn_dtree.as_mut_ptr(), (*s).d_desc.max_code);

    // Build the bit length tree
    build_tree(s, addr_of_mut!((*s).bl_desc));
    // opt_len now includes the length of the tree representations, except
    // the lengths of the bit lengths codes and the 5+5+4 bits for the counts.

    // Determine the number of bit length codes to send. The pkzip format
    // requires that at least 4 bit length codes be sent. (appnote.txt says
    // 3 but the actual value used is 4.)
    max_blindex = BL_CODES as c_ulong - 1;
    while max_blindex >= 3 {
        if (*s).bl_tree[bl_order[max_blindex as usize] as usize].dl.len != 0 {
            break;
        }
        max_blindex -= 1;
    }
    // Update opt_len to include the bit length tree and counts
    (*s).opt_len += 3 * (max_blindex + 1) + 5 + 5 + 4;
    max_blindex
}

// ===========================================================================
// Send the header for a block using dynamic Huffman trees: the counts, the
// lengths of the bit length codes, the literal tree and the distance tree.
// IN assertion: lcodes >= 257, dcodes >= 1, blcodes >= 4.
// ===========================================================================

unsafe fn send_all_trees(
    s: *mut deflate_state_s,
    lcodes: c_ulong,
    dcodes: c_ulong,
    blcodes: c_ulong,
) {
    let mut rank: c_ulong; // index in bl_order

    // not +255 as stated in appnote.txt
    send_bits(s, lcodes - 257, 5);
    send_bits(s, dcodes - 1, 5);
    // not -3 as stated in appnote.txt
    send_bits(s, blcodes - 4, 4);

    rank = 0;
    while rank < blcodes {
        send_bits(s, (*s).bl_tree[bl_order[rank as usize] as usize].dl.len as c_ulong, 3);
        rank += 1;
    }

    // literal tree
    send_tree(s, (*s).dyn_ltree.as_mut_ptr() as *mut ct_data_s, lcodes - 1);
    // distance tree
    send_tree(s, (*s).dyn_dtree.as_mut_ptr() as *mut ct_data_s, dcodes - 1);
}

// ===========================================================================
// Send the block data compressed using the given Huffman trees
// ===========================================================================

unsafe fn compress_block(
    s: *mut deflate_state_s,
    ltree: *const ct_data_s,
    dtree: *const ct_data_s,
) {
    let mut dist: c_ulong;    // distance of matched string
    let mut lenCount: c_ulong; // match length or unmatched char (if dist == 0)
    let mut lenIdx: c_ulong;  // running index in l_buf
    let mut code: c_ulong;    // the code to send
    let mut extra: c_ulong;   // number of extra bits to send

    lenIdx = 0;
    if (*s).last_lit != 0 {
        loop {
            dist     = (*s).d_buf[lenIdx as usize] as c_ulong;
            lenCount = (*s).l_buf[lenIdx as usize] as c_ulong;
            lenIdx  += 1;
            if dist == 0 {
                // send a literal byte
                send_bits(
                    s,
                    (*ltree.add(lenCount as usize)).fc.code as c_ulong,
                    (*ltree.add(lenCount as usize)).dl.len  as c_ulong,
                );
            } else {
                // Here, lenCount is the match length - MIN_MATCH
                code = tr_length_code[lenCount as usize] as c_ulong;
                // send the length code
                send_bits(
                    s,
                    (*ltree.add(code as usize + LITERALS + 1)).fc.code as c_ulong,
                    (*ltree.add(code as usize + LITERALS + 1)).dl.len  as c_ulong,
                );
                extra = extra_lbits[code as usize];
                if extra != 0 {
                    lenCount -= base_length[code as usize] as c_ulong;
                    // send the extra length bits
                    send_bits(s, lenCount, extra);
                }
                // dist is now the match distance - 1
                dist -= 1;
                code = if dist < 256 {
                    tr_dist_code[dist as usize] as c_ulong
                } else {
                    tr_dist_code[256 + (dist >> 7) as usize] as c_ulong
                };

                // send the distance code
                send_bits(
                    s,
                    (*dtree.add(code as usize)).fc.code as c_ulong,
                    (*dtree.add(code as usize)).dl.len  as c_ulong,
                );
                extra = extra_dbits[code as usize];
                if extra != 0 {
                    dist -= base_dist[code as usize] as c_ulong;
                    // send the extra distance bits
                    send_bits(s, dist, extra);
                }
            }
            if lenIdx >= (*s).last_lit { break; }
        }
    }

    send_bits(
        s,
        (*ltree.add(END_BLOCK as usize)).fc.code as c_ulong,
        (*ltree.add(END_BLOCK as usize)).dl.len  as c_ulong,
    );
    (*s).last_eob_len = (*ltree.add(END_BLOCK as usize)).dl.len as c_ulong;
}

// ===========================================================================
// Send a stored block
// ===========================================================================

unsafe fn tr_stored_block(
    s: *mut deflate_state_s,
    buf: *const u8,
    stored_len: c_ulong,
    eof: bool,
) {
    // send block type
    send_bits(s, (STORED_BLOCK as c_ulong) << 1 | eof as c_ulong, 3);

    // align on byte boundary
    bi_windup(s);
    // enough lookahead for inflate
    (*s).last_eob_len = 8;

    put_short(s, stored_len as u16);
    put_short(s, !stored_len as u16);

    let mut remaining = stored_len;
    let mut p = buf;
    while remaining > 0 {
        put_byte(s, *p);
        p = p.add(1);
        remaining -= 1;
    }
}

// ===========================================================================
// Determine the best encoding for the current block: dynamic trees, static
// trees or store, and output the encoded block to the zip file.
// ===========================================================================

unsafe fn tr_flush_block(
    s: *mut deflate_state_s,
    buf: *const u8,
    stored_len: c_ulong,
    eof: bool,
) {
    let mut opt_lenb: c_ulong;
    let mut static_lenb: c_ulong;
    let mut max_blindex: c_ulong; // index of last bit length code of non zero freq

    max_blindex = 0;

    // Build the Huffman trees unless a stored block is forced
    if (*s).level as c_int > 0 {
        // Construct the literal and distance trees
        build_tree(s, addr_of_mut!((*s).l_desc));
        build_tree(s, addr_of_mut!((*s).d_desc));
        // At this point, opt_len and static_len are the total bit lengths of
        // the compressed block data, excluding the tree representations.

        // Build the bit length tree for the above two trees, and get the index
        // in bl_order of the last bit length code to send.
        max_blindex = build_bl_tree(s);

        // Determine the best encoding. Compute first the block length in bytes
        opt_lenb    = ((*s).opt_len + 3 + 7) >> 3;
        static_lenb = ((*s).static_len + 3 + 7) >> 3;

        if static_lenb <= opt_lenb {
            opt_lenb = static_lenb;
        }
    } else {
        static_lenb = stored_len + 5;
        // force a stored block
        opt_lenb    = static_lenb;
    }

    if (stored_len + 4 <= opt_lenb) && !buf.is_null() {
        // 4: two words for the lengths
        // The test buf != NULL is only necessary if LIT_BUFSIZE > WSIZE.
        // Otherwise we can't have processed more than WSIZE input bytes since
        // the last block flush, because compression would have been
        // successful. If LIT_BUFSIZE <= WSIZE, it is never too late to
        // transform a block into a stored block.
        tr_stored_block(s, buf, stored_len, eof);
    } else if static_lenb == opt_lenb {
        send_bits(s, (STATIC_TREES as c_ulong) << 1 | eof as c_ulong, 3);
        compress_block(s, static_ltree.as_ptr(), static_dtree.as_ptr());
    } else {
        send_bits(s, (DYN_TREES as c_ulong) << 1 | eof as c_ulong, 3);
        send_all_trees(
            s,
            (*s).l_desc.max_code + 1,
            (*s).d_desc.max_code + 1,
            max_blindex + 1,
        );
        compress_block(s, (*s).dyn_ltree.as_ptr(), (*s).dyn_dtree.as_ptr());
    }

    // The above check is made mod 2^32, for files larger than 512 MB
    // and uLong implemented on 32 bits.
    init_block(s);

    if eof {
        bi_windup(s);
    }
}

// ===============================================================================
// ===============================================================================

#[inline]
unsafe fn tr_tally_lit(s: *mut deflate_state_s, c: u8) -> bool {
    (*s).d_buf[(*s).last_lit as usize] = 0;
    (*s).l_buf[(*s).last_lit as usize] = c;
    (*s).last_lit += 1;
    (*s).dyn_ltree[c as usize].fc.freq += 1;

    (*s).last_lit == LIT_BUFSIZE as c_ulong - 1
}

// ===============================================================================
// ===============================================================================

#[inline]
unsafe fn tr_tally_dist(s: *mut deflate_state_s, dist: c_ulong, len: c_ulong) -> bool {
    assert!(dist < 65536);
    assert!(len < 256);

    let dist = dist & 0xffff;
    let len  = len  & 0xff;

    (*s).d_buf[(*s).last_lit as usize] = dist as u16;
    (*s).l_buf[(*s).last_lit as usize] = len as u8;
    (*s).last_lit += 1;
    let dist_m1 = dist - 1;
    (*s).dyn_ltree[(tr_length_code[len as usize] as usize) + LITERALS + 1].fc.freq += 1;
    (*s).dyn_dtree[
        if dist_m1 < 256 {
            tr_dist_code[dist_m1 as usize] as usize
        } else {
            tr_dist_code[256 + (dist_m1 >> 7) as usize] as usize
        }
    ].fc.freq += 1;

    (*s).last_lit == LIT_BUFSIZE as c_ulong - 1
}

// ===============================================================================
// Insert string str in the dictionary and set match_head to the previous head
// of the hash chain (the most recent string with same hash key). Return
// the previous length of the hash chain.
// If this file is compiled with -DFASTEST, the compression level is forced
// to 1, and no hash chains are maintained.
// IN  assertion: all calls to to INSERT_STRING are made with consecutive
//    input characters and the first MIN_MATCH bytes of str are valid
//    (except for the last MIN_MATCH-1 bytes of the input file).
// ===============================================================================

#[inline]
unsafe fn insert_string(s: *mut deflate_state_s, str_: c_ulong, match_head: *mut c_ulong) {
    (*s).ins_h = ((((*s).ins_h << HASH_SHIFT as c_ulong)
        ^ (*s).window[(str_ as usize) + (MIN_MATCH as usize - 1)] as c_ulong)
        & HASH_MASK as c_ulong);
    *match_head = (*s).head[(*s).ins_h as usize] as c_ulong;
    (*s).prev[(str_ as usize) & WINDOW_MASK] = (*s).head[(*s).ins_h as usize];
    (*s).head[(*s).ins_h as usize] = str_ as u16;
}

// ===========================================================================
// Initialize the "longest match" routines for a new zlib stream
// ===========================================================================

unsafe fn lm_init(s: *mut deflate_state_s) {
    (*s).head[HASH_SIZE - 1] = 0; // NULL
    core::ptr::write_bytes((*s).head.as_mut_ptr(), 0, HASH_SIZE - 1);

    // Set the default configuration parameters:
    (*s).max_lazy_match   = configuration_table[(*s).level as usize].max_lazy as c_ulong;
    (*s).good_match       = configuration_table[(*s).level as usize].good_length as c_ulong;
    (*s).nice_match       = configuration_table[(*s).level as usize].nice_length as c_ulong;
    (*s).max_chain_length = configuration_table[(*s).level as usize].max_chain as c_ulong;

    (*s).strstart        = 0;
    (*s).block_start     = 0;
    (*s).lookahead       = 0;
    (*s).prev_length     = MIN_MATCH as c_ulong - 1;
    (*s).match_length    = MIN_MATCH as c_ulong - 1;
    (*s).match_available = 0;
    (*s).ins_h           = 0;
}

// ===========================================================================
// Set match_start to the longest match starting at the given string and
// return its length. Matches shorter or equal to prev_length are discarded,
// in which case the result is equal to prev_length and match_start is
// garbage.
// IN assertions: cur_match is the head of the hash chain for the current
//  string (strstart) and its distance is <= MAX_DIST, and prev_length >= 1
// OUT assertion: the match length is not greater than s->lookahead.
// ===========================================================================

// Porting note: qcmp uses x86-32 MSVC inline assembly (repe cmpsb).
// Translated to Rust core::arch::asm! using x86-32 register constraints.
// This function is only correct when compiled for x86 (32-bit).
// The MSVC _asm { push/pop esi/edi/ecx } are replaced with Rust's
// register constraint mechanism which handles save/restore implicitly.
// Parameter `match` renamed to `match_` (Rust reserved keyword).
#[inline]
unsafe fn qcmp(scan: *mut u8, match_: *mut u8, count: c_ulong) -> *mut u8 {
    let out_scan: *mut u8;
    // x86-32: repe cmpsb compares bytes at esi and edi for ecx iterations
    // until a mismatch; esi ends one past the last compared byte.
    core::arch::asm!(
        "repe cmpsb",
        inout("esi") scan     => out_scan,
        in("edi")    match_,
        inout("ecx") count    => _,
        options(nostack, preserves_flags),
    );
    out_scan.sub(1)
}

unsafe fn longest_match(s: *mut deflate_state_s, cur_match: c_ulong) -> c_ulong {
    let mut chain_length: c_ulong; // max hash chain length
    let mut limit: c_ulong;
    let mut scan: *mut u8;         // current string
    let mut match_: *mut u8;       // matched string
    let mut len: c_ulong;          // length of current match
    let mut best_len: c_ulong;     // best match length so far
    let mut nice_match: c_ulong;   // stop if match long enough
    let scan_end1: u8;
    let scan_end: u8;
    let mut cur_match = cur_match;

    chain_length = (*s).max_chain_length;
    scan         = (*s).window.as_mut_ptr().add((*s).strstart as usize);
    best_len     = (*s).prev_length;
    nice_match   = (*s).nice_match;

    // Stop when cur_match becomes <= limit. To simplify the code,
    // we prevent matches with the string of window index 0.
    limit = if (*s).strstart > (WINDOW_SIZE as c_ulong - MIN_LOOKAHEAD as c_ulong) {
        (*s).strstart - (WINDOW_SIZE as c_ulong - MIN_LOOKAHEAD as c_ulong)
    } else {
        0 // NULL
    };

    let scan_end1_ref = scan.add(best_len as usize - 1);
    let scan_end_ref  = scan.add(best_len as usize);
    let mut scan_end1 = *scan_end1_ref;
    let mut scan_end  = *scan_end_ref;

    // Do not waste too much time if we already have a good match:
    if (*s).prev_length >= (*s).good_match {
        chain_length >>= 2;
    }
    // Do not look for matches beyond the end of the input. This is necessary
    // to make deflate deterministic.
    if nice_match > (*s).lookahead {
        nice_match = (*s).lookahead;
    }
    loop {
        match_ = (*s).window.as_mut_ptr().add(cur_match as usize);

        // Skip to next match if the match length cannot increase
        // or if the match length is less than 2:
        if *match_.add(best_len as usize)     != scan_end
            || *match_.add(best_len as usize - 1) != scan_end1
            || *match_                            != *scan
            || *match_.add(1)                    != *scan.add(1)
        {
            cur_match = (*s).prev[cur_match as usize & WINDOW_MASK] as c_ulong;
            chain_length -= 1;
            if cur_match <= limit || chain_length == 0 { break; }
            continue;
        }

        // The check at best_len-1 can be removed because it will be made
        // again later. (This heuristic is not always a win.)
        // It is not necessary to compare scan[2] and match[2] since they
        // are always equal when the other bytes match, given that
        // the hash keys are equal
        scan = qcmp(scan.add(3), match_.add(3), MAX_MATCH as c_ulong - 2);

        len  = scan.offset_from((*s).window.as_ptr().add((*s).strstart as usize)) as c_ulong;
        scan = (*s).window.as_mut_ptr().add((*s).strstart as usize);

        if len > best_len {
            (*s).match_start = cur_match;
            best_len = len;
            if len >= nice_match {
                break;
            }
            scan_end1 = *scan.add(best_len as usize - 1);
            scan_end  = *scan.add(best_len as usize);
        }
        cur_match = (*s).prev[cur_match as usize & WINDOW_MASK] as c_ulong;
        chain_length -= 1;
        if cur_match <= limit || chain_length == 0 { break; }
    }

    if best_len <= (*s).lookahead {
        return best_len;
    }
    (*s).lookahead
}

// ===========================================================================
// Flush as much pending output as possible. All deflate() output goes
// through this function so some applications may wish to modify it
// to avoid allocating a large z->next_out buffer and copying into it.
// (See also read_buf()).
// ===========================================================================

unsafe fn flush_pending(z: *mut z_stream) {
    let mut len: c_ulong = (*(*z).dstate).pending;

    if len > (*z).avail_out {
        len = (*z).avail_out;
    }
    if len == 0 {
        return;
    }
    assert!(len <= MAX_BLOCK_SIZE as c_ulong + 5);
    assert!(
        (*(*z).dstate)
            .pending_out
            .add(len as usize)
            <= (*(*z).dstate).pending_buf.as_ptr().add(MAX_BLOCK_SIZE + 5)
    );

    core::ptr::copy_nonoverlapping((*(*z).dstate).pending_out, (*z).next_out, len as usize);
    (*z).next_out             = (*z).next_out.add(len as usize);
    (*z).total_out           += len;
    (*(*z).dstate).pending_out = (*(*z).dstate).pending_out.add(len as usize);
    (*z).avail_out            -= len;
    (*(*z).dstate).pending    -= len;
    if (*(*z).dstate).pending == 0 {
        (*(*z).dstate).pending_out = (*(*z).dstate).pending_buf.as_mut_ptr();
    }
}

// ===========================================================================
// Read a new buffer from the current input stream, update the adler32
// and total number of bytes read.  All deflate() input goes through
// this function so some applications may wish to modify it to avoid
// allocating a large z->next_in buffer and copying from it.
// (See also flush_pending()).
// ===========================================================================

unsafe fn read_buf(z: *mut z_stream, buf: *mut u8, size: c_ulong) -> c_ulong {
    let mut len: c_ulong;

    len = (*z).avail_in;
    if len > size {
        len = size;
    }
    if len == 0 {
        return 0;
    }
    (*z).avail_in -= len;

    if (*(*z).dstate).noheader == 0 {
        (*(*z).dstate).adler = adler32((*(*z).dstate).adler, (*z).next_in, len);
    }
    core::ptr::copy_nonoverlapping((*z).next_in, buf, len as usize);
    (*z).next_in = (*z).next_in.add(len as usize);
    len
}

// ===========================================================================
// Fill the window when the lookahead becomes insufficient.
// Updates strstart and lookahead.
//
// IN assertion: lookahead < MIN_LOOKAHEAD
// OUT assertions: strstart <= BIG_WINDOW_SIZE - MIN_LOOKAHEAD
//    At least one byte has been read, or avail_in == 0; reads are
//    performed for at least two bytes (required for the zip translate_eol
//    option -- not supported here).
// ===========================================================================

unsafe fn fill_window(s: *mut deflate_state_s) {
    let mut n: c_ulong;
    let mut m: c_ulong;
    let mut p: *mut u16;
    let mut more: c_ulong; // Amount of free space at the end of the window.

    loop {
        more = BIG_WINDOW_SIZE as c_ulong - (*s).lookahead - (*s).strstart;

        if (*s).strstart >= (WINDOW_SIZE + (WINDOW_SIZE - MIN_LOOKAHEAD)) as c_ulong {
            core::ptr::copy_nonoverlapping(
                (*s).window.as_ptr().add(WINDOW_SIZE),
                (*s).window.as_mut_ptr(),
                WINDOW_SIZE,
            );
            (*s).match_start = (*s).match_start.wrapping_sub(WINDOW_SIZE as c_ulong);
            // Make strstart >= MAX_DIST
            (*s).strstart    -= WINDOW_SIZE as c_ulong;
            (*s).block_start  = (*s).block_start.wrapping_sub(WINDOW_SIZE as c_int);

            // Slide the hash table (could be avoided with 32 bit values
            // at the expense of memory usage). We slide even when level == 0
            // to keep the hash table consistent if we switch back to level > 0
            // later. (Using level 0 permanently is not an optimal usage of
            // zlib, so we don't care about this pathological case.)
            n = HASH_SIZE as c_ulong;
            p = (*s).head.as_mut_ptr().add(HASH_SIZE);
            loop {
                p  = p.sub(1);
                m  = *p as c_ulong;
                *p = if m >= WINDOW_SIZE as c_ulong { (m - WINDOW_SIZE as c_ulong) as u16 } else { 0 };
                n -= 1;
                if n == 0 { break; }
            }

            n = WINDOW_SIZE as c_ulong;
            p = (*s).prev.as_mut_ptr().add(WINDOW_SIZE);
            loop {
                p  = p.sub(1);
                m  = *p as c_ulong;
                *p = if m >= WINDOW_SIZE as c_ulong { (m - WINDOW_SIZE as c_ulong) as u16 } else { 0 };
                // If n is not on any hash chain, prev[n] is garbage but
                // its value will never be used.
                n -= 1;
                if n == 0 { break; }
            }

            more += WINDOW_SIZE as c_ulong;
        }
        if (*(*s).z).avail_in == 0 {
            return;
        }

        // If there was no sliding:
        //    strstart <= WSIZE+MAX_DIST-1 && lookahead <= MIN_LOOKAHEAD - 1 &&
        //    more == BIG_WINDOW_SIZE - lookahead - strstart
        // => more >= BIG_WINDOW_SIZE - (MIN_LOOKAHEAD-1 + WSIZE + MAX_DIST-1)
        // => more >= BIG_WINDOW_SIZE- 2*WSIZE + 2
        // If there was sliding, more >= WSIZE. So in all cases, more >= 2.
        n = read_buf(
            (*s).z,
            (*s).window.as_mut_ptr().add((*s).strstart as usize + (*s).lookahead as usize),
            more,
        );
        (*s).lookahead += n;

        // Initialize the hash value now that we have some input:
        if (*s).lookahead >= MIN_MATCH as c_ulong {
            (*s).ins_h = ((((*s).window[(*s).strstart as usize] as c_ulong) << HASH_SHIFT as c_ulong)
                ^ ((*s).window[(*s).strstart as usize + 1] as c_ulong))
                & HASH_MASK as c_ulong;
        }
        // If the whole input has less than MIN_MATCH bytes, ins_h is garbage,
        // but this is not important since only literal bytes will be emitted.

        if (*s).lookahead >= MIN_LOOKAHEAD as c_ulong || (*(*s).z).avail_in == 0 {
            break;
        }
    }
}

// ===========================================================================
// Flush the current block, with given end-of-file flag.
// IN assertion: strstart is set to the end of the current match.
// ===========================================================================

#[inline]
unsafe fn flush_block_only(s: *mut deflate_state_s, eof: bool) {
    if (*s).block_start >= 0 {
        tr_flush_block(
            s,
            (*s).window.as_ptr().add((*s).block_start as usize),
            (*s).strstart.wrapping_sub((*s).block_start as c_ulong),
            eof,
        );
    } else {
        tr_flush_block(
            s,
            null(),
            (*s).strstart.wrapping_sub((*s).block_start as c_ulong),
            eof,
        );
    }
    (*s).block_start = (*s).strstart as c_int;
    flush_pending((*s).z);
}

// ===========================================================================
// Copy without compression as much as possible from the input stream, return
// the current block state.
// This function does not insert new strings in the dictionary since
// uncompressible data is probably not useful. This function is used
// only for the level=0 compression option.
// NOTE: this function should be optimized to avoid extra copying from
// window to pending_buf.
// ===========================================================================

unsafe extern "C" fn deflate_stored(s: *mut deflate_state_s, flush: EFlush) -> block_state {
    // Stored blocks are limited to 0xffff bytes, pending_buf is limited
    // to pending_buf_size, and each stored block has a 5 byte header:
    let mut max_start: c_ulong;

    // Copy as much as possible from input to output:
    loop {
        // Fill the window as much as possible
        if (*s).lookahead <= 1 {
            fill_window(s);
            if (*s).lookahead == 0 && (flush as c_int == EFlush::Z_NO_FLUSH as c_int) {
                return block_state::NEED_MORE;
            }
            if (*s).lookahead == 0 {
                // flush the current block
                break;
            }
        }
        (*s).strstart += (*s).lookahead;
        (*s).lookahead  = 0;

        // Emit a stored block if pending_buf will be full
        max_start = (*s).block_start as c_ulong + MAX_BLOCK_SIZE as c_ulong;
        if (*s).strstart == 0 || (*s).strstart >= max_start {
            // strstart == 0 is possible when wraparound on 16-bit machine
            (*s).lookahead = (*s).strstart.wrapping_sub(max_start);
            (*s).strstart  = max_start;
            flush_block_only(s, false);
            if (*(*s).z).avail_out == 0 {
                return block_state::NEED_MORE;
            }
        }
        // Flush if we may have to slide, otherwise block_start may become
        // negative and the data will be gone:
        if (*s).strstart.wrapping_sub((*s).block_start as c_ulong)
            >= (WINDOW_SIZE as c_ulong - MIN_LOOKAHEAD as c_ulong)
        {
            flush_block_only(s, false);
            if (*(*s).z).avail_out == 0 {
                return block_state::NEED_MORE;
            }
        }
    }
    flush_block_only(s, flush as c_int == EFlush::Z_FINISH as c_int);
    if (*(*s).z).avail_out == 0 {
        return if flush as c_int == EFlush::Z_FINISH as c_int {
            block_state::FINISH_STARTED
        } else {
            block_state::NEED_MORE
        };
    }
    if flush as c_int == EFlush::Z_FINISH as c_int {
        block_state::FINISH_DONE
    } else {
        block_state::BLOCK_DONE
    }
}

// ===========================================================================
// Compress as much as possible from the input stream, return the current block state.
// This function does not perform lazy evaluation of matches and inserts
// new strings in the dictionary only for unmatched strings or for short
// matches. It is used only for the fast compression options.
// ===========================================================================

unsafe extern "C" fn deflate_fast(s: *mut deflate_state_s, flush: EFlush) -> block_state {
    let mut hash_head: c_ulong; // head of the hash chain
    let mut bflush: bool;       // set if current block must be flushed

    hash_head = 0;
    loop {
        // Make sure that we always have enough lookahead, except
        // at the end of the input file. We need MAX_MATCH bytes
        // for the next match, plus MIN_MATCH bytes to insert the
        // string following the next match.
        if (*s).lookahead < MIN_LOOKAHEAD as c_ulong {
            fill_window(s);
            if (*s).lookahead < MIN_LOOKAHEAD as c_ulong
                && flush as c_int == EFlush::Z_NO_FLUSH as c_int
            {
                return block_state::NEED_MORE;
            }
            if (*s).lookahead == 0 {
                // flush the current block
                break;
            }
        }

        // Insert the string window[strstart .. strstart+2] in the
        // dictionary, and set hash_head to the head of the hash chain:
        if (*s).lookahead >= MIN_MATCH as c_ulong {
            insert_string(s, (*s).strstart, &mut hash_head);
        }

        // Find the longest match, discarding those <= prev_length.
        // At this point we have always match_length < MIN_MATCH
        if hash_head != 0
            && (*s).strstart.wrapping_sub(hash_head) <= WINDOW_SIZE as c_ulong - MIN_LOOKAHEAD as c_ulong
        {
            // To simplify the code, we prevent matches with the string
            // of window index 0 (in particular we have to avoid a match
            // of the string with itself at the start of the input file).
            (*s).match_length = longest_match(s, hash_head);
            // longest_match() sets match_start
        }
        if (*s).match_length >= MIN_MATCH as c_ulong {
            (*(*s).z).quality += 1;

            bflush = tr_tally_dist(
                s,
                (*s).strstart - (*s).match_start,
                (*s).match_length - MIN_MATCH as c_ulong,
            );
            (*s).lookahead -= (*s).match_length;

            // Insert new strings in the hash table only if the match length
            // is not too large. This saves time but degrades compression.
            if (*s).match_length <= (*s).max_lazy_match
                && (*s).lookahead >= MIN_MATCH as c_ulong
            {
                // string at strstart already in hash table
                (*s).match_length -= 1;
                loop {
                    // strstart never exceeds WSIZE-MAX_MATCH, so there are
                    // always MIN_MATCH bytes ahead.
                    (*s).strstart += 1;
                    insert_string(s, (*s).strstart, &mut hash_head);
                    (*s).match_length -= 1;
                    if (*s).match_length == 0 { break; }
                }
                (*s).strstart += 1;
            } else {
                (*(*s).z).quality += 1;

                (*s).strstart     += (*s).match_length;
                (*s).match_length  = 0;
                (*s).ins_h = ((((*s).window[(*s).strstart as usize] as c_ulong) << HASH_SHIFT as c_ulong)
                    ^ ((*s).window[(*s).strstart as usize + 1] as c_ulong))
                    & HASH_MASK as c_ulong;
                // If lookahead < MIN_MATCH, ins_h is garbage, but it does not
                // matter since it will be recomputed at next deflate call.
            }
        } else {
            // No match, output a literal byte
            bflush = tr_tally_lit(s, (*s).window[(*s).strstart as usize]);
            (*s).lookahead -= 1;
            (*s).strstart  += 1;
        }
        if bflush {
            flush_block_only(s, false);
            if (*(*s).z).avail_out == 0 {
                return block_state::NEED_MORE;
            }
        }
    }
    flush_block_only(s, flush as c_int == EFlush::Z_FINISH as c_int);
    if (*(*s).z).avail_out == 0 {
        return if flush as c_int == EFlush::Z_FINISH as c_int {
            block_state::FINISH_STARTED
        } else {
            block_state::NEED_MORE
        };
    }
    if flush as c_int == EFlush::Z_FINISH as c_int {
        block_state::FINISH_DONE
    } else {
        block_state::BLOCK_DONE
    }
}

// ===========================================================================
// Same as above, but achieves better compression. We use a lazy
// evaluation for matches: a match is finally adopted only if there is
// no better match at the next window position.
// ===========================================================================

unsafe extern "C" fn deflate_slow(s: *mut deflate_state_s, flush: EFlush) -> block_state {
    let mut hash_head: c_ulong; // head of hash chain
    let mut max_insert: c_ulong;
    let mut bflush: bool;       // set if current block must be flushed

    hash_head = 0;
    // Process the input block.
    loop {
        // Make sure that we always have enough lookahead, except
        // at the end of the input file. We need MAX_MATCH bytes
        // for the next match, plus MIN_MATCH bytes to insert the
        // string following the next match.
        if (*s).lookahead < MIN_LOOKAHEAD as c_ulong {
            fill_window(s);
            if (*s).lookahead < MIN_LOOKAHEAD as c_ulong
                && flush as c_int == EFlush::Z_NO_FLUSH as c_int
            {
                return block_state::NEED_MORE;
            }
            if (*s).lookahead == 0 {
                // flush the current block
                break;
            }
        }

        // Insert the string window[strstart .. strstart+2] in the
        // dictionary, and set hash_head to the head of the hash chain:
        if (*s).lookahead >= MIN_MATCH as c_ulong {
            insert_string(s, (*s).strstart, &mut hash_head);
        }

        // Find the longest match, discarding those <= prev_length.
        (*s).prev_length  = (*s).match_length;
        (*s).prev_match   = (*s).match_start;
        (*s).match_length = MIN_MATCH as c_ulong - 1;

        if hash_head != 0
            && (*s).prev_length < (*s).max_lazy_match
            && (*s).strstart.wrapping_sub(hash_head) <= WINDOW_SIZE as c_ulong - MIN_LOOKAHEAD as c_ulong
        {
            // To simplify the code, we prevent matches with the string
            // of window index 0 (in particular we have to avoid a match
            // of the string with itself at the start of the input file).
            // longest_match() sets match_start
            (*s).match_length = longest_match(s, hash_head);

            if (*s).match_length <= 5
                && ((*s).match_length == MIN_MATCH as c_ulong
                    && (*s).strstart.wrapping_sub((*s).match_start) > TOO_FAR as c_ulong)
            {
                // If prev_match is also MIN_MATCH, match_start is garbage
                // but we will ignore the current match anyway.
                (*s).match_length = MIN_MATCH as c_ulong - 1;
            }
        }
        // If there was a match at the previous step and the current
        // match is not better, output the previous match:
        if (*s).prev_length >= MIN_MATCH as c_ulong && (*s).match_length <= (*s).prev_length {
            // Do not insert strings in hash table beyond this.
            max_insert = (*s).strstart + (*s).lookahead - MIN_MATCH as c_ulong;

            bflush = tr_tally_dist(
                s,
                (*s).strstart - 1 - (*s).prev_match,
                (*s).prev_length - MIN_MATCH as c_ulong,
            );

            // Insert in hash table all strings up to the end of the match.
            // strstart-1 and strstart are already inserted. If there is not
            // enough lookahead, the last two strings are not inserted in
            // the hash table.
            (*s).lookahead   -= (*s).prev_length - 1;
            (*s).prev_length -= 2;
            loop {
                (*s).strstart += 1;
                if (*s).strstart <= max_insert {
                    insert_string(s, (*s).strstart, &mut hash_head);
                }
                (*s).prev_length -= 1;
                if (*s).prev_length == 0 { break; }
            }

            (*s).match_available = 0;
            (*s).match_length    = MIN_MATCH as c_ulong - 1;
            (*s).strstart       += 1;

            if bflush {
                flush_block_only(s, false);
                if (*(*s).z).avail_out == 0 {
                    return block_state::NEED_MORE;
                }
            }
        } else if (*s).match_available != 0 {
            // If there was no match at the previous position, output a
            // single literal. If there was a match but the current match
            // is longer, truncate the previous match to a single literal.
            bflush = tr_tally_lit(s, (*s).window[(*s).strstart as usize - 1]);
            if bflush {
                flush_block_only(s, false);
            }
            (*s).strstart  += 1;
            (*s).lookahead -= 1;
            if (*(*s).z).avail_out == 0 {
                return block_state::NEED_MORE;
            }
        } else {
            // There is no previous match to compare with, wait for
            // the next step to decide.
            (*s).match_available = 1;
            (*s).strstart       += 1;
            (*s).lookahead      -= 1;
        }
    }
    if (*s).match_available != 0 {
        let _ = tr_tally_lit(s, (*s).window[(*s).strstart as usize - 1]);
        (*s).match_available = 0;
    }
    flush_block_only(s, flush as c_int == EFlush::Z_FINISH as c_int);
    if (*(*s).z).avail_out == 0 {
        return if flush as c_int == EFlush::Z_FINISH as c_int {
            block_state::FINISH_STARTED
        } else {
            block_state::NEED_MORE
        };
    }
    if flush as c_int == EFlush::Z_FINISH as c_int {
        block_state::FINISH_DONE
    } else {
        block_state::BLOCK_DONE
    }
}

// -------------------------------------------------------------------------------------------------
// Controlling routines
// -------------------------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn deflateInit(z: *mut z_stream, level: ELevel, noWrap: c_int) -> EStatus {
    let s: *mut deflate_state_s;

    assert!(!z.is_null());

    deflate_error = b"OK\0".as_ptr() as *const c_char;
    if (level as c_int) < ELevel::Z_STORE_COMPRESSION as c_int
        || (level as c_int) > ELevel::Z_MAX_COMPRESSION as c_int
    {
        deflate_error = b"Invalid compression level\0".as_ptr() as *const c_char;
        return EStatus::Z_STREAM_ERROR;
    }
    s = Z_Malloc(
        core::mem::size_of::<deflate_state_s>() as c_int,
        TAG_DEFLATE,
        qtrue,
    ) as *mut deflate_state_s;
    (*z).dstate = s as *mut _;
    (*s).z = z;

    // undocumented feature: suppress zlib header
    (*s).noheader = noWrap;
    (*s).level    = level;

    (*z).total_out = 0;
    (*z).quality   = 0;

    (*s).pending     = 0;
    (*s).pending_out = (*s).pending_buf.as_mut_ptr();

    (*s).status      = if (*s).noheader != 0 { BUSY_STATE } else { INIT_STATE };
    (*s).adler       = 1;
    (*s).last_flush  = EFlush::Z_NO_FLUSH;

    tr_init(s);
    lm_init(s);
    EStatus::Z_OK
}

// ===========================================================================
// Copy the source state to the destination state.
// To simplify the source, this is not supported for 16-bit MSDOS (which
// doesn't have enough memory anyway to duplicate compression states).
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn deflateCopy(dest: *mut z_stream, source: *mut z_stream) -> EStatus {
    let ds: *mut deflate_state_s;
    let ss: *mut deflate_state_s;

    assert!(!source.is_null());
    assert!(!dest.is_null());
    assert!(!(*source).dstate.is_null());
    assert!((*dest).dstate.is_null());

    *dest = *source;

    ss = (*source).dstate as *mut deflate_state_s;
    ds = Z_Malloc(
        core::mem::size_of::<deflate_state_s>() as c_int,
        TAG_DEFLATE,
        qtrue,
    ) as *mut deflate_state_s;
    (*dest).dstate = ds as *mut _;
    *ds = *ss;
    (*ds).z = dest;

    (*ds).pending_out = (*ds).pending_buf.as_mut_ptr()
        .add((*ss).pending_out.offset_from((*ss).pending_buf.as_ptr()) as usize);

    (*ds).l_desc.dyn_tree  = (*ds).dyn_ltree.as_mut_ptr();
    (*ds).d_desc.dyn_tree  = (*ds).dyn_dtree.as_mut_ptr();
    (*ds).bl_desc.dyn_tree = (*ds).bl_tree.as_mut_ptr();

    EStatus::Z_OK
}

// ===========================================================================
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn deflate(z: *mut z_stream, flush: EFlush) -> EStatus {
    let old_flush: EFlush; // value of flush param for previous deflate call
    let s: *mut deflate_state_s;
    let mut header: c_ulong;
    let mut level_flags: c_ulong;

    assert!(!z.is_null());
    assert!(!(*z).dstate.is_null());

    if (flush as c_int > EFlush::Z_FINISH as c_int)
        || ((flush as c_int) < EFlush::Z_NO_FLUSH as c_int)
    {
        deflate_error = b"Invalid flush type\0".as_ptr() as *const c_char;
        return EStatus::Z_STREAM_ERROR;
    }
    s = (*z).dstate as *mut deflate_state_s;

    if (*z).next_out.is_null()
        || ((*z).next_in.is_null() && (*z).avail_in != 0)
        || ((*s).status == FINISH_STATE && flush as c_int != EFlush::Z_FINISH as c_int)
    {
        deflate_error = b"Invalid output data\0".as_ptr() as *const c_char;
        return EStatus::Z_STREAM_ERROR;
    }
    if (*z).avail_out == 0 {
        deflate_error = b"No output space\0".as_ptr() as *const c_char;
        return EStatus::Z_BUF_ERROR;
    }

    old_flush        = (*s).last_flush;
    (*s).last_flush  = flush;

    // Write the zlib header
    if (*s).status == INIT_STATE {
        header = ((ZF_DEFLATED as c_ulong + (((15 /* MAX_WBITS */ - 8) as c_ulong) << 4)) << 8) as c_ulong;
        level_flags = (((*s).level as c_int - 1) >> 1) as c_ulong;

        if level_flags > 3 {
            level_flags = 3;
        }
        header |= level_flags << 6;

        header += 31 - (header % 31);

        (*s).status = BUSY_STATE;
        put_shortMSB(s, header as u16);
        (*s).adler = 1;
    }

    // Flush as much pending output as possible
    if (*s).pending != 0 {
        flush_pending(z);
        if (*z).avail_out == 0 {
            // Since avail_out is 0, deflate will be called again with
            // more output space, but possibly with both pending and
            // avail_in equal to zero. There won't be anything to do,
            // but this is not an error situation so make sure we
            // return OK instead of BUF_ERROR at next call of deflate:
            (*s).last_flush = EFlush::Z_NEED_MORE;
            return EStatus::Z_OK;
        }

        // Make sure there is something to do and avoid duplicate consecutive
        // flushes. For repeated and useless calls with Z_FINISH, we keep
        // returning Z_STREAM_END instead of Z_BUFF_ERROR.
    } else if (*z).avail_in == 0
        && flush as c_int <= old_flush as c_int
        && flush as c_int != EFlush::Z_FINISH as c_int
    {
        deflate_error = b"No available input\0".as_ptr() as *const c_char;
        return EStatus::Z_BUF_ERROR;
    }

    // User must not provide more input after the first FINISH
    if (*s).status == FINISH_STATE && (*z).avail_in != 0 {
        deflate_error = b"Trying to finish while input available\0".as_ptr() as *const c_char;
        return EStatus::Z_BUF_ERROR;
    }

    // Start a new block or continue the current one.
    if (*z).avail_in != 0
        || (*s).lookahead != 0
        || (flush as c_int != EFlush::Z_NO_FLUSH as c_int && (*s).status != FINISH_STATE)
    {
        let bstate: block_state;

        bstate = (configuration_table[(*s).level as usize].func)(s, flush);

        if bstate == block_state::FINISH_STARTED || bstate == block_state::FINISH_DONE {
            (*s).status = FINISH_STATE;
        }
        if bstate == block_state::NEED_MORE || bstate == block_state::FINISH_STARTED {
            if (*z).avail_out == 0 {
                // avoid BUF_ERROR next call, see above
                (*s).last_flush = EFlush::Z_NEED_MORE;
            }
            return EStatus::Z_OK;

            // If flush != Z_NO_FLUSH && avail_out == 0, the next call
            // of deflate should use the same flush parameter to make sure
            // that the flush is complete. So we don't have to output an
            // empty block here, this will be done at next call. This also
            // ensures that for a very small output buffer, we emit at most
            // one empty block.
        }
        if bstate == block_state::BLOCK_DONE {
            // FULL_FLUSH or SYNC_FLUSH
            tr_stored_block(s, null(), 0, false);

            flush_pending(z);
            if (*z).avail_out == 0 {
                // avoid BUF_ERROR at next call, see above
                (*s).last_flush = EFlush::Z_NEED_MORE;
                return EStatus::Z_OK;
            }
        }
    }

    if flush as c_int != EFlush::Z_FINISH as c_int {
        return EStatus::Z_OK;
    }
    if (*s).noheader != 0 {
        return EStatus::Z_STREAM_END;
    }

    // Write the zlib trailer (adler32)
    put_longMSB(s, (*s).adler);
    flush_pending(z);

    // If avail_out is zero, the application will call deflate again
    // to flush the rest. Write the trailer only once!
    (*s).noheader = -1;
    if (*s).pending != 0 { EStatus::Z_OK } else { EStatus::Z_STREAM_END }
}

// ===========================================================================
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn deflateEnd(z: *mut z_stream) -> EStatus {
    let status: c_int;

    assert!(!z.is_null());
    assert!(!(*z).dstate.is_null());

    status = (*((*z).dstate as *mut deflate_state_s)).status as c_int;
    if status != INIT_STATE as c_int
        && status != BUSY_STATE as c_int
        && status != FINISH_STATE as c_int
    {
        deflate_error = b"Invalid state while ending\0".as_ptr() as *const c_char;
        return EStatus::Z_STREAM_ERROR;
    }

    Z_Free((*z).dstate as *mut _);
    (*z).dstate = null_mut();

    if status == BUSY_STATE as c_int {
        deflate_error = b"Ending while in busy state\0".as_ptr() as *const c_char;
        return EStatus::Z_DATA_ERROR;
    }
    EStatus::Z_OK
}

// ===========================================================================
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn deflateError() -> *const c_char {
    deflate_error
}

// ===============================================================================
// External calls
// ===============================================================================

#[no_mangle]
pub unsafe extern "C" fn DeflateFile(
    src: *mut u8,
    uncompressedSize: c_ulong,
    dst: *mut u8,
    maxCompressedSize: c_ulong,
    compressedSize: *mut c_ulong,
    level: ELevel,
    noWrap: c_int,
) -> bool {
    let mut z: z_stream = core::mem::zeroed();

    if deflateInit(&mut z, level, noWrap) != EStatus::Z_OK {
        return false;
    }

    z.next_in    = src;
    z.avail_in   = uncompressedSize;
    z.next_out   = dst;
    z.avail_out  = maxCompressedSize;

    #[cfg(feature = "timing")]
    let temp = timeGetTime();

    if deflate(&mut z, EFlush::Z_FINISH) != EStatus::Z_STREAM_END {
        deflateEnd(&mut z);
        return false;
    }

    #[cfg(feature = "timing")]
    {
        (*addr_of_mut!(totalDeflateTime))[level as usize] += timeGetTime() - temp;
        (*addr_of_mut!(totalDeflateCount))[level as usize] += 1;
    }

    if deflateEnd(&mut z) != EStatus::Z_OK {
        return false;
    }
    *compressedSize = z.total_out;
    true
}

// end
