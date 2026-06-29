//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// tr_init.c -- functions that are not called every frame

// #include "tr_local.h"

// #ifndef DEDICATED
// #if !defined __TR_WORLDEFFECTS_H
// #include "tr_WorldEffects.h"
// #endif
// #endif //!DEDICATED

// #include "tr_font.h"

// #if !defined (MINIHEAP_H_INC)
// #include "../qcommon/MiniHeap.h"

// #include "../ghoul2/G2_local.h"
// #endif


//#ifdef __USEA3D
//// Defined in snd_a3dg_refcommon.c
//void RE_A3D_RenderGeometry (void *pVoidA3D, void *pVoidGeom, void *pVoidMat, void *pVoidGeomStatus);
//#endif

use core::ffi::{c_char, c_int, c_void};

#[allow(non_snake_case)]

#[cfg(not(feature = "dedicated"))]
pub static mut G2VertSpaceServer: *mut c_void = core::ptr::null_mut();

#[cfg(not(feature = "dedicated"))]
pub static mut CMiniHeap_singleton: c_void = unsafe { core::mem::zeroed() };

const G2_VERT_SPACE_SERVER_SIZE: c_int = 256;

#[cfg(not(feature = "dedicated"))]
pub static mut glConfig: glconfig_t = unsafe { core::mem::zeroed() };
#[cfg(not(feature = "dedicated"))]
pub static mut glState: glstate_t = unsafe { core::mem::zeroed() };

#[cfg(not(feature = "dedicated"))]
extern "C" {
    fn GfxInfo_f();
}

pub static mut r_verbose: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ignore: *mut cvar_t = core::ptr::null_mut();

pub static mut r_displayRefresh: *mut cvar_t = core::ptr::null_mut();

pub static mut r_detailTextures: *mut cvar_t = core::ptr::null_mut();

pub static mut r_znear: *mut cvar_t = core::ptr::null_mut();

pub static mut r_skipBackEnd: *mut cvar_t = core::ptr::null_mut();

pub static mut r_ignorehwgamma: *mut cvar_t = core::ptr::null_mut();
pub static mut r_measureOverdraw: *mut cvar_t = core::ptr::null_mut();

pub static mut r_inGameVideo: *mut cvar_t = core::ptr::null_mut();
pub static mut r_fastsky: *mut cvar_t = core::ptr::null_mut();
pub static mut r_drawSun: *mut cvar_t = core::ptr::null_mut();
pub static mut r_dynamiclight: *mut cvar_t = core::ptr::null_mut();
// rjr - removed for hacking cvar_t	*r_dlightBacks;

pub static mut r_lodbias: *mut cvar_t = core::ptr::null_mut();
pub static mut r_lodscale: *mut cvar_t = core::ptr::null_mut();
pub static mut r_autolodscalevalue: *mut cvar_t = core::ptr::null_mut();

pub static mut r_norefresh: *mut cvar_t = core::ptr::null_mut();
pub static mut r_drawentities: *mut cvar_t = core::ptr::null_mut();
pub static mut r_drawworld: *mut cvar_t = core::ptr::null_mut();
pub static mut r_drawfog: *mut cvar_t = core::ptr::null_mut();
pub static mut r_speeds: *mut cvar_t = core::ptr::null_mut();
pub static mut r_fullbright: *mut cvar_t = core::ptr::null_mut();
pub static mut r_novis: *mut cvar_t = core::ptr::null_mut();
pub static mut r_nocull: *mut cvar_t = core::ptr::null_mut();
pub static mut r_facePlaneCull: *mut cvar_t = core::ptr::null_mut();
pub static mut r_cullRoofFaces: *mut cvar_t = core::ptr::null_mut(); //attempted smart method of culling out upwards facing surfaces on roofs for automap shots -rww
pub static mut r_roofCullCeilDist: *mut cvar_t = core::ptr::null_mut(); //ceiling distance cull tolerance -rww
pub static mut r_roofCullFloorDist: *mut cvar_t = core::ptr::null_mut(); //floor distance cull tolerance -rww
pub static mut r_showcluster: *mut cvar_t = core::ptr::null_mut();
pub static mut r_nocurves: *mut cvar_t = core::ptr::null_mut();

pub static mut r_autoMap: *mut cvar_t = core::ptr::null_mut(); //automap renderside toggle for debugging -rww
pub static mut r_autoMapBackAlpha: *mut cvar_t = core::ptr::null_mut(); //alpha of automap bg -rww
pub static mut r_autoMapDisable: *mut cvar_t = core::ptr::null_mut(); //don't calc it (since it's slow in debug) -rww

pub static mut r_dlightStyle: *mut cvar_t = core::ptr::null_mut();
pub static mut r_surfaceSprites: *mut cvar_t = core::ptr::null_mut();
pub static mut r_surfaceWeather: *mut cvar_t = core::ptr::null_mut();

pub static mut r_windSpeed: *mut cvar_t = core::ptr::null_mut();
pub static mut r_windAngle: *mut cvar_t = core::ptr::null_mut();
pub static mut r_windGust: *mut cvar_t = core::ptr::null_mut();
pub static mut r_windDampFactor: *mut cvar_t = core::ptr::null_mut();
pub static mut r_windPointForce: *mut cvar_t = core::ptr::null_mut();
pub static mut r_windPointX: *mut cvar_t = core::ptr::null_mut();
pub static mut r_windPointY: *mut cvar_t = core::ptr::null_mut();

pub static mut r_allowExtensions: *mut cvar_t = core::ptr::null_mut();

pub static mut r_ext_compressed_textures: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ext_compressed_lightmaps: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ext_preferred_tc_method: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ext_gamma_control: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ext_multitexture: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ext_compiled_vertex_array: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ext_texture_env_add: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ext_texture_filter_anisotropic: *mut cvar_t = core::ptr::null_mut();

pub static mut r_DynamicGlow: *mut cvar_t = core::ptr::null_mut();
pub static mut r_DynamicGlowPasses: *mut cvar_t = core::ptr::null_mut();
pub static mut r_DynamicGlowDelta: *mut cvar_t = core::ptr::null_mut();
pub static mut r_DynamicGlowIntensity: *mut cvar_t = core::ptr::null_mut();
pub static mut r_DynamicGlowSoft: *mut cvar_t = core::ptr::null_mut();
pub static mut r_DynamicGlowWidth: *mut cvar_t = core::ptr::null_mut();
pub static mut r_DynamicGlowHeight: *mut cvar_t = core::ptr::null_mut();

pub static mut r_ignoreGLErrors: *mut cvar_t = core::ptr::null_mut();
pub static mut r_logFile: *mut cvar_t = core::ptr::null_mut();

