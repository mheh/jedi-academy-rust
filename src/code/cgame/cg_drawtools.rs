
// this line must stay at top so the whole PCH thing works...
#![allow(non_snake_case)]

use crate::code::cgame::cg_headers_h::*;

// #include "cg_local.h"
use crate::code::cgame::cg_media_h::*;

/*
================
CG_DrawSides

Coords are virtual 640x480
================
*/
pub fn CG_DrawSides(x: f32, y: f32, w: f32, h: f32, size: f32) {
	//size *= cgs.screenXScale;
	unsafe {
		cgi_R_DrawStretchPic( x, y, size, h, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader );
		cgi_R_DrawStretchPic( x + w - size, y, size, h, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader );
	}
}

pub fn CG_DrawTopBottom(x: f32, y: f32, w: f32, h: f32, size: f32) {
	//size *= cgs.screenYScale;
	unsafe {
		cgi_R_DrawStretchPic( x, y, w, size, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader );
		cgi_R_DrawStretchPic( x, y + h - size, w, size, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader );
	}
}

/*
================
CG_DrawRect

Coordinates are 640*480 virtual values
=================
*/
pub fn CG_DrawRect( x: f32, y: f32, width: f32, height: f32, size: f32, color: *const f32 ) {
	unsafe {
		cgi_R_SetColor( color );

		CG_DrawTopBottom(x, y, width, height, size);
		CG_DrawSides(x, y, width, height, size);

		cgi_R_SetColor( core::ptr::null() );
	}
}

/*
================
CG_FillRect

Coordinates are 640*480 virtual values
=================
*/
pub fn CG_FillRect( x: f32, y: f32, width: f32, height: f32, color: *const f32 ) {
	unsafe {
		cgi_R_SetColor( color );
		cgi_R_DrawStretchPic( x, y, width, height, 0.0, 0.0, 0.0, 0.0, cgs.media.whiteShader);
		cgi_R_SetColor( core::ptr::null() );
	}
}


/*
================
CG_Scissor

Coordinates are 640*480 virtual values
=================
*/
pub fn CG_Scissor( x: f32, y: f32, width: f32, height: f32)
{

	unsafe {
		cgi_R_Scissor( x, y, width, height);
	}

}


/*
================
CG_DrawPic

Coordinates are 640*480 virtual values
A width of 0 will draw with the original image width
=================
*/
pub fn CG_DrawPic( x: f32, y: f32, width: f32, height: f32, hShader: qhandle_t ) {
	unsafe {
		cgi_R_DrawStretchPic( x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader );
	}
}

/*
================
CG_DrawPic2

Coordinates are 640*480 virtual values
A width of 0 will draw with the original image width
Can also specify the exact texture coordinates
=================
*/
pub fn CG_DrawPic2( x: f32, y: f32, width: f32, height: f32, s1: f32, t1: f32, s2: f32, t2: f32, hShader: qhandle_t )
{
	unsafe {
		cgi_R_DrawStretchPic( x, y, width, height, s1, t1, s2, t2, hShader );
	}
}

/*
================
CG_DrawRotatePic

Coordinates are 640*480 virtual values
A width of 0 will draw with the original image width
rotates around the upper right corner of the passed in point
=================
*/
pub fn CG_DrawRotatePic( x: f32, y: f32, width: f32, height: f32, angle: f32, hShader: qhandle_t ) {
	unsafe {
		cgi_R_DrawRotatePic( x, y, width, height, 0.0, 0.0, 1.0, 1.0, angle, hShader );
	}
}

/*
================
CG_DrawRotatePic2

Coordinates are 640*480 virtual values
A width of 0 will draw with the original image width
Actually rotates around the center point of the passed in coordinates
=================
*/
pub fn CG_DrawRotatePic2( x: f32, y: f32, width: f32, height: f32, angle: f32, hShader: qhandle_t ) {
	unsafe {
		cgi_R_DrawRotatePic2( x, y, width, height, 0.0, 0.0, 1.0, 1.0, angle, hShader );
	}
}

/*
===============
CG_DrawChar

Coordinates and size in 640*480 virtual screen size
===============
*/
pub fn CG_DrawChar( x: i32, y: i32, width: i32, height: i32, ch: i32 ) {
	let ch = ch & 255;

	if ch == ' ' as i32 {
		return;
	}

	let ax = x as f32;
	let ay = y as f32;
	let aw = width as f32;
	let ah = height as f32;

	let row = ch >> 4;
	let col = ch & 15;
/*
	frow = row*0.0625;
	fcol = col*0.0625;
	size = 0.0625;

	cgi_R_DrawStretchPic( ax, ay, aw, ah,
					   fcol, frow,
					   fcol + size, frow + size,
					   cgs.media.charsetShader );
*/

	let frow = row as f32 * 0.0625;
	let fcol = col as f32 * 0.0625;
	let size = 0.03125;
	let size2 = 0.0625;

	unsafe {
		cgi_R_DrawStretchPic( ax, ay, aw, ah, fcol, frow, fcol + size, frow + size2,
			cgs.media.charsetShader );
	}

}


