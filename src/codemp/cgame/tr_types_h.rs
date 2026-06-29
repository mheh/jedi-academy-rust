// Copyright (C) 1999-2000 Id Software, Inc.
//

use core::ffi::{c_int, c_char, c_void};

// Type stubs for types defined in other headers
pub type vec3_t = [f32; 3];
pub type vec2_t = [f32; 2];
pub type qhandle_t = c_int;
pub type qboolean = c_int;
pub type byte = u8;

// Stub constant - defined in qcommon.h or similar
pub const MAX_MAP_AREA_BYTES: usize = 64;

pub const MAX_DLIGHTS: c_int = 32;			// can't be increased, because bit flags are used on surfaces
#[cfg(target_os = "xbox")]
pub const MAX_ENTITIES: c_int = 1024;		// 11 bits, can't be increased without changing drawsurf bit packing (QSORT_ENTITYNUM_SHIFT)
#[cfg(not(target_os = "xbox"))]
pub const MAX_ENTITIES: c_int = 2048;		// 11 bits, can't be increased without changing drawsurf bit packing (QSORT_ENTITYNUM_SHIFT)
pub const MAX_MINI_ENTITIES: c_int = 1024;

pub const TR_WORLDENT: c_int = MAX_ENTITIES - 1;

// renderfx flags
pub const RF_MINLIGHT: c_int = 0x00001;	// allways have some light (viewmodel, some items)
pub const RF_THIRD_PERSON: c_int = 0x00002;	// don't draw through eyes, only mirrors (player bodies, chat sprites)
pub const RF_FIRST_PERSON: c_int = 0x00004;	// only draw through eyes (view weapon, damage blood blob)
pub const RF_DEPTHHACK: c_int = 0x00008;	// for view weapon Z crunching
pub const RF_NODEPTH: c_int = 0x00010;	// No depth at all (seeing through walls)

pub const RF_VOLUMETRIC: c_int = 0x00020;	// fake volumetric shading

pub const RF_NOSHADOW: c_int = 0x00040;	// don't add stencil shadows

pub const RF_LIGHTING_ORIGIN: c_int = 0x00080;	// use refEntity->lightingOrigin instead of refEntity->origin
										// for lighting.  This allows entities to sink into the floor
										// with their origin going solid, and allows all parts of a
										// player to get the same lighting
pub const RF_SHADOW_PLANE: c_int = 0x00100;	// use refEntity->shadowPlane
pub const RF_WRAP_FRAMES: c_int = 0x00200;	// mod the model frames by the maxframes to allow continuous
										// animation without needing to know the frame count

pub const RF_FORCE_ENT_ALPHA: c_int = 0x00400; // override shader alpha settings
pub const RF_RGB_TINT: c_int = 0x00800; // override shader rgb settings

pub const RF_SHADOW_ONLY: c_int = 0x01000;	//add surfs for shadowing but don't draw them -rww

pub const RF_DISTORTION: c_int = 0x02000;	//area distortion effect -rww

pub const RF_FORKED: c_int = 0x04000;	// override lightning to have forks
pub const RF_TAPERED: c_int = 0x08000;	// lightning tapers
pub const RF_GROW: c_int = 0x10000;	// lightning grows from start to end during its life

pub const RF_DISINTEGRATE1: c_int = 0x20000;	// does a procedural hole-ripping thing.
pub const RF_DISINTEGRATE2: c_int = 0x40000;	// does a procedural hole-ripping thing with scaling at the ripping point

pub const RF_SETANIMINDEX: c_int = 0x80000;	//use backEnd.currentEntity->e.skinNum for R_BindAnimatedImage

pub const RF_ALPHA_DEPTH: c_int = 0x100000; //depth write on alpha model

pub const RF_FORCEPOST: c_int = 0x200000; //force it to post-render -rww

// refdef flags
pub const RDF_NOWORLDMODEL: c_int = 1;		// used for player configuration screen
pub const RDF_HYPERSPACE: c_int = 4;		// teleportation effect

pub const RDF_SKYBOXPORTAL: c_int = 8;
pub const RDF_DRAWSKYBOX: c_int = 16;		// the above marks a scene as being a 'portal sky'.  this flag says to draw it or not

pub const RDF_AUTOMAP: c_int = 32;		//means this scene is to draw the automap -rww
pub const RDF_NOFOG: c_int = 64;		//no global fog in this scene (but still brush fog) -rww

pub static mut skyboxportal: c_int = 0;
pub static mut drawskyboxportal: c_int = 0;

pub type color4ub_t = [byte; 4];

