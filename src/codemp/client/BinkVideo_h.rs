//! Mechanical port of `codemp/client/BinkVideo.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_void};

// External Bink SDK types
// HBINK is an opaque handle to a Bink video structure
pub type HBINK = *mut c_void;

// Stub definition for the Bink structure pointer fields (Width, Height)
// In actual usage, HBINK points to a Bink structure that has these fields
#[repr(C)]
pub struct BinkStruct {
    pub Width: c_int,
    pub Height: c_int,
    // ... rest of the structure is opaque
}

pub type U32 = c_int; // Bink SDK type alias
pub type s32 = c_int; // Bink SDK type alias

// NS_BV_DEFAULT_CIN_BPS
pub const NS_BV_DEFAULT_CIN_BPS: c_int = 2;

// MAX_WIDTH
pub const MAX_WIDTH: c_int = 512;

// MAX_HEIGHT
pub const MAX_HEIGHT: c_int = 512;

// XBOX memory stage constants
pub const XBOX_MEM_STAGE_1: c_int = 32640;
pub const XBOX_MEM_STAGE_2: c_int = 786528;
pub const XBOX_MEM_STAGE_3: c_int = 557152;
pub const XBOX_MEM_STAGE_4: c_int = 106560;
pub const XBOX_MEM_STAGE_5: c_int = 138304;
pub const XBOX_MEM_STAGE_6: c_int = 25696;
pub const XBOX_MEM_STAGE_7: c_int = 100;
pub const XBOX_MEM_STAGE_8: c_int = 100;

// XBOX_BUFFER_SIZE = NS_BV_DEFAULT_CIN_BPS * MAX_WIDTH * MAX_HEIGHT
pub const XBOX_BUFFER_SIZE: c_int = NS_BV_DEFAULT_CIN_BPS * MAX_WIDTH * MAX_HEIGHT;

// Enum for movie status (unnamed in original, preserved as constants)
pub const NS_BV_PLAYING: c_int = 0; // Movie is playing
pub const NS_BV_STOPPED: c_int = 1; // Movie is stopped
pub const NS_BV_PAUSED: c_int = 2;  // Movie is paused

#[repr(C)]
pub struct BinkVideo {
    // Private members (preserved from C++)
    bink: HBINK,
    buffer: *mut c_void,
    texture: c_int,
    status: c_int,
    looping: bool,
    x1: c_float,
    y1: c_float,
    x2: c_float,
    y2: c_float,
    w: c_float,
    h: c_float,

    #[cfg(target_env = "xbox")]
    initialized: bool,
}

// Forward declare the C++ methods as extern functions
// These would be implemented in the corresponding .cpp file
unsafe extern "C" {
    pub fn BinkVideo_new() -> *mut BinkVideo;
    pub fn BinkVideo_delete(this: *mut BinkVideo);
    pub fn BinkVideo_Start(
        this: *mut BinkVideo,
        filename: *const c_char,
        xOrigin: c_float,
        yOrigin: c_float,
        width: c_float,
        height: c_float,
    ) -> bool;
    pub fn BinkVideo_Run(this: *mut BinkVideo) -> bool;
    pub fn BinkVideo_Stop(this: *mut BinkVideo);
    pub fn BinkVideo_Pause(this: *mut BinkVideo);
    pub fn BinkVideo_SetExtents(
        this: *mut BinkVideo,
        xOrigin: c_float,
        yOrigin: c_float,
        width: c_float,
        height: c_float,
    );
    pub fn BinkVideo_GetBinkData(this: *mut BinkVideo) -> *mut c_void;
    pub fn BinkVideo_Draw(this: *mut BinkVideo);
    pub fn BinkVideo_DecompressFrame(this: *mut BinkVideo) -> c_int;
    pub fn BinkVideo_SetMasterVolume(this: *mut BinkVideo, volume: s32);

    #[cfg(target_env = "xbox")]
    pub fn BinkVideo_AllocateXboxMem(this: *mut BinkVideo);

    #[cfg(target_env = "xbox")]
    pub fn BinkVideo_FreeXboxMem(this: *mut BinkVideo);

    pub fn BinkVideo_Allocate(size: U32) -> *mut c_void;
    pub fn BinkVideo_Free(ptr: *mut c_void);
}

impl BinkVideo {
    // GetStatus(void) { return status; }
    pub fn GetStatus(&self) -> c_int {
        self.status
    }

    // SetLooping(bool loop) { looping = loop; }
    pub fn SetLooping(&mut self, looping: bool) {
        self.looping = looping;
    }

    // GetBinkWidth(void) { return this->bink->Width; }
    pub fn GetBinkWidth(&self) -> c_int {
        if self.bink.is_null() {
            0
        } else {
            // SAFETY: We trust that bink points to a valid BinkStruct if non-null
            unsafe { (*(self.bink as *mut BinkStruct)).Width }
        }
    }

    // GetBinkHeight(void) { return this->bink->Height; }
    pub fn GetBinkHeight(&self) -> c_int {
        if self.bink.is_null() {
            0
        } else {
            // SAFETY: We trust that bink points to a valid BinkStruct if non-null
            unsafe { (*(self.bink as *mut BinkStruct)).Height }
        }
    }

    // Ready(void) { return this->bink != NULL; }
    pub fn Ready(&self) -> bool {
        !self.bink.is_null()
    }
}
