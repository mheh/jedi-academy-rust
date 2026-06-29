//
// Faithful C-to-Rust translation of oracle/code/client/eax/EaxMan.h
// Preserves all original comments, structure, and symbol names

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};
use core::ffi::CStr;

// COM types that would come from objbase.h and windows.h
// Representing Windows ABI types
pub type HRESULT = c_int;
pub type DWORD = u32;
pub type ULONG = u32;
pub type REFIID = *const c_void;
pub type LPVOID = *mut c_void;

// Represents IUnknown interface pointer
pub type LPUNKNOWN = *mut IUnknown;

#[repr(C)]
pub struct IUnknown {
    lpVtbl: *mut IUnknownVtbl,
}

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "C" fn(*mut IUnknown, REFIID, *mut LPVOID) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IUnknown) -> ULONG,
    pub Release: unsafe extern "C" fn(*mut IUnknown) -> ULONG,
}

// EAX types - these would come from eax.h
pub type LPEAXREVERBPROPERTIES = *mut c_void;
pub type EAXSOURCEPROPERTIES = [u8; 48]; // Placeholder size

const EM_MAX_NAME: usize = 32;

const EMFLAG_IDDEFAULT: c_int = -1;
const EMFLAG_IDNONE: c_int = -2;
const EMFLAG_LOCKPOSITION: c_int = 1;
const EMFLAG_LOADFROMMEMORY: c_int = 2;
const EMFLAG_NODIFFRACTION: c_int = 4;

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
    pub ulInsideConeAngle: c_int,
    pub ulOutsideConeAngle: c_int,
    pub lConeOutsideVolume: c_int,
    pub fConeXdir: f32,
    pub fConeYdir: f32,
    pub fConeZdir: f32,
    pub fMinDistance: f32,
    pub fMaxDistance: f32,
    pub lDupCount: c_int,
    pub lPriority: c_int,
}

pub type LPSOURCEATTRIBUTES = *mut SOURCEATTRIBUTES;

#[repr(C)]
pub struct MATERIALATTRIBUTES {
    pub lLevel: c_int,
    pub fLFRatio: f32,
    pub fRoomRatio: f32,
    pub dwFlags: DWORD,
}

pub type LPMATERIALATTRIBUTES = *mut MATERIALATTRIBUTES;

const EMMATERIAL_OBSTRUCTS: c_int = 1;
const EMMATERIAL_OCCLUDES: c_int = 3;

#[repr(C)]
pub struct DIFFRACTIONBOX {
    pub lSubspaceID: c_int,
    pub empMin: EMPOINT,
    pub empMax: EMPOINT,
}

pub type LPDIFFRACTIONBOX = *mut DIFFRACTIONBOX;

// {7CE4D6E6-562F-11d3-8812-005004062F83}
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

pub const CLSID_EAXMANAGER: GUID = GUID {
    data1: 0x60b721a1,
    data2: 0xf7c8,
    data3: 0x11d2,
    data4: [0xa0, 0x2e, 0x00, 0x50, 0x04, 0x06, 0x18, 0xb8],
};

pub type LPEAXMANAGER = *mut IEaxManager;

// {7CE4D6E8-562F-11d3-8812-005004062F83}
pub const IID_IEaxManager: GUID = GUID {
    data1: 0x60b721a2,
    data2: 0xf7c8,
    data3: 0x11d2,
    data4: [0xa0, 0x2e, 0x00, 0x50, 0x04, 0x06, 0x18, 0xb8],
};

#[repr(C)]
pub struct IEaxManager {
    lpVtbl: *mut IEaxManagerVtbl,
}

#[repr(C)]
pub struct IEaxManagerVtbl {
    // IUnknown methods
    pub QueryInterface: unsafe extern "C" fn(*mut IEaxManager, REFIID, *mut LPVOID) -> HRESULT,
    pub AddRef: unsafe extern "C" fn(*mut IEaxManager) -> ULONG,
    pub Release: unsafe extern "C" fn(*mut IEaxManager) -> ULONG,

