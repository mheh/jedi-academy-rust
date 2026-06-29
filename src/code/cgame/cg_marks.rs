// cg_marks.c -- wall marks

// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"

// //#include "cg_local.h"
// #include "cg_media.h"

use core::ffi::{c_int, c_char, c_void};
use std::ptr::{self, addr_of, addr_of_mut};

// Type definitions for the renderer and game structs
#[repr(C)]
pub struct polyVert_t {
    pub xyz: [f32; 3],
    pub st: [f32; 2],
    pub modulate: [u8; 4],
}

#[repr(C)]
pub struct poly_t {
    pub hShader: c_int,
    pub numVerts: c_int,
    pub verts: *const polyVert_t,
}

#[repr(C)]
pub struct markFragment_t {
    pub firstPoint: c_int,
    pub numPoints: c_int,
}

// Constants
const MAX_MARK_POLYS: usize = 256;
const MAX_VERTS_ON_POLY: usize = 10;
const MAX_MARK_FRAGMENTS: usize = 128;
const MAX_MARK_POINTS: usize = 384;
const MARK_TOTAL_TIME: c_int = 10000;
const MARK_FADE_TIME: c_int = 1000;

// Type for mark poly - the main struct for marks
#[repr(C)]
pub struct markPoly_t {
    pub prevMark: *mut markPoly_t,
    pub nextMark: *mut markPoly_t,
    pub time: c_int,
    pub markShader: c_int,
    pub alphaFade: c_int,
    pub color: [f32; 4],
    pub poly: poly_t,
    pub verts: [polyVert_t; MAX_VERTS_ON_POLY],
}

// Extern types and globals needed
#[repr(C)]
pub struct vmCvar_t {
    pub integer: c_int,
    // ... other fields not used here
}

#[repr(C)]
pub struct cg_t {
    // ... many fields, only time is used here
    pub time: c_int,
    // ... other fields at various offsets
}

/*
===================================================================

MARK POLYS

===================================================================
*/

// double linked list
static mut cg_activeMarkPolys: markPoly_t = markPoly_t {
    prevMark: ptr::null_mut(),
    nextMark: ptr::null_mut(),
    time: 0,
    markShader: 0,
    alphaFade: 0,
    color: [0.0; 4],
    poly: poly_t {
        hShader: 0,
        numVerts: 0,
        verts: ptr::null(),
    },
    verts: [polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    }; MAX_VERTS_ON_POLY],
};

// single linked list
static mut cg_freeMarkPolys: *mut markPoly_t = ptr::null_mut();

static mut cg_markPolys: [markPoly_t; MAX_MARK_POLYS] = [markPoly_t {
    prevMark: ptr::null_mut(),
    nextMark: ptr::null_mut(),
    time: 0,
    markShader: 0,
    alphaFade: 0,
    color: [0.0; 4],
    poly: poly_t {
        hShader: 0,
        numVerts: 0,
        verts: ptr::null(),
    },
    verts: [polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    }; MAX_VERTS_ON_POLY],
}; MAX_MARK_POLYS];