/*
==================
CG_DrawStringExt

Draws a multi-colored string with a drop shadow, optionally forcing
to a fixed color.

Coordinates are at 640 by 480 virtual resolution
==================
*/
pub fn CG_DrawStringExt( x: i32, y: i32, string: *const u8, setColor: *const f32,
		forceColor: qboolean, shadow: qboolean, charWidth: i32, charHeight: i32 ) {
	let mut color: vec4_t = [0.0; 4];
	let mut s: *const u8;
	let mut xx: i32;

	// draw the drop shadow
	if shadow != 0 {
		color[0] = 0.0;
		color[1] = 0.0;
		color[2] = 0.0;
		unsafe {
			color[3] = *setColor.add(3);
			cgi_R_SetColor( color.as_ptr() );
		}
		s = string;
		xx = x;
		unsafe {
			while *s != 0 {
				if Q_IsColorString( s ) != 0 {
					s = s.add(2);
					continue;
				}
				CG_DrawChar( xx + 2, y + 2, charWidth, charHeight, *s as i32 );
				xx += charWidth;
				s = s.add(1);
			}
		}
	}

	// draw the colored text
	s = string;
	xx = x;
	unsafe {
		cgi_R_SetColor( setColor );
	}
	unsafe {
		while *s != 0 {
			if Q_IsColorString( s ) != 0 {
				if forceColor == 0 {
					let color_table_entry = &g_color_table[ColorIndex(*(s.add(1))) as usize];
					color[0] = color_table_entry[0];
					color[1] = color_table_entry[1];
					color[2] = color_table_entry[2];
					color[3] = *setColor.add(3);
					cgi_R_SetColor( color.as_ptr() );
				}
				s = s.add(2);
				continue;
			}
			CG_DrawChar( xx, y, charWidth, charHeight, *s as i32 );
			xx += charWidth;
			s = s.add(1);
		}
	}
	unsafe {
		cgi_R_SetColor( core::ptr::null() );
	}
}


pub fn CG_DrawSmallStringColor( x: i32, y: i32, s: *const u8, color: vec4_t ) {
	CG_DrawStringExt( x, y, s, color.as_ptr(), qtrue, qfalse, SMALLCHAR_WIDTH, SMALLCHAR_HEIGHT );
}

/*
=================
CG_DrawStrlen

Returns character count, skiping color escape codes
=================
*/
pub fn CG_DrawStrlen( str: *const u8 ) -> i32 {
	let mut s = str;
	let mut count = 0;

	unsafe {
		while *s != 0 {
			if Q_IsColorString( s ) != 0 {
				s = s.add(2);
			} else {
				count += 1;
				s = s.add(1);
			}
		}
	}

	return count;
}

/*
=============
CG_TileClearBox

This repeats a 64*64 tile graphic to fill the screen around a sized down
refresh window.
=============
*/
fn CG_TileClearBox( x: i32, y: i32, w: i32, h: i32, hShader: qhandle_t ) {
	let s1 = x as f32 / 64.0;
	let t1 = y as f32 / 64.0;
	let s2 = (x + w) as f32 / 64.0;
	let t2 = (y + h) as f32 / 64.0;
	unsafe {
		cgi_R_DrawStretchPic( x as f32, y as f32, w as f32, h as f32, s1, t1, s2, t2, hShader );
	}
}



/*
==============
CG_TileClear

Clear around a sized down screen
==============
*/
pub fn CG_TileClear( ) {
	unsafe {
		let w = cgs.glconfig.vidWidth;
		let h = cgs.glconfig.vidHeight;

		if cg.refdef.x == 0 && cg.refdef.y == 0 &&
			cg.refdef.width == w && cg.refdef.height == h {
			return;		// full screen rendering
		}

		let top = cg.refdef.y;
		let bottom = top + cg.refdef.height - 1;
		let left = cg.refdef.x;
		let right = left + cg.refdef.width - 1;

		// clear above view screen
		CG_TileClearBox( 0, 0, w, top, cgs.media.backTileShader );

		// clear below view screen
		CG_TileClearBox( 0, bottom, w, h - bottom, cgs.media.backTileShader );

		// clear left of view screen
		CG_TileClearBox( 0, top, left, bottom - top + 1, cgs.media.backTileShader );

		// clear right of view screen
		CG_TileClearBox( right, top, w - right, bottom - top + 1, cgs.media.backTileShader );
	}
}



