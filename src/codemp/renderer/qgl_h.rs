//! QGL.H

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::*;

// Platform-specific GL includes are handled by Rust's linking at compile time.
// This module mirrors the extern declarations from the C header.

// ===== APIENTRY and WINAPI macros (no-ops in Rust, kept for documentation) =====
// #ifndef APIENTRY
// #define APIENTRY
// #endif
// #ifndef WINAPI
// #define WINAPI
// #endif

//===========================================================================

// multitexture extension definitions

/// GL_ACTIVE_TEXTURE_ARB
pub const GL_ACTIVE_TEXTURE_ARB: c_uint = 0x84E0;
/// GL_CLIENT_ACTIVE_TEXTURE_ARB
pub const GL_CLIENT_ACTIVE_TEXTURE_ARB: c_uint = 0x84E1;
/// GL_MAX_ACTIVE_TEXTURES_ARB
pub const GL_MAX_ACTIVE_TEXTURES_ARB: c_uint = 0x84E2;

/// GL_TEXTURE0_ARB
pub const GL_TEXTURE0_ARB: c_uint = 0x84C0;
/// GL_TEXTURE1_ARB
pub const GL_TEXTURE1_ARB: c_uint = 0x84C1;
/// GL_TEXTURE2_ARB
pub const GL_TEXTURE2_ARB: c_uint = 0x84C2;
/// GL_TEXTURE3_ARB
pub const GL_TEXTURE3_ARB: c_uint = 0x84C3;

/// GL_TEXTURE_RECTANGLE_EXT
pub const GL_TEXTURE_RECTANGLE_EXT: c_uint = 0x84F5;

// TTimo: FIXME
// linux needs those prototypes
// GL_VERSION_1_2 is defined after #include <gl.h>

/// GL multitexture coordinate procedures - double
pub type PFNGLMULTITEXCOORD1DARBPROC = unsafe extern "C" fn(target: c_uint, s: c_double);
/// GL multitexture coordinate procedures - double vector
pub type PFNGLMULTITEXCOORD1DVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_double);
/// GL multitexture coordinate procedures - float
pub type PFNGLMULTITEXCOORD1FARBPROC = unsafe extern "C" fn(target: c_uint, s: c_float);
/// GL multitexture coordinate procedures - float vector
pub type PFNGLMULTITEXCOORD1FVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_float);
/// GL multitexture coordinate procedures - int
pub type PFNGLMULTITEXCOORD1IARBPROC = unsafe extern "C" fn(target: c_uint, s: c_int);
/// GL multitexture coordinate procedures - int vector
pub type PFNGLMULTITEXCOORD1IVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_int);
/// GL multitexture coordinate procedures - short
pub type PFNGLMULTITEXCOORD1SARBPROC = unsafe extern "C" fn(target: c_uint, s: c_short);
/// GL multitexture coordinate procedures - short vector
pub type PFNGLMULTITEXCOORD1SVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_short);
/// GL multitexture coordinate procedures - 2d double
pub type PFNGLMULTITEXCOORD2DARBPROC = unsafe extern "C" fn(target: c_uint, s: c_double, t: c_double);
/// GL multitexture coordinate procedures - 2d double vector
pub type PFNGLMULTITEXCOORD2DVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_double);
/// GL multitexture coordinate procedures - 2f float
pub type PFNGLMULTITEXCOORD2FARBPROC = unsafe extern "C" fn(target: c_uint, s: c_float, t: c_float);
/// GL multitexture coordinate procedures - 2f float vector
pub type PFNGLMULTITEXCOORD2FVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_float);
/// GL multitexture coordinate procedures - 2i int
pub type PFNGLMULTITEXCOORD2IARBPROC = unsafe extern "C" fn(target: c_uint, s: c_int, t: c_int);
/// GL multitexture coordinate procedures - 2i int vector
pub type PFNGLMULTITEXCOORD2IVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_int);
/// GL multitexture coordinate procedures - 2s short
pub type PFNGLMULTITEXCOORD2SARBPROC = unsafe extern "C" fn(target: c_uint, s: c_short, t: c_short);
/// GL multitexture coordinate procedures - 2s short vector
pub type PFNGLMULTITEXCOORD2SVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_short);
/// GL multitexture coordinate procedures - 3d double
pub type PFNGLMULTITEXCOORD3DARBPROC = unsafe extern "C" fn(target: c_uint, s: c_double, t: c_double, r: c_double);
/// GL multitexture coordinate procedures - 3d double vector
pub type PFNGLMULTITEXCOORD3DVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_double);
/// GL multitexture coordinate procedures - 3f float
pub type PFNGLMULTITEXCOORD3FARBPROC = unsafe extern "C" fn(target: c_uint, s: c_float, t: c_float, r: c_float);
/// GL multitexture coordinate procedures - 3f float vector
pub type PFNGLMULTITEXCOORD3FVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_float);
/// GL multitexture coordinate procedures - 3i int
pub type PFNGLMULTITEXCOORD3IARBPROC = unsafe extern "C" fn(target: c_uint, s: c_int, t: c_int, r: c_int);
/// GL multitexture coordinate procedures - 3i int vector
pub type PFNGLMULTITEXCOORD3IVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_int);
/// GL multitexture coordinate procedures - 3s short
pub type PFNGLMULTITEXCOORD3SARBPROC = unsafe extern "C" fn(target: c_uint, s: c_short, t: c_short, r: c_short);
/// GL multitexture coordinate procedures - 3s short vector
pub type PFNGLMULTITEXCOORD3SVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_short);
/// GL multitexture coordinate procedures - 4d double
pub type PFNGLMULTITEXCOORD4DARBPROC = unsafe extern "C" fn(target: c_uint, s: c_double, t: c_double, r: c_double, q: c_double);
/// GL multitexture coordinate procedures - 4d double vector
pub type PFNGLMULTITEXCOORD4DVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_double);
/// GL multitexture coordinate procedures - 4f float
pub type PFNGLMULTITEXCOORD4FARBPROC = unsafe extern "C" fn(target: c_uint, s: c_float, t: c_float, r: c_float, q: c_float);
/// GL multitexture coordinate procedures - 4f float vector
pub type PFNGLMULTITEXCOORD4FVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_float);
/// GL multitexture coordinate procedures - 4i int
pub type PFNGLMULTITEXCOORD4IARBPROC = unsafe extern "C" fn(target: c_uint, s: c_int, t: c_int, r: c_int, q: c_int);
/// GL multitexture coordinate procedures - 4i int vector
pub type PFNGLMULTITEXCOORD4IVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_int);
/// GL multitexture coordinate procedures - 4s short
pub type PFNGLMULTITEXCOORD4SARBPROC = unsafe extern "C" fn(target: c_uint, s: c_short, t: c_short, r: c_short, q: c_short);
/// GL multitexture coordinate procedures - 4s short vector
pub type PFNGLMULTITEXCOORD4SVARBPROC = unsafe extern "C" fn(target: c_uint, v: *const c_short);
/// GL active texture ARB
pub type PFNGLACTIVETEXTUREARBPROC = unsafe extern "C" fn(target: c_uint);
/// GL client active texture ARB
pub type PFNGLCLIENTACTIVETEXTUREARBPROC = unsafe extern "C" fn(target: c_uint);

// Steps to adding a new extension:
//	- Add the typedef and function pointer externs here.
//	- Define the function pointer in tr_init.cpp and possibly add a cvar to track your ext status.
//	- Load the extension in win_glimp.cpp.

/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Register Combiner extension definitions. - AReis
/***********************************************************************************************************/
// NOTE: These are obviously not all the regcom flags. I'm only including the ones I use (to reduce code clutter), so
// if you need any of the other flags, just add them.
/// GL_REGISTER_COMBINERS_NV
pub const GL_REGISTER_COMBINERS_NV: c_uint = 0x8522;
/// GL_COMBINER0_NV
pub const GL_COMBINER0_NV: c_uint = 0x8550;
/// GL_COMBINER1_NV
pub const GL_COMBINER1_NV: c_uint = 0x8551;
/// GL_COMBINER2_NV
pub const GL_COMBINER2_NV: c_uint = 0x8552;
/// GL_COMBINER3_NV
pub const GL_COMBINER3_NV: c_uint = 0x8553;
/// GL_COMBINER4_NV
pub const GL_COMBINER4_NV: c_uint = 0x8554;
/// GL_COMBINER5_NV
pub const GL_COMBINER5_NV: c_uint = 0x8555;
/// GL_COMBINER6_NV
pub const GL_COMBINER6_NV: c_uint = 0x8556;
/// GL_COMBINER7_NV
pub const GL_COMBINER7_NV: c_uint = 0x8557;
/// GL_NUM_GENERAL_COMBINERS_NV
pub const GL_NUM_GENERAL_COMBINERS_NV: c_uint = 0x854E;
/// GL_VARIABLE_A_NV
pub const GL_VARIABLE_A_NV: c_uint = 0x8523;
/// GL_VARIABLE_B_NV
pub const GL_VARIABLE_B_NV: c_uint = 0x8524;
/// GL_VARIABLE_C_NV
pub const GL_VARIABLE_C_NV: c_uint = 0x8525;
/// GL_VARIABLE_D_NV
pub const GL_VARIABLE_D_NV: c_uint = 0x8526;
/// GL_VARIABLE_E_NV
pub const GL_VARIABLE_E_NV: c_uint = 0x8527;
/// GL_VARIABLE_F_NV
pub const GL_VARIABLE_F_NV: c_uint = 0x8528;
/// GL_VARIABLE_G_NV
pub const GL_VARIABLE_G_NV: c_uint = 0x8529;
/// GL_DISCARD_NV
pub const GL_DISCARD_NV: c_uint = 0x8530;
/// GL_CONSTANT_COLOR0_NV
pub const GL_CONSTANT_COLOR0_NV: c_uint = 0x852A;
/// GL_CONSTANT_COLOR1_NV
pub const GL_CONSTANT_COLOR1_NV: c_uint = 0x852B;
/// GL_SPARE0_NV
pub const GL_SPARE0_NV: c_uint = 0x852E;
/// GL_SPARE1_NV
pub const GL_SPARE1_NV: c_uint = 0x852F;
/// GL_UNSIGNED_IDENTITY_NV
pub const GL_UNSIGNED_IDENTITY_NV: c_uint = 0x8536;
/// GL_UNSIGNED_INVERT_NV
pub const GL_UNSIGNED_INVERT_NV: c_uint = 0x8537;

