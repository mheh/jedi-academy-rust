#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_uint, c_uchar, c_void};

// Opaque struct types for ALCdevice and ALCcontext
// These are opaque types matching the C declarations:
// typedef struct ALCdevice_struct ALCdevice;
// typedef struct ALCcontext_struct ALCcontext;
#[repr(C)]
pub struct ALCdevice_struct {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ALCcontext_struct {
    _private: [u8; 0],
}

// Type aliases matching C typedef behavior
pub type ALCdevice = ALCdevice_struct;
pub type ALCcontext = ALCcontext_struct;

// Type aliases for ALC types (from alctypes.h)
pub type ALCboolean = c_char;
pub type ALCubyte = c_uchar;
pub type ALCint = c_int;
pub type ALCsizei = c_uint;
pub type ALCenum = c_int;
pub type ALCvoid = c_void;

// Type aliases for AL types (from altypes.h)
pub type ALubyte = c_uchar;

// Function declarations when ALC_NO_PROTOTYPES is not defined (default)
#[cfg(not(feature = "alc_no_prototypes"))]
extern "C" {
    pub fn alcGetString(device: *mut ALCdevice, param: ALCenum) -> *mut ALCubyte;
    pub fn alcGetIntegerv(
        device: *mut ALCdevice,
        param: ALCenum,
        size: ALCsizei,
        data: *mut ALCint,
    );

    pub fn alcOpenDevice(deviceName: *mut ALCubyte) -> *mut ALCdevice;
    pub fn alcCloseDevice(device: *mut ALCdevice);

    pub fn alcCreateContext(
        device: *mut ALCdevice,
        attrList: *mut ALCint,
    ) -> *mut ALCcontext;
    pub fn alcMakeContextCurrent(context: *mut ALCcontext) -> ALCboolean;
    pub fn alcProcessContext(context: *mut ALCcontext);
    pub fn alcGetCurrentContext() -> *mut ALCcontext;
    pub fn alcGetContextsDevice(context: *mut ALCcontext) -> *mut ALCdevice;
    pub fn alcSuspendContext(context: *mut ALCcontext);
    pub fn alcDestroyContext(context: *mut ALCcontext);

    pub fn alcGetError(device: *mut ALCdevice) -> ALCenum;

    pub fn alcIsExtensionPresent(device: *mut ALCdevice, extName: *mut ALCubyte) -> ALCboolean;
    pub fn alcGetProcAddress(device: *mut ALCdevice, funcName: *mut ALCubyte) -> *mut ALCvoid;
    pub fn alcGetEnumValue(device: *mut ALCdevice, enumName: *mut ALCubyte) -> ALCenum;
}

// Function pointer declarations when ALC_NO_PROTOTYPES is defined
#[cfg(feature = "alc_no_prototypes")]
pub mod alc_function_pointers {
    use super::*;

    // ALCAPI ALCubyte* ALCAPIENTRY (*alcGetString)(ALCdevice *device,ALCenum param);
    pub static mut alcGetString: Option<
        unsafe extern "C" fn(*mut ALCdevice, ALCenum) -> *mut ALCubyte,
    > = None;

    // ALCAPI ALCvoid ALCAPIENTRY (*alcGetIntegerv)(ALCdevice * device,ALCenum param,ALCsizei size,ALCint *data);
    pub static mut alcGetIntegerv: Option<
        unsafe extern "C" fn(*mut ALCdevice, ALCenum, ALCsizei, *mut ALCint),
    > = None;

    // ALCAPI ALCdevice* ALCAPIENTRY (*alcOpenDevice)(ALubyte *deviceName);
    pub static mut alcOpenDevice:
        Option<unsafe extern "C" fn(*mut ALubyte) -> *mut ALCdevice> = None;

    // ALCAPI ALCvoid ALCAPIENTRY (*alcCloseDevice)(ALCdevice *device);
    pub static mut alcCloseDevice: Option<unsafe extern "C" fn(*mut ALCdevice)> = None;

    // ALCAPI ALCcontext*ALCAPIENTRY (*alcCreateContext)(ALCdevice *device,ALCint *attrList);
    pub static mut alcCreateContext:
        Option<unsafe extern "C" fn(*mut ALCdevice, *mut ALCint) -> *mut ALCcontext> = None;

    // ALCAPI ALCboolean ALCAPIENTRY (*alcMakeContextCurrent)(ALCcontext *context);
    pub static mut alcMakeContextCurrent:
        Option<unsafe extern "C" fn(*mut ALCcontext) -> ALCboolean> = None;

    // ALCAPI ALCvoid ALCAPIENTRY (*alcProcessContext)(ALCcontext *context);
    pub static mut alcProcessContext: Option<unsafe extern "C" fn(*mut ALCcontext)> = None;

    // ALCAPI ALCcontext*ALCAPIENTRY (*alcGetCurrentContext)(ALCvoid);
    pub static mut alcGetCurrentContext:
        Option<unsafe extern "C" fn() -> *mut ALCcontext> = None;

    // ALCAPI ALCdevice* ALCAPIENTRY (*alcGetContextsDevice)(ALCcontext *context);
    pub static mut alcGetContextsDevice:
        Option<unsafe extern "C" fn(*mut ALCcontext) -> *mut ALCdevice> = None;

    // ALCAPI ALCvoid ALCAPIENTRY (*alcSuspendContext)(ALCcontext *context);
    pub static mut alcSuspendContext: Option<unsafe extern "C" fn(*mut ALCcontext)> = None;

    // ALCAPI ALCvoid ALCAPIENTRY (*alcDestroyContext)(ALCcontext *context);
    pub static mut alcDestroyContext: Option<unsafe extern "C" fn(*mut ALCcontext)> = None;

    // ALCAPI ALCenum ALCAPIENTRY (*alcGetError)(ALCdevice *device);
    pub static mut alcGetError:
        Option<unsafe extern "C" fn(*mut ALCdevice) -> ALCenum> = None;

    // ALCAPI ALCboolean ALCAPIENTRY (*alcIsExtensionPresent)(ALCdevice *device,ALCubyte *extName);
    pub static mut alcIsExtensionPresent:
        Option<unsafe extern "C" fn(*mut ALCdevice, *mut ALCubyte) -> ALCboolean> = None;

    // ALCAPI ALCvoid * ALCAPIENTRY (*alcGetProcAddress)(ALCdevice *device,ALCubyte *funcName);
    pub static mut alcGetProcAddress:
        Option<unsafe extern "C" fn(*mut ALCdevice, *mut ALCubyte) -> *mut ALCvoid> = None;

    // ALCAPI ALCenum ALCAPIENTRY (*alcGetEnumValue)(ALCdevice *device,ALCubyte *enumName);
    pub static mut alcGetEnumValue:
        Option<unsafe extern "C" fn(*mut ALCdevice, *mut ALCubyte) -> ALCenum> = None;
}
