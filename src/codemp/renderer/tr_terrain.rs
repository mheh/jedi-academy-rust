#![allow(non_snake_case)]

// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// this include must remain at the top of every CPP file
// #include "tr_local.h"

// #if !defined(GENERICPARSER2_H_INC)
//     #include "../qcommon/GenericParser2.h"
// #endif

// To do:
// Alter variance dependent on global distance from player (colour code this for cg_terrainCollisionDebug)
// Improve texture blending on edge conditions
// Link to neightbouring terrains or architecture (edge conditions)
// Post process generated light data to make sure there are no bands within a patch

// #include "../qcommon/cm_landscape.h"
// #include "tr_landscape.h"

use core::ffi::{c_int, c_char, c_void, c_long, c_float};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::renderer::tr_local_h::*;
use crate::codemp::qcommon::cm_landscape_h::*;
use crate::codemp::renderer::tr_landscape_h::*;
use crate::codemp::qcommon::GenericParser2_h::*;

// Type aliases for clarity
type vec3_t = [f32; 3];
type vec3pair_t = [[f32; 3]; 2];
type color4ub_t = [u8; 4];
type byte = u8;
type qhandle_t = c_int;
type thandle_t = c_int;
type ivec5_t = [c_int; 5];

const TAG_R_TERRAIN: c_int = 8;
const MAX_QPATH: usize = 256;
const HEIGHT_RESOLUTION: usize = 256;
const CVAR_CHEAT: c_int = 16;
const SF_TERRAIN: c_int = 9;
const RDF_NOWORLDMODEL: c_int = 0x0001;
const RDF_PROJECTION2D: c_int = 0x0002;


