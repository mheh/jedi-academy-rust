//! Mechanical port of `codemp/RMG/RM_Objective.cpp`.
//!
//! Implements the CRMObjective class. This class is responsible for parsing an objective
//! from the mission file as well as linking the objective into the world.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};
use std::ffi::{CStr, CString};

// ============================================================================
// LOCAL STUBS for unported types
// ============================================================================
//
// These types are declared here to allow this file to compile structurally.
// Full definitions exist in the oracle but have not yet been ported.
// Porting these types is out of scope for this file.

/// Stub for unported `class CGPGroup` (GenericParser2.h).
/// Holds configuration key-value pairs used during objective parsing.
pub struct CGPGroup {
    _opaque: [u8; 0],
}

impl CGPGroup {
    /// Stub for `const char* CGPGroup::FindPairValue(const char *name, const char *defaultVal)`.
    /// Returns the value string associated with the given key, or default if not found.
    pub fn FindPairValue(&self, _name: *const c_char, default_val: *const c_char) -> *const c_char {
        // Porting stub: in reality, this looks up the key in internal storage
        // and returns the value or the default. For now, return the default.
        default_val
    }

    /// Stub for `const char* CGPGroup::GetName()`.
    /// Returns the name of this group.
    pub fn GetName(&self) -> *const c_char {
        core::ptr::null()
    }
}

// ============================================================================
// extern "C" functions from libc
// ============================================================================

extern "C" {
    /// C standard library function to convert a string to an integer.
    fn atoi(s: *const c_char) -> c_int;

    /// Format a string with variadic arguments (Quake engine function).
    /// Returns a pointer to a static buffer containing the formatted result.
    pub fn va(format: *const c_char, ...) -> *mut c_char;
}

// ============================================================================
// CRMObjective class
// ============================================================================

/// Represents a mission objective.
/// Holds the state, priority, description, and other properties of a mission objective.
pub struct CRMObjective {
    /// Is objective completed?
    mCompleted: bool,
    /// Set to false if the objective requires another objective to be met first.
    mActive: bool,
    /// Sequence in which objectives need to be completed.
    mPriority: c_int,
    /// Objective index in ui.
    mOrderIndex: c_int,
    /// Sound for when objective is finished.
    mCompleteSoundID: c_int,
    /// Message outputted when objective is completed.
    mMessage: CString,
    /// Description of objective.
    mDescription: CString,
    /// More info for objective.
    mInfo: CString,
    /// Name of objective.
    mName: CString,
    /// Trigger associated with objective.
    mTrigger: CString,
}

impl CRMObjective {
    /// CRMObjective::CRMObjective
    /// Constructs a random mission objective and fills in the default properties.
    pub fn new(group: *const CGPGroup) -> Self {
        let mut obj = CRMObjective {
            mCompleted: false,
            mActive: false,
            mPriority: 0,
            mOrderIndex: -1,
            mCompleteSoundID: 0,
            mMessage: CString::new("").unwrap(),
            mDescription: CString::new("").unwrap(),
            mInfo: CString::new("").unwrap(),
            mName: CString::new("").unwrap(),
            mTrigger: CString::new("").unwrap(),
        };

        unsafe {
            // Set priority from the "priority" key; default is "0"
            let priority_str = (*group).FindPairValue(
                b"priority\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
            );
            obj.SetPriority(atoi(priority_str));

            // Set message from the "message" key; default uses va()
            let message_str = (*group).FindPairValue(
                b"message\0".as_ptr() as *const c_char,
                va(
                    b"Objective %i Completed\0".as_ptr() as *const c_char,
                    obj.GetPriority(),
                ),
            );
            obj.SetMessage(message_str);

            // Set description from the "description" key; default uses va()
            let desc_str = (*group).FindPairValue(
                b"description\0".as_ptr() as *const c_char,
                va(
                    b"Objective %i\0".as_ptr() as *const c_char,
                    obj.GetPriority(),
                ),
            );
            obj.SetDescription(desc_str);

            // Set info from the "info" key; default uses va()
            let info_str = (*group).FindPairValue(
                b"info\0".as_ptr() as *const c_char,
                va(
                    b"Info %i\0".as_ptr() as *const c_char,
                    obj.GetPriority(),
                ),
            );
            obj.SetInfo(info_str);

            // Set trigger from the "trigger" key; default is empty string
            let trigger_str = (*group).FindPairValue(
                b"trigger\0".as_ptr() as *const c_char,
                b"\0".as_ptr() as *const c_char,
            );
            obj.SetTrigger(trigger_str);

            // Set name from the group name
            obj.SetName((*group).GetName());
        }

        // Commented out in original C++:
        // const char * soundPath = group->FindPairValue("completed_sound", "" );
        // if (soundPath)
        //     mCompleteSoundID = G_SoundIndex(soundPath);

        // If no priority was specified for this objective then its active by default.
        if obj.GetPriority() == 0 {
            obj.mActive = true;
        } else {
            obj.mActive = false;
        }

        obj
    }

    /// Destructor - empty in original C++
    /// Rust drop behavior is implicit.

    /// CRMObjective::Link
    /// Links the objective into the world using the current state of the world to determine
    /// where it should link.
    ///
    /// Returns: true if objective successfully linked, false otherwise.
    pub fn Link(&mut self) -> bool {
        // Commented out in original C++:
        // CTriggerAriocheObjective* trigger;
        // Look for a random trigger to associate this objective to.
        // trigger = FindRandomTrigger ( );
        // if ( NULL != trigger )
        // {
        //     trigger->SetObjective ( this );
        // }

        true
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

// Commented out in original C++:
// CRMObjective::FindRandomTrigger
// Searches the entitySystem for a random arioche trigger that matches the objective name
//
// Returns: a random trigger or NULL if one couldn't be found
//
// CTriggerAriocheObjective* CRMObjective::FindRandomTrigger ( )
// {
//     CEntity*	search;
//     CEntity*	triggers[20];
//     int			numTriggers;
//
//     // Start off the first trigger
//     numTriggers = 0;
//     search      = entitySystem->GetEntityFromClassname ( NULL, "trigger_arioche_objective" );
//
//     // Make a list of triggers
//     while ( numTriggers < 20 && search )
//     {
//         CTriggerAriocheObjective* trigger = (CTriggerAriocheObjective*) search;
//
//         // Move on to the next trigger
//         search = entitySystem->GetEntityFromClassname ( search, "trigger_arioche_objective" );
//
//         // See if this trigger is already in use
//         if ( trigger->GetObjective ( ) )
//         {
//             continue;
//         }
//
//         // If the objective names dont match then ignore this trigger
//         if ( stricmp ( trigger->GetObjectiveName ( ), GetTrigger() ) )
//         {
//             continue;
//         }
//
//         // Add the trigger to the list
//         triggers[numTriggers++] = trigger;
//     }
//
//     // If no matching triggers then just return NULL
//     if ( 0 == numTriggers )
//     {
//         return NULL;
//     }
//
//     // Return a random choice from the trigger list
//     return (CTriggerAriocheObjective*)triggers[TheRandomMissionManager->GetLandScape()->irand(0,numTriggers-1)];
// }

// typedef list<CRMObjective *>::iterator	rmObjectiveIter_t;
pub type rmObjectiveIter_t = *mut CRMObjective;

// typedef list<CRMObjective *>			rmObjectiveList_t;
pub type rmObjectiveList_t = *mut CRMObjective;
