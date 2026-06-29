#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused_mut,
    unused_variables,
    unused_assignments,
    unused_imports,
    dead_code,
    clippy::all
)]

// #include "../server/exe_headers.h"
use crate::code::server::exe_headers_h::*;

// this include must remain at the top of every CPP file
// #include "tr_local.h"
use crate::code::renderer::tr_local_h::*;

// #if !defined(GENERICPARSER2_H_INC)
// #include "../game/genericparser2.h"
// #endif
use crate::code::game::genericparser2_h::*;

// To do:
// Alter variance dependent on global distance from player (colour code this for cg_terrainCollisionDebug)
// Improve texture blending on edge conditions
// Link to neightbouring terrains or architecture (edge conditions)
// Post process generated light data to make sure there are no bands within a patch

// #include "../qcommon/cm_landscape.h"
use crate::code::qcommon::cm_landscape_h::*;
// #include "tr_landscape.h"
use crate::code::renderer::tr_landscape_h::*;

use core::ffi::{c_char, c_int, c_ulong, c_void};
use core::ptr::{addr_of, addr_of_mut};

// #define VectorSet5(v,x,y,z,a,b)  ((v)[0]=(x), (v)[1]=(y), (v)[2]=(z), (v)[3]=(a), (v)[4]=(b))
#[inline]
unsafe fn VectorSet5(v: *mut ivec5_t, x: i32, y: i32, z: i32, a: i32, b: i32) {
    (*v)[0] = x; (*v)[1] = y; (*v)[2] = z; (*v)[3] = a; (*v)[4] = b;
}

// #define VectorScaleVectorAdd(c,a,b,o) ((o)[0]=(c)[0]+((a)[0]*(b)[0]),(o)[1]=(c)[1]+((a)[1]*(b)[1]),(o)[2]=(c)[2]+((a)[2]*(b)[2]))
#[inline]
unsafe fn VectorScaleVectorAdd(c: vec3_t, a: ivec3_t, b: vec3_t, o: *mut vec3_t) {
    (*o)[0] = c[0] + (a[0] as f32 * b[0]);
    (*o)[1] = c[1] + (a[1] as f32 * b[1]);
    (*o)[2] = c[2] + (a[2] as f32 * b[2]);
}

pub static mut r_drawTerrain: *mut cvar_t = core::ptr::null_mut();
pub static mut r_terrainTessellate: *mut cvar_t = core::ptr::null_mut();
pub static mut r_terrainWaterOffset: *mut cvar_t = core::ptr::null_mut();

pub static mut r_count: *mut cvar_t = core::ptr::null_mut();

static mut TerrainFog: c_int = 0;
static mut TerrainDistanceCull: f32 = 0.0;

// Forward-declared in this translation unit; defined elsewhere.
unsafe extern "C" {
    fn CM_CullWorldBox(frustum: *const cplane_t, bounds: vec3pair_t) -> bool;
}

unsafe extern "C" {
    fn R_LightForPoint(
        point: *const vec3_t,
        ambientLight: *mut vec3_t,
        directedLight: *mut vec3_t,
        lightDir: *mut vec3_t,
    ) -> c_int;
}

unsafe extern "C" {
    fn R_CreateBlendedShader(
        a: qhandle_t,
        b: qhandle_t,
        c: qhandle_t,
        surfaceSprites: bool,
    ) -> qhandle_t;
}

unsafe extern "C" {
    fn CM_RegisterTerrain(config: *const c_char, server: bool) -> *const CCMLandScape;
    fn R_GetShaderByNum(shaderNum: c_int, worldData: *mut world_t) -> qhandle_t;
    fn CM_ShutdownTerrain(terrainId: thandle_t);
}

//
// Render the tree.
//
// CTRPatch::RenderCorner — translated as a free unsafe fn.
// Note: in C++, ivec5_t corner is an array parameter, which decays to a pointer;
// modifications to corner[3] and corner[4] inside are visible to the caller.
unsafe fn CTRPatch_RenderCorner(this_: *mut CTRPatch, corner: *mut ivec5_t) {
    if ((*corner)[3] < 0) || (tess.registration != (*corner)[4]) {
        let vert: *mut CTerVert;

        vert = (*this_).mRenderMap.add(
            ((*corner)[1] as usize) * ((*(*this_).owner).GetRealWidth() as usize)
                + (*corner)[0] as usize,
        );

        VectorCopy(&(*vert).coords, &mut tess.xyz[tess.numVertexes as usize]);
        VectorCopy(&(*vert).normal, &mut tess.normal[tess.numVertexes as usize]);

        // *(ulong *)tess.vertexColors[tess.numVertexes] = *(ulong *)vert->tint;
        *(tess.vertexColors[tess.numVertexes as usize].as_mut_ptr() as *mut c_ulong) =
            *((*vert).tint.as_ptr() as *const c_ulong);
        // *(ulong *)tess.vertexAlphas[tess.numVertexes] = corner[2];
        *(tess.vertexAlphas[tess.numVertexes as usize].as_mut_ptr() as *mut c_ulong) =
            (*corner)[2] as c_ulong;

        tess.texCoords[tess.numVertexes as usize][0][0] = (*vert).tex[0];
        tess.texCoords[tess.numVertexes as usize][0][1] = (*vert).tex[1];

        tess.indexes[tess.numIndexes as usize] = tess.numVertexes;
        tess.numIndexes += 1;
        (*corner)[3] = tess.numVertexes;
        tess.numVertexes += 1;
        (*corner)[4] = tess.registration;
    } else {
        tess.indexes[tess.numIndexes as usize] = (*corner)[3];
        tess.numIndexes += 1;
    }
}

