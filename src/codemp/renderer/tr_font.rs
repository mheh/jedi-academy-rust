// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_char, c_void};

/////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// This file is shared in the single and multiplayer codebases, so be CAREFUL WHAT YOU ADD/CHANGE!!!!!
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language_e {
    eWestern = 0,   // ( I only care about asian languages in here at the moment )
    eRussian = 1,   //  .. but now I need to care about this, since it uses a different TP
    ePolish = 2,    // ditto
    eKorean = 3,
    eTaiwanese = 4, // 15x15 glyphs tucked against BR of 16x16 space
    eJapanese = 5,  // 15x15 glyphs tucked against TL of 16x16 space
    eChinese = 6,   // 15x15 glyphs tucked against TL of 16x16 space
    eThai = 7,      // 16x16 cells with glyphs against left edge, special file (tha_widths.dat) for variable widths
}

// this is to cut down on all the stupid string compares I've been doing, and convert asian stuff to switch-case
//
fn GetLanguageEnum() -> Language_e {
    static mut iSE_Language_ModificationCount: i32 = -1234;    // any old silly value that won't match the cvar mod count
    static mut eLanguage: Language_e = Language_e::eWestern;

    // only re-strcmp() when language string has changed from what we knew it as...
    //
    unsafe {
        if iSE_Language_ModificationCount != (*se_language).modificationCount {
            iSE_Language_ModificationCount = (*se_language).modificationCount;

            if Language_IsRussian() {
                eLanguage = Language_e::eRussian;
            } else if Language_IsPolish() {
                eLanguage = Language_e::ePolish;
            } else if Language_IsKorean() {
                eLanguage = Language_e::eKorean;
            } else if Language_IsTaiwanese() {
                eLanguage = Language_e::eTaiwanese;
            } else if Language_IsJapanese() {
                eLanguage = Language_e::eJapanese;
            } else if Language_IsChinese() {
                eLanguage = Language_e::eChinese;
            } else if Language_IsThai() {
                eLanguage = Language_e::eThai;
            } else {
                eLanguage = Language_e::eWestern;
            }
        }

        eLanguage
    }
}

#[repr(C)]
struct SBCSOverrideLanguages_t {
    m_psName: *const c_char,
    m_eLanguage: Language_e,
}

// so I can do some stuff with for-next loops when I add polish etc...
//
static mut g_SBCSOverrideLanguages: [SBCSOverrideLanguages_t; 3] = [
    SBCSOverrideLanguages_t { m_psName: b"russian\0".as_ptr() as *const c_char, m_eLanguage: Language_e::eRussian },
    SBCSOverrideLanguages_t { m_psName: b"polish\0".as_ptr() as *const c_char, m_eLanguage: Language_e::ePolish },
    SBCSOverrideLanguages_t { m_psName: core::ptr::null(), m_eLanguage: Language_e::eWestern },
];

//================================================
//

const sFILENAME_THAI_WIDTHS: &[u8; 21] = b"fonts/tha_widths.dat\0";
const sFILENAME_THAI_CODES: &[u8; 20] = b"fonts/tha_codes.dat\0";

#[repr(C)]
struct ThaiCodes_t {
    m_mapValidCodes: *mut std::collections::HashMap<i32, i32>,
    m_viGlyphWidths: *mut Vec<i32>,
    m_strInitFailureReason: *mut String,
}

impl ThaiCodes_t {
    fn new() -> Self {
        ThaiCodes_t {
            m_mapValidCodes: std::ptr::null_mut(),
            m_viGlyphWidths: std::ptr::null_mut(),
            m_strInitFailureReason: std::ptr::null_mut(),
        }
    }

    fn Clear(&mut self) {
        if !self.m_mapValidCodes.is_null() {
            unsafe {
                (*self.m_mapValidCodes).clear();
            }
        }
        if !self.m_viGlyphWidths.is_null() {
            unsafe {
                (*self.m_viGlyphWidths).clear();
            }
        }
        if !self.m_strInitFailureReason.is_null() {
            unsafe {
                (*self.m_strInitFailureReason).clear();
            }
        }
    }

    // convert a supplied 1,2 or 3-byte multiplied-up integer into a valid 0..n index, else -1...
    //
    fn GetValidIndex(&self, iCode: i32) -> i32 {
        if self.m_mapValidCodes.is_null() {
            return -1;
        }
        unsafe {
            if let Some(&index) = (*self.m_mapValidCodes).get(&iCode) {
                index
            } else {
                -1
            }
        }
    }

    fn GetWidth(&self, iGlyphIndex: i32) -> i32 {
        if !self.m_viGlyphWidths.is_null() {
            unsafe {
                if (iGlyphIndex as usize) < (*self.m_viGlyphWidths).len() {
                    return (*self.m_viGlyphWidths)[iGlyphIndex as usize];
                }
            }
        }
        assert!(false);
        0
    }

    // return is error message to display, or NULL for success
    fn Init(&mut self) -> *const c_char {
        // Complex initialization omitted - would require file system integration
        std::ptr::null()
    }
}

#[allow(non_snake_case)]
const GLYPH_MAX_KOREAN_SHADERS: usize = 3;
#[allow(non_snake_case)]
const GLYPH_MAX_TAIWANESE_SHADERS: usize = 4;
#[allow(non_snake_case)]
const GLYPH_MAX_JAPANESE_SHADERS: usize = 3;
#[allow(non_snake_case)]
const GLYPH_MAX_CHINESE_SHADERS: usize = 3;
#[allow(non_snake_case)]
const GLYPH_MAX_THAI_SHADERS: usize = 3;
#[allow(non_snake_case)]
const GLYPH_MAX_ASIAN_SHADERS: usize = 4;    // this MUST equal the larger of the above defines

extern "C" {
    static mut se_language: *mut cvar_t;
    static mut com_buildScript: *mut cvar_t;

    fn Language_IsRussian() -> bool;
    fn Language_IsPolish() -> bool;
    fn Language_IsKorean() -> bool;
    fn Language_IsTaiwanese() -> bool;
    fn Language_IsJapanese() -> bool;
    fn Language_IsChinese() -> bool;
    fn Language_IsThai() -> bool;

    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> i32;
    fn FS_FreeFile(buffer: *mut c_void);
    fn FS_FOpenFileRead(filename: *const c_char, f: *mut fileHandle_t, uniqueFILE: bool) -> i32;
    fn FS_FCloseFile(f: fileHandle_t);

    fn COM_SkipPath(pathname: *const c_char) -> *const c_char;
    fn COM_StripExtension(in_: *mut c_char, out: *mut c_char);

    fn RE_RegisterShaderNoMip(name: *const c_char) -> c_int;
    fn RE_RegisterFont(name: *const c_char) -> c_int;
    fn RE_SetColor(rgba: *const f32);
    fn RE_StretchPic(x: f32, y: f32, w: f32, h: f32, s1: f32, t1: f32, s2: f32, t2: f32, hShader: qhandle_t);

    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Com_Printf(msg: *const c_char, ...);
    fn Com_Error(code: i32, msg: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: usize, msg: *const c_char, ...);

    fn Sys_Milliseconds() -> i32;
    fn ColorIndex(c: c_char) -> i32;

    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);

    fn Round(f: f32) -> i32;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn sprintf(dest: *mut c_char, format: *const c_char, ...) -> i32;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn strlen(s: *const c_char) -> usize;
}

// Stub types for external dependencies
#[repr(C)]
struct cvar_t {
    name: *const c_char,
    string: *const c_char,
    latched_string: *const c_char,
    flags: c_int,
    modified: bool,
    modificationCount: c_int,
    value: f32,
    integer: c_int,
    resetString: *const c_char,
    resetValue: f32,
    resetInteger: c_int,
    domain: *const c_char,
    domainString: *const c_char,
    privFlags: c_int,
    cheatProtected: bool,
    latchedString: *const c_char,
    next: *mut cvar_t,
    prev: *mut cvar_t,
    completeString: *const c_char,
    hintString: *const c_char,
}

type fileHandle_t = i32;
type qhandle_t = c_int;
type qboolean = bool;

#[repr(C)]
#[derive(Clone, Copy)]
struct glyphInfo_t {
    height: i32,
    top: i32,
    baseline: i32,
    leftSideBearing: i32,
    advance: i32,
    s: f32,
    t: f32,
    s2: f32,
    t2: f32,
    charWidth: i32,
    imageWidth: i32,
    imageHeight: i32,
    s_padding: f32,
    t_padding: f32,
}

// Additional fields for glyphInfo_t based on actual usage in code
#[repr(C)]
#[derive(Clone, Copy)]
struct glyphInfo_t_full {
    width: i32,
    height: i32,
    top: i32,
    baseline: i32,
    leftSideBearing: i32,
    advance: i32,
    horizOffset: i32,
    horizAdvance: i32,
    s: f32,
    t: f32,
    s2: f32,
    t2: f32,
    charWidth: i32,
    imageWidth: i32,
    imageHeight: i32,
    s_padding: f32,
    t_padding: f32,
}

