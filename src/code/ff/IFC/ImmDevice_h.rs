/**
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

FILE:       ImmDevice.h

PURPOSE:    Abstract Base Device Class for Force Foundation Classes

STARTED:    10/10/97

NOTES/REVISIONS:
   3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
   3/15/99 jrm: __declspec(dllimport/dllexport) the whole class
   3/16/99 jrm: Made abstract. Moved functionality to CImmMouse/CImmDXDevice
*/

use core::ffi::{c_char, c_int, c_void};

// ================================================================
// Device and Technology types
// ================================================================

// Company IDs
pub const IMM_OTHERPARTNER: u32 = 0x01000000;
pub const IMM_IMMERSION: u32 = 0x02000000;
pub const IMM_ACTLABS: u32 = 0x03000000;
pub const IMM_ANKO: u32 = 0x04000000;
pub const IMM_AVB: u32 = 0x05000000;
pub const IMM_BOEDER: u32 = 0x06000000;
pub const IMM_CHPRODUCTS: u32 = 0x07000000;
pub const IMM_CHIC: u32 = 0x08000000;
pub const IMM_GUILLEMOT: u32 = 0x09000000;
pub const IMM_GENIUS: u32 = 0x0a000000;
pub const IMM_HAPP: u32 = 0x0b000000;
pub const IMM_INTERACT: u32 = 0x0c000000;
pub const IMM_INTERACTIVEIO: u32 = 0x0d000000;
pub const IMM_KYE: u32 = 0x0e000000;
pub const IMM_LMP: u32 = 0x0f000000;
pub const IMM_LOGITECH: u32 = 0x10000000;
pub const IMM_MADCATZ: u32 = 0x11000000;
pub const IMM_MICROSOFT: u32 = 0x12000000;
pub const IMM_PADIX: u32 = 0x13000000;
pub const IMM_PRIMAX: u32 = 0x14000000;
pub const IMM_QUANTUM3D: u32 = 0x15000000;
pub const IMM_ROCKFIRE: u32 = 0x16000000;
pub const IMM_SCT: u32 = 0x17000000;
pub const IMM_SMELECTRONIC: u32 = 0x18000000;
pub const IMM_SYSGRATION: u32 = 0x19000000;
pub const IMM_THRUSTMASTER: u32 = 0x1a000000;
pub const IMM_TRUST: u32 = 0x1b000000;
pub const IMM_VIKINGS: u32 = 0x1c000000;

// Device IDs
pub const IMM_OTHERDEVICE: u32 = 0x00000001;
pub const IMM_JOYSTICK: u32 = 0x00000002;
pub const IMM_WHEEL: u32 = 0x00000003;
pub const IMM_GAMEPAD: u32 = 0x00000004;
pub const IMM_ABSMOUSE: u32 = 0x00000005;
pub const IMM_RELMOUSE: u32 = 0x00000006;

// Technology IDs
// Note that these are bit masks
pub const IMM_OTHERTECH: u32 = 0x00000001;
pub const IMM_FULLFF: u32 = 0x00000002;
pub const IMM_IHDFF: u32 = 0x00000004;
pub const IMM_VIBROFF: u32 = 0x00000008;

// Helper macro for combining high and low words into a DWORD
// Equivalent to MAKELONG(low, high) = ((high << 16) | (low & 0xFFFF))
const fn MAKELONG(high: u32, low: u32) -> u32 {
    (high << 16) | (low & 0xFFFF)
}

// Product Types (not to be confused with product GUIDs)
pub const IMM_JOYSTICK_FULLFF: u32 = MAKELONG(IMM_FULLFF, IMM_JOYSTICK);
pub const IMM_WHEEL_FULLFF: u32 = MAKELONG(IMM_FULLFF, IMM_WHEEL);
pub const IMM_GAMEPAD_FULLFF: u32 = MAKELONG(IMM_FULLFF, IMM_GAMEPAD);
pub const IMM_ABSMOUSE_FULLFF: u32 = MAKELONG(IMM_FULLFF, IMM_ABSMOUSE);

pub const IMM_JOYSTICK_IHDFF: u32 = MAKELONG(IMM_IHDFF, IMM_JOYSTICK);
pub const IMM_WHEEL_IHDFF: u32 = MAKELONG(IMM_IHDFF, IMM_WHEEL);
pub const IMM_GAMEPAD_IHDFF: u32 = MAKELONG(IMM_IHDFF, IMM_GAMEPAD);
pub const IMM_RELMOUSE_IHDFF: u32 = MAKELONG(IMM_IHDFF, IMM_RELMOUSE);

