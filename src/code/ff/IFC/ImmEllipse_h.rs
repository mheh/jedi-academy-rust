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

  FILE:		ImmEllipse.h

  PURPOSE:	Base Ellipse Class for Immersion Foundation Classes

  STARTED:	10/29/97

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

/// Represents a Windows RECT structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RECT {
	pub left: c_long,
	pub top: c_long,
	pub right: c_long,
	pub bottom: c_long,
}

/// Type alias for pointer to RECT
pub type LPCRECT = *const RECT;

/// Boolean type (Windows BOOL)
pub type BOOL = c_int;

/// Windows DWORD
pub type DWORD = c_uint;

/// Windows LONG
pub type LONG = c_long;

/// Character type (TCHAR)
pub type TCHAR = c_char;

/// Windows GUID type
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GUID {
	pub Data1: c_uint,
	pub Data2: u16,
	pub Data3: u16,
	pub Data4: [u8; 8],
}

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

/// Stub for IMM_ELLIPSE
#[repr(C)]
pub struct IMM_ELLIPSE {
	// Stub: structure details from FeelitApi or ImmBaseTypes
}

/// Type alias for pointer to IMM_ELLIPSE
pub type LPIMM_ELLIPSE = *const IMM_ELLIPSE;

/// Stub for DIEFFECT (DirectInput)
#[repr(C)]
pub struct DIEFFECT {
	// Stub: DirectInput structure
}

/// Type alias for pointer to DIEFFECT
pub type LPDIEFFECT = *const DIEFFECT;

// ================================================================
// Constants
// ================================================================

pub const IMM_ELLIPSE_DEFAULT_STIFFNESS: LONG = 5000;
pub const IMM_ELLIPSE_DEFAULT_SATURATION: DWORD = 10000;
pub const IMM_ELLIPSE_DEFAULT_WIDTH: DWORD = 10;
pub const IMM_ELLIPSE_HEIGHT_AUTO: DWORD = 0xFFFFFFFF; // MAXDWORD
pub const IMM_ELLIPSE_DEFAULT_HEIGHT: DWORD = IMM_ELLIPSE_HEIGHT_AUTO;
pub const IMM_ELLIPSE_WALL_WIDTH_AUTO: DWORD = 0xFFFFFFFF; // MAXDWORD
pub const IMM_ELLIPSE_DEFAULT_WALL_WIDTH: DWORD = IMM_ELLIPSE_WALL_WIDTH_AUTO;
pub const IMM_ELLIPSE_DEFAULT_STIFFNESS_MASK: DWORD = 0x0000FFFF; // IMM_STIFF_ANYWALL
pub const IMM_ELLIPSE_DEFAULT_CLIPPING_MASK: DWORD = 0x00000000; // IMM_CLIP_NONE

pub const IMM_EFFECT_DONT_CHANGE: LONG = -2147483648i32; // MINLONG
pub const IMM_EFFECT_DONT_CHANGE_POINT: POINT = POINT {
	x: 0xFFFFFFFF as c_long,
	y: 0xFFFFFFFF as c_long,
};
pub const IMM_EFFECT_MOUSE_POS_AT_START: POINT = POINT {
	x: 2147483647i32,
	y: 2147483647i32,
};
pub const IMM_EFFECT_DEFAULT_ANGLE: LONG = 0;

pub const IMM_ELLIPSE_DEFAULT_CENTER_POINT: POINT = IMM_EFFECT_MOUSE_POS_AT_START;

// Constants for effect type
pub const IMM_EFFECTTYPE_ELLIPSE: DWORD = 0x00000020;

// Constants for stiffness masks (stub values)
pub const IMM_STIFF_ANYWALL: DWORD = 0x0000FFFF;

// Constants for clipping masks
pub const IMM_CLIP_NONE: DWORD = 0x00000000;

