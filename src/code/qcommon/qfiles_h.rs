#![allow(non_snake_case)]

//
// qfiles.h: quake file formats
// This file must be identical in the quake and utils directories
//

use core::ffi::{c_char, c_int, c_short, c_uint, c_ushort};

// surface geometry should not exceed these limits
pub const SHADER_MAX_VERTEXES: usize = 1000;
pub const SHADER_MAX_INDEXES: usize = 6 * SHADER_MAX_VERTEXES;


// the maximum size of game reletive pathnames
pub const MAX_QPATH: usize = 64;

/*
========================================================================

QVM files

========================================================================
*/

pub const VM_MAGIC: c_int = 0x12721444;
#[repr(C)]
pub struct vmHeader_t {
	pub vmMagic: c_int,

	pub instructionCount: c_int,

	pub codeOffset: c_int,
	pub codeLength: c_int,

	pub dataOffset: c_int,
	pub dataLength: c_int,
	pub litLength: c_int,			// ( dataLength - litLength ) should be byteswapped on load
	pub bssLength: c_int,			// zero filled memory appended to datalength
}

/*
========================================================================

PCX files are used for 8 bit images

========================================================================
*/

#[repr(C)]
pub struct pcx_t {
    pub manufacturer: c_char,
    pub version: c_char,
    pub encoding: c_char,
    pub bits_per_pixel: c_char,
    pub xmin: c_ushort,
    pub ymin: c_ushort,
    pub xmax: c_ushort,
    pub ymax: c_ushort,
    pub hres: c_ushort,
    pub vres: c_ushort,
    pub palette: [u8; 48],
    pub reserved: c_char,
    pub color_planes: c_char,
    pub bytes_per_line: c_ushort,
    pub palette_type: c_ushort,
    pub filler: [c_char; 58],
    pub data: u8,			// unbounded
}

/*
========================================================================

TGA files are used for 24/32 bit images

========================================================================
*/

#[repr(C)]
pub struct TargaHeader {
	pub id_length: u8,
	pub colormap_type: u8,
	pub image_type: u8,
	pub colormap_index: c_ushort,
	pub colormap_length: c_ushort,
	pub colormap_size: u8,
	pub x_origin: c_ushort,
	pub y_origin: c_ushort,
	pub width: c_ushort,
	pub height: c_ushort,
	pub pixel_size: u8,
	pub attributes: u8,
}



/*
========================================================================

.MD3 triangle model file format

========================================================================
*/

pub const MD3_IDENT: c_int = (('3' as c_int) << 24) + (('P' as c_int) << 16) + (('D' as c_int) << 8) + ('I' as c_int);
pub const MD3_VERSION: c_int = 15;

// limits
pub const MD3_MAX_LODS: c_int = 3;
pub const MD3_MAX_TRIANGLES: c_int = 8192;	// per surface
pub const MD3_MAX_VERTS: c_int = 4096;	// per surface
pub const MD3_MAX_SHADERS: c_int = 256;		// per surface
pub const MD3_MAX_FRAMES: c_int = 1024;	// per model
pub const MD3_MAX_SURFACES: c_int = 32 + 32;	// per model
pub const MD3_MAX_TAGS: c_int = 16;		// per frame

// vertex scales
pub const MD3_XYZ_SCALE: f32 = 1.0 / 64.0;

pub type vec3_t = [f32; 3];

#[repr(C)]
pub struct md3Frame_t {
	pub bounds: [vec3_t; 2],
	pub localOrigin: vec3_t,
	pub radius: f32,
	pub name: [c_char; 16],
}

#[repr(C)]
pub struct md3Tag_t {
	pub name: [c_char; MAX_QPATH],	// tag name
	pub origin: vec3_t,
	pub axis: [vec3_t; 3],
}

/*
** md3Surface_t
**
** CHUNK			SIZE
** header			sizeof( md3Surface_t )
** shaders			sizeof( md3Shader_t ) * numShaders
** triangles[0]		sizeof( md3Triangle_t ) * numTriangles
** st				sizeof( md3St_t ) * numVerts
** XyzNormals		sizeof( md3XyzNormal_t ) * numVerts * numFrames
*/
#[repr(C)]
pub struct md3Surface_t {
	pub ident: c_int,				//

	pub name: [c_char; MAX_QPATH],	// polyset name

	pub flags: c_int,
	pub numFrames: c_int,			// all surfaces in a model should have the same

	pub numShaders: c_int,			// all surfaces in a model should have the same
	pub numVerts: c_int,

	pub numTriangles: c_int,
	pub ofsTriangles: c_int,

