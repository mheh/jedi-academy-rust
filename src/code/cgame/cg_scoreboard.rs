// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"

// #include "cg_local.h"
// #include "cg_media.h"
// #include "..\game\objectives.h"
// #include "..\game\b_local.h"

use core::ffi::c_char;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

const SCOREBOARD_WIDTH: usize = 26; // * BIGCHAR_WIDTH

// External C functions
extern "C" {
    fn cgi_UI_SetActive_Menu(name: *const c_char);
}

// Stub types for globals defined elsewhere in the codebase
#[repr(C)]
pub struct playerState_t {
    pub pm_type: i32,
    // ... other fields not used here
}

#[repr(C)]
pub struct cg_t {
    pub missionFailedScreen: i32,
    // Approximate field ordering for the fields we access
    pub predicted_player_state: playerState_t,
    pub missionStatusDeadTime: i32,
    pub missionStatusShow: i32,
    // ... other fields
}

#[repr(C)]
pub struct level_t {
    pub time: i32,
    // ... other fields
}

#[repr(C)]
pub struct cvar_t {
    pub integer: i32,
    // ... other fields
}

#[repr(C)]
pub struct gameImport_t {
    // Stub - actual structure defined elsewhere
    _unused: [u8; 0],
}

// Global variables defined elsewhere
extern "C" {
    pub static mut cg: cg_t;
    pub static mut level: level_t;
    pub static mut cg_paused: cvar_t;
    pub static mut gi: gameImport_t;
}

// External wrapper for gi.cvar_set
extern "C" {
    fn gi_cvar_set(key: *const c_char, value: *const c_char);
}

// =================
// CG_MissionFailed
// =================

pub static mut statusTextIndex: i32 = -1;

pub unsafe fn CG_MissionFailed() {
    let text: *const c_char;

    if (*addr_of_mut!(cg)).missionFailedScreen == 0 {
        cgi_UI_SetActive_Menu(b"missionfailed_menu\0".as_ptr() as *const c_char);
        (*addr_of_mut!(cg)).missionFailedScreen = 1; // qtrue

        match statusTextIndex {
            -1 => {
                // Our HERO DIED!!!
                text = b"@SP_INGAME_MISSIONFAILED_PLAYER\0".as_ptr() as *const c_char;
            }
            0 => {
                // MISSIONFAILED_JAN
                text = b"@SP_INGAME_MISSIONFAILED_JAN\0".as_ptr() as *const c_char;
            }
            1 => {
                // MISSIONFAILED_LUKE
                text = b"@SP_INGAME_MISSIONFAILED_LUKE\0".as_ptr() as *const c_char;
            }
            2 => {
                // MISSIONFAILED_LANDO
                text = b"@SP_INGAME_MISSIONFAILED_LANDO\0".as_ptr() as *const c_char;
            }
            3 => {
                // MISSIONFAILED_R5D2
                text = b"@SP_INGAME_MISSIONFAILED_R5D2\0".as_ptr() as *const c_char;
            }
            4 => {
                // MISSIONFAILED_WARDEN
                text = b"@SP_INGAME_MISSIONFAILED_WARDEN\0".as_ptr() as *const c_char;
            }
            5 => {
                // MISSIONFAILED_PRISONERS
                text = b"@SP_INGAME_MISSIONFAILED_PRISONERS\0".as_ptr() as *const c_char;
            }
            6 => {
                // MISSIONFAILED_EMPLACEDGUNS
                text = b"@SP_INGAME_MISSIONFAILED_EMPLACEDGUNS\0".as_ptr() as *const c_char;
            }
            7 => {
                // MISSIONFAILED_LADYLUCK
                text = b"@SP_INGAME_MISSIONFAILED_LADYLUCK\0".as_ptr() as *const c_char;
            }
            8 => {
                // MISSIONFAILED_KYLECAPTURE
                text = b"@SP_INGAME_MISSIONFAILED_KYLECAPTURE\0".as_ptr() as *const c_char;
            }
            9 => {
                // MISSIONFAILED_TOOMANYALLIESDIED
                text = b"@SP_INGAME_MISSIONFAILED_TOOMANYALLIESDIED\0".as_ptr() as *const c_char;
            }
            10 => {
                // MISSIONFAILED_CHEWIE
                text = b"@SP_INGAME_MISSIONFAILED_CHEWIE\0".as_ptr() as *const c_char;
            }
            11 => {
                // MISSIONFAILED_KYLE
                text = b"@SP_INGAME_MISSIONFAILED_KYLE\0".as_ptr() as *const c_char;
            }
            12 => {
                // MISSIONFAILED_ROSH
                text = b"@SP_INGAME_MISSIONFAILED_ROSH\0".as_ptr() as *const c_char;
            }
            13 => {
                // MISSIONFAILED_WEDGE
                text = b"@SP_INGAME_MISSIONFAILED_WEDGE\0".as_ptr() as *const c_char;
            }
            14 => {
                // MISSIONFAILED_TURNED
                text = b"@SP_INGAME_MISSIONFAILED_TURNED\0".as_ptr() as *const c_char;
            }
            _ => {
                text = b"@SP_INGAME_MISSIONFAILED_UNKNOWN\0".as_ptr() as *const c_char;
            }
        }

        gi_cvar_set(
            b"ui_missionfailed_text\0".as_ptr() as *const c_char,
            text,
        );
    }
    // w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontMedium, 1.2f);
    // cgi_R_Font_DrawString(320 - w/2, y+30, text, colorTable[CT_HUD_RED], cgs.media.qhFontMedium, -1, 1.2f);

    // cgi_SP_GetStringTextString( "SP_INGAME_RELOADMISSION", text, sizeof(text) );
    // w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 1.0f);
    // cgi_R_Font_DrawString(320 - w/2, 450, text, colorTable[CT_CYAN], cgs.media.qhFontSmall, -1, 1.0f);
}

