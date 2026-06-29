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

  FILE:		ImmPeriodic.h

  PURPOSE:	Base Periodic Class for Immersion Foundation Classes

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

/// Stub for IMM_PERIODIC
#[repr(C)]
pub struct IMM_PERIODIC {
	// Stub: Periodic data structure
}

// ================================================================
// Constants
// ================================================================

pub const IMM_PERIODIC_DEFAULT_DURATION: DWORD = 1000; // Milliseconds
pub const IMM_PERIODIC_DEFAULT_MAGNITUDE: DWORD = 5000;
pub const IMM_PERIODIC_DEFAULT_PERIOD: DWORD = 100;  // Milliseconds
pub const IMM_PERIODIC_DEFAULT_OFFSET: DWORD = 0;
pub const IMM_PERIODIC_DEFAULT_PHASE: DWORD = 0;    // Degrees
pub const IMM_PERIODIC_DEFAULT_DIRECTION_X: DWORD = 1;    // Pixels
pub const IMM_PERIODIC_DEFAULT_DIRECTION_Y: DWORD = 0;    // Pixels
pub const IMM_PERIODIC_DEFAULT_ANGLE: DWORD = 9000; // 100ths of degrees

// Constants from ImmEffect.h
pub const IMM_EFFECTTYPE_PERIODIC: DWORD = 0x00000004;
pub const IMM_EFFECT_DONT_CHANGE: DWORD = 0xFFFFFFFF;
pub const IMM_EFFECT_DONT_CHANGE_PTR: *const core::ffi::c_void = core::ptr::null();

/// Stub GUID constants for Periodic effects (from ImmBaseTypes.h)
pub const GUID_Imm_Sine: GUID = GUID {
	Data1: 0,
	Data2: 0,
	Data3: 0,
	Data4: [0; 8],
};

pub const GUID_Imm_Square: GUID = GUID {
	Data1: 0,
	Data2: 0,
	Data3: 0,
	Data4: [0; 8],
};

pub const GUID_Imm_Triangle: GUID = GUID {
	Data1: 0,
	Data2: 0,
	Data3: 0,
	Data4: [0; 8],
};

pub const GUID_Imm_SawtoothUp: GUID = GUID {
	Data1: 0,
	Data2: 0,
	Data3: 0,
	Data4: [0; 8],
};

pub const GUID_Imm_SawtoothDown: GUID = GUID {
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
// CImmPeriodic
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

/// Base Periodic Class for Immersion Foundation Classes
#[repr(C)]
pub struct CImmPeriodic {
	//
	// INTERNAL DATA
	//

	pub m_Periodic: IMM_PERIODIC,
}

impl CImmPeriodic {
	//
	// CONSTRUCTOR/DESTRUCTOR
	//

	/// You may use this form if you will immediately initialize it
	/// from an IFR file...
	pub fn new() -> Self {
		CImmPeriodic {
			m_Periodic: unsafe { core::mem::zeroed() },
		}
	}

	/// Otherwise use this form...
	pub fn new_with_guid(rguidEffect: &GUID) -> Self {
		CImmPeriodic {
			m_Periodic: unsafe { core::mem::zeroed() },
		}
	}

	// Destructor (implicit via Drop or manual implementation in derived classes)

	//
	// ATTRIBUTES
	//

	pub fn GetIsCompatibleGUID(&self, guid: &mut GUID) -> BOOL {
		unsafe {
			IsEqualGUID(*guid, GUID_Imm_Sine) ||
			IsEqualGUID(*guid, GUID_Imm_Square) ||
			IsEqualGUID(*guid, GUID_Imm_Triangle) ||
			IsEqualGUID(*guid, GUID_Imm_SawtoothUp) ||
			IsEqualGUID(*guid, GUID_Imm_SawtoothDown)
		}
	}

	pub fn GetEffectType(&self) -> DWORD {
		IMM_EFFECTTYPE_PERIODIC
	}

	pub fn ChangeParameters(
		&mut self,
		dwMagnitude: DWORD,
		dwPeriod: DWORD,
		dwDuration: DWORD,
		lDirectionX: LONG,
		lDirectionY: LONG,
		lOffset: LONG,
		dwPhase: DWORD,
		pEnvelope: LPIMM_ENVELOPE,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn ChangeParametersPolar(
		&mut self,
		dwMagnitude: DWORD,
		dwPeriod: DWORD,
		dwDuration: DWORD,
		lAngle: LONG,
		lOffset: LONG,
		dwPhase: DWORD,
		pEnvelope: LPIMM_ENVELOPE,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn ChangeMagnitude(&mut self, dwMagnitude: DWORD) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn ChangePeriod(&mut self, dwPeriod: DWORD) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn ChangeOffset(&mut self, lOffset: LONG) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn ChangePhase(&mut self, dwPhase: DWORD) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn GetMagnitude(&self, dwMagnitude: &mut DWORD) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn GetPeriod(&self, dwPeriod: &mut DWORD) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn GetOffset(&self, lOffset: &mut LONG) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn GetPhase(&self, dwPhase: &mut DWORD) -> BOOL {
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
		dwMagnitude: DWORD,
		dwPeriod: DWORD,
		dwDuration: DWORD,
		lDirectionX: LONG,
		lDirectionY: LONG,
		lOffset: LONG,
		dwPhase: DWORD,
		pEnvelope: LPIMM_ENVELOPE,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: implementation unported (defined in .cpp source)
		0
	}

	pub fn InitializePolar(
		&mut self,
		pDevice: *mut CImmDevice,
		dwMagnitude: DWORD,
		dwPeriod: DWORD,
		dwDuration: DWORD,
		lAngle: LONG,
		lOffset: LONG,
		dwPhase: DWORD,
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
		dwMagnitude: DWORD,
		dwPeriod: DWORD,
		lOffset: LONG,
		dwPhase: DWORD,
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
		dwMagnitude: DWORD,
		dwPeriod: DWORD,
		lOffset: LONG,
		dwPhase: DWORD,
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
