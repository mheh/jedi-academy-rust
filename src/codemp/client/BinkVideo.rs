#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint, c_float};

// ============================================================================
// Globals
// ============================================================================

pub static mut bvUseGCTexMem: bool = true;

#[cfg(target_env = "xbox")]
pub static mut memMarker: c_int = 0;
#[cfg(target_env = "xbox")]
pub static mut binkXboxStartAddr: *mut c_char = core::ptr::null_mut();
#[cfg(target_env = "xbox")]
pub static mut binkXboxCurrentAddr: *mut c_char = core::ptr::null_mut();
#[cfg(target_env = "xbox")]
pub static mut binkXboxNextAddr: *mut c_char = core::ptr::null_mut();

// ============================================================================
// External declarations
// ============================================================================

extern "C" {
	fn RADSetMemory(alloc: unsafe extern "C" fn(u32) -> *mut c_void, free: unsafe extern "C" fn(*mut c_void));
	fn RADSetAudioMemory(alloc: *mut c_void, free: *mut c_void);
	fn BinkSetSoundTrack(track_count: u32, track_ids: *const u32);
	fn BinkSetMixBins(bink: *mut c_void, track: u32, bins: *const u32, bin_count: u32);
	fn BinkSoundUseNGCSound();
	fn BinkOpen(filename: *const c_char, flags: u32) -> *mut c_void;
	fn BinkClose(bink: *mut c_void);
	fn BinkWait(bink: *mut c_void) -> c_int;
	fn BinkNextFrame(bink: *mut c_void);
	fn BinkDoFrame(bink: *mut c_void);
	fn BinkCopyToBuffer(bink: *mut c_void, buffer: *mut c_void, pitch: u32, height: u32, x: u32, y: u32, flags: u32) -> i32;
	fn BinkSetVolume(bink: *mut c_void, track: c_int, volume: i32);
	fn Z_Malloc(size: usize, tag: u32, clear: bool, align: c_int) -> *mut c_void;
	fn Z_Free(ptr: *mut c_void);
	fn RoundUp(value: usize, align: usize) -> usize;
	fn GLW_TexCacheLock();
	fn GLW_TexCacheUnlock();
	fn GLW_TexCacheAllocRaw(size: c_int) -> *mut c_void;
	fn GLW_TexCacheFreeRaw(ptr: *mut c_void);
	fn S_DrainRawSoundData();
	fn GL_SelectTexture(unit: c_int);
	fn RB_SetGL2D();
	fn qglFlush();
	fn qglDeleteTextures(n: c_int, textures: *const c_uint);
	fn qglGenTextures(n: c_int, textures: *mut c_uint);
	fn qglBindTexture(target: u32, texture: c_uint);
	fn qglTexImage2D(target: u32, level: c_int, internalformat: u32, width: c_int, height: c_int, border: c_int, format: u32, type_: u32, pixels: *const c_void);
	fn qglTexParameterf(target: u32, pname: u32, param: f32);
	fn qglTexSubImage2D(target: u32, level: c_int, xoffset: c_int, yoffset: c_int, width: c_int, height: c_int, format: u32, type_: u32, pixels: *const c_void);
	fn qglColor3f(red: f32, green: f32, blue: f32);
	fn qglBegin(mode: u32);
	fn qglBeginEXT(mode: u32, count: c_int, unk1: c_int, unk2: c_int, unk3: c_int, unk4: c_int);
	fn qglTexCoord2f(s: f32, t: f32);
	fn qglVertex2f(x: f32, y: f32);
	fn qglEnd();

	static mut glState: GlState;

	fn alGeti(param: c_int, value: *mut c_int);
}

// ============================================================================
// External structs (stubs)
// ============================================================================

#[repr(C)]
pub struct GlState {
	pub currenttextures: [c_uint; 2],
	pub currenttmu: usize,
	// ... other fields omitted, only needed for currenttextures access
}

// Bink structure stub - contains Width and Height at the beginning,
// and FrameNum/Frames at known offsets
#[repr(C)]
pub struct BinkStruct {
	pub Width: c_int,
	pub Height: c_int,
	pub FrameNum: i32,
	pub Frames: i32,
	// ... rest of the structure is opaque to us
}

// ============================================================================
// Constants (from headers)
// ============================================================================

const NS_BV_PLAYING: c_int = 0;
const NS_BV_STOPPED: c_int = 1;
const NS_BV_PAUSED: c_int = 2;

const NS_BV_DEFAULT_CIN_BPS: c_int = 2;
const MAX_WIDTH: c_int = 512;
const MAX_HEIGHT: c_int = 512;