// CTRPatch::RecurseRender — translated as a free unsafe fn.
// Note: ivec5_t parameters in C++ are array types that decay to pointers;
// all four pointer parameters may be mutated and callers observe the changes.
unsafe fn CTRPatch_RecurseRender(
    this_: *mut CTRPatch,
    depth: c_int,
    left: *mut ivec5_t,
    right: *mut ivec5_t,
    apex: *mut ivec5_t,
) {
    // All non-leaf nodes have both children, so just check for one
    if depth >= 0 {
        let mut center: ivec5_t = [0; 5];
        let centerAlphas: *mut byte;
        let leftAlphas: *const byte;
        let rightAlphas: *const byte;

        // Work out the centre of the hypoteneuse
        center[0] = ((*left)[0] + (*right)[0]) >> 1;
        center[1] = ((*left)[1] + (*right)[1]) >> 1;

        // Work out the relevant texture coefficients at that point
        leftAlphas = (&(*left)[2]) as *const i32 as *const byte;
        rightAlphas = (&(*right)[2]) as *const i32 as *const byte;
        centerAlphas = (&mut center[2]) as *mut i32 as *mut byte;

        *centerAlphas.offset(0) = (*leftAlphas.offset(0) + *rightAlphas.offset(0)) >> 1;
        *centerAlphas.offset(1) = (*leftAlphas.offset(1) + *rightAlphas.offset(1)) >> 1;
        *centerAlphas.offset(2) = (*leftAlphas.offset(2) + *rightAlphas.offset(2)) >> 1;
        *centerAlphas.offset(3) = (*leftAlphas.offset(3) + *rightAlphas.offset(3)) >> 1;

        // Make sure the vert index and tesselation registration are not set
        center[3] = -1;
        center[4] = 0;

        // Porting note: in C++ `depth` is a local parameter that can be re-assigned.
        // Shadow it with a mutable binding so the recursive calls see the updated value.
        let mut depth = depth;
        if (*apex)[0] == (*left)[0] && (*apex)[0] == center[0] {
            depth = 0;
        }

        CTRPatch_RecurseRender(this_, depth - 1, apex, left, &mut center);
        CTRPatch_RecurseRender(this_, depth - 1, right, apex, &mut center);
    } else {
        if (*left)[0] == (*right)[0] && (*left)[0] == (*apex)[0] {
            return;
        }
        if (*left)[1] == (*right)[1] && (*left)[1] == (*apex)[1] {
            return;
        }
        // A leaf node!  Output a triangle to be rendered.
        RB_CheckOverflow(4, 4);

        // assert(left[0] != right[0] || left[1] != right[1]);
        // assert(left[0] != apex[0] || left[1] != apex[1]);

        CTRPatch_RenderCorner(this_, left);
        CTRPatch_RenderCorner(this_, right);
        CTRPatch_RenderCorner(this_, apex);
    }
}

//
// Render the mesh.
//
// The order of triangles is critical to the subdivision working

// CTRPatch::Render
pub unsafe fn CTRPatch_Render(this_: *mut CTRPatch, Part: c_int) {
    let mut lTL: ivec5_t = [0; 5];
    let mut lTR: ivec5_t = [0; 5];
    let mut lBL: ivec5_t = [0; 5];
    let mut lBR: ivec5_t = [0; 5];
    let patchTerxels: c_int = (*(*this_).owner).GetTerxels();

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

    if (Part & PI_TOP) != 0 && !(*this_).mTLShader.is_null() {
        /*  float       d;

            d = DotProduct (backEnd.refdef.vieworg, mNormal[0]) - mDistance[0];

            if (d <= 0.0) */
        {
            CTRPatch_RecurseRender(this_, (*r_terrainTessellate).integer, &mut lBL, &mut lTR, &mut lTL);
        }
    }

    if (Part & PI_BOTTOM) != 0 && !(*this_).mBRShader.is_null() {
        /*  float       d;

            d = DotProduct (backEnd.refdef.vieworg, mNormal[1]) - mDistance[1];

            if (d >= 0.0) */
        {
            CTRPatch_RecurseRender(this_, (*r_terrainTessellate).integer, &mut lTR, &mut lBL, &mut lBR);
        }
    }
}

//
// At this point the patch is visible and at least part of it is below water level
//
// CTRPatch::RenderWaterVert
pub unsafe fn CTRPatch_RenderWaterVert(this_: *mut CTRPatch, x: c_int, y: c_int) -> c_int {
    let vert: *mut CTerVert;

    vert = (*this_).mRenderMap.add(
        x as usize + (y as usize * (*(*this_).owner).GetRealWidth() as usize),
    );

    if (*vert).tessRegistration == tess.registration {
        return (*vert).tessIndex;
    }
    tess.xyz[tess.numVertexes as usize][0] = (*vert).coords[0];
    tess.xyz[tess.numVertexes as usize][1] = (*vert).coords[1];
    tess.xyz[tess.numVertexes as usize][2] = (*(*this_).owner).GetWaterHeight();

    // *(ulong *)tess.vertexColors[tess.numVertexes] = 0xffffffff;
    *(tess.vertexColors[tess.numVertexes as usize].as_mut_ptr() as *mut c_ulong) = 0xffffffff;

    tess.texCoords[tess.numVertexes as usize][0][0] = (*vert).tex[0];
    tess.texCoords[tess.numVertexes as usize][0][1] = (*vert).tex[1];

    (*vert).tessIndex = tess.numVertexes;
    (*vert).tessRegistration = tess.registration;

    tess.numVertexes += 1;
    return (*vert).tessIndex;
}

// CTRPatch::RenderWater
pub unsafe fn CTRPatch_RenderWater(this_: *mut CTRPatch) {
    RB_CheckOverflow(4, 6);

    // Get the neighbouring patches
    let TL: c_int = CTRPatch_RenderWaterVert(this_, 0, 0);
    let TR: c_int = CTRPatch_RenderWaterVert(this_, (*(*this_).owner).GetTerxels(), 0);
    let BL: c_int = CTRPatch_RenderWaterVert(this_, 0, (*(*this_).owner).GetTerxels());
    let BR: c_int =
        CTRPatch_RenderWaterVert(this_, (*(*this_).owner).GetTerxels(), (*(*this_).owner).GetTerxels());

    // TL
    tess.indexes[tess.numIndexes as usize] = BL;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = TR;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = TL;
    tess.numIndexes += 1;

    // BR
    tess.indexes[tess.numIndexes as usize] = TR;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = BL;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = BR;
    tess.numIndexes += 1;
}

