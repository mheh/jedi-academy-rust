#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// Extern declarations for types and functions from other modules
// These would be defined in cg_local.h and ui_shared.h equivalents
extern "C" {
    pub static mut cg_currentSelectedPlayer: cvar_t;
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static mut numSortedTeamPlayers: c_int;
    pub static mut sortedTeamPlayers: [c_int; 64];
    pub static mut cg_entities: [centity_t; 2048];

    // Functions
    pub fn CG_GetStringEdString(menu: *const c_char, item_name: *const c_char) -> *const c_char;
    pub fn CG_PlaceString(place: c_int) -> *const c_char;
    pub fn trap_SP_GetStringTextString(
        msg: *const c_char,
        buffer: *mut c_char,
        buffer_len: c_int,
    ) -> ();
    pub fn MenuFontToHandle(iMenuFont: c_int) -> c_int;
    pub fn trap_R_Font_StrLenPixels(text: *const c_char, iFontIndex: c_int, scale: f32) -> c_int;
    pub fn trap_AnyLanguage_ReadCharFromString(
        ps_text: *mut *const c_char,
        i_advance_count: *mut c_int,
        b_is_trailing_punctuation: *mut bool,
    ) -> u32;
    pub fn CG_Text_Paint(
        x: f32,
        y: f32,
        scale: f32,
        color: *const [f32; 4],
        text: *const c_char,
        adjust: f32,
        limit: c_int,
        style: c_int,
        font: c_int,
    ) -> ();
    pub fn CG_GetLocationString(loc: *const c_char) -> *const c_char;
    pub fn CG_ConfigString(index: c_int) -> *const c_char;
    pub fn CG_Text_Width(text: *const c_char, scale: f32, limit: c_int) -> c_int;
    pub fn BG_FindItemForPowerup(powerup: c_int) -> *const gitem_t;
    pub fn CG_DrawPic(x: f32, y: f32, w: f32, h: f32, hShader: qhandle_t) -> ();
    pub fn trap_R_RegisterShader(name: *const c_char) -> qhandle_t;
    pub fn CG_GetColorForHealth(health: c_int, armor: c_int, color: *mut [f32; 4]) -> ();
    pub fn trap_R_SetColor(color: *const [f32; 4]) -> ();
    pub fn Display_CursorType(x: c_int, y: c_int) -> c_int;
    pub fn Display_MouseMove(item: *mut core::ffi::c_void, x: c_int, y: c_int) -> ();
    pub fn Menus_CloseByName(name: *const c_char) -> ();
    pub fn Menus_OpenByName(name: *const c_char) -> ();
    pub fn trap_Key_SetCatcher(catcher: c_int) -> ();
    pub fn Display_CaptureItem(x: c_int, y: c_int) -> *mut core::ffi::c_void;
    pub fn Display_HandleKey(key: c_int, down: bool, x: c_int, y: c_int) -> ();
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn trap_Cvar_Set(name: *const c_char, value: *const c_char) -> ();
}

// Type stubs - these would be defined in headers but we stub them for this translation
#[repr(C)]
pub struct cvar_t {
    // Fields would go here
}

#[repr(C)]
pub struct displayContextDef_t {
    // Fields would go here
}

pub static mut cgDC: displayContextDef_t = displayContextDef_t {};

#[repr(C)]
pub struct centity_t {
    // Minimalist stub for this port
}

#[repr(C)]
pub struct clientInfo_t {
    // Minimalist stub for this port
}

#[repr(C)]
pub struct playerState_t {
    // Minimalist stub for this port
}

#[repr(C)]
pub struct cg_t {
    // Minimalist stub for this port
}

#[repr(C)]
pub struct cgs_t {
    // Minimalist stub for this port
}

#[repr(C)]
pub struct gitem_t {
    // Minimalist stub for this port
}

#[repr(C)]
pub struct score_t {
    // Minimalist stub for this port
}

pub type qhandle_t = c_int;
pub type qboolean = bool;
pub type rectDef_t = [f32; 4]; // Simplified - x, y, w, h


