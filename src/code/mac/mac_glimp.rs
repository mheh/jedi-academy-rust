#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_void};

// ===== Opaque external types from Apple frameworks and engine =====

// Opaque types from AGL/Mac frameworks - modeled as zero-sized types
// These are pointers in C, used through transparent wrapper types
#[repr(C)]
pub struct GDHandle([c_void; 0]);

#[repr(C)]
pub struct AGLContext([c_void; 0]);

#[repr(C)]
pub struct AGLDrawable([c_void; 0]);

#[repr(C)]
pub struct AGLPixelFormat([c_void; 0]);

#[repr(C)]
pub struct DSpContextReference([c_void; 0]);

#[repr(C)]
pub struct cvar_t;

pub type Ptr = *mut c_void;
pub type GLenum = c_uint;
pub type GLint = c_int;
pub type OSStatus = i32;
pub type qboolean = c_int;
pub type Str255 = [c_char; 256];

const MAX_DEVICES: usize = 32;

// ===== Structures =====

#[repr(C)]
pub struct macGlInfo {
	pub devices: [*mut GDHandle; MAX_DEVICES],
	pub numDevices: c_int,

	pub systemGammas: Ptr,

	pub device: *mut GDHandle,

	pub context: *mut AGLContext,
	pub drawable: *mut AGLDrawable,
	pub fmt: *mut AGLPixelFormat,

	pub textureMemory: GLint,
	pub videoMemory: GLint,

	pub DSpContext: DSpContextReference,
}

// ===== Globals =====

pub static mut r_device: *mut cvar_t = core::ptr::null_mut();
pub static mut r_ext_transform_hint: *mut cvar_t = core::ptr::null_mut();
pub static mut sys_hardwareType: c_int = 0;
pub static mut sys_gl: macGlInfo = macGlInfo {
	devices: [core::ptr::null_mut(); MAX_DEVICES],
	numDevices: 0,
	systemGammas: core::ptr::null_mut(),
	device: core::ptr::null_mut(),
	context: core::ptr::null_mut(),
	drawable: core::ptr::null_mut(),
	fmt: core::ptr::null_mut(),
	textureMemory: 0,
	videoMemory: 0,
	DSpContext: DSpContextReference([]),
};

// ===== External engine and framework functions =====