// CTRPatch::HasWater (const)
pub unsafe fn CTRPatch_HasWater(this_: *const CTRPatch) -> bool {
    (*(*this_).owner).SetRealWaterHeight(
        (*(*this_).owner).GetBaseWaterHeight() + (*r_terrainWaterOffset).integer,
    );
    return (*(*this_).common).GetMins()[2] < (*(*this_).owner).GetWaterHeight();
}

// bool CM_CullWorldBox (const cplane_t *frustum, const vec3pair_t bounds); — declared above.

// CTRPatch::SetVisibility
pub unsafe fn CTRPatch_SetVisibility(this_: *mut CTRPatch, visCheck: bool) {
    if visCheck {
        if DistanceSquared((*this_).mCenter, backEnd.refdef.vieworg) > TerrainDistanceCull {
            (*this_).misVisible = false;
        } else {
            // Set the visibility of the patch
            (*this_).misVisible = CM_CullWorldBox(
                backEnd.viewParms.frustum.as_ptr(),
                (*this_).GetBounds(),
            );
        }
    } else {
        (*this_).misVisible = true;
    }
}

/*
void CTRPatch::CalcNormal(void)
{
    CTerVert    *vert1, *vert2, *vert3;
    ivec5_t     TL, TR, BL, BR;
    vec3_t      v1, v2;

    VectorSet5(TL, 0, 0, TEXTURE_ALPHA_TL, -1, 0);
    VectorSet5(TR, owner->GetTerxels(), 0, TEXTURE_ALPHA_TR, -1, 0);
    VectorSet5(BL, 0, owner->GetTerxels(), TEXTURE_ALPHA_BL, -1, 0);
    VectorSet5(BR, owner->GetTerxels(), owner->GetTerxels(), TEXTURE_ALPHA_BR, -1, 0);

    vert1 = mRenderMap + (BL[1] * owner->GetRealWidth()) + BL[0];
    vert2 = mRenderMap + (TR[1] * owner->GetRealWidth()) + TR[0];
    vert3 = mRenderMap + (TL[1] * owner->GetRealWidth()) + TL[0];
    VectorSubtract(vert2->coords, vert1->coords, v1);
    VectorSubtract(vert3->coords, vert1->coords, v2);
    CrossProduct(v1, v2, mNormal[0]);
    VectorNormalize(mNormal[0]);
    mDistance[0] = DotProduct (vert1->coords, mNormal[0]);

    vert1 = mRenderMap + (BL[1] * owner->GetRealWidth()) + BL[0];
    vert2 = mRenderMap + (TR[1] * owner->GetRealWidth()) + TR[0];
    vert3 = mRenderMap + (BR[1] * owner->GetRealWidth()) + BR[0];
    VectorSubtract(vert2->coords, vert1->coords, v1);
    VectorSubtract(vert3->coords, vert1->coords, v2);
    CrossProduct(v1, v2, mNormal[1]);
    VectorNormalize(mNormal[1]);
    mDistance[1] = DotProduct (vert1->coords, mNormal[1]);
}
*/

//
// Reset all patches, recompute variance if needed
//
// CTRLandScape::Reset
pub unsafe fn CTRLandScape_Reset(this_: *mut CTRLandScape, visCheck: bool) {
    let x: c_int;
    let y: c_int;
    let mut patch: *mut CTRPatch;

    TerrainDistanceCull = tr.distanceCull + (*this_).mPatchSize;
    TerrainDistanceCull *= TerrainDistanceCull;

    // Go through the patches performing resets, compute variances, and linking.
    let mut y = (*this_).mPatchMiny;
    while y < (*this_).mPatchMaxy {
        let mut x = (*this_).mPatchMinx;
        while x < (*this_).mPatchMaxx {
            patch = (*this_).GetPatch(x, y);
            CTRPatch_SetVisibility(patch, visCheck);
            x += 1;
        }
        y += 1;
    }
}


//
// Render each patch of the landscape & adjust the frame variance.
//

// CTRLandScape::Render
pub unsafe fn CTRLandScape_Render(this_: *mut CTRLandScape) {
    let mut x: c_int;
    let mut y: c_int;
    let mut patch: *mut CTRPatch;
    let mut current: *mut TPatchInfo;
    let mut i: c_int;

    // Render all the visible patches
    current = (*this_).mSortedPatches;
    i = 0;
    while i < (*this_).mSortedCount {
        if (*(*current).mPatch).isVisible() {
            if tess.shader != (*current).mShader {
                RB_EndSurface();
                RB_BeginSurface((*current).mShader, TerrainFog);
            }
            CTRPatch_Render((*current).mPatch, (*current).mPart);
        }
        current = current.add(1);
        i += 1;
    }
    RB_EndSurface();

    // Render all the water for visible patches
    // Done as a separate iteration to reduce the number of tesses created
    if !(*this_).mWaterShader.is_null() && ((*this_).mWaterShader != tr.defaultShader) {
        RB_BeginSurface((*this_).mWaterShader, (*tr.world).globalFog);

        y = (*this_).mPatchMiny;
        while y < (*this_).mPatchMaxy {
            x = (*this_).mPatchMinx;
            while x < (*this_).mPatchMaxx {
                patch = (*this_).GetPatch(x, y);
                if (*patch).isVisible() && CTRPatch_HasWater(patch) {
                    CTRPatch_RenderWater(patch);
                }
                x += 1;
            }
            y += 1;
        }
        RB_EndSurface();
    }
}

