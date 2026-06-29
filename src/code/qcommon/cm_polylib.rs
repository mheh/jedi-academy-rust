// this is only used for visualization tools in cm_ debug functions

#![allow(non_snake_case)]

use crate::codemp::game::q_shared_h::{vec3_t, vec_t};
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Constants from cm_polylib.h
pub const MAX_POINTS_ON_WINDING: c_int = 64;

pub const SIDE_FRONT: c_int = 0;
pub const SIDE_BACK: c_int = 1;
pub const SIDE_ON: c_int = 2;
pub const SIDE_CROSS: c_int = 3;

pub const CLIP_EPSILON: vec_t = 0.1;

pub const MAX_MAP_BOUNDS: c_int = 65535;

// you can define on_epsilon in the makefile as tighter
pub const ON_EPSILON: vec_t = 0.1;

// counters are only bumped when running single threaded,
// because they are an awefull coherence problem
pub static mut c_active_windings: c_int = 0;
pub static mut c_peak_windings: c_int = 0;
pub static mut c_winding_allocs: c_int = 0;
pub static mut c_winding_points: c_int = 0;

// External C functions
extern "C" {
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Com_Error(level: c_int, format: *const c_char, ...);
    fn printf(format: *const c_char, ...) -> c_int;
    fn VectorNormalize2(v: *const vec3_t, out: *mut vec3_t) -> vec_t;
    fn VectorLength(v: *const vec3_t) -> vec_t;
}

// winding_t structure from cm_polylib.h
#[repr(C)]
pub struct winding_t {
    pub numpoints: c_int,
    pub p: [[vec_t; 3]; 4], // variable sized
}

// Inline vector operation macros translated from C

#[inline]
fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> vec_t {
    unsafe { (*v1)[0] * (*v2)[0] + (*v1)[1] * (*v2)[1] + (*v1)[2] * (*v2)[2] }
}

#[inline]
fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t) {
    unsafe {
        (*c)[0] = (*a)[0] - (*b)[0];
        (*c)[1] = (*a)[1] - (*b)[1];
        (*c)[2] = (*a)[2] - (*b)[2];
    }
}

#[inline]
fn VectorAdd(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t) {
    unsafe {
        (*c)[0] = (*a)[0] + (*b)[0];
        (*c)[1] = (*a)[1] + (*b)[1];
        (*c)[2] = (*a)[2] + (*b)[2];
    }
}

#[inline]
fn VectorCopy(a: *const vec3_t, b: *mut vec3_t) {
    unsafe {
        (*b)[0] = (*a)[0];
        (*b)[1] = (*a)[1];
        (*b)[2] = (*a)[2];
    }
}

#[inline]
fn VectorScale(v: *const vec3_t, s: vec_t, o: *mut vec3_t) {
    unsafe {
        (*o)[0] = (*v)[0] * s;
        (*o)[1] = (*v)[1] * s;
        (*o)[2] = (*v)[2] * s;
    }
}

#[inline]
fn VectorMA(v: *const vec3_t, s: vec_t, b: *const vec3_t, o: *mut vec3_t) {
    unsafe {
        (*o)[0] = (*v)[0] + (*b)[0] * s;
        (*o)[1] = (*v)[1] + (*b)[1] * s;
        (*o)[2] = (*v)[2] + (*b)[2] * s;
    }
}

#[inline]
fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, cross: *mut vec3_t) {
    unsafe {
        (*cross)[0] = (*v1)[1] * (*v2)[2] - (*v1)[2] * (*v2)[1];
        (*cross)[1] = (*v1)[2] * (*v2)[0] - (*v1)[0] * (*v2)[2];
        (*cross)[2] = (*v1)[0] * (*v2)[1] - (*v1)[1] * (*v2)[0];
    }
}

pub fn pw(w: *mut winding_t) {
    unsafe {
        let mut i = 0;
        while i < (*w).numpoints {
            printf(
                b"(%5.1f, %5.1f, %5.1f)\n\0".as_ptr() as *const c_char,
                (*w).p[i as usize][0],
                (*w).p[i as usize][1],
                (*w).p[i as usize][2],
            );
            i += 1;
        }
    }
}

