#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// #include "../client/client.h"
// #include "../renderer/tr_local.h"
// #include "../win32/glw_win_dx8.h"
// #include "../win32/win_local.h"
// #include "../win32/win_file.h"
// #include "../ui/ui_splash.h"

const SP_TextureExt: &[u8] = b"dds";

extern "C" {
    fn Sys_QuickStart() -> bool;
    fn CIN_PlayAllFrames(
        name: *const c_char,
        a: c_int,
        b: c_int,
        c: c_int,
        d: c_int,
        e: c_int,
        f: bool,
    );
    fn qglGenTextures(n: c_int, textures: *mut u32);
    fn qglBindTexture(target: u32, texture: u32);
    fn qglTexImage2D(
        target: u32,
        level: c_int,
        internalformat: u32,
        width: c_int,
        height: c_int,
        border: c_int,
        format: u32,
        r#type: u32,
        pixels: *const c_void,
    );
    fn qglTexParameterf(target: u32, pname: u32, param: f32);
    fn qglColor3f(red: f32, green: f32, blue: f32);
    fn qglViewport(x: c_int, y: c_int, width: c_int, height: c_int);
    fn qglIsEnabled(cap: u32) -> bool;
    fn qglDisable(cap: u32);
    fn qglEnable(cap: u32);
    fn qglMatrixMode(mode: u32);
    fn qglLoadIdentity();
    fn qglOrtho(left: f64, right: f64, bottom: f64, top: f64, zNear: f64, zFar: f64);
    fn qglActiveTextureARB(texture: u32);
    fn qglClientActiveTextureARB(texture: u32);
    fn memset(s: *mut c_void, c: c_int, n: usize);
    fn qglBeginFrame();
    fn qglClearColor(red: f32, green: f32, blue: f32, alpha: f32);
    fn qglClear(mask: u32);
    fn qglBeginEXT(mode: u32, a: c_int, b: c_int, c: c_int, d: c_int, e: c_int);
    fn qglTexCoord2f(s: f32, t: f32);
    fn qglVertex2f(x: f32, y: f32);
    fn qglEnd();
    fn qglEndFrame();
    fn qglFlush();
    fn qglDeleteTextures(n: c_int, textures: *const u32);
    fn XGetLanguage() -> c_int;
    fn WF_Open(name: *const c_char, a: bool, b: bool) -> c_int;
    fn WF_Seek(offset: c_int, whence: c_int, handle: c_int) -> c_int;
    fn WF_Close(handle: c_int);
    fn WF_Tell(handle: c_int) -> c_int;
    fn WF_Read(buf: *mut c_void, len: c_int, handle: c_int) -> c_int;
    fn Z_Malloc(len: c_int, tag: c_int, flag: bool, align: c_int) -> *mut c_void;
    fn Z_Free(buf: *mut c_void);
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;

    static mut tess: c_void;
}

// GL constants
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_DDS1_EXT: u32 = 0x83F1;
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_LINEAR: u32 = 0x2601;
const GL_CLAMP: u32 = 0x2900;
const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
const GL_ALPHA_TEST: u32 = 0x0BC0;
const GL_BLEND: u32 = 0x0BE2;
const GL_CULL_FACE: u32 = 0x0B44;
const GL_DEPTH_TEST: u32 = 0x0B71;
const GL_FOG: u32 = 0x0B60;
const GL_LIGHTING: u32 = 0x0B50;
const GL_POLYGON_OFFSET_FILL: u32 = 0x8037;
const GL_SCISSOR_TEST: u32 = 0x0C11;
const GL_STENCIL_TEST: u32 = 0x0B90;
const GL_MODELVIEW: u32 = 0x1700;
const GL_PROJECTION: u32 = 0x1701;
const GL_TEXTURE0: u32 = 0x84C0;
const GL_TEXTURE1: u32 = 0x84C1;
const GL_TEXTURE0_ARB: u32 = 0x84C0;
const GL_TEXTURE1_ARB: u32 = 0x84C1;
const GL_TRIANGLE_STRIP: u32 = 0x0005;
const GL_UNSIGNED_BYTE: u32 = 0x1401;

// Language constants
const XC_LANGUAGE_ENGLISH: c_int = 0;
const XC_LANGUAGE_JAPANESE: c_int = 1;
const XC_LANGUAGE_GERMAN: c_int = 2;
const XC_LANGUAGE_SPANISH: c_int = 3;
const XC_LANGUAGE_ITALIAN: c_int = 4;
const XC_LANGUAGE_KOREAN: c_int = 5;
const XC_LANGUAGE_TCHINESE: c_int = 6;
const XC_LANGUAGE_PORTUGUESE: c_int = 7;

