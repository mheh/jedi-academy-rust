// Filename:	mp3struct.h
//
// this file is my struct to gather all loose MP3 global vars into one struct so we can do multiple-stream decompression
//

use crate::code::mp3code::small_header_h::{SAMPLE, IN_OUT};
use core::ffi::{c_int, c_char, c_void, c_short, c_uchar, c_uint};

pub type SBT_FUNCTION = extern "C" fn(*mut f32, *mut c_short, c_int);
pub type XFORM_FUNCTION = extern "C" fn(*mut c_void, c_int);
pub type DECODE_FUNCTION = extern "C" fn(*mut c_uchar, *mut c_uchar) -> IN_OUT;

#[repr(C)]
pub struct MP3STREAM {
    pub inner: MP3STREAM_Union,

    // from csbt.c...
    //
    // if this isn't kept per stream then the decode breaks up
    pub vb_ptr: c_int,
    //
    pub vb2_ptr: c_int,
    //
    pub vbuf: [f32; 512],
    //
    // this can be lost if we stick to mono samples
    pub vbuf2: [f32; 512],

    // L3 only...
    //
    // L3 only (99%)
    pub sr_index: c_int,
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
    // a useful dup ptr only, this whole struct will be owned by an sfx_t struct that has the actual data ptr field
    pub pbSourceData: *mut u8,
    pub iSourceBytesRemaining: c_int,
    pub iSourceReadIndex: c_int,
    pub iSourceFrameBytes: c_int,
    #[cfg(debug_assertions)]
    pub iSourceFrameCounter: c_int,
    //
    pub iBytesDecodedTotal: c_int,
    // not sure how useful this will be, it's only per-frame, so will always be full frame size (eg 2304 or below for mono) except possibly for the last frame?
    pub iBytesDecodedThisPacket: c_int,

    pub iRewind_FinalReductionCode: c_int,
    pub iRewind_FinalConvertCode: c_int,
    pub iRewind_SourceBytesRemaining: c_int,
    pub iRewind_SourceReadIndex: c_int,
    // *2 to allow for stereo now
    pub bDecodeBuffer: [u8; 2304 * 2],
    // used for painting to DMA-feeder, since 2304 won't match the size it wants
    pub iCopyOffset: c_int,

    // some new stuff added for dynamic music, to allow "how many seconds left to play" queries...
    //
    // ( m_lengthInSeconds = ((iUnpackedDataLength / iRate) / iChannels) / iWidth; )
    //
    // Note that these fields are only valid/initialised if MP3Stream_InitPlayingTimeFields() was called.
    //	If not, this->iTimeQuery_UnpackedLength will be zero.
    //
    pub iTimeQuery_UnpackedLength: c_int,
    pub iTimeQuery_SampleRate: c_int,
    pub iTimeQuery_Channels: c_int,
    pub iTimeQuery_Width: c_int,
}

#[repr(C)]
pub union MP3STREAM_Union {
    pub l1_2: MP3STREAM_L1_2,
    pub l3: MP3STREAM_L3,
}

#[repr(C)]
pub struct MP3STREAM_L1_2 {
    pub sbt: SBT_FUNCTION,

    // 768 bytes
    pub cs_factor: [[f32; 64]; 3],

    pub nbat: [c_int; 4],
    pub bat: [[c_int; 16]; 4],
    pub max_sb: c_int,
    pub stereo_sb: c_int,
    pub bit_skip: c_int,

    pub cs_factorL1: *mut f32,
    pub nbatL1: c_int,
}

#[repr(C)]
pub struct MP3STREAM_L3 {
    pub sbt_L3: SBT_FUNCTION,
    pub Xform: XFORM_FUNCTION,
    pub decode_function: DECODE_FUNCTION,

    // if this isn't kept per stream then the decode breaks up
    pub sample: [[[SAMPLE; 576]; 2]; 2],

    // the 4k version of these 2 seems to work for everything, but I'm reverting to original 8k for safety jic.
    //
    // #define NBUF (8*1024)
    // #define BUF_TRIGGER (NBUF-1500)
    //	#define NBUF (4096)	// 2048 works for all except 133+ kbps VBR files, 4096 copes with these
    //	#define BUF_TRIGGER ((NBUF/4)*3)

    pub buf: [u8; 8 * 1024],
    pub buf_ptr0: c_int,
    pub buf_ptr1: c_int,
    pub main_pos_bit: c_int,

    pub band_limit_nsb: c_int,
    /* [long/short][cb] */
    pub nBand: [[c_int; 22]; 2],
    /* [long/short][cb] */
    pub sfBandIndex: [[c_int; 22]; 2],
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

pub const NBUF: usize = 8 * 1024;
pub const BUF_TRIGGER: usize = NBUF - 1500;

pub type LP_MP3STREAM = *mut MP3STREAM;

#[allow(non_upper_case_globals)]
extern "C" {
    pub static mut pMP3Stream: LP_MP3STREAM;
    pub static mut bFastEstimateOnly: c_int;
}

////////////////// eof /////////////////////
