// leave this as first line for PCH reasons...
//
#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::mem;

/*

This file does all of the processing necessary to turn a raw grid of points
read from the map file into a srfGridMesh_t ready for rendering.

The level of detail solution is direction independent, based only on subdivided
distance from the true curve.

Only a single entry point:

srfGridMesh_t *R_SubdividePatchToGrid( int width, int height,
								drawVert_t points[MAX_PATCH_SIZE*MAX_PATCH_SIZE] ) {

*/

// Type aliases
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;
pub type surfaceType_t = c_int;

const MAX_GRID_SIZE: usize = 65;
const MAX_PATCH_SIZE: usize = 32;
const MAXLIGHTMAPS: usize = 4;
const SF_GRID: c_int = 3;
const qtrue: c_int = 1;
const qfalse: c_int = 0;

// Type stubs for drawVert_t and srfGridMesh_t structures
#[repr(C)]
#[derive(Clone, Copy)]
pub struct drawVert_t {
	pub xyz: vec3_t,
	pub st: [f32; 2],
	pub lightmap: [[f32; 2]; MAXLIGHTMAPS],
	pub normal: vec3_t,
	pub color: [[u8; 4]; MAXLIGHTMAPS],
	// Padding and other fields not accessed in this file omitted
}

#[repr(C)]
pub struct srfGridMesh_t {
	pub surfaceType: surfaceType_t,
	pub dlightBits: c_int,
	pub meshBounds: [vec3_t; 2],
	pub localOrigin: vec3_t,
	pub meshRadius: f32,
	pub lodOrigin: vec3_t,
	pub lodRadius: f32,
	pub width: c_int,
	pub height: c_int,
	pub widthLodError: *mut f32,
	pub heightLodError: *mut f32,
	pub verts: [drawVert_t; 1],
}

#[repr(C)]
pub struct cvar_t {
	pub value: f32,
}

// External function declarations
extern "C" {
	fn VectorSubtract(veca: *const vec3_t, vecb: *const vec3_t, out: *mut vec3_t);
	fn VectorLengthSquared(v: *const vec3_t) -> f32;
	fn VectorLength(v: *const vec3_t) -> f32;
	fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
	fn VectorClear(v: *mut vec3_t);
	fn VectorNormalize2(src: *const vec3_t, dst: *mut vec3_t) -> f32;
	fn VectorNormalize(v: *mut vec3_t);
	fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, cross: *mut vec3_t);
	fn VectorAdd(v1: *const vec3_t, v2: *const vec3_t, out: *mut vec3_t);
	fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t);
	fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> f32;
	fn Hunk_Alloc(size: usize, clear: c_int) -> *mut c_void;
	fn ClearBounds(mins: *mut vec3_t, maxs: *mut vec3_t);
	fn AddPointToBounds(pt: *const vec3_t, mins: *mut vec3_t, maxs: *mut vec3_t);

	pub static r_subdivisions: *const cvar_t;
}

/*
============
LerpDrawVert
============
*/
unsafe fn LerpDrawVert(a: *const drawVert_t, b: *const drawVert_t, out: *mut drawVert_t) {
	let mut k: c_int = 0;
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

		(*out).color[k as usize][0] = (((*a).color[k as usize][0] as c_int + (*b).color[k as usize][0] as c_int) >> 1) as u8;
		(*out).color[k as usize][1] = (((*a).color[k as usize][1] as c_int + (*b).color[k as usize][1] as c_int) >> 1) as u8;
		(*out).color[k as usize][2] = (((*a).color[k as usize][2] as c_int + (*b).color[k as usize][2] as c_int) >> 1) as u8;
		(*out).color[k as usize][3] = (((*a).color[k as usize][3] as c_int + (*b).color[k as usize][3] as c_int) >> 1) as u8;
		k += 1;
	}
}

