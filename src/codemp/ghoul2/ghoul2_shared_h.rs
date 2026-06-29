#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::ptr;

// Imported types from mdx_format.h (locally defined to avoid module dependency)
#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct mdxaHeader_t {
    pub ident: c_int,
    pub version: c_int,
    pub name: [u8; 260],
    pub fScale: f32,
    pub numFrames: c_int,
    pub ofsFrames: c_int,
    pub numBones: c_int,
    pub ofsCompBonePool: c_int,
    pub ofsSkel: c_int,
    pub ofsEnd: c_int,
}

// Forward declarations
pub enum model_s {}

// rww - RAGDOLL_BEGIN
pub const G2T_SV_TIME: c_int = 0;
pub const G2T_CG_TIME: c_int = 1;
pub const NUM_G2T_TIME: c_int = 2;

// rww - RAGDOLL_END

extern "C" {
    pub fn G2API_SetTime(currentTime: c_int, clock: c_int);
    pub fn G2API_GetTime(argTime: c_int) -> c_int; // this may or may not return arg depending on ghoul2_time cvar
}

// ===================================================================
//
//   G H O U L  I I  D E F I N E S
//
// we save the whole surfaceInfo_t struct
#[repr(C)]
pub struct surfaceInfo_t {
    pub offFlags: c_int,           // what the flags are for this model
    pub surface: c_int,            // index into array held inside the model definition of pointers to the actual surface data loaded in - used by both client and game
    pub genBarycentricJ: f32,      // point 0 barycentric coors
    pub genBarycentricI: f32,      // point 1 barycentric coors - point 2 is 1 - point0 - point1
    pub genPolySurfaceIndex: c_int, // used to point back to the original surface and poly if this is a generated surface
    pub genLod: c_int,             // used to determine original lod of original surface and poly hit location
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

pub const MDXABONEDEF: () = (); // used in the mdxformat.h file to stop redefinitions of the bone struct.

// we save the whole structure here.
#[repr(C)]
pub struct boneInfo_t {
    pub boneNumber: c_int,        // what bone are we overriding?
    pub matrix: mdxaBone_t,       // details of bone angle overrides - some are pre-done on the server, some in ghoul2
    pub flags: c_int,             // flags for override
    pub startFrame: c_int,        // start frame for animation
    pub endFrame: c_int,          // end frame for animation NOTE anim actually ends on endFrame+1
    pub startTime: c_int,         // time we started this animation
    pub pauseTime: c_int,         // time we paused this animation - 0 if not paused
    pub animSpeed: f32,           // speed at which this anim runs. 1.0f means full speed of animation incoming - ie if anim is 20hrtz, we run at 20hrts. If 5hrts, we run at 5 hrts
    pub blendFrame: f32,          // frame PLUS LERP value to blend from
    pub blendLerpFrame: c_int,    // frame to lerp the blend frame with.
    pub blendTime: c_int,         // Duration time for blending - used to calc amount each frame of new anim is blended with last frame of the last anim
    pub blendStart: c_int,        // Time when blending starts - not necessarily the same as startTime since we might start half way through an anim
    pub boneBlendTime: c_int,     // time for duration of bone angle blend with normal animation
    pub boneBlendStart: c_int,    // time bone angle blend with normal animation began
    pub lastTime: c_int,          // this does not go across the network
    pub newMatrix: mdxaBone_t,    // This is the lerped matrix that Ghoul2 uses on the client side - does not go across the network

