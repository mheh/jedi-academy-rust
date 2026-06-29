// cl_scrn.c -- master for refresh, status bar, console, chat, notify, etc

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Re-export dependencies from other modules
use crate::code::game::q_shared::*;
use crate::code::qcommon::qcommon::*;
use crate::code::renderer::tr_public::*;
use crate::code::renderer::tr_types_h::*;
use crate::code::client::client_h::*;
use crate::code::client::client_ui_h::*;

pub extern "C" {
	pub static con: console_t;
}

pub static mut scr_initialized: qboolean = 0;		// ready to draw

pub extern "C" {
	pub static mut cl_timegraph: *mut cvar_t;
	pub static mut cl_debuggraph: *mut cvar_t;
	pub static mut cl_graphheight: *mut cvar_t;
	pub static mut cl_graphscale: *mut cvar_t;
	pub static mut cl_graphshift: *mut cvar_t;
}

// Connection state constants
const CA_UNINITIALIZED: c_int = 0;
const CA_DISCONNECTED: c_int = 1;
const CA_CONNECTING: c_int = 2;
const CA_CHALLENGING: c_int = 3;
const CA_CONNECTED: c_int = 4;
const CA_PRIMED: c_int = 5;
const CA_ACTIVE: c_int = 6;
const CA_CINEMATIC: c_int = 7;

// Character size constants
const BIGCHAR_HEIGHT: c_int = 16;
const BIGCHAR_WIDTH: c_int = 16;
const SMALLCHAR_HEIGHT: c_int = 16;
const SMALLCHAR_WIDTH: c_int = 8;

