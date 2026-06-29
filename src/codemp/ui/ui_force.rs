//! Mechanical port of `codemp/ui/ui_force.c`.
//
/*
=======================================================================

FORCE INTERFACE

=======================================================================
*/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::addr_of_mut;
use crate::codemp::game::q_shared_h::{NUM_FORCE_POWERS, NUM_SABER_COLORS};
use crate::ffi::types::qboolean;

// use this to get a demo build without an explicit demo build, i.e. to get the demo ui files to build

// ================================================================
// Stubs for external dependencies not yet ported.
// ================================================================

pub type fileHandle_t = c_int;
pub type qhandle_t = c_int;
pub type vec4_t = [f32; 4];

#[repr(C)]
pub struct rectDef_t {
    pub x: c_int,
    pub y: c_int,
    pub w: c_int,
    pub h: c_int,
}

#[repr(C)]
pub struct menuDef_t {
    // Stub - actual fields would need to match C definition
}

#[repr(C)]
pub struct uiInfo_s {
    // Partial stub with only fields used in this file
    pub forceConfigCount: c_int,
    pub forceConfigLightIndexBegin: c_int,
    pub forceConfigDarkIndexBegin: c_int,
    pub forceConfigNames: [*const c_char; 64], // Reasonable max
    pub forceConfigSide: [c_int; 64],
    pub q3SelectedHead: c_int,
    // Remaining fields omitted for stub
}

#[repr(C)]
pub struct vmCvar_t {
    pub handle: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub string: [c_char; 256],
}

// Constants
pub const NUM_FORCE_STAR_IMAGES: c_int = 9;
pub const NUM_SABER_COLORS: c_int = 6;
pub const FORCE_LEVEL_1: c_int = 1;
pub const FORCE_LEVEL_3: c_int = 3;
pub const FORCE_LIGHTSIDE: c_int = 1;
pub const FORCE_DARKSIDE: c_int = 2;
pub const FORCE_NONJEDI: c_int = 0;
pub const FORCE_JEDI: c_int = 1;
pub const MAX_INFO_VALUE: c_int = 256;
pub const EXEC_APPEND: c_int = 0;
pub const CS_SERVERINFO: c_int = 0;
pub const FS_WRITE: c_int = 1;
pub const FS_READ: c_int = 0;
pub const FEEDER_FORCECFG: c_int = 10;
pub const FEEDER_Q3HEADS: c_int = 9;
pub const UI_FORCE_RANK: c_int = 32;
pub const UI_FORCE_RANK_LEVITATION: c_int = 33;
pub const UI_FORCE_RANK_SABERATTACK: c_int = 48;
pub const UI_FORCE_RANK_SABERDEFEND: c_int = 49;
pub const FORCE_MASTERY_JEDI_KNIGHT: c_int = 3;
pub const MAX_FORCE_RANK: c_int = 3;
pub const TEAM_RED: c_int = 1;
pub const TEAM_BLUE: c_int = 2;
pub const TEAM_SPECTATOR: c_int = 3;
pub const SABER_RED: c_int = 0;
pub const SABER_ORANGE: c_int = 1;
pub const SABER_YELLOW: c_int = 2;
pub const SABER_GREEN: c_int = 3;
pub const SABER_BLUE: c_int = 4;
pub const SABER_PURPLE: c_int = 5;
pub const A_MOUSE1: c_int = 1;
pub const A_MOUSE2: c_int = 2;
pub const A_ENTER: c_int = 13;
pub const A_KP_ENTER: c_int = 140;
pub const A_BACKSPACE: c_int = 8;
pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

// FP_ constants (Force Power indices)
pub const FP_HEAL: usize = 0;
pub const FP_LEVITATION: usize = 1;
pub const FP_SPEED: usize = 2;
pub const FP_PUSH: usize = 3;
pub const FP_PULL: usize = 4;
pub const FP_TELEPATHY: usize = 5;
pub const FP_GRIP: usize = 6;
pub const FP_LIGHTNING: usize = 7;
pub const FP_RAGE: usize = 8;
pub const FP_PROTECT: usize = 9;
pub const FP_ABSORB: usize = 10;
pub const FP_TEAM_HEAL: usize = 11;
pub const FP_TEAM_FORCE: usize = 12;
pub const FP_DRAIN: usize = 13;
pub const FP_SEE: usize = 14;
pub const FP_SABER_OFFENSE: usize = 15;
pub const FP_SABER_DEFENSE: usize = 16;
pub const FP_SABERTHROW: usize = 17;

