// snd_mix.c -- portable code to mix sounds for snd_dma.c

// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_char, c_void};
use std::ptr;

// Preserve original comments from source file:
// #include "../server/exe_headers.h"
// #include "snd_local.h"

use crate::code::client::snd_local_h::{
    portable_samplepair_t, channel_t, sfx_t, PAINTBUFFER_SIZE, MAX_CHANNELS, MAX_RAW_SAMPLES,
    dma_t, s_channels, s_paintedtime, s_rawend, s_rawsamples, s_volume, s_volumeVoice,
    s_testsound, SoundCompressionMethod_t,
};

// cvar_t structure definition (needed for field access in this file)
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,      // cvar_restart will reset to this value
    pub latchedString: *mut c_char,    // for CVAR_LATCH vars
    pub flags: c_int,
    pub modified: c_int,               // set each time the cvar is changed
    pub modificationCount: c_int,      // incremented each time the cvar is changed
    pub value: f32,                    // atof( string )
    pub integer: c_int,                // atoi( string )
    pub next: *mut cvar_t,
}

extern "C" {
    // Forward declarations needed from other modules
    fn MP3Stream_GetSamples(
        ch: *mut channel_t,
        startingSampleNum: c_int,
        count: c_int,
        buf: *mut i16,
        bStereo: c_int,
    ) -> c_int;
}

// Global variables - matching C file structure
pub static mut paintbuffer: [portable_samplepair_t; PAINTBUFFER_SIZE as usize] =
    [portable_samplepair_t { left: 0, right: 0 }; PAINTBUFFER_SIZE as usize];
pub static mut snd_p: *mut c_int = ptr::null_mut();
pub static mut snd_linear_count: c_int = 0;
pub static mut snd_vol: c_int = 0;
pub static mut snd_out: *mut i16 = ptr::null_mut();

static mut uiMMXAvailable: u32 = 0; // leave as 32 bit

unsafe fn S_WriteLinearBlastStereo16() {
    let mut i: c_int = 0;
    let mut val: c_int;

    while i < snd_linear_count {
        val = *snd_p.add(i as usize) >> 8;
        if val > 0x7fff {
            *snd_out.add(i as usize) = 0x7fff as i16;
        } else if val < (0x8000i32 as i16) as c_int {
            *snd_out.add(i as usize) = 0x8000u32 as i32 as i16;
        } else {
            *snd_out.add(i as usize) = val as i16;
        }

        val = *snd_p.add((i + 1) as usize) >> 8;
        if val > 0x7fff {
            *snd_out.add((i + 1) as usize) = 0x7fff as i16;
        } else if val < (0x8000i32 as i16) as c_int {
            *snd_out.add((i + 1) as usize) = 0x8000u32 as i32 as i16;
        } else {
            *snd_out.add((i + 1) as usize) = val as i16;
        }

        i += 2;
    }
}

// __declspec( naked ) void S_WriteLinearBlastStereo16 (void)
// This is x86 assembly code - preserved as comments since Rust doesn't have inline asm like this
// Original assembly implementation provided for reference:
//
// push edi
// push ebx
//
// mov ecx,ds:dword ptr[snd_linear_count]		// snd_linear_count is always even at this point, but not nec. mult of 4
// mov ebx,ds:dword ptr[snd_p]
// mov edi,ds:dword ptr[snd_out]
//
// cmp		[uiMMXAvailable], dword ptr 0
// je			NoMMX
//
// // writes 8 items (128 bits) per loop pass...
// //
// cmp		ecx,8
// jb			NoMMX
//
// LWLBLoopTop_MMX:
//
// movq		mm1,[-8+ebx+ecx*4]
// movq		mm0,[-16+ebx+ecx*4]
// movq		mm3,[-24+ebx+ecx*4]
// movq		mm2,[-32+ebx+ecx*4]
// psrad		mm0,8
// psrad		mm1,8
// psrad		mm2,8
// psrad		mm3,8
// packssdw	mm0,mm1
// packssdw	mm2,mm3
// movq		[-8+edi+ecx*2],mm0
// movq		[-16+edi+ecx*2],mm2
//
// sub		ecx,8
// cmp		ecx,8
// jae		LWLBLoopTop_MMX
//
// emms
//
// // now deal with any remaining count...
// //
// jecxz		LExit
//
// NoMMX:
//
// // writes 2 items (32 bits) per loop pass...
// //
// LWLBLoopTop:
// mov eax,ds:dword ptr[-8+ebx+ecx*4]
// sar eax,8
// cmp eax,07FFFh
// jg LClampHigh
// cmp eax,0FFFF8000h
// jnl LClampDone
// mov eax,0FFFF8000h
// jmp LClampDone
// LClampHigh:
// mov eax,07FFFh
// LClampDone:
// mov edx,ds:dword ptr[-4+ebx+ecx*4]
// sar edx,8
// cmp edx,07FFFh
// jg LClampHigh2
// cmp edx,0FFFF8000h
// jnl LClampDone2
// mov edx,0FFFF8000h
// jmp LClampDone2
// LClampHigh2:
// mov edx,07FFFh
// LClampDone2:
// shl edx,16
// and eax,0FFFFh
// or edx,eax
// mov ds:dword ptr[-4+edi+ecx*2],edx
//
// sub ecx,2
// jnz LWLBLoopTop
//
// LExit:
// pop ebx
// pop edi
// ret

