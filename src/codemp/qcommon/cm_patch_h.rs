//! `cm_patch.h` — curved patch collision declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use crate::codemp::game::q_shared_h::{qboolean, vec3_t};
use core::ffi::{c_char, c_int, c_short, c_uchar};

//#define CULL_BBOX

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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct patchPlane_t {
    pub plane: [f32; 4],
    pub signbits: c_int, // signx + (signy<<1) + (signz<<2), used as lookup during collision
}

const _: () = assert!(core::mem::size_of::<patchPlane_t>() == 20);
const _: () = assert!(core::mem::align_of::<patchPlane_t>() == 4);

#[cfg(feature = "xbox")]
pub const CM_MAX_GRID_SIZE: usize = 129;
#[cfg(not(feature = "xbox"))]
pub const MAX_GRID_SIZE: usize = 129;

#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct facetLoad_t {
    pub surfacePlane: c_int,
    pub numBorders: c_int, // 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
    pub borderPlanes: [c_short; 4 + 6 + 16],
    pub borderInward: [c_uchar; 4 + 6 + 16],
    pub borderNoAdjust: [c_uchar; 4 + 6 + 16],
}

#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct facet_t {
    pub surfacePlane: c_int,
    pub numBorders: c_int, // 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
    pub data: *mut c_char,
}

#[cfg(feature = "xbox")]
impl facet_t {
    #[inline]
    pub unsafe fn GetBorderPlanes(&mut self) -> *mut c_short {
        self.data as *mut c_short
    }

    #[inline]
    pub unsafe fn GetBorderInward(&mut self) -> *mut c_char {
        self.data.add((self.numBorders * 2) as usize)
    }

    #[inline]
    pub unsafe fn GetBorderNoAdjust(&mut self) -> *mut c_char {
        self.data
            .add((self.numBorders * 2) as usize)
            .add(self.numBorders as usize)
    }

    #[inline]
    pub unsafe fn GetBorderPlanes_const(&self) -> *const c_short {
        self.data as *const c_short
    }

    #[inline]
    pub unsafe fn GetBorderInward_const(&self) -> *const c_char {
        self.data.add((self.numBorders * 2) as usize)
    }

    #[inline]
    pub unsafe fn GetBorderNoAdjust_const(&self) -> *const c_char {
        self.data
            .add((self.numBorders * 2) as usize)
            .add(self.numBorders as usize)
    }
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct facet_t {
    pub surfacePlane: c_int,
    pub numBorders: c_int, // 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
    pub borderPlanes: [c_int; 4 + 6 + 16],
    pub borderInward: [c_int; 4 + 6 + 16],
    pub borderNoAdjust: [qboolean; 4 + 6 + 16],
}

#[cfg(not(feature = "xbox"))]
const _: () = assert!(core::mem::size_of::<facet_t>() == 320);
#[cfg(not(feature = "xbox"))]
const _: () = assert!(core::mem::align_of::<facet_t>() == 4);

#[repr(C)]
pub struct patchCollide_s {
    pub bounds: [vec3_t; 2],
    pub numPlanes: c_int, // surface planes plus edge planes
    pub planes: *mut patchPlane_t,
    pub numFacets: c_int,
    pub facets: *mut facet_t,
}

pub type patchCollide_t = patchCollide_s;

#[cfg(feature = "xbox")]
#[repr(C)]
pub struct cGrid_t {
    pub width: c_int,
    pub height: c_int,
    pub wrapWidth: qboolean,
    pub wrapHeight: qboolean,
    pub points: [[vec3_t; CM_MAX_GRID_SIZE]; CM_MAX_GRID_SIZE], // [width][height]
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct cGrid_t {
    pub width: c_int,
    pub height: c_int,
    pub wrapWidth: qboolean,
    pub wrapHeight: qboolean,
    pub points: [[vec3_t; MAX_GRID_SIZE]; MAX_GRID_SIZE], // [width][height]
}

#[cfg(not(feature = "xbox"))]
const _: () = assert!(core::mem::size_of::<cGrid_t>() == 199708);
#[cfg(not(feature = "xbox"))]
const _: () = assert!(core::mem::align_of::<cGrid_t>() == 4);

pub const SUBDIVIDE_DISTANCE: c_int = 16; //4	// never more than this units away from curve
pub const PLANE_TRI_EPSILON: f32 = 0.1;
pub const WRAP_POINT_EPSILON: f32 = 0.1;

#[cfg(feature = "xbox")]
unsafe extern "C" {
    pub fn CM_GeneratePatchCollide(
        width: c_int,
        height: c_int,
        points: *mut vec3_t,
        facetbuf: *mut facetLoad_t,
        gridbuf: *mut c_int,
    ) -> *mut patchCollide_s;
}

#[cfg(not(feature = "xbox"))]
unsafe extern "C" {
    pub fn CM_GeneratePatchCollide(width: c_int, height: c_int, points: *mut vec3_t)
        -> *mut patchCollide_s;
}
