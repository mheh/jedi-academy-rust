#![allow(non_snake_case)]

use core::ffi::c_int;

// #define	CULL_BBOX

/*

This file does not reference any globals, and has these entry points:

void CM_ClearLevelPatches( void );
struct patchCollide_s	*CM_GeneratePatchCollide( int width, int height, const vec3_t *points );
void CM_TraceThroughPatchCollide( traceWork_t *tw, const struct patchCollide_s *pc );
qboolean CM_PositionTestInPatchCollide( traceWork_t *tw, const struct patchCollide_s *pc );
void CM_DrawDebugSurface( void (*drawPoly)(int color, int numPoints, flaot *points) );


Issues for collision against curved surfaces:

Surface edges need to be handled differently than surface planes

Plane expansion causes raw surfaces to expand past expanded bounding box

Position test of a volume against a surface is tricky.

Position test of a point against a surface is not well defined, because the surface has no volume.


Tracing leading edge points instead of volumes?
Position test by tracing corner to corner? (8*7 traces -- ouch)

coplanar edges
triangulated patches
degenerate patches

  endcaps
  degenerate

WARNING: this may misbehave with meshes that have rows or columns that only
degenerate a few triangles.  Completely degenerate rows and columns are handled
properly.
*/

pub const MAX_FACETS: c_int = 1024;
pub const MAX_PATCH_PLANES: c_int = 2048;

pub type vec3_t = [f32; 3];

#[repr(C)]
pub struct patchPlane_t {
	pub plane: [f32; 4],
	pub signbits: c_int,		// signx + (signy<<1) + (signz<<2), used as lookup during collision
}

#[cfg(target_env = "msvc")]
#[repr(C, packed(1))]
pub struct facetLoad_t {
	pub surfacePlane: c_int,
	pub numBorders: c_int,		// 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
	pub borderPlanes: [i16; 4 + 6 + 16],
	pub borderInward: [u8; 4 + 6 + 16],
	pub borderNoAdjust: [u8; 4 + 6 + 16],
}

#[cfg(target_env = "msvc")]
#[repr(C)]
pub struct facet_t {
	pub surfacePlane: c_int,
	pub numBorders: c_int,		// 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
	pub data: *mut u8,

	// Facets are now two structures - a maximum sized version that's used
	// temporarily during load time, and smaller version that only allocates
	// as much memory as needed.  The load version is copied into the small
	// version after it's been assembled.
	// NOTE: Methods from C++ original omitted; access via data pointer with proper pointer arithmetic
}

#[cfg(not(target_env = "msvc"))]
#[repr(C)]
pub struct facet_t {
	pub surfacePlane: c_int,
	pub numBorders: c_int,		// 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
	pub borderPlanes: [c_int; 4 + 6 + 16],
	pub borderInward: [c_int; 4 + 6 + 16],
	pub borderNoAdjust: [c_int; 4 + 6 + 16],
}

#[repr(C)]
pub struct patchCollide_s {
	pub bounds: [vec3_t; 2],
	pub numPlanes: c_int,			// surface planes plus edge planes
	pub planes: *mut patchPlane_t,
	pub numFacets: c_int,
	pub facets: *mut facet_t,
}

pub const CM_MAX_GRID_SIZE: c_int = 129;

#[repr(C)]
pub struct cGrid_t {
	pub width: c_int,
	pub height: c_int,
	pub wrapWidth: c_int,
	pub wrapHeight: c_int,
	pub points: [[[f32; 3]; 129]; 129],	// [width][height]
}

pub const SUBDIVIDE_DISTANCE: c_int = 16;	//4	// never more than this units away from curve
pub const PLANE_TRI_EPSILON: f32 = 0.1;
pub const WRAP_POINT_EPSILON: f32 = 0.1;

#[cfg(target_env = "msvc")]
extern "C" {
	pub fn CM_GeneratePatchCollide(
		width: c_int,
		height: c_int,
		points: *mut vec3_t,
		facetbuf: *mut facetLoad_t,
		gridbuf: *mut c_int,
	) -> *mut patchCollide_s;
}

#[cfg(not(target_env = "msvc"))]
extern "C" {
	pub fn CM_GeneratePatchCollide(
		width: c_int,
		height: c_int,
		points: *mut vec3_t,
	) -> *mut patchCollide_s;
}