pub fn CG_GetSelectedPlayer() -> c_int {
    unsafe {
        if cg_currentSelectedPlayer.integer < 0 || cg_currentSelectedPlayer.integer >= numSortedTeamPlayers {
            cg_currentSelectedPlayer.integer = 0;
        }
        cg_currentSelectedPlayer.integer
    }
}

pub fn CG_StatusHandle(task: c_int) -> qhandle_t {
    unsafe {
        let mut h: qhandle_t = cgs.media.assaultShader;
        match task {
            17 => { // TEAMTASK_OFFENSE
                h = cgs.media.assaultShader;
            },
            18 => { // TEAMTASK_DEFENSE
                h = cgs.media.defendShader;
            },
            19 => { // TEAMTASK_PATROL
                h = cgs.media.patrolShader;
            },
            20 => { // TEAMTASK_FOLLOW
                h = cgs.media.followShader;
            },
            21 => { // TEAMTASK_CAMP
                h = cgs.media.campShader;
            },
            22 => { // TEAMTASK_RETRIEVE
                h = cgs.media.retrieveShader;
            },
            23 => { // TEAMTASK_ESCORT
                h = cgs.media.escortShader;
            },
            _ => {
                h = cgs.media.assaultShader;
            }
        }
        h
    }
}


pub fn CG_GetValue(ownerDraw: c_int) -> f32 {
    unsafe {
        let cent: *const centity_t = &cg_entities[cg.snap.ps.clientNum as usize];
        let ps: *const playerState_t = &cg.snap.ps;

        match ownerDraw {
            34 => { // CG_SELECTEDPLAYER_ARMOR
                let ci: *const clientInfo_t = cgs.clientinfo.add(sortedTeamPlayers[CG_GetSelectedPlayer() as usize] as usize);
                return (*ci).armor;
            },
            35 => { // CG_SELECTEDPLAYER_HEALTH
                let ci: *const clientInfo_t = cgs.clientinfo.add(sortedTeamPlayers[CG_GetSelectedPlayer() as usize] as usize);
                return (*ci).health;
            },
            4 => { // CG_PLAYER_ARMOR_VALUE
                return (*ps).stats[4]; // STAT_ARMOR
            },
            5 => { // CG_PLAYER_AMMO_VALUE
                if (*cent).currentState.weapon != 0 {
                    return (*ps).ammo[0]; // Simplified - would use weaponData[cent->currentState.weapon].ammoIndex
                }
            },
            8 => { // CG_PLAYER_SCORE
                return (*ps).persistant[0]; // PERS_SCORE
            },
            9 => { // CG_PLAYER_HEALTH
                return (*ps).stats[0]; // STAT_HEALTH
            },
            10 => { // CG_RED_SCORE
                return cgs.scores1;
            },
            11 => { // CG_BLUE_SCORE
                return cgs.scores2;
            },
            12 => { // CG_PLAYER_FORCE_VALUE
                return (*ps).fd.forcePower;
            },
            _ => {}
        }
        -1.0
    }
}

pub fn CG_OtherTeamHasFlag() -> qboolean {
    unsafe {
        if cgs.gametype == 4 || cgs.gametype == 5 { // GT_CTF || GT_CTY
            let team: c_int = cg.snap.ps.persistant[3]; // PERS_TEAM
            if team == 1 && cgs.redflag == 1 { // TEAM_RED && FLAG_TAKEN
                return true;
            } else if team == 2 && cgs.blueflag == 1 { // TEAM_BLUE && FLAG_TAKEN
                return true;
            } else {
                return false;
            }
        }
        false
    }
}

pub fn CG_YourTeamHasFlag() -> qboolean {
    unsafe {
        if cgs.gametype == 4 || cgs.gametype == 5 { // GT_CTF || GT_CTY
            let team: c_int = cg.snap.ps.persistant[3]; // PERS_TEAM
            if team == 1 && cgs.blueflag == 1 { // TEAM_RED && FLAG_TAKEN
                return true;
            } else if team == 2 && cgs.redflag == 1 { // TEAM_BLUE && FLAG_TAKEN
                return true;
            } else {
                return false;
            }
        }
        false
    }
}