pub static mut r_stencilbits: *mut cvar_t = core::ptr::null_mut();
pub static mut r_depthbits: *mut cvar_t = core::ptr::null_mut();
pub static mut r_colorbits: *mut cvar_t = core::ptr::null_mut();
pub static mut r_stereo: *mut cvar_t = core::ptr::null_mut();
pub static mut r_primitives: *mut cvar_t = core::ptr::null_mut();
pub static mut r_texturebits: *mut cvar_t = core::ptr::null_mut();
pub static mut r_texturebitslm: *mut cvar_t = core::ptr::null_mut();

pub static mut r_lightmap: *mut cvar_t = core::ptr::null_mut();
pub static mut r_vertexLight: *mut cvar_t = core::ptr::null_mut();
pub static mut r_uiFullScreen: *mut cvar_t = core::ptr::null_mut();
pub static mut r_shadows: *mut cvar_t = core::ptr::null_mut();
pub static mut r_shadowRange: *mut cvar_t = core::ptr::null_mut();
pub static mut r_flares: *mut cvar_t = core::ptr::null_mut();
pub static mut r_mode: *mut cvar_t = core::ptr::null_mut();
pub static mut r_nobind: *mut cvar_t = core::ptr::null_mut();
pub static mut r_singleShader: *mut cvar_t = core::ptr::null_mut();
pub static mut r_colorMipLevels: *mut cvar_t = core::ptr::null_mut();
pub static mut r_picmip: *mut cvar_t = core::ptr::null_mut();
pub static mut r_showtris: *mut cvar_t = core::ptr::null_mut();
pub static mut r_showsky: *mut cvar_t = core::ptr::null_mut();
pub static mut r_shownormals: *mut cvar_t = core::ptr::null_mut();
pub static mut r_finish: *mut cvar_t = core::ptr::null_mut();
pub static mut r_clear: *mut cvar_t = core::ptr::null_mut();
pub static mut r_swapInterval: *mut cvar_t = core::ptr::null_mut();
pub static mut r_markcount: *mut cvar_t = core::ptr::null_mut();
pub static mut r_textureMode: *mut cvar_t = core::ptr::null_mut();
pub static mut r_offsetFactor: *mut cvar_t = core::ptr::null_mut();
pub static mut r_offsetUnits: *mut cvar_t = core::ptr::null_mut();
pub static mut r_gamma: *mut cvar_t = core::ptr::null_mut();
pub static mut r_intensity: *mut cvar_t = core::ptr::null_mut();
pub static mut r_lockpvs: *mut cvar_t = core::ptr::null_mut();
pub static mut r_noportals: *mut cvar_t = core::ptr::null_mut();
pub static mut r_portalOnly: *mut cvar_t = core::ptr::null_mut();

pub static mut r_subdivisions: *mut cvar_t = core::ptr::null_mut();
pub static mut r_lodCurveError: *mut cvar_t = core::ptr::null_mut();

pub static mut r_fullscreen: *mut cvar_t = core::ptr::null_mut();

pub static mut r_customwidth: *mut cvar_t = core::ptr::null_mut();
pub static mut r_customheight: *mut cvar_t = core::ptr::null_mut();

pub static mut r_overBrightBits: *mut cvar_t = core::ptr::null_mut();

pub static mut r_debugSurface: *mut cvar_t = core::ptr::null_mut();
pub static mut r_simpleMipMaps: *mut cvar_t = core::ptr::null_mut();

pub static mut r_showImages: *mut cvar_t = core::ptr::null_mut();

pub static mut r_ambientScale: *mut cvar_t = core::ptr::null_mut();
pub static mut r_directedScale: *mut cvar_t = core::ptr::null_mut();
pub static mut r_debugLight: *mut cvar_t = core::ptr::null_mut();
pub static mut r_debugSort: *mut cvar_t = core::ptr::null_mut();

pub static mut r_maxpolys: *mut cvar_t = core::ptr::null_mut();
pub static mut max_polys: c_int = 0;
pub static mut r_maxpolyverts: *mut cvar_t = core::ptr::null_mut();
pub static mut max_polyverts: c_int = 0;

pub static mut r_modelpoolmegs: *mut cvar_t = core::ptr::null_mut();

#[cfg(target_os = "xbox")]
pub static mut r_hdreffect: *mut cvar_t = core::ptr::null_mut();
#[cfg(target_os = "xbox")]
pub static mut r_sundir_x: *mut cvar_t = core::ptr::null_mut();
#[cfg(target_os = "xbox")]
pub static mut r_sundir_y: *mut cvar_t = core::ptr::null_mut();
#[cfg(target_os = "xbox")]
pub static mut r_sundir_z: *mut cvar_t = core::ptr::null_mut();
#[cfg(target_os = "xbox")]
pub static mut r_hdrbloom: *mut cvar_t = core::ptr::null_mut();


/*
Ghoul2 Insert Start
*/
#[cfg(debug_assertions)]
pub static mut r_noPrecacheGLA: *mut cvar_t = core::ptr::null_mut();

pub static mut r_noServerGhoul2: *mut cvar_t = core::ptr::null_mut();
pub static mut r_Ghoul2AnimSmooth: *mut cvar_t = core::ptr::null_mut();
pub static mut r_Ghoul2UnSqashAfterSmooth: *mut cvar_t = core::ptr::null_mut();
//cvar_t	*r_Ghoul2UnSqash;
//cvar_t	*r_Ghoul2TimeBase=0; from single player
//cvar_t	*r_Ghoul2NoLerp;
//cvar_t	*r_Ghoul2NoBlend;
//cvar_t	*r_Ghoul2BlendMultiplier=0;

pub static mut broadsword: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_kickbones: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_kickorigin: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_playflop: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_dontstopanim: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_waitforshot: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_smallbbox: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_extra1: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_extra2: *mut cvar_t = core::ptr::null_mut();

pub static mut broadsword_effcorr: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_ragtobase: *mut cvar_t = core::ptr::null_mut();
pub static mut broadsword_dircap: *mut cvar_t = core::ptr::null_mut();

/*
Ghoul2 Insert End
*/

#[cfg(not(feature = "dedicated"))]
pub static mut qglMultiTexCoord2fARB: extern "C" fn(u32, f32, f32) = unsafe { core::mem::zeroed() };
#[cfg(not(feature = "dedicated"))]
pub static mut qglActiveTextureARB: extern "C" fn(u32) = unsafe { core::mem::zeroed() };
#[cfg(not(feature = "dedicated"))]
pub static mut qglClientActiveTextureARB: extern "C" fn(u32) = unsafe { core::mem::zeroed() };

#[cfg(not(feature = "dedicated"))]
pub static mut qglLockArraysEXT: extern "C" fn(i32, i32) = unsafe { core::mem::zeroed() };
#[cfg(not(feature = "dedicated"))]
pub static mut qglUnlockArraysEXT: extern "C" fn() = unsafe { core::mem::zeroed() };

