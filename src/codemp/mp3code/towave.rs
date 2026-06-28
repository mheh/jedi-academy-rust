#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_short, c_uchar, c_uint, c_void};
use core::ptr::{addr_of_mut, null_mut};

use super::cup::audio_decode;
use super::cupini::{audio_decode_info, audio_decode_init};
use super::mhead::head_info3;
use super::mhead_h::{DEC_INFO, MPEG_HEAD};
use super::mp3struct_h::{byte, LP_MP3STREAM, MP3STREAM};
use super::small_header_h::IN_OUT;

/*---- towave.c --------------------------------------------
  32 bit version only

decode mpeg Layer I/II/III file using portable ANSI C decoder,
output to pcm wave file.

This file exists for reference only in the original tree, but the game-side
MP3 stream globals live here.
-----------------------------------------------------------*/

#[repr(C)]
pub struct id3v1_1 {
    pub id: [c_char; 3],
    pub title: [c_char; 30],
    pub artist: [c_char; 30],
    pub album: [c_char; 30],
    pub year: [c_char; 4],
    pub comment: [c_char; 28],
    pub zero: c_char,
    pub track: c_char,
    pub genre: c_char,
}

const _: () = assert!(core::mem::size_of::<id3v1_1>() == 128);
const _: () = assert!(core::mem::align_of::<id3v1_1>() == 1);

#[no_mangle]
pub static mut gpTAG: *mut id3v1_1 = null_mut();

unsafe fn BYTESREMAINING_ACCOUNT_FOR_REAR_TAG(_pvData: *mut c_void, _iBytesRemaining: *mut c_int) {
    gpTAG = (_pvData as *mut byte)
        .offset(*_iBytesRemaining as isize)
        .offset(-(core::mem::size_of::<id3v1_1>() as isize)) as *mut id3v1_1;
    if (*gpTAG).id[0] as c_int == b'T' as c_int
        && (*gpTAG).id[1] as c_int == b'A' as c_int
        && (*gpTAG).id[2] as c_int == b'G' as c_int
    {
        *_iBytesRemaining -= core::mem::size_of::<id3v1_1>() as c_int;
    }
}

/********  pcm buffer ********/

pub const PCM_BUFBYTES: usize = 60000usize;

#[no_mangle]
pub static mut PCM_Buffer: [c_char; PCM_BUFBYTES] = [0; PCM_BUFBYTES];

pub type decode_init_fn = unsafe extern "C" fn(
    h: *mut MPEG_HEAD,
    framebytes_arg: c_int,
    reduction_code: c_int,
    transform_code: c_int,
    convert_code: c_int,
    freq_limit: c_int,
) -> c_int;
pub type decode_info_fn = unsafe extern "C" fn(info: *mut DEC_INFO);
pub type decode_fn = unsafe extern "C" fn(
    bs: *mut c_uchar,
    pcm: *mut c_short,
    pNextByteAfterData: *mut c_uchar,
) -> IN_OUT;

#[repr(C)]
pub struct AUDIO {
    pub decode_init: decode_init_fn,
    pub decode_info: decode_info_fn,
    pub decode: decode_fn,
}

static audio_table: [[AUDIO; 2]; 2] = [
    [
        AUDIO { decode_init: audio_decode_init, decode_info: audio_decode_info, decode: audio_decode },
        AUDIO { decode_init: audio_decode_init, decode_info: audio_decode_info, decode: audio_decode },
    ],
    [
        AUDIO { decode_init: audio_decode_init, decode_info: audio_decode_info, decode: audio_decode },
        AUDIO { decode_init: audio_decode_init, decode_info: audio_decode_info, decode: audio_decode },
    ],
];

static audio: AUDIO = AUDIO {
    decode_init: audio_decode_init,
    decode_info: audio_decode_info,
    decode: audio_decode,
};