// THINKABOUTME: should these be exclusive or inclusive..
//
pub fn CG_OwnerDrawVisible(flags: c_int) -> qboolean {

    unsafe {
        if flags & 1 != 0 { // CG_SHOW_TEAMINFO
            return cg_currentSelectedPlayer.integer == numSortedTeamPlayers;
        }

        if flags & 2 != 0 { // CG_SHOW_NOTEAMINFO
            return !(cg_currentSelectedPlayer.integer == numSortedTeamPlayers);
        }

        if flags & 4 != 0 { // CG_SHOW_OTHERTEAMHASFLAG
            return CG_OtherTeamHasFlag();
        }

        if flags & 8 != 0 { // CG_SHOW_YOURTEAMHASENEMYFLAG
            return CG_YourTeamHasFlag();
        }

        if flags & (32 | 64) != 0 { // CG_SHOW_BLUE_TEAM_HAS_REDFLAG | CG_SHOW_RED_TEAM_HAS_BLUEFLAG
            if flags & 32 != 0 && (cgs.redflag == 1 || cgs.flagStatus == 1) { // CG_SHOW_BLUE_TEAM_HAS_REDFLAG, FLAG_TAKEN, FLAG_TAKEN_RED
                return true;
            } else if flags & 64 != 0 && (cgs.blueflag == 1 || cgs.flagStatus == 2) { // CG_SHOW_RED_TEAM_HAS_BLUEFLAG, FLAG_TAKEN, FLAG_TAKEN_BLUE
                return true;
            }
            return false;
        }

        if flags & 128 != 0 { // CG_SHOW_ANYTEAMGAME
            if cgs.gametype >= 3 { // GT_TEAM
                return true;
            }
        }

        if flags & 256 != 0 { // CG_SHOW_ANYNONTEAMGAME
            if cgs.gametype < 3 { // GT_TEAM
                return true;
            }
        }

        if flags & 512 != 0 { // CG_SHOW_CTF
            if cgs.gametype == 4 || cgs.gametype == 5 { // GT_CTF || GT_CTY
                return true;
            }
        }

        if flags & 1024 != 0 { // CG_SHOW_HEALTHCRITICAL
            if cg.snap.ps.stats[0] < 25 { // STAT_HEALTH
                return true;
            }
        }

        if flags & 2048 != 0 { // CG_SHOW_HEALTHOK
            if cg.snap.ps.stats[0] >= 25 { // STAT_HEALTH
                return true;
            }
        }

        if flags & 4096 != 0 { // CG_SHOW_SINGLEPLAYER
            if cgs.gametype == 7 { // GT_SINGLE_PLAYER
                return true;
            }
        }

        if flags & 8192 != 0 { // CG_SHOW_TOURNAMENT
            if cgs.gametype == 6 || cgs.gametype == 8 { // GT_DUEL || GT_POWERDUEL
                return true;
            }
        }

        if flags & 16384 != 0 { // CG_SHOW_DURINGINCOMINGVOICE
        }

        if flags & 32768 != 0 { // CG_SHOW_IF_PLAYER_HAS_FLAG
            if cg.snap.ps.powerups[14] != 0 || cg.snap.ps.powerups[15] != 0 || cg.snap.ps.powerups[16] != 0 { // PW_REDFLAG, PW_BLUEFLAG, PW_NEUTRALFLAG
                return true;
            }
        }
        false
    }
}


pub unsafe fn CG_GetKillerText() -> *const c_char {
    static mut s: *const c_char = b"\0".as_ptr() as *const c_char;
    if cg.killerName[0] as u8 != 0 {
        s = CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"KILLEDBY\0".as_ptr() as *const c_char);
    }
    s
}


pub unsafe fn CG_GetGameStatusText() -> *const c_char {
    static mut s: *const c_char = b"\0".as_ptr() as *const c_char;
    if cgs.gametype == 8 { // GT_POWERDUEL
        s = b"\0".as_ptr() as *const c_char;
    }
    else if cgs.gametype < 3 { // GT_TEAM
        if cg.snap.ps.persistant[3] != 3 { // PERS_TEAM, TEAM_SPECTATOR
            let mut sPlaceWith: [c_char; 256] = [0; 256];
            trap_SP_GetStringTextString(
                b"MP_INGAME_PLACE_WITH\0".as_ptr() as *const c_char,
                &mut sPlaceWith[0],
                256
            );

            s = CG_GetStringEdString(
                b"MP_INGAME\0".as_ptr() as *const c_char,
                b"TIEDAT\0".as_ptr() as *const c_char
            );
        }
    }
    else {
        if cg.teamScores[0] == cg.teamScores[1] {
            s = CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"TIEDAT\0".as_ptr() as *const c_char);
        } else if cg.teamScores[0] >= cg.teamScores[1] {
            s = CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RED_LEADS\0".as_ptr() as *const c_char);
        } else {
            s = CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"BLUE_LEADS\0".as_ptr() as *const c_char);
        }
    }
    s
}