#[cfg(not(feature = "dedicated"))]
pub static mut qglPointParameterfEXT: extern "C" fn(u32, f32) = unsafe { core::mem::zeroed() };
#[cfg(not(feature = "dedicated"))]
pub static mut qglPointParameterfvEXT: extern "C" fn(u32, *mut f32) = unsafe { core::mem::zeroed() };

//3d textures -rww
#[cfg(not(feature = "dedicated"))]
pub static mut qglTexImage3DEXT: extern "C" fn(u32, i32, u32, i32, i32, i32, i32, u32, u32, *const c_void) = unsafe { core::mem::zeroed() };
#[cfg(not(feature = "dedicated"))]
pub static mut qglTexSubImage3DEXT: extern "C" fn(u32, i32, i32, i32, i32, i32, i32, i32, u32, u32, *const c_void) = unsafe { core::mem::zeroed() };


#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
// Declare Register Combiners function pointers.
pub static mut qglCombinerParameterfvNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglCombinerParameterivNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglCombinerParameterfNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglCombinerParameteriNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglCombinerInputNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglCombinerOutputNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglFinalCombinerInputNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetCombinerInputParameterfvNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetCombinerInputParameterivNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetCombinerOutputParameterfvNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetCombinerOutputParameterivNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetFinalCombinerInputParameterfvNV: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetFinalCombinerInputParameterivNV: *mut c_void = core::ptr::null_mut();

// Declare Pixel Format function pointers.
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglGetPixelFormatAttribivARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglGetPixelFormatAttribfvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglChoosePixelFormatARB: *mut c_void = core::ptr::null_mut();

// Declare Pixel Buffer function pointers.
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglCreatePbufferARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglGetPbufferDCARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglReleasePbufferDCARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglDestroyPbufferARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglQueryPbufferARB: *mut c_void = core::ptr::null_mut();

// Declare Render-Texture function pointers.
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglBindTexImageARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglReleaseTexImageARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qwglSetPbufferAttribARB: *mut c_void = core::ptr::null_mut();

// Declare Vertex and Fragment Program function pointers.
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramStringARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglBindProgramARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglDeleteProgramsARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGenProgramsARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramEnvParameter4dARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramEnvParameter4dvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramEnvParameter4fARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramEnvParameter4fvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramLocalParameter4dARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramLocalParameter4dvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramLocalParameter4fARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglProgramLocalParameter4fvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetProgramEnvParameterdvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetProgramEnvParameterfvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetProgramLocalParameterdvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetProgramLocalParameterfvARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetProgramivARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglGetProgramStringARB: *mut c_void = core::ptr::null_mut();
#[cfg(all(not(feature = "dedicated"), not(target_os = "xbox")))]
pub static mut qglIsProgramARB: *mut c_void = core::ptr::null_mut();


#[cfg(not(feature = "dedicated"))]
extern "C" {
    fn RE_SetLightStyle(style: c_int, color: c_int);
    fn RE_GetBModelVerts(bmodelIndex: c_int, verts: *mut [f32; 3], normal: [f32; 3]);
}

// Stub types for dependencies - these would be defined in proper module structure
#[repr(C)]
pub struct cvar_t {
    // Placeholder - actual definition would be in qcommon module
}

#[repr(C)]
pub struct glconfig_t {
    // Placeholder
}

#[repr(C)]
pub struct glstate_t {
    // Placeholder
}

#[repr(C)]
pub struct image_t {
    // Placeholder
}

fn AssertCvarRange(cv: *mut cvar_t, minVal: f32, maxVal: f32, shouldBeIntegral: bool) {
    unsafe {
        if shouldBeIntegral {
            if ((*cv).value as c_int) != (*cv).integer {
                // Com_Printf (S_COLOR_YELLOW  "WARNING: cvar '%s' must be integral (%f)\n", cv->name, cv->value );
                // Cvar_Set( cv->name, va( "%d", cv->integer ) );
            }
        }
        // if ( cv->value < minVal )
        // {
        // Com_Printf (S_COLOR_YELLOW  "WARNING: cvar '%s' out of range (%f < %f)\n", cv->name, cv->value, minVal );
        // Cvar_Set( cv->name, va( "%f", minVal ) );
        // }
        // else if ( cv->value > maxVal )
        // {
        // Com_Printf (S_COLOR_YELLOW  "WARNING: cvar '%s' out of range (%f > %f)\n", cv->name, cv->value, maxVal );
        // Cvar_Set( cv->name, va( "%f", maxVal ) );
        // }
    }
}

#[cfg(not(feature = "dedicated"))]
fn R_Splash() {
    #[cfg(not(target_os = "xbox"))]
    {
        // image_t *pImage;
        // pImage = R_FindImageFile( "menu/splash", qfalse, qfalse, qfalse, GL_CLAMP);
        // extern void	RB_SetGL2D (void);
        // RB_SetGL2D();
        // if (pImage )
        // {//invalid paths?
        // GL_Bind( pImage );
        // }
        // GL_State(GLS_SRCBLEND_ONE | GLS_DSTBLEND_ZERO);

        // const int width = 640;
        // const int height = 480;
        // const float x1 = 320 - width / 2;
        // const float x2 = 320 + width / 2;
        // const float y1 = 240 - height / 2;
        // const float y2 = 240 + height / 2;

        // qglBegin (GL_TRIANGLE_STRIP);
        // qglTexCoord2f( 0,  0 );
        // qglVertex2f(x1, y1);
        // qglTexCoord2f( 1 ,  0 );
        // qglVertex2f(x2, y1);
        // qglTexCoord2f( 0, 1 );
        // qglVertex2f(x1, y2);
        // qglTexCoord2f( 1, 1 );
        // qglVertex2f(x2, y2);
        // qglEnd();

        // GLimp_EndFrame();
    }
}

#[cfg(not(feature = "dedicated"))]
fn InitOpenGL() {
    //
    // initialize OS specific portions of the renderer
    //
    // GLimp_Init directly or indirectly references the following cvars:
    //		- r_fullscreen
    //		- r_mode
    //		- r_(color|depth|stencil)bits
    //		- r_ignorehwgamma
    //		- r_gamma
    //

    // if ( glConfig.vidWidth == 0 )
    // {
    // GLimp_Init();
    // // print info the first time only
    // GL_SetDefaultState();
    // R_Splash();	//get something on screen asap
    // GfxInfo_f();
    // }
    // else
    // {
    // // set default state
    // GL_SetDefaultState();
    // }
    // // init command buffers and SMP
    // R_InitCommandBuffers();
}

