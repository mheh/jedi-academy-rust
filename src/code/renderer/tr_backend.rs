// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_void};

// LOCAL STUBS FOR EXTERNAL TYPES
// These are declared here as opaque types since their full definitions
// are in other modules (tr_local.h, server headers, etc.)

// External dependencies not fully defined in this file
extern "C" {
    fn VID_Printf(level: c_int, fmt: *const i8, ...);
    fn Com_Error(level: c_int, fmt: *const i8, ...);
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn Sys_Milliseconds() -> c_int;
    fn Sys_PumpEvents();
    fn GLimp_LogComment(comment: *const i8);
    fn GLimp_EndFrame();
    fn DotProduct(a: *const f32, b: *const f32) -> f32;
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
    fn Z_Malloc(size: usize, tag: c_int, clear: bool) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn R_DecomposeSort(sort: u32, entity_num: *mut c_int, shader: *mut *mut shader_t, fog_num: *mut c_int, dlighted: *mut c_int);
    fn R_RotateForEntity(entity: *const trRefEntity_t, view_parms: *const viewParms_t, ori: *mut orientationr_t);
    fn R_TransformDlights(num_dlights: c_int, dlights: *const dlight_t, ori: *const orientationr_t);
    fn R_Images_StartIteration() -> c_int;
    fn R_Images_GetNextIteration() -> *mut image_t;
    fn RB_EndSurface();
    fn RB_BeginSurface(shader: *mut shader_t, fog_num: c_int);
    fn RB_ShadowFinish();

    // GL functions
    fn qglBindTexture(target: u32, texture: u32);
    fn qglActiveTextureARB(texture: u32);
    fn qglClientActiveTextureARB(texture: u32);
    fn qglDisable(cap: u32);
    fn qglEnable(cap: u32);
    fn qglCullFace(mode: u32);
    fn qglDepthFunc(func: u32);
    fn qglDepthMask(flag: u8);
    fn qglDepthRange(near: f64, far: f64);
    fn qglFinish();
    fn qglClear(mask: u32);
    fn qglClearColor(red: f32, green: f32, blue: f32, alpha: f32);
    fn qglLoadMatrixf(m: *const f32);
    fn qglViewport(x: c_int, y: c_int, width: c_int, height: c_int);
    fn qglScissor(x: c_int, y: c_int, width: c_int, height: c_int);
    fn qglTexEnvf(target: u32, pname: u32, param: f32);
    fn qglBlendFunc(sfactor: u32, dfactor: u32);
    fn qglPolygonMode(face: u32, mode: u32);
    fn qglAlphaFunc(func: u32, ref_val: f32);
    fn qglMatrixMode(mode: u32);
    fn qglLoadIdentity();
    fn qglPushMatrix();
    fn qglPopMatrix();
    fn qglTranslatef(x: f32, y: f32, z: f32);
    fn qglRotatef(angle: f32, x: f32, y: f32, z: f32);
    fn qglOrtho(left: f64, right: f64, bottom: f64, top: f64, near_val: f64, far_val: f64);
    fn qglClipPlane(plane: u32, equation: *const f64);
    fn qglDrawBuffer(mode: u32);
    fn qglBegin(mode: u32);
    fn qglEnd();
    fn qglVertex2f(x: f32, y: f32);
    fn qglVertex3f(x: f32, y: f32, z: f32);
    fn qglTexCoord2f(s: f32, t: f32);
    fn qglColor4ubv(v: *const u8);
    fn qglColor4f(red: f32, green: f32, blue: f32, alpha: f32);
    fn qglReadPixels(x: c_int, y: c_int, width: c_int, height: c_int, format: u32, type_: u32, pixels: *mut c_void);
    fn qglCopyTexImage2D(target: u32, level: c_int, internalformat: u32, x: c_int, y: c_int, width: c_int, height: c_int, border: c_int);
    fn qglCopyBackBufferToTexEXT(width: c_int, height: c_int, x: c_int, y: c_int, x2: c_int, y2: c_int);
    fn qglCopyTexSubImage2D(target: u32, level: c_int, xoffset: c_int, yoffset: c_int, x: c_int, y: c_int, width: c_int, height: c_int);
    fn qglBeginEXT(mode: u32, count: c_int, a: c_int, b: c_int, c: c_int, d: c_int);
    fn qglMultiTexCoord2fARB(target: u32, s: f32, t: f32);
    fn qglCombinerParameterfvNV(pname: u32, params: *const f32);
    fn qglCallList(list: u32);
    fn qglGenProgramsARB(n: c_int, programs: *mut u32);
    fn qglBindProgramARB(target: u32, program: u32);
    fn qglProgramEnvParameter4fARB(target: u32, index: u32, x: f32, y: f32, z: f32, w: f32);

    static mut tr: tr_t;
    static mut glState: glState_t;
    static mut glConfig: glConfig_t;
    static mut backEndData: *mut backEndData_t;
    static mut backEnd: backEndState_t;
    static mut tess: tess_t;
    static mut skyboxportal: bool;
    static mut tr_distortionPrePost: bool;
    static mut tr_distortionNegate: bool;
    fn RB_CaptureScreenImage();
    fn RB_DistortionFill();
    fn RB_RenderWorldEffects();

    #[cfg(feature = "VV_LIGHTING")]
    static mut VVLightMan: VVLightManager;
}

#[cfg(feature = "VV_LIGHTING")]
extern "C" {
    pub struct VVLightManager {
        _unused: [u8; 0],
    }

    impl VVLightManager {
        fn R_TransformDlights(&mut self, ori: *const orientationr_t);
    }
}

// Opaque external types
#[repr(C)]
pub struct backEndData_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct backEndState_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct tr_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct glState_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct glConfig_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct tess_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct image_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct shader_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct drawSurf_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct trRefEntity_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct viewParms_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct orientationr_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct dlight_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct fog_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct setColorCommand_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct stretchPicCommand_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct rotatePicCommand_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct scissorCommand_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct drawSurfsCommand_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct drawBufferCommand_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct swapBuffersCommand_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct setModeCommand_t {
    _unused: [u8; 0],
}

static mut tr_stencilled: bool = false;

// Whether we are currently rendering only glowing objects or not.
static mut g_bRenderGlowingObjects: bool = false;

// Whether the current hardware supports dynamic glows/flares.
static mut g_bDynamicGlowSupported: bool = false;

static s_flipMatrix: [f32; 16] = {
    // convert from our coordinate system (looking down X)
    // to OpenGL's coordinate system (looking down -Z)
    #[cfg(target_arch = "x86")]
    {
        0.0, 0.0, 1.0, 0.0,
        -1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    }
    #[cfg(not(target_arch = "x86"))]
    {
        0.0, 0.0, -1.0, 0.0,
        -1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    }
};

