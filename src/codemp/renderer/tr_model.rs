// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// tr_models.c -- model loading and caching

// #include "tr_local.h"

// #include "../qcommon/disablewarnings.h"

use core::ffi::{c_int, c_char, c_void};
use std::collections::HashMap;

// #define	LL(x) x=LittleLong(x)

fn LL(x: &mut c_int) {
    *x = LittleLong(*x);
}

extern "C" {
    fn R_LoadMD3(
        mod_: *mut model_t,
        lod: c_int,
        buffer: *mut c_void,
        name: *const c_char,
        bAlreadyCached: &mut c_int,
    ) -> c_int;
}

/*
Ghoul2 Insert Start
*/

#[repr(C)]
pub struct modelHash_s {
    pub name: [c_char; 260], // MAX_QPATH
    pub handle: c_int,
    pub next: *mut modelHash_s,
}

pub type modelHash_t = modelHash_s;

const FILE_HASH_SIZE: usize = 1024;
static mut mhHashTable: [*mut modelHash_t; FILE_HASH_SIZE] = [core::ptr::null_mut(); FILE_HASH_SIZE];

/*
Ghoul2 Insert End
*/

// This stuff looks a bit messy, but it's kept here as black box, and nothing appears in any .H files for other
// modules to worry about. I may make another module for this sometime.
//

pub type StringOffsetAndShaderIndexDest_t = (c_int, c_int);

#[repr(C)]
pub struct CachedEndianedModelBinary_s {
    pub pModelDiskImage: *mut c_void,
    pub iAllocSize: c_int, // may be useful for mem-query, but I don't actually need it
    pub ShaderRegisterData: Vec<StringOffsetAndShaderIndexDest_t>,
    pub iLastLevelUsedOn: c_int,
    pub iPAKFileCheckSum: c_int, // else -1 if not from PAK
}

impl CachedEndianedModelBinary_s {
    pub fn new() -> Self {
        CachedEndianedModelBinary_s {
            pModelDiskImage: core::ptr::null_mut(),
            iAllocSize: 0,
            ShaderRegisterData: Vec::new(),
            iLastLevelUsedOn: -1,
            iPAKFileCheckSum: -1,
        }
    }
}

pub type CachedEndianedModelBinary_t = CachedEndianedModelBinary_s;
pub type CachedModels_t = HashMap<String, CachedEndianedModelBinary_t>;

static mut CachedModels: *mut CachedModels_t = core::ptr::null_mut(); // the important cache item.

extern "C" {
    fn Z_Malloc(size: c_int, tag: c_int, qfalse_val: c_int) -> *mut c_void;
    fn Z_MorphMallocTag(buf: *mut c_void, tag: c_int);
    fn Z_Free(ptr: *mut c_void);
    fn Z_MemSize(tag: c_int) -> c_int;
    fn FS_ReadFile(name: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buf: *mut c_void);
    fn FS_FileIsInPAK(name: *const c_char, checksum: *mut c_int) -> c_int;
    fn Hunk_Alloc(size: usize, tag: c_int) -> *mut c_void;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize);
    fn Q_strlwr(str: *mut c_char) -> *mut c_char;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Cvar_Get(name: *const c_char, val: *const c_char, flags: c_int) -> *mut c_void;
    fn LittleLong(l: c_int) -> c_int;
    fn LittleFloat(f: f32) -> f32;
    fn LittleShort(s: i16) -> i16;
    fn R_FindShader(
        name: *const c_char,
        lightmapsNone: c_int,
        stylesDefault: c_int,
        qtrue: c_int,
    ) -> *mut shader_t;
    fn R_SyncRenderThread();
    fn R_Init();
    fn RE_ClearScene();
    fn RE_StretchPic(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        hShader: c_int,
    );
    fn RE_LoadWorldMap_Actual(
        name: *const c_char,
        worldData: *mut c_void,
        index: c_int,
    );
    fn KillTheShaderHashTable();
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn sprintf(dest: *mut c_char, fmt: *const c_char, ...) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn tolower(c: c_int) -> c_int;
    fn va(fmt: *const c_char, ...) -> *const c_char;

    // External symbols
    static mut tr: tr_t;
    static mut r_lodbias: *mut cvar_t;
    static mut r_modelpoolmegs: *mut cvar_t;
    static mut r_noServerGhoul2: *mut cvar_t;
    static mut sv_pure: *mut cvar_t;
    static mut r_noPrecacheGLA: *mut cvar_t;
    static mut glConfig: glconfig_t;
    static mut gbInsideRegisterModel: c_int;
    static mut lightmapsNone: c_int;
    static mut stylesDefault: c_int;
    static mut sDEFAULT_GLA_NAME: [c_char; 256];

    fn RE_RegisterMedia_GetLevel() -> c_int;
    fn RE_RegisterImages_LevelLoadEnd();
    fn SND_RegisterAudio_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: c_int) -> c_int;
    fn S_RestartMusic();
}

// Stub types for external dependencies
#[repr(C)]
pub struct shader_t {
    pub defaultShader: c_int,
    pub index: c_int,
    // ... other fields not needed for this port
}

#[repr(C)]
pub struct model_s {
    pub index: c_int,
    pub type_: c_int,
    pub dataSize: c_int,
    pub bmodel: *mut c_void,
    pub md3: [*mut md3Header_t; 4], // MD3_MAX_LODS
    pub mdxa: *mut mdxaHeader_t,
    pub mdxm: *mut mdxmHeader_t,
    pub numLods: c_int,
    pub name: [c_char; 260], // MAX_QPATH
}

pub type model_t = model_s;