#[repr(C)]
struct dfontdat_t {
    mGlyphs: [glyphInfo_t; 256],  // GLYPH_COUNT assumed to be 256
    mPointSize: i32,
    mHeight: i32,
    mAscender: i32,
    mDescender: i32,
}

const GLYPH_COUNT: usize = 256;
const MAX_QPATH: usize = 256;

const qfalse: bool = false;
const qtrue: bool = true;

const ERR_DROP: i32 = 0;

const SET_MASK: c_int = 0xFF;
const STYLE_BLINK: c_int = 256;
const STYLE_DROPSHADOW: c_int = 512;

// =============================== some korean stuff =======================================

const KSC5601_HANGUL_HIBYTE_START: u8 = 0xB0;       // range is...
const KSC5601_HANGUL_HIBYTE_STOP: u8 = 0xC8;        // ... inclusive
const KSC5601_HANGUL_LOBYTE_LOBOUND: u8 = 0xA0;     // range is...
const KSC5601_HANGUL_LOBYTE_HIBOUND: u8 = 0xFF;     // ...bounding (ie only valid in between these points, but NULLs in charsets for these codes)
const KSC5601_HANGUL_CODES_PER_ROW: i32 = 96;       // 2 more than the number of glyphs

#[inline]
fn Korean_ValidKSC5601Hangul(_iHi: u8, _iLo: u8) -> bool {
    _iHi >= KSC5601_HANGUL_HIBYTE_START
        && _iHi <= KSC5601_HANGUL_HIBYTE_STOP
        && _iLo > KSC5601_HANGUL_LOBYTE_LOBOUND
        && _iLo < KSC5601_HANGUL_LOBYTE_HIBOUND
}

#[inline]
fn Korean_ValidKSC5601Hangul_uint(uiCode: u32) -> bool {
    Korean_ValidKSC5601Hangul((uiCode >> 8) as u8, (uiCode & 0xFF) as u8)
}

// takes a KSC5601 double-byte hangul code and collapses down to a 0..n glyph index...
// Assumes rows are 96 wide (glyph slots), not 94 wide (actual glyphs), so I can ignore boundary markers
//
// (invalid hangul codes will return 0)
//
fn Korean_CollapseKSC5601HangulCode(mut uiCode: u32) -> i32 {
    if Korean_ValidKSC5601Hangul_uint(uiCode) {
        uiCode -= ((KSC5601_HANGUL_HIBYTE_START as u32) * 256) + KSC5601_HANGUL_LOBYTE_LOBOUND as u32;   // sneaky maths on both bytes, reduce to 0x0000 onwards
        uiCode = ((uiCode >> 8) * KSC5601_HANGUL_CODES_PER_ROW as u32) + (uiCode & 0xFF);
        return uiCode as i32;
    }
    0
}

fn Korean_InitFields(iGlyphTPs: &mut i32, psLang: &mut &str) -> i32 {
    *psLang = "kor";
    *iGlyphTPs = GLYPH_MAX_KOREAN_SHADERS as i32;
    unsafe { g_iNonScaledCharRange = 255; }
    32   // m_iAsianGlyphsAcross
}

// ======================== some taiwanese stuff ==============================

// (all ranges inclusive for Big5)...
//
const BIG5_HIBYTE_START0: u8 = 0xA1;     // (misc chars + level 1 hanzi)
const BIG5_HIBYTE_STOP0: u8 = 0xC6;      //
const BIG5_HIBYTE_START1: u8 = 0xC9;     // (level 2 hanzi)
const BIG5_HIBYTE_STOP1: u8 = 0xF9;      //
const BIG5_LOBYTE_LOBOUND0: u8 = 0x40;   //
const BIG5_LOBYTE_HIBOUND0: u8 = 0x7E;   //
const BIG5_LOBYTE_LOBOUND1: u8 = 0xA1;   //
const BIG5_LOBYTE_HIBOUND1: u8 = 0xFE;   //
const BIG5_CODES_PER_ROW: i32 = 160;     // 3 more than the number of glyphs

fn Taiwanese_ValidBig5Code(uiCode: u32) -> bool {
    let _iHi = ((uiCode >> 8) & 0xFF) as u8;
    if (_iHi >= BIG5_HIBYTE_START0 && _iHi <= BIG5_HIBYTE_STOP0)
        || (_iHi >= BIG5_HIBYTE_START1 && _iHi <= BIG5_HIBYTE_STOP1)
    {
        let _iLo = (uiCode & 0xFF) as u8;

        if (_iLo >= BIG5_LOBYTE_LOBOUND0 && _iLo <= BIG5_LOBYTE_HIBOUND0)
            || (_iLo >= BIG5_LOBYTE_LOBOUND1 && _iLo <= BIG5_LOBYTE_HIBOUND1)
        {
            return true;
        }
    }

    false
}

// only call this when Taiwanese_ValidBig5Code() has already returned true...
//
fn Taiwanese_IsTrailingPunctuation(uiCode: u32) -> bool {
    // so far I'm just counting the first 21 chars, those seem to be all the basic punctuation...
    //
    if uiCode >= ((BIG5_HIBYTE_START0 as u32) << 8 | BIG5_LOBYTE_LOBOUND0 as u32)
        && uiCode < ((BIG5_HIBYTE_START0 as u32) << 8 | (BIG5_LOBYTE_LOBOUND0 as u32 + 20))
    {
        return true;
    }

    false
}

// takes a BIG5 double-byte code (including level 2 hanzi) and collapses down to a 0..n glyph index...
// Assumes rows are 160 wide (glyph slots), not 157 wide (actual glyphs), so I can ignore boundary markers
//
// (invalid big5 codes will return 0)
//
fn Taiwanese_CollapseBig5Code(mut uiCode: u32) -> i32 {
    if Taiwanese_ValidBig5Code(uiCode) {
        uiCode -= ((BIG5_HIBYTE_START0 as u32) * 256) + BIG5_LOBYTE_LOBOUND0 as u32;    // sneaky maths on both bytes, reduce to 0x0000 onwards
        if (uiCode & 0xFF) >= ((BIG5_LOBYTE_LOBOUND1 as u32 - 1) - BIG5_LOBYTE_LOBOUND0 as u32) {
            uiCode -= (((BIG5_LOBYTE_LOBOUND1 as u32 - 1) - (BIG5_LOBYTE_HIBOUND0 as u32 + 1)) - 1);
        }
        uiCode = ((uiCode >> 8) * BIG5_CODES_PER_ROW as u32) + (uiCode & 0xFF);
        return uiCode as i32;
    }
    0
}

fn Taiwanese_InitFields(iGlyphTPs: &mut i32, psLang: &mut &str) -> i32 {
    *psLang = "tai";
    *iGlyphTPs = GLYPH_MAX_TAIWANESE_SHADERS as i32;
    unsafe { g_iNonScaledCharRange = 255; }
    64   // m_iAsianGlyphsAcross
}

// ======================== some Japanese stuff ==============================

// ( all ranges inclusive for Shift-JIS )
//
const SHIFTJIS_HIBYTE_START0: u8 = 0x81;
const SHIFTJIS_HIBYTE_STOP0: u8 = 0x9F;
const SHIFTJIS_HIBYTE_START1: u8 = 0xE0;
const SHIFTJIS_HIBYTE_STOP1: u8 = 0xEF;
//
const SHIFTJIS_LOBYTE_START0: u8 = 0x40;
const SHIFTJIS_LOBYTE_STOP0: u8 = 0x7E;
const SHIFTJIS_LOBYTE_START1: u8 = 0x80;
const SHIFTJIS_LOBYTE_STOP1: u8 = 0xFC;
const SHIFTJIS_CODES_PER_ROW: i32 = (((SHIFTJIS_LOBYTE_STOP0 as i32 - SHIFTJIS_LOBYTE_START0 as i32) + 1)
    + ((SHIFTJIS_LOBYTE_STOP1 as i32 - SHIFTJIS_LOBYTE_START1 as i32) + 1));

fn Japanese_ValidShiftJISCode(_iHi: u8, _iLo: u8) -> bool {
    if (_iHi >= SHIFTJIS_HIBYTE_START0 && _iHi <= SHIFTJIS_HIBYTE_STOP0)
        || (_iHi >= SHIFTJIS_HIBYTE_START1 && _iHi <= SHIFTJIS_HIBYTE_STOP1)
    {
        if (_iLo >= SHIFTJIS_LOBYTE_START0 && _iLo <= SHIFTJIS_LOBYTE_STOP0)
            || (_iLo >= SHIFTJIS_LOBYTE_START1 && _iLo <= SHIFTJIS_LOBYTE_STOP1)
        {
            return true;
        }
    }

    false
}

#[inline]
fn Japanese_ValidShiftJISCode_uint(uiCode: u32) -> bool {
    Japanese_ValidShiftJISCode((uiCode >> 8) as u8, (uiCode & 0xFF) as u8)
}