/*
================
SCR_DrawNamedPic

Coordinates are 640*480 virtual values
=================
*/
pub unsafe extern "C" fn SCR_DrawNamedPic( x: f32, y: f32, width: f32, height: f32, picname: *const c_char ) {
	let hShader: qhandle_t;

	assert!(width != 0.0);

	hShader = (*addr_of!(re).cast::<refexport_t>()).RegisterShader.unwrap()(picname);
	(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}


/*
================
SCR_FillRect

Coordinates are 640*480 virtual values
=================
*/
pub unsafe extern "C" fn SCR_FillRect( x: f32, y: f32, width: f32, height: f32, color: *const f32 ) {
	(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(color);

	(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(x, y, width, height, 0.0, 0.0, 0.0, 0.0, (*addr_of!(cls).cast::<clientStatic_t>()).whiteShader);

	(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(core::ptr::null());
}


/*
================
SCR_DrawPic

Coordinates are 640*480 virtual values
A width of 0 will draw with the original image width
=================
*/
pub unsafe extern "C" fn SCR_DrawPic( x: f32, y: f32, width: f32, height: f32, hShader: qhandle_t ) {
	(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}


/*
** SCR_DrawBigChar
** big chars are drawn at 640*480 virtual screen size
*/
pub unsafe extern "C" fn SCR_DrawBigChar( x: c_int, y: c_int, ch: c_int ) {
	let mut row: c_int;
	let mut col: c_int;
	let mut frow: f32;
	let mut fcol: f32;
	let mut size: f32;
	let mut ax: f32;
	let mut ay: f32;
	let mut aw: f32;
	let mut ah: f32;

	let ch = ch & 255;

	if ch == ' ' as c_int {
		return;
	}

	if y < -BIGCHAR_HEIGHT {
		return;
	}

	ax = x as f32;
	ay = y as f32;
	aw = BIGCHAR_WIDTH as f32;
	ah = BIGCHAR_HEIGHT as f32;

	row = ch>>4;
	col = ch&15;

	frow = (row as f32)*0.0625;
	fcol = (col as f32)*0.0625;
	size = 0.0625;
/*
	(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(ax, ay, aw, ah,
					   fcol, frow,
					   fcol + size, frow + size,
					   (*addr_of!(cls).cast::<clientStatic_t>()).charSetShader);
*/
	let size2: f32;

	frow = (row as f32)*0.0625;
	fcol = (col as f32)*0.0625;
	size = 0.03125;
	size2 = 0.0625;

	(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(ax, ay, aw, ah,
					   fcol, frow,
					   fcol + size, frow + size2,
					   (*addr_of!(cls).cast::<clientStatic_t>()).charSetShader);

}

/*
** SCR_DrawSmallChar
** small chars are drawn at native screen resolution
*/
pub unsafe extern "C" fn SCR_DrawSmallChar( x: c_int, y: c_int, ch: c_int ) {
	let mut row: c_int;
	let mut col: c_int;
	let mut frow: f32;
	let mut fcol: f32;
	let mut size: f32;

	let ch = ch & 255;

	if ch == ' ' as c_int {
		return;
	}

	if y < -SMALLCHAR_HEIGHT {
		return;
	}

	row = ch>>4;
	col = ch&15;
/*
	frow = row*0.0625;
	fcol = col*0.0625;
	size = 0.0625;

	(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(x, y, SMALLCHAR_WIDTH, SMALLCHAR_HEIGHT,
					   fcol, frow,
					   fcol + size, frow + size,
					   (*addr_of!(cls).cast::<clientStatic_t>()).charSetShader);
*/

	let size2: f32;

	frow = (row as f32)*0.0625;
	fcol = (col as f32)*0.0625;
	size = 0.03125;
	size2 = 0.0625;

	(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(
		(x as f32) * (*addr_of!(con).cast::<console_t>()).xadjust,
		(y as f32) * (*addr_of!(con).cast::<console_t>()).yadjust,
		(SMALLCHAR_WIDTH as f32) * (*addr_of!(con).cast::<console_t>()).xadjust,
		(SMALLCHAR_HEIGHT as f32) * (*addr_of!(con).cast::<console_t>()).yadjust,
		fcol, frow,
		fcol + size, frow + size2,
		(*addr_of!(cls).cast::<clientStatic_t>()).charSetShader);

}



/*
==================
SCR_DrawBigString[Color]

Draws a multi-colored string with a drop shadow, optionally forcing
to a fixed color.

Coordinates are at 640 by 480 virtual resolution
==================
*/
pub unsafe extern "C" fn SCR_DrawBigStringExt( x: c_int, y: c_int, string: *const c_char, setColor: *mut f32, forceColor: qboolean ) {
	let mut color: [f32; 4] = [0.0; 4];
	let mut s: *const c_char;
	let mut xx: c_int;

	// draw the drop shadow
	color[0] = 0.0;
	color[1] = 0.0;
	color[2] = 0.0;
	color[3] = *(setColor.add(3));
	(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(color.as_ptr());
	s = string;
	xx = x;
	while *s != 0 {
		if Q_IsColorString(s) != 0 {
			s = s.add(2);
			continue;
		}
		SCR_DrawBigChar(xx+2, y+2, *s as c_int);
		xx+=16;
		s = s.add(1);
	}


	// draw the colored text
	s = string;
	xx = x;
	(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(setColor);
	while *s != 0 {
		if Q_IsColorString(s) != 0 {
			if forceColor == 0 {
				let color_idx = ColorIndex(*(s.add(1)) as u8) as usize;
				core::ptr::copy_nonoverlapping(
					(addr_of!(g_color_table) as *const [f32; 4]).add(color_idx) as *const u8,
					color.as_mut_ptr() as *mut u8,
					core::mem::size_of::<[f32; 4]>()
				);
				color[3] = *(setColor.add(3));
				(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(color.as_ptr());
			}
			s = s.add(2);
			continue;
		}
		SCR_DrawBigChar(xx, y, *s as c_int);
		xx+=16;
		s = s.add(1);
	}
	(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(core::ptr::null());
}


pub unsafe extern "C" fn SCR_DrawBigString( x: c_int, y: c_int, s: *const c_char, alpha: f32 ) {
	let mut color: [f32; 4] = [0.0; 4];

	color[0] = 1.0;
	color[1] = 1.0;
	color[2] = 1.0;
	color[3] = alpha;
	SCR_DrawBigStringExt(x, y, s, color.as_mut_ptr(), 0);
}

pub unsafe extern "C" fn SCR_DrawBigStringColor( x: c_int, y: c_int, s: *const c_char, color: vec4_t ) {
	SCR_DrawBigStringExt(x, y, s, color.as_ptr() as *mut f32, 1);
}


/*
** SCR_Strlen -- skips color escape codes
*/
unsafe fn SCR_Strlen( str: *const c_char ) -> c_int {
	let mut s: *const c_char = str;
	let mut count: c_int = 0;

	while *s != 0 {
		if Q_IsColorString(s) != 0 {
			s = s.add(2);
		} else {
			count+=1;
			s = s.add(1);
		}
	}

	return count;
}

/*
** SCR_GetBigStringWidth
*/
pub unsafe extern "C" fn SCR_GetBigStringWidth( str: *const c_char ) -> c_int {
	return SCR_Strlen(str) * 16;
}

//===============================================================================


/*
===============================================================================

DEBUG GRAPH

===============================================================================
*/
#[cfg(not(target_os = "xbox"))]
#[repr(C)]
#[derive(Copy, Clone)]
struct graphsamp_t {
	value: f32,
	color: c_int,
}

#[cfg(not(target_os = "xbox"))]
static mut current: c_int = 0;

#[cfg(not(target_os = "xbox"))]
static mut values: [graphsamp_t; 1024] = [graphsamp_t { value: 0.0, color: 0 }; 1024];

/*
==============
SCR_DebugGraph
==============
*/
#[cfg(not(target_os = "xbox"))]
pub unsafe extern "C" fn SCR_DebugGraph (value: f32, color: c_int)
{
	values[(current as usize)&1023].value = value;
	values[(current as usize)&1023].color = color;
	current += 1;
}

#[cfg(target_os = "xbox")]
pub unsafe extern "C" fn SCR_DebugGraph(_value: f32, _color: c_int) {}

/*
==============
SCR_DrawDebugGraph
==============
*/
#[cfg(not(target_os = "xbox"))]
pub unsafe extern "C" fn SCR_DrawDebugGraph ()
{
	let mut a: c_int;
	let mut x: c_int;
	let mut y: c_int;
	let mut w: c_int;
	let mut i: c_int;
	let mut h: c_int;
	let mut v: f32;
	let mut color: c_int;

	//
	// draw the graph
	//
	w = (*addr_of!(cls).cast::<clientStatic_t>()).glconfig.vidWidth;
	x = 0;
	y = (*addr_of!(cls).cast::<clientStatic_t>()).glconfig.vidHeight;
	(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(g_color_table.as_ptr());
	(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(
		x as f32, (y - (*(*addr_of!(cl_graphheight)).cast::<cvar_t>()).integer) as f32,
		w as f32, (*(*addr_of!(cl_graphheight)).cast::<cvar_t>()).integer as f32, 0.0, 0.0, 0.0, 0.0, 0);
	(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(core::ptr::null());

	a = 0;
	while a < w {
		i = ((current-1-a+1024) & 1023) as c_int;
		v = values[i as usize].value;
		color = values[i as usize].color;
		v = v * (*(*addr_of!(cl_graphscale)).cast::<cvar_t>()).integer as f32 + (*(*addr_of!(cl_graphshift)).cast::<cvar_t>()).integer as f32;

		if v < 0.0 {
			v += (*(*addr_of!(cl_graphheight)).cast::<cvar_t>()).integer as f32 * (1.0+((-v / (*(*addr_of!(cl_graphheight)).cast::<cvar_t>()).integer as f32).floor()));
		}
		h = (v as c_int) % (*(*addr_of!(cl_graphheight)).cast::<cvar_t>()).integer;
		(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(
			(x+w-1-a) as f32, (y - h) as f32, 1.0, h as f32, 0.0, 0.0, 0.0, 0.0, 0);
		a += 1;
	}
}

#[cfg(target_os = "xbox")]
pub unsafe extern "C" fn SCR_DrawDebugGraph() {}

//=============================================================================

/*
==================
SCR_Init
==================
*/
pub unsafe extern "C" fn SCR_Init( ) {
	*addr_of_mut!(cl_timegraph) = Cvar_Get(b"timegraph\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
	*addr_of_mut!(cl_debuggraph) = Cvar_Get(b"debuggraph\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
	*addr_of_mut!(cl_graphheight) = Cvar_Get(b"graphheight\0".as_ptr() as *const c_char, b"32\0".as_ptr() as *const c_char, 0);
	*addr_of_mut!(cl_graphscale) = Cvar_Get(b"graphscale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
	*addr_of_mut!(cl_graphshift) = Cvar_Get(b"graphshift\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

	scr_initialized = 1;
}


//=======================================================

pub extern "C" {
	pub fn UI_SetActiveMenu( menuname: *const c_char, menuID: *const c_char );
	pub fn _UI_Refresh( realtime: c_int );
	pub fn UI_DrawConnect( servername: *const c_char, updateInfoString: *const c_char );
}

/*
==================
SCR_DrawScreenField

This will be called twice if rendering in stereo mode
==================
*/
pub unsafe extern "C" fn SCR_DrawScreenField( stereoFrame: stereoFrame_t ) {

	(*addr_of!(re).cast::<refexport_t>()).BeginFrame.unwrap()(stereoFrame);

	// wide aspect ratio screens need to have the sides cleared
	// unless they are displaying game renderings
	if (*addr_of!(cls).cast::<clientStatic_t>()).state != CA_ACTIVE {
		if (*addr_of!(cls).cast::<clientStatic_t>()).glconfig.vidWidth * 480 > (*addr_of!(cls).cast::<clientStatic_t>()).glconfig.vidHeight * 640 {
			(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(g_color_table.as_ptr());
			(*addr_of!(re).cast::<refexport_t>()).DrawStretchPic.unwrap()(
				0.0, 0.0,
				(*addr_of!(cls).cast::<clientStatic_t>()).glconfig.vidWidth as f32,
				(*addr_of!(cls).cast::<clientStatic_t>()).glconfig.vidHeight as f32,
				0.0, 0.0, 0.0, 0.0, 0);
			(*addr_of!(re).cast::<refexport_t>()).SetColor.unwrap()(core::ptr::null());
		}
	}

	// if the menu is going to cover the entire screen, we
	// don't need to render anything under it
	if _UI_IsFullscreen() == 0 {
		let cls_state = (*addr_of!(cls).cast::<clientStatic_t>()).state;
		match cls_state {
			CA_CINEMATIC => {
				SCR_DrawCinematic();
			},
			CA_DISCONNECTED => {
				// force menu up
				UI_SetActiveMenu(b"mainMenu\0".as_ptr() as *const c_char, core::ptr::null());	//			VM_Call( uivm, UI_SET_ACTIVE_MENU, UIMENU_MAIN );
			},
			CA_CONNECTING | CA_CHALLENGING | CA_CONNECTED => {
				// connecting clients will only show the connection dialog
				UI_DrawConnect((*addr_of!(clc).cast::<clientConnection_t>()).servername.as_ptr(), (*addr_of!(cls).cast::<clientStatic_t>()).updateInfoString.as_ptr());
			},
			CA_LOADING | CA_PRIMED => {
				// draw the game information screen and loading progress
				CL_CGameRendering(stereoFrame);
			},
			CA_ACTIVE => {
				if CL_IsRunningInGameCinematic() != 0 || CL_InGameCinematicOnStandBy() != 0 {
					SCR_DrawCinematic();
				}
				else {
					CL_CGameRendering(stereoFrame);
				}
			},
			_ => {
				Com_Error(1, b"SCR_DrawScreenField: bad cls.state\0".as_ptr() as *const c_char);
			},
		}
	}

#[cfg(not(target_os = "xbox"))]
	{
		(*addr_of!(re).cast::<refexport_t>()).ProcessDissolve.unwrap()();
	}

	// draw downloading progress bar

	// the menu draws next
	_UI_Refresh((*addr_of!(cls).cast::<clientStatic_t>()).realtime);

	// console draws next
	Con_DrawConsole();

	// debug graph can be drawn on top of anything
#[cfg(not(target_os = "xbox"))]
	{
		if (*(*addr_of!(cl_debuggraph)).cast::<cvar_t>()).integer != 0 || (*(*addr_of!(cl_timegraph)).cast::<cvar_t>()).integer != 0 {
			SCR_DrawDebugGraph();
		}
	}
}

/*
==================
SCR_UpdateScreen

This is called every frame, and can also be called explicitly to flush
text to the screen.
==================
*/
pub unsafe extern "C" fn SCR_UpdateScreen( ) {
	static mut recursive: c_int = 0;

	if scr_initialized == 0 {
		return;				// not initialized yet
	}

	// load the ref / ui / cgame if needed
	CL_StartHunkUsers();

	recursive += 1;
	if recursive > 2 {
		Com_Error(1, b"SCR_UpdateScreen: recursively called\0".as_ptr() as *const c_char);
	}
	recursive = 1;

	// if running in stereo, we need to draw the frame twice
	if (*addr_of!(cls).cast::<clientStatic_t>()).glconfig.stereoEnabled != 0 {
		SCR_DrawScreenField(stereoFrame_t::STEREO_LEFT);
		SCR_DrawScreenField(stereoFrame_t::STEREO_RIGHT);
	} else {
		SCR_DrawScreenField(stereoFrame_t::STEREO_CENTER);
	}

	if (*(*addr_of!(com_speeds)).cast::<cvar_t>()).integer != 0 {
		(*addr_of!(re).cast::<refexport_t>()).EndFrame.unwrap()(
			addr_of_mut!(time_frontend) as *mut c_int,
			addr_of_mut!(time_backend) as *mut c_int);
	} else {
		(*addr_of!(re).cast::<refexport_t>()).EndFrame.unwrap()(core::ptr::null_mut(), core::ptr::null_mut());
	}

	recursive = 0;
}

// this stuff is only used by the savegame (SG) code for screenshots...
//
#[cfg(target_os = "xbox")]
pub unsafe extern "C" fn SCR_PrecacheScreenshot()
{
	// No screenshots unless connected to single player local server...
	//
//	char *psInfo = cl.gameState.stringData + cl.gameState.stringOffsets[ CS_SERVERINFO ];
//	int iMaxClients = atoi(Info_ValueForKey( psInfo, "sv_maxclients" ));

	// (no need to check single-player status in voyager, this code base is all singleplayer)
	if (*addr_of!(cls).cast::<clientStatic_t>()).state != CA_ACTIVE {
		return;
	}

#[cfg(not(target_os = "xbox"))]
	{
		if (*addr_of!(cls).cast::<clientStatic_t>()).keyCatchers == 0 {
			// in-game...
			//
	//		SCR_UnprecacheScreenshot();
	//		pbScreenData = (byte *)Z_Malloc(SG_SCR_WIDTH * SG_SCR_HEIGHT * 4);
			S_ClearSoundBuffer();	// clear DMA etc because the following glReadPixels() call can take ages
			// re.GetScreenShot( (byte *) &bScreenData, SG_SCR_WIDTH, SG_SCR_HEIGHT);
			// screenDataValid = qtrue;
		}
		else {
			// we're in the console, or menu, or message input...
			//
		}
	}

	// save the current screenshot to the user space to be used
	// with a savegame
#[cfg(target_os = "xbox")]
	{
		extern "C" {
			pub fn SaveCompressedScreenshot( filename: *const c_char );
		}
		SaveCompressedScreenshot(b"u:\\saveimage.xbx\0".as_ptr() as *const c_char);
	}

}

/*
byte *SCR_GetScreenshot(qboolean *qValid)
{
	if (!screenDataValid) {
		SCR_PrecacheScreenshot();
	}
	if (qValid) {
		*qValid = screenDataValid;
	}
	return (byte *)&bScreenData;
}

// called from save-game code to set the lo-res loading screen to be the one from the save file...
//
void SCR_SetScreenshot(const byte *pbData, int w, int h)
{
	if (w == SG_SCR_WIDTH && h == SG_SCR_HEIGHT)
	{
		screenDataValid = qtrue;
		memcpy(&bScreenData, pbData, SG_SCR_WIDTH*SG_SCR_HEIGHT*4);
	}
	else
	{
		screenDataValid = qfalse;
		memset(&bScreenData, 0,      SG_SCR_WIDTH*SG_SCR_HEIGHT*4);
	}
}
*/

// This is just a client-side wrapper for the function RE_TempRawImage_ReadFromFile() in the renderer code...
//
/*
byte* SCR_TempRawImage_ReadFromFile(const char *psLocalFilename, int *piWidth, int *piHeight, byte *pbReSampleBuffer, qboolean qbVertFlip)
{
	return re.TempRawImage_ReadFromFile(psLocalFilename, piWidth, piHeight, pbReSampleBuffer, qbVertFlip);
}
//
// ditto (sort of)...
//
void  SCR_TempRawImage_CleanUp()
{
	re.TempRawImage_CleanUp();
}
*/

// External declarations for types and functions needed but not fully defined in this module
pub extern "C" {
	pub static mut com_speeds: *mut cvar_t;
	pub static mut time_frontend: c_int;
	pub static mut time_backend: c_int;
	pub fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
	pub fn Q_IsColorString(p: *const c_char) -> c_int;
	pub fn ColorIndex(c: u8) -> c_int;
	pub static g_color_table: [[f32; 4]; 16];
	pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
	pub fn S_ClearSoundBuffer();
	pub fn CL_StartHunkUsers();
	pub fn Con_DrawConsole();
	pub fn SCR_DrawCinematic();
	pub fn CL_CGameRendering(stereo: stereoFrame_t);
	pub fn CL_IsRunningInGameCinematic() -> qboolean;
	pub fn CL_InGameCinematicOnStandBy() -> qboolean;
}

#[repr(C)]
pub struct cvar_t {
	_opaque: [u8; 0],
}
