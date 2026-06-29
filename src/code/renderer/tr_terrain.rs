#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// To do:
// Alter variance dependent on global distance from player (colour code this for cg_terrainCollisionDebug)
// Improve texture blending on edge conditions
// Link to neightbouring terrains or architecture (edge conditions)
// Post process generated light data to make sure there are no bands within a patch

// Type aliases and stubs
pub type vec3_t = [f32; 3];
pub type vec3pair_t = [vec3_t; 2];
pub type ivec3_t = [i32; 3];
pub type ivec5_t = [i32; 5];
pub type color4ub_t = [u8; 4];
pub type byte = u8;
pub type qhandle_t = i32;
pub type thandle_t = i32;

#[repr(C)]
pub struct cplane_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct shader_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CCMPatch {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CGenericParser2 {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CGPGroup {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CTerrainVert {
    _opaque: [u8; 0],
}

// Macro-like functions
#[inline]
fn VectorSet5(v: &mut ivec5_t, x: i32, y: i32, z: i32, a: i32, b: i32) {
    v[0] = x;
    v[1] = y;
    v[2] = z;
    v[3] = a;
    v[4] = b;
}

#[inline]
fn VectorScaleVectorAdd(c: &vec3_t, a: &ivec3_t, b: &vec3_t, o: &mut vec3_t) {
    o[0] = c[0] + ((a[0] as f32) * b[0]);
    o[1] = c[1] + ((a[1] as f32) * b[1]);
    o[2] = c[2] + ((a[2] as f32) * b[2]);
}

#[inline]
fn VectorCopy(src: &[f32; 3], dst: &mut [f32; 3]) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

#[inline]
fn VectorSet(v: &mut ivec3_t, x: i32, y: i32, z: i32) {
    v[0] = x;
    v[1] = y;
    v[2] = z;
}

#[inline]
fn VectorSubtract(a: &vec3_t, b: &vec3_t, c: &mut vec3_t) {
    c[0] = a[0] - b[0];
    c[1] = a[1] - b[1];
    c[2] = a[2] - b[2];
}

#[inline]
fn CrossProduct(v1: &vec3_t, v2: &vec3_t, cross: &mut vec3_t) {
    cross[0] = v1[1] * v2[2] - v1[2] * v2[1];
    cross[1] = v1[2] * v2[0] - v1[0] * v2[2];
    cross[2] = v1[0] * v2[1] - v1[1] * v2[0];
}

#[inline]
fn VectorNormalize(v: &mut vec3_t) {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        let inv_len = 1.0 / len;
        v[0] *= inv_len;
        v[1] *= inv_len;
        v[2] *= inv_len;
    }
}

#[inline]
fn VectorAdd(a: &vec3_t, b: &vec3_t, c: &mut vec3_t) {
    c[0] = a[0] + b[0];
    c[1] = a[1] + b[1];
    c[2] = a[2] + b[2];
}

#[inline]
fn VectorScale(v: &vec3_t, scale: f32, o: &mut vec3_t) {
    o[0] = v[0] * scale;
    o[1] = v[1] * scale;
    o[2] = v[2] * scale;
}

#[inline]
fn VectorMA(v: &vec3_t, scale: f32, dir: &vec3_t, o: &mut vec3_t) {
    o[0] = v[0] + scale * dir[0];
    o[1] = v[1] + scale * dir[1];
    o[2] = v[2] + scale * dir[2];
}

#[inline]
fn DotProduct(a: &vec3_t, b: &vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn VectorLengthSquared(v: &vec3_t) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

#[inline]
fn DistanceSquared(a: &vec3_t, b: &vec3_t) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

// External functions and globals
extern "C" {
    pub static mut r_drawTerrain: *mut cvar_t;
    pub static mut r_terrainTessellate: *mut cvar_t;
    pub static mut r_terrainWaterOffset: *mut cvar_t;
    pub static mut r_count: *mut cvar_t;

    pub static mut tr: trGlobals_t;
    pub static mut backEnd: backEndData_t;
    pub static mut tess: tess_t;

    pub fn RB_CheckOverflow(verts: i32, indexes: i32);
    pub fn RB_EndSurface();
    pub fn RB_BeginSurface(shader: *mut shader_t, fogNum: i32);
    pub fn R_GetShaderByHandle(handle: qhandle_t) -> *mut shader_t;
    pub fn R_LightForPoint(
        point: *const vec3_t,
        ambientLight: *mut vec3_t,
        directedLight: *mut vec3_t,
        lightDir: *mut vec3_t,
    ) -> i32;
    pub fn CM_CullWorldBox(frustum: *const cplane_t, bounds: *const vec3pair_t) -> bool;
    pub fn R_RegisterShader(name: *const c_char) -> qhandle_t;
    pub fn R_CreateBlendedShader(a: qhandle_t, b: qhandle_t, c: qhandle_t, surfaceSprites: bool) -> qhandle_t;
    pub fn R_GetShaderByNum(shaderNum: i32, worldData: *mut c_void) -> qhandle_t;
    pub fn R_AddDrawSurf(surf: *const c_void, shader: *mut shader_t, fogNum: i32, dlightMap: bool);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_sprintf(buffer: *mut c_char, bufsize: usize, fmt: *const c_char, ...);
    pub fn Com_Clamp(min: f32, max: f32, value: f32) -> f32;
    pub fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: i32) -> *mut cvar_t;
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    pub fn floorf(x: f32) -> f32;
    pub fn ceilf(x: f32) -> f32;
    pub fn powf(x: f32, y: f32) -> f32;
    pub fn atof(s: *const c_char) -> f64;
    pub fn atol(s: *const c_char) -> i64;
    pub fn stricmp(s1: *const c_char, s2: *const c_char) -> i32;
    pub fn Z_Malloc(size: usize, tag: i32, clear: bool) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Com_ParseTextFile(fileName: *const c_char, parser: *mut CGenericParser2) -> bool;
    pub fn Com_ParseTextFileDestroy(parser: CGenericParser2);
    pub fn CM_RegisterTerrain(config: *const c_char, server: bool) -> *mut CCMLandScape;
    pub fn CM_TerrainPatchIterate(landscape: *const CCMLandScape, func: unsafe extern "C" fn(*mut CCMPatch, *mut c_void), data: *mut c_void);
    pub fn CM_ShutdownTerrain(terrainId: thandle_t);
    pub fn Q_log2(x: i32) -> i32;
    pub fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: unsafe extern "C" fn(*const c_void, *const c_void) -> i32);