	pub ofsShaders: c_int,			// offset from start of md3Surface_t
	pub ofsSt: c_int,				// texture coords are common for all frames
	pub ofsXyzNormals: c_int,		// numVerts * numFrames

	pub ofsEnd: c_int,				// next surface follows
}

#[repr(C)]
pub struct md3Shader_t {
	pub name: [c_char; MAX_QPATH],
	pub shaderIndex: c_int,	// for in-game use
}

#[repr(C)]
pub struct md3Triangle_t {
	pub indexes: [c_int; 3],
}

#[repr(C)]
pub struct md3St_t {
	pub st: [f32; 2],
}

#[repr(C)]
pub struct md3XyzNormal_t {
	pub xyz: [c_short; 3],
	pub normal: c_short,
}

#[repr(C)]
pub struct md3Header_t {
	pub ident: c_int,
	pub version: c_int,

	pub name: [c_char; MAX_QPATH],	// model name

	pub flags: c_int,

	pub numFrames: c_int,
	pub numTags: c_int,
	pub numSurfaces: c_int,

	pub numSkins: c_int,

	pub ofsFrames: c_int,			// offset for first frame
	pub ofsTags: c_int,			// numFrames * numTags
	pub ofsSurfaces: c_int,		// first surface, others follow

	pub ofsEnd: c_int,				// end of file
}


/*
==============================================================================

  .BSP file format

==============================================================================
*/


pub const BSP_IDENT: c_int = (('P' as c_int) << 24) + (('S' as c_int) << 16) + (('B' as c_int) << 8) + ('R' as c_int);
		// little-endian "IBSP"

pub const BSP_VERSION: c_int = 1;


// there shouldn't be any problem with increasing these values at the
// expense of more memory allocation in the utilities
pub const MAX_MAP_MODELS: c_int = 0x400;
pub const MAX_MAP_BRUSHES: c_int = 0x8000;
pub const MAX_MAP_ENTITIES: c_int = 0x800;
pub const MAX_MAP_ENTSTRING: c_int = 0x40000;
pub const MAX_MAP_SHADERS: c_int = 0x400;

pub const MAX_MAP_AREAS: c_int = 0x100;	// MAX_MAP_AREA_BYTES in q_shared must match!
pub const MAX_MAP_FOGS: c_int = 0x100;
pub const MAX_MAP_PLANES: c_int = 0x20000;
pub const MAX_MAP_NODES: c_int = 0x20000;
pub const MAX_MAP_BRUSHSIDES: c_int = 0x20000;
pub const MAX_MAP_LEAFS: c_int = 0x20000;
pub const MAX_MAP_LEAFFACES: c_int = 0x20000;
pub const MAX_MAP_LEAFBRUSHES: c_int = 0x40000;
pub const MAX_MAP_PORTALS: c_int = 0x20000;
pub const MAX_MAP_LIGHTING: c_int = 0x800000;
pub const MAX_MAP_LIGHTGRID: c_int = 65535;
pub const MAX_MAP_LIGHTGRID_ARRAY: c_int = 0x100000;

pub const MAX_MAP_VISIBILITY: c_int = 0x400000;

pub const MAX_MAP_DRAW_SURFS: c_int = 0x20000;
pub const MAX_MAP_DRAW_VERTS: c_int = 0x80000;
pub const MAX_MAP_DRAW_INDEXES: c_int = 0x80000;


// key / value pair sizes in the entities lump
pub const MAX_KEY: c_int = 32;
pub const MAX_VALUE: c_int = 1024;

// the editor uses these predefined yaw angles to orient entities up or down
pub const ANGLE_UP: c_int = -1;
pub const ANGLE_DOWN: c_int = -2;

pub const LIGHTMAP_WIDTH: c_int = 128;
pub const LIGHTMAP_HEIGHT: c_int = 128;

//=============================================================================

#[cfg(feature = "_XBOX")]
pub mod xbox {
	use core::ffi::{c_char, c_int, c_short, c_ushort};
	use super::{MAX_QPATH, vec3_t};

	pub type byte = u8;

	#[repr(C, packed)]
	pub struct dmodel_t {
		pub mins: [f32; 3],
		pub maxs: [f32; 3],
		pub firstSurface: c_int,
		pub numSurfaces: c_ushort,
		pub firstBrush: c_int,
		pub numBrushes: c_ushort,
	}

	#[repr(C, packed)]
	pub struct dshader_t {
		pub shader: [c_char; MAX_QPATH],
		pub surfaceFlags: c_int,
		pub contentFlags: c_int,
	}

	// planes x^1 is allways the opposite of plane x

	#[repr(C, packed)]
	pub struct dplane_t {
		pub normal: [f32; 3],
		pub dist: f32,
	}

