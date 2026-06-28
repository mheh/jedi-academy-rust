// console.c

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_short, c_void};

use crate::code::client::client_h::*;
use crate::code::client::keys_h::*;

// Constants
const DEFAULT_CONSOLE_WIDTH: c_int = 78;

// Macros
fn Q_IsColorString(p: *const c_char) -> bool {
	unsafe {
		!p.is_null()
			&& *p == '^' as c_char
			&& !(*p.add(1)).is_null()
			&& *p.add(1) != '^' as c_char
			&& *p.add(1) <= '7' as c_char
			&& *p.add(1) >= '0' as c_char
	}
}

fn ColorIndex(c: c_int) -> c_int {
	((c - '0' as c_int) & 7)
}

fn MAKERGBA(v: &mut [f32; 4], r: f32, g: f32, b: f32, a: f32) {
	v[0] = r;
	v[1] = g;
	v[2] = b;
	v[3] = a;
}

// External C functions
extern "C" {
	pub fn Cmd_Argc() -> c_int;
	pub fn Cmd_Argv(i: c_int) -> *const c_char;
	pub fn Cmd_AddCommand(cmd_name: *const c_char, function: Option<extern "C" fn()>);

	pub fn Com_Printf(fmt: *const c_char, ...);
	pub fn SE_GetString(psPackageReference: *const c_char) -> *const c_char;

	pub fn FS_FOpenFileWrite(qpath: *const c_char) -> c_int;
	pub fn FS_Printf(f: c_int, fmt: *const c_char, ...);
	pub fn FS_FCloseFile(f: c_int);

	pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;

	pub fn va(fmt: *const c_char, ...) -> *const c_char;
	pub fn strlen(s: *const c_char) -> usize;
	pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
	pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// Stub types for external dependencies
#[repr(C)]
pub struct cvar_t {
	pub value: f32,
	// additional fields would be here
}

/*
================
Con_ToggleConsole_f
================
*/
extern "C" fn Con_ToggleConsole_f() {
	unsafe {
		// closing a full screen console restarts the demo loop
		if cls.state == 0 && cls.keyCatchers == 1 {
			//		CL_StartDemoLoop();
			return;
		}

		Field_Clear(&mut kg.g_consoleField);
		kg.g_consoleField.widthInChars = g_console_field_width;

		Con_ClearNotify();

		cls.keyCatchers ^= 1; // KEYCATCH_CONSOLE
	}
}

/*
================
Con_MessageMode_f
================
*/
extern "C" fn Con_MessageMode_f() {
	unsafe {
		Field_Clear(&mut chatField);
		chatField.widthInChars = 30;

		//	cls.keyCatchers ^= KEYCATCH_MESSAGE;
	}
}

/*
================
Con_Clear_f
================
*/
extern "C" fn Con_Clear_f() {
	unsafe {
		for i in 0..CON_TEXTSIZE {
			con.text[i] = ((ColorIndex('7' as c_int) as c_short) << 8) | ' ' as c_short;
		}

		Con_Bottom(); // go to end
	}
}

/*
================
Con_Dump_f

Save the console contents out to a file
================
*/
extern "C" fn Con_Dump_f() {
	#[cfg(not(target_os = "xbox"))]
	{
		unsafe {
			if Cmd_Argc() != 2 {
				Com_Printf(SE_GetString(b"CON_TEXT_DUMP_USAGE\0".as_ptr() as *const c_char));
				return;
			}

			Com_Printf(
				b"Dumped console text to %s.\n\0".as_ptr() as *const c_char,
				Cmd_Argv(1),
			);

			let f = FS_FOpenFileWrite(Cmd_Argv(1));
			if f == 0 {
				Com_Printf(b"^1ERROR: couldn\'t open dump file.\n\0".as_ptr() as *const c_char);
				return;
			}

			// skip empty lines
			let mut l = con.current - con.totallines + 1;
			while l <= con.current {
				let line = con.text.as_ptr().add((((l % con.totallines) * con.linewidth) as usize) as usize);
				let mut x = 0;
				while x < con.linewidth {
					if (*line.add(x as usize) & 0xff) != ' ' as c_short {
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
			let mut buffer: [c_char; 1024] = [0; 1024];
			buffer[con.linewidth as usize] = 0;
			while l <= con.current {
				let line = con.text.as_ptr().add((((l % con.totallines) * con.linewidth) as usize) as usize);
				let mut i = 0;
				while i < con.linewidth {
					buffer[i as usize] = (*line.add(i as usize) & 0xff) as c_char;
					i += 1;
				}
				let mut x = con.linewidth - 1;
				while x >= 0 {
					if buffer[x as usize] == ' ' as c_char {
						buffer[x as usize] = 0;
					} else {
						break;
					}
					x -= 1;
				}

				FS_Printf(f, b"%s\n\0".as_ptr() as *const c_char, buffer.as_ptr());
				l += 1;
			}

			FS_FCloseFile(f);
		}
	}
}

/*
================
Con_ClearNotify
================
*/
extern "C" fn Con_ClearNotify() {
	unsafe {
		for i in 0..NUM_CON_TIMES {
			con.times[i] = 0;
		}
	}
}

/*
================
Con_CheckResize

If the line width has changed, reformat the buffer.
================
*/
extern "C" fn Con_CheckResize() {
	unsafe {
		let mut tbuf: [c_short; CON_TEXTSIZE] = [0; CON_TEXTSIZE];

		//width = (SCREEN_WIDTH / SMALLCHAR_WIDTH) - 2;
		let mut width = (cls.glconfig.vidWidth / 8) - 2;

		if width == con.linewidth {
			return;
		}

		if width < 1 {
			// video hasn't been initialized yet
			con.xadjust = 1.0f32;
			con.yadjust = 1.0f32;
			width = DEFAULT_CONSOLE_WIDTH;
			con.linewidth = width;
			con.totallines = CON_TEXTSIZE as c_int / con.linewidth;
			for i in 0..CON_TEXTSIZE {
				con.text[i] = ((ColorIndex('7' as c_int) as c_short) << 8) | ' ' as c_short;
			}
		} else {
			// on wide screens, we will center the text
			con.xadjust = 640.0f32 / cls.glconfig.vidWidth as f32;
			con.yadjust = 480.0f32 / cls.glconfig.vidHeight as f32;

			let oldwidth = con.linewidth;
			con.linewidth = width;
			let oldtotallines = con.totallines;
			con.totallines = CON_TEXTSIZE as c_int / con.linewidth;
			let mut numlines = oldtotallines;

			if con.totallines < numlines {
				numlines = con.totallines;
			}

			let mut numchars = oldwidth;

			if con.linewidth < numchars {
				numchars = con.linewidth;
			}

			memcpy(
				tbuf.as_mut_ptr() as *mut c_void,
				con.text.as_ptr() as *const c_void,
				CON_TEXTSIZE * core::mem::size_of::<c_short>(),
			);
			for i in 0..CON_TEXTSIZE {
				con.text[i] = ((ColorIndex('7' as c_int) as c_short) << 8) | ' ' as c_short;
			}

			for i in 0..numlines as usize {
				for j in 0..numchars as usize {
					con.text[((con.totallines - 1 - i as c_int) * con.linewidth + j as c_int) as usize] = tbuf
						[(((con.current - i as c_int + oldtotallines) % oldtotallines) * oldwidth + j as c_int)
							as usize];
				}
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
extern "C" fn Con_Init() {
	unsafe {
		con_notifytime = Cvar_Get(b"con_notifytime\0".as_ptr() as *const c_char, b"3\0".as_ptr() as *const c_char, 0);
		con_conspeed = Cvar_Get(b"scr_conspeed\0".as_ptr() as *const c_char, b"3\0".as_ptr() as *const c_char, 0);
		con_conAlpha = Cvar_Get(
			b"conAlpha\0".as_ptr() as *const c_char,
			b"1.6\0".as_ptr() as *const c_char,
			8, // CVAR_ARCHIVE
		);

		Field_Clear(&mut kg.g_consoleField);
		kg.g_consoleField.widthInChars = g_console_field_width;
		for i in 0..COMMAND_HISTORY {
			Field_Clear(&mut kg.historyEditLines[i]);
			kg.historyEditLines[i].widthInChars = g_console_field_width;
		}

		Cmd_AddCommand(b"toggleconsole\0".as_ptr() as *const c_char, Some(Con_ToggleConsole_f));
		Cmd_AddCommand(b"messagemode\0".as_ptr() as *const c_char, Some(Con_MessageMode_f));
		Cmd_AddCommand(b"clear\0".as_ptr() as *const c_char, Some(Con_Clear_f));
		Cmd_AddCommand(b"condump\0".as_ptr() as *const c_char, Some(Con_Dump_f));
	}
}

/*
===============
Con_Linefeed
===============
*/
extern "C" fn Con_Linefeed() {
	unsafe {
		// mark time for transparent overlay
		if con.current >= 0 {
			con.times[(con.current % NUM_CON_TIMES as c_int) as usize] = cls.realtime;
		}

		con.x = 0;
		if con.display == con.current {
			con.display += 1;
		}
		con.current += 1;
		for i in 0..con.linewidth as usize {
			con.text[((con.current % con.totallines) * con.linewidth + i as c_int) as usize] =
				((ColorIndex('7' as c_int) as c_short) << 8) | ' ' as c_short;
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
extern "C" fn CL_ConsolePrint(txt: *mut c_char) {
	unsafe {
		let mut txt = txt;
		let mut y: c_int;
		let mut c: c_int;
		let mut l: c_int;
		let mut color: c_int;

		// for some demos we don't want to ever show anything on the console
		if !cl_noprint.is_null() && (*cl_noprint).integer != 0 {
			return;
		}

		if con.initialized == 0 {
			con.color[0] = 1.0f32;
			con.color[1] = 1.0f32;
			con.color[2] = 1.0f32;
			con.color[3] = 1.0f32;
			con.linewidth = -1;
			Con_CheckResize();
			con.initialized = 1;
		}

		color = ColorIndex('7' as c_int);

		while (*txt as u8) != 0 {
			c = *txt as u8 as c_int;
			if Q_IsColorString(txt) {
				color = ColorIndex(*txt.add(1) as c_int);
				txt = txt.add(2);
				continue;
			}

			// count word length
			l = 0;
			while l < con.linewidth {
				if *txt.add(l as usize) as u8 <= b' ' {
					break;
				}
				l += 1;
			}

			// word wrap
			if l != con.linewidth && (con.x + l >= con.linewidth) {
				Con_Linefeed();
			}

			txt = txt.add(1);

			match c {
				b'\n' as c_int => {
					Con_Linefeed();
				}
				b'\r' as c_int => {
					con.x = 0;
				}
				_ => {
					// display character and advance
					y = con.current % con.totallines;
					con.text[(y * con.linewidth + con.x) as usize] =
						((color as c_short) << 8) | (c as c_short);
					con.x += 1;
					if con.x >= con.linewidth {
						Con_Linefeed();
						con.x = 0;
					}
				}
			}
		}

		// mark time for transparent overlay

		if con.current >= 0 {
			con.times[(con.current % NUM_CON_TIMES as c_int) as usize] = cls.realtime;
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
extern "C" fn Con_DrawInput() {
	unsafe {
		if cls.state != 0 && (cls.keyCatchers & 1) == 0 {
			// CA_DISCONNECTED == 0, KEYCATCH_CONSOLE == 1
			return;
		}

		let y = con.vislines - (16 * (if re.Language_IsAsian() != 0 { 3 } else { 4 }) / 2);

		re.SetColor(con.color.as_ptr());

		SCR_DrawSmallChar(con.xadjust as c_int + 1 * 8, y, ']' as c_int); // SMALLCHAR_WIDTH = 8

		Field_Draw(
			&mut kg.g_consoleField,
			con.xadjust as c_int + 2 * 8,
			y,
			640 - 3 * 8, // SCREEN_WIDTH - 3 * SMALLCHAR_WIDTH
			1, // qtrue
		);
	}
}

/*
================
Con_DrawNotify

Draws the last few lines of output transparently over the game top
================
*/
extern "C" fn Con_DrawNotify() {
	unsafe {
		let mut x: c_int;
		let mut v: c_int = 0;
		let mut text: *mut c_short;
		let mut i: c_int;
		let mut time: c_int;
		let mut skip: c_int;
		let mut currentColor: c_int = 7;

		re.SetColor(g_color_table[currentColor as usize].as_ptr() as *mut f32);

		v = 0;
		i = con.current - NUM_CON_TIMES as c_int + 1;
		while i <= con.current {
			if i < 0 {
				i += 1;
				continue;
			}
			time = con.times[(i % NUM_CON_TIMES as c_int) as usize];
			if time == 0 {
				i += 1;
				continue;
			}
			time = cls.realtime - time;
			if time > ((*con_notifytime).value * 1000.0f32) as c_int {
				i += 1;
				continue;
			}
			text = con.text.as_mut_ptr().add((((i % con.totallines) * con.linewidth) as usize) as usize);

			// asian language needs to use the new font system to print glyphs...
			//
			// (ignore colours since we're going to print the whole thing as one string)
			//
			if re.Language_IsAsian() != 0 {
				let iFontIndex = re.RegisterFont(b"ocr_a\0".as_ptr() as *const c_char); // this seems naughty
				let fFontScale = 0.75f32 * con.yadjust;
				let iPixelHeightToAdvance = 2 + ((1.3f32 / con.yadjust) * re.Font_HeightPixels(iFontIndex, fFontScale)) as c_int; // for asian spacing, since we don't want glyphs to touch.

				// concat the text to be printed...
				//
				let mut sTemp: [c_char; 4096] = [0; 4096]; // ott
				x = 0;
				while x < con.linewidth {
					if (((*text.add(x as usize) >> 8) & 7) as c_int) != currentColor {
						currentColor = ((*text.add(x as usize) >> 8) & 7) as c_int;
						strcat(sTemp.as_mut_ptr(), va(b"^%i\0".as_ptr() as *const c_char, ((*text.add(x as usize) >> 8) & 7) as c_int));
					}
					strcat(sTemp.as_mut_ptr(), va(b"%c\0".as_ptr() as *const c_char, (*text.add(x as usize) & 0xff) as c_int));
					x += 1;
				}
				//
				// and print...
				//
				re.Font_DrawString(
					(con.xadjust * (con.xadjust + (1 * 8/*aesthetics*/)) as f32) as c_int, /* con.xadjust*(con.xadjust + (1*SMALLCHAR_WIDTH/*aesthetics*/)) */
					(con.yadjust * v as f32) as c_int, /* con.yadjust*(v) */
					sTemp.as_ptr(),
					g_color_table[currentColor as usize].as_ptr() as *mut f32,
					iFontIndex,
					-1,
					fFontScale,
				);

				v += iPixelHeightToAdvance;
			} else {
				x = 0;
				while x < con.linewidth {
					if ((*text.add(x as usize) & 0xff) as u8) == b' ' {
						x += 1;
						continue;
					}
					if (((*text.add(x as usize) >> 8) & 7) as c_int) != currentColor {
						currentColor = ((*text.add(x as usize) >> 8) & 7) as c_int;
						re.SetColor(g_color_table[currentColor as usize].as_ptr() as *mut f32);
					}
					SCR_DrawSmallChar(con.xadjust as c_int + (x + 1) * 8, v, (*text.add(x as usize) & 0xff) as c_int);
					x += 1;
				}
				v += 16; // SMALLCHAR_HEIGHT
			}
			i += 1;
		}

		re.SetColor(core::ptr::null_mut());

		// draw the chat line
		if (cls.keyCatchers & 2) != 0 {
			// KEYCATCH_MESSAGE == 2
			SCR_DrawBigString(8, v, b"say:\0".as_ptr() as *const c_char, 1.0f32);
			skip = 5;

			Field_BigDraw(&mut chatField, skip * 16, v, 640 - (skip + 1) * 16, 1); // SCREEN_WIDTH - ( skip + 1 ) * BIGCHAR_WIDTH, qtrue
		}
	}
}

/*
================
Con_DrawSolidConsole

Draws the console with the solid background
================
*/
extern "C" fn Con_DrawSolidConsole(frac: f32) {
	unsafe {
		let mut i: c_int;
		let mut x: c_int;
		let mut y: c_int;
		let mut rows: c_int;
		let mut text: *mut c_short;
		let mut row: c_int;
		let mut lines: c_int = (cls.glconfig.vidHeight as f32 * frac) as c_int;
		let mut currentColor: c_int;

		if lines <= 0 {
			return;
		}

		if lines > cls.glconfig.vidHeight {
			lines = cls.glconfig.vidHeight;
		}

		// draw the background
		y = (frac * 480.0f32) as c_int - 2; // SCREEN_HEIGHT
		if y < 1 {
			y = 0;
		} else {
			// draw the background only if fullscreen
			if frac != 1.0f32 {
				let mut con_color: [f32; 4] = [0.0f32; 4];
				MAKERGBA(&mut con_color, 0.0f32, 0.0f32, 0.0f32, frac * (*con_conAlpha).value);
				re.SetColor(con_color.as_ptr() as *mut f32);
			} else {
				re.SetColor(core::ptr::null_mut());
			}
			SCR_DrawPic(0.0f32, 0.0f32, 640.0f32, y as f32, cls.consoleShader); // SCREEN_WIDTH
		}

		let color: [f32; 4] = [0.509f32, 0.609f32, 0.847f32, 1.0f32];
		// draw the bottom bar and version number

		re.SetColor(color.as_ptr() as *mut f32);
		re.DrawStretchPic(0.0f32, y as f32, 640.0f32, 2.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, cls.whiteShader); // SCREEN_WIDTH

		i = strlen(b"JA: v\0".as_ptr() as *const c_char) as c_int; // Q3_VERSION stub

		x = 0;
		while x < i {
			SCR_DrawSmallChar(
				cls.glconfig.vidWidth - ((i - x) * 8), // SMALLCHAR_WIDTH
				(lines - (16 + 8)), // SMALLCHAR_HEIGHT + SMALLCHAR_HEIGHT/2
				b"JA: v\0"
					.as_ptr()
					.cast::<c_char>()
					.add(x as usize)
					.read() as c_int,
			); // Q3_VERSION[x]
			x += 1;
		}

		// draw the text
		con.vislines = lines;
		rows = (lines - 8) / 8; // (lines-SMALLCHAR_WIDTH)/SMALLCHAR_WIDTH

		y = lines - (16 * 3); // SMALLCHAR_HEIGHT*3

		// draw from the bottom up
		if con.display != con.current {
			// draw arrows to show the buffer is backscrolled
			re.SetColor(g_color_table[ColorIndex('3' as c_int) as usize].as_ptr() as *mut f32); // COLOR_YELLOW
			x = 0;
			while x < con.linewidth {
				SCR_DrawSmallChar(con.xadjust as c_int + (x + 1) * 8, y, '^' as c_int); // SMALLCHAR_WIDTH
				x += 4;
			}
			y -= 16; // SMALLCHAR_HEIGHT
			rows -= 1;
		}

		row = con.display;

		if con.x == 0 {
			row -= 1;
		}

		currentColor = 7;
		re.SetColor(g_color_table[currentColor as usize].as_ptr() as *mut f32);

		let iFontIndexForAsian = 0; // kinda tacky, this just gets the first registered font, since Asian stuff ignores the contents anyway
		let fFontScaleForAsian = 0.75f32 * con.yadjust;
		let mut iPixelHeightToAdvance = 16; // SMALLCHAR_HEIGHT
		if re.Language_IsAsian() != 0 {
			let mut iFontIndexForAsian = iFontIndexForAsian;
			if iFontIndexForAsian == 0 {
				iFontIndexForAsian = re.RegisterFont(b"ocr_a\0".as_ptr() as *const c_char); // must be a font that's used elsewhere
			}
			iPixelHeightToAdvance = ((1.3f32 / con.yadjust) * re.Font_HeightPixels(iFontIndexForAsian, fFontScaleForAsian)) as c_int; // for asian spacing, since we don't want glyphs to touch.
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

			text = con.text.as_mut_ptr().add((((row % con.totallines) * con.linewidth) as usize) as usize);

			// asian language needs to use the new font system to print glyphs...
			//
			// (ignore colours since we're going to print the whole thing as one string)
			//
			if re.Language_IsAsian() != 0 {
				// concat the text to be printed...
				//
				let mut sTemp: [c_char; 4096] = [0; 4096]; // ott
				x = 0;
				while x < con.linewidth {
					if (((*text.add(x as usize) >> 8) & 7) as c_int) != currentColor {
						currentColor = ((*text.add(x as usize) >> 8) & 7) as c_int;
						strcat(sTemp.as_mut_ptr(), va(b"^%i\0".as_ptr() as *const c_char, ((*text.add(x as usize) >> 8) & 7) as c_int));
					}
					strcat(sTemp.as_mut_ptr(), va(b"%c\0".as_ptr() as *const c_char, (*text.add(x as usize) & 0xff) as c_int));
					x += 1;
				}
				//
				// and print...
				//
				re.Font_DrawString(
					(con.xadjust * (con.xadjust + (1 * 8/*(aesthetics)*/)) as f32) as c_int, /* con.xadjust*(con.xadjust + (1*SMALLCHAR_WIDTH/*(aesthetics)*/) */
					(con.yadjust * y as f32) as c_int, /* con.yadjust*(y) */
					sTemp.as_ptr(),
					g_color_table[currentColor as usize].as_ptr() as *mut f32,
					iFontIndexForAsian,
					-1,
					fFontScaleForAsian,
				);
			} else {
				x = 0;
				while x < con.linewidth {
					if ((*text.add(x as usize) & 0xff) as u8) == b' ' {
						x += 1;
						continue;
					}

					if (((*text.add(x as usize) >> 8) & 7) as c_int) != currentColor {
						currentColor = ((*text.add(x as usize) >> 8) & 7) as c_int;
						re.SetColor(g_color_table[currentColor as usize].as_ptr() as *mut f32);
					}
					SCR_DrawSmallChar(con.xadjust as c_int + (x + 1) * 8, y, (*text.add(x as usize) & 0xff) as c_int); // SMALLCHAR_WIDTH
					x += 1;
				}
			}
			i += 1;
		}

		// draw the input prompt, user text, and cursor if desired
		Con_DrawInput();

		re.SetColor(core::ptr::null_mut());
	}
}

/*
==================
Con_DrawConsole
==================
*/
extern "C" fn Con_DrawConsole() {
	unsafe {
		// check for console width changes from a vid mode change
		Con_CheckResize();

		// if disconnected, render console full screen
		if cls.state == 0 {
			// CA_DISCONNECTED == 0
			if (cls.keyCatchers & 4) == 0 {
				// KEYCATCH_UI == 4
				Con_DrawSolidConsole(1.0f32);
				return;
			}
		}

		if con.displayFrac != 0.0f32 {
			Con_DrawSolidConsole(con.displayFrac);
		} else {
			// draw notify lines
			if cls.state == 1 {
				// CA_ACTIVE == 1
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
extern "C" fn Con_RunConsole() {
	unsafe {
		// decide on the destination height of the console
		if (cls.keyCatchers & 1) != 0 {
			// KEYCATCH_CONSOLE
			con.finalFrac = 0.5f32; // half screen
		} else {
			con.finalFrac = 0.0f32; // none visible
		}

		// scroll towards the destination height
		if con.finalFrac < con.displayFrac {
			con.displayFrac -= (*con_conspeed).value * cls.realFrametime as f32 * 0.001f32;
			if con.finalFrac > con.displayFrac {
				con.displayFrac = con.finalFrac;
			}
		} else if con.finalFrac > con.displayFrac {
			con.displayFrac += (*con_conspeed).value * cls.realFrametime as f32 * 0.001f32;
			if con.finalFrac < con.displayFrac {
				con.displayFrac = con.finalFrac;
			}
		}
	}
}

extern "C" fn Con_PageUp() {
	unsafe {
		con.display -= 2;
		if con.current - con.display >= con.totallines {
			con.display = con.current - con.totallines + 1;
		}
	}
}

extern "C" fn Con_PageDown() {
	unsafe {
		con.display += 2;
		if con.display > con.current {
			con.display = con.current;
		}
	}
}

extern "C" fn Con_Top() {
	unsafe {
		con.display = con.totallines;
		if con.current - con.display >= con.totallines {
			con.display = con.current - con.totallines + 1;
		}
	}
}

extern "C" fn Con_Bottom() {
	unsafe {
		con.display = con.current;
	}
}

extern "C" fn Con_Close() {
	unsafe {
		Field_Clear(&mut kg.g_consoleField);
		Con_ClearNotify();
		cls.keyCatchers &= !1; // ~KEYCATCH_CONSOLE
		con.finalFrac = 0.0f32; // none visible
		con.displayFrac = 0.0f32;
	}
}