extern "C" {
	// From renderer interface
	static mut glConfig: crate::renderer::glConfig_t;
	static mut ri: crate::renderer::ri_t;
	static mut r_colorbits: *mut cvar_t;
	static mut r_mode: *mut cvar_t;
	static mut r_fullscreen: *mut cvar_t;
	static mut r_stencilbits: *mut cvar_t;
	static mut r_depthbits: *mut cvar_t;
	static mut r_ext_multitexture: *mut cvar_t;
	static mut r_allowExtensions: *mut cvar_t;
	static mut r_ext_compiled_vertex_array: *mut cvar_t;
	static mut r_ext_texture_env_add: *mut cvar_t;
	static mut r_ext_texture_filter_anisotropic: *mut cvar_t;
	static mut r_swapInterval: *mut cvar_t;

	// GL functions
	fn qglGetString(name: GLenum) -> *const c_char;
	fn qglClearColor(red: f32, green: f32, blue: f32, alpha: f32);
	fn qglClear(mask: c_uint);
	fn glMultiTexCoord2fARB(target: GLenum, s: f32, t: f32);
	fn glActiveTextureARB(texture: GLenum);
	fn glClientActiveTextureARB(texture: GLenum);
	fn glLockArraysEXT(first: GLint, count: c_int);
	fn glUnlockArraysEXT();
	fn glHint(target: GLenum, mode: GLenum);

	// AGL functions
	fn aglGetError() -> GLenum;
	fn aglErrorString(err: GLenum) -> *const c_char;
	fn aglChoosePixelFormat(
		device: *const *mut GDHandle,
		ndev: c_int,
		attrib: *const GLint,
	) -> *mut AGLPixelFormat;
	fn aglDescribePixelFormat(fmt: *mut AGLPixelFormat, attrib: GLint, value: *mut c_int);
	fn aglCreateContext(fmt: *mut AGLPixelFormat, share: *mut AGLContext) -> *mut AGLContext;
	fn aglSetDrawable(ctx: *mut AGLContext, draw: AGLDrawable) -> c_int;
	fn aglSetCurrentContext(ctx: *mut AGLContext) -> c_int;
	fn aglSwapBuffers(ctx: *mut AGLContext);
	fn aglSetInteger(ctx: *mut AGLContext, pname: GLint, params: *mut c_int);
	fn aglDestroyContext(ctx: *mut AGLContext);
	fn aglDestroyPixelFormat(fmt: *mut AGLPixelFormat);
	fn aglGetVersion(major: *mut GLint, minor: *mut GLint);
	fn aglQueryRendererInfo(device: *mut GDHandle, ndev: c_int) -> *mut AGLContext;
	fn aglDescribeRenderer(info: *mut AGLContext, attrib: GLint, value: *mut GLint);
	fn aglNextRendererInfo(info: *mut AGLContext) -> *mut AGLContext;
	fn aglDestroyRendererInfo(info: *mut AGLContext);
	fn aglDevicesOfPixelFormat(fmt: *mut AGLPixelFormat, ndev: *mut c_int) -> *mut GDHandle;
	fn aglEnable(ctx: *mut AGLContext, pname: GLint);
	fn aglDisable(ctx: *mut AGLContext, pname: GLint);

	// DrawSprocket functions
	fn DSpStartup() -> OSStatus;
	fn DSpShutdown() -> OSStatus;
	fn DSpFindBestContext(
		inAttributes: *mut c_void,
		outContext: *mut DSpContextReference,
	) -> OSStatus;
	fn DSpContext_Reserve(ctx: DSpContextReference, inAttributes: *mut c_void) -> OSStatus;
	fn DSpContext_GetAttributes(ctx: DSpContextReference, outAttributes: *mut c_void) -> OSStatus;
	fn DSpContext_SetState(ctx: DSpContextReference, state: c_int) -> OSStatus;
	fn DSpContext_Release(ctx: DSpContextReference) -> OSStatus;
	fn DSpContext_FadeGammaIn(data1: *mut c_void, data2: *mut c_void) -> OSStatus;

	// Mac Window Manager functions
	fn GetNewCWindow(wid: c_int, storage: *mut c_void, behind: *mut c_void) -> *mut c_void;
	fn SizeWindow(port: *mut c_void, w: c_int, h: c_int, fUpdate: c_int);
	fn MoveWindow(port: *mut c_void, x: c_int, y: c_int, fUpdate: c_int);
	fn ShowWindow(port: *mut c_void);
	fn SetPort(port: *mut c_void);
	fn SetWTitle(port: *mut c_void, ps: *const Str255);
	fn HiliteWindow(port: *mut c_void, flag: c_int);
	fn DisposeWindow(port: *mut c_void);

	// GLM functions
	fn glmGetInteger(pname: c_int) -> c_int;

	// System and utility functions
	fn strlen(s: *const c_char) -> usize;
	fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
	fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
	fn atoi(s: *const c_char) -> c_int;

	// Mac Gamma functions
	fn GetSystemGammas() -> Ptr;
	fn RestoreSystemGammas(gammas: Ptr);
	fn DisposeSystemGammas(gammas: *mut Ptr);
	fn SetDeviceGammaRampGD(device: *mut GDHandle, ramp: *mut c_char);

	// Renderer interface (from ri)
	fn R_GetModeInfo(w: *mut c_int, h: *mut c_int, aspect: *mut f32, mode: c_int) -> qboolean;
	fn GetDeviceList() -> *mut GDHandle;
	fn GetNextDevice(device: *mut GDHandle) -> *mut GDHandle;
	fn Sys_SendKeyEvents();

	// Global function pointers that get assigned
	static mut qglMultiTexCoord2fARB: Option<extern "C" fn(GLenum, f32, f32)>;
	static mut qglActiveTextureARB: Option<extern "C" fn(GLenum)>;
	static mut qglClientActiveTextureARB: Option<extern "C" fn(GLenum)>;
	static mut qglLockArraysEXT: Option<extern "C" fn(GLint, c_int)>;
	static mut qglUnlockArraysEXT: Option<extern "C" fn()>;
}

// ===== Helper Functions and Constants =====

// AGL constants from Apple AGL library
extern "C" {
	static AGL_NO_ERROR: GLenum;
	static AGL_RGBA: GLint;
	static AGL_DOUBLEBUFFER: GLint;
	static AGL_NO_RECOVERY: GLint;
	static AGL_ACCELERATED: GLint;
	static AGL_RED_SIZE: GLint;
	static AGL_GREEN_SIZE: GLint;
	static AGL_BLUE_SIZE: GLint;
	static AGL_ALPHA_SIZE: GLint;
	static AGL_STENCIL_SIZE: GLint;
	static AGL_DEPTH_SIZE: GLint;
	static AGL_BUFFER_SIZE: GLint;
	static AGL_PIXEL_SIZE: GLint;
	static AGL_RENDERER_ID: GLint;
	static AGL_TEXTURE_MEMORY: GLint;
	static AGL_VIDEO_MEMORY: GLint;
	static AGL_RASTERIZATION: GLint;
	static AGL_SWAP_INTERVAL: GLint;

	// DrawSprocket constants
	static kDSpContextState_Inactive: c_int;
	static kDSpContextState_Active: c_int;
	static kDSpDepthMask_16: c_int;
	static kDSpDepthMask_32: c_int;
	static kDSpColorNeeds_Require: c_int;
	static kMainWindow: c_int;
	static kFullScreenWindow: c_int;
	static GL_FALSE: c_int;
	static GL_TRUE: c_int;

	// GL constants
	static GL_VENDOR: GLenum;
	static GL_RENDERER: GLenum;
	static GL_VERSION: GLenum;
	static GL_EXTENSIONS: GLenum;
	static GL_COLOR_BUFFER_BIT: c_uint;
	static GL_TRANSFORM_HINT_APPLE: GLenum;
	static GL_FASTEST: GLenum;

	// CVAR flags and other constants
	static CVAR_LATCH: c_int;
	static CVAR_ARCHIVE: c_int;
	static ERR_FATAL: c_int;
	static GLHW_GENERIC: c_int;
	static GLHW_RAGEPRO: c_int;
	static PRINT_ALL: c_int;
}