	#[repr(C, packed)]
	pub struct dnode_t {
		pub planeNum: c_int,
		pub children: [c_short; 2],	// negative numbers are -(leafs+1), not nodes
		pub mins: [c_short; 3],		// for frustom culling
		pub maxs: [c_short; 3],
	}

	#[repr(C, packed)]
	pub struct dleaf_t {
		pub cluster: c_short,			// -1 = opaque cluster (do I still store these?)
		pub area: i8,

		pub mins: [c_short; 3],			// for frustum culling
		pub maxs: [c_short; 3],

		pub firstLeafSurface: c_ushort,
		pub numLeafSurfaces: c_ushort,

		pub firstLeafBrush: c_ushort,
		pub numLeafBrushes: c_ushort,
	}

	#[repr(C, packed)]
	pub struct dbrushside_t {
		pub planeNum: c_int,		// positive plane side faces out of the leaf
		pub shaderNum: byte,
	}

	#[repr(C, packed)]
	pub struct dbrush_t {
		pub firstSide: c_int,
		pub numSides: byte,
		pub shaderNum: c_ushort,		// the shader that determines the contents flags
	}

	#[repr(C, packed)]
	pub struct dfog_t {
		pub shader: [c_char; MAX_QPATH],
		pub brushNum: c_int,
		pub visibleSide: c_int,	// the brush side that ray tests need to clip against (-1 == none)
	}

	// Light Style Constants
	pub const MAXLIGHTMAPS: usize = 4;
	pub const LS_NORMAL: u8 = 0x00;
	pub const LS_UNUSED: u8 = 0xfe;
	pub const LS_NONE: u8 = 0xff;
	pub const MAX_LIGHT_STYLES: c_int = 64;

	#[repr(C, packed)]
	pub struct mapVert_t {
		pub lightmap: [[f32; 2]; MAXLIGHTMAPS],
		pub st: [f32; 2],
		pub xyz: [c_short; 3],
		pub normal: [c_short; 3],
		pub color: [[byte; 4]; MAXLIGHTMAPS],
	}

	pub const DRAWVERT_LIGHTMAP_SCALE: f32 = 32768.0;
	// Change texture coordinates for TriSurfs to be even more fine grain.
	// See below for note about keeping MIN_ST and MAX_ST up to date with
	// ST_SCALE. These are in 4.12. Okay, how about 5.11?
	//#define DRAWVERT_ST_SCALE 4096.0f
	pub const DRAWVERT_ST_SCALE: f32 = 2048.0;

	// We use a slightly different format for the fixed point texture
	// coords in Grid/Mesh drawverts: 10.6 rather than 12.4
	// To be sure that this is ok, keep the max and min values equal to
	// the largest and smallest whole numbers that can be stored using the
	// format. (ie: Don't change GRID_DRAWVERT_ST_SCALE without changing
	// the other two!) (And don't forget that we're using a bit for sign.)
	pub const GRID_DRAWVERT_ST_SCALE: f32 = 64.0;

	// This master switch controls whether we use compressed (4-bit per channel)
	// vertex colors in draw and surface verts. It saves memory, but I'm switching
	// it off, because we end up with that nasty green/purple streaking effect.
	// If we ever figure out how to do it better... (1555? 565?)
	//#define COMPRESS_VERTEX_COLORS

	#[repr(C, packed)]
	pub struct drawVert_t {
		pub xyz: [c_short; 3],
		pub dvst: [c_short; 2],
		pub dvlightmap: [[c_short; 2]; MAXLIGHTMAPS],
		pub normal: [c_short; 3],
		// #ifdef _XBOX
		pub tangent: vec3_t,
		// #endif
		// #ifdef COMPRESS_VERTEX_COLORS
		//	pub dvcolor: [[byte; 1]; MAXLIGHTMAPS],
		// #else
		pub dvcolor: [[byte; 4]; MAXLIGHTMAPS],
		// #endif
	}

	#[repr(C, packed)]
	pub struct dgrid_t {
		pub flags: byte,
		pub latLong: [byte; 2],
	}

	#[repr(C, packed)]
	pub struct dface_t {
		pub code: c_int,
		pub shaderNum: byte,
		pub fogNum: i8,

		pub verts: c_uint,				// high 20 bits are first vert, low 12 are num verts
		pub indexes: c_uint,			// high 20 bits are first index, low 12 are num indices

		pub lightmapStyles: [byte; MAXLIGHTMAPS],
		pub lightmapNum: [byte; MAXLIGHTMAPS],

		pub lightmapVecs: [c_short; 3],
	}

