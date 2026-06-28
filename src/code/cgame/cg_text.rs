// cg_text.c --

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"
// #include "cg_media.h"

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

//int precacheWav_i;	// Current high index of precacheWav array
//precacheWav_t precacheWav[MAX_PRECACHEWAV];

//int precacheText_i;	// Current high index of precacheText array
//precacheText_t precacheText[MAX_PRECACHETEXT];

extern "C" {
    pub static mut textcolor_caption: [f32; 4];
    pub static mut textcolor_center: [f32; 4];
    pub static mut textcolor_scroll: [f32; 4];

    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static mut cg_developer: cvar_t;
    pub static mut cg_skippingcin: cvar_t;
    pub static mut in_camera: c_int;
    pub static mut client_camera: client_camera_t;

    // Graphics/Font functions
    pub fn cgi_R_SetColor(rgba: *const [f32; 4]);
    pub fn cgi_R_Font_HeightPixels(fontHandle: c_int, scale: f32) -> c_int;
    pub fn cgi_AnyLanguage_ReadCharFromString(
        pbStr: *const u8,
        piAdvanceCount: *mut c_int,
        pbIsTrailingPunctuation: *mut u8,
    ) -> u32;
    pub fn cgi_R_Font_StrLenPixels(text: *const u8, fontHandle: c_int, scale: f32) -> c_int;
    pub fn cgi_Language_UsesSpaces() -> u8;
    pub fn cgi_R_Font_DrawString(
        x: c_int,
        y: c_int,
        text: *const u8,
        rgba: *const [f32; 4],
        fontHandle: c_int,
        iMaxPixelWidth: c_int,
        scale: f32,
    );
    pub fn cgi_S_GetSampleLength(sfxHandle: c_int) -> c_int;
    pub fn cgi_Language_IsAsian() -> u8;
    pub fn cgi_SP_GetStringTextString(
        psReference: *const u8,
        psDest: *mut u8,
        iSizeofDest: c_int,
    ) -> c_int;
    pub fn cgi_Z_Malloc(size: c_int, tag: c_int) -> *mut c_void;
    pub fn cgi_Z_Free(ptr: *mut c_void);

    // String/Printf functions
    pub fn Com_Printf(fmt: *const u8, ...);
    pub fn Q_strcat(dest: *mut u8, size: c_int, src: *const u8);
    pub fn va(fmt: *const u8, ...) -> *const u8;
    pub fn strlen(s: *const u8) -> usize;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn strrchr(s: *const u8, c: c_int) -> *const u8;
    pub fn strnicmp(s1: *const u8, s2: *const u8, n: usize) -> c_int;
    pub fn Q_strncpyz(dest: *mut u8, src: *const u8, size: c_int);

    pub fn Vector4Copy(src: *const [f32; 4], dest: *mut [f32; 4]);
    pub fn CG_FadeColor(startTime: c_int, totalTime: c_int) -> *mut [f32; 4];

    pub static mut colorTable: [[f32; 4]; 32];
}

// Placeholder struct definitions for types used in this module
#[repr(C)]
pub struct cg_t {
    pub captionTextTime: c_int,
    pub captionTextY: c_int,
    pub captionTextCurrentLine: c_int,
    pub scrollTextLines: c_int,
    pub captionText: [[u8; 256]; 20],
    pub captionLetterTime: c_int,
    pub captionNextTextTime: c_int,
    pub time: c_int,
    pub printTextY: c_int,
    pub printText: [[u8; 256]; 50],
    pub scrollTextTime: c_int,
    pub centerPrint: [u8; 1024],
    pub centerPrintTime: c_int,
    pub centerPrintY: c_int,
    pub centerPrintLines: c_int,
    pub snap: *mut c_void,
}

#[repr(C)]
pub struct cgs_t {
    pub stripLevelName: [[u8; 256]; 3],
    pub media: media_t,
}

