// tr_QuickSprite.cpp: implementation of the CQuickSpriteSystem class.
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_ulong, c_void};
use crate::code::renderer::tr_quicksprite_h::*;

// Extern function declarations
extern "C" {
    pub fn R_BindAnimatedImage(bundle: *const textureBundle_t);
    pub fn GL_State(glStateBits: c_ulong);
    pub fn GL_Bind(image: *mut c_void);
    pub fn GLimp_LogComment(comment: *const u8);

    // OpenGL function pointers/stubs
    pub static qglEnableClientState: Option<extern "C" fn(cap: c_int)>;
    pub static qglTexCoordPointer: Option<extern "C" fn(size: c_int, gltype: c_int, stride: c_int, ptr: *const c_void)>;
    pub static qglColorPointer: Option<extern "C" fn(size: c_int, gltype: c_int, stride: c_int, ptr: *const c_void)>;
    pub static qglVertexPointer: Option<extern "C" fn(size: c_int, gltype: c_int, stride: c_int, ptr: *const c_void)>;
    pub static qglLockArraysEXT: Option<extern "C" fn(first: c_int, count: c_int)>;
    pub static qglUnlockArraysEXT: Option<extern "C" fn()>;
    pub static qglDrawArrays: Option<extern "C" fn(mode: c_int, first: c_int, count: c_int)>;
    pub static qglDisableClientState: Option<extern "C" fn(cap: c_int)>;
    pub static qglColor4ubv: Option<extern "C" fn(v: *const u8)>;
    pub static qglColor4ub: Option<extern "C" fn(r: u8, g: u8, b: u8, a: u8)>;
    pub static qglEnable: Option<extern "C" fn(cap: c_int)>;
    pub static qglDisable: Option<extern "C" fn(cap: c_int)>;
    pub static qglGetIntegerv: Option<extern "C" fn(pname: c_int, params: *mut c_int)>;
}

// OpenGL constants
const GL_TEXTURE_COORD_ARRAY: c_int = 0x8081;
const GL_COLOR_ARRAY: c_int = 0x8076;
const GL_CULL_FACE: c_int = 0x0B44;
const GL_FLOAT: c_int = 0x1406;
const GL_UNSIGNED_BYTE: c_int = 0x1401;
const GL_QUADS: c_int = 0x0007;

// GL state bits
const GLS_SRCBLEND_SRC_ALPHA: c_ulong = 0x00000005;
const GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA: c_ulong = 0x00000060;
const GLS_DEPTHFUNC_EQUAL: c_ulong = 0x00020000;

// External data structures
extern "C" {
    pub static mut backEnd: BackEnd;
    pub static mut tr: Tr;
    pub static mut r_drawfog: *mut Cvar;
}

// Stub structures for external types
#[repr(C)]
pub struct BackEnd {
    pub pc: PerfCounter,
    // ... other fields
}

#[repr(C)]
pub struct PerfCounter {
    pub c_vertexes: c_int,
    pub c_indexes: c_int,
    pub c_totalIndexes: c_int,
    // ... other fields
}

#[repr(C)]
pub struct Tr {
    pub world: *mut World,
    pub fogImage: *mut c_void,
    // ... other fields
}

#[repr(C)]
pub struct World {
    pub globalFog: c_int,
    pub fogs: *mut Fog,
    // ... other fields
}

#[repr(C)]
pub struct Fog {
    pub parms: FogParms,
    pub colorInt: c_uint,
    // ... other fields
}

#[repr(C)]
pub struct FogParms {
    pub color: [f32; 3],
    pub depthForOpaque: f32,
    // ... other fields
}

#[repr(C)]
pub struct Cvar {
    pub integer: c_int,
    // ... other fields
}

//////////////////////////////////////////////////////////////////////
// Construction/Destruction
//////////////////////////////////////////////////////////////////////

impl CQuickSpriteSystem {
    pub fn new() -> Self {
        let mut sprite_system = CQuickSpriteSystem {
            mTexBundle: core::ptr::null_mut(),
            mGLStateBits: 0,
            mFogIndex: 0,
            mUseFog: 0,
            mVerts: [[0.0; 4]; 1000],
            mIndexes: [0; 1000],
            mTextureCoords: [[0.0; 2]; 1000],
            mFogTextureCoords: [[0.0; 2]; 1000],
            mColors: [0; 1000],
            mNextVert: 0,
            mTurnCullBackOn: 0,
        };

        let mut i = 0;
        while i < 1000 {
            // Bottom right
            sprite_system.mTextureCoords[i + 0][0] = 1.0;
            sprite_system.mTextureCoords[i + 0][1] = 1.0;
            // Top right
            sprite_system.mTextureCoords[i + 1][0] = 1.0;
            sprite_system.mTextureCoords[i + 1][1] = 0.0;
            // Top left
            sprite_system.mTextureCoords[i + 2][0] = 0.0;
            sprite_system.mTextureCoords[i + 2][1] = 0.0;
            // Bottom left
            sprite_system.mTextureCoords[i + 3][0] = 0.0;
            sprite_system.mTextureCoords[i + 3][1] = 1.0;

            i += 4;
        }

        sprite_system
    }

    // ~CQuickSpriteSystem destructor does nothing in C++
}

