// leave this as first line for PCH reasons...
//

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals,
         unused_imports, dead_code, unused_mut, unused_variables,
         unused_assignments, clippy::all)]

use crate::code::server::exe_headers_h::*;
use crate::code::renderer::tr_local_h::*;
#[cfg(feature = "VV_LIGHTING")]
use crate::code::renderer::tr_lightmanager_h::*;
#[cfg(feature = "_XBOX")]
use crate::code::win32::win_highdynamicrange_h::*;

use core::ffi::{c_char, c_double, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};
use core::ptr::{addr_of, addr_of_mut};

unsafe extern "C" {
    pub static mut tr_distortionPrePost: qboolean; //tr_shadows.cpp
    pub static mut tr_distortionNegate: qboolean; //tr_shadows.cpp
    pub fn RB_CaptureScreenImage(); //tr_shadows.cpp
    pub fn RB_DistortionFill(); //tr_shadows.cpp
    pub fn RB_RenderWorldEffects();
}

#[cfg(not(feature = "_XBOX"))]
unsafe extern "C" {
    static mut g_bTextureRectangleHack: bool;
}

pub static mut backEndData: *mut backEndData_t = core::ptr::null_mut();
pub static mut backEnd: backEndState_t = unsafe { core::mem::zeroed() };
pub static mut tr_stencilled: bool = false;

// Whether we are currently rendering only glowing objects or not.
pub static mut g_bRenderGlowingObjects: bool = false;

// Whether the current hardware supports dynamic glows/flares.
pub static mut g_bDynamicGlowSupported: bool = false;

#[cfg(not(feature = "_XBOX"))]
static s_flipMatrix: [f32; 16] = [
    // convert from our coordinate system (looking down X)
    // to OpenGL's coordinate system (looking down -Z)
    0.0, 0.0, -1.0, 0.0,
    -1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

#[cfg(feature = "_XBOX")]
static s_flipMatrix: [f32; 16] = [
    // convert from our coordinate system (looking down X)
    // to OpenGL's coordinate system (looking down -Z)
    0.0, 0.0, 1.0, 0.0,
    -1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

// static void RB_DrawGlowOverlay();  -- forward decls not needed in Rust
// static void RB_BlurGlowTexture();

const MAC_EVENT_PUMP_MSEC: c_int = 5;

//number of possible surfs we can postrender.
//note that postrenders lack much of the optimization that the standard sort-render crap does,
//so it's slower.
const MAX_POST_RENDERS: usize = 128;

#[repr(C)]
struct postRender_t {
    fogNum: c_int,
    entNum: c_int,
    dlighted: c_int,
    depthRange: c_int,
    drawSurf: *mut drawSurf_t,
    shader: *mut shader_t,
}

static mut g_postRenders: [postRender_t; MAX_POST_RENDERS] =
    unsafe { core::mem::zeroed() };
static mut g_numPostRenders: c_int = 0;


/*
** GL_Bind
*/
pub unsafe fn GL_Bind(image: *mut image_t) {
    let mut texnum: c_int;

    if image.is_null() {
        VID_Printf(PRINT_WARNING, b"GL_Bind: NULL image\n\0".as_ptr() as *const c_char);
        texnum = (*tr.defaultImage).texnum;
    } else {
        texnum = (*image).texnum;
    }

    #[cfg(not(feature = "_XBOX"))]
    if (*r_nobind).integer != 0 && !tr.dlightImage.is_null() { // performance evaluation option
        texnum = (*tr.dlightImage).texnum;
    }

    if glState.currenttextures[glState.currenttmu as usize] != texnum {
        #[cfg(not(feature = "_XBOX"))]
        {
            (*image).frameUsed = tr.frameCount;
        }
        glState.currenttextures[glState.currenttmu as usize] = texnum;
        qglBindTexture(GL_TEXTURE_2D, texnum);
    }
}

/*
** GL_SelectTexture
*/
pub unsafe fn GL_SelectTexture(unit: c_int) {
    if glState.currenttmu == unit {
        return;
    }

    if unit == 0 {
        qglActiveTextureARB(GL_TEXTURE0_ARB);
        GLimp_LogComment(b"glActiveTextureARB( GL_TEXTURE0_ARB )\n\0".as_ptr() as *const c_char);
        qglClientActiveTextureARB(GL_TEXTURE0_ARB);
        GLimp_LogComment(b"glClientActiveTextureARB( GL_TEXTURE0_ARB )\n\0".as_ptr() as *const c_char);
    } else if unit == 1 {
        qglActiveTextureARB(GL_TEXTURE1_ARB);
        GLimp_LogComment(b"glActiveTextureARB( GL_TEXTURE1_ARB )\n\0".as_ptr() as *const c_char);
        qglClientActiveTextureARB(GL_TEXTURE1_ARB);
        GLimp_LogComment(b"glClientActiveTextureARB( GL_TEXTURE1_ARB )\n\0".as_ptr() as *const c_char);
    } else if unit == 2 {
        qglActiveTextureARB(GL_TEXTURE2_ARB);
        GLimp_LogComment(b"glActiveTextureARB( GL_TEXTURE2_ARB )\n\0".as_ptr() as *const c_char);
        qglClientActiveTextureARB(GL_TEXTURE2_ARB);
        GLimp_LogComment(b"glClientActiveTextureARB( GL_TEXTURE2_ARB )\n\0".as_ptr() as *const c_char);
    } else if unit == 3 {
        qglActiveTextureARB(GL_TEXTURE3_ARB);
        GLimp_LogComment(b"glActiveTextureARB( GL_TEXTURE3_ARB )\n\0".as_ptr() as *const c_char);
        qglClientActiveTextureARB(GL_TEXTURE3_ARB);
        GLimp_LogComment(b"glClientActiveTextureARB( GL_TEXTURE3_ARB )\n\0".as_ptr() as *const c_char);
    } else {
        Com_Error(ERR_DROP, b"GL_SelectTexture: unit = %i\0".as_ptr() as *const c_char, unit);
    }

    glState.currenttmu = unit;
}


/*
** GL_Cull
*/
pub unsafe fn GL_Cull(cullType: c_int) {
    if glState.faceCulling == cullType {
        return;
    }
    glState.faceCulling = cullType;
    if backEnd.projection2D != 0 { //don't care, we're in 2d when it's always disabled
        return;
    }

    if cullType == CT_TWO_SIDED {
        qglDisable(GL_CULL_FACE);
    } else {
        qglEnable(GL_CULL_FACE);

        if cullType == CT_BACK_SIDED {
            if backEnd.viewParms.isMirror != 0 {
                qglCullFace(GL_FRONT);
            } else {
                qglCullFace(GL_BACK);
            }
        } else {
            if backEnd.viewParms.isMirror != 0 {
                qglCullFace(GL_BACK);
            } else {
                qglCullFace(GL_FRONT);
            }
        }
    }
}

/*
** GL_TexEnv
*/
pub unsafe fn GL_TexEnv(env: c_int) {
    if env == glState.texEnv[glState.currenttmu as usize] {
        return;
    }

    glState.texEnv[glState.currenttmu as usize] = env;

    // Note (porting): #[cfg] cannot gate else-if arms; the _XBOX GL_NONE arm is
    // handled by duplicating the chain under cfg, per porting guidelines.
    #[cfg(not(feature = "_XBOX"))]
    {
        if env == GL_MODULATE as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_MODULATE as f32);
        } else if env == GL_REPLACE as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_REPLACE as f32);
        } else if env == GL_DECAL as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_DECAL as f32);
        } else if env == GL_ADD as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_ADD as f32);
        } else {
            Com_Error(ERR_DROP, b"GL_TexEnv: invalid env '%d' passed\n\0".as_ptr() as *const c_char, env);
        }
    }
    #[cfg(feature = "_XBOX")]
    {
        if env == GL_MODULATE as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_MODULATE as f32);
        } else if env == GL_REPLACE as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_REPLACE as f32);
        } else if env == GL_DECAL as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_DECAL as f32);
        } else if env == GL_ADD as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_ADD as f32);
        } else if env == GL_NONE as c_int {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_NONE as f32);
        } else {
            Com_Error(ERR_DROP, b"GL_TexEnv: invalid env '%d' passed\n\0".as_ptr() as *const c_char, env);
        }
    }
}

