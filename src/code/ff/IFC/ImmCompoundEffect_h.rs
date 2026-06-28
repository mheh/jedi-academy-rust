/**********************************************************************
	Copyright (c) 1999 - 2000 Immersion Corporation

	Permission to use, copy, modify, distribute, and sell this
	software and its documentation may be granted without fee;
	interested parties are encouraged to request permission from
		Immersion Corporation
		801 Fox Lane
		San Jose, CA 95131
		408-467-1900

	IMMERSION DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE,
	INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS.
	IN NO EVENT SHALL IMMERSION BE LIABLE FOR ANY SPECIAL, INDIRECT OR
	CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
	LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
	NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
	CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

  FILE:		ImmCompoundEffect.h

  PURPOSE:	Manages Compound Effects for Force Foundation Classes

  STARTED:	2/24/99 by Jeff Mallett

  NOTES/REVISIONS:
     3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
	 3/15/99 jrm: __declspec(dllimport/dllexport) the whole class

**********************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_long, c_uint, c_char, c_void};

// ================================================================
// Stub types for unported dependencies
// ================================================================

/// Windows GUID type
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GUID {
	pub Data1: c_uint,
	pub Data2: u16,
	pub Data3: u16,
	pub Data4: [u8; 8],
}

/// Boolean type (Windows BOOL)
pub type BOOL = c_int;

/// Windows DWORD
pub type DWORD = c_uint;

/// Windows LONG
pub type LONG = c_long;

/// Character type (TCHAR)
pub type TCHAR = c_char;

/// Standard Windows MAX_PATH constant
pub const MAX_PATH: usize = 260;

/// Forward declaration for CImmEffect (from ImmEffect.h)
#[repr(C)]
pub struct CImmEffect {
	// Stub: actual fields defined in ImmEffect_h.rs
	_private: [u8; 0],
}

/// Forward declaration for CImmDevice (from ImmDevice)
#[repr(C)]
pub struct CImmDevice {
	// Stub: actual fields defined elsewhere
	_private: [u8; 0],
}

/// Type alias for pointer to CImmEffect
pub type GENERIC_EFFECT_PTR = *mut CImmEffect;

/// Stub for DIEFFECT (DirectInput)
#[repr(C)]
pub struct DIEFFECT {
	// Stub: DirectInput structure
}

/// Type alias for pointer to DIEFFECT
pub type LPDIEFFECT = *const DIEFFECT;

/// Stub for DIEFFECT mutable pointer
pub type LPDIEFFECT_MUT = *mut DIEFFECT;

/// Type alias for const string
pub type LPCSTR = *const c_char;

/// Stub for IFREffect structure
#[repr(C)]
pub struct IFREffect {
	pub guid: GUID,
	pub dwIterations: DWORD,
	pub effectName: *mut c_char,
	pub lpDIEffect: *const c_void,  // LPIMM_EFFECT
}

/// Stub for IMM_EFFECT
#[repr(C)]
pub struct IMM_EFFECT {
	// Stub: structure details from ImmEffect.h
}

/// Stub constant
pub const IMM_EFFECT_DONT_CHANGE: DWORD = 0xFFFFFFFF;

// ================================================================
// IMM_FFE_FILEEFFECT
// ================================================================

/*
**  IMM_FFE_FILEEFFECT - struct used by DX7 to read and write to FFE
**  files.  This struct is different from DIFILEEFFECT due to the use
**  of the non const LPDIEFFECT.  An LPDIEFFECT is needed to be able to
**  collect information from IFC class objects.  This should be defined
**  elsewhere, but no more appropriate header currently exists.
*/
#[repr(C)]
pub struct IMM_FFE_FILEEFFECT {
	pub dwSize: DWORD,
	pub GuidEffect: GUID,
	pub lpDiEffect: LPDIEFFECT_MUT,
	pub szFriendlyName: [TCHAR; MAX_PATH],
}

pub type LPIMM_FFE_FILEEFFECT = *mut IMM_FFE_FILEEFFECT;

// ================================================================
// CImmCompoundEffect
// ================================================================
// Represents a compound effect, such as might be created in
// Immersion Studio.  Contains an array of effect objects.
// Methods iterate over component effects, passing the message
// to each one.
// Also, has stuff for being used by CImmProject:
//   * next pointer so can be put on a linked list
//   * force name