/// Stub GUID constant for Ellipse (from ImmBaseTypes.h)
pub const GUID_Imm_Ellipse: GUID = GUID {
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
// CImmEllipse
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

/// Base Ellipse Class for Immersion Foundation Classes
#[repr(C)]
pub struct CImmEllipse {
	/// Base class: CImmEffect
	pub base: CImmEffect,
	/// The ellipse data structure
	pub m_ellipse: IMM_ELLIPSE,
	/// Flag indicating whether to use mouse position at start
	pub m_bUseMousePosAtStart: BOOL,
	/// Pointer to inside effect for coordinating events
	pub m_pInsideEffect: *mut CImmEffect,
}

impl CImmEllipse {
	//
	// CONSTRUCTOR/DESTRUCTOR
	//

	/// Constructor
	pub fn new() -> Self {
		CImmEllipse {
			base: CImmEffect { _private: [0; 0] },
			m_ellipse: IMM_ELLIPSE {},
			m_bUseMousePosAtStart: 0,
			m_pInsideEffect: core::ptr::null_mut(),
		}
	}

	//
	// ATTRIBUTES
	//

	/// Get whether GUID is compatible
	pub fn GetIsCompatibleGUID(&self, guid: &GUID) -> BOOL {
		unsafe { IsEqualGUID(*guid, GUID_Imm_Ellipse) }
	}

	/// Get effect type
	pub fn GetEffectType(&self) -> DWORD {
		IMM_EFFECTTYPE_ELLIPSE
	}

	/// Change parameters using center point and optional parameters
	///
	/// # Arguments
	/// * `pntCenter` - Center point of the ellipse
	/// * `dwWidth` - Width value (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwHeight` - Height value (default: IMM_EFFECT_DONT_CHANGE)
	/// * `lStiffness` - Stiffness value (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwWallThickness` - Wall thickness (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwSaturation` - Saturation value (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwStiffnessMask` - Stiffness mask (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwClippingMask` - Clipping mask (default: IMM_EFFECT_DONT_CHANGE)
	/// * `pInsideEffect` - Inside effect pointer (default: IMM_EFFECT_DONT_CHANGE)
	/// * `lAngle` - Angle value (default: IMM_EFFECT_DONT_CHANGE)
	pub fn ChangeParameters(
		&mut self,
		pntCenter: POINT,
		dwWidth: DWORD,
		dwHeight: DWORD,
		lStiffness: LONG,
		dwWallThickness: DWORD,
		dwSaturation: DWORD,
		dwStiffnessMask: DWORD,
		dwClippingMask: DWORD,
		pInsideEffect: *mut CImmEffect,
		lAngle: LONG,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change parameters using rectangle bounds
	///
	/// # Arguments
	/// * `pRectOutside` - Rectangle defining the ellipse bounds
	/// * `lStiffness` - Stiffness value (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwWallThickness` - Wall thickness (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwSaturation` - Saturation value (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwStiffnessMask` - Stiffness mask (default: IMM_EFFECT_DONT_CHANGE)
	/// * `dwClippingMask` - Clipping mask (default: IMM_EFFECT_DONT_CHANGE)
	/// * `pInsideEffect` - Inside effect pointer (default: IMM_EFFECT_DONT_CHANGE)
	/// * `lAngle` - Angle value (default: IMM_EFFECT_DONT_CHANGE)
	pub fn ChangeParameters_Rect(
		&mut self,
		pRectOutside: LPCRECT,
		lStiffness: LONG,
		dwWallThickness: DWORD,
		dwSaturation: DWORD,
		dwStiffnessMask: DWORD,
		dwClippingMask: DWORD,
		pInsideEffect: *mut CImmEffect,
		lAngle: LONG,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change stiffness
	pub fn ChangeStiffness(&mut self, lStiffness: LONG) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change wall thickness
	pub fn ChangeWallThickness(&mut self, dwThickness: DWORD) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change saturation
	pub fn ChangeSaturation(&mut self, dwSaturation: DWORD) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change stiffness mask
	pub fn ChangeStiffnessMask(&mut self, dwStiffnessMask: DWORD) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change clipping mask
	pub fn ChangeClippingMask(&mut self, dwClippingMask: DWORD) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change inside effect
	pub fn ChangeInsideEffect(&mut self, pInsideEffect: *mut CImmEffect) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change rectangle
	pub fn ChangeRect(&mut self, pRect: LPCRECT) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change center point
	pub fn ChangeCenter(&mut self, pntCenter: POINT) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change center point using x, y coordinates
	pub fn ChangeCenter_XY(&mut self, x: LONG, y: LONG) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get stiffness
	pub fn GetStiffness(&self, lStiffness: &mut LONG) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get wall thickness
	pub fn GetWallThickness(&self, dwThickness: &mut DWORD) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get saturation
	pub fn GetSaturation(&self, dwSaturation: &mut DWORD) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get stiffness mask
	pub fn GetStiffnessMask(&self, dwStiffnessMask: &mut DWORD) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get clipping mask
	pub fn GetClippingMask(&self, dwClippingMask: &mut DWORD) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get rectangle
	pub fn GetRect(&self, pRect: *mut RECT) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get center point
	pub fn GetCenter(&self, pntCenter: &mut POINT) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get center point using x, y coordinates
	pub fn GetCenter_XY(&self, x: &mut LONG, y: &mut LONG) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get inside effect
	pub fn GetInsideEffect(&self) -> *mut CImmEffect {
		// Stub: to be implemented
		core::ptr::null_mut()
	}

	//
	// OPERATIONS
	//

	/// Initialize the ellipse with given effect structure
	pub fn Initialize_Effect(
		&mut self,
		pDevice: *mut CImmDevice,
		effect: &IMM_EFFECT,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Initialize the ellipse with given parameters using center point
	///
	/// # Arguments
	/// * `pDevice` - Device pointer
	/// * `dwWidth` - Width (default: IMM_ELLIPSE_DEFAULT_WIDTH)
	/// * `dwHeight` - Height (default: IMM_ELLIPSE_DEFAULT_HEIGHT)
	/// * `lStiffness` - Stiffness (default: IMM_ELLIPSE_DEFAULT_STIFFNESS)
	/// * `dwWallWidth` - Wall width (default: IMM_ELLIPSE_DEFAULT_WALL_WIDTH)
	/// * `dwSaturation` - Saturation (default: IMM_ELLIPSE_DEFAULT_SATURATION)
	/// * `dwStiffnessMask` - Stiffness mask (default: IMM_ELLIPSE_DEFAULT_STIFFNESS_MASK)
	/// * `dwClippingMask` - Clipping mask (default: IMM_ELLIPSE_DEFAULT_CLIPPING_MASK)
	/// * `pntCenter` - Center point (default: IMM_ELLIPSE_DEFAULT_CENTER_POINT)
	/// * `pInsideEffect` - Inside effect (default: NULL)
	/// * `lAngle` - Angle (default: IMM_EFFECT_DEFAULT_ANGLE)
	/// * `dwNoDownload` - No download flag (default: 0)
	pub fn Initialize(
		&mut self,
		pDevice: *mut CImmDevice,
		dwWidth: DWORD,
		dwHeight: DWORD,
		lStiffness: LONG,
		dwWallWidth: DWORD,
		dwSaturation: DWORD,
		dwStiffnessMask: DWORD,
		dwClippingMask: DWORD,
		pntCenter: POINT,
		pInsideEffect: *mut CImmEffect,
		lAngle: LONG,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Initialize the ellipse with given parameters using rectangle
	///
	/// # Arguments
	/// * `pDevice` - Device pointer
	/// * `pRectOutside` - Rectangle defining the ellipse bounds
	/// * `lStiffness` - Stiffness (default: IMM_ELLIPSE_DEFAULT_STIFFNESS)
	/// * `dwWallWidth` - Wall width (default: IMM_ELLIPSE_DEFAULT_WALL_WIDTH)
	/// * `dwSaturation` - Saturation (default: IMM_ELLIPSE_DEFAULT_SATURATION)
	/// * `dwStiffnessMask` - Stiffness mask (default: IMM_ELLIPSE_DEFAULT_STIFFNESS_MASK)
	/// * `dwClippingMask` - Clipping mask (default: IMM_ELLIPSE_DEFAULT_CLIPPING_MASK)
	/// * `pInsideEffect` - Inside effect (default: NULL)
	/// * `lAngle` - Angle (default: IMM_EFFECT_DEFAULT_ANGLE)
	/// * `dwNoDownload` - No download flag (default: 0)
	pub fn Initialize_Rect(
		&mut self,
		pDevice: *mut CImmDevice,
		pRectOutside: LPCRECT,
		lStiffness: LONG,
		dwWallWidth: DWORD,
		dwSaturation: DWORD,
		dwStiffnessMask: DWORD,
		dwClippingMask: DWORD,
		pInsideEffect: *mut CImmEffect,
		lAngle: LONG,
		dwNoDownload: DWORD,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Start the ellipse effect
	pub fn Start(
		&mut self,
		dwIterations: DWORD,
		dwFlags: DWORD,
		bAllowStartDelayEmulation: BOOL,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Unload the ellipse effect
	pub fn Unload(&mut self) -> i32 {
		// Stub: to be implemented (returns HRESULT)
		0
	}

	/// Reload the ellipse effect
	pub fn Reload(&mut self) {
		// Stub: to be implemented
	}

	//
	// PRIVATE INTERFACE (helpers)
	//

	/// Set parameters (helper)
	#[allow(dead_code)]
	fn set_parameters(
		&mut self,
		pRectOutside: LPCRECT,
		lStiffness: LONG,
		dwWallWidth: DWORD,
		dwSaturation: DWORD,
		dwStiffnessMask: DWORD,
		dwClippingMask: DWORD,
		pInsideEffect: *mut CImmEffect,
		lAngle: LONG,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change parameters (helper)
	#[allow(dead_code)]
	fn change_parameters(
		&mut self,
		prectBoundary: LPCRECT,
		lStiffness: LONG,
		dwWallThickness: DWORD,
		dwSaturation: DWORD,
		dwStiffnessMask: DWORD,
		dwClippingMask: DWORD,
		pInsideEffect: *mut CImmEffect,
		lAngle: LONG,
	) -> DWORD {
		// Stub: to be implemented
		0
	}

	/// Buffer IFR data (helper)
	#[allow(dead_code)]
	fn buffer_ifr_data(&mut self, pData: *mut TCHAR) -> i32 {
		// Stub: to be implemented
		0
	}
}

//
// ------ INLINE METHODS ------
//

/// Inline implementation of GetIsCompatibleGUID
#[inline]
pub fn CImmEllipse_GetIsCompatibleGUID_inline(ellipse: &CImmEllipse, guid: &GUID) -> BOOL {
	unsafe { IsEqualGUID(*guid, GUID_Imm_Ellipse) }
}