/*
=================
CG_MissionCompletion
=================
#if 0
/*
void CG_MissionCompletion(void)
{
	char text[1024]={0};
	int w,x,y;
	const int pad = 18;

	cgi_SP_GetStringTextString( "SP_INGAME_MISSIONCOMPLETION", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontMedium, 1.2f);
	cgi_R_Font_DrawString(320 - w/2, 53, text, colorTable[CT_LTGOLD1], cgs.media.qhFontMedium, -1, 1.2f);

	x = 75;
	y =86;
	cgi_SP_GetStringTextString( "SP_INGAME_SECRETAREAS", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,    y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_SP_GetStringTextString( "SP_INGAME_SECRETAREAS_OF", text, sizeof(text) );
	cgi_R_Font_DrawString(x+w,  y, va("%d %s %d",
										cg_entities[0].gent->client->sess.missionStats.secretsFound,
										text,
										cg_entities[0].gent->client->sess.missionStats.totalSecrets
										),
							colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_ENEMIESKILLED", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x, y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w,y, va("%d",cg_entities[0].gent->client->sess.missionStats.enemiesKilled), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);
	/*
	cgi_SP_GetStringTextString( "SP_INGAME_SECRETAREAS_OF", text, sizeof(text) );
	cgi_R_Font_DrawString(x+w,y, va("%d %s %d",
										cg_entities[0].gent->client->sess.missionStats.enemiesKilled,
										text,
										cg_entities[0].gent->client->sess.missionStats.enemiesSpawned
										),
							colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);
	*/

	y +=pad;
	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_FAVORITEWEAPON", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x, y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);

	int wpn=0,i;
	int max_wpn = cg_entities[0].gent->client->sess.missionStats.weaponUsed[0];
	for (i = 1; i<WP_NUM_WEAPONS; i++)
	{
		if (cg_entities[0].gent->client->sess.missionStats.weaponUsed[i] > max_wpn)
		{
			max_wpn = cg_entities[0].gent->client->sess.missionStats.weaponUsed[i];
			wpn = i;
		}
	}

	if ( wpn )
	{
		gitem_t	*wItem= FindItemForWeapon( (weapon_t)wpn);
		cgi_SP_GetStringTextString( va("SP_INGAME_%s",wItem->classname ), text, sizeof( text ));
	//	cgi_R_Font_DrawString(x+w, y, va("%d",wpn), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);
		cgi_R_Font_DrawString(x+w, y, text, colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);
	}

	x = 334+70;
	y = 86;
	cgi_SP_GetStringTextString( "SP_INGAME_SHOTSFIRED", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x, y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.shotsFired), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);


	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_HITS", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x, y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.hits), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);


	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_ACCURACY", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x, y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	const float percent = cg_entities[0].gent->client->sess.missionStats.shotsFired? 100.0f * (float)cg_entities[0].gent->client->sess.missionStats.hits / cg_entities[0].gent->client->sess.missionStats.shotsFired : 0;
	cgi_R_Font_DrawString(x+w, y, va("%.2f%%",percent), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	if ( cg_entities[0].gent->client->sess.missionStats.weaponUsed[WP_SABER] <= 0 )
	{
		return; //don't have saber yet, so don't print any stats
	}
//first column, FORCE POWERS
	y =180;
	cgi_SP_GetStringTextString( "SP_INGAME_FORCEUSE", text, sizeof(text) );
	cgi_R_Font_DrawString(x, y, text, colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_HEAL", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.forceUsed[FP_HEAL]), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_SPEED", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.forceUsed[FP_SPEED]), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_PULL", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.forceUsed[FP_PULL]), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_PUSH", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.forceUsed[FP_PUSH]), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString("SP_INGAME_MINDTRICK", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.forceUsed[FP_TELEPATHY]), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_GRIP", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.forceUsed[FP_GRIP]), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_LIGHTNING", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.forceUsed[FP_LIGHTNING]), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