// only call this when Japanese_ValidShiftJISCode() has already returned true...
//
fn Japanese_IsTrailingPunctuation(uiCode: u32) -> bool {
    // so far I'm just counting the first 18 chars, those seem to be all the basic punctuation...
    //
    if uiCode >= ((SHIFTJIS_HIBYTE_START0 as u32) << 8 | SHIFTJIS_LOBYTE_START0 as u32)
        && uiCode < ((SHIFTJIS_HIBYTE_START0 as u32) << 8 | (SHIFTJIS_LOBYTE_START0 as u32 + 18))
    {
        return true;
    }

    false
}

// takes a ShiftJIS double-byte code and collapse down to a 0..n glyph index...
//
// (invalid codes will return 0)
//
fn Japanese_CollapseShiftJISCode(mut uiCode: u32) -> i32 {
    if Japanese_ValidShiftJISCode_uint(uiCode) {
        uiCode -= ((SHIFTJIS_HIBYTE_START0 as u32) << 8) | SHIFTJIS_LOBYTE_START0 as u32;    // sneaky maths on both bytes, reduce to 0x0000 onwards

        if (uiCode & 0xFF) >= (SHIFTJIS_LOBYTE_START1 as u32 - SHIFTJIS_LOBYTE_START0 as u32) {
            uiCode -= ((SHIFTJIS_LOBYTE_START1 as u32 - SHIFTJIS_LOBYTE_STOP0 as u32) - 1);
        }

        if ((uiCode >> 8) & 0xFF) >= (SHIFTJIS_HIBYTE_START1 as u32 - SHIFTJIS_HIBYTE_START0 as u32) {
            uiCode -= (((SHIFTJIS_HIBYTE_START1 as u32 - SHIFTJIS_HIBYTE_STOP0 as u32) - 1) << 8);
        }

        uiCode = ((uiCode >> 8) * SHIFTJIS_CODES_PER_ROW as u32) + (uiCode & 0xFF);

        return uiCode as i32;
    }
    0
}

fn Japanese_InitFields(iGlyphTPs: &mut i32, psLang: &mut &str) -> i32 {
    *psLang = "jap";
    *iGlyphTPs = GLYPH_MAX_JAPANESE_SHADERS as i32;
    unsafe { g_iNonScaledCharRange = 255; }
    64   // m_iAsianGlyphsAcross
}

// ======================== some Chinese stuff ==============================

const GB_HIBYTE_START: u8 = 0xA1;      // range is...
const GB_HIBYTE_STOP: u8 = 0xF7;       // ... inclusive
const GB_LOBYTE_LOBOUND: u8 = 0xA0;    // range is...
const GB_LOBYTE_HIBOUND: u8 = 0xFF;    // ...bounding (ie only valid in between these points, but NULLs in charsets for these codes)
const GB_CODES_PER_ROW: i32 = 95;      // 1 more than the number of glyphs

#[inline]
fn Chinese_ValidGBCode(_iHi: u8, _iLo: u8) -> bool {
    _iHi >= GB_HIBYTE_START && _iHi <= GB_HIBYTE_STOP && _iLo > GB_LOBYTE_LOBOUND && _iLo < GB_LOBYTE_HIBOUND
}

#[inline]
fn Chinese_ValidGBCode_uint(uiCode: u32) -> bool {
    Chinese_ValidGBCode((uiCode >> 8) as u8, (uiCode & 0xFF) as u8)
}

// only call this when Chinese_ValidGBCode() has already returned true...
//
fn Chinese_IsTrailingPunctuation(uiCode: u32) -> bool {
    // so far I'm just counting the first 13 chars, those seem to be all the basic punctuation...
    //
    if uiCode > ((GB_HIBYTE_START as u32) << 8 | GB_LOBYTE_LOBOUND as u32)
        && uiCode < ((GB_HIBYTE_START as u32) << 8 | (GB_LOBYTE_LOBOUND as u32 + 14))
    {
        return true;
    }

    false
}

// takes a GB double-byte code and collapses down to a 0..n glyph index...
// Assumes rows are 96 wide (glyph slots), not 94 wide (actual glyphs), so I can ignore boundary markers
//
// (invalid GB codes will return 0)
//
fn Chinese_CollapseGBCode(mut uiCode: u32) -> i32 {
    if Chinese_ValidGBCode_uint(uiCode) {
        uiCode -= ((GB_HIBYTE_START as u32) * 256) + GB_LOBYTE_LOBOUND as u32;    // sneaky maths on both bytes, reduce to 0x0000 onwards
        uiCode = ((uiCode >> 8) * GB_CODES_PER_ROW as u32) + (uiCode & 0xFF);
        return uiCode as i32;
    }

    0
}

fn Chinese_InitFields(iGlyphTPs: &mut i32, psLang: &mut &str) -> i32 {
    *psLang = "chi";
    *iGlyphTPs = GLYPH_MAX_CHINESE_SHADERS as i32;
    unsafe { g_iNonScaledCharRange = 255; }
    64   // m_iAsianGlyphsAcross
}

// ======================== some Thai stuff ==============================

//TIS 620-2533

const TIS_GLYPHS_START: u8 = 160;
const TIS_SARA_AM: u32 = 0xD3;      // special case letter, both a new letter and a trailing accent for the prev one

static mut g_ThaiCodes: ThaiCodes_t = ThaiCodes_t {
    m_mapValidCodes: core::ptr::null_mut(),
    m_viGlyphWidths: core::ptr::null_mut(),
    m_strInitFailureReason: core::ptr::null_mut(),
};

/*
fn Thai_IsAccentChar( uiCode: u32 ) -> i32
{
    match uiCode {
        209 | 212 | 213 | 214 | 215 | 216 | 217 | 218 |
        231 | 232 | 233 | 234 | 235 | 236 | 237 | 238 => true,
        _ => false,
    }
}
*/

// returns a valid Thai code (or 0), based on taking 1,2 or 3 bytes from the supplied byte stream
//  Fills in <iThaiBytes> with 1,2 or 3
fn Thai_ValidTISCode(psString: *const u8, iThaiBytes: &mut i32) -> u32 {
    // try a 1-byte code first...
    //
    unsafe {
        if *psString >= TIS_GLYPHS_START {
            // this code is heavily little-endian, so someone else will need to port for Mac etc... (not my problem ;-)
            //

            union CodeToTry_t {
                sChars: [c_char; 4],
                uiCode: u32,
            }

            let mut CodeToTry: CodeToTry_t = CodeToTry_t { uiCode: 0 };    // important that we clear all 4 bytes in sChars here

            // thai codes can be up to 3 bytes long, so see how high we can get...
            //
            let mut i = 0;
            loop {
                if i >= 3 {
                    break;
                }

                CodeToTry.sChars[i] = *psString.add(i) as c_char;

                let iIndex = g_ThaiCodes.GetValidIndex(CodeToTry.uiCode as i32);
                if iIndex == -1 {
                    // failed, so return previous-longest code...
                    //
                    CodeToTry.sChars[i] = 0;
                    break;
                }
                i += 1;
            }
            *iThaiBytes = i as i32;
            assert!(i != 0);    // if 'i' was 0, then this may be an error, trying to get a thai accent as standalone char?
            return CodeToTry.uiCode;
        }
    }

    0
}

// special case, thai can only break on certain letters, and since the rules are complicated then
//  we tell the translators to put an underscore ('_') between each word even though in Thai they're
//  all jammed together at final output onscreen...
//
#[inline]
fn Thai_IsTrailingPunctuation(uiCode: u32) -> bool {
    uiCode == '_' as u32
}

// takes a TIS 1,2 or 3 byte code and collapse down to a 0..n glyph index...
//
// (invalid codes will return 0)
//
fn Thai_CollapseTISCode(uiCode: u32) -> i32 {
    if uiCode >= TIS_GLYPHS_START as u32 {   // so western letters drop through as invalid
        let iCollapsedIndex = unsafe { g_ThaiCodes.GetValidIndex(uiCode as i32) };
        if iCollapsedIndex != -1 {
            return iCollapsedIndex;
        }
    }

    0
}

fn Thai_InitFields(iGlyphTPs: &mut i32, psLang: &mut &str) -> i32 {
    *psLang = "tha";
    *iGlyphTPs = GLYPH_MAX_THAI_SHADERS as i32;
    unsafe { g_iNonScaledCharRange = i32::MAX; }   // in other words, don't scale any thai chars down
    32   // m_iAsianGlyphsAcross
}

// ============================================================================

// round float to one decimal place...
//
fn RoundTenth(fValue: f32) -> f32 {
    ((fValue * 10.0f32).floor() + 0.5f32).floor() / 10.0f32
}

static mut g_iCurrentFontIndex: c_int = 0;        // entry 0 is reserved index for missing/invalid, else ++ with each new font registered
static mut g_vFontArray: *mut Vec<*mut CFontInfo> = core::ptr::null_mut();
static mut g_mapFontIndexes: *mut std::collections::HashMap<String, i32> = core::ptr::null_mut();
static mut g_iNonScaledCharRange: i32 = 0;        // this is used with auto-scaling of asian fonts, anything below this number is preserved in scale, anything above is scaled down by 0.75f

