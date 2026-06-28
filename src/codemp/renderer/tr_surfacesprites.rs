#![allow(non_snake_case)]

use core::ffi::{c_float, c_int, c_uint, c_void};
use core::f32::consts::PI;
use core::mem::size_of;

// Stub type declarations for cross-module types
type vec3_t = [f32; 3];
type vec2_t = [f32; 2];
type byte = u8;
type qboolean = i32;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// Stub types from tr_local.h
#[repr(C)]
pub struct color4ub_t([byte; 4]);

impl color4ub_t {
	fn get_mut(&mut self, i: usize) -> &mut byte {
		&mut self.0[i]
	}
	fn get(&self, i: usize) -> byte {
		self.0[i]
	}
}

#[repr(C)]
pub struct vec3_color_t {
	r: byte,
	g: byte,
	b: byte,
}

// Stub declarations for external types
#[repr(C)]
pub struct shaderStage_t {
	pub stateBits: c_uint,
	pub bundle: [bundle_t; 1],
	pub ss: *mut surfaceSpriteData_t,
}

#[repr(C)]
pub struct surfaceSpriteData_t {
	pub width: f32,
	pub height: f32,
	pub density: f32,
	pub variance: [f32; 2],
	pub windIdle: f32,
	pub wind: f32,
	pub windGust: f32,
	pub windScale: f32,
	pub fadeMax: f32,
	pub fadeDist: f32,
	pub fadeScale: f32,
	pub vertSkew: f32,
	pub facing: c_int,
	pub surfaceSpriteType: c_int,
	pub fxAlphaStart: f32,
	pub fxAlphaEnd: f32,
	pub fxDuration: f32,
	pub fxGrow: [f32; 2],
}

#[repr(C)]
pub struct bundle_t {
	pub image: *mut c_void,
}

#[repr(C)]
pub struct shaderCommands_t {
	pub xyz: *mut vec3_t,
	pub normal: *mut vec3_t,
	pub vertexColors: *mut [byte; 4],
	pub indexes: *mut c_int,
	pub numVertexes: c_int,
	pub numIndexes: c_int,
}

#[repr(C)]
pub struct trRefEntity_t {
	pub unused: [u8; 1],
}

#[repr(C)]
pub struct refdef_t {
	pub time: c_int,
	pub fov_x: f32,
}

#[repr(C)]
pub struct viewParms_t {
	pub ori: orient_t,
}

#[repr(C)]
pub struct orient_t {
	pub origin: vec3_t,
	pub axis: [vec3_t; 3],
	pub viewOrigin: vec3_t,
}

// Stubs for global state
#[repr(C)]
pub struct backEndState_t {
	pub refdef: refdef_t,
	pub currentEntity: *mut trRefEntity_t,
	pub viewParms: viewParms_t,
	pub ori: orient_t,
}

#[repr(C)]
pub struct tessState_t {
	pub SSInitializedWind: qboolean,
	pub svars: shaderVars_t,
}

#[repr(C)]
pub struct shaderVars_t {
	pub texcoords: [*mut f32; 2],
}

#[repr(C)]
pub struct trGlobals_t {
	pub refdef: refdef_t,
	pub worldEntity: trRefEntity_t,
}

// External globals (stubs)
extern "C" {
	static mut backEnd: backEndState_t;
	static mut tess: tessState_t;
	static mut tr: trGlobals_t;

	static mut r_windSpeed: *mut cvar_t;
	static mut r_windGust: *mut cvar_t;
	static mut r_windAngle: *mut cvar_t;
	static mut r_windDampFactor: *mut cvar_t;
	static mut r_windPointForce: *mut cvar_t;
	static mut r_windPointX: *mut cvar_t;
	static mut r_windPointY: *mut cvar_t;
	static mut r_surfaceWeather: *mut cvar_t;
	static mut r_surfaceSprites: *mut cvar_t;
	static mut r_drawfog: *mut cvar_t;

	// External types
	pub struct cvar_t {
		value: f32,
		integer: c_int,
	}
}

// External functions
extern "C" {
	fn R_IsRaining() -> qboolean;
	fn R_IsPuffing() -> qboolean;
	fn R_GetWindSpeed(speed: *mut f32) -> qboolean;
	fn R_GetWindVector(vec: *mut vec3_t) -> qboolean;
	fn R_WorldNormalToEntity(world: *mut vec3_t, local: *mut vec3_t);
	fn Com_Printf(fmt: *const i8, ...);
	fn flrand(min: f32, max: f32) -> f32;
	fn Q_rsqrt(x: f32) -> f32;

	// Math functions
	fn VectorScale(a: *const vec3_t, b: f32, c: *mut vec3_t);
	fn VectorMA(a: *const vec3_t, scale: f32, b: *const vec3_t, c: *mut vec3_t);
	fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
	fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t);
	fn VectorLengthSquared(v: *const vec3_t) -> f32;
	fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, cross: *mut vec3_t);
	fn vectoangles(vec: *const vec3_t, angles: *mut vec3_t);
	fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
}

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// tr_shade.c

// #include "tr_local.h"

// #include "tr_QuickSprite.h"
// #include "tr_WorldEffects.h"

/////===== Part of the VERTIGON system =====/////
// The surfacesprites are a simple system.  When a polygon with this shader stage on it is drawn,
// there are randomly distributed images (defined by the shader stage) placed on the surface.
// these are capable of doing effects, grass, or simple oriented sprites.
// They usually stick vertically off the surface, hence the term vertigons.

// The vertigons are applied as part of the renderer backend.  That is, they access OpenGL calls directly.

pub static mut randomindex: byte = 0;
pub static mut randominterval: byte = 0;

pub const randomchart: [f32; 256] = [
	0.6554f32, 0.6909f32, 0.4806f32, 0.6218f32, 0.5717f32, 0.3896f32, 0.0677f32, 0.7356f32,
	0.8333f32, 0.1105f32, 0.4445f32, 0.8161f32, 0.4689f32, 0.0433f32, 0.7152f32, 0.0336f32,
	0.0186f32, 0.9140f32, 0.1626f32, 0.6553f32, 0.8340f32, 0.7094f32, 0.2020f32, 0.8087f32,
	0.9119f32, 0.8009f32, 0.1339f32, 0.8492f32, 0.9173f32, 0.5003f32, 0.6012f32, 0.6117f32,
	0.5525f32, 0.5787f32, 0.1586f32, 0.3293f32, 0.9273f32, 0.7791f32, 0.8589f32, 0.4985f32,
	0.0883f32, 0.8545f32, 0.2634f32, 0.4727f32, 0.3624f32, 0.1631f32, 0.7825f32, 0.0662f32,
	0.6704f32, 0.3510f32, 0.7525f32, 0.9486f32, 0.4685f32, 0.1535f32, 0.1545f32, 0.1121f32,
	0.4724f32, 0.8483f32, 0.3833f32, 0.1917f32, 0.8207f32, 0.3885f32, 0.9702f32, 0.9200f32,
	0.8348f32, 0.7501f32, 0.6675f32, 0.4994f32, 0.0301f32, 0.5225f32, 0.8011f32, 0.1696f32,
	0.5351f32, 0.2752f32, 0.2962f32, 0.7550f32, 0.5762f32, 0.7303f32, 0.2835f32, 0.4717f32,
	0.1818f32, 0.2739f32, 0.6914f32, 0.7748f32, 0.7640f32, 0.8355f32, 0.7314f32, 0.5288f32,
	0.7340f32, 0.6692f32, 0.6813f32, 0.2810f32, 0.8057f32, 0.0648f32, 0.8749f32, 0.9199f32,
	0.1462f32, 0.5237f32, 0.3014f32, 0.4994f32, 0.0278f32, 0.4268f32, 0.7238f32, 0.5107f32,
	0.1378f32, 0.7303f32, 0.7200f32, 0.3819f32, 0.2034f32, 0.7157f32, 0.5552f32, 0.4887f32,
	0.0871f32, 0.3293f32, 0.2892f32, 0.4545f32, 0.0088f32, 0.1404f32, 0.0275f32, 0.0238f32,
	0.0515f32, 0.4494f32, 0.7206f32, 0.2893f32, 0.6060f32, 0.5785f32, 0.4182f32, 0.5528f32,
	0.9118f32, 0.8742f32, 0.3859f32, 0.6030f32, 0.3495f32, 0.4550f32, 0.9875f32, 0.6900f32,
	0.6416f32, 0.2337f32, 0.7431f32, 0.9788f32, 0.6181f32, 0.2464f32, 0.4661f32, 0.7621f32,
	0.7020f32, 0.8203f32, 0.8869f32, 0.2145f32, 0.7724f32, 0.6093f32, 0.6692f32, 0.9686f32,
	0.5609f32, 0.0310f32, 0.2248f32, 0.2950f32, 0.2365f32, 0.1347f32, 0.2342f32, 0.1668f32,
	0.3378f32, 0.4330f32, 0.2775f32, 0.9901f32, 0.7053f32, 0.7266f32, 0.4840f32, 0.2820f32,
	0.5733f32, 0.4555f32, 0.6049f32, 0.0770f32, 0.4760f32, 0.6060f32, 0.4159f32, 0.3427f32,
	0.1234f32, 0.7062f32, 0.8569f32, 0.1878f32, 0.9057f32, 0.9399f32, 0.8139f32, 0.1407f32,
	0.1794f32, 0.9123f32, 0.9493f32, 0.2827f32, 0.9934f32, 0.0952f32, 0.4879f32, 0.5160f32,
	0.4118f32, 0.4873f32, 0.3642f32, 0.7470f32, 0.0866f32, 0.5172f32, 0.6365f32, 0.2676f32,
	0.2407f32, 0.7223f32, 0.5761f32, 0.1143f32, 0.7137f32, 0.2342f32, 0.3353f32, 0.6880f32,
	0.2296f32, 0.6023f32, 0.6027f32, 0.4138f32, 0.5408f32, 0.9859f32, 0.1503f32, 0.7238f32,
	0.6054f32, 0.2477f32, 0.6804f32, 0.1432f32, 0.4540f32, 0.9776f32, 0.8762f32, 0.7607f32,
	0.9025f32, 0.9807f32, 0.0652f32, 0.8661f32, 0.7663f32, 0.2586f32, 0.3994f32, 0.0335f32,
	0.7328f32, 0.0166f32, 0.9589f32, 0.4348f32, 0.5493f32, 0.7269f32, 0.6867f32, 0.6614f32,
	0.6800f32, 0.7804f32, 0.5591f32, 0.8381f32, 0.0910f32, 0.7573f32, 0.8985f32, 0.3083f32,
	0.3188f32, 0.8481f32, 0.2356f32, 0.6736f32, 0.4770f32, 0.4560f32, 0.6266f32, 0.4677f32,
];

