// Faithful re-port of oracle/codemp/RMG/RM_Objective.h
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// #ifdef DEBUG_LINKING
// 	#pragma message("...including RM_Objective.h")
// #endif

// CGPGroup is defined in GenericParser2.h; glob-imported per #include translation
use crate::codemp::RMG::GenericParser2_h::*;

use core::ffi::{c_char, c_int, CStr};

pub struct CRMObjective {
    // protected:
    mCompleted:       bool,   // Is objective completed?
    mActive:          bool,   // set to false if the objective requires another objective to be met first
    mPriority:        c_int,  // sequence in which objectives need to be completed
    mOrderIndex:      c_int,  // objective index in ui
    mCompleteSoundID: c_int,  // sound for when objective is finished
    mMessage:         String, // message outputed when objective is completed
    mDescription:     String, // description of objective
    mInfo:            String, // more info for objective
    mName:            String, // name of objective
    mTrigger:         String, // trigger associated with objective
}

impl CRMObjective {
    // CRMObjective(CGPGroup *group);
    // port: constructor body is in RM_Objective.cpp; todo!() holds the declared signature
    pub fn new(group: *mut CGPGroup) -> Self {
        todo!("CRMObjective::CRMObjective(CGPGroup*) -- defined in RM_Objective.cpp")
    }

    // ~CRMObjective(void) {}
    // Rust drop is implicit; empty destructor requires no translation

    // bool Link(void); -- body is in RM_Objective.cpp
    pub fn Link(&mut self) -> bool {
        todo!("CRMObjective::Link() -- defined in RM_Objective.cpp")
    }

    pub fn IsCompleted(&self) -> bool { self.mCompleted }
    pub fn IsActive(&self) -> bool    { self.mActive }

    pub fn Activate(&mut self)            { self.mActive = true; }
    pub fn Complete(&mut self, comp: bool) { self.mCompleted = comp; }

    // Get methods
    pub fn GetPriority(&self) -> c_int    { self.mPriority }
    pub fn GetOrderIndex(&self) -> c_int  { self.mOrderIndex }
    // port: std::string::c_str() returns null-terminated pointer; String::as_ptr() is not
    // null-terminated. Faithful mechanical translation of the inline return expression.
    pub fn GetMessage(&self) -> *const c_char     { self.mMessage.as_ptr() as *const c_char }
    pub fn GetDescription(&self) -> *const c_char { self.mDescription.as_ptr() as *const c_char }
    pub fn GetInfo(&self) -> *const c_char        { self.mInfo.as_ptr() as *const c_char }
    pub fn GetName(&self) -> *const c_char        { self.mName.as_ptr() as *const c_char }
    pub fn GetTrigger(&self) -> *const c_char     { self.mTrigger.as_ptr() as *const c_char }
    pub fn CompleteSoundID(&self) -> c_int        { self.mCompleteSoundID }

    // Set methods
    pub fn SetPriority(&mut self, priority: c_int)  { self.mPriority = priority; }
    pub fn SetOrderIndex(&mut self, order: c_int)   { self.mOrderIndex = order; }
    // port: mMessage = msg translates std::string assignment from const char*; unsafe for raw ptr
    pub unsafe fn SetMessage(&mut self, msg: *const c_char) {
        self.mMessage = CStr::from_ptr(msg).to_string_lossy().into_owned();
    }
    pub unsafe fn SetDescription(&mut self, desc: *const c_char) {
        self.mDescription = CStr::from_ptr(desc).to_string_lossy().into_owned();
    }
    pub unsafe fn SetInfo(&mut self, info: *const c_char) {
        self.mInfo = CStr::from_ptr(info).to_string_lossy().into_owned();
    }
    pub unsafe fn SetName(&mut self, name: *const c_char) {
        self.mName = CStr::from_ptr(name).to_string_lossy().into_owned();
    }
    pub unsafe fn SetTrigger(&mut self, name: *const c_char) {
        self.mTrigger = CStr::from_ptr(name).to_string_lossy().into_owned();
    }

    // private:

    // CTriggerAriocheObjective*		FindRandomTrigger		( );
}

// typedef list<CRMObjective *>::iterator	rmObjectiveIter_t;
// port: C++ std::list<T>::iterator has no direct Rust equivalent; approximated as raw pointer
pub type rmObjectiveIter_t = *mut CRMObjective;
// typedef list<CRMObjective *>			rmObjectiveList_t;
pub type rmObjectiveList_t = std::collections::LinkedList<*mut CRMObjective>;
