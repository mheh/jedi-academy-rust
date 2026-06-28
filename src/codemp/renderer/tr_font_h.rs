#![allow(non_snake_case)]

// Filename:-	tr_font.h
//
// font support

// This file is shared in the single and multiplayer codebases, so be CAREFUL WHAT YOU ADD/CHANGE!!!!!

use core::ffi::{c_char, c_int, c_uint};

type qboolean = c_int;

extern "C" {
    pub fn R_ShutdownFonts();
    pub fn R_InitFonts();
    pub fn RE_RegisterFont(psName: *const c_char) -> c_int;
    pub fn RE_Font_StrLenPixels(psText: *const c_char, iFontHandle: c_int, fScale: f32) -> c_int;
    pub fn RE_Font_StrLenChars(psText: *const c_char) -> c_int;
    pub fn RE_Font_HeightPixels(iFontHandle: c_int, fScale: f32) -> c_int;
    pub fn RE_Font_DrawString(
        ox: c_int,
        oy: c_int,
        psText: *const c_char,
        rgba: *const f32,
        iFontHandle: c_int,
        iMaxPixelWidth: c_int,
        fScale: f32,
    );

    // Dammit, I can't use this more elegant form because of !@#@!$%% VM code... (can't alter passed in ptrs, only contents of)
    //
    // unsigned int AnyLanguage_ReadCharFromString( const char **ppsText, qboolean *pbIsTrailingPunctuation = NULL);
    //
    // so instead we have to use this messier method...
    //
    pub fn AnyLanguage_ReadCharFromString(
        psText: *const c_char,
        piAdvanceCount: *mut c_int,
        pbIsTrailingPunctuation: *mut qboolean,
    ) -> c_uint;

    pub fn Language_IsAsian() -> qboolean;
    pub fn Language_UsesSpaces() -> qboolean;
}

// end
