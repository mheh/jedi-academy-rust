#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_ulong};

// this include must remain at the top of every CPP file
// #include "../game/q_math.h"
// #include "tr_headers.h"

// tr_QuickSprite.h: interface for the CQuickSprite class.
//
//////////////////////////////////////////////////////////////////////

// Forward declarations for types used in method signatures and fields
#[repr(C)]
pub struct textureBundle_t;

#[repr(C)]
pub struct color4ub_t;

// Vector types (C fixed arrays)
pub type vec4_t = [f32; 4];
pub type vec2_t = [f32; 2];

// SHADER_MAX_VERTEXES is defined in tr_headers.h or q_math.h
// Placeholder value must match C code definitions
const SHADER_MAX_VERTEXES: usize = 4096;

#[repr(C)]
pub struct CQuickSpriteSystem {
    mTexBundle: *mut textureBundle_t,
    mGLStateBits: c_ulong,
    mFogIndex: c_int,
    mUseFog: c_int,
    mVerts: [vec4_t; SHADER_MAX_VERTEXES],
    mIndexes: [c_uint; SHADER_MAX_VERTEXES],           // Ideally this would be static, cause it never changes
    mTextureCoords: [vec2_t; SHADER_MAX_VERTEXES],    // Ideally this would be static, cause it never changes
    mFogTextureCoords: [vec2_t; SHADER_MAX_VERTEXES],
    mColors: [c_ulong; SHADER_MAX_VERTEXES],
    mNextVert: c_int,
}

// Method declarations (implementations in tr_quicksprite.cpp):
// private:
//     void Flush(void);
// public:
//     CQuickSpriteSystem();
//     virtual ~CQuickSpriteSystem();
//     void StartGroup(textureBundle_t *bundle, unsigned long glbits, int fogIndex = -1);
//     void EndGroup(void);
//     void Add(float *pointdata, color4ub_t color, vec2_t fog=NULL);

extern "C" {
    pub static mut SQuickSprite: CQuickSpriteSystem;
}