	#[repr(C, packed)]
	pub struct dpatch_t {
		pub code: c_int,
		pub shaderNum: byte,
		pub fogNum: i8,

		pub verts: c_uint,				// high 20 bits are first vert, low 12 are num verts

		pub lightmapStyles: [byte; MAXLIGHTMAPS],
		pub lightmapNum: [byte; MAXLIGHTMAPS],

		pub lightmapVecs: [[c_short; 3]; 2],		// for patches, [0] and [1] are lodbounds

		pub patchWidth: byte,
		pub patchHeight: byte,
	}

	#[repr(C, packed)]
	pub struct dtrisurf_t {
		pub code: c_int,
		pub shaderNum: byte,
		pub fogNum: i8,

		pub verts: c_uint,				// high 20 bits are first vert, low 12 are num verts
		pub indexes: c_uint,			// high 20 bits are first index, low 12 are num indices

		pub lightmapStyles: [byte; MAXLIGHTMAPS],
	}

	#[repr(C, packed)]
	pub struct dflare_t {
		pub code: c_int,
		pub shaderNum: byte,
		pub fogNum: i8,

		pub origin: [c_short; 3],
		pub normal: [c_short; 3],
		pub color: [byte; 3],
	}
}

#[cfg(not(feature = "_XBOX"))]
pub mod non_xbox {
	use core::ffi::{c_char, c_int, c_short};
	use super::{MAX_QPATH, vec3_t};

	pub type byte = u8;

	#[repr(C)]
	pub struct lump_t {
		pub fileofs: c_int,
		pub filelen: c_int,
	}

	pub const LUMP_ENTITIES: c_int = 0;
	pub const LUMP_SHADERS: c_int = 1;
	pub const LUMP_PLANES: c_int = 2;
	pub const LUMP_NODES: c_int = 3;
	pub const LUMP_LEAFS: c_int = 4;
	pub const LUMP_LEAFSURFACES: c_int = 5;
	pub const LUMP_LEAFBRUSHES: c_int = 6;
	pub const LUMP_MODELS: c_int = 7;
	pub const LUMP_BRUSHES: c_int = 8;
	pub const LUMP_BRUSHSIDES: c_int = 9;
	pub const LUMP_DRAWVERTS: c_int = 10;
	pub const LUMP_DRAWINDEXES: c_int = 11;
	pub const LUMP_FOGS: c_int = 12;
	pub const LUMP_SURFACES: c_int = 13;
	pub const LUMP_LIGHTMAPS: c_int = 14;
	pub const LUMP_LIGHTGRID: c_int = 15;
	pub const LUMP_VISIBILITY: c_int = 16;
	pub const LUMP_LIGHTARRAY: c_int = 17;
	pub const HEADER_LUMPS: c_int = 18;

	#[repr(C)]
	pub struct dheader_t {
		pub ident: c_int,
		pub version: c_int,

		pub lumps: [lump_t; HEADER_LUMPS as usize],
	}

	#[repr(C)]
	pub struct dmodel_t {
		pub mins: [f32; 3],
		pub maxs: [f32; 3],
		pub firstSurface: c_int,
		pub numSurfaces: c_int,
		pub firstBrush: c_int,
		pub numBrushes: c_int,
	}

	#[repr(C)]
	pub struct dshader_t {
		pub shader: [c_char; MAX_QPATH],
		pub surfaceFlags: c_int,
		pub contentFlags: c_int,
	}

	// planes x^1 is allways the opposite of plane x

	#[repr(C)]
	pub struct dplane_t {
		pub normal: [f32; 3],
		pub dist: f32,
	}

	#[repr(C)]
	pub struct dnode_t {
		pub planeNum: c_int,
		pub children: [c_int; 2],	// negative numbers are -(leafs+1), not nodes
		pub mins: [c_int; 3],		// for frustom culling
		pub maxs: [c_int; 3],
	}

	#[repr(C)]
	pub struct dleaf_t {
		pub cluster: c_int,			// -1 = opaque cluster (do I still store these?)
		pub area: c_int,

		pub mins: [c_int; 3],			// for frustum culling
		pub maxs: [c_int; 3],

		pub firstLeafSurface: c_int,
		pub numLeafSurfaces: c_int,

		pub firstLeafBrush: c_int,
		pub numLeafBrushes: c_int,
	}

	#[repr(C)]
	pub struct dbrushside_t {
		pub planeNum: c_int,			// positive plane side faces out of the leaf
		pub shaderNum: c_int,
		pub drawSurfNum: c_int,
	}

	#[repr(C)]
	pub struct dbrush_t {
		pub firstSide: c_int,
		pub numSides: c_int,
		pub shaderNum: c_int,		// the shader that determines the contents flags
	}