#[repr(C)]
pub struct media_t {
    pub qhFontMedium: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

#[repr(C)]
pub struct client_camera_t {
    pub bar_height_dest: c_int,
}

// this is execrable, and should NOT have had to've been done now, but...
//
static mut gfAdvanceHack: f32 = 0.0f32; // MUST default to this
static mut giLinesOutput: c_int = 0; // hack-city after release, only used by one function

// display text in a supplied box, start at top left and going down by however many pixels I feel like internally,
//	return value is NULL if all fitted, else char * of next char to continue from that didn't fit.
//
// (coords are in the usual 640x480 virtual space)...
//
// ( if you get the same char * returned as what you passed in, then none of it fitted at all (box too small) )
//
pub unsafe extern "C" fn CG_DisplayBoxedText(
    iBoxX: c_int,
    iBoxY: c_int,
    iBoxWidth: c_int,
    iBoxHeight: c_int,
    psText: *const u8,
    iFontHandle: c_int,
    fScale: f32,
    v4Color: *const [f32; 4],
) -> *const u8 {
    *addr_of_mut!(giLinesOutput) = 0;
    cgi_R_SetColor(v4Color);

    // Setup a reasonable vertical spacing (taiwanese & japanese need 1.5 fontheight, so use that for all)...
    //
    let iFontHeight = cgi_R_Font_HeightPixels(iFontHandle, fScale);
    let iFontHeightAdvance = ((if *addr_of!(gfAdvanceHack) == 0.0f32 { 1.5f32 } else { *addr_of!(gfAdvanceHack) }) * (iFontHeight as f32)) as c_int;
    let mut iYpos = iBoxY; // start print pos

    // this could probably be simplified now, but it was converted from something else I didn't originally write,
    //	and it works anyway so wtf...
    //
    let mut psCurrentTextReadPos = psText;
    let mut psReadPosAtLineStart = psCurrentTextReadPos;
    let mut psBestLineBreakSrcPos = psCurrentTextReadPos;
    while *psCurrentTextReadPos != 0 && iYpos + iFontHeight < (iBoxY + iBoxHeight) {
        let mut sLineForDisplay: [u8; 2048] = [0; 2048]; // ott

        // construct a line...
        //
        psCurrentTextReadPos = psReadPosAtLineStart;
        sLineForDisplay[0] = 0;
        while *psCurrentTextReadPos != 0 {
            let psLastGood_s = psCurrentTextReadPos; // needed if we get a full screen of chars with no punctuation or space (see usage notes)

            // read letter...
            //
            let mut bIsTrailingPunctuation: u8 = 0;
            let mut iAdvanceCount: c_int = 0;
            let uiLetter =
                cgi_AnyLanguage_ReadCharFromString(psCurrentTextReadPos, &mut iAdvanceCount, &mut bIsTrailingPunctuation);
            psCurrentTextReadPos = psCurrentTextReadPos.add(iAdvanceCount as usize);

            // concat onto string so far...
            //
            if uiLetter == 32 && sLineForDisplay[0] as u8 == 0 {
                psReadPosAtLineStart = psReadPosAtLineStart.add(1);
                continue; // unless it's a space at the start of a line, in which case ignore it.
            }

            if uiLetter > 255 {
                Q_strcat(
                    sLineForDisplay.as_mut_ptr(),
                    sLineForDisplay.len() as c_int,
                    va(
                        "%c%c\0".as_ptr(),
                        ((uiLetter >> 8) & 0xFF) as u8,
                        (uiLetter & 0xFF) as u8,
                    ),
                );
            } else {
                Q_strcat(
                    sLineForDisplay.as_mut_ptr(),
                    sLineForDisplay.len() as c_int,
                    va("%c\0".as_ptr(), (uiLetter & 0xFF) as u8),
                );
            }

            if uiLetter == '\n' as u32 {
                // explicit new line...
                //
                sLineForDisplay[(strlen(sLineForDisplay.as_ptr())) - 1] = 0; // kill the CR
                psReadPosAtLineStart = psCurrentTextReadPos;
                psBestLineBreakSrcPos = psCurrentTextReadPos;
                break; // print this line
            } else if cgi_R_Font_StrLenPixels(sLineForDisplay.as_ptr(), iFontHandle, fScale) >= iBoxWidth {
                // reached screen edge, so cap off string at bytepos after last good position...
                //
                if uiLetter > 255 && bIsTrailingPunctuation != 0 && cgi_Language_UsesSpaces() == 0 {
                    // Special case, don't consider line breaking if you're on an asian punctuation char of
                    //	a language that doesn't use spaces...
                    //
                } else {
                    if psBestLineBreakSrcPos == psReadPosAtLineStart {
                        //  aarrrggh!!!!!   we'll only get here is someone has fed in a (probably) garbage string,
                        //		since it doesn't have a single space or punctuation mark right the way across one line
                        //		of the screen.  So far, this has only happened in testing when I hardwired a taiwanese
                        //		string into this function while the game was running in english (which should NEVER happen
                        //		normally).  On the other hand I suppose it'psCurrentTextReadPos entirely possible that some taiwanese string
                        //		might have no punctuation at all, so...
                        //
                        psBestLineBreakSrcPos = psLastGood_s; // force a break after last good letter
                    }

                    sLineForDisplay[(psBestLineBreakSrcPos as usize - psReadPosAtLineStart as usize)] = 0;
                    psReadPosAtLineStart = psCurrentTextReadPos;
                    psCurrentTextReadPos = psBestLineBreakSrcPos;
                    break; // print this line
                }
            }

            // record last-good linebreak pos...  (ie if we've just concat'd a punctuation point (western or asian) or space)
            //
            if bIsTrailingPunctuation != 0 || uiLetter == ' ' as u32 || (uiLetter > 255 && cgi_Language_UsesSpaces() == 0) {
                psBestLineBreakSrcPos = psCurrentTextReadPos;
            }
        }

        // ... and print it...
        //
        cgi_R_Font_DrawString(iBoxX, iYpos, sLineForDisplay.as_ptr(), v4Color, iFontHandle, -1, fScale);
        iYpos += iFontHeightAdvance;
        *addr_of_mut!(giLinesOutput) += 1;

        // and echo to console in dev mode...
        //
        if (*addr_of!(cg_developer)).integer != 0 {
            //			Com_Printf( "%psCurrentTextReadPos\n", sLineForDisplay );
        }
    }
    psReadPosAtLineStart
}

/*
===============================================================================

CAPTION TEXT

===============================================================================
*/
pub unsafe extern "C" fn CG_CaptionTextStop() {
    (*addr_of_mut!(cg)).captionTextTime = 0;
}

// try and get the correct StripEd text (with retry) for a given reference...
//
// returns 0 if failed, else strlen...
//
unsafe extern "C" fn cg_SP_GetStringTextStringWithRetry(psReference: *const u8, psDest: *mut u8, iSizeofDest: c_int) -> c_int {
    let mut iReturn: c_int;

    if *psReference as u8 as char == '#' {
        // then we know the striped package name is already built in, so do NOT try prepending anything else...
        //
        return cgi_SP_GetStringTextString(va("%s\0".as_ptr(), psReference.add(1)), psDest, iSizeofDest);
    }

    let STRIPED_LEVELNAME_VARIATIONS = 3;
    for i in 0..STRIPED_LEVELNAME_VARIATIONS {
        if (*addr_of!(cgs)).stripLevelName[i][0] as u8 != 0 {
            // entry present?
            iReturn = cgi_SP_GetStringTextString(
                va("%s_%s\0".as_ptr(), (*addr_of!(cgs)).stripLevelName[i].as_ptr(), psReference),
                psDest,
                iSizeofDest,
            );
            if iReturn != 0 {
                return iReturn;
            }
        }
    }

    0
}

// slightly confusingly, the char arg for this function is an audio filename of the form "path/path/filename",
//	the "filename" part of which should be the same as the StripEd reference we're looking for in the current
//	level's string package...
//
pub unsafe extern "C" fn CG_CaptionText(str: *const u8, sound: c_int, y: c_int) {
    let mut s: *const u8;
    let mut holds: *const u8;
    let mut i: c_int;
    let mut holdTime: c_int;
    let mut text: [u8; 8192] = [0; 8192];

    let fFontScale = if cgi_Language_IsAsian() != 0 { 0.8f32 } else { 1.0f32 };

    holds = strrchr(str, '/' as c_int);
    if holds.is_null() {
        #[cfg(not(feature = "FINAL_BUILD"))]
        Com_Printf("WARNING: CG_CaptionText given audio filename with no '/':'%s'\n\0".as_ptr(), str);
        return;
    }
    i = cg_SP_GetStringTextStringWithRetry(holds.add(1), text.as_mut_ptr(), text.len() as c_int);
    //ensure we found a match
    if i == 0 {
        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            // we only care about some sound dirs...
            if strnicmp(str, "sound/chars/\0".as_ptr(), 12) == 0 {
                // whichever language it is, it'll be pathed as english at this point
                Com_Printf("WARNING: CG_CaptionText given invalid text key :'%s'\n\0".as_ptr(), str);
            } else {
                // anything else is probably stuff we don't care about. It certainly shouldn't be speech, anyway
            }
        }
        return;
    }

