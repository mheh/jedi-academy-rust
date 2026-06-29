// tr_draw.c
// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_float};
use core::mem;
use core::ptr::{addr_of, addr_of_mut};

// ========== Type Definitions and Stubs ==========

// q3 convention: byte = unsigned byte
type byte = u8;
// q3 convention: qboolean = int (1 or 0)
type qboolean = c_int;

const qtrue: c_int = 1;
const qfalse: c_int = 0;

// GL constants (local definitions since we can't include GL headers)
const GL_TEXTURE_2D: c_int = 0x0DE1;
const GL_RGB8: c_int = 0x8051;
const GL_RGBA8: c_int = 0x8058;
const GL_RGBA: c_int = 0x1908;
const GL_UNSIGNED_BYTE: c_int = 0x1401;
const GL_TEXTURE_MIN_FILTER: c_int = 0x2801;
const GL_TEXTURE_MAG_FILTER: c_int = 0x2800;
const GL_TEXTURE_WRAP_S: c_int = 0x2802;
const GL_TEXTURE_WRAP_T: c_int = 0x2803;
const GL_LINEAR: c_int = 0x2601;
const GL_CLAMP: c_int = 0x2900;
const GL_REPEAT: c_int = 0x2901;
const GL_QUADS: c_int = 0x0007;
const GL_TRIANGLE_STRIP: c_int = 0x0005;
const GL_DEPTH_BUFFER_BIT: c_int = 0x00000100;
const GL_DEPTH_CLEAR_VALUE: c_int = 0x0B73;
const GL_COMPRESSED_RGB_S3TC_DXT1_EXT: c_int = 0x83F0;
const GL_LIN_RGBA: c_int = 0x1908; // fallback for non-Xbox
const GL_RGB: c_int = 0x1907;

const GL_DEPTHMASK_TRUE: c_int = 0x01;
const GL_SRCBLEND_ZERO: c_int = 0x04;
const GL_SRCBLEND_ONE: c_int = 0x08;
const GL_DSTBLEND_ONE: c_int = 0x08;
const GL_DSTBLEND_ZERO: c_int = 0x04;
const GL_ATEST_LT_80: c_int = 0x0200;
const GL_DEPTHFUNC_EQUAL: c_int = 0x0202;

const GLS_DEPTHMASK_TRUE: c_int = GL_DEPTHMASK_TRUE;
const GLS_SRCBLEND_ZERO: c_int = GL_SRCBLEND_ZERO;
const GLS_DSTBLEND_ONE: c_int = GL_DSTBLEND_ONE;
const GLS_SRCBLEND_ONE: c_int = GL_SRCBLEND_ONE;
const GLS_DSTBLEND_ZERO: c_int = GL_DSTBLEND_ZERO;
const GLS_ATEST_LT_80: c_int = GL_ATEST_LT_80;
const GLS_DEPTHFUNC_EQUAL: c_int = GL_DEPTHFUNC_EQUAL;

// Other constants
const SCREEN_WIDTH: c_int = 640;
const SCREEN_HEIGHT: c_int = 480;
const TAG_TEMP_WORKSPACE: c_int = 0;
const PRINT_ALL: c_int = 0;
const ERR_DROP: c_int = 0;
const CT_TWO_SIDED: c_int = 2;

// Stub types for external dependencies (declared but not fully defined)
// These are defined in tr_local.h and other engine headers
#[repr(C)]
#[derive(Debug)]
pub struct image_t {
    // Stub: full definition in tr_local.h
    pub width: c_int,
    pub height: c_int,
    // ... other fields not needed for this file
}

#[repr(C)]
pub struct glConfig_s {
    pub vidWidth: c_int,
    pub vidHeight: c_int,
    pub maxTextureSize: c_int,
    // ... other fields
}

#[repr(C)]
pub struct refdef_t {
    // Stub
}

#[repr(C)]
pub struct backEndState_t {
    pub projection2D: qboolean,
    // ... other fields
}

#[repr(C)]
pub struct trGlobals_t {
    pub scratchImage: [*mut image_t; 2], // indexed by iClient; assuming max 2 clients per commentary
    pub identityLight: c_float,
    pub registered: qboolean,
    // ... other fields
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // ... other fields
}

// Global variables from tr_local.h and elsewhere
extern "C" {
    pub static mut tr: trGlobals_t;
    pub static mut glConfig: glConfig_s;
    pub static mut backEnd: backEndState_t;
    pub static mut com_buildScript: *mut cvar_t;
}

// ========== Dissolve Structures ==========

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dissolve_e {
    eDISSOLVE_RT_TO_LT = 0,
    eDISSOLVE_LT_TO_RT = 1,
    eDISSOLVE_TP_TO_BT = 2,
    eDISSOLVE_BT_TO_TP = 3,
    eDISSOLVE_CIRCULAR_OUT = 4,  // new image comes out from centre
    //
    eDISSOLVE_RAND_LIMIT = 5,    // label only, not valid to select
    //
    // any others...
    //
    eDISSOLVE_CIRCULAR_IN = 6,   // new image comes in from edges
    //
    eDISSOLVE_NUMBEROF = 7,
}

#[repr(C)]
pub struct Dissolve_t {
    pub iWidth: c_int,
    pub iHeight: c_int,
    pub iUploadWidth: c_int,
    pub iUploadHeight: c_int,
    pub iScratchPadNumber: c_int,
    pub pImage: *mut image_t,     // old image screen
    pub pDissolve: *mut image_t,  // fuzzy thing
    pub pBlack: *mut image_t,     // small black image for clearing
    pub iStartTime: c_int,        // 0 = not processing
    pub eDissolveType: Dissolve_e,
    pub bTouchNeeded: qboolean,
}

// ========== External Functions ==========

