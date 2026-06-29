// leave this as first line for PCH reasons...
//
// oracle/codemp/win32/win_glimp_console.cpp

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

use core::ffi::{c_char, c_int, c_void};

// Constants
const OPENGL_DRIVER_NAME: &[u8] = b"opengl32.dll\0";

// Type aliases for C compatibility
type qboolean = c_int;
const QTRUE: qboolean = 1;
const QFALSE: qboolean = 0;

//
// Stub/external struct types for FFI compatibility
//

#[repr(C)]
pub struct glwstate_t {
    // Content details not needed for console port stub
}

#[repr(C)]
pub struct glconfig_t {
    textureCompression: c_int,
    textureEnvAddAvailable: qboolean,
    maxTextureFilterAnisotropy: f32,
    clampToEdgeAvailable: qboolean,
    maxActiveTextures: c_int,
    maxTextureSize: c_int,
    vendor_string: *const c_char,
    renderer_string: *const c_char,
    version_string: *const c_char,
    extensions_string: *const c_char,
}

#[repr(C)]
pub struct glstate_t {
    // Content details not needed for console port stub
}

//
// External declarations and function pointers
//

extern "C" {
    fn WG_CheckHardwareGamma();

    // QGL subsystem
    fn QGL_EnableLogging(enable: qboolean);
    fn QGL_Init(dllname: *const c_char) -> qboolean;
    fn QGL_Shutdown();

    // GLW subsystem
    fn GLW_Init(width: c_int, height: c_int, colorbits: c_int, cdsFullscreen: qboolean);
    fn GLW_Shutdown();

    // Input system
    fn IN_Init();
    fn IN_Shutdown();

    // Console interface
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn va(fmt: *const c_char, ...) -> *const c_char;

    // OpenGL functions
    fn qglGetString(name: c_int) -> *const c_char;
    fn qglGetIntegerv(pname: c_int, params: *mut c_int);
    fn qglGetFloatv(pname: c_int, params: *mut f32);

    // GL function pointers (may be NULL)
    static mut qglActiveTextureARB: Option<unsafe extern "C" fn(c_int)>;
    static mut qglMultiTexCoord2fARB: Option<unsafe extern "C" fn(c_int, f32, f32)>;
    static mut qglClientActiveTextureARB: Option<unsafe extern "C" fn(c_int)>;

    // Global GL config/state
    static mut glConfig: glconfig_t;
    static mut glState: glstate_t;
}

//
// variable declarations
//
pub static mut glw_state: *mut glwstate_t = core::ptr::null_mut();

//
// function declarations (forward)
//
fn GLW_InitExtensions();
fn GLW_CreateWindow(width: c_int, height: c_int, colorbits: c_int, cdsFullscreen: qboolean) -> qboolean;

/*
** GLW_CreateWindow
**
** Responsible for creating the Alchemy window and initializing the OpenGL driver.
*/
fn GLW_CreateWindow(width: c_int, height: c_int, colorbits: c_int, cdsFullscreen: qboolean) -> qboolean
{
    unsafe {
        GLW_Init(width, height, colorbits, cdsFullscreen);
        IN_Init();
    }

    QTRUE
}

//--------------------------------------------
fn GLW_InitTextureCompression()
{
    unsafe {
        glConfig.textureCompression = 0; // TC_NONE
    }
}

