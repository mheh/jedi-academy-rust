#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

/*****************************************************************************
 * name:		cl_cin_console.cpp
 *
 * desc:		video and cinematic playback interface for Xbox (using Bink)
 *
 *****************************************************************************/

// Type aliases and constants
pub type connstate_t = c_int;
pub type e_status = c_int;
pub type byte = u8;
pub type qboolean = c_int;

const MAX_OSPATH: usize = 256;

const XBOX_VIDEO_PATH: &[u8] = b"d:\\base\\video\\";
const XBOX_VIDEO_FORMAT: &[u8] = b"d:\\base\\video\\%s.bik\0";

// connstate_t values
const CA_UNINITIALIZED: connstate_t = 0;
const CA_CINEMATIC: connstate_t = 1;

// e_status values
const FMV_EOF: e_status = 0;
const FMV_PLAY: e_status = 1;

// CIN flags
const CIN_loop: c_int = 1;
const CIN_silent: c_int = 2;
const CIN_shader: c_int = 4;

// Bink video status
const NS_BV_STOPPED: c_int = 0;

const qfalse: qboolean = 0;

// BinkVideo opaque C++ class
#[repr(C)]
pub struct BinkVideo {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CinematicData {
    pub filename: [c_char; MAX_OSPATH],	// No path, no extension
    pub x: c_int,
    pub y: c_int,
    pub w: c_int,				// Dimensions
    pub h: c_int,
    pub bits: c_int,				// Flags (loop, silent, shader)
}

// External dependencies from client.h and other headers
extern "C" {
    pub static mut cls: ClientState;
    pub static mut kg: KeyGlobals;

    pub fn Com_Printf(fmt: *const c_char, ...) -> ();
    pub fn Cmd_Argv(i: c_int) -> *const c_char;
    pub fn COM_SkipPath(path: *const c_char) -> *const c_char;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Key_ClearStates() -> ();
    pub fn SCR_UpdateScreen() -> ();
    pub fn IN_Frame() -> ();
    pub fn Com_EventLoop() -> ();
    pub fn va(fmt: *const c_char, ...) -> *const c_char;

    // Stub structures - just for type checking
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

// Stub structures for external types
#[repr(C)]
pub struct ClientState {
    pub state: connstate_t,
}

#[repr(C)]
pub struct KeyGlobals {
    pub anykeydown: c_int,
}

// BinkVideo method wrappers (C++ methods exposed as C functions)
extern "C" {
    pub fn BinkVideo_Stop(this: *mut BinkVideo) -> ();
    pub fn BinkVideo_GetStatus(this: *const BinkVideo) -> c_int;
    pub fn BinkVideo_Start(this: *mut BinkVideo, path: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int) -> c_int;
    pub fn BinkVideo_SetLooping(this: *mut BinkVideo, loop_: c_int) -> ();
    pub fn BinkVideo_SetMasterVolume(this: *mut BinkVideo, vol: c_int) -> ();
    pub fn BinkVideo_Run(this: *mut BinkVideo) -> ();
    pub fn BinkVideo_Ready(this: *const BinkVideo) -> c_int;
    pub fn BinkVideo_GetBinkWidth(this: *const BinkVideo) -> c_int;
    pub fn BinkVideo_GetBinkHeight(this: *const BinkVideo) -> c_int;
    pub fn BinkVideo_GetBinkData(this: *const BinkVideo) -> *mut byte;
    pub fn BinkVideo_SetExtents(this: *mut BinkVideo, x: c_int, y: c_int, w: c_int, h: c_int) -> ();
    pub fn BinkVideo_AllocateXboxMem(this: *mut BinkVideo) -> ();
    pub fn BinkVideo_FreeXboxMem(this: *mut BinkVideo) -> ();
}

// BinkVideo bVideo;	// bink video object
pub static mut bVideo: BinkVideo = BinkVideo { _opaque: [] };

// connstate_t	previousState = CA_UNINITIALIZED;	// previous cinematic state
pub static mut previousState: connstate_t = CA_UNINITIALIZED;

// We have a fixed lookup table of all cinematics that can be played
// Video handles are just indices into the array. An entry is not
// considered initialized until its width is nonzero
// Porting note: C++ uses string literal initializers for filenames at compile time.
// Rust const initialization cannot easily replicate this, so filenames are zero-initialized.
// The table structure and ordering are preserved faithfully; actual strings would need runtime init.
pub static mut cinFiles: [CinematicData; 15] = [
	// Opening logos
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "logos"

	// Planet shaders
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "cos"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "bakura"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "blenjeel"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "chandrila"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "core"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "ast"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "dosunn"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "krildor"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "narkreeta"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "ordman"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "tanaab"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "tatooine"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "yalara"
	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "zonju"

	// Others
	//	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "jk1"
	//	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "jk2"
	//	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "jk3"
	//	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "jk4"
	//	CinematicData { filename: [0; MAX_OSPATH], x: 0, y: 0, w: 0, h: 0, bits: 0 },	// "jk5"
];

pub const cinNumFiles: usize = 15;  // 15 entries in cinFiles
pub static mut currentHandle: c_int = -1;

/********
CIN_CloseAllVideos
Stops all currently running videos
*********/
pub fn CIN_CloseAllVideos() {
	// Stop the current bink video
	unsafe {
		BinkVideo_Stop(&mut bVideo);
	}
	unsafe {
		currentHandle = -1;
	}
}

/********
CIN_StopCinematic

handle	- Not used
return	- FMV status

Stops the current cinematic
*********/
pub fn CIN_StopCinematic(handle: c_int) -> e_status {
	unsafe {
		debug_assert!(handle == currentHandle);
		currentHandle = -1;

		if previousState != CA_UNINITIALIZED {
			cls.state = previousState;
			previousState = CA_UNINITIALIZED;
		}
		if BinkVideo_GetStatus(&bVideo) != NS_BV_STOPPED {
			BinkVideo_Stop(&mut bVideo);
		}
	}
	FMV_EOF
}

/********
CIN_RunCinematic

handle	- Ensure that the supplied cinematic is the one running
return	- FMV status

Fetch and decompress the pending frame
*********/
pub fn CIN_RunCinematic(handle: c_int) -> e_status {
	unsafe {
		if handle < 0 || handle >= cinNumFiles as c_int || cinFiles[handle as usize].w == 0 {
			debug_assert!(false);
			return FMV_EOF;
		}

		// If we weren't playing a movie, or playing the wrong one - start up
		if handle != currentHandle {
			CIN_StopCinematic(currentHandle);
			let path = va(
				XBOX_VIDEO_FORMAT.as_ptr() as *const c_char,
				cinFiles[handle as usize].filename.as_ptr()
			);
			if BinkVideo_Start(
				&mut bVideo,
				path,
				cinFiles[handle as usize].x,
				cinFiles[handle as usize].y,
				cinFiles[handle as usize].w,
				cinFiles[handle as usize].h
			) == 0 {
				return FMV_EOF;
			}

			if (cinFiles[handle as usize].bits & CIN_loop) != 0 {
				BinkVideo_SetLooping(&mut bVideo, 1);
			}
			else {
				BinkVideo_SetLooping(&mut bVideo, 0);
			}

			if (cinFiles[handle as usize].bits & CIN_silent) != 0 {
				BinkVideo_SetMasterVolume(&mut bVideo, 0);
			}
			else {
				BinkVideo_SetMasterVolume(&mut bVideo, 32768);	// Default Bink volume
			}

			if (cinFiles[handle as usize].bits & CIN_shader) == 0 {
				previousState = cls.state;
				cls.state = CA_CINEMATIC;
			}

			currentHandle = handle;
		}

		// Normal case does nothing here
		if BinkVideo_GetStatus(&bVideo) == NS_BV_STOPPED {
			FMV_EOF
		}
		else {
			FMV_PLAY
		}
	}
}

/********
CIN_PlayCinematic

arg0	- filename of bink video
xpos	- x origin
ypos	- y origin
width	- width of the movie window
height	- height of the movie window
bits	- CIN flags
psAudioFile	- audio file for movie (not used)

Starts playing the given bink video file
*********/
pub fn CIN_PlayCinematic(
	arg0: *const c_char,
	xpos: c_int,
	ypos: c_int,
	width: c_int,
	height: c_int,
	bits: c_int,
) -> c_int {
	//	char	name[MAX_OSPATH];
	let mut arg: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
	let nameonly: *const c_char;
	let mut handle: c_int = 0;

	unsafe {
		// get a local copy of the name
		strcpy(arg.as_mut_ptr(), arg0);

		// remove path, find in list
		nameonly = COM_SkipPath(arg.as_ptr());
		handle = 0;
		while handle < cinNumFiles as c_int {
			if Q_stricmp(cinFiles[handle as usize].filename.as_ptr(), nameonly) == 0 {
				break;
			}
			handle += 1;
		}

		// Don't have the requested movie in our table?
		if handle == cinNumFiles as c_int {
			Com_Printf(b"ERROR: Movie file %s not found!\n\0".as_ptr() as *const c_char, nameonly);
			return -1;
		}

		// Store off information about the movie in the right place. Don't
		// actually play them movie, CIN_RunCinematic takes care of that.
		cinFiles[handle as usize].x = xpos;
		cinFiles[handle as usize].y = ypos;
		cinFiles[handle as usize].w = width;
		cinFiles[handle as usize].h = height;
		cinFiles[handle as usize].bits = bits;
		currentHandle = -1;
	}

	handle
}

/*********
CIN_SetExtents

handle	- handle to a video (not used)
x		- x origin for window
y		- y origin for window
w		- width for window
h		- height for window
*********/
pub fn CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
	unsafe {
		if handle < 0 || handle >= cinNumFiles as c_int {
			return;
		}

		cinFiles[handle as usize].x = x;
		cinFiles[handle as usize].y = y;
		cinFiles[handle as usize].w = w;
		cinFiles[handle as usize].h = h;

		if handle == currentHandle {
			BinkVideo_SetExtents(&mut bVideo, x, y, w, h);
		}
	}
}

/********
CIN_DrawCinematic

handle	- handle to a video (not used)

Updates the current frame of the current video
*********/
pub fn CIN_DrawCinematic(handle: c_int) {
	unsafe {
		debug_assert!(handle == currentHandle);

		BinkVideo_Run(&mut bVideo);
	}
}

/*********
SCR_DrawCinematic
*********/
pub fn SCR_DrawCinematic() {
	unsafe {
		CIN_DrawCinematic(currentHandle);
	}
}

/*********
SCR_RunCinematic
*********/
pub fn SCR_RunCinematic() {
	// This is called every frame, even when we're not playing a movie
	unsafe {
		if currentHandle > 0 && currentHandle < cinNumFiles as c_int {
			CIN_RunCinematic(currentHandle);
		}
	}
}

/*********
SCR_StopCinematic
*********/
pub fn SCR_StopCinematic() {
	unsafe {
		CIN_StopCinematic(currentHandle);
	}
}

/*********
CIN_UploadCinematic

handle		- (not used)

This function can be used to render a frame of a movie, if
it needs to be done outside of CA_CINEMATIC. For example,
a menu background or wall texture.
*********/
pub fn CIN_UploadCinematic(handle: c_int) {
	let mut w: c_int;
	let mut h: c_int;
	let data: *mut byte;

	unsafe {
		debug_assert!(handle == currentHandle);

		if BinkVideo_Ready(&bVideo) == 0 {
			return;
		}

		w		= BinkVideo_GetBinkWidth(&bVideo);
		h		= BinkVideo_GetBinkHeight(&bVideo);
		data	= BinkVideo_GetBinkData(&bVideo);

		// handle is actually being used to pick from scratchImages in
		// this function - we only have two on Xbox, let's just use one.
		//re.UploadCinematic( w, h, data, handle, 1);
		re_UploadCinematic(w, h, data, 0, 1);
	}
}

extern "C" {
	pub fn re_UploadCinematic(w: c_int, h: c_int, data: *mut byte, handle: c_int, stretch: c_int) -> ();
}

/*********
CIN_PlayAllFrames

arg				- bink video filename
x				- x origin for movie
y				- y origin for movie
w				- width of the movie
h				- height of the movie
systemBits		- bit rate for movie
keyBreakAllowed	- if true, button press will end playback

Plays the target movie in full
*********/
pub fn CIN_PlayAllFrames(
	arg: *const c_char,
	x: c_int,
	y: c_int,
	w: c_int,
	h: c_int,
	systemBits: c_int,
	keyBreakAllowed: bool,
) -> bool {
	let retval: bool;
	let mut Handle: c_int;

	unsafe {
		Key_ClearStates();

		Handle = CIN_PlayCinematic(arg, x, y, w, h, systemBits);
		if Handle != -1 {
			// Wait for video to finish or key to be pressed
			loop {
				if CIN_RunCinematic(Handle) != FMV_PLAY {
					break;
				}
				if keyBreakAllowed && kg.anykeydown != 0 {
					break;
				}

				SCR_UpdateScreen();
				IN_Frame();
				Com_EventLoop();
			}

			// XBOX: wait for key to be released
			loop {
				if CIN_RunCinematic(Handle) != FMV_PLAY {
					break;
				}
				if keyBreakAllowed && kg.anykeydown == 0 {
					break;
				}

				SCR_UpdateScreen();
				IN_Frame();
				Com_EventLoop();
			}

			CIN_StopCinematic(Handle);
		}

		retval = (keyBreakAllowed && kg.anykeydown != 0);
		Key_ClearStates();
	}

	retval
}

/*********
CIN_Init
Initializes cinematic system
*********/
pub fn CIN_Init() {
	// Allocate Memory for Bink System
	unsafe {
		BinkVideo_AllocateXboxMem(&mut bVideo);
	}
}

/********
CIN_Shutdown
Shutdown the cinematic system
********/
pub fn CIN_Shutdown() {
	// Free Memory for the Bink System
	unsafe {
		BinkVideo_FreeXboxMem(&mut bVideo);
	}
}


/***** Possible FIXME *****/
/***** The following function may need to be implemented *****/
/***** BEGIN *****/
pub fn CL_PlayCinematic_f() {
	let arg: *const c_char;

	unsafe {
		arg = Cmd_Argv(1);
		CIN_PlayAllFrames(arg, 48, 36, 544, 408, 0, true);
	}
}

pub fn CL_IsRunningInGameCinematic() -> qboolean {
	// Nothing
	qfalse
}

pub fn CL_CheckPendingCinematic() -> qboolean {
	// Nothing
	qfalse
}

pub fn CL_PlayInGameCinematic_f() {
	// Nothing
}

pub fn CL_InGameCinematicOnStandBy() -> qboolean {
	// Nothing
	qfalse
}
/***** END *****/