//
// ------ PUBLIC INTERFACE ------
//

#[repr(C)]
pub struct CImmCompoundEffect {
	//
	// INTERNAL DATA
	//

	// Array of force class object pointers
	pub m_paEffects: *mut GENERIC_EFFECT_PTR,
	// Number of effects in m_paEffects
	pub m_nEffects: c_long,

	// Name of the compound effect
	pub m_lpszName: *mut c_char,
	pub m_objID: GUID,
	pub m_pContainedObjIDs: *mut GUID,
	// Next compound effect in the project
	pub m_pNext: *mut CImmCompoundEffect,
	// #ifdef PROTECT_AGAINST_DELETION
	//   pub m_pOwningProject: *mut CImmProject,
	// #endif
}

impl CImmCompoundEffect {
	//
	// CONSTRUCTOR/DESTRUCTOR
	//

	/// Constructs a CImmCompoundEffect
	/// Don't try to construct a CImmCompoundEffect yourself.
	/// Instead let CImmProject construct it for you.
	pub fn new(
		hEffects: *mut *mut IFREffect,
		nEffects: c_long,
		pEffectName: LPCSTR,
	) -> Self {
		CImmCompoundEffect {
			m_paEffects: core::ptr::null_mut(),
			m_nEffects: nEffects,
			m_lpszName: core::ptr::null_mut(),
			m_objID: GUID {
				Data1: 0,
				Data2: 0,
				Data3: 0,
				Data4: [0; 8],
			},
			m_pContainedObjIDs: core::ptr::null_mut(),
			m_pNext: core::ptr::null_mut(),
		}
	}

	//
	// ATTRIBUTES
	//

	/// Get the number of contained effects
	pub fn GetNumberOfContainedEffects(&self) -> c_long {
		self.m_nEffects
	}

	/// Get the name of the compound effect
	pub fn GetName(&self) -> LPCSTR {
		self.m_lpszName
	}

	/// Get a contained effect by index
	pub fn GetContainedEffect_ByIndex(&self, index: c_long) -> GENERIC_EFFECT_PTR {
		// Stub: implementation unported (defined in .cpp source)
		core::ptr::null_mut()
	}

	/// Get a contained effect by name
	pub fn GetContainedEffect_ByName(&self, lpszEffectName: LPCSTR) -> GENERIC_EFFECT_PTR {
		// Stub: implementation unported (defined in .cpp source)
		core::ptr::null_mut()
	}

	/// Get the effect type
	pub fn GetEffectType(&mut self) -> DWORD {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	//
	// OPERATIONS
	//

	/// Start all the contained effects
	pub fn Start(
		&mut self,
		dwIterations: DWORD,
		dwFlags: DWORD,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	/// Stop all the contained effects
	pub fn Stop(&mut self) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	//
	// ------ PRIVATE INTERFACE ------
	//

	//
	// HELPERS
	//

	/// Initialize helper
	pub fn initialize(
		&mut self,
		pDevice: *mut CImmDevice,
		hEffects: *mut *mut IFREffect,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	/// Set a contained effect
	pub fn set_contained_effect(&mut self, pObject: GENERIC_EFFECT_PTR, index: c_int) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	/// Set the name of the compound effect
	pub fn set_name(&mut self, lpszName: *const c_char) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	/// Set the next compound effect in the project
	pub fn set_next(&mut self, pNext: *mut CImmCompoundEffect) {
		self.m_pNext = pNext;
	}

	/// Get the next compound effect in the project
	pub fn get_next(&self) -> *mut CImmCompoundEffect {
		self.m_pNext
	}

	/// Set the object ID
	pub fn set_objID(&mut self, pobjID: *const GUID) {
		if !pobjID.is_null() {
			unsafe {
				self.m_objID = *pobjID;
			}
		}
	}

	/// Set the contained object IDs
	pub fn set_contained_obj_IDs(&mut self, guidList: *mut GUID) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	/// Buffer IFR object
	pub fn buffer_ifr_object(&mut self, pData: *mut TCHAR) -> c_int {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	/// Get FFE object
	pub fn get_ffe_object(&mut self, pffeObject: LPIMM_FFE_FILEEFFECT) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}
}

//
// ------ FRIENDS ------
//
// friend class CImmProject;