    let fontHeight = ((if cgi_Language_IsAsian() != 0 { 1.4f32 } else { 1.0f32 })
        * (cgi_R_Font_HeightPixels((*addr_of!(cgs)).media.qhFontMedium, fFontScale) as f32)) as c_int; // taiwanese & japanese need 1.5 fontheight spacing

    (*addr_of_mut!(cg)).captionTextTime = (*addr_of!(cg)).time;
    if in_camera != 0 {
        (*addr_of_mut!(cg)).captionTextY = 480 - ((*addr_of!(client_camera)).bar_height_dest / 2); // ths is now a centre'd Y, not a start Y
    } else {
        //get above the hud
        (*addr_of_mut!(cg)).captionTextY =
            (0.88f32 * ((480 as f32) - (fontHeight as f32) * 1.5f32)) as c_int; // do NOT move this, it has to fit in between the weapon HUD and the datapad update.
    }
    (*addr_of_mut!(cg)).captionTextCurrentLine = 0;

    // count the number of lines for centering
    (*addr_of_mut!(cg)).scrollTextLines = 1;

    memset(
        (*addr_of_mut!(cg)).captionText.as_mut_ptr() as *mut c_void,
        0,
        std::mem::size_of_val(&(*addr_of!(cg)).captionText),
    );

