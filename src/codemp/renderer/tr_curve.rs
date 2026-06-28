// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #include "tr_local.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::addr_of_mut;

/*

This file does all of the processing necessary to turn a raw grid of points
read from the map file into a srfGridMesh_t ready for rendering.

The level of detail solution is direction independent, based only on subdivided
distance from the true curve.

Only a single entry point:

srfGridMesh_t *R_SubdividePatchToGrid( int width, int height,
								drawVert_t points[MAX_PATCH_SIZE*MAX_PATCH_SIZE] ) {

*/

// External type declarations
#[repr(C)]
pub struct vec3_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct drawVert_t {
    pub xyz: [f32; 3],
    pub st: [f32; 2],
    pub lightmap: [[f32; 2]; 4],
    pub normal: [f32; 3],
    pub color: [[u8; 4]; 4],
}

#[repr(C)]
pub struct srfGridMesh_t {
    pub surfaceType: c_int,
    pub dlightBits: c_int,
    pub meshBounds: [[f32; 3]; 2],
    pub localOrigin: [f32; 3],
    pub meshRadius: f32,
    pub lodOrigin: [f32; 3],
    pub lodRadius: f32,
    pub lodFixed: c_int,
    pub lodStitched: c_int,
    pub width: c_int,
    pub height: c_int,
    pub widthLodError: *mut f32,
    pub heightLodError: *mut f32,
    pub verts: [u8; 1],
}

#[repr(C)]
pub struct cvar_t {
    pub value: f32,
}

// Constants
const MAX_GRID_SIZE: usize = 65;
const MAX_PATCH_SIZE: usize = 32;
const MAXLIGHTMAPS: usize = 4;
const TAG_GRIDMESH: c_int = 39; // from codemp/qcommon/tags.h
const SF_GRID: c_int = 3;

// Configuration flag
const PATCH_STITCHING: bool = true;

// External function declarations
extern "C" {
    fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorClear(v: *mut f32);
    fn VectorLengthSquared(v: *const f32) -> f32;
    fn VectorNormalize(v: *mut f32) -> f32;
    fn VectorNormalize2(src: *const f32, dst: *mut f32) -> f32;
    fn CrossProduct(src1: *const f32, src2: *const f32, out: *mut f32);
    fn VectorAdd(a: *const f32, b: *const f32, out: *mut f32);
    fn VectorScale(src: *const f32, scale: f32, out: *mut f32);
    fn VectorLength(v: *const f32) -> f32;
    fn DotProduct(a: *const f32, b: *const f32) -> f32;
    fn ClearBounds(mins: *mut f32, maxs: *mut f32);
    fn AddPointToBounds(v: *const f32, mins: *mut f32, maxs: *mut f32);
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Com_Memcpy(dst: *mut c_void, src: *const c_void, count: usize);
    fn Com_Memset(dst: *mut c_void, c: c_int, count: usize);
    fn Hunk_Alloc(size: usize) -> *mut c_void;

    // External cvar
    pub static r_subdivisions: *const cvar_t;
}