/*
============
Transpose
============
*/
unsafe fn Transpose(mut width: c_int, mut height: c_int, ctrl: &mut [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE]) {
	let mut i: c_int = 0;
	let mut j: c_int = 0;
	let mut temp: drawVert_t = mem::zeroed();

	if width > height {
		i = 0;
		while i < height {
			j = i + 1;
			while j < width {
				if j < height {
					// swap the value
					temp = ctrl[j as usize][i as usize];
					ctrl[j as usize][i as usize] = ctrl[i as usize][j as usize];
					ctrl[i as usize][j as usize] = temp;
				} else {
					// just copy
					ctrl[j as usize][i as usize] = ctrl[i as usize][j as usize];
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
					temp = ctrl[i as usize][j as usize];
					ctrl[i as usize][j as usize] = ctrl[j as usize][i as usize];
					ctrl[j as usize][i as usize] = temp;
				} else {
					// just copy
					ctrl[i as usize][j as usize] = ctrl[j as usize][i as usize];
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
unsafe fn MakeMeshNormals(width: c_int, height: c_int, ctrl: &mut [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE]) {
	let mut i: c_int = 0;
	let mut j: c_int = 0;
	let mut k: c_int = 0;
	let mut dist: c_int = 0;
	let mut normal: vec3_t = mem::zeroed();
	let mut sum: vec3_t = mem::zeroed();
	let mut count: c_int = 0;
	let mut base: vec3_t = mem::zeroed();
	let mut delta: vec3_t = mem::zeroed();
	let mut x: c_int = 0;
	let mut y: c_int = 0;
	let mut dv: *mut drawVert_t;
	let mut around: [vec3_t; 8] = mem::zeroed();
	let mut temp: vec3_t = mem::zeroed();
	let mut good: [qboolean; 8] = mem::zeroed();
	let mut wrapWidth: qboolean;
	let mut wrapHeight: qboolean;
	let mut len: f32 = 0.0;
	static neighbors: [[c_int; 2]; 8] = [
		[0,1], [1,1], [1,0], [1,-1], [0,-1], [-1,-1], [-1,0], [-1,1]
	];

	wrapWidth = qfalse;
	i = 0;
	while i < height {
		VectorSubtract(&ctrl[i as usize][0].xyz, &ctrl[i as usize][(width-1) as usize].xyz, &mut delta);
		len = VectorLength(&delta);
		if len > 1.0 {
			break;
		}
		i += 1;
	}
	if i == height {
		wrapWidth = qtrue;
	}

	wrapHeight = qfalse;
	i = 0;
	while i < width {
		VectorSubtract(&ctrl[0][i as usize].xyz, &ctrl[(height-1) as usize][i as usize].xyz, &mut delta);
		len = VectorLength(&delta);
		if len > 1.0 {
			break;
		}
		i += 1;
	}
	if i == width {
		wrapHeight = qtrue;
	}

	i = 0;
	while i < width {
		j = 0;
		while j < height {
			count = 0;
			dv = &mut ctrl[j as usize][i as usize];
			VectorCopy(&(*dv).xyz, &mut base);
			k = 0;
			while k < 8 {
				VectorClear(&mut around[k as usize]);
				good[k as usize] = qfalse;

				dist = 1;
				while dist <= 3 {
					x = i + neighbors[k as usize][0] * dist;
					y = j + neighbors[k as usize][1] * dist;
					if wrapWidth != qfalse {
						if x < 0 {
							x = width - 1 + x;
						} else if x >= width {
							x = 1 + x - width;
						}
					}
					if wrapHeight != qfalse {
						if y < 0 {
							y = height - 1 + y;
						} else if y >= height {
							y = 1 + y - height;
						}
					}

					if x < 0 || x >= width || y < 0 || y >= height {
						break;					// edge of patch
					}
					VectorSubtract(&ctrl[y as usize][x as usize].xyz, &base, &mut temp);
					if VectorNormalize2(&temp, &mut temp) == 0.0 {
						// degenerate edge, get more dist
					} else {
						good[k as usize] = qtrue;
						VectorCopy(&temp, &mut around[k as usize]);
						break;					// good edge
					}
					dist += 1;
				}
				k += 1;
			}

			VectorClear(&mut sum);
			k = 0;
			while k < 8 {
				if good[k as usize] == qfalse || good[((k+1)&7) as usize] == qfalse {
					// didn't get two points
				} else {
					CrossProduct(&around[((k+1)&7) as usize], &around[k as usize], &mut normal);
					if VectorNormalize2(&normal, &mut normal) == 0.0 {
						// continue
					} else {
						VectorAdd(&normal, &sum, &mut sum);
						count += 1;
					}
				}
				k += 1;
			}
			if count == 0 {
//printf("bad normal\n");
				count = 1;
			}
			VectorNormalize2(&sum, &mut (*dv).normal);
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
unsafe fn InvertCtrl(width: c_int, height: c_int, ctrl: &mut [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE]) {
	let mut i: c_int = 0;
	let mut j: c_int = 0;
	let mut temp: drawVert_t = mem::zeroed();

	i = 0;
	while i < height {
		j = 0;
		while j < width/2 {
			temp = ctrl[i as usize][j as usize];
			ctrl[i as usize][j as usize] = ctrl[i as usize][(width-1-j) as usize];
			ctrl[i as usize][(width-1-j) as usize] = temp;
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
unsafe fn InvertErrorTable(errorTable: &mut [[f32; MAX_GRID_SIZE]; 2], width: c_int, height: c_int) {
	let mut i: c_int = 0;
	let mut copy: [[f32; MAX_GRID_SIZE]; 2] = mem::zeroed();

	// memcpy( copy, errorTable, sizeof( copy ) );
	copy = *errorTable;

	i = 0;
	while i < width {
		errorTable[1][i as usize] = copy[0][i as usize];	//[width-1-i];
		i += 1;
	}

	i = 0;
	while i < height {
		errorTable[0][i as usize] = copy[1][(height-1-i) as usize];
		i += 1;
	}
}

/*
==================
PutPointsOnCurve
==================
*/
unsafe fn PutPointsOnCurve(ctrl: &mut [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE], width: c_int, height: c_int) {
	let mut i: c_int = 0;
	let mut j: c_int = 0;
	let mut prev: drawVert_t = mem::zeroed();
	let mut next: drawVert_t = mem::zeroed();

	i = 0;
	while i < width {
		j = 1;
		while j < height {
			LerpDrawVert(&ctrl[j as usize][i as usize], &ctrl[(j+1) as usize][i as usize], &mut prev);
			LerpDrawVert(&ctrl[j as usize][i as usize], &ctrl[(j-1) as usize][i as usize], &mut next);
			LerpDrawVert(&prev, &next, &mut ctrl[j as usize][i as usize]);
			j += 2;
		}
		i += 1;
	}


	j = 0;
	while j < height {
		i = 1;
		while i < width {
			LerpDrawVert(&ctrl[j as usize][i as usize], &ctrl[j as usize][(i+1) as usize], &mut prev);
			LerpDrawVert(&ctrl[j as usize][i as usize], &ctrl[j as usize][(i-1) as usize], &mut next);
			LerpDrawVert(&prev, &next, &mut ctrl[j as usize][i as usize]);
			i += 2;
		}
		j += 1;
	}
}

/*
=================
R_SubdividePatchToGrid

=================
*/
pub unsafe fn R_SubdividePatchToGrid(width: c_int, height: c_int,
								points: *const drawVert_t) -> *mut srfGridMesh_t {
	let mut i: c_int = 0;
	let mut j: c_int = 0;
	let mut k: c_int = 0;
	let mut l: c_int = 0;
	let mut prev: drawVert_t = mem::zeroed();
	let mut next: drawVert_t = mem::zeroed();
	let mut mid: drawVert_t = mem::zeroed();
	let mut len: f32 = 0.0;
	let mut maxLen: f32 = 0.0;
	let mut dir: c_int = 0;
	let mut t: c_int = 0;
	let mut ctrl: [[drawVert_t; MAX_GRID_SIZE]; MAX_GRID_SIZE] = mem::zeroed();
	let mut errorTable: [[f32; MAX_GRID_SIZE]; 2] = mem::zeroed();
	let mut grid: *mut srfGridMesh_t;
	let mut vert: *mut drawVert_t;
	let mut tmpVec: vec3_t = mem::zeroed();
	let mut width = width;
	let mut height = height;

	i = 0;
	while i < width {
		j = 0;
		while j < height {
			ctrl[j as usize][i as usize] = *points.add((j*width+i) as usize);
			j += 1;
		}
		i += 1;
	}

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
			maxLen = 0.0;
			i = 0;
			while i < height {
				let mut midxyz: vec3_t = mem::zeroed();
				let mut dir_vec: vec3_t = mem::zeroed();
				let mut projected: vec3_t = mem::zeroed();
				let mut d: f32 = 0.0;

				// calculate the point on the curve
				l = 0;
				while l < 3 {
					midxyz[l as usize] = (ctrl[i as usize][j as usize].xyz[l as usize] + ctrl[i as usize][(j+1) as usize].xyz[l as usize] * 2.0
							+ ctrl[i as usize][(j+2) as usize].xyz[l as usize] ) * 0.25;
					l += 1;
				}

				// see how far off the line it is
				// using dist-from-line will not account for internal
				// texture warping, but it gives a lot less polygons than
				// dist-from-midpoint
				VectorSubtract(&midxyz, &ctrl[i as usize][j as usize].xyz, &mut midxyz);
				VectorSubtract(&ctrl[i as usize][(j+2) as usize].xyz, &ctrl[i as usize][j as usize].xyz, &mut dir_vec);
				VectorNormalize(&mut dir_vec);

				d = DotProduct(&midxyz, &dir_vec);
				VectorScale(&dir_vec, d, &mut projected);
				VectorSubtract(&midxyz, &projected, &mut midxyz);
				len = VectorLength(&midxyz);

				if len > maxLen {
					maxLen = len;
				}
				i += 1;
			}

			// if all the points are on the lines, remove the entire columns
			if maxLen < 0.1 {
				errorTable[dir as usize][(j+1) as usize] = 999.0;
				j += 2;
				continue;
			}

			// see if we want to insert subdivided columns
			if width + 2 > MAX_GRID_SIZE as c_int {
				errorTable[dir as usize][(j+1) as usize] = 1.0/maxLen;
				j += 2;
				continue;	// can't subdivide any more
			}

			if maxLen <= (*r_subdivisions).value {
				errorTable[dir as usize][(j+1) as usize] = 1.0/maxLen;
				j += 2;
				continue;	// didn't need subdivision
			}

			errorTable[dir as usize][(j+2) as usize] = 1.0/maxLen;

			// insert two columns and replace the peak
			width += 2;
			i = 0;
			while i < height {
				LerpDrawVert(&ctrl[i as usize][j as usize], &ctrl[i as usize][(j+1) as usize], &mut prev);
				LerpDrawVert(&ctrl[i as usize][(j+1) as usize], &ctrl[i as usize][(j+2) as usize], &mut next);
				LerpDrawVert(&prev, &next, &mut mid);

				k = width - 1;
				while k > j + 3 {
					ctrl[i as usize][k as usize] = ctrl[i as usize][(k-2) as usize];
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

		Transpose(width, height, &mut ctrl);
		t = width;
		width = height;
		height = t;
		dir += 1;
	}


	// put all the aproximating points on the curve
	PutPointsOnCurve(&mut ctrl, width, height);

	// cull out any rows or columns that are colinear
	i = 1;
	while i < width-1 {
		if errorTable[0][i as usize] != 999.0 {
			i += 1;
			continue;
		}
		j = i+1;
		while j < width {
			k = 0;
			while k < height {
				ctrl[k as usize][(j-1) as usize] = ctrl[k as usize][j as usize];
				k += 1;
			}
			errorTable[0][(j-1) as usize] = errorTable[0][j as usize];
			j += 1;
		}
		width -= 1;
	}

	i = 1;
	while i < height-1 {
		if errorTable[1][i as usize] != 999.0 {
			i += 1;
			continue;
		}
		j = i+1;
		while j < height {
			k = 0;
			while k < width {
				ctrl[(j-1) as usize][k as usize] = ctrl[j as usize][k as usize];
				k += 1;
			}
			errorTable[1][(j-1) as usize] = errorTable[1][j as usize];
			j += 1;
		}
		height -= 1;
	}

// #if 1
	// flip for longest tristrips as an optimization
	// the results should be visually identical with or
	// without this step
	if height > width {
		Transpose(width, height, &mut ctrl);
		InvertErrorTable(&mut errorTable, width, height);
		t = width;
		width = height;
		height = t;
		InvertCtrl(width, height, &mut ctrl);
	}
// #endif

	// calculate normals
	MakeMeshNormals(width, height, &mut ctrl);

	// copy the results out to a grid
	grid = Hunk_Alloc(((width * height - 1) as usize * mem::size_of::<drawVert_t>() + mem::size_of::<srfGridMesh_t>()), qtrue) as *mut srfGridMesh_t;

	(*grid).widthLodError = ((grid as *mut u8).add((width * height - 1) as usize * mem::size_of::<drawVert_t>() + mem::size_of::<srfGridMesh_t>())) as *mut f32;
	core::ptr::copy_nonoverlapping(&errorTable[0][0] as *const f32, (*grid).widthLodError, width as usize);

	(*grid).heightLodError = (((*grid).widthLodError as *mut u8).add((width as usize) * mem::size_of::<f32>())) as *mut f32;
	core::ptr::copy_nonoverlapping(&errorTable[1][0] as *const f32, (*grid).heightLodError, height as usize);

	(*grid).width = width;
	(*grid).height = height;
	(*grid).surfaceType = SF_GRID;
	ClearBounds(&mut (*grid).meshBounds[0], &mut (*grid).meshBounds[1]);
	i = 0;
	while i < width {
		j = 0;
		while j < height {
			vert = &mut (*grid).verts[0] as *mut drawVert_t;
			vert = vert.add((j*width+i) as usize);
			*vert = ctrl[j as usize][i as usize];
			AddPointToBounds(&(*vert).xyz, &mut (*grid).meshBounds[0], &mut (*grid).meshBounds[1]);
			j += 1;
		}
		i += 1;
	}

	// compute local origin and bounds
	VectorAdd(&(*grid).meshBounds[0], &(*grid).meshBounds[1], &mut (*grid).localOrigin);
	VectorScale(&(*grid).localOrigin, 0.5, &mut (*grid).localOrigin);
	VectorSubtract(&(*grid).meshBounds[0], &(*grid).localOrigin, &mut tmpVec);
	(*grid).meshRadius = VectorLength(&tmpVec);

	VectorCopy(&(*grid).localOrigin, &mut (*grid).lodOrigin);
	(*grid).lodRadius = (*grid).meshRadius;

	return grid;
}
