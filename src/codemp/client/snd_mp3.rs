//! Mechanical port of `codemp/client/snd_mp3.cpp`.
//!
//! Filename: snd_mp3.cpp
//!
//! (The interface module between all the MP3 stuff and the game)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_short, c_void};
use core::ptr;

use crate::codemp::client::snd_local_h::{byte, channel_t, cvar_t, dma_t, sboolean, sfx_t};
use crate::codemp::client::snd_mp3_h::{
    id3v1_1, C_MP3Stream_Decode, C_MP3Stream_DecodeInit, C_MP3_GetHeaderData,
    C_MP3_GetUnpackedSize, C_MP3_IsValid, C_MP3_UnpackRawPCM,
};
use crate::codemp::mp3code::mp3struct_h::{LP_MP3STREAM, MP3STREAM};

//
// expects data already loaded, filename arg is for error printing only
//
// returns success/fail
//
pub unsafe fn MP3_IsValid(
    psLocalFilename: *const c_char,
    pvData: *mut c_void,
    iDataLen: c_int,
    bStereoDesired: sboolean, // = qfalse
) -> sboolean {
    let psError = C_MP3_IsValid(pvData, iDataLen, bStereoDesired);

    if !psError.is_null() {
        Com_Printf(va(
            "^1%s(%s)\n\0".as_ptr() as *const c_char,
            psError,
            psLocalFilename,
        ));
    }

    if psError.is_null() {
        1
    } else {
        0
    }
}

//
// expects data already loaded, filename arg is for error printing only
//
// returns unpacked length, or 0 for errors (which will be printed internally)
//
pub unsafe fn MP3_GetUnpackedSize(
    psLocalFilename: *const c_char,
    pvData: *mut c_void,
    iDataLen: c_int,
    qbIgnoreID3Tag: sboolean, // = qfalse
    bStereoDesired: sboolean, // = qfalse
) -> c_int {
    let mut iUnpackedSize: c_int = 0;

    // always do this now that we have fast-unpack code for measuring output size... (much safer than relying on tags that may have been edited, or if MP3 has been re-saved with same tag)
    //
    if true
    //qbIgnoreID3Tag || !MP3_ReadSpecialTagInfo((byte *)pvData, iDataLen, NULL, &iUnpackedSize))
    {
        let psError = C_MP3_GetUnpackedSize(pvData, iDataLen, &mut iUnpackedSize, bStereoDesired);

        if !psError.is_null() {
            Com_Printf(va(
                "^1%s\n(File: %s)\n\0".as_ptr() as *const c_char,
                psError,
                psLocalFilename,
            ));
            return 0;
        }
    }

    iUnpackedSize
}

//
// expects data already loaded, filename arg is for error printing only
//
// returns byte count of unpacked data (effectively a success/fail bool)
//
pub unsafe fn MP3_UnpackRawPCM(
    psLocalFilename: *const c_char,
    pvData: *mut c_void,
    iDataLen: c_int,
    pbUnpackBuffer: *mut byte,
    bStereoDesired: sboolean, // = qfalse
) -> c_int {
    let mut iUnpackedSize: c_int = 0;
    let psError = C_MP3_UnpackRawPCM(
        pvData,
        iDataLen,
        &mut iUnpackedSize,
        pbUnpackBuffer as *mut c_void,
        bStereoDesired,
    );

    if !psError.is_null() {
        Com_Printf(va(
            "^1%s\n(File: %s)\n\0".as_ptr() as *const c_char,
            psError,
            psLocalFilename,
        ));
        return 0;
    }

    iUnpackedSize
}

