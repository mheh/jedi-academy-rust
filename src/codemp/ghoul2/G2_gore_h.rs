#![allow(non_snake_case)]

use core::ffi::c_int;

// #if !defined(G2_GORE_H_INC)
// #define G2_GORE_H_INC

// #ifdef _G2_GORE

const MAX_LODS: usize = 8;

#[repr(C)]
pub struct GoreTextureCoordinates {
    pub tex: [*mut f32; MAX_LODS],
}

impl GoreTextureCoordinates {
    pub fn new() -> Self {
        GoreTextureCoordinates {
            tex: [std::ptr::null_mut(); MAX_LODS],
        }
    }
}

impl Drop for GoreTextureCoordinates {
    fn drop(&mut self) {
        let mut i: usize;
        for i in 0..MAX_LODS {
            if !self.tex[i].is_null() {
                // #ifdef _DEBUG
                // extern int g_goreTexAllocs;
                // g_goreTexAllocs--;
                // #endif
                unsafe {
                    Z_Free(self.tex[i] as *mut core::ffi::c_void);
                    self.tex[i] = std::ptr::null_mut();
                }
            }
        }
    }
}

extern "C" {
    pub fn AllocGoreRecord() -> c_int;
    pub fn FindGoreRecord(tag: c_int) -> *mut GoreTextureCoordinates;
    pub fn DeleteGoreRecord(tag: c_int);
}

#[repr(C)]
pub struct SGoreSurface {
    pub shader: c_int,
    pub mGoreTag: c_int,
    pub mDeleteTime: c_int,
    pub mFadeTime: c_int,
    pub mFadeRGB: bool,

    pub mGoreGrowStartTime: c_int,
    pub mGoreGrowEndTime: c_int,    // set this to -1 to disable growing
    //curscale = (curtime-mGoreGrowStartTime)*mGoreGrowFactor + mGoreGrowOffset;
    pub mGoreGrowFactor: f32,
    pub mGoreGrowOffset: f32,
}

// class CGoreSet
// {
// public:
//     int		 mMyGoreSetTag;
//     unsigned char mRefCount;
//     multimap<int,SGoreSurface> mGoreRecords; // a map from surface index
//     CGoreSet(int tag) : mMyGoreSetTag(tag), mRefCount(0) {}
//     ~CGoreSet();
// };
pub struct CGoreSet {
    pub mMyGoreSetTag: c_int,
    pub mRefCount: u8,
    // multimap<int,SGoreSurface> mGoreRecords; // a map from surface index
    // NOTE: multimap is a C++ STL container; skipping Rust translation of this field
    // as it requires external dependency or complex porting. Declaration kept for reference.
}

extern "C" {
    pub fn FindGoreSet(goreSetTag: c_int) -> *mut CGoreSet;
    pub fn NewGoreSet() -> *mut CGoreSet;
    pub fn DeleteGoreSet(goreSetTag: c_int);
}

// #endif // _G2_GORE

//rww - RAGDOLL_BEGIN

/// ragdoll stuff

// #pragma warning(disable: 4512)

#[repr(C)]
pub struct SRagDollEffectorCollision {
    pub effectorPosition: [f32; 3],  // vec3_t
    pub tr: *const trace_t,          // const trace_t &tr
    pub useTracePlane: bool,
}

// Note: The C++ constructor logic is not directly representable in Rust.
// In C++: SRagDollEffectorCollision(const vec3_t effectorPos,const trace_t &t) :
//     tr(t),
//     useTracePlane(false)
// {
//     VectorCopy(effectorPos,effectorPosition);
// }

impl SRagDollEffectorCollision {
    pub fn new(effectorPos: [f32; 3], t: *const trace_t) -> Self {
        SRagDollEffectorCollision {
            effectorPosition: effectorPos,
            tr: t,
            useTracePlane: false,
        }
    }
}

// Stub for trace_t as it's defined elsewhere
#[repr(C)]
pub struct trace_t {
    // Placeholder: full definition would be in another header
}

// class CRagDollUpdateParams
// {
// public:
//     vec3_t angles;
//     vec3_t position;
//     vec3_t scale;
//     vec3_t velocity;
//     //CServerEntity *me;
//     int	me; //index!
//     int settleFrame;
//
//     //at some point I'll want to make VM callbacks in here. For now I am just doing nothing.
//     virtual void EffectorCollision(const SRagDollEffectorCollision &data)
//     {
//     //	assert(0); // you probably meant to override this
//     }
//     virtual void RagDollBegin()
//     {
//     //	assert(0); // you probably meant to override this
//     }
//     virtual void RagDollSettled()
//     {
//     //	assert(0); // you probably meant to override this
//     }
//
//     virtual void Collision()
//     {
//     //	assert(0); // you probably meant to override this
//         // we had a collision, uhh I guess call SetRagDoll RP_DEATH_COLLISION
//     }
//
// #ifdef _DEBUG
//     virtual void DebugLine(const vec3_t p1,const vec3_t p2,bool bbox) {assert(0);}
// #endif
// };

pub struct CRagDollUpdateParams {
    pub angles: [f32; 3],
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub velocity: [f32; 3],
    //CServerEntity *me;
    pub me: c_int, //index!
    pub settleFrame: c_int,

    //at some point I'll want to make VM callbacks in here. For now I am just doing nothing.
}

impl CRagDollUpdateParams {
    //virtual void EffectorCollision(const SRagDollEffectorCollision &data)
    //{
    ////	assert(0); // you probably meant to override this
    //}
    pub fn EffectorCollision(&mut self, data: &SRagDollEffectorCollision) {
        //	assert(0); // you probably meant to override this
    }

