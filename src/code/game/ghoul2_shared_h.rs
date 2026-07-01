#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(improper_ctypes)]

/*
Ghoul2 Insert Start
*/
// #pragma warning (push, 3)   //go back down to 3 for the stl include
// #pragma warning (disable:4503)   // decorated name length xceeded, name was truncated
// #pragma warning(disable:4702)    //unreachable code
// <vector> and <map> are C++ STL — not imported in Rust
// using namespace std; — not applicable in Rust
/*
Ghoul2 Insert End
*/

// #include "../renderer/mdx_format.h"
use crate::code::renderer::mdx_format_h::*;
use core::ffi::{c_int, c_float, c_char};

pub const G2T_SV_TIME: c_int = 0;
pub const G2T_CG_TIME: c_int = 1;
pub const NUM_G2T_TIME: c_int = 2;

extern "C" {
    pub fn G2API_SetTime(currentTime: c_int, clock: c_int);
    pub fn G2API_GetTime(argTime: c_int) -> c_int; // this may or may not return arg depending on ghoul2_time cvar
}


//===================================================================
//
//   G H O U L  I I  D E F I N E S
//
// we save the whole surfaceInfo_t struct
#[repr(C)]
pub struct surfaceInfo_t
{
    pub offFlags: c_int,            // what the flags are for this model
    pub surface: c_int,             // index into array held inside the model definition of pointers to the actual surface data loaded in - used by both client and game
    pub genBarycentricJ: c_float,   // point 0 barycentric coors
    pub genBarycentricI: c_float,   // point 1 barycentric coors - point 2 is 1 - point0 - point1
    pub genPolySurfaceIndex: c_int, // used to point back to the original surface and poly if this is a generated surface
    pub genLod: c_int,              // used to determine original lod of original surface and poly hit location
}

impl surfaceInfo_t {
    pub fn new() -> Self {
        surfaceInfo_t {
            offFlags: 0,
            surface: 0,
            genBarycentricJ: 0.0,
            genBarycentricI: 0.0,
            genPolySurfaceIndex: 0,
            genLod: 0,
        }
    }
}

pub const BONE_ANGLES_PREMULT: c_int            = 0x0001;
pub const BONE_ANGLES_POSTMULT: c_int           = 0x0002;
pub const BONE_ANGLES_REPLACE: c_int            = 0x0004;
//rww - RAGDOLL_BEGIN
pub const BONE_ANGLES_RAGDOLL: c_int            = 0x2000;  // the rag flags give more details
pub const BONE_ANGLES_IK: c_int                 = 0x4000;  // the rag flags give more details
//rww - RAGDOLL_END
pub const BONE_ANGLES_TOTAL: c_int              = BONE_ANGLES_PREMULT | BONE_ANGLES_POSTMULT | BONE_ANGLES_REPLACE;

pub const BONE_ANIM_OVERRIDE: c_int             = 0x0008;
pub const BONE_ANIM_OVERRIDE_LOOP: c_int        = 0x0010;  // Causes Last Frame To Lerp to First Frame And Start Over
pub const BONE_ANIM_OVERRIDE_FREEZE: c_int      = 0x0040 + BONE_ANIM_OVERRIDE; // Causes Last Frame To Freeze And Not Loop To Beginning
pub const BONE_ANIM_BLEND: c_int                = 0x0080;  // Blends to and from previously played frame on same bone for given time
pub const BONE_ANIM_NO_LERP: c_int              = 0x1000;
pub const BONE_ANIM_TOTAL: c_int                = BONE_ANIM_NO_LERP | BONE_ANIM_OVERRIDE | BONE_ANIM_OVERRIDE_LOOP | BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND;

pub const BONE_INDEX_INVALID: c_int             = -1;

/* #define MDXABONEDEF              // used in the mdxformat.h file to stop redefinitions of the bone struct.

typedef struct {
	float		matrix[3][4];
} mdxaBone_t;
*/
// #include "../renderer/mdx_format.h"  (glob-imported above)