/*
** GL_State
**
** This routine is responsible for setting the most commonly changed state
** in Q3.
*/
pub unsafe fn GL_State(stateBits: c_ulong) {
    let diff: c_ulong = stateBits ^ glState.glStateBits;

    if diff == 0 {
        return;
    }

    //
    // check depthFunc bits
    //
    if diff & GLS_DEPTHFUNC_EQUAL as c_ulong != 0 {
        if stateBits & GLS_DEPTHFUNC_EQUAL as c_ulong != 0 {
            qglDepthFunc(GL_EQUAL);
        } else {
            qglDepthFunc(GL_LEQUAL);
        }
    }

    //
    // check blend bits
    //
    if diff & (GLS_SRCBLEND_BITS as c_ulong | GLS_DSTBLEND_BITS as c_ulong) != 0 {
        let mut srcFactor: GLenum;
        let mut dstFactor: GLenum;

        if stateBits & (GLS_SRCBLEND_BITS as c_ulong | GLS_DSTBLEND_BITS as c_ulong) != 0 {
            let src_bits = stateBits & GLS_SRCBLEND_BITS as c_ulong;
            if src_bits == GLS_SRCBLEND_ZERO as c_ulong {
                srcFactor = GL_ZERO;
            } else if src_bits == GLS_SRCBLEND_ONE as c_ulong {
                srcFactor = GL_ONE;
            } else if src_bits == GLS_SRCBLEND_DST_COLOR as c_ulong {
                srcFactor = GL_DST_COLOR;
            } else if src_bits == GLS_SRCBLEND_ONE_MINUS_DST_COLOR as c_ulong {
                srcFactor = GL_ONE_MINUS_DST_COLOR;
            } else if src_bits == GLS_SRCBLEND_SRC_ALPHA as c_ulong {
                srcFactor = GL_SRC_ALPHA;
            } else if src_bits == GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA as c_ulong {
                srcFactor = GL_ONE_MINUS_SRC_ALPHA;
            } else if src_bits == GLS_SRCBLEND_DST_ALPHA as c_ulong {
                srcFactor = GL_DST_ALPHA;
            } else if src_bits == GLS_SRCBLEND_ONE_MINUS_DST_ALPHA as c_ulong {
                srcFactor = GL_ONE_MINUS_DST_ALPHA;
            } else if src_bits == GLS_SRCBLEND_ALPHA_SATURATE as c_ulong {
                srcFactor = GL_SRC_ALPHA_SATURATE;
            } else {
                srcFactor = GL_ONE; // to get warning to shut up
                Com_Error(ERR_DROP, b"GL_State: invalid src blend state bits\n\0".as_ptr() as *const c_char);
            }

            let dst_bits = stateBits & GLS_DSTBLEND_BITS as c_ulong;
            if dst_bits == GLS_DSTBLEND_ZERO as c_ulong {
                dstFactor = GL_ZERO;
            } else if dst_bits == GLS_DSTBLEND_ONE as c_ulong {
                dstFactor = GL_ONE;
            } else if dst_bits == GLS_DSTBLEND_SRC_COLOR as c_ulong {
                dstFactor = GL_SRC_COLOR;
            } else if dst_bits == GLS_DSTBLEND_ONE_MINUS_SRC_COLOR as c_ulong {
                dstFactor = GL_ONE_MINUS_SRC_COLOR;
            } else if dst_bits == GLS_DSTBLEND_SRC_ALPHA as c_ulong {
                dstFactor = GL_SRC_ALPHA;
            } else if dst_bits == GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA as c_ulong {
                dstFactor = GL_ONE_MINUS_SRC_ALPHA;
            } else if dst_bits == GLS_DSTBLEND_DST_ALPHA as c_ulong {
                dstFactor = GL_DST_ALPHA;
            } else if dst_bits == GLS_DSTBLEND_ONE_MINUS_DST_ALPHA as c_ulong {
                dstFactor = GL_ONE_MINUS_DST_ALPHA;
            } else {
                dstFactor = GL_ONE; // to get warning to shut up
                Com_Error(ERR_DROP, b"GL_State: invalid dst blend state bits\n\0".as_ptr() as *const c_char);
            }

            qglEnable(GL_BLEND);
            qglBlendFunc(srcFactor, dstFactor);
        } else {
            qglDisable(GL_BLEND);
        }
    }

    //
    // check depthmask
    //
    if diff & GLS_DEPTHMASK_TRUE as c_ulong != 0 {
        if stateBits & GLS_DEPTHMASK_TRUE as c_ulong != 0 {
            qglDepthMask(GL_TRUE);
        } else {
            qglDepthMask(GL_FALSE);
        }
    }

    //
    // fill/line mode
    //
    if diff & GLS_POLYMODE_LINE as c_ulong != 0 {
        if stateBits & GLS_POLYMODE_LINE as c_ulong != 0 {
            qglPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
        } else {
            qglPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
        }
    }

    //
    // depthtest
    //
    if diff & GLS_DEPTHTEST_DISABLE as c_ulong != 0 {
        if stateBits & GLS_DEPTHTEST_DISABLE as c_ulong != 0 {
            qglDisable(GL_DEPTH_TEST);
        } else {
            qglEnable(GL_DEPTH_TEST);
        }
    }

    //
    // alpha test
    //
    if diff & GLS_ATEST_BITS as c_ulong != 0 {
        let atest_bits = stateBits & GLS_ATEST_BITS as c_ulong;
        if atest_bits == 0 {
            qglDisable(GL_ALPHA_TEST);
        } else if atest_bits == GLS_ATEST_GT_0 as c_ulong {
            qglEnable(GL_ALPHA_TEST);
            qglAlphaFunc(GL_GREATER, 0.0f32);
        } else if atest_bits == GLS_ATEST_LT_80 as c_ulong {
            qglEnable(GL_ALPHA_TEST);
            qglAlphaFunc(GL_LESS, 0.5f32);
        } else if atest_bits == GLS_ATEST_GE_80 as c_ulong {
            qglEnable(GL_ALPHA_TEST);
            qglAlphaFunc(GL_GEQUAL, 0.5f32);
        } else if atest_bits == GLS_ATEST_GE_C0 as c_ulong {
            qglEnable(GL_ALPHA_TEST);
            qglAlphaFunc(GL_GEQUAL, 0.75f32);
        } else {
            assert!(false);
        }
    }

    glState.glStateBits = stateBits;
}


/*
================
RB_Hyperspace

A player has predicted a teleport, but hasn't arrived yet
================
*/
unsafe fn RB_Hyperspace() {
    let c: f32;

    if backEnd.isHyperspace == 0 {
        // do initialization shit
    }

    c = (backEnd.refdef.time & 255) as f32 / 255.0f32;
    qglClearColor(c, c, c, 1.0f32);
    qglClear(GL_COLOR_BUFFER_BIT);

    backEnd.isHyperspace = qtrue;
}


pub unsafe fn SetViewportAndScissor() {
    qglMatrixMode(GL_PROJECTION);
    qglLoadMatrixf(backEnd.viewParms.projectionMatrix.as_ptr());
    qglMatrixMode(GL_MODELVIEW);

    // set the window clipping
    qglViewport(backEnd.viewParms.viewportX, backEnd.viewParms.viewportY,
        backEnd.viewParms.viewportWidth, backEnd.viewParms.viewportHeight);
    qglScissor(backEnd.viewParms.viewportX, backEnd.viewParms.viewportY,
        backEnd.viewParms.viewportWidth, backEnd.viewParms.viewportHeight);
}

/*
=================
RB_BeginDrawingView

Any mirrored or portaled views have already been drawn, so prepare
to actually render the visible surfaces for this view
=================
*/
unsafe fn RB_BeginDrawingView() {
    let mut clearBits: c_int = GL_DEPTH_BUFFER_BIT as c_int;

    // sync with gl if needed
    if (*r_finish).integer == 1 && glState.finishCalled == 0 {
        qglFinish();
        glState.finishCalled = qtrue;
    }
    if (*r_finish).integer == 0 {
        glState.finishCalled = qtrue;
    }

    // we will need to change the projection matrix before drawing
    // 2D images again
    backEnd.projection2D = qfalse;

    //
    // set the modelview matrix for the viewer
    //
    SetViewportAndScissor();

    // ensures that depth writes are enabled for the depth clear
    GL_State(GLS_DEFAULT as c_ulong);

    // clear relevant buffers
    if (*r_measureOverdraw).integer != 0 || (*r_shadows).integer == 2 || tr_stencilled {
        clearBits |= GL_STENCIL_BUFFER_BIT as c_int;
        tr_stencilled = false;
    }

    if skyboxportal != 0 {
        if backEnd.refdef.rdflags & RDF_SKYBOXPORTAL != 0 {
            // portal scene, clear whatever is necessary
            if (*r_fastsky).integer != 0 || (backEnd.refdef.rdflags & RDF_NOWORLDMODEL != 0) {
                // fastsky: clear color
                // try clearing first with the portal sky fog color, then the world fog color, then finally a default
                clearBits |= GL_COLOR_BUFFER_BIT as c_int;
                if !tr.world.is_null() && (*tr.world).globalFog != -1 {
                    let fog: *const fog_t = &(*tr.world).fogs[(*tr.world).globalFog as usize];
                    qglClearColor((*fog).parms.color[0], (*fog).parms.color[1], (*fog).parms.color[2], 1.0f32);
                } else {
                    qglClearColor(0.3f32, 0.3f32, 0.3f32, 1.0f32);
                }
            }
        }
    } else {
        if (*r_fastsky).integer != 0 && (backEnd.refdef.rdflags & RDF_NOWORLDMODEL == 0) && !g_bRenderGlowingObjects {
            if !tr.world.is_null() && (*tr.world).globalFog != -1 {
                let fog: *const fog_t = &(*tr.world).fogs[(*tr.world).globalFog as usize];
                qglClearColor((*fog).parms.color[0], (*fog).parms.color[1], (*fog).parms.color[2], 1.0f32);
            } else {
                qglClearColor(0.3f32, 0.3f32, 0.3f32, 1.0f32); // FIXME: get color of sky
            }
            clearBits |= GL_COLOR_BUFFER_BIT as c_int; // FIXME: only if sky shaders have been used
        }
    }

    if (backEnd.refdef.rdflags & RDF_NOWORLDMODEL == 0) && ((*r_DynamicGlow).integer != 0 && !g_bRenderGlowingObjects) {
        if !tr.world.is_null() && (*tr.world).globalFog != -1 {
            //this is because of a bug in multiple scenes I think, it needs to clear for the second scene but it doesn't normally.
            let fog: *const fog_t = &(*tr.world).fogs[(*tr.world).globalFog as usize];

            qglClearColor((*fog).parms.color[0], (*fog).parms.color[1], (*fog).parms.color[2], 1.0f32);
            clearBits |= GL_COLOR_BUFFER_BIT as c_int;
        }
    }
    // If this pass is to just render the glowing objects, don't clear the depth buffer since
    // we're sharing it with the main scene (since the main scene has already been rendered). -AReis
    if g_bRenderGlowingObjects {
        clearBits &= !(GL_DEPTH_BUFFER_BIT as c_int);
    }

    if clearBits != 0 {
        qglClear(clearBits);
    }

    if backEnd.refdef.rdflags & RDF_HYPERSPACE != 0 {
        RB_Hyperspace();
        return;
    } else {
        backEnd.isHyperspace = qfalse;
    }

    glState.faceCulling = -1; // force face culling to set next time

    // we will only draw a sun if there was sky rendered in this view
    backEnd.skyRenderedThisView = qfalse;

    // clip to the plane of the portal
    if backEnd.viewParms.isPortal != 0 {
        let mut plane: [f32; 4] = [0.0f32; 4];
        let mut plane2: [f64; 4] = [0.0f64; 4];

        plane[0] = backEnd.viewParms.portalPlane.normal[0];
        plane[1] = backEnd.viewParms.portalPlane.normal[1];
        plane[2] = backEnd.viewParms.portalPlane.normal[2];
        plane[3] = backEnd.viewParms.portalPlane.dist;

        plane2[0] = DotProduct(backEnd.viewParms.or.axis[0].as_ptr(), plane.as_ptr()) as f64;
        plane2[1] = DotProduct(backEnd.viewParms.or.axis[1].as_ptr(), plane.as_ptr()) as f64;
        plane2[2] = DotProduct(backEnd.viewParms.or.axis[2].as_ptr(), plane.as_ptr()) as f64;
        plane2[3] = DotProduct(plane.as_ptr(), backEnd.viewParms.or.origin.as_ptr()) as f64 - plane[3] as f64;

        qglLoadMatrixf(s_flipMatrix.as_ptr());
        qglClipPlane(GL_CLIP_PLANE0, plane2.as_ptr());
        qglEnable(GL_CLIP_PLANE0);
    } else {
        qglDisable(GL_CLIP_PLANE0);
    }
}