#[cfg(not(feature = "dedicated"))]
fn GL_CheckErrors() {
    // int		err;
    // char	s[64];

    // err = qglGetError();
    // if ( err == GL_NO_ERROR ) {
    // return;
    // }
    // if ( r_ignoreGLErrors->integer ) {
    // return;
    // }
    // switch( err ) {
    // case GL_INVALID_ENUM:
    // strcpy( s, "GL_INVALID_ENUM" );
    // break;
    // case GL_INVALID_VALUE:
    // strcpy( s, "GL_INVALID_VALUE" );
    // break;
    // case GL_INVALID_OPERATION:
    // strcpy( s, "GL_INVALID_OPERATION" );
    // break;
    // case GL_STACK_OVERFLOW:
    // strcpy( s, "GL_STACK_OVERFLOW" );
    // break;
    // case GL_STACK_UNDERFLOW:
    // strcpy( s, "GL_STACK_UNDERFLOW" );
    // break;
    // case GL_OUT_OF_MEMORY:
    // strcpy( s, "GL_OUT_OF_MEMORY" );
    // break;
    // default:
    // Com_sprintf( s, sizeof(s), "%i", err);
    // break;
    // }

    // Com_Error( ERR_FATAL, "GL_CheckErrors: %s", s );
}

#[cfg(not(target_os = "xbox"))]
#[repr(C)]
pub struct vidmode_s {
    pub description: *const c_char,
    pub width: c_int,
    pub height: c_int,
}

pub type vidmode_t = vidmode_s;

#[cfg(not(target_os = "xbox"))]
const R_VID_MODES: &[vidmode_t] = &[
    vidmode_t { description: b"Mode  0: 320x240\0".as_ptr() as *const c_char, width: 320, height: 240 },
    vidmode_t { description: b"Mode  1: 400x300\0".as_ptr() as *const c_char, width: 400, height: 300 },
    vidmode_t { description: b"Mode  2: 512x384\0".as_ptr() as *const c_char, width: 512, height: 384 },
    vidmode_t { description: b"Mode  3: 640x480\0".as_ptr() as *const c_char, width: 640, height: 480 },
    vidmode_t { description: b"Mode  4: 800x600\0".as_ptr() as *const c_char, width: 800, height: 600 },
    vidmode_t { description: b"Mode  5: 960x720\0".as_ptr() as *const c_char, width: 960, height: 720 },
    vidmode_t { description: b"Mode  6: 1024x768\0".as_ptr() as *const c_char, width: 1024, height: 768 },
    vidmode_t { description: b"Mode  7: 1152x864\0".as_ptr() as *const c_char, width: 1152, height: 864 },
    vidmode_t { description: b"Mode  8: 1280x1024\0".as_ptr() as *const c_char, width: 1280, height: 1024 },
    vidmode_t { description: b"Mode  9: 1600x1200\0".as_ptr() as *const c_char, width: 1600, height: 1200 },
    vidmode_t { description: b"Mode 10: 2048x1536\0".as_ptr() as *const c_char, width: 2048, height: 1536 },
    vidmode_t { description: b"Mode 11: 856x480 (wide)\0".as_ptr() as *const c_char, width: 856, height: 480 },
    vidmode_t { description: b"Mode 12: 2400x600(surround)\0".as_ptr() as *const c_char, width: 2400, height: 600 }
];

#[cfg(not(target_os = "xbox"))]
static S_NUM_VID_MODES: c_int = 13; // (sizeof( r_vidModes ) / sizeof( r_vidModes[0] ))

#[cfg(not(target_os = "xbox"))]
pub fn R_GetModeInfo(width: *mut c_int, height: *mut c_int, mode: c_int) -> bool {
    unsafe {
        if mode < -1 {
            return false;
        }
        if mode >= S_NUM_VID_MODES {
            return false;
        }

        if mode == -1 {
            *width = (*r_customwidth).integer;
            *height = (*r_customheight).integer;
            return true;
        }

        let vm = &R_VID_MODES[mode as usize];

        *width = vm.width;
        *height = vm.height;

        return true;
    }
}

#[cfg(not(target_os = "xbox"))]
fn R_ModeList_f() {
    // int i;

    // Com_Printf ("\n" );
    // for ( i = 0; i < s_numVidModes; i++ )
    // {
    // Com_Printf ("%s\n", r_vidModes[i].description );
    // }
    // Com_Printf ("\n" );
}

/*
==============================================================================

						SCREEN SHOTS

==============================================================================
*/
#[cfg(not(feature = "dedicated"))]
/*
==================
R_TakeScreenshot
==================
*/
fn R_TakeScreenshot(x: c_int, y: c_int, width: c_int, height: c_int, fileName: *mut c_char) {
    #[cfg(not(target_os = "xbox"))]
    {
        // byte		*buffer;
        // int			i, c, temp;

        // buffer = (unsigned char *)Hunk_AllocateTempMemory(glConfig.vidWidth*glConfig.vidHeight*3+18);

        // Com_Memset (buffer, 0, 18);
        // buffer[2] = 2;		// uncompressed type
        // buffer[12] = width & 255;
        // buffer[13] = width >> 8;
        // buffer[14] = height & 255;
        // buffer[15] = height >> 8;
        // buffer[16] = 24;	// pixel size

        // qglReadPixels( x, y, width, height, GL_RGB, GL_UNSIGNED_BYTE, buffer+18 );

        // // swap rgb to bgr
        // c = 18 + width * height * 3;
        // for (i=18 ; i<c ; i+=3) {
        // temp = buffer[i];
        // buffer[i] = buffer[i+2];
        // buffer[i+2] = temp;
        // }

        // // gamma correct
        // if ( ( tr.overbrightBits > 0 ) && glConfig.deviceSupportsGamma ) {
        // R_GammaCorrect( buffer + 18, glConfig.vidWidth * glConfig.vidHeight * 3 );
        // }

        // FS_WriteFile( fileName, buffer, c );

        // Hunk_FreeTempMemory( buffer );
    }
}

#[cfg(not(feature = "dedicated"))]
/*
==================
R_TakeScreenshotJPEG
==================
*/
fn R_TakeScreenshotJPEG(x: c_int, y: c_int, width: c_int, height: c_int, fileName: *mut c_char) {
    #[cfg(not(target_os = "xbox"))]
    {
        // byte		*buffer;

        // buffer = (unsigned char *)Hunk_AllocateTempMemory(glConfig.vidWidth*glConfig.vidHeight*4);

        // qglReadPixels( x, y, width, height, GL_RGBA, GL_UNSIGNED_BYTE, buffer );

        // // gamma correct
        // if ( ( tr.overbrightBits > 0 ) && glConfig.deviceSupportsGamma ) {
        // R_GammaCorrect( buffer, glConfig.vidWidth * glConfig.vidHeight * 4 );
        // }

        // FS_WriteFile( fileName, buffer, 1 );		// create path
        // SaveJPG( fileName, 95, glConfig.vidWidth, glConfig.vidHeight, buffer);

        // Hunk_FreeTempMemory( buffer );
    }
}