    // Break into individual lines
    i = 0; // this could be completely removed and replace by "cg.scrollTextLines-1", but wtf?

    s = text.as_ptr() as *const u8;
    // tai...
    //	s="…(truncated - contains unportable asian characters - see original)
    // kor...
    //	s="…(truncated - contains unportable korean characters - see original)
    holds = s;

    let iPlayingTimeMS = cgi_S_GetSampleLength(sound);
    let mut iLengthInChars = strlen(s) as c_int; //cgi_R_Font_StrLenChars(s);	// strlen is also good for MBCS in this instance, since it's for timing
    if iLengthInChars == 0 {
        iLengthInChars = 1;
    }
    (*addr_of_mut!(cg)).captionLetterTime = iPlayingTimeMS / iLengthInChars;

    let mut psBestLineBreakSrcPos = s;
    while *s != 0 {
        let psLastGood_s = s; // needed if we get a full screen of chars with no punctuation or space (see usage notes)

        // read letter...
        //
        let mut bIsTrailingPunctuation: u8 = 0;
        let mut iAdvanceCount: c_int = 0;
        let uiLetter = cgi_AnyLanguage_ReadCharFromString(s, &mut iAdvanceCount, &mut bIsTrailingPunctuation);
        s = s.add(iAdvanceCount as usize);

        // concat onto string so far...
        //
        if uiLetter == 32 && (*addr_of!(cg)).captionText[i as usize][0] as u8 == 0 {
            holds = holds.add(1);
            continue; // unless it's a space at the start of a line, in which case ignore it.
        }

        if uiLetter > 255 {
            Q_strcat(
                (*addr_of_mut!(cg)).captionText[i as usize].as_mut_ptr(),
                std::mem::size_of_val(&(*addr_of!(cg)).captionText[0]) as c_int,
                va(
                    "%c%c\0".as_ptr(),
                    ((uiLetter >> 8) & 0xFF) as u8,
                    (uiLetter & 0xFF) as u8,
                ),
            );
        } else {
            Q_strcat(
                (*addr_of_mut!(cg)).captionText[i as usize].as_mut_ptr(),
                std::mem::size_of_val(&(*addr_of!(cg)).captionText[0]) as c_int,
                va("%c\0".as_ptr(), (uiLetter & 0xFF) as u8),
            );
        }

        if uiLetter == '\n' as u32 {
            // explicit new line...
            //
            (*addr_of_mut!(cg)).captionText[i as usize][(strlen((*addr_of!(cg)).captionText[i as usize].as_ptr())) - 1] = 0; // kill the CR
            i += 1;
            holds = s;
            psBestLineBreakSrcPos = s;
            (*addr_of_mut!(cg)).scrollTextLines += 1;
        } else if cgi_R_Font_StrLenPixels(
            (*addr_of!(cg)).captionText[i as usize].as_ptr(),
            (*addr_of!(cgs)).media.qhFontMedium,
            fFontScale,
        ) >= 640
        {
            // reached screen edge, so cap off string at bytepos after last good position...
            //
            if uiLetter > 255 && bIsTrailingPunctuation != 0 && cgi_Language_UsesSpaces() == 0 {
                // Special case, don't consider line breaking if you're on an asian punctuation char of
                //	a language that doesn't use spaces...
                //
            } else {
                if psBestLineBreakSrcPos == holds {
                    //  aarrrggh!!!!!   we'll only get here is someone has fed in a (probably) garbage string,
                    //		since it doesn't have a single space or punctuation mark right the way across one line
                    //		of the screen.  So far, this has only happened in testing when I hardwired a taiwanese
                    //		string into this function while the game was running in english (which should NEVER happen
                    //		normally).  On the other hand I suppose it's entirely possible that some taiwanese string
                    //		might have no punctuation at all, so...
                    //
                    psBestLineBreakSrcPos = psLastGood_s; // force a break after last good letter
                }

                (*addr_of_mut!(cg)).captionText[i as usize][(psBestLineBreakSrcPos as usize - holds as usize)] = 0;
                holds = s;
                s = psBestLineBreakSrcPos;
                i += 1;
                (*addr_of_mut!(cg)).scrollTextLines += 1;
            }
        }

        // record last-good linebreak pos...  (ie if we've just concat'd a punctuation point (western or asian) or space)
        //
        if bIsTrailingPunctuation != 0 || uiLetter == ' ' as u32 || (uiLetter > 255 && cgi_Language_UsesSpaces() == 0) {
            psBestLineBreakSrcPos = s;
        }
    }

