//
// QGL.H
//

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_void};

// Platform-specific includes translated as comments
// #if defined( __LINT__ )
// #include <GL/gl.h>
// #elif defined( _WIN32 )
// #pragma warning (disable: 4201)
// #pragma warning (disable: 4214)
// #pragma warning (disable: 4514)
// #pragma warning (disable: 4032)
// #pragma warning (disable: 4201)
// #pragma warning (disable: 4214)
// #include <windows.h>
// #include <gl/gl.h>
// #elif defined( __APPLE__ ) && defined( __MACH__ )
// #include <MesaGL/gl.h>
// #elif defined( __linux__ )
// #include <GL/gl.h>
// #include <GL/glx.h>
// #include <GL/fxmesa.h>
// #else
// #include <gl.h>
// #endif

// Placeholder OpenGL type aliases from standard GL headers
pub type GLenum = core::ffi::c_uint;
pub type GLboolean = core::ffi::c_uchar;
pub type GLbitfield = core::ffi::c_uint;
pub type GLbyte = core::ffi::c_schar;
pub type GLshort = core::ffi::c_short;
pub type GLint = core::ffi::c_int;
pub type GLsizei = core::ffi::c_int;
pub type GLubyte = core::ffi::c_uchar;
pub type GLushort = core::ffi::c_ushort;
pub type GLuint = core::ffi::c_uint;
pub type GLfloat = f32;
pub type GLclampf = f32;
pub type GLdouble = f64;
pub type GLclampd = f64;
pub type GLvoid = c_void;

// Windows-specific types (stubbed for non-Windows)
#[cfg(target_os = "windows")]
pub use std::ffi::c_char;
#[cfg(not(target_os = "windows"))]
pub type c_char = core::ffi::c_char;

#[cfg(target_os = "windows")]
pub type HDC = *mut c_void;
#[cfg(not(target_os = "windows"))]
pub type HDC = *mut c_void;

#[cfg(target_os = "windows")]
pub type HGLRC = *mut c_void;
#[cfg(not(target_os = "windows"))]
pub type HGLRC = *mut c_void;

#[cfg(target_os = "windows")]
pub type PROC = *mut c_void;
#[cfg(not(target_os = "windows"))]
pub type PROC = *mut c_void;

pub type DWORD = core::ffi::c_ulong;
pub type UINT = core::ffi::c_uint;
pub type BOOL = core::ffi::c_int;
pub type FLOAT = f32;
pub type COLORREF = core::ffi::c_ulong;
pub type LPCSTR = *const c_char;
pub type LPGLYPHMETRICSFLOAT = *mut c_void;
pub type LPLAYERPLANEDESCRIPTOR = *mut c_void;
pub type CONST = core::ffi::c_void;

// Linux-specific types (stubbed)
#[cfg(target_os = "linux")]
pub type Display = core::ffi::c_void;
#[cfg(not(target_os = "linux"))]
pub type Display = core::ffi::c_void;

#[cfg(target_os = "linux")]
pub type XVisualInfo = core::ffi::c_void;
#[cfg(not(target_os = "linux"))]
pub type XVisualInfo = core::ffi::c_void;

#[cfg(target_os = "linux")]
pub type GLXContext = core::ffi::c_void;
#[cfg(not(target_os = "linux"))]
pub type GLXContext = core::ffi::c_void;

#[cfg(target_os = "linux")]
pub type GLXDrawable = core::ffi::c_ulong;
#[cfg(not(target_os = "linux"))]
pub type GLXDrawable = core::ffi::c_ulong;

#[cfg(target_os = "linux")]
pub type Bool = c_int;
#[cfg(not(target_os = "linux"))]
pub type Bool = c_int;

#[cfg(target_os = "linux")]
pub type fxMesaContext = core::ffi::c_void;
#[cfg(not(target_os = "linux"))]
pub type fxMesaContext = core::ffi::c_void;

#[cfg(target_os = "linux")]
pub type GrScreenResolution_t = core::ffi::c_uint;
#[cfg(not(target_os = "linux"))]
pub type GrScreenResolution_t = core::ffi::c_uint;

#[cfg(target_os = "linux")]
pub type GrScreenRefresh_t = core::ffi::c_uint;
#[cfg(not(target_os = "linux"))]
pub type GrScreenRefresh_t = core::ffi::c_uint;

// APIENTRY and WINAPI macros are empty in standard headers
pub const APIENTRY: () = ();
pub const WINAPI: () = ();

// DECLARE_HANDLE stub for HPBUFFERARB
pub type HPBUFFERARB = *mut core::ffi::c_void;

//===========================================================================

//
// multitexture extension definitions
//
pub const GL_ACTIVE_TEXTURE_ARB: GLenum = 0x84E0;
pub const GL_CLIENT_ACTIVE_TEXTURE_ARB: GLenum = 0x84E1;
pub const GL_MAX_ACTIVE_TEXTURES_ARB: GLenum = 0x84E2;

pub const GL_TEXTURE0_ARB: GLenum = 0x84C0;
pub const GL_TEXTURE1_ARB: GLenum = 0x84C1;
pub const GL_TEXTURE2_ARB: GLenum = 0x84C2;
pub const GL_TEXTURE3_ARB: GLenum = 0x84C3;

pub const GL_TEXTURE_RECTANGLE_EXT: GLenum = 0x84F5;