extern "C" {
    pub fn R_SyncRenderThread();
    pub fn Com_Error(error_level: c_int, msg: *const c_char, ...);
    pub fn GL_Bind(image: *const image_t);
    pub fn GL_State(state: c_int);
    pub fn GL_Cull(cull_type: c_int);
    pub fn Sys_Milliseconds() -> c_int;
    pub fn VID_Printf(level: c_int, msg: *const c_char, ...);
    pub fn R_LoadImage(
        filename: *const c_char,
        pic: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
        format: *mut c_int,
    );
    pub fn Z_Malloc(size: c_int, tag: c_int, clear: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn R_Free(ptr: *mut c_void);
    pub fn R_Images_DeleteImage(image: *mut image_t);
    pub fn R_CreateImage(
        name: *const c_char,
        pic: *const byte,
        width: c_int,
        height: c_int,
        format: c_int,
        mipmap: qboolean,
        allow_picmip: qboolean,
        allow_tc: qboolean,
        wrap_mode: c_int,
    ) -> *mut image_t;
    pub fn R_FindImageFile(
        name: *const c_char,
        mipmap: qboolean,
        allow_picmip: qboolean,
        allow_tc: qboolean,
        wrap_mode: c_int,
    ) -> *mut image_t;
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
    pub fn RB_SetGL2D();

    // GL functions
    pub fn qglFinish();
    pub fn qglColor3f(r: c_float, g: c_float, b: c_float);
    pub fn qglTexImage2D(
        target: c_int,
        level: c_int,
        internalFormat: c_int,
        width: c_int,
        height: c_int,
        border: c_int,
        format: c_int,
        typ: c_int,
        pixels: *const c_void,
    );
    pub fn qglTexParameterf(target: c_int, pname: c_int, param: c_float);
    pub fn qglTexSubImage2D(
        target: c_int,
        level: c_int,
        xoffset: c_int,
        yoffset: c_int,
        width: c_int,
        height: c_int,
        format: c_int,
        typ: c_int,
        pixels: *const c_void,
    );
    pub fn qglBegin(mode: c_int);
    pub fn qglEnd();
    pub fn qglTexCoord2f(u: c_float, v: c_float);
    pub fn qglVertex2f(x: c_float, y: c_float);
    pub fn qglBeginEXT(mode: c_int, count: c_int, a: c_int, b: c_int, c: c_int, d: c_int);
    pub fn qglReadPixels(
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        format: c_int,
        typ: c_int,
        pixels: *mut c_void,
    );
    pub fn qglClearDepth(depth: c_float);
    pub fn qglClear(mask: c_int);
    pub fn qglCopyBackBufferToTexEXT(w: c_float, h: c_float, a: c_float, b: c_float, c: c_float, d: c_float);

    // libc functions
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// ========== Global Variables ==========

// this is so the server (or anyone else) can get access to raw pixels if they really need to,
//	currently it's only used by the server so that savegames can embed a graphic in the auto-save files
//	(which can't do a screenshot since they're saved out before the level is drawn).
static mut pbLoadedPic: *mut byte = core::ptr::null_mut();

static mut Dissolve: Dissolve_t = Dissolve_t {
    iWidth: 0,
    iHeight: 0,
    iUploadWidth: 0,
    iUploadHeight: 0,
    iScratchPadNumber: 0,
    pImage: core::ptr::null_mut(),
    pDissolve: core::ptr::null_mut(),
    pBlack: core::ptr::null_mut(),
    iStartTime: 0,
    eDissolveType: Dissolve_e::eDISSOLVE_RT_TO_LT,
    bTouchNeeded: qfalse,
};

const fDISSOLVE_SECONDS: c_float = 0.75;

// ========== Function Implementations ==========

/*
=============
RE_StretchRaw

Stretches a raw 32 bit power of 2 bitmap image over the given screen rectangle.
Used for cinematics.
=============
*/

// param 'bDirty' should be true 99% of the time
pub unsafe extern "C" fn RE_StretchRaw(
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
    cols: c_int,
    rows: c_int,
    data: *const byte,
    iClient: c_int,
    bDirty: qboolean,
) {
    R_SyncRenderThread();

    //===========
    // Q3Final added this:
    // we definately want to sync every frame for the cinematics
    //qglFinish();

    #[cfg(feature = "timebind")]
    {
        let mut start: c_int = 0;
        let mut end: c_int = 0;
        // only to stop compiler whining, don't need to be initialised
    }

    // make sure rows and cols are powers of 2
    if (cols & (cols - 1)) != 0 || (rows & (rows - 1)) != 0 {
        Com_Error(
            ERR_DROP,
            b"Draw_StretchRaw: size not a power of 2: %i by %i\0".as_ptr() as *const c_char,
            cols,
            rows,
        );
    }

    GL_Bind((*addr_of_mut!(tr)).scratchImage[iClient as usize]);

    // if the scratchImage isn't in the format we want, specify it as a new texture...
    //
    let scratch_img = (*addr_of_mut!(tr)).scratchImage[iClient as usize];
    if cols != (*scratch_img).width || rows != (*scratch_img).height {
        (*scratch_img).width = cols;
        (*scratch_img).height = rows;

        #[cfg(feature = "timebind")]
        {
            if (*addr_of_mut!(r_ignore)).integer != 0 {
                let start = Sys_Milliseconds();
            }
        }

        qglTexImage2D(GL_TEXTURE_2D, 0, GL_RGB8, cols, rows, 0, GL_RGBA, GL_UNSIGNED_BYTE, data as *const c_void);

        qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as c_float);
        qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as c_float);
        qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP as c_float);
        qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP as c_float);

        #[cfg(feature = "timebind")]
        {
            if (*addr_of_mut!(r_ignore)).integer != 0 {
                let end = Sys_Milliseconds();
                VID_Printf(
                    PRINT_ALL,
                    b"qglTexImage2D %i, %i: %i msec\n\0".as_ptr() as *const c_char,
                    cols,
                    rows,
                    end,
                );
            }
        }
    } else {
        if bDirty != qfalse {
            // FIXME: some TA addition or other, not sure why, yet. Should probably be true 99% of the time?
            // otherwise, just subimage upload it so that drivers can tell we are going to be changing
            // it and don't try and do a texture compression

            #[cfg(feature = "timebind")]
            {
                if (*addr_of_mut!(r_ignore)).integer != 0 {
                    let start = Sys_Milliseconds();
                }
            }

            qglTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, cols, rows, GL_RGBA, GL_UNSIGNED_BYTE, data as *const c_void);

            #[cfg(feature = "timebind")]
            {
                if (*addr_of_mut!(r_ignore)).integer != 0 {
                    let end = Sys_Milliseconds();
                    VID_Printf(
                        PRINT_ALL,
                        b"qglTexSubImage2D %i, %i: %i msec\n\0".as_ptr() as *const c_char,
                        cols,
                        rows,
                        end,
                    );
                }
            }
        }
    }

    extern "C" {
        fn RB_SetGL2D();
    }
    if (*addr_of_mut!(backEnd)).projection2D == qfalse {
        RB_SetGL2D();
    }
    qglColor3f((*addr_of_mut!(tr)).identityLight, (*addr_of_mut!(tr)).identityLight, (*addr_of_mut!(tr)).identityLight);

    #[cfg(feature = "xbox")]
    {
        qglBeginEXT(GL_TRIANGLE_STRIP, 4, 0, 0, 4, 0);
        qglTexCoord2f(0.5 / cols as c_float, 0.5 / rows as c_float);
        qglVertex2f(x as c_float, y as c_float);
        qglTexCoord2f(((cols - 1) as c_float - 0.5) / cols as c_float, 0.5 / rows as c_float);
        qglVertex2f((x + w) as c_float, y as c_float);
        qglTexCoord2f(0.5 / cols as c_float, ((rows - 1) as c_float - 0.5) / rows as c_float);
        qglVertex2f(x as c_float, (y + h) as c_float);
        qglTexCoord2f(
            ((cols - 1) as c_float - 0.5) / cols as c_float,
            ((rows - 1) as c_float - 0.5) / rows as c_float,
        );
        qglVertex2f((x + w) as c_float, (y + h) as c_float);
        qglEnd();
    }
    #[cfg(not(feature = "xbox"))]
    {
        qglBegin(GL_QUADS);
        qglTexCoord2f(0.5 / cols as c_float, 0.5 / rows as c_float);
        qglVertex2f(x as c_float, y as c_float);
        qglTexCoord2f(((cols - 1) as c_float - 0.5) / cols as c_float, 0.5 / rows as c_float);
        qglVertex2f((x + w) as c_float, y as c_float);
        qglTexCoord2f(
            ((cols - 1) as c_float - 0.5) / cols as c_float,
            ((rows - 1) as c_float - 0.5) / rows as c_float,
        );
        qglVertex2f((x + w) as c_float, (y + h) as c_float);
        qglTexCoord2f(0.5 / cols as c_float, ((rows - 1) as c_float - 0.5) / rows as c_float);
        qglVertex2f(x as c_float, (y + h) as c_float);
        qglEnd();
    }
}

