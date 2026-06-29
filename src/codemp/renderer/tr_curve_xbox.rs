//Anything above this #include will be ignored by the compiler

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::addr_of;
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
#[derive(Copy, Clone)]
pub struct drawVert_t {
    pub xyz: [f32; 3],
    pub dvst: [i16; 2],
    pub normal: [f32; 3],
    pub dvlightmap: [[i16; 2]; 4],
    pub dvcolor: [[u8; 2]; 4],
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
const TAG_GRIDMESH: c_int = 39;
const SF_GRID: c_int = 3;
const h_low: c_int = 2;

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

    (*out).dvst[0] = (0.5 * ((*a).dvst[0] as f32 + (*b).dvst[0] as f32)) as i16;
    (*out).dvst[1] = (0.5 * ((*a).dvst[1] as f32 + (*b).dvst[1] as f32)) as i16;

    (*out).normal[0] = 0.5 * ((*a).normal[0] + (*b).normal[0]);
    (*out).normal[1] = 0.5 * ((*a).normal[1] + (*b).normal[1]);
    (*out).normal[2] = 0.5 * ((*a).normal[2] + (*b).normal[2]);

    k = 0;
    while k < MAXLIGHTMAPS as c_int {
        (*out).dvlightmap[k as usize][0] =
            (0.5 * ((*a).dvlightmap[k as usize][0] as f32 + (*b).dvlightmap[k as usize][0] as f32)) as i16;
        (*out).dvlightmap[k as usize][1] =
            (0.5 * ((*a).dvlightmap[k as usize][1] as f32 + (*b).dvlightmap[k as usize][1] as f32)) as i16;

        // Need to do averaging per every four bits
        let mut j: c_int = 0;
        while j < 2 {
            let ah: u8 = ((*a).dvcolor[k as usize][j as usize] >> 4) & 0x0F;
            let al: u8 = (*a).dvcolor[k as usize][j as usize] & 0x0F;
            let bh: u8 = ((*b).dvcolor[k as usize][j as usize] >> 4) & 0x0F;
            let bl: u8 = (*b).dvcolor[k as usize][j as usize] & 0x0F;
            (*out).dvcolor[k as usize][j as usize] =
                ((((ah as c_int + bh as c_int) / 2) << 4) | ((al as c_int + bl as c_int) / 2)) as u8;
            j += 1;
        }
        k += 1;
    }
}

