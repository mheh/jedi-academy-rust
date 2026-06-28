//! `RoffSystem.h` — ROFF caching/playback system declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

use core::ffi::{c_char, c_int, c_long};
use core::marker::PhantomData;
use core::ptr::null_mut;

use crate::codemp::game::q_shared_h::{
    byte, qboolean, trajectory_t, trType_t, vec3_t, MAX_QPATH, QFALSE,
};

// ROFF Defines
//-------------------
pub const ROFF_VERSION: c_int = 1;
pub const ROFF_NEW_VERSION: c_int = 2;
pub const ROFF_STRING: &[u8; 4] = b"ROFF";
pub const ROFF_SAMPLE_RATE: c_int = 10; // 10hz
// #define ROFF_AUTO_FIX_BAD_ANGLES
// exporter can mess up angles,
//	defining this attempts to detect and fix these problems
pub const ROFF_AUTO_FIX_BAD_ANGLES: bool = true;

// Porting stub: the original header stores STL containers by value. Their C++ layout is
// intentionally opaque here; only the element/key parameters are preserved for traceability.
#[repr(C)]
pub struct std_map<K, V> {
    _opaque: [usize; 0],
    _K: PhantomData<K>,
    _V: PhantomData<V>,
}

impl<K, V> std_map<K, V> {
    pub const fn new() -> Self {
        Self {
            _opaque: [],
            _K: PhantomData,
            _V: PhantomData,
        }
    }
}

// Porting stub: see [`std_map`].
#[repr(C)]
pub struct std_vector<T> {
    _opaque: [usize; 0],
    _T: PhantomData<T>,
}

impl<T> std_vector<T> {
    pub const fn new() -> Self {
        Self {
            _opaque: [],
            _T: PhantomData,
        }
    }
}

pub type TROFFList = std_map<c_int, *mut CROFFSystem_CROFF>;
pub type TROFFEntList = std_vector<*mut CROFFSystem_SROFFEntity>;

// ROFF Header file definition, nothing else needs to see this
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TROFFHeader {
    pub mHeader: [c_char; 4], // should match roff_string defined above
    pub mVersion: c_long,    // version num, supported version defined above
    pub mCount: f32,         // I think this is a float because of a limitation of the roff exporter
}

