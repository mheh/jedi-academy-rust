//Anything above this include will be ignored by the compiler

// console.c

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Stub type definitions for structures - actual definitions in oracle headers
#[repr(C)]
pub struct glconfig_t {
    pub vidWidth: c_int,
    pub vidHeight: c_int,
}

#[repr(C)]
pub struct console_t {
    pub text: [i16; 32768],
    pub current: c_int,
    pub totallines: c_int,
    pub linewidth: c_int,
    pub x: c_int,
    pub display: c_int,
    pub color: [f32; 4],
    pub xadjust: f32,
    pub yadjust: f32,
    pub times: [c_int; 4],
    pub vislines: c_int,
    pub displayFrac: f32,
    pub finalFrac: f32,
    pub initialized: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    pub value: f32,
}

#[repr(C)]
pub struct clientState_t {
    pub state: c_int,
    pub keyCatchers: c_int,
    pub glconfig: glconfig_t,
    pub realtime: c_int,
    pub realFrametime: c_int,
    pub consoleShader: c_int,
    pub whiteShader: c_int,
}

#[repr(C)]
pub struct renderInterface_t {
    pub SetColor: extern "C" fn(*const c_void),
    pub DrawStretchPic: extern "C" fn(c_int, c_int, c_int, c_int, f32, f32, f32, f32, c_int),
    pub Language_IsAsian: extern "C" fn() -> c_int,
    pub RegisterFont: extern "C" fn(*const c_char) -> c_int,
    pub Font_HeightPixels: extern "C" fn(c_int, f32) -> f32,
    pub Font_DrawString: extern "C" fn(c_int, c_int, *const c_char, *const c_void, c_int, c_int, f32),
}

// External function declarations
extern "C" {
    fn Field_Clear(field: *mut c_void);
    fn Con_ClearNotify();
    fn Con_Bottom();
    fn CL_StartDemoLoop();
    fn ColorIndex(color: c_int) -> c_int;
    fn Cmd_Argc() -> c_int;
    fn Cmd_Argv(argc: c_int) -> *const c_char;
    fn Cmd_AddCommand(name: *const c_char, func: extern "C" fn());
    fn Com_Printf(fmt: *const c_char, ...);
    fn FS_FOpenFileWrite(filename: *const c_char) -> c_int;
    fn FS_Write(buffer: *const c_void, len: c_int, f: c_int);
    fn FS_FCloseFile(f: c_int);
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn VM_Call(vm: *mut c_void, ...) -> c_int;
    fn Q_IsColorString(s: *const c_char) -> c_int;
    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize);
    fn SCR_DrawSmallChar(x: c_int, y: c_int, c: c_int);
    fn SCR_DrawBigString(x: c_int, y: c_int, s: *const c_char, scale: f32);
    fn SCR_DrawPic(x: c_int, y: c_int, w: c_int, h: f32, handle: c_int);
    fn SE_GetString(table: *const c_char, str: *const c_char) -> *const c_char;
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn Field_Draw(field: *mut c_void, x: c_int, y: c_int, width: c_int, showCursor: c_int);
    fn Field_BigDraw(field: *mut c_void, x: c_int, y: c_int, width: c_int, showCursor: c_int);
}

// External globals
extern "C" {
    pub static mut cls: clientState_t;
    pub static mut kg: c_void;
    pub static mut cgvm: *mut c_void;
    pub static mut chatField: c_void;
    pub static mut chat_playerNum: c_int;
    pub static mut chat_team: c_int;
    pub static mut cl_noprint: *const cvar_t;
    pub static mut cl: c_void;
    pub static mut cl_conXOffset: *mut cvar_t;
    pub static mut com_cl_running: *const cvar_t;
    pub static mut g_color_table: *const [f32; 4];
    pub static Q3_VERSION: *const c_char;
    pub static re: renderInterface_t;
}

pub static mut g_console_field_width: c_int = 78;

pub static mut con: console_t = console_t {
    text: [0; 32768],
    current: 0,
    totallines: 0,
    linewidth: 0,
    x: 0,
    display: 0,
    color: [0.0; 4],
    xadjust: 0.0,
    yadjust: 0.0,
    times: [0; 4],
    vislines: 0,
    displayFrac: 0.0,
    finalFrac: 0.0,
    initialized: 0,
};

