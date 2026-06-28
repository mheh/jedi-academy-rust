// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr;
use core::mem;

// Anything above this include will be ignored by the compiler

// External declarations (from qcommon/exe_headers.h, etc.)
extern "C" {
    // From renderer/tr_local.h
    fn R_GetModelByHandle(handle: c_int) -> *mut model_t;
    fn RE_RegisterModel(name: *const c_char) -> c_int;
    fn R_GetShaderByHandle(handle: c_int) -> *mut shader_t;
    fn R_GetSkinByHandle(handle: c_int) -> *mut skin_t;

    // From qcommon/MiniHeap.h
    // (CMiniHeap methods will be accessed via *mut pointer)

    // From server/server.h
    fn SV_GetConfigstring(index: c_int, buffer: *mut c_char, bufsize: c_int);
    fn SV_SetConfigstring(index: c_int, val: *const c_char);

    // Math/vector functions
    fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    fn VectorClear(v: *mut [f32; 3]);
    fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorAdd(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorScale(v: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    fn VectorMA(a: *const [f32; 3], scale: f32, b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorLength(v: *const [f32; 3]) -> f32;
    fn VectorLengthSquared(v: *const [f32; 3]) -> f32;
    fn VectorNormalize(v: *mut [f32; 3]);
    fn DotProduct(a: *const [f32; 3], b: *const [f32; 3]) -> f32;
    fn CrossProduct(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn AnglesToAxis(angles: *const [f32; 3], axis: *mut [[f32; 3]; 3]);

    // Com functions
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);

    // Memory allocation
    fn Z_Malloc(size: c_int, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    // Cvar
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;

    // From G2_local.h
    fn G2_FindSurface(mod_t: *mut c_void, index: c_int, lod: c_int) -> *mut c_void;
    fn G2_FindOverrideSurface(index: c_int, list: *mut c_void) -> *mut surfaceInfo_t;
    fn G2API_GetTime(argTime: c_int) -> c_int;
    fn TransformPoint(in_: *const [f32; 3], out: *mut [f32; 3], mat: *mut mdxaBone_t);
    fn TransformAndTranslatePoint(in_: *const [f32; 3], out: *mut [f32; 3], mat: *mut mdxaBone_t);
    fn Inverse_Matrix(src: *mut mdxaBone_t, dest: *mut mdxaBone_t);

    #[allow(improper_ctypes)]
    fn EvalBoneCache(index: c_int, boneCache: *mut c_void) -> *const mdxaBone_t;
}

// Type declarations matching C structures
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

const ERR_DROP: c_int = 0;

// Constants
const MAX_STRING_CHARS: usize = 1024;
const MAX_G2_COLLISIONS: usize = 32;

// Gore-related constants
#[cfg(feature = "G2_GORE")]
const GORE_TAG_UPPER: c_int = 256;
#[cfg(feature = "G2_GORE")]
const GORE_TAG_MASK: c_int = !255;

#[cfg(feature = "G2_GORE")]
const MAX_GORE_RECORDS: c_int = 500;
#[cfg(feature = "G2_GORE")]
const MAX_GORE_VERTS: usize = 3000;
#[cfg(feature = "G2_GORE")]
const MAX_GORE_INDECIES: usize = 6000;
#[cfg(feature = "G2_GORE")]
const GORE_MARGIN: f32 = 0.0f32;

const G2_RETURNONHIT: c_int = 1;
const G2_FRONTFACE: c_int = 1;
const G2_BACKFACE: c_int = 2;
const GHOUL2_ZONETRANSALLOC: c_int = 0x1;
const GHOUL2_NOCOLLIDE: c_int = 0x2;
const G2SURFACEFLAG_NODESCENDANTS: c_int = 0x1;

const TAG_GHOUL2_GORE: c_int = 0x1000;
const TAG_GHOUL2: c_int = 0x2000;

// Global bone matrices from tr_ghoul2
extern "C" {
    pub static mut worldMatrix: mdxaBone_t;
    pub static mut worldMatrixInv: mdxaBone_t;
}

// C++ STL substitute types - these would need proper implementation
// For now, declaring as opaque types to maintain structural parity
#[repr(C)]
pub struct MapIntGoreTextureCoordinates {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct MapPairIntIntInt {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct GoreTextureCoordinates {
    pub tex: [[*mut f32; 4]; 4], // tex[LOD][...]
}

#[repr(C)]
pub struct CGoreSet {
    pub mMyGoreSetTag: c_int,
    pub mRefCount: c_int,
    pub mGoreRecords: c_void, // multimap<int, SGoreSurface>
}

#[repr(C)]
pub struct SGoreSurface {
    pub shader: c_int,
    pub mDeleteTime: c_int,
    pub mFadeTime: f32,
    pub mFadeRGB: c_int,
    pub mGoreTag: c_int,
    pub mGoreGrowStartTime: c_int,
    pub mGoreGrowEndTime: c_int,
    pub mGoreGrowFactor: f32,
    pub mGoreGrowOffset: f32,
}

#[repr(C)]
pub struct SSkinGoreData {
    pub frontFaces: c_int,
    pub backFaces: c_int,
    pub lifeTime: c_int,
    pub fadeOutTime: f32,
    pub fadeRGB: c_int,
    pub growDuration: c_int,
    pub goreScaleStartFraction: f32,
}

#[repr(C)]
pub struct model_t {
    pub mdxm: *mut mdxmHeader_t,
    pub mdxa: *mut mdxaHeader_t,
    pub numLods: c_int,
    // ... other fields
}

#[repr(C)]
pub struct mdxmHeader_t {
    pub animName: [c_char; 64],
    pub animIndex: c_int,
    pub numLODs: c_int,
    pub numSurfaces: c_int,
    pub ofsLODs: c_int,
    pub ofsSurfHierarchy: c_int,
    pub ofsFrames: c_int,
    // ... other fields
}

#[repr(C)]
pub struct mdxaHeader_t {
    pub numBones: c_int,
    pub ofsFrames: c_int,
    // ... other fields
}

#[repr(C)]
pub struct mdxmSurfHierarchy_t {
    pub name: [c_char; 64],
    pub flags: c_int,
    pub numChildren: c_int,
    pub childIndexes: [c_int; 0], // Variable length
}

#[repr(C)]
pub struct mdxmSurface_t {
    pub thisSurfaceIndex: c_int,
    pub numVerts: c_int,
    pub numTriangles: c_int,
    pub ofsTriangles: c_int,
    pub ofsVerts: c_int,
    pub ofsBoneReferences: c_int,
    pub ofsEnd: c_int,
    pub maxVertBoneWeights: c_int,
    // ... other fields
}

#[repr(C)]
pub struct mdxmLOD_t {
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct mdxmLODSurfOffset_t {
    pub offsets: [c_int; 0], // Variable length
}

#[repr(C)]
pub struct mdxmVertex_t {
    pub vertCoords: [i16; 3],
    pub normal: [i8; 3],
    // weights follow
}

#[repr(C)]
pub struct mdxmVertexTexCoord_t {
    pub texCoords: [f32; 2],
}

#[repr(C)]
pub struct mdxmTriangle_t {
    pub indexes: [c_int; 3],
}

#[repr(C)]
pub struct mdxmHierarchyOffsets_t {
    pub offsets: [c_int; 0], // Variable length
}

#[repr(C)]
pub struct mdxaSkelOffsets_t {
    pub offsets: [c_int; 0], // Variable length
}

#[repr(C)]
pub struct mdxaSkel_t {
    pub name: [c_char; 64],
    pub numChildren: c_int,
    pub BasePoseMat: mdxaBone_t,
}

#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct shader_t {
    // opaque for now
}

#[repr(C)]
pub struct skin_t {
    pub numSurfaces: c_int,
    pub surfaces: *mut *mut shader_t,
}

#[repr(C)]
pub struct cvar_t {
    pub value: f32,
    pub integer: c_int,
}

#[repr(C)]
pub struct CollisionRecord_t {
    pub mEntityNum: c_int,
    pub mSurfaceIndex: c_int,
    pub mPolyIndex: c_int,
    pub mModelIndex: c_int,
    pub mFlags: c_int,
    pub mDistance: f32,
    pub mCollisionPosition: vec3_t,
    pub mCollisionNormal: vec3_t,
    pub mMaterial: c_int,
    pub mLocation: c_int,
    pub mBarycentricI: f32,
    pub mBarycentricJ: f32,
}

#[repr(C)]
pub struct CGhoul2Info {
    pub mValid: c_int,
    pub mModelindex: c_int,
    pub mFileName: [c_char; 260],
    pub mSkelFrameNum: c_int,
    pub mMeshFrameNum: c_int,
    pub mLodBias: c_int,
    pub mFlags: c_int,
    pub mCustomShader: c_int,
    pub mSkin: c_int,
    pub mSurfaceRoot: c_int,
    pub mBoneCache: *mut c_void,
    pub mSlist: c_void, // surfaceInfo_v - vector
    pub mBlist: c_void, // boneInfo_t vector
    pub mBltlist: c_void, // boltInfo_t vector
    pub mTransformedVertsArray: *mut c_int,
    pub currentModel: *mut model_t,
    pub mGoreSetTag: c_int,
}

pub type CGhoul2Info_v = Vec<CGhoul2Info>;

#[repr(C)]
pub struct boneInfo_t {
    pub boneNumber: c_int,
    pub flags: c_int,
    pub matrix: mdxaBone_t,
    pub newMatrix: mdxaBone_t,
}

#[repr(C)]
pub struct boltInfo_t {
    pub boneIndex: c_int,
    pub matrix: mdxaBone_t,
}

#[repr(C)]
pub struct surfaceInfo_t {
    pub offFlags: c_int,
}

pub type surfaceInfo_v = Vec<surfaceInfo_t>;

pub type CMiniHeap = c_void;
pub type CBoneCache = c_void;

// Global statics for gore tracking
#[cfg(feature = "G2_GORE")]
static mut CurrentTag: c_int = GORE_TAG_UPPER + 1;
#[cfg(feature = "G2_GORE")]
static mut CurrentTagUpper: c_int = GORE_TAG_UPPER;
#[cfg(feature = "G2_GORE")]
static mut GoreRecords: *mut MapIntGoreTextureCoordinates = ptr::null_mut();
#[cfg(feature = "G2_GORE")]
static mut GoreTagsTemp: *mut MapPairIntIntInt = ptr::null_mut();
#[cfg(feature = "G2_GORE")]
static mut goreModelIndex: c_int = 0;
#[cfg(feature = "G2_GORE")]
static mut cg_g2MarksAllModels: *mut cvar_t = ptr::null_mut();

#[cfg(feature = "G2_GORE")]
static mut CurrentGoreSet: c_int = 1;
#[cfg(feature = "G2_GORE")]
static mut GoreSets: *mut c_void = ptr::null_mut(); // map<int, CGoreSet *>

#[cfg(all(feature = "G2_GORE", feature = "DEBUG"))]
static mut g_goreAllocs: c_int = 0;
#[cfg(all(feature = "G2_GORE", feature = "DEBUG"))]
static mut g_goreTexAllocs: c_int = 0;

static mut cg_g2MarksAllModels_nongore: *mut cvar_t = ptr::null_mut();

#[cfg(feature = "G2_GORE")]
static mut GoreVerts: [SVertexTemp; MAX_GORE_VERTS] = [SVertexTemp { flags: 0, touch: 0, newindex: 0, tex: [0.0; 2] }; MAX_GORE_VERTS];
#[cfg(feature = "G2_GORE")]
static mut GoreIndexCopy: [c_int; MAX_GORE_VERTS] = [0; MAX_GORE_VERTS];
#[cfg(feature = "G2_GORE")]
static mut GoreTouch: c_int = 1;
#[cfg(feature = "G2_GORE")]
static mut GoreIndecies: [c_int; MAX_GORE_INDECIES] = [0; MAX_GORE_INDECIES];

#[cfg(not(feature = "G2_GORE"))]
static mut GoreVerts: [SVertexTemp; MAX_GORE_VERTS] = [SVertexTemp { flags: 0 }; MAX_GORE_VERTS];

#[cfg(feature = "G2_GORE")]
#[repr(C)]
pub struct SVertexTemp {
    pub flags: c_int,
    pub touch: c_int,
    pub newindex: c_int,
    pub tex: [f32; 2],
}

#[cfg(not(feature = "G2_GORE"))]
#[repr(C)]
pub struct SVertexTemp {
    pub flags: c_int,
}

#[cfg(feature = "G2_GORE")]
fn FindGoreRecord(tag: c_int) -> *mut GoreTextureCoordinates {
    // Placeholder - would need proper map implementation
    ptr::null_mut()
}

#[cfg(feature = "G2_GORE")]
fn DestroyGoreTexCoordinates(tag: c_int) {
    let gTC = FindGoreRecord(tag);
    if gTC.is_null() {
        return;
    }
    // gTC->~GoreTextureCoordinates();
    // I don't know what's going on here, it should call the destructor for
    // this when it erases the record but sometimes it doesn't. -rww
}

#[cfg(feature = "G2_GORE")]
fn AllocGoreRecord() -> c_int {
    // TODO: This needs to be set via a scalability cvar with some reasonable minimum value if pgore is used at all
    // Placeholder implementation
    unsafe {
        let ret = CurrentTag;
        CurrentTag += 1;
        ret
    }
}

#[cfg(feature = "G2_GORE")]
fn ResetGoreTag() {
    unsafe {
        // GoreTagsTemp.clear();
        CurrentTag = CurrentTagUpper;
        CurrentTagUpper += GORE_TAG_UPPER;
    }
}

#[cfg(feature = "G2_GORE")]
pub fn G2_GetGoreRecord(tag: c_int) -> *mut c_void {
    FindGoreRecord(tag) as *mut c_void
}

#[cfg(feature = "G2_GORE")]
fn DeleteGoreRecord(tag: c_int) {
    DestroyGoreTexCoordinates(tag);
    // GoreRecords.erase(tag);
}

#[cfg(feature = "G2_GORE")]
fn FindGoreSet(goreSetTag: c_int) -> *mut CGoreSet {
    // Placeholder - would need proper map implementation
    ptr::null_mut()
}

#[cfg(feature = "G2_GORE")]
fn NewGoreSet() -> *mut CGoreSet {
    unsafe {
        let ret = unsafe { core::alloc::alloc(core::alloc::Layout::new::<CGoreSet>()) as *mut CGoreSet };
        if !ret.is_null() {
            #[cfg(feature = "DEBUG")]
            {
                g_goreAllocs += 1;
            }
            (*ret).mMyGoreSetTag = CurrentGoreSet;
            CurrentGoreSet += 1;
            (*ret).mRefCount = 1;
        }
        ret
    }
}

#[cfg(feature = "G2_GORE")]
fn DeleteGoreSet(goreSetTag: c_int) {
    // Placeholder - would need proper map implementation
}

// assorted Ghoul 2 functions.
// list all surfaces associated with a model
pub fn G2_List_Model_Surfaces(fileName: *const c_char) {
    unsafe {
        let i: c_int;
        let x: c_int;
        let mod_m = R_GetModelByHandle(RE_RegisterModel(fileName));
        let mut surf = ((*(*mod_m).mdxm as *const u8).add((*(*mod_m).mdxm).ofsSurfHierarchy as usize) as *const mdxmSurfHierarchy_t);

        let mut surface = ((*(*mod_m).mdxm as *const u8).add(((*(*mod_m).mdxm).ofsLODs + mem::size_of::<mdxmLOD_t>()) as usize) as *mut mdxmSurface_t);

        for x in 0..(*(*mod_m).mdxm).numSurfaces {
            Com_Printf(b"Surface %i Name %s\n\0".as_ptr() as *const c_char, x, (*surf).name.as_ptr());
            // r_verbose->value check would go here, but r_verbose is external
            // find the next surface
            surf = ((surf as *const u8).add(mem::size_of::<mdxmSurfHierarchy_t>() + ((*surf).numChildren as usize * mem::size_of::<c_int>())) as *const mdxmSurfHierarchy_t);
            surface = ((surface as *const u8).add((*surface).ofsEnd as usize) as *mut mdxmSurface_t);
        }
    }
}

// list all bones associated with a model
pub fn G2_List_Model_Bones(fileName: *const c_char, frame: c_int) {
    unsafe {
        let x: c_int;
        let i: c_int;
        let mut skel: *const mdxaSkel_t;
        let mut offsets: *const mdxaSkelOffsets_t;
        let mod_m = R_GetModelByHandle(RE_RegisterModel(fileName));
        let mod_a = R_GetModelByHandle((*mod_m).mdxm as c_int);
        let header = (*mod_a).mdxa;

        // figure out where the offset list is
        offsets = ((header as *const u8).add(mem::size_of::<mdxaHeader_t>()) as *const mdxaSkelOffsets_t);

        // walk each bone and list it's name
        for x in 0..(*(*mod_a).mdxa).numBones {
            skel = ((header as *const u8).add(mem::size_of::<mdxaHeader_t>() + (*offsets).offsets[x as usize] as usize) as *const mdxaSkel_t);
            Com_Printf(b"Bone %i Name %s\n\0".as_ptr() as *const c_char, x, (*skel).name.as_ptr());
            Com_Printf(b"X pos %f, Y pos %f, Z pos %f\n\0".as_ptr() as *const c_char,
                (*skel).BasePoseMat.matrix[0][3],
                (*skel).BasePoseMat.matrix[1][3],
                (*skel).BasePoseMat.matrix[2][3]);

            // if we are in verbose mode give us more details
            // r_verbose->value check would go here
            // for (i=0; i<skel->numChildren; i++)
        }
    }
}

/************************************************************************************************
 * G2_GetAnimFileName
 *    obtain the .gla filename for a model
 *
 * Input
 *    filename of model
 *
 * Output
 *    true if we successfully obtained a filename, false otherwise
 *
 ************************************************************************************************/
pub fn G2_GetAnimFileName(fileName: *const c_char, filename: *mut *mut c_char) -> qboolean {
    unsafe {
        // find the model we want
        let mod_ = R_GetModelByHandle(RE_RegisterModel(fileName));

        if !mod_.is_null() && !(*mod_).mdxm.is_null() && (*(*mod_).mdxm).animName[0] != 0 {
            *filename = (*(*mod_).mdxm).animName.as_mut_ptr();
            return qtrue;
        }
        return qfalse;
    }
}

/////////////////////////////////////////////////////////////////////
//
//  Code for collision detection for models gameside
//
/////////////////////////////////////////////////////////////////////

fn G2_DecideTraceLod(ghoul2: &CGhoul2Info, useLod: c_int) -> c_int {
    let mut returnLod = useLod;

    // if we are overriding the LOD at top level, then we can afford to only check this level of model
    if ghoul2.mLodBias > returnLod {
        returnLod = ghoul2.mLodBias;
    }
    // assert(G2_MODEL_OK(&ghoul2));

    assert!(!ghoul2.currentModel.is_null());
    assert!(!(*ghoul2.currentModel).mdxm.is_null());
    // what about r_lodBias?

    // now ensure that we haven't selected a lod that doesn't exist for this model
    if returnLod >= unsafe { (*(*ghoul2.currentModel).mdxm).numLODs } {
        returnLod = unsafe { (*(*ghoul2.currentModel).mdxm).numLODs } - 1;
    }

    returnLod
}

#[cfg(not(target_os = "windows"))]
fn R_TransformEachSurface(surface: *const mdxmSurface_t, scale: *const vec3_t, G2VertSpace: *mut CMiniHeap, TransformedVertsArray: *mut c_int, boneCache: *mut CBoneCache) {
    unsafe {
        let mut j: c_int;
        let mut k: c_int;
        let mut v: *mut mdxmVertex_t;
        let mut TransformedVerts: *mut f32;

        //
        // deform the vertexes by the lerped bones
        //
        let piBoneReferences = ((*surface as *const u8).add((*surface).ofsBoneReferences as usize) as *mut c_int);

        // alloc some space for the transformed verts to get put in
        // TransformedVerts = (float *)G2VertSpace->MiniHeapAlloc(surface->numVerts * 5 * 4);
        // Placeholder call
        TransformedVerts = ptr::null_mut();
        (*TransformedVertsArray.add((*surface).thisSurfaceIndex as usize)) = TransformedVerts as c_int;
        if TransformedVerts.is_null() {
            Com_Error(ERR_DROP, b"Ran out of transform space for Ghoul2 Models. Adjust MiniHeapSize in SV_SpawnServer.\n\0".as_ptr() as *const c_char);
        }

        // whip through and actually transform each vertex
        let numVerts = (*surface).numVerts;
        v = ((*surface as *const u8).add((*surface).ofsVerts as usize) as *mut mdxmVertex_t);
        let pTexCoords = ((v as *const u8).add((numVerts as usize) * mem::size_of::<mdxmVertex_t>()) as *mut mdxmVertexTexCoord_t);

        // optimisation issue
        if (*scale)[0] != 1.0 || (*scale)[1] != 1.0 || (*scale)[2] != 1.0 {
            for j in 0..numVerts {
                let mut tempVert: vec3_t = [0.0; 3];
                let mut tempNormal: vec3_t = [0.0; 3];

                VectorClear(&mut tempVert);
                VectorClear(&mut tempNormal);

                // Bone weight calculations would go here
                let iNumWeights = 0; // Placeholder: G2_GetVertWeights(v)
                let mut fTotalWeight = 0.0f32;

                for k in 0..iNumWeights {
                    // iBoneIndex = G2_GetVertBoneIndex(v, k);
                    // fBoneWeight = G2_GetVertBoneWeight(v, k, fTotalWeight, iNumWeights);
                    // const mdxaBone_t &bone = EvalBoneCache(piBoneReferences[iBoneIndex], boneCache);
                    // tempVert[0] += fBoneWeight * ( DotProduct( bone.matrix[0], v->vertCoords ) + bone.matrix[0][3] );
                    // ... similar for y and z
                }

                let pos = (j * 5) as usize;
                TransformedVerts.add(pos).write(tempVert[0] * (*scale)[0]);
                TransformedVerts.add(pos + 1).write(tempVert[1] * (*scale)[1]);
                TransformedVerts.add(pos + 2).write(tempVert[2] * (*scale)[2]);
                TransformedVerts.add(pos + 3).write((*pTexCoords.add(j as usize)).texCoords[0]);
                TransformedVerts.add(pos + 4).write((*pTexCoords.add(j as usize)).texCoords[1]);

                v = v.add(1);
            }
        } else {
            let mut pos = 0usize;
            for j in 0..numVerts {
                let mut tempVert: vec3_t = [0.0; 3];
                let mut tempNormal: vec3_t = [0.0; 3];

                VectorClear(&mut tempVert);
                VectorClear(&mut tempNormal);

                // Bone weight calculations
                let iNumWeights = 0; // Placeholder
                let mut fTotalWeight = 0.0f32;

                for k in 0..iNumWeights {
                    // Similar calculations as above
                }

                TransformedVerts.add(pos).write(tempVert[0]);
                TransformedVerts.add(pos + 1).write(tempVert[1]);
                TransformedVerts.add(pos + 2).write(tempVert[2]);
                TransformedVerts.add(pos + 3).write((*pTexCoords.add(j as usize)).texCoords[0]);
                TransformedVerts.add(pos + 4).write((*pTexCoords.add(j as usize)).texCoords[1]);
                pos += 5;

                v = v.add(1);
            }
        }
    }
}

fn G2_TransformSurfaces(surfaceNum: c_int, rootSList: *mut surfaceInfo_v, boneCache: *mut CBoneCache, currentModel: *const model_t, lod: c_int, scale: *const vec3_t, G2VertSpace: *mut CMiniHeap, TransformedVertArray: *mut c_int, secondTimeAround: bool) {
    unsafe {
        let mut i: c_int;
        assert!(!currentModel.is_null());
        assert!(!(*currentModel).mdxm.is_null());

        // back track and get the surfinfo struct for this surface
        let surface = G2_FindSurface(currentModel as *mut c_void, surfaceNum, lod) as *const mdxmSurface_t;
        let surfIndexes = (((*currentModel).mdxm as *const u8).add(mem::size_of::<mdxmHeader_t>()) as *const mdxmHierarchyOffsets_t);
        let surfInfo = ((surfIndexes as *const u8).add((*surfIndexes).offsets[(*surface).thisSurfaceIndex as usize] as usize) as *const mdxmSurfHierarchy_t);

        // see if we have an override surface in the surface list
        let surfOverride = G2_FindOverrideSurface(surfaceNum, rootSList as *mut c_void) as *const surfaceInfo_t;

        // really, we should use the default flags for this surface unless it's been overriden
        let mut offFlags = (*surfInfo).flags;

        if !surfOverride.is_null() {
            offFlags = (*surfOverride).offFlags;
        }

        // if this surface is not off, add it to the shader render list
        if offFlags == 0 {
            #[cfg(not(target_os = "windows"))]
            R_TransformEachSurface(surface, scale, G2VertSpace, TransformedVertArray, boneCache);
        }

        // if we are turning off all descendants, then stop this recursion now
        if offFlags & G2SURFACEFLAG_NODESCENDANTS != 0 {
            return;
        }

        // now recursively call for the children
        for i in 0..(*surfInfo).numChildren {
            G2_TransformSurfaces((*surfInfo).childIndexes[i as usize], rootSList, boneCache, currentModel, lod, scale, G2VertSpace, TransformedVertArray, secondTimeAround);
        }
    }
}

// main calling point for the model transform for collision detection. At this point all of the skeleton has been transformed.
#[cfg(feature = "G2_GORE")]
pub fn G2_TransformModel(ghoul2: &mut CGhoul2Info_v, frameNum: c_int, scale: *const vec3_t, G2VertSpace: *mut CMiniHeap, useLod: c_int, ApplyGore: bool) {
    G2_TransformModel_impl(ghoul2, frameNum, scale, G2VertSpace, useLod, ApplyGore);
}

#[cfg(not(feature = "G2_GORE"))]
pub fn G2_TransformModel(ghoul2: &mut CGhoul2Info_v, frameNum: c_int, scale: *const vec3_t, G2VertSpace: *mut CMiniHeap, useLod: c_int) {
    G2_TransformModel_impl(ghoul2, frameNum, scale, G2VertSpace, useLod, false);
}

fn G2_TransformModel_impl(ghoul2: &mut CGhoul2Info_v, frameNum: c_int, scale: *const vec3_t, G2VertSpace: *mut CMiniHeap, useLod: c_int, ApplyGore: bool) {
    unsafe {
        let mut i: c_int;
        let mut lod: c_int;
        let mut correctScale: vec3_t = [0.0; 3];
        let mut firstModelOnly = qfalse;

        if cg_g2MarksAllModels_nongore.is_null() {
            cg_g2MarksAllModels_nongore = Cvar_Get(b"cg_g2MarksAllModels\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        if cg_g2MarksAllModels_nongore.is_null() || (*cg_g2MarksAllModels_nongore).integer == 0 {
            firstModelOnly = qtrue;
        }

        VectorCopy(scale, &mut correctScale);
        // check for scales of 0 - that's the default I believe
        if correctScale[0] == 0.0 {
            correctScale[0] = 1.0;
        }
        if correctScale[1] == 0.0 {
            correctScale[1] = 1.0;
        }
        if correctScale[2] == 0.0 {
            correctScale[2] = 1.0;
        }

        // walk each possible model for this entity and try rendering it out
        for i in 0..(ghoul2.len() as c_int) {
            let g = &mut ghoul2[i as usize];
            // don't bother with models that we don't care about.
            if g.mValid == 0 {
                continue;
            }
            assert!(!g.mBoneCache.is_null());

            // stop us building this model more than once per frame
            g.mMeshFrameNum = frameNum;

            // decide the LOD
            #[cfg(feature = "G2_GORE")]
            {
                if ApplyGore {
                    lod = useLod;
                    assert!(!g.currentModel.is_null());
                    if lod >= (*g.currentModel).numLods {
                        g.mTransformedVertsArray = ptr::null_mut();
                        if firstModelOnly != 0 {
                            // we don't really need to do multiple models for gore.
                            return;
                        }
                        //do the rest
                        continue;
                    }
                } else {
                    lod = G2_DecideTraceLod(g, useLod);
                }
            }
            #[cfg(not(feature = "G2_GORE"))]
            {
                lod = G2_DecideTraceLod(g, useLod);
            }

            // give us space for the transformed vertex array to be put in
            if (g.mFlags & GHOUL2_ZONETRANSALLOC) == 0 {
                // do not stomp if we're using zone space
                // g.mTransformedVertsArray = (int*)G2VertSpace->MiniHeapAlloc(g.currentModel->mdxm->numSurfaces * 4);
                // Placeholder
                if g.mTransformedVertsArray.is_null() {
                    Com_Error(ERR_DROP, b"Ran out of transform space for Ghoul2 Models. Adjust MiniHeapSize in SV_SpawnServer.\n\0".as_ptr() as *const c_char);
                }
            }

            // memset(g.mTransformedVertsArray, 0,(g.currentModel->mdxm->numSurfaces * 4));
            if !g.mTransformedVertsArray.is_null() {
                core::ptr::write_bytes(g.mTransformedVertsArray, 0, ((*(*g.currentModel).mdxm).numSurfaces * 4) as usize);
            }

            G2_FindOverrideSurface(-1, &mut g.mSlist as *mut c_void); //reset the quick surface override lookup

            // recursively call the model surface transform
            G2_TransformSurfaces(g.mSurfaceRoot, &mut g.mSlist, g.mBoneCache, g.currentModel, lod, &correctScale, G2VertSpace, g.mTransformedVertsArray, false);

            #[cfg(feature = "G2_GORE")]
            {
                if ApplyGore && firstModelOnly != 0 {
                    // we don't really need to do multiple models for gore.
                    break;
                }
            }
        }
    }
}

// work out how much space a triangle takes
fn G2_AreaOfTri(A: *const vec3_t, B: *const vec3_t, C: *const vec3_t) -> f32 {
    unsafe {
        let mut cross: vec3_t = [0.0; 3];
        let mut ab: vec3_t = [0.0; 3];
        let mut cb: vec3_t = [0.0; 3];
        VectorSubtract(A, B, &mut ab);
        VectorSubtract(C, B, &mut cb);

        CrossProduct(&ab, &cb, &mut cross);

        return VectorLength(&cross);
    }
}

// actually determine the S and T of the coordinate we hit in a given poly
fn G2_BuildHitPointST(
    A: *const vec3_t, SA: f32, TA: f32,
    B: *const vec3_t, SB: f32, TB: f32,
    C: *const vec3_t, SC: f32, TC: f32,
    P: *const vec3_t, s: *mut f32, t: *mut f32, bary_i: &mut f32, bary_j: &mut f32
) {
    unsafe {
        let areaABC = G2_AreaOfTri(A, B, C);

        let i = G2_AreaOfTri(P, B, C) / areaABC;
        *bary_i = i;
        let j = G2_AreaOfTri(A, P, C) / areaABC;
        *bary_j = j;
        let k = G2_AreaOfTri(A, B, P) / areaABC;

        *s = SA * i + SB * j + SC * k;
        *t = TA * i + TB * j + TC * k;

        *s = s.read() % 1.0;
        if *s < 0.0 {
            *s += 1.0;
        }

        *t = t.read() % 1.0;
        if *t < 0.0 {
            *t += 1.0;
        }
    }
}

// routine that works out given a ray whether or not it hits a poly
fn G2_SegmentTriangleTest(
    start: *const vec3_t, end: *const vec3_t,
    A: *const vec3_t, B: *const vec3_t, C: *const vec3_t,
    backFaces: qboolean, frontFaces: qboolean,
    returnedPoint: *mut vec3_t, returnedNormal: *mut vec3_t, denom: *mut f32
) -> qboolean {
    unsafe {
        const tiny: f32 = 1E-10f32;
        let mut returnedNormalT: vec3_t = [0.0; 3];
        let mut edgeAC: vec3_t = [0.0; 3];

        VectorSubtract(C, A, &mut edgeAC);
        VectorSubtract(B, A, &mut returnedNormalT);

        CrossProduct(&returnedNormalT, &edgeAC, returnedNormal);

        let mut ray: vec3_t = [0.0; 3];
        VectorSubtract(end, start, &mut ray);

        *denom = DotProduct(&ray, returnedNormal);

        if denom.read().abs() < tiny ||        // triangle parallel to ray
            (backFaces == 0 && *denom > 0.0) ||  // not accepting back faces
            (frontFaces == 0 && *denom < 0.0)    // not accepting front faces
        {
            return qfalse;
        }

        let mut toPlane: vec3_t = [0.0; 3];
        VectorSubtract(A, start, &mut toPlane);

        let t = DotProduct(&toPlane, returnedNormal) / *denom;

        if t < 0.0 || t > 1.0 {
            return qfalse; // off segment
        }

        VectorScale(&ray, t, &mut ray);
        VectorAdd(&ray, start, returnedPoint);

        let mut edgePA: vec3_t = [0.0; 3];
        VectorSubtract(A, returnedPoint, &mut edgePA);

        let mut edgePB: vec3_t = [0.0; 3];
        VectorSubtract(B, returnedPoint, &mut edgePB);

        let mut edgePC: vec3_t = [0.0; 3];
        VectorSubtract(C, returnedPoint, &mut edgePC);

        let mut temp: vec3_t = [0.0; 3];

        CrossProduct(&edgePA, &edgePB, &mut temp);
        if DotProduct(&temp, returnedNormal) < 0.0 {
            return qfalse; // off triangle
        }

        CrossProduct(&edgePC, &edgePA, &mut temp);
        if DotProduct(&temp, returnedNormal) < 0.0 {
            return qfalse; // off triangle
        }

        CrossProduct(&edgePB, &edgePC, &mut temp);
        if DotProduct(&temp, returnedNormal) < 0.0 {
            return qfalse; // off triangle
        }
        return qtrue;
    }
}

#[cfg(feature = "G2_GORE")]
fn G2_GorePolys(surface: *const mdxmSurface_t, TS: &mut CTraceSurface, surfInfo: *const mdxmSurfHierarchy_t) {
    unsafe {
        let mut j: c_int;
        let mut basis1: vec3_t = [0.0; 3];
        let mut basis2: vec3_t = [0.0; 3];
        let mut taxis: vec3_t = [0.0; 3];
        let mut saxis: vec3_t = [0.0; 3];

        basis2[0] = 0.0;
        basis2[1] = 0.0;
        basis2[2] = 1.0;

        CrossProduct(TS.rayEnd.as_ptr(), &basis2, &mut basis1);

        if DotProduct(&basis1, &basis1) < 0.1 {
            basis2[0] = 0.0;
            basis2[1] = 1.0;
            basis2[2] = 0.0;
            CrossProduct(TS.rayEnd.as_ptr(), &basis2, &mut basis1);
        }

        CrossProduct(TS.rayEnd.as_ptr(), &basis1, &mut basis2);
        // Give me a shot direction not a bunch of zeros :) -Gil
        assert!(DotProduct(&basis1, &basis1) > 0.0001);
        assert!(DotProduct(&basis2, &basis2) > 0.0001);

        VectorNormalize(&mut basis1);
        VectorNormalize(&mut basis2);

        let c = (TS.theta).cos();
        let s = (TS.theta).sin();

        VectorScale(&basis1, 0.5 * c / TS.tsize, &mut taxis);
        VectorMA(&taxis, 0.5 * s / TS.tsize, &basis2, &mut taxis);

        VectorScale(&basis1, -0.5 * s / TS.ssize, &mut saxis);
        VectorMA(&saxis, 0.5 * c / TS.ssize, &basis2, &mut saxis);

        let verts = (*(TS.TransformedVertsArray.add((*surface).thisSurfaceIndex as usize) as *const c_int) as *const f32);
        let numVerts = (*surface).numVerts;
        let mut flags = 15;
        assert!((numVerts as usize) < MAX_GORE_VERTS);

        for j in 0..numVerts {
            let pos = (j * 5) as usize;
            let mut delta: vec3_t = [
                verts.add(pos).read() - TS.rayStart[0],
                verts.add(pos + 1).read() - TS.rayStart[1],
                verts.add(pos + 2).read() - TS.rayStart[2],
            ];
            let s_val = DotProduct(&delta, &saxis) + 0.5;
            let t_val = DotProduct(&delta, &taxis) + 0.5;
            let mut vflags = 0;
            if s_val > GORE_MARGIN {
                vflags |= 1;
            }
            if s_val < 1.0 - GORE_MARGIN {
                vflags |= 2;
            }
            if t_val > GORE_MARGIN {
                vflags |= 4;
            }
            if t_val < 1.0 - GORE_MARGIN {
                vflags |= 8;
            }
            vflags = !vflags;
            flags &= vflags;
            GoreVerts[j as usize].flags = vflags;
            GoreVerts[j as usize].tex[0] = s_val;
            GoreVerts[j as usize].tex[1] = t_val;
        }

        if flags != 0 {
            return; // completely off the gore splotch.
        }

        // The rest of this function involves complex gore mesh processing
        // that would require proper Rust map/vector implementations for the C++ STL usage
        // Placeholder for structural parity
    }
}

// now we're at poly level, check each model space transformed poly against the model world transfomed ray
fn G2_TracePolys(surface: *const mdxmSurface_t, surfInfo: *const mdxmSurfHierarchy_t, TS: &mut CTraceSurface) -> bool {
    unsafe {
        let mut j: c_int;
        let numTris: c_int;

        // whip through and actually transform each vertex
        let tris = ((*surface as *const u8).add((*surface).ofsTriangles as usize) as *const mdxmTriangle_t);
        let verts = (*(TS.TransformedVertsArray.add((*surface).thisSurfaceIndex as usize) as *const c_int) as *const f32);
        let numTris = (*surface).numTriangles;

        for j in 0..numTris {
            let mut face: f32 = 0.0;
            let mut hitPoint: vec3_t = [0.0; 3];
            let mut normal: vec3_t = [0.0; 3];

            // determine actual coords for this triangle
            let point1 = verts.add(((*tris.add(j as usize)).indexes[0] as usize * 5));
            let point2 = verts.add(((*tris.add(j as usize)).indexes[1] as usize * 5));
            let point3 = verts.add(((*tris.add(j as usize)).indexes[2] as usize * 5));

            // did we hit it?
            let mut i: c_int;
            if G2_SegmentTriangleTest(TS.rayStart.as_ptr(), TS.rayEnd.as_ptr(),
                &[point1.read(), point1.add(1).read(), point1.add(2).read()],
                &[point2.read(), point2.add(1).read(), point2.add(2).read()],
                &[point3.read(), point3.add(1).read(), point3.add(2).read()],
                qtrue, qtrue, &mut hitPoint, &mut normal, &mut face) != 0 {

                // find space in the collision records for this record
                for i in 0..MAX_G2_COLLISIONS as c_int {
                    if (*TS.collRecMap.add(i as usize)).mEntityNum == -1 {
                        let newCol = &mut (*TS.collRecMap.add(i as usize));
                        let mut distVect: vec3_t = [0.0; 3];
                        let mut x_pos: f32 = 0.0;
                        let mut y_pos: f32 = 0.0;

                        newCol.mPolyIndex = j;
                        newCol.mEntityNum = TS.entNum;
                        newCol.mSurfaceIndex = (*surface).thisSurfaceIndex;
                        newCol.mModelIndex = TS.modelIndex;
                        if face > 0.0 {
                            newCol.mFlags = G2_FRONTFACE;
                        } else {
                            newCol.mFlags = G2_BACKFACE;
                        }

                        VectorSubtract(&hitPoint, TS.rayStart.as_ptr(), &mut distVect);
                        newCol.mDistance = VectorLength(&distVect);

                        // put the hit point back into world space
                        TransformAndTranslatePoint(&hitPoint, &mut newCol.mCollisionPosition, &mut worldMatrix);

                        // transform normal (but don't translate) into world angles
                        TransformPoint(&normal, &mut newCol.mCollisionNormal, &mut worldMatrix);
                        VectorNormalize(&mut newCol.mCollisionNormal);

                        newCol.mMaterial = 0;
                        newCol.mLocation = 0;

                        // Determine our location within the texture, and barycentric coordinates
                        G2_BuildHitPointST(
                            &[point1.read(), point1.add(1).read(), point1.add(2).read()],
                            point1.add(3).read(),
                            point1.add(4).read(),
                            &[point2.read(), point2.add(1).read(), point2.add(2).read()],
                            point2.add(3).read(),
                            point2.add(4).read(),
                            &[point3.read(), point3.add(1).read(), point3.add(2).read()],
                            point3.add(3).read(),
                            point3.add(4).read(),
                            &hitPoint,
                            &mut x_pos,
                            &mut y_pos,
                            &mut newCol.mBarycentricI,
                            &mut newCol.mBarycentricJ
                        );

                        // Commented out shader lookup code in original:
                        /*
                        const shader_t *shader = 0;
                        // now, we know what surface this hit belongs to, we need to go get the shader handle so we can get the correct hit location and hit material info
                        if ( cust_shader )
                        {
                            shader = cust_shader;
                        }
                        else if ( skin )
                        {
                            int j;
                            // match the surface name to something in the skin file
                            shader = tr.defaultShader;
                            for ( j = 0 ; j < skin->numSurfaces ; j++ )
                            {
                                // the names have both been lowercased
                                if ( !strcmp( skin->surfaces[j]->name, surfInfo->name ) )
                                {
                                    shader = skin->surfaces[j]->shader;
                                    break;
                                }
                            }
                        }
                        else
                        {
                            shader = R_GetShaderByHandle( surfInfo->shaderIndex );
                        }

                        // do we even care to decide what the hit or location area's are? If we don't have them in the shader there is little point
                        if ((shader->hitLocation) || (shader->hitMaterial))
                        {
                            // ok, we have a floating point position. - determine location in data we need to look at
                            if (shader->hitLocation)
                            {
                                newCol.mLocation = *(hitMatReg[shader->hitLocation].loc +
                                                    ((int)(y_pos * hitMatReg[shader->hitLocation].height) * hitMatReg[shader->hitLocation].width) +
                                                    ((int)(x_pos * hitMatReg[shader->hitLocation].width)));
                                Com_Printf("G2_TracePolys hit location: %d\n", newCol.mLocation);
                            }

                            if (shader->hitMaterial)
                            {
                                newCol.mMaterial = *(hitMatReg[shader->hitMaterial].loc +
                                                    ((int)(y_pos * hitMatReg[shader->hitMaterial].height) * hitMatReg[shader->hitMaterial].width) +
                                                    ((int)(x_pos * hitMatReg[shader->hitMaterial].width)));
                            }
                        }
                        */

                        // exit now if we should
                        if TS.traceFlags == G2_RETURNONHIT {
                            TS.hitOne = true;
                            return true;
                        }

                        break;
                    }
                }
                if i as usize == MAX_G2_COLLISIONS {
                    // run out of collision record space - will probably never happen
                    // It happens. And the assert is bugging me.
                    TS.hitOne = true;
                    return true; // return true to avoid wasting further time, but no hit will result without a record
                }
            }
        }
        return false;
    }
}

// now we're at poly level, check each model space transformed poly against the model world transfomed ray
fn G2_RadiusTracePolys(surface: *const mdxmSurface_t, TS: &mut CTraceSurface) -> bool {
    unsafe {
        let mut j: c_int;
        let mut basis1: vec3_t = [0.0; 3];
        let mut basis2: vec3_t = [0.0; 3];
        let mut taxis: vec3_t = [0.0; 3];
        let mut saxis: vec3_t = [0.0; 3];

        basis2[0] = 0.0;
        basis2[1] = 0.0;
        basis2[2] = 1.0;

        let mut v3RayDir: vec3_t = [0.0; 3];
        VectorSubtract(TS.rayEnd.as_ptr(), TS.rayStart.as_ptr(), &mut v3RayDir);

        CrossProduct(&v3RayDir, &basis2, &mut basis1);

        if DotProduct(&basis1, &basis1) < 0.1 {
            basis2[0] = 0.0;
            basis2[1] = 1.0;
            basis2[2] = 0.0;
            CrossProduct(&v3RayDir, &basis2, &mut basis1);
        }

        CrossProduct(&v3RayDir, &basis1, &mut basis2);
        // Give me a shot direction not a bunch of zeros :) -Gil
        // assert(DotProduct(basis1,basis1)>.0001f);
        // assert(DotProduct(basis2,basis2)>.0001f);

        VectorNormalize(&mut basis1);
        VectorNormalize(&mut basis2);

        let c = (0.0f32).cos(); // theta
        let s = (0.0f32).sin(); // theta

        VectorScale(&basis1, 0.5 * c / TS.m_fRadius, &mut taxis);
        VectorMA(&taxis, 0.5 * s / TS.m_fRadius, &basis2, &mut taxis);

        VectorScale(&basis1, -0.5 * s / TS.m_fRadius, &mut saxis);
        VectorMA(&saxis, 0.5 * c / TS.m_fRadius, &basis2, &mut saxis);

        let verts = (*(TS.TransformedVertsArray.add((*surface).thisSurfaceIndex as usize) as *const c_int) as *const f32);
        let numVerts = (*surface).numVerts;

        let mut flags = 63;
        let f = VectorLengthSquared(&v3RayDir);
        v3RayDir[0] /= f;
        v3RayDir[1] /= f;
        v3RayDir[2] /= f;

        for j in 0..numVerts {
            let pos = (j * 5) as usize;
            let mut delta: vec3_t = [
                verts.add(pos).read() - TS.rayStart[0],
                verts.add(pos + 1).read() - TS.rayStart[1],
                verts.add(pos + 2).read() - TS.rayStart[2],
            ];
            let s = DotProduct(&delta, &saxis) + 0.5;
            let t = DotProduct(&delta, &taxis) + 0.5;
            let u = DotProduct(&delta, &v3RayDir);
            let mut vflags = 0;

            if s > 0.0 {
                vflags |= 1;
            }
            if s < 1.0 {
                vflags |= 2;
            }
            if t > 0.0 {
                vflags |= 4;
            }
            if t < 1.0 {
                vflags |= 8;
            }
            if u > 0.0 {
                vflags |= 16;
            }
            if u < 1.0 {
                vflags |= 32;
            }

            vflags = !vflags;
            flags &= vflags;
            GoreVerts[j as usize].flags = vflags;
        }

        if flags != 0 {
            return false; // completely off the gore splotch (so presumably hit nothing? -Ste)
        }

        let numTris = (*surface).numTriangles;
        let tris = ((*surface as *const u8).add((*surface).ofsTriangles as usize) as *const mdxmTriangle_t);

        for j in 0..numTris {
            assert!((*tris.add(j as usize)).indexes[0] >= 0 && (*tris.add(j as usize)).indexes[0] < numVerts);
            assert!((*tris.add(j as usize)).indexes[1] >= 0 && (*tris.add(j as usize)).indexes[1] < numVerts);
            assert!((*tris.add(j as usize)).indexes[2] >= 0 && (*tris.add(j as usize)).indexes[2] < numVerts);

            let flags = 63 &
                GoreVerts[(*tris.add(j as usize)).indexes[0] as usize].flags &
                GoreVerts[(*tris.add(j as usize)).indexes[1] as usize].flags &
                GoreVerts[(*tris.add(j as usize)).indexes[2] as usize].flags;

            let mut i: c_int;
            if flags != 0 {
                continue;
            } else {
                // we hit a triangle, so init a collision record...
                for i in 0..MAX_G2_COLLISIONS as c_int {
                    if (*TS.collRecMap.add(i as usize)).mEntityNum == -1 {
                        let newCol = &mut (*TS.collRecMap.add(i as usize));

                        newCol.mPolyIndex = j;
                        newCol.mEntityNum = TS.entNum;
                        newCol.mSurfaceIndex = (*surface).thisSurfaceIndex;
                        newCol.mModelIndex = TS.modelIndex;

                        newCol.mFlags = G2_FRONTFACE;

                        // get normal from triangle
                        let A = verts.add(((*tris.add(j as usize)).indexes[0] as usize * 5));
                        let B = verts.add(((*tris.add(j as usize)).indexes[1] as usize * 5));
                        let C = verts.add(((*tris.add(j as usize)).indexes[2] as usize * 5));
                        let mut normal: vec3_t = [0.0; 3];
                        let mut edgeAC: vec3_t = [0.0; 3];
                        let mut edgeBA: vec3_t = [0.0; 3];

                        VectorSubtract(
                            &[C.read(), C.add(1).read(), C.add(2).read()],
                            &[A.read(), A.add(1).read(), A.add(2).read()],
                            &mut edgeAC
                        );
                        VectorSubtract(
                            &[B.read(), B.add(1).read(), B.add(2).read()],
                            &[A.read(), A.add(1).read(), A.add(2).read()],
                            &mut edgeBA
                        );
                        CrossProduct(&edgeBA, &edgeAC, &mut normal);

                        // transform normal (but don't translate) into world angles
                        TransformPoint(&normal, &mut newCol.mCollisionNormal, &mut worldMatrix);
                        VectorNormalize(&mut newCol.mCollisionNormal);

                        newCol.mMaterial = 0;
                        newCol.mLocation = 0;

                        // exit now if we should
                        if TS.traceFlags == G2_RETURNONHIT {
                            TS.hitOne = true;
                            return true;
                        }

                        let mut distVect: vec3_t = [0.0; 3];

                        // Yeah, I want the collision point. Let's work out the impact point on the triangle. -rww
                        let mut hitPoint: vec3_t = [0.0; 3];
                        let side: f32;
                        let mut side2: f32;
                        let dist: f32;
                        let third = -(A.read() * (B.add(1).read() * C.add(2).read() - C.add(1).read() * B.add(2).read()) +
                                     B.read() * (C.add(1).read() * A.add(2).read() - A.add(1).read() * C.add(2).read()) +
                                     C.read() * (A.add(1).read() * B.add(2).read() - B.add(1).read() * A.add(2).read()));

                        VectorSubtract(TS.rayEnd.as_ptr(), TS.rayStart.as_ptr(), &mut distVect);
                        side = normal[0] * TS.rayStart[0] + normal[1] * TS.rayStart[1] + normal[2] * TS.rayStart[2] + third;
                        side2 = normal[0] * distVect[0] + normal[1] * distVect[1] + normal[2] * distVect[2];
                        dist = side / side2;
                        VectorMA(TS.rayStart.as_ptr(), -dist, &distVect, &mut hitPoint);

                        VectorSubtract(&hitPoint, TS.rayStart.as_ptr(), &mut distVect);
                        newCol.mDistance = VectorLength(&distVect);

                        // put the hit point back into world space
                        TransformAndTranslatePoint(&hitPoint, &mut newCol.mCollisionPosition, &mut worldMatrix);
                        newCol.mBarycentricI = 0.0;
                        newCol.mBarycentricJ = 0.0;
                        break;
                    }
                }
                if i as usize == MAX_G2_COLLISIONS {
                    TS.hitOne = true;
                    return true; // return true to avoid wasting further time, but no hit will result without a record
                }
            }
        }

        return false;
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CTraceSurface {
    pub surfaceNum: c_int,
    pub rootSList: *mut surfaceInfo_v,
    pub currentModel: *mut model_t,
    pub lod: c_int,
    pub rayStart: vec3_t,
    pub rayEnd: vec3_t,
    pub collRecMap: *mut CollisionRecord_t,
    pub entNum: c_int,
    pub modelIndex: c_int,
    pub skin: *mut skin_t,
    pub cust_shader: *mut shader_t,
    pub TransformedVertsArray: *mut c_int,
    pub traceFlags: c_int,
    pub hitOne: bool,
    pub m_fRadius: f32,
    #[cfg(feature = "G2_GORE")]
    pub ssize: f32,
    #[cfg(feature = "G2_GORE")]
    pub tsize: f32,
    #[cfg(feature = "G2_GORE")]
    pub theta: f32,
    #[cfg(feature = "G2_GORE")]
    pub goreShader: c_int,
    #[cfg(feature = "G2_GORE")]
    pub ghoul2info: *mut CGhoul2Info,
    #[cfg(feature = "G2_GORE")]
    pub gore: *mut SSkinGoreData,
}

// look at a surface and then do the trace on each poly
fn G2_TraceSurfaces(TS: &mut CTraceSurface) {
    unsafe {
        let mut i: c_int;
        // back track and get the surfinfo struct for this surface
        assert!(!TS.currentModel.is_null());
        assert!(!(*TS.currentModel).mdxm.is_null());

        let surface = G2_FindSurface(TS.currentModel as *mut c_void, TS.surfaceNum, TS.lod) as *const mdxmSurface_t;
        let surfIndexes = (((*TS.currentModel).mdxm as *const u8).add(mem::size_of::<mdxmHeader_t>()) as *const mdxmHierarchyOffsets_t);
        let surfInfo = ((surfIndexes as *const u8).add((*surfIndexes).offsets[(*surface).thisSurfaceIndex as usize] as usize) as *const mdxmSurfHierarchy_t);

        // see if we have an override surface in the surface list
        let surfOverride = G2_FindOverrideSurface(TS.surfaceNum, TS.rootSList as *mut c_void) as *const surfaceInfo_t;

        // don't allow recursion if we've already hit a polygon
        if TS.hitOne {
            return;
        }

        // really, we should use the default flags for this surface unless it's been overriden
        let mut offFlags = (*surfInfo).flags;

        // set the off flags if we have some
        if !surfOverride.is_null() {
            offFlags = (*surfOverride).offFlags;
        }

        // if this surface is not off, try to hit it
        if offFlags == 0 {
            #[cfg(feature = "G2_GORE")]
            {
                if !TS.collRecMap.is_null() {
                    if !(TS.m_fRadius.abs() < 0.1) { // if not a point-trace
                        // .. then use radius check
                        if G2_RadiusTracePolys(surface, TS) && (TS.traceFlags == G2_RETURNONHIT) {
                            TS.hitOne = true;
                            return;
                        }
                    } else {
                        // go away and trace the polys in this surface
                        if G2_TracePolys(surface, surfInfo, TS) && (TS.traceFlags == G2_RETURNONHIT) {
                            // ok, we hit one, *and* we want to return instantly because the returnOnHit is set
                            // so indicate we've hit one, so other surfaces don't get hit and return
                            TS.hitOne = true;
                            return;
                        }
                    }
                } else {
                    G2_GorePolys(surface, TS, surfInfo);
                }
            }
            #[cfg(not(feature = "G2_GORE"))]
            {
                if !(TS.m_fRadius.abs() < 0.1) { // if not a point-trace
                    if G2_RadiusTracePolys(surface, TS) && (TS.traceFlags == G2_RETURNONHIT) {
                        TS.hitOne = true;
                        return;
                    }
                } else {
                    if G2_TracePolys(surface, surfInfo, TS) && (TS.traceFlags == G2_RETURNONHIT) {
                        TS.hitOne = true;
                        return;
                    }
                }
            }
        }

        // if we are turning off all descendants, then stop this recursion now
        if offFlags & G2SURFACEFLAG_NODESCENDANTS != 0 {
            return;
        }

        // now recursively call for the children
        for i in 0..(*surfInfo).numChildren {
            if !TS.hitOne {
                TS.surfaceNum = (*surfInfo).childIndexes[i as usize];
                G2_TraceSurfaces(TS);
            }
        }
    }
}

#[cfg(feature = "G2_GORE")]
pub fn G2_TraceModels(ghoul2: &mut CGhoul2Info_v, rayStart: *const vec3_t, rayEnd: *const vec3_t, collRecMap: *mut CollisionRecord_t, entNum: c_int, eG2TraceType: c_int, useLod: c_int, fRadius: f32, ssize: f32, tsize: f32, theta: f32, shader: c_int, gore: *mut SSkinGoreData, skipIfLODNotMatch: qboolean) {
    G2_TraceModels_impl(ghoul2, rayStart, rayEnd, collRecMap, entNum, eG2TraceType, useLod, fRadius, ssize, tsize, theta, shader, gore, skipIfLODNotMatch);
}

#[cfg(not(feature = "G2_GORE"))]
pub fn G2_TraceModels(ghoul2: &mut CGhoul2Info_v, rayStart: *const vec3_t, rayEnd: *const vec3_t, collRecMap: *mut CollisionRecord_t, entNum: c_int, eG2TraceType: c_int, useLod: c_int, fRadius: f32) {
    G2_TraceModels_impl(ghoul2, rayStart, rayEnd, collRecMap, entNum, eG2TraceType, useLod, fRadius, 0.0, 0.0, 0.0, 0, ptr::null_mut(), qtrue);
}

#[cfg(feature = "G2_GORE")]
fn G2_TraceModels_impl(ghoul2: &mut CGhoul2Info_v, rayStart: *const vec3_t, rayEnd: *const vec3_t, collRecMap: *mut CollisionRecord_t, entNum: c_int, eG2TraceType: c_int, useLod: c_int, fRadius: f32, ssize: f32, tsize: f32, theta: f32, shader: c_int, gore: *mut SSkinGoreData, skipIfLODNotMatch: qboolean) {
    unsafe {
        let mut i: c_int;
        let mut lod: c_int;
        let mut skin: *mut skin_t;
        let mut cust_shader: *mut shader_t;
        let mut firstModelOnly = qfalse;

        if cg_g2MarksAllModels.is_null() {
            cg_g2MarksAllModels = Cvar_Get(b"cg_g2MarksAllModels\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        if cg_g2MarksAllModels.is_null() || (*cg_g2MarksAllModels).integer == 0 {
            firstModelOnly = qtrue;
        }

        // walk each possible model for this entity and try tracing against it
        for i in 0..(ghoul2.len() as c_int) {
            goreModelIndex = i;

            // don't bother with models that we don't care about.
            if ghoul2[i as usize].mModelindex == -1 {
                continue;
            }

            // don't bother with models that we don't care about.
            if ghoul2[i as usize].mValid == 0 {
                continue;
            }

            // do we really want to collide with this object?
            if (ghoul2[i as usize].mFlags & GHOUL2_NOCOLLIDE) != 0 {
                continue;
            }

            if ghoul2[i as usize].mCustomShader != 0 && ghoul2[i as usize].mCustomShader != -20 { // rww - -20 is a server instance (hack)
                cust_shader = R_GetShaderByHandle(ghoul2[i as usize].mCustomShader);
            } else {
                cust_shader = ptr::null_mut();
            }

            // figure out the custom skin thing
            if ghoul2[i as usize].mSkin > 0 && ghoul2[i as usize].mSkin < 0 { // tr.numSkins would be external
                skin = R_GetSkinByHandle(ghoul2[i as usize].mSkin);
            } else {
                skin = ptr::null_mut();
            }

            lod = G2_DecideTraceLod(&ghoul2[i as usize], useLod);

            if skipIfLODNotMatch != 0 {
                // we only want to hit this SPECIFIC LOD...
                if lod != useLod {
                    // doesn't match, skip this model
                    continue;
                }
            }

            // reset the quick surface override lookup
            G2_FindOverrideSurface(-1, &mut ghoul2[i as usize].mSlist as *mut c_void);

            let mut TS = CTraceSurface {
                surfaceNum: ghoul2[i as usize].mSurfaceRoot,
                rootSList: &mut ghoul2[i as usize].mSlist,
                currentModel: ghoul2[i as usize].currentModel,
                lod,
                rayStart: *rayStart,
                rayEnd: *rayEnd,
                collRecMap,
                entNum,
                modelIndex: i,
                skin,
                cust_shader,
                TransformedVertsArray: ghoul2[i as usize].mTransformedVertsArray,
                traceFlags: eG2TraceType,
                hitOne: false,
                m_fRadius: fRadius,
                #[cfg(feature = "G2_GORE")]
                ssize,
                #[cfg(feature = "G2_GORE")]
                tsize,
                #[cfg(feature = "G2_GORE")]
                theta,
                #[cfg(feature = "G2_GORE")]
                goreShader: shader,
                #[cfg(feature = "G2_GORE")]
                ghoul2info: &mut ghoul2[i as usize],
                #[cfg(feature = "G2_GORE")]
                gore,
            };

            // start the surface recursion loop
            G2_TraceSurfaces(&mut TS);

            // if we've hit one surface on one model, don't bother doing the rest
            if TS.hitOne {
                break;
            }

            if !collRecMap.is_null() && firstModelOnly != 0 {
                // we don't really need to do multiple models for gore.
                break;
            }
        }
    }
}

#[cfg(not(feature = "G2_GORE"))]
fn G2_TraceModels_impl(ghoul2: &mut CGhoul2Info_v, rayStart: *const vec3_t, rayEnd: *const vec3_t, collRecMap: *mut CollisionRecord_t, entNum: c_int, eG2TraceType: c_int, useLod: c_int, fRadius: f32, _ssize: f32, _tsize: f32, _theta: f32, _shader: c_int, _gore: *mut SSkinGoreData, _skipIfLODNotMatch: qboolean) {
    unsafe {
        let mut i: c_int;
        let mut lod: c_int;
        let mut skin: *mut skin_t;
        let mut cust_shader: *mut shader_t;
        let mut firstModelOnly = qfalse;

        if cg_g2MarksAllModels_nongore.is_null() {
            cg_g2MarksAllModels_nongore = Cvar_Get(b"cg_g2MarksAllModels\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        if cg_g2MarksAllModels_nongore.is_null() || (*cg_g2MarksAllModels_nongore).integer == 0 {
            firstModelOnly = qtrue;
        }

        // walk each possible model for this entity and try tracing against it
        for i in 0..(ghoul2.len() as c_int) {
            // don't bother with models that we don't care about.
            if ghoul2[i as usize].mValid == 0 {
                continue;
            }

            // do we really want to collide with this object?
            if (ghoul2[i as usize].mFlags & GHOUL2_NOCOLLIDE) != 0 {
                continue;
            }

            if ghoul2[i as usize].mCustomShader != 0 && ghoul2[i as usize].mCustomShader != -20 {
                cust_shader = R_GetShaderByHandle(ghoul2[i as usize].mCustomShader);
            } else {
                cust_shader = ptr::null_mut();
            }

            // figure out the custom skin thing
            if ghoul2[i as usize].mSkin > 0 && ghoul2[i as usize].mSkin < 0 {
                skin = R_GetSkinByHandle(ghoul2[i as usize].mSkin);
            } else {
                skin = ptr::null_mut();
            }

            lod = G2_DecideTraceLod(&ghoul2[i as usize], useLod);

            // reset the quick surface override lookup
            G2_FindOverrideSurface(-1, &mut ghoul2[i as usize].mSlist as *mut c_void);

            let mut TS = CTraceSurface {
                surfaceNum: ghoul2[i as usize].mSurfaceRoot,
                rootSList: &mut ghoul2[i as usize].mSlist,
                currentModel: ghoul2[i as usize].currentModel,
                lod,
                rayStart: *rayStart,
                rayEnd: *rayEnd,
                collRecMap,
                entNum,
                modelIndex: i,
                skin,
                cust_shader,
                TransformedVertsArray: ghoul2[i as usize].mTransformedVertsArray,
                traceFlags: eG2TraceType,
                hitOne: false,
                m_fRadius: fRadius,
            };

            // start the surface recursion loop
            G2_TraceSurfaces(&mut TS);

            // if we've hit one surface on one model, don't bother doing the rest
            if TS.hitOne {
                break;
            }
        }
    }
}

pub fn TransformPoint(in_: *const vec3_t, out: *mut vec3_t, mat: *mut mdxaBone_t) {
    unsafe {
        for i in 0..3 {
            (*out)[i] = (*in_)[0] * (*mat).matrix[i][0] + (*in_)[1] * (*mat).matrix[i][1] + (*in_)[2] * (*mat).matrix[i][2];
        }
    }
}

pub fn TransformAndTranslatePoint(in_: *const vec3_t, out: *mut vec3_t, mat: *mut mdxaBone_t) {
    unsafe {
        for i in 0..3 {
            (*out)[i] = (*in_)[0] * (*mat).matrix[i][0] + (*in_)[1] * (*mat).matrix[i][1] + (*in_)[2] * (*mat).matrix[i][2] + (*mat).matrix[i][3];
        }
    }
}

// create a matrix using a set of angles
pub fn Create_Matrix(angle: *const vec3_t, matrix: *mut mdxaBone_t) {
    unsafe {
        let mut axis: [[f32; 3]; 3] = [[0.0; 3]; 3];

        // convert angles to axis
        AnglesToAxis(angle, &mut axis);
        (*matrix).matrix[0][0] = axis[0][0];
        (*matrix).matrix[1][0] = axis[0][1];
        (*matrix).matrix[2][0] = axis[0][2];

        (*matrix).matrix[0][1] = axis[1][0];
        (*matrix).matrix[1][1] = axis[1][1];
        (*matrix).matrix[2][1] = axis[1][2];

        (*matrix).matrix[0][2] = axis[2][0];
        (*matrix).matrix[1][2] = axis[2][1];
        (*matrix).matrix[2][2] = axis[2][2];

        (*matrix).matrix[0][3] = 0.0;
        (*matrix).matrix[1][3] = 0.0;
        (*matrix).matrix[2][3] = 0.0;
    }
}

// given a matrix, generate the inverse of that matrix
pub fn Inverse_Matrix(src: *mut mdxaBone_t, dest: *mut mdxaBone_t) {
    unsafe {
        let mut i: usize;
        let mut j: usize;

        for i in 0..3 {
            for j in 0..3 {
                (*dest).matrix[i][j] = (*src).matrix[j][i];
            }
        }
        for i in 0..3 {
            (*dest).matrix[i][3] = 0.0;
            for j in 0..3 {
                (*dest).matrix[i][3] -= (*dest).matrix[i][j] * (*src).matrix[j][3];
            }
        }
    }
}

// generate the world matrix for a given set of angles and origin - called from lots of places
pub fn G2_GenerateWorldMatrix(angles: *const vec3_t, origin: *const vec3_t) {
    unsafe {
        Create_Matrix(angles, &mut worldMatrix);
        worldMatrix.matrix[0][3] = (*origin)[0];
        worldMatrix.matrix[1][3] = (*origin)[1];
        worldMatrix.matrix[2][3] = (*origin)[2];

        Inverse_Matrix(&mut worldMatrix, &mut worldMatrixInv);
    }
}

// go away and determine what the pointer for a specific surface definition within the model definition is
pub fn G2_FindSurface_impl(mod_t: *mut model_t, index: c_int, lod: c_int) -> *mut c_void {
    unsafe {
        // point at first lod list
        let mut current = ((*mod_t).mdxm as *const u8).add((*(*mod_t).mdxm).ofsLODs as usize) as *mut u8;
        let mut i: c_int;

        //walk the lods
        for i in 0..lod {
            let lodData = current as *mut mdxmLOD_t;
            current = current.add((*lodData).ofsEnd as usize);
        }

        // avoid the lod pointer data structure
        current = current.add(mem::size_of::<mdxmLOD_t>());

        let indexes = current as *mut mdxmLODSurfOffset_t;
        // we are now looking at the offset array
        current = current.add((*indexes).offsets[index as usize] as usize);

        current as *mut c_void
    }
}

const SURFACE_SAVE_BLOCK_SIZE: usize = mem::size_of::<surfaceInfo_t>();
// #define BOLT_SAVE_BLOCK_SIZE (sizeof(boltInfo_t) - sizeof(mdxaBone_t))
const BONE_SAVE_BLOCK_SIZE: usize = mem::size_of::<boneInfo_t>();

pub fn G2_SaveGhoul2Models(ghoul2: &CGhoul2Info_v, buffer: *mut *mut c_char, size: *mut c_int) -> qboolean {
    unsafe {
        // is there anything to save?
        if ghoul2.is_empty() {
            *buffer = Z_Malloc(4, TAG_GHOUL2, qtrue) as *mut c_char;
            let tempBuffer = *buffer as *mut c_int;
            *tempBuffer = 0;
            *size = 4;
            return qtrue;
        }

        // yeah, lets get busy
        *size = 0;

        // this one isn't a define since I couldn't work out how to figure it out at compile time
        // Size from mModelindex to mTransformedVertsArray
        let ghoul2BlockSize = mem::size_of::<c_int>() * 13; // Approximation for member offset

        // add in count for number of ghoul2 models
        *size += 4;
        // start out working out the total size of the buffer we need to allocate
        for i in 0..ghoul2.len() {
            *size += ghoul2BlockSize as c_int;
            // add in count for number of surfaces
            *size += 4;
            *size += ((ghoul2[i].mSlist.len() as c_int) * SURFACE_SAVE_BLOCK_SIZE as c_int);
            // add in count for number of bones
            *size += 4;
            *size += ((ghoul2[i].mBlist.len() as c_int) * BONE_SAVE_BLOCK_SIZE as c_int);
            // add in count for number of bolts
            *size += 4;
            // BOLT_SAVE_BLOCK_SIZE would be used here
        }

        // ok, we should know how much space we need now
        *buffer = Z_Malloc(*size, TAG_GHOUL2, qtrue) as *mut c_char;

        // now lets start putting the data we care about into the buffer
        let mut tempBuffer = *buffer;

        // save out how many ghoul2 models we have
        *(tempBuffer as *mut c_int) = ghoul2.len() as c_int;
        tempBuffer = tempBuffer.add(4);

        for i in 0..ghoul2.len() {
            // first save out the ghoul2 details themselves
            core::ptr::copy_nonoverlapping(&ghoul2[i].mModelindex as *const c_int as *const c_char, tempBuffer, ghoul2BlockSize);
            tempBuffer = tempBuffer.add(ghoul2BlockSize);

            // save out how many surfaces we have
            *(tempBuffer as *mut c_int) = ghoul2[i].mSlist.len() as c_int;
            tempBuffer = tempBuffer.add(4);

            // now save the all the surface list info
            for x in 0..ghoul2[i].mSlist.len() {
                core::ptr::copy_nonoverlapping(&ghoul2[i].mSlist[x] as *const surfaceInfo_t as *const c_char, tempBuffer, SURFACE_SAVE_BLOCK_SIZE);
                tempBuffer = tempBuffer.add(SURFACE_SAVE_BLOCK_SIZE);
            }

            // save out how many bones we have
            *(tempBuffer as *mut c_int) = ghoul2[i].mBlist.len() as c_int;
            tempBuffer = tempBuffer.add(4);

            // now save the all the bone list info
            for x in 0..ghoul2[i].mBlist.len() {
                core::ptr::copy_nonoverlapping(&ghoul2[i].mBlist[x] as *const boneInfo_t as *const c_char, tempBuffer, BONE_SAVE_BLOCK_SIZE);
                tempBuffer = tempBuffer.add(BONE_SAVE_BLOCK_SIZE);
            }

            // save out how many bolts we have
            *(tempBuffer as *mut c_int) = ghoul2[i].mBltlist.len() as c_int;
            tempBuffer = tempBuffer.add(4);

            // lastly save the all the bolt list info
            // for (x=0; x<ghoul2[i].mBltlist.size(); x++)
            // {
            //     memcpy(tempBuffer, &ghoul2[i].mBltlist[x], BOLT_SAVE_BLOCK_SIZE);
            //     tempBuffer += BOLT_SAVE_BLOCK_SIZE;
            // }
        }

        return qtrue;
    }
}

// have to free space malloced in the save system here because the game DLL can't.
pub fn G2_FreeSaveBuffer(buffer: *mut c_char) {
    unsafe {
        Z_Free(buffer as *mut c_void);
    }
}

pub fn G2_FindConfigStringSpace(name: *mut c_char, start: c_int, max: c_int) -> c_int {
    unsafe {
        let mut s: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
        let mut i: c_int;

        for i in 1..max {
            SV_GetConfigstring(start + i, s.as_mut_ptr(), MAX_STRING_CHARS as c_int);
            if s[0] == 0 {
                break;
            }
            if strcmp(s.as_ptr(), name) == 0 {
                return i;
            }
        }

        SV_SetConfigstring(start + i, s.as_ptr());
        return i;
    }
}

// Forward declarations
extern "C" {
    fn G2_SetupModelPointers(ghlInfo: *mut CGhoul2Info) -> qboolean;
}

pub fn G2_LoadGhoul2Model(ghoul2: &mut CGhoul2Info_v, buffer: *mut c_char) {
    unsafe {
        // first thing, lets see how many ghoul2 models we have, and resize our buffers accordingly
        let newSize = *(buffer as *const c_int);
        ghoul2.resize(newSize as usize, Default::default());
        let mut buffer = buffer.add(4);

        // did we actually resize to a value?
        if newSize == 0 {
            // no, ok, well, done then.
            return;
        }

        // this one isn't a define since I couldn't work out how to figure it out at compile time
        let ghoul2BlockSize = mem::size_of::<c_int>() * 13;

        // now we have enough instances, lets go through each one and load up the relevant details
        for i in 0..ghoul2.len() {
            ghoul2[i].mSkelFrameNum = 0;
            ghoul2[i].mModelindex = -1;
            ghoul2[i].mFileName[0] = 0;
            ghoul2[i].mValid = 0;

            // load the ghoul2 info from the buffer
            core::ptr::copy_nonoverlapping(buffer as *const c_char, &mut ghoul2[i].mModelindex as *mut c_int as *mut c_char, ghoul2BlockSize);
            buffer = buffer.add(ghoul2BlockSize);

            if ghoul2[i].mModelindex != -1 && ghoul2[i].mFileName[0] != 0 {
                ghoul2[i].mModelindex = i as c_int;
                G2_SetupModelPointers(&mut ghoul2[i]);
            }

            // give us enough surfaces to load up the data
            let surfSize = *(buffer as *const c_int);
            // ghoul2[i].mSlist.resize(surfSize as usize);
            buffer = buffer.add(4);

            // now load all the surfaces
            for x in 0..surfSize {
                // memcpy(&ghoul2[i].mSlist[x], buffer, SURFACE_SAVE_BLOCK_SIZE);
                buffer = buffer.add(SURFACE_SAVE_BLOCK_SIZE);
            }

            // give us enough bones to load up the data
            let boneSize = *(buffer as *const c_int);
            // ghoul2[i].mBlist.resize(boneSize as usize);
            buffer = buffer.add(4);

            // now load all the bones
            for x in 0..boneSize {
                // memcpy(&ghoul2[i].mBlist[x], buffer, BONE_SAVE_BLOCK_SIZE);
                buffer = buffer.add(BONE_SAVE_BLOCK_SIZE);
            }

            // give us enough bolts to load up the data
            let boltSize = *(buffer as *const c_int);
            // ghoul2[i].mBltlist.resize(boltSize as usize);
            buffer = buffer.add(4);

            // now load all the bolts
            for x in 0..boltSize {
                // memcpy(&ghoul2[i].mBltlist[x], buffer, BOLT_SAVE_BLOCK_SIZE);
                // buffer += BOLT_SAVE_BLOCK_SIZE;
            }
        }
    }
}

pub fn G2_LerpAngles(ghoul2: &mut CGhoul2Info_v, nextGhoul2: &CGhoul2Info_v, interpolation: f32) {
    // loop each model
    for i in 0..ghoul2.len() {
        if ghoul2[i].mModelindex != -1 {
            // now walk the bone list
            for x in 0..ghoul2[i].mBlist.len() {
                let bone = &mut ghoul2[i].mBlist[x];
                // sure we have one to lerp to?
                if (nextGhoul2.len() > i) &&
                    (nextGhoul2[i].mModelindex != -1) &&
                    (nextGhoul2[i].mBlist.len() > x) &&
                    (nextGhoul2[i].mBlist[x].boneNumber != -1)
                {
                    let nextBone = &nextGhoul2[i].mBlist[x];
                    // does this bone override actually have anything in it, and if it does, is it a bone angles override?
                    if (bone.boneNumber != -1) && ((bone.flags) & (0x01 | 0x02)) != 0 { // BONE_ANGLES_TOTAL would be defined
                        let nowMatrix = &bone.matrix.matrix as *const _ as *const f32;
                        let nextMatrix = &nextBone.matrix.matrix as *const _ as *const f32;
                        let newMatrix = &mut bone.newMatrix.matrix as *mut _ as *mut f32;
                        // now interpolate the matrix
                        for z in 0..12 {
                            unsafe {
                                *newMatrix.add(z) = *nowMatrix.add(z) + interpolation * (*nextMatrix.add(z) - *nowMatrix.add(z));
                            }
                        }
                    }
                } else {
                    core::ptr::copy_nonoverlapping(&ghoul2[i].mBlist[x].matrix as *const mdxaBone_t,
                        &mut ghoul2[i].mBlist[x].newMatrix,
                        1);
                }
            }
        }
    }
}

// Helper function for string comparison
fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut p1 = s1;
        let mut p2 = s2;
        loop {
            let c1 = *p1 as u8;
            let c2 = *p2 as u8;
            if c1 != c2 {
                return (c1 as i32) - (c2 as i32);
            }
            if c1 == 0 {
                return 0;
            }
            p1 = p1.add(1);
            p2 = p2.add(1);
        }
    }
}