const WIND_DAMP_INTERVAL: f32 = 50.0;
const WIND_GUST_TIME: f32 = 2500.0;
const WIND_GUST_DECAY: f32 = 1.0 / WIND_GUST_TIME;

pub static mut lastSSUpdateTime: c_int = 0;
pub static mut curWindSpeed: f32 = 0.0;
pub static mut curWindGust: f32 = 5.0;
pub static mut curWeatherAmount: f32 = 1.0;
pub static mut curWindBlowVect: vec3_t = [0.0, 0.0, 0.0];
pub static mut targetWindBlowVect: vec3_t = [0.0, 0.0, 0.0];
pub static mut curWindGrassDir: vec3_t = [0.0, 0.0, 0.0];
pub static mut targetWindGrassDir: vec3_t = [0.0, 0.0, 0.0];
pub static mut totalsurfsprites: c_int = 0;
pub static mut sssurfaces: c_int = 0;

pub static mut curWindPointActive: qboolean = qfalse;
pub static mut curWindPointForce: f32 = 0.0;
pub static mut curWindPoint: vec3_t = [0.0, 0.0, 0.0];
pub static mut nextGustTime: c_int = 0;
pub static mut gustLeft: f32 = 0.0;

pub static mut standardfovinitialized: qboolean = qfalse;
pub static mut standardfovx: f32 = 90.0;
pub static mut standardscalex: f32 = 1.0;
pub static mut rangescalefactor: f32 = 1.0;

pub static mut ssrightvectors: [vec3_t; 4] = [[0.0; 3]; 4];
pub static mut ssfwdvector: vec3_t = [0.0, 0.0, 0.0];
pub static mut rightvectorcount: c_int = 0;

pub static mut ssLastEntityDrawn: *mut trRefEntity_t = core::ptr::null_mut();
pub static mut ssViewOrigin: vec3_t = [0.0, 0.0, 0.0];
pub static mut ssViewRight: vec3_t = [0.0, 0.0, 0.0];
pub static mut ssViewUp: vec3_t = [0.0, 0.0, 0.0];

fn R_SurfaceSpriteFrameUpdate() {
	let mut dtime: f32;
	let mut dampfactor: f32; // Time since last update and damping time for wind changes
	let mut ratio: f32;
	let mut ang: vec3_t = [0.0, 0.0, 0.0];
	let mut diff: vec3_t = [0.0, 0.0, 0.0];
	let mut retwindvec: vec3_t = [0.0, 0.0, 0.0];
	let mut targetspeed: f32 = 0.0;
	let up: vec3_t = [0.0, 0.0, 1.0];

	unsafe {
		if backEnd.refdef.time == lastSSUpdateTime {
			return;
		}

		if backEnd.refdef.time < lastSSUpdateTime {
			// Time is BEFORE the last update time, so reset everything.
			curWindGust = 5.0;
			curWindSpeed = (*r_windSpeed).value;
			nextGustTime = 0;
			gustLeft = 0.0;
		}

		// Reset the last entity drawn, since this is a new frame.
		ssLastEntityDrawn = core::ptr::null_mut();

		// Adjust for an FOV.  If things look twice as wide on the screen, pretend the shaders have twice the range.
		// ASSUMPTION HERE IS THAT "standard" fov is the first one rendered.

		if standardfovinitialized == 0 {
			// This isn't initialized yet.
			if backEnd.refdef.fov_x > 50.0 && backEnd.refdef.fov_x < 135.0 { // I don't consider anything below 50 or above 135 to be "normal".
				standardfovx = backEnd.refdef.fov_x;
				standardscalex = (standardfovx * 0.5 * (PI / 180.0)).tan();
				standardfovinitialized = qtrue;
			} else {
				standardfovx = 90.0;
				standardscalex = (standardfovx * 0.5 * (PI / 180.0)).tan();
			}
			rangescalefactor = 1.0; // Don't multiply the shader range by anything.
		} else if standardfovx == backEnd.refdef.fov_x {
			// This is the standard FOV (or higher), don't multiply the shader range.
			rangescalefactor = 1.0;
		} else {
			// We are using a non-standard FOV.  We need to multiply the range of the shader by a scale factor.
			if backEnd.refdef.fov_x > 135.0 {
				rangescalefactor = standardscalex / (135.0 * 0.5 * (PI / 180.0)).tan();
			} else {
				rangescalefactor = standardscalex / (backEnd.refdef.fov_x * 0.5 * (PI / 180.0)).tan();
			}
		}

		// Create a set of four right vectors so that vertical sprites aren't always facing the same way.
		// First generate a HORIZONTAL forward vector (important).
		CrossProduct(&ssViewRight as *const _, &up as *const _, &mut ssfwdvector as *mut _);

		// Right Zero has a nudge forward (10 degrees).
		VectorScale(&ssViewRight as *const _, 0.985, &mut ssrightvectors[0] as *mut _);
		VectorMA(&ssrightvectors[0] as *const _, 0.174, &ssfwdvector as *const _, &mut ssrightvectors[0] as *mut _);

		// Right One has a big nudge back (30 degrees).
		VectorScale(&ssViewRight as *const _, 0.866, &mut ssrightvectors[1] as *mut _);
		VectorMA(&ssrightvectors[1] as *const _, -0.5, &ssfwdvector as *const _, &mut ssrightvectors[1] as *mut _);

		// Right two has a big nudge forward (30 degrees).
		VectorScale(&ssViewRight as *const _, 0.866, &mut ssrightvectors[2] as *mut _);
		VectorMA(&ssrightvectors[2] as *const _, 0.5, &ssfwdvector as *const _, &mut ssrightvectors[2] as *mut _);

		// Right three has a nudge back (10 degrees).
		VectorScale(&ssViewRight as *const _, 0.985, &mut ssrightvectors[3] as *mut _);
		VectorMA(&ssrightvectors[3] as *const _, -0.174, &ssfwdvector as *const _, &mut ssrightvectors[3] as *mut _);

		// Update the wind.
		// If it is raining, get the windspeed from the rain system rather than the cvar
		if R_IsRaining() != 0 || R_IsPuffing() != 0 {
			curWeatherAmount = 1.0;
		} else {
			curWeatherAmount = (*r_surfaceWeather).value;
		}

		if R_GetWindSpeed(&mut targetspeed) != 0 {
			// We successfully got a speed from the rain system.
			// Set the windgust to 5, since that looks pretty good.
			targetspeed *= 0.3;
			if targetspeed >= 1.0 {
				curWindGust = 300.0 / targetspeed;
			} else {
				curWindGust = 0.0;
			}
		} else {
			// Use the cvar.
			targetspeed = (*r_windSpeed).value; // Minimum gust delay, in seconds.
			curWindGust = (*r_windGust).value;
		}

		if targetspeed > 0.0 && curWindGust != 0.0 {
			if gustLeft > 0.0 {
				// We are gusting
				// Add an amount to the target wind speed
				targetspeed *= 1.0 + gustLeft;

				gustLeft -= (backEnd.refdef.time as f32 - lastSSUpdateTime as f32) * WIND_GUST_DECAY;
				if gustLeft <= 0.0 {
					nextGustTime = backEnd.refdef.time + (curWindGust * 1000.0 * flrand(1.0, 4.0)) as c_int;
				}
			} else if backEnd.refdef.time as f32 >= nextGustTime as f32 {
				// See if there is another right now
				// Gust next time, mano
				gustLeft = flrand(0.75, 1.5);
			}
		}

		// See if there is a weather system that will tell us a windspeed.
		if R_GetWindVector(&mut retwindvec as *mut _) != 0 {
			retwindvec[2] = 0.0;
			VectorScale(&retwindvec as *const _, -1.0, &mut retwindvec as *mut _);
			vectoangles(&retwindvec as *const _, &mut ang as *mut _);
		} else {
			// Calculate the target wind vector based off cvars
			ang[1] = (*r_windAngle).value; // YAW
		}

		ang[0] = -90.0 + targetspeed; // PITCH
		if ang[0] > -45.0 {
			ang[0] = -45.0;
		}
		ang[2] = 0.0; // ROLL

		if targetspeed > 0.0 {
			// ang[1] += cos(tr.refdef.time*0.01+flrand(-1.0,1.0))*targetspeed*0.5;
			// ang[0] += sin(tr.refdef.time*0.01+flrand(-1.0,1.0))*targetspeed*0.5;
		}

		// Get the grass wind vector first
		AngleVectors(&ang as *const _, &mut targetWindGrassDir as *mut _, core::ptr::null_mut(), core::ptr::null_mut());
		targetWindGrassDir[2] -= 1.0;
		// VectorScale(targetWindGrassDir, targetspeed, targetWindGrassDir);

		// Now get the general wind vector (no pitch)
		ang[0] = 0.0;
		AngleVectors(&ang as *const _, &mut targetWindBlowVect as *mut _, core::ptr::null_mut(), core::ptr::null_mut());

		// Start calculating a smoothing factor so wind doesn't change abruptly between speeds.
		dampfactor = 1.0 - (*r_windDampFactor).value; // We must exponent the amount LEFT rather than the amount bled off
		dtime = (backEnd.refdef.time as f32 - lastSSUpdateTime as f32) * (1.0 / WIND_DAMP_INTERVAL); // Our dampfactor is geared towards a time interval equal to "1".

		// Note that since there are a finite number of "practical" delta millisecond values possible,
		// the ratio should be initialized into a chart ultimately.
		ratio = dampfactor.powf(dtime);

		// Apply this ratio to the windspeed...
		curWindSpeed = targetspeed - (ratio * (targetspeed - curWindSpeed));

		// Use the curWindSpeed to calculate the final target wind vector (with speed)
		VectorScale(&targetWindBlowVect as *const _, curWindSpeed, &mut targetWindBlowVect as *mut _);
		VectorSubtract(&targetWindBlowVect as *const _, &curWindBlowVect as *const _, &mut diff as *mut _);
		VectorMA(&targetWindBlowVect as *const _, -ratio, &diff as *const _, &mut curWindBlowVect as *mut _);

		// Update the grass vector now
		VectorSubtract(&targetWindGrassDir as *const _, &curWindGrassDir as *const _, &mut diff as *mut _);
		VectorMA(&targetWindGrassDir as *const _, -ratio, &diff as *const _, &mut curWindGrassDir as *mut _);

		lastSSUpdateTime = backEnd.refdef.time;

		curWindPointForce = (*r_windPointForce).value - (ratio * ((*r_windPointForce).value - curWindPointForce));
		if curWindPointForce < 0.01 {
			curWindPointActive = qfalse;
		} else {
			curWindPointActive = qtrue;
			curWindPoint[0] = (*r_windPointX).value;
			curWindPoint[1] = (*r_windPointY).value;
			curWindPoint[2] = 0.0;
		}

		if (*r_surfaceSprites).integer >= 2 {
			Com_Printf(b"Surfacesprites Drawn: %d, on %d surfaces\n\0".as_ptr() as *const i8, totalsurfsprites, sssurfaces);
		}

		totalsurfsprites = 0;
		sssurfaces = 0;
	}
}