/*
============
LerpDrawVert
============
*/
unsafe fn LerpDrawVert(a: *const drawVert_t, b: *const drawVert_t, out: *mut drawVert_t) {
    let mut k: c_int;

    (*out).xyz[0] = 0.5 * ((*a).xyz[0] + (*b).xyz[0]);
    (*out).xyz[1] = 0.5 * ((*a).xyz[1] + (*b).xyz[1]);
    (*out).xyz[2] = 0.5 * ((*a).xyz[2] + (*b).xyz[2]);

    (*out).st[0] = 0.5 * ((*a).st[0] + (*b).st[0]);
    (*out).st[1] = 0.5 * ((*a).st[1] + (*b).st[1]);

    (*out).normal[0] = 0.5 * ((*a).normal[0] + (*b).normal[0]);
    (*out).normal[1] = 0.5 * ((*a).normal[1] + (*b).normal[1]);
    (*out).normal[2] = 0.5 * ((*a).normal[2] + (*b).normal[2]);

    k = 0;
    while k < MAXLIGHTMAPS as c_int {
        (*out).lightmap[k as usize][0] = 0.5 * ((*a).lightmap[k as usize][0] + (*b).lightmap[k as usize][0]);
        (*out).lightmap[k as usize][1] = 0.5 * ((*a).lightmap[k as usize][1] + (*b).lightmap[k as usize][1]);

        (*out).color[k as usize][0] = ((((*a).color[k as usize][0] as c_int
            + (*b).color[k as usize][0] as c_int)
            >> 1) & 0xFF) as u8;
        (*out).color[k as usize][1] = ((((*a).color[k as usize][1] as c_int
            + (*b).color[k as usize][1] as c_int)
            >> 1) & 0xFF) as u8;
        (*out).color[k as usize][2] = ((((*a).color[k as usize][2] as c_int
            + (*b).color[k as usize][2] as c_int)
            >> 1) & 0xFF) as u8;
        (*out).color[k as usize][3] = ((((*a).color[k as usize][3] as c_int
            + (*b).color[k as usize][3] as c_int)
            >> 1) & 0xFF) as u8;

        k += 1;
    }
}

/*
============
Transpose
============
*/
unsafe fn Transpose(width: c_int, height: c_int, ctrl: *mut [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE]) {
    let mut i: c_int;
    let mut j: c_int;
    let mut temp: drawVert_t;

    if width > height {
        i = 0;
        while i < height {
            j = i + 1;
            while j < width {
                if j < height {
                    // swap the value
                    temp = (*ctrl)[j as usize][i as usize];
                    (*ctrl)[j as usize][i as usize] = (*ctrl)[i as usize][j as usize];
                    (*ctrl)[i as usize][j as usize] = temp;
                } else {
                    // just copy
                    (*ctrl)[j as usize][i as usize] = (*ctrl)[i as usize][j as usize];
                }
                j += 1;
            }
            i += 1;
        }
    } else {
        i = 0;
        while i < width {
            j = i + 1;
            while j < height {
                if j < width {
                    // swap the value
                    temp = (*ctrl)[i as usize][j as usize];
                    (*ctrl)[i as usize][j as usize] = (*ctrl)[j as usize][i as usize];
                    (*ctrl)[j as usize][i as usize] = temp;
                } else {
                    // just copy
                    (*ctrl)[i as usize][j as usize] = (*ctrl)[j as usize][i as usize];
                }
                j += 1;
            }
            i += 1;
        }
    }
}

