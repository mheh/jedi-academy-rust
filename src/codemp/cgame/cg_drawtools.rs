// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_drawtools.c -- helper functions called by cg_draw, cg_scoreboard, cg_info, etc

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::{c_int, c_char, c_void};

// ============================================================================
// Type definitions
// ============================================================================

pub type vec_t = f32;
pub type vec3_t = [vec_t; 3];
pub type vec4_t = [vec_t; 4];
pub type qhandle_t = c_int;
pub type qboolean = c_int;

const QFALSE: c_int = 0;
const QTRUE: c_int = 1;

// Constants from cg_local.h and q_shared.h
const FADE_TIME: c_int = 200;
const BIGCHAR_WIDTH: c_int = 16;
const BIGCHAR_HEIGHT: c_int = 16;
const SMALLCHAR_WIDTH: c_int = 8;
const SMALLCHAR_HEIGHT: c_int = 8;
const STAT_MINUS: c_int = 10;
const NUM_FONT_SMALL: c_int = 2;
const NUM_FONT_CHUNKY: c_int = 1;
const NUM_FONT_BIG: c_int = 0;
const STAT_HEALTH: usize = 0;
const STAT_ARMOR: usize = 2;

// Constants from bg_public.h
const ARMOR_PROTECTION: f32 = 0.50;

// UI style constants from ui_shared.h
const UI_LEFT: c_int = 0x00000000;
const UI_CENTER: c_int = 0x00000010;
const UI_RIGHT: c_int = 0x00000020;
const UI_SMALLFONT: c_int = 0x00000001;
const UI_DROPSHADOW: c_int = 0x00000040;
const UI_BLINK: c_int = 0x00000004;
const UI_PULSE: c_int = 0x00000002;

// Font types for CG_Text_Paint
const FONT_SMALL: c_int = 0;
const FONT_MEDIUM: c_int = 1;

// Text styles for CG_Text_Paint
const ITEM_TEXTSTYLE_NORMAL: c_int = 0;
const ITEM_TEXTSTYLE_SHADOWED: c_int = 1;
const ITEM_TEXTSTYLE_BLINK: c_int = 2;

// ============================================================================
// Type stubs for structural coherence
// ============================================================================

#[repr(C)]
pub struct cg_media_t {
    pub whiteShader: qhandle_t,
    pub charsetShader: qhandle_t,
    pub backTileShader: qhandle_t,
    pub numberShaders: [qhandle_t; 11],
    pub smallnumberShaders: [qhandle_t; 11],
    pub chunkyNumberShaders: [qhandle_t; 11],
    // ... other fields
    _rest: [u8; 0],
}

#[repr(C)]
pub struct glconfig_t {
    pub vidWidth: c_int,
    pub vidHeight: c_int,
    // ... other fields
    _rest: [u8; 0],
}

#[repr(C)]
pub struct refdef_t {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    // ... other fields
    _rest: [u8; 0],
}

#[repr(C)]
pub struct playerState_t {
    pub stats: [c_int; 16],
    // ... other fields
    _rest: [u8; 0],
}

#[repr(C)]
pub struct snapshot_t {
    pub ps: playerState_t,
    // ... other fields
    _rest: [u8; 0],
}

#[repr(C)]
pub struct cgs_t {
    pub screenXScale: f32,
    pub screenYScale: f32,
    pub media: cg_media_t,
    pub glconfig: glconfig_t,
    // ... other fields
    _rest: [u8; 0],
}

#[repr(C)]
pub struct cg_t {
    pub time: c_int,
    pub refdef: *mut refdef_t,
    pub snap: *mut snapshot_t,
    // ... other fields
    _rest: [u8; 0],
}

// ============================================================================
// External functions and globals
// ============================================================================

