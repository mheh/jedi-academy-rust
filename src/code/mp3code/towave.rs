/*____________________________________________________________________________

	FreeAmp - The Free MP3 Player

        MP3 Decoder originally Copyright (C) 1995-1997 Xing Technology
        Corp.  http://www.xingtech.com

	Portions Copyright (C) 1998-1999 EMusic.com

	This program is free software; you can redistribute it and/or modify
	it under the terms of the GNU General Public License as published by
	the Free Software Foundation; either version 2 of the License, or
	(at your option) any later version.

	This program is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU General Public License for more details.

	You should have received a copy of the GNU General Public License
	along with this program; if not, write to the Free Software
	Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

	$Id: towave.c,v 1.3 1999/10/19 07:13:09 elrod Exp $
____________________________________________________________________________*/

/* ------------------------------------------------------------------------

      NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE

        This file exists for reference only. It is not actually used
        in the FreeAmp project. There is no need to mess with this
        file. There is no need to flatten the beavers, either.

      NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE NOTE

/*---- towave.c --------------------------------------------
  32 bit version only

decode mpeg Layer I/II/III file using portable ANSI C decoder,
output to pcm wave file.

mod 8/19/98 decode 22 sf bands

mod 5/14/98  allow mpeg25 (dec8 not supported for mpeg25 samp rate)

mod 3/4/98 bs_trigger  bs_bufbytes  made signed, unsigned may
            not terminate properly.  Also extra test in bs_fill.

mod 8/6/96 add 8 bit output to standard decoder

ver 1.4 mods 7/18/96 32 bit and add asm option

mods 6/29/95  allow MS wave file for u-law.  bugfix u-law table dec8.c

mods 2/95 add sample rate reduction, freq_limit and conversions.
          add _decode8 for 8Ks output, 16bit 8bit, u-law output.
          add additional control parameters to init.
          add _info function

mod 5/12/95 add quick window cwinq.c

mod 5/19/95 change from stream io to handle io

mod 11/16/95 add Layer I

mod 1/5/95   integer overflow mod iup.c

ver 1.3
mod 2/5/96   portability mods
             drop Tom and Gloria pcm file types

ver 2.0
mod 1/7/97   Layer 3 (float mpeg-1 only)
    2/6/97   Layer 3 MPEG-2

ver 3.01     Layer III bugfix crc problem 8/18/97
ver 3.02     Layer III fix wannabe.mp3 problem 10/9/97
ver 3.03     allow mpeg 2.5  5/14/98

Decoder functions for _decode8 are defined in dec8.c.  Useage
is same as regular decoder.

Towave illustrates use of decoder.  Towave converts
mpeg audio files to 16 bit (short) pcm.  Default pcm file
format is wave. Other formats can be accommodated by
adding alternative write_pcm_header and write_pcm_tailer
functions.  The functions kbhit and getch used in towave.c
may not port to other systems.

The decoder handles all mpeg1 and mpeg2 Layer I/II  bitstreams.

For compatability with the asm decoder and future C versions,
source code users are discouraged from making modifications
to the decoder proper.  MS Windows applications can use wrapper
functions in a separate module if decoder functions need to
be exported.

NOTE additional control parameters.

mod 8/6/96 standard decoder adds 8 bit output

decode8 (8Ks output) convert_code:
   convert_code = 4*bit_code + chan_code
       bit_code:   1 = 16 bit linear pcm
                   2 =  8 bit (unsigned) linear pcm
                   3 = u-law (8 bits unsigned)
       chan_code:  0 = convert two chan to mono
                   1 = convert two chan to mono
                   2 = convert two chan to left chan
                   3 = convert two chan to right chan

decode (standard decoder) convert_code:
             0 = two chan output
             1 = convert two chan to mono
             2 = convert two chan to left chan
             3 = convert two chan to right chan
     or with 8 = 8 bit output
          (other bits ignored)

decode (standard decoder) reduction_code:
             0 = full sample rate output
             1 = half rate
             2 = quarter rate

-----------------------------------------------------------*/

use core::ffi::{c_int, c_char, c_void};
use core::ptr::addr_of_mut;
use core::mem::size_of;

// Local stub types for external dependencies
// These are declared but not fully defined in this module
#[repr(C)]
pub struct MPEG_HEAD {
    pub mode: c_int,
    // ... other fields not specified
}