/*
=================
MakeMeshNormals

Handles all the complicated wrapping and degenerate cases
=================
*/
unsafe fn MakeMeshNormals(
    width: c_int,
    height: c_int,
    ctrl: *mut [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE],
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut dist: c_int;
    let mut normal: [f32; 3] = [0.0; 3];
    let mut sum: [f32; 3] = [0.0; 3];
    let mut count: c_int;
    let mut base: [f32; 3] = [0.0; 3];
    let mut delta: [f32; 3] = [0.0; 3];
    let mut x: c_int;
    let mut y: c_int;
    let mut dv: *const drawVert_t;
    let mut around: [[f32; 3]; 8] = [[0.0; 3]; 8];
    let mut temp: [f32; 3] = [0.0; 3];
    let mut good: [bool; 8] = [false; 8];
    let mut wrapWidth: bool;
    let mut wrapHeight: bool;
    let mut len: f32;
    static neighbors: [[c_int; 2]; 8] = [
        [0, 1],
        [1, 1],
        [1, 0],
        [1, -1],
        [0, -1],
        [-1, -1],
        [-1, 0],
        [-1, 1],
    ];

    wrapWidth = false;
    i = 0;
    while i < height {
        VectorSubtract(
            addr_of!((*ctrl)[i as usize][0].xyz[0]) as *const f32,
            addr_of!((*ctrl)[i as usize][(width - 1) as usize].xyz[0]) as *const f32,
            delta.as_mut_ptr(),
        );
        len = VectorLengthSquared(delta.as_ptr());
        if len > 1.0 {
            break;
        }
        i += 1;
    }
    if i == height {
        wrapWidth = true;
    }

    wrapHeight = false;
    i = 0;
    while i < width {
        VectorSubtract(
            addr_of!((*ctrl)[0][i as usize].xyz[0]) as *const f32,
            addr_of!((*ctrl)[(height - 1) as usize][i as usize].xyz[0]) as *const f32,
            delta.as_mut_ptr(),
        );
        len = VectorLengthSquared(delta.as_ptr());
        if len > 1.0 {
            break;
        }
        i += 1;
    }
    if i == width {
        wrapHeight = true;
    }

    i = 0;
    while i < width {
        j = 0;
        while j < height {
            count = 0;
            dv = addr_of!((*ctrl)[j as usize][i as usize]);
            VectorCopy(
                addr_of!((*dv).xyz[0]) as *const f32,
                base.as_mut_ptr(),
            );
            k = 0;
            while k < 8 {
                VectorClear(addr_of_mut!(around[k as usize]) as *mut [f32; 3] as *mut f32);
                good[k as usize] = false;

                dist = 1;
                while dist <= 3 {
                    x = i + neighbors[k as usize][0] * dist;
                    y = j + neighbors[k as usize][1] * dist;
                    if wrapWidth {
                        if x < 0 {
                            x = width - 1 + x;
                        } else if x >= width {
                            x = 1 + x - width;
                        }
                    }
                    if wrapHeight {
                        if y < 0 {
                            y = height - 1 + y;
                        } else if y >= height {
                            y = 1 + y - height;
                        }
                    }

                    if x < 0 || x >= width || y < 0 || y >= height {
                        break; // edge of patch
                    }
                    VectorSubtract(
                        addr_of!((*ctrl)[y as usize][x as usize].xyz[0]) as *const f32,
                        base.as_ptr(),
                        temp.as_mut_ptr(),
                    );
                    if VectorNormalize2(temp.as_ptr(), temp.as_mut_ptr()) == 0.0 {
                        dist += 1;
                        continue; // degenerate edge, get more dist
                    } else {
                        good[k as usize] = true;
                        VectorCopy(temp.as_ptr(), around[k as usize].as_mut_ptr());
                        break; // good edge
                    }
                }
                k += 1;
            }

            VectorClear(sum.as_mut_ptr());
            k = 0;
            while k < 8 {
                if !good[k as usize] || !good[((k + 1) & 7) as usize] {
                    k += 1;
                    continue; // didn't get two points
                }
                CrossProduct(
                    addr_of!(around[((k + 1) & 7) as usize]) as *const [f32; 3] as *const f32,
                    addr_of!(around[k as usize]) as *const [f32; 3] as *const f32,
                    normal.as_mut_ptr(),
                );
                if VectorNormalize2(normal.as_ptr(), normal.as_mut_ptr()) == 0.0 {
                    k += 1;
                    continue;
                }
                VectorAdd(normal.as_ptr(), sum.as_ptr(), sum.as_mut_ptr());
                count += 1;
                k += 1;
            }
            if count == 0 {
                //printf("bad normal\n");
                count = 1;
            }
            VectorNormalize2(sum.as_ptr(), addr_of_mut!((*dv).normal[0]) as *mut f32);
            j += 1;
        }
        i += 1;
    }
}

/*
============
InvertCtrl
============
*/
unsafe fn InvertCtrl(
    width: c_int,
    height: c_int,
    ctrl: *mut [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE],
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut temp: drawVert_t;

    i = 0;
    while i < height {
        j = 0;
        while j < width / 2 {
            temp = (*ctrl)[i as usize][j as usize];
            (*ctrl)[i as usize][j as usize] = (*ctrl)[i as usize][(width - 1 - j) as usize];
            (*ctrl)[i as usize][(width - 1 - j) as usize] = temp;
            j += 1;
        }
        i += 1;
    }
}