extern "C" {
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static g_color_table: [vec4_t; 8];

    // Trap functions
    pub fn trap_R_SetColor(rgba: *const f32);
    pub fn trap_R_DrawStretchPic(x: f32, y: f32, w: f32, h: f32,
                                  s1: f32, t1: f32, s2: f32, t2: f32, hShader: qhandle_t);
    pub fn trap_R_DrawRotatePic(x: f32, y: f32, w: f32, h: f32,
                                 s1: f32, t1: f32, s2: f32, t2: f32, a: f32, hShader: qhandle_t);
    pub fn trap_R_DrawRotatePic2(x: f32, y: f32, w: f32, h: f32,
                                  s1: f32, t1: f32, s2: f32, t2: f32, a: f32, hShader: qhandle_t);
    pub fn trap_Language_IsAsian() -> c_int;

    // Text rendering functions (from cg_text.c or ui module)
    pub fn CG_Text_Paint(x: f32, y: f32, scale: f32, color: *const vec4_t, text: *const u8,
                         cursorPos: c_int, limit: c_int, style: c_int, font: c_int);
    pub fn CG_Text_Width(text: *const u8, scale: f32, font: c_int) -> f32;

    // Standard C library functions
    pub fn memcpy(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;
    pub fn strlen(s: *const u8) -> usize;
    pub fn Com_sprintf(dest: *mut c_char, destsize: c_int, fmt: *const c_char, ...);
}

// Local stub for safe strlen wrapper
pub unsafe fn safe_strlen(s: *const u8) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

// Macro equivalent for ColorIndex: ( ( (c) - '0' ) & 7 )
#[inline]
fn ColorIndex(c: u8) -> usize {
    (((c as c_int) - (b'0' as c_int)) & 7) as usize
}

// Macro equivalent for Q_IsColorString
#[inline]
pub fn Q_IsColorString(p: *const u8) -> bool {
    unsafe {
        *p == b'^' as u8 && *p.add(1) != 0
    }
}

// ============================================================================
// CG_DrawRect
// ============================================================================

/*
================
UI_DrawRect

Coordinates are 640*480 virtual values
=================
*/
pub unsafe extern "C" fn CG_DrawRect(x: f32, y: f32, width: f32, height: f32, size: f32, color: *const f32) {
    trap_R_SetColor(color);

    CG_DrawTopBottom(x, y, width, height, size);
    CG_DrawSides(x, y, width, height, size);

    trap_R_SetColor(core::ptr::null());
}

// ============================================================================
// CG_GetColorForHealth
// ============================================================================

/*
=================
CG_GetColorForHealth
=================
*/
pub unsafe extern "C" fn CG_GetColorForHealth(health: c_int, armor: c_int, hcolor: *mut vec4_t) {
    let mut count: c_int;
    let mut max: c_int;

    // calculate the total points of damage that can
    // be sustained at the current health / armor level
    if health <= 0 {
        VectorClear(&mut *hcolor); // black
        (*hcolor)[3] = 1.0;
        return;
    }
    count = armor;
    max = (health as f32 * ARMOR_PROTECTION / (1.0 - ARMOR_PROTECTION)) as c_int;
    if max < count {
        count = max;
    }
    let total_health = health + count;

    // set the color based on health
    (*hcolor)[0] = 1.0;
    (*hcolor)[3] = 1.0;
    if total_health >= 100 {
        (*hcolor)[2] = 1.0;
    } else if total_health < 66 {
        (*hcolor)[2] = 0.0;
    } else {
        (*hcolor)[2] = (total_health as f32 - 66.0) / 33.0;
    }

    if total_health > 60 {
        (*hcolor)[1] = 1.0;
    } else if total_health < 30 {
        (*hcolor)[1] = 0.0;
    } else {
        (*hcolor)[1] = (total_health as f32 - 30.0) / 30.0;
    }
}

// ============================================================================
// CG_DrawSides
// ============================================================================

/*
================
CG_DrawSides

Coords are virtual 640x480
================
*/
pub unsafe extern "C" fn CG_DrawSides(x: f32, y: f32, w: f32, h: f32, size: f32) {
    let scaled_size = size * cgs.screenXScale;
    trap_R_DrawStretchPic(x, y, scaled_size, h, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader);
    trap_R_DrawStretchPic(x + w - scaled_size, y, scaled_size, h, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader);
}

pub unsafe extern "C" fn CG_DrawTopBottom(x: f32, y: f32, w: f32, h: f32, size: f32) {
    let scaled_size = size * cgs.screenYScale;
    trap_R_DrawStretchPic(x, y, w, scaled_size, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader);
    trap_R_DrawStretchPic(x, y + h - scaled_size, w, scaled_size, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader);
}

// ============================================================================
// CG_FillRect2
// ============================================================================

/*
-------------------------
CGC_FillRect2
real coords
-------------------------
*/
pub unsafe extern "C" fn CG_FillRect2(x: f32, y: f32, width: f32, height: f32, color: *const f32) {
    trap_R_SetColor(color);
    trap_R_DrawStretchPic(x, y, width, height, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader);
    trap_R_SetColor(core::ptr::null());
}

// ============================================================================
// CG_FillRect
// ============================================================================

/*
================
CG_FillRect

Coordinates are 640*480 virtual values
=================
*/
pub unsafe extern "C" fn CG_FillRect(x: f32, y: f32, width: f32, height: f32, color: *const f32) {
    trap_R_SetColor(color);
    trap_R_DrawStretchPic(x, y, width, height, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader);
    trap_R_SetColor(core::ptr::null());
}

// ============================================================================
// CG_DrawPic
// ============================================================================

/*
================
CG_DrawPic

Coordinates are 640*480 virtual values
A width of 0 will draw with the original image width
=================
*/
pub unsafe extern "C" fn CG_DrawPic(x: f32, y: f32, width: f32, height: f32, hShader: qhandle_t) {
    trap_R_DrawStretchPic(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}

// ============================================================================
// CG_DrawRotatePic
// ============================================================================

/*
================
CG_DrawRotatePic

Coordinates are 640*480 virtual values
A width of 0 will draw with the original image width
rotates around the upper right corner of the passed in point
=================
*/
pub unsafe extern "C" fn CG_DrawRotatePic(x: f32, y: f32, width: f32, height: f32, angle: f32, hShader: qhandle_t) {
    trap_R_DrawRotatePic(x, y, width, height, 0.0, 0.0, 1.0, 1.0, angle, hShader);
}

// ============================================================================
// CG_DrawRotatePic2
// ============================================================================

/*
================
CG_DrawRotatePic2

Coordinates are 640*480 virtual values
A width of 0 will draw with the original image width
Actually rotates around the center point of the passed in coordinates
=================
*/
pub unsafe extern "C" fn CG_DrawRotatePic2(x: f32, y: f32, width: f32, height: f32, angle: f32, hShader: qhandle_t) {
    trap_R_DrawRotatePic2(x, y, width, height, 0.0, 0.0, 1.0, 1.0, angle, hShader);
}

// ============================================================================
// CG_DrawChar
// ============================================================================

/*
===============
CG_DrawChar

Coordinates and size in 640*480 virtual screen size
===============
*/
pub unsafe extern "C" fn CG_DrawChar(x: c_int, y: c_int, width: c_int, height: c_int, ch: c_int) {
    let ch = (ch & 255) as c_int;

    if ch == b' ' as c_int {
        return;
    }

    let ax = x as f32;
    let ay = y as f32;
    let aw = width as f32;
    let ah = height as f32;

    let row = ch >> 4;
    let col = ch & 15;

    let frow = row as f32 * 0.0625;
    let fcol = col as f32 * 0.0625;
    let size = 0.03125;
    let size2 = 0.0625;

    trap_R_DrawStretchPic(ax, ay, aw, ah, fcol, frow, fcol + size, frow + size2,
        cgs.media.charsetShader);
}

// ============================================================================
// CG_DrawStringExt
// ============================================================================

/*
==================
CG_DrawStringExt

Draws a multi-colored string with a drop shadow, optionally forcing
to a fixed color.

Coordinates are at 640 by 480 virtual resolution
==================
*/
pub unsafe extern "C" fn CG_DrawStringExt(x: c_int, y: c_int, string: *const c_char, setColor: *const f32,
        forceColor: qboolean, shadow: qboolean, charWidth: c_int, charHeight: c_int, maxChars: c_int)
{
    if trap_Language_IsAsian() != 0 {
        // hack-a-doodle-do (post-release quick fix code)...
        //
        let mut color: vec4_t = [0.0; 4];
        memcpy(color.as_mut_ptr() as *mut c_void, setColor as *const c_void, core::mem::size_of::<vec4_t>());
        let iStyle = if shadow != 0 { ITEM_TEXTSTYLE_SHADOWED } else { ITEM_TEXTSTYLE_NORMAL };
        CG_Text_Paint(x as f32, y as f32, 1.0,
                        &color,
                        string as *const u8,
                        0,
                        0,
                        iStyle,
                        FONT_MEDIUM);
    } else {
        let mut color: vec4_t = [0.0; 4];
        let mut s: *const c_char;
        let mut xx: c_int;

        // draw the drop shadow
        if shadow != 0 {
            color[0] = 0.0;
            color[1] = 0.0;
            color[2] = 0.0;
            color[3] = *setColor.add(3);
            trap_R_SetColor(color.as_ptr());
            s = string;
            xx = x;
            while *s != 0 {
                if Q_IsColorString(s as *const u8) {
                    s = s.add(2);
                    continue;
                }
                CG_DrawChar(xx + 2, y + 2, charWidth, charHeight, *s as c_int);
                xx += charWidth;
                s = s.add(1);
            }
        }

        // draw the colored text
        s = string;
        xx = x;
        trap_R_SetColor(setColor);
        while *s != 0 {
            if Q_IsColorString(s as *const u8) {
                if forceColor == 0 {
                    let idx = ColorIndex(*s.add(1) as u8);
                    memcpy(color.as_mut_ptr() as *mut c_void, g_color_table[idx].as_ptr() as *const c_void, core::mem::size_of::<vec4_t>());
                    color[3] = *setColor.add(3);
                    trap_R_SetColor(color.as_ptr());
                }
                s = s.add(2);
                continue;
            }
            CG_DrawChar(xx, y, charWidth, charHeight, *s as c_int);
            xx += charWidth;
            s = s.add(1);
        }
        trap_R_SetColor(core::ptr::null());
    }
}

// ============================================================================
// CG_DrawBigString
// ============================================================================

pub unsafe extern "C" fn CG_DrawBigString(x: c_int, y: c_int, s: *const c_char, alpha: f32) {
    let mut color: vec4_t = [0.0; 4];

    color[0] = 1.0;
    color[1] = 1.0;
    color[2] = 1.0;
    color[3] = alpha;
    CG_DrawStringExt(x, y, s, color.as_ptr(), QFALSE, QTRUE, BIGCHAR_WIDTH, BIGCHAR_HEIGHT, 0);
}

// ============================================================================
// CG_DrawBigStringColor
// ============================================================================

pub unsafe extern "C" fn CG_DrawBigStringColor(x: c_int, y: c_int, s: *const c_char, color: *const vec4_t) {
    CG_DrawStringExt(x, y, s, color as *const f32, QTRUE, QTRUE, BIGCHAR_WIDTH, BIGCHAR_HEIGHT, 0);
}

// ============================================================================
// CG_DrawSmallString
// ============================================================================

pub unsafe extern "C" fn CG_DrawSmallString(x: c_int, y: c_int, s: *const c_char, alpha: f32) {
    let mut color: vec4_t = [0.0; 4];

    color[0] = 1.0;
    color[1] = 1.0;
    color[2] = 1.0;
    color[3] = alpha;
    CG_DrawStringExt(x, y, s, color.as_ptr(), QFALSE, QFALSE, SMALLCHAR_WIDTH, SMALLCHAR_HEIGHT, 0);
}

// ============================================================================
// CG_DrawSmallStringColor
// ============================================================================

pub unsafe extern "C" fn CG_DrawSmallStringColor(x: c_int, y: c_int, s: *const c_char, color: *const vec4_t) {
    CG_DrawStringExt(x, y, s, color as *const f32, QTRUE, QFALSE, SMALLCHAR_WIDTH, SMALLCHAR_HEIGHT, 0);
}

// ============================================================================
// CG_DrawStrlen
// ============================================================================

/*
=================
CG_DrawStrlen

Returns character count, skiping color escape codes
=================
*/
pub unsafe extern "C" fn CG_DrawStrlen(str: *const c_char) -> c_int {
    let mut s = str;
    let mut count = 0;

    while *s != 0 {
        if Q_IsColorString(s as *const u8) {
            s = s.add(2);
        } else {
            count += 1;
            s = s.add(1);
        }
    }

    count
}

// ============================================================================
// CG_TileClearBox
// ============================================================================

/*
=============
CG_TileClearBox

This repeats a 64*64 tile graphic to fill the screen around a sized down
refresh window.
=============
*/
unsafe fn CG_TileClearBox(x: c_int, y: c_int, w: c_int, h: c_int, hShader: qhandle_t) {
    let s1 = x as f32 / 64.0;
    let t1 = y as f32 / 64.0;
    let s2 = (x + w) as f32 / 64.0;
    let t2 = (y + h) as f32 / 64.0;
    trap_R_DrawStretchPic(x as f32, y as f32, w as f32, h as f32, s1, t1, s2, t2, hShader);
}

// ============================================================================
// CG_TileClear
// ============================================================================

/*
==============
CG_TileClear

Clear around a sized down screen
==============
*/
pub unsafe extern "C" fn CG_TileClear() {
    let mut top: c_int;
    let mut bottom: c_int;
    let mut left: c_int;
    let mut right: c_int;
    let w: c_int;
    let h: c_int;

    let w = cgs.glconfig.vidWidth;
    let h = cgs.glconfig.vidHeight;

    if cg.refdef.is_null() {
        return;
    }

    if (*cg.refdef).x == 0 && (*cg.refdef).y == 0 &&
        (*cg.refdef).width == w && (*cg.refdef).height == h {
        return; // full screen rendering
    }

    let top = (*cg.refdef).y;
    let bottom = top + (*cg.refdef).height - 1;
    let left = (*cg.refdef).x;
    let right = left + (*cg.refdef).width - 1;

    // clear above view screen
    CG_TileClearBox(0, 0, w, top, cgs.media.backTileShader);

    // clear below view screen
    CG_TileClearBox(0, bottom, w, h - bottom, cgs.media.backTileShader);

    // clear left of view screen
    CG_TileClearBox(0, top, left, bottom - top + 1, cgs.media.backTileShader);

    // clear right of view screen
    CG_TileClearBox(right, top, w - right, bottom - top + 1, cgs.media.backTileShader);
}

// ============================================================================
// CG_FadeColor
// ============================================================================

/*
================
CG_FadeColor
================
*/
pub unsafe extern "C" fn CG_FadeColor(startMsec: c_int, totalMsec: c_int) -> *const vec4_t {
    static mut color: vec4_t = [0.0; 4];

    if startMsec == 0 {
        return core::ptr::null();
    }

    let t = cg.time - startMsec;

    if t >= totalMsec {
        return core::ptr::null();
    }

    // fade out
    if totalMsec - t < FADE_TIME {
        color[3] = ((totalMsec - t) as f32) * 1.0 / (FADE_TIME as f32);
    } else {
        color[3] = 1.0;
    }
    color[0] = 1.0;
    color[1] = 1.0;
    color[2] = 1.0;

    &color as *const vec4_t
}

// ============================================================================
// CG_ColorForGivenHealth
// ============================================================================

/*
=================
CG_ColorForHealth
=================
*/
pub unsafe extern "C" fn CG_ColorForGivenHealth(hcolor: *mut vec4_t, health: c_int)
{
    // set the color based on health
    (*hcolor)[0] = 1.0;
    if health >= 100
    {
        (*hcolor)[2] = 1.0;
    }
    else if health < 66
    {
        (*hcolor)[2] = 0.0;
    }
    else
    {
        (*hcolor)[2] = (health as f32 - 66.0) / 33.0;
    }

    if health > 60
    {
        (*hcolor)[1] = 1.0;
    }
    else if health < 30
    {
        (*hcolor)[1] = 0.0;
    }
    else
    {
        (*hcolor)[1] = (health as f32 - 30.0) / 30.0;
    }
}

// ============================================================================
// CG_ColorForHealth
// ============================================================================

/*
=================
CG_ColorForHealth
=================
*/
pub unsafe extern "C" fn CG_ColorForHealth(hcolor: *mut vec4_t)
{
    let health: c_int;
    let count: c_int;
    let max: c_int;

    // calculate the total points of damage that can
    // be sustained at the current health / armor level
    if cg.snap.is_null() {
        return;
    }

    let health = (*(*cg.snap)).ps.stats[STAT_HEALTH];

    if health <= 0
    {
        VectorClear(&mut *hcolor);
        (*hcolor)[3] = 1.0;
        return;
    }

    let count = (*(*cg.snap)).ps.stats[STAT_ARMOR];
    let max = (health as f32 * ARMOR_PROTECTION / (1.0 - ARMOR_PROTECTION)) as c_int;
    let count = if max < count { max } else { count };

    (*hcolor)[3] = 1.0;
    CG_ColorForGivenHealth(hcolor, health + count);
}

// ============================================================================
// CG_DrawNumField
// ============================================================================

/*
==============
CG_DrawNumField

Take x,y positions as if 640 x 480 and scales them to the proper resolution

==============
*/
pub unsafe extern "C" fn CG_DrawNumField(mut x: c_int, y: c_int, width: c_int, value: c_int,
        charWidth: c_int, charHeight: c_int, style: c_int, zeroFill: qboolean)
{
    let mut num: [c_char; 16] = [0; 16];
    let mut ptr: *const c_char;
    let frame: c_int;
    let mut xWidth: c_int;
    let mut i: c_int = 0;

    if width < 1 {
        return;
    }

    // draw number string
    let width = if width > 5 { 5 } else { width };

    let value = match width {
        1 => if value > 9 { 9 } else if value < 0 { 0 } else { value },
        2 => if value > 99 { 99 } else if value < -9 { -9 } else { value },
        3 => if value > 999 { 999 } else if value < -99 { -99 } else { value },
        4 => if value > 9999 { 9999 } else if value < -999 { -999 } else { value },
        _ => value,
    };

    Com_sprintf(num.as_mut_ptr(), 16, b"%i\0".as_ptr() as *const c_char, value);
    let l = safe_strlen(num.as_ptr() as *const u8) as c_int;
    let l = if l > width { width } else { l };

    // FIXME: Might need to do something different for the chunky font??
    let xWidth = match style {
        NUM_FONT_SMALL => charWidth,
        NUM_FONT_CHUNKY => ((charWidth as f32 / 1.2) as c_int) + 2,
        NUM_FONT_BIG | _ => ((charWidth as f32 / 2.0) as c_int) + 7,
    };

    if zeroFill != 0 {
        let mut i = 0;
        while i < (width - l) {
            match style {
                NUM_FONT_SMALL => {
                    CG_DrawPic(x, y, charWidth as f32, charHeight as f32, cgs.media.smallnumberShaders[0]);
                },
                NUM_FONT_CHUNKY => {
                    CG_DrawPic(x, y, charWidth as f32, charHeight as f32, cgs.media.chunkyNumberShaders[0]);
                },
                NUM_FONT_BIG | _ => {
                    CG_DrawPic(x, y, charWidth as f32, charHeight as f32, cgs.media.numberShaders[0]);
                },
            }
            x += 2 + xWidth;
            i += 1;
        }
    } else {
        x += 2 + xWidth * (width - l);
    }

    ptr = num.as_ptr();
    let mut l_remaining = l;
    while *ptr != 0 && l_remaining > 0 {
        let frame = if *ptr == b'-' as c_char {
            STAT_MINUS
        } else {
            (*ptr as c_int) - (b'0' as c_int)
        };

        match style {
            NUM_FONT_SMALL => {
                CG_DrawPic(x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.smallnumberShaders[frame as usize]);
                x += 1; // For a one line gap
            },
            NUM_FONT_CHUNKY => {
                CG_DrawPic(x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.chunkyNumberShaders[frame as usize]);
            },
            NUM_FONT_BIG | _ => {
                CG_DrawPic(x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.numberShaders[frame as usize]);
            },
        }

        x += xWidth;
        ptr = ptr.add(1);
        l_remaining -= 1;
    }
}

// ============================================================================
// UI_DrawProportionalString
// ============================================================================

pub unsafe extern "C" fn UI_DrawProportionalString(x: c_int, y: c_int, str: *const c_char, style: c_int, color: *const vec4_t)
{
    // having all these different style defines (1 for UI, one for CG, and now one for the re->font stuff)
    //	is dumb, but for now...
    //
    let mut iStyle: c_int = ITEM_TEXTSTYLE_NORMAL;
    let iMenuFont: c_int = if (style & UI_SMALLFONT) != 0 { FONT_SMALL } else { FONT_MEDIUM };

    let x = match style & (UI_LEFT | UI_CENTER | UI_RIGHT) {
        UI_CENTER => x - (CG_Text_Width(str as *const u8, 1.0, iMenuFont) as c_int) / 2,
        UI_RIGHT => x - (CG_Text_Width(str as *const u8, 1.0, iMenuFont) as c_int) / 2,
        _ => x, // UI_LEFT (default)
    };

    if (style & UI_DROPSHADOW) != 0 {
        iStyle = ITEM_TEXTSTYLE_SHADOWED;
    } else if (style & UI_BLINK) != 0 || (style & UI_PULSE) != 0 {
        iStyle = ITEM_TEXTSTYLE_BLINK;
    }

    CG_Text_Paint(x as f32, y as f32, 1.0, color, str as *const u8, 0, 0, iStyle, iMenuFont);
}

// ============================================================================
// UI_DrawScaledProportionalString
// ============================================================================

pub unsafe extern "C" fn UI_DrawScaledProportionalString(x: c_int, y: c_int, str: *const c_char, style: c_int, color: *const vec4_t, scale: f32)
{
    // having all these different style defines (1 for UI, one for CG, and now one for the re->font stuff)
    //	is dumb, but for now...
    //
    let mut iStyle: c_int = ITEM_TEXTSTYLE_NORMAL;

    let x = match style & (UI_LEFT | UI_CENTER | UI_RIGHT) {
        UI_CENTER => x - (CG_Text_Width(str as *const u8, scale, FONT_MEDIUM) as c_int) / 2,
        UI_RIGHT => x - (CG_Text_Width(str as *const u8, scale, FONT_MEDIUM) as c_int) / 2,
        _ => x, // UI_LEFT (default)
    };

    if (style & UI_DROPSHADOW) != 0 {
        iStyle = ITEM_TEXTSTYLE_SHADOWED;
    } else if (style & UI_BLINK) != 0 || (style & UI_PULSE) != 0 {
        iStyle = ITEM_TEXTSTYLE_BLINK;
    }

    CG_Text_Paint(x as f32, y as f32, scale, color, str as *const u8, 0, 0, iStyle, FONT_MEDIUM);
}

// ============================================================================
// Helper functions
// ============================================================================

#[inline]
fn VectorClear(a: &mut vec4_t) {
    a[0] = 0.0;
    a[1] = 0.0;
    a[2] = 0.0;
}