pub type PFNGLMULTITEXCOORD1DARBPROC = extern "C" fn(GLenum, GLdouble);
pub type PFNGLMULTITEXCOORD1DVARBPROC = extern "C" fn(GLenum, *const GLdouble);
pub type PFNGLMULTITEXCOORD1FARBPROC = extern "C" fn(GLenum, GLfloat);
pub type PFNGLMULTITEXCOORD1FVARBPROC = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLMULTITEXCOORD1IARBPROC = extern "C" fn(GLenum, GLint);
pub type PFNGLMULTITEXCOORD1IVARBPROC = extern "C" fn(GLenum, *const GLint);
pub type PFNGLMULTITEXCOORD1SARBPROC = extern "C" fn(GLenum, GLshort);
pub type PFNGLMULTITEXCOORD1SVARBPROC = extern "C" fn(GLenum, *const GLshort);
pub type PFNGLMULTITEXCOORD2DARBPROC = extern "C" fn(GLenum, GLdouble, GLdouble);
pub type PFNGLMULTITEXCOORD2DVARBPROC = extern "C" fn(GLenum, *const GLdouble);
pub type PFNGLMULTITEXCOORD2FARBPROC = extern "C" fn(GLenum, GLfloat, GLfloat);
pub type PFNGLMULTITEXCOORD2FVARBPROC = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLMULTITEXCOORD2IARBPROC = extern "C" fn(GLenum, GLint, GLint);
pub type PFNGLMULTITEXCOORD2IVARBPROC = extern "C" fn(GLenum, *const GLint);
pub type PFNGLMULTITEXCOORD2SARBPROC = extern "C" fn(GLenum, GLshort, GLshort);
pub type PFNGLMULTITEXCOORD2SVARBPROC = extern "C" fn(GLenum, *const GLshort);
pub type PFNGLMULTITEXCOORD3DARBPROC = extern "C" fn(GLenum, GLdouble, GLdouble, GLdouble);
pub type PFNGLMULTITEXCOORD3DVARBPROC = extern "C" fn(GLenum, *const GLdouble);
pub type PFNGLMULTITEXCOORD3FARBPROC = extern "C" fn(GLenum, GLfloat, GLfloat, GLfloat);
pub type PFNGLMULTITEXCOORD3FVARBPROC = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLMULTITEXCOORD3IARBPROC = extern "C" fn(GLenum, GLint, GLint, GLint);
pub type PFNGLMULTITEXCOORD3IVARBPROC = extern "C" fn(GLenum, *const GLint);
pub type PFNGLMULTITEXCOORD3SARBPROC = extern "C" fn(GLenum, GLshort, GLshort, GLshort);
pub type PFNGLMULTITEXCOORD3SVARBPROC = extern "C" fn(GLenum, *const GLshort);
pub type PFNGLMULTITEXCOORD4DARBPROC = extern "C" fn(GLenum, GLdouble, GLdouble, GLdouble, GLdouble);
pub type PFNGLMULTITEXCOORD4DVARBPROC = extern "C" fn(GLenum, *const GLdouble);
pub type PFNGLMULTITEXCOORD4FARBPROC = extern "C" fn(GLenum, GLfloat, GLfloat, GLfloat, GLfloat);
pub type PFNGLMULTITEXCOORD4FVARBPROC = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLMULTITEXCOORD4IARBPROC = extern "C" fn(GLenum, GLint, GLint, GLint, GLint);
pub type PFNGLMULTITEXCOORD4IVARBPROC = extern "C" fn(GLenum, *const GLint);
pub type PFNGLMULTITEXCOORD4SARBPROC = extern "C" fn(GLenum, GLshort, GLshort, GLshort, GLshort);
pub type PFNGLMULTITEXCOORD4SVARBPROC = extern "C" fn(GLenum, *const GLshort);
pub type PFNGLACTIVETEXTUREARBPROC = extern "C" fn(GLenum);
pub type PFNGLCLIENTACTIVETEXTUREARBPROC = extern "C" fn(GLenum);


// Steps to adding a new extension:
//	- Add the typedef and function pointer externs here.
//	- Define the function pointer in tr_init.cpp and possibly add a cvar to track your ext status.
//	- Load the extension in win_glimp.cpp.


/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Register Combiner extension definitions. - AReis
/***********************************************************************************************************/
// NOTE: These are obviously not all the regcom flags. I'm only including the ones I use (to reduce code clutter), so
// if you need any of the other flags, just add them.
pub const GL_REGISTER_COMBINERS_NV: GLenum = 0x8522;
pub const GL_COMBINER0_NV: GLenum = 0x8550;
pub const GL_COMBINER1_NV: GLenum = 0x8551;
pub const GL_COMBINER2_NV: GLenum = 0x8552;
pub const GL_COMBINER3_NV: GLenum = 0x8553;
pub const GL_COMBINER4_NV: GLenum = 0x8554;
pub const GL_COMBINER5_NV: GLenum = 0x8555;
pub const GL_COMBINER6_NV: GLenum = 0x8556;
pub const GL_COMBINER7_NV: GLenum = 0x8557;
pub const GL_NUM_GENERAL_COMBINERS_NV: GLenum = 0x854E;
pub const GL_VARIABLE_A_NV: GLenum = 0x8523;
pub const GL_VARIABLE_B_NV: GLenum = 0x8524;
pub const GL_VARIABLE_C_NV: GLenum = 0x8525;
pub const GL_VARIABLE_D_NV: GLenum = 0x8526;
pub const GL_VARIABLE_E_NV: GLenum = 0x8527;
pub const GL_VARIABLE_F_NV: GLenum = 0x8528;
pub const GL_VARIABLE_G_NV: GLenum = 0x8529;
pub const GL_DISCARD_NV: GLenum = 0x8530;
pub const GL_CONSTANT_COLOR0_NV: GLenum = 0x852A;
pub const GL_CONSTANT_COLOR1_NV: GLenum = 0x852B;
pub const GL_SPARE0_NV: GLenum = 0x852E;
pub const GL_SPARE1_NV: GLenum = 0x852F;
pub const GL_UNSIGNED_IDENTITY_NV: GLenum = 0x8536;
pub const GL_UNSIGNED_INVERT_NV: GLenum = 0x8537;

pub type PFNGLCOMBINERPARAMETERFVNV = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLCOMBINERPARAMETERIVNV = extern "C" fn(GLenum, *const GLint);
pub type PFNGLCOMBINERPARAMETERFNV = extern "C" fn(GLenum, GLfloat);
pub type PFNGLCOMBINERPARAMETERINV = extern "C" fn(GLenum, GLint);
pub type PFNGLCOMBINERINPUTNV = extern "C" fn(GLenum, GLenum, GLenum, GLenum, GLenum, GLenum);
pub type PFNGLCOMBINEROUTPUTNV = extern "C" fn(GLenum, GLenum, GLenum, GLenum, GLenum, GLenum, GLenum, GLboolean, GLboolean, GLboolean);
pub type PFNGLFINALCOMBINERINPUTNV = extern "C" fn(GLenum, GLenum, GLenum, GLenum);

pub type PFNGLGETCOMBINERINPUTPARAMETERFVNV = extern "C" fn(GLenum, GLenum, GLenum, GLenum, *mut GLfloat);
pub type PFNGLGETCOMBINERINPUTPARAMETERIVNV = extern "C" fn(GLenum, GLenum, GLenum, GLenum, *mut GLint);
pub type PFNGLGETCOMBINEROUTPUTPARAMETERFVNV = extern "C" fn(GLenum, GLenum, GLenum, *mut GLfloat);
pub type PFNGLGETCOMBINEROUTPUTPARAMETERIVNV = extern "C" fn(GLenum, GLenum, GLenum, *mut GLint);
pub type PFNGLGETFINALCOMBINERINPUTPARAMETERFVNV = extern "C" fn(GLenum, GLenum, *mut GLfloat);
pub type PFNGLGETFINALCOMBINERINPUTPARAMETERIVNV = extern "C" fn(GLenum, GLenum, *mut GLfloat);
/***********************************************************************************************************/

