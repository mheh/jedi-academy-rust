//! `cm_draw.h` — 32-bit pixel drawing declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::byte;
use core::ffi::c_long;
use core::ptr::{addr_of, addr_of_mut};

// calc offset into image array for a pixel at (x,y)
#[inline]
pub const fn PIXPOS(x: c_long, y: c_long, stride: c_long) -> c_long {
    (y * stride) + x
}

// handy macros
#[inline]
pub const fn MIN(a: c_long, b: c_long) -> c_long {
    if a < b { a } else { b }
}

#[inline]
pub const fn MAX(a: c_long, b: c_long) -> c_long {
    if a > b { a } else { b }
}

#[inline]
pub const fn ABS(x: c_long) -> c_long {
    if x < 0 { -x } else { x }
}

#[inline]
pub const fn SIGN(x: c_long) -> c_long {
    if x < 0 {
        -1
    } else if x > 0 {
        1
    } else {
        0
    }
}

#[inline]
pub fn SWAP(a: *mut c_long, b: *mut c_long) {
    unsafe {
        *a ^= *b;
        *b ^= *a;
        *a ^= *b;
    }
}

#[inline]
pub const fn SQR(a: c_long) -> c_long {
    a * a
}

#[inline]
pub const fn CLAMP(v: c_long, l: c_long, h: c_long) -> c_long {
    if v < l {
        l
    } else if v > h {
        h
    } else {
        v
    }
}

#[inline]
pub fn LERP(t: f32, a: f32, b: f32) -> f32 {
    ((b - a) * t) + a
}

// round a to nearest integer towards 0
#[inline]
pub fn FLOOR(a: f64) -> c_long {
    if a > 0.0 {
        a as c_long
    } else {
        -((-a) as c_long)
    }
}