const BINKSNDTRACK: u32 = 0x1;
const BINKCOPYALL: u32 = 0x80000000;
const BINKSURFACE565: u32 = 0x0;

const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_RGB5: u32 = 0x8050;
const GL_RGB_SWIZZLE_EXT: u32 = 0x8C60;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
const GL_LINEAR: u32 = 0x2601;
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_CLAMP: u32 = 0x2900;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_TRIANGLE_STRIP: u32 = 0x0005;

const TAG_BINK: u32 = 7;
const DSMIXBIN_FRONT_LEFT: u32 = 0;
const DSMIXBIN_FRONT_RIGHT: u32 = 1;
const DSMIXBIN_FRONT_CENTER: u32 = 2;
const DSMIXBIN_LOW_FREQUENCY: u32 = 3;
const DSMIXBIN_BACK_LEFT: u32 = 4;
const DSMIXBIN_BACK_RIGHT: u32 = 5;

const AL_MEMORY_ALLOCATOR: c_int = 0x100009;
const AL_MEMORY_DEALLOCATOR: c_int = 0x10000A;

#[cfg(target_env = "xbox")]
const XBOX_MEM_STAGE_1: usize = 32640;
#[cfg(target_env = "xbox")]
const XBOX_MEM_STAGE_2: usize = 786528;
#[cfg(target_env = "xbox")]
const XBOX_MEM_STAGE_3: usize = 557152;
#[cfg(target_env = "xbox")]
const XBOX_MEM_STAGE_4: usize = 106560;
#[cfg(target_env = "xbox")]
const XBOX_MEM_STAGE_5: usize = 138304;
#[cfg(target_env = "xbox")]
const XBOX_MEM_STAGE_6: usize = 25696;
#[cfg(target_env = "xbox")]
const XBOX_MEM_STAGE_7: usize = 100;
#[cfg(target_env = "xbox")]
const XBOX_MEM_STAGE_8: usize = 100;
#[cfg(target_env = "xbox")]
const XBOX_BUFFER_SIZE: usize = NS_BV_DEFAULT_CIN_BPS as usize * MAX_WIDTH as usize * MAX_HEIGHT as usize;

// ============================================================================
// Static wrapper functions
// ============================================================================

unsafe extern "C" fn AllocWrapper(size: u32) -> *mut c_void {
	#[cfg(target_env = "xbox")]
	{
		// Give bink pre-initialized mem on xbox
		match memMarker {
			0 => {
				memMarker += 1;
				binkXboxCurrentAddr = binkXboxStartAddr;
				binkXboxNextAddr = binkXboxCurrentAddr.add(size as usize);
				binkXboxStartAddr as *mut c_void
			}
			1 | 2 | 3 | 4 | 5 | 6 | 7 => {
				memMarker += 1;
				binkXboxCurrentAddr = binkXboxNextAddr;
				binkXboxNextAddr = binkXboxCurrentAddr.add(size as usize);
				binkXboxCurrentAddr as *mut c_void
			}
			8 => {
				memMarker = -1;
				binkXboxCurrentAddr = binkXboxNextAddr;
				binkXboxNextAddr = binkXboxStartAddr;
				binkXboxCurrentAddr as *mut c_void
			}
			_ => BinkVideo::Allocate(size),
		}
	}

	#[cfg(not(target_env = "xbox"))]
	{
		BinkVideo::Allocate(size)
	}
}

unsafe extern "C" fn FreeWrapper(ptr: *mut c_void) {
	#[cfg(target_env = "xbox")]
	{
		// Don't free the preinitialized mem
		if memMarker < 6 {
			memMarker += 1;
			return;
		} else if memMarker == 6 {
			memMarker = 1;
			binkXboxNextAddr = binkXboxStartAddr.add(XBOX_MEM_STAGE_1);
			return;
		}
	}

	BinkVideo::Free(ptr);
}

// ============================================================================
// BinkVideo struct
// ============================================================================

pub struct BinkVideo {
	pub bink: *mut c_void,
	pub buffer: *mut c_void,
	pub texture: c_int,
	pub x1: c_float,
	pub y1: c_float,
	pub x2: c_float,
	pub y2: c_float,
	pub w: c_float,
	pub h: c_float,
	pub status: c_int,
	pub looping: bool,
	#[cfg(target_env = "xbox")]
	pub initialized: bool,
}

// ============================================================================
// BinkVideo implementation
// ============================================================================