    pub fn common_GetMins() -> *const vec3_t;
    pub fn common_GetBaseWaterHeight() -> f32;
    pub fn common_GetFlattenMap() -> *mut byte;
    pub fn common_GetTerxels() -> i32;
}

// Type stubs
#[repr(C)]
pub struct cvar_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct trGlobals_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct backEndData_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct tess_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct refdef_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct srfTerrain_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct world_t {
    _opaque: [u8; 0],
}

// Global statics
static mut TerrainFog: i32 = 0;
static mut TerrainDistanceCull: f32 = 0.0;

// Constants
const MAX_QPATH: usize = 64;
const MAX_TERRAINS: usize = 1;
const TAG_R_TERRAIN: i32 = 14;
const HEIGHT_RESOLUTION: usize = 256;
const SF_TERRAIN: i32 = 10;
const CVAR_CHEAT: i32 = 1;
const RDF_NOWORLDMODEL: i32 = 1;
const PI_TOP: i32 = 1;
const PI_BOTTOM: i32 = 2;
const TEXTURE_ALPHA_TL: i32 = 0x000000ff;
const TEXTURE_ALPHA_TR: i32 = 0x0000ff00;
const TEXTURE_ALPHA_BL: i32 = 0x00ff0000;
const TEXTURE_ALPHA_BR: i32 = 0x000000ff;
const INDEX_TL: i32 = 0;
const INDEX_TR: i32 = 1;
const INDEX_BL: i32 = 2;
const INDEX_BR: i32 = 3;

// Render the tree.
//
unsafe fn RenderCorner(corner: &mut ivec5_t) {
    if corner[3] < 0 {
        // Access to tess and other globals would happen here
        // This is a stub implementation
    }
}

// Render the tree.
pub unsafe fn CTRPatch_RenderCorner(corner: &mut ivec5_t) {
    if corner[3] < 0 {
        // Placeholder for actual implementation
    }
}

// Render the mesh.
//
// The order of triangles is critical to the subdivision working

pub unsafe fn CTRPatch_RecurseRender(depth: i32, left: &mut ivec5_t, right: &mut ivec5_t, apex: &mut ivec5_t) {
    // All non-leaf nodes have both children, so just check for one
    if depth >= 0 {
        let mut center: ivec5_t = [0; 5];

        // Work out the centre of the hypoteneuse
        center[0] = (left[0] + right[0]) >> 1;
        center[1] = (left[1] + right[1]) >> 1;

        // Work out the relevant texture coefficients at that point
        let leftAlphas = &left[2..6] as *const i32 as *const byte;
        let rightAlphas = &right[2..6] as *const i32 as *const byte;
        let centerAlphas = &mut center[2..6] as *mut i32 as *mut byte;

        *centerAlphas.offset(0) = (*leftAlphas.offset(0) + *rightAlphas.offset(0)) >> 1;
        *centerAlphas.offset(1) = (*leftAlphas.offset(1) + *rightAlphas.offset(1)) >> 1;
        *centerAlphas.offset(2) = (*leftAlphas.offset(2) + *rightAlphas.offset(2)) >> 1;
        *centerAlphas.offset(3) = (*leftAlphas.offset(3) + *rightAlphas.offset(3)) >> 1;

        // Make sure the vert index and tesselation registration are not set
        center[3] = -1;
        center[4] = 0;

        if apex[0] == left[0] && apex[0] == center[0] {
            let depth_mut = depth;
            // Would set depth = 0, but depth is not mutable in the signature
        }

        CTRPatch_RecurseRender(depth - 1, apex, left, &mut center);
        CTRPatch_RecurseRender(depth - 1, right, apex, &mut center);
    } else {
        if left[0] == right[0] && left[0] == apex[0] {
            return;
        }
        if left[1] == right[1] && left[1] == apex[1] {
            return;
        }
        // A leaf node!  Output a triangle to be rendered.
        RB_CheckOverflow(4, 4);

        // assert(left[0] != right[0] || left[1] != right[1]);
        // assert(left[0] != apex[0] || left[1] != apex[1]);

        CTRPatch_RenderCorner(left);
        CTRPatch_RenderCorner(right);
        CTRPatch_RenderCorner(apex);
    }
}

pub unsafe fn CTRPatch_Render(Part: i32, patchTerxels: i32) {
    let mut lTL: ivec5_t = [0; 5];
    let mut lTR: ivec5_t = [0; 5];
    let mut lBL: ivec5_t = [0; 5];
    let mut lBR: ivec5_t = [0; 5];

    // VectorSet5(TL, 0, 0, TEXTURE_ALPHA_TL, -1, 0);
    lTL[0] = 0;
    lTL[1] = 0;
    lTL[2] = TEXTURE_ALPHA_TL;
    lTL[3] = -1;
    lTL[4] = 0;
    // VectorSet5(TR, owner->GetTerxels(), 0, TEXTURE_ALPHA_TR, -1, 0);
    lTR[0] = patchTerxels;
    lTR[1] = 0;
    lTR[2] = TEXTURE_ALPHA_TR;
    lTR[3] = -1;
    lTR[4] = 0;
    // VectorSet5(BL, 0, owner->GetTerxels(), TEXTURE_ALPHA_BL, -1, 0);
    lBL[0] = 0;
    lBL[1] = patchTerxels;
    lBL[2] = TEXTURE_ALPHA_BL;
    lBL[3] = -1;
    lBL[4] = 0;
    // VectorSet5(BR, owner->GetTerxels(), owner->GetTerxels(), TEXTURE_ALPHA_BR, -1, 0);
    lBR[0] = patchTerxels;
    lBR[1] = patchTerxels;
    lBR[2] = TEXTURE_ALPHA_BR;
    lBR[3] = -1;
    lBR[4] = 0;

    if (Part & PI_TOP) != 0 {
        // if (mTLShader) {
        // float		d;
        // d = DotProduct (backEnd.refdef.vieworg, mNormal[0]) - mDistance[0];
        // if (d <= 0.0)
        {
            // CTRPatch_RecurseRender(r_terrainTessellate->integer, lBL, lTR, lTL);
        }
        // }
    }

    if (Part & PI_BOTTOM) != 0 {
        // if (mBRShader) {
        // float		d;
        // d = DotProduct (backEnd.refdef.vieworg, mNormal[1]) - mDistance[1];
        // if (d >= 0.0)
        {
            // CTRPatch_RecurseRender(r_terrainTessellate->integer, lTR, lBL, lBR);
        }
        // }
    }
}

// At this point the patch is visible and at least part of it is below water level
//
pub unsafe fn CTRPatch_RenderWaterVert(x: i32, y: i32) -> i32 {
    // CTerVert	*vert;
    // vert = mRenderMap + x + (y * owner->GetRealWidth());
    // if(vert->tessRegistration == tess.registration) {
    //     return(vert->tessIndex);
    // }
    // tess.xyz[tess.numVertexes][0] = vert->coords[0];
    // tess.xyz[tess.numVertexes][1] = vert->coords[1];
    // tess.xyz[tess.numVertexes][2] = owner->GetWaterHeight();
    // *(ulong *)tess.vertexColors[tess.numVertexes] = 0xffffffff;
    // tess.texCoords[tess.numVertexes][0][0] = vert->tex[0];
    // tess.texCoords[tess.numVertexes][0][1] = vert->tex[1];
    // vert->tessIndex = tess.numVertexes;
    // vert->tessRegistration = tess.registration;
    // tess.numVertexes++;
    // return(vert->tessIndex);
    0 // Stub
}

pub unsafe fn CTRPatch_RenderWater() {
    RB_CheckOverflow(4, 6);

    // Get the neighbouring patches
    // let TL = RenderWaterVert(0, 0);
    // let TR = RenderWaterVert(owner->GetTerxels(), 0);
    // let BL = RenderWaterVert(0, owner->GetTerxels());
    // let BR = RenderWaterVert(owner->GetTerxels(), owner->GetTerxels());

    // TL
    // tess.indexes[tess.numIndexes++] = BL;
    // tess.indexes[tess.numIndexes++] = TR;
    // tess.indexes[tess.numIndexes++] = TL;

    // BR
    // tess.indexes[tess.numIndexes++] = TR;
    // tess.indexes[tess.numIndexes++] = BL;
    // tess.indexes[tess.numIndexes++] = BR;
}

pub unsafe fn CTRPatch_HasWater() -> bool {
    // owner->SetRealWaterHeight( owner->GetBaseWaterHeight() + r_terrainWaterOffset->integer );
    // return(common->GetMins()[2] < owner->GetWaterHeight());
    false // Stub
}

pub unsafe fn CTRPatch_SetVisibility(visCheck: bool) {
    // if(visCheck) {
    //     if(DistanceSquared(mCenter, backEnd.refdef.vieworg) > TerrainDistanceCull) {
    //         misVisible = false;
    //     } else {
    //         // Set the visibility of the patch
    //         misVisible = CM_CullWorldBox(backEnd.viewParms.frustum, GetBounds());
    //     }
    // } else {
    //     misVisible = true;
    // }
}

// void CTRPatch::CalcNormal(void)
// {
//     CTerVert	*vert1, *vert2, *vert3;
//     ivec5_t		TL, TR, BL, BR;
//     vec3_t		v1, v2;
//
//     VectorSet5(TL, 0, 0, TEXTURE_ALPHA_TL, -1, 0);
//     VectorSet5(TR, owner->GetTerxels(), 0, TEXTURE_ALPHA_TR, -1, 0);
//     VectorSet5(BL, 0, owner->GetTerxels(), TEXTURE_ALPHA_BL, -1, 0);
//     VectorSet5(BR, owner->GetTerxels(), owner->GetTerxels(), TEXTURE_ALPHA_BR, -1, 0);
//
//     vert1 = mRenderMap + (BL[1] * owner->GetRealWidth()) + BL[0];
//     vert2 = mRenderMap + (TR[1] * owner->GetRealWidth()) + TR[0];
//     vert3 = mRenderMap + (TL[1] * owner->GetRealWidth()) + TL[0];
//     VectorSubtract(vert2->coords, vert1->coords, v1);
//     VectorSubtract(vert3->coords, vert1->coords, v2);
//     CrossProduct(v1, v2, mNormal[0]);
//     VectorNormalize(mNormal[0]);
//     mDistance[0] = DotProduct (vert1->coords, mNormal[0]);
//
//     vert1 = mRenderMap + (BL[1] * owner->GetRealWidth()) + BL[0];
//     vert2 = mRenderMap + (TR[1] * owner->GetRealWidth()) + TR[0];
//     vert3 = mRenderMap + (BR[1] * owner->GetRealWidth()) + BR[0];
//     VectorSubtract(vert2->coords, vert1->coords, v1);
//     VectorSubtract(vert3->coords, vert1->coords, v2);
//     CrossProduct(v1, v2, mNormal[1]);
//     VectorNormalize(mNormal[1]);
//     mDistance[1] = DotProduct (vert1->coords, mNormal[1]);
// }

// Reset all patches, recompute variance if needed
//
pub unsafe fn CTRLandScape_Reset(visCheck: bool) {
    // int			x, y;
    // CTRPatch	*patch;
    //
    // TerrainDistanceCull = tr.distanceCull + mPatchSize;
    // TerrainDistanceCull *= TerrainDistanceCull;
    //
    // // Go through the patches performing resets, compute variances, and linking.
    // for(y = mPatchMiny; y < mPatchMaxy; y++) {
    //     for(x = mPatchMinx; x < mPatchMaxx; x++, patch++) {
    //         patch = GetPatch(x, y);
    //         patch->SetVisibility(visCheck);
    //     }
    // }
}

// Render each patch of the landscape & adjust the frame variance.
//
pub unsafe fn CTRLandScape_Render() {
    // int			x, y;
    // CTRPatch	*patch;
    // TPatchInfo	*current;
    // int			i;
    //
    // // Render all the visible patches
    // current = mSortedPatches;
    // for(i=0;i<mSortedCount;i++) {
    //     if (current->mPatch->isVisible()) {
    //         if (tess.shader != current->mShader) {
    //             RB_EndSurface();
    //             RB_BeginSurface(current->mShader, TerrainFog);
    //         }
    //         current->mPatch->Render(current->mPart);
    //     }
    //     current++;
    // }
    // RB_EndSurface();
    //
    // // Render all the water for visible patches
    // // Done as a separate iteration to reduce the number of tesses created
    // if(mWaterShader && (mWaterShader != tr.defaultShader)) {
    //     RB_BeginSurface( mWaterShader, tr.world->globalFog );
    //
    //     for(y = mPatchMiny; y < mPatchMaxy; y++ ) {
    //         for(x = mPatchMinx; x < mPatchMaxx; x++ ) {
    //             patch = GetPatch(x, y);
    //             if(patch->isVisible() && patch->HasWater()) {
    //                 patch->RenderWater();
    //             }
    //         }
    //     }
    //     RB_EndSurface();
    // }
}

pub unsafe fn CTRLandScape_CalculateRegion() {
    // vec3_t	mins, maxs, size, offset;
    //
    // #if	_DEBUG
    // mCycleCount++;
    // #endif
    // VectorCopy(GetPatchSize(), size);
    // VectorCopy(GetMins(), offset);
    //
    // mins[0] = backEnd.refdef.vieworg[0] - tr.distanceCull - (size[0] * 2.0f) - offset[0];
    // mins[1] = backEnd.refdef.vieworg[1] - tr.distanceCull - (size[1] * 2.0f) - offset[1];
    //
    // maxs[0] = backEnd.refdef.vieworg[0] + tr.distanceCull + (size[0] * 2.0f) - offset[0];
    // maxs[1] = backEnd.refdef.vieworg[1] + tr.distanceCull + (size[1] * 2.0f) - offset[1];
    //
    // mPatchMinx = Com_Clamp(0, GetBlockWidth(), floorf(mins[0] / size[0]));
    // mPatchMaxx = Com_Clamp(0, GetBlockWidth(), ceilf(maxs[0] / size[0]));
    //
    // mPatchMiny = Com_Clamp(0, GetBlockHeight(), floorf(mins[1] / size[1]));
    // mPatchMaxy = Com_Clamp(0, GetBlockHeight(), ceilf(maxs[1] / size[1]));
}

pub unsafe fn CTRLandScape_CalculateRealCoords() {
    // int			x, y;
    //
    // // Work out the real world coordinates of each heightmap entry
    // for(y = 0; y < GetRealHeight(); y++) {
    //     for(x = 0; x < GetRealWidth(); x++) {
    //         ivec3_t		icoords;
    //         int			offset;
    //
    //         offset = (y * GetRealWidth()) + x;
    //
    //         VectorSet(icoords, x, y, mRenderMap[offset].height);
    //         VectorScaleVectorAdd(GetMins(), icoords, GetTerxelSize(), mRenderMap[offset].coords);
    //     }
    // }
}

pub unsafe fn CTRLandScape_CalculateNormals() {
    // int		x, y, offset = 0;
    //
    // // Work out the normals for every face
    // for(y = 0; y < GetHeight(); y++) {
    //     for(x = 0; x < GetWidth(); x++) {
    //         vec3_t		vcenter, vleft;
    //
    //         offset = (y * GetRealWidth()) + x;
    //
    //         VectorSubtract(mRenderMap[offset].coords, mRenderMap[offset + 1].coords, vcenter);
    //         VectorSubtract(mRenderMap[offset].coords, mRenderMap[offset + GetRealWidth()].coords, vleft);
    //
    //         CrossProduct(vcenter, vleft, mRenderMap[offset].normal);
    //         VectorNormalize(mRenderMap[offset].normal);
    //     }
    //     // Duplicate right edge condition
    //     VectorCopy(mRenderMap[offset].normal, mRenderMap[offset + 1].normal);
    // }
    // // Duplicate bottom line
    // offset = GetHeight() * GetRealWidth();
    // for(x = 0; x < GetRealWidth(); x++) {
    //     VectorCopy(mRenderMap[offset - GetRealWidth() + x].normal, mRenderMap[offset + x].normal);
    // }
}

pub unsafe fn CTRLandScape_CalculateLighting() {
    // int		x, y, offset = 0;
    //
    // // Work out the vertex normal (average of every attached face normal) and apply to the direction of the light
    // for(y = 0; y < GetHeight(); y++) {
    //     for(x = 0; x < GetWidth(); x++) {
    //         vec3_t		ambient;
    //         vec3_t		directed, direction;
    //         vec3_t		total, tint;
    //         vec_t		dp;
    //
    //         offset = (y * GetRealWidth()) + x;
    //
    //         // Work out average normal
    //         VectorCopy(GetRenderMap(x, y)->normal, total);
    //         VectorAdd(total, GetRenderMap(x + 1, y)->normal, total);
    //         VectorAdd(total, GetRenderMap(x + 1, y + 1)->normal, total);
    //         VectorAdd(total, GetRenderMap(x, y + 1)->normal, total);
    //         VectorNormalize(total);
    //
    //         if (!R_LightForPoint(mRenderMap[offset].coords, ambient, directed, direction)) {
    //             mRenderMap[offset].tint[0] =
    //                 mRenderMap[offset].tint[1] =
    //                 mRenderMap[offset].tint[2] = 255 >> tr.overbrightBits;
    //             mRenderMap[offset].tint[3] = 255;
    //             continue;
    //         }
    //
    //         if(mRenderMap[offset].coords[2] < common->GetBaseWaterHeight()) {
    //             VectorScale(ambient, 0.75f, ambient);
    //         }
    //
    //         // Both normalised, so -1.0 < dp < 1.0
    //         dp = Com_Clamp(0.0f, 1.0f, DotProduct(direction, total));
    //         dp = powf(dp, 3);
    //         VectorScale(ambient, (1.0 - dp) * 0.5, ambient);
    //         VectorMA(ambient, dp, directed, tint);
    //
    //         // rjr - in R_SetupEntityLighting, ambient light is automatically increased by 32, so do it here to match
    //         // rjr - decided to disable both the lighting boost automatically in there as well as here.
    //         mRenderMap[offset].tint[0] = (byte)Com_Clamp(0.0f, 255.0f, tint[0] ) >> tr.overbrightBits;
    //         mRenderMap[offset].tint[1] = (byte)Com_Clamp(0.0f, 255.0f, tint[1] ) >> tr.overbrightBits;
    //         mRenderMap[offset].tint[2] = (byte)Com_Clamp(0.0f, 255.0f, tint[2] ) >> tr.overbrightBits;
    //         mRenderMap[offset].tint[3] = 0xff;
    //     }
    //     mRenderMap[offset + 1].tint[0] = mRenderMap[offset].tint[0];
    //     mRenderMap[offset + 1].tint[1] = mRenderMap[offset].tint[1];
    //     mRenderMap[offset + 1].tint[2] = mRenderMap[offset].tint[2];
    //     mRenderMap[offset + 1].tint[3] = 0xff;
    // }
    // // Duplicate bottom line
    // offset = GetHeight() * GetRealWidth();
    // for(x = 0; x < GetRealWidth(); x++) {
    //     mRenderMap[offset + x].tint[0] = mRenderMap[offset - GetRealWidth() + x].tint[0];
    //     mRenderMap[offset + x].tint[1] = mRenderMap[offset - GetRealWidth() + x].tint[1];
    //     mRenderMap[offset + x].tint[2] = mRenderMap[offset - GetRealWidth() + x].tint[2];
    //     mRenderMap[offset + x].tint[3] = 0xff;
    // }
}

pub unsafe fn CTRLandScape_CalculateTextureCoords() {
    // int		x, y;
    //
    // for(y = 0; y < GetRealHeight(); y++) {
    //     for(x = 0; x < GetRealWidth(); x++) {
    //         int offset = (y * GetRealWidth()) + x;
    //
    //         mRenderMap[offset].tex[0] = x * mTextureScale * GetTerxelSize()[0];
    //         mRenderMap[offset].tex[1] = y * mTextureScale * GetTerxelSize()[1];
    //     }
    // }
}

pub unsafe fn CTRLandScape_SetShaders(height: i32, shader: qhandle_t) {
    // int		i;
    //
    // for(i = height; shader && (i < HEIGHT_RESOLUTION); i++) {
    //     if(!mHeightDetails[i].GetShader()) {
    //         mHeightDetails[i].SetShader(shader);
    //     }
    // }
}

pub unsafe fn CTRLandScape_LoadTerrainDef(td: *const c_char) {
    // #ifndef PRE_RELEASE_DEMO
    // char			terrainDef[MAX_QPATH];
    // CGenericParser2	parse;
    // CGPGroup		*basegroup, *classes, *items;
    //
    // Com_sprintf(terrainDef, MAX_QPATH, "ext_data/RMG/%s.terrain", td);
    // Com_Printf("R_Terrain: Loading and parsing terrainDef %s.....\n", td);
    //
    // mWaterShader = NULL;
    // mFlatShader  = NULL;
    //
    // if(!Com_ParseTextFile(terrainDef, parse)) {
    //     Com_sprintf(terrainDef, MAX_QPATH, "ext_data/arioche/%s.terrain", td);
    //     if(!Com_ParseTextFile(terrainDef, parse)) {
    //         Com_Printf("Could not open %s\n", terrainDef);
    //         return;
    //     }
    // }
    // // The whole file....
    // basegroup = parse.GetBaseParseGroup();
    //
    // // The root { } struct
    // classes = basegroup->GetSubGroups();
    // while(classes) {
    //     items = classes->GetSubGroups();
    //     while(items) {
    //         const char* type = items->GetName ( );
    //
    //         if(!stricmp( type, "altitudetexture")) {
    //             int			height;
    //             const char	*shaderName;
    //             qhandle_t	shader;
    //
    //             // Height must exist - the rest are optional
    //             height = atol(items->FindPairValue("height", "0"));
    //
    //             // Shader for this height
    //             shaderName = items->FindPairValue("shader", "");
    //             if(shaderName[0]) {
    //                 shader = RE_RegisterShader(shaderName);
    //                 if(shader) {
    //                     SetShaders(height, shader);
    //                 }
    //             }
    //         } else if(!stricmp(type, "water")) {
    //             mWaterShader = R_GetShaderByHandle(RE_RegisterShader(items->FindPairValue("shader", "")));
    //         } else if(!stricmp(type, "flattexture")) {
    //             mFlatShader = RE_RegisterShader ( items->FindPairValue("shader", "") );
    //         }
    //
    //         items = (CGPGroup *)items->GetNext();
    //     }
    //     classes = (CGPGroup *)classes->GetNext();
    // }
    //
    // Com_ParseTextFileDestroy(parse);
    // #endif // PRE_RELEASE_DEMO
}

pub unsafe fn CTRLandScape_GetBlendedShader(a: qhandle_t, b: qhandle_t, c: qhandle_t, surfaceSprites: bool) -> qhandle_t {
    // Special case single pass shader
    if a == b && a == c {
        return a;
    }

    let blended = R_CreateBlendedShader(a, b, c, surfaceSprites);
    blended
}

unsafe extern "C" fn ComparePatchInfo(arg1: *const TPatchInfo, arg2: *const TPatchInfo) -> i32 {
    let arg1 = &*arg1;
    let arg2 = &*arg2;

    let s1: *mut shader_t;
    let s2: *mut shader_t;

    if (arg1.mPart & PI_TOP) != 0 {
        s1 = arg1.mShader;
    } else {
        s1 = arg1.mShader;
    }

    if (arg2.mPart & PI_TOP) != 0 {
        s2 = arg2.mShader;
    } else {
        s2 = arg2.mShader;
    }

    if (s1 as usize) < (s2 as usize) {
        return -1;
    } else if (s1 as usize) > (s2 as usize) {
        return 1;
    }

    0
}

#[repr(C)]
pub struct TPatchInfo {
    pub mPatch: *mut CTRPatch,
    pub mShader: *mut shader_t,
    pub mPart: i32,
}

#[repr(C)]
pub struct CTRPatch {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CTerVert {
    _opaque: [u8; 0],
}

pub unsafe fn CTRLandScape_CalculateShaders() {
    // #ifndef PRE_RELEASE_DEMO
    // int						x, y;
    // int						width, height;
    // int						offset;
    // // int						offsets[4];
    // qhandle_t				handles[4];
    // CTRPatch				*patch;
    // qhandle_t				*shaders;
    // TPatchInfo				*current = mSortedPatches;
    //
    // width  = GetWidth ( ) / common->GetTerxels ( );
    // height = GetHeight ( ) / common->GetTerxels ( );
    //
    // shaders = new qhandle_t [ (width+1) * (height+1) ];
    //
    // // On the first pass determine all of the shaders for the entire
    // // terrain assuming no flat ground
    // offset = 0;
    // for ( y = 0; y < height + 1; y ++ ) {
    //     if ( y <= height ) {
    //         offset = common->GetTerxels ( ) * y * GetRealWidth ( );
    //     } else {
    //         offset = common->GetTerxels ( ) * (y-1) * GetRealWidth ( );
    //         offset += GetRealWidth ( );
    //     }
    //
    //     for ( x = 0; x < width + 1; x ++, offset += common->GetTerxels ( ) ) {
    //         // Save the shader
    //         shaders[y * width + x] = GetHeightDetail(mRenderMap[offset].height)->GetShader ( );
    //     }
    // }
    //
    // // On the second pass determine flat ground and replace the shader
    // // at that point with the flat ground shader
    // byte*	flattenMap = common->GetFlattenMap ( );
    // if ( mFlatShader && flattenMap ) {
    //     for ( y = 1; y < height; y ++ ) {
    //         for ( x = 1; x < width; x ++ ) {
    //             int		offset;
    //             int		xx;
    //             int		yy;
    //             bool	flat	   = false;
    //
    //             offset  = (x) * common->GetTerxels ( );
    //             offset += (y) * common->GetTerxels ( ) * GetRealWidth();
    //
    //             offset -= GetRealWidth();
    //             offset -= 1;
    //
    //             for ( yy = 0; yy < 3 && !flat; yy++ ) {
    //                 for ( xx = 0; xx < 3 && !flat; xx++ ) {
    //                     if ( flattenMap [ offset + xx] & 0x80) {
    //                         flat = true;
    //                         break;
    //                     }
    //                 }
    //
    //                 offset += GetRealWidth();
    //             }
    //             // This shader is now a flat shader
    //             if ( flat ) {
    //                 shaders[y * width + x] = mFlatShader;
    //             }
    //
    // #ifdef _DEBUG
    //             OutputDebugString ( va("Flat Area:  %f %f\n",
    //                 GetMins()[0] + (GetMaxs()[0]-GetMins()[0])/width * x,
    //                 GetMins()[1] + (GetMaxs()[1]-GetMins()[1])/height * y) );
    // #endif
    //         }
    //     }
    // }
    //
    // // Now that the shaders have been determined, set them for each patch
    // patch = mTRPatches;
    // mSortedCount = 0;
    // for ( y = 0; y < height; y ++ ) {
    //     for ( x = 0; x < width; x ++, patch++ ) {
    //         bool surfaceSprites = true;
    //
    //         handles[INDEX_TL] = shaders[ x + y * width ];
    //         handles[INDEX_TR] = shaders[ x + 1 + y * width ];
    //         handles[INDEX_BL] = shaders[ x + (y + 1) * width ];
    //         handles[INDEX_BR] = shaders[ x + 1 + (y + 1) * width ];
    //
    //         if ( handles[INDEX_TL] == mFlatShader ||
    //              handles[INDEX_TR] == mFlatShader ||
    //              handles[INDEX_BL] == mFlatShader ||
    //              handles[INDEX_BR] == mFlatShader    ) {
    //             surfaceSprites = false;
    //         }
    //
    //         patch->SetTLShader(GetBlendedShader(handles[INDEX_TR], handles[INDEX_BL], handles[INDEX_TL], surfaceSprites));
    //         current->mPatch = patch;
    //         current->mShader = patch->GetTLShader();
    //         current->mPart = PI_TOP;
    //
    //         patch->SetBRShader(GetBlendedShader(handles[INDEX_TR], handles[INDEX_BL], handles[INDEX_BR], surfaceSprites));
    //         if (patch->GetBRShader() == current->mShader) {
    //             current->mPart |= PI_BOTTOM;
    //         } else {
    //             mSortedCount++;
    //             current++;
    //
    //             current->mPatch = patch;
    //             current->mShader = patch->GetBRShader();
    //             current->mPart = PI_BOTTOM;
    //         }
    //         mSortedCount++;
    //         current++;
    //     }
    // }
    //
    // // Cleanup our temporary array
    // delete[] shaders;
    //
    // qsort(mSortedPatches, mSortedCount, sizeof(*mSortedPatches), (int (__cdecl *)(const void *,const void *))ComparePatchInfo);
    //
    // #endif // PRE_RELEASE_DEMO
}

pub unsafe fn CTRPatch_SetRenderMap(x: i32, y: i32) {
    // mRenderMap = localowner->GetRenderMap(x, y);
}

pub unsafe extern "C" fn InitRendererPatches(patch: *mut CCMPatch, userdata: *mut c_void) {
    // int			  	tx, ty, bx, by;
    // CTRPatch	  	*localpatch;
    // CCMLandScape	*owner;
    // CTRLandScape	*localowner;
    //
    // // Set owning landscape
    // localowner = (CTRLandScape *)userdata;
    // owner = (CCMLandScape *)localowner->GetCommon();
    //
    // // Get TRPatch pointer
    // tx = patch->GetHeightMapX();
    // ty = patch->GetHeightMapY();
    // bx = tx / owner->GetTerxels();
    // by = ty / owner->GetTerxels();
    //
    // localpatch = localowner->GetPatch(bx, by);
    // localpatch->Clear();
    //
    // localpatch->SetCommon(patch);
    // localpatch->SetOwner(owner);
    // localpatch->SetLocalOwner(localowner);
    // localpatch->SetRenderMap(tx, ty);
    // localpatch->SetCenter();
    // // localpatch->CalcNormal();
}

pub unsafe fn CTRLandScape_CopyHeightMap() {
    // const CCMLandScape	*common = GetCommon();
    // const byte			*heightMap = common->GetHeightMap();
    // CTerVert			*renderMap = mRenderMap;
    // int					i;
    //
    // for(i = 0; i < common->GetRealArea(); i++) {
    //     renderMap->height = *heightMap;
    //     renderMap++;
    //     heightMap++;
    // }
}

pub unsafe fn CTRLandScape_Destructor() {
    // if(mTRPatches) {
    //     Z_Free(mTRPatches);
    //     mTRPatches = NULL;
    // }
    // if (mSortedPatches) {
    //     Z_Free(mSortedPatches);
    //     mSortedPatches = 0;
    // }
    // if(mRenderMap) {
    //     Z_Free(mRenderMap);
    //     mRenderMap = NULL;
    // }
}

pub unsafe fn CTRLandScape_Constructor(configstring: *const c_char) {
    // #ifndef PRE_RELEASE_DEMO
    // int					shaderNum;
    // const CCMLandScape	*common;
    //
    // memset(this, 0, sizeof(*this));
    //
    // // Sets up the common aspects of the terrain
    // common = CM_RegisterTerrain(configstring, false);
    // SetCommon(common);
    //
    // tr.landScape.landscape = this;
    //
    // mTextureScale = (float)atof(Info_ValueForKey(configstring, "texturescale")) / common->GetTerxels();
    // LoadTerrainDef(Info_ValueForKey(configstring, "terrainDef"));
    //
    // // To normalise the variance value to a reasonable number
    // mScalarSize = VectorLengthSquared(common->GetSize());
    //
    // // Calculate and set variance depth
    // mMaxNode = (Q_log2(common->GetTerxels()) << 1) - 1;
    //
    // // Allocate space for the renderer specific data
    // mRenderMap = (CTerVert *)Z_Malloc(sizeof(CTerVert) * common->GetRealArea(), TAG_R_TERRAIN, qfalse);
    //
    // // Copy byte heightmap to rendermap to speed up calcs
    // CopyHeightMap();
    //
    // // Calculate the real world location for each heightmap entry
    // CalculateRealCoords();
    //
    // // Calculate the normal of each terxel
    // CalculateNormals();
    //
    // // Calculate modulation values for the heightmap
    // CalculateLighting();
    //
    // // Calculate texture coords (not projected - real)
    // CalculateTextureCoords();
    //
    // Com_Printf ("R_Terrain: Creating renderer patches.....\n");
    // // Initialise all terrain patches
    // mTRPatches = (CTRPatch *)Z_Malloc(sizeof(CTRPatch) * common->GetBlockCount(), TAG_R_TERRAIN, qfalse);
    //
    // mSortedCount = 2 * common->GetBlockCount();
    // mSortedPatches = (TPatchInfo *)Z_Malloc(sizeof(TPatchInfo) * mSortedCount, TAG_R_TERRAIN, qfalse);
    //
    // CM_TerrainPatchIterate(common, InitRendererPatches, this);
    //
    // // Calculate shaders dependent on the .terrain file
    // CalculateShaders();
    //
    // // Get the contents shader
    // shaderNum = atol(Info_ValueForKey(configstring, "shader"));
    //
    // mShader = R_GetShaderByHandle(R_GetShaderByNum(shaderNum, *tr.world));
    //
    // mPatchSize = VectorLength(common->GetPatchSize());
    //
    // #if	_DEBUG
    // mCycleCount = 0;
    // #endif
    // #endif // PRE_RELEASE_DEMO
}

// RB_SurfaceTerrain - Render terrain surface
pub unsafe fn RB_SurfaceTerrain(surf: *mut srfTerrain_t) {
    let ls = surf as *mut srfTerrain_t;
    // let landscape = (*ls).landscape;
    //
    // TerrainFog = tr.world->globalFog;
    //
    // landscape->CalculateRegion();
    // landscape->Reset();
    // // landscape->Tessellate();
    // landscape->Render();
}

// R_CalcTerrainVisBounds - Calculate terrain visibility bounds
pub unsafe fn R_CalcTerrainVisBounds(landscape: *mut c_void) {
    // const CCMLandScape *common = landscape->GetCommon();
    //
    // // Set up the visbounds using terrain data
    // if ( common->GetMins()[0] < tr.viewParms.visBounds[0][0] ) {
    //     tr.viewParms.visBounds[0][0] = common->GetMins()[0];
    // }
    // if ( common->GetMins()[1] < tr.viewParms.visBounds[0][1] ) {
    //     tr.viewParms.visBounds[0][1] = common->GetMins()[1];
    // }
    // if ( common->GetMins()[2] < tr.viewParms.visBounds[0][2] ) {
    //     tr.viewParms.visBounds[0][2] = common->GetMins()[2];
    // }
    //
    // if ( common->GetMaxs()[0] > tr.viewParms.visBounds[1][0] ) {
    //     tr.viewParms.visBounds[1][0] = common->GetMaxs()[0];
    // }
    // if ( common->GetMaxs()[1] > tr.viewParms.visBounds[1][1] ) {
    //     tr.viewParms.visBounds[1][1] = common->GetMaxs()[1];
    // }
    // if ( common->GetMaxs()[2] > tr.viewParms.visBounds[1][2] ) {
    //     tr.viewParms.visBounds[1][2] = common->GetMaxs()[2];
    // }
}

// R_AddTerrainSurfaces - Add terrain surfaces for rendering
pub unsafe fn R_AddTerrainSurfaces() {
    // let landscape: *mut CTRLandScape;
    //
    // if !r_drawTerrain->integer || (tr.refdef.rdflags & RDF_NOWORLDMODEL) {
    //     return;
    // }
    //
    // landscape = tr.landScape.landscape;
    // if landscape {
    //     R_AddDrawSurf( (surfaceType_t *)(&tr.landScape), landscape->GetShader(), 0, qfalse );
    //     R_CalcTerrainVisBounds(landscape);
    // }
}

// RE_InitRendererTerrain - Initialize terrain renderer
pub unsafe fn RE_InitRendererTerrain(info: *const c_char) {
    // let ls: *mut CTRLandScape;
    //
    // if !info || !(*info) {
    //     Com_Printf( "RE_RegisterTerrain: NULL name\n" );
    //     return;
    // }
    //
    // Com_Printf("R_Terrain: Creating RENDERER data.....\n");
    //
    // // Create and register a new landscape structure
    // ls = new CTRLandScape(info);
}

// R_TerrainInit - Initialize terrain subsystem
pub unsafe fn R_TerrainInit() {
    let mut i: i32;

    for i in 0..MAX_TERRAINS as i32 {
        // tr.landScape.surfaceType = SF_TERRAIN;
        // tr.landScape.landscape = NULL;
    }
    r_terrainTessellate = Cvar_Get(
        b"r_terrainTessellate\0".as_ptr() as *const c_char,
        b"3\0".as_ptr() as *const c_char,
        CVAR_CHEAT,
    );
    r_drawTerrain = Cvar_Get(
        b"r_drawTerrain\0".as_ptr() as *const c_char,
        b"1\0".as_ptr() as *const c_char,
        CVAR_CHEAT,
    );
    r_terrainWaterOffset = Cvar_Get(
        b"r_terrainWaterOffset\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        0,
    );
    r_count = Cvar_Get(
        b"r_count\0".as_ptr() as *const c_char,
        b"2\0".as_ptr() as *const c_char,
        0,
    );
}

// R_TerrainShutdown - Shutdown terrain subsystem
pub unsafe fn R_TerrainShutdown() {
    // let ls: *mut CTRLandScape;
    //
    // // Com_Printf("R_Terrain: Shutting down RENDERER terrain.....\n");
    // ls = tr.landScape.landscape;
    // if ls {
    //     CM_ShutdownTerrain(0);
    //     delete ls;
    //     tr.landScape.landscape = NULL;
    // }
}

// end