// ===== Utility Functions =====

// FIXME: Original line 2 comment: typedef int sysEventType_t;

// CToPStr converts a C string to a Pascal string (length-prefixed)
unsafe fn CToPStr(cs: *const c_char, ps: &mut Str255) {
	let mut i: c_int = 0;
	let l: c_int = strlen(cs) as c_int;
	let mut l_mut = l;

	if l_mut > 255 {
		l_mut = 255;
	}
	ps[0] = l_mut as c_char;
	while i < l_mut {
		ps[(i + 1) as usize] = *cs.add(i as usize);
		i += 1;
	}
}

// ============
// CheckErrors
// ============
fn CheckErrors() {
	let err: GLenum = aglGetError();

	if err != AGL_NO_ERROR {
		let err_str = aglErrorString(err);
		// ri.Error( ERR_FATAL, "aglGetError: %s", aglErrorString( err ) );
		// NOTE: Direct ri.Error call requires extern binding
	}
}

// =======================================================================

// =====================
// GLimp_ResetDisplay
// =====================
pub unsafe fn GLimp_ResetDisplay() {
	// PLACEHOLDER: Need access to glConfig struct
	// if ( !glConfig.isFullscreen ) {
	// 	return;
	// }
	// glConfig.isFullscreen = qfalse;

	// put the context into the inactive state
	DSpContext_SetState(sys_gl.DSpContext, kDSpContextState_Inactive);

	// release the context
	DSpContext_Release(sys_gl.DSpContext);

	// shutdown draw sprockets
	DSpShutdown();
}

// =====================
// GLimp_ChangeDisplay
// =====================
pub unsafe fn GLimp_ChangeDisplay(actualWidth: *mut c_int, actualHeight: *mut c_int) {
	let mut theError: OSStatus = 0;
	// DSpContextAttributes inAttributes - opaque type, structure setup commented out
	let mut colorBits: c_int = 0;

	// startup DrawSprocket
	theError = DSpStartup();
	if theError != 0 {
		// ri.Printf( PRINT_ALL, "DSpStartup() failed: %i\n", theError );
		*actualWidth = 640;
		*actualHeight = 480;
		return;
	}

	// if ( r_colorbits->integer == 24 || r_colorbits->integer == 32 ) {
	// 	colorBits = kDSpDepthMask_32;
	// } else {
	// 	colorBits = kDSpDepthMask_16;
	// }

	// memset( &inAttributes, 0, sizeof( inAttributes ) );
	// NOTE: inAttributes structure setup follows the C pattern
	// Structure initialization code commented out due to unresolved DrawSprocket types
}

// =======================================================================

