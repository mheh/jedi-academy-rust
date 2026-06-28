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

  FILE:		ImmEnclosure.h

  PURPOSE:	Base Enclosure Class for Immersion Foundation Classes

  STARTED:	10/29/97

  NOTES/REVISIONS:
     3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
	 3/2/99 jrm: Added GetIsCompatibleGUID
	 3/15/99 jrm: __declspec(dllimport/dllexport) the whole class
	 11/15/99 sdr (Steve Rank): Converted to IFC

**********************************************************************/

use core::ffi::{c_int, c_long, c_char, c_void};

//================================================================
// Constants
//================================================================

pub const IMM_ENCLOSURE_DEFAULT_STIFFNESS: c_long = 5000;
pub const IMM_ENCLOSURE_DEFAULT_SATURATION: c_long = 10000;
pub const IMM_ENCLOSURE_DEFAULT_WIDTH: c_long = 10;
pub const IMM_ENCLOSURE_HEIGHT_AUTO: c_long = 0xFFFFFFFF as c_long;
pub const IMM_ENCLOSURE_DEFAULT_HEIGHT: c_long = IMM_ENCLOSURE_HEIGHT_AUTO;
pub const IMM_ENCLOSURE_WALL_WIDTH_AUTO: c_long = 0xFFFFFFFF as c_long;
pub const IMM_ENCLOSURE_DEFAULT_WALL_WIDTH: c_long = IMM_ENCLOSURE_WALL_WIDTH_AUTO;
pub const IMM_ENCLOSURE_DEFAULT_STIFFNESS_MASK: c_long = 0; // IMM_STIFF_ANYWALL - stub
pub const IMM_ENCLOSURE_DEFAULT_CLIPPING_MASK: c_long = 0; // IMM_CLIP_NONE - stub

pub const IMM_ENCLOSURE_DEFAULT_CENTER_POINT: c_long = 0; // IMM_EFFECT_MOUSE_POS_AT_START - stub

//================================================================
// Type Stubs for unported dependencies
//================================================================

// Windows GUID type
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

// Windows POINT type
#[repr(C)]
#[derive(Copy, Clone)]
pub struct POINT {
    pub x: c_long,
    pub y: c_long,
}

// Windows RECT type
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RECT {
    pub left: c_long,
    pub top: c_long,
    pub right: c_long,
    pub bottom: c_long,
}

// Windows BOOL type
pub type BOOL = c_int;

// Windows DWORD type (unsigned 32-bit)
pub type DWORD = u32;

// Windows HRESULT type
pub type HRESULT = i32;

// LPCRECT is const RECT pointer
pub type LPCRECT = *const RECT;

// TCHAR is character type
pub type TCHAR = c_char;

// Forward declaration stubs for unported Immersion types
#[repr(C)]
pub struct CImmDevice {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CImmEffect {
    _opaque: [u8; 0],
}

// IMM_EFFECT structure - stub
#[repr(C)]
pub struct IMM_EFFECT {
    _opaque: [u8; 0],
}

// IMM_ENCLOSURE structure - stub
#[repr(C)]
pub struct IMM_ENCLOSURE {
    _opaque: [u8; 0],
}

// GUID_Imm_Enclosure - stub global
pub static GUID_Imm_Enclosure: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

// Constants for effect initialization
pub const IMM_EFFECT_DONT_CHANGE: c_long = -1;
pub const IMM_EFFECT_DEFAULT_ANGLE: c_long = 0;
pub const IMM_EFFECTTYPE_ENCLOSURE: c_long = 4;

// Helper function stub for GUID comparison
pub fn IsEqualGUID(guid1: &GUID, guid2: &GUID) -> BOOL {
    if guid1.Data1 == guid2.Data1
        && guid1.Data2 == guid2.Data2
        && guid1.Data3 == guid2.Data3
        && guid1.Data4 == guid2.Data4
    {
        1
    } else {
        0
    }
}

//================================================================
// CImmEnclosure
//================================================================

//
// ------ PUBLIC INTERFACE ------
//

#[repr(C)]
#[allow(non_snake_case)]
pub struct CImmEnclosure {
    // Needed for co-ordinating events for Enclosures/Ellipes and the inside effects.
    pub m_pInsideEffect: *mut CImmEffect,
    pub m_enclosure: IMM_ENCLOSURE,
    pub m_bUseMousePosAtStart: BOOL,
}

impl CImmEnclosure {
    // Constructor
    pub fn new() -> Self {
        CImmEnclosure {
            m_pInsideEffect: core::ptr::null_mut(),
            m_enclosure: IMM_ENCLOSURE { _opaque: [] },
            m_bUseMousePosAtStart: 0,
        }
    }

