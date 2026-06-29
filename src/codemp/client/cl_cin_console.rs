#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

/*****************************************************************************
 * name:		cl_cin_stubs.cpp
 *
 * desc:		video and cinematic playback stubs to avoid link errors
 *
 *
 * cl_glconfig.hwtype trtypes 3dfx/ragepro need 256x256
 *
 *****************************************************************************/

// ============================================================================
// Type declarations - stubs for external types
// ============================================================================

pub type connstate_t = c_int;
pub type e_status = c_int;
pub type qboolean = c_int;
pub type byte = u8;

// FMV status values
pub const FMV_EOF: e_status = 1;
pub const FMV_PLAY: e_status = 2;

// BinkVideo status values
pub const NS_BV_STOPPED: c_int = 0;

// CIN flags
pub const CIN_loop: c_int = 1;
pub const CIN_silent: c_int = 2;
pub const CIN_shader: c_int = 4;

// Connection states
pub const CA_UNINITIALIZED: connstate_t = 0;
pub const CA_CINEMATIC: connstate_t = 2;

// ============================================================================
// External struct declarations
// ============================================================================

// Opaque BinkVideo type - actual definition is in BinkVideo.rs
#[repr(C)]
pub struct BinkVideo {
    _opaque: [u8; 0],
}

// Stub connection state structure
#[repr(C)]
pub struct ClientState {
    pub state: connstate_t,
    // ... other fields not defined in this stub file
}

// Stub keyboard state structure
#[repr(C)]
pub struct KeyState {
    pub anykeydown: qboolean,
    // ... other fields not defined in this stub file
}

// Stub renderer state structure with UploadCinematic function pointer
#[repr(C)]
pub struct RendererState {
    pub UploadCinematic: extern "C" fn(c_int, c_int, *const byte, c_int, c_int),
    // ... other fields not defined in this stub file
}

// ============================================================================
// Global variables
// ============================================================================

pub static mut bVideo: BinkVideo = BinkVideo { _opaque: [] };	// bink video object
pub static mut previousState: connstate_t = CA_UNINITIALIZED;	// previous cinematic state

// Stub global state variables
pub static mut cls: ClientState = ClientState { state: 0 };
pub static mut kg: KeyState = KeyState { anykeydown: 0 };
pub static mut re: RendererState = RendererState {
    UploadCinematic: _dummy_upload_cinematic,
};

// ============================================================================
// External function declarations
// ============================================================================

extern "C" {
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    pub fn COM_SkipPath(path: *const c_char) -> *mut c_char;
    pub fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    pub fn COM_DefaultExtension(dest: *mut c_char, size: usize, ext: *const c_char);

    pub fn Key_ClearStates();
    pub fn SCR_UpdateScreen();
    pub fn IN_Frame();
    pub fn Com_EventLoop();

    // BinkVideo methods as C functions (mangled from C++ class methods)
    pub fn BinkVideo_Stop(bv: *mut BinkVideo);
    pub fn BinkVideo_GetStatus(bv: *const BinkVideo) -> c_int;
    pub fn BinkVideo_SetLooping(bv: *mut BinkVideo, loop_: bool);
    pub fn BinkVideo_SetMasterVolume(bv: *mut BinkVideo, vol: c_int);
    pub fn BinkVideo_Start(bv: *mut BinkVideo, name: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int) -> bool;
    pub fn BinkVideo_SetExtents(bv: *mut BinkVideo, x: c_int, y: c_int, w: c_int, h: c_int);
    pub fn BinkVideo_Run(bv: *mut BinkVideo);
    pub fn BinkVideo_Ready(bv: *const BinkVideo) -> bool;
    pub fn BinkVideo_GetBinkWidth(bv: *const BinkVideo) -> c_int;
    pub fn BinkVideo_GetBinkHeight(bv: *const BinkVideo) -> c_int;
    pub fn BinkVideo_GetBinkData(bv: *const BinkVideo) -> *const byte;
    pub fn BinkVideo_AllocateXboxMem(bv: *mut BinkVideo);
    pub fn BinkVideo_FreeXboxMem(bv: *mut BinkVideo);
}

// Dummy implementation for UploadCinematic function pointer
extern "C" fn _dummy_upload_cinematic(_w: c_int, _h: c_int, _data: *const byte, _handle: c_int, _x: c_int) {
}

// ============================================================================
// Constants
// ============================================================================

const XBOX_VIDEO_PATH: &[u8] = b"d:\\base\\video\\";
const MAX_OSPATH: usize = 256;

// ============================================================================
// Function definitions
// ============================================================================

/********
CIN_CloseAllVideos
Stops all currently running videos
*********/
pub fn CIN_CloseAllVideos() {
	unsafe {
		// Stop the current bink video
		BinkVideo_Stop(core::ptr::addr_of_mut!(bVideo));
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
		if previousState != CA_UNINITIALIZED {
			(*core::ptr::addr_of_mut!(cls)).state = previousState;
			previousState = CA_UNINITIALIZED;
		}
		if BinkVideo_GetStatus(core::ptr::addr_of_mut!(bVideo)) != NS_BV_STOPPED {
			BinkVideo_Stop(core::ptr::addr_of_mut!(bVideo));
		}
	}
	FMV_EOF
}