/*
=================
InvertErrorTable
=================
*/
unsafe fn InvertErrorTable(
    errorTable: *mut [[f32; MAX_GRID_SIZE]; 2],
    width: c_int,
    height: c_int,
) {
    let mut i: c_int;
    let mut copy: [[f32; MAX_GRID_SIZE]; 2] = [[0.0; MAX_GRID_SIZE]; 2];

    Com_Memcpy(
        addr_of_mut!(copy) as *mut c_void,
        errorTable as *const c_void,
        core::mem::size_of_val(&copy),
    );

    i = 0;
    while i < width {
        (*errorTable)[1][i as usize] = copy[0][i as usize]; //[width-1-i];
        i += 1;
    }

    i = 0;
    while i < height {
        (*errorTable)[0][i as usize] = copy[1][(height - 1 - i) as usize];
        i += 1;
    }
}

/*
==================
PutPointsOnCurve
==================
*/
unsafe fn PutPointsOnCurve(
    ctrl: *mut [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE],
    width: c_int,
    height: c_int,
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut prev: drawVert_t;
    let mut next: drawVert_t;

    i = 0;
    while i < width {
        j = 1;
        while j < height {
            LerpDrawVert(
                addr_of!((*ctrl)[j as usize][i as usize]),
                addr_of!((*ctrl)[(j + 1) as usize][i as usize]),
                addr_of_mut!(prev),
            );
            LerpDrawVert(
                addr_of!((*ctrl)[j as usize][i as usize]),
                addr_of!((*ctrl)[(j - 1) as usize][i as usize]),
                addr_of_mut!(next),
            );
            LerpDrawVert(
                addr_of!(prev),
                addr_of!(next),
                addr_of_mut!((*ctrl)[j as usize][i as usize]),
            );
            j += 2;
        }
        i += 1;
    }

    j = 0;
    while j < height {
        i = 1;
        while i < width {
            LerpDrawVert(
                addr_of!((*ctrl)[j as usize][i as usize]),
                addr_of!((*ctrl)[j as usize][(i + 1) as usize]),
                addr_of_mut!(prev),
            );
            LerpDrawVert(
                addr_of!((*ctrl)[j as usize][i as usize]),
                addr_of!((*ctrl)[j as usize][(i - 1) as usize]),
                addr_of_mut!(next),
            );
            LerpDrawVert(
                addr_of!(prev),
                addr_of!(next),
                addr_of_mut!((*ctrl)[j as usize][i as usize]),
            );
            i += 2;
        }
        j += 1;
    }
}