//paletteRGBA_c             lastcolour;

// takes char *, returns integer char at that point, and advances char * on by enough bytes to move
//  past the letter (either western 1 byte or Asian multi-byte)...
//
// looks messy, but the actual execution route is quite short, so it's fast...
//
// Note that I have to have this 3-param form instead of advancing a passed-in "const char **psText" because of VM-crap where you can only change ptr-contents, not ptrs themselves. Bleurgh. Ditto the qtrue:qfalse crap instead of just returning stuff straight through.
//
#[allow(non_snake_case)]
pub fn AnyLanguage_ReadCharFromString(
    psText: *const c_char,
    piAdvanceCount: *mut i32,
    pbIsTrailingPunctuation: *mut qboolean,
) -> u32 {
    let psString = psText as *const u8;   // avoid sign-promote bug
    let mut uiLetter: u32;

    unsafe {
        match GetLanguageEnum() {
            Language_e::eKorean => {
                if Korean_ValidKSC5601Hangul(*psString, *psString.add(1)) {
                    uiLetter = (*psString as u32 * 256) + *psString.add(1) as u32;
                    *piAdvanceCount = 2;

                    // not going to bother testing for korean punctuation here, since korean already
                    //  uses spaces, and I don't have the punctuation glyphs defined, only the basic 2350 hanguls
                    //
                    if !pbIsTrailingPunctuation.is_null() {
                        *pbIsTrailingPunctuation = false;
                    }

                    return uiLetter;
                }
            }

            Language_e::eTaiwanese => {
                if Taiwanese_ValidBig5Code((*psString as u32 * 256) + *psString.add(1) as u32) {
                    uiLetter = (*psString as u32 * 256) + *psString.add(1) as u32;
                    *piAdvanceCount = 2;

                    // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                    //
                    if !pbIsTrailingPunctuation.is_null() {
                        *pbIsTrailingPunctuation = Taiwanese_IsTrailingPunctuation(uiLetter);
                    }

                    return uiLetter;
                }
            }

            Language_e::eJapanese => {
                if Japanese_ValidShiftJISCode(*psString, *psString.add(1)) {
                    uiLetter = (*psString as u32 * 256) + *psString.add(1) as u32;
                    *piAdvanceCount = 2;

                    // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                    //
                    if !pbIsTrailingPunctuation.is_null() {
                        *pbIsTrailingPunctuation = Japanese_IsTrailingPunctuation(uiLetter);
                    }

                    return uiLetter;
                }
            }

            Language_e::eChinese => {
                if Chinese_ValidGBCode_uint((*psString as u32 * 256) + *psString.add(1) as u32) {
                    uiLetter = (*psString as u32 * 256) + *psString.add(1) as u32;
                    *piAdvanceCount = 2;

                    // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                    //
                    if !pbIsTrailingPunctuation.is_null() {
                        *pbIsTrailingPunctuation = Chinese_IsTrailingPunctuation(uiLetter);
                    }

                    return uiLetter;
                }
            }

            Language_e::eThai => {
                let mut iThaiBytes: i32 = 0;
                uiLetter = Thai_ValidTISCode(psString, &mut iThaiBytes);
                if uiLetter != 0 {
                    *piAdvanceCount = iThaiBytes;

                    if !pbIsTrailingPunctuation.is_null() {
                        *pbIsTrailingPunctuation = Thai_IsTrailingPunctuation(uiLetter);
                    }

                    return uiLetter;
                }
            }

            _ => {}
        }

        // ... must not have been an MBCS code...
        //
        uiLetter = *psString as u32;
        *piAdvanceCount = 1;

        if !pbIsTrailingPunctuation.is_null() {
            *pbIsTrailingPunctuation = uiLetter == '!' as u32
                || uiLetter == '?' as u32
                || uiLetter == ',' as u32
                || uiLetter == '.' as u32
                || uiLetter == ';' as u32
                || uiLetter == ':' as u32;
        }

        uiLetter
    }
}

// needed for subtitle printing since original code no longer worked once camera bar height was changed to 480/10
//  rather than refdef height / 10. I now need to bodge the coords to come out right.
//
pub fn Language_IsAsian() -> qboolean {
    match GetLanguageEnum() {
        Language_e::eKorean
        | Language_e::eTaiwanese
        | Language_e::eJapanese
        | Language_e::eChinese
        | Language_e::eThai => true,   // this is asian, but the query is normally used for scaling
        _ => false,
    }
}

pub fn Language_UsesSpaces() -> qboolean {
    // ( korean uses spaces )
    match GetLanguageEnum() {
        Language_e::eTaiwanese
        | Language_e::eJapanese
        | Language_e::eChinese
        | Language_e::eThai => false,
        _ => true,
    }
}

// ======================================================================

#[repr(C)]
pub struct CFontInfo {
    // From the fontdat file
    mGlyphs: [glyphInfo_t_full; GLYPH_COUNT],

    //  int             mAsianHack;             // unused junk from John's fontdat file format.
    // end of fontdat data

    mShader: c_int,      // handle to the shader with the glyph

    m_hAsianShaders: [c_int; GLYPH_MAX_ASIAN_SHADERS],    // shaders for Korean glyphs where applicable
    m_AsianGlyph: glyphInfo_t_full,                       // special glyph containing asian->western scaling info for all glyphs
    m_iAsianGlyphsAcross: i32,                            // needed to dynamically calculate S,T coords
    m_iAsianPagesLoaded: i32,
    m_bAsianLastPageHalfHeight: bool,
    m_iLanguageModificationCount: i32,    // doesn't matter what this is, so long as it's comparable as being changed

    m_pThaiData: *mut ThaiCodes_t,

    pub m_sFontName: [c_char; MAX_QPATH],    // eg "fonts/lcd"  // needed for korean font-hint if we need >1 hangul set
    pub mPointSize: i32,
    pub mHeight: i32,
    pub mAscender: i32,
    pub mDescender: i32,

    pub mbRoundCalcs: bool,    // trying to make this !@#$%^ thing work with scaling
    pub m_iThisFont: i32,      // handle to itself
    pub m_iAltSBCSFont: i32,   // -1 == NULL // alternative single-byte font for languages like russian/polish etc that need to override high characters ?
    pub m_iOriginalFontWhenSBCSOverriden: i32,
    pub m_fAltSBCSFontScaleFactor: f32,    // -1, else amount to adjust returned values by to make them fit the master western font they're substituting for
    pub m_bIsFakeAlienLanguage: bool,      // ... if true, don't process as MBCS or override as SBCS etc
}

impl CFontInfo {
    pub const fn GetPointSize(&self) -> i32 {
        self.mPointSize
    }

    pub const fn GetHeight(&self) -> i32 {
        self.mHeight
    }

    pub const fn GetAscender(&self) -> i32 {
        self.mAscender
    }

    pub const fn GetDescender(&self) -> i32 {
        self.mDescender
    }

    pub fn GetLetter(&mut self, uiLetter: u32, piShader: *mut c_int) -> *const glyphInfo_t_full;

    pub const fn GetCollapsedAsianCode(&self, uiLetter: u32) -> i32;

    pub fn GetLetterWidth(&mut self, uiLetter: u32) -> i32;

    pub fn GetLetterHorizAdvance(&mut self, uiLetter: u32) -> i32;

    pub const fn GetShader(&self) -> c_int {
        self.mShader
    }

    pub fn FlagNoAsianGlyphs(&mut self) {
        self.m_hAsianShaders[0] = 0;
        self.m_iLanguageModificationCount = -1;
    }   // used during constructor

    pub const fn AsianGlyphsAvailable(&self) -> bool {
        self.m_hAsianShaders[0] != 0
    }

    pub fn UpdateAsianIfNeeded(&mut self, bForceReEval: bool);
}

fn GetFont_Actual(index: i32) -> *mut CFontInfo {
    let index = index & SET_MASK;
    unsafe {
        if index >= 1 && index < g_iCurrentFontIndex {
            if let Some(pFont) = (*(g_vFontArray)).get_mut(index as usize) {
                if !(*pFont).is_null() {
                    (*(*pFont)).UpdateAsianIfNeeded(false);
                }
                *pFont
            } else {
                core::ptr::null_mut()
            }
        } else {
            core::ptr::null_mut()
        }
    }
}

