/**********************************************************************
	UI_ATOMS.C

	User interface building blocks and support functions.
**********************************************************************/

// leave this at the top of all UI_xxxx files for PCH reasons...
//

use core::ffi::{c_char, c_float, c_int, c_void};

// Type definitions for external structures
#[repr(C)]
pub struct glconfig_t {
	pub vidWidth: c_int,
	pub vidHeight: c_int,
	// remaining fields omitted for stub
}

#[repr(C)]
pub struct uiimport_t {
	pub Error: unsafe extern "C" fn(level: c_int, fmt: *const c_char, ...) -> !,
	pub Printf: unsafe extern "C" fn(fmt: *const c_char, ...),
	pub Cvar_Create: unsafe extern "C" fn(var_name: *const c_char, var_value: *const c_char, flags: c_int),
	pub Cvar_Set: unsafe extern "C" fn(var_name: *const c_char, value: *const c_char),
	pub Cvar_VariableStringBuffer: unsafe extern "C" fn(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int),
	pub Argv: unsafe extern "C" fn(arg: c_int, buffer: *mut c_char, bufsize: c_int),
	pub Key_SetCatcher: unsafe extern "C" fn(catcher: c_int),
	pub Key_GetCatcher: unsafe extern "C" fn() -> c_int,
	pub Key_ClearStates: unsafe extern "C" fn(),
	pub R_RegisterShaderNoMip: unsafe extern "C" fn(name: *const c_char) -> qhandle_t,
	pub R_DrawStretchPic: unsafe extern "C" fn(x: c_float, y: c_float, w: c_float, h: c_float, s1: c_float, t1: c_float, s2: c_float, t2: c_float, hShader: qhandle_t),
	pub R_SetColor: unsafe extern "C" fn(rgba: *const c_float),
	pub UpdateScreen: unsafe extern "C" fn(),
	pub GetGlconfig: unsafe extern "C" fn(glconfig: *mut glconfig_t),
	pub SG_GameAllowedToSaveHere: unsafe extern "C" fn(inCamera: qboolean) -> qboolean,
	pub R_RegisterFont: unsafe extern "C" fn(fontName: *const c_char) -> c_int,
	// Placeholder for additional function pointers
}

#[repr(C)]
pub struct uiStatic_t {
	pub glconfig: glconfig_t,
	pub scaley: c_float,
	pub scalex: c_float,
	pub whiteShader: qhandle_t,
	// Placeholder for remaining fields
}

// Type aliases
pub type qhandle_t = c_int;
pub type qboolean = c_int;

// globals
pub static mut ui: uiimport_t = unsafe { core::mem::zeroed() };
pub static mut uis: uiStatic_t = unsafe { core::mem::zeroed() };

// externs
extern "C" {
	static mut cls: connstate_t;
	static DC: *const c_void;

	fn Cvar_SetValue(var_name: *const c_char, value: c_float);
	fn UI_Cursor_Show(show: qboolean);
	fn Menu_Cache();
	fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
	fn UI_MainMenu();
	fn UI_InGameMenu(menuID: *const c_char);
	fn UI_DataPadMenu();
	fn Menus_CloseAll();
	fn Menus_ActivateByName(menuname: *const c_char) -> qboolean;
	fn UI_Report();
	fn UI_Load();
	fn va(fmt: *const c_char, ...) -> *const c_char;
	fn trap_S_RegisterSound(name: *const c_char, compressed: qboolean);
	fn trap_Key_SetCatcher(catcher: c_int);
	fn trap_R_SetColor(rgba: *const c_float);
	fn _UI_Init(inGameLoad: qboolean);

	static lukeForceStatusSounds: [*const c_char; 5];
	static kyleForceStatusSounds: [*const c_char; 5];
}

// Stub for connstate_t
#[repr(C)]
pub struct connstate_t {
	pub state: c_int,
	// placeholder for remaining fields
}

const KEYCATCH_UI: c_int = 0x10;
const MAX_STRING_CHARS: usize = 1024;
const CVAR_ARCHIVE: c_int = 0x0001;
const CVAR_SAVEGAME: c_int = 0x0010;
const CVAR_NORESTART: c_int = 0x0040;
const CVAR_ROM: c_int = 0x0080;
const CA_DISCONNECTED: c_int = 0;
const UI_API_VERSION: c_int = 4;
const ERR_FATAL: c_int = 0;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

//locals