/*
=================
R_CreateSurfaceGridMesh
=================
*/
pub unsafe fn R_CreateSurfaceGridMesh(
    width: c_int,
    height: c_int,
    ctrl: *const [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE],
    errorTable: *const [[f32; MAX_GRID_SIZE]; 2],
) -> *mut srfGridMesh_t {
    let mut i: c_int;
    let mut j: c_int;
    let size: usize;
    let mut vert: *mut drawVert_t;
    let mut tmpVec: [f32; 3] = [0.0; 3];
    let mut grid: *mut srfGridMesh_t;

    // copy the results out to a grid
    size = ((width * height - 1) as usize * core::mem::size_of::<drawVert_t>()) + core::mem::size_of::<srfGridMesh_t>();

    if PATCH_STITCHING {
        grid = Z_Malloc(size, TAG_GRIDMESH, 1) as *mut srfGridMesh_t;
        Com_Memset(grid as *mut c_void, 0, size);

        (*grid).widthLodError = Z_Malloc((width as usize) * 4, TAG_GRIDMESH, 1) as *mut f32;
        Com_Memcpy(
            (*grid).widthLodError as *mut c_void,
            addr_of!((*errorTable)[0]) as *const c_void,
            (width as usize) * 4,
        );

        (*grid).heightLodError = Z_Malloc((height as usize) * 4, TAG_GRIDMESH, 1) as *mut f32;
        Com_Memcpy(
            (*grid).heightLodError as *mut c_void,
            addr_of!((*errorTable)[1]) as *const c_void,
            (height as usize) * 4,
        );
    } else {
        grid = Hunk_Alloc(size) as *mut srfGridMesh_t;
        Com_Memset(grid as *mut c_void, 0, size);

        (*grid).widthLodError = Hunk_Alloc((width as usize) * 4) as *mut f32;
        Com_Memcpy(
            (*grid).widthLodError as *mut c_void,
            addr_of!((*errorTable)[0]) as *const c_void,
            (width as usize) * 4,
        );

        (*grid).heightLodError = Hunk_Alloc((height as usize) * 4) as *mut f32;
        Com_Memcpy(
            (*grid).heightLodError as *mut c_void,
            addr_of!((*errorTable)[1]) as *const c_void,
            (height as usize) * 4,
        );
    }

    (*grid).width = width;
    (*grid).height = height;
    (*grid).surfaceType = SF_GRID;
    ClearBounds(
        addr_of_mut!((*grid).meshBounds[0][0]),
        addr_of_mut!((*grid).meshBounds[1][0]),
    );
    i = 0;
    while i < width {
        j = 0;
        while j < height {
            vert = (addr_of_mut!((*grid).verts[0]) as *mut drawVert_t)
                .offset(((j * width + i) as usize) as isize);
            *vert = (*ctrl)[j as usize][i as usize];
            AddPointToBounds(
                addr_of!((*vert).xyz[0]) as *const f32,
                addr_of_mut!((*grid).meshBounds[0][0]),
                addr_of_mut!((*grid).meshBounds[1][0]),
            );
            j += 1;
        }
        i += 1;
    }

    // compute local origin and bounds
    VectorAdd(
        addr_of!((*grid).meshBounds[0][0]),
        addr_of!((*grid).meshBounds[1][0]),
        addr_of_mut!((*grid).localOrigin[0]),
    );
    VectorScale(
        addr_of!((*grid).localOrigin[0]),
        0.5f32,
        addr_of_mut!((*grid).localOrigin[0]),
    );
    VectorSubtract(
        addr_of!((*grid).meshBounds[0][0]),
        addr_of!((*grid).localOrigin[0]),
        tmpVec.as_mut_ptr(),
    );
    (*grid).meshRadius = VectorLength(tmpVec.as_ptr());

    VectorCopy(
        addr_of!((*grid).localOrigin[0]),
        addr_of_mut!((*grid).lodOrigin[0]),
    );
    (*grid).lodRadius = (*grid).meshRadius;
    //
    grid
}

/*
=================
R_FreeSurfaceGridMesh
=================
*/
pub unsafe fn R_FreeSurfaceGridMesh(grid: *mut srfGridMesh_t) {
    Z_Free((*grid).widthLodError as *mut c_void);
    Z_Free((*grid).heightLodError as *mut c_void);
    Z_Free(grid as *mut c_void);
}