impl BinkVideo {
	/*********
	BinkVideo
	*********/
	pub fn new() -> Self {
		BinkVideo {
			bink: core::ptr::null_mut(),
			buffer: core::ptr::null_mut(),
			texture: 0,
			x1: 0.0,
			y1: 0.0,
			x2: 0.0,
			y2: 0.0,
			w: 0.0,
			h: 0.0,
			status: NS_BV_STOPPED,
			looping: false,
			#[cfg(target_env = "xbox")]
			initialized: false,
		}
	}

	/*********
	~BinkVideo
	*********/
	pub fn drop(&mut self) {
		unsafe {
			Self::Free(self.buffer);
			BinkClose(self.bink);
		}
	}

	/*********
	AllocateXboxMem
	Pre-Allocates memory for xbox
	*********/
	#[cfg(target_env = "xbox")]
	pub fn AllocateXboxMem(&mut self) {
		unsafe {
			let memToAllocate: usize = XBOX_MEM_STAGE_1
				+ XBOX_MEM_STAGE_2
				+ XBOX_MEM_STAGE_3
				+ XBOX_MEM_STAGE_4
				+ XBOX_MEM_STAGE_5
				+ XBOX_MEM_STAGE_6
				+ XBOX_MEM_STAGE_7
				+ XBOX_MEM_STAGE_8
				+ XBOX_BUFFER_SIZE;
			binkXboxStartAddr = Self::Allocate(memToAllocate as u32) as *mut c_char;
			memMarker = 0;
			self.initialized = true;
		}
	}

	/*********
	FreeXboxMem
	*********/
	#[cfg(target_env = "xbox")]
	pub fn FreeXboxMem(&mut self) {
		unsafe {
			self.initialized = false;
			Z_Free(binkXboxStartAddr as *mut c_void);
			memMarker = 0;
		}
	}

