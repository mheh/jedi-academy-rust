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

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
use crate::codemp::qcommon::exe_headers_h::*;
// tr_surf.c
// #include "tr_local.h"
use crate::codemp::renderer::tr_local_h::*;

use core::ffi::*;
use core::ptr::{addr_of, addr_of_mut};

/*

  THIS ENTIRE FILE IS BACK END

backEnd.currentEntity will be valid.

Tess_Begin has already been called for the surface's shader.

The modelview matrix will be set.

It is safe to actually issue drawing commands here if you don't want to
use the shader system.
*/


//============================================================================

// #define NUM_CYLINDER_SEGMENTS 32
const NUM_CYLINDER_SEGMENTS: usize = 32;

// #define LIGHTNING_RECURSION_LEVEL 1 // was 2
const LIGHTNING_RECURSION_LEVEL: c_int = 1; // was 2

static mut sh1: vec3_t = [0.0_f32; 3];
static mut sh2: vec3_t = [0.0_f32; 3];
static mut f_count: f32 = 0.0_f32;

/*
==============
RB_CheckOverflow
==============
*/
pub unsafe fn RB_CheckOverflow(verts: c_int, indexes: c_int) {
    if tess.shader == tr.shadowShader {
        if tess.numVertexes + verts < SHADER_MAX_VERTEXES / 2
            && tess.numIndexes + indexes < SHADER_MAX_INDEXES
        {
            return;
        }
    } else if tess.numVertexes + verts < SHADER_MAX_VERTEXES
        && tess.numIndexes + indexes < SHADER_MAX_INDEXES
    {
        return;
    }

    RB_EndSurface();

    if verts >= SHADER_MAX_VERTEXES {
        Com_Error(
            ERR_DROP,
            b"RB_CheckOverflow: verts > MAX (%d > %d)\0".as_ptr() as *const c_char,
            verts,
            SHADER_MAX_VERTEXES,
        );
    }
    if indexes >= SHADER_MAX_INDEXES {
        Com_Error(
            ERR_DROP,
            b"RB_CheckOverflow: indices > MAX (%d > %d)\0".as_ptr() as *const c_char,
            indexes,
            SHADER_MAX_INDEXES,
        );
    }

    RB_BeginSurface(tess.shader, tess.fogNum);
}


/*
==============
RB_AddQuadStampExt
==============
*/
pub unsafe fn RB_AddQuadStampExt(
    origin: vec3_t,
    left: vec3_t,
    up: vec3_t,
    color: *mut byte,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
) {
    let mut normal: vec3_t = [0.0; 3];
    let ndx: c_int;

    RB_CheckOverflow(4, 6);

    ndx = tess.numVertexes;

    // triangle indexes for a simple quad
    tess.indexes[tess.numIndexes as usize] = ndx;
    tess.indexes[tess.numIndexes as usize + 1] = ndx + 1;
    tess.indexes[tess.numIndexes as usize + 2] = ndx + 3;

    tess.indexes[tess.numIndexes as usize + 3] = ndx + 3;
    tess.indexes[tess.numIndexes as usize + 4] = ndx + 1;
    tess.indexes[tess.numIndexes as usize + 5] = ndx + 2;

    tess.xyz[ndx as usize][0] = origin[0] + left[0] + up[0];
    tess.xyz[ndx as usize][1] = origin[1] + left[1] + up[1];
    tess.xyz[ndx as usize][2] = origin[2] + left[2] + up[2];

    tess.xyz[ndx as usize + 1][0] = origin[0] - left[0] + up[0];
    tess.xyz[ndx as usize + 1][1] = origin[1] - left[1] + up[1];
    tess.xyz[ndx as usize + 1][2] = origin[2] - left[2] + up[2];

    tess.xyz[ndx as usize + 2][0] = origin[0] - left[0] - up[0];
    tess.xyz[ndx as usize + 2][1] = origin[1] - left[1] - up[1];
    tess.xyz[ndx as usize + 2][2] = origin[2] - left[2] - up[2];

    tess.xyz[ndx as usize + 3][0] = origin[0] + left[0] - up[0];
    tess.xyz[ndx as usize + 3][1] = origin[1] + left[1] - up[1];
    tess.xyz[ndx as usize + 3][2] = origin[2] + left[2] - up[2];


    // constant normal all the way around
    VectorSubtract(&vec3_origin, &backEnd.viewParms.ori.axis[0], &mut normal);

    tess.normal[ndx as usize][0] = normal[0];
    tess.normal[ndx as usize + 1][0] = normal[0];
    tess.normal[ndx as usize + 2][0] = normal[0];
    tess.normal[ndx as usize + 3][0] = normal[0];
    tess.normal[ndx as usize][1] = normal[1];
    tess.normal[ndx as usize + 1][1] = normal[1];
    tess.normal[ndx as usize + 2][1] = normal[1];
    tess.normal[ndx as usize + 3][1] = normal[1];
    tess.normal[ndx as usize][2] = normal[2];
    tess.normal[ndx as usize + 1][2] = normal[2];
    tess.normal[ndx as usize + 2][2] = normal[2];
    tess.normal[ndx as usize + 3][2] = normal[2];

    // standard square texture coordinates
    tess.texCoords[ndx as usize][0][0] = s1;
    tess.texCoords[ndx as usize][1][0] = s1;
    tess.texCoords[ndx as usize][0][1] = t1;
    tess.texCoords[ndx as usize][1][1] = t1;

    tess.texCoords[ndx as usize + 1][0][0] = s2;
    tess.texCoords[ndx as usize + 1][1][0] = s2;
    tess.texCoords[ndx as usize + 1][0][1] = t1;
    tess.texCoords[ndx as usize + 1][1][1] = t1;

    tess.texCoords[ndx as usize + 2][0][0] = s2;
    tess.texCoords[ndx as usize + 2][1][0] = s2;
    tess.texCoords[ndx as usize + 2][0][1] = t2;
    tess.texCoords[ndx as usize + 2][1][1] = t2;

    tess.texCoords[ndx as usize + 3][0][0] = s1;
    tess.texCoords[ndx as usize + 3][1][0] = s1;
    tess.texCoords[ndx as usize + 3][0][1] = t2;
    tess.texCoords[ndx as usize + 3][1][1] = t2;

    // constant color all the way around
    // should this be identity and let the shader specify from entity?
    let color_val: c_uint = *(color as *const c_uint);
    *(tess.vertexColors[ndx as usize].as_mut_ptr() as *mut c_uint) = color_val;
    *(tess.vertexColors[ndx as usize + 1].as_mut_ptr() as *mut c_uint) = color_val;
    *(tess.vertexColors[ndx as usize + 2].as_mut_ptr() as *mut c_uint) = color_val;
    *(tess.vertexColors[ndx as usize + 3].as_mut_ptr() as *mut c_uint) = color_val;


    tess.numVertexes += 4;
    tess.numIndexes += 6;
}

/*
==============
RB_AddQuadStamp
==============
*/
pub unsafe fn RB_AddQuadStamp(origin: vec3_t, left: vec3_t, up: vec3_t, color: *mut byte) {
    RB_AddQuadStampExt(origin, left, up, color, 0.0, 0.0, 1.0, 1.0);
}

/*
==============
RB_SurfaceSprite
==============
*/
unsafe fn RB_SurfaceSprite() {
    let mut left: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let radius: f32;

    // calculate the xyz locations for the four corners
    radius = (*backEnd.currentEntity).e.radius;
    if (*backEnd.currentEntity).e.rotation == 0.0 {
        VectorScale(&backEnd.viewParms.ori.axis[1], radius, &mut left);
        VectorScale(&backEnd.viewParms.ori.axis[2], radius, &mut up);
    } else {
        let s: f32;
        let c: f32;
        let ang: f32;

        ang = M_PI * (*backEnd.currentEntity).e.rotation / 180.0;
        s = sin(ang as c_double) as f32;
        c = cos(ang as c_double) as f32;

        VectorScale(&backEnd.viewParms.ori.axis[1], c * radius, &mut left);
        VectorMA(&left.clone(), -s * radius, &backEnd.viewParms.ori.axis[2], &mut left);

        VectorScale(&backEnd.viewParms.ori.axis[2], c * radius, &mut up);
        VectorMA(&up.clone(), s * radius, &backEnd.viewParms.ori.axis[1], &mut up);
    }
    if backEnd.viewParms.isMirror != 0 {
        VectorSubtract(&vec3_origin, &left.clone(), &mut left);
    }

    RB_AddQuadStamp(
        (*backEnd.currentEntity).e.origin,
        left,
        up,
        (*backEnd.currentEntity).e.shaderRGBA.as_mut_ptr(),
    );
}


/*
=======================
RB_SurfaceOrientedQuad
=======================
*/
unsafe fn RB_SurfaceOrientedQuad() {
    let mut left: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let radius: f32;

    // calculate the xyz locations for the four corners
    radius = (*backEnd.currentEntity).e.radius;
    //	MakeNormalVectors( backEnd.currentEntity->e.axis[0], left, up );
    VectorCopy(&(*backEnd.currentEntity).e.axis[1], &mut left);
    VectorCopy(&(*backEnd.currentEntity).e.axis[2], &mut up);

    if (*backEnd.currentEntity).e.rotation == 0.0 {
        VectorScale(&left.clone(), radius, &mut left);
        VectorScale(&up.clone(), radius, &mut up);
    } else {
        let mut tempLeft: vec3_t = [0.0; 3];
        let mut tempUp: vec3_t = [0.0; 3];
        let s: f32;
        let c: f32;
        let ang: f32;

        ang = M_PI * (*backEnd.currentEntity).e.rotation / 180.0;
        s = sin(ang as c_double) as f32;
        c = cos(ang as c_double) as f32;

        // Use a temp so we don't trash the values we'll need later
        VectorScale(&left, c * radius, &mut tempLeft);
        VectorMA(&tempLeft.clone(), -s * radius, &up, &mut tempLeft);

        VectorScale(&up, c * radius, &mut tempUp);
        VectorMA(&tempUp.clone(), s * radius, &left, &mut up); // no need to use the temp anymore, so copy into the dest vector ( up )

        // This was copied for safekeeping, we're done, so we can move it back to left
        VectorCopy(&tempLeft, &mut left);
    }

    if backEnd.viewParms.isMirror != 0 {
        VectorSubtract(&vec3_origin, &left.clone(), &mut left);
    }

    RB_AddQuadStamp(
        (*backEnd.currentEntity).e.origin,
        left,
        up,
        (*backEnd.currentEntity).e.shaderRGBA.as_mut_ptr(),
    );
}

/*
=============
RB_SurfacePolychain
=============
*/
pub unsafe fn RB_SurfacePolychain(p: *mut srfPoly_t) {
    let mut i: c_int;
    let mut numv: c_int;

    RB_CheckOverflow((*p).numVerts, 3 * ((*p).numVerts - 2));

    // fan triangles into the tess array
    numv = tess.numVertexes;
    i = 0;
    while i < (*p).numVerts {
        VectorCopy(
            &(*p).verts[i as usize].xyz,
            &mut *(tess.xyz[numv as usize].as_mut_ptr() as *mut vec3_t),
        );
        tess.texCoords[numv as usize][0][0] = (*p).verts[i as usize].st[0];
        tess.texCoords[numv as usize][0][1] = (*p).verts[i as usize].st[1];
        *(tess.vertexColors[numv as usize].as_mut_ptr() as *mut c_int) =
            *((*p).verts[i as usize].modulate.as_ptr() as *const c_int);

        numv += 1;
        i += 1;
    }

    // generate fan indexes into the tess array
    i = 0;
    while i < (*p).numVerts - 2 {
        tess.indexes[tess.numIndexes as usize + 0] = tess.numVertexes;
        tess.indexes[tess.numIndexes as usize + 1] = tess.numVertexes + i + 1;
        tess.indexes[tess.numIndexes as usize + 2] = tess.numVertexes + i + 2;
        tess.numIndexes += 3;
        i += 1;
    }

    tess.numVertexes = numv;
}