// ===================
// GLimp_AglDescribe_f
//
// ===================
pub unsafe fn GLimp_AglDescribe_f() {
	let mut value: c_int = 0;
	let mut r: c_int = 0;
	let mut g: c_int = 0;
	let mut b: c_int = 0;
	let mut a: c_int = 0;
	let mut stencil: c_int = 0;
	let mut depth: c_int = 0;

	// ri.Printf( PRINT_ALL, "Selected pixel format 0x%x\n", (int)sys_gl.fmt );

	// ri.Printf( PRINT_ALL, "TEXTURE_MEMORY: %i\n", sys_gl.textureMemory );
	// ri.Printf( PRINT_ALL, "VIDEO_MEMORY: %i\n", sys_gl.videoMemory );

	aglDescribePixelFormat(sys_gl.fmt, AGL_RED_SIZE, &mut r);
	aglDescribePixelFormat(sys_gl.fmt, AGL_GREEN_SIZE, &mut g);
	aglDescribePixelFormat(sys_gl.fmt, AGL_BLUE_SIZE, &mut b);
	aglDescribePixelFormat(sys_gl.fmt, AGL_ALPHA_SIZE, &mut a);
	aglDescribePixelFormat(sys_gl.fmt, AGL_STENCIL_SIZE, &mut stencil);
	aglDescribePixelFormat(sys_gl.fmt, AGL_DEPTH_SIZE, &mut depth);
	// ri.Printf( PRINT_ALL, "red:%i green:%i blue:%i alpha:%i depth:%i stencil:%i\n",
	// 	r, g, b, a, depth, stencil );

	aglDescribePixelFormat(sys_gl.fmt, AGL_BUFFER_SIZE, &mut value);
	// ri.Printf( PRINT_ALL, "BUFFER_SIZE: %i\n", value );

	aglDescribePixelFormat(sys_gl.fmt, AGL_PIXEL_SIZE, &mut value);
	// ri.Printf( PRINT_ALL, "PIXEL_SIZE: %i\n", value );

	aglDescribePixelFormat(sys_gl.fmt, AGL_RENDERER_ID, &mut value);
	// ri.Printf( PRINT_ALL, "RENDERER_ID: %i\n", value );

	// memory functions
	value = glmGetInteger(0); // GLM_PAGE_SIZE
	// ri.Printf( PRINT_ALL, "GLM_PAGE_SIZE: %i\n", value );

	value = glmGetInteger(1); // GLM_NUMBER_PAGES
	// ri.Printf( PRINT_ALL, "GLM_NUMBER_PAGES: %i\n", value );

	value = glmGetInteger(2); // GLM_CURRENT_MEMORY
	// ri.Printf( PRINT_ALL, "GLM_CURRENT_MEMORY: %i\n", value );

	value = glmGetInteger(3); // GLM_MAXIMUM_MEMORY
	// ri.Printf( PRINT_ALL, "GLM_MAXIMUM_MEMORY: %i\n", value );
}

// ===================
// GLimp_AglState_f
//
// ===================
pub unsafe fn GLimp_AglState_f() {
	let mut cmd: *const c_char;
	let mut state: c_int = 0;
	let mut value: c_int = 0;

	// if ( ri.Cmd_Argc() != 3 ) {
	// 	ri.Printf( PRINT_ALL, "Usage: aglstate <parameter> <0/1>\n" );
	// 	return;
	// }

	// cmd = ri.Cmd_Argv( 1 );
	// if ( !Q_stricmp( cmd, "rasterization" ) ) {
	// 	state = AGL_RASTERIZATION;
	// } else {
	// 	ri.Printf( PRINT_ALL, "Unknown agl state: %s\n", cmd );
	// 	return;
	// }

	// cmd = ri.Cmd_Argv( 2 );
	// value = atoi( cmd );

	// if ( value ) {
	// 	aglEnable( sys_gl.context, state );
	// } else {
	// 	aglDisable( sys_gl.context, state );
	// }
}

