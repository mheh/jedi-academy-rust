#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// LOCAL STUB: Import types from header and utils
// Note: FFMultiSet, FFConfigParser, FFSet, CImmDevices, qboolean, TNameTable
// are defined in ff_MultiSet_h module

// LOCAL STUB: Windows globals structure from win_local.h
#[repr(C)]
pub struct winVars_t {
    pub hInstance: *mut c_void,
    pub hWnd: *mut c_void,
}

extern "C" {
    pub static mut g_wv: winVars_t;
    pub fn Com_Printf(fmt: *const c_char, ...) -> c_int;
}

// LOCAL STUB: External C++ wrappers for CImmDevices and FFSet methods
// These represent calls to C++ class member functions through extern "C" interface
extern "C" {
    // new CImmDevices() wrapper
    fn FFMultiSet_NewCImmDevices() -> *mut c_void;

    // CImmDevices::CreateDevices wrapper
    fn CImmDevices_CreateDevices(this: *mut c_void, hInstance: *mut c_void, hWnd: *mut c_void) -> c_int;

    // CImmDevices::GetNumDevices wrapper
    fn CImmDevices_GetNumDevices(this: *const c_void) -> c_int;

    // CImmDevices::GetDevice wrapper
    fn CImmDevices_GetDevice(this: *mut c_void, index: c_int) -> *mut c_void;

    // new FFSet(config, device) wrapper
    fn FFMultiSet_NewFFSet(config: *mut c_void, device: *mut c_void) -> *mut c_void;

    // FFSet::GetRegisteredNames wrapper
    fn FFSet_GetRegisteredNames(this: *mut c_void, table: *mut c_void) -> c_int;

    // FFSet::StopAll wrapper
    fn FFSet_StopAll(this: *mut c_void) -> c_int;

    // FFSet::GetDisplayTokens static wrapper
    fn FFSet_GetDisplayTokens(table: *mut c_void);

    // FFSet::Display wrapper
    fn FFSet_Display(this: *mut c_void, unprocessed: *mut c_void, processed: *mut c_void);

    // CImmDevice::GetProductName wrapper
    fn CImmDevice_GetProductName(this: *mut c_void, buffer: *mut c_char, max_len: c_int) -> c_int;
}

// Import implementations from header stubs and utilities
use crate::code::ff::ff_MultiSet_h::{FFMultiSet, FFConfigParser, FFSet, CImmDevices, qboolean, TNameTable};
use crate::code::ff::ff_utils_h::DeletePointer;

const FF_MAX_PATH: c_int = 256;
const QTRUE: qboolean = 1;
const QFALSE: qboolean = 0;

impl FFMultiSet {
    ////----------------
    /// FFMultiSet::Init
    //--------------------
    //	Initializes all attached force feedback devices. An empty FFSet is created
    //	for each device. Each device will have its own copy of whatever .ifr file
    //	set 'config' specifies.
    //
    //	Always pair with clear()
    //
    pub fn Init(&mut self, config: &mut FFConfigParser) -> qboolean {
        self.mConfig = config as *mut FFConfigParser;

        #[cfg(feature = "FF_PRINT")]
        {
            //Com_Printf( "Feedback devices:\n" );
        }

        unsafe {
            let hInstance = g_wv.hInstance;
            let hWnd = g_wv.hWnd;

            // new CImmDevices
            self.mDevices = FFMultiSet_NewCImmDevices() as *mut CImmDevices;
            if !self.mDevices.is_null() && CImmDevices_CreateDevices(self.mDevices as *mut c_void, hInstance, hWnd) != 0 {
                let num_devices = CImmDevices_GetNumDevices(self.mDevices as *const c_void);
                for i in 0..num_devices {
                    let device = CImmDevices_GetDevice(self.mDevices as *mut c_void, i);
                    // new FFSet(config, device)
                    let ffSet = FFMultiSet_NewFFSet(self.mConfig as *mut c_void, device) as *mut FFSet;

                    if !ffSet.is_null() {
                        #[cfg(feature = "FF_PRINT")]
                        {
                            let mut ProductName: [c_char; 256] = [0; 256];
                            let _ = CImmDevice_GetProductName(
                                device,
                                ProductName.as_mut_ptr(),
                                FF_MAX_PATH - 1,
                            );
                            Com_Printf(
                                b"%d) %s\n\0".as_ptr() as *const c_char,
                                i,
                                ProductName.as_ptr(),
                            );
                        }
                        self.mSet.push(ffSet);
                    }
                }
            }
        }

        if self.mSet.len() > 0 { QTRUE } else { QFALSE }
    }

    ////------------------------------
    /// FFMultiSet::GetRegisteredNames
    //----------------------------------
    //
    //
    pub fn GetRegisteredNames(&mut self, NameTable: &mut TNameTable) -> qboolean {
        for i in 0..self.mSet.len() {
            unsafe {
                FFSet_GetRegisteredNames(self.mSet[i] as *mut c_void, NameTable as *mut TNameTable as *mut c_void);
            }
        }

        QTRUE
    }

    ////-------------------
    /// FFMultiSet::StopAll
    //-----------------------
    //	Stops all effects in every FFSet.
    //	Returns qfalse if any return false.
    //
    pub fn StopAll(&mut self) -> qboolean {
        let mut result = QTRUE;

        for i in 0..self.mSet.len() {
            unsafe {
                result &= FFSet_StopAll(self.mSet[i] as *mut c_void);
            }
        }

        result
    }

    ////-----------------
    /// FFMultiSet::clear
    //---------------------
    //	Cleans up.
    //
    pub fn clear(&mut self) {
        self.mConfig = core::ptr::null_mut();
        for item in self.mSet.iter_mut() {
            DeletePointer(item, None);
        }
        self.mSet.clear();
        DeletePointer(&mut self.mDevices, None);
    }
}

#[cfg(feature = "FF_CONSOLECOMMAND")]
impl FFMultiSet {
    pub fn GetDisplayTokens(Tokens: &mut TNameTable) {
        unsafe {
            FFSet_GetDisplayTokens(Tokens as *mut TNameTable as *mut c_void);
        }
    }

    ////-------------------
    ///	FFMultiSet::Display
    //-----------------------
    //
    //
    pub fn Display(&mut self, Unprocessed: &mut TNameTable, Processed: &mut TNameTable) {
        for i in 0..self.mSet.len() {
            let mut Temp1: TNameTable = Vec::new();
            let mut Temp2: TNameTable = Vec::new();
            Temp1.clear();
            Temp2.clear();
            Temp1.extend(Processed.iter().cloned());
            unsafe {
                FFSet_Display(
                    self.mSet[i] as *mut c_void,
                    &mut Temp1 as *mut TNameTable as *mut c_void,
                    &mut Temp2 as *mut TNameTable as *mut c_void,
                );
            }
        }
    }
}
