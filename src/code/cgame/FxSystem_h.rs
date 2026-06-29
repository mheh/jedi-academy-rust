// if !defined(CG_LOCAL_H_INC)
//  #include "cg_local.h"
// endif

// #ifndef FX_SYSTEM_H_INC
// #define FX_SYSTEM_H_INC

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// #define irand	Q_irand
pub use crate::code::qcommon::q_math::Q_irand as irand;

// #define flrand	Q_flrand
pub use crate::code::qcommon::q_math::Q_flrand as flrand;

extern "C" {
    pub static mut fx_debug: vmCvar_t;
    pub static mut fx_freeze: vmCvar_t;
}

// inline void Vector2Clear(vec2_t a)
// {
// 	a[0] = 0.0f;
// 	a[1] = 0.0f;
// }
#[inline]
pub fn Vector2Clear(a: *mut vec2_t) {
    unsafe {
        (*a)[0] = 0.0f32;
        (*a)[1] = 0.0f32;
    }
}

// inline void Vector2Set(vec2_t a,float b,float c)
// {
// 	a[0] = b;
// 	a[1] = c;
// }
#[inline]
pub fn Vector2Set(a: *mut vec2_t, b: f32, c: f32) {
    unsafe {
        (*a)[0] = b;
        (*a)[1] = c;
    }
}

// inline void Vector2Copy(vec2_t src,vec2_t dst)
// {
// 	dst[0] = src[0];
// 	dst[1] = src[1];
// }
#[inline]
pub fn Vector2Copy(src: *const vec2_t, dst: *mut vec2_t) {
    unsafe {
        (*dst)[0] = (*src)[0];
        (*dst)[1] = (*src)[1];
    }
}

extern "C" {
    pub fn CG_CalcEntityLerpPositions(cent: *mut centity_t);
}

// struct SFxHelper
// {
// 	int		mTime;
// 	int		mFrameTime;
// 	float	mFloatFrameTime;
//
// 	void	Init();
// 	void	AdjustTime( int time );
//
// 	// These functions are wrapped and used by the fx system in case it makes things a bit more portable
// 	void	Print( const char *msg, ... );
//
// 	// File handling
// 	int		OpenFile( const char *path, fileHandle_t *fh, int mode );
// 	int		ReadFile( void *data, int len, fileHandle_t fh );
// 	void	CloseFile( fileHandle_t fh );
//
// 	// Sound
// 	void	PlaySound( const vec3_t origin, int entityNum, int entchannel, sfxHandle_t sfx );
// 	void	PlayLocalSound( sfxHandle_t sfx, int channelNum );
// 	int		RegisterSound( const char *sound );
//
// #ifdef _IMMERSION
// 	void	PlayForce( int entityNum, ffHandle_t ff );
// 	ffHandle_t RegisterForce( const char *force, int channel );
// #endif // _IMMERSION
// 	//G2
// 	int		GetOriginAxisFromBolt(const centity_t &cent, int modelNum, int boltNum, vec3_t /*out*/origin, vec3_t /*out*/*axis);
//
// 	// Physics/collision
// 	void	Trace( trace_t *tr, vec3_t start, vec3_t min, vec3_t max, vec3_t end, int skipEntNum, int flags );
// 	void	G2Trace( trace_t *tr, vec3_t start, vec3_t min, vec3_t max, vec3_t end, int skipEntNum, int flags );
//
// 	void	AddFxToScene( refEntity_t *ent );
// 	void	AddLightToScene( vec3_t org, float radius, float red, float green, float blue );
//
// 	int		RegisterShader( const char *shader );
// 	int		RegisterModel( const char *model );
//
// 	void	AddPolyToScene( int shader, int count, polyVert_t *verts );
//
// 	void	CameraShake( vec3_t origin, float intensity, int radius, int time );
// };

#[repr(C)]
pub struct SFxHelper {
    pub mTime: c_int,
    pub mFrameTime: c_int,
    pub mFloatFrameTime: f32,
}

