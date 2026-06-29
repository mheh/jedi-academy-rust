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

  FILE:		ImmMouse.h

  PURPOSE:	Abstraction of Feelit mouse device

  STARTED:	10/10/97

  NOTES/REVISIONS:
     3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
	 3/15/99 jrm: __declspec(dllimport/dllexport) the whole class

**********************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// ================================================================
// Stub types for unported dependencies
// ================================================================

/// Windows HANDLE type (opaque pointer)
pub type HANDLE = *mut c_void;

/// Windows HWND type (window handle, opaque pointer)
pub type HWND = *mut c_void;

/// Windows HINSTANCE type (instance handle, opaque pointer)
pub type HINSTANCE = *mut c_void;

/// Windows DWORD type
pub type DWORD = c_int;

/// Windows BOOL type
pub type BOOL = c_int;

/// Windows LPTSTR type (pointer to string)
pub type LPTSTR = *mut c_int;

/// GUID structure for identifying devices
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GUID {
	pub Data1: u32,
	pub Data2: u16,
	pub Data3: u16,
	pub Data4: [u8; 8],
}

/// IFC API pointer type (opaque)
pub type LPIIMM_API = *mut c_void;

/// IFC Device pointer type (opaque)
pub type LPIIMM_DEVICE = *mut c_void;

/// IFC Device instance pointer type (opaque)
pub type LPIMM_DEVICEINSTANCE = *mut c_void;

/// Generic void pointer
pub type LPVOID = *mut c_void;

/// Forward declaration for CImmDevice (base class)
#[repr(C)]
pub struct CImmDevice {
	// Stub: actual fields defined in ImmDevice_h.rs
	_private: [u8; 0],
}

// ================================================================
// CImmMouse
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

/// Abstraction of Feelit mouse device
#[repr(C)]
pub struct CImmMouse {
	//
	// CONSTRUCTOR/DESTRUCTOR
	//

	//
	// ATTRIBUTES
	//

	//
	// OPERATIONS
	//

	//
	// ------ PRIVATE INTERFACE ------
	//

	//
	// HELPERS
	//

	//
	// INTERNAL DATA
	//

	/// IFC API interface pointer
	pub m_piApi: LPIIMM_API,

	/// IFC Device interface pointer
	pub m_piDevice: LPIIMM_DEVICE,
}

impl CImmMouse {
	//
	// CONSTRUCTOR/DESTRUCTOR
	//

	/// Constructor
	pub fn new() -> Self {
		CImmMouse {
			m_piApi: core::ptr::null_mut(),
			m_piDevice: core::ptr::null_mut(),
		}
	}

	//
	// ATTRIBUTES
	//

	/// Get API interface pointer
	pub fn GetAPI(&self) -> LPIIMM_API {
		self.m_piApi
	}

	/// Get Device interface pointer
	pub fn GetDevice(&self) -> LPIIMM_DEVICE {
		self.m_piDevice
	}

	/// Get the product type
	pub fn GetProductType(&mut self) -> DWORD {
		// Stub: to be implemented
		0
	}

	/// Get driver version information
	pub fn GetDriverVersion(
		&mut self,
		dwFFDriverVersion: &mut DWORD,
		dwFirmwareRevision: &mut DWORD,
		dwHardwareRevision: &mut DWORD,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get product name
	pub fn GetProductName(&mut self, lpszProductName: LPTSTR, nMaxCount: c_int) -> c_int {
		// Stub: to be implemented
		0
	}

	/// Get product GUID string
	pub fn GetProductGUIDString(&mut self, lpszGUID: LPTSTR, nMaxCount: c_int) -> c_int {
		// Stub: to be implemented
		0
	}

	/// Get product GUID
	pub fn GetProductGUID(&mut self) -> GUID {
		// Stub: to be implemented
		GUID {
			Data1: 0,
			Data2: 0,
			Data3: 0,
			Data4: [0; 8],
		}
	}

	/// Check if Imm mouse is available
	pub fn HaveImmMouse(&self) -> BOOL {
		if !self.m_piDevice.is_null() {
			1
		} else {
			0
		}
	}

	//
	// OPERATIONS
	//

	/// Initialize the mouse device
	pub fn Initialize(
		&mut self,
		hinstApp: HANDLE,
		hwndApp: HWND,
		dwCooperativeFlag: DWORD,
		bEnumerate: BOOL,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Change screen resolution
	pub fn ChangeScreenResolution(
		&mut self,
		bAutoSet: BOOL,
		dwXScreenSize: DWORD,
		dwYScreenSize: DWORD,
	) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Switch to absolute or relative mode
	/// The default is Absolute mode.  Call only to switch to Relative mode or
	/// to switch back to Absolute mode.
	pub fn SwitchToAbsoluteMode(&mut self, bAbsMode: BOOL) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Get current cursor position
	pub fn GetCurrentPosition(&mut self, lXPos: &mut i32, lYPos: &mut i32) -> BOOL {
		// Stub: to be implemented
		0
	}

	//
	// ------ PRIVATE INTERFACE ------
	//

	//
	// HELPERS
	//

	/// Reset the device
	pub fn reset(&mut self) {
		// Stub: to be implemented
	}

	/// Prepare device after initialization
	pub fn prepare_device(&mut self) -> BOOL {
		// Stub: to be implemented
		0
	}

	/// Static callback for device enumeration
	pub fn devices_enum_proc(
		pImmDevInst: LPIMM_DEVICEINSTANCE,
		pv: LPVOID,
	) -> BOOL {
		// Stub: to be implemented
		0
	}
}

//
// Note: Destructor would normally be virtual ~CImmMouse()
// In Rust, this would be handled via Drop trait if needed
//
