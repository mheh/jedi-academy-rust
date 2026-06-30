// ///////////////////////////////////////////////////////////////////////////////
// CDraw32 Class Interface
//
// Basic drawing routines for 32-bit per pixel buffer
// ///////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use core::ffi::c_long;
use core::ptr::{addr_of, addr_of_mut};

// #ifndef __linux__
// //#include <windows.h>
use crate::codemp::qcommon::platform_h::*;
// #endif

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

// #include <stdlib.h>  -- C system header; no Rust module import needed

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
        Self { r: R, g: G, b: B, a: A }
    }

    // CPixel32(byte R = 0, byte G = 0, byte B = 0, byte A = 255) default ctor
    #[inline]
    pub const fn default_c() -> Self {
        Self::new(0, 0, 0, 255)
    }

    // CPixel32(long l)
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
        // t.a = (byte)((x.a*alpha + y.a*inv_alpha)>>8);  return t;}
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

// Colors are 32-bit RGBA

// draw class
// static drawing context - static so we set only ONCE for many draw calls
pub static mut buffer: *mut CPixel32 = core::ptr::null_mut(); // pointer to pixel buffer (one active)
pub static mut buf_width: c_long = 0;                          // size of buffer
pub static mut buf_height: c_long = 0;                         // size of buffer
pub static mut stride: c_long = 0;                             // stride of buffer in pixels
pub static mut clip_min_x: c_long = 0;                        // clip bounds
pub static mut clip_min_y: c_long = 0;                        // clip bounds
pub static mut clip_max_x: c_long = 0;                        // clip bounds
pub static mut clip_max_y: c_long = 0;                        // clip bounds
pub static mut row_off: *mut c_long = core::ptr::null_mut();  // Table for quick Y calculations

// CDraw32 has ONLY static members; zero-sized unit struct is the faithful representation.
pub struct CDraw32;

impl CDraw32 {
    // CDraw32()  -- constructor defined in cm_draw.cpp
    pub fn new() -> Self { CDraw32 }

    // ~CDraw32() -- destructor defined in cm_draw.cpp

