#![allow(non_camel_case_types)]

//
// qfiles.h: quake file formats
// This file must be identical in the quake and utils directories
//

use core::ffi::{c_char, c_int};

// surface geometry should not exceed these limits
pub const SHADER_MAX_VERTEXES: c_int = 1000;
pub const SHADER_MAX_INDEXES: c_int = 6 * SHADER_MAX_VERTEXES;

// the maximum size of game relative pathnames
pub const MAX_QPATH: c_int = 64;

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
    pub litLength: c_int,            // ( dataLength - litLength ) should be byteswapped on load
    pub bssLength: c_int,            // zero filled memory appended to datalength
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
    pub xmin: u16,
    pub ymin: u16,
    pub xmax: u16,
    pub ymax: u16,
    pub hres: u16,
    pub vres: u16,
    pub palette: [u8; 48],
    pub reserved: c_char,
    pub color_planes: c_char,
    pub bytes_per_line: u16,
    pub palette_type: u16,
    pub filler: [c_char; 58],
    pub data: u8,            // unbounded
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
    pub colormap_index: u16,
    pub colormap_length: u16,
    pub colormap_size: u8,
    pub x_origin: u16,
    pub y_origin: u16,
    pub width: u16,
    pub height: u16,
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
pub const MD3_MAX_TRIANGLES: c_int = 8192;    // per surface
pub const MD3_MAX_VERTS: c_int = 4096;        // per surface
pub const MD3_MAX_SHADERS: c_int = 256;       // per surface
pub const MD3_MAX_FRAMES: c_int = 1024;       // per model
pub const MD3_MAX_SURFACES: c_int = 32 + 32;  // per model
pub const MD3_MAX_TAGS: c_int = 16;           // per frame

// vertex scales
pub const MD3_XYZ_SCALE: f32 = 1.0 / 64.0;

// Local stub for vec3_t - defined elsewhere in the codebase
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
    pub name: [c_char; MAX_QPATH as usize],    // tag name
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
    pub ident: c_int,               //

    pub name: [c_char; MAX_QPATH as usize],    // polyset name

    pub flags: c_int,
    pub numFrames: c_int,           // all surfaces in a model should have the same

    pub numShaders: c_int,          // all surfaces in a model should have the same
    pub numVerts: c_int,

    pub numTriangles: c_int,
    pub ofsTriangles: c_int,

    pub ofsShaders: c_int,          // offset from start of md3Surface_t
    pub ofsSt: c_int,               // texture coords are common for all frames
    pub ofsXyzNormals: c_int,       // numVerts * numFrames

    pub ofsEnd: c_int,              // next surface follows
}

#[repr(C)]
pub struct md3Shader_t {
    pub name: [c_char; MAX_QPATH as usize],
    pub shaderIndex: c_int,    // for in-game use
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
    pub xyz: [i16; 3],
    pub normal: i16,
}

#[repr(C)]
pub struct md3Header_t {
    pub ident: c_int,
    pub version: c_int,

    pub name: [c_char; MAX_QPATH as usize],    // model name

    pub flags: c_int,

    pub numFrames: c_int,
    pub numTags: c_int,
    pub numSurfaces: c_int,

    pub numSkins: c_int,

    pub ofsFrames: c_int,           // offset for first frame
    pub ofsTags: c_int,             // numFrames * numTags
    pub ofsSurfaces: c_int,         // first surface, others follow

    pub ofsEnd: c_int,              // end of file
}

/*
==============================================================================

  .BSP file format

==============================================================================
*/

// little-endian "RBSP"
pub const BSP_IDENT: c_int = (('P' as c_int) << 24) + (('S' as c_int) << 16) + (('B' as c_int) << 8) + ('R' as c_int);

pub const BSP_VERSION: c_int = 1;

// there shouldn't be any problem with increasing these values at the
// expense of more memory allocation in the utilities
pub const MAX_MAP_MODELS: c_int = 0x400;
pub const MAX_MAP_BRUSHES: c_int = 0x8000;
pub const MAX_MAP_ENTITIES: c_int = 0x800;
pub const MAX_MAP_ENTSTRING: c_int = 0x40000;
pub const MAX_MAP_SHADERS: c_int = 0x400;

pub const MAX_MAP_AREAS: c_int = 0x100;    // MAX_MAP_AREA_BYTES in q_shared must match!
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
pub const MAX_MAP_VISIBILITY: c_int = 0x600000;

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

#[cfg(feature = "XBOX")]
pub mod xbox {
    use super::*;

    #[repr(C)]
    pub struct dmodel_t {
        pub mins: [f32; 3],
        pub maxs: [f32; 3],
        pub firstSurface: c_int,
        pub numSurfaces: u16,
        pub firstBrush: c_int,
        pub numBrushes: u16,
    }

    #[repr(C)]
    pub struct dshader_t {
        pub shader: [c_char; super::MAX_QPATH as usize],
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
        pub children: [i16; 2],    // negative numbers are -(leafs+1), not nodes
        pub mins: [i16; 3],        // for frustom culling
        pub maxs: [i16; 3],
    }