#[repr(C)]
pub struct DEC_INFO {
    pub bits: c_int,
    pub samprate: c_int,
    pub channels: c_int,
    pub type_: c_int,
    // ... other fields
}

#[repr(C)]
pub struct IN_OUT {
    pub in_bytes: c_int,
    pub out_bytes: c_int,
}

#[repr(C)]
pub struct MP3STREAM {
    pub pbSourceData: *mut u8,
    pub iSourceBytesRemaining: c_int,
    pub iSourceFrameBytes: c_int,
    pub iSourceReadIndex: c_int,
    pub iRewind_SourceReadIndex: c_int,
    pub iRewind_SourceBytesRemaining: c_int,
    pub bDecodeBuffer: [u8; 4608],  // Based on max output packet size
    pub iRewind_FinalReductionCode: c_int,
    pub iRewind_FinalConvertCode: c_int,
    pub iBytesDecodedTotal: c_int,
    pub iBytesDecodedThisPacket: c_int,
    #[cfg(debug_assertions)]
    pub iSourceFrameCounter: c_int,
}

pub type LP_MP3STREAM = *mut MP3STREAM;

// Function pointer types
pub type DecodeInitFn = extern "C" fn(*mut MPEG_HEAD, c_int, c_int, c_int, c_int, c_int) -> c_int;
pub type DecodeInfoFn = extern "C" fn(*mut DEC_INFO);
pub type DecodeFn = extern "C" fn(*mut u8, *mut i16, *mut u8) -> IN_OUT;

#[repr(C)]
pub struct AUDIO {
    pub decode_init: Option<DecodeInitFn>,
    pub decode_info: Option<DecodeInfoFn>,
    pub decode: Option<DecodeFn>,
}

// External decoder functions
extern "C" {
    pub fn audio_decode_init(h: *mut MPEG_HEAD, framebytes_arg: c_int,
                             reduction_code: c_int, transform_code: c_int,
                             convert_code: c_int, freq_limit: c_int) -> c_int;
    pub fn audio_decode_info(info: *mut DEC_INFO);
    pub fn audio_decode(bs: *mut u8, pcm: *mut i16, pNextByteAfterData: *mut u8) -> IN_OUT;

    pub fn head_info3(pvData: *mut c_void, iDataLen: c_int, head: *mut MPEG_HEAD,
                      iBitRate: *mut c_int, iRealDataStart: *mut c_int) -> c_int;
}

#[repr(C)]
pub struct id3v1_1 {
    pub id: [c_char; 3],
    pub title: [c_char; 30],      // <file basename>
    pub artist: [c_char; 30],     // "Raven Software"
    pub album: [c_char; 30],      // "#UNCOMP %d"		// needed
    pub year: [c_char; 4],        // "2000"
    pub comment: [c_char; 28],    // "#MAXVOL %g"		// needed
    pub zero: c_char,
    pub track: c_char,
    pub genre: c_char,
}

pub static mut gpTAG: *mut id3v1_1 = 0 as *mut id3v1_1;

// account for trailing ID3 tag in _iBytesRemaining
// This macro is translated into an inline function that modifies iSourceBytesRemaining
#[inline]
unsafe fn BYTESREMAINING_ACCOUNT_FOR_REAR_TAG(pvData: *mut c_void, iSourceBytesRemaining: &mut c_int) {
    // sizeof(id3v1_1) = 128
    gpTAG = ((*pvData as *mut u8).add(*iSourceBytesRemaining as usize - 128) as *mut id3v1_1);
    if core::ffi::CStr::from_ptr((*gpTAG).id.as_ptr()).to_bytes().starts_with(b"TAG") {
        *iSourceBytesRemaining -= 128;
    }
}

/********  pcm buffer ********/

const PCM_BUFBYTES: usize = 60000;  // more than enough to cover the largest that one packet will ever expand to
pub static mut PCM_Buffer: [u8; PCM_BUFBYTES] = [0u8; PCM_BUFBYTES];  // better off being declared, so we don't do mallocs in this module (MAC reasons)

#[allow(non_upper_case_globals)]
static audio_table: [[AUDIO; 2]; 2] = [
    [
        AUDIO { decode_init: Some(audio_decode_init), decode_info: Some(audio_decode_info), decode: Some(audio_decode) },
        AUDIO { decode_init: Some(audio_decode_init), decode_info: Some(audio_decode_info), decode: Some(audio_decode) },
    ],
    [
        AUDIO { decode_init: Some(audio_decode_init), decode_info: Some(audio_decode_info), decode: Some(audio_decode) },
        AUDIO { decode_init: Some(audio_decode_init), decode_info: Some(audio_decode_info), decode: Some(audio_decode) },
    ]
];