    pub GetDataSetSize: unsafe extern "C" fn(*mut IEaxManager, *mut c_int, DWORD) -> HRESULT,
    pub LoadDataSet: unsafe extern "C" fn(*mut IEaxManager, *const c_char, DWORD) -> HRESULT,
    pub FreeDataSet: unsafe extern "C" fn(*mut IEaxManager, DWORD) -> HRESULT,
    pub GetListenerAttributes: unsafe extern "C" fn(*mut IEaxManager, LPLISTENERATTRIBUTES) -> HRESULT,
    pub GetSourceID: unsafe extern "C" fn(*mut IEaxManager, *const c_char, *mut c_int) -> HRESULT,
    pub GetSourceAttributes: unsafe extern "C" fn(*mut IEaxManager, c_int, LPSOURCEATTRIBUTES) -> HRESULT,
    pub GetSourceNumInstances: unsafe extern "C" fn(*mut IEaxManager, c_int, *mut c_int) -> HRESULT,
    pub GetSourceInstancePos: unsafe extern "C" fn(*mut IEaxManager, c_int, c_int, LPEMPOINT) -> HRESULT,
    pub GetEnvironmentID: unsafe extern "C" fn(*mut IEaxManager, *const c_char, *mut c_int) -> HRESULT,
    pub GetEnvironmentAttributes: unsafe extern "C" fn(*mut IEaxManager, c_int, LPEAXREVERBPROPERTIES) -> HRESULT,
    pub GetMaterialID: unsafe extern "C" fn(*mut IEaxManager, *const c_char, *mut c_int) -> HRESULT,
    pub GetMaterialAttributes: unsafe extern "C" fn(*mut IEaxManager, c_int, LPMATERIALATTRIBUTES) -> HRESULT,
    pub GetGeometrySetID: unsafe extern "C" fn(*mut IEaxManager, *const c_char, *mut c_int) -> HRESULT,
    pub GetListenerDynamicAttributes: unsafe extern "C" fn(*mut IEaxManager, c_int, LPEMPOINT, *mut c_int, DWORD) -> HRESULT,
    pub GetSourceDynamicAttributes: unsafe extern "C" fn(*mut IEaxManager, c_int, LPEMPOINT, *mut c_int, *mut f32, *mut c_int, *mut f32, *mut f32, LPEMPOINT, DWORD) -> HRESULT,
    pub GetEnvironmentName: unsafe extern "C" fn(*mut IEaxManager, c_int, *mut c_char, c_int) -> HRESULT,
}

// External function declaration
extern "C" {
    pub fn EaxManagerCreate(ppManager: *mut LPEAXMANAGER) -> HRESULT;
}

pub type LPEAXMANAGERCREATE = unsafe extern "C" fn(*mut LPEAXMANAGER) -> HRESULT;

// COM error codes
pub const EM_OK: HRESULT = 0;

// MAKE_HRESULT(1, FACILITY_ITF, 1) = 0x80004001
pub const EM_INVALIDID: HRESULT = 0x80004001;
// MAKE_HRESULT(1, FACILITY_ITF, 2) = 0x80004002
pub const EM_IDNOTFOUND: HRESULT = 0x80004002;
// MAKE_HRESULT(1, FACILITY_ITF, 3) = 0x80004003
pub const EM_FILENOTFOUND: HRESULT = 0x80004003;
// MAKE_HRESULT(1, FACILITY_ITF, 4) = 0x80004004
pub const EM_FILEINVALID: HRESULT = 0x80004004;
// MAKE_HRESULT(1, FACILITY_ITF, 5) = 0x80004005
pub const EM_VERSIONINVALID: HRESULT = 0x80004005;
// MAKE_HRESULT(1, FACILITY_ITF, 6) = 0x80004006
pub const EM_INSTANCENOTFOUND: HRESULT = 0x80004006;