/////////////////////////////////////////////
// Surface sprite calculation and drawing.
/////////////////////////////////////////////

const FADE_RANGE: f32 = 250.0;
const WINDPOINT_RADIUS: f32 = 750.0;

static mut SSVertAlpha: [f32; 1024] = [0.0; 1024]; // SHADER_MAX_VERTEXES
static mut SSVertWindForce: [f32; 1024] = [0.0; 1024]; // SHADER_MAX_VERTEXES
static mut SSVertWindDir: [vec2_t; 1024] = [[0.0; 2]; 1024]; // SHADER_MAX_VERTEXES

pub static mut SSAdditiveTransparency: qboolean = qfalse;
pub static mut SSUsingFog: qboolean = qfalse;

/////////////////////////////////////////////
// Vertical surface sprites

fn RB_VerticalSurfaceSprite(
	loc: *const vec3_t,
	width: f32,
	height: f32,
	light: byte,
	alpha: byte,
	wind: f32,
	windidle: f32,
	fog: *const vec2_t,
	hangdown: c_int,
	skew: *const vec2_t,
) {
	let mut loc2: vec3_t = [0.0, 0.0, 0.0];
	let mut right: vec3_t = [0.0, 0.0, 0.0];
	let mut angle: f32;
	let mut windsway: f32;
	let mut points: [f32; 16] = [0.0; 16];
	let mut color: color4ub_t = color4ub_t([light, light, light, alpha]);

	unsafe {
		angle = ((*loc)[0] + (*loc)[1]) * 0.02 + (backEnd.refdef.time as f32 * 0.0015);

		if windidle > 0.0 {
			windsway = height * windidle * 0.075;
			loc2[0] = (*loc)[0] + (*skew)[0] + (angle).cos() * windsway;
			loc2[1] = (*loc)[1] + (*skew)[1] + (angle).sin() * windsway;

			if hangdown != 0 {
				loc2[2] = (*loc)[2] - height;
			} else {
				loc2[2] = (*loc)[2] + height;
			}
		} else {
			loc2[0] = (*loc)[0] + (*skew)[0];
			loc2[1] = (*loc)[1] + (*skew)[1];
			if hangdown != 0 {
				loc2[2] = (*loc)[2] - height;
			} else {
				loc2[2] = (*loc)[2] + height;
			}
		}

		if wind > 0.0 && curWindSpeed > 0.001 {
			windsway = height * wind * 0.075;

			// Add the angle
			VectorMA(&loc2 as *const _, height * wind, &curWindGrassDir as *const _, &mut loc2 as *mut _);
			// Bob up and down
			if curWindSpeed < 40.0 {
				windsway *= curWindSpeed * (1.0 / 100.0);
			} else {
				windsway *= 0.4;
			}
			loc2[2] += (angle * 2.5).sin() * windsway;
		}

		VectorScale(&ssrightvectors[rightvectorcount as usize] as *const _, width * 0.5, &mut right as *mut _);

		color.0[0] = light;
		color.0[1] = light;
		color.0[2] = light;
		color.0[3] = alpha;

		// Bottom right
		// VectorAdd(loc, right, point);
		points[0] = (*loc)[0] + right[0];
		points[1] = (*loc)[1] + right[1];
		points[2] = (*loc)[2] + right[2];
		points[3] = 0.0;

		// Top right
		// VectorAdd(loc2, right, point);
		points[4] = loc2[0] + right[0];
		points[5] = loc2[1] + right[1];
		points[6] = loc2[2] + right[2];
		points[7] = 0.0;

		// Top left
		// VectorSubtract(loc2, right, point);
		points[8] = loc2[0] - right[0] + ssfwdvector[0] * width * 0.2;
		points[9] = loc2[1] - right[1] + ssfwdvector[1] * width * 0.2;
		points[10] = loc2[2] - right[2];
		points[11] = 0.0;

		// Bottom left
		// VectorSubtract(loc, right, point);
		points[12] = (*loc)[0] - right[0];
		points[13] = (*loc)[1] - right[1];
		points[14] = (*loc)[2] - right[2];
		points[15] = 0.0;

		// Add the sprite to the render list.
		// SQuickSprite.Add(points, color, fog);
	}
}

