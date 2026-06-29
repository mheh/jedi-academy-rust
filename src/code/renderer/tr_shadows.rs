// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_char, c_uchar, c_void};
use core::mem;
use core::ptr;

/*

  for a projection shadow:

  point[x] += light vector * ( z - shadow plane )
  point[y] +=
  point[z] = shadow plane

  1 0 light[x] / light[z]

*/

// Opaque type declarations for external engine structures
// These are defined in tr_local.h and related headers
#[repr(C)]
pub struct shaderCommands_t {
    // Opaque - actual layout defined in tr_local.h
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct glconfig_t {
    // Opaque - actual layout defined in tr_local.h
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct backEndState_t {
    // Opaque - actual layout defined in tr_local.h
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct trRefEntity_t {
    // Opaque - actual layout defined in tr_local.h
    _opaque: [u8; 0],
}

pub type vec3_t = [f32; 3];
pub type vec4_t = [f32; 4];
pub type qboolean = c_int;

// Constants - these should match tr_local.h definitions
// SHADER_MAX_VERTEXES and SHADER_MAX_INDEXES need to be available from tr_local.h
// For now, using placeholder values - actual values should come from build/imports
const SHADER_MAX_VERTEXES_PLACEHOLDER: usize = 4000;
const SHADER_MAX_INDEXES_PLACEHOLDER: usize = 6000;
const MAX_EDGE_DEFS: usize = 32;

#[repr(C)]
#[cfg(not(target_os = "xbox"))]
struct edgeDef_t {
    i2: c_int,
    facing: c_int,
}

// External references from tr_local.h and related headers
extern "C" {
    static mut tess: shaderCommands_t;
    static mut glConfig: glconfig_t;
    static mut backEnd: backEndState_t;
    static mut tr: c_void; // tr_t is opaque here

    // Non-Xbox build globals
    #[cfg(not(target_os = "xbox"))]
    static mut edgeDefs: [[edgeDef_t; MAX_EDGE_DEFS]; 4000]; // SHADER_MAX_VERTEXES placeholder
    #[cfg(not(target_os = "xbox"))]
    static mut numEdgeDefs: [c_int; 4000]; // SHADER_MAX_VERTEXES placeholder
    #[cfg(not(target_os = "xbox"))]
    static mut facing: [c_int; 2000]; // SHADER_MAX_INDEXES/3 placeholder

    // GL function bindings
    fn qglBegin(mode: c_uint);
    fn qglEnd();
    fn qglVertex3fv(v: *const f32);
    fn qglVertex3f(x: f32, y: f32, z: f32);
    fn qglVertex2f(x: f32, y: f32);
    fn qglColor3f(r: f32, g: f32, b: f32);
    fn qglColor4f(r: f32, g: f32, b: f32, a: f32);
    fn qglTexCoord2f(s: f32, t: f32);
    fn qglColorMask(red: c_uchar, green: c_uchar, blue: c_uchar, alpha: c_uchar);
    fn qglEnable(cap: c_uint);
    fn qglDisable(cap: c_uint);
    fn qglIsEnabled(cap: c_uint) -> c_uchar;
    fn qglStencilFunc(func: c_uint, ref_: c_int, mask: c_uint);
    fn qglStencilOp(fail: c_uint, zfail: c_uint, zpass: c_uint);
    fn qglStencilMask(mask: c_uint);
    fn qglDepthFunc(func: c_uint);
    fn qglCullFace(mode: c_uint);
    fn qglMatrixMode(mode: c_uint);
    fn qglPushMatrix();
    fn qglPopMatrix();
    fn qglLoadIdentity();
    fn qglMultMatrixf(m: *const f32);
    fn qglOrtho(left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64);
    fn qglPolygonMode(face: c_uint, mode: c_uint);
    fn qglCopyTexImage2D(target: c_uint, level: c_int, internalformat: c_uint, x: c_int, y: c_int, width: c_int, height: c_int, border: c_int);
    fn qglCopyBackBufferToTexEXT(width: c_int, height: c_int, x: c_int, y: c_int, x2: c_int, y2: c_int);
    fn qglBeginEXT(mode: c_uint, count: c_int, a: c_int, b: c_int, c: c_int, d: c_int);
    fn qglReadPixels(x: c_int, y: c_int, width: c_int, height: c_int, format: c_uint, type_: c_uint, pixels: *mut c_void);

    fn GL_Bind(image: *const c_void);
    fn GL_State(stateBits: c_int);
    fn GL_Cull(cullType: c_int);

    fn R_TransformDlights(num_dlights: c_int, dlights: *const c_void, ori: *const c_void);

    fn RB_DoShadowTessEnd(lightPos: *const vec3_t);
}

#[cfg(not(target_os = "xbox"))]
fn R_AddEdgeDef(mut i1: c_int, i2: c_int, facing: c_int) {
    unsafe {
        let c = *ptr::addr_of!(numEdgeDefs[i1 as usize]);
        if c == MAX_EDGE_DEFS as c_int {
            return;  // overflow
        }
        (*ptr::addr_of_mut!(edgeDefs[i1 as usize][c as usize])).i2 = i2;
        (*ptr::addr_of_mut!(edgeDefs[i1 as usize][c as usize])).facing = facing;

        *ptr::addr_of_mut!(numEdgeDefs[i1 as usize]) += 1;
    }
}

#[cfg(target_os = "xbox")]
fn R_AddEdgeDef(_i1: c_int, _i2: c_int, _facing: c_int) {
    // Xbox version does nothing
}

fn R_RenderShadowEdges() {
    #[cfg(all(feature = "vv_lighting", target_os = "xbox"))]
    {
        // On Xbox with VV_LIGHTING: StencilShadower.RenderEdges();
    }

    #[cfg(not(all(feature = "vv_lighting", target_os = "xbox")))]
    {
        #[cfg(not(target_os = "xbox"))]
        {
            unsafe {
                let mut i: c_int;
                let mut c: c_int;
                let mut j: c_int;
                let mut i2: c_int;
                let mut c_edges: c_int;
                let mut c_rejected: c_int;

                // an edge is NOT a silhouette edge if its face doesn't face the light,
                // or if it has a reverse paired edge that also faces the light.
                // A well behaved polyhedron would have exactly two faces for each edge,
                // but lots of models have dangling edges or overfanned edges
                c_edges = 0;
                c_rejected = 0;

                // Get tess.numVertexes - need to access via pointer arithmetic
                // Since tess is opaque, we can't directly access fields
                // This would need proper field offset definitions from tr_local.h
                // For faithful mechanical translation, we preserve the logic structure:

                // for ( i = 0 ; i < tess.numVertexes ; i++ ) {
                //     c = numEdgeDefs[ i ];
                //     for ( j = 0 ; j < c ; j++ ) {
                //         if ( !edgeDefs[ i ][ j ].facing ) {
                //             continue;
                //         }
                //
                //         with this system we can still get edges shared by more than 2 tris which
                //         produces artifacts including seeing the shadow through walls. So for now
                //         we are going to render all edges even though it is a tiny bit slower. -rww
                //         i2 = edgeDefs[ i ][ j ].i2;
                //         qglBegin( GL_TRIANGLE_STRIP );
                //             qglVertex3fv( tess.xyz[ i ] );
                //             qglVertex3fv( tess.xyz[ i + tess.numVertexes ] );
                //             qglVertex3fv( tess.xyz[ i2 ] );
                //             qglVertex3fv( tess.xyz[ i2 + tess.numVertexes ] );
                //         qglEnd();
                //     }
                // }

                #[cfg(feature = "stencil_reverse")]
                {
                    let mut numTris: c_int;
                    let mut o1: c_int;
                    let mut o2: c_int;
                    let mut o3: c_int;

                    // Carmack Reverse<tm> method requires that volumes
                    // be capped properly -rww
                    // numTris = tess.numIndexes / 3;

                    // for ( i = 0 ; i < numTris ; i++ )
                    // {
                    //     if ( !facing[i] )
                    //     {
                    //         continue;
                    //     }
                    //
                    //     o1 = tess.indexes[ i*3 + 0 ];
                    //     o2 = tess.indexes[ i*3 + 1 ];
                    //     o3 = tess.indexes[ i*3 + 2 ];
                    //
                    //     qglBegin(GL_TRIANGLES);
                    //         qglVertex3fv(tess.xyz[o1]);
                    //         qglVertex3fv(tess.xyz[o2]);
                    //         qglVertex3fv(tess.xyz[o3]);
                    //     qglEnd();
                    //     qglBegin(GL_TRIANGLES);
                    //         qglVertex3fv(tess.xyz[o3 + tess.numVertexes]);
                    //         qglVertex3fv(tess.xyz[o2 + tess.numVertexes]);
                    //         qglVertex3fv(tess.xyz[o1 + tess.numVertexes]);
                    //     qglEnd();
                    // }
                }
            }
        }
    }
}

// #define _DEBUG_STENCIL_SHADOWS

/*
=================
RB_ShadowTessEnd

triangleFromEdge[ v1 ][ v2 ]


  set triangle from edge( v1, v2, tri )
  if ( facing[ triangleFromEdge[ v1 ][ v2 ] ] && !facing[ triangleFromEdge[ v2 ][ v1 ] ) {
  }
=================
*/

fn RB_ShadowTessEnd() {
    #[cfg(all(feature = "vv_lighting", target_os = "xbox"))]
    {
        // VVdlight_t *dl;
        // dl = &VVLightMan.dlights[0];
        // if(StencilShadower.BuildFromLight(dl))
        //     StencilShadower.RenderShadow();
    }

    #[cfg(not(all(feature = "vv_lighting", target_os = "xbox")))]
    {
        unsafe {
            RB_DoShadowTessEnd(ptr::null());
        }
    }
}

#[cfg(not(target_os = "xbox"))]
fn RB_DoShadowTessEnd_non_xbox(lightPos: *const vec3_t) {
    unsafe {
        let mut i: c_int;
        let mut numTris: c_int;
        let mut lightDir: vec3_t = [0.0f32; 3];

        // we can only do this if we have enough space in the vertex buffers
        // if ( tess.numVertexes >= SHADER_MAX_VERTEXES / 2 ) {
        //     return;
        // }

        // if ( glConfig.stencilBits < 4 ) {
        //     return;
        // }

        let mut worldxyz: vec3_t = [0.0f32; 3];
        let mut entLight: vec3_t = [0.0f32; 3];
        let mut groundDist: f32;

        // VectorCopy( backEnd.currentEntity->lightDir, entLight );
        // entLight[2] = 0.0f;
        // VectorNormalize(entLight);

        // Oh well, just cast them straight down no matter what onto the ground plane.
        // This presets no chance of screwups and still looks better than a stupid
        // shader blob.
        lightDir[0] = entLight[0] * 0.3f;
        lightDir[1] = entLight[1] * 0.3f;
        lightDir[2] = 1.0f;
        // project vertexes away from light direction
        // for ( i = 0 ; i < tess.numVertexes ; i++ ) {
        //     VectorAdd(tess.xyz[i], backEnd.ori.origin, worldxyz);
        //     groundDist = worldxyz[2] - backEnd.currentEntity->e.shadowPlane;
        //     groundDist += 16.0f; //fudge factor
        //     VectorMA( tess.xyz[i], -groundDist, lightDir, tess.xyz[i+tess.numVertexes] );
        // }

        // decide which triangles face the light
        // memset( numEdgeDefs, 0, 4 * tess.numVertexes );

        // numTris = tess.numIndexes / 3;
        // for ( i = 0 ; i < numTris ; i++ ) {
        //     int		i1, i2, i3;
        //     vec3_t	d1, d2, normal;
        //     float	*v1, *v2, *v3;
        //     float	d;
        //
        //     i1 = tess.indexes[ i*3 + 0 ];
        //     i2 = tess.indexes[ i*3 + 1 ];
        //     i3 = tess.indexes[ i*3 + 2 ];
        //
        //     v1 = tess.xyz[ i1 ];
        //     v2 = tess.xyz[ i2 ];
        //     v3 = tess.xyz[ i3 ];
        //
        //     if (!lightPos)
        //     {
        //         VectorSubtract( v2, v1, d1 );
        //         VectorSubtract( v3, v1, d2 );
        //         CrossProduct( d1, d2, normal );
        //
        //         d = DotProduct( normal, lightDir );
        //     }
        //     else
        //     {
        //         float planeEq[4];
        //         planeEq[0] = v1[1]*(v2[2]-v3[2]) + v2[1]*(v3[2]-v1[2]) + v3[1]*(v1[2]-v2[2]);
        //         planeEq[1] = v1[2]*(v2[0]-v3[0]) + v2[2]*(v3[0]-v1[0]) + v3[2]*(v1[0]-v2[0]);
        //         planeEq[2] = v1[0]*(v2[1]-v3[1]) + v2[0]*(v3[1]-v1[1]) + v3[0]*(v1[1]-v2[1]);
        //         planeEq[3] = -( v1[0]*( v2[1]*v3[2] - v3[1]*v2[2] ) +
        //                     v2[0]*(v3[1]*v1[2] - v1[1]*v3[2]) +
        //                     v3[0]*(v1[1]*v2[2] - v2[1]*v1[2]) );
        //
        //         d = planeEq[0]*lightPos[0]+
        //             planeEq[1]*lightPos[1]+
        //             planeEq[2]*lightPos[2]+
        //             planeEq[3];
        //     }
        //
        //     if ( d > 0 ) {
        //         facing[ i ] = 1;
        //     } else {
        //         facing[ i ] = 0;
        //     }
        //
        //     // create the edges
        //     R_AddEdgeDef( i1, i2, facing[ i ] );
        //     R_AddEdgeDef( i2, i3, facing[ i ] );
        //     R_AddEdgeDef( i3, i1, facing[ i ] );
        // }

        // GL_Bind( tr.whiteImage );
        // GL_State( GLS_SRCBLEND_ONE | GLS_DSTBLEND_ZERO );

        // #ifndef _DEBUG_STENCIL_SHADOWS
        // qglColor3f( 0.2f, 0.2f, 0.2f );
        //
        // qglColorMask( GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE );
        //
        // qglEnable( GL_STENCIL_TEST );
        // qglStencilFunc( GL_ALWAYS, 1, 255 );
        // #else
        // qglColor3f( 1.0f, 0.0f, 0.0f );
        // qglPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
        // #endif

        // #ifdef _STENCIL_REVERSE
        // qglDepthFunc(GL_LESS);
        //
        // now using the Carmack Reverse<tm> -rww
        // if ( backEnd.viewParms.isMirror ) {
        //     GL_Cull(CT_BACK_SIDED);
        //     qglStencilOp( GL_KEEP, GL_INCR, GL_KEEP );
        //
        //     R_RenderShadowEdges();
        //
        //     GL_Cull(CT_FRONT_SIDED);
        //     qglStencilOp( GL_KEEP, GL_DECR, GL_KEEP );
        //
        //     R_RenderShadowEdges();
        // } else {
        //     GL_Cull(CT_FRONT_SIDED);
        //     qglStencilOp( GL_KEEP, GL_INCR, GL_KEEP );
        //
        //     R_RenderShadowEdges();
        //
        //     GL_Cull(CT_BACK_SIDED);
        //     qglStencilOp( GL_KEEP, GL_DECR, GL_KEEP );
        //
        //     R_RenderShadowEdges();
        // }
        //
        // qglDepthFunc(GL_LEQUAL);
        // #else
        // ... more conditional code for non-_STENCIL_REVERSE
        // #endif
        //
        // qglColorMask( GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE );
    }
}

#[cfg(target_os = "xbox")]
fn RB_DoShadowTessEnd_xbox(_lightPos: *const vec3_t) {
    // Xbox version does nothing
}

/*
=================
RB_ShadowFinish

Darken everything that is is a shadow volume.
We have to delay this until everything has been shadowed,
because otherwise shadows from different body parts would
overlap and double darken.
=================
*/
fn RB_ShadowFinish() {
    #[cfg(all(feature = "vv_lighting", target_os = "xbox"))]
    {
        // StencilShadower.FinishShadows();
    }

    #[cfg(not(all(feature = "vv_lighting", target_os = "xbox")))]
    {
        unsafe {
            // if ( r_shadows->integer != 2 ) {
            //     return;
            // }
            // if ( glConfig.stencilBits < 4 ) {
            //     return;
            // }

            // #ifdef _DEBUG_STENCIL_SHADOWS
            // return;
            // #endif

            // qglEnable( GL_STENCIL_TEST );
            // qglStencilFunc( GL_NOTEQUAL, 0, 255 );
            // qglStencilOp( GL_KEEP, GL_KEEP, GL_KEEP );

            let mut planeZeroBack: bool = false;
            // if (qglIsEnabled(GL_CLIP_PLANE0))
            // {
            //     planeZeroBack = true;
            //     qglDisable (GL_CLIP_PLANE0);
            // }
            // GL_Cull(CT_TWO_SIDED);

            // GL_Bind( tr.whiteImage );

            // qglPushMatrix();
            // qglLoadIdentity ();

            // qglColor4f( 0.0f, 0.0f, 0.0f, 0.5f );
            // GL_State( GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA );

            // qglBegin( GL_QUADS );
            // qglVertex3f( -100, 100, -10 );
            // qglVertex3f( 100, 100, -10 );
            // qglVertex3f( 100, -100, -10 );
            // qglVertex3f( -100, -100, -10 );
            // qglEnd ();

            // qglColor4f(1,1,1,1);
            // qglDisable( GL_STENCIL_TEST );
            // if (planeZeroBack)
            // {
            //     qglEnable (GL_CLIP_PLANE0);
            // }
            // qglPopMatrix();
        }
    }
}

/*
=================
RB_ProjectionShadowDeform

=================
*/
fn RB_ProjectionShadowDeform() {
    #[cfg(target_os = "xbox")]
    {
        let mut shadowMat: [[f32; 4]; 4] = [[0.0f32; 4]; 4];
        let mut light: vec3_t = [0.0f32; 3];
        let mut ground: vec3_t = [0.0f32; 3];
        let mut d: f32;
        let mut dot: f32;

        unsafe {
            // ground[0] = backEnd.ori.axis[0][2];
            // ground[1] = backEnd.ori.axis[1][2];
            // ground[2] = backEnd.ori.axis[2][2];
            // d = backEnd.ori.origin[2] - backEnd.currentEntity->e.shadowPlane;

            // light[0] = backEnd.currentEntity->lightDir[0];
            // light[1] = backEnd.currentEntity->lightDir[1];
            // light[2] = backEnd.currentEntity->lightDir[2];

            dot = ground[0] * light[0] +
                  ground[1] * light[1] +
                  ground[2] * light[2];
            // don't let the shadows get too long or go negative
            if dot < 0.5 {
                // VectorMA( light, (0.5 - dot), ground, light );
                dot = light[0] * ground[0] + light[1] * ground[1] + light[2] * ground[2];
            }

            shadowMat[0][0] = dot - light[0] * ground[0];
            shadowMat[1][0] = 0.0f - light[0] * ground[1];
            shadowMat[2][0] = 0.0f - light[0] * ground[2];
            shadowMat[3][0] = 0.0f - light[0] * d;
            shadowMat[0][1] = 0.0f - light[1] * ground[0];
            shadowMat[1][1] = dot - light[1] * ground[1];
            shadowMat[2][1] = 0.0f - light[1] * ground[2];
            shadowMat[3][1] = 0.0f - light[1] * d;
            shadowMat[0][2] = 0.0f - light[2] * ground[0];
            shadowMat[1][2] = 0.0f - light[2] * ground[1];
            shadowMat[2][2] = dot - light[2] * ground[2];
            shadowMat[3][2] = 0.0f - light[2] * d;
            shadowMat[0][3] = 0.0f;
            shadowMat[1][3] = 0.0f;
            shadowMat[2][3] = 0.0f;
            shadowMat[3][3] = dot;

            // qglMatrixMode(GL_MODELVIEW);
            // qglMultMatrixf(&shadowMat[0][0]);

            // Turn on stenciling
            // This is done to prevent overlapping shadow artifacts
            // qglEnable( GL_STENCIL_TEST );
            // qglStencilFunc( GL_NOTEQUAL, 0x1, 0xffffffff );
            // qglStencilMask( 0xffffffff );
            // qglStencilOp( GL_KEEP, GL_KEEP, GL_INCR );
        }
    }

    #[cfg(not(target_os = "xbox"))]
    {
        let mut xyz: *mut f32;
        let mut i: c_int;
        let mut h: f32;
        let mut ground: vec3_t = [0.0f32; 3];
        let mut light: vec3_t = [0.0f32; 3];
        let mut groundDist: f32;
        let mut d: f32;
        let mut lightDir: vec3_t = [0.0f32; 3];

        unsafe {
            // xyz = ( float * ) tess.xyz;

            // ground[0] = backEnd.ori.axis[0][2];
            // ground[1] = backEnd.ori.axis[1][2];
            // ground[2] = backEnd.ori.axis[2][2];

            // groundDist = backEnd.ori.origin[2] - backEnd.currentEntity->e.shadowPlane;

            // VectorCopy( backEnd.currentEntity->lightDir, lightDir );
            d = lightDir[0] * ground[0] + lightDir[1] * ground[1] + lightDir[2] * ground[2];
            // don't let the shadows get too long or go negative
            if d < 0.5 {
                // VectorMA( lightDir, (0.5 - d), ground, lightDir );
                d = lightDir[0] * ground[0] + lightDir[1] * ground[1] + lightDir[2] * ground[2];
            }
            d = 1.0 / d;

            light[0] = lightDir[0] * d;
            light[1] = lightDir[1] * d;
            light[2] = lightDir[2] * d;

            // for ( i = 0; i < tess.numVertexes; i++, xyz += 4 ) {
            //     h = DotProduct( xyz, ground ) + groundDist;
            //
            //     xyz[0] -= light[0] * h;
            //     xyz[1] -= light[1] * h;
            //     xyz[2] -= light[2] * h;
            // }
        }
    }
}

// update tr.screenImage
fn RB_CaptureScreenImage() {
    let mut radX: c_int = 2048;
    let mut radY: c_int = 2048;
    let mut x: c_int;
    let mut y: c_int;
    let mut cX: c_int;
    let mut cY: c_int;

    unsafe {
        // x = glConfig.vidWidth/2;
        // y = glConfig.vidHeight/2;

        // GL_Bind( tr.screenImage );
        // using this method, we could pixel-filter the texture and all sorts of crazy stuff.
        // but, it is slow as hell.
        /*
        static byte *tmp = NULL;
        if (!tmp)
        {
            tmp = (byte *)Z_Malloc((sizeof(byte)*4)*(glConfig.vidWidth*glConfig.vidHeight), TAG_ICARUS, qtrue);
        }
        qglReadPixels(0, 0, glConfig.vidWidth, glConfig.vidHeight, GL_RGBA, GL_UNSIGNED_BYTE, tmp);
        qglTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, 512, 512, 0, GL_RGBA, GL_UNSIGNED_BYTE, tmp);
        */

        // if (radX > glConfig.maxTextureSize)
        // {
        //     radX = glConfig.maxTextureSize;
        // }
        // if (radY > glConfig.maxTextureSize)
        // {
        //     radY = glConfig.maxTextureSize;
        // }

        // while (glConfig.vidWidth < radX)
        // {
        //     radX /= 2;
        // }
        // while (glConfig.vidHeight < radY)
        // {
        //     radY /= 2;
        // }

        // cX = x-(radX/2);
        // cY = y-(radY/2);

        // if (cX+radX > glConfig.vidWidth)
        // { //would it go off screen?
        //     cX = glConfig.vidWidth-radX;
        // }
        // else if (cX < 0)
        // { //cap it off at 0
        //     cX = 0;
        // }

        // if (cY+radY > glConfig.vidHeight)
        // { //would it go off screen?
        //     cY = glConfig.vidHeight-radY;
        // }
        // else if (cY < 0)
        // { //cap it off at 0
        //     cY = 0;
        // }

        // #ifndef _XBOX
        // qglCopyTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA16, cX, cY, radX, radY, 0);
        // #else
        // qglCopyBackBufferToTexEXT(radX, radY, cX, (480 - cY), (cX + radX), (480 - (cY + radY)));
        // #endif // _XBOX
    }
}

// yeah.. not really shadow-related.. but it's stencil-related. -rww
static mut tr_distortionAlpha: f32 = 1.0f; // opaque
static mut tr_distortionStretch: f32 = 0.0f; // no stretch override
static mut tr_distortionPrePost: bool = false; // capture before postrender phase?
static mut tr_distortionNegate: bool = false; // negative blend mode

fn RB_DistortionFill() {
    unsafe {
        let mut alpha: f32 = tr_distortionAlpha;
        let mut spost: f32 = 0.0f;
        let mut spost2: f32 = 0.0f;

        // if ( glConfig.stencilBits < 4 )
        // {
        //     return;
        // }

        // ok, cap the stupid thing now I guess
        if !tr_distortionPrePost {
            RB_CaptureScreenImage();
        }

        // qglEnable(GL_STENCIL_TEST);
        // qglStencilFunc(GL_NOTEQUAL, 0, 0xFFFFFFFF);
        // qglStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);

        // qglDisable (GL_CLIP_PLANE0);
        // GL_Cull( CT_TWO_SIDED );

        // reset the view matrices and go into ortho mode
        // qglMatrixMode(GL_PROJECTION);
        // qglPushMatrix();
        // qglLoadIdentity();
        // qglOrtho(0, glConfig.vidWidth, glConfig.vidHeight, 32, -1, 1);
        // qglMatrixMode(GL_MODELVIEW);
        // qglPushMatrix();
        // qglLoadIdentity();

        if tr_distortionStretch != 0.0f {
            // override
            spost = tr_distortionStretch;
            spost2 = tr_distortionStretch;
        } else {
            // do slow stretchy effect
            // spost = sin(tr.refdef.time*0.0005f);
            if spost < 0.0f {
                spost = -spost;
            }
            spost *= 0.2f;

            // spost2 = sin(tr.refdef.time*0.0005f);
            if spost2 < 0.0f {
                spost2 = -spost2;
            }
            spost2 *= 0.08f;
        }

        if alpha != 1.0f {
            // blend
            // GL_State(GLS_SRCBLEND_SRC_ALPHA|GLS_DSTBLEND_SRC_ALPHA);
        } else {
            // be sure to reset the draw state
            // GL_State(0);
        }

        // #ifdef _XBOX
        // qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
        // #else
        // qglBegin(GL_QUADS);
        // #endif // _XBOX
        qglColor4f(1.0f, 1.0f, 1.0f, alpha);
        qglTexCoord2f(0.0f + spost2, 1.0f - spost);
        qglVertex2f(0.0f, 0.0f);

        qglTexCoord2f(0.0f + spost2, 0.0f + spost);
        qglVertex2f(0.0f, 0.0f); // glConfig.vidHeight

        qglTexCoord2f(1.0f - spost2, 0.0f + spost);
        qglVertex2f(0.0f, 0.0f); // glConfig.vidWidth, glConfig.vidHeight

        qglTexCoord2f(1.0f - spost2, 1.0f - spost);
        qglVertex2f(0.0f, 0.0f); // glConfig.vidWidth, 0
        // qglEnd();

        if tr_distortionAlpha == 1.0f && tr_distortionStretch == 0.0f {
            // no overrides
            if tr_distortionNegate {
                // probably the crazy alternate saber trail
                alpha = 0.8f;
                // GL_State(GLS_SRCBLEND_ZERO|GLS_DSTBLEND_ONE_MINUS_SRC_COLOR);
            } else {
                alpha = 0.5f;
                // GL_State(GLS_SRCBLEND_SRC_ALPHA|GLS_DSTBLEND_SRC_ALPHA);
            }

            // spost = sin(tr.refdef.time*0.0008f);
            if spost < 0.0f {
                spost = -spost;
            }
            spost *= 0.08f;

            // spost2 = sin(tr.refdef.time*0.0008f);
            if spost2 < 0.0f {
                spost2 = -spost2;
            }
            spost2 *= 0.2f;

            // #ifdef _XBOX
            // qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
            // #else
            // qglBegin(GL_QUADS);
            // #endif // _XBOX
            qglColor4f(1.0f, 1.0f, 1.0f, alpha);
            qglTexCoord2f(0.0f + spost2, 1.0f - spost);
            qglVertex2f(0.0f, 0.0f);

            qglTexCoord2f(0.0f + spost2, 0.0f + spost);
            qglVertex2f(0.0f, 0.0f); // glConfig.vidHeight

            qglTexCoord2f(1.0f - spost2, 0.0f + spost);
            qglVertex2f(0.0f, 0.0f); // glConfig.vidWidth, glConfig.vidHeight

            qglTexCoord2f(1.0f - spost2, 1.0f - spost);
            qglVertex2f(0.0f, 0.0f); // glConfig.vidWidth, 0
            // qglEnd();
        }

        // pop the view matrices back
        // qglMatrixMode(GL_PROJECTION);
        // qglPopMatrix();
        // qglMatrixMode(GL_MODELVIEW);
        // qglPopMatrix();

        // qglDisable( GL_STENCIL_TEST );
    }
}