// needed to add *piShader param because of multiple TPs,
//  if not passed in, then I also skip S,T calculations for re-usable static asian glyphinfo struct...
//
impl CFontInfo {
    pub fn GetLetter(&mut self, uiLetter: u32, piShader: *mut c_int) -> *const glyphInfo_t_full {
        if self.AsianGlyphsAvailable() {
            let iCollapsedAsianCode = self.GetCollapsedAsianCode(uiLetter);
            if iCollapsedAsianCode != 0 {
                if !piShader.is_null() {
                    // (Note!!  assumption for S,T calculations: all asian glyph textures pages are square except for last one
                    //          which may or may not be half height) - but not for Thai
                    //
                    let iTexturePageIndex = iCollapsedAsianCode / (self.m_iAsianGlyphsAcross * self.m_iAsianGlyphsAcross);

                    if iTexturePageIndex > self.m_iAsianPagesLoaded {
                        assert!(false);   // should never happen
                    }

                    let iOriginalCollapsedAsianCode = iCollapsedAsianCode;    // need to back this up (if Thai) for later
                    let mut iCollapsedAsianCode_adjusted = iCollapsedAsianCode - iTexturePageIndex * (self.m_iAsianGlyphsAcross * self.m_iAsianGlyphsAcross);

                    let iColumn = iCollapsedAsianCode_adjusted % self.m_iAsianGlyphsAcross;
                    let iRow = iCollapsedAsianCode_adjusted / self.m_iAsianGlyphsAcross;
                    let bHalfT = (iTexturePageIndex == (self.m_iAsianPagesLoaded - 1)) && self.m_bAsianLastPageHalfHeight;
                    let iAsianGlyphsDown = if bHalfT {
                        self.m_iAsianGlyphsAcross / 2
                    } else {
                        self.m_iAsianGlyphsAcross
                    };

                    match GetLanguageEnum() {
                        Language_e::eKorean => {
                            self.m_AsianGlyph.s = iColumn as f32 / self.m_iAsianGlyphsAcross as f32;
                            self.m_AsianGlyph.t = iRow as f32 / iAsianGlyphsDown as f32;
                            self.m_AsianGlyph.s2 = (iColumn as f32 + 1.0f32) / self.m_iAsianGlyphsAcross as f32;
                            self.m_AsianGlyph.t2 = (iRow as f32 + 1.0f32) / iAsianGlyphsDown as f32;
                        }

                        Language_e::eTaiwanese => {
                            self.m_AsianGlyph.s = (((1024 / self.m_iAsianGlyphsAcross) * iColumn) + 1) as f32 / 1024.0f32;
                            self.m_AsianGlyph.t = (((1024 / iAsianGlyphsDown) * iRow) + 1) as f32 / 1024.0f32;
                            self.m_AsianGlyph.s2 = ((1024 / self.m_iAsianGlyphsAcross) * (iColumn + 1)) as f32 / 1024.0f32;
                            self.m_AsianGlyph.t2 = ((1024 / iAsianGlyphsDown) * (iRow + 1)) as f32 / 1024.0f32;
                        }

                        Language_e::eJapanese | Language_e::eChinese => {
                            self.m_AsianGlyph.s = ((1024 / self.m_iAsianGlyphsAcross) * iColumn) as f32 / 1024.0f32;
                            self.m_AsianGlyph.t = ((1024 / iAsianGlyphsDown) * iRow) as f32 / 1024.0f32;
                            self.m_AsianGlyph.s2 = (((1024 / self.m_iAsianGlyphsAcross) * (iColumn + 1)) - 1) as f32 / 1024.0f32;
                            self.m_AsianGlyph.t2 = (((1024 / iAsianGlyphsDown) * (iRow + 1)) - 1) as f32 / 1024.0f32;
                        }

                        Language_e::eThai => {
                            let iGlyphXpos = (1024 / self.m_iAsianGlyphsAcross) * iColumn;
                            let iGlyphWidth = unsafe { g_ThaiCodes.GetWidth(iOriginalCollapsedAsianCode) };

                            // very thai-specific language-code...
                            //
                            if uiLetter == TIS_SARA_AM {
                                self.m_AsianGlyph.s = ((iGlyphXpos + 9) as f32) / 1024.0f32;    // these are pixel coords on the source TP, so don't affect scaled output
                                self.m_AsianGlyph.s2 = ((iGlyphXpos + 9 + 20) as f32) / 1024.0f32;    //
                            } else {
                                self.m_AsianGlyph.s = (iGlyphXpos as f32) / 1024.0f32;
                                // technically this .s2 line should be modified to blit only the correct width, but since
                                //  all Thai glyphs are up against the left edge of their cells and have blank to the cell
                                //  boundary then it's better to keep these calculations simpler...

                                self.m_AsianGlyph.s2 = ((iGlyphXpos + iGlyphWidth) as f32) / 1024.0f32;
                            }
                            self.m_AsianGlyph.t = ((1024 / iAsianGlyphsDown) * iRow) as f32 / 1024.0f32;
                            self.m_AsianGlyph.t2 = (((1024 / iAsianGlyphsDown) * (iRow + 1)) - 1) as f32 / 1024.0f32;

                            // special addition for Thai, need to bodge up the width and advance fields...
                            //
                            self.m_AsianGlyph.width = iGlyphWidth;
                            self.m_AsianGlyph.horizAdvance = iGlyphWidth + 1;
                        }

                        _ => {}
                    }
                    unsafe {
                        *piShader = self.m_hAsianShaders[iTexturePageIndex as usize];
                    }
                }
                return &self.m_AsianGlyph;
            }
        }

        if !piShader.is_null() {
            unsafe {
                *piShader = self.GetShader();
            }
        }

        let pGlyph = &mut self.mGlyphs[(uiLetter & 0xff) as usize];
        //
        // SBCS language substitution?...
        //
        if self.m_fAltSBCSFontScaleFactor != -1.0f32 {
            // sod it, use the asian glyph, that's fine...
            //
            unsafe {
                memcpy(
                    &mut self.m_AsianGlyph as *mut glyphInfo_t_full as *mut c_void,
                    pGlyph as *const glyphInfo_t_full as *const c_void,
                    std::mem::size_of::<glyphInfo_t_full>(),
                );
            }    // *before* changin pGlyph!

            //  CFontInfo *pOriginalFont = GetFont_Actual( this->m_iOriginalFontWhenSBCSOverriden );
            //  pGlyph = &pOriginalFont->mGlyphs[ uiLetter & 0xff ];

            macro_rules! ASSIGN_WITH_ROUNDING {
                ($dst:expr, $src:expr) => {
                    if self.mbRoundCalcs {
                        $dst = unsafe { Round((self.m_fAltSBCSFontScaleFactor * $src as f32) as f32) as i32 as f32 };
                    } else {
                        $dst = self.m_fAltSBCSFontScaleFactor * $src as f32;
                    }
                };
            }

            ASSIGN_WITH_ROUNDING!(self.m_AsianGlyph.baseline, pGlyph.baseline);
            ASSIGN_WITH_ROUNDING!(self.m_AsianGlyph.height, pGlyph.height);
            ASSIGN_WITH_ROUNDING!(self.m_AsianGlyph.horizAdvance, pGlyph.horizAdvance);
            //  m_AsianGlyph.horizOffset = /*Round*/( m_fAltSBCSFontScaleFactor * pGlyph->horizOffset );
            ASSIGN_WITH_ROUNDING!(self.m_AsianGlyph.width, pGlyph.width);

            return &self.m_AsianGlyph;
        }

        pGlyph
    }

    pub const fn GetCollapsedAsianCode(&self, uiLetter: u32) -> i32 {
        let mut iCollapsedAsianCode = 0;

        if self.AsianGlyphsAvailable() {
            iCollapsedAsianCode = match GetLanguageEnum() {
                Language_e::eKorean => Korean_CollapseKSC5601HangulCode(uiLetter),
                Language_e::eTaiwanese => Taiwanese_CollapseBig5Code(uiLetter),
                Language_e::eJapanese => Japanese_CollapseShiftJISCode(uiLetter),
                Language_e::eChinese => Chinese_CollapseGBCode(uiLetter),
                Language_e::eThai => Thai_CollapseTISCode(uiLetter),
                _ => {
                    assert!(false);   /* unhandled asian language */
                    0
                }
            };
        }

        iCollapsedAsianCode
    }

    pub fn GetLetterWidth(&mut self, uiLetter: u32) -> i32 {
        let pGlyph = self.GetLetter(uiLetter, core::ptr::null_mut());
        if unsafe { (*pGlyph).width } != 0 {
            unsafe { (*pGlyph).width }
        } else {
            self.mGlyphs[b'.' as usize].width
        }
    }

    pub fn GetLetterHorizAdvance(&mut self, uiLetter: u32) -> i32 {
        let pGlyph = self.GetLetter(uiLetter, core::ptr::null_mut());
        if unsafe { (*pGlyph).horizAdvance } != 0 {
            unsafe { (*pGlyph).horizAdvance }
        } else {
            self.mGlyphs[b'.' as usize].horizAdvance
        }
    }