/********
CIN_StopCinematic

handle	- Not used
return	- FMV status

Checks the status of the current cinematic
*********/
pub fn CIN_RunCinematic(handle: c_int) -> e_status {
	unsafe {
		if BinkVideo_GetStatus(core::ptr::addr_of_mut!(bVideo)) == NS_BV_STOPPED {
			return FMV_EOF;
		} else {
			return FMV_PLAY;
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
pub fn CIN_PlayCinematic(arg0: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int, bits: c_int) -> c_int {
	let mut name: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
	let mut arg: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

	unsafe {
		// get a local copy of the name
		strcpy(arg.as_mut_ptr(), arg0);

		// remove path
		let nameonly = COM_SkipPath(arg.as_ptr());

		// form the proper name with path
		Com_sprintf(name.as_mut_ptr(), core::mem::size_of_val(&name), XBOX_VIDEO_PATH.as_ptr() as *const c_char);
		strcat(name.as_mut_ptr(), nameonly);
		COM_DefaultExtension(name.as_mut_ptr(), core::mem::size_of_val(&name), b".bik\0".as_ptr() as *const c_char);

		if BinkVideo_Start(core::ptr::addr_of_mut!(bVideo), name.as_ptr(), xpos, ypos, width, height) {
			if (bits & CIN_loop) != 0 {
				BinkVideo_SetLooping(core::ptr::addr_of_mut!(bVideo), true);
			}
			if (bits & CIN_silent) != 0 {
				BinkVideo_SetMasterVolume(core::ptr::addr_of_mut!(bVideo), 0);
			}
			if (bits & CIN_shader) == 0 {
				previousState = (*core::ptr::addr_of_mut!(cls)).state;
				(*core::ptr::addr_of_mut!(cls)).state = CA_CINEMATIC;
			}
			return 1;
		} else {
			return -1;
		}
	}
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
		BinkVideo_SetExtents(core::ptr::addr_of_mut!(bVideo), x, y, w, h);
	}
}

/********
CIN_DrawCinematic

handle	- handle to a video (not used)

Updates the current frame of the current video
*********/
pub fn CIN_DrawCinematic(handle: c_int) {
	unsafe {
		BinkVideo_Run(core::ptr::addr_of_mut!(bVideo));
	}
}

/*********
SCR_DrawCinematic
*********/
pub fn SCR_DrawCinematic() {
	CIN_DrawCinematic(1);
}

/*********
SCR_RunCinematic
*********/
pub fn SCR_RunCinematic() {
	CIN_RunCinematic(1);
}

/*********
SCR_StopCinematic
*********/
pub fn SCR_StopCinematic() {
	CIN_StopCinematic(1);
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
	let data: *const byte;

	unsafe {
		if !BinkVideo_Ready(core::ptr::addr_of_mut!(bVideo)) {
			return;
		}

		w		= BinkVideo_GetBinkWidth(core::ptr::addr_of_mut!(bVideo));
		h		= BinkVideo_GetBinkHeight(core::ptr::addr_of_mut!(bVideo));
		data	= BinkVideo_GetBinkData(core::ptr::addr_of_mut!(bVideo));

		(re.UploadCinematic)( w, h, data, handle, 1);
	}
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
pub fn CIN_PlayAllFrames(arg: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, systemBits: c_int, keyBreakAllowed: bool) -> bool {
	//JLF new
	let retval: bool;
	//endJLF
	unsafe {
		Key_ClearStates();

		let Handle = CIN_PlayCinematic(arg, x, y, w, h, systemBits);
		if Handle != -1 {
			while CIN_RunCinematic(Handle) == FMV_PLAY && !(keyBreakAllowed && (*core::ptr::addr_of_mut!(kg)).anykeydown != 0) {
				SCR_UpdateScreen();
				IN_Frame();
				Com_EventLoop();
			}
			CIN_StopCinematic(Handle);
		}
		retval = keyBreakAllowed && (*core::ptr::addr_of_mut!(kg)).anykeydown != 0;
		Key_ClearStates();
	}
	retval
}

/*********
CIN_DisplayIntros
Draws intro movies to the screen
*********/
pub fn CIN_DisplayIntros() {

	////////////////////////////////////
	// Play 1st video: Activision
	////////////////////////////////////
	CIN_PlayAllFrames( b"atvi.bik\0".as_ptr() as *const c_char, 0, 0, 640, 480, 0, true );


	////////////////////////////////////
	// Play 2nd video: Vicarious Visions
	////////////////////////////////////
	CIN_PlayAllFrames( b"vvintro.bik\0".as_ptr() as *const c_char, 0, 0, 640, 480, 0, true );
}

/*********
CIN_Init
Initializes cinematic system
*********/
pub fn CIN_Init() {
	unsafe {
		// Allocate Memory for Bink System
		BinkVideo_AllocateXboxMem(core::ptr::addr_of_mut!(bVideo));
	}
}

/********
CIN_Shutdown
Shutdown the cinematic system
********/
pub fn CIN_Shutdown() {
	unsafe {
		// Free Memory for the Bink System
		BinkVideo_FreeXboxMem(core::ptr::addr_of_mut!(bVideo));
	}
}


/***** Possible FIXME *****/
/***** The following function may need to be implemented *****/
/***** BEGIN *****/
pub fn CL_PlayCinematic_f() {
	// Nothing
}

pub fn CL_IsRunningInGameCinematic() -> qboolean {
	// Nothing
	return 0 as qboolean;
}

pub fn CL_CheckPendingCinematic() -> qboolean {
	// Nothing
	return 0 as qboolean;
}

pub fn CL_PlayInGameCinematic_f() {
	// Nothing
}

pub fn CL_InGameCinematicOnStandBy() -> qboolean {
	// Nothing
	return 0 as qboolean;
}
/***** END *****/
