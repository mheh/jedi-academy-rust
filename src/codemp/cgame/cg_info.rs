// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_info.c -- display information while data is being loading

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of_mut};

const MAX_LOADING_PLAYER_ICONS: usize = 16;
const MAX_LOADING_ITEM_ICONS: usize = 26;

// Stubs for external functions from other modules
extern "C" {
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    fn trap_UpdateScreen();
    fn CG_GetStringEdString(table: *const c_char, key: *const c_char) -> *const c_char;
    fn CG_ConfigString(index: c_int) -> *const c_char;
    fn Q_strupr(s: *mut c_char) -> *mut c_char;
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    fn Q_CleanStr(string: *mut c_char);
    fn trap_R_RegisterShaderNoMip(name: *const c_char) -> c_int;
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn trap_Cvar_VariableStringBuffer(
        var_name: *const c_char,
        buffer: *mut c_char,
        bufsize: usize,
    );
    fn trap_R_SetColor(rgba: *const c_void);
    fn CG_DrawPic(x: c_int, y: c_int, width: c_int, height: c_int, hshader: c_int);
    fn UI_DrawProportionalString(
        x: c_int,
        y: c_int,
        s: *const c_char,
        style: c_int,
        color: *const c_void,
    );
    fn trap_SP_GetStringTextString(text_label: *const c_char, buffer: *mut c_char, bufsize: usize);
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn atoi(str: *const c_char) -> c_int;
    fn va(fmt: *const c_char, ...) -> *const c_char;
}

// Stub constants from engine/game headers - these come from external compilation units
// CS_PLAYERS and similar are indices into a config string array
const CS_PLAYERS: c_int = 0;      // stub - actual value from qcommon.h
const CS_SERVERINFO: c_int = 0;   // stub - actual value from qcommon.h
const CS_SYSTEMINFO: c_int = 1;   // stub - actual value from qcommon.h
const CS_MOTD: c_int = 2;         // stub - actual value from qcommon.h
const CS_MESSAGE: c_int = 3;      // stub - actual value from qcommon.h

// Game type constants
const GT_FFA: c_int = 0;              // stub - actual value from game headers
const GT_HOLOCRON: c_int = 1;         // stub
const GT_JEDIMASTER: c_int = 2;       // stub
const GT_SINGLE_PLAYER: c_int = 3;    // stub
const GT_DUEL: c_int = 4;             // stub
const GT_POWERDUEL: c_int = 5;        // stub
const GT_TEAM: c_int = 6;             // stub
const GT_SIEGE: c_int = 7;            // stub
const GT_CTF: c_int = 8;              // stub
const GT_CTY: c_int = 9;              // stub

// Screen and UI constants
const SCREEN_WIDTH: c_int = 640;  // stub - actual value from ui headers
const SCREEN_HEIGHT: c_int = 480; // stub - actual value from ui headers
const UI_BIGFONT: c_int = 0;      // stub
const UI_CENTER: c_int = 0;       // stub
const UI_INFOFONT: c_int = 0;     // stub
const UI_DROPSHADOW: c_int = 0;   // stub
const NUM_FORCE_MASTERY_LEVELS: c_int = 8; // stub

// Stubs for external globals and types from other modules
extern "C" {
    // Global state
    static mut cg: CgState;
    static mut cgs: CgsState;
    static bg_itemlist: [GitemT; 32];
    static colorWhite: [f32; 4];
    static forceMasteryLevels: [*const c_char; 8];
}

#[repr(C)]
pub struct CgState {
    infoScreenText: [c_char; 1024],
    loadLCARSStage: c_int,
}

#[repr(C)]
pub struct CgsState {
    gametype: c_int,
    media: MediaType,
}

#[repr(C)]
pub struct MediaType {
    loadBarLEDSurround: c_int,
    loadBarLED: c_int,
    loadBarLEDCap: c_int,
}

#[repr(C)]
pub struct GitemT {
    classname: *const c_char,
    // Additional fields omitted for stub
}

/*
======================
CG_LoadingString

======================
*/
pub unsafe fn CG_LoadingString(s: *const c_char) {
    Q_strncpyz(
        (*addr_of_mut!(cg)).infoScreenText.as_mut_ptr(),
        s,
        ::core::mem::size_of_val(&(*addr_of_mut!(cg)).infoScreenText),
    );

    trap_UpdateScreen();
}

