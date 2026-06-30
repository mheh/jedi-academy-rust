// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_scoreboard -- draw the scoreboard on top of the game screen

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_int, c_void};
use crate::codemp::cgame::cg_local_h::*;   // cg_t, cgs_t, score_t, clientInfo_t, snapshot_t (+ cvar_t/playerState_t via its PCH chain) — not yet ported on disk; trust-import
use crate::codemp::ui::ui_shared_h::*;     // UI text-style consts
use crate::codemp::game::bg_saga_h::*;     // siegeClass_t

extern "C" {
    // From cg_local.h
    static mut cg: cg_t;
    static mut cgs: cgs_t;
    static cg_paused: cvar_t;
    fn CG_FadeColor(start_time: c_int, fade_time: c_int) -> *const [f32; 4];
    fn CG_PlaceString(rank: c_int) -> *const u8;
    fn CG_DrawFlagModel(x: c_int, y: c_int, w: c_int, h: c_int, team: c_int, bob: c_int);
    fn CG_DrawPic(x: c_int, y: c_int, w: c_int, h: c_int, hShader: c_int);
    fn CG_FillRect(x: f32, y: f32, w: f32, h: f32, color: *const [f32; 4]);
    fn CG_Text_Paint(x: f32, y: f32, scale: f32, color: *const [f32; 4], text: *const u8,
                     cursorPos: c_int, limit: c_int, style: c_int, font: c_int);
    fn CG_Text_Width(text: *const u8, scale: f32, font: c_int) -> f32;
    fn CG_GetStringEdString(table: *const u8, entry: *const u8) -> *const u8;
    fn CG_LoadDeferredPlayers();
    fn CG_DrawTeamBackground(x: f32, y: f32, w: f32, h: f32, alpha: f32, team: c_int);

    // From ui_shared.h
    static colorWhite: [f32; 4];
    static colorTable: [[f32; 4]; 32];
    fn UI_DrawProportionalString(x: c_int, y: c_int, str: *const u8, style: c_int, color: [f32; 4]);

    // From bg_saga.h
    static bgSiegeClasses: [siegeClass_t; 256];

    // utility
    fn va(fmt: *const u8, ...) -> *const u8;
}


// Constants for screen dimensions and layout
const SCOREBOARD_X: c_int = 0;

const SB_HEADER: c_int = 86;
const SB_TOP: c_int = SB_HEADER + 32;

// Where the status bar starts, so we don't overwrite it
const SB_STATUSBAR: c_int = 420;

const SB_NORMAL_HEIGHT: c_int = 25;
const SB_INTER_HEIGHT: c_int = 15; // interleaved height

const SB_MAXCLIENTS_NORMAL: c_int = (SB_STATUSBAR - SB_TOP) / SB_NORMAL_HEIGHT;
const SB_MAXCLIENTS_INTER: c_int = (SB_STATUSBAR - SB_TOP) / SB_INTER_HEIGHT - 1;

// Used when interleaved

const SB_LEFT_BOTICON_X: c_int = SCOREBOARD_X + 0;
const SB_LEFT_HEAD_X: c_int = SCOREBOARD_X + 32;
const SB_RIGHT_BOTICON_X: c_int = SCOREBOARD_X + 64;
const SB_RIGHT_HEAD_X: c_int = SCOREBOARD_X + 96;
// Normal
const SB_BOTICON_X: c_int = SCOREBOARD_X + 32;
const SB_HEAD_X: c_int = SCOREBOARD_X + 64;

const SB_SCORELINE_X: c_int = 100;
const SB_SCORELINE_WIDTH: c_int = 640 - SB_SCORELINE_X * 2;

const SB_RATING_WIDTH: c_int = 0; // (6 * BIGCHAR_WIDTH)
const SB_NAME_X: f32 = SB_SCORELINE_X as f32;
const SB_SCORE_X: f32 = SB_SCORELINE_X as f32 + 0.55 * SB_SCORELINE_WIDTH as f32;
const SB_PING_X: f32 = SB_SCORELINE_X as f32 + 0.70 * SB_SCORELINE_WIDTH as f32;
const SB_TIME_X: f32 = SB_SCORELINE_X as f32 + 0.85 * SB_SCORELINE_WIDTH as f32;

// The new and improved score board
//
// In cases where the number of clients is high, the score board heads are interleaved
// here's the layout