pub static mut con_conspeed: *mut cvar_t = core::ptr::null_mut();
pub static mut con_notifytime: *mut cvar_t = core::ptr::null_mut();

const DEFAULT_CONSOLE_WIDTH: c_int = 78;

pub static mut console_color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// Constants
const CON_TEXTSIZE: c_int = 32768;
const NUM_CON_TIMES: c_int = 4;
const COLOR_WHITE: c_int = 7;
const COLOR_RED: c_int = 1;
const KEYCATCH_CONSOLE: c_int = 0x0001;
const KEYCATCH_MESSAGE: c_int = 0x0002;
const KEYCATCH_UI: c_int = 0x0004;
const KEYCATCH_CGAME: c_int = 0x0008;
const CA_DISCONNECTED: c_int = 0;
const CA_ACTIVE: c_int = 1;
const SMALLCHAR_WIDTH: c_int = 8;
const SMALLCHAR_HEIGHT: c_int = 8;
const SCREEN_WIDTH: c_int = 640;
const SCREEN_HEIGHT: c_int = 480;
const BIGCHAR_WIDTH: c_int = 16;
const BIGCHAR_HEIGHT: c_int = 16;
const PM_INTERMISSION: c_int = 4;
const MAX_CLIENTS: c_int = 64;
const CG_CROSSHAIR_PLAYER: c_int = 0;
const CG_LAST_ATTACKER: c_int = 1;
const COMMAND_HISTORY: c_int = 32;
const qfalse: c_int = 0;
const qtrue: c_int = 1;


/*
================
Con_ToggleConsole_f
================
*/
#[cfg(not(target_os = "xbox"))]
pub extern "C" fn Con_ToggleConsole_f () {
	// closing a full screen console restarts the demo loop
	unsafe {
		if cls.state == CA_DISCONNECTED && cls.keyCatchers == KEYCATCH_CONSOLE {
			CL_StartDemoLoop();
			return;
		}

		Field_Clear(addr_of_mut!(kg) as *mut c_void);
		// kg.g_consoleField.widthInChars = g_console_field_width;
	}

	unsafe { Con_ClearNotify (); }
	unsafe {
		cls.keyCatchers ^= KEYCATCH_CONSOLE;
	}
}

/*
================
Con_MessageMode_f
================
*/
pub extern "C" fn Con_MessageMode_f () {	//yell
	unsafe {
		chat_playerNum = -1;
		chat_team = qfalse;
		Field_Clear(addr_of_mut!(chatField) as *mut c_void);
		// chatField.widthInChars = 30;

		cls.keyCatchers ^= KEYCATCH_MESSAGE;
	}
}

/*
================
Con_MessageMode2_f
================
*/
pub extern "C" fn Con_MessageMode2_f () {	//team chat
	unsafe {
		chat_playerNum = -1;
		chat_team = qtrue;
		Field_Clear(addr_of_mut!(chatField) as *mut c_void);
		// chatField.widthInChars = 25;
		cls.keyCatchers ^= KEYCATCH_MESSAGE;
	}
}

/*
================
Con_MessageMode3_f
================
*/
pub extern "C" fn Con_MessageMode3_f ()
{		//target chat
	unsafe {
		if addr_of!(cgvm).is_null() {
			// assert(!"null cgvm");
			return;
		}

		chat_playerNum = VM_Call(cgvm, CG_CROSSHAIR_PLAYER);
		if chat_playerNum < 0 || chat_playerNum >= MAX_CLIENTS {
			chat_playerNum = -1;
			return;
		}
		chat_team = qfalse;
		Field_Clear(addr_of_mut!(chatField) as *mut c_void);
		// chatField.widthInChars = 30;
		cls.keyCatchers ^= KEYCATCH_MESSAGE;
	}
}

/*
================
Con_MessageMode4_f
================
*/
pub extern "C" fn Con_MessageMode4_f ()
{	//attacker
	unsafe {
		if addr_of!(cgvm).is_null() {
			// assert(!"null cgvm");
			return;
		}

		chat_playerNum = VM_Call(cgvm, CG_LAST_ATTACKER);
		if chat_playerNum < 0 || chat_playerNum >= MAX_CLIENTS {
			chat_playerNum = -1;
			return;
		}
		chat_team = qfalse;
		Field_Clear(addr_of_mut!(chatField) as *mut c_void);
		// chatField.widthInChars = 30;
		cls.keyCatchers ^= KEYCATCH_MESSAGE;
	}
}

