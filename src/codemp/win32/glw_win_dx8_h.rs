/*
 * UNPUBLISHED -- Rights  reserved  under  the  copyright  laws  of the
 * United States.  Use  of a copyright notice is precautionary only and
 * does not imply publication or disclosure.
 *
 * THIS DOCUMENTATION CONTAINS CONFIDENTIAL AND PROPRIETARY INFORMATION
 * OF    VICARIOUS   VISIONS,  INC.    ANY  DUPLICATION,  MODIFICATION,
 * DISTRIBUTION, OR DISCLOSURE IS STRICTLY PROHIBITED WITHOUT THE PRIOR
 * EXPRESS WRITTEN PERMISSION OF VICARIOUS VISIONS, INC.
 */

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Opaque DirectX COM interface types (from d3d8.h / d3dx8.h)
#[repr(C)]
pub struct IDirect3DDevice8 {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct IDirect3DTexture8 {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct ID3DXMatrixStack {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct LightEffects {
    _opaque: [u8; 0],
}

// Type aliases for DirectX primitive types
pub type D3DPRIMITIVETYPE = c_int;
pub type D3DTEXTUREOP = c_int;
pub type D3DTEXTUREFILTERTYPE = c_int;
pub type D3DTEXTUREADDRESS = c_int;
pub type D3DCULL = c_int;
pub type D3DCOLOR = u32;
pub type DWORD = u32;
pub type CHAR = c_char;
pub type GLuint = u32;
pub type GLushort = u16;
pub type GLsizei = c_int;

// Opaque DirectX struct types (from d3d8.h)
#[repr(C)]
pub struct D3DLIGHT8 {
    // Placeholder for opaque DirectX light structure
    _opaque: [u8; 80],
}

#[repr(C)]
pub struct D3DMATERIAL8 {
    // Placeholder for opaque DirectX material structure
    _opaque: [u8; 64],
}

#[repr(C)]
pub struct D3DVIEWPORT8 {
    // Placeholder for opaque DirectX viewport structure
    _opaque: [u8; 28],
}

#[repr(C)]
pub struct D3DRECT {
    // Placeholder for opaque DirectX rectangle structure
    _opaque: [u8; 16],
}

// Stub for C++ std::map<GLuint, TextureInfo>
// Original: typedef std::map<GLuint, TextureInfo> texturexlat_t;
// This is an opaque representation; C++ map internals are implementation-dependent
#[repr(C)]
pub struct std_map_GLuint_TextureInfo {
    _opaque: [u64; 3],
}

// Matrix mode enum
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixMode {
    MatrixMode_Model = 0,
    MatrixMode_Projection = 1,
    MatrixMode_Texture0 = 2,
    MatrixMode_Texture1 = 3,
    MatrixMode_Texture2 = 4,
    MatrixMode_Texture3 = 5,

    Num_MatrixModes = 6,
}

// Texture info struct
#[repr(C)]
pub struct TextureInfo {
    pub mipmap: *mut IDirect3DTexture8,
    pub minFilter: D3DTEXTUREFILTERTYPE,
    pub mipFilter: D3DTEXTUREFILTERTYPE,
    pub magFilter: D3DTEXTUREFILTERTYPE,
    pub wrapU: D3DTEXTUREADDRESS,
    pub wrapV: D3DTEXTUREADDRESS,
    pub anisotropy: f32,
}

const GLW_MAX_TEXTURE_STAGES: usize = 2;
const GLW_MAX_STRIPS: usize = 2048;

// SHADER_MAX_INDEXES is defined in qfiles.h; imported here for array sizing
// TODO: verify actual value from qfiles.h port
const SHADER_MAX_INDEXES: usize = 4096;

#[repr(C)]
pub struct glwstate_t {
    // Interface to DX
    pub device: *mut IDirect3DDevice8,

    // Matrix stuff
    pub matrixStack: [*mut ID3DXMatrixStack; 6],
    pub matrixMode: MatrixMode,

    // Current primitive mode (triangles/quads/strips)
    pub primitiveMode: D3DPRIMITIVETYPE,