    // private: BlitClip
    unsafe fn BlitClip(
        &mut self,
        dstX: *mut c_long,
        dstY: *mut c_long,
        width: *mut c_long,
        height: *mut c_long,
        srcX: *mut c_long,
        srcY: *mut c_long,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

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
    pub unsafe fn SetBufferSize(width: c_long, height: c_long, stride_len: c_long) -> bool {
        unimplemented!() // defined in cm_draw.cpp
    }

    // call this to free the table for quick y calcs before the program ends
    #[inline]
    pub unsafe fn CleanUp() {
        unsafe {
            // C++: if (row_off) delete [] row_off;
            // porting note: dealloc must match SetBufferSize allocation; see cm_draw.cpp port
            let ro = *addr_of!(row_off);
            if !ro.is_null() {
                let _ = ro; // TODO: free with correct deallocator when cm_draw.cpp is ported
            }
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
    pub unsafe fn ClearLines(&mut self, color: CPixel32, start: c_long, end: c_long) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // clear screen buffer to color provided
    #[inline]
    pub unsafe fn ClearBuffer(&mut self, color: CPixel32) {
        unsafe {
            self.ClearLines(color, 0, *addr_of!(buf_height) - 1);
        }
    }

    // fill buffer alpha from start to end line
    pub unsafe fn SetAlphaLines(&mut self, alpha: byte, start: c_long, end: c_long) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // clear screen buffer to color provided
    #[inline]
    pub unsafe fn SetAlphaBuffer(&mut self, alpha: byte) {
        unsafe {
            self.SetAlphaLines(alpha, 0, *addr_of!(buf_height) - 1);
        }
    }

    // clip a line segment to the clip rect
    pub fn ClipLine(
        &self,
        x1: *mut c_long,
        y1: *mut c_long,
        x2: *mut c_long,
        y2: *mut c_long,
    ) -> bool {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a solid colored line, no clipping
    pub unsafe fn DrawLineNC(
        &mut self,
        x1: c_long,
        y1: c_long,
        x2: c_long,
        y2: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a solid color line
    #[inline]
    pub unsafe fn DrawLine(
        &mut self,
        mut x1: c_long,
        mut y1: c_long,
        mut x2: c_long,
        mut y2: c_long,
        color: CPixel32,
    ) {
        if self.ClipLine(&mut x1 as *mut c_long, &mut y1 as *mut c_long, &mut x2 as *mut c_long, &mut y2 as *mut c_long) {
            unsafe { self.DrawLineNC(x1, y1, x2, y2, color); }
        }
    }

    pub unsafe fn DrawLineAveNC(
        &mut self,
        x1: c_long,
        y1: c_long,
        x2: c_long,
        y2: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a translucent solid color line
    #[inline]
    pub unsafe fn DrawLineAve(
        &mut self,
        mut x1: c_long,
        mut y1: c_long,
        mut x2: c_long,
        mut y2: c_long,
        color: CPixel32,
    ) {
        if self.ClipLine(&mut x1 as *mut c_long, &mut y1 as *mut c_long, &mut x2 as *mut c_long, &mut y2 as *mut c_long) {
            unsafe { self.DrawLineAveNC(x1, y1, x2, y2, color); }
        }
    }

    // draw an anti-aliased line, no clipping
    pub unsafe fn DrawLineAANC(
        &mut self,
        x0: c_long,
        y0: c_long,
        x1: c_long,
        y1: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw an anti-aliased line
    #[inline]
    pub unsafe fn DrawLineAA(
        &mut self,
        mut x1: c_long,
        mut y1: c_long,
        mut x2: c_long,
        mut y2: c_long,
        color: CPixel32,
    ) {
        if self.ClipLine(&mut x1 as *mut c_long, &mut y1 as *mut c_long, &mut x2 as *mut c_long, &mut y2 as *mut c_long) {
            unsafe { self.DrawLineAANC(x1, y1, x2, y2, color); }
        }
    }

    // draw a filled rectangle, no clipping
    pub unsafe fn DrawRectNC(
        &mut self,
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a filled rectangle
    pub fn DrawRect(
        &mut self,
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a filled rectangle
    pub fn DrawRectAve(
        &mut self,
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a box (unfilled rectangle) no clip
    pub unsafe fn DrawBoxNC(
        &mut self,
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a box (unfilled rectangle)
    pub fn DrawBox(
        &mut self,
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a box (unfilled rectangle)
    pub fn DrawBoxAve(
        &mut self,
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a circle with fill and edge colors
    pub fn DrawCircle(
        &mut self,
        xc: c_long,
        yc: c_long,
        r: c_long,
        edge: CPixel32,
        fill: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a circle with fill and edge colors averaged with dest
    pub fn DrawCircleAve(
        &mut self,
        xc: c_long,
        yc: c_long,
        r: c_long,
        edge: CPixel32,
        fill: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // draw a polygon (complex) with fill and edge colors
    pub fn DrawPolygon(
        &mut self,
        nvert: c_long,
        point: *mut POINT,
        edge: CPixel32,
        fill: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // simple blit function
    pub unsafe fn BlitNC(
        &mut self,
        dstX: c_long,
        dstY: c_long,
        dstWidth: c_long,
        dstHeight: c_long,
        srcImage: *mut CPixel32,
        srcX: c_long,
        srcY: c_long,
        srcStride: c_long,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    pub fn Blit(
        &mut self,
        dstX: c_long,
        dstY: c_long,
        dstWidth: c_long,
        dstHeight: c_long,
        srcImage: *mut CPixel32,
        srcX: c_long,
        srcY: c_long,
        srcStride: c_long,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    // blit image times color
    pub fn BlitColor(
        &mut self,
        dstX: c_long,
        dstY: c_long,
        dstWidth: c_long,
        dstHeight: c_long,
        srcImage: *mut CPixel32,
        srcX: c_long,
        srcY: c_long,
        srcStride: c_long,
        color: CPixel32,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }

    pub fn Emboss(
        &mut self,
        dstX: c_long,
        dstY: c_long,
        width: c_long,
        height: c_long,
        clrImage: *mut CPixel32,
        clrX: c_long,
        clrY: c_long,
        clrStride: c_long,
    ) {
        unimplemented!() // defined in cm_draw.cpp
    }
}

// ///////////////////////////////////////////////////////////////////////////////