/*
=================
R_SubdividePatchToGrid
=================
*/
pub unsafe fn R_SubdividePatchToGrid(
    width: c_int,
    height: c_int,
    points: *const drawVert_t,
) -> *mut srfGridMesh_t {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut l: c_int;
    let mut prev: drawVert_t;
    let mut next: drawVert_t;
    let mut mid: drawVert_t;
    let mut len: f32;
    let mut maxLen: f32;
    let mut dir: c_int;
    let mut t: c_int;
    let mut ctrl: [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE] = Default::default();
    let mut errorTable: [[f32; MAX_GRID_SIZE]; 2] = [[0.0; MAX_GRID_SIZE]; 2];

    i = 0;
    while i < width {
        j = 0;
        while j < height {
            ctrl[j as usize][i as usize] = *points.offset((j * width + i) as isize);
            j += 1;
        }
        i += 1;
    }

    let mut width = width;
    let mut height = height;

    dir = 0;
    while dir < 2 {
        j = 0;
        while j < MAX_GRID_SIZE as c_int {
            errorTable[dir as usize][j as usize] = 0.0;
            j += 1;
        }

        // horizontal subdivisions
        j = 0;
        while j + 2 < width {
            // check subdivided midpoints against control points

            // FIXME: also check midpoints of adjacent patches against the control points
            // this would basically stitch all patches in the same LOD group together.

            maxLen = 0.0;
            i = 0;
            while i < height {
                let mut midxyz: [f32; 3] = [0.0; 3];
                let mut dirv: [f32; 3] = [0.0; 3];
                let mut projected: [f32; 3] = [0.0; 3];
                let mut d: f32;

                // calculate the point on the curve
                l = 0;
                while l < 3 {
                    midxyz[l as usize] = ((ctrl[i as usize][j as usize].xyz[l as usize] as f32
                        + ctrl[i as usize][(j + 1) as usize].xyz[l as usize] as f32 * 2.0
                        + ctrl[i as usize][(j + 2) as usize].xyz[l as usize] as f32)
                        * 0.25f32);
                    l += 1;
                }

                // see how far off the line it is
                // using dist-from-line will not account for internal
                // texture warping, but it gives a lot less polygons than
                // dist-from-midpoint
                VectorSubtract(
                    midxyz.as_ptr(),
                    addr_of!(ctrl[i as usize][j as usize].xyz[0]) as *const f32,
                    midxyz.as_mut_ptr(),
                );
                VectorSubtract(
                    addr_of!(ctrl[i as usize][(j + 2) as usize].xyz[0]) as *const f32,
                    addr_of!(ctrl[i as usize][j as usize].xyz[0]) as *const f32,
                    dirv.as_mut_ptr(),
                );
                VectorNormalize(dirv.as_mut_ptr());

                d = DotProduct(midxyz.as_ptr(), dirv.as_ptr());
                VectorScale(dirv.as_ptr(), d, projected.as_mut_ptr());
                VectorSubtract(
                    midxyz.as_ptr(),
                    projected.as_ptr(),
                    midxyz.as_mut_ptr(),
                );
                len = VectorLengthSquared(midxyz.as_ptr()); // we will do the sqrt later

                if len > maxLen {
                    maxLen = len;
                }
                i += 1;
            }

            maxLen = maxLen.sqrt();
            // if all the points are on the lines, remove the entire columns
            if maxLen < 0.1f32 {
                errorTable[dir as usize][(j + 1) as usize] = 999.0;
                j += 2;
                continue;
            }

            // see if we want to insert subdivided columns
            if width + 2 > MAX_GRID_SIZE as c_int {
                errorTable[dir as usize][(j + 1) as usize] = 1.0f32 / maxLen;
                j += 2;
                continue; // can't subdivide any more
            }

            if maxLen <= (*r_subdivisions).value {
                errorTable[dir as usize][(j + 1) as usize] = 1.0f32 / maxLen;
                j += 2;
                continue; // didn't need subdivision
            }

            errorTable[dir as usize][(j + 2) as usize] = 1.0f32 / maxLen;

            // insert two columns and replace the peak
            width += 2;
            i = 0;
            while i < height {
                LerpDrawVert(
                    addr_of!(ctrl[i as usize][j as usize]),
                    addr_of!(ctrl[i as usize][(j + 1) as usize]),
                    addr_of_mut!(prev),
                );
                LerpDrawVert(
                    addr_of!(ctrl[i as usize][(j + 1) as usize]),
                    addr_of!(ctrl[i as usize][(j + 2) as usize]),
                    addr_of_mut!(next),
                );
                LerpDrawVert(
                    addr_of!(prev),
                    addr_of!(next),
                    addr_of_mut!(mid),
                );

                k = width - 1;
                while k > j + 3 {
                    ctrl[i as usize][k as usize] = ctrl[i as usize][(k - 2) as usize];
                    k -= 1;
                }
                ctrl[i as usize][(j + 1) as usize] = prev;
                ctrl[i as usize][(j + 2) as usize] = mid;
                ctrl[i as usize][(j + 3) as usize] = next;
                i += 1;
            }

            // back up and recheck this set again, it may need more subdivision
            j -= 2;
            j += 2;
        }

        Transpose(width, height, addr_of_mut!(ctrl));
        t = width;
        width = height;
        height = t;
        dir += 1;
    }

    // put all the aproximating points on the curve
    PutPointsOnCurve(addr_of_mut!(ctrl), width, height);

    // cull out any rows or columns that are colinear
    i = 1;
    while i < width - 1 {
        if errorTable[0][i as usize] != 999.0 {
            i += 1;
            continue;
        }
        j = i + 1;
        while j < width {
            k = 0;
            while k < height {
                ctrl[k as usize][(j - 1) as usize] = ctrl[k as usize][j as usize];
                k += 1;
            }
            errorTable[0][(j - 1) as usize] = errorTable[0][j as usize];
            j += 1;
        }
        width -= 1;
    }

    i = 1;
    while i < height - 1 {
        if errorTable[1][i as usize] != 999.0 {
            i += 1;
            continue;
        }
        j = i + 1;
        while j < height {
            k = 0;
            while k < width {
                ctrl[j as usize - 1][k as usize] = ctrl[j as usize][k as usize];
                k += 1;
            }
            errorTable[1][(j - 1) as usize] = errorTable[1][j as usize];
            j += 1;
        }
        height -= 1;
    }

    // flip for longest tristrips as an optimization
    // the results should be visually identical with or
    // without this step
    if height > width {
        Transpose(width, height, addr_of_mut!(ctrl));
        InvertErrorTable(addr_of_mut!(errorTable), width, height);
        t = width;
        width = height;
        height = t;
        InvertCtrl(width, height, addr_of_mut!(ctrl));
    }

    // calculate normals
    MakeMeshNormals(width, height, addr_of_mut!(ctrl));

    R_CreateSurfaceGridMesh(width, height, addr_of!(ctrl), addr_of!(errorTable))
}