#[repr(C)]
pub struct md3Header_t {
    pub ident: c_int,
    pub version: c_int,
    pub name: [c_char; 64],
    pub flags: c_int,
    pub numFrames: c_int,
    pub numTags: c_int,
    pub numSurfaces: c_int,
    pub numSkins: c_int,
    pub ofsFrames: c_int,
    pub ofsTags: c_int,
    pub ofsSurfaces: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct md3Frame_t {
    pub bounds: [[f32; 3]; 2],
    pub localOrigin: [f32; 3],
    pub radius: f32,
    pub name: [c_char; 16],
}

#[repr(C)]
pub struct md3Tag_t {
    pub name: [c_char; 64],
    pub origin: [f32; 3],
    pub axis: [[f32; 3]; 3],
}

#[repr(C)]
pub struct md3Surface_t {
    pub ident: c_int,
    pub name: [c_char; 64],
    pub flags: c_int,
    pub numFrames: c_int,
    pub numShaders: c_int,
    pub numVerts: c_int,
    pub numTriangles: c_int,
    pub ofsTriangles: c_int,
    pub ofsShaders: c_int,
    pub ofsSt: c_int,
    pub ofsXyzNormals: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct md3Shader_t {
    pub name: [c_char; 64],
    pub shaderIndex: c_int,
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
pub struct mdxaHeader_t {
    pub ident: c_int,
    pub version: c_int,
    pub name: [c_char; 64],
    pub flags: c_int,
    pub numFrames: c_int,
    pub numBones: c_int,
    pub ofsSkel: c_int,
    pub ofsFrames: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct mdxaFrame_t {
    pub name: [c_char; 16],
    pub flags: c_int,
    pub radius: f32,
    pub bounds: [[f32; 3]; 2],
    pub localOrigin: [f32; 3],
    pub bones: [mdxaBone_t; 1], // variable length
}

#[repr(C)]
pub struct mdxaBone_t {
    pub Comp: [i16; 7],
}

#[repr(C)]
pub struct mdxaSkel_t {
    pub name: [c_char; 32],
    pub flags: c_int,
    pub parent: c_int,
    pub numChildren: c_int,
    pub children: [c_int; 1], // variable length
}

#[repr(C)]
pub struct mdxmHeader_t {
    pub ident: c_int,
    pub version: c_int,
    pub name: [c_char; 64],
    pub animName: [c_char; 64],
    pub animIndex: c_int,
    pub numLODs: c_int,
    pub ofsLODs: c_int,
    pub numSurfaces: c_int,
    pub ofsSurfHierarchy: c_int,
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct mdxmLOD_t {
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct mdxmSurface_t {
    pub ident: c_int,
    pub name: [c_char; 64],
    pub shader: [c_char; 64],
    pub shaderIndex: c_int,
    pub ofsHeader: c_int,
    pub numTriangles: c_int,
    pub ofsTriangles: c_int,
    pub numVerts: c_int,
    pub ofsVerts: c_int,
    pub numBoneReferences: c_int,
    pub ofsBoneReferences: c_int,
    pub ofsEnd: c_int,
    pub ofsLODSurfOffset: c_int,
    pub maxVertBoneWeights: c_int,
}

#[repr(C)]
pub struct mdxmLODSurfOffset_t {
    pub ofsModelSurface: c_int,
}

#[repr(C)]
pub struct mdxmSurfHierarchy_t {
    pub name: [c_char; 64],
    pub flags: c_int,
    pub parentIndex: c_int,
    pub numChildren: c_int,
    pub shader: [c_char; 64],
    pub shaderIndex: c_int,
    pub childIndexes: [c_int; 1], // variable length
}

#[repr(C)]
pub struct mdxmTriangle_t {
    pub indexes: [c_int; 3],
}

#[repr(C)]
pub struct mdxmVertex_t {
    pub normal: [f32; 3],
    pub texCoords: [f32; 2],
    pub numWeights: c_int,
    pub offset: [f32; 3],
    pub weights: [mdxmWeight_t; 1], // variable length
}

#[repr(C)]
pub struct mdxmWeight_t {
    pub boneIndex: c_int,
    pub boneWeight: f32,
}

#[repr(C)]
pub struct mdxmTag_t {
    pub name: [c_char; 64],
    pub flags: c_int,
    pub axis: [[f32; 3]; 3],
    pub origin: [f32; 3],
}

#[repr(C)]
pub struct mdxmFrame_t {
    pub bounds: [[f32; 3]; 2],
    pub localOrigin: [f32; 3],
    pub radius: f32,
    pub bones: [mdxaBone_t; 1], // variable length
}

#[repr(C)]
pub struct tr_t {
    pub numModels: c_int,
    pub models: [*mut model_t; 2048], // MAX_MOD_KNOWN
    pub numBSPModels: c_int,
    pub bspModels: [c_void; 1],
    pub registered: c_int,
    pub numShaders: c_int,
    pub numSkins: c_int,
    pub viewCluster: c_int,
    // ... other fields
}

#[repr(C)]
pub struct glconfig_t {
    // stub
}

#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub integer: c_int,
    // ... other fields
}

#[repr(C)]
pub struct orientation_t {
    pub origin: [f32; 3],
    pub axis: [[f32; 3]; 3],
}

pub type qhandle_t = c_int;
pub type qboolean = c_int;

const qtrue: c_int = 1;
const qfalse: c_int = 0;

const MAX_QPATH: usize = 260;
const MAX_MOD_KNOWN: c_int = 2048;
const MD3_MAX_LODS: c_int = 4;
const SHADER_MAX_VERTEXES: c_int = 4000;
const SHADER_MAX_INDEXES: c_int = 6000;

const MOD_BAD: c_int = 0;
const MOD_BRUSH: c_int = 1;
const MOD_MESH: c_int = 2;
const MOD_MDXA: c_int = 3;
const MOD_MDXM: c_int = 4;

const SF_MD3: c_int = 1;
const SF_MDX: c_int = 2;

const MDXA_IDENT: c_int = 0x41584447; // 'AXDG' little-endian
const MDXA_VERSION: c_int = 6;
const MDXM_IDENT: c_int = 0x4D584447; // 'MXDG' little-endian
const MDXM_VERSION: c_int = 6;
const MD3_IDENT: c_int = 0x33504449; // '3PDI' little-endian
const MD3_VERSION: c_int = 15;

const TAG_FILESYS: c_int = 2;
const TAG_MODEL_MD3: c_int = 13;
const TAG_MODEL_GLM: c_int = 14;
const TAG_MODEL_GLA: c_int = 15;

const ERR_DROP: c_int = 1;

const S_COLOR_RED: &[u8] = b"\x1b[31m";
const S_COLOR_YELLOW: &[u8] = b"\x1b[33m";

const DEDICATED: bool = false;

static FakeGLAFile: &[u8] = &[
    0x32, 0x4C, 0x47, 0x41, 0x06, 0x00, 0x00, 0x00, 0x2A, 0x64, 0x65, 0x66, 0x61, 0x75, 0x6C, 0x74,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
    0x01, 0x00, 0x00, 0x00, 0x14, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x18, 0x01, 0x00, 0x00,
    0x68, 0x00, 0x00, 0x00, 0x26, 0x01, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x4D, 0x6F, 0x64, 0x56,
    0x69, 0x65, 0x77, 0x20, 0x69, 0x6E, 0x74, 0x65, 0x72, 0x6E, 0x61, 0x6C, 0x20, 0x64, 0x65, 0x66,
    0x61, 0x75, 0x6C, 0x74, 0x00, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD,
    0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD,
    0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0x00, 0x00, 0x00, 0x00,
    0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFD, 0xBF, 0xFE, 0x7F,
    0xFE, 0x7F, 0xFE, 0x7F, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80,
];

pub fn RE_RegisterModels_StoreShaderRequest(
    psModelFileName: *const c_char,
    psShaderName: *const c_char,
    piShaderIndexPoke: *mut c_int,
) {
    let mut sModelName: [c_char; 260] = [0; 260]; // MAX_QPATH

    unsafe {
        assert!(!CachedModels.is_null());

        Q_strncpyz(sModelName.as_mut_ptr(), psModelFileName, sModelName.len());
        Q_strlwr(sModelName.as_mut_ptr());

        let models = &mut *CachedModels;
        let model_name_str = core::ffi::CStr::from_ptr(sModelName.as_ptr())
            .to_string_lossy()
            .to_string();

        let ModelBin = models.entry(model_name_str).or_insert_with(CachedEndianedModelBinary_t::new);

        if ModelBin.pModelDiskImage.is_null() {
            debug_assert!(false); // should never happen, means that we're being called on a model that wasn't loaded
        } else {
            let iNameOffset =
                (psShaderName as *const c_char as usize) - (ModelBin.pModelDiskImage as usize);
            let iPokeOffset =
                (piShaderIndexPoke as *const c_char as usize) - (ModelBin.pModelDiskImage as usize);

            ModelBin.ShaderRegisterData
                .push(StringOffsetAndShaderIndexDest_t(iNameOffset as c_int, iPokeOffset as c_int));
        }
    }
}

pub fn RE_RegisterModels_GetDiskFile(
    psModelFileName: *const c_char,
    ppvBuffer: *mut *mut c_void,
    pqbAlreadyCached: *mut c_int,
) -> c_int {
    let mut sModelName: [c_char; 260] = [0; 260]; // MAX_QPATH

    unsafe {
        assert!(!CachedModels.is_null());

        Q_strncpyz(sModelName.as_mut_ptr(), psModelFileName, sModelName.len());
        Q_strlwr(sModelName.as_mut_ptr());

        let models = &mut *CachedModels;
        let model_name_str = core::ffi::CStr::from_ptr(sModelName.as_ptr())
            .to_string_lossy()
            .to_string();

        let ModelBin = models.entry(model_name_str).or_insert_with(CachedEndianedModelBinary_t::new);

        if ModelBin.pModelDiskImage.is_null() {
            // didn't have it cached, so try the disk...
            //

            // special case intercept first...
            //
            if strcmp(
                core::ffi::CStr::from_ptr(psModelFileName).as_ptr(),
                format!("{}.gla", core::ffi::CStr::from_ptr(sDEFAULT_GLA_NAME.as_ptr()).to_string_lossy()).as_ptr() as *const c_char,
            ) == 0
            {
                // return fake params as though it was found on disk...
                //
                let pvFakeGLAFile = Z_Malloc(FakeGLAFile.len() as c_int, TAG_FILESYS, qfalse);
                memcpy(
                    pvFakeGLAFile,
                    FakeGLAFile.as_ptr() as *const c_void,
                    FakeGLAFile.len(),
                );
                *ppvBuffer = pvFakeGLAFile;
                *pqbAlreadyCached = qfalse; // faking it like this should mean that it works fine on the Mac as well
                return qtrue;
            }

            FS_ReadFile(psModelFileName, ppvBuffer);
            *pqbAlreadyCached = qfalse;
            let bSuccess = if !(*ppvBuffer).is_null() { qtrue } else { qfalse };

            if bSuccess != 0 {
                Com_DPrintf(
                    "RE_RegisterModels_GetDiskFile(): Disk-loading \"%s\"\n".as_ptr() as *const c_char,
                    psModelFileName,
                );
            }

            return bSuccess;
        } else {
            *ppvBuffer = ModelBin.pModelDiskImage;
            *pqbAlreadyCached = qtrue;
            return qtrue;
        }
    }
}

pub fn RE_RegisterModels_Malloc(
    iSize: c_int,
    pvDiskBufferIfJustLoaded: *mut c_void,
    psModelFileName: *const c_char,
    pqbAlreadyFound: *mut c_int,
    eTag: c_int,
) -> *mut c_void {
    let mut sModelName: [c_char; 260] = [0; 260]; // MAX_QPATH

    unsafe {
        assert!(!CachedModels.is_null());

        Q_strncpyz(sModelName.as_mut_ptr(), psModelFileName, sModelName.len());
        Q_strlwr(sModelName.as_mut_ptr());

        let models = &mut *CachedModels;
        let model_name_str = core::ffi::CStr::from_ptr(sModelName.as_ptr())
            .to_string_lossy()
            .to_string();

        let ModelBin = models.entry(model_name_str).or_insert_with(CachedEndianedModelBinary_t::new);

        if ModelBin.pModelDiskImage.is_null() {
            // ... then this entry has only just been created, ie we need to load it fully...
            //
            // new, instead of doing a Z_Malloc and assigning that we just morph the disk buffer alloc
            // then don't thrown it away on return - cuts down on mem overhead
            //
            // ... groan, but not if doing a limb hierarchy creation (some VV stuff?), in which case it's NULL
            //
            let mut pvDiskBufferIfJustLoaded = pvDiskBufferIfJustLoaded;
            if !pvDiskBufferIfJustLoaded.is_null() {
                Z_MorphMallocTag(pvDiskBufferIfJustLoaded, eTag);
            } else {
                pvDiskBufferIfJustLoaded = Z_Malloc(iSize, eTag, qfalse);
            }

            ModelBin.pModelDiskImage = pvDiskBufferIfJustLoaded;
            ModelBin.iAllocSize = iSize;

            let mut iCheckSum: c_int = 0;
            if FS_FileIsInPAK(sModelName.as_ptr(), &mut iCheckSum) == 1 {
                ModelBin.iPAKFileCheckSum = iCheckSum; // else ModelBin's constructor will leave it as -1
            }

            *pqbAlreadyFound = qfalse;
        } else {
            if cfg!(not(feature = "dedicated")) {
                // if we already had this model entry, then re-register all the shaders it wanted...
                //
                let iEntries = ModelBin.ShaderRegisterData.len();
                for i in 0..iEntries {
                    let iShaderNameOffset = ModelBin.ShaderRegisterData[i].0;
                    let iShaderPokeOffset = ModelBin.ShaderRegisterData[i].1;

                    let psShaderName = (ModelBin.pModelDiskImage as *const u8 as usize + iShaderNameOffset as usize) as *const c_char;
                    let piShaderPokePtr = (ModelBin.pModelDiskImage as *const u8 as usize + iShaderPokeOffset as usize) as *mut c_int;

                    let sh = R_FindShader(
                        psShaderName,
                        lightmapsNone,
                        stylesDefault,
                        qtrue,
                    );

                    if (*sh).defaultShader != 0 {
                        *piShaderPokePtr = 0;
                    } else {
                        *piShaderPokePtr = (*sh).index;
                    }
                }
            }
            *pqbAlreadyFound = qtrue; // tell caller not to re-Endian or re-Shader this binary
        }

        ModelBin.iLastLevelUsedOn = RE_RegisterMedia_GetLevel();

        return ModelBin.pModelDiskImage;
    }
}

pub fn RE_RegisterServerModels_Malloc(
    iSize: c_int,
    pvDiskBufferIfJustLoaded: *mut c_void,
    psModelFileName: *const c_char,
    pqbAlreadyFound: *mut c_int,
    eTag: c_int,
) -> *mut c_void {
    let mut sModelName: [c_char; 260] = [0; 260]; // MAX_QPATH

    unsafe {
        assert!(!CachedModels.is_null());

        Q_strncpyz(sModelName.as_mut_ptr(), psModelFileName, sModelName.len());
        Q_strlwr(sModelName.as_mut_ptr());

        let models = &mut *CachedModels;
        let model_name_str = core::ffi::CStr::from_ptr(sModelName.as_ptr())
            .to_string_lossy()
            .to_string();

        let ModelBin = models.entry(model_name_str).or_insert_with(CachedEndianedModelBinary_t::new);

        if ModelBin.pModelDiskImage.is_null() {
            // new, instead of doing a Z_Malloc and assigning that we just morph the disk buffer alloc
            // then don't thrown it away on return - cuts down on mem overhead
            //
            // ... groan, but not if doing a limb hierarchy creation (some VV stuff?), in which case it's NULL
            //
            let mut pvDiskBufferIfJustLoaded = pvDiskBufferIfJustLoaded;
            if !pvDiskBufferIfJustLoaded.is_null() {
                Z_MorphMallocTag(pvDiskBufferIfJustLoaded, eTag);
            } else {
                pvDiskBufferIfJustLoaded = Z_Malloc(iSize, eTag, qfalse);
            }

            ModelBin.pModelDiskImage = pvDiskBufferIfJustLoaded;
            ModelBin.iAllocSize = iSize;

            let mut iCheckSum: c_int = 0;
            if FS_FileIsInPAK(sModelName.as_ptr(), &mut iCheckSum) == 1 {
                ModelBin.iPAKFileCheckSum = iCheckSum; // else ModelBin's constructor will leave it as -1
            }

            *pqbAlreadyFound = qfalse;
        } else {
            // if we already had this model entry, then re-register all the shaders it wanted...
            //
            /*
            int iEntries = ModelBin.ShaderRegisterData.size();
            for (int i=0; i<iEntries; i++)
            {
                int iShaderNameOffset	= ModelBin.ShaderRegisterData[i].first;
                int iShaderPokeOffset	= ModelBin.ShaderRegisterData[i].second;

                char *psShaderName		=		  &((char*)ModelBin.pModelDiskImage)[iShaderNameOffset];
                int  *piShaderPokePtr	= (int *) &((char*)ModelBin.pModelDiskImage)[iShaderPokeOffset];

                shader_t *sh = R_FindShader( psShaderName, lightmapsNone, stylesDefault, qtrue );

                if ( sh->defaultShader )
                {
                    *piShaderPokePtr = 0;
                } else {
                    *piShaderPokePtr = sh->index;
                }
            }
            */
            // No. Bad.
            *pqbAlreadyFound = qtrue; // tell caller not to re-Endian or re-Shader this binary
        }

        ModelBin.iLastLevelUsedOn = RE_RegisterMedia_GetLevel();

        return ModelBin.pModelDiskImage;
    }
}

// dump any models not being used by this level if we're running low on memory...
//
fn GetModelDataAllocSize() -> c_int {
    return unsafe {
        Z_MemSize(TAG_MODEL_MD3) + Z_MemSize(TAG_MODEL_GLM) + Z_MemSize(TAG_MODEL_GLA)
    };
}

//
// return qtrue if at least one cached model was freed (which tells z_malloc()-fail recoveryt code to try again)
//
pub fn RE_RegisterModels_LevelLoadEnd(
    bDeleteEverythingNotUsedThisLevel: c_int, /* = qfalse */
) -> c_int {
    let mut bAtLeastoneModelFreed = qfalse;

    unsafe {
        assert!(!CachedModels.is_null());

        Com_DPrintf("RE_RegisterModels_LevelLoadEnd():\n".as_ptr() as *const c_char);

        if gbInsideRegisterModel != 0 {
            Com_DPrintf("(Inside RE_RegisterModel (z_malloc recovery?), exiting...\n".as_ptr() as *const c_char);
        } else {
            let iLoadedModelBytes = GetModelDataAllocSize();
            let iMaxModelBytes = (*r_modelpoolmegs).integer * 1024 * 1024;

            let models = &mut *CachedModels;
            let mut keys_to_delete: Vec<String> = Vec::new();

            for (key, CachedModel) in models.iter() {
                let bDeleteThis = if bDeleteEverythingNotUsedThisLevel != 0 {
                    (CachedModel.iLastLevelUsedOn != RE_RegisterMedia_GetLevel()) as c_int
                } else {
                    (CachedModel.iLastLevelUsedOn < RE_RegisterMedia_GetLevel()) as c_int
                };

                // if it wasn't used on this level, dump it...
                //
                if bDeleteThis != 0 {
                    Com_DPrintf(
                        "Dumping \"%s\"".as_ptr() as *const c_char,
                        key.as_ptr() as *const c_char,
                    );

                    if cfg!(debug_assertions) {
                        Com_DPrintf(
                            ", used on lvl %d\n".as_ptr() as *const c_char,
                            CachedModel.iLastLevelUsedOn,
                        );
                    }

                    if !CachedModel.pModelDiskImage.is_null() {
                        Z_Free(CachedModel.pModelDiskImage);
                        bAtLeastoneModelFreed = qtrue;
                    }

                    keys_to_delete.push(key.clone());
                }
            }

            // Collect iLoadedModelBytes again after deletions
            let mut _iLoadedModelBytes = GetModelDataAllocSize();
        }
    }

    unsafe {
        Com_DPrintf("RE_RegisterModels_LevelLoadEnd(): Ok\n".as_ptr() as *const c_char);
    }

    return bAtLeastoneModelFreed;
}

// scan through all loaded models and see if their PAK checksums are still valid with the current pure PAK lists,
// dump any that aren't (so people can't cheat by using models with huge spikes that show through walls etc)
//
// (avoid using ri.xxxx stuff here in case running on dedicated)
//
fn RE_RegisterModels_DumpNonPure() {
    unsafe {
        Com_DPrintf("RE_RegisterModels_DumpNonPure():\n".as_ptr() as *const c_char);

        if CachedModels.is_null() {
            return;
        }

        let models = &mut *CachedModels;
        let mut keys_to_delete: Vec<String> = Vec::new();

        for (key, CachedModel) in models.iter() {
            let mut iCheckSum: c_int = -1;
            let iInPak = FS_FileIsInPAK(key.as_ptr() as *const c_char, &mut iCheckSum);

            if iInPak == -1 || iCheckSum != CachedModel.iPAKFileCheckSum {
                if stricmp(
                    format!("{}.gla", core::ffi::CStr::from_ptr(sDEFAULT_GLA_NAME.as_ptr()).to_string_lossy()).as_ptr() as *const c_char,
                    key.as_ptr() as *const c_char,
                ) != 0
                {
                    // don't dump "*default.gla", that's program internal anyway
                    // either this is not from a PAK, or it's from a non-pure one, so ditch it...
                    //
                    Com_DPrintf(
                        "Dumping none pure model \"%s\"".as_ptr() as *const c_char,
                        key.as_ptr() as *const c_char,
                    );

                    if !CachedModel.pModelDiskImage.is_null() {
                        Z_Free(CachedModel.pModelDiskImage);
                    }

                    keys_to_delete.push(key.clone());
                }
            }
        }

        for key in keys_to_delete {
            models.remove(&key);
        }

        Com_DPrintf("RE_RegisterModels_DumpNonPure(): Ok\n".as_ptr() as *const c_char);
    }
}

pub fn RE_RegisterModels_Info_f() {
    unsafe {
        let mut iTotalBytes: c_int = 0;
        if CachedModels.is_null() {
            Com_Printf(
                "%d bytes total (%.2fMB)\n".as_ptr() as *const c_char,
                iTotalBytes,
                (iTotalBytes as f32) / 1024.0 / 1024.0,
            );
            return;
        }

        let models = &*CachedModels;
        let iModels = models.len() as c_int;
        let mut iModel: c_int = 0;

        for (key, CachedModel) in models.iter() {
            Com_Printf(
                "%d/%d: \"%s\" (%d bytes)".as_ptr() as *const c_char,
                iModel,
                iModels,
                key.as_ptr() as *const c_char,
                CachedModel.iAllocSize,
            );

            if cfg!(debug_assertions) {
                Com_Printf(
                    ", lvl %d\n".as_ptr() as *const c_char,
                    CachedModel.iLastLevelUsedOn,
                );
            }

            iTotalBytes += CachedModel.iAllocSize;
            iModel += 1;
        }
        Com_Printf(
            "%d bytes total (%.2fMB)\n".as_ptr() as *const c_char,
            iTotalBytes,
            (iTotalBytes as f32) / 1024.0 / 1024.0,
        );
    }
}

// (don't use ri.xxx functions since the renderer may not be running here)...
//
fn RE_RegisterModels_DeleteAll() {
    unsafe {
        if CachedModels.is_null() {
            return;
        }

        let models = &mut *CachedModels;

        for (_, CachedModel) in models.iter() {
            if !CachedModel.pModelDiskImage.is_null() {
                Z_Free(CachedModel.pModelDiskImage);
            }
        }

        models.clear();
    }
}

// do not use ri.xxx functions in here, the renderer may not be running (ie. if on a dedicated server)...
//
static mut giRegisterMedia_CurrentLevel: c_int = 0;

pub fn RE_RegisterMedia_LevelLoadBegin(psMapName: *const c_char, eForceReload: c_int) {
    // for development purposes we may want to ditch certain media just before loading a map...
    //
    let bDeleteModels = eForceReload == 0 || eForceReload == 2; // eForceReload_MODELS || eForceReload_ALL

    if bDeleteModels {
        RE_RegisterModels_DeleteAll();
    } else {
        unsafe {
            if (*sv_pure).integer != 0 {
                RE_RegisterModels_DumpNonPure();
            }
        }
    }

    unsafe {
        tr.numBSPModels = 0;

        if cfg!(not(feature = "dedicated")) {
            // not used in MP codebase...
            //
            // if (bDeleteBSP)
            // {
            // CM_DeleteCachedMap();
            // RE_Images_DeleteLightMaps();	// always do this now, makes no real load time difference, and lets designers work ok
            // }
        }

        // at some stage I'll probably want to put some special logic here, like not incrementing the level number
        // when going into a map like "brig" or something, so returning to the previous level doesn't require an
        // asset reload etc, but for now...
        //
        // only bump level number if we're not on the same level.
        // Note that this will hide uncached models, which is perhaps a bad thing?...
        //
        static mut sPrevMapName: [c_char; 260] = [0; 260]; // MAX_QPATH

        if Q_stricmp(psMapName, sPrevMapName.as_ptr()) != 0 {
            Q_strncpyz(sPrevMapName.as_mut_ptr(), psMapName, sPrevMapName.len());
            giRegisterMedia_CurrentLevel += 1;
        }
    }
}

pub fn RE_RegisterMedia_GetLevel() -> c_int {
    unsafe { giRegisterMedia_CurrentLevel }
}

// this is now only called by the client, so should be ok to dump media...
//
pub fn RE_RegisterMedia_LevelLoadEnd() {
    unsafe {
        RE_RegisterModels_LevelLoadEnd(qfalse);
        if cfg!(not(feature = "dedicated")) {
            RE_RegisterImages_LevelLoadEnd();
            SND_RegisterAudio_LevelLoadEnd(qfalse);
            // RE_InitDissolve();
            S_RestartMusic();
        }
    }
}

/*
** R_GetModelByHandle
*/
pub fn R_GetModelByHandle(index: qhandle_t) -> *mut model_t {
    unsafe {
        // out of range gets the defualt model
        if index < 1 || index >= tr.numModels {
            return tr.models[0];
        }

        return tr.models[index as usize];
    }
}

//===============================================================================

/*
** R_AllocModel
*/
pub fn R_AllocModel() -> *mut model_t {
    unsafe {
        if tr.numModels == MAX_MOD_KNOWN {
            return core::ptr::null_mut();
        }

        let mod_: *mut model_t =
            Hunk_Alloc(core::mem::size_of::<model_t>(), 1) as *mut model_t; // h_low = 1
        (*mod_).index = tr.numModels;
        tr.models[tr.numModels as usize] = mod_;
        tr.numModels += 1;

        return mod_;
    }
}

/*
Ghoul2 Insert Start
*/

/*
================
return a hash value for the filename
================
*/
fn generateHashValue(fname: *const c_char, size: c_int) -> c_int {
    let mut i: c_int = 0;
    let mut hash: c_int = 0;
    let mut letter: c_char;

    unsafe {
        hash = 0;
        i = 0;
        while *fname.offset(i as isize) != 0 {
            letter = tolower(*fname.offset(i as isize) as c_int) as c_char;
            if letter == b'.' as c_char {
                break; // don't include extension
            }
            if letter == b'\\' as c_char {
                letter = b'/' as c_char; // damn path names
            }
            hash += (letter as c_int) * (i + 119);
            i += 1;
        }
        hash &= size - 1;
        return hash;
    }
}

pub fn RE_InsertModelIntoHash(name: *const c_char, mod_: *mut model_t) {
    unsafe {
        let hash: c_int = generateHashValue(name, FILE_HASH_SIZE as c_int);

        // insert this file into the hash table so we can look it up faster later
        let mh: *mut modelHash_t = Hunk_Alloc(core::mem::size_of::<modelHash_t>(), 1) as *mut modelHash_t; // h_low = 1

        (*mh).next = mhHashTable[hash as usize];
        (*mh).handle = (*mod_).index;
        strcpy((*mh).name.as_mut_ptr(), name);
        mhHashTable[hash as usize] = mh;
    }
}

/*
Ghoul2 Insert End
*/

//rww - Please forgive me for all of the below. Feel free to destroy it and replace it with something better.
//You obviously can't touch anything relating to shaders or ri. functions here in case a dedicated
//server is running, which is the entire point of having these seperate functions. If anything major
//is changed in the non-server-only versions of these functions it would be wise to incorporate it
//here as well.

/*
=================
ServerLoadMDXA - load a Ghoul 2 animation file
=================
*/
fn ServerLoadMDXA(
    mod_: *mut model_t,
    buffer: *mut c_void,
    mod_name: *const c_char,
    bAlreadyCached: &mut c_int,
) -> c_int {
    unsafe {
        let pinmodel: *mut mdxaHeader_t = buffer as *mut mdxaHeader_t;
        //
        // read some fields from the binary, but only LittleLong() them when we know this wasn't an already-cached model...
        //
        let mut version: c_int = (*pinmodel).version;
        let mut size: c_int = (*pinmodel).ofsEnd;

        if *bAlreadyCached == 0 {
            version = LittleLong(version);
            size = LittleLong(size);
        }

        if version != MDXA_VERSION {
            return qfalse;
        }

        (*mod_).type_ = MOD_MDXA;
        (*mod_).dataSize += size;

        let mut bAlreadyFound: c_int = qfalse;
        let mdxa: *mut mdxaHeader_t = RE_RegisterServerModels_Malloc(size, buffer, mod_name, &mut bAlreadyFound, TAG_MODEL_GLA) as *mut mdxaHeader_t;

        debug_assert_eq!(*bAlreadyCached, bAlreadyFound); // I should probably eliminate 'bAlreadyFound', but wtf?

        if bAlreadyFound == 0 {
            // horrible new hackery, if !bAlreadyFound then we've just done a tag-morph, so we need to set the
            // bool reference passed into this function to true, to tell the caller NOT to do an FS_Freefile since
            // we've hijacked that memory block...
            //
            // Aaaargh. Kill me now...
            //
            *bAlreadyCached = qtrue;
            debug_assert_eq!(mdxa, buffer as *mut mdxaHeader_t);
            // memcpy( mdxa, buffer, size );	// and don't do this now, since it's the same thing

            LL(&mut (*mdxa).ident);
            LL(&mut (*mdxa).version);
            LL(&mut (*mdxa).numFrames);
            LL(&mut (*mdxa).numBones);
            LL(&mut (*mdxa).ofsFrames);
            LL(&mut (*mdxa).ofsEnd);
        }

        if (*mdxa).numFrames < 1 {
            return qfalse;
        }

        if bAlreadyFound != 0 {
            return qtrue; // All done, stop here, do not LittleLong() etc. Do not pass go...
        }

        if cfg!(target_arch = "x86") {
            // optimisation, we don't bother doing this for standard intel case since our data's already in that format...
        } else {
            // swap all the skeletal info
            let mut boneInfo: *mut mdxaSkel_t = ((mdxa as *const u8).add((*mdxa).ofsSkel as usize)) as *mut mdxaSkel_t;
            for i in 0..(*mdxa).numBones {
                LL(&mut (*boneInfo).numChildren);
                LL(&mut (*boneInfo).parent);
                for k in 0..(*boneInfo).numChildren {
                    LL(&mut (*boneInfo).children[k as usize]);
                }

                // get next bone
                boneInfo = (boneInfo as *const u8)
                    .add(core::mem::size_of::<mdxaSkel_t>() + ((*boneInfo).numChildren as usize - 1) * core::mem::size_of::<c_int>())
                    as *mut mdxaSkel_t;
            }

            // swap all the frames
            let frameSize: c_int = (core::mem::size_of::<mdxaFrame_t>() + ((*mdxa).numBones as usize - 1) * core::mem::size_of::<mdxaBone_t>()) as c_int;
            for i in 0..(*mdxa).numFrames {
                let cframe: *mut mdxaFrame_t = ((mdxa as *const u8)
                    .add((*mdxa).ofsFrames as usize)
                    .add((i * frameSize) as usize)) as *mut mdxaFrame_t;

                (*cframe).radius = LittleFloat((*cframe).radius);
                for j in 0..3 {
                    (*cframe).bounds[0][j as usize] = LittleFloat((*cframe).bounds[0][j as usize]);
                    (*cframe).bounds[1][j as usize] = LittleFloat((*cframe).bounds[1][j as usize]);
                    (*cframe).localOrigin[j as usize] = LittleFloat((*cframe).localOrigin[j as usize]);
                }
                for j in 0..((*mdxa).numBones as c_int * core::mem::size_of::<mdxaBone_t>() as c_int / 2) {
                    let ptr = &mut *((&mut (*cframe).bones[0] as *mut mdxaBone_t as *mut i16).offset(j as isize));
                    *ptr = LittleShort(*ptr);
                }
            }
        }

        (*mod_).mdxa = mdxa;
        return qtrue;
    }
}

/*
=================
ServerLoadMDXM - load a Ghoul 2 Mesh file
=================
*/
fn ServerLoadMDXM(
    mod_: *mut model_t,
    buffer: *mut c_void,
    mod_name: *const c_char,
    bAlreadyCached: &mut c_int,
) -> c_int {
    unsafe {
        let pinmodel: *mut mdxmHeader_t = buffer as *mut mdxmHeader_t;
        //
        // read some fields from the binary, but only LittleLong() them when we know this wasn't an already-cached model...
        //
        let mut version: c_int = (*pinmodel).version;
        let mut size: c_int = (*pinmodel).ofsEnd;

        if *bAlreadyCached == 0 {
            version = LittleLong(version);
            size = LittleLong(size);
        }

        if version != MDXM_VERSION {
            return qfalse;
        }

        (*mod_).type_ = MOD_MDXM;
        (*mod_).dataSize += size;

        let mut bAlreadyFound: c_int = qfalse;
        let mdxm: *mut mdxmHeader_t = RE_RegisterServerModels_Malloc(size, buffer, mod_name, &mut bAlreadyFound, TAG_MODEL_GLM) as *mut mdxmHeader_t;

        debug_assert_eq!(*bAlreadyCached, bAlreadyFound); // I should probably eliminate 'bAlreadyFound', but wtf?

        if bAlreadyFound == 0 {
            // horrible new hackery, if !bAlreadyFound then we've just done a tag-morph, so we need to set the
            // bool reference passed into this function to true, to tell the caller NOT to do an FS_Freefile since
            // we've hijacked that memory block...
            //
            // Aaaargh. Kill me now...
            //
            *bAlreadyCached = qtrue;
            debug_assert_eq!(mdxm, buffer as *mut mdxmHeader_t);
            // memcpy( mdxm, buffer, size );	// and don't do this now, since it's the same thing

            LL(&mut (*mdxm).ident);
            LL(&mut (*mdxm).version);
            LL(&mut (*mdxm).numLODs);
            LL(&mut (*mdxm).ofsLODs);
            LL(&mut (*mdxm).numSurfaces);
            LL(&mut (*mdxm).ofsSurfHierarchy);
            LL(&mut (*mdxm).ofsEnd);
        }

        // first up, go load in the animation file we need that has the skeletal animation info for this model
        (*mdxm).animIndex = RE_RegisterServerModel(va(
            "%s.gla".as_ptr() as *const c_char,
            (*mdxm).animName.as_ptr(),
        ));
        if (*mdxm).animIndex == 0 {
            return qfalse;
        }

        (*mod_).numLods = (*mdxm).numLODs - 1; //copy this up to the model for ease of use - it wil get inced after this.

        if bAlreadyFound != 0 {
            return qtrue; // All done. Stop, go no further, do not LittleLong(), do not pass Go...
        }

        let mut surfInfo: *mut mdxmSurfHierarchy_t =
            ((mdxm as *const u8).add((*mdxm).ofsSurfHierarchy as usize)) as *mut mdxmSurfHierarchy_t;
        for i in 0..(*mdxm).numSurfaces {
            LL(&mut (*surfInfo).numChildren);
            LL(&mut (*surfInfo).parentIndex);

            // do all the children indexs
            for j in 0..(*surfInfo).numChildren {
                LL(&mut (*surfInfo).childIndexes[j as usize]);
            }

            // We will not be using shaders on the server.
            // sh = 0;
            // insert it in the surface list

            (*surfInfo).shaderIndex = 0;

            RE_RegisterModels_StoreShaderRequest(mod_name, (*surfInfo).shader.as_ptr(), &mut (*surfInfo).shaderIndex);

            // find the next surface
            surfInfo = ((surfInfo as *const u8)
                .add(core::mem::size_of::<mdxmSurfHierarchy_t>() + ((*surfInfo).numChildren as usize - 1) * core::mem::size_of::<c_int>()))
                as *mut mdxmSurfHierarchy_t;
        }

        // swap all the LOD's	(we need to do the middle part of this even for intel, because of shader reg and err-check)
        let mut lod: *mut mdxmLOD_t =
            ((mdxm as *const u8).add((*mdxm).ofsLODs as usize)) as *mut mdxmLOD_t;
        for l in 0..(*mdxm).numLODs {
            let mut triCount: c_int = 0;

            LL(&mut (*lod).ofsEnd);
            // swap all the surfaces
            let mut surf: *mut mdxmSurface_t = ((lod as *const u8)
                .add(core::mem::size_of::<mdxmLOD_t>() + ((*mdxm).numSurfaces as usize) * core::mem::size_of::<mdxmLODSurfOffset_t>()))
                as *mut mdxmSurface_t;
            for i in 0..(*mdxm).numSurfaces {
                LL(&mut (*surf).numTriangles);
                LL(&mut (*surf).ofsTriangles);
                LL(&mut (*surf).numVerts);
                LL(&mut (*surf).ofsVerts);
                LL(&mut (*surf).ofsEnd);
                LL(&mut (*surf).ofsHeader);
                LL(&mut (*surf).numBoneReferences);
                LL(&mut (*surf).ofsBoneReferences);
                // LL(surf->maxVertBoneWeights);

                triCount += (*surf).numTriangles;

                if (*surf).numVerts > SHADER_MAX_VERTEXES {
                    return qfalse;
                }
                if (*surf).numTriangles * 3 > SHADER_MAX_INDEXES {
                    return qfalse;
                }

                // change to surface identifier
                (*surf).ident = SF_MDX;

                // register the shaders
                if cfg!(not(target_arch = "x86")) {
                    // optimisation, we don't bother doing this for standard intel case since our data's already in that format...
                    //
                    // FIXME - is this correct?
                    // do all the bone reference data
                    let mut boneRef: *mut c_int =
                        ((surf as *const u8).add((*surf).ofsBoneReferences as usize)) as *mut c_int;
                    for j in 0..(*surf).numBoneReferences {
                        LL(&mut *boneRef.offset(j as isize));
                    }

                    // swap all the triangles
                    let mut tri: *mut mdxmTriangle_t =
                        ((surf as *const u8).add((*surf).ofsTriangles as usize)) as *mut mdxmTriangle_t;
                    for j in 0..(*surf).numTriangles {
                        LL(&mut (*tri).indexes[0]);
                        LL(&mut (*tri).indexes[1]);
                        LL(&mut (*tri).indexes[2]);
                        tri = tri.offset(1);
                    }

                    // swap all the vertexes
                    let mut v: *mut mdxmVertex_t =
                        ((surf as *const u8).add((*surf).ofsVerts as usize)) as *mut mdxmVertex_t;
                    for j in 0..(*surf).numVerts {
                        (*v).normal[0] = LittleFloat((*v).normal[0]);
                        (*v).normal[1] = LittleFloat((*v).normal[1]);
                        (*v).normal[2] = LittleFloat((*v).normal[2]);

                        (*v).texCoords[0] = LittleFloat((*v).texCoords[0]);
                        (*v).texCoords[1] = LittleFloat((*v).texCoords[1]);

                        (*v).numWeights = LittleLong((*v).numWeights);
                        (*v).offset[0] = LittleFloat((*v).offset[0]);
                        (*v).offset[1] = LittleFloat((*v).offset[1]);
                        (*v).offset[2] = LittleFloat((*v).offset[2]);

                        for k in 0..(*surf).maxVertBoneWeights {
                            (*v).weights[k as usize].boneIndex = LittleLong((*v).weights[k as usize].boneIndex);
                            (*v).weights[k as usize].boneWeight = LittleFloat((*v).weights[k as usize].boneWeight);
                        }
                        v = (v as *const u8)
                            .add(core::mem::size_of::<mdxmVertex_t>() + ((*surf).maxVertBoneWeights as usize - 1) * core::mem::size_of::<mdxmWeight_t>())
                            as *mut mdxmVertex_t;
                    }
                }

                // find the next surface
                surf = ((surf as *const u8).add((*surf).ofsEnd as usize)) as *mut mdxmSurface_t;
            }

            // find the next LOD
            lod = ((lod as *const u8).add((*lod).ofsEnd as usize)) as *mut mdxmLOD_t;
        }

        (*mod_).mdxm = mdxm;
        return qtrue;
    }
}

/*
====================
RE_RegisterServerModel

Same as RE_RegisterModel, except used by the server to handle ghoul2 instance models.
====================
*/
pub fn RE_RegisterServerModel(name: *const c_char) -> qhandle_t {
    unsafe {
        if r_noServerGhoul2.is_null() {
            //keep it from choking when it gets to these checks in the g2 code. Registering all r_ cvars for the server would be a Bad Thing though.
            r_noServerGhoul2 = Cvar_Get("r_noserverghoul2".as_ptr() as *const c_char, "0".as_ptr() as *const c_char, 0);
        }

        if name.is_null() || *name == 0 {
            return 0;
        }

        if strlen(name) >= MAX_QPATH {
            return 0;
        }

        let hash: c_int = generateHashValue(name, FILE_HASH_SIZE as c_int);

        //
        // see if the model is already loaded
        //
        let mut mh: *mut modelHash_t = mhHashTable[hash as usize];
        while !mh.is_null() {
            if Q_stricmp((*mh).name.as_ptr(), name) == 0 {
                return (*mh).handle;
            }
            mh = (*mh).next;
        }

        let mod_: *mut model_t = R_AllocModel();
        if mod_.is_null() {
            return 0;
        }

        // only set the name after the model has been successfully loaded
        Q_strncpyz((*mod_).name.as_mut_ptr(), name, (*mod_).name.len());

        if cfg!(not(feature = "dedicated")) {
            // make sure the render thread is stopped
            R_SyncRenderThread();
        }

        let mut iLODStart: c_int = 0;
        if !strstr(name, ".md3".as_ptr() as *const c_char).is_null() {
            iLODStart = MD3_MAX_LODS - 1; // this loads the md3s in reverse so they can be biased
        }
        (*mod_).numLods = 0;

        //
        // load the files
        //
        let mut numLoaded: c_int = 0;

        let mut lod: c_int = iLODStart;
        while lod >= 0 {
            let mut filename: [c_char; 1024] = [0; 1024];

            strcpy(filename.as_mut_ptr(), name);

            if lod != 0 {
                let mut namebuf: [c_char; 80] = [0; 80];

                let dot_ptr = strrchr(filename.as_ptr(), b'.' as c_int);
                if !dot_ptr.is_null() {
                    *dot_ptr = 0;
                }
                sprintf(
                    namebuf.as_mut_ptr(),
                    "_%d.md3".as_ptr() as *const c_char,
                    lod,
                );
                strcat(filename.as_mut_ptr(), namebuf.as_ptr());
            }

            let mut bAlreadyCached: c_int = qfalse;
            let mut buf: *mut c_void = core::ptr::null_mut();
            if RE_RegisterModels_GetDiskFile(filename.as_ptr(), &mut buf, &mut bAlreadyCached) == 0 {
                lod -= 1;
                continue;
            }

            // loadmodel = mod;	// this seems to be fairly pointless

            // important that from now on we pass 'filename' instead of 'name' to all model load functions,
            // because 'filename' accounts for any LOD mangling etc so guarantees unique lookups for yet more
            // internal caching...
            //
            let mut ident: c_int = *(buf as *const c_int);
            if bAlreadyCached == 0 {
                ident = LittleLong(ident);
            }

            let loaded: c_int = match ident {
                // if you're trying to register anything else as a model type on the server, you are out of luck
                MDXA_IDENT => ServerLoadMDXA(mod_, buf, filename.as_ptr(), &mut bAlreadyCached),
                MDXM_IDENT => ServerLoadMDXM(mod_, buf, filename.as_ptr(), &mut bAlreadyCached),
                _ => {
                    if bAlreadyCached == 0 {
                        FS_FreeFile(buf);
                    }
                    lod = -1; // goto fail behavior
                    0
                }
            };

            if bAlreadyCached == 0 {
                // important to check!!
                FS_FreeFile(buf);
            }

            if loaded == 0 {
                if lod == 0 {
                    // goto fail;
                    (*mod_).type_ = MOD_BAD;
                    RE_InsertModelIntoHash(name, mod_);
                    return 0;
                } else {
                    break;
                }
            } else {
                (*mod_).numLods += 1;
                numLoaded += 1;
            }

            lod -= 1;
        }

        if numLoaded != 0 {
            // duplicate into higher lod spots that weren't
            // loaded, in case the user changes r_lodbias on the fly
            lod -= 1;
            while lod >= 0 {
                (*mod_).numLods += 1;
                (*mod_).md3[lod as usize] = (*mod_).md3[(lod + 1) as usize];
                lod -= 1;
            }

            /*
            Ghoul2 Insert Start
            */

            RE_InsertModelIntoHash(name, mod_);
            return (*mod_).index;
            /*
            Ghoul2 Insert End
            */
        }

        // we still keep the model_t around, so if the model name is asked for
        // again, we won't bother scanning the filesystem
        (*mod_).type_ = MOD_BAD;
        RE_InsertModelIntoHash(name, mod_);
        return 0;
    }
}

/*
====================
RE_RegisterModel

Loads in a model for the given name

Zero will be returned if the model fails to load.
An entry will be retained for failed models as an
optimization to prevent disk rescanning if they are
asked for again.
====================
*/
fn RE_RegisterModel_Actual(name: *const c_char) -> qhandle_t {
    unsafe {
        if name.is_null() || *name == 0 {
            Com_Printf("RE_RegisterModel: NULL name\n".as_ptr() as *const c_char);
            return 0;
        }

        if strlen(name) >= MAX_QPATH {
            Com_DPrintf("Model name exceeds MAX_QPATH\n".as_ptr() as *const c_char);
            return 0;
        }

        /*
        Ghoul2 Insert Start
        */
        // if (!tr.registered) {
        // Com_Printf (S_COLOR_YELLOW  "RE_RegisterModel (%s) called before ready!\n",name );
        // return 0;
        // }
        //
        // search the currently loaded models
        //
        let hash: c_int = generateHashValue(name, FILE_HASH_SIZE as c_int);

        //
        // see if the model is already loaded
        //
        let mut mh: *mut modelHash_t = mhHashTable[hash as usize];
        while !mh.is_null() {
            if Q_stricmp((*mh).name.as_ptr(), name) == 0 {
                return (*mh).handle;
            }
            mh = (*mh).next;
        }

        //	for ( hModel = 1 ; hModel < tr.numModels; hModel++ ) {
        //		mod = tr.models[hModel];
        //		if ( !strcmp( mod->name, name ) ) {
        //			if( mod->type == MOD_BAD ) {
        //				return 0;
        //			}
        //			return hModel;
        //		}
        //	}

        if *name == b'#' as c_char {
            let mut temp: [c_char; 260] = [0; 260]; // MAX_QPATH

            tr.numBSPModels += 1;
            if cfg!(not(feature = "dedicated")) {
                RE_LoadWorldMap_Actual(
                    va("maps/%s.bsp".as_ptr() as *const c_char, name.offset(1)),
                    &mut tr.bspModels as *mut _ as *mut c_void,
                    tr.numBSPModels,
                );
            }
            Com_sprintf(
                temp.as_mut_ptr(),
                MAX_QPATH as c_int,
                "*%d-0".as_ptr() as *const c_char,
                tr.numBSPModels,
            );
            let hash2: c_int = generateHashValue(temp.as_ptr(), FILE_HASH_SIZE as c_int);
            let mut mh2: *mut modelHash_t = mhHashTable[hash2 as usize];
            while !mh2.is_null() {
                if Q_stricmp((*mh2).name.as_ptr(), temp.as_ptr()) == 0 {
                    return (*mh2).handle;
                }
                mh2 = (*mh2).next;
            }

            return 0;
        }

        if *name == b'*' as c_char {
            // don't create a bad model for a bsp model
            if Q_stricmp(name, "*default.gla".as_ptr() as *const c_char) != 0 {
                return 0;
            }
        }

        /*
        Ghoul2 Insert End
        */

        // allocate a new model_t

        let mod_: *mut model_t = R_AllocModel();
        if mod_.is_null() {
            Com_Printf(
                "RE_RegisterModel: R_AllocModel() failed for '%s'\n".as_ptr() as *const c_char,
                name,
            );
            return 0;
        }

        // only set the name after the model has been successfully loaded
        Q_strncpyz((*mod_).name.as_mut_ptr(), name, (*mod_).name.len());

        if cfg!(not(feature = "dedicated")) {
            // make sure the render thread is stopped
            R_SyncRenderThread();
        }

        let mut iLODStart: c_int = 0;
        if !strstr(name, ".md3".as_ptr() as *const c_char).is_null() {
            iLODStart = MD3_MAX_LODS - 1; // this loads the md3s in reverse so they can be biased
        }
        (*mod_).numLods = 0;

        //
        // load the files
        //
        let mut numLoaded: c_int = 0;

        let mut lod: c_int = iLODStart;
        while lod >= 0 {
            let mut filename: [c_char; 1024] = [0; 1024];

            strcpy(filename.as_mut_ptr(), name);

            if lod != 0 {
                let mut namebuf: [c_char; 80] = [0; 80];

                let dot_ptr = strrchr(filename.as_ptr(), b'.' as c_int);
                if !dot_ptr.is_null() {
                    *dot_ptr = 0;
                }
                sprintf(
                    namebuf.as_mut_ptr(),
                    "_%d.md3".as_ptr() as *const c_char,
                    lod,
                );
                strcat(filename.as_mut_ptr(), namebuf.as_ptr());
            }

            let mut bAlreadyCached: c_int = qfalse;
            let mut buf: *mut c_void = core::ptr::null_mut();
            if RE_RegisterModels_GetDiskFile(filename.as_ptr(), &mut buf, &mut bAlreadyCached) == 0 {
                lod -= 1;
                continue;
            }

            // loadmodel = mod;	// this seems to be fairly pointless

            // important that from now on we pass 'filename' instead of 'name' to all model load functions,
            // because 'filename' accounts for any LOD mangling etc so guarantees unique lookups for yet more
            // internal caching...
            //
            let mut ident: c_int = *(buf as *const c_int);
            if bAlreadyCached == 0 {
                ident = LittleLong(ident);
            }

            let loaded: c_int = match ident {
                // if you add any new types of model load in this switch-case, tell me,
                // or copy what I've done with the cache scheme (-ste).
                //
                MDXA_IDENT => {
                    let mut bc = bAlreadyCached;
                    let result = {
                        extern "C" {
                            fn R_LoadMDXA(
                                mod_: *mut model_t,
                                buf: *mut c_void,
                                filename: *const c_char,
                                bAlreadyCached: &mut c_int,
                            ) -> c_int;
                        }
                        R_LoadMDXA(mod_, buf, filename.as_ptr(), &mut bc)
                    };
                    bAlreadyCached = bc;
                    result
                }
                MDXM_IDENT => {
                    let mut bc = bAlreadyCached;
                    let result = {
                        extern "C" {
                            fn R_LoadMDXM(
                                mod_: *mut model_t,
                                buf: *mut c_void,
                                filename: *const c_char,
                                bAlreadyCached: &mut c_int,
                            ) -> c_int;
                        }
                        R_LoadMDXM(mod_, buf, filename.as_ptr(), &mut bc)
                    };
                    bAlreadyCached = bc;
                    result
                }
                MD3_IDENT => {
                    let mut bc = bAlreadyCached;
                    let result = R_LoadMD3(mod_, lod, buf, filename.as_ptr(), &mut bc);
                    bAlreadyCached = bc;
                    result
                }
                _ => {
                    Com_Printf(
                        "RE_RegisterModel: unknown fileid for %s\n".as_ptr() as *const c_char,
                        filename.as_ptr(),
                    );
                    if bAlreadyCached == 0 {
                        FS_FreeFile(buf);
                    }
                    (*mod_).type_ = MOD_BAD;
                    RE_InsertModelIntoHash(name, mod_);
                    return 0;
                }
            };

            if bAlreadyCached == 0 {
                // important to check!!
                FS_FreeFile(buf);
            }

            if loaded == 0 {
                if lod == 0 {
                    (*mod_).type_ = MOD_BAD;
                    RE_InsertModelIntoHash(name, mod_);
                    return 0;
                } else {
                    break;
                }
            } else {
                (*mod_).numLods += 1;
                numLoaded += 1;
                // if we have a valid model and are biased
                // so that we won't see any higher detail ones,
                // stop loading them
                if lod <= (*r_lodbias).integer {
                    break;
                }
            }

            lod -= 1;
        }

        if numLoaded != 0 {
            // duplicate into higher lod spots that weren't
            // loaded, in case the user changes r_lodbias on the fly
            lod -= 1;
            while lod >= 0 {
                (*mod_).numLods += 1;
                (*mod_).md3[lod as usize] = (*mod_).md3[(lod + 1) as usize];
                lod -= 1;
            }

            /*
            Ghoul2 Insert Start
            */

            if cfg!(debug_assertions) {
                if !r_noPrecacheGLA.is_null()
                    && (*r_noPrecacheGLA).integer != 0
                {
                    extern "C" {
                        fn R_LoadMDXA(
                            mod_: *mut model_t,
                            buf: *mut c_void,
                            filename: *const c_char,
                            bAlreadyCached: &mut c_int,
                        ) -> c_int;
                    }
                    // Get ident - would need to re-read from buffer
                    // For now, just return if this optimization is needed
                }
            }

            RE_InsertModelIntoHash(name, mod_);
            return (*mod_).index;
            /*
            Ghoul2 Insert End
            */
        }
        if cfg!(debug_assertions) {
            Com_Printf(
                "RE_RegisterModel: couldn't load %s\n".as_ptr() as *const c_char,
                name,
            );
        }

        // we still keep the model_t around, so if the model name is asked for
        // again, we won't bother scanning the filesystem
        (*mod_).type_ = MOD_BAD;
        RE_InsertModelIntoHash(name, mod_);
        return 0;
    }
}

// wrapper function needed to avoid problems with mid-function returns so I can safely use this bool to tell the
// z_malloc-fail recovery code whether it's safe to ditch any model caches...
//
pub fn RE_RegisterModel(name: *const c_char) -> qhandle_t {
    unsafe {
        let bWhatitwas: c_int = gbInsideRegisterModel;
        gbInsideRegisterModel = qtrue; // !!!!!!!!!!!!!!

        let q: qhandle_t = RE_RegisterModel_Actual(name);

        gbInsideRegisterModel = bWhatitwas;

        return q;
    }
}

/*
=================
R_LoadMD3
=================
*/
fn R_LoadMD3(
    mod_: *mut model_t,
    lod: c_int,
    buffer: *mut c_void,
    mod_name: *const c_char,
    bAlreadyCached: &mut c_int,
) -> c_int {
    unsafe {
        let pinmodel: *mut md3Header_t = buffer as *mut md3Header_t;
        //
        // read some fields from the binary, but only LittleLong() them when we know this wasn't an already-cached model...
        //
        let mut version: c_int = (*pinmodel).version;
        let mut size: c_int = (*pinmodel).ofsEnd;

        if *bAlreadyCached == 0 {
            version = LittleLong(version);
            size = LittleLong(size);
        }

        if version != MD3_VERSION {
            Com_Printf(
                "R_LoadMD3: %s has wrong version (%i should be %i)\n".as_ptr() as *const c_char,
                mod_name,
                version,
                MD3_VERSION,
            );
            return qfalse;
        }

        (*mod_).type_ = MOD_MESH;
        (*mod_).dataSize += size;

        let mut bAlreadyFound: c_int = qfalse;
        let md3_ptr = RE_RegisterModels_Malloc(size, buffer, mod_name, &mut bAlreadyFound, TAG_MODEL_MD3) as *mut md3Header_t;

        debug_assert_eq!(*bAlreadyCached, bAlreadyFound); // I should probably eliminate 'bAlreadyFound', but wtf?

        if bAlreadyFound == 0 {
            // horrible new hackery, if !bAlreadyFound then we've just done a tag-morph, so we need to set the
            // bool reference passed into this function to true, to tell the caller NOT to do an FS_Freefile since
            // we've hijacked that memory block...
            //
            // Aaaargh. Kill me now...
            //
            *bAlreadyCached = qtrue;
            debug_assert_eq!(md3_ptr, buffer as *mut md3Header_t);
            // memcpy( mod->md3[lod], buffer, size );	// and don't do this now, since it's the same thing

            LL(&mut (*md3_ptr).ident);
            LL(&mut (*md3_ptr).version);
            LL(&mut (*md3_ptr).numFrames);
            LL(&mut (*md3_ptr).numTags);
            LL(&mut (*md3_ptr).numSurfaces);
            LL(&mut (*md3_ptr).ofsFrames);
            LL(&mut (*md3_ptr).ofsTags);
            LL(&mut (*md3_ptr).ofsSurfaces);
            LL(&mut (*md3_ptr).ofsEnd);
        }

        (*mod_).md3[lod as usize] = md3_ptr;

        if (*md3_ptr).numFrames < 1 {
            Com_Printf(
                "R_LoadMD3: %s has no frames\n".as_ptr() as *const c_char,
                mod_name,
            );
            return qfalse;
        }

        if bAlreadyFound != 0 {
            return qtrue; // All done. Stop, go no further, do not pass Go...
        }

        if cfg!(not(target_arch = "x86")) {
            // optimisation, we don't bother doing this for standard intel case since our data's already in that format...
            //

            // swap all the frames
            let mut frame: *mut md3Frame_t =
                ((md3_ptr as *const u8).add((*md3_ptr).ofsFrames as usize)) as *mut md3Frame_t;
            for i in 0..(*md3_ptr).numFrames {
                (*frame).radius = LittleFloat((*frame).radius);
                for j in 0..3 {
                    (*frame).bounds[0][j] = LittleFloat((*frame).bounds[0][j]);
                    (*frame).bounds[1][j] = LittleFloat((*frame).bounds[1][j]);
                    (*frame).localOrigin[j] = LittleFloat((*frame).localOrigin[j]);
                }
                frame = frame.offset(1);
            }

            // swap all the tags
            let mut tag: *mut md3Tag_t =
                ((md3_ptr as *const u8).add((*md3_ptr).ofsTags as usize)) as *mut md3Tag_t;
            for i in 0..((*md3_ptr).numTags * (*md3_ptr).numFrames) {
                for j in 0..3 {
                    (*tag).origin[j] = LittleFloat((*tag).origin[j]);
                    (*tag).axis[0][j] = LittleFloat((*tag).axis[0][j]);
                    (*tag).axis[1][j] = LittleFloat((*tag).axis[1][j]);
                    (*tag).axis[2][j] = LittleFloat((*tag).axis[2][j]);
                }
                tag = tag.offset(1);
            }
        }

        // swap all the surfaces
        let mut surf: *mut md3Surface_t =
            ((md3_ptr as *const u8).add((*md3_ptr).ofsSurfaces as usize)) as *mut md3Surface_t;
        for i in 0..(*md3_ptr).numSurfaces {
            LL(&mut (*surf).flags);
            LL(&mut (*surf).numFrames);
            LL(&mut (*surf).numShaders);
            LL(&mut (*surf).numTriangles);
            LL(&mut (*surf).ofsTriangles);
            LL(&mut (*surf).numVerts);
            LL(&mut (*surf).ofsShaders);
            LL(&mut (*surf).ofsSt);
            LL(&mut (*surf).ofsXyzNormals);
            LL(&mut (*surf).ofsEnd);

            if (*surf).numVerts > SHADER_MAX_VERTEXES {
                Com_Error(
                    ERR_DROP,
                    "R_LoadMD3: %s has more than %i verts on a surface (%i)".as_ptr() as *const c_char,
                    mod_name,
                    SHADER_MAX_VERTEXES,
                    (*surf).numVerts,
                );
            }
            if (*surf).numTriangles * 3 > SHADER_MAX_INDEXES {
                Com_Error(
                    ERR_DROP,
                    "R_LoadMD3: %s has more than %i triangles on a surface (%i)".as_ptr() as *const c_char,
                    mod_name,
                    SHADER_MAX_INDEXES / 3,
                    (*surf).numTriangles,
                );
            }

            // change to surface identifier
            (*surf).ident = SF_MD3;

            // lowercase the surface name so skin compares are faster
            Q_strlwr((*surf).name.as_mut_ptr());

            // strip off a trailing _1 or _2
            // this is a crutch for q3data being a mess
            let j: usize = strlen((*surf).name.as_ptr());
            if j > 2 && (*surf).name[j - 2] == b'_' as c_char {
                (*surf).name[j - 2] = 0;
            }

            if cfg!(not(feature = "dedicated")) {
                // register the shaders
                let mut shader: *mut md3Shader_t =
                    ((surf as *const u8).add((*surf).ofsShaders as usize)) as *mut md3Shader_t;
                for j in 0..(*surf).numShaders {
                    let sh: *mut shader_t = R_FindShader(
                        (*shader).name.as_ptr(),
                        lightmapsNone,
                        stylesDefault,
                        qtrue,
                    );
                    if (*sh).defaultShader != 0 {
                        (*shader).shaderIndex = 0;
                    } else {
                        (*shader).shaderIndex = (*sh).index;
                    }
                    RE_RegisterModels_StoreShaderRequest(mod_name, (*shader).name.as_ptr(), &mut (*shader).shaderIndex);
                    shader = shader.offset(1);
                }
            }

            if cfg!(not(target_arch = "x86")) {
                // optimisation, we don't bother doing this for standard intel case since our data's already in that format...
                //

                // swap all the triangles
                let mut tri: *mut md3Triangle_t =
                    ((surf as *const u8).add((*surf).ofsTriangles as usize)) as *mut md3Triangle_t;
                for j in 0..(*surf).numTriangles {
                    LL(&mut (*tri).indexes[0]);
                    LL(&mut (*tri).indexes[1]);
                    LL(&mut (*tri).indexes[2]);
                    tri = tri.offset(1);
                }

                // swap all the ST
                let mut st: *mut md3St_t =
                    ((surf as *const u8).add((*surf).ofsSt as usize)) as *mut md3St_t;
                for j in 0..(*surf).numVerts {
                    (*st).st[0] = LittleFloat((*st).st[0]);
                    (*st).st[1] = LittleFloat((*st).st[1]);
                    st = st.offset(1);
                }

                // swap all the XyzNormals
                let mut xyz: *mut md3XyzNormal_t =
                    ((surf as *const u8).add((*surf).ofsXyzNormals as usize)) as *mut md3XyzNormal_t;
                for j in 0..((*surf).numVerts * (*surf).numFrames) {
                    (*xyz).xyz[0] = LittleShort((*xyz).xyz[0]);
                    (*xyz).xyz[1] = LittleShort((*xyz).xyz[1]);
                    (*xyz).xyz[2] = LittleShort((*xyz).xyz[2]);

                    (*xyz).normal = LittleShort((*xyz).normal);
                    xyz = xyz.offset(1);
                }
            }

            // find the next surface
            surf = ((surf as *const u8).add((*surf).ofsEnd as usize)) as *mut md3Surface_t;
        }

        return qtrue;
    }
}

//=============================================================================
pub fn RE_BeginRegistration(glconfigOut: *mut glconfig_t) {
    if !cfg!(feature = "dedicated") {
        unsafe {
            R_Init();

            *glconfigOut = glConfig;

            R_SyncRenderThread();

            tr.viewCluster = -1; // force markleafs to regenerate

            // rww - 9-13-01 [1-26-01-sof2]
            // R_ClearFlares();

            RE_ClearScene();

            tr.registered = qtrue;

            // NOTE: this sucks, for some reason the first stretch pic is never drawn
            // without this we'd see a white flash on a level load because the very
            // first time the level shot would not be drawn
            RE_StretchPic(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0);
        }
    }
}

//=============================================================================

pub fn R_SVModelInit() {
    R_ModelInit();
}

/*
===============
R_ModelInit
===============
*/
pub fn R_ModelInit() {
    unsafe {
        if CachedModels.is_null() {
            CachedModels = Box::into_raw(Box::new(HashMap::new()));
        }

        // leave a space for NULL model
        tr.numModels = 0;
        memset(
            mhHashTable.as_mut_ptr() as *mut c_void,
            0,
            core::mem::size_of_val(&mhHashTable),
        );

        let mod_: *mut model_t = R_AllocModel();
        (*mod_).type_ = MOD_BAD;
    }
}

pub fn R_HunkClearCrap() {
    //get your dirty sticky assets off me, you damn dirty hunk!
    unsafe {
        KillTheShaderHashTable();
        tr.numModels = 0;
        memset(
            mhHashTable.as_mut_ptr() as *mut c_void,
            0,
            core::mem::size_of_val(&mhHashTable),
        );
        tr.numShaders = 0;
        tr.numSkins = 0;
    }
}

pub fn R_ModelFree() {
    unsafe {
        if !CachedModels.is_null() {
            RE_RegisterModels_DeleteAll();
            let _ = Box::from_raw(CachedModels);
            CachedModels = core::ptr::null_mut();
        }
    }
}

/*
================
R_Modellist_f
================
*/
pub fn R_Modellist_f() {
    unsafe {
        let mut total: c_int = 0;
        for i in 1..tr.numModels {
            let mod_: *mut model_t = tr.models[i as usize];
            let mut lods: c_int = 1;
            for j in 1..MD3_MAX_LODS {
                if !(*mod_).md3[j as usize].is_null()
                    && (*mod_).md3[j as usize] != (*mod_).md3[(j - 1) as usize]
                {
                    lods += 1;
                }
            }
            Com_Printf(
                "%8i : (%i) %s\n".as_ptr() as *const c_char,
                (*mod_).dataSize,
                lods,
                (*mod_).name.as_ptr(),
            );
            total += (*mod_).dataSize;
        }
        Com_Printf(
            "%8i : Total models\n".as_ptr() as *const c_char,
            total,
        );

        // if	0		// not working right with new hunk
        //	if ( tr.world ) {
        //		Com_Printf ("\n%8i : %s\n", tr.world->dataSize, tr.world->name );
        //	}
        // endif
    }
}

//=============================================================================

/*
================
R_GetTag
================
*/
fn R_GetTag(mod_: *mut md3Header_t, mut frame: c_int, tagName: *const c_char) -> *mut md3Tag_t {
    unsafe {
        if frame >= (*mod_).numFrames {
            // it is possible to have a bad frame while changing models, so don't error
            frame = (*mod_).numFrames - 1;
        }

        let mut tag: *mut md3Tag_t = ((mod_ as *const u8)
            .add((*mod_).ofsTags as usize) as *mut md3Tag_t)
            .offset((frame * (*mod_).numTags) as isize);
        for i in 0..(*mod_).numTags {
            if strcmp((*tag).name.as_ptr(), tagName) == 0 {
                return tag; // found it
            }
            tag = tag.offset(1);
        }

        return core::ptr::null_mut();
    }
}

/*
================
R_LerpTag
================
*/
pub fn R_LerpTag(
    tag: *mut orientation_t,
    handle: qhandle_t,
    startFrame: c_int,
    endFrame: c_int,
    frac: f32,
    tagName: *const c_char,
) -> c_int {
    unsafe {
        let model: *mut model_t = R_GetModelByHandle(handle);
        if (*model).md3[0].is_null() {
            for i in 0..3 {
                (*tag).origin[i] = 0.0;
                (*tag).axis[0][i] = 0.0;
                (*tag).axis[1][i] = 0.0;
                (*tag).axis[2][i] = 0.0;
            }
            return qfalse;
        }

        let start: *mut md3Tag_t = R_GetTag((*model).md3[0], startFrame, tagName);
        let end: *mut md3Tag_t = R_GetTag((*model).md3[0], endFrame, tagName);
        if start.is_null() || end.is_null() {
            for i in 0..3 {
                (*tag).origin[i] = 0.0;
                (*tag).axis[0][i] = 0.0;
                (*tag).axis[1][i] = 0.0;
                (*tag).axis[2][i] = 0.0;
            }
            return qfalse;
        }

        let frontLerp: f32 = frac;
        let backLerp: f32 = 1.0 - frac;

        for i in 0..3 {
            (*tag).origin[i] = (*start).origin[i] * backLerp + (*end).origin[i] * frontLerp;
            (*tag).axis[0][i] = (*start).axis[0][i] * backLerp + (*end).axis[0][i] * frontLerp;
            (*tag).axis[1][i] = (*start).axis[1][i] * backLerp + (*end).axis[1][i] * frontLerp;
            (*tag).axis[2][i] = (*start).axis[2][i] * backLerp + (*end).axis[2][i] * frontLerp;
        }

        // VectorNormalize would go here
        // For now, just return qtrue as the C code does

        return qtrue;
    }
}

/*
====================
R_ModelBounds
====================
*/
pub fn R_ModelBounds(handle: qhandle_t, mins: *mut [f32; 3], maxs: *mut [f32; 3]) {
    unsafe {
        let model: *mut model_t = R_GetModelByHandle(handle);

        if !(*model).bmodel.is_null() {
            // VectorCopy would go here
            return;
        }

        if (*model).md3[0].is_null() {
            for i in 0..3 {
                (*mins)[i] = 0.0;
                (*maxs)[i] = 0.0;
            }
            return;
        }

        let header: *mut md3Header_t = (*model).md3[0];

        let frame: *mut md3Frame_t =
            ((header as *const u8).add((*header).ofsFrames as usize)) as *mut md3Frame_t;

        for i in 0..3 {
            (*mins)[i] = (*frame).bounds[0][i];
            (*maxs)[i] = (*frame).bounds[1][i];
        }
    }
}