// #define MAC_EVENT_PUMP_MSEC  5  (already defined as const above)

//used by RF_DISTORTION
#[inline]
unsafe fn R_WorldCoordToScreenCoordFloat(worldCoord: *mut f32, x: *mut f32, y: *mut f32) -> bool {
    let xcenter: c_int;
    let ycenter: c_int;
    let mut local: [f32; 3] = [0.0f32; 3];
    let mut transformed: [f32; 3] = [0.0f32; 3];
    let mut vfwd: [f32; 3] = [0.0f32; 3];
    let mut vright: [f32; 3] = [0.0f32; 3];
    let mut vup: [f32; 3] = [0.0f32; 3];
    let xzi: f32;
    let yzi: f32;

    xcenter = glConfig.vidWidth / 2;
    ycenter = glConfig.vidHeight / 2;

    //AngleVectors (tr.refdef.viewangles, vfwd, vright, vup);
    VectorCopy(tr.refdef.viewaxis[0].as_ptr(), vfwd.as_mut_ptr());
    VectorCopy(tr.refdef.viewaxis[1].as_ptr(), vright.as_mut_ptr());
    VectorCopy(tr.refdef.viewaxis[2].as_ptr(), vup.as_mut_ptr());

    VectorSubtract(worldCoord, tr.refdef.vieworg.as_ptr(), local.as_mut_ptr());

    transformed[0] = DotProduct(local.as_ptr(), vright.as_ptr());
    transformed[1] = DotProduct(local.as_ptr(), vup.as_ptr());
    transformed[2] = DotProduct(local.as_ptr(), vfwd.as_ptr());

    // Make sure Z is not negative.
    if transformed[2] < 0.01f32 {
        return false;
    }

    xzi = xcenter as f32 / transformed[2] * (90.0f32 / tr.refdef.fov_x);
    yzi = ycenter as f32 / transformed[2] * (90.0f32 / tr.refdef.fov_y);

    *x = xcenter as f32 + xzi * transformed[0];
    *y = ycenter as f32 - yzi * transformed[1];

    true
}

//used by RF_DISTORTION
#[inline]
unsafe fn R_WorldCoordToScreenCoord(worldCoord: *mut f32, x: *mut c_int, y: *mut c_int) -> bool {
    let mut xF: f32 = 0.0f32;
    let mut yF: f32 = 0.0f32;
    let retVal: bool = R_WorldCoordToScreenCoordFloat(worldCoord, &mut xF, &mut yF);
    *x = xF as c_int;
    *y = yF as c_int;
    retVal
}

