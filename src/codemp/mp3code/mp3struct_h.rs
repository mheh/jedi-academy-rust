// Filename: mp3struct.h
//
// this file is my struct to gather all loose MP3 global vars into one struct so we can do multiple-stream decompression
//

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_short, c_uchar, c_uint, c_void};

use super::small_header_h::{IN_OUT, SAMPLE};

pub type byte = c_uchar;

pub type SBT_FUNCTION = Option<unsafe extern "C" fn(sample: *mut f32, pcm: *mut c_short, n: c_int)>;
pub type XFORM_FUNCTION = Option<unsafe extern "C" fn(pcm: *mut c_void, igr: c_int)>;
pub type DECODE_FUNCTION = Option<unsafe extern "C" fn(bs: *mut c_uchar, pcm: *mut c_uchar) -> IN_OUT>;

pub const NBUF: usize = 8 * 1024;
pub const BUF_TRIGGER: usize = NBUF - 1500;

#[repr(C)]
#[derive(Clone, Copy)]
pub union MP3STREAM_u {
    pub L1_2: MP3STREAM_L1_2,
    pub L3: MP3STREAM_L3,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MP3STREAM_L1_2 {
    pub sbt: SBT_FUNCTION,

    pub cs_factor: [[f32; 64]; 3], /* 768 bytes */

    pub nbat: [c_int; 4],
    pub bat: [[c_int; 16]; 4],
    pub max_sb: c_int,
    pub stereo_sb: c_int,
    pub bit_skip: c_int,

    pub cs_factorL1: *mut f32,
    pub nbatL1: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MP3STREAM_L3 {
    pub sbt_L3: SBT_FUNCTION,
    pub Xform: XFORM_FUNCTION,
    pub decode_function: DECODE_FUNCTION,

    pub sample: [[[SAMPLE; 576]; 2]; 2], /* if this isn't kept per stream then the decode breaks up */

    // the 4k version of these 2 seems to work for everything, but I'm reverting to original 8k for safety jic.
    //
    // #define NBUF (4096) // 2048 works for all except 133+ kbps VBR files, 4096 copes with these
    // #define BUF_TRIGGER ((NBUF/4)*3)

    pub buf: [c_uchar; NBUF],
    pub buf_ptr0: c_int,
    pub buf_ptr1: c_int,
    pub main_pos_bit: c_int,

    pub band_limit_nsb: c_int,
    pub nBand: [[c_int; 22]; 2],       /* [long/short][cb] */
    pub sfBandIndex: [[c_int; 22]; 2], /* [long/short][cb] */
    pub half_outbytes: c_int,
    pub crcbytes: c_int,
    pub nchan: c_int,
    pub ms_mode: c_int,
    pub is_mode: c_int,
    pub zero_level_pcm: c_uint,
    pub mpeg25_flag: c_int,
    pub band_limit: c_int,
    pub band_limit21: c_int,
    pub band_limit12: c_int,
    pub gain_adjust: c_int,
    pub ncbl_mixed: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MP3STREAM {
    pub u: MP3STREAM_u,
    // from csbt.c...
    //
    // if this isn't kept per stream then the decode breaks up
    pub vb_ptr: c_int,  //
    pub vb2_ptr: c_int, //
    pub vbuf: [f32; 512],
    pub vbuf2: [f32; 512], // this can be lost if we stick to mono samples

    // L3 only...
    //
    pub sr_index: c_int, // L3 only (99%)
    pub id: c_int,

    // any type...
    //
    pub outvalues: c_int,
    pub outbytes: c_int,
    pub framebytes: c_int,
    pub pad: c_int,
    pub nsb_limit: c_int,

    // stuff added now that the game uses streaming MP3s...
    //
    pub pbSourceData: *mut byte, // a useful dup ptr only, this whole struct will be owned by an sfx_t struct that has the actual data ptr field
    pub iSourceBytesRemaining: c_int,
    pub iSourceReadIndex: c_int,
    pub iSourceFrameBytes: c_int,
    #[cfg(debug_assertions)]
    pub iSourceFrameCounter: c_int, // not really important
    pub iBytesDecodedTotal: c_int,
    pub iBytesDecodedThisPacket: c_int, // not sure how useful this will be, it's only per-frame, so will always be full frame size (eg 2304 or below for mono) except possibly for the last frame?

    pub iRewind_FinalReductionCode: c_int,
    pub iRewind_FinalConvertCode: c_int,
    pub iRewind_SourceBytesRemaining: c_int,
    pub iRewind_SourceReadIndex: c_int,
    pub bDecodeBuffer: [byte; 2304 * 2], // *2 to allow for stereo now
    pub iCopyOffset: c_int,              // used for painting to DMA-feeder, since 2304 won't match the size it wants

    // some new stuff added for dynamic music, to allow "how many seconds left to play" queries...
    //
    // ( m_lengthInSeconds = ((iUnpackedDataLength / iRate) / iChannels) / iWidth; )
    //
    // Note that these fields are only valid/initialised if MP3Stream_InitPlayingTimeFields() was called.
    // If not, this->iTimeQuery_UnpackedLength will be zero.
    //
    pub iTimeQuery_UnpackedLength: c_int,
    pub iTimeQuery_SampleRate: c_int,
    pub iTimeQuery_Channels: c_int,
    pub iTimeQuery_Width: c_int,
}

pub type LP_MP3STREAM = *mut MP3STREAM;

unsafe extern "C" {
    pub static mut pMP3Stream: LP_MP3STREAM;
    pub static mut bFastEstimateOnly: c_int;
}

const _: () = assert!(core::mem::size_of::<SBT_FUNCTION>() == core::mem::size_of::<*mut c_void>());
const _: () = assert!(core::mem::size_of::<XFORM_FUNCTION>() == core::mem::size_of::<*mut c_void>());
const _: () = assert!(core::mem::size_of::<DECODE_FUNCTION>() == core::mem::size_of::<*mut c_void>());

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<MP3STREAM_L1_2>() == 1080);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<MP3STREAM_L1_2>() == 1064);
const _: () = assert!(core::mem::align_of::<MP3STREAM_L1_2>() == core::mem::align_of::<*mut c_void>());

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<MP3STREAM_L3>() == 17848);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<MP3STREAM_L3>() == 17836);
const _: () = assert!(core::mem::align_of::<MP3STREAM_L3>() == core::mem::align_of::<*mut c_void>());

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<MP3STREAM_u>() == 17848);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<MP3STREAM_u>() == 17836);
const _: () = assert!(core::mem::align_of::<MP3STREAM_u>() == core::mem::align_of::<*mut c_void>());

#[cfg(all(target_pointer_width = "64", not(debug_assertions)))]
const _: () = assert!(core::mem::size_of::<MP3STREAM>() == 26656);
#[cfg(all(target_pointer_width = "64", debug_assertions))]
const _: () = assert!(core::mem::size_of::<MP3STREAM>() == 26664);
#[cfg(all(target_pointer_width = "32", not(debug_assertions)))]
const _: () = assert!(core::mem::size_of::<MP3STREAM>() == 26636);
#[cfg(all(target_pointer_width = "32", debug_assertions))]
const _: () = assert!(core::mem::size_of::<MP3STREAM>() == 26640);
const _: () = assert!(core::mem::align_of::<MP3STREAM>() == core::mem::align_of::<*mut c_void>());