// File constants
const MAX_QPATH: usize = 260;
const SEEK_SET: c_int = 0;
const SEEK_END: c_int = 2;

// Tags for memory
const TAG_TEMP_WORKSPACE: c_int = 0;

/*********
Globals
*********/
static mut SP_LicenseDone: bool = false;

/*********
SP_DisplayIntros
Draws intro movies to the screen
*********/
pub unsafe fn SP_DisplayLogos() {
    if !Sys_QuickStart() {
        CIN_PlayAllFrames(b"logos\0".as_ptr() as *const c_char, 0, 0, 640, 480, 0, true);
    }
}

/*********
SP_DrawTexture
*********/
pub unsafe fn SP_DrawTexture(pixels: *mut c_void, width: f32, height: f32, vShift: f32) {
    if pixels.is_null() {
        // Ug.  We were not even able to load the error message texture.
        return;
    }

    // Create a texture from the buffered file
    let mut texid: u32 = 0;
    qglGenTextures(1, &mut texid);
    qglBindTexture(GL_TEXTURE_2D, texid);
    qglTexImage2D(
        GL_TEXTURE_2D,
        0,
        GL_DDS1_EXT,
        width as c_int,
        height as c_int,
        0,
        GL_DDS1_EXT,
        GL_UNSIGNED_BYTE,
        pixels,
    );

    qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as f32);
    qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as f32);
    qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP as f32);
    qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP as f32);

    // Reset every GL state we've got.  Who knows what state
    // the renderer could be in when this function gets called.
    qglColor3f(1.0, 1.0, 1.0);
    qglViewport(0, 0, 640, 480);

    let alpha = qglIsEnabled(GL_ALPHA_TEST);
    qglDisable(GL_ALPHA_TEST);

    let blend = qglIsEnabled(GL_BLEND);
    qglDisable(GL_BLEND);

    let cull = qglIsEnabled(GL_CULL_FACE);
    qglDisable(GL_CULL_FACE);

    let depth = qglIsEnabled(GL_DEPTH_TEST);
    qglDisable(GL_DEPTH_TEST);

    let fog = qglIsEnabled(GL_FOG);
    qglDisable(GL_FOG);

    let lighting = qglIsEnabled(GL_LIGHTING);
    qglDisable(GL_LIGHTING);

    let offset = qglIsEnabled(GL_POLYGON_OFFSET_FILL);
    qglDisable(GL_POLYGON_OFFSET_FILL);

    let scissor = qglIsEnabled(GL_SCISSOR_TEST);
    qglDisable(GL_SCISSOR_TEST);

    let stencil = qglIsEnabled(GL_STENCIL_TEST);
    qglDisable(GL_STENCIL_TEST);

    let texture = qglIsEnabled(GL_TEXTURE_2D);
    qglEnable(GL_TEXTURE_2D);

    qglMatrixMode(GL_MODELVIEW);
    qglLoadIdentity();
    qglMatrixMode(GL_PROJECTION);
    qglLoadIdentity();
    qglOrtho(0.0, 640.0, 0.0, 480.0, 0.0, 1.0);

    qglMatrixMode(GL_TEXTURE0);
    qglLoadIdentity();
    qglMatrixMode(GL_TEXTURE1);
    qglLoadIdentity();

    qglActiveTextureARB(GL_TEXTURE0_ARB);
    qglClientActiveTextureARB(GL_TEXTURE0_ARB);

    memset(&mut tess as *mut _ as *mut c_void, 0, core::mem::size_of_val(&tess));

    // Draw the error message
    qglBeginFrame();

    if !SP_LicenseDone {
        // clear the screen if we haven't done the
        // license yet...
        qglClearColor(0.0, 0.0, 0.0, 1.0);
        qglClear(GL_COLOR_BUFFER_BIT);
    }

    let x1 = 320.0 - width / 2.0;
    let x2 = 320.0 + width / 2.0;
    let mut y1 = 240.0 - height / 2.0;
    let mut y2 = 240.0 + height / 2.0;

    y1 += vShift;
    y2 += vShift;

    qglBeginEXT(GL_TRIANGLE_STRIP, 4, 0, 0, 4, 0);
    qglTexCoord2f(0.0, 0.0);
    qglVertex2f(x1, y1);
    qglTexCoord2f(1.0, 0.0);
    qglVertex2f(x2, y1);
    qglTexCoord2f(0.0, 1.0);
    qglVertex2f(x1, y2);
    qglTexCoord2f(1.0, 1.0);
    qglVertex2f(x2, y2);
    qglEnd();

    qglEndFrame();
    qglFlush();

    // Restore (most) of the render states we reset
    if alpha {
        qglEnable(GL_ALPHA_TEST);
    } else {
        qglDisable(GL_ALPHA_TEST);
    }

    if blend {
        qglEnable(GL_BLEND);
    } else {
        qglDisable(GL_BLEND);
    }

    if cull {
        qglEnable(GL_CULL_FACE);
    } else {
        qglDisable(GL_CULL_FACE);
    }

    if depth {
        qglEnable(GL_DEPTH_TEST);
    } else {
        qglDisable(GL_DEPTH_TEST);
    }

    if fog {
        qglEnable(GL_FOG);
    } else {
        qglDisable(GL_FOG);
    }

    if lighting {
        qglEnable(GL_LIGHTING);
    } else {
        qglDisable(GL_LIGHTING);
    }

    if offset {
        qglEnable(GL_POLYGON_OFFSET_FILL);
    } else {
        qglDisable(GL_POLYGON_OFFSET_FILL);
    }

    if scissor {
        qglEnable(GL_SCISSOR_TEST);
    } else {
        qglDisable(GL_SCISSOR_TEST);
    }

    if stencil {
        qglEnable(GL_STENCIL_TEST);
    } else {
        qglDisable(GL_STENCIL_TEST);
    }

    if texture {
        qglEnable(GL_TEXTURE_2D);
    } else {
        qglDisable(GL_TEXTURE_2D);
    }

    // Kill the texture
    qglDeleteTextures(1, &texid);
}