pub unsafe extern "C" fn RE_UploadCinematic(cols: c_int, rows: c_int, data: *const byte, client: c_int, dirty: qboolean) {
    GL_Bind((*addr_of_mut!(tr)).scratchImage[client as usize]);

    // if the scratchImage isn't in the format we want, specify it as a new texture
    let scratch_img = (*addr_of_mut!(tr)).scratchImage[client as usize];
    if cols != (*scratch_img).width || rows != (*scratch_img).height {
        (*scratch_img).width = cols;
        (*scratch_img).height = rows;

        #[cfg(feature = "xbox")]
        {
            qglTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, cols, rows, 0, GL_LIN_RGBA, GL_UNSIGNED_BYTE, data as *const c_void);
        }
        #[cfg(not(feature = "xbox"))]
        {
            qglTexImage2D(GL_TEXTURE_2D, 0, GL_RGB8, cols, rows, 0, GL_RGBA, GL_UNSIGNED_BYTE, data as *const c_void);
        }

        qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as c_float);
        qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as c_float);
        qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP as c_float);
        qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP as c_float);
    } else {
        if dirty != qfalse {
            // otherwise, just subimage upload it so that drivers can tell we are going to be changing
            // it and don't try and do a texture compression
            qglTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, cols, rows, GL_RGBA, GL_UNSIGNED_BYTE, data as *const c_void);
        }
    }
}

#[cfg(feature = "disabled")]
pub unsafe extern "C" fn RE_GetScreenShot(data: *mut byte, w: c_int, h: c_int) {
    let mut buffer: *mut byte = (R_Malloc(((*addr_of_mut!(glConfig)).vidWidth * (*addr_of_mut!(glConfig)).vidHeight * 3) as usize) as *mut byte);
    if buffer.is_null() {
        return;
    }
    qglReadPixels(
        0,
        0,
        (*addr_of_mut!(glConfig)).vidWidth,
        (*addr_of_mut!(glConfig)).vidHeight,
        GL_RGB,
        GL_UNSIGNED_BYTE,
        buffer as *mut c_void,
    );

    let xstep: c_int = ((*addr_of_mut!(glConfig)).vidWidth << 16) / w;
    let ystep: c_int = ((*addr_of_mut!(glConfig)).vidHeight << 16) / h;
    let mut yc: c_int = 0;
    let mut y: c_int = 0;
    while y < h {
        let mut xc: c_int = 0;
        let mut x: c_int = 0;
        while x < w {
            let offset: c_int = (((*addr_of_mut!(glConfig)).vidWidth * (yc >> 16)) + (xc >> 16)) * 3;
            *data.offset(0) = *buffer.offset(offset as isize);
            *data.offset(1) = *buffer.offset((offset + 1) as isize);
            *data.offset(2) = *buffer.offset((offset + 2) as isize);
            *data.offset(3) = 0xff;
            x += 1;
            xc += xstep;
        }
        y += 1;
        yc += ystep;
    }
    R_Free(buffer as *mut c_void);
}

pub unsafe extern "C" fn RE_GetScreenShot(buffer: *mut byte, w: c_int, h: c_int) {
    #[cfg(not(feature = "xbox"))]
    {
        let mut source: *mut byte;
        let mut src: *mut byte;
        let mut dst: *mut byte;
        let mut x: c_int;
        let mut y: c_int;
        let mut r: c_int;
        let mut g: c_int;
        let mut b: c_int;
        let mut xScale: c_float;
        let mut yScale: c_float;
        let mut xx: c_int;
        let mut yy: c_int;

        qglFinish(); // try and fix broken Radeon cards (7500 & 8500) that don't read screen pixels properly

        source = Z_Malloc(
            (*addr_of_mut!(glConfig)).vidWidth * (*addr_of_mut!(glConfig)).vidHeight * 3,
            TAG_TEMP_WORKSPACE,
            qfalse,
        ) as *mut byte;
        if source.is_null() {
            return;
        }
        qglReadPixels(
            0,
            0,
            (*addr_of_mut!(glConfig)).vidWidth,
            (*addr_of_mut!(glConfig)).vidHeight,
            GL_RGB,
            GL_UNSIGNED_BYTE,
            source as *mut c_void,
        );

        let mut count: c_int = 0;
        // resample from source
        xScale = (*addr_of_mut!(glConfig)).vidWidth as c_float / (4.0 * w as c_float);
        yScale = (*addr_of_mut!(glConfig)).vidHeight as c_float / (3.0 * w as c_float);
        y = 0;
        while y < w {
            x = 0;
            while x < w {
                r = 0;
                g = 0;
                b = 0;
                yy = 0;
                while yy < 3 {
                    xx = 0;
                    while xx < 4 {
                        src = source
                            .add((3 * ((*addr_of_mut!(glConfig)).vidWidth * ((y * 3 + yy) as c_float * yScale) as c_int
                                + ((x * 4 + xx) as c_float * xScale) as c_int)) as usize);
                        r += *src as c_int;
                        g += *src.add(1) as c_int;
                        b += *src.add(2) as c_int;
                        xx += 1;
                    }
                    yy += 1;
                }
                dst = buffer.add((4 * (y * w + x)) as usize);
                *dst = (r / 12) as u8;
                *dst.add(1) = (g / 12) as u8;
                *dst.add(2) = (b / 12) as u8;
                count += 1;
                x += 1;
            }
            y += 1;
        }

        Z_Free(source as *mut c_void);
    }
}

// this is just a chunk of code from RE_TempRawImage_ReadFromFile() below, subroutinised so I can call it
//	from the screen dissolve code as well...
//
unsafe fn RE_ReSample(
    pbLoadedPic: *mut byte,
    iLoadedWidth: c_int,
    iLoadedHeight: c_int,
    pbReSampleBuffer: *mut byte,
    piWidth: *mut c_int,
    piHeight: *mut c_int,
) -> *mut byte {
    let mut pbReturn: *mut byte = core::ptr::null_mut();

    // if not resampling, just return some values and return...
    //
    if pbReSampleBuffer.is_null() || (iLoadedWidth == *piWidth && iLoadedHeight == *piHeight) {
        // if not resampling, we're done, just return the loaded size...
        //
        *piWidth = iLoadedWidth;
        *piHeight = iLoadedHeight;
        pbReturn = pbLoadedPic;
    } else {
        // resample from pbLoadedPic to pbReSampledBuffer...
        //
        let fXStep: c_float = iLoadedWidth as c_float / *piWidth as c_float;
        let fYStep: c_float = iLoadedHeight as c_float / *piHeight as c_float;
        let iTotPixelsPerDownSample: c_int = ((fXStep.ceil() as c_int) * (fYStep.ceil() as c_int));

        let mut r: c_int;
        let mut g: c_int;
        let mut b: c_int;

        let mut pbDst: *mut byte = pbReSampleBuffer;

        let mut y: c_int = 0;
        while y < *piHeight {
            let mut x: c_int = 0;
            while x < *piWidth {
                r = 0;
                g = 0;
                b = 0;

                let mut yy: c_float = (y as c_float) * fYStep;
                while yy < ((y + 1) as c_float) * fYStep {
                    let mut xx: c_float = (x as c_float) * fXStep;
                    while xx < ((x + 1) as c_float) * fXStep {
                        let pbSrc: *mut byte = pbLoadedPic.add(
                            (4 * (((yy as c_int) * iLoadedWidth) + (xx as c_int))) as usize,
                        );

                        r += *pbSrc as c_int;
                        g += *pbSrc.add(1) as c_int;
                        b += *pbSrc.add(2) as c_int;

                        xx += 1.0;
                    }
                    yy += 1.0;
                }

                *pbDst = (r / iTotPixelsPerDownSample) as u8;
                *pbDst.add(1) = (g / iTotPixelsPerDownSample) as u8;
                *pbDst.add(2) = (b / iTotPixelsPerDownSample) as u8;
                *pbDst.add(3) = 255;
                pbDst = pbDst.add(4);

                x += 1;
            }
            y += 1;
        }

        // set return value...
        //
        pbReturn = pbReSampleBuffer;
    }

    pbReturn
}