fn RB_VerticalSurfaceSpriteWindPoint(
	loc: *const vec3_t,
	width: f32,
	height: f32,
	light: byte,
	alpha: byte,
	wind: f32,
	windidle: f32,
	fog: *const vec2_t,
	hangdown: c_int,
	skew: *const vec2_t,
	winddiff: *const vec2_t,
	windforce: f32,
) {
	let mut loc2: vec3_t = [0.0, 0.0, 0.0];
	let mut right: vec3_t = [0.0, 0.0, 0.0];
	let mut angle: f32;
	let mut windsway: f32;
	let mut points: [f32; 16] = [0.0; 16];
	let mut color: color4ub_t = color4ub_t([light, light, light, alpha]);

	unsafe {
		let mut windforce_mut = windforce;
		if windforce_mut > 1.0 {
			windforce_mut = 1.0;
		}

		// wind += 1.0-windforce;

		angle = ((*loc)[0] + (*loc)[1]) * 0.02 + (backEnd.refdef.time as f32 * 0.0015);

		if curWindSpeed < 80.0 {
			windsway = (height * windidle * 0.1) * (1.0 + windforce_mut);
			loc2[0] = (*loc)[0] + (*skew)[0] + (angle).cos() * windsway;
			loc2[1] = (*loc)[1] + (*skew)[1] + (angle).sin() * windsway;
		} else {
			loc2[0] = (*loc)[0] + (*skew)[0];
			loc2[1] = (*loc)[1] + (*skew)[1];
		}
		if hangdown != 0 {
			loc2[2] = (*loc)[2] - height;
		} else {
			loc2[2] = (*loc)[2] + height;
		}

		if curWindSpeed > 0.001 {
			// Add the angle
			VectorMA(&loc2 as *const _, height * wind, &curWindGrassDir as *const _, &mut loc2 as *mut _);
		}

		loc2[0] += height * (*winddiff)[0] * windforce_mut;
		loc2[1] += height * (*winddiff)[1] * windforce_mut;
		loc2[2] -= height * windforce_mut * (0.75 + 0.15 * ((backEnd.refdef.time as f32 + 500.0 * windforce_mut) * 0.01).sin());

		VectorScale(&ssrightvectors[rightvectorcount as usize] as *const _, width * 0.5, &mut right as *mut _);

		color.0[0] = light;
		color.0[1] = light;
		color.0[2] = light;
		color.0[3] = alpha;

		// Bottom right
		// VectorAdd(loc, right, point);
		points[0] = (*loc)[0] + right[0];
		points[1] = (*loc)[1] + right[1];
		points[2] = (*loc)[2] + right[2];
		points[3] = 0.0;

		// Top right
		// VectorAdd(loc2, right, point);
		points[4] = loc2[0] + right[0];
		points[5] = loc2[1] + right[1];
		points[6] = loc2[2] + right[2];
		points[7] = 0.0;

		// Top left
		// VectorSubtract(loc2, right, point);
		points[8] = loc2[0] - right[0] + ssfwdvector[0] * width * 0.15;
		points[9] = loc2[1] - right[1] + ssfwdvector[1] * width * 0.15;
		points[10] = loc2[2] - right[2];
		points[11] = 0.0;

		// Bottom left
		// VectorSubtract(loc, right, point);
		points[12] = (*loc)[0] - right[0];
		points[13] = (*loc)[1] - right[1];
		points[14] = (*loc)[2] - right[2];
		points[15] = 0.0;

		// Add the sprite to the render list.
		// SQuickSprite.Add(points, color, fog);
	}
}