/*
================
Con_Clear_f
================
*/
pub extern "C" fn Con_Clear_f () {
	let mut i: c_int = 0;

	while i < CON_TEXTSIZE {
		unsafe {
			con.text[i as usize] = ((ColorIndex(COLOR_WHITE))<<8) | (' ' as i16);
		}
		i += 1;
	}

	unsafe { Con_Bottom(); }		// go to end
}


/*
================
Con_Dump_f

Save the console contents out to a file
================
*/
pub extern "C" fn Con_Dump_f ()
{
	let mut l: c_int;
	let mut x: c_int;
	let mut i: c_int;
	let mut line: *mut i16;
	let mut f: c_int;
	let mut buffer: [c_char; 1024] = [0; 1024];

	unsafe {
		if Cmd_Argc() != 2
		{
			Com_Printf(SE_GetString(c"CON_TEXT_DUMP_USAGE".as_ptr() as *const c_char));
			return;
		}

		Com_Printf(c"Dumped console text to %s.\n".as_ptr() as *const c_char, Cmd_Argv(1));

		f = FS_FOpenFileWrite(Cmd_Argv(1));
		if f == 0
		{
			Com_Printf(c"ERROR: couldn't open.\n".as_ptr() as *const c_char);
			return;
		}

		// skip empty lines
		l = con.current - con.totallines + 1;
		while l <= con.current {
			line = con.text.as_mut_ptr().add(((l % con.totallines) * con.linewidth) as usize);
			x = 0;
			while x < con.linewidth {
				if *line.add(x as usize) & 0xff != (' ' as i16) {
					break;
				}
				x += 1;
			}
			if x != con.linewidth {
				break;
			}
			l += 1;
		}

		// write the remaining lines
		buffer[con.linewidth as usize] = 0;
		while l <= con.current {
			line = con.text.as_mut_ptr().add(((l % con.totallines) * con.linewidth) as usize);
			i = 0;
			while i < con.linewidth {
				buffer[i as usize] = (*line.add(i as usize) & 0xff) as c_char;
				i += 1;
			}
			x = con.linewidth - 1;
			while x >= 0 {
				if buffer[x as usize] == (' ' as c_char) {
					buffer[x as usize] = 0;
				} else {
					break;
				}
				x -= 1;
			}
			strcat(buffer.as_mut_ptr(), c"\n".as_ptr() as *const c_char);
			FS_Write(buffer.as_ptr() as *const c_void, strlen(buffer.as_ptr()) as c_int, f);
			l += 1;
		}

		FS_FCloseFile(f);
	}
}


/*
================
Con_ClearNotify
================
*/
pub extern "C" fn Con_ClearNotify( ) {
	let mut i: c_int = 0;

	while i < NUM_CON_TIMES {
		unsafe {
			con.times[i as usize] = 0;
		}
		i += 1;
	}
}