// psLocalFilename is just for error reporting (if any)...
//
pub unsafe fn MP3Stream_InitPlayingTimeFields(
    lpMP3Stream: LP_MP3STREAM,
    psLocalFilename: *const c_char,
    pvData: *mut c_void,
    iDataLen: c_int,
    bStereoDesired: sboolean, // = qfalse
) -> sboolean {
    let mut bRetval: sboolean = 0; // qfalse

    let mut iRate: c_int = 0;
    let mut iWidth: c_int = 0;
    let mut iChannels: c_int = 0;

    let psError = C_MP3_GetHeaderData(
        pvData,
        iDataLen,
        &mut iRate,
        &mut iWidth,
        &mut iChannels,
        bStereoDesired,
    );
    if !psError.is_null() {
        Com_Printf(va(
            "^1MP3Stream_InitPlayingTimeFields(): %s\n(File: %s)\n\0".as_ptr() as *const c_char,
            psError,
            psLocalFilename,
        ));
    } else {
        let iUnpackLength = MP3_GetUnpackedSize(
            psLocalFilename,
            pvData,
            iDataLen,
            0, // sboolean qbIgnoreID3Tag
            bStereoDesired,
        );
        if iUnpackLength != 0 {
            (*lpMP3Stream).iTimeQuery_UnpackedLength = iUnpackLength;
            (*lpMP3Stream).iTimeQuery_SampleRate = iRate;
            (*lpMP3Stream).iTimeQuery_Channels = iChannels;
            (*lpMP3Stream).iTimeQuery_Width = iWidth;

            bRetval = 1; // qtrue
        }
    }

    bRetval
}

pub unsafe fn MP3Stream_GetPlayingTimeInSeconds(lpMP3Stream: LP_MP3STREAM) -> c_float {
    if (*lpMP3Stream).iTimeQuery_UnpackedLength != 0 {
        // fields initialised?
        return ((((*lpMP3Stream).iTimeQuery_UnpackedLength as f64
            / (*lpMP3Stream).iTimeQuery_SampleRate as f64)
            / (*lpMP3Stream).iTimeQuery_Channels as f64)
            / (*lpMP3Stream).iTimeQuery_Width as f64) as c_float;
    }

    0.0f
}

pub unsafe fn MP3Stream_GetRemainingTimeInSeconds(lpMP3Stream: LP_MP3STREAM) -> c_float {
    if (*lpMP3Stream).iTimeQuery_UnpackedLength != 0 {
        // fields initialised?
        return (((((((*lpMP3Stream).iTimeQuery_UnpackedLength
            - ((*lpMP3Stream).iBytesDecodedTotal
                * ((*lpMP3Stream).iTimeQuery_SampleRate / dma.speed)))
            as f64)
            / ((*lpMP3Stream).iTimeQuery_SampleRate as f64))
            / ((*lpMP3Stream).iTimeQuery_Channels as f64))
            / ((*lpMP3Stream).iTimeQuery_Width as f64)) as c_float);
    }

    0.0f
}

//
// expects data already loaded, filename arg is for error printing only
//
pub unsafe fn MP3_FakeUpWAVInfo(
    psLocalFilename: *const c_char,
    pvData: *mut c_void,
    iDataLen: c_int,
    iUnpackedDataLength: c_int,
    format: *mut c_int,
    rate: *mut c_int,
    width: *mut c_int,
    channels: *mut c_int,
    samples: *mut c_int,
    dataofs: *mut c_int,
    bStereoDesired: sboolean, // = qfalse
) -> sboolean {
    // some things can be done instantly...
    //
    *format = 1; // 1 for MS format
    *dataofs = 0; // will be 0 for me (since there's no header in the unpacked data)

    // some things need to be read...  (though the whole stereo flag thing is crap)
    //
    let psError = C_MP3_GetHeaderData(pvData, iDataLen, rate, width, channels, bStereoDesired);
    if !psError.is_null() {
        Com_Printf(va(
            "^1%s\n(File: %s)\n\0".as_ptr() as *const c_char,
            psError,
            psLocalFilename,
        ));
    }

    // and some stuff needs calculating...
    //
    *samples = iUnpackedDataLength / *width;

    if psError.is_null() {
        1
    } else {
        0
    } // return !psError
}

