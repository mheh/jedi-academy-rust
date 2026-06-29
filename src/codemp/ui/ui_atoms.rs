// Copyright (C) 1999-2000 Id Software, Inc.
//
// UI_ATOMS.C
//
// User interface building blocks and support functions.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// Type declarations and extern bindings
// ============================================================================

// After a frame, so caching won't disrupt the sound
pub static mut m_entersound: c_int = 0;

// These are stubs - the real definitions come from ui_local.h and related headers.
// We declare them here to allow this module to compile independently.

#[repr(C)]
pub struct postGameInfo_t {
    pub accuracy: c_int,
    pub impressives: c_int,
    pub excellents: c_int,
    pub defends: c_int,
    pub assists: c_int,
    pub gauntlets: c_int,
    pub baseScore: c_int,
    pub perfects: c_int,
    pub redScore: c_int,
    pub blueScore: c_int,
    pub score: c_int,
    pub timeBonus: c_int,
    pub shutoutBonus: c_int,
    pub skillBonus: c_int,
    pub time: c_int,
    pub captures: c_int,
}

#[repr(C)]
pub struct uiDC_t {
    pub frameTime: c_int,
    pub realTime: c_int,
    pub whiteShader: c_int,
    pub cursorx: c_int,
    pub cursory: c_int,
}

#[repr(C)]
pub struct mapInfo_t {
    pub timeToBeat: [c_int; 4],
}

#[repr(C)]
pub struct uiInfo_t {
    pub uiDC: uiDC_t,
    pub demoAvailable: c_int,
    pub newHighScoreTime: c_int,
    pub newBestTime: c_int,
    pub q3HeadCount: c_int,
    pub q3HeadNames: [*const c_char; 64],
    pub mapList: [mapInfo_t; 1],
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // Note: actual cvar_t is larger but we only need integer field here
}

pub static mut newUI: c_int = 0;

extern "C" {
    pub static mut uiInfo: uiInfo_t;
    pub static mut ui_currentMap: cvar_t;
}

// ============================================================================
// Engine syscalls (trap functions)
// ============================================================================

extern "C" {
    pub fn trap_Cmd_ExecuteText(exec_when: c_int, text: *const c_char);
    pub fn trap_Argv(arg: c_int, buffer: *mut c_char, bufferlen: c_int);
    pub fn trap_Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufferlen: c_int);
    pub fn trap_Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn trap_Cvar_VariableValue(var_name: *const c_char) -> f32;
    pub fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut c_int, mode: c_int) -> c_int;
    pub fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: c_int);
    pub fn trap_FS_Write(buffer: *const c_void, len: c_int, f: c_int);
    pub fn trap_FS_FCloseFile(f: c_int);
    pub fn trap_FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int;
    pub fn trap_GetConfigString(index: c_int, buf: *mut c_char, size: c_int);
    pub fn trap_Print(msg: *const c_char);
    pub fn trap_Error(msg: *const c_char) -> !;
    pub fn trap_R_RegisterShaderNoMip(name: *const c_char) -> c_int;
    pub fn trap_R_DrawStretchPic(x: f32, y: f32, w: f32, h: f32, s1: f32, t1: f32, s2: f32, t2: f32, hShader: c_int);
    pub fn trap_R_SetColor(rgba: *const f32);
    pub fn trap_UpdateScreen();
    pub fn trap_Key_SetCatcher(catcher: c_int);
    pub fn trap_Argc() -> c_int;
}

// ============================================================================
// Game module functions
// ============================================================================

extern "C" {
    pub fn Display_CacheAll();
    pub fn Menus_CloseAll();
    pub fn Menus_ActivateByName(name: *const c_char) -> c_int;
    pub fn UI_Report();
    pub fn UI_Load();
    pub fn UI_ShowPostGame(newHigh: c_int);
}

// ============================================================================
// Shared code functions (from q_shared.c)
// ============================================================================

extern "C" {
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    pub fn Com_sprintf(dest: *mut c_char, destsize: c_int, fmt: *const c_char, ...) -> c_int;
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
}

// ============================================================================
// C library functions
// ============================================================================