/*
===================
CG_LoadingItem
===================
*/
pub unsafe fn CG_LoadingItem(itemNum: c_int) {
    let mut item: *const GitemT;
    let mut upperKey: [c_char; 1024] = [0; 1024];

    item = &bg_itemlist[itemNum as usize];

    if (*item).classname.is_null() || *(*item).classname == 0 {
        //	CG_LoadingString( "Unknown item" );
        return;
    }

    strcpy(upperKey.as_mut_ptr(), (*item).classname);
    CG_LoadingString(CG_GetStringEdString(
        b"SP_INGAME\0".as_ptr() as *const c_char,
        Q_strupr(upperKey.as_mut_ptr()),
    ));
}

/*
===================
CG_LoadingClient
===================
*/
pub unsafe fn CG_LoadingClient(clientNum: c_int) {
    let mut info: *const c_char;
    let mut personality: [c_char; 64] = [0; 64]; // MAX_QPATH

    info = CG_ConfigString(CS_PLAYERS + clientNum);

    /*
    char			model[MAX_QPATH];
    char			iconName[MAX_QPATH];
    char			*skin;
    if ( loadingPlayerIconCount < MAX_LOADING_PLAYER_ICONS ) {
        Q_strncpyz( model, Info_ValueForKey( info, "model" ), sizeof( model ) );
        skin = Q_strrchr( model, '/' );
        if ( skin ) {
            *skin++ = '\0';
        } else {
            skin = "default";
        }

        Com_sprintf( iconName, MAX_QPATH, "models/players/%s/icon_%s.tga", model, skin );

        loadingPlayerIcons[loadingPlayerIconCount] = trap_R_RegisterShaderNoMip( iconName );
        if ( !loadingPlayerIcons[loadingPlayerIconCount] ) {
            Com_sprintf( iconName, MAX_QPATH, "models/players/characters/%s/icon_%s.tga", model, skin );
            loadingPlayerIcons[loadingPlayerIconCount] = trap_R_RegisterShaderNoMip( iconName );
        }
        if ( !loadingPlayerIcons[loadingPlayerIconCount] ) {
            Com_sprintf( iconName, MAX_QPATH, "models/players/%s/icon_%s.tga", DEFAULT_MODEL, "default" );
            loadingPlayerIcons[loadingPlayerIconCount] = trap_R_RegisterShaderNoMip( iconName );
        }
        if ( loadingPlayerIcons[loadingPlayerIconCount] ) {
            loadingPlayerIconCount++;
        }
    }
    */
    Q_strncpyz(
        personality.as_mut_ptr(),
        Info_ValueForKey(info, b"n\0".as_ptr() as *const c_char),
        ::core::mem::size_of_val(&personality),
    );
    Q_CleanStr(personality.as_mut_ptr());

    /*
    if( cgs.gametype == GT_SINGLE_PLAYER ) {
        trap_S_RegisterSound( va( "sound/player/announce/%s.wav", personality ));
    }
    */

    CG_LoadingString(personality.as_ptr());
}

