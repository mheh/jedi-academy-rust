#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use crate::codemp::game::q_shared_h::{
    byte, qboolean, qhandle_t, vec2_t, vec3_t, vec_t,
};

// =============================================================================
// Types from ../cgame/tr_types.h (local stubs for structural coherence)
// =============================================================================

/// `REF_API_VERSION` — the version number of the refexport_t interface.
pub const REF_API_VERSION: c_int = 8;

/// `color4ub_t` (tr_types.h) — a 4-component byte color (RGBA).
pub type color4ub_t = [byte; 4];

/// `polyVert_t` (tr_types.h) — a single vertex for a poly with position, texture coords, and modulation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct polyVert_t {
    pub xyz: vec3_t,
    pub st: vec2_t,
    pub modulate: color4ub_t,
}

/// `poly_t` (tr_types.h) — a single poly with a shader handle, vertices, and vertex count.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct poly_t {
    pub hShader: qhandle_t,
    pub numVerts: c_int,
    pub verts: *mut polyVert_t,
}

/// `refEntityType_t` (tr_types.h) — enum of reference entity types.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum refEntityType_t {
    RT_MODEL = 0,
    RT_POLY = 1,
    RT_SPRITE = 2,
    RT_ORIENTED_QUAD = 3,
    RT_BEAM = 4,
    RT_SABER_GLOW = 5,
    RT_ELECTRICITY = 6,
    RT_PORTALSURFACE = 7,
    RT_LINE = 8,
    RT_ORIENTEDLINE = 9,
    RT_CYLINDER = 10,
    RT_ENT_CHAIN = 11,
    RT_MAX_REF_ENTITY_TYPE = 12,
}

/// `miniRefEntity_t` (tr_types.h) — a minimal ref entity with core rendering data.
/// This structure must remain identical as defined in tr_types.h.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct miniRefEntity_t {
    pub reType: refEntityType_t,
    pub renderfx: c_int,
    pub hModel: qhandle_t,
    pub axis: [vec3_t; 3],
    pub nonNormalizedAxes: qboolean,
    pub origin: vec3_t,
    pub oldorigin: vec3_t,
    pub customShader: qhandle_t,
    pub shaderRGBA: [byte; 4],
    pub shaderTexCoord: vec2_t,
    pub radius: vec_t,
    pub rotation: vec_t,
    pub shaderTime: vec_t,
    pub frame: c_int,
}

/// `refEntity_t` (tr_types.h) — a full ref entity with extended data beyond miniRefEntity_t.
/// The first part must remain identical as miniRefEntity_t.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct refEntity_t {
    // Begin miniRefEntity_t portion
    pub reType: refEntityType_t,
    pub renderfx: c_int,
    pub hModel: qhandle_t,
    pub axis: [vec3_t; 3],
    pub nonNormalizedAxes: qboolean,
    pub origin: vec3_t,
    pub oldorigin: vec3_t,
    pub customShader: qhandle_t,
    pub shaderRGBA: [byte; 4],
    pub shaderTexCoord: vec2_t,
    pub radius: vec_t,
    pub rotation: vec_t,
    pub shaderTime: vec_t,
    pub frame: c_int,
    // End miniRefEntity_t portion
    pub lightingOrigin: vec3_t,
    pub shadowPlane: vec_t,
    pub oldframe: c_int,
    pub backlerp: vec_t,
    pub skinNum: c_int,
    pub customSkin: qhandle_t,
    pub uRefEnt: RefEnt_uUnion,
    pub data: RefEntity_DataUnion,
    pub endTime: vec_t,
    pub saberLength: vec_t,
    pub angles: vec3_t,
    pub modelScale: vec3_t,
    pub ghoul2: *mut c_void,
}