// this is so the server (or anyone else) can get access to raw pixels if they really need to,
//	currently it's only used by the server so that savegames can embed a graphic in the auto-save files
//	(which can't do a screenshot since they're saved out before the level is drawn).
//
// by default, the pic will be returned as the original dims, but if pbReSampleBuffer != NULL then it's assumed to
//	be a big enough buffer to hold the resampled image, which also means that the width and height params are read as
//	inputs (as well as still being inherently outputs) and the pic is scaled to that size, and to that buffer.
//
// the return value is either NULL, or a pointer to the pixels to use (which may be either the pbReSampleBuffer param,
//	or the local ptr below).
//
// In either case, you MUST call the free-up function afterwards ( RE_TempRawImage_CleanUp() ) to get rid of any temp
//	memory after you've finished with the pic.
//
// Note: ALWAYS use the return value if != NULL, even if you passed in a declared resample buffer. This is because the
//	resample will get skipped if the values you want are the same size as the pic that it loaded, so it'll return a
//	different buffer.
//
// the vertflip param is used for those functions that expect things in OpenGL's upside-down pixel-read format (sigh)
//
// (not brilliantly fast, but it's only used for weird stuff anyway)
//

#[cfg(not(feature = "xbox"))]
pub unsafe extern "C" fn RE_TempRawImage_ReadFromFile(
    psLocalFilename: *const c_char,
    piWidth: *mut c_int,
    piHeight: *mut c_int,
    pbReSampleBuffer: *mut byte,
    qbVertFlip: qboolean,
) -> *mut byte {
    RE_TempRawImage_CleanUp(); // jic

    let mut pbReturn: *mut byte = core::ptr::null_mut();

    if !psLocalFilename.is_null() && !piWidth.is_null() && !piHeight.is_null() {
        let mut iLoadedWidth: c_int = 0;
        let mut iLoadedHeight: c_int = 0;

        let mut format: c_int = 0;
        R_LoadImage(
            psLocalFilename,
            addr_of_mut!(pbLoadedPic),
            &mut iLoadedWidth,
            &mut iLoadedHeight,
            &mut format,
        );
        if !pbLoadedPic.is_null() {
            pbReturn = RE_ReSample(pbLoadedPic, iLoadedWidth, iLoadedHeight, pbReSampleBuffer, piWidth, piHeight);
        }
    }

    if !pbReturn.is_null() && qbVertFlip != qfalse {
        let mut pSrcLine: *mut u32 = pbReturn as *mut u32;
        let mut pDstLine: *mut u32 = (pbReturn as *mut u32).add((*piHeight * *piWidth) as usize);
        pDstLine = pDstLine.sub(*piWidth as usize); // point at start of last line, not first after buffer

        let mut iLineCount: c_int = 0;
        while iLineCount < *piHeight / 2 {
            let mut x: c_int = 0;
            while x < *piWidth {
                let l: u32 = *pSrcLine.add(x as usize);
                *pSrcLine.add(x as usize) = *pDstLine.add(x as usize);
                *pDstLine.add(x as usize) = l;
                x += 1;
            }
            pSrcLine = pSrcLine.add(*piWidth as usize);
            pDstLine = pDstLine.sub(*piWidth as usize);
            iLineCount += 1;
        }
    }

    pbReturn
}