// Constants and macros
const PRINT_WARNING: c_int = 1;
const PRINT_ALL: c_int = -1;
const ERR_DROP: c_int = 1;
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_BIND: u32 = 0x2014;
const GL_TEXTURE0_ARB: u32 = 0x84C0;
const GL_TEXTURE1_ARB: u32 = 0x84C1;
const GL_TEXTURE2_ARB: u32 = 0x84C2;
const GL_TEXTURE3_ARB: u32 = 0x84C3;
const GLS_DEPTHFUNC_EQUAL: u32 = 0x00000004;
const GLS_SRCBLEND_BITS: u32 = 0x0000FF00;
const GLS_DSTBLEND_BITS: u32 = 0x000000FF;
const GLS_SRCBLEND_ZERO: u32 = 0x00000100;
const GLS_SRCBLEND_ONE: u32 = 0x00000200;
const GLS_SRCBLEND_DST_COLOR: u32 = 0x00000300;
const GLS_SRCBLEND_ONE_MINUS_DST_COLOR: u32 = 0x00000400;
const GLS_SRCBLEND_SRC_ALPHA: u32 = 0x00000500;
const GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA: u32 = 0x00000600;
const GLS_SRCBLEND_DST_ALPHA: u32 = 0x00000700;
const GLS_SRCBLEND_ONE_MINUS_DST_ALPHA: u32 = 0x00000800;
const GLS_SRCBLEND_ALPHA_SATURATE: u32 = 0x00000900;
const GLS_DSTBLEND_ZERO: u32 = 0x00000001;
const GLS_DSTBLEND_ONE: u32 = 0x00000002;
const GLS_DSTBLEND_SRC_COLOR: u32 = 0x00000003;
const GLS_DSTBLEND_ONE_MINUS_SRC_COLOR: u32 = 0x00000004;
const GLS_DSTBLEND_SRC_ALPHA: u32 = 0x00000005;
const GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA: u32 = 0x00000006;
const GLS_DSTBLEND_DST_ALPHA: u32 = 0x00000007;
const GLS_DSTBLEND_ONE_MINUS_DST_ALPHA: u32 = 0x00000008;
const GLS_DEPTHMASK_TRUE: u32 = 0x00010000;
const GLS_POLYMODE_LINE: u32 = 0x00001000;
const GLS_DEPTHTEST_DISABLE: u32 = 0x00000010;
const GLS_ATEST_BITS: u32 = 0x0000C000;
const GLS_ATEST_GT_0: u32 = 0x00004000;
const GLS_ATEST_LT_80: u32 = 0x00008000;
const GLS_ATEST_GE_80: u32 = 0x0000C000;
const GLS_ATEST_GE_C0: u32 = 0x0000C000;
const GLS_DEFAULT: u32 = 0;
const GL_EQUAL: u32 = 0x0202;
const GL_LEQUAL: u32 = 0x0203;
const GL_ZERO: u32 = 0;
const GL_ONE: u32 = 1;
const GL_DST_COLOR: u32 = 0x0306;
const GL_ONE_MINUS_DST_COLOR: u32 = 0x0307;
const GL_SRC_ALPHA: u32 = 0x0302;
const GL_ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
const GL_DST_ALPHA: u32 = 0x0304;
const GL_ONE_MINUS_DST_ALPHA: u32 = 0x0305;
const GL_SRC_ALPHA_SATURATE: u32 = 0x0308;
const GL_SRC_COLOR: u32 = 0x0300;
const GL_ONE_MINUS_SRC_COLOR: u32 = 0x0301;
const GL_BLEND: u32 = 0x0BE2;
const GL_CULL_FACE: u32 = 0x0B44;
const GL_DEPTH_TEST: u32 = 0x0B71;
const GL_ALPHA_TEST: u32 = 0x0BC0;
const GL_GREATER: u32 = 0x0204;
const GL_LESS: u32 = 0x0201;
const GL_GEQUAL: u32 = 0x0206;
const GL_BACK: u32 = 0x0405;
const GL_FRONT: u32 = 0x0404;
const GL_FRONT_AND_BACK: u32 = 0x0408;
const GL_FILL: u32 = 0x1B02;
const GL_LINE: u32 = 0x1B01;
const GL_MODULATE: u32 = 0x2100;
const GL_REPLACE: u32 = 0x1E01;
const GL_DECAL: u32 = 0x2101;
const GL_ADD: u32 = 0x0104;
const GL_TEXTURE_ENV: u32 = 0x2300;
const GL_TEXTURE_ENV_MODE: u32 = 0x2200;
const GL_NONE: u32 = 0;
const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
const GL_DEPTH_BUFFER_BIT: u32 = 0x00000100;
const GL_STENCIL_BUFFER_BIT: u32 = 0x00000400;
const GL_CLIP_PLANE0: u32 = 0x3000;
const GL_PROJECTION: u32 = 0x1701;
const GL_MODELVIEW: u32 = 0x1700;
const GL_TEXTURE_RECTANGLE_EXT: u32 = 0x84F5;
const GL_REGISTER_COMBINERS_NV: u32 = 0x8522;
const GL_FRAGMENT_PROGRAM_ARB: u32 = 0x8804;
const GL_VERTEX_PROGRAM_ARB: u32 = 0x8620;
const GL_CONSTANT_COLOR0_NV: u32 = 0x852A;
const GL_QUADS: u32 = 0x0007;
const GL_RGBA16: u32 = 0x805B;
const GL_STENCIL_INDEX: u32 = 0x1901;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_FALSE: u8 = 0;
const GL_TRUE: u8 = 1;
const CT_TWO_SIDED: i32 = 0;
const CT_BACK_SIDED: i32 = 1;
const TR_WORLDENT: i32 = -1;
const RDF_SKYBOXPORTAL: u32 = 0x0001;
const RDF_NOWORLDMODEL: u32 = 0x0002;
const RDF_HYPERSPACE: u32 = 0x0010;
const RDF_doLAGoggles: u32 = 0x0200;
const RF_NODEPTH: u32 = 0x80;
const RF_DEPTHHACK: u32 = 0x0001;
const RF_DISTORTION: u32 = 0x0040;
const TAG_TEMP_WORKSPACE: c_int = 16;
const MAC_EVENT_PUMP_MSEC: c_int = 5;
const MAX_POST_RENDERS: usize = 128;
const RC_SET_COLOR: u32 = 0;
const RC_STRETCH_PIC: u32 = 1;
const RC_ROTATE_PIC: u32 = 2;
const RC_ROTATE_PIC2: u32 = 3;
const RC_SCISSOR: u32 = 4;
const RC_DRAW_SURFS: u32 = 5;
const RC_DRAW_BUFFER: u32 = 6;
const RC_SWAP_BUFFERS: u32 = 7;
const RC_WORLD_EFFECTS: u32 = 8;
const RC_END_OF_LIST: u32 = 9;

#[repr(C)]
pub struct postRender_t {
    pub fogNum: c_int,
    pub entNum: c_int,
    pub dlighted: c_int,
    pub depthRange: c_int,
    pub drawSurf: *mut drawSurf_t,
    pub shader: *mut shader_t,
}

static mut g_postRenders: [postRender_t; MAX_POST_RENDERS] = unsafe {
    core::mem::zeroed()
};
static mut g_numPostRenders: c_int = 0;

static mut g_uiCurrentPixelShaderType: u32 = 0x0;
static mut g_bTextureRectangleHack: bool = false;

/*
** GL_Bind
*/
#[allow(non_snake_case)]
pub unsafe fn GL_Bind(image: *mut image_t) {
    let mut texnum: c_int;

    if image.is_null() {
        VID_Printf(PRINT_WARNING, b"GL_Bind: NULL image\n" as *const u8 as *const i8);
        texnum = (*tr.defaultImage).texnum;
    } else {
        texnum = (*image).texnum;
    }

    #[cfg(not(target_os = "windows"))]
    {
        if (*r_nobind).integer != 0 && !(*tr.dlightImage).is_null() {
            // performance evaluation option
            texnum = (*(*tr).dlightImage).texnum;
        }
    }

    if (*glState).currenttextures[(*glState).currenttmu] != texnum {
        #[cfg(not(target_os = "windows"))]
        {
            (*image).frameUsed = (*tr).frameCount;
        }
        (*glState).currenttextures[(*glState).currenttmu] = texnum;
        qglBindTexture(GL_TEXTURE_2D, texnum as u32);
    }
}

/*
** GL_SelectTexture
*/
#[allow(non_snake_case)]
pub unsafe fn GL_SelectTexture(unit: c_int) {
    if (*glState).currenttmu == unit {
        return;
    }

    if unit == 0 {
        qglActiveTextureARB(GL_TEXTURE0_ARB);
        GLimp_LogComment(b"glActiveTextureARB( GL_TEXTURE0_ARB )\n" as *const u8 as *const i8);
        qglClientActiveTextureARB(GL_TEXTURE0_ARB);
        GLimp_LogComment(b"glClientActiveTextureARB( GL_TEXTURE0_ARB )\n" as *const u8 as *const i8);
    } else if unit == 1 {
        qglActiveTextureARB(GL_TEXTURE1_ARB);
        GLimp_LogComment(b"glActiveTextureARB( GL_TEXTURE1_ARB )\n" as *const u8 as *const i8);
        qglClientActiveTextureARB(GL_TEXTURE1_ARB);
        GLimp_LogComment(b"glClientActiveTextureARB( GL_TEXTURE1_ARB )\n" as *const u8 as *const i8);
    } else if unit == 2 {
        qglActiveTextureARB(GL_TEXTURE2_ARB);
        GLimp_LogComment(b"glActiveTextureARB( GL_TEXTURE2_ARB )\n" as *const u8 as *const i8);
        qglClientActiveTextureARB(GL_TEXTURE2_ARB);
        GLimp_LogComment(b"glClientActiveTextureARB( GL_TEXTURE2_ARB )\n" as *const u8 as *const i8);
    } else if unit == 3 {
        qglActiveTextureARB(GL_TEXTURE3_ARB);
        GLimp_LogComment(b"glActiveTextureARB( GL_TEXTURE3_ARB )\n" as *const u8 as *const i8);
        qglClientActiveTextureARB(GL_TEXTURE3_ARB);
        GLimp_LogComment(b"glClientActiveTextureARB( GL_TEXTURE3_ARB )\n" as *const u8 as *const i8);
    } else {
        Com_Error(ERR_DROP, b"GL_SelectTexture: unit = %i\0" as *const u8 as *const i8, unit);
    }

    (*glState).currenttmu = unit;
}

/*
** GL_Cull
*/
#[allow(non_snake_case)]
pub unsafe fn GL_Cull(cullType: c_int) {
    if (*glState).faceCulling == cullType {
        return;
    }
    (*glState).faceCulling = cullType;
    if (*backEnd).projection2D {
        // don't care, we're in 2d when it's always disabled
        return;
    }

    if cullType == CT_TWO_SIDED {
        qglDisable(GL_CULL_FACE);
    } else {
        qglEnable(GL_CULL_FACE);

        if cullType == CT_BACK_SIDED {
            if (*(*backEnd).viewParms).isMirror {
                qglCullFace(GL_FRONT);
            } else {
                qglCullFace(GL_BACK);
            }
        } else {
            if (*(*backEnd).viewParms).isMirror {
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
#[allow(non_snake_case)]
pub unsafe fn GL_TexEnv(env: c_int) {
    if env == (*glState).texEnv[(*glState).currenttmu] {
        return;
    }

    (*glState).texEnv[(*glState).currenttmu] = env;

    match env {
        GL_MODULATE as c_int => {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_MODULATE as f32);
        }
        GL_REPLACE as c_int => {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_REPLACE as f32);
        }
        GL_DECAL as c_int => {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_DECAL as f32);
        }
        GL_ADD as c_int => {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_ADD as f32);
        }
        #[cfg(target_os = "windows")]
        GL_NONE as c_int => {
            qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_NONE as f32);
        }
        _ => {
            Com_Error(ERR_DROP, b"GL_TexEnv: invalid env '%d' passed\n\0" as *const u8 as *const i8, env);
        }
    }
}