// External references
extern "C" {
    pub static mut cg_addMarks: vmCvar_t;
    pub static mut cg: cg_t;

    pub fn CG_Error(msg: *const c_char, ...);
    pub fn cgi_CM_MarkFragments(
        numPoints: c_int,
        points: *const [f32; 3],
        projection: [f32; 3],
        maxPoints: c_int,
        pointBuffer: *mut [f32; 3],
        maxFragments: c_int,
        fragmentBuffer: *mut markFragment_t,
    ) -> c_int;
    pub fn cgi_R_AddPolyToScene(hShader: c_int, numVerts: c_int, verts: *const polyVert_t);

    // Math functions
    pub fn VectorNormalize2(v: *const [f32; 3], out: *mut [f32; 3]);
    pub fn PerpendicularVector(dst: *mut [f32; 3], src: *const [f32; 3]);
    pub fn RotatePointAroundVector(dst: *mut [f32; 3], axis: *const [f32; 3], point: *const [f32; 3], degrees: f32);
    pub fn CrossProduct(v1: *const [f32; 3], v2: *const [f32; 3], cross: *mut [f32; 3]);
    pub fn VectorScale(v: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    pub fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    pub fn VectorSubtract(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn DotProduct(v1: *const [f32; 3], v2: *const [f32; 3]) -> f32;

    pub fn memset(dst: *mut c_void, val: c_int, len: usize) -> *mut c_void;
}

/*
===================
CG_InitMarkPolys

This is called at startup and for tournement restarts
===================
*/
pub unsafe extern "C" fn CG_InitMarkPolys() {
    let mut i: c_int = 0;

    memset(
        addr_of_mut!(cg_markPolys) as *mut c_void,
        0,
        std::mem::size_of_val(&cg_markPolys),
    );

    (*addr_of_mut!(cg_activeMarkPolys)).nextMark = addr_of_mut!(cg_activeMarkPolys);
    (*addr_of_mut!(cg_activeMarkPolys)).prevMark = addr_of_mut!(cg_activeMarkPolys);
    cg_freeMarkPolys = cg_markPolys.as_mut_ptr();
    i = 0;
    while i < (MAX_MARK_POLYS as c_int) - 1 {
        (*cg_markPolys.as_mut_ptr().add(i as usize)).nextMark = cg_markPolys.as_mut_ptr().add((i + 1) as usize);
        i += 1;
    }
}

/*
==================
CG_FreeMarkPoly
==================
*/
pub unsafe extern "C" fn CG_FreeMarkPoly(le: *mut markPoly_t) {
    if (*le).prevMark.is_null() {
        CG_Error(b"CG_FreeLocalEntity: not active\0".as_ptr() as *const c_char);
    }

    // remove from the doubly linked active list
    (*(*le).prevMark).nextMark = (*le).nextMark;
    (*(*le).nextMark).prevMark = (*le).prevMark;

    // the free list is only singly linked
    (*le).nextMark = cg_freeMarkPolys;
    cg_freeMarkPolys = le;
}

/*
===================
CG_AllocMark

Will allways succeed, even if it requires freeing an old active mark
===================
*/
pub unsafe extern "C" fn CG_AllocMark() -> *mut markPoly_t {
    let mut le: *mut markPoly_t = ptr::null_mut();
    let mut time: c_int = 0;

    if cg_freeMarkPolys.is_null() {
        // no free entities, so free the one at the end of the chain
        // remove the oldest active entity
        time = (*(*addr_of!(cg_activeMarkPolys)).prevMark).time;
        while !(*addr_of!(cg_activeMarkPolys)).prevMark.is_null()
            && time == (*(*addr_of!(cg_activeMarkPolys)).prevMark).time
        {
            CG_FreeMarkPoly((*addr_of!(cg_activeMarkPolys)).prevMark);
        }
    }

    le = cg_freeMarkPolys;
    cg_freeMarkPolys = (*cg_freeMarkPolys).nextMark;

    memset(
        le as *mut c_void,
        0,
        std::mem::size_of::<markPoly_t>(),
    );

    // link into the active list
    (*le).nextMark = (*addr_of!(cg_activeMarkPolys)).nextMark;
    (*le).prevMark = addr_of_mut!(cg_activeMarkPolys);
    (*(*addr_of!(cg_activeMarkPolys)).nextMark).prevMark = le;
    (*addr_of_mut!(cg_activeMarkPolys)).nextMark = le;
    le
}

/*
=================
CG_ImpactMark

origin should be a point within a unit of the plane
dir should be the plane normal

temporary marks will not be stored or randomly oriented, but immediately
passed to the renderer.
=================
*/
pub unsafe extern "C" fn CG_ImpactMark(
    markShader: c_int,
    origin: *const [f32; 3],
    dir: *const [f32; 3],
    orientation: f32,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    alphaFade: c_int,
    radius: f32,
    temporary: c_int,
) {
    let mut axis: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut texCoordScale: f32 = 0.0;
    let mut originalPoints: [[f32; 3]; 4] = [[0.0; 3]; 4];
    let mut colors: [u8; 4] = [0; 4];
    let mut i: c_int = 0;
    let mut j: c_int = 0;
    let mut numFragments: c_int = 0;
    let mut markFragments: [markFragment_t; MAX_MARK_FRAGMENTS] = [markFragment_t {
        firstPoint: 0,
        numPoints: 0,
    }; MAX_MARK_FRAGMENTS];
    let mut mf: *mut markFragment_t = ptr::null_mut();
    let mut markPoints: [[f32; 3]; MAX_MARK_POINTS] = [[0.0; 3]; MAX_MARK_POINTS];
    let mut projection: [f32; 3] = [0.0; 3];

    if (*addr_of!(cg_addMarks)).integer == 0 {
        return;
    }

    if radius <= 0.0 {
        CG_Error(b"CG_ImpactMark called with <= 0 radius\0".as_ptr() as *const c_char);
    }

    // create the texture axis
    VectorNormalize2(dir, &mut axis[0]);
    PerpendicularVector(&mut axis[1], &axis[0]);
    RotatePointAroundVector(&mut axis[2], &axis[0], &axis[1], orientation);
    CrossProduct(&axis[0], &axis[2], &mut axis[1]);

    texCoordScale = 0.5 * 1.0 / radius;

    // create the full polygon
    i = 0;
    while i < 3 {
        originalPoints[0][i as usize] = (*origin)[i as usize]
            - radius * axis[1][i as usize]
            - radius * axis[2][i as usize];
        originalPoints[1][i as usize] = (*origin)[i as usize]
            + radius * axis[1][i as usize]
            - radius * axis[2][i as usize];
        originalPoints[2][i as usize] = (*origin)[i as usize]
            + radius * axis[1][i as usize]
            + radius * axis[2][i as usize];
        originalPoints[3][i as usize] = (*origin)[i as usize]
            - radius * axis[1][i as usize]
            + radius * axis[2][i as usize];
        i += 1;
    }

    // get the fragments
    VectorScale(dir, -20.0, &mut projection);
    numFragments = cgi_CM_MarkFragments(
        4,
        originalPoints.as_ptr(),
        projection,
        MAX_MARK_POINTS as c_int,
        markPoints.as_mut_ptr(),
        MAX_MARK_FRAGMENTS as c_int,
        markFragments.as_mut_ptr(),
    );

    colors[0] = (red * 255.0) as u8;
    colors[1] = (green * 255.0) as u8;
    colors[2] = (blue * 255.0) as u8;
    colors[3] = (alpha * 255.0) as u8;

    i = 0;
    mf = markFragments.as_mut_ptr();
    while i < numFragments {
        let mut v: *mut polyVert_t = ptr::null_mut();
        let mut verts: [polyVert_t; MAX_VERTS_ON_POLY] = [polyVert_t {
            xyz: [0.0; 3],
            st: [0.0; 2],
            modulate: [0; 4],
        }; MAX_VERTS_ON_POLY];
        let mut mark: *mut markPoly_t = ptr::null_mut();

        // we have an upper limit on the complexity of polygons
        // that we store persistantly
        if (*mf).numPoints > MAX_VERTS_ON_POLY as c_int {
            (*mf).numPoints = MAX_VERTS_ON_POLY as c_int;
        }
        j = 0;
        v = verts.as_mut_ptr();
        while j < (*mf).numPoints {
            let mut delta: [f32; 3] = [0.0; 3];

            let idx = ((*mf).firstPoint + j) as usize;
            if idx < MAX_MARK_POINTS {
                VectorCopy(
                    &markPoints[idx],
                    &mut (*v).xyz,
                );
            }

            VectorSubtract(&(*v).xyz, origin, &mut delta);
            (*v).st[0] = 0.5 + DotProduct(&delta, &axis[1]) * texCoordScale;
            (*v).st[1] = 0.5 + DotProduct(&delta, &axis[2]) * texCoordScale;
            // Copy color data: *(int *)v->modulate = *(int *)colors;
            (*v).modulate[0] = colors[0];
            (*v).modulate[1] = colors[1];
            (*v).modulate[2] = colors[2];
            (*v).modulate[3] = colors[3];

            j += 1;
            v = v.add(1);
        }

        // if it is a temporary (shadow) mark, add it immediately and forget about it
        if temporary != 0 {
            cgi_R_AddPolyToScene(markShader, (*mf).numPoints, verts.as_ptr());
        } else {
            // otherwise save it persistantly
            mark = CG_AllocMark();
            (*mark).time = (*addr_of!(cg)).time;
            (*mark).alphaFade = alphaFade;
            (*mark).markShader = markShader;
            (*mark).poly.numVerts = (*mf).numPoints;
            (*mark).color[0] = colors[0] as f32; // red
            (*mark).color[1] = colors[1] as f32; // green
            (*mark).color[2] = colors[2] as f32; // blue
            (*mark).color[3] = colors[3] as f32; // alpha
            // memcpy( mark->verts, verts, mf->numPoints * sizeof( verts[0] ) );
            let copy_count = (*mf).numPoints as usize;
            if copy_count <= MAX_VERTS_ON_POLY {
                ptr::copy_nonoverlapping(
                    verts.as_ptr(),
                    (*mark).verts.as_mut_ptr(),
                    copy_count,
                );
            }
        }

        i += 1;
        mf = mf.add(1);
    }
}

/*
===============
CG_AddMarks
===============
*/
pub unsafe extern "C" fn CG_AddMarks() {
    let mut j: c_int = 0;
    let mut mp: *mut markPoly_t = ptr::null_mut();
    let mut next: *mut markPoly_t = ptr::null_mut();
    let mut t: c_int = 0;
    let mut fade: c_int = 0;

    if (*addr_of!(cg_addMarks)).integer == 0 {
        return;
    }

    mp = (*addr_of!(cg_activeMarkPolys)).nextMark;
    while mp != addr_of_mut!(cg_activeMarkPolys) {
        // grab next now, so if the local entity is freed we
        // still have it
        next = (*mp).nextMark;

        // see if it is time to completely remove it
        if (*addr_of!(cg)).time > (*mp).time + MARK_TOTAL_TIME {
            CG_FreeMarkPoly(mp);
        } else {
            // fade all marks out with time
            t = (*mp).time + MARK_TOTAL_TIME - (*addr_of!(cg)).time;
            if t < MARK_FADE_TIME {
                fade = 255 * t / MARK_FADE_TIME;
                if (*mp).alphaFade != 0 {
                    j = 0;
                    while j < (*mp).poly.numVerts {
                        (*mp).verts[j as usize].modulate[3] = fade as u8;
                        j += 1;
                    }
                } else {
                    let f: f32 = t as f32 / MARK_FADE_TIME as f32;
                    j = 0;
                    while j < (*mp).poly.numVerts {
                        (*mp).verts[j as usize].modulate[0] = ((*mp).color[0] * f) as u8;
                        (*mp).verts[j as usize].modulate[1] = ((*mp).color[1] * f) as u8;
                        (*mp).verts[j as usize].modulate[2] = ((*mp).color[2] * f) as u8;
                        j += 1;
                    }
                }
            } else {
                j = 0;
                while j < (*mp).poly.numVerts {
                    (*mp).verts[j as usize].modulate[0] = (*mp).color[0] as u8;
                    (*mp).verts[j as usize].modulate[1] = (*mp).color[1] as u8;
                    (*mp).verts[j as usize].modulate[2] = (*mp).color[2] as u8;
                    j += 1;
                }
            }

            cgi_R_AddPolyToScene((*mp).markShader, (*mp).poly.numVerts, (*mp).verts.as_ptr());
        }

        mp = next;
    }
}