const sKEY_MAXVOL: &[u8] = b"#MAXVOL\0"; // formerly #defines
const sKEY_UNCOMP: &[u8] = b"#UNCOMP\0"; //    "        "

// returns qtrue for success...
//
pub unsafe fn MP3_ReadSpecialTagInfo(
    pbLoadedFile: *mut byte,
    iLoadedFileLen: c_int,
    ppTAG: *mut *mut id3v1_1,       // = NULL
    piUncompressedSize: *mut c_int, // = NULL
    pfMaxVol: *mut c_float,         // = NULL
) -> sboolean {
    let mut qbError: sboolean = 0; // qfalse

    let mut pTAG = (pbLoadedFile as usize + iLoadedFileLen as usize
        - core::mem::size_of::<id3v1_1>()) as *mut id3v1_1; // sizeof = 128

    if strncmp(
        (&(*pTAG).id[0]) as *const c_char,
        "TAG\0".as_ptr() as *const c_char,
        3,
    ) == 0
    {
        // TAG found...
        //

        // read MAXVOL key...
        //
        if strncmp(
            (&(*pTAG).comment[0]) as *const c_char,
            sKEY_MAXVOL.as_ptr() as *const c_char,
            strlen(sKEY_MAXVOL.as_ptr() as *const c_char) as usize,
        ) != 0
        {
            qbError = 1; // qtrue
        } else {
            if !pfMaxVol.is_null() {
                *pfMaxVol = atof(
                    ((&(*pTAG).comment[0]) as *const c_char)
                        .add(strlen(sKEY_MAXVOL.as_ptr() as *const c_char) as usize),
                );
            }
        }

        //
        // read UNCOMP key...
        //
        if strncmp(
            (&(*pTAG).album[0]) as *const c_char,
            sKEY_UNCOMP.as_ptr() as *const c_char,
            strlen(sKEY_UNCOMP.as_ptr() as *const c_char) as usize,
        ) != 0
        {
            qbError = 1; // qtrue
        } else {
            if !piUncompressedSize.is_null() {
                *piUncompressedSize = atoi(
                    ((&(*pTAG).album[0]) as *const c_char)
                        .add(strlen(sKEY_UNCOMP.as_ptr() as *const c_char) as usize),
                );
            }
        }
    } else {
        pTAG = ptr::null_mut();
    }

    if !ppTAG.is_null() {
        *ppTAG = pTAG;
    }

    if !pTAG.is_null() && qbError == 0 {
        1 // qtrue
    } else {
        0 // qfalse
    }
}

const FUZZY_AMOUNT: c_int = 5 * 1024; // so it has to be significantly over, not just break even, because of
                                      // the xtra CPU time versus memory saving

static mut cv_MP3overhead: *mut cvar_t = ptr::null_mut();

pub unsafe fn MP3_InitCvars() {
    cv_MP3overhead = Cvar_Get(
        "s_mp3overhead\0".as_ptr() as *const c_char,
        va(
            "%d\0".as_ptr() as *const c_char,
            core::mem::size_of::<MP3STREAM>() as i32 + FUZZY_AMOUNT,
        ),
        1, // CVAR_ARCHIVE
    );
}