/*
** GLW_InitExtensions
*/
fn GLW_InitExtensions()
{
    unsafe {
        // Select our tc scheme
        GLW_InitTextureCompression();

        // GL_EXT_texture_env_add
        glConfig.textureEnvAddAvailable = QFALSE;
        if !strstr_c(
            glConfig.extensions_string,
            b"EXT_texture_env_add\0".as_ptr() as *const c_char
        ).is_null()
        {
            glConfig.textureEnvAddAvailable = QTRUE;
        }

        // GL_EXT_texture_filter_anisotropic
        glConfig.maxTextureFilterAnisotropy = 0.0;
        if !strstr_c(
            glConfig.extensions_string,
            b"EXT_texture_filter_anisotropic\0".as_ptr() as *const c_char
        ).is_null()
        {
            qglGetFloatv(0x84FF, &mut glConfig.maxTextureFilterAnisotropy); // GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT

            Com_Printf(b"...GL_EXT_texture_filter_anisotropic available\n\0".as_ptr() as *const c_char);

            if r_ext_texture_filter_anisotropic_get_integer() > 1
            {
                Com_Printf(b"...using GL_EXT_texture_filter_anisotropic\n\0".as_ptr() as *const c_char);
            }
            else
            {
                Com_Printf(b"...ignoring GL_EXT_texture_filter_anisotropic\n\0".as_ptr() as *const c_char);
            }
            Cvar_Set(
                b"r_ext_texture_filter_anisotropic_avail\0".as_ptr() as *const c_char,
                va(b"%f\0".as_ptr() as *const c_char, glConfig.maxTextureFilterAnisotropy)
            );
            if r_ext_texture_filter_anisotropic_get_value() > glConfig.maxTextureFilterAnisotropy
            {
                Cvar_Set(
                    b"r_ext_texture_filter_anisotropic\0".as_ptr() as *const c_char,
                    va(b"%f\0".as_ptr() as *const c_char, glConfig.maxTextureFilterAnisotropy)
                );
            }
        }
        else
        {
            Com_Printf(b"...GL_EXT_texture_filter_anisotropic not found\n\0".as_ptr() as *const c_char);
            Cvar_Set(
                b"r_ext_texture_filter_anisotropic_avail\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char
            );
        }

        // GL_EXT_clamp_to_edge
        glConfig.clampToEdgeAvailable = QFALSE;
        if !strstr_c(
            glConfig.extensions_string,
            b"GL_EXT_texture_edge_clamp\0".as_ptr() as *const c_char
        ).is_null()
        {
            glConfig.clampToEdgeAvailable = QTRUE;
        }

        // GL_ARB_multitexture
        if !strstr_c(
            glConfig.extensions_string,
            b"GL_ARB_multitexture\0".as_ptr() as *const c_char
        ).is_null()
        {
            if !qglActiveTextureARB.is_none()
            {
                qglGetIntegerv(0x84E8, &mut glConfig.maxActiveTextures); // GL_MAX_ACTIVE_TEXTURES_ARB

                if glConfig.maxActiveTextures < 2
                {
                    qglMultiTexCoord2fARB = None;
                    qglActiveTextureARB = None;
                    qglClientActiveTextureARB = None;
                }
            }
        }
    }
}

/*
** GLW_LoadOpenGL
**
** GLimp_win.c internal function that attempts to load and use
** a specific OpenGL DLL.
*/
fn GLW_LoadOpenGL() -> qboolean
{
    let mut buffer: [c_char; 1024] = [0; 1024];

    unsafe {
        strlwr_c(&mut buffer as *mut _ as *mut c_char, OPENGL_DRIVER_NAME.as_ptr() as *const c_char);

        //
        // load the driver and bind our function pointers to it
        //
        if QGL_Init(&buffer as *const _ as *const c_char) != QFALSE
        {
            let _ = GLW_CreateWindow(640, 480, 24, 1);
            return QTRUE;
        }

        QGL_Shutdown();
    }

    QFALSE
}


/*
** GLimp_EndFrame
*/
pub extern "C" fn GLimp_EndFrame()
{
    // don't flip if drawing to front buffer
    //	if ( stricmp( r_drawBuffer->string, "GL_FRONT" ) != 0 )
    {
    }
}

