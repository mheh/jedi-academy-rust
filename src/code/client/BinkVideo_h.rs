#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// From bink.h - opaque Bink type (local stub for structural coherence)
// Partial definition needed for BinkVideo inline method implementations
// ============================================================================

#[repr(C)]
pub struct BINK {
    pub Width: c_int,
    pub Height: c_int,
    // Other fields omitted; only Width and Height are accessed by this header
}

pub type HBINK = *mut BINK;

// ============================================================================
// Macros from BinkVideo.h
// ============================================================================

pub const NS_BV_DEFAULT_CIN_BPS: c_int = 4;
pub const MAX_WIDTH: c_int = 512;
pub const MAX_HEIGHT: c_int = 512;

pub const XBOX_BUFFER_SIZE: usize =
    (NS_BV_DEFAULT_CIN_BPS as usize) * (MAX_WIDTH as usize) * (MAX_HEIGHT as usize);
pub const XBOX_BINK_SND_MEM: c_int = 16448;

// ============================================================================
// Enum constants for BinkVideo status
// ============================================================================

// Movie is playing
pub const NS_BV_PLAYING: c_int = 0;
// Movie is stopped
pub const NS_BV_STOPPED: c_int = 1;
// Movie is paused
pub const NS_BV_PAUSED: c_int = 2;

// ============================================================================
// BinkVideo class translated to Rust struct
// ============================================================================

#[repr(C)]
pub struct BinkVideo {
    // Private members
    bink: HBINK,
    buffer: *mut c_void,
    texture: c_int,
    status: c_int,
    looping: bool,
    alpha: bool,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    w: f32,
    h: f32,
    initialized: bool,
}

impl BinkVideo {
    // BinkVideo();
    pub fn BinkVideo() -> Self {
        todo!()
    }

    // ~BinkVideo();
    pub fn destruct(self) {
        todo!()
    }

    // bool	Start(const char *filename, float xOrigin, float yOrigin, float width, float height);
    pub fn Start(&mut self, filename: *const c_char, xOrigin: f32, yOrigin: f32, width: f32, height: f32) -> bool {
        todo!()
    }

    // bool	Run(void);
    pub fn Run(&mut self) -> bool {
        todo!()
    }

    // void	Stop(void);
    pub fn Stop(&mut self) {
        todo!()
    }

    // void	Pause(void);
    pub fn Pause(&mut self) {
        todo!()
    }

    // void	SetExtents(float xOrigin, float yOrigin, float width, float height);
    pub fn SetExtents(&mut self, xOrigin: f32, yOrigin: f32, width: f32, height: f32) {
        todo!()
    }

    // int		GetStatus(void) { return status; }
    #[inline]
    pub fn GetStatus(&self) -> c_int {
        self.status
    }

    // void	SetLooping(bool loop) { looping = loop; }
    #[inline]
    pub fn SetLooping(&mut self, r#loop: bool) {
        self.looping = r#loop;
    }

    // void*	GetBinkData(void);
    pub fn GetBinkData(&self) -> *mut c_void {
        todo!()
    }

    // int		GetBinkWidth(void) { return this->bink->Width; }
    #[inline]
    pub fn GetBinkWidth(&self) -> c_int {
        unsafe { (*self.bink).Width }
    }

    // int		GetBinkHeight(void) { return this->bink->Height; }
    #[inline]
    pub fn GetBinkHeight(&self) -> c_int {
        unsafe { (*self.bink).Height }
    }

    // void	SetMasterVolume(s32 volume);
    pub fn SetMasterVolume(&mut self, volume: c_int) {
        todo!()
    }

    // void	AllocateXboxMem(void);
    pub fn AllocateXboxMem(&mut self) {
        todo!()
    }

    // void	FreeXboxMem(void);
    pub fn FreeXboxMem(&mut self) {
        todo!()
    }

    // static void*	Allocate(U32 size);
    pub fn Allocate(size: u32) -> *mut c_void {
        todo!()
    }

    // static void		Free(void* ptr);
    pub fn Free(ptr: *mut c_void) {
        todo!()
    }

    // bool	Ready(void) { return (bool)this->bink; }
    #[inline]
    pub fn Ready(&self) -> bool {
        !self.bink.is_null()
    }

    // Private methods

    // void	Draw(void);
    fn Draw(&mut self) {
        todo!()
    }

    // S32		DecompressFrame();
    fn DecompressFrame(&mut self) -> c_int {
        todo!()
    }
}