	#[repr(C)]
	pub struct dfog_t {
		pub shader: [c_char; MAX_QPATH],
		pub brushNum: c_int,
		pub visibleSide: c_int,	// the brush side that ray tests need to clip against (-1 == none)
	}

	// Light Style Constants
	pub const MAXLIGHTMAPS: usize = 4;
	pub const LS_NORMAL: u8 = 0x00;
	pub const LS_UNUSED: u8 = 0xfe;
	pub const LS_NONE: u8 = 0xff;
	pub const MAX_LIGHT_STYLES: c_int = 64;

	#[repr(C)]
	pub struct mapVert_t {
		pub xyz: vec3_t,
		pub st: [f32; 2],
		pub lightmap: [[f32; 2]; MAXLIGHTMAPS],
		pub normal: vec3_t,
		pub color: [[byte; 4]; MAXLIGHTMAPS],
	}

	#[repr(C)]
	pub struct drawVert_t {
		pub xyz: vec3_t,
		pub st: [f32; 2],
		pub lightmap: [[f32; 2]; MAXLIGHTMAPS],
		pub normal: vec3_t,
		pub color: [[byte; 4]; MAXLIGHTMAPS],
	}

	#[repr(C)]
	pub struct dgrid_t {
		pub ambientLight: [[byte; 3]; MAXLIGHTMAPS],
		pub directLight: [[byte; 3]; MAXLIGHTMAPS],
		pub styles: [byte; MAXLIGHTMAPS],
		pub latLong: [byte; 2],
	}

	#[repr(C)]
	pub enum mapSurfaceType_t {
		MST_BAD,
		MST_PLANAR,
		MST_PATCH,
		MST_TRIANGLE_SOUP,
		MST_FLARE
	}

	#[repr(C)]
	pub struct dsurface_t {
		pub shaderNum: c_int,
		pub fogNum: c_int,
		pub surfaceType: c_int,

		pub firstVert: c_int,
		pub numVerts: c_int,

		pub firstIndex: c_int,
		pub numIndexes: c_int,

		pub lightmapStyles: [byte; MAXLIGHTMAPS],
		pub vertexStyles: [byte; MAXLIGHTMAPS],
		pub lightmapNum: [c_int; MAXLIGHTMAPS],
		pub lightmapX: [c_int; MAXLIGHTMAPS],
		pub lightmapY: [c_int; MAXLIGHTMAPS],
		pub lightmapWidth: c_int,
		pub lightmapHeight: c_int,

		pub lightmapOrigin: vec3_t,
		pub lightmapVecs: [vec3_t; 3],	// for patches, [0] and [1] are lodbounds

		pub patchWidth: c_int,
		pub patchHeight: c_int,
	}
}

#[repr(C)]
pub enum hunkAllocType_t {
	HA_MISC,
	HA_MAP,
	HA_SHADERS,
	HA_LIGHTING,
	HA_FOG,
	HA_PATCHES,
	HA_VIS,
	HA_SUBMODELS,
	HA_MODELS,
	MAX_HA_TYPES
}



/////////////////////////////////////////////////////////////
//
// Defines and structures required for fonts

pub const GLYPH_COUNT: usize = 256;

// Must match define in stmparse.h
pub const STYLE_DROPSHADOW: c_uint = 0x80000000;
pub const STYLE_BLINK: c_uint = 0x40000000;
pub const SET_MASK: c_uint = 0x00ffffff;

#[repr(C)]
pub struct glyphInfo_t {
	pub width: c_short,					// number of pixels wide
	pub height: c_short,					// number of scan lines
	pub horizAdvance: c_short,			// number of pixels to advance to the next char
	pub horizOffset: c_short,			// x offset into space to render glyph
	pub baseline: c_int,				// y offset
	pub s: f32,						// x start tex coord
	pub t: f32,						// y start tex coord
	pub s2: f32,						// x end tex coord
	pub t2: f32,						// y end tex coord
}


// this file corresponds 1:1 with the "*.fontdat" files, so don't change it unless you're going to
//	recompile the fontgen util and regenerate all the fonts!
//
#[repr(C)]
pub struct dfontdat_t {
	pub mGlyphs: [glyphInfo_t; GLYPH_COUNT],

	pub mPointSize: c_short,
	pub mHeight: c_short,				// max height of font
	pub mAscender: c_short,
	pub mDescender: c_short,

	pub mKoreanHack: c_short,			// unused field, written out by John's fontgen program but we have to leave it there for disk structs <sigh>
}

/////////////////// fonts end ////////////////////////////////////