/*
==================
RB_RenderDrawSurfList
==================
*/
pub unsafe fn RB_RenderDrawSurfList(drawSurfs: *mut drawSurf_t, numDrawSurfs: c_int) {
    let mut shader: *mut shader_t;
    let mut oldShader: *mut shader_t;
    let mut fogNum: c_int;
    let mut oldFogNum: c_int;
    let mut entityNum: c_int;
    let mut oldEntityNum: c_int;
    let mut dlighted: c_int;
    let mut oldDlighted: c_int;
    let mut depthRange: c_int;
    let mut oldDepthRange: c_int;
    let mut i: c_int;
    let mut drawSurf: *mut drawSurf_t;
    let mut oldSort: c_uint;
    let mut originalTime: f32;
    let mut curEnt: *mut trRefEntity_t;
    let mut pRender: *mut postRender_t;
    let mut didShadowPass: bool = false;
    #[cfg(feature = "__MACOS__")]
    let mut macEventTime: c_int;

    #[cfg(feature = "__MACOS__")]
    {
        Sys_PumpEvents(); // crutch up the mac's limited buffer queue size

        // we don't want to pump the event loop too often and waste time, so
        // we are going to check every shader change
        macEventTime = Sys_Milliseconds() + MAC_EVENT_PUMP_MSEC;
    }

    if g_bRenderGlowingObjects {
        //only shadow on initial passes
        didShadowPass = true;
    }

    // save original time for entity shader offsets
    originalTime = backEnd.refdef.floatTime;

    // clear the z buffer, set the modelview, etc
    RB_BeginDrawingView();

    // draw everything
    oldEntityNum = -1;
    backEnd.currentEntity = &mut tr.worldEntity;
    oldShader = core::ptr::null_mut();
    oldFogNum = -1;
    oldDepthRange = qfalse;
    oldDlighted = qfalse;
    oldSort = c_uint::MAX;
    depthRange = qfalse;

    backEnd.pc.c_surfaces += numDrawSurfs;

    i = 0;
    drawSurf = drawSurfs;
    while i < numDrawSurfs {
        if (*drawSurf).sort == oldSort {
            // fast path, same as previous sort
            (rb_surfaceTable[*(*drawSurf).surface as usize])((*drawSurf).surface as *mut c_void);
            i += 1;
            drawSurf = drawSurf.add(1);
            continue;
        }
        R_DecomposeSort((*drawSurf).sort, addr_of_mut!(entityNum), addr_of_mut!(shader), addr_of_mut!(fogNum), addr_of_mut!(dlighted));

        #[cfg(not(feature = "_XBOX"))] // GLOWXXX
        // If we're rendering glowing objects, but this shader has no stages with glow, skip it!
        if g_bRenderGlowingObjects && (*shader).hasGlow == 0 {
            shader = oldShader;
            entityNum = oldEntityNum;
            fogNum = oldFogNum;
            dlighted = oldDlighted;
            i += 1;
            drawSurf = drawSurf.add(1);
            continue;
        }
        oldSort = (*drawSurf).sort;

        //
        // change the tess parameters if needed
        // a "entityMergable" shader is a shader that can have surfaces from seperate
        // entities merged into a single batch, like smoke and blood puff sprites
        if entityNum != TR_WORLDENT && g_numPostRenders < MAX_POST_RENDERS as c_int {
            if ((*backEnd.refdef.entities.add(entityNum as usize)).e.renderfx & RF_DISTORTION != 0)
                /* || ((*backEnd.refdef.entities.add(entityNum as usize)).e.renderfx & RF_FORCE_ENT_ALPHA != 0) */
                //not sure if we need this alpha fix for sp or not, leaving it out for now -rww
            {
                //must render last
                curEnt = backEnd.refdef.entities.add(entityNum as usize) as *mut trRefEntity_t;
                pRender = addr_of_mut!(g_postRenders[g_numPostRenders as usize]);

                g_numPostRenders += 1;

                depthRange = 0;
                //figure this stuff out now and store it
                if (*curEnt).e.renderfx & RF_NODEPTH != 0 {
                    depthRange = 2;
                } else if (*curEnt).e.renderfx & RF_DEPTHHACK != 0 {
                    depthRange = 1;
                }
                (*pRender).depthRange = depthRange;

                //It is not necessary to update the old* values because
                //we are not updating now with the current values.
                depthRange = oldDepthRange;

                //store off the ent num
                (*pRender).entNum = entityNum;

                //remember the other values necessary for rendering this surf
                (*pRender).drawSurf = drawSurf;
                (*pRender).dlighted = dlighted;
                (*pRender).fogNum = fogNum;
                (*pRender).shader = shader;

                //assure the info is back to the last set state
                shader = oldShader;
                entityNum = oldEntityNum;
                fogNum = oldFogNum;
                dlighted = oldDlighted;

                oldSort = c_uint::MAX; //invalidate this thing, cause we may want to postrender more surfs of the same sort

                //continue without bothering to begin a draw surf
                i += 1;
                drawSurf = drawSurf.add(1);
                continue;
            }
        }

        if shader != oldShader || fogNum != oldFogNum || dlighted != oldDlighted
            || (entityNum != oldEntityNum && (*shader).entityMergable == 0)
        {
            if !oldShader.is_null() {
                #[cfg(feature = "__MACOS__")] // crutch up the mac's limited buffer queue size
                {
                    let t: c_int = Sys_Milliseconds();
                    if t > macEventTime {
                        macEventTime = t + MAC_EVENT_PUMP_MSEC;
                        Sys_PumpEvents();
                    }
                }
                RB_EndSurface();

                if !didShadowPass && !shader.is_null() && (*shader).sort > SS_BANNER as f32 {
                    RB_ShadowFinish();
                    didShadowPass = true;
                }
            }
            RB_BeginSurface(shader, fogNum);
            oldShader = shader;
            oldFogNum = fogNum;
            oldDlighted = dlighted;
        }

        //
        // change the modelview matrix if needed
        //
        if entityNum != oldEntityNum {
            depthRange = qfalse;

            if entityNum != TR_WORLDENT {
                backEnd.currentEntity = backEnd.refdef.entities.add(entityNum as usize) as *mut trRefEntity_t;
                backEnd.refdef.floatTime = originalTime - (*backEnd.currentEntity).e.shaderTime;

                // set up the transformation matrix
                R_RotateForEntity(backEnd.currentEntity, &mut backEnd.viewParms, &mut backEnd.ori);

                // set up the dynamic lighting if needed
                if (*backEnd.currentEntity).needDlights != 0 {
                    #[cfg(feature = "VV_LIGHTING")]
                    VVLightMan.R_TransformDlights(addr_of_mut!(backEnd.ori));
                    #[cfg(not(feature = "VV_LIGHTING"))]
                    R_TransformDlights(backEnd.refdef.num_dlights, backEnd.refdef.dlights, addr_of_mut!(backEnd.ori));
                }

                if (*backEnd.currentEntity).e.renderfx & RF_NODEPTH != 0 {
                    // No depth at all, very rare but some things for seeing through walls
                    depthRange = 2;
                } else if (*backEnd.currentEntity).e.renderfx & RF_DEPTHHACK != 0 {
                    // hack the depth range to prevent view model from poking into walls
                    depthRange = qtrue;
                }
            } else {
                backEnd.currentEntity = &mut tr.worldEntity;
                backEnd.refdef.floatTime = originalTime;
                backEnd.ori = backEnd.viewParms.world;
                #[cfg(feature = "VV_LIGHTING")]
                VVLightMan.R_TransformDlights(addr_of_mut!(backEnd.ori));
                #[cfg(not(feature = "VV_LIGHTING"))]
                R_TransformDlights(backEnd.refdef.num_dlights, backEnd.refdef.dlights, addr_of_mut!(backEnd.ori));
            }

            qglLoadMatrixf(backEnd.ori.modelMatrix.as_ptr());

            //
            // change depthrange if needed
            //
            if oldDepthRange != depthRange {
                if depthRange == 0 {
                    qglDepthRange(0.0f64, 1.0f64);
                } else if depthRange == 1 {
                    qglDepthRange(0.0f64, 0.3f64);
                } else if depthRange == 2 {
                    qglDepthRange(0.0f64, 0.0f64);
                } else {
                    qglDepthRange(0.0f64, 1.0f64);
                }

                oldDepthRange = depthRange;
            }

            oldEntityNum = entityNum;
        }

        // add the triangles for this surface
        (rb_surfaceTable[*(*drawSurf).surface as usize])((*drawSurf).surface as *mut c_void);
        i += 1;
        drawSurf = drawSurf.add(1);
    }

    // draw the contents of the last shader batch
    if !oldShader.is_null() {
        RB_EndSurface();
    }

    if tr_stencilled && tr_distortionPrePost != 0 {
        //ok, cap it now
        RB_CaptureScreenImage();
        RB_DistortionFill();
    }

    //render distortion surfs (or anything else that needs to be post-rendered)
    if g_numPostRenders > 0 {
        let mut lastPostEnt: c_int = -1;

        while g_numPostRenders > 0 {
            g_numPostRenders -= 1;
            pRender = addr_of_mut!(g_postRenders[g_numPostRenders as usize]);

            RB_BeginSurface((*pRender).shader, (*pRender).fogNum);

            backEnd.currentEntity = backEnd.refdef.entities.add((*pRender).entNum as usize) as *mut trRefEntity_t;

            backEnd.refdef.floatTime = originalTime - (*backEnd.currentEntity).e.shaderTime;

            // set up the transformation matrix
            R_RotateForEntity(backEnd.currentEntity, &mut backEnd.viewParms, &mut backEnd.ori);

            // set up the dynamic lighting if needed
            if (*backEnd.currentEntity).needDlights != 0 {
                #[cfg(feature = "VV_LIGHTING")]
                VVLightMan.R_TransformDlights(addr_of_mut!(backEnd.ori));
                #[cfg(not(feature = "VV_LIGHTING"))]
                R_TransformDlights(backEnd.refdef.num_dlights, backEnd.refdef.dlights, addr_of_mut!(backEnd.ori));
            }

            qglLoadMatrixf(backEnd.ori.modelMatrix.as_ptr());

            depthRange = (*pRender).depthRange;
            if depthRange == 1 {
                qglDepthRange(0.0f64, 0.3f64);
            } else if depthRange == 2 {
                qglDepthRange(0.0f64, 0.0f64);
            } else {
                qglDepthRange(0.0f64, 1.0f64);
            }

            if ((*backEnd.currentEntity).e.renderfx & RF_DISTORTION != 0) && lastPostEnt != (*pRender).entNum {
                //do the capture now, we only need to do it once per ent
                let mut x: c_int = 0;
                let mut y: c_int = 0;
                let rad: c_int = (*backEnd.currentEntity).e.radius as c_int;
                //We are going to just bind this, and then the CopyTexImage is going to
                //stomp over this texture num in texture memory.
                GL_Bind(tr.screenImage);

                if R_WorldCoordToScreenCoord((*backEnd.currentEntity).e.origin.as_mut_ptr(), &mut x, &mut y) {
                    let mut cX: c_int = glConfig.vidWidth - x - (rad / 2);
                    let mut cY: c_int = glConfig.vidHeight - y - (rad / 2);

                    if cX + rad > glConfig.vidWidth {
                        //would it go off screen?
                        cX = glConfig.vidWidth - rad;
                    } else if cX < 0 {
                        //cap it off at 0
                        cX = 0;
                    }

                    if cY + rad > glConfig.vidHeight {
                        //would it go off screen?
                        cY = glConfig.vidHeight - rad;
                    } else if cY < 0 {
                        //cap it off at 0
                        cY = 0;
                    }

                    //now copy a portion of the screen to this texture
                    #[cfg(feature = "_XBOX")]
                    qglCopyBackBufferToTexEXT(rad, rad, cX, (480 - cY), (cX + rad), (480 - (cY + rad)));
                    #[cfg(not(feature = "_XBOX"))]
                    qglCopyTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA16, cX, cY, rad, rad, 0);

                    lastPostEnt = (*pRender).entNum;
                }
            }

            (rb_surfaceTable[*(*(*pRender).drawSurf).surface as usize])((*(*pRender).drawSurf).surface as *mut c_void);
            RB_EndSurface();
        }
    }

    // go back to the world modelview matrix
    qglLoadMatrixf(backEnd.viewParms.world.modelMatrix.as_ptr());
    if depthRange != 0 {
        qglDepthRange(0.0f64, 1.0f64);
    }

    // #if 0
    // RB_DrawSun();
    // #endif
    if tr_stencilled && tr_distortionPrePost == 0 {
        //draw in the stencil buffer's cutout
        RB_DistortionFill();
    }
    if !didShadowPass {
        // darken down any stencil shadows
        RB_ShadowFinish();
        didShadowPass = true;
    }

    #[cfg(feature = "_XBOX")]
    if (*r_hdreffect).integer != 0 {
        HDREffect.Render();
    }

    // add light flares on lights that aren't obscured
//	RB_RenderFlares();

    #[cfg(feature = "__MACOS__")]
    Sys_PumpEvents(); // crutch up the mac's limited buffer queue size
}


/*
============================================================================

RENDER BACK END THREAD FUNCTIONS

============================================================================
*/