    // calc the length of time to hold each 2 lines of text on the screen.... presumably this works?
    //
    holdTime = strlen((*addr_of!(cg)).captionText[0].as_ptr()) as c_int;
    if (*addr_of!(cg)).scrollTextLines > 1 {
        holdTime += strlen((*addr_of!(cg)).captionText[1].as_ptr()) as c_int; // strlen is also good for MBCS in this instance, since it's for timing
    }
    (*addr_of_mut!(cg)).captionNextTextTime = (*addr_of!(cg)).time + (holdTime * (*addr_of!(cg)).captionLetterTime);

    (*addr_of_mut!(cg)).scrollTextTime = 0; // No scrolling during captions

    //Echo to console in dev mode
    if (*addr_of!(cg_developer)).integer != 0 {
        Com_Printf(
            "%s\n\0".as_ptr(),
            (*addr_of!(cg)).captionText[0].as_ptr(),
        ); // ste:  was [i], but surely sentence 0 is more useful than last?
    }
}

pub unsafe extern "C" fn CG_DrawCaptionText() {
    let i: c_int;
    let x: c_int;
    let mut y: c_int;
    let w: c_int;
    let holdTime: c_int;

    if (*addr_of!(cg)).captionTextTime == 0 {
        return;
    }

    let fFontScale = if cgi_Language_IsAsian() != 0 { 0.8f32 } else { 1.0f32 };

    if (*addr_of!(cg_skippingcin)).integer != 0 {
        (*addr_of_mut!(cg)).captionTextTime = 0;
        return;
    }

    if (*addr_of!(cg)).captionNextTextTime < (*addr_of!(cg)).time {
        (*addr_of_mut!(cg)).captionTextCurrentLine += 2;

        if (*addr_of!(cg)).captionTextCurrentLine >= (*addr_of!(cg)).scrollTextLines {
            (*addr_of_mut!(cg)).captionTextTime = 0;
            return;
        } else {
            let holdTime = strlen((*addr_of!(cg)).captionText[(*addr_of!(cg)).captionTextCurrentLine as usize].as_ptr()) as c_int;
            if (*addr_of!(cg)).scrollTextLines >= (*addr_of!(cg)).captionTextCurrentLine {
                // ( strlen is also good for MBCS in this instance, since it's for timing -ste)
                //
                let holdTime2 = strlen((*addr_of!(cg)).captionText[((*addr_of!(cg)).captionTextCurrentLine + 1) as usize].as_ptr()) as c_int;
                (*addr_of_mut!(cg)).captionNextTextTime = (*addr_of!(cg)).time + ((holdTime + holdTime2) * (*addr_of!(cg)).captionLetterTime);
            } else {
                (*addr_of_mut!(cg)).captionNextTextTime = (*addr_of!(cg)).time + (holdTime * (*addr_of!(cg)).captionLetterTime);
            }
        }
    }

    // Give a color if one wasn't given
    if (textcolor_caption[0] == 0.0f32)
        && (textcolor_caption[1] == 0.0f32)
        && (textcolor_caption[2] == 0.0f32)
        && (textcolor_caption[3] == 0.0f32)
    {
        Vector4Copy(addr_of!(colorTable[7]), addr_of_mut!(textcolor_caption)); // CT_WHITE = 7
    }

    cgi_R_SetColor(addr_of!(textcolor_caption));

    // Set Y of the first line (varies if only printing one line of text)
    // (this all works, please don't mess with it)
    let fontHeight = ((if cgi_Language_IsAsian() != 0 { 1.4f32 } else { 1.0f32 })
        * (cgi_R_Font_HeightPixels((*addr_of!(cgs)).media.qhFontMedium, fFontScale) as f32)) as c_int;
    let bPrinting2Lines = (*addr_of!(cg)).captionText[((*addr_of!(cg)).captionTextCurrentLine + 1) as usize][0] as u8 != 0;
    y = (*addr_of!(cg)).captionTextY - ((fontHeight as f32) * (if bPrinting2Lines { 1.0f32 } else { 0.5f32 })) as c_int; // captionTextY was a centered Y pos, not a top one
    y -= if cgi_Language_IsAsian() != 0 { 0 } else { 4 };

    for i_iter in 0..2 {
        let i_idx = (*addr_of!(cg)).captionTextCurrentLine + i_iter;
        if i_idx < (*addr_of!(cg)).scrollTextLines {
            let w = cgi_R_Font_StrLenPixels(
                (*addr_of!(cg)).captionText[i_idx as usize].as_ptr(),
                (*addr_of!(cgs)).media.qhFontMedium,
                fFontScale,
            );
            if w != 0 {
                let x = (640 - w) / 2;
                cgi_R_Font_DrawString(
                    x,
                    y,
                    (*addr_of!(cg)).captionText[i_idx as usize].as_ptr(),
                    addr_of!(textcolor_caption),
                    (*addr_of!(cgs)).media.qhFontMedium,
                    -1,
                    fFontScale,
                );
                y += fontHeight;
            }
        }
    }

    cgi_R_SetColor(std::ptr::null());
}