// a file has been loaded in memory, see if we want to keep it as MP3, else as normal WAV...
//
// return = qtrue if keeping as MP3
//
// (note: the reason I pass in the unpacked size rather than working it out here is simply because I already have it)
//
pub unsafe fn MP3Stream_InitFromFile(
    sfx: *mut sfx_t,
    pbSrcData: *mut byte,
    iSrcDatalen: c_int,
    psSrcDataFilename: *const c_char,
    iMP3UnPackedSize: c_int,
    bStereoDesired: sboolean, // = qfalse
) -> sboolean {
    // first, make a decision based on size here as to whether or not it's worth it because of MP3 buffer space
    //	making small files much bigger (and therefore best left as WAV)...
    //

    if !cv_MP3overhead.is_null()
        && (
            //iSrcDatalen + sizeof(MP3STREAM) + FUZZY_AMOUNT < iMP3UnPackedSize
            iSrcDatalen + (*cv_MP3overhead).integer < iMP3UnPackedSize
        )
    {
        // ok, let's keep it as MP3 then...
        //
        let mut fMaxVol: c_float = 128.0; // seems to be a reasonable typical default for maxvol (for lip synch). Naturally there's no #define I can use instead...

        MP3_ReadSpecialTagInfo(
            pbSrcData,
            iSrcDatalen,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut fMaxVol,
        ); // try and read a read maxvol from MP3 header

        // fill in some sfx_t fields...
        //
        //		Q_strncpyz( sfx->name, psSrcDataFilename, sizeof(sfx->name) );
        (*sfx).eSoundCompressionMethod = 1; // ct_MP3
        (*sfx).fVolRange = fMaxVol;
        //sfx->width  = 2;
        (*sfx).iSoundLengthInSamples = (((iMP3UnPackedSize / 2/*sfx->width*/)
            / (44100 / dma.speed))
            / if bStereoDesired != 0 { 2 } else { 1 });
        //
        // alloc mem for data and store it (raw MP3 in this case)...
        //
        (*sfx).pSoundData = SND_malloc(iSrcDatalen, sfx) as *mut c_short;
        memcpy(
            (*sfx).pSoundData as *mut c_void,
            pbSrcData as *mut c_void,
            iSrcDatalen as usize,
        );

        // now init the low-level MP3 stuff...
        //
        let mut SFX_MP3Stream: MP3STREAM = core::mem::zeroed(); // important to init to all zeroes!
        let psError = C_MP3Stream_DecodeInit(
            &mut SFX_MP3Stream,
            /*sfx->data*/ /*sfx->soundData*/ pbSrcData as *mut c_void,
            iSrcDatalen,
            dma.speed, //(s_khz->value == 44)?44100:(s_khz->value == 22)?22050:11025,
            2 /*sfx->width*/ * 8,
            bStereoDesired,
        );
        SFX_MP3Stream.pbSourceData = pbSrcData;
        if !psError.is_null() {
            // This should never happen, since any errors or problems with the MP3 file would have stopped us getting
            //	to this whole function, but just in case...
            //
            Com_Printf(va(
                "^3File \"%s\": %s\n\0".as_ptr() as *const c_char,
                psSrcDataFilename,
                psError,
            ));

            // This will leave iSrcDatalen bytes on the hunk stack (since you can't dealloc that), but MP3 files are
            //	usually small, and like I say, it should never happen.
            //
            // Strictly speaking, I should do a Z_Malloc above, then I could do a Z_Free if failed, else do a Hunk_Alloc
            //	to copy the Z_Malloc data into, then Z_Free, but for something that shouldn't happen it seemed bad to
            //	penalise the rest of the game with extra alloc demands.
            //
            return 0; // qfalse
        }

        // success ( ...on a plate).
        //
        // make a copy of the filled-in stream struct and attach to the sfx_t struct...
        //
        (*sfx).pMP3StreamHeader =
            Z_Malloc(core::mem::size_of::<MP3STREAM>() as c_int, 7, 0) as *mut MP3STREAM; // TAG_SND_MP3STREAMHDR
        memcpy(
            (*sfx).pMP3StreamHeader as *mut c_void,
            &SFX_MP3Stream as *const MP3STREAM as *const c_void,
            core::mem::size_of::<MP3STREAM>(),
        );
        //
        return 1; // qtrue
    }

    0 // qfalse
}

