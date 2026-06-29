///////////////////////////////////////////////////////////////////////////////
// CDraw32 Class Implementation
//
// Basic drawing routines for 32-bit buffer
///////////////////////////////////////////////////////////////////////////////

use core::ffi::{c_int, c_void};
use core::mem;
use core::ptr::{addr_of, addr_of_mut, copy_nonoverlapping};

// Type definitions
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CPixel32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct POINT {
    pub x: c_int,
    pub y: c_int,
}

#[repr(C)]
struct POLYEDGE {
    x: c_int,       // x coordinate of edge's intersection with current scanline
    dx: c_int,      // change in x with respect to y
    i: c_int,       // edge number: edge i goes from pt[i] to pt[i+1]
}

// Constants
const KWIDTH: c_int = 2;
const INT_SHIFT: c_int = 13;
const LEFT: c_int = 1;    // code bits
const RIGHT: c_int = 2;
const TOP: c_int = 4;
const BOTTOM: c_int = 8;

///////////// statics for CDraw32 //////////////////////////////////
// Used by all drawing routines as the "current" drawing context
pub static mut buffer: *mut CPixel32 = core::ptr::null_mut(); // pointer to 32-bit deep pixel buffer
pub static mut buf_width: c_int = 0;        // width of buffer in pixels
pub static mut buf_height: c_int = 0;       // height of buffer in pixels
pub static mut stride: c_int = 0;           // stride in pixels
pub static mut clip_min_x: c_int = 0;       // clip bounds
pub static mut clip_min_y: c_int = 0;       // clip bounds
pub static mut clip_max_x: c_int = 0;       // clip bounds
pub static mut clip_max_y: c_int = 0;       // clip bounds
pub static mut row_off: *mut c_int = core::ptr::null_mut(); // Table for quick Y calculations

static mut imgKernel: [[c_int; 5]; 5] = [
    [-1,-1,-1,-1, 0],
    [-1,-1,-1, 0, 1],
    [-1,-1, 0, 1, 1],
    [-1, 0, 1, 1, 1],
    [ 0, 1, 1, 1, 1]
];

// Inline helper functions for macros
#[inline]
fn PIXPOS(x: c_int, y: c_int, stride: c_int) -> c_int {
    y * stride + x
}

#[inline]
fn CLAMP(v: c_int, l: c_int, h: c_int) -> c_int {
    if v < l { l } else if v > h { h } else { v }
}

#[inline]
fn SWAP(a: &mut c_int, b: &mut c_int) {
    *a ^= *b;
    *b ^= *a;
    *a ^= *b;
}

#[inline]
fn ABS(x: c_int) -> c_int {
    if x < 0 { -x } else { x }
}

#[inline]
fn SIGN(x: c_int) -> c_int {
    if x < 0 { -1 } else if x > 0 { 1 } else { 0 }
}