    //virtual void RagDollBegin()
    //{
    ////	assert(0); // you probably meant to override this
    //}
    pub fn RagDollBegin(&mut self) {
        //	assert(0); // you probably meant to override this
    }

    //virtual void RagDollSettled()
    //{
    ////	assert(0); // you probably meant to override this
    //}
    pub fn RagDollSettled(&mut self) {
        //	assert(0); // you probably meant to override this
    }

    //virtual void Collision()
    //{
    ////	assert(0); // you probably meant to override this
        // we had a collision, uhh I guess call SetRagDoll RP_DEATH_COLLISION
    //}
    pub fn Collision(&mut self) {
        //	assert(0); // you probably meant to override this
        // we had a collision, uhh I guess call SetRagDoll RP_DEATH_COLLISION
    }

    // #ifdef _DEBUG
    //virtual void DebugLine(const vec3_t p1,const vec3_t p2,bool bbox) {assert(0);}
    // #endif
}


// enum ERagPhase (nested in C++ class CRagDollParams)
pub mod CRagDollParams_ERagPhase {
    use core::ffi::c_int;

    pub const RP_START_DEATH_ANIM: c_int = 0;
    pub const RP_END_DEATH_ANIM: c_int = 1;
    pub const RP_DEATH_COLLISION: c_int = 2;
    pub const RP_CORPSE_SHOT: c_int = 3;
    pub const RP_GET_PELVIS_OFFSET: c_int = 4;  // this actually does nothing but set the pelvisAnglesOffset, and pelvisPositionOffset
    pub const RP_SET_PELVIS_OFFSET: c_int = 5;  // this actually does nothing but set the pelvisAnglesOffset, and pelvisPositionOffset
    pub const RP_DISABLE_EFFECTORS: c_int = 6; // this removes effectors given by the effectorsToTurnOff member
}

// enum ERagEffector (nested in C++ class CRagDollParams)
pub mod CRagDollParams_ERagEffector {
    use core::ffi::c_int;

    pub const RE_MODEL_ROOT: c_int =        0x00000001; //"model_root"
    pub const RE_PELVIS: c_int =            0x00000002; //"pelvis"
    pub const RE_LOWER_LUMBAR: c_int =      0x00000004; //"lower_lumbar"
    pub const RE_UPPER_LUMBAR: c_int =      0x00000008; //"upper_lumbar"
    pub const RE_THORACIC: c_int =          0x00000010; //"thoracic"
    pub const RE_CRANIUM: c_int =           0x00000020; //"cranium"
    pub const RE_RHUMEROUS: c_int =         0x00000040; //"rhumerus"
    pub const RE_LHUMEROUS: c_int =         0x00000080; //"lhumerus"
    pub const RE_RRADIUS: c_int =           0x00000100; //"rradius"
    pub const RE_LRADIUS: c_int =           0x00000200; //"lradius"
    pub const RE_RFEMURYZ: c_int =          0x00000400; //"rfemurYZ"
    pub const RE_LFEMURYZ: c_int =          0x00000800; //"lfemurYZ"
    pub const RE_RTIBIA: c_int =            0x00001000; //"rtibia"
    pub const RE_LTIBIA: c_int =            0x00002000; //"ltibia"
    pub const RE_RHAND: c_int =             0x00004000; //"rhand"
    pub const RE_LHAND: c_int =             0x00008000; //"lhand"
    pub const RE_RTARSAL: c_int =           0x00010000; //"rtarsal"
    pub const RE_LTARSAL: c_int =           0x00020000; //"ltarsal"
    pub const RE_RTALUS: c_int =            0x00040000; //"rtalus"
    pub const RE_LTALUS: c_int =            0x00080000; //"ltalus"
    pub const RE_RRADIUSX: c_int =          0x00100000; //"rradiusX"
    pub const RE_LRADIUSX: c_int =          0x00200000; //"lradiusX"
    pub const RE_RFEMURX: c_int =           0x00400000; //"rfemurX"
    pub const RE_LFEMURX: c_int =           0x00800000; //"lfemurX"
    pub const RE_CEYEBROW: c_int =          0x01000000; //"ceyebrow"
}

pub struct CRagDollParams {
    pub angles: [f32; 3],
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub pelvisAnglesOffset: [f32; 3],    // always set on return, an argument for RP_SET_PELVIS_OFFSET
    pub pelvisPositionOffset: [f32; 3], // always set on return, an argument for RP_SET_PELVIS_OFFSET

    pub fImpactStrength: f32, //should be applicable when RagPhase is RP_DEATH_COLLISION
    pub fShotStrength: f32, //should be applicable for setting velocity of corpse on shot (probably only on RP_CORPSE_SHOT)
    //CServerEntity *me;
    pub me: c_int,

    //rww - we have convenient animation/frame access in the game, so just send this info over from there.
    pub startFrame: c_int,
    pub endFrame: c_int,

    pub collisionType: c_int, // 1 = from a fall, 0 from effectors, this will be going away soon, hence no enum

    pub CallRagDollBegin: bool, // a return value, means that we are now begininng ragdoll and the NPC stuff needs to happen

    pub RagPhase: c_int,

    // effector control, used for RP_DISABLE_EFFECTORS call
    pub effectorsToTurnOff: c_int,  // set this to an | of the above flags for a RP_DISABLE_EFFECTORS
}

// #endif //G2_GORE_H_INC
//rww - RAGDOLL_END

extern "C" {
    pub fn Z_Free(ptr: *mut core::ffi::c_void);
}
