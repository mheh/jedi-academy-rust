//Client camera controls for cinematics

// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"

// #include "cg_media.h"

// #include "..\game\g_roff.h"

use core::ffi::{c_int, c_char, c_void};

pub static mut in_camera: bool = false;
pub static mut client_camera: camera_t = camera_t::ZEROED;
extern "C" {
	pub static mut player_locked: u32;  // qboolean stub
}

extern "C" {
	pub fn G_Find(from: *mut gentity_t, fieldofs: c_int, match_: *const c_char) -> *mut gentity_t;
	pub fn G_UseTargets(ent: *mut gentity_t, activator: *mut gentity_t);
	pub fn CG_CalcFOVFromX(fov_x: f32) -> c_int;
	pub fn WP_SaberCatch(slf: *mut gentity_t, saber: *mut gentity_t, switchToSaber: u32); // qboolean
	pub fn WP_ForcePowerStop(slf: *mut gentity_t, forcePower: c_int); // forcePowers_t
	pub fn CG_CalcVrect();
	pub fn G_LoadRoff(roff: *const c_char) -> c_int;
	pub static mut g_entities: [gentity_t; 2048]; // MAX_GENTITIES stub
	pub static mut cg_entities: [centity_t; 2048]; // MAX_GENTITIES stub
	pub static mut cg: cg_t;
	pub static mut cgs: cgs_t;
	pub static mut gi: game_import_t;
	pub static mut cg_developer: cvar_t;
	pub static mut cg_roffdebug: cvar_t;
	pub static mut cg_roffval1: cvar_t;
	pub static mut cg_roffval2: cvar_t;
	pub static mut cg_roffval3: cvar_t;
	pub static mut cg_roffval4: cvar_t;
	pub static mut roffs: [roff_list_t; 64]; // MAX_ROFFS stub
}

pub struct camera_t {
	pub origin: [f32; 3],
	pub angles: [f32; 3],
	pub origin2: [f32; 3],
	pub angles2: [f32; 3],
	pub info_state: c_int,
	pub FOV: f32,
	pub FOV2: f32,
	pub FOV_time: i32,
	pub FOV_duration: f32,
	pub FOV_vel: f32,
	pub FOV_acc: f32,
	pub bar_alpha: f32,
	pub bar_alpha_source: f32,
	pub bar_alpha_dest: f32,
	pub bar_height: f32,
	pub bar_height_source: f32,
	pub bar_height_dest: f32,
	pub bar_time: i32,
	pub fade_color: [f32; 4],
	pub fade_source: [f32; 4],
	pub fade_dest: [f32; 4],
	pub fade_time: i32,
	pub fade_duration: f32,
	pub moveDir: [f32; 3],
	pub move_time: i32,
	pub move_duration: f32,
	pub pan_time: i32,
	pub pan_duration: f32,
	pub followInitLerp: bool,
	pub followSpeed: f32,
	pub cameraGroup: [c_char; 64],
	pub cameraGroupZOfs: f32,
	pub cameraGroupTag: [c_char; 64],
	pub subjectPos: [f32; 3],
	pub subjectSpeed: f32,
	pub trackEntNum: c_int,
	pub initSpeed: f32,
	pub speed: f32,
	pub distance: f32,
	pub distanceInitLerp: bool,
	pub trackToOrg: [f32; 3],
	pub trackInitLerp: bool,
	pub nextTrackEntUpdateTime: i32,
	pub shake_intensity: f32,
	pub shake_duration: c_int,
	pub shake_start: i32,
	pub sRoff: [c_char; 64],
	pub roff_frame: c_int,
	pub next_roff_time: i32,
	pub smooth_active: bool,
	pub smooth_origin: [f32; 3],
	pub smooth_intensity: f32,
	pub smooth_duration: c_int,
	pub smooth_start: i32,
	pub aimEntNum: c_int,
	#[cfg(target_os = "xbox")]
	pub widescreen: bool,
}

impl camera_t {
	const ZEROED: camera_t = camera_t {
		origin: [0.0; 3],
		angles: [0.0; 3],
		origin2: [0.0; 3],
		angles2: [0.0; 3],
		info_state: 0,
		FOV: 0.0,
		FOV2: 0.0,
		FOV_time: 0,
		FOV_duration: 0.0,
		FOV_vel: 0.0,
		FOV_acc: 0.0,
		bar_alpha: 0.0,
		bar_alpha_source: 0.0,
		bar_alpha_dest: 0.0,
		bar_height: 0.0,
		bar_height_source: 0.0,
		bar_height_dest: 0.0,
		bar_time: 0,
		fade_color: [0.0; 4],
		fade_source: [0.0; 4],
		fade_dest: [0.0; 4],
		fade_time: 0,
		fade_duration: 0.0,
		moveDir: [0.0; 3],
		move_time: 0,
		move_duration: 0.0,
		pan_time: 0,
		pan_duration: 0.0,
		followInitLerp: false,
		followSpeed: 0.0,
		cameraGroup: [0; 64],
		cameraGroupZOfs: 0.0,
		cameraGroupTag: [0; 64],
		subjectPos: [0.0; 3],
		subjectSpeed: 0.0,
		trackEntNum: 0,
		initSpeed: 0.0,
		speed: 0.0,
		distance: 0.0,
		distanceInitLerp: false,
		trackToOrg: [0.0; 3],
		trackInitLerp: false,
		nextTrackEntUpdateTime: 0,
		shake_intensity: 0.0,
		shake_duration: 0,
		shake_start: 0,
		sRoff: [0; 64],
		roff_frame: 0,
		next_roff_time: 0,
		smooth_active: false,
		smooth_origin: [0.0; 3],
		smooth_intensity: 0.0,
		smooth_duration: 0,
		smooth_start: 0,
		aimEntNum: 0,
		#[cfg(target_os = "xbox")]
		widescreen: false,
	};
}

// Stub types for external dependencies
#[repr(C)]
pub struct gentity_t {
	pub s: entityState_t,
	pub client: *mut gclient_t,
	pub currentOrigin: [f32; 3],
	pub target: *const c_char,
	pub targetname: *const c_char,
	pub radius: f32,
	pub speed: f32,
	pub spawnflags: c_int,
	pub cameraGroup: *const c_char,
	pub contents: c_int,
	pub playerModel: c_int,
	pub ghoul2: Ghoul2Vec,
}

#[repr(C)]
pub struct centity_t {
	pub gent: *mut gentity_t,
	pub lerpOrigin: [f32; 3],
	pub currentState: entityState_t,
}

#[repr(C)]
pub struct entityState_t {
	pub number: c_int,
	pub pos: trInfo_t,
	pub trType: c_int,
}

#[repr(C)]
pub struct trInfo_t {
	pub trType: c_int,
}

#[repr(C)]
pub struct gclient_t {
	pub ps: playerState_t,
	pub legsYaw: f32,
}

#[repr(C)]
pub struct playerState_t {
	pub velocity: [f32; 3],
	pub viewangles: [f32; 3],
	pub viewheight: f32,
	pub saberInFlight: u32,
	pub saber: [saber_t; 2],
	pub saberEntityNum: c_int,
	pub forcePowerDuration: [c_int; 20], // NUM_FORCE_POWERS stub
	pub forcePowersActive: c_int,
}

#[repr(C)]
pub struct saber_t {
	pub active: u32,
}

impl saber_t {
	fn Active(&self) -> bool {
		self.active != 0
	}
}

#[repr(C)]
pub struct cg_t {
	pub time: i32,
	pub frametime: i32,
	pub refdef: refdef_t,
	pub refdefViewAngles: [f32; 3],
	pub zoomMode: c_int,
}

#[repr(C)]
pub struct refdef_t {
	pub vieworg: [f32; 3],
	pub viewaxis: [[f32; 3]; 3],
	pub x: c_int,
	pub y: c_int,
}

#[repr(C)]
pub struct cgs_t {
	pub model_draw: *const c_void,
}

#[repr(C)]
pub struct game_import_t {
	_unused: c_int,
}

impl game_import_t {
	pub fn Printf(&self, fmt: *const c_char, _args: ...) {
		// Stub: Printf implementation would go here
	}

	pub fn SendServerCommand(&self, ent: *const c_void, cmd: *const c_char) {
		// Stub: SendServerCommand implementation would go here
	}

	pub fn cvar_set(&self, key: *const c_char, value: *const c_char) {
		// Stub: cvar_set implementation would go here
	}

	pub fn G2API_AddBolt(&self, ghoul2: *const Ghoul2Vec, tag: *const c_char) -> c_int {
		-1
	}

	pub fn G2API_GetBoltMatrix(&self, ghoul2: *const Ghoul2Vec, model: c_int, bolt: c_int, matrix: *mut mdxaBone_t, angles: [f32; 3], origin: [f32; 3], time: i32, model_draw: *const c_void, scale: f32) {
		// Stub
	}

	pub fn G2API_GiveMeVectorFromMatrix(&self, matrix: mdxaBone_t, mode: c_int, vec: *mut [f32; 3]) {
		// Stub
	}
}

pub struct Ghoul2Vec {
	_unused: c_int,
}

impl Ghoul2Vec {
	pub fn size(&self) -> usize {
		0
	}
}

#[repr(C)]
pub struct mdxaBone_t {
	pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct cvar_t {
	pub integer: c_int,
	pub value: f32,
}

#[repr(C)]
pub struct roff_list_t {
	pub data: *mut c_void,
	pub mFrameTime: i32,
	pub frames: c_int,
	pub mNoteTrackIndexes: *mut *const c_char,
	pub mStartNote: c_int,
	pub mNumNotes: c_int,
	pub typ: c_int,
}

impl roff_list_t {
	pub fn r#type(&self) -> c_int {
		self.typ
	}
}

#[repr(C)]
pub struct move_rotate_t {
	pub origin_delta: [f32; 3],
	pub rotate_delta: [f32; 3],
}

