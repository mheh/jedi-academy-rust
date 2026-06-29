/**********************************************************************
	Copyright (c) 2000 Immersion Corporation

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
	NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF THE
	CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

  FILE:		ImmDevices.h

  PURPOSE:	Abstract Base Device Class for Immersion Foundation Classes

  STARTED:	3/29/00

  NOTES/REVISIONS:
     3/29/00 jrm (Jeff Mallett): Started

**********************************************************************/

use core::ffi::{c_int, c_void};

// Note: This is a Windows-specific header using HANDLE, HWND, HINSTANCE types.
// These are declared as opaque pointers (void* equivalents) from core::ffi.
pub type HANDLE = *mut c_void;
pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type DWORD = core::ffi::c_uint;
pub type BOOL = c_int;

// Forward declarations for types that depend on external headers
// Included from ImmBaseTypes.h (dependency stub)
mod _imm_base_types {
    // Placeholder for external ImmBaseTypes definitions
    // Actual definitions would come from ImmBaseTypes.h
}

#[repr(C)]
pub enum IMM_ENUMERATE {
	IMM_ENUMERATE_IMM_DEVICES = 0x00000001,
	IMM_ENUMERATE_DX_DEVICES = 0x00000002,
	IMM_ENUMERATE_ALL = 0xFFFFFFFF,
}

#[repr(C)]
pub enum IMM_ENUMERATE_PREFERENCE {
	IMM_NO_PREFERENCE = 0x00000000,
	IMM_PREFER_IMM_DEVICES = 0x00000001,
	IMM_PREFER_DX_DEVICES = 0x00000002,
}

// Forward declaration
pub struct CImmDevice;

#[repr(C)]
pub struct CInitializeEnum {
	pub m_hinstApp: HINSTANCE,
	pub m_hwndApp: HWND,
	pub m_dwCooperativeFlag: DWORD,
	pub m_pDevices: *mut CImmDevices,
	pub m_lMaximumDevices: c_int,
}

pub type IMM_DEVICE_PTR = *mut CImmDevice;

//================================================================
// CImmDevices
//================================================================

//
// ------ PUBLIC INTERFACE ------
//

#[repr(C)]
pub struct CImmDevices {
	//
	// ATTRIBUTES
	//

	/// INTERNAL DATA
	pub m_lNumDevices: c_int,
	pub m_DeviceArray: *mut IMM_DEVICE_PTR,
}

//
// Note: Method implementations are defined in corresponding .cpp file
// The following are forward declarations matching the C++ class interface
//

extern "C" {
	//
	// CONSTRUCTOR/DESCTRUCTOR
	//

	pub fn CImmDevices_new() -> *mut CImmDevices;
	pub fn CImmDevices_delete(this: *mut CImmDevices);

	//
	// ATTRIBUTES
	//

	pub fn CImmDevices_GetNumDevices(this: *const CImmDevices) -> c_int;
	pub fn CImmDevices_GetDevice(this: *mut CImmDevices, lIndex: c_int) -> IMM_DEVICE_PTR;

	//
	// OPERATIONS
	//

	pub fn CImmDevices_AddDevice(this: *mut CImmDevices, pDevice: IMM_DEVICE_PTR);

	pub fn CImmDevices_CreateDevices(
		this: *mut CImmDevices,
		hinstApp: HINSTANCE,
		hwndApp: HWND,
		lMaximumDevices: c_int, // means "all" when -1
		typ: IMM_ENUMERATE,
		preference: IMM_ENUMERATE_PREFERENCE,
	) -> c_int;

	//
	// ------ PRIVATE INTERFACE ------
	//

	//
	// HELPERS
	//

	pub fn CImmDevices_enumerate_dx_devices(
		this: *mut CImmDevices,
		pIE: *mut CInitializeEnum,
	) -> BOOL;

	pub fn CImmDevices_enumerate_imm_devices(
		this: *mut CImmDevices,
		pIE: *mut CInitializeEnum,
	) -> BOOL;

	pub fn CImmDevices_clean_up(this: *mut CImmDevices);
}