fn RB_DrawVerticalSurfaceSprites(stage: *mut shaderStage_t, input: *mut shaderCommands_t) {
	unsafe {
		let mut curindex: c_int;
		let mut curvert: c_int;
		let mut dist: vec3_t = [0.0; 3];
		let mut triarea: f32;
		let mut vec1to2: vec2_t = [0.0; 2];
		let mut vec1to3: vec2_t = [0.0; 2];

		let mut v1: vec3_t = [0.0; 3];
		let mut v2: vec3_t = [0.0; 3];
		let mut v3: vec3_t = [0.0; 3];
		let mut a1: f32;
		let mut a2: f32;
		let mut a3: f32;
		let mut l1: f32;
		let mut l2: f32;
		let mut l3: f32;
		let mut fog1: vec2_t = [0.0; 2];
		let mut fog2: vec2_t = [0.0; 2];
		let mut fog3: vec2_t = [0.0; 2];
		let mut winddiff1: vec2_t = [0.0; 2];
		let mut winddiff2: vec2_t = [0.0; 2];
		let mut winddiff3: vec2_t = [0.0; 2];
		let mut windforce1: f32;
		let mut windforce2: f32;
		let mut windforce3: f32;

		let mut posi: f32;
		let mut posj: f32;
		let mut step: f32;
		let mut fa: f32;
		let mut fb: f32;
		let mut fc: f32;

		let mut curpoint: vec3_t = [0.0; 3];
		let mut width: f32;
		let mut height: f32;
		let mut alpha: f32;
		let mut alphapos: f32;
		let mut thisspritesfadestart: f32;
		let mut light: f32;

		let mut randomindex2: byte;

		let mut skew: vec2_t = [0.0, 0.0];
		let mut fogv: vec2_t = [0.0; 2];
		let mut winddiffv: vec2_t = [0.0; 2];
		let mut windforce: f32 = 0.0;
		let usewindpoint: qboolean = if curWindPointActive != 0 && (*(*stage).ss).wind > 0.0 { 1 } else { 0 };

		let cutdist = (*(*stage).ss).fadeMax * rangescalefactor;
		let cutdist2 = cutdist * cutdist;
		let fadedist = (*(*stage).ss).fadeDist * rangescalefactor;
		let fadedist2 = fadedist * fadedist;

		let inv_fadediff = 1.0 / (cutdist2 - fadedist2);

		// The faderange is the fraction amount it takes for these sprites to fade out, assuming an ideal fade range of 250
		let mut faderange = FADE_RANGE / (cutdist - fadedist);

		if faderange > 1.0 {
			// Don't want to force a new fade_rand
			faderange = 1.0;
		}

		// Quickly calc all the alphas and windstuff for each vertex
		curindex = 0;
		while curindex < (*input).numVertexes {
			VectorSubtract(&ssViewOrigin as *const _, (*(*input).xyz.add(curindex as usize)), &mut dist as *mut _);
			SSVertAlpha[curindex as usize] = 1.0 - (VectorLengthSquared(&dist as *const _) - fadedist2) * inv_fadediff;
			curindex += 1;
		}

		// Wind only needs initialization once per tess.
		if usewindpoint != 0 && tess.SSInitializedWind == 0 {
			curvert = 0;
			while curvert < (*input).numVertexes {
				// Calc wind at each point
				dist[0] = (*(*input).xyz.add(curvert as usize))[0] - curWindPoint[0];
				dist[1] = (*(*input).xyz.add(curvert as usize))[1] - curWindPoint[1];
				step = dist[0] * dist[0] + dist[1] * dist[1]; // dist squared

				if step >= (WINDPOINT_RADIUS * WINDPOINT_RADIUS) {
					// No wind
					SSVertWindDir[curvert as usize][0] = 0.0;
					SSVertWindDir[curvert as usize][1] = 0.0;
					SSVertWindForce[curvert as usize] = 0.0; // Should be < 1
				} else {
					if step < 1.0 {
						// Don't want to divide by zero
						SSVertWindDir[curvert as usize][0] = 0.0;
						SSVertWindDir[curvert as usize][1] = 0.0;
						SSVertWindForce[curvert as usize] = curWindPointForce * (*(*stage).ss).wind;
					} else {
						step = Q_rsqrt(step); // Equals 1 over the distance.
						SSVertWindDir[curvert as usize][0] = dist[0] * step;
						SSVertWindDir[curvert as usize][1] = dist[1] * step;
						step = 1.0 - (1.0 / (step * WINDPOINT_RADIUS)); // 1- (dist/maxradius) = a scale from 0 to 1 linearly dropping off
						SSVertWindForce[curvert as usize] = curWindPointForce * (*(*stage).ss).wind * step; // *step means divide by the distance.
					}
				}
				curvert += 1;
			}
			tess.SSInitializedWind = qtrue;
		}

		curindex = 0;
		while curindex < (*input).numIndexes - 2 {
			curvert = *(*input).indexes.add((curindex) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v1 as *mut _);
			if (*(*stage).ss).facing != 0 {
				// Hang down
				if (*(*input).normal.add(curvert as usize))[2] > -0.5 {
					curindex += 3;
					continue;
				}
			} else {
				// Point up
				if (*(*input).normal.add(curvert as usize))[2] < 0.5 {
					curindex += 3;
					continue;
				}
			}
			l1 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a1 = SSVertAlpha[curvert as usize];
			fog1[0] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize)));
			fog1[1] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize).add(1)));
			winddiff1[0] = SSVertWindDir[curvert as usize][0];
			winddiff1[1] = SSVertWindDir[curvert as usize][1];
			windforce1 = SSVertWindForce[curvert as usize];

			curvert = *(*input).indexes.add((curindex + 1) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v2 as *mut _);
			if (*(*stage).ss).facing != 0 {
				// Hang down
				if (*(*input).normal.add(curvert as usize))[2] > -0.5 {
					curindex += 3;
					continue;
				}
			} else {
				// Point up
				if (*(*input).normal.add(curvert as usize))[2] < 0.5 {
					curindex += 3;
					continue;
				}
			}
			l2 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a2 = SSVertAlpha[curvert as usize];
			fog2[0] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize)));
			fog2[1] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize).add(1)));
			winddiff2[0] = SSVertWindDir[curvert as usize][0];
			winddiff2[1] = SSVertWindDir[curvert as usize][1];
			windforce2 = SSVertWindForce[curvert as usize];

			curvert = *(*input).indexes.add((curindex + 2) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v3 as *mut _);
			if (*(*stage).ss).facing != 0 {
				// Hang down
				if (*(*input).normal.add(curvert as usize))[2] > -0.5 {
					curindex += 3;
					continue;
				}
			} else {
				// Point up
				if (*(*input).normal.add(curvert as usize))[2] < 0.5 {
					curindex += 3;
					continue;
				}
			}
			l3 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a3 = SSVertAlpha[curvert as usize];
			fog3[0] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize)));
			fog3[1] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize).add(1)));
			winddiff3[0] = SSVertWindDir[curvert as usize][0];
			winddiff3[1] = SSVertWindDir[curvert as usize][1];
			windforce3 = SSVertWindForce[curvert as usize];

			if a1 <= 0.0 && a2 <= 0.0 && a3 <= 0.0 {
				curindex += 3;
				continue;
			}

			// Find the area in order to calculate the stepsize
			vec1to2[0] = v2[0] - v1[0];
			vec1to2[1] = v2[1] - v1[1];
			vec1to3[0] = v3[0] - v1[0];
			vec1to3[1] = v3[1] - v1[1];

			// Now get the cross product of this sum.
			triarea = vec1to3[0] * vec1to2[1] - vec1to3[1] * vec1to2[0];
			triarea = triarea.abs();
			if triarea <= 1.0 {
				// Insanely small abhorrent triangle.
				curindex += 3;
				continue;
			}
			step = (*(*stage).ss).density * Q_rsqrt(triarea);

			randomindex = (v1[0] + v1[1] + v2[0] + v2[1] + v3[0] + v3[1]) as byte;
			randominterval = (((v1[0] + v2[1] + v3[2]) as i32) as byte) | 0x03; // Make sure the interval is at least 3, and always odd
			rightvectorcount = 0;

			posi = 0.0;
			while posi < 1.0 {
				posj = 0.0;
				while posj < (1.0 - posi) {
					fa = posi + randomchart[randomindex as usize] * step;
					randomindex = randomindex.wrapping_add(randominterval);

					fb = posj + randomchart[randomindex as usize] * step;
					randomindex = randomindex.wrapping_add(randominterval);

					rightvectorcount = (rightvectorcount + 1) & 3;

					if fa > 1.0 {
						posj += step;
						continue;
					}

					if fb > (1.0 - fa) {
						posj += step;
						continue;
					}

					fc = 1.0 - fa - fb;

					// total alpha, minus random factor so some things fade out sooner.
					alphapos = a1 * fa + a2 * fb + a3 * fc;

					// Note that the alpha at this point is a value from 1.0 to 0.0, but represents when to START fading
					thisspritesfadestart = faderange + (1.0 - faderange) * randomchart[randomindex as usize];
					randomindex = randomindex.wrapping_add(randominterval);

					// Find where the alpha is relative to the fadestart, and calc the real alpha to draw at.
					alpha = 1.0 - ((thisspritesfadestart - alphapos) / faderange);
					if alpha > 0.0 {
						if alpha > 1.0 {
							alpha = 1.0;
						}

						if SSUsingFog != 0 {
							fogv[0] = fog1[0] * fa + fog2[0] * fb + fog3[0] * fc;
							fogv[1] = fog1[1] * fa + fog2[1] * fb + fog3[1] * fc;
						}

						if usewindpoint != 0 {
							winddiffv[0] = winddiff1[0] * fa + winddiff2[0] * fb + winddiff3[0] * fc;
							winddiffv[1] = winddiff1[1] * fa + winddiff2[1] * fb + winddiff3[1] * fc;
							windforce = windforce1 * fa + windforce2 * fb + windforce3 * fc;
						}

						VectorScale(&v1 as *const _, fa, &mut curpoint as *mut _);
						VectorMA(&curpoint as *const _, fb, &v2 as *const _, &mut curpoint as *mut _);
						VectorMA(&curpoint as *const _, fc, &v3 as *const _, &mut curpoint as *mut _);

						light = l1 * fa + l2 * fb + l3 * fc;
						if SSAdditiveTransparency != 0 {
							// Additive transparency, scale light value
							// light *= alpha;
							light = (128.0 + (light * 0.5)) * alpha;
							alpha = 1.0;
						}

						randomindex2 = randomindex;
						width = (*(*stage).ss).width * (1.0 + ((*(*stage).ss).variance[0] * randomchart[randomindex2 as usize]));
						height = (*(*stage).ss).height * (1.0 + ((*(*stage).ss).variance[1] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);
						if randomchart[randomindex2 as usize] > 0.5 {
							randomindex2 = randomindex2.wrapping_add(1);
							width = -width;
						} else {
							randomindex2 = randomindex2.wrapping_add(1);
						}
						if (*(*stage).ss).fadeScale != 0.0 && alphapos < 1.0 {
							width *= 1.0 + ((*(*stage).ss).fadeScale * (1.0 - alphapos));
						}

						if (*(*stage).ss).vertSkew != 0.0 {
							// flrand(-vertskew, vertskew)
							skew[0] = height * (((*(*stage).ss).vertSkew * 2.0 * randomchart[randomindex2 as usize]) - (*(*stage).ss).vertSkew);
							randomindex2 = randomindex2.wrapping_add(1);
							skew[1] = height * (((*(*stage).ss).vertSkew * 2.0 * randomchart[randomindex2 as usize]) - (*(*stage).ss).vertSkew);
							randomindex2 = randomindex2.wrapping_add(1);
						}

						if usewindpoint != 0 && windforce > 0.0 && (*(*stage).ss).wind > 0.0 {
							if SSUsingFog != 0 {
								RB_VerticalSurfaceSpriteWindPoint(
									&curpoint as *const _,
									width,
									height,
									light as byte,
									(alpha * 255.0) as byte,
									(*(*stage).ss).wind,
									(*(*stage).ss).windIdle,
									&fogv as *const _,
									(*(*stage).ss).facing,
									&skew as *const _,
									&winddiffv as *const _,
									windforce,
								);
							} else {
								RB_VerticalSurfaceSpriteWindPoint(
									&curpoint as *const _,
									width,
									height,
									light as byte,
									(alpha * 255.0) as byte,
									(*(*stage).ss).wind,
									(*(*stage).ss).windIdle,
									core::ptr::null(),
									(*(*stage).ss).facing,
									&skew as *const _,
									&winddiffv as *const _,
									windforce,
								);
							}
						} else {
							if SSUsingFog != 0 {
								RB_VerticalSurfaceSprite(
									&curpoint as *const _,
									width,
									height,
									light as byte,
									(alpha * 255.0) as byte,
									(*(*stage).ss).wind,
									(*(*stage).ss).windIdle,
									&fogv as *const _,
									(*(*stage).ss).facing,
									&skew as *const _,
								);
							} else {
								RB_VerticalSurfaceSprite(
									&curpoint as *const _,
									width,
									height,
									light as byte,
									(alpha * 255.0) as byte,
									(*(*stage).ss).wind,
									(*(*stage).ss).windIdle,
									core::ptr::null(),
									(*(*stage).ss).facing,
									&skew as *const _,
								);
							}
						}

						totalsurfsprites += 1;
					}
					posj += step;
				}
				posi += step;
			}
			curindex += 3;
		}
	}
}

/////////////////////////////////////////////
// Oriented surface sprites