// CTRLandScape::CalculateRegion
pub unsafe fn CTRLandScape_CalculateRegion(this_: *mut CTRLandScape) {
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let size: vec3_t;
    let offset: vec3_t;

    #[cfg(debug_assertions)]
    {
        (*this_).mCycleCount += 1;
    }
    size = (*this_).GetPatchSize();
    offset = (*this_).GetMins();

    mins[0] = backEnd.refdef.vieworg[0] - tr.distanceCull - (size[0] * 2.0_f32) - offset[0];
    mins[1] = backEnd.refdef.vieworg[1] - tr.distanceCull - (size[1] * 2.0_f32) - offset[1];

    maxs[0] = backEnd.refdef.vieworg[0] + tr.distanceCull + (size[0] * 2.0_f32) - offset[0];
    maxs[1] = backEnd.refdef.vieworg[1] + tr.distanceCull + (size[1] * 2.0_f32) - offset[1];

    (*this_).mPatchMinx =
        Com_Clamp(0.0_f32, (*this_).GetBlockWidth() as f32, floorf(mins[0] / size[0])) as c_int;
    (*this_).mPatchMaxx =
        Com_Clamp(0.0_f32, (*this_).GetBlockWidth() as f32, ceilf(maxs[0] / size[0])) as c_int;

    (*this_).mPatchMiny =
        Com_Clamp(0.0_f32, (*this_).GetBlockHeight() as f32, floorf(mins[1] / size[1])) as c_int;
    (*this_).mPatchMaxy =
        Com_Clamp(0.0_f32, (*this_).GetBlockHeight() as f32, ceilf(maxs[1] / size[1])) as c_int;
}

// CTRLandScape::CalculateRealCoords
pub unsafe fn CTRLandScape_CalculateRealCoords(this_: *mut CTRLandScape) {
    let mut x: c_int;
    let mut y: c_int;

    // Work out the real world coordinates of each heightmap entry
    y = 0;
    while y < (*this_).GetRealHeight() {
        x = 0;
        while x < (*this_).GetRealWidth() {
            let mut icoords: ivec3_t = [0; 3];
            let offset: usize;

            offset = (y as usize * (*this_).GetRealWidth() as usize) + x as usize;

            VectorSet(&mut icoords, x, y, (*(*this_).mRenderMap.add(offset)).height as c_int);
            VectorScaleVectorAdd(
                (*this_).GetMins(),
                icoords,
                (*this_).GetTerxelSize(),
                &mut (*(*this_).mRenderMap.add(offset)).coords,
            );
            x += 1;
        }
        y += 1;
    }
}

// CTRLandScape::CalculateNormals
pub unsafe fn CTRLandScape_CalculateNormals(this_: *mut CTRLandScape) {
    let mut x: c_int;
    let mut y: c_int;
    let mut offset: usize = 0;

    // Work out the normals for every face
    y = 0;
    while y < (*this_).GetHeight() {
        x = 0;
        while x < (*this_).GetWidth() {
            let mut vcenter: vec3_t = [0.0; 3];
            let mut vleft: vec3_t = [0.0; 3];

            offset = (y as usize * (*this_).GetRealWidth() as usize) + x as usize;

            VectorSubtract(
                &(*(*this_).mRenderMap.add(offset)).coords,
                &(*(*this_).mRenderMap.add(offset + 1)).coords,
                &mut vcenter,
            );
            VectorSubtract(
                &(*(*this_).mRenderMap.add(offset)).coords,
                &(*(*this_).mRenderMap.add(offset + (*this_).GetRealWidth() as usize)).coords,
                &mut vleft,
            );

            CrossProduct(&vcenter, &vleft, &mut (*(*this_).mRenderMap.add(offset)).normal);
            VectorNormalize(&mut (*(*this_).mRenderMap.add(offset)).normal);
            x += 1;
        }
        // Duplicate right edge condition
        VectorCopy(
            &(*(*this_).mRenderMap.add(offset)).normal,
            &mut (*(*this_).mRenderMap.add(offset + 1)).normal,
        );
        y += 1;
    }
    // Duplicate bottom line
    offset = (*this_).GetHeight() as usize * (*this_).GetRealWidth() as usize;
    x = 0;
    while x < (*this_).GetRealWidth() {
        VectorCopy(
            &(*(*this_).mRenderMap.add(offset - (*this_).GetRealWidth() as usize + x as usize)).normal,
            &mut (*(*this_).mRenderMap.add(offset + x as usize)).normal,
        );
        x += 1;
    }
}

// int R_LightForPoint( vec3_t point, vec3_t ambientLight, vec3_t directedLight, vec3_t lightDir ); — declared above.