/// GL combiner parameter fv NV
pub type PFNGLCOMBINERPARAMETERFVNV = unsafe extern "C" fn(pname: c_uint, params: *const c_float);
/// GL combiner parameter iv NV
pub type PFNGLCOMBINERPARAMETERIVNV = unsafe extern "C" fn(pname: c_uint, params: *const c_int);
/// GL combiner parameter f NV
pub type PFNGLCOMBINERPARAMETERFNV = unsafe extern "C" fn(pname: c_uint, param: c_float);
/// GL combiner parameter i NV
pub type PFNGLCOMBINERPARAMETERINV = unsafe extern "C" fn(pname: c_uint, param: c_int);
/// GL combiner input NV
pub type PFNGLCOMBINERINPUTNV = unsafe extern "C" fn(stage: c_uint, portion: c_uint, variable: c_uint, input: c_uint, mapping: c_uint, componentUsage: c_uint);
/// GL combiner output NV
pub type PFNGLCOMBINEROUTPUTNV = unsafe extern "C" fn(stage: c_uint, portion: c_uint, abOutput: c_uint, cdOutput: c_uint, sumOutput: c_uint, scale: c_uint, bias: c_uint, abDotProduct: c_uchar, cdDotProduct: c_uchar, muxSum: c_uchar);
/// GL final combiner input NV
pub type PFNGLFINALCOMBINERINPUTNV = unsafe extern "C" fn(variable: c_uint, input: c_uint, mapping: c_uint, componentUsage: c_uint);

/// GL get combiner input parameter fv NV
pub type PFNGLGETCOMBINERINPUTPARAMETERFVNV = unsafe extern "C" fn(stage: c_uint, portion: c_uint, variable: c_uint, pname: c_uint, params: *mut c_float);
/// GL get combiner input parameter iv NV
pub type PFNGLGETCOMBINERINPUTPARAMETERIVNV = unsafe extern "C" fn(stage: c_uint, portion: c_uint, variable: c_uint, pname: c_uint, params: *mut c_int);
/// GL get combiner output parameter fv NV
pub type PFNGLGETCOMBINEROUTPUTPARAMETERFVNV = unsafe extern "C" fn(stage: c_uint, portion: c_uint, pname: c_uint, params: *mut c_float);
/// GL get combiner output parameter iv NV
pub type PFNGLGETCOMBINEROUTPUTPARAMETERIVNV = unsafe extern "C" fn(stage: c_uint, portion: c_uint, pname: c_uint, params: *mut c_int);
/// GL get final combiner input parameter fv NV
pub type PFNGLGETFINALCOMBINERINPUTPARAMETERFVNV = unsafe extern "C" fn(variable: c_uint, pname: c_uint, params: *mut c_float);
/// GL get final combiner input parameter iv NV
pub type PFNGLGETFINALCOMBINERINPUTPARAMETERIVNV = unsafe extern "C" fn(variable: c_uint, pname: c_uint, params: *mut c_float);
/***********************************************************************************************************/