/*
================
Con_CheckResize

If the line width has changed, reformat the buffer.
================
*/
pub extern "C" fn Con_CheckResize ()
{
	let mut i: c_int;
	let mut j: c_int;
	let mut width: c_int;
	let mut oldwidth: c_int;
	let mut oldtotallines: c_int;
	let mut numlines: c_int;
	let mut numchars: c_int;
	let mut tbuf: [i16; 32768] = [0; 32768];

	unsafe {
//	width = (SCREEN_WIDTH / SMALLCHAR_WIDTH) - 2;
		width = (cls.glconfig.vidWidth / SMALLCHAR_WIDTH) - 2;

		if width == con.linewidth {
			return;
		}


		if width < 1 {			// video hasn't been initialized yet
			con.xadjust = 1.0;
			con.yadjust = 1.0;
			con.linewidth = width;
			con.totallines = CON_TEXTSIZE / width;
			i = 0;
			while i < CON_TEXTSIZE {
				con.text[i as usize] = ((ColorIndex(COLOR_WHITE))<<8) | (' ' as i16);
				i += 1;
			}
		}
		else
		{
			// on wide screens, we will center the text
			con.xadjust = 640.0 / cls.glconfig.vidWidth as f32;
			con.yadjust = 480.0 / cls.glconfig.vidHeight as f32;

			oldwidth = con.linewidth;
			con.linewidth = width;
			oldtotallines = con.totallines;
			con.totallines = CON_TEXTSIZE / width;
			numlines = oldtotallines;

			if con.totallines < numlines {
				numlines = con.totallines;
			}

			numchars = oldwidth;

			if con.linewidth < numchars {
				numchars = con.linewidth;
			}

			Com_Memcpy(tbuf.as_mut_ptr() as *mut c_void, con.text.as_mut_ptr() as *const c_void, (CON_TEXTSIZE * core::mem::size_of::<i16>()) as usize);
			i = 0;
			while i < CON_TEXTSIZE {
				con.text[i as usize] = ((ColorIndex(COLOR_WHITE))<<8) | (' ' as i16);
				i += 1;
			}


			i = 0;
			while i < numlines {
				j = 0;
				while j < numchars {
					con.text[((con.totallines - 1 - i) * con.linewidth + j) as usize] =
							tbuf[(((con.current - i + oldtotallines) %
								  oldtotallines) * oldwidth + j) as usize];
					j += 1;
				}
				i += 1;
			}

			Con_ClearNotify();
		}

		con.current = con.totallines - 1;
		con.display = con.current;
	}
}


/*
================
Con_Init
================
*/
pub extern "C" fn Con_Init () {
	let mut i: c_int;

	unsafe {
		con_notifytime = Cvar_Get(c"con_notifytime".as_ptr() as *const c_char, c"3".as_ptr() as *const c_char, 0);
		con_conspeed = Cvar_Get(c"scr_conspeed".as_ptr() as *const c_char, c"3".as_ptr() as *const c_char, 0);

		Field_Clear(addr_of_mut!(kg) as *mut c_void);
		// kg.g_consoleField.widthInChars = g_console_field_width;
		i = 0;
		while i < COMMAND_HISTORY {
			Field_Clear((addr_of_mut!(kg) as *mut c_void).add((i * core::mem::size_of::<c_void>()) as usize));
			// kg.historyEditLines[i].widthInChars = g_console_field_width;
			i += 1;
		}

		#[cfg(not(target_os = "xbox"))]
		{
			Cmd_AddCommand(c"toggleconsole".as_ptr() as *const c_char, Con_ToggleConsole_f);
		}
		Cmd_AddCommand(c"messagemode".as_ptr() as *const c_char, Con_MessageMode_f);
		Cmd_AddCommand(c"messagemode2".as_ptr() as *const c_char, Con_MessageMode2_f);
		Cmd_AddCommand(c"messagemode3".as_ptr() as *const c_char, Con_MessageMode3_f);
		Cmd_AddCommand(c"messagemode4".as_ptr() as *const c_char, Con_MessageMode4_f);
		Cmd_AddCommand(c"clear".as_ptr() as *const c_char, Con_Clear_f);
		Cmd_AddCommand(c"condump".as_ptr() as *const c_char, Con_Dump_f);

		//Initialize values on first print
		con.initialized = qfalse;
	}
}


/*
===============
Con_Linefeed
===============
*/
extern "C" fn Con_Linefeed (silent: c_int)
{
	let mut i: c_int;

	unsafe {
		// mark time for transparent overlay
		if con.current >= 0 && silent == 0 {
			con.times[(con.current % NUM_CON_TIMES) as usize] = cls.realtime;
		} else {
			con.times[(con.current % NUM_CON_TIMES) as usize] = 0;
		}

		con.x = 0;
		if con.display == con.current {
			con.display += 1;
		}
		con.current += 1;
		i = 0;
		while i < con.linewidth {
			con.text[((con.current % con.totallines) * con.linewidth + i) as usize] = ((ColorIndex(COLOR_WHITE))<<8) | (' ' as i16);
			i += 1;
		}
	}
}