#[allow(non_upper_case_globals)]
static audio: AUDIO = AUDIO { decode_init: Some(audio_decode_init), decode_info: Some(audio_decode_info), decode: Some(audio_decode) };

// Do NOT change these, ever!!!!!!!!!!!!!!!!!!
//
pub const reduction_code: c_int = 0;           // unpack at full sample rate output
pub const convert_code_mono: c_int = 1;
pub const convert_code_stereo: c_int = 0;
pub const freq_limit: c_int = 24000;           // no idea what this is about, but it's always this value so...

// the entire decode mechanism uses this now...
//
pub static mut _MP3Stream: MP3STREAM = MP3STREAM {
    pbSourceData: core::ptr::null_mut(),
    iSourceBytesRemaining: 0,
    iSourceFrameBytes: 0,
    iSourceReadIndex: 0,
    iRewind_SourceReadIndex: 0,
    iRewind_SourceBytesRemaining: 0,
    bDecodeBuffer: [0u8; 4608],
    iRewind_FinalReductionCode: 0,
    iRewind_FinalConvertCode: 0,
    iBytesDecodedTotal: 0,
    iBytesDecodedThisPacket: 0,
    #[cfg(debug_assertions)]
    iSourceFrameCounter: 0,
};

pub static mut pMP3Stream: *mut MP3STREAM = unsafe { addr_of_mut!(_MP3Stream) };
pub static mut bFastEstimateOnly: c_int = 0;   // MUST DEFAULT TO THIS VALUE!!!!!!!!!


// char *return is NZ for any errors (no trailing CR!)
//
#[no_mangle]
pub extern "C" fn C_MP3_IsValid(pvData: *mut c_void, iDataLen: c_int, bStereoDesired: c_int) -> *const c_char {
    unsafe {
        let mut iRealDataStart: c_int = 0;
        let mut head: MPEG_HEAD = core::mem::zeroed();
        let mut decinfo: DEC_INFO = core::mem::zeroed();
        let mut iBitRate: c_int = 0;
        let iFrameBytes: c_int;

        core::ptr::write_bytes(pMP3Stream as *mut u8, 0, size_of::<MP3STREAM>());

        iFrameBytes = head_info3(pvData, iDataLen / 2, &mut head, &mut iBitRate, &mut iRealDataStart);
        if iFrameBytes == 0 {
            return b"MP3ERR: Bad or unsupported file!\0".as_ptr() as *const c_char;
        }

        // check for files with bad frame unpack sizes (that would crash the game), or stereo files.
        //
        // although the decoder can convert stereo to mono (apparently), we want to know about stereo files
        //	because they're a waste of source space... (all FX are mono, and moved via panning)
        //
        if head.mode != 3 && bStereoDesired == 0 && iDataLen > 98000 {
            // 3 seems to mean mono
            // we'll allow it for small files even if stereo
            if iDataLen != 1050024 {
                // fixme, make cinematic_1 play as music instead
                return b"MP3ERR: Sound file is stereo!\0".as_ptr() as *const c_char;
            }
        }

        if audio.decode_init.map_or(false, |f| f(&mut head, iFrameBytes, reduction_code, iRealDataStart, if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono }, freq_limit) != 0) {
            if bStereoDesired != 0 {
                if (*pMP3Stream).iSourceBytesRemaining > 4608 {
                    return b"MP3ERR: Source file has output packet size > 2304 (*2 for stereo) bytes!\0".as_ptr() as *const c_char;
                }
            } else {
                if (*pMP3Stream).iSourceBytesRemaining > 2304 {
                    return b"MP3ERR: Source file has output packet size > 2304 bytes!\0".as_ptr() as *const c_char;
                }
            }

            if let Some(decode_info_fn) = audio.decode_info {
                decode_info_fn(&mut decinfo);
            }

            if decinfo.bits != 16 {
                return b"MP3ERR: Source file is not 16bit!\0".as_ptr() as *const c_char;
                // will this ever happen? oh well...
            }

            if decinfo.samprate != 44100 {
                return b"MP3ERR: Source file is not sampled @ 44100!\0".as_ptr() as *const c_char;
            }
            if bStereoDesired != 0 && decinfo.channels != 2 {
                return b"MP3ERR: Source file is not stereo!\0".as_ptr() as *const c_char;
                // sod it, I'm going to count this as an error now
            }
        } else {
            return b"MP3ERR: Decoder failed to initialise\0".as_ptr() as *const c_char;
        }

        // file seems to be valid...
        //
        core::ptr::null()
    }
}



