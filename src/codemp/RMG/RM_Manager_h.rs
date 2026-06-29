#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;

// #if !defined(CM_LANDSCAPE_H_INC)
// #include "../qcommon/cm_landscape.h"
use crate::codemp::qcommon::cm_landscape_h::*;

// External types — per triage cautions, imported from the headers that define them:
// CRMMission  (RM_Mission.h)
use crate::codemp::RMG::RM_Mission_h::*;
// CRandomTerrain  (../qcommon/cm_randomterrain.h)
use crate::codemp::qcommon::cm_randomterrain_h::*;
// CRMObjective  (RM_Objective.h)
use crate::codemp::RMG::RM_Objective_h::*;

// rmAutomapSymbol_t and MAX_AUTOMAP_SYMBOLS are defined in client/client.h (included
// transitively before this header via RM_Headers.h → client.h).  They are kept as
// local definitions here per triage guidance — they are the genuine local types of
// this module.
pub const MAX_AUTOMAP_SYMBOLS: usize = 512;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rmAutomapSymbol_t {
    pub mType:   c_int,
    pub mSide:   c_int,
    pub mOrigin: vec3_t,
}

#[repr(C)]
pub struct CRMManager {
    // private:
    mMission:            *mut CRMMission,
    mLandScape:          *mut CCMLandScape,
    mTerrain:            *mut CRandomTerrain,
    mPreviewTimer:       c_int,
    mCurPriority:        c_int,
    mUseTimeLimit:       bool,

    mAutomapSymbols:     [rmAutomapSymbol_t; MAX_AUTOMAP_SYMBOLS],
    mAutomapSymbolCount: c_int,
}

impl CRMManager {
    fn UpdateStatisticCvars(&mut self) { todo!() }
}

impl CRMManager {
    // Constructors
    pub fn new() -> Self { todo!() }

    pub fn LoadMission(&mut self, IsServer: qboolean) -> bool { todo!() }
    pub fn SpawnMission(&mut self, IsServer: qboolean) -> bool { todo!() }

    // Accessors
    pub fn SetLandScape(&mut self, landscape: *mut CCMLandScape) { todo!() }
    pub fn SetCurPriority(&mut self, priority: c_int) { self.mCurPriority = priority; }

    pub fn GetTerrain(&self) -> *mut CRandomTerrain { self.mTerrain }
    pub fn GetLandScape(&self) -> *mut CCMLandScape { self.mLandScape }
    pub fn GetMission(&self) -> *mut CRMMission { self.mMission }
    pub fn GetCurPriority(&self) -> c_int { self.mCurPriority }

    pub fn AddAutomapSymbol(&mut self, type_: c_int, origin: vec3_t, side: c_int) { todo!() }
    pub fn GetAutomapSymbolCount(&self) -> c_int { todo!() }
    pub fn GetAutomapSymbol(&mut self, index: c_int) -> *mut rmAutomapSymbol_t { todo!() }
    pub fn ProcessAutomapSymbols(count: c_int, symbols: *mut rmAutomapSymbol_t) { todo!() }

    pub fn Preview(&self, from: vec3_t) { todo!() }

    pub fn IsMissionComplete(&self) -> bool { todo!() }
    pub fn HasTimeExpired(&self) -> bool { todo!() }
    pub fn CompleteObjective(&mut self, obj: *mut CRMObjective) { todo!() }
    pub fn CompleteMission(&mut self) { todo!() }
    pub fn FailedMission(&mut self, TimeExpired: bool) { todo!() }

    // eek
    // static CRMObjective *mCurObjective; — translated as module-level static below
}

impl Drop for CRMManager {
    fn drop(&mut self) { todo!() }
}

// eek
pub static mut mCurObjective: *mut CRMObjective = core::ptr::null_mut();

// extern CRMManager* TheRandomMissionManager;
// -- defined and initialized in RM_Manager.rs; not re-declared here to avoid a
//    duplicate static in the same crate.

// #endif // RANDOMMISSION_H_INC