fn RB_OrientedSurfaceSprite(
	loc: *const vec3_t,
	width: f32,
	height: f32,
	light: byte,
	alpha: byte,
	fog: *const vec2_t,
	faceup: c_int,
) {
	let mut loc2: vec3_t = [0.0, 0.0, 0.0];
	let mut right: vec3_t = [0.0, 0.0, 0.0];
	let mut points: [f32; 16] = [0.0; 16];
	let mut color: color4ub_t = color4ub_t([light, light, light, alpha]);

	color.0[0] = light;
	color.0[1] = light;
	color.0[2] = light;
	color.0[3] = alpha;

	unsafe {
		if faceup != 0 {
			let mut width_mut = width * 0.5;
			let mut height_mut = height * 0.5;

			// Bottom right
			// VectorAdd(loc, right, point);
			points[0] = (*loc)[0] + width_mut;
			points[1] = (*loc)[1] - width_mut;
			points[2] = (*loc)[2] + 1.0;
			points[3] = 0.0;

			// Top right
			// VectorAdd(loc, right, point);
			points[4] = (*loc)[0] + width_mut;
			points[5] = (*loc)[1] + width_mut;
			points[6] = (*loc)[2] + 1.0;
			points[7] = 0.0;

			// Top left
			// VectorSubtract(loc, right, point);
			points[8] = (*loc)[0] - width_mut;
			points[9] = (*loc)[1] + width_mut;
			points[10] = (*loc)[2] + 1.0;
			points[11] = 0.0;

			// Bottom left
			// VectorSubtract(loc, right, point);
			points[12] = (*loc)[0] - width_mut;
			points[13] = (*loc)[1] - width_mut;
			points[14] = (*loc)[2] + 1.0;
			points[15] = 0.0;
		} else {
			VectorMA(loc, height, &ssViewUp as *const _, &mut loc2 as *mut _);
			VectorScale(&ssViewRight as *const _, width * 0.5, &mut right as *mut _);

			// Bottom right
			// VectorAdd(loc, right, point);
			points[0] = (*loc)[0] + right[0];
			points[1] = (*loc)[1] + right[1];
			points[2] = (*loc)[2] + right[2];
			points[3] = 0.0;

			// Top right
			// VectorAdd(loc2, right, point);
			points[4] = loc2[0] + right[0];
			points[5] = loc2[1] + right[1];
			points[6] = loc2[2] + right[2];
			points[7] = 0.0;

			// Top left
			// VectorSubtract(loc2, right, point);
			points[8] = loc2[0] - right[0];
			points[9] = loc2[1] - right[1];
			points[10] = loc2[2] - right[2];
			points[11] = 0.0;

			// Bottom left
			// VectorSubtract(loc, right, point);
			points[12] = (*loc)[0] - right[0];
			points[13] = (*loc)[1] - right[1];
			points[14] = (*loc)[2] - right[2];
			points[15] = 0.0;
		}

		// Add the sprite to the render list.
		// SQuickSprite.Add(points, color, fog);
	}
}

fn RB_DrawOrientedSurfaceSprites(stage: *mut shaderStage_t, input: *mut shaderCommands_t) {
	unsafe {
		let mut curindex: c_int;
		let mut curvert: c_int;
		let mut dist: vec3_t = [0.0; 3];
		let mut triarea: f32;
		let mut minnormal: f32;
		let mut vec1to2: vec2_t = [0.0; 2];
		let mut vec1to3: vec2_t = [0.0; 2];

		let mut v1: vec3_t = [0.0; 3];
		let mut v2: vec3_t = [0.0; 3];
		let mut v3: vec3_t = [0.0; 3];
		let mut a1: f32;
		let mut a2: f32;
		let mut a3: f32;
		let mut l1: f32;
		let mut l2: f32;
		let mut l3: f32;
		let mut fog1: vec2_t = [0.0; 2];
		let mut fog2: vec2_t = [0.0; 2];
		let mut fog3: vec2_t = [0.0; 2];

		let mut posi: f32;
		let mut posj: f32;
		let mut step: f32;
		let mut fa: f32;
		let mut fb: f32;
		let mut fc: f32;

		let mut curpoint: vec3_t = [0.0; 3];
		let mut width: f32;
		let mut height: f32;
		let mut alpha: f32;
		let mut alphapos: f32;
		let mut thisspritesfadestart: f32;
		let mut light: f32;
		let mut randomindex2: byte;
		let mut fogv: vec2_t = [0.0; 2];

		let cutdist = (*(*stage).ss).fadeMax * rangescalefactor;
		let cutdist2 = cutdist * cutdist;
		let fadedist = (*(*stage).ss).fadeDist * rangescalefactor;
		let fadedist2 = fadedist * fadedist;

		let inv_fadediff = 1.0 / (cutdist2 - fadedist2);

		// The faderange is the fraction amount it takes for these sprites to fade out, assuming an ideal fade range of 250
		let mut faderange = FADE_RANGE / (cutdist - fadedist);

		if faderange > 1.0 {
			// Don't want to force a new fade_rand
			faderange = 1.0;
		}

		if (*(*stage).ss).facing != 0 {
			// Faceup sprite.
			minnormal = 0.99;
		} else {
			// Normal oriented sprite
			minnormal = 0.5;
		}

		// Quickly calc all the alphas for each vertex
		curvert = 0;
		while curvert < (*input).numVertexes {
			// Calc alpha at each point
			VectorSubtract(&ssViewOrigin as *const _, (*(*input).xyz.add(curvert as usize)), &mut dist as *mut _);
			SSVertAlpha[curvert as usize] = 1.0 - (VectorLengthSquared(&dist as *const _) - fadedist2) * inv_fadediff;
			curvert += 1;
		}

		curindex = 0;
		while curindex < (*input).numIndexes - 2 {
			curvert = *(*input).indexes.add((curindex) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v1 as *mut _);
			if (*(*input).normal.add(curvert as usize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l1 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a1 = SSVertAlpha[curvert as usize];
			fog1[0] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize)));
			fog1[1] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize).add(1)));

			curvert = *(*input).indexes.add((curindex + 1) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v2 as *mut _);
			if (*(*input).normal.add(curvert as usize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l2 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a2 = SSVertAlpha[curvert as usize];
			fog2[0] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize)));
			fog2[1] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize).add(1)));

			curvert = *(*input).indexes.add((curindex + 2) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v3 as *mut _);
			if (*(*input).normal.add(curvert as usize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l3 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a3 = SSVertAlpha[curvert as usize];
			fog3[0] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize)));
			fog3[1] = *(((*tess.svars.texcoords[0]).add((curvert << 1) as usize).add(1)));

			if a1 <= 0.0 && a2 <= 0.0 && a3 <= 0.0 {
				curindex += 3;
				continue;
			}

			// Find the area in order to calculate the stepsize
			vec1to2[0] = v2[0] - v1[0];
			vec1to2[1] = v2[1] - v1[1];
			vec1to3[0] = v3[0] - v1[0];
			vec1to3[1] = v3[1] - v1[1];

			// Now get the cross product of this sum.
			triarea = vec1to3[0] * vec1to2[1] - vec1to3[1] * vec1to2[0];
			triarea = triarea.abs();
			if triarea <= 1.0 {
				// Insanely small abhorrent triangle.
				curindex += 3;
				continue;
			}
			step = (*(*stage).ss).density * Q_rsqrt(triarea);

			randomindex = (v1[0] + v1[1] + v2[0] + v2[1] + v3[0] + v3[1]) as byte;
			randominterval = (((v1[0] + v2[1] + v3[2]) as i32) as byte) | 0x03; // Make sure the interval is at least 3, and always odd

			posi = 0.0;
			while posi < 1.0 {
				posj = 0.0;
				while posj < (1.0 - posi) {
					fa = posi + randomchart[randomindex as usize] * step;
					randomindex = randomindex.wrapping_add(randominterval);
					if fa > 1.0 {
						posj += step;
						continue;
					}

					fb = posj + randomchart[randomindex as usize] * step;
					randomindex = randomindex.wrapping_add(randominterval);
					if fb > (1.0 - fa) {
						posj += step;
						continue;
					}

					fc = 1.0 - fa - fb;

					// total alpha, minus random factor so some things fade out sooner.
					alphapos = a1 * fa + a2 * fb + a3 * fc;

					// Note that the alpha at this point is a value from 1.0 to 0.0, but represents when to START fading
					thisspritesfadestart = faderange + (1.0 - faderange) * randomchart[randomindex as usize];
					randomindex = randomindex.wrapping_add(randominterval);

					// Find where the alpha is relative to the fadestart, and calc the real alpha to draw at.
					alpha = 1.0 - ((thisspritesfadestart - alphapos) / faderange);

					randomindex = randomindex.wrapping_add(randominterval);
					if alpha > 0.0 {
						if alpha > 1.0 {
							alpha = 1.0;
						}

						if SSUsingFog != 0 {
							fogv[0] = fog1[0] * fa + fog2[0] * fb + fog3[0] * fc;
							fogv[1] = fog1[1] * fa + fog2[1] * fb + fog3[1] * fc;
						}

						VectorScale(&v1 as *const _, fa, &mut curpoint as *mut _);
						VectorMA(&curpoint as *const _, fb, &v2 as *const _, &mut curpoint as *mut _);
						VectorMA(&curpoint as *const _, fc, &v3 as *const _, &mut curpoint as *mut _);

						light = l1 * fa + l2 * fb + l3 * fc;
						if SSAdditiveTransparency != 0 {
							// Additive transparency, scale light value
							// light *= alpha;
							light = (128.0 + (light * 0.5)) * alpha;
							alpha = 1.0;
						}

						randomindex2 = randomindex;
						width = (*(*stage).ss).width * (1.0 + ((*(*stage).ss).variance[0] * randomchart[randomindex2 as usize]));
						height = (*(*stage).ss).height * (1.0 + ((*(*stage).ss).variance[1] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);
						if randomchart[randomindex2 as usize] > 0.5 {
							randomindex2 = randomindex2.wrapping_add(1);
							width = -width;
						} else {
							randomindex2 = randomindex2.wrapping_add(1);
						}
						if (*(*stage).ss).fadeScale != 0.0 && alphapos < 1.0 {
							width *= 1.0 + ((*(*stage).ss).fadeScale * (1.0 - alphapos));
						}

						if SSUsingFog != 0 {
							RB_OrientedSurfaceSprite(&curpoint as *const _, width, height, light as byte, (alpha * 255.0) as byte, &fogv as *const _, (*(*stage).ss).facing);
						} else {
							RB_OrientedSurfaceSprite(&curpoint as *const _, width, height, light as byte, (alpha * 255.0) as byte, core::ptr::null(), (*(*stage).ss).facing);
						}

						totalsurfsprites += 1;
					}
					posj += step;
				}
				posi += step;
			}
			curindex += 3;
		}
	}
}