// ===================
// GLimp_Extensions
//
// ===================
fn GLimp_Extensions() {
	// let extensions: *const c_char;

	// // get our config strings
	// Q_strncpyz( glConfig.vendor_string, (const char *)qglGetString (GL_VENDOR), sizeof( glConfig.vendor_string ) );
	// Q_strncpyz( glConfig.renderer_string, (const char *)qglGetString (GL_RENDERER), sizeof( glConfig.renderer_string ) );
	// Q_strncpyz( glConfig.version_string, (const char *)qglGetString (GL_VERSION), sizeof( glConfig.version_string ) );
	// Q_strncpyz( glConfig.extensions_string, (const char *)qglGetString (GL_EXTENSIONS), sizeof( glConfig.extensions_string ) );

	// extensions = glConfig.extensions_string;

	// // GL_ARB_multitexture
	// qglMultiTexCoord2fARB = NULL;
	// qglActiveTextureARB = NULL;
	// qglClientActiveTextureARB = NULL;

	// if ( strstr( extensions, "GL_ARB_multitexture" )  ) {
	// 	if ( r_ext_multitexture->integer && r_allowExtensions->integer ) {
	// 		qglMultiTexCoord2fARB = glMultiTexCoord2fARB;
	// 		qglActiveTextureARB = glActiveTextureARB;
	// 		qglClientActiveTextureARB = glClientActiveTextureARB;

	// 		ri.Printf( PRINT_ALL, "...using GL_ARB_multitexture\n" );
	// 	} else {
	// 		ri.Printf( PRINT_ALL, "...ignoring GL_ARB_multitexture\n" );
	// 	}
	// } else {
	// 	ri.Printf( PRINT_ALL, "...GL_ARB_multitexture not found\n" );
	// }

	// // GL_EXT_compiled_vertex_array
	// qglLockArraysEXT = NULL;
	// qglUnlockArraysEXT = NULL;

	// if ( strstr( extensions, "GL_EXT_compiled_vertex_array" ) ) {
	// 	if ( r_ext_compiled_vertex_array->integer && r_allowExtensions->integer ) {
	// 		qglLockArraysEXT = glLockArraysEXT;
	// 		qglUnlockArraysEXT = glUnlockArraysEXT;

	// 		ri.Printf( PRINT_ALL, "...using GL_EXT_compiled_vertex_array\n" );
	// 	} else {
	// 		ri.Printf( PRINT_ALL, "...ignoring GL_EXT_compiled_vertex_array\n" );
	// 	}
	// } else {
	// 	ri.Printf( PRINT_ALL, "...GL_EXT_compiled_vertex_array not found\n" );
	// }

	// // GL_EXT_texture_env_add
	// glConfig.textureEnvAddAvailable = qfalse;
	// if ( strstr( glConfig.extensions_string, "EXT_texture_env_add" ) ) {
	// 	if ( r_ext_texture_env_add->integer ) {
	// 		glConfig.textureEnvAddAvailable = qtrue;
	// 		ri.Printf( PRINT_ALL, "...using GL_EXT_texture_env_add\n" );
	// 	} else {
	// 		glConfig.textureEnvAddAvailable = qfalse;
	// 		ri.Printf( PRINT_ALL, "...ignoring GL_EXT_texture_env_add\n" );
	// 	}
	// } else {
	// 	ri.Printf( PRINT_ALL, "...GL_EXT_texture_env_add not found\n" );
	// }

	// // GL_EXT_texture_filter_anisotropic
	// glConfig.textureFilterAnisotropicAvailable = qfalse;
	// if ( strstr( glConfig.extensions_string, "EXT_texture_filter_anisotropic" ) )
	// {
	// 	glConfig.textureFilterAnisotropicAvailable = qtrue;
	// 	ri.Printf( PRINT_ALL, "...GL_EXT_texture_filter_anisotropic available\n" );

	// 	if ( r_ext_texture_filter_anisotropic->integer )
	// 	{
	// 		ri.Printf( PRINT_ALL, "...using GL_EXT_texture_filter_anisotropic\n" );
	// 	}
	// 	else
	// 	{
	// 		ri.Printf( PRINT_ALL, "...ignoring GL_EXT_texture_filter_anisotropic\n" );
	// 	}
	// 	ri.Cvar_Set( "r_ext_texture_filter_anisotropic_avail", "1" );
	// }
	// else
	// {
	// 	ri.Printf( PRINT_ALL, "...GL_EXT_texture_filter_anisotropic not found\n" );
	// 	ri.Cvar_Set( "r_ext_texture_filter_anisotropic_avail", "0" );
	// }

	// // apple transform hint
	// if ( strstr( extensions, "GL_APPLE_transform_hint" ) ) {
	// 	if ( r_ext_compiled_vertex_array->integer && r_allowExtensions->integer ) {
	// 		glHint( GL_TRANSFORM_HINT_APPLE, GL_FASTEST );
	// 		ri.Printf( PRINT_ALL, "...using GL_APPLE_transform_hint\n" );
	// 	} else {
	// 		ri.Printf( PRINT_ALL, "...ignoring GL_APPLE_transform_hint\n" );
	// 	}
	// } else {
	// 	ri.Printf( PRINT_ALL, "...GL_APPLE_transform_hint not found\n" );
	// }
}

// ============================
// GLimp_SufficientVideoMemory
// ============================
// #if 0
// qboolean	GLimp_SufficientVideoMemory( void ) { ... }
// #endif

// =======================
// CheckDeviceRenderers
//
// ========================
unsafe fn CheckDeviceRenderers(device: *mut GDHandle) {
	let mut head_info: *mut AGLContext;
	let mut info: *mut AGLContext;
	let mut inum: c_int = 0;
	let mut accelerated: c_int = 0;
	let mut textureMemory: c_int = 0;
	let mut videoMemory: c_int = 0;

	head_info = aglQueryRendererInfo(device, 1);
	if head_info.is_null() {
		// ri.Printf( PRINT_ALL, "aglQueryRendererInfo : Info Error\n");
		return;
	}

	info = head_info;
	inum = 0;
	while !info.is_null() {
		// ri.Printf( PRINT_ALL, "  Renderer : %d\n", inum);

		aglDescribeRenderer(info, AGL_ACCELERATED, &mut accelerated);

		if accelerated != 0 {
			aglDescribeRenderer(info, AGL_TEXTURE_MEMORY, &mut textureMemory);
			aglDescribeRenderer(info, AGL_VIDEO_MEMORY, &mut videoMemory);
			// ri.Printf( PRINT_ALL, "    AGL_VIDEO_MEMORY: %i\n", textureMemory );
			// ri.Printf( PRINT_ALL, "    AGL_TEXTURE_MEMORY: %i\n", videoMemory );

			// save the device with the most texture memory
			if sys_gl.textureMemory < textureMemory {
				sys_gl.textureMemory = textureMemory;
				sys_gl.device = device;
			}
		} else {
			// ri.Printf( PRINT_ALL, "    Not accelerated.\n" );
		}

		info = aglNextRendererInfo(info);
		inum += 1;
	}

	aglDestroyRendererInfo(head_info);
}