unsafe fn S_TransferStereo16(pbuf: *mut u32, endtime: c_int) {
    let mut lpos: c_int;
    let mut ls_paintedtime: c_int;

    snd_p = paintbuffer.as_mut_ptr() as *mut c_int;
    ls_paintedtime = s_paintedtime;

    while ls_paintedtime < endtime {
        // handle recirculating buffer issues
        lpos = ls_paintedtime & ((dma.samples >> 1) - 1);

        snd_out = (pbuf as *mut i16).add((lpos << 1) as usize);

        snd_linear_count = (dma.samples >> 1) - lpos;
        if ls_paintedtime + snd_linear_count > endtime {
            snd_linear_count = endtime - ls_paintedtime;
        }

        snd_linear_count <<= 1;

        // write a linear blast of samples
        S_WriteLinearBlastStereo16();

        snd_p = snd_p.add(snd_linear_count as usize);
        ls_paintedtime += (snd_linear_count >> 1);
    }
}

/*
===================
S_TransferPaintBuffer

===================
*/
unsafe fn S_TransferPaintBuffer(endtime: c_int) {
    let mut out_idx: c_int;
    let mut count: c_int;
    let mut out_mask: c_int;
    let mut p: *mut c_int;
    let mut step: c_int;
    let mut val: c_int;
    let mut pbuf: *mut u32;

    pbuf = dma.buffer as *mut u32;

    if (*(s_testsound as *mut cvar_t)).integer != 0 {
        // write a fixed sine wave
        count = (endtime - s_paintedtime);
        for idx in 0..count {
            let sample = (((s_paintedtime + idx) as f32 * 0.1).sin() * 20000.0 * 256.0) as c_int;
            paintbuffer[idx as usize].left = sample;
            paintbuffer[idx as usize].right = sample;
        }
    }

    if dma.samplebits == 16 && dma.channels == 2 {
        // optimized case
        S_TransferStereo16(pbuf, endtime);
    } else {
        // general case
        p = paintbuffer.as_mut_ptr() as *mut c_int;
        count = (endtime - s_paintedtime) * dma.channels;
        out_mask = dma.samples - 1;
        out_idx = (s_paintedtime * dma.channels) & out_mask;
        step = 3 - dma.channels;

        if dma.samplebits == 16 {
            let out: *mut i16 = pbuf as *mut i16;
            while count > 0 {
                val = *p >> 8;
                p = p.add(step as usize);
                if val > 0x7fff {
                    val = 0x7fff;
                } else if val < (0x8000i32 as i16) as c_int {
                    val = (0x8000i32 as i16) as c_int;
                }
                *out.add(out_idx as usize) = val as i16;
                out_idx = (out_idx + 1) & out_mask;
                count -= 1;
            }
        } else if dma.samplebits == 8 {
            let out: *mut u8 = pbuf as *mut u8;
            while count > 0 {
                val = *p >> 8;
                p = p.add(step as usize);
                if val > 0x7fff {
                    val = 0x7fff;
                } else if val < (0x8000i32 as i16) as c_int {
                    val = (0x8000i32 as i16) as c_int;
                }
                *out.add(out_idx as usize) = ((val >> 8) + 128) as u8;
                out_idx = (out_idx + 1) & out_mask;
                count -= 1;
            }
        }
    }
}

/*
===============================================================================

CHANNEL MIXING

===============================================================================
*/