// round a to nearest integer away from 0
#[inline]
pub fn CEILING(a: f64) -> f64 {
    if a == (a as c_long) as f64 {
        a
    } else if a > 0.0 {
        (1 + a as c_long) as f64
    } else {
        -(1 + (-a) as c_long) as f64
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CPixel32 {
    pub r: byte,
    pub g: byte,
    pub b: byte,
    pub a: byte,
}

impl Default for CPixel32 {
    #[inline]
    fn default() -> Self {
        Self::default_c()
    }
}

impl CPixel32 {
    #[inline]
    pub const fn new(R: byte, G: byte, B: byte, A: byte) -> Self {
        Self {
            r: R,
            g: G,
            b: B,
            a: A,
        }
    }

    #[inline]
    pub const fn default_c() -> Self {
        Self::new(0, 0, 0, 255)
    }

    #[inline]
    pub const fn from_long(l: c_long) -> Self {
        Self {
            r: ((l >> 24) & 0xff) as byte,
            g: ((l >> 16) & 0xff) as byte,
            b: ((l >> 8) & 0xff) as byte,
            a: (l & 0xff) as byte,
        }
    }
}

pub const PIX32_SIZE: usize = core::mem::size_of::<CPixel32>();

const _: () = assert!(core::mem::size_of::<CPixel32>() == 4);
const _: () = assert!(core::mem::align_of::<CPixel32>() == 1);

// standard image operator macros
#[inline]
pub const fn IMAGE_SIZE(width: usize, height: usize) -> usize {
    width * height * PIX32_SIZE
}

#[inline]
pub const fn AVE_PIX(x: CPixel32, y: CPixel32) -> CPixel32 {
    CPixel32 {
        r: (((x.r as i32 + y.r as i32) >> 1) & 0xff) as byte,
        g: (((x.g as i32 + y.g as i32) >> 1) & 0xff) as byte,
        b: (((x.b as i32 + y.b as i32) >> 1) & 0xff) as byte,
        a: (((x.a as i32 + y.a as i32) >> 1) & 0xff) as byte,
    }
}

#[inline]
pub const fn ALPHA_PIX(x: CPixel32, y: CPixel32, alpha: c_long, inv_alpha: c_long) -> CPixel32 {
    CPixel32 {
        r: (((x.r as c_long * alpha + y.r as c_long * inv_alpha) >> 8) & 0xff) as byte,
        g: (((x.g as c_long * alpha + y.g as c_long * inv_alpha) >> 8) & 0xff) as byte,
        b: (((x.b as c_long * alpha + y.b as c_long * inv_alpha) >> 8) & 0xff) as byte,
        // t.a = (byte)((x.a*alpha + y.a*inv_alpha)>>8);  return t;
        a: y.a,
    }
}

#[inline]
pub const fn LIGHT_PIX(p: CPixel32, light: c_long) -> CPixel32 {
    CPixel32 {
        r: CLAMP(((p.r as c_long * light) >> 10) + p.r as c_long, 0, 255) as byte,
        g: CLAMP(((p.g as c_long * light) >> 10) + p.g as c_long, 0, 255) as byte,
        b: CLAMP(((p.b as c_long * light) >> 10) + p.b as c_long, 0, 255) as byte,
        a: p.a,
    }
}

// `POINT` comes from Windows headers in the original non-Linux include path.
// It is left as a minimal layout stub until the platform header is fully ported.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct POINT {
    pub x: c_long,
    pub y: c_long,
}

// draw class
#[repr(C)]
pub struct CDraw32 {
    _private: [u8; 0],
}

pub static mut buffer: *mut CPixel32 = core::ptr::null_mut(); // pointer to pixel buffer (one active)
pub static mut buf_width: c_long = 0; // size of buffer
pub static mut buf_height: c_long = 0; // size of buffer
pub static mut stride: c_long = 0; // stride of buffer in pixels
pub static mut clip_min_x: c_long = 0; // clip bounds
pub static mut clip_min_y: c_long = 0; // clip bounds
pub static mut clip_max_x: c_long = 0; // clip bounds
pub static mut clip_max_y: c_long = 0; // clip bounds
pub static mut row_off: *mut c_long = core::ptr::null_mut(); // Table for quick Y calculations

impl CDraw32 {
    // private: BlitClip
    // public: CDraw32, ~CDraw32

    // set the rect to clip drawing functions to
    #[inline]
    pub unsafe fn SetClip(min_x: c_long, min_y: c_long, max_x: c_long, max_y: c_long) {
        unsafe {
            *addr_of_mut!(clip_min_x) = MAX(min_x, 0);
            *addr_of_mut!(clip_max_x) = MIN(max_x, *addr_of!(buf_width) - 1);
            *addr_of_mut!(clip_min_y) = MAX(min_y, 0);
            *addr_of_mut!(clip_max_y) = MIN(max_y, *addr_of!(buf_height) - 1);
        }
    }

    #[inline]
    pub unsafe fn GetClip(
        min_x: *mut c_long,
        min_y: *mut c_long,
        max_x: *mut c_long,
        max_y: *mut c_long,
    ) {
        unsafe {
            *min_x = *addr_of!(clip_min_x);
            *min_y = *addr_of!(clip_min_y);
            *max_x = *addr_of!(clip_max_x);
            *max_y = *addr_of!(clip_max_y);
        }
    }

    // set the buffer to use for drawing off-screen
    #[inline]
    pub unsafe fn SetBuffer(buf: *mut CPixel32) {
        unsafe {
            *addr_of_mut!(buffer) = buf;
        }
    }

    // set the dimensions of the off-screen buffer
    // static bool SetBufferSize(long width,long height,long stride_len);

    // call this to free the table for quick y calcs before the program ends
    #[inline]
    pub unsafe fn CleanUp() {
        unsafe {
            // Original C++ deletes row_off here. Allocation ownership is unported,
            // so this blind header only clears the static drawing context.
            *addr_of_mut!(row_off) = core::ptr::null_mut();
            *addr_of_mut!(buf_width) = 0;
            *addr_of_mut!(buf_height) = 0;
        }
    }

    // set a pixel at (x,y) to color (no clipping)
    #[inline]
    pub unsafe fn PutPixNC(&mut self, x: c_long, y: c_long, color: CPixel32) {
        unsafe {
            let offset = *(*addr_of!(row_off)).offset(y as isize) + x;
            *(*addr_of!(buffer)).offset(offset as isize) = color;
        }
    }

    // set a pixel at (x,y) to color
    #[inline]
    pub unsafe fn PutPix(&mut self, x: c_long, y: c_long, color: CPixel32) {
        unsafe {
            // clipping check
            if x < *addr_of!(clip_min_x)
                || x > *addr_of!(clip_max_x)
                || y < *addr_of!(clip_min_y)
                || y > *addr_of!(clip_max_y)
            {
                return;
            }
            self.PutPixNC(x, y, color);
        }
    }

    // get the color of a pixel at (x,y)
    #[inline]
    pub unsafe fn GetPix(&self, x: c_long, y: c_long) -> CPixel32 {
        unsafe {
            let offset = *(*addr_of!(row_off)).offset(y as isize) + x;
            *(*addr_of!(buffer)).offset(offset as isize)
        }
    }

    // set a pixel at (x,y) with 50% translucency (no clip)
    #[inline]
    pub unsafe fn PutPixAveNC(&mut self, x: c_long, y: c_long, color: CPixel32) {
        unsafe {
            self.PutPixNC(x, y, AVE_PIX(self.GetPix(x, y), color));
        }
    }

    // set a pixel at (x,y) with 50% translucency
    #[inline]
    pub unsafe fn PutPixAve(&mut self, x: c_long, y: c_long, color: CPixel32) {
        unsafe {
            // clipping check
            if x < *addr_of!(clip_min_x)
                || x > *addr_of!(clip_max_x)
                || y < *addr_of!(clip_min_y)
                || y > *addr_of!(clip_max_y)
            {
                return;
            }
            self.PutPixNC(x, y, AVE_PIX(self.GetPix(x, y), color));
        }
    }

    // set a pixel at (x,y) with translucency level (no clip)
    #[inline]
    pub unsafe fn PutPixAlphaNC(&mut self, x: c_long, y: c_long, color: CPixel32) {
        unsafe {
            self.PutPixNC(
                x,
                y,
                ALPHA_PIX(color, self.GetPix(x, y), color.a as c_long, 256 - color.a as c_long),
            );
        }
    }

    // set a pixel at (x,y) with translucency level
    #[inline]
    pub unsafe fn PutPixAlpha(&mut self, x: c_long, y: c_long, color: CPixel32) {
        unsafe {
            // clipping check
            if x < *addr_of!(clip_min_x)
                || x > *addr_of!(clip_max_x)
                || y < *addr_of!(clip_min_y)
                || y > *addr_of!(clip_max_y)
            {
                return;
            }
            self.PutPixNC(
                x,
                y,
                ALPHA_PIX(color, self.GetPix(x, y), color.a as c_long, 256 - color.a as c_long),
            );
        }
    }

    // clear screen buffer to color from start to end line
    // void ClearLines(CPixel32 color,long start,long end);

    // clear screen buffer to color provided
    // void ClearBuffer(CPixel32 color) {ClearLines(color,0,buf_height-1);}

    // fill buffer alpha from start to end line
    // void SetAlphaLines(byte alpha,long start,long end);

    // clear screen buffer to color provided
    // void SetAlphaBuffer(byte alpha) {SetAlphaLines(alpha,0,buf_height-1);}

    // clip a line segment to the clip rect
    // bool ClipLine(long& x1, long& y1, long& x2, long& y2);

    // draw a solid colored line, no clipping
    // void DrawLineNC(long x1, long y1, long x2, long y2, CPixel32 color);

    // draw a solid color line
    // void DrawLine(long x1, long y1, long x2, long y2, CPixel32 color)
    //     { if (ClipLine(x1,y1,x2,y2)) DrawLineNC(x1,y1,x2,y2,color); }

    // void DrawLineAveNC(long x1, long y1, long x2, long y2, CPixel32 color);

    // draw a translucent solid color line
    // void DrawLineAve(long x1, long y1, long x2, long y2, CPixel32 color)
    //     { if (ClipLine(x1,y1,x2,y2)) DrawLineAveNC(x1,y1,x2,y2,color); }

    // draw an anti-aliased line, no clipping
    // void DrawLineAANC(long x0, long y0, long x1, long y1, CPixel32 color);

    // draw an anti-aliased line
    // void DrawLineAA(long x1, long y1, long x2, long y2, CPixel32 color)
    //     { if (ClipLine(x1,y1,x2,y2)) DrawLineAANC(x1,y1,x2,y2,color); }

    // draw a filled rectangle, no clipping
    // void DrawRectNC(long ulx, long uly, long width, long height,CPixel32 color);

    // draw a filled rectangle
    // void DrawRect(long ulx, long uly, long width, long height, CPixel32 color);

    // draw a filled rectangle
    // void DrawRectAve(long ulx, long uly, long width, long height,CPixel32 color);

    // draw a box (unfilled rectangle) no clip
    // void DrawBoxNC(long ulx, long uly, long width, long height, CPixel32 color);

    // draw a box (unfilled rectangle)
    // void DrawBox(long ulx, long uly, long width, long height, CPixel32 color);

    // draw a box (unfilled rectangle)
    // void DrawBoxAve(long ulx, long uly, long width, long height, CPixel32 color);

    // draw a circle with fill and edge colors
    // void DrawCircle(long xc, long yc, long r, CPixel32 edge, CPixel32 fill);

    // draw a circle with fill and edge colors averaged with dest
    // void DrawCircleAve(long xc, long yc, long r, CPixel32 edge, CPixel32 fill);

    // draw a polygon (complex) with fill and edge colors
    // void DrawPolygon(long nvert, POINT *point, CPixel32 edge, CPixel32 fill);

    // simple blit function
    // void BlitNC(long dstX, long dstY, long dstWidth, long dstHeight,
    //             CPixel32* srcImage, long srcX, long srcY, long srcStride);

    // void Blit(long dstX, long dstY, long dstWidth, long dstHeight,
    //           CPixel32* srcImage, long srcX, long srcY, long srcStride);

    // blit image times color
    // void BlitColor(long dstX, long dstY, long dstWidth, long dstHeight,
    //                CPixel32* srcImage, long srcX, long srcY, long srcStride, CPixel32 color);

    // void Emboss(long dstX, long dstY, long width, long height,
    //             CPixel32* clrImage, long clrX, long clrY, long clrStride);
}