/*
** GL_State
**
** This routine is responsible for setting the most commonly changed state
** in Q3.
*/
#[allow(non_snake_case)]
pub unsafe fn GL_State(stateBits: u32) {
    let diff: u32 = stateBits ^ (*glState).glStateBits;

    if diff == 0 {
        return;
    }

    //
    // check depthFunc bits
    //
    if (diff & GLS_DEPTHFUNC_EQUAL) != 0 {
        if (stateBits & GLS_DEPTHFUNC_EQUAL) != 0 {
            qglDepthFunc(GL_EQUAL);
        } else {
            qglDepthFunc(GL_LEQUAL);
        }
    }

    //
    // check blend bits
    //
    if (diff & (GLS_SRCBLEND_BITS | GLS_DSTBLEND_BITS)) != 0 {
        let srcFactor: u32;
        let dstFactor: u32;

        if (stateBits & (GLS_SRCBLEND_BITS | GLS_DSTBLEND_BITS)) != 0 {
            srcFactor = match stateBits & GLS_SRCBLEND_BITS {
                GLS_SRCBLEND_ZERO => GL_ZERO,
                GLS_SRCBLEND_ONE => GL_ONE,
                GLS_SRCBLEND_DST_COLOR => GL_DST_COLOR,
                GLS_SRCBLEND_ONE_MINUS_DST_COLOR => GL_ONE_MINUS_DST_COLOR,
                GLS_SRCBLEND_SRC_ALPHA => GL_SRC_ALPHA,
                GLS_SRCBLEND_ONE_MINUS_SRC_ALPHA => GL_ONE_MINUS_SRC_ALPHA,
                GLS_SRCBLEND_DST_ALPHA => GL_DST_ALPHA,
                GLS_SRCBLEND_ONE_MINUS_DST_ALPHA => GL_ONE_MINUS_DST_ALPHA,
                GLS_SRCBLEND_ALPHA_SATURATE => GL_SRC_ALPHA_SATURATE,
                _ => {
                    let srcFactor: u32 = GL_ONE; // to get warning to shut up
                    Com_Error(ERR_DROP, b"GL_State: invalid src blend state bits\n\0" as *const u8 as *const i8);
                    srcFactor
                }
            };

            dstFactor = match stateBits & GLS_DSTBLEND_BITS {
                GLS_DSTBLEND_ZERO => GL_ZERO,
                GLS_DSTBLEND_ONE => GL_ONE,
                GLS_DSTBLEND_SRC_COLOR => GL_SRC_COLOR,
                GLS_DSTBLEND_ONE_MINUS_SRC_COLOR => GL_ONE_MINUS_SRC_COLOR,
                GLS_DSTBLEND_SRC_ALPHA => GL_SRC_ALPHA,
                GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA => GL_ONE_MINUS_SRC_ALPHA,
                GLS_DSTBLEND_DST_ALPHA => GL_DST_ALPHA,
                GLS_DSTBLEND_ONE_MINUS_DST_ALPHA => GL_ONE_MINUS_DST_ALPHA,
                _ => {
                    let dstFactor: u32 = GL_ONE; // to get warning to shut up
                    Com_Error(ERR_DROP, b"GL_State: invalid dst blend state bits\n\0" as *const u8 as *const i8);
                    dstFactor
                }
            };

            qglEnable(GL_BLEND);
            qglBlendFunc(srcFactor, dstFactor);
        } else {
            qglDisable(GL_BLEND);
        }
    }

    //
    // check depthmask
    //
    if (diff & GLS_DEPTHMASK_TRUE) != 0 {
        if (stateBits & GLS_DEPTHMASK_TRUE) != 0 {
            qglDepthMask(GL_TRUE);
        } else {
            qglDepthMask(GL_FALSE);
        }
    }

    //
    // fill/line mode
    //
    if (diff & GLS_POLYMODE_LINE) != 0 {
        if (stateBits & GLS_POLYMODE_LINE) != 0 {
            qglPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
        } else {
            qglPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
        }
    }

    //
    // depthtest
    //
    if (diff & GLS_DEPTHTEST_DISABLE) != 0 {
        if (stateBits & GLS_DEPTHTEST_DISABLE) != 0 {
            qglDisable(GL_DEPTH_TEST);
        } else {
            qglEnable(GL_DEPTH_TEST);
        }
    }

    //
    // alpha test
    //
    if (diff & GLS_ATEST_BITS) != 0 {
        match stateBits & GLS_ATEST_BITS {
            0 => {
                qglDisable(GL_ALPHA_TEST);
            }
            GLS_ATEST_GT_0 => {
                qglEnable(GL_ALPHA_TEST);
                qglAlphaFunc(GL_GREATER, 0.0f32);
            }
            GLS_ATEST_LT_80 => {
                qglEnable(GL_ALPHA_TEST);
                qglAlphaFunc(GL_LESS, 0.5f32);
            }
            GLS_ATEST_GE_80 => {
                qglEnable(GL_ALPHA_TEST);
                qglAlphaFunc(GL_GEQUAL, 0.5f32);
            }
            GLS_ATEST_GE_C0 => {
                qglEnable(GL_ALPHA_TEST);
                qglAlphaFunc(GL_GEQUAL, 0.75f32);
            }
            _ => {
                debug_assert!(false);
            }
        }
    }

    (*glState).glStateBits = stateBits;
}

/*
================
RB_Hyperspace

A player has predicted a teleport, but hasn't arrived yet
================
*/
unsafe fn RB_Hyperspace() {
    let c: f32;

    if !(*backEnd).isHyperspace {
        // do initialization shit
    }

    c = ((((*backEnd).refdef).time & 255) as f32) / 255.0f32;
    qglClearColor(c, c, c, 1.0f32);
    qglClear(GL_COLOR_BUFFER_BIT);

    (*backEnd).isHyperspace = true;
}

#[allow(non_snake_case)]
unsafe fn SetViewportAndScissor() {
    qglMatrixMode(GL_PROJECTION);
    qglLoadMatrixf((*(*backEnd).viewParms).projectionMatrix as *const f32);
    qglMatrixMode(GL_MODELVIEW);

    // set the window clipping
    qglViewport(
        (*(*backEnd).viewParms).viewportX,
        (*(*backEnd).viewParms).viewportY,
        (*(*backEnd).viewParms).viewportWidth,
        (*(*backEnd).viewParms).viewportHeight,
    );
    qglScissor(
        (*(*backEnd).viewParms).viewportX,
        (*(*backEnd).viewParms).viewportY,
        (*(*backEnd).viewParms).viewportWidth,
        (*(*backEnd).viewParms).viewportHeight,
    );
}

/*
=================
RB_BeginDrawingView

Any mirrored or portaled views have already been drawn, so prepare
to actually render the visible surfaces for this view
=================
*/
unsafe fn RB_BeginDrawingView() {
    let mut clearBits: u32 = GL_DEPTH_BUFFER_BIT;

    // sync with gl if needed
    if (*r_finish).integer == 1 && !(*glState).finishCalled {
        qglFinish();
        (*glState).finishCalled = true;
    }
    if (*r_finish).integer == 0 {
        (*glState).finishCalled = true;
    }

    // we will need to change the projection matrix before drawing
    // 2D images again
    (*backEnd).projection2D = false;

    //
    // set the modelview matrix for the viewer
    //
    SetViewportAndScissor();

    // ensures that depth writes are enabled for the depth clear
    GL_State(GLS_DEFAULT);

    // clear relevant buffers
    if (*r_measureOverdraw).integer != 0 || (*r_shadows).integer == 2 || tr_stencilled {
        clearBits |= GL_STENCIL_BUFFER_BIT;
        tr_stencilled = false;
    }

    if skyboxportal {
        if ((*(*backEnd).refdef).rdflags & RDF_SKYBOXPORTAL) != 0 {
            // portal scene, clear whatever is necessary
            if (*r_fastsky).integer != 0 || ((*(*backEnd).refdef).rdflags & RDF_NOWORLDMODEL) != 0 {
                // fastsky: clear color
                // try clearing first with the portal sky fog color, then the world fog color, then finally a default
                clearBits |= GL_COLOR_BUFFER_BIT;
                if !(*tr).world.is_null() && (*(*tr).world).globalFog != -1 {
                    let fog: *const fog_t = &(*(*(*tr).world).fogs.as_ptr().add((*(*tr).world).globalFog as usize));
                    qglClearColor(
                        (*fog).parms.color[0],
                        (*fog).parms.color[1],
                        (*fog).parms.color[2],
                        1.0f32,
                    );
                } else {
                    qglClearColor(0.3f32, 0.3f32, 0.3f32, 1.0f32);
                }
            }
        }
    } else {
        if (*r_fastsky).integer != 0 && ((*(*backEnd).refdef).rdflags & RDF_NOWORLDMODEL) == 0 && !g_bRenderGlowingObjects {
            if !(*tr).world.is_null() && (*(*tr).world).globalFog != -1 {
                let fog: *const fog_t = &(*(*(*tr).world).fogs.as_ptr().add((*(*tr).world).globalFog as usize));
                qglClearColor(
                    (*fog).parms.color[0],
                    (*fog).parms.color[1],
                    (*fog).parms.color[2],
                    1.0f32,
                );
            } else {
                qglClearColor(0.3f32, 0.3f32, 0.3f32, 1.0f32); // FIXME: get color of sky
            }
            clearBits |= GL_COLOR_BUFFER_BIT; // FIXME: only if sky shaders have been used
        }
    }

    if ((*(*backEnd).refdef).rdflags & RDF_NOWORLDMODEL) == 0
        && (*r_DynamicGlow).integer != 0
        && !g_bRenderGlowingObjects
    {
        if !(*tr).world.is_null() && (*(*tr).world).globalFog != -1 {
            // this is because of a bug in multiple scenes I think, it needs to clear for the second scene but it doesn't normally.
            let fog: *const fog_t = &(*(*(*tr).world).fogs.as_ptr().add((*(*tr).world).globalFog as usize));

            qglClearColor(
                (*fog).parms.color[0],
                (*fog).parms.color[1],
                (*fog).parms.color[2],
                1.0f32,
            );
            clearBits |= GL_COLOR_BUFFER_BIT;
        }
    }
    // If this pass is to just render the glowing objects, don't clear the depth buffer since
    // we're sharing it with the main scene (since the main scene has already been rendered). -AReis
    if g_bRenderGlowingObjects {
        clearBits &= !GL_DEPTH_BUFFER_BIT;
    }

    if clearBits != 0 {
        qglClear(clearBits);
    }

    if ((*(*backEnd).refdef).rdflags & RDF_HYPERSPACE) != 0 {
        RB_Hyperspace();
        return;
    } else {
        (*backEnd).isHyperspace = false;
    }

    (*glState).faceCulling = -1; // force face culling to set next time

    // we will only draw a sun if there was sky rendered in this view
    (*backEnd).skyRenderedThisView = false;

    // clip to the plane of the portal
    if (*(*backEnd).viewParms).isPortal {
        let mut plane: [f32; 4] = [0.0; 4];
        let mut plane2: [f64; 4] = [0.0; 4];

        plane[0] = (*(*(*backEnd).viewParms).portalPlane).normal[0];
        plane[1] = (*(*(*backEnd).viewParms).portalPlane).normal[1];
        plane[2] = (*(*(*backEnd).viewParms).portalPlane).normal[2];
        plane[3] = (*(*(*backEnd).viewParms).portalPlane).dist;

        plane2[0] = DotProduct((*(*(*backEnd).viewParms).or_).axis[0] as *const f32, plane.as_ptr());
        plane2[1] = DotProduct((*(*(*backEnd).viewParms).or_).axis[1] as *const f32, plane.as_ptr());
        plane2[2] = DotProduct((*(*(*backEnd).viewParms).or_).axis[2] as *const f32, plane.as_ptr());
        plane2[3] = (DotProduct(plane.as_ptr(), (*(*(*backEnd).viewParms).or_).origin as *const f32) as f64) - (plane[3] as f64);

        qglLoadMatrixf(s_flipMatrix.as_ptr());
        qglClipPlane(GL_CLIP_PLANE0, plane2.as_ptr());
        qglEnable(GL_CLIP_PLANE0);
    } else {
        qglDisable(GL_CLIP_PLANE0);
    }
}

