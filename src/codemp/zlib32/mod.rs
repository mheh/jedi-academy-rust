//! Header ports for original `codemp/zlib32/` sources.

#![allow(non_camel_case_types)]

pub mod deflate_h;
pub mod inflate_h;
pub mod zip_h;

pub use zip_h::{
    byte, ulong, word, z_stream, z_stream_s, D_CODES, MAX_MATCH, MAX_WBITS, MIN_MATCH,
    WINDOW_SIZE, EFlush, ELevel, EStatus,
};
