// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use core::ptr::addr_of_mut;

// tr_QuickSprite.cpp: implementation of the CQuickSpriteSystem class.
//
//////////////////////////////////////////////////////////////////////
// #include "../server/exe_headers.h"
// #include "tr_local.h"
// #include "tr_QuickSprite.h"

// Vector types (C fixed arrays)
pub type vec4_t = [f32; 4];
pub type vec2_t = [f32; 2];

// Forward declarations for opaque types
#[repr(C)]
pub struct textureBundle_t {
    _opaque: [u8; 0],
}

// color4ub_t is a 4-byte array representing RGBA
pub type color4ub_t = [u8; 4];

// SHADER_MAX_VERTEXES from qfiles.h
const SHADER_MAX_VERTEXES: usize = 1000;

#[repr(C)]
pub struct CQuickSpriteSystem {
    mTexBundle: *mut textureBundle_t,
    mGLStateBits: c_ulong,
    mFogIndex: c_int,
    mUseFog: c_int,
    mVerts: [vec4_t; SHADER_MAX_VERTEXES],
    mIndexes: [c_uint; SHADER_MAX_VERTEXES],        // Ideally this would be static, cause it never changes
    mTextureCoords: [vec2_t; SHADER_MAX_VERTEXES], // Ideally this would be static, cause it never changes
    mFogTextureCoords: [vec2_t; SHADER_MAX_VERTEXES],
    mColors: [c_ulong; SHADER_MAX_VERTEXES],
    mNextVert: c_int,
}

// External C functions
extern "C" {
    fn R_BindAnimatedImage(bundle: *mut textureBundle_t);
    fn GL_State(stateBits: c_ulong);
    fn qglTexCoordPointer(size: c_int, gltype: c_int, stride: c_int, pointer: *const c_void);
    fn qglEnableClientState(cap: c_int);
    fn qglColorPointer(size: c_int, gltype: c_int, stride: c_int, pointer: *const c_void);
    fn qglVertexPointer(size: c_int, gltype: c_int, stride: c_int, pointer: *const c_void);
    fn qglLockArraysEXT(first: c_int, count: c_int);
    fn qglUnlockArraysEXT();
    fn qglDisableClientState(cap: c_int);
    fn qglColor4ubv(v: *const u8);
    fn qglDrawArrays(mode: c_int, first: c_int, count: c_int);
    fn qglDisable(cap: c_int);
    fn qglEnable(cap: c_int);
    fn GLimp_LogComment(comment: *const c_char);
    fn GL_Bind(image: *mut c_void);

    pub static mut SQuickSprite: CQuickSpriteSystem;
    pub static mut backEnd: backEnd_t;
    pub static mut tr: tr_t;
}

// Stub types for external dependencies
#[repr(C)]
pub struct backEnd_t {
    pub pc: performanceCounters_t,
}

#[repr(C)]
pub struct performanceCounters_t {
    pub c_vertexes: c_int,
    pub c_indexes: c_int,
    pub c_totalIndexes: c_int,
}

#[repr(C)]
pub struct tr_t {
    pub fogImage: *mut c_void,
    pub world: *mut world_t,
}

#[repr(C)]
pub struct world_t {
    pub globalFog: c_int,
    pub fogs: *mut fog_t,
}

#[repr(C)]
pub struct fog_t {
    pub colorInt: u32,
}

// GL Constants
const GL_QUADS: c_int = 0x0007;
const GL_FLOAT: c_int = 0x1406;
const GL_UNSIGNED_BYTE: c_int = 0x1401;
const GL_TEXTURE_COORD_ARRAY: c_int = 0x8078;
const GL_COLOR_ARRAY: c_int = 0x8076;
const GL_VERTEX_ARRAY: c_int = 0x8074;
const GL_CULL_FACE: c_int = 0x0B44;
const GL_UNSIGNED_INT: c_int = 0x1405;
const GL_FOG: c_int = 0x0B60;

const GLS_SRCBLEND_SRC_ALPHA: c_ulong = 0x00000004;
const GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA: c_ulong = 0x00000008;
const GLS_DEPTHFUNC_EQUAL: c_ulong = 0x00004000;

//////////////////////////////////////////////////////////////////////
// Singleton System
//////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////
// Construction/Destruction
//////////////////////////////////////////////////////////////////////

impl CQuickSpriteSystem {
    pub fn new() -> Self {
        let mut system = CQuickSpriteSystem {
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
        };

        let mut i: c_int = 0;
        while i < SHADER_MAX_VERTEXES as c_int {
            let idx = i as usize;
            // Bottom right
            system.mTextureCoords[idx + 0][0] = 1.0;
            system.mTextureCoords[idx + 0][1] = 1.0;
            // Top right
            system.mTextureCoords[idx + 1][0] = 1.0;
            system.mTextureCoords[idx + 1][1] = 0.0;
            // Top left
            system.mTextureCoords[idx + 2][0] = 0.0;
            system.mTextureCoords[idx + 2][1] = 0.0;
            // Bottom left
            system.mTextureCoords[idx + 3][0] = 0.0;
            system.mTextureCoords[idx + 3][1] = 1.0;
            i += 4;
        }

        system
    }
}

