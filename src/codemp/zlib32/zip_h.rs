//! Mechanical port of `codemp/zlib32/zip.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_ulong};

use super::{deflate_h, inflate_h};

/// `byte` - unsigned char.
pub type byte = u8;

/// `word` - unsigned short.
pub type word = u16;

/// `ulong` - unsigned long.
pub type ulong = c_ulong;

// The deflate compression method
pub const ZF_STORED: c_int = 0;
pub const ZF_DEFLATED: c_int = 8;

// Compression levels
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ELevel {
    Z_STORE_COMPRESSION,
    Z_FAST_COMPRESSION_LOW,
    Z_FAST_COMPRESSION,
    Z_FAST_COMPRESSION_HIGH,
    Z_SLOW_COMPRESSION_LOWEST,
    Z_SLOW_COMPRESSION_LOW,
    Z_DEFAULT_COMPRESSION,
    Z_SLOW_COMPRESSION_HIGH,
    Z_SLOW_COMPRESSION_HIGHEST,
    Z_MAX_COMPRESSION,
}

// Allowed flush values
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EFlush {
    Z_NEED_MORE = -1, // Special case when finishing up the stream
    Z_NO_FLUSH,
    Z_SYNC_FLUSH, // Sync up the stream ready for another call
    Z_FINISH,     // Finish up the stream
}

// Return codes for the compression/decompression functions. Negative
// values are errors, positive values are used for special but normal events.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EStatus {
    Z_STREAM_ERROR = -3, // Basic error from failed sanity checks
    Z_BUF_ERROR,         // Not enough input or output
    Z_DATA_ERROR,        // Invalid data in the stream
    Z_OK,
    Z_STREAM_END, // End of stream
}

// Maximum value for windowBits in deflateInit and inflateInit.
// The memory requirements for inflate are (in bytes) 1 << windowBits
// that is, 32K for windowBits=15 (default value) plus a few kilobytes
// for small objects.
pub const MAX_WBITS: usize = 15; // 32K LZ77 window
pub const WINDOW_SIZE: usize = 1 << MAX_WBITS;
pub const BIG_WINDOW_SIZE: usize = WINDOW_SIZE << 1;
pub const WINDOW_MASK: usize = WINDOW_SIZE - 1;

// The three kinds of block type
pub const STORED_BLOCK: c_int = 0;
pub const STATIC_TREES: c_int = 1;
pub const DYN_TREES: c_int = 2;
pub const MODE_ILLEGAL: c_int = 3;

// The minimum and maximum match lengths
pub const MIN_MATCH: usize = 3;
pub const MAX_MATCH: usize = 258;

// number of distance codes
pub const D_CODES: usize = 30;

extern "C" {
    pub static extra_dbits: [ulong; D_CODES];
}

// Structure to be used by external applications

//  The application must update next_in and avail_in when avail_in has
//  dropped to zero. It must update next_out and avail_out when avail_out
//  has dropped to zero. All other fields are set by the
//  compression library and must not be updated by the application.
#[repr(C)]
#[derive(Debug)]
pub struct z_stream_s {
    pub next_in: *mut byte, // next input unsigned char
    pub avail_in: ulong,   // number of unsigned chars available at next_in
    pub total_in: ulong,   // total number of bytes processed so far

    pub next_out: *mut byte, // next output unsigned char should be put there
    pub avail_out: ulong,   // remaining free space at next_out
    pub total_out: ulong,   // total number of bytes output

    pub status: EStatus,
    pub error: EStatus, // error code

    pub istate: *mut inflate_h::inflate_state_s, // not visible by applications
    pub dstate: *mut deflate_h::deflate_state_s, // not visible by applications

    pub quality: ulong,
}

pub type z_stream = z_stream_s;

extern "C" {
    pub fn crc32(crc: ulong, buf: *const byte, len: ulong) -> ulong;
    pub fn adler32(adler: ulong, buf: *const byte, len: ulong) -> ulong;

    pub fn deflateInit(strm: *mut z_stream, level: ELevel, noWrap: c_int) -> EStatus;
    pub fn deflateCopy(dest: *mut z_stream, source: *mut z_stream) -> EStatus;
    pub fn deflate(strm: *mut z_stream, flush: EFlush) -> EStatus;
    pub fn deflateEnd(strm: *mut z_stream) -> EStatus;
    pub fn deflateError() -> *const c_char;

    pub fn inflateInit(strm: *mut z_stream, flush: EFlush, noWrap: c_int) -> EStatus;
    pub fn inflate(z: *mut z_stream) -> EStatus;
    pub fn inflateEnd(strm: *mut z_stream) -> EStatus;
    pub fn inflateError() -> *const c_char;

    pub fn InflateFile(
        src: *mut byte,
        compressedSize: ulong,
        dst: *mut byte,
        uncompressedSize: ulong,
        noWrap: c_int,
    ) -> bool;
    pub fn DeflateFile(
        src: *mut byte,
        uncompressedSize: ulong,
        dst: *mut byte,
        maxCompressedSize: ulong,
        compressedSize: *mut ulong,
        level: ELevel,
        noWrap: c_int,
    ) -> bool;
}

// end
