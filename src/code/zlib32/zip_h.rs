use core::ffi::{c_char, c_int, c_ulong};

//
// zlib.h -- interface of the 'zlib' general purpose compression library
//  version 1.1.3, July 9th, 1998
//
//  Copyright (C) 1995-1998 Jean-loup Gailly and Mark Adler
//
//  This software is provided 'as-is', without any express or implied
//  warranty.  In no event will the authors be held liable for any damages
//  arising from the use of this software.
//
//  Permission is granted to anyone to use this software for any purpose,
//  including commercial applications, and to alter it and redistribute it
//  freely, subject to the following restrictions:
//
//  1. The origin of this software must not be misrepresented; you must not
//     claim that you wrote the original software. If you use this software
//     in a product, an acknowledgment in the product documentation would be
//     appreciated but is not required.
//  2. Altered source versions must be plainly marked as such, and must not be
//     misrepresented as being the original software.
//  3. This notice may not be removed or altered from any source distribution.
//
//  Jean-loup Gailly        Mark Adler
//  jloup@gzip.org          madler@alumni.caltech.edu
//
//  The data format used by the zlib library is described by RFCs (Request for
//  Comments) 1950 to 1952 in the files ftp://ds.internic.net/rfc/rfc1950.txt
//  (zlib format), rfc1951.txt (deflate format) and rfc1952.txt (gzip format).
//

//     The 'zlib' compression library provides in-memory compression and
//  decompression functions, including integrity checks of the uncompressed
//  data.  This version of the library supports only one compression method
//  (deflation) but other algorithms will be added later and will have the same
//  stream interface.
//
//     Compression can be done in a single step if the buffers are large
//  enough (for example if an input file is mmap'ed), or can be done by
//  repeated calls of the compression function.  In the latter case, the
//  application must provide more input and/or consume the output
//  (providing more output space) before each call.
//
//     The library does not install any signal handler. The decoder checks
//  the consistency of the compressed data, so the library should never
//  crash even in case of corrupted input.

// This particular implementation has been heavily modified by jscott@ravensoft.com
// to increase inflate/deflate speeds on 32 bit machines.

// for more info about .ZIP format, see
//    ftp://ftp.cdrom.com/pub/infozip/doc/appnote-970311-iz.zip
// PkWare has also a specification at :
//    ftp://ftp.pkware.com/probdesc.zip

// ========================================================================================
// External calls and defines required for the zlib
// ========================================================================================

// The deflate compression method
pub const ZF_STORED: c_int = 0;
pub const ZF_DEFLATED: c_int = 8;

// Compression levels
#[repr(C)]
pub enum ELevel {
    Z_STORE_COMPRESSION = 0,
    Z_FAST_COMPRESSION_LOW = 1,
    Z_FAST_COMPRESSION = 2,
    Z_FAST_COMPRESSION_HIGH = 3,
    Z_SLOW_COMPRESSION_LOWEST = 4,
    Z_SLOW_COMPRESSION_LOW = 5,
    Z_DEFAULT_COMPRESSION = 6,
    Z_SLOW_COMPRESSION_HIGH = 7,
    Z_SLOW_COMPRESSION_HIGHEST = 8,
    Z_MAX_COMPRESSION = 9,
}

// Allowed flush values
#[repr(C)]
pub enum EFlush {
    Z_NEED_MORE = -1, // Special case when finishing up the stream
    Z_NO_FLUSH = 0,
    Z_SYNC_FLUSH = 1, // Sync up the stream ready for another call
    Z_FINISH = 2,     // Finish up the stream
}

// Return codes for the compression/decompression functions. Negative
// values are errors, positive values are used for special but normal events.
#[repr(C)]
pub enum EStatus {
    Z_STREAM_ERROR = -3, // Basic error from failed sanity checks
    Z_BUF_ERROR = -2,    // Not enough input or output
    Z_DATA_ERROR = -1,   // Invalid data in the stream
    Z_OK = 0,
    Z_STREAM_END = 1,    // End of stream
}

// Maximum value for windowBits in deflateInit and inflateInit.
// The memory requirements for inflate are (in bytes) 1 << windowBits
// that is, 32K for windowBits=15 (default value) plus a few kilobytes
// for small objects.
pub const MAX_WBITS: c_int = 15; // 32K LZ77 window
pub const WINDOW_SIZE: usize = 1 << MAX_WBITS;
pub const BIG_WINDOW_SIZE: usize = WINDOW_SIZE << 1;
pub const WINDOW_MASK: usize = WINDOW_SIZE - 1;

