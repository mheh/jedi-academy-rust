#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::code::client::BinkVideo_h::{
    self, HBINK, NS_BV_DEFAULT_CIN_BPS, MAX_WIDTH, MAX_HEIGHT,
    XBOX_BUFFER_SIZE, XBOX_BINK_SND_MEM, NS_BV_PLAYING, NS_BV_STOPPED, NS_BV_PAUSED,
    BinkVideo
};

// Extended BINK struct for this file's internal access to additional fields
// The actual BINK library has these fields; this struct provides access to them
#[repr(C)]
struct BINK_Full {
    pub Width: c_int,
    pub Height: c_int,
    pub FrameNum: c_int,
    pub Frames: c_int,
    pub OpenFlags: u32,
}

/*
 * This version of BinkVideo.cpp now ONLY works on Xbox.
 * GCN support is hosed.
 */

// ============================================================================
// Global and wrapper functions
// ============================================================================

pub static mut binkSndMem: *mut c_char = ptr::null_mut();

extern "C" {
    fn BinkOpen(filename: *const c_char, flags: u32) -> HBINK;
    fn BinkClose(bink: HBINK);
    fn BinkWait(bink: HBINK) -> c_int;
    fn BinkNextFrame(bink: HBINK);
    fn BinkDoFrame(bink: HBINK);
    fn BinkCopyToBuffer(
        bink: HBINK,
        buffer: *mut c_void,
        pitch: c_int,
        height: c_int,
        x: c_int,
        y: c_int,
        flags: u32,
    ) -> c_int;
    fn BinkSetSoundTrack(count: u32, track_ids: *const u32);
    fn BinkSetMixBins(bink: HBINK, track: u32, bins: *const u32, bin_count: u32);
    fn BinkSetVolume(bink: HBINK, track: u32, volume: c_int);

    fn RADSetMemory(alloc: *const c_void, free: *const c_void);

    fn Z_Malloc(size: usize, tag: c_int, zero: c_int, align: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    fn S_DrainRawSoundData();
    fn RB_SetGL2D();

    fn qglDeleteTextures(n: c_int, textures: *const u32);
    fn qglGenTextures(n: c_int, textures: *mut u32);
    fn qglBindTexture(target: u32, texture: u32);
    fn qglTexImage2D(
        target: u32,
        level: c_int,
        internalformat: c_int,
        width: c_int,
        height: c_int,
        border: c_int,
        format: u32,
        img_type: u32,
        pixels: *const c_void,
    );
    fn qglTexParameterf(target: u32, pname: u32, param: f32);
    fn qglTexSubImage2D(
        target: u32,
        level: c_int,
        xoffset: c_int,
        yoffset: c_int,
        width: c_int,
        height: c_int,
        format: u32,
        img_type: u32,
        pixels: *const c_void,
    );
    fn qglColor3f(red: f32, green: f32, blue: f32);
    fn qglBeginEXT(mode: u32, count: c_int, unused1: c_int, unused2: c_int, unused3: c_int, unused4: c_int);
    fn qglBegin(mode: u32);
    fn qglTexCoord2f(s: f32, t: f32);
    fn qglVertex2f(x: f32, y: f32);
    fn qglEnd();
    fn qglFlush();

    fn GL_SelectTexture(unit: c_int);

    // glState and related globals
    static mut glState: GLState;
}

#[repr(C)]
pub struct GLState {
    pub currenttextures: [u32; 2],
    pub currenttmu: c_int,
    // ... other fields not accessed by this file
}

// ============================================================================
// Bink constants/macros (from bink.h, approximate)
// ============================================================================

const BINKSNDTRACK: u32 = 0x00000004;
const BINKALPHA: u32 = 0x00000800;
const BINKCOPYALL: u32 = 0x00000001;
const BINKSURFACE32: u32 = 0x00000200;
const BINKSURFACE32A: u32 = 0x00000400;

// ============================================================================
// DirectSound mix bins (Xbox constants)
// ============================================================================

const DSMIXBIN_FRONT_LEFT: u32 = 0;
const DSMIXBIN_FRONT_RIGHT: u32 = 1;
const DSMIXBIN_FRONT_CENTER: u32 = 2;
const DSMIXBIN_LOW_FREQUENCY: u32 = 3;
const DSMIXBIN_BACK_LEFT: u32 = 4;
const DSMIXBIN_BACK_RIGHT: u32 = 5;

// ============================================================================
// GL constants
// ============================================================================

const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_LINEAR: f32 = 0x2601 as f32;
const GL_CLAMP: f32 = 0x2900 as f32;
const GL_TRIANGLE_STRIP: u32 = 0x0005;
const GL_LIN_RGB8: c_int = 0x8051;
const GL_LIN_RGBA8: c_int = 0x8058;
const GL_LIN_RGB: u32 = 0x1907;
const GL_LIN_RGBA: u32 = 0x1908;
const GL_UNSIGNED_BYTE: u32 = 0x1401;

const TAG_BINK: c_int = 9;
const qfalse: c_int = 0;

// ============================================================================
// Wrapper functions for memory allocation
// ============================================================================

extern "C" fn AllocWrapper(size: u32) -> *mut c_void {
    // Give bink pre-initialized sound mem on xbox
    if size == XBOX_BINK_SND_MEM as u32 {
        unsafe { return binkSndMem as *mut c_void; }
    }

    BinkVideo::Allocate(size)
}

extern "C" fn FreeWrapper(ptr: *mut c_void) {
    BinkVideo::Free(ptr);
}

// ============================================================================
// BinkVideo implementation
// ============================================================================

impl BinkVideo {
    /*********
    BinkVideo
    *********/
    pub fn BinkVideo() -> Self {
        BinkVideo {
            bink: ptr::null_mut(),
            buffer: ptr::null_mut(),
            texture: 0,
            x1: 0.0f32,
            y1: 0.0f32,
            x2: 0.0f32,
            y2: 0.0f32,
            w: 0.0f32,
            h: 0.0f32,
            status: NS_BV_STOPPED,
            looping: false,
            alpha: false,
            initialized: false,
        }
    }

    /*********
    ~BinkVideo
    *********/
    pub fn destruct(&mut self) {
        Self::Free(self.buffer);
        unsafe {
            BinkClose(self.bink);
        }
    }

    /*********
    AllocateXboxMem
    Pre-Allocates sound memory for xbox to avoid fragmenting
    *********/
    pub fn AllocateXboxMem(&mut self) {
        unsafe {
            binkSndMem = Self::Allocate(XBOX_BINK_SND_MEM as u32) as *mut c_char;
            self.initialized = true;
        }
    }

    /*********
    FreeXboxMem
    *********/
    pub fn FreeXboxMem(&mut self) {
        self.initialized = false;
        unsafe {
            Z_Free(binkSndMem as *mut c_void);
        }
    }

    /*********
    Start
    Opens a bink file and gets it ready to play
    *********/
    pub fn Start(&mut self, filename: *const c_char, xOrigin: f32, yOrigin: f32, width: f32, height: f32) -> bool {
        assert!(self.initialized);

        // Check to see if a video is being played.
        if self.status == NS_BV_PLAYING {
            // stop
            self.Stop();
        }

        // Set memory allocation wrapper
        unsafe {
            RADSetMemory(
                &AllocWrapper as *const _ as *const c_void,
                &FreeWrapper as *const _ as *const c_void,
            );
        }

        // Set up sound for consoles

        // We are on XBox, tell Bink to play all of the 5.1 tracks
        let mut TrackIDsToPlay: [u32; 4] = [0, 1, 2, 3];
        unsafe {
            BinkSetSoundTrack(4, TrackIDsToPlay.as_ptr());
        }

        // Now route the sound tracks to the correct speaker
        let mut bins: [u32; 2] = [0; 2];

        unsafe {
            bins[0] = DSMIXBIN_FRONT_LEFT;
            bins[1] = DSMIXBIN_FRONT_RIGHT;
            BinkSetMixBins(self.bink, 0, bins.as_ptr(), 2);
            bins[0] = DSMIXBIN_FRONT_CENTER;
            BinkSetMixBins(self.bink, 1, bins.as_ptr(), 1);
            bins[0] = DSMIXBIN_LOW_FREQUENCY;
            BinkSetMixBins(self.bink, 2, bins.as_ptr(), 1);
            bins[0] = DSMIXBIN_BACK_LEFT;
            bins[1] = DSMIXBIN_BACK_RIGHT;
            BinkSetMixBins(self.bink, 3, bins.as_ptr(), 2);
        }

        // Try to open the Bink file.
        unsafe {
            self.bink = BinkOpen(filename, BINKSNDTRACK | BINKALPHA);
        }
        if self.bink.is_null() {
            return false;
        }

        unsafe {
            let bink_full = self.bink as *mut BINK_Full;
            assert!((*bink_full).Width <= MAX_WIDTH && (*bink_full).Height <= MAX_HEIGHT);
        }

        // allocate memory for the frame buffer
        self.buffer = AllocWrapper(XBOX_BUFFER_SIZE as u32);

        // set the height, width, etc...
        self.x1 = xOrigin;
        self.y1 = yOrigin;
        self.x2 = self.x1 + width;
        self.y2 = self.y1 + height;
        self.w = width;
        self.h = height;
        // Did the source .bik file have an alpha plane?
        unsafe {
            let bink_full = self.bink as *mut BINK_Full;
            self.alpha = ((*bink_full).OpenFlags & BINKALPHA) != 0;
        }

        // flush any background sound reads
        unsafe {
            S_DrainRawSoundData();
        }

        // Create the video texture
        unsafe {
            let bink_full = self.bink as *mut BINK_Full;
            let mut tex = self.texture as u32;
            if tex != 0 {
                qglDeleteTextures(1, &tex);
            }

            qglGenTextures(1, &mut tex);
            qglBindTexture(GL_TEXTURE_2D, tex);
            glState.currenttextures[glState.currenttmu as usize] = tex;

            let internal_format = if self.alpha { GL_LIN_RGBA8 } else { GL_LIN_RGB8 };
            let format = if self.alpha { GL_LIN_RGBA } else { GL_LIN_RGB };

            qglTexImage2D(
                GL_TEXTURE_2D,
                0,
                internal_format,
                (*bink_full).Width,
                (*bink_full).Height,
                0,
                format,
                GL_UNSIGNED_BYTE,
                self.buffer,
            );

            qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
            qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
            qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP);
            qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP);

            self.texture = tex as c_int;
        }

        self.status = NS_BV_PLAYING;

        return true;
    }

    /*********
    Run
    Decompresses a frame, renders it to the screen, and advances to
    the next frame.
    *********/
    pub fn Run(&mut self) -> bool {
        if self.status == NS_BV_STOPPED {
            // A movie can't be run if it's not started first
            return false;
        }

        unsafe {
            while BinkWait(self.bink) != 0 {
                // Wait
            }
        }

        self.DecompressFrame(); // Decompress
        self.Draw(); // Render

        if self.status != NS_BV_PAUSED {
            // Only advance the frame is not paused
            unsafe {
                BinkNextFrame(self.bink);
            }
        }

        unsafe {
            let bink_full = self.bink as *mut BINK_Full;
            if (*bink_full).FrameNum == ((*bink_full).Frames - 1) && !self.looping {
                // The movie is done
                self.Stop();
                return false;
            }
        }

        return true;
    }

    /*********
    GetBinkData
    Returns the buffer data for the next frame of the video
    *********/
    pub fn GetBinkData(&mut self) -> *mut c_void {
        //while(BinkWait(bink));
        // This doesn't follow Bink guidelines. They suggest that you call BinkWait()
        // very frequently, something like 4 to 5 times as fast as the framerate of
        // the movie. We're technically coming close to that, but this code won't work
        // if we have videoMap shaders with higher framerates than the planets (8).
        unsafe {
            if BinkWait(self.bink) == 0 {
                self.DecompressFrame();
                BinkNextFrame(self.bink);
            }
        }
        return self.buffer;
    }

    /********
    Draw
    Copies the decompressed frame to a texture to be rendered on
    the screen.
    ********/
    fn Draw(&mut self) {
        if !self.buffer.is_null() {
            unsafe {
                let bink_full = self.bink as *mut BINK_Full;
                qglFlush();

                RB_SetGL2D();

                GL_SelectTexture(0);

                // Update the video texture
                qglBindTexture(GL_TEXTURE_2D, self.texture as u32);
                glState.currenttextures[glState.currenttmu as usize] = self.texture as u32;

                let format = if self.alpha { GL_LIN_RGBA } else { GL_LIN_RGB };

                qglTexSubImage2D(
                    GL_TEXTURE_2D,
                    0,
                    0,
                    0,
                    (*bink_full).Width,
                    (*bink_full).Height,
                    format,
                    GL_UNSIGNED_BYTE,
                    self.buffer,
                );

                // Clear the screen.  We use triangles here (instead
                // of glClear) because we want the back buffer to stick
                // around... so we can get a nice, cheap fade on Gamecube
                // reset.
                qglColor3f(0.0f32, 0.0f32, 0.0f32);
                #[cfg(feature = "xbox")]
                {
                    qglBeginEXT(GL_TRIANGLE_STRIP, 4, 0, 0, 4, 0);
                }
                #[cfg(not(feature = "xbox"))]
                {
                    qglBegin(GL_TRIANGLE_STRIP);
                }
                qglTexCoord2f(0.0f32, 0.0f32);
                qglVertex2f(-10.0f32, -10.0f32);
                qglTexCoord2f((*bink_full).Width as f32, 0.0f32);
                qglVertex2f(650.0f32, -10.0f32);
                qglTexCoord2f(0.0f32, (*bink_full).Height as f32);
                qglVertex2f(-10.0f32, 490.0f32);
                qglTexCoord2f((*bink_full).Width as f32, (*bink_full).Height as f32);
                qglVertex2f(650.0f32, 490.0f32);
                qglEnd();

                // Draw the video
                qglColor3f(1.0f32, 1.0f32, 1.0f32);
                #[cfg(feature = "xbox")]
                {
                    qglBeginEXT(GL_TRIANGLE_STRIP, 4, 0, 0, 4, 0);
                }
                #[cfg(not(feature = "xbox"))]
                {
                    qglBegin(GL_TRIANGLE_STRIP);
                }
                qglTexCoord2f(0.0f32, 0.0f32);
                qglVertex2f(self.x1, self.y1);
                qglTexCoord2f((*bink_full).Width as f32, 0.0f32);
                qglVertex2f(self.x2, self.y1);
                qglTexCoord2f(0.0f32, (*bink_full).Height as f32);
                qglVertex2f(self.x1, self.y2);
                qglTexCoord2f((*bink_full).Width as f32, (*bink_full).Height as f32);
                qglVertex2f(self.x2, self.y2);
                qglEnd();
            }
        }
    }

    /*********
    Stop
    Stops the current movie, and clears it from memory
    *********/
    pub fn Stop(&mut self) {
        if !self.bink.is_null() {
            unsafe {
                BinkClose(self.bink);
            }
        }
        self.bink = ptr::null_mut();

        if !self.buffer.is_null() {
            FreeWrapper(self.buffer);
        }
        self.buffer = ptr::null_mut();

        unsafe {
            let mut tex = self.texture as u32;
            if tex != 0 {
                qglDeleteTextures(1, &tex);
            }
        }

        self.texture = 0;
        self.x1 = 0.0f32;
        self.y1 = 0.0f32;
        self.x2 = 0.0f32;
        self.y2 = 0.0f32;
        self.w = 0.0f32;
        self.h = 0.0f32;
        self.status = NS_BV_STOPPED;
    }

    /*********
    Pause
    Pauses the current movie. Only the current frame is rendered
    *********/
    pub fn Pause(&mut self) {
        self.status = NS_BV_PAUSED;
    }

    /*********
    SetExtents
    Sets dimmension variables
    *********/
    pub fn SetExtents(&mut self, xOrigin: f32, yOrigin: f32, width: f32, height: f32) {
        self.x1 = xOrigin;
        self.y1 = yOrigin;
        self.x2 = self.x1 + width;
        self.y2 = self.y1 + height;
        self.w = width;
        self.h = height;
    }

    /*********
    SetMasterVolume
    Sets the volume of the specified track
    *********/
    pub fn SetMasterVolume(&mut self, volume: c_int) {
        for i in 0..4 {
            unsafe {
                BinkSetVolume(self.bink, i as u32, volume);
            }
        }
    }

    /*********
    DecompressFrame
    Decompresses current frame and copies the data to
    the buffer
    *********/
    fn DecompressFrame(&mut self) -> c_int {
        unsafe {
            BinkDoFrame(self.bink);
        }

        let skip: c_int;
        unsafe {
            let bink_full = self.bink as *mut BINK_Full;
            skip = BinkCopyToBuffer(
                self.bink,
                self.buffer,
                NS_BV_DEFAULT_CIN_BPS * (*bink_full).Width, //pitch
                (*bink_full).Height,
                0,
                0,
                if self.alpha {
                    BINKCOPYALL | BINKSURFACE32A
                } else {
                    BINKCOPYALL | BINKSURFACE32
                },
            );
        }
        return skip;
    }

    /*********
    Allocate
    Allocates memory for the frame buffer
    *********/
    pub fn Allocate(size: u32) -> *mut c_void {
        unsafe { Z_Malloc(size as usize, TAG_BINK, qfalse, 32) }
    }

    /*********
    FreeBuffer
    Releases the frame buffer memory
    *********/
    pub fn Free(ptr: *mut c_void) {
        unsafe {
            Z_Free(ptr);
        }
    }
}