//used by RF_DISTORTION
#[inline]
unsafe fn R_WorldCoordToScreenCoordFloat(worldCoord: *const f32, x: *mut f32, y: *mut f32) -> bool {
    let xcenter: c_int;
    let ycenter: c_int;
    let mut local: [f32; 3] = [0.0; 3];
    let mut transformed: [f32; 3] = [0.0; 3];
    let mut vfwd: [f32; 3] = [0.0; 3];
    let mut vright: [f32; 3] = [0.0; 3];
    let mut vup: [f32; 3] = [0.0; 3];
    let xzi: f32;
    let yzi: f32;

    xcenter = (*glConfig).vidWidth / 2;
    ycenter = (*glConfig).vidHeight / 2;

    //AngleVectors (tr.refdef.viewangles, vfwd, vright, vup);
    VectorCopy((*(*tr).refdef).viewaxis[0] as *const f32, vfwd.as_mut_ptr());
    VectorCopy((*(*tr).refdef).viewaxis[1] as *const f32, vright.as_mut_ptr());
    VectorCopy((*(*tr).refdef).viewaxis[2] as *const f32, vup.as_mut_ptr());

    VectorSubtract(worldCoord, (*(*tr).refdef).vieworg as *const f32, local.as_mut_ptr());

    transformed[0] = DotProduct(local.as_ptr(), vright.as_ptr());
    transformed[1] = DotProduct(local.as_ptr(), vup.as_ptr());
    transformed[2] = DotProduct(local.as_ptr(), vfwd.as_ptr());

    // Make sure Z is not negative.
    if transformed[2] < 0.01 {
        return false;
    }

    xzi = (xcenter as f32) / transformed[2] * (90.0 / (*(*tr).refdef).fov_x);
    yzi = (ycenter as f32) / transformed[2] * (90.0 / (*(*tr).refdef).fov_y);

    *x = (xcenter as f32) + xzi * transformed[0];
    *y = (ycenter as f32) - yzi * transformed[1];

    true
}