//
// decode one packet of MP3 data only (typical output size is 2304, or 2304*2 for stereo, so input size is less
//
// return is decoded byte count, else 0 for finished
//
pub unsafe fn MP3Stream_Decode(lpMP3Stream: LP_MP3STREAM, bDoingMusic: sboolean) -> c_int {
    (*lpMP3Stream).iCopyOffset = 0;

    if false
    // !bDoingMusic)
    {
        /*
        // SOF2: need to make a local buffer up so we can decode the piece we want from a contiguous bitstream rather than
        //	this linklist junk...
        //
        // since MP3 packets are generally 416 or 417 bytes in length it seems reasonable to just find which linked-chunk
        //	the current read offset lies within then grab the next one as well (since they're 2048 bytes) and make one
        //	buffer with just the two concat'd together. Shouldn't be much of a processor hit.
        //
        sndBuffer *pChunk = (sndBuffer *) lpMP3Stream->pbSourceData;
        //
        // may as well make this static to avoid cut down on stack-validation run-time...
        //
        static byte	byRawBuffer[SND_CHUNK_SIZE_BYTE*2];	// *2 for byte->short	// easily enough to decode one frame of MP3 data, most are 416 or 417 bytes

        // fast-forward to the correct chunk...
        //
        int iBytesToSkipPast = lpMP3Stream->iSourceReadIndex;

        while (iBytesToSkipPast >= SND_CHUNK_SIZE_BYTE)
        {
            pChunk = pChunk->next;
            if (!pChunk)
            {
                // err.... reading off the end of the data stream guys...
                //
                // pChunk = (sndBuffer *) lpMP3Stream->pbSourceData;	// restart
                return 0;	// ... 0 bytes decoded, so will just stop caller-decoder all nice and legal as EOS
            }
            iBytesToSkipPast -= SND_CHUNK_SIZE_BYTE;
        }

        {
            // ok, pChunk is now the 2k or so chunk we're in the middle of...
            //
            int iChunk1BytesToCopy = SND_CHUNK_SIZE_BYTE - iBytesToSkipPast;
            memcpy(byRawBuffer,((byte *)pChunk->sndChunk) + iBytesToSkipPast, iChunk1BytesToCopy);
            //
            // concat next chunk on to this as well...
            //
            pChunk = pChunk->next;
            if (pChunk)
            {
                memcpy(byRawBuffer + iChunk1BytesToCopy, pChunk->sndChunk,	SND_CHUNK_SIZE_BYTE);
            }
            else
            {
                memset(byRawBuffer + iChunk1BytesToCopy, 0,					SND_CHUNK_SIZE_BYTE);
            }
        }


        {
            // now we need to backup some struct fields, fake 'em, do the lo-level call, then restore 'em...
            //
            byte *pbSourceData_Old	= lpMP3Stream->pbSourceData;
            int iSourceReadIndex_Old= lpMP3Stream->iSourceReadIndex;

            lpMP3Stream->pbSourceData	= &byRawBuffer[0];
            lpMP3Stream->iSourceReadIndex= 0;	// since this is zero, not the buffer offset within a chunk, we can play tricks further down when restoring

            {
                unsigned int uiBytesDecoded = C_MP3Stream_Decode( lpMP3Stream, qfalse );

                lpMP3Stream->iSourceReadIndex += iSourceReadIndex_Old;	// note '+=' rather than '=', to take account of movement.
                lpMP3Stream->pbSourceData	   = pbSourceData_Old;

                return uiBytesDecoded;
            }
        }
        */
    } else {
        // SOF2 music, or EF1 anything...
        //
        return C_MP3Stream_Decode(lpMP3Stream, 0) as c_int; // bFastForwarding
    }
}