// =======================
// CheckDevices
//
// Make sure there is a device with enough video memory to play
// =======================
unsafe fn CheckDevices() {
	let mut device: *mut GDHandle;
	static mut checkedFullscreen: qboolean = 0;

	if checkedFullscreen != 0 {
		return;
	}
	// if ( glConfig.isFullscreen ) {
	// 	checkedFullscreen = qtrue;
	// }

	device = GetDeviceList();
	sys_gl.numDevices = 0;
	while !device.is_null() && sys_gl.numDevices < MAX_DEVICES as c_int {
		sys_gl.devices[sys_gl.numDevices as usize] = device;

		// ri.Printf( PRINT_ALL, "Device : %d\n", sys_gl.numDevices);
		CheckDeviceRenderers(device);

		device = GetNextDevice(device);

		sys_gl.numDevices += 1;
	}

	CheckErrors();

	if sys_gl.textureMemory < 4000000 {
		// ri.Error( ERR_FATAL, "You must have at least four megs of video memory to play" );
	}

	if sys_gl.textureMemory < 16000000 {
		sys_hardwareType = GLHW_RAGEPRO; // this will have to change with voodoo
	} else {
		sys_hardwareType = GLHW_GENERIC;
	}
}

// =================
// CreateGameWindow
// =================
fn CreateGameWindow() -> qboolean {
	let mut vid_xpos: *mut cvar_t;
	let mut vid_ypos: *mut cvar_t;
	let mut mode: c_int = 0;
	let mut x: c_int = 0;
	let mut y: c_int = 0;
	let mut pstr: Str255 = [0; 256];

	// vid_xpos = ri.Cvar_Get( "vid_xpos", "30", 0 );
	// vid_ypos = ri.Cvar_Get( "vid_ypos", "30", 0 );

	// get mode info
	// mode = r_mode->integer;
	// ri.Printf( PRINT_ALL, "...setting mode %d:", mode );

	// if ( !R_GetModeInfo( &glConfig.vidWidth, &glConfig.vidHeight, &glConfig.windowAspect, mode ) )  {
	//     ri.Printf( PRINT_ALL, " invalid mode\n" );
	//     return false;
	// }
	// ri.Printf( PRINT_ALL, " %d %d\n", glConfig.vidWidth, glConfig.vidHeight );

	// /* Create window */
	// if ( r_fullscreen->integer ) {
	// 	int		actualWidth, actualHeight;

	// 	// change display resolution
	// 	GLimp_ChangeDisplay( &actualWidth, &actualHeight );

	// 	x = ( actualWidth - glConfig.vidWidth ) / 2;
	// 	y = ( actualHeight - glConfig.vidHeight ) / 2;
	// 	sys_gl.drawable = (AGLDrawable) GetNewCWindow(kFullScreenWindow,nil,(WindowPtr)-1L);
	// } else {
	// 	x = vid_xpos->integer;
	// 	y = vid_ypos->integer;
	// 	sys_gl.drawable = (AGLDrawable) GetNewCWindow(kMainWindow,nil,(WindowPtr)-1L);
	// }
	// if( !sys_gl.drawable ) {
	// 	return qfalse;
	// }

	// SizeWindow((GrafPort *) sys_gl.drawable, glConfig.vidWidth, glConfig.vidHeight,GL_FALSE);
	// MoveWindow((GrafPort *) sys_gl.drawable,x, y, GL_FALSE);
	// ShowWindow((GrafPort *) sys_gl.drawable);
	// SetPort((GrafPort *) sys_gl.drawable);
	// CToPStr("Quake3: Arena", pstr);
	// SetWTitle((GrafPort *) sys_gl.drawable, pstr);
	// HiliteWindow((GrafPort *) sys_gl.drawable, 1);

	return 1; // qtrue
}