/*
===============================================================================

SCROLL TEXT

===============================================================================

CG_ScrollText - split text up into seperate lines

 'str' arg is StripEd string reference, eg "CREDITS_RAVEN"

*/
pub static mut giScrollTextPixelWidth: c_int = 640; // SCREEN_WIDTH
pub unsafe extern "C" fn CG_ScrollText(str: *const u8, iPixelWidth: c_int) {
    giScrollTextPixelWidth = iPixelWidth;

    // first, ask the strlen of the final string...
    //
    let i_len = cgi_SP_GetStringTextString(str, std::ptr::null_mut(), 0);

    //ensure we found a match
    if i_len == 0 {
        #[cfg(not(feature = "FINAL_BUILD"))]
        Com_Printf("WARNING: CG_ScrollText given invalid text key :'%s'\n\0".as_ptr(), str);
        return;
    }
    //
    // malloc space to hold it...
    //
    let psText = cgi_Z_Malloc(i_len + 1, 4) as *mut u8; // TAG_TEMP_WORKSPACE = 4
    //
    // now get the string...
    //
    let i_result = cgi_SP_GetStringTextString(str, psText, i_len + 1);
    //ensure we found a match
    if i_result == 0 {
        //assert(0);	// should never get here now, but wtf?
        cgi_Z_Free(psText as *mut c_void);
        #[cfg(not(feature = "FINAL_BUILD"))]
        Com_Printf("WARNING: CG_ScrollText given invalid text key :'%s'\n\0".as_ptr(), str);
        return;
    }

    (*addr_of_mut!(cg)).scrollTextTime = (*addr_of!(cg)).time;
    (*addr_of_mut!(cg)).printTextY = 480; // SCREEN_HEIGHT = 480
    (*addr_of_mut!(cg)).scrollTextLines = 1;

    let mut s = psText;
    let mut i = 0;
    let mut holds = s;

    let mut psBestLineBreakSrcPos = s;
    while *s != 0 {
        let psLastGood_s = s; // needed if we get a full screen of chars with no punctuation or space (see usage notes)

        // read letter...
        //
        let mut bIsTrailingPunctuation: u8 = 0;
        let mut iAdvanceCount: c_int = 0;
        let uiLetter = cgi_AnyLanguage_ReadCharFromString(s, &mut iAdvanceCount, &mut bIsTrailingPunctuation);
        s = s.add(iAdvanceCount as usize);

        // concat onto string so far...
        //
        if uiLetter == 32 && (*addr_of!(cg)).printText[i as usize][0] as u8 == 0 {
            holds = holds.add(1);
            continue; // unless it's a space at the start of a line, in which case ignore it.
        }

        if uiLetter > 255 {
            Q_strcat(
                (*addr_of_mut!(cg)).printText[i as usize].as_mut_ptr(),
                std::mem::size_of_val(&(*addr_of!(cg)).printText[0]) as c_int,
                va(
                    "%c%c\0".as_ptr(),
                    ((uiLetter >> 8) & 0xFF) as u8,
                    (uiLetter & 0xFF) as u8,
                ),
            );
        } else {
            Q_strcat(
                (*addr_of_mut!(cg)).printText[i as usize].as_mut_ptr(),
                std::mem::size_of_val(&(*addr_of!(cg)).printText[0]) as c_int,
                va("%c\0".as_ptr(), (uiLetter & 0xFF) as u8),
            );
        }

        // record last-good linebreak pos...  (ie if we've just concat'd a punctuation point (western or asian) or space)
        //
        if bIsTrailingPunctuation != 0 || uiLetter == ' ' as u32 {
            psBestLineBreakSrcPos = s;
        }

        if uiLetter == '\n' as u32 {
            // explicit new line...
            //
            (*addr_of_mut!(cg)).printText[i as usize][(strlen((*addr_of!(cg)).printText[i as usize].as_ptr())) - 1] = 0; // kill the CR
            i += 1;
            //assert (i < (sizeof(cg.printText)/sizeof(cg.printText[0])) );
            if i >= 50 {
                // 50 = MAX_PRINTTEXT
                break;
            }
            holds = s;
            (*addr_of_mut!(cg)).scrollTextLines += 1;
        } else if cgi_R_Font_StrLenPixels(
            (*addr_of!(cg)).printText[i as usize].as_ptr(),
            (*addr_of!(cgs)).media.qhFontMedium,
            1.0f32,
        ) >= iPixelWidth
        {
            // reached screen edge, so cap off string at bytepos after last good position...
            //
            if psBestLineBreakSrcPos == holds {
                //  aarrrggh!!!!!   we'll only get here is someone has fed in a (probably) garbage string,
                //		since it doesn't have a single space or punctuation mark right the way across one line
                //		of the screen.  So far, this has only happened in testing when I hardwired a taiwanese
                //		string into this function while the game was running in english (which should NEVER happen
                //		normally).  On the other hand I suppose it's entirely possible that some taiwanese string
                //		might have no punctuation at all, so...
                //
                psBestLineBreakSrcPos = psLastGood_s; // force a break after last good letter
            }

            (*addr_of_mut!(cg)).printText[i as usize][(psBestLineBreakSrcPos as usize - holds as usize)] = 0;
            holds = s;
            s = psBestLineBreakSrcPos;
            i += 1;
            //assert (i < (sizeof(cg.printText)/sizeof(cg.printText[0])) );
            (*addr_of_mut!(cg)).scrollTextLines += 1;
        }
    }

    (*addr_of_mut!(cg)).captionTextTime = 0; // No captions during scrolling
    cgi_Z_Free(psText as *mut c_void);
}