// Declare Register Combiners function pointers.
pub static mut qglCombinerParameterfvNV: Option<PFNGLCOMBINERPARAMETERFVNV> = None;
pub static mut qglCombinerParameterivNV: Option<PFNGLCOMBINERPARAMETERIVNV> = None;
pub static mut qglCombinerParameterfNV: Option<PFNGLCOMBINERPARAMETERFNV> = None;
pub static mut qglCombinerParameteriNV: Option<PFNGLCOMBINERPARAMETERINV> = None;
pub static mut qglCombinerInputNV: Option<PFNGLCOMBINERINPUTNV> = None;
pub static mut qglCombinerOutputNV: Option<PFNGLCOMBINEROUTPUTNV> = None;
pub static mut qglFinalCombinerInputNV: Option<PFNGLFINALCOMBINERINPUTNV> = None;
pub static mut qglGetCombinerInputParameterfvNV: Option<PFNGLGETCOMBINERINPUTPARAMETERFVNV> = None;
pub static mut qglGetCombinerInputParameterivNV: Option<PFNGLGETCOMBINERINPUTPARAMETERIVNV> = None;
pub static mut qglGetCombinerOutputParameterfvNV: Option<PFNGLGETCOMBINEROUTPUTPARAMETERFVNV> = None;
pub static mut qglGetCombinerOutputParameterivNV: Option<PFNGLGETCOMBINEROUTPUTPARAMETERIVNV> = None;
pub static mut qglGetFinalCombinerInputParameterfvNV: Option<PFNGLGETFINALCOMBINERINPUTPARAMETERFVNV> = None;
pub static mut qglGetFinalCombinerInputParameterivNV: Option<PFNGLGETFINALCOMBINERINPUTPARAMETERIVNV> = None;


/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Pixel Format extension definitions. - AReis
/***********************************************************************************************************/
pub const WGL_COLOR_BITS_ARB: c_int = 0x2014;
pub const WGL_ALPHA_BITS_ARB: c_int = 0x201B;
pub const WGL_DEPTH_BITS_ARB: c_int = 0x2022;
pub const WGL_STENCIL_BITS_ARB: c_int = 0x2023;

pub type PFNWGLGETPIXELFORMATATTRIBIVARBPROC = extern "C" fn(HDC, c_int, c_int, UINT, *const c_int, *mut c_int) -> BOOL;
pub type PFNWGLGETPIXELFORMATATTRIBFVARBPROC = extern "C" fn(HDC, c_int, c_int, UINT, *const c_int, *mut FLOAT) -> BOOL;
pub type PFNWGLCHOOSEPIXELFORMATARBPROC = extern "C" fn(HDC, *const c_int, *const FLOAT, UINT, *mut c_int, *mut UINT) -> BOOL;
/***********************************************************************************************************/

// Declare Pixel Format function pointers.
pub static mut qwglGetPixelFormatAttribivARB: Option<PFNWGLGETPIXELFORMATATTRIBIVARBPROC> = None;
pub static mut qwglGetPixelFormatAttribfvARB: Option<PFNWGLGETPIXELFORMATATTRIBFVARBPROC> = None;
pub static mut qwglChoosePixelFormatARB: Option<PFNWGLCHOOSEPIXELFORMATARBPROC> = None;


/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Pixel Buffer extension definitions. - AReis
/***********************************************************************************************************/
// DECLARE_HANDLE(HPBUFFERARB);

pub const WGL_SUPPORT_OPENGL_ARB: c_int = 0x2010;
pub const WGL_DOUBLE_BUFFER_ARB: c_int = 0x2011;
pub const WGL_DRAW_TO_PBUFFER_ARB: c_int = 0x202D;
pub const WGL_PBUFFER_WIDTH_ARB: c_int = 0x2034;
pub const WGL_PBUFFER_HEIGHT_ARB: c_int = 0x2035;
pub const WGL_RED_BITS_ARB: c_int = 0x2015;
pub const WGL_GREEN_BITS_ARB: c_int = 0x2017;
pub const WGL_BLUE_BITS_ARB: c_int = 0x2019;

pub type PFNWGLCREATEPBUFFERARBPROC = extern "C" fn(HDC, c_int, c_int, c_int, *const c_int) -> HPBUFFERARB;
pub type PFNWGLGETPBUFFERDCARBPROC = extern "C" fn(HPBUFFERARB) -> HDC;
pub type PFNWGLRELEASEPBUFFERDCARBPROC = extern "C" fn(HPBUFFERARB, HDC) -> c_int;
pub type PFNWGLDESTROYPBUFFERARBPROC = extern "C" fn(HPBUFFERARB) -> BOOL;
pub type PFNWGLQUERYPBUFFERARBPROC = extern "C" fn(HPBUFFERARB, c_int, *mut c_int) -> BOOL;
/***********************************************************************************************************/

// Declare Pixel Buffer function pointers.
pub static mut qwglCreatePbufferARB: Option<PFNWGLCREATEPBUFFERARBPROC> = None;
pub static mut qwglGetPbufferDCARB: Option<PFNWGLGETPBUFFERDCARBPROC> = None;
pub static mut qwglReleasePbufferDCARB: Option<PFNWGLRELEASEPBUFFERDCARBPROC> = None;
pub static mut qwglDestroyPbufferARB: Option<PFNWGLDESTROYPBUFFERARBPROC> = None;
pub static mut qwglQueryPbufferARB: Option<PFNWGLQUERYPBUFFERARBPROC> = None;


/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Render-Texture extension definitions. - AReis
/***********************************************************************************************************/
pub const WGL_BIND_TO_TEXTURE_RGBA_ARB: c_int = 0x2071;
pub const WGL_TEXTURE_FORMAT_ARB: c_int = 0x2072;
pub const WGL_TEXTURE_TARGET_ARB: c_int = 0x2073;
pub const WGL_TEXTURE_RGB_ARB: c_int = 0x2075;
pub const WGL_TEXTURE_RGBA_ARB: c_int = 0x2076;
pub const WGL_TEXTURE_2D_ARB: c_int = 0x207A;
pub const WGL_FRONT_LEFT_ARB: c_int = 0x2083;

pub type PFNWGLBINDTEXIMAGEARBPROC = extern "C" fn(HPBUFFERARB, c_int) -> BOOL;
pub type PFNWGLRELEASETEXIMAGEARBPROC = extern "C" fn(HPBUFFERARB, c_int) -> BOOL;
pub type PFNWGLSETPBUFFERATTRIBARBPROC = extern "C" fn(HPBUFFERARB, *const c_int) -> BOOL;
/***********************************************************************************************************/

// Declare Render-Texture function pointers.
pub static mut qwglBindTexImageARB: Option<PFNWGLBINDTEXIMAGEARBPROC> = None;
pub static mut qwglReleaseTexImageARB: Option<PFNWGLRELEASETEXIMAGEARBPROC> = None;
pub static mut qwglSetPbufferAttribARB: Option<PFNWGLSETPBUFFERATTRIBARBPROC> = None;