// CTRLandScape::CalculateLighting
pub unsafe fn CTRLandScape_CalculateLighting(this_: *mut CTRLandScape) {
    let mut x: c_int;
    let mut y: c_int;
    let mut offset: usize = 0;
    let common: *const CCMLandScape = (*this_).common;

    // Work out the vertex normal (average of every attached face normal) and apply to the direction of the light
    y = 0;
    while y < (*this_).GetHeight() {
        x = 0;
        while x < (*this_).GetWidth() {
            let mut ambient: vec3_t = [0.0; 3];
            let mut directed: vec3_t = [0.0; 3];
            let mut direction: vec3_t = [0.0; 3];
            let mut total: vec3_t = [0.0; 3];
            let mut tint: vec3_t = [0.0; 3];
            let dp: vec_t;

            offset = (y as usize * (*this_).GetRealWidth() as usize) + x as usize;

            // Work out average normal
            VectorCopy(
                &(*(*this_).GetRenderMap(x, y)).normal,
                &mut total,
            );
            VectorAdd(&total, &(*(*this_).GetRenderMap(x + 1, y)).normal, &mut total);
            VectorAdd(&total, &(*(*this_).GetRenderMap(x + 1, y + 1)).normal, &mut total);
            VectorAdd(&total, &(*(*this_).GetRenderMap(x, y + 1)).normal, &mut total);
            VectorNormalize(&mut total);

            if R_LightForPoint(
                &(*(*this_).mRenderMap.add(offset)).coords,
                &mut ambient,
                &mut directed,
                &mut direction,
            ) == 0
            {
                let t = (255 >> tr.overbrightBits) as byte;
                (*(*this_).mRenderMap.add(offset)).tint[0] = t;
                (*(*this_).mRenderMap.add(offset)).tint[1] = t;
                (*(*this_).mRenderMap.add(offset)).tint[2] = t;
                (*(*this_).mRenderMap.add(offset)).tint[3] = 255;
                x += 1;
                continue;
            }

            if (*(*this_).mRenderMap.add(offset)).coords[2] < (*common).GetBaseWaterHeight() {
                VectorScale(&ambient, 0.75_f32, &mut ambient);
            }

            // Both normalised, so -1.0 < dp < 1.0
            let dp = Com_Clamp(0.0_f32, 1.0_f32, DotProduct(&direction, &total));
            let dp = powf(dp, 3.0_f32);
            VectorScale(&ambient, (1.0_f64 - dp as f64) as f32 * 0.5_f32, &mut ambient);
            VectorMA(&ambient, dp, &directed, &mut tint);

            // rjr - in R_SetupEntityLighting, ambient light is automatically increased by 32, so do it here to match
            // rjr - decided to disable both the lighting boost automatically in there as well as here.
            (*(*this_).mRenderMap.add(offset)).tint[0] =
                ((Com_Clamp(0.0_f32, 255.0_f32, tint[0]) as u8 as c_int) >> tr.overbrightBits) as u8;
            (*(*this_).mRenderMap.add(offset)).tint[1] =
                ((Com_Clamp(0.0_f32, 255.0_f32, tint[1]) as u8 as c_int) >> tr.overbrightBits) as u8;
            (*(*this_).mRenderMap.add(offset)).tint[2] =
                ((Com_Clamp(0.0_f32, 255.0_f32, tint[2]) as u8 as c_int) >> tr.overbrightBits) as u8;
            (*(*this_).mRenderMap.add(offset)).tint[3] = 0xff;
            x += 1;
        }
        (*(*this_).mRenderMap.add(offset + 1)).tint[0] = (*(*this_).mRenderMap.add(offset)).tint[0];
        (*(*this_).mRenderMap.add(offset + 1)).tint[1] = (*(*this_).mRenderMap.add(offset)).tint[1];
        (*(*this_).mRenderMap.add(offset + 1)).tint[2] = (*(*this_).mRenderMap.add(offset)).tint[2];
        (*(*this_).mRenderMap.add(offset + 1)).tint[3] = 0xff;
        y += 1;
    }
    // Duplicate bottom line
    offset = (*this_).GetHeight() as usize * (*this_).GetRealWidth() as usize;
    x = 0;
    while x < (*this_).GetRealWidth() {
        (*(*this_).mRenderMap.add(offset + x as usize)).tint[0] =
            (*(*this_).mRenderMap.add(offset - (*this_).GetRealWidth() as usize + x as usize)).tint[0];
        (*(*this_).mRenderMap.add(offset + x as usize)).tint[1] =
            (*(*this_).mRenderMap.add(offset - (*this_).GetRealWidth() as usize + x as usize)).tint[1];
        (*(*this_).mRenderMap.add(offset + x as usize)).tint[2] =
            (*(*this_).mRenderMap.add(offset - (*this_).GetRealWidth() as usize + x as usize)).tint[2];
        (*(*this_).mRenderMap.add(offset + x as usize)).tint[3] = 0xff;
        x += 1;
    }
}

// CTRLandScape::CalculateTextureCoords
pub unsafe fn CTRLandScape_CalculateTextureCoords(this_: *mut CTRLandScape) {
    let mut x: c_int;
    let mut y: c_int;

    y = 0;
    while y < (*this_).GetRealHeight() {
        x = 0;
        while x < (*this_).GetRealWidth() {
            let offset: usize = (y as usize * (*this_).GetRealWidth() as usize) + x as usize;

            (*(*this_).mRenderMap.add(offset)).tex[0] =
                x as f32 * (*this_).mTextureScale * (*this_).GetTerxelSize()[0];
            (*(*this_).mRenderMap.add(offset)).tex[1] =
                y as f32 * (*this_).mTextureScale * (*this_).GetTerxelSize()[1];
            x += 1;
        }
        y += 1;
    }
}

// CTRLandScape::SetShaders
pub unsafe fn CTRLandScape_SetShaders(this_: *mut CTRLandScape, height: c_int, shader: qhandle_t) {
    let mut i: c_int;

    i = height;
    while shader != 0 && i < HEIGHT_RESOLUTION as c_int {
        if (*this_).mHeightDetails[i as usize].GetShader() == 0 {
            (*this_).mHeightDetails[i as usize].SetShader(shader);
        }
        i += 1;
    }
}