// Declare Register Combiners function pointers.
extern "C" {
    pub static mut qglCombinerParameterfvNV: Option<PFNGLCOMBINERPARAMETERFVNV>;
    pub static mut qglCombinerParameterivNV: Option<PFNGLCOMBINERPARAMETERIVNV>;
    pub static mut qglCombinerParameterfNV: Option<PFNGLCOMBINERPARAMETERFNV>;
    pub static mut qglCombinerParameteriNV: Option<PFNGLCOMBINERPARAMETERINV>;
    pub static mut qglCombinerInputNV: Option<PFNGLCOMBINERINPUTNV>;
    pub static mut qglCombinerOutputNV: Option<PFNGLCOMBINEROUTPUTNV>;
    pub static mut qglFinalCombinerInputNV: Option<PFNGLFINALCOMBINERINPUTNV>;
    pub static mut qglGetCombinerInputParameterfvNV: Option<PFNGLGETCOMBINERINPUTPARAMETERFVNV>;
    pub static mut qglGetCombinerInputParameterivNV: Option<PFNGLGETCOMBINERINPUTPARAMETERIVNV>;
    pub static mut qglGetCombinerOutputParameterfvNV: Option<PFNGLGETCOMBINEROUTPUTPARAMETERFVNV>;
    pub static mut qglGetCombinerOutputParameterivNV: Option<PFNGLGETCOMBINEROUTPUTPARAMETERIVNV>;
    pub static mut qglGetFinalCombinerInputParameterfvNV: Option<PFNGLGETFINALCOMBINERINPUTPARAMETERFVNV>;
    pub static mut qglGetFinalCombinerInputParameterivNV: Option<PFNGLGETFINALCOMBINERINPUTPARAMETERIVNV>;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Pixel Format extension definitions. - AReis
/***********************************************************************************************************/
/// WGL_COLOR_BITS_ARB
pub const WGL_COLOR_BITS_ARB: c_int = 0x2014;
/// WGL_ALPHA_BITS_ARB
pub const WGL_ALPHA_BITS_ARB: c_int = 0x201B;
/// WGL_DEPTH_BITS_ARB
pub const WGL_DEPTH_BITS_ARB: c_int = 0x2022;
/// WGL_STENCIL_BITS_ARB
pub const WGL_STENCIL_BITS_ARB: c_int = 0x2023;

// Windows platform types - stub declarations for cross-platform compilation
// These are not actual Windows types in Rust; they are opaque handles.
#[cfg(windows)]
pub type HDC = *mut c_void;
#[cfg(not(windows))]
pub type HDC = *mut c_void;

/// WGL get pixel format attrib iv ARB
pub type PFNWGLGETPIXELFORMATATTRIBIVARBPROC = unsafe extern "C" fn(hdc: HDC, iPixelFormat: c_int, iLayerPlane: c_int, nAttributes: c_uint, piAttributes: *const c_int, piValues: *mut c_int) -> c_int;
/// WGL get pixel format attrib fv ARB
pub type PFNWGLGETPIXELFORMATATTRIBFVARBPROC = unsafe extern "C" fn(hdc: HDC, iPixelFormat: c_int, iLayerPlane: c_int, nAttributes: c_uint, piAttributes: *const c_int, pfValues: *mut c_float) -> c_int;
/// WGL choose pixel format ARB
pub type PFNWGLCHOOSEPIXELFORMATARBPROC = unsafe extern "C" fn(hdc: HDC, piAttribIList: *const c_int, pfAttribFList: *const c_float, nMaxFormats: c_uint, piFormats: *mut c_int, nNumFormats: *mut c_uint) -> c_int;
/***********************************************************************************************************/

// Declare Pixel Format function pointers.
extern "C" {
    pub static mut qwglGetPixelFormatAttribivARB: Option<PFNWGLGETPIXELFORMATATTRIBIVARBPROC>;
    pub static mut qwglGetPixelFormatAttribfvARB: Option<PFNWGLGETPIXELFORMATATTRIBFVARBPROC>;
    pub static mut qwglChoosePixelFormatARB: Option<PFNWGLCHOOSEPIXELFORMATARBPROC>;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Pixel Buffer extension definitions. - AReis
/***********************************************************************************************************/
// DECLARE_HANDLE(HPBUFFERARB) - stub for cross-platform
pub type HPBUFFERARB = *mut c_void;

/// WGL_SUPPORT_OPENGL_ARB
pub const WGL_SUPPORT_OPENGL_ARB: c_int = 0x2010;
/// WGL_DOUBLE_BUFFER_ARB
pub const WGL_DOUBLE_BUFFER_ARB: c_int = 0x2011;
/// WGL_DRAW_TO_PBUFFER_ARB
pub const WGL_DRAW_TO_PBUFFER_ARB: c_int = 0x202D;
/// WGL_PBUFFER_WIDTH_ARB
pub const WGL_PBUFFER_WIDTH_ARB: c_int = 0x2034;
/// WGL_PBUFFER_HEIGHT_ARB
pub const WGL_PBUFFER_HEIGHT_ARB: c_int = 0x2035;
/// WGL_RED_BITS_ARB
pub const WGL_RED_BITS_ARB: c_int = 0x2015;
/// WGL_GREEN_BITS_ARB
pub const WGL_GREEN_BITS_ARB: c_int = 0x2017;
/// WGL_BLUE_BITS_ARB
pub const WGL_BLUE_BITS_ARB: c_int = 0x2019;

/// WGL create pbuffer ARB
pub type PFNWGLCREATEPBUFFERARBPROC = unsafe extern "C" fn(hDC: HDC, iPixelFormat: c_int, iWidth: c_int, iHeight: c_int, piAttribList: *const c_int) -> HPBUFFERARB;
/// WGL get pbuffer DC ARB
pub type PFNWGLGETPBUFFERDCARBPROC = unsafe extern "C" fn(hPbuffer: HPBUFFERARB) -> HDC;
/// WGL release pbuffer DC ARB
pub type PFNWGLRELEASEPBUFFERDCARBPROC = unsafe extern "C" fn(hPbuffer: HPBUFFERARB, hDC: HDC) -> c_int;
/// WGL destroy pbuffer ARB
pub type PFNWGLDESTROYPBUFFERARBPROC = unsafe extern "C" fn(hPbuffer: HPBUFFERARB) -> c_int;
/// WGL query pbuffer ARB
pub type PFNWGLQUERYPBUFFERARBPROC = unsafe extern "C" fn(hPbuffer: HPBUFFERARB, iAttribute: c_int, piValue: *mut c_int) -> c_int;
/***********************************************************************************************************/

// Declare Pixel Buffer function pointers.
extern "C" {
    pub static mut qwglCreatePbufferARB: Option<PFNWGLCREATEPBUFFERARBPROC>;
    pub static mut qwglGetPbufferDCARB: Option<PFNWGLGETPBUFFERDCARBPROC>;
    pub static mut qwglReleasePbufferDCARB: Option<PFNWGLRELEASEPBUFFERDCARBPROC>;
    pub static mut qwglDestroyPbufferARB: Option<PFNWGLDESTROYPBUFFERARBPROC>;
    pub static mut qwglQueryPbufferARB: Option<PFNWGLQUERYPBUFFERARBPROC>;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Render-Texture extension definitions. - AReis
/***********************************************************************************************************/
/// WGL_BIND_TO_TEXTURE_RGBA_ARB
pub const WGL_BIND_TO_TEXTURE_RGBA_ARB: c_int = 0x2071;
/// WGL_TEXTURE_FORMAT_ARB
pub const WGL_TEXTURE_FORMAT_ARB: c_int = 0x2072;
/// WGL_TEXTURE_TARGET_ARB
pub const WGL_TEXTURE_TARGET_ARB: c_int = 0x2073;
/// WGL_TEXTURE_RGB_ARB
pub const WGL_TEXTURE_RGB_ARB: c_int = 0x2075;
/// WGL_TEXTURE_RGBA_ARB
pub const WGL_TEXTURE_RGBA_ARB: c_int = 0x2076;
/// WGL_TEXTURE_2D_ARB
pub const WGL_TEXTURE_2D_ARB: c_int = 0x207A;
/// WGL_FRONT_LEFT_ARB
pub const WGL_FRONT_LEFT_ARB: c_int = 0x2083;

/// WGL bind tex image ARB
pub type PFNWGLBINDTEXIMAGEARBPROC = unsafe extern "C" fn(hPbuffer: HPBUFFERARB, iBuffer: c_int) -> c_int;
/// WGL release tex image ARB
pub type PFNWGLRELEASETEXIMAGEARBPROC = unsafe extern "C" fn(hPbuffer: HPBUFFERARB, iBuffer: c_int) -> c_int;
/// WGL set pbuffer attrib ARB
pub type PFNWGLSETPBUFFERATTRIBARBPROC = unsafe extern "C" fn(hPbuffer: HPBUFFERARB, piAttribList: *const c_int) -> c_int;
/***********************************************************************************************************/

// Declare Render-Texture function pointers.
extern "C" {
    pub static mut qwglBindTexImageARB: Option<PFNWGLBINDTEXIMAGEARBPROC>;
    pub static mut qwglReleaseTexImageARB: Option<PFNWGLRELEASETEXIMAGEARBPROC>;
    pub static mut qwglSetPbufferAttribARB: Option<PFNWGLSETPBUFFERATTRIBARBPROC>;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Vertex and Fragment Program extension definitions. - AReis
/***********************************************************************************************************/
// NOTE: These are obviously not all the vertex program flags (have you seen how many there actually are!). I'm
// only including the ones I use (to reduce code clutter), so if you need any of the other flags, just add them.

/// GL_FRAGMENT_PROGRAM_ARB
pub const GL_FRAGMENT_PROGRAM_ARB: c_uint = 0x8804;
/// GL_PROGRAM_ALU_INSTRUCTIONS_ARB
pub const GL_PROGRAM_ALU_INSTRUCTIONS_ARB: c_uint = 0x8805;
/// GL_PROGRAM_TEX_INSTRUCTIONS_ARB
pub const GL_PROGRAM_TEX_INSTRUCTIONS_ARB: c_uint = 0x8806;
/// GL_PROGRAM_TEX_INDIRECTIONS_ARB
pub const GL_PROGRAM_TEX_INDIRECTIONS_ARB: c_uint = 0x8807;
/// GL_PROGRAM_NATIVE_ALU_INSTRUCTIONS_ARB
pub const GL_PROGRAM_NATIVE_ALU_INSTRUCTIONS_ARB: c_uint = 0x8808;
/// GL_PROGRAM_NATIVE_TEX_INSTRUCTIONS_ARB
pub const GL_PROGRAM_NATIVE_TEX_INSTRUCTIONS_ARB: c_uint = 0x8809;
/// GL_PROGRAM_NATIVE_TEX_INDIRECTIONS_ARB
pub const GL_PROGRAM_NATIVE_TEX_INDIRECTIONS_ARB: c_uint = 0x880A;
/// GL_MAX_PROGRAM_ALU_INSTRUCTIONS_ARB
pub const GL_MAX_PROGRAM_ALU_INSTRUCTIONS_ARB: c_uint = 0x880B;
/// GL_MAX_PROGRAM_TEX_INSTRUCTIONS_ARB
pub const GL_MAX_PROGRAM_TEX_INSTRUCTIONS_ARB: c_uint = 0x880C;
/// GL_MAX_PROGRAM_TEX_INDIRECTIONS_ARB
pub const GL_MAX_PROGRAM_TEX_INDIRECTIONS_ARB: c_uint = 0x880D;
/// GL_MAX_PROGRAM_NATIVE_ALU_INSTRUCTIONS_ARB
pub const GL_MAX_PROGRAM_NATIVE_ALU_INSTRUCTIONS_ARB: c_uint = 0x880E;
/// GL_MAX_PROGRAM_NATIVE_TEX_INSTRUCTIONS_ARB
pub const GL_MAX_PROGRAM_NATIVE_TEX_INSTRUCTIONS_ARB: c_uint = 0x880F;
/// GL_MAX_PROGRAM_NATIVE_TEX_INDIRECTIONS_ARB
pub const GL_MAX_PROGRAM_NATIVE_TEX_INDIRECTIONS_ARB: c_uint = 0x8810;
/// GL_MAX_TEXTURE_COORDS_ARB
pub const GL_MAX_TEXTURE_COORDS_ARB: c_uint = 0x8871;
/// GL_MAX_TEXTURE_IMAGE_UNITS_ARB
pub const GL_MAX_TEXTURE_IMAGE_UNITS_ARB: c_uint = 0x8872;

/// GL_VERTEX_PROGRAM_ARB
pub const GL_VERTEX_PROGRAM_ARB: c_uint = 0x8620;
/// GL_PROGRAM_FORMAT_ASCII_ARB
pub const GL_PROGRAM_FORMAT_ASCII_ARB: c_uint = 0x8875;

/// GL program string ARB
pub type PFNGLPROGRAMSTRINGARBPROC = unsafe extern "C" fn(target: c_uint, format: c_uint, len: c_int, string: *const c_void);
/// GL bind program ARB
pub type PFNGLBINDPROGRAMARBPROC = unsafe extern "C" fn(target: c_uint, program: c_uint);
/// GL delete programs ARB
pub type PFNGLDELETEPROGRAMSARBPROC = unsafe extern "C" fn(n: c_int, programs: *const c_uint);
/// GL gen programs ARB
pub type PFNGLGENPROGRAMSARBPROC = unsafe extern "C" fn(n: c_int, programs: *mut c_uint);
/// GL program env parameter 4d ARB
pub type PFNGLPROGRAMENVPARAMETER4DARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, x: c_double, y: c_double, z: c_double, w: c_double);
/// GL program env parameter 4dv ARB
pub type PFNGLPROGRAMENVPARAMETER4DVARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, params: *const c_double);
/// GL program env parameter 4f ARB
pub type PFNGLPROGRAMENVPARAMETER4FARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, x: c_float, y: c_float, z: c_float, w: c_float);
/// GL program env parameter 4fv ARB
pub type PFNGLPROGRAMENVPARAMETER4FVARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, params: *const c_float);
/// GL program local parameter 4d ARB
pub type PFNGLPROGRAMLOCALPARAMETER4DARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, x: c_double, y: c_double, z: c_double, w: c_double);
/// GL program local parameter 4dv ARB
pub type PFNGLPROGRAMLOCALPARAMETER4DVARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, params: *const c_double);
/// GL program local parameter 4f ARB
pub type PFNGLPROGRAMLOCALPARAMETER4FARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, x: c_float, y: c_float, z: c_float, w: c_float);
/// GL program local parameter 4fv ARB
pub type PFNGLPROGRAMLOCALPARAMETER4FVARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, params: *const c_float);
/// GL get program env parameter dv ARB
pub type PFNGLGETPROGRAMENVPARAMETERDVARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, params: *mut c_double);
/// GL get program env parameter fv ARB
pub type PFNGLGETPROGRAMENVPARAMETERFVARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, params: *mut c_float);
/// GL get program local parameter dv ARB
pub type PFNGLGETPROGRAMLOCALPARAMETERDVARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, params: *mut c_double);
/// GL get program local parameter fv ARB
pub type PFNGLGETPROGRAMLOCALPARAMETERFVARBPROC = unsafe extern "C" fn(target: c_uint, index: c_uint, params: *mut c_float);
/// GL get program iv ARB
pub type PFNGLGETPROGRAMIVARBPROC = unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *mut c_int);
/// GL get program string ARB
pub type PFNGLGETPROGRAMSTRINGARBPROC = unsafe extern "C" fn(target: c_uint, pname: c_uint, string: *mut c_void);
/// GL is program ARB
pub type PFNGLISPROGRAMARBPROC = unsafe extern "C" fn(program: c_uint) -> c_uchar;
/***********************************************************************************************************/

