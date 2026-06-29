// Filename: cg_credits.rs
//
// module for end credits code

// this line must stay at top so the whole PCH thing works...
//
// #include "cg_headers.h"

// #include "cg_local.h"
// #include "cg_media.h"

use std::collections::VecDeque;
use std::cell::Cell;

fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

const CARD_FADESECONDS: f32 = 1.0f32;  // fade up time, also fade down time
const CARD_SUSTAINSECONDS: f32 = 2.0f32;  // hold time before fade down
const LINE_SECONDTOSCROLLUP: f32 = 15.0f32;  // how long one line takes to scroll up the screen

const MAX_LINE_BYTES: usize = 2048;

pub static mut ghFontHandle: u32 = 0;
pub static mut gfFontScale: f32 = 1.0f32;
pub static mut gv4Color: [f32; 4] = [0.0; 4];

#[derive(Clone)]
pub struct StringAndSize_t {
    pub iStrLenPixels: Cell<i32>,
    pub str: String,
}

impl StringAndSize_t {
    pub fn new() -> Self {
        StringAndSize_t {
            iStrLenPixels: Cell::new(-1),
            str: String::new(),
        }
    }

    pub fn from_c_str(ps_string: &str) -> Self {
        StringAndSize_t {
            iStrLenPixels: Cell::new(-1),
            str: ps_string.to_string(),
        }
    }

    pub fn c_str(&self) -> &str {
        &self.str
    }

    pub fn GetPixelLength(&self) -> i32 {
        if self.iStrLenPixels.get() == -1 {
            unsafe {
                let len = cgi_R_Font_StrLenPixels(self.str.as_ptr(), ghFontHandle, gfFontScale);
                self.iStrLenPixels.set(len);
            }
        }

        self.iStrLenPixels.get()
    }

    pub fn IsEmpty(&self) -> bool {
        self.str.is_empty()
    }
}

#[derive(Clone)]
pub struct CreditCard_t {
    pub iTime: i32,
    pub strTitle: StringAndSize_t,
    pub vstrText: Vec<StringAndSize_t>,
}