/*
===============
R_GridInsertColumn
===============
*/
pub unsafe fn R_GridInsertColumn(
    grid: *mut srfGridMesh_t,
    column: c_int,
    row: c_int,
    point: *const f32,
    loderror: f32,
) -> *mut srfGridMesh_t {
    let mut i: c_int;
    let mut j: c_int;
    let mut width: c_int;
    let mut height: c_int;
    let mut oldwidth: c_int;
    let mut ctrl: [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE] = Default::default();
    let mut errorTable: [[f32; MAX_GRID_SIZE]; 2] = [[0.0; MAX_GRID_SIZE]; 2];
    let mut lodRadius: f32;
    let mut lodOrigin: [f32; 3] = [0.0; 3];

    oldwidth = 0;
    width = (*grid).width + 1;
    if width > MAX_GRID_SIZE as c_int {
        return core::ptr::null_mut();
    }
    height = (*grid).height;
    i = 0;
    while i < width {
        if i == column {
            //insert new column
            j = 0;
            while j < (*grid).height {
                LerpDrawVert(
                    (addr_of!((*grid).verts[0]) as *mut drawVert_t)
                        .offset((j * (*grid).width + i - 1) as isize),
                    (addr_of!((*grid).verts[0]) as *mut drawVert_t).offset((j * (*grid).width + i) as isize),
                    addr_of_mut!(ctrl[j as usize][i as usize]),
                );
                if j == row {
                    VectorCopy(point, addr_of_mut!(ctrl[j as usize][i as usize].xyz[0]) as *mut f32);
                }
                j += 1;
            }
            errorTable[0][i as usize] = loderror;
            i += 1;
            continue;
        }
        errorTable[0][i as usize] = *(*grid).widthLodError.offset(oldwidth as isize);
        j = 0;
        while j < (*grid).height {
            ctrl[j as usize][i as usize] =
                *(addr_of!((*grid).verts[0]) as *mut drawVert_t).offset((j * (*grid).width + oldwidth) as isize);
            j += 1;
        }
        oldwidth += 1;
        i += 1;
    }
    j = 0;
    while j < (*grid).height {
        errorTable[1][j as usize] = *(*grid).heightLodError.offset(j as isize);
        j += 1;
    }
    // put all the aproximating points on the curve
    //PutPointsOnCurve( ctrl, width, height );
    // calculate normals
    MakeMeshNormals(width, height, addr_of_mut!(ctrl));

    VectorCopy(
        addr_of!((*grid).lodOrigin[0]),
        lodOrigin.as_mut_ptr(),
    );
    lodRadius = (*grid).lodRadius;
    // free the old grid
    R_FreeSurfaceGridMesh(grid);
    // create a new grid
    let grid = R_CreateSurfaceGridMesh(width, height, addr_of!(ctrl), addr_of!(errorTable));
    (*grid).lodRadius = lodRadius;
    VectorCopy(
        lodOrigin.as_ptr(),
        addr_of_mut!((*grid).lodOrigin[0]),
    );
    grid
}

