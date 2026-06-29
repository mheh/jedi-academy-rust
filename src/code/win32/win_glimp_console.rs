// leave this as first line for PCH reasons...
//
// [Ported from ../server/exe_headers.h]

/*
** WIN_GLIMP.C
**
** This file contains ALL Win32 specific stuff having to do with the
** OpenGL refresh.  When a port is being made the following functions
** must be implemented by the port:
**
** GLimp_EndFrame
** GLimp_Init
** GLimp_LogComment
** GLimp_Shutdown
**
** Note that the GLW_xxx functions are Windows specific GL-subsystem
** related functions that are relevant ONLY to win_glimp.c
*/

use core::ffi::{c_int, c_char, c_void};
use core::ptr;

// Opaque type declarations for structures defined in ported headers
// (definitions will be in glw_win_dx8_h.rs, tr_local_h.rs, etc.)
#[repr(C)]
pub struct glwstate_t {
    _opaque: [u8; 0],
}

// Type aliases matching C types
pub type qboolean = c_int;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// [../renderer/tr_local.h]
// These would be properly typed when tr_local.h is ported
#[repr(C)]
pub struct glConfig_s {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct glState_s {
    _opaque: [u8; 0],
}

extern "C" {
    pub static mut glConfig: glConfig_s;
    pub static mut glState: glState_s;
}

// [win_local.h and libc]
extern "C" {
    // From libc
    pub fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strlwr(str: *mut c_char) -> *mut c_char;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    // Engine functions from qcommon
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...) -> !;
    pub fn VID_Printf(level: c_int, fmt: *const c_char, ...);

    // Input system
    pub fn IN_Init();
    pub fn IN_Shutdown();

    // Hardware
    pub fn WG_CheckHardwareGamma();

    // GL function loading
    pub fn QGL_EnableLogging(enable: qboolean);
    pub fn QGL_Init(dllname: *const c_char) -> qboolean;
    pub fn QGL_Shutdown();

    // GL subsystem initialization
    pub fn GLW_Init(width: c_int, height: c_int, colorbits: c_int, cdsFullscreen: qboolean);
    pub fn GLW_Shutdown();
}

//
// variable declarations
//
pub static mut glw_state: *mut glwstate_t = ptr::null_mut();


/*
** GLW_CreateWindow
**
** Responsible for creating the Alchemy window and initializing the OpenGL driver.
*/
unsafe fn GLW_CreateWindow(width: c_int, height: c_int, colorbits: c_int, cdsFullscreen: qboolean) -> qboolean
{
    GLW_Init(width, height, colorbits, cdsFullscreen);
    IN_Init();

    return qtrue;
}

//--------------------------------------------
unsafe fn GLW_InitTextureCompression()
{
    // glConfig.textureCompression = TC_NONE;
    // Note: Actual field access would be done once glConfig_s is fully defined
}

/*
** GLW_InitExtensions
*/
unsafe fn GLW_InitExtensions()
{
    // Select our tc scheme
    GLW_InitTextureCompression();

    // GL_EXT_texture_env_add
    // glConfig.textureEnvAddAvailable = qfalse;
    // if ( strstr( glConfig.extensions_string, "EXT_texture_env_add" ) )
    // {
    //     glConfig.textureEnvAddAvailable = qtrue;
    // }

    // GL_EXT_texture_filter_anisotropic
    // glConfig.textureFilterAnisotropicAvailable = qfalse;
    // if ( strstr( glConfig.extensions_string, "EXT_texture_filter_anisotropic" ) )
    // {
    //     glConfig.textureFilterAnisotropicAvailable = qtrue;
    // }

    // GL_EXT_clamp_to_edge
    // glConfig.clampToEdgeAvailable = qfalse;
    // if ( strstr( glConfig.extensions_string, "GL_EXT_texture_edge_clamp" ) )
    // {
    //     glConfig.clampToEdgeAvailable = qtrue;
    // }

    // GL_ARB_multitexture
    // if ( strstr( glConfig.extensions_string, "GL_ARB_multitexture" )  )
    // {
    //     if ( qglActiveTextureARB )
    //     {
    //         qglGetIntegerv( GL_MAX_ACTIVE_TEXTURES_ARB, &glConfig.maxActiveTextures );
    //
    //         if ( glConfig.maxActiveTextures < 2 )
    //         {
    //             qglMultiTexCoord2fARB = NULL;
    //             qglActiveTextureARB = NULL;
    //             qglClientActiveTextureARB = NULL;
    //         }
    //     }
    // }
}