// we save the whole structure here.
#[repr(C)]
pub struct boneInfo_t
{
    pub boneNumber: c_int,          // what bone are we overriding?
    pub matrix: mdxaBone_t,         // details of bone angle overrides - some are pre-done on the server, some in ghoul2
    pub flags: c_int,               // flags for override
    pub startFrame: c_int,          // start frame for animation
    pub endFrame: c_int,            // end frame for animation NOTE anim actually ends on endFrame+1
    pub startTime: c_int,           // time we started this animation
    pub pauseTime: c_int,           // time we paused this animation - 0 if not paused
    pub animSpeed: c_float,         // speed at which this anim runs. 1.0f means full speed of animation incoming - ie if anim is 20hrtz, we run at 20hrts. If 5hrts, we run at 5 hrts
    pub blendFrame: c_float,        // frame PLUS LERP value to blend from
    pub blendLerpFrame: c_int,      // frame to lerp the blend frame with.
    pub blendTime: c_int,           // Duration time for blending - used to calc amount each frame of new anim is blended with last frame of the last anim
    pub blendStart: c_int,          // Time when blending starts - not necessarily the same as startTime since we might start half way through an anim
    pub boneBlendTime: c_int,       // time for duration of bone angle blend with normal animation
    pub boneBlendStart: c_int,      // time bone angle blend with normal animation began
    pub newMatrix: mdxaBone_t,      // This is the lerped matrix that Ghoul2 uses on the client side - does not go across the network

    //rww - RAGDOLL_BEGIN
    pub lastTimeUpdated: c_int,     // if non-zero this is all intialized
    pub lastContents: c_int,
    pub lastPosition: vec3_t,
    pub velocityEffector: vec3_t,
    pub lastAngles: vec3_t,
    pub minAngles: vec3_t,
    pub maxAngles: vec3_t,
    pub currentAngles: vec3_t,
    pub anglesOffset: vec3_t,
    pub positionOffset: vec3_t,
    pub radius: c_float,
    pub weight: c_float,            // current radius cubed
    pub ragIndex: c_int,
    pub velocityRoot: vec3_t,       // I am really tired of recomiling the whole game to add a param here
    pub ragStartTime: c_int,
    pub firstTime: c_int,
    pub firstCollisionTime: c_int,
    pub restTime: c_int,
    pub RagFlags: c_int,
    pub DependentRagIndexMask: c_int,
    pub originalTrueBoneMatrix: mdxaBone_t,
    pub parentTrueBoneMatrix: mdxaBone_t,           // figure I will need this sooner or later
    pub parentOriginalTrueBoneMatrix: mdxaBone_t,   // figure I will need this sooner or later
    pub originalOrigin: vec3_t,
    pub originalAngles: vec3_t,
    pub lastShotDir: vec3_t,
    pub basepose: *mut mdxaBone_t,
    pub baseposeInv: *mut mdxaBone_t,
    pub baseposeParent: *mut mdxaBone_t,
    pub baseposeInvParent: *mut mdxaBone_t,
    pub parentRawBoneIndex: c_int,
    pub ragOverrideMatrix: mdxaBone_t,  // figure I will need this sooner or later

    pub extraMatrix: mdxaBone_t,    // figure I will need this sooner or later
    pub extraVec1: vec3_t,          // I am really tired of recomiling the whole game to add a param here
    pub extraFloat1: c_float,
    pub extraInt1: c_int,

    pub ikPosition: vec3_t,
    pub ikSpeed: c_float,

    //new ragdoll stuff -rww
    pub epVelocity: vec3_t,     // velocity factor, can be set, and is also maintained by physics based on gravity, mass, etc.
    pub epGravFactor: c_float,  // gravity factor maintained by bone physics
    pub solidCount: c_int,      // incremented every time we try to move and are in solid - if we get out of solid, it is reset to 0
    pub physicsSettled: bool,   // true when the bone is on ground and finished bouncing, etc. but may still be pushed into solid by other bones
    pub snapped: bool,          // the bone is broken out of standard constraints

    pub parentBoneIndex: c_int,

    pub offsetRotation: c_float,

    //user api overrides
    pub overGradSpeed: c_float,

    pub overGoalSpot: vec3_t,
    pub hasOverGoal: bool,

    pub animFrameMatrix: mdxaBone_t, // matrix for the bone in the desired settling pose -rww
    pub hasAnimFrameMatrix: c_int,

    pub airTime: c_int, // base is in air, be more quick and sensitive about collisions
    //rww - RAGDOLL_END
}

impl boneInfo_t {
    pub fn new() -> Self {
        // C++ constructor initializes only the listed members; ragdoll fields left uninitialised.
        // Rust requires full initialisation; remaining fields are zero-filled.
        let mut s: boneInfo_t = unsafe { core::mem::zeroed() };
        s.boneNumber    = -1;
        s.flags         = 0;
        s.startFrame    = 0;
        s.endFrame      = 0;
        s.startTime     = 0;
        s.pauseTime     = 0;
        s.animSpeed     = 0.0;
        s.blendFrame    = 0.0;
        s.blendLerpFrame = 0;
        s.blendTime     = 0;
        s.blendStart    = 0;
        s.boneBlendTime  = 0;
        s.boneBlendStart = 0;
        s.matrix.matrix[0][0] = 0.0; s.matrix.matrix[0][1] = 0.0; s.matrix.matrix[0][2] = 0.0; s.matrix.matrix[0][3] = 0.0;
        s.matrix.matrix[1][0] = 0.0; s.matrix.matrix[1][1] = 0.0; s.matrix.matrix[1][2] = 0.0; s.matrix.matrix[1][3] = 0.0;
        s.matrix.matrix[2][0] = 0.0; s.matrix.matrix[2][1] = 0.0; s.matrix.matrix[2][2] = 0.0; s.matrix.matrix[2][3] = 0.0;
        s
    }
}