// Do NOT change these, ever!!!!!!!!!!!!!!!!!!
pub const reduction_code: c_int = 0;
pub const convert_code_mono: c_int = 1;
pub const convert_code_stereo: c_int = 0;
pub const freq_limit: c_int = 24000;

#[no_mangle]
pub static mut _MP3Stream: MP3STREAM = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut pMP3Stream: LP_MP3STREAM = addr_of_mut!(_MP3Stream);

#[no_mangle]
pub static mut bFastEstimateOnly: c_int = 0;

static MP3ERR_Bad_or_unsupported_file: &[u8] = b"MP3ERR: Bad or unsupported file!\0";
static MP3ERR_Sound_file_is_stereo: &[u8] = b"MP3ERR: Sound file is stereo!\0";
static MP3ERR_Source_file_has_output_packet_size_stereo: &[u8] =
    b"MP3ERR: Source file has output packet size > 2304 (*2 for stereo) bytes!\0";
static MP3ERR_Source_file_has_output_packet_size: &[u8] =
    b"MP3ERR: Source file has output packet size > 2304 bytes!\0";
static MP3ERR_Source_file_is_not_16bit: &[u8] = b"MP3ERR: Source file is not 16bit!\0";
static MP3ERR_Source_file_is_not_sampled_44100: &[u8] =
    b"MP3ERR: Source file is not sampled @ 44100!\0";
static MP3ERR_Source_file_is_not_stereo: &[u8] = b"MP3ERR: Source file is not stereo!\0";
static MP3ERR_Decoder_failed_to_initialise: &[u8] =
    b"MP3ERR: Decoder failed to initialise\0";
static MP3ERR_Bad_or_Unsupported_MP3_file: &[u8] =
    b"MP3ERR: Bad or Unsupported MP3 file!\0";
static MP3ERR_Decoder_unable_to_convert: &[u8] =
    b"MP3ERR: Decoder unable to convert to current game audio settings\0";
static MP3ERR_Decoder_failed_pass_2: &[u8] =
    b"MP3ERR: Decoder failed to initialise for pass 2 sample adjust\0";
static MP3ERR_Something_broken: &[u8] =
    b"MP3ERR: Errr.... something's broken with this MP3 file\0";
static MP3ERR_Failed_reinit_rewind: &[u8] =
    b"MP3ERR: Failed to re-init decoder for rewind!\0";
static MP3ERR_Frame_bytes_mismatch: &[u8] =
    b"MP3ERR: Frame bytes mismatch during rewind header-read!\0";

#[inline]
fn cstr(s: &'static [u8]) -> *mut c_char {
    s.as_ptr() as *mut c_char
}

// char *return is NZ for any errors (no trailing CR!)
#[no_mangle]
pub unsafe extern "C" fn C_MP3_IsValid(
    pvData: *mut c_void,
    iDataLen: c_int,
    bStereoDesired: c_int,
) -> *mut c_char {
    let mut iRealDataStart: c_uint = 0;
    let mut head: MPEG_HEAD = core::mem::zeroed();
    let mut decinfo: DEC_INFO = core::mem::zeroed();
    let mut iBitRate: c_int = 0;
    let iFrameBytes: c_int;

    core::ptr::write_bytes(pMP3Stream, 0, 1);

    iFrameBytes = head_info3(
        pvData as *mut c_uchar,
        (iDataLen / 2) as c_uint,
        &mut head,
        &mut iBitRate,
        &mut iRealDataStart,
    );
    if iFrameBytes == 0 {
        return cstr(MP3ERR_Bad_or_unsupported_file);
    }

    if head.mode != 3 && bStereoDesired == 0 {
        if iDataLen > 98000 {
            return cstr(MP3ERR_Sound_file_is_stereo);
        }
    }
    if (audio.decode_init)(
        &mut head,
        iFrameBytes,
        reduction_code,
        iRealDataStart as c_int,
        if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono },
        freq_limit,
    ) != 0
    {
        if bStereoDesired != 0 {
            if (*pMP3Stream).outbytes > 4608 {
                return cstr(MP3ERR_Source_file_has_output_packet_size_stereo);
            }
        } else {
            if (*pMP3Stream).outbytes > 2304 {
                return cstr(MP3ERR_Source_file_has_output_packet_size);
            }
        }

        (audio.decode_info)(&mut decinfo);

        if decinfo.bits != 16 {
            return cstr(MP3ERR_Source_file_is_not_16bit);
        }

        if decinfo.samprate != 44100 {
            return cstr(MP3ERR_Source_file_is_not_sampled_44100);
        }

        if bStereoDesired != 0 && decinfo.channels != 2 {
            return cstr(MP3ERR_Source_file_is_not_stereo);
        }
    } else {
        return cstr(MP3ERR_Decoder_failed_to_initialise);
    }

    null_mut()
}