//used by RF_DISTORTION
#[inline]
unsafe fn R_WorldCoordToScreenCoord(worldCoord: *const f32, x: *mut c_int, y: *mut c_int) -> bool {
    let mut xF: f32 = 0.0;
    let mut yF: f32 = 0.0;
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
//number of possible surfs we can postrender.
//note that postrenders lack much of the optimization that the standard sort-render crap does,
//so it's slower.

#[allow(non_snake_case)]
pub unsafe fn RB_RenderDrawSurfList(drawSurfs: *mut drawSurf_t, numDrawSurfs: c_int) {
    let shader: *mut shader_t;
    let oldShader: *mut shader_t;
    let fogNum: c_int;
    let oldFogNum: c_int;
    let entityNum: c_int;
    let oldEntityNum: c_int;
    let dlighted: c_int;
    let oldDlighted: c_int;
    let mut depthRange: c_int;
    let oldDepthRange: c_int;
    let mut i: c_int;
    let mut drawSurf: *mut drawSurf_t;
    let mut oldSort: u32;
    let originalTime: f32;
    let mut curEnt: *mut trRefEntity_t;
    let mut pRender: *mut postRender_t;
    let mut didShadowPass: bool = false;
    #[cfg(target_os = "macos")]
    let mut macEventTime: c_int;

    #[cfg(target_os = "macos")]
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
    originalTime = (*(*backEnd).refdef).floatTime;

    // clear the z buffer, set the modelview, etc
    RB_BeginDrawingView();

    // draw everything
    oldEntityNum = -1;
    (*backEnd).currentEntity = &mut (*tr).worldEntity;
    oldShader = 0 as *mut shader_t;
    oldFogNum = -1;
    oldDepthRange = 0;
    oldDlighted = 0;
    oldSort = ((!0u32)) as u32;
    depthRange = 0;

    (*(*backEnd).pc).c_surfaces += numDrawSurfs;

    i = 0;
    drawSurf = drawSurfs;
    while i < numDrawSurfs {
        if (*drawSurf).sort == oldSort {
            // fast path, same as previous sort
            ((*rb_surfaceTable[*(*drawSurf).surface as usize])((*drawSurf).surface));
            i += 1;
            drawSurf = drawSurf.add(1);
            continue;
        }
        R_DecomposeSort(
            (*drawSurf).sort,
            &mut (entityNum as *mut c_int) as *mut c_int,
            &mut shader,
            &mut (fogNum as *mut c_int) as *mut c_int,
            &mut (dlighted as *mut c_int) as *mut c_int,
        );

        #[cfg(not(target_os = "windows"))]
        {
            // If we're rendering glowing objects, but this shader has no stages with glow, skip it!
            if g_bRenderGlowingObjects && !(*shader).hasGlow {
                shader = oldShader;
                entityNum = oldEntityNum;
                fogNum = oldFogNum;
                dlighted = oldDlighted;
                i += 1;
                drawSurf = drawSurf.add(1);
                continue;
            }
        }
        oldSort = (*drawSurf).sort;

        //
        // change the tess parameters if needed
        // a "entityMergable" shader is a shader that can have surfaces from seperate
        // entities merged into a single batch, like smoke and blood puff sprites
        if entityNum != TR_WORLDENT && (g_numPostRenders as usize) < MAX_POST_RENDERS {
            if ((*(*(*backEnd).refdef).entities.add(entityNum as usize)).e).renderfx & RF_DISTORTION != 0
            {
                //must render last
                curEnt = &mut (*(*backEnd).refdef).entities.add(entityNum as usize);
                pRender = &mut g_postRenders[g_numPostRenders as usize];

                g_numPostRenders += 1;

                depthRange = 0;
                //figure this stuff out now and store it
                if ((*curEnt).e).renderfx & RF_NODEPTH != 0 {
                    depthRange = 2;
                } else if ((*curEnt).e).renderfx & RF_DEPTHHACK != 0 {
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

                oldSort = ((!0u32)) as u32; //invalidate this thing, cause we may want to postrender more surfs of the same sort

                //continue without bothering to begin a draw surf
                i += 1;
                drawSurf = drawSurf.add(1);
                continue;
            }
        }

        if shader != oldShader
            || fogNum != oldFogNum
            || dlighted != oldDlighted
            || (entityNum != oldEntityNum && !(*shader).entityMergable)
        {
            if oldShader != (0 as *mut shader_t) {
                #[cfg(target_os = "macos")]
                {
                    let mut t: c_int;

                    t = Sys_Milliseconds();
                    if t > macEventTime {
                        macEventTime = t + MAC_EVENT_PUMP_MSEC;
                        Sys_PumpEvents();
                    }
                }
                RB_EndSurface();

                if !didShadowPass && shader != (0 as *mut shader_t) && (*shader).sort > SS_BANNER {
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
            depthRange = 0;

            if entityNum != TR_WORLDENT {
                (*backEnd).currentEntity = &mut (*(*backEnd).refdef).entities.add(entityNum as usize);
                (*(*backEnd).refdef).floatTime = originalTime - ((*(*backEnd).currentEntity).e).shaderTime;

                // set up the transformation matrix
                R_RotateForEntity(
                    (*backEnd).currentEntity,
                    &mut (*(*backEnd).viewParms) as *mut viewParms_t,
                    &mut (*backEnd).ori,
                );

                // set up the dynamic lighting if needed
                if (*(*backEnd).currentEntity).needDlights {
                    #[cfg(feature = "VV_LIGHTING")]
                    {
                        VVLightMan.R_TransformDlights(&mut (*backEnd).ori);
                    }
                    #[cfg(not(feature = "VV_LIGHTING"))]
                    {
                        R_TransformDlights(
                            (*(*backEnd).refdef).num_dlights,
                            (*(*backEnd).refdef).dlights as *const dlight_t,
                            &(*backEnd).ori,
                        );
                    }
                }

                if ((*(*backEnd).currentEntity).e).renderfx & RF_NODEPTH != 0 {
                    // No depth at all, very rare but some things for seeing through walls
                    depthRange = 2;
                } else if ((*(*backEnd).currentEntity).e).renderfx & RF_DEPTHHACK != 0 {
                    // hack the depth range to prevent view model from poking into walls
                    depthRange = 1;
                }
            } else {
                (*backEnd).currentEntity = &mut (*tr).worldEntity;
                (*(*backEnd).refdef).floatTime = originalTime;
                (*backEnd).ori = (*(*backEnd).viewParms).world;
                #[cfg(feature = "VV_LIGHTING")]
                {
                    VVLightMan.R_TransformDlights(&mut (*backEnd).ori);
                }
                #[cfg(not(feature = "VV_LIGHTING"))]
                {
                    R_TransformDlights(
                        (*(*backEnd).refdef).num_dlights,
                        (*(*backEnd).refdef).dlights as *const dlight_t,
                        &(*backEnd).ori,
                    );
                }
            }

            qglLoadMatrixf((*(*backEnd).ori).modelMatrix as *const f32);

            //
            // change depthrange if needed
            //
            if oldDepthRange != depthRange {
                match depthRange {
                    1 => {
                        qglDepthRange(0.0, 0.3);
                    }
                    2 => {
                        qglDepthRange(0.0, 0.0);
                    }
                    _ => {
                        qglDepthRange(0.0, 1.0);
                    }
                }

                oldDepthRange = depthRange;
            }

            oldEntityNum = entityNum;
        }

        // add the triangles for this surface
        ((*rb_surfaceTable[*(*drawSurf).surface as usize])((*drawSurf).surface));

        i += 1;
        drawSurf = drawSurf.add(1);
    }

    // draw the contents of the last shader batch
    if oldShader != (0 as *mut shader_t) {
        RB_EndSurface();
    }

    if tr_stencilled && tr_distortionPrePost {
        //ok, cap it now
        RB_CaptureScreenImage();
        RB_DistortionFill();
    }

    //render distortion surfs (or anything else that needs to be post-rendered)
    if g_numPostRenders > 0 {
        let mut lastPostEnt: c_int = -1;

        while g_numPostRenders > 0 {
            g_numPostRenders -= 1;
            pRender = &mut g_postRenders[g_numPostRenders as usize];

            RB_BeginSurface((*pRender).shader, (*pRender).fogNum);

            (*backEnd).currentEntity = &mut (*(*backEnd).refdef).entities.add((*pRender).entNum as usize);

            (*(*backEnd).refdef).floatTime = originalTime - ((*(*backEnd).currentEntity).e).shaderTime;

            // set up the transformation matrix
            R_RotateForEntity(
                (*backEnd).currentEntity,
                &mut (*(*backEnd).viewParms) as *mut viewParms_t,
                &mut (*backEnd).ori,
            );

            // set up the dynamic lighting if needed
            if (*(*backEnd).currentEntity).needDlights {
                #[cfg(feature = "VV_LIGHTING")]
                {
                    VVLightMan.R_TransformDlights(&mut (*backEnd).ori);
                }
                #[cfg(not(feature = "VV_LIGHTING"))]
                {
                    R_TransformDlights(
                        (*(*backEnd).refdef).num_dlights,
                        (*(*backEnd).refdef).dlights as *const dlight_t,
                        &(*backEnd).ori,
                    );
                }
            }

            qglLoadMatrixf((*(*backEnd).ori).modelMatrix as *const f32);

            depthRange = (*pRender).depthRange;
            match depthRange {
                1 => {
                    qglDepthRange(0.0, 0.3);
                }
                2 => {
                    qglDepthRange(0.0, 0.0);
                }
                _ => {
                    qglDepthRange(0.0, 1.0);
                }
            }

            if ((*(*backEnd).currentEntity).e).renderfx & RF_DISTORTION != 0 && lastPostEnt != (*pRender).entNum {
                //do the capture now, we only need to do it once per ent
                let mut x: c_int;
                let mut y: c_int;
                let rad: c_int = ((*(*backEnd).currentEntity).e).radius;
                //We are going to just bind this, and then the CopyTexImage is going to
                //stomp over this texture num in texture memory.
                GL_Bind((*tr).screenImage as *mut image_t);

                if R_WorldCoordToScreenCoord(((*(*backEnd).currentEntity).e).origin as *const f32, &mut x, &mut y) {
                    let mut cX: c_int;
                    let mut cY: c_int;
                    cX = (*glConfig).vidWidth - x - (rad / 2);
                    cY = (*glConfig).vidHeight - y - (rad / 2);

                    if cX + rad > (*glConfig).vidWidth {
                        //would it go off screen?
                        cX = (*glConfig).vidWidth - rad;
                    } else if cX < 0 {
                        //cap it off at 0
                        cX = 0;
                    }

                    if cY + rad > (*glConfig).vidHeight {
                        //would it go off screen?
                        cY = (*glConfig).vidHeight - rad;
                    } else if cY < 0 {
                        //cap it off at 0
                        cY = 0;
                    }

                    //now copy a portion of the screen to this texture
                    #[cfg(target_os = "windows")]
                    {
                        qglCopyBackBufferToTexEXT(rad, rad, cX, 480 - cY, cX + rad, 480 - (cY + rad));
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        qglCopyTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA16, cX, cY, rad, rad, 0);
                    }

                    lastPostEnt = (*pRender).entNum;
                }
            }

            ((*rb_surfaceTable[*(*(*pRender).drawSurf).surface as usize])((*(*pRender).drawSurf).surface));
            RB_EndSurface();
        }
    }

    // go back to the world modelview matrix
    qglLoadMatrixf((*(*(*backEnd).viewParms).world).modelMatrix as *const f32);
    if depthRange != 0 {
        qglDepthRange(0.0, 1.0);
    }

    if tr_stencilled && !tr_distortionPrePost {
        //draw in the stencil buffer's cutout
        RB_DistortionFill();
    }
    if !didShadowPass {
        // darken down any stencil shadows
        RB_ShadowFinish();
        didShadowPass = true;
    }

    #[cfg(target_os = "windows")]
    {
        if (*r_hdreffect).integer != 0 {
            HDREffect.Render();
        }
    }

    // add light flares on lights that aren't obscured
    //	RB_RenderFlares();

    #[cfg(target_os = "macos")]
    {
        Sys_PumpEvents(); // crutch up the mac's limited buffer queue size
    }
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
#[allow(non_snake_case)]
pub unsafe fn RB_SetGL2D() {
    (*backEnd).projection2D = true;

    // set 2D virtual screen size
    qglViewport(0, 0, (*glConfig).vidWidth, (*glConfig).vidHeight);
    qglScissor(0, 0, (*glConfig).vidWidth, (*glConfig).vidHeight);
    qglMatrixMode(GL_PROJECTION);
    qglLoadIdentity();
    #[cfg(target_os = "windows")]
    {
        qglOrtho(0.0, 640.0, 0.0, 480.0, 0.0, 1.0);
    }
    #[cfg(not(target_os = "windows"))]
    {
        qglOrtho(0.0, 640.0, 480.0, 0.0, 0.0, 1.0);
    }
    qglMatrixMode(GL_MODELVIEW);
    qglLoadIdentity();

    GL_State(
        GLS_DEPTHTEST_DISABLE | GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA,
    );

    qglDisable(GL_CULL_FACE);
    qglDisable(GL_CLIP_PLANE0);

    // set time for 2D shaders
    (*(*backEnd).refdef).time = Sys_Milliseconds();
    (*(*backEnd).refdef).floatTime = ((*(*backEnd).refdef).time as f32) * 0.001;
}

/*
=============
RB_SetColor

=============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_SetColor(data: *const c_void) -> *const c_void {
    let cmd: *const setColorCommand_t;

    cmd = data as *const setColorCommand_t;

    (*(*backEnd).color2D)[0] = ((*cmd).color[0] * 255.0) as c_int;
    (*(*backEnd).color2D)[1] = ((*cmd).color[1] * 255.0) as c_int;
    (*(*backEnd).color2D)[2] = ((*cmd).color[2] * 255.0) as c_int;
    (*(*backEnd).color2D)[3] = ((*cmd).color[3] * 255.0) as c_int;

    (cmd.add(1)) as *const c_void
}

/*
=============
RB_StretchPic
=============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_StretchPic(data: *const c_void) -> *const c_void {
    let cmd: *const stretchPicCommand_t;
    let shader: *mut shader_t;
    let mut numVerts: c_int;
    let mut numIndexes: c_int;

    cmd = data as *const stretchPicCommand_t;

    shader = (*cmd).shader;
    if shader != (*tess).shader {
        if (*tess).numIndexes != 0 {
            RB_EndSurface(); //this might change culling and other states
        }
        (*backEnd).currentEntity = &mut (*backEnd).entity2D;
        RB_BeginSurface(shader, 0);
    }

    if !(*backEnd).projection2D {
        RB_SetGL2D(); //set culling and other states
    }

    RB_CHECKOVERFLOW(4, 6);
    numVerts = (*tess).numVertexes;
    numIndexes = (*tess).numIndexes;

    (*tess).numVertexes += 4;
    (*tess).numIndexes += 6;

    (*tess).indexes[numIndexes as usize] = (numVerts + 3) as u32;
    (*tess).indexes[(numIndexes + 1) as usize] = (numVerts + 0) as u32;
    (*tess).indexes[(numIndexes + 2) as usize] = (numVerts + 2) as u32;
    (*tess).indexes[(numIndexes + 3) as usize] = (numVerts + 2) as u32;
    (*tess).indexes[(numIndexes + 4) as usize] = (numVerts + 0) as u32;
    (*tess).indexes[(numIndexes + 5) as usize] = (numVerts + 1) as u32;

    *((*tess).vertexColors[numVerts as usize] as *mut i32) =
        *((*tess).vertexColors[(numVerts + 1) as usize] as *mut i32) =
            *((*tess).vertexColors[(numVerts + 2) as usize] as *mut i32) =
                *((*tess).vertexColors[(numVerts + 3) as usize] as *mut i32) =
                    *((*backEnd).color2D as *mut i32);

    (*tess).xyz[numVerts as usize][0] = (*cmd).x;
    (*tess).xyz[numVerts as usize][1] = (*cmd).y;
    (*tess).xyz[numVerts as usize][2] = 0.0;

    (*tess).texCoords[numVerts as usize][0][0] = (*cmd).s1;
    (*tess).texCoords[numVerts as usize][0][1] = (*cmd).t1;

    (*tess).xyz[(numVerts + 1) as usize][0] = (*cmd).x + (*cmd).w;
    (*tess).xyz[(numVerts + 1) as usize][1] = (*cmd).y;
    (*tess).xyz[(numVerts + 1) as usize][2] = 0.0;

    (*tess).texCoords[(numVerts + 1) as usize][0][0] = (*cmd).s2;
    (*tess).texCoords[(numVerts + 1) as usize][0][1] = (*cmd).t1;

    (*tess).xyz[(numVerts + 2) as usize][0] = (*cmd).x + (*cmd).w;
    (*tess).xyz[(numVerts + 2) as usize][1] = (*cmd).y + (*cmd).h;
    (*tess).xyz[(numVerts + 2) as usize][2] = 0.0;

    (*tess).texCoords[(numVerts + 2) as usize][0][0] = (*cmd).s2;
    (*tess).texCoords[(numVerts + 2) as usize][0][1] = (*cmd).t2;

    (*tess).xyz[(numVerts + 3) as usize][0] = (*cmd).x;
    (*tess).xyz[(numVerts + 3) as usize][1] = (*cmd).y + (*cmd).h;
    (*tess).xyz[(numVerts + 3) as usize][2] = 0.0;

    (*tess).texCoords[(numVerts + 3) as usize][0][0] = (*cmd).s1;
    (*tess).texCoords[(numVerts + 3) as usize][0][1] = (*cmd).t2;

    (cmd.add(1)) as *const c_void
}

/*
=============
RB_DrawRotatePic
=============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_RotatePic(data: *const c_void) -> *const c_void {
    let cmd: *const rotatePicCommand_t;
    let image: *mut image_t;
    let shader: *mut shader_t;

    cmd = data as *const rotatePicCommand_t;

    shader = (*cmd).shader;
    image = &mut (*(*(*(*shader).stages.as_mut_ptr()).bundle.as_mut_ptr()).image.as_mut_ptr());

    if !image.is_null() {
        if !(*backEnd).projection2D {
            RB_SetGL2D();
        }

        qglColor4ubv((*backEnd).color2D as *const u8);
        qglPushMatrix();

        qglTranslatef((*cmd).x + (*cmd).w, (*cmd).y, 0.0);
        qglRotatef((*cmd).a, 0.0, 0.0, 1.0);

        GL_Bind(image);
        #[cfg(target_os = "windows")]
        {
            qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
        }
        #[cfg(not(target_os = "windows"))]
        {
            qglBegin(GL_QUADS);
        }
        qglTexCoord2f((*cmd).s1, (*cmd).t1);
        qglVertex2f(-(*cmd).w, 0.0);
        qglTexCoord2f((*cmd).s2, (*cmd).t1);
        qglVertex2f(0.0, 0.0);
        qglTexCoord2f((*cmd).s2, (*cmd).t2);
        qglVertex2f(0.0, (*cmd).h);
        qglTexCoord2f((*cmd).s1, (*cmd).t2);
        qglVertex2f(-(*cmd).w, (*cmd).h);
        qglEnd();

        qglPopMatrix();
    }

    (cmd.add(1)) as *const c_void
}

/*
=============
RB_DrawRotatePic2
=============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_RotatePic2(data: *const c_void) -> *const c_void {
    let cmd: *const rotatePicCommand_t;
    let image: *mut image_t;
    let shader: *mut shader_t;

    cmd = data as *const rotatePicCommand_t;

    shader = (*cmd).shader;

    if (*shader).numUnfoggedPasses != 0 {
        image = &mut (*(*(*(*shader).stages.as_mut_ptr()).bundle.as_mut_ptr()).image.as_mut_ptr());

        if !image.is_null() {
            if !(*backEnd).projection2D {
                RB_SetGL2D();
            }

            // Get our current blend mode, etc.
            GL_State((*(*(*shader).stages.as_ptr()).stateBits));

            qglColor4ubv((*backEnd).color2D as *const u8);
            qglPushMatrix();

            // rotation point is going to be around the center of the passed in coordinates
            qglTranslatef((*cmd).x, (*cmd).y, 0.0);
            qglRotatef((*cmd).a, 0.0, 0.0, 1.0);

            GL_Bind(image);
            #[cfg(target_os = "windows")]
            {
                qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
            }
            #[cfg(not(target_os = "windows"))]
            {
                qglBegin(GL_QUADS);
            }
            qglTexCoord2f((*cmd).s1, (*cmd).t1);
            qglVertex2f(-(*cmd).w * 0.5, -(*cmd).h * 0.5);

            qglTexCoord2f((*cmd).s2, (*cmd).t1);
            qglVertex2f((*cmd).w * 0.5, -(*cmd).h * 0.5);

            qglTexCoord2f((*cmd).s2, (*cmd).t2);
            qglVertex2f((*cmd).w * 0.5, (*cmd).h * 0.5);

            qglTexCoord2f((*cmd).s1, (*cmd).t2);
            qglVertex2f(-(*cmd).w * 0.5, (*cmd).h * 0.5);
            qglEnd();

            qglPopMatrix();

            // Hmmm, this is not too cool
            GL_State(
                GLS_DEPTHTEST_DISABLE | GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA,
            );
        }
    }

    (cmd.add(1)) as *const c_void
}

/*
=============
RB_LAGoggles
=============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_LAGoggles(data: *const c_void) -> *const c_void {
    data
}

/*
=============
RB_ScissorPic
=============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_Scissor(data: *const c_void) -> *const c_void {
    let cmd: *const scissorCommand_t;

    cmd = data as *const scissorCommand_t;

    if !(*backEnd).projection2D {
        RB_SetGL2D();
    }

    if (*cmd).x >= 0 {
        qglScissor(
            (*cmd).x,
            (*glConfig).vidHeight - (*cmd).y - (*cmd).h,
            (*cmd).w,
            (*cmd).h,
        );
    } else {
        qglScissor(0, 0, (*glConfig).vidWidth, (*glConfig).vidHeight);
    }

    (cmd.add(1)) as *const c_void
}

/*
=============
RB_DrawSurfs

=============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_DrawSurfs(data: *const c_void) -> *const c_void {
    let cmd: *const drawSurfsCommand_t;

    // finish any 2D drawing if needed
    if (*tess).numIndexes != 0 {
        RB_EndSurface();
    }

    cmd = data as *const drawSurfsCommand_t;

    (*backEnd).refdef = (*cmd).refdef;
    (*backEnd).viewParms = (*cmd).viewParms;

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
    #[cfg(not(target_os = "windows"))]
    {
        if ((*(*backEnd).refdef).rdflags & RDF_NOWORLDMODEL) == 0
            && g_bDynamicGlowSupported
            && (*r_DynamicGlow).integer != 0
        {
            // Copy the normal scene to texture.
            qglDisable(GL_TEXTURE_2D);
            qglEnable(GL_TEXTURE_RECTANGLE_EXT);
            qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, (*tr).sceneImage);
            qglCopyTexSubImage2D(
                GL_TEXTURE_RECTANGLE_EXT,
                0,
                0,
                0,
                (*(*backEnd).viewParms).viewportX,
                (*(*backEnd).viewParms).viewportY,
                (*(*backEnd).viewParms).viewportWidth,
                (*(*backEnd).viewParms).viewportHeight,
            );
            qglDisable(GL_TEXTURE_RECTANGLE_EXT);
            qglEnable(GL_TEXTURE_2D);

            // Just clear colors, but leave the depth buffer intact so we can 'share' it.
            qglClearColor(0.0, 0.0, 0.0, 0.0);
            qglClear(GL_COLOR_BUFFER_BIT);

            // Render the glowing objects.
            g_bRenderGlowingObjects = true;
            RB_RenderDrawSurfList((*cmd).drawSurfs, (*cmd).numDrawSurfs);
            g_bRenderGlowingObjects = false;
            qglFinish();

            // Copy the glow scene to texture.
            qglDisable(GL_TEXTURE_2D);
            qglEnable(GL_TEXTURE_RECTANGLE_EXT);
            qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, (*tr).screenGlow);
            qglCopyTexSubImage2D(
                GL_TEXTURE_RECTANGLE_EXT,
                0,
                0,
                0,
                (*(*backEnd).viewParms).viewportX,
                (*(*backEnd).viewParms).viewportY,
                (*(*backEnd).viewParms).viewportWidth,
                (*(*backEnd).viewParms).viewportHeight,
            );
            qglDisable(GL_TEXTURE_RECTANGLE_EXT);
            qglEnable(GL_TEXTURE_2D);

            // Resize the viewport to the blur texture size.
            let oldViewWidth: c_int = (*(*backEnd).viewParms).viewportWidth;
            let oldViewHeight: c_int = (*(*backEnd).viewParms).viewportHeight;
            (*(*backEnd).viewParms).viewportWidth = (*r_DynamicGlowWidth).integer;
            (*(*backEnd).viewParms).viewportHeight = (*r_DynamicGlowHeight).integer;
            SetViewportAndScissor();

            // Blur the scene.
            RB_BlurGlowTexture();

            // Copy the finished glow scene back to texture.
            qglDisable(GL_TEXTURE_2D);
            qglEnable(GL_TEXTURE_RECTANGLE_EXT);
            qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, (*tr).blurImage);
            qglCopyTexSubImage2D(
                GL_TEXTURE_RECTANGLE_EXT,
                0,
                0,
                0,
                0,
                0,
                (*(*backEnd).viewParms).viewportWidth,
                (*(*backEnd).viewParms).viewportHeight,
            );
            qglDisable(GL_TEXTURE_RECTANGLE_EXT);
            qglEnable(GL_TEXTURE_2D);

            // Set the viewport back to normal.
            (*(*backEnd).viewParms).viewportWidth = oldViewWidth;
            (*(*backEnd).viewParms).viewportHeight = oldViewHeight;
            SetViewportAndScissor();
            qglClear(GL_COLOR_BUFFER_BIT);

            // Draw the glow additively over the screen.
            RB_DrawGlowOverlay();
        }
    }

    (cmd.add(1)) as *const c_void
}

/*
=============
RB_DrawBuffer

=============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_DrawBuffer(data: *const c_void) -> *const c_void {
    let cmd: *const drawBufferCommand_t;

    cmd = data as *const drawBufferCommand_t;

    qglDrawBuffer((*cmd).buffer);

    // clear screen for debugging
    if ((*(*backEnd).refdef).rdflags & RDF_NOWORLDMODEL) == 0
        && !(*tr).world.is_null()
        && ((*tr).refdef.rdflags & RDF_doLAGoggles) != 0
    {
        let fog: *const fog_t = &(*(*(*tr).world).fogs.as_ptr().add((*(*tr).world).numfogs as usize));

        qglClearColor(
            (*fog).parms.color[0],
            (*fog).parms.color[1],
            (*fog).parms.color[2],
            1.0,
        );
        qglClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    } else if ((*(*backEnd).refdef).rdflags & RDF_NOWORLDMODEL) == 0
        && !(*tr).world.is_null()
        && (*(*tr).world).globalFog != -1
        && (*tr).sceneCount != 0
    {
        // don't clear during menus, wait for real scene
        let fog: *const fog_t = &(*(*(*tr).world).fogs.as_ptr().add((*(*tr).world).globalFog as usize));

        qglClearColor(
            (*fog).parms.color[0],
            (*fog).parms.color[1],
            (*fog).parms.color[2],
            1.0,
        );
        qglClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    } else if (*r_clear).integer != 0 {
        // clear screen for debugging
        let mut i: c_int = (*r_clear).integer;
        if i == 42 {
            i = Q_irand(0, 8);
        }
        match i {
            1 => {
                qglClearColor(1.0, 0.0, 0.0, 1.0); //red
            }
            2 => {
                qglClearColor(0.0, 1.0, 0.0, 1.0); //green
            }
            3 => {
                qglClearColor(1.0, 1.0, 0.0, 1.0); //yellow
            }
            4 => {
                qglClearColor(0.0, 0.0, 1.0, 1.0); //blue
            }
            5 => {
                qglClearColor(0.0, 1.0, 1.0, 1.0); //cyan
            }
            6 => {
                qglClearColor(1.0, 0.0, 1.0, 1.0); //magenta
            }
            7 => {
                qglClearColor(1.0, 1.0, 1.0, 1.0); //white
            }
            8 => {
                qglClearColor(0.0, 0.0, 0.0, 1.0); //black
            }
            _ => {
                qglClearColor(1.0, 0.0, 0.5, 1.0);
            }
        }
        qglClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }

    (cmd.add(1)) as *const c_void
}

/*
===============
RB_ShowImages

Draw all the images to the screen, on top of whatever
was there.  This is used to test for texture thrashing.

Also called by RE_EndRegistration
===============
*/
#[allow(non_snake_case)]
pub unsafe fn RB_ShowImages() {
    let image: *mut image_t;
    let mut x: f32;
    let mut y: f32;
    let mut w: f32;
    let mut h: f32;
    let start: c_int;
    let end: c_int;

    if !(*backEnd).projection2D {
        RB_SetGL2D();
    }

    qglFinish();

    start = Sys_Milliseconds();

    let mut i: c_int = 0;
    //	int iNumImages =
    R_Images_StartIteration();
    while !((image = R_Images_GetNextIteration()).is_null()) {
        w = ((*glConfig).vidWidth as f32) / 20.0;
        h = ((*glConfig).vidHeight as f32) / 15.0;
        x = ((i % 20) as f32) * w;
        y = ((i / 20) as f32) * h;

        // show in proportional size in mode 2
        if (*r_showImages).integer == 2 {
            w *= ((*image).width as f32) / 512.0;
            h *= ((*image).height as f32) / 512.0;
        }

        GL_Bind(image);
        #[cfg(target_os = "windows")]
        {
            qglBeginEXT(GL_QUADS, 4, 0, 0, 4, 0);
        }
        #[cfg(not(target_os = "windows"))]
        {
            qglBegin(GL_QUADS);
        }
        qglTexCoord2f(0.0, 0.0);
        qglVertex2f(x, y);
        qglTexCoord2f(1.0, 0.0);
        qglVertex2f(x + w, y);
        qglTexCoord2f(1.0, 1.0);
        qglVertex2f(x + w, y + h);
        qglTexCoord2f(0.0, 1.0);
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
extern "C" {
    fn RB_RenderWorldEffects();
}

#[allow(non_snake_case)]
pub unsafe fn RB_SwapBuffers(data: *const c_void) -> *const c_void {
    let cmd: *const swapBuffersCommand_t;

    // finish any 2D drawing if needed
    if (*tess).numIndexes != 0 {
        RB_EndSurface();
    }

    // texture swapping test
    if (*r_showImages).integer != 0 {
        RB_ShowImages();
    }

    cmd = data as *const swapBuffersCommand_t;

    // we measure overdraw by reading back the stencil buffer and
    // counting up the number of increments that have happened
    #[cfg(not(target_os = "windows"))]
    {
        if (*r_measureOverdraw).integer != 0 {
            let mut i: c_int;
            let mut sum: i64 = 0;
            let stencilReadback: *mut u8;

            stencilReadback = Z_Malloc(
                ((*glConfig).vidWidth * (*glConfig).vidHeight) as usize,
                TAG_TEMP_WORKSPACE,
                false,
            ) as *mut u8;
            qglReadPixels(
                0,
                0,
                (*glConfig).vidWidth,
                (*glConfig).vidHeight,
                GL_STENCIL_INDEX,
                GL_UNSIGNED_BYTE,
                stencilReadback as *mut c_void,
            );

            i = 0;
            while i < (*glConfig).vidWidth * (*glConfig).vidHeight {
                sum += *stencilReadback.add(i as usize) as i64;
                i += 1;
            }

            (*(*backEnd).pc).c_overDraw += sum;
            Z_Free(stencilReadback as *mut c_void);
        }
    }

    if !(*glState).finishCalled {
        qglFinish();
    }

    GLimp_LogComment(b"***************** RB_SwapBuffers *****************\n\n\n" as *const u8 as *const i8);

    GLimp_EndFrame();

    (*backEnd).projection2D = false;

    (cmd.add(1)) as *const c_void
}

#[allow(non_snake_case)]
pub unsafe fn RB_WorldEffects(data: *const c_void) -> *const c_void {
    let cmd: *const setModeCommand_t;

    cmd = data as *const setModeCommand_t;

    // Always flush the tess buffer
    if !(*tess).shader.is_null() && (*tess).numIndexes != 0 {
        RB_EndSurface();
    }
    RB_RenderWorldEffects();

    if !(*tess).shader.is_null() {
        RB_BeginSurface((*tess).shader, (*tess).fogNum);
    }

    (cmd.add(1)) as *const c_void
}

/*
====================
RB_ExecuteRenderCommands

This function will be called syncronously if running without
smp extensions, or asyncronously by another thread.
====================
*/
#[allow(non_snake_case)]
pub unsafe fn RB_ExecuteRenderCommands(mut data: *const c_void) {
    let t1: c_int;
    let t2: c_int;

    t1 = Sys_Milliseconds();

    loop {
        match *(data as *const u32) {
            RC_SET_COLOR => {
                data = RB_SetColor(data);
            }
            RC_STRETCH_PIC => {
                data = RB_StretchPic(data);
            }
            RC_ROTATE_PIC => {
                data = RB_RotatePic(data);
            }
            RC_ROTATE_PIC2 => {
                data = RB_RotatePic2(data);
            }
            RC_SCISSOR => {
                data = RB_Scissor(data);
            }
            RC_DRAW_SURFS => {
                data = RB_DrawSurfs(data);
            }
            RC_DRAW_BUFFER => {
                data = RB_DrawBuffer(data);
            }
            RC_SWAP_BUFFERS => {
                data = RB_SwapBuffers(data);
            }
            RC_WORLD_EFFECTS => {
                data = RB_WorldEffects(data);
            }
            RC_END_OF_LIST | _ => {
                // stop rendering on this thread
                t2 = Sys_Milliseconds();
                (*(*backEnd).pc).msec = t2 - t1;
                return;
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
// What Pixel Shader type is currently active (regcoms or fragment programs).
unsafe fn BeginPixelShader(uiType: u32, uiID: u32) {
    match uiType {
        // Using Register Combiners, so call the Display List that stores it.
        GL_REGISTER_COMBINERS_NV => {
            // Just in case...
            if qglCombinerParameterfvNV.is_null() {
                return;
            }

            // Call the list with the regcom in it.
            qglEnable(GL_REGISTER_COMBINERS_NV);
            qglCallList(uiID);

            g_uiCurrentPixelShaderType = GL_REGISTER_COMBINERS_NV;
        }

        // Using Fragment Programs, so call the program.
        GL_FRAGMENT_PROGRAM_ARB => {
            // Just in case...
            if qglGenProgramsARB.is_null() {
                return;
            }

            qglEnable(GL_FRAGMENT_PROGRAM_ARB);
            qglBindProgramARB(GL_FRAGMENT_PROGRAM_ARB, uiID);

            g_uiCurrentPixelShaderType = GL_FRAGMENT_PROGRAM_ARB;
        }
        _ => {}
    }
}

// Stop using a Pixel Shader and return states to normal.
#[cfg(not(target_os = "windows"))]
unsafe fn EndPixelShader() {
    if g_uiCurrentPixelShaderType == 0x0 {
        return;
    }

    qglDisable(g_uiCurrentPixelShaderType);
}

// Hack variable for deciding which kind of texture rectangle thing to do (for some
// reason it acts different on radeon! It's against the spec!).

#[cfg(not(target_os = "windows"))]
#[inline]
unsafe fn RB_BlurGlowTexture() {
    qglDisable(GL_CLIP_PLANE0);
    GL_Cull(CT_TWO_SIDED);

    // Go into orthographic 2d mode.
    qglMatrixMode(GL_PROJECTION);
    qglPushMatrix();
    qglLoadIdentity();
    qglOrtho(
        0.0,
        (*(*backEnd).viewParms).viewportWidth as f64,
        (*(*backEnd).viewParms).viewportHeight as f64,
        0.0,
        -1.0,
        1.0,
    );
    qglMatrixMode(GL_MODELVIEW);
    qglPushMatrix();
    qglLoadIdentity();

    GL_State(GLS_DEPTHTEST_DISABLE);

    /////////////////////////////////////////////////////////
    // Setup vertex and pixel programs.
    /////////////////////////////////////////////////////////

    // NOTE: The 0.25 is because we're blending 4 textures (so = 1.0) and we want a relatively normalized pixel
    // intensity distribution, but this won't happen anyways if intensity is higher than 1.0.
    let fBlurDistribution: f32 = (*r_DynamicGlowIntensity).value * 0.25;
    let mut fBlurWeight: [f32; 4] = [fBlurDistribution, fBlurDistribution, fBlurDistribution, 1.0];

    // Enable and set the Vertex Program.
    qglEnable(GL_VERTEX_PROGRAM_ARB);
    qglBindProgramARB(GL_VERTEX_PROGRAM_ARB, (*tr).glowVShader);

    // Apply Pixel Shaders.
    if !qglCombinerParameterfvNV.is_null() {
        BeginPixelShader(GL_REGISTER_COMBINERS_NV, (*tr).glowPShader);

        // Pass the blur weight to the regcom.
        qglCombinerParameterfvNV(GL_CONSTANT_COLOR0_NV, fBlurWeight.as_mut_ptr());
    } else if !qglProgramEnvParameter4fARB.is_null() {
        BeginPixelShader(GL_FRAGMENT_PROGRAM_ARB, (*tr).glowPShader);

        // Pass the blur weight to the Fragment Program.
        qglProgramEnvParameter4fARB(
            GL_FRAGMENT_PROGRAM_ARB,
            0,
            fBlurWeight[0],
            fBlurWeight[1],
            fBlurWeight[2],
            fBlurWeight[3],
        );
    }

    /////////////////////////////////////////////////////////
    // Set the blur texture to the 4 texture stages.
    /////////////////////////////////////////////////////////

    // How much to offset each texel by.
    let fTexelWidthOffset: f32 = 0.1;
    let fTexelHeightOffset: f32 = 0.1;

    let uiTex: u32 = (*tr).screenGlow;

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
    let mut iTexWidth: c_int = (*glConfig).vidWidth;
    let mut iTexHeight: c_int = (*glConfig).vidHeight;

    let mut iNumBlurPasses: c_int = 0;
    let mut fTexelWidthOffset: f32 = fTexelWidthOffset;
    let mut fTexelHeightOffset: f32 = fTexelHeightOffset;

    while iNumBlurPasses < (*r_DynamicGlowPasses).integer {
        // Load the Texel Offsets into the Vertex Program.
        qglProgramEnvParameter4fARB(GL_VERTEX_PROGRAM_ARB, 0, -fTexelWidthOffset, -fTexelWidthOffset, 0.0, 0.0);
        qglProgramEnvParameter4fARB(GL_VERTEX_PROGRAM_ARB, 1, -fTexelWidthOffset, fTexelWidthOffset, 0.0, 0.0);
        qglProgramEnvParameter4fARB(GL_VERTEX_PROGRAM_ARB, 2, fTexelWidthOffset, -fTexelWidthOffset, 0.0, 0.0);
        qglProgramEnvParameter4fARB(GL_VERTEX_PROGRAM_ARB, 3, fTexelWidthOffset, fTexelWidthOffset, 0.0, 0.0);

        // After first pass put the tex coords to the viewport size.
        let mut uiTex: u32 = uiTex;
        if iNumBlurPasses == 1 {
            if !g_bTextureRectangleHack {
                iTexWidth = (*(*backEnd).viewParms).viewportWidth;
                iTexHeight = (*(*backEnd).viewParms).viewportHeight;
            }

            uiTex = (*tr).blurImage;
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
            qglCopyTexSubImage2D(
                GL_TEXTURE_RECTANGLE_EXT,
                0,
                0,
                0,
                0,
                0,
                (*(*backEnd).viewParms).viewportWidth,
                (*(*backEnd).viewParms).viewportHeight,
            );
        }

        // Draw the fullscreen quad.
        qglBegin(GL_QUADS);
        qglMultiTexCoord2fARB(GL_TEXTURE0_ARB, 0.0, iTexHeight as f32);
        qglVertex2f(0.0, 0.0);

        qglMultiTexCoord2fARB(GL_TEXTURE0_ARB, 0.0, 0.0);
        qglVertex2f(0.0, (*(*backEnd).viewParms).viewportHeight as f32);

        qglMultiTexCoord2fARB(GL_TEXTURE0_ARB, iTexWidth as f32, 0.0);
        qglVertex2f((*(*backEnd).viewParms).viewportWidth as f32, (*(*backEnd).viewParms).viewportHeight as f32);

        qglMultiTexCoord2fARB(GL_TEXTURE0_ARB, iTexWidth as f32, iTexHeight as f32);
        qglVertex2f((*(*backEnd).viewParms).viewportWidth as f32, 0.0);
        qglEnd();

        qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, (*tr).blurImage);
        qglCopyTexSubImage2D(
            GL_TEXTURE_RECTANGLE_EXT,
            0,
            0,
            0,
            0,
            0,
            (*(*backEnd).viewParms).viewportWidth,
            (*(*backEnd).viewParms).viewportHeight,
        );

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
    (*glState).currenttmu = 0; //this matches the last one we activated
}

// Draw the glow blur over the screen additively.
#[cfg(not(target_os = "windows"))]
#[inline]
unsafe fn RB_DrawGlowOverlay() {
    qglDisable(GL_CLIP_PLANE0);
    GL_Cull(CT_TWO_SIDED);

    // Go into orthographic 2d mode.
    qglMatrixMode(GL_PROJECTION);
    qglPushMatrix();
    qglLoadIdentity();
    qglOrtho(0.0, (*glConfig).vidWidth as f64, (*glConfig).vidHeight as f64, 0.0, -1.0, 1.0);
    qglMatrixMode(GL_MODELVIEW);
    qglPushMatrix();
    qglLoadIdentity();

    GL_State(GLS_DEPTHTEST_DISABLE);

    qglDisable(GL_TEXTURE_2D);
    qglEnable(GL_TEXTURE_RECTANGLE_EXT);

    // For debug purposes.
    if (*r_DynamicGlow).integer != 2 {
        // Render the normal scene texture.
        qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, (*tr).sceneImage);
        qglBegin(GL_QUADS);
        qglColor4f(1.0, 1.0, 1.0, 1.0);
        qglTexCoord2f(0.0, (*glConfig).vidHeight as f32);
        qglVertex2f(0.0, 0.0);

        qglTexCoord2f(0.0, 0.0);
        qglVertex2f(0.0, (*glConfig).vidHeight as f32);

        qglTexCoord2f((*glConfig).vidWidth as f32, 0.0);
        qglVertex2f((*glConfig).vidWidth as f32, (*glConfig).vidHeight as f32);

        qglTexCoord2f((*glConfig).vidWidth as f32, (*glConfig).vidHeight as f32);
        qglVertex2f((*glConfig).vidWidth as f32, 0.0);
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
    qglBindTexture(GL_TEXTURE_RECTANGLE_EXT, (*tr).blurImage);
    qglBegin(GL_QUADS);
    qglColor4f(1.0, 1.0, 1.0, 1.0);
    qglTexCoord2f(0.0, (*r_DynamicGlowHeight).integer as f32);
    qglVertex2f(0.0, 0.0);

    qglTexCoord2f(0.0, 0.0);
    qglVertex2f(0.0, (*glConfig).vidHeight as f32);

    qglTexCoord2f((*r_DynamicGlowWidth).integer as f32, 0.0);
    qglVertex2f((*glConfig).vidWidth as f32, (*glConfig).vidHeight as f32);

    qglTexCoord2f((*r_DynamicGlowWidth).integer as f32, (*r_DynamicGlowHeight).integer as f32);
    qglVertex2f((*glConfig).vidWidth as f32, 0.0);
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

// STUBS FOR MISSING TYPES AND FUNCTIONS THAT REQUIRE EXTERNAL CONTEXT
extern "C" {
    static mut rb_surfaceTable: [unsafe extern "C" fn(*mut c_void); 256];
    static mut r_nobind: *mut cvar_t;
    static mut r_finish: *mut cvar_t;
    static mut r_fastsky: *mut cvar_t;
    static mut r_clear: *mut cvar_t;
    static mut r_measureOverdraw: *mut cvar_t;
    static mut r_shadows: *mut cvar_t;
    static mut r_DynamicGlow: *mut cvar_t;
    static mut r_DynamicGlowWidth: *mut cvar_t;
    static mut r_DynamicGlowHeight: *mut cvar_t;
    static mut r_DynamicGlowIntensity: *mut cvar_t;
    static mut r_DynamicGlowPasses: *mut cvar_t;
    static mut r_DynamicGlowDelta: *mut cvar_t;
    static mut r_DynamicGlowSoft: *mut cvar_t;
    static mut r_showImages: *mut cvar_t;
    static mut r_hdreffect: *mut cvar_t;

    pub struct cvar_t {
        _unused: [u8; 0],
    }

    static SS_BANNER: c_int;
    static HDREffect: c_int; // stub for HDREffect object
}

fn RB_CHECKOVERFLOW(verts: c_int, indexes: c_int) {
    // Stub for overflow check - implementation would depend on actual tessellation state
}