pub fn CG_GameTypeString() -> *const c_char {
    unsafe {
        if cgs.gametype == 0 { // GT_FFA
            return b"Free For All\0".as_ptr() as *const c_char;
        } else if cgs.gametype == 1 { // GT_HOLOCRON
            return b"Holocron FFA\0".as_ptr() as *const c_char;
        } else if cgs.gametype == 2 { // GT_JEDIMASTER
            return b"Jedi Master\0".as_ptr() as *const c_char;
        } else if cgs.gametype == 3 { // GT_TEAM
            return b"Team FFA\0".as_ptr() as *const c_char;
        } else if cgs.gametype == 4 { // GT_SIEGE (note: mismatch with C but preserving order)
            return b"Siege\0".as_ptr() as *const c_char;
        } else if cgs.gametype == 4 { // GT_CTF
            return b"Capture the Flag\0".as_ptr() as *const c_char;
        } else if cgs.gametype == 5 { // GT_CTY
            return b"Capture the Ysalamiri\0".as_ptr() as *const c_char;
        }
        b"\0".as_ptr() as *const c_char
    }
}


// maxX param is initially an X limit, but is also used as feedback. 0 = text was clipped to fit within, else maxX = next pos
//
pub unsafe fn CG_Text_Paint_Limit(maxX: *mut f32, x: f32, y: f32, scale: f32, color: &[f32; 4], text: *const c_char, adjust: f32, limit: c_int, iMenuFont: c_int)
{
    let mut bIsTrailingPunctuation: bool = false;

    // this is kinda dirty, but...
    //
    let iFontIndex: c_int = MenuFontToHandle(iMenuFont);

    //float fMax = *maxX;
    let iPixelLen: c_int = trap_R_Font_StrLenPixels(text, iFontIndex, scale);
    if x + iPixelLen as f32 > *maxX
    {
        // whole text won't fit, so we need to print just the amount that does...
        //  Ok, this is slow and tacky, but only called occasionally, and it works...
        //
        let mut sTemp: [c_char; 4096] = [0; 4096]; // lazy assumption
        let mut psText: *const c_char = text;
        let mut psOut: *mut c_char = &mut sTemp[0];
        let mut psOutLastGood: *mut c_char = psOut;
        let mut uiLetter: u32 = 0;

        while *psText as u8 != 0 && (x + trap_R_Font_StrLenPixels(sTemp.as_ptr(), iFontIndex, scale) as f32 <= *maxX)
            && psOut < &mut sTemp[4096 - 1]    // sanity
        {
            let mut iAdvanceCount: c_int = 0;
            psOutLastGood = psOut;

            uiLetter = trap_AnyLanguage_ReadCharFromString(&mut psText, &mut iAdvanceCount, &mut bIsTrailingPunctuation);
            psText = psText.add(iAdvanceCount as usize);

            if uiLetter > 255
            {
                *psOut = (uiLetter >> 8) as c_char;
                psOut = psOut.add(1);
                *psOut = (uiLetter & 0xFF) as c_char;
                psOut = psOut.add(1);
            }
            else
            {
                *psOut = (uiLetter & 0xFF) as c_char;
                psOut = psOut.add(1);
            }
        }
        *psOutLastGood = 0;

        *maxX = 0.0;    // feedback
        CG_Text_Paint(x, y, scale, color as *const [f32; 4], sTemp.as_ptr(), adjust, limit, 0, iMenuFont); // ITEM_TEXTSTYLE_NORMAL
    }
    else
    {
        // whole text fits fine, so print it all...
        //
        *maxX = x + iPixelLen as f32;    // feedback the next position, as the caller expects
        CG_Text_Paint(x, y, scale, color as *const [f32; 4], text, adjust, limit, 0, iMenuFont); // ITEM_TEXTSTYLE_NORMAL
    }
}