// char *return is NZ for any errors (no trailing CR!)
#[no_mangle]
pub unsafe extern "C" fn C_MP3_GetHeaderData(
    pvData: *mut c_void,
    iDataLen: c_int,
    piRate: *mut c_int,
    piWidth: *mut c_int,
    piChannels: *mut c_int,
    bStereoDesired: c_int,
) -> *mut c_char {
    let mut iRealDataStart: c_uint = 0;
    let mut head: MPEG_HEAD = core::mem::zeroed();
    let mut decinfo: DEC_INFO = core::mem::zeroed();
    let mut iBitRate: c_int = 0;
    let iFrameBytes: c_int;

    core::ptr::write_bytes(pMP3Stream, 0, 1);

    iFrameBytes = head_info3(
        pvData as *mut c_uchar,
        (iDataLen / 2) as c_uint,
        &mut head,
        &mut iBitRate,
        &mut iRealDataStart,
    );
    if iFrameBytes == 0 {
        return cstr(MP3ERR_Bad_or_unsupported_file);
    }

    if (audio.decode_init)(
        &mut head,
        iFrameBytes,
        reduction_code,
        iRealDataStart as c_int,
        if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono },
        freq_limit,
    ) != 0
    {
        (audio.decode_info)(&mut decinfo);

        *piRate = decinfo.samprate as c_int;
        *piWidth = decinfo.bits / 8;
        *piChannels = decinfo.channels;
    } else {
        return cstr(MP3ERR_Decoder_failed_to_initialise);
    }

    null_mut()
}

// char *return is NZ for any errors (no trailing CR!)
#[no_mangle]
pub unsafe extern "C" fn C_MP3_GetUnpackedSize(
    pvData: *mut c_void,
    mut iSourceBytesRemaining: c_int,
    piUnpackedSize: *mut c_int,
    bStereoDesired: c_int,
) -> *mut c_char {
    let iReadLimit: c_int;
    let mut iRealDataStart: c_uint = 0;
    let mut head: MPEG_HEAD = core::mem::zeroed();
    let mut iBitRate: c_int = 0;

    let pPCM_Buffer: *mut c_char = PCM_Buffer.as_mut_ptr();
    let mut psReturn: *mut c_char = null_mut();
    let mut iDestWriteIndex: c_int = 0;

    let iFrameBytes: c_int;
    let mut iFrameCounter: c_int;

    let mut decinfo: DEC_INFO = core::mem::zeroed();
    let mut x: IN_OUT;

    core::ptr::write_bytes(pMP3Stream, 0, 1);

    iFrameBytes = head_info3(
        pvData as *mut c_uchar,
        (iSourceBytesRemaining / 2) as c_uint,
        &mut head,
        &mut iBitRate,
        &mut iRealDataStart,
    );

    BYTESREMAINING_ACCOUNT_FOR_REAR_TAG(pvData, &mut iSourceBytesRemaining);
    iSourceBytesRemaining -= iRealDataStart as c_int;

    iReadLimit = iRealDataStart as c_int + iSourceBytesRemaining;

    if iFrameBytes != 0 {
        if (audio.decode_init)(
            &mut head,
            iFrameBytes,
            reduction_code,
            iRealDataStart as c_int,
            if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono },
            freq_limit,
        ) != 0
        {
            (audio.decode_info)(&mut decinfo);

            iFrameCounter = 0;
            loop {
                if iSourceBytesRemaining == 0 || iSourceBytesRemaining < iFrameBytes {
                    break;
                }

                bFastEstimateOnly = 1;

                x = (audio.decode)(
                    (pvData as *mut c_uchar).offset(iRealDataStart as isize),
                    pPCM_Buffer as *mut c_short,
                    (pvData as *mut c_uchar).offset(iReadLimit as isize),
                );

                bFastEstimateOnly = 0;

                iRealDataStart = iRealDataStart.wrapping_add(x.in_bytes as c_uint);
                iSourceBytesRemaining -= x.in_bytes;
                iDestWriteIndex += x.out_bytes;

                if x.in_bytes <= 0 {
                    break;
                }

                iFrameCounter += 1;
            }

            *piUnpackedSize = iDestWriteIndex;
        } else {
            psReturn = cstr(MP3ERR_Decoder_failed_to_initialise);
        }
    } else {
        psReturn = cstr(MP3ERR_Bad_or_Unsupported_MP3_file);
    }

    psReturn
}