// char *return is NZ for any errors (no trailing CR!)
//
#[no_mangle]
pub extern "C" fn C_MP3_GetHeaderData(pvData: *mut c_void, iDataLen: c_int, piRate: *mut c_int, piWidth: *mut c_int, piChannels: *mut c_int, bStereoDesired: c_int) -> *const c_char {
    unsafe {
        let mut iRealDataStart: c_int = 0;
        let mut head: MPEG_HEAD = core::mem::zeroed();
        let mut decinfo: DEC_INFO = core::mem::zeroed();
        let mut iBitRate: c_int = 0;
        let iFrameBytes: c_int;

        core::ptr::write_bytes(pMP3Stream as *mut u8, 0, size_of::<MP3STREAM>());

        iFrameBytes = head_info3(pvData, iDataLen / 2, &mut head, &mut iBitRate, &mut iRealDataStart);
        if iFrameBytes == 0 {
            return b"MP3ERR: Bad or unsupported file!\0".as_ptr() as *const c_char;
        }

        if audio.decode_init.map_or(false, |f| f(&mut head, iFrameBytes, reduction_code, iRealDataStart, if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono }, freq_limit) != 0) {
            if let Some(decode_info_fn) = audio.decode_info {
                decode_info_fn(&mut decinfo);
            }

            *piRate = decinfo.samprate;     // rate (eg 22050, 44100 etc)
            *piWidth = decinfo.bits / 8;   // 1 for 8bit, 2 for 16 bit
            *piChannels = decinfo.channels; // 1 for mono, 2 for stereo
        } else {
            return b"MP3ERR: Decoder failed to initialise\0".as_ptr() as *const c_char;
        }

        // everything ok...
        //
        core::ptr::null()
    }
}




// this duplicates work done in C_MP3_IsValid(), but it avoids global structs, and means that you can call this anytime
//	if you just want info for some reason
//
// ( size is now workd out just by decompressing each packet header, not the whole stream. MUCH faster :-)
//
// char *return is NZ for any errors (no trailing CR!)
//
#[no_mangle]
pub extern "C" fn C_MP3_GetUnpackedSize(pvData: *mut c_void, mut iSourceBytesRemaining: c_int, piUnpackedSize: *mut c_int, bStereoDesired: c_int) -> *const c_char {
    unsafe {
        let mut iReadLimit: c_int;
        let mut iRealDataStart: c_int = 0;
        let mut head: MPEG_HEAD = core::mem::zeroed();
        let mut iBitRate: c_int = 0;

        let pPCM_Buffer: *mut u8 = addr_of_mut!(PCM_Buffer) as *mut u8;
        let mut psReturn: *const c_char = core::ptr::null();
        let mut iDestWriteIndex: c_int = 0;

        let iFrameBytes: c_int;
        let mut iFrameCounter: c_int;

        let mut decinfo: DEC_INFO = core::mem::zeroed();
        let mut x: IN_OUT;

        core::ptr::write_bytes(pMP3Stream as *mut u8, 0, size_of::<MP3STREAM>());

        let iSourceReadIndex: *mut c_int = &mut iRealDataStart;

        iFrameBytes = head_info3(pvData, iSourceBytesRemaining / 2, &mut head, &mut iBitRate, &mut iRealDataStart);

        BYTESREMAINING_ACCOUNT_FOR_REAR_TAG(pvData, &mut iSourceBytesRemaining);
        iSourceBytesRemaining -= iRealDataStart;

        iReadLimit = *iSourceReadIndex + iSourceBytesRemaining;

        if iFrameBytes != 0 {
            if audio.decode_init.map_or(false, |f| f(&mut head, iFrameBytes, reduction_code, iRealDataStart, if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono }, freq_limit) != 0) {
                if let Some(decode_info_fn) = audio.decode_info {
                    decode_info_fn(&mut decinfo);
                }

                // decode...
                //
                iFrameCounter = 0;
                loop {
                    if iSourceBytesRemaining == 0 || iSourceBytesRemaining < iFrameBytes {
                        break; // end of file
                    }

                    bFastEstimateOnly = 1;

                    if let Some(decode_fn) = audio.decode {
                        x = decode_fn((pvData as *mut u8).add(*iSourceReadIndex as usize), pPCM_Buffer as *mut i16,
                                      (pvData as *mut u8).add(iReadLimit as usize));

                        bFastEstimateOnly = 0;

                        *iSourceReadIndex += x.in_bytes;
                        iSourceBytesRemaining -= x.in_bytes;
                        iDestWriteIndex += x.out_bytes;

                        if x.in_bytes <= 0 {
                            break;
                        }
                    }

                    iFrameCounter += 1;
                }

                *piUnpackedSize = iDestWriteIndex; // yeeehaaa!
            } else {
                psReturn = b"MP3ERR: Decoder failed to initialise\0".as_ptr() as *const c_char;
            }
        } else {
            psReturn = b"MP3ERR: Bad or Unsupported MP3 file!\0".as_ptr() as *const c_char;
        }

        psReturn
    }
}




