// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use std::collections::{HashMap, BTreeMap};
use std::mem;

// External types and declarations would go here
// For now, using placeholder types to represent missing dependencies

// Extern declarations for libc functions
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn fabsf(x: f32) -> f32;
}

// Type definitions from headers (stubbed to maintain structure)
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

// Constants from C
pub const qtrue: c_int = 1;
pub const qfalse: c_int = 0;

pub const Q3_ASE: c_int = 1;
pub const Q3_SM: c_int = 1;

pub const MAX_STRING_CHARS: usize = 1024;
pub const ERR_DROP: c_int = 1;

pub const G2_RETURNONHIT: c_int = 1;
pub const G2_FRONTFACE: c_int = 1;
pub const G2_BACKFACE: c_int = 2;
pub const G2SURFACEFLAG_NODESCENDANTS: c_int = 0x1;
pub const GHOUL2_NOCOLLIDE: c_int = 0x2;

pub const TAG_GHOUL2: c_int = 1;

pub const MAX_G2_COLLISIONS: usize = 32;

// Vector operations
pub fn VectorCopy(src: &vec3_t, dst: &mut vec3_t) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

pub fn VectorClear(v: &mut vec3_t) {
    v[0] = 0.0;
    v[1] = 0.0;
    v[2] = 0.0;
}

pub fn VectorSubtract(a: &vec3_t, b: &vec3_t, out: &mut vec3_t) {
    out[0] = a[0] - b[0];
    out[1] = a[1] - b[1];
    out[2] = a[2] - b[2];
}

pub fn VectorAdd(a: &vec3_t, b: &vec3_t, out: &mut vec3_t) {
    out[0] = a[0] + b[0];
    out[1] = a[1] + b[1];
    out[2] = a[2] + b[2];
}

pub fn VectorScale(v: &vec3_t, scale: f32, out: &mut vec3_t) {
    out[0] = v[0] * scale;
    out[1] = v[1] * scale;
    out[2] = v[2] * scale;
}

pub fn VectorMA(v: &vec3_t, scale: f32, dir: &vec3_t, out: &mut vec3_t) {
    out[0] = v[0] + scale * dir[0];
    out[1] = v[1] + scale * dir[1];
    out[2] = v[2] + scale * dir[2];
}

pub fn DotProduct(a: &vec3_t, b: &vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn CrossProduct(a: &vec3_t, b: &vec3_t, out: &mut vec3_t) {
    out[0] = a[1] * b[2] - a[2] * b[1];
    out[1] = a[2] * b[0] - a[0] * b[2];
    out[2] = a[0] * b[1] - a[1] * b[0];
}

pub fn VectorLength(v: &vec3_t) -> f32 {
    ((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]) as f64).sqrt() as f32
}

pub fn VectorLengthSquared(v: &vec3_t) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

pub fn VectorNormalize(v: &mut vec3_t) {
    let len = VectorLength(v);
    if len > 0.0 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
}

pub fn Q_fabs(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

// Type stubs for external dependencies
#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct model_t {
    // Stub for model structure
}

#[repr(C)]
pub struct mdxm_t {
    // Stub for mdxm structure
}

#[repr(C)]
pub struct mdxmSurface_t {
    // Stub for surface structure
}

#[repr(C)]
pub struct mdxmSurfHierarchy_t {
    pub name: [c_char; 64],
    pub numChildren: c_int,
    pub childIndexes: [c_int; 0],
}

#[repr(C)]
pub struct mdxmHierarchyOffsets_t {
    pub offsets: [c_int; 0],
}