/*
================
CL_ConsolePrint

Handles cursor positioning, line wrapping, etc
All console printing must go through this in order to be logged to disk
If no console is visible, the text will appear at the top of the game window
================
*/
pub extern "C" fn CL_ConsolePrint( txt: *const c_char, silent: c_int) {
	let mut y: c_int;
	let mut c: c_int;
	let mut l: c_int;
	let mut color: c_int;
	let mut txt_mut = txt;

	unsafe {
		// for some demos we don't want to ever show anything on the console
		if !addr_of!(cl_noprint).is_null() && (*cl_noprint).integer != 0 {
			return;
		}

		if con.initialized == 0 {
			con.color[0] = 1.0;
			con.color[1] = 1.0;
			con.color[2] = 1.0;
			con.color[3] = 1.0;
			con.linewidth = -1;
			Con_CheckResize();
			con.initialized = qtrue;
		}

		color = ColorIndex(COLOR_WHITE);

		while *txt_mut as u8 != 0 {
			c = *txt_mut as u8 as c_int;
			if Q_IsColorString(txt_mut as *const c_char) != 0 {
				color = ColorIndex(*txt_mut.add(1) as u8 as c_int);
				txt_mut = txt_mut.add(2);
				continue;
			}

			// count word length
			l = 0;
			while l < con.linewidth {
				if *txt_mut.add(l as usize) as u8 as c_int <= (' ' as c_int) {
					break;
				}
				l += 1;
			}

			// word wrap
			if l != con.linewidth && (con.x + l >= con.linewidth) {
				Con_Linefeed(silent);
			}

			txt_mut = txt_mut.add(1);

			match c as u8 {
				b'\n' => {
					Con_Linefeed(silent);
				},
				b'\r' => {
					con.x = 0;
				},
				_ => {	// display character and advance
					y = con.current % con.totallines;
					con.text[(y * con.linewidth + con.x) as usize] = ((color << 8) | c) as i16;
					con.x += 1;
					if con.x >= con.linewidth {
						Con_Linefeed(silent);
						con.x = 0;
					}
				}
			}
		}


		// mark time for transparent overlay
		if con.current >= 0 && silent == 0 {
			con.times[(con.current % NUM_CON_TIMES) as usize] = cls.realtime;
		} else {
			con.times[(con.current % NUM_CON_TIMES) as usize] = 0;
		}
	}
}


/*
==============================================================================

DRAWING

==============================================================================
*/


/*
================
Con_DrawInput

Draw the editline after a ] prompt
================
*/
pub extern "C" fn Con_DrawInput () {
	let mut y: c_int;

	unsafe {
		if cls.state != CA_DISCONNECTED && (cls.keyCatchers & KEYCATCH_CONSOLE) == 0 {
			return;
		}

		y = con.vislines - (SMALLCHAR_HEIGHT * (if re.Language_IsAsian() != 0 { 3 } else { 2 }) / 2);

		(re.SetColor)(con.color.as_ptr() as *const c_void);

		SCR_DrawSmallChar((con.xadjust + 1.0 * SMALLCHAR_WIDTH as f32) as c_int, y, ']' as c_int);

		Field_Draw(addr_of_mut!(kg) as *mut c_void, (con.xadjust + 2.0 * SMALLCHAR_WIDTH as f32) as c_int, y,
				SCREEN_WIDTH - 3 * SMALLCHAR_WIDTH, qtrue);
	}
}




