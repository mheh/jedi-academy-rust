/*
*/

use core::ffi::{c_int, c_char, c_void};

// Stubs for Windows COM types from objbase.h
pub type HRESULT = i32;
pub type DWORD = u32;
pub type REFIID = *const c_void;
pub type LPVOID = *mut c_void;
pub type ULONG = u32;

// Stub for EAXSOURCEPROPERTIES from eax.h
#[repr(C)]
pub struct EAXSOURCEPROPERTIES {
    // Porting note: structure from eax.h, stubbed for structural completeness
}

pub type LPEAXREVERBPROPERTIES = *mut c_void;

pub const EM_MAX_NAME: c_int = 32;

pub const EMFLAG_IDDEFAULT: c_int = -1;
pub const EMFLAG_IDNONE: c_int = -2;
pub const EMFLAG_LOCKPOSITION: c_int = 1;
pub const EMFLAG_LOADFROMMEMORY: c_int = 2;
pub const EMFLAG_NODIFFRACTION: c_int = 4;

#[repr(C)]
pub struct EMPOINT {
    pub fX: f32,
    pub fY: f32,
    pub fZ: f32,
}
pub type LPEMPOINT = *mut EMPOINT;

#[repr(C)]
pub struct LISTENERATTRIBUTES {
    pub fDistanceFactor: f32,
    pub fRolloffFactor: f32,
    pub fDopplerFactor: f32,
}
pub type LPLISTENERATTRIBUTES = *mut LISTENERATTRIBUTES;

#[repr(C)]
pub struct SOURCEATTRIBUTES {
    pub eaxAttributes: EAXSOURCEPROPERTIES,
    pub ulInsideConeAngle: u32,
    pub ulOutsideConeAngle: u32,
    pub lConeOutsideVolume: i32,
    pub fConeXdir: f32,
    pub fConeYdir: f32,
    pub fConeZdir: f32,
    pub fMinDistance: f32,
    pub fMaxDistance: f32,
    pub lDupCount: i32,
    pub lPriority: i32,
}
pub type LPSOURCEATTRIBUTES = *mut SOURCEATTRIBUTES;

#[repr(C)]
pub struct MATERIALATTRIBUTES {
    pub lLevel: i32,
    pub fLFRatio: f32,
    pub fRoomRatio: f32,
    pub dwFlags: DWORD,
}
pub type LPMATERIALATTRIBUTES = *mut MATERIALATTRIBUTES;

pub const EMMATERIAL_OBSTRUCTS: c_int = 1;
pub const EMMATERIAL_OCCLUDES: c_int = 3;

#[repr(C)]
pub struct DIFFRACTIONBOX {
    pub lSubspaceID: i32,
    pub empMin: EMPOINT,
    pub empMax: EMPOINT,
}
pub type LPDIFFRACTIONBOX = *mut DIFFRACTIONBOX;

// {7CE4D6E6-562F-11d3-8812-005004062F83}
pub const CLSID_EAXMANAGER: [u8; 16] = [
    0xa1, 0x21, 0xb7, 0x60, 0xc8, 0xf7, 0xd2, 0x11,
    0xa0, 0x2e, 0x00, 0x50, 0x04, 0x06, 0x18, 0xb8,
];

pub type LPEAXMANAGER = *mut IEaxManager;

// {7CE4D6E8-562F-11d3-8812-005004062F83}
pub const IID_IEaxManager: [u8; 16] = [
    0xa2, 0x21, 0xb7, 0x60, 0xc8, 0xf7, 0xd2, 0x11,
    0xa0, 0x2e, 0x00, 0x50, 0x04, 0x06, 0x18, 0xb8,
];

#[repr(C)]
pub struct IEaxManager {
    pub lpVtbl: *mut IEaxManager_VTable,
}