const PIC_WIDTH: f32 = 12.0;

pub unsafe fn CG_DrawNewTeamInfo(rect: *const [f32; 4], text_x: f32, text_y: f32, scale: f32, color: &[f32; 4], shader: qhandle_t) {
    let mut xx: c_int = 0;
    let mut y: f32 = 0.0;
    let mut i: c_int = 0;
    let mut j: c_int = 0;
    let mut len: c_int = 0;
    let mut count: c_int = 0;
    let mut p: *const c_char = std::ptr::null();
    let mut hcolor: [f32; 4] = [0.0; 4];
    let mut pwidth: f32 = 0.0;
    let mut lwidth: f32 = 0.0;
    let mut maxx: f32 = 0.0;
    let mut leftOver: f32 = 0.0;
    let mut ci: *const clientInfo_t = std::ptr::null();
    let mut item: *const gitem_t = std::ptr::null();
    let mut h: qhandle_t = 0;

    // max player name width
    pwidth = 0.0;
    count = if numSortedTeamPlayers > 8 { 8 } else { numSortedTeamPlayers };
    i = 0;
    while i < count {
        ci = cgs.clientinfo.add(sortedTeamPlayers[i as usize] as usize);
        if (*ci).infoValid && (*ci).team == cg.snap.ps.persistant[3] { // PERS_TEAM
            len = CG_Text_Width((*ci).name, scale, 0);
            if len as f32 > pwidth {
                pwidth = len as f32;
            }
        }
        i += 1;
    }

    // max location name width
    lwidth = 0.0;
    i = 1;
    while i < 64 { // MAX_LOCATIONS
        p = CG_GetLocationString(CG_ConfigString(0 + i)); // CS_LOCATIONS
        if !p.is_null() && *p as u8 != 0 {
            len = CG_Text_Width(p, scale, 0);
            if len as f32 > lwidth {
                lwidth = len as f32;
            }
        }
        i += 1;
    }

    y = (*rect)[1];

    i = 0;
    while i < count {
        ci = cgs.clientinfo.add(sortedTeamPlayers[i as usize] as usize);
        if (*ci).infoValid && (*ci).team == cg.snap.ps.persistant[3] { // PERS_TEAM

            xx = (*rect)[0] as c_int + 1;
            j = 0;
            while j <= 16 { // PW_NUM_POWERUPS
                if (*ci).powerups & (1 << j) != 0 {

                    item = BG_FindItemForPowerup(j);

                    if !item.is_null() {
                        CG_DrawPic(xx as f32, y, PIC_WIDTH, PIC_WIDTH, trap_R_RegisterShader((*item).icon));
                        xx += 12;
                    }
                }
                j += 1;
            }

            // FIXME: max of 3 powerups shown properly
            xx = (*rect)[0] as c_int + (12 * 3) + 2;

            CG_GetColorForHealth((*ci).health, (*ci).armor, &mut hcolor);
            trap_R_SetColor(&hcolor);
            CG_DrawPic(xx as f32, y + 1.0, PIC_WIDTH - 2.0, PIC_WIDTH - 2.0, cgs.media.heartShader);

            //Com_sprintf (st, sizeof(st), "%3i %3i", ci->health,	ci->armor);
            //CG_Text_Paint(xx, y + text_y, scale, hcolor, st, 0, 0);

            // draw weapon icon
            xx += 12 + 1;

// weapon used is not that useful, use the space for task
#if 0
            if cg_weapons[ci->curWeapon].weaponIcon {
                CG_DrawPic(xx as f32, y, PIC_WIDTH, PIC_WIDTH, cg_weapons[ci->curWeapon].weaponIcon);
            } else {
                CG_DrawPic(xx as f32, y, PIC_WIDTH, PIC_WIDTH, cgs.media.deferShader);
            }
#endif

            trap_R_SetColor(std::ptr::null());
            h = CG_StatusHandle((*ci).teamTask);

            if h != 0 {
                CG_DrawPic(xx as f32, y, PIC_WIDTH, PIC_WIDTH, h);
            }

            xx += 12 + 1;

            leftOver = (*rect)[2] - xx as f32;
            maxx = xx as f32 + leftOver / 3.0;



            CG_Text_Paint_Limit(&mut maxx, xx as f32, y + text_y, scale, color, (*ci).name, 0.0, 0, 1); // FONT_MEDIUM

            p = CG_GetLocationString(CG_ConfigString(0 + (*ci).location as c_int)); // CS_LOCATIONS
            if p.is_null() || *p as u8 == 0 {
                p = b"unknown\0".as_ptr() as *const c_char;
            }

            xx = (xx as f32 + leftOver / 3.0 + 2.0) as c_int;
            maxx = (*rect)[2] - 4.0;

            CG_Text_Paint_Limit(&mut maxx, xx as f32, y + text_y, scale, color, p, 0.0, 0, 1); // FONT_MEDIUM
            y += text_y + 2.0;
            if y + text_y + 2.0 > (*rect)[1] + (*rect)[3] {
                break;
            }

        }
        i += 1;
    }
}