/*
================
Con_DrawNotify

Draws the last few lines of output transparently over the game top
================
*/
pub extern "C" fn Con_DrawNotify ()
{
	let mut x: c_int;
	let mut v: c_int;
	let mut text: *mut i16;
	let mut i: c_int;
	let mut time: c_int;
	let mut skip: c_int;
	let mut currentColor: c_int;
	let mut chattext: *const c_char;

	unsafe {
		currentColor = 7;
		(re.SetColor)((*g_color_table).add(currentColor as usize) as *const c_void);

		v = 0;
		i = con.current - NUM_CON_TIMES + 1;
		while i <= con.current {
			if i < 0 {
				i += 1;
				continue;
			}
			time = con.times[(i % NUM_CON_TIMES) as usize];
			if time == 0 {
				i += 1;
				continue;
			}
			time = cls.realtime - time;
			if time > ((*con_notifytime).value * 1000.0) as c_int {
				i += 1;
				continue;
			}
			text = con.text.as_mut_ptr().add(((i % con.totallines) * con.linewidth) as usize);

			if *(addr_of!(cl) as *const c_int).add(5) != PM_INTERMISSION && (cls.keyCatchers & (KEYCATCH_UI | KEYCATCH_CGAME)) != 0 {
				i += 1;
				continue;
			}


			if addr_of!(cl_conXOffset).is_null() {
				cl_conXOffset = Cvar_Get(c"cl_conXOffset".as_ptr() as *const c_char, c"0".as_ptr() as *const c_char, 0);
			}

			// asian language needs to use the new font system to print glyphs...
			//
			// (ignore colours since we're going to print the whole thing as one string)
			//
			if re.Language_IsAsian() != 0 {
				static mut iFontIndex: c_int = 0;	// this seems naughty
				let fFontScale: f32 = 0.75 * con.yadjust;
				let iPixelHeightToAdvance: c_int = 2 + ((1.3 / con.yadjust) * (re.Font_HeightPixels)(iFontIndex, fFontScale)) as c_int;	// for asian spacing, since we don't want glyphs to touch.

				// concat the text to be printed...
				//
				let mut sTemp: [c_char; 4096] = [0; 4096];	// ott
				x = 0;
				while x < con.linewidth {
					if (((*text.add(x as usize) >> 8) & 7) as c_int) != currentColor {
						currentColor = ((*text.add(x as usize) >> 8) & 7) as c_int;
						strcat(sTemp.as_mut_ptr(), va(c"^%i".as_ptr() as *const c_char, ((*text.add(x as usize) >> 8) & 7) as c_int));
					}
					strcat(sTemp.as_mut_ptr(), va(c"%c".as_ptr() as *const c_char, *text.add(x as usize) & 0xFF));
					x += 1;
				}
				//
				// and print...
				//
				(re.Font_DrawString)((*cl_conXOffset).integer + (con.xadjust * (con.xadjust + (1.0 * SMALLCHAR_WIDTH as f32))) as c_int, (con.yadjust * v as f32) as c_int, sTemp.as_ptr(), (*g_color_table).add(currentColor as usize) as *const c_void, iFontIndex, -1, fFontScale);

				v += iPixelHeightToAdvance;
			} else {
				x = 0;
				while x < con.linewidth {
					if *text.add(x as usize) & 0xff == (' ' as i16) {
						x += 1;
						continue;
					}
					if (((*text.add(x as usize) >> 8) & 7) as c_int) != currentColor {
						currentColor = ((*text.add(x as usize) >> 8) & 7) as c_int;
						(re.SetColor)((*g_color_table).add(currentColor as usize) as *const c_void);
					}
					if addr_of!(cl_conXOffset).is_null() {
						cl_conXOffset = Cvar_Get(c"cl_conXOffset".as_ptr() as *const c_char, c"0".as_ptr() as *const c_char, 0);
					}
					SCR_DrawSmallChar((*cl_conXOffset).integer + (con.xadjust + (x as f32 + 1.0) * SMALLCHAR_WIDTH as f32) as c_int, v, (*text.add(x as usize) & 0xff) as c_int);
					x += 1;
				}

				v += SMALLCHAR_HEIGHT;
			}
			i += 1;
		}

		(re.SetColor)(core::ptr::null());

		if (cls.keyCatchers & (KEYCATCH_UI | KEYCATCH_CGAME)) != 0 {
			return;
		}

		// draw the chat line
		if (cls.keyCatchers & KEYCATCH_MESSAGE) != 0 {
			if chat_team != 0 {
				chattext = SE_GetString(c"MP_SVGAME".as_ptr() as *const c_char, c"SAY_TEAM".as_ptr() as *const c_char);
				SCR_DrawBigString(8, v, chattext, 1.0);
				skip = strlen(chattext) as c_int + 1;
			} else {
				chattext = SE_GetString(c"MP_SVGAME".as_ptr() as *const c_char, c"SAY".as_ptr() as *const c_char);
				SCR_DrawBigString(8, v, chattext, 1.0);
				skip = strlen(chattext) as c_int + 1;
			}

			Field_BigDraw(addr_of_mut!(chatField) as *mut c_void, skip * BIGCHAR_WIDTH, v,
				SCREEN_WIDTH - (skip + 1) * BIGCHAR_WIDTH, qtrue);

			v += BIGCHAR_HEIGHT;
		}
	}
}

