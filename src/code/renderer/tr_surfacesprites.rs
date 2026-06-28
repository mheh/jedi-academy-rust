// tr_surfacesprites.c

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_uint, c_char};
use std::f32::consts::PI as M_PI;

// Forward declarations and external types
// These would be defined in their respective header files
pub type vec3_t = [f32; 3];
pub type vec2_t = [f32; 2];
pub type byte = u8;
pub type qboolean = bool;
pub type color4ub_t = [u8; 4];

const SHADER_MAX_VERTEXES: usize = 4000; // Common Quake3 constant

// External C functions and types (declared but not defined in this module)
extern "C" {
    // tr_QuickSprite.h
    pub static mut SQuickSprite: QuickSprite;

    // tr_worldeffects.h
    fn R_IsRaining() -> qboolean;
    fn R_IsSnowing() -> qboolean;
    fn R_IsPuffing() -> qboolean;
    fn R_GetWindSpeed(speed: *mut f32, unk: *mut core::ffi::c_void) -> qboolean;
    fn R_GetWindVector(vec: *mut vec3_t, unk: *mut core::ffi::c_void) -> qboolean;

    // math functions
    fn tan(x: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn pow(x: f64, y: f64) -> f64;
    fn fabsf(x: f32) -> f32;
    fn fabs(x: f64) -> f64;

    // renderer functions
    fn Com_Printf(msg: *const c_char, ...) -> ();
    fn Q_flrand(lower: f32, upper: f32) -> f32;
    fn Q_rsqrt(x: f32) -> f32;
    fn vectoangles(vec: *const vec3_t, angles: *mut vec3_t) -> ();
    fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t) -> ();
    fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t) -> ();
    fn VectorScale(v: *const vec3_t, s: f32, out: *mut vec3_t) -> ();
    fn VectorMA(v1: *const vec3_t, s: f32, v2: *const vec3_t, out: *mut vec3_t) -> ();
    fn VectorSubtract(v1: *const vec3_t, v2: *const vec3_t, out: *mut vec3_t) -> ();
    fn VectorLengthSquared(v: *const vec3_t) -> f32;
    fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, out: *mut vec3_t) -> ();
    fn R_WorldNormalToEntity(world: *const vec3_t, local: *mut vec3_t) -> ();

    pub static mut backEnd: BackEnd;
    pub static mut tr: Tr;
    pub static mut tess: Tess;
    pub static r_windSpeed: *mut cvar_t;
    pub static r_windGust: *mut cvar_t;
    pub static r_windAngle: *mut cvar_t;
    pub static r_windDampFactor: *mut cvar_t;
    pub static r_windPointForce: *mut cvar_t;
    pub static r_windPointX: *mut cvar_t;
    pub static r_windPointY: *mut cvar_t;
    pub static r_surfaceWeather: *mut cvar_t;
    pub static r_surfaceSprites: *mut cvar_t;
    pub static r_drawfog: *mut cvar_t;
}

// Type stubs for external structures
#[repr(C)]
pub struct QuickSprite {
    // Stub - not fully defined here
    _placeholder: [u8; 1],
}

impl QuickSprite {
    pub fn Add(&mut self, points: *const f32, color: *const color4ub_t, fog: *const vec2_t) {}
    pub fn StartGroup(&mut self, bundle: *mut core::ffi::c_void, glbits: c_uint) {}
    pub fn EndGroup(&mut self) {}
}