#[inline]
unsafe fn ComputeFinalVertexColor(colors: *const byte) -> c_ulong {
    let mut k: c_int;
    let mut result: [byte; 4] = [0; 4];

    *(result.as_mut_ptr() as *mut c_int) = *(colors as *const c_int);
    if (*tess.shader).lightmapIndex[0] != LIGHTMAP_BY_VERTEX {
        return *(result.as_ptr() as *const c_ulong);
    }
    if (*r_fullbright).integer != 0 {
        result[0] = 255;
        result[1] = 255;
        result[2] = 255;
        return *(result.as_ptr() as *const c_ulong);
    }
    // an optimization could be added here to compute the style[0] (which is always the world normal light)
    let mut r: c_ulong = 0;
    let mut g: c_ulong = 0;
    let mut b: c_ulong = 0;
    let mut colors = colors;
    k = 0;
    while k < MAXLIGHTMAPS {
        if (*tess.shader).styles[k as usize] < LS_UNUSED {
            let mut styleColor: *mut byte =
                styleColors[(*tess.shader).styles[k as usize] as usize];

            r += (*colors as c_ulong) * (*styleColor as c_ulong);
            colors = colors.add(1);
            styleColor = styleColor.add(1);
            g += (*colors as c_ulong) * (*styleColor as c_ulong);
            colors = colors.add(1);
            styleColor = styleColor.add(1);
            b += (*colors as c_ulong) * (*styleColor as c_ulong);
            colors = colors.add(1);
            colors = colors.add(1);
        } else {
            break;
        }
        k += 1;
    }
    result[0] = Com_Clamp(0, 255, (r >> 8) as c_int) as byte;
    result[1] = Com_Clamp(0, 255, (g >> 8) as c_int) as byte;
    result[2] = Com_Clamp(0, 255, (b >> 8) as c_int) as byte;

    *(result.as_ptr() as *const c_ulong)
}

#[cfg(feature = "_XBOX")]
//16 bits in, 32 bits out
#[inline]
unsafe fn ComputeFinalVertexColor16(colors: *const byte) -> c_ulong {
    let mut k: c_int;
    let mut result: [byte; 4] = [0; 4];
    let mut color32: [byte; 4] = [0; 4];

    result[0] = (*colors.add(0)) & 0xF0;
    result[1] = (*colors.add(0)) << 4;
    result[2] = (*colors.add(1)) & 0xF0;
    result[3] = (*colors.add(1)) << 4;
    if (*tess.shader).lightmapIndex[0] != LIGHTMAP_BY_VERTEX || (*r_fullbright).integer != 0 {
        result[0] = 255;
        result[1] = 255;
        result[2] = 255;
        return *(result.as_ptr() as *const c_ulong);
    }
    // an optimization could be added here to compute the style[0] (which is always the world normal light)
    let mut r: c_ulong = 0;
    let mut g: c_ulong = 0;
    let mut b: c_ulong = 0;
    k = 0;
    while k < MAXLIGHTMAPS {
        if (*tess.shader).styles[k as usize] < LS_UNUSED {
            let mut styleColor: *mut byte =
                styleColors[(*tess.shader).styles[k as usize] as usize];

            color32[0] = (*colors.add(k as usize * 2)) & 0xF0;
            color32[1] = (*colors.add(k as usize * 2)) << 4;
            color32[2] = (*colors.add(k as usize * 2 + 1)) & 0xF0;

            r += (color32[0] as c_ulong) * (*styleColor as c_ulong);
            styleColor = styleColor.add(1);
            g += (color32[1] as c_ulong) * (*styleColor as c_ulong);
            styleColor = styleColor.add(1);
            b += (color32[2] as c_ulong) * (*styleColor as c_ulong);
        } else {
            break;
        }
        k += 1;
    }
    result[0] = Com_Clamp(0, 255, (r >> 8) as c_int) as byte;
    result[1] = Com_Clamp(0, 255, (g >> 8) as c_int) as byte;
    result[2] = Com_Clamp(0, 255, (b >> 8) as c_int) as byte;

    *(result.as_ptr() as *const c_ulong)
}


/*
=============
RB_SurfaceTriangles
=============
*/
pub unsafe fn RB_SurfaceTriangles(srf: *mut srfTriangles_t) {
    let mut i: c_int;
    let mut k: c_int;
    let mut dv: *mut drawVert_t;
    let mut xyz: *mut f32;
    let mut normal: *mut f32;
    let mut texCoords: *mut f32;
    #[cfg(feature = "_XBOX")]
    let mut tangent: *mut f32;
    let mut color: *mut byte;
    let dlightBits: c_int;

    dlightBits = (*srf).dlightBits;
    tess.dlightBits |= dlightBits;

    RB_CheckOverflow((*srf).numVerts, (*srf).numIndexes);

    i = 0;
    while i < (*srf).numIndexes {
        tess.indexes[(tess.numIndexes + i + 0) as usize] =
            tess.numVertexes + (*srf).indexes[(i + 0) as usize];
        tess.indexes[(tess.numIndexes + i + 1) as usize] =
            tess.numVertexes + (*srf).indexes[(i + 1) as usize];
        tess.indexes[(tess.numIndexes + i + 2) as usize] =
            tess.numVertexes + (*srf).indexes[(i + 2) as usize];
        i += 3;
    }
    tess.numIndexes += (*srf).numIndexes;

    dv = (*srf).verts;
    xyz = tess.xyz[tess.numVertexes as usize].as_mut_ptr();
    normal = tess.normal[tess.numVertexes as usize].as_mut_ptr();
    texCoords = tess.texCoords[tess.numVertexes as usize][0].as_mut_ptr();
    color = tess.vertexColors[tess.numVertexes as usize].as_mut_ptr();
    #[cfg(feature = "_XBOX")]
    {
        tangent = tess.tangent[tess.numVertexes as usize].as_mut_ptr();
    }

    i = 0;
    while i < (*srf).numVerts {
        #[cfg(feature = "_XBOX")]
        {
            *xyz.add(0) = (*dv).xyz[0];
            *xyz.add(1) = (*dv).xyz[1];
            *xyz.add(2) = (*dv).xyz[2];
            xyz = xyz.add(4);

            if (*tess.shader).needsNormal != 0 || tess.dlightBits != 0 {
                *normal.add(0) = (*dv).normal[0] as f32 / 32767.0_f32;
                *normal.add(1) = (*dv).normal[1] as f32 / 32767.0_f32;
                *normal.add(2) = (*dv).normal[2] as f32 / 32767.0_f32;
                normal = normal.add(4);
            }

            if (*tess.shader).needsTangent != 0 || tess.dlightBits != 0 {
                *tangent.add(0) = (*dv).tangent[0];
                *tangent.add(1) = (*dv).tangent[1];
                *tangent.add(2) = (*dv).tangent[2];
                tangent = tangent.add(4);

                tess.setTangents = true;
            }

            Q_CastShort2FloatScale(
                texCoords.add(0),
                &(*dv).dvst[0],
                1.0_f32 / DRAWVERT_ST_SCALE,
            );
            Q_CastShort2FloatScale(
                texCoords.add(1),
                &(*dv).dvst[1],
                1.0_f32 / DRAWVERT_ST_SCALE,
            );

            k = 0;
            while k < MAXLIGHTMAPS {
                if (*tess.shader).lightmapIndex[k as usize] >= 0 {
                    Q_CastShort2FloatScale(
                        texCoords.add(2 + (k as usize * 2) + 0),
                        &(*dv).dvlightmap[k as usize][0],
                        1.0_f32 / DRAWVERT_LIGHTMAP_SCALE,
                    );
                    Q_CastShort2FloatScale(
                        texCoords.add(2 + (k as usize * 2) + 1),
                        &(*dv).dvlightmap[k as usize][1],
                        1.0_f32 / DRAWVERT_LIGHTMAP_SCALE,
                    );
                } else {
                    // can't have an empty slot in the middle, so we are done
                    break;
                }
                k += 1;
            }
            texCoords = texCoords.add(NUM_TEX_COORDS as usize * 2);

            *(color as *mut c_uint) =
                ComputeFinalVertexColor16((*dv).dvcolor.as_ptr()) as c_uint;
            color = color.add(4);
        }
        #[cfg(not(feature = "_XBOX"))]
        {
            *xyz.add(0) = (*dv).xyz[0];
            *xyz.add(1) = (*dv).xyz[1];
            *xyz.add(2) = (*dv).xyz[2];
            xyz = xyz.add(4);

            *normal.add(0) = (*dv).normal[0];
            *normal.add(1) = (*dv).normal[1];
            *normal.add(2) = (*dv).normal[2];
            normal = normal.add(4);

            *texCoords.add(0) = (*dv).st[0];
            *texCoords.add(1) = (*dv).st[1];

            k = 0;
            while k < MAXLIGHTMAPS {
                if (*tess.shader).lightmapIndex[k as usize] >= 0 {
                    *texCoords.add(2 + (k as usize * 2)) = (*dv).lightmap[k as usize][0];
                    *texCoords.add(2 + (k as usize * 2) + 1) = (*dv).lightmap[k as usize][1];
                } else {
                    // can't have an empty slot in the middle, so we are done
                    break;
                }
                k += 1;
            }
            texCoords = texCoords.add(NUM_TEX_COORDS as usize * 2);

            *(color as *mut c_uint) =
                ComputeFinalVertexColor((*dv).color.as_ptr()) as c_uint;
            color = color.add(4);
        }

        dv = dv.add(1);
        i += 1;
    }

    i = 0;
    while i < (*srf).numVerts {
        tess.vertexDlightBits[(tess.numVertexes + i) as usize] = dlightBits;
        i += 1;
    }

    tess.numVertexes += (*srf).numVerts;
}