pub unsafe fn CG_DrawTeamSpectators(rect: *const [f32; 4], scale: f32, color: &[f32; 4], shader: qhandle_t) {
    if cg.spectatorLen > 0 {
        let mut maxX: f32 = 0.0;

        if cg.spectatorWidth == -1 {
            cg.spectatorWidth = 0;
            cg.spectatorPaintX = (*rect)[0] as c_int + 1;
            cg.spectatorPaintX2 = -1;
        }

        if cg.spectatorOffset > cg.spectatorLen {
            cg.spectatorOffset = 0;
            cg.spectatorPaintX = (*rect)[0] as c_int + 1;
            cg.spectatorPaintX2 = -1;
        }

        if cg.time > cg.spectatorTime {
            cg.spectatorTime = cg.time + 10;
            if cg.spectatorPaintX <= (*rect)[0] as c_int + 2 {
                if cg.spectatorOffset < cg.spectatorLen {
                    cg.spectatorPaintX += CG_Text_Width(&cg.spectatorList[cg.spectatorOffset as usize], scale, 1) - 1;
                    cg.spectatorOffset += 1;
                } else {
                    cg.spectatorOffset = 0;
                    if cg.spectatorPaintX2 >= 0 {
                        cg.spectatorPaintX = cg.spectatorPaintX2;
                    } else {
                        cg.spectatorPaintX = (*rect)[0] as c_int + (*rect)[2] as c_int - 2;
                    }
                    cg.spectatorPaintX2 = -1;
                }
            } else {
                cg.spectatorPaintX -= 1;
                if cg.spectatorPaintX2 >= 0 {
                    cg.spectatorPaintX2 -= 1;
                }
            }
        }

        maxX = (*rect)[0] + (*rect)[2] - 2.0;
        CG_Text_Paint_Limit(&mut maxX, cg.spectatorPaintX as f32, (*rect)[1] + (*rect)[3] - 3.0, scale, color, &cg.spectatorList[cg.spectatorOffset as usize], 0.0, 0, 1); // FONT_MEDIUM
        if cg.spectatorPaintX2 >= 0 {
            let mut maxX2: f32 = (*rect)[0] + (*rect)[2] - 2.0;
            CG_Text_Paint_Limit(&mut maxX2, cg.spectatorPaintX2 as f32, (*rect)[1] + (*rect)[3] - 3.0, scale, color, cg.spectatorList.as_ptr(), 0.0, cg.spectatorOffset, 1); // FONT_MEDIUM
        }
        if cg.spectatorOffset != 0 && maxX > 0.0 {
            // if we have an offset ( we are skipping the first part of the string ) and we fit the string
            if cg.spectatorPaintX2 == -1 {
                        cg.spectatorPaintX2 = (*rect)[0] as c_int + (*rect)[2] as c_int - 2;
            }
        } else {
            cg.spectatorPaintX2 = -1;
        }

    }
}