unsafe fn S_PaintChannelFrom16(
    ch: *mut channel_t,
    sfx: *const sfx_t,
    count: c_int,
    sampleOffset: c_int,
    bufferOffset: c_int,
) {
    let pSamplesDest: *mut portable_samplepair_t;
    let iData: c_int;

    let iLeftVol: c_int = (*ch).leftvol * snd_vol;
    let iRightVol: c_int = (*ch).rightvol * snd_vol;

    pSamplesDest = paintbuffer.as_mut_ptr().add(bufferOffset as usize);

    for i in 0..count {
        let iData: c_int = *(*sfx).pSoundData.add((sampleOffset + i) as usize) as c_int;

        (*pSamplesDest.add(i as usize)).left += (iData * iLeftVol) >> 8;
        (*pSamplesDest.add(i as usize)).right += (iData * iRightVol) >> 8;
    }
}

unsafe fn S_PaintChannelFromMP3(
    ch: *mut channel_t,
    sc: *const sfx_t,
    count: c_int,
    sampleOffset: c_int,
    bufferOffset: c_int,
) {
    let mut data: c_int;
    let mut leftvol: c_int;
    let mut rightvol: c_int;
    let mut sfx: *mut i16;
    let mut i: c_int;
    let mut samp: *mut portable_samplepair_t;
    // PAINTBUFFER_SIZE = 1024
    let mut tempMP3Buffer: [i16; 1024] = [0; 1024];

    MP3Stream_GetSamples(ch, sampleOffset, count, tempMP3Buffer.as_mut_ptr(), 0); // 0 = not stereo

    leftvol = (*ch).leftvol * snd_vol;
    rightvol = (*ch).rightvol * snd_vol;
    sfx = tempMP3Buffer.as_mut_ptr();

    samp = paintbuffer.as_mut_ptr().add(bufferOffset as usize);

    let mut count_remaining: c_int = count;
    while (count_remaining & 3) != 0 {
        data = *sfx as c_int;
        (*samp).left += (data * leftvol) >> 8;
        (*samp).right += (data * rightvol) >> 8;

        sfx = sfx.add(1);
        samp = samp.add(1);
        count_remaining -= 1;
    }

    i = 0;
    while i < count_remaining {
        data = *sfx.add(i as usize) as c_int;
        (*samp.add(i as usize)).left += (data * leftvol) >> 8;
        (*samp.add(i as usize)).right += (data * rightvol) >> 8;

        data = *sfx.add((i + 1) as usize) as c_int;
        (*samp.add((i + 1) as usize)).left += (data * leftvol) >> 8;
        (*samp.add((i + 1) as usize)).right += (data * rightvol) >> 8;

        data = *sfx.add((i + 2) as usize) as c_int;
        (*samp.add((i + 2) as usize)).left += (data * leftvol) >> 8;
        (*samp.add((i + 2) as usize)).right += (data * rightvol) >> 8;

        data = *sfx.add((i + 3) as usize) as c_int;
        (*samp.add((i + 3) as usize)).left += (data * leftvol) >> 8;
        (*samp.add((i + 3) as usize)).right += (data * rightvol) >> 8;

        i += 4;
    }
}

// subroutinised to save code dup (called twice)	-ste
//
unsafe fn ChannelPaint(
    ch: *mut channel_t,
    sc: *mut sfx_t,
    count: c_int,
    sampleOffset: c_int,
    bufferOffset: c_int,
) {
    match (*sc).eSoundCompressionMethod {
        SoundCompressionMethod_t::ct_16 => {
            S_PaintChannelFrom16(ch, sc, count, sampleOffset, bufferOffset);
        }

        SoundCompressionMethod_t::ct_MP3 => {
            S_PaintChannelFromMP3(ch, sc, count, sampleOffset, bufferOffset);
        }

        SoundCompressionMethod_t::ct_NUMBEROF => {
            // debug aid, ignored in release. FIXME: Should we ERR_DROP here for badness-catch?
            // assert(0);
        }
    }
}