// ===================
// GLimp_SetMode
//
// Returns false if the mode / fullscrenn / options combination failed,
// so another fallback can be tried
// ===================
pub unsafe fn GLimp_SetMode() -> qboolean {
	let mut value: GLint = 0;
	let mut attrib: [GLint; 64] = [0; 64];
	let mut i: c_int = 0;

	if CreateGameWindow() == 0 {
		// ri.Printf( PRINT_ALL, "GLimp_Init: window could not be created" );
		return 0; // qfalse
	}

	// check devices now that the game has set the display mode,
	// because RAVE devices don't get reported if in an 8 bit desktop
	CheckDevices();

	// set up the attribute list
	i = 0;
	attrib[i as usize] = AGL_RGBA;
	i += 1;
	attrib[i as usize] = AGL_DOUBLEBUFFER;
	i += 1;
	attrib[i as usize] = AGL_NO_RECOVERY;
	i += 1;
	attrib[i as usize] = AGL_ACCELERATED;
	i += 1;

	// if ( r_colorbits->integer >= 16 ) {
	// 	attrib[i++] = AGL_RED_SIZE;
	// 	attrib[i++] = 8;
	// 	attrib[i++] = AGL_GREEN_SIZE;
	// 	attrib[i++] = 8;
	// 	attrib[i++] = AGL_BLUE_SIZE;
	// 	attrib[i++] = 8;
	// 	attrib[i++] = AGL_ALPHA_SIZE;
	// 	attrib[i++] = 0;
	// } else {
	// 	attrib[i++] = AGL_RED_SIZE;
	// 	attrib[i++] = 5;
	// 	attrib[i++] = AGL_GREEN_SIZE;
	// 	attrib[i++] = 5;
	// 	attrib[i++] = AGL_BLUE_SIZE;
	// 	attrib[i++] = 5;
	// 	attrib[i++] = AGL_ALPHA_SIZE;
	// 	attrib[i++] = 0;
	// }

	// attrib[i++] = AGL_STENCIL_SIZE;
	// if ( r_stencilbits->integer ) {
	// 	attrib[i++] = r_stencilbits->integer;
	// } else {
	// 	attrib[i++] = 0;
	// }

	// attrib[i++] = AGL_DEPTH_SIZE;
	// if ( r_depthbits->integer ) {
	// 	attrib[i++] = r_depthbits->integer;
	// } else {
	// 	attrib[i++] = 16;
	// }

	attrib[i as usize] = 0;

	// /* Choose pixel format */
	// ri.Printf( PRINT_ALL, "aglChoosePixelFormat\n" );
	// if ( r_device->integer < 0 || r_device->integer >= sys_gl.numDevices ) {
	// 	ri.Cvar_Set( "r_device", "0" );
	// }
	// sys_gl.fmt = aglChoosePixelFormat( &sys_gl.devices[ r_device->integer ], 1, attrib);
	// if(!sys_gl.fmt) {
	// 	ri.Printf( PRINT_ALL, "GLimp_Init: Pixel format could not be achieved\n");
	// 	return qfalse;
	// }
	// ri.Printf( PRINT_ALL, "Selected pixel format 0x%x\n", (int)sys_gl.fmt );

	// aglDescribePixelFormat(sys_gl.fmt, AGL_RED_SIZE, &value);
	// glConfig.colorBits = value * 3;
	// aglDescribePixelFormat(sys_gl.fmt, AGL_STENCIL_SIZE, &value);
	// glConfig.stencilBits = value;
	// aglDescribePixelFormat(sys_gl.fmt, AGL_DEPTH_SIZE, &value);
	// glConfig.depthBits = value;

	// CheckErrors();

	// /* Create context */
	// sys_gl.context = aglCreateContext(sys_gl.fmt, NULL);
	// if(!sys_gl.context) {
	// 	ri.Printf( PRINT_ALL, "GLimp_init: Context could not be created\n");
	// 	return qfalse;
	// }

	// CheckErrors();

	// /* Make context current */

	// if(!aglSetDrawable(sys_gl.context, sys_gl.drawable)) {
	// 	ri.Printf( PRINT_ALL, "GLimp_Init: Could not attach to context\n" );
	// 	return qfalse;
	// }

	// CheckErrors();

	// if( !aglSetCurrentContext(sys_gl.context) ) {
	// 	ri.Printf( PRINT_ALL, "GLimp_Init: Could not attach to context");
	// 	return qfalse;
	// }

	// CheckErrors();

	// // check video memory and determine ragePro status
	// #if 0
	// if ( !GLimp_SufficientVideoMemory() ) {
	// 	return qfalse;
	// }
	// #endif
	// glConfig.hardwareType = sys_hardwareType;		// FIXME: this isn't really right

	// // draw something to show that GL is alive
	// qglClearColor( 1, 0.5, 0.2, 0 );
	// qglClear( GL_COLOR_BUFFER_BIT );
	// GLimp_EndFrame();

	// CheckErrors();

	// // get the extensions
	// GLimp_Extensions();

	// CheckErrors();

	return 1; // qtrue
}