/////////////////////////////////////////////
// Effect surface sprites

fn RB_EffectSurfaceSprite(
	loc: *const vec3_t,
	width: f32,
	height: f32,
	light: byte,
	alpha: byte,
	life: f32,
	faceup: c_int,
) {
	let mut loc2: vec3_t = [0.0, 0.0, 0.0];
	let mut right: vec3_t = [0.0, 0.0, 0.0];
	let mut points: [f32; 16] = [0.0; 16];
	let mut color: color4ub_t = color4ub_t([light, light, light, alpha]);

	color.0[0] = light; //light;
	color.0[1] = light; //light;
	color.0[2] = light; //light;
	color.0[3] = alpha; //alpha;

	unsafe {
		if faceup != 0 {
			let mut width_mut = width * 0.5;
			let mut height_mut = height * 0.5;

			// Bottom right
			// VectorAdd(loc, right, point);
			points[0] = (*loc)[0] + width_mut;
			points[1] = (*loc)[1] - width_mut;
			points[2] = (*loc)[2] + 1.0;
			points[3] = 0.0;

			// Top right
			// VectorAdd(loc, right, point);
			points[4] = (*loc)[0] + width_mut;
			points[5] = (*loc)[1] + width_mut;
			points[6] = (*loc)[2] + 1.0;
			points[7] = 0.0;

			// Top left
			// VectorSubtract(loc, right, point);
			points[8] = (*loc)[0] - width_mut;
			points[9] = (*loc)[1] + width_mut;
			points[10] = (*loc)[2] + 1.0;
			points[11] = 0.0;

			// Bottom left
			// VectorSubtract(loc, right, point);
			points[12] = (*loc)[0] - width_mut;
			points[13] = (*loc)[1] - width_mut;
			points[14] = (*loc)[2] + 1.0;
			points[15] = 0.0;
		} else {
			VectorMA(loc, height, &ssViewUp as *const _, &mut loc2 as *mut _);
			VectorScale(&ssViewRight as *const _, width * 0.5, &mut right as *mut _);

			// Bottom right
			// VectorAdd(loc, right, point);
			points[0] = (*loc)[0] + right[0];
			points[1] = (*loc)[1] + right[1];
			points[2] = (*loc)[2] + right[2];
			points[3] = 0.0;

			// Top right
			// VectorAdd(loc2, right, point);
			points[4] = loc2[0] + right[0];
			points[5] = loc2[1] + right[1];
			points[6] = loc2[2] + right[2];
			points[7] = 0.0;

			// Top left
			// VectorSubtract(loc2, right, point);
			points[8] = loc2[0] - right[0];
			points[9] = loc2[1] - right[1];
			points[10] = loc2[2] - right[2];
			points[11] = 0.0;

			// Bottom left
			// VectorSubtract(loc, right, point);
			points[12] = (*loc)[0] - right[0];
			points[13] = (*loc)[1] - right[1];
			points[14] = (*loc)[2] - right[2];
			points[15] = 0.0;
		}

		// Add the sprite to the render list.
		// SQuickSprite.Add(points, color, NULL);
	}
}

const SURFSPRITE_WEATHERFX: c_int = 2;