/*
============
Transpose
============
*/
unsafe fn Transpose(width: c_int, height: c_int, ctrl: *mut drawVert_t) {
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
                    temp = *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize));
                    *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize)) =
                        *(ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize));
                    *(ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize)) = temp;
                } else {
                    // just copy
                    *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize)) =
                        *(ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize));
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
                    temp = *(ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize));
                    *(ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize)) =
                        *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize));
                    *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize)) = temp;
                } else {
                    // just copy
                    *(ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize)) =
                        *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize));
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
unsafe fn MakeMeshNormals(width: c_int, height: c_int, ctrl: *mut drawVert_t) {
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
        [0, 1], [1, 1], [1, 0], [1, -1], [0, -1], [-1, -1], [-1, 0], [-1, 1]
    ];

    wrapWidth = false;
    i = 0;
    while i < height {
        VectorSubtract(
            addr_of!((*ctrl.add((i as usize * MAX_GRID_SIZE) + 0).xyz[0])) as *const f32,
            addr_of!((*ctrl.add((i as usize * MAX_GRID_SIZE) + (width as usize - 1)).xyz[0])) as *const f32,
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
            addr_of!((*ctrl.add((0 * MAX_GRID_SIZE) + i as usize).xyz[0])) as *const f32,
            addr_of!((*ctrl.add(((height as usize - 1) * MAX_GRID_SIZE) + i as usize).xyz[0])) as *const f32,
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
            dv = ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize);
            VectorCopy(
                addr_of!((*dv).xyz[0]) as *const f32,
                base.as_mut_ptr(),
            );
            k = 0;
            while k < 8 {
                VectorClear(around[k as usize].as_mut_ptr());
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
                        addr_of!((*ctrl.add((y as usize * MAX_GRID_SIZE) + x as usize).xyz[0])) as *const f32,
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
                    around[((k + 1) & 7) as usize].as_ptr(),
                    around[k as usize].as_ptr(),
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
                // printf("bad normal\n");
                count = 1;
            }
            VectorNormalize2(
                sum.as_ptr(),
                addr_of_mut!((*ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize).normal[0])) as *mut f32,
            );
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
unsafe fn InvertCtrl(width: c_int, height: c_int, ctrl: *mut drawVert_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut temp: drawVert_t;

    i = 0;
    while i < height {
        j = 0;
        while j < width / 2 {
            temp = *(ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize));
            *(ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize)) =
                *(ctrl.add((i as usize * MAX_GRID_SIZE) + (width as usize - 1 - j as usize)));
            *(ctrl.add((i as usize * MAX_GRID_SIZE) + (width as usize - 1 - j as usize))) = temp;
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
unsafe fn InvertErrorTable(errorTable: *mut f32, width: c_int, height: c_int) {
    let mut i: c_int;
    let mut copy: [[f32; MAX_GRID_SIZE]; 2] = [[0.0; MAX_GRID_SIZE]; 2];

    core::ptr::copy_nonoverlapping(
        errorTable,
        copy.as_mut_ptr() as *mut f32,
        core::mem::size_of_val(&copy),
    );

    i = 0;
    while i < width {
        *(errorTable.add((1 * MAX_GRID_SIZE) + i as usize)) = copy[0][i as usize]; //[width-1-i];
        i += 1;
    }

    i = 0;
    while i < height {
        *(errorTable.add((0 * MAX_GRID_SIZE) + i as usize)) = copy[1][(height as usize - 1 - i as usize)];
        i += 1;
    }
}

/*
==================
PutPointsOnCurve
==================
*/
unsafe fn PutPointsOnCurve(ctrl: *mut drawVert_t, width: c_int, height: c_int) {
    let mut i: c_int;
    let mut j: c_int;
    let mut prev: drawVert_t;
    let mut next: drawVert_t;

    i = 0;
    while i < width {
        j = 1;
        while j < height {
            LerpDrawVert(
                ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize),
                ctrl.add(((j + 1) as usize * MAX_GRID_SIZE) + i as usize),
                addr_of_mut!(prev),
            );
            LerpDrawVert(
                ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize),
                ctrl.add(((j - 1) as usize * MAX_GRID_SIZE) + i as usize),
                addr_of_mut!(next),
            );
            LerpDrawVert(addr_of!(prev), addr_of!(next), ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize));
            j += 2;
        }
        i += 1;
    }

    j = 0;
    while j < height {
        i = 1;
        while i < width {
            LerpDrawVert(
                ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize),
                ctrl.add((j as usize * MAX_GRID_SIZE) + (i + 1) as usize),
                addr_of_mut!(prev),
            );
            LerpDrawVert(
                ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize),
                ctrl.add((j as usize * MAX_GRID_SIZE) + (i - 1) as usize),
                addr_of_mut!(next),
            );
            LerpDrawVert(addr_of!(prev), addr_of!(next), ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize));
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
    ctrl: *mut drawVert_t,
    errorTable: *mut f32,
) -> *mut srfGridMesh_t {
    let mut i: c_int;
    let mut j: c_int;
    let mut size: usize;
    let mut vert: *mut drawVert_t;
    let mut tmpVec: [f32; 3] = [0.0; 3];
    let mut grid: *mut srfGridMesh_t;

    // copy the results out to a grid
    size = ((width as usize * height as usize) - 1) * core::mem::size_of::<drawVert_t>() + core::mem::size_of::<srfGridMesh_t>();

    if PATCH_STITCHING {
        grid = Z_Malloc(size, TAG_GRIDMESH, 0 as c_int) as *mut srfGridMesh_t;
        Com_Memset(grid as *mut c_void, 0, size);

        (*grid).widthLodError = Z_Malloc(width as usize * 4, TAG_GRIDMESH, 0 as c_int) as *mut f32;
        Com_Memcpy(
            (*grid).widthLodError as *mut c_void,
            errorTable.add(0 * MAX_GRID_SIZE) as *const c_void,
            width as usize * 4,
        );

        (*grid).heightLodError = Z_Malloc(height as usize * 4, TAG_GRIDMESH, 0 as c_int) as *mut f32;
        Com_Memcpy(
            (*grid).heightLodError as *mut c_void,
            errorTable.add(1 * MAX_GRID_SIZE) as *const c_void,
            height as usize * 4,
        );
    } else {
        grid = Hunk_Alloc(size) as *mut srfGridMesh_t;
        Com_Memset(grid as *mut c_void, 0, size);

        (*grid).widthLodError = Hunk_Alloc(width as usize * 4) as *mut f32;
        Com_Memcpy(
            (*grid).widthLodError as *mut c_void,
            errorTable.add(0 * MAX_GRID_SIZE) as *const c_void,
            width as usize * 4,
        );

        (*grid).heightLodError = Hunk_Alloc(height as usize * 4) as *mut f32;
        Com_Memcpy(
            (*grid).heightLodError as *mut c_void,
            errorTable.add(1 * MAX_GRID_SIZE) as *const c_void,
            height as usize * 4,
        );
    }

    (*grid).width = width;
    (*grid).height = height;
    (*grid).surfaceType = SF_GRID;
    ClearBounds(
        addr_of_mut!((*grid).meshBounds[0][0]) as *mut f32,
        addr_of_mut!((*grid).meshBounds[1][0]) as *mut f32,
    );
    i = 0;
    while i < width {
        j = 0;
        while j < height {
            vert = addr_of_mut!((*grid).verts[0]) as *mut drawVert_t;
            vert = vert.add((j as usize * width as usize) + i as usize);
            *vert = *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize));
            AddPointToBounds(
                addr_of!((*vert).xyz[0]) as *const f32,
                addr_of_mut!((*grid).meshBounds[0][0]) as *mut f32,
                addr_of_mut!((*grid).meshBounds[1][0]) as *mut f32,
            );
            j += 1;
        }
        i += 1;
    }

    // compute local origin and bounds
    VectorAdd(
        addr_of!((*grid).meshBounds[0][0]) as *const f32,
        addr_of!((*grid).meshBounds[1][0]) as *const f32,
        addr_of_mut!((*grid).localOrigin[0]) as *mut f32,
    );
    VectorScale(
        addr_of!((*grid).localOrigin[0]) as *const f32,
        0.5f32,
        addr_of_mut!((*grid).localOrigin[0]) as *mut f32,
    );
    VectorSubtract(
        addr_of!((*grid).meshBounds[0][0]) as *const f32,
        addr_of!((*grid).localOrigin[0]) as *const f32,
        tmpVec.as_mut_ptr(),
    );
    (*grid).meshRadius = VectorLength(tmpVec.as_ptr());

    VectorCopy(
        addr_of!((*grid).localOrigin[0]) as *const f32,
        addr_of_mut!((*grid).lodOrigin[0]) as *mut f32,
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
    ctrl: *mut drawVert_t,
    errorTable: *mut f32,
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
    let mut grid: *mut srfGridMesh_t;
    let mut vert: *mut drawVert_t;
    let mut tmpVec: [f32; 3] = [0.0; 3];

    i = 0;
    while i < width {
        j = 0;
        while j < height {
            *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize)) =
                *(points.add((j as usize * width as usize) + i as usize));
            j += 1;
        }
        i += 1;
    }

    dir = 0;
    while dir < 2 {
        j = 0;
        while j < MAX_GRID_SIZE as c_int {
            *(errorTable.add((dir as usize * MAX_GRID_SIZE) + j as usize)) = 0.0;
            j += 1;
        }

        // horizontal subdivisions
        j = 0;
        while j + 2 < width {
            // check subdivided midpoints against control points
            maxLen = 0.0;
            i = 0;
            while i < height {
                let mut midxyz: [f32; 3] = [0.0; 3];
                let mut dir_vec: [f32; 3] = [0.0; 3];
                let mut projected: [f32; 3] = [0.0; 3];
                let mut d: f32;

                // calculate the point on the curve
                l = 0;
                while l < 3 {
                    midxyz[l as usize] = ((*ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize).xyz[l as usize]
                        + (*ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 1) as usize).xyz[l as usize]) * 2.0
                        + (*ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 2) as usize).xyz[l as usize]))
                        * 0.25;
                    l += 1;
                }

                // see how far off the line it is
                // using dist-from-line will not account for internal
                // texture warping, but it gives a lot less polygons than
                // dist-from-midpoint
                VectorSubtract(
                    midxyz.as_ptr(),
                    addr_of!((*ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize).xyz[0])) as *const f32,
                    midxyz.as_mut_ptr(),
                );
                VectorSubtract(
                    addr_of!((*ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 2) as usize).xyz[0])) as *const f32,
                    addr_of!((*ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize).xyz[0])) as *const f32,
                    dir_vec.as_mut_ptr(),
                );
                VectorNormalize(dir_vec.as_mut_ptr());

                d = DotProduct(midxyz.as_ptr(), dir_vec.as_ptr());
                VectorScale(dir_vec.as_ptr(), d, projected.as_mut_ptr());
                VectorSubtract(midxyz.as_ptr(), projected.as_ptr(), midxyz.as_mut_ptr());
                len = VectorLengthSquared(midxyz.as_ptr());

                if len > maxLen {
                    maxLen = len;
                }
                i += 1;
            }
            maxLen = maxLen.sqrt();

            // if all the points are on the lines, remove the entire columns
            if maxLen < 0.1 {
                *(errorTable.add((dir as usize * MAX_GRID_SIZE) + (j + 1) as usize)) = 999.0;
                j += 2;
                continue;
            }

            // see if we want to insert subdivided columns
            if width + 2 > MAX_GRID_SIZE as c_int {
                *(errorTable.add((dir as usize * MAX_GRID_SIZE) + (j + 1) as usize)) = 1.0 / maxLen;
                j += 2;
                continue; // can't subdivide any more
            }

            if maxLen <= (*r_subdivisions).value {
                *(errorTable.add((dir as usize * MAX_GRID_SIZE) + (j + 1) as usize)) = 1.0 / maxLen;
                j += 2;
                continue; // didn't need subdivision
            }

            *(errorTable.add((dir as usize * MAX_GRID_SIZE) + (j + 2) as usize)) = 1.0 / maxLen;

            // insert two columns and replace the peak
            let mut width_mut = width;
            width_mut += 2;
            i = 0;
            while i < height {
                LerpDrawVert(
                    ctrl.add((i as usize * MAX_GRID_SIZE) + j as usize),
                    ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 1) as usize),
                    addr_of_mut!(prev),
                );
                LerpDrawVert(
                    ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 1) as usize),
                    ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 2) as usize),
                    addr_of_mut!(next),
                );
                LerpDrawVert(addr_of!(prev), addr_of!(next), addr_of_mut!(mid));

                k = width_mut - 1;
                while k > j + 3 {
                    *(ctrl.add((i as usize * MAX_GRID_SIZE) + k as usize)) =
                        *(ctrl.add((i as usize * MAX_GRID_SIZE) + (k - 2) as usize));
                    k -= 1;
                }
                *(ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 1) as usize)) = prev;
                *(ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 2) as usize)) = mid;
                *(ctrl.add((i as usize * MAX_GRID_SIZE) + (j + 3) as usize)) = next;
                i += 1;
            }

            // back up and recheck this set again, it may need more subdivision
            j -= 2;
            j += 2;
        }

        Transpose(width, height, ctrl);
        t = width;
        width = height;
        height = t;
        dir += 1;
    }

    // put all the aproximating points on the curve
    PutPointsOnCurve(ctrl, width, height);

    // cull out any rows or columns that are colinear
    i = 1;
    while i < width - 1 {
        if *(errorTable.add((0 * MAX_GRID_SIZE) + i as usize)) != 999.0 {
            i += 1;
            continue;
        }
        j = i + 1;
        while j < width {
            k = 0;
            while k < height {
                *(ctrl.add((k as usize * MAX_GRID_SIZE) + (j - 1) as usize)) =
                    *(ctrl.add((k as usize * MAX_GRID_SIZE) + j as usize));
                k += 1;
            }
            *(errorTable.add((0 * MAX_GRID_SIZE) + (j - 1) as usize)) =
                *(errorTable.add((0 * MAX_GRID_SIZE) + j as usize));
            j += 1;
        }
        width -= 1;
    }

    i = 1;
    while i < height - 1 {
        if *(errorTable.add((1 * MAX_GRID_SIZE) + i as usize)) != 999.0 {
            i += 1;
            continue;
        }
        j = i + 1;
        while j < height {
            k = 0;
            while k < width {
                *(ctrl.add(((j - 1) as usize * MAX_GRID_SIZE) + k as usize)) =
                    *(ctrl.add((j as usize * MAX_GRID_SIZE) + k as usize));
                k += 1;
            }
            *(errorTable.add((1 * MAX_GRID_SIZE) + (j - 1) as usize)) =
                *(errorTable.add((1 * MAX_GRID_SIZE) + j as usize));
            j += 1;
        }
        height -= 1;
    }

    // flip for longest tristrips as an optimization
    // the results should be visually identical with or
    // without this step
    if height > width {
        Transpose(width, height, ctrl);
        InvertErrorTable(errorTable, width, height);
        t = width;
        width = height;
        height = t;
        InvertCtrl(width, height, ctrl);
    }

    // calculate normals
    MakeMeshNormals(width, height, ctrl);

    // copy the results out to a grid
    grid = Hunk_Alloc(
        ((width as usize * height as usize) - 1) * core::mem::size_of::<drawVert_t>()
            + core::mem::size_of::<srfGridMesh_t>()
            + (width as usize * 4)
            + (height as usize * 4),
    ) as *mut srfGridMesh_t;

    (*grid).widthLodError = (((grid as *const c_void as *const u8)
        .add(((width as usize * height as usize) - 1) * core::mem::size_of::<drawVert_t>()
            + core::mem::size_of::<srfGridMesh_t>())) as *mut f32);
    core::ptr::copy_nonoverlapping(
        errorTable.add(0 * MAX_GRID_SIZE),
        (*grid).widthLodError,
        width as usize,
    );

    (*grid).heightLodError =
        ((((*grid).widthLodError as *const u8).add(width as usize * 4)) as *mut f32);
    core::ptr::copy_nonoverlapping(
        errorTable.add(1 * MAX_GRID_SIZE),
        (*grid).heightLodError,
        height as usize,
    );

    (*grid).width = width;
    (*grid).height = height;
    (*grid).surfaceType = SF_GRID;
    ClearBounds(
        addr_of_mut!((*grid).meshBounds[0][0]) as *mut f32,
        addr_of_mut!((*grid).meshBounds[1][0]) as *mut f32,
    );
    i = 0;
    while i < width {
        j = 0;
        while j < height {
            vert = addr_of_mut!((*grid).verts[0]) as *mut drawVert_t;
            vert = vert.add((j as usize * width as usize) + i as usize);
            *vert = *(ctrl.add((j as usize * MAX_GRID_SIZE) + i as usize));
            AddPointToBounds(
                addr_of!((*vert).xyz[0]) as *const f32,
                addr_of_mut!((*grid).meshBounds[0][0]) as *mut f32,
                addr_of_mut!((*grid).meshBounds[1][0]) as *mut f32,
            );
            j += 1;
        }
        i += 1;
    }

    // compute local origin and bounds
    VectorAdd(
        addr_of!((*grid).meshBounds[0][0]) as *const f32,
        addr_of!((*grid).meshBounds[1][0]) as *const f32,
        addr_of_mut!((*grid).localOrigin[0]) as *mut f32,
    );
    VectorScale(
        addr_of!((*grid).localOrigin[0]) as *const f32,
        0.5f32,
        addr_of_mut!((*grid).localOrigin[0]) as *mut f32,
    );
    VectorSubtract(
        addr_of!((*grid).meshBounds[0][0]) as *const f32,
        addr_of!((*grid).localOrigin[0]) as *const f32,
        tmpVec.as_mut_ptr(),
    );
    (*grid).meshRadius = VectorLength(tmpVec.as_ptr());

    VectorCopy(
        addr_of!((*grid).localOrigin[0]) as *const f32,
        addr_of_mut!((*grid).lodOrigin[0]) as *mut f32,
    );
    (*grid).lodRadius = (*grid).meshRadius;

    grid
}