// ===================
// GLimp_Init
//
// Don't return unless OpenGL has been properly initialized
// ===================
pub unsafe fn GLimp_Init() {
	let mut major: GLint = 0;
	let mut minor: GLint = 0;
	static mut registered: qboolean = 0;

	// ri.Printf( PRINT_ALL, "--- GLimp_Init ---\n" );

	aglGetVersion(&mut major, &mut minor);
	// ri.Printf( PRINT_ALL, "aglVersion: %i.%i\n", (int)major, (int)minor );

	// r_device = ri.Cvar_Get( "r_device", "0", CVAR_LATCH | CVAR_ARCHIVE );
	// r_ext_transform_hint = ri.Cvar_Get( "r_ext_transform_hint", "1", CVAR_LATCH | CVAR_ARCHIVE );

	if registered == 0 {
		// ri.Cmd_AddCommand( "aglDescribe", GLimp_AglDescribe_f );
		// ri.Cmd_AddCommand( "aglState", GLimp_AglState_f );
	}

	// memset( &glConfig, 0, sizeof( glConfig ) );

	// r_swapInterval->modified = qtrue;	// force a set next frame

	// glConfig.deviceSupportsGamma = qtrue;

	// // FIXME: try for a voodoo first
	// sys_gl.systemGammas = GetSystemGammas();

	// if ( GLimp_SetMode() ) {
	// 	ri.Printf( PRINT_ALL, "------------------\n" );
	// 	return;
	// }

	// // fall back to the known-good mode
	// ri.Cvar_Set( "r_fullscreen", "1" );
	// ri.Cvar_Set( "r_mode", "3" );
	// ri.Cvar_Set( "r_stereo", "0" );
	// ri.Cvar_Set( "r_depthBits", "16" );
	// ri.Cvar_Set( "r_colorBits", "16" );
	// ri.Cvar_Set( "r_stencilBits", "0" );
	// if ( GLimp_SetMode() ) {
	// 	ri.Printf( PRINT_ALL, "------------------\n" );
	// 	return;
	// }

	// ri.Error( ERR_FATAL, "Could not initialize OpenGL" );
}

// ===============
// GLimp_EndFrame
//
// ===============
pub unsafe fn GLimp_EndFrame() {
	// check for variable changes
	// if ( r_swapInterval->modified ) {
	// 	r_swapInterval->modified = qfalse;
	// 	ri.Printf( PRINT_ALL, "Changing AGL_SWAP_INTERVAL\n" );
	// 	aglSetInteger( sys_gl.context, AGL_SWAP_INTERVAL, (long *)&r_swapInterval->integer );
	// }

	// make sure the event loop is pumped
	Sys_SendKeyEvents();

	// aglSwapBuffers( sys_gl.context );
}

// ===============
// GLimp_Shutdown
//
// ===============
pub unsafe fn GLimp_Shutdown() {
	if !sys_gl.systemGammas.is_null() {
		RestoreSystemGammas(sys_gl.systemGammas);
		DisposeSystemGammas(&mut sys_gl.systemGammas);
		sys_gl.systemGammas = core::ptr::null_mut();
	}

	if !(&mut sys_gl.context as *mut _ as *mut c_void).is_null() {
		// aglDestroyContext(sys_gl.context);
		// sys_gl.context = 0;
	}
	if !(&mut sys_gl.fmt as *mut _ as *mut c_void).is_null() {
		// aglDestroyPixelFormat(sys_gl.fmt);
		// sys_gl.fmt = 0;
	}
	if !(&mut sys_gl.drawable as *mut _ as *mut c_void).is_null() {
		// DisposeWindow((GrafPort *) sys_gl.drawable);
		// sys_gl.drawable = 0;
	}
	GLimp_ResetDisplay();

	// memset( &glConfig, 0, sizeof( glConfig ) );
}

pub unsafe fn GLimp_LogComment(_comment: *mut c_char) {}

pub unsafe fn GLimp_SpawnRenderThread(_function: Option<extern "C" fn()>) -> qboolean {
	0
}

pub unsafe fn GLimp_RendererSleep() -> *mut c_void {
	core::ptr::null_mut()
}

pub unsafe fn GLimp_FrontEndSleep() {}

pub unsafe fn GLimp_WakeRenderer(_data: *mut c_void) {}

// ===============
// GLimp_SetGamma
//
// ===============
pub unsafe fn GLimp_SetGamma(
	red: *mut c_char,
	green: *mut c_char,
	blue: *mut c_char,
) {
	let mut color: [[c_char; 256]; 3] = [[0; 256]; 3];
	let mut i: c_int = 0;

	while i < 256 {
		color[0][i as usize] = *red.add(i as usize);
		color[1][i as usize] = *green.add(i as usize);
		color[2][i as usize] = *blue.add(i as usize);
		i += 1;
	}
	SetDeviceGammaRampGD(sys_gl.device, &mut color[0] as *mut _);
}
