//! Mechanical port of `oracle/code/cgame/cg_camera.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_char};

use crate::codemp::game::q_shared_h::{vec3_t, vec4_t, qboolean, MAX_QPATH};

//
// cg_camera.cpp

pub const MAX_CAMERA_GROUP_SUBJECTS: c_int = 16;
pub const MAX_ACCEL_PER_FRAME: f32 = 10.0f32;
pub const MAX_SHAKE_INTENSITY: f32 = 16.0f32;
pub const CAMERA_DEFAULT_FOV: f32 = 90.0f32;
pub const CAMERA_WIDESCREEN_FOV: f32 = 120.0f32;
pub const BAR_DURATION: f32 = 1000.0f32;
pub const BAR_RATIO: f32 = 48.0f32;

pub const CAMERA_MOVING: c_int = 0x00000001;
pub const CAMERA_PANNING: c_int = 0x00000002;
pub const CAMERA_ZOOMING: c_int = 0x00000004;
pub const CAMERA_BAR_FADING: c_int = 0x00000008;
pub const CAMERA_FADING: c_int = 0x00000010;
pub const CAMERA_FOLLOWING: c_int = 0x00000020;
pub const CAMERA_TRACKING: c_int = 0x00000040;
pub const CAMERA_ROFFING: c_int = 0x00000080;
pub const CAMERA_SMOOTHING: c_int = 0x00000100;
pub const CAMERA_CUT: c_int = 0x00000200;
pub const CAMERA_ACCEL: c_int = 0x00000400;

// NOTE!! This structure is now saved out as part of the load/save game, so tell me if you put any pointers or
//	other goofy crap in... -Ste
//
#[repr(C)]
pub struct camera_s {
    // Position / Facing information
    pub origin: vec3_t,
    pub angles: vec3_t,

    pub origin2: vec3_t,
    pub angles2: vec3_t,

    // Movement information
    pub move_duration: f32,
    pub move_time: f32,
    pub move_type: c_int, // CMOVE_LINEAR, CMOVE_BEZIER

    // FOV information
    pub FOV: f32,
    pub FOV2: f32,
    pub FOV_duration: f32,
    pub FOV_time: f32,
    pub FOV_vel: f32,
    pub FOV_acc: f32,

    // Pan information
    pub pan_time: f32,
    pub pan_duration: f32,

    // Following information
    pub cameraGroup: [c_char; MAX_QPATH],
    pub cameraGroupZOfs: f32,
    pub cameraGroupTag: [c_char; MAX_QPATH],
    pub subjectPos: vec3_t,
    pub subjectSpeed: f32,
    pub followSpeed: f32,
    pub followInitLerp: qboolean,
    pub distance: f32,
    pub distanceInitLerp: qboolean,
    // int		aimEntNum;//FIXME: remove

    // Tracking information
    pub trackEntNum: c_int,
    pub trackToOrg: vec3_t,
    pub moveDir: vec3_t,
    pub speed: f32,
    pub initSpeed: f32,
    pub trackInitLerp: f32,
    pub nextTrackEntUpdateTime: c_int,

    // Cine-bar information
    pub bar_alpha: f32,
    pub bar_alpha_source: f32,
    pub bar_alpha_dest: f32,
    pub bar_time: f32,

    pub bar_height_source: f32,
    pub bar_height_dest: f32,
    pub bar_height: f32,

    pub fade_color: vec4_t,
    pub fade_source: vec4_t,
    pub fade_dest: vec4_t,
    pub fade_time: f32,
    pub fade_duration: f32,

    // State information
    pub info_state: c_int,

    // Shake information
    pub shake_intensity: f32,
    pub shake_duration: c_int,
    pub shake_start: c_int,

    // Smooth information
    pub smooth_intensity: f32,
    pub smooth_duration: c_int,
    pub smooth_start: c_int,
    pub smooth_origin: vec3_t,
    pub smooth_active: bool, // means smooth_origin and angles are valid

    // ROFF information
    pub sRoff: [c_char; MAX_QPATH], // name of a cached roff
    pub roff_frame: c_int, // current frame in the roff data
    pub next_roff_time: c_int, // time when it's ok to apply the next roff frame

    #[cfg(feature = "xbox")]
    pub widescreen: qboolean,
}

extern "C" {
    pub static mut in_camera: bool;
    pub static mut client_camera: camera_s;

    pub fn CGCam_Init();

    pub fn CGCam_Enable();
    pub fn CGCam_Disable();

    #[cfg(feature = "xbox")]
    pub fn CGCam_SetWidescreen(widescreen: qboolean);

    pub fn CGCam_SetPosition(org: vec3_t);
    pub fn CGCam_SetAngles(ang: vec3_t);
    pub fn CGCam_SetFOV(FOV: f32);
    #[cfg(feature = "xbox")]
    pub fn CGCam_SetFOV2(FOV2: f32);

    pub fn CGCam_Zoom(FOV: f32, duration: f32);
    // void CGCam_Pan( vec3_t	dest, float duration );
    pub fn CGCam_Pan(dest: vec3_t, panDirection: vec3_t, duration: f32);
    pub fn CGCam_Move(dest: vec3_t, duration: f32);
    pub fn CGCam_Fade(source: vec4_t, dest: vec4_t, duration: f32);

    pub fn CGCam_UpdateFade();

    pub fn CGCam_Update();
    pub fn CGCam_RenderScene();
    pub fn CGCam_DrawWideScreen();

    pub fn CGCam_Shake(intensity: f32, duration: c_int);
    pub fn CGCam_UpdateShake(origin: vec3_t, angles: vec3_t);

    pub fn CGCam_Follow(cameraGroup: *const c_char, speed: f32, initLerp: f32);
    pub fn CGCam_Track(trackName: *const c_char, speed: f32, initLerp: f32);
    pub fn CGCam_Distance(distance: f32, initLerp: f32);
    pub fn CGCam_Roll(dest: f32, duration: f32);

    pub fn CGCam_StartRoff(roff: *mut c_char);

    pub fn CGCam_Smooth(intensity: f32, duration: c_int);
    pub fn CGCam_UpdateSmooth(origin: vec3_t, angles: vec3_t);
}