#[cfg(not(feature = "dedicated"))]
/*
==================
R_ScreenshotFilename
==================
*/
fn R_ScreenshotFilename(mut lastNumber: c_int, fileName: *mut c_char, psExt: *const c_char) {
    let mut a: c_int;
    let mut b: c_int;
    let mut c: c_int;
    let mut d: c_int;

    if lastNumber < 0 || lastNumber > 9999 {
        // Com_sprintf( fileName, MAX_OSPATH, "screenshots/shot9999%s",psExt );
        return;
    }

    a = lastNumber / 1000;
    lastNumber -= a*1000;
    b = lastNumber / 100;
    lastNumber -= b*100;
    c = lastNumber / 10;
    lastNumber -= c*10;
    d = lastNumber;

    // Com_sprintf( fileName, MAX_OSPATH, "screenshots/shot%i%i%i%i%s"
    // , a, b, c, d, psExt );
}

#[cfg(not(feature = "dedicated"))]
/*
====================
R_LevelShot

levelshots are specialized 256*256 thumbnails for
the menu system, sampled down from full screen distorted images
====================
*/
#[allow(non_upper_case_globals)]
const LEVELSHOTSIZE: c_int = 256;
#[cfg(not(feature = "dedicated"))]
fn R_LevelShot() {
    #[cfg(not(target_os = "xbox"))]
    {
        // char		checkname[MAX_OSPATH];
        // byte		*buffer;
        // byte		*source;
        // byte		*src, *dst;
        // int			x, y;
        // int			r, g, b;
        // float		xScale, yScale;
        // int			xx, yy;

        // sprintf( checkname, "levelshots/%s.tga", tr.world->baseName );

        // source = (unsigned char *)Hunk_AllocateTempMemory( glConfig.vidWidth * glConfig.vidHeight * 3 );

        // buffer = (unsigned char *)Hunk_AllocateTempMemory( LEVELSHOTSIZE * LEVELSHOTSIZE*3 + 18);
        // Com_Memset (buffer, 0, 18);
        // buffer[2] = 2;		// uncompressed type
        // buffer[12] = LEVELSHOTSIZE & 255;
        // buffer[13] = LEVELSHOTSIZE >> 8;
        // buffer[14] = LEVELSHOTSIZE & 255;
        // buffer[15] = LEVELSHOTSIZE >> 8;
        // buffer[16] = 24;	// pixel size

        // qglReadPixels( 0, 0, glConfig.vidWidth, glConfig.vidHeight, GL_RGB, GL_UNSIGNED_BYTE, source );

        // // resample from source
        // xScale = glConfig.vidWidth / (4.0*LEVELSHOTSIZE);
        // yScale = glConfig.vidHeight / (3.0*LEVELSHOTSIZE);
        // for ( y = 0 ; y < LEVELSHOTSIZE ; y++ ) {
        // for ( x = 0 ; x < LEVELSHOTSIZE ; x++ ) {
        // r = g = b = 0;
        // for ( yy = 0 ; yy < 3 ; yy++ ) {
        // for ( xx = 0 ; xx < 4 ; xx++ ) {
        // src = source + 3 * ( glConfig.vidWidth * (int)( (y*3+yy)*yScale ) + (int)( (x*4+xx)*xScale ) );
        // r += src[0];
        // g += src[1];
        // b += src[2];
        // }
        // }
        // dst = buffer + 18 + 3 * ( y * LEVELSHOTSIZE + x );
        // dst[0] = b / 12;
        // dst[1] = g / 12;
        // dst[2] = r / 12;
        // }
        // }

        // // gamma correct
        // if ( ( tr.overbrightBits > 0 ) && glConfig.deviceSupportsGamma ) {
        // R_GammaCorrect( buffer + 18, LEVELSHOTSIZE * LEVELSHOTSIZE * 3 );
        // }

        // FS_WriteFile( checkname, buffer, LEVELSHOTSIZE * LEVELSHOTSIZE*3 + 18 );

        // Hunk_FreeTempMemory( buffer );
        // Hunk_FreeTempMemory( source );

        // Com_Printf ("Wrote %s\n", checkname );
    }
}

#[cfg(not(feature = "dedicated"))]
/*
==================
R_ScreenShotTGA_f

screenshot
screenshot [silent]
screenshot [levelshot]
screenshot [filename]

Doesn't print the pacifier message if there is a second arg
==================
*/
fn R_ScreenShotTGA_f() {
    #[cfg(not(target_os = "xbox"))]
    {
        // char		checkname[MAX_OSPATH];
        // static	int	lastNumber = -1;
        // qboolean	silent;

        // if ( !strcmp( Cmd_Argv(1), "levelshot" ) ) {
        // R_LevelShot();
        // return;
        // }

        // if ( !strcmp( Cmd_Argv(1), "silent" ) ) {
        // silent = qtrue;
        // } else {
        // silent = qfalse;
        // }

        // if ( Cmd_Argc() == 2 && !silent ) {
        // // explicit filename
        // Com_sprintf( checkname, MAX_OSPATH, "screenshots/%s.tga", Cmd_Argv( 1 ) );
        // } else {
        // // scan for a free filename

        // // if we have saved a previous screenshot, don't scan
        // // again, because recording demo avis can involve
        // // thousands of shots
        // if ( lastNumber == -1 ) {
        // lastNumber = 0;
        // }
        // // scan for a free number
        // for ( ; lastNumber <= 9999 ; lastNumber++ ) {
        // R_ScreenshotFilename( lastNumber, checkname, ".tga" );

        // if (!FS_FileExists( checkname ))
        // {
        // break; // file doesn't exist
        // }
        // }

        // if ( lastNumber >= 9999 ) {
        // Com_Printf ( "ScreenShot: Couldn't create a file\n");
        // return;
        // }

        // lastNumber++;
        // }


        // R_TakeScreenshot( 0, 0, glConfig.vidWidth, glConfig.vidHeight, checkname );

        // if ( !silent ) {
        // Com_Printf ( "Wrote %s\n", checkname);
        // }
    }
}

#[cfg(not(feature = "dedicated"))]
//jpeg  vession
fn R_ScreenShot_f() {
    #[cfg(not(target_os = "xbox"))]
    {
        // char		checkname[MAX_OSPATH];
        // static	int	lastNumber = -1;
        // qboolean	silent;

        // if ( !strcmp( Cmd_Argv(1), "levelshot" ) ) {
        // R_LevelShot();
        // return;
        // }
        // if ( !strcmp( Cmd_Argv(1), "silent" ) ) {
        // silent = qtrue;
        // } else {
        // silent = qfalse;
        // }

        // if ( Cmd_Argc() == 2 && !silent ) {
        // // explicit filename
        // Com_sprintf( checkname, MAX_OSPATH, "screenshots/%s.jpg", Cmd_Argv( 1 ) );
        // } else {
        // // scan for a free filename

        // // if we have saved a previous screenshot, don't scan
        // // again, because recording demo avis can involve
        // // thousands of shots
        // if ( lastNumber == -1 ) {
        // lastNumber = 0;
        // }
        // // scan for a free number
        // for ( ; lastNumber <= 9999 ; lastNumber++ ) {
        // R_ScreenshotFilename( lastNumber, checkname, ".jpg" );

        // if (!FS_FileExists( checkname ))
        // {
        // break; // file doesn't exist
        // }
        // }

        // if ( lastNumber == 10000 ) {
        // Com_Printf ( "ScreenShot: Couldn't create a file\n");
        // return;
        // }

        // lastNumber++;
        // }


        // R_TakeScreenshotJPEG( 0, 0, glConfig.vidWidth, glConfig.vidHeight, checkname );

        // if ( !silent ) {
        // Com_Printf ( "Wrote %s\n", checkname);
        // }
    }
}