/*
================
CG_FadeColor
================
*/
pub fn CG_FadeColor( startMsec: i32, totalMsec: i32 ) -> *const f32 {
	static mut color: vec4_t = [0.0; 4];

	if startMsec == 0 {
		return core::ptr::null();
	}

	unsafe {
		let t = cg.time - startMsec;

		if t >= totalMsec {
			return core::ptr::null();
		}

		// fade out
		if totalMsec - t < FADE_TIME {
			color[3] = (totalMsec - t) as f32 * 1.0 / FADE_TIME as f32;
		} else {
			color[3] = 1.0;
		}
		color[0] = 1.0;
		color[1] = 1.0;
		color[2] = 1.0;

		return color.as_ptr();
	}
}

/*
==============
CG_DrawNumField

Take x,y positions as if 640 x 480 and scales them to the proper resolution

==============
*/
pub fn CG_DrawNumField (x: i32, y: i32, width: i32, value: i32, charWidth: i32, charHeight: i32, style: i32, zeroFill: qboolean)
{
	let mut num: [u8; 16] = [0; 16];
	let mut ptr: *const u8;
	let mut l: i32;
	let mut frame: i32;

	if width < 1 {
		return;
	}

	// draw number string
	let width = if width > 5 { 5 } else { width };

	let mut value = value;
	match width {
	1 => {
		value = if value > 9 { 9 } else { value };
		value = if value < 0 { 0 } else { value };
	},
	2 => {
		value = if value > 99 { 99 } else { value };
		value = if value < -9 { -9 } else { value };
	},
	3 => {
		value = if value > 999 { 999 } else { value };
		value = if value < -99 { -99 } else { value };
	},
	4 => {
		value = if value > 9999 { 9999 } else { value };
		value = if value < -999 { -999 } else { value };
	},
	_ => {}
	}

	unsafe {
		Com_sprintf (num.as_mut_ptr() as *mut i8, core::mem::size_of_val(&num) as i32, b"%i\0".as_ptr() as *const i8, value);
	}
	l = unsafe { crate::q_string::strlen(num.as_ptr() as *const i8) } as i32;
	if l > width {
		l = width;
	}

	// FIXME: Might need to do something different for the chunky font??
	let xWidth = match style {
	NUM_FONT_SMALL => {
		charWidth
	},
	NUM_FONT_CHUNKY => {
		(charWidth as f32 / 1.2) as i32 + 2
	},
	_ | NUM_FONT_BIG => {
		(charWidth as f32 / 2.0) as i32 + 7
	}
	};

	let mut x = x;
	if zeroFill != 0 {
		for _i in 0..(width - l) {
			match style {
			NUM_FONT_SMALL => {
				unsafe {
					CG_DrawPic( x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.smallnumberShaders[0] );
				}
			},
			NUM_FONT_CHUNKY => {
				unsafe {
					CG_DrawPic( x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.chunkyNumberShaders[0] );
				}
			},
			_ | NUM_FONT_BIG => {
				unsafe {
					CG_DrawPic( x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.numberShaders[0] );
				}
			}
			}
			x += 2 + xWidth;
		}
	} else {
		x += 2 + xWidth * (width - l);
	}

	ptr = num.as_ptr();
	while unsafe { *ptr } != 0 && l > 0 {
		if unsafe { *ptr } as i32 == '-' as i32 {
			frame = STAT_MINUS;
		} else {
			frame = unsafe { *ptr } as i32 - '0' as i32;
		}

		match style {
		NUM_FONT_SMALL => {
			unsafe {
				CG_DrawPic( x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.smallnumberShaders[frame as usize] );
			}
			x += 1;	// For a one line gap
		},
		NUM_FONT_CHUNKY => {
			unsafe {
				CG_DrawPic( x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.chunkyNumberShaders[frame as usize] );
			}
		},
		_ | NUM_FONT_BIG => {
			unsafe {
				CG_DrawPic( x as f32, y as f32, charWidth as f32, charHeight as f32, cgs.media.numberShaders[frame as usize] );
			}
		}
		}

		x += xWidth;
		unsafe { ptr = ptr.add(1); }
		l -= 1;
	}

}

/*
=================
CG_DrawProportionalString
=================
*/
pub fn CG_DrawProportionalString( x: i32, y: i32, str: *const u8, style: i32, color: vec4_t )
{
	//assert(!style);//call this directly if you need style (OR it into the font handle)
	unsafe {
		cgi_R_Font_DrawString (x, y, str as *const i8, color.as_ptr(), cgs.media.qhFontMedium, -1, 1.0);
	}
}
