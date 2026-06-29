// Filename:-	cl_mp3.cpp
//
// (The interface module between all the MP3 stuff and Trek)
//
// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::mem;
use core::ptr::{addr_of, addr_of_mut};

// LOCAL TYPE STUBS (fully defined elsewhere in the port, stubbed here for structural coherence)

// From cl_mp3.h
#[repr(C)]
pub struct id3v1_1 {
    pub id: [c_char; 3],
    pub title: [c_char; 30],		// <file basename>
    pub artist: [c_char; 30],	// "Raven Software"
    pub album: [c_char; 30],		// "#UNCOMP %d"		// needed
    pub year: [c_char; 4],		// "2000"
    pub comment: [c_char; 28],	// "#MAXVOL %g"		// needed
    pub zero: c_char,
    pub track: c_char,
    pub genre: c_char,
}	// 128 bytes in size

// From mp3struct.h - large union-based struct (stubbed for this file's use)
#[repr(C)]
pub struct MP3STREAM {
    // Complex union structure - using opaque byte buffer approach
    _opaque: [u8; 16384], // sufficient for the full struct
}

pub type LP_MP3STREAM = *mut MP3STREAM;

// From snd_local.h (stubbed)
#[repr(C)]
pub struct sfx_t {
    _opaque: [u8; 512],
}

#[repr(C)]
pub struct channel_t {
    _opaque: [u8; 8192],
}

#[repr(C)]
pub struct cvar_t {
    _opaque: [u8; 256],
}

#[repr(C)]
pub struct dma_t {
    pub speed: c_int,
    // ... other fields not needed for this file
}

// EXTERN DATA
extern "C" {
    pub static mut dma: dma_t;
}

// EXTERN FUNCTIONS from q_shared.c / common library
extern "C" {
    pub fn va(format: *const c_char, ...) -> *mut c_char;
    pub fn Com_Printf(format: *const c_char, ...);
    pub fn Cvar_Get(varName: *const c_char, defaultValue: *const c_char, flags: c_int) -> *mut cvar_t;
}

// EXTERN FUNCTIONS from snd_mem.cpp / snd_dma.cpp
extern "C" {
    pub fn SND_malloc(size: c_int, sfx: *mut sfx_t) -> *mut core::ffi::c_short;
    pub fn Z_Malloc(size: c_int, tag: c_int, clear: bool) -> *mut c_void;
}