/*
================
RB_SetGL2D

================
*/
pub unsafe fn RB_SetGL2D() {
    backEnd.projection2D = qtrue;

    // set 2D virtual screen size
    qglViewport(0, 0, glConfig.vidWidth, glConfig.vidHeight);
    qglScissor(0, 0, glConfig.vidWidth, glConfig.vidHeight);
    qglMatrixMode(GL_PROJECTION);
    qglLoadIdentity();
    #[cfg(feature = "_XBOX")]
    qglOrtho(0.0f64, 640.0f64, 0.0f64, 480.0f64, 0.0f64, 1.0f64);
    #[cfg(not(feature = "_XBOX"))]
    qglOrtho(0.0f64, 640.0f64, 480.0f64, 0.0f64, 0.0f64, 1.0f64);
    qglMatrixMode(GL_MODELVIEW);
    qglLoadIdentity();

    GL_State(GLS_DEPTHTEST_DISABLE as c_ulong
           | GLS_SRCBLEND_SRC_ALPHA as c_ulong
           | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA as c_ulong);

    qglDisable(GL_CULL_FACE);
    qglDisable(GL_CLIP_PLANE0);

    // set time for 2D shaders
    backEnd.refdef.time = Sys_Milliseconds();
    backEnd.refdef.floatTime = backEnd.refdef.time as f32 * 0.001f32;
}


/*
=============
RB_SetColor

=============
*/
pub unsafe fn RB_SetColor(data: *const c_void) -> *const c_void {
    let cmd: *const setColorCommand_t = data as *const setColorCommand_t;

    backEnd.color2D[0] = ((*cmd).color[0] * 255.0f32) as c_uchar;
    backEnd.color2D[1] = ((*cmd).color[1] * 255.0f32) as c_uchar;
    backEnd.color2D[2] = ((*cmd).color[2] * 255.0f32) as c_uchar;
    backEnd.color2D[3] = ((*cmd).color[3] * 255.0f32) as c_uchar;

    cmd.add(1) as *const c_void
}

/*
=============
RB_StretchPic
=============
*/
pub unsafe fn RB_StretchPic(data: *const c_void) -> *const c_void {
    let cmd: *const stretchPicCommand_t = data as *const stretchPicCommand_t;
    let shader: *mut shader_t;
    let mut numVerts: c_int;
    let mut numIndexes: c_int;

    shader = (*cmd).shader;
    if shader != tess.shader {
        if tess.numIndexes != 0 {
            RB_EndSurface(); //this might change culling and other states
        }
        backEnd.currentEntity = &mut backEnd.entity2D;
        RB_BeginSurface(shader, 0);
    }

    if backEnd.projection2D == 0 {
        RB_SetGL2D(); //set culling and other states
    }

    RB_CHECKOVERFLOW(4, 6);
    numVerts = tess.numVertexes;
    numIndexes = tess.numIndexes;

    tess.numVertexes += 4;
    tess.numIndexes += 6;

    tess.indexes[numIndexes as usize] = (numVerts + 3) as glIndex_t;
    tess.indexes[(numIndexes + 1) as usize] = (numVerts + 0) as glIndex_t;
    tess.indexes[(numIndexes + 2) as usize] = (numVerts + 2) as glIndex_t;
    tess.indexes[(numIndexes + 3) as usize] = (numVerts + 2) as glIndex_t;
    tess.indexes[(numIndexes + 4) as usize] = (numVerts + 0) as glIndex_t;
    tess.indexes[(numIndexes + 5) as usize] = (numVerts + 1) as glIndex_t;

    // *(int *)tess.vertexColors[ numVerts ] =
    //     *(int *)tess.vertexColors[ numVerts + 1 ] =
    //     *(int *)tess.vertexColors[ numVerts + 2 ] =
    //     *(int *)tess.vertexColors[ numVerts + 3 ] = *(int *)backEnd.color2D;
    let color_int: c_int = *(backEnd.color2D.as_ptr() as *const c_int);
    *(tess.vertexColors[numVerts as usize].as_mut_ptr() as *mut c_int) = color_int;
    *(tess.vertexColors[(numVerts + 1) as usize].as_mut_ptr() as *mut c_int) = color_int;
    *(tess.vertexColors[(numVerts + 2) as usize].as_mut_ptr() as *mut c_int) = color_int;
    *(tess.vertexColors[(numVerts + 3) as usize].as_mut_ptr() as *mut c_int) = color_int;

    tess.xyz[numVerts as usize][0] = (*cmd).x;
    tess.xyz[numVerts as usize][1] = (*cmd).y;
    tess.xyz[numVerts as usize][2] = 0.0f32;

    tess.texCoords[numVerts as usize][0][0] = (*cmd).s1;
    tess.texCoords[numVerts as usize][0][1] = (*cmd).t1;

    tess.xyz[(numVerts + 1) as usize][0] = (*cmd).x + (*cmd).w;
    tess.xyz[(numVerts + 1) as usize][1] = (*cmd).y;
    tess.xyz[(numVerts + 1) as usize][2] = 0.0f32;

    tess.texCoords[(numVerts + 1) as usize][0][0] = (*cmd).s2;
    tess.texCoords[(numVerts + 1) as usize][0][1] = (*cmd).t1;

    tess.xyz[(numVerts + 2) as usize][0] = (*cmd).x + (*cmd).w;
    tess.xyz[(numVerts + 2) as usize][1] = (*cmd).y + (*cmd).h;
    tess.xyz[(numVerts + 2) as usize][2] = 0.0f32;

    tess.texCoords[(numVerts + 2) as usize][0][0] = (*cmd).s2;
    tess.texCoords[(numVerts + 2) as usize][0][1] = (*cmd).t2;

    tess.xyz[(numVerts + 3) as usize][0] = (*cmd).x;
    tess.xyz[(numVerts + 3) as usize][1] = (*cmd).y + (*cmd).h;
    tess.xyz[(numVerts + 3) as usize][2] = 0.0f32;

    tess.texCoords[(numVerts + 3) as usize][0][0] = (*cmd).s1;
    tess.texCoords[(numVerts + 3) as usize][0][1] = (*cmd).t2;

    cmd.add(1) as *const c_void
}


/*
=============
RB_DrawRotatePic
=============
*/
pub unsafe fn RB_RotatePic(data: *const c_void) -> *const c_void {
    let cmd: *const rotatePicCommand_t = data as *const rotatePicCommand_t;
    let image: *mut image_t;
    let shader: *mut shader_t;

    shader = (*cmd).shader;
    image = &mut (*(*shader).stages[0]).bundle[0].image[0];

    if !image.is_null() {
        if backEnd.projection2D == 0 {
            RB_SetGL2D();
        }

        qglColor4ubv(backEnd.color2D.as_ptr());
        qglPushMatrix();

        qglTranslatef((*cmd).x + (*cmd).w, (*cmd).y, 0.0f32);
        qglRotatef((*cmd).a, 0.0f32, 0.0f32, 1.0f32);

        GL_Bind(image);
        #[cfg(feature = "_XBOX")]
        qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
        #[cfg(not(feature = "_XBOX"))]
        qglBegin(GL_QUADS);
        qglTexCoord2f((*cmd).s1, (*cmd).t1);
        qglVertex2f(-(*cmd).w, 0.0f32);
        qglTexCoord2f((*cmd).s2, (*cmd).t1);
        qglVertex2f(0.0f32, 0.0f32);
        qglTexCoord2f((*cmd).s2, (*cmd).t2);
        qglVertex2f(0.0f32, (*cmd).h);
        qglTexCoord2f((*cmd).s1, (*cmd).t2);
        qglVertex2f(-(*cmd).w, (*cmd).h);
        qglEnd();

        qglPopMatrix();
    }

    cmd.add(1) as *const c_void
}

/*
=============
RB_DrawRotatePic2
=============
*/
pub unsafe fn RB_RotatePic2(data: *const c_void) -> *const c_void {
    let cmd: *const rotatePicCommand_t = data as *const rotatePicCommand_t;
    let image: *mut image_t;
    let shader: *mut shader_t;

    shader = (*cmd).shader;

    if (*shader).numUnfoggedPasses != 0 {
        image = &mut (*(*shader).stages[0]).bundle[0].image[0];

        if !image.is_null() {
            if backEnd.projection2D == 0 {
                RB_SetGL2D();
            }

            // Get our current blend mode, etc.
            GL_State((*(*shader).stages[0]).stateBits as c_ulong);

            qglColor4ubv(backEnd.color2D.as_ptr());
            qglPushMatrix();

            // rotation point is going to be around the center of the passed in coordinates
            qglTranslatef((*cmd).x, (*cmd).y, 0.0f32);
            qglRotatef((*cmd).a, 0.0f32, 0.0f32, 1.0f32);

            GL_Bind(image);
            #[cfg(feature = "_XBOX")]
            qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
            #[cfg(not(feature = "_XBOX"))]
            qglBegin(GL_QUADS);
                qglTexCoord2f((*cmd).s1, (*cmd).t1);
                qglVertex2f(-(*cmd).w * 0.5f32, -(*cmd).h * 0.5f32);

                qglTexCoord2f((*cmd).s2, (*cmd).t1);
                qglVertex2f((*cmd).w * 0.5f32, -(*cmd).h * 0.5f32);

                qglTexCoord2f((*cmd).s2, (*cmd).t2);
                qglVertex2f((*cmd).w * 0.5f32, (*cmd).h * 0.5f32);

                qglTexCoord2f((*cmd).s1, (*cmd).t2);
                qglVertex2f(-(*cmd).w * 0.5f32, (*cmd).h * 0.5f32);
            qglEnd();

            qglPopMatrix();

            // Hmmm, this is not too cool
            GL_State(GLS_DEPTHTEST_DISABLE as c_ulong
                   | GLS_SRCBLEND_SRC_ALPHA as c_ulong
                   | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA as c_ulong);
        }
    }

    cmd.add(1) as *const c_void
}