    pub fn UpdateAsianIfNeeded(&mut self, bForceReEval: bool) {
        // if asian language, then provide an alternative glyph set and fill in relevant fields...
        //
        if self.mHeight != 0 && !self.m_bIsFakeAlienLanguage {    // western charset exists in first place, and isn't alien rubbish?
            let eLanguage = GetLanguageEnum();

            if eLanguage == Language_e::eKorean
                || eLanguage == Language_e::eTaiwanese
                || eLanguage == Language_e::eJapanese
                || eLanguage == Language_e::eChinese
                || eLanguage == Language_e::eThai
            {
                let iCappedHeight = if self.mHeight < 16 { 16 } else { self.mHeight };    // arbitrary limit on small char sizes because Asian chars don't squash well

                unsafe {
                    if self.m_iLanguageModificationCount != (*se_language).modificationCount
                        || !self.AsianGlyphsAvailable()
                        || bForceReEval
                    {
                        self.m_iLanguageModificationCount = (*se_language).modificationCount;

                        let mut iGlyphTPs: i32 = 0;
                        let mut psLang: &str = "";

                        match eLanguage {
                            Language_e::eKorean => {
                                self.m_iAsianGlyphsAcross = Korean_InitFields(&mut iGlyphTPs, &mut psLang);
                            }
                            Language_e::eTaiwanese => {
                                self.m_iAsianGlyphsAcross = Taiwanese_InitFields(&mut iGlyphTPs, &mut psLang);
                            }
                            Language_e::eJapanese => {
                                self.m_iAsianGlyphsAcross = Japanese_InitFields(&mut iGlyphTPs, &mut psLang);
                            }
                            Language_e::eChinese => {
                                self.m_iAsianGlyphsAcross = Chinese_InitFields(&mut iGlyphTPs, &mut psLang);
                            }
                            Language_e::eThai => {
                                self.m_iAsianGlyphsAcross = Thai_InitFields(&mut iGlyphTPs, &mut psLang);

                                if self.m_pThaiData.is_null() {
                                    let psFailureReason = g_ThaiCodes.Init();
                                    if !psFailureReason.is_null() && *psFailureReason == 0 {
                                        self.m_pThaiData = &mut g_ThaiCodes;
                                    } else {
                                        // failed to load a needed file, reset to English...
                                        //
                                        Cvar_Set(b"se_language\0".as_ptr() as *const c_char, b"english\0".as_ptr() as *const c_char);
                                        Com_Error(ERR_DROP, psFailureReason);
                                    }
                                }
                            }
                            _ => {}
                        }

                        // textures need loading...
                        //
                        if self.m_sFontName[0] != 0 {
                            // Use this sometime if we need to do logic to load alternate-height glyphs to better fit other fonts.
                            // (but for now, we just use the one glyph set)
                            //
                        }

                        for i in 0..iGlyphTPs {
                            // (Note!!  assumption for S,T calculations: all Asian glyph textures pages are square except for last one)
                            //
                            let mut sTemp: [c_char; MAX_QPATH] = [0; MAX_QPATH];
                            Com_sprintf(
                                sTemp.as_mut_ptr(),
                                MAX_QPATH,
                                b"fonts/%s_%d_1024_%d\0".as_ptr() as *const c_char,
                                psLang.as_ptr() as *const c_char,
                                1024 / self.m_iAsianGlyphsAcross,
                                i,
                            );
                            //
                            // returning 0 here will automatically inhibit Asian glyph calculations at runtime...
                            //
                            self.m_hAsianShaders[i as usize] = RE_RegisterShaderNoMip(sTemp.as_ptr());
                        }

                        // for now I'm hardwiring these, but if we ever have more than one glyph set per language then they'll be changed...
                        //
                        self.m_iAsianPagesLoaded = iGlyphTPs;    // not necessarily true, but will be safe, and show up obvious if something missing
                        self.m_bAsianLastPageHalfHeight = true;
                    }
                }

                if bForceReEval {
                    // now init the Asian member glyph fields to make them come out the same size as the western ones
                    //  that they serve as an alternative for...
                    //
                    self.m_AsianGlyph.width = iCappedHeight;    // square Asian chars same size as height of western set
                    self.m_AsianGlyph.height = iCappedHeight;   // ""
                    match eLanguage {
                        Language_e::eKorean => {
                            self.m_AsianGlyph.horizAdvance = iCappedHeight - 1;    //korean has a small amount of space at the edge of the glyph
                        }
                        Language_e::eTaiwanese | Language_e::eJapanese | Language_e::eChinese => {
                            self.m_AsianGlyph.horizAdvance = iCappedHeight + 3;    // need to force some spacing for these
                        }
                        _ => {
                            self.m_AsianGlyph.horizAdvance = iCappedHeight;    //default
                        }
                    }
                    //  case eThai:  // this is done dynamically elsewhere, since Thai glyphs are variable width
                    self.m_AsianGlyph.horizOffset = 0;    // ""
                    self.m_AsianGlyph.baseline = self.mAscender + ((iCappedHeight - self.mHeight) >> 1);
                }
            } else {
                // not using Asian...
                //
                self.FlagNoAsianGlyphs();
            }
        } else {
            // no western glyphs available, so don't attempt to match asian...
            //
            self.FlagNoAsianGlyphs();
        }
    }
}

// ensure any GetFont calls that need SBCS overriding (such as when playing in Russian) have the appropriate stuff done...
//
fn GetFont_SBCSOverride(pFont: *mut CFontInfo, eLanguageSBCS: Language_e, psLanguageNameSBCS: *const c_char) -> *mut CFontInfo {
    unsafe {
        if !(*pFont).m_bIsFakeAlienLanguage {
            if GetLanguageEnum() == eLanguageSBCS {
                if (*pFont).m_iAltSBCSFont == -1 {   // no reg attempted yet?
                    // need to register this alternative SBCS font...
                    //
                    let mut sTemp: [c_char; MAX_QPATH] = [0; MAX_QPATH];
                    Com_sprintf(
                        sTemp.as_mut_ptr(),
                        MAX_QPATH,
                        b"%s/%s\0".as_ptr() as *const c_char,
                        COM_SkipPath((*pFont).m_sFontName.as_ptr()),
                        psLanguageNameSBCS,
                    );   // ensure unique name (eg: "lcd/russian")

                    let iAltFontIndex = RE_RegisterFont(sTemp.as_ptr());
                    let pAltFont = GetFont_Actual(iAltFontIndex);
                    if !pAltFont.is_null() {
                        // work out the scaling factor for this font's glyphs...( round it to 1 decimal place to cut down on silly scale factors like 0.53125 )
                        //
                        (*pAltFont).m_fAltSBCSFontScaleFactor = RoundTenth((*pFont).GetPointSize() as f32 / (*pAltFont).GetPointSize() as f32);
                        //
                        // then override with the main properties of the original font...
                        //
                        (*pAltFont).mPointSize = (*pFont).GetPointSize();    //(float) pAltFont->GetPointSize() * pAltFont->m_fAltSBCSFontScaleFactor;
                        (*pAltFont).mHeight = (*pFont).GetHeight();    //(float) pAltFont->GetHeight()    * pAltFont->m_fAltSBCSFontScaleFactor;
                        (*pAltFont).mAscender = (*pFont).GetAscender();    //(float) pAltFont->GetAscender()   * pAltFont->m_fAltSBCSFontScaleFactor;
                        (*pAltFont).mDescender = (*pFont).GetDescender();    //(float) pAltFont->GetDescender()  * pAltFont->m_fAltSBCSFontScaleFactor;

                        //  pAltFont->mPointSize = (float) pAltFont->GetPointSize() * pAltFont->m_fAltSBCSFontScaleFactor;
                        //  pAltFont->mHeight    = (float) pAltFont->GetHeight()    * pAltFont->m_fAltSBCSFontScaleFactor;
                        //  pAltFont->mAscender  = (float) pAltFont->GetAscender()   * pAltFont->m_fAltSBCSFontScaleFactor;
                        //  pAltFont->mDescender = (float) pAltFont->GetDescender()  * pAltFont->m_fAltSBCSFontScaleFactor;

                        (*pAltFont).mbRoundCalcs = true;
                        (*pAltFont).m_iOriginalFontWhenSBCSOverriden = (*pFont).m_iThisFont;
                    }
                    (*pFont).m_iAltSBCSFont = iAltFontIndex;
                }

                if (*pFont).m_iAltSBCSFont > 0 {
                    return GetFont_Actual((*pFont).m_iAltSBCSFont);
                }
            }
        }
    }

    core::ptr::null_mut()
}

fn GetFont(index: i32) -> *mut CFontInfo {
    let pFont = GetFont_Actual(index);

    if !pFont.is_null() {
        unsafe {
            // any SBCS overrides? (this has to be pretty quick, and is (sort of))...
            //
            let mut i = 0;
            loop {
                if g_SBCSOverrideLanguages[i].m_psName.is_null() {
                    break;
                }

                let pAltFont = GetFont_SBCSOverride(pFont, g_SBCSOverrideLanguages[i].m_eLanguage, g_SBCSOverrideLanguages[i].m_psName);
                if !pAltFont.is_null() {
                    return pAltFont;
                }

                i += 1;
            }
        }
    }

    pFont
}