/*
==============
RB_SurfaceBeam
==============
*/
unsafe fn RB_SurfaceBeam() {
    // #define NUM_BEAM_SEGS 6
    const NUM_BEAM_SEGS: usize = 6;
    let e: *mut refEntity_t;
    let mut i: c_int;
    let mut perpvec: vec3_t = [0.0; 3];
    let mut direction: vec3_t = [0.0; 3];
    let mut normalized_direction: vec3_t = [0.0; 3];
    let mut start_points: [vec3_t; NUM_BEAM_SEGS] = [[0.0; 3]; NUM_BEAM_SEGS];
    let mut end_points: [vec3_t; NUM_BEAM_SEGS] = [[0.0; 3]; NUM_BEAM_SEGS];
    let mut oldorigin: vec3_t = [0.0; 3];
    let mut origin: vec3_t = [0.0; 3];

    e = &mut (*backEnd.currentEntity).e as *mut refEntity_t;

    oldorigin[0] = (*e).oldorigin[0];
    oldorigin[1] = (*e).oldorigin[1];
    oldorigin[2] = (*e).oldorigin[2];

    origin[0] = (*e).origin[0];
    origin[1] = (*e).origin[1];
    origin[2] = (*e).origin[2];

    normalized_direction[0] = oldorigin[0] - origin[0];
    direction[0] = normalized_direction[0];
    normalized_direction[1] = oldorigin[1] - origin[1];
    direction[1] = normalized_direction[1];
    normalized_direction[2] = oldorigin[2] - origin[2];
    direction[2] = normalized_direction[2];

    if VectorNormalize(&mut normalized_direction) == 0.0 {
        return;
    }

    PerpendicularVector(&mut perpvec, &normalized_direction);

    VectorScale(&perpvec.clone(), 4.0, &mut perpvec);

    i = 0;
    while i < NUM_BEAM_SEGS as c_int {
        RotatePointAroundVector(
            &mut start_points[i as usize],
            &normalized_direction,
            &perpvec,
            (360.0 / NUM_BEAM_SEGS as f32) * i as f32,
        );
        //		VectorAdd( start_points[i], origin, start_points[i] );
        VectorAdd(
            &start_points[i as usize].clone(),
            &direction,
            &mut end_points[i as usize],
        );
        i += 1;
    }

    GL_Bind(tr.whiteImage);

    GL_State(GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE);

    qglColor3f(1.0, 0.0, 0.0);

    qglBegin(GL_TRIANGLE_STRIP);
    i = 0;
    while i <= NUM_BEAM_SEGS as c_int {
        qglVertex3fv(start_points[(i as usize) % NUM_BEAM_SEGS].as_ptr());
        qglVertex3fv(end_points[(i as usize) % NUM_BEAM_SEGS].as_ptr());
        i += 1;
    }
    qglEnd();
}

//------------------
// DoSprite
//------------------
unsafe fn DoSprite(origin: vec3_t, radius: f32, rotation: f32) {
    let s: f32;
    let c: f32;
    let ang: f32;
    let mut left: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    ang = M_PI * rotation / 180.0_f32;
    s = sin(ang as c_double) as f32;
    c = cos(ang as c_double) as f32;

    VectorScale(&backEnd.viewParms.ori.axis[1], c * radius, &mut left);
    VectorMA(&left.clone(), -s * radius, &backEnd.viewParms.ori.axis[2], &mut left);

    VectorScale(&backEnd.viewParms.ori.axis[2], c * radius, &mut up);
    VectorMA(&up.clone(), s * radius, &backEnd.viewParms.ori.axis[1], &mut up);

    if backEnd.viewParms.isMirror != 0 {
        VectorSubtract(&vec3_origin, &left.clone(), &mut left);
    }

    RB_AddQuadStamp(
        origin,
        left,
        up,
        (*backEnd.currentEntity).e.shaderRGBA.as_mut_ptr(),
    );
}

//------------------
// RB_SurfaceSaber
//------------------
unsafe fn RB_SurfaceSaberGlow() {
    let mut end: vec3_t = [0.0; 3];
    let e: *mut refEntity_t;

    e = &mut (*backEnd.currentEntity).e as *mut refEntity_t;

    // Render the glow part of the blade
    let mut i: f32 = (*e).saberLength;
    while i > 0.0 {
        VectorMA(&(*e).origin, i, &(*e).axis[0], &mut end);

        DoSprite(end, (*e).radius, 0.0_f32); //random() * 360.0f );
        (*e).radius += 0.017_f32;

        i -= (*e).radius * 0.65_f32;
    }

    // Big hilt sprite
    // Please don't kill me Pat...I liked the hilt glow blob, but wanted a subtle pulse.:)  Feel free to ditch it if you don't like it.  --Jeff
    // Please don't kill me Jeff...  The pulse is good, but now I want the halo bigger if the saber is shorter...  --Pat
    DoSprite((*e).origin, 5.5_f32 + random() * 0.25_f32, 0.0_f32); //random() * 360.0f );
}

/*
==============
RB_SurfaceLine
==============
*/
//
//	Values for a proper line render primitive...
//		Width
//		STScale (how many times to loop a texture)
//		alpha
//		RGB
//
//  Values for proper line object...
//		lifetime
//		dscale
//		startalpha, endalpha
//		startRGB, endRGB
//

unsafe fn DoLine(start: &vec3_t, end: &vec3_t, up: &vec3_t, spanWidth: f32) {
    let spanWidth2: f32;
    let vbase: c_int;

    RB_CheckOverflow(4, 6);

    vbase = tess.numVertexes;

    spanWidth2 = -spanWidth;

    VectorMA(start, spanWidth, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 0.0;
    tess.texCoords[tess.numVertexes as usize][0][1] = 0.0;
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0]; // * 0.25;//wtf??not sure why the code would be doing this
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1]; // * 0.25;
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2]; // * 0.25;
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3]; // * 0.25;
    tess.numVertexes += 1;

    VectorMA(start, spanWidth2, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 1.0; //backEnd.currentEntity->e.shaderTexCoord[0];
    tess.texCoords[tess.numVertexes as usize][0][1] = 0.0;
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3];
    tess.numVertexes += 1;

    VectorMA(end, spanWidth, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));

    tess.texCoords[tess.numVertexes as usize][0][0] = 0.0;
    tess.texCoords[tess.numVertexes as usize][0][1] = 1.0; //backEnd.currentEntity->e.shaderTexCoord[1];
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3];
    tess.numVertexes += 1;

    VectorMA(end, spanWidth2, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 1.0; //backEnd.currentEntity->e.shaderTexCoord[0];
    tess.texCoords[tess.numVertexes as usize][0][1] = 1.0; //backEnd.currentEntity->e.shaderTexCoord[1];
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3];
    tess.numVertexes += 1;

    tess.indexes[tess.numIndexes as usize] = vbase;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 1;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 2;
    tess.numIndexes += 1;

    tess.indexes[tess.numIndexes as usize] = vbase + 2;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 1;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 3;
    tess.numIndexes += 1;
}

unsafe fn DoLine2(
    start: &vec3_t,
    end: &vec3_t,
    up: &vec3_t,
    spanWidth: f32,
    spanWidth2: f32,
) {
    let vbase: c_int;

    RB_CheckOverflow(4, 6);

    vbase = tess.numVertexes;

    VectorMA(start, spanWidth, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 0.0;
    tess.texCoords[tess.numVertexes as usize][0][1] = 0.0;
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0]; // * 0.25;//wtf??not sure why the code would be doing this
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1]; // * 0.25;
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2]; // * 0.25;
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3]; // * 0.25;
    tess.numVertexes += 1;

    VectorMA(start, -spanWidth, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 1.0; //backEnd.currentEntity->e.shaderTexCoord[0];
    tess.texCoords[tess.numVertexes as usize][0][1] = 0.0;
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3];
    tess.numVertexes += 1;

    VectorMA(end, spanWidth2, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));

    tess.texCoords[tess.numVertexes as usize][0][0] = 0.0;
    tess.texCoords[tess.numVertexes as usize][0][1] = 1.0; //backEnd.currentEntity->e.shaderTexCoord[1];
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3];
    tess.numVertexes += 1;

    VectorMA(end, -spanWidth2, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 1.0; //backEnd.currentEntity->e.shaderTexCoord[0];
    tess.texCoords[tess.numVertexes as usize][0][1] = 1.0; //backEnd.currentEntity->e.shaderTexCoord[1];
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3];
    tess.numVertexes += 1;

    tess.indexes[tess.numIndexes as usize] = vbase;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 1;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 2;
    tess.numIndexes += 1;

    tess.indexes[tess.numIndexes as usize] = vbase + 2;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 1;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 3;
    tess.numIndexes += 1;
}

unsafe fn DoLine_Oriented(start: &vec3_t, end: &vec3_t, up: &vec3_t, spanWidth: f32) {
    let spanWidth2: f32;
    let vbase: c_int;

    vbase = tess.numVertexes;

    spanWidth2 = -spanWidth;

    // FIXME: use quad stamp?
    VectorMA(start, spanWidth, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 0.0;
    tess.texCoords[tess.numVertexes as usize][0][1] = 0.0;
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0]; // * 0.25;
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1]; // * 0.25;
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2]; // * 0.25;
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3]; // * 0.25;
    tess.numVertexes += 1;

    VectorMA(start, spanWidth2, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 1.0;
    tess.texCoords[tess.numVertexes as usize][0][1] = 0.0;
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3]; // * 0.25;
    tess.numVertexes += 1;

    VectorMA(end, spanWidth, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));

    tess.texCoords[tess.numVertexes as usize][0][0] = 0.0;
    tess.texCoords[tess.numVertexes as usize][0][1] = (*backEnd.currentEntity).e.data.line.stscale;
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3]; // * 0.25;
    tess.numVertexes += 1;

    VectorMA(end, spanWidth2, up, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
    tess.texCoords[tess.numVertexes as usize][0][0] = 1.0;
    tess.texCoords[tess.numVertexes as usize][0][1] = (*backEnd.currentEntity).e.data.line.stscale;
    tess.vertexColors[tess.numVertexes as usize][0] = (*backEnd.currentEntity).e.shaderRGBA[0];
    tess.vertexColors[tess.numVertexes as usize][1] = (*backEnd.currentEntity).e.shaderRGBA[1];
    tess.vertexColors[tess.numVertexes as usize][2] = (*backEnd.currentEntity).e.shaderRGBA[2];
    tess.vertexColors[tess.numVertexes as usize][3] = (*backEnd.currentEntity).e.shaderRGBA[3]; // * 0.25;
    tess.numVertexes += 1;

    tess.indexes[tess.numIndexes as usize] = vbase;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 1;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 2;
    tess.numIndexes += 1;

    tess.indexes[tess.numIndexes as usize] = vbase + 2;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 1;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 3;
    tess.numIndexes += 1;
}

//-----------------
// RB_SurfaceLine
//-----------------
unsafe fn RB_SurfaceLine() {
    let e: *mut refEntity_t;
    let mut right: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];

    e = &mut (*backEnd.currentEntity).e as *mut refEntity_t;

    VectorCopy(&(*e).oldorigin, &mut end);
    VectorCopy(&(*e).origin, &mut start);

    // compute side vector
    VectorSubtract(&start, &backEnd.viewParms.ori.origin, &mut v1);
    VectorSubtract(&end, &backEnd.viewParms.ori.origin, &mut v2);
    CrossProduct(&v1, &v2, &mut right);
    VectorNormalize(&mut right);

    DoLine(&start, &end, &right, (*e).radius);
}

unsafe fn RB_SurfaceOrientedLine() {
    let e: *mut refEntity_t;
    let mut right: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    e = &mut (*backEnd.currentEntity).e as *mut refEntity_t;

    VectorCopy(&(*e).oldorigin, &mut end);
    VectorCopy(&(*e).origin, &mut start);

    // compute side vector
    VectorNormalize(&mut (*e).axis[1]);
    VectorCopy(&(*e).axis[1], &mut right);
    DoLine_Oriented(&start, &end, &right, (*e).data.line.width * 0.5);
}

/*
==============
RB_SurfaceCylinder
==============
*/