/*
================
Con_DrawSolidConsole

Draws the console with the solid background
================
*/
pub extern "C" fn Con_DrawSolidConsole( frac: f32 ) {
	let mut i: c_int;
	let mut x: c_int;
	let mut y: c_int;
	let mut rows: c_int;
	let mut text: *mut i16;
	let mut row: c_int;
	let mut lines: c_int;
//	qhandle_t		conShader;
	let mut currentColor: c_int;

	unsafe {
		lines = (cls.glconfig.vidHeight as f32 * frac) as c_int;
		if lines <= 0 {
			return;
		}

		if lines > cls.glconfig.vidHeight {
			lines = cls.glconfig.vidHeight;
		}

		// draw the background
		y = (frac * SCREEN_HEIGHT as f32 - 2.0) as c_int;
		if y < 1 {
			y = 0;
		} else {
			SCR_DrawPic(0, 0, SCREEN_WIDTH, y as f32, cls.consoleShader);
		}

		let color: [f32; 4] = [0.509, 0.609, 0.847, 1.0];
		// draw the bottom bar and version number

		(re.SetColor)(color.as_ptr() as *const c_void);
		(re.DrawStretchPic)(0, y, SCREEN_WIDTH, 2, 0.0, 0.0, 0.0, 0.0, cls.whiteShader);

		i = strlen(Q3_VERSION) as c_int;

		x = 0;
		while x < i {
			SCR_DrawSmallChar(cls.glconfig.vidWidth - (i - x) * SMALLCHAR_WIDTH,
				(lines - (SMALLCHAR_HEIGHT + SMALLCHAR_HEIGHT / 2)), *Q3_VERSION.add(x as usize) as c_int);
			x += 1;
		}


		// draw the text
		con.vislines = lines;
		rows = (lines - SMALLCHAR_WIDTH) / SMALLCHAR_WIDTH;		// rows of text to draw

		y = lines - (SMALLCHAR_HEIGHT * 3);

		// draw from the bottom up
		if con.display != con.current {
		// draw arrows to show the buffer is backscrolled
			(re.SetColor)((*g_color_table).add(ColorIndex(COLOR_RED) as usize) as *const c_void);
			x = 0;
			while x < con.linewidth {
				SCR_DrawSmallChar((con.xadjust + (x as f32 + 1.0) * SMALLCHAR_WIDTH as f32) as c_int, y, '^' as c_int);
				x += 4;
			}
			y -= SMALLCHAR_HEIGHT;
			rows -= 1;
		}

		row = con.display;

		if con.x == 0 {
			row -= 1;
		}

		currentColor = 7;
		(re.SetColor)((*g_color_table).add(currentColor as usize) as *const c_void);

		static mut iFontIndexForAsian: c_int = 0;
		let fFontScaleForAsian: f32 = 0.75 * con.yadjust;
		let mut iPixelHeightToAdvance: c_int = SMALLCHAR_HEIGHT;
		if re.Language_IsAsian() != 0 {
			if iFontIndexForAsian == 0 {
				iFontIndexForAsian = (re.RegisterFont)(c"ocr_a".as_ptr() as *const c_char);
			}
			iPixelHeightToAdvance = ((1.3 / con.yadjust) * (re.Font_HeightPixels)(iFontIndexForAsian, fFontScaleForAsian)) as c_int;	// for asian spacing, since we don't want glyphs to touch.
		}

		i = 0;
		while i < rows {
			y -= iPixelHeightToAdvance;
			row -= 1;
			if row < 0 {
				break;
			}
			if con.current - row >= con.totallines {
				// past scrollback wrap point
				i += 1;
				continue;
			}

			text = con.text.as_mut_ptr().add(((row % con.totallines) * con.linewidth) as usize);

			// asian language needs to use the new font system to print glyphs...
			//
			// (ignore colours since we're going to print the whole thing as one string)
			//
			if re.Language_IsAsian() != 0 {
				// concat the text to be printed...
				//
				let mut sTemp: [c_char; 4096] = [0; 4096];	// ott
				x = 0;
				while x < con.linewidth {
					if (((*text.add(x as usize) >> 8) & 7) as c_int) != currentColor {
						currentColor = ((*text.add(x as usize) >> 8) & 7) as c_int;
						strcat(sTemp.as_mut_ptr(), va(c"^%i".as_ptr() as *const c_char, ((*text.add(x as usize) >> 8) & 7) as c_int));
					}
					strcat(sTemp.as_mut_ptr(), va(c"%c".as_ptr() as *const c_char, *text.add(x as usize) & 0xFF));
					x += 1;
				}
				//
				// and print...
				//
				(re.Font_DrawString)((con.xadjust * (con.xadjust + (1.0 * SMALLCHAR_WIDTH as f32))) as c_int, (con.yadjust * y as f32) as c_int, sTemp.as_ptr(), (*g_color_table).add(currentColor as usize) as *const c_void, iFontIndexForAsian, -1, fFontScaleForAsian);
			} else {
				x = 0;
				while x < con.linewidth {
					if *text.add(x as usize) & 0xff == (' ' as i16) {
						x += 1;
						continue;
					}

					if (((*text.add(x as usize) >> 8) & 7) as c_int) != currentColor {
						currentColor = ((*text.add(x as usize) >> 8) & 7) as c_int;
						(re.SetColor)((*g_color_table).add(currentColor as usize) as *const c_void);
					}
					SCR_DrawSmallChar((con.xadjust + (x as f32 + 1.0) * SMALLCHAR_WIDTH as f32) as c_int, y, (*text.add(x as usize) & 0xff) as c_int);
					x += 1;
				}
			}
			i += 1;
		}

		// draw the input prompt, user text, and cursor if desired
		Con_DrawInput();

		(re.SetColor)(core::ptr::null());
	}
}



