// Filename: tr_public.h

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use super::tr_types_h::*;

// get font functions with default arguments that we need below
// (Xbox specific include, not translated as it's Xbox-specific C++ template code)

pub const REF_API_VERSION: c_int = 9;

// Type alias for force reload enum (from q_shared.h)
pub type ForceReload_e = c_int;

//
// these are the functions exported by the refresh module
//
#[repr(C)]
pub struct refexport_t {
	// called before the library is unloaded
	// if the system is just reconfiguring, pass destroyWindow = qfalse,
	// which will keep the screen from flashing to the desktop.
	pub Shutdown: Option<extern "C" fn(qboolean)>,

	// All data that will be used in a level should be
	// registered before rendering any frames to prevent disk hits,
	// but they can still be registered at a later time
	// if necessary.
	//
	// BeginRegistration makes any existing media pointers invalid
	// and returns the current gl configuration, including screen width
	// and height, which can be used by the client to intelligently
	// size display elements
	pub BeginRegistration: Option<extern "C" fn(*mut glconfig_t)>,
	pub RegisterModel: Option<extern "C" fn(*const c_char) -> qhandle_t>,
	pub RegisterSkin: Option<extern "C" fn(*const c_char) -> qhandle_t>,
	pub GetAnimationCFG: Option<extern "C" fn(*const c_char, *mut c_char, c_int) -> c_int>,
	pub RegisterShader: Option<extern "C" fn(*const c_char) -> qhandle_t>,
	pub RegisterShaderNoMip: Option<extern "C" fn(*const c_char) -> qhandle_t>,
	pub LoadWorld: Option<extern "C" fn(*const c_char)>,

	// these two functions added to help with the new model alloc scheme...
	//
	pub RegisterMedia_LevelLoadBegin: Option<extern "C" fn(*const c_char, ForceReload_e, qboolean)>,
	pub RegisterMedia_LevelLoadEnd: Option<extern "C" fn()>,

	// the vis data is a large enough block of data that we go to the trouble
	// of sharing it with the clipmodel subsystem
	pub SetWorldVisData: Option<extern "C" fn(*const u8)>,

	// EndRegistration will draw a tiny polygon with each texture, forcing
	// them to be loaded into card memory
	pub EndRegistration: Option<extern "C" fn()>,

	// a scene is built up by calls to R_ClearScene and the various R_Add functions.
	// Nothing is drawn until R_RenderScene is called.
	pub ClearScene: Option<extern "C" fn()>,
	pub AddRefEntityToScene: Option<extern "C" fn(*const refEntity_t)>,
	pub AddPolyToScene: Option<extern "C" fn(qhandle_t, c_int, *const polyVert_t)>,
	pub AddLightToScene: Option<extern "C" fn(*const vec3_t, f32, f32, f32, f32)>,
	pub RenderScene: Option<extern "C" fn(*const refdef_t)>,
	pub GetLighting: Option<extern "C" fn(*const vec3_t, *mut vec3_t, *mut vec3_t, *mut vec3_t) -> qboolean>,

	pub SetColor: Option<extern "C" fn(*const f32)>,	// NULL = 1,1,1,1
	pub DrawStretchPic: Option<extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, qhandle_t)>,	// 0 = white
	pub DrawRotatePic: Option<extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, f32, qhandle_t)>,	// 0 = white
	pub DrawRotatePic2: Option<extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, f32, qhandle_t)>,	// 0 = white
	pub LAGoggles: Option<extern "C" fn()>,
	pub Scissor: Option<extern "C" fn(f32, f32, f32, f32)>,	// 0 = white

	// Draw images for cinematic rendering, pass as 32 bit rgba
	pub DrawStretchRaw: Option<extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, *const u8, c_int, qboolean)>,
	pub UploadCinematic: Option<extern "C" fn(c_int, c_int, *const u8, c_int, qboolean)>,

	pub BeginFrame: Option<extern "C" fn(stereoFrame_t)>,

	// if the pointers are not NULL, timing info will be returned
	pub EndFrame: Option<extern "C" fn(*mut c_int, *mut c_int)>,

	pub ProcessDissolve: Option<extern "C" fn() -> qboolean>,
	pub InitDissolve: Option<extern "C" fn(qboolean) -> qboolean>,


	// for use with save-games mainly...
	pub GetScreenShot: Option<extern "C" fn(*mut u8, c_int, c_int)>,

	// this is so you can get access to raw pixels from a graphics format (TGA/JPG/BMP etc),
	//	currently only the save game uses it (to make raw shots for the autosaves)
	//
	pub TempRawImage_ReadFromFile: Option<extern "C" fn(*const c_char, *mut c_int, *mut c_int, *mut u8, qboolean) -> *mut u8>,
	pub TempRawImage_CleanUp: Option<extern "C" fn()>,

	//misc stuff
	pub MarkFragments: Option<extern "C" fn(c_int, *const *const vec3_t, *const vec3_t, c_int, *mut vec3_t, c_int, *mut markFragment_t) -> c_int>,

	//model stuff
	pub LerpTag: Option<extern "C" fn(*mut orientation_t, qhandle_t, c_int, c_int, f32, *const c_char)>,
	pub ModelBounds: Option<extern "C" fn(qhandle_t, *mut vec3_t, *mut vec3_t)>,

	pub GetLightStyle: Option<extern "C" fn(c_int, *mut color4ub_t)>,
	pub SetLightStyle: Option<extern "C" fn(c_int, c_int)>,

	pub GetBModelVerts: Option<extern "C" fn(c_int, *mut *mut vec3_t, *mut vec3_t)>,
	pub WorldEffectCommand: Option<extern "C" fn(*const c_char)>,

	pub RegisterFont: Option<extern "C" fn(*const c_char) -> c_int>,
	pub Font_HeightPixels: Option<extern "C" fn(c_int, f32) -> c_int>,
	pub Font_StrLenPixels: Option<extern "C" fn(*const c_char, c_int, f32) -> c_int>,
	pub Font_DrawString: Option<extern "C" fn(c_int, c_int, *const c_char, *const f32, c_int, c_int, f32)>,
	pub Font_StrLenChars: Option<extern "C" fn(*const c_char) -> c_int>,
	pub Language_IsAsian: Option<extern "C" fn() -> qboolean>,
	pub Language_UsesSpaces: Option<extern "C" fn() -> qboolean>,
	pub AnyLanguage_ReadCharFromString: Option<extern "C" fn(*const c_char, *mut c_int, *mut qboolean) -> c_uint>,
}

// Stub types for forward references needed by refexport_t
// These are defined elsewhere in the codebase but needed here for structural completeness
#[repr(C)]
pub struct markFragment_t {
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct orientation_t {
	_opaque: [u8; 0],
}

// this is the only function actually exported at the linker level
// If the module can't init to a valid rendering state, NULL will be
// returned.
extern "C" {
	pub fn GetRefAPI(apiVersion: c_int) -> *mut refexport_t;
}
