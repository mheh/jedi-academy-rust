// Faithful port of oracle/codemp/RMG/RM_Objective.h
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};
use std::ffi::{CStr, CString};

// Forward declaration - opaque type
pub struct CGPGroup {
    _opaque: [u8; 0],
}

pub struct CRMObjective {
    // Is objective completed?
    mCompleted: bool,
    // set to false if the objective requires another objective to be met first
    mActive: bool,
    // sequence in which objectives need to be completed
    mPriority: c_int,
    // objective index in ui
    mOrderIndex: c_int,
    // sound for when objective is finished
    mCompleteSoundID: c_int,
    // message outputed when objective is completed
    mMessage: CString,
    // description of objective
    mDescription: CString,
    // more info for objective
    mInfo: CString,
    // name of objective
    mName: CString,
    // trigger associated with objective
    mTrigger: CString,
}

impl CRMObjective {
    pub fn new(group: *mut CGPGroup) -> Self {
        // Stub: actual constructor implementation would be in .cpp file
        CRMObjective {
            mCompleted: false,
            mActive: false,
            mPriority: 0,
            mOrderIndex: 0,
            mCompleteSoundID: 0,
            mMessage: CString::new("").unwrap(),
            mDescription: CString::new("").unwrap(),
            mInfo: CString::new("").unwrap(),
            mName: CString::new("").unwrap(),
            mTrigger: CString::new("").unwrap(),
        }
    }

    // Destructor is empty in original C++
    // Rust drop behavior is implicit

    pub fn Link(&mut self) -> bool {
        // Stub: implementation in .cpp file
        false
    }

    pub fn IsCompleted(&self) -> bool {
        self.mCompleted
    }

    pub fn IsActive(&self) -> bool {
        self.mActive
    }

    pub fn Activate(&mut self) {
        self.mActive = true;
    }

    pub fn Complete(&mut self, comp: bool) {
        self.mCompleted = comp;
    }

    // Get methods
    pub fn GetPriority(&self) -> c_int {
        self.mPriority
    }

    pub fn GetOrderIndex(&self) -> c_int {
        self.mOrderIndex
    }

    pub fn GetMessage(&self) -> *const c_char {
        self.mMessage.as_ptr()
    }

    pub fn GetDescription(&self) -> *const c_char {
        self.mDescription.as_ptr()
    }

    pub fn GetInfo(&self) -> *const c_char {
        self.mInfo.as_ptr()
    }

    pub fn GetName(&self) -> *const c_char {
        self.mName.as_ptr()
    }

    pub fn GetTrigger(&self) -> *const c_char {
        self.mTrigger.as_ptr()
    }

    pub fn CompleteSoundID(&self) -> c_int {
        self.mCompleteSoundID
    }

    // Set methods
    pub fn SetPriority(&mut self, priority: c_int) {
        self.mPriority = priority;
    }

    pub fn SetOrderIndex(&mut self, order: c_int) {
        self.mOrderIndex = order;
    }

    pub fn SetMessage(&mut self, msg: *const c_char) {
        if !msg.is_null() {
            unsafe {
                self.mMessage = CString::from_vec_unchecked(
                    CStr::from_ptr(msg).to_bytes().to_vec()
                );
            }
        }
    }

    pub fn SetDescription(&mut self, desc: *const c_char) {
        if !desc.is_null() {
            unsafe {
                self.mDescription = CString::from_vec_unchecked(
                    CStr::from_ptr(desc).to_bytes().to_vec()
                );
            }
        }
    }

    pub fn SetInfo(&mut self, info: *const c_char) {
        if !info.is_null() {
            unsafe {
                self.mInfo = CString::from_vec_unchecked(
                    CStr::from_ptr(info).to_bytes().to_vec()
                );
            }
        }
    }

    pub fn SetName(&mut self, name: *const c_char) {
        if !name.is_null() {
            unsafe {
                self.mName = CString::from_vec_unchecked(
                    CStr::from_ptr(name).to_bytes().to_vec()
                );
            }
        }
    }

    pub fn SetTrigger(&mut self, name: *const c_char) {
        if !name.is_null() {
            unsafe {
                self.mTrigger = CString::from_vec_unchecked(
                    CStr::from_ptr(name).to_bytes().to_vec()
                );
            }
        }
    }
}

// typedef list<CRMObjective *>::iterator	rmObjectiveIter_t;
pub type rmObjectiveIter_t = *mut CRMObjective;

// typedef list<CRMObjective *>			rmObjectiveList_t;
pub type rmObjectiveList_t = *mut CRMObjective;

// CTriggerAriocheObjective*		FindRandomTrigger		( );