//============================================================================

#[cfg(not(feature = "dedicated"))]
/*
** GL_SetDefaultState
*/
fn GL_SetDefaultState() {
    // qglClearDepth( 1.0f );

    // qglCullFace(GL_FRONT);

    // qglColor4f (1,1,1,1);

    // // initialize downstream texture unit if we're running
    // // in a multitexture environment
    // if ( qglActiveTextureARB ) {
    // GL_SelectTexture( 1 );
    // GL_TextureMode( r_textureMode->string );
    // GL_TexEnv( GL_MODULATE );
    // qglDisable( GL_TEXTURE_2D );
    // GL_SelectTexture( 0 );
    // }

    // qglEnable(GL_TEXTURE_2D);
    // GL_TextureMode( r_textureMode->string );
    // GL_TexEnv( GL_MODULATE );

    // qglShadeModel( GL_SMOOTH );
    // qglDepthFunc( GL_LEQUAL );

    // // the vertex array is always enabled, but the color and texture
    // // arrays are enabled and disabled around the compiled vertex array call
    // qglEnableClientState (GL_VERTEX_ARRAY);

    // //
    // // make sure our GL state vector is set correctly
    // //
    // glState.glStateBits = GLS_DEPTHTEST_DISABLE | GLS_DEPTHMASK_TRUE;

    // qglPolygonMode (GL_FRONT_AND_BACK, GL_FILL);
    // qglDepthMask( GL_TRUE );
    // qglDisable( GL_DEPTH_TEST );
    // qglEnable( GL_SCISSOR_TEST );
    // qglDisable( GL_CULL_FACE );
    // qglDisable( GL_BLEND );
    // #ifdef _XBOX
    // qglDisable( GL_LIGHTING );
    // #endif
}


#[cfg(not(feature = "dedicated"))]
/*
================
GfxInfo_f
================
*/
extern "C" {
    static mut g_bTextureRectangleHack: bool;
}

#[cfg(not(feature = "dedicated"))]
fn GfxInfo_f() {
    // cvar_t *sys_cpustring = Cvar_Get( "sys_cpustring", "", CVAR_ROM );
    // const char *enablestrings[] =
    // {
    // "disabled",
    // "enabled"
    // };
    // const char *fsstrings[] =
    // {
    // "windowed",
    // "fullscreen"
    // };

    // const char *tc_table[] =
    // {
    // "None",
    // "GL_S3_s3tc",
    // "GL_EXT_texture_compression_s3tc",
    // };

    // Com_Printf ("\nGL_VENDOR: %s\n", glConfig.vendor_string );
    // Com_Printf ("GL_RENDERER: %s\n", glConfig.renderer_string );
    // Com_Printf ("GL_VERSION: %s\n", glConfig.version_string );
    // Com_Printf ("GL_EXTENSIONS: %s\n", glConfig.extensions_string );
    // Com_Printf ("GL_MAX_TEXTURE_SIZE: %d\n", glConfig.maxTextureSize );
    // Com_Printf ("GL_MAX_ACTIVE_TEXTURES_ARB: %d\n", glConfig.maxActiveTextures );
    // Com_Printf ("\nPIXELFORMAT: color(%d-bits) Z(%d-bit) stencil(%d-bits)\n", glConfig.colorBits, glConfig.depthBits, glConfig.stencilBits );
    // Com_Printf ("MODE: %d, %d x %d %s hz:", r_mode->integer, glConfig.vidWidth, glConfig.vidHeight, fsstrings[r_fullscreen->integer == 1] );
    // if ( glConfig.displayFrequency )
    // {
    // Com_Printf ("%d\n", glConfig.displayFrequency );
    // }
    // else
    // {
    // Com_Printf ("N/A\n" );
    // }
    // if ( glConfig.deviceSupportsGamma )
    // {
    // Com_Printf ("GAMMA: hardware w/ %d overbright bits\n", tr.overbrightBits );
    // }
    // else
    // {
    // Com_Printf ("GAMMA: software w/ %d overbright bits\n", tr.overbrightBits );
    // }
    // Com_Printf ("CPU: %s @ %s MHz\n", sys_cpustring->string, Cvar_VariableString("sys_cpuspeed") );

    // // rendering primitives
    // {
    // int		primitives;

    // // default is to use triangles if compiled vertex arrays are present
    // Com_Printf ("rendering primitives: " );
    // primitives = r_primitives->integer;
    // if ( primitives == 0 ) {
    // if ( qglLockArraysEXT ) {
    // primitives = 2;
    // } else {
    // primitives = 1;
    // }
    // }
    // if ( primitives == -1 ) {
    // Com_Printf ("none\n" );
    // } else if ( primitives == 2 ) {
    // Com_Printf ("single glDrawElements\n" );
    // } else if ( primitives == 1 ) {
    // Com_Printf ("multiple glArrayElement\n" );
    // } else if ( primitives == 3 ) {
    // Com_Printf ("multiple glColor4ubv + glTexCoord2fv + glVertex3fv\n" );
    // }
    // }

    // Com_Printf ("texturemode: %s\n", r_textureMode->string );
    // Com_Printf ("picmip: %d\n", r_picmip->integer );
    // Com_Printf ("texture bits: %d\n", r_texturebits->integer );
    // Com_Printf ("lightmap texture bits: %d\n", r_texturebitslm->integer );
    // Com_Printf ("multitexture: %s\n", enablestrings[qglActiveTextureARB != 0] );
    // Com_Printf ("compiled vertex arrays: %s\n", enablestrings[qglLockArraysEXT != 0 ] );
    // Com_Printf ("texenv add: %s\n", enablestrings[glConfig.textureEnvAddAvailable != 0] );
    // Com_Printf ("compressed textures: %s\n", enablestrings[glConfig.textureCompression != TC_NONE] );
    // Com_Printf ("compressed lightmaps: %s\n", enablestrings[(r_ext_compressed_lightmaps->integer != 0 && glConfig.textureCompression != TC_NONE)] );
    // Com_Printf ("texture compression method: %s\n", tc_table[glConfig.textureCompression] );
    // Com_Printf ("anisotropic filtering: %s  ", enablestrings[(r_ext_texture_filter_anisotropic->integer != 0) && glConfig.maxTextureFilterAnisotropy] );
    // Com_Printf ("(%f of %f)\n", r_ext_texture_filter_anisotropic->value, glConfig.maxTextureFilterAnisotropy );
    // Com_Printf ("Dynamic Glow: %s\n", enablestrings[r_DynamicGlow->integer] );
    // if (g_bTextureRectangleHack) Com_Printf ("Dynamic Glow ATI BAD DRIVER HACK %s\n", enablestrings[g_bTextureRectangleHack] );

    // if ( r_finish->integer ) {
    // Com_Printf ("Forcing glFinish\n" );
    // }
    // if ( r_displayRefresh ->integer ) {
    // Com_Printf ("Display refresh set to %d\n", r_displayRefresh->integer );
    // }
    // if (tr.world)
    // {
    // Com_Printf ("Light Grid size set to (%.2f %.2f %.2f)\n", tr.world->lightGridSize[0], tr.world->lightGridSize[1], tr.world->lightGridSize[2] );
    // }
}

