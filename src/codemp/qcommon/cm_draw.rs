///////////////////////////////////////////////////////////////////////////////
// CDraw32 Class Implementation
//
// Basic drawing routines for 32-bit buffer
///////////////////////////////////////////////////////////////////////////////

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::byte;
use crate::codemp::qcommon::cm_draw_h::*;
use core::ffi::c_long;
use core::ptr::{addr_of, addr_of_mut};

// Image kernel for embossing
pub static IMG_KERNEL: [[i32; 5]; 5] = [
    [-1, -1, -1, -1, 0],
    [-1, -1, -1, 0, 1],
    [-1, -1, 0, 1, 1],
    [-1, 0, 1, 1, 1],
    [0, 1, 1, 1, 1],
];

// globals for polygon scan conversion
static mut n: i32 = 0;                          // number of vertices
static mut pt: *mut POINT = core::ptr::null_mut();  // vertices
static mut nact: i32 = 0;                       // number of active edges
static mut active: [POLYEDGE; 256] = unsafe { core::mem::zeroed() }; // active edge list: edges crossing scanline y
static mut ind: [i32; 256] = [0; 256];          // list of vertex indices, sorted by pt[ind[j]].y

#[repr(C)]
pub struct POLYEDGE {
    pub x: i32,  // x coordinate of edge's intersection with current scanline
    pub dx: i32, // change in x with respect to y
    pub i: i32,  // edge number: edge i goes from pt[i] to pt[i+1]
}

const INT_SHIFT: i32 = 13;

// Clipping codes
const LEFT: i32 = 1;   // code bits
const RIGHT: i32 = 2;
const TOP: i32 = 4;
const BOTTOM: i32 = 8;

// Helper functions for polygon rendering
fn del_edge(i: i32) {
    // remove edge i from active list
    let mut j: i32 = 0;

    unsafe {
        while j < nact && active[j as usize].i != i {
            j += 1;
        }

        // edge not in active list; happens at cliprect->top
        if j >= nact {
            return;
        }

        nact -= 1;
        core::ptr::copy_nonoverlapping(
            &active[(j + 1) as usize] as *const POLYEDGE,
            &mut active[j as usize] as *mut POLYEDGE,
            (nact - j) as usize,
        );
    }
}

fn ins_edge(i: i32, y: i32) {
    // append edge i to end of active list
    let mut j: i32;
    let mut dx: i32;
    let mut p: *mut POINT;
    let mut q: *mut POINT;

    unsafe {
        j = if i < n - 1 { i + 1 } else { 0 };
        if (*pt.offset(i as isize)).y < (*pt.offset(j as isize)).y {
            p = &mut (*pt.offset(i as isize)) as *mut POINT;
            q = &mut (*pt.offset(j as isize)) as *mut POINT;
        } else {
            p = &mut (*pt.offset(j as isize)) as *mut POINT;
            q = &mut (*pt.offset(i as isize)) as *mut POINT;
        }

        // initialize x position at intersection of edge with scanline y
        if ((*q).y - (*p).y) != 0 {
            dx = ((((*q).x - (*p).x) as i32 * (1i32 << INT_SHIFT)) as i32);
            dx /= (((*q).y - (*p).y) as i32);
        } else {
            // horizontal line
            dx = 0;
        }

        active[nact as usize].dx = dx;
        active[nact as usize].x = (dx * (y - (*p).y)) + ((*p).x << INT_SHIFT);
        active[nact as usize].i = i;
        nact += 1;
    }
}

// comparison routines for shellsort
fn compare_ind(u: *const i32, v: *const i32) -> i32 {
    unsafe { if (*pt.offset(*u as isize)).y <= (*pt.offset(*v as isize)).y { -1 } else { 1 } }
}

fn compare_active(u: *const POLYEDGE, v: *const POLYEDGE) -> i32 {
    unsafe {
        if (*u).x <= (*v).x {
            -1
        } else {
            1
        }
    }
}

fn shell_sort(
    vec: *mut core::ffi::c_void,
    n_sort: i32,
    siz: i32,
    compare: fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32,
) {
    // USE:  shell sort aka heap sort. Best sort algorithm for almost sorted list.
    let mut a: *mut u8 = vec as *mut u8;
    let mut v: [u8; 128] = [0; 128]; // temp object
    let mut i: i32;
    let mut j: i32;
    let mut h: i32;

    // choose size of "heap"
    h = 1;
    while h <= n_sort / 9 {
        h = 3 * h + 1;
    }

    // divide and conq.
    while h > 0 {
        i = h;
        while i < n_sort {
            // v = a[i];
            unsafe {
                core::ptr::copy_nonoverlapping(
                    a.offset((i as isize) * siz as isize),
                    v.as_mut_ptr(),
                    siz as usize,
                );
            }
            j = i;
            // j >= h && a[j-h] > v
            while j >= h
                && compare(
                    unsafe { a.offset(((j - h) as isize) * siz as isize) } as *const core::ffi::c_void,
                    v.as_ptr() as *const core::ffi::c_void,
                ) > 0
            {
                // a[j] = a[j-h]
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        a.offset(((j - h) as isize) * siz as isize),
                        a.offset((j as isize) * siz as isize),
                        siz as usize,
                    );
                }
                j -= h;
            }
            // a[j] = v;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    v.as_ptr(),
                    a.offset((j as isize) * siz as isize),
                    siz as usize,
                );
            }
            i += 1;
        }
        h /= 3;
    }
}