#[no_mangle]
pub extern "C" fn C_MP3_UnpackRawPCM(pvData: *mut c_void, mut iSourceBytesRemaining: c_int, piUnpackedSize: *mut c_int, pbUnpackBuffer: *mut c_void, bStereoDesired: c_int) -> *const c_char {
    unsafe {
        let mut iReadLimit: c_int;
        let mut iRealDataStart: c_int = 0;
        let mut head: MPEG_HEAD = core::mem::zeroed();
        let mut iBitRate: c_int = 0;

        let mut psReturn: *const c_char = core::ptr::null();
        let mut iDestWriteIndex: c_int = 0;

        let iFrameBytes: c_int;
        let mut iFrameCounter: c_int;

        let mut decinfo: DEC_INFO = core::mem::zeroed();
        let mut x: IN_OUT;

        core::ptr::write_bytes(pMP3Stream as *mut u8, 0, size_of::<MP3STREAM>());

        let iSourceReadIndex: *mut c_int = &mut iRealDataStart;

        iFrameBytes = head_info3(pvData, iSourceBytesRemaining / 2, &mut head, &mut iBitRate, &mut iRealDataStart);

        BYTESREMAINING_ACCOUNT_FOR_REAR_TAG(pvData, &mut iSourceBytesRemaining);
        iSourceBytesRemaining -= iRealDataStart;

        iReadLimit = *iSourceReadIndex + iSourceBytesRemaining;

        if iFrameBytes != 0 {
            if audio.decode_init.map_or(false, |f| f(&mut head, iFrameBytes, reduction_code, iRealDataStart, if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono }, freq_limit) != 0) {
                if let Some(decode_info_fn) = audio.decode_info {
                    decode_info_fn(&mut decinfo);
                }

                // decode...
                //
                iFrameCounter = 0;
                loop {
                    if iSourceBytesRemaining == 0 || iSourceBytesRemaining < iFrameBytes {
                        break; // end of file
                    }

                    if let Some(decode_fn) = audio.decode {
                        x = decode_fn((pvData as *mut u8).add(*iSourceReadIndex as usize), (pbUnpackBuffer as *mut u8).add(iDestWriteIndex as usize) as *mut i16,
                                      (pvData as *mut u8).add(iReadLimit as usize));

                        *iSourceReadIndex += x.in_bytes;
                        iSourceBytesRemaining -= x.in_bytes;
                        iDestWriteIndex += x.out_bytes;

                        if x.in_bytes <= 0 {
                            break;
                        }
                    }

                    iFrameCounter += 1;
                }

                *piUnpackedSize = iDestWriteIndex; // yeeehaaa!
            } else {
                psReturn = b"MP3ERR: Decoder failed to initialise\0".as_ptr() as *const c_char;
            }
        } else {
            psReturn = b"MP3ERR: Bad or Unsupported MP3 file!\0".as_ptr() as *const c_char;
        }

        psReturn
    }
}