// Declare Vertex and Fragment Program function pointers.
extern "C" {
    pub static mut qglProgramStringARB: Option<PFNGLPROGRAMSTRINGARBPROC>;
    pub static mut qglBindProgramARB: Option<PFNGLBINDPROGRAMARBPROC>;
    pub static mut qglDeleteProgramsARB: Option<PFNGLDELETEPROGRAMSARBPROC>;
    pub static mut qglGenProgramsARB: Option<PFNGLGENPROGRAMSARBPROC>;
    pub static mut qglProgramEnvParameter4dARB: Option<PFNGLPROGRAMENVPARAMETER4DARBPROC>;
    pub static mut qglProgramEnvParameter4dvARB: Option<PFNGLPROGRAMENVPARAMETER4DVARBPROC>;
    pub static mut qglProgramEnvParameter4fARB: Option<PFNGLPROGRAMENVPARAMETER4FARBPROC>;
    pub static mut qglProgramEnvParameter4fvARB: Option<PFNGLPROGRAMENVPARAMETER4FVARBPROC>;
    pub static mut qglProgramLocalParameter4dARB: Option<PFNGLPROGRAMLOCALPARAMETER4DARBPROC>;
    pub static mut qglProgramLocalParameter4dvARB: Option<PFNGLPROGRAMLOCALPARAMETER4DVARBPROC>;
    pub static mut qglProgramLocalParameter4fARB: Option<PFNGLPROGRAMLOCALPARAMETER4FARBPROC>;
    pub static mut qglProgramLocalParameter4fvARB: Option<PFNGLPROGRAMLOCALPARAMETER4FVARBPROC>;
    pub static mut qglGetProgramEnvParameterdvARB: Option<PFNGLGETPROGRAMENVPARAMETERDVARBPROC>;
    pub static mut qglGetProgramEnvParameterfvARB: Option<PFNGLGETPROGRAMENVPARAMETERFVARBPROC>;
    pub static mut qglGetProgramLocalParameterdvARB: Option<PFNGLGETPROGRAMLOCALPARAMETERDVARBPROC>;
    pub static mut qglGetProgramLocalParameterfvARB: Option<PFNGLGETPROGRAMLOCALPARAMETERFVARBPROC>;
    pub static mut qglGetProgramivARB: Option<PFNGLGETPROGRAMIVARBPROC>;
    pub static mut qglGetProgramStringARB: Option<PFNGLGETPROGRAMSTRINGARBPROC>;
    pub static mut qglIsProgramARB: Option<PFNGLISPROGRAMARBPROC>;
}

//===========================================================================
// extension constants

// S3TC compression constants
/// GL_RGB_S3TC
pub const GL_RGB_S3TC: c_uint = 0x83A0;
/// GL_RGB4_S3TC
pub const GL_RGB4_S3TC: c_uint = 0x83A1;

// extensions will be function pointers on all platforms

extern "C" {
    pub static mut qglMultiTexCoord2fARB: Option<unsafe extern "C" fn(texture: c_uint, s: c_float, t: c_float)>;
    pub static mut qglActiveTextureARB: Option<unsafe extern "C" fn(texture: c_uint)>;
    pub static mut qglClientActiveTextureARB: Option<unsafe extern "C" fn(texture: c_uint)>;

    pub static mut qglLockArraysEXT: Option<unsafe extern "C" fn(c_int, c_int)>;
    pub static mut qglUnlockArraysEXT: Option<unsafe extern "C" fn()>;

    pub static mut qglPointParameterfEXT: Option<unsafe extern "C" fn(c_uint, c_float)>;
    pub static mut qglPointParameterfvEXT: Option<unsafe extern "C" fn(c_uint, *mut c_float)>;

    // 3d textures -rww
    pub static mut qglTexImage3DEXT: Option<unsafe extern "C" fn(c_uint, c_int, c_uint, c_int, c_int, c_int, c_int, c_uint, c_uint, *const c_void)>;
    pub static mut qglTexSubImage3DEXT: Option<unsafe extern "C" fn(c_uint, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_uint, c_uint, *const c_void)>;
}

// GL function pointers - core OpenGL functions
// windows systems use a function pointer for each call so we can load minidrivers