pub const IMM_JOYSTICK_VIBROFF: u32 = MAKELONG(IMM_VIBROFF, IMM_JOYSTICK);
pub const IMM_WHEEL_VIBROFF: u32 = MAKELONG(IMM_VIBROFF, IMM_WHEEL);
pub const IMM_GAMEPAD_VIBROFF: u32 = MAKELONG(IMM_VIBROFF, IMM_GAMEPAD);
pub const IMM_RELMOUSE_VIBROFF: u32 = MAKELONG(IMM_VIBROFF, IMM_RELMOUSE);

// ================================================================
// CImmDevice
// ================================================================

//
// ------ PUBLIC INTERFACE ------
//

/// Abstract Base Device Class for Force Foundation Classes
#[repr(C)]
pub struct CImmDevice {
    //
    // INTERNAL DATA
    //

    // Performs device preparation by setting the device's parameters
    pub m_bInitialized: c_int,
    pub m_dwDeviceType: u32,
    pub m_guidDevice: crate::code::ff::IFC::FeelitAPI_h::GUID,
    pub m_bGuidValid: c_int,
    pub m_dwProductType: u32,
}

impl CImmDevice {
    //
    // CONSTRUCTOR/DESTRUCTOR
    //

    /// Constructor
    pub fn CImmDevice() -> Self {
        // Stub implementation - actual construction would be in C++ subclass
        Self {
            m_bInitialized: 0,
            m_dwDeviceType: 0,
            m_guidDevice: crate::code::ff::IFC::FeelitAPI_h::GUID {
                data1: 0,
                data2: 0,
                data3: 0,
                data4: [0; 8],
            },
            m_bGuidValid: 0,
            m_dwProductType: 0,
        }
    }

    /// Destructor
    /// virtual ~CImmDevice();
    pub fn destroy(&mut self) {
        // Stub - actual implementation in subclass
    }

    //
    // ATTRIBUTES
    //

    /// virtual LPIIMM_API GetAPI() = 0; // pure virtual function
    pub fn GetAPI(&self) -> *mut c_void {
        // Pure virtual - must be overridden in subclass
        std::ptr::null_mut()
    }

    /// virtual LPIIMM_DEVICE GetDevice() = 0; // pure virtual function
    /// Will actually return LPDIRECTINPUTDEVICE2 for DX supported device
    pub fn GetDevice(&self) -> *mut c_void {
        // Pure virtual - must be overridden in subclass
        std::ptr::null_mut()
    }

    pub fn GetDeviceType(&self) -> u32 {
        self.m_dwDeviceType
    }

    /// virtual DWORD GetProductType() = 0;
    pub fn GetProductType(&self) -> u32 {
        // Pure virtual - must be overridden in subclass
        0
    }

    /// virtual BOOL GetDriverVersion(
    ///     DWORD &dwFFDriverVersion,
    ///     DWORD &dwFirmwareRevision,
    ///     DWORD &dwHardwareRevision)
    ///     = 0;
    pub fn GetDriverVersion(
        &self,
        dwFFDriverVersion: &mut u32,
        dwFirmwareRevision: &mut u32,
        dwHardwareRevision: &mut u32,
    ) -> c_int {
        // Pure virtual - must be overridden in subclass
        0
    }

    /// virtual int GetProductName(LPTSTR lpszProductName, int nMaxCount) = 0;
    pub fn GetProductName(&self, lpszProductName: *mut c_char, nMaxCount: c_int) -> c_int {
        // Pure virtual - must be overridden in subclass
        0
    }

    /// virtual int GetProductGUIDString(LPTSTR lpszGUID, int nMaxCount) = 0;
    pub fn GetProductGUIDString(&self, lpszGUID: *mut c_char, nMaxCount: c_int) -> c_int {
        // Pure virtual - must be overridden in subclass
        0
    }

    /// virtual GUID GetProductGUID() = 0;
    pub fn GetProductGUID(&self) -> crate::code::ff::IFC::FeelitAPI_h::GUID {
        // Pure virtual - must be overridden in subclass
        crate::code::ff::IFC::FeelitAPI_h::GUID {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: [0; 8],
        }
    }

    //
    // OPERATIONS
    //

    /// virtual BOOL GetCurrentPosition( long &lXPos, long &lYPos ) = 0; // pure virtual function
    pub fn GetCurrentPosition(&self, lXPos: &mut c_int, lYPos: &mut c_int) -> c_int {
        // Pure virtual - must be overridden in subclass
        0
    }

    /// virtual BOOL
    /// ChangeScreenResolution(
    ///     BOOL bAutoSet,
    ///     DWORD dwXScreenSize = 0,
    ///     DWORD dwYScreenSize = 0
    ///     );
    pub fn ChangeScreenResolution(
        &mut self,
        bAutoSet: c_int,
        dwXScreenSize: u32,
        dwYScreenSize: u32,
    ) -> c_int {
        // Default implementation - can be overridden in subclass
        0
    }