// FIXME: use quad stamp?
unsafe fn DoCylinderPart(verts: *mut polyVert_t) {
    let vbase: c_int;
    let mut i: c_int;
    let mut verts = verts;

    RB_CheckOverflow(4, 6);

    vbase = tess.numVertexes;

    i = 0;
    while i < 4 {
        VectorCopy(&(*verts).xyz, &mut *(tess.xyz[tess.numVertexes as usize].as_mut_ptr() as *mut vec3_t));
        tess.texCoords[tess.numVertexes as usize][0][0] = (*verts).st[0];
        tess.texCoords[tess.numVertexes as usize][0][1] = (*verts).st[1];
        tess.vertexColors[tess.numVertexes as usize][0] = (*verts).modulate[0];
        tess.vertexColors[tess.numVertexes as usize][1] = (*verts).modulate[1];
        tess.vertexColors[tess.numVertexes as usize][2] = (*verts).modulate[2];
        tess.vertexColors[tess.numVertexes as usize][3] = (*verts).modulate[3];
        tess.numVertexes += 1;
        verts = verts.add(1);
        i += 1;
    }

    tess.indexes[tess.numIndexes as usize] = vbase;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 1;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 2;
    tess.numIndexes += 1;

    tess.indexes[tess.numIndexes as usize] = vbase + 2;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase + 3;
    tess.numIndexes += 1;
    tess.indexes[tess.numIndexes as usize] = vbase;
    tess.numIndexes += 1;
}

// e->origin holds the bottom point
// e->oldorigin holds the top point
// e->radius holds the radius

unsafe fn RB_SurfaceCylinder() {
    static mut lower_points: [polyVert_t; NUM_CYLINDER_SEGMENTS] =
        unsafe { core::mem::zeroed() };
    static mut upper_points: [polyVert_t; NUM_CYLINDER_SEGMENTS] =
        unsafe { core::mem::zeroed() };
    static mut verts: [polyVert_t; 4] = unsafe { core::mem::zeroed() };

    let mut vr: vec3_t = [0.0; 3];
    let mut vu: vec3_t = [0.0; 3];
    let mut midpoint: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut detail: f32;
    let mut length: f32;
    let mut i: c_int;
    let mut segments: c_int;
    let e: *mut refEntity_t;
    let mut nextSegment: c_int;

    e = &mut (*backEnd.currentEntity).e as *mut refEntity_t;

    //Work out the detail level of this cylinder
    VectorAdd(&(*e).origin, &(*e).oldorigin, &mut midpoint);
    VectorScale(&midpoint.clone(), 0.5_f32, &mut midpoint); // Average start and end

    VectorSubtract(&midpoint.clone(), &backEnd.viewParms.ori.origin, &mut midpoint);
    length = VectorNormalize(&mut midpoint);

    // this doesn't need to be perfect....just a rough compensation for zoom level is enough
    length *= backEnd.viewParms.fovX / 90.0_f32;

    detail = 1.0 - length / 1024.0;
    segments = (NUM_CYLINDER_SEGMENTS as f32 * detail) as c_int;

    // 3 is the absolute minimum, but the pop between 3-8 is too noticeable
    if segments < 8 {
        segments = 8;
    }

    if segments > NUM_CYLINDER_SEGMENTS as c_int {
        segments = NUM_CYLINDER_SEGMENTS as c_int;
    }

    //Get the direction vector
    MakeNormalVectors(&(*e).axis[0], &mut vr, &mut vu);

    VectorScale(&vu.clone(), (*e).radius, &mut v1); // size1
    VectorScale(&vu.clone(), (*e).rotation, &mut vu); // size2

    // Calculate the step around the cylinder
    detail = 360.0_f32 / segments as f32;

    i = 0;
    while i < segments {
        //Upper ring
        RotatePointAroundVector(
            &mut upper_points[i as usize].xyz,
            &(*e).axis[0],
            &vu,
            detail * i as f32,
        );
        VectorAdd(
            &upper_points[i as usize].xyz.clone(),
            &(*e).origin,
            &mut upper_points[i as usize].xyz,
        );

        //Lower ring
        RotatePointAroundVector(
            &mut lower_points[i as usize].xyz,
            &(*e).axis[0],
            &v1,
            detail * i as f32,
        );
        VectorAdd(
            &lower_points[i as usize].xyz.clone(),
            &(*e).oldorigin,
            &mut lower_points[i as usize].xyz,
        );

        i += 1;
    }

    // Calculate the texture coords so the texture can wrap around the whole cylinder
    detail = 1.0_f32 / segments as f32;

    i = 0;
    while i < segments {
        if i + 1 < segments {
            nextSegment = i + 1;
        } else {
            nextSegment = 0;
        }

        VectorCopy(&upper_points[i as usize].xyz, &mut verts[0].xyz);
        verts[0].st[1] = 1.0_f32;
        verts[0].st[0] = detail * i as f32;
        verts[0].modulate[0] = (*e).shaderRGBA[0] as byte;
        verts[0].modulate[1] = (*e).shaderRGBA[1] as byte;
        verts[0].modulate[2] = (*e).shaderRGBA[2] as byte;
        verts[0].modulate[3] = (*e).shaderRGBA[3] as byte;

        VectorCopy(&lower_points[i as usize].xyz, &mut verts[1].xyz);
        verts[1].st[1] = 0.0_f32;
        verts[1].st[0] = detail * i as f32;
        verts[1].modulate[0] = (*e).shaderRGBA[0] as byte;
        verts[1].modulate[1] = (*e).shaderRGBA[1] as byte;
        verts[1].modulate[2] = (*e).shaderRGBA[2] as byte;
        verts[1].modulate[3] = (*e).shaderRGBA[3] as byte;

        VectorCopy(&lower_points[nextSegment as usize].xyz, &mut verts[2].xyz);
        verts[2].st[1] = 0.0_f32;
        verts[2].st[0] = detail * (i + 1) as f32;
        verts[2].modulate[0] = (*e).shaderRGBA[0] as byte;
        verts[2].modulate[1] = (*e).shaderRGBA[1] as byte;
        verts[2].modulate[2] = (*e).shaderRGBA[2] as byte;
        verts[2].modulate[3] = (*e).shaderRGBA[3] as byte;

        VectorCopy(&upper_points[nextSegment as usize].xyz, &mut verts[3].xyz);
        verts[3].st[1] = 1.0_f32;
        verts[3].st[0] = detail * (i + 1) as f32;
        verts[3].modulate[0] = (*e).shaderRGBA[0] as byte;
        verts[3].modulate[1] = (*e).shaderRGBA[1] as byte;
        verts[3].modulate[2] = (*e).shaderRGBA[2] as byte;
        verts[3].modulate[3] = (*e).shaderRGBA[3] as byte;

        DoCylinderPart(verts.as_mut_ptr());

        i += 1;
    }
}

// these functions are pretty crappy in terms of returning a nice range of rnd numbers, but it's probably good enough?
/*static int Q_rand( int *seed ) {
	*seed = (69069 * *seed + 1);
	return *seed;
}

static float Q_random( int *seed ) {
	return ( Q_rand( seed ) & 0xffff ) / (float)0x10000;
}

static float Q_crandom( int *seed ) {
	return 2.0F * ( Q_random( seed ) - 0.5f );
}
*/
// Up front, we create a random "shape", then apply that to each line segment...and then again to each of those segments...kind of like a fractal
//----------------------------------------------------------------------------
unsafe fn CreateShape()
//----------------------------------------------------------------------------
{
    VectorSet(
        &mut sh1,
        0.66_f32 + crandom() * 0.1_f32, // fwd
        0.07_f32 + crandom() * 0.025_f32,
        0.07_f32 + crandom() * 0.025_f32,
    );

    // it seems to look best to have a point on one side of the ideal line, then the other point on the other side.
    VectorSet(
        &mut sh2,
        0.33_f32 + crandom() * 0.1_f32, // fwd
        -sh1[1] + crandom() * 0.02_f32,  // forcing point to be on the opposite side of the line -- right
        -sh1[2] + crandom() * 0.02_f32,  // up
    );
}

//----------------------------------------------------------------------------
unsafe fn ApplyShape(
    start: vec3_t,
    end: vec3_t,
    right: &vec3_t,
    sradius: f32,
    eradius: f32,
    count: c_int,
)
//----------------------------------------------------------------------------
{
    let mut point1: vec3_t = [0.0; 3];
    let mut point2: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut rt: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut perc: f32;
    let dis: f32;

    if count < 1 {
        // done recursing
        DoLine2(&start, &end, right, sradius, eradius);
        return;
    }

    CreateShape();

    VectorSubtract(&end, &start, &mut fwd);
    dis = VectorNormalize(&mut fwd) * 0.7_f32;
    MakeNormalVectors(&fwd, &mut rt, &mut up);

    perc = sh1[0];

    VectorScale(&start, perc, &mut point1);
    VectorMA(&point1.clone(), 1.0_f32 - perc, &end, &mut point1);
    VectorMA(&point1.clone(), dis * sh1[1], &rt, &mut point1);
    VectorMA(&point1.clone(), dis * sh1[2], &up, &mut point1);

    // do a quick and dirty interpolation of the radius at that point
    let rads1: f32;
    let rads2: f32;

    rads1 = sradius * 0.666_f32 + eradius * 0.333_f32;
    rads2 = sradius * 0.333_f32 + eradius * 0.666_f32;

    // recursion
    ApplyShape(start, point1, right, sradius, rads1, count - 1);

    perc = sh2[0];

    VectorScale(&start, perc, &mut point2);
    VectorMA(&point2.clone(), 1.0_f32 - perc, &end, &mut point2);
    VectorMA(&point2.clone(), dis * sh2[1], &rt, &mut point2);
    VectorMA(&point2.clone(), dis * sh2[2], &up, &mut point2);

    // recursion
    ApplyShape(point2, point1, right, rads1, rads2, count - 1);
    ApplyShape(point2, end, right, rads2, eradius, count - 1);
}