//
//	0   32   80  112  144   240  320  400   <-- pixel position
//  bot head bot head score ping time name
//
//  wins/losses are drawn on bot icon now

static mut localClient: bool = false; // true if local client has been displayed

/*
=================
CG_DrawScoreboard
=================
*/
unsafe fn CG_DrawClientScore(
    y: c_int,
    score: *mut score_t,
    color: *mut [f32; 4],
    fade: f32,
    largeFormat: c_int,
) {
    //vec3_t	headAngles;
    let ci: *mut clientInfo_t;
    let mut iconx: c_int;
    let mut headx: c_int;
    let mut scale: f32;

    if largeFormat != 0 {
        scale = 1.0f32;
    } else {
        scale = 0.75f32;
    }

    if (*score).client < 0 || (*score).client >= (*cgs).maxclients {
        crate::common::Com_Printf(b"Bad score->client: %i\n\0" as *const u8, (*score).client);
        return;
    }

    ci = &mut (*cgs).clientinfo[(*score).client as usize];

    iconx = SB_BOTICON_X + (SB_RATING_WIDTH / 2);
    headx = SB_HEAD_X + (SB_RATING_WIDTH / 2);

    // draw the handicap or bot skill marker (unless player has flag)
    if ((*ci).powerups & (1 << PW_NEUTRALFLAG)) != 0 {
        if largeFormat != 0 {
            CG_DrawFlagModel(
                iconx,
                y - (32 - BIGCHAR_HEIGHT) / 2,
                32,
                32,
                TEAM_FREE,
                0,
            );
        } else {
            CG_DrawFlagModel(iconx, y, 16, 16, TEAM_FREE, 0);
        }
    } else if ((*ci).powerups & (1 << PW_REDFLAG)) != 0 {
        if largeFormat != 0 {
            CG_DrawFlagModel(
                (iconx as f32 * (*cgs).screenXScale) as c_int,
                (y as f32 * (*cgs).screenYScale) as c_int,
                (32 as f32 * (*cgs).screenXScale) as c_int,
                (32 as f32 * (*cgs).screenYScale) as c_int,
                TEAM_RED,
                0,
            );
        } else {
            CG_DrawFlagModel(
                (iconx as f32 * (*cgs).screenXScale) as c_int,
                (y as f32 * (*cgs).screenYScale) as c_int,
                (32 as f32 * (*cgs).screenXScale) as c_int,
                (32 as f32 * (*cgs).screenYScale) as c_int,
                TEAM_RED,
                0,
            );
        }
    } else if ((*ci).powerups & (1 << PW_BLUEFLAG)) != 0 {
        if largeFormat != 0 {
            CG_DrawFlagModel(
                (iconx as f32 * (*cgs).screenXScale) as c_int,
                (y as f32 * (*cgs).screenYScale) as c_int,
                (32 as f32 * (*cgs).screenXScale) as c_int,
                (32 as f32 * (*cgs).screenYScale) as c_int,
                TEAM_BLUE,
                0,
            );
        } else {
            CG_DrawFlagModel(
                (iconx as f32 * (*cgs).screenXScale) as c_int,
                (y as f32 * (*cgs).screenYScale) as c_int,
                (32 as f32 * (*cgs).screenXScale) as c_int,
                (32 as f32 * (*cgs).screenYScale) as c_int,
                TEAM_BLUE,
                0,
            );
        }
    } else if (*cgs).gametype == GT_POWERDUEL
        && ((*ci).duelTeam == DUELTEAM_LONE || (*ci).duelTeam == DUELTEAM_DOUBLE)
    {
        if (*ci).duelTeam == DUELTEAM_LONE {
            CG_DrawPic(
                iconx,
                y,
                32,
                32,
                trap_R_RegisterShaderNoMip(b"gfx/mp/pduel_icon_lone\0" as *const u8),
            );
        } else {
            CG_DrawPic(
                iconx,
                y,
                32,
                32,
                trap_R_RegisterShaderNoMip(b"gfx/mp/pduel_icon_double\0" as *const u8),
            );
        }
    } else if (*cgs).gametype == GT_SIEGE {
        //try to draw the shader for this class on the scoreboard
        if (*ci).siegeIndex != -1 {
            let scl: *mut siegeClass_t = &mut bgSiegeClasses[(*ci).siegeIndex as usize];

            if (*scl).classShader != 0 {
                CG_DrawPic(
                    iconx,
                    y,
                    if largeFormat != 0 { 24 } else { 12 },
                    if largeFormat != 0 { 24 } else { 12 },
                    (*scl).classShader,
                );
            }
        }
    } else {
        // draw the wins / losses
        /*
        if ( cgs.gametype == GT_DUEL || cgs.gametype == GT_POWERDUEL )
        {
            CG_DrawSmallStringColor( iconx, y + SMALLCHAR_HEIGHT/2, va("%i/%i", ci->wins, ci->losses ), color );
        }
        */
        //rww - in duel, we now show wins/losses in place of "frags". This is because duel now defaults to 1 kill per round.
    }

    // highlight your position
    if (*score).client == (*(*cg).snap).ps.clientNum {
        let mut hcolor: [f32; 4] = [0.0; 4];
        let mut rank: c_int;

        localClient = true;

        if (*(*cg).snap).ps.persistant[PERS_TEAM as usize] == TEAM_SPECTATOR || (*cgs).gametype >= GT_TEAM
        {
            rank = -1;
        } else {
            rank = (*(*cg).snap).ps.persistant[PERS_RANK as usize] & !RANK_TIED_FLAG;
        }
        if rank == 0 {
            hcolor[0] = 0.0f32;
            hcolor[1] = 0.0f32;
            hcolor[2] = 0.7f32;
        } else if rank == 1 {
            hcolor[0] = 0.7f32;
            hcolor[1] = 0.0f32;
            hcolor[2] = 0.0f32;
        } else if rank == 2 {
            hcolor[0] = 0.7f32;
            hcolor[1] = 0.7f32;
            hcolor[2] = 0.0f32;
        } else {
            hcolor[0] = 0.7f32;
            hcolor[1] = 0.7f32;
            hcolor[2] = 0.7f32;
        }

        hcolor[3] = fade * 0.7f32;
        CG_FillRect(
            SB_SCORELINE_X as f32 - 5.0f32,
            y as f32 + 2.0f32,
            (640 - SB_SCORELINE_X * 2 + 10) as f32,
            if largeFormat != 0 {
                SB_NORMAL_HEIGHT as f32
            } else {
                SB_INTER_HEIGHT as f32
            },
            &hcolor,
        );
    }

    CG_Text_Paint(
        SB_NAME_X,
        y as f32,
        0.9f32 * scale,
        &colorWhite,
        (*ci).name.as_ptr(),
        0,
        0,
        ITEM_TEXTSTYLE_OUTLINED,
        FONT_MEDIUM,
    );

    if (*score).ping != -1 {
        if (*ci).team != TEAM_SPECTATOR || (*cgs).gametype == GT_DUEL || (*cgs).gametype == GT_POWERDUEL
        {
            if (*cgs).gametype == GT_DUEL || (*cgs).gametype == GT_POWERDUEL {
                CG_Text_Paint(
                    SB_SCORE_X,
                    y as f32,
                    1.0f32 * scale,
                    &colorWhite,
                    va(
                        b"%i/%i\0" as *const u8,
                        (*ci).wins,
                        (*ci).losses,
                    ),
                    0,
                    0,
                    ITEM_TEXTSTYLE_OUTLINED,
                    FONT_SMALL,
                );
            } else {
                CG_Text_Paint(
                    SB_SCORE_X,
                    y as f32,
                    1.0f32 * scale,
                    &colorWhite,
                    va(b"%i\0" as *const u8, (*score).score),
                    0,
                    0,
                    ITEM_TEXTSTYLE_OUTLINED,
                    FONT_SMALL,
                );
            }
        }

        CG_Text_Paint(
            SB_PING_X,
            y as f32,
            1.0f32 * scale,
            &colorWhite,
            va(b"%i\0" as *const u8, (*score).ping),
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
        CG_Text_Paint(
            SB_TIME_X,
            y as f32,
            1.0f32 * scale,
            &colorWhite,
            va(b"%i\0" as *const u8, (*score).time),
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
    } else {
        CG_Text_Paint(
            SB_SCORE_X,
            y as f32,
            1.0f32 * scale,
            &colorWhite,
            b"-\0" as *const u8,
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
        CG_Text_Paint(
            SB_PING_X,
            y as f32,
            1.0f32 * scale,
            &colorWhite,
            b"-\0" as *const u8,
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
        CG_Text_Paint(
            SB_TIME_X,
            y as f32,
            1.0f32 * scale,
            &colorWhite,
            b"-\0" as *const u8,
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
    }

    // add the "ready" marker for intermission exiting
    if ((*(*cg).snap).ps.stats[STAT_CLIENTS_READY as usize] & (1 << (*score).client)) != 0 {
        CG_Text_Paint(
            SB_NAME_X - 64.0f32,
            y as f32 + 2.0f32,
            0.7f32 * scale,
            &colorWhite,
            CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"READY\0" as *const u8),
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    }
}

/*
=================
CG_TeamScoreboard
=================
*/
unsafe fn CG_TeamScoreboard(
    mut y: c_int,
    team: c_int,
    fade: f32,
    maxClients: c_int,
    lineHeight: c_int,
    countOnly: c_int,
) -> c_int {
    let mut i: c_int;
    let mut score: *mut score_t;
    let mut color: [f32; 4] = [0.0; 4];
    let mut count: c_int;
    let mut ci: *mut clientInfo_t;

    color[0] = 1.0f32;
    color[1] = 1.0f32;
    color[2] = 1.0f32;
    color[3] = fade;

    count = 0;
    i = 0;
    while i < (*cg).numScores && count < maxClients {
        score = &mut (*cg).scores[i as usize];
        ci = &mut (*cgs).clientinfo[(*score).client as usize];

        if team != (*ci).team {
            i += 1;
            continue;
        }

        if countOnly == 0 {
            CG_DrawClientScore(
                y + lineHeight * count,
                score,
                &mut color,
                fade,
                if lineHeight == SB_NORMAL_HEIGHT { 1 } else { 0 },
            );
        }

        count += 1;
        i += 1;
    }

    count
}

unsafe fn CG_GetClassCount(team: c_int, siegeClass: c_int) -> c_int {
    let mut i: c_int = 0;
    let mut count: c_int = 0;
    let mut ci: *mut clientInfo_t;
    let mut scl: *mut siegeClass_t;

    i = 0;
    while i < (*cgs).maxclients {
        ci = &mut (*cgs).clientinfo[i as usize];

        if (*ci).infoValid == 0 || team != (*ci).team {
            i += 1;
            continue;
        }

        scl = &mut bgSiegeClasses[(*ci).siegeIndex as usize];

        // Correct class?
        if siegeClass != (*scl).classShader {
            i += 1;
            continue;
        }

        count += 1;
        i += 1;
    }

    count
}

unsafe fn CG_GetTeamNonScoreCount(team: c_int) -> c_int {
    let mut i: c_int = 0;
    let mut count: c_int = 0;
    let mut ci: *mut clientInfo_t;

    i = 0;
    while i < (*cgs).maxclients {
        ci = &mut (*cgs).clientinfo[i as usize];

        if (*ci).infoValid == 0 || (team != (*ci).team && team != (*ci).siegeDesiredTeam) {
            i += 1;
            continue;
        }

        count += 1;
        i += 1;
    }

    count
}

unsafe fn CG_GetTeamCount(team: c_int, maxClients: c_int) -> c_int {
    let mut i: c_int = 0;
    let mut count: c_int = 0;
    let mut ci: *mut clientInfo_t;
    let mut score: *mut score_t;

    i = 0;
    while i < (*cg).numScores && count < maxClients {
        score = &mut (*cg).scores[i as usize];
        ci = &mut (*cgs).clientinfo[(*score).client as usize];

        if team != (*ci).team {
            i += 1;
            continue;
        }

        count += 1;
        i += 1;
    }

    count
}

/*
=================
CG_DrawScoreboard

Draw the normal in-game scoreboard
=================
*/
pub static mut cg_siegeWinTeam: c_int = 0;

pub unsafe fn CG_DrawOldScoreboard() -> c_int {
    let mut x: c_int;
    let mut y: c_int;
    let mut w: c_int;
    let mut i: c_int;
    let mut n1: c_int;
    let mut n2: c_int;
    let mut fade: f32;
    let mut fadeColor: *const [f32; 4];
    let mut s: *const u8;
    let mut maxClients: c_int;
    let mut lineHeight: c_int;
    let mut topBorderSize: c_int;
    let mut bottomBorderSize: c_int;

    // don't draw amuthing if the menu or console is up
    if cg_paused.integer != 0 {
        (*cg).deferredPlayerLoading = 0;
        return 0;
    }

    // don't draw scoreboard during death while warmup up
    if (*cg).warmup != 0 && (*cg).showScores == 0 {
        return 0;
    }

    if (*cg).showScores != 0
        || (*cg).predictedPlayerState.pm_type == PM_DEAD
        || (*cg).predictedPlayerState.pm_type == PM_INTERMISSION
    {
        fade = 1.0f32;
        fadeColor = &colorWhite;
    } else {
        fadeColor = CG_FadeColor((*cg).scoreFadeTime, FADE_TIME);

        if fadeColor.is_null() {
            // next time scoreboard comes up, don't print killer
            (*cg).deferredPlayerLoading = 0;
            (*cg).killerName[0] = 0;
            return 0;
        }
        fade = (*fadeColor)[0];
    }

    // fragged by ... line
    // or if in intermission and duel, prints the winner of the duel round
    if ((*cgs).gametype == GT_DUEL || (*cgs).gametype == GT_POWERDUEL) && (*cgs).duelWinner != -1
        && (*cg).predictedPlayerState.pm_type == PM_INTERMISSION
    {
        s = va(
            b"%s^7 %s\0" as *const u8,
            (*cgs).clientinfo[(*cgs).duelWinner as usize].name.as_ptr(),
            CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"DUEL_WINS\0" as *const u8),
        );
        /*w = CG_DrawStrlen( s ) * BIGCHAR_WIDTH;
        x = ( SCREEN_WIDTH - w ) / 2;
        y = 40;
        CG_DrawBigString( x, y, s, fade );
        */
        x = SCREEN_WIDTH / 2;
        y = 40;
        CG_Text_Paint(
            (x as f32 - CG_Text_Width(s, 1.0f32, FONT_MEDIUM) / 2.0f32),
            y as f32,
            1.0f32,
            &colorWhite,
            s,
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    } else if ((*cgs).gametype == GT_DUEL || (*cgs).gametype == GT_POWERDUEL)
        && (*cgs).duelist1 != -1
        && (*cgs).duelist2 != -1
        && (*cg).predictedPlayerState.pm_type == PM_INTERMISSION
    {
        if (*cgs).gametype == GT_POWERDUEL && (*cgs).duelist3 != -1 {
            s = va(
                b"%s^7 %s %s^7 %s %s\0" as *const u8,
                (*cgs).clientinfo[(*cgs).duelist1 as usize].name.as_ptr(),
                CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"SPECHUD_VERSUS\0" as *const u8),
                (*cgs).clientinfo[(*cgs).duelist2 as usize].name.as_ptr(),
                CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"AND\0" as *const u8),
                (*cgs).clientinfo[(*cgs).duelist3 as usize].name.as_ptr(),
            );
        } else {
            s = va(
                b"%s^7 %s %s\0" as *const u8,
                (*cgs).clientinfo[(*cgs).duelist1 as usize].name.as_ptr(),
                CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"SPECHUD_VERSUS\0" as *const u8),
                (*cgs).clientinfo[(*cgs).duelist2 as usize].name.as_ptr(),
            );
        }
        /*w = CG_DrawStrlen( s ) * BIGCHAR_WIDTH;
        x = ( SCREEN_WIDTH - w ) / 2;
        y = 40;
        CG_DrawBigString( x, y, s, fade );
        */
        x = SCREEN_WIDTH / 2;
        y = 40;
        CG_Text_Paint(
            (x as f32 - CG_Text_Width(s, 1.0f32, FONT_MEDIUM) / 2.0f32),
            y as f32,
            1.0f32,
            &colorWhite,
            s,
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    } else if (*cg).killerName[0] != 0 {
        s = va(
            b"%s %s\0" as *const u8,
            CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"KILLEDBY\0" as *const u8),
            (*cg).killerName.as_ptr(),
        );
        /*w = CG_DrawStrlen( s ) * BIGCHAR_WIDTH;
        x = ( SCREEN_WIDTH - w ) / 2;
        y = 40;
        CG_DrawBigString( x, y, s, fade );
        */
        x = SCREEN_WIDTH / 2;
        y = 40;
        CG_Text_Paint(
            (x as f32 - CG_Text_Width(s, 1.0f32, FONT_MEDIUM) / 2.0f32),
            y as f32,
            1.0f32,
            &colorWhite,
            s,
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    }

    // current rank
    if (*cgs).gametype == GT_POWERDUEL {
        //do nothing?
    } else if (*cgs).gametype < GT_TEAM {
        if (*cg).snap.is_null() == false
            && (*(*cg).snap).ps.persistant[PERS_TEAM as usize] != TEAM_SPECTATOR
        {
            let mut sPlace: [u8; 256] = [0; 256];
            let mut sOf: [u8; 256] = [0; 256];
            let mut sWith: [u8; 256] = [0; 256];

            trap_SP_GetStringTextString(
                b"MP_INGAME_PLACE\0" as *const u8,
                sPlace.as_mut_ptr(),
                256,
            );
            trap_SP_GetStringTextString(
                b"MP_INGAME_OF\0" as *const u8,
                sOf.as_mut_ptr(),
                256,
            );
            trap_SP_GetStringTextString(
                b"MP_INGAME_WITH\0" as *const u8,
                sWith.as_mut_ptr(),
                256,
            );

            s = va(
                b"%s %s (%s %i) %s %i\0" as *const u8,
                CG_PlaceString((*(*cg).snap).ps.persistant[PERS_RANK as usize] + 1),
                sPlace.as_ptr(),
                sOf.as_ptr(),
                (*cg).numScores,
                sWith.as_ptr(),
                (*(*cg).snap).ps.persistant[PERS_SCORE as usize],
            );
            w = CG_DrawStrlen(s) * 8; // BIGCHAR_WIDTH
            x = SCREEN_WIDTH / 2;
            y = 60;
            //CG_DrawBigString( x, y, s, fade );
            UI_DrawProportionalString(x, y, s, UI_CENTER | UI_DROPSHADOW, colorTable[CT_WHITE as usize]);
        }
    } else if (*cgs).gametype != GT_SIEGE {
        if (*cg).teamScores[0] == (*cg).teamScores[1] {
            s = va(
                b"%s %i\0" as *const u8,
                CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"TIEDAT\0" as *const u8),
                (*cg).teamScores[0],
            );
        } else if (*cg).teamScores[0] >= (*cg).teamScores[1] {
            s = va(
                b"%s, %i / %i\0" as *const u8,
                CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"RED_LEADS\0" as *const u8),
                (*cg).teamScores[0],
                (*cg).teamScores[1],
            );
        } else {
            s = va(
                b"%s, %i / %i\0" as *const u8,
                CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"BLUE_LEADS\0" as *const u8),
                (*cg).teamScores[1],
                (*cg).teamScores[0],
            );
        }

        x = SCREEN_WIDTH / 2;
        y = 60;

        CG_Text_Paint(
            (x as f32 - CG_Text_Width(s, 1.0f32, FONT_MEDIUM) / 2.0f32),
            y as f32,
            1.0f32,
            &colorWhite,
            s,
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    } else if (*cgs).gametype == GT_SIEGE && (cg_siegeWinTeam == 1 || cg_siegeWinTeam == 2) {
        if cg_siegeWinTeam == 1 {
            s = va(
                b"%s\0" as *const u8,
                CG_GetStringEdString(
                    b"MP_INGAME\0" as *const u8,
                    b"SIEGETEAM1WIN\0" as *const u8,
                ),
            );
        } else {
            s = va(
                b"%s\0" as *const u8,
                CG_GetStringEdString(
                    b"MP_INGAME\0" as *const u8,
                    b"SIEGETEAM2WIN\0" as *const u8,
                ),
            );
        }

        x = SCREEN_WIDTH / 2;
        y = 60;

        CG_Text_Paint(
            (x as f32 - CG_Text_Width(s, 1.0f32, FONT_MEDIUM) / 2.0f32),
            y as f32,
            1.0f32,
            &colorWhite,
            s,
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    }

    // scoreboard
    y = SB_HEADER;

    CG_DrawPic(
        SB_SCORELINE_X - 40,
        y - 5,
        SB_SCORELINE_WIDTH + 80,
        40,
        trap_R_RegisterShaderNoMip(b"gfx/menus/menu_buttonback.tga\0" as *const u8),
    );

    CG_Text_Paint(
        SB_NAME_X,
        y as f32,
        1.0f32,
        &colorWhite,
        CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"NAME\0" as *const u8),
        0,
        0,
        ITEM_TEXTSTYLE_OUTLINED,
        FONT_MEDIUM,
    );
    if (*cgs).gametype == GT_DUEL || (*cgs).gametype == GT_POWERDUEL {
        let mut sWL: [u8; 100] = [0; 100];
        trap_SP_GetStringTextString(
            b"MP_INGAME_W_L\0" as *const u8,
            sWL.as_mut_ptr(),
            100,
        );

        CG_Text_Paint(
            SB_SCORE_X,
            y as f32,
            1.0f32,
            &colorWhite,
            sWL.as_ptr(),
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    } else {
        CG_Text_Paint(
            SB_SCORE_X,
            y as f32,
            1.0f32,
            &colorWhite,
            CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"SCORE\0" as *const u8),
            0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_MEDIUM,
        );
    }
    CG_Text_Paint(
        SB_PING_X,
        y as f32,
        1.0f32,
        &colorWhite,
        CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"PING\0" as *const u8),
        0,
        0,
        ITEM_TEXTSTYLE_OUTLINED,
        FONT_MEDIUM,
    );
    CG_Text_Paint(
        SB_TIME_X,
        y as f32,
        1.0f32,
        &colorWhite,
        CG_GetStringEdString(b"MP_INGAME\0" as *const u8, b"TIME\0" as *const u8),
        0,
        0,
        ITEM_TEXTSTYLE_OUTLINED,
        FONT_MEDIUM,
    );

    y = SB_TOP;

    // If there are more than SB_MAXCLIENTS_NORMAL, use the interleaved scores
    if (*cg).numScores > SB_MAXCLIENTS_NORMAL {
        maxClients = SB_MAXCLIENTS_INTER;
        lineHeight = SB_INTER_HEIGHT;
        topBorderSize = 8;
        bottomBorderSize = 16;
    } else {
        maxClients = SB_MAXCLIENTS_NORMAL;
        lineHeight = SB_NORMAL_HEIGHT;
        topBorderSize = 8;
        bottomBorderSize = 8;
    }

    localClient = false;

    //I guess this should end up being able to display 19 clients at once.
    //In a team game, if there are 9 or more clients on the team not in the lead,
    //we only want to show 10 of the clients on the team in the lead, so that we
    //have room to display the clients in the lead on the losing team.

    //I guess this can be accomplished simply by printing the first teams score with a maxClients
    //value passed in related to how many players are on both teams.
    if (*cgs).gametype >= GT_TEAM {
        //
        // teamplay scoreboard
        //
        y += lineHeight / 2;

        if (*cg).teamScores[0] >= (*cg).teamScores[1] {
            let mut team1MaxCl: c_int = CG_GetTeamCount(TEAM_RED, maxClients);
            let mut team2MaxCl: c_int = CG_GetTeamCount(TEAM_BLUE, maxClients);

            if team1MaxCl > 10 && (team1MaxCl + team2MaxCl) > maxClients {
                team1MaxCl -= team2MaxCl;
                //subtract as many as you have to down to 10, once we get there
                //we just set it to 10

                if team1MaxCl < 10 {
                    team1MaxCl = 10;
                }
            }

            team2MaxCl = maxClients - team1MaxCl; //team2 can display however many is left over after team1's display

            n1 = CG_TeamScoreboard(y, TEAM_RED, fade, team1MaxCl, lineHeight, 1);
            CG_DrawTeamBackground(
                SB_SCORELINE_X as f32 - 5.0f32,
                y as f32 - topBorderSize as f32,
                (640 - SB_SCORELINE_X * 2 + 10) as f32,
                (n1 * lineHeight + bottomBorderSize) as f32,
                0.33f32,
                TEAM_RED,
            );
            CG_TeamScoreboard(y, TEAM_RED, fade, team1MaxCl, lineHeight, 0);
            y += (n1 * lineHeight) + 16; // BIGCHAR_HEIGHT

            //maxClients -= n1;

            n2 = CG_TeamScoreboard(y, TEAM_BLUE, fade, team2MaxCl, lineHeight, 1);
            CG_DrawTeamBackground(
                SB_SCORELINE_X as f32 - 5.0f32,
                y as f32 - topBorderSize as f32,
                (640 - SB_SCORELINE_X * 2 + 10) as f32,
                (n2 * lineHeight + bottomBorderSize) as f32,
                0.33f32,
                TEAM_BLUE,
            );
            CG_TeamScoreboard(y, TEAM_BLUE, fade, team2MaxCl, lineHeight, 0);
            y += (n2 * lineHeight) + 16; // BIGCHAR_HEIGHT

            //maxClients -= n2;

            maxClients -= team1MaxCl + team2MaxCl;
        } else {
            let mut team1MaxCl: c_int = CG_GetTeamCount(TEAM_BLUE, maxClients);
            let mut team2MaxCl: c_int = CG_GetTeamCount(TEAM_RED, maxClients);

            if team1MaxCl > 10 && (team1MaxCl + team2MaxCl) > maxClients {
                team1MaxCl -= team2MaxCl;
                //subtract as many as you have to down to 10, once we get there
                //we just set it to 10

                if team1MaxCl < 10 {
                    team1MaxCl = 10;
                }
            }

            team2MaxCl = maxClients - team1MaxCl; //team2 can display however many is left over after team1's display

            n1 = CG_TeamScoreboard(y, TEAM_BLUE, fade, team1MaxCl, lineHeight, 1);
            CG_DrawTeamBackground(
                SB_SCORELINE_X as f32 - 5.0f32,
                y as f32 - topBorderSize as f32,
                (640 - SB_SCORELINE_X * 2 + 10) as f32,
                (n1 * lineHeight + bottomBorderSize) as f32,
                0.33f32,
                TEAM_BLUE,
            );
            CG_TeamScoreboard(y, TEAM_BLUE, fade, team1MaxCl, lineHeight, 0);
            y += (n1 * lineHeight) + 16; // BIGCHAR_HEIGHT

            //maxClients -= n1;

            n2 = CG_TeamScoreboard(y, TEAM_RED, fade, team2MaxCl, lineHeight, 1);
            CG_DrawTeamBackground(
                SB_SCORELINE_X as f32 - 5.0f32,
                y as f32 - topBorderSize as f32,
                (640 - SB_SCORELINE_X * 2 + 10) as f32,
                (n2 * lineHeight + bottomBorderSize) as f32,
                0.33f32,
                TEAM_RED,
            );
            CG_TeamScoreboard(y, TEAM_RED, fade, team2MaxCl, lineHeight, 0);
            y += (n2 * lineHeight) + 16; // BIGCHAR_HEIGHT

            //maxClients -= n2;

            maxClients -= team1MaxCl + team2MaxCl;
        }
        n1 = CG_TeamScoreboard(y, TEAM_SPECTATOR, fade, maxClients, lineHeight, 0);
        y += (n1 * lineHeight) + 16; // BIGCHAR_HEIGHT
    } else {
        //
        // free for all scoreboard
        //
        n1 = CG_TeamScoreboard(y, TEAM_FREE, fade, maxClients, lineHeight, 0);
        y += (n1 * lineHeight) + 16; // BIGCHAR_HEIGHT
        n2 = CG_TeamScoreboard(y, TEAM_SPECTATOR, fade, maxClients - n1, lineHeight, 0);
        y += (n2 * lineHeight) + 16; // BIGCHAR_HEIGHT
    }

    if !localClient {
        // draw local client at the bottom
        i = 0;
        while i < (*cg).numScores {
            if (*cg).scores[i as usize].client == (*(*cg).snap).ps.clientNum {
                CG_DrawClientScore(
                    y,
                    &mut (*cg).scores[i as usize],
                    fadeColor as *mut [f32; 4],
                    fade,
                    if lineHeight == SB_NORMAL_HEIGHT { 1 } else { 0 },
                );
                break;
            }
            i += 1;
        }
    }

    // load any models that have been deferred
    (*cg).deferredPlayerLoading += 1;
    if (*cg).deferredPlayerLoading > 10 {
        CG_LoadDeferredPlayers();
    }

    1
}

//================================================================================

// Stubs for external functions not defined in this file
extern "C" {
    fn CG_DrawStrlen(str: *const u8) -> c_int;
    fn trap_R_RegisterShaderNoMip(name: *const u8) -> c_int;
    fn trap_SP_GetStringTextString(table: *const u8, buf: *mut u8, buflen: c_int);
}