    /// The default state is using standard Win32 Mouse messages (e.g., WM_MOUSEMOVE)
    /// and functions (e.g, GetCursorPos).  Call only to switch to relative mode
    /// if not using standard Win32 Mouse services (e.g., DirectInput) for mouse
    /// input.
    pub fn UsesWin32MouseServices(&mut self, bWin32MouseServ: c_int) -> c_int {
        // Default implementation
        0
    }

    /// Another syntax for SwitchToAbsoluteMode.
    /// The default is Absolute mode.  Call only to switch to Relative mode or
    /// to switch back to Absolute mode.
    pub fn SwitchToAbsoluteMode(&mut self, bAbsMode: c_int) -> c_int {
        // Default implementation - can be overridden in subclass
        0
    }

    //
    // STATIC OPERATIONS
    //

    /// static BOOL GetIFCVersion(DWORD &dwMajor, DWORD &dwMinor, DWORD &dwBuild, DWORD &dwBuildMinor);
    pub fn GetIFCVersion(
        dwMajor: &mut u32,
        dwMinor: &mut u32,
        dwBuild: &mut u32,
        dwBuildMinor: &mut u32,
    ) -> c_int {
        // Stub - would be implemented separately
        0
    }

    /// static BOOL GetImmAPIVersion(DWORD &dwMajor, DWORD &dwMinor, DWORD &dwBuild, DWORD &dwBuildMinor);
    pub fn GetImmAPIVersion(
        dwMajor: &mut u32,
        dwMinor: &mut u32,
        dwBuild: &mut u32,
        dwBuildMinor: &mut u32,
    ) -> c_int {
        // Stub - would be implemented separately
        0
    }

    /// static BOOL GetDXVersion(DWORD &dwMajor, DWORD &dwMinor, DWORD &dwBuild, DWORD &dwBuildMinor);
    pub fn GetDXVersion(
        dwMajor: &mut u32,
        dwMinor: &mut u32,
        dwBuild: &mut u32,
        dwBuildMinor: &mut u32,
    ) -> c_int {
        // Stub - would be implemented separately
        0
    }

    /// static CImmDevice *
    /// CreateDevice(HINSTANCE hinstApp, HWND hwndApp);
    pub fn CreateDevice(hinstApp: *mut c_void, hwndApp: *mut c_void) -> *mut CImmDevice {
        // Stub - would be implemented separately
        std::ptr::null_mut()
    }

    //
    // PROTECTED HELPERS
    //

    /// Performs device preparation by setting the device's parameters
    pub fn prepare_device(&mut self) -> c_int {
        // Default implementation - can be overridden in subclass
        0
    }

    /// virtual void reset() = 0; // pure virtual function
    pub fn reset(&mut self) {
        // Pure virtual - must be overridden in subclass
    }

    /// static BOOL CALLBACK
    /// enum_didevices_proc(
    ///     LPDIDEVICEINSTANCE pImmDevInst,
    ///     LPVOID pv
    ///     );
    pub unsafe extern "C" fn enum_didevices_proc(
        pImmDevInst: *mut c_void,
        pv: *mut c_void,
    ) -> c_int {
        // Stub - would be implemented separately
        0
    }

    /// static BOOL CALLBACK
    /// enum_devices_proc(
    ///     LPIMM_DEVICEINSTANCE pImmDevInst,
    ///     LPVOID pv
    ///     );
    pub unsafe extern "C" fn enum_devices_proc(
        pImmDevInst: *mut c_void,
        pv: *mut c_void,
    ) -> c_int {
        // Stub - would be implemented separately
        0
    }

    pub fn detach_effects(&mut self) {
        // Stub - would be implemented separately
    }

    //
    // CACHING (conditional on IFC_EFFECT_CACHING)
    //

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn Cache_AddEffect(&mut self, pImmEffect: *mut c_void) {
        // Stub - would be implemented separately
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn Cache_RemoveEffect(&mut self, pImmEffect: *const c_void) {
        // Stub - would be implemented separately
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn Cache_SwapOutEffect(&mut self) {
        // Stub - would be implemented separately
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn Cache_LoadEffectSuite(&mut self, pSuite: *mut c_void, bCreateOnDevice: c_int) {
        // Stub - would be implemented separately
    }

    #[cfg(feature = "IFC_EFFECT_CACHING")]
    pub fn Cache_UnloadEffectSuite(&mut self, pSuite: *mut c_void, bUnloadFromDevice: c_int) {
        // Stub - would be implemented separately
    }
}

#[cfg(feature = "IFC_EFFECT_CACHING")]
pub type CEffectList = ();