/*
====================
CG_DrawInformation

Draw all the status / pacifier stuff during level loading
====================
overlays UI_DrawConnectScreen
*/
// #define UI_INFOFONT (UI_BIGFONT)
pub unsafe fn CG_DrawInformation() {
    let mut s: *const c_char;
    let mut info: *const c_char;
    let mut sysInfo: *const c_char;
    let mut y: c_int;
    let mut value: c_int;
    let mut valueNOFP: c_int;
    let mut levelshot: c_int;
    let mut buf: [c_char; 1024] = [0; 1024];
    let iPropHeight: c_int = 18; // I know, this is total crap, but as a post release asian-hack....  -Ste

    info = CG_ConfigString(CS_SERVERINFO);
    sysInfo = CG_ConfigString(CS_SYSTEMINFO);

    s = Info_ValueForKey(info, b"mapname\0".as_ptr() as *const c_char);
    levelshot = trap_R_RegisterShaderNoMip(va(
        b"levelshots/%s\0".as_ptr() as *const c_char,
        s,
    ));
    if levelshot == 0 {
        levelshot = trap_R_RegisterShaderNoMip(b"menu/art/unknownmap_mp\0".as_ptr() as *const c_char);
    }
    trap_R_SetColor(std::ptr::null());
    CG_DrawPic(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, levelshot);

    CG_LoadBar();

    // draw the icons of things as they are loaded
    //	CG_DrawLoadingIcons();

    // the first 150 rows are reserved for the client connection
    // screen to write into
    if (*addr_of_mut!(cg)).infoScreenText[0] != 0 {
        let psLoading: *const c_char = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"LOADING_MAPNAME\0".as_ptr() as *const c_char);
        UI_DrawProportionalString(
            320,
            128 - 32,
            va( /*"Loading... %s"*/ psLoading, (*addr_of_mut!(cg)).infoScreenText.as_ptr()),
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite.as_ptr() as *const c_void,
        );
    } else {
        let psAwaitingSnapshot: *const c_char = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"AWAITING_SNAPSHOT\0".as_ptr() as *const c_char);
        UI_DrawProportionalString(
            320,
            128 - 32, /*"Awaiting snapshot..."*/
            psAwaitingSnapshot,
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite.as_ptr() as *const c_void,
        );
    }

    // draw info string information

    y = 180 - 32;

    // don't print server lines if playing a local game
    trap_Cvar_VariableStringBuffer(
        b"sv_running\0".as_ptr() as *const c_char,
        buf.as_mut_ptr(),
        ::core::mem::size_of_val(&buf),
    );
    if atoi(buf.as_ptr()) == 0 {
        // server hostname
        Q_strncpyz(
            buf.as_mut_ptr(),
            Info_ValueForKey(info, b"sv_hostname\0".as_ptr() as *const c_char),
            1024,
        );
        Q_CleanStr(buf.as_mut_ptr());
        UI_DrawProportionalString(
            320,
            y,
            buf.as_ptr(),
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite.as_ptr() as *const c_void,
        );
        y += iPropHeight;

        // pure server
        s = Info_ValueForKey(sysInfo, b"sv_pure\0".as_ptr() as *const c_char);
        if *s == b'1' as c_char {
            let psPure: *const c_char = CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"PURE_SERVER\0".as_ptr() as *const c_char);
            UI_DrawProportionalString(
                320,
                y,
                psPure,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }

        // server-specific message of the day
        s = CG_ConfigString(CS_MOTD);
        if *s != 0 {
            UI_DrawProportionalString(
                320,
                y,
                s,
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }

        {
            // display global MOTD at bottom (mirrors ui_main UI_DrawConnectScreen
            let mut motdString: [c_char; 1024] = [0; 1024];
            trap_Cvar_VariableStringBuffer(
                b"cl_motdString\0".as_ptr() as *const c_char,
                motdString.as_mut_ptr(),
                ::core::mem::size_of_val(&motdString),
            );

            if motdString[0] != 0 {
                UI_DrawProportionalString(
                    320,
                    425,
                    motdString.as_ptr(),
                    UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                    colorWhite.as_ptr() as *const c_void,
                );
            }
        }

        // some extra space after hostname and motd
        y += 10;
    }

    // map-specific message (long map name)
    s = CG_ConfigString(CS_MESSAGE);
    if *s != 0 {
        UI_DrawProportionalString(
            320,
            y,
            s,
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite.as_ptr() as *const c_void,
        );
        y += iPropHeight;
    }

    // cheats warning
    s = Info_ValueForKey(sysInfo, b"sv_cheats\0".as_ptr() as *const c_char);
    if *s == b'1' as c_char {
        UI_DrawProportionalString(
            320,
            y,
            CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"CHEATSAREENABLED\0".as_ptr() as *const c_char),
            UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
            colorWhite.as_ptr() as *const c_void,
        );
        y += iPropHeight;
    }

    // game type
    match (*addr_of_mut!(cgs)).gametype {
        GT_FFA => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"FREE_FOR_ALL\0".as_ptr() as *const c_char); //"Free For All";
                                                                                                                             //		s = "Free For All";
        }
        GT_HOLOCRON => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"HOLOCRON_FFA\0".as_ptr() as *const c_char); //"Holocron FFA";
                                                                                                                           //		s = "Holocron FFA";
        }
        GT_JEDIMASTER => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"SAGA\0".as_ptr() as *const c_char); //"Jedi Master";??

            //		s = "Jedi Master";
        }
        GT_SINGLE_PLAYER => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"SAGA\0".as_ptr() as *const c_char); //"Team FFA";

            //s = "Single Player";
        }
        GT_DUEL => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"DUEL\0".as_ptr() as *const c_char); //"Team FFA";
                                                                                                                    //s = "Duel";
        }
        GT_POWERDUEL => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"POWERDUEL\0".as_ptr() as *const c_char); //"Team FFA";
                                                                                                                         //s = "Power Duel";
        }
        GT_TEAM => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"TEAM_FFA\0".as_ptr() as *const c_char); //"Team FFA";

            //s = "Team FFA";
        }
        GT_SIEGE => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"SIEGE\0".as_ptr() as *const c_char); //"Siege";

            //s = "Siege";
        }
        GT_CTF => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"CAPTURE_THE_FLAG\0".as_ptr() as *const c_char); //"Capture the Flag";

            //s = "Capture The Flag";
        }
        GT_CTY => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"CAPTURE_THE_YSALIMARI\0".as_ptr() as *const c_char); //"Capture the Ysalamiri";

            //s = "Capture The Ysalamiri";
        }
        _ => {
            s = CG_GetStringEdString(b"MENUS\0".as_ptr() as *const c_char, b"SAGA\0".as_ptr() as *const c_char); //"Team FFA";

            //s = "Unknown Gametype";
        }
    }
    UI_DrawProportionalString(
        320,
        y,
        s,
        UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
        colorWhite.as_ptr() as *const c_void,
    );
    y += iPropHeight;

    if (*addr_of_mut!(cgs)).gametype != GT_SIEGE {
        value = atoi(Info_ValueForKey(info, b"timelimit\0".as_ptr() as *const c_char));
        if value != 0 {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s %i\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"TIMELIMIT\0".as_ptr() as *const c_char),
                    value,
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }

        if (*addr_of_mut!(cgs)).gametype < GT_CTF {
            value = atoi(Info_ValueForKey(info, b"fraglimit\0".as_ptr() as *const c_char));
            if value != 0 {
                UI_DrawProportionalString(
                    320,
                    y,
                    va(
                        b"%s %i\0".as_ptr() as *const c_char,
                        CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"FRAGLIMIT\0".as_ptr() as *const c_char),
                        value,
                    ),
                    UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                    colorWhite.as_ptr() as *const c_void,
                );
                y += iPropHeight;
            }

            if (*addr_of_mut!(cgs)).gametype == GT_DUEL || (*addr_of_mut!(cgs)).gametype == GT_POWERDUEL {
                value = atoi(Info_ValueForKey(info, b"duel_fraglimit\0".as_ptr() as *const c_char));
                if value != 0 {
                    UI_DrawProportionalString(
                        320,
                        y,
                        va(
                            b"%s %i\0".as_ptr() as *const c_char,
                            CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"WINLIMIT\0".as_ptr() as *const c_char),
                            value,
                        ),
                        UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                        colorWhite.as_ptr() as *const c_void,
                    );
                    y += iPropHeight;
                }
            }
        }
    }

    if (*addr_of_mut!(cgs)).gametype >= GT_CTF {
        value = atoi(Info_ValueForKey(info, b"capturelimit\0".as_ptr() as *const c_char));
        if value != 0 {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s %i\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"CAPTURELIMIT\0".as_ptr() as *const c_char),
                    value,
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
    }

    if (*addr_of_mut!(cgs)).gametype >= GT_TEAM {
        value = atoi(Info_ValueForKey(info, b"g_forceBasedTeams\0".as_ptr() as *const c_char));
        if value != 0 {
            UI_DrawProportionalString(
                320,
                y,
                CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"FORCEBASEDTEAMS\0".as_ptr() as *const c_char),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
    }

    if (*addr_of_mut!(cgs)).gametype != GT_SIEGE {
        valueNOFP = atoi(Info_ValueForKey(info, b"g_forcePowerDisable\0".as_ptr() as *const c_char));

        value = atoi(Info_ValueForKey(info, b"g_maxForceRank\0".as_ptr() as *const c_char));
        if value != 0 && valueNOFP == 0 && value < NUM_FORCE_MASTERY_LEVELS {
            let mut fmStr: [c_char; 1024] = [0; 1024];

            trap_SP_GetStringTextString(
                b"MP_INGAME_MAXFORCERANK\0".as_ptr() as *const c_char,
                fmStr.as_mut_ptr(),
                ::core::mem::size_of_val(&fmStr),
            );

            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s %s\0".as_ptr() as *const c_char,
                    fmStr.as_ptr(),
                    CG_GetStringEdString(
                        b"MP_INGAME\0".as_ptr() as *const c_char,
                        *forceMasteryLevels.get_unchecked(value as usize),
                    ),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        } else if valueNOFP == 0 {
            let mut fmStr: [c_char; 1024] = [0; 1024];
            trap_SP_GetStringTextString(
                b"MP_INGAME_MAXFORCERANK\0".as_ptr() as *const c_char,
                fmStr.as_mut_ptr(),
                ::core::mem::size_of_val(&fmStr),
            );

            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s %s\0".as_ptr() as *const c_char,
                    fmStr.as_ptr(),
                    *forceMasteryLevels.get_unchecked(7) as *const c_void as *const c_char,
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }

        if (*addr_of_mut!(cgs)).gametype == GT_DUEL || (*addr_of_mut!(cgs)).gametype == GT_POWERDUEL {
            value = atoi(Info_ValueForKey(info, b"g_duelWeaponDisable\0".as_ptr() as *const c_char));
        } else {
            value = atoi(Info_ValueForKey(info, b"g_weaponDisable\0".as_ptr() as *const c_char));
        }
        if (*addr_of_mut!(cgs)).gametype != GT_JEDIMASTER && value != 0 {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"SABERONLYSET\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }

        if valueNOFP != 0 {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"NOFPSET\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
    }

    // Display the rules based on type
    y += iPropHeight;
    match (*addr_of_mut!(cgs)).gametype {
        GT_FFA => {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_FFA_1\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
        GT_HOLOCRON => {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_HOLO_1\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_HOLO_2\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
        GT_JEDIMASTER => {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_JEDI_1\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_JEDI_2\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
        GT_SINGLE_PLAYER => {}
        GT_DUEL => {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_DUEL_1\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_DUEL_2\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
        GT_POWERDUEL => {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_POWERDUEL_1\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_POWERDUEL_2\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
        GT_TEAM => {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_TEAM_1\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_TEAM_2\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
        GT_SIEGE => {}
        GT_CTF => {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_CTF_1\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_CTF_2\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
        GT_CTY => {
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_CTY_1\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
            UI_DrawProportionalString(
                320,
                y,
                va(
                    b"%s\0".as_ptr() as *const c_char,
                    CG_GetStringEdString(b"MP_INGAME\0".as_ptr() as *const c_char, b"RULES_CTY_2\0".as_ptr() as *const c_char),
                ),
                UI_CENTER | UI_INFOFONT | UI_DROPSHADOW,
                colorWhite.as_ptr() as *const c_void,
            );
            y += iPropHeight;
        }
        _ => {}
    }
}

/*
===================
CG_LoadBar
===================
*/
pub unsafe fn CG_LoadBar() {
    let numticks: c_int = 9;
    let tickwidth: c_int = 40;
    let tickheight: c_int = 8;
    let tickpadx: c_int = 20;
    let tickpady: c_int = 12;
    let capwidth: c_int = 8;
    let barwidth: c_int = numticks * tickwidth + tickpadx * 2 + capwidth * 2;
    let barleft: c_int = (640 - barwidth) / 2;
    let barheight: c_int = tickheight + tickpady * 2;
    let bartop: c_int = 480 - barheight;
    let capleft: c_int = barleft + tickpadx;
    let tickleft: c_int = capleft + capwidth;
    let ticktop: c_int = bartop + tickpady;

    trap_R_SetColor(colorWhite.as_ptr() as *const c_void);
    // Draw background
    CG_DrawPic(
        barleft,
        bartop,
        barwidth,
        barheight,
        (*addr_of_mut!(cgs)).media.loadBarLEDSurround,
    );

    // Draw left cap (backwards)
    CG_DrawPic(tickleft, ticktop, -capwidth, tickheight, (*addr_of_mut!(cgs)).media.loadBarLEDCap);

    // Draw bar
    CG_DrawPic(
        tickleft,
        ticktop,
        tickwidth * (*addr_of_mut!(cg)).loadLCARSStage,
        tickheight,
        (*addr_of_mut!(cgs)).media.loadBarLED,
    );

    // Draw right cap
    CG_DrawPic(
        tickleft + tickwidth * (*addr_of_mut!(cg)).loadLCARSStage,
        ticktop,
        capwidth,
        tickheight,
        (*addr_of_mut!(cgs)).media.loadBarLEDCap,
    );
}
