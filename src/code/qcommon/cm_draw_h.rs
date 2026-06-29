///////////////////////////////////////////////////////////////////////////////
// CDraw32 Class Interface
//
// Basic drawing routines for 32-bit per pixel buffer
///////////////////////////////////////////////////////////////////////////////

use core::ffi::c_int;

// calc offset into image array for a pixel at (x,y)
#[inline]
pub const fn PIXPOS(x: c_int, y: c_int, stride: c_int) -> c_int {
    ((y) * (stride)) + (x)
}

// handy macros
#[inline]
pub const fn MIN(a: c_int, b: c_int) -> c_int {
    if (a) < (b) { (a) } else { (b) }
}

#[inline]
pub const fn MAX(a: c_int, b: c_int) -> c_int {
    if (a) > (b) { (a) } else { (b) }
}

#[inline]
pub const fn ABS(x: c_int) -> c_int {
    if (x) < 0 { -(x) } else { (x) }
}

#[inline]
pub const fn SIGN(x: c_int) -> c_int {
    if (x) < 0 { -1 } else { if (x) > 0 { 1 } else { 0 } }
}

#[inline]
pub fn SWAP(a: &mut c_int, b: &mut c_int) {
    *a ^= *b;
    *b ^= *a;
    *a ^= *b;
}

#[inline]
pub const fn SQR(a: c_int) -> c_int {
    (a) * (a)
}

#[inline]
pub const fn CLAMP(v: c_int, l: c_int, h: c_int) -> c_int {
    if (v) < (l) { (l) } else { if (v) > (h) { (h) } else { (v) } }
}

#[inline]
pub fn LERP(t: f32, a: f32, b: f32) -> f32 {
    ((b) - (a)) * (t) + (a)
}

// round a to nearest integer towards 0
#[inline]
pub const fn FLOOR(a: f32) -> c_int {
    if (a) > 0.0 { (a) as c_int } else { -((-(a)) as c_int) }
}