    #[repr(C)]
    pub struct dleaf_t {
        pub cluster: i16,          // -1 = opaque cluster (do I still store these?)
        pub area: i8,

        pub mins: [i16; 3],        // for frustum culling
        pub maxs: [i16; 3],

        pub firstLeafSurface: u16,
        pub numLeafSurfaces: u16,

        pub firstLeafBrush: u16,
        pub numLeafBrushes: u16,
    }

    #[repr(C)]
    pub struct dbrushside_t {
        pub planeNum: c_int,       // positive plane side faces out of the leaf
        pub shaderNum: u8,
    }

    #[repr(C)]
    pub struct dbrush_t {
        pub firstSide: c_int,
        pub numSides: u8,
        pub shaderNum: u16,        // the shader that determines the contents flags
    }

    #[repr(C)]
    pub struct dfog_t {
        pub shader: [c_char; super::MAX_QPATH as usize],
        pub brushNum: c_int,
        pub visibleSide: c_int,    // the brush side that ray tests need to clip against (-1 == none)
    }

    // Light Style Constants
    pub const MAXLIGHTMAPS: c_int = 4;
    pub const LS_NORMAL: c_int = 0x00;
    pub const LS_UNUSED: c_int = 0xfe;
    pub const LS_LSNONE: c_int = 0xff;
    pub const MAX_LIGHT_STYLES: c_int = 64;

    #[repr(C)]
    pub struct mapVert_t {
        pub lightmap: [[f32; 2]; super::MAXLIGHTMAPS as usize],
        pub st: [f32; 2],
        pub xyz: [i16; 3],
        pub normal: [i16; 3],
        pub color: [[u8; 4]; super::MAXLIGHTMAPS as usize],
    }

    pub const DRAWVERT_LIGHTMAP_SCALE: f32 = 32768.0;
    // Change texture coordinates for TriSurfs to be even more fine grain.
    // See below for note about keeping MIN_ST and MAX_ST up to date with
    // ST_SCALE. These are in 4.12. OK, how about 5.11?
    //#define DRAWVERT_ST_SCALE 4096.0f
    pub const DRAWVERT_ST_SCALE: f32 = 2048.0;

    // We use a slightly different format for the fixed point texture
    // coords in Grid/Mesh drawverts: 10.6 rather than 12.4
    // To be sure that this is ok, keep the max and min values equal to
    // the largest and smallest whole numbers that can be stored using the
    // format. (ie: Don't change GRID_DRAWVERT_ST_SCALE without changing
    // the other two!) (And don't forget that we're using a bit for sign.)
    pub const GRID_DRAWVERT_ST_SCALE: f32 = 64.0;

    #[repr(C)]
    pub struct drawVert_t {
        pub xyz: [f32; 3],
        pub dvst: [i16; 2],
        pub dvlightmap: [[i16; 2]; super::MAXLIGHTMAPS as usize],
        pub normal: [f32; 3],
        #[cfg(feature = "XBOX")]
        pub tangent: [f32; 3],
        pub dvcolor: [[u8; 2]; super::MAXLIGHTMAPS as usize],
    }

    #[repr(C)]
    pub struct dgrid_t {
        pub flags: u8,
        pub latLong: [u8; 2],
    }

    #[repr(C)]
    pub struct dface_t {
        pub code: c_int,
        pub shaderNum: u8,
        pub fogNum: i8,

        pub verts: u32,            // high 20 bits are first vert, low 12 are num verts
        pub indexes: u32,          // high 20 bits are first index, low 12 are num indices

        pub lightmapStyles: [u8; super::MAXLIGHTMAPS as usize],
        pub lightmapNum: [u8; super::MAXLIGHTMAPS as usize],

        pub lightmapVecs: [i16; 3],
    }

    #[repr(C)]
    pub struct dpatch_t {
        pub code: c_int,
        pub shaderNum: u8,
        pub fogNum: i8,

        pub verts: u32,            // high 20 bits are first vert, low 12 are num verts

        pub lightmapStyles: [u8; super::MAXLIGHTMAPS as usize],
        pub lightmapNum: [u8; super::MAXLIGHTMAPS as usize],

        pub lightmapVecs: [[i16; 3]; 2],       // for patches, [0] and [1] are lodbounds

        pub patchWidth: u8,
        pub patchHeight: u8,
    }

    #[repr(C)]
    pub struct dtrisurf_t {
        pub code: c_int,
        pub shaderNum: u8,
        pub fogNum: i8,

        pub verts: u32,            // high 20 bits are first vert, low 12 are num verts
        pub indexes: u32,          // high 20 bits are first index, low 12 are num indices

        pub lightmapStyles: [u8; super::MAXLIGHTMAPS as usize],
    }

    #[repr(C)]
    pub struct dflare_t {
        pub code: c_int,
        pub shaderNum: u8,
        pub fogNum: i8,

        pub origin: [i16; 3],
        pub normal: [i16; 3],
        pub color: [u8; 3],
    }
}

#[cfg(not(feature = "XBOX"))]
pub mod non_xbox {
    use super::*;

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