pub unsafe fn MP3Stream_SeekTo(ch: *mut channel_t, mut fTimeToSeekTo: c_float) -> sboolean {
    const fEpsilon: c_float = 0.05f; // accurate to 1/50 of a second, but plus or minus this gives 1/10 of second

    MP3Stream_Rewind(ch);
    //
    // sanity... :-)
    //
    let fTrackLengthInSeconds = MP3Stream_GetPlayingTimeInSeconds(&mut (*ch).MP3StreamHeader);
    if fTimeToSeekTo > fTrackLengthInSeconds {
        fTimeToSeekTo = fTrackLengthInSeconds;
    }

    // now do the seek...
    //
    loop {
        let fPlayingTimeElapsed = MP3Stream_GetPlayingTimeInSeconds(&mut (*ch).MP3StreamHeader)
            - MP3Stream_GetRemainingTimeInSeconds(&mut (*ch).MP3StreamHeader);
        let fAbsTimeDiff = (fTimeToSeekTo - fPlayingTimeElapsed).abs();

        if fAbsTimeDiff <= fEpsilon {
            return 1; // qtrue
        }

        // when decoding, use fast-forward until within 3 seconds, then slow-decode (which should init stuff properly?)...
        //
        let iBytesDecodedThisPacket =
            C_MP3Stream_Decode(&mut (*ch).MP3StreamHeader, (fAbsTimeDiff > 3.0f) as c_int) as c_int; // bFastForwarding
        if iBytesDecodedThisPacket == 0 {
            break; // EOS
        }
    }

    0 // qfalse
}

// returns qtrue for all ok
//
pub unsafe fn MP3Stream_Rewind(ch: *mut channel_t) -> sboolean {
    (*ch).iMP3SlidingDecodeWritePos = 0;
    (*ch).iMP3SlidingDecodeWindowPos = 0;

    /*
    char *psError = C_MP3Stream_Rewind( &ch->MP3StreamHeader );

    if (psError)
    {
        Com_Printf(S_COLOR_YELLOW"%s\n",psError);
        return qfalse;
    }

    return qtrue;
    */

    // speed opt, since I know I already have the right data setup here...
    //
    memcpy(
        &mut (*ch).MP3StreamHeader as *mut MP3STREAM as *mut c_void,
        (*(*ch).thesfx).pMP3StreamHeader as *const c_void,
        core::mem::size_of::<MP3STREAM>(),
    );
    1 // qtrue
}

