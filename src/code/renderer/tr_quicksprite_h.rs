// this include must remain at the top of every CPP file
// #include "../game/q_math.h"
// #include "tr_local.h"

// tr_QuickSprite.h: interface for the CQuickSprite class.
//
//////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_ulong};

// Forward declarations for external types from tr_local.h and q_shared.h
// These are opaque stubs until their full definitions are ported.

#[repr(C)]
pub struct textureBundle_t {
    // Defined in tr_local.h - opaque stub
    _private: [u8; 0],
}

// qboolean - typically c_int in Quake 3 engine
pub type qboolean = c_int;

// vec4_t and vec2_t - vector types
pub type vec4_t = [f32; 4];
pub type vec2_t = [f32; 2];

// color4ub_t - RGBA color as unsigned bytes
#[repr(C)]
pub struct color4ub_t {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

// SHADER_MAX_VERTEXES - constant from shader system
// TODO: Import from actual header definition when available
pub const SHADER_MAX_VERTEXES: usize = 1024;

pub struct CQuickSpriteSystem {
    // private:
    mTexBundle: *mut textureBundle_t,
    mGLStateBits: c_ulong,
    mFogIndex: c_int,
    mUseFog: qboolean,
    mVerts: [vec4_t; SHADER_MAX_VERTEXES],
    mIndexes: [c_uint; SHADER_MAX_VERTEXES],          // Ideally this would be static, cause it never changes
    mTextureCoords: [vec2_t; SHADER_MAX_VERTEXES],   // Ideally this would be static, cause it never changes
    mFogTextureCoords: [vec2_t; SHADER_MAX_VERTEXES],
    mColors: [c_ulong; SHADER_MAX_VERTEXES],
    mNextVert: c_int,
    mTurnCullBackOn: qboolean,
}

impl CQuickSpriteSystem {
    fn Flush(&mut self) {
        // Implementation in tr_quicksprite.cpp
    }

    // public:
    pub fn new() -> Self {
        // CQuickSpriteSystem constructor - implementation in tr_quicksprite.cpp
        CQuickSpriteSystem {
            mTexBundle: core::ptr::null_mut(),
            mGLStateBits: 0,
            mFogIndex: 0,
            mUseFog: 0,
            mVerts: [[0.0; 4]; SHADER_MAX_VERTEXES],
            mIndexes: [0; SHADER_MAX_VERTEXES],
            mTextureCoords: [[0.0; 2]; SHADER_MAX_VERTEXES],
            mFogTextureCoords: [[0.0; 2]; SHADER_MAX_VERTEXES],
            mColors: [0; SHADER_MAX_VERTEXES],
            mNextVert: 0,
            mTurnCullBackOn: 0,
        }
    }

    // ~CQuickSpriteSystem destructor - implementation in tr_quicksprite.cpp

    pub fn StartGroup(&mut self, bundle: *mut textureBundle_t, glbits: c_ulong, fogIndex: c_int) {
        // Default parameter in C++ signature: int fogIndex = -1
        // Implementation in tr_quicksprite.cpp
    }

    pub fn EndGroup(&mut self) {
        // Implementation in tr_quicksprite.cpp
    }

    pub fn Add(&mut self, pointdata: *const f32, color: color4ub_t, fog: *const vec2_t) {
        // Default parameter in C++ signature: vec2_t fog=NULL
        // Note: C++ signature appears to use NULL as default for non-pointer type;
        // translated as raw pointer to preserve C behavior.
        // Implementation in tr_quicksprite.cpp
    }
}

pub static mut SQuickSprite: CQuickSpriteSystem = CQuickSpriteSystem {
    mTexBundle: core::ptr::null_mut(),
    mGLStateBits: 0,
    mFogIndex: 0,
    mUseFog: 0,
    mVerts: [[0.0; 4]; SHADER_MAX_VERTEXES],
    mIndexes: [0; SHADER_MAX_VERTEXES],
    mTextureCoords: [[0.0; 2]; SHADER_MAX_VERTEXES],
    mFogTextureCoords: [[0.0; 2]; SHADER_MAX_VERTEXES],
    mColors: [0; SHADER_MAX_VERTEXES],
    mNextVert: 0,
    mTurnCullBackOn: 0,
};
