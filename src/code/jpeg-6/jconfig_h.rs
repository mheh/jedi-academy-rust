//! jconfig.h --- jconfig.h for Watcom C/C++ on MS-DOS or OS/2.
//! see jconfig.doc for explanations

#![allow(non_upper_case_globals)]

use core::ffi::c_int;

pub const HAVE_PROTOTYPES: bool = true;
pub const HAVE_UNSIGNED_CHAR: bool = true;
pub const HAVE_UNSIGNED_SHORT: bool = true;
// #define void char -- not applicable in Rust
// #define const -- not applicable in Rust
pub const CHAR_IS_UNSIGNED: bool = true;
pub const HAVE_STDDEF_H: bool = true;
pub const HAVE_STDLIB_H: bool = true;
// NEED_BSD_STRINGS not defined
// NEED_SYS_TYPES_H not defined
// NEED_FAR_POINTERS not defined (Watcom uses flat 32-bit addressing)
// NEED_SHORT_EXTERNAL_NAMES not defined
// INCOMPLETE_TYPES_BROKEN not defined

pub const JDCT_DEFAULT: c_int = JDCT_FLOAT;
pub const JDCT_FASTEST: c_int = JDCT_FLOAT;

// JPEG_INTERNALS configuration
// RIGHT_SHIFT_IS_UNSIGNED is not defined

// JPEG_CJPEG_DJPEG configuration
pub const BMP_SUPPORTED: bool = true;           // BMP image file format
pub const GIF_SUPPORTED: bool = true;           // GIF image file format
pub const PPM_SUPPORTED: bool = true;           // PBMPLUS PPM/PGM image file format
// RLE_SUPPORTED not defined (Utah RLE image file format)
pub const TARGA_SUPPORTED: bool = true;         // Targa image file format

// TWO_FILE_COMMANDLINE not defined (optional)
pub const USE_SETMODE: bool = true;             // Needed to make one-file style work in Watcom
// NEED_SIGNAL_CATCHER not defined (Define this if you use jmemname.c)
// DONT_USE_B_MODE not defined
// PROGRESS_REPORT not defined (optional)