extern "C" {
    // CVars
    pub static mut r_terrainTessellate: *mut cvar_t;
    pub static mut r_drawTerrain: *mut cvar_t;
    pub static mut r_showFrameVariance: *mut cvar_t;
    pub static mut r_terrainWaterOffset: *mut cvar_t;

    // Global renderer state
    pub static mut backEnd: backEndState_t;
    pub static mut tr: trGlobals_t;
    pub static mut tess: shaderCommands_t;
    pub static mut common: *mut CCommon;

    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;

    // Terrain functions
    fn RB_CheckOverflow(verts: c_int, indexes: c_int);
    fn RB_EndSurface();
    fn RB_BeginSurface(shader: *mut shader_t, fog: c_int);
    fn R_LightForPoint(
        coords: *const vec3_t,
        ambient: *mut vec3_t,
        directed: *mut vec3_t,
        direction: *mut vec3_t,
    ) -> bool;
    fn DistanceSquared(a: *const vec3_t, b: *const vec3_t) -> f32;
    fn CM_CullWorldBox(frustum: *const cplane_t, bounds: *const vec3pair_t) -> bool;
    fn CM_RegisterTerrain(config: *const c_char, server: bool) -> *mut CCMLandScape;
    fn CM_TerrainPatchIterate(
        landscape: *const CCMLandScape,
        callback: unsafe extern "C" fn(*mut CCMPatch, *mut c_void),
        userdata: *mut c_void,
    );
    fn CM_ShutdownTerrain(terrainId: thandle_t);

    fn RE_RegisterShader(name: *const c_char) -> qhandle_t;
    fn R_GetShaderByHandle(handle: qhandle_t) -> *mut shader_t;
    fn R_CreateBlendedShader(a: qhandle_t, b: qhandle_t, c: qhandle_t, surfaceSprites: bool) -> qhandle_t;
    fn R_GetShaderByNum(num: c_int, world: *mut refdef_t) -> qhandle_t;
    fn R_AddDrawSurf(surface: *mut c_void, shader: *mut shader_t, num: c_int, trans: bool);

    fn Info_ValueForKey(info: *const c_char, key: *const c_char) -> *const c_char;
    fn Com_ParseTextFile(filename: *const c_char, parse: *mut CGenericParser2) -> bool;
    fn Com_ParseTextFileDestroy(parse: CGenericParser2);
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Clampi(min: c_int, max: c_int, value: c_int) -> c_int;
    fn Com_Clampf(min: f32, max: f32, value: f32) -> f32;

    fn Z_Malloc(size: usize, tag: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    fn VectorSet(v: *mut vec3_t, x: f32, y: f32, z: f32);
    fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    fn VectorAdd(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t);
    fn VectorScaleVectorAdd(a: *const vec3_t, b: *const vec3_t, c: f32, out: *mut vec3_t);
    fn VectorMA(a: *const vec3_t, b: f32, c: *const vec3_t, out: *mut vec3_t);
    fn VectorNormalize(v: *mut vec3_t) -> f32;
    fn VectorLengthSquared(v: *const vec3_t) -> f32;
    fn VectorLength(v: *const vec3_t) -> f32;
    fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;
    fn CrossProduct(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);

    fn Q_log2(v: c_int) -> c_int;
    fn atol(str: *const c_char) -> c_long;
    fn atof(str: *const c_char) -> f64;
    fn strlen(str: *const c_char) -> usize;
    fn stricmp(a: *const c_char, b: *const c_char) -> c_int;
    fn powf(x: f32, y: f32) -> f32;
    fn floorf(x: f32) -> f32;
    fn ceilf(x: f32) -> f32;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn qsort(
        base: *mut c_void,
        nmemb: usize,
        size: usize,
        compar: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int,
    );
    fn va(fmt: *const c_char, ...) -> *const c_char;

    #[cfg(feature = "_DEBUG")]
    fn OutputDebugString(str: *const c_char);
}

// Global variables
static mut TerrainFog: c_int = 0;
static mut TerrainDistanceCull: f32 = 0.0;

//
// Render the tree.
//
impl CTRPatch {
    pub fn RenderCorner(&mut self, mut corner: ivec5_t) {
        unsafe {
            if corner[3] < 0 || tess.registration != corner[4] {
                let vert: *mut CTerVert = self.mRenderMap.add(
                    (corner[1] as usize * (*self.owner).GetRealWidth() as usize) + corner[0] as usize
                );

                VectorCopy(&(*vert).coords, &mut tess.xyz[tess.numVertexes as usize]);
                VectorCopy(&(*vert).normal, &mut tess.normal[tess.numVertexes as usize]);

                *(tess.vertexColors[tess.numVertexes as usize].as_mut_ptr() as *mut u32) =
                    *((*vert).tint.as_ptr() as *const u32);
                *(tess.vertexAlphas[tess.numVertexes as usize].as_mut_ptr() as *mut c_int) = corner[2];

                tess.texCoords[tess.numVertexes as usize][0][0] = (*vert).tex[0]; //rwwRMG - reverse coords array from sof2
                tess.texCoords[tess.numVertexes as usize][0][1] = (*vert).tex[1];

                tess.indexes[tess.numIndexes as usize] = tess.numVertexes as u16;
                corner[3] = tess.numVertexes;
                corner[4] = tess.registration;
                tess.numIndexes += 1;
                tess.numVertexes += 1;
            } else {
                tess.indexes[tess.numIndexes as usize] = corner[3] as u16;
                tess.numIndexes += 1;
            }
        }
    }

    pub fn RecurseRender(&mut self, depth: c_int, left: ivec5_t, right: ivec5_t, apex: ivec5_t) {
        unsafe {
            // All non-leaf nodes have both children, so just check for one
            if depth >= 0 {
                let mut center: ivec5_t = [0; 5];
                let centerAlphas: *mut byte;
                let leftAlphas: *mut byte;
                let rightAlphas: *mut byte;

                // Work out the centre of the hypoteneuse
                center[0] = (left[0] + right[0]) >> 1;
                center[1] = (left[1] + right[1]) >> 1;

                // Work out the relevant texture coefficients at that point
                leftAlphas = &mut left[2] as *mut c_int as *mut byte;
                rightAlphas = &mut right[2] as *mut c_int as *mut byte;
                centerAlphas = &mut center[2] as *mut c_int as *mut byte;

                *centerAlphas.add(0) = ((*leftAlphas.add(0) as c_int + *rightAlphas.add(0) as c_int) >> 1) as byte;
                *centerAlphas.add(1) = ((*leftAlphas.add(1) as c_int + *rightAlphas.add(1) as c_int) >> 1) as byte;
                *centerAlphas.add(2) = ((*leftAlphas.add(2) as c_int + *rightAlphas.add(2) as c_int) >> 1) as byte;
                *centerAlphas.add(3) = ((*leftAlphas.add(3) as c_int + *rightAlphas.add(3) as c_int) >> 1) as byte;

                // Make sure the vert index and tesselation registration are not set
                center[3] = -1;
                center[4] = 0;

                if apex[0] == left[0] && apex[0] == center[0] {
                    // depth = 0 assignment doesn't affect the local, just let it fall through with depth - 1
                }

                self.RecurseRender(depth - 1, apex, left, center);
                self.RecurseRender(depth - 1, right, apex, center);
            } else {
                if left[0] == right[0] && left[0] == apex[0] {
                    return;
                }
                if left[1] == right[1] && left[1] == apex[1] {
                    return;
                }
                // A leaf node!  Output a triangle to be rendered.
                RB_CheckOverflow(4, 4);

                //		assert(left[0] != right[0] || left[1] != right[1]);
                //		assert(left[0] != apex[0] || left[1] != apex[1]);

                self.RenderCorner(left);
                self.RenderCorner(right);
                self.RenderCorner(apex);
            }
        }
    }
}

//
// Render the mesh.
//
// The order of triangles is critical to the subdivision working

impl CTRPatch {
    pub fn Render(&mut self, Part: c_int) {
        unsafe {
            let mut TL: ivec5_t = [0; 5];
            let mut TR: ivec5_t = [0; 5];
            let mut BL: ivec5_t = [0; 5];
            let mut BR: ivec5_t = [0; 5];

            // VectorSet5 implementation inline
            TL[0] = 0;
            TL[1] = 0;
            TL[2] = TEXTURE_ALPHA_TL;
            TL[3] = -1;
            TL[4] = 0;

            let terxels = (*self.owner).GetTerxels();
            TR[0] = terxels;
            TR[1] = 0;
            TR[2] = TEXTURE_ALPHA_TR;
            TR[3] = -1;
            TR[4] = 0;

            BL[0] = 0;
            BL[1] = terxels;
            BL[2] = TEXTURE_ALPHA_BL;
            BL[3] = -1;
            BL[4] = 0;

            BR[0] = terxels;
            BR[1] = terxels;
            BR[2] = TEXTURE_ALPHA_BR;
            BR[3] = -1;
            BR[4] = 0;

            if (Part & PI_TOP) != 0 && !self.mTLShader.is_null() {
                /*		float		d;

                d = DotProduct (backEnd.refdef.vieworg, mNormal[0]) - mDistance[0];

                if (d <= 0.0)*/
                {
                    self.RecurseRender((*r_terrainTessellate).integer, BL, TR, TL);
                }
            }

            if (Part & PI_BOTTOM) != 0 && !self.mBRShader.is_null() {
                /*		float		d;

                d = DotProduct (backEnd.refdef.vieworg, mNormal[1]) - mDistance[1];

                if (d >= 0.0)*/
                {
                    self.RecurseRender((*r_terrainTessellate).integer, TR, BL, BR);
                }
            }
        }
    }
}

//
// At this point the patch is visible and at least part of it is below water level
//
impl CTRPatch {
    pub fn RenderWaterVert(&mut self, x: c_int, y: c_int) -> c_int {
        unsafe {
            let vert: *mut CTerVert = self.mRenderMap.add(
                (x as usize + (y as usize * (*self.owner).GetRealWidth() as usize)) as usize
            );

            if (*vert).tessRegistration == tess.registration {
                return (*vert).tessIndex;
            }
            tess.xyz[tess.numVertexes as usize][0] = (*vert).coords[0];
            tess.xyz[tess.numVertexes as usize][1] = (*vert).coords[1];
            tess.xyz[tess.numVertexes as usize][2] = (*self.owner).GetWaterHeight();

            *(tess.vertexColors[tess.numVertexes as usize].as_mut_ptr() as *mut u32) = 0xffffffff;

            tess.texCoords[tess.numVertexes as usize][0][0] = (*vert).tex[0]; //rwwRMG - reverse coords from sof2mp
            tess.texCoords[tess.numVertexes as usize][0][1] = (*vert).tex[1];

            (*vert).tessIndex = tess.numVertexes;
            (*vert).tessRegistration = tess.registration;

            tess.numVertexes += 1;
            return (*vert).tessIndex;
        }
    }

    pub fn RenderWater(&mut self) {
        unsafe {
            RB_CheckOverflow(4, 6);

            // Get the neighbouring patches
            let TL: c_int = self.RenderWaterVert(0, 0);
            let terxels = (*self.owner).GetTerxels();
            let TR: c_int = self.RenderWaterVert(terxels, 0);
            let BL: c_int = self.RenderWaterVert(0, terxels);
            let BR: c_int = self.RenderWaterVert(terxels, terxels);

            // TL
            tess.indexes[tess.numIndexes as usize] = BL as u16;
            tess.numIndexes += 1;
            tess.indexes[tess.numIndexes as usize] = TR as u16;
            tess.numIndexes += 1;
            tess.indexes[tess.numIndexes as usize] = TL as u16;
            tess.numIndexes += 1;

            // BR
            tess.indexes[tess.numIndexes as usize] = TR as u16;
            tess.numIndexes += 1;
            tess.indexes[tess.numIndexes as usize] = BL as u16;
            tess.numIndexes += 1;
            tess.indexes[tess.numIndexes as usize] = BR as u16;
            tess.numIndexes += 1;
        }
    }

    pub fn HasWater(&self) -> bool {
        unsafe {
            (*self.owner).SetRealWaterHeight((*self.owner).GetBaseWaterHeight() + (*r_terrainWaterOffset).integer);
            return (*common).GetMins()[2] < (*self.owner).GetWaterHeight();
        }
    }
}

extern "C" {
    fn CM_CullWorldBox(frustum: *const cplane_t, bounds: *const vec3pair_t) -> bool; //rwwRMG - added (cm_trace.cpp)
}

impl CTRPatch {
    pub fn SetVisibility(&mut self, visCheck: bool) {
        unsafe {
            if visCheck {
                if DistanceSquared(&self.mCenter, &backEnd.refdef.vieworg) > TerrainDistanceCull {
                    self.misVisible = false;
                } else {
                    // Set the visibility of the patch
                    self.misVisible = !CM_CullWorldBox(&backEnd.viewParms.frustum[0], self.GetBounds());
                }
            } else {
                self.misVisible = true;
            }
        }
    }
}

/*
impl CTRPatch {
    pub fn CalcNormal(&mut self) {
        let mut vert1: *mut CTerVert;
        let mut vert2: *mut CTerVert;
        let mut vert3: *mut CTerVert;
        let mut TL: ivec5_t = [0; 5];
        let mut TR: ivec5_t = [0; 5];
        let mut BL: ivec5_t = [0; 5];
        let mut BR: ivec5_t = [0; 5];
        let mut v1: vec3_t = [0.0; 3];
        let mut v2: vec3_t = [0.0; 3];

        TL[0] = 0;
        TL[1] = 0;
        TL[2] = TEXTURE_ALPHA_TL;
        TL[3] = -1;
        TL[4] = 0;
        TR[0] = (*(*self.owner).GetTerxels());
        TR[1] = 0;
        TR[2] = TEXTURE_ALPHA_TR;
        TR[3] = -1;
        TR[4] = 0;
        BL[0] = 0;
        BL[1] = (*(*self.owner).GetTerxels());
        BL[2] = TEXTURE_ALPHA_BL;
        BL[3] = -1;
        BL[4] = 0;
        BR[0] = (*(*self.owner).GetTerxels());
        BR[1] = (*(*self.owner).GetTerxels());
        BR[2] = TEXTURE_ALPHA_BR;
        BR[3] = -1;
        BR[4] = 0;

        vert1 = self.mRenderMap.add((BL[1] as usize * (*(*self.owner).GetRealWidth()) as usize) + BL[0] as usize);
        vert2 = self.mRenderMap.add((TR[1] as usize * (*(*self.owner).GetRealWidth()) as usize) + TR[0] as usize);
        vert3 = self.mRenderMap.add((TL[1] as usize * (*(*self.owner).GetRealWidth()) as usize) + TL[0] as usize);
        VectorSubtract(&(*vert2).coords, &(*vert1).coords, &mut v1);
        VectorSubtract(&(*vert3).coords, &(*vert1).coords, &mut v2);
        CrossProduct(&v1, &v2, &mut self.mNormal[0]);
        VectorNormalize(&mut self.mNormal[0]);
        self.mDistance[0] = DotProduct(&(*vert1).coords, &self.mNormal[0]);

        vert1 = self.mRenderMap.add((BL[1] as usize * (*(*self.owner).GetRealWidth()) as usize) + BL[0] as usize);
        vert2 = self.mRenderMap.add((TR[1] as usize * (*(*self.owner).GetRealWidth()) as usize) + TR[0] as usize);
        vert3 = self.mRenderMap.add((BR[1] as usize * (*(*self.owner).GetRealWidth()) as usize) + BR[0] as usize);
        VectorSubtract(&(*vert2).coords, &(*vert1).coords, &mut v1);
        VectorSubtract(&(*vert3).coords, &(*vert1).coords, &mut v2);
        CrossProduct(&v1, &v2, &mut self.mNormal[1]);
        VectorNormalize(&mut self.mNormal[1]);
        self.mDistance[1] = DotProduct(&(*vert1).coords, &self.mNormal[1]);
    }
}
*/

//
// Reset all patches, recompute variance if needed
//
impl CTRLandScape {
    pub fn Reset(&mut self, visCheck: bool) {
        unsafe {
            let mut x: c_int;
            let mut y: c_int;
            let mut patch: *mut CTRPatch;

            TerrainDistanceCull = tr.distanceCull + self.mPatchSize;
            TerrainDistanceCull *= TerrainDistanceCull;

            // Go through the patches performing resets, compute variances, and linking.
            y = self.mPatchMiny;
            while y < self.mPatchMaxy {
                x = self.mPatchMinx;
                while x < self.mPatchMaxx {
                    patch = self.GetPatch(x, y);
                    (*patch).SetVisibility(visCheck);
                    x += 1;
                }
                y += 1;
            }
        }
    }
}

//
// Render each patch of the landscape & adjust the frame variance.
//

impl CTRLandScape {
    pub fn Render(&mut self) {
        unsafe {
            let mut x: c_int;
            let mut y: c_int;
            let mut patch: *mut CTRPatch;
            let mut current: *mut TPatchInfo;
            let mut i: c_int;

            // Render all the visible patches
            current = self.mSortedPatches;
            i = 0;
            while i < self.mSortedCount {
                if (*(*current).mPatch).isVisible() {
                    if tess.shader != (*current).mShader {
                        RB_EndSurface();
                        RB_BeginSurface((*current).mShader, TerrainFog);
                    }
                    (*(*current).mPatch).Render((*current).mPart);
                }
                current = current.add(1);
                i += 1;
            }
            RB_EndSurface();

            // Render all the water for visible patches
            // Done as a separate iteration to reduce the number of tesses created
            if self.mWaterShader != null_mut() && self.mWaterShader != tr.defaultShader {
                RB_BeginSurface(self.mWaterShader, (*tr.world).globalFog);

                y = self.mPatchMiny;
                while y < self.mPatchMaxy {
                    x = self.mPatchMinx;
                    while x < self.mPatchMaxx {
                        patch = self.GetPatch(x, y);
                        if (*patch).isVisible() && (*patch).HasWater() {
                            (*patch).RenderWater();
                        }
                        x += 1;
                    }
                    y += 1;
                }
                RB_EndSurface();
            }
        }
    }
}

impl CTRLandScape {
    pub fn CalculateRegion(&mut self) {
        unsafe {
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            let mut size: vec3_t = [0.0; 3];
            let mut offset: vec3_t = [0.0; 3];

            #[cfg(feature = "_DEBUG")]
            {
                self.mCycleCount += 1;
            }
            VectorCopy(self.GetPatchSize(), &mut size);
            VectorCopy(self.GetMins(), &mut offset);

            mins[0] = backEnd.refdef.vieworg[0] - tr.distanceCull - (size[0] * 2.0f32) - offset[0];
            mins[1] = backEnd.refdef.vieworg[1] - tr.distanceCull - (size[1] * 2.0f32) - offset[1];

            maxs[0] = backEnd.refdef.vieworg[0] + tr.distanceCull + (size[0] * 2.0f32) - offset[0];
            maxs[1] = backEnd.refdef.vieworg[1] + tr.distanceCull + (size[1] * 2.0f32) - offset[1];

            self.mPatchMinx = Com_Clampi(0, self.GetBlockWidth(), floorf(mins[0] / size[0]) as c_int);
            self.mPatchMaxx = Com_Clampi(0, self.GetBlockWidth(), ceilf(maxs[0] / size[0]) as c_int);

            self.mPatchMiny = Com_Clampi(0, self.GetBlockHeight(), floorf(mins[1] / size[1]) as c_int);
            self.mPatchMaxy = Com_Clampi(0, self.GetBlockHeight(), ceilf(maxs[1] / size[1]) as c_int);
        }
    }
}

impl CTRLandScape {
    pub fn CalculateRealCoords(&mut self) {
        unsafe {
            let mut x: c_int;
            let mut y: c_int;

            // Work out the real world coordinates of each heightmap entry
            y = 0;
            while y < self.GetRealHeight() {
                x = 0;
                while x < self.GetRealWidth() {
                    let mut icoords: [c_int; 3] = [0; 3];
                    let mut offset: c_int;

                    offset = (y * self.GetRealWidth()) + x;

                    icoords[0] = x;
                    icoords[1] = y;
                    icoords[2] = (*self.mRenderMap.add(offset as usize)).height;
                    VectorScaleVectorAdd(
                        self.GetMins(),
                        &icoords as *const _ as *const vec3_t,
                        self.GetTerxelSize()[0],
                        &mut (*self.mRenderMap.add(offset as usize)).coords,
                    );
                    x += 1;
                }
                y += 1;
            }
        }
    }
}

impl CTRLandScape {
    pub fn CalculateNormals(&mut self) {
        unsafe {
            let mut x: c_int;
            let mut y: c_int;
            let mut offset: c_int = 0;

            // Work out the normals for every face
            y = 0;
            while y < self.GetHeight() {
                x = 0;
                while x < self.GetWidth() {
                    let mut vcenter: vec3_t = [0.0; 3];
                    let mut vleft: vec3_t = [0.0; 3];

                    offset = (y * self.GetRealWidth()) + x;

                    VectorSubtract(
                        &(*self.mRenderMap.add(offset as usize)).coords,
                        &(*self.mRenderMap.add((offset + 1) as usize)).coords,
                        &mut vcenter,
                    );
                    VectorSubtract(
                        &(*self.mRenderMap.add(offset as usize)).coords,
                        &(*self.mRenderMap.add((offset + self.GetRealWidth()) as usize)).coords,
                        &mut vleft,
                    );

                    CrossProduct(&vcenter, &vleft, &mut (*self.mRenderMap.add(offset as usize)).normal);
                    VectorNormalize(&mut (*self.mRenderMap.add(offset as usize)).normal);
                    x += 1;
                }
                // Duplicate right edge condition
                VectorCopy(
                    &(*self.mRenderMap.add(offset as usize)).normal,
                    &mut (*self.mRenderMap.add((offset + 1) as usize)).normal,
                );
                y += 1;
            }
            // Duplicate bottom line
            offset = self.GetHeight() * self.GetRealWidth();
            x = 0;
            while x < self.GetRealWidth() {
                VectorCopy(
                    &(*self.mRenderMap.add((offset - self.GetRealWidth() + x) as usize)).normal,
                    &mut (*self.mRenderMap.add((offset + x) as usize)).normal,
                );
                x += 1;
            }
        }
    }
}

impl CTRLandScape {
    pub fn CalculateLighting(&mut self) {
        unsafe {
            let mut x: c_int;
            let mut y: c_int;
            let mut offset: c_int = 0;

            // Work out the vertex normal (average of every attached face normal) and apply to the direction of the light
            y = 0;
            while y < self.GetHeight() {
                x = 0;
                while x < self.GetWidth() {
                    let mut ambient: vec3_t = [0.0; 3];
                    let mut directed: vec3_t = [0.0; 3];
                    let mut direction: vec3_t = [0.0; 3];
                    let mut total: vec3_t = [0.0; 3];
                    let mut tint: vec3_t = [0.0; 3];
                    let mut dp: f32;

                    offset = (y * self.GetRealWidth()) + x;

                    // Work out average normal
                    let rm_xy = self.GetRenderMap(x, y);
                    let rm_x1y = self.GetRenderMap(x + 1, y);
                    let rm_x1y1 = self.GetRenderMap(x + 1, y + 1);
                    let rm_xy1 = self.GetRenderMap(x, y + 1);

                    VectorCopy(&(*rm_xy).coords, &mut total);
                    VectorAdd(&total, &(*rm_x1y).coords, &mut total);
                    VectorAdd(&total, &(*rm_x1y1).coords, &mut total);
                    VectorAdd(&total, &(*rm_xy1).coords, &mut total);
                    VectorNormalize(&mut total);

                    if !R_LightForPoint(
                        &(*self.mRenderMap.add(offset as usize)).coords,
                        &mut ambient,
                        &mut directed,
                        &mut direction,
                    ) {
                        (*self.mRenderMap.add(offset as usize)).tint[0] =
                            ((255 >> tr.overbrightBits) as u8);
                        (*self.mRenderMap.add(offset as usize)).tint[1] =
                            ((255 >> tr.overbrightBits) as u8);
                        (*self.mRenderMap.add(offset as usize)).tint[2] =
                            ((255 >> tr.overbrightBits) as u8);
                        (*self.mRenderMap.add(offset as usize)).tint[3] = 255;
                        x += 1;
                        continue;
                    }

                    if (*self.mRenderMap.add(offset as usize)).coords[2] < (*common).GetBaseWaterHeight() {
                        VectorScale(&ambient, 0.75f32, &mut ambient);
                    }

                    // Both normalised, so -1.0 < dp < 1.0
                    dp = Com_Clampf(0.0f32, 1.0f32, DotProduct(&direction, &total));
                    dp = powf(dp, 3.0);
                    VectorScale(&ambient, (1.0 - dp) * 0.5, &mut ambient);
                    VectorMA(&ambient, dp, &directed, &mut tint);

                    (*self.mRenderMap.add(offset as usize)).tint[0] =
                        ((Com_Clampf(0.0f32, 255.0f32, tint[0]) as c_int >> tr.overbrightBits)) as u8;
                    (*self.mRenderMap.add(offset as usize)).tint[1] =
                        ((Com_Clampf(0.0f32, 255.0f32, tint[1]) as c_int >> tr.overbrightBits)) as u8;
                    (*self.mRenderMap.add(offset as usize)).tint[2] =
                        ((Com_Clampf(0.0f32, 255.0f32, tint[2]) as c_int >> tr.overbrightBits)) as u8;
                    (*self.mRenderMap.add(offset as usize)).tint[3] = 0xff;

                    /*
                    (*self.mRenderMap.add(offset as usize)).tint[0] += tr.identityLight * 32;
                    (*self.mRenderMap.add(offset as usize)).tint[1] += tr.identityLight * 32;
                    (*self.mRenderMap.add(offset as usize)).tint[2] += tr.identityLight * 32;
                    */
                    x += 1;
                }
                (*self.mRenderMap.add((offset + 1) as usize)).tint[0] = (*self.mRenderMap.add(offset as usize)).tint[0];
                (*self.mRenderMap.add((offset + 1) as usize)).tint[1] = (*self.mRenderMap.add(offset as usize)).tint[1];
                (*self.mRenderMap.add((offset + 1) as usize)).tint[2] = (*self.mRenderMap.add(offset as usize)).tint[2];
                (*self.mRenderMap.add((offset + 1) as usize)).tint[3] = 0xff;
                y += 1;
            }
            // Duplicate bottom line
            offset = self.GetHeight() * self.GetRealWidth();
            x = 0;
            while x < self.GetRealWidth() {
                (*self.mRenderMap.add((offset + x) as usize)).tint[0] =
                    (*self.mRenderMap.add((offset - self.GetRealWidth() + x) as usize)).tint[0];
                (*self.mRenderMap.add((offset + x) as usize)).tint[1] =
                    (*self.mRenderMap.add((offset - self.GetRealWidth() + x) as usize)).tint[1];
                (*self.mRenderMap.add((offset + x) as usize)).tint[2] =
                    (*self.mRenderMap.add((offset - self.GetRealWidth() + x) as usize)).tint[2];
                (*self.mRenderMap.add((offset + x) as usize)).tint[3] = 0xff;
                x += 1;
            }
        }
    }
}

impl CTRLandScape {
    pub fn CalculateTextureCoords(&mut self) {
        unsafe {
            let mut x: c_int;
            let mut y: c_int;

            y = 0;
            while y < self.GetRealHeight() {
                x = 0;
                while x < self.GetRealWidth() {
                    let offset: c_int = (y * self.GetRealWidth()) + x;

                    (*self.mRenderMap.add(offset as usize)).tex[0] = x as f32 * self.mTextureScale * self.GetTerxelSize()[0];
                    (*self.mRenderMap.add(offset as usize)).tex[1] = y as f32 * self.mTextureScale * self.GetTerxelSize()[1];
                    x += 1;
                }
                y += 1;
            }
        }
    }
}

impl CTRLandScape {
    pub fn SetShaders(&mut self, height: c_int, shader: qhandle_t) {
        let mut i: c_int = height;

        while shader != 0 && i < HEIGHT_RESOLUTION as c_int {
            if self.mHeightDetails[i as usize].GetShader() == 0 {
                self.mHeightDetails[i as usize].SetShader(shader);
            }
            i += 1;
        }
    }
}

impl CTRLandScape {
    pub fn LoadTerrainDef(&mut self, td: *const c_char) {
        #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        unsafe {
            let mut terrainDef: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            let mut parse: CGenericParser2;
            let mut basegroup: *mut CGPGroup;
            let mut classes: *mut CGPGroup;
            let mut items: *mut CGPGroup;

            Com_sprintf(
                terrainDef.as_mut_ptr(),
                MAX_QPATH as c_int,
                b"ext_data/RMG/%s.terrain\0".as_ptr() as *const c_char,
                td,
            );
            Com_Printf(b"R_Terrain: Loading and parsing terrainDef %s.....\n\0".as_ptr() as *const c_char, td);

            self.mWaterShader = null_mut();
            self.mFlatShader = 0;

            if !Com_ParseTextFile(terrainDef.as_ptr(), &mut parse) {
                Com_sprintf(
                    terrainDef.as_mut_ptr(),
                    MAX_QPATH as c_int,
                    b"ext_data/arioche/%s.terrain\0".as_ptr() as *const c_char,
                    td,
                );
                if !Com_ParseTextFile(terrainDef.as_ptr(), &mut parse) {
                    Com_Printf(b"Could not open %s\n\0".as_ptr() as *const c_char, terrainDef.as_ptr());
                    return;
                }
            }
            // The whole file....
            basegroup = (*(&parse as *const _ as *const CGPGroup)).GetBaseParseGroup();

            // The root { } struct
            classes = (*basegroup).GetSubGroups();
            while !classes.is_null() {
                items = (*classes).GetSubGroups();
                while !items.is_null() {
                    let type_: *const c_char = (*items).GetName();

                    if stricmp(type_, b"altitudetexture\0".as_ptr() as *const c_char) == 0 {
                        let height: c_int;
                        let shaderName: *const c_char;
                        let shader: qhandle_t;

                        // Height must exist - the rest are optional
                        height = atol((*items).FindPairValue(b"height\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char)) as c_int;

                        // Shader for this height
                        shaderName = (*items).FindPairValue(b"shader\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
                        if strlen(shaderName) > 0 {
                            shader = RE_RegisterShader(shaderName);
                            if shader != 0 {
                                self.SetShaders(height, shader);
                            }
                        }
                    } else if stricmp(type_, b"water\0".as_ptr() as *const c_char) == 0 {
                        self.mWaterShader = R_GetShaderByHandle(RE_RegisterShader(
                            (*items).FindPairValue(b"shader\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char),
                        ));
                    } else if stricmp(type_, b"flattexture\0".as_ptr() as *const c_char) == 0 {
                        self.mFlatShader = RE_RegisterShader(
                            (*items).FindPairValue(b"shader\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char),
                        );
                    }

                    items = (*items).GetNext() as *mut CGPGroup;
                }
                classes = (*classes).GetNext() as *mut CGPGroup;
            }

            Com_ParseTextFileDestroy(parse);
        }
        #[cfg(feature = "PRE_RELEASE_DEMO")]
        {
            // PRE_RELEASE_DEMO: empty implementation
        }
    }
}

impl CTRLandScape {
    pub fn GetBlendedShader(&self, a: qhandle_t, b: qhandle_t, c: qhandle_t, surfaceSprites: bool) -> qhandle_t {
        // Special case single pass shader
        if (a == b) && (a == c) {
            return a;
        }

        unsafe {
            let blended: qhandle_t = R_CreateBlendedShader(a, b, c, surfaceSprites);
            return blended;
        }
    }
}

unsafe extern "C" fn ComparePatchInfo(arg1: *const c_void, arg2: *const c_void) -> c_int {
    let arg1 = arg1 as *const TPatchInfo;
    let arg2 = arg2 as *const TPatchInfo;
    let s1: *mut shader_t;
    let s2: *mut shader_t;

    if ((*arg1).mPart & PI_TOP) != 0 {
        s1 = (*(*arg1).mPatch).GetTLShader();
    } else {
        s1 = (*(*arg1).mPatch).GetBRShader();
    }

    if ((*arg2).mPart & PI_TOP) != 0 {
        s2 = (*(*arg2).mPatch).GetTLShader();
    } else {
        s2 = (*(*arg2).mPatch).GetBRShader();
    }

    if (s1 as *const c_void) < (s2 as *const c_void) {
        return -1;
    } else if (s1 as *const c_void) > (s2 as *const c_void) {
        return 1;
    }

    return 0;
}

impl CTRLandScape {
    pub fn CalculateShaders(&mut self) {
        #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        unsafe {
            let mut x: c_int;
            let mut y: c_int;
            let width: c_int;
            let height: c_int;
            let mut offset: c_int;
            let mut patch: *mut CTRPatch;
            let shaders: *mut qhandle_t;
            let mut current: *mut TPatchInfo = self.mSortedPatches;

            width = self.GetWidth() / (*common).GetTerxels();
            height = self.GetHeight() / (*common).GetTerxels();

            shaders = Z_Malloc(((width + 1) * (height + 1)) as usize * core::mem::size_of::<qhandle_t>(), TAG_R_TERRAIN) as *mut qhandle_t;

            // On the first pass determine all of the shaders for the entire
            // terrain assuming no flat ground
            offset = 0;
            y = 0;
            while y < height + 1 {
                if y <= height {
                    offset = (*common).GetTerxels() * y * self.GetRealWidth();
                } else {
                    offset = (*common).GetTerxels() * (y - 1) * self.GetRealWidth();
                    offset += self.GetRealWidth();
                }

                x = 0;
                while x < width + 1 {
                    // Save the shader
                    *shaders.add((y * width + x) as usize) =
                        self.GetHeightDetail((*self.mRenderMap.add(offset as usize)).height).GetShader();
                    offset += (*common).GetTerxels();
                    x += 1;
                }
                y += 1;
            }

            // On the second pass determine flat ground and replace the shader
            // at that point with the flat ground shader
            if self.mFlatShader != 0 {
                y = 1;
                while y < height {
                    x = 1;
                    while x < width {
                        let mut offset2: c_int;
                        let mut xx: c_int;
                        let mut yy: c_int;
                        let flattenMap: *mut byte = (*common).GetFlattenMap();
                        let mut flat: bool = false;

                        offset2 = (x) * (*common).GetTerxels();
                        offset2 += (y) * (*common).GetTerxels() * self.GetRealWidth();

                        offset2 -= self.GetRealWidth();
                        offset2 -= 1;

                        yy = 0;
                        while yy < 3 && !flat {
                            xx = 0;
                            while xx < 3 && !flat {
                                if (*(flattenMap.add((offset2 + xx) as usize)) & 0x80) != 0 {
                                    flat = true;
                                    break;
                                }
                                xx += 1;
                            }

                            offset2 += self.GetRealWidth();
                            yy += 1;
                        }

                        /*
                        // Calculate the height map offset
                        offset2  = x * (*common).GetTerxels ( );
                        offset2 += (y * (*common).GetTerxels ( ) * self.GetRealWidth());

                        // Calculate the offsets around this particular shader location
                        offsets[INDEX_TL] = offset2 - 1 - self.GetRealWidth();
                        offsets[INDEX_TR] = offsets[INDEX_TL] + 1;
                        offsets[INDEX_BL] = offsets[INDEX_TL] + self.GetRealWidth();
                        offsets[INDEX_BR] = offsets[INDEX_BL] + 1;

                        // If not equal to the top left one then skip
                        if ( (*self.mRenderMap.add(offset2 as usize)).height != (*self.mRenderMap.add(offsets[INDEX_TL as usize] as usize)).height )
                        {
                            continue;
                        }

                        // If not equal to the top right one then skip
                        if ( (*self.mRenderMap.add(offset2 as usize)).height != (*self.mRenderMap.add(offsets[INDEX_TR as usize] as usize)).height )
                        {
                            continue;
                        }

                        // If not equal to the bottom left one then skip
                        if ( (*self.mRenderMap.add(offset2 as usize)).height != (*self.mRenderMap.add(offsets[INDEX_BL as usize] as usize)).height )
                        {
                            continue;
                        }

                        // If not equal to the bottom right one then skip
                        if ( (*self.mRenderMap.add(offset2 as usize)).height != (*self.mRenderMap.add(offsets[INDEX_BR as usize] as usize)).height )
                        {
                            continue;
                        }
                        */

                        // This shader is now a flat shader
                        if flat {
                            *shaders.add((y * width + x) as usize) = self.mFlatShader;
                        }

                        #[cfg(feature = "_DEBUG")]
                        {
                            OutputDebugString(
                                va(
                                    b"Flat Area:  %f %f\n\0".as_ptr() as *const c_char,
                                    self.GetMins()[0] + (self.GetMaxs()[0] - self.GetMins()[0]) / width as f32 * x as f32,
                                    self.GetMins()[1] + (self.GetMaxs()[1] - self.GetMins()[1]) / height as f32 * y as f32,
                                ) as *const c_char,
                            );
                        }
                        x += 1;
                    }
                    y += 1;
                }
            }

            // Now that the shaders have been determined, set them for each patch
            patch = self.mTRPatches;
            self.mSortedCount = 0;
            y = 0;
            while y < height {
                x = 0;
                while x < width {
                    let mut handles_mut: [qhandle_t; 4] = [0; 4];
                    let surfaceSprites: bool = true;

                    handles_mut[INDEX_TL as usize] = *shaders.add((x + y * width) as usize);
                    handles_mut[INDEX_TR as usize] = *shaders.add(((x + 1 + y * width)) as usize);
                    handles_mut[INDEX_BL as usize] = *shaders.add(((x + (y + 1) * width)) as usize);
                    handles_mut[INDEX_BR as usize] = *shaders.add(((x + 1 + (y + 1) * width)) as usize);

                    let mut surfaceSprites_mut = surfaceSprites;
                    if handles_mut[INDEX_TL as usize] == self.mFlatShader
                        || handles_mut[INDEX_TR as usize] == self.mFlatShader
                        || handles_mut[INDEX_BL as usize] == self.mFlatShader
                        || handles_mut[INDEX_BR as usize] == self.mFlatShader
                    {
                        surfaceSprites_mut = false;
                    }

                    (*patch).SetTLShader(self.GetBlendedShader(
                        handles_mut[INDEX_TR as usize],
                        handles_mut[INDEX_BL as usize],
                        handles_mut[INDEX_TL as usize],
                        surfaceSprites_mut,
                    ));
                    (*current).mPatch = patch;
                    (*current).mShader = (*patch).GetTLShader();
                    (*current).mPart = PI_TOP;

                    (*patch).SetBRShader(self.GetBlendedShader(
                        handles_mut[INDEX_TR as usize],
                        handles_mut[INDEX_BL as usize],
                        handles_mut[INDEX_BR as usize],
                        surfaceSprites_mut,
                    ));
                    if (*patch).GetBRShader() == (*current).mShader {
                        (*current).mPart |= PI_BOTTOM;
                    } else {
                        self.mSortedCount += 1;
                        current = current.add(1);

                        (*current).mPatch = patch;
                        (*current).mShader = (*patch).GetBRShader();
                        (*current).mPart = PI_BOTTOM;
                    }
                    self.mSortedCount += 1;
                    current = current.add(1);
                    patch = patch.add(1);
                    x += 1;
                }
                y += 1;
            }

            // Cleanup our temporary array
            Z_Free(shaders as *mut c_void);

            qsort(
                self.mSortedPatches as *mut c_void,
                self.mSortedCount as usize,
                core::mem::size_of::<TPatchInfo>(),
                ComparePatchInfo,
            );
        }
        #[cfg(feature = "PRE_RELEASE_DEMO")]
        {
            // PRE_RELEASE_DEMO: empty implementation
        }
    }
}

impl CTRPatch {
    pub fn SetRenderMap(&mut self, x: c_int, y: c_int) {
        unsafe {
            self.mRenderMap = self.localowner.as_mut().unwrap().GetRenderMap(x, y);
        }
    }
}

unsafe extern "C" fn InitRendererPatches(patch: *mut CCMPatch, userdata: *mut c_void) {
    let tx: c_int;
    let ty: c_int;
    let bx: c_int;
    let by: c_int;
    let localpatch: *mut CTRPatch;
    let owner: *mut CCMLandScape;
    let localowner: *mut CTRLandScape;

    // Set owning landscape
    localowner = userdata as *mut CTRLandScape;
    owner = (*localowner).GetCommon() as *mut CCMLandScape;

    // Get TRPatch pointer
    tx = (*patch).GetHeightMapX();
    ty = (*patch).GetHeightMapY();
    bx = tx / (*owner).GetTerxels();
    by = ty / (*owner).GetTerxels();

    localpatch = (*localowner).GetPatch(bx, by);
    (*localpatch).Clear();

    (*localpatch).SetCommon(patch);
    (*localpatch).SetOwner(owner);
    (*localpatch).SetLocalOwner(localowner);
    (*localpatch).SetRenderMap(tx, ty);
    (*localpatch).SetCenter();
    //	(*localpatch).CalcNormal();
}

impl CTRLandScape {
    pub fn CopyHeightMap(&mut self) {
        unsafe {
            let common: *const CCMLandScape = self.GetCommon();
            let mut heightMap: *const byte = (*common).GetHeightMap();
            let mut renderMap: *mut CTerVert = self.mRenderMap;
            let mut i: c_int;

            i = 0;
            while i < (*common).GetRealArea() {
                (*renderMap).height = *heightMap as c_int;
                renderMap = renderMap.add(1);
                heightMap = heightMap.add(1);
                i += 1;
            }
        }
    }
}

impl CTRLandScape {
    pub fn drop(&mut self) {
        unsafe {
            if !self.mTRPatches.is_null() {
                Z_Free(self.mTRPatches as *mut c_void);
                self.mTRPatches = null_mut();
            }
            if !self.mSortedPatches.is_null() {
                Z_Free(self.mSortedPatches as *mut c_void);
                self.mSortedPatches = null_mut();
            }
            if !self.mRenderMap.is_null() {
                Z_Free(self.mRenderMap as *mut c_void);
                self.mRenderMap = null_mut();
            }
        }
    }
}

extern "C" {
    fn CM_RegisterTerrain(config: *const c_char, server: bool) -> *mut CCMLandScape; //cm_load.cpp
}

impl CTRLandScape {
    pub fn new(configstring: *const c_char) -> Self {
        #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        unsafe {
            let mut shaderNum: c_int;
            let common: *const CCMLandScape;
            let mut landscape = CTRLandScape {
                common: null_mut(),
                mTRPatches: null_mut(),
                mSortedPatches: null_mut(),
                mPatchMinx: 0,
                mPatchMaxx: 0,
                mPatchMiny: 0,
                mPatchMaxy: 0,
                mMaxNode: 0,
                mSortedCount: 0,
                mPatchSize: 0.0,
                mShader: null_mut(),
                mRenderMap: null_mut(),
                mTextureScale: 0.0,
                mScalarSize: 0.0,
                mWaterShader: null_mut(),
                mFlatShader: 0,
                mHeightDetails: [CTRHeightDetails::new(); HEIGHT_RESOLUTION],
                mCycleCount: 0,
            };

            memset(&mut landscape as *mut _ as *mut c_void, 0, core::mem::size_of::<CTRLandScape>());
            let _ = 0; // suppress memset return value

            // Sets up the common aspects of the terrain
            common = CM_RegisterTerrain(configstring, false);
            landscape.SetCommon(common);

            tr.landScape.landscape = &mut landscape as *mut _;

            landscape.mTextureScale =
                (atof(Info_ValueForKey(configstring, b"texturescale\0".as_ptr() as *const c_char)) as f32)
                    / (*common).GetTerxels() as f32;
            landscape.LoadTerrainDef(Info_ValueForKey(
                configstring,
                b"terrainDef\0".as_ptr() as *const c_char,
            ));

            // To normalise the variance value to a reasonable number
            landscape.mScalarSize = VectorLengthSquared((*common).GetSize());

            // Calculate and set variance depth
            landscape.mMaxNode = (Q_log2((*common).GetTerxels()) << 1) - 1;

            // Allocate space for the renderer specific data
            landscape.mRenderMap = Z_Malloc(
                core::mem::size_of::<CTerVert>() * (*common).GetRealArea() as usize,
                TAG_R_TERRAIN,
            ) as *mut CTerVert;

            // Copy byte heightmap to rendermap to speed up calcs
            landscape.CopyHeightMap();

            // Calculate the real world location for each heightmap entry
            landscape.CalculateRealCoords();

            // Calculate the normal of each terxel
            landscape.CalculateNormals();

            // Calculate modulation values for the heightmap
            landscape.CalculateLighting();

            // Calculate texture coords (not projected - real)
            landscape.CalculateTextureCoords();

            Com_Printf(b"R_Terrain: Creating renderer patches.....\n\0".as_ptr() as *const c_char);
            // Initialise all terrain patches
            landscape.mTRPatches = Z_Malloc(
                core::mem::size_of::<CTRPatch>() * (*common).GetBlockCount() as usize,
                TAG_R_TERRAIN,
            ) as *mut CTRPatch;

            landscape.mSortedCount = 2 * (*common).GetBlockCount();
            landscape.mSortedPatches = Z_Malloc(
                core::mem::size_of::<TPatchInfo>() * landscape.mSortedCount as usize,
                TAG_R_TERRAIN,
            ) as *mut TPatchInfo;

            CM_TerrainPatchIterate(common, InitRendererPatches, &mut landscape as *mut _ as *mut c_void);

            // Calculate shaders dependent on the .terrain file
            landscape.CalculateShaders();

            // Get the contents shader
            shaderNum = atol(Info_ValueForKey(configstring, b"shader\0".as_ptr() as *const c_char)) as c_int;
            landscape.mShader = R_GetShaderByHandle(R_GetShaderByNum(
                shaderNum,
                &mut tr.world as *mut _ as *mut refdef_t,
            ));

            landscape.mPatchSize = VectorLength((*common).GetPatchSize());

            #[cfg(feature = "_DEBUG")]
            {
                landscape.mCycleCount = 0;
            }

            landscape
        }
        #[cfg(feature = "PRE_RELEASE_DEMO")]
        {
            // PRE_RELEASE_DEMO: empty implementation
            CTRLandScape {
                common: null_mut(),
                mTRPatches: null_mut(),
                mSortedPatches: null_mut(),
                mPatchMinx: 0,
                mPatchMaxx: 0,
                mPatchMiny: 0,
                mPatchMaxy: 0,
                mMaxNode: 0,
                mSortedCount: 0,
                mPatchSize: 0.0,
                mShader: null_mut(),
                mRenderMap: null_mut(),
                mTextureScale: 0.0,
                mScalarSize: 0.0,
                mWaterShader: null_mut(),
                mFlatShader: 0,
                mHeightDetails: [CTRHeightDetails::new(); HEIGHT_RESOLUTION],
                mCycleCount: 0,
            }
        }
    }
}

// ---------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn RB_SurfaceTerrain(surf: *mut surfaceInfo_t) {
    /*
    if(backEnd.refdef.rdflags & RDF_PROJECTION2D)
    {
        return;
    }
    */
    let ls: *mut srfTerrain_t = surf as *mut srfTerrain_t;
    let landscape: *mut CTRLandScape = (*ls).landscape;

    TerrainFog = tr.world.globalFog;

    (*landscape).CalculateRegion();
    (*landscape).Reset(true);
    //	(*landscape).Tessellate();
    (*landscape).Render();
}

#[no_mangle]
pub unsafe extern "C" fn R_CalcTerrainVisBounds(landscape: *mut CTRLandScape) {
    let common: *const CCMLandScape = (*landscape).GetCommon();

    // Set up the visbounds using terrain data
    if (*common).GetMins()[0] < tr.viewParms.visBounds[0][0] {
        tr.viewParms.visBounds[0][0] = (*common).GetMins()[0];
    }
    if (*common).GetMins()[1] < tr.viewParms.visBounds[0][1] {
        tr.viewParms.visBounds[0][1] = (*common).GetMins()[1];
    }
    if (*common).GetMins()[2] < tr.viewParms.visBounds[0][2] {
        tr.viewParms.visBounds[0][2] = (*common).GetMins()[2];
    }

    if (*common).GetMaxs()[0] > tr.viewParms.visBounds[1][0] {
        tr.viewParms.visBounds[1][0] = (*common).GetMaxs()[0];
    }
    if (*common).GetMaxs()[1] > tr.viewParms.visBounds[1][1] {
        tr.viewParms.visBounds[1][1] = (*common).GetMaxs()[1];
    }
    if (*common).GetMaxs()[2] > tr.viewParms.visBounds[1][2] {
        tr.viewParms.visBounds[1][2] = (*common).GetMaxs()[2];
    }
}

#[no_mangle]
pub unsafe extern "C" fn R_AddTerrainSurfaces() {
    let landscape: *mut CTRLandScape;

    if (*r_drawTerrain).integer == 0 || (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        return;
    }

    landscape = tr.landScape.landscape;
    if !landscape.is_null() {
        R_AddDrawSurf(
            &mut tr.landScape as *mut _ as *mut c_void,
            (*landscape).GetShader(),
            0,
            false,
        );
        R_CalcTerrainVisBounds(landscape);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RE_InitRendererTerrain(info: *const c_char) {
    let ls: *mut CTRLandScape;

    if info.is_null() || *info == 0 {
        Com_Printf(b"RE_RegisterTerrain: NULL name\n\0".as_ptr() as *const c_char);
        return;
    }

    Com_Printf(b"R_Terrain: Creating RENDERER data.....\n\0".as_ptr() as *const c_char);

    // Create and register a new landscape structure
    ls = Box::leak(Box::new(CTRLandScape::new(info)));
}

#[no_mangle]
pub unsafe extern "C" fn R_TerrainInit() {
    tr.landScape.surfaceType = SF_TERRAIN;
    tr.landScape.landscape = null_mut();

    r_terrainTessellate = Cvar_Get(b"r_terrainTessellate\0".as_ptr() as *const c_char, b"3\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_drawTerrain = Cvar_Get(b"r_drawTerrain\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_showFrameVariance = Cvar_Get(
        b"r_showFrameVariance\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        0,
    );
    r_terrainWaterOffset = Cvar_Get(
        b"r_terrainWaterOffset\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        0,
    );

    tr.distanceCull = 6000.0;
    tr.distanceCullSquared = tr.distanceCull * tr.distanceCull;
}

extern "C" {
    fn CM_ShutdownTerrain(terrainId: thandle_t); //cm_load.cpp
}

#[no_mangle]
pub unsafe extern "C" fn R_TerrainShutdown() {
    let ls: *mut CTRLandScape;

    //	Com_Printf("R_Terrain: Shutting down RENDERER terrain.....\n");
    ls = tr.landScape.landscape;
    if !ls.is_null() {
        CM_ShutdownTerrain(0);
        Box::from_raw(ls);
        tr.landScape.landscape = null_mut();
    }
}

// end