#[repr(C)]
pub struct mdxmVertex_t {
    pub vertCoords: [f32; 3],
    pub normal: [f32; 3],
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
pub struct mdxmLOD_t {
    pub ofsEnd: c_int,
}

#[repr(C)]
pub struct mdxmLODSurfOffset_t {
    pub offsets: [c_int; 0],
}

#[repr(C)]
pub struct mdxaHeader_t {
    pub numBones: c_int,
    pub ofsFrames: c_int,
}

#[repr(C)]
pub struct mdxaSkel_t {
    pub name: [c_char; 64],
    pub numChildren: c_int,
    pub BasePoseMat: mdxaBone_t,
}

#[repr(C)]
pub struct mdxaSkelOffsets_t {
    pub offsets: [c_int; 0],
}

#[repr(C)]
pub struct mdxmHeader_t {
    pub ofsSurfHierarchy: c_int,
    pub ofsLODs: c_int,
    pub numSurfaces: c_int,
    pub numLODs: c_int,
    pub animIndex: c_int,
    pub animName: [c_char; 64],
}

#[repr(C)]
pub struct skin_t {
    pub name: [c_char; 64],
}

#[repr(C)]
pub struct shader_t {
    pub name: [c_char; 64],
}

#[repr(C)]
pub struct cvar_t {
    pub name: [c_char; 64],
    pub value: f32,
    pub integer: c_int,
}

#[repr(C)]
pub struct surfaceInfo_t {
    pub offFlags: c_int,
}

#[repr(C)]
pub struct boltInfo_t {
    pub unused: c_int,
}

#[repr(C)]
pub struct boneInfo_t {
    pub unused: c_int,
}

#[repr(C)]
pub struct CCollisionRecord {
    pub mPolyIndex: c_int,
    pub mEntityNum: c_int,
    pub mSurfaceIndex: c_int,
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

pub type surfaceInfo_v = Vec<surfaceInfo_t>;
pub type CGhoul2Info_v = Vec<CGhoul2Info>;

#[repr(C)]
pub struct CGhoul2Info {
    pub mValid: bool,
    pub mModelindex: c_int,
    pub mFileName: [c_char; 256],
    pub currentModel: *const model_t,
    pub mLodBias: c_int,
    pub mSkelFrameNum: c_int,
    pub mMeshFrameNum: c_int,
    pub mBoneCache: *mut c_void,
    pub mSlist: Vec<surfaceInfo_t>,
    pub mBlist: Vec<boneInfo_t>,
    pub mBltlist: Vec<boltInfo_t>,
    pub mSurfaceRoot: c_int,
    pub mFlags: c_int,
    pub mCustomShader: c_int,
    pub mSkin: c_int,
    pub mTransformedVertsArray: *mut c_int,
    pub animModel: *const model_t,
    pub aHeader: *const c_void,
    pub BSAVE_START_FIELD: c_int,
    pub BSAVE_END_FIELD: c_int,
}

#[cfg(feature = "G2_GORE")]
#[repr(C)]
pub struct GoreTextureCoordinates {
    pub tex: [*mut f32; 4],
}

#[cfg(feature = "G2_GORE")]
#[repr(C)]
pub struct SGoreSurface {
    pub shader: c_int,
    pub mDeleteTime: c_int,
    pub mFadeTime: f32,
    pub mFadeRGB: [f32; 3],
    pub mGoreTag: c_int,
    pub mGoreGrowStartTime: c_int,
    pub mGoreGrowEndTime: c_int,
    pub mGoreGrowFactor: f32,
    pub mGoreGrowOffset: f32,
}

#[cfg(feature = "G2_GORE")]
#[repr(C)]
pub struct CGoreSet {
    pub mMyGoreSetTag: c_int,
    pub mRefCount: c_int,
    pub mGoreRecords: BTreeMap<c_int, SGoreSurface>,
}

#[cfg(feature = "G2_GORE")]
#[repr(C)]
pub struct SSkinGoreData {
    pub useTheta: bool,
    pub uaxis: vec3_t,
    pub depthStart: f32,
    pub depthEnd: f32,
    pub lifeTime: c_int,
    pub fadeOutTime: f32,
    pub fadeRGB: [f32; 3],
    pub growDuration: c_int,
    pub goreScaleStartFraction: f32,
    pub frontFaces: bool,
    pub backFaces: bool,
    pub firstModel: c_int,
}

// External function declarations
extern "C" {
    pub fn R_GetModelByHandle(handle: c_int) -> *mut model_t;
    pub fn RE_RegisterModel(name: *const c_char) -> c_int;
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Error(error_level: c_int, fmt: *const c_char, ...);
    pub fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn R_GetShaderByHandle(handle: c_int) -> *mut shader_t;
    pub fn R_GetSkinByHandle(handle: c_int) -> *mut skin_t;
    pub fn Z_Malloc(size: usize, tag: c_int, zero: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn SG_Append(tag: u32, data: *mut *mut c_char, size: c_int);
    pub fn SV_GetConfigstring(index: c_int, buffer: *mut c_char, bufsize: usize);
    pub fn SV_SetConfigstring(index: c_int, value: *const c_char);
    pub fn G2_SetupModelPointers(ghoul2: *mut CGhoul2Info);
    pub fn G2_FindSurface(model: *const model_t, index: c_int, lod: c_int) -> *mut c_void;
    pub fn G2_FindOverrideSurface(index: c_int, list: &surfaceInfo_v) -> *const surfaceInfo_t;
    pub fn G2_GetVertWeights(v: *const mdxmVertex_t) -> c_int;
    pub fn G2_GetVertBoneIndex(v: *const mdxmVertex_t, k: c_int) -> c_int;
    pub fn G2_GetVertBoneWeight(v: *const mdxmVertex_t, k: c_int, total_weight: f32, num_weights: c_int) -> f32;
    pub fn EvalBoneCache(index: c_int, bone_cache: *mut c_void) -> *const mdxaBone_t;
    pub fn AnglesToAxis(angles: *const vec3_t, axis: *mut [vec3_t; 3]);
    pub fn TransformPoint(input: &vec3_t, output: &mut vec3_t, mat: *mut mdxaBone_t);
    pub fn TransformAndTranslatePoint(input: &vec3_t, output: &mut vec3_t, mat: *mut mdxaBone_t);
    pub fn G2API_GetTime(arg_time: c_int) -> c_int;
}

// External globals
pub static mut worldMatrix: mdxaBone_t = mdxaBone_t { matrix: [[0.0; 4]; 3] };
pub static mut worldMatrixInv: mdxaBone_t = mdxaBone_t { matrix: [[0.0; 4]; 3] };

// Gore-related globals and statics
#[cfg(feature = "G2_GORE")]
pub static mut CurrentTag: c_int = 256 + 1;

#[cfg(feature = "G2_GORE")]
pub static mut CurrentTagUpper: c_int = 256;

#[cfg(feature = "G2_GORE")]
pub static mut GoreRecords: HashMap<c_int, GoreTextureCoordinates> = HashMap::new();

#[cfg(feature = "G2_GORE")]
pub static mut GoreTagsTemp: HashMap<(c_int, c_int), c_int> = HashMap::new();

#[cfg(feature = "G2_GORE")]
pub static mut goreModelIndex: c_int = 0;

pub static mut cg_g2MarksAllModels: *mut cvar_t = std::ptr::null_mut();

#[cfg(feature = "G2_GORE")]
const GORE_TAG_UPPER: c_int = 256;

#[cfg(feature = "G2_GORE")]
const GORE_TAG_MASK: c_int = !255;

#[cfg(feature = "G2_GORE")]
const MAX_GORE_RECORDS: usize = 500;

#[cfg(feature = "G2_GORE")]
#[repr(C)]
struct SVertexTemp {
    flags: c_int,
    touch: c_int,
    newindex: c_int,
    tex: [f32; 2],
}

#[cfg(feature = "G2_GORE")]
impl SVertexTemp {
    fn new() -> Self {
        SVertexTemp {
            flags: 0,
            touch: 0,
            newindex: 0,
            tex: [0.0; 2],
        }
    }
}

#[cfg(feature = "G2_GORE")]
const MAX_GORE_VERTS: usize = 3000;

#[cfg(feature = "G2_GORE")]
static mut GoreVerts: [SVertexTemp; MAX_GORE_VERTS] = [SVertexTemp { flags: 0, touch: 0, newindex: 0, tex: [0.0; 2] }; MAX_GORE_VERTS];

#[cfg(feature = "G2_GORE")]
static mut GoreIndexCopy: [c_int; MAX_GORE_VERTS] = [0; MAX_GORE_VERTS];

#[cfg(feature = "G2_GORE")]
static mut GoreTouch: c_int = 1;

#[cfg(feature = "G2_GORE")]
const MAX_GORE_INDECIES: usize = 6000;

#[cfg(feature = "G2_GORE")]
static mut GoreIndecies: [c_int; MAX_GORE_INDECIES] = [0; MAX_GORE_INDECIES];

#[cfg(feature = "G2_GORE")]
const GORE_MARGIN: f32 = 0.0;

#[cfg(not(feature = "G2_GORE"))]
#[repr(C)]
struct SVertexTemp {
    flags: c_int,
}

#[cfg(not(feature = "G2_GORE"))]
impl SVertexTemp {
    fn new() -> Self {
        SVertexTemp { flags: 0 }
    }
}

#[cfg(not(feature = "G2_GORE"))]
const MAX_GORE_VERTS: usize = 3000;

#[cfg(not(feature = "G2_GORE"))]
static mut GoreVerts: [SVertexTemp; MAX_GORE_VERTS] = [SVertexTemp { flags: 0 }; MAX_GORE_VERTS];

pub struct CTraceSurface {
    pub surfaceNum: c_int,
    pub rootSList: *mut surfaceInfo_v,
    pub currentModel: *const model_t,
    pub lod: c_int,
    pub rayStart: vec3_t,
    pub rayEnd: vec3_t,
    pub collRecMap: *mut CCollisionRecord,
    pub entNum: c_int,
    pub modelIndex: c_int,
    pub skin: *const skin_t,
    pub cust_shader: *const shader_t,
    pub TransformedVertsArray: *const c_int,
    pub eG2TraceType: c_int,
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

// assorted Ghoul 2 functions.
// list all surfaces associated with a model
pub fn G2_List_Model_Surfaces(fileName: *const c_char) {
    unsafe {
        let mod_m = R_GetModelByHandle(RE_RegisterModel(fileName));
        if mod_m.is_null() {
            return;
        }

        let mdxm = (*mod_m).mdxm;
        if mdxm.is_null() {
            return;
        }

        let surf = (mdxm as *mut u8).offset((*mdxm).ofsSurfHierarchy as isize) as *mut mdxmSurfHierarchy_t;
        let mut surface = ((mdxm as *mut u8).offset(((*mdxm).ofsLODs as isize) + mem::size_of::<mdxmLOD_t>() as isize)) as *mut mdxmSurface_t;

        for x in 0..(*mdxm).numSurfaces {
            Com_Printf(b"Surface %i Name %s\n\0".as_ptr() as *const c_char, x, (*surf).name.as_ptr());

            // This would require accessing r_verbose which is external
            // if r_verbose->value ...
        }
    }
}

// list all bones associated with a model
pub fn G2_List_Model_Bones(fileName: *const c_char, frame: c_int) {
    // Stub implementation
}

// obtain the .gla filename for a model
pub fn G2_GetAnimFileName(fileName: *const c_char, filename: *mut *mut c_char) -> qboolean {
    unsafe {
        let mod_m = R_GetModelByHandle(RE_RegisterModel(fileName));

        if !mod_m.is_null() {
            let mdxm = (*mod_m).mdxm;
            if !mdxm.is_null() && (*mdxm).animName[0] as u8 != 0 {
                *filename = (*mdxm).animName.as_mut_ptr();
                return qtrue;
            }
        }
        qfalse
    }
}

pub fn G2_DecideTraceLod(ghoul2: &CGhoul2Info, useLod: c_int) -> c_int {
    let mut returnLod = useLod;

    if ghoul2.mLodBias > returnLod {
        returnLod = ghoul2.mLodBias;
    }

    if !ghoul2.currentModel.is_null() {
        unsafe {
            let mdxm = (*ghoul2.currentModel).mdxm;
            if !mdxm.is_null() {
                if returnLod >= (*mdxm).numLODs {
                    returnLod = (*mdxm).numLODs - 1;
                }
            }
        }
    }

    returnLod
}

// work out how much space a triangle takes
static fn G2_AreaOfTri(A: &vec3_t, B: &vec3_t, C: &vec3_t) -> f32 {
    let mut ab: vec3_t = [0.0; 3];
    let mut cb: vec3_t = [0.0; 3];
    let mut cross: vec3_t = [0.0; 3];

    VectorSubtract(A, B, &mut ab);
    VectorSubtract(C, B, &mut cb);
    CrossProduct(&ab, &cb, &mut cross);

    VectorLength(&cross)
}

// actually determine the S and T of the coordinate we hit in a given poly
static fn G2_BuildHitPointST(
    A: &vec3_t, SA: f32, TA: f32,
    B: &vec3_t, SB: f32, TB: f32,
    C: &vec3_t, SC: f32, TC: f32,
    P: &vec3_t, s: &mut f32, t: &mut f32, bary_i: &mut f32, bary_j: &mut f32
) {
    let areaABC = G2_AreaOfTri(A, B, C);

    let i = G2_AreaOfTri(P, B, C) / areaABC;
    *bary_i = i;
    let j = G2_AreaOfTri(A, P, C) / areaABC;
    *bary_j = j;
    let k = G2_AreaOfTri(A, B, P) / areaABC;

    *s = SA * i + SB * j + SC * k;
    *t = TA * i + TB * j + TC * k;

    *s = s.fmod(1.0);
    if *s < 0.0 {
        *s += 1.0;
    }

    *t = t.fmod(1.0);
    if *t < 0.0 {
        *t += 1.0;
    }
}

// routine that works out given a ray whether or not it hits a poly
static fn G2_SegmentTriangleTest(
    start: &vec3_t, end: &vec3_t,
    A: &vec3_t, B: &vec3_t, C: &vec3_t,
    backFaces: qboolean, frontFaces: qboolean,
    returnedPoint: &mut vec3_t, returnedNormal: &mut vec3_t, denom: &mut f32
) -> qboolean {
    const TINY: f32 = 1E-10;
    let mut returnedNormalT: vec3_t = [0.0; 3];
    let mut edgeAC: vec3_t = [0.0; 3];

    VectorSubtract(C, A, &mut edgeAC);
    VectorSubtract(B, A, &mut returnedNormalT);
    CrossProduct(&returnedNormalT, &edgeAC, returnedNormal);

    let mut ray: vec3_t = [0.0; 3];
    VectorSubtract(end, start, &mut ray);

    *denom = DotProduct(&ray, returnedNormal);

    if Q_fabs(*denom) < TINY ||
       (backFaces == 0 && *denom > 0.0) ||
       (frontFaces == 0 && *denom < 0.0) {
        return qfalse;
    }

    let mut toPlane: vec3_t = [0.0; 3];
    VectorSubtract(A, start, &mut toPlane);

    let t = DotProduct(&toPlane, returnedNormal) / *denom;

    if t < 0.0 || t > 1.0 {
        return qfalse;
    }

    let mut ray_scaled: vec3_t = [0.0; 3];
    VectorScale(&ray, t, &mut ray_scaled);
    VectorAdd(&ray_scaled, start, returnedPoint);

    let mut edgePA: vec3_t = [0.0; 3];
    VectorSubtract(A, returnedPoint, &mut edgePA);

    let mut edgePB: vec3_t = [0.0; 3];
    VectorSubtract(B, returnedPoint, &mut edgePB);

    let mut edgePC: vec3_t = [0.0; 3];
    VectorSubtract(C, returnedPoint, &mut edgePC);

    let mut temp: vec3_t = [0.0; 3];

    CrossProduct(&edgePA, &edgePB, &mut temp);
    if DotProduct(&temp, returnedNormal) < 0.0 {
        return qfalse;
    }

    CrossProduct(&edgePC, &edgePA, &mut temp);
    if DotProduct(&temp, returnedNormal) < 0.0 {
        return qfalse;
    }

    CrossProduct(&edgePB, &edgePC, &mut temp);
    if DotProduct(&temp, returnedNormal) < 0.0 {
        return qfalse;
    }

    qtrue
}

// now we're at poly level, check each model space transformed poly against the model world transformed ray
static fn G2_TracePolys(
    surface: *const mdxmSurface_t,
    surfInfo: *const mdxmSurfHierarchy_t,
    TS: &mut CTraceSurface
) -> bool {
    unsafe {
        let tris = (surface as *mut u8).offset((*surface).ofsTriangles as isize) as *const mdxmTriangle_t;
        let verts = *(TS.TransformedVertsArray as *const *const f32);
        let numTris = (*surface).numTriangles;

        for j in 0..numTris {
            let mut hitPoint: vec3_t = [0.0; 3];
            let mut normal: vec3_t = [0.0; 3];
            let mut face = 0.0;

            let point1 = &verts[(((*tris).indexes[0] * 5) as usize) as usize..];
            let point2 = &verts[(((*tris).indexes[1] * 5) as usize) as usize..];
            let point3 = &verts[(((*tris).indexes[2] * 5) as usize) as usize..];

            let p1: [f32; 3] = [point1[0], point1[1], point1[2]];
            let p2: [f32; 3] = [point2[0], point2[1], point2[2]];
            let p3: [f32; 3] = [point3[0], point3[1], point3[2]];

            if G2_SegmentTriangleTest(&TS.rayStart, &TS.rayEnd, &p1, &p2, &p3, qtrue, qtrue, &mut hitPoint, &mut normal, &mut face) != 0 {
                for i in 0..MAX_G2_COLLISIONS {
                    if (*TS.collRecMap.add(i)).mEntityNum == -1 {
                        let newCol = &mut *TS.collRecMap.add(i);
                        let mut distVect: vec3_t = [0.0; 3];
                        let mut x_pos = 0.0;
                        let mut y_pos = 0.0;

                        newCol.mPolyIndex = j as c_int;
                        newCol.mEntityNum = TS.entNum;
                        newCol.mSurfaceIndex = (*surface).thisSurfaceIndex;
                        newCol.mModelIndex = TS.modelIndex;

                        if face > 0.0 {
                            newCol.mFlags = G2_FRONTFACE;
                        } else {
                            newCol.mFlags = G2_BACKFACE;
                        }

                        VectorSubtract(&hitPoint, &TS.rayStart, &mut distVect);
                        newCol.mDistance = VectorLength(&distVect);

                        TransformAndTranslatePoint(&hitPoint, &mut newCol.mCollisionPosition, addr_of_mut!(worldMatrix));
                        TransformPoint(&normal, &mut newCol.mCollisionNormal, addr_of_mut!(worldMatrix));
                        VectorNormalize(&mut newCol.mCollisionNormal);

                        newCol.mMaterial = 0;
                        newCol.mLocation = 0;

                        G2_BuildHitPointST(&p1, point1[3], point1[4],
                                         &p2, point2[3], point2[4],
                                         &p3, point3[3], point3[4],
                                         &hitPoint, &mut x_pos, &mut y_pos, &mut newCol.mBarycentricI, &mut newCol.mBarycentricJ);

                        if TS.eG2TraceType == G2_RETURNONHIT {
                            TS.hitOne = true;
                            return true;
                        }

                        break;
                    }
                }
                if i == MAX_G2_COLLISIONS {
                    TS.hitOne = true;
                    return true;
                }
            }
        }
        false
    }
}

// now we're at poly level, check each model space transformed poly against the model world transformed ray
static fn G2_RadiusTracePolys(
    surface: *const mdxmSurface_t,
    TS: &mut CTraceSurface
) -> bool {
    unsafe {
        let mut basis1: vec3_t = [0.0; 3];
        let mut basis2: vec3_t = [0.0, 0.0, 1.0];
        let mut taxis: vec3_t = [0.0; 3];
        let mut saxis: vec3_t = [0.0; 3];

        let mut v3RayDir: vec3_t = [0.0; 3];
        VectorSubtract(&TS.rayEnd, &TS.rayStart, &mut v3RayDir);

        CrossProduct(&v3RayDir, &basis2, &mut basis1);

        if DotProduct(&basis1, &basis1) < 0.1 {
            basis2 = [0.0, 1.0, 0.0];
            CrossProduct(&v3RayDir, &basis2, &mut basis1);
        }

        CrossProduct(&v3RayDir, &basis1, &mut basis2);

        VectorNormalize(&mut basis1);
        VectorNormalize(&mut basis2);

        let c = 0.0_f32.cos();
        let s = 0.0_f32.sin();

        VectorScale(&basis1, 0.5 * c / TS.m_fRadius, &mut taxis);
        VectorMA(&taxis, 0.5 * s / TS.m_fRadius, &basis2, &mut taxis);

        VectorScale(&basis1, -0.5 * s / TS.m_fRadius, &mut saxis);
        VectorMA(&saxis, 0.5 * c / TS.m_fRadius, &basis2, &mut saxis);

        let verts = *(TS.TransformedVertsArray as *const *const f32);
        let numVerts = (*surface).numVerts;

        let mut flags = 63;
        let f = VectorLengthSquared(&v3RayDir);
        v3RayDir[0] /= f;
        v3RayDir[1] /= f;
        v3RayDir[2] /= f;

        for j in 0..numVerts {
            let pos = j * 5;
            let mut delta: vec3_t = [0.0; 3];
            delta[0] = verts[pos] - TS.rayStart[0];
            delta[1] = verts[pos + 1] - TS.rayStart[1];
            delta[2] = verts[pos + 2] - TS.rayStart[2];

            let s = DotProduct(&delta, &saxis) + 0.5;
            let t = DotProduct(&delta, &taxis) + 0.5;
            let u = DotProduct(&delta, &v3RayDir);
            let mut vflags = 0;

            if s > 0.0 { vflags |= 1; }
            if s < 1.0 { vflags |= 2; }
            if t > 0.0 { vflags |= 4; }
            if t < 1.0 { vflags |= 8; }
            if u > 0.0 { vflags |= 16; }
            if u < 1.0 { vflags |= 32; }

            vflags = !vflags;
            flags &= vflags;
            GoreVerts[j as usize].flags = vflags;
        }

        if flags != 0 {
            return false;
        }

        let numTris = (*surface).numTriangles;
        let tris = (surface as *mut u8).offset((*surface).ofsTriangles as isize) as *const mdxmTriangle_t;

        for j in 0..numTris {
            let flags = 63 & GoreVerts[(*tris.add(j as usize)).indexes[0] as usize].flags &
                            GoreVerts[(*tris.add(j as usize)).indexes[1] as usize].flags &
                            GoreVerts[(*tris.add(j as usize)).indexes[2] as usize].flags;

            if flags != 0 {
                continue;
            }

            for i in 0..MAX_G2_COLLISIONS {
                if (*TS.collRecMap.add(i)).mEntityNum == -1 {
                    let newCol = &mut *TS.collRecMap.add(i);

                    newCol.mPolyIndex = j as c_int;
                    newCol.mEntityNum = TS.entNum;
                    newCol.mSurfaceIndex = (*surface).thisSurfaceIndex;
                    newCol.mModelIndex = TS.modelIndex;
                    newCol.mFlags = G2_FRONTFACE;

                    let A = &verts[((*tris.add(j as usize)).indexes[0] as usize * 5)..];
                    let B = &verts[((*tris.add(j as usize)).indexes[1] as usize * 5)..];
                    let C = &verts[((*tris.add(j as usize)).indexes[2] as usize * 5)..];

                    let a: [f32; 3] = [A[0], A[1], A[2]];
                    let b: [f32; 3] = [B[0], B[1], B[2]];
                    let c: [f32; 3] = [C[0], C[1], C[2]];

                    let mut edgeAC: vec3_t = [0.0; 3];
                    let mut edgeBA: vec3_t = [0.0; 3];
                    let mut normal: vec3_t = [0.0; 3];

                    VectorSubtract(&c, &a, &mut edgeAC);
                    VectorSubtract(&b, &a, &mut edgeBA);
                    CrossProduct(&edgeBA, &edgeAC, &mut normal);

                    TransformPoint(&normal, &mut newCol.mCollisionNormal, addr_of_mut!(worldMatrix));
                    VectorNormalize(&mut newCol.mCollisionNormal);

                    newCol.mMaterial = 0;
                    newCol.mLocation = 0;

                    if TS.eG2TraceType == G2_RETURNONHIT {
                        TS.hitOne = true;
                        return true;
                    }

                    let mut distVect: vec3_t = [0.0; 3];
                    let mut hitPoint: vec3_t = [0.0; 3];
                    let side = normal[0] * TS.rayStart[0] + normal[1] * TS.rayStart[1] + normal[2] * TS.rayStart[2] +
                               (-(A[0] * (B[1] * C[2] - C[1] * B[2]) + B[0] * (C[1] * A[2] - A[1] * C[2]) + C[0] * (A[1] * B[2] - B[1] * A[2])));

                    VectorSubtract(&TS.rayEnd, &TS.rayStart, &mut distVect);
                    let side2 = normal[0] * distVect[0] + normal[1] * distVect[1] + normal[2] * distVect[2];

                    let dist;
                    if fabsf(side2) < 1E-8 {
                        VectorSubtract(&a, &TS.rayStart, &mut distVect);
                        dist = VectorLength(&distVect);
                        VectorSubtract(&TS.rayEnd, &TS.rayStart, &mut distVect);
                        VectorMA(&TS.rayStart, dist / VectorLength(&distVect), &distVect, &mut hitPoint);
                    } else {
                        dist = side / side2;
                        VectorMA(&TS.rayStart, -dist, &distVect, &mut hitPoint);
                    }

                    VectorSubtract(&hitPoint, &TS.rayStart, &mut distVect);
                    newCol.mDistance = VectorLength(&distVect);

                    TransformAndTranslatePoint(&hitPoint, &mut newCol.mCollisionPosition, addr_of_mut!(worldMatrix));
                    newCol.mBarycentricI = 0.0;
                    newCol.mBarycentricJ = 0.0;

                    break;
                }
            }
            if i == MAX_G2_COLLISIONS {
                TS.hitOne = true;
                return true;
            }
        }
        false
    }
}

// look at a surface and then do the trace on each poly
static fn G2_TraceSurfaces(TS: &mut CTraceSurface) {
    unsafe {
        let surface = G2_FindSurface(TS.currentModel, TS.surfaceNum, TS.lod) as *const mdxmSurface_t;

        if surface.is_null() {
            return;
        }

        // Stub: access to surfInfo and other complex logic would go here

        if TS.hitOne {
            return;
        }
    }
}

pub fn TransformPoint(r#in: &vec3_t, out: &mut vec3_t, mat: *mut mdxaBone_t) {
    unsafe {
        for i in 0..3 {
            out[i] = r#in[0] * (*mat).matrix[i][0] + r#in[1] * (*mat).matrix[i][1] + r#in[2] * (*mat).matrix[i][2];
        }
    }
}

pub fn TransformAndTranslatePoint(r#in: &vec3_t, out: &mut vec3_t, mat: *mut mdxaBone_t) {
    unsafe {
        for i in 0..3 {
            out[i] = r#in[0] * (*mat).matrix[i][0] + r#in[1] * (*mat).matrix[i][1] + r#in[2] * (*mat).matrix[i][2] + (*mat).matrix[i][3];
        }
    }
}

// create a matrix using a set of angles
pub fn Create_Matrix(angle: &[f32; 3], matrix: *mut mdxaBone_t) {
    unsafe {
        let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];

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
pub fn G2_GenerateWorldMatrix(angles: &vec3_t, origin: &vec3_t) {
    unsafe {
        Create_Matrix(&(angles.clone() as [f32; 3]), addr_of_mut!(worldMatrix));
        worldMatrix.matrix[0][3] = origin[0];
        worldMatrix.matrix[1][3] = origin[1];
        worldMatrix.matrix[2][3] = origin[2];

        Inverse_Matrix(addr_of_mut!(worldMatrix), addr_of_mut!(worldMatrixInv));
    }
}

// go away and determine what the pointer for a specific surface definition within the model definition is
pub fn G2_FindSurfacePtr(mod_: *const model_t, index: c_int, lod: c_int) -> *mut c_void {
    unsafe {
        if mod_.is_null() {
            return std::ptr::null_mut();
        }

        let mdxm = (*mod_).mdxm;
        if mdxm.is_null() {
            return std::ptr::null_mut();
        }

        let mut current = (mdxm as *mut u8).offset((*mdxm).ofsLODs as isize);

        for i in 0..lod {
            let lodData = current as *mut mdxmLOD_t;
            current = current.offset((*lodData).ofsEnd as isize);
        }

        current = current.offset(mem::size_of::<mdxmLOD_t>() as isize);

        let indexes = current as *mut mdxmLODSurfOffset_t;
        current = current.offset(*(&(*indexes).offsets as *const c_int).offset(index as isize) as isize);

        current as *mut c_void
    }
}

const SURFACE_SAVE_BLOCK_SIZE: usize = mem::size_of::<surfaceInfo_t>();
const BOLT_SAVE_BLOCK_SIZE: usize = mem::size_of::<boltInfo_t>();
const BONE_SAVE_BLOCK_SIZE: usize = mem::size_of::<boneInfo_t>();

pub fn G2_SaveGhoul2Models(ghoul2: &CGhoul2Info_v) {
    unsafe {
        if !ghoul2.is_empty() {
            let ghoul2BlockSize = mem::size_of::<CGhoul2Info>(); // Stub: actual size calculation would differ

            let mut iGhoul2Size = 4;
            for i in 0..ghoul2.len() {
                iGhoul2Size += ghoul2BlockSize;
                iGhoul2Size += 4;
                iGhoul2Size += ghoul2[i].mSlist.len() * SURFACE_SAVE_BLOCK_SIZE;
                iGhoul2Size += 4;
                iGhoul2Size += ghoul2[i].mBlist.len() * BONE_SAVE_BLOCK_SIZE;
                iGhoul2Size += 4;
                iGhoul2Size += ghoul2[i].mBltlist.len() * BOLT_SAVE_BLOCK_SIZE;
            }

            let pGhoul2Data = Z_Malloc(iGhoul2Size, TAG_GHOUL2, qfalse);
            let mut tempBuffer = pGhoul2Data as *mut c_int;

            *tempBuffer = ghoul2.len() as c_int;
            tempBuffer = tempBuffer.offset(1);

            // Further implementation would copy data structures

            Z_Free(pGhoul2Data);
        }
    }
}

pub fn G2_FindConfigStringSpace(name: *const c_char, start: c_int, max: c_int) -> c_int {
    unsafe {
        let mut s: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

        for i in 1..max {
            SV_GetConfigstring(start + i, s.as_mut_ptr(), mem::size_of_val(&s));
            if s[0] as u8 == 0 {
                break;
            }
            // stricmp would be called here
        }

        // SV_SetConfigstring would be called here
        0 // Stub return
    }
}

pub fn G2_LoadGhoul2Model(ghoul2: &mut CGhoul2Info_v, buffer: *mut c_char) {
    unsafe {
        let newSize = *(buffer as *const c_int);
        ghoul2.resize(newSize as usize, Default::default());

        if newSize == 0 {
            return;
        }

        let ghoul2BlockSize = mem::size_of::<c_int>();
        let mut buf_ptr = buffer.offset(4);

        for i in 0..ghoul2.len() {
            ghoul2[i].mSkelFrameNum = 0;
            ghoul2[i].mModelindex = -1;

            // memcpy would copy structure data
            buf_ptr = buf_ptr.offset(ghoul2BlockSize as isize);

            if ghoul2[i].mModelindex != -1 {
                G2_SetupModelPointers(&mut ghoul2[i]);
            }

            let slist_size = *(buf_ptr as *const c_int);
            buf_ptr = buf_ptr.offset(4);
            ghoul2[i].mSlist.resize(slist_size as usize, Default::default());
            buf_ptr = buf_ptr.offset((slist_size as usize * SURFACE_SAVE_BLOCK_SIZE) as isize);

            let blist_size = *(buf_ptr as *const c_int);
            buf_ptr = buf_ptr.offset(4);
            ghoul2[i].mBlist.resize(blist_size as usize, Default::default());
            buf_ptr = buf_ptr.offset((blist_size as usize * BONE_SAVE_BLOCK_SIZE) as isize);

            let bltlist_size = *(buf_ptr as *const c_int);
            buf_ptr = buf_ptr.offset(4);
            ghoul2[i].mBltlist.resize(bltlist_size as usize, Default::default());
            buf_ptr = buf_ptr.offset((bltlist_size as usize * BOLT_SAVE_BLOCK_SIZE) as isize);
        }
    }
}

#[cfg(not(feature = "XBOX"))]
pub fn R_TransformEachSurface(
    surface: *const mdxmSurface_t,
    scale: &vec3_t,
    G2VertSpace: *mut c_void,
    TransformedVertsArray: *mut c_int,
    boneCache: *mut c_void
) {
    unsafe {
        let piBoneReferences = (surface as *mut u8).offset((*surface).ofsBoneReferences as isize) as *mut c_int;

        let TransformedVerts = G2VertSpace; // Stub: actual allocation would happen
        *TransformedVertsArray.offset((*surface).thisSurfaceIndex as isize) = TransformedVerts as c_int;

        let numVerts = (*surface).numVerts;
        let v = (surface as *mut u8).offset((*surface).ofsVerts as isize) as *mut mdxmVertex_t;
        let pTexCoords = v.offset(numVerts as isize) as *mut mdxmVertexTexCoord_t;

        if (scale[0] - 1.0).abs() > f32::EPSILON || (scale[1] - 1.0).abs() > f32::EPSILON || (scale[2] - 1.0).abs() > f32::EPSILON {
            // Scaled case - stub
        } else {
            // Unscaled case - stub
        }
    }
}

pub fn G2_TransformSurfaces(
    surfaceNum: c_int,
    rootSList: &mut surfaceInfo_v,
    boneCache: *mut c_void,
    currentModel: *const model_t,
    lod: c_int,
    scale: &vec3_t,
    G2VertSpace: *mut c_void,
    TransformedVertArray: *mut c_int,
    secondTimeAround: bool
) {
    unsafe {
        let surface = G2_FindSurface(currentModel, surfaceNum, lod) as *const mdxmSurface_t;

        if surface.is_null() {
            return;
        }

        // Stub: complex surface hierarchy navigation
        // R_TransformEachSurface(surface, scale, G2VertSpace, TransformedVertArray, boneCache);
    }
}

#[cfg(feature = "G2_GORE")]
pub fn G2_TransformModel(
    ghoul2: &mut CGhoul2Info_v,
    frameNum: c_int,
    scale: &vec3_t,
    G2VertSpace: *mut c_void,
    useLod: c_int,
    ApplyGore: bool,
    gore: *mut SSkinGoreData
) {
    unsafe {
        if cg_g2MarksAllModels.is_null() {
            cg_g2MarksAllModels = Cvar_Get(b"cg_g2MarksAllModels\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        let mut firstModelOnly = true;
        if !cg_g2MarksAllModels.is_null() && (*cg_g2MarksAllModels).integer != 0 {
            firstModelOnly = false;
        }

        if !gore.is_null() && (*gore).firstModel > 0 {
            firstModelOnly = false;
        }

        let mut correctScale: vec3_t = [0.0; 3];
        VectorCopy(scale, &mut correctScale);

        if correctScale[0] == 0.0 { correctScale[0] = 1.0; }
        if correctScale[1] == 0.0 { correctScale[1] = 1.0; }
        if correctScale[2] == 0.0 { correctScale[2] = 1.0; }

        for i in 0..ghoul2.len() {
            if !ghoul2[i].mValid {
                continue;
            }

            ghoul2[i].mMeshFrameNum = frameNum;

            let lod = if ApplyGore {
                useLod
            } else {
                G2_DecideTraceLod(&ghoul2[i], useLod)
            };

            // Stub: mTransformedVertsArray allocation and transformation
            if firstModelOnly {
                break;
            }
        }
    }
}

#[cfg(not(feature = "G2_GORE"))]
pub fn G2_TransformModel(
    ghoul2: &mut CGhoul2Info_v,
    frameNum: c_int,
    scale: &vec3_t,
    G2VertSpace: *mut c_void,
    useLod: c_int
) {
    unsafe {
        if cg_g2MarksAllModels.is_null() {
            cg_g2MarksAllModels = Cvar_Get(b"cg_g2MarksAllModels\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        let mut firstModelOnly = true;
        if !cg_g2MarksAllModels.is_null() && (*cg_g2MarksAllModels).integer != 0 {
            firstModelOnly = false;
        }

        let mut correctScale: vec3_t = [0.0; 3];
        VectorCopy(scale, &mut correctScale);

        if correctScale[0] == 0.0 { correctScale[0] = 1.0; }
        if correctScale[1] == 0.0 { correctScale[1] = 1.0; }
        if correctScale[2] == 0.0 { correctScale[2] = 1.0; }

        for i in 0..ghoul2.len() {
            if !ghoul2[i].mValid {
                continue;
            }

            ghoul2[i].mMeshFrameNum = frameNum;

            let lod = G2_DecideTraceLod(&ghoul2[i], useLod);

            // Stub: mTransformedVertsArray allocation and transformation
            if firstModelOnly {
                break;
            }
        }
    }
}

#[cfg(feature = "G2_GORE")]
pub fn G2_TraceModels(
    ghoul2: &mut CGhoul2Info_v,
    rayStart: &vec3_t,
    rayEnd: &vec3_t,
    collRecMap: *mut CCollisionRecord,
    entNum: c_int,
    eG2TraceType: c_int,
    useLod: c_int,
    fRadius: f32,
    ssize: f32,
    tsize: f32,
    theta: f32,
    shader: c_int,
    gore: *mut SSkinGoreData,
    skipIfLODNotMatch: qboolean
) {
    unsafe {
        if cg_g2MarksAllModels.is_null() {
            cg_g2MarksAllModels = Cvar_Get(b"cg_g2MarksAllModels\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        let mut firstModelOnly = true;
        let mut firstModel = 0;

        if !cg_g2MarksAllModels.is_null() && (*cg_g2MarksAllModels).integer != 0 {
            firstModelOnly = false;
        }

        if !gore.is_null() && (*gore).firstModel > 0 {
            firstModel = (*gore).firstModel;
            firstModelOnly = false;
        }

        for i in firstModel as usize..ghoul2.len() {
            goreModelIndex = i as c_int;

            if ghoul2[i].mModelindex == -1 {
                continue;
            }

            if !ghoul2[i].mValid {
                continue;
            }

            if ghoul2[i].mFlags & GHOUL2_NOCOLLIDE != 0 {
                continue;
            }

            let cust_shader = if ghoul2[i].mCustomShader != 0 {
                R_GetShaderByHandle(ghoul2[i].mCustomShader)
            } else {
                std::ptr::null_mut()
            };

            let skin = if ghoul2[i].mSkin > 0 {
                R_GetSkinByHandle(ghoul2[i].mSkin)
            } else {
                std::ptr::null_mut()
            };

            let lod = G2_DecideTraceLod(&ghoul2[i], useLod);

            if skipIfLODNotMatch != 0 {
                if lod != useLod {
                    continue;
                }
            }

            // Stub: CTraceSurface construction and G2_TraceSurfaces call

            if firstModelOnly {
                break;
            }
        }
    }
}

#[cfg(not(feature = "G2_GORE"))]
pub fn G2_TraceModels(
    ghoul2: &mut CGhoul2Info_v,
    rayStart: &vec3_t,
    rayEnd: &vec3_t,
    collRecMap: *mut CCollisionRecord,
    entNum: c_int,
    eG2TraceType: c_int,
    useLod: c_int,
    fRadius: f32
) {
    unsafe {
        if cg_g2MarksAllModels.is_null() {
            cg_g2MarksAllModels = Cvar_Get(b"cg_g2MarksAllModels\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }

        let mut firstModelOnly = true;
        if !cg_g2MarksAllModels.is_null() && (*cg_g2MarksAllModels).integer != 0 {
            firstModelOnly = false;
        }

        for i in 0..ghoul2.len() {
            if !ghoul2[i].mValid {
                continue;
            }

            if ghoul2[i].mFlags & GHOUL2_NOCOLLIDE != 0 {
                continue;
            }

            let cust_shader = if ghoul2[i].mCustomShader != 0 {
                R_GetShaderByHandle(ghoul2[i].mCustomShader)
            } else {
                std::ptr::null_mut()
            };

            let skin = if ghoul2[i].mSkin > 0 {
                R_GetSkinByHandle(ghoul2[i].mSkin)
            } else {
                std::ptr::null_mut()
            };

            let lod = G2_DecideTraceLod(&ghoul2[i], useLod);

            // Stub: CTraceSurface construction and G2_TraceSurfaces call

            if firstModelOnly {
                break;
            }
        }
    }
}

#[cfg(feature = "G2_GORE")]
#[derive(Clone)]
#[repr(C)]
struct GoreTextureCoordinates {
    tex: [*mut f32; 4],
}

#[cfg(feature = "G2_GORE")]
impl Default for GoreTextureCoordinates {
    fn default() -> Self {
        GoreTextureCoordinates { tex: [std::ptr::null_mut(); 4] }
    }
}

#[cfg(feature = "G2_GORE")]
#[derive(Clone)]
#[repr(C)]
struct CGoreSet {
    mMyGoreSetTag: c_int,
    mRefCount: c_int,
    mGoreRecords: BTreeMap<c_int, SGoreSurface>,
}

#[cfg(feature = "G2_GORE")]
fn FindGoreRecord(tag: c_int) -> Option<*mut GoreTextureCoordinates> {
    unsafe {
        if let Some(value) = GoreRecords.get(&tag) {
            Some(value as *const _ as *mut _)
        } else {
            None
        }
    }
}

#[cfg(feature = "G2_GORE")]
fn AllocGoreRecord() -> c_int {
    unsafe {
        while GoreRecords.len() > MAX_GORE_RECORDS {
            // Tag high from first record
            if let Some((tag, _)) = GoreRecords.iter().next() {
                let tagHigh = tag & GORE_TAG_MASK;

                // Remove records with same tag high
                let tags_to_remove: Vec<_> = GoreRecords.iter()
                    .filter(|(k, _)| (k & GORE_TAG_MASK) == tagHigh)
                    .map(|(k, _)| *k)
                    .collect();

                for tag_to_remove in tags_to_remove {
                    GoreRecords.remove(&tag_to_remove);
                }
            } else {
                break;
            }
        }

        let ret = CurrentTag;
        GoreRecords.insert(CurrentTag, Default::default());
        CurrentTag += 1;
        ret
    }
}

#[cfg(feature = "G2_GORE")]
fn DeleteGoreRecord(tag: c_int) {
    unsafe {
        GoreRecords.remove(&tag);
    }
}

#[cfg(feature = "G2_GORE")]
fn ResetGoreTag() {
    unsafe {
        GoreTagsTemp.clear();
        CurrentTag = CurrentTagUpper;
        CurrentTagUpper += GORE_TAG_UPPER;
    }
}

#[cfg(feature = "G2_GORE")]
fn NewGoreSet() -> *mut CGoreSet {
    unsafe {
        let goreSet = Box::new(CGoreSet {
            mMyGoreSetTag: CurrentGoreSet,
            mRefCount: 1,
            mGoreRecords: BTreeMap::new(),
        });
        CurrentGoreSet += 1;
        Box::into_raw(goreSet)
    }
}

#[cfg(feature = "G2_GORE")]
fn FindGoreSet(goreSetTag: c_int) -> *mut CGoreSet {
    unsafe {
        // Stub: actual map lookup
        std::ptr::null_mut()
    }
}

#[cfg(feature = "G2_GORE")]
fn DeleteGoreSet(goreSetTag: c_int) {
    // Stub: actual deletion logic
}

#[cfg(feature = "G2_GORE")]
static mut CurrentGoreSet: c_int = 1;

#[cfg(feature = "G2_GORE")]
pub fn G2_GetGoreRecord(tag: c_int) -> *mut c_void {
    if let Some(rec) = FindGoreRecord(tag) {
        rec as *mut c_void
    } else {
        std::ptr::null_mut()
    }
}