/*
=============
RB_LAGoggles
=============
*/
pub unsafe fn RB_LAGoggles(data: *const c_void) -> *const c_void {
    data
}

/*
=============
RB_ScissorPic
=============
*/
pub unsafe fn RB_Scissor(data: *const c_void) -> *const c_void {
    let cmd: *const scissorCommand_t = data as *const scissorCommand_t;

    if backEnd.projection2D == 0 {
        RB_SetGL2D();
    }

    if (*cmd).x >= 0 {
        qglScissor((*cmd).x, (glConfig.vidHeight - (*cmd).y - (*cmd).h), (*cmd).w, (*cmd).h);
    } else {
        qglScissor(0, 0, glConfig.vidWidth, glConfig.vidHeight);
    }

    cmd.add(1) as *const c_void
}

/*
=============
RB_DrawSurfs

=============
*/
pub unsafe fn RB_DrawSurfs(data: *const c_void) -> *const c_void {
    let cmd: *const drawSurfsCommand_t;

    // finish any 2D drawing if needed
    if tess.numIndexes != 0 {
        RB_EndSurface();
    }

    cmd = data as *const drawSurfsCommand_t;

    backEnd.refdef = (*cmd).refdef;
    backEnd.viewParms = (*cmd).viewParms;

    RB_RenderDrawSurfList((*cmd).drawSurfs, (*cmd).numDrawSurfs);

    // Dynamic Glow/Flares:
    /*
        The basic idea is to render the glowing parts of the scene to an offscreen buffer, then take
        that buffer and blur it. After it is sufficiently blurred, re-apply that image back to
        the normal screen using a additive blending. To blur the scene I use a vertex program to supply
        four texture coordinate offsets that allow 'peeking' into adjacent pixels. In the register
        combiner (pixel shader), I combine the adjacent pixels using a weighting factor. - Aurelio
    */

    // Render dynamic glowing/flaring objects.
    #[cfg(not(feature = "_XBOX"))] // GLOWXXX
    if (backEnd.refdef.rdflags & RDF_NOWORLDMODEL == 0) && g_bDynamicGlowSupported && (*r_DynamicGlow).integer != 0 {
        // Copy the normal scene to texture.
        qglDisable(GL_TEXTURE_2D);
        qglEnable(GL_TEXTURE_RECTANGLE_EXT);
        qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, tr.sceneImage);
        qglCopyTexSubImage2D(GL_TEXTURE_RECTANGLE_EXT, 0, 0, 0, backEnd.viewParms.viewportX, backEnd.viewParms.viewportY, backEnd.viewParms.viewportWidth, backEnd.viewParms.viewportHeight);
        qglDisable(GL_TEXTURE_RECTANGLE_EXT);
        qglEnable(GL_TEXTURE_2D);

        // Just clear colors, but leave the depth buffer intact so we can 'share' it.
        qglClearColor(0.0f32, 0.0f32, 0.0f32, 0.0f32);
        qglClear(GL_COLOR_BUFFER_BIT);

        // Render the glowing objects.
        g_bRenderGlowingObjects = true;
        RB_RenderDrawSurfList((*cmd).drawSurfs, (*cmd).numDrawSurfs);
        g_bRenderGlowingObjects = false;
        qglFinish();

        // Copy the glow scene to texture.
        qglDisable(GL_TEXTURE_2D);
        qglEnable(GL_TEXTURE_RECTANGLE_EXT);
        qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, tr.screenGlow);
        qglCopyTexSubImage2D(GL_TEXTURE_RECTANGLE_EXT, 0, 0, 0, backEnd.viewParms.viewportX, backEnd.viewParms.viewportY, backEnd.viewParms.viewportWidth, backEnd.viewParms.viewportHeight);
        qglDisable(GL_TEXTURE_RECTANGLE_EXT);
        qglEnable(GL_TEXTURE_2D);

        // Resize the viewport to the blur texture size.
        let oldViewWidth: c_int = backEnd.viewParms.viewportWidth;
        let oldViewHeight: c_int = backEnd.viewParms.viewportHeight;
        backEnd.viewParms.viewportWidth = (*r_DynamicGlowWidth).integer;
        backEnd.viewParms.viewportHeight = (*r_DynamicGlowHeight).integer;
        SetViewportAndScissor();

        // Blur the scene.
        RB_BlurGlowTexture();

        // Copy the finished glow scene back to texture.
        qglDisable(GL_TEXTURE_2D);
        qglEnable(GL_TEXTURE_RECTANGLE_EXT);
        qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, tr.blurImage);
        qglCopyTexSubImage2D(GL_TEXTURE_RECTANGLE_EXT, 0, 0, 0, 0, 0, backEnd.viewParms.viewportWidth, backEnd.viewParms.viewportHeight);
        qglDisable(GL_TEXTURE_RECTANGLE_EXT);
        qglEnable(GL_TEXTURE_2D);

        // Set the viewport back to normal.
        backEnd.viewParms.viewportWidth = oldViewWidth;
        backEnd.viewParms.viewportHeight = oldViewHeight;
        SetViewportAndScissor();
        qglClear(GL_COLOR_BUFFER_BIT);

        // Draw the glow additively over the screen.
        RB_DrawGlowOverlay();
    }
    // #endif // _XBOX

    cmd.add(1) as *const c_void
}


/*
=============
RB_DrawBuffer

=============
*/
pub unsafe fn RB_DrawBuffer(data: *const c_void) -> *const c_void {
    let cmd: *const drawBufferCommand_t = data as *const drawBufferCommand_t;

    qglDrawBuffer((*cmd).buffer);

        // clear screen for debugging
    if (backEnd.refdef.rdflags & RDF_NOWORLDMODEL == 0) && !tr.world.is_null() && tr.refdef.rdflags & RDF_doLAGoggles != 0 {
        let fog: *const fog_t = &(*tr.world).fogs[(*tr.world).numfogs as usize];

        qglClearColor((*fog).parms.color[0], (*fog).parms.color[1], (*fog).parms.color[2], 1.0f32);
        qglClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    } else if (backEnd.refdef.rdflags & RDF_NOWORLDMODEL == 0) && !tr.world.is_null() && (*tr.world).globalFog != -1 && tr.sceneCount != 0 {
        //don't clear during menus, wait for real scene
        let fog: *const fog_t = &(*tr.world).fogs[(*tr.world).globalFog as usize];

        qglClearColor((*fog).parms.color[0], (*fog).parms.color[1], (*fog).parms.color[2], 1.0f32);
        qglClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    } else if (*r_clear).integer != 0 {
        // clear screen for debugging
        let mut i: c_int = (*r_clear).integer;
        if i == 42 {
            i = Q_irand(0, 8);
        }
        if i == 1 {
            qglClearColor(1.0f32, 0.0f32, 0.0f32, 1.0f32); //red
        } else if i == 2 {
            qglClearColor(0.0f32, 1.0f32, 0.0f32, 1.0f32); //green
        } else if i == 3 {
            qglClearColor(1.0f32, 1.0f32, 0.0f32, 1.0f32); //yellow
        } else if i == 4 {
            qglClearColor(0.0f32, 0.0f32, 1.0f32, 1.0f32); //blue
        } else if i == 5 {
            qglClearColor(0.0f32, 1.0f32, 1.0f32, 1.0f32); //cyan
        } else if i == 6 {
            qglClearColor(1.0f32, 0.0f32, 1.0f32, 1.0f32); //magenta
        } else if i == 7 {
            qglClearColor(1.0f32, 1.0f32, 1.0f32, 1.0f32); //white
        } else if i == 8 {
            qglClearColor(0.0f32, 0.0f32, 0.0f32, 1.0f32); //black
        } else {
            qglClearColor(1.0f32, 0.0f32, 0.5f32, 1.0f32);
        }
        qglClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }

    cmd.add(1) as *const c_void
}

/*
===============
RB_ShowImages

Draw all the images to the screen, on top of whatever
was there.  This is used to test for texture thrashing.

Also called by RE_EndRegistration
===============
*/
pub unsafe fn RB_ShowImages() {
    let mut image: *mut image_t;
    let mut x: f32;
    let mut y: f32;
    let mut w: f32;
    let mut h: f32;
    let mut start: c_int;
    let mut end: c_int;

    if backEnd.projection2D == 0 {
        RB_SetGL2D();
    }

    qglFinish();

    start = Sys_Milliseconds();

    let mut i: c_int = 0;
//	int iNumImages =
                     R_Images_StartIteration();
    loop {
        image = R_Images_GetNextIteration();
        if image.is_null() {
            break;
        }

        w = glConfig.vidWidth as f32 / 20.0f32;
        h = glConfig.vidHeight as f32 / 15.0f32;
        x = (i % 20) as f32 * w;
        y = (i / 20) as f32 * h;

        // show in proportional size in mode 2
        if (*r_showImages).integer == 2 {
            w *= (*image).width as f32 / 512.0f32;
            h *= (*image).height as f32 / 512.0f32;
        }

        GL_Bind(image);
        #[cfg(feature = "_XBOX")]
        qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
        #[cfg(not(feature = "_XBOX"))]
        qglBegin(GL_QUADS);
        qglTexCoord2f(0.0f32, 0.0f32);
        qglVertex2f(x, y);
        qglTexCoord2f(1.0f32, 0.0f32);
        qglVertex2f(x + w, y);
        qglTexCoord2f(1.0f32, 1.0f32);
        qglVertex2f(x + w, y + h);
        qglTexCoord2f(0.0f32, 1.0f32);
        qglVertex2f(x, y + h);
        qglEnd();
        i += 1;
    }

    qglFinish();

    end = Sys_Milliseconds();
    //VID_Printf( PRINT_ALL, "%i msec to draw all images\n", end - start );
}