#[no_mangle]
pub unsafe extern "C" fn C_MP3_UnpackRawPCM(
    pvData: *mut c_void,
    mut iSourceBytesRemaining: c_int,
    piUnpackedSize: *mut c_int,
    pbUnpackBuffer: *mut c_void,
    bStereoDesired: c_int,
) -> *mut c_char {
    let iReadLimit: c_int;
    let mut iRealDataStart: c_uint = 0;
    let mut head: MPEG_HEAD = core::mem::zeroed();
    let mut iBitRate: c_int = 0;

    let mut psReturn: *mut c_char = null_mut();
    let mut iDestWriteIndex: c_int = 0;

    let iFrameBytes: c_int;
    let mut iFrameCounter: c_int;

    let mut decinfo: DEC_INFO = core::mem::zeroed();
    let mut x: IN_OUT;

    core::ptr::write_bytes(pMP3Stream, 0, 1);

    iFrameBytes = head_info3(
        pvData as *mut c_uchar,
        (iSourceBytesRemaining / 2) as c_uint,
        &mut head,
        &mut iBitRate,
        &mut iRealDataStart,
    );

    BYTESREMAINING_ACCOUNT_FOR_REAR_TAG(pvData, &mut iSourceBytesRemaining);
    iSourceBytesRemaining -= iRealDataStart as c_int;

    iReadLimit = iRealDataStart as c_int + iSourceBytesRemaining;

    if iFrameBytes != 0 {
        if (audio.decode_init)(
            &mut head,
            iFrameBytes,
            reduction_code,
            iRealDataStart as c_int,
            if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono },
            freq_limit,
        ) != 0
        {
            (audio.decode_info)(&mut decinfo);

            iFrameCounter = 0;
            loop {
                if iSourceBytesRemaining == 0 || iSourceBytesRemaining < iFrameBytes {
                    break;
                }

                x = (audio.decode)(
                    (pvData as *mut c_uchar).offset(iRealDataStart as isize),
                    (pbUnpackBuffer as *mut c_char).offset(iDestWriteIndex as isize) as *mut c_short,
                    (pvData as *mut c_uchar).offset(iReadLimit as isize),
                );

                iRealDataStart = iRealDataStart.wrapping_add(x.in_bytes as c_uint);
                iSourceBytesRemaining -= x.in_bytes;
                iDestWriteIndex += x.out_bytes;

                if x.in_bytes <= 0 {
                    break;
                }

                iFrameCounter += 1;
            }

            *piUnpackedSize = iDestWriteIndex;
        } else {
            psReturn = cstr(MP3ERR_Decoder_failed_to_initialise);
        }
    } else {
        psReturn = cstr(MP3ERR_Bad_or_Unsupported_MP3_file);
    }

    psReturn
}