    // Destructor
    pub fn delete(&mut self) {
        // cleanup if needed
    }

    //
    // ATTRIBUTES
    //

    pub fn GetIsCompatibleGUID(&self, guid: &GUID) -> BOOL {
        IsEqualGUID(guid, &GUID_Imm_Enclosure)
    }

    pub fn GetEffectType(&self) -> c_long {
        IMM_EFFECTTYPE_ENCLOSURE
    }

    pub fn ChangeParameters(
        &mut self,
        pntCenter: POINT,
        dwWidth: c_long,
        dwHeight: c_long,
        lTopAndBottomWallStiffness: c_long,
        lLeftAndRightWallStiffness: c_long,
        dwTopAndBottomWallWallWidth: c_long,
        dwLeftAndRightWallWallWidth: c_long,
        dwTopAndBottomWallSaturation: c_long,
        dwLeftAndRightWallSaturation: c_long,
        dwStiffnessMask: c_long,
        dwClippingMask: c_long,
        pInsideEffect: *mut CImmEffect,
        lAngle: c_long,
    ) -> BOOL {
        0 // stub
    }

    pub fn ChangeParameters_rect(
        &mut self,
        pRectOutside: LPCRECT,
        lTopAndBottomWallStiffness: c_long,
        lLeftAndRightWallStiffness: c_long,
        dwTopAndBottomWallWallThickness: c_long,
        dwLeftAndRightWallWallThickness: c_long,
        dwTopAndBottomWallSaturation: c_long,
        dwLeftAndRightWallSaturation: c_long,
        dwStiffnessMask: c_long,
        dwClippingMask: c_long,
        pInsideEffect: *mut CImmEffect,
        lAngle: c_long,
    ) -> BOOL {
        0 // stub
    }

    pub fn ChangeTopAndBottomWallStiffness(&mut self, lTopAndBottomWallStiffness: c_long) -> BOOL {
        0 // stub
    }

    pub fn ChangeLeftAndRightWallStiffness(&mut self, lLeftAndRightWallStiffness: c_long) -> BOOL {
        0 // stub
    }

    pub fn ChangeTopAndBottomWallThickness(&mut self, dwTopAndBottomWallThickness: c_long) -> BOOL {
        0 // stub
    }

    pub fn ChangeLeftAndRightWallThickness(&mut self, dwLeftAndRightWallThickness: c_long) -> BOOL {
        0 // stub
    }

    pub fn ChangeTopAndBottomWallSaturation(&mut self, dwTopAndBottomWallSaturation: c_long) -> BOOL {
        0 // stub
    }

    pub fn ChangeLeftAndRightWallSaturation(&mut self, dwLeftAndRightWallSaturation: c_long) -> BOOL {
        0 // stub
    }

    pub fn ChangeStiffnessMask(&mut self, dwStiffnessMask: c_long) -> BOOL {
        0 // stub
    }

    pub fn ChangeClippingMask(&mut self, dwClippingMask: c_long) -> BOOL {
        0 // stub
    }

    pub fn ChangeInsideEffect(&mut self, pInsideEffect: *mut CImmEffect) -> BOOL {
        0 // stub
    }

    pub fn ChangeRect(&mut self, pRect: LPCRECT) -> BOOL {
        0 // stub
    }

    pub fn ChangeCenter(&mut self, pntCenter: POINT) -> BOOL {
        0 // stub
    }

    pub fn ChangeCenter_xy(&mut self, x: c_long, y: c_long) -> BOOL {
        0 // stub
    }

    pub fn ShowRect(&mut self, bRectOn: BOOL) -> BOOL {
        0 // stub
    }