extern "C" {
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn vsprintf(str: *mut c_char, format: *const c_char, ap: *mut c_void) -> c_int;
    pub fn va_start(ap: *mut c_void, lastarg: *const c_char);
    pub fn va_end(ap: *mut c_void);
    pub fn atoi(str: *const c_char) -> c_int;
}

// ============================================================================
// Conditional compilation for hard-linked code
// ============================================================================

// These are here so the functions in q_shared.c can link
#[cfg(not(feature = "UI_HARD_LINKED"))]
pub extern "C" fn Com_Error(level: c_int, error: *const c_char) {
    let mut argptr: [c_int; 1] = [0];
    let mut text: [c_char; 1024] = [0; 1024];

    unsafe {
        va_start(&mut argptr as *mut _ as *mut c_void, error);
        vsprintf(text.as_mut_ptr(), error, &mut argptr as *mut _ as *mut c_void);
        va_end(&mut argptr as *mut _ as *mut c_void);

        trap_Error(va(b"%s\0".as_ptr() as *const c_char, text.as_ptr()));
    }
}

#[cfg(not(feature = "UI_HARD_LINKED"))]
pub extern "C" fn Com_Printf(msg: *const c_char) {
    let mut argptr: [c_int; 1] = [0];
    let mut text: [c_char; 1024] = [0; 1024];

    unsafe {
        va_start(&mut argptr as *mut _ as *mut c_void, msg);
        vsprintf(text.as_mut_ptr(), msg, &mut argptr as *mut _ as *mut c_void);
        va_end(&mut argptr as *mut _ as *mut c_void);

        trap_Print(va(b"%s\0".as_ptr() as *const c_char, text.as_ptr()));
    }
}

