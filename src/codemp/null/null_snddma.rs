#![allow(non_snake_case, non_camel_case_types, unused_variables)]

use core::ffi::{c_char, c_int};

use crate::ffi::types::{qboolean, QFALSE};

pub type sfxHandle_t = c_int;

// important to default to this!!!
#[no_mangle]
pub static mut gbInsideLoadSound: qboolean = QFALSE;

#[no_mangle]
pub extern "C" fn SNDDMA_Init() -> qboolean {
    QFALSE
}

#[no_mangle]
pub extern "C" fn SNDDMA_GetDMAPos() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn SNDDMA_Shutdown() {}

#[no_mangle]
pub extern "C" fn SNDDMA_BeginPainting() {}

#[no_mangle]
pub extern "C" fn SNDDMA_Submit() {}

#[no_mangle]
pub extern "C" fn S_RegisterSound(name: *const c_char) -> sfxHandle_t {
    0
}

#[no_mangle]
pub extern "C" fn S_StartLocalSound(sfxHandle: sfxHandle_t, channelNum: c_int) {}

#[no_mangle]
pub extern "C" fn S_ClearSoundBuffer() {}

#[no_mangle]
pub extern "C" fn SND_RegisterAudio_LevelLoadEnd(something: qboolean) -> qboolean {
    QFALSE
}

#[no_mangle]
pub extern "C" fn SND_FreeOldestSound() -> c_int {
    0
}