// ROFF Entry, nothing else needs to see this
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TROFFEntry {
    pub mOriginOffset: [f32; 3],
    pub mRotateOffset: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TROFF2Header {
    pub mHeader: [c_char; 4], // should match roff_string defined above
    pub mVersion: c_long,    // version num, supported version defined above
    pub mCount: c_int,       // I think this is a float because of a limitation of the roff exporter
    pub mFrameRate: c_int,   // Frame rate the roff should be played at
    pub mNumNotes: c_int,    // number of notes (null terminated strings) after the roff data
}

// ROFF Entry, nothing else needs to see this
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TROFF2Entry {
    pub mOriginOffset: [f32; 3],
    pub mRotateOffset: [f32; 3],
    pub mStartNote: c_int,
    pub mNumNotes: c_int, // note track info
}

// An individual ROFF object,
//	contains actual rotation/offset information
//--------------------------------------
#[repr(C)]
pub struct CROFFSystem_CROFF {
    pub mID: c_int,                         // id for this roff file
    pub mROFFFilePath: [c_char; MAX_QPATH], // roff file path
    pub mROFFEntries: c_int,                // count of move/rotate commands
    pub mFrameTime: c_int,                  // frame rate
    pub mLerp: c_int,                       // Lerp rate (FPS)
    pub mMoveRotateList: *mut TROFF2Entry,  // move rotate/command list
    pub mNumNoteTracks: c_int,
    pub mNoteTrackIndexes: *mut *mut c_char,
    pub mUsedByClient: qboolean,
    pub mUsedByServer: qboolean,
}

impl CROFFSystem_CROFF {
    pub const fn new() -> Self {
        Self {
            mID: 0,
            mROFFFilePath: [0; MAX_QPATH],
            mROFFEntries: 0,
            mFrameTime: 0,
            mLerp: 0,
            mMoveRotateList: null_mut(),
            mNumNoteTracks: 0,
            mNoteTrackIndexes: null_mut(),
            mUsedByClient: QFALSE,
            mUsedByServer: QFALSE,
        }
    }

    // CROFF( const char *file, int id );
    // ~CROFF();
}

// The roff system tracks entities that are
//	roffing, so this is the internal structure
//	that represents these objects.
//--------------------------------------
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CROFFSystem_SROFFEntity {
    pub mEntID: c_int, // the entity that is currently roffing

    pub mROFFID: c_int,       // the roff to be applied to that entity
    pub mNextROFFTime: c_int, // next time we should roff
    pub mROFFFrame: c_int,    // current roff frame we are applying

    pub mKill: qboolean,       // flag to kill a roffing ent
    pub mSignal: qboolean,     // TODO:  Need to implement some sort of signal to Icarus when roff is done.
    pub mTranslated: qboolean, // should this roff be "rotated" to fit the entity's initial position?
    pub mIsClient: qboolean,
    pub mStartAngles: vec3_t, // initial angle of the entity
}

// The CROFFSystem object provides all of the functionality of ROFF
//	caching, playback, and clean-up, plus some useful debug features.
//--------------------------------------
#[repr(C)]
pub struct CROFFSystem {
    pub mROFFList: TROFFList,       // List of cached roffs
    pub mID: c_int,                 // unique ID generator for new roff objects
    pub mROFFEntList: TROFFEntList, // List of roffing entities
}

impl CROFFSystem {
    pub const fn new() -> Self {
        Self {
            mROFFList: TROFFList::new(),
            mID: 0,
            mROFFEntList: TROFFEntList::new(),
        }
    }

    // Increment before return so we can use zero as failed return val
    pub unsafe fn NewID(&mut self) -> c_int {
        self.mID = self.mID.wrapping_add(1);
        self.mID
    }

    // Makes sure the file is a valid roff file
    pub unsafe fn IsROFF(&mut self, _file: *mut byte) -> qboolean {
        todo!("CROFFSystem::IsROFF body is in the unported .cpp")
    }

    // Handles stashing raw roff data into the roff object
    pub unsafe fn InitROFF(&mut self, _file: *mut byte, _obj: *mut CROFFSystem_CROFF) -> qboolean {
        todo!("CROFFSystem::InitROFF body is in the unported .cpp")
    }

    // Handles stashing raw roff data into the roff object
    pub unsafe fn InitROFF2(&mut self, _file: *mut byte, _obj: *mut CROFFSystem_CROFF) -> qboolean {
        todo!("CROFFSystem::InitROFF2 body is in the unported .cpp")
    }

    pub unsafe fn FixBadAngles(&mut self, _obj: *mut CROFFSystem_CROFF) {
        todo!("CROFFSystem::FixBadAngles body is in the unported .cpp")
    }

    // True = success; False = roff complete
    pub unsafe fn ApplyROFF(
        &mut self,
        _roff_ent: *mut CROFFSystem_SROFFEntity,
        _roff: *mut CROFFSystem_CROFF,
    ) -> qboolean {
        todo!("CROFFSystem::ApplyROFF body is in the unported .cpp")
    }

    pub unsafe fn ProcessNote(&mut self, _roff_ent: *mut CROFFSystem_SROFFEntity, _note: *mut c_char) {
        todo!("CROFFSystem::ProcessNote body is in the unported .cpp")
    }

    pub unsafe fn SetLerp(
        &mut self,
        _tr: *mut trajectory_t,
        _trType: trType_t,
        _origin: vec3_t,
        _delta: vec3_t,
        _time: c_int,
        _rate: c_int,
    ) {
        todo!("CROFFSystem::SetLerp body is in the unported .cpp")
    }

    // Clears out the angular and position lerp fields
    pub unsafe fn ClearLerp(&mut self, _roff_ent: *mut CROFFSystem_SROFFEntity) -> qboolean {
        todo!("CROFFSystem::ClearLerp body is in the unported .cpp")
    }

    // Free up all system resources and reset the ID counter
    pub unsafe fn Restart(&mut self) -> qboolean {
        todo!("CROFFSystem::Restart body is in the unported .cpp")
    }

    // roffs should be precached at the start of each level
    pub unsafe fn Cache(&mut self, _file: *const c_char, _isClient: qboolean) -> c_int {
        todo!("CROFFSystem::Cache body is in the unported .cpp")
    }

    // find the roff id by filename
    pub unsafe fn GetID(&mut self, _file: *const c_char) -> c_int {
        todo!("CROFFSystem::GetID body is in the unported .cpp")
    }

    // when a roff is done, it can be removed to free up resources
    pub unsafe fn Unload(&mut self, _id: c_int) -> qboolean {
        todo!("CROFFSystem::Unload body is in the unported .cpp")
    }

    // should be called when level is done, frees all roff resources
    pub unsafe fn Clean(&mut self, _isClient: qboolean) -> qboolean {
        todo!("CROFFSystem::Clean body is in the unported .cpp")
    }

    // dumps a list of all cached roff files to the console
    pub unsafe fn List(&mut self) {
        todo!("CROFFSystem::List body is in the unported .cpp")
    }

    // dumps the contents of the specified roff to the console
    pub unsafe fn List_id(&mut self, _id: c_int) -> qboolean {
        todo!("CROFFSystem::List(int) body is in the unported .cpp")
    }

    // TODO: implement signal on playback completion.
    pub unsafe fn Play(
        &mut self,
        _entID: c_int,
        _roffID: c_int,
        _doTranslation: qboolean,
        _isClient: qboolean,
    ) -> qboolean {
        todo!("CROFFSystem::Play body is in the unported .cpp")
    }

    // List the entities that are currently roffing
    pub unsafe fn ListEnts(&mut self) {
        todo!("CROFFSystem::ListEnts body is in the unported .cpp")
    }

    // Purge the specified entity from the entity list by id
    pub unsafe fn PurgeEnt(&mut self, _entID: c_int, _isClient: qboolean) -> qboolean {
        todo!("CROFFSystem::PurgeEnt body is in the unported .cpp")
    }

    // Purge the specified entity from the entity list by name
    pub unsafe fn PurgeEnt_file(&mut self, _file: *mut c_char) -> qboolean {
        todo!("CROFFSystem::PurgeEnt(char *) body is in the unported .cpp")
    }

    // applys roff data to roffing entities.
    pub unsafe fn UpdateEntities(&mut self, _isClient: qboolean) {
        todo!("CROFFSystem::UpdateEntities body is in the unported .cpp")
    }
}

pub type CROFF = CROFFSystem_CROFF;
pub type SROFFEntity = CROFFSystem_SROFFEntity;

unsafe extern "C" {
    pub static mut theROFFSystem: CROFFSystem;
}