impl CDraw32 {
    // CDraw32() - constructor
    // USE: constructor
    pub fn new() -> Self {
        CDraw32 {
            _private: [],
        }
    }

    // ~CDraw32() - destructor
    // USE: Destructor
    pub fn drop(&mut self) {
        // Cleanup is handled by CleanUp() method
    }

    pub unsafe fn Emboss(
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
        let mut dst: *mut CPixel32;
        let mut clr: *mut CPixel32;
        let mut x: i32;
        let mut y: i32;
        let mut i: i32;
        let mut j: i32;
        let mut dstNextLine: i32;
        let mut clrNextLine: i32;
        let mut dstX = dstX;
        let mut dstY = dstY;
        let mut width = width;
        let mut height = height;
        let mut clrX = clrX;
        let mut clrY = clrY;

        assert!(!(*addr_of!(buffer)).is_null());

        self.BlitClip(&mut dstX, &mut dstY, &mut width, &mut height, &mut clrX, &mut clrY);

        if width < 1 || height < 1 {
            return;
        }

        dst = (*addr_of!(buffer)).offset(PIXPOS(dstX, dstY, *addr_of!(stride)) as isize);
        clr = clrImage.offset(PIXPOS(clrX, clrY, clrStride) as isize);

        dstNextLine = (*addr_of!(stride) - width) as i32;
        clrNextLine = (clrStride - width) as i32;

        y = 0;
        while y < height as i32 {
            x = 0;
            while x < width as i32 {
                let mut accum: i32 = 0;
                j = -KWIDTH;
                while j <= KWIDTH {
                    i = -KWIDTH;
                    while i <= KWIDTH {
                        let xk = CLAMP(x + i, clrX as i32, (clrX + width - 1) as i32);
                        let yk = CLAMP(y + j, clrY as i32, (clrY + height - 1) as i32);
                        accum += (*clrImage
                            .offset(PIXPOS(xk as c_long, yk as c_long, clrStride) as isize))
                        .a as i32
                            * IMG_KERNEL[(j + KWIDTH) as usize][(i + KWIDTH) as usize];
                        i += 1;
                    }
                    j += 1;
                }
                *dst = LIGHT_PIX(*clr, accum as c_long);
                (*dst).a = 255;
                dst = dst.offset(1);
                clr = clr.offset(1);
                x += 1;
            }
            dst = dst.offset(dstNextLine as isize);
            clr = clr.offset(clrNextLine as isize);
            y += 1;
        }
    }

    pub unsafe fn SetBufferSize(&mut self, width: c_long, height: c_long, stride_len: c_long) -> bool {
        // USE: setup for a particular size drawing buffer
        // (do not re-setup if buffer size has not changed)
        // IN: width,height - size of buffer
        // stride_len - distance to next line
        // OUT: true if everything goes OK, otherwise false
        let mut i: i32;

        assert!(width != 0);
        assert!(height != 0);
        assert!(stride_len != 0);

        if *addr_of!(buf_width) != width || *addr_of!(buf_height) != height || stride_len != *addr_of!(stride)
        {
            // need to re-create row_off table
            *addr_of_mut!(buf_width) = width;
            *addr_of_mut!(buf_height) = height;
            *addr_of_mut!(stride) = stride_len;

            if !(*addr_of!(row_off)).is_null() {
                libc::free(*addr_of!(row_off) as *mut libc::c_void);
            }

            // row offsets used for quick pixel address calcs
            *addr_of_mut!(row_off) = libc::malloc((height as usize) * core::mem::size_of::<i32>())
                as *mut i32;

            assert!(!(*addr_of!(row_off)).is_null());
            if (*addr_of!(row_off)).is_null() {
                return false;
            }

            // table for quick pixel lookups
            i = 0;
            while i < height as i32 {
                *(*addr_of_mut!(row_off)).offset(i as isize) = i as c_long * stride_len;
                i += 1;
            }
        }
        // set default clip bounds
        self.SetClip(0, 0, width - 1, height - 1);

        true
    }

    pub unsafe fn ClearLines(&mut self, color: CPixel32, start: c_long, end: c_long) {
        // USE: clear screen buffer to color provided for lines
        // IN: color - 32-bit color value
        // start through end - line numbers
        // OUT: none
        let mut dest: *mut CPixel32;
        let mut line: i32;
        let mut i: i32;
        let next_line: i32;

        assert!(!(*addr_of!(buffer)).is_null());
        assert!(start >= 0);
        assert!(end < *addr_of!(buf_height));

        dest = (*addr_of!(buffer)).offset(*(*addr_of!(row_off)).offset(start as isize) as isize);

        next_line = (*addr_of!(stride) - *addr_of!(buf_width)) as i32;
        line = (end - start + 1) as i32;

        while line != 0 {
            line -= 1;
            // very simple-minded fill loop
            i = *addr_of!(buf_width) as i32;
            while i != 0 {
                i -= 1;
                *dest = color;
                dest = dest.offset(1);
            }
            dest = dest.offset(next_line as isize);
        }
    }

    pub unsafe fn SetAlphaLines(&mut self, alpha: byte, start: c_long, end: c_long) {
        // USE: set the alpha value only
        // IN: alpha - 8-bit alpha value
        // start through end - line numbers
        // OUT: none
        let mut dest: *mut CPixel32;
        let mut line: i32;
        let mut i: i32;
        let next_line: i32;

        assert!(!(*addr_of!(buffer)).is_null());
        assert!(start >= 0);
        assert!(end < *addr_of!(buf_height));

        dest = (*addr_of!(buffer)).offset(*(*addr_of!(row_off)).offset(start as isize) as isize);

        next_line = (*addr_of!(stride) - *addr_of!(buf_width)) as i32;
        line = (end - start + 1) as i32;

        while line != 0 {
            line -= 1;
            // very simple-minded fill loop
            i = *addr_of!(buf_width) as i32;
            while i != 0 {
                i -= 1;
                (*dest).a = alpha;
                dest = dest.offset(1);
            }
            dest = dest.offset(next_line as isize);
        }
    }