extern "C" {
    // void	Init();
    pub fn SFxHelper_Init(this: *mut SFxHelper);

    // void	AdjustTime( int time );
    pub fn SFxHelper_AdjustTime(this: *mut SFxHelper, time: c_int);

    // These functions are wrapped and used by the fx system in case it makes things a bit more portable
    // void	Print( const char *msg, ... );
    pub fn SFxHelper_Print(this: *mut SFxHelper, msg: *const c_char, ...);

    // File handling
    // int		OpenFile( const char *path, fileHandle_t *fh, int mode );
    pub fn SFxHelper_OpenFile(this: *mut SFxHelper, path: *const c_char, fh: *mut fileHandle_t, mode: c_int) -> c_int;

    // int		ReadFile( void *data, int len, fileHandle_t fh );
    pub fn SFxHelper_ReadFile(this: *mut SFxHelper, data: *mut c_void, len: c_int, fh: fileHandle_t) -> c_int;

    // void	CloseFile( fileHandle_t fh );
    pub fn SFxHelper_CloseFile(this: *mut SFxHelper, fh: fileHandle_t);

    // Sound
    // void	PlaySound( const vec3_t origin, int entityNum, int entchannel, sfxHandle_t sfx );
    pub fn SFxHelper_PlaySound(this: *mut SFxHelper, origin: *const vec3_t, entityNum: c_int, entchannel: c_int, sfx: sfxHandle_t);

    // void	PlayLocalSound( sfxHandle_t sfx, int channelNum );
    pub fn SFxHelper_PlayLocalSound(this: *mut SFxHelper, sfx: sfxHandle_t, channelNum: c_int);

    // int		RegisterSound( const char *sound );
    pub fn SFxHelper_RegisterSound(this: *mut SFxHelper, sound: *const c_char) -> c_int;

    // #ifdef _IMMERSION
    // void	PlayForce( int entityNum, ffHandle_t ff );
    pub fn SFxHelper_PlayForce(this: *mut SFxHelper, entityNum: c_int, ff: ffHandle_t);

    // ffHandle_t RegisterForce( const char *force, int channel );
    pub fn SFxHelper_RegisterForce(this: *mut SFxHelper, force: *const c_char, channel: c_int) -> ffHandle_t;
    // #endif // _IMMERSION

    // //G2
    // int		GetOriginAxisFromBolt(const centity_t &cent, int modelNum, int boltNum, vec3_t /*out*/origin, vec3_t /*out*/*axis);
    pub fn SFxHelper_GetOriginAxisFromBolt(this: *mut SFxHelper, cent: *const centity_t, modelNum: c_int, boltNum: c_int, origin: *mut vec3_t, axis: *mut *mut vec3_t) -> c_int;

    // Physics/collision
    // void	Trace( trace_t *tr, vec3_t start, vec3_t min, vec3_t max, vec3_t end, int skipEntNum, int flags );
    pub fn SFxHelper_Trace(this: *mut SFxHelper, tr: *mut trace_t, start: *const vec3_t, min: *const vec3_t, max: *const vec3_t, end: *const vec3_t, skipEntNum: c_int, flags: c_int);

    // void	G2Trace( trace_t *tr, vec3_t start, vec3_t min, vec3_t max, vec3_t end, int skipEntNum, int flags );
    pub fn SFxHelper_G2Trace(this: *mut SFxHelper, tr: *mut trace_t, start: *const vec3_t, min: *const vec3_t, max: *const vec3_t, end: *const vec3_t, skipEntNum: c_int, flags: c_int);

    // void	AddFxToScene( refEntity_t *ent );
    pub fn SFxHelper_AddFxToScene(this: *mut SFxHelper, ent: *mut refEntity_t);

    // void	AddLightToScene( vec3_t org, float radius, float red, float green, float blue );
    pub fn SFxHelper_AddLightToScene(this: *mut SFxHelper, org: *const vec3_t, radius: f32, red: f32, green: f32, blue: f32);

    // int		RegisterShader( const char *shader );
    pub fn SFxHelper_RegisterShader(this: *mut SFxHelper, shader: *const c_char) -> c_int;

    // int		RegisterModel( const char *model );
    pub fn SFxHelper_RegisterModel(this: *mut SFxHelper, model: *const c_char) -> c_int;

    // void	AddPolyToScene( int shader, int count, polyVert_t *verts );
    pub fn SFxHelper_AddPolyToScene(this: *mut SFxHelper, shader: c_int, count: c_int, verts: *mut polyVert_t);

    // void	CameraShake( vec3_t origin, float intensity, int radius, int time );
    pub fn SFxHelper_CameraShake(this: *mut SFxHelper, origin: *const vec3_t, intensity: f32, radius: c_int, time: c_int);
}

extern "C" {
    pub static mut theFxHelper: SFxHelper;
}

// #endif // FX_SYSTEM_H_INC

// Stub types for foreign dependencies
pub type vmCvar_t = c_int; // Placeholder; resolve from codebase definition
pub type centity_t = c_void; // Placeholder; resolve from cg_local.h
pub type vec3_t = [f32; 3];
pub type vec2_t = [f32; 2];
pub type fileHandle_t = c_int;
pub type sfxHandle_t = c_int;
pub type ffHandle_t = c_int;
pub type trace_t = c_void; // Placeholder
pub type refEntity_t = c_void; // Placeholder
pub type polyVert_t = c_void; // Placeholder
