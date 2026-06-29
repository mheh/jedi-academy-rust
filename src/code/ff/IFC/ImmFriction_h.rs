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

  FILE:		ImmFriction.h

  PURPOSE:	Immersion Foundation Classes Friction Effect

  STARTED:	Dec.29.97

  NOTES/REVISIONS:
     Mar.02.99 jrm (Jeff Mallett): Force-->Feel renaming
     Mar.02.99 jrm: Added GetIsCompatibleGUID
     Mar.15.99 jrm: __declspec(dllimport/dllexport) the whole class
     Nov.15.99 efw (Evan Wies): Converted to IFC

**********************************************************************/

use core::ffi::{c_int, c_long};

// Local stub for unported CImmCondition base class
pub struct CImmCondition;

// Local stub for unported CImmDevice type
pub struct CImmDevice;

// Local stub for Windows GUID type (from windows.h)
#[repr(C)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

// Windows type mappings
pub type BOOL = c_int;
pub type DWORD = u32;
pub type LONG = c_long;

// ================================================================
// Constants
// ================================================================

pub const IMM_FRICTION_DEFAULT_COEFFICIENT: DWORD = 2500;
pub const IMM_FRICTION_DEFAULT_MIN_VELOCITY: DWORD = 0;

// Local stubs for constants from ImmCondition.h
pub const IMM_EFFECT_DONT_CHANGE: DWORD = u32::MAX;
pub const IMM_EFFECT_AXIS_BOTH: DWORD = 3;
pub const IMM_EFFECT_DEFAULT_DIRECTION_X: LONG = 0;
pub const IMM_EFFECT_DEFAULT_DIRECTION_Y: LONG = 0;

// Local stub for GUID constant from ImmCondition.h
pub static GUID_Imm_Friction: GUID = GUID {
    data1: 0,
    data2: 0,
    data3: 0,
    data4: [0; 8],
};

// ================================================================
// CImmFriction
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

#[repr(C)]
pub struct CImmFriction {
    // Base class member (opaque)
    _base: CImmCondition,
}

impl CImmFriction {
    //
    // CONSTRUCTOR/DESTRUCTOR
    //

    // Constructor
    pub fn new() -> Self {
        CImmFriction {
            _base: CImmCondition,
        }
    }

    //
    // ATTRIBUTES
    //

    pub fn GetIsCompatibleGUID(&self, guid: &GUID) -> BOOL {
        // Stub: IsEqualGUID would compare GUIDs
        // For now, return comparison (simplified - actual implementation would check GUID_Imm_Friction)
        if guid.data1 == GUID_Imm_Friction.data1
            && guid.data2 == GUID_Imm_Friction.data2
            && guid.data3 == GUID_Imm_Friction.data3
            && guid.data4 == GUID_Imm_Friction.data4
        {
            1 // TRUE
        } else {
            0 // FALSE
        }
    }

    pub fn ChangeParameters(
        &mut self,
        dwCoefficient: DWORD,
        dwMinVelocity: DWORD,
        lDirectionX: LONG,
        lDirectionY: LONG,
    ) -> BOOL {
        // Method body to be implemented with actual force feedback logic
        0
    }

    pub fn ChangeParametersPolar(
        &mut self,
        dwCoefficient: DWORD,
        dwMinVelocity: DWORD,
        lAngle: LONG,
    ) -> BOOL {
        // Method body to be implemented with actual force feedback logic
        0
    }

    pub fn ChangeMinVelocityX(&mut self, dwMinVelocity: DWORD) -> BOOL {
        0
    }

    pub fn ChangeMinVelocityY(&mut self, dwMinVelocity: DWORD) -> BOOL {
        0
    }

    // For setting both axes to the same value
    pub fn ChangeMinVelocity(&mut self, dwMinVelocity: DWORD) -> BOOL {
        0
    }

    pub fn GetMinVelocityX(&self, dwMinVelocity: &mut DWORD) -> BOOL {
        0
    }

    pub fn GetMinVelocityY(&self, dwMinVelocity: &mut DWORD) -> BOOL {
        0
    }

    //
    // OPERATIONS
    //

    pub fn Initialize(
        &mut self,
        pDevice: *mut CImmDevice,
        dwCoefficient: DWORD,
        dwMinVelocity: DWORD,
        dwfAxis: DWORD,
        lDirectionX: LONG,
        lDirectionY: LONG,
        dwNoDownload: DWORD,
    ) -> BOOL {
        // Method body to be implemented with actual initialization logic
        0
    }

    pub fn InitializePolar(
        &mut self,
        pDevice: *mut CImmDevice,
        dwCoefficient: DWORD,
        dwMinVelocity: DWORD,
        lAngle: LONG,
        dwNoDownload: DWORD,
    ) -> BOOL {
        // Method body to be implemented with actual initialization logic
        0
    }
}

//
// ------ PRIVATE INTERFACE ------
//

// HELPERS
// (none in original)

// INTERNAL DATA
// (none declared in original)