extern "C" {
    pub static mut qglAccum: Option<unsafe extern "C" fn(op: c_uint, value: c_float)>;
    pub static mut qglAlphaFunc: Option<unsafe extern "C" fn(func: c_uint, ref_: c_float)>;
    pub static mut qglAreTexturesResident: Option<unsafe extern "C" fn(n: c_int, textures: *const c_uint, residences: *mut c_uchar) -> c_uchar>;
    pub static mut qglArrayElement: Option<unsafe extern "C" fn(i: c_int)>;
    pub static mut qglBegin: Option<unsafe extern "C" fn(mode: c_uint)>;
    pub static mut qglBindTexture: Option<unsafe extern "C" fn(target: c_uint, texture: c_uint)>;
    pub static mut qglBitmap: Option<unsafe extern "C" fn(width: c_int, height: c_int, xorig: c_float, yorig: c_float, xmove: c_float, ymove: c_float, bitmap: *const c_uchar)>;
    pub static mut qglBlendFunc: Option<unsafe extern "C" fn(sfactor: c_uint, dfactor: c_uint)>;
    pub static mut qglCallList: Option<unsafe extern "C" fn(list: c_uint)>;
    pub static mut qglCallLists: Option<unsafe extern "C" fn(n: c_int, type_: c_uint, lists: *const c_void)>;
    pub static mut qglClear: Option<unsafe extern "C" fn(mask: c_uint)>;
    pub static mut qglClearAccum: Option<unsafe extern "C" fn(red: c_float, green: c_float, blue: c_float, alpha: c_float)>;
    pub static mut qglClearColor: Option<unsafe extern "C" fn(red: c_float, green: c_float, blue: c_float, alpha: c_float)>;
    pub static mut qglClearDepth: Option<unsafe extern "C" fn(depth: c_double)>;
    pub static mut qglClearIndex: Option<unsafe extern "C" fn(c: c_float)>;
    pub static mut qglClearStencil: Option<unsafe extern "C" fn(s: c_int)>;
    pub static mut qglClipPlane: Option<unsafe extern "C" fn(plane: c_uint, equation: *const c_double)>;
    pub static mut qglColor3b: Option<unsafe extern "C" fn(red: c_schar, green: c_schar, blue: c_schar)>;
    pub static mut qglColor3bv: Option<unsafe extern "C" fn(v: *const c_schar)>;
    pub static mut qglColor3d: Option<unsafe extern "C" fn(red: c_double, green: c_double, blue: c_double)>;
    pub static mut qglColor3dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglColor3f: Option<unsafe extern "C" fn(red: c_float, green: c_float, blue: c_float)>;
    pub static mut qglColor3fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglColor3i: Option<unsafe extern "C" fn(red: c_int, green: c_int, blue: c_int)>;
    pub static mut qglColor3iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglColor3s: Option<unsafe extern "C" fn(red: c_short, green: c_short, blue: c_short)>;
    pub static mut qglColor3sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglColor3ub: Option<unsafe extern "C" fn(red: c_uchar, green: c_uchar, blue: c_uchar)>;
    pub static mut qglColor3ubv: Option<unsafe extern "C" fn(v: *const c_uchar)>;
    pub static mut qglColor3ui: Option<unsafe extern "C" fn(red: c_uint, green: c_uint, blue: c_uint)>;
    pub static mut qglColor3uiv: Option<unsafe extern "C" fn(v: *const c_uint)>;
    pub static mut qglColor3us: Option<unsafe extern "C" fn(red: c_ushort, green: c_ushort, blue: c_ushort)>;
    pub static mut qglColor3usv: Option<unsafe extern "C" fn(v: *const c_ushort)>;
    pub static mut qglColor4b: Option<unsafe extern "C" fn(red: c_schar, green: c_schar, blue: c_schar, alpha: c_schar)>;
    pub static mut qglColor4bv: Option<unsafe extern "C" fn(v: *const c_schar)>;
    pub static mut qglColor4d: Option<unsafe extern "C" fn(red: c_double, green: c_double, blue: c_double, alpha: c_double)>;
    pub static mut qglColor4dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglColor4f: Option<unsafe extern "C" fn(red: c_float, green: c_float, blue: c_float, alpha: c_float)>;
    pub static mut qglColor4fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglColor4i: Option<unsafe extern "C" fn(red: c_int, green: c_int, blue: c_int, alpha: c_int)>;
    pub static mut qglColor4iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglColor4s: Option<unsafe extern "C" fn(red: c_short, green: c_short, blue: c_short, alpha: c_short)>;
    pub static mut qglColor4sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglColor4ub: Option<unsafe extern "C" fn(red: c_uchar, green: c_uchar, blue: c_uchar, alpha: c_uchar)>;
    pub static mut qglColor4ubv: Option<unsafe extern "C" fn(v: *const c_uchar)>;
    pub static mut qglColor4ui: Option<unsafe extern "C" fn(red: c_uint, green: c_uint, blue: c_uint, alpha: c_uint)>;
    pub static mut qglColor4uiv: Option<unsafe extern "C" fn(v: *const c_uint)>;
    pub static mut qglColor4us: Option<unsafe extern "C" fn(red: c_ushort, green: c_ushort, blue: c_ushort, alpha: c_ushort)>;
    pub static mut qglColor4usv: Option<unsafe extern "C" fn(v: *const c_ushort)>;
    pub static mut qglColorMask: Option<unsafe extern "C" fn(red: c_uchar, green: c_uchar, blue: c_uchar, alpha: c_uchar)>;
    pub static mut qglColorMaterial: Option<unsafe extern "C" fn(face: c_uint, mode: c_uint)>;
    pub static mut qglColorPointer: Option<unsafe extern "C" fn(size: c_int, type_: c_uint, stride: c_int, pointer: *const c_void)>;
    pub static mut qglCopyPixels: Option<unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int, type_: c_uint)>;
    pub static mut qglCopyTexImage1D: Option<unsafe extern "C" fn(target: c_uint, level: c_int, internalFormat: c_uint, x: c_int, y: c_int, width: c_int, border: c_int)>;
    pub static mut qglCopyTexImage2D: Option<unsafe extern "C" fn(target: c_uint, level: c_int, internalFormat: c_uint, x: c_int, y: c_int, width: c_int, height: c_int, border: c_int)>;
    pub static mut qglCopyTexSubImage1D: Option<unsafe extern "C" fn(target: c_uint, level: c_int, xoffset: c_int, x: c_int, y: c_int, width: c_int)>;
    pub static mut qglCopyTexSubImage2D: Option<unsafe extern "C" fn(target: c_uint, level: c_int, xoffset: c_int, yoffset: c_int, x: c_int, y: c_int, width: c_int, height: c_int)>;
    pub static mut qglCullFace: Option<unsafe extern "C" fn(mode: c_uint)>;
    pub static mut qglDeleteLists: Option<unsafe extern "C" fn(list: c_uint, range: c_int)>;
    pub static mut qglDeleteTextures: Option<unsafe extern "C" fn(n: c_int, textures: *const c_uint)>;
    pub static mut qglDepthFunc: Option<unsafe extern "C" fn(func: c_uint)>;
    pub static mut qglDepthMask: Option<unsafe extern "C" fn(flag: c_uchar)>;
    pub static mut qglDepthRange: Option<unsafe extern "C" fn(zNear: c_double, zFar: c_double)>;
    pub static mut qglDisable: Option<unsafe extern "C" fn(cap: c_uint)>;
    pub static mut qglDisableClientState: Option<unsafe extern "C" fn(array: c_uint)>;
    pub static mut qglDrawArrays: Option<unsafe extern "C" fn(mode: c_uint, first: c_int, count: c_int)>;
    pub static mut qglDrawBuffer: Option<unsafe extern "C" fn(mode: c_uint)>;
    pub static mut qglDrawElements: Option<unsafe extern "C" fn(mode: c_uint, count: c_int, type_: c_uint, indices: *const c_void)>;
    pub static mut qglDrawPixels: Option<unsafe extern "C" fn(width: c_int, height: c_int, format: c_uint, type_: c_uint, pixels: *const c_void)>;
    pub static mut qglEdgeFlag: Option<unsafe extern "C" fn(flag: c_uchar)>;
    pub static mut qglEdgeFlagPointer: Option<unsafe extern "C" fn(stride: c_int, pointer: *const c_void)>;
    pub static mut qglEdgeFlagv: Option<unsafe extern "C" fn(flag: *const c_uchar)>;
    pub static mut qglEnable: Option<unsafe extern "C" fn(cap: c_uint)>;
    pub static mut qglEnableClientState: Option<unsafe extern "C" fn(array: c_uint)>;
    pub static mut qglEnd: Option<unsafe extern "C" fn()>;
    pub static mut qglEndList: Option<unsafe extern "C" fn()>;
    pub static mut qglEvalCoord1d: Option<unsafe extern "C" fn(u: c_double)>;
    pub static mut qglEvalCoord1dv: Option<unsafe extern "C" fn(u: *const c_double)>;
    pub static mut qglEvalCoord1f: Option<unsafe extern "C" fn(u: c_float)>;
    pub static mut qglEvalCoord1fv: Option<unsafe extern "C" fn(u: *const c_float)>;
    pub static mut qglEvalCoord2d: Option<unsafe extern "C" fn(u: c_double, v: c_double)>;
    pub static mut qglEvalCoord2dv: Option<unsafe extern "C" fn(u: *const c_double)>;
    pub static mut qglEvalCoord2f: Option<unsafe extern "C" fn(u: c_float, v: c_float)>;
    pub static mut qglEvalCoord2fv: Option<unsafe extern "C" fn(u: *const c_float)>;
    pub static mut qglEvalMesh1: Option<unsafe extern "C" fn(mode: c_uint, i1: c_int, i2: c_int)>;
    pub static mut qglEvalMesh2: Option<unsafe extern "C" fn(mode: c_uint, i1: c_int, i2: c_int, j1: c_int, j2: c_int)>;
    pub static mut qglEvalPoint1: Option<unsafe extern "C" fn(i: c_int)>;
    pub static mut qglEvalPoint2: Option<unsafe extern "C" fn(i: c_int, j: c_int)>;
    pub static mut qglFeedbackBuffer: Option<unsafe extern "C" fn(size: c_int, type_: c_uint, buffer: *mut c_float)>;
    pub static mut qglFinish: Option<unsafe extern "C" fn()>;
    pub static mut qglFlush: Option<unsafe extern "C" fn()>;
    pub static mut qglFogf: Option<unsafe extern "C" fn(pname: c_uint, param: c_float)>;
    pub static mut qglFogfv: Option<unsafe extern "C" fn(pname: c_uint, params: *const c_float)>;
    pub static mut qglFogi: Option<unsafe extern "C" fn(pname: c_uint, param: c_int)>;
    pub static mut qglFogiv: Option<unsafe extern "C" fn(pname: c_uint, params: *const c_int)>;
    pub static mut qglFrontFace: Option<unsafe extern "C" fn(mode: c_uint)>;
    pub static mut qglFrustum: Option<unsafe extern "C" fn(left: c_double, right: c_double, bottom: c_double, top: c_double, zNear: c_double, zFar: c_double)>;
    pub static mut qglGenLists: Option<unsafe extern "C" fn(range: c_int) -> c_uint>;
    pub static mut qglGenTextures: Option<unsafe extern "C" fn(n: c_int, textures: *mut c_uint)>;
    pub static mut qglGetBooleanv: Option<unsafe extern "C" fn(pname: c_uint, params: *mut c_uchar)>;
    pub static mut qglGetClipPlane: Option<unsafe extern "C" fn(plane: c_uint, equation: *mut c_double)>;
    pub static mut qglGetDoublev: Option<unsafe extern "C" fn(pname: c_uint, params: *mut c_double)>;
    pub static mut qglGetError: Option<unsafe extern "C" fn() -> c_uint>;
    pub static mut qglGetFloatv: Option<unsafe extern "C" fn(pname: c_uint, params: *mut c_float)>;
    pub static mut qglGetIntegerv: Option<unsafe extern "C" fn(pname: c_uint, params: *mut c_int)>;
    pub static mut qglGetLightfv: Option<unsafe extern "C" fn(light: c_uint, pname: c_uint, params: *mut c_float)>;
    pub static mut qglGetLightiv: Option<unsafe extern "C" fn(light: c_uint, pname: c_uint, params: *mut c_int)>;
    pub static mut qglGetMapdv: Option<unsafe extern "C" fn(target: c_uint, query: c_uint, v: *mut c_double)>;
    pub static mut qglGetMapfv: Option<unsafe extern "C" fn(target: c_uint, query: c_uint, v: *mut c_float)>;
    pub static mut qglGetMapiv: Option<unsafe extern "C" fn(target: c_uint, query: c_uint, v: *mut c_int)>;
    pub static mut qglGetMaterialfv: Option<unsafe extern "C" fn(face: c_uint, pname: c_uint, params: *mut c_float)>;
    pub static mut qglGetMaterialiv: Option<unsafe extern "C" fn(face: c_uint, pname: c_uint, params: *mut c_int)>;
    pub static mut qglGetPixelMapfv: Option<unsafe extern "C" fn(map: c_uint, values: *mut c_float)>;
    pub static mut qglGetPixelMapuiv: Option<unsafe extern "C" fn(map: c_uint, values: *mut c_uint)>;
    pub static mut qglGetPixelMapusv: Option<unsafe extern "C" fn(map: c_uint, values: *mut c_ushort)>;
    pub static mut qglGetPointerv: Option<unsafe extern "C" fn(pname: c_uint, params: *mut *mut c_void)>;
    pub static mut qglGetPolygonStipple: Option<unsafe extern "C" fn(mask: *mut c_uchar)>;
    pub static mut qglGetString: Option<unsafe extern "C" fn(name: c_uint) -> *const c_uchar>;
    pub static mut qglGetTexEnvfv: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *mut c_float)>;
    pub static mut qglGetTexEnviv: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *mut c_int)>;
    pub static mut qglGetTexGendv: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, params: *mut c_double)>;
    pub static mut qglGetTexGenfv: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, params: *mut c_float)>;
    pub static mut qglGetTexGeniv: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, params: *mut c_int)>;
    pub static mut qglGetTexImage: Option<unsafe extern "C" fn(target: c_uint, level: c_int, format: c_uint, type_: c_uint, pixels: *mut c_void)>;
    pub static mut qglGetTexLevelParameterfv: Option<unsafe extern "C" fn(target: c_uint, level: c_int, pname: c_uint, params: *mut c_float)>;
    pub static mut qglGetTexLevelParameteriv: Option<unsafe extern "C" fn(target: c_uint, level: c_int, pname: c_uint, params: *mut c_int)>;
    pub static mut qglGetTexParameterfv: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *mut c_float)>;
    pub static mut qglGetTexParameteriv: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *mut c_int)>;
    pub static mut qglHint: Option<unsafe extern "C" fn(target: c_uint, mode: c_uint)>;
    pub static mut qglIndexMask: Option<unsafe extern "C" fn(mask: c_uint)>;
    pub static mut qglIndexPointer: Option<unsafe extern "C" fn(type_: c_uint, stride: c_int, pointer: *const c_void)>;
    pub static mut qglIndexd: Option<unsafe extern "C" fn(c: c_double)>;
    pub static mut qglIndexdv: Option<unsafe extern "C" fn(c: *const c_double)>;
    pub static mut qglIndexf: Option<unsafe extern "C" fn(c: c_float)>;
    pub static mut qglIndexfv: Option<unsafe extern "C" fn(c: *const c_float)>;
    pub static mut qglIndexi: Option<unsafe extern "C" fn(c: c_int)>;
    pub static mut qglIndexiv: Option<unsafe extern "C" fn(c: *const c_int)>;
    pub static mut qglIndexs: Option<unsafe extern "C" fn(c: c_short)>;
    pub static mut qglIndexsv: Option<unsafe extern "C" fn(c: *const c_short)>;
    pub static mut qglIndexub: Option<unsafe extern "C" fn(c: c_uchar)>;
    pub static mut qglIndexubv: Option<unsafe extern "C" fn(c: *const c_uchar)>;
    pub static mut qglInitNames: Option<unsafe extern "C" fn()>;
    pub static mut qglInterleavedArrays: Option<unsafe extern "C" fn(format: c_uint, stride: c_int, pointer: *const c_void)>;
    pub static mut qglIsEnabled: Option<unsafe extern "C" fn(cap: c_uint) -> c_uchar>;
    pub static mut qglIsList: Option<unsafe extern "C" fn(ilist: c_uint) -> c_uchar>;
    pub static mut qglIsTexture: Option<unsafe extern "C" fn(texture: c_uint) -> c_uchar>;
    pub static mut qglLightModelf: Option<unsafe extern "C" fn(pname: c_uint, param: c_float)>;
    pub static mut qglLightModelfv: Option<unsafe extern "C" fn(pname: c_uint, params: *const c_float)>;
    pub static mut qglLightModeli: Option<unsafe extern "C" fn(pname: c_uint, param: c_int)>;
    pub static mut qglLightModeliv: Option<unsafe extern "C" fn(pname: c_uint, params: *const c_int)>;
    pub static mut qglLightf: Option<unsafe extern "C" fn(light: c_uint, pname: c_uint, param: c_float)>;
    pub static mut qglLightfv: Option<unsafe extern "C" fn(light: c_uint, pname: c_uint, params: *const c_float)>;
    pub static mut qglLighti: Option<unsafe extern "C" fn(light: c_uint, pname: c_uint, param: c_int)>;
    pub static mut qglLightiv: Option<unsafe extern "C" fn(light: c_uint, pname: c_uint, params: *const c_int)>;
    pub static mut qglLineStipple: Option<unsafe extern "C" fn(factor: c_int, pattern: c_ushort)>;
    pub static mut qglLineWidth: Option<unsafe extern "C" fn(width: c_float)>;
    pub static mut qglListBase: Option<unsafe extern "C" fn(base: c_uint)>;
    pub static mut qglLoadIdentity: Option<unsafe extern "C" fn()>;
    pub static mut qglLoadMatrixd: Option<unsafe extern "C" fn(m: *const c_double)>;
    pub static mut qglLoadMatrixf: Option<unsafe extern "C" fn(m: *const c_float)>;
    pub static mut qglLoadName: Option<unsafe extern "C" fn(name: c_uint)>;
    pub static mut qglLogicOp: Option<unsafe extern "C" fn(opcode: c_uint)>;
    pub static mut qglMap1d: Option<unsafe extern "C" fn(target: c_uint, u1: c_double, u2: c_double, stride: c_int, order: c_int, points: *const c_double)>;
    pub static mut qglMap1f: Option<unsafe extern "C" fn(target: c_uint, u1: c_float, u2: c_float, stride: c_int, order: c_int, points: *const c_float)>;
    pub static mut qglMap2d: Option<unsafe extern "C" fn(target: c_uint, u1: c_double, u2: c_double, ustride: c_int, uorder: c_int, v1: c_double, v2: c_double, vstride: c_int, vorder: c_int, points: *const c_double)>;
    pub static mut qglMap2f: Option<unsafe extern "C" fn(target: c_uint, u1: c_float, u2: c_float, ustride: c_int, uorder: c_int, v1: c_float, v2: c_float, vstride: c_int, vorder: c_int, points: *const c_float)>;
    pub static mut qglMapGrid1d: Option<unsafe extern "C" fn(un: c_int, u1: c_double, u2: c_double)>;
    pub static mut qglMapGrid1f: Option<unsafe extern "C" fn(un: c_int, u1: c_float, u2: c_float)>;
    pub static mut qglMapGrid2d: Option<unsafe extern "C" fn(un: c_int, u1: c_double, u2: c_double, vn: c_int, v1: c_double, v2: c_double)>;
    pub static mut qglMapGrid2f: Option<unsafe extern "C" fn(un: c_int, u1: c_float, u2: c_float, vn: c_int, v1: c_float, v2: c_float)>;
    pub static mut qglMaterialf: Option<unsafe extern "C" fn(face: c_uint, pname: c_uint, param: c_float)>;
    pub static mut qglMaterialfv: Option<unsafe extern "C" fn(face: c_uint, pname: c_uint, params: *const c_float)>;
    pub static mut qglMateriali: Option<unsafe extern "C" fn(face: c_uint, pname: c_uint, param: c_int)>;
    pub static mut qglMaterialiv: Option<unsafe extern "C" fn(face: c_uint, pname: c_uint, params: *const c_int)>;
    pub static mut qglMatrixMode: Option<unsafe extern "C" fn(mode: c_uint)>;
    pub static mut qglMultMatrixd: Option<unsafe extern "C" fn(m: *const c_double)>;
    pub static mut qglMultMatrixf: Option<unsafe extern "C" fn(m: *const c_float)>;
    pub static mut qglNewList: Option<unsafe extern "C" fn(list: c_uint, mode: c_uint)>;
    pub static mut qglNormal3b: Option<unsafe extern "C" fn(nx: c_schar, ny: c_schar, nz: c_schar)>;
    pub static mut qglNormal3bv: Option<unsafe extern "C" fn(v: *const c_schar)>;
    pub static mut qglNormal3d: Option<unsafe extern "C" fn(nx: c_double, ny: c_double, nz: c_double)>;
    pub static mut qglNormal3dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglNormal3f: Option<unsafe extern "C" fn(nx: c_float, ny: c_float, nz: c_float)>;
    pub static mut qglNormal3fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglNormal3i: Option<unsafe extern "C" fn(nx: c_int, ny: c_int, nz: c_int)>;
    pub static mut qglNormal3iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglNormal3s: Option<unsafe extern "C" fn(nx: c_short, ny: c_short, nz: c_short)>;
    pub static mut qglNormal3sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglNormalPointer: Option<unsafe extern "C" fn(type_: c_uint, stride: c_int, pointer: *const c_void)>;
    pub static mut qglOrtho: Option<unsafe extern "C" fn(left: c_double, right: c_double, bottom: c_double, top: c_double, zNear: c_double, zFar: c_double)>;
    pub static mut qglPassThrough: Option<unsafe extern "C" fn(token: c_float)>;
    pub static mut qglPixelMapfv: Option<unsafe extern "C" fn(map: c_uint, mapsize: c_int, values: *const c_float)>;
    pub static mut qglPixelMapuiv: Option<unsafe extern "C" fn(map: c_uint, mapsize: c_int, values: *const c_uint)>;
    pub static mut qglPixelMapusv: Option<unsafe extern "C" fn(map: c_uint, mapsize: c_int, values: *const c_ushort)>;
    pub static mut qglPixelStoref: Option<unsafe extern "C" fn(pname: c_uint, param: c_float)>;
    pub static mut qglPixelStorei: Option<unsafe extern "C" fn(pname: c_uint, param: c_int)>;
    pub static mut qglPixelTransferf: Option<unsafe extern "C" fn(pname: c_uint, param: c_float)>;
    pub static mut qglPixelTransferi: Option<unsafe extern "C" fn(pname: c_uint, param: c_int)>;
    pub static mut qglPixelZoom: Option<unsafe extern "C" fn(xfactor: c_float, yfactor: c_float)>;
    pub static mut qglPointSize: Option<unsafe extern "C" fn(size: c_float)>;
    pub static mut qglPolygonMode: Option<unsafe extern "C" fn(face: c_uint, mode: c_uint)>;
    pub static mut qglPolygonOffset: Option<unsafe extern "C" fn(factor: c_float, units: c_float)>;
    pub static mut qglPolygonStipple: Option<unsafe extern "C" fn(mask: *const c_uchar)>;
    pub static mut qglPopAttrib: Option<unsafe extern "C" fn()>;
    pub static mut qglPopClientAttrib: Option<unsafe extern "C" fn()>;
    pub static mut qglPopMatrix: Option<unsafe extern "C" fn()>;
    pub static mut qglPopName: Option<unsafe extern "C" fn()>;
    pub static mut qglPrioritizeTextures: Option<unsafe extern "C" fn(n: c_int, textures: *const c_uint, priorities: *const c_float)>;
    pub static mut qglPushAttrib: Option<unsafe extern "C" fn(mask: c_uint)>;
    pub static mut qglPushClientAttrib: Option<unsafe extern "C" fn(mask: c_uint)>;
    pub static mut qglPushMatrix: Option<unsafe extern "C" fn()>;
    pub static mut qglPushName: Option<unsafe extern "C" fn(name: c_uint)>;
    pub static mut qglRasterPos2d: Option<unsafe extern "C" fn(x: c_double, y: c_double)>;
    pub static mut qglRasterPos2dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglRasterPos2f: Option<unsafe extern "C" fn(x: c_float, y: c_float)>;
    pub static mut qglRasterPos2fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglRasterPos2i: Option<unsafe extern "C" fn(x: c_int, y: c_int)>;
    pub static mut qglRasterPos2iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglRasterPos2s: Option<unsafe extern "C" fn(x: c_short, y: c_short)>;
    pub static mut qglRasterPos2sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglRasterPos3d: Option<unsafe extern "C" fn(x: c_double, y: c_double, z: c_double)>;
    pub static mut qglRasterPos3dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglRasterPos3f: Option<unsafe extern "C" fn(x: c_float, y: c_float, z: c_float)>;
    pub static mut qglRasterPos3fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglRasterPos3i: Option<unsafe extern "C" fn(x: c_int, y: c_int, z: c_int)>;
    pub static mut qglRasterPos3iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglRasterPos3s: Option<unsafe extern "C" fn(x: c_short, y: c_short, z: c_short)>;
    pub static mut qglRasterPos3sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglRasterPos4d: Option<unsafe extern "C" fn(x: c_double, y: c_double, z: c_double, w: c_double)>;
    pub static mut qglRasterPos4dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglRasterPos4f: Option<unsafe extern "C" fn(x: c_float, y: c_float, z: c_float, w: c_float)>;
    pub static mut qglRasterPos4fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglRasterPos4i: Option<unsafe extern "C" fn(x: c_int, y: c_int, z: c_int, w: c_int)>;
    pub static mut qglRasterPos4iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglRasterPos4s: Option<unsafe extern "C" fn(x: c_short, y: c_short, z: c_short, w: c_short)>;
    pub static mut qglRasterPos4sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglReadBuffer: Option<unsafe extern "C" fn(mode: c_uint)>;
    pub static mut qglReadPixels: Option<unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int, format: c_uint, type_: c_uint, pixels: *mut c_void)>;
    pub static mut qglRectd: Option<unsafe extern "C" fn(x1: c_double, y1: c_double, x2: c_double, y2: c_double)>;
    pub static mut qglRectdv: Option<unsafe extern "C" fn(v1: *const c_double, v2: *const c_double)>;
    pub static mut qglRectf: Option<unsafe extern "C" fn(x1: c_float, y1: c_float, x2: c_float, y2: c_float)>;
    pub static mut qglRectfv: Option<unsafe extern "C" fn(v1: *const c_float, v2: *const c_float)>;
    pub static mut qglRecti: Option<unsafe extern "C" fn(x1: c_int, y1: c_int, x2: c_int, y2: c_int)>;
    pub static mut qglRectiv: Option<unsafe extern "C" fn(v1: *const c_int, v2: *const c_int)>;
    pub static mut qglRects: Option<unsafe extern "C" fn(x1: c_short, y1: c_short, x2: c_short, y2: c_short)>;
    pub static mut qglRectsv: Option<unsafe extern "C" fn(v1: *const c_short, v2: *const c_short)>;
    pub static mut qglRenderMode: Option<unsafe extern "C" fn(mode: c_uint) -> c_int>;
    pub static mut qglRotated: Option<unsafe extern "C" fn(angle: c_double, x: c_double, y: c_double, z: c_double)>;
    pub static mut qglRotatef: Option<unsafe extern "C" fn(angle: c_float, x: c_float, y: c_float, z: c_float)>;
    pub static mut qglScaled: Option<unsafe extern "C" fn(x: c_double, y: c_double, z: c_double)>;
    pub static mut qglScalef: Option<unsafe extern "C" fn(x: c_float, y: c_float, z: c_float)>;
    pub static mut qglScissor: Option<unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int)>;
    pub static mut qglSelectBuffer: Option<unsafe extern "C" fn(size: c_int, buffer: *mut c_uint)>;
    pub static mut qglShadeModel: Option<unsafe extern "C" fn(mode: c_uint)>;
    pub static mut qglStencilFunc: Option<unsafe extern "C" fn(func: c_uint, ref_: c_int, mask: c_uint)>;
    pub static mut qglStencilMask: Option<unsafe extern "C" fn(mask: c_uint)>;
    pub static mut qglStencilOp: Option<unsafe extern "C" fn(fail: c_uint, zfail: c_uint, zpass: c_uint)>;
    pub static mut qglTexCoord1d: Option<unsafe extern "C" fn(s: c_double)>;
    pub static mut qglTexCoord1dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglTexCoord1f: Option<unsafe extern "C" fn(s: c_float)>;
    pub static mut qglTexCoord1fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglTexCoord1i: Option<unsafe extern "C" fn(s: c_int)>;
    pub static mut qglTexCoord1iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglTexCoord1s: Option<unsafe extern "C" fn(s: c_short)>;
    pub static mut qglTexCoord1sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglTexCoord2d: Option<unsafe extern "C" fn(s: c_double, t: c_double)>;
    pub static mut qglTexCoord2dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglTexCoord2f: Option<unsafe extern "C" fn(s: c_float, t: c_float)>;
    pub static mut qglTexCoord2fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglTexCoord2i: Option<unsafe extern "C" fn(s: c_int, t: c_int)>;
    pub static mut qglTexCoord2iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglTexCoord2s: Option<unsafe extern "C" fn(s: c_short, t: c_short)>;
    pub static mut qglTexCoord2sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglTexCoord3d: Option<unsafe extern "C" fn(s: c_double, t: c_double, r: c_double)>;
    pub static mut qglTexCoord3dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglTexCoord3f: Option<unsafe extern "C" fn(s: c_float, t: c_float, r: c_float)>;
    pub static mut qglTexCoord3fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglTexCoord3i: Option<unsafe extern "C" fn(s: c_int, t: c_int, r: c_int)>;
    pub static mut qglTexCoord3iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglTexCoord3s: Option<unsafe extern "C" fn(s: c_short, t: c_short, r: c_short)>;
    pub static mut qglTexCoord3sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglTexCoord4d: Option<unsafe extern "C" fn(s: c_double, t: c_double, r: c_double, q: c_double)>;
    pub static mut qglTexCoord4dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglTexCoord4f: Option<unsafe extern "C" fn(s: c_float, t: c_float, r: c_float, q: c_float)>;
    pub static mut qglTexCoord4fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglTexCoord4i: Option<unsafe extern "C" fn(s: c_int, t: c_int, r: c_int, q: c_int)>;
    pub static mut qglTexCoord4iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglTexCoord4s: Option<unsafe extern "C" fn(s: c_short, t: c_short, r: c_short, q: c_short)>;
    pub static mut qglTexCoord4sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglTexCoordPointer: Option<unsafe extern "C" fn(size: c_int, type_: c_uint, stride: c_int, pointer: *const c_void)>;
    pub static mut qglTexEnvf: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, param: c_float)>;
    pub static mut qglTexEnvfv: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *const c_float)>;
    pub static mut qglTexEnvi: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, param: c_int)>;
    pub static mut qglTexEnviv: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *const c_int)>;
    pub static mut qglTexGend: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, param: c_double)>;
    pub static mut qglTexGendv: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, params: *const c_double)>;
    pub static mut qglTexGenf: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, param: c_float)>;
    pub static mut qglTexGenfv: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, params: *const c_float)>;
    pub static mut qglTexGeni: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, param: c_int)>;
    pub static mut qglTexGeniv: Option<unsafe extern "C" fn(coord: c_uint, pname: c_uint, params: *const c_int)>;
    pub static mut qglTexImage1D: Option<unsafe extern "C" fn(target: c_uint, level: c_int, internalformat: c_int, width: c_int, border: c_int, format: c_uint, type_: c_uint, pixels: *const c_void)>;
    pub static mut qglTexImage2D: Option<unsafe extern "C" fn(target: c_uint, level: c_int, internalformat: c_int, width: c_int, height: c_int, border: c_int, format: c_uint, type_: c_uint, pixels: *const c_void)>;
    pub static mut qglTexParameterf: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, param: c_float)>;
    pub static mut qglTexParameterfv: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *const c_float)>;
    pub static mut qglTexParameteri: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, param: c_int)>;
    pub static mut qglTexParameteriv: Option<unsafe extern "C" fn(target: c_uint, pname: c_uint, params: *const c_int)>;
    pub static mut qglTexSubImage1D: Option<unsafe extern "C" fn(target: c_uint, level: c_int, xoffset: c_int, width: c_int, format: c_uint, type_: c_uint, pixels: *const c_void)>;
    pub static mut qglTexSubImage2D: Option<unsafe extern "C" fn(target: c_uint, level: c_int, xoffset: c_int, yoffset: c_int, width: c_int, height: c_int, format: c_uint, type_: c_uint, pixels: *const c_void)>;
    pub static mut qglTranslated: Option<unsafe extern "C" fn(x: c_double, y: c_double, z: c_double)>;
    pub static mut qglTranslatef: Option<unsafe extern "C" fn(x: c_float, y: c_float, z: c_float)>;
    pub static mut qglVertex2d: Option<unsafe extern "C" fn(x: c_double, y: c_double)>;
    pub static mut qglVertex2dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglVertex2f: Option<unsafe extern "C" fn(x: c_float, y: c_float)>;
    pub static mut qglVertex2fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglVertex2i: Option<unsafe extern "C" fn(x: c_int, y: c_int)>;
    pub static mut qglVertex2iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglVertex2s: Option<unsafe extern "C" fn(x: c_short, y: c_short)>;
    pub static mut qglVertex2sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglVertex3d: Option<unsafe extern "C" fn(x: c_double, y: c_double, z: c_double)>;
    pub static mut qglVertex3dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglVertex3f: Option<unsafe extern "C" fn(x: c_float, y: c_float, z: c_float)>;
    pub static mut qglVertex3fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglVertex3i: Option<unsafe extern "C" fn(x: c_int, y: c_int, z: c_int)>;
    pub static mut qglVertex3iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglVertex3s: Option<unsafe extern "C" fn(x: c_short, y: c_short, z: c_short)>;
    pub static mut qglVertex3sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglVertex4d: Option<unsafe extern "C" fn(x: c_double, y: c_double, z: c_double, w: c_double)>;
    pub static mut qglVertex4dv: Option<unsafe extern "C" fn(v: *const c_double)>;
    pub static mut qglVertex4f: Option<unsafe extern "C" fn(x: c_float, y: c_float, z: c_float, w: c_float)>;
    pub static mut qglVertex4fv: Option<unsafe extern "C" fn(v: *const c_float)>;
    pub static mut qglVertex4i: Option<unsafe extern "C" fn(x: c_int, y: c_int, z: c_int, w: c_int)>;
    pub static mut qglVertex4iv: Option<unsafe extern "C" fn(v: *const c_int)>;
    pub static mut qglVertex4s: Option<unsafe extern "C" fn(x: c_short, y: c_short, z: c_short, w: c_short)>;
    pub static mut qglVertex4sv: Option<unsafe extern "C" fn(v: *const c_short)>;
    pub static mut qglVertexPointer: Option<unsafe extern "C" fn(size: c_int, type_: c_uint, stride: c_int, pointer: *const c_void)>;
    pub static mut qglViewport: Option<unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int)>;
}