// called once, after we've decided to keep something as MP3. This just sets up the decoder for subsequent stream-calls.
//
// (the struct pSFX_MP3Stream is cleared internally, so pass as args anything you want stored in it)
//
// char * return is NULL for ok, else error string
//
#[no_mangle]
pub extern "C" fn C_MP3Stream_DecodeInit(pSFX_MP3Stream: LP_MP3STREAM, pvSourceData: *mut c_void, mut iSourceBytesRemaining: c_int,
                                          iGameAudioSampleRate: c_int, iGameAudioSampleBits: c_int, bStereoDesired: c_int) -> *const c_char {
    unsafe {
        let mut psReturn: *const c_char = core::ptr::null();
        let mut head: MPEG_HEAD = core::mem::zeroed();         // only relevant within this function during init
        let mut decinfo: DEC_INFO = core::mem::zeroed();       //   " "
        let mut iBitRate: c_int = 0;                           // not used after being filled in by head_info3()

        pMP3Stream = pSFX_MP3Stream;

        core::ptr::write_bytes(pMP3Stream as *mut u8, 0, size_of::<MP3STREAM>());

        (*pMP3Stream).pbSourceData = pvSourceData as *mut u8;
        (*pMP3Stream).iSourceBytesRemaining = iSourceBytesRemaining;
        (*pMP3Stream).iSourceFrameBytes = head_info3(pvSourceData, iSourceBytesRemaining / 2, &mut head, &mut iBitRate, &mut (*pMP3Stream).iSourceReadIndex);

        // hack, do NOT do this for stereo, since music files are now streamed and therefore the data isn't actually fully
        //	loaded at this point, only about 4k or so for the header is actually in memory!!!...
        //
        if bStereoDesired == 0 {
            BYTESREMAINING_ACCOUNT_FOR_REAR_TAG(pvSourceData, &mut (*pMP3Stream).iSourceBytesRemaining);
            (*pMP3Stream).iSourceBytesRemaining -= (*pMP3Stream).iSourceReadIndex;
        }

        // backup a couple of fields so we can play this again later...
        //
        (*pMP3Stream).iRewind_SourceReadIndex = (*pMP3Stream).iSourceReadIndex;
        (*pMP3Stream).iRewind_SourceBytesRemaining = (*pMP3Stream).iSourceBytesRemaining;

        assert!((*pMP3Stream).iSourceFrameBytes != 0);
        if (*pMP3Stream).iSourceFrameBytes != 0 {
            if audio.decode_init.map_or(false, |f| f(&mut head, (*pMP3Stream).iSourceFrameBytes, reduction_code, (*pMP3Stream).iSourceReadIndex, if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono }, freq_limit) != 0) {
                (*pMP3Stream).iRewind_FinalReductionCode = reduction_code; // default = 0 (no reduction), 1=half, 2 = quarter

                (*pMP3Stream).iRewind_FinalConvertCode = if bStereoDesired != 0 { convert_code_stereo } else { convert_code_mono };
                                                         // default = 1 (mono), OR with 8 for 8-bit output

                // only now can we ask what kind of properties this file has, and then adjust to fit what the game wants...
                //
                if let Some(decode_info_fn) = audio.decode_info {
                    decode_info_fn(&mut decinfo);
                }

                // decoder offers half or quarter rate adjustement only...
                //
                if iGameAudioSampleRate == decinfo.samprate >> 1 {
                    (*pMP3Stream).iRewind_FinalReductionCode = 1;
                } else if iGameAudioSampleRate == decinfo.samprate >> 2 {
                    (*pMP3Stream).iRewind_FinalReductionCode = 2;
                }

                if iGameAudioSampleBits == decinfo.bits >> 1 {
                    // if game wants 8 bit sounds, then setup for that
                    (*pMP3Stream).iRewind_FinalConvertCode |= 8;
                }

                if audio.decode_init.map_or(false, |f| f(&mut head, (*pMP3Stream).iSourceFrameBytes, (*pMP3Stream).iRewind_FinalReductionCode, (*pMP3Stream).iSourceReadIndex, (*pMP3Stream).iRewind_FinalConvertCode, freq_limit) != 0) {
                    if let Some(decode_info_fn) = audio.decode_info {
                        decode_info_fn(&mut decinfo);
                    }

                    // sod it, no harm in one last check... (should never happen)
                    //
                    if iGameAudioSampleRate != decinfo.samprate || iGameAudioSampleBits != decinfo.bits {
                        psReturn = b"MP3ERR: Decoder unable to convert to current game audio settings\0".as_ptr() as *const c_char;
                    }
                } else {
                    psReturn = b"MP3ERR: Decoder failed to initialise for pass 2 sample adjust\0".as_ptr() as *const c_char;
                }
            } else {
                psReturn = b"MP3ERR: Decoder failed to initialise\0".as_ptr() as *const c_char;
            }
        } else {
            psReturn = b"MP3ERR: Errr.... something's broken with this MP3 file\0".as_ptr() as *const c_char; // should never happen by this point
        }

        // restore global stream ptr before returning to normal functions (so the rest of the MP3 code still works)...
        //
        pMP3Stream = &mut _MP3Stream;

        psReturn
    }
}