// returns qtrue while still playing normally, else qfalse for either finished or request-offset-error
//
pub unsafe fn MP3Stream_GetSamples(
    ch: *mut channel_t,
    mut startingSampleNum: c_int,
    mut count: c_int,
    buf: *mut c_short,
    bStereo: sboolean,
) -> sboolean {
    let mut qbStreamStillGoing: sboolean = 1; // qtrue

    let iQuarterOfSlidingBuffer =
        core::mem::size_of_val(&(*ch).MP3SlidingDecodeBuffer) as c_int / 4;
    let iThreeQuartersOfSlidingBuffer =
        (core::mem::size_of_val(&(*ch).MP3SlidingDecodeBuffer) as c_int * 3) / 4;

    //	Com_Printf("startingSampleNum %d\n",startingSampleNum);

    count *= 2; /* <- = SOF2; ch->sfx->width */
 // count arg was for words, so double it for bytes;

    // convert sample number into a byte offset... (make new variable for clarity?)
    //
    startingSampleNum *= 2 /* <- = SOF2; ch->sfx->width */ * if bStereo != 0 { 2 } else { 1 };

    if startingSampleNum < (*ch).iMP3SlidingDecodeWindowPos {
        // what?!?!?!   smegging time travel needed or something?, forget it
        memset(buf as *mut c_void, 0, count as usize);
        return 0; // qfalse
    }

    //	OutputDebugString(va("\nRequest: startingSampleNum %d, count %d\n",startingSampleNum,count));
    //	OutputDebugString(va("WindowPos %d, WindowWritePos %d\n",ch->iMP3SlidingDecodeWindowPos,ch->iMP3SlidingDecodeWritePos));

    //	sboolean _bDecoded = qfalse;

    loop {
        if !((startingSampleNum >= (*ch).iMP3SlidingDecodeWindowPos)
            && (startingSampleNum + count
                < (*ch).iMP3SlidingDecodeWindowPos + (*ch).iMP3SlidingDecodeWritePos))
        {
            //		if (!_bDecoded)
            //		{
            //			Com_Printf(S_COLOR_YELLOW"Decode needed!\n");
            //		}
            //		_bDecoded = qtrue;
            //		OutputDebugString("Scrolling...");

            let _iBytesDecoded = MP3Stream_Decode(&mut (*ch).MP3StreamHeader, bStereo); // stereo only for music, so this is safe
                                                                                        //		OutputDebugString(va("%d bytes decoded\n",_iBytesDecoded));
            if _iBytesDecoded == 0 {
                // no more source data left so clear the remainder of the buffer...
                //
                memset(
                    ((*ch).MP3SlidingDecodeBuffer.as_mut_ptr() as *mut c_void)
                        .add((*ch).iMP3SlidingDecodeWritePos as usize),
                    0,
                    (core::mem::size_of_val(&(*ch).MP3SlidingDecodeBuffer)
                        - (*ch).iMP3SlidingDecodeWritePos as usize),
                );
                //			OutputDebugString("Finished\n");
                qbStreamStillGoing = 0; // qfalse
                break;
            } else {
                memcpy(
                    ((*ch).MP3SlidingDecodeBuffer.as_mut_ptr() as *mut c_void)
                        .add((*ch).iMP3SlidingDecodeWritePos as usize),
                    (*ch).MP3StreamHeader.bDecodeBuffer.as_ptr() as *const c_void,
                    _iBytesDecoded as usize,
                );

                (*ch).iMP3SlidingDecodeWritePos += _iBytesDecoded;

                // if reached 3/4 of buffer pos, backscroll the decode window by one quarter...
                //
                if (*ch).iMP3SlidingDecodeWritePos > iThreeQuartersOfSlidingBuffer {
                    memmove(
                        (*ch).MP3SlidingDecodeBuffer.as_mut_ptr() as *mut c_void,
                        ((*ch).MP3SlidingDecodeBuffer.as_ptr() as usize
                            + iQuarterOfSlidingBuffer as usize)
                            as *const c_void,
                        iThreeQuartersOfSlidingBuffer as usize,
                    );
                    (*ch).iMP3SlidingDecodeWritePos -= iQuarterOfSlidingBuffer;
                    (*ch).iMP3SlidingDecodeWindowPos += iQuarterOfSlidingBuffer;
                }
            }
        } else {
            break;
        }
        //		OutputDebugString(va("WindowPos %d, WindowWritePos %d\n",ch->iMP3SlidingDecodeWindowPos,ch->iMP3SlidingDecodeWritePos));
    }

    //	if (!_bDecoded)
    //	{
    //		Com_Printf(S_COLOR_YELLOW"No decode needed\n");
    //	}

    debug_assert!(startingSampleNum >= (*ch).iMP3SlidingDecodeWindowPos); // Preserve C assert behavior
    memcpy(
        buf as *mut c_void,
        ((*ch).MP3SlidingDecodeBuffer.as_ptr() as usize
            + (startingSampleNum - (*ch).iMP3SlidingDecodeWindowPos) as usize)
            as *const c_void,
        count as usize,
    );

    //	OutputDebugString("OK\n\n");

    qbStreamStillGoing
}

// ============================================================================
// C FFI externs — libc and engine functions
// ============================================================================

extern "C" {
    fn Com_Printf(fmt: *const c_char, ...);
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn Cvar_Get(name: *const c_char, val: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Z_Malloc(size: c_int, tag: c_int, zero: c_int) -> *mut c_void;
    fn SND_malloc(size: c_int, sfx: *mut sfx_t) -> *mut c_void;

    // libc function stubs
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn atof(s: *const c_char) -> c_float;
    fn atoi(s: *const c_char) -> c_int;
}

extern "C" {
    // Extern global from snd_local.h
    static mut dma: dma_t;
}

///////////// eof /////////////