/*
===============
R_GridInsertRow
===============
*/
pub unsafe fn R_GridInsertRow(
    grid: *mut srfGridMesh_t,
    row: c_int,
    column: c_int,
    point: *const f32,
    loderror: f32,
) -> *mut srfGridMesh_t {
    let mut i: c_int;
    let mut j: c_int;
    let mut width: c_int;
    let mut height: c_int;
    let mut oldheight: c_int;
    let mut ctrl: [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE] = Default::default();
    let mut errorTable: [[f32; MAX_GRID_SIZE]; 2] = [[0.0; MAX_GRID_SIZE]; 2];
    let mut lodRadius: f32;
    let mut lodOrigin: [f32; 3] = [0.0; 3];

    oldheight = 0;
    width = (*grid).width;
    height = (*grid).height + 1;
    if height > MAX_GRID_SIZE as c_int {
        return core::ptr::null_mut();
    }
    i = 0;
    while i < height {
        if i == row {
            //insert new row
            j = 0;
            while j < (*grid).width {
                LerpDrawVert(
                    (addr_of!((*grid).verts[0]) as *mut drawVert_t)
                        .offset(((i - 1) * (*grid).width + j) as isize),
                    (addr_of!((*grid).verts[0]) as *mut drawVert_t).offset((i * (*grid).width + j) as isize),
                    addr_of_mut!(ctrl[i as usize][j as usize]),
                );
                if j == column {
                    VectorCopy(point, addr_of_mut!(ctrl[i as usize][j as usize].xyz[0]) as *mut f32);
                }
                j += 1;
            }
            errorTable[1][i as usize] = loderror;
            i += 1;
            continue;
        }
        errorTable[1][i as usize] = *(*grid).heightLodError.offset(oldheight as isize);
        j = 0;
        while j < (*grid).width {
            ctrl[i as usize][j as usize] =
                *(addr_of!((*grid).verts[0]) as *mut drawVert_t).offset((oldheight * (*grid).width + j) as isize);
            j += 1;
        }
        oldheight += 1;
        i += 1;
    }
    j = 0;
    while j < width {
        errorTable[0][j as usize] = *(*grid).widthLodError.offset(j as isize);
        j += 1;
    }
    // put all the aproximating points on the curve
    //PutPointsOnCurve( ctrl, width, height );
    // calculate normals
    MakeMeshNormals(width, height, addr_of_mut!(ctrl));

    VectorCopy(
        addr_of!((*grid).lodOrigin[0]),
        lodOrigin.as_mut_ptr(),
    );
    lodRadius = (*grid).lodRadius;
    // free the old grid
    R_FreeSurfaceGridMesh(grid);
    // create a new grid
    let grid = R_CreateSurfaceGridMesh(width, height, addr_of!(ctrl), addr_of!(errorTable));
    (*grid).lodRadius = lodRadius;
    VectorCopy(
        lodOrigin.as_ptr(),
        addr_of_mut!((*grid).lodOrigin[0]),
    );
    grid
}
