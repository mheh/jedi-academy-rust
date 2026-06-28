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

  FILE:        ImmRamp.h

  PURPOSE:    Base Ramp Force Class for Immersion Foundation Classes

  STARTED:    12/11/97

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

/// Stub for IMM_RAMPFORCE
#[repr(C)]
pub struct IMM_RAMPFORCE {
    // Stub: Ramp force data structure
}

// ================================================================
// Constants
// ================================================================

pub const IMM_RAMP_DEFAULT_DURATION: DWORD = 1000; // Milliseconds
pub const IMM_RAMP_DEFAULT_MAGNITUDE_START: LONG = 0;
pub const IMM_RAMP_DEFAULT_MAGNITUDE_END: LONG = 10000;

// Constants from ImmEffect.h
pub const IMM_EFFECTTYPE_RAMPFORCE: DWORD = 0x00000008;
pub const IMM_EFFECT_DONT_CHANGE: DWORD = 0xFFFFFFFF;
pub const IMM_EFFECT_DONT_CHANGE_PTR: *const core::ffi::c_void = core::ptr::null();
pub const IMM_EFFECT_DEFAULT_DIRECTION_X: LONG = 1;
pub const IMM_EFFECT_DEFAULT_DIRECTION_Y: LONG = 1;
pub const IMM_EFFECT_DEFAULT_ANGLE: LONG = 0;

/// Stub GUID constant for Ramp Force (from ImmBaseTypes.h)
pub const GUID_Imm_RampForce: GUID = GUID {
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
// CImmRamp
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

/// Base Ramp Force Class for Immersion Foundation Classes
#[repr(C)]
pub struct CImmRamp {
    //
    // INTERNAL DATA
    //

    pub m_RampForce: IMM_RAMPFORCE,
}

impl CImmRamp {
    //
    // CONSTRUCTOR/DESTRUCTOR
    //

    /// Constructor
    pub fn new() -> Self {
        CImmRamp {
            m_RampForce: unsafe { core::mem::zeroed() },
        }
    }

    // Destructor (implicit via Drop or manual implementation in derived classes)

    //
    // ATTRIBUTES
    //

    pub fn GetIsCompatibleGUID(&self, guid: &mut GUID) -> BOOL {
        unsafe { IsEqualGUID(*guid, GUID_Imm_RampForce) }
    }

    pub fn GetEffectType(&self) -> DWORD {
        IMM_EFFECTTYPE_RAMPFORCE
    }

    pub fn ChangeParameters(
        &mut self,
        lDirectionX: LONG,
        lDirectionY: LONG,
        dwDuration: DWORD,
        lMagStart: LONG,
        lMagEnd: LONG,
        pEnvelope: LPIMM_ENVELOPE,
    ) -> BOOL {
        // Stub: implementation unported (defined in .cpp source)
        0
    }

    pub fn ChangeParametersPolar(
        &mut self,
        lAngle: LONG,
        dwDuration: DWORD,
        lMagStart: LONG,
        lMagEnd: LONG,
        pEnvelope: LPIMM_ENVELOPE,
    ) -> BOOL {
        // Stub: implementation unported (defined in .cpp source)
        0
    }

    pub fn ChangeStartMagnitude(&mut self, lMagStart: LONG) -> BOOL {
        // Stub: implementation unported (defined in .cpp source)
        0
    }

    pub fn ChangeEndMagnitude(&mut self, lMagEnd: LONG) -> BOOL {
        // Stub: implementation unported (defined in .cpp source)
        0
    }

    pub fn GetStartMagnitude(&self, lMagStart: &mut LONG) -> BOOL {
        // Stub: implementation unported (defined in .cpp source)
        0
    }

    pub fn GetEndMagnitude(&self, lMagEnd: &mut LONG) -> BOOL {
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
        dwFlags: DWORD,
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
        lMagStart: LONG,
        lMagEnd: LONG,
        pEnvelope: LPIMM_ENVELOPE,
        dwFlags: DWORD,
    ) -> BOOL {
        // Stub: implementation unported (defined in .cpp source)
        0
    }

    pub fn InitializePolar(
        &mut self,
        pDevice: *mut CImmDevice,
        lAngle: LONG,
        dwDuration: DWORD,
        lMagStart: LONG,
        lMagEnd: LONG,
        pEnvelope: LPIMM_ENVELOPE,
        dwFlags: DWORD,
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
        lMagStart: LONG,
        lMagEnd: LONG,
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
        lMagStart: LONG,
        lMagEnd: LONG,
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
