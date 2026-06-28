// ICARUS Interface header file

use core::ffi::{c_char, c_int, c_uint, c_void, c_ulong};

pub type DWORD = c_ulong;

pub type vec_t = f32;
pub type vec3_t = [f32; 3];

// Forward declarations for C++ classes (opaque in Rust)
pub struct CSequencer {
    _private: [u8; 0],
}

pub struct CTaskManager {
    _private: [u8; 0],
}

// Stub for engine type sharedEntity_t
pub struct sharedEntity_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct interface_export_s {
    // General
    pub I_LoadFile: Option<extern "C" fn(*const c_char, *mut *mut c_void) -> c_int>,
    pub I_CenterPrint: Option<extern "C" fn(*const c_char, ...) -> ()>,
    pub I_DPrintf: Option<extern "C" fn(c_int, *const c_char, ...) -> ()>,
    pub I_GetEntityByName: Option<extern "C" fn(*const c_char) -> *mut sharedEntity_t>, // Polls the engine for the sequencer of the entity matching the name passed
    pub I_GetTime: Option<extern "C" fn() -> DWORD>, // Gets the current time
    pub I_GetTimeScale: Option<extern "C" fn() -> DWORD>,
    pub I_PlaySound: Option<extern "C" fn(c_int, c_int, *const c_char, *const c_char) -> c_int>,
    pub I_Lerp2Pos: Option<extern "C" fn(c_int, c_int, *const vec3_t, *const vec3_t, f32) -> ()>,
    pub I_Lerp2Origin: Option<extern "C" fn(c_int, c_int, *const vec3_t, f32) -> ()>,
    pub I_Lerp2Angles: Option<extern "C" fn(c_int, c_int, *const vec3_t, f32) -> ()>,
    pub I_GetTag: Option<extern "C" fn(c_int, *const c_char, c_int, *mut vec3_t) -> c_int>,
    pub I_Lerp2Start: Option<extern "C" fn(c_int, c_int, f32) -> ()>,
    pub I_Lerp2End: Option<extern "C" fn(c_int, c_int, f32) -> ()>,
    pub I_Set: Option<extern "C" fn(c_int, c_int, *const c_char, *const c_char) -> ()>,
    pub I_Use: Option<extern "C" fn(c_int, *const c_char) -> ()>,
    pub I_Kill: Option<extern "C" fn(c_int, *const c_char) -> ()>,
    pub I_Remove: Option<extern "C" fn(c_int, *const c_char) -> ()>,
    pub I_Random: Option<extern "C" fn(f32, f32) -> f32>,
    pub I_Play: Option<extern "C" fn(c_int, c_int, *const c_char, *const c_char) -> ()>,

    // Camera functions
    pub I_CameraPan: Option<extern "C" fn(*const vec3_t, *const vec3_t, f32) -> ()>,
    pub I_CameraMove: Option<extern "C" fn(*const vec3_t, f32) -> ()>,
    pub I_CameraZoom: Option<extern "C" fn(f32, f32) -> ()>,
    pub I_CameraRoll: Option<extern "C" fn(f32, f32) -> ()>,
    pub I_CameraFollow: Option<extern "C" fn(*const c_char, f32, f32) -> ()>,
    pub I_CameraTrack: Option<extern "C" fn(*const c_char, f32, f32) -> ()>,
    pub I_CameraDistance: Option<extern "C" fn(f32, f32) -> ()>,
    pub I_CameraFade: Option<extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, f32) -> ()>,
    pub I_CameraPath: Option<extern "C" fn(*const c_char) -> ()>,
    pub I_CameraEnable: Option<extern "C" fn() -> ()>,
    pub I_CameraDisable: Option<extern "C" fn() -> ()>,
    pub I_CameraShake: Option<extern "C" fn(f32, c_int) -> ()>,

    pub I_GetFloat: Option<extern "C" fn(c_int, c_int, *const c_char, *mut f32) -> c_int>,
    pub I_GetVector: Option<extern "C" fn(c_int, c_int, *const c_char, *mut vec3_t) -> c_int>,
    pub I_GetString: Option<extern "C" fn(c_int, c_int, *const c_char, *mut *mut c_char) -> c_int>,

    pub I_Evaluate: Option<extern "C" fn(c_int, *const c_char, c_int, *const c_char, c_int) -> c_int>,

    pub I_DeclareVariable: Option<extern "C" fn(c_int, *const c_char) -> ()>,
    pub I_FreeVariable: Option<extern "C" fn(*const c_char) -> ()>,

    // Save / Load functions

    pub I_WriteSaveData: Option<extern "C" fn(c_ulong, *mut c_void, c_int) -> c_int>,
    // Below changed by BTO (VV). Visual C++ 7.1 compiler no longer allows default args on function pointers. Ack.
    pub I_ReadSaveData: Option<extern "C" fn(c_ulong, *mut c_void, c_int) -> c_int>,
    pub I_LinkEntity: Option<extern "C" fn(c_int, *mut CSequencer, *mut CTaskManager) -> c_int>,
}

pub type interface_export_t = interface_export_s;
