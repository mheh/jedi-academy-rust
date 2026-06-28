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

  FILE:        ImmDXDevice.h

  PURPOSE:    Abstraction of DirectX Force Feedback device

  STARTED:    10/10/97

  NOTES/REVISIONS:
     3/2/99 jrm (Jeff Mallett): Force-->Feel renaming
     3/15/99 jrm: __declspec(dllimport/dllexport) the whole class

**********************************************************************/

// Note: DLLIFC in C++ was __declspec(dllimport) or __declspec(dllexport)
// depending on _IFCDLL_. This is not directly applicable to Rust, but kept
// for documentation of the original intent.

use core::ffi::{c_int, c_void};

// Forward declarations for types from ImmDevice.h
// CImmDevice is the parent class; exact definition depends on ImmDevice.h
pub struct CImmDevice {
    // Parent class structure - exact layout defined in ImmDevice.h
}

// Windows type stubs for DirectX interfaces (opaque pointers)
pub type HANDLE = *mut c_void;
pub type HWND = *mut c_void;
pub type DWORD = u32;
pub type BOOL = c_int;
pub type LPTSTR = *mut i32; // wide char string pointer

// DirectX interface pointers (opaque)
pub type LPDIRECTINPUT = *mut c_void;
pub type LPDIRECTINPUTDEVICE2 = *mut c_void;

// GUID structure
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

// IFC interface types
pub type LPIIMM_API = *mut c_void; // actually LPDIRECTINPUT
pub type LPIIMM_DEVICE = *mut c_void; // actually LPDIRECTINPUTDEVICE2

//================================================================
// CImmDXDevice
//================================================================

//
// ------ PUBLIC INTERFACE ------
//

#[repr(C)]
pub struct CImmDXDevice {
    // Parent class fields (inherited from CImmDevice)
    // Layout preserved to match C++ vtable and member layout

    //
    // CONSTRUCTOR/DESCTRUCTOR
    //
    // Constructor: CImmDXDevice()
    // Destructor: virtual ~CImmDXDevice()

    //
    // ATTRIBUTES
    //

    //
    // OPERATIONS
    //

    // BOOL Initialize(
    //     HANDLE hinstApp,
    //     HANDLE hwndApp,
    //     LPDIRECTINPUT pDI = NULL,
    //     LPDIRECTINPUTDEVICE2 piDevice = NULL,
    //     BOOL bEnumerate = TRUE
    // );

    // virtual BOOL GetCurrentPosition(long &lXPos, long &lYPos);

    //
    // ------ PRIVATE INTERFACE ------
    //

    //
    // HELPERS
    //

    // virtual void reset();

    // friend class CImmDevices;
    // static BOOL CALLBACK devices_enum_proc(
    //     LPDIDEVICEINSTANCE pImmDevInst,
    //     LPVOID pv
    // );

    //
    // INTERNAL DATA
    //

    // TODO: these are unused... delete them in future rev
    pub m_bpDIPreExist: BOOL,
    pub m_bpDIDevicePreExist: BOOL,
    // end of useless variables

    pub m_piApi: LPDIRECTINPUT,
    pub m_piDevice: LPDIRECTINPUTDEVICE2,
}

//
// Method stubs - these would normally be virtual methods in C++
// extern "C" declarations or trait implementations would follow
//

impl CImmDXDevice {
    // virtual LPIIMM_API GetAPI()
    // { return (LPIIMM_API) m_piApi; } // actually LPDIRECTINPUT
    pub fn GetAPI(&self) -> LPIIMM_API {
        self.m_piApi as LPIIMM_API
    }

    // virtual LPIIMM_DEVICE GetDevice()
    // { return (LPIIMM_DEVICE) m_piDevice; } // actually LPDIRECTINPUTDEVICE2
    pub fn GetDevice(&self) -> LPIIMM_DEVICE {
        self.m_piDevice as LPIIMM_DEVICE
    }

    // virtual DWORD GetProductType();
    pub fn GetProductType(&self) -> DWORD {
        // Implementation would be in the .cpp file
        0
    }

    // virtual BOOL GetDriverVersion(
    //     DWORD &dwFFDriverVersion,
    //     DWORD &dwFirmwareRevision,
    //     DWORD &dwHardwareRevision);
    pub fn GetDriverVersion(
        &self,
        dwFFDriverVersion: &mut DWORD,
        dwFirmwareRevision: &mut DWORD,
        dwHardwareRevision: &mut DWORD,
    ) -> BOOL {
        // Implementation would be in the .cpp file
        0
    }

    // virtual int GetProductName(LPTSTR lpszProductName, int nMaxCount);
    pub fn GetProductName(&self, lpszProductName: LPTSTR, nMaxCount: c_int) -> c_int {
        // Implementation would be in the .cpp file
        0
    }

    // virtual int GetProductGUIDString(LPTSTR lpszGUID, int nMaxCount);
    pub fn GetProductGUIDString(&self, lpszGUID: LPTSTR, nMaxCount: c_int) -> c_int {
        // Implementation would be in the .cpp file
        0
    }

    // virtual GUID GetProductGUID();
    pub fn GetProductGUID(&self) -> GUID {
        // Implementation would be in the .cpp file
        GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        }
    }

    // BOOL Initialize(
    //     HANDLE hinstApp,
    //     HANDLE hwndApp,
    //     LPDIRECTINPUT pDI = NULL,
    //     LPDIRECTINPUTDEVICE2 piDevice = NULL,
    //     BOOL bEnumerate = TRUE
    // );
    pub fn Initialize(
        &mut self,
        hinstApp: HANDLE,
        hwndApp: HWND,
        pDI: Option<LPDIRECTINPUT>,
        piDevice: Option<LPDIRECTINPUTDEVICE2>,
        bEnumerate: BOOL,
    ) -> BOOL {
        // Implementation would be in the .cpp file
        0
    }

    // virtual BOOL GetCurrentPosition(long &lXPos, long &lYPos);
    pub fn GetCurrentPosition(&self, lXPos: &mut i32, lYPos: &mut i32) -> BOOL {
        // Implementation would be in the .cpp file
        0
    }

    // virtual void reset();
    pub fn reset(&mut self) {
        // Implementation would be in the .cpp file
    }

    // static BOOL CALLBACK devices_enum_proc(
    //     LPDIDEVICEINSTANCE pImmDevInst,
    //     LPVOID pv
    // );
    pub extern "C" fn devices_enum_proc(
        pImmDevInst: *mut c_void,
        pv: *mut c_void,
    ) -> BOOL {
        // Implementation would be in the .cpp file
        0
    }
}