fn GLW_StartOpenGL()
{
    //
    // load and initialize the specific OpenGL driver
    //
    unsafe {
        if GLW_LoadOpenGL() == QFALSE
        {
            Com_Error(1, b"GLW_StartOpenGL() - could not load OpenGL subsystem\n\0".as_ptr() as *const c_char); // ERR_FATAL = 1
        }
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
pub extern "C" fn GLimp_Init()
{
    unsafe {
        // load appropriate DLL and initialize subsystem
        GLW_StartOpenGL();

        // get our config strings
        glConfig.vendor_string = qglGetString(0x1F00) as *const c_char; // GL_VENDOR
        glConfig.renderer_string = qglGetString(0x1F01) as *const c_char; // GL_RENDERER
        glConfig.version_string = qglGetString(0x1F02) as *const c_char; // GL_VERSION
        glConfig.extensions_string = qglGetString(0x1F03) as *const c_char; // GL_EXTENSIONS

        if glConfig.vendor_string.is_null() || glConfig.renderer_string.is_null() || glConfig.version_string.is_null() || glConfig.extensions_string.is_null()
        {
            Com_Error(1, b"GLimp_Init() - Invalid GL Driver\n\0".as_ptr() as *const c_char); // ERR_FATAL = 1
        }

        // OpenGL driver constants
        qglGetIntegerv(0x0D33, &mut glConfig.maxTextureSize); // GL_MAX_TEXTURE_SIZE
        // stubbed or broken drivers may have reported 0...
        if glConfig.maxTextureSize <= 0
        {
            glConfig.maxTextureSize = 0;
        }

        GLW_InitExtensions();
        WG_CheckHardwareGamma();
    }
}

/*
** GLimp_Shutdown
**
** This routine does all OS specific shutdown procedures for the OpenGL
** subsystem.
*/
pub extern "C" fn GLimp_Shutdown()
{
    unsafe {
        // FIXME: Brian, we need better fallbacks from partially initialized failures
        Com_Printf(b"Shutting down OpenGL subsystem\n\0".as_ptr() as *const c_char);

        // Set the gamma back to normal
        //	GLimp_SetGamma(1.f);

        // kill input system (tied to window)
        IN_Shutdown();

        // shutdown QGL subsystem
        GLW_Shutdown();
        QGL_Shutdown();

        core::ptr::write_bytes(&mut glConfig as *mut _ as *mut c_char, 0, core::mem::size_of::<glconfig_t>());
        core::ptr::write_bytes(&mut glState as *mut _ as *mut c_char, 0, core::mem::size_of::<glstate_t>());
    }
}

/*
** GLimp_LogComment
*/
pub extern "C" fn GLimp_LogComment(comment: *mut c_char)
{
}


/*
===========================================================

SMP acceleration

===========================================================
*/

pub extern "C" fn GLimp_SpawnRenderThread(function: Option<extern "C" fn()>) -> qboolean {
    QFALSE
}

pub extern "C" fn GLimp_RendererSleep() -> *mut c_void {
    core::ptr::null_mut()
}


pub extern "C" fn GLimp_FrontEndSleep()
{
}


pub extern "C" fn GLimp_WakeRenderer(data: *mut c_void)
{
}

//
// Local helper stubs for C library functions
//

// Stub for strstr that matches C behavior
fn strstr_c(haystack: *const c_char, needle: *const c_char) -> *const c_char {
    unsafe {
        if haystack.is_null() || needle.is_null() {
            return core::ptr::null();
        }

        let mut h_ptr = haystack;
        let mut n_ptr = needle;

        while *h_ptr != 0 {
            let mut h = h_ptr;
            let mut n = n_ptr;

            while *n != 0 && *h == *n {
                h = h.offset(1);
                n = n.offset(1);
            }

            if *n == 0 {
                return h_ptr;
            }

            h_ptr = h_ptr.offset(1);
        }

        core::ptr::null()
    }
}

// Stub for strlwr that copies and lowercases the string
fn strlwr_c(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    unsafe {
        let mut d = dst;
        let mut s = src;

        while *s != 0 {
            *d = if *s >= b'A' as c_char && *s <= b'Z' as c_char {
                (*s as u8 - b'A' as u8 + b'a' as u8) as c_char
            } else {
                *s
            };
            d = d.offset(1);
            s = s.offset(1);
        }

        *d = 0;
        dst
    }
}

// Stub accessors for cvar values (these would normally come from external code)
fn r_ext_texture_filter_anisotropic_get_integer() -> c_int {
    0 // Stub
}

fn r_ext_texture_filter_anisotropic_get_value() -> f32 {
    0.0 // Stub
}