/*
** GLW_LoadOpenGL
**
** GLimp_win.c internal function that attempts to load and use
** a specific OpenGL DLL.
*/
unsafe fn GLW_LoadOpenGL() -> qboolean
{
    let mut buffer: [c_char; 1024] = [0; 1024];

    strlwr(strcpy(buffer.as_mut_ptr(), b"opengl32\0".as_ptr() as *const c_char));

    //
    // load the driver and bind our function pointers to it
    //
    if QGL_Init(buffer.as_ptr()) != 0
    {
        GLW_CreateWindow(640, 480, 24, 1);
        return qtrue;
    }

    QGL_Shutdown();

    return qfalse;
}


/*
** GLimp_EndFrame
*/
pub fn GLimp_EndFrame()
{
    // don't flip if drawing to front buffer
//    if ( stricmp( r_drawBuffer->string, "GL_FRONT" ) != 0 )
    {
    }
}

unsafe fn GLW_StartOpenGL()
{
    //
    // load and initialize the specific OpenGL driver
    //
    if GLW_LoadOpenGL() == 0
    {
        Com_Error(3, b"GLW_StartOpenGL() - could not load OpenGL subsystem\n\0".as_ptr() as *const c_char);
    }
}

/*
** GLimp_Init
**
** This is the platform specific OpenGL initialization function.  It
** is responsible for loading OpenGL, initializing it, setting
** extensions, creating a window of the appropriate size, doing
** fullscreen manipulations, etc.  Its overall responsibility is
** to make sure that a functional OpenGL subsystem is operating
** when it returns to the ref.
*/
pub unsafe fn GLimp_Init()
{
    // load appropriate DLL and initialize subsystem
    GLW_StartOpenGL();

    // get our config strings
    // glConfig.vendor_string = (const char *) qglGetString (GL_VENDOR);
    // glConfig.renderer_string = (const char *) qglGetString (GL_RENDERER);
    // glConfig.version_string = (const char *) qglGetString (GL_VERSION);
    // glConfig.extensions_string = (const char *) qglGetString (GL_EXTENSIONS);

    // if (!glConfig.vendor_string || !glConfig.renderer_string || !glConfig.version_string || !glConfig.extensions_string)
    // {
    //     Com_Error( ERR_FATAL, "GLimp_Init() - Invalid GL Driver\n" );
    // }

    // OpenGL driver constants
    // qglGetIntegerv( GL_MAX_TEXTURE_SIZE, &glConfig.maxTextureSize );
    // stubbed or broken drivers may have reported 0...
    // if ( glConfig.maxTextureSize <= 0 )
    // {
    //     glConfig.maxTextureSize = 0;
    // }

    GLW_InitExtensions();
    WG_CheckHardwareGamma();
}

/*
** GLimp_Shutdown
**
** This routine does all OS specific shutdown procedures for the OpenGL
** subsystem.
*/
pub unsafe fn GLimp_Shutdown()
{
    // FIXME: Brian, we need better fallbacks from partially initialized failures
    VID_Printf(0, b"Shutting down OpenGL subsystem\n\0".as_ptr() as *const c_char);

    // Set the gamma back to normal
//    GLimp_SetGamma(1.f);

    // kill input system (tied to window)
    IN_Shutdown();

    // shutdown QGL subsystem
    GLW_Shutdown();
    QGL_Shutdown();

    memset(&mut glConfig as *mut glConfig_s as *mut c_void, 0, core::mem::size_of::<glConfig_s>());
    memset(&mut glState as *mut glState_s as *mut c_void, 0, core::mem::size_of::<glState_s>());
}

/*
** GLimp_LogComment
*/
pub fn GLimp_LogComment(comment: *mut c_char)
{
}


/*
===========================================================

SMP acceleration

===========================================================
*/

pub fn GLimp_SpawnRenderThread(_function: Option<unsafe extern "C" fn()>) -> qboolean {
    return qfalse;
}

pub fn GLimp_RendererSleep() -> *mut c_void {
    return ptr::null_mut();
}


pub fn GLimp_FrontEndSleep()
{
}


pub fn GLimp_WakeRenderer(_data: *mut c_void)
{
}