pub unsafe extern "C" fn RE_TempRawImage_CleanUp() {
    if !pbLoadedPic.is_null() {
        Z_Free(pbLoadedPic as *mut c_void);
        pbLoadedPic = core::ptr::null_mut();
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dissolve_e_enum {
    eDISSOLVE_RT_TO_LT = 0,
    eDISSOLVE_LT_TO_RT = 1,
    eDISSOLVE_TP_TO_BT = 2,
    eDISSOLVE_BT_TO_TP = 3,
    eDISSOLVE_CIRCULAR_OUT = 4, // new image comes out from centre
    //
    eDISSOLVE_RAND_LIMIT = 5,   // label only, not valid to select
    //
    // any others...
    //
    eDISSOLVE_CIRCULAR_IN = 6,  // new image comes in from edges
    //
    eDISSOLVE_NUMBEROF = 7,
}

#[inline]
unsafe fn PowerOf2(iArg: c_int) -> c_int {
    if (iArg & (iArg - 1)) != 0 {
        let mut iShift: c_int = 0;
        let mut arg = iArg;
        while arg != 0 {
            arg >>= 1;
            iShift += 1;
        }
        return 1 << iShift;
    }
    iArg
}

// leave the UV stuff in for now as comments in case I ever need to do some sneaky stuff, but for now...
//
unsafe fn RE_Blit(
    fX0: c_float,
    fY0: c_float,
    fX1: c_float,
    fY1: c_float,
    fX2: c_float,
    fY2: c_float,
    fX3: c_float,
    fY3: c_float,
    //float fU0, float fV0, float fU1, float fV1, float fU2, float fV2, float fU3, float fV3,
    pImage: *mut image_t,
    iGLState: c_int,
) {
    //
    // some junk they had at the top of other StretchRaw code...
    //
    R_SyncRenderThread();
    //#ifndef _XBOX
    //	qglFinish();
    //#endif // _XBOX

    GL_Bind(pImage);
    GL_State(iGLState);
    GL_Cull(CT_TWO_SIDED);

    qglColor3f(1.0, 1.0, 1.0);

    #[cfg(feature = "xbox")]
    {
        qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
    }
    #[cfg(not(feature = "xbox"))]
    {
        qglBegin(GL_QUADS);
    }

    // TL...
    //
    //		qglTexCoord2f( fU0 / (float)pImage->width,  fV0 / (float)pImage->height );
    qglTexCoord2f(0.0, 0.0);
    qglVertex2f(fX0, fY0);

    // TR...
    //
    //		qglTexCoord2f( fU1 / (float)pImage->width,  fV1 / (float)pImage->height );
    qglTexCoord2f(1.0, 0.0);
    qglVertex2f(fX1, fY1);

    // BR...
    //
    //		qglTexCoord2f( fU2 / (float)pImage->width,  fV2 / (float)pImage->height );
    qglTexCoord2f(1.0, 1.0);
    qglVertex2f(fX2, fY2);

    // BL...
    //
    //		qglTexCoord2f( fU3 / (float)pImage->width,  fV3 / (float)pImage->height );
    qglTexCoord2f(0.0, 1.0);
    qglVertex2f(fX3, fY3);

    qglEnd();
}

unsafe fn RE_KillDissolve() {
    (*addr_of_mut!(Dissolve)).iStartTime = 0;

    if !(*addr_of_mut!(Dissolve)).pImage.is_null() {
        R_Images_DeleteImage((*addr_of_mut!(Dissolve)).pImage);
        (*addr_of_mut!(Dissolve)).pImage = core::ptr::null_mut();
    }
}

// Draw the dissolve pic to the screen, over the top of what's already been rendered.
//
// return = qtrue while still processing, for those interested...
//
const iSAFETY_SPRITE_OVERLAP: c_int = 2; // #pixels to overlap blit region by, in case some drivers leave onscreen seams

pub unsafe extern "C" fn RE_ProcessDissolve() -> qboolean {
    if (*addr_of_mut!(Dissolve)).iStartTime != 0 {
        if (*addr_of_mut!(Dissolve)).bTouchNeeded != qfalse {
            // Stuff to avoid music stutter...
            //
            //	The problem is, that if I call RE_InitDissolve() then call RestartMusic, then by the time the music
            //	has loaded in if it took longer than one second the dissolve would think that it had finished,
            //	even if it had never actually drawn up. However, if I called RE_InitDissolve() AFTER the music had
            //	restarted, then the music would stutter on slow video cards or CPUs while I did the binding/resampling.
            //
            // This way, I restart the millisecond counter the first time we actually get as far as rendering, which
            //	should let things work properly...
            //
            (*addr_of_mut!(Dissolve)).bTouchNeeded = qfalse;
            (*addr_of_mut!(Dissolve)).iStartTime = Sys_Milliseconds();
        }

        let iDissolvePercentage: c_int =
            ((Sys_Milliseconds() - (*addr_of_mut!(Dissolve)).iStartTime) * 100) / (1000 * fDISSOLVE_SECONDS as c_int);

        //		VID_Printf(PRINT_ALL,"iDissolvePercentage %d\n",iDissolvePercentage);

        if iDissolvePercentage <= 100 {
            extern "C" {
                fn RB_SetGL2D();
            }
            RB_SetGL2D();

            //			GLdouble glD;
            //			qglGetDoublev(GL_DEPTH_CLEAR_VALUE,&glD);
            //			qglClearColor(0,0,0,1);
            qglClearDepth(1.0);
            qglClear(GL_DEPTH_BUFFER_BIT);

            let fXScaleFactor: c_float = (SCREEN_WIDTH as c_float) / ((*addr_of_mut!(Dissolve)).iWidth as c_float);
            let fYScaleFactor: c_float = (SCREEN_HEIGHT as c_float) / ((*addr_of_mut!(Dissolve)).iHeight as c_float);
            let mut x0: c_float;
            let mut y0: c_float;
            let mut x1: c_float;
            let mut y1: c_float;
            let mut x2: c_float;
            let mut y2: c_float;
            let mut x3: c_float;
            let mut y3: c_float;

            match (*addr_of_mut!(Dissolve)).eDissolveType {
                Dissolve_e::eDISSOLVE_RT_TO_LT => {
                    let fXboundary: c_float = ((*addr_of_mut!(Dissolve)).iWidth as c_float)
                        - ((((*addr_of_mut!(Dissolve)).iWidth + (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width) as c_float
                            * iDissolvePercentage as c_float)
                            / 100.0);

                    // blit the fuzzy-dissolve sprite...
                    //
                    x0 = fXScaleFactor * fXboundary;
                    y0 = 0.0;
                    x1 = fXScaleFactor * (fXboundary + (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width as c_float);
                    y1 = 0.0;
                    x2 = x1;
                    y2 = fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float;
                    x3 = x0;
                    y3 = y2;

                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pDissolve,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE | GLS_ATEST_LT_80,
                    );

                    // blit a blank thing over the area the old screen is to be displayed on to enable screen-writing...
                    // (to the left of fXboundary)
                    //
                    x0 = 0.0;
                    y0 = 0.0;
                    x1 = fXScaleFactor * (fXboundary + iSAFETY_SPRITE_OVERLAP as c_float);
                    y1 = 0.0;
                    x2 = x1;
                    y2 = fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float;
                    x3 = x0;
                    y3 = y2;
                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pBlack,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE,
                    );
                }

                Dissolve_e::eDISSOLVE_LT_TO_RT => {
                    let fXboundary: c_float = ((((*addr_of_mut!(Dissolve)).iWidth + (2 * (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width)) as c_float
                        * iDissolvePercentage as c_float)
                        / 100.0)
                        - (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width as c_float;

                    // blit the fuzzy-dissolve sprite...
                    //
                    x0 = fXScaleFactor * (fXboundary + (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width as c_float);
                    y0 = 0.0;
                    x1 = fXScaleFactor * fXboundary;
                    y1 = 0.0;
                    x2 = x1;
                    y2 = fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float;
                    x3 = x0;
                    y3 = y2;

                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pDissolve,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE | GLS_ATEST_LT_80,
                    );

                    // blit a blank thing over the area the old screen is to be displayed on to enable screen-writing...
                    // (to the right of fXboundary)
                    //
                    x0 = fXScaleFactor
                        * ((fXboundary + (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width as c_float)
                            - iSAFETY_SPRITE_OVERLAP as c_float);
                    y0 = 0.0;
                    x1 = fXScaleFactor * (*addr_of_mut!(Dissolve)).iWidth as c_float;
                    y0 = 0.0;
                    x2 = x1;
                    y2 = fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float;
                    x3 = x0;
                    y3 = y2;
                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pBlack,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE,
                    );
                }

                Dissolve_e::eDISSOLVE_TP_TO_BT => {
                    let fYboundary: c_float = ((((*addr_of_mut!(Dissolve)).iHeight
                        + (2 * (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width)) as c_float
                        * iDissolvePercentage as c_float)
                        / 100.0)
                        - (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width as c_float;

                    // blit the fuzzy-dissolve sprite...
                    //
                    x0 = 0.0;
                    y0 = fYScaleFactor * (fYboundary + (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width as c_float);
                    x1 = x0;
                    y1 = fYScaleFactor * fYboundary;
                    x2 = fXScaleFactor * (*addr_of_mut!(Dissolve)).iWidth as c_float;
                    y2 = y1;
                    x3 = x2;
                    y3 = y0;

                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pDissolve,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE | GLS_ATEST_LT_80,
                    );

                    // blit a blank thing over the area the old screen is to be displayed on to enable screen-writing...
                    // (underneath fYboundary)
                    //
                    x0 = 0.0;
                    y0 = fYScaleFactor * ((fYboundary + (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width as c_float)
                        - iSAFETY_SPRITE_OVERLAP as c_float);
                    x1 = fXScaleFactor * (*addr_of_mut!(Dissolve)).iWidth as c_float;
                    y1 = y0;
                    x2 = x1;
                    y2 = fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float;
                    x3 = x0;
                    y3 = y2;
                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pBlack,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE,
                    );
                }

                Dissolve_e::eDISSOLVE_BT_TO_TP => {
                    let fYboundary: c_float = ((*addr_of_mut!(Dissolve)).iHeight as c_float)
                        - ((((*addr_of_mut!(Dissolve)).iHeight + (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width) as c_float
                            * iDissolvePercentage as c_float)
                            / 100.0);

                    // blit the fuzzy-dissolve sprite...
                    //
                    x0 = 0.0;
                    y0 = fYScaleFactor * fYboundary;
                    x1 = x0;
                    y1 = fYScaleFactor * (fYboundary + (*addr_of_mut!(Dissolve)).pDissolve.as_ref().unwrap().width as c_float);
                    x2 = fXScaleFactor * (*addr_of_mut!(Dissolve)).iWidth as c_float;
                    y2 = y1;
                    x3 = x2;
                    y3 = y0;

                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pDissolve,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE | GLS_ATEST_LT_80,
                    );

                    // blit a blank thing over the area the old screen is to be displayed on to enable screen-writing...
                    // (above fYboundary)
                    //
                    x0 = 0.0;
                    y0 = 0.0;
                    x1 = fXScaleFactor * (*addr_of_mut!(Dissolve)).iWidth as c_float;
                    y1 = y0;
                    x2 = x1;
                    y2 = fYScaleFactor * (fYboundary + iSAFETY_SPRITE_OVERLAP as c_float);
                    x3 = x0;
                    y3 = y2;
                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pBlack,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE,
                    );
                }

                Dissolve_e::eDISSOLVE_CIRCULAR_IN => {
                    let fDiagZoom: c_float = ((((*addr_of_mut!(Dissolve)).iWidth as c_float * 0.8)
                        * (100 - iDissolvePercentage) as c_float)
                        / 100.0);

                    //
                    // blit circular graphic...
                    //
                    x0 = fXScaleFactor * ((((*addr_of_mut!(Dissolve)).iWidth / 2) as c_float) - fDiagZoom);
                    y0 = fYScaleFactor * ((((*addr_of_mut!(Dissolve)).iHeight / 2) as c_float) - fDiagZoom);
                    x1 = fXScaleFactor * ((((*addr_of_mut!(Dissolve)).iWidth / 2) as c_float) + fDiagZoom);
                    y1 = y0;
                    x2 = x1;
                    y2 = fYScaleFactor * ((((*addr_of_mut!(Dissolve)).iHeight / 2) as c_float) + fDiagZoom);
                    x3 = x0;
                    y3 = y2;

                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pDissolve,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE | GLS_ATEST_LT_80,
                    );
                }

                Dissolve_e::eDISSOLVE_CIRCULAR_OUT => {
                    let fDiagZoom: c_float = ((((*addr_of_mut!(Dissolve)).iWidth as c_float * 0.8) * iDissolvePercentage as c_float) / 100.0);

                    //
                    // blit circular graphic...
                    //
                    x0 = fXScaleFactor * ((((*addr_of_mut!(Dissolve)).iWidth / 2) as c_float) - fDiagZoom);
                    y0 = fYScaleFactor * ((((*addr_of_mut!(Dissolve)).iHeight / 2) as c_float) - fDiagZoom);
                    x1 = fXScaleFactor * ((((*addr_of_mut!(Dissolve)).iWidth / 2) as c_float) + fDiagZoom);
                    y1 = y0;
                    x2 = x1;
                    y2 = fYScaleFactor * ((((*addr_of_mut!(Dissolve)).iHeight / 2) as c_float) + fDiagZoom);
                    x3 = x0;
                    y3 = y2;

                    RE_Blit(
                        x0, y0, x1, y1, x2, y2, x3, y3,
                        (*addr_of_mut!(Dissolve)).pDissolve,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE | GLS_ATEST_LT_80,
                    );
                    // now blit the 4 black squares around it to mask off the rest of the screen...
                    //
                    // LHS, top to bottom...
                    //
                    RE_Blit(
                        0.0,
                        0.0, // x0,y0
                        x0 + iSAFETY_SPRITE_OVERLAP as c_float,
                        0.0, // x1,y1
                        x0 + iSAFETY_SPRITE_OVERLAP as c_float,
                        (fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float), // x2,y2
                        0.0,
                        (fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float), // x3,y3,
                        (*addr_of_mut!(Dissolve)).pBlack,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE,
                    );

                    // RHS top to bottom...
                    //
                    RE_Blit(
                        x1 - iSAFETY_SPRITE_OVERLAP as c_float,
                        0.0, // x0,y0
                        (fXScaleFactor * (*addr_of_mut!(Dissolve)).iWidth as c_float),
                        0.0, // x1,y1
                        (fXScaleFactor * (*addr_of_mut!(Dissolve)).iWidth as c_float),
                        (fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float), // x2,y2
                        x1 - iSAFETY_SPRITE_OVERLAP as c_float,
                        (fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float), // x3,y3,
                        (*addr_of_mut!(Dissolve)).pBlack,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE,
                    );

                    // top...
                    //
                    RE_Blit(
                        x0 - iSAFETY_SPRITE_OVERLAP as c_float,
                        0.0, // x0,y0
                        x1 + iSAFETY_SPRITE_OVERLAP as c_float,
                        0.0, // x1,y1
                        x1 + iSAFETY_SPRITE_OVERLAP as c_float,
                        y0 + iSAFETY_SPRITE_OVERLAP as c_float, // x2,y2
                        x0 - iSAFETY_SPRITE_OVERLAP as c_float,
                        y0 + iSAFETY_SPRITE_OVERLAP as c_float, // x3,y3
                        (*addr_of_mut!(Dissolve)).pBlack,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE,
                    );

                    // bottom...
                    //
                    RE_Blit(
                        x0 - iSAFETY_SPRITE_OVERLAP as c_float,
                        y3 - iSAFETY_SPRITE_OVERLAP as c_float, // x0,y0
                        x1 + iSAFETY_SPRITE_OVERLAP as c_float,
                        y2 - iSAFETY_SPRITE_OVERLAP as c_float, // x1,y1
                        x1 + iSAFETY_SPRITE_OVERLAP as c_float,
                        (fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float), // x2,y2
                        x0 - iSAFETY_SPRITE_OVERLAP as c_float,
                        (fYScaleFactor * (*addr_of_mut!(Dissolve)).iHeight as c_float), // x3,y3
                        (*addr_of_mut!(Dissolve)).pBlack,
                        GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ZERO | GLS_DSTBLEND_ONE,
                    );
                }

                _ => {
                    // assert(0);
                    // iDissolvePercentage = 101;	// force a dissolve-kill
                }
            }

            // re-check in case we hit the default case above...
            //
            if iDissolvePercentage <= 100 {
                // still dissolving, so now (finally), blit old image over top...
                //
                x0 = 0.0;
                y0 = 0.0;
                #[cfg(feature = "xbox")]
                {
                    x1 = 640.0;
                }
                #[cfg(not(feature = "xbox"))]
                {
                    x1 = fXScaleFactor * (*(*addr_of_mut!(Dissolve)).pImage).width as c_float;
                }
                y1 = y0;
                x2 = x1;
                #[cfg(feature = "xbox")]
                {
                    y2 = 480.0;
                }
                #[cfg(not(feature = "xbox"))]
                {
                    y2 = fYScaleFactor * (*(*addr_of_mut!(Dissolve)).pImage).height as c_float;
                }
                x3 = x0;
                y3 = y2;

                RE_Blit(x0, y0, x1, y1, x2, y2, x3, y3, (*addr_of_mut!(Dissolve)).pImage, GLS_DEPTHFUNC_EQUAL);
            }
        }

        if iDissolvePercentage > 100 {
            RE_KillDissolve();
        }
    }

    qfalse
}

#[cfg(feature = "xbox")]
unsafe fn RE_GetCompressedBackbuffer() {
    let data: *mut byte = Z_Malloc(32768, TAG_TEMP_WORKSPACE, qtrue) as *mut byte;

    (*addr_of_mut!(Dissolve)).pImage = R_CreateImage(
        b"*DissolveImage\0".as_ptr() as *const c_char, // const char *name
        data,                                           // const byte *pic
        256,                                            // int width
        256,                                            // int height
        GL_COMPRESSED_RGB_S3TC_DXT1_EXT,
        qfalse,         // qboolean mipmap
        qfalse,         // qboolean allowPicmip
        GL_CLAMP,       // int glWrapClampMode
    );
    Z_Free(data as *mut c_void);

    GL_Bind((*addr_of_mut!(Dissolve)).pImage);

    qglCopyBackBufferToTexEXT(256.0, 256.0, 0.0, 0.0, 640.0, 480.0);
}

// return = qtrue(success) else fail, for those interested...
//
pub unsafe extern "C" fn RE_InitDissolve(bForceCircularExtroWipe: qboolean) -> qboolean {
    R_SyncRenderThread();

    //	VID_Printf( PRINT_ALL, "RE_InitDissolve()\n");
    let mut bReturn: qboolean = qfalse;

    if //Dissolve.iStartTime == 0	// no point in interruping an existing one
    //&&
    (*addr_of_mut!(tr)).registered == qtrue
    // ... stops it crashing during first cinematic before the menus... :-)
    {
        RE_KillDissolve(); // kill any that are already running

        #[cfg(feature = "xbox")]
        {
            // Things are much simpler on Xbox, and use far less RAM

            if true {
                // Silly if(1) to match up with control flow below, as the #ifdef ends inside the block
                RE_GetCompressedBackbuffer();
                (*addr_of_mut!(Dissolve)).iWidth = (*addr_of_mut!(glConfig)).vidWidth;
                (*addr_of_mut!(Dissolve)).iHeight = (*addr_of_mut!(glConfig)).vidHeight;
            }
        }
        #[cfg(not(feature = "xbox"))]
        {
            let iPow2VidWidth: c_int = PowerOf2((*addr_of_mut!(glConfig)).vidWidth);
            let iPow2VidHeight: c_int = PowerOf2((*addr_of_mut!(glConfig)).vidHeight);

            let iBufferBytes: c_int = iPow2VidWidth * iPow2VidHeight * 4;
            let pBuffer: *mut byte = Z_Malloc(iBufferBytes, TAG_TEMP_WORKSPACE, qfalse) as *mut byte;
            if !pBuffer.is_null() {
                // read current screen image...  (GL_RGBA should work even on 3DFX in that the RGB parts will be valid at least)
                //
                qglReadPixels(
                    0,
                    0,
                    (*addr_of_mut!(glConfig)).vidWidth,
                    (*addr_of_mut!(glConfig)).vidHeight,
                    GL_RGBA,
                    GL_UNSIGNED_BYTE,
                    pBuffer as *mut c_void,
                );
                //
                // now expand the pic over the top of itself so that it has a stride value of {PowerOf2(glConfig.vidWidth)}
                //	(for GL power-of-2 rules)
                //
                let pbSrc: *mut byte =
                    pBuffer.add(((*addr_of_mut!(glConfig)).vidWidth * (*addr_of_mut!(glConfig)).vidHeight * 4) as usize);
                let pbDst: *mut byte = pBuffer.add((iPow2VidWidth * (*addr_of_mut!(glConfig)).vidHeight * 4) as usize);
                //
                // ( clear to end, since we've got pbDst nicely setup here)
                //
                let iClearBytes: usize = (pBuffer.add(iBufferBytes as usize) as usize) - (pbDst as usize);
                memset(pbDst as *mut c_void, 0, iClearBytes);
                //
                // work out copy/stride vals...
                //
                let iClearBytesPerLine: c_int = (iPow2VidWidth - (*addr_of_mut!(glConfig)).vidWidth) * 4;
                let iCopyBytes: c_int = (*addr_of_mut!(glConfig)).vidWidth * 4;
                //
                // do it...
                //
                let mut y: c_int = 0;
                while y < (*addr_of_mut!(glConfig)).vidHeight {
                    let pbDst_mut = pbDst.sub((iClearBytesPerLine as usize));
                    memset(pbDst_mut as *mut c_void, 0, iClearBytesPerLine as usize);
                    let pbDst_mut2 = pbDst_mut.sub((iCopyBytes as usize));
                    let pbSrc_mut = pbSrc.sub((iCopyBytes as usize));
                    memmove(pbDst_mut2 as *mut c_void, pbSrc_mut as *const c_void, iCopyBytes as usize);
                    y += 1;
                }
                //
                // ok, now we've got the screen image in the top left of the power-of-2 texture square,
                //	but of course the damn thing's upside down (thanks, GL), so invert it, but only within
                //	the picture pixels, NOT the upload texture as a whole...
                //
                let pbSwapLineBuffer: *mut byte = Z_Malloc(iCopyBytes, TAG_TEMP_WORKSPACE, qfalse) as *mut byte;
                let pbSrc_swap: *mut byte = pBuffer;
                let pbDst_swap: *mut byte = pBuffer.add((((*addr_of_mut!(glConfig)).vidHeight - 1) * iPow2VidWidth * 4) as usize);
                let mut y: c_int = 0;
                while y < (*addr_of_mut!(glConfig)).vidHeight / 2 {
                    memcpy(pbSwapLineBuffer as *mut c_void, pbDst_swap as *const c_void, iCopyBytes as usize);
                    memcpy(pbDst_swap as *mut c_void, pbSrc_swap as *const c_void, iCopyBytes as usize);
                    memcpy(pbSrc_swap as *mut c_void, pbSwapLineBuffer as *const c_void, iCopyBytes as usize);
                    let pbDst_swap_mut = pbDst_swap.sub((iPow2VidWidth * 4) as usize);
                    let pbSrc_swap_mut = pbSrc_swap.add((iPow2VidWidth * 4) as usize);
                    y += 1;
                }
                Z_Free(pbSwapLineBuffer as *mut c_void);

                //
                // Now, in case of busted drivers, 3DFX cards, etc etc we stomp the alphas to 255...
                //
                let mut pPix: *mut byte = pBuffer;
                let mut i: c_int = 0;
                while i < iBufferBytes / 4 {
                    *pPix.add(3) = 255;
                    pPix = pPix.add(4);
                    i += 1;
                }

                // work out what res we're capable of storing/xfading this "screen sprite"...
                //
                (*addr_of_mut!(Dissolve)).iWidth = (*addr_of_mut!(glConfig)).vidWidth;
                (*addr_of_mut!(Dissolve)).iHeight = (*addr_of_mut!(glConfig)).vidHeight;
                (*addr_of_mut!(Dissolve)).iUploadWidth = iPow2VidWidth;
                (*addr_of_mut!(Dissolve)).iUploadHeight = iPow2VidHeight;
                let mut iTexSize: c_int = (*addr_of_mut!(glConfig)).maxTextureSize;

                if (*addr_of_mut!(glConfig)).maxTextureSize < 256 {
                    // jic the driver sucks
                    iTexSize = 256;
                }

                if (*addr_of_mut!(Dissolve)).iUploadWidth > iTexSize {
                    (*addr_of_mut!(Dissolve)).iUploadWidth = iTexSize;
                }

                if (*addr_of_mut!(Dissolve)).iUploadHeight > iTexSize {
                    (*addr_of_mut!(Dissolve)).iUploadHeight = iTexSize;
                }

                // alloc resample buffer...  (note slight optimisation to avoid spurious alloc)
                //
                let pbReSampleBuffer: *mut byte = if (iPow2VidWidth == (*addr_of_mut!(Dissolve)).iUploadWidth
                    && iPow2VidHeight == (*addr_of_mut!(Dissolve)).iUploadHeight)
                {
                    core::ptr::null_mut()
                } else {
                    Z_Malloc(
                        iPow2VidWidth * iPow2VidHeight * 4,
                        TAG_TEMP_WORKSPACE,
                        qfalse,
                    ) as *mut byte
                };

                // re-sample screen...
                //
                let pbScreenSprite: *mut byte = RE_ReSample(
                    pBuffer,                            // byte *pbLoadedPic
                    iPow2VidWidth,                      // int iLoadedWidth
                    iPow2VidHeight,                     // int iLoadedHeight
                    pbReSampleBuffer,                   // byte *pbReSampleBuffer
                    &mut (*addr_of_mut!(Dissolve)).iUploadWidth, // int *piWidth
                    &mut (*addr_of_mut!(Dissolve)).iUploadHeight, // int *piHeight
                );

                (*addr_of_mut!(Dissolve)).pImage = R_CreateImage(
                    b"*DissolveImage\0".as_ptr() as *const c_char, // const char *name
                    pbScreenSprite,                                 // const byte *pic
                    (*addr_of_mut!(Dissolve)).iUploadWidth,         // int width
                    (*addr_of_mut!(Dissolve)).iUploadHeight,        // int height
                    GL_RGBA,
                    qfalse,         // qboolean mipmap
                    qfalse,         // qboolean allowPicmip
                    qfalse,         // qboolean allowTC
                    GL_CLAMP,       // int glWrapClampMode
                );

                static mut bBlack: [byte; 8 * 8 * 4] = [0; 8 * 8 * 4];
                let mut j: c_int = 0;
                while j < 8 * 8 * 4 {
                    bBlack[(j + 3) as usize] = 255; // itu?
                    j += 4;
                }

                (*addr_of_mut!(Dissolve)).pBlack = R_CreateImage(
                    b"*DissolveBlack\0".as_ptr() as *const c_char, // const char *name
                    bBlack.as_ptr(),                                 // const byte *pic
                    8,                                               // int width
                    8,                                               // int height
                    GL_RGBA,
                    qfalse,         // qboolean mipmap
                    qfalse,         // qboolean allowPicmip
                    GL_CLAMP,       // int glWrapClampMode
                );

                if !pbReSampleBuffer.is_null() {
                    Z_Free(pbReSampleBuffer as *mut c_void);
                }
                Z_Free(pBuffer as *mut c_void);

                // pick dissolve type...
                //
                #[cfg(feature = "disabled")]
                {
                    // cycles through every dissolve type, for testing...
                    //
                    static mut eDissolve: Dissolve_e = Dissolve_e::eDISSOLVE_RT_TO_LT;
                    (*addr_of_mut!(Dissolve)).eDissolveType = eDissolve;
                    eDissolve = unsafe {
                        mem::transmute::<c_int, Dissolve_e>(mem::transmute::<Dissolve_e, c_int>(eDissolve) + 1)
                    };
                    if eDissolve == Dissolve_e::eDISSOLVE_RAND_LIMIT {
                        eDissolve = unsafe {
                            mem::transmute::<c_int, Dissolve_e>(mem::transmute::<Dissolve_e, c_int>(eDissolve) + 1)
                        };
                    }
                    if (eDissolve as c_int) >= (Dissolve_e::eDISSOLVE_NUMBEROF as c_int) {
                        eDissolve = Dissolve_e::eDISSOLVE_RT_TO_LT;
                    }
                }
                #[cfg(not(feature = "disabled"))]
                {
                    // final (& random) version...
                    //
                    (*addr_of_mut!(Dissolve)).eDissolveType = unsafe {
                        mem::transmute::<c_int, Dissolve_e>(Q_irand(0, Dissolve_e::eDISSOLVE_RAND_LIMIT as c_int - 1))
                    };
                }

                if bForceCircularExtroWipe != qfalse {
                    (*addr_of_mut!(Dissolve)).eDissolveType = Dissolve_e::eDISSOLVE_CIRCULAR_IN;
                }

                // ... and load appropriate graphics...
                //

                // special tweak, although this code is normally called just before client spawns into world (and
                //	is therefore pretty much immune to precache issues) I also need to make sure that the inverse
                //	iris graphic is loaded so for the special case of doing a circular wipe at the end of the last
                //	level doesn't stall on loading the image. So I'll load it here anyway - to prime the image -
                //	then allow the random wiper to overwrite the ptr if needed. This way the end of level call
                //	will be instant.  Downside: every level has one extra 256x256 texture.
                #[cfg(not(feature = "xbox"))]
                {
                    (*addr_of_mut!(Dissolve)).pDissolve = R_FindImageFile(
                        b"gfx/2d/iris_mono_rev\0".as_ptr() as *const c_char, // const char *name
                        qfalse,                                                 // qboolean mipmap
                        qfalse,                                                 // qboolean allowPicmip
                        qfalse,                                                 // qboolean allowTC
                        GL_CLAMP,                                               // int glWrapClampMode
                    );
                }

                if !(*addr_of_mut!(com_buildScript)).is_null() && (*(*addr_of_mut!(com_buildScript))).integer != 0 {
                    // register any/all of the possible CASE statements below...
                    //
                    (*addr_of_mut!(Dissolve)).pDissolve = R_FindImageFile(
                        b"gfx/2d/iris_mono\0".as_ptr() as *const c_char, // const char *name
                        qfalse,                                             // qboolean mipmap
                        qfalse,                                             // qboolean allowPicmip
                        qfalse,                                             // qboolean allowTC
                        GL_CLAMP,                                           // int glWrapClampMode
                    );
                    (*addr_of_mut!(Dissolve)).pDissolve = R_FindImageFile(
                        b"textures/common/dissolve\0".as_ptr() as *const c_char, // const char *name
                        qfalse,                                                    // qboolean mipmap
                        qfalse,                                                    // qboolean allowPicmip
                        qfalse,                                                    // qboolean allowTC
                        GL_REPEAT,                                                 // int glWrapClampMode
                    );
                }

                match (*addr_of_mut!(Dissolve)).eDissolveType {
                    Dissolve_e::eDISSOLVE_CIRCULAR_IN => {
                        (*addr_of_mut!(Dissolve)).pDissolve = R_FindImageFile(
                            b"gfx/2d/iris_mono_rev\0".as_ptr() as *const c_char, // const char *name
                            qfalse,                                                 // qboolean mipmap
                            qfalse,                                                 // qboolean allowPicmip
                            qfalse,                                                 // qboolean allowTC
                            GL_CLAMP,                                               // int glWrapClampMode
                        );
                    }

                    Dissolve_e::eDISSOLVE_CIRCULAR_OUT => {
                        (*addr_of_mut!(Dissolve)).pDissolve = R_FindImageFile(
                            b"gfx/2d/iris_mono\0".as_ptr() as *const c_char, // const char *name
                            qfalse,                                             // qboolean mipmap
                            qfalse,                                             // qboolean allowPicmip
                            qfalse,                                             // qboolean allowTC
                            GL_CLAMP,                                           // int glWrapClampMode
                        );
                    }

                    _ => {
                        (*addr_of_mut!(Dissolve)).pDissolve = R_FindImageFile(
                            b"textures/common/dissolve\0".as_ptr() as *const c_char, // const char *name
                            qfalse,                                                    // qboolean mipmap
                            qfalse,                                                    // qboolean allowPicmip
                            qfalse,                                                    // qboolean allowTC
                            GL_REPEAT,                                                 // int glWrapClampMode
                        );
                    }
                }

                // all good?...
                //
                if !(*addr_of_mut!(Dissolve)).pDissolve.is_null() {
                    // test if image was found, if not, don't do dissolves
                    (*addr_of_mut!(Dissolve)).iStartTime = Sys_Milliseconds(); // gets overwritten first time, but MUST be set to NZ
                    (*addr_of_mut!(Dissolve)).bTouchNeeded = qtrue;
                    bReturn = qtrue;
                } else {
                    RE_KillDissolve();
                }
            }
        }
    }

    bReturn
}