/*
=============
RB_SwapBuffers

=============
*/
pub unsafe fn RB_SwapBuffers(data: *const c_void) -> *const c_void {
    let cmd: *const swapBuffersCommand_t;

    // finish any 2D drawing if needed
    if tess.numIndexes != 0 {
        RB_EndSurface();
    }

    // texture swapping test
    if (*r_showImages).integer != 0 {
        RB_ShowImages();
    }

    cmd = data as *const swapBuffersCommand_t;

    // we measure overdraw by reading back the stencil buffer and
    // counting up the number of increments that have happened
    #[cfg(not(feature = "_XBOX"))]
    if (*r_measureOverdraw).integer != 0 {
        let mut i: c_int;
        let mut sum: c_long = 0;
        let stencilReadback: *mut c_uchar;

        stencilReadback = Z_Malloc(glConfig.vidWidth * glConfig.vidHeight, TAG_TEMP_WORKSPACE, qfalse) as *mut c_uchar;
        qglReadPixels(0, 0, glConfig.vidWidth, glConfig.vidHeight, GL_STENCIL_INDEX, GL_UNSIGNED_BYTE, stencilReadback as *mut c_void);

        i = 0;
        while i < glConfig.vidWidth * glConfig.vidHeight {
            sum += *stencilReadback.add(i as usize) as c_long;
            i += 1;
        }

        backEnd.pc.c_overDraw += sum;
        Z_Free(stencilReadback as *mut c_void);
    }

    if glState.finishCalled == 0 {
        qglFinish();
    }

    GLimp_LogComment(b"***************** RB_SwapBuffers *****************\n\n\n\0".as_ptr() as *const c_char);

    GLimp_EndFrame();

    backEnd.projection2D = qfalse;

    cmd.add(1) as *const c_void
}

pub unsafe fn RB_WorldEffects(data: *const c_void) -> *const c_void {
    let cmd: *const setModeCommand_t = data as *const setModeCommand_t;

    // Always flush the tess buffer
    if !tess.shader.is_null() && tess.numIndexes != 0 {
        RB_EndSurface();
    }

    RB_RenderWorldEffects();

    if !tess.shader.is_null() {
        RB_BeginSurface(tess.shader, tess.fogNum);
    }

    cmd.add(1) as *const c_void
}

/*
====================
RB_ExecuteRenderCommands

This function will be called syncronously if running without
smp extensions, or asyncronously by another thread.
====================
*/
pub unsafe fn RB_ExecuteRenderCommands(data: *const c_void) {
    let mut t1: c_int;
    let mut t2: c_int;
    let mut data: *const c_void = data;

    t1 = Sys_Milliseconds();

    loop {
        let cmd_id: c_int = *(data as *const c_int);
        if cmd_id == RC_SET_COLOR {
            data = RB_SetColor(data);
        } else if cmd_id == RC_STRETCH_PIC {
            data = RB_StretchPic(data);
        } else if cmd_id == RC_ROTATE_PIC {
            data = RB_RotatePic(data);
        } else if cmd_id == RC_ROTATE_PIC2 {
            data = RB_RotatePic2(data);
        } else if cmd_id == RC_SCISSOR {
            data = RB_Scissor(data);
        } else if cmd_id == RC_DRAW_SURFS {
            data = RB_DrawSurfs(data);
        } else if cmd_id == RC_DRAW_BUFFER {
            data = RB_DrawBuffer(data);
        } else if cmd_id == RC_SWAP_BUFFERS {
            data = RB_SwapBuffers(data);
        } else if cmd_id == RC_WORLD_EFFECTS {
            data = RB_WorldEffects(data);
        } else {
            // RC_END_OF_LIST default:
            // stop rendering on this thread
            t2 = Sys_Milliseconds();
            backEnd.pc.msec = t2 - t1;
            return;
        }
    }
}

#[cfg(not(feature = "_XBOX"))] // GLOWXXX
// What Pixel Shader type is currently active (regcoms or fragment programs).
pub static mut g_uiCurrentPixelShaderType: GLuint = 0x0;

// Begin using a Pixel Shader.
#[cfg(not(feature = "_XBOX"))]
pub unsafe fn BeginPixelShader(uiType: GLuint, uiID: GLuint) {
    if uiType == GL_REGISTER_COMBINERS_NV {
        // Using Register Combiners, so call the Display List that stores it.

        // Just in case...
        if qglCombinerParameterfvNV.is_none() {
            return;
        }

        // Call the list with the regcom in it.
        qglEnable(GL_REGISTER_COMBINERS_NV);
        qglCallList(uiID);

        g_uiCurrentPixelShaderType = GL_REGISTER_COMBINERS_NV;
        return;
    }

    if uiType == GL_FRAGMENT_PROGRAM_ARB {
        // Using Fragment Programs, so call the program.

        // Just in case...
        if qglGenProgramsARB.is_none() {
            return;
        }

        qglEnable(GL_FRAGMENT_PROGRAM_ARB);
        qglBindProgramARB(GL_FRAGMENT_PROGRAM_ARB, uiID);

        g_uiCurrentPixelShaderType = GL_FRAGMENT_PROGRAM_ARB;
        return;
    }
}

// Stop using a Pixel Shader and return states to normal.
#[cfg(not(feature = "_XBOX"))]
pub unsafe fn EndPixelShader() {
    if g_uiCurrentPixelShaderType == 0x0 {
        return;
    }

    qglDisable(g_uiCurrentPixelShaderType);
}

// Hack variable for deciding which kind of texture rectangle thing to do (for some
// reason it acts different on radeon! It's against the spec!).
// extern bool g_bTextureRectangleHack;  -- declared in extern "C" block above