// CTRLandScape::LoadTerrainDef
pub unsafe fn CTRLandScape_LoadTerrainDef(this_: *mut CTRLandScape, td: *const c_char) {
    #[cfg(not(feature = "pre_release_demo"))]
    {
        let mut terrainDef: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut parse: CGenericParser2 = core::mem::zeroed();
        let basegroup: *mut CGPGroup;
        let mut classes: *mut CGPGroup;
        let mut items: *mut CGPGroup;

        Com_sprintf(
            terrainDef.as_mut_ptr(),
            MAX_QPATH,
            b"ext_data/RMG/%s.terrain\0".as_ptr() as *const c_char,
            td,
        );
        Com_Printf(
            b"R_Terrain: Loading and parsing terrainDef %s.....\n\0".as_ptr() as *const c_char,
            td,
        );

        (*this_).mWaterShader = core::ptr::null_mut();
        (*this_).mFlatShader = 0;

        if Com_ParseTextFile(terrainDef.as_ptr(), &mut parse) == qfalse {
            Com_sprintf(
                terrainDef.as_mut_ptr(),
                MAX_QPATH,
                b"ext_data/arioche/%s.terrain\0".as_ptr() as *const c_char,
                td,
            );
            if Com_ParseTextFile(terrainDef.as_ptr(), &mut parse) == qfalse {
                Com_Printf(
                    b"Could not open %s\n\0".as_ptr() as *const c_char,
                    terrainDef.as_ptr(),
                );
                return;
            }
        }
        // The whole file....
        basegroup = parse.GetBaseParseGroup();

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
                    height = atol((*items).FindPairValue(
                        b"height\0".as_ptr() as *const c_char,
                        b"0\0".as_ptr() as *const c_char,
                    )) as c_int;

                    // Shader for this height
                    shaderName = (*items).FindPairValue(
                        b"shader\0".as_ptr() as *const c_char,
                        b"\0".as_ptr() as *const c_char,
                    );
                    if *shaderName != 0 {
                        shader = RE_RegisterShader(shaderName);
                        if shader != 0 {
                            CTRLandScape_SetShaders(this_, height, shader);
                        }
                    }
                } else if stricmp(type_, b"water\0".as_ptr() as *const c_char) == 0 {
                    (*this_).mWaterShader = R_GetShaderByHandle(RE_RegisterShader(
                        (*items).FindPairValue(
                            b"shader\0".as_ptr() as *const c_char,
                            b"\0".as_ptr() as *const c_char,
                        ),
                    ));
                } else if stricmp(type_, b"flattexture\0".as_ptr() as *const c_char) == 0 {
                    (*this_).mFlatShader = RE_RegisterShader((*items).FindPairValue(
                        b"shader\0".as_ptr() as *const c_char,
                        b"\0".as_ptr() as *const c_char,
                    ));
                }

                items = (*items).GetNext() as *mut CGPGroup;
            }
            classes = (*classes).GetNext() as *mut CGPGroup;
        }

        Com_ParseTextFileDestroy(parse);
    }
    // #endif // PRE_RELEASE_DEMO
}

// qhandle_t R_CreateBlendedShader(qhandle_t a, qhandle_t b, qhandle_t c, bool surfaceSprites ); — declared above.

// CTRLandScape::GetBlendedShader
pub unsafe fn CTRLandScape_GetBlendedShader(
    this_: *mut CTRLandScape,
    a: qhandle_t,
    b: qhandle_t,
    c: qhandle_t,
    surfaceSprites: bool,
) -> qhandle_t {
    let blended: qhandle_t;

    // Special case single pass shader
    if (a == b) && (a == c) {
        return a;
    }

    blended = R_CreateBlendedShader(a, b, c, surfaceSprites);
    return blended;
}

unsafe extern "C" fn ComparePatchInfo(
    arg1: *const c_void,
    arg2: *const c_void,
) -> c_int {
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

    if (s1 as usize) < (s2 as usize) {
        return -1;
    } else if (s1 as usize) > (s2 as usize) {
        return 1;
    }

    return 0;
}