// round a to nearest integer away from 0
#[inline]
pub fn CEILING(a: f32) -> c_int {
    if (a) == ((a) as c_int) as f32 {
        (a) as c_int
    } else {
        if (a) > 0.0 {
            1 + ((a) as c_int)
        } else {
            -(1 + ((-(a)) as c_int))
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CPixel32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl CPixel32 {
    pub fn new(R: u8, G: u8, B: u8, A: u8) -> Self {
        CPixel32 {
            r: R,
            g: G,
            b: B,
            a: A,
        }
    }

    pub fn default() -> Self {
        CPixel32 {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn from_long(l: c_int) -> Self {
        CPixel32 {
            r: ((l >> 24) & 0xff) as u8,
            g: ((l >> 16) & 0xff) as u8,
            b: ((l >> 8) & 0xff) as u8,
            a: (l & 0xff) as u8,
        }
    }
}

pub const PIX32_SIZE: usize = std::mem::size_of::<CPixel32>();

// standard image operator macros
#[inline]
pub const fn IMAGE_SIZE(width: c_int, height: c_int) -> c_int {
    ((width) * (height) * (PIX32_SIZE as c_int))
}

#[inline]
pub fn AVE_PIX(x: CPixel32, y: CPixel32) -> CPixel32 {
    let mut t = CPixel32::default();
    t.r = ((((x.r as c_int) + (y.r as c_int)) >> 1) & 0xff) as u8;
    t.g = ((((x.g as c_int) + (y.g as c_int)) >> 1) & 0xff) as u8;
    t.b = ((((x.b as c_int) + (y.b as c_int)) >> 1) & 0xff) as u8;
    t.a = ((((x.a as c_int) + (y.a as c_int)) >> 1) & 0xff) as u8;
    t
}

#[inline]
pub fn ALPHA_PIX(x: CPixel32, y: CPixel32, alpha: c_int, inv_alpha: c_int) -> CPixel32 {
    let mut t = CPixel32::default();
    t.r = (((x.r as c_int) * alpha + (y.r as c_int) * inv_alpha) >> 8) as u8;
    t.g = (((x.g as c_int) * alpha + (y.g as c_int) * inv_alpha) >> 8) as u8;
    t.b = (((x.b as c_int) * alpha + (y.b as c_int) * inv_alpha) >> 8) as u8;
    //              t.a = (((x.a as c_int) * alpha + (y.a as c_int) * inv_alpha) >> 8) as u8;
    t.a = y.a;
    t
}

#[inline]
pub fn LIGHT_PIX(p: CPixel32, light: c_int) -> CPixel32 {
    let mut t = CPixel32::default();
    t.r = (CLAMP((((p.r as c_int) * light) >> 10) + (p.r as c_int), 0, 255)) as u8;
    t.g = (CLAMP((((p.g as c_int) * light) >> 10) + (p.g as c_int), 0, 255)) as u8;
    t.b = (CLAMP((((p.b as c_int) * light) >> 10) + (p.b as c_int), 0, 255)) as u8;
    t.a = p.a;
    t
}

// Colors are 32-bit RGBA

// draw class
pub struct CDraw32;

// static drawing context - static so we set only ONCE for many draw calls
pub static mut buffer: *mut CPixel32 = core::ptr::null_mut(); // pointer to pixel buffer (one active)
pub static mut buf_width: c_int = 0; // size of buffer
pub static mut buf_height: c_int = 0; // size of buffer
pub static mut stride: c_int = 0; // stride of buffer in pixels
pub static mut clip_min_x: c_int = 0; // clip bounds
pub static mut clip_min_y: c_int = 0; // clip bounds
pub static mut clip_max_x: c_int = 0; // clip bounds
pub static mut clip_max_y: c_int = 0; // clip bounds
pub static mut row_off: *mut c_int = core::ptr::null_mut(); // Table for quick Y calculations

impl CDraw32 {
    // set the rect to clip drawing functions to
    pub fn SetClip(min_x: c_int, min_y: c_int, max_x: c_int, max_y: c_int) {
        unsafe {
            clip_min_x = MAX(min_x, 0);
            clip_max_x = MIN(max_x, buf_width - 1);
            clip_min_y = MAX(min_y, 0);
            clip_max_y = MIN(max_y, buf_height - 1);
        }
    }

    pub fn GetClip(min_x: &mut c_int, min_y: &mut c_int, max_x: &mut c_int, max_y: &mut c_int) {
        unsafe {
            *min_x = clip_min_x;
            *min_y = clip_min_y;
            *max_x = clip_max_x;
            *max_y = clip_max_y;
        }
    }

    // set the buffer to use for drawing off-screen
    pub fn SetBuffer(buf: *mut CPixel32) {
        unsafe {
            buffer = buf;
        }
    }

    // set the dimensions of the off-screen buffer
    pub fn SetBufferSize(width: c_int, height: c_int, stride_len: c_int) -> bool {
        // stub - returns bool, implementation in .cpp
        false
    }

    // call this to free the table for quick y calcs before the program ends
    pub fn CleanUp() {
        unsafe {
            if !row_off.is_null() {
                let _ = Box::from_raw(row_off);
                row_off = core::ptr::null_mut();
            }
            buf_width = 0;
            buf_height = 0;
        }
    }

    // set a pixel at (x,y) to color (no clipping)
    pub fn PutPixNC(x: c_int, y: c_int, color: CPixel32) {
        unsafe {
            let offset = (row_off.add(y as usize) as *const c_int).read() + x;
            *buffer.add(offset as usize) = color;
        }
    }

    // set a pixel at (x,y) to color
    pub fn PutPix(x: c_int, y: c_int, color: CPixel32) {
        // clipping check
        unsafe {
            if x < clip_min_x || x > clip_max_x || y < clip_min_y || y > clip_max_y {
                return;
            }
            CDraw32::PutPixNC(x, y, color);
        }
    }

    // get the color of a pixel at (x,y)
    pub fn GetPix(x: c_int, y: c_int) -> CPixel32 {
        unsafe {
            let offset = (row_off.add(y as usize) as *const c_int).read() + x;
            *buffer.add(offset as usize)
        }
    }

    // set a pixel at (x,y) with 50% translucency (no clip)
    pub fn PutPixAveNC(x: c_int, y: c_int, color: CPixel32) {
        CDraw32::PutPixNC(x, y, AVE_PIX(CDraw32::GetPix(x, y), color));
    }

    // set a pixel at (x,y) with 50% translucency
    pub fn PutPixAve(x: c_int, y: c_int, color: CPixel32) {
        // clipping check
        unsafe {
            if x < clip_min_x || x > clip_max_x || y < clip_min_y || y > clip_max_y {
                return;
            }
            CDraw32::PutPixNC(x, y, AVE_PIX(CDraw32::GetPix(x, y), color));
        }
    }

    // set a pixel at (x,y) with translucency level (no clip)
    pub fn PutPixAlphaNC(x: c_int, y: c_int, color: CPixel32) {
        CDraw32::PutPixNC(
            x,
            y,
            ALPHA_PIX(color, CDraw32::GetPix(x, y), color.a as c_int, 256 - (color.a as c_int)),
        );
    }

    // set a pixel at (x,y) with translucency level
    pub fn PutPixAlpha(x: c_int, y: c_int, color: CPixel32) {
        // clipping check
        unsafe {
            if x < clip_min_x || x > clip_max_x || y < clip_min_y || y > clip_max_y {
                return;
            }
            CDraw32::PutPixAlphaNC(
                x,
                y,
                ALPHA_PIX(
                    color,
                    CDraw32::GetPix(x, y),
                    color.a as c_int,
                    256 - (color.a as c_int),
                ),
            );
        }
    }

    // clear screen buffer to color from start to end line
    pub fn ClearLines(color: CPixel32, start: c_int, end: c_int) {
        // implementation stub
    }

    // clear screen buffer to color provided
    pub fn ClearBuffer(color: CPixel32) {
        unsafe {
            CDraw32::ClearLines(color, 0, buf_height - 1);
        }
    }

    // fill buffer alpha from start to end line
    pub fn SetAlphaLines(alpha: u8, start: c_int, end: c_int) {
        // implementation stub
    }

    // clear screen buffer to color provided
    pub fn SetAlphaBuffer(alpha: u8) {
        unsafe {
            CDraw32::SetAlphaLines(alpha, 0, buf_height - 1);
        }
    }

    // clip a line segment to the clip rect
    pub fn ClipLine(x1: &mut c_int, y1: &mut c_int, x2: &mut c_int, y2: &mut c_int) -> bool {
        // implementation stub
        false
    }

    // draw a solid colored line, no clipping
    pub fn DrawLineNC(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw a solid color line
    pub fn DrawLine(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
        let mut lx1 = x1;
        let mut ly1 = y1;
        let mut lx2 = x2;
        let mut ly2 = y2;
        if CDraw32::ClipLine(&mut lx1, &mut ly1, &mut lx2, &mut ly2) {
            CDraw32::DrawLineNC(lx1, ly1, lx2, ly2, color);
        }
    }

    pub fn DrawLineAveNC(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw a translucent solid color line
    pub fn DrawLineAve(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
        let mut lx1 = x1;
        let mut ly1 = y1;
        let mut lx2 = x2;
        let mut ly2 = y2;
        if CDraw32::ClipLine(&mut lx1, &mut ly1, &mut lx2, &mut ly2) {
            CDraw32::DrawLineAveNC(lx1, ly1, lx2, ly2, color);
        }
    }

    // draw an anti-aliased line, no clipping
    pub fn DrawLineAANC(x0: c_int, y0: c_int, x1: c_int, y1: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw an anti-aliased line
    pub fn DrawLineAA(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
        let mut lx1 = x1;
        let mut ly1 = y1;
        let mut lx2 = x2;
        let mut ly2 = y2;
        if CDraw32::ClipLine(&mut lx1, &mut ly1, &mut lx2, &mut ly2) {
            CDraw32::DrawLineAANC(lx1, ly1, lx2, ly2, color);
        }
    }

    // draw a filled rectangle, no clipping
    pub fn DrawRectNC(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw a filled rectangle
    pub fn DrawRect(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw a filled rectangle
    pub fn DrawRectAve(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw a box (unfilled rectangle) no clip
    pub fn DrawBoxNC(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw a box (unfilled rectangle)
    pub fn DrawBox(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw a box (unfilled rectangle)
    pub fn DrawBoxAve(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
        // implementation stub
    }

    // draw a circle with fill and edge colors
    pub fn DrawCircle(xc: c_int, yc: c_int, r: c_int, edge: CPixel32, fill: CPixel32) {
        // implementation stub
    }

    // draw a circle with fill and edge colors averaged with dest
    pub fn DrawCircleAve(xc: c_int, yc: c_int, r: c_int, edge: CPixel32, fill: CPixel32) {
        // implementation stub
    }

    // draw a polygon (complex) with fill and edge colors
    pub fn DrawPolygon(nvert: c_int, point: *mut POINT, edge: CPixel32, fill: CPixel32) {
        // implementation stub
    }

    // simple blit function
    pub fn BlitNC(
        dstX: c_int,
        dstY: c_int,
        dstWidth: c_int,
        dstHeight: c_int,
        srcImage: *mut CPixel32,
        srcX: c_int,
        srcY: c_int,
        srcStride: c_int,
    ) {
        // implementation stub
    }

    pub fn Blit(
        dstX: c_int,
        dstY: c_int,
        dstWidth: c_int,
        dstHeight: c_int,
        srcImage: *mut CPixel32,
        srcX: c_int,
        srcY: c_int,
        srcStride: c_int,
    ) {
        // implementation stub
    }

    // blit image times color
    pub fn BlitColor(
        dstX: c_int,
        dstY: c_int,
        dstWidth: c_int,
        dstHeight: c_int,
        srcImage: *mut CPixel32,
        srcX: c_int,
        srcY: c_int,
        srcStride: c_int,
        color: CPixel32,
    ) {
        // implementation stub
    }

    pub fn Emboss(
        dstX: c_int,
        dstY: c_int,
        width: c_int,
        height: c_int,
        clrImage: *mut CPixel32,
        clrX: c_int,
        clrY: c_int,
        clrStride: c_int,
    ) {
        // implementation stub
    }
}

// Stub for POINT type - needed for DrawPolygon
#[repr(C)]
pub struct POINT {
    pub x: c_int,
    pub y: c_int,
}

///////////////////////////////////////////////////////////////////////////////