#[cfg(not(feature = "dedicated"))]
fn R_AtiHackToggle_f() {
    unsafe {
        g_bTextureRectangleHack = !g_bTextureRectangleHack;
    }
}

/*
===============
R_Register
===============
*/
pub fn R_Register() {
    //
    // latched and archived variables
    //
    // r_allowExtensions = Cvar_Get( "r_allowExtensions", "1", CVAR_ARCHIVE | CVAR_LATCH );
    // r_ext_compressed_textures = Cvar_Get( "r_ext_compress_textures", "1", CVAR_ARCHIVE | CVAR_LATCH );
    // ... (many more Cvar_Get calls)

    // The actual implementation would call Cvar_Get for all the cvars
    // and register console commands, but this is stubbed out for the faithful port
}

/*
===============
R_Init
===============
*/
extern "C" {
    fn R_InitWorldEffects();
}

pub fn R_Init() {
    // let mut i: c_int;
    // let mut ptr: *mut u8;

    // //	Com_Printf ("----- R_Init -----\n" );
    // #ifdef _XBOX
    // /*
    // Hunk_Clear();

    // extern void CM_Free(void);
    // CM_Free();
    // */

    // //Save visibility info as it has already been set.
    // SPARC<byte> *vis = tr.externalVisData;
    // #endif

    // // clear all our internal state
    // Com_Memset( &tr, 0, sizeof( tr ) );
    // Com_Memset( &backEnd, 0, sizeof( backEnd ) );
    // #ifndef DEDICATED
    // Com_Memset( &tess, 0, sizeof( tess ) );
    // #endif

    // #ifdef _XBOX
    // //Restore visibility info.
    // tr.externalVisData = vis;
    // #endif

    // //	Swap_Init();

    // #ifndef DEDICATED
    // #ifndef FINAL_BUILD
    // if ( (int)tess.xyz & 15 ) {
    // Com_Printf( "WARNING: tess.xyz not 16 byte aligned (%x)\n",(int)tess.xyz & 15 );
    // }
    // #endif
    // #endif
    // //
    // // init function tables
    // //
    // for ( i = 0; i < FUNCTABLE_SIZE; i++ )
    // {
    // tr.sinTable[i]		= sin( DEG2RAD( i * 360.0f / ( ( float ) ( FUNCTABLE_SIZE - 1 ) ) ) );
    // tr.squareTable[i]	= ( i < FUNCTABLE_SIZE/2 ) ? 1.0f : -1.0f;
    // tr.sawToothTable[i] = (float)i / FUNCTABLE_SIZE;
    // tr.inverseSawToothTable[i] = 1.0f - tr.sawToothTable[i];

    // if ( i < FUNCTABLE_SIZE / 2 )
    // {
    // if ( i < FUNCTABLE_SIZE / 4 )
    // {
    // tr.triangleTable[i] = ( float ) i / ( FUNCTABLE_SIZE / 4 );
    // }
    // else
    // {
    // tr.triangleTable[i] = 1.0f - tr.triangleTable[i-FUNCTABLE_SIZE / 4];
    // }
    // }
    // else
    // {
    // tr.triangleTable[i] = -tr.triangleTable[i-FUNCTABLE_SIZE/2];
    // }
    // }
    // #ifndef DEDICATED
    // R_InitFogTable();

    // R_NoiseInit();
    // #endif
    // R_Register();

    // max_polys = r_maxpolys->integer;
    // if (max_polys < MAX_POLYS)
    // max_polys = MAX_POLYS;

    // max_polyverts = r_maxpolyverts->integer;
    // if (max_polyverts < MAX_POLYVERTS)
    // max_polyverts = MAX_POLYVERTS;

    // ptr = (unsigned char *)Hunk_Alloc( sizeof( *backEndData ) + sizeof(srfPoly_t) * max_polys + sizeof(polyVert_t) * max_polyverts, h_low);
    // backEndData = (backEndData_t *) ptr;
    // backEndData->polys = (srfPoly_t *) ((char *) ptr + sizeof( *backEndData ));
    // backEndData->polyVerts = (polyVert_t *) ((char *) ptr + sizeof( *backEndData ) + sizeof(srfPoly_t) * max_polys);
    // #ifndef DEDICATED
    // R_ToggleSmpFrame();

    // for(i = 0; i < MAX_LIGHT_STYLES; i++)
    // {
    // RE_SetLightStyle(i, -1);
    // }
    // InitOpenGL();

    // R_InitImages();
    // R_InitShaders(qfalse);
    // R_InitSkins();

    // R_TerrainInit(); //rwwRMG - added

    // R_InitFonts();
    // #endif
    // R_ModelInit();
    // G2VertSpaceServer = &CMiniHeap_singleton;
    // #ifndef DEDICATED
    // R_InitDecals ( );

    // R_InitWorldEffects();

    // int	err = qglGetError();
    // if ( err != GL_NO_ERROR )
    // Com_Printf ( "glGetError() = 0x%x\n", err);
    // #endif
    // //	Com_Printf ("----- finished R_Init -----\n" );
}