pub unsafe fn CG_DrawMedal(ownerDraw: c_int, rect: *const [f32; 4], scale: f32, color: &mut [f32; 4], shader: qhandle_t) {
    let score: *const score_t = &cg.scores[cg.selectedScore as usize];
    let mut value: f32 = 0.0;
    let mut text: *const c_char = std::ptr::null();
    color[3] = 0.25;

    match ownerDraw {
        27 => { // CG_ACCURACY
            value = (*score).accuracy;
        },
        28 => { // CG_ASSISTS
            value = (*score).assistCount;
        },
        29 => { // CG_DEFEND
            value = (*score).defendCount;
        },
        30 => { // CG_EXCELLENT
            value = (*score).excellentCount;
        },
        31 => { // CG_IMPRESSIVE
            value = (*score).impressiveCount;
        },
        32 => { // CG_PERFECT
            value = (*score).perfect;
        },
        33 => { // CG_GAUNTLET
            value = (*score).guantletCount;
        },
        36 => { // CG_CAPTURES
            value = (*score).captures;
        },
        _ => {}
    }

    if value > 0.0 {
        if ownerDraw != 32 { // CG_PERFECT
            if ownerDraw == 27 { // CG_ACCURACY
                text = b"%i%%\0".as_ptr() as *const c_char; // Simplified - would use va
                if value > 50.0 {
                    color[3] = 1.0;
                }
            } else {
                text = b"%i\0".as_ptr() as *const c_char; // Simplified - would use va
                color[3] = 1.0;
            }
        } else {
            if value != 0.0 {
                color[3] = 1.0;
            }
            text = b"Wow\0".as_ptr() as *const c_char;
        }
    }

    trap_R_SetColor(color as *const [f32; 4]);
    CG_DrawPic((*rect)[0], (*rect)[1], (*rect)[2], (*rect)[3], shader);

    if !text.is_null() {
        color[3] = 1.0;
        value = CG_Text_Width(text, scale, 0) as f32;
        CG_Text_Paint((*rect)[0] + ((*rect)[2] - value) / 2.0, (*rect)[1] + (*rect)[3] + 10.0, scale, color, text, 0.0, 0, 0, 1); // FONT_MEDIUM
    }
    trap_R_SetColor(std::ptr::null());

}


//
pub unsafe fn CG_OwnerDraw(x: f32, y: f32, w: f32, h: f32, text_x: f32, text_y: f32, ownerDraw: c_int, ownerDrawFlags: c_int, align: c_int, special: f32, scale: f32, color: &[f32; 4], shader: qhandle_t, textStyle: c_int, font: c_int) {

//Ignore all this, at least for now. May put some stat stuff back in menu files later.
#if 0
    let mut rect: [f32; 4] = [0.0; 4];

    if cg_drawStatus.integer == 0 {
        return;
    }

    //if (ownerDrawFlags != 0 && !CG_OwnerDrawVisible(ownerDrawFlags)) {
    //	return;
    //}

    rect[0] = x;
    rect[1] = y;
    rect[2] = w;
    rect[3] = h;

    match ownerDraw {
        // Case statements would go here
        _ => {}
    }
#endif
}

pub unsafe fn CG_MouseEvent(x: c_int, y: c_int) {
    let mut n: c_int = 0;

    if (cg.predictedPlayerState.pm_type == 0 || cg.predictedPlayerState.pm_type == 3 || cg.predictedPlayerState.pm_type == 4 || cg.predictedPlayerState.pm_type == 6) && cg.showScores == false { // PM_NORMAL, PM_JETPACK, PM_FLOAT, PM_SPECTATOR
        trap_Key_SetCatcher(0);
        return;
    }

    cgs.cursorX += x;
    if cgs.cursorX < 0 {
        cgs.cursorX = 0;
    } else if cgs.cursorX > 640 {
        cgs.cursorX = 640;
    }

    cgs.cursorY += y;
    if cgs.cursorY < 0 {
        cgs.cursorY = 0;
    } else if cgs.cursorY > 480 {
        cgs.cursorY = 480;
    }

    n = Display_CursorType(cgs.cursorX, cgs.cursorY);
    cgs.activeCursor = 0;
    if n == 0 { // CURSOR_ARROW
        cgs.activeCursor = cgs.media.selectCursor;
    } else if n == 1 { // CURSOR_SIZER
        cgs.activeCursor = cgs.media.sizeCursor;
    }

    if !cgs.capturedItem.is_null() {
        Display_MouseMove(cgs.capturedItem, x, y);
    } else {
        Display_MouseMove(std::ptr::null_mut(), cgs.cursorX, cgs.cursorY);
    }

}