    // rww - RAGDOLL_BEGIN
    pub lastTimeUpdated: c_int,   // if non-zero this is all intialized
    pub lastContents: c_int,
    pub lastPosition: [f32; 3],   // vec3_t
    pub velocityEffector: [f32; 3], // vec3_t
    pub lastAngles: [f32; 3],     // vec3_t
    pub minAngles: [f32; 3],      // vec3_t
    pub maxAngles: [f32; 3],      // vec3_t
    pub currentAngles: [f32; 3],  // vec3_t
    pub anglesOffset: [f32; 3],   // vec3_t
    pub positionOffset: [f32; 3], // vec3_t
    pub radius: f32,
    pub weight: f32,              // current radius cubed
    pub ragIndex: c_int,
    pub velocityRoot: [f32; 3],   // vec3_t // I am really tired of recomiling the whole game to add a param here
    pub ragStartTime: c_int,
    pub firstTime: c_int,
    pub firstCollisionTime: c_int,
    pub restTime: c_int,
    pub RagFlags: c_int,
    pub DependentRagIndexMask: c_int,
    pub originalTrueBoneMatrix: mdxaBone_t,
    pub parentTrueBoneMatrix: mdxaBone_t,  // figure I will need this sooner or later
    pub parentOriginalTrueBoneMatrix: mdxaBone_t, // figure I will need this sooner or later
    pub originalOrigin: [f32; 3], // vec3_t
    pub originalAngles: [f32; 3], // vec3_t
    pub lastShotDir: [f32; 3],    // vec3_t
    pub basepose: *mut mdxaBone_t,
    pub baseposeInv: *mut mdxaBone_t,
    pub baseposeParent: *mut mdxaBone_t,
    pub baseposeInvParent: *mut mdxaBone_t,
    pub parentRawBoneIndex: c_int,
    pub ragOverrideMatrix: mdxaBone_t, // figure I will need this sooner or later

    pub extraMatrix: mdxaBone_t,  // figure I will need this sooner or later
    pub extraVec1: [f32; 3],      // vec3_t // I am really tired of recomiling the whole game to add a param here
    pub extraFloat1: f32,
    pub extraInt1: c_int,

    pub ikPosition: [f32; 3],     // vec3_t
    pub ikSpeed: f32,

    pub epVelocity: [f32; 3],     // vec3_t //velocity factor, can be set, and is also maintained by physics based on gravity, mass, etc.
    pub epGravFactor: f32,        //gravity factor maintained by bone physics
    pub solidCount: c_int,        //incremented every time we try to move and are in solid - if we get out of solid, it is reset to 0
    pub physicsSettled: bool,     //true when the bone is on ground and finished bouncing, etc. but may still be pushed into solid by other bones
    pub snapped: bool,            //the bone is broken out of standard constraints

    pub parentBoneIndex: c_int,

    pub offsetRotation: f32,

    //user api overrides
    pub overGradSpeed: f32,

    pub overGoalSpot: [f32; 3],   // vec3_t
    pub hasOverGoal: bool,

    pub animFrameMatrix: mdxaBone_t, //matrix for the bone in the desired settling pose -rww
    pub hasAnimFrameMatrix: c_int,