/*
===============
RE_Shutdown
===============
*/
pub fn RE_Shutdown(destroyWindow: bool) {

    //	Com_Printf ("RE_Shutdown( %i )\n", destroyWindow );

    // Cmd_RemoveCommand ("imagelist");
    // Cmd_RemoveCommand ("shaderlist");
    // Cmd_RemoveCommand ("skinlist");
    // Cmd_RemoveCommand ("screenshot");
    // Cmd_RemoveCommand ("screenshot_tga");
    // Cmd_RemoveCommand ("gfxinfo");
    // Cmd_RemoveCommand ("r_atihack");
    // Cmd_RemoveCommand ("r_we");
    // Cmd_RemoveCommand ("imagecacheinfo");
    // Cmd_RemoveCommand ("modellist");
    // Cmd_RemoveCommand ("modelist");
    // Cmd_RemoveCommand ("modelcacheinfo");
    // #ifndef DEDICATED

    // #ifndef _XBOX	// GLOWXXX
    // if ( r_DynamicGlow && r_DynamicGlow->integer )
    // {
    // // Release the Glow Vertex Shader.
    // if ( tr.glowVShader )
    // {
    // qglDeleteProgramsARB( 1, &tr.glowVShader );
    // }

    // // Release Pixel Shader.
    // if ( tr.glowPShader )
    // {
    // if ( qglCombinerParameteriNV  )
    // {
    // // Release the Glow Regcom call list.
    // qglDeleteLists( tr.glowPShader, 1 );
    // }
    // else if ( qglGenProgramsARB )
    // {
    // // Release the Glow Fragment Shader.
    // qglDeleteProgramsARB( 1, &tr.glowPShader );
    // }
    // }

    // // Release the scene glow texture.
    // qglDeleteTextures( 1, &tr.screenGlow );

    // // Release the scene texture.
    // qglDeleteTextures( 1, &tr.sceneImage );

    // // Release the blur texture.
    // qglDeleteTextures( 1, &tr.blurImage );
    // }
    // #endif

    // R_TerrainShutdown(); //rwwRMG - added

    // R_ShutdownFonts();
    // if ( tr.registered ) {
    // R_SyncRenderThread();
    // R_ShutdownCommandBuffers();
    // //#ifndef _XBOX
    // if (destroyWindow)
    // //#endif
    // {
    // R_DeleteTextures();		// only do this for vid_restart now, not during things like map load
    // }
    // }

    // // shut down platform specific OpenGL stuff
    // if ( destroyWindow ) {
    // GLimp_Shutdown();
    // }
    // #endif //!DEDICATED

    // tr.registered = qfalse;
}

#[cfg(not(feature = "dedicated"))]

/*
=============
RE_EndRegistration

Touch all images to make sure they are resident
=============
*/
pub fn RE_EndRegistration() {
    // R_SyncRenderThread();
    // if (!Sys_LowPhysicalMemory()) {
    // #ifndef _XBOX
    // RB_ShowImages();
    // #endif
    // }
}

#[cfg(not(feature = "dedicated"))]
pub fn RE_GetLightStyle(style: c_int, color: *mut u32) {
    // if (style >= MAX_LIGHT_STYLES)
    // {
    // Com_Error( ERR_FATAL, "RE_GetLightStyle: %d is out of range", (int)style );
    // return;
    // }

    // *(int *)color = *(int *)styleColors[style];
}

#[cfg(not(feature = "dedicated"))]
pub fn RE_SetLightStyle(style: c_int, color: c_int) {
    // if (style >= MAX_LIGHT_STYLES)
    // {
    // Com_Error( ERR_FATAL, "RE_SetLightStyle: %d is out of range", (int)style );
    // return;
    // }

    // if (*(int*)styleColors[style] != color)
    // {
    // *(int *)styleColors[style] = color;
    // }
}

/*
@@@@@@@@@@@@@@@@@@@@@
GetRefAPI

@@@@@@@@@@@@@@@@@@@@@
*/
#[repr(C)]
pub struct refexport_t {
    // Placeholder for refexport structure
}

pub fn GetRefAPI(apiVersion: c_int) -> *mut refexport_t {
    // static refexport_t	re;

    // Com_Memset( &re, 0, sizeof( re ) );

    // if ( apiVersion != REF_API_VERSION ) {
    // Com_Printf ( "Mismatched REF_API_VERSION: expected %i, got %i\n",
    // REF_API_VERSION, apiVersion );
    // return NULL;
    // }

    // // the RE_ functions are Renderer Entry points

    // re.Shutdown = RE_Shutdown;
    // #ifndef DEDICATED
    // re.BeginRegistration = RE_BeginRegistration;
    // re.RegisterModel = RE_RegisterModel;
    // re.RegisterSkin = RE_RegisterSkin;
    // re.RegisterShader = RE_RegisterShader;
    // re.RegisterShaderNoMip = RE_RegisterShaderNoMip;
    // re.ShaderNameFromIndex = RE_ShaderNameFromIndex;
    // re.LoadWorld = RE_LoadWorldMap;
    // re.SetWorldVisData = RE_SetWorldVisData;
    // re.EndRegistration = RE_EndRegistration;

    // re.BeginFrame = RE_BeginFrame;
    // re.EndFrame = RE_EndFrame;

    // re.MarkFragments = R_MarkFragments;
    // re.LerpTag = R_LerpTag;
    // re.ModelBounds = R_ModelBounds;

    // re.DrawRotatePic = RE_RotatePic;
    // re.DrawRotatePic2 = RE_RotatePic2;

    // re.ClearScene = RE_ClearScene;
    // re.ClearDecals = RE_ClearDecals;
    // re.AddRefEntityToScene = RE_AddRefEntityToScene;
    // re.AddMiniRefEntityToScene = RE_AddMiniRefEntityToScene;
    // re.AddPolyToScene = RE_AddPolyToScene;
    // re.AddDecalToScene = RE_AddDecalToScene;
    // re.LightForPoint = R_LightForPoint;
    // #ifndef VV_LIGHTING
    // re.AddLightToScene = RE_AddLightToScene;
    // re.AddAdditiveLightToScene = RE_AddAdditiveLightToScene;
    // #endif
    // re.RenderScene = RE_RenderScene;

    // re.SetColor = RE_SetColor;
    // re.DrawStretchPic = RE_StretchPic;
    // re.DrawStretchRaw = RE_StretchRaw;
    // re.UploadCinematic = RE_UploadCinematic;

    // re.RegisterFont = RE_RegisterFont;
    // re.Font_StrLenPixels = RE_Font_StrLenPixels;
    // re.Font_StrLenChars = RE_Font_StrLenChars;
    // re.Font_HeightPixels = RE_Font_HeightPixels;
    // re.Font_DrawString = RE_Font_DrawString;
    // re.Language_IsAsian = Language_IsAsian;
    // re.Language_UsesSpaces = Language_UsesSpaces;
    // re.AnyLanguage_ReadCharFromString = AnyLanguage_ReadCharFromString;

    // re.RemapShader = R_RemapShader;
    // re.GetEntityToken = R_GetEntityToken;
    // re.inPVS = R_inPVS;

    // re.GetLightStyle = RE_GetLightStyle;
    // re.SetLightStyle = RE_SetLightStyle;

    // re.GetBModelVerts = RE_GetBModelVerts;
    // #endif //!DEDICATED
    // return &re;

    core::ptr::null_mut()
}