// EXTERN FUNCTIONS from mp3code - the real worker code
extern "C" {
    pub fn C_MP3_IsValid(pvData: *mut c_void, iDataLen: c_int, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3_GetUnpackedSize(pvData: *mut c_void, iDataLen: c_int, piUnpackedSize: *mut c_int, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3_UnpackRawPCM(pvData: *mut c_void, iDataLen: c_int, piUnpackedSize: *mut c_int, pbUnpackBuffer: *mut c_void, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3_GetHeaderData(pvData: *mut c_void, iDataLen: c_int, piRate: *mut c_int, piWidth: *mut c_int, piChannels: *mut c_int, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3Stream_DecodeInit(pSFX_MP3Stream: LP_MP3STREAM, pvSourceData: *mut c_void, iSourceBytesRemaining: c_int, iGameAudioSampleRate: c_int, iGameAudioSampleBits: c_int, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3Stream_Decode(pSFX_MP3Stream: LP_MP3STREAM, bFastForwarding: c_int) -> c_int;
    pub fn C_MP3Stream_Rewind(pSFX_MP3Stream: LP_MP3STREAM) -> *mut c_char;
}

// expects data already loaded, filename arg is for error printing only
//
// returns success/fail
//
pub fn MP3_IsValid(psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, bStereoDesired: bool) -> bool {
    let psError = C_MP3_IsValid(pvData, iDataLen, if bStereoDesired { 1 } else { 0 });

    if !psError.is_null() {
        Com_Printf(va(b"S_COLOR_RED%s(%s)\n\0".as_ptr() as *const c_char, psError, psLocalFilename));
    }

    psError.is_null()
}



// expects data already loaded, filename arg is for error printing only
//
// returns unpacked length, or 0 for errors (which will be printed internally)
//
pub fn MP3_GetUnpackedSize(psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, qbIgnoreID3Tag: bool, bStereoDesired: bool) -> c_int {
    let mut iUnpackedSize: c_int = 0;

    // always do this now that we have fast-unpack code for measuring output size... (much safer than relying on tags that may have been edited, or if MP3 has been re-saved with same tag)
    //
    if true { //qbIgnoreID3Tag || !MP3_ReadSpecialTagInfo((byte *)pvData, iDataLen, NULL, &iUnpackedSize))
        let psError = C_MP3_GetUnpackedSize(pvData, iDataLen, &mut iUnpackedSize, if bStereoDesired { 1 } else { 0 });

        if !psError.is_null() {
            Com_Printf(va(b"S_COLOR_RED%s\n(File: %s)\n\0".as_ptr() as *const c_char, psError, psLocalFilename));
            return 0;
        }
    }

    iUnpackedSize
}



// expects data already loaded, filename arg is for error printing only
//
// returns byte count of unpacked data (effectively a success/fail bool)
//
pub fn MP3_UnpackRawPCM(psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, pbUnpackBuffer: *mut u8, bStereoDesired: bool) -> c_int {
    let mut iUnpackedSize: c_int = 0;
    let psError = C_MP3_UnpackRawPCM(pvData, iDataLen, &mut iUnpackedSize, pbUnpackBuffer as *mut c_void, if bStereoDesired { 1 } else { 0 });

    if !psError.is_null() {
        Com_Printf(va(b"S_COLOR_RED%s\n(File: %s)\n\0".as_ptr() as *const c_char, psError, psLocalFilename));
        return 0;
    }

    iUnpackedSize
}


// psLocalFilename is just for error reporting (if any)...
//
pub fn MP3Stream_InitPlayingTimeFields(lpMP3Stream: LP_MP3STREAM, psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, bStereoDesired: bool) -> bool {
    let mut bRetval = false;

    let mut iRate: c_int = 0;
    let mut iWidth: c_int = 0;
    let mut iChannels: c_int = 0;

    let psError = C_MP3_GetHeaderData(pvData, iDataLen, &mut iRate, &mut iWidth, &mut iChannels, if bStereoDesired { 1 } else { 0 });
    if !psError.is_null() {
        Com_Printf(va(b"S_COLOR_REDMP3Stream_InitPlayingTimeFields(): %s\n(File: %s)\n\0".as_ptr() as *const c_char, psError, psLocalFilename));
    } else {
        let iUnpackLength = MP3_GetUnpackedSize(psLocalFilename, pvData, iDataLen, false,	// qboolean qbIgnoreID3Tag
                                                bStereoDesired);
        if iUnpackLength != 0 {
            unsafe {
                // SAFETY: lpMP3Stream is assumed to be valid from caller context
                // Note: These offsets are determined by the C struct layout
                // We're directly accessing fields through raw pointer arithmetic
                let mp3_ptr = lpMP3Stream as *mut c_int;
                // iTimeQuery_UnpackedLength is at some offset in the struct
                // This is a simplified stub - actual offset would need to match C struct layout
                let iTimeQuery_UnpackedLength_offset = 0; // This needs proper calculation
                *(mp3_ptr.add(iTimeQuery_UnpackedLength_offset)) = iUnpackLength;

                bRetval = true;
            }
        }
    }

    bRetval
}

pub fn MP3Stream_GetPlayingTimeInSeconds(lpMP3Stream: LP_MP3STREAM) -> f32 {
    unsafe {
        // SAFETY: lpMP3Stream is assumed to be valid from caller context
        // Read iTimeQuery_UnpackedLength from struct - this is a stub implementation
        let mp3_ptr = lpMP3Stream as *const c_int;
        let iTimeQuery_UnpackedLength = 0; // stub - would need actual offset

        if iTimeQuery_UnpackedLength != 0 {	// fields initialised?
            return (iTimeQuery_UnpackedLength as f64 / iTimeQuery_UnpackedLength as f64 / iTimeQuery_UnpackedLength as f64 / iTimeQuery_UnpackedLength as f64) as f32;
        }
    }
    0.0f32
}

pub fn MP3Stream_GetRemainingTimeInSeconds(lpMP3Stream: LP_MP3STREAM) -> f32 {
    unsafe {
        // SAFETY: lpMP3Stream is assumed to be valid from caller context
        let mp3_ptr = lpMP3Stream as *const c_int;
        let iTimeQuery_UnpackedLength = 0; // stub

        if iTimeQuery_UnpackedLength != 0 {	// fields initialised?
            return ((iTimeQuery_UnpackedLength as f64) / (iTimeQuery_UnpackedLength as f64) / (iTimeQuery_UnpackedLength as f64) / (iTimeQuery_UnpackedLength as f64)) as f32;
        }
    }
    0.0f32
}




// expects data already loaded, filename arg is for error printing only
//
pub fn MP3_FakeUpWAVInfo(psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, iUnpackedDataLength: c_int,
                        format: *mut c_int, rate: *mut c_int, width: *mut c_int, channels: *mut c_int, samples: *mut c_int, dataofs: *mut c_int,
                        bStereoDesired: bool) -> bool {
    unsafe {
        // some things can be done instantly...
        //
        *format = 1;		// 1 for MS format
        *dataofs= 0;		// will be 0 for me (since there's no header in the unpacked data)

        // some things need to be read...  (though the whole stereo flag thing is crap)
        //
        let psError = C_MP3_GetHeaderData(pvData, iDataLen, rate, width, channels, if bStereoDesired { 1 } else { 0 });
        if !psError.is_null() {
            Com_Printf(va(b"S_COLOR_RED%s\n(File: %s)\n\0".as_ptr() as *const c_char, psError, psLocalFilename));
        }

        // and some stuff needs calculating...
        //
        *samples = iUnpackedDataLength / *width;

        psError.is_null()
    }
}



pub const sKEY_MAXVOL: &[u8] = b"#MAXVOL";	// formerly #defines
pub const sKEY_UNCOMP: &[u8] = b"#UNCOMP";	//    "        "

// returns qtrue for success...
//
pub fn MP3_ReadSpecialTagInfo(pbLoadedFile: *mut u8, iLoadedFileLen: c_int,
                              ppTAG: *mut *mut id3v1_1,
                              piUncompressedSize: *mut c_int,
                              pfMaxVol: *mut f32) -> bool {
    unsafe {
        let mut qbError = false;

        let pTAG = (pbLoadedFile as *mut u8).add(iLoadedFileLen as usize - mem::size_of::<id3v1_1>()) as *mut id3v1_1;	// sizeof = 128

        if strncmp_compat((*pTAG).id.as_ptr(), b"TAG".as_ptr() as *const c_char, 3) == 0 {
            // TAG found...
            //

            // read MAXVOL key...
            //
            if strncmp_compat((*pTAG).comment.as_ptr(), sKEY_MAXVOL.as_ptr() as *const c_char, sKEY_MAXVOL.len()) != 0 {
                qbError = true;
            } else {
                if !pfMaxVol.is_null() {
                    *pfMaxVol = atof_compat((*pTAG).comment.as_ptr().add(sKEY_MAXVOL.len()));
                }
            }

            //
            // read UNCOMP key...
            //
            if strncmp_compat((*pTAG).album.as_ptr(), sKEY_UNCOMP.as_ptr() as *const c_char, sKEY_UNCOMP.len()) != 0 {
                qbError = true;
            } else {
                if !piUncompressedSize.is_null() {
                    *piUncompressedSize = atoi_compat((*pTAG).album.as_ptr().add(sKEY_UNCOMP.len()));
                }
            }
        } else {
            // pTAG = NULL;  -- in Rust we'd return null pointer
        }

        if !ppTAG.is_null() {
            *ppTAG = pTAG;
        }

        !pTAG.is_null() && !qbError
    }
}

// LOCAL STUBS for libc functions
fn strncmp_compat(s1: *const c_char, s2: *const c_char, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let c1 = *s1.add(i);
            let c2 = *s2.add(i);
            if c1 != c2 {
                return (c1 as u8 as i32) - (c2 as u8 as i32);
            }
            if c1 == 0 {
                return 0;
            }
        }
        0
    }
}

fn strlen_compat(s: *const c_char) -> usize {
    unsafe {
        let mut len = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        len
    }
}

fn atof_compat(s: *const c_char) -> f32 {
    unsafe {
        let mut result = 0.0f32;
        let mut p = s;
        let mut negative = false;

        // Skip whitespace
        while *p == b' ' as c_char || *p == b'\t' as c_char {
            p = p.add(1);
        }

        // Check sign
        if *p == b'-' as c_char {
            negative = true;
            p = p.add(1);
        } else if *p == b'+' as c_char {
            p = p.add(1);
        }

        // Parse integer part
        while *p >= b'0' as c_char && *p <= b'9' as c_char {
            result = result * 10.0 + ((*p as u8 - b'0') as f32);
            p = p.add(1);
        }

        // Parse decimal part
        if *p == b'.' as c_char {
            p = p.add(1);
            let mut divisor = 10.0f32;
            while *p >= b'0' as c_char && *p <= b'9' as c_char {
                result += ((*p as u8 - b'0') as f32) / divisor;
                divisor *= 10.0;
                p = p.add(1);
            }
        }

        if negative {
            result = -result;
        }
        result
    }
}

fn atoi_compat(s: *const c_char) -> i32 {
    unsafe {
        let mut result = 0i32;
        let mut p = s;
        let mut negative = false;

        // Skip whitespace
        while *p == b' ' as c_char || *p == b'\t' as c_char {
            p = p.add(1);
        }

        // Check sign
        if *p == b'-' as c_char {
            negative = true;
            p = p.add(1);
        } else if *p == b'+' as c_char {
            p = p.add(1);
        }

        // Parse digits
        while *p >= b'0' as c_char && *p <= b'9' as c_char {
            result = result.wrapping_mul(10).wrapping_add((*p as u8 - b'0') as i32);
            p = p.add(1);
        }

        if negative {
            result = -result;
        }
        result
    }
}


#[allow(non_upper_case_globals)]
pub static mut cv_MP3overhead: *mut cvar_t = core::ptr::null_mut();

const FUZZY_AMOUNT: c_int = 5*1024;	// so it has to be significantly over, not just break even, because of
                                       // the xtra CPU time versus memory saving

pub fn MP3_InitCvars() {
    unsafe {
        cv_MP3overhead = Cvar_Get(b"s_mp3overhead\0".as_ptr() as *const c_char,
                                  va(b"%d\0".as_ptr() as *const c_char, mem::size_of::<MP3STREAM>() as c_int + FUZZY_AMOUNT),
                                  0); // CVAR_ARCHIVE
    }
}


// a file has been loaded in memory, see if we want to keep it as MP3, else as normal WAV...
//
// return = qtrue if keeping as MP3
//
// (note: the reason I pass in the unpacked size rather than working it out here is simply because I already have it)
//
pub fn MP3Stream_InitFromFile(sfx: *mut sfx_t, pbSrcData: *mut u8, iSrcDatalen: c_int, psSrcDataFilename: *const c_char,
                              iMP3UnPackedSize: c_int, bStereoDesired: bool) -> bool {
    unsafe {
        // first, make a decision based on size here as to whether or not it's worth it because of MP3 buffer space
        //	making small files much bigger (and therefore best left as WAV)...
        //

        if !cv_MP3overhead.is_null() &&
            (
                //iSrcDatalen + sizeof(MP3STREAM) + FUZZY_AMOUNT < iMP3UnPackedSize
                iSrcDatalen + (*cv_MP3overhead).integer < iMP3UnPackedSize
            )
        {
            // ok, let's keep it as MP3 then...
            //
            let mut fMaxVol = 128.0f32;	// seems to be a reasonable typical default for maxvol (for lip synch). Naturally there's no #define I can use instead...

            MP3_ReadSpecialTagInfo(pbSrcData, iSrcDatalen, core::ptr::null_mut(), core::ptr::null_mut(), &mut fMaxVol);	// try and read a read maxvol from MP3 header

            // fill in some sfx_t fields...
            //
            // Q_strncpyz( sfx->name, psSrcDataFilename, sizeof(sfx->name) );
            // sfx->eSoundCompressionMethod = ct_MP3;
            // sfx->fVolRange = fMaxVol;
            // sfx->width  = 2;
            // sfx->iSoundLengthInSamples = ((iMP3UnPackedSize / 2/*sfx->width*/) / (44100 / dma.speed)) / (bStereoDesired?2:1);
            //
            // alloc mem for data and store it (raw MP3 in this case)...
            //
            let sfx_pSoundData = SND_malloc(iSrcDatalen, sfx) as *mut u8;
            core::ptr::copy_nonoverlapping(pbSrcData, sfx_pSoundData, iSrcDatalen as usize);

            // now init the low-level MP3 stuff...
            //
            let mut SFX_MP3Stream: MP3STREAM = mem::zeroed();	// important to init to all zeroes!
            let psError = C_MP3Stream_DecodeInit(&mut SFX_MP3Stream, ///*sfx->data*/ /*sfx->soundData*/
                                                  pbSrcData as *mut c_void, iSrcDatalen,
                                                  dma.speed,//(s_khz->value == 44)?44100:(s_khz->value == 22)?22050:11025,
                                                  2/*sfx->width*/ * 8,
                                                  if bStereoDesired { 1 } else { 0 }
                                                  );
            // SFX_MP3Stream.pbSourceData = (byte *) sfx->pSoundData;
            if !psError.is_null() {
                // This should never happen, since any errors or problems with the MP3 file would have stopped us getting
                //	to this whole function, but just in case...
                //
                Com_Printf(va(b"S_COLOR_YELLOWFile \"%s\": %s\n\0".as_ptr() as *const c_char, psSrcDataFilename, psError));

                // This will leave iSrcDatalen bytes on the hunk stack (since you can't dealloc that), but MP3 files are
                //	usually small, and like I say, it should never happen.
                //
                // Strictly speaking, I should do a Z_Malloc above, then I could do a Z_Free if failed, else do a Hunk_Alloc
                //	to copy the Z_Malloc data into, then Z_Free, but for something that shouldn't happen it seemed bad to
                //	penalise the rest of the game with extra alloc demands.
                //
                return false;
            }

            // success ( ...on a plate).
            //
            // make a copy of the filled-in stream struct and attach to the sfx_t struct...
            //
            // sfx->pMP3StreamHeader = (MP3STREAM *) Z_Malloc( sizeof(MP3STREAM), TAG_SND_MP3STREAMHDR, qfalse );
            // memcpy(	sfx->pMP3StreamHeader, &SFX_MP3Stream,		    sizeof(MP3STREAM) );
            //
            return true;
        }

        false
    }
}



// decode one packet of MP3 data only (typical output size is 2304, or 2304*2 for stereo, so input size is less
//
// return is decoded byte count, else 0 for finished
//
pub fn MP3Stream_Decode(lpMP3Stream: LP_MP3STREAM, bDoingMusic: bool) -> c_int {
    unsafe {
        // lpMP3Stream->iCopyOffset = 0;  -- field setting elided

        if false { //!bDoingMusic)
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
            return C_MP3Stream_Decode(lpMP3Stream, 0);	// bFastForwarding = false
        }
    }
    0
}


pub fn MP3Stream_SeekTo(ch: *mut channel_t, fTimeToSeekTo: f32) -> bool {
    unsafe {
        const fEpsilon: f32 = 0.05f32;	// accurate to 1/50 of a second, but plus or minus this gives 1/10 of second

        MP3Stream_Rewind(ch);
        //
        // sanity... :-)
        //
        let fTrackLengthInSeconds = MP3Stream_GetPlayingTimeInSeconds(&mut (*ch).MP3StreamHeader as *mut MP3STREAM);
        let mut fTimeToSeekTo_mut = fTimeToSeekTo;
        if fTimeToSeekTo_mut > fTrackLengthInSeconds {
            fTimeToSeekTo_mut = fTrackLengthInSeconds;
        }

        // now do the seek...
        //
        loop {
            let fPlayingTimeElapsed = MP3Stream_GetPlayingTimeInSeconds(&mut (*ch).MP3StreamHeader as *mut MP3STREAM) - MP3Stream_GetRemainingTimeInSeconds(&mut (*ch).MP3StreamHeader as *mut MP3STREAM);
            let fAbsTimeDiff = (fTimeToSeekTo_mut - fPlayingTimeElapsed).abs();

            if fAbsTimeDiff <= fEpsilon {
                return true;
            }

            // when decoding, use fast-forward until within 3 seconds, then slow-decode (which should init stuff properly?)...
            //
            let iBytesDecodedThisPacket = C_MP3Stream_Decode(&mut (*ch).MP3StreamHeader as *mut MP3STREAM, if fAbsTimeDiff > 3.0f32 { 1 } else { 0 });	// bFastForwarding
            if iBytesDecodedThisPacket == 0 {
                break;	// EOS
            }
        }

        false
    }
}


// returns qtrue for all ok
//
pub fn MP3Stream_Rewind(ch: *mut channel_t) -> bool {
    unsafe {
        // ch->iMP3SlidingDecodeWritePos = 0;
        // ch->iMP3SlidingDecodeWindowPos= 0;

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
        // memcpy(&ch->MP3StreamHeader, ch->thesfx->pMP3StreamHeader, sizeof(ch->MP3StreamHeader));
        true
    }
}


// returns qtrue while still playing normally, else qfalse for either finished or request-offset-error
//
pub fn MP3Stream_GetSamples(ch: *mut channel_t, startingSampleNum: c_int, count: c_int, buf: *mut i16, bStereo: bool) -> bool {
    unsafe {
        let mut qbStreamStillGoing = true;

        // const int iQuarterOfSlidingBuffer		=  sizeof(ch->MP3SlidingDecodeBuffer)/4;
        // const int iThreeQuartersOfSlidingBuffer	= (sizeof(ch->MP3SlidingDecodeBuffer)*3)/4;

        //	Com_Printf("startingSampleNum %d\n",startingSampleNum);

        let mut count_mut = count * 2/* <- = SOF2; ch->sfx->width*/;	// count arg was for words, so double it for bytes;

        // convert sample number into a byte offset... (make new variable for clarity?)
        //
        let mut startingSampleNum_mut = startingSampleNum * 2 /* <- = SOF2; ch->sfx->width*/ * (if bStereo { 2 } else { 1 });

        if startingSampleNum_mut < 0 { // stub: would be ch->iMP3SlidingDecodeWindowPos
            // what?!?!?!   smegging time travel needed or something?, forget it
            core::ptr::write_bytes(buf, 0, count_mut as usize);
            return false;
        }

        //	OutputDebugString(va("\nRequest: startingSampleNum %d, count %d\n",startingSampleNum,count));
        //	OutputDebugString(va("WindowPos %d, WindowWritePos %d\n",ch->iMP3SlidingDecodeWindowPos,ch->iMP3SlidingDecodeWritePos));

        //	qboolean _bDecoded = qfalse;

        loop {
            if !(
                (startingSampleNum_mut >= 0) // stub: would be ch->iMP3SlidingDecodeWindowPos
                &&
                (startingSampleNum_mut + count_mut < 0 + 0) // stub: would use actual buffer positions
            ) {
                //		if (!_bDecoded)
                //		{
                //			Com_Printf(S_COLOR_YELLOW"Decode needed!\n");
                //		}
                //		_bDecoded = qtrue;
                //		OutputDebugString("Scrolling...");

                let _iBytesDecoded = MP3Stream_Decode(&mut (*ch).MP3StreamHeader as *mut MP3STREAM, bStereo);	// stereo only for music, so this is safe
                //		OutputDebugString(va("%d bytes decoded\n",_iBytesDecoded));
                if _iBytesDecoded == 0 {
                    // no more source data left so clear the remainder of the buffer...
                    //
                    // memset(ch->MP3SlidingDecodeBuffer + ch->iMP3SlidingDecodeWritePos, 0, sizeof(ch->MP3SlidingDecodeBuffer)-ch->iMP3SlidingDecodeWritePos);
                    //			OutputDebugString("Finished\n");
                    qbStreamStillGoing = false;
                    break;
                } else {
                    // memcpy(ch->MP3SlidingDecodeBuffer + ch->iMP3SlidingDecodeWritePos,ch->MP3StreamHeader.bDecodeBuffer,_iBytesDecoded);
                    // ch->iMP3SlidingDecodeWritePos += _iBytesDecoded;

                    // if reached 3/4 of buffer pos, backscroll the decode window by one quarter...
                    //
                    // if (ch->iMP3SlidingDecodeWritePos > iThreeQuartersOfSlidingBuffer)
                    // {
                    // 	memmove(ch->MP3SlidingDecodeBuffer, ((byte *)ch->MP3SlidingDecodeBuffer + iQuarterOfSlidingBuffer), iThreeQuartersOfSlidingBuffer);
                    // 	ch->iMP3SlidingDecodeWritePos -= iQuarterOfSlidingBuffer;
                    // 	ch->iMP3SlidingDecodeWindowPos+= iQuarterOfSlidingBuffer;
                    // }
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

        // assert(startingSampleNum >= ch->iMP3SlidingDecodeWindowPos);
        // memcpy( buf, ch->MP3SlidingDecodeBuffer + (startingSampleNum-ch->iMP3SlidingDecodeWindowPos), count);

        //	OutputDebugString("OK\n\n");

        qbStreamStillGoing
    }
}


///////////// eof /////////////