#[inline]
fn MAX(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

#[inline]
fn MIN(a: c_int, b: c_int) -> c_int {
    if a < b { a } else { b }
}

#[inline]
fn AVE_PIX(x: CPixel32, y: CPixel32) -> CPixel32 {
    CPixel32 {
        r: (((x.r as c_int + y.r as c_int) >> 1) as u8),
        g: (((x.g as c_int + y.g as c_int) >> 1) as u8),
        b: (((x.b as c_int + y.b as c_int) >> 1) as u8),
        a: (((x.a as c_int + y.a as c_int) >> 1) as u8),
    }
}

#[inline]
fn ALPHA_PIX(x: CPixel32, y: CPixel32, alpha: c_int, inv_alpha: c_int) -> CPixel32 {
    CPixel32 {
        r: (((x.r as c_int * alpha + y.r as c_int * inv_alpha) >> 8) as u8),
        g: (((x.g as c_int * alpha + y.g as c_int * inv_alpha) >> 8) as u8),
        b: (((x.b as c_int * alpha + y.b as c_int * inv_alpha) >> 8) as u8),
        a: y.a,
    }
}

#[inline]
fn LIGHT_PIX(p: CPixel32, light: c_int) -> CPixel32 {
    CPixel32 {
        r: (CLAMP(((p.r as c_int * light) >> 10) + p.r as c_int, 0, 255) as u8),
        g: (CLAMP(((p.g as c_int * light) >> 10) + p.g as c_int, 0, 255) as u8),
        b: (CLAMP(((p.b as c_int * light) >> 10) + p.b as c_int, 0, 255) as u8),
        a: p.a,
    }
}

// Stubs for methods not in this file
#[inline]
unsafe fn SetClip(min_x: c_int, min_y: c_int, max_x: c_int, max_y: c_int) {
    *addr_of_mut!(clip_min_x) = MAX(min_x, 0);
    *addr_of_mut!(clip_max_x) = MIN(max_x, *addr_of!(buf_width) - 1);
    *addr_of_mut!(clip_min_y) = MAX(min_y, 0);
    *addr_of_mut!(clip_max_y) = MIN(max_y, *addr_of!(buf_height) - 1);
}

#[inline]
unsafe fn GetClip(min_x: &mut c_int, min_y: &mut c_int, max_x: &mut c_int, max_y: &mut c_int) {
    *min_x = *addr_of!(clip_min_x);
    *min_y = *addr_of!(clip_min_y);
    *max_x = *addr_of!(clip_max_x);
    *max_y = *addr_of!(clip_max_y);
}

#[inline]
unsafe fn SetBuffer(buf: *mut CPixel32) {
    *addr_of_mut!(buffer) = buf;
}

#[inline]
unsafe fn PutPixNC(x: c_int, y: c_int, color: CPixel32) {
    let buf = *addr_of!(buffer);
    let r_off = *addr_of!(row_off);
    let offset = *r_off.add(y as usize) + x;
    *buf.add(offset as usize) = color;
}

#[inline]
unsafe fn PutPix(x: c_int, y: c_int, color: CPixel32) {
    // clipping check
    if x < *addr_of!(clip_min_x) || x > *addr_of!(clip_max_x) ||
       y < *addr_of!(clip_min_y) || y > *addr_of!(clip_max_y) {
        return;
    }
    PutPixNC(x, y, color);
}

#[inline]
unsafe fn GetPix(x: c_int, y: c_int) -> CPixel32 {
    let buf = *addr_of!(buffer);
    let r_off = *addr_of!(row_off);
    let offset = *r_off.add(y as usize) + x;
    *buf.add(offset as usize)
}

#[inline]
unsafe fn PutPixAveNC(x: c_int, y: c_int, color: CPixel32) {
    PutPixNC(x, y, AVE_PIX(GetPix(x, y), color));
}

#[inline]
unsafe fn PutPixAve(x: c_int, y: c_int, color: CPixel32) {
    // clipping check
    if x < *addr_of!(clip_min_x) || x > *addr_of!(clip_max_x) ||
       y < *addr_of!(clip_min_y) || y > *addr_of!(clip_max_y) {
        return;
    }
    PutPixNC(x, y, AVE_PIX(GetPix(x, y), color));
}

#[inline]
unsafe fn PutPixAlphaNC(x: c_int, y: c_int, color: CPixel32) {
    PutPixNC(x, y, ALPHA_PIX(color, GetPix(x, y), color.a as c_int, 256 - color.a as c_int));
}

#[inline]
unsafe fn PutPixAlpha(x: c_int, y: c_int, color: CPixel32) {
    // clipping check
    if x < *addr_of!(clip_min_x) || x > *addr_of!(clip_max_x) ||
       y < *addr_of!(clip_min_y) || y > *addr_of!(clip_max_y) {
        return;
    }
    PutPixNC(x, y, ALPHA_PIX(color, GetPix(x, y), color.a as c_int, 256 - color.a as c_int));
}

// Destructor / Cleanup
pub unsafe fn CleanUp() {
    let r_off = *addr_of!(row_off);
    if !r_off.is_null() {
        libc::free(r_off as *mut c_void);
        *addr_of_mut!(row_off) = core::ptr::null_mut();
    }
    *addr_of_mut!(buf_width) = 0;
    *addr_of_mut!(buf_height) = 0;
}

// Constructor stub
pub fn CDraw32_new() {
    // Constructor does nothing
}

// Destructor stub
pub fn CDraw32_drop() {
    // Destructor does nothing in original
}

pub unsafe fn Emboss(dstX: c_int, dstY: c_int, width: c_int, height: c_int,
                     clrImage: *mut CPixel32, clrX: c_int, clrY: c_int, clrStride: c_int) {
    let mut dst: *mut CPixel32;
    let mut clr: *mut CPixel32;
    let mut x: c_int;
    let mut y: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let dstNextLine: c_int;
    let clrNextLine: c_int;

    assert!(!(*addr_of!(buffer)).is_null());

    let mut d_dstX = dstX;
    let mut d_dstY = dstY;
    let mut d_width = width;
    let mut d_height = height;
    let mut d_clrX = clrX;
    let mut d_clrY = clrY;

    BlitClip(&mut d_dstX, &mut d_dstY, &mut d_width, &mut d_height, &mut d_clrX, &mut d_clrY);

    if d_width < 1 || d_height < 1 {
        return;
    }

    let buf = *addr_of!(buffer);
    let r_off = *addr_of!(row_off);
    let s = *addr_of!(stride);

    dst = buf.add(PIXPOS(d_dstX, d_dstY, s) as usize);
    clr = clrImage.add(PIXPOS(d_clrX, d_clrY, clrStride) as usize);

    dstNextLine = s - d_width;
    clrNextLine = clrStride - d_width;

    y = 0;
    while y < d_height {
        x = 0;
        while x < d_width {
            let mut accum: c_int = 0;
            j = -KWIDTH;
            while j <= KWIDTH {
                i = -KWIDTH;
                while i <= KWIDTH {
                    let xk = CLAMP(x + i, d_clrX, d_clrX + d_width - 1);
                    let yk = CLAMP(y + j, d_clrY, d_clrY + d_height - 1);
                    accum += (*clrImage.add(PIXPOS(xk, yk, clrStride) as usize)).a as c_int * imgKernel[(j + KWIDTH) as usize][(i + KWIDTH) as usize];
                    i += 1;
                }
                j += 1;
            }
            *dst = LIGHT_PIX(*clr, accum);
            (*dst).a = 255;
            dst = dst.add(1);
            clr = clr.add(1);
            x += 1;
        }
        dst = dst.add(dstNextLine as usize);
        clr = clr.add(clrNextLine as usize);
        y += 1;
    }
}

pub unsafe fn SetBufferSize(width: c_int, height: c_int, stride_len: c_int) -> bool {
    //USE:	setup for a particular size drawing buffer
    //			(do not re-setup if buffer size has not changed)
    //IN:		width,height	- size of buffer
    //			stride_len		- distance to next line
    //OUT:	true if everything goes OK, otherwise false
    let mut i: c_int;

    assert!(width != 0);
    assert!(height != 0);
    assert!(stride_len != 0);

    if *addr_of!(buf_width) != width || *addr_of!(buf_height) != height ||
       stride_len != *addr_of!(stride) {
        // need to re-create row_off table
        *addr_of_mut!(buf_width) = width;
        *addr_of_mut!(buf_height) = height;
        *addr_of_mut!(stride) = stride_len;

        let existing_row_off = *addr_of!(row_off);
        if !existing_row_off.is_null() {
            libc::free(existing_row_off as *mut c_void);
        }

        // row offsets used for quick pixel address calcs
        let new_row_off = libc::malloc(mem::size_of::<c_int>() * height as usize) as *mut c_int;
        *addr_of_mut!(row_off) = new_row_off;

        assert!(!new_row_off.is_null());
        if new_row_off.is_null() {
            return false;
        }

        // table for quick pixel lookups
        i = 0;
        while i < height {
            *new_row_off.add(i as usize) = i * stride_len;
            i += 1;
        }
    }
    // set default clip bounds
    SetClip(0, 0, width - 1, height - 1);

    true
}

pub unsafe fn ClearLines(color: CPixel32, start: c_int, end: c_int) {
    //USE:	clear screen buffer to color provided for lines
    //IN:		color	 				- 32-bit color value
    //			start through end		- line numbers
    //OUT:	none
    let mut dest: *mut CPixel32;
    let mut line: c_int;
    let mut i: c_int;
    let next_line: c_int;

    assert!(!(*addr_of!(buffer)).is_null());
    assert!(start >= 0);
    assert!(end < *addr_of!(buf_height));

    let buf = *addr_of!(buffer);
    let r_off = *addr_of!(row_off);
    let w = *addr_of!(buf_width);
    let s = *addr_of!(stride);

    dest = buf.add(*r_off.add(start as usize) as usize);

    next_line = s - w;
    line = end - start + 1;

    while line != 0 {
        line -= 1;
        // very simple-minded fill loop
        i = w;
        while i != 0 {
            i -= 1;
            *dest = color;
            dest = dest.add(1);
        }
        dest = dest.add(next_line as usize);
    }
}

pub unsafe fn SetAlphaLines(alpha: u8, start: c_int, end: c_int) {
    //USE:	set the alpha value only
    //IN:		alpha	 				- 8-bit alpha value
    //			start through end		- line numbers
    //OUT:	none
    let mut dest: *mut CPixel32;
    let mut line: c_int;
    let mut i: c_int;
    let next_line: c_int;

    assert!(!(*addr_of!(buffer)).is_null());
    assert!(start >= 0);
    assert!(end < *addr_of!(buf_height));

    let buf = *addr_of!(buffer);
    let r_off = *addr_of!(row_off);
    let w = *addr_of!(buf_width);
    let s = *addr_of!(stride);

    dest = buf.add(*r_off.add(start as usize) as usize);

    next_line = s - w;
    line = end - start + 1;

    while line != 0 {
        line -= 1;
        // very simple-minded fill loop
        i = w;
        while i != 0 {
            i -= 1;
            (*dest).a = alpha;
            dest = dest.add(1);
        }
        dest = dest.add(next_line as usize);
    }
}

// Line clipping helper
fn code(x: c_int, y: c_int) -> c_int {
    //USE:	determines where a point is in relation to a bounding box
    //IN:		x,y		- coordinate pair
    //OUT:	clipping code compaired to global clip context
    let mut c: c_int = 0;

    unsafe {
        if x < *addr_of!(clip_min_x) {
            c |= LEFT;
        }
        if x > *addr_of!(clip_max_x) {
            c |= RIGHT;
        }
        if y < *addr_of!(clip_min_y) {
            c |= BOTTOM;
        }
        if y > *addr_of!(clip_max_y) {
            c |= TOP;
        }
    }

    c
}

pub unsafe fn ClipLine(x1: &mut c_int, y1: &mut c_int, x2: &mut c_int, y2: &mut c_int) -> bool {
    //USE:	clip a line from (x1,y1) to (x2,y2) to clip bounds
    //IN:		(x1,y1)-(x2,y2) line
    //OUT:	return true if something left to draw, otherwise false
    let mut c1: c_int;
    let mut c2: c_int;
    let mut c: c_int;
    let mut x: c_int;
    let mut y: c_int;
    let mut f: c_int;

    x = *x1;
    y = *y1;

    c1 = code(*x1, *y1); // find where first pt. is
    c2 = code(*x2, *y2); // find where second pt. is

    if (c1 & c2) == 0 {
        // the line may be visible
        while (c1 | c2) != 0 {
            // where there is 2D clipping to be done
            if c1 & c2 != 0 {
                return false;  // if both on same side, quit
            }

            c = c1;
            if c == 0 {
                c = c2; // pick a point
            }

            if (c & TOP) != 0 {
                f = ((*addr_of!(clip_max_y) - *y1) << 15) / (*y2 - *y1);
                x = *x1 + (((*x2 - *x1) * f + 16384) >> 15);
                y = *addr_of!(clip_max_y);
            } else if (c & BOTTOM) != 0 {
                f = ((*addr_of!(clip_min_y) - *y1) << 15) / (*y2 - *y1);
                x = *x1 + (((*x2 - *x1) * f + 16384) >> 15);
                y = *addr_of!(clip_min_y);
            } else if (c & LEFT) != 0 {
                f = ((*addr_of!(clip_min_x) - *x1) << 15) / (*x2 - *x1);
                y = *y1 + (((*y2 - *y1) * f + 16384) >> 15);
                x = *addr_of!(clip_min_x);
            } else if (c & RIGHT) != 0 {
                f = ((*addr_of!(clip_max_x) - *x1) << 15) / (*x2 - *x1);
                y = *y1 + (((*y2 - *y1) * f + 16384) >> 15);
                x = *addr_of!(clip_max_x);
            }

            if c == c1 {
                *x1 = x;
                *y1 = y;
                c1 = code(*x1, *y1);
            } else {
                *x2 = x;
                *y2 = y;
                c2 = code(*x2, *y2);
            }
        } // while still needs clipping
    } else {
        // line not visible
        return false;
    }
    true
}

pub unsafe fn DrawLineNC(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
    //USE:	draw a line from (x1,y1) to (x2,y2) in color (no clip)
    //IN:		(x1,y1)			- starting coordinate
    //			(x2,y2)			- ending coordinate
    //			color			- 32-bit color value
    //OUT:	none
    let mut d: c_int;
    let mut ax: c_int;
    let mut ay: c_int;
    let mut sx: c_int;
    let mut sy: c_int;
    let mut dx: c_int;
    let mut dy: c_int;
    let mut dest: *mut CPixel32;
    let mut x1_mut = x1;
    let mut y1_mut = y1;
    let mut x2_mut = x2;
    let mut y2_mut = y2;

    assert!(!(*addr_of!(buffer)).is_null());

    let buf = *addr_of!(buffer);
    let r_off = *addr_of!(row_off);
    let s = *addr_of!(stride);

    dx = x2_mut - x1_mut;
    ax = ABS(dx) << 1;
    sx = SIGN(dx);
    dy = y2_mut - y1_mut;
    ay = ABS(dy) << 1;
    sy = SIGN(dy);

    if 255 == color.a as c_int {
        if dy == 0 {
            // horz line
            if dx >= 0 {
                dest = buf.add((*r_off.add(y1_mut as usize) + x1_mut) as usize);
                let mut i = dx + 1;
                while i != 0 {
                    i -= 1;
                    *dest = color;
                    dest = dest.add(1);
                }
            } else {
                dest = buf.add((*r_off.add(y1_mut as usize) + x1_mut + dx) as usize);
                let mut i = -dx + 1;
                while i != 0 {
                    i -= 1;
                    *dest = color;
                    dest = dest.add(1);
                }
            }
            return;
        }

        if dx == 0 {
            // vert line
            if dy >= 0 {
                dest = buf.add((*r_off.add(y1_mut as usize) + x1_mut) as usize);
                y2_mut = dy + 1;
            } else {
                dest = buf.add((*r_off.add(y2_mut as usize) + x1_mut) as usize);
                y2_mut = -dy + 1;
            }

            while y2_mut != 0 {
                y2_mut -= 1;
                *dest = color;
                dest = dest.add(s as usize);
            }
            return;
        }
    }

    // bressenham's algorithm
    if ax > ay {
        d = ay - (ax >> 1);
        while x1_mut != x2 {
            PutPixAlphaNC(x1_mut, y1_mut, color);

            if d >= 0 {
                y1_mut += sy;
                d -= ax;
            }
            x1_mut += sx;
            d += ay;
        }
    } else {
        d = ax - (ay >> 1);
        while y1_mut != y2 {
            PutPixAlphaNC(x1_mut, y1_mut, color);
            if d >= 0 {
                x1_mut += sx;
                d -= ay;
            }
            y1_mut += sy;
            d += ax;
        }
    }
    PutPixAlphaNC(x1_mut, y1_mut, color);
}

pub unsafe fn DrawLine(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
    let mut x1_mut = x1;
    let mut y1_mut = y1;
    let mut x2_mut = x2;
    let mut y2_mut = y2;
    if ClipLine(&mut x1_mut, &mut y1_mut, &mut x2_mut, &mut y2_mut) {
        DrawLineNC(x1_mut, y1_mut, x2_mut, y2_mut, color);
    }
}

pub unsafe fn DrawLineAveNC(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
    //USE:	draw a translucent line from (x1,y1) to (x2,y2) in color (no clip)
    //IN:		(x1,y1)			- starting coordinate
    //			(x2,y2)			- ending coordinate
    //			color 			- 32-bit color value
    //OUT:	none
    let mut d: c_int;
    let mut ax: c_int;
    let mut ay: c_int;
    let mut sx: c_int;
    let mut sy: c_int;
    let mut dx: c_int;
    let mut dy: c_int;
    let mut dest: *mut CPixel32;
    let mut x1_mut = x1;
    let mut y1_mut = y1;
    let mut x2_mut = x2;
    let mut y2_mut = y2;

    assert!(!(*addr_of!(buffer)).is_null());

    let buf = *addr_of!(buffer);
    let r_off = *addr_of!(row_off);
    let s = *addr_of!(stride);

    dx = x2_mut - x1_mut;
    ax = ABS(dx) << 1;
    sx = SIGN(dx);
    dy = y2_mut - y1_mut;
    ay = ABS(dy) << 1;
    sy = SIGN(dy);

    if dy == 0 {
        // horz line
        if dx >= 0 {
            dest = buf.add((*r_off.add(y1_mut as usize) + x1_mut) as usize);
            let mut i = dx + 1;
            while i != 0 {
                i -= 1;
                *dest = AVE_PIX(*dest, color);
                dest = dest.add(1);
            }
        } else {
            dest = buf.add((*r_off.add(y1_mut as usize) + x1_mut + dx) as usize);
            let mut i = -dx + 1;
            while i != 0 {
                i -= 1;
                *dest = AVE_PIX(*dest, color);
                dest = dest.add(1);
            }
        }
        return;
    }

    if dx == 0 {
        // vert line
        if dy >= 0 {
            dest = buf.add((*r_off.add(y1_mut as usize) + x1_mut) as usize);
            y2_mut = dy + 1;
        } else {
            dest = buf.add((*r_off.add(y2_mut as usize) + x1_mut) as usize);
            y2_mut = -dy + 1;
        }

        while y2_mut != 0 {
            y2_mut -= 1;
            *dest = AVE_PIX(*dest, color);
            dest = dest.add(s as usize);
        }
        return;
    }

    // bressenham's algorithm
    if ax > ay {
        d = ay - (ax >> 1);
        while x1_mut != x2 {
            PutPixAveNC(x1_mut, y1_mut, color);

            if d >= 0 {
                y1_mut += sy;
                d -= ax;
            }
            x1_mut += sx;
            d += ay;
        }
    } else {
        d = ax - (ay >> 1);
        while y1_mut != y2 {
            PutPixAveNC(x1_mut, y1_mut, color);
            if d >= 0 {
                x1_mut += sx;
                d -= ay;
            }
            y1_mut += sy;
            d += ax;
        }
    }
    PutPixAveNC(x1_mut, y1_mut, color);
}

pub unsafe fn DrawLineAve(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
    let mut x1_mut = x1;
    let mut y1_mut = y1;
    let mut x2_mut = x2;
    let mut y2_mut = y2;
    if ClipLine(&mut x1_mut, &mut y1_mut, &mut x2_mut, &mut y2_mut) {
        DrawLineAveNC(x1_mut, y1_mut, x2_mut, y2_mut, color);
    }
}

pub unsafe fn DrawLineAANC(mut x0: c_int, mut y0: c_int, x1: c_int, y1: c_int, color: CPixel32) {
    // Wu antialiased line drawer.
    //USE:	Function to draw an antialiased line from (x0,y0) to (x1,y1), using an
    //			antialiasing approach published by Xiaolin Wu in the July 1991 issue of
    //			Computer Graphics (SIGGRAPH proceedings).
    //
    //IN:		(x0,y0),(x1,y1) = line to draw
    //			color = 32-bit color
    //OUT:	none

    assert!(!(*addr_of!(buffer)).is_null());

    // Make sure the line runs top to bottom
    let (mut x0, mut y0, mut x1_final, mut y1_final) = if y0 > y1 {
        (x1, y1, x0, y0)
    } else {
        (x0, y0, x1, y1)
    };

    let mut DeltaX = x1_final - x0;
    let mut DeltaY = y1_final - y0;
    let XDir: c_int;

    // Draw the initial pixel, which is always exactly intersected by
    // the line and so needs no Alpha
    PutPixAlphaNC(x0, y0, color);

    if DeltaX >= 0 {
        XDir = 1;
    } else {
        XDir = -1;
        DeltaX = -DeltaX; // make DeltaX positive
    }

    // Special-case horizontal, vertical, and diagonal lines, which
    // require no Alpha because they go right through the center of
    // every pixel
    if DeltaY == 0 {
        // Horizontal line
        while DeltaX != 0 {
            DeltaX -= 1;
            x0 += XDir;
            PutPixAlphaNC(x0, y0, color);
        }
        return;
    }
    if DeltaX == 0 {
        // Vertical line
        loop {
            y0 += 1;
            PutPixAlphaNC(x0, y0, color);
            DeltaY -= 1;
            if DeltaY == 0 {
                break;
            }
        }
        return;
    }
    if DeltaX == DeltaY {
        // Diagonal line
        loop {
            x0 += XDir;
            y0 += 1;
            PutPixAlphaNC(x0, y0, color);
            DeltaY -= 1;
            if DeltaY == 0 {
                break;
            }
        }
        return;
    }

    // Line is not horizontal, diagonal, or vertical
    let mut ErrorAcc: u16 = 0;  // initialize the line error accumulator to 0

    // # of bits by which to shift ErrorAcc to get intensity level
    const IntensityShift: u16 = 16 - 8;

    // Is this an X-major or Y-major line?
    if DeltaY > DeltaX {
        // Y-major line; calculate 16-bit fixed-point fractional part of a
        // pixel that X advances each time Y advances 1 pixel, truncating the
        // result so that we won't overrun the endpoint along the X axis
        let ErrorAdj = ((DeltaX as u32) << 16) / (DeltaY as u32) as u16;

        // Draw all pixels other than the first and last
        while DeltaY > 1 {
            DeltaY -= 1;
            let ErrorAccTemp = ErrorAcc;   // remember currrent accumulated error
            ErrorAcc = ErrorAcc.wrapping_add(ErrorAdj);      // calculate error for next pixel
            if ErrorAcc <= ErrorAccTemp {
                // The error accumulator turned over, so advance the X coord
                x0 += XDir;
            }
            y0 += 1; // Y-major, so always advance Y
            // The IntensityBits most significant bits of ErrorAcc give us the
            // intensity Alpha for this pixel, and the complement of the
            // Alpha for the paired pixel
            let Alpha = (ErrorAcc >> IntensityShift) as c_int;
            let InvAlpha = 256 - Alpha;

            PutPixAlphaNC(x0, y0,
                ALPHA_PIX(GetPix(x0, y0), color, Alpha, InvAlpha));
            PutPixAlphaNC(x0 + XDir, y0,
                ALPHA_PIX(GetPix(x0 + XDir, y0), color, InvAlpha, Alpha));
        }
        // Draw the final pixel, which is always exactly intersected by the line
        // and so needs no Alpha
        PutPixAlphaNC(x1_final, y1_final, color);
        return;
    }
    // It's an X-major line; calculate 16-bit fixed-point fractional part of a
    // pixel that Y advances each time X advances 1 pixel, truncating the
    // result to avoid overrunning the endpoint along the X axis
    let ErrorAdj = ((DeltaY as u32) << 16) / (DeltaX as u32) as u16;
    // Draw all pixels other than the first and last
    while DeltaX > 1 {
        DeltaX -= 1;
        let ErrorAccTemp = ErrorAcc;   // remember currrent accumulated error
        ErrorAcc = ErrorAcc.wrapping_add(ErrorAdj);      // calculate error for next pixel
        if ErrorAcc <= ErrorAccTemp {
            // The error accumulator turned over, so advance the Y coord
            y0 += 1;
        }
        x0 += XDir; // X-major, so always advance X
        // The IntensityBits most significant bits of ErrorAcc give us the
        // intensity Alpha for this pixel, and the complement of the
        // Alpha for the paired pixel
        let Alpha = (ErrorAcc >> IntensityShift) as c_int;
        let InvAlpha = 256 - Alpha;
        PutPixAlphaNC(x0, y0,
            ALPHA_PIX(GetPix(x0, y0), color, Alpha, InvAlpha));
        PutPixAlphaNC(x0, y0 + 1,
            ALPHA_PIX(GetPix(x0, y0 + 1), color, InvAlpha, Alpha));
    }
    // Draw the final pixel, which is always exactly intersected by the line
    // and so needs no Alpha
    PutPixAlphaNC(x1_final, y1_final, color);
}

pub unsafe fn DrawLineAA(x1: c_int, y1: c_int, x2: c_int, y2: c_int, color: CPixel32) {
    let mut x1_mut = x1;
    let mut y1_mut = y1;
    let mut x2_mut = x2;
    let mut y2_mut = y2;
    if ClipLine(&mut x1_mut, &mut y1_mut, &mut x2_mut, &mut y2_mut) {
        DrawLineAANC(x1_mut, y1_mut, x2_mut, y2_mut, color);
    }
}

pub unsafe fn DrawRectNC(ulx: c_int, mut uly: c_int, width: c_int, mut height: c_int, color: CPixel32) {
    //USE:	draw rectangle in solid color, no clipping
    //IN:		(ulx,uly)			- coordinates of upper-left corner of rect
    //			width, height		- dimensions of rectangle
    //			color				- color value
    //OUT:	none

    assert!(!(*addr_of!(buffer)).is_null());
    assert!(ulx >= 0);
    assert!(uly >= 0);
    assert!(ulx + width < *addr_of!(buf_width));
    assert!(uly + height < *addr_of!(buf_height));

    if height < 1 || width < 1 {
        return;
    }

    while height != 0 {
        height -= 1;
        DrawLineNC(ulx, uly, ulx + width - 1, uly, color);
        uly += 1;
    }
}

pub unsafe fn DrawRect(ulx: c_int, mut uly: c_int, width: c_int, mut height: c_int, color: CPixel32) {
    //USE:	draw rectangle in solid color
    //IN:		(ulx,uly)			- coordinates of upper-left corner of rect
    //			width, height		- dimensions of rectangle
    //			color				- color value
    //OUT:	none
    assert!(!(*addr_of!(buffer)).is_null());

    if height < 1 || width < 1 {
        return;
    }

    while height != 0 {
        height -= 1;
        DrawLine(ulx, uly, ulx + width - 1, uly, color);
        uly += 1;
    }
}

pub unsafe fn DrawRectAve(ulx: c_int, mut uly: c_int, width: c_int, mut height: c_int, color: CPixel32) {
    //USE:	draw rectangle in solid color, translucent
    //IN:		(ulx,uly)			- coordinates of upper-left corner of rect
    //			width, height		- dimensions of rectangle
    //			color				- color value
    //OUT:	none
    assert!(!(*addr_of!(buffer)).is_null());

    if height < 1 || width < 1 {
        return;
    }

    while height != 0 {
        height -= 1;
        DrawLineAve(ulx, uly, ulx + width - 1, uly, color);
        uly += 1;
    }
}

pub unsafe fn DrawBoxNC(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
    //USE:	Draw an empty box, no clipping
    //IN:		(ulx,uly)		- coordinates of upper-left corner of box
    //			width, height	- dimensions of box
    //			color			- color value
    //OUT:	none
    assert!(!(*addr_of!(buffer)).is_null());

    if height < 1 || width < 1 {
        return;
    }

    DrawLineNC(ulx, uly, ulx + width - 1, uly, color);
    DrawLineNC(ulx, uly + height - 1, ulx + width - 1, uly + height - 1, color);
    DrawLineNC(ulx, uly, ulx, uly + height - 1, color);
    DrawLineNC(ulx + width - 1, uly, ulx + width - 1, uly + height - 1, color);
}

pub unsafe fn DrawBox(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
    //USE:	Draw an empty box
    //IN:		(ulx,uly)		- coordinates of upper-left corner of rect
    //			width, height	- dimensions of rectangle
    //			color			- color value
    //OUT:	none
    assert!(!(*addr_of!(buffer)).is_null());

    if height < 1 || width < 1 {
        return;
    }

    DrawLine(ulx, uly, ulx + width - 1, uly, color);
    DrawLine(ulx, uly + height - 1, ulx + width - 1, uly + height - 1, color);
    DrawLine(ulx, uly, ulx, uly + height - 1, color);
    DrawLine(ulx + width - 1, uly, ulx + width - 1, uly + height - 1, color);
}

pub unsafe fn DrawBoxAve(ulx: c_int, uly: c_int, width: c_int, height: c_int, color: CPixel32) {
    //USE: Draw an empty box, translucent
    //IN:		(ulx,uly)		- coordinates of upper-left corner of rect
    //			width, height	- dimensions of rectangle
    //			color			- color value
    //OUT:	none
    assert!(!(*addr_of!(buffer)).is_null());

    if height < 1 || width < 1 {
        return;
    }

    DrawLineAve(ulx, uly, ulx + width - 1, uly, color);
    DrawLineAve(ulx, uly + height - 1, ulx + width - 1, uly + height - 1, color);
    DrawLineAve(ulx, uly, ulx, uly + height - 1, color);
    DrawLineAve(ulx + width - 1, uly, ulx + width - 1, uly + height - 1, color);
}

pub unsafe fn DrawCircle(xc: c_int, yc: c_int, r: c_int, edge: CPixel32, fill: CPixel32) {
    //USE:	Draw a simple circle in current color with Bresenham's
    //		circle algorithm.
    // See PROCEDURAL ELEMENTS FOR COMPUTER GRAPHICS
    // David F. Rogers Pg. 48.
    //
    //IN:   xc,yc - center
    //      r     - radius
    //      edge  - edge color
    //      fill  - fill color
    //
    //OUT: NONE (a circle drawn in the off-screen buffer)
    let mut x: c_int;
    let mut y: c_int;
    let limit: c_int;
    let mut di: c_int;
    let mut delta: c_int;
    let mut last_x: c_int;
    let mut last_y: c_int;

    assert!(!(*addr_of!(buffer)).is_null());

    if r < 1 {
        return;
    }

    // draw fill
    if fill.a != 0 {
        x = 0;
        last_x = x;
        y = r;
        last_y = y;
        di = 2 * (1 - r);
        let limit = 0;

        loop {
            if y >= limit {
                if di < 0 {
                    delta = 2 * di + 2 * y - 1;
                    if delta <= 0 {
                        // move horizontal
                        last_x = x;
                        x += 1;
                        di += 2 * x + 1;
                    } else {
                        // move diagonal
                        last_x = x;
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                } else {
                    if di > 0 {
                        delta = 2 * di - 2 * x - 1;
                        if delta <= 0 {
                            // move diagonal
                            last_x = x;
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        } else {
                            // move vertical
                            y -= 1;
                            di += 1 - 2 * y;
                        }
                    } else {
                        // di = 0 - move diagonal
                        last_x = x;
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                }
            }

            if y != last_y {
                // circle fill
                DrawLine(xc - last_x, yc + last_y, xc + last_x, yc + last_y, fill);
                if last_y > limit {
                    DrawLine(xc - last_x, yc - last_y, xc + last_x, yc - last_y, fill);
                }
                last_y = y;
            }
            if y < limit {
                break;
            }
        }
    }

    // draw edge
    if edge.a != 0 {
        x = 0;
        y = r;
        let limit = 0;
        di = 2 * (1 - r);

        loop {
            // circle edge
            PutPix(xc + x, yc + y, edge);
            PutPix(xc - x, yc + y, edge);
            if y > limit {
                PutPix(xc + x, yc - y, edge);
                PutPix(xc - x, yc - y, edge);
            }

            if y >= limit {
                if di < 0 {
                    delta = 2 * di + 2 * y - 1;
                    if delta <= 0 {
                        // move horizontal
                        x += 1;
                        di += 2 * x + 1;
                    } else {
                        // move diagonal
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                } else {
                    if di > 0 {
                        delta = 2 * di - 2 * x - 1;
                        if delta <= 0 {
                            // move diagonal
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        } else {
                            // move vertical
                            y -= 1;
                            di += 1 - 2 * y;
                        }
                    } else {
                        // di = 0 - move diagonal
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                }
            }
            if y < limit {
                break;
            }
        }
    }
} // DrawCircle

pub unsafe fn DrawCircleAve(xc: c_int, yc: c_int, r: c_int, edge: CPixel32, fill: CPixel32) {
    //USE:	Draw a simple circle in current color with Bresenham's
    //			circle algorithm.
    //			a circle with fill and edge colors averaged with dest (-1 = no color)
    // See PROCEDURAL ELEMENTS FOR COMPUTER GRAPHICS
    // David F. Rogers Pg. 48.
    //
    //IN:   xc,yc - center
    //      r     - radius
    //      edge  - edge color
    //      fill  - fill color
    //
    //OUT:	none (a circle on in the off-screen buffer)
    let mut x: c_int;
    let mut y: c_int;
    let limit: c_int;
    let mut di: c_int;
    let mut delta: c_int;
    let mut last_x: c_int;
    let mut last_y: c_int;
    let mut f: c_int;

    assert!(!(*addr_of!(buffer)).is_null());

    if r < 1 {
        return;
    }

    // draw fill
    if fill.a != 0 {
        x = 0;
        last_x = x;
        y = r;
        last_y = y;
        di = 2 * (1 - r);
        let limit = 0;

        loop {
            if y >= limit {
                if di < 0 {
                    delta = 2 * di + 2 * y - 1;
                    if delta <= 0 {
                        // move horizontal
                        last_x = x;
                        x += 1;
                        di += 2 * x + 1;
                    } else {
                        // move diagonal
                        last_x = x;
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                } else {
                    if di > 0 {
                        delta = 2 * di - 2 * x - 1;
                        if delta <= 0 {
                            // move diagonal
                            last_x = x;
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        } else {
                            // move vertical
                            y -= 1;
                            di += 1 - 2 * y;
                        }
                    } else {
                        // di = 0 - move diagonal
                        last_x = x;
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                }
            }

            if y != last_y {
                // circle fill
                f = xc - last_x;
                while f <= xc + last_x {
                    PutPixAve(f, yc + last_y, fill);
                    f += 1;
                }
                if last_y > limit {
                    f = xc - last_x;
                    while f <= xc + last_x {
                        PutPixAve(f, yc - last_y, fill);
                        f += 1;
                    }
                }
                last_y = y;
            }
            if y < limit {
                break;
            }
        }
    }

    // draw edge
    if edge.a != 0 {
        x = 0;
        y = r;
        let limit = 0;
        di = 2 * (1 - r);

        loop {
            // circle edge
            PutPixAve(xc + x, yc + y, edge);
            PutPixAve(xc - x, yc + y, edge);
            if y > limit {
                PutPixAve(xc + x, yc - y, edge);
                PutPixAve(xc - x, yc - y, edge);
            }
            if y >= limit {
                if di < 0 {
                    delta = 2 * di + 2 * y - 1;
                    if delta <= 0 {
                        // move horizontal
                        x += 1;
                        di += 2 * x + 1;
                    } else {
                        // move diagonal
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                } else {
                    if di > 0 {
                        delta = 2 * di - 2 * x - 1;
                        if delta <= 0 {
                            // move diagonal
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        } else {
                            // move vertical
                            y -= 1;
                            di += 1 - 2 * y;
                        }
                    } else {
                        // di = 0 - move diagonal
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                }
            }
            if y < limit {
                break;
            }
        }
    }
} // DrawCircleAve

////////////////////////////////////////////////////////////////////////////
//Concave Polygon Scan Conversion
////////////////////////////////////////////////////////////////////////////

// concave: scan convert nvert-sided concave non-simple polygon
// with vertices at (point[i].x, point[i].y) for i in
// [0..nvert-1] within the window win by
// calling spanproc for each visible span of pixels.
//
// Polygon can be clockwise or counterclockwise.
//
// Algorithm does uniform point sampling at pixel centers.
// Inside-outside test done by even-odd rule: a point is
// considered inside if an emanating ray intersects the polygon
// an odd number of times.
//
// spanproc should fill in pixels from xl to xr inclusive on scanline y,
//
// e.g:
//	spanproc(short y, short xl, short xr)
//	{
//	    short x;
//	    for (x=xl; x<=xr; x++)
//			pixel_write(x, y, pixelvalue);
//	}

// global for speed
static mut n: c_int = 0;               // number of vertices
static mut pt: *mut POINT = core::ptr::null_mut(); // vertices
static mut nact: c_int = 0;             // number of active edges
static mut active: [POLYEDGE; 256] = [POLYEDGE { x: 0, dx: 0, i: 0 }; 256]; // active edge list:edges crossing scanline y
static mut ind: [c_int; 256] = [0; 256]; // list of vertex indices, sorted by pt[ind[j]].y

fn del_edge(i: c_int) {
    // remove edge i from active list
    let mut j: c_int = 0;

    unsafe {
        while j < *addr_of!(nact) && (*addr_of!(active)).as_ref()[j as usize].i != i {
            j += 1;
        }

        // edge not in active list; happens at cliprect->top
        if j >= *addr_of!(nact) {
            return;
        }

        *addr_of_mut!(nact) -= 1;
        let src = (*addr_of!(active)).as_ptr().add((j + 1) as usize);
        let dst = (*addr_of_mut!(active)).as_mut_ptr().add(j as usize);
        copy_nonoverlapping(src, dst, (*addr_of!(nact - j)) as usize * mem::size_of::<POLYEDGE>());
    }
}

fn ins_edge(i: c_int, y: c_int) {
    // append edge i to end of active list
    let mut j: c_int;
    let dx: c_int;
    let p: *mut POINT;
    let q: *mut POINT;

    unsafe {
        let n_val = *addr_of!(n);
        j = if i < n_val - 1 { i + 1 } else { 0 };
        let pt_val = *addr_of!(pt);
        if (*pt_val.add(i as usize)).y < (*pt_val.add(j as usize)).y {
            p = pt_val.add(i as usize);
            q = pt_val.add(j as usize);
        } else {
            p = pt_val.add(j as usize);
            q = pt_val.add(i as usize);
        }

        // initialize x position at intersection of edge with scanline y
        if ((*q).y - (*p).y) != 0 {
            dx = ((((*q).x - (*p).x) as c_int) * (1 << INT_SHIFT)) / ((*q).y - (*p).y);
        } else {
            // horizontal line
            dx = 0;
        }

        let active_ptr = (*addr_of_mut!(active)).as_mut_ptr();
        let nact_val = *addr_of!(nact);
        (*active_ptr.add(nact_val as usize)).dx = dx;
        (*active_ptr.add(nact_val as usize)).x = (dx * (y - (*p).y)) + ((*p).x << INT_SHIFT);
        (*active_ptr.add(nact_val as usize)).i = i;
        *addr_of_mut!(nact) += 1;
    }
}


fn shell_sort_polyedge(vec: *mut POLYEDGE, n: c_int) {
    // USE:  shell sort aka heap sort. Best sort algorithm for almost sorted list.
    unsafe {
        let mut h: c_int = 1;

        // choose size of "heap"
        while h <= n / 9 {
            h = 3 * h + 1;
        }

        // divide and conq.
        while h > 0 {
            let mut i = h;
            while i < n {
                let temp = *vec.add(i as usize);
                let mut j = i;
                // j >= h && vec[j-h] > temp
                while j >= h && (*vec.add((j - h) as usize)).x > temp.x {
                    *vec.add(j as usize) = *vec.add((j - h) as usize);
                    j -= h;
                }
                *vec.add(j as usize) = temp;
                i += 1;
            }
            h /= 3;
        }
    }
}

fn shell_sort_ind(vec: *mut c_int, n: c_int) {
    // USE:  shell sort for index array
    unsafe {
        let mut h: c_int = 1;

        // choose size of "heap"
        while h <= n / 9 {
            h = 3 * h + 1;
        }

        // divide and conq.
        while h > 0 {
            let mut i = h;
            while i < n {
                let temp = *vec.add(i as usize);
                let pt_val = *addr_of!(pt);
                let temp_y = (*pt_val.add(temp as usize)).y;
                let mut j = i;
                // j >= h && pt[vec[j-h]].y > pt[temp].y
                while j >= h {
                    let prev_idx = *vec.add((j - h) as usize);
                    if (*pt_val.add(prev_idx as usize)).y <= temp_y {
                        break;
                    }
                    *vec.add(j as usize) = prev_idx;
                    j -= h;
                }
                *vec.add(j as usize) = temp;
                i += 1;
            }
            h /= 3;
        }
    }
}

pub unsafe fn DrawPolygon(nvert: c_int, point: *mut POINT, edge: CPixel32, fill: CPixel32) {
    //USE:    Scan convert a polygon
    //IN:     nvert:        Number of vertices
    //        point:        Vertices of polygon
    //        edge:         edge color
    //        fill:         fill color
    //OUT:		none
    let mut k: c_int;
    let mut y0: c_int;
    let mut y1: c_int;
    let mut y: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut xl: c_int;
    let mut xr: c_int;

    assert!(!(*addr_of!(buffer)).is_null());

    *addr_of_mut!(n) = nvert;

    if *addr_of!(n) <= 0 {
        // nothing to do
        return;
    }

    *addr_of_mut!(pt) = point;

    if fill.a != 0 {
        // draw fill

        // create y-sorted array of indices ind[k] into vertex list
        k = 0;
        while k < *addr_of!(n) {
            (*addr_of_mut!(ind)).as_mut()[k as usize] = k;
            k += 1;
        }

        // sort ind by pt[ind[k]].y
        shell_sort_ind((*addr_of_mut!(ind)).as_mut_ptr(), *addr_of!(n));

        *addr_of_mut!(nact) = 0;                     // start with empty active list
        k = 0;                                       // ind[k] is next vertex to process

        let pt_val = *addr_of!(pt);
        // ymin of polygon
        y0 = MAX(*addr_of!(clip_min_y) - 1, (*pt_val.add((*addr_of_mut!(ind)).as_ref()[0] as usize)).y);

        // ymax of polygon
        y1 = MIN(*addr_of!(clip_max_y) + 1, (*pt_val.add((*addr_of_mut!(ind)).as_ref()[(*addr_of!(n) - 1) as usize] as usize)).y);

        // step through scanlines
        y = y0;
        while y < y1 {
            // Check vertices between previous scanline
            // and current one, if any
            let n_val = *addr_of!(n);
            while k < n_val && (*pt_val.add((*addr_of_mut!(ind)).as_ref()[k as usize] as usize)).y <= y {
                i = (*addr_of_mut!(ind)).as_ref()[k as usize];
                //  insert or delete edges before and after vertex i
                //  (i-1 to i, and i to i+1)
                //  from active list if they cross scanline y
                j = if i > 0 { i - 1 } else { n_val - 1 }; // vertex previous to i
                if (*pt_val.add(j as usize)).y < y {
                    // old edge, remove from active list
                    del_edge(j);
                } else {
                    if (*pt_val.add(j as usize)).y > y {
                        // new edge, add to active list
                        ins_edge(j, y);
                    }
                }
                j = if i < n_val - 1 { i + 1 } else { 0 }; // vertex next after i
                if (*pt_val.add(j as usize)).y < y {
                    // old edge, remove from active list
                    del_edge(i);
                } else {
                    if (*pt_val.add(j as usize)).y > y {
                        // new edge, add to active list
                        ins_edge(i, y);
                    }
                }
                k += 1;
            }

            // sort active edge list by active[j].x
            shell_sort_polyedge((*addr_of_mut!(active)).as_mut_ptr(), *addr_of!(nact));

            // draw horizontal segments for scanline y
            j = 0;
            while j < *addr_of!(nact) {
                // draw horizontal segments
                // span 'tween j & j+1 is inside, span tween
                // j+1 & j+2 is outside

                // left end of span
                // convert back from fixed point - round down
                xl = (*addr_of!(active)).as_ref()[j as usize].x >> INT_SHIFT;
                if xl < *addr_of!(clip_min_x) - 1 {
                    xl = *addr_of!(clip_min_x) - 1;
                }

                // right end of span
                // convert back from fixed point - round down
                xr = (*addr_of!(active)).as_ref()[(j + 1) as usize].x >> INT_SHIFT;
                if xr > *addr_of!(clip_max_x) {
                    xr = *addr_of!(clip_max_x);
                }

                if xl < xr {
                    // draw pixels in span
                    DrawLine(xl + 1, y, xr, y, fill);
                }

                // increment edge coords
                (*addr_of_mut!(active)).as_mut()[j as usize].x += (*addr_of!(active)).as_ref()[j as usize].dx;
                (*addr_of_mut!(active)).as_mut()[(j + 1) as usize].x += (*addr_of!(active)).as_ref()[(j + 1) as usize].dx;

                j += 2;
            }

            y += 1;
        }

        if edge.a != 0 {
            // draw edges
            k = 0;
            let n_val = *addr_of!(n);
            let pt_val = *addr_of!(pt);
            while k < n_val - 1 {
                DrawLineAA((*pt_val.add(k as usize)).x, (*pt_val.add(k as usize)).y,
                           (*pt_val.add((k + 1) as usize)).x, (*pt_val.add((k + 1) as usize)).y, edge);
                k += 1;
            }

            DrawLineAA((*pt_val.add((n_val - 1) as usize)).x, (*pt_val.add((n_val - 1) as usize)).y,
                       (*pt_val.add(0)).x, (*pt_val.add(0)).y, edge);
        }

        return;
    }
}

pub unsafe fn Blit(dstX: c_int, dstY: c_int, width: c_int, height: c_int,
                   srcImage: *mut CPixel32, srcX: c_int, srcY: c_int, srcStride: c_int) {
    //USE:	simple blit
    //IN:	dstX, dstY				- upper left corner of where image will land in buffer
    //		width, height			- width and height of image
    //		srcImage				- src image buffer
    //		srcX, srcY				- upper left corner in src image
    //		srcStride				- number of pixels per line in src image
    //OUT:	none
    let mut dst: *mut CPixel32;
    let mut src: *mut CPixel32;
    let mut x: c_int;
    let mut y: c_int;

    assert!(!(*addr_of!(buffer)).is_null());

    let mut d_dstX = dstX;
    let mut d_dstY = dstY;
    let mut d_width = width;
    let mut d_height = height;
    let mut d_srcX = srcX;
    let mut d_srcY = srcY;

    BlitClip(&mut d_dstX, &mut d_dstY, &mut d_width, &mut d_height, &mut d_srcX, &mut d_srcY);

    if d_width < 1 || d_height < 1 {
        return;
    }

    let buf = *addr_of!(buffer);
    let s = *addr_of!(stride);

    dst = buf.add(PIXPOS(d_dstX, d_dstY, s) as usize);
    src = srcImage.add(PIXPOS(d_srcX, d_srcY, srcStride) as usize);

    y = 0;
    while y < d_height {
        x = 0;
        while x < d_width {
            // *dst++ = *src++;
            let alpha = (*src).a as c_int;
            let dst_alpha = (*dst).a;
            *dst = ALPHA_PIX(*src, *dst, alpha, 256 - alpha);
            (*dst).a = dst_alpha;
            dst = dst.add(1);
            src = src.add(1);
            x += 1;
        }
        dst = dst.add((s - d_width) as usize);
        src = src.add((srcStride - d_width) as usize);
        y += 1;
    }
}

pub unsafe fn BlitClip(dstX: &mut c_int, dstY: &mut c_int,
                       width: &mut c_int, height: &mut c_int,
                       srcX: &mut c_int, srcY: &mut c_int) {
    //USE:	simple blit clip
    //IN:	dstX, dstY				- upper left corner of where image will land in buffer
    //		width, height			- width and height of image
    //		srcX, srcY				- upper left corner in src image
    //OUT:	none

    // clip to our buffer size
    if *dstX < *addr_of!(clip_min_x) {
        let dif = *addr_of!(clip_min_x) - *dstX;
        *dstX += dif;
        *srcX += dif;
        *width -= dif;
    }

    if *dstY < *addr_of!(clip_min_y) {
        let dif = *addr_of!(clip_min_y) - *dstY;
        *dstY += dif;
        *srcY += dif;
        *height -= dif;
    }

    if *dstX + *width - 1 > *addr_of!(clip_max_x) {
        *width -= *dstX + *width - 1 - *addr_of!(clip_max_x);
    }

    if *dstY + *height - 1 > *addr_of!(clip_max_y) {
        *height -= *dstY + *height - 1 - *addr_of!(clip_max_y);
    }
}

pub unsafe fn BlitColor(dstX: c_int, dstY: c_int, width: c_int, height: c_int,
                        srcImage: *mut CPixel32, srcX: c_int, srcY: c_int, srcStride: c_int, color: CPixel32) {
    //USE:	blit using image alpha as mask
    //IN:	dstX, dstY				- upper left corner of where image will land in buffer
    //		width, height			- width and height of image
    //		srcImage				- src image buffer
    //		srcX, srcY				- upper left corner in src image
    //		srcStride				- number of pixels per line in src image
    //		color					- color to apply to srcImage
    //OUT:	none
    let mut dst: *mut CPixel32;
    let mut src: *mut CPixel32;
    let mut x: c_int;
    let mut y: c_int;
    let dstNextLine: c_int;
    let srcNextLine: c_int;

    assert!(!(*addr_of!(buffer)).is_null());

    let mut d_dstX = dstX;
    let mut d_dstY = dstY;
    let mut d_width = width;
    let mut d_height = height;
    let mut d_srcX = srcX;
    let mut d_srcY = srcY;

    BlitClip(&mut d_dstX, &mut d_dstY, &mut d_width, &mut d_height, &mut d_srcX, &mut d_srcY);

    if d_width < 1 || d_height < 1 {
        return;
    }

    let buf = *addr_of!(buffer);
    let s = *addr_of!(stride);

    dst = buf.add(PIXPOS(d_dstX, d_dstY, s) as usize);
    src = srcImage.add(PIXPOS(d_srcX, d_srcY, srcStride) as usize);

    dstNextLine = s - d_width;
    srcNextLine = srcStride - d_width;

    y = 0;
    while y < d_height {
        x = 0;
        while x < d_width {
            let alpha = (*src).a as c_int;
            *dst = ALPHA_PIX(color, *dst, alpha, 256 - alpha);
            dst = dst.add(1);
            src = src.add(1);
            x += 1;
        }
        dst = dst.add(dstNextLine as usize);
        src = src.add(srcNextLine as usize);
        y += 1;
    }
}

// Extern libc declarations
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

// Re-export libc functions for use in this module
mod libc {
    use core::ffi::c_void;
    pub unsafe fn malloc(size: usize) -> *mut c_void {
        extern "C" {
            fn malloc(size: usize) -> *mut c_void;
        }
        malloc(size)
    }

    pub unsafe fn free(ptr: *mut c_void) {
        extern "C" {
            fn free(ptr: *mut c_void);
        }
        free(ptr)
    }
}