    pub fn GetTopAndBottomWallStiffness(&self, lTopAndBottomWallStiffness: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetLeftAndRightWallStiffness(&self, lLeftAndRightWallStiffness: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetTopAndBottomWallThickness(&self, dwTopAndBottomWallThickness: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetLeftAndRightWallThickness(&self, dwLeftAndRightWallThickness: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetTopAndBottomWallSaturation(&self, dwTopAndBottomWallSaturation: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetLeftAndRightWallSaturation(&self, dwLeftAndRightWallSaturation: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetStiffnessMask(&self, dwStiffnessMask: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetClippingMask(&self, dwClippingMask: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetRect(&self, pRect: *mut RECT) -> BOOL {
        0 // stub
    }

    pub fn GetCenter(&self, pntCenter: &mut POINT) -> BOOL {
        0 // stub
    }

    pub fn GetCenter_xy(&self, x: &mut c_long, y: &mut c_long) -> BOOL {
        0 // stub
    }

    pub fn GetInsideEffect(&self) -> *mut CImmEffect {
        core::ptr::null_mut()
    }

    //
    // OPERATIONS
    //

    pub fn Initialize(
        &mut self,
        pDevice: *mut CImmDevice,
        effect: &IMM_EFFECT,
        dwNoDownload: c_long,
    ) -> BOOL {
        0 // stub
    }

    pub fn Initialize_defaults(
        &mut self,
        pDevice: *mut CImmDevice,
        dwWidth: c_long,
        dwHeight: c_long,
        lTopAndBottomWallStiffness: c_long,
        lLeftAndRightWallStiffness: c_long,
        dwTopAndBottomWallWallWidth: c_long,
        dwLeftAndRightWallWallWidth: c_long,
        dwTopAndBottomWallSaturation: c_long,
        dwLeftAndRightWallSaturation: c_long,
        dwStiffnessMask: c_long,
        dwClippingMask: c_long,
        pntCenter: POINT,
        pInsideEffect: *mut CImmEffect,
        lAngle: c_long,
        dwNoDownload: c_long,
    ) -> BOOL {
        0 // stub
    }

    pub fn Initialize_rect(
        &mut self,
        pDevice: *mut CImmDevice,
        pRectOutside: LPCRECT,
        lTopAndBottomWallStiffness: c_long,
        lLeftAndRightWallStiffness: c_long,
        dwTopAndBottomWallWallWidth: c_long,
        dwLeftAndRightWallWallWidth: c_long,
        dwTopAndBottomWallSaturation: c_long,
        dwLeftAndRightWallSaturation: c_long,
        dwStiffnessMask: c_long,
        dwClippingMask: c_long,
        pInsideEffect: *mut CImmEffect,
        lAngle: c_long,
        dwNoDownload: c_long,
    ) -> BOOL {
        0 // stub
    }

    pub fn Start(
        &mut self,
        dwIterations: c_long,
        dwFlags: c_long,
        bAllowStartDelayEmulation: BOOL,
    ) -> BOOL {
        0 // stub
    }

    pub fn Stop(&mut self) -> BOOL {
        0 // stub
    }

    pub fn Unload(&mut self) -> HRESULT {
        0
    }

    pub fn Reload(&mut self) {
        // stub
    }

    //
    // ------ PRIVATE INTERFACE ------
    //

    //
    // HELPERS
    //

    fn set_parameters(
        &mut self,
        pRectOutside: LPCRECT,
        lTopAndBottomWallStiffness: c_long,
        lLeftAndRightWallStiffness: c_long,
        dwTopAndBottomWallWallWidth: c_long,
        dwLeftAndRightWallWallWidth: c_long,
        dwTopAndBottomWallSaturation: c_long,
        dwLeftAndRightWallSaturation: c_long,
        dwStiffnessMask: c_long,
        dwClippingMask: c_long,
        pInsideEffect: *mut CImmEffect,
        lAngle: c_long,
    ) -> BOOL {
        0 // stub
    }

    fn change_parameters(
        &mut self,
        prectBoundary: LPCRECT,
        lTopAndBottomWallStiffness: c_long,
        lLeftAndRightWallStiffness: c_long,
        dwTopAndBottomWallThickness: c_long,
        dwLeftAndRightWallThickness: c_long,
        dwTopAndBottomWallSaturation: c_long,
        dwLeftAndRightWallSaturation: c_long,
        dwStiffnessMask: c_long,
        dwClippingMask: c_long,
        pInsideEffect: *mut CImmEffect,
        lAngle: c_long,
    ) -> c_long {
        0 // stub
    }

    fn buffer_ifr_data(&mut self, pData: *mut TCHAR) -> c_int {
        0 // stub
    }
}