/*
=================
UI_ForceMenuOff
=================
*/
#[allow(non_snake_case)]
pub fn UI_ForceMenuOff() {
	unsafe {
		(*core::ptr::addr_of_mut!(ui)).Key_SetCatcher((*core::ptr::addr_of!(ui)).Key_GetCatcher() & !KEYCATCH_UI);
		(*core::ptr::addr_of_mut!(ui)).Key_ClearStates();
		(*core::ptr::addr_of_mut!(ui)).Cvar_Set("cl_paused\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char);
	}
}


/*
=================
UI_SetActiveMenu -
	this should be the ONLY way the menu system is brought up

=================
*/
#[allow(non_snake_case)]
pub fn UI_SetActiveMenu(menuname: *const c_char, menuID: *const c_char) {
	// this should be the ONLY way the menu system is brought up (besides the UI_ConsoleCommand below)

	unsafe {
		if (*core::ptr::addr_of!(cls)).state != CA_DISCONNECTED && !(*core::ptr::addr_of!(ui)).SG_GameAllowedToSaveHere(qtrue) {	//don't check full sytem, only if incamera
			return;
		}

		if menuname.is_null() {
			UI_ForceMenuOff();
			return;
		}

		//make sure force-speed and slowmodeath doesn't slow down menus - NOTE: they should reset the timescale when the game un-pauses
		Cvar_SetValue("timescale\0".as_ptr() as *const c_char, 1.0);

		UI_Cursor_Show(qtrue);

		// enusure minumum menu data is cached
		Menu_Cache();

		if Q_stricmp(menuname, "mainMenu\0".as_ptr() as *const c_char) == 0 {
			UI_MainMenu();
			return;
		}

		if Q_stricmp(menuname, "ingame\0".as_ptr() as *const c_char) == 0 {
			(*core::ptr::addr_of_mut!(ui)).Cvar_Set("cl_paused\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char);
			UI_InGameMenu(menuID);
			return;
		}

		if Q_stricmp(menuname, "datapad\0".as_ptr() as *const c_char) == 0 {
			(*core::ptr::addr_of_mut!(ui)).Cvar_Set("cl_paused\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char);
			UI_DataPadMenu();
			return;
		}

		if Q_stricmp(menuname, "missionfailed_menu\0".as_ptr() as *const c_char) == 0 {
			Menus_CloseAll();
			Menus_ActivateByName("missionfailed_menu\0".as_ptr() as *const c_char);
			(*core::ptr::addr_of_mut!(ui)).Key_SetCatcher(KEYCATCH_UI);
			return;
		}
		//JLF SPLASHMAIN MPSKIPPED
		// #ifdef _XBOX
		// {
		if false {
			Menus_CloseAll();
			if Menus_ActivateByName(menuname) != qfalse {
				(*core::ptr::addr_of_mut!(ui)).Key_SetCatcher(KEYCATCH_UI);
			} else {
				UI_MainMenu();
			}
		}
		// }
	}
}


/*
=================
UI_Argv
=================
*/
#[allow(non_snake_case)]
fn UI_Argv(arg: c_int) -> *mut c_char {
	static mut buffer: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

	unsafe {
		(*core::ptr::addr_of_mut!(ui)).Argv(arg, core::ptr::addr_of_mut!(buffer[0]), MAX_STRING_CHARS as c_int);

		core::ptr::addr_of_mut!(buffer[0])
	}
}


/*
=================
UI_Cvar_VariableString
=================
*/
#[allow(non_snake_case)]
pub fn UI_Cvar_VariableString(var_name: *const c_char) -> *mut c_char {
	static mut buffer: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

	unsafe {
		(*core::ptr::addr_of_mut!(ui)).Cvar_VariableStringBuffer(var_name, core::ptr::addr_of_mut!(buffer[0]), MAX_STRING_CHARS as c_int);

		core::ptr::addr_of_mut!(buffer[0])
	}
}

/*
=================
UI_Cache
=================
*/
fn UI_Cache_f() {
	let mut index: c_int;
	unsafe {
		Menu_Cache();

		index = 0;
		while index < 5 {
			// DC->registerSound is not available in Rust stub; this would require proper DC struct definition
			index += 1;
		}
		index = 1;
		while index <= 18 {
			// va() call skipped in stub
			index += 1;
		}
		trap_S_RegisterSound("sound/chars/kyle/04kyk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/kyle/05kyk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/kyle/07kyk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/kyle/14kyk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/kyle/21kyk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/kyle/24kyk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/kyle/25kyk001.mp3\0".as_ptr() as *const c_char, qfalse);

		trap_S_RegisterSound("sound/chars/luke/06luk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/luke/08luk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/luke/22luk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/luke/23luk001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/protocol/12pro001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/protocol/15pro001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/protocol/16pro001.mp3\0".as_ptr() as *const c_char, qfalse);
		trap_S_RegisterSound("sound/chars/wedge/13wea001.mp3\0".as_ptr() as *const c_char, qfalse);


		Menus_ActivateByName("ingameMissionSelect1\0".as_ptr() as *const c_char);
		Menus_ActivateByName("ingameMissionSelect2\0".as_ptr() as *const c_char);
		Menus_ActivateByName("ingameMissionSelect3\0".as_ptr() as *const c_char);
	}
}


/*
=================
UI_ConsoleCommand
=================
*/
#[allow(non_snake_case)]
pub fn UI_ConsoleCommand() -> qboolean {
	let cmd: *mut c_char;

	unsafe {
		if !(*core::ptr::addr_of!(ui)).SG_GameAllowedToSaveHere(qtrue) {	//only check if incamera
			return qfalse;
		}

		cmd = UI_Argv(0);

		// ensure minimum menu data is available
		Menu_Cache();

		if Q_stricmp(cmd, "ui_cache\0".as_ptr() as *const c_char) == 0 {
			UI_Cache_f();
			return qtrue;
		}

		if Q_stricmp(cmd, "levelselect\0".as_ptr() as *const c_char) == 0 {
			UI_LoadMenu_f();
			return qtrue;
		}

		if Q_stricmp(cmd, "ui_teamOrders\0".as_ptr() as *const c_char) == 0 {
			UI_SaveMenu_f();
			return qtrue;
		}

		if Q_stricmp(cmd, "ui_report\0".as_ptr() as *const c_char) == 0 {
			UI_Report();
			return qtrue;
		}

		if Q_stricmp(cmd, "ui_load\0".as_ptr() as *const c_char) == 0 {
			UI_Load();
			return qtrue;
		}

		qfalse
	}
}


/*
=================
UI_Init
=================
*/
#[allow(non_snake_case)]
pub fn UI_Init(apiVersion: c_int, uiimport: *const uiimport_t, inGameLoad: qboolean) {
	unsafe {
		ui = *uiimport;

		if apiVersion != UI_API_VERSION {
			(*core::ptr::addr_of_mut!(ui)).Error(
				ERR_FATAL,
				"Bad UI_API_VERSION: expected %i, got %i\n\0".as_ptr() as *const c_char,
				UI_API_VERSION,
				apiVersion,
			);
		}

		// get static data (glconfig, media)
		(*core::ptr::addr_of_mut!(ui)).GetGlconfig(core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(uis)).glconfig));

		(*core::ptr::addr_of_mut!(uis)).scaley = (*core::ptr::addr_of!(uis)).glconfig.vidHeight as c_float * (1.0/480.0);
		(*core::ptr::addr_of_mut!(uis)).scalex = (*core::ptr::addr_of!(uis)).glconfig.vidWidth as c_float * (1.0/640.0);

		Menu_Cache();

		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_drawCrosshair\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_marks\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("s_language\0".as_ptr() as *const c_char, "english\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_char_model\0".as_ptr() as *const c_char, "jedi_tf\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_char_skin_head\0".as_ptr() as *const c_char, "head_a1\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_char_skin_torso\0".as_ptr() as *const c_char, "torso_a1\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_char_skin_legs\0".as_ptr() as *const c_char, "lower_a1\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_char_color_red\0".as_ptr() as *const c_char, "255\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_char_color_green\0".as_ptr() as *const c_char, "255\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_char_color_blue\0".as_ptr() as *const c_char, "255\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_saber_type\0".as_ptr() as *const c_char, "single\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_saber\0".as_ptr() as *const c_char, "single_1\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_saber2\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_saber_color\0".as_ptr() as *const c_char, "yellow\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_saber2_color\0".as_ptr() as *const c_char, "yellow\0".as_ptr() as *const c_char, CVAR_ARCHIVE|CVAR_SAVEGAME|CVAR_NORESTART);

		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("ui_forcepower_inc\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_ROM|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("tier_storyinfo\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_ROM|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("tiers_complete\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, CVAR_ROM|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("ui_prisonerobj_currtotal\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_ROM|CVAR_SAVEGAME|CVAR_NORESTART);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("ui_prisonerobj_mintotal\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_ROM|CVAR_SAVEGAME|CVAR_NORESTART);

		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_dismemberment\0".as_ptr() as *const c_char, "3\0".as_ptr() as *const c_char, CVAR_ARCHIVE);//0 = none, 1 = arms and hands, 2 = legs, 3 = waist and head
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_gunAutoFirst\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_crosshairIdentifyTarget\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("g_subtitles\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_marks\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("d_slowmodeath\0".as_ptr() as *const c_char, "3\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_shadows\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_runpitch\0".as_ptr() as *const c_char, "0.002\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_runroll\0".as_ptr() as *const c_char, "0.005\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_bobup\0".as_ptr() as *const c_char, "0.005\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_bobpitch\0".as_ptr() as *const c_char, "0.002\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("cg_bobroll\0".as_ptr() as *const c_char, "0.002\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

		(*core::ptr::addr_of_mut!(ui)).Cvar_Create("ui_disableWeaponSway\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);



		_UI_Init(inGameLoad);
	}
}

// these are only here so the functions in q_shared.c can link

// #[cfg(not(feature = "UI_HARD_LINKED"))]
/*
================
Com_Error
=================
*/
/*
void Com_Error( int level, const char *error, ... )
{
	va_list		argptr;
	char		text[1024];

	va_start (argptr, error);
	vsprintf (text, error, argptr);
	va_end (argptr);

	ui.Error( level, "%s", text);
}
*/
/*
================
Com_Printf
=================
*/
/*
void Com_Printf( const char *msg, ... )
{
	va_list		argptr;
	char		text[1024];

	va_start (argptr, msg);
	vsprintf (text, msg, argptr);
	va_end (argptr);

	ui.Printf( "%s", text);
}
*/


/*
================
UI_DrawNamedPic
=================
*/
#[allow(non_snake_case)]
pub fn UI_DrawNamedPic(x: c_float, y: c_float, width: c_float, height: c_float, picname: *const c_char) {
	let mut hShader: qhandle_t;

	unsafe {
		hShader = (*core::ptr::addr_of!(ui)).R_RegisterShaderNoMip(picname);
		(*core::ptr::addr_of!(ui)).R_DrawStretchPic(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
	}
}


/*
================
UI_DrawHandlePic
=================
*/
#[allow(non_snake_case)]
pub fn UI_DrawHandlePic(x: c_float, y: c_float, w: c_float, h: c_float, hShader: qhandle_t) {
	let mut w = w;
	let mut h = h;
	let mut s0: c_float;
	let mut s1: c_float;
	let mut t0: c_float;
	let mut t1: c_float;

	if w < 0.0 {	// flip about horizontal
		w  = -w;
		s0 = 1.0;
		s1 = 0.0;
	}
	else {
		s0 = 0.0;
		s1 = 1.0;
	}

	if h < 0.0 {	// flip about vertical
		h  = -h;
		t0 = 1.0;
		t1 = 0.0;
	}
	else {
		t0 = 0.0;
		t1 = 1.0;
	}

	unsafe {
		(*core::ptr::addr_of!(ui)).R_DrawStretchPic(x, y, w, h, s0, t0, s1, t1, hShader);
	}
}

/*
================
UI_FillRect

Coordinates are 640*480 virtual values
=================
*/
#[allow(non_snake_case)]
pub fn UI_FillRect(x: c_float, y: c_float, width: c_float, height: c_float, color: *const c_float) {
	unsafe {
		(*core::ptr::addr_of!(ui)).R_SetColor(color);

		(*core::ptr::addr_of!(ui)).R_DrawStretchPic(x, y, width, height, 0.0, 0.0, 0.0, 0.0, (*core::ptr::addr_of!(uis)).whiteShader);

		(*core::ptr::addr_of!(ui)).R_SetColor(core::ptr::null());
	}
}

/*
=================
UI_UpdateScreen
=================
*/
#[allow(non_snake_case)]
pub fn UI_UpdateScreen() {
	unsafe {
		(*core::ptr::addr_of!(ui)).UpdateScreen();
	}
}


/*
===============
UI_LoadMenu_f
===============
*/
fn UI_LoadMenu_f() {
	unsafe {
		trap_Key_SetCatcher(KEYCATCH_UI);
		Menus_ActivateByName("ingameloadMenu\0".as_ptr() as *const c_char);
	}
}

/*
===============
UI_SaveMenu_f
===============
*/
fn UI_SaveMenu_f() {
	//	ui.PrecacheScreenshot();

	unsafe {
		trap_Key_SetCatcher(KEYCATCH_UI);
		Menus_ActivateByName("ingamesaveMenu\0".as_ptr() as *const c_char);
	}
}


//--------------------------------------------

/*
=================
UI_SetColor
=================
*/
#[allow(non_snake_case)]
pub fn UI_SetColor(rgba: *const c_float) {
	unsafe {
		trap_R_SetColor(rgba);
	}
}

/*int registeredFontCount = 0;
#define MAX_FONTS 6
static fontInfo_t registeredFont[MAX_FONTS];
*/

/*
=================
UI_RegisterFont
=================
*/

#[allow(non_snake_case)]
pub fn UI_RegisterFont(fontName: *const c_char) -> c_int {
	let mut iFontIndex: c_int = unsafe { (*core::ptr::addr_of!(ui)).R_RegisterFont(fontName) };
	if iFontIndex == 0 {
		iFontIndex = unsafe { (*core::ptr::addr_of!(ui)).R_RegisterFont("ergoec\0".as_ptr() as *const c_char) };	// fall back
	}

	iFontIndex
}