/*********
SP_GetLanguageExt

Retuns the extension for the current language, or
english if the language is unknown.
*********/
pub unsafe fn SP_GetLanguageExt() -> *const c_char {
    match XGetLanguage() {
        XC_LANGUAGE_ENGLISH => b"EN\0".as_ptr() as *const c_char,
        XC_LANGUAGE_JAPANESE => b"JA\0".as_ptr() as *const c_char,
        XC_LANGUAGE_GERMAN => b"GE\0".as_ptr() as *const c_char,
        XC_LANGUAGE_SPANISH => b"SP\0".as_ptr() as *const c_char,
        XC_LANGUAGE_ITALIAN => b"IT\0".as_ptr() as *const c_char,
        XC_LANGUAGE_KOREAN => b"KO\0".as_ptr() as *const c_char,
        XC_LANGUAGE_TCHINESE => b"CH\0".as_ptr() as *const c_char,
        XC_LANGUAGE_PORTUGUESE => b"PO\0".as_ptr() as *const c_char,
        _ => b"EN\0".as_ptr() as *const c_char,
    }
}

/*********
SP_LoadFileWithLanguage

Loads a screen with the appropriate language
*********/
pub unsafe fn SP_LoadFileWithLanguage(name: *const c_char) -> *mut c_void {
    let mut fullname: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut buffer: *mut c_void = core::ptr::null_mut();
    let ext: *const c_char;

    // get the language extension
    ext = SP_GetLanguageExt();

    // creat the fullpath name and try to load the texture
    sprintf(
        fullname.as_mut_ptr(),
        b"%s_%s.dds\0".as_ptr() as *const c_char,
        name,
        ext,
    );
    buffer = SP_LoadFile(fullname.as_ptr());

    if buffer.is_null() {
        sprintf(
            fullname.as_mut_ptr(),
            b"%s.dds\0".as_ptr() as *const c_char,
            name,
        );
        buffer = SP_LoadFile(fullname.as_ptr());
    }

    buffer
}

/*********
SP_LoadFile
*********/
pub unsafe fn SP_LoadFile(name: *const c_char) -> *mut c_void {
    let h = WF_Open(name, true, false);
    if h < 0 {
        return core::ptr::null_mut();
    }

    if WF_Seek(0, SEEK_END, h) != 0 {
        WF_Close(h);
        return core::ptr::null_mut();
    }

    let len = WF_Tell(h);

    if WF_Seek(0, SEEK_SET, h) != 0 {
        WF_Close(h);
        return core::ptr::null_mut();
    }

    let buf = Z_Malloc(len, TAG_TEMP_WORKSPACE, false, 32);

    if WF_Read(buf, len, h) != len {
        Z_Free(buf);
        WF_Close(h);
        return core::ptr::null_mut();
    }

    WF_Close(h);

    buf
}

/********
SP_DoLicense

Draws the license splash to the screen
*********/
pub unsafe fn SP_DoLicense() {
    if Sys_QuickStart() {
        return;
    }

    // Load the license screen
    let license: *mut c_void;
    license = SP_LoadFileWithLanguage(b"d:\\base\\media\\LicenseScreen\0".as_ptr() as *const c_char);

    if !license.is_null() {
        for _c in 0..3 {
            SP_DrawTexture(license, 1024.0, 1024.0, -20.0);
        }
        Z_Free(license);
    }

    SP_LicenseDone = true;
}