/*
=============
AllocWinding
=============
*/
pub fn AllocWinding(points: c_int) -> *mut winding_t {
    unsafe {
        c_winding_allocs += 1;
        c_winding_points += points;
        c_active_windings += 1;
        if c_active_windings > c_peak_windings {
            c_peak_windings = c_active_windings;
        }

        let s = core::mem::size_of::<vec_t>() * 3 * points as usize + core::mem::size_of::<c_int>();
        let w = Z_Malloc(s, 0, 1) as *mut winding_t; // TAG_BSP = 0, qtrue = 1
        //	memset (w, 0, s);	// qtrue above does this
        w
    }
}

pub fn FreeWinding(w: *mut winding_t) {
    unsafe {
        let w_ptr = w as *mut u32;
        if *w_ptr == 0xdeaddead {
            Com_Error(
                1, // ERR_DROP
                b"FreeWinding: freed a freed winding\0".as_ptr() as *const c_char,
            );
        }
        *w_ptr = 0xdeaddead;

        c_active_windings -= 1;
        Z_Free(w as *mut c_void);
    }
}

/*
============
RemoveColinearPoints
============
*/
pub static mut c_removed: c_int = 0;

pub fn RemoveColinearPoints(w: *mut winding_t) {
    unsafe {
        let mut i: c_int;
        let mut j: c_int;
        let mut k: c_int;
        let mut v1: vec3_t = [0.0; 3];
        let mut v2: vec3_t = [0.0; 3];
        let mut nump: c_int;
        let mut p: [[vec_t; 3]; 64] = [[0.0; 3]; 64]; // MAX_POINTS_ON_WINDING = 64

        nump = 0;
        i = 0;
        while i < (*w).numpoints {
            j = (i + 1) % (*w).numpoints;
            k = (i + (*w).numpoints - 1) % (*w).numpoints;
            VectorSubtract(addr_of!((*w).p[j as usize]), addr_of!((*w).p[i as usize]), addr_of_mut!(v1));
            VectorSubtract(addr_of!((*w).p[i as usize]), addr_of!((*w).p[k as usize]), addr_of_mut!(v2));
            VectorNormalize2(addr_of!(v1), addr_of_mut!(v1));
            VectorNormalize2(addr_of!(v2), addr_of_mut!(v2));
            if DotProduct(addr_of!(v1), addr_of!(v2)) < 0.999 {
                VectorCopy(addr_of!((*w).p[i as usize]), addr_of_mut!(p[nump as usize]));
                nump += 1;
            }
            i += 1;
        }

        if nump == (*w).numpoints {
            return;
        }

        c_removed += (*w).numpoints - nump;
        (*w).numpoints = nump;
        core::ptr::copy_nonoverlapping(
            addr_of!(p[0]) as *const c_void,
            addr_of_mut!((*w).p[0]) as *mut c_void,
            (nump as usize) * core::mem::size_of::<vec3_t>(),
        );
    }
}

/*
============
WindingPlane
============
*/
pub fn WindingPlane(w: *mut winding_t, normal: *mut vec3_t, dist: *mut vec_t) {
    unsafe {
        let mut v1: vec3_t = [0.0; 3];
        let mut v2: vec3_t = [0.0; 3];

        VectorSubtract(addr_of!((*w).p[1]), addr_of!((*w).p[0]), addr_of_mut!(v1));
        VectorSubtract(addr_of!((*w).p[2]), addr_of!((*w).p[0]), addr_of_mut!(v2));
        CrossProduct(addr_of!(v2), addr_of!(v1), normal);
        VectorNormalize2(normal, normal);
        *dist = DotProduct(addr_of!((*w).p[0]), normal);
    }
}

/*
=============
WindingArea
=============
*/
pub fn WindingArea(w: *mut winding_t) -> vec_t {
    unsafe {
        let mut i: c_int;
        let mut d1: vec3_t = [0.0; 3];
        let mut d2: vec3_t = [0.0; 3];
        let mut cross: vec3_t = [0.0; 3];
        let mut total: vec_t;

        total = 0.0;
        i = 2;
        while i < (*w).numpoints {
            VectorSubtract(addr_of!((*w).p[(i - 1) as usize]), addr_of!((*w).p[0]), addr_of_mut!(d1));
            VectorSubtract(addr_of!((*w).p[i as usize]), addr_of!((*w).p[0]), addr_of_mut!(d2));
            CrossProduct(addr_of!(d1), addr_of!(d2), addr_of_mut!(cross));
            total += 0.5 * VectorLength(addr_of!(cross));
            i += 1;
        }
        total
    }
}