//we save from top to boltUsed here. Don't bother saving the position, it gets rebuilt every frame anyway
#[repr(C)]
pub struct boltInfo_t {
    pub boneNumber: c_int,      // bone number bolt attaches to
    pub surfaceNumber: c_int,   // surface number bolt attaches to
    pub surfaceType: c_int,     // if we attach to a surface, this tells us if it is an original surface or a generated one - doesn't go across the network
    pub boltUsed: c_int,        // nor does this
}

impl boltInfo_t {
    pub fn new() -> Self {
        boltInfo_t {
            boneNumber:    -1,
            surfaceNumber: -1,
            surfaceType:    0,
            boltUsed:       0,
        }
    }
}


pub const MAX_GHOUL_COUNT_BITS: c_int = 8; // bits required to send across the MAX_G2_MODELS inside of the networking - this is the only restriction on ghoul models possible per entity

pub type surfaceInfo_v = Vec<surfaceInfo_t>;
pub type boneInfo_v    = Vec<boneInfo_t>;
pub type boltInfo_v    = Vec<boltInfo_t>;
pub type mdxaBone_v    = Vec<mdxaBone_t>;

// defines for stuff to go into the mflags
pub const GHOUL2_NOCOLLIDE: c_int = 0x001;
pub const GHOUL2_NORENDER: c_int  = 0x002;
pub const GHOUL2_NOMODEL: c_int   = 0x004;
pub const GHOUL2_NEWORIGIN: c_int = 0x008;


// NOTE order in here matters. We save out from mModelindex to mFlags, but not the STL vectors that are at the top or the bottom.
// class CBoneCache;        -- forward declaration; trusted from glob import
// struct model_s;          -- forward declaration; trusted from glob import
// //struct mdxaHeader_t;

#[cfg(feature = "VV_GHOUL_HACKS")]
#[repr(C)]
pub struct CRenderableSurface
{
    pub ident: c_int,               // ident of this surface - required so the materials renderer knows what sort of surface this refers to
    pub boneCache: *mut CBoneCache, // pointer to transformed bone list for this surf
    pub surfaceData: *mut mdxmSurface_t, // pointer to surface data loaded into file - only used by client renderer DO NOT USE IN GAME SIDE - if there is a vid restart this will be out of wack on the game
}

#[cfg(feature = "VV_GHOUL_HACKS")]
impl CRenderableSurface {
    pub fn new() -> Self {
        CRenderableSurface {
            ident:       8, //SF_MDX
            boneCache:   core::ptr::null_mut(),
            surfaceData: core::ptr::null_mut(),
        }
    }

    pub fn from_rs(rs: &CRenderableSurface) -> Self {
        CRenderableSurface {
            ident:       rs.ident,
            boneCache:   rs.boneCache,
            surfaceData: rs.surfaceData,
        }
    }
}

#[repr(C)]
pub struct CGhoul2Info
{
    pub mSlist:   Vec<surfaceInfo_t>,
    pub mBltlist: Vec<boltInfo_t>,
    pub mBlist:   Vec<boneInfo_t>,
// save from here (do not put any ptrs etc within this save block unless you adds special handlers to G2_SaveGhoul2Models / G2_LoadGhoul2Models!!!!!!!!!!!!
// #define BSAVE_START_FIELD mModelindex   // this is the start point for loadsave, keep it up to date it you change anything
    pub mModelindex:          c_int,
    pub animModelIndexOffset: c_int,
    pub mCustomShader:        qhandle_t,
    pub mCustomSkin:          qhandle_t,
    pub mModelBoltLink:       c_int,
    pub mSurfaceRoot:         c_int,
    pub mLodBias:             c_int,
    pub mNewOrigin:           c_int,    // this contains the bolt index of the new origin for this model
    #[cfg(feature = "_G2_GORE")]
    pub mGoreSetTag:          c_int,
    pub mModel:               qhandle_t, // this and the next entries do NOT go across the network. They are for gameside access ONLY
    pub mFileName:            [c_char; MAX_QPATH],
    pub mAnimFrameDefault:    c_int,
    pub mSkelFrameNum:        c_int,
    pub mMeshFrameNum:        c_int,
    pub mFlags:               c_int,    // used for determining whether to do full collision detection against this object
// to here
// #define BSAVE_END_FIELD mTransformedVertsArray  // this is the end point for loadsave, keep it up to date it you change anything
    pub mTransformedVertsArray: *mut c_int, // used to create an array of pointers to transformed verts per surface for collision detection
    pub mBoneCache:           *mut CBoneCache,
    pub mSkin:                c_int,

