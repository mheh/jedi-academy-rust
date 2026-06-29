// #pragma once
// #if !defined(RM_OBJECTIVE_H_INC)
// #define RM_OBJECTIVE_H_INC

// #ifdef DEBUG_LINKING
// 	#pragma message("...including RM_Objective.h")
// #endif

#![allow(non_snake_case, non_camel_case_types)]

// Port note: oracle header has no explicit #include directive; CGPGroup originates from
// genericparser2.h (oracle/code/game/genericparser2.h) per triage — glob-imported here.
use crate::code::game::genericparser2_h::*;
use core::ffi::{c_char, c_int};
use std::collections::LinkedList;
use std::ffi::{CStr, CString};

pub struct CRMObjective {
    // protected:
    pub(crate) mCompleted: bool,        // Is objective completed?
    pub(crate) mActive: bool,           // set to false if the objective requires another objective to be met first
    pub(crate) mPriority: c_int,        // sequence in which objectives need to be completed
    pub(crate) mOrderIndex: c_int,      // objective index in ui
    pub(crate) mCompleteSoundID: c_int, // sound for when objective is finished
    pub(crate) mMessage: CString,       // message outputed when objective is completed
    pub(crate) mDescription: CString,   // description of objective
    pub(crate) mInfo: CString,          // more info for objective
    pub(crate) mName: CString,          // name of objective
    pub(crate) mTrigger: CString,       // trigger associated with objective
}

impl CRMObjective {
    // public:

    // CRMObjective(CGPGroup *group);
    // ~CRMObjective(void) {}

    // bool			Link			(void);
    // (declared here; defined in RM_Objective.cpp)

    // bool			IsCompleted		(void) const { return mCompleted; }
    pub fn IsCompleted(&self) -> bool {
        self.mCompleted
    }

    // bool			IsActive		(void) const { return mActive; }
    pub fn IsActive(&self) -> bool {
        self.mActive
    }

    // void			Activate		(void)		 { mActive = true; }
    pub fn Activate(&mut self) {
        self.mActive = true;
    }

    // void			Complete		(bool comp)  { mCompleted = comp;}
    pub fn Complete(&mut self, comp: bool) {
        self.mCompleted = comp;
    }

    // Get methods
    // int				GetPriority(void){return mPriority;}
    pub fn GetPriority(&mut self) -> c_int {
        self.mPriority
    }

    // int				GetOrderIndex(void) { return mOrderIndex; }
    pub fn GetOrderIndex(&mut self) -> c_int {
        self.mOrderIndex
    }

    // const char*		GetMessage(void) { return mMessage.c_str(); }
    pub fn GetMessage(&mut self) -> *const c_char {
        self.mMessage.as_ptr()
    }

    // const char*		GetDescription(void) { return mDescription.c_str(); }
    pub fn GetDescription(&mut self) -> *const c_char {
        self.mDescription.as_ptr()
    }

    // const char*		GetInfo(void) { return mInfo.c_str(); }
    pub fn GetInfo(&mut self) -> *const c_char {
        self.mInfo.as_ptr()
    }

    // const char*		GetName(void) { return mName.c_str(); }
    pub fn GetName(&mut self) -> *const c_char {
        self.mName.as_ptr()
    }

    // const char*		GetTrigger(void) { return mTrigger.c_str(); }
    pub fn GetTrigger(&mut self) -> *const c_char {
        self.mTrigger.as_ptr()
    }

    // int				CompleteSoundID() { return mCompleteSoundID; };
    pub fn CompleteSoundID(&mut self) -> c_int {
        self.mCompleteSoundID
    }

    // Set methods
    // void			SetPriority(int priority){mPriority = priority;}
    pub fn SetPriority(&mut self, priority: c_int) {
        self.mPriority = priority;
    }

    // void			SetOrderIndex(int order) { mOrderIndex = order; }
    pub fn SetOrderIndex(&mut self, order: c_int) {
        self.mOrderIndex = order;
    }

    // void			SetMessage(const char* msg) { mMessage = msg; }
    pub unsafe fn SetMessage(&mut self, msg: *const c_char) {
        self.mMessage = CStr::from_ptr(msg).to_owned();
    }

    // void			SetDescription(const char* desc) { mDescription = desc; }
    pub unsafe fn SetDescription(&mut self, desc: *const c_char) {
        self.mDescription = CStr::from_ptr(desc).to_owned();
    }

    // void			SetInfo(const char* info) { mInfo = info; }
    pub unsafe fn SetInfo(&mut self, info: *const c_char) {
        self.mInfo = CStr::from_ptr(info).to_owned();
    }

    // void			SetName(const char* name) { mName = name; }
    pub unsafe fn SetName(&mut self, name: *const c_char) {
        self.mName = CStr::from_ptr(name).to_owned();
    }

    // void			SetTrigger(const char* name) { mTrigger = name; }
    pub unsafe fn SetTrigger(&mut self, name: *const c_char) {
        self.mTrigger = CStr::from_ptr(name).to_owned();
    }

    // private:

    //	CTriggerAriocheObjective*		FindRandomTrigger		( );
}

pub type rmObjectiveIter_t = *mut CRMObjective;
pub type rmObjectiveList_t = LinkedList<*mut CRMObjective>;

// #endif
