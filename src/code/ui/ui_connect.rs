// leave this at the top of all UI_xxxx files for PCH reasons...
//
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// External C functions
extern "C" {
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn UI_UpdateScreen();
    fn UI_DrawHandlePic(x: c_int, y: c_int, w: c_int, h: c_int, hShader: c_int);
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
}

// Type stubs
pub type qhandle_t = c_int;

// Constants
pub const SCREEN_WIDTH: c_int = 640;
pub const SCREEN_HEIGHT: c_int = 480;
pub const A_ESCAPE: c_int = 27;
pub const EXEC_APPEND: c_int = 0;

// External globals (from ui_local.h)
extern "C" {
    pub static mut uis: uis_t;
    pub static mut ui: ui_t;
}

// Type stubs for external structs
#[repr(C)]
pub struct uis_t {
    pub menuBackShader: qhandle_t,
    // ... other fields not needed for this file
}

#[repr(C)]
pub struct ui_t {
    pub Cmd_ExecuteText: unsafe extern "C" fn(c_int, *const c_char),
    // ... other fields not needed for this file
}

/*
===============================================================================

CONNECTION SCREEN

===============================================================================
*/

pub static mut connectionDialogString: [c_char; 1024] = [0; 1024];
pub static mut connectionMessageString: [c_char; 1024] = [0; 1024];


/*
========================
UI_DrawConnect

========================
*/

pub fn UI_DrawConnect(servername: *const c_char, updateInfoString: *const c_char) {
	// if connecting to a local host, don't draw anything before the
	// gamestate message.  This allows cinematics to start seamlessly
	// if ( connState < CA_LOADING && !strcmp( cls.servername, "localhost" ) ) {
	// 	UI_SetColor( g_color_table[0] );
	// 	re.DrawFill (0, 0, re.scrWidth, re.scrHeight);
	// 	UI_SetColor( NULL );
	// 	return;
	// }

	//	qboolean qValid;
	//	byte *levelPic = ui.SCR_GetScreenshot(&qValid);
	// draw the dialog background
	//	if (!qValid)
	{
		unsafe {
			UI_DrawHandlePic(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, uis.menuBackShader);
		}
	}
	//	else {
	//		UI_DrawThumbNail(0,SCREEN_HEIGHT, SCREEN_WIDTH, -SCREEN_HEIGHT, levelPic );
	//	}
}


/*
========================
UI_UpdateConnectionString

========================
*/
pub fn UI_UpdateConnectionString(string: *mut c_char) {
	unsafe {
		Q_strncpyz(connectionDialogString.as_mut_ptr(), string, 1024);
		UI_UpdateScreen();
	}
}

/*
========================
UI_UpdateConnectionMessageString

========================
*/
pub fn UI_UpdateConnectionMessageString(string: *mut c_char) {
	let mut s: *mut c_char;

	unsafe {
		Q_strncpyz(connectionMessageString.as_mut_ptr(), string, 1024);

		// strip \n
		s = strstr(connectionMessageString.as_mut_ptr(), b"\n\0".as_ptr() as *const c_char);
		if !s.is_null() {
			*s = 0 as c_char;
		}
		UI_UpdateScreen();
	}
}

/*
===================
UI_KeyConnect
===================
*/
pub fn UI_KeyConnect(key: c_int)
{
	if key == A_ESCAPE
	{
		unsafe {
			(ui.Cmd_ExecuteText)(EXEC_APPEND, b"disconnect\n\0".as_ptr() as *const c_char);
		}
		return;
	}
}