#[repr(C)]
pub struct move_rotate2_t {
	pub origin_delta: [f32; 3],
	pub rotate_delta: [f32; 3],
	pub mStartNote: c_int,
	pub mNumNotes: c_int,
}

// Constants (stubs from C defines)
const CAMERA_DEFAULT_FOV: f32 = 75.0;
const CAMERA_BAR_FADING: c_int = 1;
const CAMERA_MOVING: c_int = 2;
const CAMERA_PANNING: c_int = 4;
const CAMERA_FOLLOWING: c_int = 8;
const CAMERA_TRACKING: c_int = 16;
const CAMERA_ZOOMING: c_int = 32;
const CAMERA_FADING: c_int = 64;
const CAMERA_ROFFING: c_int = 128;
const CAMERA_ACCEL: c_int = 256;
const CAMERA_CUT: c_int = 512;
const CAMERA_SMOOTHING: c_int = 1024;
const ENTITYNUM_WORLD: c_int = 2047;
const NUM_FORCE_POWERS: usize = 20;
const MAX_CAMERA_GROUP_SUBJECTS: usize = 16;
const MAX_SHAKE_INTENSITY: f32 = 16.0;
const MAX_ACCEL_PER_FRAME: f32 = 10.0;
const BAR_DURATION: f32 = 200.0;
const FINAL_BUILD: bool = false;
const ORIGIN: c_int = 0;
const TR_STATIONARY: c_int = 0;
const TR_INTERPOLATE: c_int = 1;
const S_COLOR_RED: &str = "^1";
const S_COLOR_GREEN: &str = "^2";

// Macro stubs
fn VectorCopy(src: &[f32; 3], dst: &mut [f32; 3]) {
	dst[0] = src[0];
	dst[1] = src[1];
	dst[2] = src[2];
}

fn Vector4Copy(src: &[f32; 4], dst: &mut [f32; 4]) {
	dst[0] = src[0];
	dst[1] = src[1];
	dst[2] = src[2];
	dst[3] = src[3];
}

fn VectorClear(v: &mut [f32; 3]) {
	v[0] = 0.0;
	v[1] = 0.0;
	v[2] = 0.0;
}

fn VectorCompare(a: &[f32; 3], b: &[f32; 3]) -> bool {
	a[0] == b[0] && a[1] == b[1] && a[2] == b[2]
}

fn VectorAdd(a: &[f32; 3], b: &[f32; 3], out: &mut [f32; 3]) {
	out[0] = a[0] + b[0];
	out[1] = a[1] + b[1];
	out[2] = a[2] + b[2];
}

fn VectorSubtract(a: &[f32; 3], b: &[f32; 3], out: &mut [f32; 3]) {
	out[0] = a[0] - b[0];
	out[1] = a[1] - b[1];
	out[2] = a[2] - b[2];
}

fn VectorScale(v: &[f32; 3], scale: f32, out: &mut [f32; 3]) {
	out[0] = v[0] * scale;
	out[1] = v[1] * scale;
	out[2] = v[2] * scale;
}

fn VectorMA(v: &[f32; 3], scale: f32, add: &[f32; 3], out: &mut [f32; 3]) {
	out[0] = v[0] + add[0] * scale;
	out[1] = v[1] + add[1] * scale;
	out[2] = v[2] + add[2] * scale;
}

fn VectorNormalize(v: &mut [f32; 3]) -> f32 {
	let mut len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
	if len > 0.0 {
		let ilen = 1.0 / len;
		v[0] *= ilen;
		v[1] *= ilen;
		v[2] *= ilen;
	}
	len
}