// COM method wrapper macros translated to inline helper functions for C interface
// These preserve the original calling convention patterns
#[inline]
pub unsafe fn IEaxManager_QueryInterface(p: LPEAXMANAGER, a: REFIID, b: *mut LPVOID) -> HRESULT {
    ((*(*p).lpVtbl).QueryInterface)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_AddRef(p: LPEAXMANAGER) -> ULONG {
    ((*(*p).lpVtbl).AddRef)(p)
}

#[inline]
pub unsafe fn IEaxManager_Release(p: LPEAXMANAGER) -> ULONG {
    ((*(*p).lpVtbl).Release)(p)
}

#[inline]
pub unsafe fn IEaxManager_GetDataSetSize(p: LPEAXMANAGER, a: *mut c_int, b: DWORD) -> HRESULT {
    ((*(*p).lpVtbl).GetDataSetSize)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_LoadDataSet(p: LPEAXMANAGER, a: *const c_char, b: DWORD) -> HRESULT {
    ((*(*p).lpVtbl).LoadDataSet)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_FreeDataSet(p: LPEAXMANAGER, a: DWORD) -> HRESULT {
    ((*(*p).lpVtbl).FreeDataSet)(p, a)
}

#[inline]
pub unsafe fn IEaxManager_GetListenerAttributes(p: LPEAXMANAGER, a: LPLISTENERATTRIBUTES) -> HRESULT {
    ((*(*p).lpVtbl).GetListenerAttributes)(p, a)
}

#[inline]
pub unsafe fn IEaxManager_GetSourceID(p: LPEAXMANAGER, a: *const c_char, b: *mut c_int) -> HRESULT {
    ((*(*p).lpVtbl).GetSourceID)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_GetSourceAttributes(p: LPEAXMANAGER, a: c_int, b: LPSOURCEATTRIBUTES) -> HRESULT {
    ((*(*p).lpVtbl).GetSourceAttributes)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_GetSourceNumInstances(p: LPEAXMANAGER, a: c_int, b: *mut c_int) -> HRESULT {
    ((*(*p).lpVtbl).GetSourceNumInstances)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_GetSourceInstancePos(p: LPEAXMANAGER, a: c_int, b: c_int, c: LPEMPOINT) -> HRESULT {
    ((*(*p).lpVtbl).GetSourceInstancePos)(p, a, b, c)
}

#[inline]
pub unsafe fn IEaxManager_GetEnvironmentID(p: LPEAXMANAGER, a: *const c_char, b: *mut c_int) -> HRESULT {
    ((*(*p).lpVtbl).GetEnvironmentID)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_GetEnvironmentAttributes(p: LPEAXMANAGER, a: c_int, b: LPEAXREVERBPROPERTIES) -> HRESULT {
    ((*(*p).lpVtbl).GetEnvironmentAttributes)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_GetMaterialID(p: LPEAXMANAGER, a: *const c_char, b: *mut c_int) -> HRESULT {
    ((*(*p).lpVtbl).GetMaterialID)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_GetMaterialAttributes(p: LPEAXMANAGER, a: c_int, b: LPMATERIALATTRIBUTES) -> HRESULT {
    ((*(*p).lpVtbl).GetMaterialAttributes)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_GetGeometrySetID(p: LPEAXMANAGER, a: *const c_char, b: *mut c_int) -> HRESULT {
    ((*(*p).lpVtbl).GetGeometrySetID)(p, a, b)
}

#[inline]
pub unsafe fn IEaxManager_GetListenerDynamicAttributes(p: LPEAXMANAGER, a: c_int, b: LPEMPOINT, c: *mut c_int, d: DWORD) -> HRESULT {
    ((*(*p).lpVtbl).GetListenerDynamicAttributes)(p, a, b, c, d)
}

#[inline]
pub unsafe fn IEaxManager_GetSourceDynamicAttributes(
    p: LPEAXMANAGER,
    a: c_int,
    b: LPEMPOINT,
    c: *mut c_int,
    d: *mut f32,
    e: *mut c_int,
    f: *mut f32,
    g: *mut f32,
    h: LPEMPOINT,
    i: DWORD,
) -> HRESULT {
    ((*(*p).lpVtbl).GetSourceDynamicAttributes)(p, a, b, c, d, e, f, g, h, i)
}

#[inline]
pub unsafe fn IEaxManager_GetEnvironmentName(p: LPEAXMANAGER, a: c_int, b: *mut c_char, c: c_int) -> HRESULT {
    ((*(*p).lpVtbl).GetEnvironmentName)(p, a, b, c)
}