// CTRLandScape::CalculateShaders
pub unsafe fn CTRLandScape_CalculateShaders(this_: *mut CTRLandScape) {
    #[cfg(not(feature = "pre_release_demo"))]
    {
        let mut x: c_int;
        let mut y: c_int;
        let width: c_int;
        let height: c_int;
        let mut offset: usize;
        // int offsets[4];
        let mut handles: [qhandle_t; 4] = [0; 4];
        let mut patch: *mut CTRPatch;
        let common: *const CCMLandScape = (*this_).common;
        let shaders: *mut qhandle_t;
        let mut current: *mut TPatchInfo = (*this_).mSortedPatches;

        width = (*this_).GetWidth() / (*common).GetTerxels();
        height = (*this_).GetHeight() / (*common).GetTerxels();

        // Porting note: C++ `new qhandle_t[(width+1)*(height+1)]` — using Vec for heap allocation;
        // `delete[] shaders` at end becomes drop via the Vec going out of scope.
        let mut shaders_vec: Vec<qhandle_t> =
            vec![0; ((width + 1) * (height + 1)) as usize];
        let shaders: *mut qhandle_t = shaders_vec.as_mut_ptr();

        // On the first pass determine all of the shaders for the entire
        // terrain assuming no flat ground
        offset = 0;
        y = 0;
        while y < height + 1 {
            if y <= height {
                offset = ((*common).GetTerxels() * y * (*this_).GetRealWidth()) as usize;
            } else {
                offset = ((*common).GetTerxels() * (y - 1) * (*this_).GetRealWidth()) as usize;
                offset += (*this_).GetRealWidth() as usize;
            }

            x = 0;
            while x < width + 1 {
                // Save the shader
                *shaders.add((y * width + x) as usize) =
                    (*this_).GetHeightDetail((*(*this_).mRenderMap.add(offset)).height).GetShader();
                x += 1;
                offset += (*common).GetTerxels() as usize;
            }
            y += 1;
        }

        // On the second pass determine flat ground and replace the shader
        // at that point with the flat ground shader
        let flattenMap: *mut byte = (*common).GetFlattenMap();
        if (*this_).mFlatShader != 0 && !flattenMap.is_null() {
            y = 1;
            while y < height {
                x = 1;
                while x < width {
                    let mut off: usize;
                    let mut xx: c_int;
                    let mut yy: c_int;
                    let mut flat: bool = false;

                    off = (x * (*common).GetTerxels()) as usize;
                    off += (y * (*common).GetTerxels() * (*this_).GetRealWidth()) as usize;

                    off -= (*this_).GetRealWidth() as usize;
                    off -= 1;

                    yy = 0;
                    while yy < 3 && !flat {
                        xx = 0;
                        while xx < 3 && !flat {
                            if *flattenMap.add(off + xx as usize) & 0x80 != 0 {
                                flat = true;
                                break;
                            }
                            xx += 1;
                        }

                        off += (*this_).GetRealWidth() as usize;
                        yy += 1;
                    }
                    // This shader is now a flat shader
                    if flat {
                        *shaders.add((y * width + x) as usize) = (*this_).mFlatShader;
                    }

                    #[cfg(debug_assertions)]
                    {
                        OutputDebugString(va(
                            b"Flat Area:  %f %f\n\0".as_ptr() as *const c_char,
                            (*this_).GetMins()[0]
                                + ((*this_).GetMaxs()[0] - (*this_).GetMins()[0]) / width as f32
                                    * x as f32,
                            (*this_).GetMins()[1]
                                + ((*this_).GetMaxs()[1] - (*this_).GetMins()[1]) / height as f32
                                    * y as f32,
                        ));
                    }
                    x += 1;
                }
                y += 1;
            }
        }

        // Now that the shaders have been determined, set them for each patch
        patch = (*this_).mTRPatches;
        (*this_).mSortedCount = 0;
        y = 0;
        while y < height {
            x = 0;
            while x < width {
                let mut surfaceSprites: bool = true;

                /*
                handles[INDEX_TL] = shaders[ (x + y) * width ];
                handles[INDEX_TR] = shaders[ ((x + 1) + y) * width ];
                handles[INDEX_BL] = shaders[ (x + (y + 1)) * width ];
                handles[INDEX_BR] = shaders[ ((x + 1) + (y + 1)) * width ];
                */
                handles[INDEX_TL as usize] = *shaders.add((x + y * width) as usize);
                handles[INDEX_TR as usize] = *shaders.add((x + 1 + y * width) as usize);
                handles[INDEX_BL as usize] = *shaders.add((x + (y + 1) * width) as usize);
                handles[INDEX_BR as usize] = *shaders.add((x + 1 + (y + 1) * width) as usize);

                if handles[INDEX_TL as usize] == (*this_).mFlatShader
                    || handles[INDEX_TR as usize] == (*this_).mFlatShader
                    || handles[INDEX_BL as usize] == (*this_).mFlatShader
                    || handles[INDEX_BR as usize] == (*this_).mFlatShader
                {
                    surfaceSprites = false;
                }

                (*patch).SetTLShader(CTRLandScape_GetBlendedShader(
                    this_,
                    handles[INDEX_TR as usize],
                    handles[INDEX_BL as usize],
                    handles[INDEX_TL as usize],
                    surfaceSprites,
                ));
                (*current).mPatch = patch;
                (*current).mShader = (*patch).GetTLShader();
                (*current).mPart = PI_TOP;

                (*patch).SetBRShader(CTRLandScape_GetBlendedShader(
                    this_,
                    handles[INDEX_TR as usize],
                    handles[INDEX_BL as usize],
                    handles[INDEX_BR as usize],
                    surfaceSprites,
                ));
                if (*patch).GetBRShader() == (*current).mShader {
                    (*current).mPart |= PI_BOTTOM;
                } else {
                    (*this_).mSortedCount += 1;
                    current = current.add(1);

                    (*current).mPatch = patch;
                    (*current).mShader = (*patch).GetBRShader();
                    (*current).mPart = PI_BOTTOM;
                }
                (*this_).mSortedCount += 1;
                current = current.add(1);

                patch = patch.add(1);
                x += 1;
            }
            y += 1;
        }

        // Cleanup our temporary array
        // delete[] shaders; — handled by shaders_vec drop at end of block

        qsort(
            (*this_).mSortedPatches as *mut c_void,
            (*this_).mSortedCount as usize,
            core::mem::size_of::<TPatchInfo>(),
            ComparePatchInfo,
        );
    }
    // #endif // PRE_RELEASE_DEMO
}

// CTRPatch::SetRenderMap
pub unsafe fn CTRPatch_SetRenderMap(this_: *mut CTRPatch, x: c_int, y: c_int) {
    (*this_).mRenderMap = (*(*this_).localowner).GetRenderMap(x, y);
}

// InitRendererPatches
pub unsafe extern "C" fn InitRendererPatches(patch: *mut CCMPatch, userdata: *mut c_void) {
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
    CTRPatch_SetRenderMap(localpatch, tx, ty);
    (*localpatch).SetCenter();
    // localpatch->CalcNormal();
}

// CTRLandScape::CopyHeightMap
pub unsafe fn CTRLandScape_CopyHeightMap(this_: *mut CTRLandScape) {
    let common: *const CCMLandScape = (*this_).GetCommon();
    let heightMap: *const byte = (*common).GetHeightMap();
    let renderMap: *mut CTerVert = (*this_).mRenderMap;
    let mut i: c_int;

    i = 0;
    while i < (*common).GetRealArea() {
        (*renderMap.add(i as usize)).height = *heightMap.add(i as usize);
        i += 1;
    }
}

// CTRLandScape::~CTRLandScape
pub unsafe fn CTRLandScape_dtor(this_: *mut CTRLandScape) {
    if !(*this_).mTRPatches.is_null() {
        Z_Free((*this_).mTRPatches as *mut c_void);
        (*this_).mTRPatches = core::ptr::null_mut();
    }
    if !(*this_).mSortedPatches.is_null() {
        Z_Free((*this_).mSortedPatches as *mut c_void);
        (*this_).mSortedPatches = core::ptr::null_mut();
    }
    if !(*this_).mRenderMap.is_null() {
        Z_Free((*this_).mRenderMap as *mut c_void);
        (*this_).mRenderMap = core::ptr::null_mut();
    }
}

// CCMLandScape *CM_RegisterTerrain(const char *config, bool server); — declared above.

// qhandle_t R_GetShaderByNum(int shaderNum, world_t &worldData); — declared above.