fn VectorLengthSquared(v: &[f32; 3]) -> f32 {
	v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

fn DotProduct(a: &[f32; 3], b: &[f32; 3]) -> f32 {
	a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn AngleNormalize180(angle: f32) -> f32 {
	let mut a = ((angle + 180.0) % 360.0) - 180.0;
	if a < -180.0 {
		a += 360.0;
	}
	a
}

fn AngleNormalize360(angle: f32) -> f32 {
	((angle % 360.0) + 360.0) % 360.0
}

fn AngleDelta(a: f32, b: f32) -> f32 {
	let mut c = a - b;
	if c > 180.0 {
		c -= 360.0;
	} else if c < -180.0 {
		c += 360.0;
	}
	c
}

fn AnglesToAxis(angles: &[f32; 3], axis: &mut [[f32; 3]; 3]) {
	// Stub
}

fn vectoangles(vec: &[f32; 3], angles: &mut [f32; 3]) {
	// Stub
}

fn Q_stricmp(a: *const c_char, b: *const c_char) -> c_int {
	// Stub: should compare strings case-insensitively
	0
}

fn Q_fabs(f: f32) -> f32 {
	f.abs()
}

fn crandom() -> f32 {
	// Stub: random value between -1 and 1
	0.0
}

extern "C" {
	pub static mut qbVidRestartOccured: u32; // qboolean stub
	pub static vec3_origin: [f32; 3];
	pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
	pub fn strlen(s: *const c_char) -> usize;
	pub fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
	pub fn atof(s: *const c_char) -> f32;
	pub fn isdigit(c: c_int) -> c_int;
	pub fn isspace(c: c_int) -> c_int;
	pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
	pub fn fabs(f: f64) -> f64;
	pub fn Com_Printf(fmt: *const c_char, ...);
}

/*
TODO:
CloseUp, FullShot & Longshot commands:

  camera( CLOSEUP, <entity targetname>, angles(pitch yaw roll) )
  Will find the ent, apply angle offset to their head forward(minus pitch),
  get a preset distance away and set the FOV.  Trace to point, if less than
  1.0, put it there and open up FOV accordingly.
  Be sure to frame it so that eyespot and tag_head are positioned at proper
  places in the frame - ie: eyespot not in center, but not closer than 1/4
  screen width to the top...?
*/
/*
-------------------------
CGCam_Init
-------------------------
*/

pub fn CGCam_Init() {
	extern "C" {
		static mut qbVidRestartOccured: u32;
	}
	unsafe {
		if qbVidRestartOccured == 0 {
			let ptr = core::ptr::addr_of_mut!(client_camera) as *mut u8;
			memset(ptr, 0, core::mem::size_of::<camera_t>());
		}
	}
}

#[cfg(target_os = "xbox")]
pub fn CGCam_SetWidescreen(widescreen: bool) {
	unsafe {
		client_camera.widescreen = widescreen;
		cg.widescreen = widescreen;
	}
}

/*
-------------------------
CGCam_Enable
-------------------------
*/
pub fn CGCam_Enable() {
	unsafe {
		client_camera.bar_alpha = 0.0;
		client_camera.bar_time = cg.time;

		client_camera.bar_alpha_source = 0.0;
		client_camera.bar_alpha_dest = 1.0;

		client_camera.bar_height_source = 0.0;
		client_camera.bar_height_dest = 480.0/10.0;
		client_camera.bar_height = 0.0;

		client_camera.info_state |= CAMERA_BAR_FADING;

		client_camera.FOV = CAMERA_DEFAULT_FOV;
		client_camera.FOV2 = CAMERA_DEFAULT_FOV;

		in_camera = true;

		client_camera.next_roff_time = 0;

		if !g_entities.as_ptr().is_null() && !g_entities[0].client.is_null() {
			//Player zero not allowed to do anything
			VectorClear(&mut (*g_entities[0].client).ps.velocity);
			g_entities[0].contents = 0;

			if cg.zoomMode != 0 {
				// need to shut off some form of zooming
				cg.zoomMode = 0;
			}

			if (*g_entities[0].client).ps.saberInFlight != 0 && g_entities[0].client.as_ref().unwrap().ps.saber[0].Active() {
				//saber is out
				let saberent = &mut g_entities[(*g_entities[0].client).ps.saberEntityNum as usize];
				if !saberent as *const _ as *const c_void == core::ptr::null() {
					WP_SaberCatch(&mut g_entities[0], saberent, 0);
				}
			}

			for i in 0..NUM_FORCE_POWERS {
				//deactivate any active force powers
				(*g_entities[0].client).ps.forcePowerDuration[i] = 0;
				if (*g_entities[0].client).ps.forcePowerDuration[i] != 0 || ((*g_entities[0].client).ps.forcePowersActive & (1 << i)) != 0 {
					WP_ForcePowerStop(&mut g_entities[0], i as c_int);
				}
			}
		}
	}
}
/*
-------------------------
CGCam_Disable
-------------------------
*/

pub fn CGCam_Disable() {
	unsafe {
		in_camera = false;

		client_camera.bar_alpha = 1.0;
		client_camera.bar_time = cg.time;

		client_camera.bar_alpha_source = 1.0;
		client_camera.bar_alpha_dest = 0.0;

		client_camera.bar_height_source = 480.0/10.0;
		client_camera.bar_height_dest = 0.0;

		client_camera.info_state |= CAMERA_BAR_FADING;

		if !g_entities.as_ptr().is_null() && !g_entities[0].client.is_null() {
			g_entities[0].contents = 0x02;  //CONTENTS_BODY;//MASK_PLAYERSOLID;
		}

		gi.SendServerCommand(core::ptr::null(), b"cts\0".as_ptr() as *const c_char);

		//if ( cg_skippingcin.integer )
		{
			//We're skipping the cinematic and it's over now
			gi.cvar_set(b"timescale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);
			gi.cvar_set(b"skippingCinematic\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
		}

		//we just came out of camera, so update cg.refdef.vieworg out of the camera's origin so the snapshot will know our new ori
		VectorCopy(&g_entities[0].currentOrigin, &mut cg.refdef.vieworg);
		VectorCopy(&(*g_entities[0].client).ps.viewangles, &mut cg.refdefViewAngles);
	}
}

/*
-------------------------
CGCam_SetPosition
-------------------------
*/

pub fn CGCam_SetPosition(org: &[f32; 3]) {
	unsafe {
		VectorCopy(org, &mut client_camera.origin);
		VectorCopy(&client_camera.origin, &mut cg.refdef.vieworg);
	}
}

/*
-------------------------
CGCam_Move
-------------------------
*/

pub fn CGCam_Move(dest: &[f32; 3], duration: f32) {
	unsafe {
		if (client_camera.info_state & CAMERA_ROFFING) != 0 {
			client_camera.info_state &= !CAMERA_ROFFING;
		}

		CGCam_TrackDisable();
		CGCam_DistanceDisable();

		if duration == 0.0 {
			client_camera.info_state &= !CAMERA_MOVING;
			CGCam_SetPosition(dest);
			return;
		}

		client_camera.info_state |= CAMERA_MOVING;

		VectorCopy(dest, &mut client_camera.origin2);

		client_camera.move_duration = duration;
		client_camera.move_time = cg.time;
	}
}

/*
-------------------------
CGCam_SetAngles
-------------------------
*/

pub fn CGCam_SetAngles(ang: &[f32; 3]) {
	unsafe {
		VectorCopy(ang, &mut client_camera.angles);
		VectorCopy(&client_camera.angles, &mut cg.refdefViewAngles);
	}
}

/*
-------------------------
CGCam_Pan
-------------------------
*/

pub fn CGCam_Pan(dest: &[f32; 3], panDirection: &[f32; 3], duration: f32) {
	//vec3_t	panDirection = {0, 0, 0};
	unsafe {
		let mut i: usize;
		let mut delta1: f32;
		let mut delta2: f32;

		CGCam_FollowDisable();
		CGCam_DistanceDisable();

		if duration == 0.0 {
			CGCam_SetAngles(dest);
			client_camera.info_state &= !CAMERA_PANNING;
			return;
		}

		//FIXME: make the dest an absolute value, and pass in a
		//panDirection as well.  If a panDirection's axis value is
		//zero, find the shortest difference for that axis.
		//Store the delta in client_camera.angles2.
		for i in 0..3 {
			let dest_norm = AngleNormalize360(dest[i]);
			delta1 = dest_norm - AngleNormalize360(client_camera.angles[i]);
			if delta1 < 0.0 {
				delta2 = delta1 + 360.0;
			} else {
				delta2 = delta1 - 360.0;
			}
			if panDirection[i] == 0.0 {
				//Didn't specify a direction, pick shortest
				if Q_fabs(delta1) < Q_fabs(delta2) {
					client_camera.angles2[i] = delta1;
				} else {
					client_camera.angles2[i] = delta2;
				}
			} else if panDirection[i] < 0.0 {
				if delta1 < 0.0 {
					client_camera.angles2[i] = delta1;
				} else if delta1 > 0.0 {
					client_camera.angles2[i] = delta2;
				} else {
					//exact
					client_camera.angles2[i] = 0.0;
				}
			} else if panDirection[i] > 0.0 {
				if delta1 > 0.0 {
					client_camera.angles2[i] = delta1;
				} else if delta1 < 0.0 {
					client_camera.angles2[i] = delta2;
				} else {
					//exact
					client_camera.angles2[i] = 0.0;
				}
			}
		}
		//VectorCopy( dest, client_camera.angles2 );

		client_camera.info_state |= CAMERA_PANNING;

		client_camera.pan_duration = duration;
		client_camera.pan_time = cg.time;
	}
}

/*
-------------------------
CGCam_SetRoll
-------------------------
*/

pub fn CGCam_SetRoll(roll: f32) {
	unsafe {
		client_camera.angles[2] = roll;
	}
}

/*
-------------------------
CGCam_Roll
-------------------------
*/

pub fn CGCam_Roll(dest: f32, duration: f32) {
	unsafe {
		if duration == 0.0 {
			CGCam_SetRoll(dest);
			return;
		}

		//FIXME/NOTE: this will override current panning!!!
		client_camera.info_state |= CAMERA_PANNING;

		VectorCopy(&client_camera.angles, &mut client_camera.angles2);
		client_camera.angles2[2] = AngleDelta(dest, client_camera.angles[2]);

		client_camera.pan_duration = duration;
		client_camera.pan_time = cg.time;
	}
}

/*
-------------------------
CGCam_SetFOV
-------------------------
*/

pub fn CGCam_SetFOV(FOV: f32) {
	unsafe {
		client_camera.FOV = FOV;
	}
}

/*
-------------------------
CGCam_Zoom
-------------------------
*/

pub fn CGCam_Zoom(FOV: f32, duration: f32) {
	unsafe {
		if duration == 0.0 {
			CGCam_SetFOV(FOV);
			return;
		}
		client_camera.info_state |= CAMERA_ZOOMING;

		client_camera.FOV_time = cg.time;
		client_camera.FOV2 = FOV;

		client_camera.FOV_duration = duration;
	}
}

pub fn CGCam_Zoom2(FOV: f32, FOV2: f32, duration: f32) {
	unsafe {
		if duration == 0.0 {
			CGCam_SetFOV(FOV2);
			return;
		}
		client_camera.info_state |= CAMERA_ZOOMING;

		client_camera.FOV_time = cg.time;
		client_camera.FOV = FOV;
		client_camera.FOV2 = FOV2;

		client_camera.FOV_duration = duration;
	}
}

pub fn CGCam_ZoomAccel(initialFOV: f32, fovVelocity: f32, fovAccel: f32, duration: f32) {
	unsafe {
		if duration == 0.0 {
			return;
		}
		client_camera.info_state |= CAMERA_ACCEL;

		client_camera.FOV_time = cg.time;
		client_camera.FOV2 = initialFOV;
		client_camera.FOV_vel = fovVelocity;
		client_camera.FOV_acc = fovAccel;

		client_camera.FOV_duration = duration;
	}
}

/*
-------------------------
CGCam_Fade
-------------------------
*/

pub fn CGCam_SetFade(dest: &[f32; 4]) {
	//Instant completion
	unsafe {
		client_camera.info_state &= !CAMERA_FADING;
		client_camera.fade_duration = 0.0;
		Vector4Copy(dest, &mut client_camera.fade_source);
		Vector4Copy(dest, &mut client_camera.fade_color);
	}
}

/*
-------------------------
CGCam_Fade
-------------------------
*/

pub fn CGCam_Fade(source: &[f32; 4], dest: &[f32; 4], duration: f32) {
	unsafe {
		if duration == 0.0 {
			CGCam_SetFade(dest);
			return;
		}

		Vector4Copy(source, &mut client_camera.fade_source);
		Vector4Copy(dest, &mut client_camera.fade_dest);

		client_camera.fade_duration = duration;
		client_camera.fade_time = cg.time;

		client_camera.info_state |= CAMERA_FADING;
	}
}

pub fn CGCam_FollowDisable() {
	unsafe {
		client_camera.info_state &= !CAMERA_FOLLOWING;
		client_camera.cameraGroup[0] = 0;
		client_camera.cameraGroupZOfs = 0.0;
		client_camera.cameraGroupTag[0] = 0;
	}
}

pub fn CGCam_TrackDisable() {
	unsafe {
		client_camera.info_state &= !CAMERA_TRACKING;
		client_camera.trackEntNum = ENTITYNUM_WORLD;
	}
}

pub fn CGCam_DistanceDisable() {
	unsafe {
		client_camera.distance = 0.0;
	}
}
/*
-------------------------
CGCam_Follow
-------------------------
*/

pub fn CGCam_Follow(cameraGroup: *const c_char, speed: f32, initLerp: f32) {
	unsafe {
		let mut len: usize;

		//Clear any previous
		CGCam_FollowDisable();

		if cameraGroup.is_null() || *cameraGroup as u8 == 0 {
			return;
		}

		if Q_stricmp(b"none\0".as_ptr() as *const c_char, cameraGroup as *mut c_char) == 0 {
			//Turn off all aiming
			return;
		}

		if Q_stricmp(b"NULL\0".as_ptr() as *const c_char, cameraGroup as *mut c_char) == 0 {
			//Turn off all aiming
			return;
		}

		//NOTE: if this interrupts a pan before it's done, need to copy the cg.refdef.viewAngles to the camera.angles!
		client_camera.info_state |= CAMERA_FOLLOWING;
		client_camera.info_state &= !CAMERA_PANNING;

		len = strlen(cameraGroup);
		strncpy(client_camera.cameraGroup.as_mut_ptr(), cameraGroup, client_camera.cameraGroup.len());
		//NULL terminate last char in case they type a name too long
		client_camera.cameraGroup[len] = 0;

		if speed != 0.0 {
			client_camera.followSpeed = speed;
		} else {
			client_camera.followSpeed = 100.0;
		}

		if initLerp != 0.0 {
			client_camera.followInitLerp = true;
		} else {
			client_camera.followInitLerp = false;
		}
	}
}

/*
-------------------------
Q3_CameraAutoAim

  Keeps camera pointed at an entity, usually will be a misc_camera_focus
  misc_camera_focus can be on a track that stays closest to it's subjects on that
  path (like Q3_CameraAutoTrack) or is can simply always put itself between it's subjects.
  misc_camera_focus can also set FOV/camera dist needed to keep the subjects in frame
-------------------------
*/

pub fn CG_CameraAutoAim(_name: *const c_char) {
	/*
	gentity_t *aimEnt = NULL;

	//Clear any previous
	CGCam_FollowDisable();

	if(Q_stricmp("none", (char *)name) == 0)
	{//Turn off all aiming
		return;
	}

	aimEnt = G_Find(NULL, FOFS(targetname), (char *)name);

	if(!aimEnt)
	{
		gi.Printf(S_COLOR_RED"ERROR: %s camera aim target not found\n", name);
		return;
	}

	//Lerp time...
	//aimEnt->aimDebounceTime = level.time;//FIXME: over time
	client_camera.aimEntNum = aimEnt->s.number;
	CGCam_Follow( aimEnt->cameraGroup, aimEnt->speed, aimEnt->spawnflags&1 );
	*/
}

/*
-------------------------
CGCam_Track
-------------------------
*/
//void CGCam_Track( char *trackName, float speed, float duration )
pub fn CGCam_Track(trackName: *const c_char, speed: f32, initLerp: f32) {
	unsafe {
		let mut trackEnt: *mut gentity_t = core::ptr::null_mut();

		CGCam_TrackDisable();

		if Q_stricmp(b"none\0".as_ptr() as *const c_char, trackName as *mut c_char) == 0 {
			//turn off tracking
			return;
		}

		//NOTE: if this interrupts a move before it's done, need to copy the cg.refdef.vieworg to the camera.origin!
		//This will find a path_corner now, not a misc_camera_track
		trackEnt = G_Find(core::ptr::null_mut(), 0, trackName);

		if trackEnt.is_null() {
			gi.Printf(S_COLOR_RED.as_ptr() as *const c_char, b"ERROR: %s camera track target not found\n\0".as_ptr() as *const c_char, trackName);
			return;
		}

		client_camera.info_state |= CAMERA_TRACKING;
		client_camera.info_state &= !CAMERA_MOVING;

		client_camera.trackEntNum = (*trackEnt).s.number;
		client_camera.initSpeed = speed/10.0;
		client_camera.speed = speed;
		client_camera.nextTrackEntUpdateTime = cg.time;

		if initLerp != 0.0 {
			client_camera.trackInitLerp = true;
		} else {
			client_camera.trackInitLerp = false;
		}
		/*
		if ( client_camera.info_state & CAMERA_FOLLOWING )
		{//Used to snap angles?  Do what...?
		}
		*/

		//Set a moveDir
		VectorSubtract(&(*trackEnt).currentOrigin, &client_camera.origin, &mut client_camera.moveDir);

		if !client_camera.trackInitLerp {
			//want to snap to first position
			//Snap to trackEnt's origin
			VectorCopy(&(*trackEnt).currentOrigin, &mut client_camera.origin);

			//Set new moveDir if trackEnt has a next path_corner
			//Possible that track has no next point, in which case we won't be moving anyway
			if !(*trackEnt).target.is_null() && *(*trackEnt).target as u8 != 0 {
				let newTrackEnt = G_Find(core::ptr::null_mut(), 0, (*trackEnt).target);
				if !newTrackEnt.is_null() {
					VectorSubtract(&(*newTrackEnt).currentOrigin, &client_camera.origin, &mut client_camera.moveDir);
				}
			}
		}

		VectorNormalize(&mut client_camera.moveDir);
	}
}

/*
-------------------------
Q3_CameraAutoTrack

  Keeps camera a certain distance from target entity but on the specified CameraPath
  The distance can be set in a script or derived from a misc_camera_focus.
  Dist will interpolate when changed, can also set acceleration/deceleration values.
  FOV will also interpolate.

  CameraPath might be a MAX path or perhaps a series of path_corners on the map itself
-------------------------
*/

pub fn CG_CameraAutoTrack(_name: *const c_char) {
	/*
	gentity_t *trackEnt = NULL;

	CGCam_TrackDisable();

	if(Q_stricmp("none", (char *)name) == 0)
	{//turn off tracking
		return;
	}

	//This will find a path_corner now, not a misc_camera_track
	trackEnt = G_Find(NULL, FOFS(targetname), (char *)name);

	if(!trackEnt)
	{
		gi.Printf(S_COLOR_RED"ERROR: %s camera track target not found\n", name);
		return;
	}

	//FIXME: last arg will be passed in
	CGCam_Track( trackEnt->s.number, trackEnt->speed, qfalse );
	//FIXME: this will be a seperate call
	CGCam_Distance( trackEnt->radius, qtrue);
	*/
}

/*
-------------------------
CGCam_Distance
-------------------------
*/

pub fn CGCam_Distance(distance: f32, initLerp: f32) {
	unsafe {
		client_camera.distance = distance;

		if initLerp != 0.0 {
			client_camera.distanceInitLerp = true;
		} else {
			client_camera.distanceInitLerp = false;
		}
	}
}

//========================================================================================


pub fn CGCam_FollowUpdate() {
	unsafe {
		let mut center: [f32; 3] = [0.0; 3];
		let mut dir: [f32; 3] = [0.0; 3];
		let mut cameraAngles: [f32; 3] = [0.0; 3];
		let mut vec: [f32; 3] = [0.0; 3];
		let mut focus: [[f32; 3]; MAX_CAMERA_GROUP_SUBJECTS] = [[0.0; 3]; MAX_CAMERA_GROUP_SUBJECTS];
		let mut from: *mut gentity_t = core::ptr::null_mut();
		let mut fromCent: *mut centity_t;
		let mut num_subjects: usize = 0;
		let mut i: usize;
		let mut focused: bool = false;

		if !client_camera.cameraGroup.as_ptr().is_null() && client_camera.cameraGroup[0] != 0 {
			//Stay centered in my cameraGroup, if I have one
			loop {
				from = G_Find(from, 0, client_camera.cameraGroup.as_ptr());
				if from.is_null() {
					break;
				}
				/*
				if ( from->s.number == client_camera.aimEntNum )
				{//This is the misc_camera_focus, we'll be removing this ent altogether eventually
					continue;
				}
				*/

				if num_subjects >= MAX_CAMERA_GROUP_SUBJECTS {
					gi.Printf(S_COLOR_RED.as_ptr() as *const c_char, b"ERROR: Too many subjects in shot composition %s\0".as_ptr() as *const c_char, client_camera.cameraGroup.as_ptr());
					break;
				}

				fromCent = cg_entities.as_mut_ptr().offset((*from).s.number as isize);
				if fromCent.is_null() {
					continue;
				}

				focused = false;
				if !(*from).client.is_null() && !client_camera.cameraGroupTag.as_ptr().is_null() && client_camera.cameraGroupTag[0] != 0 && !(*(*from).client).ps.viewheight.is_nan() {
					let newBolt = gi.G2API_AddBolt(&(*(*fromCent).gent).ghoul2, client_camera.cameraGroupTag.as_ptr());
					if newBolt != -1 {
						let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
						let mut fromAngles: [f32; 3] = [0.0, (*(*from).client).legsYaw, 0.0];

						gi.G2API_GetBoltMatrix(&(*(*fromCent).gent).ghoul2, (*from).playerModel, newBolt, &mut boltMatrix, fromAngles, (*fromCent).lerpOrigin, cg.time, cgs.model_draw, 1.0);
						gi.G2API_GiveMeVectorFromMatrix(boltMatrix, ORIGIN, &mut focus[num_subjects]);

						focused = true;
					}
				}
				if !focused {
					if (*from).s.pos.trType != TR_STATIONARY {
						//if ( from->s.pos.trType == TR_INTERPOLATE )
						{
							//use interpolated origin?
							if !VectorCompare(&vec3_origin, &(*fromCent).lerpOrigin) {
								//hunh?  Somehow we've never seen this gentity on the client, so there is no lerpOrigin, so cheat over to the game and use the currentOrigin
								VectorCopy(&(*from).currentOrigin, &mut focus[num_subjects]);
							} else {
								VectorCopy(&(*fromCent).lerpOrigin, &mut focus[num_subjects]);
							}
						}
					} else {
						VectorCopy(&(*from).currentOrigin, &mut focus[num_subjects]);
					}
					//FIXME: make a list here of their s.numbers instead so we can do other stuff with the list below
					if !(*from).client.is_null() {
						//Track to their eyes - FIXME: maybe go off a tag?
						//FIXME:
						//Based on FOV and distance to subject from camera, pick the point that
						//keeps eyes 3/4 up from bottom of screen... what about bars?
						focus[num_subjects][2] += (*(*from).client).ps.viewheight;
					}
				}
				if client_camera.cameraGroupZOfs != 0.0 {
					focus[num_subjects][2] += client_camera.cameraGroupZOfs;
				}
				num_subjects += 1;
			}

			if num_subjects == 0 {
				// Bad cameragroup
				if !FINAL_BUILD {
					gi.Printf(S_COLOR_RED.as_ptr() as *const c_char, b"ERROR: Camera Focus unable to locate cameragroup: %s\n\0".as_ptr() as *const c_char, client_camera.cameraGroup.as_ptr());
				}
				return;
			}

			//Now average all points
			VectorCopy(&focus[0], &mut center);
			for i in 1..num_subjects {
				VectorAdd(&focus[i], &center, &mut center);
			}
			VectorScale(&center, 1.0/((num_subjects as f32)), &mut center);
		} else {
			return;
		}

		//Need to set a speed to keep a distance from
		//the subject- fixme: only do this if have a distance
		//set
		VectorSubtract(&client_camera.subjectPos, &center, &mut vec);
		client_camera.subjectSpeed = VectorLengthSquared(&vec) * 100.0 / (cg.frametime as f32);

		/*
		if ( !cg_skippingcin.integer )
		{
			Com_Printf( S_COLOR_RED"org: %s\n", vtos(center) );
		}
		*/
		VectorCopy(&center, &mut client_camera.subjectPos);

		VectorSubtract(&center, &cg.refdef.vieworg, &mut dir);//can't use client_camera.origin because it's not updated until the end of the move.

		//Get desired angle
		vectoangles(&dir, &mut cameraAngles);

		if client_camera.followInitLerp {
			//Lerping
			let frac = (cg.frametime as f32)/100.0 * client_camera.followSpeed/100.0;
			for i in 0..3 {
				cameraAngles[i] = AngleNormalize180(cameraAngles[i]);
				cameraAngles[i] = AngleNormalize180(client_camera.angles[i] + frac * AngleNormalize180(cameraAngles[i] - client_camera.angles[i]));
				cameraAngles[i] = AngleNormalize180(cameraAngles[i]);
			}
#if 0
			Com_Printf( "%s\n", vtos(cameraAngles) );
#endif
		} else {
			//Snapping, should do this first time if follow_lerp_to_start_duration is zero
			//will lerp from this point on
			client_camera.followInitLerp = true;
			for i in 0..3 {
				//normalize so that when we start lerping, it doesn't freak out
				cameraAngles[i] = AngleNormalize180(cameraAngles[i]);
			}
			//So tracker doesn't move right away thinking the first angle change
			//is the subject moving... FIXME: shouldn't set this until lerp done OR snapped?
			client_camera.subjectSpeed = 0.0;
		}

		//Point camera to lerp angles
		/*
		if ( !cg_skippingcin.integer )
		{
			Com_Printf( "ang: %s\n", vtos(cameraAngles) );
		}
		*/
		VectorCopy(&cameraAngles, &mut client_camera.angles);
	}
}

pub fn CGCam_TrackEntUpdate() {
	//FIXME: only do every 100 ms
	unsafe {
		let mut trackEnt: *mut gentity_t = core::ptr::null_mut();
		let mut newTrackEnt: *mut gentity_t = core::ptr::null_mut();
		let mut reached: bool = false;
		let mut vec: [f32; 3] = [0.0; 3];
		let mut dist: f32;

		if client_camera.trackEntNum >= 0 && client_camera.trackEntNum < ENTITYNUM_WORLD {
			//We're already heading to a path_corner
			trackEnt = &mut g_entities[client_camera.trackEntNum as usize];
			VectorSubtract(&(*trackEnt).currentOrigin, &client_camera.origin, &mut vec);
			dist = VectorLengthSquared(&vec);
			if dist < 256.0 {
				//16 squared
				//FIXME: who should be doing the using here?
				G_UseTargets(trackEnt, trackEnt);
				reached = true;
			}
		}

		if !trackEnt.is_null() && reached {

			if !(*trackEnt).target.is_null() && *(*trackEnt).target as u8 != 0 {
				//Find our next path_corner
				newTrackEnt = G_Find(core::ptr::null_mut(), 0, (*trackEnt).target);
				if !newTrackEnt.is_null() {
					if (*newTrackEnt).radius < 0.0 {
						//Don't bother trying to maintain a radius
						client_camera.distance = 0.0;
						client_camera.speed = client_camera.initSpeed;
					} else if (*newTrackEnt).radius > 0.0 {
						client_camera.distance = (*newTrackEnt).radius;
					}

					if (*newTrackEnt).speed < 0.0 {
						//go back to our default speed
						client_camera.speed = client_camera.initSpeed;
					} else if (*newTrackEnt).speed > 0.0 {
						client_camera.speed = (*newTrackEnt).speed/10.0;
					}
				}
			} else {
				//stop thinking if this is the last one
				CGCam_TrackDisable();
			}
		}

		if !newTrackEnt.is_null() {
			//Update will lerp this
			client_camera.info_state |= CAMERA_TRACKING;
			client_camera.trackEntNum = (*newTrackEnt).s.number;
			VectorCopy(&(*newTrackEnt).currentOrigin, &mut client_camera.trackToOrg);
		}

		client_camera.nextTrackEntUpdateTime = cg.time + 100;
	}
}

pub fn CGCam_TrackUpdate() {
	unsafe {
		let mut goalVec: [f32; 3] = [0.0; 3];
		let mut curVec: [f32; 3] = [0.0; 3];
		let mut trackPos: [f32; 3] = [0.0; 3];
		let mut vec: [f32; 3] = [0.0; 3];
		let mut goalDist: f32;
		let mut dist: f32;
		let mut slowDown: bool = false;

		if client_camera.nextTrackEntUpdateTime <= cg.time {
			CGCam_TrackEntUpdate();
		}

		VectorSubtract(&client_camera.trackToOrg, &client_camera.origin, &mut goalVec);
		goalDist = VectorNormalize(&mut goalVec);
		if goalDist > 100.0 {
			goalDist = 100.0;
		} else if goalDist < 10.0 {
			goalDist = 10.0;
		}

		if client_camera.distance != 0.0 && (client_camera.info_state & CAMERA_FOLLOWING) != 0 {
			let mut adjust: f32 = 0.0;
			let mut desiredSpeed: f32 = 0.0;
			let mut dot: f32;

			if !client_camera.distanceInitLerp {
				VectorSubtract(&client_camera.origin, &client_camera.subjectPos, &mut vec);
				VectorNormalize(&mut vec);
				//FIXME: use client_camera.moveDir here?
				VectorMA(&client_camera.subjectPos, client_camera.distance, &vec, &mut client_camera.origin);
				//Snap to first time only
				client_camera.distanceInitLerp = true;
				return;
			} else if client_camera.subjectSpeed > 0.05 {
				//Don't start moving until subject moves
				VectorSubtract(&client_camera.subjectPos, &client_camera.origin, &mut vec);
				dist = VectorNormalize(&mut vec);
				dot = DotProduct(&goalVec, &vec);

				if dist > client_camera.distance {
					//too far away
					if dot > 0.0 {
						//Camera is moving toward the subject
						adjust = (dist - client_camera.distance);//Speed up
					} else if dot < 0.0 {
						//Camera is moving away from the subject
						adjust = (dist - client_camera.distance) * -1.0;//Slow down
					}
				} else if dist < client_camera.distance {
					//too close
					if dot > 0.0 {
						//Camera is moving toward the subject
						adjust = (client_camera.distance - dist) * -1.0;//Slow down
					} else if dot < 0.0 {
						//Camera is moving away from the subject
						adjust = (client_camera.distance - dist);//Speed up
					}
				}

				//Speed of the focus + our error
				//desiredSpeed = aimCent->gent->speed + (adjust * cg.frametime/100.0f);//cg.frameInterpolation);
				desiredSpeed = (adjust);// * cg.frametime/100.0f);//cg.frameInterpolation);

				//self->moveInfo.speed = desiredSpeed;

				//Don't change speeds faster than 10 every 10th of a second
				let max_allowed_accel = MAX_ACCEL_PER_FRAME * ((cg.frametime as f32)/100.0);

				if client_camera.subjectSpeed == 0.0 {
					//full stop
					client_camera.speed = desiredSpeed;
				} else if client_camera.speed - desiredSpeed > max_allowed_accel {
					//new speed much slower, slow down at max accel
					client_camera.speed -= max_allowed_accel;
				} else if desiredSpeed - client_camera.speed > max_allowed_accel {
					//new speed much faster, speed up at max accel
					client_camera.speed += max_allowed_accel;
				} else {
					//remember this speed
					client_camera.speed = desiredSpeed;
				}

				//Com_Printf("Speed: %4.2f (%4.2f)\n", self->moveInfo.speed, aimCent->gent->speed);
			}
		} else {
			//slowDown = qtrue;
		}


		//FIXME: this probably isn't right, round it out more
		VectorScale(&goalVec, (cg.frametime as f32)/100.0, &mut goalVec);
		VectorScale(&client_camera.moveDir, (100.0 - (cg.frametime as f32))/100.0, &mut curVec);
		VectorAdd(&goalVec, &curVec, &mut client_camera.moveDir);
		VectorNormalize(&mut client_camera.moveDir);
		if slowDown {
			VectorMA(&client_camera.origin, client_camera.speed * goalDist/100.0 * (cg.frametime as f32)/100.0, &client_camera.moveDir, &mut trackPos);
		} else {
			VectorMA(&client_camera.origin, client_camera.speed * (cg.frametime as f32)/100.0, &client_camera.moveDir, &mut trackPos);
		}

		//FIXME: Implement
		//Need to find point on camera's path that is closest to the desired distance from subject
		//OR: Need to intelligently pick this desired distance based on framing...
		VectorCopy(&trackPos, &mut client_camera.origin);
	}
}

//=========================================================================================

/*
-------------------------
CGCam_UpdateBarFade
-------------------------
*/

pub fn CGCam_UpdateBarFade() {
	unsafe {
		if client_camera.bar_time + (BAR_DURATION as i32) < cg.time {
			client_camera.bar_alpha = client_camera.bar_alpha_dest;
			client_camera.info_state &= !CAMERA_BAR_FADING;
			client_camera.bar_height = client_camera.bar_height_dest;
		} else {
			client_camera.bar_alpha = client_camera.bar_alpha_source + ((client_camera.bar_alpha_dest - client_camera.bar_alpha_source) / BAR_DURATION) * ((cg.time - client_camera.bar_time) as f32);
			client_camera.bar_height = client_camera.bar_height_source + ((client_camera.bar_height_dest - client_camera.bar_height_source) / BAR_DURATION) * ((cg.time - client_camera.bar_time) as f32);
		}
	}
}

/*
-------------------------
CGCam_UpdateFade
-------------------------
*/

pub fn CGCam_UpdateFade() {
	unsafe {
		if (client_camera.info_state & CAMERA_FADING) != 0 {
			if client_camera.fade_time + (client_camera.fade_duration as i32) < cg.time {
				Vector4Copy(&client_camera.fade_dest, &mut client_camera.fade_color);
				client_camera.info_state &= !CAMERA_FADING;
			} else {
				for i in 0..4 {
					client_camera.fade_color[i] = client_camera.fade_source[i] + (((client_camera.fade_dest[i] - client_camera.fade_source[i])) / client_camera.fade_duration) * ((cg.time - client_camera.fade_time) as f32);
				}
			}
		}
	}
}
/*
-------------------------
CGCam_Update
-------------------------
*/
fn CGCam_Roff();

pub fn CGCam_Update() {
	unsafe {
		let mut i: usize;
		let mut checkFollow: bool = false;
		let mut checkTrack: bool = false;

		// Apply new roff data to the camera as needed
		if (client_camera.info_state & CAMERA_ROFFING) != 0 {
			CGCam_Roff();
		}

		//Check for a zoom
		if (client_camera.info_state & CAMERA_ACCEL) != 0 {
			// x = x0 + vt + 0.5*a*t*t
			let mut actualFOV_X = client_camera.FOV;
			let sanityMin = 1.0;
			let sanityMax = 180.0;
			let t = ((cg.time - client_camera.FOV_time) as f32)*0.001; // mult by 0.001 cuz otherwise t is too darned big
			let mut fovDuration = client_camera.FOV_duration;

			if !FINAL_BUILD {
				if cg_roffval4.integer != 0 {
					fovDuration = cg_roffval4.integer as f32;
				}
			}
			if client_camera.FOV_time + (fovDuration as i32) < cg.time {
				client_camera.info_state &= !CAMERA_ACCEL;
			} else {
				let mut initialPosVal = client_camera.FOV2;
				let mut velVal = client_camera.FOV_vel;
				let mut accVal = client_camera.FOV_acc;

				if !FINAL_BUILD {
					if cg_roffdebug.integer != 0 {
						if fabs(cg_roffval1.value as f64) > 0.001 {
							initialPosVal = cg_roffval1.value;
						}
						if fabs(cg_roffval2.value as f64) > 0.001 {
							velVal = cg_roffval2.value;
						}
						if fabs(cg_roffval3.value as f64) > 0.001 {
							accVal = cg_roffval3.value;
						}
					}
				}
				let initialPos = initialPosVal;
				let vel = velVal*t;
				let acc = 0.5*accVal*t*t;

				actualFOV_X = initialPos + vel + acc;
				if cg_roffdebug.integer != 0 {
					Com_Printf(b"frame: %d o:<%.2f %.2f %.2f> a:<%.2f %.2f %.2f>\n\0".as_ptr() as *const c_char,
						cg.time, initialPosVal, velVal, accVal, actualFOV_X);
				}

				if actualFOV_X < sanityMin {
					actualFOV_X = sanityMin;
				} else if actualFOV_X > sanityMax {
					actualFOV_X = sanityMax;
				}
				client_camera.FOV = actualFOV_X;
			}
			CG_CalcFOVFromX(actualFOV_X);
		} else if (client_camera.info_state & CAMERA_ZOOMING) != 0 {
			let actualFOV_X: f32;

			if client_camera.FOV_time + (client_camera.FOV_duration as i32) < cg.time {
				actualFOV_X = client_camera.FOV2;
				client_camera.FOV = client_camera.FOV2;
				client_camera.info_state &= !CAMERA_ZOOMING;
			} else {
				actualFOV_X = client_camera.FOV + (((client_camera.FOV2 - client_camera.FOV)) / client_camera.FOV_duration) * ((cg.time - client_camera.FOV_time) as f32);
			}
			CG_CalcFOVFromX(actualFOV_X);
		} else {
			CG_CalcFOVFromX(client_camera.FOV);
		}

		//Check for roffing angles
		if ((client_camera.info_state & CAMERA_ROFFING) != 0) && ((client_camera.info_state & CAMERA_FOLLOWING) == 0) {
			if (client_camera.info_state & CAMERA_CUT) != 0 {
				// we're doing a cut, so just go to the new angles. none of this hifalutin lerping business.
				for i in 0..3 {
					cg.refdefViewAngles[i] = AngleNormalize360((client_camera.angles[i] + client_camera.angles2[i]));
				}
			} else {
				for i in 0..3 {
					cg.refdefViewAngles[i] = client_camera.angles[i] + (client_camera.angles2[i] / client_camera.pan_duration) * ((cg.time - client_camera.pan_time) as f32);
				}
			}
		} else if (client_camera.info_state & CAMERA_PANNING) != 0 {
			if (client_camera.info_state & CAMERA_CUT) != 0 {
				// we're doing a cut, so just go to the new angles. none of this hifalutin lerping business.
				for i in 0..3 {
					cg.refdefViewAngles[i] = AngleNormalize360((client_camera.angles[i] + client_camera.angles2[i]));
				}
			} else {
				//Note: does not actually change the camera's angles until the pan time is done!
				if client_camera.pan_time + (client_camera.pan_duration as i32) < cg.time {
					//finished panning
					for i in 0..3 {
						client_camera.angles[i] = AngleNormalize360((client_camera.angles[i] + client_camera.angles2[i]));
					}

					client_camera.info_state &= !CAMERA_PANNING;
					VectorCopy(&client_camera.angles, &mut cg.refdefViewAngles);
				} else {
					//still panning
					for i in 0..3 {
						//NOTE: does not store the resultant angle in client_camera.angles until pan is done
						cg.refdefViewAngles[i] = client_camera.angles[i] + (client_camera.angles2[i] / client_camera.pan_duration) * ((cg.time - client_camera.pan_time) as f32);
					}
				}
			}
		} else {
			checkFollow = true;
		}

		//Check for movement
		if (client_camera.info_state & CAMERA_MOVING) != 0 {
			//NOTE: does not actually move the camera until the movement time is done!
			if client_camera.move_time + (client_camera.move_duration as i32) < cg.time {
				VectorCopy(&client_camera.origin2, &mut client_camera.origin);
				client_camera.info_state &= !CAMERA_MOVING;
				VectorCopy(&client_camera.origin, &mut cg.refdef.vieworg);
			} else {
				if (client_camera.info_state & CAMERA_CUT) != 0 {
					// we're doing a cut, so just go to the new origin. none of this fancypants lerping stuff.
					for i in 0..3 {
						cg.refdef.vieworg[i] = client_camera.origin2[i];
					}
				} else {
					for i in 0..3 {
						cg.refdef.vieworg[i] = client_camera.origin[i] + (((client_camera.origin2[i] - client_camera.origin[i])) / client_camera.move_duration) * ((cg.time - client_camera.move_time) as f32);
					}
				}
			}
		} else {
			checkTrack = true;
		}

		if checkFollow {
			if (client_camera.info_state & CAMERA_FOLLOWING) != 0 {
				//This needs to be done after camera movement
				CGCam_FollowUpdate();
			}
			VectorCopy(&client_camera.angles, &mut cg.refdefViewAngles);
		}

		if checkTrack {
			if (client_camera.info_state & CAMERA_TRACKING) != 0 {
				//This has to run AFTER Follow if the camera is following a cameraGroup
				CGCam_TrackUpdate();
			}

			VectorCopy(&client_camera.origin, &mut cg.refdef.vieworg);
		}

		//Bar fading
		if (client_camera.info_state & CAMERA_BAR_FADING) != 0 {
			CGCam_UpdateBarFade();
		}

		//Normal fading - separate call because can finish after camera is disabled
		CGCam_UpdateFade();

		//Update shaking if there's any
		//CGCam_UpdateSmooth( cg.refdef.vieworg, cg.refdefViewAngles );
		CGCam_UpdateShake(&mut cg.refdef.vieworg, &mut cg.refdefViewAngles);
		AnglesToAxis(&cg.refdefViewAngles, &mut cg.refdef.viewaxis);
	}
}

/*
-------------------------
CGCam_DrawWideScreen
-------------------------
*/

pub fn CGCam_DrawWideScreen() {
	unsafe {
		let mut modulate: [f32; 4];

		//Only draw if visible
		if client_camera.bar_alpha != 0.0 {
			CGCam_UpdateBarFade();

			modulate[0] = 0.0;
			modulate[1] = 0.0;
			modulate[2] = 0.0;
			modulate[3] = client_camera.bar_alpha;

			CG_FillRect(cg.refdef.x, cg.refdef.y, 640, (client_camera.bar_height as c_int), &modulate);
			CG_FillRect(cg.refdef.x, cg.refdef.y + 480 - (client_camera.bar_height as c_int), 640, (client_camera.bar_height as c_int), &modulate);
		}

		//NOTENOTE: Camera always draws the fades unless the alpha is 0
		if client_camera.fade_color[3] == 0.0 {
			return;
		}

		CG_FillRect(cg.refdef.x, cg.refdef.y, 640, 480, &client_camera.fade_color);
	}
}

/*
-------------------------
CGCam_RenderScene
-------------------------
*/
pub fn CGCam_RenderScene() {
	CGCam_Update();
	unsafe {
		CG_CalcVrect();
	}
}

/*
-------------------------
CGCam_Shake
-------------------------
*/

pub fn CGCam_Shake(intensity: f32, duration: c_int) {
	unsafe {
		let mut intensity = intensity;
		if intensity > MAX_SHAKE_INTENSITY {
			intensity = MAX_SHAKE_INTENSITY;
		}

		client_camera.shake_intensity = intensity;
		client_camera.shake_duration = duration;
		client_camera.shake_start = cg.time;
#ifdef _IMMERSION
		// FIX ME: This is far too weak... but I don't want it to interfere with other effects.
		cgi_FF_Shake((intensity * 625.0) as c_int, duration);	// 625 = (10000 / MAX_SHAKE_INTENSITY)
#endif // _IMMERSION
#ifdef _XBOX
		cgi_FF_Xbox_Shake(intensity,duration);
#endif
	}
}

/*
-------------------------
CGCam_UpdateShake

This doesn't actually affect the camera's info, but passed information instead
-------------------------
*/

pub fn CGCam_UpdateShake(origin: &mut [f32; 3], angles: &mut [f32; 3]) {
	unsafe {
		let mut moveDir: [f32; 3];
		let mut intensity_scale: f32;
		let mut intensity: f32;

		if client_camera.shake_duration <= 0 {
			return;
		}

		if cg.time > (client_camera.shake_start + client_camera.shake_duration) {
			client_camera.shake_intensity = 0.0;
			client_camera.shake_duration = 0;
			client_camera.shake_start = 0;
			return;
		}

		//intensity_scale now also takes into account FOV with 90.0 as normal
		intensity_scale = 1.0 - (((cg.time - client_camera.shake_start) as f32) / (client_camera.shake_duration as f32)) * (((client_camera.FOV+client_camera.FOV2)/2.0)/90.0);

		intensity = client_camera.shake_intensity * intensity_scale;

		for i in 0..3 {
			moveDir[i] = (crandom() * intensity);
		}

		//FIXME: Lerp

		//Move the camera
		VectorAdd(origin, &moveDir, origin);

		for i in 0..2 {
			// Don't do ROLL
			moveDir[i] = (crandom() * intensity);
		}

		//FIXME: Lerp

		//Move the angles
		VectorAdd(angles, &moveDir, angles);
	}
}

pub fn CGCam_Smooth(intensity: f32, duration: c_int) {
	unsafe {
		client_camera.smooth_active = false; // means smooth_origin and angles are valid
		if intensity > 1.0 || intensity == 0.0 || duration < 1 {
			client_camera.info_state &= !CAMERA_SMOOTHING;
			return;
		}
		client_camera.info_state |= CAMERA_SMOOTHING;
		client_camera.smooth_intensity = intensity;
		client_camera.smooth_duration = duration;
		client_camera.smooth_start = cg.time;
	}
}

pub fn CGCam_UpdateSmooth(origin: &mut [f32; 3], _angles: &mut [f32; 3]) {
	unsafe {
		if ((client_camera.info_state&CAMERA_SMOOTHING) == 0) || cg.time > (client_camera.smooth_start + client_camera.smooth_duration) {
			client_camera.info_state &= !CAMERA_SMOOTHING;
			return;
		}
		if !client_camera.smooth_active {
			client_camera.smooth_active = true;
			VectorCopy(origin, &mut client_camera.smooth_origin);
			return;
		}
		let mut factor = client_camera.smooth_intensity;
		if client_camera.smooth_duration > 200 && cg.time > (client_camera.smooth_start + client_camera.smooth_duration-100) {
			factor += (1.0-client_camera.smooth_intensity)*
				(100.0-((client_camera.smooth_start + client_camera.smooth_duration-cg.time) as f32))/100.0;
		}
		for i in 0..3 {
			client_camera.smooth_origin[i] *= (1.0-factor);
			client_camera.smooth_origin[i] += factor*origin[i];
			origin[i] = client_camera.smooth_origin[i];
		}
	}
}

pub fn CGCam_NotetrackProcessFov(addlArg: *const c_char) {
	unsafe {
		let mut a: usize = 0;
		let mut t: [c_char; 64] = [0; 64];

		if addlArg.is_null() || *addlArg as u8 == 0 {
			Com_Printf(b"camera roff 'fov' notetrack missing fov argument\n\0".as_ptr() as *const c_char, addlArg);
			return;
		}
		if isdigit(*addlArg as c_int) != 0 {
			// "fov <new fov>"
			let mut d: usize = 0;
			let tsize: usize = 64;

			memset(t.as_mut_ptr() as *mut c_void, 0, tsize);
			while *addlArg.offset(a as isize) as u8 != 0 && d < tsize {
				t[d] = *addlArg.offset(a as isize);
				d += 1;
				a += 1;
			}
			// now the contents of t represent our desired fov
			let mut newFov = atof(t.as_ptr());
#ifndef FINAL_BUILD
			if cg_roffdebug.integer != 0 {
				if fabs(cg_roffval1.value as f64) > 0.001 {
					newFov = cg_roffval1.value;
				}
			}
#endif
			if cg_roffdebug.integer != 0 {
				Com_Printf(b"notetrack: 'fov %2.2f' on frame %d\n\0".as_ptr() as *const c_char, newFov, client_camera.roff_frame);
			}
			CGCam_Zoom(newFov, 0.0);
		}
	}
}

pub fn CGCam_NotetrackProcessFovZoom(addlArg: *const c_char) {
	unsafe {
		let mut a: usize = 0;
		let mut beginFOV: f32 = 0.0;
		let mut endFOV: f32 = 0.0;
		let mut fovTime: f32 = 0.0;

		if addlArg.is_null() || *addlArg as u8 == 0 {
			Com_Printf(b"camera roff 'fovzoom' notetrack missing arguments\n\0".as_ptr() as *const c_char, addlArg);
			return;
		}
		//
		// "fovzoom <begin fov> <end fov> <time>"
		//
		let mut t: [c_char; 64] = [0; 64];
		let mut d: usize = 0;
		let tsize: usize = 64;

		memset(t.as_mut_ptr() as *mut c_void, 0, tsize);
		while *addlArg.offset(a as isize) as u8 != 0 && !isspace(*addlArg.offset(a as isize) as c_int) != 0 && d < tsize {
			t[d] = *addlArg.offset(a as isize);
			d += 1;
			a += 1;
		}
		if isdigit(t[0] as c_int) == 0 {
			// assume a non-number here means we should start from our current fov
			beginFOV = client_camera.FOV;
		} else {
			// now the contents of t represent our beginning fov
			beginFOV = atof(t.as_ptr());
		}

		// eat leading whitespace
		while *addlArg.offset(a as isize) as u8 != 0 && *addlArg.offset(a as isize) as u8 == b' ' {
			a += 1;
		}
		if *addlArg.offset(a as isize) as u8 != 0 {
			d = 0;
			memset(t.as_mut_ptr() as *mut c_void, 0, tsize);
			while *addlArg.offset(a as isize) as u8 != 0 && !isspace(*addlArg.offset(a as isize) as c_int) != 0 && d < tsize {
				t[d] = *addlArg.offset(a as isize);
				d += 1;
				a += 1;
			}
			// now the contents of t represent our end fov
			endFOV = atof(t.as_ptr());

			// eat leading whitespace
			while *addlArg.offset(a as isize) as u8 != 0 && *addlArg.offset(a as isize) as u8 == b' ' {
				a += 1;
			}
			if *addlArg.offset(a as isize) as u8 != 0 {
				d = 0;
				memset(t.as_mut_ptr() as *mut c_void, 0, tsize);
				while *addlArg.offset(a as isize) as u8 != 0 && !isspace(*addlArg.offset(a as isize) as c_int) != 0 && d < tsize {
					t[d] = *addlArg.offset(a as isize);
					d += 1;
					a += 1;
				}
				// now the contents of t represent our time
				fovTime = atof(t.as_ptr());
			} else {
				Com_Printf(b"camera roff 'fovzoom' notetrack missing 'time' argument\n\0".as_ptr() as *const c_char, addlArg);
				return;
			}
#ifndef FINAL_BUILD
			if cg_roffdebug.integer != 0 {
				if fabs(cg_roffval1.value as f64) > 0.001 {
					beginFOV = cg_roffval1.value;
				}
				if fabs(cg_roffval2.value as f64) > 0.001 {
					endFOV = cg_roffval2.value;
				}
				if fabs(cg_roffval3.value as f64) > 0.001 {
					fovTime = cg_roffval3.value;
				}
			}
#endif
			if cg_roffdebug.integer != 0 {
				Com_Printf(b"notetrack: 'fovzoom %2.2f %2.2f %5.1f' on frame %d\n\0".as_ptr() as *const c_char, beginFOV, endFOV, fovTime, client_camera.roff_frame);
			}
			CGCam_Zoom2(beginFOV, endFOV, fovTime);
		} else {
			Com_Printf(b"camera roff 'fovzoom' notetrack missing 'end fov' argument\n\0".as_ptr() as *const c_char, addlArg);
			return;
		}
	}
}

pub fn CGCam_NotetrackProcessFovAccel(addlArg: *const c_char) {
	unsafe {
		let mut a: usize = 0;
		let mut beginFOV: f32 = 0.0;
		let mut fovDelta: f32 = 0.0;
		let mut fovDelta2: f32 = 0.0;
		let mut fovTime: f32 = 0.0;

		if addlArg.is_null() || *addlArg as u8 == 0 {
			Com_Printf(b"camera roff 'fovaccel' notetrack missing arguments\n\0".as_ptr() as *const c_char, addlArg);
			return;
		}
		//
		// "fovaccel <begin fov> <fov delta> <fov delta2> <time>"
		//
		// where 'begin fov' is initial position, 'fov delta' is velocity, and 'fov delta2' is acceleration.
		let mut t: [c_char; 64] = [0; 64];
		let mut d: usize = 0;
		let tsize: usize = 64;

		memset(t.as_mut_ptr() as *mut c_void, 0, tsize);
		while *addlArg.offset(a as isize) as u8 != 0 && !isspace(*addlArg.offset(a as isize) as c_int) != 0 && d < tsize {
			t[d] = *addlArg.offset(a as isize);
			d += 1;
			a += 1;
		}
		if isdigit(t[0] as c_int) == 0 {
			// assume a non-number here means we should start from our current fov
			beginFOV = client_camera.FOV;
		} else {
			// now the contents of t represent our beginning fov
			beginFOV = atof(t.as_ptr());
		}

		// eat leading whitespace
		while *addlArg.offset(a as isize) as u8 != 0 && *addlArg.offset(a as isize) as u8 == b' ' {
			a += 1;
		}
		if *addlArg.offset(a as isize) as u8 != 0 {
			d = 0;
			memset(t.as_mut_ptr() as *mut c_void, 0, tsize);
			while *addlArg.offset(a as isize) as u8 != 0 && !isspace(*addlArg.offset(a as isize) as c_int) != 0 && d < tsize {
				t[d] = *addlArg.offset(a as isize);
				d += 1;
				a += 1;
			}
			// now the contents of t represent our delta
			fovDelta = atof(t.as_ptr());

			// eat leading whitespace
			while *addlArg.offset(a as isize) as u8 != 0 && *addlArg.offset(a as isize) as u8 == b' ' {
				a += 1;
			}
			if *addlArg.offset(a as isize) as u8 != 0 {
				d = 0;
				memset(t.as_mut_ptr() as *mut c_void, 0, tsize);
				while *addlArg.offset(a as isize) as u8 != 0 && !isspace(*addlArg.offset(a as isize) as c_int) != 0 && d < tsize {
					t[d] = *addlArg.offset(a as isize);
					d += 1;
					a += 1;
				}
				// now the contents of t represent our fovDelta2
				fovDelta2 = atof(t.as_ptr());

				// eat leading whitespace
				while *addlArg.offset(a as isize) as u8 != 0 && *addlArg.offset(a as isize) as u8 == b' ' {
					a += 1;
				}
				if *addlArg.offset(a as isize) as u8 != 0 {
					d = 0;
					memset(t.as_mut_ptr() as *mut c_void, 0, tsize);
					while *addlArg.offset(a as isize) as u8 != 0 && !isspace(*addlArg.offset(a as isize) as c_int) != 0 && d < tsize {
						t[d] = *addlArg.offset(a as isize);
						d += 1;
						a += 1;
					}
					// now the contents of t represent our time
					fovTime = atof(t.as_ptr());
				} else {
					Com_Printf(b"camera roff 'fovaccel' notetrack missing 'time' argument\n\0".as_ptr() as *const c_char, addlArg);
					return;
				}
				if cg_roffdebug.integer != 0 {
					Com_Printf(b"notetrack: 'fovaccel %2.2f %3.5f %3.5f %d' on frame %d\n\0".as_ptr() as *const c_char, beginFOV, fovDelta, fovDelta2, fovTime, client_camera.roff_frame);
				}
				CGCam_ZoomAccel(beginFOV, fovDelta, fovDelta2, fovTime);
			} else {
				Com_Printf(b"camera roff 'fovaccel' notetrack missing 'delta2' argument\n\0".as_ptr() as *const c_char, addlArg);
				return;
			}
		} else {
			Com_Printf(b"camera roff 'fovaccel' notetrack missing 'delta' argument\n\0".as_ptr() as *const c_char, addlArg);
			return;
		}
	}
}

// 3/18/03 kef -- blatantly thieved from G_RoffNotetrackCallback
fn CG_RoffNotetrackCallback(notetrack: *const c_char) {
	unsafe {
		let mut i: usize = 0;
		let mut r: usize = 0;
		let mut typ: [c_char; 256] = [0; 256];
		//	char argument[512];
		let mut addlArg: [c_char; 512] = [0; 512];
		let mut addlArgs: c_int = 0;

		if notetrack.is_null() {
			return;
		}

		//notetrack = "fov 65";

		while *notetrack.offset(i as isize) as u8 != 0 && *notetrack.offset(i as isize) as u8 != b' ' {
			typ[i] = *notetrack.offset(i as isize);
			i += 1;
		}

		typ[i] = 0;

		//if (notetrack[i] != ' ')
		//{ //didn't pass in a valid notetrack type, or forgot the argument for it
		//	return;
		//}

		/*	i++;

		while (notetrack[i] && notetrack[i] != ' ')
		{
			if (notetrack[i] != '\n' && notetrack[i] != '\r')
			{ //don't read line ends for an argument
				argument[r] = notetrack[i];
				r++;
			}
			i++;
		}
		argument[r] = '\0';
		if (!r)
		{
			return;
		}
		*/

		if *notetrack.offset(i as isize) as u8 == b' ' {
			//additional arguments...
			addlArgs = 1;

			i += 1;
			r = 0;
			while *notetrack.offset(i as isize) as u8 != 0 {
				addlArg[r] = *notetrack.offset(i as isize);
				r += 1;
				i += 1;
			}
			addlArg[r] = 0;
		}

		if strcmp(typ.as_ptr(), b"cut\0".as_ptr() as *const c_char) == 0 {
			client_camera.info_state |= CAMERA_CUT;
			if cg_roffdebug.integer != 0 {
				Com_Printf(b"notetrack: 'cut' on frame %d\n\0".as_ptr() as *const c_char, client_camera.roff_frame);
			}

			// this is just a really hacky way of getting a cut and a fov command on the same frame
			if addlArgs != 0 {
				CG_RoffNotetrackCallback(addlArg.as_ptr());
			}
		} else if strcmp(typ.as_ptr(), b"fov\0".as_ptr() as *const c_char) == 0 {
			if addlArgs != 0 {
				CGCam_NotetrackProcessFov(addlArg.as_ptr());
				return;
			}
			Com_Printf(b"camera roff 'fov' notetrack missing fov argument\n\0".as_ptr() as *const c_char, addlArg.as_ptr());
		} else if strcmp(typ.as_ptr(), b"fovzoom\0".as_ptr() as *const c_char) == 0 {
			if addlArgs != 0 {
				CGCam_NotetrackProcessFovZoom(addlArg.as_ptr());
				return;
			}
			Com_Printf(b"camera roff 'fovzoom' notetrack missing 'begin fov' argument\n\0".as_ptr() as *const c_char, addlArg.as_ptr());
		} else if strcmp(typ.as_ptr(), b"fovaccel\0".as_ptr() as *const c_char) == 0 {
			if addlArgs != 0 {
				CGCam_NotetrackProcessFovAccel(addlArg.as_ptr());
				return;
			}
			Com_Printf(b"camera roff 'fovaccel' notetrack missing 'begin fov' argument\n\0".as_ptr() as *const c_char, addlArg.as_ptr());
		}
	}
}

/*
-------------------------
CGCam_StartRoff

Sets up the camera to use
a rof file
-------------------------
*/

pub fn CGCam_StartRoff(roff: *const c_char) {
	unsafe {
		CGCam_FollowDisable();
		CGCam_TrackDisable();

		// Set up the roff state info..we'll hijack the moving and panning code until told otherwise
		//	...CAMERA_FOLLOWING would be a case that could override this..
		client_camera.info_state |= CAMERA_MOVING;
		client_camera.info_state |= CAMERA_PANNING;

		if G_LoadRoff(roff) == 0 {
			// The load failed so don't turn on the roff playback...
			Com_Printf(S_COLOR_RED.as_ptr() as *const c_char, b"ROFF camera playback failed\n\0".as_ptr() as *const c_char);
			return;
		}

		client_camera.info_state |= CAMERA_ROFFING;

		strncpy(client_camera.sRoff.as_mut_ptr(), roff, client_camera.sRoff.len());
		client_camera.roff_frame = 0;
		client_camera.next_roff_time = cg.time;	// I can work right away
	}
}

/*
-------------------------
CGCam_StopRoff

Stops camera rof
-------------------------
*/

fn CGCam_StopRoff() {
	unsafe {
		// Clear the roff flag
		client_camera.info_state &= !CAMERA_ROFFING;
		client_camera.info_state &= !CAMERA_MOVING;
	}
}

/*
------------------------------------------------------
CGCam_Roff

Applies the sampled roff data to the camera and does
the lerping itself...this is done because the current
camera interpolation doesn't seem to work all that
great when you are adjusting the camera org and angles
so often...or maybe I'm just on crack.
------------------------------------------------------
*/

fn CGCam_Roff() {
	unsafe {
		while client_camera.next_roff_time <= cg.time {
			// Make sure that the roff is cached
			let roff_id = G_LoadRoff(client_camera.sRoff.as_ptr());

			if roff_id == 0 {
				return;
			}

			// The ID is one higher than the array index
			let roff = &roffs[(roff_id - 1) as usize];
			let mut org: [f32; 3] = [0.0; 3];
			let mut ang: [f32; 3] = [0.0; 3];

			if roff.typ == 2 {
				let data = &((roff.data as *const move_rotate2_t).offset(client_camera.roff_frame as isize)) as *const move_rotate2_t;
				VectorCopy(&(*data).origin_delta, &mut org);
				VectorCopy(&(*data).rotate_delta, &mut ang);

				// since we just hit a new frame, clear our CUT flag
				client_camera.info_state &= !CAMERA_CUT;

				if (*data).mStartNote != -1 || (*data).mNumNotes != 0 {
					CG_RoffNotetrackCallback(*roffs[(roff_id - 1) as usize].mNoteTrackIndexes.offset((*data).mStartNote as isize));
				}
			} else {
				let data = &((roff.data as *const move_rotate_t).offset(client_camera.roff_frame as isize)) as *const move_rotate_t;
				VectorCopy(&(*data).origin_delta, &mut org);
				VectorCopy(&(*data).rotate_delta, &mut ang);
			}

			// Yeah, um, I guess this just has to be negated?
			//ang[PITCH]	=- ang[PITCH];
			ang[2] = -ang[2];  // ROLL
			// might need to to yaw as well.  need a test...

			if cg_developer.integer != 0 {
				Com_Printf(S_COLOR_GREEN.as_ptr() as *const c_char, b"CamROFF: frame: %d o:<%.2f %.2f %.2f> a:<%.2f %.2f %.2f>\n\0".as_ptr() as *const c_char,
							client_camera.roff_frame,
							org[0], org[1], org[2],
							ang[0], ang[1], ang[2]);
			}

			if client_camera.roff_frame != 0 {
				// Don't mess with angles if we are following
				if (client_camera.info_state & CAMERA_FOLLOWING) == 0 {
					VectorAdd(&client_camera.angles, &client_camera.angles2, &mut client_camera.angles);
				}

				VectorCopy(&client_camera.origin2, &mut client_camera.origin);
			}

			// Don't mess with angles if we are following
			if (client_camera.info_state & CAMERA_FOLLOWING) == 0 {
				VectorCopy(&ang, &mut client_camera.angles2);
				client_camera.pan_time = cg.time;
				client_camera.pan_duration = roff.mFrameTime as f32;
			}

			VectorAdd(&client_camera.origin, &org, &mut client_camera.origin2);

			client_camera.move_time = cg.time;
			client_camera.move_duration = roff.mFrameTime as f32;

			client_camera.roff_frame += 1;
			if client_camera.roff_frame >= roff.frames {
				CGCam_StopRoff();
				return;
			}

			// Check back in frameTime to get the next roff entry
			client_camera.next_roff_time += roff.mFrameTime;
		}
	}
}

pub fn CMD_CGCam_Disable() {
	unsafe {
		let fade: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

		CGCam_Disable();
		CGCam_SetFade(&fade);
		player_locked = 0;  // qfalse
	}
}

extern "C" {
	pub fn CG_FillRect(x: c_int, y: c_int, w: c_int, h: c_int, color: *const [f32; 4]);
}