// char * return is NULL for ok, else error string
#[no_mangle]
pub unsafe extern "C" fn C_MP3Stream_DecodeInit(
    pSFX_MP3Stream: LP_MP3STREAM,
    pvSourceData: *mut c_void,
    mut iSourceBytesRemaining: c_int,
    iGameAudioSampleRate: c_int,
    iGameAudioSampleBits: c_int,
    bStereoDesired: c_int,
) -> *mut c_char {
    let mut psReturn: *mut c_char = null_mut();
    let mut head: MPEG_HEAD = core::mem::zeroed();
    let mut decinfo: DEC_INFO = core::mem::zeroed();
    let mut iBitRate: c_int = 0;

    pMP3Stream = pSFX_MP3Stream;

    core::ptr::write_bytes(pMP3Stream, 0, 1);

    (*pMP3Stream).pbSourceData = pvSourceData as *mut byte;
    (*pMP3Stream).iSourceBytesRemaining = iSourceBytesRemaining;
    (*pMP3Stream).iSourceFrameBytes = head_info3(
        pvSourceData as *mut byte,
        (iSourceBytesRemaining / 2) as c_uint,
        &mut head,
        &mut iBitRate,
        addr_of_mut!((*pMP3Stream).iSourceReadIndex) as *mut c_uint,
    );

    if bStereoDesired == 0 {
        BYTESREMAINING_ACCOUNT_FOR_REAR_TAG(pvSourceData, addr_of_mut!((*pMP3Stream).iSourceBytesRemaining));
        (*pMP3Stream).iSourceBytesRemaining -= (*pMP3Stream).iSourceReadIndex;
    }

    (*pMP3Stream).iRewind_SourceReadIndex = (*pMP3Stream).iSourceReadIndex;
    (*pMP3Stream).iRewind_SourceBytesRemaining = (*pMP3Stream).iSourceBytesRemaining;

    debug_assert!((*pMP3Stream).iSourceFrameBytes != 0);
    if (*pMP3Stream).iSourceFrameBytes != 0 {
        if (audio.decode_init)(
            &mut head,
            (*pMP3Stream).iSourceFrameBytes,
            reduction_code,
            (*pMP3Stream).iSourceReadIndex,
            if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono },
            freq_limit,
        ) != 0
        {
            (*pMP3Stream).iRewind_FinalReductionCode = reduction_code;

            (*pMP3Stream).iRewind_FinalConvertCode =
                if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono };

            (audio.decode_info)(&mut decinfo);

            if iGameAudioSampleRate == (decinfo.samprate >> 1) as c_int {
                (*pMP3Stream).iRewind_FinalReductionCode = 1;
            } else if iGameAudioSampleRate == (decinfo.samprate >> 2) as c_int {
                (*pMP3Stream).iRewind_FinalReductionCode = 2;
            }

            if iGameAudioSampleBits == decinfo.bits >> 1 {
                (*pMP3Stream).iRewind_FinalConvertCode |= 8;
            }

            if (audio.decode_init)(
                &mut head,
                (*pMP3Stream).iSourceFrameBytes,
                (*pMP3Stream).iRewind_FinalReductionCode,
                (*pMP3Stream).iSourceReadIndex,
                (*pMP3Stream).iRewind_FinalConvertCode,
                freq_limit,
            ) != 0
            {
                (audio.decode_info)(&mut decinfo);
                debug_assert!(iGameAudioSampleRate == decinfo.samprate as c_int);
                debug_assert!(iGameAudioSampleBits == decinfo.bits);

                if iGameAudioSampleRate != decinfo.samprate as c_int
                    || iGameAudioSampleBits != decinfo.bits
                {
                    psReturn = cstr(MP3ERR_Decoder_unable_to_convert);
                }
            } else {
                psReturn = cstr(MP3ERR_Decoder_failed_pass_2);
            }
        } else {
            psReturn = cstr(MP3ERR_Decoder_failed_to_initialise);
        }
    } else {
        psReturn = cstr(MP3ERR_Something_broken);
    }

    pMP3Stream = addr_of_mut!(_MP3Stream);

    psReturn
}