pub fn WindingBounds(w: *mut winding_t, mins: *mut vec3_t, maxs: *mut vec3_t) {
    unsafe {
        let mut v: vec_t;
        let mut i: c_int;
        let mut j: c_int;

        (*mins)[0] = MAX_MAP_BOUNDS as vec_t; // 99999;	// WORLD_SIZE instead of MAX_WORLD_COORD so that...
        (*mins)[1] = MAX_MAP_BOUNDS as vec_t;
        (*mins)[2] = MAX_MAP_BOUNDS as vec_t;
        (*maxs)[0] = -(MAX_MAP_BOUNDS as vec_t); //-99999;	// ... it's guaranteed to be outide of legal
        (*maxs)[1] = -(MAX_MAP_BOUNDS as vec_t);
        (*maxs)[2] = -(MAX_MAP_BOUNDS as vec_t);

        i = 0;
        while i < (*w).numpoints {
            j = 0;
            while j < 3 {
                v = (*w).p[i as usize][j as usize];
                if v < (*mins)[j as usize] {
                    (*mins)[j as usize] = v;
                }
                if v > (*maxs)[j as usize] {
                    (*maxs)[j as usize] = v;
                }
                j += 1;
            }
            i += 1;
        }
    }
}

/*
=============
WindingCenter
=============
*/
pub fn WindingCenter(w: *mut winding_t, center: *mut vec3_t) {
    unsafe {
        let mut i: c_int;
        let mut scale: vec_t;

        (*center)[0] = 0.0;
        (*center)[1] = 0.0;
        (*center)[2] = 0.0;
        i = 0;
        while i < (*w).numpoints {
            VectorAdd(addr_of!((*w).p[i as usize]), center, center);
            i += 1;
        }

        scale = 1.0 / (*w).numpoints as vec_t;
        VectorScale(center, scale, center);
    }
}

/*
=================
BaseWindingForPlane
=================
*/
pub fn BaseWindingForPlane(normal: *mut vec3_t, dist: vec_t) -> *mut winding_t {
    unsafe {
        let mut i: c_int;
        let mut x: c_int;
        let mut max: vec_t;
        let mut v: vec_t;
        let mut org: vec3_t = [0.0; 3];
        let mut vright: vec3_t = [0.0; 3];
        let mut vup: vec3_t = [0.0; 3];
        let mut w: *mut winding_t;

        // find the major axis

        max = -(MAX_MAP_BOUNDS as vec_t);
        x = -1;
        i = 0;
        while i < 3 {
            v = (*normal)[i as usize].abs();
            if v > max {
                x = i;
                max = v;
            }
            i += 1;
        }
        if x == -1 {
            Com_Error(
                1, // ERR_DROP
                b"BaseWindingForPlane: no axis found\0".as_ptr() as *const c_char,
            );
        }

        vup[0] = 0.0;
        vup[1] = 0.0;
        vup[2] = 0.0;
        match x {
            0 | 1 => {
                vup[2] = 1.0;
            }
            2 => {
                vup[0] = 1.0;
            }
            _ => {}
        }

        v = DotProduct(addr_of!(vup), normal);
        VectorMA(addr_of!(vup), -v, normal, addr_of_mut!(vup));
        VectorNormalize2(addr_of!(vup), addr_of_mut!(vup));

        VectorScale(normal, dist, addr_of_mut!(org));

        CrossProduct(addr_of!(vup), normal, addr_of_mut!(vright));

        VectorScale(addr_of!(vup), MAX_MAP_BOUNDS as vec_t, addr_of_mut!(vup));
        VectorScale(addr_of!(vright), MAX_MAP_BOUNDS as vec_t, addr_of_mut!(vright));

        // project a really big	axis aligned box onto the plane
        w = AllocWinding(4);

        VectorSubtract(addr_of!(org), addr_of!(vright), addr_of_mut!((*w).p[0]));
        VectorAdd(addr_of!((*w).p[0]), addr_of!(vup), addr_of_mut!((*w).p[0]));

        VectorAdd(addr_of!(org), addr_of!(vright), addr_of_mut!((*w).p[1]));
        VectorAdd(addr_of!((*w).p[1]), addr_of!(vup), addr_of_mut!((*w).p[1]));

        VectorAdd(addr_of!(org), addr_of!(vright), addr_of_mut!((*w).p[2]));
        VectorSubtract(addr_of!((*w).p[2]), addr_of!(vup), addr_of_mut!((*w).p[2]));

        VectorSubtract(addr_of!(org), addr_of!(vright), addr_of_mut!((*w).p[3]));
        VectorSubtract(addr_of!((*w).p[3]), addr_of!(vup), addr_of_mut!((*w).p[3]));

        (*w).numpoints = 4;

        w
    }
}