    fn code(&self, x: c_long, y: c_long) -> i32 {
        // USE: determines where a point is in relation to a bounding box
        // IN: x,y - coordinate pair
        // OUT: clipping code compared to global clip context
        let mut c: i32 = 0;

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

    pub unsafe fn ClipLine(&mut self, x1: &mut c_long, y1: &mut c_long, x2: &mut c_long, y2: &mut c_long) -> bool {
        // USE: clip a line from (x1,y1) to (x2,y2) to clip bounds
        // IN: (x1,y1)-(x2,y2) line
        // OUT: return true if something left to draw, otherwise false
        let mut c1: i32;
        let mut c2: i32;
        let mut c: i32;
        let mut x: c_long;
        let mut y: c_long;
        let mut f: c_long;

        x = *x1;
        y = *y1;

        c1 = self.code(*x1, *y1); // find where first pt. is
        c2 = self.code(*x2, *y2); // find where second pt. is

        if (c1 & c2) == 0 {
            // the line may be visible
            while (c1 | c2) != 0 {
                // where there is 2D clipping to be done
                if c1 & c2 != 0 {
                    return false; // if both on same side, quit
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
                    c1 = self.code(*x1, *y1);
                } else {
                    *x2 = x;
                    *y2 = y;
                    c2 = self.code(*x2, *y2);
                }
            } // while still needs clipping
        } else {
            // line not visible
            return false;
        }
        true
    }

    pub unsafe fn DrawLineNC(&mut self, x1: c_long, y1: c_long, x2: c_long, y2: c_long, color: CPixel32) {
        // USE: draw a line from (x1,y1) to (x2,y2) in color (no clip)
        // IN: (x1,y1) - starting coordinate
        // (x2,y2) - ending coordinate
        // color - 32-bit color value
        // OUT: none
        let mut d: c_long;
        let mut ax: c_long;
        let mut ay: c_long;
        let mut sx: c_long;
        let mut sy: c_long;
        let mut dx: c_long;
        let mut dy: c_long;
        let mut dest: *mut CPixel32;
        let mut x1_mut = x1;
        let mut y1_mut = y1;
        let mut x2_mut = x2;
        let mut y2_mut = y2;

        assert!(!(*addr_of!(buffer)).is_null());

        dx = x2_mut - x1_mut;
        ax = ABS(dx) << 1;
        sx = SIGN(dx);
        dy = y2_mut - y1_mut;
        ay = ABS(dy) << 1;
        sy = SIGN(dy);

        if color.a == 255 {
            if dy == 0 {
                // horz line
                if dx >= 0 {
                    dest = (*addr_of!(buffer)).offset((*(*addr_of!(row_off)).offset(y1_mut as isize) + x1_mut) as isize);
                    let mut i = dx + 1;
                    while i != 0 {
                        i -= 1;
                        *dest = color;
                        dest = dest.offset(1);
                    }
                } else {
                    dest = (*addr_of!(buffer))
                        .offset((*(*addr_of!(row_off)).offset(y1_mut as isize) + x1_mut + dx) as isize);
                    let mut i = -dx + 1;
                    while i != 0 {
                        i -= 1;
                        *dest = color;
                        dest = dest.offset(1);
                    }
                }
                return;
            }

            if dx == 0 {
                // vert line
                if dy >= 0 {
                    dest = (*addr_of!(buffer))
                        .offset((*(*addr_of!(row_off)).offset(y1_mut as isize) + x1_mut) as isize);
                    y2_mut += 1;
                } else {
                    dest = (*addr_of!(buffer))
                        .offset((*(*addr_of!(row_off)).offset(y2_mut as isize) + x1_mut) as isize);
                    y2_mut = -dy + 1;
                }

                while y2_mut != 0 {
                    y2_mut -= 1;
                    *dest = color;
                    dest = dest.offset(*addr_of!(stride) as isize);
                }
                return;
            }
        }

        // bressenham's algorithm
        if ax > ay {
            d = ay - (ax >> 1);
            while x1_mut != x2_mut {
                self.PutPixAlphaNC(x1_mut, y1_mut, color);

                if d >= 0 {
                    y1_mut += sy;
                    d -= ax;
                }
                x1_mut += sx;
                d += ay;
            }
        } else {
            d = ax - (ay >> 1);
            while y1_mut != y2_mut {
                self.PutPixAlphaNC(x1_mut, y1_mut, color);
                if d >= 0 {
                    x1_mut += sx;
                    d -= ay;
                }
                y1_mut += sy;
                d += ax;
            }
        }
        self.PutPixAlphaNC(x1_mut, y1_mut, color);
    }

    pub unsafe fn DrawLineAveNC(&mut self, x1: c_long, y1: c_long, x2: c_long, y2: c_long, color: CPixel32) {
        // USE: draw a translucent line from (x1,y1) to (x2,y2) in color (no clip)
        // IN: (x1,y1) - starting coordinate
        // (x2,y2) - ending coordinate
        // color - 32-bit color value
        // OUT: none
        let mut d: c_long;
        let mut ax: c_long;
        let mut ay: c_long;
        let mut sx: c_long;
        let mut sy: c_long;
        let mut dx: c_long;
        let mut dy: c_long;
        let mut dest: *mut CPixel32;
        let mut x1_mut = x1;
        let mut y1_mut = y1;
        let mut x2_mut = x2;
        let mut y2_mut = y2;

        assert!(!(*addr_of!(buffer)).is_null());

        dx = x2_mut - x1_mut;
        ax = ABS(dx) << 1;
        sx = SIGN(dx);
        dy = y2_mut - y1_mut;
        ay = ABS(dy) << 1;
        sy = SIGN(dy);

        if dy == 0 {
            // horz line
            if dx >= 0 {
                dest = (*addr_of!(buffer)).offset((*(*addr_of!(row_off)).offset(y1_mut as isize) + x1_mut) as isize);
                let mut i = dx + 1;
                while i != 0 {
                    i -= 1;
                    *dest = AVE_PIX(*dest, color);
                    dest = dest.offset(1);
                }
            } else {
                dest = (*addr_of!(buffer))
                    .offset((*(*addr_of!(row_off)).offset(y1_mut as isize) + x1_mut + dx) as isize);
                let mut i = -dx + 1;
                while i != 0 {
                    i -= 1;
                    *dest = AVE_PIX(*dest, color);
                    dest = dest.offset(1);
                }
            }
            return;
        }

        if dx == 0 {
            // vert line
            if dy >= 0 {
                dest = (*addr_of!(buffer))
                    .offset((*(*addr_of!(row_off)).offset(y1_mut as isize) + x1_mut) as isize);
                y2_mut += 1;
            } else {
                dest = (*addr_of!(buffer))
                    .offset((*(*addr_of!(row_off)).offset(y2_mut as isize) + x1_mut) as isize);
                y2_mut = -dy + 1;
            }

            while y2_mut != 0 {
                y2_mut -= 1;
                *dest = AVE_PIX(*dest, color);
                dest = dest.offset(*addr_of!(stride) as isize);
            }
            return;
        }

        // bressenham's algorithm
        if ax > ay {
            d = ay - (ax >> 1);
            while x1_mut != x2_mut {
                self.PutPixAveNC(x1_mut, y1_mut, color);

                if d >= 0 {
                    y1_mut += sy;
                    d -= ax;
                }
                x1_mut += sx;
                d += ay;
            }
        } else {
            d = ax - (ay >> 1);
            while y1_mut != y2_mut {
                self.PutPixAveNC(x1_mut, y1_mut, color);
                if d >= 0 {
                    x1_mut += sx;
                    d -= ay;
                }
                y1_mut += sy;
                d += ax;
            }
        }
        self.PutPixAveNC(x1_mut, y1_mut, color);
    }

    pub unsafe fn DrawLineAANC(&mut self, x0: c_long, y0: c_long, x1: c_long, y1: c_long, color: CPixel32) {
        // Wu antialiased line drawer.
        // USE: Function to draw an antialiased line from (x0,y0) to (x1,y1), using an
        // antialiasing approach published by Xiaolin Wu in the July 1991 issue of
        // Computer Graphics (SIGGRAPH proceedings).
        //
        // IN: (x0,y0),(x1,y1) = line to draw
        // color = 32-bit color
        // OUT: none
        let mut x0_mut = x0;
        let mut y0_mut = y0;
        let mut x1_mut = x1;
        let mut y1_mut = y1;

        assert!(!(*addr_of!(buffer)).is_null());

        // Make sure the line runs top to bottom
        if y0_mut > y1_mut {
            let temp = y0_mut;
            y0_mut = y1_mut;
            y1_mut = temp;
            let temp = x0_mut;
            x0_mut = x1_mut;
            x1_mut = temp;
        }

        let mut DeltaX = x1_mut - x0_mut;
        let mut DeltaY = y1_mut - y0_mut;
        let XDir: i32;

        // Draw the initial pixel, which is always exactly intersected by
        // the line and so needs no Alpha
        self.PutPixAlphaNC(x0_mut, y0_mut, color);

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
                x0_mut += XDir as c_long;
                self.PutPixAlphaNC(x0_mut, y0_mut, color);
            }
            return;
        }
        if DeltaX == 0 {
            // Vertical line
            loop {
                y0_mut += 1;
                self.PutPixAlphaNC(x0_mut, y0_mut, color);
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
                x0_mut += XDir as c_long;
                y0_mut += 1;
                self.PutPixAlphaNC(x0_mut, y0_mut, color);
                DeltaY -= 1;
                if DeltaY == 0 {
                    break;
                }
            }
            return;
        }