//second column, LIGHT SABER
	y = 180;
	x = 140;
	cgi_SP_GetStringTextString( "SP_INGAME_LIGHTSABERUSE", text, sizeof(text) );
	cgi_R_Font_DrawString(x, y, text, colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_THROWN", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.saberThrownCnt), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_BLOCKS", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.saberBlocksCnt), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_LEGATTACKS", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.legAttacksCnt), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_ARMATTACKS", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.armAttacksCnt), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_BODYATTACKS", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.torsoAttacksCnt), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);

	y +=pad;
	cgi_SP_GetStringTextString( "SP_INGAME_OTHERATTACKS", text, sizeof(text) );
w = cgi_R_Font_StrLenPixels(text, cgs.media.qhFontSmall, 0.8f);
	cgi_R_Font_DrawString(x,   y, text, colorTable[CT_LTGOLD1], cgs.media.qhFontSmall, -1, 0.8f);
	cgi_R_Font_DrawString(x+w, y, va("%d",cg_entities[0].gent->client->sess.missionStats.otherAttacksCnt), colorTable[CT_WHITE], cgs.media.qhFontSmall, -1, 0.8f);
}
*/
#endif
*/

// =================
// CG_DrawScoreboard
//
// Draw the normal in-game scoreboard
// return value is bool to NOT draw centerstring
// =================

pub fn CG_DrawScoreboard() -> bool {
    // don't draw anything if the menu is up
    if unsafe { (*addr_of!(cg_paused)).integer != 0 } {
        return false;
    }

    // Character is either dead, or a script has brought up the screen
    if (unsafe {
        ((*addr_of!(cg)).predicted_player_state.pm_type == 4) // PM_DEAD
            && ((*addr_of!(cg)).missionStatusDeadTime < (*addr_of!(level)).time)
    }) || (unsafe { (*addr_of!(cg)).missionStatusShow != 0 })
    {
        unsafe {
            CG_MissionFailed();
        }
        return true;
    }

    false
}

pub fn ScoreBoardReset() {}

// ================================================================================
