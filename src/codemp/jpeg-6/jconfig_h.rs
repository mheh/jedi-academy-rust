//! jconfig.h --- jconfig.h for Watcom C/C++ on MS-DOS or OS/2.
//! see jconfig.doc for explanations

pub const HAVE_PROTOTYPES: bool = true;
pub const HAVE_UNSIGNED_CHAR: bool = true;
pub const HAVE_UNSIGNED_SHORT: bool = true;
// #define void char
// #define const
pub const CHAR_IS_UNSIGNED: bool = true;
pub const HAVE_STDDEF_H: bool = true;
pub const HAVE_STDLIB_H: bool = true;
pub const NEED_BSD_STRINGS: bool = false;
pub const NEED_SYS_TYPES_H: bool = false;
pub const NEED_FAR_POINTERS: bool = false; // Watcom uses flat 32-bit addressing
pub const NEED_SHORT_EXTERNAL_NAMES: bool = false;
pub const INCOMPLETE_TYPES_BROKEN: bool = false;

// JDCT_FLOAT must be defined in jpeglib headers
pub const JDCT_DEFAULT: i32 = 2; // JDCT_FLOAT
pub const JDCT_FASTEST: i32 = 2; // JDCT_FLOAT

#[cfg(feature = "jpeg_internals")]
pub mod jpeg_internals {
    pub const RIGHT_SHIFT_IS_UNSIGNED: bool = false;
}

#[cfg(feature = "jpeg_cjpeg_djpeg")]
pub mod jpeg_cjpeg_djpeg {
    pub const BMP_SUPPORTED: bool = true; // BMP image file format
    pub const GIF_SUPPORTED: bool = true; // GIF image file format
    pub const PPM_SUPPORTED: bool = true; // PBMPLUS PPM/PGM image file format
    pub const RLE_SUPPORTED: bool = false; // Utah RLE image file format
    pub const TARGA_SUPPORTED: bool = true; // Targa image file format

    pub const TWO_FILE_COMMANDLINE: bool = false; // optional
    pub const USE_SETMODE: bool = true; // Needed to make one-file style work in Watcom
    pub const NEED_SIGNAL_CATCHER: bool = false; // Define this if you use jmemname.c
    pub const DONT_USE_B_MODE: bool = false;
    pub const PROGRESS_REPORT: bool = false; // optional
}