#[cfg(target_os = "windows")]
extern "C" {
    // Windows WGL functions
    pub static mut qwglCopyContext: Option<unsafe extern "C" fn(hglrc0: *mut c_void, hglrc1: *mut c_void, mask: c_uint) -> c_int>;
    pub static mut qwglCreateContext: Option<unsafe extern "C" fn(hdc: HDC) -> *mut c_void>;
    pub static mut qwglCreateLayerContext: Option<unsafe extern "C" fn(hdc: HDC, layer: c_int) -> *mut c_void>;
    pub static mut qwglDeleteContext: Option<unsafe extern "C" fn(hglrc: *mut c_void) -> c_int>;
    pub static mut qwglGetCurrentContext: Option<unsafe extern "C" fn() -> *mut c_void>;
    pub static mut qwglGetCurrentDC: Option<unsafe extern "C" fn() -> HDC>;
    pub static mut qwglGetProcAddress: Option<unsafe extern "C" fn(lpszProc: *const c_char) -> unsafe extern "C" fn()>;
    pub static mut qwglMakeCurrent: Option<unsafe extern "C" fn(hdc: HDC, hglrc: *mut c_void) -> c_int>;
    pub static mut qwglShareLists: Option<unsafe extern "C" fn(hglrc0: *mut c_void, hglrc1: *mut c_void) -> c_int>;
    pub static mut qwglUseFontBitmaps: Option<unsafe extern "C" fn(hdc: HDC, first: c_uint, count: c_uint, listBase: c_uint) -> c_int>;

    pub static mut qwglUseFontOutlines: Option<unsafe extern "C" fn(hdc: HDC, first: c_uint, count: c_uint, listBase: c_uint, deviation: c_float, extrusion: c_float, format: c_int, lpgmf: *mut c_void) -> c_int>;

    pub static mut qwglDescribeLayerPlane: Option<unsafe extern "C" fn(hdc: HDC, pixelFormat: c_int, layerPlane: c_int, nBytes: c_uint, plpd: *mut c_void) -> c_int>;
    pub static mut qwglSetLayerPaletteEntries: Option<unsafe extern "C" fn(hdc: HDC, iLayerPlane: c_int, iStart: c_int, cEntries: c_int, pcr: *const c_uint) -> c_int>;
    pub static mut qwglGetLayerPaletteEntries: Option<unsafe extern "C" fn(hdc: HDC, iLayerPlane: c_int, iStart: c_int, cEntries: c_int, pcr: *mut c_uint) -> c_int>;
    pub static mut qwglRealizeLayerPalette: Option<unsafe extern "C" fn(hdc: HDC, iLayerPlane: c_int, bRealize: c_int) -> c_int>;
    pub static mut qwglSwapLayerBuffers: Option<unsafe extern "C" fn(hdc: HDC, fuFlags: c_uint) -> c_int>;

    pub static mut qwglSwapIntervalEXT: Option<unsafe extern "C" fn(interval: c_int) -> c_int>;
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
extern "C" {
    // FX Mesa Functions - bk001129 - from cvs1.17 (mkv)
    // These are only declared if __FX__ is defined in the original, but we include them for completeness.
    // In actual Rust usage, these would be conditionally compiled.

    // GLX Functions
    pub static mut qglXChooseVisual: Option<unsafe extern "C" fn(dpy: *mut c_void, screen: c_int, attribList: *mut c_int) -> *mut c_void>;
    pub static mut qglXCreateContext: Option<unsafe extern "C" fn(dpy: *mut c_void, vis: *mut c_void, shareList: *mut c_void, direct: c_int) -> *mut c_void>;
    pub static mut qglXDestroyContext: Option<unsafe extern "C" fn(dpy: *mut c_void, ctx: *mut c_void)>;
    pub static mut qglXMakeCurrent: Option<unsafe extern "C" fn(dpy: *mut c_void, drawable: *mut c_void, ctx: *mut c_void) -> c_int>;
    pub static mut qglXCopyContext: Option<unsafe extern "C" fn(dpy: *mut c_void, src: *mut c_void, dst: *mut c_void, mask: c_uint)>;
    pub static mut qglXSwapBuffers: Option<unsafe extern "C" fn(dpy: *mut c_void, drawable: *mut c_void)>;
}