// CTRLandScape::CTRLandScape(const char *configstring)
// Porting note: C++ constructor — translated as a free unsafe fn taking an already-allocated this_.
// The caller is responsible for allocation (see RE_InitRendererTerrain).
pub unsafe fn CTRLandScape_ctor(this_: *mut CTRLandScape, configstring: *const c_char) {
    #[cfg(not(feature = "pre_release_demo"))]
    {
        let shaderNum: c_int;
        let common: *const CCMLandScape;

        core::ptr::write_bytes(this_ as *mut u8, 0u8, core::mem::size_of::<CTRLandScape>());

        // Sets up the common aspects of the terrain
        common = CM_RegisterTerrain(configstring, false);
        (*this_).SetCommon(common);

        tr.landScape.landscape = this_;

        (*this_).mTextureScale = atof(Info_ValueForKey(
            configstring,
            b"texturescale\0".as_ptr() as *const c_char,
        )) as f32
            / (*common).GetTerxels() as f32;
        CTRLandScape_LoadTerrainDef(
            this_,
            Info_ValueForKey(configstring, b"terrainDef\0".as_ptr() as *const c_char),
        );

        // To normalise the variance value to a reasonable number
        (*this_).mScalarSize = VectorLengthSquared(&(*common).GetSize());

        // Calculate and set variance depth
        (*this_).mMaxNode = (Q_log2((*common).GetTerxels()) << 1) - 1;

        // Allocate space for the renderer specific data
        (*this_).mRenderMap = Z_Malloc(
            core::mem::size_of::<CTerVert>() * (*common).GetRealArea() as usize,
            TAG_R_TERRAIN,
            qfalse,
        ) as *mut CTerVert;

        // Copy byte heightmap to rendermap to speed up calcs
        CTRLandScape_CopyHeightMap(this_);

        // Calculate the real world location for each heightmap entry
        CTRLandScape_CalculateRealCoords(this_);

        // Calculate the normal of each terxel
        CTRLandScape_CalculateNormals(this_);

        // Calculate modulation values for the heightmap
        CTRLandScape_CalculateLighting(this_);

        // Calculate texture coords (not projected - real)
        CTRLandScape_CalculateTextureCoords(this_);

        Com_Printf(b"R_Terrain: Creating renderer patches.....\n\0".as_ptr() as *const c_char);
        // Initialise all terrain patches
        (*this_).mTRPatches = Z_Malloc(
            core::mem::size_of::<CTRPatch>() * (*common).GetBlockCount() as usize,
            TAG_R_TERRAIN,
            qfalse,
        ) as *mut CTRPatch;

        (*this_).mSortedCount = 2 * (*common).GetBlockCount();
        (*this_).mSortedPatches = Z_Malloc(
            core::mem::size_of::<TPatchInfo>() * (*this_).mSortedCount as usize,
            TAG_R_TERRAIN,
            qfalse,
        ) as *mut TPatchInfo;

        CM_TerrainPatchIterate(common, InitRendererPatches, this_ as *mut c_void);

        // Calculate shaders dependent on the .terrain file
        CTRLandScape_CalculateShaders(this_);

        // Get the contents shader
        shaderNum =
            atol(Info_ValueForKey(configstring, b"shader\0".as_ptr() as *const c_char)) as c_int;

        (*this_).mShader = R_GetShaderByHandle(R_GetShaderByNum(shaderNum, tr.world));

        (*this_).mPatchSize = VectorLength(&(*common).GetPatchSize());

        #[cfg(debug_assertions)]
        {
            (*this_).mCycleCount = 0;
        }
    }
    // #endif // PRE_RELEASE_DEMO
}

// ---------------------------------------------------------------------

pub unsafe fn RB_SurfaceTerrain(surf: *mut surfaceInfo_t) {
    let ls: *mut srfTerrain_t = surf as *mut srfTerrain_t;
    let landscape: *mut CTRLandScape = (*ls).landscape;

    TerrainFog = (*tr.world).globalFog;

    CTRLandScape_CalculateRegion(landscape);
    CTRLandScape_Reset(landscape, true);
    // landscape->Tessellate();
    CTRLandScape_Render(landscape);
}

pub unsafe fn R_CalcTerrainVisBounds(landscape: *mut CTRLandScape) {
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

pub unsafe fn R_AddTerrainSurfaces() {
    let landscape: *mut CTRLandScape;

    if (*r_drawTerrain).integer == 0 || (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        return;
    }

    landscape = tr.landScape.landscape;
    if !landscape.is_null() {
        R_AddDrawSurf(
            (&tr.landScape) as *const _ as *const surfaceType_t,
            (*landscape).GetShader(),
            0,
            qfalse,
        );
        R_CalcTerrainVisBounds(landscape);
    }
}

pub unsafe fn RE_InitRendererTerrain(info: *const c_char) {
    let ls: *mut CTRLandScape;

    if info.is_null() || *info == 0 {
        Com_Printf(
            b"RE_RegisterTerrain: NULL name\n\0".as_ptr() as *const c_char,
        );
        return;
    }

    Com_Printf(b"R_Terrain: Creating RENDERER data.....\n\0".as_ptr() as *const c_char);

    // Create and register a new landscape structure
    // Porting note: C++ `new CTRLandScape(info)` — allocate via Z_Malloc then run constructor.
    ls = Z_Malloc(
        core::mem::size_of::<CTRLandScape>(),
        TAG_R_TERRAIN,
        qfalse,
    ) as *mut CTRLandScape;
    CTRLandScape_ctor(ls, info);
}

pub unsafe fn R_TerrainInit() {
    let mut i: c_int;

    i = 0;
    while i < MAX_TERRAINS as c_int {
        tr.landScape.surfaceType = SF_TERRAIN;
        tr.landScape.landscape = core::ptr::null_mut();
        i += 1;
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

// void CM_ShutdownTerrain( thandle_t terrainId); — declared above.

pub unsafe fn R_TerrainShutdown() {
    let ls: *mut CTRLandScape;

    // Com_Printf("R_Terrain: Shutting down RENDERER terrain.....\n");
    ls = tr.landScape.landscape;
    if !ls.is_null() {
        CM_ShutdownTerrain(0);
        // delete ls — call destructor then free
        CTRLandScape_dtor(ls);
        Z_Free(ls as *mut c_void);
        tr.landScape.landscape = core::ptr::null_mut();
    }
}

// end