pub unsafe fn S_PaintChannels(endtime: c_int) {
    let mut i: c_int;
    let mut end: c_int;
    let mut ch: *mut channel_t;
    let mut sc: *mut sfx_t;
    let mut ltime: c_int;
    let mut count: c_int;
    let mut sampleOffset: c_int;
    let mut normal_vol: c_int;
    let mut voice_vol: c_int;

    snd_vol = ((*(s_volume as *mut cvar_t)).value * 256.0) as c_int;
    normal_vol = snd_vol;
    voice_vol = ((*(s_volumeVoice as *mut cvar_t)).value * 256.0) as c_int;

    // Com_Printf ("%i to %i\n", s_paintedtime, endtime);
    while s_paintedtime < endtime {
        // if paintbuffer is smaller than DMA buffer
        // we may need to fill it multiple times
        end = endtime;
        if endtime - s_paintedtime > PAINTBUFFER_SIZE {
            end = s_paintedtime + PAINTBUFFER_SIZE;
        }

        // clear the paint buffer to either music or zeros
        if s_rawend < s_paintedtime {
            if s_rawend != 0 {
                // Com_DPrintf ("background sound underrun\n");
            }
            let len: usize = (end - s_paintedtime) as usize;
            ptr::write_bytes(
                paintbuffer.as_mut_ptr() as *mut u8,
                0,
                len * core::mem::size_of::<portable_samplepair_t>(),
            );
        } else {
            // copy from the streaming sound source
            let mut s: c_int;
            let mut stop: c_int;

            stop = if end < s_rawend { end } else { s_rawend };

            i = s_paintedtime;
            while i < stop {
                s = i & (MAX_RAW_SAMPLES - 1);
                paintbuffer[(i - s_paintedtime) as usize] = s_rawsamples[s as usize];
                i += 1;
            }

            // if (i != end)
            //     Com_Printf ("partial stream\n");
            // else
            //     Com_Printf ("full stream\n");
            while i < end {
                paintbuffer[(i - s_paintedtime) as usize].left = 0;
                paintbuffer[(i - s_paintedtime) as usize].right = 0;
                i += 1;
            }
        }

        // paint in the channels.
        ch = s_channels.as_mut_ptr();
        i = 0;
        while i < MAX_CHANNELS {
            if (*ch).thesfx.is_null() || ((*ch).leftvol < 0.25 && (*ch).rightvol < 0.25) {
                ch = ch.add(1);
                i += 1;
                continue;
            }

            if (*ch).entchannel == 3
                || (*ch).entchannel == 4
                || (*ch).entchannel == 5
            {
                // CHAN_VOICE, CHAN_VOICE_ATTEN, CHAN_VOICE_GLOBAL
                snd_vol = voice_vol;
            } else {
                snd_vol = normal_vol;
            }

            ltime = s_paintedtime;
            sc = (*ch).thesfx;

            // we might have to make 2 passes if it is
            // a looping sound effect and the end of
            // the sameple is hit...
            //
            loop {
                if (*ch).loopSound != 0 {
                    sampleOffset = ltime % (*sc).iSoundLengthInSamples;
                } else {
                    sampleOffset = ltime - (*ch).startSample;
                }

                count = end - ltime;
                if sampleOffset + count > (*sc).iSoundLengthInSamples {
                    count = (*sc).iSoundLengthInSamples - sampleOffset;
                }

                if count > 0 {
                    ChannelPaint(ch, sc, count, sampleOffset, ltime - s_paintedtime);
                    ltime += count;
                }

                if !(ltime < end && (*ch).loopSound != 0) {
                    break;
                }
            }

            ch = ch.add(1);
            i += 1;
        }

        // /* temprem
        // paint in the looped channels.
        // ch = loop_channels;
        // for ( i = 0; i < numLoopChannels ; i++, ch++ ) {
        //     if ( !ch->thesfx || (!ch->leftvol && !ch->rightvol )) {
        //         continue;
        //     }
        //
        //     {
        //
        //         ltime = s_paintedtime;
        //         sc = ch->thesfx;
        //
        //         if (sc->soundData==NULL || sc->soundLength==0) {
        //             continue;
        //         }
        //         // we might have to make two passes if it
        //         // is a looping sound effect and the end of
        //         // the sample is hit
        //         do {
        //             sampleOffset = (ltime % sc->soundLength);
        //
        //             count = end - ltime;
        //             if ( sampleOffset + count > sc->soundLength ) {
        //                 count = sc->soundLength - sampleOffset;
        //             }
        //
        //             if ( count > 0 )
        //             {
        //                 ChannelPaint(ch, sc, count, sampleOffset, ltime - s_paintedtime);
        //                 ltime += count;
        //             }
        //
        //         } while ( ltime < end);
        //     }
        // }
        // */

        // transfer out according to DMA format
        S_TransferPaintBuffer(end);
        s_paintedtime = end;
    }
}
