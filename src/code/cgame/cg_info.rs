// this line must stay at top so the whole PCH thing works...
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types,
         dead_code, unused_variables, unused_mut, unused_assignments,
         unused_imports, unused_unsafe, clippy::all)]

use crate::code::cgame::cg_headers_h::*;

//#include "cg_local.h"
use crate::code::cgame::cg_media_h::*;
use crate::code::game::objectives_h::*;

use core::ffi::{c_int, c_char, c_float, c_short};

// For printing objectives
static objectiveStartingYpos: c_short = 75;		// Y starting position for objective text
static objectiveStartingXpos: c_short = 60;		// X starting position for objective text
const objectiveTextBoxWidth: c_int = 500;		// Width (in pixels) of text box
const objectiveTextBoxHeight: c_int = 300;		// Height (in pixels) of text box

pub static mut showLoadPowersName: [*const c_char; 12] = [
    b"SP_INGAME_HEAL2\0".as_ptr() as *const c_char,
    b"SP_INGAME_JUMP2\0".as_ptr() as *const c_char,
    b"SP_INGAME_SPEED2\0".as_ptr() as *const c_char,
    b"SP_INGAME_PUSH2\0".as_ptr() as *const c_char,
    b"SP_INGAME_PULL2\0".as_ptr() as *const c_char,
    b"SP_INGAME_MINDTRICK2\0".as_ptr() as *const c_char,
    b"SP_INGAME_GRIP2\0".as_ptr() as *const c_char,
    b"SP_INGAME_LIGHTNING2\0".as_ptr() as *const c_char,
    b"SP_INGAME_SABER_THROW2\0".as_ptr() as *const c_char,
    b"SP_INGAME_SABER_OFFENSE2\0".as_ptr() as *const c_char,
    b"SP_INGAME_SABER_DEFENSE2\0".as_ptr() as *const c_char,
    core::ptr::null(),
];

const MAX_OBJ_GRAPHICS: usize = 4;
const OBJ_GRAPHIC_SIZE: c_int = 240;
pub static mut obj_graphics: [c_int; MAX_OBJ_GRAPHICS] = [0; MAX_OBJ_GRAPHICS];

// qboolean CG_ForcePower_Valid(int forceKnownBits, int index);
// (forward declaration — function is defined later in this file; no Rust forward decl needed)

// Symbols declared with explicit `extern` keyword inside function bodies in this C++ file:
extern "C" {
    // declared extern inside ObjectivePrint_Line:
    fn CG_DisplayBoxedText(
        iBoxX: c_int,
        iBoxY: c_int,
        iBoxWidth: c_int,
        iBoxHeight: c_int,
        psText: *const c_char,
        iFontHandle: c_int,
        fScale: c_float,
        v4Color: *const vec4_t,
    ) -> *const c_char;
    static mut giLinesOutput: c_int;
    static mut gfAdvanceHack: c_float;

    // declared extern inside CG_DrawInformation:
    static mut g_eSavedGameJustLoaded: SavedGameJustLoaded_e;
}

// int CG_WeaponCheck( int weaponIndex );
// (forward declared at file scope; external function not defined in this file)
extern "C" {
    fn CG_WeaponCheck(weaponIndex: c_int) -> c_int;
}

// C stdlib functions used in this file (from system includes, not game modules):
extern "C" {
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
    fn atoi(s: *const c_char) -> c_int;
    fn strtok(s: *mut c_char, delim: *const c_char) -> *mut c_char;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
}

