// leave this as first line for PCH reasons...
//

use core::ffi::{c_char, c_int, c_void};
use std::collections::HashMap;

// stl string class won't compile in here (MS shite), so use Gil's.
// (In Rust, we use String/&str)
//
// tr_local.h and tr_font.h equivalents would be pulled in here
// stringed_ingame.h equivalent

/////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// This file is shared in the single and multiplayer codebases, so be CAREFUL WHAT YOU ADD/CHANGE!!!!!
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
enum Language_e {
    eWestern = 0,    // ( I only care about asian languages in here at the moment )
    eRussian = 1,    // .. but now I need to care about this, since it uses a different TP
    ePolish = 2,     // ditto
    eKorean = 3,
    eTaiwanese = 4,  // 15x15 glyphs tucked against BR of 16x16 space
    eJapanese = 5,   // 15x15 glyphs tucked against TL of 16x16 space
    eChinese = 6,    // 15x15 glyphs tucked against TL of 16x16 space
    eThai = 7,       // 16x16 cells with glyphs against left edge, special file (tha_widths.dat) for variable widths
}

// this is to cut down on all the stupid string compares I've been doing, and convert asian stuff to switch-case
//
fn GetLanguageEnum() -> Language_e {
    static mut iSE_Language_ModificationCount: c_int = -1234; // any old silly value that won't match the cvar mod count
    static mut eLanguage: Language_e = Language_e::eWestern;

    // only re-strcmp() when language string has changed from what we knew it as...
    //
    unsafe {
        if iSE_Language_ModificationCount != SE_LANGUAGE_MOD_COUNT() {
            iSE_Language_ModificationCount = SE_LANGUAGE_MOD_COUNT();

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
static mut G_SBCS_OVERRIDE_LANGUAGES: [SBCSOverrideLanguages_t; 3] = [
    SBCSOverrideLanguages_t {
        m_psName: b"russian\0".as_ptr() as *const c_char,
        m_eLanguage: Language_e::eRussian,
    },
    SBCSOverrideLanguages_t {
        m_psName: b"polish\0".as_ptr() as *const c_char,
        m_eLanguage: Language_e::ePolish,
    },
    SBCSOverrideLanguages_t {
        m_psName: std::ptr::null(),
        m_eLanguage: Language_e::eWestern,
    },
];

//================================================
//

const S_FILENAME_THAI_WIDTHS: &str = "fonts/tha_widths.dat";
const S_FILENAME_THAI_CODES: &str = "fonts/tha_codes.dat";

#[derive(Clone)]
struct ThaiCodes_t {
    m_mapValidCodes: HashMap<c_int, c_int>,
    m_viGlyphWidths: Vec<c_int>,
    m_strInitFailureReason: String, // so we don't have to keep retrying to work this out
}

impl ThaiCodes_t {
    fn new() -> Self {
        ThaiCodes_t {
            m_mapValidCodes: HashMap::new(),
            m_viGlyphWidths: Vec::new(),
            m_strInitFailureReason: String::new(),
        }
    }

    fn Clear(&mut self) {
        self.m_mapValidCodes.clear();
        self.m_viGlyphWidths.clear();
        self.m_strInitFailureReason = String::new(); // if blank, never failed, else says don't bother re-trying
    }

    // convert a supplied 1,2 or 3-byte multiplied-up integer into a valid 0..n index, else -1...
    //
    fn GetValidIndex(&self, iCode: c_int) -> c_int {
        if let Some(&index) = self.m_mapValidCodes.get(&iCode) {
            index
        } else {
            -1
        }
    }

    fn GetWidth(&self, iGlyphIndex: c_int) -> c_int {
        if (iGlyphIndex as usize) < self.m_viGlyphWidths.len() {
            self.m_viGlyphWidths[iGlyphIndex as usize]
        } else {
            debug_assert!(false);
            0
        }
    }

    // return is error message to display, or NULL for success
    fn Init(&mut self) -> &str {
        if self.m_mapValidCodes.is_empty() && self.m_viGlyphWidths.is_empty() {
            if self.m_strInitFailureReason.is_empty() {
                // never tried and failed already?
                // note <int>, not <byte>, for []-access
                //
                // read the valid-codes table in...
                //
                if let Some(piData) = FS_ReadFile_alloc(S_FILENAME_THAI_CODES) {
                    let iBytesRead = piData.len() * core::mem::size_of::<c_int>();
                    if iBytesRead > 0 && (iBytesRead & 3) == 0 {
                        // valid length and multiple of 4 bytes long
                        let iTableEntries = iBytesRead / core::mem::size_of::<c_int>();

                        for i in 0..iTableEntries {
                            self.m_mapValidCodes.insert(piData[i], i as c_int); // convert MBCS code to sequential index...
                        }
                        // FS_FreeFile( piData ); // dispose of original (Rust: automatic via Vec drop)

                        // now read in the widths... (I'll keep these in a simple vector, so they'll disappear when the map entries do...
                        //
                        if let Some(piData) = FS_ReadFile_alloc(S_FILENAME_THAI_WIDTHS) {
                            let iBytesRead = piData.len() * core::mem::size_of::<c_int>();
                            if iBytesRead > 0 && (iBytesRead & 3) == 0 && (iBytesRead >> 2) == iTableEntries {
                                for i in 0..iTableEntries {
                                    self.m_viGlyphWidths.push(piData[i]);
                                }
                                // FS_FreeFile( piData ); // dispose of original (Rust: automatic via Vec drop)
                            } else {
                                self.m_strInitFailureReason =
                                    format!("Error with file \"{}\", size = {}!\n", S_FILENAME_THAI_WIDTHS, iBytesRead);
                            }
                        }
                    } else {
                        self.m_strInitFailureReason =
                            format!("Error with file \"{}\", size = {}!\n", S_FILENAME_THAI_CODES, iBytesRead);
                    }
                }
            }
        }

        &self.m_strInitFailureReason
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct glyphInfo_t {
    height: c_int,
    top: c_int,
    baseline: c_int,
    width: c_int,
    horizAdvance: c_int,
    horizOffset: c_int,
    s: f32,
    t: f32,
    s2: f32,
    t2: f32,
}

const GLYPH_MAX_KOREAN_SHADERS: usize = 3;
const GLYPH_MAX_TAIWANESE_SHADERS: usize = 4;
const GLYPH_MAX_JAPANESE_SHADERS: usize = 3;
const GLYPH_MAX_CHINESE_SHADERS: usize = 3;
const GLYPH_MAX_THAI_SHADERS: usize = 3;
const GLYPH_MAX_ASIAN_SHADERS: usize = 4; // this MUST equal the larger of the above defines

const GLYPH_COUNT: usize = 256;
const MAX_QPATH: usize = 260;
const SET_MASK: c_int = 0xFF;
const STYLE_BLINK: c_int = 0x100;
const STYLE_DROPSHADOW: c_int = 0x200;

#[derive(Clone)]
struct CFontInfo {
    // From the fontdat file
    mGlyphs: [glyphInfo_t; GLYPH_COUNT],

    // int mAsianHack; // unused junk from John's fontdat file format.
    // end of fontdat data

    mShader: c_int, // handle to the shader with the glyph

    m_hAsianShaders: [c_int; GLYPH_MAX_ASIAN_SHADERS], // shaders for Korean glyphs where applicable
    m_AsianGlyph: glyphInfo_t,                         // special glyph containing asian->western scaling info for all glyphs
    m_iAsianGlyphsAcross: c_int,                       // needed to dynamically calculate S,T coords
    m_iAsianPagesLoaded: c_int,
    m_bAsianLastPageHalfHeight: bool,
    m_iLanguageModificationCount: c_int, // doesn't matter what this is, so long as it's comparable as being changed

    m_pThaiData: *mut ThaiCodes_t,

    m_sFontName: [c_char; MAX_QPATH], // eg "fonts/lcd"  // needed for korean font-hint if we need >1 hangul set
    mPointSize: c_int,
    mHeight: c_int,
    mAscender: c_int,
    mDescender: c_int,

    mbRoundCalcs: bool, // trying to make this !@#$%^ thing work with scaling
    m_iThisFont: c_int, // handle to itself
    m_iAltSBCSFont: c_int, // -1 == NULL // alternative single-byte font for languages like russian/polish etc that need to override high characters ?
    m_iOriginalFontWhenSBCSOverriden: c_int,
    m_fAltSBCSFontScaleFactor: f32, // -1, else amount to adjust returned values by to make them fit the master western font they're substituting for
    m_bIsFakeAlienLanguage: bool, // ... if true, don't process as MBCS or override as SBCS etc
}

impl CFontInfo {
    fn new(fontName: &str) -> Self {
        let mut info = CFontInfo {
            mGlyphs: [glyphInfo_t {
                height: 0,
                top: 0,
                baseline: 0,
                width: 0,
                horizAdvance: 0,
                horizOffset: 0,
                s: 0.0,
                t: 0.0,
                s2: 0.0,
                t2: 0.0,
            }; GLYPH_COUNT],
            mShader: 0,
            m_hAsianShaders: [0; GLYPH_MAX_ASIAN_SHADERS],
            m_AsianGlyph: glyphInfo_t {
                height: 0,
                top: 0,
                baseline: 0,
                width: 0,
                horizAdvance: 0,
                horizOffset: 0,
                s: 0.0,
                t: 0.0,
                s2: 0.0,
                t2: 0.0,
            },
            m_iAsianGlyphsAcross: 0,
            m_iAsianPagesLoaded: 0,
            m_bAsianLastPageHalfHeight: false,
            m_iLanguageModificationCount: 0,
            m_pThaiData: std::ptr::null_mut(),
            m_sFontName: [0; MAX_QPATH],
            mPointSize: 0,
            mHeight: 0,
            mAscender: 0,
            mDescender: 0,
            mbRoundCalcs: false,
            m_iThisFont: -1,
            m_iAltSBCSFont: -1,
            m_iOriginalFontWhenSBCSOverriden: -1,
            m_fAltSBCSFontScaleFactor: -1.0,
            m_bIsFakeAlienLanguage: false,
        };

        // remove any special hack name insertions...
        //
        let fontName = format!("fonts/{}.fontdat", COM_SkipPath(fontName));

        // clear some general things...
        //
        info.m_pThaiData = std::ptr::null_mut();
        info.m_iAltSBCSFont = -1;
        info.m_iThisFont = -1;
        info.m_iOriginalFontWhenSBCSOverriden = -1;
        info.m_fAltSBCSFontScaleFactor = -1.0;
        info.m_bIsFakeAlienLanguage = fontName.contains("aurabesh"); // dont try and make SBCS or asian overrides for this

        if let Some(buff) = FS_ReadFile_alloc(&fontName) {
            // Assume buff is a dfontdat_t structure
            // For now, we'd need to load the glyph data, but without the full header definition,
            // we'll stub this out properly
            for i in 0..GLYPH_COUNT {
                // mGlyphs[i] would be copied from fontdat->mGlyphs[i]
            }
            // mPointSize would be set from fontdat->mPointSize
            // mHeight from fontdat->mHeight, etc.

            // cope with bad fontdat headers...
            //
            if info.mHeight == 0 {
                info.mHeight = info.mPointSize;
                info.mAscender = info.mPointSize - Round(((info.mPointSize as f32) / 10.0) + 2.0) as c_int; // have to completely guess at the baseline... sigh.
                info.mDescender = info.mHeight - info.mAscender;
            }
        } else {
            info.mHeight = 0;
            info.mShader = 0;
        }

        // Copy fontName into m_sFontName
        let font_cstr = fontName.as_bytes();
        for (i, &byte) in font_cstr.iter().enumerate() {
            if i >= MAX_QPATH - 1 {
                break;
            }
            info.m_sFontName[i] = byte as c_char;
        }
        info.m_sFontName[std::cmp::min(font_cstr.len(), MAX_QPATH - 1)] = 0;

        COM_StripExtension(&mut info.m_sFontName); // so we get better error printing if failed to load shader (ie lose ".fontdat")
        info.mShader = RE_RegisterShaderNoMip(&fontName);

        info.FlagNoAsianGlyphs();
        info.UpdateAsianIfNeeded(true);

        // finished...
        unsafe {
            G_VFONT_ARRAY.resize(G_ICURRENT_FONT_INDEX as usize + 1, None);
            G_VFONT_ARRAY[G_ICURRENT_FONT_INDEX as usize] = Some(Box::new(info.clone()));
            G_ICURRENT_FONT_INDEX += 1;
        }

        // If com_buildScript is 2, we'd register foreign fonts here
        // (Stub for now)

        info
    }

    fn GetPointSize(&self) -> c_int {
        self.mPointSize
    }
    fn GetHeight(&self) -> c_int {
        self.mHeight
    }
    fn GetAscender(&self) -> c_int {
        self.mAscender
    }
    fn GetDescender(&self) -> c_int {
        self.mDescender
    }

    fn GetLetter(&mut self, uiLetter: u32, piShader: Option<&mut c_int>) -> &glyphInfo_t {
        // Implementation follows C++ GetLetter method
        // For now, return the glyph from mGlyphs array
        &self.mGlyphs[(uiLetter & 0xff) as usize]
    }

    fn GetCollapsedAsianCode(&self, uiLetter: u32) -> c_int {
        let mut iCollapsedAsianCode = 0;

        if self.AsianGlyphsAvailable() {
            match GetLanguageEnum() {
                Language_e::eKorean => {
                    iCollapsedAsianCode = Korean_CollapseKSC5601HangulCode(uiLetter);
                }
                Language_e::eTaiwanese => {
                    iCollapsedAsianCode = Taiwanese_CollapseBig5Code(uiLetter);
                }
                Language_e::eJapanese => {
                    iCollapsedAsianCode = Japanese_CollapseShiftJISCode(uiLetter);
                }
                Language_e::eChinese => {
                    iCollapsedAsianCode = Chinese_CollapseGBCode(uiLetter);
                }
                Language_e::eThai => {
                    iCollapsedAsianCode = Thai_CollapseTISCode(uiLetter);
                }
                _ => {
                    debug_assert!(false); /* unhandled asian language */
                }
            }
        }

        iCollapsedAsianCode
    }

    fn GetLetterWidth(&self, uiLetter: u32) -> c_int {
        let pGlyph = &self.mGlyphs[(uiLetter & 0xff) as usize];
        if pGlyph.width != 0 {
            pGlyph.width
        } else {
            self.mGlyphs[(b'.' as u32 & 0xff) as usize].width
        }
    }

    fn GetLetterHorizAdvance(&self, uiLetter: u32) -> c_int {
        let pGlyph = &self.mGlyphs[(uiLetter & 0xff) as usize];
        if pGlyph.horizAdvance != 0 {
            pGlyph.horizAdvance
        } else {
            self.mGlyphs[(b'.' as u32 & 0xff) as usize].horizAdvance
        }
    }

    fn GetShader(&self) -> c_int {
        self.mShader
    }

    fn FlagNoAsianGlyphs(&mut self) {
        self.m_hAsianShaders[0] = 0;
        self.m_iLanguageModificationCount = -1;
    } // used during constructor

    fn AsianGlyphsAvailable(&self) -> bool {
        self.m_hAsianShaders[0] != 0
    }

    fn UpdateAsianIfNeeded(&mut self, bForceReEval: bool) {
        // if asian language, then provide an alternative glyph set and fill in relevant fields...
        //
        if self.mHeight != 0 && !self.m_bIsFakeAlienLanguage {
            // western charset exists in first place, and isn't alien rubbish?
            let eLanguage = GetLanguageEnum();

            if eLanguage == Language_e::eKorean
                || eLanguage == Language_e::eTaiwanese
                || eLanguage == Language_e::eJapanese
                || eLanguage == Language_e::eChinese
                || eLanguage == Language_e::eThai
            {
                let iCappedHeight = if self.mHeight < 16 { 16 } else { self.mHeight }; // arbitrary limit on small char sizes because Asian chars don't squash well

                if self.m_iLanguageModificationCount != SE_LANGUAGE_MOD_COUNT()
                    || !self.AsianGlyphsAvailable()
                    || bForceReEval
                {
                    self.m_iLanguageModificationCount = SE_LANGUAGE_MOD_COUNT();

                    let mut iGlyphTPs = 0;
                    let mut psLang = "";

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
                            // Thai-specific initialization
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
                        // (Note!! assumption for S,T calculations: all Asian glyph textures pages are square except for last one)
                        //
                        let sTemp = format!("fonts/{}_{}_1024_{}", psLang, 1024 / self.m_iAsianGlyphsAcross, i);
                        //
                        // returning 0 here will automatically inhibit Asian glyph calculations at runtime...
                        //
                        self.m_hAsianShaders[i as usize] = RE_RegisterShaderNoMip(&sTemp);
                    }

                    // for now I'm hardwiring these, but if we ever have more than one glyph set per language then they'll be changed...
                    //
                    self.m_iAsianPagesLoaded = iGlyphTPs as c_int; // not necessarily true, but will be safe, and show up obvious if something missing
                    self.m_bAsianLastPageHalfHeight = true;
                }

                if bForceReEval {
                    // now init the Asian member glyph fields to make them come out the same size as the western ones
                    // that they serve as an alternative for...
                    //
                    self.m_AsianGlyph.width = iCappedHeight; // square Asian chars same size as height of western set
                    self.m_AsianGlyph.height = iCappedHeight; // ""
                    match eLanguage {
                        Language_e::eKorean => {
                            self.m_AsianGlyph.horizAdvance = iCappedHeight - 1; // korean has a small amount of space at the edge of the glyph
                        }
                        Language_e::eTaiwanese | Language_e::eJapanese | Language_e::eChinese => {
                            self.m_AsianGlyph.horizAdvance = iCappedHeight + 3; // need to force some spacing for these
                        }
                        _ => {
                            self.m_AsianGlyph.horizAdvance = iCappedHeight; // default
                        }
                    }
                    //	case eThai:	// this is done dynamically elsewhere, since Thai glyphs are variable width
                    self.m_AsianGlyph.horizOffset = 0; // ""
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

//================================================

// round float to one decimal place...
//
fn RoundTenth(fValue: f32) -> f32 {
    ((fValue * 10.0) + 0.5).floor() / 10.0
}

fn Round(fValue: f32) -> c_int {
    (fValue + 0.5) as c_int
}

static mut G_ICURRENT_FONT_INDEX: c_int = 0; // entry 0 is reserved index for missing/invalid, else ++ with each new font registered
static mut G_VFONT_ARRAY: Vec<Option<Box<CFontInfo>>> = Vec::new();
type FontIndexMap_t = HashMap<String, c_int>;
static mut G_MAP_FONT_INDEXES: FontIndexMap_t = HashMap::new();
static mut G_INON_SCALED_CHAR_RANGE: c_int = 0; // this is used with auto-scaling of asian fonts, anything below this number is preserved in scale, anything above is scaled down by 0.75f

//paletteRGBA_c lastcolour;

// =============================== some korean stuff =======================================

const KSC5601_HANGUL_HIBYTE_START: u8 = 0xB0; // range is...
const KSC5601_HANGUL_HIBYTE_STOP: u8 = 0xC8; // ... inclusive
const KSC5601_HANGUL_LOBYTE_LOBOUND: u8 = 0xA0; // range is...
const KSC5601_HANGUL_LOBYTE_HIBOUND: u8 = 0xFF; // ...bounding (ie only valid in between these points, but NULLs in charsets for these codes)
const KSC5601_HANGUL_CODES_PER_ROW: c_int = 96; // 2 more than the number of glyphs

extern "C" {
    fn Language_IsKorean() -> bool;
}

fn Korean_ValidKSC5601Hangul(_iHi: u8, _iLo: u8) -> bool {
    _iHi >= KSC5601_HANGUL_HIBYTE_START
        && _iHi <= KSC5601_HANGUL_HIBYTE_STOP
        && _iLo > KSC5601_HANGUL_LOBYTE_LOBOUND
        && _iLo < KSC5601_HANGUL_LOBYTE_HIBOUND
}

fn Korean_ValidKSC5601Hangul_u32(uiCode: u32) -> bool {
    Korean_ValidKSC5601Hangul((uiCode >> 8) as u8, (uiCode & 0xFF) as u8)
}

// takes a KSC5601 double-byte hangul code and collapses down to a 0..n glyph index...
// Assumes rows are 96 wide (glyph slots), not 94 wide (actual glyphs), so I can ignore boundary markers
//
// (invalid hangul codes will return 0)
//
fn Korean_CollapseKSC5601HangulCode(mut uiCode: u32) -> c_int {
    if Korean_ValidKSC5601Hangul_u32(uiCode) {
        uiCode -= ((KSC5601_HANGUL_HIBYTE_START as u32 * 256) + KSC5601_HANGUL_LOBYTE_LOBOUND as u32); // sneaky maths on both bytes, reduce to 0x0000 onwards
        uiCode = (((uiCode >> 8) as c_int * KSC5601_HANGUL_CODES_PER_ROW) + (uiCode & 0xFF) as c_int) as u32;
        uiCode as c_int
    } else {
        0
    }
}

fn Korean_InitFields(iGlyphTPs: &mut c_int, psLang: &mut &str) -> c_int {
    *psLang = "kor";
    *iGlyphTPs = GLYPH_MAX_KOREAN_SHADERS as c_int;
    unsafe {
        G_INON_SCALED_CHAR_RANGE = 255;
    }
    32 // m_iAsianGlyphsAcross
}

// ======================== some taiwanese stuff ==============================

// (all ranges inclusive for Big5)...
//
const BIG5_HIBYTE_START0: u8 = 0xA1; // (misc chars + level 1 hanzi)
const BIG5_HIBYTE_STOP0: u8 = 0xC6;
const BIG5_HIBYTE_START1: u8 = 0xC9; // (level 2 hanzi)
const BIG5_HIBYTE_STOP1: u8 = 0xF9;
const BIG5_LOBYTE_LOBOUND0: u8 = 0x40;
const BIG5_LOBYTE_HIBOUND0: u8 = 0x7E;
const BIG5_LOBYTE_LOBOUND1: u8 = 0xA1;
const BIG5_LOBYTE_HIBOUND1: u8 = 0xFE;
const BIG5_CODES_PER_ROW: c_int = 160; // 3 more than the number of glyphs

extern "C" {
    fn Language_IsTaiwanese() -> bool;
}

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
fn Taiwanese_CollapseBig5Code(mut uiCode: u32) -> c_int {
    if Taiwanese_ValidBig5Code(uiCode) {
        uiCode -= (BIG5_HIBYTE_START0 as u32 * 256) + BIG5_LOBYTE_LOBOUND0 as u32; // sneaky maths on both bytes, reduce to 0x0000 onwards
        if (uiCode & 0xFF) >= ((BIG5_LOBYTE_LOBOUND1 as u32 - 1) - BIG5_LOBYTE_LOBOUND0 as u32) {
            uiCode -= (((BIG5_LOBYTE_LOBOUND1 as u32 - 1) - (BIG5_LOBYTE_HIBOUND0 as u32 + 1)) - 1);
        }
        uiCode = (((uiCode >> 8) as c_int * BIG5_CODES_PER_ROW) + (uiCode & 0xFF) as c_int) as u32;
        uiCode as c_int
    } else {
        0
    }
}

fn Taiwanese_InitFields(iGlyphTPs: &mut c_int, psLang: &mut &str) -> c_int {
    *psLang = "tai";
    *iGlyphTPs = GLYPH_MAX_TAIWANESE_SHADERS as c_int;
    unsafe {
        G_INON_SCALED_CHAR_RANGE = 255;
    }
    64 // m_iAsianGlyphsAcross
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
const SHIFTJIS_CODES_PER_ROW: c_int =
    (((SHIFTJIS_LOBYTE_STOP0 as c_int - SHIFTJIS_LOBYTE_START0 as c_int) + 1)
        + ((SHIFTJIS_LOBYTE_STOP1 as c_int - SHIFTJIS_LOBYTE_START1 as c_int) + 1));

extern "C" {
    fn Language_IsJapanese() -> bool;
}

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

fn Japanese_ValidShiftJISCode_u32(uiCode: u32) -> bool {
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
fn Japanese_CollapseShiftJISCode(mut uiCode: u32) -> c_int {
    if Japanese_ValidShiftJISCode_u32(uiCode) {
        uiCode -= ((SHIFTJIS_HIBYTE_START0 as u32) << 8) | SHIFTJIS_LOBYTE_START0 as u32; // sneaky maths on both bytes, reduce to 0x0000 onwards

        if (uiCode & 0xFF) >= ((SHIFTJIS_LOBYTE_START1 as u32) - SHIFTJIS_LOBYTE_START0 as u32) {
            uiCode -= (((SHIFTJIS_LOBYTE_START1 as u32) - SHIFTJIS_LOBYTE_STOP0 as u32) - 1);
        }

        if (((uiCode >> 8) & 0xFF) as u8) >= (SHIFTJIS_HIBYTE_START1 - SHIFTJIS_HIBYTE_START0) {
            uiCode -= (((SHIFTJIS_HIBYTE_START1 as u32 - SHIFTJIS_HIBYTE_STOP0 as u32) - 1) << 8);
        }

        uiCode = (((uiCode >> 8) as c_int * SHIFTJIS_CODES_PER_ROW) + (uiCode & 0xFF) as c_int) as u32;

        uiCode as c_int
    } else {
        0
    }
}

fn Japanese_InitFields(iGlyphTPs: &mut c_int, psLang: &mut &str) -> c_int {
    *psLang = "jap";
    *iGlyphTPs = GLYPH_MAX_JAPANESE_SHADERS as c_int;
    unsafe {
        G_INON_SCALED_CHAR_RANGE = 255;
    }
    64 // m_iAsianGlyphsAcross
}

// ======================== some Chinese stuff ==============================

const GB_HIBYTE_START: u8 = 0xA1; // range is...
const GB_HIBYTE_STOP: u8 = 0xF7; // ... inclusive
const GB_LOBYTE_LOBOUND: u8 = 0xA0; // range is...
const GB_LOBYTE_HIBOUND: u8 = 0xFF; // ...bounding (ie only valid in between these points, but NULLs in charsets for these codes)
const GB_CODES_PER_ROW: c_int = 95; // 1 more than the number of glyphs

extern "C" {
    fn Language_IsChinese() -> bool;
}

fn Chinese_ValidGBCode(_iHi: u8, _iLo: u8) -> bool {
    _iHi >= GB_HIBYTE_START && _iHi <= GB_HIBYTE_STOP && _iLo > GB_LOBYTE_LOBOUND && _iLo < GB_LOBYTE_HIBOUND
}

fn Chinese_ValidGBCode_u32(uiCode: u32) -> bool {
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
fn Chinese_CollapseGBCode(mut uiCode: u32) -> c_int {
    if Chinese_ValidGBCode_u32(uiCode) {
        uiCode -= (GB_HIBYTE_START as u32 * 256) + GB_LOBYTE_LOBOUND as u32; // sneaky maths on both bytes, reduce to 0x0000 onwards
        uiCode = (((uiCode >> 8) as c_int * GB_CODES_PER_ROW) + (uiCode & 0xFF) as c_int) as u32;
        uiCode as c_int
    } else {
        0
    }
}

fn Chinese_InitFields(iGlyphTPs: &mut c_int, psLang: &mut &str) -> c_int {
    *psLang = "chi";
    *iGlyphTPs = GLYPH_MAX_CHINESE_SHADERS as c_int;
    unsafe {
        G_INON_SCALED_CHAR_RANGE = 255;
    }
    64 // m_iAsianGlyphsAcross
}

// ======================== some Thai stuff ==============================

//TIS 620-2533

const TIS_GLYPHS_START: u8 = 160;
const TIS_SARA_AM: u32 = 0xD3; // special case letter, both a new letter and a trailing accent for the prev one
static mut G_THAI_CODES: ThaiCodes_t = ThaiCodes_t {
    m_mapValidCodes: HashMap::new(),
    m_viGlyphWidths: Vec::new(),
    m_strInitFailureReason: String::new(),
}; // the one and only instance of this object

extern "C" {
    fn Language_IsThai() -> bool;
}

/*
fn Thai_IsAccentChar(uiCode: u32) -> bool {
    match uiCode {
        209 | 212 | 213 | 214 | 215 | 216 | 217 | 218 | 231 | 232 | 233 | 234 | 235 | 236 | 237 | 238 => true,
        _ => false,
    }
}
*/

// returns a valid Thai code (or 0), based on taking 1,2 or 3 bytes from the supplied byte stream
// Fills in <iThaiBytes> with 1,2 or 3
fn Thai_ValidTISCode(psString: &[u8], iThaiBytes: &mut c_int) -> u32 {
    // try a 1-byte code first...
    //
    if psString[0] >= 160 {
        // so western letters drop through and use normal font
        // this code is heavily little-endian, so someone else will need to port for Mac etc... (not my problem ;-)

        let mut CodeToTry_sChars: [u8; 4] = [0; 4];
        let mut CodeToTry_uiCode: u32 = 0; // important that we clear all 4 bytes here

        // thai codes can be up to 3 bytes long, so see how high we can get...
        //
        let mut i = 0;
        while i < 3 {
            CodeToTry_sChars[i] = psString[i];

            // reconstruct u32 from bytes
            CodeToTry_uiCode = (CodeToTry_sChars[0] as u32)
                | ((CodeToTry_sChars[1] as u32) << 8)
                | ((CodeToTry_sChars[2] as u32) << 16)
                | ((CodeToTry_sChars[3] as u32) << 24);

            let iIndex = unsafe { G_THAI_CODES.GetValidIndex(CodeToTry_uiCode as c_int) };
            if iIndex == -1 {
                // failed, so return previous-longest code...
                //
                CodeToTry_sChars[i] = 0;
                break;
            }
            i += 1;
        }
        *iThaiBytes = i as c_int;
        debug_assert!(i != 0); // if 'i' was 0, then this may be an error, trying to get a thai accent as standalone char?
        CodeToTry_uiCode
    } else {
        0
    }
}

// special case, thai can only break on certain letters, and since the rules are complicated then
// we tell the translators to put an underscore ('_') between each word even though in Thai they're
// all jammed together at final output onscreen...
//
fn Thai_IsTrailingPunctuation(uiCode: u32) -> bool {
    uiCode == (b'_' as u32)
}

// takes a TIS 1,2 or 3 byte code and collapse down to a 0..n glyph index...
//
// (invalid codes will return 0)
//
fn Thai_CollapseTISCode(uiCode: u32) -> c_int {
    if uiCode >= TIS_GLYPHS_START as u32 {
        // so western letters drop through as invalid
        let iCollapsedIndex = unsafe { G_THAI_CODES.GetValidIndex(uiCode as c_int) };
        if iCollapsedIndex != -1 {
            return iCollapsedIndex;
        }
    }

    0
}

fn Thai_InitFields(iGlyphTPs: &mut c_int, psLang: &mut &str) -> c_int {
    *psLang = "tha";
    *iGlyphTPs = GLYPH_MAX_THAI_SHADERS as c_int;
    unsafe {
        G_INON_SCALED_CHAR_RANGE = i32::MAX;
    } // in other words, don't scale any thai chars down
    32 // m_iAsianGlyphsAcross
}

// ============================================================================

// takes char *, returns integer char at that point, and advances char * on by enough bytes to move
// past the letter (either western 1 byte or Asian multi-byte)...
//
// looks messy, but the actual execution route is quite short, so it's fast...
//
// Note that I have to have this 3-param form instead of advancing a passed-in "const char **psText" because of VM-crap where you can only change ptr-contents, not ptrs themselves. Bleurgh. Ditto the qtrue:qfalse crap instead of just returning stuff straight through.
//
fn AnyLanguage_ReadCharFromString(
    psText: &[u8],
    piAdvanceCount: &mut c_int,
    pbIsTrailingPunctuation: Option<&mut bool>,
) -> u32 {
    let psString = psText; // already &[u8]
    let mut uiLetter: u32;

    match GetLanguageEnum() {
        Language_e::eKorean => {
            if Korean_ValidKSC5601Hangul(psString[0], psString[1]) {
                uiLetter = ((psString[0] as u32) * 256) + psString[1] as u32;
                *piAdvanceCount = 2;

                // not going to bother testing for korean punctuation here, since korean already
                // uses spaces, and I don't have the punctuation glyphs defined, only the basic 2350 hanguls
                //
                if let Some(pbIsTrail) = pbIsTrailingPunctuation {
                    *pbIsTrail = false;
                }

                return uiLetter;
            }
        }

        Language_e::eTaiwanese => {
            if Taiwanese_ValidBig5Code(((psString[0] as u32) * 256) + psString[1] as u32) {
                uiLetter = ((psString[0] as u32) * 256) + psString[1] as u32;
                *piAdvanceCount = 2;

                // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                //
                if let Some(pbIsTrail) = pbIsTrailingPunctuation {
                    *pbIsTrail = Taiwanese_IsTrailingPunctuation(uiLetter);
                }

                return uiLetter;
            }
        }

        Language_e::eJapanese => {
            if Japanese_ValidShiftJISCode(psString[0], psString[1]) {
                uiLetter = ((psString[0] as u32) * 256) + psString[1] as u32;
                *piAdvanceCount = 2;

                // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                //
                if let Some(pbIsTrail) = pbIsTrailingPunctuation {
                    *pbIsTrail = Japanese_IsTrailingPunctuation(uiLetter);
                }

                return uiLetter;
            }
        }

        Language_e::eChinese => {
            if Chinese_ValidGBCode(psString[0], psString[1]) {
                uiLetter = ((psString[0] as u32) * 256) + psString[1] as u32;
                *piAdvanceCount = 2;

                // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                //
                if let Some(pbIsTrail) = pbIsTrailingPunctuation {
                    *pbIsTrail = Chinese_IsTrailingPunctuation(uiLetter);
                }

                return uiLetter;
            }
        }

        Language_e::eThai => {
            let mut iThaiBytes = 0;
            uiLetter = Thai_ValidTISCode(psString, &mut iThaiBytes);
            if uiLetter != 0 {
                *piAdvanceCount = iThaiBytes;

                if let Some(pbIsTrail) = pbIsTrailingPunctuation {
                    *pbIsTrail = Thai_IsTrailingPunctuation(uiLetter);
                }

                return uiLetter;
            }
        }

        _ => {}
    }

    // ... must not have been an MBCS code...
    //
    uiLetter = psString[0] as u32;
    *piAdvanceCount = 1;

    if let Some(pbIsTrail) = pbIsTrailingPunctuation {
        *pbIsTrail = uiLetter == (b'!' as u32)
            || uiLetter == (b'?' as u32)
            || uiLetter == (b',' as u32)
            || uiLetter == (b'.' as u32)
            || uiLetter == (b';' as u32)
            || uiLetter == (b':' as u32);
    }

    uiLetter
}

// needed for subtitle printing since original code no longer worked once camera bar height was changed to 480/10
// rather than refdef height / 10. I now need to bodge the coords to come out right.
//
fn Language_IsAsian() -> bool {
    match GetLanguageEnum() {
        Language_e::eKorean | Language_e::eTaiwanese | Language_e::eJapanese | Language_e::eChinese | Language_e::eThai => {
            // this is asian, but the query is normally used for scaling
            true
        }
        _ => false,
    }
}

fn Language_UsesSpaces() -> bool {
    // ( korean uses spaces )
    match GetLanguageEnum() {
        Language_e::eTaiwanese | Language_e::eJapanese | Language_e::eChinese | Language_e::eThai => false,
        _ => true,
    }
}

// ======================================================================
// name is (eg) "ergo" or "lcd", no extension.
//
// If path present, it's a special language hack for SBCS override languages, eg: "lcd/russian", which means
// just treat the file as "russian", but with the "lcd" part ensuring we don't find a different registered russian font
//

fn GetFont_Actual(index: c_int) -> Option<Box<CFontInfo>> {
    let index = index & SET_MASK;
    if index >= 1 && index < unsafe { G_ICURRENT_FONT_INDEX } {
        if let Some(Some(ref mut pFont)) = unsafe { G_VFONT_ARRAY.get_mut(index as usize) } {
            // pFont.UpdateAsianIfNeeded(); // would update in place
            return None; // simplified stub
        }
        unsafe { G_VFONT_ARRAY[index as usize].clone() }
    } else {
        None
    }
}

pub fn RE_Font_StrLenPixels(psText: &str, iFontHandle: c_int, fScale: f32) -> c_int {
    let mut iMaxWidth = 0;
    let mut iThisWidth = 0;

    let curfont_opt = GetFont_Actual(iFontHandle);
    if curfont_opt.is_none() {
        return 0;
    }
    let curfont = curfont_opt.unwrap();

    let mut fScaleA = fScale;
    if Language_IsAsian() && fScale > 0.7 {
        fScaleA = fScale * 0.75;
    }

    let psText_bytes = psText.as_bytes();
    let mut psText_idx = 0;

    while psText_idx < psText_bytes.len() {
        let mut iAdvanceCount = 0;
        let uiLetter = AnyLanguage_ReadCharFromString(&psText_bytes[psText_idx..], &mut iAdvanceCount, None);
        psText_idx += iAdvanceCount as usize;

        if uiLetter == (b'^' as u32) {
            if psText_idx < psText_bytes.len() && psText_bytes[psText_idx] >= b'0' && psText_bytes[psText_idx] <= b'9' {
                let mut iAdvanceCount2 = 0;
                AnyLanguage_ReadCharFromString(&psText_bytes[psText_idx..], &mut iAdvanceCount2, None);
                psText_idx += iAdvanceCount2 as usize;
                continue;
            }
        }

        if uiLetter == 0x0A {
            iThisWidth = 0;
        } else {
            let iPixelAdvance = curfont.GetLetterHorizAdvance(uiLetter);

            let fValue = iPixelAdvance as f32 * if uiLetter > unsafe { G_INON_SCALED_CHAR_RANGE as u32 } { fScaleA } else { fScale };
            iThisWidth += if curfont.mbRoundCalcs {
                Round(fValue)
            } else {
                fValue as c_int
            };
            if iThisWidth > iMaxWidth {
                iMaxWidth = iThisWidth;
            }
        }
    }

    iMaxWidth
}

// not really a font function, but keeps naming consistant...
//
pub fn RE_Font_StrLenChars(psText: &str) -> c_int {
    // logic for this function's letter counting must be kept same in this function and RE_Font_DrawString()
    //
    let mut iCharCount = 0;
    let psText_bytes = psText.as_bytes();
    let mut psText_idx = 0;

    while psText_idx < psText_bytes.len() {
        // in other words, colour codes and CR/LF don't count as chars, all else does...
        //
        let mut iAdvanceCount = 0;
        let uiLetter = AnyLanguage_ReadCharFromString(&psText_bytes[psText_idx..], &mut iAdvanceCount, None);
        psText_idx += iAdvanceCount as usize;

        match uiLetter {
            b'^' as u32 => {
                if psText_idx < psText_bytes.len() && psText_bytes[psText_idx] >= b'0' && psText_bytes[psText_idx] <= b'9' {
                    psText_idx += 1;
                } else {
                    iCharCount += 1;
                }
            }
            10 => {}, // linefeed
            13 => {}, // return
            b'_' as u32 => {
                // special word-break hack
                if GetLanguageEnum() == Language_e::eThai && (psText_idx < psText_bytes.len() && psText_bytes[psText_idx] >= TIS_GLYPHS_START) {
                    // skip
                } else {
                    iCharCount += 1;
                }
            }
            _ => {
                iCharCount += 1;
            }
        }
    }

    iCharCount
}

pub fn RE_Font_HeightPixels(iFontHandle: c_int, fScale: f32) -> c_int {
    let curfont_opt = GetFont_Actual(iFontHandle);
    if let Some(curfont) = curfont_opt {
        let fValue = curfont.GetPointSize() as f32 * fScale;
        return if curfont.mbRoundCalcs {
            Round(fValue)
        } else {
            fValue as c_int
        };
    }
    0
}

// iMaxPixelWidth is -1 for "all of string", else pixel display count...
//
pub fn RE_Font_DrawString(
    _ox: c_int,
    _oy: c_int,
    psText: &str,
    _rgba: &[f32; 4],
    iFontHandle: c_int,
    _iMaxPixelWidth: c_int,
    fScale: f32,
) {
    // Stub: Core implementation would follow closely from the C++ version
    // This is a complex function with deep integration into the rendering system
    let _curfont_opt = GetFont_Actual(iFontHandle);
    // ... full implementation omitted for brevity
}

pub fn RE_RegisterFont(psName: &str) -> c_int {
    unsafe {
        if let Some(&iFontIndex) = G_MAP_FONT_INDEXES.get(psName) {
            return iFontIndex;
        }

        // not registered, so...
        //
        let pFont = CFontInfo::new(psName);
        if pFont.GetPointSize() > 0 {
            let iFontIndex = G_ICURRENT_FONT_INDEX - 1;
            G_MAP_FONT_INDEXES.insert(psName.to_string(), iFontIndex);
            // pFont.m_iThisFont = iFontIndex;
            return iFontIndex;
        } else {
            G_MAP_FONT_INDEXES.insert(psName.to_string(), 0); // missing/invalid
        }

        0
    }
}

pub fn R_InitFonts() {
    unsafe {
        G_ICURRENT_FONT_INDEX = 1; // entry 0 is reserved for "missing/invalid"
        G_INON_SCALED_CHAR_RANGE = i32::MAX; // default all chars to have no special scaling (other than user supplied)
    }
}

pub fn R_ShutdownFonts() {
    unsafe {
        for i in 1..G_ICURRENT_FONT_INDEX {
            // delete g_vFontArray[i];  (automatic via Rust drop)
        }
        G_MAP_FONT_INDEXES.clear();
        G_VFONT_ARRAY.clear();
        G_ICURRENT_FONT_INDEX = 1; // entry 0 is reserved for "missing/invalid"

        G_THAI_CODES.Clear();
    }
}

// this is only really for debugging while tinkering with fonts, but harmless to leave in...
//
pub fn R_ReloadFonts_f() {
    // first, grab all the currently-registered fonts IN THE ORDER THEY WERE REGISTERED...
    //
    let mut vstrFonts: Vec<String> = Vec::new();

    unsafe {
        for iFontToFind in 1..G_ICURRENT_FONT_INDEX {
            let mut found = false;
            for (key, &value) in &G_MAP_FONT_INDEXES {
                if iFontToFind == value {
                    vstrFonts.push(key.clone());
                    found = true;
                    break;
                }
            }
            if !found {
                break; // couldn't find this font
            }
        }

        if vstrFonts.len() == (G_ICURRENT_FONT_INDEX - 1) as usize {
            // found all of them?
            // now restart the font system...
            //
            R_ShutdownFonts();
            R_InitFonts();
            //
            // and re-register our fonts in the same order as before (note that some menu items etc cache the string lengths so really a vid_restart is better, but this is just for my testing)
            //
            for iFont in 0..vstrFonts.len() {
                let iNewFontHandle = RE_RegisterFont(&vstrFonts[iFont]);
                #[cfg(debug_assertions)]
                {
                    debug_assert_eq!(iNewFontHandle, (iFont as c_int) + 1);
                }
            }
            Com_Printf("Done.\n");
        } else {
            Com_Printf("Problem encountered finding current fonts, ignoring.\n"); // poo. Oh well, forget it.
        }
    }
}

// ============================================================================
// External stubs for engine functions we depend on
// ============================================================================

extern "C" {
    fn Language_IsRussian() -> bool;
    fn Language_IsPolish() -> bool;

    fn FS_ReadFile(name: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buf: *mut c_void);
    fn FS_FOpenFileRead(filename: *const c_char, f: *mut *mut c_void, flush: bool) -> c_int;
    fn FS_FCloseFile(f: *mut c_void);

    fn Com_Printf(msg: *const c_char, ...);
    fn Com_Error(level: c_int, msg: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: usize, msg: *const c_char, ...);

    fn Cvar_Set(name: *const c_char, value: *const c_char);
    fn SE_LANGUAGE_MOD_COUNT() -> c_int;

    fn RE_RegisterShaderNoMip(name: &str) -> c_int;
    fn RE_SetColor(rgba: &[f32; 4]);
    fn RE_StretchPic(x: f32, y: f32, w: f32, h: f32, s1: f32, t1: f32, s2: f32, t2: f32, hShader: c_int);

    fn COM_SkipPath(path: &str) -> &str;
    fn COM_StripExtension(path: &mut [c_char]);

    fn Sys_Milliseconds() -> c_int;
    fn ColorIndex(c: u8) -> c_int;

    fn Round(x: f32) -> c_int;

    static mut g_color_table: [[f32; 4]; 16];
}

// Stub for reading files in Rust-friendly way
fn FS_ReadFile_alloc(filename: &str) -> Option<Vec<c_int>> {
    // This would be implemented to call FS_ReadFile and convert to Vec
    // For now, return None as stub
    None
}

fn Com_Printf(msg: &str) {
    // Stub implementation
}

// end