//----------------------------------------------------------------------------
unsafe fn DoBoltSeg(start: vec3_t, end: vec3_t, right: &vec3_t, radius: f32)
//----------------------------------------------------------------------------
{
    let e: *mut refEntity_t;
    let mut fwd: vec3_t = [0.0; 3];
    let mut old: vec3_t = [0.0; 3];
    let mut cur: vec3_t = [0.0; 3];
    let mut off: vec3_t = [10.0, 10.0, 10.0];
    let mut rt: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut temp: vec3_t = [0.0; 3];
    let mut i: c_int;
    let dis: f32;
    let mut oldPerc: f32 = 0.0_f32;
    let mut perc: f32;
    let mut oldRadius: f32;
    let mut newRadius: f32;

    e = &mut (*backEnd.currentEntity).e as *mut refEntity_t;

    VectorSubtract(&end, &start, &mut fwd);
    dis = VectorNormalize(&mut fwd);

    MakeNormalVectors(&fwd, &mut rt, &mut up);

    VectorCopy(&start, &mut old);

    oldRadius = radius;
    newRadius = radius;

    i = 20;
    while i <= dis as c_int {
        // because of our large step size, we may not actually draw to the end.  In this case, fudge our percent so that we are basically complete
        if i + 20 > dis as c_int {
            perc = 1.0_f32;
        } else {
            // percentage of the amount of line completed
            perc = i as f32 / dis;
        }

        // create our level of deviation for this point
        VectorScale(&fwd, Q_crandom(&mut (*e).frame) * 3.0_f32, &mut temp); // move less in fwd direction, chaos also does not affect this
        VectorMA(&temp.clone(), Q_crandom(&mut (*e).frame) * 7.0_f32 * (*e).axis[0][0], &rt, &mut temp); // move more in direction perpendicular to line, angles is really the chaos
        VectorMA(&temp.clone(), Q_crandom(&mut (*e).frame) * 7.0_f32 * (*e).axis[0][0], &up, &mut temp); // move more in direction perpendicular to line

        // track our total level of offset from the ideal line
        VectorAdd(&off.clone(), &temp, &mut off);

        // Move from start to end, always adding our current level of offset from the ideal line
        //	Even though we are adding a random offset.....by nature, we always move from exactly start....to end
        VectorAdd(&start, &off, &mut cur);
        VectorScale(&cur.clone(), 1.0_f32 - perc, &mut cur);
        VectorMA(&cur.clone(), perc, &end, &mut cur);

        if (*e).renderfx & RF_TAPERED != 0 {
            // This does pretty close to perfect tapering since apply shape interpolates the old and new as it goes along.
            //	by using one minus the square, the radius stays fairly constant, then drops off quickly at the very point of the bolt
            oldRadius = radius * (1.0_f32 - oldPerc * oldPerc);
            newRadius = radius * (1.0_f32 - perc * perc);
        }

        // Apply the random shape to our line seg to give it some micro-detail-jaggy-coolness.
        ApplyShape(cur, old, right, newRadius, oldRadius, LIGHTNING_RECURSION_LEVEL);

        // randomly split off to create little tendrils, but don't do it too close to the end and especially if we are not even of the forked variety
        if ((*e).renderfx & RF_FORKED) != 0
            && f_count > 0.0
            && Q_random(&mut (*e).frame) > 0.94_f32
            && radius * (1.0_f32 - perc) > 0.2_f32
        {
            let mut newDest: vec3_t = [0.0; 3];

            f_count -= 1.0;

            // Pick a point somewhere between the current point and the final endpoint
            VectorAdd(&cur, &(*e).oldorigin, &mut newDest);
            VectorScale(&newDest.clone(), 0.5_f32, &mut newDest);

            // And then add some crazy offset
            for t in 0..3_usize {
                newDest[t] += Q_crandom(&mut (*e).frame) * 80.0;
            }

            // we could branch off using OLD and NEWDEST, but that would allow multiple forks...whereas, we just want simpler brancing
            DoBoltSeg(cur, newDest, right, newRadius);
        }

        // Current point along the line becomes our new old attach point
        VectorCopy(&cur, &mut old);
        oldPerc = perc;

        i += 20;
    }
}

//------------------------------------------
unsafe fn RB_SurfaceElectricity()
//------------------------------------------
{
    let e: *mut refEntity_t;
    let mut right: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];
    let radius: f32;
    let mut perc: f32 = 1.0_f32;
    let dis: f32;

    e = &mut (*backEnd.currentEntity).e as *mut refEntity_t;
    radius = (*e).radius;

    VectorCopy(&(*e).origin, &mut start);

    VectorSubtract(&(*e).oldorigin, &start, &mut fwd);
    dis = VectorNormalize(&mut fwd);

    // see if we should grow from start to end
    if (*e).renderfx & RF_GROW != 0 {
        perc = 1.0_f32
            - ((*e).axis[0][2] /*endTime*/ - tr.refdef.time as f32)
                / (*e).axis[0][1]; /*duration*/

        if perc > 1.0_f32 {
            perc = 1.0_f32;
        } else if perc < 0.0_f32 {
            perc = 0.0_f32;
        }
    }

    VectorMA(&start, perc * dis, &fwd, &mut (*e).oldorigin);
    VectorCopy(&(*e).oldorigin, &mut end);

    // compute side vector
    VectorSubtract(&start, &backEnd.viewParms.ori.origin, &mut v1);
    VectorSubtract(&end, &backEnd.viewParms.ori.origin, &mut v2);
    CrossProduct(&v1, &v2, &mut right);
    VectorNormalize(&mut right);

    DoBoltSeg(start, end, &right, radius);
}

//================================================================================


/*
** VectorArrayNormalize
*
* The inputs to this routing seem to always be close to length = 1.0 (about 0.6 to 2.0)
* This means that we don't have to worry about zero length or enormously long vectors.
*/
unsafe fn VectorArrayNormalize(normals: *mut vec4_t, mut count: c_uint) {
    //    assert(count);

    #[cfg(feature = "idppc")]
    {
        // Vanilla PPC code, but since PPC has a reciprocal square root estimate instruction,
        // runs *much* faster than calling sqrt().  We'll use a single Newton-Raphson
        // refinement step to get a little more precision.  This seems to yeild results
        // that are correct to 3 decimal places and usually correct to at least 4 (sometimes 5).
        // (That is, for the given input range of about 0.6 to 2.0).
        let half: f32 = 0.5;
        let one: f32 = 1.0;
        let mut components: *mut f32 = normals as *mut f32;
        // C: do { ... } while(count--);
        loop {
            let x: f32;
            let y: f32;
            let z: f32;
            let b_val: f32;
            let y1: f32;

            x = *components.add(0);
            y = *components.add(1);
            z = *components.add(2);
            components = components.add(4);
            b_val = x * x + y * y + z * z;

            // #ifdef __GNUC__
            //     asm("frsqrte %0,%1" : "=f" (y0) : "f" (B));
            // #else
            //     y0 = __frsqrte(B);
            // #endif
            // -- PPC frsqrte instruction; best-effort inline asm translation below
            #[cfg(all(feature = "idppc", target_arch = "powerpc"))]
            let y0: f32 = {
                let out: f32;
                core::arch::asm!("frsqrte {0},{1}", out(freg) out, in(freg) b_val);
                out
            };
            #[cfg(not(all(feature = "idppc", target_arch = "powerpc")))]
            let y0: f32 = 1.0_f32 / b_val.sqrt(); // fallback for non-PPC idppc builds

            y1 = y0 + half * y0 * (one - b_val * y0 * y0);

            let xn = x * y1;
            let yn = y * y1;
            *components.sub(4) = xn;
            let zn = z * y1;
            *components.sub(3) = yn;
            *components.sub(2) = zn;

            // do { ... } while(count--): post-decrement check
            if count == 0 {
                break;
            }
            count -= 1;
        }
    }
    #[cfg(not(feature = "idppc"))]
    {
        // given the input, it's safe to call VectorNormalizeFast
        let mut normals = normals;
        // C: while (count--) { VectorNormalizeFast(normals[0]); normals++; }
        while count > 0 {
            VectorNormalizeFast(&mut *(normals as *mut vec3_t));
            normals = normals.add(1);
            count -= 1;
        }
    }
}