/*
==================
Con_DrawConsole
==================
*/
pub extern "C" fn Con_DrawConsole( ) {
	unsafe {
		// check for console width changes from a vid mode change
		Con_CheckResize();

		// if disconnected, render console full screen
		if cls.state == CA_DISCONNECTED {
			if (cls.keyCatchers & (KEYCATCH_UI | KEYCATCH_CGAME)) == 0 {
				Con_DrawSolidConsole(1.0);
				return;
			}
		}

		if con.displayFrac != 0.0 {
			Con_DrawSolidConsole(con.displayFrac);
		} else {
			// draw notify lines
			if cls.state == CA_ACTIVE {
				Con_DrawNotify();
			}
		}
	}
}

//================================================================

/*
==================
Con_RunConsole

Scroll it up or down
==================
*/
pub extern "C" fn Con_RunConsole () {
	unsafe {
		// decide on the destination height of the console
		if (cls.keyCatchers & KEYCATCH_CONSOLE) != 0 {
			con.finalFrac = 0.5;		// half screen
		} else {
			con.finalFrac = 0.0;				// none visible
		}

		// scroll towards the destination height
		if con.finalFrac < con.displayFrac {
			con.displayFrac -= (*con_conspeed).value * (cls.realFrametime as f32 * 0.001);
			if con.finalFrac > con.displayFrac {
				con.displayFrac = con.finalFrac;
			}

		} else if con.finalFrac > con.displayFrac {
			con.displayFrac += (*con_conspeed).value * (cls.realFrametime as f32 * 0.001);
			if con.finalFrac < con.displayFrac {
				con.displayFrac = con.finalFrac;
			}
		}
	}
}


pub extern "C" fn Con_PageUp( ) {
	unsafe {
		con.display -= 2;
		if con.current - con.display >= con.totallines {
			con.display = con.current - con.totallines + 1;
		}
	}
}

pub extern "C" fn Con_PageDown( ) {
	unsafe {
		con.display += 2;
		if con.display > con.current {
			con.display = con.current;
		}
	}
}

pub extern "C" fn Con_Top( ) {
	unsafe {
		con.display = con.totallines;
		if con.current - con.display >= con.totallines {
			con.display = con.current - con.totallines + 1;
		}
	}
}

pub extern "C" fn Con_Bottom( ) {
	unsafe {
		con.display = con.current;
	}
}


pub extern "C" fn Con_Close( ) {
	unsafe {
		if !addr_of!(com_cl_running).is_null() && (*com_cl_running).integer == 0 {
			return;
		}
		Field_Clear(addr_of_mut!(kg) as *mut c_void);
		Con_ClearNotify();
		cls.keyCatchers &= !KEYCATCH_CONSOLE;
		con.finalFrac = 0.0;				// none visible
		con.displayFrac = 0.0;
	}
}
