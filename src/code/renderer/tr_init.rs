// tr_init.c -- functions that are not called every frame

// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_void, c_char, c_float};
use core::mem;
use core::ptr::{addr_of, addr_of_mut};

// STUB: These are external dependencies that need to be provided by the build environment
extern "C" {
    static mut glConfig: glconfig_t;
    static mut glState: glstate_t;

    fn GfxInfo_f();
    fn R_TerrainInit();
    fn R_TerrainShutdown();

    // cvar pointers - these are assigned from Cvar_Get
    static mut r_verbose: *mut cvar_t;
    static mut r_ignore: *mut cvar_t;
    static mut r_displayRefresh: *mut cvar_t;
    static mut r_detailTextures: *mut cvar_t;
    static mut r_znear: *mut cvar_t;
    static mut r_skipBackEnd: *mut cvar_t;
    static mut r_ignorehwgamma: *mut cvar_t;
    static mut r_measureOverdraw: *mut cvar_t;
    static mut r_fastsky: *mut cvar_t;
    static mut r_drawSun: *mut cvar_t;
    static mut r_dynamiclight: *mut cvar_t;
    static mut r_dlightBacks: *mut cvar_t;
    static mut r_lodbias: *mut cvar_t;
    static mut r_lodscale: *mut cvar_t;
    static mut r_norefresh: *mut cvar_t;
    static mut r_drawentities: *mut cvar_t;
    static mut r_drawworld: *mut cvar_t;
    static mut r_drawfog: *mut cvar_t;
    static mut r_speeds: *mut cvar_t;
    static mut r_fullbright: *mut cvar_t;
    static mut r_novis: *mut cvar_t;
    static mut r_nocull: *mut cvar_t;
    static mut r_facePlaneCull: *mut cvar_t;
    static mut r_showcluster: *mut cvar_t;
    static mut r_nocurves: *mut cvar_t;
    static mut r_dlightStyle: *mut cvar_t;
    static mut r_surfaceSprites: *mut cvar_t;
    static mut r_surfaceWeather: *mut cvar_t;
    static mut r_windSpeed: *mut cvar_t;
    static mut r_windAngle: *mut cvar_t;
    static mut r_windGust: *mut cvar_t;
    static mut r_windDampFactor: *mut cvar_t;
    static mut r_windPointForce: *mut cvar_t;
    static mut r_windPointX: *mut cvar_t;
    static mut r_windPointY: *mut cvar_t;
    static mut r_allowExtensions: *mut cvar_t;
    static mut r_ext_compressed_textures: *mut cvar_t;
    static mut r_ext_compressed_lightmaps: *mut cvar_t;
    static mut r_ext_preferred_tc_method: *mut cvar_t;
    static mut r_ext_gamma_control: *mut cvar_t;
    static mut r_ext_multitexture: *mut cvar_t;
    static mut r_ext_compiled_vertex_array: *mut cvar_t;
    static mut r_ext_texture_env_add: *mut cvar_t;
    static mut r_ext_texture_filter_anisotropic: *mut cvar_t;
    static mut r_DynamicGlow: *mut cvar_t;
    static mut r_DynamicGlowPasses: *mut cvar_t;
    static mut r_DynamicGlowDelta: *mut cvar_t;
    static mut r_DynamicGlowIntensity: *mut cvar_t;
    static mut r_DynamicGlowSoft: *mut cvar_t;
    static mut r_DynamicGlowWidth: *mut cvar_t;
    static mut r_DynamicGlowHeight: *mut cvar_t;

    // Point sprite support.
    static mut r_ext_point_parameters: *mut cvar_t;
    static mut r_ext_nv_point_sprite: *mut cvar_t;

    static mut r_ignoreGLErrors: *mut cvar_t;
    static mut r_logFile: *mut cvar_t;

    static mut r_stencilbits: *mut cvar_t;
    static mut r_depthbits: *mut cvar_t;
    static mut r_colorbits: *mut cvar_t;
    static mut r_stereo: *mut cvar_t;
    static mut r_primitives: *mut cvar_t;
    static mut r_texturebits: *mut cvar_t;
    static mut r_texturebitslm: *mut cvar_t;

    static mut r_lightmap: *mut cvar_t;
    static mut r_vertexLight: *mut cvar_t;
    static mut r_shadows: *mut cvar_t;
    static mut r_shadowRange: *mut cvar_t;
    static mut r_flares: *mut cvar_t;
    static mut r_mode: *mut cvar_t;
    static mut r_nobind: *mut cvar_t;
    static mut r_singleShader: *mut cvar_t;
    static mut r_colorMipLevels: *mut cvar_t;
    static mut r_picmip: *mut cvar_t;
    static mut r_showtris: *mut cvar_t;
    static mut r_showtriscolor: *mut cvar_t;
    static mut r_showsky: *mut cvar_t;
    static mut r_shownormals: *mut cvar_t;
    static mut r_finish: *mut cvar_t;
    static mut r_clear: *mut cvar_t;
    static mut r_swapInterval: *mut cvar_t;
    static mut r_textureMode: *mut cvar_t;
    static mut r_offsetFactor: *mut cvar_t;
    static mut r_offsetUnits: *mut cvar_t;
    static mut r_gamma: *mut cvar_t;
    static mut r_intensity: *mut cvar_t;
    static mut r_lockpvs: *mut cvar_t;
    static mut r_noportals: *mut cvar_t;
    static mut r_portalOnly: *mut cvar_t;

    static mut r_subdivisions: *mut cvar_t;
    static mut r_lodCurveError: *mut cvar_t;

    static mut r_fullscreen: *mut cvar_t;

    static mut r_customwidth: *mut cvar_t;
    static mut r_customheight: *mut cvar_t;

    static mut r_overBrightBits: *mut cvar_t;

    static mut r_debugSurface: *mut cvar_t;
    static mut r_simpleMipMaps: *mut cvar_t;

    static mut r_showImages: *mut cvar_t;

    static mut r_ambientScale: *mut cvar_t;
    static mut r_directedScale: *mut cvar_t;
    static mut r_debugLight: *mut cvar_t;
    static mut r_debugSort: *mut cvar_t;
    static mut r_debugStyle: *mut cvar_t;

    static mut r_modelpoolmegs: *mut cvar_t;

    #[cfg(target_os = "xbox")]
    static mut r_hdreffect: *mut cvar_t;
    #[cfg(target_os = "xbox")]
    static mut r_sundir_x: *mut cvar_t;
    #[cfg(target_os = "xbox")]
    static mut r_sundir_y: *mut cvar_t;
    #[cfg(target_os = "xbox")]
    static mut r_sundir_z: *mut cvar_t;
    #[cfg(target_os = "xbox")]
    static mut r_hdrbloom: *mut cvar_t;
    #[cfg(target_os = "xbox")]
    static mut r_hdrcutoff: *mut cvar_t;

    /*
    Ghoul2 Insert Start
    */

    static mut r_noGhoul2: *mut cvar_t;
    static mut r_Ghoul2AnimSmooth: *mut cvar_t;
    static mut r_Ghoul2UnSqash: *mut cvar_t;
    static mut r_Ghoul2TimeBase: *mut cvar_t;
    static mut r_Ghoul2NoLerp: *mut cvar_t;
    static mut r_Ghoul2NoBlend: *mut cvar_t;
    static mut r_Ghoul2BlendMultiplier: *mut cvar_t;
    static mut r_Ghoul2UnSqashAfterSmooth: *mut cvar_t;

    static mut broadsword: *mut cvar_t;
    static mut broadsword_kickbones: *mut cvar_t;
    static mut broadsword_kickorigin: *mut cvar_t;
    static mut broadsword_playflop: *mut cvar_t;
    static mut broadsword_dontstopanim: *mut cvar_t;
    static mut broadsword_waitforshot: *mut cvar_t;
    static mut broadsword_smallbbox: *mut cvar_t;
    static mut broadsword_extra1: *mut cvar_t;
    static mut broadsword_extra2: *mut cvar_t;

    static mut broadsword_effcorr: *mut cvar_t;
    static mut broadsword_ragtobase: *mut cvar_t;
    static mut broadsword_dircap: *mut cvar_t;

    /*
    Ghoul2 Insert End
    */

    // GL function pointers
    static mut qglMultiTexCoord2fARB: Option<extern "C" fn(GLenum, c_float, c_float)>;
    static mut qglActiveTextureARB: Option<extern "C" fn(GLenum)>;
    static mut qglClientActiveTextureARB: Option<extern "C" fn(GLenum)>;

    static mut qglLockArraysEXT: Option<extern "C" fn(c_int, c_int)>;
    static mut qglUnlockArraysEXT: Option<extern "C" fn()>;

    static mut qglPointParameterfEXT: Option<extern "C" fn(GLenum, c_float)>;
    static mut qglPointParameterfvEXT: Option<extern "C" fn(GLenum, *mut c_float)>;

    // Added 10/23/02 by Aurelio Reis.
    static mut qglPointParameteriNV: Option<extern "C" fn(GLenum, c_int)>;
    static mut qglPointParameterivNV: Option<extern "C" fn(GLenum, *const c_int)>;

    fn VID_Printf(print_level: c_int, fmt: *const c_char, ...);
    fn Cvar_Set(name: *const c_char, value: *const c_char);
    fn Cvar_Get(name: *const c_char, default_val: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_VariableString(name: *const c_char) -> *const c_char;
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Com_Error(error_level: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn Cmd_Argc() -> c_int;
    fn Cmd_AddCommand(name: *const c_char, func: unsafe extern "C" fn());
    fn Cmd_RemoveCommand(name: *const c_char);
    fn FS_ReadFile(filename: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn FS_WriteFile(filename: *const c_char, buf: *const c_void, size: c_int);
    fn Z_Malloc(size: c_int, tag: c_int, zero: qboolean) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn GLimp_Init();
    fn GLimp_Shutdown();
    fn GLimp_EndFrame();
    fn GL_SetDefaultState();
    fn GL_Bind(image: *mut image_t);
    fn GL_State(state_bits: c_int);
    fn GL_SelectTexture(texture: c_int);
    fn GL_TextureMode(mode: *const c_char);
    fn GL_TexEnv(env: c_int);
    fn qglGetError() -> c_int;
    fn qglClearDepth(depth: c_float);
    fn qglCullFace(mode: c_int);
    fn qglColor4f(r: c_float, g: c_float, b: c_float, a: c_float);
    fn qglDisable(cap: c_int);
    fn qglEnable(cap: c_int);
    fn qglShadeModel(mode: c_int);
    fn qglDepthFunc(func: c_int);
    fn qglEnableClientState(array: c_int);
    fn qglPolygonMode(face: c_int, mode: c_int);
    fn qglDepthMask(flag: c_int);
    fn qglBlendFunc(sfactor: c_int, dfactor: c_int);
    fn qglBegin(mode: c_int);
    fn qglEnd();
    fn qglTexCoord2f(s: c_float, t: c_float);
    fn qglVertex2f(x: c_float, y: c_float);
    fn qglReadPixels(x: c_int, y: c_int, width: c_int, height: c_int, format: c_int, ptype: c_int, pixels: *mut c_void);
    fn qglDeleteTextures(n: c_int, textures: *const c_int);
    fn qglDeleteProgramsARB(n: c_int, programs: *const c_int);
    fn qglDeleteLists(list: c_int, range: c_int);
    fn R_FindImageFile(name: *const c_char, unused1: qboolean, unused2: qboolean, unused3: qboolean, wrap: c_int) -> *mut image_t;
    fn RB_SetGL2D();
    fn R_ImageList_f();
    fn R_ShaderList_f();
    fn R_SkinList_f();
    fn R_Modellist_f();
    fn R_GammaCorrect(buffer: *mut c_void, size: c_int);
    fn SaveJPG(filename: *const c_char, quality: c_int, width: c_int, height: c_int, buffer: *mut c_void);
    fn R_InitFogTable();
    fn R_NoiseInit();
    fn R_ToggleSmpFrame();
    fn R_InitImages();
    fn R_InitShaders();
    fn R_InitSkins();
    fn R_ModelInit();
    fn R_InitFonts();
    fn R_ShutdownCommandBuffers();
    fn R_DeleteTextures();
    fn R_SyncRenderThread();
    fn Sys_LowPhysicalMemory() -> qboolean;
    fn Hunk_Alloc(size: c_int, clear: qboolean) -> *mut c_void;
    fn Hunk_Clear();
    fn CM_Free();
    fn CM_CleanLeafCache();
    fn ShaderEntryPtrs_Clear();
    fn Swap_Init();
    fn memset(s: *mut c_void, c: c_int, n: c_int) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: c_int) -> *mut c_void;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> c_int;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn atof(nptr: *const c_char) -> c_float;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strnicmp(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn RE_RegisterModels_Info_f();
    fn RE_RegisterImages_Info_f();
    fn R_WorldEffect_f();
    fn R_ReloadFonts_f();
    fn R_InitWorldEffects();
    fn R_ShutdownWorldEffects();
    fn RE_BeginRegistration();
    fn RE_RegisterModel(name: *const c_char) -> *mut c_void;
    fn RE_RegisterSkin(name: *const c_char) -> *mut c_void;
    fn RE_GetAnimationCFG(name: *const c_char) -> *mut c_void;
    fn RE_RegisterShader(name: *const c_char) -> *mut c_void;
    fn RE_RegisterShaderNoMip(name: *const c_char) -> *mut c_void;
    fn RE_LoadWorldMap(name: *const c_char);
    fn RE_SetWorldVisData(vis: *const c_void);
    fn RE_RegisterMedia_LevelLoadBegin();
    fn RE_RegisterMedia_LevelLoadEnd();
    fn RE_BeginFrame();
    fn RE_EndFrame();
    fn RE_ProcessDissolve();
    fn RE_InitDissolve();
    fn R_MarkFragments();
    fn R_LerpTag();
    fn R_ModelBounds();
    fn RE_ClearScene();
    fn RE_AddRefEntityToScene();
    fn RE_GetLighting();
    fn RE_AddPolyToScene();
    fn RE_AddLightToScene();
    fn RE_RenderScene();
    fn RE_SetColor();
    fn RE_StretchPic();
    fn RE_StretchRaw();
    fn RE_UploadCinematic();
    fn RE_RotatePic();
    fn RE_RotatePic2();
    fn RE_LAGoggles();
    fn RE_Scissor();
    fn RE_GetScreenShot();
    #[cfg(not(target_os = "xbox"))]
    fn RE_TempRawImage_ReadFromFile();
    fn RE_TempRawImage_CleanUp();
    fn RE_GetBModelVerts();
    fn RE_RegisterFont(font_name: *const c_char) -> *mut c_void;
    #[cfg(not(target_os = "xbox"))]
    fn RE_Font_StrLenPixels(text: *const c_char, font: *const c_void) -> c_int;
    #[cfg(not(target_os = "xbox"))]
    fn RE_Font_HeightPixels(font: *const c_void) -> c_int;
    #[cfg(not(target_os = "xbox"))]
    fn RE_Font_DrawString(x: c_int, y: c_int, text: *const c_char, font: *const c_void);
    fn RE_Font_StrLenChars(text: *const c_char) -> c_int;
    fn Language_IsAsian() -> qboolean;
    fn Language_UsesSpaces() -> qboolean;
    fn AnyLanguage_ReadCharFromString() -> c_int;

    // Global structs that are extern
    static mut tr: trGlobals_t;
    static mut backEnd: backEndState_t;
    static mut tess: tessVertexBuffer_t;
    static mut backEndData: *mut backEndData_t;
    static mut styleColors: [color4ub_t; 64];
    static mut styleUpdated: [bool; 64];

    static mut g_bTextureRectangleHack: bool;

    #[cfg(target_os = "xbox")]
    static mut vidRestartReloadMap: qboolean;
}

// Stub types - these need to be provided by the build environment
#[repr(C)]
pub struct glconfig_t {
    // Stub for now
}

#[repr(C)]
pub struct glstate_t {
    // Stub for now
}

#[repr(C)]
pub struct cvar_t {
    // Stub for now
}

#[repr(C)]
pub struct image_t {
    // Stub for now
}

#[repr(C)]
pub struct color4ub_t {
    pub data: [u8; 4],
}

#[repr(C)]
pub struct trGlobals_t {
    // Stub for now
}

#[repr(C)]
pub struct backEndState_t {
    // Stub for now
}

#[repr(C)]
pub struct tessVertexBuffer_t {
    // Stub for now
}

#[repr(C)]
pub struct backEndData_t {
    // Stub for now
}

#[repr(C)]
pub struct vidmode_s {
    pub description: *const c_char,
    pub width: c_int,
    pub height: c_int,
}

type vidmode_t = vidmode_s;

type qboolean = c_int;

const PRINT_WARNING: c_int = 1;
const PRINT_ALL: c_int = 0;
const CVAR_ROM: c_int = 0x80;
const CVAR_ARCHIVE: c_int = 0x1;
const CVAR_LATCH: c_int = 0x2;
const CVAR_CHEAT: c_int = 0x4;
const CVAR_TEMP: c_int = 0x20;
const TAG_TEMP_WORKSPACE: c_int = 0;
const GL_CLAMP: c_int = 0x2900;
const GL_SRCBLEND_ONE: c_int = 0x1;
const GL_DSTBLEND_ZERO: c_int = 0x0;
const GLS_SRCBLEND_ONE: c_int = 0x1;
const GLS_DSTBLEND_ZERO: c_int = 0x0;
const GLS_DEPTHTEST_DISABLE: c_int = 0x1;
const GLS_DEPTHMASK_TRUE: c_int = 0x2;
const GL_TRIANGLE_STRIP: c_int = 5;
const GL_FRONT: c_int = 0x0404;
const GL_FRONT_AND_BACK: c_int = 0x0408;
const GL_FILL: c_int = 0x1B02;
const GL_TRUE: c_int = 1;
const GL_FALSE: c_int = 0;
const GL_DEPTH_TEST: c_int = 0x0B71;
const GL_SCISSOR_TEST: c_int = 0x0C11;
const GL_CULL_FACE: c_int = 0x0B44;
const GL_BLEND: c_int = 0x0BE2;
const GL_ALPHA_TEST: c_int = 0x0BC0;
const GL_SRC_ALPHA: c_int = 0x0302;
const GL_ONE_MINUS_SRC_ALPHA: c_int = 0x0303;
const GL_TEXTURE_2D: c_int = 0x0DE1;
const GL_MODULATE: c_int = 0x2100;
const GL_SMOOTH: c_int = 0x1D01;
const GL_LEQUAL: c_int = 0x0203;
const GL_VERTEX_ARRAY: c_int = 0x8074;
const GL_NO_ERROR: c_int = 0;
const GL_INVALID_ENUM: c_int = 0x0500;
const GL_INVALID_VALUE: c_int = 0x0501;
const GL_INVALID_OPERATION: c_int = 0x0502;
const GL_STACK_OVERFLOW: c_int = 0x0503;
const GL_STACK_UNDERFLOW: c_int = 0x0504;
const GL_OUT_OF_MEMORY: c_int = 0x0505;
const GL_RGBA: c_int = 0x1908;
const GL_RGB: c_int = 0x1907;
const GL_UNSIGNED_BYTE: c_int = 0x1401;
const GL_LIGHTING: c_int = 0x0B50;
const MAX_OSPATH: c_int = 256;
const LEVELSHOTSIZE: c_int = 256;
const ERR_FATAL: c_int = 1;
const FUNCTABLE_SIZE: c_int = 1024;
const MAX_LIGHT_STYLES: c_int = 64;
const REF_API_VERSION: c_int = 8;
const DEG2RAD: f32 = std::f32::consts::PI / 180.0;
const TC_NONE: c_int = 0;

#[allow(non_snake_case)]
unsafe fn AssertCvarRange(cv: *mut cvar_t, minVal: c_float, maxVal: c_float, shouldBeIntegral: qboolean, shouldBeMult2: qboolean)
{
    if shouldBeIntegral != 0
    {
        if (*cv).value as c_int != (*cv).integer
        {
            VID_Printf( PRINT_WARNING, b"WARNING: cvar '%s' must be integral (%f)\n\0".as_ptr() as *const c_char, (*cv).name, (*cv).value );
            Cvar_Set( (*cv).name, va( b"%d\0".as_ptr() as *const c_char, (*cv).integer ) );
        }
    }

    if (*cv).value < minVal
    {
        VID_Printf( PRINT_WARNING, b"WARNING: cvar '%s' out of range (%f < %f)\n\0".as_ptr() as *const c_char, (*cv).name, (*cv).value, minVal );
        Cvar_Set( (*cv).name, va( b"%f\0".as_ptr() as *const c_char, minVal ) );
    }
    else if (*cv).value > maxVal
    {
        VID_Printf( PRINT_WARNING, b"WARNING: cvar '%s' out of range (%f > %f)\n\0".as_ptr() as *const c_char, (*cv).name, (*cv).value, maxVal );
        Cvar_Set( (*cv).name, va( b"%f\0".as_ptr() as *const c_char, maxVal ) );
    }

    if shouldBeMult2 != 0
    {
        if ( ((*cv).integer & ((*cv).integer - 1)) != 0 )
        {
            let mut newvalue: c_int;
            newvalue = 1;
            while newvalue < (*cv).integer {
                newvalue <<= 1;
            }
            VID_Printf( PRINT_WARNING, b"WARNING: cvar '%s' must be multiple of 2(%f)\n\0".as_ptr() as *const c_char, (*cv).name, (*cv).value );
            Cvar_Set( (*cv).name, va( b"%d\0".as_ptr() as *const c_char, newvalue ) );
        }
    }
}

unsafe fn va(fmt: *const c_char, args: ...) -> *const c_char {
    // STUB: This needs a proper implementation using variadic argument handling
    fmt
}

unsafe fn R_Splash()
{
    #[cfg(not(target_os = "xbox"))]
    {
        let mut pImage: *mut image_t;
        /*
        const char* s = Cvar_VariableString("se_language");
        if (stricmp(s,"english"))
        {
            pImage = R_FindImageFile( "menu/splash_eur", qfalse, qfalse, qfalse, GL_CLAMP);
        }
        else
        {
            pImage = R_FindImageFile( "menu/splash", qfalse, qfalse, qfalse, GL_CLAMP);
        }
        */
        pImage = R_FindImageFile( b"menu/splash\0".as_ptr() as *const c_char, 0, 0, 0, GL_CLAMP);

        RB_SetGL2D();
        if !pImage.is_null() {//invalid paths?
            GL_Bind( pImage );
        }
        GL_State(GLS_SRCBLEND_ONE | GLS_DSTBLEND_ZERO);

        let width: c_int = 640;
        let height: c_int = 480;
        let x1: c_float = 320.0 - width as c_float / 2.0;
        let x2: c_float = 320.0 + width as c_float / 2.0;
        let y1: c_float = 240.0 - height as c_float / 2.0;
        let y2: c_float = 240.0 + height as c_float / 2.0;


        qglBegin (GL_TRIANGLE_STRIP);
            qglTexCoord2f( 0.0,  0.0 );
            qglVertex2f(x1, y1);
            qglTexCoord2f( 1.0 ,  0.0 );
            qglVertex2f(x2, y1);
            qglTexCoord2f( 0.0, 1.0 );
            qglVertex2f(x1, y2);
            qglTexCoord2f( 1.0, 1.0 );
            qglVertex2f(x2, y2);
        qglEnd();

        GLimp_EndFrame();
    }
}

/*
** InitOpenGL
**
** This function is responsible for initializing a valid OpenGL subsystem.  This
** is done by calling GLimp_Init (which gives us a working OGL subsystem) then
** setting variables, checking GL constants, and reporting the gfx system config
** to the user.
*/
unsafe fn InitOpenGL()
{
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

    if (*addr_of!(glConfig)).vidWidth == 0
    {
        GLimp_Init();
        // print info the first time only
        // set default state
        GL_SetDefaultState();
        R_Splash();	//get something on screen asap
        GfxInfo_f();
    }
    else
    {
        // set default state
        GL_SetDefaultState();
    }
}

/*
==================
GL_CheckErrors
==================
*/
pub unsafe fn GL_CheckErrors() {
    let mut err: c_int;
    let mut s: [c_char; 64] = [0; 64];

    err = qglGetError();
    if err == GL_NO_ERROR {
        return;
    }
    if (*r_ignoreGLErrors).integer != 0 {
        return;
    }
    match err {
        GL_INVALID_ENUM => {
            strcpy(s.as_mut_ptr(), b"GL_INVALID_ENUM\0".as_ptr() as *const c_char);
        },
        GL_INVALID_VALUE => {
            strcpy(s.as_mut_ptr(), b"GL_INVALID_VALUE\0".as_ptr() as *const c_char);
        },
        GL_INVALID_OPERATION => {
            strcpy(s.as_mut_ptr(), b"GL_INVALID_OPERATION\0".as_ptr() as *const c_char);
        },
        GL_STACK_OVERFLOW => {
            strcpy(s.as_mut_ptr(), b"GL_STACK_OVERFLOW\0".as_ptr() as *const c_char);
        },
        GL_STACK_UNDERFLOW => {
            strcpy(s.as_mut_ptr(), b"GL_STACK_UNDERFLOW\0".as_ptr() as *const c_char);
        },
        GL_OUT_OF_MEMORY => {
            strcpy(s.as_mut_ptr(), b"GL_OUT_OF_MEMORY\0".as_ptr() as *const c_char);
        },
        _ => {
            Com_sprintf( s.as_mut_ptr(), 64, b"%i\0".as_ptr() as *const c_char, err);
        },
    }

    Com_Error( ERR_FATAL, b"GL_CheckErrors: %s\0".as_ptr() as *const c_char, s.as_ptr() );
}

#[cfg(not(target_os = "xbox"))]
{
    /*
    ** R_GetModeInfo
    */
    const r_vidModes: &[vidmode_t] = &[
        vidmode_s { description: b"Mode  0: 320x240\0".as_ptr() as *const c_char,		width: 320,	height: 240 },
        vidmode_s { description: b"Mode  1: 400x300\0".as_ptr() as *const c_char,		width: 400,	height: 300 },
        vidmode_s { description: b"Mode  2: 512x384\0".as_ptr() as *const c_char,		width: 512,	height: 384 },
        vidmode_s { description: b"Mode  3: 640x480\0".as_ptr() as *const c_char,		width: 640,	height: 480 },
        vidmode_s { description: b"Mode  4: 800x600\0".as_ptr() as *const c_char,		width: 800,	height: 600 },
        vidmode_s { description: b"Mode  5: 960x720\0".as_ptr() as *const c_char,		width: 960,	height: 720 },
        vidmode_s { description: b"Mode  6: 1024x768\0".as_ptr() as *const c_char,		width: 1024,	height: 768 },
        vidmode_s { description: b"Mode  7: 1152x864\0".as_ptr() as *const c_char,		width: 1152,	height: 864 },
        vidmode_s { description: b"Mode  8: 1280x1024\0".as_ptr() as *const c_char,		width: 1280,	height: 1024 },
        vidmode_s { description: b"Mode  9: 1600x1200\0".as_ptr() as *const c_char,		width: 1600,	height: 1200 },
        vidmode_s { description: b"Mode 10: 2048x1536\0".as_ptr() as *const c_char,		width: 2048,	height: 1536 },
        vidmode_s { description: b"Mode 11: 856x480 (wide)\0".as_ptr() as *const c_char, width: 856,	height: 480 },
        vidmode_s { description: b"Mode 12: 2400x600(surround)\0".as_ptr() as *const c_char,  width: 2400, height: 600 }
    ];
    static s_numVidModes: c_int = 13;

    pub unsafe fn R_GetModeInfo(width: *mut c_int, height: *mut c_int, mode: c_int) -> qboolean {
        let vm: *const vidmode_t;

        if mode < -1 {
            return 0;
        }
        if mode >= s_numVidModes {
            return 0;
        }

        if mode == -1 {
            *width = (*r_customwidth).integer;
            *height = (*r_customheight).integer;
            return 1;
        }

        vm = &r_vidModes[mode as usize];

        *width  = (*vm).width;
        *height = (*vm).height;

        return 1;
    }

    /*
    ** R_ModeList_f
    */
    unsafe fn R_ModeList_f()
    {
        let mut i: c_int;

        VID_Printf( PRINT_ALL, b"\n\0".as_ptr() as *const c_char );
        for i in 0..s_numVidModes
        {
            VID_Printf( PRINT_ALL, b"%s\n\0".as_ptr() as *const c_char, r_vidModes[i as usize].description );
        }
        VID_Printf( PRINT_ALL, b"\n\0".as_ptr() as *const c_char );
    }
}

/*
==============================================================================

						SCREEN SHOTS

==============================================================================
*/

/*
==================
R_TakeScreenshot
==================
*/
// "filename" param is something like "screenshots/shot0000.tga"
//	note that if the last extension is ".jpg", then it'll save a JPG, else TGA
//
pub unsafe fn R_TakeScreenshot(x: c_int, y: c_int, width: c_int, height: c_int, fileName: *mut c_char) {
    #[cfg(not(target_os = "xbox"))]
    {
        let mut buffer: *mut c_char;
        let mut i: c_int;
        let mut c: c_int;
        let mut temp: c_int;

        let bSaveAsJPG: qboolean = if strnicmp(&(*fileName.offset(strlen(fileName) as isize - 4)), b".jpg\0".as_ptr() as *const c_char, 4) == 0 { 1 } else { 0 };

        if bSaveAsJPG != 0
        {
            // JPG saver expects to be fed RGBA data, though it presumably ignores 'A'...
            //
            buffer = Z_Malloc((*addr_of!(glConfig)).vidWidth*(*addr_of!(glConfig)).vidHeight*4, TAG_TEMP_WORKSPACE, 0) as *mut c_char;
            qglReadPixels( x, y, width, height, GL_RGBA, GL_UNSIGNED_BYTE, buffer as *mut c_void );

            // gamma correct
            if tr.overbrightBits>0 && (*addr_of!(glConfig)).deviceSupportsGamma != 0 {
                R_GammaCorrect( buffer as *mut c_void, (*addr_of!(glConfig)).vidWidth * (*addr_of!(glConfig)).vidHeight * 4 );
            }

            SaveJPG(fileName, 95, width, height, buffer as *mut c_void);
        }
        else
        {
            // TGA...
            //
            buffer = Z_Malloc((*addr_of!(glConfig)).vidWidth*(*addr_of!(glConfig)).vidHeight*3 + 18, TAG_TEMP_WORKSPACE, 0) as *mut c_char;
            memset (buffer as *mut c_void, 0, 18);
            *buffer.offset(2) = 2 as c_char;		// uncompressed type
            *buffer.offset(12) = (width & 255) as c_char;
            *buffer.offset(13) = (width >> 8) as c_char;
            *buffer.offset(14) = (height & 255) as c_char;
            *buffer.offset(15) = (height >> 8) as c_char;
            *buffer.offset(16) = 24 as c_char;	// pixel size

            qglReadPixels( x, y, width, height, GL_RGB, GL_UNSIGNED_BYTE, buffer.offset(18) as *mut c_void );

            // swap rgb to bgr
            c = 18 + width * height * 3;
            i = 18;
            while i < c {
                temp = *buffer.offset(i as isize) as c_int;
                *buffer.offset(i as isize) = *buffer.offset((i+2) as isize);
                *buffer.offset((i+2) as isize) = temp as c_char;
                i += 3;
            }

            // gamma correct
            if tr.overbrightBits>0 && (*addr_of!(glConfig)).deviceSupportsGamma != 0 {
                R_GammaCorrect( buffer.offset(18) as *mut c_void, (*addr_of!(glConfig)).vidWidth * (*addr_of!(glConfig)).vidHeight * 3 );
            }
            FS_WriteFile( fileName, buffer as *const c_void, c );
        }

        Z_Free( buffer as *mut c_void );
    }
}

/*
==================
R_ScreenshotFilename
==================
*/
pub unsafe fn R_ScreenshotFilename(lastNumber: c_int, fileName: *mut c_char, psExt: *const c_char) {
    let mut a: c_int;
    let mut b: c_int;
    let mut c: c_int;
    let mut d: c_int;
    let mut lastNumber = lastNumber;

    if lastNumber < 0 || lastNumber > 9999 {
        Com_sprintf( fileName, MAX_OSPATH, b"screenshots/shot9999%s\0".as_ptr() as *const c_char, psExt );
        return;
    }

    a = lastNumber / 1000;
    lastNumber -= a*1000;
    b = lastNumber / 100;
    lastNumber -= b*100;
    c = lastNumber / 10;
    lastNumber -= c*10;
    d = lastNumber;

    Com_sprintf( fileName, MAX_OSPATH, b"screenshots/shot%i%i%i%i%s\0".as_ptr() as *const c_char
        , a, b, c, d, psExt );
}

/*
====================
R_LevelShot

levelshots are specialized 256*256 thumbnails for
the menu system, sampled down from full screen distorted images
====================
*/
unsafe fn R_LevelShot() {
    #[cfg(not(target_os = "xbox"))]
    {
        let mut checkname: [c_char; 256];
        let mut buffer: *mut c_char;
        let mut source: *mut c_char;
        let mut src: *mut c_char;
        let mut dst: *mut c_char;
        let mut x: c_int;
        let mut y: c_int;
        let mut r: c_int;
        let mut g: c_int;
        let mut b: c_int;
        let mut xScale: c_float;
        let mut yScale: c_float;
        let mut xx: c_int;
        let mut yy: c_int;

        sprintf( checkname.as_mut_ptr(), b"levelshots/%s.tga\0".as_ptr() as *const c_char, (*addr_of!(tr)).worldDir.offset(strlen(b"maps/\0".as_ptr() as *const c_char) as isize) );

        source = Z_Malloc( (*addr_of!(glConfig)).vidWidth * (*addr_of!(glConfig)).vidHeight * 3, TAG_TEMP_WORKSPACE, 0 ) as *mut c_char;

        buffer = Z_Malloc( LEVELSHOTSIZE * LEVELSHOTSIZE*3 + 18, TAG_TEMP_WORKSPACE, 0 ) as *mut c_char;
        memset (buffer as *mut c_void, 0, 18);
        *buffer.offset(2) = 2 as c_char;		// uncompressed type
        *buffer.offset(12) = (LEVELSHOTSIZE & 255) as c_char;
        *buffer.offset(13) = (LEVELSHOTSIZE >> 8) as c_char;
        *buffer.offset(14) = (LEVELSHOTSIZE & 255) as c_char;
        *buffer.offset(15) = (LEVELSHOTSIZE >> 8) as c_char;
        *buffer.offset(16) = 24 as c_char;	// pixel size

        qglReadPixels( 0, 0, (*addr_of!(glConfig)).vidWidth, (*addr_of!(glConfig)).vidHeight, GL_RGB, GL_UNSIGNED_BYTE, source as *mut c_void );

        // resample from source
        xScale = (*addr_of!(glConfig)).vidWidth as c_float / (4.0*LEVELSHOTSIZE as c_float);
        yScale = (*addr_of!(glConfig)).vidHeight as c_float / (3.0*LEVELSHOTSIZE as c_float);
        y = 0;
        while y < LEVELSHOTSIZE {
            x = 0;
            while x < LEVELSHOTSIZE {
                r = 0;
                g = 0;
                b = 0;
                yy = 0;
                while yy < 3 {
                    xx = 0;
                    while xx < 4 {
                        src = source.offset(3 * ( (*addr_of!(glConfig)).vidWidth * (((y*3+yy) as c_float)*yScale) as c_int + (((x*4+xx) as c_float)*xScale) as c_int ));
                        r += *src as c_int;
                        g += *src.offset(1) as c_int;
                        b += *src.offset(2) as c_int;
                        xx += 1;
                    }
                    yy += 1;
                }
                dst = buffer.offset(18 + 3 * ( y * LEVELSHOTSIZE + x ));
                *dst = (b / 12) as c_char;
                *dst.offset(1) = (g / 12) as c_char;
                *dst.offset(2) = (r / 12) as c_char;
                x += 1;
            }
            y += 1;
        }

        // gamma correct
        if (*addr_of!(glConfig)).deviceSupportsGamma != 0 {
            R_GammaCorrect( buffer.offset(18) as *mut c_void, LEVELSHOTSIZE * LEVELSHOTSIZE * 3 );
        }

        FS_WriteFile( checkname.as_ptr(), buffer as *const c_void, LEVELSHOTSIZE * LEVELSHOTSIZE*3 + 18 );

        Z_Free( buffer as *mut c_void );
        Z_Free( source as *mut c_void );

        VID_Printf( PRINT_ALL, b"Wrote %s\n\0".as_ptr() as *const c_char, checkname.as_ptr() );
    }
}

/*
==================
R_ScreenShot_f

screenshot
screenshot [silent]
screenshot [levelshot]
screenshot [filename]

Doesn't print the pacifier message if there is a second arg
==================
*/
unsafe fn R_ScreenShot_f () {
    #[cfg(not(target_os = "xbox"))]
    {
        let mut checkname: [c_char; 256];
        let mut len: c_int;
        static mut lastNumber: c_int = -1;
        let mut silent: qboolean;

        if strcmp( Cmd_Argv(1), b"levelshot\0".as_ptr() as *const c_char ) == 0 {
            R_LevelShot();
            return;
        }
        if strcmp( Cmd_Argv(1), b"silent\0".as_ptr() as *const c_char ) == 0 {
            silent = 1;
        } else {
            silent = 0;
        }

        if Cmd_Argc() == 2 && silent == 0 {
            // explicit filename
            Com_sprintf( checkname.as_mut_ptr(), MAX_OSPATH, b"screenshots/%s.jpg\0".as_ptr() as *const c_char, Cmd_Argv( 1 ) );
        } else {
            // scan for a free filename

            // if we have saved a previous screenshot, don't scan
            // again, because recording demo avis can involve
            // thousands of shots
            if lastNumber == -1 {
                // scan for a free number
                lastNumber = 0;
                while lastNumber <= 9999 {
                    R_ScreenshotFilename( lastNumber, checkname.as_mut_ptr(), b".jpg\0".as_ptr() as *const c_char );

                    len = FS_ReadFile( checkname.as_ptr(), core::ptr::null_mut() );
                    if len <= 0 {
                        break;	// file doesn't exist
                    }
                    lastNumber += 1;
                }
            } else {
                R_ScreenshotFilename( lastNumber, checkname.as_mut_ptr(), b".jpg\0".as_ptr() as *const c_char );
            }

            if lastNumber == 10000 {
                VID_Printf (PRINT_ALL, b"ScreenShot: Couldn't create a file\n\0".as_ptr() as *const c_char);
                return;
            }

            lastNumber += 1;
        }


        R_TakeScreenshot( 0, 0, (*addr_of!(glConfig)).vidWidth, (*addr_of!(glConfig)).vidHeight, checkname.as_mut_ptr() );

        if silent == 0 {
            VID_Printf (PRINT_ALL, b"Wrote %s\n\0".as_ptr() as *const c_char, checkname.as_ptr());
        }
    }
}



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
unsafe fn R_ScreenShotTGA_f () {
    #[cfg(not(target_os = "xbox"))]
    {
        let mut checkname: [c_char; 256];
        let mut len: c_int;
        static mut lastNumber: c_int = -1;
        let mut silent: qboolean;

        if strcmp( Cmd_Argv(1), b"levelshot\0".as_ptr() as *const c_char ) == 0 {
            R_LevelShot();
            return;
        }
        if strcmp( Cmd_Argv(1), b"silent\0".as_ptr() as *const c_char ) == 0 {
            silent = 1;
        } else {
            silent = 0;
        }

        if Cmd_Argc() == 2 && silent == 0 {
            // explicit filename
            Com_sprintf( checkname.as_mut_ptr(), MAX_OSPATH, b"screenshots/%s.tga\0".as_ptr() as *const c_char, Cmd_Argv( 1 ) );
        } else {
            // scan for a free filename

            // if we have saved a previous screenshot, don't scan
            // again, because recording demo avis can involve
            // thousands of shots
            if lastNumber == -1 {
                // scan for a free number
                lastNumber = 0;
                while lastNumber <= 9999 {
                    R_ScreenshotFilename( lastNumber, checkname.as_mut_ptr(), b".tga\0".as_ptr() as *const c_char );

                    len = FS_ReadFile( checkname.as_ptr(), core::ptr::null_mut() );
                    if len <= 0 {
                        break;	// file doesn't exist
                    }
                    lastNumber += 1;
                }
            } else {
                R_ScreenshotFilename( lastNumber, checkname.as_mut_ptr(), b".tga\0".as_ptr() as *const c_char );
            }

            if lastNumber == 10000 {
                VID_Printf (PRINT_ALL, b"ScreenShot: Couldn't create a file\n\0".as_ptr() as *const c_char);
                return;
            }

            lastNumber += 1;
        }


        R_TakeScreenshot( 0, 0, (*addr_of!(glConfig)).vidWidth, (*addr_of!(glConfig)).vidHeight, checkname.as_mut_ptr() );

        if silent == 0 {
            VID_Printf (PRINT_ALL, b"Wrote %s\n\0".as_ptr() as *const c_char, checkname.as_ptr());
        }
    }
}



//============================================================================

/*
** GL_SetDefaultState
*/
pub unsafe fn GL_SetDefaultState()
{
    qglClearDepth( 1.0 );

    qglCullFace(GL_FRONT);

    qglColor4f (1.0, 1.0, 1.0, 1.0);

    // initialize downstream texture unit if we're running
    // in a multitexture environment
    if !qglActiveTextureARB.is_none() {
        GL_SelectTexture( 1 );
        GL_TextureMode( (*r_textureMode).string );
        GL_TexEnv( GL_MODULATE );
        qglDisable( GL_TEXTURE_2D );
        GL_SelectTexture( 0 );
    }

    qglEnable(GL_TEXTURE_2D);
    GL_TextureMode( (*r_textureMode).string );
    GL_TexEnv( GL_MODULATE );

    qglShadeModel( GL_SMOOTH );
    qglDepthFunc( GL_LEQUAL );

    // the vertex array is always enabled, but the color and texture
    // arrays are enabled and disabled around the compiled vertex array call
    qglEnableClientState (GL_VERTEX_ARRAY);

    //
    // make sure our GL state vector is set correctly
    //
    (*addr_of_mut!(glState)).glStateBits = GLS_DEPTHTEST_DISABLE | GLS_DEPTHMASK_TRUE;

    qglPolygonMode (GL_FRONT_AND_BACK, GL_FILL);
    qglDepthMask( GL_TRUE );
    qglDisable( GL_DEPTH_TEST );
    qglEnable( GL_SCISSOR_TEST );
    qglDisable( GL_CULL_FACE );
    qglDisable( GL_BLEND );
    qglDisable( GL_ALPHA_TEST );
    qglBlendFunc(GL_SRC_ALPHA,GL_ONE_MINUS_SRC_ALPHA);
    #[cfg(target_os = "xbox")]
    qglDisable( GL_LIGHTING );
}


/*
================
GfxInfo_f
================
*/

unsafe fn R_AtiHackToggle_f()
{
    g_bTextureRectangleHack = !g_bTextureRectangleHack;
}

/************************************************************************************************
 * R_FogDistance_f                                                                              *
 *    Console command to change the global fog opacity distance.  If you specify nothing on the *
 *    command line, it will display the current fog opacity distance.  Specifying a float       *
 *    representing the world units away the fog should be completely opaque will change the     *
 *    value.                                                                                    *
 *                                                                                              *
 * Input                                                                                        *
 *    none                                                                                      *
 *                                                                                              *
 * Output / Return                                                                              *
 *    none                                                                                      *
 *                                                                                              *
 ************************************************************************************************/
unsafe fn R_FogDistance_f()
{
    let mut distance: c_float;

    if (*addr_of!(tr)).world.is_null()
    {
        VID_Printf(PRINT_ALL, b"R_FogDistance_f: World is not initialized\n\0".as_ptr() as *const c_char);
        return;
    }

    if (*(*addr_of!(tr)).world).globalFog == -1
    {
        VID_Printf(PRINT_ALL, b"R_FogDistance_f: World does not have a global fog\n\0".as_ptr() as *const c_char);
        return;
    }

    if Cmd_Argc() <= 1
    {
        //		should not ever be 0.0
        //		if (tr.world->fogs[tr.world->globalFog].tcScale == 0.0)
        //		{
        //			distance = 0.0;
        //		}
        //		else
        {
            distance = 1.0 / (8.0 * (*(*addr_of!(tr)).world).fogs[(*(*addr_of!(tr)).world).globalFog as usize].tcScale);
        }

        VID_Printf(PRINT_ALL, b"R_FogDistance_f: Current Distance: %.0f\n\0".as_ptr() as *const c_char, distance);
        return;
    }

    if Cmd_Argc() != 2
    {
        VID_Printf(PRINT_ALL, b"R_FogDistance_f: Invalid number of arguments to set distance\n\0".as_ptr() as *const c_char);
        return;
    }

    distance = atof(Cmd_Argv(1));
    if distance < 1.0
    {
        distance = 1.0;
    }
    (*(*addr_of!(tr)).world).fogs[(*(*addr_of!(tr)).world).globalFog as usize].parms.depthForOpaque = distance;
    (*(*addr_of!(tr)).world).fogs[(*(*addr_of!(tr)).world).globalFog as usize].tcScale = 1.0 / ( distance * 8.0 );
}

/************************************************************************************************
 * R_FogColor_f                                                                                 *
 *    Console command to change the global fog color.  Specifying nothing on the command will   *
 *    display the current global fog color.  Specifying a float R G B values between 0.0 and    *
 *    1.0 will change the fog color.                                                            *
 *                                                                                              *
 * Input                                                                                        *
 *    none                                                                                      *
 *                                                                                              *
 * Output / Return                                                                              *
 *    none                                                                                      *
 *                                                                                              *
 ************************************************************************************************/
unsafe fn R_FogColor_f()
{
    if (*addr_of!(tr)).world.is_null()
    {
        VID_Printf(PRINT_ALL, b"R_FogColor_f: World is not initialized\n\0".as_ptr() as *const c_char);
        return;
    }

    if (*(*addr_of!(tr)).world).globalFog == -1
    {
        VID_Printf(PRINT_ALL, b"R_FogColor_f: World does not have a global fog\n\0".as_ptr() as *const c_char);
        return;
    }

    if Cmd_Argc() <= 1
    {
        let i: c_int = (*(*addr_of!(tr)).world).fogs[(*(*addr_of!(tr)).world).globalFog as usize].colorInt;

        VID_Printf(PRINT_ALL, b"R_FogColor_f: Current Color: %0f %0f %0f\n\0".as_ptr() as *const c_char,
            ( ((&i as *const c_int) as *const u8).offset(0) as *const u8 as c_int) as c_float / 255.0,
            ( ((&i as *const c_int) as *const u8).offset(1) as *const u8 as c_int) as c_float / 255.0,
            ( ((&i as *const c_int) as *const u8).offset(2) as *const u8 as c_int) as c_float / 255.0);
        return;
    }

    if Cmd_Argc() != 4
    {
        VID_Printf(PRINT_ALL, b"R_FogColor_f: Invalid number of arguments to set color\n\0".as_ptr() as *const c_char);
        return;
    }

    (*(*addr_of!(tr)).world).fogs[(*(*addr_of!(tr)).world).globalFog as usize].parms.color[0] = atof(Cmd_Argv(1));
    (*(*addr_of!(tr)).world).fogs[(*(*addr_of!(tr)).world).globalFog as usize].parms.color[1] = atof(Cmd_Argv(2));
    (*(*addr_of!(tr)).world).fogs[(*(*addr_of!(tr)).world).globalFog as usize].parms.color[2] = atof(Cmd_Argv(3));
    (*(*addr_of!(tr)).world).fogs[(*(*addr_of!(tr)).world).globalFog as usize].colorInt = ColorBytes4 ( atof(Cmd_Argv(1)) * (*addr_of!(tr)).identityLight,
            atof(Cmd_Argv(2)) * (*addr_of!(tr)).identityLight,
            atof(Cmd_Argv(3)) * (*addr_of!(tr)).identityLight, 1.0 );
}

fn ColorBytes4(r: c_float, g: c_float, b: c_float, a: c_float) -> c_int {
    // STUB: This needs proper implementation
    0
}

/*
===============
R_Register
===============
*/
pub unsafe fn R_Register()
{
    //
    // latched and archived variables
    //
    r_allowExtensions = Cvar_Get( b"r_allowExtensions\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_ext_compressed_textures = Cvar_Get( b"r_ext_compress_textures\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_ext_compressed_lightmaps = Cvar_Get( b"r_ext_compress_lightmaps\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_ext_preferred_tc_method = Cvar_Get( b"r_ext_preferred_tc_method\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_ext_gamma_control = Cvar_Get( b"r_ext_gamma_control\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_ext_multitexture = Cvar_Get( b"r_ext_multitexture\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_ext_compiled_vertex_array = Cvar_Get( b"r_ext_compiled_vertex_array\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH);
    r_ext_texture_env_add = Cvar_Get( b"r_ext_texture_env_add\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH);
    r_ext_texture_filter_anisotropic = Cvar_Get( b"r_ext_texture_filter_anisotropic\0".as_ptr() as *const c_char, b"16\0".as_ptr() as *const c_char, CVAR_ARCHIVE );

    r_DynamicGlow = Cvar_Get( b"r_DynamicGlow\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_DynamicGlowPasses = Cvar_Get( b"r_DynamicGlowPasses\0".as_ptr() as *const c_char, b"5\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_DynamicGlowDelta  = Cvar_Get( b"r_DynamicGlowDelta\0".as_ptr() as *const c_char, b"0.8f\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_DynamicGlowIntensity = Cvar_Get( b"r_DynamicGlowIntensity\0".as_ptr() as *const c_char, b"1.13f\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_DynamicGlowSoft = Cvar_Get( b"r_DynamicGlowSoft\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_DynamicGlowWidth = Cvar_Get( b"r_DynamicGlowWidth\0".as_ptr() as *const c_char, b"320\0".as_ptr() as *const c_char, CVAR_CHEAT | CVAR_LATCH );
    r_DynamicGlowHeight = Cvar_Get( b"r_DynamicGlowHeight\0".as_ptr() as *const c_char, b"240\0".as_ptr() as *const c_char, CVAR_CHEAT | CVAR_LATCH );

    // Register point sprite stuff here.
    r_ext_point_parameters = Cvar_Get( b"r_ext_point_parameters\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_ext_nv_point_sprite = Cvar_Get( b"r_ext_nv_point_sprite\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE );

    r_picmip = Cvar_Get (b"r_picmip\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_colorMipLevels = Cvar_Get (b"r_colorMipLevels\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_LATCH );
    AssertCvarRange( r_picmip, 0.0, 16.0, 1, 0 );
    r_detailTextures = Cvar_Get( b"r_detailtextures\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_texturebits = Cvar_Get( b"r_texturebits\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_texturebitslm = Cvar_Get( b"r_texturebitslm\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_colorbits = Cvar_Get( b"r_colorbits\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_stereo = Cvar_Get( b"r_stereo\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    #[cfg(target_os = "linux")]
    {
        r_stencilbits = Cvar_Get( b"r_stencilbits\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    }
    #[cfg(not(target_os = "linux"))]
    {
        r_stencilbits = Cvar_Get( b"r_stencilbits\0".as_ptr() as *const c_char, b"8\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    }
    r_depthbits = Cvar_Get( b"r_depthbits\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_overBrightBits = Cvar_Get (b"r_overBrightBits\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_ignorehwgamma = Cvar_Get( b"r_ignorehwgamma\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH);
    r_mode = Cvar_Get( b"r_mode\0".as_ptr() as *const c_char, b"4\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_fullscreen = Cvar_Get( b"r_fullscreen\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_customwidth = Cvar_Get( b"r_customwidth\0".as_ptr() as *const c_char, b"1600\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_customheight = Cvar_Get( b"r_customheight\0".as_ptr() as *const c_char, b"1024\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_simpleMipMaps = Cvar_Get( b"r_simpleMipMaps\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_vertexLight = Cvar_Get( b"r_vertexLight\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
    r_subdivisions = Cvar_Get (b"r_subdivisions\0".as_ptr() as *const c_char, b"4\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_LATCH);
    r_intensity = Cvar_Get (b"r_intensity\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_LATCH|CVAR_ARCHIVE );

    //
    // temporary latched variables that can only change over a restart
    //
    r_displayRefresh = Cvar_Get( b"r_displayRefresh\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_LATCH );
    AssertCvarRange( r_displayRefresh, 0.0, 200.0, 1, 0 );
    r_fullbright = Cvar_Get (b"r_fullbright\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_singleShader = Cvar_Get (b"r_singleShader\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT | CVAR_LATCH );

    //
    // archived variables that can change at any time
    //
    r_lodCurveError = Cvar_Get( b"r_lodCurveError\0".as_ptr() as *const c_char, b"250\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_lodbias = Cvar_Get( b"r_lodbias\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_flares = Cvar_Get (b"r_flares\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_lodscale = Cvar_Get( b"r_lodscale\0".as_ptr() as *const c_char, b"10\0".as_ptr() as *const c_char, CVAR_ARCHIVE );

    #[cfg(target_os = "xbox")]
    {
        r_znear = Cvar_Get( b"r_znear\0".as_ptr() as *const c_char, b"2\0".as_ptr() as *const c_char, CVAR_CHEAT );	//lose a lot of precision in the distance
    }
    #[cfg(not(target_os = "xbox"))]
    {
        r_znear = Cvar_Get( b"r_znear\0".as_ptr() as *const c_char, b"4\0".as_ptr() as *const c_char, CVAR_CHEAT );	//if set any lower, you lose a lot of precision in the distance
    }
    AssertCvarRange( r_znear, 0.001, 200.0, 0, 0 );
    r_ignoreGLErrors = Cvar_Get( b"r_ignoreGLErrors\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_fastsky = Cvar_Get( b"r_fastsky\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_drawSun = Cvar_Get( b"r_drawSun\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_dynamiclight = Cvar_Get( b"r_dynamiclight\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_dlightBacks = Cvar_Get( b"r_dlightBacks\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_finish = Cvar_Get (b"r_finish\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
    r_textureMode = Cvar_Get( b"r_textureMode\0".as_ptr() as *const c_char, b"GL_LINEAR_MIPMAP_NEAREST\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    r_swapInterval = Cvar_Get( b"r_swapInterval\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    #[cfg(target_os = "macos")]
    {
        r_gamma = Cvar_Get( b"r_gamma\0".as_ptr() as *const c_char, b"1.2\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    }
    #[cfg(not(target_os = "macos"))]
    {
        r_gamma = Cvar_Get( b"r_gamma\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE );
    }
    r_facePlaneCull = Cvar_Get (b"r_facePlaneCull\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE );

    r_dlightStyle = Cvar_Get (b"r_dlightStyle\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_TEMP);
    r_surfaceSprites = Cvar_Get (b"r_surfaceSprites\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_TEMP);
    r_surfaceWeather = Cvar_Get (b"r_surfaceWeather\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_TEMP);

    r_windSpeed = Cvar_Get (b"r_windSpeed\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    r_windAngle = Cvar_Get (b"r_windAngle\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    r_windGust = Cvar_Get (b"r_windGust\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    r_windDampFactor = Cvar_Get (b"r_windDampFactor\0".as_ptr() as *const c_char, b"0.1\0".as_ptr() as *const c_char, 0);
    r_windPointForce = Cvar_Get (b"r_windPointForce\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    r_windPointX = Cvar_Get (b"r_windPointX\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    r_windPointY = Cvar_Get (b"r_windPointY\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

    r_primitives = Cvar_Get( b"r_primitives\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE );

    r_ambientScale = Cvar_Get( b"r_ambientScale\0".as_ptr() as *const c_char, b"0.5\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_directedScale = Cvar_Get( b"r_directedScale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_CHEAT );

    //
    // temporary variables that can change at any time
    //
    r_showImages = Cvar_Get( b"r_showImages\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );

    r_debugLight = Cvar_Get( b"r_debuglight\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_TEMP );
    r_debugStyle = Cvar_Get( b"r_debugStyle\0".as_ptr() as *const c_char, b"-1\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_debugSort = Cvar_Get( b"r_debugSort\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );

    r_nocurves = Cvar_Get (b"r_nocurves\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_drawworld = Cvar_Get (b"r_drawworld\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_drawfog = Cvar_Get (b"r_drawfog\0".as_ptr() as *const c_char, b"2\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_lightmap = Cvar_Get (b"r_lightmap\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_portalOnly = Cvar_Get (b"r_portalOnly\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );

    r_skipBackEnd = Cvar_Get (b"r_skipBackEnd\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);

    r_measureOverdraw = Cvar_Get( b"r_measureOverdraw\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_norefresh = Cvar_Get (b"r_norefresh\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_drawentities = Cvar_Get (b"r_drawentities\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_ignore = Cvar_Get( b"r_ignore\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_TEMP );
    r_nocull = Cvar_Get (b"r_nocull\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_novis = Cvar_Get (b"r_novis\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_showcluster = Cvar_Get (b"r_showcluster\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_speeds = Cvar_Get (b"r_speeds\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_verbose = Cvar_Get( b"r_verbose\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_logFile = Cvar_Get( b"r_logFile\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_debugSurface = Cvar_Get (b"r_debugSurface\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_nobind = Cvar_Get (b"r_nobind\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_showtris = Cvar_Get (b"r_showtris\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_showtriscolor = Cvar_Get (b"r_showtriscolor\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
    r_showsky = Cvar_Get (b"r_showsky\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_shownormals = Cvar_Get (b"r_shownormals\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_clear = Cvar_Get (b"r_clear\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_offsetFactor = Cvar_Get( b"r_offsetfactor\0".as_ptr() as *const c_char, b"-1\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_offsetUnits = Cvar_Get( b"r_offsetunits\0".as_ptr() as *const c_char, b"-2\0".as_ptr() as *const c_char, CVAR_CHEAT );
    r_lockpvs = Cvar_Get (b"r_lockpvs\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_noportals = Cvar_Get (b"r_noportals\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_shadows = Cvar_Get( b"cg_shadows\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0 );
    r_shadowRange = Cvar_Get( b"r_shadowRange\0".as_ptr() as *const c_char, b"1000\0".as_ptr() as *const c_char, CVAR_ARCHIVE );

    #[cfg(target_os = "xbox")]
    {
        r_hdreffect = Cvar_Get( b"r_hdreffect\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0 );
        r_sundir_x = Cvar_Get( b"r_sundir_x\0".as_ptr() as *const c_char, b"0.45\0".as_ptr() as *const c_char, 0 );
        r_sundir_y = Cvar_Get( b"r_sundir_y\0".as_ptr() as *const c_char, b"0.3\0".as_ptr() as *const c_char, 0 );
        r_sundir_z = Cvar_Get( b"r_sundir_z\0".as_ptr() as *const c_char, b"0.9\0".as_ptr() as *const c_char, 0 );
        r_hdrbloom = Cvar_Get( b"r_hdrbloom\0".as_ptr() as *const c_char, b"1.0\0".as_ptr() as *const c_char, 0 );
        r_hdrcutoff = Cvar_Get( b"r_hdrcutoff\0".as_ptr() as *const c_char, b"0.5\0".as_ptr() as *const c_char, 0 );
    }
    /*
    Ghoul2 Insert Start
    */
    r_noGhoul2 = Cvar_Get( b"r_noghoul2\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
    r_Ghoul2AnimSmooth = Cvar_Get( b"r_ghoul2animsmooth\0".as_ptr() as *const c_char, b"0.25\0".as_ptr() as *const c_char, 0);
    r_Ghoul2UnSqash = Cvar_Get( b"r_ghoul2unsquash\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
    r_Ghoul2TimeBase = Cvar_Get( b"r_ghoul2timebase\0".as_ptr() as *const c_char, b"2\0".as_ptr() as *const c_char, 0);
    r_Ghoul2NoLerp = Cvar_Get( b"r_ghoul2nolerp\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    r_Ghoul2NoBlend = Cvar_Get( b"r_ghoul2noblend\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    r_Ghoul2BlendMultiplier = Cvar_Get( b"r_ghoul2blendmultiplier\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
    r_Ghoul2UnSqashAfterSmooth = Cvar_Get( b"r_ghoul2unsquashaftersmooth\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);

    broadsword = Cvar_Get( b"broadsword\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
    broadsword_kickbones = Cvar_Get( b"broadsword_kickbones\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
    broadsword_kickorigin = Cvar_Get( b"broadsword_kickorigin\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
    broadsword_dontstopanim = Cvar_Get( b"broadsword_dontstopanim\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    broadsword_waitforshot = Cvar_Get( b"broadsword_waitforshot\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    broadsword_playflop = Cvar_Get( b"broadsword_playflop\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
    broadsword_smallbbox = Cvar_Get( b"broadsword_smallbbox\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    broadsword_extra1 = Cvar_Get( b"broadsword_extra1\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    broadsword_extra2 = Cvar_Get( b"broadsword_extra2\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    broadsword_effcorr = Cvar_Get( b"broadsword_effcorr\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
    broadsword_ragtobase = Cvar_Get( b"broadsword_ragtobase\0".as_ptr() as *const c_char, b"2\0".as_ptr() as *const c_char, 0);
    broadsword_dircap = Cvar_Get( b"broadsword_dircap\0".as_ptr() as *const c_char, b"64\0".as_ptr() as *const c_char, 0);

    /*
    Ghoul2 Insert End
    */
    r_modelpoolmegs = Cvar_Get(b"r_modelpoolmegs\0".as_ptr() as *const c_char, b"20\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
    if Sys_LowPhysicalMemory() != 0
    {
        Cvar_Set(b"r_modelpoolmegs\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
    }

    // make sure all the commands added here are also
    // removed in R_Shutdown
    Cmd_AddCommand( b"imagelist\0".as_ptr() as *const c_char, R_ImageList_f );
    Cmd_AddCommand( b"shaderlist\0".as_ptr() as *const c_char, R_ShaderList_f );
    Cmd_AddCommand( b"skinlist\0".as_ptr() as *const c_char, R_SkinList_f );
    Cmd_AddCommand( b"modellist\0".as_ptr() as *const c_char, R_Modellist_f );
    #[cfg(not(target_os = "xbox"))]
    {
        Cmd_AddCommand( b"modelist\0".as_ptr() as *const c_char, R_ModeList_f );
        Cmd_AddCommand( b"r_atihack\0".as_ptr() as *const c_char, R_AtiHackToggle_f );
    }
    Cmd_AddCommand( b"screenshot\0".as_ptr() as *const c_char, R_ScreenShot_f );
    Cmd_AddCommand( b"screenshot_tga\0".as_ptr() as *const c_char, R_ScreenShotTGA_f );
    Cmd_AddCommand( b"gfxinfo\0".as_ptr() as *const c_char, GfxInfo_f );
    Cmd_AddCommand( b"r_fogDistance\0".as_ptr() as *const c_char, R_FogDistance_f);
    Cmd_AddCommand( b"r_fogColor\0".as_ptr() as *const c_char, R_FogColor_f);
    Cmd_AddCommand( b"modelcacheinfo\0".as_ptr() as *const c_char, RE_RegisterModels_Info_f);
    Cmd_AddCommand( b"imagecacheinfo\0".as_ptr() as *const c_char, RE_RegisterImages_Info_f);
    Cmd_AddCommand( b"r_we\0".as_ptr() as *const c_char, R_WorldEffect_f );
    Cmd_AddCommand( b"r_reloadfonts\0".as_ptr() as *const c_char, R_ReloadFonts_f );
    // make sure all the commands added above are also
    // removed in R_Shutdown
}

// need to do this hackery so ghoul2 doesn't crash the game because of ITS hackery...
//
pub unsafe fn R_ClearStuffToStopGhoul2CrashingThings()
{
    memset( addr_of_mut!(tr) as *mut c_void, 0, mem::size_of::<trGlobals_t>() as c_int );
}

/*
===============
R_Init
===============
*/
pub unsafe fn R_Init() {
    let mut err: c_int;
    let mut i: c_int;

    //VID_Printf( PRINT_ALL, "----- R_Init -----\n" );
    #[cfg(target_os = "xbox")]
    {
        if vidRestartReloadMap == 0
        {
            Hunk_Clear();

            CM_Free();

            CM_CleanLeafCache();
        }
    }

    ShaderEntryPtrs_Clear();

    #[cfg(target_os = "xbox")]
    {
        //Save visibility info as it has already been set.
        // STUB: visibility data handling
    }

    // clear all our internal state
    memset( addr_of_mut!(tr) as *mut c_void, 0, mem::size_of::<trGlobals_t>() as c_int );
    memset( addr_of_mut!(backEnd) as *mut c_void, 0, mem::size_of::<backEndState_t>() as c_int );
    memset( addr_of_mut!(tess) as *mut c_void, 0, mem::size_of::<tessVertexBuffer_t>() as c_int );

    #[cfg(target_os = "xbox")]
    {
        //Restore visibility info.
        // STUB: visibility data handling
    }

    Swap_Init();

    #[cfg(not(feature = "FINAL_BUILD"))]
    {
        if (addr_of!(tess) as *const c_void as c_int & 15) != 0 {
            Com_Printf( b"WARNING: tess.xyz not 16 byte aligned (%x)\n\0".as_ptr() as *const c_char, (addr_of!(tess) as *const c_void as c_int & 15) );
        }
    }

    //
    // init function tables
    //
    i = 0;
    while i < FUNCTABLE_SIZE
    {
        (*addr_of_mut!(tr)).sinTable[i as usize] = (DEG2RAD * i as f32 * 360.0 / ((FUNCTABLE_SIZE - 1) as f32)).sin();
        (*addr_of_mut!(tr)).squareTable[i as usize] = if i < FUNCTABLE_SIZE/2 { 1.0 } else { -1.0 };
        (*addr_of_mut!(tr)).sawToothTable[i as usize] = i as c_float / FUNCTABLE_SIZE as c_float;
        (*addr_of_mut!(tr)).inverseSawToothTable[i as usize] = 1.0 - (*addr_of_mut!(tr)).sawToothTable[i as usize];

        if i < FUNCTABLE_SIZE / 2
        {
            if i < FUNCTABLE_SIZE / 4
            {
                (*addr_of_mut!(tr)).triangleTable[i as usize] = i as c_float / (FUNCTABLE_SIZE / 4) as c_float;
            }
            else
            {
                (*addr_of_mut!(tr)).triangleTable[i as usize] = 1.0 - (*addr_of_mut!(tr)).triangleTable[(i-FUNCTABLE_SIZE / 4) as usize];
            }
        }
        else
        {
            (*addr_of_mut!(tr)).triangleTable[i as usize] = -(*addr_of_mut!(tr)).triangleTable[(i-FUNCTABLE_SIZE/2) as usize];
        }
        i += 1;
    }

    R_InitFogTable();

    R_NoiseInit();

    R_Register();

    backEndData = Hunk_Alloc( mem::size_of::<backEndData_t>() as c_int, 1 ) as *mut backEndData_t;
    R_ToggleSmpFrame();	//r_smp

    let color: color4ub_t = color4ub_t { data: [0xff, 0xff, 0xff, 0xff] };
    i = 0;
    while i < MAX_LIGHT_STYLES
    {
        RE_SetLightStyle(i, *(addr_of!(color) as *const c_int));
        i += 1;
    }

    InitOpenGL();

    R_InitImages();
    R_InitShaders();
    R_InitSkins();
    #[cfg(not(target_os = "xbox"))]
    {
        R_TerrainInit();
    }
    R_ModelInit();
    //	R_InitWorldEffects();
    R_InitFonts();

    err = qglGetError();
    if err != GL_NO_ERROR
    {
        VID_Printf (PRINT_ALL, b"glGetError() = 0x%x\n\0".as_ptr() as *const c_char, err);
    }

    //VID_Printf( PRINT_ALL, "----- finished R_Init -----\n" );
}

/*
===============
RE_Shutdown
===============
*/
pub unsafe fn RE_Shutdown(destroyWindow: qboolean) {
    //VID_Printf( PRINT_ALL, "RE_Shutdown( %i )\n", destroyWindow );

    Cmd_RemoveCommand (b"imagelist\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"shaderlist\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"skinlist\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"modellist\0".as_ptr() as *const c_char);
    #[cfg(not(target_os = "xbox"))]
    {
        Cmd_RemoveCommand (b"modelist\0".as_ptr() as *const c_char );
        Cmd_RemoveCommand (b"r_atihack\0".as_ptr() as *const c_char);
    }
    Cmd_RemoveCommand (b"screenshot\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"screenshot_tga\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"gfxinfo\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"r_fogDistance\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"r_fogColor\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"modelcacheinfo\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"imagecacheinfo\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"r_we\0".as_ptr() as *const c_char);
    Cmd_RemoveCommand (b"r_reloadfonts\0".as_ptr() as *const c_char);

    R_ShutdownWorldEffects();
    #[cfg(not(target_os = "xbox"))]
    {
        R_TerrainShutdown();
    }
    R_ShutdownFonts();

    if (*addr_of!(tr)).registered != 0 {
        #[cfg(not(target_os = "xbox"))]
        {
            if !r_DynamicGlow.is_null() && (*r_DynamicGlow).integer != 0
            {
                // Release the Glow Vertex Shader.
                if (*addr_of!(tr)).glowVShader != 0
                {
                    qglDeleteProgramsARB( 1, addr_of!((*addr_of!(tr)).glowVShader) as *const c_int );
                }

                // Release Pixel Shader.
                if (*addr_of!(tr)).glowPShader != 0
                {
                    if !qglCombinerParameteriNV.is_none()
                    {
                        // Release the Glow Regcom call list.
                        qglDeleteLists( (*addr_of!(tr)).glowPShader, 1 );
                    }
                    else if !qglGenProgramsARB.is_none()
                    {
                        // Release the Glow Fragment Shader.
                        qglDeleteProgramsARB( 1, addr_of!((*addr_of!(tr)).glowPShader) as *const c_int );
                    }
                }

                // Release the scene glow texture.
                qglDeleteTextures( 1, addr_of!((*addr_of!(tr)).screenGlow) as *const c_int );

                // Release the scene texture.
                qglDeleteTextures( 1, addr_of!((*addr_of!(tr)).sceneImage) as *const c_int );

                // Release the blur texture.
                qglDeleteTextures( 1, addr_of!((*addr_of!(tr)).blurImage) as *const c_int );
            }
        }
        //		R_SyncRenderThread();
        R_ShutdownCommandBuffers();
        //#ifndef _XBOX
        if destroyWindow != 0
        //#endif
        {
            R_DeleteTextures();	// only do this for vid_restart now, not during things like map load
        }
    }

    // shut down platform specific OpenGL stuff
    if destroyWindow != 0 {
        GLimp_Shutdown();
    }
    (*addr_of_mut!(tr)).registered = 0;
}

/*
=============
RE_EndRegistration

Touch all images to make sure they are resident
=============
*/
pub unsafe fn RE_EndRegistration() {
    R_SyncRenderThread();
    if Sys_LowPhysicalMemory() == 0 {
        #[cfg(not(target_os = "xbox"))]
        {
            //		RB_ShowImages();
        }
    }
}


pub unsafe fn RE_GetLightStyle(style: c_int, color: color4ub_t)
{
    if style >= MAX_LIGHT_STYLES
    {
        Com_Error( ERR_FATAL, b"RE_GetLightStyle: %d is out of range\0".as_ptr() as *const c_char, style );
        return;
    }

    *(addr_of!(color) as *mut c_int) = *(addr_of!(styleColors[style as usize]) as *mut c_int);
}

pub unsafe fn RE_SetLightStyle(style: c_int, color: c_int)
{
    if style >= MAX_LIGHT_STYLES
    {
        Com_Error( ERR_FATAL, b"RE_SetLightStyle: %d is out of range\0".as_ptr() as *const c_char, style );
        return;
    }

    if *(addr_of!(styleColors[style as usize]) as *mut c_int) != color
    {
        *(addr_of_mut!(styleColors[style as usize]) as *mut c_int) = color;
        styleUpdated[style as usize] = true;
    }
}


/*
@@@@@@@@@@@@@@@@@@@@@
GetRefAPI

@@@@@@@@@@@@@@@@@@@@@
*/
#[repr(C)]
pub struct refexport_t {
    pub Shutdown: unsafe extern "C" fn(qboolean),
    pub BeginRegistration: unsafe extern "C" fn(),
    pub RegisterModel: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    pub RegisterSkin: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    pub GetAnimationCFG: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    pub RegisterShader: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    pub RegisterShaderNoMip: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    pub LoadWorld: unsafe extern "C" fn(*const c_char),
    pub SetWorldVisData: unsafe extern "C" fn(*const c_void),
    pub EndRegistration: unsafe extern "C" fn(),
    pub RegisterMedia_LevelLoadBegin: unsafe extern "C" fn(),
    pub RegisterMedia_LevelLoadEnd: unsafe extern "C" fn(),
    pub BeginFrame: unsafe extern "C" fn(),
    pub EndFrame: unsafe extern "C" fn(),
    pub ProcessDissolve: unsafe extern "C" fn(),
    pub InitDissolve: unsafe extern "C" fn(),
    pub MarkFragments: unsafe extern "C" fn(),
    pub LerpTag: unsafe extern "C" fn(),
    pub ModelBounds: unsafe extern "C" fn(),
    pub ClearScene: unsafe extern "C" fn(),
    pub AddRefEntityToScene: unsafe extern "C" fn(),
    pub GetLighting: unsafe extern "C" fn(),
    pub AddPolyToScene: unsafe extern "C" fn(),
    pub AddLightToScene: unsafe extern "C" fn(),
    pub RenderScene: unsafe extern "C" fn(),
    pub SetColor: unsafe extern "C" fn(),
    pub DrawStretchPic: unsafe extern "C" fn(),
    pub DrawStretchRaw: unsafe extern "C" fn(),
    pub UploadCinematic: unsafe extern "C" fn(),
    pub DrawRotatePic: unsafe extern "C" fn(),
    pub DrawRotatePic2: unsafe extern "C" fn(),
    pub LAGoggles: unsafe extern "C" fn(),
    pub Scissor: unsafe extern "C" fn(),
    pub GetScreenShot: unsafe extern "C" fn(),
    #[cfg(not(target_os = "xbox"))]
    pub TempRawImage_ReadFromFile: unsafe extern "C" fn(),
    pub TempRawImage_CleanUp: unsafe extern "C" fn(),
    pub GetLightStyle: unsafe extern "C" fn(c_int, color4ub_t),
    pub SetLightStyle: unsafe extern "C" fn(c_int, c_int),
    pub WorldEffectCommand: unsafe extern "C" fn(*const c_char),
    pub GetBModelVerts: unsafe extern "C" fn(),
    pub RegisterFont: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    #[cfg(not(target_os = "xbox"))]
    pub Font_StrLenPixels: unsafe extern "C" fn(*const c_char, *const c_void) -> c_int,
    #[cfg(not(target_os = "xbox"))]
    pub Font_HeightPixels: unsafe extern "C" fn(*const c_void) -> c_int,
    #[cfg(not(target_os = "xbox"))]
    pub Font_DrawString: unsafe extern "C" fn(c_int, c_int, *const c_char, *const c_void),
    pub Font_StrLenChars: unsafe extern "C" fn(*const c_char) -> c_int,
    pub Language_IsAsian: unsafe extern "C" fn() -> qboolean,
    pub Language_UsesSpaces: unsafe extern "C" fn() -> qboolean,
    pub AnyLanguage_ReadCharFromString: unsafe extern "C" fn() -> c_int,
}

pub unsafe fn GetRefAPI(apiVersion: c_int) -> *mut refexport_t {
    static mut re: refexport_t = refexport_t {
        Shutdown: RE_Shutdown,
        BeginRegistration: RE_BeginRegistration,
        RegisterModel: RE_RegisterModel,
        RegisterSkin: RE_RegisterSkin,
        GetAnimationCFG: RE_GetAnimationCFG,
        RegisterShader: RE_RegisterShader,
        RegisterShaderNoMip: RE_RegisterShaderNoMip,
        LoadWorld: RE_LoadWorldMap,
        SetWorldVisData: RE_SetWorldVisData,
        EndRegistration: RE_EndRegistration,
        RegisterMedia_LevelLoadBegin: RE_RegisterMedia_LevelLoadBegin,
        RegisterMedia_LevelLoadEnd: RE_RegisterMedia_LevelLoadEnd,
        BeginFrame: RE_BeginFrame,
        EndFrame: RE_EndFrame,
        ProcessDissolve: RE_ProcessDissolve,
        InitDissolve: RE_InitDissolve,
        MarkFragments: R_MarkFragments,
        LerpTag: R_LerpTag,
        ModelBounds: R_ModelBounds,
        ClearScene: RE_ClearScene,
        AddRefEntityToScene: RE_AddRefEntityToScene,
        GetLighting: RE_GetLighting,
        AddPolyToScene: RE_AddPolyToScene,
        AddLightToScene: RE_AddLightToScene,
        RenderScene: RE_RenderScene,
        SetColor: RE_SetColor,
        DrawStretchPic: RE_StretchPic,
        DrawStretchRaw: RE_StretchRaw,
        UploadCinematic: RE_UploadCinematic,
        DrawRotatePic: RE_RotatePic,
        DrawRotatePic2: RE_RotatePic2,
        LAGoggles: RE_LAGoggles,
        Scissor: RE_Scissor,
        GetScreenShot: RE_GetScreenShot,
        #[cfg(not(target_os = "xbox"))]
        TempRawImage_ReadFromFile: RE_TempRawImage_ReadFromFile,
        TempRawImage_CleanUp: RE_TempRawImage_CleanUp,
        GetLightStyle: RE_GetLightStyle,
        SetLightStyle: RE_SetLightStyle,
        WorldEffectCommand: R_WorldEffectCommand,
        GetBModelVerts: RE_GetBModelVerts,
        RegisterFont: RE_RegisterFont,
        #[cfg(not(target_os = "xbox"))]
        Font_StrLenPixels: RE_Font_StrLenPixels,
        #[cfg(not(target_os = "xbox"))]
        Font_HeightPixels: RE_Font_HeightPixels,
        #[cfg(not(target_os = "xbox"))]
        Font_DrawString: RE_Font_DrawString,
        Font_StrLenChars: RE_Font_StrLenChars,
        Language_IsAsian: Language_IsAsian,
        Language_UsesSpaces: Language_UsesSpaces,
        AnyLanguage_ReadCharFromString: AnyLanguage_ReadCharFromString,
    };

    memset( addr_of_mut!(re) as *mut c_void, 0, mem::size_of::<refexport_t>() as c_int );

    if apiVersion != REF_API_VERSION {
        VID_Printf(PRINT_ALL, b"Mismatched REF_API_VERSION: expected %i, got %i\n\0".as_ptr() as *const c_char,
            REF_API_VERSION, apiVersion );
        return core::ptr::null_mut();
    }

    // the RE_ functions are Renderer Entry points

    re.Shutdown = RE_Shutdown;

    re.BeginRegistration = RE_BeginRegistration;
    re.RegisterModel = RE_RegisterModel;
    re.RegisterSkin = RE_RegisterSkin;
    re.GetAnimationCFG = RE_GetAnimationCFG;
    re.RegisterShader = RE_RegisterShader;
    re.RegisterShaderNoMip = RE_RegisterShaderNoMip;
    re.LoadWorld = RE_LoadWorldMap;
    re.SetWorldVisData = RE_SetWorldVisData;
    re.EndRegistration = RE_EndRegistration;

    re.RegisterMedia_LevelLoadBegin = RE_RegisterMedia_LevelLoadBegin;
    re.RegisterMedia_LevelLoadEnd   = RE_RegisterMedia_LevelLoadEnd;

    re.BeginFrame = RE_BeginFrame;
    re.EndFrame = RE_EndFrame;

    re.ProcessDissolve = RE_ProcessDissolve;
    re.InitDissolve = RE_InitDissolve;

    re.MarkFragments = R_MarkFragments;
    re.LerpTag = R_LerpTag;
    re.ModelBounds = R_ModelBounds;

    re.ClearScene = RE_ClearScene;
    re.AddRefEntityToScene = RE_AddRefEntityToScene;
    re.GetLighting = RE_GetLighting;
    re.AddPolyToScene = RE_AddPolyToScene;
    re.AddLightToScene = RE_AddLightToScene;
    re.RenderScene = RE_RenderScene;

    re.SetColor = RE_SetColor;
    re.DrawStretchPic = RE_StretchPic;
    re.DrawStretchRaw = RE_StretchRaw;
    re.UploadCinematic = RE_UploadCinematic;

    re.DrawRotatePic = RE_RotatePic;
    re.DrawRotatePic2 = RE_RotatePic2;
    re.LAGoggles = RE_LAGoggles;
    re.Scissor = RE_Scissor;

    re.GetScreenShot = RE_GetScreenShot;
    #[cfg(not(target_os = "xbox"))]
    {
        re.TempRawImage_ReadFromFile = RE_TempRawImage_ReadFromFile;
    }
    re.TempRawImage_CleanUp = RE_TempRawImage_CleanUp;

    re.GetLightStyle = RE_GetLightStyle;
    re.SetLightStyle = RE_SetLightStyle;
    re.WorldEffectCommand = R_WorldEffectCommand;

    re.GetBModelVerts = RE_GetBModelVerts;

    re.RegisterFont = RE_RegisterFont;
    #[cfg(not(target_os = "xbox"))]
    {
        re.Font_StrLenPixels = RE_Font_StrLenPixels;
        re.Font_HeightPixels = RE_Font_HeightPixels;
        re.Font_DrawString = RE_Font_DrawString;
    }
    re.Font_StrLenChars = RE_Font_StrLenChars;
    re.Language_IsAsian = Language_IsAsian;
    re.Language_UsesSpaces = Language_UsesSpaces;
    re.AnyLanguage_ReadCharFromString = AnyLanguage_ReadCharFromString;

    return addr_of_mut!(re);
}