	/*********
	Start
	Opens a bink file and gets it ready to play
	*********/
	pub fn Start(&mut self, filename: *const c_char, xOrigin: c_float, yOrigin: c_float, width: c_float, height: c_float) -> bool {
		unsafe {
			#[cfg(target_env = "xbox")]
			{
				assert!(self.initialized);
			}

			// Check to see if a video is being played.
			if self.status == NS_BV_PLAYING {
				// stop
				self.Stop();
			}

			// Set memory allocation wrapper
			RADSetMemory(AllocWrapper, FreeWrapper);

			// Set up sound for consoles
			#[cfg(target_env = "xbox")]
			{
				// If we are on XBox, tell Bink to play all of the 5.1 tracks
				let TrackIDsToPlay: [u32; 4] = [0, 1, 2, 3];
				BinkSetSoundTrack(4, &TrackIDsToPlay[0]);

				// Now route the sound tracks to the correct speaker
				let mut bins: [u32; 2] = [0; 2];

				bins[0] = DSMIXBIN_FRONT_LEFT;
				bins[1] = DSMIXBIN_FRONT_RIGHT;
				BinkSetMixBins(self.bink, 0, &bins[0], 2);
				bins[0] = DSMIXBIN_FRONT_CENTER;
				BinkSetMixBins(self.bink, 1, &bins[0], 1);
				bins[0] = DSMIXBIN_LOW_FREQUENCY;
				BinkSetMixBins(self.bink, 2, &bins[0], 1);
				bins[0] = DSMIXBIN_BACK_LEFT;
				bins[1] = DSMIXBIN_BACK_RIGHT;
				BinkSetMixBins(self.bink, 3, &bins[0], 2);
			}

			#[cfg(target_env = "gamecube")]
			{
				BinkSoundUseNGCSound();

				let mut a: c_int = 0;
				alGeti(AL_MEMORY_ALLOCATOR, &mut a);

				let mut f: c_int = 0;
				alGeti(AL_MEMORY_DEALLOCATOR, &mut f);

				RADSetAudioMemory(a as *mut c_void, f as *mut c_void);
			}

			// Try to open the Bink file.
			#[cfg(target_env = "xbox")]
			{
				self.bink = BinkOpen(filename, BINKSNDTRACK);
				if self.bink.is_null() {
					return false;
				}
			}

			#[cfg(target_env = "gamecube")]
			{
				if bvUseGCTexMem {
					GLW_TexCacheLock();
				}

				self.bink = BinkOpen(filename, 0);

				if self.bink.is_null() {
					GLW_TexCacheUnlock();
					return false;
				}
			}

			let bink_struct = &*(self.bink as *const BinkStruct);
			assert!(bink_struct.Width <= MAX_WIDTH && bink_struct.Height <= MAX_HEIGHT);

			// allocate memory for the frame buffer
			#[cfg(target_env = "xbox")]
			{
				self.buffer = AllocWrapper(XBOX_BUFFER_SIZE as u32);
			}

			#[cfg(target_env = "gamecube")]
			{
				self.buffer = Self::Allocate(XBOX_BUFFER_SIZE as u32);
			}

			// set the height, width, etc...
			self.x1 = xOrigin;
			self.y1 = yOrigin;
			self.x2 = self.x1 + width;
			self.y2 = self.y1 + height;
			self.w = width;
			self.h = height;

			// flush any background sound reads
			#[cfg(any(target_env = "xbox", target_env = "gamecube"))]
			{
				S_DrainRawSoundData();
			}

			// Create the video texture
			let mut tex: c_uint = self.texture as c_uint;
			if tex != 0 {
				qglDeleteTextures(1, &tex);
			}

			qglGenTextures(1, &mut tex);
			qglBindTexture(GL_TEXTURE_2D, tex);
			glState.currenttextures[glState.currenttmu] = tex;

			let bink_struct = &*(self.bink as *const BinkStruct);
			qglTexImage2D(GL_TEXTURE_2D, 0, GL_RGB5, bink_struct.Width, bink_struct.Height, 0,
				GL_RGB_SWIZZLE_EXT, GL_UNSIGNED_BYTE, self.buffer);

			qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as f32);
			qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as f32);
			qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP as f32);
			qglTexParameterf(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP as f32);

			self.texture = tex as c_int;

			self.status = NS_BV_PLAYING;

			true
		}
	}

	/*********
	Run
	Decompresses a frame, renders it to the screen, and advances to
	the next frame.
	*********/
	pub fn Run(&mut self) -> bool {
		unsafe {
			if self.status == NS_BV_STOPPED {
				// A movie can't be run if it's not started first
				return false;
			}

			while BinkWait(self.bink) != 0 {} // Wait

			self.DecompressFrame(); // Decompress
			self.Draw(); // Render

			if self.status != NS_BV_PAUSED {
				// Only advance the frame is not paused
				BinkNextFrame(self.bink);
			}

			let bink_struct = &*(self.bink as *const BinkStruct);
			if bink_struct.FrameNum == (bink_struct.Frames - 1) && !self.looping {
				// The movie is done
				self.Stop();
				return false;
			}

			true
		}
	}

	/*********
	GetBinkData
	Returns the buffer data for the next frame of the video
	*********/
	pub fn GetBinkData(&mut self) -> *mut c_void {
		unsafe {
			while BinkWait(self.bink) != 0 {}
			self.DecompressFrame();
			BinkNextFrame(self.bink);
			self.buffer
		}
	}

	/********
	Draw
	Copies the decompressed frame to a texture to be rendered on
	the screen.
	********/
	pub fn Draw(&mut self) {
		unsafe {
			if !self.buffer.is_null() {
				qglFlush();

				RB_SetGL2D();

				GL_SelectTexture(0);

				// Update the video texture
				qglBindTexture(GL_TEXTURE_2D, self.texture as c_uint);
				glState.currenttextures[glState.currenttmu] = self.texture as c_uint;

				let bink_struct = &*(self.bink as *const BinkStruct);
				qglTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, bink_struct.Width, bink_struct.Height,
					GL_RGB_SWIZZLE_EXT, GL_UNSIGNED_BYTE, self.buffer);

				// Clear the screen.  We use triangles here (instead
				// of glClear) because we want the back buffer to stick
				// around... so we can get a nice, cheap fade on Gamecube
				// reset.
				qglColor3f(0.0, 0.0, 0.0);
				#[cfg(any(target_env = "xbox", target_env = "gamecube"))]
				{
					qglBeginEXT(GL_TRIANGLE_STRIP, 4, 0, 0, 4, 0);
				}
				#[cfg(not(any(target_env = "xbox", target_env = "gamecube")))]
				{
					qglBegin(GL_TRIANGLE_STRIP);
				}
				qglTexCoord2f(0.0, 0.0);
				qglVertex2f(-10.0, -10.0);
				qglTexCoord2f(1.0, 0.0);
				qglVertex2f(650.0, -10.0);
				qglTexCoord2f(0.0, 1.0);
				qglVertex2f(-10.0, 490.0);
				qglTexCoord2f(1.0, 1.0);
				qglVertex2f(650.0, 490.0);
				qglEnd();

				// Draw the video
				qglColor3f(1.0, 1.0, 1.0);
				#[cfg(any(target_env = "xbox", target_env = "gamecube"))]
				{
					qglBeginEXT(GL_TRIANGLE_STRIP, 4, 0, 0, 4, 0);
				}
				#[cfg(not(any(target_env = "xbox", target_env = "gamecube")))]
				{
					qglBegin(GL_TRIANGLE_STRIP);
				}
				qglTexCoord2f(0.0, 0.0);
				qglVertex2f(self.x1, self.y1);
				qglTexCoord2f(1.0, 0.0);
				qglVertex2f(self.x2, self.y1);
				qglTexCoord2f(0.0, 1.0);
				qglVertex2f(self.x1, self.y2);
				qglTexCoord2f(1.0, 1.0);
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
		unsafe {
			BinkClose(self.bink);
			self.bink = core::ptr::null_mut();
			#[cfg(target_env = "xbox")]
			{
				FreeWrapper(self.buffer);
			}
			#[cfg(target_env = "gamecube")]
			{
				Self::Free(self.buffer);
			}
			self.buffer = core::ptr::null_mut();

			let mut tex: c_uint = self.texture as c_uint;
			if tex != 0 {
				qglDeleteTextures(1, &tex);
			}

			self.texture = 0;
			self.x1 = 0.0;
			self.y1 = 0.0;
			self.x2 = 0.0;
			self.y2 = 0.0;
			self.w = 0.0;
			self.h = 0.0;
			self.status = NS_BV_STOPPED;
			#[cfg(target_env = "xbox")]
			{
				memMarker = 0;
			}

			#[cfg(target_env = "gamecube")]
			{
				GLW_TexCacheUnlock();
			}
		}
	}

	/*********
	Pause
	Pauses the current movie. Only the current frame is rendered
	*********/
	pub fn Pause(&mut self) {
		self.status = NS_BV_PAUSED;
	}

	/*********
	SetExtends
	Sets dimmension variables
	*********/
	pub fn SetExtents(&mut self, xOrigin: c_float, yOrigin: c_float, width: c_float, height: c_float) {
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
	pub fn SetMasterVolume(&mut self, volume: i32) {
		unsafe {
			#[cfg(target_env = "xbox")]
			{
				for i in 0..4 {
					BinkSetVolume(self.bink, i, volume);
				}
			}
			#[cfg(not(target_env = "xbox"))]
			{
				BinkSetVolume(self.bink, 0, volume);
			}
		}
	}

	/*********
	DecompressFrame
	Decompresses current frame and copies the data to
	the buffer
	*********/
	pub fn DecompressFrame(&mut self) -> i32 {
		unsafe {
			BinkDoFrame(self.bink);

			let skip: i32;
			let bink_struct = &*(self.bink as *const BinkStruct);
			skip = BinkCopyToBuffer(
				self.bink,
				self.buffer,
				(NS_BV_DEFAULT_CIN_BPS * bink_struct.Width) as u32, // pitch
				bink_struct.Height as u32,
				0,
				0,
				BINKCOPYALL | BINKSURFACE565);
			skip
		}
	}

	/*********
	Allocate
	Allocates memory for the frame buffer
	*********/
	pub fn Allocate(size: u32) -> *mut c_void {
		unsafe {
			let mut size = RoundUp(size as usize + 32, 32);
			let mut ptr: *mut c_char = core::ptr::null_mut();

			#[cfg(target_env = "gamecube")]
			{
				if bvUseGCTexMem {
					// Try allocating from texture cache
					ptr = GLW_TexCacheAllocRaw(size as c_int) as *mut c_char;
				}
			}

			if ptr.is_null() {
				// Did not allocate texture cache memory, fall
				// back to main memory..
				ptr = Z_Malloc(size, TAG_BINK, false, 32) as *mut c_char;
				*ptr = b'z' as c_char;
			} else {
				// Allocated memory from the texture cache
				*ptr = b't' as c_char;
			}

			(ptr.add(32)) as *mut c_void
		}
	}

	/*********
	FreeBuffer
	Releases the frame buffer memory
	*********/
	pub fn Free(ptr: *mut c_void) {
		unsafe {
			let base: *mut c_char = (ptr as *mut c_char).sub(32);

			match *base as u8 {
				#[cfg(target_env = "gamecube")]
				b't' => {
					// Free texture cache memory
					GLW_TexCacheFreeRaw(base as *mut c_void);
				}

				b'z' => {
					// Free main memory
					Z_Free(base as *mut c_void);
				}

				_ => {
					assert!(false);
				}
			}
		}
	}
}