#[cfg(not(feature = "_XBOX"))]
#[inline]
unsafe fn RB_BlurGlowTexture() {
    qglDisable(GL_CLIP_PLANE0);
    GL_Cull(CT_TWO_SIDED);

    // Go into orthographic 2d mode.
    qglMatrixMode(GL_PROJECTION);
    qglPushMatrix();
    qglLoadIdentity();
    qglOrtho(0.0f64, backEnd.viewParms.viewportWidth as f64, backEnd.viewParms.viewportHeight as f64, 0.0f64, -1.0f64, 1.0f64);
    qglMatrixMode(GL_MODELVIEW);
    qglPushMatrix();
    qglLoadIdentity();

    GL_State(GLS_DEPTHTEST_DISABLE as c_ulong);

    /////////////////////////////////////////////////////////
    // Setup vertex and pixel programs.
    /////////////////////////////////////////////////////////

    // NOTE: The 0.25 is because we're blending 4 textures (so = 1.0) and we want a relatively normalized pixel
    // intensity distribution, but this won't happen anyways if intensity is higher than 1.0.
    let fBlurDistribution: f32 = (*r_DynamicGlowIntensity).value * 0.25f32;
    let fBlurWeight: [f32; 4] = [fBlurDistribution, fBlurDistribution, fBlurDistribution, 1.0f32];

    // Enable and set the Vertex Program.
    qglEnable(GL_VERTEX_PROGRAM_ARB);
    qglBindProgramARB(GL_VERTEX_PROGRAM_ARB, tr.glowVShader);

    // Apply Pixel Shaders.
    if qglCombinerParameterfvNV.is_some() {
        BeginPixelShader(GL_REGISTER_COMBINERS_NV, tr.glowPShader);

        // Pass the blur weight to the regcom.
        qglCombinerParameterfvNV(GL_CONSTANT_COLOR0_NV, fBlurWeight.as_ptr());
    } else if qglProgramEnvParameter4fARB.is_some() {
        BeginPixelShader(GL_FRAGMENT_PROGRAM_ARB, tr.glowPShader);

        // Pass the blur weight to the Fragment Program.
        qglProgramEnvParameter4fARB(GL_FRAGMENT_PROGRAM_ARB, 0, fBlurWeight[0], fBlurWeight[1], fBlurWeight[2], fBlurWeight[3]);
    }

    /////////////////////////////////////////////////////////
    // Set the blur texture to the 4 texture stages.
    /////////////////////////////////////////////////////////

    // How much to offset each texel by.
    let mut fTexelWidthOffset: f32 = 0.1f32;
    let mut fTexelHeightOffset: f32 = 0.1f32;

    let mut uiTex: GLuint = tr.screenGlow;

    qglActiveTextureARB(GL_TEXTURE3_ARB);
    qglEnable(GL_TEXTURE_RECTANGLE_EXT);
    qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);

    qglActiveTextureARB(GL_TEXTURE2_ARB);
    qglEnable(GL_TEXTURE_RECTANGLE_EXT);
    qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);

    qglActiveTextureARB(GL_TEXTURE1_ARB);
    qglEnable(GL_TEXTURE_RECTANGLE_EXT);
    qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);

    qglActiveTextureARB(GL_TEXTURE0_ARB);
    qglDisable(GL_TEXTURE_2D);
    qglEnable(GL_TEXTURE_RECTANGLE_EXT);
    qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);

    /////////////////////////////////////////////////////////
    // Draw the blur passes (each pass blurs it more, increasing the blur radius ).
    /////////////////////////////////////////////////////////

    //int iTexWidth = backEnd.viewParms.viewportWidth, iTexHeight = backEnd.viewParms.viewportHeight;
    let mut iTexWidth: c_int = glConfig.vidWidth;
    let mut iTexHeight: c_int = glConfig.vidHeight;

    let mut iNumBlurPasses: c_int = 0;
    while iNumBlurPasses < (*r_DynamicGlowPasses).integer {
        // Load the Texel Offsets into the Vertex Program.
        qglProgramEnvParameter4fARB(GL_VERTEX_PROGRAM_ARB, 0, -fTexelWidthOffset, -fTexelWidthOffset, 0.0f32, 0.0f32);
        qglProgramEnvParameter4fARB(GL_VERTEX_PROGRAM_ARB, 1, -fTexelWidthOffset, fTexelWidthOffset, 0.0f32, 0.0f32);
        qglProgramEnvParameter4fARB(GL_VERTEX_PROGRAM_ARB, 2, fTexelWidthOffset, -fTexelWidthOffset, 0.0f32, 0.0f32);
        qglProgramEnvParameter4fARB(GL_VERTEX_PROGRAM_ARB, 3, fTexelWidthOffset, fTexelWidthOffset, 0.0f32, 0.0f32);

        // After first pass put the tex coords to the viewport size.
        if iNumBlurPasses == 1 {
            if !g_bTextureRectangleHack {
                iTexWidth = backEnd.viewParms.viewportWidth;
                iTexHeight = backEnd.viewParms.viewportHeight;
            }

            uiTex = tr.blurImage;
            qglActiveTextureARB(GL_TEXTURE3_ARB);
            qglDisable(GL_TEXTURE_2D);
            qglEnable(GL_TEXTURE_RECTANGLE_EXT);
            qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);
            qglActiveTextureARB(GL_TEXTURE2_ARB);
            qglDisable(GL_TEXTURE_2D);
            qglEnable(GL_TEXTURE_RECTANGLE_EXT);
            qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);
            qglActiveTextureARB(GL_TEXTURE1_ARB);
            qglDisable(GL_TEXTURE_2D);
            qglEnable(GL_TEXTURE_RECTANGLE_EXT);
            qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);
            qglActiveTextureARB(GL_TEXTURE0_ARB);
            qglDisable(GL_TEXTURE_2D);
            qglEnable(GL_TEXTURE_RECTANGLE_EXT);
            qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);

            // Copy the current image over.
            qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, uiTex);
            qglCopyTexSubImage2D(GL_TEXTURE_RECTANGLE_EXT, 0, 0, 0, 0, 0, backEnd.viewParms.viewportWidth, backEnd.viewParms.viewportHeight);
        }

        // Draw the fullscreen quad.
        qglBegin(GL_QUADS);
            qglMultiTexCoord2fARB(GL_TEXTURE0_ARB, 0.0f32, iTexHeight as f32);
            qglVertex2f(0.0f32, 0.0f32);

            qglMultiTexCoord2fARB(GL_TEXTURE0_ARB, 0.0f32, 0.0f32);
            qglVertex2f(0.0f32, backEnd.viewParms.viewportHeight as f32);

            qglMultiTexCoord2fARB(GL_TEXTURE0_ARB, iTexWidth as f32, 0.0f32);
            qglVertex2f(backEnd.viewParms.viewportWidth as f32, backEnd.viewParms.viewportHeight as f32);

            qglMultiTexCoord2fARB(GL_TEXTURE0_ARB, iTexWidth as f32, iTexHeight as f32);
            qglVertex2f(backEnd.viewParms.viewportWidth as f32, 0.0f32);
        qglEnd();

        qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, tr.blurImage);
        qglCopyTexSubImage2D(GL_TEXTURE_RECTANGLE_EXT, 0, 0, 0, 0, 0, backEnd.viewParms.viewportWidth, backEnd.viewParms.viewportHeight);

        // Increase the texel offsets.
        // NOTE: This is possibly the most important input to the effect. Even by using an exponential function I've been able to
        // make it look better (at a much higher cost of course). This is cheap though and still looks pretty great. In the future
        // I might want to use an actual gaussian equation to correctly calculate the pixel coefficients and attenuates, texel
        // offsets, gaussian amplitude and radius...
        fTexelWidthOffset += (*r_DynamicGlowDelta).value;
        fTexelHeightOffset += (*r_DynamicGlowDelta).value;

        iNumBlurPasses += 1;
    }

    // Disable multi-texturing.
    qglActiveTextureARB(GL_TEXTURE3_ARB);
    qglDisable(GL_TEXTURE_RECTANGLE_EXT);

    qglActiveTextureARB(GL_TEXTURE2_ARB);
    qglDisable(GL_TEXTURE_RECTANGLE_EXT);

    qglActiveTextureARB(GL_TEXTURE1_ARB);
    qglDisable(GL_TEXTURE_RECTANGLE_EXT);

    qglActiveTextureARB(GL_TEXTURE0_ARB);
    qglDisable(GL_TEXTURE_RECTANGLE_EXT);
    qglEnable(GL_TEXTURE_2D);

    qglDisable(GL_VERTEX_PROGRAM_ARB);
    EndPixelShader();

    qglMatrixMode(GL_PROJECTION);
    qglPopMatrix();
    qglMatrixMode(GL_MODELVIEW);
    qglPopMatrix();

    qglDisable(GL_BLEND);
    glState.currenttmu = 0; //this matches the last one we activated
}

// Draw the glow blur over the screen additively.
#[cfg(not(feature = "_XBOX"))]
#[inline]
unsafe fn RB_DrawGlowOverlay() {
    qglDisable(GL_CLIP_PLANE0);
    GL_Cull(CT_TWO_SIDED);

    // Go into orthographic 2d mode.
    qglMatrixMode(GL_PROJECTION);
    qglPushMatrix();
    qglLoadIdentity();
    qglOrtho(0.0f64, glConfig.vidWidth as f64, glConfig.vidHeight as f64, 0.0f64, -1.0f64, 1.0f64);
    qglMatrixMode(GL_MODELVIEW);
    qglPushMatrix();
    qglLoadIdentity();

    GL_State(GLS_DEPTHTEST_DISABLE as c_ulong);

    qglDisable(GL_TEXTURE_2D);
    qglEnable(GL_TEXTURE_RECTANGLE_EXT);

    // For debug purposes.
    if (*r_DynamicGlow).integer != 2 {
        // Render the normal scene texture.
        qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, tr.sceneImage);
        qglBegin(GL_QUADS);
            qglColor4f(1.0f32, 1.0f32, 1.0f32, 1.0f32);
            qglTexCoord2f(0.0f32, glConfig.vidHeight as f32);
            qglVertex2f(0.0f32, 0.0f32);

            qglTexCoord2f(0.0f32, 0.0f32);
            qglVertex2f(0.0f32, glConfig.vidHeight as f32);

            qglTexCoord2f(glConfig.vidWidth as f32, 0.0f32);
            qglVertex2f(glConfig.vidWidth as f32, glConfig.vidHeight as f32);

            qglTexCoord2f(glConfig.vidWidth as f32, glConfig.vidHeight as f32);
            qglVertex2f(glConfig.vidWidth as f32, 0.0f32);
        qglEnd();
    }

    // One and Inverse Src Color give a very soft addition, while one one is a bit stronger. With one one we can
    // use additive blending through multitexture though.
    if (*r_DynamicGlowSoft).integer != 0 {
        qglBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_COLOR);
    } else {
        qglBlendFunc(GL_ONE, GL_ONE);
    }
    qglEnable(GL_BLEND);

    // Now additively render the glow texture.
    qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, tr.blurImage);
    qglBegin(GL_QUADS);
        qglColor4f(1.0f32, 1.0f32, 1.0f32, 1.0f32);
        qglTexCoord2f(0.0f32, (*r_DynamicGlowHeight).integer as f32);
        qglVertex2f(0.0f32, 0.0f32);

        qglTexCoord2f(0.0f32, 0.0f32);
        qglVertex2f(0.0f32, glConfig.vidHeight as f32);

        qglTexCoord2f((*r_DynamicGlowWidth).integer as f32, 0.0f32);
        qglVertex2f(glConfig.vidWidth as f32, glConfig.vidHeight as f32);

        qglTexCoord2f((*r_DynamicGlowWidth).integer as f32, (*r_DynamicGlowHeight).integer as f32);
        qglVertex2f(glConfig.vidWidth as f32, 0.0f32);
    qglEnd();

    qglDisable(GL_TEXTURE_RECTANGLE_EXT);
    qglEnable(GL_TEXTURE_2D);
    qglBlendFunc(GL_SRC_COLOR, GL_ONE_MINUS_SRC_COLOR);
    qglDisable(GL_BLEND);

    qglMatrixMode(GL_PROJECTION);
    qglPopMatrix();
    qglMatrixMode(GL_MODELVIEW);
    qglPopMatrix();
}
// #endif  (not(_XBOX))