        // Line is not horizontal, diagonal, or vertical
        let mut ErrorAcc: u16 = 0; // initialize the line error accumulator to 0

        // # of bits by which to shift ErrorAcc to get intensity level
        const IntensityShift: u32 = 16 - 8;

        // Is this an X-major or Y-major line?
        if DeltaY > DeltaX {
            // Y-major line; calculate 16-bit fixed-point fractional part of a
            // pixel that X advances each time Y advances 1 pixel, truncating the
            // result so that we won't overrun the endpoint along the X axis
            let ErrorAdj: u16 = (((DeltaX as u32) << 16) / (DeltaY as u32)) as u16;

            // Draw all pixels other than the first and last
            while DeltaY != 0 {
                DeltaY -= 1;
                if DeltaY == 0 {
                    break;
                }
                let ErrorAccTemp = ErrorAcc; // remember current accumulated error
                ErrorAcc = (ErrorAcc as u32).wrapping_add(ErrorAdj as u32) as u16; // calculate error for next pixel
                if ErrorAcc <= ErrorAccTemp {
                    // The error accumulator turned over, so advance the X coord
                    x0_mut += XDir as c_long;
                }
                y0_mut += 1; // Y-major, so always advance Y
                           // The IntensityBits most significant bits of ErrorAcc give us the
                           // intensity Alpha for this pixel, and the complement of the
                           // Alpha for the paired pixel
                let Alpha = (ErrorAcc as u32) >> IntensityShift;
                let InvAlpha = 256 - Alpha;

                self.PutPixAlphaNC(x0_mut, y0_mut, ALPHA_PIX(self.GetPix(x0_mut, y0_mut), color, Alpha as c_long, InvAlpha as c_long));
                self.PutPixAlphaNC(
                    x0_mut + (XDir as c_long),
                    y0_mut,
                    ALPHA_PIX(self.GetPix(x0_mut + (XDir as c_long), y0_mut), color, InvAlpha as c_long, Alpha as c_long),
                );
            }
            // Draw the final pixel, which is always exactly intersected by the line
            // and so needs no Alpha
            self.PutPixAlphaNC(x1_mut, y1_mut, color);
            return;
        }
        // It's an X-major line; calculate 16-bit fixed-point fractional part of a
        // pixel that Y advances each time X advances 1 pixel, truncating the
        // result to avoid overrunning the endpoint along the X axis
        let ErrorAdj: u16 = (((DeltaY as u32) << 16) / (DeltaX as u32)) as u16;
        // Draw all pixels other than the first and last
        while DeltaX != 0 {
            DeltaX -= 1;
            if DeltaX == 0 {
                break;
            }
            let ErrorAccTemp = ErrorAcc; // remember current accumulated error
            ErrorAcc = (ErrorAcc as u32).wrapping_add(ErrorAdj as u32) as u16; // calculate error for next pixel
            if ErrorAcc <= ErrorAccTemp {
                // The error accumulator turned over, so advance the Y coord
                y0_mut += 1;
            }
            x0_mut += XDir as c_long; // X-major, so always advance X
                                      // The IntensityBits most significant bits of ErrorAcc give us the
                                      // intensity Alpha for this pixel, and the complement of the
                                      // Alpha for the paired pixel
            let Alpha = (ErrorAcc as u32) >> IntensityShift;
            let InvAlpha = 256 - Alpha;
            self.PutPixAlphaNC(x0_mut, y0_mut, ALPHA_PIX(self.GetPix(x0_mut, y0_mut), color, Alpha as c_long, InvAlpha as c_long));
            self.PutPixAlphaNC(
                x0_mut,
                y0_mut + 1,
                ALPHA_PIX(self.GetPix(x0_mut, y0_mut + 1), color, InvAlpha as c_long, Alpha as c_long),
            );
        }
        // Draw the final pixel, which is always exactly intersected by the line
        // and so needs no Alpha
        self.PutPixAlphaNC(x1_mut, y1_mut, color);
    }

    pub unsafe fn DrawRectNC(&mut self, ulx: c_long, uly: c_long, width: c_long, height: c_long, color: CPixel32) {
        // USE: draw rectangle in solid color, no clipping
        // IN: (ulx,uly) - coordinates of upper-left corner of rect
        // width, height - dimensions of rectangle
        // color - color value
        // OUT: none
        assert!(!(*addr_of!(buffer)).is_null());
        assert!(ulx >= 0);
        assert!(uly >= 0);
        assert!(ulx + width < *addr_of!(buf_width));
        assert!(uly + height < *addr_of!(buf_height));

        if height < 1 || width < 1 {
            return;
        }

        let mut uly_mut = uly;
        let mut height_mut = height;
        while height_mut != 0 {
            height_mut -= 1;
            self.DrawLineNC(ulx, uly_mut, ulx + width - 1, uly_mut, color);
            uly_mut += 1;
        }
    }

    pub unsafe fn DrawRect(&mut self, ulx: c_long, uly: c_long, width: c_long, height: c_long, color: CPixel32) {
        // USE: draw rectangle in solid color
        // IN: (ulx,uly) - coordinates of upper-left corner of rect
        // width, height - dimensions of rectangle
        // color - color value
        // OUT: none
        assert!(!(*addr_of!(buffer)).is_null());

        if height < 1 || width < 1 {
            return;
        }

        let mut uly_mut = uly;
        let mut height_mut = height;
        while height_mut != 0 {
            height_mut -= 1;
            self.DrawLine(ulx, uly_mut, ulx + width - 1, uly_mut, color);
            uly_mut += 1;
        }
    }

    pub unsafe fn DrawLine(&mut self, x1: c_long, y1: c_long, x2: c_long, y2: c_long, color: CPixel32) {
        let mut x1_mut = x1;
        let mut y1_mut = y1;
        let mut x2_mut = x2;
        let mut y2_mut = y2;
        if self.ClipLine(&mut x1_mut, &mut y1_mut, &mut x2_mut, &mut y2_mut) {
            self.DrawLineNC(x1_mut, y1_mut, x2_mut, y2_mut, color);
        }
    }

    pub unsafe fn DrawRectAve(&mut self, ulx: c_long, uly: c_long, width: c_long, height: c_long, color: CPixel32) {
        // USE: draw rectangle in solid color, translucent
        // IN: (ulx,uly) - coordinates of upper-left corner of rect
        // width, height - dimensions of rectangle
        // color - color value
        // OUT: none
        assert!(!(*addr_of!(buffer)).is_null());

        if height < 1 || width < 1 {
            return;
        }

        let mut uly_mut = uly;
        let mut height_mut = height;
        while height_mut != 0 {
            height_mut -= 1;
            self.DrawLineAve(ulx, uly_mut, ulx + width - 1, uly_mut, color);
            uly_mut += 1;
        }
    }

    pub unsafe fn DrawLineAve(&mut self, x1: c_long, y1: c_long, x2: c_long, y2: c_long, color: CPixel32) {
        let mut x1_mut = x1;
        let mut y1_mut = y1;
        let mut x2_mut = x2;
        let mut y2_mut = y2;
        if self.ClipLine(&mut x1_mut, &mut y1_mut, &mut x2_mut, &mut y2_mut) {
            self.DrawLineAveNC(x1_mut, y1_mut, x2_mut, y2_mut, color);
        }
    }

    pub unsafe fn DrawBoxNC(&mut self, ulx: c_long, uly: c_long, width: c_long, height: c_long, color: CPixel32) {
        // USE: Draw an empty box, no clipping
        // IN: (ulx,uly) - coordinates of upper-left corner of box
        // width, height - dimensions of box
        // color - color value
        // OUT: none
        assert!(!(*addr_of!(buffer)).is_null());

        if height < 1 || width < 1 {
            return;
        }

        self.DrawLineNC(ulx, uly, ulx + width - 1, uly, color);
        self.DrawLineNC(ulx, uly + height - 1, ulx + width - 1, uly + height - 1, color);
        self.DrawLineNC(ulx, uly, ulx, uly + height - 1, color);
        self.DrawLineNC(ulx + width - 1, uly, ulx + width - 1, uly + height - 1, color);
    }

    pub unsafe fn DrawBox(&mut self, ulx: c_long, uly: c_long, width: c_long, height: c_long, color: CPixel32) {
        // USE: Draw an empty box
        // IN: (ulx,uly) - coordinates of upper-left corner of rect
        // width, height - dimensions of rectangle
        // color - color value
        // OUT: none
        assert!(!(*addr_of!(buffer)).is_null());

        if height < 1 || width < 1 {
            return;
        }

        self.DrawLine(ulx, uly, ulx + width - 1, uly, color);
        self.DrawLine(ulx, uly + height - 1, ulx + width - 1, uly + height - 1, color);
        self.DrawLine(ulx, uly, ulx, uly + height - 1, color);
        self.DrawLine(ulx + width - 1, uly, ulx + width - 1, uly + height - 1, color);
    }

    pub unsafe fn DrawBoxAve(&mut self, ulx: c_long, uly: c_long, width: c_long, height: c_long, color: CPixel32) {
        // USE: Draw an empty box, translucent
        // IN: (ulx,uly) - coordinates of upper-left corner of rect
        // width, height - dimensions of rectangle
        // color - color value
        // OUT: none
        assert!(!(*addr_of!(buffer)).is_null());

        if height < 1 || width < 1 {
            return;
        }

        self.DrawLineAve(ulx, uly, ulx + width - 1, uly, color);
        self.DrawLineAve(ulx, uly + height - 1, ulx + width - 1, uly + height - 1, color);
        self.DrawLineAve(ulx, uly, ulx, uly + height - 1, color);
        self.DrawLineAve(ulx + width - 1, uly, ulx + width - 1, uly + height - 1, color);
    }

    pub unsafe fn DrawCircle(&mut self, xc: c_long, yc: c_long, r: c_long, edge: CPixel32, fill: CPixel32) {
        // USE: Draw a simple circle in current color with Bresenham's
        // circle algorithm.
        // See PROCEDURAL ELEMENTS FOR COMPUTER GRAPHICS
        // David F. Rogers Pg. 48.
        //
        // IN:   xc,yc - center
        //       r     - radius
        //       edge  - edge color
        //       fill  - fill color
        //
        // OUT: NONE (a circle drawn in the off-screen buffer)
        let mut x: c_long;
        let mut y: c_long;
        let mut limit: c_long;
        let mut di: c_long;
        let mut delta: c_long;
        let mut last_x: c_long;
        let mut last_y: c_long;

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
            limit = 0;

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
                            /* di = 0 */
                            // move diagonal
                            last_x = x;
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        }
                    }
                }

                if y != last_y {
                    // circle fill
                    self.DrawLine(xc - last_x, yc + last_y, xc + last_x, yc + last_y, fill);
                    if last_y > limit {
                        self.DrawLine(xc - last_x, yc - last_y, xc + last_x, yc - last_y, fill);
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
            limit = 0;
            di = 2 * (1 - r);

            loop {
                // circle edge
                self.PutPix(xc + x, yc + y, edge);
                self.PutPix(xc - x, yc + y, edge);
                if y > limit {
                    self.PutPix(xc + x, yc - y, edge);
                    self.PutPix(xc - x, yc - y, edge);
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
                            /* di = 0 */
                            // move diagonal
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

    pub unsafe fn DrawCircleAve(&mut self, xc: c_long, yc: c_long, r: c_long, edge: CPixel32, fill: CPixel32) {
        // USE: Draw a simple circle in current color with Bresenham's
        // circle algorithm.
        // a circle with fill and edge colors averaged with dest (-1 = no color)
        // See PROCEDURAL ELEMENTS FOR COMPUTER GRAPHICS
        // David F. Rogers Pg. 48.
        //
        // IN:   xc,yc - center
        //       r     - radius
        //       edge  - edge color
        //       fill  - fill color
        //
        // OUT: none (a circle on in the off-screen buffer)
        let mut x: c_long;
        let mut y: c_long;
        let mut limit: c_long;
        let mut di: c_long;
        let mut delta: c_long;
        let mut last_x: c_long;
        let mut last_y: c_long;
        let mut f: c_long;

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
            limit = 0;

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
                            /* di = 0 */
                            // move diagonal
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
                        self.PutPixAve(f, yc + last_y, fill);
                        f += 1;
                    }
                    if last_y > limit {
                        f = xc - last_x;
                        while f <= xc + last_x {
                            self.PutPixAve(f, yc - last_y, fill);
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
            limit = 0;
            di = 2 * (1 - r);

            loop {
                // circle edge
                self.PutPixAve(xc + x, yc + y, edge);
                self.PutPixAve(xc - x, yc + y, edge);
                if y > limit {
                    self.PutPixAve(xc + x, yc - y, edge);
                    self.PutPixAve(xc - x, yc - y, edge);
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
                            /* di = 0 */
                            // move diagonal
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
    // Concave Polygon Scan Conversion
    ////////////////////////////////////////////////////////////////////////////

    pub unsafe fn DrawPolygon(&mut self, nvert: i32, point: *mut POINT, edge: CPixel32, fill: CPixel32) {
        // USE:    Scan convert a polygon
        // IN:     nvert:        Number of vertices
        //         point:        Vertices of polygon
        //         edge:         edge color
        //         fill:         fill color
        // OUT:    none
        let mut k: i32;
        let mut y0: i32;
        let mut y1: i32;
        let mut y: i32;
        let mut i: i32;
        let mut j: i32;
        let mut xl: i32;
        let mut xr: i32;

        assert!(!(*addr_of!(buffer)).is_null());

        n = nvert;

        if n <= 0 {
            // nothing to do
            return;
        }

        pt = point;

        if fill.a != 0 {
            // draw fill

            // create y-sorted array of indices ind[k] into vertex list
            k = 0;
            while k < n {
                ind[k as usize] = k;
                k += 1;
            }

            // sort ind by pt[ind[k]].y
            shell_sort(
                ind.as_mut_ptr() as *mut core::ffi::c_void,
                n,
                core::mem::size_of::<i32>() as i32,
                |u, v| compare_ind(u as *const i32, v as *const i32),
            );

            nact = 0; // start with empty active list
            k = 0; // ind[k] is next vertex to process

            // ymin of polygon
            y0 = MAX(*addr_of!(clip_min_y) - 1, (*pt.offset(ind[0] as isize)).y) as i32;

            // ymax of polygon
            y1 = MIN(*addr_of!(clip_max_y) + 1, (*pt.offset(ind[(n - 1) as usize] as isize)).y) as i32;

            // step through scanlines
            y = y0;
            while y < y1 {
                // Check vertices between previous scanline
                // and current one, if any
                while k < n && (*pt.offset(ind[k as usize] as isize)).y <= y as c_long {
                    i = ind[k as usize];
                    // insert or delete edges before and after vertex i
                    // (i-1 to i, and i to i+1)
                    // from active list if they cross scanline y
                    j = if i > 0 { i - 1 } else { n - 1 }; // vertex previous to i
                    if (*pt.offset(j as isize)).y < y as c_long {
                        // old edge, remove from active list
                        del_edge(j);
                    } else {
                        if (*pt.offset(j as isize)).y > y as c_long {
                            // new edge, add to active list
                            ins_edge(j, y);
                        }
                    }
                    j = if i < n - 1 { i + 1 } else { 0 }; // vertex next after i
                    if (*pt.offset(j as isize)).y < y as c_long {
                        // old edge, remove from active list
                        del_edge(i);
                    } else {
                        if (*pt.offset(j as isize)).y > y as c_long {
                            // new edge, add to active list
                            ins_edge(i, y);
                        }
                    }
                    k += 1;
                }

                // sort active edge list by active[j].x
                shell_sort(
                    active.as_mut_ptr() as *mut core::ffi::c_void,
                    nact,
                    core::mem::size_of::<POLYEDGE>() as i32,
                    |u, v| compare_active(u as *const POLYEDGE, v as *const POLYEDGE),
                );

                // draw horizontal segments for scanline y
                j = 0;
                while j < nact {
                    // draw horizontal segments
                    // span 'tween j & j+1 is inside, span tween
                    // j+1 & j+2 is outside

                    // left end of span
                    // convert back from fixed point - round down
                    xl = (active[j as usize].x >> INT_SHIFT) as i32;
                    if xl < *addr_of!(clip_min_x) as i32 - 1 {
                        xl = *addr_of!(clip_min_x) as i32 - 1;
                    }

                    // right end of span
                    // convert back from fixed point - round down
                    xr = (active[(j + 1) as usize].x >> INT_SHIFT) as i32;
                    if xr > *addr_of!(clip_max_x) as i32 {
                        xr = *addr_of!(clip_max_x) as i32;
                    }

                    if xl < xr {
                        // draw pixels in span
                        self.DrawLine((xl + 1) as c_long, y as c_long, xr as c_long, y as c_long, fill);
                    }

                    // increment edge coords
                    active[j as usize].x += active[j as usize].dx;
                    active[(j + 1) as usize].x += active[(j + 1) as usize].dx;
                    j += 2;
                }

                y += 1;
            }

            if edge.a != 0 {
                // draw edges
                k = 0;
                while k < n - 1 {
                    self.DrawLineAA(
                        (*pt.offset(k as isize)).x,
                        (*pt.offset(k as isize)).y,
                        (*pt.offset((k + 1) as isize)).x,
                        (*pt.offset((k + 1) as isize)).y,
                        edge,
                    );
                    k += 1;
                }

                self.DrawLineAA(
                    (*pt.offset((n - 1) as isize)).x,
                    (*pt.offset((n - 1) as isize)).y,
                    (*pt.offset(0)).x,
                    (*pt.offset(0)).y,
                    edge,
                );
            }
        }
    }

    pub unsafe fn DrawLineAA(&mut self, x1: c_long, y1: c_long, x2: c_long, y2: c_long, color: CPixel32) {
        let mut x1_mut = x1;
        let mut y1_mut = y1;
        let mut x2_mut = x2;
        let mut y2_mut = y2;
        if self.ClipLine(&mut x1_mut, &mut y1_mut, &mut x2_mut, &mut y2_mut) {
            self.DrawLineAANC(x1_mut, y1_mut, x2_mut, y2_mut, color);
        }
    }

    pub unsafe fn Blit(
        &mut self,
        dstX: c_long,
        dstY: c_long,
        width: c_long,
        height: c_long,
        srcImage: *mut CPixel32,
        srcX: c_long,
        srcY: c_long,
        srcStride: c_long,
    ) {
        // USE: simple blit
        // IN: dstX, dstY - upper left corner of where image will land in buffer
        // width, height - width and height of image
        // srcImage - src image buffer
        // srcX, srcY - upper left corner in src image
        // srcStride - number of pixels per line in src image
        // OUT: none
        let mut dst: *mut CPixel32;
        let mut src: *mut CPixel32;
        let mut x: i32;
        let mut y: i32;
        let mut dstX = dstX;
        let mut dstY = dstY;
        let mut width = width;
        let mut height = height;
        let mut srcX = srcX;
        let mut srcY = srcY;

        assert!(!(*addr_of!(buffer)).is_null());

        self.BlitClip(&mut dstX, &mut dstY, &mut width, &mut height, &mut srcX, &mut srcY);

        if width < 1 || height < 1 {
            return;
        }

        dst = (*addr_of!(buffer)).offset(PIXPOS(dstX, dstY, *addr_of!(stride)) as isize);
        src = srcImage.offset(PIXPOS(srcX, srcY, srcStride) as isize);

        y = 0;
        while y < height as i32 {
            x = 0;
            while x < width as i32 {
                // *dst++ = *src++;
                let alpha = (*src).a;
                let dst_alpha = (*dst).a;
                *dst = ALPHA_PIX(*src, *dst, alpha as c_long, (256 - alpha as i32) as c_long);
                (*dst).a = dst_alpha;
                dst = dst.offset(1);
                src = src.offset(1);
                x += 1;
            }
            dst = dst.offset((*addr_of!(stride) - width) as isize);
            src = src.offset((srcStride - width) as isize);
            y += 1;
        }
    }

    pub unsafe fn BlitClip(
        &mut self,
        dstX: &mut c_long,
        dstY: &mut c_long,
        width: &mut c_long,
        height: &mut c_long,
        srcX: &mut c_long,
        srcY: &mut c_long,
    ) {
        // USE: simple blit clip
        // IN: dstX, dstY - upper left corner of where image will land in buffer
        // width, height - width and height of image
        // srcX, srcY - upper left corner in src image
        // OUT: none

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

    pub unsafe fn BlitColor(
        &mut self,
        dstX: c_long,
        dstY: c_long,
        width: c_long,
        height: c_long,
        srcImage: *mut CPixel32,
        srcX: c_long,
        srcY: c_long,
        srcStride: c_long,
        color: CPixel32,
    ) {
        // USE: blit using image alpha as mask
        // IN: dstX, dstY - upper left corner of where image will land in buffer
        // width, height - width and height of image
        // srcImage - src image buffer
        // srcX, srcY - upper left corner in src image
        // srcStride - number of pixels per line in src image
        // color - color to apply to srcImage
        // OUT: none
        let mut dst: *mut CPixel32;
        let mut src: *mut CPixel32;
        let mut x: i32;
        let mut y: i32;
        let mut dstX = dstX;
        let mut dstY = dstY;
        let mut width = width;
        let mut height = height;
        let mut srcX = srcX;
        let mut srcY = srcY;
        let dstNextLine: i32;
        let srcNextLine: i32;

        assert!(!(*addr_of!(buffer)).is_null());

        self.BlitClip(&mut dstX, &mut dstY, &mut width, &mut height, &mut srcX, &mut srcY);

        if width < 1 || height < 1 {
            return;
        }

        dst = (*addr_of!(buffer)).offset(PIXPOS(dstX, dstY, *addr_of!(stride)) as isize);
        src = srcImage.offset(PIXPOS(srcX, srcY, srcStride) as isize);

        dstNextLine = (*addr_of!(stride) - width) as i32;
        srcNextLine = (srcStride - width) as i32;

        y = 0;
        while y < height as i32 {
            x = 0;
            while x < width as i32 {
                let alpha = (*src).a;
                *dst = ALPHA_PIX(color, *dst, alpha as c_long, (256 - alpha as i32) as c_long);
                dst = dst.offset(1);
                src = src.offset(1);
                x += 1;
            }
            dst = dst.offset(dstNextLine as isize);
            src = src.offset(srcNextLine as isize);
            y += 1;
        }
    }
}