// The three kinds of block type
pub const STORED_BLOCK: c_int = 0;
pub const STATIC_TREES: c_int = 1;
pub const DYN_TREES: c_int = 2;
pub const MODE_ILLEGAL: c_int = 3;

// The minimum and maximum match lengths
pub const MIN_MATCH: c_int = 3;
pub const MAX_MATCH: c_int = 258;

// number of distance codes
pub const D_CODES: c_int = 30;

extern "C" {
    pub static extra_dbits: [c_ulong; 30];
}

// Structure to be used by external applications

//  The application must update next_in and avail_in when avail_in has
//  dropped to zero. It must update next_out and avail_out when avail_out
//  has dropped to zero. All other fields are set by the
//  compression library and must not be updated by the application.

// Opaque forward declarations for internal state structures
#[repr(C)]
pub struct inflate_state_s {
    // Internal state - not exposed in header
}

#[repr(C)]
pub struct deflate_state_s {
    // Internal state - not exposed in header
}

#[repr(C)]
pub struct z_stream_s {
    pub next_in: *mut u8,      // next input unsigned char
    pub avail_in: c_ulong,     // number of unsigned chars available at next_in
    pub total_in: c_ulong,     // total number of bytes processed so far

    pub next_out: *mut u8,     // next output unsigned char should be put there
    pub avail_out: c_ulong,    // remaining free space at next_out
    pub total_out: c_ulong,    // total number of bytes output

    pub status: EStatus,
    pub error: EStatus,        // error code

    pub istate: *mut inflate_state_s, // not visible by applications
    pub dstate: *mut deflate_state_s, // not visible by applications

    pub quality: c_ulong,
}

pub type z_stream = z_stream_s;

//     Update a running crc with the bytes buf[0..len-1] and return the updated
//   crc. If buf is NULL, this function returns the required initial value
//   for the crc. Pre- and post-conditioning (one's complement) is performed
//   within this function so it shouldn't be done by the application.
//   Usage example:
//
//     ulong crc = crc32(0L, NULL, 0);
//
//     while (read_buffer(buffer, length) != EOF) {
//       crc = crc32(crc, buffer, length);
//     }
//     if (crc != original_crc) error();

extern "C" {
    pub fn crc32(crc: c_ulong, buf: *const u8, len: c_ulong) -> c_ulong;
}

//     Update a running Adler-32 checksum with the bytes buf[0..len-1] and
//   return the updated checksum. If buf is NULL, this function returns
//   the required initial value for the checksum.
//   An Adler-32 checksum is almost as reliable as a CRC32 but can be computed
//   much faster. Usage example:
//
//     ulong adler = adler32(0L, NULL, 0);
//
//     while (read_buffer(buffer, length) != EOF) {
//       adler = adler32(adler, buffer, length);
//     }
//     if (adler != original_adler) error();

extern "C" {
    pub fn adler32(adler: c_ulong, buf: *const u8, len: c_ulong) -> c_ulong;
}

// External calls to the deflate code
extern "C" {
    pub fn deflateInit(strm: *mut z_stream, level: ELevel, noWrap: c_int) -> EStatus;
    pub fn deflateCopy(dest: *mut z_stream, source: *mut z_stream) -> EStatus;
    pub fn deflate(strm: *mut z_stream, flush: EFlush) -> EStatus;
    pub fn deflateEnd(strm: *mut z_stream) -> EStatus;
    pub fn deflateError() -> *const c_char;
}

// External calls to the inflate code
extern "C" {
    pub fn inflateInit(strm: *mut z_stream, flush: EFlush, noWrap: c_int) -> EStatus;
    pub fn inflate(z: *mut z_stream) -> EStatus;
    pub fn inflateEnd(strm: *mut z_stream) -> EStatus;
    pub fn inflateError() -> *const c_char;
}

// External calls to the zipfile code
extern "C" {
    pub fn InflateFile(
        src: *mut u8,
        compressedSize: c_ulong,
        dst: *mut u8,
        uncompressedSize: c_ulong,
        noWrap: c_int,
    ) -> bool;
    pub fn DeflateFile(
        src: *mut u8,
        uncompressedSize: c_ulong,
        dst: *mut u8,
        maxCompressedSize: c_ulong,
        compressedSize: *mut c_ulong,
        level: ELevel,
        noWrap: c_int,
    ) -> bool;
}

// end