#[repr(C)]
pub struct polyVert_t {
	pub xyz: vec3_t,
	pub st: [f32; 2],
	pub modulate: [byte; 4],
}

#[repr(C)]
pub struct poly_s {
	pub hShader: qhandle_t,
	pub numVerts: c_int,
	pub verts: *mut polyVert_t,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum refEntityType_t {
	RT_MODEL,
	RT_POLY,
	RT_SPRITE,
	RT_ORIENTED_QUAD,
	RT_BEAM,
	RT_SABER_GLOW,
	RT_ELECTRICITY,
	RT_PORTALSURFACE,		// doesn't draw anything, just info for portals
	RT_LINE,
	RT_ORIENTEDLINE,
	RT_CYLINDER,
	RT_ENT_CHAIN,

	RT_MAX_REF_ENTITY_TYPE,
}

#[repr(C)]
pub struct miniRefEntity_t {
	pub reType: refEntityType_t,
	pub renderfx: c_int,

	pub hModel: qhandle_t,				// opaque type outside refresh

	// most recent data
	pub axis: [vec3_t; 3],			// rotation vectors
	pub nonNormalizedAxes: qboolean,	// axis are not normalized, i.e. they have scale
	pub origin: vec3_t,				// also used as MODEL_BEAM's "from"

	// previous data for frame interpolation
	pub oldorigin: vec3_t,			// also used as MODEL_BEAM's "to"

	// texturing
	pub customShader: qhandle_t,		// use one image for the entire thing

	// misc
	pub shaderRGBA: [byte; 4],		// colors used by rgbgen entity shaders
	pub shaderTexCoord: vec2_t,		// texture coordinates used by tcMod entity modifiers

	// extra sprite information
	pub radius: f32,
	pub rotation: f32,			// size 2 for RT_CYLINDER or number of verts in RT_ELECTRICITY

	// misc
	pub shaderTime: f32,			// subtracted from refdef time to control effect start times
	pub frame: c_int,				// also used as MODEL_BEAM's diameter

}

// warning (disable : 4201 )
#[repr(C)]
pub struct refEntity_t {
	// this stucture must remain identical as the miniRefEntity_t
	//
	//
	pub reType: refEntityType_t,
	pub renderfx: c_int,

	pub hModel: qhandle_t,				// opaque type outside refresh

	// most recent data
	pub axis: [vec3_t; 3],			// rotation vectors
	pub nonNormalizedAxes: qboolean,	// axis are not normalized, i.e. they have scale
	pub origin: vec3_t,				// also used as MODEL_BEAM's "from"

	// previous data for frame interpolation
	pub oldorigin: vec3_t,			// also used as MODEL_BEAM's "to"

	// texturing
	pub customShader: qhandle_t,		// use one image for the entire thing

	// misc
	pub shaderRGBA: [byte; 4],		// colors used by rgbgen entity shaders
	pub shaderTexCoord: vec2_t,		// texture coordinates used by tcMod entity modifiers

	// extra sprite information
	pub radius: f32,
	pub rotation: f32,

	// misc
	pub shaderTime: f32,			// subtracted from refdef time to control effect start times
	pub frame: c_int,				// also used as MODEL_BEAM's diameter
	//
	//
	// end miniRefEntity_t

	//
	//
	// specific full refEntity_t data
	//
	//

	// most recent data
	pub lightingOrigin: vec3_t,		// so multi-part models can be lit identically (RF_LIGHTING_ORIGIN)
	pub shadowPlane: f32,		// projection shadows go here, stencils go slightly lower

	// previous data for frame interpolation
	pub oldframe: c_int,
	pub backlerp: f32,			// 0.0 = current, 1.0 = old

	// texturing
	pub skinNum: c_int,			// inline skin index
	pub customSkin: qhandle_t,			// NULL for default skin

	// texturing
	pub uRefEnt: refEntity_t_uRefEnt,

	// extra sprite information
	pub data: refEntity_t_data,

	pub endTime: f32,
	pub saberLength: f32,

/*
Ghoul2 Insert Start
*/
	pub angles: vec3_t,				// rotation angles - used for Ghoul2