// return value is decoded bytes for this packet, which is effectively a BOOL, NZ for not finished decoding yet...
//
#[no_mangle]
pub extern "C" fn C_MP3Stream_Decode(pSFX_MP3Stream: LP_MP3STREAM, bFastForwarding: c_int) -> c_int {
    unsafe {
        let mut uiDecoded: c_int = 0;   // default to "finished"
        let mut x: IN_OUT;

        pMP3Stream = pSFX_MP3Stream;

        loop {
            if (*pSFX_MP3Stream).iSourceBytesRemaining == 0 {
                uiDecoded = 0;   // finished
                break;
            }

            bFastEstimateOnly = bFastForwarding;

            if let Some(decode_fn) = audio.decode {
                x = decode_fn((*pSFX_MP3Stream).pbSourceData.add((*pSFX_MP3Stream).iSourceReadIndex as usize),
                             (*pSFX_MP3Stream).bDecodeBuffer.as_mut_ptr() as *mut i16,
                             (*pSFX_MP3Stream).pbSourceData.add(((*pSFX_MP3Stream).iRewind_SourceReadIndex + (*pSFX_MP3Stream).iRewind_SourceBytesRemaining) as usize));

                bFastEstimateOnly = 0;

                #[cfg(debug_assertions)] {
                    (*pSFX_MP3Stream).iSourceFrameCounter += 1;
                }

                (*pSFX_MP3Stream).iSourceReadIndex += x.in_bytes;
                (*pSFX_MP3Stream).iSourceBytesRemaining -= x.in_bytes;
                (*pSFX_MP3Stream).iBytesDecodedTotal += x.out_bytes;
                (*pSFX_MP3Stream).iBytesDecodedThisPacket = x.out_bytes;

                uiDecoded = x.out_bytes;

                if x.in_bytes <= 0 {
                    uiDecoded = 0;   // finished
                    break;
                }
            }

            break;  // while (0) equivalent
        }

        // restore global stream ptr before returning to normal functions (so the rest of the MP3 code still works)...
        //
        pMP3Stream = &mut _MP3Stream;

        uiDecoded
    }
}


// ret is char* errstring, else NULL for ok
//
#[no_mangle]
pub extern "C" fn C_MP3Stream_Rewind(pSFX_MP3Stream: LP_MP3STREAM) -> *const c_char {
    unsafe {
        let mut psReturn: *const c_char = core::ptr::null();
        let mut head: MPEG_HEAD = core::mem::zeroed();         // only relevant within this function during init
        let mut iBitRate: c_int = 0;                           // ditto
        let mut iNULL: c_int = 0;

        pMP3Stream = pSFX_MP3Stream;

        (*pMP3Stream).iSourceReadIndex = (*pMP3Stream).iRewind_SourceReadIndex;
        (*pMP3Stream).iSourceBytesRemaining = (*pMP3Stream).iRewind_SourceBytesRemaining;   // already adjusted for tags etc

        // I'm not sure that this is needed, but where else does decode_init get passed useful data ptrs?...
        //
        if (*pMP3Stream).iSourceFrameBytes == head_info3((*pMP3Stream).pbSourceData as *mut c_void, (*pMP3Stream).iSourceBytesRemaining / 2, &mut head, &mut iBitRate, &mut iNULL) {
            if audio.decode_init.map_or(false, |f| f(&mut head, (*pMP3Stream).iSourceFrameBytes, (*pMP3Stream).iRewind_FinalReductionCode, (*pMP3Stream).iSourceReadIndex, (*pMP3Stream).iRewind_FinalConvertCode, freq_limit) != 0) {
                // we should always get here...
                //
            } else {
                psReturn = b"MP3ERR: Failed to re-init decoder for rewind!\0".as_ptr() as *const c_char;
            }
        } else {
            psReturn = b"MP3ERR: Frame bytes mismatch during rewind header-read!\0".as_ptr() as *const c_char;
        }

        // restore global stream ptr before returning to normal functions (so the rest of the MP3 code still works)...
        //
        pMP3Stream = &mut _MP3Stream;

        psReturn
    }
}