pub fn RE_Font_StrLenPixels(psText: *const c_char, iFontHandle: c_int, fScale: f32) -> i32 {
    let mut iMaxWidth: i32 = 0;
    let mut iThisWidth: i32 = 0;

    let curfont = GetFont(iFontHandle);
    if curfont.is_null() {
        return 0;
    }

    unsafe {
        let mut fScaleA = fScale;
        if Language_IsAsian() && fScale > 0.7f32 {
            fScaleA = fScale * 0.75f32;
        }

        let mut psText_mut = psText;
        while *psText_mut != 0 {
            let mut iAdvanceCount: i32 = 0;
            let uiLetter = AnyLanguage_ReadCharFromString(psText_mut, &mut iAdvanceCount, core::ptr::null_mut());
            psText_mut = psText_mut.add(iAdvanceCount as usize);

            if uiLetter == '^' as u32 {
                if *psText_mut >= '0' as i32 as u8 as c_char && *psText_mut <= '9' as i32 as u8 as c_char {
                    let mut iAdvanceCount2: i32 = 0;
                    let _uiLetter2 = AnyLanguage_ReadCharFromString(psText_mut, &mut iAdvanceCount2, core::ptr::null_mut());
                    psText_mut = psText_mut.add(iAdvanceCount2 as usize);
                    continue;
                }
            }

            if uiLetter == 0x0A {
                iThisWidth = 0;
            } else {
                let iPixelAdvance = (*curfont).GetLetterHorizAdvance(uiLetter);

                let fValue = iPixelAdvance as f32 * if uiLetter > g_iNonScaledCharRange as u32 { fScaleA } else { fScale };
                iThisWidth += if (*curfont).mbRoundCalcs { Round(fValue) } else { fValue as i32 };
                if iThisWidth > iMaxWidth {
                    iMaxWidth = iThisWidth;
                }
            }
        }

        iMaxWidth
    }
}

// not really a font function, but keeps naming consistant...
//
pub fn RE_Font_StrLenChars(psText: *const c_char) -> i32 {
    // logic for this function's letter counting must be kept same in this function and RE_Font_DrawString()
    //
    let mut iCharCount: i32 = 0;

    unsafe {
        let mut psText_mut = psText;
        while *psText_mut != 0 {
            // in other words, colour codes and CR/LF don't count as chars, all else does...
            //
            let mut iAdvanceCount: i32 = 0;
            let uiLetter = AnyLanguage_ReadCharFromString(psText_mut, &mut iAdvanceCount, core::ptr::null_mut());
            psText_mut = psText_mut.add(iAdvanceCount as usize);

            match uiLetter {
                c if c == '^' as u32 => {
                    if *psText_mut >= '0' as i32 as u8 as c_char && *psText_mut <= '9' as i32 as u8 as c_char {
                        psText_mut = psText_mut.add(1);
                    } else {
                        iCharCount += 1;
                    }
                }                         // colour code (note next-char skip)
                10 => {}                  // linefeed
                13 => {}                  // return
                c if c == '_' as u32 => {
                    iCharCount += if GetLanguageEnum() == Language_e::eThai && ((*psText_mut) as u8 >= TIS_GLYPHS_START) { 0 } else { 1 };
                }   // special word-break hack
                _ => {
                    iCharCount += 1;
                }
            }
        }

        iCharCount
    }
}

pub fn RE_Font_HeightPixels(iFontHandle: c_int, fScale: f32) -> i32 {
    let curfont = GetFont(iFontHandle);
    if !curfont.is_null() {
        unsafe {
            let fValue = (*curfont).GetPointSize() as f32 * fScale;
            return if (*curfont).mbRoundCalcs { Round(fValue) } else { fValue as i32 };
        }
    }
    0
}

// iMaxPixelWidth is -1 for "all of string", else pixel display count...
//
pub fn RE_Font_DrawString(
    ox: i32,
    oy: i32,
    psText: *const c_char,
    rgba: *const f32,
    iFontHandle: c_int,
    iMaxPixelWidth: i32,
    fScale: f32,
) {
    static mut gbInShadow: qboolean = false;    // MUST default to this
    let mut x: i32;
    let mut y: i32;
    let mut colour: i32;
    let mut offset: i32;
    let pLetter: *const glyphInfo_t_full;
    let hShader: qhandle_t;

    assert!(!psText.is_null());

    unsafe {
        if (iFontHandle & STYLE_BLINK) != 0 {
            if ((Sys_Milliseconds() >> 7) & 1) != 0 {
                return;
            }
        }

        let curfont = GetFont(iFontHandle);
        if curfont.is_null() || psText.is_null() {
            return;
        }

        let mut fScaleA = fScale;
        let mut iAsianYAdjust: i32 = 0;
        if Language_IsAsian() && fScale > 0.7f32 {
            fScaleA = fScale * 0.75f32;
            iAsianYAdjust = ((((*curfont).GetPointSize() as f32 * fScale) - ((*curfont).GetPointSize() as f32 * fScaleA)) / 2.0f32) as i32;
        }

        // Draw a dropshadow if required
        if (iFontHandle & STYLE_DROPSHADOW) != 0 {
            offset = Round((*curfont).GetPointSize() as f32 * fScale * 0.075f32);

            static v4DKGREY2: [f32; 4] = [0.15f32, 0.15f32, 0.15f32, 1.0f32];

            gbInShadow = true;
            RE_Font_DrawString(ox + offset, oy + offset, psText, v4DKGREY2.as_ptr(), iFontHandle & SET_MASK, iMaxPixelWidth, fScale);
            gbInShadow = false;
        }

        RE_SetColor(rgba);

        x = ox;
        oy = oy + Round(((*curfont).GetHeight() - ((*curfont).GetDescender() >> 1)) as f32 * fScale);

        let mut bNextTextWouldOverflow: qboolean = false;
        let mut psText_mut = psText;
        while *psText_mut != 0 && !bNextTextWouldOverflow {
            let mut iAdvanceCount: i32 = 0;
            let uiLetter = AnyLanguage_ReadCharFromString(psText_mut, &mut iAdvanceCount, core::ptr::null_mut());
            psText_mut = psText_mut.add(iAdvanceCount as usize);

            match uiLetter {
                10 => {   //linefeed
                    x = ox;
                    oy += Round((*curfont).GetPointSize() as f32 * fScale);
                    if Language_IsAsian() {
                        oy += 4;   // this only comes into effect when playing in asian for "A long time ago in a galaxy" etc, all other text is line-broken in feeder functions
                    }
                }
                13 => {   // Return
                }
                32 => {   // Space
                    let pLetter_space = (*curfont).GetLetter(' ' as u32, core::ptr::null_mut());
                    x += Round((*pLetter_space).horizAdvance as f32 * fScale);
                    bNextTextWouldOverflow =
                        if iMaxPixelWidth != -1 && ((x - ox) > iMaxPixelWidth) { true } else { false };   // yeuch
                }
                c if c == '_' as u32 => {   // has a special word-break usage if in Thai (and followed by a thai char), and should not be displayed, else treat as normal
                    if GetLanguageEnum() == Language_e::eThai && ((*psText_mut) as u8 >= TIS_GLYPHS_START) {
                        // break;
                    } else {
                        // else drop through and display as normal...
                        let pLetter_underscore = (*curfont).GetLetter('_' as u32, core::ptr::null_mut());
                        if (*pLetter_underscore).width != 0 {
                            let fThisScale = if uiLetter > g_iNonScaledCharRange as u32 { fScaleA } else { fScale };

                            let iAdvancePixels = Round((*pLetter_underscore).horizAdvance as f32 * fThisScale);
                            bNextTextWouldOverflow = if iMaxPixelWidth != -1 && (((x + iAdvancePixels) - ox) > iMaxPixelWidth) { true } else { false };   // yeuch
                            if !bNextTextWouldOverflow {
                                let y_calc = oy - (if (*curfont).mbRoundCalcs { Round((*pLetter_underscore).baseline as f32 * fThisScale) } else { ((*pLetter_underscore).baseline as f32 * fThisScale) as i32 });
                                RE_StretchPic(
                                    (x + Round((*pLetter_underscore).horizOffset as f32 * fScale)) as f32,    // float x
                                    if uiLetter > g_iNonScaledCharRange as u32 { (y_calc - iAsianYAdjust) as f32 } else { y_calc as f32 },    // float y
                                    if (*curfont).mbRoundCalcs { Round((*pLetter_underscore).width as f32 * fThisScale) } else { ((*pLetter_underscore).width as f32 * fThisScale) as i32 } as f32,    // float w
                                    if (*curfont).mbRoundCalcs { Round((*pLetter_underscore).height as f32 * fThisScale) } else { ((*pLetter_underscore).height as f32 * fThisScale) as i32 } as f32,    // float h
                                    (*pLetter_underscore).s,    // float s1
                                    (*pLetter_underscore).t,    // float t1
                                    (*pLetter_underscore).s2,    // float s2
                                    (*pLetter_underscore).t2,    // float t2
                                    (*curfont).GetShader(),    // qhandle_t hShader
                                );

                                x += iAdvancePixels;
                            }
                        }
                    }
                }
                c if c == '^' as u32 => {   // purposely falls through
                    if uiLetter != '_' as u32 {   // necessary because of fallthrough above
                        if *psText_mut >= '0' as i32 as u8 as c_char && *psText_mut <= '9' as i32 as u8 as c_char {
                            colour = ColorIndex(*psText_mut);
                            if !gbInShadow {
                                // Need to handle g_color_table properly
                                // For now, just skip
                            }
                            psText_mut = psText_mut.add(1);
                            continue;
                        }
                    }
                    // fallthrough to default
                    let pLetter_caret = (*curfont).GetLetter('^' as u32, core::ptr::null_mut());
                    if (*pLetter_caret).width != 0 {
                        let mut hShader_local: c_int = 0;
                        let pLetter_final = (*curfont).GetLetter('^' as u32, &mut hShader_local);
                        let fThisScale = if uiLetter > g_iNonScaledCharRange as u32 { fScaleA } else { fScale };

                        let iAdvancePixels = Round((*pLetter_final).horizAdvance as f32 * fThisScale);
                        bNextTextWouldOverflow = if iMaxPixelWidth != -1 && (((x + iAdvancePixels) - ox) > iMaxPixelWidth) { true } else { false };   // yeuch
                        if !bNextTextWouldOverflow {
                            let y_calc = oy - (if (*curfont).mbRoundCalcs { Round((*pLetter_final).baseline as f32 * fThisScale) } else { ((*pLetter_final).baseline as f32 * fThisScale) as i32 });
                            let y_final = if uiLetter > g_iNonScaledCharRange as u32 { y_calc - iAsianYAdjust } else { y_calc };
                            if (*curfont).m_fAltSBCSFontScaleFactor != -1.0f32 {
                                // y += 3;  // I'm sick and tired of going round in circles trying to do this legally, so bollocks to it
                            }

                            RE_StretchPic(
                                (x + Round((*pLetter_final).horizOffset as f32 * fScale)) as f32,    // float x
                                y_final as f32,    // float y
                                if (*curfont).mbRoundCalcs { Round((*pLetter_final).width as f32 * fThisScale) } else { ((*pLetter_final).width as f32 * fThisScale) as i32 } as f32,    // float w
                                if (*curfont).mbRoundCalcs { Round((*pLetter_final).height as f32 * fThisScale) } else { ((*pLetter_final).height as f32 * fThisScale) as i32 } as f32,    // float h
                                (*pLetter_final).s,    // float s1
                                (*pLetter_final).t,    // float t1
                                (*pLetter_final).s2,    // float s2
                                (*pLetter_final).t2,    // float t2
                                hShader_local,    // qhandle_t hShader
                            );

                            x += iAdvancePixels;
                        }
                    }
                }
                _ => {
                    let mut hShader_local: c_int = 0;
                    let pLetter_default = (*curfont).GetLetter(uiLetter, &mut hShader_local);    // Description of pLetter
                    let pLetter_use = if (*pLetter_default).width == 0 {
                        (*curfont).GetLetter('.' as u32, core::ptr::null_mut())
                    } else {
                        pLetter_default
                    };

                    let fThisScale = if uiLetter > g_iNonScaledCharRange as u32 { fScaleA } else { fScale };

                    // sigh, super-language-specific hack...
                    //
                    if uiLetter == TIS_SARA_AM && GetLanguageEnum() == Language_e::eThai {
                        x -= Round(7.0f32 * fThisScale);
                    }

                    let iAdvancePixels = Round((*pLetter_use).horizAdvance as f32 * fThisScale);
                    bNextTextWouldOverflow =
                        if iMaxPixelWidth != -1 && (((x + iAdvancePixels) - ox) > iMaxPixelWidth) { true } else { false };   // yeuch
                    if !bNextTextWouldOverflow {
                        // this 'mbRoundCalcs' stuff is crap, but the only way to make the font code work. Sigh...
                        //
                        let y_calc = oy - (if (*curfont).mbRoundCalcs { Round((*pLetter_use).baseline as f32 * fThisScale) } else { ((*pLetter_use).baseline as f32 * fThisScale) as i32 });
                        let mut y = y_calc;
                        if (*curfont).m_fAltSBCSFontScaleFactor != -1.0f32 {
                            y += 3;   // I'm sick and tired of going round in circles trying to do this legally, so bollocks to it
                        }

                        let y_final = if uiLetter > g_iNonScaledCharRange as u32 { y - iAsianYAdjust } else { y };

                        RE_StretchPic(
                            (x + Round((*pLetter_use).horizOffset as f32 * fScale)) as f32,    // float x
                            y_final as f32,    // float y
                            if (*curfont).mbRoundCalcs { Round((*pLetter_use).width as f32 * fThisScale) } else { ((*pLetter_use).width as f32 * fThisScale) as i32 } as f32,    // float w
                            if (*curfont).mbRoundCalcs { Round((*pLetter_use).height as f32 * fThisScale) } else { ((*pLetter_use).height as f32 * fThisScale) as i32 } as f32,    // float h
                            (*pLetter_use).s,    // float s1
                            (*pLetter_use).t,    // float t1
                            (*pLetter_use).s2,    // float s2
                            (*pLetter_use).t2,    // float t2
                            hShader_local,    // qhandle_t hShader
                        );

                        x += iAdvancePixels;
                    }
                }
            }
        }
        //let it remember the old color //RE_SetColor(NULL);;
    }
}