#[repr(C)]
pub struct BackEnd {
    refdef: RefDef,
    viewParms: ViewParms,
    currentEntity: *mut trRefEntity_t,
    ori: Orientation,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct RefDef {
    time: c_int,
    fov_x: f32,
    fov_y: f32,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct ViewParms {
    or: Orientation,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct Orientation {
    origin: vec3_t,
    axis: [[f32; 3]; 3],
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct Tr {
    refdef: RefDef,
    worldEntity: trRefEntity_t,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct Tess {
    svars: SvarsExt,
    fogNum: c_int,
    shader: *mut Shader,
    SSInitializedWind: qboolean,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct SvarsExt {
    texcoords: [[*mut f32; 2]; 256],
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct Shader {
    fogPass: qboolean,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct trRefEntity_t {
    // Stub
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct cvar_t {
    value: f32,
    integer: c_int,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct shaderStage_t {
    stateBits: c_uint,
    bundle: [ShaderBundle; 2],
    ss: *mut surfaceSprite_t,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct ShaderBundle {
    // Stub
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct surfaceSprite_t {
    width: f32,
    height: f32,
    density: f32,
    variance: [f32; 2],
    wind: f32,
    windIdle: f32,
    facing: c_int,
    fadeMax: f32,
    fadeDist: f32,
    fadeScale: f32,
    vertSkew: f32,
    fxAlphaStart: f32,
    fxAlphaEnd: f32,
    fxDuration: f32,
    fxGrow: [f32; 2],
    surfaceSpriteType: c_int,
    // More fields...
    _placeholder: [u8; 1],
}

#[repr(C)]
pub struct shaderCommands_t {
    xyz: *mut vec3_t,
    normal: *mut vec3_t,
    numVertexes: c_int,
    vertexColors: *mut color4ub_t,
    indexes: *mut c_int,
    numIndexes: c_int,
    // More fields...
    _placeholder: [u8; 1],
}

// Constants
const WIND_DAMP_INTERVAL: i32 = 50;
const WIND_GUST_TIME: f32 = 2500.0;
const WIND_GUST_DECAY: f32 = 1.0 / WIND_GUST_TIME;

const FADE_RANGE: f32 = 250.0;
const WINDPOINT_RADIUS: f32 = 750.0;

const SURFSPRITE_FLATTENED: c_int = 0;
const SURFSPRITE_VERTICAL: c_int = 1;
const SURFSPRITE_ORIENTED: c_int = 2;
const SURFSPRITE_EFFECT: c_int = 3;
const SURFSPRITE_WEATHERFX: c_int = 4;

const YAW: usize = 0;
const PITCH: usize = 1;
const ROLL: usize = 2;

const DEG2RAD: f32 = M_PI / 180.0;

// Macros translated to inline functions
#[inline]
fn DEG2RAD_CONVERT(deg: f32) -> f32 {
    deg * (M_PI / 180.0)
}

// include tr_QuickSprite.h
// include tr_worldeffects.h

/////===== Part of the VERTIGON system =====/////
// The surfacesprites are a simple system.  When a polygon with this shader stage on it is drawn,
// there are randomly distributed images (defined by the shader stage) placed on the surface.
// these are capable of doing effects, grass, or simple oriented sprites.
// They usually stick vertically off the surface, hence the term vertigons.

// The vertigons are applied as part of the renderer backend.  That is, they access OpenGL calls directly.


pub static mut randomindex: byte = 0;
pub static mut randominterval: byte = 0;

pub static randomchart: [f32; 256] = [
	0.6554, 0.6909, 0.4806, 0.6218, 0.5717, 0.3896, 0.0677, 0.7356,
	0.8333, 0.1105, 0.4445, 0.8161, 0.4689, 0.0433, 0.7152, 0.0336,
	0.0186, 0.9140, 0.1626, 0.6553, 0.8340, 0.7094, 0.2020, 0.8087,
	0.9119, 0.8009, 0.1339, 0.8492, 0.9173, 0.5003, 0.6012, 0.6117,
	0.5525, 0.5787, 0.1586, 0.3293, 0.9273, 0.7791, 0.8589, 0.4985,
	0.0883, 0.8545, 0.2634, 0.4727, 0.3624, 0.1631, 0.7825, 0.0662,
	0.6704, 0.3510, 0.7525, 0.9486, 0.4685, 0.1535, 0.1545, 0.1121,
	0.4724, 0.8483, 0.3833, 0.1917, 0.8207, 0.3885, 0.9702, 0.9200,
	0.8348, 0.7501, 0.6675, 0.4994, 0.0301, 0.5225, 0.8011, 0.1696,
	0.5351, 0.2752, 0.2962, 0.7550, 0.5762, 0.7303, 0.2835, 0.4717,
	0.1818, 0.2739, 0.6914, 0.7748, 0.7640, 0.8355, 0.7314, 0.5288,
	0.7340, 0.6692, 0.6813, 0.2810, 0.8057, 0.0648, 0.8749, 0.9199,
	0.1462, 0.5237, 0.3014, 0.4994, 0.0278, 0.4268, 0.7238, 0.5107,
	0.1378, 0.7303, 0.7200, 0.3819, 0.2034, 0.7157, 0.5552, 0.4887,
	0.0871, 0.3293, 0.2892, 0.4545, 0.0088, 0.1404, 0.0275, 0.0238,
	0.0515, 0.4494, 0.7206, 0.2893, 0.6060, 0.5785, 0.4182, 0.5528,
	0.9118, 0.8742, 0.3859, 0.6030, 0.3495, 0.4550, 0.9875, 0.6900,
	0.6416, 0.2337, 0.7431, 0.9788, 0.6181, 0.2464, 0.4661, 0.7621,
	0.7020, 0.8203, 0.8869, 0.2145, 0.7724, 0.6093, 0.6692, 0.9686,
	0.5609, 0.0310, 0.2248, 0.2950, 0.2365, 0.1347, 0.2342, 0.1668,
	0.3378, 0.4330, 0.2775, 0.9901, 0.7053, 0.7266, 0.4840, 0.2820,
	0.5733, 0.4555, 0.6049, 0.0770, 0.4760, 0.6060, 0.4159, 0.3427,
	0.1234, 0.7062, 0.8569, 0.1878, 0.9057, 0.9399, 0.8139, 0.1407,
	0.1794, 0.9123, 0.9493, 0.2827, 0.9934, 0.0952, 0.4879, 0.5160,
	0.4118, 0.4873, 0.3642, 0.7470, 0.0866, 0.5172, 0.6365, 0.2676,
	0.2407, 0.7223, 0.5761, 0.1143, 0.7137, 0.2342, 0.3353, 0.6880,
	0.2296, 0.6023, 0.6027, 0.4138, 0.5408, 0.9859, 0.1503, 0.7238,
	0.6054, 0.2477, 0.6804, 0.1432, 0.4540, 0.9776, 0.8762, 0.7607,
	0.9025, 0.9807, 0.0652, 0.8661, 0.7663, 0.2586, 0.3994, 0.0335,
	0.7328, 0.0166, 0.9589, 0.4348, 0.5493, 0.7269, 0.6867, 0.6614,
	0.6800, 0.7804, 0.5591, 0.8381, 0.0910, 0.7573, 0.8985, 0.3083,
	0.3188, 0.8481, 0.2356, 0.6736, 0.4770, 0.4560, 0.6266, 0.4677,
];

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

pub static mut curWindPointActive: qboolean = false;
pub static mut curWindPointForce: f32 = 0.0;
pub static mut curWindPoint: vec3_t = [0.0, 0.0, 0.0];
pub static mut nextGustTime: c_int = 0;
pub static mut gustLeft: f32 = 0.0;

pub static mut standardfovinitialized: qboolean = false;
pub static mut standardfovx: f32 = 90.0;
pub static mut standardscalex: f32 = 1.0;
pub static mut rangescalefactor: f32 = 1.0;

pub static mut ssrightvectors: [vec3_t; 4] = [[0.0; 3]; 4];
pub static mut ssfwdvector: vec3_t = [0.0, 0.0, 0.0];
pub static mut rightvectorcount: c_int = 0;

pub static mut ssLastEntityDrawn: *mut trRefEntity_t = std::ptr::null_mut();
pub static mut ssViewOrigin: vec3_t = [0.0, 0.0, 0.0];
pub static mut ssViewRight: vec3_t = [0.0, 0.0, 0.0];
pub static mut ssViewUp: vec3_t = [0.0, 0.0, 0.0];


static fn R_SurfaceSpriteFrameUpdate() {
	let mut dtime: f32;
	let mut dampfactor: f32;	// Time since last update and damping time for wind changes
	let mut ratio: f32;
	let mut ang: vec3_t = [0.0; 3];
	let mut diff: vec3_t = [0.0; 3];
	let mut retwindvec: vec3_t = [0.0; 3];
	let mut targetspeed: f32;
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
			curWindGrassDir[0] = 0.0;
			curWindGrassDir[1] = 0.0;
			curWindGrassDir[2] = 0.0;
		}

		// Reset the last entity drawn, since this is a new frame.
		ssLastEntityDrawn = std::ptr::null_mut();

		// Adjust for an FOV.  If things look twice as wide on the screen, pretend the shaders have twice the range.
		// ASSUMPTION HERE IS THAT "standard" fov is the first one rendered.

		if !standardfovinitialized {
			// This isn't initialized yet.
			if backEnd.refdef.fov_x > 50.0 && backEnd.refdef.fov_x < 135.0 {
				// I don't consider anything below 50 or above 135 to be "normal".
				standardfovx = backEnd.refdef.fov_x;
				standardscalex = (tan((standardfovx * 0.5) as f64 * (M_PI / 180.0)) as f32);
				standardfovinitialized = true;
			} else {
				standardfovx = 90.0;
				standardscalex = (tan((standardfovx * 0.5) as f64 * (M_PI / 180.0)) as f32);
			}
			rangescalefactor = 1.0;		// Don't multiply the shader range by anything.
		} else if standardfovx == backEnd.refdef.fov_x {
			// This is the standard FOV (or higher), don't multiply the shader range.
			rangescalefactor = 1.0;
		} else {
			// We are using a non-standard FOV.  We need to multiply the range of the shader by a scale factor.
			if backEnd.refdef.fov_x > 135.0 {
				rangescalefactor = standardscalex / (tan(135.0 * 0.5 * (M_PI / 180.0)) as f32);
			} else {
				rangescalefactor = standardscalex / (tan((backEnd.refdef.fov_x * 0.5) as f64 * (M_PI / 180.0)) as f32);
			}
		}

		// Create a set of four right vectors so that vertical sprites aren't always facing the same way.
		// First generate a HORIZONTAL forward vector (important).
		CrossProduct(&ssViewRight, &up, &mut ssfwdvector);

		// Right Zero has a nudge forward (10 degrees).
		VectorScale(&ssViewRight, 0.985, &mut ssrightvectors[0]);
		VectorMA(&ssrightvectors[0], 0.174, &ssfwdvector, &mut ssrightvectors[0]);

		// Right One has a big nudge back (30 degrees).
		VectorScale(&ssViewRight, 0.866, &mut ssrightvectors[1]);
		VectorMA(&ssrightvectors[1], -0.5, &ssfwdvector, &mut ssrightvectors[1]);


		// Right two has a big nudge forward (30 degrees).
		VectorScale(&ssViewRight, 0.866, &mut ssrightvectors[2]);
		VectorMA(&ssrightvectors[2], 0.5, &ssfwdvector, &mut ssrightvectors[2]);


		// Right three has a nudge back (10 degrees).
		VectorScale(&ssViewRight, 0.985, &mut ssrightvectors[3]);
		VectorMA(&ssrightvectors[3], -0.174, &ssfwdvector, &mut ssrightvectors[3]);


		// Update the wind.
		// If it is raining, get the windspeed from the rain system rather than the cvar
		if R_IsRaining() /*|| R_IsSnowing()*/ || R_IsPuffing() {
			curWeatherAmount = 1.0;
		} else {
			curWeatherAmount = (*r_surfaceWeather).value;
		}

		targetspeed = 0.0;
		if R_GetWindSpeed(&mut targetspeed, std::ptr::null_mut()) {
			// We successfully got a speed from the rain system.
			// Set the windgust to 5, since that looks pretty good.
			targetspeed *= 0.02;
			if targetspeed >= 1.0 {
				curWindGust = 300.0 / targetspeed;
			} else {
				curWindGust = 0.0;
			}
		} else {
			// Use the cvar.
			targetspeed = (*r_windSpeed).value;	// Minimum gust delay, in seconds.
			curWindGust = (*r_windGust).value;
		}

		if targetspeed > 0.0 && curWindGust != 0.0 {
			if gustLeft > 0.0 {
				// We are gusting
				// Add an amount to the target wind speed
				targetspeed *= 1.0 + gustLeft;

				gustLeft -= (backEnd.refdef.time as f32 - lastSSUpdateTime as f32) * WIND_GUST_DECAY;
				if gustLeft <= 0.0 {
					nextGustTime = backEnd.refdef.time + ((curWindGust * 1000.0) * Q_flrand(1.0, 4.0)) as c_int;
				}
			} else if backEnd.refdef.time >= nextGustTime {
				// See if there is another right now
				// Gust next time, mano
				gustLeft = Q_flrand(0.75, 1.5);
			}
		}

		// See if there is a weather system that will tell us a windspeed.
		if R_GetWindVector(&mut retwindvec, std::ptr::null_mut()) {
			retwindvec[2] = 0.0;
			//VectorScale(retwindvec, -1.0f, retwindvec);
			vectoangles(&retwindvec, &mut ang);
		} else {
			// Calculate the target wind vector based off cvars
			ang[YAW] = (*r_windAngle).value;
		}

		ang[PITCH] = -90.0 + targetspeed;
		if ang[PITCH] > -45.0 {
			ang[PITCH] = -45.0;
		}
		ang[ROLL] = 0.0;

		if targetspeed > 0.0 {
	//		ang[YAW] += cos(tr.refdef.time*0.01+flrand(-1.0,1.0))*targetspeed*0.5;
	//		ang[PITCH] += sin(tr.refdef.time*0.01+flrand(-1.0,1.0))*targetspeed*0.5;
		}

		// Get the grass wind vector first
		AngleVectors(&ang, &mut targetWindGrassDir, std::ptr::null_mut(), std::ptr::null_mut());
		targetWindGrassDir[2] -= 1.0;
	//		VectorScale(targetWindGrassDir, targetspeed, targetWindGrassDir);

		// Now get the general wind vector (no pitch)
		ang[PITCH] = 0.0;
		AngleVectors(&ang, &mut targetWindBlowVect, std::ptr::null_mut(), std::ptr::null_mut());

		// Start calculating a smoothing factor so wind doesn't change abruptly between speeds.
		dampfactor = 1.0 - (*r_windDampFactor).value;	// We must exponent the amount LEFT rather than the amount bled off
		dtime = (backEnd.refdef.time as f32 - lastSSUpdateTime as f32) * (1.0 / (WIND_DAMP_INTERVAL as f32));	// Our dampfactor is geared towards a time interval equal to "1".

		// Note that since there are a finite number of "practical" delta millisecond values possible,
		// the ratio should be initialized into a chart ultimately.
		ratio = (pow(dampfactor as f64, dtime as f64) as f32);

		// Apply this ratio to the windspeed...
		if fabsf(targetspeed - curWindSpeed) > ratio {
			curWindSpeed = targetspeed - (ratio * (targetspeed - curWindSpeed));
		}


		// Use the curWindSpeed to calculate the final target wind vector (with speed)
		VectorScale(&targetWindBlowVect, curWindSpeed, &mut targetWindBlowVect);
		VectorSubtract(&targetWindBlowVect, &curWindBlowVect, &mut diff);
		VectorMA(&targetWindBlowVect, -ratio, &diff, &mut curWindBlowVect);

		// Update the grass vector now
		VectorSubtract(&targetWindGrassDir, &curWindGrassDir, &mut diff);
		VectorMA(&targetWindGrassDir, -ratio, &diff, &mut curWindGrassDir);

		lastSSUpdateTime = backEnd.refdef.time;

		if fabsf((*r_windPointForce).value - curWindPointForce) > ratio {
			// Make sure not to get infinitly small number here
			curWindPointForce = (*r_windPointForce).value - (ratio * ((*r_windPointForce).value - curWindPointForce));
		}
		debug_assert!(!curWindPointForce.is_nan());
		if curWindPointForce < 0.01 {
			curWindPointActive = false;
		} else {
			curWindPointActive = true;
			curWindPoint[0] = (*r_windPointX).value;
			curWindPoint[1] = (*r_windPointY).value;
			curWindPoint[2] = 0.0;
		}

		if (*r_surfaceSprites).integer >= 2 {
			Com_Printf(b"Surfacesprites Drawn: %d, on %d surfaces\n\0".as_ptr() as *const c_char, totalsurfsprites, sssurfaces);
		}

		totalsurfsprites = 0;
		sssurfaces = 0;
	}
}



/////////////////////////////////////////////
// Surface sprite calculation and drawing.
/////////////////////////////////////////////

pub static mut SSVertAlpha: [f32; SHADER_MAX_VERTEXES] = [0.0; SHADER_MAX_VERTEXES];
pub static mut SSVertWindForce: [f32; SHADER_MAX_VERTEXES] = [0.0; SHADER_MAX_VERTEXES];
pub static mut SSVertWindDir: [vec2_t; SHADER_MAX_VERTEXES] = [[0.0; 2]; SHADER_MAX_VERTEXES];

pub static mut SSAdditiveTransparency: qboolean = false;
pub static mut SSUsingFog: qboolean = false;


/////////////////////////////////////////////
// Vertical surface sprites

static fn RB_VerticalSurfaceSprite(loc: *const vec3_t, width: f32, height: f32, light: byte,
										alpha: byte, wind: f32, windidle: f32, fog: *const vec2_t, hangdown: c_int, skew: *const vec2_t, flattened: bool) {
	let mut loc2: vec3_t = [0.0; 3];
	let mut right: vec3_t = [0.0; 3];
	let mut angle: f32;
	let mut windsway: f32;
	let mut points: [f32; 16] = [0.0; 16];
	let mut color: color4ub_t = [0; 4];

	unsafe {
		angle = (((*loc)[0] + (*loc)[1]) * 0.02 + (backEnd.refdef.time as f32 * 0.0015));

		if windidle > 0.0 {
			windsway = (height * windidle * 0.075);
			loc2[0] = (*loc)[0] + (*skew)[0] + (cos(angle as f64) as f32) * windsway;
			loc2[1] = (*loc)[1] + (*skew)[1] + (sin(angle as f64) as f32) * windsway;

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
			windsway = (height * wind * 0.075);

			// Add the angle
			VectorMA(&loc2, height * wind, &curWindGrassDir, &mut loc2);
			// Bob up and down
			if curWindSpeed < 40.0 {
				windsway *= curWindSpeed * (1.0 / 100.0);
			} else {
				windsway *= 0.4;
			}
			loc2[2] += (sin((angle * 2.5) as f64) as f32) * windsway;
		}

		if flattened {
			right[0] = (sin(DEG2RAD_CONVERT((*loc)[0]) as f64) as f32) * width;
			right[1] = (cos(DEG2RAD_CONVERT((*loc)[0]) as f64) as f32) * height;
			right[2] = 0.0;
		} else {
			VectorScale(&ssrightvectors[rightvectorcount as usize], width * 0.5, &mut right);
		}

		color[0] = light;
		color[1] = light;
		color[2] = light;
		color[3] = alpha;

		// Bottom right
	//	VectorAdd(loc, right, point);
		points[0] = (*loc)[0] + right[0];
		points[1] = (*loc)[1] + right[1];
		points[2] = (*loc)[2] + right[2];
		points[3] = 0.0;

		// Top right
	//	VectorAdd(loc2, right, point);
		points[4] = loc2[0] + right[0];
		points[5] = loc2[1] + right[1];
		points[6] = loc2[2] + right[2];
		points[7] = 0.0;

		// Top left
	//	VectorSubtract(loc2, right, point);
		points[8] = loc2[0] - right[0] + ssfwdvector[0] * width * 0.2;
		points[9] = loc2[1] - right[1] + ssfwdvector[1] * width * 0.2;
		points[10] = loc2[2] - right[2];
		points[11] = 0.0;

		// Bottom left
	//	VectorSubtract(loc, right, point);
		points[12] = (*loc)[0] - right[0];
		points[13] = (*loc)[1] - right[1];
		points[14] = (*loc)[2] - right[2];
		points[15] = 0.0;

		// Add the sprite to the render list.
		SQuickSprite.Add(points.as_ptr(), &color, fog);
	}
}

static fn RB_VerticalSurfaceSpriteWindPoint(loc: *const vec3_t, width: f32, height: f32, light: byte,
												alpha: byte, wind: f32, windidle: f32, fog: *const vec2_t,
												hangdown: c_int, skew: *const vec2_t, winddiff: *const vec2_t, windforce: f32, flattened: bool) {
	let mut loc2: vec3_t = [0.0; 3];
	let mut right: vec3_t = [0.0; 3];
	let mut angle: f32;
	let mut windsway: f32;
	let mut points: [f32; 16] = [0.0; 16];
	let mut color: color4ub_t = [0; 4];
	let mut windforce = windforce;

	unsafe {
		if windforce > 1.0 {
			windforce = 1.0;
		}

	//	wind += 1.0-windforce;

		angle = ((*loc)[0] + (*loc)[1]) * 0.02 + (backEnd.refdef.time as f32 * 0.0015);

		if curWindSpeed < 80.0 {
			windsway = (height * windidle * 0.1) * (1.0 + windforce);
			loc2[0] = (*loc)[0] + (*skew)[0] + (cos(angle as f64) as f32) * windsway;
			loc2[1] = (*loc)[1] + (*skew)[1] + (sin(angle as f64) as f32) * windsway;
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
			VectorMA(&loc2, height * wind, &curWindGrassDir, &mut loc2);
		}

		loc2[0] += height * (*winddiff)[0] * windforce;
		loc2[1] += height * (*winddiff)[1] * windforce;
		loc2[2] -= height * windforce * (0.75 + 0.15 * (sin(((backEnd.refdef.time as f32 + 500.0 * windforce) * 0.01) as f64) as f32));

		if flattened {
			right[0] = (sin(DEG2RAD_CONVERT((*loc)[0]) as f64) as f32) * width;
			right[1] = (cos(DEG2RAD_CONVERT((*loc)[0]) as f64) as f32) * height;
			right[2] = 0.0;
		} else {
			VectorScale(&ssrightvectors[rightvectorcount as usize], width * 0.5, &mut right);
		}


		color[0] = light;
		color[1] = light;
		color[2] = light;
		color[3] = alpha;

		// Bottom right
	//	VectorAdd(loc, right, point);
		points[0] = (*loc)[0] + right[0];
		points[1] = (*loc)[1] + right[1];
		points[2] = (*loc)[2] + right[2];
		points[3] = 0.0;

		// Top right
	//	VectorAdd(loc2, right, point);
		points[4] = loc2[0] + right[0];
		points[5] = loc2[1] + right[1];
		points[6] = loc2[2] + right[2];
		points[7] = 0.0;

		// Top left
	//	VectorSubtract(loc2, right, point);
		points[8] = loc2[0] - right[0] + ssfwdvector[0] * width * 0.15;
		points[9] = loc2[1] - right[1] + ssfwdvector[1] * width * 0.15;
		points[10] = loc2[2] - right[2];
		points[11] = 0.0;

		// Bottom left
	//	VectorSubtract(loc, right, point);
		points[12] = (*loc)[0] - right[0];
		points[13] = (*loc)[1] - right[1];
		points[14] = (*loc)[2] - right[2];
		points[15] = 0.0;

		// Add the sprite to the render list.
		SQuickSprite.Add(points.as_ptr(), &color, fog);
	}
}

static fn RB_DrawVerticalSurfaceSprites(stage: *mut shaderStage_t, input: *const shaderCommands_t) {
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

	let mut skew: vec2_t = [0.0; 2];
	let mut fogv: vec2_t = [0.0; 2];
	let mut winddiffv: vec2_t = [0.0; 2];
	let mut windforce: f32 = 0.0;
	let usewindpoint: bool = unsafe { curWindPointActive && (*stage).ss.is_null() == false && (*(*stage).ss).wind > 0.0 };

	unsafe {
		let cutdist: f32 = (*(*stage).ss).fadeMax * rangescalefactor;
		let cutdist2: f32 = cutdist * cutdist;
		let fadedist: f32 = (*(*stage).ss).fadeDist * rangescalefactor;
		let fadedist2: f32 = fadedist * fadedist;

		debug_assert!(cutdist2 != fadedist2);
		let inv_fadediff: f32 = 1.0 / (cutdist2 - fadedist2);

		// The faderange is the fraction amount it takes for these sprites to fade out, assuming an ideal fade range of 250
		let mut faderange: f32 = FADE_RANGE / (cutdist - fadedist);

		if faderange > 1.0 {
			// Don't want to force a new fade_rand
			faderange = 1.0;
		}

		// Quickly calc all the alphas and windstuff for each vertex
		for curvert in 0..(*input).numVertexes {
			VectorSubtract(&ssViewOrigin, &*((*input).xyz.offset(curvert as isize)), &mut dist);
			SSVertAlpha[curvert as usize] = 1.0 - (VectorLengthSquared(&dist) - fadedist2) * inv_fadediff;
		}

		// Wind only needs initialization once per tess.
		if usewindpoint && !tess.SSInitializedWind {
			for curvert in 0..(*input).numVertexes {
				// Calc wind at each point
				dist[0] = (*input).xyz.offset(curvert as isize).as_ref().unwrap()[0] - curWindPoint[0];
				dist[1] = (*input).xyz.offset(curvert as isize).as_ref().unwrap()[1] - curWindPoint[1];
				step = (dist[0] * dist[0] + dist[1] * dist[1]);	// dist squared

				if step >= (WINDPOINT_RADIUS * WINDPOINT_RADIUS) {
					// No wind
					SSVertWindDir[curvert as usize][0] = 0.0;
					SSVertWindDir[curvert as usize][1] = 0.0;
					SSVertWindForce[curvert as usize] = 0.0;		// Should be < 1
				} else {
					if step < 1.0 {
						// Don't want to divide by zero
						SSVertWindDir[curvert as usize][0] = 0.0;
						SSVertWindDir[curvert as usize][1] = 0.0;
						SSVertWindForce[curvert as usize] = curWindPointForce * (*(*stage).ss).wind;
					} else {
						step = Q_rsqrt(step);		// Equals 1 over the distance.
						SSVertWindDir[curvert as usize][0] = dist[0] * step;
						SSVertWindDir[curvert as usize][1] = dist[1] * step;
						step = 1.0 - (1.0 / (step * WINDPOINT_RADIUS));	// 1- (dist/maxradius) = a scale from 0 to 1 linearly dropping off
						SSVertWindForce[curvert as usize] = curWindPointForce * (*(*stage).ss).wind * step;	// *step means divide by the distance.
					}
				}
			}
			tess.SSInitializedWind = true;
		}

		curindex = 0;
		while curindex < (*input).numIndexes - 2 {
			curvert = *(*input).indexes.offset(curindex as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v1);
			if (*(*stage).ss).facing != 0 {
				// Hang down
				if (*(*input).normal.offset(curvert as isize))[2] > -0.5 {
					curindex += 3;
					continue;
				}
			} else {
				// Point up
				if (*(*input).normal.offset(curvert as isize))[2] < 0.5 {
					curindex += 3;
					continue;
				}
			}
			l1 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
			a1 = SSVertAlpha[curvert as usize];
			fog1[0] = *((tess.svars.texcoords[0][0]).offset((curvert << 1) as isize));
			fog1[1] = *((tess.svars.texcoords[0][0]).offset(((curvert << 1) + 1) as isize));
			winddiff1[0] = SSVertWindDir[curvert as usize][0];
			winddiff1[1] = SSVertWindDir[curvert as usize][1];
			windforce1 = SSVertWindForce[curvert as usize];

			curvert = *(*input).indexes.offset((curindex + 1) as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v2);
			if (*(*stage).ss).facing != 0 {
				// Hang down
				if (*(*input).normal.offset(curvert as isize))[2] > -0.5 {
					curindex += 3;
					continue;
				}
			} else {
				// Point up
				if (*(*input).normal.offset(curvert as isize))[2] < 0.5 {
					curindex += 3;
					continue;
				}
			}
			l2 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
			a2 = SSVertAlpha[curvert as usize];
			fog2[0] = *((tess.svars.texcoords[0][0]).offset((curvert << 1) as isize));
			fog2[1] = *((tess.svars.texcoords[0][0]).offset(((curvert << 1) + 1) as isize));
			winddiff2[0] = SSVertWindDir[curvert as usize][0];
			winddiff2[1] = SSVertWindDir[curvert as usize][1];
			windforce2 = SSVertWindForce[curvert as usize];

			curvert = *(*input).indexes.offset((curindex + 2) as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v3);
			if (*(*stage).ss).facing != 0 {
				// Hang down
				if (*(*input).normal.offset(curvert as isize))[2] > -0.5 {
					curindex += 3;
					continue;
				}
			} else {
				// Point up
				if (*(*input).normal.offset(curvert as isize))[2] < 0.5 {
					curindex += 3;
					continue;
				}
			}
			l3 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
			a3 = SSVertAlpha[curvert as usize];
			fog3[0] = *((tess.svars.texcoords[0][0]).offset((curvert << 1) as isize));
			fog3[1] = *((tess.svars.texcoords[0][0]).offset(((curvert << 1) + 1) as isize));
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
			triarea = fabs(triarea);
			if triarea <= 1.0 {
				// Insanely small abhorrent triangle.
				curindex += 3;
				continue;
			}
			step = (*(*stage).ss).density * Q_rsqrt(triarea);

			let mut randomindex_local: byte = ((v1[0] + v1[1] + v2[0] + v2[1] + v3[0] + v3[1]) as c_int) as byte;
			randominterval = (((v1[0] + v2[1] + v3[2]) as c_int) | 0x03) as byte;	// Make sure the interval is at least 3, and always odd
			rightvectorcount = 0;

			posi = 0.0;
			while posi < 1.0 {
				posj = 0.0;
				while posj < (1.0 - posi) {
					fa = posi + randomchart[randomindex_local as usize] * step;
					randomindex_local = randomindex_local.wrapping_add(randominterval);

					fb = posj + randomchart[randomindex_local as usize] * step;
					randomindex_local = randomindex_local.wrapping_add(randominterval);

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
					thisspritesfadestart = faderange + (1.0 - faderange) * randomchart[randomindex_local as usize];
					randomindex_local = randomindex_local.wrapping_add(randominterval);

					// Find where the alpha is relative to the fadestart, and calc the real alpha to draw at.
					alpha = 1.0 - ((thisspritesfadestart - alphapos) / faderange);
					if alpha > 0.0 {
						if alpha > 1.0 {
							alpha = 1.0;
						}

						if SSUsingFog {
							fogv[0] = fog1[0] * fa + fog2[0] * fb + fog3[0] * fc;
							fogv[1] = fog1[1] * fa + fog2[1] * fb + fog3[1] * fc;
						}

						if usewindpoint {
							winddiffv[0] = winddiff1[0] * fa + winddiff2[0] * fb + winddiff3[0] * fc;
							winddiffv[1] = winddiff1[1] * fa + winddiff2[1] * fb + winddiff3[1] * fc;
							windforce = windforce1 * fa + windforce2 * fb + windforce3 * fc;
						}

						VectorScale(&v1, fa, &mut curpoint);
						VectorMA(&curpoint, fb, &v2, &mut curpoint);
						VectorMA(&curpoint, fc, &v3, &mut curpoint);

						light = l1 * fa + l2 * fb + l3 * fc;
						if SSAdditiveTransparency {
							// Additive transparency, scale light value
	//						light *= alpha;
							light = (128.0 + (light * 0.5)) * alpha;
							alpha = 1.0;
						}

						randomindex2 = randomindex_local;
						width = (*(*stage).ss).width * (1.0 + ((*(*stage).ss).variance[0] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);
						height = (*(*stage).ss).height * (1.0 + ((*(*stage).ss).variance[1] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);
						if randomchart[randomindex2 as usize] > 0.5 {
							width = -width;
						}
						randomindex2 = randomindex2.wrapping_add(1);
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

						if usewindpoint && windforce > 0.0 && (*(*stage).ss).wind > 0.0 {
							if SSUsingFog {
								RB_VerticalSurfaceSpriteWindPoint(&curpoint, width, height, light as byte, (alpha * 255.0) as byte,
											(*(*stage).ss).wind, (*(*stage).ss).windIdle, &fogv, (*(*stage).ss).facing, &skew,
											&winddiffv, windforce, SURFSPRITE_FLATTENED == (*(*stage).ss).surfaceSpriteType);
							} else {
								RB_VerticalSurfaceSpriteWindPoint(&curpoint, width, height, light as byte, (alpha * 255.0) as byte,
											(*(*stage).ss).wind, (*(*stage).ss).windIdle, std::ptr::null(), (*(*stage).ss).facing, &skew,
											&winddiffv, windforce, SURFSPRITE_FLATTENED == (*(*stage).ss).surfaceSpriteType);
							}
						} else {
							if SSUsingFog {
								RB_VerticalSurfaceSprite(&curpoint, width, height, light as byte, (alpha * 255.0) as byte,
											(*(*stage).ss).wind, (*(*stage).ss).windIdle, &fogv, (*(*stage).ss).facing, &skew, SURFSPRITE_FLATTENED == (*(*stage).ss).surfaceSpriteType);
							} else {
								RB_VerticalSurfaceSprite(&curpoint, width, height, light as byte, (alpha * 255.0) as byte,
											(*(*stage).ss).wind, (*(*stage).ss).windIdle, std::ptr::null(), (*(*stage).ss).facing, &skew, SURFSPRITE_FLATTENED == (*(*stage).ss).surfaceSpriteType);
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

static fn RB_OrientedSurfaceSprite(loc: *const vec3_t, width: f32, height: f32, light: byte, alpha: byte, fog: *const vec2_t, faceup: c_int) {
	let mut loc2: vec3_t = [0.0; 3];
	let mut right: vec3_t = [0.0; 3];
	let mut points: [f32; 16] = [0.0; 16];
	let mut color: color4ub_t = [0; 4];

	unsafe {
		color[0] = light;
		color[1] = light;
		color[2] = light;
		color[3] = alpha;

		if faceup != 0 {
			let mut width = width;
			let mut height = height;
			width *= 0.5;
			height *= 0.5;

			// Bottom right
		//	VectorAdd(loc, right, point);
			points[0] = (*loc)[0] + width;
			points[1] = (*loc)[1] - width;
			points[2] = (*loc)[2] + 1.0;
			points[3] = 0.0;

			// Top right
		//	VectorAdd(loc, right, point);
			points[4] = (*loc)[0] + width;
			points[5] = (*loc)[1] + width;
			points[6] = (*loc)[2] + 1.0;
			points[7] = 0.0;

			// Top left
		//	VectorSubtract(loc, right, point);
			points[8] = (*loc)[0] - width;
			points[9] = (*loc)[1] + width;
			points[10] = (*loc)[2] + 1.0;
			points[11] = 0.0;

			// Bottom left
		//	VectorSubtract(loc, right, point);
			points[12] = (*loc)[0] - width;
			points[13] = (*loc)[1] - width;
			points[14] = (*loc)[2] + 1.0;
			points[15] = 0.0;
		} else {
			VectorMA(loc, height, &ssViewUp, &mut loc2);
			VectorScale(&ssViewRight, width * 0.5, &mut right);

			// Bottom right
		//	VectorAdd(loc, right, point);
			points[0] = (*loc)[0] + right[0];
			points[1] = (*loc)[1] + right[1];
			points[2] = (*loc)[2] + right[2];
			points[3] = 0.0;

			// Top right
		//	VectorAdd(loc2, right, point);
			points[4] = loc2[0] + right[0];
			points[5] = loc2[1] + right[1];
			points[6] = loc2[2] + right[2];
			points[7] = 0.0;

			// Top left
		//	VectorSubtract(loc2, right, point);
			points[8] = loc2[0] - right[0];
			points[9] = loc2[1] - right[1];
			points[10] = loc2[2] - right[2];
			points[11] = 0.0;

			// Bottom left
		//	VectorSubtract(loc, right, point);
			points[12] = (*loc)[0] - right[0];
			points[13] = (*loc)[1] - right[1];
			points[14] = (*loc)[2] - right[2];
			points[15] = 0.0;
		}

		// Add the sprite to the render list.
		SQuickSprite.Add(points.as_ptr(), &color, fog);
	}
}

static fn RB_DrawOrientedSurfaceSprites(stage: *mut shaderStage_t, input: *const shaderCommands_t) {
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

	unsafe {
		let cutdist: f32 = (*(*stage).ss).fadeMax * rangescalefactor;
		let cutdist2: f32 = cutdist * cutdist;
		let fadedist: f32 = (*(*stage).ss).fadeDist * rangescalefactor;
		let fadedist2: f32 = fadedist * fadedist;

		debug_assert!(cutdist2 != fadedist2);
		let inv_fadediff: f32 = 1.0 / (cutdist2 - fadedist2);

		// The faderange is the fraction amount it takes for these sprites to fade out, assuming an ideal fade range of 250
		let mut faderange: f32 = FADE_RANGE / (cutdist - fadedist);

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
		for curvert in 0..(*input).numVertexes {
			// Calc alpha at each point
			VectorSubtract(&ssViewOrigin, &*(*input).xyz.offset(curvert as isize), &mut dist);
			SSVertAlpha[curvert as usize] = 1.0 - (VectorLengthSquared(&dist) - fadedist2) * inv_fadediff;
		}

		curindex = 0;
		while curindex < (*input).numIndexes - 2 {
			curvert = *(*input).indexes.offset(curindex as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v1);
			if (*(*input).normal.offset(curvert as isize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l1 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
			a1 = SSVertAlpha[curvert as usize];
			fog1[0] = *((tess.svars.texcoords[0][0]).offset((curvert << 1) as isize));
			fog1[1] = *((tess.svars.texcoords[0][0]).offset(((curvert << 1) + 1) as isize));

			curvert = *(*input).indexes.offset((curindex + 1) as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v2);
			if (*(*input).normal.offset(curvert as isize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l2 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
			a2 = SSVertAlpha[curvert as usize];
			fog2[0] = *((tess.svars.texcoords[0][0]).offset((curvert << 1) as isize));
			fog2[1] = *((tess.svars.texcoords[0][0]).offset(((curvert << 1) + 1) as isize));

			curvert = *(*input).indexes.offset((curindex + 2) as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v3);
			if (*(*input).normal.offset(curvert as isize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l3 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
			a3 = SSVertAlpha[curvert as usize];
			fog3[0] = *((tess.svars.texcoords[0][0]).offset((curvert << 1) as isize));
			fog3[1] = *((tess.svars.texcoords[0][0]).offset(((curvert << 1) + 1) as isize));

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
			triarea = fabs(triarea);
			if triarea <= 1.0 {
				// Insanely small abhorrent triangle.
				curindex += 3;
				continue;
			}
			step = (*(*stage).ss).density * Q_rsqrt(triarea);

			let mut randomindex_local: byte = ((v1[0] + v1[1] + v2[0] + v2[1] + v3[0] + v3[1]) as c_int) as byte;
			randominterval = (((v1[0] + v2[1] + v3[2]) as c_int) | 0x03) as byte;	// Make sure the interval is at least 3, and always odd

			posi = 0.0;
			while posi < 1.0 {
				posj = 0.0;
				while posj < (1.0 - posi) {
					fa = posi + randomchart[randomindex_local as usize] * step;
					randomindex_local = randomindex_local.wrapping_add(randominterval);
					if fa > 1.0 {
						posj += step;
						continue;
					}

					fb = posj + randomchart[randomindex_local as usize] * step;
					randomindex_local = randomindex_local.wrapping_add(randominterval);
					if fb > (1.0 - fa) {
						posj += step;
						continue;
					}

					fc = 1.0 - fa - fb;

					// total alpha, minus random factor so some things fade out sooner.
					alphapos = a1 * fa + a2 * fb + a3 * fc;

					// Note that the alpha at this point is a value from 1.0 to 0.0, but represents when to START fading
					thisspritesfadestart = faderange + (1.0 - faderange) * randomchart[randomindex_local as usize];
					randomindex_local = randomindex_local.wrapping_add(randominterval);

					// Find where the alpha is relative to the fadestart, and calc the real alpha to draw at.
					alpha = 1.0 - ((thisspritesfadestart - alphapos) / faderange);

					randomindex_local = randomindex_local.wrapping_add(randominterval);
					if alpha > 0.0 {
						if alpha > 1.0 {
							alpha = 1.0;
						}

						if SSUsingFog {
							fogv[0] = fog1[0] * fa + fog2[0] * fb + fog3[0] * fc;
							fogv[1] = fog1[1] * fa + fog2[1] * fb + fog3[1] * fc;
						}

						VectorScale(&v1, fa, &mut curpoint);
						VectorMA(&curpoint, fb, &v2, &mut curpoint);
						VectorMA(&curpoint, fc, &v3, &mut curpoint);

						light = l1 * fa + l2 * fb + l3 * fc;
						if SSAdditiveTransparency {
							// Additive transparency, scale light value
	//						light *= alpha;
							light = (128.0 + (light * 0.5)) * alpha;
							alpha = 1.0;
						}

						randomindex2 = randomindex_local;
						width = (*(*stage).ss).width * (1.0 + ((*(*stage).ss).variance[0] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);
						height = (*(*stage).ss).height * (1.0 + ((*(*stage).ss).variance[1] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);
						if randomchart[randomindex2 as usize] > 0.5 {
							width = -width;
						}
						randomindex2 = randomindex2.wrapping_add(1);
						if (*(*stage).ss).fadeScale != 0.0 && alphapos < 1.0 {
							width *= 1.0 + ((*(*stage).ss).fadeScale * (1.0 - alphapos));
						}

						if SSUsingFog {
							RB_OrientedSurfaceSprite(&curpoint, width, height, light as byte, (alpha * 255.0) as byte, &fogv, (*(*stage).ss).facing);
						} else {
							RB_OrientedSurfaceSprite(&curpoint, width, height, light as byte, (alpha * 255.0) as byte, std::ptr::null(), (*(*stage).ss).facing);
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

static fn RB_EffectSurfaceSprite(loc: *const vec3_t, width: f32, height: f32, light: byte, alpha: byte, life: f32, faceup: c_int) {
	let mut loc2: vec3_t = [0.0; 3];
	let mut right: vec3_t = [0.0; 3];
	let mut points: [f32; 16] = [0.0; 16];
	let mut color: color4ub_t = [0; 4];

	unsafe {
		color[0] = light;	//light;
		color[1] = light;	//light;
		color[2] = light;	//light;
		color[3] = alpha;	//alpha;

		if faceup != 0 {
			let mut width = width;
			let mut height = height;
			width *= 0.5;
			height *= 0.5;

			// Bottom right
		//	VectorAdd(loc, right, point);
			points[0] = (*loc)[0] + width;
			points[1] = (*loc)[1] - width;
			points[2] = (*loc)[2] + 1.0;
			points[3] = 0.0;

			// Top right
		//	VectorAdd(loc, right, point);
			points[4] = (*loc)[0] + width;
			points[5] = (*loc)[1] + width;
			points[6] = (*loc)[2] + 1.0;
			points[7] = 0.0;

			// Top left
		//	VectorSubtract(loc, right, point);
			points[8] = (*loc)[0] - width;
			points[9] = (*loc)[1] + width;
			points[10] = (*loc)[2] + 1.0;
			points[11] = 0.0;

			// Bottom left
		//	VectorSubtract(loc, right, point);
			points[12] = (*loc)[0] - width;
			points[13] = (*loc)[1] - width;
			points[14] = (*loc)[2] + 1.0;
			points[15] = 0.0;
		} else {
			VectorMA(loc, height, &ssViewUp, &mut loc2);
			VectorScale(&ssViewRight, width * 0.5, &mut right);

			// Bottom right
		//	VectorAdd(loc, right, point);
			points[0] = (*loc)[0] + right[0];
			points[1] = (*loc)[1] + right[1];
			points[2] = (*loc)[2] + right[2];
			points[3] = 0.0;

			// Top right
		//	VectorAdd(loc2, right, point);
			points[4] = loc2[0] + right[0];
			points[5] = loc2[1] + right[1];
			points[6] = loc2[2] + right[2];
			points[7] = 0.0;

			// Top left
		//	VectorSubtract(loc2, right, point);
			points[8] = loc2[0] - right[0];
			points[9] = loc2[1] - right[1];
			points[10] = loc2[2] - right[2];
			points[11] = 0.0;

			// Bottom left
		//	VectorSubtract(loc, right, point);
			points[12] = (*loc)[0] - right[0];
			points[13] = (*loc)[1] - right[1];
			points[14] = (*loc)[2] - right[2];
			points[15] = 0.0;
		}

		// Add the sprite to the render list.
		SQuickSprite.Add(points.as_ptr(), &color, std::ptr::null());
	}
}

static fn RB_DrawEffectSurfaceSprites(stage: *mut shaderStage_t, input: *const shaderCommands_t) {
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

	unsafe {
		let cutdist: f32 = (*(*stage).ss).fadeMax * rangescalefactor;
		let cutdist2: f32 = cutdist * cutdist;
		let fadedist: f32 = (*(*stage).ss).fadeDist * rangescalefactor;
		let fadedist2: f32 = fadedist * fadedist;

		let fxalpha: f32 = (*(*stage).ss).fxAlphaEnd - (*(*stage).ss).fxAlphaStart;
		let mut fadeinout: qboolean = false;

		debug_assert!(cutdist2 != fadedist2);
		let inv_fadediff: f32 = 1.0 / (cutdist2 - fadedist2);

		// The faderange is the fraction amount it takes for these sprites to fade out, assuming an ideal fade range of 250
		let mut faderange: f32 = FADE_RANGE / (cutdist - fadedist);
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
			fadeinout = true;
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
		for curvert in 0..(*input).numVertexes {
			// Calc alpha at each point
			VectorSubtract(&ssViewOrigin, &*(*input).xyz.offset(curvert as isize), &mut dist);
			SSVertAlpha[curvert as usize] = 1.0 - (VectorLengthSquared(&dist) - fadedist2) * inv_fadediff;

		// Note this is the proper equation, but isn't used right now because it would be just a tad slower.
			// Formula for alpha is 1.0f - ((len-fade)/(cut-fade))
			// Which is equal to (1.0+fade/(cut-fade)) - (len/(cut-fade))
			// So mult=1/(cut-fade), and base=(1+fade*mult).
		//	SSVertAlpha[curvert as usize] = fadebase - (VectorLength(dist) * fademult);

		}

		curindex = 0;
		while curindex < (*input).numIndexes - 2 {
			curvert = *(*input).indexes.offset(curindex as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v1);
			if (*(*input).normal.offset(curvert as isize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l1 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
			a1 = SSVertAlpha[curvert as usize];

			curvert = *(*input).indexes.offset((curindex + 1) as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v2);
			if (*(*input).normal.offset(curvert as isize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l2 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
			a2 = SSVertAlpha[curvert as usize];

			curvert = *(*input).indexes.offset((curindex + 2) as isize);
			VectorCopy(&*(*input).xyz.offset(curvert as isize), &mut v3);
			if (*(*input).normal.offset(curvert as isize))[2] < minnormal {
				curindex += 3;
				continue;
			}
			l3 = (*(*input).vertexColors.offset(curvert as isize))[2] as f32;
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
			triarea = fabs(triarea);
			if triarea <= 1.0 {
				// Insanely small abhorrent triangle.
				curindex += 3;
				continue;
			}
			step = density * Q_rsqrt(triarea);

			let mut randomindex_local: byte = ((v1[0] + v1[1] + v2[0] + v2[1] + v3[0] + v3[1]) as c_int) as byte;
			randominterval = (((v1[0] + v2[1] + v3[2]) as c_int) | 0x03) as byte;	// Make sure the interval is at least 3, and always odd

			posi = 0.0;
			while posi < 1.0 {
				posj = 0.0;
				while posj < (1.0 - posi) {
					effecttime = (backEnd.refdef.time as f32 + 10000.0 * randomchart[randomindex_local as usize]) / (*(*stage).ss).fxDuration;
					effectpos = effecttime - (effecttime.floor());

					randomindex2 = (randomindex_local.wrapping_add(effecttime as byte)) as byte;
					randomindex_local = randomindex_local.wrapping_add(randominterval);
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

						VectorScale(&v1, fa, &mut curpoint);
						VectorMA(&curpoint, fb, &v2, &mut curpoint);
						VectorMA(&curpoint, fc, &v3, &mut curpoint);

						light = l1 * fa + l2 * fb + l3 * fc;
						randomindex2 = randomindex_local;
						width = (*(*stage).ss).width * (1.0 + ((*(*stage).ss).variance[0] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);
						height = (*(*stage).ss).height * (1.0 + ((*(*stage).ss).variance[1] * randomchart[randomindex2 as usize]));
						randomindex2 = randomindex2.wrapping_add(1);

						width = width + (effectpos * (*(*stage).ss).fxGrow[0] * width);
						height = height + (effectpos * (*(*stage).ss).fxGrow[1] * height);

						// If we want to fade in and out, that's different than a straight fade.
						if fadeinout {
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

						if SSAdditiveTransparency {
							// Additive transparency, scale light value
	//						light *= alpha;
							light = (128.0 + (light * 0.5)) * alpha;
							alpha = 1.0;
						}

						if randomchart[randomindex2 as usize] > 0.5 {
							width = -width;
						}
						randomindex2 = randomindex2.wrapping_add(1);
						if (*(*stage).ss).fadeScale != 0.0 && alphapos < 1.0 {
							width *= 1.0 + ((*(*stage).ss).fadeScale * (1.0 - alphapos));
						}

						if (*(*stage).ss).wind > 0.0 && curWindSpeed > 0.001 {
							let mut drawpoint: vec3_t = [0.0; 3];

							VectorMA(&curpoint, effectpos * (*(*stage).ss).wind, &curWindBlowVect, &mut drawpoint);
							RB_EffectSurfaceSprite(&drawpoint, width, height, light as byte, (alpha * 255.0) as byte, (*(*stage).ss).fxDuration, (*(*stage).ss).facing);
						} else {
							RB_EffectSurfaceSprite(&curpoint, width, height, light as byte, (alpha * 255.0) as byte, (*(*stage).ss).fxDuration, (*(*stage).ss).facing);
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

pub fn RB_DrawSurfaceSprites(stage: *mut shaderStage_t, input: *const shaderCommands_t) {
	unsafe {
		let glbits: c_uint = (*stage).stateBits;

		R_SurfaceSpriteFrameUpdate();

		//
		// Check fog
		//
		if tess.fogNum != 0 && (*tess.shader).fogPass && (*r_drawfog).value != 0.0 {
			SSUsingFog = true;
			SQuickSprite.StartGroup(&mut (*stage).bundle[0] as *mut _ as *mut core::ffi::c_void, glbits);
		} else {
			SSUsingFog = false;
			SQuickSprite.StartGroup(&mut (*stage).bundle[0] as *mut _ as *mut core::ffi::c_void, glbits);
		}

		// Special provision in case the transparency is additive.
		let GLS_SRCBLEND_BITS: c_uint = 0x0000000F;
		let GLS_DSTBLEND_BITS: c_uint = 0x000000F0;
		let GLS_SRCBLEND_ONE: c_uint = 0x00000004;
		let GLS_DSTBLEND_ONE: c_uint = 0x00000040;

		if (glbits & (GLS_SRCBLEND_BITS | GLS_DSTBLEND_BITS)) == (GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE) {
			// Additive transparency, scale light value
			SSAdditiveTransparency = true;
		} else {
			SSAdditiveTransparency = false;
		}


		//Check if this is a new entity transformation (incl. world entity), and update the appropriate vectors if so.
		if backEnd.currentEntity != ssLastEntityDrawn {
			if backEnd.currentEntity == &mut tr.worldEntity as *mut _ {
				// Drawing the world, so our job is dead-easy, in the viewparms
				VectorCopy(&backEnd.viewParms.or.origin, &mut ssViewOrigin);
				VectorCopy(&backEnd.viewParms.or.axis[1], &mut ssViewRight);
				VectorCopy(&backEnd.viewParms.or.axis[2], &mut ssViewUp);
			} else {
				// Drawing an entity, so we need to transform the viewparms to the model's coordinate system
	//			R_WorldPointToEntity (backEnd.viewParms.or.origin, ssViewOrigin);
				R_WorldNormalToEntity(&backEnd.viewParms.or.axis[1], &mut ssViewRight);
				R_WorldNormalToEntity(&backEnd.viewParms.or.axis[2], &mut ssViewUp);
				VectorCopy(&backEnd.ori.origin, &mut ssViewOrigin);
	//			R_WorldToLocal(backEnd.viewParms.or.axis[1], ssViewRight);
	//			R_WorldToLocal(backEnd.viewParms.or.axis[2], ssViewUp);
			}
			ssLastEntityDrawn = backEnd.currentEntity;
		}

		match (*(*stage).ss).surfaceSpriteType {
			SURFSPRITE_FLATTENED => RB_DrawVerticalSurfaceSprites(stage, input),
			SURFSPRITE_VERTICAL => RB_DrawVerticalSurfaceSprites(stage, input),
			SURFSPRITE_ORIENTED => RB_DrawOrientedSurfaceSprites(stage, input),
			SURFSPRITE_EFFECT => RB_DrawEffectSurfaceSprites(stage, input),
			SURFSPRITE_WEATHERFX => RB_DrawEffectSurfaceSprites(stage, input),
			_ => {},
		}

		SQuickSprite.EndGroup();

		sssurfaces += 1;
	}
}