/// Union for refEntity_t.uRefEnt field
#[repr(C)]
#[derive(Clone, Copy)]
pub union RefEnt_uUnion {
    pub uMini: RefEnt_uMini,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RefEnt_uMini {
    pub miniStart: c_int,
    pub miniCount: c_int,
}

/// Union for refEntity_t.data field (sprite/line/bezier/cylinder/electricity data)
#[repr(C)]
#[derive(Clone, Copy)]
pub union RefEntity_DataUnion {
    pub sprite: RefEntity_Sprite,
    pub line: RefEntity_Line,
    pub bezier: RefEntity_Bezier,
    pub cylinder: RefEntity_Cylinder,
    pub electricity: RefEntity_Electricity,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RefEntity_Sprite {
    pub rotation: vec_t,
    pub radius: vec_t,
    pub vertRGBA: [[byte; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RefEntity_Line {
    pub width: vec_t,
    pub width2: vec_t,
    pub stscale: vec_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RefEntity_Bezier {
    pub width: vec_t,
    pub control1: vec3_t,
    pub control2: vec3_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RefEntity_Cylinder {
    pub width: vec_t,
    pub width2: vec_t,
    pub stscale: vec_t,
    pub height: vec_t,
    pub bias: vec_t,
    pub wrap: qboolean,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RefEntity_Electricity {
    pub width: vec_t,
    pub deviation: vec_t,
    pub stscale: vec_t,
    pub wrap: qboolean,
    pub taper: qboolean,
}

/// `orientation_t` (tr_types.h) — origin and axis transformation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct orientation_t {
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
}

/// `markFragment_t` (q_shared.h) — a single mark fragment from CM_MarkFragments().
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct markFragment_t {
    pub firstPoint: c_int,
    pub numPoints: c_int,
}

/// `MAX_MAP_AREA_BYTES` — bit vector size for area visibility (q_shared.h).
pub const MAX_MAP_AREA_BYTES: usize = 32;

/// `MAX_RENDER_STRINGS` — maximum number of text message strings in refdef_t.
pub const MAX_RENDER_STRINGS: usize = 8;

/// `MAX_RENDER_STRING_LENGTH` — maximum length of each text message string.
pub const MAX_RENDER_STRING_LENGTH: usize = 32;

/// `refdef_t` (tr_types.h) — the rendering definition passed to the refresh module.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct refdef_t {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub fov_x: vec_t,
    pub fov_y: vec_t,
    pub vieworg: vec3_t,
    pub viewangles: vec3_t,
    pub viewaxis: [vec3_t; 3],
    pub viewContents: c_int,
    pub time: c_int,
    pub rdflags: c_int,
    pub areamask: [byte; MAX_MAP_AREA_BYTES],
    pub text: [[c_char; MAX_RENDER_STRING_LENGTH]; MAX_RENDER_STRINGS],
}

/// `stereoFrame_t` (tr_types.h) — stereo rendering frame type.
pub type stereoFrame_t = c_int;

/// `STEREO_CENTER` — center stereo frame.
pub const STEREO_CENTER: stereoFrame_t = 0;

/// `STEREO_LEFT` — left stereo frame.
pub const STEREO_LEFT: stereoFrame_t = 1;

/// `STEREO_RIGHT` — right stereo frame.
pub const STEREO_RIGHT: stereoFrame_t = 2;

/// `textureCompression_t` (tr_types.h) — enum of texture compression types.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum textureCompression_t {
    TC_NONE = 0,
    TC_S3TC = 1,
    TC_S3TC_DXT = 2,
}

/// `glconfig_t` (tr_types.h) — OpenGL configuration and capability information.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct glconfig_t {
    pub renderer_string: *const c_char,
    pub vendor_string: *const c_char,
    pub version_string: *const c_char,
    pub extensions_string: *const c_char,
    pub maxTextureSize: c_int,
    pub maxActiveTextures: c_int,
    pub maxTextureFilterAnisotropy: vec_t,
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
    pub isFullscreen: qboolean,
    pub stereoEnabled: qboolean,
}

// =============================================================================
// Callback function types for refexport_t
// =============================================================================

/// Type alias for a Shutdown function callback.
pub type Shutdown_fn = extern "C" fn(qboolean);

/// Type alias for a BeginRegistration function callback.
pub type BeginRegistration_fn = extern "C" fn(*mut glconfig_t);

/// Type alias for a RegisterModel function callback.
pub type RegisterModel_fn = extern "C" fn(*const c_char) -> qhandle_t;

/// Type alias for a RegisterSkin function callback.
pub type RegisterSkin_fn = extern "C" fn(*const c_char) -> qhandle_t;

/// Type alias for a RegisterShader function callback.
pub type RegisterShader_fn = extern "C" fn(*const c_char) -> qhandle_t;

/// Type alias for a RegisterShaderNoMip function callback.
pub type RegisterShaderNoMip_fn = extern "C" fn(*const c_char) -> qhandle_t;

/// Type alias for a ShaderNameFromIndex function callback.
pub type ShaderNameFromIndex_fn = extern "C" fn(c_int) -> *const c_char;

/// Type alias for a LoadWorld function callback.
pub type LoadWorld_fn = extern "C" fn(*const c_char);

/// Type alias for a SetWorldVisData function callback.
pub type SetWorldVisData_fn = extern "C" fn(*const byte);

/// Type alias for an EndRegistration function callback.
pub type EndRegistration_fn = extern "C" fn();

/// Type alias for a ClearScene function callback.
pub type ClearScene_fn = extern "C" fn();

/// Type alias for a ClearDecals function callback.
pub type ClearDecals_fn = extern "C" fn();

/// Type alias for an AddRefEntityToScene function callback.
pub type AddRefEntityToScene_fn = extern "C" fn(*const refEntity_t);

/// Type alias for an AddMiniRefEntityToScene function callback.
pub type AddMiniRefEntityToScene_fn = extern "C" fn(*const miniRefEntity_t);

/// Type alias for an AddPolyToScene function callback.
pub type AddPolyToScene_fn = extern "C" fn(qhandle_t, c_int, *const polyVert_t, c_int);

/// Type alias for an AddDecalToScene function callback.
pub type AddDecalToScene_fn = extern "C" fn(
    qhandle_t,
    *const vec3_t,
    *const vec3_t,
    vec_t,
    vec_t,
    vec_t,
    vec_t,
    vec_t,
    qboolean,
    vec_t,
    qboolean,
);

/// Type alias for a LightForPoint function callback.
pub type LightForPoint_fn = extern "C" fn(*mut vec3_t, *mut vec3_t, *mut vec3_t, *mut vec3_t) -> c_int;

/// Type alias for an AddLightToScene function callback.
pub type AddLightToScene_fn = extern "C" fn(*const vec3_t, vec_t, vec_t, vec_t, vec_t);

/// Type alias for an AddAdditiveLightToScene function callback.
pub type AddAdditiveLightToScene_fn = extern "C" fn(*const vec3_t, vec_t, vec_t, vec_t, vec_t);

/// Type alias for a RenderScene function callback.
pub type RenderScene_fn = extern "C" fn(*const refdef_t);

/// Type alias for a SetColor function callback.
pub type SetColor_fn = extern "C" fn(*const vec_t);

/// Type alias for a DrawStretchPic function callback.
pub type DrawStretchPic_fn = extern "C" fn(vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, qhandle_t);

/// Type alias for a DrawRotatePic function callback.
pub type DrawRotatePic_fn = extern "C" fn(vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, qhandle_t);

/// Type alias for a DrawRotatePic2 function callback.
pub type DrawRotatePic2_fn = extern "C" fn(vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, vec_t, qhandle_t);

/// Type alias for a DrawStretchRaw function callback.
pub type DrawStretchRaw_fn = extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, *const byte, c_int, qboolean);

/// Type alias for an UploadCinematic function callback.
pub type UploadCinematic_fn = extern "C" fn(c_int, c_int, *const byte, c_int, qboolean);

/// Type alias for a BeginFrame function callback.
pub type BeginFrame_fn = extern "C" fn(stereoFrame_t);

/// Type alias for an EndFrame function callback.
pub type EndFrame_fn = extern "C" fn(*mut c_int, *mut c_int);

/// Type alias for a MarkFragments function callback.
pub type MarkFragments_fn = extern "C" fn(c_int, *const vec3_t, *const vec3_t, c_int, *mut vec3_t, c_int, *mut markFragment_t) -> c_int;

/// Type alias for a LerpTag function callback.
pub type LerpTag_fn = extern "C" fn(*mut orientation_t, qhandle_t, c_int, c_int, vec_t, *const c_char) -> c_int;

/// Type alias for a ModelBounds function callback.
pub type ModelBounds_fn = extern "C" fn(qhandle_t, *mut vec3_t, *mut vec3_t);

/// Type alias for a RegisterFont function callback.
pub type RegisterFont_fn = extern "C" fn(*const c_char) -> qhandle_t;

/// Type alias for a Font_StrLenPixels function callback.
pub type Font_StrLenPixels_fn = extern "C" fn(*const c_char, c_int, vec_t) -> c_int;

/// Type alias for a Font_StrLenChars function callback.
pub type Font_StrLenChars_fn = extern "C" fn(*const c_char) -> c_int;

/// Type alias for a Font_HeightPixels function callback.
pub type Font_HeightPixels_fn = extern "C" fn(c_int, vec_t) -> c_int;

/// Type alias for a Font_DrawString function callback.
pub type Font_DrawString_fn = extern "C" fn(c_int, c_int, *const c_char, *const vec_t, c_int, c_int, vec_t);

/// Type alias for a Language_IsAsian function callback.
pub type Language_IsAsian_fn = extern "C" fn() -> qboolean;

/// Type alias for a Language_UsesSpaces function callback.
pub type Language_UsesSpaces_fn = extern "C" fn() -> qboolean;

/// Type alias for an AnyLanguage_ReadCharFromString function callback.
pub type AnyLanguage_ReadCharFromString_fn =
    extern "C" fn(*const c_char, *mut c_int, *mut qboolean) -> c_int;

/// Type alias for a RemapShader function callback.
pub type RemapShader_fn = extern "C" fn(*const c_char, *const c_char, *const c_char);

/// Type alias for a GetEntityToken function callback.
pub type GetEntityToken_fn = extern "C" fn(*mut c_char, c_int) -> qboolean;

/// Type alias for an inPVS function callback.
pub type inPVS_fn = extern "C" fn(*const vec3_t, *const vec3_t, *mut byte) -> qboolean;

/// Type alias for a GetLightStyle function callback.
pub type GetLightStyle_fn = extern "C" fn(c_int, *mut color4ub_t);

/// Type alias for a SetLightStyle function callback.
pub type SetLightStyle_fn = extern "C" fn(c_int, c_int);

/// Type alias for a GetBModelVerts function callback.
pub type GetBModelVerts_fn = extern "C" fn(c_int, *mut vec3_t, *mut vec3_t);

// =============================================================================
// refexport_t — the main API export from the refresh module
// =============================================================================

//
// these are the functions exported by the refresh module
//
/// `refexport_t` (tr_public.h) — the function pointers exported by the renderer module.
/// All data that will be used in a level should be registered before rendering any frames
/// to prevent disk hits, but they can still be registered at a later time if necessary.
///
/// `BeginRegistration` makes any existing media pointers invalid and returns the current GL
/// configuration, including screen width and height, which can be used by the client to
/// intelligently size display elements.
///
/// A scene is built up by calls to `ClearScene` and the various `R_Add` functions.
/// Nothing is drawn until `RenderScene` is called.
#[repr(C)]
pub struct refexport_t {
    // called before the library is unloaded
    // if the system is just reconfiguring, pass destroyWindow = qfalse,
    // which will keep the screen from flashing to the desktop.
    pub Shutdown: Option<Shutdown_fn>,

    // All data that will be used in a level should be
    // registered before rendering any frames to prevent disk hits,
    // but they can still be registered at a later time
    // if necessary.
    //
    // BeginRegistration makes any existing media pointers invalid
    // and returns the current gl configuration, including screen width
    // and height, which can be used by the client to intelligently
    // size display elements
    pub BeginRegistration: Option<BeginRegistration_fn>,
    pub RegisterModel: Option<RegisterModel_fn>,
    pub RegisterSkin: Option<RegisterSkin_fn>,
    pub RegisterShader: Option<RegisterShader_fn>,
    pub RegisterShaderNoMip: Option<RegisterShaderNoMip_fn>,
    pub ShaderNameFromIndex: Option<ShaderNameFromIndex_fn>,
    pub LoadWorld: Option<LoadWorld_fn>,

    // the vis data is a large enough block of data that we go to the trouble
    // of sharing it with the clipmodel subsystem
    pub SetWorldVisData: Option<SetWorldVisData_fn>,

    // EndRegistration will draw a tiny polygon with each texture, forcing
    // them to be loaded into card memory
    pub EndRegistration: Option<EndRegistration_fn>,

    // a scene is built up by calls to R_ClearScene and the various R_Add functions.
    // Nothing is drawn until R_RenderScene is called.
    pub ClearScene: Option<ClearScene_fn>,
    pub ClearDecals: Option<ClearDecals_fn>,
    pub AddRefEntityToScene: Option<AddRefEntityToScene_fn>,
    pub AddMiniRefEntityToScene: Option<AddMiniRefEntityToScene_fn>,
    pub AddPolyToScene: Option<AddPolyToScene_fn>,
    pub AddDecalToScene: Option<AddDecalToScene_fn>,
    pub LightForPoint: Option<LightForPoint_fn>,
    pub AddLightToScene: Option<AddLightToScene_fn>,
    pub AddAdditiveLightToScene: Option<AddAdditiveLightToScene_fn>,
    pub RenderScene: Option<RenderScene_fn>,

    pub SetColor: Option<SetColor_fn>, // NULL = 1,1,1,1
    pub DrawStretchPic: Option<DrawStretchPic_fn>, // 0 = white
    pub DrawRotatePic: Option<DrawRotatePic_fn>, // 0 = white
    pub DrawRotatePic2: Option<DrawRotatePic2_fn>, // 0 = white

    // Draw images for cinematic rendering, pass as 32 bit rgba
    pub DrawStretchRaw: Option<DrawStretchRaw_fn>,
    pub UploadCinematic: Option<UploadCinematic_fn>,

    pub BeginFrame: Option<BeginFrame_fn>,

    // if the pointers are not NULL, timing info will be returned
    pub EndFrame: Option<EndFrame_fn>,

    pub MarkFragments: Option<MarkFragments_fn>,

    pub LerpTag: Option<LerpTag_fn>,
    pub ModelBounds: Option<ModelBounds_fn>,

    pub RegisterFont: Option<RegisterFont_fn>,
    pub Font_StrLenPixels: Option<Font_StrLenPixels_fn>,
    pub Font_StrLenChars: Option<Font_StrLenChars_fn>,
    pub Font_HeightPixels: Option<Font_HeightPixels_fn>,
    pub Font_DrawString: Option<Font_DrawString_fn>,
    pub Language_IsAsian: Option<Language_IsAsian_fn>,
    pub Language_UsesSpaces: Option<Language_UsesSpaces_fn>,
    pub AnyLanguage_ReadCharFromString: Option<AnyLanguage_ReadCharFromString_fn>,

    pub RemapShader: Option<RemapShader_fn>,
    pub GetEntityToken: Option<GetEntityToken_fn>,
    pub inPVS: Option<inPVS_fn>,

    pub GetLightStyle: Option<GetLightStyle_fn>,
    pub SetLightStyle: Option<SetLightStyle_fn>,

    pub GetBModelVerts: Option<GetBModelVerts_fn>,
}

// =============================================================================
// Module-level export function
// =============================================================================

// this is the only function actually exported at the linker level
// If the module can't init to a valid rendering state, NULL will be
// returned.
extern "C" {
    /// `GetRefAPI` — retrieve the refexport_t interface from the renderer module.
    /// This is the only function actually exported at the linker level.
    /// If the module can't init to a valid rendering state, NULL will be returned.
    pub fn GetRefAPI(apiVersion: c_int) -> *mut refexport_t;
}
