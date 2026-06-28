#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_int, c_long, c_short, c_uchar};

pub use super::small_header_h::{IN_OUT, SAMPLE};

/* portable copy of eco\mhead.h */
/* mpeg audio header   */
#[repr(C)]
pub struct MPEG_HEAD {
    pub sync: c_int, /* 1 if valid sync */
    pub id: c_int,
    pub option: c_int,
    pub prot: c_int,
    pub br_index: c_int,
    pub sr_index: c_int,
    pub pad: c_int,
    pub private_bit: c_int,
    pub mode: c_int,
    pub mode_ext: c_int,
    pub cr: c_int,
    pub original: c_int,
    pub emphasis: c_int,
}

/* portable mpeg audio decoder, decoder functions */

#[repr(C)]
pub struct DEC_INFO {
    pub channels: c_int,
    pub outvalues: c_int,
    pub samprate: c_long,
    pub bits: c_int,
    pub framebytes: c_int,
    pub r#type: c_int,
}

const _: () = assert!(core::mem::size_of::<MPEG_HEAD>() == 52);
const _: () = assert!(core::mem::align_of::<MPEG_HEAD>() == 4);

const _: () = {
    let expected = if core::mem::size_of::<c_long>() == 8 {
        32
    } else {
        24
    };
    assert!(core::mem::size_of::<DEC_INFO>() == expected);
};
const _: () = assert!(core::mem::align_of::<DEC_INFO>() == core::mem::align_of::<c_long>());

unsafe extern "C" {
    pub fn audio_decode_init(
        h: *mut MPEG_HEAD,
        framebytes_arg: c_int,
        reduction_code: c_int,
        transform_code: c_int,
        convert_code: c_int,
        freq_limit: c_int,
    ) -> c_int;
    pub fn audio_decode_info(info: *mut DEC_INFO);
    pub fn audio_decode(
        bs: *mut c_uchar,
        pcm: *mut c_short,
        pNextByteAfterData: *mut c_uchar,
    ) -> IN_OUT;

    pub fn audio_decode8_init(
        h: *mut MPEG_HEAD,
        framebytes_arg: c_int,
        reduction_code: c_int,
        transform_code: c_int,
        convert_code: c_int,
        freq_limit: c_int,
    ) -> c_int;
    pub fn audio_decode8_info(info: *mut DEC_INFO);
    pub fn audio_decode8(bs: *mut c_uchar, pcmbuf: *mut c_short) -> IN_OUT;
}
