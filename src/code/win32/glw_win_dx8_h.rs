#![allow(non_snake_case)]

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

use core::ffi::{c_char, c_int, c_void};

// <d3d8.h>/<d3dx8.h> system types — trust-import (module is a future task; unresolved is benign)
use crate::code::win32::d3d8_h::*;
// GLuint, GLushort, GLsizei
use crate::code::renderer::qgl_console_h::*;
// DWORD + win types
use crate::code::game::q_shared::*;
// SHADER_MAX_INDEXES / SHADER_MAX_VERTEXES
use crate::code::qcommon::qfiles_h::*;

pub const GLW_MAX_TEXTURE_STAGES: usize = 2;
pub const GLW_MAX_STRIPS: usize = 2048;

// Nested enum MatrixMode (matches C++ enum inside glwstate_t)
#[derive(Clone, Copy, Debug)]
pub enum MatrixMode {
    MatrixMode_Model = 0,
    MatrixMode_Projection = 1,
    MatrixMode_Texture0 = 2,
    MatrixMode_Texture1 = 3,
    MatrixMode_Texture2 = 4,
    MatrixMode_Texture3 = 5,
}

pub const NUM_MATRIX_MODES: usize = 6;

#[repr(C)]
pub struct glwstate_t {
    // Interface to DX
    pub device: *mut IDirect3DDevice8,

    // Matrix stuff
    pub matrixMode: MatrixMode,
    pub matrixStack: [*mut ID3DXMatrixStack; NUM_MATRIX_MODES],

    // Current primitive mode (triangles/quads/strips)
    pub primitiveMode: D3DPRIMITIVETYPE,

    // Are we in a glBegin/glEnd block? (Used for sanity checks.)
    pub inDrawBlock: bool,

    // Texturing
    pub textureStageDirty: [bool; GLW_MAX_TEXTURE_STAGES],
    pub textureStageEnable: [bool; GLW_MAX_TEXTURE_STAGES],
    pub currentTexture: [GLuint; GLW_MAX_TEXTURE_STAGES],
    pub textureEnv: [D3DTEXTUREOP; GLW_MAX_TEXTURE_STAGES],

    // C++ std::map<GLuint, TextureInfo> - represented as opaque pointer
    // The original uses std::map which requires C++ runtime support
    pub textureXlat: *mut c_void,

    pub textureBindNum: GLuint,

    pub serverTU: GLuint,
    pub clientTU: GLuint,

    // Pointers to various draw buffers
    pub vertexPointer: *const c_void,
    pub normalPointer: *const c_void,
    pub texCoordPointer: [*const c_void; GLW_MAX_TEXTURE_STAGES],
    pub colorPointer: *const c_void,

    // Temporary storage used when rendering quads (_WINDOWS)
    pub vertexPointerBack: *const c_void,
    pub normalPointerBack: *const c_void,
    pub texCoordPointerBack: [*const c_void; GLW_MAX_TEXTURE_STAGES],
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
    pub matricesDirty: [bool; NUM_MATRIX_MODES],

    // Render commands go here
    pub drawArray: *mut DWORD,
    pub drawStride: DWORD,

    // This is designed to be an optimization for triangle strips
    // as well as making life easier for the flare effect
    pub strip_dest: [GLushort; SHADER_MAX_INDEXES],
    pub strip_lengths: [GLuint; GLW_MAX_STRIPS],
    pub num_strip_lengths: GLsizei,
}

// Forward declaration for TextureInfo struct (used in std::map in C++)
// The C++ code uses std::map<GLuint, TextureInfo>, which requires C++ runtime support.
// This is kept as an opaque type since it's internal to the glwstate_t.
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

extern "C" {
    pub static mut glw_state: *mut glwstate_t;

    pub fn renderObject_HACK();
    pub fn renderObject_Light();
    pub fn renderObject_Env();
    pub fn renderObject_Bump();
    pub fn CreateVertexShader(strFilename: *const c_char, pdwVertexDecl: *const DWORD, pdwVertexShader: *mut DWORD) -> bool;
    pub fn CreatePixelShader(strFilename: *const c_char, pdwPixelShader: *mut DWORD) -> bool;
}