#[repr(C)]
pub struct IEaxManager_VTable {
    // IUnknown methods
    pub QueryInterface: extern "C" fn(*mut IEaxManager, REFIID, *mut LPVOID) -> HRESULT,
    pub AddRef: extern "C" fn(*mut IEaxManager) -> ULONG,
    pub Release: extern "C" fn(*mut IEaxManager) -> ULONG,

    pub GetDataSetSize: extern "C" fn(*mut IEaxManager, *mut u32, DWORD) -> HRESULT,
    pub LoadDataSet: extern "C" fn(*mut IEaxManager, *mut c_char, DWORD) -> HRESULT,
    pub FreeDataSet: extern "C" fn(*mut IEaxManager, DWORD) -> HRESULT,
    pub GetListenerAttributes: extern "C" fn(*mut IEaxManager, LPLISTENERATTRIBUTES) -> HRESULT,
    pub GetSourceID: extern "C" fn(*mut IEaxManager, *mut c_char, *mut i32) -> HRESULT,
    pub GetSourceAttributes: extern "C" fn(*mut IEaxManager, i32, LPSOURCEATTRIBUTES) -> HRESULT,
    pub GetSourceNumInstances: extern "C" fn(*mut IEaxManager, i32, *mut i32) -> HRESULT,
    pub GetSourceInstancePos: extern "C" fn(*mut IEaxManager, i32, i32, LPEMPOINT) -> HRESULT,
    pub GetEnvironmentID: extern "C" fn(*mut IEaxManager, *mut c_char, *mut i32) -> HRESULT,
    pub GetEnvironmentAttributes: extern "C" fn(*mut IEaxManager, i32, LPEAXREVERBPROPERTIES) -> HRESULT,
    pub GetMaterialID: extern "C" fn(*mut IEaxManager, *mut c_char, *mut i32) -> HRESULT,
    pub GetMaterialAttributes: extern "C" fn(*mut IEaxManager, i32, LPMATERIALATTRIBUTES) -> HRESULT,
    pub GetGeometrySetID: extern "C" fn(*mut IEaxManager, *mut c_char, *mut i32) -> HRESULT,
    pub GetListenerDynamicAttributes: extern "C" fn(*mut IEaxManager, i32, LPEMPOINT, *mut i32, DWORD) -> HRESULT,
    pub GetSourceDynamicAttributes: extern "C" fn(*mut IEaxManager, i32, LPEMPOINT, *mut i32, *mut f32, *mut i32, *mut f32, *mut f32, LPEMPOINT, DWORD) -> HRESULT,
    // pub GetSubSpaceID: extern "C" fn(*mut IEaxManager, i32, LPEMPOINT, *mut i32) -> HRESULT,
    pub GetEnvironmentName: extern "C" fn(*mut IEaxManager, i32, *mut c_char, i32) -> HRESULT,
}

pub type LPEAXMANAGERCREATE = extern "C" fn(*mut LPEAXMANAGER) -> HRESULT;

extern "C" {
    pub fn EaxManagerCreate(ppManager: *mut LPEAXMANAGER) -> HRESULT;
}

pub const EM_OK: HRESULT = 0;
pub const EM_INVALIDID: HRESULT = make_hresult(1, 4, 1);
pub const EM_IDNOTFOUND: HRESULT = make_hresult(1, 4, 2);
pub const EM_FILENOTFOUND: HRESULT = make_hresult(1, 4, 3);
pub const EM_FILEINVALID: HRESULT = make_hresult(1, 4, 4);
pub const EM_VERSIONINVALID: HRESULT = make_hresult(1, 4, 5);
pub const EM_INSTANCENOTFOUND: HRESULT = make_hresult(1, 4, 6);

// Porting note: MAKE_HRESULT(severity, facility, code) macro implementation
// FACILITY_ITF = 4
const fn make_hresult(severity: i32, facility: i32, code: i32) -> HRESULT {
    ((severity & 0x1) << 31) | ((facility & 0x1fff) << 16) | (code & 0xffff)
}