/*
====================
ObjectivePrint_Line

Print a single mission objective
====================
*/
unsafe fn ObjectivePrint_Line(color: c_int, objectIndex: c_int, missionYcnt: &mut c_int) {
    let mut str_: *mut c_char;
    let mut strBegin: *mut c_char;
    let mut y: c_int = 0;
    let mut pixelLen: c_int;
    let mut charLen: c_int = 0;
    let mut i: c_int;
    const maxHoldText: usize = 1024;
    let mut holdText: [c_char; 1024] = [0; 1024];
    let mut finalText: [c_char; 2048] = [0; 2048];
    let mut graphic: qhandle_t = 0;

    let iYPixelsPerLine: c_int = cgi_R_Font_HeightPixels((*core::ptr::addr_of!(cgs)).media.qhFontMedium, 1.0f32);

    if ((*core::ptr::addr_of!(gi)).Cvar_VariableIntegerValue)(b"com_demo\0".as_ptr() as *const c_char) != 0 {
        cgi_SP_GetStringTextString(
            va(b"OBJECTIVES_DEMO_%s\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(objectiveTable))[objectIndex as usize].name),
            finalText.as_mut_ptr(),
            core::mem::size_of_val(&finalText),
        );
    } else {
        cgi_SP_GetStringTextString(
            va(b"OBJECTIVES_%s\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(objectiveTable))[objectIndex as usize].name),
            finalText.as_mut_ptr(),
            core::mem::size_of_val(&finalText),
        );
    }
    // A hack to be able to count prisoners
    if objectIndex == T2_RANCOR_OBJ5 {
        let mut value: [c_char; 64] = [0; 64];
        let currTotal: c_int;
        let minTotal: c_int;

        ((*core::ptr::addr_of!(gi)).Cvar_VariableStringBuffer)(b"ui_prisonerobj_currtotal\0".as_ptr() as *const c_char, value.as_mut_ptr(), core::mem::size_of_val(&value));
        currTotal = atoi(value.as_ptr());
        ((*core::ptr::addr_of!(gi)).Cvar_VariableStringBuffer)(b"ui_prisonerobj_maxtotal\0".as_ptr() as *const c_char, value.as_mut_ptr(), core::mem::size_of_val(&value));
        minTotal = atoi(value.as_ptr());

        sprintf(finalText.as_mut_ptr(), va(finalText.as_ptr(), currTotal, minTotal));
    }

    pixelLen = cgi_R_Font_StrLenPixels(finalText.as_ptr(), (*core::ptr::addr_of!(cgs)).media.qhFontMedium, 1.0f32);

    str_ = finalText.as_mut_ptr();

    if cgi_Language_IsAsian() != 0 {
        // this is execrable, and should NOT have had to've been done now, but...
        //
        // extern const char *CG_DisplayBoxedText(	int iBoxX, int iBoxY, int iBoxWidth, int iBoxHeight,
        //											const char *psText, int iFontHandle, float fScale,
        //											const vec4_t v4Color);
        // extern int giLinesOutput;
        // extern float gfAdvanceHack;

        gfAdvanceHack = 1.0f32;	// override internal vertical advance
        y = objectiveStartingYpos as c_int + (iYPixelsPerLine * *missionYcnt);

        // Advance line if a graphic has printed
        i = 0;
        while i < MAX_OBJ_GRAPHICS as c_int {
            if (*core::ptr::addr_of!(obj_graphics))[i as usize] != 0 {
                y += OBJ_GRAPHIC_SIZE + 4;
            }
            i += 1;
        }

        CG_DisplayBoxedText(
            objectiveStartingXpos as c_int,
            y,
            objectiveTextBoxWidth,
            objectiveTextBoxHeight,
            finalText.as_ptr(),	// int iBoxX, int iBoxY, int iBoxWidth, int iBoxHeight, const char *psText
            (*core::ptr::addr_of!(cgs)).media.qhFontMedium,		// int iFontHandle,
            1.0f32,						// float fScale,
            core::ptr::addr_of!(colorTable[color as usize]),			// const vec4_t v4Color
        );

        gfAdvanceHack = 0.0f32;	// restore
        *missionYcnt += giLinesOutput;
    } else {
        // western...
        //
        if pixelLen < objectiveTextBoxWidth {	// One shot - small enough to print entirely on one line
            y = objectiveStartingYpos as c_int + (iYPixelsPerLine * (*missionYcnt));

            cgi_R_Font_DrawString(
                objectiveStartingXpos as c_int,
                y,
                str_,
                colorTable[color as usize],
                (*core::ptr::addr_of!(cgs)).media.qhFontMedium,
                -1,
                1.0f32);

            *missionYcnt += 1;
        }
        // Text is too long, break into lines.
        else {
            let mut holdText2: [c_char; 2] = [0; 2];
            pixelLen = 0;
            charLen = 0;
            holdText2[1] = 0 as c_char; // NULL
            strBegin = str_;

            while *str_ != 0 {
                holdText2[0] = *str_;
                pixelLen += cgi_R_Font_StrLenPixels(holdText2.as_ptr(), (*core::ptr::addr_of!(cgs)).media.qhFontMedium, 1.0f32);

                pixelLen += 2; // For kerning
                charLen += 1;

                if pixelLen > objectiveTextBoxWidth {
                    //Reached max length of this line
                    //step back until we find a space
                    while (charLen > 10) && (*str_ != b' ' as c_char) {
                        str_ = str_.offset(-1);
                        charLen -= 1;
                    }

                    if *str_ == b' ' as c_char {
                        str_ = str_.offset(1);	// To get past space
                    }

                    assert!(charLen < maxHoldText as c_int);	// Too big?

                    Q_strncpyz(holdText.as_mut_ptr(), strBegin, charLen as usize);
                    holdText[charLen as usize] = 0 as c_char; // NULL
                    strBegin = str_;
                    pixelLen = 0;
                    charLen = 1;

                    y = objectiveStartingYpos as c_int + (iYPixelsPerLine * *missionYcnt);

                    CG_DrawProportionalString(
                        objectiveStartingXpos as c_int,
                        y,
                        holdText.as_ptr(),
                        CG_SMALLFONT,
                        colorTable[color as usize]);

                    *missionYcnt += 1;
                } else if *str_.offset(1) == 0 {	// NULL
                    charLen += 1;

                    assert!(charLen < maxHoldText as c_int);	// Too big?

                    y = objectiveStartingYpos as c_int + (iYPixelsPerLine * *missionYcnt);

                    Q_strncpyz(holdText.as_mut_ptr(), strBegin, charLen as usize);
                    CG_DrawProportionalString(
                        objectiveStartingXpos as c_int,
                        y,
                        holdText.as_ptr(),
                        CG_SMALLFONT,
                        colorTable[color as usize]);

                    *missionYcnt += 1;
                    break;
                }
                str_ = str_.offset(1);
            }
        }
    }

    if objectIndex == T3_BOUNTY_OBJ1 {
        y = objectiveStartingYpos as c_int + (iYPixelsPerLine * *missionYcnt);
        if (*core::ptr::addr_of!(obj_graphics))[1] != 0 {
            y += OBJ_GRAPHIC_SIZE + 4;
        }
        if (*core::ptr::addr_of!(obj_graphics))[2] != 0 {
            y += OBJ_GRAPHIC_SIZE + 4;
        }
        graphic = cgi_R_RegisterShaderNoMip(b"textures/system/viewscreen1\0".as_ptr() as *const c_char);
        CG_DrawPic(355, 50, OBJ_GRAPHIC_SIZE, OBJ_GRAPHIC_SIZE, graphic);
        (*core::ptr::addr_of_mut!(obj_graphics))[3] = qtrue;
    }
}

/*
====================
CG_DrawDataPadObjectives

Draw routine for the objective info screen of the data pad.
====================
*/
#[no_mangle]
pub unsafe extern "C" fn CG_DrawDataPadObjectives(cent: *const centity_t) {
    let mut i: c_int;
    let mut totalY: c_int;
    let iYPixelsPerLine: c_int = cgi_R_Font_HeightPixels((*core::ptr::addr_of!(cgs)).media.qhFontMedium, 1.0f32);

    let titleXPos: c_short = objectiveStartingXpos - 22;		// X starting position for title text
    let titleYPos: c_short = objectiveStartingYpos - 23;		// Y starting position for title text
    let graphic_size: c_short = 16;							// Size (width and height) of graphic used to show status of objective
    let graphicXpos: c_short = objectiveStartingXpos - graphic_size - 8;	// Amount of X to backup from text starting position
    let graphicYOffset: c_short = ((iYPixelsPerLine - graphic_size as c_int) / 2) as c_short;	// Amount of Y to raise graphic so it's in the center of the text line

    *core::ptr::addr_of_mut!(missionInfo_Updated) = qfalse;		// This will stop the text from flashing
    (*core::ptr::addr_of_mut!(cg)).missionInfoFlashTime = 0;

    // zero out objective graphics
    i = 0;
    while i < MAX_OBJ_GRAPHICS as c_int {
        (*core::ptr::addr_of_mut!(obj_graphics))[i as usize] = qfalse;
        i += 1;
    }

    // Title Text at the top
    let mut text: [c_char; 1024] = [0; 1024];
    cgi_SP_GetStringTextString(b"SP_INGAME_OBJECTIVES\0".as_ptr() as *const c_char, text.as_mut_ptr(), core::mem::size_of_val(&text));
    cgi_R_Font_DrawString(titleXPos as c_int, titleYPos as c_int, text.as_ptr(), colorTable[CT_TITLE as usize], (*core::ptr::addr_of!(cgs)).media.qhFontMedium, -1, 1.0f32);

    let mut missionYcnt: c_int = 0;

    // Print all active objectives
    i = 0;
    while i < MAX_OBJECTIVES {
        // Is there an objective to see?
        if (*(*(*cent).gent).client).sess.mission_objectives[i as usize].display != 0 {
            // Calculate the Y position
            totalY = objectiveStartingYpos as c_int + (iYPixelsPerLine * (missionYcnt)) + (iYPixelsPerLine / 2);

            //	Draw graphics that show if mission has been accomplished or not
            cgi_R_SetColor(colorTable[CT_BLUE3 as usize]);
            CG_DrawPic(graphicXpos as c_int, (totalY - graphicYOffset as c_int), graphic_size as c_int, graphic_size as c_int, (*core::ptr::addr_of!(cgs)).media.messageObjCircle);	// Circle in front
            if (*(*(*cent).gent).client).sess.mission_objectives[i as usize].status == OBJECTIVE_STAT_SUCCEEDED {
                CG_DrawPic(graphicXpos as c_int, (totalY - graphicYOffset as c_int), graphic_size as c_int, graphic_size as c_int, (*core::ptr::addr_of!(cgs)).media.messageLitOn);	// Center Dot
            }

            // Print current objective text
            ObjectivePrint_Line(CT_WHITE, i, &mut missionYcnt);
        }
        i += 1;
    }

    // No mission text?
    if missionYcnt == 0 {
        // Set the message a quarter of the way down and in the center of the text box
        let messageYPosition: c_int = objectiveStartingYpos as c_int + (objectiveTextBoxHeight / 4);

        cgi_SP_GetStringTextString(b"SP_INGAME_OBJNONE\0".as_ptr() as *const c_char, text.as_mut_ptr(), core::mem::size_of_val(&text));
        let messageXPosition: c_int = objectiveStartingXpos as c_int + (objectiveTextBoxWidth / 2) - (cgi_R_Font_StrLenPixels(text.as_ptr(), (*core::ptr::addr_of!(cgs)).media.qhFontMedium, 1.0f32) / 2);

        cgi_R_Font_DrawString(
            messageXPosition,
            messageYPosition,
            text.as_ptr(),
            colorTable[CT_WHITE as usize],
            (*core::ptr::addr_of!(cgs)).media.qhFontMedium,
            -1,
            1.0f32);
    }
}

// /*

//-------------------------------------------------------
// static void CG_DrawForceCount( const int force, int x, float *y, const int pad,qboolean *hasForcePowers )
// {
// 	char	s[MAX_STRING_CHARS];
// 	int		val, textColor;
// 	char	text[1024]={0};
//
// 	gi.Cvar_VariableStringBuffer( va("playerfplvl%d", force ),s, sizeof(s) );
//
// 	sscanf( s, "%d",&val );
//
// 	if ((val<1) || (val> NUM_FORCE_POWERS))
// 	{
// 		return;
// 	}
//
// 	textColor = CT_ICON_BLUE;
//
// 	// Draw title
// 	cgi_SP_GetStringTextString( showLoadPowersName[force], text, sizeof(text) );
// 	CG_DrawProportionalString( x, *y, text, CG_BIGFONT, colorTable[textColor] );
//
//
// 	// Draw icons
// 	cgi_R_SetColor( colorTable[CT_WHITE]);
// 	const int iconSize = 30;
// 	if ( val >= 0 )
// 	{
// 		x -= 10;	// Back up from title a little
//
// 		for ( int i = 0; i < val; i++ )
// 		{
// 			CG_DrawPic( x - iconSize - i * (iconSize + 10) , *y, iconSize, iconSize, force_icons[force] );
// 		}
// 	}
//
// 	*y += pad;
//
// 	*hasForcePowers = qtrue;
// }
//
//
// /*
// ====================
// CG_LoadScreen_PersonalInfo
// ====================
// */
// /*
// static void CG_LoadScreen_PersonalInfo(void)
// {
// 	float	x, y;
// 	int		pad = 25;
// 	char	text[1024]={0};
// 	qboolean	hasForcePowers;
//
// 	y = 65 + 30;
//
// 	pad = 28;
// 	x = 300;
// 	hasForcePowers=qfalse;
//
// 	CG_DrawForceCount( FP_HEAL, x, &y, pad,&hasForcePowers);
// 	CG_DrawForceCount( FP_LEVITATION, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_SPEED, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_PUSH, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_PULL, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_TELEPATHY, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_GRIP, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_LIGHTNING, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_SABERTHROW, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_SABER_OFFENSE, x, &y, pad,&hasForcePowers );
// 	CG_DrawForceCount( FP_SABER_DEFENSE, x, &y, pad,&hasForcePowers );
//
// 	if (hasForcePowers)
// 	{
// 		cgi_SP_GetStringTextString( "SP_INGAME_CURRENTFORCEPOWERS", text, sizeof(text) );
// 		CG_DrawProportionalString( 200, 65, text, CG_CENTER | CG_BIGFONT, colorTable[CT_WHITE] );
// 	}
// 	else
// 	{	//you are only totally empty on the very first map?
// //		cgi_SP_GetStringTextString( "SP_INGAME_NONE", text, sizeof(text) );
// //		CG_DrawProportionalString( 320, y+30, text, CG_CENTER | CG_BIGFONT, colorTable[CT_ICON_BLUE] );
// 		cgi_SP_GetStringTextString( "SP_INGAME_ALONGTIME", text, sizeof(text) );
// 		int w = cgi_R_Font_StrLenPixels(text,cgs.media.qhFontMedium, 1.5f);
// 		cgi_R_Font_DrawString((320)-(w/2), y+40, text,  colorTable[CT_ICON_BLUE], cgs.media.qhFontMedium, -1, 1.5f);
// 	}
//
// }
// */

unsafe fn CG_LoadBar() {
    const numticks: c_int = 9;
    const tickwidth: c_int = 40;
    const tickheight: c_int = 8;
    const tickpadx: c_int = 20;
    const tickpady: c_int = 12;
    const capwidth: c_int = 8;
    let barwidth: c_int = numticks * tickwidth + tickpadx * 2 + capwidth * 2;
    let barleft: c_int = (640 - barwidth) / 2;
    let barheight: c_int = tickheight + tickpady * 2;
    let bartop: c_int = 475 - barheight;
    let capleft: c_int = barleft + tickpadx;
    let tickleft: c_int = capleft + capwidth;
    let ticktop: c_int = bartop + tickpady;

    cgi_R_SetColor(colorTable[CT_WHITE as usize]);
    // Draw background
    CG_DrawPic(barleft, bartop, barwidth, barheight, (*core::ptr::addr_of!(cgs)).media.levelLoad);

    // Draw left cap (backwards)
    CG_DrawPic(tickleft, ticktop, -capwidth, tickheight, (*core::ptr::addr_of!(cgs)).media.loadTickCap);

    // Draw bar
    CG_DrawPic(tickleft, ticktop, tickwidth * (*core::ptr::addr_of!(cg)).loadLCARSStage, tickheight, (*core::ptr::addr_of!(cgs)).media.loadTick);

    // Draw right cap
    CG_DrawPic(tickleft + tickwidth * (*core::ptr::addr_of!(cg)).loadLCARSStage, ticktop, capwidth, tickheight, (*core::ptr::addr_of!(cgs)).media.loadTickCap);
}

// int CG_WeaponCheck( int weaponIndex );
// (declared above in extern "C" block)

// For printing load screen icons
const MAXLOADICONSPERROW: c_int = 8;		// Max icons displayed per row
const MAXLOADWEAPONS: c_int = 16;
const MAXLOADFORCEPOWERS: c_int = 12;
const MAXLOAD_FORCEICONSIZE: c_int = 40;	// Size of force power icons
const MAXLOAD_FORCEICONPAD: c_int = 12;	// Padding space between icons

unsafe fn CG_DrawLoadWeaponsPrintRow(itemName: *const c_char, weaponsBits: c_int, rowIconCnt: c_int, startIndex: c_int) -> c_int {
    let mut i: c_int;
    let mut endIndex: c_int = 0;
    let mut printedIconCnt: c_int = 0;
    let mut iconSize: c_int;
    let mut holdX: c_int;
    let mut x: c_int = 0;
    let mut y: c_int = 0;
    let mut pad: c_int;
    let mut yOffset: c_int = 0;
    let mut width: c_int = 0;
    let mut height: c_int = 0;
    let mut color: vec4_t = core::mem::zeroed();
    let mut background: qhandle_t = 0;

    if cgi_UI_GetMenuItemInfo(
        b"loadScreen\0".as_ptr() as *const c_char,
        itemName,
        &mut x,
        &mut y,
        &mut width,
        &mut height,
        color.as_mut_ptr(),
        &mut background) == 0
    {
        return 0;
    }

    cgi_R_SetColor(color.as_ptr());

    iconSize = 60;
    pad = 12;

    // calculate placement of weapon icons
    holdX = x + (width - ((iconSize * rowIconCnt) + (pad * (rowIconCnt - 1)))) / 2;

    i = startIndex;
    while i < MAXLOADWEAPONS {
        if (weaponsBits & (1 << i)) == 0 {	// Does he have this weapon?
            i += 1;
            continue;
        }

        if (*core::ptr::addr_of!(weaponData))[i as usize].weaponIcon[0] != 0 {
            let weaponInfo: *mut weaponInfo_t;
            CG_RegisterWeapon(i);
            weaponInfo = core::ptr::addr_of_mut!(cg_weapons[i as usize]);
            endIndex = i;

    // NOTE : during loading screen always show the have ammo icon
    //		if (!CG_WeaponCheck(i))
    //		{
    //			CG_DrawPic( holdX, y+yOffset, iconSize, iconSize, weaponInfo->weaponIconNoAmmo );
    //		}
    //		else
            {
                CG_DrawPic(holdX, y + yOffset, iconSize, iconSize, (*weaponInfo).weaponIcon);
            }

            printedIconCnt += 1;
            if printedIconCnt == MAXLOADICONSPERROW {
                break;
            }

            holdX += iconSize + pad;
        }
        i += 1;
    }

    endIndex
}

// Print weapons the player is carrying
// Two rows print if there are too many
unsafe fn CG_DrawLoadWeapons(weaponBits: c_int) {
    let mut i: c_int;
    let mut endIndex: c_int = 0;
    let mut iconCnt: c_int;
    let mut rowIconCnt: c_int;

    // count the number of weapons owned
    iconCnt = 0;
    i = 1;
    while i < MAXLOADWEAPONS {
        if weaponBits & (1 << i) != 0 {
            iconCnt += 1;
        }
        i += 1;
    }

    if iconCnt == 0 {	// If no weapons, don't display
        return;
    }

    // Single line of icons
    if iconCnt <= MAXLOADICONSPERROW {
        CG_DrawLoadWeaponsPrintRow(b"weaponicons_singlerow\0".as_ptr() as *const c_char, weaponBits, iconCnt, 0);
    }
    // Two lines of icons
    else {
        // Print top row
        endIndex = CG_DrawLoadWeaponsPrintRow(b"weaponicons_row1\0".as_ptr() as *const c_char, weaponBits, MAXLOADICONSPERROW, 0);

        // Print second row
        rowIconCnt = iconCnt - MAXLOADICONSPERROW;
        CG_DrawLoadWeaponsPrintRow(b"weaponicons_row2\0".as_ptr() as *const c_char, weaponBits, rowIconCnt, endIndex + 1);
    }

    cgi_R_SetColor(core::ptr::null());
}


unsafe fn CG_DrawLoadForcePrintRow(itemName: *const c_char, forceBits: c_int, rowIconCnt: c_int, startIndex: c_int) -> c_int {
    let mut i: c_int;
    let mut endIndex: c_int = 0;
    let mut printedIconCnt: c_int = 0;
    let mut holdX: c_int;
    let mut x: c_int = 0;
    let mut y: c_int = 0;
    let mut yOffset: c_int = 0;
    let mut width: c_int = 0;
    let mut height: c_int = 0;
    let mut color: vec4_t = core::mem::zeroed();
    let mut background: qhandle_t = 0;

    if cgi_UI_GetMenuItemInfo(
        b"loadScreen\0".as_ptr() as *const c_char,
        itemName,
        &mut x,
        &mut y,
        &mut width,
        &mut height,
        color.as_mut_ptr(),
        &mut background) == 0
    {
        return 0;
    }

    cgi_R_SetColor(color.as_ptr());

    // calculate placement of weapon icons
    holdX = x + (width - ((MAXLOAD_FORCEICONSIZE * rowIconCnt) + (MAXLOAD_FORCEICONPAD * (rowIconCnt - 1)))) / 2;

    i = startIndex;
    while i < MAX_SHOWPOWERS {
        if CG_ForcePower_Valid(forceBits, i) == 0 {	// Does he have this power?
            i += 1;
            continue;
        }

        if (*core::ptr::addr_of!(force_icons))[(*core::ptr::addr_of!(showPowers))[i as usize] as usize] != 0 {
            endIndex = i;

            CG_DrawPic(holdX, y + yOffset, MAXLOAD_FORCEICONSIZE, MAXLOAD_FORCEICONSIZE, (*core::ptr::addr_of!(force_icons))[(*core::ptr::addr_of!(showPowers))[i as usize] as usize]);

            printedIconCnt += 1;
            if printedIconCnt == MAXLOADICONSPERROW {
                break;
            }

            holdX += MAXLOAD_FORCEICONSIZE + MAXLOAD_FORCEICONPAD;
        }
        i += 1;
    }

    endIndex
}

pub static mut loadForcePowerLevel: [c_int; NUM_FORCE_POWERS as usize] = [0; NUM_FORCE_POWERS as usize];

/*
===============
ForcePowerDataPad_Valid
===============
*/
#[no_mangle]
pub unsafe extern "C" fn CG_ForcePower_Valid(forceKnownBits: c_int, index: c_int) -> qboolean {
    if (forceKnownBits & (1 << (*core::ptr::addr_of!(showPowers))[index as usize])) != 0
        && (*core::ptr::addr_of!(loadForcePowerLevel))[(*core::ptr::addr_of!(showPowers))[index as usize] as usize] != 0  // Does he have the force power?
    {
        return qtrue;
    }

    qfalse
}

// Print force powers the player is using
// Two rows print if there are too many
unsafe fn CG_DrawLoadForcePowers(forceBits: c_int) {
    let mut i: c_int;
    let mut endIndex: c_int = 0;
    let mut iconCnt: c_int = 0;
    let mut rowIconCnt: c_int;

    // Count the number of force powers known
    i = 0;
    while i < MAX_SHOWPOWERS {
        if CG_ForcePower_Valid(forceBits, i) != 0 {
            iconCnt += 1;
        }
        i += 1;
    }

    if iconCnt == 0 {	// If no force powers, don't display
        return;
    }

    // Single line of icons
    if iconCnt <= MAXLOADICONSPERROW {
        CG_DrawLoadForcePrintRow(b"forceicons_singlerow\0".as_ptr() as *const c_char, forceBits, iconCnt, 0);
    }
    // Two lines of icons
    else {
        // Print top row
        endIndex = CG_DrawLoadForcePrintRow(b"forceicons_row1\0".as_ptr() as *const c_char, forceBits, MAXLOADICONSPERROW, 0);

        // Print second row
        rowIconCnt = iconCnt - MAXLOADICONSPERROW;
        CG_DrawLoadForcePrintRow(b"forceicons_row2\0".as_ptr() as *const c_char, forceBits, rowIconCnt, endIndex + 1);
    }

    cgi_R_SetColor(core::ptr::null());
}

// Get the player weapons and force power info
unsafe fn CG_GetLoadScreenInfo(weaponBits: *mut c_int, forceBits: *mut c_int) {
    let mut s: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];
    let mut iDummy: c_int = 0;
    let mut i: c_int;
    let mut fDummy: c_float = 0.0;
    let mut var: *const c_char;

    ((*core::ptr::addr_of!(gi)).Cvar_VariableStringBuffer)(sCVARNAME_PLAYERSAVE, s.as_mut_ptr(), core::mem::size_of_val(&s));

    // Get player weapons and force powers known
    if s[0] != 0 {
        //				|general info				  |-force powers
        sscanf(s.as_ptr(), b"%i %i %i %i %i %i %i %f %f %f %i %i\0".as_ptr() as *const c_char,
                &mut iDummy,	//	&client->ps.stats[STAT_HEALTH],
                &mut iDummy,	//	&client->ps.stats[STAT_ARMOR],
                weaponBits,     //	&client->ps.stats[STAT_WEAPONS],
                &mut iDummy,	//	&client->ps.stats[STAT_ITEMS],
                &mut iDummy,	//	&client->ps.weapon,
                &mut iDummy,	//	&client->ps.weaponstate,
                &mut iDummy,	//	&client->ps.batteryCharge,
                &mut fDummy,	//	&client->ps.viewangles[0],
                &mut fDummy,	//	&client->ps.viewangles[1],
                &mut fDummy,	//	&client->ps.viewangles[2],
                                //force power data
                forceBits,      //	&client->ps.forcePowersKnown,
                &mut iDummy,	//	&client->ps.forcePower,
        );
    } else {
        // will also need to do this for weapons
        if ((*core::ptr::addr_of!(gi)).Cvar_VariableIntegerValue)(b"com_demo\0".as_ptr() as *const c_char) != 0 {
            ((*core::ptr::addr_of!(gi)).Cvar_VariableStringBuffer)(b"demo_playerwpns\0".as_ptr() as *const c_char, s.as_mut_ptr(), core::mem::size_of_val(&s));

            *weaponBits = atoi(s.as_ptr());
        }
    }

    if ((*core::ptr::addr_of!(gi)).Cvar_VariableIntegerValue)(b"com_demo\0".as_ptr() as *const c_char) != 0 {
        // le Demo stuff...
        // the new JK2 stuff - force powers, etc...
        //
        *forceBits = 0; // need to zero it out it might have already been set above if coming from a true
                        // map transition in the demo
        ((*core::ptr::addr_of!(gi)).Cvar_VariableStringBuffer)(b"demo_playerfplvl\0".as_ptr() as *const c_char, s.as_mut_ptr(), core::mem::size_of_val(&s));
        let mut j: c_int = 0;
        var = strtok(s.as_mut_ptr(), b" \0".as_ptr() as *const c_char);
        while !var.is_null() {
            /* While there are tokens in "s" */
            (*core::ptr::addr_of_mut!(loadForcePowerLevel))[j as usize] = atoi(var);
            if (*core::ptr::addr_of!(loadForcePowerLevel))[j as usize] != 0 {
                *forceBits |= 1 << j;
            }
            j += 1;
            /* Get next token: */
            var = strtok(core::ptr::null_mut(), b" \0".as_ptr() as *const c_char);
        }
    } else {
        // the new JK2 stuff - force powers, etc...
        //
        ((*core::ptr::addr_of!(gi)).Cvar_VariableStringBuffer)(b"playerfplvl\0".as_ptr() as *const c_char, s.as_mut_ptr(), core::mem::size_of_val(&s));
        i = 0;
        var = strtok(s.as_mut_ptr(), b" \0".as_ptr() as *const c_char);
        while !var.is_null() {
            /* While there are tokens in "s" */
            (*core::ptr::addr_of_mut!(loadForcePowerLevel))[i as usize] = atoi(var);
            i += 1;
            /* Get next token: */
            var = strtok(core::ptr::null_mut(), b" \0".as_ptr() as *const c_char);
        }
    }
}

/*
====================
CG_DrawLoadingScreen

Load screen displays the map pic, the mission briefing and weapons/force powers
====================
*/
unsafe fn CG_DrawLoadingScreen(levelshot: qhandle_t, mapName: *const c_char) {
    let mut xPos: c_int = 0;
    let mut yPos: c_int = 0;
    let mut width: c_int = 0;
    let mut height: c_int = 0;
    let mut color: vec4_t = core::mem::zeroed();
    let mut background: qhandle_t = 0;
    let mut weapons: c_int = 0;
    let mut forcepowers: c_int = 0;

    // Get mission briefing for load screen
    if cgi_SP_GetStringTextString(va(b"BRIEFINGS_%s\0".as_ptr() as *const c_char, mapName), core::ptr::null_mut(), 0) == 0 {
        cgi_Cvar_Set(b"ui_missionbriefing\0".as_ptr() as *const c_char, b"@BRIEFINGS_NONE\0".as_ptr() as *const c_char);
    } else {
        cgi_Cvar_Set(b"ui_missionbriefing\0".as_ptr() as *const c_char, va(b"@BRIEFINGS_%s\0".as_ptr() as *const c_char, mapName));
    }

    // Print background
    if cgi_UI_GetMenuItemInfo(
        b"loadScreen\0".as_ptr() as *const c_char,
        b"background\0".as_ptr() as *const c_char,
        &mut xPos,
        &mut yPos,
        &mut width,
        &mut height,
        color.as_mut_ptr(),
        &mut background) != 0
    {
        cgi_R_SetColor(color.as_ptr());
        CG_DrawPic(xPos, yPos, width, height, background);
    }

    // Print level pic
    if cgi_UI_GetMenuItemInfo(
        b"loadScreen\0".as_ptr() as *const c_char,
        b"mappic\0".as_ptr() as *const c_char,
        &mut xPos,
        &mut yPos,
        &mut width,
        &mut height,
        color.as_mut_ptr(),
        &mut background) != 0
    {
        //if (!levelshot)
        //{// No level shot so use screenshot.
 	    //		CG_DrawPic( xPos, yPos, 1, 1, 0);	//force the tess to flush
        //	cgi_R_DrawScreenShot( xPos, yPos+height, width, -height );
        //}
        //else
        {
            cgi_R_SetColor(color.as_ptr());
            CG_DrawPic(xPos, yPos, width, height, levelshot);
        }
    }

    // Get player weapons and force power info
    CG_GetLoadScreenInfo(&mut weapons, &mut forcepowers);

    // Print weapon icons
    if weapons != 0 {
        CG_DrawLoadWeapons(weapons);
    }

    // Print force power icons
    if forcepowers != 0 {
        CG_DrawLoadForcePowers(forcepowers);
    }
}

/*
====================
CG_DrawInformation

Draw all the status / pacifier stuff during level loading
====================
*/
#[no_mangle]
pub unsafe extern "C" fn CG_DrawInformation() {
    let mut y: c_int;

    // draw the dialog background
    let info: *const c_char = CG_ConfigString(CS_SERVERINFO);
    let mut s: *const c_char = Info_ValueForKey(info, b"mapname\0".as_ptr() as *const c_char);

    let mut levelshot: qhandle_t;

    // extern SavedGameJustLoaded_e g_eSavedGameJustLoaded;	// hack! (hey, it's the last week of coding, ok?
    //#ifndef _XBOX
    //	if ( g_eSavedGameJustLoaded == eFULL )
    //	{
    //		levelshot = 0;	//use the loaded thumbnail instead of the levelshot
    //	}
    //	else
    //#endif
    {
        levelshot = cgi_R_RegisterShaderNoMip(va(b"levelshots/%s\0".as_ptr() as *const c_char, s));
        #[cfg(not(feature = "final_build"))]
        if levelshot == 0 && strncmp(s, b"work/\0".as_ptr() as *const c_char, 5) == 0 {
            levelshot = cgi_R_RegisterShaderNoMip(va(b"levelshots/%s\0".as_ptr() as *const c_char, s.offset(5)));
        }
        if levelshot == 0 {
            levelshot = cgi_R_RegisterShaderNoMip(b"menu/art/unknownmap\0".as_ptr() as *const c_char);
        }
    }

    if g_eSavedGameJustLoaded != eFULL && (strcmp(s, b"yavin1\0".as_ptr() as *const c_char) == 0 || strcmp(s, b"demo\0".as_ptr() as *const c_char) == 0) //special case for first map!
    {
        let mut text: [c_char; 1024] = [0; 1024];

        //
        cgi_R_SetColor(colorTable[CT_BLACK as usize]);
        CG_DrawPic(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, (*core::ptr::addr_of!(cgs)).media.whiteShader);

        cgi_SP_GetStringTextString(b"SP_INGAME_ALONGTIME\0".as_ptr() as *const c_char, text.as_mut_ptr(), core::mem::size_of_val(&text));

        let w: c_int = cgi_R_Font_StrLenPixels(text.as_ptr(), (*core::ptr::addr_of!(cgs)).media.qhFontMedium, 1.0f32);
        cgi_R_Font_DrawString((320) - (w / 2), 140, text.as_ptr(), colorTable[CT_ICON_BLUE as usize], (*core::ptr::addr_of!(cgs)).media.qhFontMedium, -1, 1.0f32);
    } else {
        CG_DrawLoadingScreen(levelshot, s);
        cgi_UI_MenuPaintAll();
    }

    CG_LoadBar();


    // the first 150 rows are reserved for the client connection
    // screen to write into
    //	if ( cg.processedSnapshotNum == 0 )
    {
        // still loading
        // print the current item being loaded

    #[cfg(debug_assertions)]
        cgi_R_Font_DrawString(40, 416, va(b"LOADING ... %s\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(cg)).infoScreenText.as_ptr()), colorTable[CT_LTGOLD1 as usize], (*core::ptr::addr_of!(cgs)).media.qhFontSmall, -1, 1.0f32);
    }

    // draw info string information

    y = 20;
    // map-specific message (long map name)
    s = CG_ConfigString(CS_MESSAGE);

    if *s != 0 {
        if *s == b'@' as c_char {
            let mut text: [c_char; 1024] = [0; 1024];
            cgi_SP_GetStringTextString(s.offset(1), text.as_mut_ptr(), core::mem::size_of_val(&text));
            cgi_R_Font_DrawString(15, y, va(b"\"%s\"\0".as_ptr() as *const c_char, text.as_ptr()), colorTable[CT_WHITE as usize], (*core::ptr::addr_of!(cgs)).media.qhFontMedium, -1, 1.0f32);
        } else {
            cgi_R_Font_DrawString(15, y, va(b"\"%s\"\0".as_ptr() as *const c_char, s), colorTable[CT_WHITE as usize], (*core::ptr::addr_of!(cgs)).media.qhFontMedium, -1, 1.0f32);
        }
        y += 20;
    }
}
