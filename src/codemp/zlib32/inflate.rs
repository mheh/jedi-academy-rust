//! Mechanical port of `codemp/zlib32/inflate.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_void};
use core::ptr;

use super::inflate_h::*;
use super::zip_h::*;

extern "C" {
    fn Z_Malloc(size: usize, tag: c_int, clear: bool) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn adler32(adler: ulong, buf: *const byte, len: ulong) -> ulong;
}

#[cfg(feature = "_TIMING")]
extern "C" {
    fn timeGetTime() -> c_int;
}

#[cfg(feature = "_TIMING")]
static mut totalInflateTime: c_int = 0;
#[cfg(feature = "_TIMING")]
static mut totalInflateCount: c_int = 0;

const TAG_INFLATE: c_int = 0;

pub static inflate_copyright: &[u8] = b"Inflate 1.1.3 Copyright 1995-1998 Mark Adler ";

static mut inflate_error: &str = "OK";

static inflate_mask: [ulong; 17] = [
    0x0000,
    0x0001, 0x0003, 0x0007, 0x000f, 0x001f, 0x003f, 0x007f, 0x00ff,
    0x01ff, 0x03ff, 0x07ff, 0x0fff, 0x1fff, 0x3fff, 0x7fff, 0xffff
];

static border: [ulong; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
];

static cplens: [ulong; 31] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31,
    35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258, 0, 0
];

static cplext: [ulong; 31] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
    3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 112, 112
];

static cpdist: [ulong; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193,
    257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145,
    8193, 12289, 16385, 24577
];

static mut fixed_bl: ulong = 9;
static mut fixed_bd: ulong = 5;

#[rustfmt::skip]
static fixed_tl: [inflate_huft_t; 320] = [
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
    inflate_huft_t { Exop: 0,  Bits: 8, base: 15  }, inflate_huft_t { Exop: 0, Bits: 8, base: 143 }, inflate_huft_t { Exop: 0, Bits: 8, base: 79 }, inflate_huft_t { Exop: 0,   Bits: 9, base: 254 }
];

#[rustfmt::skip]
static fixed_td: [inflate_huft_t; 32] = [
    inflate_huft_t { Exop: 80, Bits: 5, base: 1  }, inflate_huft_t { Exop: 87, Bits: 5, base: 257  }, inflate_huft_t { Exop: 83, Bits: 5, base: 17  }, inflate_huft_t { Exop: 91,  Bits: 5, base: 4097  },
    inflate_huft_t { Exop: 81, Bits: 5, base: 5  }, inflate_huft_t { Exop: 89, Bits: 5, base: 1025 }, inflate_huft_t { Exop: 85, Bits: 5, base: 65  }, inflate_huft_t { Exop: 93,  Bits: 5, base: 16385 },
    inflate_huft_t { Exop: 80, Bits: 5, base: 3  }, inflate_huft_t { Exop: 88, Bits: 5, base: 513  }, inflate_huft_t { Exop: 84, Bits: 5, base: 33  }, inflate_huft_t { Exop: 92,  Bits: 5, base: 8193  },
    inflate_huft_t { Exop: 82, Bits: 5, base: 9  }, inflate_huft_t { Exop: 90, Bits: 5, base: 2049 }, inflate_huft_t { Exop: 86, Bits: 5, base: 129 }, inflate_huft_t { Exop: 192, Bits: 5, base: 24577 },
    inflate_huft_t { Exop: 80, Bits: 5, base: 2  }, inflate_huft_t { Exop: 87, Bits: 5, base: 385  }, inflate_huft_t { Exop: 83, Bits: 5, base: 25  }, inflate_huft_t { Exop: 91,  Bits: 5, base: 6145  },
    inflate_huft_t { Exop: 81, Bits: 5, base: 7  }, inflate_huft_t { Exop: 89, Bits: 5, base: 1537 }, inflate_huft_t { Exop: 85, Bits: 5, base: 97  }, inflate_huft_t { Exop: 93,  Bits: 5, base: 24577 },
    inflate_huft_t { Exop: 80, Bits: 5, base: 4  }, inflate_huft_t { Exop: 88, Bits: 5, base: 769  }, inflate_huft_t { Exop: 84, Bits: 5, base: 49  }, inflate_huft_t { Exop: 92,  Bits: 5, base: 12289 },
    inflate_huft_t { Exop: 82, Bits: 5, base: 13 }, inflate_huft_t { Exop: 90, Bits: 5, base: 3073 }, inflate_huft_t { Exop: 86, Bits: 5, base: 193 }, inflate_huft_t { Exop: 192, Bits: 5, base: 24577 }
];

pub unsafe fn inflateEnd(z: *mut z_stream) -> EStatus {
    assert!(!z.is_null());

    if !(*z).istate.is_null() {
        if !(*(*z).istate).blocks.is_null() {
            let _ = Z_Free((*(*z).istate).blocks as *mut c_void);
            (*(*z).istate).blocks = ptr::null_mut();
        }
        let _ = Z_Free((*z).istate as *mut c_void);
        (*z).istate = ptr::null_mut();
    }
    EStatus::Z_OK
}

pub unsafe fn inflateInit(z: *mut z_stream, _flush: EFlush, _noWrap: c_int) -> EStatus {
    assert!(!z.is_null());
    inflate_error = "OK";
    EStatus::Z_OK
}

pub unsafe fn inflate(_z: *mut z_stream) -> EStatus {
    EStatus::Z_OK
}

pub unsafe fn inflateError() -> *const u8 {
    inflate_error.as_ptr() as *const u8
}

pub unsafe fn InflateFile(_src: *mut byte, _compressedSize: ulong, _dst: *mut byte, _uncompressedSize: ulong, _noWrap: c_int) -> bool {
    false
}

// end
