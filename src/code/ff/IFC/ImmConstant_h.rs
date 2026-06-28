/**********************************************************************
	Copyright (c) 1997 - 2000 Immersion Corporation

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

  FILE:		ImmConstant.h

  PURPOSE:	Base Constant Class for Immersion Foundation Classes

  STARTED:	11/03/97

  NOTES/REVISIONS:
     3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
	 3/2/99 jrm: Added GetIsCompatibleGUID
	 3/15/99 jrm: __declspec(dllimport/dllexport) the whole class
	 11/15/99 sdr (Steve Rank): Converted to IFC
**********************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_long, c_uint, c_char};

// ================================================================
// Stub types for unported dependencies
// ================================================================

/// Represents a Windows POINT structure (x, y coordinates)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct POINT {
	pub x: c_long,
	pub y: c_long,
}

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

/// Stub for IMM_ENVELOPE
#[repr(C)]
pub struct IMM_ENVELOPE {
	// Stub: structure details from ImmEffect.h
}

/// Stub for IMM_EFFECT
#[repr(C)]
pub struct IMM_EFFECT {
	// Stub: structure details from ImmEffect.h
}

/// Type alias for pointer to IMM_ENVELOPE
pub type LPIMM_ENVELOPE = *const IMM_ENVELOPE;

/// Stub for DIEFFECT (DirectInput)
#[repr(C)]
pub struct DIEFFECT {
	// Stub: DirectInput structure
}

/// Type alias for pointer to DIEFFECT
pub type LPDIEFFECT = *const DIEFFECT;

/// Stub for IMM_CONSTANTFORCE
#[repr(C)]
pub struct IMM_CONSTANTFORCE {
	// Stub: Constant force data structure
}

// ================================================================
// Constants
// ================================================================

pub const IMM_CONSTANT_DEFAULT_DIRECTION: POINT = POINT { x: 1, y: 0 };
pub const IMM_CONSTANT_DEFAULT_DURATION: DWORD = 1000; // Milliseconds
pub const IMM_CONSTANT_DEFAULT_MAGNITUDE: LONG = 5000;

// Constants from ImmEffect.h
pub const IMM_EFFECTTYPE_CONSTANTFORCE: DWORD = 0x00000001;
pub const IMM_EFFECT_DONT_CHANGE: DWORD = 0xFFFFFFFF;
pub const IMM_EFFECT_DONT_CHANGE_PTR: *const core::ffi::c_void = core::ptr::null();
pub const IMM_EFFECT_DEFAULT_DIRECTION_X: LONG = 1;
pub const IMM_EFFECT_DEFAULT_DIRECTION_Y: LONG = 1;
pub const IMM_EFFECT_DEFAULT_ANGLE: LONG = 0;

/// Stub GUID constant for Constant Force (from ImmBaseTypes.h)
pub const GUID_Imm_ConstantForce: GUID = GUID {
	Data1: 0,
	Data2: 0,
	Data3: 0,
	Data4: [0; 8],
};

// ================================================================
// External functions
// ================================================================

extern "C" {
	/// IsEqualGUID function from Windows
	pub fn IsEqualGUID(guid1: GUID, guid2: GUID) -> BOOL;
}

// ================================================================
// CImmConstant
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

/// Base Constant Class for Immersion Foundation Classes
#[repr(C)]
pub struct CImmConstant {
	//
	// INTERNAL DATA
	//

	pub m_ConstantForce: IMM_CONSTANTFORCE,
}

impl CImmConstant {
	//
	// CONSTRUCTOR/DESTRUCTOR
	//

	/// Constructor
	pub fn new() -> Self {
		CImmConstant {
			m_ConstantForce: unsafe { core::mem::zeroed() },
		}
	}

	// Destructor (implicit via Drop or manual implementation in derived classes)

	//
	// ATTRIBUTES
	//

	pub fn GetIsCompatibleGUID(&self, guid: &mut GUID) -> BOOL {
		unsafe { IsEqualGUID(*guid, GUID_Imm_ConstantForce) }
	}

	pub fn GetEffectType(&self) -> DWORD {
		IMM_EFFECTTYPE_CONSTANTFORCE
	}

	pub fn ChangeParameters(
		&mut self,
		lDirectionX: LONG,
		lDirectionY: LONG,
		dwDuration: DWORD,
		lMagnitude: LONG,
		pEnvelope: LPIMM_ENVELOPE,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn ChangeParametersPolar(
		&mut self,
		lAngle: LONG,
		dwDuration: DWORD,
		lMagnitude: LONG,
		pEnvelope: LPIMM_ENVELOPE,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn ChangeMagnitude(&mut self, lMagnitude: LONG) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn GetMagnitude(&self, lMagnitude: &mut LONG) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	//
	// OPERATIONS
	//

	pub fn Initialize(
		&mut self,
		pDevice: *mut CImmDevice,
		effect: &IMM_EFFECT,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn Initialize_With_Params(
		&mut self,
		pDevice: *mut CImmDevice,
		lDirectionX: LONG,
		lDirectionY: LONG,
		dwDuration: DWORD,
		lMagnitude: LONG,
		pEnvelope: LPIMM_ENVELOPE,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn InitializePolar(
		&mut self,
		pDevice: *mut CImmDevice,
		lAngle: LONG,
		dwDuration: DWORD,
		lMagnitude: LONG,
		pEnvelope: LPIMM_ENVELOPE,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	//
	// PRIVATE INTERFACE
	//

	//
	// HELPERS
	//

	fn set_parameters(
		&mut self,
		dwfCoordinates: DWORD,
		lDirection0: LONG,
		lDirection1: LONG,
		dwDuration: DWORD,
		lMagnitude: LONG,
		pEnvelope: LPIMM_ENVELOPE,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	fn change_parameters(
		&mut self,
		lDirection0: LONG,
		lDirection1: LONG,
		dwDuration: DWORD,
		lMagnitude: LONG,
		pEnvelope: LPIMM_ENVELOPE,
	) -> DWORD {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	fn buffer_ifr_data(&mut self, pData: *mut TCHAR) -> c_int {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	fn get_ffe_data(&mut self, pdiEffect: LPDIEFFECT) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}
}