    pub airTime: c_int,           //base is in air, be more quick and sensitive about collisions
    // rww - RAGDOLL_END
}

impl boneInfo_t {
    pub fn new() -> Self {
        let mut bi = boneInfo_t {
            boneNumber: -1,
            matrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            flags: 0,
            startFrame: 0,
            endFrame: 0,
            startTime: 0,
            pauseTime: 0,
            animSpeed: 0.0,
            blendFrame: 0.0,
            blendLerpFrame: 0,
            blendTime: 0,
            blendStart: 0,
            boneBlendTime: 0,
            boneBlendStart: 0,
            lastTime: 0,
            newMatrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            lastTimeUpdated: 0,
            lastContents: 0,
            lastPosition: [0.0; 3],
            velocityEffector: [0.0; 3],
            lastAngles: [0.0; 3],
            minAngles: [0.0; 3],
            maxAngles: [0.0; 3],
            currentAngles: [0.0; 3],
            anglesOffset: [0.0; 3],
            positionOffset: [0.0; 3],
            radius: 0.0,
            weight: 0.0,
            ragIndex: 0,
            velocityRoot: [0.0; 3],
            ragStartTime: 0,
            firstTime: 0,
            firstCollisionTime: 0,
            restTime: 0,
            RagFlags: 0,
            DependentRagIndexMask: 0,
            originalTrueBoneMatrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            parentTrueBoneMatrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            parentOriginalTrueBoneMatrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            originalOrigin: [0.0; 3],
            originalAngles: [0.0; 3],
            lastShotDir: [0.0; 3],
            basepose: ptr::null_mut(),
            baseposeInv: ptr::null_mut(),
            baseposeParent: ptr::null_mut(),
            baseposeInvParent: ptr::null_mut(),
            parentRawBoneIndex: 0,
            ragOverrideMatrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            extraMatrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            extraVec1: [0.0; 3],
            extraFloat1: 0.0,
            extraInt1: 0,
            ikPosition: [0.0; 3],
            ikSpeed: 0.0,
            epVelocity: [0.0; 3],
            epGravFactor: 0.0,
            solidCount: 0,
            physicsSettled: false,
            snapped: false,
            parentBoneIndex: 0,
            offsetRotation: 0.0,
            overGradSpeed: 0.0,
            overGoalSpot: [0.0; 3],
            hasOverGoal: false,
            animFrameMatrix: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
            hasAnimFrameMatrix: 0,
            airTime: 0,
        };
        bi
    }
}

// we save from top to boltUsed here. Don't bother saving the position, it gets rebuilt every frame anyway
#[repr(C)]
pub struct boltInfo_t {
    pub boneNumber: c_int,        // bone number bolt attaches to
    pub surfaceNumber: c_int,     // surface number bolt attaches to
    pub surfaceType: c_int,       // if we attach to a surface, this tells us if it is an original surface or a generated one - doesn't go across the network
    pub boltUsed: c_int,          // nor does this
    pub position: mdxaBone_t,     // this does not go across the network
}

impl boltInfo_t {
    pub fn new() -> Self {
        boltInfo_t {
            boneNumber: -1,
            surfaceNumber: -1,
            surfaceType: 0,
            boltUsed: 0,
            position: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
        }
    }
}

// #ifdef _SOF2
#[repr(C)]
pub enum goreEnum_t {
    PGORE_NONE = 0,
    PGORE_ARMOR = 1,
    PGORE_BULLETSMALL = 2,
    PGORE_BULLETMED = 3,
    PGORE_BULLETBIG = 4,
    PGORE_HEGRENADE = 5,
    PGORE_COUNT = 6,
}

pub const MAX_QPATH: usize = 260;

#[repr(C)]
pub struct goreEnumShader_t {
    pub shaderEnum: goreEnum_t,
    pub shaderName: [c_char; MAX_QPATH],
}

#[repr(C)]
pub struct SSkinGoreData {
    pub angles: [f32; 3],         // vec3_t
    pub position: [f32; 3],       // vec3_t
    pub currentTime: c_int,
    pub entNum: c_int,
    pub rayDirection: [f32; 3],   // vec3_t in world space
    pub hitLocation: [f32; 3],    // vec3_t in world space
    pub scale: [f32; 3],          // vec3_t
    pub SSize: f32,               // size of splotch in the S texture direction in world units
    pub TSize: f32,               // size of splotch in the T texture direction in world units
    pub theta: f32,               // angle to rotate the splotch

    // qhandle_t		shader;			// handle to shader for gore, this better be rendered after the shader of the underlying surface
    //                              // this shader should also have "clamp" mode, not tiled.
    pub shaderEnum: goreEnum_t,   // enum that'll get switched over to the shader's actual handle
}
// #endif // _SOF2

pub const MAX_GHOUL_COUNT_BITS: c_int = 8; // bits required to send across the MAX_G2_MODELS inside of the networking - this is the only restriction on ghoul models possible per entity

// NOTE: The following typedef'd vectors are C++ STL containers in the original.
// In Rust, these would be represented as Vec types, but for faithful porting they are
// kept as opaque type aliases. Runtime usage will involve dynamic collections.
// C++ STL container types - represented as opaque in Rust to maintain ABI compatibility
pub enum surfaceInfo_v {}
pub enum boneInfo_v {}
pub enum boltInfo_v {}
pub enum mdxaBone_v {}

// defines for stuff to go into the mflags
pub const GHOUL2_NOCOLLIDE: c_int = 0x001;
pub const GHOUL2_NORENDER: c_int = 0x002;
pub const GHOUL2_NOMODEL: c_int = 0x004;
pub const GHOUL2_NEWORIGIN: c_int = 0x008;

// for transform optimization -rww
pub const GHOUL2_ZONETRANSALLOC: c_int = 0x2000;

// Forward declaration for CBoneCache
pub enum CBoneCache {}

pub type qhandle_t = c_int;

// NOTE order in here matters. We save out from mModelindex to mFlags, but not the STL vectors that are at the top or the bottom.
#[repr(C)]
pub struct CGhoul2Info {
    pub mSlist: *mut surfaceInfo_v,
    pub mBltlist: *mut boltInfo_v,
    pub mBlist: *mut boneInfo_v,
    // save from here
    pub mModelindex: c_int,
    pub mCustomShader: qhandle_t,
    pub mCustomSkin: qhandle_t,
    pub mModelBoltLink: c_int,
    pub mSurfaceRoot: c_int,
    pub mLodBias: c_int,
    pub mNewOrigin: c_int,        // this contains the bolt index of the new origin for this model
    // #ifdef _G2_GORE
    pub mGoreSetTag: c_int,
    // #endif
    pub mModel: qhandle_t,        // this and the next entries do NOT go across the network. They are for gameside access ONLY
    pub mFileName: [c_char; MAX_QPATH],
    pub mAnimFrameDefault: c_int,
    pub mSkelFrameNum: c_int,
    pub mMeshFrameNum: c_int,
    pub mFlags: c_int,            // used for determining whether to do full collision detection against this object
    // to here
    pub mTransformedVertsArray: *mut c_int, // used to create an array of pointers to transformed verts per surface for collision detection
    pub mBoneCache: *mut CBoneCache,
    pub mSkin: c_int,