    // Are we in a glBegin/glEnd block? (Used for sanity checks.)
    pub inDrawBlock: bool,

    // Texturing
    pub textureStageDirty: [bool; GLW_MAX_TEXTURE_STAGES],
    pub textureStageEnable: [bool; GLW_MAX_TEXTURE_STAGES],
    pub currentTexture: [GLuint; GLW_MAX_TEXTURE_STAGES],
    pub textureEnv: [D3DTEXTUREOP; GLW_MAX_TEXTURE_STAGES],

    pub textureXlat: std_map_GLuint_TextureInfo,

    pub textureBindNum: GLuint,

    pub serverTU: GLuint,
    pub clientTU: GLuint,

    // Pointers to various draw buffers
    pub vertexPointer: *const c_void,
    pub normalPointer: *const c_void,
    pub texCoordPointer: [*const c_void; GLW_MAX_TEXTURE_STAGES],
    pub colorPointer: *const c_void,

    #[cfg(target_os = "windows")]
    // Temporary storage used when rendering quads
    pub vertexPointerBack: *const c_void,
    #[cfg(target_os = "windows")]
    pub normalPointerBack: *const c_void,
    #[cfg(target_os = "windows")]
    pub texCoordPointerBack: [*const c_void; GLW_MAX_TEXTURE_STAGES],
    #[cfg(target_os = "windows")]
    pub colorPointerBack: *const c_void,

    // State of draw buffers
    pub colorArrayState: bool,
    pub texCoordArrayState: [bool; GLW_MAX_TEXTURE_STAGES],
    pub vertexArrayState: bool,
    pub normalArrayState: bool,

    // Stride of various draw buffers
    pub vertexStride: c_int,
    pub texCoordStride: [c_int; GLW_MAX_TEXTURE_STAGES],
    pub colorStride: c_int,
    pub normalStride: c_int,

    // Current number of verts in this packet
    pub numVertices: c_int,

    // Max verts allowed in this packet
    pub maxVertices: c_int,

    // Total verts to draw (may take multiple packets)
    pub totalVertices: c_int,

    // Current number of indices in this packet
    pub numIndices: c_int,

    // Max indices allowed in this packet
    pub maxIndices: c_int,

    // Total indices to draw
    pub totalIndices: c_int,

    // Culling
    pub cullEnable: bool,
    pub cullMode: D3DCULL,

    // Viewport
    pub viewport: D3DVIEWPORT8,

    // Clearing info
    pub clearColor: D3DCOLOR,
    pub clearDepth: f32,
    pub clearStencil: c_int,

    // Widescreen mode
    pub isWidescreen: bool,

    // Global color
    pub currentColor: D3DCOLOR,

    // Scissoring
    pub scissorEnable: bool,
    pub scissorBox: D3DRECT,

    // Directional Light
    pub dirLight: D3DLIGHT8,
    pub mtrl: D3DMATERIAL8,

    // Description of current shader
    pub shaderMask: DWORD,

    // Should we reset matrices on next draw?
    pub matricesDirty: [bool; 6],

    // Render commands go here
    pub drawArray: *mut DWORD,
    pub drawStride: DWORD,

    // This is designed to be an optimization for triangle strips
    // as well as making life easier for the flare effect
    pub strip_dest: [GLushort; SHADER_MAX_INDEXES],
    pub strip_lengths: [GLuint; GLW_MAX_STRIPS],
    pub num_strip_lengths: GLsizei,

    #[cfg(target_os = "xbox")]
    pub lightEffects: *mut LightEffects,
}

extern "C" {
    pub static mut glw_state: *mut glwstate_t;

    pub fn renderObject_HACK();
    pub fn renderObject_Light();
    pub fn renderObject_Env();
    pub fn renderObject_Bump();
    pub fn CreateVertexShader(
        strFilename: *const CHAR,
        pdwVertexDecl: *const DWORD,
        pdwVertexShader: *mut DWORD,
    ) -> bool;
    pub fn CreatePixelShader(strFilename: *const CHAR, pdwPixelShader: *mut DWORD) -> bool;
}