/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Vertex and Fragment Program extension definitions. - AReis
/***********************************************************************************************************/
#[cfg(not(target_os = "windows"))]
pub const GL_FRAGMENT_PROGRAM_ARB: GLenum = 0x8804;
#[cfg(not(target_os = "windows"))]
pub const GL_PROGRAM_ALU_INSTRUCTIONS_ARB: GLenum = 0x8805;
#[cfg(not(target_os = "windows"))]
pub const GL_PROGRAM_TEX_INSTRUCTIONS_ARB: GLenum = 0x8806;
#[cfg(not(target_os = "windows"))]
pub const GL_PROGRAM_TEX_INDIRECTIONS_ARB: GLenum = 0x8807;
#[cfg(not(target_os = "windows"))]
pub const GL_PROGRAM_NATIVE_ALU_INSTRUCTIONS_ARB: GLenum = 0x8808;
#[cfg(not(target_os = "windows"))]
pub const GL_PROGRAM_NATIVE_TEX_INSTRUCTIONS_ARB: GLenum = 0x8809;
#[cfg(not(target_os = "windows"))]
pub const GL_PROGRAM_NATIVE_TEX_INDIRECTIONS_ARB: GLenum = 0x880A;
#[cfg(not(target_os = "windows"))]
pub const GL_MAX_PROGRAM_ALU_INSTRUCTIONS_ARB: GLenum = 0x880B;
#[cfg(not(target_os = "windows"))]
pub const GL_MAX_PROGRAM_TEX_INSTRUCTIONS_ARB: GLenum = 0x880C;
#[cfg(not(target_os = "windows"))]
pub const GL_MAX_PROGRAM_TEX_INDIRECTIONS_ARB: GLenum = 0x880D;
#[cfg(not(target_os = "windows"))]
pub const GL_MAX_PROGRAM_NATIVE_ALU_INSTRUCTIONS_ARB: GLenum = 0x880E;
#[cfg(not(target_os = "windows"))]
pub const GL_MAX_PROGRAM_NATIVE_TEX_INSTRUCTIONS_ARB: GLenum = 0x880F;
#[cfg(not(target_os = "windows"))]
pub const GL_MAX_PROGRAM_NATIVE_TEX_INDIRECTIONS_ARB: GLenum = 0x8810;
#[cfg(not(target_os = "windows"))]
pub const GL_MAX_TEXTURE_COORDS_ARB: GLenum = 0x8871;
#[cfg(not(target_os = "windows"))]
pub const GL_MAX_TEXTURE_IMAGE_UNITS_ARB: GLenum = 0x8872;

// NOTE: These are obviously not all the vertex program flags (have you seen how many there actually are!). I'm
// only including the ones I use (to reduce code clutter), so if you need any of the other flags, just add them.
pub const GL_VERTEX_PROGRAM_ARB: GLenum = 0x8620;
pub const GL_PROGRAM_FORMAT_ASCII_ARB: GLenum = 0x8875;

pub type PFNGLPROGRAMSTRINGARBPROC = extern "C" fn(GLenum, GLenum, GLsizei, *const GLvoid);
pub type PFNGLBINDPROGRAMARBPROC = extern "C" fn(GLenum, GLuint);
pub type PFNGLDELETEPROGRAMSARBPROC = extern "C" fn(GLsizei, *const GLuint);
pub type PFNGLGENPROGRAMSARBPROC = extern "C" fn(GLsizei, *mut GLuint);
pub type PFNGLPROGRAMENVPARAMETER4DARBPROC = extern "C" fn(GLenum, GLuint, GLdouble, GLdouble, GLdouble, GLdouble);
pub type PFNGLPROGRAMENVPARAMETER4DVARBPROC = extern "C" fn(GLenum, GLuint, *const GLdouble);
pub type PFNGLPROGRAMENVPARAMETER4FARBPROC = extern "C" fn(GLenum, GLuint, GLfloat, GLfloat, GLfloat, GLfloat);
pub type PFNGLPROGRAMENVPARAMETER4FVARBPROC = extern "C" fn(GLenum, GLuint, *const GLfloat);
pub type PFNGLPROGRAMLOCALPARAMETER4DARBPROC = extern "C" fn(GLenum, GLuint, GLdouble, GLdouble, GLdouble, GLdouble);
pub type PFNGLPROGRAMLOCALPARAMETER4DVARBPROC = extern "C" fn(GLenum, GLuint, *const GLdouble);
pub type PFNGLPROGRAMLOCALPARAMETER4FARBPROC = extern "C" fn(GLenum, GLuint, GLfloat, GLfloat, GLfloat, GLfloat);
pub type PFNGLPROGRAMLOCALPARAMETER4FVARBPROC = extern "C" fn(GLenum, GLuint, *const GLfloat);
pub type PFNGLGETPROGRAMENVPARAMETERDVARBPROC = extern "C" fn(GLenum, GLuint, *mut GLdouble);
pub type PFNGLGETPROGRAMENVPARAMETERFVARBPROC = extern "C" fn(GLenum, GLuint, *mut GLfloat);
pub type PFNGLGETPROGRAMLOCALPARAMETERDVARBPROC = extern "C" fn(GLenum, GLuint, *mut GLdouble);
pub type PFNGLGETPROGRAMLOCALPARAMETERFVARBPROC = extern "C" fn(GLenum, GLuint, *mut GLfloat);
pub type PFNGLGETPROGRAMIVARBPROC = extern "C" fn(GLenum, GLenum, *mut GLint);
pub type PFNGLGETPROGRAMSTRINGARBPROC = extern "C" fn(GLenum, GLenum, *mut GLvoid);
pub type PFNGLISPROGRAMARBPROC = extern "C" fn(GLuint) -> GLboolean;
/***********************************************************************************************************/