    // these occasionally are not valid (like after a vid_restart)
    // call the questionably efficient G2_SetupModelPointers(this) to insure validity
    pub mValid:               bool,             // all the below are proper and valid
    pub currentModel:         *const model_s,
    pub currentModelSize:     c_int,
    pub animModel:            *const model_s,
    pub currentAnimModelSize: c_int,
    pub aHeader:              *const mdxaHeader_t,
}

impl CGhoul2Info {
    pub fn new() -> Self {
        CGhoul2Info {
            mSlist:                Vec::new(),
            mBltlist:              Vec::new(),
            mBlist:                Vec::new(),
            mModelindex:           -1,
            mCustomShader:          0,
            mCustomSkin:            0,
            mModelBoltLink:         0,
            mModel:                 0,
            mSurfaceRoot:           0,
            mAnimFrameDefault:      0,
            mSkelFrameNum:         -1,
            mMeshFrameNum:         -1,
            mFlags:                 0,
            mTransformedVertsArray: core::ptr::null_mut(),
            mLodBias:               0,
            mSkin:                  0,
            mNewOrigin:            -1,
            #[cfg(feature = "_G2_GORE")]
            mGoreSetTag:            0,
            mBoneCache:             core::ptr::null_mut(),
            currentModel:           core::ptr::null(),
            currentModelSize:       0,
            animModel:              core::ptr::null(),
            animModelIndexOffset:   0,
            currentAnimModelSize:   0,
            aHeader:                core::ptr::null(),
            mValid:                 false,
            mFileName:              [0; MAX_QPATH],
        }
    }
}

// class CGhoul2Info_v;  -- forward declaration (defined below)

pub trait IGhoul2InfoArray
{
    fn New(&mut self) -> c_int;
    fn Delete(&mut self, handle: c_int);
    fn IsValid(&self, handle: c_int) -> bool;
    // C++ has two overloads of Get (const and non-const); const overload renamed GetConst.
    fn Get(&mut self, handle: c_int) -> &mut Vec<CGhoul2Info>;
    fn GetConst(&self, handle: c_int) -> &Vec<CGhoul2Info>;
}

// C++ returns IGhoul2InfoArray& (reference to abstract class); represented as raw trait-object pointer.
// Not ABI-safe but preserves type intent for no-compile faithful port.
extern "C" {
    pub fn TheGhoul2InfoArray() -> *mut dyn IGhoul2InfoArray;
    pub fn TheGameGhoul2InfoArray() -> *mut dyn IGhoul2InfoArray;
}

pub struct CGhoul2Info_v
{
    mItem: c_int,
}

impl CGhoul2Info_v
{
    fn InfoArray(&self) -> *mut dyn IGhoul2InfoArray
    {
        #[cfg(feature = "_JK2EXE")]
        { unsafe { TheGhoul2InfoArray() } }
        #[cfg(not(feature = "_JK2EXE"))]
        { unsafe { TheGameGhoul2InfoArray() } }
    }

    fn Alloc(&mut self)
    {
        assert!(self.mItem == 0); //already alloced
        self.mItem = unsafe { (*self.InfoArray()).New() };
        assert!(self.Array().is_empty());
    }

    fn Free(&mut self)
    {
        if self.mItem != 0
        {
            assert!(unsafe { (*self.InfoArray()).IsValid(self.mItem) });
            unsafe { (*self.InfoArray()).Delete(self.mItem); }
            self.mItem = 0;
        }
    }

    fn Array(&mut self) -> &mut Vec<CGhoul2Info>
    {
        assert!(unsafe { (*self.InfoArray()).IsValid(self.mItem) });
        unsafe { (*self.InfoArray()).Get(self.mItem) }
    }

    fn Array_const(&self) -> &Vec<CGhoul2Info>
    {
        assert!(unsafe { (*self.InfoArray()).IsValid(self.mItem) });
        unsafe { (*self.InfoArray()).GetConst(self.mItem) }
    }

