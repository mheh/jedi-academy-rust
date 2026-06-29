#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::{c_int, c_void};
use super::alctypes_h::{
    ALCenum, ALCboolean, ALCint, ALCsizei, ALCubyte,
};

// Opaque ALC device structure
#[repr(C)]
pub struct ALCdevice_struct;
pub type ALCdevice = ALCdevice_struct;

// Opaque ALC context structure
#[repr(C)]
pub struct ALCcontext_struct;
pub type ALCcontext = ALCcontext_struct;

// Platform-specific ABI specifiers are implicit in extern "C" declarations.
// Original macro definitions:
// #ifdef _XBOX:     ALCAPI (empty), ALCAPIENTRY (empty)
// #ifdef _WIN32:    ALCAPI (__declspec(dllexport/dllimport)), ALCAPIENTRY (__cdecl)
// #else:            ALCAPI (empty), ALCAPIENTRY (__cdecl)

#[cfg(not(target_os = "xbox"))]
extern "C" {
    pub fn alcGetString(device: *mut ALCdevice, param: ALCenum) -> *mut ALCubyte;
    pub fn alcGetIntegerv(device: *mut ALCdevice, param: ALCenum, size: ALCsizei, data: *mut ALCint);

    pub fn alcOpenDevice(deviceName: *mut ALCubyte) -> *mut ALCdevice;
    pub fn alcCloseDevice(device: *mut ALCdevice);

    pub fn alcCreateContext(device: *mut ALCdevice, attrList: *mut ALCint) -> *mut ALCcontext;
    pub fn alcMakeContextCurrent(context: *mut ALCcontext) -> ALCboolean;
    pub fn alcProcessContext(context: *mut ALCcontext);
    pub fn alcGetCurrentContext() -> *mut ALCcontext;
    pub fn alcGetContextsDevice(context: *mut ALCcontext) -> *mut ALCdevice;
    pub fn alcSuspendContext(context: *mut ALCcontext);
    pub fn alcDestroyContext(context: *mut ALCcontext);

    pub fn alcGetError(device: *mut ALCdevice) -> ALCenum;

    pub fn alcIsExtensionPresent(device: *mut ALCdevice, extName: *mut ALCubyte) -> ALCboolean;
    pub fn alcGetProcAddress(device: *mut ALCdevice, funcName: *mut ALCubyte) -> *mut c_void;
    pub fn alcGetEnumValue(device: *mut ALCdevice, enumName: *mut ALCubyte) -> ALCenum;
}

// Function pointer versions (when ALC_NO_PROTOTYPES is defined)
#[cfg(all(not(target_os = "xbox"), feature = "alc_no_prototypes"))]
pub mod function_pointers {
    use core::ffi::{c_int, c_void};
    use super::super::alctypes_h::{
        ALCenum, ALCboolean, ALCint, ALCsizei, ALCubyte,
    };
    use super::{ALCdevice, ALCcontext};

    pub static mut alcGetString: Option<unsafe extern "C" fn(*mut ALCdevice, ALCenum) -> *mut ALCubyte> = None;
    pub static mut alcGetIntegerv: Option<unsafe extern "C" fn(*mut ALCdevice, ALCenum, ALCsizei, *mut ALCint)> = None;

    pub static mut alcOpenDevice: Option<unsafe extern "C" fn(*mut ALCubyte) -> *mut ALCdevice> = None;
    pub static mut alcCloseDevice: Option<unsafe extern "C" fn(*mut ALCdevice)> = None;

    pub static mut alcCreateContext: Option<unsafe extern "C" fn(*mut ALCdevice, *mut ALCint) -> *mut ALCcontext> = None;
    pub static mut alcMakeContextCurrent: Option<unsafe extern "C" fn(*mut ALCcontext) -> ALCboolean> = None;
    pub static mut alcProcessContext: Option<unsafe extern "C" fn(*mut ALCcontext)> = None;
    pub static mut alcGetCurrentContext: Option<unsafe extern "C" fn() -> *mut ALCcontext> = None;
    pub static mut alcGetContextsDevice: Option<unsafe extern "C" fn(*mut ALCcontext) -> *mut ALCdevice> = None;
    pub static mut alcSuspendContext: Option<unsafe extern "C" fn(*mut ALCcontext)> = None;
    pub static mut alcDestroyContext: Option<unsafe extern "C" fn(*mut ALCcontext)> = None;

    pub static mut alcGetError: Option<unsafe extern "C" fn(*mut ALCdevice) -> ALCenum> = None;

    pub static mut alcIsExtensionPresent: Option<unsafe extern "C" fn(*mut ALCdevice, *mut ALCubyte) -> ALCboolean> = None;
    pub static mut alcGetProcAddress: Option<unsafe extern "C" fn(*mut ALCdevice, *mut ALCubyte) -> *mut c_void> = None;
    pub static mut alcGetEnumValue: Option<unsafe extern "C" fn(*mut ALCdevice, *mut ALCubyte) -> ALCenum> = None;
}