// return value is decoded bytes for this packet, which is effectively a BOOL, NZ for not finished decoding yet...
#[no_mangle]
pub unsafe extern "C" fn C_MP3Stream_Decode(pSFX_MP3Stream: LP_MP3STREAM) -> c_uint {
    let mut uiDecoded: c_uint = 0;
    let x: IN_OUT;

    pMP3Stream = pSFX_MP3Stream;

    loop {
        if (*pSFX_MP3Stream).iSourceBytesRemaining == 0 {
            uiDecoded = 0;
            break;
        }

        x = (audio.decode)(
            (*pSFX_MP3Stream)
                .pbSourceData
                .offset((*pSFX_MP3Stream).iSourceReadIndex as isize),
            (*pSFX_MP3Stream).bDecodeBuffer.as_mut_ptr() as *mut c_short,
            (*pSFX_MP3Stream).pbSourceData.offset(
                ((*pSFX_MP3Stream).iRewind_SourceReadIndex
                    + (*pSFX_MP3Stream).iRewind_SourceBytesRemaining) as isize,
            ),
        );

        #[cfg(debug_assertions)]
        {
            (*pSFX_MP3Stream).iSourceFrameCounter += 1;
        }

        (*pSFX_MP3Stream).iSourceReadIndex += x.in_bytes;
        (*pSFX_MP3Stream).iSourceBytesRemaining -= x.in_bytes;
        (*pSFX_MP3Stream).iBytesDecodedTotal += x.out_bytes;
        (*pSFX_MP3Stream).iBytesDecodedThisPacket = x.out_bytes;

        uiDecoded = x.out_bytes as c_uint;

        if x.in_bytes <= 0 {
            uiDecoded = 0;
            break;
        }

        break;
    }

    pMP3Stream = addr_of_mut!(_MP3Stream);

    uiDecoded
}

// ret is char* errstring, else NULL for ok
#[no_mangle]
pub unsafe extern "C" fn C_MP3Stream_Rewind(pSFX_MP3Stream: LP_MP3STREAM) -> *mut c_char {
    let mut psReturn: *mut c_char = null_mut();
    let mut head: MPEG_HEAD = core::mem::zeroed();
    let mut iBitRate: c_int = 0;
    let mut iNULL: c_int = 0;

    pMP3Stream = pSFX_MP3Stream;

    (*pMP3Stream).iSourceReadIndex = (*pMP3Stream).iRewind_SourceReadIndex;
    (*pMP3Stream).iSourceBytesRemaining = (*pMP3Stream).iRewind_SourceBytesRemaining;

    if (*pMP3Stream).iSourceFrameBytes
        == head_info3(
            (*pMP3Stream).pbSourceData,
            ((*pMP3Stream).iSourceBytesRemaining / 2) as c_uint,
            &mut head,
            &mut iBitRate,
            &mut iNULL as *mut c_int as *mut c_uint,
        )
    {
        if (audio.decode_init)(
            &mut head,
            (*pMP3Stream).iSourceFrameBytes,
            (*pMP3Stream).iRewind_FinalReductionCode,
            (*pMP3Stream).iSourceReadIndex,
            (*pMP3Stream).iRewind_FinalConvertCode,
            freq_limit,
        ) != 0
        {
        } else {
            psReturn = cstr(MP3ERR_Failed_reinit_rewind);
        }
    } else {
        psReturn = cstr(MP3ERR_Frame_bytes_mismatch);
    }

    pMP3Stream = addr_of_mut!(_MP3Stream);

    psReturn
}