/*
** LerpMeshVertexes
*/
unsafe fn LerpMeshVertexes(surf: *mut md3Surface_t, backlerp: f32) {
    let mut oldXyz: *mut i16;
    let mut newXyz: *mut i16;
    let mut oldNormals: *mut i16;
    let mut newNormals: *mut i16;
    let mut outXyz: *mut f32;
    let mut outNormal: *mut f32;
    let oldXyzScale: f32;
    let newXyzScale: f32;
    let oldNormalScale: f32;
    let newNormalScale: f32;
    let mut vertNum: c_int;
    let mut lat: c_uint;
    let mut lng: c_uint;
    let numVerts: c_int;

    outXyz = tess.xyz[tess.numVertexes as usize].as_mut_ptr();
    outNormal = tess.normal[tess.numVertexes as usize].as_mut_ptr();

    newXyz = (surf as *mut byte).add((*surf).ofsXyzNormals as usize) as *mut i16;
    newXyz = newXyz.add(
        ((*backEnd.currentEntity).e.frame * (*surf).numVerts * 4) as usize,
    );
    newNormals = newXyz.add(3);

    newXyzScale = MD3_XYZ_SCALE * (1.0 - backlerp);
    newNormalScale = 1.0 - backlerp;

    numVerts = (*surf).numVerts;

    if backlerp == 0.0 {
        //
        // just copy the vertexes
        //
        vertNum = 0;
        while vertNum < numVerts {
            *outXyz.add(0) = *newXyz.add(0) as f32 * newXyzScale;
            *outXyz.add(1) = *newXyz.add(1) as f32 * newXyzScale;
            *outXyz.add(2) = *newXyz.add(2) as f32 * newXyzScale;

            lat = ((*newNormals.add(0) as c_uint >> 8) & 0xff) as c_uint;
            lng = (*newNormals.add(0) as c_uint & 0xff) as c_uint;
            lat *= FUNCTABLE_SIZE / 256;
            lng *= FUNCTABLE_SIZE / 256;

            // decode X as cos( lat ) * sin( long )
            // decode Y as sin( lat ) * sin( long )
            // decode Z as cos( long )
            #[cfg(feature = "_XBOX")]
            {
                if (*tess.shader).needsNormal != 0 || tess.dlightBits != 0 {
                    *outNormal.add(0) = tr.sinTable[((lat + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize]
                        * tr.sinTable[lng as usize];
                    *outNormal.add(1) = tr.sinTable[lat as usize] * tr.sinTable[lng as usize];
                    *outNormal.add(2) =
                        tr.sinTable[((lng + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize];
                }
            }
            #[cfg(not(feature = "_XBOX"))]
            {
                *outNormal.add(0) = tr.sinTable[((lat + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize]
                    * tr.sinTable[lng as usize];
                *outNormal.add(1) = tr.sinTable[lat as usize] * tr.sinTable[lng as usize];
                *outNormal.add(2) =
                    tr.sinTable[((lng + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize];
            }

            newXyz = newXyz.add(4);
            newNormals = newNormals.add(4);
            outXyz = outXyz.add(4);
            outNormal = outNormal.add(4);
            vertNum += 1;
        }
    } else {
        //
        // interpolate and copy the vertex and normal
        //
        oldXyz = (surf as *mut byte).add((*surf).ofsXyzNormals as usize) as *mut i16;
        oldXyz = oldXyz.add(
            ((*backEnd.currentEntity).e.oldframe * (*surf).numVerts * 4) as usize,
        );
        oldNormals = oldXyz.add(3);

        oldXyzScale = MD3_XYZ_SCALE * backlerp;
        oldNormalScale = backlerp;

        vertNum = 0;
        while vertNum < numVerts {
            let mut uncompressedOldNormal: vec3_t = [0.0; 3];
            let mut uncompressedNewNormal: vec3_t = [0.0; 3];

            // interpolate the xyz
            *outXyz.add(0) = *oldXyz.add(0) as f32 * oldXyzScale + *newXyz.add(0) as f32 * newXyzScale;
            *outXyz.add(1) = *oldXyz.add(1) as f32 * oldXyzScale + *newXyz.add(1) as f32 * newXyzScale;
            *outXyz.add(2) = *oldXyz.add(2) as f32 * oldXyzScale + *newXyz.add(2) as f32 * newXyzScale;

            #[cfg(feature = "_XBOX")]
            let _xbox_cond = (*tess.shader).needsNormal != 0 || tess.dlightBits != 0;
            #[cfg(feature = "_XBOX")]
            if _xbox_cond {
            // FIXME: interpolate lat/long instead?
            lat = ((*newNormals.add(0) as c_uint >> 8) & 0xff) as c_uint;
            lng = (*newNormals.add(0) as c_uint & 0xff) as c_uint;
            lat *= 4;
            lng *= 4;
            uncompressedNewNormal[0] = tr.sinTable[((lat + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize]
                * tr.sinTable[lng as usize];
            uncompressedNewNormal[1] = tr.sinTable[lat as usize] * tr.sinTable[lng as usize];
            uncompressedNewNormal[2] =
                tr.sinTable[((lng + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize];

            lat = ((*oldNormals.add(0) as c_uint >> 8) & 0xff) as c_uint;
            lng = (*oldNormals.add(0) as c_uint & 0xff) as c_uint;
            lat *= 4;
            lng *= 4;

            uncompressedOldNormal[0] = tr.sinTable[((lat + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize]
                * tr.sinTable[lng as usize];
            uncompressedOldNormal[1] = tr.sinTable[lat as usize] * tr.sinTable[lng as usize];
            uncompressedOldNormal[2] =
                tr.sinTable[((lng + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize];

            *outNormal.add(0) = uncompressedOldNormal[0] * oldNormalScale
                + uncompressedNewNormal[0] * newNormalScale;
            *outNormal.add(1) = uncompressedOldNormal[1] * oldNormalScale
                + uncompressedNewNormal[1] * newNormalScale;
            *outNormal.add(2) = uncompressedOldNormal[2] * oldNormalScale
                + uncompressedNewNormal[2] * newNormalScale;
            //			VectorNormalize (outNormal);
            }
            #[cfg(not(feature = "_XBOX"))]
            {
            // FIXME: interpolate lat/long instead?
            lat = ((*newNormals.add(0) as c_uint >> 8) & 0xff) as c_uint;
            lng = (*newNormals.add(0) as c_uint & 0xff) as c_uint;
            lat *= 4;
            lng *= 4;
            uncompressedNewNormal[0] = tr.sinTable[((lat + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize]
                * tr.sinTable[lng as usize];
            uncompressedNewNormal[1] = tr.sinTable[lat as usize] * tr.sinTable[lng as usize];
            uncompressedNewNormal[2] =
                tr.sinTable[((lng + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize];

            lat = ((*oldNormals.add(0) as c_uint >> 8) & 0xff) as c_uint;
            lng = (*oldNormals.add(0) as c_uint & 0xff) as c_uint;
            lat *= 4;
            lng *= 4;

            uncompressedOldNormal[0] = tr.sinTable[((lat + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize]
                * tr.sinTable[lng as usize];
            uncompressedOldNormal[1] = tr.sinTable[lat as usize] * tr.sinTable[lng as usize];
            uncompressedOldNormal[2] =
                tr.sinTable[((lng + (FUNCTABLE_SIZE / 4)) & FUNCTABLE_MASK) as usize];

            *outNormal.add(0) = uncompressedOldNormal[0] * oldNormalScale
                + uncompressedNewNormal[0] * newNormalScale;
            *outNormal.add(1) = uncompressedOldNormal[1] * oldNormalScale
                + uncompressedNewNormal[1] * newNormalScale;
            *outNormal.add(2) = uncompressedOldNormal[2] * oldNormalScale
                + uncompressedNewNormal[2] * newNormalScale;

            //			VectorNormalize (outNormal);
            }

            oldXyz = oldXyz.add(4);
            newXyz = newXyz.add(4);
            oldNormals = oldNormals.add(4);
            newNormals = newNormals.add(4);
            outXyz = outXyz.add(4);
            outNormal = outNormal.add(4);
            vertNum += 1;
        }
        VectorArrayNormalize(
            tess.normal[tess.numVertexes as usize].as_mut_ptr() as *mut vec4_t,
            numVerts as c_uint,
        );
    }
}

/*
=============
RB_SurfaceMesh
=============
*/
pub unsafe fn RB_SurfaceMesh(surface: *mut md3Surface_t) {
    let mut j: c_int;
    let backlerp: f32;
    let mut triangles: *mut c_int;
    let mut texCoords: *mut f32;
    let indexes: c_int;
    let Bob: c_int;
    let Doug: c_int;
    let numVerts: c_int;

    if (*backEnd.currentEntity).e.oldframe == (*backEnd.currentEntity).e.frame {
        backlerp = 0.0;
    } else {
        backlerp = (*backEnd.currentEntity).e.backlerp;
    }

    #[cfg(feature = "VV_LIGHTING")]
    {
        if (*backEnd.currentEntity).dlightBits != 0 {
            tess.dlightBits = (*backEnd.currentEntity).dlightBits;
        }
    }

    RB_CheckOverflow((*surface).numVerts, (*surface).numTriangles * 3);

    LerpMeshVertexes(surface, backlerp);

    triangles = (surface as *mut byte).add((*surface).ofsTriangles as usize) as *mut c_int;
    indexes = (*surface).numTriangles * 3;
    Bob = tess.numIndexes;
    Doug = tess.numVertexes;
    j = 0;
    while j < indexes {
        tess.indexes[(Bob + j) as usize] = Doug + *triangles.add(j as usize);
        j += 1;
    }
    tess.numIndexes += indexes;

    texCoords = (surface as *mut byte).add((*surface).ofsSt as usize) as *mut f32;

    numVerts = (*surface).numVerts;
    j = 0;
    while j < numVerts {
        tess.texCoords[(Doug + j) as usize][0][0] = *texCoords.add(j as usize * 2 + 0);
        tess.texCoords[(Doug + j) as usize][0][1] = *texCoords.add(j as usize * 2 + 1);
        // FIXME: fill in lightmapST for completeness?
        j += 1;
    }

    tess.numVertexes += (*surface).numVerts;
}


/*
==============
RB_SurfaceFace
==============
*/
pub unsafe fn RB_SurfaceFace(surf: *mut srfSurfaceFace_t) {
    let mut i: c_int;
    let mut k: c_int;
    // VVFIXME : Sooper hack. Indices in the surface are still 32-bit, we need to make them 16 bit here.
    let mut normal: *mut f32;
    let mut ndx: c_int;
    let Bob: c_int;
    let numPoints: c_int;
    let dlightBits: c_int;

    RB_CheckOverflow((*surf).numPoints, (*surf).numIndices);

    dlightBits = (*surf).dlightBits;
    tess.dlightBits |= dlightBits;

    Bob = tess.numVertexes;

    // indices assignment and index copy loop: types differ between _XBOX and non-_XBOX
    #[cfg(feature = "_XBOX")]
    {
        let indices: *mut c_uchar =
            (surf as *mut c_char).add((*surf).ofsIndices as usize) as *mut c_uchar;
        let tessIndexes: *mut c_ushort =
            tess.indexes.as_mut_ptr().add(tess.numIndexes as usize) as *mut c_ushort;
        i = (*surf).numIndices - 1;
        while i >= 0 {
            *tessIndexes.add(i as usize) =
                (*indices.add(i as usize) as c_ushort)
                + Bob as c_ushort;
            i -= 1;
        }
    }
    #[cfg(not(feature = "_XBOX"))]
    {
        let indices: *mut c_uint =
            (surf as *mut c_char).add((*surf).ofsIndices as usize) as *mut c_uint;
        let tessIndexes: *mut glIndex_t =
            tess.indexes.as_mut_ptr().add(tess.numIndexes as usize);
        i = (*surf).numIndices - 1;
        while i >= 0 {
            *tessIndexes.add(i as usize) =
                *indices.add(i as usize) as glIndex_t + Bob as glIndex_t;
            i -= 1;
        }
    }

    tess.numIndexes += (*surf).numIndices;

    #[cfg(feature = "_XBOX")]
    {
        ndx = tess.numVertexes;

        numPoints = (*surf).numPoints;

        if (*tess.shader).needsNormal != 0 || tess.dlightBits != 0 {
            normal = (*surf).plane.normal.as_mut_ptr();
            i = 0;
            ndx = tess.numVertexes;
            while i < numPoints {
                VectorCopy(
                    &*(normal as *const vec3_t),
                    &mut *(tess.normal[ndx as usize].as_mut_ptr() as *mut vec3_t),
                );
                i += 1;
                ndx += 1;
            }
        }

        let nextSurfPoint: c_int = NEXT_SURFPOINT((*surf).flags);
        let numLightMaps: c_int = (*surf).flags & 0x7F;
        i = 0;
        let mut v: *mut c_ushort = (*surf).srfPoints;
        ndx = tess.numVertexes;
        while i < numPoints {
            Q_CastShort2Float(
                &mut tess.xyz[ndx as usize][0],
                v.add(0) as *mut i16,
            );
            Q_CastShort2Float(
                &mut tess.xyz[ndx as usize][1],
                v.add(1) as *mut i16,
            );
            Q_CastShort2Float(
                &mut tess.xyz[ndx as usize][2],
                v.add(2) as *mut i16,
            );

            Q_CastShort2Float(
                &mut tess.tangent[ndx as usize][0],
                v.add(3) as *mut i16,
            );
            Q_CastShort2Float(
                &mut tess.tangent[ndx as usize][1],
                v.add(4) as *mut i16,
            );
            Q_CastShort2Float(
                &mut tess.tangent[ndx as usize][2],
                v.add(5) as *mut i16,
            );

            tess.tangent[ndx as usize][0] /= 32767.0_f32;
            tess.tangent[ndx as usize][1] /= 32767.0_f32;
            tess.tangent[ndx as usize][2] /= 32767.0_f32;

            tess.setTangents = true;

            Q_CastShort2FloatScale(
                &mut tess.texCoords[ndx as usize][0][0],
                v.add(6) as *mut i16,
                1.0_f32 / POINTS_ST_SCALE,
            );
            Q_CastShort2FloatScale(
                &mut tess.texCoords[ndx as usize][0][1],
                v.add(7) as *mut i16,
                1.0_f32 / POINTS_ST_SCALE,
            );
            k = 0;
            while k < numLightMaps {
                if (*tess.shader).lightmapIndex[k as usize] >= 0 {
                    Q_CastUShort2FloatScale(
                        &mut tess.texCoords[ndx as usize][(k + 1) as usize][0],
                        v.add((VERTEX_LM + (k * 2) + 0) as usize),
                        1.0_f32 / POINTS_LIGHT_SCALE,
                    );
                    Q_CastUShort2FloatScale(
                        &mut tess.texCoords[ndx as usize][(k + 1) as usize][1],
                        v.add((VERTEX_LM + (k * 2) + 1) as usize),
                        1.0_f32 / POINTS_LIGHT_SCALE,
                    );
                } else {
                    //This causes problems.  See bug 57.
                    //assert(0);
                    break;
                }
                k += 1;
            }
            if ((*surf).flags & 0x80) >> 7 != 0 {
                *(tess.vertexColors[ndx as usize].as_mut_ptr() as *mut c_uint) =
                    ComputeFinalVertexColor16(
                        v.add(VERTEX_COLOR((*surf).flags) as usize) as *const byte,
                    ) as c_uint;
            }

            v = v.add(nextSurfPoint as usize);
            i += 1;
            ndx += 1;
        }
    }
    #[cfg(not(feature = "_XBOX"))]
    {
        let mut v: *mut f32 = (*surf).points[0].as_mut_ptr();

        ndx = tess.numVertexes;

        numPoints = (*surf).numPoints;

        //if ( tess.shader->needsNormal )
        {
            normal = (*surf).plane.normal.as_mut_ptr();
            i = 0;
            ndx = tess.numVertexes;
            while i < numPoints {
                VectorCopy(
                    &*(normal as *const vec3_t),
                    &mut *(tess.normal[ndx as usize].as_mut_ptr() as *mut vec3_t),
                );
                i += 1;
                ndx += 1;
            }
        }

        i = 0;
        v = (*surf).points[0].as_mut_ptr();
        ndx = tess.numVertexes;
        while i < numPoints {
            VectorCopy(&*(v as *const vec3_t), &mut *(tess.xyz[ndx as usize].as_mut_ptr() as *mut vec3_t));
            tess.texCoords[ndx as usize][0][0] = *v.add(3);
            tess.texCoords[ndx as usize][0][1] = *v.add(4);
            k = 0;
            while k < MAXLIGHTMAPS {
                if (*tess.shader).lightmapIndex[k as usize] >= 0 {
                    tess.texCoords[ndx as usize][(k + 1) as usize][0] =
                        *v.add((VERTEX_LM + (k * 2)) as usize);
                    tess.texCoords[ndx as usize][(k + 1) as usize][1] =
                        *v.add((VERTEX_LM + (k * 2) + 1) as usize);
                } else {
                    break;
                }
                k += 1;
            }
            *(tess.vertexColors[ndx as usize].as_mut_ptr() as *mut c_uint) =
                ComputeFinalVertexColor(v.add(VERTEX_COLOR as usize) as *const byte) as c_uint;
            tess.vertexDlightBits[ndx as usize] = dlightBits;

            v = v.add(VERTEXSIZE as usize);
            i += 1;
            ndx += 1;
        }
    }

    tess.numVertexes += (*surf).numPoints;
}


unsafe fn LodErrorForVolume(local: vec3_t, radius: f32) -> f32 {
    let mut world: vec3_t = [0.0; 3];
    let d: f32;

    // never let it go negative
    if (*r_lodCurveError).value < 0.0 {
        return 0.0;
    }

    world[0] = local[0] * backEnd.ori.axis[0][0]
        + local[1] * backEnd.ori.axis[1][0]
        + local[2] * backEnd.ori.axis[2][0]
        + backEnd.ori.origin[0];
    world[1] = local[0] * backEnd.ori.axis[0][1]
        + local[1] * backEnd.ori.axis[1][1]
        + local[2] * backEnd.ori.axis[2][1]
        + backEnd.ori.origin[1];
    world[2] = local[0] * backEnd.ori.axis[0][2]
        + local[1] * backEnd.ori.axis[1][2]
        + local[2] * backEnd.ori.axis[2][2]
        + backEnd.ori.origin[2];

    VectorSubtract(&world.clone(), &backEnd.viewParms.ori.origin, &mut world);
    let mut d = DotProduct(&world, &backEnd.viewParms.ori.axis[0]);

    if d < 0.0 {
        d = -d;
    }
    d -= radius;
    if d < 1.0 {
        d = 1.0;
    }

    (*r_lodCurveError).value / d
}

/*
=============
RB_SurfaceGrid

Just copy the grid of points and triangulate
=============
*/
pub unsafe fn RB_SurfaceGrid(cv: *mut srfGridMesh_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut xyz: *mut f32;
    let mut texCoords: *mut f32;
    let mut normal: *mut f32;
    let mut color: *mut c_uchar;
    let mut dv: *mut drawVert_t;
    let mut rows: c_int;
    let mut irows: c_int;
    let mut vrows: c_int;
    let mut used: c_int;
    let mut widthTable: [c_int; MAX_GRID_SIZE as usize] = [0; MAX_GRID_SIZE as usize];
    let mut heightTable: [c_int; MAX_GRID_SIZE as usize] = [0; MAX_GRID_SIZE as usize];
    let mut lodError: f32;
    let mut lodWidth: c_int;
    let mut lodHeight: c_int;
    let mut numVertexes: c_int;
    let dlightBits: c_int;
    let mut vDlightBits: *mut c_int;

    dlightBits = (*cv).dlightBits;
    tess.dlightBits |= dlightBits;

    // determine the allowable discrepance
    lodError = LodErrorForVolume((*cv).lodOrigin, (*cv).lodRadius);

    // determine which rows and columns of the subdivision
    // we are actually going to use
    widthTable[0] = 0;
    lodWidth = 1;
    i = 1;
    while i < (*cv).width - 1 {
        if (*cv).widthLodError[i as usize] <= lodError {
            widthTable[lodWidth as usize] = i;
            lodWidth += 1;
        }
        i += 1;
    }
    widthTable[lodWidth as usize] = (*cv).width - 1;
    lodWidth += 1;

    heightTable[0] = 0;
    lodHeight = 1;
    i = 1;
    while i < (*cv).height - 1 {
        if (*cv).heightLodError[i as usize] <= lodError {
            heightTable[lodHeight as usize] = i;
            lodHeight += 1;
        }
        i += 1;
    }
    heightTable[lodHeight as usize] = (*cv).height - 1;
    lodHeight += 1;


    // very large grids may have more points or indexes than can be fit
    // in the tess structure, so we may have to issue it in multiple passes

    used = 0;
    rows = 0;
    while used < lodHeight - 1 {
        // see how many rows of both verts and indexes we can add without overflowing
        loop {
            vrows = (SHADER_MAX_VERTEXES - tess.numVertexes) / lodWidth;
            irows = (SHADER_MAX_INDEXES - tess.numIndexes) / (lodWidth * 6);

            // if we don't have enough space for at least one strip, flush the buffer
            if vrows < 2 || irows < 1 {
                RB_EndSurface();
                RB_BeginSurface(tess.shader, tess.fogNum);
            } else {
                break;
            }
        }

        rows = irows;
        if vrows < irows + 1 {
            rows = vrows - 1;
        }
        if used + rows > lodHeight {
            rows = lodHeight - used;
        }

        numVertexes = tess.numVertexes;

        xyz = tess.xyz[numVertexes as usize].as_mut_ptr();
        normal = tess.normal[numVertexes as usize].as_mut_ptr();
        texCoords = tess.texCoords[numVertexes as usize][0].as_mut_ptr();
        color = tess.vertexColors[numVertexes as usize].as_mut_ptr() as *mut c_uchar;
        vDlightBits = &mut tess.vertexDlightBits[numVertexes as usize] as *mut c_int;

        #[cfg(feature = "_XBOX")]
        {
            i = 0;
            while i < rows {
                j = 0;
                while j < lodWidth {
                    dv = (*cv).verts.add(
                        (heightTable[(used + i) as usize] * (*cv).width
                            + widthTable[j as usize]) as usize,
                    );

                    *xyz.add(0) = (*dv).xyz[0];
                    *xyz.add(1) = (*dv).xyz[1];
                    *xyz.add(2) = (*dv).xyz[2];
                    xyz = xyz.add(4);

                    Q_CastShort2FloatScale(
                        texCoords.add(0),
                        &(*dv).dvst[0],
                        1.0_f32 / GRID_DRAWVERT_ST_SCALE,
                    );
                    Q_CastShort2FloatScale(
                        texCoords.add(1),
                        &(*dv).dvst[1],
                        1.0_f32 / GRID_DRAWVERT_ST_SCALE,
                    );

                    k = 0;
                    while k < MAXLIGHTMAPS {
                        Q_CastShort2FloatScale(
                            texCoords.add(2 + (k as usize * 2) + 0),
                            &(*dv).dvlightmap[k as usize][0],
                            1.0_f32 / DRAWVERT_LIGHTMAP_SCALE,
                        );
                        Q_CastShort2FloatScale(
                            texCoords.add(2 + (k as usize * 2) + 1),
                            &(*dv).dvlightmap[k as usize][1],
                            1.0_f32 / DRAWVERT_LIGHTMAP_SCALE,
                        );
                        k += 1;
                    }
                    texCoords = texCoords.add(NUM_TEX_COORDS as usize * 2);

                    if (*tess.shader).needsNormal != 0 || tess.dlightBits != 0 {
                        *normal.add(0) = (*dv).normal[0];
                        *normal.add(1) = (*dv).normal[1];
                        *normal.add(2) = (*dv).normal[2];
                        normal = normal.add(4);
                    }

                    *(color as *mut c_uint) =
                        ComputeFinalVertexColor16((*dv).dvcolor.as_ptr()) as c_uint;
                    color = color.add(4);
                    *vDlightBits = dlightBits;
                    vDlightBits = vDlightBits.add(1);

                    j += 1;
                }
                i += 1;
            }
        }
        #[cfg(not(feature = "_XBOX"))]
        {
            i = 0;
            while i < rows {
                j = 0;
                while j < lodWidth {
                    dv = (*cv).verts.add(
                        (heightTable[(used + i) as usize] * (*cv).width
                            + widthTable[j as usize]) as usize,
                    );

                    *xyz.add(0) = (*dv).xyz[0];
                    *xyz.add(1) = (*dv).xyz[1];
                    *xyz.add(2) = (*dv).xyz[2];
                    xyz = xyz.add(4);

                    *texCoords.add(0) = (*dv).st[0];
                    *texCoords.add(1) = (*dv).st[1];
                    k = 0;
                    while k < MAXLIGHTMAPS {
                        *texCoords.add(2 + (k as usize * 2)) = (*dv).lightmap[k as usize][0];
                        *texCoords.add(2 + (k as usize * 2) + 1) = (*dv).lightmap[k as usize][1];
                        k += 1;
                    }
                    texCoords = texCoords.add(NUM_TEX_COORDS as usize * 2);

                    //if ( needsNormal )
                    {
                        *normal.add(0) = (*dv).normal[0];
                        *normal.add(1) = (*dv).normal[1];
                        *normal.add(2) = (*dv).normal[2];
                    }
                    normal = normal.add(4);

                    *(color as *mut c_uint) =
                        ComputeFinalVertexColor((*dv).color.as_ptr()) as c_uint;
                    color = color.add(4);
                    *vDlightBits = dlightBits;
                    vDlightBits = vDlightBits.add(1);

                    j += 1;
                }
                i += 1;
            }
        }

        // add the indexes
        {
            let mut numIndexes: c_int;
            let w: c_int;
            let h: c_int;

            h = rows - 1;
            w = lodWidth - 1;
            numIndexes = tess.numIndexes;
            i = 0;
            while i < h {
                j = 0;
                while j < w {
                    let v1: c_int;
                    let v2: c_int;
                    let v3: c_int;
                    let v4: c_int;

                    // vertex order to be reckognized as tristrips
                    v1 = numVertexes + i * lodWidth + j + 1;
                    v2 = v1 - 1;
                    v3 = v2 + lodWidth;
                    v4 = v3 + 1;

                    tess.indexes[numIndexes as usize] = v2;
                    tess.indexes[numIndexes as usize + 1] = v3;
                    tess.indexes[numIndexes as usize + 2] = v1;

                    tess.indexes[numIndexes as usize + 3] = v1;
                    tess.indexes[numIndexes as usize + 4] = v3;
                    tess.indexes[numIndexes as usize + 5] = v4;
                    numIndexes += 6;

                    j += 1;
                }
                i += 1;
            }

            tess.numIndexes = numIndexes;
        }

        tess.numVertexes += rows * lodWidth;

        used += rows - 1;
    }
}


/*
===========================================================================

NULL MODEL

===========================================================================
*/

/*
===================
RB_SurfaceAxis

Draws x/y/z lines from the origin for orientation debugging
===================
*/
unsafe fn RB_SurfaceAxis() {
    GL_Bind(tr.whiteImage);
    qglLineWidth(3.0);
    #[cfg(feature = "_XBOX")]
    {
        qglBeginEXT(GL_LINES, 6, 3, 0, 0, 0);
    }
    #[cfg(not(feature = "_XBOX"))]
    {
        qglBegin(GL_LINES);
    }
    qglColor3f(1.0, 0.0, 0.0);
    qglVertex3f(0.0, 0.0, 0.0);
    qglVertex3f(16.0, 0.0, 0.0);
    qglColor3f(0.0, 1.0, 0.0);
    qglVertex3f(0.0, 0.0, 0.0);
    qglVertex3f(0.0, 16.0, 0.0);
    qglColor3f(0.0, 0.0, 1.0);
    qglVertex3f(0.0, 0.0, 0.0);
    qglVertex3f(0.0, 0.0, 16.0);
    qglEnd();
    qglLineWidth(1.0);
}

//===========================================================================

/*
====================
RB_SurfaceEntity

Entities that have a single procedurally generated surface
====================
*/
pub unsafe fn RB_SurfaceEntity(surfType: *mut surfaceType_t) {
    match (*backEnd.currentEntity).e.reType {
        RT_SPRITE => {
            RB_SurfaceSprite();
        }
        RT_ORIENTED_QUAD => {
            RB_SurfaceOrientedQuad();
        }
        RT_BEAM => {
            RB_SurfaceBeam();
        }
        RT_ELECTRICITY => {
            RB_SurfaceElectricity();
        }
        RT_LINE => {
            RB_SurfaceLine();
        }
        RT_ORIENTEDLINE => {
            RB_SurfaceOrientedLine();
        }
        RT_SABER_GLOW => {
            RB_SurfaceSaberGlow();
        }
        RT_CYLINDER => {
            RB_SurfaceCylinder();
        }
        RT_ENT_CHAIN => {
            let mut i: c_int;
            let count: c_int;
            let start: c_int;
            static mut TEMP_ENT: trRefEntity_t = unsafe { core::mem::zeroed() };
            //rww - if not static then currentEntity is garbage because
            //this is a local. This was not static in sof2.. but I guess
            //they never check ce.renderfx so it didn't show up.

            start = (*backEnd.currentEntity).e.uRefEnt.uMini.miniStart;
            count = (*backEnd.currentEntity).e.uRefEnt.uMini.miniCount;
            assert!(count > 0);
            TEMP_ENT = *backEnd.currentEntity;
            backEnd.currentEntity = addr_of_mut!(TEMP_ENT);

            assert!((*backEnd.currentEntity).e.renderfx >= 0);

            i = 0;
            while i < count {
                // memcpy(&backEnd.currentEntity->e, &backEnd.refdef.miniEntities[start+i], sizeof(...))
                core::ptr::copy_nonoverlapping::<u8>(
                    &backEnd.refdef.miniEntities[(start + i) as usize]
                        as *const _ as *const u8,
                    &mut (*backEnd.currentEntity).e as *mut _ as *mut u8,
                    core::mem::size_of_val(
                        &backEnd.refdef.miniEntities[(start + i) as usize],
                    ),
                );

                assert!((*backEnd.currentEntity).e.renderfx >= 0);

                RB_SurfaceEntity(surfType);
                i += 1;
            }
        }
        _ => {
            RB_SurfaceAxis();
        }
    }
}

pub unsafe fn RB_SurfaceBad(surfType: *mut surfaceType_t) {
    Com_Printf(b"Bad surface tesselated.\n\0".as_ptr() as *const c_char);
}

/*
==================
RB_TestZFlare

This is called at surface tesselation time
==================
*/
unsafe fn RB_TestZFlare(point: vec3_t) -> bool {
    let mut i: c_int;
    let mut eye: vec4_t = [0.0; 4];
    let mut clip: vec4_t = [0.0; 4];
    let mut normalized: vec4_t = [0.0; 4];
    let mut window: vec4_t = [0.0; 4];

    // if the point is off the screen, don't bother adding it
    // calculate screen coordinates and depth
    R_TransformModelToClip(
        &point,
        backEnd.ori.modelMatrix.as_ptr(),
        backEnd.viewParms.projectionMatrix.as_ptr(),
        &mut eye,
        &mut clip,
    );

    // check to see if the point is completely off screen
    i = 0;
    while i < 3 {
        if clip[i as usize] >= clip[3] || clip[i as usize] <= -clip[3] {
            return qfalse != 0;
        }
        i += 1;
    }

    R_TransformClipToWindow(
        &clip,
        &backEnd.viewParms as *const _,
        &mut normalized,
        &mut window,
    );

    if window[0] < 0.0
        || window[0] >= backEnd.viewParms.viewportWidth as f32
        || window[1] < 0.0
        || window[1] >= backEnd.viewParms.viewportHeight as f32
    {
        return qfalse != 0; // shouldn't happen, since we check the clip[] above, except for FP rounding
    }

    //do test
    let mut depth: f32 = 0.0_f32;
    let visible: bool;
    let screenZ: f32;

    // read back the z buffer contents
    #[cfg(feature = "_XBOX")]
    {
        depth = 0.0_f32;
    }
    #[cfg(not(feature = "_XBOX"))]
    {
        if (*r_flares).integer != 1 {
            //skipping the the z-test
            return true;
        }
        // doing a readpixels is as good as doing a glFinish(), so
        // don't bother with another sync
        glState.finishCalled = qfalse;
        qglReadPixels(
            backEnd.viewParms.viewportX + window[0] as c_int,
            backEnd.viewParms.viewportY + window[1] as c_int,
            1,
            1,
            GL_DEPTH_COMPONENT,
            GL_FLOAT,
            &mut depth as *mut f32 as *mut c_void,
        );
    }

    screenZ = backEnd.viewParms.projectionMatrix[14]
        / ((2.0 * depth - 1.0) * backEnd.viewParms.projectionMatrix[11]
            - backEnd.viewParms.projectionMatrix[10]);

    visible = (-eye[2] - -screenZ) < 24.0;
    visible
}

pub unsafe fn RB_SurfaceFlare(surf: *mut srfFlare_t) {
    let mut left: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut radius: f32;
    let mut color: [byte; 4] = [0; 4];
    let mut dir: vec3_t = [0.0; 3];
    let mut origin: vec3_t = [0.0; 3];
    let mut d: f32;
    let dist: f32;
    // snormal: in _XBOX path this is a decoded vec3_t; in non-XBOX path it aliases surf->normal
    let mut snormal: vec3_t = [0.0; 3];

    if (*r_flares).integer == 0 {
        return;
    }

    #[cfg(feature = "_XBOX")]
    {
        let mut sorigin: vec3_t = [0.0; 3];

        Q_CastShort2Float(&mut sorigin[0], (*surf).origin.as_ptr() as *mut i16);
        Q_CastShort2Float(&mut sorigin[1], ((*surf).origin.as_ptr() as *mut i16).add(1));
        Q_CastShort2Float(&mut sorigin[2], ((*surf).origin.as_ptr() as *mut i16).add(2));
        Q_CastShort2Float(&mut snormal[0], (*surf).normal.as_ptr() as *mut i16);
        Q_CastShort2Float(&mut snormal[1], ((*surf).normal.as_ptr() as *mut i16).add(1));
        Q_CastShort2Float(&mut snormal[2], ((*surf).normal.as_ptr() as *mut i16).add(2));
        snormal[0] /= 32767.0_f32;
        snormal[1] /= 32767.0_f32;
        snormal[2] /= 32767.0_f32;

        if !RB_TestZFlare(sorigin) {
            return;
        }

        // calculate the xyz locations for the four corners
        VectorMA(&sorigin, 3.0, &snormal, &mut origin);
    }
    #[cfg(not(feature = "_XBOX"))]
    {
        if !RB_TestZFlare((*surf).origin) {
            return;
        }

        // calculate the xyz locations for the four corners
        VectorMA(&(*surf).origin, 3.0, &(*surf).normal, &mut origin);
        // float* snormal = surf->normal;
        VectorCopy(&(*surf).normal, &mut snormal);
    }

    VectorSubtract(&origin, &backEnd.viewParms.ori.origin, &mut dir);
    dist = VectorNormalize(&mut dir);

    d = -DotProduct(&dir, &snormal);
    if d < 0.0 {
        d = -d;
    }

    // fade the intensity of the flare down as the
    // light surface turns away from the viewer
    color[0] = (d * 255.0) as byte;
    color[1] = (d * 255.0) as byte;
    color[2] = (d * 255.0) as byte;
    color[3] = 255; //only gets used if the shader has cgen exact_vertex!

    radius = if (*tess.shader).portalRange != 0.0 {
        (*tess.shader).portalRange
    } else {
        30.0
    };
    if dist < 512.0_f32 {
        radius = radius * dist / 512.0_f32;
    }
    if radius < 5.0_f32 {
        radius = 5.0_f32;
    }
    VectorScale(&backEnd.viewParms.ori.axis[1], radius, &mut left);
    VectorScale(&backEnd.viewParms.ori.axis[2], radius, &mut up);
    if backEnd.viewParms.isMirror != 0 {
        VectorSubtract(&vec3_origin, &left.clone(), &mut left);
    }

    RB_AddQuadStamp(origin, left, up, color.as_mut_ptr());
}


pub unsafe fn RB_SurfaceDisplayList(surf: *mut srfDisplayList_t) {
    // all apropriate state must be set in RB_BeginSurface
    // this isn't implemented yet...
    qglCallList((*surf).listNum);
}

pub unsafe fn RB_SurfaceSkip(surf: *mut c_void) {}


pub static rb_surfaceTable: [unsafe fn(*mut c_void); SF_NUM_SURFACE_TYPES as usize] = unsafe {
    [
        core::mem::transmute::<unsafe fn(*mut surfaceType_t), unsafe fn(*mut c_void)>(
            RB_SurfaceBad,
        ),       // SF_BAD,
        core::mem::transmute::<unsafe fn(*mut c_void), unsafe fn(*mut c_void)>(RB_SurfaceSkip), // SF_SKIP,
        core::mem::transmute::<unsafe fn(*mut srfSurfaceFace_t), unsafe fn(*mut c_void)>(
            RB_SurfaceFace,
        ),       // SF_FACE,
        core::mem::transmute::<unsafe fn(*mut srfGridMesh_t), unsafe fn(*mut c_void)>(
            RB_SurfaceGrid,
        ),       // SF_GRID,
        core::mem::transmute::<unsafe fn(*mut srfTriangles_t), unsafe fn(*mut c_void)>(
            RB_SurfaceTriangles,
        ),       // SF_TRIANGLES,
        core::mem::transmute::<unsafe fn(*mut srfPoly_t), unsafe fn(*mut c_void)>(
            RB_SurfacePolychain,
        ),       // SF_POLY,
        core::mem::transmute::<unsafe fn(*mut c_void), unsafe fn(*mut c_void)>(
            RB_SurfaceTerrain,
        ),       // SF_TERRAIN, //rwwRMG - added
        core::mem::transmute::<unsafe fn(*mut md3Surface_t), unsafe fn(*mut c_void)>(
            RB_SurfaceMesh,
        ),       // SF_MD3,
        /*
        Ghoul2 Insert Start
        */
        core::mem::transmute::<unsafe fn(*mut c_void), unsafe fn(*mut c_void)>(RB_SurfaceGhoul), // SF_MDX,
        /*
        Ghoul2 Insert End
        */
        core::mem::transmute::<unsafe fn(*mut srfFlare_t), unsafe fn(*mut c_void)>(
            RB_SurfaceFlare,
        ),       // SF_FLARE,
        core::mem::transmute::<unsafe fn(*mut surfaceType_t), unsafe fn(*mut c_void)>(
            RB_SurfaceEntity,
        ),       // SF_ENTITY
        core::mem::transmute::<unsafe fn(*mut srfDisplayList_t), unsafe fn(*mut c_void)>(
            RB_SurfaceDisplayList,
        ),       // SF_DISPLAY_LIST
    ]
};