// draws using [textcolor_scroll]...
//
const SCROLL_LPM: f32 = 1.0f32 / 50.0f32; // 1 line per 50 ms
pub unsafe extern "C" fn CG_DrawScrollText() {
    let mut i: c_int;
    let mut x: c_int;
    let mut y: c_int;
    let fontHeight =
        ((1.5f32 * (cgi_R_Font_HeightPixels((*addr_of!(cgs)).media.qhFontMedium, 1.0f32) as f32)) as c_int); // taiwanese & japanese need 1.5 fontheight spacing

    if (*addr_of!(cg)).scrollTextTime == 0 {
        return;
    }

    cgi_R_SetColor(addr_of!(textcolor_scroll));

    y = (*addr_of!(cg)).printTextY - (((*addr_of!(cg)).time - (*addr_of!(cg)).scrollTextTime) as f32 * SCROLL_LPM) as c_int;

    //	cgi_R_Font_DrawString(320, 200, va("Scrolltext printing @ %d",y), colorTable[CT_LTGOLD1], cgs.media.qhFontMedium, -1, 1.0f);

    // See if text has finished scrolling off screen
    if (y + (*addr_of!(cg)).scrollTextLines * fontHeight) < 1 {
        (*addr_of_mut!(cg)).scrollTextTime = 0;
        return;
    }

    i = 0;
    while i < (*addr_of!(cg)).scrollTextLines {
        // Is this line off top of screen?
        if (y + ((i + 1) * fontHeight)) < 1 {
            y += fontHeight;
            i += 1;
            continue;
        }
        // or past bottom of screen?
        else if y > 480 {
            // SCREEN_HEIGHT = 480
            break;
        }

        let start = (*addr_of!(cg)).printText[i as usize].as_mut_ptr();

        //		w = cgi_R_Font_StrLenPixels(cg.printText[i], cgs.media.qhFontMedium, 1.0f);
        //		if (w)
        {
            x = (640 - giScrollTextPixelWidth) / 2; // SCREEN_WIDTH = 640
            cgi_R_Font_DrawString(
                x,
                y,
                (*addr_of!(cg)).printText[i as usize].as_ptr(),
                addr_of!(textcolor_scroll),
                (*addr_of!(cgs)).media.qhFontMedium,
                -1,
                1.0f32,
            );
            y += fontHeight;
        }
        i += 1;
    }

    cgi_R_SetColor(std::ptr::null());
}

/*
===============================================================================

CENTER PRINTING

===============================================================================
*/