    // these occasionally are not valid (like after a vid_restart)
    // call the questionably efficient G2_SetupModelPointers(this) to insure validity
    pub mValid: bool,             // all the below are proper and valid
    pub currentModel: *const model_s,
    pub currentModelSize: c_int,
    pub animModel: *const model_s,
    pub currentAnimModelSize: c_int,
    pub aHeader: *const mdxaHeader_t,

    // #ifdef _G2_LISTEN_SERVER_OPT
    pub entityNum: c_int,
    // #endif
}

pub const ENTITYNUM_NONE: c_int = -1;

impl CGhoul2Info {
    pub fn new() -> Self {
        CGhoul2Info {
            mSlist: ptr::null_mut(),
            mBltlist: ptr::null_mut(),
            mBlist: ptr::null_mut(),
            mModelindex: -1,
            mCustomShader: 0,
            mCustomSkin: 0,
            mModelBoltLink: 0,
            mModel: 0,
            mSurfaceRoot: 0,
            mAnimFrameDefault: 0,
            mSkelFrameNum: -1,
            mMeshFrameNum: -1,
            mFlags: 0,
            mTransformedVertsArray: ptr::null_mut(),
            mLodBias: 0,
            mSkin: 0,
            mNewOrigin: -1,
            // #ifdef _G2_GORE
            mGoreSetTag: 0,
            // #endif
            mBoneCache: ptr::null_mut(),
            currentModel: ptr::null(),
            currentModelSize: 0,
            animModel: ptr::null(),
            currentAnimModelSize: 0,
            aHeader: ptr::null(),
            // #ifdef _G2_LISTEN_SERVER_OPT
            entityNum: ENTITYNUM_NONE,
            // #endif
            mValid: false,
        }
    }
}

// Forward declaration for CGhoul2Info_v
#[repr(C)]
pub struct CGhoul2Info_v {
    pub mItem: c_int, // dont' be bad and muck with this
}

// Virtual base class equivalent - abstract C++ interface kept as opaque reference
pub enum IGhoul2InfoArray {}

extern "C" {
    pub fn TheGhoul2InfoArray() -> *mut IGhoul2InfoArray;
}

impl CGhoul2Info_v {
    fn InfoArray(&self) -> *mut IGhoul2InfoArray {
        unsafe { TheGhoul2InfoArray() }
    }

    // Stub for Alloc - in the original, asserts that mItem is 0 and allocates
    // Original: void Alloc() { assert(!mItem); mItem=InfoArray().New(); assert(!Array().size()); }
    #[allow(non_snake_case)]
    fn Alloc(&mut self) {
        // assert that mItem is not already allocated
        // mItem = InfoArray().New()
        // assert that the resulting array is empty
    }