pub fn RE_RegisterFont(psName: *const c_char) -> c_int {
    unsafe {
        if g_mapFontIndexes.is_null() {
            g_mapFontIndexes = Box::into_raw(Box::new(std::collections::HashMap::new()));
        }

        // Check if already registered
        // This would require C string handling - simplified for now
        // For now, just create a new font
        // In a full implementation, we'd need proper string handling

        // Simplified: always create new
        let pFont = Box::into_raw(Box::new(CFontInfo {
            mGlyphs: [glyphInfo_t_full {
                width: 0,
                height: 0,
                top: 0,
                baseline: 0,
                leftSideBearing: 0,
                advance: 0,
                horizOffset: 0,
                horizAdvance: 0,
                s: 0.0,
                t: 0.0,
                s2: 0.0,
                t2: 0.0,
                charWidth: 0,
                imageWidth: 0,
                imageHeight: 0,
                s_padding: 0.0,
                t_padding: 0.0,
            }; GLYPH_COUNT],
            mShader: 0,
            m_hAsianShaders: [0; GLYPH_MAX_ASIAN_SHADERS],
            m_AsianGlyph: glyphInfo_t_full {
                width: 0,
                height: 0,
                top: 0,
                baseline: 0,
                leftSideBearing: 0,
                advance: 0,
                horizOffset: 0,
                horizAdvance: 0,
                s: 0.0,
                t: 0.0,
                s2: 0.0,
                t2: 0.0,
                charWidth: 0,
                imageWidth: 0,
                imageHeight: 0,
                s_padding: 0.0,
                t_padding: 0.0,
            },
            m_iAsianGlyphsAcross: 0,
            m_iAsianPagesLoaded: 0,
            m_bAsianLastPageHalfHeight: false,
            m_iLanguageModificationCount: 0,
            m_pThaiData: core::ptr::null_mut(),
            m_sFontName: [0; MAX_QPATH],
            mPointSize: 0,
            mHeight: 0,
            mAscender: 0,
            mDescender: 0,
            mbRoundCalcs: false,
            m_iThisFont: 0,
            m_iAltSBCSFont: -1,
            m_iOriginalFontWhenSBCSOverriden: -1,
            m_fAltSBCSFontScaleFactor: -1.0,
            m_bIsFakeAlienLanguage: false,
        }));

        if !(*pFont).GetPointSize() > 0 {
            g_iCurrentFontIndex += 1;
            return g_iCurrentFontIndex - 1;
        }

        0
    }
}

pub fn R_InitFonts() {
    unsafe {
        g_iCurrentFontIndex = 1;       // entry 0 is reserved for "missing/invalid"
        g_iNonScaledCharRange = i32::MAX;   // default all chars to have no special scaling (other than user supplied)

        if g_vFontArray.is_null() {
            g_vFontArray = Box::into_raw(Box::new(Vec::new()));
        }

        if g_mapFontIndexes.is_null() {
            g_mapFontIndexes = Box::into_raw(Box::new(std::collections::HashMap::new()));
        }
    }
}

pub fn R_ShutdownFonts() {
    unsafe {
        if !g_vFontArray.is_null() {
            for i in 1..g_iCurrentFontIndex {
                // Delete fonts
                if let Some(pFont) = (*g_vFontArray).get_mut(i as usize) {
                    if !(*pFont).is_null() {
                        let _ = Box::from_raw(*pFont);
                    }
                }
            }
            (*g_vFontArray).clear();
        }

        if !g_mapFontIndexes.is_null() {
            (*g_mapFontIndexes).clear();
        }

        g_iCurrentFontIndex = 1;   // entry 0 is reserved for "missing/invalid"

        g_ThaiCodes.Clear();
    }
}

// this is only really for debugging while tinkering with fonts, but harmless to leave in...
//
pub fn R_ReloadFonts_f() {
    // Complex implementation omitted - would require extensive string/font handling
    unsafe {
        Com_Printf(b"Font reloading not yet fully implemented.\0".as_ptr() as *const c_char);
    }
}

// end