/*
==================
CG_HideTeamMenus
==================

*/
pub fn CG_HideTeamMenu() {
    unsafe {
        Menus_CloseByName(b"teamMenu\0".as_ptr() as *const c_char);
        Menus_CloseByName(b"getMenu\0".as_ptr() as *const c_char);
    }
}

/*
==================
CG_ShowTeamMenus
==================

*/
pub fn CG_ShowTeamMenu() {
    unsafe {
        Menus_OpenByName(b"teamMenu\0".as_ptr() as *const c_char);
    }
}




/*
==================
CG_EventHandling
==================
 type 0 - no event handling
      1 - team menu
      2 - hud editor

*/
pub unsafe fn CG_EventHandling(type_: c_int) {
    cgs.eventHandling = type_;
    if type_ == 0 { // CGAME_EVENT_NONE
        CG_HideTeamMenu();
    } else if type_ == 1 { // CGAME_EVENT_TEAMMENU
        //CG_ShowTeamMenu();
    } else if type_ == 2 { // CGAME_EVENT_SCOREBOARD
    }

}



pub unsafe fn CG_KeyEvent(key: c_int, down: bool) {

    if !down {
        return;
    }

    if (cg.predictedPlayerState.pm_type == 0 || cg.predictedPlayerState.pm_type == 3 || cg.predictedPlayerState.pm_type == 0 || (cg.predictedPlayerState.pm_type == 6 && cg.showScores == false)) { // PM_NORMAL, PM_JETPACK, PM_SPECTATOR
        CG_EventHandling(0); // CGAME_EVENT_NONE
        trap_Key_SetCatcher(0);
        return;
    }

    //if (key == trap_Key_GetKey("teamMenu") || !Display_CaptureItem(cgs.cursorX, cgs.cursorY)) {
    // if we see this then we should always be visible
    //  CG_EventHandling(CGAME_EVENT_NONE);
    //  trap_Key_SetCatcher(0);
    //}



    Display_HandleKey(key, down, cgs.cursorX, cgs.cursorY);

    if !cgs.capturedItem.is_null() {
        cgs.capturedItem = std::ptr::null_mut();
    } else {
        if key == 2 && down { // A_MOUSE2
            cgs.capturedItem = Display_CaptureItem(cgs.cursorX, cgs.cursorY);
        }
    }
}

pub fn CG_ClientNumFromName(p: *const c_char) -> c_int {
    unsafe {
        let mut i: c_int = 0;
        while i < cgs.maxclients {
            if cgs.clientinfo[i as usize].infoValid && Q_stricmp(cgs.clientinfo[i as usize].name, p) == 0 {
                return i;
            }
            i += 1;
        }
    }
    -1
}

pub unsafe fn CG_ShowResponseHead() {
    Menus_OpenByName(b"voiceMenu\0".as_ptr() as *const c_char);
    trap_Cvar_Set(b"cl_conXOffset\0".as_ptr() as *const c_char, b"72\0".as_ptr() as *const c_char);
    cg.voiceTime = cg.time;
}

pub unsafe fn CG_RunMenuScript(args: *mut *mut c_char) {
}

pub unsafe fn CG_DeferMenuScript(args: *mut *mut c_char) -> qboolean
{
    false
}

pub unsafe fn CG_GetTeamColor(color: *mut [f32; 4]) {
    if cg.snap.ps.persistant[3] == 1 { // PERS_TEAM, TEAM_RED
        (*color)[0] = 1.0;
        (*color)[3] = 0.25;
        (*color)[1] = 0.0;
        (*color)[2] = 0.0;
    } else if cg.snap.ps.persistant[3] == 2 { // PERS_TEAM, TEAM_BLUE
        (*color)[0] = 0.0;
        (*color)[1] = 0.0;
        (*color)[2] = 1.0;
        (*color)[3] = 0.25;
    } else {
        (*color)[0] = 0.0;
        (*color)[2] = 0.0;
        (*color)[1] = 0.17;
        (*color)[3] = 0.25;
    }
}