// Declare Vertex and Fragment Program function pointers.
pub static mut qglProgramStringARB: Option<PFNGLPROGRAMSTRINGARBPROC> = None;
pub static mut qglBindProgramARB: Option<PFNGLBINDPROGRAMARBPROC> = None;
pub static mut qglDeleteProgramsARB: Option<PFNGLDELETEPROGRAMSARBPROC> = None;
pub static mut qglGenProgramsARB: Option<PFNGLGENPROGRAMSARBPROC> = None;
pub static mut qglProgramEnvParameter4dARB: Option<PFNGLPROGRAMENVPARAMETER4DARBPROC> = None;
pub static mut qglProgramEnvParameter4dvARB: Option<PFNGLPROGRAMENVPARAMETER4DVARBPROC> = None;
pub static mut qglProgramEnvParameter4fARB: Option<PFNGLPROGRAMENVPARAMETER4FARBPROC> = None;
pub static mut qglProgramEnvParameter4fvARB: Option<PFNGLPROGRAMENVPARAMETER4FVARBPROC> = None;
pub static mut qglProgramLocalParameter4dARB: Option<PFNGLPROGRAMLOCALPARAMETER4DARBPROC> = None;
pub static mut qglProgramLocalParameter4dvARB: Option<PFNGLPROGRAMLOCALPARAMETER4DVARBPROC> = None;
pub static mut qglProgramLocalParameter4fARB: Option<PFNGLPROGRAMLOCALPARAMETER4FARBPROC> = None;
pub static mut qglProgramLocalParameter4fvARB: Option<PFNGLPROGRAMLOCALPARAMETER4FVARBPROC> = None;
pub static mut qglGetProgramEnvParameterdvARB: Option<PFNGLGETPROGRAMENVPARAMETERDVARBPROC> = None;
pub static mut qglGetProgramEnvParameterfvARB: Option<PFNGLGETPROGRAMENVPARAMETERFVARBPROC> = None;
pub static mut qglGetProgramLocalParameterdvARB: Option<PFNGLGETPROGRAMLOCALPARAMETERDVARBPROC> = None;
pub static mut qglGetProgramLocalParameterfvARB: Option<PFNGLGETPROGRAMLOCALPARAMETERFVARBPROC> = None;
pub static mut qglGetProgramivARB: Option<PFNGLGETPROGRAMIVARBPROC> = None;
pub static mut qglGetProgramStringARB: Option<PFNGLGETPROGRAMSTRINGARBPROC> = None;
pub static mut qglIsProgramARB: Option<PFNGLISPROGRAMARBPROC> = None;


//
// extension constants
//


// S3TC compression constants
pub const GL_RGB_S3TC: GLenum = 0x83A0;
pub const GL_RGB4_S3TC: GLenum = 0x83A1;


// extensions will be function pointers on all platforms

pub static mut qglMultiTexCoord2fARB: Option<extern "C" fn(GLenum, GLfloat, GLfloat)> = None;
pub static mut qglActiveTextureARB: Option<extern "C" fn(GLenum)> = None;
pub static mut qglClientActiveTextureARB: Option<extern "C" fn(GLenum)> = None;

pub static mut qglLockArraysEXT: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglUnlockArraysEXT: Option<extern "C" fn()> = None;

pub static mut qglPointParameterfEXT: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglPointParameterfvEXT: Option<extern "C" fn(GLenum, *mut GLfloat)> = None;

// Added 10/23/02 by Aurelio Reis.
pub static mut qglPointParameteriNV: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglPointParameterivNV: Option<extern "C" fn(GLenum, *const GLint)> = None;

//===========================================================================