impl CQuickSpriteSystem {
    pub fn Flush(&mut self) {
        if self.mNextVert == 0 {
            return;
        }

        /*
        if (mUseFog && r_drawfog->integer == 2 &&
            mFogIndex == tr.world->globalFog)
        { //enable hardware fog when we draw this thing if applicable -rww
            fog_t *fog = tr.world->fogs + mFogIndex;

        #ifdef _XBOX
            qglFogi(GL_FOG_MODE, GL_EXP2);
        #else
            qglFogf(GL_FOG_MODE, GL_EXP2);
        #endif
            qglFogf(GL_FOG_DENSITY, logtestExp2 / fog->parms.depthForOpaque);
            qglFogfv(GL_FOG_COLOR, fog->parms.color);
            qglEnable(GL_FOG);
        }
        */
        // this should not be needed, since I just wait to disable fog for the surface til after surface sprites are done

        // render the main pass
        unsafe {
            R_BindAnimatedImage(self.mTexBundle);
            GL_State(self.mGLStateBits);

            // set arrays and lock
            qglTexCoordPointer(2, GL_FLOAT, 0, self.mTextureCoords.as_ptr() as *const c_void);
            qglEnableClientState(GL_TEXTURE_COORD_ARRAY);

            qglEnableClientState(GL_COLOR_ARRAY);
            qglColorPointer(4, GL_UNSIGNED_BYTE, 0, self.mColors.as_ptr() as *const c_void);

            qglVertexPointer(3, GL_FLOAT, 16, self.mVerts.as_ptr() as *const c_void);

            if !qglLockArraysEXT as *const _ as usize == 0 {
                qglLockArraysEXT(0, self.mNextVert);
                GLimp_LogComment(b"glLockArraysEXT\n\0".as_ptr() as *const c_char);
            }

            qglDrawArrays(GL_QUADS, 0, self.mNextVert);

            backEnd.pc.c_vertexes += self.mNextVert;
            backEnd.pc.c_indexes += self.mNextVert;
            backEnd.pc.c_totalIndexes += self.mNextVert;

            // only for software fog pass (global soft/volumetric) -rww
            if self.mUseFog != 0
                && (false || self.mFogIndex != (*tr.world).globalFog)
            {
                let fog: *mut fog_t = (*tr.world).fogs.add(self.mFogIndex as usize);

                // render the fog pass
                GL_Bind(tr.fogImage);
                GL_State(GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA | GLS_DEPTHFUNC_EQUAL);

                // set arrays and lock
                qglTexCoordPointer(
                    2,
                    GL_FLOAT,
                    0,
                    self.mFogTextureCoords.as_ptr() as *const c_void,
                );
                // qglEnableClientState( GL_TEXTURE_COORD_ARRAY);	// Done above

                qglDisableClientState(GL_COLOR_ARRAY);
                qglColor4ubv(&(*fog).colorInt as *const u32 as *const u8);

                // qglVertexPointer (3, GL_FLOAT, 16, mVerts);	// Done above

                qglDrawArrays(GL_QUADS, 0, self.mNextVert);

                // Second pass from fog
                backEnd.pc.c_totalIndexes += self.mNextVert;
            }

            // unlock arrays
            if !qglUnlockArraysEXT as *const _ as usize == 0 {
                qglUnlockArraysEXT();
                GLimp_LogComment(b"glUnlockArraysEXT\n\0".as_ptr() as *const core::ffi::c_char);
            }
        }

        self.mNextVert = 0;
    }

    pub fn StartGroup(&mut self, bundle: *mut textureBundle_t, glbits: c_ulong, fogIndex: c_int) {
        self.mNextVert = 0;

        self.mTexBundle = bundle;
        self.mGLStateBits = glbits;
        if fogIndex != -1 {
            self.mUseFog = 1; // qtrue
            self.mFogIndex = fogIndex;
        } else {
            self.mUseFog = 0; // qfalse
        }

        unsafe {
            qglDisable(GL_CULL_FACE);
        }
    }

    pub fn EndGroup(&mut self) {
        self.Flush();

        unsafe {
            qglColor4ubv(&[255u8, 255u8, 255u8, 255u8] as *const u8);
            qglEnable(GL_CULL_FACE);
        }
    }

    pub fn Add(&mut self, pointdata: *const f32, color: *const color4ub_t, fog: *const vec2_t) {
        if self.mNextVert > SHADER_MAX_VERTEXES as c_int - 4 {
            self.Flush();
        }

        let idx = self.mNextVert as usize;

        // Copy 4 vec4_t (16 floats) from pointdata to mVerts[mNextVert]
        unsafe {
            core::ptr::copy_nonoverlapping(
                pointdata,
                self.mVerts[idx].as_mut_ptr(),
                16,
            );
        }

        // Set up color
        // Cast color to u32 and replicate it 4 times
        let color_value: c_ulong = unsafe {
            // Read the 4 bytes as a 32-bit little-endian value
            *(color as *const c_ulong)
        };

        for i in 0..4 {
            self.mColors[idx + i] = color_value;
        }

        if !fog.is_null() {
            unsafe {
                for i in 0..4 {
                    self.mFogTextureCoords[idx + i][0] = (*fog)[0];
                    self.mFogTextureCoords[idx + i][1] = (*fog)[1];
                }
            }

            self.mUseFog = 1; // qtrue
        } else {
            self.mUseFog = 0; // qfalse
        }

        self.mNextVert += 4;
    }
}