extern "C" {
    // From ui_local.h / game engine
    fn trap_R_RegisterShaderNoMip(name: *const c_char) -> c_int;
    fn trap_Cvar_Set(cvar: *const c_char, value: *const c_char);
    fn trap_Cmd_ExecuteText(exec_level: c_int, text: *const c_char);
    fn trap_FS_FOpenFile(path: *const c_char, f: *mut fileHandle_t, mode: c_int) -> c_int;
    fn trap_FS_Write(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int;
    fn trap_FS_FCloseFile(f: fileHandle_t);
    fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int;
    fn trap_GetConfigString(index: c_int, buffer: *mut c_char, bufsize: c_int);
    fn trap_Cvar_VariableValue(name: *const c_char) -> f32;
    fn trap_R_SetColor(color: *const c_void);
    fn UI_DrawHandlePic(x: c_int, y: c_int, w: c_int, h: c_int, shader: c_int);
    fn UI_Cvar_VariableString(name: *const c_char) -> *const c_char;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Menus_FindByName(name: *const c_char) -> *mut menuDef_t;
    fn Menu_SetFeederSelection(menu: *mut menuDef_t, feeder: c_int, index: c_int, p: *mut c_char);
    fn Menu_ShowItemByName(menu: *mut menuDef_t, p: *const c_char, bShow: qboolean);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn atoi(str: *const c_char) -> c_int;
    fn strlen(str: *const c_char) -> usize;
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    fn BG_LegalizedForcePowers(powers: *mut c_char, rank: c_int, freeSaber: c_int, forceTeam: c_int, gametype: c_int, p: c_int) -> c_int;
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn UI_TrueJediEnabled() -> c_int;
    fn UI_TeamName(team: c_int) -> *const c_char;
    fn UI_FeederSelection(feeder: c_int, index: c_int, p: *mut c_char);
    fn UI_LoadForceConfig_List();

    // Globals from other modules
    pub static mut bgForcePowerCost: [[c_int; 4]; NUM_FORCE_POWERS as usize];
    pub static mut forceMasteryPoints: [c_int; 4];
    pub static mut uiInfo: uiInfo_s;
    pub static mut uiSkinColor: c_int;
    pub static mut uiHoldSkinColor: c_int;
}

// Global variables
pub static mut uiForceSide: c_int = FORCE_LIGHTSIDE;
pub static mut uiJediNonJedi: c_int = -1;
pub static mut uiForceRank: c_int = FORCE_MASTERY_JEDI_KNIGHT;
pub static mut uiMaxRank: c_int = MAX_FORCE_RANK;
pub static mut uiMaxPoints: c_int = 20;
pub static mut uiForceUsed: c_int = 0;
pub static mut uiForceAvailable: c_int = 0;

pub static mut gTouchedForce: qboolean = qfalse;
pub static mut ui_freeSaber: vmCvar_t = vmCvar_t {
    handle: 0,
    modificationCount: 0,
    value: 0.0,
    integer: 0,
    string: [0; 256],
};
pub static mut ui_forcePowerDisable: vmCvar_t = vmCvar_t {
    handle: 0,
    modificationCount: 0,
    value: 0.0,
    integer: 0,
    string: [0; 256],
};

pub static mut uiForcePowersDisabled: [qboolean; NUM_FORCE_POWERS as usize] = [
    qfalse,//FP_HEAL,//instant
    qfalse,//FP_LEVITATION,//hold/duration
    qfalse,//FP_SPEED,//duration
    qfalse,//FP_PUSH,//hold/duration
    qfalse,//FP_PULL,//hold/duration
    qfalse,//FP_TELEPATHY,//instant
    qfalse,//FP_GRIP,//hold/duration
    qfalse,//FP_LIGHTNING,//hold/duration
    qfalse,//FP_RAGE,//duration
    qfalse,//FP_PROTECT,
    qfalse,//FP_ABSORB,
    qfalse,//FP_TEAM_HEAL,
    qfalse,//FP_TEAM_FORCE,
    qfalse,//FP_DRAIN,
    qfalse,//FP_SEE,
    qfalse,//FP_SABER_OFFENSE,
    qfalse,//FP_SABER_DEFENSE,
    qfalse//FP_SABERTHROW,
];

pub static mut uiForcePowersRank: [c_int; NUM_FORCE_POWERS as usize] = [
    0,//FP_HEAL = 0,//instant
    1,//FP_LEVITATION,//hold/duration, this one defaults to 1 (gives a free point)
    0,//FP_SPEED,//duration
    0,//FP_PUSH,//hold/duration
    0,//FP_PULL,//hold/duration
    0,//FP_TELEPATHY,//instant
    0,//FP_GRIP,//hold/duration
    0,//FP_LIGHTNING,//hold/duration
    0,//FP_RAGE,//duration
    0,//FP_PROTECT,
    0,//FP_ABSORB,
    0,//FP_TEAM_HEAL,
    0,//FP_TEAM_FORCE,
    0,//FP_DRAIN,
    0,//FP_SEE,
    1,//FP_SABER_OFFENSE, //default to 1 point in attack
    1,//FP_SABER_DEFENSE, //defualt to 1 point in defense
    0//FP_SABERTHROW,
];

pub static mut uiForcePowerDarkLight: [c_int; NUM_FORCE_POWERS as usize] = [ //0 == neutral
    FORCE_LIGHTSIDE,//FP_HEAL,//instant
    0,//FP_LEVITATION,//hold/duration
    0,//FP_SPEED,//duration
    0,//FP_PUSH,//hold/duration
    0,//FP_PULL,//hold/duration
    FORCE_LIGHTSIDE,//FP_TELEPATHY,//instant
    FORCE_DARKSIDE,//FP_GRIP,//hold/duration
    FORCE_DARKSIDE,//FP_LIGHTNING,//hold/duration
    FORCE_DARKSIDE,//FP_RAGE,//duration
    FORCE_LIGHTSIDE,//FP_PROTECT,//duration
    FORCE_LIGHTSIDE,//FP_ABSORB,//duration
    FORCE_LIGHTSIDE,//FP_TEAM_HEAL,//instant
    FORCE_DARKSIDE,//FP_TEAM_FORCE,//instant
    FORCE_DARKSIDE,//FP_DRAIN,//hold/duration
    0,//FP_SEE,//duration
    0,//FP_SABER_OFFENSE,
    0,//FP_SABER_DEFENSE,
    0//FP_SABERTHROW,
        //NUM_FORCE_POWERS
];

pub static mut uiForceStarShaders: [[c_int; 2]; NUM_FORCE_STAR_IMAGES as usize] = [[0; 2]; 9];
pub static mut uiSaberColorShaders: [c_int; NUM_SABER_COLORS as usize] = [0; 6];

pub unsafe fn UI_InitForceShaders()
{
    *addr_of_mut!(uiForceStarShaders[0][0]) = trap_R_RegisterShaderNoMip(b"forcestar0\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[0][1]) = trap_R_RegisterShaderNoMip(b"forcestar0\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[1][0]) = trap_R_RegisterShaderNoMip(b"forcecircle1\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[1][1]) = trap_R_RegisterShaderNoMip(b"forcestar1\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[2][0]) = trap_R_RegisterShaderNoMip(b"forcecircle2\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[2][1]) = trap_R_RegisterShaderNoMip(b"forcestar2\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[3][0]) = trap_R_RegisterShaderNoMip(b"forcecircle3\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[3][1]) = trap_R_RegisterShaderNoMip(b"forcestar3\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[4][0]) = trap_R_RegisterShaderNoMip(b"forcecircle4\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[4][1]) = trap_R_RegisterShaderNoMip(b"forcestar4\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[5][0]) = trap_R_RegisterShaderNoMip(b"forcecircle5\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[5][1]) = trap_R_RegisterShaderNoMip(b"forcestar5\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[6][0]) = trap_R_RegisterShaderNoMip(b"forcecircle6\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[6][1]) = trap_R_RegisterShaderNoMip(b"forcestar6\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[7][0]) = trap_R_RegisterShaderNoMip(b"forcecircle7\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[7][1]) = trap_R_RegisterShaderNoMip(b"forcestar7\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[8][0]) = trap_R_RegisterShaderNoMip(b"forcecircle8\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiForceStarShaders[8][1]) = trap_R_RegisterShaderNoMip(b"forcestar8\0".as_ptr() as *const c_char);

    *addr_of_mut!(uiSaberColorShaders[SABER_RED as usize]) = trap_R_RegisterShaderNoMip(b"menu/art/saber_red\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiSaberColorShaders[SABER_ORANGE as usize]) = trap_R_RegisterShaderNoMip(b"menu/art/saber_orange\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiSaberColorShaders[SABER_YELLOW as usize]) = trap_R_RegisterShaderNoMip(b"menu/art/saber_yellow\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiSaberColorShaders[SABER_GREEN as usize]) = trap_R_RegisterShaderNoMip(b"menu/art/saber_green\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiSaberColorShaders[SABER_BLUE as usize]) = trap_R_RegisterShaderNoMip(b"menu/art/saber_blue\0".as_ptr() as *const c_char);
    *addr_of_mut!(uiSaberColorShaders[SABER_PURPLE as usize]) = trap_R_RegisterShaderNoMip(b"menu/art/saber_purple\0".as_ptr() as *const c_char);
}

// Draw the stars spent on the current force power
pub unsafe fn UI_DrawForceStars(rect: *mut rectDef_t, scale: f32, color: *const c_void, textStyle: c_int, forceindex: c_int, val: c_int, min: c_int, mut max: c_int)
{
    let mut i: c_int;
    let pad: c_int = 4;
    let mut xPos: c_int;
    let width: c_int = 16;
    let mut starcolor: c_int;

    let mut val = val;
    if val < min || val > max
    {
        val = min;
    }

    if 1 != 0 {	// if (val)
        xPos = (*rect).x;

        i = FORCE_LEVEL_1;
        while i <= max {
            starcolor = bgForcePowerCost[forceindex as usize][i as usize];

            if uiForcePowersDisabled[forceindex as usize] != 0
            {
                let grColor: vec4_t = [0.2, 0.2, 0.2, 1.0];
                trap_R_SetColor(addr_of_mut!(grColor) as *const c_void);
            }

            if val >= i
            {	// Draw a star.
                UI_DrawHandlePic( xPos, (*rect).y+6, width, width, uiForceStarShaders[starcolor as usize][1] );
            }
            else
            {	// Draw a circle.
                UI_DrawHandlePic( xPos, (*rect).y+6, width, width, uiForceStarShaders[starcolor as usize][0] );
            }

            if uiForcePowersDisabled[forceindex as usize] != 0
            {
                trap_R_SetColor(core::ptr::null());
            }

            xPos += width + pad;
            i += 1;
        }
    }
}

// Set the client's force power layout.
pub unsafe fn UI_UpdateClientForcePowers(teamArg: *const c_char)
{
    trap_Cvar_Set( b"forcepowers\0".as_ptr() as *const c_char, va(b"%i-%i-%i%i%i%i%i%i%i%i%i%i%i%i%i%i%i%i%i%i\0".as_ptr() as *const c_char,
        uiForceRank, uiForceSide, uiForcePowersRank[0], uiForcePowersRank[1],
        uiForcePowersRank[2], uiForcePowersRank[3], uiForcePowersRank[4],
        uiForcePowersRank[5], uiForcePowersRank[6], uiForcePowersRank[7],
        uiForcePowersRank[8], uiForcePowersRank[9], uiForcePowersRank[10],
        uiForcePowersRank[11], uiForcePowersRank[12], uiForcePowersRank[13],
        uiForcePowersRank[14], uiForcePowersRank[15], uiForcePowersRank[16],
        uiForcePowersRank[17]) );

    if gTouchedForce != 0
    {
        if !teamArg.is_null() && *teamArg != 0
        {
            trap_Cmd_ExecuteText( EXEC_APPEND, va(b"forcechanged \"%s\"\n\0".as_ptr() as *const c_char, teamArg) );
        }
        else
        {
            trap_Cmd_ExecuteText( EXEC_APPEND, b"forcechanged\n\0".as_ptr() as *const c_char );
        }
    }

    gTouchedForce = qfalse;
}

pub unsafe fn UI_TranslateFCFIndex(index: c_int) -> c_int
{
    if uiForceSide == FORCE_LIGHTSIDE
    {
        return index-uiInfo.forceConfigLightIndexBegin;
    }

    return index-uiInfo.forceConfigDarkIndexBegin;
}

pub unsafe fn UI_SaveForceTemplate()
{
    let selectedName: *const c_char = UI_Cvar_VariableString(b"ui_SaveFCF\0".as_ptr() as *const c_char);
    let mut fcfString: [c_char; 512] = [0; 512];
    let mut forceStringValue: [c_char; 4] = [0; 4];
    let mut f: fileHandle_t = 0;
    let mut strPlace: usize = 0;
    let mut forcePlace: c_int = 0;
    let mut i: c_int = 0;
    let mut foundFeederItem: qboolean = qfalse;

    if selectedName.is_null() || *selectedName == 0
    {
        Com_Printf(b"You did not provide a name for the template.\n\0".as_ptr() as *const c_char);
        return;
    }

    if uiForceSide == FORCE_LIGHTSIDE
    { //write it into the light side folder
        trap_FS_FOpenFile(va(b"forcecfg/light/%s.fcf\0".as_ptr() as *const c_char, selectedName), addr_of_mut!(f), FS_WRITE);
    }
    else
    { //if it isn't light it must be dark
        trap_FS_FOpenFile(va(b"forcecfg/dark/%s.fcf\0".as_ptr() as *const c_char, selectedName), addr_of_mut!(f), FS_WRITE);
    }

    if f == 0
    {
        Com_Printf(b"There was an error writing the template file (read-only?).\n\0".as_ptr() as *const c_char);
        return;
    }

    Com_sprintf(fcfString.as_mut_ptr(), 512 as c_int, b"%i-%i-\0".as_ptr() as *const c_char, uiForceRank, uiForceSide);
    strPlace = strlen(fcfString.as_ptr());

    while forcePlace < NUM_FORCE_POWERS as c_int
    {
        Com_sprintf(forceStringValue.as_mut_ptr(), 4 as c_int, b"%i\0".as_ptr() as *const c_char, uiForcePowersRank[forcePlace as usize]);
        //Just use the force digit even if multiple digits. Shouldn't be longer than 1.
        fcfString[strPlace] = forceStringValue[0];
        strPlace += 1;
        forcePlace += 1;
    }
    fcfString[strPlace] = '\n' as c_char;
    fcfString[strPlace+1] = 0;

    trap_FS_Write(fcfString.as_ptr() as *const c_void, strlen(fcfString.as_ptr()) as c_int, f);
    trap_FS_FCloseFile(f);

    Com_Printf(b"Template saved as \"%s\".\n\0".as_ptr() as *const c_char, selectedName);

    //Now, update the FCF list
    UI_LoadForceConfig_List();

    //Then, scroll through and select the template for the file we just saved
    i = 0;
    while i < uiInfo.forceConfigCount
    {
        if Q_stricmp(uiInfo.forceConfigNames[i as usize], selectedName) == 0
        {
            if (uiForceSide == FORCE_LIGHTSIDE && uiInfo.forceConfigSide[i as usize] != 0) ||
                (uiForceSide == FORCE_DARKSIDE && uiInfo.forceConfigSide[i as usize] == 0)
            {
                Menu_SetFeederSelection(core::ptr::null_mut(), FEEDER_FORCECFG, UI_TranslateFCFIndex(i), core::ptr::null_mut());
                foundFeederItem = qtrue;
            }
        }

        i += 1;
    }

    //Else, go back to 0
    if foundFeederItem == 0
    {
        Menu_SetFeederSelection(core::ptr::null_mut(), FEEDER_FORCECFG, 0, core::ptr::null_mut());
    }
}


//
pub unsafe fn UpdateForceUsed()
{
    let mut curpower: c_int;
    let mut currank: c_int;
    let menu: *mut menuDef_t;

    // Currently we don't make a distinction between those that wish to play Jedi of lower than maximum skill.
    uiForceRank = uiMaxRank;

    uiForceUsed = 0;
    uiForceAvailable = forceMasteryPoints[uiForceRank as usize];

    // Make sure that we have one freebie in jump.
    if uiForcePowersRank[FP_LEVITATION]<1
    {
        uiForcePowersRank[FP_LEVITATION]=1;
    }

    if UI_TrueJediEnabled() != 0
    {//true jedi mode is set
        if uiJediNonJedi == -1
        {
            let mut x: c_int = 0;
            let mut clear: qboolean = qfalse;
            let mut update: qboolean = qfalse;
            uiJediNonJedi = FORCE_NONJEDI;
            while x < NUM_FORCE_POWERS as c_int
            {//if any force power is set, we must be a jedi
                if x == FP_LEVITATION as c_int || x == FP_SABER_OFFENSE as c_int
                {
                    if uiForcePowersRank[x as usize] > 1
                    {
                        uiJediNonJedi = FORCE_JEDI;
                        break;
                    }
                    else if uiForcePowersRank[x as usize] > 0
                    {
                        clear = qtrue;
                    }
                }
                else if uiForcePowersRank[x as usize] > 0
                {
                    uiJediNonJedi = FORCE_JEDI;
                    break;
                }
                x += 1;
            }
            if uiJediNonJedi == FORCE_JEDI
            {
                if uiForcePowersRank[FP_SABER_OFFENSE] < 1
                {
                    uiForcePowersRank[FP_SABER_OFFENSE]=1;
                    update = qtrue;
                }
            }
            else if clear != 0
            {
                x = 0;
                while x < NUM_FORCE_POWERS as c_int
                {//clear all force
                    uiForcePowersRank[x as usize] = 0;
                    x += 1;
                }
                update = qtrue;
            }
            if update != 0
            {
                let mut myTeam: c_int;
                myTeam = trap_Cvar_VariableValue(b"ui_myteam\0".as_ptr() as *const c_char) as c_int;
                if myTeam != TEAM_SPECTATOR
                {
                    UI_UpdateClientForcePowers(UI_TeamName(myTeam));//will cause him to respawn, if it's been 5 seconds since last one
                }
                else
                {
                    UI_UpdateClientForcePowers(core::ptr::null());//just update powers
                }
            }
        }
    }

    menu = Menus_FindByName(b"ingame_playerforce\0".as_ptr() as *const c_char);
    // Set the cost of the saberattack according to whether its free.
    if ui_freeSaber.integer != 0
    {	// Make saber free
        bgForcePowerCost[FP_SABER_OFFENSE][FORCE_LEVEL_1 as usize] = 0;
        bgForcePowerCost[FP_SABER_DEFENSE][FORCE_LEVEL_1 as usize] = 0;
        // Make sure that we have one freebie in saber if applicable.
        if uiForcePowersRank[FP_SABER_OFFENSE]<1
        {
            uiForcePowersRank[FP_SABER_OFFENSE]=1;
        }
        if uiForcePowersRank[FP_SABER_DEFENSE]<1
        {
            uiForcePowersRank[FP_SABER_DEFENSE]=1;
        }
        if !menu.is_null()
        {
            Menu_ShowItemByName(menu, b"setFP_SABER_DEFENSE\0".as_ptr() as *const c_char, qtrue);
            Menu_ShowItemByName(menu, b"setfp_saberthrow\0".as_ptr() as *const c_char, qtrue);
            Menu_ShowItemByName(menu, b"effectentry\0".as_ptr() as *const c_char, qtrue);
            Menu_ShowItemByName(menu, b"effectfield\0".as_ptr() as *const c_char, qtrue);
            Menu_ShowItemByName(menu, b"nosaber\0".as_ptr() as *const c_char, qfalse);
        }
    }
    else
    {	// Make saber normal cost
        bgForcePowerCost[FP_SABER_OFFENSE][FORCE_LEVEL_1 as usize] = 1;
        bgForcePowerCost[FP_SABER_DEFENSE][FORCE_LEVEL_1 as usize] = 1;
        // Also, check if there is no saberattack.  If there isn't, there had better not be any defense or throw!
        if uiForcePowersRank[FP_SABER_OFFENSE]<1
        {
            uiForcePowersRank[FP_SABER_DEFENSE]=0;
            uiForcePowersRank[FP_SABERTHROW]=0;
            if !menu.is_null()
            {
                Menu_ShowItemByName(menu, b"setfp_saberdefend\0".as_ptr() as *const c_char, qfalse);
                Menu_ShowItemByName(menu, b"setfp_saberthrow\0".as_ptr() as *const c_char, qfalse);
                Menu_ShowItemByName(menu, b"effectentry\0".as_ptr() as *const c_char, qfalse);
                Menu_ShowItemByName(menu, b"effectfield\0".as_ptr() as *const c_char, qfalse);
                Menu_ShowItemByName(menu, b"nosaber\0".as_ptr() as *const c_char, qtrue);
            }
        }
        else
        {
            if !menu.is_null()
            {
                Menu_ShowItemByName(menu, b"setfp_saberdefend\0".as_ptr() as *const c_char, qtrue);
                Menu_ShowItemByName(menu, b"setfp_saberthrow\0".as_ptr() as *const c_char, qtrue);
                Menu_ShowItemByName(menu, b"effectentry\0".as_ptr() as *const c_char, qtrue);
                Menu_ShowItemByName(menu, b"effectfield\0".as_ptr() as *const c_char, qtrue);
                Menu_ShowItemByName(menu, b"nosaber\0".as_ptr() as *const c_char, qfalse);
            }
        }
    }

    // Make sure that we're still legal.
    curpower = 0;
    while curpower < NUM_FORCE_POWERS as c_int {
        // Make sure that our ranks are within legal limits.
        if uiForcePowersRank[curpower as usize]<0
        {
            uiForcePowersRank[curpower as usize]=0;
        }
        else if uiForcePowersRank[curpower as usize]>=4
        {
            uiForcePowersRank[curpower as usize]=(4-1);
        }

        currank = FORCE_LEVEL_1;
        while currank <= uiForcePowersRank[curpower as usize] {
            // Check on this force power
            if uiForcePowersRank[curpower as usize]>0
            {	// Do not charge the player for the one freebie in jump, or if there is one in saber.
                if (curpower == FP_LEVITATION as c_int && currank == FORCE_LEVEL_1) ||
                    (curpower == FP_SABER_OFFENSE as c_int && currank == FORCE_LEVEL_1 && ui_freeSaber.integer != 0) ||
                    (curpower == FP_SABER_DEFENSE as c_int && currank == FORCE_LEVEL_1 && ui_freeSaber.integer != 0)
                {
                    // Do nothing (written this way for clarity)
                }
                else
                {	// Check if we can accrue the cost of this power.
                    if bgForcePowerCost[curpower as usize][currank as usize] > uiForceAvailable
                    {	// We can't afford this power.  Break to the next one.
                        // Remove this power from the player's roster.
                        uiForcePowersRank[curpower as usize] = currank-1;
                        break;
                    }
                    else
                    {	// Sure we can afford it.
                        uiForceUsed += bgForcePowerCost[curpower as usize][currank as usize];
                        uiForceAvailable -= bgForcePowerCost[curpower as usize][currank as usize];
                    }
                }
            }
            currank += 1;
        }
        curpower += 1;
    }

}


//Mostly parts of other functions merged into one another.
//Puts the current UI stuff into a string, legalizes it, and then reads it back out.
pub unsafe fn UI_ReadLegalForce()
{
    let mut fcfString: [c_char; 512] = [0; 512];
    let mut forceStringValue: [c_char; 4] = [0; 4];
    let mut strPlace: usize = 0;
    let mut forcePlace: c_int = 0;
    let mut i: c_int = 0;
    let mut singleBuf: [c_char; 64] = [0; 64];
    let mut info: [c_char; MAX_INFO_VALUE as usize] = [0; MAX_INFO_VALUE as usize];
    let mut c: usize = 0;
    let mut iBuf: c_int = 0;
    let mut forcePowerRank: c_int = 0;
    let mut currank: c_int = 0;
    let mut forceTeam: c_int = 0;
    let mut updateForceLater: qboolean = qfalse;

    //First, stick them into a string.
    Com_sprintf(fcfString.as_mut_ptr(), 512 as c_int, b"%i-%i-\0".as_ptr() as *const c_char, uiForceRank, uiForceSide);
    strPlace = strlen(fcfString.as_ptr());

    while forcePlace < NUM_FORCE_POWERS as c_int
    {
        Com_sprintf(forceStringValue.as_mut_ptr(), 4 as c_int, b"%i\0".as_ptr() as *const c_char, uiForcePowersRank[forcePlace as usize]);
        //Just use the force digit even if multiple digits. Shouldn't be longer than 1.
        fcfString[strPlace] = forceStringValue[0];
        strPlace += 1;
        forcePlace += 1;
    }
    fcfString[strPlace] = '\n' as c_char;
    fcfString[strPlace+1] = 0;

    info[0] = '\0' as c_char;
    trap_GetConfigString(CS_SERVERINFO, info.as_mut_ptr(), (MAX_INFO_VALUE as c_int));

    if atoi( Info_ValueForKey( info.as_ptr(), b"g_forceBasedTeams\0".as_ptr() as *const c_char ) ) != 0
    {
        match (trap_Cvar_VariableValue(b"ui_myteam\0".as_ptr() as *const c_char) as c_int) {
        TEAM_RED => {
            forceTeam = FORCE_DARKSIDE;
        },
        TEAM_BLUE => {
            forceTeam = FORCE_LIGHTSIDE;
        },
        _ => {
        },
        }
    }
    //Second, legalize them.
    if BG_LegalizedForcePowers(fcfString.as_mut_ptr(), uiMaxRank, ui_freeSaber.integer, forceTeam, atoi( Info_ValueForKey( info.as_ptr(), b"g_gametype\0".as_ptr() as *const c_char )), 0) == 0
    { //if they were illegal, we should refresh them.
        updateForceLater = qtrue;
    }

    //Lastly, put them back into the UI storage from the legalized string
    i = 0;
    c = 0;

    while fcfString[i as usize] != 0 && fcfString[i as usize] != '-' as c_char
    {
        singleBuf[c] = fcfString[i as usize];
        c += 1;
        i += 1;
    }
    singleBuf[c] = 0;
    c = 0;
    i += 1;

    iBuf = atoi(singleBuf.as_ptr());

    if iBuf > uiMaxRank || iBuf < 0
    { //this force config uses a rank level higher than our currently restricted level.. so we can't use it
      //FIXME: Print a message indicating this to the user
    //	return;
    }

    uiForceRank = iBuf;

    while fcfString[i as usize] != 0 && fcfString[i as usize] != '-' as c_char
    {
        singleBuf[c] = fcfString[i as usize];
        c += 1;
        i += 1;
    }
    singleBuf[c] = 0;
    c = 0;
    i += 1;

    uiForceSide = atoi(singleBuf.as_ptr());

    if uiForceSide != FORCE_LIGHTSIDE &&
        uiForceSide != FORCE_DARKSIDE
    {
        uiForceSide = FORCE_LIGHTSIDE;
        return;
    }

    //clear out the existing powers
    c = 0;
    while c < NUM_FORCE_POWERS as usize
    {
        uiForcePowersRank[c] = 0;
        c += 1;
    }
    uiForceUsed = 0;
    uiForceAvailable = forceMasteryPoints[uiForceRank as usize];
    gTouchedForce = qtrue;

    c = 0;
    while fcfString[i as usize] != 0 && c < NUM_FORCE_POWERS as usize {
        singleBuf[0] = fcfString[i as usize];
        singleBuf[1] = 0;
        iBuf = atoi(singleBuf.as_ptr());	// So, that means that Force Power "c" wants to be set to rank "iBuf".

        if iBuf < 0
        {
            iBuf = 0;
        }

        forcePowerRank = iBuf;

        if forcePowerRank > FORCE_LEVEL_3 || forcePowerRank < 0
        { //err..  not correct
            c += 1;
            i += 1;
            continue;  // skip this power
        }

        if uiForcePowerDarkLight[c] != 0 && uiForcePowerDarkLight[c] != uiForceSide
        { //Apparently the user has crafted a force config that has powers that don't fit with the config's side.
            c += 1;
            i += 1;
            continue;  // skip this power
        }

        // Accrue cost for each assigned rank for this power.
        currank = FORCE_LEVEL_1;
        while currank <= forcePowerRank {
            if bgForcePowerCost[c][currank as usize] > uiForceAvailable
            {	// Break out, we can't afford any more power.
                break;
            }
            // Pay for this rank of this power.
            uiForceUsed += bgForcePowerCost[c][currank as usize];
            uiForceAvailable -= bgForcePowerCost[c][currank as usize];

            uiForcePowersRank[c] += 1;
            currank += 1;
        }
        c += 1;
        i += 1;
    }

    if uiForcePowersRank[FP_LEVITATION] < 1
    {
        uiForcePowersRank[FP_LEVITATION]=1;
    }
    if uiForcePowersRank[FP_SABER_OFFENSE] < 1 && ui_freeSaber.integer != 0
    {
        uiForcePowersRank[FP_SABER_OFFENSE]=1;
    }
    if uiForcePowersRank[FP_SABER_DEFENSE] < 1 && ui_freeSaber.integer != 0
    {
        uiForcePowersRank[FP_SABER_DEFENSE]=1;
    }

    UpdateForceUsed();

    if updateForceLater != 0
    {
        gTouchedForce = qtrue;
        UI_UpdateClientForcePowers(core::ptr::null());
    }
}

pub unsafe fn UI_UpdateForcePowers()
{
    let forcePowers: *const c_char = UI_Cvar_VariableString(b"forcepowers\0".as_ptr() as *const c_char);
    let mut readBuf: [c_char; 256] = [0; 256];
    let mut i: c_int = 0;
    let mut i_f: c_int = 0;
    let mut i_r: c_int = 0;

    uiForceSide = 0;

    if !forcePowers.is_null() && *forcePowers != 0
    {
        while *forcePowers.offset(i as isize) != 0
        {
            i_r = 0;

            while *forcePowers.offset(i as isize) != 0 && *forcePowers.offset(i as isize) != '-' as c_char && i_r < 255
            {
                readBuf[i_r as usize] = *forcePowers.offset(i as isize);
                i_r += 1;
                i += 1;
            }
            readBuf[i_r as usize] = '\0' as c_char;
            if i_r >= 255 || *forcePowers.offset(i as isize) == 0 || *forcePowers.offset(i as isize) != '-' as c_char
            {
                uiForceSide = 0;
                goto validitycheck;
            }
            uiForceRank = atoi(readBuf.as_ptr());
            i_r = 0;

            if uiForceRank > uiMaxRank
            {
                uiForceRank = uiMaxRank;
            }

            i += 1;

            while *forcePowers.offset(i as isize) != 0 && *forcePowers.offset(i as isize) != '-' as c_char && i_r < 255
            {
                readBuf[i_r as usize] = *forcePowers.offset(i as isize);
                i_r += 1;
                i += 1;
            }
            readBuf[i_r as usize] = '\0' as c_char;
            if i_r >= 255 || *forcePowers.offset(i as isize) == 0 || *forcePowers.offset(i as isize) != '-' as c_char
            {
                uiForceSide = 0;
                goto validitycheck;
            }
            uiForceSide = atoi(readBuf.as_ptr());
            i_r = 0;

            i += 1;

            i_f = FP_HEAL as c_int;

            while *forcePowers.offset(i as isize) != 0 && i_f < NUM_FORCE_POWERS as c_int
            {
                readBuf[0] = *forcePowers.offset(i as isize);
                readBuf[1] = '\0' as c_char;
                uiForcePowersRank[i_f as usize] = atoi(readBuf.as_ptr());

                if i_f == FP_LEVITATION as c_int &&
                    uiForcePowersRank[i_f as usize] < 1
                {
                    uiForcePowersRank[i_f as usize] = 1;
                }

                if i_f == FP_SABER_OFFENSE as c_int &&
                    uiForcePowersRank[i_f as usize] < 1 &&
                    ui_freeSaber.integer != 0
                {
                    uiForcePowersRank[i_f as usize] = 1;
                }

                if i_f == FP_SABER_DEFENSE as c_int &&
                    uiForcePowersRank[i_f as usize] < 1 &&
                    ui_freeSaber.integer != 0
                {
                    uiForcePowersRank[i_f as usize] = 1;
                }

                i_f += 1;
                i += 1;
            }

            if i_f < NUM_FORCE_POWERS as c_int
            { //info for all the powers wasn't there..
                uiForceSide = 0;
                goto validitycheck;
            }
            i += 1;
        }
    }

    validitycheck:

    if uiForceSide == 0
    {
        uiForceSide = 1;
        uiForceRank = 1;
        i = 0;
        while i < NUM_FORCE_POWERS as c_int
        {
            if i == FP_LEVITATION as c_int
            {
                uiForcePowersRank[i as usize] = 1;
            }
            else if i == FP_SABER_OFFENSE as c_int && ui_freeSaber.integer != 0
            {
                uiForcePowersRank[i as usize] = 1;
            }
            else if i == FP_SABER_DEFENSE as c_int && ui_freeSaber.integer != 0
            {
                uiForcePowersRank[i as usize] = 1;
            }
            else
            {
                uiForcePowersRank[i as usize] = 0;
            }

            i += 1;
        }

        UI_UpdateClientForcePowers(core::ptr::null());
    }

    UpdateForceUsed();
}

pub unsafe fn UI_SkinColor_HandleKey(flags: c_int, special: *mut f32, key: c_int, num: c_int, min: c_int, max: c_int, r#type: c_int) -> qboolean
{
  if key == A_MOUSE1 || key == A_MOUSE2 || key == A_ENTER || key == A_KP_ENTER
  {
  	let mut i: c_int = num;

    if key == A_MOUSE2
    {
        i -= 1;
    }
    else
    {
        i += 1;
    }

    if i < min
    {
        i = max;
    }
    else if i > max
    {
        i = min;
    }

    let num = i;

    uiSkinColor = num;

    uiHoldSkinColor = uiSkinColor;

    UI_FeederSelection(FEEDER_Q3HEADS, uiInfo.q3SelectedHead, core::ptr::null_mut());

    return qtrue;
  }
  return qfalse;
}




pub unsafe fn UI_ForceSide_HandleKey(flags: c_int, special: *mut f32, key: c_int, num: c_int, min: c_int, max: c_int, r#type: c_int) -> qboolean
{
    let mut info: [c_char; MAX_INFO_VALUE as usize] = [0; MAX_INFO_VALUE as usize];

    info[0] = '\0' as c_char;
    trap_GetConfigString(CS_SERVERINFO, info.as_mut_ptr(), (MAX_INFO_VALUE as c_int));

    if atoi( Info_ValueForKey( info.as_ptr(), b"g_forceBasedTeams\0".as_ptr() as *const c_char ) ) != 0
    {
        match (trap_Cvar_VariableValue(b"ui_myteam\0".as_ptr() as *const c_char) as c_int) {
        TEAM_RED => {
            return qfalse;
        },
        TEAM_BLUE => {
            return qfalse;
        },
        _ => {
        },
        }
    }

    if key == A_MOUSE1 || key == A_MOUSE2 || key == A_ENTER || key == A_KP_ENTER
    {
        let mut i: c_int = num;
        let mut x: c_int = 0;

        //update the feeder item selection, it might be different depending on side
        Menu_SetFeederSelection(core::ptr::null_mut(), FEEDER_FORCECFG, 0, core::ptr::null_mut());

        if key == A_MOUSE2
        {
            i -= 1;
        }
        else
        {
            i += 1;
        }

        if i < min
        {
            i = max;
        }
        else if i > max
        {
            i = min;
        }

        let num = i;

        uiForceSide = num;

        // Resetting power ranks based on if light or dark side is chosen
        while x < NUM_FORCE_POWERS as c_int
        {
            if uiForcePowerDarkLight[x as usize] != 0 && uiForceSide != uiForcePowerDarkLight[x as usize]
            {
                uiForcePowersRank[x as usize] = 0;
            }
            x += 1;
        }

        UpdateForceUsed();

        gTouchedForce = qtrue;
        return qtrue;
    }
    return qfalse;
}

pub unsafe fn UI_JediNonJedi_HandleKey(flags: c_int, special: *mut f32, key: c_int, num: c_int, min: c_int, max: c_int, r#type: c_int) -> qboolean
{
    let mut info: [c_char; MAX_INFO_VALUE as usize] = [0; MAX_INFO_VALUE as usize];

    info[0] = '\0' as c_char;
    trap_GetConfigString(CS_SERVERINFO, info.as_mut_ptr(), (MAX_INFO_VALUE as c_int));

    if UI_TrueJediEnabled() == 0
    {//true jedi mode is not set
        return qfalse;
    }

    if key == A_MOUSE1 || key == A_MOUSE2 || key == A_ENTER || key == A_KP_ENTER
    {
        let mut i: c_int = num;
        let mut x: c_int = 0;

        if key == A_MOUSE2
        {
            i -= 1;
        }
        else
        {
            i += 1;
        }

        if i < min
        {
            i = max;
        }
        else if i > max
        {
            i = min;
        }

        let num = i;

        uiJediNonJedi = num;

        // Resetting power ranks based on if light or dark side is chosen
        if num == 0
        {//not a jedi?
            let myTeam: c_int = trap_Cvar_VariableValue(b"ui_myteam\0".as_ptr() as *const c_char) as c_int;
            while x < NUM_FORCE_POWERS as c_int
            {//clear all force powers
                uiForcePowersRank[x as usize] = 0;
                x += 1;
            }
            if myTeam != TEAM_SPECTATOR
            {
                UI_UpdateClientForcePowers(UI_TeamName(myTeam));//will cause him to respawn, if it's been 5 seconds since last one
            }
            else
            {
                UI_UpdateClientForcePowers(core::ptr::null());//just update powers
            }
        }
        else if num != 0
        {//a jedi, set the minimums, hopefuly they know to set the rest!
            if uiForcePowersRank[FP_LEVITATION] < FORCE_LEVEL_1
            {//force jump 1 minimum
                uiForcePowersRank[FP_LEVITATION] = FORCE_LEVEL_1;
            }
            if uiForcePowersRank[FP_SABER_OFFENSE] < FORCE_LEVEL_1
            {//saber attack 1, minimum
                uiForcePowersRank[FP_SABER_OFFENSE] = FORCE_LEVEL_1;
            }
        }

        UpdateForceUsed();

        gTouchedForce = qtrue;
        return qtrue;
    }
    return qfalse;
}

pub unsafe fn UI_ForceMaxRank_HandleKey(flags: c_int, special: *mut f32, key: c_int, num: c_int, min: c_int, max: c_int, r#type: c_int) -> qboolean
{
  if key == A_MOUSE1 || key == A_MOUSE2 || key == A_ENTER || key == A_KP_ENTER
  {
  	let mut i: c_int = num;

    if key == A_MOUSE2
    {
        i -= 1;
    }
    else
    {
        i += 1;
    }

    if i < min
    {
        i = max;
    }
    else if i > max
    {
        i = min;
    }

    let num = i;

    uiMaxRank = num;

    trap_Cvar_Set( b"g_maxForceRank\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, num));

    // The update force used will remove overallocated powers automatically.
    UpdateForceUsed();

    gTouchedForce = qtrue;

    return qtrue;
  }
  return qfalse;
}


// This function will either raise or lower a power by one rank.
pub unsafe fn UI_ForcePowerRank_HandleKey(flags: c_int, special: *mut f32, key: c_int, num: c_int, min: c_int, mut max: c_int, r#type: c_int) -> qboolean
{
    let mut raising: qboolean;

    if key == A_MOUSE1 || key == A_MOUSE2 || key == A_ENTER || key == A_KP_ENTER || key == A_BACKSPACE
    {
        let mut forcepower: c_int;
        let mut rank: c_int;

        //this will give us the index as long as UI_FORCE_RANK is always one below the first force rank index
        forcepower = (r#type-UI_FORCE_RANK)-1;

        //the power is disabled on the server
        if uiForcePowersDisabled[forcepower as usize] != 0
        {
            return qtrue;
        }

        // If we are not on the same side as a power, or if we are not of any rank at all.
        if uiForcePowerDarkLight[forcepower as usize] != 0 && uiForceSide != uiForcePowerDarkLight[forcepower as usize]
        {
            return qtrue;
        }
        else if forcepower == FP_SABER_DEFENSE as c_int || forcepower == FP_SABERTHROW as c_int
        {	// Saberdefend and saberthrow can't be bought if there is no saberattack
            if uiForcePowersRank[FP_SABER_OFFENSE] < 1
            {
                return qtrue;
            }
        }

        let mut min = min;
        if r#type == UI_FORCE_RANK_LEVITATION
        {
            min += 1;
        }
        if r#type == UI_FORCE_RANK_SABERATTACK && ui_freeSaber.integer != 0
        {
            min += 1;
        }
        if r#type == UI_FORCE_RANK_SABERDEFEND && ui_freeSaber.integer != 0
        {
            min += 1;
        }

        if key == A_MOUSE2 || key == A_BACKSPACE
        {	// Lower a point.
            if uiForcePowersRank[forcepower as usize]<=min
            {
                return qtrue;
            }
            raising = qfalse;
        }
        else
        {	// Raise a point.
            if uiForcePowersRank[forcepower as usize]>=max
            {
                return qtrue;
            }
            raising = qtrue;
        }

        if raising != 0
        {	// Check if we can accrue the cost of this power.
            rank = uiForcePowersRank[forcepower as usize]+1;
            if bgForcePowerCost[forcepower as usize][rank as usize] > uiForceAvailable
            {	// We can't afford this power.  Abandon ship.
                return qtrue;
            }
            else
            {	// Sure we can afford it.
                uiForceUsed += bgForcePowerCost[forcepower as usize][rank as usize];
                uiForceAvailable -= bgForcePowerCost[forcepower as usize][rank as usize];
                uiForcePowersRank[forcepower as usize]=rank;
            }
        }
        else
        {	// Lower the point.
            rank = uiForcePowersRank[forcepower as usize];
            uiForceUsed -= bgForcePowerCost[forcepower as usize][rank as usize];
            uiForceAvailable += bgForcePowerCost[forcepower as usize][rank as usize];
            uiForcePowersRank[forcepower as usize] -= 1;
        }

        UpdateForceUsed();

        gTouchedForce = qtrue;

        return qtrue;
    }
    return qfalse;
}


pub static mut gCustRank: c_int = 0;
pub static mut gCustSide: c_int = 0;

pub static mut gCustPowersRank: [c_int; NUM_FORCE_POWERS as usize] = [
    0,//FP_HEAL = 0,//instant
    1,//FP_LEVITATION,//hold/duration, this one defaults to 1 (gives a free point)
    0,//FP_SPEED,//duration
    0,//FP_PUSH,//hold/duration
    0,//FP_PULL,//hold/duration
    0,//FP_TELEPATHY,//instant
    0,//FP_GRIP,//hold/duration
    0,//FP_LIGHTNING,//hold/duration
    0,//FP_RAGE,//duration
    0,//FP_PROTECT,
    0,//FP_ABSORB,
    0,//FP_TEAM_HEAL,
    0,//FP_TEAM_FORCE,
    0,//FP_DRAIN,
    0,//FP_SEE,
    0,//FP_SABER_OFFENSE,
    0,//FP_SABER_DEFENSE,
    0//FP_SABERTHROW,
];

/*
=================
UI_ForceConfigHandle
=================
*/
pub unsafe fn UI_ForceConfigHandle( oldindex: c_int, newindex: c_int )
{
    let mut f: fileHandle_t = 0;
    let mut len: c_int = 0;
    let mut i: c_int = 0;
    let mut c: usize = 0;
    let mut iBuf: c_int = 0;
    let mut forcePowerRank: c_int;
    let mut currank: c_int;
    let mut fcfBuffer: [c_char; 8192] = [0; 8192];
    let mut singleBuf: [c_char; 64] = [0; 64];
    let mut info: [c_char; MAX_INFO_VALUE as usize] = [0; MAX_INFO_VALUE as usize];
    let mut forceTeam: c_int = 0;
    let mut newindex = newindex;

    if oldindex == 0
    { //switching out from custom config, so first shove the current values into the custom storage
        i = 0;

        while i < NUM_FORCE_POWERS as c_int
        {
            gCustPowersRank[i as usize] = uiForcePowersRank[i as usize];
            i += 1;
        }
        gCustRank = uiForceRank;
        gCustSide = uiForceSide;
    }

    if newindex == 0
    { //switching back to custom, shove the values back in from the custom storage
        i = 0;
        uiForceUsed = 0;
        gTouchedForce = qtrue;

        while i < NUM_FORCE_POWERS as c_int
        {
            uiForcePowersRank[i as usize] = gCustPowersRank[i as usize];
            uiForceUsed += uiForcePowersRank[i as usize];
            i += 1;
        }
        uiForceRank = gCustRank;
        uiForceSide = gCustSide;

        UpdateForceUsed();
        return;
    }

    //If we made it here, we want to load in a new config
    if uiForceSide == FORCE_LIGHTSIDE
    { //we should only be displaying lightside configs, so.. look in the light folder
        newindex += uiInfo.forceConfigLightIndexBegin;
        if newindex >= uiInfo.forceConfigCount
        {
            return;
        }
        len = trap_FS_FOpenFile(va(b"forcecfg/light/%s.fcf\0".as_ptr() as *const c_char, uiInfo.forceConfigNames[newindex as usize]), addr_of_mut!(f), FS_READ);
    }
    else
    { //else dark
        newindex += uiInfo.forceConfigDarkIndexBegin;
        if newindex >= uiInfo.forceConfigCount || newindex > uiInfo.forceConfigLightIndexBegin
        { //dark gets read in before light
            return;
        }
        len = trap_FS_FOpenFile(va(b"forcecfg/dark/%s.fcf\0".as_ptr() as *const c_char, uiInfo.forceConfigNames[newindex as usize]), addr_of_mut!(f), FS_READ);
    }

    if len <= 0
    { //This should not have happened. But, before we quit out, attempt searching the other light/dark folder for the file.
        if uiForceSide == FORCE_LIGHTSIDE
        {
            len = trap_FS_FOpenFile(va(b"forcecfg/dark/%s.fcf\0".as_ptr() as *const c_char, uiInfo.forceConfigNames[newindex as usize]), addr_of_mut!(f), FS_READ);
        }
        else
        {
            len = trap_FS_FOpenFile(va(b"forcecfg/light/%s.fcf\0".as_ptr() as *const c_char, uiInfo.forceConfigNames[newindex as usize]), addr_of_mut!(f), FS_READ);
        }

        if len <= 0
        { //still failure? Oh well.
            return;
        }
    }

    if len >= 8192
    {
        return;
    }

    trap_FS_Read(fcfBuffer.as_mut_ptr() as *mut c_void, len, f);
    fcfBuffer[len as usize] = 0;
    trap_FS_FCloseFile(f);

    i = 0;
    c = 0;

    info[0] = '\0' as c_char;
    trap_GetConfigString(CS_SERVERINFO, info.as_mut_ptr(), (MAX_INFO_VALUE as c_int));

    if atoi( Info_ValueForKey( info.as_ptr(), b"g_forceBasedTeams\0".as_ptr() as *const c_char ) ) != 0
    {
        match (trap_Cvar_VariableValue(b"ui_myteam\0".as_ptr() as *const c_char) as c_int) {
        TEAM_RED => {
            forceTeam = FORCE_DARKSIDE;
        },
        TEAM_BLUE => {
            forceTeam = FORCE_LIGHTSIDE;
        },
        _ => {
        },
        }
    }

    BG_LegalizedForcePowers(fcfBuffer.as_mut_ptr(), uiMaxRank, ui_freeSaber.integer, forceTeam, atoi( Info_ValueForKey( info.as_ptr(), b"g_gametype\0".as_ptr() as *const c_char )), 0);
    //legalize the config based on the max rank

    //now that we're done with the handle, it's time to parse our force data out of the string
    //we store strings in rank-side-xxxxxxxxx format (where the x's are individual force power levels)
    while fcfBuffer[i as usize] != 0 && fcfBuffer[i as usize] != '-' as c_char
    {
        singleBuf[c] = fcfBuffer[i as usize];
        c += 1;
        i += 1;
    }
    singleBuf[c] = 0;
    c = 0;
    i += 1;

    iBuf = atoi(singleBuf.as_ptr());

    if iBuf > uiMaxRank || iBuf < 0
    { //this force config uses a rank level higher than our currently restricted level.. so we can't use it
      //FIXME: Print a message indicating this to the user
        return;
    }

    uiForceRank = iBuf;

    while fcfBuffer[i as usize] != 0 && fcfBuffer[i as usize] != '-' as c_char
    {
        singleBuf[c] = fcfBuffer[i as usize];
        c += 1;
        i += 1;
    }
    singleBuf[c] = 0;
    c = 0;
    i += 1;

    uiForceSide = atoi(singleBuf.as_ptr());

    if uiForceSide != FORCE_LIGHTSIDE &&
        uiForceSide != FORCE_DARKSIDE
    {
        uiForceSide = FORCE_LIGHTSIDE;
        return;
    }

    //clear out the existing powers
    c = 0;
    while c < NUM_FORCE_POWERS as usize
    {
        /*
        if (c==FP_LEVITATION)
        {
            uiForcePowersRank[c]=1;
        }
        else if (c==FP_SABER_OFFENSE && ui_freeSaber.integer)
        {
            uiForcePowersRank[c]=1;
        }
        else if (c==FP_SABER_DEFENSE && ui_freeSaber.integer)
        {
            uiForcePowersRank[c]=1;
        }
        else
        {
            uiForcePowersRank[c] = 0;
        }
        */
        //rww - don't need to do these checks. Just trust whatever the saber config says.
        uiForcePowersRank[c] = 0;
        c += 1;
    }
    uiForceUsed = 0;
    uiForceAvailable = forceMasteryPoints[uiForceRank as usize];
    gTouchedForce = qtrue;

    c = 0;
    while fcfBuffer[i as usize] != 0 && c < NUM_FORCE_POWERS as usize {
        singleBuf[0] = fcfBuffer[i as usize];
        singleBuf[1] = 0;
        iBuf = atoi(singleBuf.as_ptr());	// So, that means that Force Power "c" wants to be set to rank "iBuf".

        if iBuf < 0
        {
            iBuf = 0;
        }

        forcePowerRank = iBuf;

        if forcePowerRank > FORCE_LEVEL_3 || forcePowerRank < 0
        { //err..  not correct
            c += 1;
            i += 1;
            continue;  // skip this power
        }

        if uiForcePowerDarkLight[c] != 0 && uiForcePowerDarkLight[c] != uiForceSide
        { //Apparently the user has crafted a force config that has powers that don't fit with the config's side.
            c += 1;
            i += 1;
            continue;  // skip this power
        }

        // Accrue cost for each assigned rank for this power.
        currank = FORCE_LEVEL_1;
        while currank <= forcePowerRank {
            if bgForcePowerCost[c][currank as usize] > uiForceAvailable
            {	// Break out, we can't afford any more power.
                break;
            }
            // Pay for this rank of this power.
            uiForceUsed += bgForcePowerCost[c][currank as usize];
            uiForceAvailable -= bgForcePowerCost[c][currank as usize];

            uiForcePowersRank[c] += 1;
            currank += 1;
        }
        c += 1;
        i += 1;
    }

    if uiForcePowersRank[FP_LEVITATION] < 1
    {
        uiForcePowersRank[FP_LEVITATION]=1;
    }
    if uiForcePowersRank[FP_SABER_OFFENSE] < 1 && ui_freeSaber.integer != 0
    {
        uiForcePowersRank[FP_SABER_OFFENSE]=1;
    }
    if uiForcePowersRank[FP_SABER_DEFENSE] < 1 && ui_freeSaber.integer != 0
    {
        uiForcePowersRank[FP_SABER_DEFENSE]=1;
    }

    UpdateForceUsed();
}