    pub fn new() -> Self
    {
        CGhoul2Info_v {
            mItem: 0,
        }
    }

    // C++ operator=(const CGhoul2Info_v &other) — shallow copy of handle only
    pub fn assign(&mut self, other: &CGhoul2Info_v)
    {
        self.mItem = other.mItem;
    }

    pub fn DeepCopy(&mut self, other: &CGhoul2Info_v)
    {
        self.Free();
        if other.mItem != 0
        {
            self.Alloc();
            let other_arr = other.Array_const().to_vec();
            *self.Array() = other_arr;
            let sz = self.size() as usize;
            {
                let arr = self.Array();
                for i in 0..sz
                {
                    arr[i].mBoneCache              = core::ptr::null_mut();
                    arr[i].mTransformedVertsArray  = core::ptr::null_mut();
                    arr[i].mSkelFrameNum           = 0;
                    arr[i].mMeshFrameNum           = 0;
                }
            }
        }
    }

    // C++ operator[](int idx)
    pub fn get_mut(&mut self, idx: c_int) -> &mut CGhoul2Info
    {
        assert!(self.mItem != 0);
        assert!(idx >= 0 && idx < self.size());
        &mut self.Array()[idx as usize]
    }

    // C++ operator[](int idx) const
    pub fn get(&self, idx: c_int) -> &CGhoul2Info
    {
        assert!(self.mItem != 0);
        assert!(idx >= 0 && idx < self.size());
        &self.Array_const()[idx as usize]
    }

    pub fn resize(&mut self, num: c_int)
    {
        assert!(num >= 0);
        if num != 0
        {
            if self.mItem == 0
            {
                self.Alloc();
            }
        }
        if self.mItem != 0 || num != 0
        {
            self.Array().resize(num as usize, CGhoul2Info::new());
        }
    }

    pub fn clear(&mut self)
    {
        self.Free();
    }

    pub fn push_back(&mut self, model: CGhoul2Info)
    {
        if self.mItem == 0
        {
            self.Alloc();
        }
        self.Array().push(model);
    }

    pub fn size(&self) -> c_int
    {
        if !self.IsValid()
        {
            return 0;
        }
        self.Array_const().len() as c_int
    }

    pub fn IsValid(&self) -> bool
    {
        unsafe { (*self.InfoArray()).IsValid(self.mItem) }
    }

    pub fn kill(&mut self)
    {
        // this scary method zeros the infovector handle without actually freeing it
        // it is used for some places where a copy is made, but we don't want to go through the trouble
        // of making a deep copy
        self.mItem = 0;
    }
}

impl core::ops::Index<c_int> for CGhoul2Info_v {
    type Output = CGhoul2Info;
    fn index(&self, idx: c_int) -> &CGhoul2Info {
        self.get(idx)
    }
}

impl core::ops::IndexMut<c_int> for CGhoul2Info_v {
    fn index_mut(&mut self, idx: c_int) -> &mut CGhoul2Info {
        self.get_mut(idx)
    }
}

impl Drop for CGhoul2Info_v {
    fn drop(&mut self) {
        self.Free(); //this had better be taken care of via the clean ghoul2 models call
    }
}



// collision detection stuff
pub const G2_FRONTFACE: c_int = 1;
pub const G2_BACKFACE: c_int  = 0;


#[repr(C)]
pub struct CCollisionRecord
{
    pub mDistance:          c_float,
    pub mEntityNum:         c_int,
    pub mModelIndex:        c_int,
    pub mPolyIndex:         c_int,
    pub mSurfaceIndex:      c_int,
    pub mCollisionPosition: vec3_t,
    pub mCollisionNormal:   vec3_t,
    pub mFlags:             c_int,
    pub mMaterial:          c_int,
    pub mLocation:          c_int,
    pub mBarycentricI:      c_float, // two barycentic coodinates for the hit point
    pub mBarycentricJ:      c_float, // K = 1-I-J
}

impl CCollisionRecord {
    pub fn new() -> Self {
        CCollisionRecord {
            mEntityNum:         -1,
            mDistance:          100000.0,
            mModelIndex:        0,
            mPolyIndex:         0,
            mSurfaceIndex:      0,
            mCollisionPosition: [0.0; 3],
            mCollisionNormal:   [0.0; 3],
            mFlags:             0,
            mMaterial:          0,
            mLocation:          0,
            mBarycentricI:      0.0,
            mBarycentricJ:      0.0,
        }
    }
}

// calling defines for the trace function
#[repr(C)]
pub enum EG2_Collision
{
    G2_NOCOLLIDE,
    G2_COLLIDE,
    G2_RETURNONHIT,
}



//====================================================================