    // Stub for Free - in the original, cleans up if mItem is set
    // Original: void Free() { if (mItem) { assert(InfoArray().IsValid(mItem)); InfoArray().Delete(mItem); mItem=0; } }
    #[allow(non_snake_case)]
    fn Free(&mut self) {
        if self.mItem != 0 {
            // assert that mItem is valid in InfoArray
            // InfoArray().Delete(mItem)
            self.mItem = 0;
        }
    }

    pub fn new() -> Self {
        CGhoul2Info_v { mItem: 0 }
    }

    pub fn from_item(item: c_int) -> Self {
        // be VERY carefull with what you pass in here
        CGhoul2Info_v { mItem: item }
    }

    pub fn operator_assign_v(&mut self, other: &CGhoul2Info_v) {
        self.mItem = other.mItem;
    }

    pub fn operator_assign_i(&mut self, otherItem: c_int) {
        // assigning one from the VM side item number
        self.mItem = otherItem;
    }

    pub fn DeepCopy(&mut self, other: &CGhoul2Info_v) {
        self.Free();
        if other.mItem != 0 {
            self.Alloc();
            // Copy the array from other and zero out certain fields
            // Original C++:
            // Array()=other.Array();
            // for (i=0;i<size();i++)
            // {
            //     Array()[i].mBoneCache=0;
            //     Array()[i].mTransformedVertsArray=0;
            //     Array()[i].mSkelFrameNum=0;
            //     Array()[i].mMeshFrameNum=0;
            // }
        }
    }

    pub fn resize(&mut self, num: c_int) {
        assert!(num >= 0);
        if num > 0 {
            if self.mItem == 0 {
                self.Alloc();
            }
        }
        if self.mItem != 0 || num > 0 {
            // Array().resize(num as usize, CGhoul2Info::new())
        }
    }

    pub fn clear(&mut self) {
        self.Free();
    }

    pub fn push_back(&mut self, _model: &CGhoul2Info) {
        if self.mItem == 0 {
            self.Alloc();
        }
        // Array().push_back(model)
    }

    pub fn size(&self) -> c_int {
        if !self.IsValid() {
            return 0;
        }
        // return Array().size() as c_int
        0
    }

    pub fn IsValid(&self) -> bool {
        // InfoArray().IsValid(self.mItem)
        false
    }

    pub fn kill(&mut self) {
        // this scary method zeros the infovector handle without actually freeing it
        // it is used for some places where a copy is made, but we don't want to go through the trouble
        // of making a deep copy
        self.mItem = 0;
    }
}

impl Clone for CGhoul2Info {
    fn clone(&self) -> Self {
        CGhoul2Info {
            mSlist: self.mSlist,
            mBltlist: self.mBltlist,
            mBlist: self.mBlist,
            mModelindex: self.mModelindex,
            mCustomShader: self.mCustomShader,
            mCustomSkin: self.mCustomSkin,
            mModelBoltLink: self.mModelBoltLink,
            mSurfaceRoot: self.mSurfaceRoot,
            mLodBias: self.mLodBias,
            mNewOrigin: self.mNewOrigin,
            mGoreSetTag: self.mGoreSetTag,
            mModel: self.mModel,
            mFileName: self.mFileName,
            mAnimFrameDefault: self.mAnimFrameDefault,
            mSkelFrameNum: self.mSkelFrameNum,
            mMeshFrameNum: self.mMeshFrameNum,
            mFlags: self.mFlags,
            mTransformedVertsArray: self.mTransformedVertsArray,
            mBoneCache: self.mBoneCache,
            mSkin: self.mSkin,
            mValid: self.mValid,
            currentModel: self.currentModel,
            currentModelSize: self.currentModelSize,
            animModel: self.animModel,
            currentAnimModelSize: self.currentAnimModelSize,
            aHeader: self.aHeader,
            entityNum: self.entityNum,
        }
    }
}

impl Drop for CGhoul2Info_v {
    fn drop(&mut self) {
        self.Free();
    }
}

// collision detection stuff
pub const G2_FRONTFACE: c_int = 1;
pub const G2_BACKFACE: c_int = 0;

// calling defines for the trace function
#[repr(C)]
pub enum EG2_Collision {
    G2_NOCOLLIDE = 0,
    G2_COLLIDE = 1,
    G2_RETURNONHIT = 2,
}
