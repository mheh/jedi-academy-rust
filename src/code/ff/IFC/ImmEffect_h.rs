// Copyright (c) 1997 - 2000 Immersion Corporation
//
// Permission to use, copy, modify, distribute, and sell this
// software and its documentation may be granted without fee;
// interested parties are encouraged to request permission from
//     Immersion Corporation
//     801 Fox Lane
//     San Jose, CA 95131
//     408-467-1900
//
// IMMERSION DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE,
// INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS.
// IN NO EVENT SHALL IMMERSION BE LIABLE FOR ANY SPECIAL, INDIRECT OR
// CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
// NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//
// FILE:		ImmEffect.h
//
// PURPOSE:	Immersion Foundation Classes Base Effect
//
// STARTED:	Oct.10.97
//
// NOTES/REVISIONS:
//    Mar.02.99 jrm (Jeff Mallett): Force-->Feel renaming
//	   Mar.02.99 jrm: Added GetIsCompatibleGUID and feel_to_DI_GUID
//	   Mar.15.99 jrm: __declspec(dllimport/dllexport) the whole class
//	   Nov.15.99 efw (Evan Wies): Converted to IFC

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};

// Minimal type stubs for Windows types needed for struct layout
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GUID {
    pub Data1: c_uint,
    pub Data2: c_uint,
    pub Data3: c_uint,
    pub Data4: [c_char; 8],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct POINT {
    pub x: c_long,
    pub y: c_long,
}

// Forward declaration stub for CImmProject (defined elsewhere)
#[repr(C)]
pub struct CImmProject;

// Forward declarations for interface pointers (from ImmDevice.h)
pub type LPIIMM_DEVICE = *mut c_void;
pub type LPIIMM_EFFECT = *mut c_void;
pub type LPIMM_ENVELOPE = *mut c_void;
pub type LPDIEFFECT = *mut c_void;

// Basic Windows type aliases
pub type DWORD = c_uint;
pub type LONG = c_long;
pub type BOOL = c_int;
pub type LPCSTR = *const c_char;

// ================================================================
// Constants
// ================================================================

pub const IMM_EFFECT_AXIS_X: c_int = 1;
pub const IMM_EFFECT_AXIS_Y: c_int = 2;
pub const IMM_EFFECT_AXIS_BOTH: c_int = 3;
pub const IMM_EFFECT_AXIS_DIRECTIONAL: c_int = 4;
pub const IMM_EFFECT_DONT_CHANGE: LONG = -2147483648i32;  // MINLONG
pub const IMM_EFFECT_DONT_CHANGE_PTR: DWORD = 0xFFFFFFFF;  // MAXDWORD

pub const IMM_EFFECT_DONT_CHANGE_POINT: POINT = POINT {
    x: -1i32,
    y: -1i32,
};
pub const IMM_EFFECT_MOUSE_POS_AT_START: POINT = POINT {
    x: 2147483647i32,  // MAXLONG
    y: 2147483647i32,  // MAXLONG
};

pub const IMM_EFFECT_DEFAULT_ENVELOPE: LPIMM_ENVELOPE = core::ptr::null_mut();  // NULL
pub const IMM_EFFECT_DEFAULT_DIRECTION_X: c_int = 1;
pub const IMM_EFFECT_DEFAULT_DIRECTION_Y: c_int = 1;
pub const IMM_EFFECT_DEFAULT_ANGLE: c_int = 0;

// GENERIC_EFFECT_PTR
// This is really a pointer to a child of CImmEffect.
pub type GENERIC_EFFECT_PTR = *mut CImmEffect;

// ================================================================
// CImmEffect
// ================================================================

// Note: In the original C++, this was decorated with DLLIFC (__declspec(dllimport/dllexport))
// which is Windows-specific DLL import/export syntax. Rust does not have a direct equivalent.

#[repr(C)]
pub struct CImmEffect {
    //
    // INTERNAL DATA
    //

    pub m_Effect: IMM_EFFECT,
    pub m_dwaAxes: [DWORD; 2],
    pub m_laDirections: [LONG; 2],
    pub m_Envelope: IMM_ENVELOPE,

    pub m_guidEffect: GUID,
    pub m_bIsPlaying: BOOL,
    pub m_dwDeviceType: DWORD,
    pub m_piImmDevice: LPIIMM_DEVICE,  // Might also be holding LPDIRECTINPUTDEVICE2
    pub m_piImmEffect: LPIIMM_EFFECT,
    pub m_cAxes: DWORD,  // Number of axes
    pub m_dwNoDownload: DWORD,
    pub m_dwIterations: DWORD,
    pub m_lpszName: *mut c_char,  // Name of this effect primative

    // Needed for co-ordinating events for Enclosures/Ellipes and the inside effects.
    pub m_bIsInsideEffect: BOOL,
    pub m_pOutsideEffect: *mut CImmEffect,

    #[cfg(feature = "IFC_START_DELAY")]
    // Prevents access to dangling pointer when this is deleted
    // All relevent code may be removed when all hardware and drivers support start delay
    pub m_ppTimerRef: *mut *mut CImmEffect,  // pointer to pointer to this.

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub m_pImmDevice: *mut CImmDevice,  // ### Use instead of m_piImmDevice

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub m_CacheState: ECacheState,  // effect's status in the cache

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub m_bInCurrentSuite: BOOL,  // is the effect in the currently loaded suite?

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub m_Priority: i16,  // Priority within suite: higher number is higher priority

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub m_dwLastStarted: DWORD,  // when last started (0 = never) or when param change made on device

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub m_dwLastStopped: DWORD,  // when last stopped (0 = not since last start)

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub m_dwLastLoaded: DWORD,  // when last loaded with CImmEffectSuite::Load or Create
}

// Stub types for conditional compilation features
#[cfg(feature = "IFC_EFFECT_CACHING")]
#[repr(C)]
pub enum ECacheState {
    // Placeholder - actual values would come from ImmDevice.h
    Unknown = 0,
}

#[cfg(feature = "IFC_EFFECT_CACHING")]
#[repr(C)]
pub struct CImmDevice;

// Minimal stub for IMM_EFFECT structure (from ImmBaseTypes.h)
#[repr(C)]
pub struct IMM_EFFECT {
    // Placeholder structure - actual definition in ImmBaseTypes.h
    pub dwSize: DWORD,
}

// Minimal stub for IMM_ENVELOPE structure
#[repr(C)]
pub struct IMM_ENVELOPE {
    // Placeholder structure
    pub dwSize: DWORD,
}

impl CImmEffect {
    //
    // CONSTRUCTOR/DESTRUCTOR
    //

    // Constructor
    pub fn new(rguidEffect: &GUID) -> Self {
        todo!()
    }

    // Destructor (implicit in Rust, but documented here from original)
    // pub fn drop(self) { }

    //
    // ATTRIBUTES
    //

    pub fn GetEffect(&self) -> LPIIMM_EFFECT {
        self.m_piImmEffect
    }

    pub fn GetDevice(&self) -> *mut CImmDevice {
        #[cfg(feature = "IFC_EFFECT_CACHING")]
        {
            self.m_pImmDevice
        }
        #[cfg(not(feature = "IFC_EFFECT_CACHING"))]
        {
            core::ptr::null_mut()
        }
    }

    pub fn GetStatus(&mut self, pdwStatus: *mut DWORD) -> BOOL {
        todo!()
    }

    pub fn GetParameters(&mut self, Effect: &mut IMM_EFFECT) {
        todo!()
    }

    pub fn GetEnvelope(&mut self, pEnvelope: LPIMM_ENVELOPE) -> BOOL {
        todo!()
    }

    pub fn GetDuration(&mut self, dwDuration: &mut DWORD) -> BOOL {
        todo!()
    }

    pub fn GetGain(&mut self, dwGain: &mut DWORD) -> BOOL {
        todo!()
    }

    pub fn GetStartDelay(&mut self, dwStartDelay: &mut DWORD) -> BOOL {
        todo!()
    }

    pub fn GetTriggerButton(&mut self, dwTriggerButton: &mut DWORD) -> BOOL {
        todo!()
    }

    pub fn GetTriggerRepeatInterval(&mut self, dwTriggerRepeatInterval: &mut DWORD) -> BOOL {
        todo!()
    }

    pub fn GetDirection_XY(&mut self, lDirectionX: &mut LONG, lDirectionY: &mut LONG) -> BOOL {
        todo!()
    }

    pub fn GetDirection_Angle(&mut self, lAngle: &mut LONG) -> BOOL {
        todo!()
    }

    pub fn GetIterations(&mut self, dwIterations: &mut DWORD) -> BOOL {
        todo!()
    }

    pub fn GetGUID(&self) -> GUID {
        self.m_guidEffect
    }

    // Virtual method in original C++
    pub fn GetIsCompatibleGUID(&self, guid: &GUID) -> BOOL {
        1  // true
    }

    // Virtual method in original C++
    pub fn GetEffectType(&self) -> DWORD {
        0
    }

    pub fn GetName(&self) -> LPCSTR {
        self.m_lpszName
    }

    // Allocates an object of the correct IFC class from the given GUID
    pub fn NewObjectFromGUID(guid: &GUID) -> GENERIC_EFFECT_PTR {
        todo!()
    }

    pub fn ChangeBaseParams(
        &mut self,
        lDirectionX: LONG,
        lDirectionY: LONG,
        dwDuration: DWORD,
        pEnvelope: LPIMM_ENVELOPE,
        dwSamplePeriod: DWORD,
        dwGain: DWORD,
        dwTriggerButton: DWORD,
        dwTriggerRepeatInterval: DWORD,
        #[cfg(feature = "IFC_START_DELAY")]
        dwStartDelay: DWORD,
    ) -> BOOL {
        todo!()
    }

    pub fn ChangeBaseParamsPolar(
        &mut self,
        lAngle: LONG,
        dwDuration: DWORD,  // milliseconds
        pEnvelope: LPIMM_ENVELOPE,
        dwSamplePeriod: DWORD,
        dwGain: DWORD,
        dwTriggerButton: DWORD,
        dwTriggerRepeatInterval: DWORD,
        #[cfg(feature = "IFC_START_DELAY")]
        dwStartDelay: DWORD,  // milliseconds
    ) -> BOOL {
        todo!()
    }

    pub fn ChangeDirection_XY(&mut self, lDirectionX: LONG, lDirectionY: LONG) -> BOOL {
        todo!()
    }

    pub fn ChangeDirection_Angle(&mut self, lAngle: LONG) -> BOOL {
        todo!()
    }

    pub fn ChangeDuration(&mut self, dwDuration: DWORD) -> BOOL {
        todo!()
    }

    pub fn ChangeGain(&mut self, dwGain: DWORD) -> BOOL {
        todo!()
    }

    pub fn ChangeStartDelay(&mut self, dwStartDelay: DWORD) -> BOOL {
        todo!()
    }

    pub fn ChangeTriggerButton(&mut self, dwTriggerButton: DWORD) -> BOOL {
        todo!()
    }

    pub fn ChangeTriggerRepeatInterval(&mut self, dwTriggerRepeatInterval: DWORD) -> BOOL {
        todo!()
    }

    pub fn ChangeIterations(&mut self, dwIterations: DWORD) -> BOOL {
        todo!()
    }

    pub fn ChangeEnvelope_Fields(
        &mut self,
        dwAttackLevel: DWORD,
        dwAttackTime: DWORD,  // microseconds
        dwFadeLevel: DWORD,
        dwFadeTime: DWORD,  // microseconds
    ) -> BOOL {
        todo!()
    }

    pub fn ChangeEnvelope_Ptr(&mut self, pEnvelope: LPIMM_ENVELOPE) -> BOOL {
        todo!()
    }

    //
    // OPERATIONS
    //

    // Virtual method in original C++
    pub fn Initialize(
        &mut self,
        pDevice: *mut CImmDevice,
        effect: &IMM_EFFECT,
        dwNoDownload: DWORD,
    ) -> BOOL {
        todo!()
    }

    // Virtual method in original C++
    pub fn InitializeFromProject(
        &mut self,
        project: &mut CImmProject,
        lpszEffectName: LPCSTR,
        pDevice: *mut CImmDevice,
        dwNoDownload: DWORD,
    ) -> BOOL {
        todo!()
    }

    // Virtual method in original C++
    pub fn Start(
        &mut self,
        dwIterations: DWORD,
        dwFlags: DWORD,
        #[cfg(feature = "IFC_START_DELAY")]
        bAllowStartDelayEmulation: BOOL,
    ) -> BOOL {
        todo!()
    }

    // Virtual method in original C++
    pub fn Stop(&mut self) -> BOOL {
        todo!()
    }

    //
    // CACHING (conditional, requires IFC_EFFECT_CACHING feature)
    //

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn GetIsPlaying(&self) -> BOOL {
        todo!()
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn GetIsTriggered(&self) -> BOOL {
        todo!()
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn GetPriority(&self) -> i16 {
        self.m_Priority
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn SetPriority(&mut self, priority: i16) {
        self.m_Priority = priority;
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    // Virtual method in original C++
    pub fn Unload(&mut self) -> i32 {  // HRESULT
        todo!()
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    // Virtual method in original C++
    pub fn Reload(&mut self) {
        todo!()
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    // Althought public, this should only be used internally.
    pub fn set_outside_effect(&mut self, pImmOutsideEffect: *mut CImmEffect) -> BOOL {
        todo!()
    }

    pub fn get_is_inside_effect(&self) -> BOOL {
        self.m_bIsInsideEffect
    }

    //
    // HELPERS
    //

    #[cfg(feature = "IFC_START_DELAY")]
    fn EmulateStartDelay(&mut self, dwIterations: DWORD, dwNoDownload: DWORD) {
        todo!()
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    // Note: initialize needs to be called by CImmDevice (was public in conditional block)
    fn initialize(&mut self, pDevice: *mut CImmDevice, dwNoDownload: DWORD) -> BOOL {
        todo!()
    }

    #[cfg(not(feature = "IFC_EFFECT_CACHING"))]
    fn initialize(&mut self, pDevice: *mut CImmDevice, dwNoDownload: DWORD) -> BOOL {
        todo!()
    }

    fn set_parameters_on_device(&mut self, dwFlags: DWORD) -> i32 {  // HRESULT
        todo!()
    }

    fn set_name(&mut self, lpszName: *const c_char) -> BOOL {
        todo!()
    }

    fn imm_to_DI_GUID(&self, guid: &mut GUID) {
        todo!()
    }

    fn DI_to_imm_GUID(&self, guid: &mut GUID) {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }

    fn reset_effect_struct(&mut self) {
        todo!()
    }

    fn reset_device(&mut self) {
        todo!()
    }

    fn buffer_direction(&self, pData: *mut *mut c_char) {
        todo!()
    }

    fn buffer_long_param(
        &self,
        pData: *mut *mut c_char,
        lpszKey: LPCSTR,
        lDefault: c_long,
        lValue: c_long,
    ) {
        todo!()
    }

    fn buffer_dword_param(
        &self,
        pData: *mut *mut c_char,
        lpszKey: LPCSTR,
        dwDefault: DWORD,
        dwValue: DWORD,
    ) {
        todo!()
    }

    // Virtual method in original C++
    fn buffer_ifr_data(&mut self, pData: *mut c_char) -> c_int {
        todo!()
    }

    // Virtual method in original C++
    fn get_ffe_data(&mut self, pdiEffect: LPDIEFFECT) -> BOOL {
        todo!()
    }
}