	pub modelScale: vec3_t,			// axis scale for models
//	CGhoul2Info_v	*ghoul2;  		// has to be at the end of the ref-ent in order for it to be created properly
	pub ghoul2: *mut c_void,  		// has to be at the end of the ref-ent in order for it to be created properly
/*
Ghoul2 Insert End
*/
}

#[repr(C)]
pub union refEntity_t_uRefEnt {
//		pub skinNum: c_int,		// inline skin index
//		pub terxelCoords: ivec3_t,	// coords of patch for RT_TERXELS
	pub uMini: refEntity_t_uRefEnt_uMini,
}

#[repr(C)]
pub struct refEntity_t_uRefEnt_uMini {
	pub miniStart: c_int,
	pub miniCount: c_int,
}

#[repr(C)]
pub union refEntity_t_data {
	pub sprite: refEntity_t_data_sprite,
	pub line: refEntity_t_data_line,
	pub bezier: refEntity_t_data_bezier,	// that whole put-the-opening-brace-on-the-same-line-as-the-beginning-of-the-definition coding style is fecal
	pub cylinder: refEntity_t_data_cylinder,
	pub electricity: refEntity_t_data_electricity,
}

#[repr(C)]
pub struct refEntity_t_data_sprite {
	pub rotation: f32,
	pub radius: f32,
	pub vertRGBA: [[byte; 4]; 4],
}

#[repr(C)]
pub struct refEntity_t_data_line {
	pub width: f32,
	pub width2: f32,
	pub stscale: f32,
}

#[repr(C)]
pub struct refEntity_t_data_bezier {
	pub width: f32,
	pub control1: vec3_t,
	pub control2: vec3_t,
}

#[repr(C)]
pub struct refEntity_t_data_cylinder {
	pub width: f32,
	pub width2: f32,
	pub stscale: f32,
	pub height: f32,
	pub bias: f32,
	pub wrap: qboolean,
}

#[repr(C)]
pub struct refEntity_t_data_electricity {
	pub width: f32,
	pub deviation: f32,
	pub stscale: f32,
	pub wrap: qboolean,
	pub taper: qboolean,
}

pub const MAX_RENDER_STRINGS: usize = 8;
pub const MAX_RENDER_STRING_LENGTH: usize = 32;

#[repr(C)]
pub struct refdef_t {
	pub x: c_int,
	pub y: c_int,
	pub width: c_int,
	pub height: c_int,
	pub fov_x: f32,
	pub fov_y: f32,
	pub vieworg: vec3_t,
	pub viewangles: vec3_t,
	pub viewaxis: [vec3_t; 3],		// transformation matrix
	pub viewContents: c_int,		// world contents at vieworg

	// time in milliseconds for shader effects and other time dependent rendering issues
	pub time: c_int,

	pub rdflags: c_int,			// RDF_NOWORLDMODEL, etc

	// 1 bits will prevent the associated area from rendering at all
	pub areamask: [byte; MAX_MAP_AREA_BYTES],

	// text messages for deform text shaders
	pub text: [[c_char; MAX_RENDER_STRING_LENGTH]; MAX_RENDER_STRINGS],
}

// typedef enum {
//     STEREO_CENTER,
//     STEREO_LEFT,
//     STEREO_RIGHT
// };
// typedef int stereoFrame_t;
pub type stereoFrame_t = c_int;
pub const STEREO_CENTER: stereoFrame_t = 0;
pub const STEREO_LEFT: stereoFrame_t = 1;
pub const STEREO_RIGHT: stereoFrame_t = 2;

/*
** glconfig_t
**
** Contains variables specific to the OpenGL configuration
** being run right now.  These are constant once the OpenGL
** subsystem is initialized.
*/
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum textureCompression_t {
	TC_NONE,
	TC_S3TC,
	TC_S3TC_DXT,
}

#[repr(C)]
pub struct glconfig_t {
	pub renderer_string: *const c_char,
	pub vendor_string: *const c_char,
	pub version_string: *const c_char,
	pub extensions_string: *const c_char,

	pub maxTextureSize: c_int,			// queried from GL
	pub maxActiveTextures: c_int,		// multitexture ability
	pub maxTextureFilterAnisotropy: f32,

	pub colorBits: c_int,
	pub depthBits: c_int,
	pub stencilBits: c_int,

	pub deviceSupportsGamma: qboolean,
	pub textureCompression: textureCompression_t,
	pub textureEnvAddAvailable: qboolean,
	pub clampToEdgeAvailable: qboolean,

	pub vidWidth: c_int,
	pub vidHeight: c_int,

	pub displayFrequency: c_int,

	// synonymous with "does rendering consume the entire screen?", therefore
	// a Voodoo or Voodoo2 will have this set to TRUE, as will a Win32 ICD that
	// used CDS.
	pub isFullscreen: qboolean,
	pub stereoEnabled: qboolean,
}

#[cfg(not(target_os = "windows"))]
pub const OPENGL_DRIVER_NAME: &str = "libGL.so";

#[cfg(target_os = "windows")]
pub const OPENGL_DRIVER_NAME: &str = "opengl32";