/*
==================
CopyWinding
==================
*/
pub fn CopyWinding(w: *mut winding_t) -> *mut winding_t {
    unsafe {
        let mut size: c_int;
        let mut c: *mut winding_t;

        c = AllocWinding((*w).numpoints);
        size = (4 + ((*w).numpoints as usize) * core::mem::size_of::<vec3_t>()) as c_int;
        core::ptr::copy_nonoverlapping(
            w as *const c_void,
            c as *mut c_void,
            size as usize,
        );
        c
    }
}

/*
==================
ReverseWinding
==================
*/
pub fn ReverseWinding(w: *mut winding_t) -> *mut winding_t {
    unsafe {
        let mut i: c_int;
        let mut c: *mut winding_t;

        c = AllocWinding((*w).numpoints);
        i = 0;
        while i < (*w).numpoints {
            VectorCopy(
                addr_of!((*w).p[((*w).numpoints - 1 - i) as usize]),
                addr_of_mut!((*c).p[i as usize]),
            );
            i += 1;
        }
        (*c).numpoints = (*w).numpoints;
        c
    }
}

/*
=============
ClipWindingEpsilon
=============
*/
pub fn ClipWindingEpsilon(
    r#in: *mut winding_t,
    normal: *mut vec3_t,
    dist: vec_t,
    epsilon: vec_t,
    front: *mut *mut winding_t,
    back: *mut *mut winding_t,
) {
    unsafe {
        let mut dists: [vec_t; 68] = [0.0; 68]; // MAX_POINTS_ON_WINDING+4 = 68
        let mut sides: [c_int; 68] = [0; 68];
        let mut counts: [c_int; 3] = [0, 0, 0];
        static mut dot: vec_t = 0.0; // VC 4.2 optimizer bug if not static
        let mut i: c_int;
        let mut j: c_int;
        let mut p1: *mut vec3_t;
        let mut p2: *mut vec3_t;
        let mut mid: vec3_t = [0.0; 3];
        let mut f: *mut winding_t;
        let mut b: *mut winding_t;
        let mut maxpts: c_int;

        counts[0] = 0;
        counts[1] = 0;
        counts[2] = 0;

        // determine sides for each point
        i = 0;
        while i < (*r#in).numpoints {
            dot = DotProduct(addr_of!((*r#in).p[i as usize]), normal);
            dot -= dist;
            dists[i as usize] = dot;
            if dot > epsilon {
                sides[i as usize] = SIDE_FRONT;
            } else if dot < -epsilon {
                sides[i as usize] = SIDE_BACK;
            } else {
                sides[i as usize] = SIDE_ON;
            }
            counts[sides[i as usize] as usize] += 1;
            i += 1;
        }
        sides[(*r#in).numpoints as usize] = sides[0];
        dists[(*r#in).numpoints as usize] = dists[0];

        *front = core::ptr::null_mut();
        *back = core::ptr::null_mut();

        if counts[0] == 0 {
            *back = CopyWinding(r#in);
            return;
        }
        if counts[1] == 0 {
            *front = CopyWinding(r#in);
            return;
        }

        maxpts = (*r#in).numpoints + 4; // cant use counts[0]+2 because
                                        // of fp grouping errors

        *front = AllocWinding(maxpts);
        f = *front;
        *back = AllocWinding(maxpts);
        b = *back;

        i = 0;
        while i < (*r#in).numpoints {
            p1 = addr_of_mut!((*r#in).p[i as usize]);

            if sides[i as usize] == SIDE_ON {
                VectorCopy(p1, addr_of_mut!((*f).p[(*f).numpoints as usize]));
                (*f).numpoints += 1;
                VectorCopy(p1, addr_of_mut!((*b).p[(*b).numpoints as usize]));
                (*b).numpoints += 1;
                i += 1;
                continue;
            }

            if sides[i as usize] == SIDE_FRONT {
                VectorCopy(p1, addr_of_mut!((*f).p[(*f).numpoints as usize]));
                (*f).numpoints += 1;
            }
            if sides[i as usize] == SIDE_BACK {
                VectorCopy(p1, addr_of_mut!((*b).p[(*b).numpoints as usize]));
                (*b).numpoints += 1;
            }

            if sides[(i + 1) as usize] == SIDE_ON || sides[(i + 1) as usize] == sides[i as usize] {
                i += 1;
                continue;
            }

            // generate a split point
            p2 = addr_of_mut!((*r#in).p[(((i + 1) % (*r#in).numpoints) as usize)]);

            dot = dists[i as usize] / (dists[i as usize] - dists[(i + 1) as usize]);
            j = 0;
            while j < 3 {
                // avoid round off error when possible
                if (*normal)[j as usize] == 1.0 {
                    mid[j as usize] = dist;
                } else if (*normal)[j as usize] == -1.0 {
                    mid[j as usize] = -dist;
                } else {
                    mid[j as usize] = (*p1)[j as usize] + dot * ((*p2)[j as usize] - (*p1)[j as usize]);
                }
                j += 1;
            }

            VectorCopy(addr_of!(mid), addr_of_mut!((*f).p[(*f).numpoints as usize]));
            (*f).numpoints += 1;
            VectorCopy(addr_of!(mid), addr_of_mut!((*b).p[(*b).numpoints as usize]));
            (*b).numpoints += 1;

            i += 1;
        }

        if (*f).numpoints > maxpts || (*b).numpoints > maxpts {
            Com_Error(
                1, // ERR_DROP
                b"ClipWinding: points exceeded estimate\0".as_ptr() as *const c_char,
            );
        }
        if (*f).numpoints > MAX_POINTS_ON_WINDING || (*b).numpoints > MAX_POINTS_ON_WINDING {
            Com_Error(
                1, // ERR_DROP
                b"ClipWinding: MAX_POINTS_ON_WINDING\0".as_ptr() as *const c_char,
            );
        }
    }
}

/*
=============
ChopWindingInPlace
=============
*/
pub fn ChopWindingInPlace(
    inout: *mut *mut winding_t,
    normal: *mut vec3_t,
    dist: vec_t,
    epsilon: vec_t,
) {
    unsafe {
        let mut r#in: *mut winding_t;
        let mut dists: [vec_t; 68] = [0.0; 68]; // MAX_POINTS_ON_WINDING+4 = 68
        let mut sides: [c_int; 68] = [0; 68];
        let mut counts: [c_int; 3] = [0, 0, 0];
        static mut dot: vec_t = 0.0; // VC 4.2 optimizer bug if not static
        let mut i: c_int;
        let mut j: c_int;
        let mut p1: *mut vec3_t;
        let mut p2: *mut vec3_t;
        let mut mid: vec3_t = [0.0; 3];
        let mut f: *mut winding_t;
        let mut maxpts: c_int;

        r#in = *inout;
        counts[0] = 0;
        counts[1] = 0;
        counts[2] = 0;

        // determine sides for each point
        i = 0;
        while i < (*r#in).numpoints {
            dot = DotProduct(addr_of!((*r#in).p[i as usize]), normal);
            dot -= dist;
            dists[i as usize] = dot;
            if dot > epsilon {
                sides[i as usize] = SIDE_FRONT;
            } else if dot < -epsilon {
                sides[i as usize] = SIDE_BACK;
            } else {
                sides[i as usize] = SIDE_ON;
            }
            counts[sides[i as usize] as usize] += 1;
            i += 1;
        }
        sides[(*r#in).numpoints as usize] = sides[0];
        dists[(*r#in).numpoints as usize] = dists[0];

        if counts[0] == 0 {
            FreeWinding(r#in);
            *inout = core::ptr::null_mut();
            return;
        }
        if counts[1] == 0 {
            return; // inout stays the same
        }

        maxpts = (*r#in).numpoints + 4; // cant use counts[0]+2 because
                                        // of fp grouping errors

        f = AllocWinding(maxpts);

        i = 0;
        while i < (*r#in).numpoints {
            p1 = addr_of_mut!((*r#in).p[i as usize]);

            if sides[i as usize] == SIDE_ON {
                VectorCopy(p1, addr_of_mut!((*f).p[(*f).numpoints as usize]));
                (*f).numpoints += 1;
                i += 1;
                continue;
            }

            if sides[i as usize] == SIDE_FRONT {
                VectorCopy(p1, addr_of_mut!((*f).p[(*f).numpoints as usize]));
                (*f).numpoints += 1;
            }

            if sides[(i + 1) as usize] == SIDE_ON || sides[(i + 1) as usize] == sides[i as usize] {
                i += 1;
                continue;
            }

            // generate a split point
            p2 = addr_of_mut!((*r#in).p[(((i + 1) % (*r#in).numpoints) as usize)]);

            dot = dists[i as usize] / (dists[i as usize] - dists[(i + 1) as usize]);
            j = 0;
            while j < 3 {
                // avoid round off error when possible
                if (*normal)[j as usize] == 1.0 {
                    mid[j as usize] = dist;
                } else if (*normal)[j as usize] == -1.0 {
                    mid[j as usize] = -dist;
                } else {
                    mid[j as usize] = (*p1)[j as usize] + dot * ((*p2)[j as usize] - (*p1)[j as usize]);
                }
                j += 1;
            }

            VectorCopy(addr_of!(mid), addr_of_mut!((*f).p[(*f).numpoints as usize]));
            (*f).numpoints += 1;

            i += 1;
        }

        if (*f).numpoints > maxpts {
            Com_Error(
                1, // ERR_DROP
                b"ClipWinding: points exceeded estimate\0".as_ptr() as *const c_char,
            );
        }
        if (*f).numpoints > MAX_POINTS_ON_WINDING {
            Com_Error(
                1, // ERR_DROP
                b"ClipWinding: MAX_POINTS_ON_WINDING\0".as_ptr() as *const c_char,
            );
        }

        FreeWinding(r#in);
        *inout = f;
    }
}

/*
=================
ChopWinding

Returns the fragment of in that is on the front side
of the cliping plane.  The original is freed.
=================
*/
pub fn ChopWinding(r#in: *mut winding_t, normal: *mut vec3_t, dist: vec_t) -> *mut winding_t {
    unsafe {
        let mut f: *mut winding_t = core::ptr::null_mut();
        let mut b: *mut winding_t = core::ptr::null_mut();

        ClipWindingEpsilon(r#in, normal, dist, ON_EPSILON, addr_of_mut!(f), addr_of_mut!(b));
        FreeWinding(r#in);
        if !b.is_null() {
            FreeWinding(b);
        }
        f
    }
}

/*
=================
CheckWinding

=================
*/
pub fn CheckWinding(w: *mut winding_t) {
    unsafe {
        let mut i: c_int;
        let mut j: c_int;
        let mut p1: *mut vec3_t;
        let mut p2: *mut vec3_t;
        let mut d: vec_t;
        let mut edgedist: vec_t;
        let mut dir: vec3_t = [0.0; 3];
        let mut edgenormal: vec3_t = [0.0; 3];
        let mut facenormal: vec3_t = [0.0; 3];
        let mut area: vec_t;
        let mut facedist: vec_t = 0.0;

        if (*w).numpoints < 3 {
            Com_Error(
                1, // ERR_DROP
                b"CheckWinding: %i points\0".as_ptr() as *const c_char,
                (*w).numpoints,
            );
        }

        area = WindingArea(w);
        if area < 1.0 {
            Com_Error(
                1, // ERR_DROP
                b"CheckWinding: %f area\0".as_ptr() as *const c_char,
                area,
            );
        }

        WindingPlane(w, addr_of_mut!(facenormal), addr_of_mut!(facedist));

        i = 0;
        while i < (*w).numpoints {
            p1 = addr_of_mut!((*w).p[i as usize]);

            j = 0;
            while j < 3 {
                if (*p1)[j as usize] > (MAX_MAP_BOUNDS as vec_t) || (*p1)[j as usize] < (-(MAX_MAP_BOUNDS as vec_t)) {
                    Com_Error(
                        1, // ERR_DROP
                        b"CheckFace: BOGUS_RANGE: %f\0".as_ptr() as *const c_char,
                        (*p1)[j as usize],
                    );
                }
                j += 1;
            }

            j = if i + 1 == (*w).numpoints { 0 } else { i + 1 };

            // check the point is on the face plane
            d = DotProduct(p1, addr_of!(facenormal)) - facedist;
            if d < -ON_EPSILON || d > ON_EPSILON {
                Com_Error(
                    1, // ERR_DROP
                    b"CheckWinding: point off plane\0".as_ptr() as *const c_char,
                );
            }

            // check the edge isnt degenerate
            p2 = addr_of_mut!((*w).p[j as usize]);
            VectorSubtract(p2, p1, addr_of_mut!(dir));

            if VectorLength(addr_of!(dir)) < ON_EPSILON {
                Com_Error(
                    1, // ERR_DROP
                    b"CheckWinding: degenerate edge\0".as_ptr() as *const c_char,
                );
            }

            CrossProduct(addr_of!(facenormal), addr_of!(dir), addr_of_mut!(edgenormal));
            VectorNormalize2(addr_of!(edgenormal), addr_of_mut!(edgenormal));
            edgedist = DotProduct(p1, addr_of!(edgenormal));
            edgedist += ON_EPSILON;

            // all other points must be on front side
            j = 0;
            while j < (*w).numpoints {
                if j == i {
                    j += 1;
                    continue;
                }
                d = DotProduct(addr_of!((*w).p[j as usize]), addr_of!(edgenormal));
                if d > edgedist {
                    Com_Error(
                        1, // ERR_DROP
                        b"CheckWinding: non-convex\0".as_ptr() as *const c_char,
                    );
                }
                j += 1;
            }

            i += 1;
        }
    }
}

/*
============
WindingOnPlaneSide
============
*/
pub fn WindingOnPlaneSide(w: *mut winding_t, normal: *mut vec3_t, dist: vec_t) -> c_int {
    unsafe {
        let mut front: c_int = 0; // qfalse
        let mut back: c_int = 0; // qfalse
        let mut i: c_int;
        let mut d: vec_t;

        i = 0;
        while i < (*w).numpoints {
            d = DotProduct(addr_of!((*w).p[i as usize]), normal) - dist;
            if d < -ON_EPSILON {
                if front != 0 {
                    return SIDE_CROSS;
                }
                back = 1; // qtrue
                i += 1;
                continue;
            }
            if d > ON_EPSILON {
                if back != 0 {
                    return SIDE_CROSS;
                }
                front = 1; // qtrue
                i += 1;
                continue;
            }
            i += 1;
        }

        if back != 0 {
            return SIDE_BACK;
        }
        if front != 0 {
            return SIDE_FRONT;
        }
        SIDE_ON
    }
}

/*
=================
AddWindingToConvexHull

Both w and *hull are on the same plane
=================
*/
const MAX_HULL_POINTS: usize = 128;

pub fn AddWindingToConvexHull(w: *mut winding_t, hull: *mut *mut winding_t, normal: *mut vec3_t) {
    unsafe {
        let mut i: c_int;
        let mut j: c_int;
        let mut k: c_int;
        let mut p: *mut vec3_t;
        let mut copy: *mut vec3_t;
        let mut dir: vec3_t = [0.0; 3];
        let mut d: vec_t;
        let mut numHullPoints: c_int;
        let mut numNew: c_int;
        let mut hullPoints: [[vec_t; 3]; MAX_HULL_POINTS] = [[0.0; 3]; MAX_HULL_POINTS];
        let mut newHullPoints: [[vec_t; 3]; MAX_HULL_POINTS] = [[0.0; 3]; MAX_HULL_POINTS];
        let mut hullDirs: [[vec_t; 3]; MAX_HULL_POINTS] = [[0.0; 3]; MAX_HULL_POINTS];
        let mut hullSide: [c_int; MAX_HULL_POINTS] = [0; MAX_HULL_POINTS];
        let mut outside: c_int;

        if (*hull).is_null() {
            *hull = CopyWinding(w);
            return;
        }

        numHullPoints = (**hull).numpoints;
        core::ptr::copy_nonoverlapping(
            addr_of!((**hull).p[0]) as *const c_void,
            addr_of_mut!(hullPoints[0]) as *mut c_void,
            (numHullPoints as usize) * core::mem::size_of::<vec3_t>(),
        );

        i = 0;
        while i < (*w).numpoints {
            p = addr_of_mut!((*w).p[i as usize]);

            // calculate hull side vectors
            j = 0;
            while j < numHullPoints {
                k = (j + 1) % numHullPoints;

                VectorSubtract(
                    addr_of!(hullPoints[k as usize]),
                    addr_of!(hullPoints[j as usize]),
                    addr_of_mut!(dir),
                );
                VectorNormalize2(addr_of!(dir), addr_of_mut!(dir));
                CrossProduct(normal, addr_of!(dir), addr_of_mut!(hullDirs[j as usize]));
                j += 1;
            }

            outside = 0; // qfalse
            j = 0;
            while j < numHullPoints {
                VectorSubtract(p, addr_of!(hullPoints[j as usize]), addr_of_mut!(dir));
                d = DotProduct(addr_of!(dir), addr_of!(hullDirs[j as usize]));
                if d >= ON_EPSILON {
                    outside = 1; // qtrue
                }
                if d >= -ON_EPSILON {
                    hullSide[j as usize] = 1; // qtrue
                } else {
                    hullSide[j as usize] = 0; // qfalse
                }
                j += 1;
            }

            // if the point is effectively inside, do nothing
            if outside == 0 {
                i += 1;
                continue;
            }

            // find the back side to front side transition
            j = 0;
            loop {
                if j >= numHullPoints {
                    break;
                }
                if hullSide[(j % numHullPoints) as usize] == 0
                    && hullSide[(((j + 1) % numHullPoints) as usize)] != 0
                {
                    break;
                }
                j += 1;
            }
            if j == numHullPoints {
                i += 1;
                continue;
            }

            // insert the point here
            VectorCopy(p, addr_of_mut!(newHullPoints[0]));
            numNew = 1;

            // copy over all points that aren't double fronts
            j = (j + 1) % numHullPoints;
            k = 0;
            while k < numHullPoints {
                if hullSide[(((j + k) % numHullPoints) as usize)] != 0
                    && hullSide[(((j + k + 1) % numHullPoints) as usize)] != 0
                {
                    k += 1;
                    continue;
                }
                copy = addr_of_mut!(hullPoints[(((j + k + 1) % numHullPoints) as usize)]);
                VectorCopy(copy, addr_of_mut!(newHullPoints[numNew as usize]));
                numNew += 1;
                k += 1;
            }

            numHullPoints = numNew;
            core::ptr::copy_nonoverlapping(
                addr_of_mut!(newHullPoints[0]) as *const c_void,
                addr_of_mut!(hullPoints[0]) as *mut c_void,
                (numHullPoints as usize) * core::mem::size_of::<vec3_t>(),
            );

            i += 1;
        }

        FreeWinding(*hull);
        w = AllocWinding(numHullPoints);
        (*w).numpoints = numHullPoints;
        *hull = w;
        core::ptr::copy_nonoverlapping(
            addr_of!(hullPoints[0]) as *const c_void,
            addr_of_mut!((*w).p[0]) as *mut c_void,
            (numHullPoints as usize) * core::mem::size_of::<vec3_t>(),
        );
    }
}
