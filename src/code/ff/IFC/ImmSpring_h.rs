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

  FILE:		ImmSpring.h

  PURPOSE:	Immersion Foundation Classes Spring Effect

  STARTED:	Oct.10.97

  NOTES/REVISIONS:
     Mar.02.99 jrm (Jeff Mallett): Force-->Feel renaming
	 Mar.02.99 jrm: Added GetIsCompatibleGUID
	 Mar.15.99 jrm: __declspec(dllimport/dllexport) the whole class
	 Nov.15.99 efw (Evan Wies): Converted to IFC

**********************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_long, c_uint};

// ================================================================
// Stub types for unported dependencies
// ================================================================

/// Boolean type (Windows BOOL)
pub type BOOL = c_int;

/// Windows DWORD
pub type DWORD = c_uint;

/// Windows LONG
pub type LONG = c_long;

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

/// Forward declaration for CImmCondition (from ImmCondition.h)
#[repr(C)]
pub struct CImmCondition {
	// Stub: actual fields defined in ImmCondition_h.rs
	_private: [u8; 0],
}

/// Forward declaration for CImmDevice (from ImmDevice)
#[repr(C)]
pub struct CImmDevice {
	// Stub: actual fields defined elsewhere
	_private: [u8; 0],
}

// Constants from ImmCondition.h
pub const IMM_EFFECT_DONT_CHANGE: DWORD = 0xFFFFFFFF;
pub const IMM_EFFECT_MOUSE_POS_AT_START: LONG = 0;
pub const IMM_EFFECT_AXIS_BOTH: DWORD = 0x00000003;

/// Stub GUID constant for Spring (from ImmBaseTypes.h)
pub const GUID_Imm_Spring: GUID = GUID {
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

//================================================================
// Constants
//================================================================

pub const IMM_SPRING_DEFAULT_STIFFNESS: LONG = 2500;
pub const IMM_SPRING_DEFAULT_SATURATION: DWORD = 10000;
pub const IMM_SPRING_DEFAULT_DEADBAND: DWORD = 100;
pub const IMM_SPRING_DEFAULT_CENTER_POINT: LONG = IMM_EFFECT_MOUSE_POS_AT_START;
pub const IMM_SPRING_DEFAULT_DIRECTION_X: LONG = 1;
pub const IMM_SPRING_DEFAULT_DIRECTION_Y: LONG = 0;
pub const IMM_SPRING_DEFAULT_ANGLE: LONG = 0;


//================================================================
// CImmSpring
//================================================================

//
// ------ PUBLIC INTERFACE ------
//

/// Immersion Foundation Classes Spring Effect
#[repr(C)]
pub struct CImmSpring {
	// Base class: CImmCondition
	pub base: CImmCondition,
}

impl CImmSpring {
	//
	// CONSTRUCTOR/DESTRUCTOR
	//

	/// Constructor
	pub fn new() -> Self {
		CImmSpring {
			base: CImmCondition { _private: [0; 0] },
		}
	}

	// Destructor (implicit via Drop or manual implementation in derived classes)

	//
	// ATTRIBUTES
	//

	pub fn GetIsCompatibleGUID(&self, guid: &mut GUID) -> BOOL {
		unsafe { IsEqualGUID(*guid, GUID_Imm_Spring) }
	}

	pub fn ChangeParameters(
		&mut self,
		pntCenter: POINT,
		lStiffness: LONG,
		dwSaturation: DWORD,
		dwDeadband: DWORD,
		lDirectionX: LONG,
		lDirectionY: LONG,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn ChangeParametersPolar(
		&mut self,
		pntCenter: POINT,
		lStiffness: LONG,
		dwSaturation: DWORD,
		dwDeadband: DWORD,
		lAngle: LONG,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	//
	// OPERATIONS
	//

	pub fn Initialize(
		&mut self,
		pDevice: *mut CImmDevice,
		lStiffness: LONG,
		dwSaturation: DWORD,
		dwDeadband: DWORD,
		dwfAxis: DWORD,
		pntCenter: POINT,
		lDirectionX: LONG,
		lDirectionY: LONG,
		bUseDeviceCoordinates: BOOL,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn InitializePolar(
		&mut self,
		pDevice: *mut CImmDevice,
		lStiffness: LONG,
		dwSaturation: DWORD,
		dwDeadband: DWORD,
		pntCenter: POINT,
		lAngle: LONG,
		bUseDeviceCoordinates: BOOL,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}
}

//
// ------ PRIVATE INTERFACE ------
//

//
// HELPERS
//

// (none defined in the header)

//
// INTERNAL DATA
//

// (inherited from CImmCondition)
