// this line must stay at top so the whole PCH thing works...
// extern module declarations would go here for cg_headers, cg_media, objectives

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_short};

// Opaque type declarations for external types from other modules
// These would be properly defined in their respective modules
#[repr(C)]
pub struct centity_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct cg_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct cgs_t {
    _private: [u8; 0],
}

pub type qboolean = c_int;
pub type qhandle_t = c_int;
pub type vec4_t = [f32; 4];

// External declarations for globals from other modules
extern "C" {
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static colorTable: [[f32; 4]; 32];  // Assuming array of vec4_t
    pub static objectiveTable: *mut core::ffi::c_void;  // Opaque type reference
}

// External function declarations
extern "C" {
    fn cgi_R_Font_HeightPixels(fontHandle: c_int, scale: f32) -> c_int;
    fn gi_Cvar_VariableIntegerValue(name: *const c_char) -> c_int;
    fn cgi_SP_GetStringTextString(
        label: *const c_char,
        buffer: *mut c_char,
        bufsize: usize,
    ) -> c_int;
    fn cgi_R_Font_StrLenPixels(text: *const c_char, fontHandle: c_int, scale: f32) -> c_int;
    fn cgi_Language_IsAsian() -> qboolean;
    fn CG_DisplayBoxedText(
        iBoxX: c_int,
        iBoxY: c_int,
        iBoxWidth: c_int,
        iBoxHeight: c_int,
        psText: *const c_char,
        iFontHandle: c_int,
        fScale: f32,
        v4Color: *const vec4_t,
    ) -> *const c_char;
    pub static mut giLinesOutput: c_int;
    pub static mut gfAdvanceHack: f32;
    fn cgi_R_Font_DrawString(
        x: c_int,
        y: c_int,
        text: *const c_char,
        color: *const vec4_t,
        fontHandle: c_int,
        iMaxPixelWidth: c_int,
        scale: f32,
    );
    fn CG_DrawProportionalString(
        x: c_int,
        y: c_int,
        text: *const c_char,
        flags: c_int,
        color: *const vec4_t,
    );
    fn cgi_R_SetColor(color: *const vec4_t);
    fn CG_DrawPic(x: c_int, y: c_int, width: c_int, height: c_int, hShader: qhandle_t);
    fn cgi_R_RegisterShaderNoMip(name: *const c_char) -> qhandle_t;
    fn CG_ConfigString(index: c_int) -> *const c_char;
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    fn cgi_Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn cgi_UI_GetMenuItemInfo(
        menuName: *const c_char,
        itemName: *const c_char,
        x: *mut c_int,
        y: *mut c_int,
        width: *mut c_int,
        height: *mut c_int,
        color: *mut vec4_t,
        background: *mut qhandle_t,
    ) -> qboolean;
    fn cgi_UI_MenuPaintAll();
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    fn atoi(str: *const c_char) -> c_int;
    fn sprintf(str: *mut c_char, format: *const c_char, ...);
    fn CG_RegisterWeapon(weaponIndex: c_int);
}

// For printing objectives
const objectiveStartingYpos: c_short = 75;		// Y starting position for objective text
const objectiveStartingXpos: c_short = 60;		// X starting position for objective text
const objectiveTextBoxWidth: c_int = 500;		// Width (in pixels) of text box
const objectiveTextBoxHeight: c_int = 300;		// Height (in pixels) of text box