// non-windows systems will just redefine qgl* to gl*
#[cfg(all(not(target_os = "windows"), not(target_os = "linux")))]
pub mod qgl_linked {
    // qgl_linked.h included here for non-Windows, non-Linux systems
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
pub mod windows_and_linux_stubs {
// windows systems use a function pointer for each call so we can load minidrivers

pub static mut qglAccum: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglAlphaFunc: Option<extern "C" fn(GLenum, GLclampf)> = None;
pub static mut qglAreTexturesResident: Option<extern "C" fn(GLsizei, *const GLuint, *mut GLboolean) -> GLboolean> = None;
pub static mut qglArrayElement: Option<extern "C" fn(GLint)> = None;
pub static mut qglBegin: Option<extern "C" fn(GLenum)> = None;
pub static mut qglBindTexture: Option<extern "C" fn(GLenum, GLuint)> = None;
pub static mut qglBitmap: Option<extern "C" fn(GLsizei, GLsizei, GLfloat, GLfloat, GLfloat, GLfloat, *const GLubyte)> = None;
pub static mut qglBlendFunc: Option<extern "C" fn(GLenum, GLenum)> = None;
pub static mut qglCallList: Option<extern "C" fn(GLuint)> = None;
pub static mut qglCallLists: Option<extern "C" fn(GLsizei, GLenum, *const GLvoid)> = None;
pub static mut qglClear: Option<extern "C" fn(GLbitfield)> = None;
pub static mut qglClearAccum: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglClearColor: Option<extern "C" fn(GLclampf, GLclampf, GLclampf, GLclampf)> = None;
pub static mut qglClearDepth: Option<extern "C" fn(GLclampd)> = None;
pub static mut qglClearIndex: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglClearStencil: Option<extern "C" fn(GLint)> = None;
pub static mut qglClipPlane: Option<extern "C" fn(GLenum, *const GLdouble)> = None;
pub static mut qglColor3b: Option<extern "C" fn(GLbyte, GLbyte, GLbyte)> = None;
pub static mut qglColor3bv: Option<extern "C" fn(*const GLbyte)> = None;
pub static mut qglColor3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglColor3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglColor3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglColor3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglColor3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglColor3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglColor3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglColor3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglColor3ub: Option<extern "C" fn(GLubyte, GLubyte, GLubyte)> = None;
pub static mut qglColor3ubv: Option<extern "C" fn(*const GLubyte)> = None;
pub static mut qglColor3ui: Option<extern "C" fn(GLuint, GLuint, GLuint)> = None;
pub static mut qglColor3uiv: Option<extern "C" fn(*const GLuint)> = None;
pub static mut qglColor3us: Option<extern "C" fn(GLushort, GLushort, GLushort)> = None;
pub static mut qglColor3usv: Option<extern "C" fn(*const GLushort)> = None;
pub static mut qglColor4b: Option<extern "C" fn(GLbyte, GLbyte, GLbyte, GLbyte)> = None;
pub static mut qglColor4bv: Option<extern "C" fn(*const GLbyte)> = None;
pub static mut qglColor4d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglColor4dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglColor4f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglColor4fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglColor4i: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglColor4iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglColor4s: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglColor4sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglColor4ub: Option<extern "C" fn(GLubyte, GLubyte, GLubyte, GLubyte)> = None;
pub static mut qglColor4ubv: Option<extern "C" fn(*const GLubyte)> = None;
pub static mut qglColor4ui: Option<extern "C" fn(GLuint, GLuint, GLuint, GLuint)> = None;
pub static mut qglColor4uiv: Option<extern "C" fn(*const GLuint)> = None;
pub static mut qglColor4us: Option<extern "C" fn(GLushort, GLushort, GLushort, GLushort)> = None;
pub static mut qglColor4usv: Option<extern "C" fn(*const GLushort)> = None;
pub static mut qglColorMask: Option<extern "C" fn(GLboolean, GLboolean, GLboolean, GLboolean)> = None;
pub static mut qglColorMaterial: Option<extern "C" fn(GLenum, GLenum)> = None;
pub static mut qglColorPointer: Option<extern "C" fn(GLint, GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglCopyPixels: Option<extern "C" fn(GLint, GLint, GLsizei, GLsizei, GLenum)> = None;
pub static mut qglCopyTexImage1D: Option<extern "C" fn(GLenum, GLint, GLenum, GLint, GLint, GLsizei, GLint)> = None;
pub static mut qglCopyTexImage2D: Option<extern "C" fn(GLenum, GLint, GLenum, GLint, GLint, GLsizei, GLsizei, GLint)> = None;
pub static mut qglCopyTexSubImage1D: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLsizei)> = None;
pub static mut qglCopyTexSubImage2D: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLint, GLsizei, GLsizei)> = None;
pub static mut qglCullFace: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDeleteLists: Option<extern "C" fn(GLuint, GLsizei)> = None;
pub static mut qglDeleteTextures: Option<extern "C" fn(GLsizei, *const GLuint)> = None;
pub static mut qglDepthFunc: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDepthMask: Option<extern "C" fn(GLboolean)> = None;
pub static mut qglDepthRange: Option<extern "C" fn(GLclampd, GLclampd)> = None;
pub static mut qglDisable: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDisableClientState: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDrawArrays: Option<extern "C" fn(GLenum, GLint, GLsizei)> = None;
pub static mut qglDrawBuffer: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDrawElements: Option<extern "C" fn(GLenum, GLsizei, GLenum, *const GLvoid)> = None;
pub static mut qglDrawPixels: Option<extern "C" fn(GLsizei, GLsizei, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglEdgeFlag: Option<extern "C" fn(GLboolean)> = None;
pub static mut qglEdgeFlagPointer: Option<extern "C" fn(GLsizei, *const GLvoid)> = None;
pub static mut qglEdgeFlagv: Option<extern "C" fn(*const GLboolean)> = None;
pub static mut qglEnable: Option<extern "C" fn(GLenum)> = None;
pub static mut qglEnableClientState: Option<extern "C" fn(GLenum)> = None;
pub static mut qglEnd: Option<extern "C" fn()> = None;
pub static mut qglEndList: Option<extern "C" fn()> = None;
pub static mut qglEvalCoord1d: Option<extern "C" fn(GLdouble)> = None;
pub static mut qglEvalCoord1dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglEvalCoord1f: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglEvalCoord1fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglEvalCoord2d: Option<extern "C" fn(GLdouble, GLdouble)> = None;
pub static mut qglEvalCoord2dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglEvalCoord2f: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglEvalCoord2fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglEvalMesh1: Option<extern "C" fn(GLenum, GLint, GLint)> = None;
pub static mut qglEvalMesh2: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLint)> = None;
pub static mut qglEvalPoint1: Option<extern "C" fn(GLint)> = None;
pub static mut qglEvalPoint2: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglFeedbackBuffer: Option<extern "C" fn(GLsizei, GLenum, *mut GLfloat)> = None;
pub static mut qglFinish: Option<extern "C" fn()> = None;
pub static mut qglFlush: Option<extern "C" fn()> = None;
pub static mut qglFogf: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglFogfv: Option<extern "C" fn(GLenum, *const GLfloat)> = None;
pub static mut qglFogi: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglFogiv: Option<extern "C" fn(GLenum, *const GLint)> = None;
pub static mut qglFrontFace: Option<extern "C" fn(GLenum)> = None;
pub static mut qglFrustum: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglGenLists: Option<extern "C" fn(GLsizei) -> GLuint> = None;
pub static mut qglGenTextures: Option<extern "C" fn(GLsizei, *mut GLuint)> = None;
pub static mut qglGetBooleanv: Option<extern "C" fn(GLenum, *mut GLboolean)> = None;
pub static mut qglGetClipPlane: Option<extern "C" fn(GLenum, *mut GLdouble)> = None;
pub static mut qglGetDoublev: Option<extern "C" fn(GLenum, *mut GLdouble)> = None;
pub static mut qglGetError: Option<extern "C" fn() -> GLenum> = None;
pub static mut qglGetFloatv: Option<extern "C" fn(GLenum, *mut GLfloat)> = None;
pub static mut qglGetIntegerv: Option<extern "C" fn(GLenum, *mut GLint)> = None;
pub static mut qglGetLightfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetLightiv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetMapdv: Option<extern "C" fn(GLenum, GLenum, *mut GLdouble)> = None;
pub static mut qglGetMapfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetMapiv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetMaterialfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetMaterialiv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetPixelMapfv: Option<extern "C" fn(GLenum, *mut GLfloat)> = None;
pub static mut qglGetPixelMapuiv: Option<extern "C" fn(GLenum, *mut GLuint)> = None;
pub static mut qglGetPixelMapusv: Option<extern "C" fn(GLenum, *mut GLushort)> = None;
pub static mut qglGetPointerv: Option<extern "C" fn(GLenum, *mut *mut GLvoid)> = None;
pub static mut qglGetPolygonStipple: Option<extern "C" fn(*mut GLubyte)> = None;
pub static mut qglGetString: Option<extern "C" fn(GLenum) -> *const GLubyte> = None;
pub static mut qglGetTexEnvfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetTexEnviv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetTexGendv: Option<extern "C" fn(GLenum, GLenum, *mut GLdouble)> = None;
pub static mut qglGetTexGenfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetTexGeniv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetTexImage: Option<extern "C" fn(GLenum, GLint, GLenum, GLenum, *mut GLvoid)> = None;
pub static mut qglGetTexLevelParameterfv: Option<extern "C" fn(GLenum, GLint, GLenum, *mut GLfloat)> = None;
pub static mut qglGetTexLevelParameteriv: Option<extern "C" fn(GLenum, GLint, GLenum, *mut GLint)> = None;
pub static mut qglGetTexParameterfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetTexParameteriv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglHint: Option<extern "C" fn(GLenum, GLenum)> = None;
pub static mut qglIndexMask: Option<extern "C" fn(GLuint)> = None;
pub static mut qglIndexPointer: Option<extern "C" fn(GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglIndexd: Option<extern "C" fn(GLdouble)> = None;
pub static mut qglIndexdv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglIndexf: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglIndexfv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglIndexi: Option<extern "C" fn(GLint)> = None;
pub static mut qglIndexiv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglIndexs: Option<extern "C" fn(GLshort)> = None;
pub static mut qglIndexsv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglIndexub: Option<extern "C" fn(GLubyte)> = None;
pub static mut qglIndexubv: Option<extern "C" fn(*const GLubyte)> = None;
pub static mut qglInitNames: Option<extern "C" fn()> = None;
pub static mut qglInterleavedArrays: Option<extern "C" fn(GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglIsEnabled: Option<extern "C" fn(GLenum) -> GLboolean> = None;
pub static mut qglIsList: Option<extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut qglIsTexture: Option<extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut qglLightModelf: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglLightModelfv: Option<extern "C" fn(GLenum, *const GLfloat)> = None;
pub static mut qglLightModeli: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglLightModeliv: Option<extern "C" fn(GLenum, *const GLint)> = None;
pub static mut qglLightf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglLightfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglLighti: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglLightiv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglLineStipple: Option<extern "C" fn(GLint, GLushort)> = None;
pub static mut qglLineWidth: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglListBase: Option<extern "C" fn(GLuint)> = None;
pub static mut qglLoadIdentity: Option<extern "C" fn()> = None;
pub static mut qglLoadMatrixd: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglLoadMatrixf: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglLoadName: Option<extern "C" fn(GLuint)> = None;
pub static mut qglLogicOp: Option<extern "C" fn(GLenum)> = None;
pub static mut qglMap1d: Option<extern "C" fn(GLenum, GLdouble, GLdouble, GLint, GLint, *const GLdouble)> = None;
pub static mut qglMap1f: Option<extern "C" fn(GLenum, GLfloat, GLfloat, GLint, GLint, *const GLfloat)> = None;
pub static mut qglMap2d: Option<extern "C" fn(GLenum, GLdouble, GLdouble, GLint, GLint, GLdouble, GLdouble, GLint, GLint, *const GLdouble)> = None;
pub static mut qglMap2f: Option<extern "C" fn(GLenum, GLfloat, GLfloat, GLint, GLint, GLfloat, GLfloat, GLint, GLint, *const GLfloat)> = None;
pub static mut qglMapGrid1d: Option<extern "C" fn(GLint, GLdouble, GLdouble)> = None;
pub static mut qglMapGrid1f: Option<extern "C" fn(GLint, GLfloat, GLfloat)> = None;
pub static mut qglMapGrid2d: Option<extern "C" fn(GLint, GLdouble, GLdouble, GLint, GLdouble, GLdouble)> = None;
pub static mut qglMapGrid2f: Option<extern "C" fn(GLint, GLfloat, GLfloat, GLint, GLfloat, GLfloat)> = None;
pub static mut qglMaterialf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglMaterialfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglMateriali: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglMaterialiv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglMatrixMode: Option<extern "C" fn(GLenum)> = None;
pub static mut qglMultMatrixd: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglMultMatrixf: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglNewList: Option<extern "C" fn(GLuint, GLenum)> = None;
pub static mut qglNormal3b: Option<extern "C" fn(GLbyte, GLbyte, GLbyte)> = None;
pub static mut qglNormal3bv: Option<extern "C" fn(*const GLbyte)> = None;
pub static mut qglNormal3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglNormal3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglNormal3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglNormal3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglNormal3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglNormal3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglNormal3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglNormal3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglNormalPointer: Option<extern "C" fn(GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglOrtho: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglPassThrough: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglPixelMapfv: Option<extern "C" fn(GLenum, GLsizei, *const GLfloat)> = None;
pub static mut qglPixelMapuiv: Option<extern "C" fn(GLenum, GLsizei, *const GLuint)> = None;
pub static mut qglPixelMapusv: Option<extern "C" fn(GLenum, GLsizei, *const GLushort)> = None;
pub static mut qglPixelStoref: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglPixelStorei: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglPixelTransferf: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglPixelTransferi: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglPixelZoom: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglPointSize: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglPolygonMode: Option<extern "C" fn(GLenum, GLenum)> = None;
pub static mut qglPolygonOffset: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglPolygonStipple: Option<extern "C" fn(*const GLubyte)> = None;
pub static mut qglPopAttrib: Option<extern "C" fn()> = None;
pub static mut qglPopClientAttrib: Option<extern "C" fn()> = None;
pub static mut qglPopMatrix: Option<extern "C" fn()> = None;
pub static mut qglPopName: Option<extern "C" fn()> = None;
pub static mut qglPrioritizeTextures: Option<extern "C" fn(GLsizei, *const GLuint, *const GLclampf)> = None;
pub static mut qglPushAttrib: Option<extern "C" fn(GLbitfield)> = None;
pub static mut qglPushClientAttrib: Option<extern "C" fn(GLbitfield)> = None;
pub static mut qglPushMatrix: Option<extern "C" fn()> = None;
pub static mut qglPushName: Option<extern "C" fn(GLuint)> = None;
pub static mut qglRasterPos2d: Option<extern "C" fn(GLdouble, GLdouble)> = None;
pub static mut qglRasterPos2dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglRasterPos2f: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglRasterPos2fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglRasterPos2i: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglRasterPos2iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglRasterPos2s: Option<extern "C" fn(GLshort, GLshort)> = None;
pub static mut qglRasterPos2sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglRasterPos3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglRasterPos3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglRasterPos3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglRasterPos3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglRasterPos3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglRasterPos3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglRasterPos3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglRasterPos3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglRasterPos4d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglRasterPos4dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglRasterPos4f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglRasterPos4fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglRasterPos4i: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglRasterPos4iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglRasterPos4s: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglRasterPos4sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglReadBuffer: Option<extern "C" fn(GLenum)> = None;
pub static mut qglReadPixels: Option<extern "C" fn(GLint, GLint, GLsizei, GLsizei, GLenum, GLenum, *mut GLvoid)> = None;
pub static mut qglRectd: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglRectdv: Option<extern "C" fn(*const GLdouble, *const GLdouble)> = None;
pub static mut qglRectf: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglRectfv: Option<extern "C" fn(*const GLfloat, *const GLfloat)> = None;
pub static mut qglRecti: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglRectiv: Option<extern "C" fn(*const GLint, *const GLint)> = None;
pub static mut qglRects: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglRectsv: Option<extern "C" fn(*const GLshort, *const GLshort)> = None;
pub static mut qglRenderMode: Option<extern "C" fn(GLenum) -> GLint> = None;
pub static mut qglRotated: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglRotatef: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglScaled: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglScalef: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglScissor: Option<extern "C" fn(GLint, GLint, GLsizei, GLsizei)> = None;
pub static mut qglSelectBuffer: Option<extern "C" fn(GLsizei, *mut GLuint)> = None;
pub static mut qglShadeModel: Option<extern "C" fn(GLenum)> = None;
pub static mut qglStencilFunc: Option<extern "C" fn(GLenum, GLint, GLuint)> = None;
pub static mut qglStencilMask: Option<extern "C" fn(GLuint)> = None;
pub static mut qglStencilOp: Option<extern "C" fn(GLenum, GLenum, GLenum)> = None;
pub static mut qglTexCoord1d: Option<extern "C" fn(GLdouble)> = None;
pub static mut qglTexCoord1dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglTexCoord1f: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglTexCoord1fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglTexCoord1i: Option<extern "C" fn(GLint)> = None;
pub static mut qglTexCoord1iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglTexCoord1s: Option<extern "C" fn(GLshort)> = None;
pub static mut qglTexCoord1sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglTexCoord2d: Option<extern "C" fn(GLdouble, GLdouble)> = None;
pub static mut qglTexCoord2dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglTexCoord2f: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglTexCoord2fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglTexCoord2i: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglTexCoord2iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglTexCoord2s: Option<extern "C" fn(GLshort, GLshort)> = None;
pub static mut qglTexCoord2sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglTexCoord3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglTexCoord3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglTexCoord3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglTexCoord3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglTexCoord3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglTexCoord3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglTexCoord3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglTexCoord3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglTexCoord4d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglTexCoord4dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglTexCoord4f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglTexCoord4fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglTexCoord4i: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglTexCoord4iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglTexCoord4s: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglTexCoord4sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglTexCoordPointer: Option<extern "C" fn(GLint, GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglTexEnvf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglTexEnvfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglTexEnvi: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglTexEnviv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglTexGend: Option<extern "C" fn(GLenum, GLenum, GLdouble)> = None;
pub static mut qglTexGendv: Option<extern "C" fn(GLenum, GLenum, *const GLdouble)> = None;
pub static mut qglTexGenf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglTexGenfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglTexGeni: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglTexGeniv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglTexImage1D: Option<extern "C" fn(GLenum, GLint, GLint, GLsizei, GLint, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTexImage2D: Option<extern "C" fn(GLenum, GLint, GLint, GLsizei, GLsizei, GLint, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTexParameterf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglTexParameterfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglTexParameteri: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglTexParameteriv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglTexSubImage1D: Option<extern "C" fn(GLenum, GLint, GLint, GLsizei, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTexSubImage2D: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLsizei, GLsizei, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTranslated: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglTranslatef: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglVertex2d: Option<extern "C" fn(GLdouble, GLdouble)> = None;
pub static mut qglVertex2dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglVertex2f: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglVertex2fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglVertex2i: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglVertex2iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglVertex2s: Option<extern "C" fn(GLshort, GLshort)> = None;
pub static mut qglVertex2sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglVertex3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglVertex3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglVertex3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglVertex3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglVertex3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglVertex3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglVertex3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglVertex3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglVertex4d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglVertex4dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglVertex4f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglVertex4fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglVertex4i: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglVertex4iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglVertex4s: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglVertex4sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglVertexPointer: Option<extern "C" fn(GLint, GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglViewport: Option<extern "C" fn(GLint, GLint, GLsizei, GLsizei)> = None;
}

#[cfg(target_os = "windows")]
pub mod windows_specific {
pub static mut qwglCopyContext: Option<extern "C" fn(HGLRC, HGLRC, UINT) -> BOOL> = None;
pub static mut qwglCreateContext: Option<extern "C" fn(HDC) -> HGLRC> = None;
pub static mut qwglCreateLayerContext: Option<extern "C" fn(HDC, c_int) -> HGLRC> = None;
pub static mut qwglDeleteContext: Option<extern "C" fn(HGLRC) -> BOOL> = None;
pub static mut qwglGetCurrentContext: Option<extern "C" fn() -> HGLRC> = None;
pub static mut qwglGetCurrentDC: Option<extern "C" fn() -> HDC> = None;
pub static mut qwglGetProcAddress: Option<extern "C" fn(LPCSTR) -> PROC> = None;
pub static mut qwglMakeCurrent: Option<extern "C" fn(HDC, HGLRC) -> BOOL> = None;
pub static mut qwglShareLists: Option<extern "C" fn(HGLRC, HGLRC) -> BOOL> = None;
pub static mut qwglUseFontBitmaps: Option<extern "C" fn(HDC, DWORD, DWORD, DWORD) -> BOOL> = None;

pub static mut qwglUseFontOutlines: Option<extern "C" fn(HDC, DWORD, DWORD, DWORD, FLOAT, FLOAT, c_int, LPGLYPHMETRICSFLOAT) -> BOOL> = None;

pub static mut qwglDescribeLayerPlane: Option<extern "C" fn(HDC, c_int, c_int, UINT, LPLAYERPLANEDESCRIPTOR) -> BOOL> = None;
pub static mut qwglSetLayerPaletteEntries: Option<extern "C" fn(HDC, c_int, c_int, c_int, *const COLORREF) -> c_int> = None;
pub static mut qwglGetLayerPaletteEntries: Option<extern "C" fn(HDC, c_int, c_int, c_int, *mut COLORREF) -> c_int> = None;
pub static mut qwglRealizeLayerPalette: Option<extern "C" fn(HDC, c_int, BOOL) -> BOOL> = None;
pub static mut qwglSwapLayerBuffers: Option<extern "C" fn(HDC, UINT) -> BOOL> = None;

pub static mut qwglSwapIntervalEXT: Option<extern "C" fn(c_int) -> BOOL> = None;
}

#[cfg(target_os = "linux")]
pub mod linux_specific {
//FX Mesa Functions
pub static mut qfxMesaCreateContext: Option<extern "C" fn(GLuint, GrScreenResolution_t, GrScreenRefresh_t, *const c_int) -> fxMesaContext> = None;
pub static mut qfxMesaCreateBestContext: Option<extern "C" fn(GLuint, c_int, c_int, *const c_int) -> fxMesaContext> = None;
pub static mut qfxMesaDestroyContext: Option<extern "C" fn(fxMesaContext)> = None;
pub static mut qfxMesaMakeCurrent: Option<extern "C" fn(fxMesaContext)> = None;
pub static mut qfxMesaGetCurrentContext: Option<extern "C" fn() -> fxMesaContext> = None;
pub static mut qfxMesaSwapBuffers: Option<extern "C" fn()> = None;

//GLX Functions
pub static mut qglXChooseVisual: Option<extern "C" fn(*mut Display, c_int, *mut c_int) -> *mut XVisualInfo> = None;
pub static mut qglXCreateContext: Option<extern "C" fn(*mut Display, *mut XVisualInfo, GLXContext, Bool) -> GLXContext> = None;
pub static mut qglXDestroyContext: Option<extern "C" fn(*mut Display, GLXContext)> = None;
pub static mut qglXMakeCurrent: Option<extern "C" fn(*mut Display, GLXDrawable, GLXContext) -> Bool> = None;
pub static mut qglXCopyContext: Option<extern "C" fn(*mut Display, GLXContext, GLXContext, GLuint)> = None;
pub static mut qglXSwapBuffers: Option<extern "C" fn(*mut Display, GLXDrawable)> = None;
}