fn RB_DrawEffectSurfaceSprites(stage: *mut shaderStage_t, input: *mut shaderCommands_t) {
	unsafe {
		let mut curindex: c_int;
		let mut curvert: c_int;
		let mut dist: vec3_t = [0.0; 3];
		let mut triarea: f32;
		let mut minnormal: f32;
		let mut vec1to2: vec2_t = [0.0; 2];
		let mut vec1to3: vec2_t = [0.0; 2];

		let mut v1: vec3_t = [0.0; 3];
		let mut v2: vec3_t = [0.0; 3];
		let mut v3: vec3_t = [0.0; 3];
		let mut a1: f32;
		let mut a2: f32;
		let mut a3: f32;
		let mut l1: f32;
		let mut l2: f32;
		let mut l3: f32;

		let mut posi: f32;
		let mut posj: f32;
		let mut step: f32;
		let mut fa: f32;
		let mut fb: f32;
		let mut fc: f32;
		let mut effecttime: f32;
		let mut effectpos: f32;
		let mut density: f32;

		let mut curpoint: vec3_t = [0.0; 3];
		let mut width: f32;
		let mut height: f32;
		let mut alpha: f32;
		let mut alphapos: f32;
		let mut thisspritesfadestart: f32;
		let mut light: f32;
		let mut randomindex2: byte;

		let cutdist = (*(*stage).ss).fadeMax * rangescalefactor;
		let cutdist2 = cutdist * cutdist;
		let fadedist = (*(*stage).ss).fadeDist * rangescalefactor;
		let fadedist2 = fadedist * fadedist;

		let fxalpha = (*(*stage).ss).fxAlphaEnd - (*(*stage).ss).fxAlphaStart;
		let mut fadeinout: qboolean = qfalse;

		let inv_fadediff = 1.0 / (cutdist2 - fadedist2);

		// The faderange is the fraction amount it takes for these sprites to fade out, assuming an ideal fade range of 250
		let mut faderange = FADE_RANGE / (cutdist - fadedist);
		if faderange > 1.0 {
			// Don't want to force a new fade_rand
			faderange = 1.0;
		}

		if (*(*stage).ss).facing != 0 {
			// Faceup sprite.
			minnormal = 0.99;
		} else {
			// Normal oriented sprite
			minnormal = 0.5;
		}

		// Make the object fade in.
		if (*(*stage).ss).fxAlphaEnd < 0.05 && (*(*stage).ss).height >= 0.1 && (*(*stage).ss).width >= 0.1 {
			// The sprite fades out, and it doesn't start at a pinpoint.  Let's fade it in.
			fadeinout = qtrue;
		}

		if (*(*stage).ss).surfaceSpriteType == SURFSPRITE_WEATHERFX {
			// This effect is affected by weather settings.
			if curWeatherAmount < 0.01 {
				// Don't show these effects
				return;
			} else {
				density = (*(*stage).ss).density / curWeatherAmount;
			}
		} else {
			density = (*(*stage).ss).density;
		}

		// Quickly calc all the alphas for each vertex
		curvert = 0;
		while curvert < (*input).numVertexes {
			// Calc alpha at each point
			VectorSubtract(&ssViewOrigin as *const _, (*(*input).xyz.add(curvert as usize)), &mut dist as *mut _);
			SSVertAlpha[curvert as usize] = 1.0 - (VectorLengthSquared(&dist as *const _) - fadedist2) * inv_fadediff;

			// Note this is the proper equation, but isn't used right now because it would be just a tad slower.
			// Formula for alpha is 1.0f - ((len-fade)/(cut-fade))
			// Which is equal to (1.0+fade/(cut-fade)) - (len/(cut-fade))
			// So mult=1/(cut-fade), and base=(1+fade*mult).
			// SSVertAlpha[curvert] = fadebase - (VectorLength(dist) * fademult);

			curvert += 1;
		}

		curindex = 0;
		while curindex < (*input).numIndexes - 2 {
			curvert = *(*input).indexes.add((curindex) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v1 as *mut _);
			if (*(*input).normal.add(curvert as usize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l1 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a1 = SSVertAlpha[curvert as usize];

			curvert = *(*input).indexes.add((curindex + 1) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v2 as *mut _);
			if (*(*input).normal.add(curvert as usize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l2 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a2 = SSVertAlpha[curvert as usize];

			curvert = *(*input).indexes.add((curindex + 2) as usize);
			VectorCopy((*(*input).xyz.add(curvert as usize)), &mut v3 as *mut _);
			if (*(*input).normal.add(curvert as usize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l3 = (*(*input).vertexColors.add(curvert as usize))[2] as f32;
			a3 = SSVertAlpha[curvert as usize];

			if a1 <= 0.0 && a2 <= 0.0 && a3 <= 0.0 {
				curindex += 3;
				continue;
			}

			// Find the area in order to calculate the stepsize
			vec1to2[0] = v2[0] - v1[0];
			vec1to2[1] = v2[1] - v1[1];
			vec1to3[0] = v3[0] - v1[0];
			vec1to3[1] = v3[1] - v1[1];

			// Now get the cross product of this sum.
			triarea = vec1to3[0] * vec1to2[1] - vec1to3[1] * vec1to2[0];
			triarea = triarea.abs();
			if triarea <= 1.0 {
				// Insanely small abhorrent triangle.
				curindex += 3;
				continue;
			}
			step = density * Q_rsqrt(triarea);

			randomindex = (v1[0] + v1[1] + v2[0] + v2[1] + v3[0] + v3[1]) as byte;
			randominterval = (((v1[0] + v2[1] + v3[2]) as i32) as byte) | 0x03; // Make sure the interval is at least 3, and always odd

			posi = 0.0;
			while posi < 1.0 {
				posj = 0.0;
				while posj < (1.0 - posi) {
					effecttime = (backEnd.refdef.time as f32 + 10000.0 * randomchart[randomindex as usize]) / (*(*stage).ss).fxDuration;
					effectpos = effecttime - (effecttime as i32) as f32;

					randomindex2 = (randomindex as i32 + effecttime as i32) as byte;
					randomindex = randomindex.wrapping_add(randominterval);
					fa = posi + randomchart[randomindex2 as usize] * step;
					randomindex2 = randomindex2.wrapping_add(1);
					if fa > 1.0 {
						posj += step;
						continue;
					}

					fb = posj + randomchart[randomindex2 as usize] * step;
					randomindex2 = randomindex2.wrapping_add(1);
					if fb > (1.0 - fa) {
						posj += step;
						continue;
					}

					fc = 1.0 - fa - fb;

					// total alpha, minus random factor so some things fade out sooner.
					alphapos = a1 * fa + a2 * fb + a3 * fc;

					// Note that the alpha at this point is a value from 1.0f to 0.0, but represents when to START fading
					thisspritesfadestart = faderange + (1.0 - faderange) * randomchart[randomindex2 as usize];
					randomindex2 = randomindex2.wrapping_add(randominterval);

					// Find where the alpha is relative to the fadestart, and calc the real alpha to draw at.
					alpha = 1.0 - ((thisspritesfadestart - alphapos) / faderange);
					if alpha > 0.0 {
						if alpha > 1.0 {
							alpha = 1.0;
						}

						VectorScale(&v1 as *const _, fa, &mut curpoint as *mut _);
						VectorMA(&curpoint as *const _, fb, &v2 as *const _, &mut curpoint as *mut _);
						VectorMA(&curpoint as *const _, fc, &v3 as *const _, &mut curpoint as *mut _);

						light = l1 * fa + l2 * fb + l3 * fc;
						randomindex2 = randomindex;
						width = (*(*stage).ss).width * (1.0 + ((*(*stage).ss).variance[0] * randomchart[randomindex2 as usize]));
						height = (*(*stage).ss).height * (1.0 + ((*(*stage).ss).variance[1] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);

						width = width + (effectpos * (*(*stage).ss).fxGrow[0] * width);
						height = height + (effectpos * (*(*stage).ss).fxGrow[1] * height);

						// If we want to fade in and out, that's different than a straight fade.
						if fadeinout != 0 {
							if effectpos > 0.5 {
								// Fade out
								alpha = alpha * ((*(*stage).ss).fxAlphaStart + (fxalpha * (effectpos - 0.5) * 2.0));
							} else {
								// Fade in
								alpha = alpha * ((*(*stage).ss).fxAlphaStart + (fxalpha * (0.5 - effectpos) * 2.0));
							}
						} else {
							// Normal fade
							alpha = alpha * ((*(*stage).ss).fxAlphaStart + (fxalpha * effectpos));
						}

						if SSAdditiveTransparency != 0 {
							// Additive transparency, scale light value
							// light *= alpha;
							light = (128.0 + (light * 0.5)) * alpha;
							alpha = 1.0;
						}

						if randomchart[randomindex2 as usize] > 0.5 {
							randomindex2 = randomindex2.wrapping_add(1);
							width = -width;
						} else {
							randomindex2 = randomindex2.wrapping_add(1);
						}
						if (*(*stage).ss).fadeScale != 0.0 && alphapos < 1.0 {
							width *= 1.0 + ((*(*stage).ss).fadeScale * (1.0 - alphapos));
						}

						if (*(*stage).ss).wind > 0.0 && curWindSpeed > 0.001 {
							let mut drawpoint: vec3_t = [0.0; 3];

							VectorMA(&curpoint as *const _, effectpos * (*(*stage).ss).wind, &curWindBlowVect as *const _, &mut drawpoint as *mut _);
							RB_EffectSurfaceSprite(&drawpoint as *const _, width, height, light as byte, (alpha * 255.0) as byte, (*(*stage).ss).fxDuration, (*(*stage).ss).facing);
						} else {
							RB_EffectSurfaceSprite(&curpoint as *const _, width, height, light as byte, (alpha * 255.0) as byte, (*(*stage).ss).fxDuration, (*(*stage).ss).facing);
						}

						totalsurfsprites += 1;
					}
					posj += step;
				}
				posi += step;
			}
			curindex += 3;
		}
	}
}

extern "C" {
	fn R_WorldToLocal(world: *mut vec3_t, localVec: *mut vec3_t);
	// extern float preTransEntMatrix[16], invEntMatrix[16];
	fn R_InvertMatrix(sourcemat: *mut f32, destmat: *mut f32);
}

pub fn RB_DrawSurfaceSprites(stage: *mut shaderStage_t, input: *mut shaderCommands_t) {
	unsafe {
		let glbits: c_uint = (*stage).stateBits;

		R_SurfaceSpriteFrameUpdate();

		//
		// Check fog
		//
		if tess.SSInitializedWind != 0 && (*tess.svars.texcoords[0] as *const _) as *const _ != core::ptr::null() {
			SSUsingFog = qtrue;
			// SQuickSprite.StartGroup(&stage->bundle[0], glbits, tess.fogNum);
		} else {
			SSUsingFog = qfalse;
			// SQuickSprite.StartGroup(&stage->bundle[0], glbits);
		}

		// Special provision in case the transparency is additive.
		const GLS_SRCBLEND_BITS: c_uint = 0x0000000f;
		const GLS_DSTBLEND_BITS: c_uint = 0x000000f0;
		const GLS_SRCBLEND_ONE: c_uint = 0x00000001;
		const GLS_DSTBLEND_ONE: c_uint = 0x00000010;

		if (glbits & (GLS_SRCBLEND_BITS | GLS_DSTBLEND_BITS)) == (GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE) {
			// Additive transparency, scale light value
			SSAdditiveTransparency = qtrue;
		} else {
			SSAdditiveTransparency = qfalse;
		}

		//Check if this is a new entity transformation (incl. world entity), and update the appropriate vectors if so.
		if backEnd.currentEntity != ssLastEntityDrawn {
			if backEnd.currentEntity == &tr.worldEntity as *const _ as *mut _ {
				// Drawing the world, so our job is dead-easy, in the viewparms
				VectorCopy(&backEnd.viewParms.ori.origin as *const _, &mut ssViewOrigin as *mut _);
				VectorCopy(&backEnd.viewParms.ori.axis[1] as *const _, &mut ssViewRight as *mut _);
				VectorCopy(&backEnd.viewParms.ori.axis[2] as *const _, &mut ssViewUp as *mut _);
			} else {
				// Drawing an entity, so we need to transform the viewparms to the model's coordinate system
				// R_WorldPointToEntity (backEnd.viewParms.ori.origin, ssViewOrigin);
				R_WorldNormalToEntity(&mut backEnd.viewParms.ori.axis[1], &mut ssViewRight);
				R_WorldNormalToEntity(&mut backEnd.viewParms.ori.axis[2], &mut ssViewUp);
				VectorCopy(&backEnd.ori.viewOrigin as *const _, &mut ssViewOrigin as *mut _);
				// R_WorldToLocal(backEnd.viewParms.ori.axis[1], ssViewRight);
				// R_WorldToLocal(backEnd.viewParms.ori.axis[2], ssViewUp);
			}
			ssLastEntityDrawn = backEnd.currentEntity;
		}

		const SURFSPRITE_VERTICAL: c_int = 0;
		const SURFSPRITE_ORIENTED: c_int = 1;
		const SURFSPRITE_EFFECT: c_int = 2;

		match (*(*stage).ss).surfaceSpriteType {
			SURFSPRITE_VERTICAL => {
				RB_DrawVerticalSurfaceSprites(stage, input);
			}
			SURFSPRITE_ORIENTED => {
				RB_DrawOrientedSurfaceSprites(stage, input);
			}
			SURFSPRITE_EFFECT | SURFSPRITE_WEATHERFX => {
				RB_DrawEffectSurfaceSprites(stage, input);
			}
			_ => {}
		}

		// SQuickSprite.EndGroup();

		sssurfaces += 1;
	}
}