impl CQuickSpriteSystem {
    fn Flush(&mut self) {
        unsafe {
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
            //this should not be needed, since I just wait to disable fog for the surface til after surface sprites are done

            //
            // render the main pass
            //
            R_BindAnimatedImage(self.mTexBundle);
            GL_State(self.mGLStateBits);

            //
            // set arrays and lock
            //
            if let Some(f) = qglEnableClientState {
                f(GL_TEXTURE_COORD_ARRAY);
            }
            if let Some(f) = qglTexCoordPointer {
                f(2, GL_FLOAT, 0, self.mTextureCoords.as_ptr() as *const c_void);
            }

            if let Some(f) = qglEnableClientState {
                f(GL_COLOR_ARRAY);
            }
            if let Some(f) = qglColorPointer {
                f(4, GL_UNSIGNED_BYTE, 0, self.mColors.as_ptr() as *const c_void);
            }

            if let Some(f) = qglVertexPointer {
                f(3, GL_FLOAT, 16, self.mVerts.as_ptr() as *const c_void);
            }

            if let Some(f) = qglLockArraysEXT {
                f(0, self.mNextVert);
                GLimp_LogComment(b"glLockArraysEXT\n".as_ptr());
            }

            if let Some(f) = qglDrawArrays {
                f(GL_QUADS, 0, self.mNextVert);
            }

            backEnd.pc.c_vertexes += self.mNextVert;
            backEnd.pc.c_indexes += self.mNextVert;
            backEnd.pc.c_totalIndexes += self.mNextVert;

            //only for software fog pass (global soft/volumetric) -rww
            if self.mUseFog != 0
                && (r_drawfog.is_null()
                    || (*r_drawfog).integer != 2
                    || self.mFogIndex != (*tr.world).globalFog)
            {
                let fog = (*tr.world).fogs.offset(self.mFogIndex as isize);

                //
                // render the fog pass
                //
                GL_Bind(tr.fogImage);
                GL_State(
                    GLS_SRCBLEND_SRC_ALPHA
                        | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA
                        | GLS_DEPTHFUNC_EQUAL,
                );

                //
                // set arrays and lock
                //
                if let Some(f) = qglTexCoordPointer {
                    f(2, GL_FLOAT, 0, self.mFogTextureCoords.as_ptr() as *const c_void);
                }
                //		qglEnableClientState( GL_TEXTURE_COORD_ARRAY);	// Done above

                if let Some(f) = qglDisableClientState {
                    f(GL_COLOR_ARRAY);
                }
                if let Some(f) = qglColor4ubv {
                    f((&(*fog).colorInt as *const c_uint) as *const u8);
                }

                //		qglVertexPointer (3, GL_FLOAT, 16, mVerts);	// Done above

                if let Some(f) = qglDrawArrays {
                    f(GL_QUADS, 0, self.mNextVert);
                }

                // Second pass from fog
                backEnd.pc.c_totalIndexes += self.mNextVert;
            }

            //
            // unlock arrays
            //
            if let Some(f) = qglUnlockArraysEXT {
                f();
                GLimp_LogComment(b"glUnlockArraysEXT\n".as_ptr());
            }

            self.mNextVert = 0;
        }
    }

    pub fn StartGroup(&mut self, bundle: *mut textureBundle_t, glbits: c_ulong, fogIndex: c_int) {
        unsafe {
            self.mNextVert = 0;

            self.mTexBundle = bundle;
            self.mGLStateBits = glbits;
            if fogIndex != -1 {
                self.mUseFog = 1; // qtrue
                self.mFogIndex = fogIndex;
            } else {
                self.mUseFog = 0; // qfalse
            }

            let mut cullingOn: c_int = 0;
            if let Some(f) = qglGetIntegerv {
                f(GL_CULL_FACE, &mut cullingOn);
            }

            if cullingOn != 0 {
                self.mTurnCullBackOn = 1; // true
            } else {
                self.mTurnCullBackOn = 0; // false
            }
            if let Some(f) = qglDisable {
                f(GL_CULL_FACE);
            }
        }
    }

    pub fn EndGroup(&mut self) {
        unsafe {
            self.Flush();

            if let Some(f) = qglColor4ub {
                f(255, 255, 255, 255);
            }
            if self.mTurnCullBackOn != 0 {
                if let Some(f) = qglEnable {
                    f(GL_CULL_FACE);
                }
            }
        }
    }

    pub fn Add(&mut self, pointdata: *const f32, color: color4ub_t, fog: *const [f32; 2]) {
        unsafe {
            if self.mNextVert > 1000 - 4 {
                self.Flush();
            }

            let curcoord = self.mVerts[self.mNextVert as usize].as_mut_ptr();
            core::ptr::copy_nonoverlapping(pointdata, curcoord, 4 * 4); // 4 * sizeof(vec4_t)

            // Set up color
            let curcolor = &mut self.mColors[self.mNextVert as usize] as *mut c_ulong;
            let color_val = *((&color as *const color4ub_t) as *const c_ulong);
            *curcolor = color_val;
            *curcolor.offset(1) = color_val;
            *curcolor.offset(2) = color_val;
            *curcolor.offset(3) = color_val;

            if !fog.is_null() {
                let curfogtexcoord =
                    &mut self.mFogTextureCoords[self.mNextVert as usize][0] as *mut f32;
                *curfogtexcoord = (*fog)[0];
                *curfogtexcoord.offset(1) = (*fog)[1];

                *curfogtexcoord.offset(2) = (*fog)[0];
                *curfogtexcoord.offset(3) = (*fog)[1];

                *curfogtexcoord.offset(4) = (*fog)[0];
                *curfogtexcoord.offset(5) = (*fog)[1];

                *curfogtexcoord.offset(6) = (*fog)[0];
                *curfogtexcoord.offset(7) = (*fog)[1];

                self.mUseFog = 1; // qtrue
            } else {
                self.mUseFog = 0; // qfalse
            }

            self.mNextVert += 4;
        }
    }
}