static SHOW_LOAD_POWERS_NAME: &[*const c_char] = &[
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

#[allow(non_upper_case_globals)]
const MAX_OBJ_GRAPHICS: usize = 4;
#[allow(non_upper_case_globals)]
const OBJ_GRAPHIC_SIZE: c_int = 240;
static mut obj_graphics: [c_int; 4] = [0; 4];

fn CG_ForcePower_Valid(forceKnownBits: c_int, index: c_int) -> qboolean;

/*
====================
ObjectivePrint_Line

Print a single mission objective
====================
*/
fn ObjectivePrint_Line(color: c_int, objectIndex: c_int, missionYcnt: &mut c_int) {
    let mut str_var: *mut c_char;
    let mut strBegin: *mut c_char;
    let mut y: c_int;
    let mut pixelLen: c_int;
    let mut charLen: c_int;
    let mut i: c_int = 0;
    const maxHoldText: usize = 1024;
    let mut holdText: [c_char; 1024] = [0; 1024];
    let mut finalText: [c_char; 2048] = [0; 2048];
    let mut graphic: qhandle_t = 0;

    let iYPixelsPerLine: c_int = unsafe { cgi_R_Font_HeightPixels(unsafe { (*core::addr_of!(cgs)).media.qhFontMedium }, 1.0) };

    unsafe {
        if gi_Cvar_VariableIntegerValue(b"com_demo\0".as_ptr() as *const c_char) != 0 {
            cgi_SP_GetStringTextString(
                va(b"OBJECTIVES_DEMO_%s\0".as_ptr() as *const c_char, unsafe { (*(*objectiveTable as *const c_void as *const c_void)).name }),
                finalText.as_mut_ptr(),
                core::mem::size_of_val(&finalText),
            );
        } else {
            cgi_SP_GetStringTextString(
                va(b"OBJECTIVES_%s\0".as_ptr() as *const c_char, unsafe { (*(*objectiveTable as *const c_void as *const c_void)).name }),
                finalText.as_mut_ptr(),
                core::mem::size_of_val(&finalText),
            );
        }
    }

    // A hack to be able to count prisoners
    if objectIndex == 0x04 {  // T2_RANCOR_OBJ5
        let mut value: [c_char; 64] = [0; 64];
        let mut currTotal: c_int;
        let mut minTotal: c_int;

        unsafe {
            let gi_Cvar_VariableStringBuffer_ptr = core::mem::transmute::<_, fn(*const c_char, *mut c_char, usize)>(
                gi_Cvar_VariableIntegerValue as *const (),
            );
            // Note: We need the proper gi struct access - this is a placeholder
            // gi.Cvar_VariableStringBuffer("ui_prisonerobj_currtotal",value,sizeof(value));
            currTotal = atoi(value.as_ptr());
            // gi.Cvar_VariableStringBuffer("ui_prisonerobj_maxtotal",value,sizeof(value));
            minTotal = atoi(value.as_ptr());

            sprintf(
                finalText.as_mut_ptr(),
                va(
                    finalText.as_ptr(),
                    currTotal,
                    minTotal,
                ),
            );
        }
    }

    pixelLen = unsafe { cgi_R_Font_StrLenPixels(finalText.as_ptr(), unsafe { (*core::addr_of!(cgs)).media.qhFontMedium }, 1.0) };

    str_var = finalText.as_mut_ptr();

    unsafe {
        if cgi_Language_IsAsian() != 0 {
            // this is execrable, and should NOT have had to've been done now, but...
            //
            // extern const char *CG_DisplayBoxedText(	int iBoxX, int iBoxY, int iBoxWidth, int iBoxHeight,
            //												const char *psText, int iFontHandle, float fScale,
            //												const vec4_t v4Color);
            // extern int giLinesOutput;
            // extern float gfAdvanceHack;

            gfAdvanceHack = 1.0;	// override internal vertical advance
            y = objectiveStartingYpos as c_int + (iYPixelsPerLine * *missionYcnt);

            // Advance line if a graphic has printed
            i = 0;
            while i < 4 {
                if *core::addr_of_mut!(obj_graphics)[i] != 0 {
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
                unsafe { (*core::addr_of!(cgs)).media.qhFontMedium },		// int iFontHandle,
                1.0,						// float fScale,
                core::addr_of!(colorTable[0]),			// const vec4_t v4Color
            );

            gfAdvanceHack = 0.0;	// restore
            *missionYcnt += giLinesOutput;
        } else {
            // western...
            //
            if pixelLen < objectiveTextBoxWidth {	// One shot - small enough to print entirely on one line
                y = objectiveStartingYpos as c_int + (iYPixelsPerLine * (*missionYcnt));

                cgi_R_Font_DrawString(
                    objectiveStartingXpos as c_int,
                    y,
                    str_var,
                    core::addr_of!(colorTable[0]),
                    unsafe { (*core::addr_of!(cgs)).media.qhFontMedium },
                    -1,
                    1.0,
                );

                *missionYcnt += 1;
            } else {
                // Text is too long, break into lines.
                let mut holdText2: [c_char; 2] = [0; 2];
                pixelLen = 0;
                charLen = 0;
                holdText2[1] = 0 as c_char;
                strBegin = str_var;

                while *str_var != 0 {
                    holdText2[0] = *str_var;
                    pixelLen += cgi_R_Font_StrLenPixels(holdText2.as_ptr(), unsafe { (*core::addr_of!(cgs)).media.qhFontMedium }, 1.0);

                    pixelLen += 2; // For kerning
                    charLen += 1;

                    if pixelLen > objectiveTextBoxWidth {
                        //Reached max length of this line
                        //step back until we find a space
                        while (charLen > 10) && (*str_var != b' ' as c_char) {
                            str_var = str_var.offset(-1);
                            charLen -= 1;
                        }

                        if *str_var == b' ' as c_char {
                            str_var = str_var.offset(1);	// To get past space
                        }

                        assert!(charLen < maxHoldText);	// Too big?

                        Q_strncpyz(holdText.as_mut_ptr(), strBegin, charLen as usize);
                        holdText[charLen as usize] = 0 as c_char;
                        strBegin = str_var;
                        pixelLen = 0;
                        charLen = 1;

                        y = objectiveStartingYpos as c_int + (iYPixelsPerLine * *missionYcnt);

                        CG_DrawProportionalString(
                            objectiveStartingXpos as c_int,
                            y,
                            holdText.as_ptr(),
                            0,  // CG_SMALLFONT
                            core::addr_of!(colorTable[0]),
                        );

                        *missionYcnt += 1;
                    } else if *str_var.offset(1) == 0 {
                        charLen += 1;

                        assert!(charLen < maxHoldText as c_int);	// Too big?

                        y = objectiveStartingYpos as c_int + (iYPixelsPerLine * *missionYcnt);

                        Q_strncpyz(holdText.as_mut_ptr(), strBegin, charLen as usize);
                        CG_DrawProportionalString(
                            objectiveStartingXpos as c_int,
                            y,
                            holdText.as_ptr(),
                            0,  // CG_SMALLFONT
                            core::addr_of!(colorTable[0]),
                        );

                        *missionYcnt += 1;
                        break;
                    }
                    str_var = str_var.offset(1);
                }
            }
        }

        if objectIndex == 0x01 {  // T3_BOUNTY_OBJ1
            y = objectiveStartingYpos as c_int + (iYPixelsPerLine * *missionYcnt);
            if *core::addr_of_mut!(obj_graphics)[1] != 0 {
                y += OBJ_GRAPHIC_SIZE + 4;
            }
            if *core::addr_of_mut!(obj_graphics)[2] != 0 {
                y += OBJ_GRAPHIC_SIZE + 4;
            }
            graphic = cgi_R_RegisterShaderNoMip(b"textures/system/viewscreen1\0".as_ptr() as *const c_char);
            CG_DrawPic(355, 50, OBJ_GRAPHIC_SIZE, OBJ_GRAPHIC_SIZE, graphic);
            *core::addr_of_mut!(obj_graphics)[3] = 1;
        }
    }
}

/*
====================
CG_DrawDataPadObjectives

Draw routine for the objective info screen of the data pad.
====================
*/
fn CG_DrawDataPadObjectives(cent: *const centity_t) {
    let mut i: c_int;
    let mut totalY: c_int;
    let iYPixelsPerLine: c_int = unsafe { cgi_R_Font_HeightPixels(unsafe { (*core::addr_of!(cgs)).media.qhFontMedium }, 1.0) };

    let titleXPos: c_short = objectiveStartingXpos - 22;		// X starting position for title text
    let titleYPos: c_short = objectiveStartingYpos - 23;		// Y starting position for title text
    let graphic_size: c_short = 16;							// Size (width and height) of graphic used to show status of objective
    let graphicXpos: c_short = objectiveStartingXpos - graphic_size - 8;	// Amount of X to backup from text starting position
    let graphicYOffset: c_short = ((iYPixelsPerLine - graphic_size as c_int) / 2) as c_short;	// Amount of Y to raise graphic so it's in the center of the text line

    unsafe {
        // Placeholder for missionInfo_Updated = qfalse;		// This will stop the text from flashing
        // (*core::addr_of_mut!(cg)).missionInfoFlashTime = 0;

        // zero out objective graphics
        i = 0;
        while i < 4 {
            *core::addr_of_mut!(obj_graphics)[i as usize] = 0;
            i += 1;
        }

        // Title Text at the top
        let mut text: [c_char; 1024] = [0; 1024];
        cgi_SP_GetStringTextString(b"SP_INGAME_OBJECTIVES\0".as_ptr() as *const c_char, text.as_mut_ptr(), core::mem::size_of_val(&text));
        cgi_R_Font_DrawString(
            titleXPos as c_int,
            titleYPos as c_int,
            text.as_ptr(),
            core::addr_of!(colorTable[5]),  // CT_TITLE
            unsafe { (*core::addr_of!(cgs)).media.qhFontMedium },
            -1,
            1.0,
        );

        let mut missionYcnt: c_int = 0;

        // Print all active objectives
        i = 0;
        while i < 32 {  // MAX_OBJECTIVES
            // Is there an objective to see?
            // if (cent->gent->client->sess.mission_objectives[i].display)
            {
                // Calculate the Y position
                totalY = objectiveStartingYpos as c_int + (iYPixelsPerLine * (missionYcnt)) + (iYPixelsPerLine / 2);

                //	Draw graphics that show if mission has been accomplished or not
                cgi_R_SetColor(core::addr_of!(colorTable[2]));  // CT_BLUE3
                CG_DrawPic(
                    graphicXpos as c_int,
                    totalY - graphicYOffset as c_int,
                    graphic_size as c_int,
                    graphic_size as c_int,
                    0,  // cgs.media.messageObjCircle
                );	// Circle in front
                // if (cent->gent->client->sess.mission_objectives[i].status == OBJECTIVE_STAT_SUCCEEDED)
                {
                    CG_DrawPic(
                        graphicXpos as c_int,
                        totalY - graphicYOffset as c_int,
                        graphic_size as c_int,
                        graphic_size as c_int,
                        0,  // cgs.media.messageLitOn
                    );	// Center Dot
                }

                // Print current objective text
                ObjectivePrint_Line(7, i, &mut missionYcnt);	// CT_WHITE
            }
            i += 1;
        }

        // No mission text?
        if missionYcnt == 0 {
            // Set the message a quarter of the way down and in the center of the text box
            let messageYPosition: c_int = objectiveStartingYpos as c_int + (objectiveTextBoxHeight / 4);

            cgi_SP_GetStringTextString(b"SP_INGAME_OBJNONE\0".as_ptr() as *const c_char, text.as_mut_ptr(), core::mem::size_of_val(&text));
            let messageXPosition: c_int =
                objectiveStartingXpos as c_int + (objectiveTextBoxWidth / 2) -
                (cgi_R_Font_StrLenPixels(text.as_ptr(), unsafe { (*core::addr_of!(cgs)).media.qhFontMedium }, 1.0) / 2);

            cgi_R_Font_DrawString(
                messageXPosition,
                messageYPosition,
                text.as_ptr(),
                core::addr_of!(colorTable[7]),  // CT_WHITE
                unsafe { (*core::addr_of!(cgs)).media.qhFontMedium },
                -1,
                1.0,
            );
        }
    }
}

/*

//-------------------------------------------------------
static void CG_DrawForceCount( const int force, int x, float *y, const int pad,qboolean *hasForcePowers )
{
	char	s[MAX_STRING_CHARS];
	int		val, textColor;
	char	text[1024]={0};

	gi.Cvar_VariableStringBuffer( va("playerfplvl%d", force ),s, sizeof(s) );

	sscanf( s, "%d",&val );

	if ((val<1) || (val> NUM_FORCE_POWERS))
	{
		return;
	}

	textColor = CT_ICON_BLUE;

	// Draw title
	cgi_SP_GetStringTextString( showLoadPowersName[force], text, sizeof(text) );
	CG_DrawProportionalString( x, *y, text, CG_BIGFONT, colorTable[textColor] );


	// Draw icons
	cgi_R_SetColor( colorTable[CT_WHITE]);
	const int iconSize = 30;
	if ( val >= 0 )
	{
		x -= 10;	// Back up from title a little

		for ( int i = 0; i < val; i++ )
		{
			CG_DrawPic( x - iconSize - i * (iconSize + 10) , *y, iconSize, iconSize, force_icons[force] );
		}
	}

	*y += pad;

	*hasForcePowers = qtrue;
}


/*
====================
CG_LoadScreen_PersonalInfo
====================
*/
/*
static void CG_LoadScreen_PersonalInfo(void)
{
	float	x, y;
	int		pad = 25;
	char	text[1024]={0};
	qboolean	hasForcePowers;

	y = 65 + 30;

	pad = 28;
	x = 300;
	hasForcePowers=qfalse;

	CG_DrawForceCount( FP_HEAL, x, &y, pad,&hasForcePowers);
	CG_DrawForceCount( FP_LEVITATION, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_SPEED, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_PUSH, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_PULL, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_TELEPATHY, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_GRIP, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_LIGHTNING, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_SABERTHROW, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_SABER_OFFENSE, x, &y, pad,&hasForcePowers );
	CG_DrawForceCount( FP_SABER_DEFENSE, x, &y, pad,&hasForcePowers );

	if (hasForcePowers)
	{
		cgi_SP_GetStringTextString( "SP_INGAME_CURRENTFORCEPOWERS", text, sizeof(text) );
		CG_DrawProportionalString( 200, 65, text, CG_CENTER | CG_BIGFONT, colorTable[CT_WHITE] );
	}
	else
	{	//you are only totally empty on the very first map?
//		cgi_SP_GetStringTextString( "SP_INGAME_NONE", text, sizeof(text) );
//		CG_DrawProportionalString( 320, y+30, text, CG_CENTER | CG_BIGFONT, colorTable[CT_ICON_BLUE] );
		cgi_SP_GetStringTextString( "SP_INGAME_ALONGTIME", text, sizeof(text) );
		int w = cgi_R_Font_StrLenPixels(text,cgs.media.qhFontMedium, 1.5f);
		cgi_R_Font_DrawString((320)-(w/2), y+40, text,  colorTable[CT_ICON_BLUE], cgs.media.qhFontMedium, -1, 1.5f);
	}

}
*/

fn CG_LoadBar() {
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

    unsafe {
        cgi_R_SetColor(core::addr_of!(colorTable[7]));  // CT_WHITE
        // Draw background
        CG_DrawPic(barleft, bartop, barwidth, barheight, 0);  // cgs.media.levelLoad

        // Draw left cap (backwards)
        CG_DrawPic(tickleft, ticktop, -capwidth, tickheight, 0);  // cgs.media.loadTickCap

        // Draw bar
        CG_DrawPic(tickleft, ticktop, tickwidth * (*core::addr_of!(cg)).loadLCARSStage, tickheight, 0);  // cgs.media.loadTick

        // Draw right cap
        CG_DrawPic(tickleft + tickwidth * (*core::addr_of!(cg)).loadLCARSStage, ticktop, capwidth, tickheight, 0);  // cgs.media.loadTickCap
    }
}

extern "C" {
    fn CG_WeaponCheck(weaponIndex: c_int) -> c_int;
}

// For printing load screen icons
const MAXLOADICONSPERROW: c_int = 8;		// Max icons displayed per row
const MAXLOADWEAPONS: c_int = 16;
const MAXLOADFORCEPOWERS: c_int = 12;
const MAXLOAD_FORCEICONSIZE: c_int = 40;	// Size of force power icons
const MAXLOAD_FORCEICONPAD: c_int = 12;	// Padding space between icons

fn CG_DrawLoadWeaponsPrintRow(itemName: *const c_char, weaponsBits: c_int, rowIconCnt: c_int, startIndex: c_int) -> c_int {
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
    let mut color: vec4_t = [0.0; 4];
    let mut background: qhandle_t = 0;

    unsafe {
        if cgi_UI_GetMenuItemInfo(
            b"loadScreen\0".as_ptr() as *const c_char,
            itemName,
            &mut x,
            &mut y,
            &mut width,
            &mut height,
            color.as_mut_ptr() as *mut vec4_t,
            &mut background,
        ) == 0
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

            // if (weaponData[i].weaponIcon[0])
            {
                // weaponInfo_t	*weaponInfo;
                CG_RegisterWeapon(i);
                // weaponInfo = &cg_weapons[i];
                endIndex = i;

                // NOTE : during loading screen always show the have ammo icon
                //		if (!CG_WeaponCheck(i))
                //		{
                //			CG_DrawPic( holdX, y+yOffset, iconSize, iconSize, weaponInfo->weaponIconNoAmmo );
                //		}
                //		else
                {
                    CG_DrawPic(holdX, y + yOffset, iconSize, iconSize, 0);  // weaponInfo->weaponIcon
                }

                printedIconCnt += 1;
                if printedIconCnt == MAXLOADICONSPERROW {
                    break;
                }

                holdX += (iconSize + pad);
            }
            i += 1;
        }
    }

    endIndex
}

// Print weapons the player is carrying
// Two rows print if there are too many
fn CG_DrawLoadWeapons(weaponBits: c_int) {
    let mut i: c_int;
    let mut endIndex: c_int = 0;
    let mut iconCnt: c_int;
    let mut rowIconCnt: c_int;

    // count the number of weapons owned
    iconCnt = 0;
    i = 1;
    while i < MAXLOADWEAPONS {
        if (weaponBits & (1 << i)) != 0 {
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
    } else {
        // Two lines of icons
        // Print top row
        endIndex = CG_DrawLoadWeaponsPrintRow(b"weaponicons_row1\0".as_ptr() as *const c_char, weaponBits, MAXLOADICONSPERROW, 0);

        // Print second row
        rowIconCnt = iconCnt - MAXLOADICONSPERROW;
        CG_DrawLoadWeaponsPrintRow(b"weaponicons_row2\0".as_ptr() as *const c_char, weaponBits, rowIconCnt, endIndex + 1);
    }

    unsafe {
        cgi_R_SetColor(core::ptr::null());
    }
}


fn CG_DrawLoadForcePrintRow(itemName: *const c_char, forceBits: c_int, rowIconCnt: c_int, startIndex: c_int) -> c_int {
    let mut i: c_int;
    let mut endIndex: c_int = 0;
    let mut printedIconCnt: c_int = 0;
    let mut holdX: c_int;
    let mut x: c_int = 0;
    let mut y: c_int = 0;
    let mut yOffset: c_int = 0;
    let mut width: c_int = 0;
    let mut height: c_int = 0;
    let mut color: vec4_t = [0.0; 4];
    let mut background: qhandle_t = 0;

    unsafe {
        if cgi_UI_GetMenuItemInfo(
            b"loadScreen\0".as_ptr() as *const c_char,
            itemName,
            &mut x,
            &mut y,
            &mut width,
            &mut height,
            color.as_mut_ptr() as *mut vec4_t,
            &mut background,
        ) == 0
        {
            return 0;
        }

        cgi_R_SetColor(color.as_ptr());

        // calculate placement of weapon icons
        holdX = x + (width - ((MAXLOAD_FORCEICONSIZE * rowIconCnt) + (MAXLOAD_FORCEICONPAD * (rowIconCnt - 1)))) / 2;

        i = startIndex;
        while i < 12 {  // MAX_SHOWPOWERS
            if CG_ForcePower_Valid(forceBits, i) == 0 {	// Does he have this power?
                i += 1;
                continue;
            }

            // if (force_icons[showPowers[i]])
            {
                endIndex = i;

                CG_DrawPic(holdX, y + yOffset, MAXLOAD_FORCEICONSIZE, MAXLOAD_FORCEICONSIZE, 0);  // force_icons[showPowers[i]]

                printedIconCnt += 1;
                if printedIconCnt == MAXLOADICONSPERROW {
                    break;
                }

                holdX += (MAXLOAD_FORCEICONSIZE + MAXLOAD_FORCEICONPAD);
            }
            i += 1;
        }
    }

    endIndex
}

static mut loadForcePowerLevel: [c_int; 12] = [0; 12];  // NUM_FORCE_POWERS

/*
===============
ForcePowerDataPad_Valid
===============
*/
fn CG_ForcePower_Valid(forceKnownBits: c_int, index: c_int) -> qboolean {
    unsafe {
        // Placeholder for showPowers array access
        if (forceKnownBits & (1 << 0)) != 0 && loadForcePowerLevel[0] != 0 {	// Does he have the force power?
            return 1;
        }
    }

    0
}

// Print force powers the player is using
// Two rows print if there are too many
fn CG_DrawLoadForcePowers(forceBits: c_int) {
    let mut i: c_int;
    let mut endIndex: c_int = 0;
    let mut iconCnt: c_int = 0;
    let mut rowIconCnt: c_int;

    // Count the number of force powers known
    i = 0;
    while i < 12 {  // MAX_SHOWPOWERS
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
    } else {
        // Two lines of icons
        // Print top row
        endIndex = CG_DrawLoadForcePrintRow(b"forceicons_row1\0".as_ptr() as *const c_char, forceBits, MAXLOADICONSPERROW, 0);

        // Print second row
        rowIconCnt = iconCnt - MAXLOADICONSPERROW;
        CG_DrawLoadForcePrintRow(b"forceicons_row2\0".as_ptr() as *const c_char, forceBits, rowIconCnt, endIndex + 1);
    }

    unsafe {
        cgi_R_SetColor(core::ptr::null());
    }
}

// Get the player weapons and force power info
fn CG_GetLoadScreenInfo(weaponBits: &mut c_int, forceBits: &mut c_int) {
    let mut s: [c_char; 4096] = [0; 4096];  // MAX_STRING_CHARS
    let mut iDummy: c_int;
    let mut i: c_int;
    let mut fDummy: f32;
    let mut var: *const c_char;

    unsafe {
        // gi.Cvar_VariableStringBuffer( sCVARNAME_PLAYERSAVE, s, sizeof(s) );
        // Placeholder - needs actual CVAR name

        // Get player weapons and force powers known
        if s[0] as u8 != 0 {
            //				|general info				  |-force powers
            // sscanf( s, "%i %i %i %i %i %i %i %f %f %f %i %i",
            //		&iDummy,	//	&client->ps.stats[STAT_HEALTH],
            //		&iDummy,	//	&client->ps.stats[STAT_ARMOR],
            //		&*weaponBits,//	&client->ps.stats[STAT_WEAPONS],
            //		&iDummy,	//	&client->ps.stats[STAT_ITEMS],
            //		&iDummy,	//	&client->ps.weapon,
            //		&iDummy,	//	&client->ps.weaponstate,
            //		&iDummy,	//	&client->ps.batteryCharge,
            //		&fDummy,	//	&client->ps.viewangles[0],
            //		&fDummy,	//	&client->ps.viewangles[1],
            //		&fDummy,	//	&client->ps.viewangles[2],
            //					//force power data
            //		&*forceBits,	//	&client->ps.forcePowersKnown,
            //		&iDummy		//	&client->ps.forcePower,
            //
            //		);
        } else {
            // will also need to do this for weapons
            if gi_Cvar_VariableIntegerValue(b"com_demo\0".as_ptr() as *const c_char) != 0 {
                // gi.Cvar_VariableStringBuffer( "demo_playerwpns", s, sizeof(s) );

                *weaponBits = atoi(s.as_ptr());
            }
        }

        if gi_Cvar_VariableIntegerValue(b"com_demo\0".as_ptr() as *const c_char) != 0 {
            // le Demo stuff...
            // the new JK2 stuff - force powers, etc...
            //
            *forceBits = 0; // need to zero it out it might have already been set above if coming from a true
                            // map transition in the demo
            // gi.Cvar_VariableStringBuffer( "demo_playerfplvl", s, sizeof(s) );
            let mut j: c_int = 0;
            // var = strtok( s, " " );
            // while( var != NULL )
            {
                /* While there are tokens in "s" */
                loadForcePowerLevel[j as usize] = atoi(var);
                if loadForcePowerLevel[j as usize] != 0 {
                    *forceBits |= 1 << j;
                }
                j += 1;
                /* Get next token: */
                // var = strtok( NULL, " " );
            }
        } else {
            // the new JK2 stuff - force powers, etc...
            //
            // gi.Cvar_VariableStringBuffer( "playerfplvl", s, sizeof(s) );
            i = 0;
            // var = strtok( s, " " );
            // while( var != NULL )
            {
                /* While there are tokens in "s" */
                loadForcePowerLevel[i as usize] = atoi(var);
                i += 1;
                /* Get next token: */
                // var = strtok( NULL, " " );
            }
        }
    }
}

/*
====================
CG_DrawLoadingScreen

Load screen displays the map pic, the mission briefing and weapons/force powers
====================
*/
fn CG_DrawLoadingScreen(levelshot: qhandle_t, mapName: *const c_char) {
    let mut xPos: c_int = 0;
    let mut yPos: c_int = 0;
    let mut width: c_int = 0;
    let mut height: c_int = 0;
    let mut color: vec4_t = [0.0; 4];
    let mut background: qhandle_t = 0;
    let mut weapons: c_int = 0;
    let mut forcepowers: c_int = 0;

    unsafe {
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
            color.as_mut_ptr() as *mut vec4_t,
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
            color.as_mut_ptr() as *mut vec4_t,
            &mut background) != 0
        {
            //if (!levelshot)
            //{// No level shot so use screenshot.
            //		CG_DrawPic( xPos, yPos, 1, 1, 0);	//force the tess to flush
            //		cgi_R_DrawScreenShot( xPos, yPos+height, width, -height );
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
}

/*
====================
CG_DrawInformation

Draw all the status / pacifier stuff during level loading
====================
*/
pub fn CG_DrawInformation() {
    let mut y: c_int;

    unsafe {
        // draw the dialog background
        let info: *const c_char = CG_ConfigString(16);  // CS_SERVERINFO
        let s: *const c_char = Info_ValueForKey(info, b"mapname\0".as_ptr() as *const c_char);

        let mut levelshot: qhandle_t = 0;

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
            // #ifndef FINAL_BUILD
            if levelshot == 0 && s as *const u8 != core::ptr::null() {
                if core::ptr::eq(s, b"work/\0".as_ptr() as *const c_char) {
                    levelshot = cgi_R_RegisterShaderNoMip(va(b"levelshots/%s\0".as_ptr() as *const c_char, s.offset(5)));
                }
            }
            // #endif
            if levelshot == 0 {
                levelshot = cgi_R_RegisterShaderNoMip(b"menu/art/unknownmap\0".as_ptr() as *const c_char);
            }
        }

        // if ( g_eSavedGameJustLoaded != eFULL && (!strcmp(s,"yavin1") || !strcmp(s,"demo")) )//special case for first map!
        // Placeholder check - simplified
        if core::ptr::eq(s, b"yavin1\0".as_ptr() as *const c_char) || core::ptr::eq(s, b"demo\0".as_ptr() as *const c_char) {
            let mut text: [c_char; 1024] = [0; 1024];

            //
            cgi_R_SetColor(core::addr_of!(colorTable[0]));  // CT_BLACK
            CG_DrawPic(0, 0, 640, 480, 0);  // SCREEN_WIDTH, SCREEN_HEIGHT, cgs.media.whiteShader

            cgi_SP_GetStringTextString(b"SP_INGAME_ALONGTIME\0".as_ptr() as *const c_char, text.as_mut_ptr(), core::mem::size_of_val(&text));

            let w: c_int = cgi_R_Font_StrLenPixels(text.as_ptr(), unsafe { (*core::addr_of!(cgs)).media.qhFontMedium }, 1.0);
            cgi_R_Font_DrawString(
                (320) - (w / 2),
                140,
                text.as_ptr(),
                core::addr_of!(colorTable[3]),  // CT_ICON_BLUE
                unsafe { (*core::addr_of!(cgs)).media.qhFontMedium },
                -1,
                1.0,
            );
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

            // #ifdef _DEBUG
            // cgi_R_Font_DrawString( 40, 416, va("LOADING ... %s",cg.infoScreenText),colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 1.0f );
            // #endif
        }

        // draw info string information

        y = 20;
        // map-specific message (long map name)
        let msg_s: *const c_char = CG_ConfigString(5);  // CS_MESSAGE

        if *msg_s != 0 {
            if *msg_s as c_char == b'@' as c_char {
                let mut text: [c_char; 1024] = [0; 1024];
                cgi_SP_GetStringTextString(msg_s.offset(1), text.as_mut_ptr(), core::mem::size_of_val(&text));
                cgi_R_Font_DrawString(
                    15,
                    y,
                    va(b"\"%s\"\0".as_ptr() as *const c_char, text.as_ptr()),
                    core::addr_of!(colorTable[7]),  // CT_WHITE
                    unsafe { (*core::addr_of!(cgs)).media.qhFontMedium },
                    -1,
                    1.0,
                );
            } else {
                cgi_R_Font_DrawString(
                    15,
                    y,
                    va(b"\"%s\"\0".as_ptr() as *const c_char, msg_s),
                    core::addr_of!(colorTable[7]),  // CT_WHITE
                    unsafe { (*core::addr_of!(cgs)).media.qhFontMedium },
                    -1,
                    1.0,
                );
            }
            y += 20;
        }
    }
}