// =================
// UI_ClampCvar
// =================
pub fn UI_ClampCvar(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

// =================
// UI_StartDemoLoop
// =================
pub fn UI_StartDemoLoop() {
    unsafe {
        trap_Cmd_ExecuteText(0, b"d1\n\0".as_ptr() as *const c_char);
    }
}

static mut UI_ARGV_BUFFER: [c_char; 1024] = [0; 1024];

pub fn UI_Argv(arg: c_int) -> *mut c_char {
    unsafe {
        trap_Argv(arg, UI_ARGV_BUFFER.as_mut_ptr(), 1024);
        UI_ARGV_BUFFER.as_mut_ptr()
    }
}

static mut UI_CVAR_BUFFER: [c_char; 1024] = [0; 1024];

pub fn UI_Cvar_VariableString(var_name: *const c_char) -> *mut c_char {
    unsafe {
        trap_Cvar_VariableStringBuffer(var_name, UI_CVAR_BUFFER.as_mut_ptr(), 1024);
        UI_CVAR_BUFFER.as_mut_ptr()
    }
}

pub fn UI_SetBestScores(newInfo: *mut postGameInfo_t, postGame: c_int) {
    unsafe {
        trap_Cvar_Set(b"ui_scoreAccuracy\0".as_ptr() as *const c_char, va(b"%i%%\0".as_ptr() as *const c_char, (*newInfo).accuracy));
        trap_Cvar_Set(b"ui_scoreImpressives\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).impressives));
        trap_Cvar_Set(b"ui_scoreExcellents\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).excellents));
        trap_Cvar_Set(b"ui_scoreDefends\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).defends));
        trap_Cvar_Set(b"ui_scoreAssists\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).assists));
        trap_Cvar_Set(b"ui_scoreGauntlets\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).gauntlets));
        trap_Cvar_Set(b"ui_scoreScore\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).score));
        trap_Cvar_Set(b"ui_scorePerfect\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).perfects));
        trap_Cvar_Set(b"ui_scoreTeam\0".as_ptr() as *const c_char, va(b"%i to %i\0".as_ptr() as *const c_char, (*newInfo).redScore, (*newInfo).blueScore));
        trap_Cvar_Set(b"ui_scoreBase\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).baseScore));
        trap_Cvar_Set(b"ui_scoreTimeBonus\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).timeBonus));
        trap_Cvar_Set(b"ui_scoreSkillBonus\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).skillBonus));
        trap_Cvar_Set(b"ui_scoreShutoutBonus\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).shutoutBonus));
        trap_Cvar_Set(b"ui_scoreTime\0".as_ptr() as *const c_char, va(b"%02i:%02i\0".as_ptr() as *const c_char, (*newInfo).time / 60, (*newInfo).time % 60));
        trap_Cvar_Set(b"ui_scoreCaptures\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).captures));
        if postGame != 0 {
            trap_Cvar_Set(b"ui_scoreAccuracy2\0".as_ptr() as *const c_char, va(b"%i%%\0".as_ptr() as *const c_char, (*newInfo).accuracy));
            trap_Cvar_Set(b"ui_scoreImpressives2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).impressives));
            trap_Cvar_Set(b"ui_scoreExcellents2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).excellents));
            trap_Cvar_Set(b"ui_scoreDefends2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).defends));
            trap_Cvar_Set(b"ui_scoreAssists2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).assists));
            trap_Cvar_Set(b"ui_scoreGauntlets2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).gauntlets));
            trap_Cvar_Set(b"ui_scoreScore2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).score));
            trap_Cvar_Set(b"ui_scorePerfect2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).perfects));
            trap_Cvar_Set(b"ui_scoreTeam2\0".as_ptr() as *const c_char, va(b"%i to %i\0".as_ptr() as *const c_char, (*newInfo).redScore, (*newInfo).blueScore));
            trap_Cvar_Set(b"ui_scoreBase2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).baseScore));
            trap_Cvar_Set(b"ui_scoreTimeBonus2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).timeBonus));
            trap_Cvar_Set(b"ui_scoreSkillBonus2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).skillBonus));
            trap_Cvar_Set(b"ui_scoreShutoutBonus2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).shutoutBonus));
            trap_Cvar_Set(b"ui_scoreTime2\0".as_ptr() as *const c_char, va(b"%02i:%02i\0".as_ptr() as *const c_char, (*newInfo).time / 60, (*newInfo).time % 60));
            trap_Cvar_Set(b"ui_scoreCaptures2\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*newInfo).captures));
        }
    }
}

pub fn UI_LoadBestScores(map: *const c_char, game: c_int) {
    let mut fileName: [c_char; 64] = [0; 64];
    let mut f: c_int = 0;
    let mut newInfo: postGameInfo_t = postGameInfo_t {
        accuracy: 0,
        impressives: 0,
        excellents: 0,
        defends: 0,
        assists: 0,
        gauntlets: 0,
        baseScore: 0,
        perfects: 0,
        redScore: 0,
        blueScore: 0,
        score: 0,
        timeBonus: 0,
        shutoutBonus: 0,
        skillBonus: 0,
        time: 0,
        captures: 0,
    };

    unsafe {
        let _ = memset(&mut newInfo as *mut _ as *mut c_void, 0, core::mem::size_of::<postGameInfo_t>());
        Com_sprintf(fileName.as_mut_ptr(), 64, b"games/%s_%i.game\0".as_ptr() as *const c_char, map, game);
        if trap_FS_FOpenFile(fileName.as_ptr(), &mut f, 0) >= 0 {
            let mut size: c_int = 0;
            trap_FS_Read(&mut size as *mut _ as *mut c_void, core::mem::size_of::<c_int>() as c_int, f);
            if size == core::mem::size_of::<postGameInfo_t>() as c_int {
                trap_FS_Read(&mut newInfo as *mut _ as *mut c_void, core::mem::size_of::<postGameInfo_t>() as c_int, f);
            }
            trap_FS_FCloseFile(f);
        }
        UI_SetBestScores(&mut newInfo, 0);

        Com_sprintf(fileName.as_mut_ptr(), 64, b"demos/%s_%d.dm_%d\0".as_ptr() as *const c_char, map, game, trap_Cvar_VariableValue(b"protocol\0".as_ptr() as *const c_char) as c_int);
        uiInfo.demoAvailable = 0;
        if trap_FS_FOpenFile(fileName.as_ptr(), &mut f, 0) >= 0 {
            uiInfo.demoAvailable = 1;
            trap_FS_FCloseFile(f);
        }
    }
}

// ===============
// UI_ClearScores
// ===============
pub fn UI_ClearScores() {
    let mut gameList: [c_char; 4096] = [0; 4096];
    let mut gameFile: *mut c_char;
    let mut i: c_int;
    let mut len: usize;
    let mut count: c_int;
    let mut size: c_int;
    let mut f: c_int;
    let mut newInfo: postGameInfo_t = postGameInfo_t {
        accuracy: 0,
        impressives: 0,
        excellents: 0,
        defends: 0,
        assists: 0,
        gauntlets: 0,
        baseScore: 0,
        perfects: 0,
        redScore: 0,
        blueScore: 0,
        score: 0,
        timeBonus: 0,
        shutoutBonus: 0,
        skillBonus: 0,
        time: 0,
        captures: 0,
    };

    unsafe {
        count = trap_FS_GetFileList(b"games\0".as_ptr() as *const c_char, b"game\0".as_ptr() as *const c_char, gameList.as_mut_ptr(), 4096);

        size = core::mem::size_of::<postGameInfo_t>() as c_int;
        let _ = memset(&mut newInfo as *mut _ as *mut c_void, 0, size as usize);

        if count > 0 {
            gameFile = gameList.as_mut_ptr();
            i = 0;
            while i < count {
                len = strlen(gameFile);
                if trap_FS_FOpenFile(va(b"games/%s\0".as_ptr() as *const c_char, gameFile), &mut f, 1) >= 0 {
                    trap_FS_Write(&size as *const _ as *const c_void, core::mem::size_of::<c_int>() as c_int, f);
                    trap_FS_Write(&newInfo as *const _ as *const c_void, size, f);
                    trap_FS_FCloseFile(f);
                }
                gameFile = gameFile.add(len + 1);
                i += 1;
            }
        }

        UI_SetBestScores(&mut newInfo, 0);
    }
}

static mut UI_CACHE_F_INITIALIZED: c_int = 0;

pub fn UI_Cache_f() {
    let mut i: c_int;

    unsafe {
        Display_CacheAll();
        if trap_Argc() == 2 {
            i = 0;
            while i < uiInfo.q3HeadCount {
                trap_Print(va(b"model %s\n\0".as_ptr() as *const c_char, uiInfo.q3HeadNames[i as usize]));
                i += 1;
            }
        }
    }
}

// =======================
// UI_CalcPostGameStats
// =======================
pub fn UI_CalcPostGameStats() {
    let mut map: [c_char; 64] = [0; 64];
    let mut fileName: [c_char; 64] = [0; 64];
    let mut info: [c_char; 1024] = [0; 1024];
    let mut f: c_int;
    let mut size: c_int;
    let mut game: c_int;
    let mut time: c_int;
    let mut adjustedTime: c_int;
    let mut oldInfo: postGameInfo_t = postGameInfo_t {
        accuracy: 0,
        impressives: 0,
        excellents: 0,
        defends: 0,
        assists: 0,
        gauntlets: 0,
        baseScore: 0,
        perfects: 0,
        redScore: 0,
        blueScore: 0,
        score: 0,
        timeBonus: 0,
        shutoutBonus: 0,
        skillBonus: 0,
        time: 0,
        captures: 0,
    };
    let mut newInfo: postGameInfo_t = postGameInfo_t {
        accuracy: 0,
        impressives: 0,
        excellents: 0,
        defends: 0,
        assists: 0,
        gauntlets: 0,
        baseScore: 0,
        perfects: 0,
        redScore: 0,
        blueScore: 0,
        score: 0,
        timeBonus: 0,
        shutoutBonus: 0,
        skillBonus: 0,
        time: 0,
        captures: 0,
    };
    let mut newHigh: c_int;

    unsafe {
        trap_GetConfigString(0, info.as_mut_ptr(), 1024);
        Q_strncpyz(map.as_mut_ptr(), Info_ValueForKey(info.as_ptr(), b"mapname\0".as_ptr() as *const c_char), 64);
        game = atoi(Info_ValueForKey(info.as_ptr(), b"g_gametype\0".as_ptr() as *const c_char));

        // compose file name
        Com_sprintf(fileName.as_mut_ptr(), 64, b"games/%s_%i.game\0".as_ptr() as *const c_char, map.as_ptr(), game);
        // see if we have one already
        let _ = memset(&mut oldInfo as *mut _ as *mut c_void, 0, core::mem::size_of::<postGameInfo_t>());
        if trap_FS_FOpenFile(fileName.as_ptr(), &mut f, 0) >= 0 {
            // if so load it
            size = 0;
            trap_FS_Read(&mut size as *mut _ as *mut c_void, core::mem::size_of::<c_int>() as c_int, f);
            if size == core::mem::size_of::<postGameInfo_t>() as c_int {
                trap_FS_Read(&mut oldInfo as *mut _ as *mut c_void, core::mem::size_of::<postGameInfo_t>() as c_int, f);
            }
            trap_FS_FCloseFile(f);
        }

        newInfo.accuracy = atoi(UI_Argv(3));
        newInfo.impressives = atoi(UI_Argv(4));
        newInfo.excellents = atoi(UI_Argv(5));
        newInfo.defends = atoi(UI_Argv(6));
        newInfo.assists = atoi(UI_Argv(7));
        newInfo.gauntlets = atoi(UI_Argv(8));
        newInfo.baseScore = atoi(UI_Argv(9));
        newInfo.perfects = atoi(UI_Argv(10));
        newInfo.redScore = atoi(UI_Argv(11));
        newInfo.blueScore = atoi(UI_Argv(12));
        time = atoi(UI_Argv(13));
        newInfo.captures = atoi(UI_Argv(14));

        newInfo.time = (time - trap_Cvar_VariableValue(b"ui_matchStartTime\0".as_ptr() as *const c_char) as c_int) / 1000;
        adjustedTime = uiInfo.mapList[0].timeToBeat[game as usize];
        if newInfo.time < adjustedTime {
            newInfo.timeBonus = (adjustedTime - newInfo.time) * 10;
        } else {
            newInfo.timeBonus = 0;
        }

        if newInfo.redScore > newInfo.blueScore && newInfo.blueScore <= 0 {
            newInfo.shutoutBonus = 100;
        } else {
            newInfo.shutoutBonus = 0;
        }

        newInfo.skillBonus = trap_Cvar_VariableValue(b"g_spSkill\0".as_ptr() as *const c_char) as c_int;
        if newInfo.skillBonus <= 0 {
            newInfo.skillBonus = 1;
        }
        newInfo.score = newInfo.baseScore + newInfo.shutoutBonus + newInfo.timeBonus;
        newInfo.score *= newInfo.skillBonus;

        // see if the score is higher for this one
        newHigh = if newInfo.redScore > newInfo.blueScore && newInfo.score > oldInfo.score {
            1
        } else {
            0
        };

        if newHigh != 0 {
            // if so write out the new one
            uiInfo.newHighScoreTime = uiInfo.uiDC.realTime + 20000;
            if trap_FS_FOpenFile(fileName.as_ptr(), &mut f, 1) >= 0 {
                size = core::mem::size_of::<postGameInfo_t>() as c_int;
                trap_FS_Write(&size as *const _ as *const c_void, core::mem::size_of::<c_int>() as c_int, f);
                trap_FS_Write(&newInfo as *const _ as *const c_void, core::mem::size_of::<postGameInfo_t>() as c_int, f);
                trap_FS_FCloseFile(f);
            }
        }

        if newInfo.time < oldInfo.time {
            uiInfo.newBestTime = uiInfo.uiDC.realTime + 20000;
        }

        // put back all the ui overrides
        trap_Cvar_Set(b"capturelimit\0".as_ptr() as *const c_char, UI_Cvar_VariableString(b"ui_saveCaptureLimit\0".as_ptr() as *const c_char));
        trap_Cvar_Set(b"fraglimit\0".as_ptr() as *const c_char, UI_Cvar_VariableString(b"ui_saveFragLimit\0".as_ptr() as *const c_char));
        trap_Cvar_Set(b"duel_fraglimit\0".as_ptr() as *const c_char, UI_Cvar_VariableString(b"ui_saveDuelLimit\0".as_ptr() as *const c_char));
        trap_Cvar_Set(b"cg_drawTimer\0".as_ptr() as *const c_char, UI_Cvar_VariableString(b"ui_drawTimer\0".as_ptr() as *const c_char));
        trap_Cvar_Set(b"g_doWarmup\0".as_ptr() as *const c_char, UI_Cvar_VariableString(b"ui_doWarmup\0".as_ptr() as *const c_char));
        trap_Cvar_Set(b"g_Warmup\0".as_ptr() as *const c_char, UI_Cvar_VariableString(b"ui_Warmup\0".as_ptr() as *const c_char));
        trap_Cvar_Set(b"sv_pure\0".as_ptr() as *const c_char, UI_Cvar_VariableString(b"ui_pure\0".as_ptr() as *const c_char));
        trap_Cvar_Set(b"g_friendlyFire\0".as_ptr() as *const c_char, UI_Cvar_VariableString(b"ui_friendlyFire\0".as_ptr() as *const c_char));

        UI_SetBestScores(&mut newInfo, 1);
        UI_ShowPostGame(newHigh);
    }
}

// =================
// UI_ConsoleCommand
// =================
pub fn UI_ConsoleCommand(realTime: c_int) -> c_int {
    let mut cmd: *mut c_char;

    unsafe {
        uiInfo.uiDC.frameTime = realTime - uiInfo.uiDC.realTime;
        uiInfo.uiDC.realTime = realTime;

        cmd = UI_Argv(0);

        // ensure minimum menu data is available
        // Menu_Cache();

        if Q_stricmp(cmd, b"ui_test\0".as_ptr() as *const c_char) == 0 {
            UI_ShowPostGame(1);
        }

        if Q_stricmp(cmd, b"ui_report\0".as_ptr() as *const c_char) == 0 {
            UI_Report();
            return 1;
        }

        if Q_stricmp(cmd, b"ui_load\0".as_ptr() as *const c_char) == 0 {
            UI_Load();
            return 1;
        }

        if Q_stricmp(cmd, b"ui_opensiegemenu\0".as_ptr() as *const c_char) == 0 {
            if trap_Cvar_VariableValue(b"g_gametype\0".as_ptr() as *const c_char) == 4.0 {
                Menus_CloseAll();
                if Menus_ActivateByName(UI_Argv(1)) != 0 {
                    trap_Key_SetCatcher(16);
                }
            }
            return 1;
        }

        if Q_stricmp(cmd, b"ui_openmenu\0".as_ptr() as *const c_char) == 0 {
            // if ( trap_Cvar_VariableValue ( "developer" ) )
            {
                Menus_CloseAll();
                if Menus_ActivateByName(UI_Argv(1)) != 0 {
                    trap_Key_SetCatcher(16);
                }
                return 1;
            }
        }

        // if ( Q_stricmp (cmd, "remapShader") == 0 ) {
        //     if (trap_Argc() == 4) {
        //         char shader1[MAX_QPATH];
        //         char shader2[MAX_QPATH];
        //         Q_strncpyz(shader1, UI_Argv(1), sizeof(shader1));
        //         Q_strncpyz(shader2, UI_Argv(2), sizeof(shader2));
        //         trap_R_RemapShader(shader1, shader2, UI_Argv(3));
        //         return qtrue;
        //     }
        // }

        if Q_stricmp(cmd, b"postgame\0".as_ptr() as *const c_char) == 0 {
            UI_CalcPostGameStats();
            return 1;
        }

        if Q_stricmp(cmd, b"ui_cache\0".as_ptr() as *const c_char) == 0 {
            UI_Cache_f();
            return 1;
        }

        if Q_stricmp(cmd, b"ui_teamOrders\0".as_ptr() as *const c_char) == 0 {
            // UI_TeamOrdersMenu_f();
            return 1;
        }

        if Q_stricmp(cmd, b"ui_cdkey\0".as_ptr() as *const c_char) == 0 {
            // UI_CDKeyMenu_f();
            return 1;
        }

        0
    }
}

// =================
// UI_Shutdown
// =================
pub fn UI_Shutdown() {
}

pub fn UI_DrawNamedPic(x: f32, y: f32, width: f32, height: f32, picname: *const c_char) {
    let mut hShader: c_int;

    unsafe {
        hShader = trap_R_RegisterShaderNoMip(picname);
        trap_R_DrawStretchPic(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
    }
}

pub fn UI_DrawHandlePic(x: f32, y: f32, mut w: f32, mut h: f32, hShader: c_int) {
    let mut s0: f32;
    let mut s1: f32;
    let mut t0: f32;
    let mut t1: f32;

    if w < 0.0 {
        // flip about vertical
        w = -w;
        s0 = 1.0;
        s1 = 0.0;
    } else {
        s0 = 0.0;
        s1 = 1.0;
    }

    if h < 0.0 {
        // flip about horizontal
        h = -h;
        t0 = 1.0;
        t1 = 0.0;
    } else {
        t0 = 0.0;
        t1 = 1.0;
    }

    unsafe {
        trap_R_DrawStretchPic(x, y, w, h, s0, t0, s1, t1, hShader);
    }
}

// ================
// UI_FillRect
//
// Coordinates are 640*480 virtual values
// =================
pub fn UI_FillRect(x: f32, y: f32, width: f32, height: f32, color: *const f32) {
    unsafe {
        trap_R_SetColor(color);
        trap_R_DrawStretchPic(x, y, width, height, 0.0, 0.0, 0.0, 0.0, uiInfo.uiDC.whiteShader);
        trap_R_SetColor(core::ptr::null());
    }
}

pub fn UI_DrawSides(x: f32, y: f32, w: f32, h: f32) {
    unsafe {
        trap_R_DrawStretchPic(x, y, 1.0, h, 0.0, 0.0, 0.0, 0.0, uiInfo.uiDC.whiteShader);
        trap_R_DrawStretchPic(x + w - 1.0, y, 1.0, h, 0.0, 0.0, 0.0, 0.0, uiInfo.uiDC.whiteShader);
    }
}

pub fn UI_DrawTopBottom(x: f32, y: f32, w: f32, h: f32) {
    unsafe {
        trap_R_DrawStretchPic(x, y, w, 1.0, 0.0, 0.0, 0.0, 0.0, uiInfo.uiDC.whiteShader);
        trap_R_DrawStretchPic(x, y + h - 1.0, w, 1.0, 0.0, 0.0, 0.0, 0.0, uiInfo.uiDC.whiteShader);
    }
}

// ================
// UI_DrawRect
//
// Coordinates are 640*480 virtual values
// =================
pub fn UI_DrawRect(x: f32, y: f32, width: f32, height: f32, color: *const f32) {
    unsafe {
        trap_R_SetColor(color);
        UI_DrawTopBottom(x, y, width, height);
        UI_DrawSides(x, y, width, height);
        trap_R_SetColor(core::ptr::null());
    }
}

pub fn UI_SetColor(rgba: *const f32) {
    unsafe {
        trap_R_SetColor(rgba);
    }
}

pub fn UI_UpdateScreen() {
    unsafe {
        trap_UpdateScreen();
    }
}

pub fn UI_DrawTextBox(x: c_int, y: c_int, width: c_int, lines: c_int) {
    unsafe {
        UI_FillRect((x as f32) + 4.0, (y as f32) + 8.0, ((width + 1) as f32) * 8.0, ((lines + 1) as f32) * 16.0, [0.0, 0.0, 0.0, 1.0].as_ptr());
        UI_DrawRect((x as f32) + 4.0, (y as f32) + 8.0, ((width + 1) as f32) * 8.0, ((lines + 1) as f32) * 16.0, [1.0, 1.0, 1.0, 1.0].as_ptr());
    }
}

pub fn UI_CursorInRect(x: c_int, y: c_int, width: c_int, height: c_int) -> c_int {
    unsafe {
        if uiInfo.uiDC.cursorx < x || uiInfo.uiDC.cursory < y || uiInfo.uiDC.cursorx > x + width || uiInfo.uiDC.cursory > y + height {
            return 0;
        }
        1
    }
}