        pub lumps: [lump_t; super::HEADER_LUMPS as usize],
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
        pub shader: [c_char; super::MAX_QPATH as usize],
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
        pub children: [c_int; 2],  // negative numbers are -(leafs+1), not nodes
        pub mins: [c_int; 3],      // for frustom culling
        pub maxs: [c_int; 3],
    }

    #[repr(C)]
    pub struct dleaf_t {
        pub cluster: c_int,        // -1 = opaque cluster (do I still store these?)
        pub area: c_int,

        pub mins: [c_int; 3],      // for frustum culling
        pub maxs: [c_int; 3],

        pub firstLeafSurface: c_int,
        pub numLeafSurfaces: c_int,

        pub firstLeafBrush: c_int,
        pub numLeafBrushes: c_int,
    }

    #[repr(C)]
    pub struct dbrushside_t {
        pub planeNum: c_int,       // positive plane side faces out of the leaf
        pub shaderNum: c_int,
        pub drawSurfNum: c_int,
    }

    #[repr(C)]
    pub struct dbrush_t {
        pub firstSide: c_int,
        pub numSides: c_int,
        pub shaderNum: c_int,      // the shader that determines the contents flags
    }

    #[repr(C)]
    pub struct dfog_t {
        pub shader: [c_char; super::MAX_QPATH as usize],
        pub brushNum: c_int,
        pub visibleSide: c_int,    // the brush side that ray tests need to clip against (-1 == none)
    }

    // Light Style Constants
    pub const MAXLIGHTMAPS: c_int = 4;
    pub const LS_NORMAL: c_int = 0x00;
    pub const LS_UNUSED: c_int = 0xfe;
    pub const LS_LSNONE: c_int = 0xff; //rww - changed name because it unhappily conflicts with a lightsaber state name and changing this is just easier
    pub const MAX_LIGHT_STYLES: c_int = 64;

    #[repr(C)]
    pub struct mapVert_t {
        pub xyz: [f32; 3],
        pub st: [f32; 2],
        pub lightmap: [[f32; 2]; super::MAXLIGHTMAPS as usize],
        pub normal: [f32; 3],
        pub color: [[u8; 4]; super::MAXLIGHTMAPS as usize],
    }

    #[repr(C)]
    pub struct drawVert_t {
        pub xyz: [f32; 3],
        pub st: [f32; 2],
        pub lightmap: [[f32; 2]; super::MAXLIGHTMAPS as usize],
        pub normal: [f32; 3],
        pub color: [[u8; 4]; super::MAXLIGHTMAPS as usize],
    }

    #[repr(C)]
    pub struct dgrid_t {
        pub ambientLight: [[u8; 3]; super::MAXLIGHTMAPS as usize],
        pub directLight: [[u8; 3]; super::MAXLIGHTMAPS as usize],
        pub styles: [u8; super::MAXLIGHTMAPS as usize],
        pub latLong: [u8; 2],
    }

    pub enum mapSurfaceType_t {
        MST_BAD = 0,
        MST_PLANAR = 1,
        MST_PATCH = 2,
        MST_TRIANGLE_SOUP = 3,
        MST_FLARE = 4,
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

        pub lightmapStyles: [u8; super::MAXLIGHTMAPS as usize],
        pub vertexStyles: [u8; super::MAXLIGHTMAPS as usize],
        pub lightmapNum: [c_int; super::MAXLIGHTMAPS as usize],
        pub lightmapX: [c_int; super::MAXLIGHTMAPS as usize],
        pub lightmapY: [c_int; super::MAXLIGHTMAPS as usize],
        pub lightmapWidth: c_int,
        pub lightmapHeight: c_int,

        pub lightmapOrigin: [f32; 3],
        pub lightmapVecs: [[f32; 3]; 3],       // for patches, [0] and [1] are lodbounds

        pub patchWidth: c_int,
        pub patchHeight: c_int,
    }
}

/////////////////////////////////////////////////////////////
//
// Defines and structures required for fonts

pub const GLYPH_COUNT: c_int = 256;

// Must match define in stmparse.h
pub const STYLE_DROPSHADOW: c_int = 0x80000000;
pub const STYLE_BLINK: c_int = 0x40000000;
pub const SET_MASK: c_int = 0x00ffffff;

#[repr(C)]
pub struct glyphInfo_t {
    pub width: i16,                // number of pixels wide
    pub height: i16,               // number of scan lines
    pub horizAdvance: i16,         // number of pixels to advance to the next char
    pub horizOffset: i16,          // x offset into space to render glyph
    pub baseline: c_int,           // y offset
    pub s: f32,                    // x start tex coord
    pub t: f32,                    // y start tex coord
    pub s2: f32,                   // x end tex coord
    pub t2: f32,                   // y end tex coord
}

// this file corresponds 1:1 with the "*.fontdat" files, so don't change it unless you're going to
//	recompile the fontgen util and regenerate all the fonts!
//
#[repr(C)]
pub struct dfontdat_t {
    pub mGlyphs: [glyphInfo_t; GLYPH_COUNT as usize],

    pub mPointSize: i16,
    pub mHeight: i16,              // max height of font
    pub mAscender: i16,
    pub mDescender: i16,

    pub mKoreanHack: i16,
}

/////////////////// fonts end ////////////////////////////////////