/*
==============
CG_CenterPrint

Called for important messages that should stay in the center of the screen
for a few moments
==============
*/
pub unsafe extern "C" fn CG_CenterPrint(str: *const u8, y: c_int) {
    // Find text to match the str given
    if *str as u8 as char == '@' {
        let i = cgi_SP_GetStringTextString(str.add(1), (*addr_of_mut!(cg)).centerPrint.as_mut_ptr(), std::mem::size_of_val(&(*addr_of!(cg)).centerPrint) as c_int);

        if i == 0 {
            Com_Printf(
                "CG_CenterPrint: cannot find reference '%s' in StringPackage!\n\0".as_ptr(),
                str,
            );
            Q_strncpyz(
                (*addr_of_mut!(cg)).centerPrint.as_mut_ptr(),
                str,
                std::mem::size_of_val(&(*addr_of!(cg)).centerPrint) as c_int,
            );
        }
    } else {
        Q_strncpyz(
            (*addr_of_mut!(cg)).centerPrint.as_mut_ptr(),
            str,
            std::mem::size_of_val(&(*addr_of!(cg)).centerPrint) as c_int,
        );
    }

    (*addr_of_mut!(cg)).centerPrintTime = (*addr_of!(cg)).time;
    (*addr_of_mut!(cg)).centerPrintY = y;

    // count the number of lines for centering
    (*addr_of_mut!(cg)).centerPrintLines = 1;
    let mut s = (*addr_of!(cg)).centerPrint.as_ptr();
    while *s != 0 {
        if *s as u8 as char == '\n' {
            (*addr_of_mut!(cg)).centerPrintLines += 1;
        }
        s = s.add(1);
    }
}

/*
===================
CG_DrawCenterString
===================
*/
pub unsafe extern "C" fn CG_DrawCenterString() {
    let mut start: *mut u8;
    let l: c_int;
    let mut x: c_int;
    let mut y: c_int;
    let mut w: c_int;
    let color: *mut [f32; 4];

    if (*addr_of!(cg)).centerPrintTime == 0 {
        return;
    }

    let color = CG_FadeColor((*addr_of!(cg)).centerPrintTime, 1000 * 3);
    if color.is_null() {
        return;
    }

    if ((textcolor_center[0] == 0.0f32) && (textcolor_center[1] == 0.0f32) && (textcolor_center[2] == 0.0f32)
        && (textcolor_center[3] == 0.0f32))
    {
        Vector4Copy(addr_of!(colorTable[7]), addr_of_mut!(textcolor_center)); // CT_WHITE = 7
    }

    start = (*addr_of_mut!(cg)).centerPrint.as_mut_ptr();

    let fontHeight = cgi_R_Font_HeightPixels((*addr_of!(cgs)).media.qhFontMedium, 1.0f32);
    y = (*addr_of!(cg)).centerPrintY - ((*addr_of!(cg)).centerPrintLines * fontHeight) / 2;

    loop {
        let mut linebuffer: [u8; 1024] = [0; 1024];

        // this is kind of unpleasant when dealing with MBCS, but...
        //
        let mut psString = start;
        let mut iOutIndex = 0;
        let mut l = 0;
        while l < (1024 - 1) {
            let mut iAdvanceCount: c_int = 0;
            let uiLetter = cgi_AnyLanguage_ReadCharFromString(
                psString,
                &mut iAdvanceCount,
                std::ptr::null_mut(),
            );
            psString = psString.add(iAdvanceCount as usize);
            if uiLetter == 0 || uiLetter == '\n' as u32 {
                break;
            }
            if uiLetter > 255 {
                linebuffer[iOutIndex] = (uiLetter >> 8) as u8;
                iOutIndex += 1;
                linebuffer[iOutIndex] = (uiLetter & 0xFF) as u8;
                iOutIndex += 1;
            } else {
                linebuffer[iOutIndex] = (uiLetter & 0xFF) as u8;
                iOutIndex += 1;
            }
            l += 1;
        }
        linebuffer[iOutIndex] = 0;

        w = cgi_R_Font_StrLenPixels(linebuffer.as_ptr(), (*addr_of!(cgs)).media.qhFontMedium, 1.0f32);

        x = (640 - w) / 2; // SCREEN_WIDTH = 640

        cgi_R_Font_DrawString(
            x,
            y,
            linebuffer.as_ptr(),
            addr_of!(textcolor_center),
            (*addr_of!(cgs)).media.qhFontMedium,
            -1,
            1.0f32,
        );

        y += fontHeight;

        while *start != 0 && (*start as u8 as char != '\n') {
            start = start.add(1);
        }
        if *start == 0 {
            break;
        }
        start = start.add(1);
    }
}
