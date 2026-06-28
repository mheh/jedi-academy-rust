//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
//
// #include "cm_local.h"
// #include "cm_patch.h"
// #include "cm_landscape.h"
// #include "../qcommon/GenericParser2.h"
// #include "cm_randomterrain.h"

use core::ffi::{c_char, c_int, c_void};
use std::ptr::{addr_of, addr_of_mut};

// Forward declarations for external engine functions
extern "C" {
    fn R_LoadDataImage(name: *const c_char, pic: *mut *mut u8, width: *mut c_int, height: *mut c_int);
    fn R_InvertImage(data: *mut u8, width: c_int, height: c_int, depth: c_int);
    fn R_Resample(
        source: *mut u8,
        swidth: c_int,
        sheight: c_int,
        dest: *mut u8,
        dwidth: c_int,
        dheight: c_int,
        components: c_int,
    );
    fn Z_Malloc(size: usize, tag: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Info_ValueForKey(info: *const c_char, key: *const c_char) -> *const c_char;
    fn Com_ParseTextFile(filename: *const c_char, parse: *mut c_void) -> bool;
    fn Com_ParseTextFileDestroy(parse: c_void);
    fn strlen(s: *const c_char) -> usize;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn atol(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f32;
    fn strtoul(s: *const c_char, endptr: *mut *mut c_char, radix: c_int) -> u32;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn fabs(x: f32) -> f32;
    fn sqrtf(x: f32) -> f32;
    fn sqrt(x: f64) -> f64;
    fn floorf(x: f32) -> f32;
    fn ceilf(x: f32) -> f32;
    fn sinf(x: f32) -> f32;
    fn fabsf(x: f32) -> f32;

    // Game-specific externs
    fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    fn VectorSet(v: *mut [f32; 3], x: f32, y: f32, z: f32);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn CrossProduct(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn DotProduct(a: *const [f32; 3], b: *const [f32; 3]) -> f32;
    fn VectorLength(v: *const [f32; 3]) -> f32;
    fn VectorDec(v: *mut [f32; 3]);
    fn VectorInc(v: *mut [f32; 3]);
    fn VectorInverseScaleVector(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorScaleVectorAdd(a: *const [f32; 3], b: *const [f32; 3], c: *const [f32; 3], out: *mut [f32; 3]);
    fn PlaneTypeForNormal(normal: *const [f32; 3]) -> c_int;
    fn SetPlaneSignbits(plane: *mut cplane_t);
    fn Round(x: f32) -> c_int;
    fn Com_Clamp(min: c_int, max: c_int, value: c_int) -> c_int;
    fn Distance(a: *const [f32; 3], b: *const [f32; 3]) -> f32;
    fn CM_CalcExtents(start: *const [f32; 3], end: *const [f32; 3], tw: *mut traceWork_s, bounds: *mut [[f32; 3]; 2]);
    fn CM_HandlePatchCollision(tw: *mut traceWork_s, trace: *mut trace_t, mins: *const [f32; 3], maxs: *const [f32; 3], patch: *mut CCMPatch, checkcount: c_int);

    // Stub functions that need to be defined elsewhere
    fn CM_GetShaderInfo(name: *const c_char) -> *mut CCMShader;

    // Forward declare classes as extern types
    static mut com_newtrace: *mut cvar_t;
}

// Constants from defines
const MAX_QPATH: usize = 256;
const HEIGHT_RESOLUTION: usize = 256;
const SURFACE_CLIP_EPSILON: f32 = 0.125f32;
const MAX_WORLD_COORD: f32 = 131072.0f32;
const MIN_WORLD_COORD: f32 = -131072.0f32;
const TAG_CM_TERRAIN: c_int = 1;
const TAG_CM_TERRAIN_TEMP: c_int = 2;
const M_PI: f32 = 3.141592653589793f32;
const ERR_FATAL: c_int = 3;

#[cfg(feature = "_SMOOTH_TERXEL_BRUSH")]
const BRUSH_SIDES_PER_TERXEL: usize = 8;

#[cfg(not(feature = "_SMOOTH_TERXEL_BRUSH"))]
const BRUSH_SIDES_PER_TERXEL: usize = 5;

// Type aliases for C types
pub type vec3_t = [f32; 3];
pub type ivec3_t = [c_int; 3];
pub type vec3pair_t = [[f32; 3]; 2];

// Macro for LERP
const fn LERP(t: f32, a: f32, b: f32) -> f32 {
    (b - a) * t + a
}

// Forward declarations for C++ classes translated to Rust
#[repr(C)]
pub struct cbrush_t {
    // Placeholder - actual definition in cm_local.rs
}

#[repr(C)]
pub struct cbrushside_t {
    // Placeholder - actual definition in cm_local.rs
}

#[repr(C)]
pub struct cplane_t {
    normal: [f32; 3],
    dist: f32,
    pub plane_type: u8,
    signbits: u8,
    pad: [u8; 2],
}

#[repr(C)]
pub struct trace_t {
    allsolid: bool,
    startsolid: bool,
    pub fraction: f32,
    endpos: [f32; 3],
    plane: cplane_t,
    surfaceFlags: c_int,
    contents: c_int,
    entityNum: c_int,
    pub ent: *mut c_void,
}

#[repr(C)]
pub struct traceWork_s {
    pub size: [[f32; 3]; 2],
    // Placeholder for rest of structure
}

#[repr(C)]
pub struct CCMShader {
    contentFlags: c_int,
    surfaceFlags: c_int,
}

#[repr(C)]
pub struct CCMHeightDetails {
    // Placeholder - needs to be defined from cm_landscape.h
}

#[repr(C)]
pub struct CArea {
    // Placeholder - needs to be defined
}

#[repr(C)]
pub struct CGenericParser2 {
    // Placeholder
}

#[repr(C)]
pub struct CGPGroup {
    // Placeholder
}

#[repr(C)]
pub struct CRandomTerrain {
    // Placeholder
}

#[repr(C)]
pub struct cvar_t {
    // Placeholder
}

#[repr(C)]
pub struct CCMPatch {
    owner: *mut CCMLandScape,
    mWorldCoords: [f32; 3],
    mHx: c_int,
    mHy: c_int,
    mHeightMap: *mut u8,
    mBounds: [[f32; 3]; 2],
    mCornerHeights: [u8; 4],
    mSurfaceFlags: c_int,
    mContentFlags: c_int,
    mNumBrushes: c_int,
    mPatchBrushData: *mut cbrush_t,
}

impl CCMPatch {
    // Initialise a plane from 3 coords
    unsafe fn InitPlane(&mut self, side: *mut cbrushside_t, plane: *mut cplane_t, p0: *const [f32; 3], p1: *const [f32; 3], p2: *const [f32; 3]) {
        let mut dx: [f32; 3] = [0.0; 3];
        let mut dy: [f32; 3] = [0.0; 3];

        VectorSubtract(p1, p0, addr_of_mut!(dx));
        VectorSubtract(p2, p0, addr_of_mut!(dy));
        CrossProduct(addr_of!(dx), addr_of!(dy), addr_of_mut!((*plane).normal));
        VectorNormalize(addr_of_mut!((*plane).normal));

        (*plane).dist = DotProduct(p0, addr_of!((*plane).normal));
        (*plane).plane_type = PlaneTypeForNormal(addr_of!((*plane).normal)) as u8;
        SetPlaneSignbits(plane);

        #[cfg(target_os = "xbox")]
        {
            // cmg.planes[side->planeNum.GetValue()] = *plane;
            // Not implemented on non-Xbox
        }

        #[cfg(not(target_os = "xbox"))]
        {
            // side->plane = plane;
            // Assignment handled in C layer
        }
    }

    // Create the planes required for collision detection
    // 2 brushes per terxel - each brush has 5 sides and 5 planes
    unsafe fn GetAdjacentBrushY(&self, x: c_int, y: c_int) -> *mut cbrush_t {
        let yo1 = y % (*self.owner).GetTerxels();
        let yo2 = (y - 1) % (*self.owner).GetTerxels();
        let xo = x % (*self.owner).GetTerxels();

        let patch = if yo2 > yo1 {
            // Different patch
            (*self.owner).GetPatch(x / (*self.owner).GetTerxels(), (y - 1) / (*self.owner).GetTerxels())
        } else {
            self as *const _ as *mut _
        };

        let mut brush = (*patch).mPatchBrushData;
        brush = (brush as *mut u8).offset(((yo2 * (*self.owner).GetTerxels() + xo) * 2) as isize * std::mem::size_of::<cbrush_t>() as isize) as *mut cbrush_t;
        brush = (brush as *mut u8).offset(std::mem::size_of::<cbrush_t>() as isize) as *mut cbrush_t;

        brush
    }

    unsafe fn GetAdjacentBrushX(&self, x: c_int, y: c_int) -> *mut cbrush_t {
        let xo1 = x % (*self.owner).GetTerxels();
        let xo2 = (x - 1) % (*self.owner).GetTerxels();
        let yo = y % (*self.owner).GetTerxels();

        let patch = if xo2 > xo1 {
            // Different patch
            (*self.owner).GetPatch((x - 1) / (*self.owner).GetTerxels(), y / (*self.owner).GetTerxels())
        } else {
            self as *const _ as *mut _
        };

        let mut brush = (*patch).mPatchBrushData;
        brush = (brush as *mut u8).offset(((yo * (*self.owner).GetTerxels() + xo2) * 2) as isize * std::mem::size_of::<cbrush_t>() as isize) as *mut cbrush_t;

        if !((x + y) & 1 != 0) {
            brush = (brush as *mut u8).offset(std::mem::size_of::<cbrush_t>() as isize) as *mut cbrush_t;
        }

        brush
    }

    unsafe fn CreatePatchPlaneData(&mut self) {
        #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        {
            let realWidth: c_int;
            let mut x: c_int;
            let mut y: c_int;
            let mut i: c_int;
            let mut j: c_int;

            let brush: *mut cbrush_t;
            let side: *mut cbrushside_t;
            let plane: *mut cplane_t;
            let coords: *mut [f32; 3];
            let mut localCoords: [[f32; 3]; 8] = [[0.0; 3]; 8];

            self.mNumBrushes = (*self.owner).GetTerxels() * (*self.owner).GetTerxels() * 2;
            realWidth = (*self.owner).GetRealWidth();
            coords = (*self.owner).GetCoords();

            brush = self.mPatchBrushData;
            side = (self.mPatchBrushData as *mut u8).offset(self.mNumBrushes as isize * std::mem::size_of::<cbrush_t>() as isize) as *mut cbrushside_t;
            plane = (side as *mut u8).offset((self.mNumBrushes as usize * BRUSH_SIDES_PER_TERXEL * 2) as isize * std::mem::size_of::<cbrushside_t>() as isize) as *mut cplane_t;

            y = self.mHy;
            while y < self.mHy + (*self.owner).GetTerxels() {
                x = self.mHx;
                while x < self.mHx + (*self.owner).GetTerxels() {
                    let mut offsets: [c_int; 4] = [0; 4];

                    if (x + y) & 1 != 0 {
                        offsets[0] = (y * realWidth) + x;
                        offsets[1] = (y * realWidth) + x + 1;
                        offsets[2] = ((y + 1) * realWidth) + x;
                        offsets[3] = ((y + 1) * realWidth) + x + 1;
                    } else {
                        offsets[2] = (y * realWidth) + x;
                        offsets[0] = (y * realWidth) + x + 1;
                        offsets[3] = ((y + 1) * realWidth) + x;
                        offsets[1] = ((y + 1) * realWidth) + x + 1;
                    }

                    i = 0;
                    while i < 4 {
                        VectorCopy(
                            addr_of!((*coords.offset(offsets[i as usize] as isize))),
                            addr_of_mut!(localCoords[i as usize]),
                        );
                        VectorCopy(
                            addr_of!((*coords.offset(offsets[i as usize] as isize))),
                            addr_of_mut!(localCoords[(i + 4) as usize]),
                        );
                        localCoords[(i + 4) as usize][2] = (*self.owner).GetMins()[2];
                        i += 1;
                    }

                    // Set the bounds of the terxel
                    VectorSet(addr_of_mut!((*brush).bounds[0]), MAX_WORLD_COORD, MAX_WORLD_COORD, MAX_WORLD_COORD);
                    VectorSet(addr_of_mut!((*brush).bounds[1]), MIN_WORLD_COORD, MIN_WORLD_COORD, MIN_WORLD_COORD);

                    i = 0;
                    while i < 8 {
                        j = 0;
                        while j < 3 {
                            if localCoords[i as usize][j as usize] < (*brush).bounds[0][j as usize] {
                                (*brush).bounds[0][j as usize] = localCoords[i as usize][j as usize];
                            }
                            if localCoords[i as usize][j as usize] > (*brush).bounds[1][j as usize] {
                                (*brush).bounds[1][j as usize] = localCoords[i as usize][j as usize];
                            }
                            j += 1;
                        }
                        i += 1;
                    }

                    VectorDec(addr_of_mut!((*brush).bounds[0]));
                    VectorInc(addr_of_mut!((*brush).bounds[1]));
                    VectorCopy(addr_of!((*brush).bounds[0]), addr_of_mut!((*brush.offset(1)).bounds[0]));
                    VectorCopy(addr_of!((*brush).bounds[1]), addr_of_mut!((*brush.offset(1)).bounds[1]));

                    (*brush).contents = self.mContentFlags;
                    (*brush.offset(1)).contents = self.mContentFlags;

                    #[cfg(not(feature = "_SMOOTH_TERXEL_BRUSH"))]
                    {
                        // Set up sides of the brushes
                        (*brush).numsides = 5;
                        (*brush).sides = side;
                        (*brush.offset(1)).numsides = 5;
                        (*brush.offset(1)).sides = side.offset(5);

                        i = 0;
                        while i < 8 {
                            localCoords[i as usize][0] = (localCoords[i as usize][0] as c_int) as f32;
                            localCoords[i as usize][1] = (localCoords[i as usize][1] as c_int) as f32;
                            localCoords[i as usize][2] = (localCoords[i as usize][2] as c_int) as f32;
                            i += 1;
                        }

                        // Create the planes of the 2 triangles that make up the tops of the brushes
                        self.InitPlane(side, plane, addr_of!(localCoords[0]), addr_of!(localCoords[1]), addr_of!(localCoords[2]));
                        self.InitPlane(side.offset(5), plane.offset(5), addr_of!(localCoords[3]), addr_of!(localCoords[2]), addr_of!(localCoords[1]));

                        // Create the bottom face of the brushes
                        self.InitPlane(side.offset(1), plane.offset(1), addr_of!(localCoords[6]), addr_of!(localCoords[5]), addr_of!(localCoords[4]));
                        self.InitPlane(side.offset(6), plane.offset(6), addr_of!(localCoords[5]), addr_of!(localCoords[6]), addr_of!(localCoords[7]));

                        // Create the 3 vertical faces
                        self.InitPlane(side.offset(2), plane.offset(2), addr_of!(localCoords[0]), addr_of!(localCoords[2]), addr_of!(localCoords[4]));
                        self.InitPlane(side.offset(7), plane.offset(7), addr_of!(localCoords[3]), addr_of!(localCoords[1]), addr_of!(localCoords[7]));

                        self.InitPlane(side.offset(3), plane.offset(3), addr_of!(localCoords[0]), addr_of!(localCoords[4]), addr_of!(localCoords[1]));
                        self.InitPlane(side.offset(8), plane.offset(8), addr_of!(localCoords[3]), addr_of!(localCoords[7]), addr_of!(localCoords[2]));

                        self.InitPlane(side.offset(4), plane.offset(4), addr_of!(localCoords[2]), addr_of!(localCoords[1]), addr_of!(localCoords[6]));
                        self.InitPlane(side.offset(9), plane.offset(9), addr_of!(localCoords[5]), addr_of!(localCoords[1]), addr_of!(localCoords[6]));

                        // Increment to next terxel
                        brush = brush.offset(2);
                        side = side.offset(10);
                        plane = plane.offset(10);
                    }

                    #[cfg(feature = "_SMOOTH_TERXEL_BRUSH")]
                    {
                        // Set up sides of the brushes
                        (*brush).numsides = 5;
                        (*brush).sides = side;
                        (*brush.offset(1)).numsides = 5;
                        (*brush.offset(1)).sides = side.offset(8);

                        // Create the planes of the 2 triangles that make up the tops of the brushes
                        self.InitPlane(side, plane, addr_of!(localCoords[0]), addr_of!(localCoords[1]), addr_of!(localCoords[2]));
                        self.InitPlane(side.offset(8), plane.offset(8), addr_of!(localCoords[3]), addr_of!(localCoords[2]), addr_of!(localCoords[1]));

                        // Create the bottom face of the brushes
                        self.InitPlane(side.offset(1), plane.offset(1), addr_of!(localCoords[4]), addr_of!(localCoords[6]), addr_of!(localCoords[5]));
                        self.InitPlane(side.offset(9), plane.offset(9), addr_of!(localCoords[7]), addr_of!(localCoords[5]), addr_of!(localCoords[6]));

                        // Create the 3 vertical faces
                        self.InitPlane(side.offset(2), plane.offset(2), addr_of!(localCoords[0]), addr_of!(localCoords[2]), addr_of!(localCoords[4]));
                        self.InitPlane(side.offset(10), plane.offset(10), addr_of!(localCoords[3]), addr_of!(localCoords[1]), addr_of!(localCoords[7]));

                        self.InitPlane(side.offset(3), plane.offset(3), addr_of!(localCoords[0]), addr_of!(localCoords[4]), addr_of!(localCoords[1]));
                        self.InitPlane(side.offset(11), plane.offset(11), addr_of!(localCoords[3]), addr_of!(localCoords[7]), addr_of!(localCoords[2]));

                        self.InitPlane(side.offset(4), plane.offset(4), addr_of!(localCoords[2]), addr_of!(localCoords[1]), addr_of!(localCoords[6]));
                        self.InitPlane(side.offset(12), plane.offset(12), addr_of!(localCoords[5]), addr_of!(localCoords[1]), addr_of!(localCoords[6]));

                        let V: f32 = DotProduct(addr_of!((*plane.offset(8)).normal), addr_of!(localCoords[0])) - (*plane.offset(8)).dist;

                        if V < 0.0 {
                            self.InitPlane(
                                addr_of_mut!((*brush).sides.offset((*brush).numsides as isize)),
                                addr_of_mut!(*plane.offset((*brush).numsides as isize)),
                                addr_of!(localCoords[3]),
                                addr_of!(localCoords[2]),
                                addr_of!(localCoords[1]),
                            );
                            (*brush).numsides += 1;

                            self.InitPlane(
                                addr_of_mut!((*brush.offset(1)).sides.offset((*brush.offset(1)).numsides as isize)),
                                addr_of_mut!(*plane.offset(8 + (*brush.offset(1)).numsides as isize)),
                                addr_of!(localCoords[0]),
                                addr_of!(localCoords[1]),
                                addr_of!(localCoords[2]),
                            );
                            (*brush.offset(1)).numsides += 1;
                        }

                        // Determine if we need to smooth the brush transition from the brush above us
                        if y > 0 && y < (*self.owner).GetPatchHeight() - 1 {
                            let abovebrush = self.GetAdjacentBrushY(x, y);

                            #[cfg(target_os = "xbox")]
                            let aboveplane = std::ptr::null_mut::<cplane_t>();  // Stub
                            #[cfg(not(target_os = "xbox"))]
                            let aboveplane = (*abovebrush).sides;  // Stub - actual definition would need access to plane pointer

                            let V = DotProduct(
                                addr_of!((*aboveplane).normal),
                                if (y + x) & 1 != 0 { addr_of!(localCoords[2]) } else { addr_of!(localCoords[1]) },
                            ) - (*aboveplane).dist;

                            if V < 0.0 {
                                memcpy(
                                    addr_of_mut!((*brush).sides.offset((*brush).numsides as isize)) as *mut c_void,
                                    addr_of!((*abovebrush).sides) as *const c_void,
                                    std::mem::size_of::<cbrushside_t>(),
                                );
                                (*brush).numsides += 1;

                                memcpy(
                                    addr_of_mut!((*abovebrush).sides.offset((*abovebrush).numsides as isize)) as *mut c_void,
                                    addr_of!(*side) as *const c_void,
                                    std::mem::size_of::<cbrushside_t>(),
                                );
                                (*abovebrush).numsides += 1;
                            }
                        }

                        // Determine if we need to smooth the brush transition from the brush to the left of us
                        if x > 0 && x < (*self.owner).GetPatchWidth() - 1 {
                            let abovebrush = self.GetAdjacentBrushX(x, y);

                            #[cfg(target_os = "xbox")]
                            let aboveplane = std::ptr::null_mut::<cplane_t>();  // Stub
                            #[cfg(not(target_os = "xbox"))]
                            let aboveplane = (*abovebrush).sides;  // Stub

                            let V = DotProduct(addr_of!((*aboveplane).normal), addr_of!(localCoords[1])) - (*aboveplane).dist;

                            if V < 0.0 {
                                if (x + y) & 1 != 0 {
                                    memcpy(
                                        addr_of_mut!((*brush).sides.offset((*brush).numsides as isize)) as *mut c_void,
                                        addr_of!((*abovebrush).sides) as *const c_void,
                                        std::mem::size_of::<cbrushside_t>(),
                                    );
                                    (*brush).numsides += 1;

                                    memcpy(
                                        addr_of_mut!((*abovebrush).sides.offset((*abovebrush).numsides as isize)) as *mut c_void,
                                        addr_of!(*side) as *const c_void,
                                        std::mem::size_of::<cbrushside_t>(),
                                    );
                                    (*abovebrush).numsides += 1;
                                } else {
                                    memcpy(
                                        addr_of_mut!((*brush.offset(1)).sides.offset((*brush.offset(1)).numsides as isize)) as *mut c_void,
                                        addr_of!((*abovebrush).sides) as *const c_void,
                                        std::mem::size_of::<cbrushside_t>(),
                                    );
                                    (*brush.offset(1)).numsides += 1;

                                    memcpy(
                                        addr_of_mut!((*abovebrush).sides.offset((*abovebrush).numsides as isize)) as *mut c_void,
                                        addr_of!(*side.offset(8)) as *const c_void,
                                        std::mem::size_of::<cbrushside_t>(),
                                    );
                                    (*abovebrush).numsides += 1;
                                }
                            }
                        }

                        // Increment to next terxel
                        brush = brush.offset(2);
                        side = side.offset(16);
                        plane = plane.offset(16);
                    }

                    x += 1;
                }
                y += 1;
            }
        }
    }

    unsafe fn Init(&mut self, ls: *mut CCMLandScape, heightX: c_int, heightY: c_int, world: *const [f32; 3], hMap: *mut u8, patchBrushData: *mut cbrush_t) {
        #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        {
            let mut min: c_int = 256;
            let mut max: c_int = -1;
            let mut x: c_int;
            let mut y: c_int;
            let mut height: c_int;

            // Set owning landscape
            self.owner = ls;

            // Store the base of the top left corner
            VectorCopy(world, addr_of_mut!(self.mWorldCoords));

            // Store pointer to first byte of the height data for this patch.
            self.mHx = heightX;
            self.mHy = heightY;
            self.mHeightMap = (hMap as *mut u8).offset(((heightY * (*ls).GetRealWidth()) + heightX) as isize);

            // Calculate the bounds for culling
            // Use the dimensions 1 terxel outside the patch to allow for sloping of edge terxels
            y = heightY - 1;
            while y < heightY + (*ls).GetTerxels() + 1 {
                if y >= 0 {
                    x = heightX - 1;
                    while x < heightX + (*ls).GetTerxels() + 1 {
                        if x >= 0 {
                            height = *(hMap.offset((y * (*ls).GetRealWidth() + x) as isize)) as c_int;

                            if height > max {
                                max = height;
                            }
                            if height < min {
                                min = height;
                            }
                        }
                        x += 1;
                    }
                }
                y += 1;
            }

            // Mins
            self.mBounds[0][0] = (*world)[0];
            self.mBounds[0][1] = (*world)[1];
            self.mBounds[0][2] = (*world)[2] + ((min as f32) * (*ls).GetTerxelSize()[2]);

            // Maxs
            self.mBounds[1][0] = (*world)[0] + (*ls).GetPatchSize()[0];
            self.mBounds[1][1] = (*world)[1] + (*ls).GetPatchSize()[1];
            self.mBounds[1][2] = (*world)[2] + ((max as f32) * (*ls).GetTerxelSize()[2]);

            // Corner heights
            self.mCornerHeights[0] = *(self.mHeightMap) as u8;
            self.mCornerHeights[1] = *(self.mHeightMap.offset((*ls).GetTerxels() as isize)) as u8;
            self.mCornerHeights[2] = *(self.mHeightMap.offset(((*ls).GetTerxels() * (*ls).GetRealWidth()) as isize)) as u8;
            self.mCornerHeights[3] = *(self.mHeightMap.offset(((*ls).GetTerxels() * (*ls).GetRealWidth() + (*ls).GetTerxels()) as isize)) as u8;

            // Set the surfaceFlags using average height (may want a more complex algo here)
            self.mSurfaceFlags = (*ls).GetSurfaceFlags((min + max) >> 1);
            self.mContentFlags = (*ls).GetContentFlags((min + max) >> 1);

            // Set base of brush data from big array
            self.mPatchBrushData = patchBrushData;
            self.CreatePatchPlaneData();
        }
    }
}

impl Drop for CCMPatch {
    fn drop(&mut self) {
        // Empty destructor in original
    }
}

// Landscape class
#[repr(C)]
pub struct CCMLandScape {
    mRefCount: c_int,
    mWidth: c_int,
    mHeight: c_int,
    mBlockWidth: c_int,
    mBlockHeight: c_int,
    mTerxels: c_int,
    mBounds: [[f32; 3]; 2],
    mSize: [f32; 3],
    mTerxelSize: [f32; 3],
    mPatchSize: [f32; 3],
    mPatchScalarSize: f32,
    mHeightMap: *mut u8,
    mFlattenMap: *mut u8,
    mCoords: *mut [f32; 3],
    mPatches: *mut CCMPatch,
    mPatchBrushData: *mut u8,
    mBaseWaterHeight: c_int,
    mWaterHeight: f32,
    mWaterContents: c_int,
    mWaterSurfaceFlags: c_int,
    mHasPhysics: bool,
    mRandomTerrain: *mut CRandomTerrain,
    mAreas: Vec<*mut CArea>,
    mAreasIt: std::vec::IntoIter<*mut CArea>,
    holdrand: u32,
}

impl CCMLandScape {
    unsafe fn GetTerxels(&self) -> c_int {
        self.mTerxels
    }

    unsafe fn GetRealWidth(&self) -> c_int {
        self.mWidth * self.mTerxels
    }

    unsafe fn GetRealHeight(&self) -> c_int {
        self.mHeight * self.mTerxels
    }

    unsafe fn GetRealArea(&self) -> usize {
        (self.GetRealWidth() * self.GetRealHeight()) as usize
    }

    unsafe fn GetBlockCount(&self) -> c_int {
        self.mBlockWidth * self.mBlockHeight
    }

    unsafe fn GetMins(&self) -> &[f32; 3] {
        &self.mBounds[0]
    }

    unsafe fn GetPatchSize(&self) -> &[f32; 3] {
        &self.mPatchSize
    }

    unsafe fn GetTerxelSize(&self) -> &[f32; 3] {
        &self.mTerxelSize
    }

    unsafe fn GetCoords(&self) -> *mut [f32; 3] {
        self.mCoords
    }

    unsafe fn GetSurfaceFlags(&self, height: c_int) -> c_int {
        if height < 0 || height >= HEIGHT_RESOLUTION as c_int {
            0
        } else {
            // Note: This is a placeholder - would need access to mHeightDetails
            0
        }
    }

    unsafe fn GetContentFlags(&self, height: c_int) -> c_int {
        if height < 0 || height >= HEIGHT_RESOLUTION as c_int {
            0
        } else {
            // Note: This is a placeholder - would need access to mHeightDetails
            0
        }
    }

    unsafe fn GetPatch(&self, x: c_int, y: c_int) -> *mut CCMLandScape {
        (self.mPatches as *mut u8).offset(((y * self.mBlockWidth) + x) as isize * std::mem::size_of::<CCMPatch>() as isize) as *mut CCMLandScape
    }

    unsafe fn GetPatchWidth(&self) -> c_int {
        self.mBlockWidth
    }

    unsafe fn GetPatchHeight(&self) -> c_int {
        self.mBlockHeight
    }

    unsafe fn GetWidth(&self) -> c_int {
        self.mWidth
    }

    unsafe fn GetHeight(&self) -> c_int {
        self.mHeight
    }

    unsafe fn SetTerrainId(&mut self, _terrainId: c_int) {
        // Placeholder
    }

    unsafe fn SetShaders(&mut self, height: c_int, shader: *mut CCMShader) {
        let mut i = height;
        while shader.is_null() == false && i < HEIGHT_RESOLUTION as c_int {
            // Check mHeightDetails if surface flags not set
            // This is a placeholder and would need proper struct access
            i += 1;
        }
    }

    unsafe fn LoadTerrainDef(&mut self, td: *const c_char) {
        let mut terrainDef: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut parse: CGenericParser2 = std::mem::zeroed();
        let mut basegroup: *mut CGPGroup;
        let mut classes: *mut CGPGroup;
        let mut items: *mut CGPGroup;

        Com_sprintf(
            addr_of_mut!(terrainDef[0]),
            MAX_QPATH,
            b"ext_data/RMG/%s.terrain\0".as_ptr() as *const c_char,
            Info_ValueForKey(td, b"terrainDef\0".as_ptr() as *const c_char),
        );
        Com_DPrintf(
            b"CM_Terrain: Loading and parsing terrainDef %s.....\n\0".as_ptr() as *const c_char,
            Info_ValueForKey(td, b"terrainDef\0".as_ptr() as *const c_char),
        );

        if !Com_ParseTextFile(addr_of!(terrainDef[0]), addr_of_mut!(parse) as *mut c_void) {
            Com_sprintf(
                addr_of_mut!(terrainDef[0]),
                MAX_QPATH,
                b"ext_data/arioche/%s.terrain\0".as_ptr() as *const c_char,
                Info_ValueForKey(td, b"terrainDef\0".as_ptr() as *const c_char),
            );
            if !Com_ParseTextFile(addr_of!(terrainDef[0]), addr_of_mut!(parse) as *mut c_void) {
                Com_Printf(b"Could not open %s\n\0".as_ptr() as *const c_char, addr_of!(terrainDef[0]));
                return;
            }
        }
        // The whole file....
        // basegroup = parse.GetBaseParseGroup();
        // This is a placeholder and would need proper implementation

        // The root { } struct
        // classes = basegroup->GetSubGroups();
        // while(classes)
        // {
        //     items = classes->GetSubGroups();
        //     while(items)
        //     {
        //         if(!stricmp(items->GetName(), "altitudetexture"))
        //         {
        //             ...
        //         }
        //     }
        // }
        // Com_ParseTextFileDestroy(parse);
    }

    unsafe fn new(configstring: *const c_char, server: bool) -> Self {
        let mut this = CCMLandScape {
            mRefCount: 1,
            mWidth: 0,
            mHeight: 0,
            mBlockWidth: 0,
            mBlockHeight: 0,
            mTerxels: 0,
            mBounds: [[0.0; 3]; 2],
            mSize: [0.0; 3],
            mTerxelSize: [0.0; 3],
            mPatchSize: [0.0; 3],
            mPatchScalarSize: 0.0,
            mHeightMap: std::ptr::null_mut(),
            mFlattenMap: std::ptr::null_mut(),
            mCoords: std::ptr::null_mut(),
            mPatches: std::ptr::null_mut(),
            mPatchBrushData: std::ptr::null_mut(),
            mBaseWaterHeight: 0,
            mWaterHeight: 0.0,
            mWaterContents: 0,
            mWaterSurfaceFlags: 0,
            mHasPhysics: false,
            mRandomTerrain: std::ptr::null_mut(),
            mAreas: Vec::new(),
            mAreasIt: vec![].into_iter(),
            holdrand: 0x89abcdef,
        };

        let mut heightMap: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut ptr: *mut c_char = std::ptr::null_mut();

        // Clear out the height details
        // memset(this.mHeightDetails, 0, sizeof(CCMHeightDetails) * HEIGHT_RESOLUTION);

        this.mBaseWaterHeight = 0;
        this.mWaterHeight = 0.0;

        // When constructed, referenced once
        this.mRefCount = 1;

        // Extract the relevant data from the config string
        Com_sprintf(
            addr_of_mut!(heightMap[0]),
            MAX_QPATH,
            b"%s\0".as_ptr() as *const c_char,
            Info_ValueForKey(configstring, b"heightMap\0".as_ptr() as *const c_char),
        );
        this.mTerxels = atol(Info_ValueForKey(configstring, b"terxels\0".as_ptr() as *const c_char));
        this.mHasPhysics = atol(Info_ValueForKey(configstring, b"physics\0".as_ptr() as *const c_char)) != 0;
        let seed = strtoul(Info_ValueForKey(configstring, b"seed\0".as_ptr() as *const c_char), addr_of_mut!(ptr), 10);

        this.mBounds[0][0] = atof(Info_ValueForKey(configstring, b"minx\0".as_ptr() as *const c_char));
        this.mBounds[0][1] = atof(Info_ValueForKey(configstring, b"miny\0".as_ptr() as *const c_char));
        this.mBounds[0][2] = atof(Info_ValueForKey(configstring, b"minz\0".as_ptr() as *const c_char));
        this.mBounds[1][0] = atof(Info_ValueForKey(configstring, b"maxx\0".as_ptr() as *const c_char));
        this.mBounds[1][1] = atof(Info_ValueForKey(configstring, b"maxy\0".as_ptr() as *const c_char));
        this.mBounds[1][2] = atof(Info_ValueForKey(configstring, b"maxz\0".as_ptr() as *const c_char));

        // Calculate size of the brush
        VectorSubtract(addr_of!(this.mBounds[1]), addr_of!(this.mBounds[0]), addr_of_mut!(this.mSize));

        let numPatches = atol(Info_ValueForKey(configstring, b"numPatches\0".as_ptr() as *const c_char));

        // Work out the dimensions of the brush in blocks - the object is to make the blocks as square as possible
        this.mBlockWidth = Round(sqrtf((numPatches as f32) * this.mSize[0] / this.mSize[1]));
        this.mBlockHeight = Round(sqrtf((numPatches as f32) * this.mSize[1] / this.mSize[0]));

        // ...which lets us get the size of the heightmap
        this.mWidth = this.mBlockWidth * this.mTerxels;
        this.mHeight = this.mBlockHeight * this.mTerxels;

        this.mHeightMap = Z_Malloc(this.GetRealArea(), TAG_CM_TERRAIN) as *mut u8;
        this.mFlattenMap = Z_Malloc(this.GetRealArea(), TAG_CM_TERRAIN) as *mut u8;

        // Zero means unused.
        memset(this.mFlattenMap as *mut c_void, 0, this.GetRealArea());

        if strlen(addr_of!(heightMap[0])) > 0 {
            let mut imageData: *mut u8 = std::ptr::null_mut();
            let mut iWidth: c_int = 0;
            let mut iHeight: c_int = 0;

            Com_DPrintf(b"CM_Terrain: Loading heightmap %s.....\n\0".as_ptr() as *const c_char, addr_of!(heightMap[0]));
            this.mRandomTerrain = std::ptr::null_mut();

            #[cfg(not(feature = "DEDICATED"))]
            {
                R_LoadDataImage(addr_of!(heightMap[0]), addr_of_mut!(imageData), addr_of_mut!(iWidth), addr_of_mut!(iHeight));
                if !imageData.is_null() {
                    if !strstr(addr_of!(heightMap[0]), b"random_\0".as_ptr() as *const c_char).is_null() {
                        this.mRandomTerrain = CreateRandomTerrain(configstring, addr_of_mut!(this), this.mHeightMap, this.GetRealWidth(), this.GetRealHeight());
                    } else {
                        // Flip to make the same as GenSurf
                        R_InvertImage(imageData, iWidth, iHeight, 1);
                        R_Resample(imageData, iWidth, iHeight, this.mHeightMap, this.GetRealWidth(), this.GetRealHeight(), 1);
                    }
                    Z_Free(imageData as *mut c_void);
                }
            }
        } else {
            Com_Error(ERR_FATAL, b"Terrain has no heightmap specified\n\0".as_ptr() as *const c_char);
        }

        // Work out the dimensions of the terxel - should be almost square
        this.mTerxelSize[0] = this.mSize[0] / (this.mWidth as f32);
        this.mTerxelSize[1] = this.mSize[1] / (this.mHeight as f32);
        this.mTerxelSize[2] = this.mSize[2] / 255.0;

        // Work out the patchsize
        this.mPatchSize[0] = this.mSize[0] / (this.mBlockWidth as f32);
        this.mPatchSize[1] = this.mSize[1] / (this.mBlockHeight as f32);
        this.mPatchSize[2] = 1.0;
        this.mPatchScalarSize = VectorLength(addr_of!(this.mPatchSize));

        // Loads in the water height and properties
        // Gets the shader properties for the blended shaders
        this.LoadTerrainDef(configstring);

        Com_DPrintf(b"CM_Terrain: Creating patches.....\n\0".as_ptr() as *const c_char);
        this.mPatches = Z_Malloc(std::mem::size_of::<CCMPatch>() * this.GetBlockCount() as usize, TAG_CM_TERRAIN) as *mut CCMPatch;

        let numBrushesPerPatch = this.mTerxels * this.mTerxels * 2;
        let size = ((numBrushesPerPatch * std::mem::size_of::<cbrush_t>() as c_int)
            + (numBrushesPerPatch * BRUSH_SIDES_PER_TERXEL as c_int * 2 * (std::mem::size_of::<cbrushside_t>() as c_int + std::mem::size_of::<cplane_t>() as c_int))) as usize;
        this.mPatchBrushData = Z_Malloc(size * this.GetBlockCount() as usize, TAG_CM_TERRAIN) as *mut u8;

        // Initialize all terrain patches
        this.UpdatePatches();

        this
    }

    unsafe fn PatchCollide(&mut self, tw: *mut traceWork_s, trace: &mut trace_t, start: *const [f32; 3], end: *const [f32; 3], checkcount: c_int) {
        let mut tBounds: [[f32; 3]; 2] = [[0.0; 3]; 2];

        // Convert to valid bounding box
        CM_CalcExtents(start, end, tw, addr_of_mut!(tBounds));

        //if (com_newtrace->integer)
        if true {
            let mut slope: f32;
            let mut offset: f32;
            let mut startPatchLoc: f32;
            let mut endPatchLoc: f32;
            let mut startPos: f32;
            let mut endPos: f32;
            let mut patchDirection: f32 = 1.0;
            let mut checkDirection: f32 = 1.0;
            let mut countPatches: c_int;
            let mut count: c_int;
            let mut patch: *mut CCMPatch;
            let fraction: f32 = trace.fraction;

            if fabs((*end)[0] - (*start)[0]) >= fabs(fabs((*end)[1] - (*start)[1])) {
                // x travels more than y
                // calculate line slope and offset
                if ((*end)[0] - (*start)[0]) != 0.0 {
                    slope = ((*end)[1] - (*start)[1]) / ((*end)[0] - (*start)[0]);
                } else {
                    slope = 0.0;
                }
                offset = (*start)[1] - ((*start)[0] * slope);

                // find the starting
                startPatchLoc = floorf(((*start)[0] - self.mBounds[0][0]) / self.mPatchSize[0]);
                endPatchLoc = floorf(((*end)[0] - self.mBounds[0][0]) / self.mPatchSize[0]);

                if startPatchLoc <= endPatchLoc {
                    // moving along slope in a positive direction
                    endPatchLoc += 1.0;
                    startPatchLoc -= 1.0;
                    countPatches = (endPatchLoc - startPatchLoc + 1.0) as c_int;
                } else {
                    // moving along slope in a negative direction
                    endPatchLoc -= 1.0;
                    startPatchLoc += 1.0;
                    patchDirection = -1.0;
                    countPatches = (startPatchLoc - endPatchLoc + 1.0) as c_int;
                }
                if slope < 0.0 {
                    checkDirection = -1.0;
                }

                // first calculate the real world location
                startPos = ((startPatchLoc * self.mPatchSize[0] + self.mBounds[0][0]) * slope) + offset;
                // calculate it back into patch coords
                startPos = floorf((startPos - self.mBounds[0][1] + (*tw).size[0][1]) / self.mPatchSize[1]);
                loop {
                    if startPatchLoc as c_int >= 0 && (startPatchLoc as c_int) < self.mBlockWidth {
                        // valid location
                        // first calculate the real world location
                        endPos = (((startPatchLoc + patchDirection) * self.mPatchSize[0] + self.mBounds[0][0]) * slope) + offset;
                        // calculate it back into patch coords
                        endPos = floorf((endPos - self.mBounds[0][1] + (*tw).size[1][1]) / self.mPatchSize[1]);

                        if checkDirection < 0.0 {
                            startPos += 1.0;
                            endPos -= 1.0;
                        } else {
                            startPos -= 1.0;
                            endPos += 1.0;
                        }
                        count = (fabs(endPos - startPos) + 1.0) as c_int;
                        while count > 0 {
                            if startPos as c_int >= 0 && (startPos as c_int) < self.mBlockHeight {
                                // valid location
                                patch = self.GetPatch(startPatchLoc as c_int, startPos as c_int) as *mut CCMPatch;
                                // Collide with every patch to find the minimum fraction
                                CM_HandlePatchCollision(tw, trace, addr_of!(tBounds[0]), addr_of!(tBounds[1]), patch, checkcount);

                                if trace.fraction <= 0.0 {
                                    return;
                                }
                            }
                            startPos += checkDirection;
                            count -= 1;
                        }

                        if trace.fraction < fraction {
                            return;
                        }
                    }
                    // move to the next spot
                    // we still stay one behind, to get the opposite edge of the terrain patch
                    startPos = ((startPatchLoc * self.mPatchSize[0] + self.mBounds[0][0]) * slope) + offset;
                    startPatchLoc += patchDirection;
                    // first calculate the real world location
                    // calculate it back into patch coords
                    startPos = floorf((startPos - self.mBounds[0][1] + (*tw).size[0][1]) / self.mPatchSize[1]);

                    countPatches -= 1;
                    if countPatches <= 0 {
                        break;
                    }
                }
            } else {
                // calculate line slope and offset
                slope = ((*end)[0] - (*start)[0]) / ((*end)[1] - (*start)[1]);
                offset = (*start)[0] - ((*start)[1] * slope);

                // find the starting
                startPatchLoc = floorf(((*start)[1] - self.mBounds[0][1]) / self.mPatchSize[1]);
                endPatchLoc = floorf(((*end)[1] - self.mBounds[0][1]) / self.mPatchSize[1]);

                if startPatchLoc <= endPatchLoc {
                    // moving along slope in a positive direction
                    endPatchLoc += 1.0;
                    startPatchLoc -= 1.0;
                    countPatches = (endPatchLoc - startPatchLoc + 1.0) as c_int;
                } else {
                    // moving along slope in a negative direction
                    endPatchLoc -= 1.0;
                    startPatchLoc += 1.0;
                    patchDirection = -1.0;
                    countPatches = (startPatchLoc - endPatchLoc + 1.0) as c_int;
                }
                if slope < 0.0 {
                    checkDirection = -1.0;
                }

                // first calculate the real world location
                startPos = ((startPatchLoc * self.mPatchSize[1] + self.mBounds[0][1]) * slope) + offset;
                // calculate it back into patch coords
                startPos = floorf((startPos - self.mBounds[0][0] + (*tw).size[0][0]) / self.mPatchSize[0]);
                loop {
                    if startPatchLoc as c_int >= 0 && (startPatchLoc as c_int) < self.mBlockHeight {
                        // valid location
                        // first calculate the real world location
                        endPos = (((startPatchLoc + patchDirection) * self.mPatchSize[1] + self.mBounds[0][1]) * slope) + offset;
                        // calculate it back into patch coords
                        endPos = floorf((endPos - self.mBounds[0][0] + (*tw).size[1][0]) / self.mPatchSize[0]);

                        if checkDirection < 0.0 {
                            startPos += 1.0;
                            endPos -= 1.0;
                        } else {
                            startPos -= 1.0;
                            endPos += 1.0;
                        }

                        count = (fabs(endPos - startPos) + 1.0) as c_int;
                        while count > 0 {
                            if startPos as c_int >= 0 && (startPos as c_int) < self.mBlockWidth {
                                // valid location
                                patch = self.GetPatch(startPos as c_int, startPatchLoc as c_int) as *mut CCMPatch;
                                // Collide with every patch to find the minimum fraction
                                CM_HandlePatchCollision(tw, trace, addr_of!(tBounds[0]), addr_of!(tBounds[1]), patch, checkcount);

                                if trace.fraction <= 0.0 {
                                    return;
                                }
                            }
                            startPos += checkDirection;
                            count -= 1;
                        }

                        if trace.fraction < fraction {
                            return;
                        }
                    }

                    // move to the next spot
                    // we still stay one behind, to get the opposite edge of the terrain patch
                    startPos = ((startPatchLoc * self.mPatchSize[1] + self.mBounds[0][1]) * slope) + offset;
                    startPatchLoc += patchDirection;
                    // first calculate the real world location
                    // calculate it back into patch coords
                    startPos = floorf((startPos - self.mBounds[0][0] + (*tw).size[0][0]) / self.mPatchSize[0]);
                    countPatches -= 1;
                    if countPatches <= 0 {
                        break;
                    }
                }
            }
        } else {
            let mut x: c_int;
            let mut y: c_int;
            let mut tWork: [f32; 3] = [0.0; 3];
            let mut pStart: [f32; 3] = [0.0; 3];
            let mut pEnd: [f32; 3] = [0.0; 3];
            let mut minx: c_int;
            let mut maxx: c_int;
            let mut miny: c_int;
            let mut maxy: c_int;
            let mut patch: *mut CCMPatch;

            // Work out and grab the relevant patches
            VectorSubtract(addr_of!(tBounds[0]), addr_of!(self.mBounds[0]), addr_of_mut!(tWork));
            VectorInverseScaleVector(addr_of!(tWork), addr_of!(self.mPatchSize), addr_of_mut!(pStart));
            VectorSubtract(addr_of!(tBounds[1]), addr_of!(self.mBounds[0]), addr_of_mut!(tWork));
            VectorInverseScaleVector(addr_of!(tWork), addr_of!(self.mPatchSize), addr_of_mut!(pEnd));

            minx = Com_Clamp(0, self.mBlockWidth - 1, floorf(pStart[0]) as c_int);
            maxx = Com_Clamp(0, self.mBlockWidth - 1, ceilf(pEnd[0]) as c_int);
            miny = Com_Clamp(0, self.mBlockHeight - 1, floorf(pStart[1]) as c_int);
            maxy = Com_Clamp(0, self.mBlockHeight - 1, ceilf(pEnd[1]) as c_int);

            // generic box collide with each one
            y = miny;
            while y <= maxy {
                x = minx;
                while x <= maxx {
                    patch = self.GetPatch(x, y) as *mut CCMPatch;
                    // Collide with every patch to find the minimum fraction
                    CM_HandlePatchCollision(tw, trace, addr_of!(tBounds[0]), addr_of!(tBounds[1]), patch, checkcount);

                    if trace.fraction <= 0.0 {
                        break;
                    }
                    x += 1;
                }
                y += 1;
            }
        }
    }

    unsafe fn WaterCollide(&self, begin: *const [f32; 3], end: *const [f32; 3], fraction: f32) -> f32 {
        // Check for completely above water
        if ((*begin)[2] > self.mWaterHeight) && ((*end)[2] > self.mWaterHeight) {
            return fraction;
        }
        // Check for completely below water
        if ((*begin)[2] < self.mWaterHeight) && ((*end)[2] < self.mWaterHeight) {
            return fraction;
        }
        // Check for starting in water and leaving
        if (*begin)[2] < self.mWaterHeight - SURFACE_CLIP_EPSILON {
            return ((self.mWaterHeight - SURFACE_CLIP_EPSILON) - (*begin)[2]) / ((*end)[2] - (*begin)[2]);
        }
        // Now the trace must be entering the water
        if (*begin)[2] > self.mWaterHeight + SURFACE_CLIP_EPSILON {
            return ((*begin)[2] - (self.mWaterHeight + SURFACE_CLIP_EPSILON)) / ((*begin)[2] - (*end)[2]);
        }
        fraction
    }

    unsafe fn GetTerxelLocalCoords(&self, x: c_int, y: c_int, localCoords: &mut [[f32; 3]; 8]) {
        let realWidth: c_int = self.GetRealWidth();
        let coords: *mut [f32; 3] = self.GetCoords();
        let mut offsets: [c_int; 4] = [0; 4];
        let mut i: c_int;

        if (x + y) & 1 != 0 {
            offsets[0] = (y * realWidth) + x;
            offsets[1] = (y * realWidth) + x + 1;
            offsets[2] = ((y + 1) * realWidth) + x;
            offsets[3] = ((y + 1) * realWidth) + x + 1;
        } else {
            offsets[2] = (y * realWidth) + x;
            offsets[0] = (y * realWidth) + x + 1;
            offsets[3] = ((y + 1) * realWidth) + x;
            offsets[1] = ((y + 1) * realWidth) + x + 1;
        }

        i = 0;
        while i < 4 {
            VectorCopy(addr_of!((*coords.offset(offsets[i as usize] as isize))), addr_of_mut!(localCoords[i as usize]));
            VectorCopy(addr_of!((*coords.offset(offsets[i as usize] as isize))), addr_of_mut!(localCoords[(i + 4) as usize]));

            // Set z of base of brush to bottom of landscape brush
            localCoords[(i + 4) as usize][2] = self.GetMins()[2];
            i += 1;
        }
    }

    unsafe fn UpdatePatches(&mut self) {
        let mut patch: *mut CCMPatch;
        let mut x: c_int;
        let mut y: c_int;
        let mut ix: c_int;
        let mut iy: c_int;
        let mut numBrushesPerPatch: c_int;
        let mut world: [f32; 3] = [0.0; 3];
        let size: usize;

        // Calculate real world coordinates from the heightmap
        self.CalcRealCoords();

        numBrushesPerPatch = self.mTerxels * self.mTerxels * 2;
        size = ((numBrushesPerPatch as usize * std::mem::size_of::<cbrush_t>())
            + (numBrushesPerPatch as usize * BRUSH_SIDES_PER_TERXEL * 2 * (std::mem::size_of::<cbrushside_t>() + std::mem::size_of::<cplane_t>()))) as usize;

        patch = self.mPatches;
        y = 0;
        iy = 0;
        while y < self.mHeight {
            x = 0;
            ix = 0;
            while x < self.mWidth {
                VectorSet(addr_of_mut!(world), self.mBounds[0][0] + ((x as f32) * self.mTerxelSize[0]), self.mBounds[0][1] + ((y as f32) * self.mTerxelSize[1]), self.mBounds[0][2]);
                (*patch).Init(
                    self,
                    x,
                    y,
                    addr_of!(world),
                    self.mHeightMap,
                    (self.mPatchBrushData as *mut u8).offset((size as isize * ((ix + (iy * self.mBlockWidth)) as isize)) as isize) as *mut cbrush_t,
                );
                x += self.mTerxels;
                ix += 1;
                patch = patch.offset(1);
            }
            y += self.mTerxels;
            iy += 1;
        }

        // Cleanup coord array
        Z_Free(self.mCoords as *mut c_void);
    }

    unsafe fn CalcRealCoords(&mut self) {
        let mut x: c_int;
        let mut y: c_int;

        self.mCoords = Z_Malloc(std::mem::size_of::<[f32; 3]>() * (self.GetRealWidth() * self.GetRealHeight()) as usize, TAG_CM_TERRAIN_TEMP) as *mut [f32; 3];

        // Work out the real world coordinates of each heightmap entry
        y = 0;
        while y < self.GetRealHeight() {
            x = 0;
            while x < self.GetRealWidth() {
                let mut icoords: [c_int; 3] = [0; 3];
                let offset: c_int = (y * self.GetRealWidth()) + x;

                VectorSet(addr_of_mut!(icoords), x, y, *(self.mHeightMap.offset(offset as isize)) as c_int);
                VectorScaleVectorAdd(addr_of!(self.mBounds[0]), addr_of!(icoords), addr_of!(self.mTerxelSize), addr_of_mut!((*self.mCoords.offset(offset as isize))));
                x += 1;
            }
            y += 1;
        }
    }

    unsafe fn TerrainPatchIterate(&self, IterateFunc: extern "C" fn(*mut CCMPatch, *mut c_void), userdata: *mut c_void) {
        let mut i: c_int = 0;
        let mut patch: *mut CCMPatch = self.mPatches;

        while i < self.GetBlockCount() {
            IterateFunc(patch, userdata);
            patch = patch.offset(1);
            i += 1;
        }
    }

    unsafe fn GetWorldHeight(&self, origin: *mut [f32; 3], bounds: *const [[f32; 3]; 2], aboveGround: bool) -> f32 {
        let mut work: [f32; 3] = [0.0; 3];
        let mut minx: c_int;
        let mut maxx: c_int;
        let mut miny: c_int;
        let mut maxy: c_int;
        let mut TL: u8;
        let mut TR: u8;
        let mut BL: u8;
        let mut BR: u8;
        let mut final_val: f32;

        VectorSubtract(origin, addr_of!(self.mBounds[0]), addr_of_mut!(work));
        VectorInverseScaleVector(addr_of!(work), addr_of!(self.mTerxelSize), addr_of_mut!(work));

        // Presume the bases of all misc models are less than 1 terxel square
        minx = Com_Clamp(0, self.GetWidth(), floorf(work[0]) as c_int);
        maxx = Com_Clamp(0, self.GetWidth(), ceilf(work[0]) as c_int);
        miny = Com_Clamp(0, self.GetHeight(), floorf(work[1]) as c_int);
        maxy = Com_Clamp(0, self.GetHeight(), ceilf(work[1]) as c_int);

        TL = *(self.mHeightMap.offset(((miny * self.GetRealWidth()) + minx) as isize));
        TR = *(self.mHeightMap.offset(((miny * self.GetRealWidth()) + maxx) as isize));
        BL = *(self.mHeightMap.offset(((maxy * self.GetRealWidth()) + minx) as isize));
        BR = *(self.mHeightMap.offset(((maxy * self.GetRealWidth()) + maxx) as isize));

        if aboveGround {
            let h1: f32;
            let h2: f32;
            let tx: f32;
            let ty: f32;
            tx = (work[0] - minx as f32) / ((maxx - minx) as f32);
            ty = (work[1] - miny as f32) / ((maxy - miny) as f32);
            h1 = LERP(tx, TL as f32, TR as f32);
            h2 = LERP(tx, BL as f32, BR as f32);
            final_val = LERP(ty, h1, h2);
        } else {
            let min1: u8;
            let min2: u8;

            min1 = if TL < TR { TL } else { TR };
            min2 = if BL < BR { BL } else { BR };
            final_val = if min1 < min2 { min1 as f32 } else { min2 as f32 };
        }
        (*origin)[2] = ((final_val as c_int) as f32 * self.mTerxelSize[2]) + self.mBounds[0][2];

        // compute slope at this spot
        if maxx == minx {
            maxx = Com_Clamp(0, self.GetWidth(), minx + 1);
        }
        if maxy == miny {
            maxy = Com_Clamp(0, self.GetHeight(), miny + 1);
        }
        BR = *(self.mHeightMap.offset(((maxy * self.GetRealWidth()) + maxx) as isize));

        // rise over run
        (fabs((BR as f32 - TL as f32) * self.mTerxelSize[2]) / self.mTerxelSize[0])
    }

    unsafe fn SaveArea(&mut self, area: *mut CArea) {
        self.mAreas.push(area);
    }

    unsafe fn CarveLine(&mut self, start: *const [f32; 3], end: *const [f32; 3], depth: c_int, width: c_int) {
        let mut x: c_int 	 = ((*start)[0]) as c_int;
        let mut y: c_int 	 = ((*start)[1]) as c_int;
        let x1: c_int = x;
        let y1: c_int = y;
        let x2: c_int = ((*end)[0]) as c_int;
        let y2: c_int = ((*end)[1]) as c_int;

        let deltax = if x2 >= x1 { x2 - x1 } else { x1 - x2 };
        let deltay = if y2 >= y1 { y2 - y1 } else { y1 - y2 };

        let mut xinc1: c_int;
        let mut xinc2: c_int;
        let mut yinc1: c_int;
        let mut yinc2: c_int;
        let mut den: c_int;
        let mut num: c_int;
        let mut add: c_int;
        let mut count: c_int;

        // The x-values are increasing
        if x2 >= x1 {
            xinc1 = 1;
            xinc2 = 1;
        } else {
            // The x-values are decreasing
            xinc1 = -1;
            xinc2 = -1;
        }

        // The y-values are increasing
        if y2 >= y1 {
            yinc1 = 1;
            yinc2 = 1;
        } else {
            // The y-values are decreasing
            yinc1 = -1;
            yinc2 = -1;
        }

        if deltax >= deltay {
            // There is at least one x-value for every y-value
            xinc1 = 0;
            yinc2 = 0;
            den = deltax;
            num = deltax / 2;
            add = deltay;
            count = deltax;
        } else {
            // There is at least one y-value for every x-value
            xinc2 = 0;
            yinc1 = 0;
            den = deltay;
            num = deltay / 2;
            add = deltax;
            count = deltay;
        }

        let mut pt: [f32; 3] = [0.0; 3];
        let bounds: [[f32; 3]; 2] = [[-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]];

        pt[0] = (*start)[0];
        pt[1] = (*start)[1];
        let heightStart: f32 = self.GetWorldHeight(addr_of_mut!(pt), addr_of!(bounds), false);

        pt[0] = (*end)[0];
        pt[1] = (*end)[1];
        let heightEnd: f32 = self.GetWorldHeight(addr_of_mut!(pt), addr_of!(bounds), false);

        let heightStep: f32 = (heightEnd - heightStart) / (count as f32);

        let mut i: c_int = 0;
        while i <= count {
            pt[0] = x as f32;
            pt[1] = y as f32;
            // CArea area;
            // area.Init(pt, width / 2 + (irand(0, width/2)));
            // FlattenArea(&area, heightStart + (heightStep * i) - (depth/2 - (irand(0, depth/2))), false, true, true);
            // Placeholder - needs CArea implementation

            num += add;

            if num >= den {
                num -= den;
                x += xinc1;
                y += yinc1;
            }

            x += xinc2;
            y += yinc2;
            i += 1;
        }
    }

    unsafe fn CarveBezierCurve(&mut self, numCtlPoints: c_int, ctlPoints: *mut [f32; 3], steps: c_int, depth: c_int, size: c_int) {
        let mut i: c_int;
        let mut choose: c_int;
        let n: c_int = numCtlPoints - 1;
        let mut u: f32;
        let mut t: f32;
        let mut tt: f32;
        let t1: f32;
        let step: f32 = 1.0 / (steps as f32);
        let mut pt: [f32; 3] = [0.0; 3];
        let mut lastpt: [f32; 3] = [0.0; 3];
        let mut b: [[f32; 3]; 10] = [[0.0; 3]; 10];

        choose = 1;

        i = 1;
        while i <= n {
            if i == 1 {
                choose = n;
            } else {
                choose = choose * (n - i + 1) / i;
            }

            (*(ctlPoints.offset(i as isize)))[0] *= choose as f32;
            (*(ctlPoints.offset(i as isize)))[1] *= choose as f32;
            i += 1;
        }

        choose = 0;
        t = step;
        while t < 1.0 {
            b[0][0] = (*(ctlPoints.offset(0)))[0];
            b[0][1] = (*(ctlPoints.offset(0)))[1];

            u = t;
            i = 1;
            while i <= n {
                b[i as usize][0] = (*(ctlPoints.offset(i as isize)))[0] * u;
                b[i as usize][1] = (*(ctlPoints.offset(i as isize)))[1] * u;

                u = u * t;
                i += 1;
            }

            pt[0] = b[n as usize][0];
            pt[1] = b[n as usize][1];

            let t1 = 1.0 - t;
            tt = t1;

            i = n - 1;
            while i >= 0 {
                pt[0] += b[i as usize][0] * tt;
                pt[1] += b[i as usize][1] * tt;

                tt = tt * t1;
                i -= 1;
            }

            if choose != 0 {
                self.CarveLine(addr_of!(lastpt), addr_of!(pt), depth, size);
            }

            lastpt[0] = pt[0];
            lastpt[1] = pt[1];
            t += step;
            choose += 1;
        }
    }

    unsafe fn FlattenArea(&mut self, area: *mut CArea, height: c_int, save: bool, forceHeight: bool, smooth: bool) {
        let mut temp: [f32; 3] = [0.0; 3];
        let mut icoords: [c_int; 3] = [0; 3];
        let mut radius: c_int;
        let mut height2: c_int = height;

        if save {
            self.SaveArea(area);
        }

        // Work out coords in the heightmap
        // Note: This would need proper implementation with actual area and bounds calculations
    }

    unsafe fn FractionBelowLevel(&self, area: *mut CArea, height: c_int) -> f32 {
        // Placeholder - needs CArea implementation
        0.0
    }

    unsafe fn GetFirstArea(&mut self) -> *mut CArea {
        if self.mAreas.is_empty() {
            std::ptr::null_mut()
        } else {
            self.mAreasIt = self.mAreas.clone().into_iter();
            self.mAreasIt.next().unwrap_or(std::ptr::null_mut())
        }
    }

    unsafe fn GetFirstObjectiveArea(&mut self) -> *mut CArea {
        if self.mAreas.is_empty() {
            std::ptr::null_mut()
        } else {
            self.mAreasIt = self.mAreas.clone().into_iter();
            while let Some(area) = self.mAreasIt.next() {
                // if(*area)->GetType() == AT_OBJECTIVE
                // return area;
            }
            std::ptr::null_mut()
        }
    }

    unsafe fn GetPlayerArea(&mut self) -> *mut CArea {
        if self.mAreas.is_empty() {
            std::ptr::null_mut()
        } else {
            self.mAreasIt = self.mAreas.clone().into_iter();
            while let Some(area) = self.mAreasIt.next() {
                // if(*area)->GetType() == AT_PLAYER
                // return area;
            }
            std::ptr::null_mut()
        }
    }

    unsafe fn GetNextArea(&mut self) -> *mut CArea {
        self.mAreasIt.next().unwrap_or(std::ptr::null_mut())
    }

    unsafe fn GetNextObjectiveArea(&mut self) -> *mut CArea {
        while let Some(area) = self.mAreasIt.next() {
            // if(*area)->GetType() == AT_OBJECTIVE
            // return area;
        }
        std::ptr::null_mut()
    }

    unsafe fn AreaCollision(&self, area: *mut CArea, areaTypes: *const c_int, areaTypeCount: c_int) -> bool {
        // Placeholder - needs CArea implementation
        false
    }

    unsafe fn rand_seed(&mut self, seed: u32) {
        self.holdrand = seed;
        Com_Printf(b"rand_seed = %d\n\0".as_ptr() as *const c_char, seed);
    }

    unsafe fn flrand(&mut self, min: f32, max: f32) -> f32 {
        let result: f32;

        assert!((max - min) < 32768.0);

        self.holdrand = (self.holdrand.wrapping_mul(214013u32)).wrapping_add(2531011u32);
        result = ((self.holdrand >> 17) as f32);
        ((result * (max - min)) / 32768.0) + min
    }

    unsafe fn irand(&mut self, min: c_int, max: c_int) -> c_int {
        let mut result: c_int;

        assert!((max - min) < 32768);

        let max_mut = max + 1;
        self.holdrand = (self.holdrand.wrapping_mul(214013u32)).wrapping_add(2531011u32);
        result = (self.holdrand >> 17) as c_int;
        result = (((result * (max_mut - min)) >> 15) as c_int) + min;

        result
    }
}

impl Drop for CCMLandScape {
    fn drop(&mut self) {
        unsafe {
            if !self.mHeightMap.is_null() {
                Z_Free(self.mHeightMap as *mut c_void);
                self.mHeightMap = std::ptr::null_mut();
            }
            if !self.mFlattenMap.is_null() {
                Z_Free(self.mFlattenMap as *mut c_void);
                self.mFlattenMap = std::ptr::null_mut();
            }
            if !self.mPatchBrushData.is_null() {
                Z_Free(self.mPatchBrushData as *mut c_void);
                self.mPatchBrushData = std::ptr::null_mut();
            }
            if !self.mPatches.is_null() {
                Z_Free(self.mPatches as *mut c_void);
                self.mPatches = std::ptr::null_mut();
            }
            if !self.mRandomTerrain.is_null() {
                // delete mRandomTerrain;
            }

            for area in &self.mAreas {
                // delete *area;
            }

            self.mAreas.clear();
        }
    }
}

// Helper functions for circular iteration
extern "C" fn CM_CircularIterate(
    data: *mut u8,
    width: c_int,
    height: c_int,
    xo: c_int,
    yo: c_int,
    insideRadius: c_int,
    outsideRadius: c_int,
    user: *mut c_int,
    callback: extern "C" fn(*mut u8, f32, *mut c_int),
) {
    let mut x: c_int;
    let mut y: c_int;
    let mut offset: c_int;
    let mut work: *mut u8;

    y = -outsideRadius;
    while y < outsideRadius + 1 {
        if y + yo >= 0 && y + yo < height {
            offset = sqrtf(((outsideRadius * outsideRadius) - (y * y)) as f32) as c_int;
            x = -offset;
            while x < offset + 1 {
                if x + xo >= 0 && x + xo < width {
                    let radius: f32 = sqrtf(((x * x + y * y) as f32));

                    if radius as c_int >= insideRadius {
                        work = (data as *mut u8).offset(((x + xo) + ((y + yo) * width)) as isize);
                        callback(work, (radius - (insideRadius as f32)) / ((outsideRadius - insideRadius) as f32), user);
                    }
                }
                x += 1;
            }
        }
        y += 1;
    }
}

extern "C" fn CM_ForceHeight(work: *mut u8, _lerp: f32, user: *mut c_int) {
    unsafe {
        *work = (Com_Clamp(0, 255, *user) as u8);
    }
}

extern "C" fn CM_GetAverage(work: *mut u8, _lerp: f32, user: *mut c_int) {
    unsafe {
        *user.offset(0) += *work as c_int;
        *user.offset(1) += 1;
    }
}

extern "C" fn CM_Smooth(work: *mut u8, lerp: f32, user: *mut c_int) {
    unsafe {
        let smooth: f32 = sinf(M_PI / 2.0 * 3.0 + (1.0 - lerp) * (M_PI / 2.0)) + 1.0;
        *work = (*work as c_int + ((*user as f32 - *work as f32) * smooth) as c_int) as u8;
    }
}

extern "C" fn CM_MakeAverage(work: *mut u8, _lerp: f32, user: *mut c_int) {
    unsafe {
        let height: c_int = *work as c_int;
        let diff: c_int = *user - height;
        let new_diff = if diff.abs() > 3 { diff >> 2 } else { diff };
        let new_height = (height + new_diff) as u8;
        *work = new_height;
    }
}

extern "C" fn CM_BelowLevel(data: *mut u8, _lerp: f32, info: *mut c_int) {
    unsafe {
        *info.offset(1) += 1;
        if (*data as c_int) < *info.offset(2) {
            *info.offset(0) += 1;
        }
    }
}

// C API functions
#[no_mangle]
pub unsafe extern "C" fn CM_InitTerrain(configstring: *const c_char, terrainId: c_int, server: bool) -> *mut CCMLandScape {
    let mut ls: *mut CCMLandScape = Box::leak(Box::new(CCMLandScape::new(configstring, server)));
    (*ls).SetTerrainId(terrainId);
    ls
}

#[no_mangle]
pub unsafe extern "C" fn CM_TerrainPatchIterate(
    landscape: *const CCMLandScape,
    IterateFunc: extern "C" fn(*mut CCMPatch, *mut c_void),
    userdata: *mut c_void,
) {
    (*landscape as *mut CCMLandScape).TerrainPatchIterate(IterateFunc, userdata);
}

#[no_mangle]
pub unsafe extern "C" fn CM_GetWorldHeight(landscape: *const CCMLandScape, origin: *mut [f32; 3], bounds: *const [[f32; 3]; 2], aboveGround: bool) -> f32 {
    (*landscape as *mut CCMLandScape).GetWorldHeight(origin, bounds, aboveGround)
}

#[no_mangle]
pub unsafe extern "C" fn CM_FlattenArea(landscape: *mut CCMLandScape, area: *mut CArea, height: c_int, save: bool, forceHeight: bool, smooth: bool) {
    (*landscape).FlattenArea(area, height, save, forceHeight, smooth);
}

#[no_mangle]
pub unsafe extern "C" fn CM_CarveBezierCurve(landscape: *mut CCMLandScape, numCtls: c_int, ctls: *mut [f32; 3], steps: c_int, depth: c_int, size: c_int) {
    (*landscape).CarveBezierCurve(numCtls, ctls, steps, depth, size);
}

#[no_mangle]
pub unsafe extern "C" fn CM_SaveArea(landscape: *mut CCMLandScape, area: *mut CArea) {
    (*landscape).SaveArea(area);
}

#[no_mangle]
pub unsafe extern "C" fn CM_FractionBelowLevel(landscape: *mut CCMLandScape, area: *mut CArea, height: c_int) -> f32 {
    (*landscape).FractionBelowLevel(area, height)
}

#[no_mangle]
pub unsafe extern "C" fn CM_AreaCollision(landscape: *mut CCMLandScape, area: *mut CArea, areaTypes: *const c_int, areaTypeCount: c_int) -> bool {
    (*landscape).AreaCollision(area, areaTypes, areaTypeCount)
}

#[no_mangle]
pub unsafe extern "C" fn CM_GetFirstArea(landscape: *mut CCMLandScape) -> *mut CArea {
    (*landscape).GetFirstArea()
}

#[no_mangle]
pub unsafe extern "C" fn CM_GetFirstObjectiveArea(landscape: *mut CCMLandScape) -> *mut CArea {
    (*landscape).GetFirstObjectiveArea()
}

#[no_mangle]
pub unsafe extern "C" fn CM_GetPlayerArea(landscape: *mut CCMLandScape) -> *mut CArea {
    (*landscape).GetPlayerArea()
}

#[no_mangle]
pub unsafe extern "C" fn CM_GetNextArea(landscape: *mut CCMLandScape) -> *mut CArea {
    (*landscape).GetNextArea()
}

#[no_mangle]
pub unsafe extern "C" fn CM_GetNextObjectiveArea(landscape: *mut CCMLandScape) -> *mut CArea {
    (*landscape).GetNextObjectiveArea()
}

#[no_mangle]
pub unsafe extern "C" fn CreateRandomTerrain(config: *const c_char, landscape: *mut CCMLandScape, heightmap: *mut u8, width: c_int, height: c_int) -> *mut CRandomTerrain {
    #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
    {
        let mut ptr: *mut c_char = std::ptr::null_mut();
        let seed: u32 = strtoul(Info_ValueForKey(config, b"seed\0".as_ptr() as *const c_char), addr_of_mut!(ptr), 10);

        (*landscape).rand_seed(seed);

        let RandomTerrain: *mut CRandomTerrain = Box::leak(Box::new(unsafe { std::mem::zeroed::<CRandomTerrain>() }));
        // RandomTerrain->Init(landscape, heightmap, width, height);
        RandomTerrain
    }

    #[cfg(feature = "PRE_RELEASE_DEMO")]
    {
        std::ptr::null_mut()
    }
}

// end