impl CreditCard_t {
    pub fn new() -> Self {
        CreditCard_t {
            iTime: -1,  // flag "not set yet"
            strTitle: StringAndSize_t::new(),
            vstrText: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct CreditLine_t {
    pub iLine: i32,
    pub strText: StringAndSize_t,
    pub vstrText: Vec<StringAndSize_t>,
    pub bDotted: bool,
}

impl CreditLine_t {
    pub fn new() -> Self {
        CreditLine_t {
            iLine: 0,
            strText: StringAndSize_t::new(),
            vstrText: Vec::new(),
            bDotted: false,
        }
    }
}

type CreditLines_t = VecDeque<CreditLine_t>;
type CreditCards_t = VecDeque<CreditCard_t>;

pub struct CreditData_t {
    pub iStartTime: i32,

    pub CreditCards: CreditCards_t,
    pub CreditLines: CreditLines_t,
}

impl CreditData_t {
    pub fn new() -> Self {
        CreditData_t {
            iStartTime: 0,
            CreditCards: VecDeque::new(),
            CreditLines: VecDeque::new(),
        }
    }

    pub fn Running(&self) -> bool {
        !self.CreditCards.is_empty() || !self.CreditLines.is_empty()
    }
}

pub static mut CreditData: CreditData_t = CreditData_t {
    iStartTime: 0,
    CreditCards: VecDeque::new(),
    CreditLines: VecDeque::new(),
};

// External C functions and game engine interface
extern "C" {
    pub fn Q_strncpyz(dest: *mut u8, src: *const u8, size: usize);
    pub fn strupr(str: *mut i8) -> *mut i8;
    pub fn strlwr(str: *mut i8) -> *mut i8;
    pub fn isspace(c: core::ffi::c_int) -> core::ffi::c_int;
    pub fn isalpha(c: core::ffi::c_int) -> core::ffi::c_int;
    pub fn toupper(c: core::ffi::c_int) -> core::ffi::c_int;
    pub fn strlen(s: *const u8) -> usize;
    pub fn strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    pub fn strchr(s: *const u8, c: core::ffi::c_int) -> *const u8;
    pub fn stricmp(s1: *const u8, s2: *const u8) -> core::ffi::c_int;
    pub fn strstr(haystack: *const u8, needle: *const u8) -> *const u8;
    pub fn cgi_R_Font_StrLenPixels(text: *const u8, handle: u32, scale: f32) -> i32;
    pub fn cgi_S_StartBackgroundTrack(intro: *const u8, loop_track: *const u8, bLoop: bool);
    pub fn cgi_SP_GetStringTextString(text_key: *const u8, buffer: *mut u8, buffer_size: i32) -> i32;
    pub fn cgi_Z_Malloc(size: usize, tag: i32) -> *mut u8;
    pub fn cgi_Z_Free(ptr: *mut u8);
    pub fn cgi_AnyLanguage_ReadCharFromString(
        ps_text: *const u8,
        advance_count: *mut i32,
        is_trailing_punctuation: *mut u8,
    ) -> u32;
    pub fn cgi_R_Font_HeightPixels(handle: u32, scale: f32) -> f32;
    pub fn cgi_R_Font_DrawString(
        x: i32,
        y: i32,
        text: *const u8,
        color: *const [f32; 4],
        handle: u32,
        limit: i32,
        scale: f32,
    );
    pub fn Q_strcat(dest: *mut u8, size: usize, src: *const u8);
    pub fn va(fmt: *const u8, ...) -> *const u8;
    pub fn Com_Printf(fmt: *const u8, ...);

    pub static mut cgs: cgs_t;
    pub static mut cg: cg_t;
    pub static mut g_entities: [gentity_t; 1024];
}

// Stub external types for references
#[repr(C)]
pub struct cgs_t {
    pub media: media_t,
}

#[repr(C)]
pub struct media_t {
    pub qhFontMedium: u32,
}

#[repr(C)]
pub struct cg_t {
    pub time: i32,
}

#[repr(C)]
pub struct gentity_t {
    pub client: *mut gclient_t,
}

#[repr(C)]
pub struct gclient_t {
    pub sess: clientSession_t,
}

#[repr(C)]
pub struct clientSession_t {
    pub mission_objectives: [mission_objective_t; 1],
}

#[repr(C)]
pub struct mission_objective_t {
    pub status: i32,
}

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;
const TAG_TEMP_WORKSPACE: i32 = 0;

// cope with hyphenated names and initials (awkward gits)...
//
unsafe fn CountsAsWhiteSpaceForCaps(c: u8) -> bool {
    isspace(c as core::ffi::c_int) != 0
        || c == b'-'
        || c == b'.'
        || c == b'('
        || c == b')'
        || c == b'\''
}

thread_local! {
    static Capitalize_sTemp: std::cell::RefCell<[u8; MAX_LINE_BYTES]> = std::cell::RefCell::new([0; MAX_LINE_BYTES]);
}

unsafe fn Capitalize(ps_test: *const u8) -> *const u8 {
    Capitalize_sTemp.with(|sTemp_cell| {
        let mut sTemp = sTemp_cell.borrow_mut();

        Q_strncpyz(sTemp.as_mut_ptr(), ps_test, sTemp.len());

        // if (!cgi_Language_IsAsian())	// we don't have asian credits, so this is ok to do now
        {
            strupr(sTemp.as_mut_ptr() as *mut i8);  // capitalise titles (if not asian!!!!)
        }

        sTemp.as_ptr()
    })
}

thread_local! {
    static UpperCaseFirstLettersOnly_sTemp: std::cell::RefCell<[u8; MAX_LINE_BYTES]> = std::cell::RefCell::new([0; MAX_LINE_BYTES]);
}

unsafe fn UpperCaseFirstLettersOnly(ps_test: *const u8) -> *const u8 {
    UpperCaseFirstLettersOnly_sTemp.with(|sTemp_cell| {
        let mut sTemp = sTemp_cell.borrow_mut();

        Q_strncpyz(sTemp.as_mut_ptr(), ps_test, sTemp.len());

        // if (!cgi_Language_IsAsian())	// we don't have asian credits, so this is ok to do now
        {
            strlwr(sTemp.as_mut_ptr() as *mut i8);

            let mut p = sTemp.as_mut_ptr();
            while *p != 0 {
                while *p != 0 && CountsAsWhiteSpaceForCaps(*p) {
                    p = p.add(1);
                }
                if *p != 0 {
                    *p = toupper(*p as core::ffi::c_int) as u8;
                    while *p != 0 && !CountsAsWhiteSpaceForCaps(*p) {
                        p = p.add(1);
                    }
                }
            }
        }

        // now restore any weird stuff...
        //
        let mut p = strstr(
            sTemp.as_ptr(),
            b" Mc\0".as_ptr(),
        );  // eg "Mcfarrell" should be "McFarrell"
        if !p.is_null() && isalpha(*(p.add(3)) as core::ffi::c_int) != 0 {
            *(p.add(3) as *mut u8) = toupper(*(p.add(3)) as core::ffi::c_int) as u8;
        }
        p = strstr(sTemp.as_ptr(), b" O'\0".as_ptr());  // eg "O'flaherty" should be "O'Flaherty" (this is probably done automatically now, but wtf.
        if !p.is_null() && isalpha(*(p.add(3)) as core::ffi::c_int) != 0 {
            *(p.add(3) as *mut u8) = toupper(*(p.add(3)) as core::ffi::c_int) as u8;
        }
        p = strstr(sTemp.as_ptr(), b"Lucasarts\0".as_ptr());
        if !p.is_null() {
            *(p.add(5) as *mut u8) = b'A';  // capitalise the 'A' in LucasArts (jeez...)
        }

        sTemp.as_ptr()
    })
}

thread_local! {
    static GetSubString_sTemp: std::cell::RefCell<[u8; MAX_LINE_BYTES]> = std::cell::RefCell::new([0; MAX_LINE_BYTES]);
}

unsafe fn GetSubString(str_result: &mut String) -> *const u8 {
    GetSubString_sTemp.with(|sTemp_cell| {
        let mut sTemp = sTemp_cell.borrow_mut();

        if str_result.is_empty() {
            return std::ptr::null();
        }

        strncpy(
            sTemp.as_mut_ptr(),
            str_result.as_ptr() as *const u8,
            sTemp.len() - 1,
        );
        sTemp[sTemp.len() - 1] = b'\0';

        let ps_semi_colon = strchr(sTemp.as_ptr(), b';' as core::ffi::c_int);
        if !ps_semi_colon.is_null() {
            *(ps_semi_colon as *mut u8) = b'\0';

            let offset = (ps_semi_colon as usize - sTemp.as_ptr() as usize) + 1;
            if offset <= str_result.len() {
                str_result.drain(0..offset);
            } else {
                str_result.clear();
            }
        } else {
            // no semicolon found, probably last entry? (though i think even those have them on, oh well)
            //
            str_result.clear();
        }

        sTemp.as_ptr()
    })
}

// sort entries by their last name (starts at back of string and moves forward until start or just before whitespace)
// ...
unsafe extern "C" fn SortBySurname(elem1: *const core::ffi::c_void, elem2: *const core::ffi::c_void) -> core::ffi::c_int {
    let p1 = elem1 as *const StringAndSize_t;
    let p2 = elem2 as *const StringAndSize_t;

    let s1 = (*p1).c_str();
    let s2 = (*p2).c_str();

    let mut ps_surname1 = s1.as_ptr().add(s1.len() - 1) as *const u8;
    let mut ps_surname2 = s2.as_ptr().add(s2.len() - 1) as *const u8;

    while ps_surname1 > s1.as_ptr() as *const u8 && isspace(*ps_surname1 as core::ffi::c_int) == 0 {
        ps_surname1 = ps_surname1.offset(-1);
    }
    while ps_surname2 > s2.as_ptr() as *const u8 && isspace(*ps_surname2 as core::ffi::c_int) == 0 {
        ps_surname2 = ps_surname2.offset(-1);
    }
    if isspace(*ps_surname1 as core::ffi::c_int) != 0 {
        ps_surname1 = ps_surname1.add(1);
    }
    if isspace(*ps_surname2 as core::ffi::c_int) != 0 {
        ps_surname2 = ps_surname2.add(1);
    }

    stricmp(ps_surname1, ps_surname2)
}

pub unsafe fn CG_Credits_Init(ps_strip_reference: *const u8, pv4_color: *const [f32; 4]) {
    // Play the light side end credits music.
    if (*g_entities[0].client).sess.mission_objectives[0].status != 2 {
        cgi_S_StartBackgroundTrack(b"music/endcredits.mp3\0".as_ptr(), std::ptr::null(), false);
    }
    // Play the dark side end credits music.
    else {
        cgi_S_StartBackgroundTrack(
            b"music/vjun3/vjun3_explore.mp3\0".as_ptr(),
            std::ptr::null(),
            false,
        );
    }

    // could make these into parameters later, but for now...
    //
    ghFontHandle = cgs.media.qhFontMedium;
    gfFontScale = 1.0f32;

    // memcpy so we can poke into alpha channel
    gv4Color = *pv4_color;

    // first, ask the strlen of the final string...
    //
    let i_str_len = cgi_SP_GetStringTextString(ps_strip_reference, std::ptr::null_mut(), 0);
    if i_str_len == 0 {
        #[cfg(not(feature = "FINAL_BUILD"))]
        Com_Printf(
            b"WARNING: CG_Credits_Init(): invalid text key :'%s'\n\0".as_ptr(),
            ps_strip_reference,
        );
        return;
    }
    //
    // malloc space to hold it...
    //
    let ps_malloc_text = cgi_Z_Malloc((i_str_len + 1) as usize, TAG_TEMP_WORKSPACE);
    //
    // now get the string...
    //
    let i_str_len = cgi_SP_GetStringTextString(ps_strip_reference, ps_malloc_text, i_str_len + 1);
    //ensure we found a match
    if i_str_len == 0 {
        assert!(false);  // should never get here now, but wtf?
        cgi_Z_Free(ps_malloc_text);
        #[cfg(not(feature = "FINAL_BUILD"))]
        Com_Printf(
            b"WARNING: CG_Credits_Init(): invalid text key :'%s'\n\0".as_ptr(),
            ps_strip_reference,
        );
        return;
    }

    // Clear credit data for fresh init
    let mut credit_data = &mut CreditData;
    credit_data.CreditCards.clear();
    credit_data.CreditLines.clear();

    // read whole string in and process as cards, lines etc...
    //
    #[derive(PartialEq, Eq, Copy, Clone)]
    enum Mode_e {
        eNothing = 0,
        eLine,
        eDotEntry,
        eTitle,
        eCard,
        eFinished,
    }
    let mut e_mode = Mode_e::eNothing;

    let mut b_cards_finished = false;
    let mut i_line_number = 0;
    let mut ps_text_parse = ps_malloc_text;

    while *ps_text_parse != 0 {
        // read a line...
        //
        let mut s_line: [u8; MAX_LINE_BYTES] = [0; MAX_LINE_BYTES];
        s_line[0] = 0;
        let mut b_was_command = true;
        loop {
            let mut b_is_trailing_punctuation: u8 = 0;
            let mut i_advance_count: i32 = 0;
            let ui_letter = cgi_AnyLanguage_ReadCharFromString(
                ps_text_parse,
                &mut i_advance_count,
                &mut b_is_trailing_punctuation,
            );
            ps_text_parse = ps_text_parse.add(i_advance_count as usize);

            // concat onto string so far...
            //
            if ui_letter == 32 && s_line[0] == 0 {
                continue;  // unless it's a space at the start of a line, in which case ignore it.
            }

            if ui_letter == b'\n' as u32 || ui_letter == 0 {
                // have we got a command word?...
                //
                let s_line_cstr = std::ffi::CStr::from_ptr(s_line.as_ptr() as *const i8)
                    .to_string_lossy();
                if s_line_cstr.len() >= 2 && s_line_cstr.starts_with("(#") {
                    // yep...
                    //
                    if s_line_cstr == "(#CARD)" {
                        if !b_cards_finished {
                            e_mode = Mode_e::eCard;
                        } else {
                            #[cfg(not(feature = "FINAL_BUILD"))]
                            Com_Printf(
                                b"\x1b[33mCG_Credits_Init(): No current support for cards after scroll!\n\0".as_ptr(),
                            );
                            e_mode = Mode_e::eNothing;
                        }
                        break;
                    } else if s_line_cstr == "(#TITLE)" {
                        e_mode = Mode_e::eTitle;
                        b_cards_finished = true;
                        break;
                    } else if s_line_cstr == "(#LINE)" {
                        e_mode = Mode_e::eLine;
                        b_cards_finished = true;
                        break;
                    } else if s_line_cstr == "(#DOTENTRY)" {
                        e_mode = Mode_e::eDotEntry;
                        b_cards_finished = true;
                        break;
                    } else {
                        #[cfg(not(feature = "FINAL_BUILD"))]
                        Com_Printf(
                            b"\x1b[33mCG_Credits_Init(): bad keyword \"%s\"!\n\0".as_ptr(),
                            s_line.as_ptr() as *const i8,
                        );
                        e_mode = Mode_e::eNothing;
                    }
                } else {
                    // I guess not...
                    //
                    b_was_command = false;
                    break;
                }
            } else {
                // must be a letter...
                //
                if ui_letter > 255 {
                    assert!(false);  // this means we're attempting to display asian credits, and we don't
                                     //	support these now because the auto-capitalisation rules etc would have to
                                     //	be inhibited.
                    let c1 = (ui_letter >> 8) as u8 as i8;
                    let c2 = (ui_letter & 0xFF) as u8 as i8;
                    Q_strcat(
                        s_line.as_mut_ptr(),
                        s_line.len(),
                        va(b"%c%c\0".as_ptr(), c1, c2),
                    );
                } else {
                    let ch = (ui_letter & 0xFF) as u8 as i8;
                    Q_strcat(s_line.as_mut_ptr(), s_line.len(), va(b"%c\0".as_ptr(), ch));
                }
            }
        }

        // command?...
        //
        if b_was_command {
            // this'll just be a mode change, so ignore...
            //
        } else {
            // else we've got some text to display...
            //
            let s_line_str = std::ffi::CStr::from_ptr(s_line.as_ptr() as *const i8)
                .to_string_lossy()
                .to_string();
            match e_mode {
                Mode_e::eNothing => {}
                Mode_e::eLine => {
                    let mut credit_line = CreditLine_t::new();
                    credit_line.iLine = i_line_number;
                    i_line_number += 1;
                    credit_line.strText = StringAndSize_t::from_c_str(&s_line_str);

                    credit_data.CreditLines.push_back(credit_line);
                }

                Mode_e::eDotEntry => {
                    let mut credit_line = CreditLine_t::new();
                    credit_line.iLine = i_line_number;
                    credit_line.bDotted = true;

                    let mut str_result = s_line_str.clone();
                    let mut p;
                    loop {
                        p = GetSubString(&mut str_result);
                        if p.is_null() {
                            break;
                        }
                        let p_str = std::ffi::CStr::from_ptr(p as *const i8)
                            .to_string_lossy()
                            .to_string();
                        if credit_line.strText.IsEmpty() {
                            credit_line.strText = StringAndSize_t::from_c_str(&p_str);
                        } else {
                            let upper_p = UpperCaseFirstLettersOnly(p);
                            let upper_p_str = std::ffi::CStr::from_ptr(upper_p as *const i8)
                                .to_string_lossy()
                                .to_string();
                            credit_line.vstrText.push(StringAndSize_t::from_c_str(&upper_p_str));
                        }
                    }

                    if !credit_line.strText.IsEmpty() && !credit_line.vstrText.is_empty() {
                        // sort entries RHS dotted entries by alpha...
                        //
                        credit_line
                            .vstrText
                            .sort_by(|a, b| {
                                let cmp = stricmp(a.c_str().as_ptr() as *const u8, b.c_str().as_ptr() as *const u8);
                                if cmp < 0 {
                                    std::cmp::Ordering::Less
                                } else if cmp > 0 {
                                    std::cmp::Ordering::Greater
                                } else {
                                    std::cmp::Ordering::Equal
                                }
                            });

                        credit_data.CreditLines.push_back(credit_line.clone());
                        i_line_number += credit_line.vstrText.len() as i32;
                    }
                }

                Mode_e::eTitle => {
                    i_line_number += 1;  // leading blank line

                    let mut credit_line = CreditLine_t::new();
                    credit_line.iLine = i_line_number;
                    i_line_number += 1;
                    let cap_text = Capitalize(s_line_str.as_ptr() as *const u8);
                    let cap_str = std::ffi::CStr::from_ptr(cap_text as *const i8)
                        .to_string_lossy()
                        .to_string();
                    credit_line.strText = StringAndSize_t::from_c_str(&cap_str);

                    credit_data.CreditLines.push_back(credit_line);

                    i_line_number += 1;  // trailing blank line
                }
                Mode_e::eCard => {
                    let mut credit_card = CreditCard_t::new();

                    let mut str_result = s_line_str.clone();
                    let mut p;
                    loop {
                        p = GetSubString(&mut str_result);
                        if p.is_null() {
                            break;
                        }
                        let p_str = std::ffi::CStr::from_ptr(p as *const i8)
                            .to_string_lossy()
                            .to_string();
                        if credit_card.strTitle.IsEmpty() {
                            let cap_p = Capitalize(p);
                            let cap_str = std::ffi::CStr::from_ptr(cap_p as *const i8)
                                .to_string_lossy()
                                .to_string();
                            credit_card.strTitle = StringAndSize_t::from_c_str(&cap_str);
                        } else {
                            let upper_p = UpperCaseFirstLettersOnly(p);
                            let upper_p_str = std::ffi::CStr::from_ptr(upper_p as *const i8)
                                .to_string_lossy()
                                .to_string();
                            credit_card.vstrText.push(StringAndSize_t::from_c_str(&upper_p_str));
                        }
                    }

                    if !credit_card.strTitle.IsEmpty() {
                        // sort entries by alpha...
                        //
                        credit_card
                            .vstrText
                            .sort_by(|a, b| {
                                let cmp = stricmp(a.c_str().as_ptr() as *const u8, b.c_str().as_ptr() as *const u8);
                                if cmp < 0 {
                                    std::cmp::Ordering::Less
                                } else if cmp > 0 {
                                    std::cmp::Ordering::Greater
                                } else {
                                    std::cmp::Ordering::Equal
                                }
                            });

                        credit_data.CreditCards.push_back(credit_card);
                    }
                }
                _ => {}
            }
        }
    }

    cgi_Z_Free(ps_malloc_text);
    credit_data.iStartTime = cg.time;
}

pub fn CG_Credits_Running() -> bool {
    unsafe { CreditData.Running() }
}

// returns qtrue if still drawing...
//
pub unsafe fn CG_Credits_Draw() -> bool {
    if CG_Credits_Running() {
        let i_font_height = (1.5f32 * cgi_R_Font_HeightPixels(ghFontHandle, gfFontScale)) as i32;  // taiwanese & japanese need 1.5 fontheight spacing

        //		cgi_R_SetColor( *gpv4Color );

        // display cards first...
        //
        if !CreditData.CreditCards.is_empty() {
            // grab first card off the list (we know there's at least one here, so...)
            //
            let credit_card = &mut CreditData.CreditCards.front_mut().unwrap();

            if credit_card.iTime == -1 {
                // onceonly time init...
                //
                credit_card.iTime = cg.time;
            }

            // play with the alpha channel for fade up/down...
            //
            let f_milli_seconds_elapsed = (cg.time - credit_card.iTime) as f32;
            let f_seconds_elapsed = f_milli_seconds_elapsed / 1000.0f32;
            if f_seconds_elapsed < CARD_FADESECONDS {
                // fading up...
                //
                gv4Color[3] = f_seconds_elapsed / CARD_FADESECONDS;
                //				OutputDebugString(va("fade up: %f\n",gv4Color[3]));
            } else if f_seconds_elapsed > CARD_FADESECONDS + CARD_SUSTAINSECONDS {
                // fading down...
                //
                let f_fade_down_seconds = f_seconds_elapsed - (CARD_FADESECONDS + CARD_SUSTAINSECONDS);
                gv4Color[3] = 1.0f32 - (f_fade_down_seconds / CARD_FADESECONDS);
                //				OutputDebugString(va("fade dw: %f\n",gv4Color[3]));
            } else {
                gv4Color[3] = 1.0f32;
                //				OutputDebugString(va("normal: %f\n",gv4Color[3]));
            }
            if gv4Color[3] < 0.0f32 {
                gv4Color[3] = 0.0f32;  // ... otherwise numbers that have dipped slightly -ve flash up fullbright after fade down
            }

            //
            // how many lines is it?
            //
            let i_lines = credit_card.vstrText.len() + 2;  // +2 for title itself & one seperator line
            //
            let mut i_ypos = (SCREEN_HEIGHT - (i_lines as i32 * i_font_height)) / 2;
            //
            // draw it, title first...
            //
            let i_width = credit_card.strTitle.GetPixelLength();
            let i_xpos = (SCREEN_WIDTH - i_width) / 2;
            cgi_R_Font_DrawString(
                i_xpos,
                i_ypos,
                credit_card.strTitle.c_str().as_ptr() as *const u8,
                &gv4Color,
                ghFontHandle,
                -1,
                gfFontScale,
            );
            //
            i_ypos += i_font_height * 2;  // skip blank line then move to main pos
            //
            for i in 0..credit_card.vstrText.len() {
                let string_and_size = &credit_card.vstrText[i];
                let i_width = string_and_size.GetPixelLength();
                let i_xpos = (SCREEN_WIDTH - i_width) / 2;
                cgi_R_Font_DrawString(
                    i_xpos,
                    i_ypos,
                    string_and_size.c_str().as_ptr() as *const u8,
                    &gv4Color,
                    ghFontHandle,
                    -1,
                    gfFontScale,
                );
                i_ypos += i_font_height;
            }

            // next card?...
            //
            if f_seconds_elapsed > CARD_FADESECONDS + CARD_SUSTAINSECONDS + CARD_FADESECONDS {
                // yep, so erase the first entry (which will trigger the next one to be initialised on re-entry)...
                //
                CreditData.CreditCards.pop_front();

                if CreditData.CreditCards.is_empty() {
                    // all cards gone, so re-init timer for lines...
                    //
                    CreditData.iStartTime = cg.time;
                }
            }
            //
            return true;
        } else {
            // doing scroll text...
            //
            if !CreditData.CreditLines.is_empty() {
                // process all lines...
                //
                let f_milli_seconds_elapsed = (cg.time - CreditData.iStartTime) as f32;
                let f_seconds_elapsed = f_milli_seconds_elapsed / 1000.0f32;

                let f_pixels_per_second = (SCREEN_HEIGHT as f32) / LINE_SECONDTOSCROLLUP;

                let mut indices_to_remove = Vec::new();

                for i in 0..CreditData.CreditLines.len() {
                    let credit_line = &CreditData.CreditLines[i];
                    let mut i_ypos = SCREEN_HEIGHT + (credit_line.iLine * i_font_height);
                    i_ypos -= (f_pixels_per_second * f_seconds_elapsed) as i32;

                    let i_text_lines_this_item = max(credit_line.vstrText.len() as i32, 1);
                    if i_ypos + (i_text_lines_this_item * i_font_height) < 0 {
                        // scrolled off top of screen, so mark for removal...
                        //
                        indices_to_remove.push(i);
                    } else if i_ypos < SCREEN_HEIGHT {
                        // onscreen, so print it...
                        //
                        let credit_line = &CreditData.CreditLines[i];
                        let b_is_dotted = !credit_line.vstrText.is_empty();  // eg "STUNTS ...................... MR ED"

                        let i_width = credit_line.strText.GetPixelLength();
                        let i_xpos = if b_is_dotted { 4 } else { (SCREEN_WIDTH - i_width) / 2 };

                        gv4Color[3] = 1.0f32;

                        cgi_R_Font_DrawString(
                            i_xpos,
                            i_ypos,
                            credit_line.strText.c_str().as_ptr() as *const u8,
                            &gv4Color,
                            ghFontHandle,
                            -1,
                            gfFontScale,
                        );

                        // now print any dotted members...
                        //
                        let mut i_ypos_inner = i_ypos;
                        for j in 0..credit_line.vstrText.len() {
                            let string_and_size = &credit_line.vstrText[j];
                            let i_width = string_and_size.GetPixelLength();
                            let i_xpos = SCREEN_WIDTH - 4 - i_width;
                            cgi_R_Font_DrawString(
                                i_xpos,
                                i_ypos_inner,
                                string_and_size.c_str().as_ptr() as *const u8,
                                &gv4Color,
                                ghFontHandle,
                                -1,
                                gfFontScale,
                            );
                            i_ypos_inner += i_font_height;
                        }
                    }
                }

                // Remove marked indices in reverse order to maintain validity
                for i in indices_to_remove.iter().rev() {
                    CreditData.CreditLines.remove(*i);
                }

                return true;
            }
        }
    }

    false
}

////////////////////// eof /////////////////////
