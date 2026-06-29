// Copyright (C) 1999-2000 Id Software, Inc.
//

// This module mirrors oracle/codemp/ui/ui_local.h
// Original #ifndef __UI_LOCAL_H__
// Original #define __UI_LOCAL_H__

// Stub type declarations for external dependencies not yet ported
// These represent types defined in the included headers
pub type vmCvar_t = i32; // Defined in q_shared.h
pub type qboolean = i32; // Defined in q_shared.h
pub type qhandle_t = i32; // Defined in q_shared.h
pub type sfxHandle_t = i32; // Defined in tr_types.h
pub type vec3_t = [f32; 3]; // Defined in q_shared.h
pub type vec4_t = [f32; 4]; // Defined in q_shared.h
pub type glconfig_t = u32; // Defined in tr_types.h - placeholder
pub type displayContextDef_t = u32; // Defined in ui_shared.h - placeholder
pub type refEntity_t = u32; // Defined in tr_types.h - placeholder
pub type polyVert_t = u32; // Defined in tr_types.h - placeholder
pub type refdef_t = u32; // Defined in tr_types.h - placeholder
pub type orientation_t = u32; // Defined in tr_types.h - placeholder
pub type clipHandle_t = i32; // Defined in q_shared.h
pub type uiClientState_t = u32; // Defined in ui_public.h - placeholder
pub type fileHandle_t = i32; // Defined in q_shared.h
pub type fsMode_t = i32; // Defined in q_shared.h
pub type itemDef_t = u32; // Defined in ui_shared.h - placeholder
pub type uiMenuCommand_t = i32; // Defined in ui_public.h
pub type weapon_t = i32; // Defined in bg_public.h
pub type e_status = i32; // Defined in cl_cin.h - placeholder
pub type qtime_t = u32; // Defined in q_shared.h - placeholder
pub type animation_t = u32; // Defined in bg_public.h - placeholder

pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_CLIENTS: usize = 32;
pub const MAX_GAMEINFO_SIZE: usize = 16384;
pub const MAX_STRING_CHARS: usize = 1024;
pub const BIGCHAR_WIDTH: i32 = 16;
pub const MAX_SERVERSTATUSREQUESTS: usize = 16;

use std::ffi::c_char;
use core::ffi::{c_int, c_uint, c_void};

#[allow(non_snake_case)]
extern "C" {
    // global display context

    pub static mut ui_ffa_fraglimit: vmCvar_t;
    pub static mut ui_ffa_timelimit: vmCvar_t;

    pub static mut ui_selectedModelIndex: vmCvar_t;

    pub static mut ui_team_fraglimit: vmCvar_t;
    pub static mut ui_team_timelimit: vmCvar_t;
    pub static mut ui_team_friendly: vmCvar_t;

    pub static mut ui_ctf_capturelimit: vmCvar_t;
    pub static mut ui_ctf_timelimit: vmCvar_t;
    pub static mut ui_ctf_friendly: vmCvar_t;

    pub static mut ui_arenasFile: vmCvar_t;
    pub static mut ui_botsFile: vmCvar_t;

    pub static mut ui_browserMaster: vmCvar_t;
    pub static mut ui_browserGameType: vmCvar_t;
    pub static mut ui_browserSortKey: vmCvar_t;
    pub static mut ui_browserShowFull: vmCvar_t;
    pub static mut ui_browserShowEmpty: vmCvar_t;

    pub static mut ui_drawCrosshair: vmCvar_t;
    pub static mut ui_drawCrosshairNames: vmCvar_t;
    pub static mut ui_marks: vmCvar_t;

    pub static mut ui_captureLimit: vmCvar_t;
    pub static mut ui_fragLimit: vmCvar_t;
    pub static mut ui_gameType: vmCvar_t;
    pub static mut ui_netGameType: vmCvar_t;
    pub static mut ui_actualNetGameType: vmCvar_t;
    pub static mut ui_joinGameType: vmCvar_t;
    #[cfg(feature = "xbox")]
    pub static mut ui_optiGameType: vmCvar_t;
    #[cfg(feature = "xbox")]
    pub static mut ui_optiCurrentMap: vmCvar_t;
    #[cfg(feature = "xbox")]
    pub static mut ui_optiMinPlayers: vmCvar_t;
    #[cfg(feature = "xbox")]
    pub static mut ui_optiMaxPlayers: vmCvar_t;
    #[cfg(feature = "xbox")]
    pub static mut ui_optiFriendlyFire: vmCvar_t;
    #[cfg(feature = "xbox")]
    pub static mut ui_optiJediMastery: vmCvar_t;
    #[cfg(feature = "xbox")]
    pub static mut ui_optiSaberOnly: vmCvar_t;
    pub static mut ui_netSource: vmCvar_t;
    pub static mut ui_serverFilterType: vmCvar_t;
    pub static mut ui_dedicated: vmCvar_t;
    pub static mut ui_opponentName: vmCvar_t;
    pub static mut ui_menuFiles: vmCvar_t;
    pub static mut ui_currentMap: vmCvar_t;
    pub static mut ui_currentNetMap: vmCvar_t;
    pub static mut ui_mapIndex: vmCvar_t;
    pub static mut ui_currentOpponent: vmCvar_t;
    pub static mut ui_selectedPlayer: vmCvar_t;
    pub static mut ui_selectedPlayerName: vmCvar_t;
    pub static mut ui_lastServerRefresh_0: vmCvar_t;
    pub static mut ui_lastServerRefresh_1: vmCvar_t;
    pub static mut ui_lastServerRefresh_2: vmCvar_t;
    pub static mut ui_lastServerRefresh_3: vmCvar_t;
    pub static mut ui_singlePlayerActive: vmCvar_t;
    pub static mut ui_scoreAccuracy: vmCvar_t;
    pub static mut ui_scoreImpressives: vmCvar_t;
    pub static mut ui_scoreExcellents: vmCvar_t;
    pub static mut ui_scoreDefends: vmCvar_t;
    pub static mut ui_scoreAssists: vmCvar_t;
    pub static mut ui_scoreGauntlets: vmCvar_t;
    pub static mut ui_scoreScore: vmCvar_t;
    pub static mut ui_scorePerfect: vmCvar_t;
    pub static mut ui_scoreTeam: vmCvar_t;
    pub static mut ui_scoreBase: vmCvar_t;
    pub static mut ui_scoreTimeBonus: vmCvar_t;
    pub static mut ui_scoreSkillBonus: vmCvar_t;
    pub static mut ui_scoreShutoutBonus: vmCvar_t;
    pub static mut ui_scoreTime: vmCvar_t;
    pub static mut ui_serverStatusTimeOut: vmCvar_t;

    pub static mut ui_bypassMainMenuLoad: vmCvar_t;
}

//
// ui_qmenu.c
//

pub const RCOLUMN_OFFSET: i32 = BIGCHAR_WIDTH;
pub const LCOLUMN_OFFSET: i32 = -BIGCHAR_WIDTH;

pub const SLIDER_RANGE: i32 = 10;
pub const MAX_EDIT_LINE: usize = 256;

pub const MAX_MENUDEPTH: usize = 8;
pub const MAX_MENUITEMS: usize = 256;

pub const MAX_FORCE_CONFIGS: usize = 128;

pub const MTYPE_NULL: i32 = 0;
pub const MTYPE_SLIDER: i32 = 1;
pub const MTYPE_ACTION: i32 = 2;
pub const MTYPE_SPINCONTROL: i32 = 3;
pub const MTYPE_FIELD: i32 = 4;
pub const MTYPE_RADIOBUTTON: i32 = 5;
pub const MTYPE_BITMAP: i32 = 6;
pub const MTYPE_TEXT: i32 = 7;
pub const MTYPE_SCROLLLIST: i32 = 8;
pub const MTYPE_PTEXT: i32 = 9;
pub const MTYPE_BTEXT: i32 = 10;

pub const QMF_BLINK: i32 = 0x00000001;
pub const QMF_SMALLFONT: i32 = 0x00000002;
pub const QMF_LEFT_JUSTIFY: i32 = 0x00000004;
pub const QMF_CENTER_JUSTIFY: i32 = 0x00000008;
pub const QMF_RIGHT_JUSTIFY: i32 = 0x00000010;
pub const QMF_NUMBERSONLY: i32 = 0x00000020; // edit field is only numbers
pub const QMF_HIGHLIGHT: i32 = 0x00000040;
pub const QMF_HIGHLIGHT_IF_FOCUS: i32 = 0x00000080; // steady focus
pub const QMF_PULSEIFFOCUS: i32 = 0x00000100; // pulse if focus
pub const QMF_HASMOUSEFOCUS: i32 = 0x00000200;
pub const QMF_NOONOFFTEXT: i32 = 0x00000400;
pub const QMF_MOUSEONLY: i32 = 0x00000800; // only mouse input allowed
pub const QMF_HIDDEN: i32 = 0x00001000; // skips drawing
pub const QMF_GRAYED: i32 = 0x00002000; // grays and disables
pub const QMF_INACTIVE: i32 = 0x00004000; // disables any input
pub const QMF_NODEFAULTINIT: i32 = 0x00008000; // skip default initialization
pub const QMF_OWNERDRAW: i32 = 0x00010000;
pub const QMF_PULSE: i32 = 0x00020000;
pub const QMF_LOWERCASE: i32 = 0x00040000; // edit field is all lower case
pub const QMF_UPPERCASE: i32 = 0x00080000; // edit field is all upper case
pub const QMF_SILENT: i32 = 0x00100000;

// callback notifications
pub const QM_GOTFOCUS: i32 = 1;
pub const QM_LOSTFOCUS: i32 = 2;
pub const QM_ACTIVATED: i32 = 3;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menuframework_s {
    pub cursor: c_int,
    pub cursor_prev: c_int,

    pub nitems: c_int,
    pub items: [*mut c_void; MAX_MENUITEMS],

    pub draw: Option<extern "C" fn()>,
    pub key: Option<extern "C" fn(key: c_int) -> sfxHandle_t>,

    pub wrapAround: qboolean,
    pub fullscreen: qboolean,
    pub showlogo: qboolean,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menucommon_s {
    pub r#type: c_int,
    pub name: *const c_char,
    pub id: c_int,
    pub x: c_int,
    pub y: c_int,
    pub left: c_int,
    pub top: c_int,
    pub right: c_int,
    pub bottom: c_int,
    pub parent: *mut menuframework_s,
    pub menuPosition: c_int,
    pub flags: c_uint,

    pub callback: Option<extern "C" fn(*mut c_void, c_int)>,
    pub statusbar: Option<extern "C" fn(*mut c_void)>,
    pub ownerdraw: Option<extern "C" fn(*mut c_void)>,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct mfield_t {
    pub cursor: c_int,
    pub scroll: c_int,
    pub widthInChars: c_int,
    pub buffer: [c_char; MAX_EDIT_LINE],
    pub maxchars: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menufield_s {
    pub generic: menucommon_s,
    pub field: mfield_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menuslider_s {
    pub generic: menucommon_s,

    pub minvalue: f32,
    pub maxvalue: f32,
    pub curvalue: f32,

    pub range: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menulist_s {
    pub generic: menucommon_s,

    pub oldvalue: c_int,
    pub curvalue: c_int,
    pub numitems: c_int,
    pub top: c_int,

    pub itemnames: *const *const c_char,

    pub width: c_int,
    pub height: c_int,
    pub columns: c_int,
    pub seperation: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menuaction_s {
    pub generic: menucommon_s,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menuradiobutton_s {
    pub generic: menucommon_s,
    pub curvalue: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menubitmap_s {
    pub generic: menucommon_s,
    pub focuspic: *mut c_char,
    pub errorpic: *mut c_char,
    pub shader: qhandle_t,
    pub focusshader: qhandle_t,
    pub width: c_int,
    pub height: c_int,
    pub focuscolor: *mut f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct menutext_s {
    pub generic: menucommon_s,
    pub string: *mut c_char,
    pub style: c_int,
    pub color: *mut f32,
}

extern "C" {
    pub fn Menu_Cache();
    pub fn Menu_Focus(m: *mut menucommon_s);
    pub fn Menu_AddItem(menu: *mut menuframework_s, item: *mut c_void);
    pub fn Menu_AdjustCursor(menu: *mut menuframework_s, dir: c_int);
    pub fn Menu_Draw(menu: *mut menuframework_s);
    pub fn Menu_ItemAtCursor(m: *mut menuframework_s) -> *mut c_void;
    pub fn Menu_ActivateItem(s: *mut menuframework_s, item: *mut menucommon_s) -> sfxHandle_t;
    pub fn Menu_SetCursor(s: *mut menuframework_s, cursor: c_int);
    pub fn Menu_SetCursorToItem(m: *mut menuframework_s, ptr: *mut c_void);
    pub fn Menu_DefaultKey(s: *mut menuframework_s, key: c_int) -> sfxHandle_t;
    pub fn Bitmap_Init(b: *mut menubitmap_s);
    pub fn Bitmap_Draw(b: *mut menubitmap_s);
    pub fn ScrollList_Draw(l: *mut menulist_s);
    pub fn ScrollList_Key(l: *mut menulist_s, key: c_int) -> sfxHandle_t;
    pub static mut menu_in_sound: sfxHandle_t;
    pub static mut menu_move_sound: sfxHandle_t;
    pub static mut menu_out_sound: sfxHandle_t;
    pub static mut menu_buzz_sound: sfxHandle_t;
    pub static mut menu_null_sound: sfxHandle_t;
    pub static mut weaponChangeSound: sfxHandle_t;
    pub static mut menu_text_color: vec4_t;
    pub static mut menu_grayed_color: vec4_t;
    pub static mut menu_dark_color: vec4_t;
    pub static mut menu_highlight_color: vec4_t;
    pub static mut menu_red_color: vec4_t;
    pub static mut menu_black_color: vec4_t;
    pub static mut menu_dim_color: vec4_t;
    pub static mut color_black: vec4_t;
    pub static mut color_white: vec4_t;
    pub static mut color_yellow: vec4_t;
    pub static mut color_blue: vec4_t;
    pub static mut color_orange: vec4_t;
    pub static mut color_red: vec4_t;
    pub static mut color_dim: vec4_t;
    pub static mut name_color: vec4_t;
    pub static mut list_color: vec4_t;
    pub static mut listbar_color: vec4_t;
    pub static mut text_color_disabled: vec4_t;
    pub static mut text_color_normal: vec4_t;
    pub static mut text_color_highlight: vec4_t;

    pub static mut ui_medalNames: [*mut c_char; 256];
    pub static mut ui_medalPicNames: [*mut c_char; 256];
    pub static mut ui_medalSounds: [*mut c_char; 256];
}

//
// ui_mfield.c
//
extern "C" {
    pub fn MField_Clear(edit: *mut mfield_t);
    pub fn MField_KeyDownEvent(edit: *mut mfield_t, key: c_int);
    pub fn MField_CharEvent(edit: *mut mfield_t, ch: c_int);
    pub fn MField_Draw(edit: *mut mfield_t, x: c_int, y: c_int, style: c_int, color: vec4_t);
    pub fn MenuField_Init(m: *mut menufield_s);
    pub fn MenuField_Draw(f: *mut menufield_s);
    pub fn MenuField_Key(m: *mut menufield_s, key: *mut c_int) -> sfxHandle_t;
}

//
// ui_main.c
//
extern "C" {
    pub fn UI_FeederSelection(feederID: f32, index: c_int, item: *mut itemDef_t) -> qboolean;
    pub fn UI_Report();
    pub fn UI_Load();
    pub fn UI_LoadMenus(menuFile: *const c_char, reset: qboolean);
    pub fn _UI_SetActiveMenu(menu: uiMenuCommand_t);
    pub fn UI_AdjustTimeByGame(time: c_int) -> c_int;
    pub fn UI_ShowPostGame(newHigh: qboolean);
    pub fn UI_ClearScores();
    pub fn UI_LoadArenas();
    pub fn UI_LoadForceConfig_List();
}

//
// ui_menu.c
//
extern "C" {
    pub fn MainMenu_Cache();
    pub fn UI_MainMenu();
    pub fn UI_RegisterCvars();
    pub fn UI_UpdateCvars();
}

//
// ui_credits.c
//
extern "C" {
    pub fn UI_CreditMenu();
}

//
// ui_ingame.c
//
extern "C" {
    pub fn InGame_Cache();
    pub fn UI_InGameMenu();
}

//
// ui_confirm.c
//
extern "C" {
    pub fn ConfirmMenu_Cache();
    pub fn UI_ConfirmMenu(
        question: *const c_char,
        draw: Option<extern "C" fn()>,
        action: Option<extern "C" fn(qboolean)>,
    );
}

//
// ui_setup.c
//
extern "C" {
    pub fn UI_SetupMenu_Cache();
    pub fn UI_SetupMenu();
}

//
// ui_team.c
//
extern "C" {
    pub fn UI_TeamMainMenu();
    pub fn TeamMain_Cache();
}

//
// ui_connect.c
//
extern "C" {
    pub fn UI_DrawConnectScreen(overlay: qboolean);
}

//
// ui_controls2.c
//
extern "C" {
    pub fn UI_ControlsMenu();
    pub fn Controls_Cache();
}

//
// ui_demo2.c
//
extern "C" {
    pub fn UI_DemosMenu();
    pub fn Demos_Cache();
}

//
// ui_cinematics.c
//
extern "C" {
    pub fn UI_CinematicsMenu();
    pub fn UI_CinematicsMenu_f();
    pub fn UI_CinematicsMenu_Cache();
}

//
// ui_mods.c
//
extern "C" {
    pub fn UI_ModsMenu();
    pub fn UI_ModsMenu_Cache();
}

//
// ui_cdkey.c
//
extern "C" {
    pub fn UI_CDKeyMenu();
    pub fn UI_CDKeyMenu_Cache();
    pub fn UI_CDKeyMenu_f();
}

//
// ui_playermodel.c
//
extern "C" {
    pub fn UI_PlayerModelMenu();
    pub fn PlayerModel_Cache();
}

//
// ui_playersettings.c
//
extern "C" {
    pub fn UI_PlayerSettingsMenu();
    pub fn PlayerSettings_Cache();
}

//
// ui_preferences.c
//
extern "C" {
    pub fn UI_PreferencesMenu();
    pub fn Preferences_Cache();
}

//
// ui_specifyleague.c
//
extern "C" {
    pub fn UI_SpecifyLeagueMenu();
    pub fn SpecifyLeague_Cache();
}

//
// ui_specifyserver.c
//
extern "C" {
    pub fn UI_SpecifyServerMenu();
    pub fn SpecifyServer_Cache();
}

//
// ui_servers2.c
//
pub const MAX_FAVORITESERVERS: usize = 16;

extern "C" {
    pub fn UI_ArenaServersMenu();
    pub fn ArenaServers_Cache();
}

//
// ui_startserver.c
//
extern "C" {
    pub fn UI_StartServerMenu(multiplayer: qboolean);
    pub fn StartServer_Cache();
    pub fn ServerOptions_Cache();
    pub fn UI_BotSelectMenu(bot: *mut c_char);
    pub fn UI_BotSelectMenu_Cache();
}

//
// ui_serverinfo.c
//
extern "C" {
    pub fn UI_ServerInfoMenu();
    pub fn ServerInfo_Cache();
}

//
// ui_video.c
//
extern "C" {
    pub fn UI_GraphicsOptionsMenu();
    pub fn GraphicsOptions_Cache();
    pub fn DriverInfo_Cache();
}

//
// ui_players.c
//

// FIXME ripped from cg_local.h
#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct lerpFrame_t {
    pub oldFrame: c_int,
    pub oldFrameTime: c_int, // time when ->oldFrame was exactly on

    pub frame: c_int,
    pub frameTime: c_int, // time when ->frame will be exactly on

    pub backlerp: f32,

    pub yawAngle: f32,
    pub yawing: qboolean,
    pub pitchAngle: f32,
    pub pitching: qboolean,

    pub animationNumber: c_int,
    pub animation: *mut animation_t,
    pub animationTime: c_int, // time when the first frame of the animation will be exact
}

pub const MAX_TOTALANIMATIONS: usize = 128; // Placeholder value, adjust as needed

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct playerInfo_t {
    // model info
    pub legsModel: qhandle_t,
    pub legsSkin: qhandle_t,
    pub legs: lerpFrame_t,

    pub torsoModel: qhandle_t,
    pub torsoSkin: qhandle_t,
    pub torso: lerpFrame_t,

    // qhandle_t		headModel;
    // qhandle_t		headSkin;

    pub animations: [animation_t; MAX_TOTALANIMATIONS],

    pub weaponModel: qhandle_t,
    pub barrelModel: qhandle_t,
    pub flashModel: qhandle_t,
    pub flashDlightColor: vec3_t,
    pub muzzleFlashTime: c_int,

    // currently in use drawing parms
    pub viewAngles: vec3_t,
    pub moveAngles: vec3_t,
    pub currentWeapon: weapon_t,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,

    // animation vars
    pub weapon: weapon_t,
    pub lastWeapon: weapon_t,
    pub pendingWeapon: weapon_t,
    pub weaponTimer: c_int,
    pub pendingLegsAnim: c_int,
    pub torsoAnimationTimer: c_int,

    pub pendingTorsoAnim: c_int,
    pub legsAnimationTimer: c_int,

    pub chat: qboolean,
    pub newModel: qboolean,

    pub barrelSpinning: qboolean,
    pub barrelAngle: f32,
    pub barrelTime: c_int,

    pub realWeapon: c_int,
}

// void UI_DrawPlayer( float x, float y, float w, float h, playerInfo_t *pi, int time );
// void UI_PlayerInfo_SetModel( playerInfo_t *pi, const char *model, const char *headmodel, char *teamName );
// void UI_PlayerInfo_SetInfo( playerInfo_t *pi, int legsAnim, int torsoAnim, vec3_t viewAngles, vec3_t moveAngles, weapon_t weaponNum, qboolean chat );
// qboolean UI_RegisterClientModelname( playerInfo_t *pi, const char *modelSkinName , const char *headName, const char *teamName);

//
// ui_atoms.c
//
// this is only used in the old ui, the new ui has it's own version
#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct uiStatic_t {
    pub frametime: c_int,
    pub realtime: c_int,
    pub cursorx: c_int,
    pub cursory: c_int,
    pub glconfig: glconfig_t,
    pub debug: qboolean,
    pub whiteShader: qhandle_t,
    pub menuBackShader: qhandle_t,
    pub menuBackShader2: qhandle_t,
    pub menuBackNoLogoShader: qhandle_t,
    pub charset: qhandle_t,
    pub cursor: qhandle_t,
    pub rb_on: qhandle_t,
    pub rb_off: qhandle_t,
    pub scale: f32,
    pub bias: f32,
    pub demoversion: qboolean,
    pub firstdraw: qboolean,
}

// new ui stuff
pub const UI_NUMFX: usize = 7;
pub const MAX_HEADS: usize = 64;
pub const MAX_ALIASES: usize = 64;
pub const MAX_HEADNAME: usize = 32;
pub const MAX_TEAMS: usize = 64;
pub const MAX_GAMETYPES: usize = 16;
pub const MAX_MAPS: usize = 128;
pub const MAX_SPMAPS: usize = 16;
pub const PLAYERS_PER_TEAM: usize = 8; // 5
pub const MAX_PINGREQUESTS: usize = 32;
pub const MAX_ADDRESSLENGTH: usize = 64;
pub const MAX_HOSTNAMELENGTH: usize = 22;
pub const MAX_MAPNAMELENGTH: usize = 16;
pub const MAX_STATUSLENGTH: usize = 64;
pub const MAX_LISTBOXWIDTH: usize = 59;
pub const UI_FONT_THRESHOLD: f32 = 0.1;
pub const MAX_DISPLAY_SERVERS: usize = 2048;
pub const MAX_SERVERSTATUS_LINES: usize = 128;
pub const MAX_SERVERSTATUS_TEXT: usize = 1024;
pub const MAX_FOUNDPLAYER_SERVERS: usize = 16;
pub const TEAM_MEMBERS: usize = 8; // 5
pub const GAMES_ALL: i32 = 0;
pub const GAMES_FFA: i32 = 1;
pub const GAMES_HOLOCRON: i32 = 2;
pub const GAMES_TEAMPLAY: i32 = 3;
pub const GAMES_TOURNEY: i32 = 4;
pub const GAMES_CTF: i32 = 5;
pub const MAPS_PER_TIER: usize = 3;
pub const MAX_TIERS: usize = 16;
pub const MAX_MODS: usize = 64;
pub const MAX_DEMOS: usize = 256;
pub const MAX_MOVIES: usize = 256;
pub const MAX_Q3PLAYERMODELS: usize = 256;
pub const MAX_PLAYERMODELS: usize = 32;

pub const MAX_SCROLLTEXT_SIZE: usize = 4096;
pub const MAX_SCROLLTEXT_LINES: usize = 64;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct characterInfo {
    pub name: *const c_char,
    pub imageName: *const c_char,
    pub headImage: qhandle_t,
    pub base: *const c_char,
    pub active: qboolean,
    pub reference: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct aliasInfo {
    pub name: *const c_char,
    pub ai: *const c_char,
    pub action: *const c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct teamInfo {
    pub teamName: *const c_char,
    pub imageName: *const c_char,
    pub teamMembers: [*const c_char; TEAM_MEMBERS],
    pub teamIcon: qhandle_t,
    pub teamIcon_Metal: qhandle_t,
    pub teamIcon_Name: qhandle_t,
    pub cinematic: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct gameTypeInfo {
    pub gameType: *const c_char,
    pub gtEnum: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct mapInfo {
    pub mapName: *const c_char,
    pub mapLoadName: *const c_char,
    pub imageName: *const c_char,
    pub opponentName: *const c_char,
    pub teamMembers: c_int,
    pub typeBits: c_int,
    pub cinematic: c_int,
    pub timeToBeat: [c_int; MAX_GAMETYPES],
    pub levelShot: qhandle_t,
    pub active: qboolean,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct tierInfo {
    pub tierName: *const c_char,
    pub maps: [*const c_char; MAPS_PER_TIER],
    pub gameTypes: [c_int; MAPS_PER_TIER],
    pub mapHandles: [qhandle_t; MAPS_PER_TIER],
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct serverFilter_s {
    pub description: *const c_char,
    pub basedir: *const c_char,
}

pub type serverFilter_t = serverFilter_s;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct pinglist_t {
    pub adrstr: [c_char; MAX_ADDRESSLENGTH],
    pub start: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct serverStatus_s {
    pub pingList: [pinglist_t; MAX_PINGREQUESTS],
    pub numqueriedservers: c_int,
    pub currentping: c_int,
    pub nextpingtime: c_int,
    pub maxservers: c_int,
    pub refreshtime: c_int,
    pub numServers: c_int,
    pub sortKey: c_int,
    pub sortDir: c_int,
    pub lastCount: c_int,
    pub refreshActive: qboolean,
    pub currentServer: c_int,
    pub displayServers: [c_int; MAX_DISPLAY_SERVERS],
    pub numDisplayServers: c_int,
    pub numPlayersOnServers: c_int,
    pub nextDisplayRefresh: c_int,
    pub nextSortTime: c_int,
    pub currentServerPreview: qhandle_t,
    pub currentServerCinematic: c_int,
    pub motdLen: c_int,
    pub motdWidth: c_int,
    pub motdPaintX: c_int,
    pub motdPaintX2: c_int,
    pub motdOffset: c_int,
    pub motdTime: c_int,
    pub motd: [c_char; MAX_STRING_CHARS],
}

pub type serverStatus_t = serverStatus_s;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct pendingServer_t {
    pub adrstr: [c_char; MAX_ADDRESSLENGTH],
    pub name: [c_char; MAX_ADDRESSLENGTH],
    pub startTime: c_int,
    pub serverNum: c_int,
    pub valid: qboolean,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct pendingServerStatus_t {
    pub num: c_int,
    pub server: [pendingServer_t; MAX_SERVERSTATUSREQUESTS],
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct serverStatusInfo_t {
    pub address: [c_char; MAX_ADDRESSLENGTH],
    pub lines: [[*mut c_char; 4]; MAX_SERVERSTATUS_LINES],
    pub text: [c_char; MAX_SERVERSTATUS_TEXT],
    pub pings: [c_char; MAX_CLIENTS * 3],
    pub numLines: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct modInfo_t {
    pub modName: *const c_char,
    pub modDescr: *const c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct playerSpeciesInfo_t {
    pub Name: [c_char; 64],
    pub SkinHeadCount: c_int,
    pub SkinHeadNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub SkinTorsoCount: c_int,
    pub SkinTorsoNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub SkinLegCount: c_int,
    pub SkinLegNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub ColorShader: [[c_char; 64]; MAX_PLAYERMODELS],
    pub ColorCount: c_int,
    pub ColorActionText: [[c_char; 128]; MAX_PLAYERMODELS],
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct uiInfo_t {
    pub uiDC: displayContextDef_t,
    pub newHighScoreTime: c_int,
    pub newBestTime: c_int,
    pub showPostGameTime: c_int,
    pub newHighScore: qboolean,
    pub demoAvailable: qboolean,
    pub soundHighScore: qboolean,

    pub characterCount: c_int,
    pub botIndex: c_int,
    // characterInfo characterList[MAX_HEADS];

    pub aliasCount: c_int,
    pub aliasList: [aliasInfo; MAX_ALIASES],

    pub teamCount: c_int,
    pub teamList: [teamInfo; MAX_TEAMS],

    pub numGameTypes: c_int,
    pub gameTypes: [gameTypeInfo; MAX_GAMETYPES],

    pub numJoinGameTypes: c_int,
    pub joinGameTypes: [gameTypeInfo; MAX_GAMETYPES],

    pub redBlue: c_int,
    pub playerCount: c_int,
    pub myTeamCount: c_int,
    pub teamIndex: c_int,
    pub playerRefresh: c_int,
    pub playerIndex: c_int,
    pub playerNumber: c_int,
    pub teamLeader: qboolean,
    pub playerNames: [[c_char; MAX_NAME_LENGTH]; MAX_CLIENTS],
    pub teamNames: [[c_char; MAX_NAME_LENGTH]; MAX_CLIENTS],
    pub teamClientNums: [c_int; MAX_CLIENTS],

    pub playerIndexes: [c_int; MAX_CLIENTS], // so we can vote-kick by index

    pub mapCount: c_int,
    pub mapList: [mapInfo; MAX_MAPS],

    pub tierCount: c_int,
    pub tierList: [tierInfo; MAX_TIERS],

    pub skillIndex: c_int,

    pub modList: [modInfo_t; MAX_MODS],
    pub modCount: c_int,
    pub modIndex: c_int,

    pub demoList: [*const c_char; MAX_DEMOS],
    pub demoCount: c_int,
    pub demoIndex: c_int,

    pub movieList: [*const c_char; MAX_MOVIES],
    pub movieCount: c_int,
    pub movieIndex: c_int,
    pub previewMovie: c_int,

    pub scrolltext: [c_char; MAX_SCROLLTEXT_SIZE],
    pub scrolltextLine: [*const c_char; MAX_SCROLLTEXT_LINES],
    pub scrolltextLineCount: c_int,

    pub serverStatus: serverStatus_t,

    // for the showing the status of a server
    pub serverStatusAddress: [c_char; MAX_ADDRESSLENGTH],
    pub serverStatusInfo: serverStatusInfo_t,
    pub nextServerStatusRefresh: c_int,

    // to retrieve the status of server to find a player
    pub pendingServerStatus: pendingServerStatus_t,
    pub findPlayerName: [c_char; MAX_STRING_CHARS],
    pub foundPlayerServerAddresses: [[c_char; MAX_ADDRESSLENGTH]; MAX_FOUNDPLAYER_SERVERS],
    pub foundPlayerServerNames: [[c_char; MAX_ADDRESSLENGTH]; MAX_FOUNDPLAYER_SERVERS],
    pub currentFoundPlayerServer: c_int,
    pub numFoundPlayerServers: c_int,
    pub nextFindPlayerRefresh: c_int,

    pub currentCrosshair: c_int,
    pub startPostGameTime: c_int,
    pub newHighScoreSound: sfxHandle_t,

    pub q3HeadCount: c_int,
    pub q3HeadNames: [[c_char; 64]; MAX_Q3PLAYERMODELS],
    pub q3HeadIcons: [qhandle_t; MAX_Q3PLAYERMODELS],
    pub q3SelectedHead: c_int,

    pub forceConfigCount: c_int,
    pub forceConfigSelected: c_int,
    pub forceConfigNames: [[c_char; 128]; MAX_FORCE_CONFIGS],
    pub forceConfigSide: [qboolean; MAX_FORCE_CONFIGS], // true if it's a light side config, false if dark side
    pub forceConfigDarkIndexBegin: c_int, // mark the index number dark configs start at
    pub forceConfigLightIndexBegin: c_int, // mark the index number light configs start at

    pub effectsColor: c_int,

    pub inGameLoad: qboolean,

    pub playerSpeciesCount: c_int,
    pub playerSpecies: [playerSpeciesInfo_t; MAX_PLAYERMODELS],
    pub playerSpeciesIndex: c_int,

    pub movesTitleIndex: i16,
    pub movesBaseAnim: *mut c_char,
    pub moveAnimTime: c_int,

    pub languageCount: c_int,
    pub languageCountIndex: c_int,
}

extern "C" {
    pub static mut uiInfo: uiInfo_t;
}

extern "C" {
    pub fn UI_Init();
    pub fn UI_Shutdown();
    pub fn UI_KeyEvent(key: c_int);
    pub fn UI_MouseEvent(dx: c_int, dy: c_int);
    pub fn UI_Refresh(realtime: c_int);
    pub fn UI_ConsoleCommand(realTime: c_int) -> qboolean;
    pub fn UI_ClampCvar(min: f32, max: f32, value: f32) -> f32;
    pub fn UI_DrawNamedPic(x: f32, y: f32, width: f32, height: f32, picname: *const c_char);
    pub fn UI_DrawHandlePic(x: f32, y: f32, w: f32, h: f32, hShader: qhandle_t);
    pub fn UI_FillRect(x: f32, y: f32, width: f32, height: f32, color: *const f32);
    pub fn UI_DrawRect(x: f32, y: f32, width: f32, height: f32, color: *const f32);
    pub fn UI_DrawTopBottom(x: f32, y: f32, w: f32, h: f32);
    pub fn UI_DrawSides(x: f32, y: f32, w: f32, h: f32);
    pub fn UI_UpdateScreen();
    pub fn UI_SetColor(rgba: *const f32);
    pub fn UI_LerpColor(a: vec4_t, b: vec4_t, c: vec4_t, t: f32);
    pub fn UI_DrawBannerString(x: c_int, y: c_int, str: *const c_char, style: c_int, color: vec4_t);
    pub fn UI_ProportionalSizeScale(style: c_int) -> f32;
    pub fn UI_DrawProportionalString(x: c_int, y: c_int, str: *const c_char, style: c_int, color: vec4_t);
    pub fn UI_ProportionalStringWidth(str: *const c_char) -> c_int;
    pub fn UI_DrawString(x: c_int, y: c_int, str: *const c_char, style: c_int, color: vec4_t);
    pub fn UI_DrawChar(x: c_int, y: c_int, ch: c_int, style: c_int, color: vec4_t);
    pub fn UI_CursorInRect(x: c_int, y: c_int, width: c_int, height: c_int) -> qboolean;
    pub fn UI_DrawTextBox(x: c_int, y: c_int, width: c_int, lines: c_int);
    pub fn UI_IsFullscreen() -> qboolean;
    pub fn UI_SetActiveMenu(menu: uiMenuCommand_t);
    pub fn UI_PushMenu(menu: *mut menuframework_s);
    pub fn UI_PopMenu();
    pub fn UI_ForceMenuOff();
    pub fn UI_Argv(arg: c_int) -> *mut c_char;
    pub fn UI_Cvar_VariableString(var_name: *const c_char) -> *mut c_char;
    pub fn UI_StartDemoLoop();
    pub static mut m_entersound: qboolean;
    pub fn UI_LoadBestScores(map: *const c_char, game: c_int);
    pub static mut uis: uiStatic_t;
}

//
// ui_spLevel.c
//
extern "C" {
    pub fn UI_SPLevelMenu_Cache();
    pub fn UI_SPLevelMenu();
    pub fn UI_SPLevelMenu_f();
    pub fn UI_SPLevelMenu_ReInit();
}

//
// ui_spArena.c
//
extern "C" {
    pub fn UI_SPArena_Start(arenaInfo: *const c_char);
}

//
// ui_spPostgame.c
//
extern "C" {
    pub fn UI_SPPostgameMenu_Cache();
    pub fn UI_SPPostgameMenu_f();
}

//
// ui_spSkill.c
//
extern "C" {
    pub fn UI_SPSkillMenu(arenaInfo: *const c_char);
    pub fn UI_SPSkillMenu_Cache();
}

//
// ui_syscalls.c
//

// Namespace handling: The C code includes namespace_begin.h and namespace_end.h
// These are C namespace macros for preventing symbol collisions. In Rust,
// we use module organization instead, so these are preserved here as comments
// for documentation of the original intent.
// #include "../namespace_begin.h"

extern "C" {
    pub fn trap_Print(string: *const c_char);
    pub fn trap_Error(string: *const c_char);
    pub fn trap_Milliseconds() -> c_int;
    pub fn trap_Cvar_Register(vmCvar: *mut vmCvar_t, varName: *const c_char, defaultValue: *const c_char, flags: c_int);
    pub fn trap_Cvar_Update(vmCvar: *mut vmCvar_t);
    pub fn trap_Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn trap_Cvar_VariableValue(var_name: *const c_char) -> f32;
    pub fn trap_Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
    pub fn trap_Cvar_SetValue(var_name: *const c_char, value: f32);
    pub fn trap_Cvar_Reset(name: *const c_char);
    pub fn trap_Cvar_Create(var_name: *const c_char, var_value: *const c_char, flags: c_int);
    pub fn trap_Cvar_InfoStringBuffer(bit: c_int, buffer: *mut c_char, bufsize: c_int);
    pub fn trap_Argc() -> c_int;
    pub fn trap_Argv(n: c_int, buffer: *mut c_char, bufferLength: c_int);
    pub fn trap_Cmd_ExecuteText(exec_when: c_int, text: *const c_char); // don't use EXEC_NOW!
    pub fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t) -> c_int;
    pub fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t);
    pub fn trap_FS_Write(buffer: *const c_void, len: c_int, f: fileHandle_t);
    pub fn trap_FS_FCloseFile(f: fileHandle_t);
    pub fn trap_FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int;
    pub fn trap_R_RegisterModel(name: *const c_char) -> qhandle_t;
    pub fn trap_R_RegisterSkin(name: *const c_char) -> qhandle_t;
    pub fn trap_R_RegisterShaderNoMip(name: *const c_char) -> qhandle_t;
    pub fn trap_R_ShaderNameFromIndex(name: *mut c_char, index: c_int);
    pub fn trap_R_ClearScene();
    pub fn trap_R_AddRefEntityToScene(re: *const refEntity_t);
    pub fn trap_R_AddPolyToScene(hShader: qhandle_t, numVerts: c_int, verts: *const polyVert_t);
    pub fn trap_R_AddLightToScene(org: *const vec3_t, intensity: f32, r: f32, g: f32, b: f32);
    pub fn trap_R_RenderScene(fd: *const refdef_t);
    pub fn trap_R_SetColor(rgba: *const f32);
    pub fn trap_R_DrawStretchPic(x: f32, y: f32, w: f32, h: f32, s1: f32, t1: f32, s2: f32, t2: f32, hShader: qhandle_t);
    pub fn trap_R_ModelBounds(model: clipHandle_t, mins: *mut vec3_t, maxs: *mut vec3_t);
    pub fn trap_UpdateScreen();
    pub fn trap_CM_LerpTag(tag: *mut orientation_t, mod_: clipHandle_t, startFrame: c_int, endFrame: c_int, frac: f32, tagName: *const c_char) -> c_int;
    pub fn trap_S_StartLocalSound(sfx: sfxHandle_t, channelNum: c_int);
    pub fn trap_S_RegisterSound(sample: *const c_char) -> sfxHandle_t;
    pub fn trap_Key_KeynumToStringBuf(keynum: c_int, buf: *mut c_char, buflen: c_int);
    pub fn trap_Key_GetBindingBuf(keynum: c_int, buf: *mut c_char, buflen: c_int);
    pub fn trap_Key_SetBinding(keynum: c_int, binding: *const c_char);
    pub fn trap_Key_IsDown(keynum: c_int) -> qboolean;
    pub fn trap_Key_GetOverstrikeMode() -> qboolean;
    pub fn trap_Key_SetOverstrikeMode(state: qboolean);
    pub fn trap_Key_ClearStates();
    pub fn trap_Key_GetCatcher() -> c_int;
    pub fn trap_Key_SetCatcher(catcher: c_int);
    pub fn trap_GetClipboardData(buf: *mut c_char, bufsize: c_int);
    pub fn trap_GetClientState(state: *mut uiClientState_t);
    pub fn trap_GetGlconfig(glconfig: *mut glconfig_t);
    pub fn trap_GetConfigString(index: c_int, buff: *mut c_char, buffsize: c_int) -> c_int;
    pub fn trap_LAN_GetServerCount(source: c_int) -> c_int;
    pub fn trap_LAN_GetServerAddressString(source: c_int, n: c_int, buf: *mut c_char, buflen: c_int);
    pub fn trap_LAN_GetServerInfo(source: c_int, n: c_int, buf: *mut c_char, buflen: c_int);
    pub fn trap_LAN_GetServerPing(source: c_int, n: c_int) -> c_int;
    pub fn trap_LAN_GetPingQueueCount() -> c_int;
    pub fn trap_LAN_ClearPing(n: c_int);
    pub fn trap_LAN_GetPing(n: c_int, buf: *mut c_char, buflen: c_int, pingtime: *mut c_int);
    pub fn trap_LAN_GetPingInfo(n: c_int, buf: *mut c_char, buflen: c_int);
    pub fn trap_LAN_LoadCachedServers();
    pub fn trap_LAN_SaveCachedServers();
    pub fn trap_LAN_MarkServerVisible(source: c_int, n: c_int, visible: qboolean);
    pub fn trap_LAN_ServerIsVisible(source: c_int, n: c_int) -> c_int;
    pub fn trap_LAN_UpdateVisiblePings(source: c_int) -> qboolean;
    pub fn trap_LAN_AddServer(source: c_int, name: *const c_char, addr: *const c_char) -> c_int;
    pub fn trap_LAN_RemoveServer(source: c_int, addr: *const c_char);
    pub fn trap_LAN_ResetPings(n: c_int);
    pub fn trap_LAN_ServerStatus(serverAddress: *const c_char, serverStatus: *mut c_char, maxLen: c_int) -> c_int;
    pub fn trap_LAN_CompareServers(source: c_int, sortKey: c_int, sortDir: c_int, s1: c_int, s2: c_int) -> c_int;
    pub fn trap_MemoryRemaining() -> c_int;

    #[cfg(feature = "use_cd_key")]
    pub fn trap_GetCDKey(buf: *mut c_char, buflen: c_int);
    #[cfg(feature = "use_cd_key")]
    pub fn trap_SetCDKey(buf: *mut c_char);
    #[cfg(feature = "use_cd_key")]
    pub fn trap_VerifyCDKey(key: *const c_char, chksum: *const c_char) -> qboolean;

    pub fn trap_R_RegisterFont(name: *const c_char) -> qhandle_t;
    pub fn trap_R_Font_StrLenPixels(text: *const c_char, iFontIndex: c_int, scale: f32) -> c_int;
    pub fn trap_R_Font_StrLenChars(text: *const c_char) -> c_int;
    pub fn trap_R_Font_HeightPixels(iFontIndex: c_int, scale: f32) -> c_int;
    pub fn trap_R_Font_DrawString(ox: c_int, oy: c_int, text: *const c_char, rgba: *const f32, setIndex: c_int, iCharLimit: c_int, scale: f32);
    pub fn trap_Language_IsAsian() -> qboolean;
    pub fn trap_Language_UsesSpaces() -> qboolean;
    pub fn trap_AnyLanguage_ReadCharFromString(psText: *const c_char, piAdvanceCount: *mut c_int, pbIsTrailingPunctuation: *mut qboolean) -> c_uint;
    pub fn trap_S_StopBackgroundTrack();
    pub fn trap_S_StartBackgroundTrack(intro: *const c_char, loop_: *const c_char, bReturnWithoutStarting: qboolean);
    pub fn trap_CIN_PlayCinematic(arg0: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int, bits: c_int) -> c_int;
    pub fn trap_CIN_StopCinematic(handle: c_int) -> e_status;
    pub fn trap_CIN_RunCinematic(handle: c_int) -> e_status;
    pub fn trap_CIN_DrawCinematic(handle: c_int);
    pub fn trap_CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int);
    pub fn trap_RealTime(qtime: *mut qtime_t) -> c_int;
    pub fn trap_R_RemapShader(oldShader: *const c_char, newShader: *const c_char, timeOffset: *const c_char);
}

// #include "../namespace_end.h"

//
// ui_addbots.c
//
extern "C" {
    pub fn UI_AddBots_Cache();
    pub fn UI_AddBotsMenu();
}

//
// ui_removebots.c
//
extern "C" {
    pub fn UI_RemoveBots_Cache();
    pub fn UI_RemoveBotsMenu();
}

//
// ui_teamorders.c
//
extern "C" {
    pub fn UI_TeamOrdersMenu();
    pub fn UI_TeamOrdersMenu_f();
    pub fn UI_TeamOrdersMenu_Cache();
}

//
// ui_loadconfig.c
//
extern "C" {
    pub fn UI_LoadConfig_Cache();
    pub fn UI_LoadConfigMenu();
}

//
// ui_saveconfig.c
//
extern "C" {
    pub fn UI_SaveConfigMenu_Cache();
    pub fn UI_SaveConfigMenu();
}

//
// ui_display.c
//
extern "C" {
    pub fn UI_DisplayOptionsMenu_Cache();
    pub fn UI_DisplayOptionsMenu();
}

//
// ui_sound.c
//
extern "C" {
    pub fn UI_SoundOptionsMenu_Cache();
    pub fn UI_SoundOptionsMenu();
}

//
// ui_network.c
//
extern "C" {
    pub fn UI_NetworkOptionsMenu_Cache();
    pub fn UI_NetworkOptionsMenu();
}

//
// ui_gameinfo.c
//
#[repr(u32)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub enum awardType_t {
    AWARD_ACCURACY = 0,
    AWARD_IMPRESSIVE = 1,
    AWARD_EXCELLENT = 2,
    AWARD_GAUNTLET = 3,
    AWARD_FRAGS = 4,
    AWARD_PERFECT = 5,
}

extern "C" {
    pub fn UI_GetArenaInfoByNumber(num: c_int) -> *const c_char;
    pub fn UI_GetArenaInfoByMap(map: *const c_char) -> *const c_char;
    pub fn UI_GetSpecialArenaInfo(tag: *const c_char) -> *const c_char;
    pub fn UI_GetNumArenas() -> c_int;
    pub fn UI_GetNumSPArenas() -> c_int;
    pub fn UI_GetNumSPTiers() -> c_int;

    pub fn UI_GetBotInfoByNumber(num: c_int) -> *mut c_char;
    pub fn UI_GetBotInfoByName(name: *const c_char) -> *mut c_char;
    pub fn UI_GetNumBots() -> c_int;
    pub fn UI_LoadBots();
    pub fn UI_GetBotNameByNumber(num: c_int) -> *mut c_char;

    pub fn UI_GetBestScore(level: c_int, score: *mut c_int, skill: *mut c_int);
    pub fn UI_SetBestScore(level: c_int, score: c_int);
    pub fn UI_TierCompleted(levelWon: c_int) -> c_int;
    pub fn UI_ShowTierVideo(tier: c_int) -> qboolean;
    pub fn UI_CanShowTierVideo(tier: c_int) -> qboolean;
    pub fn UI_GetCurrentGame() -> c_int;
    pub fn UI_NewGame();
    pub fn UI_LogAwardData(award: c_int, data: c_int);
    pub fn UI_GetAwardLevel(award: c_int) -> c_int;

    pub fn UI_SPUnlock_f();
    pub fn UI_SPUnlockMedals_f();

    pub fn UI_InitGameinfo();
}

//
// ui_login.c
//
extern "C" {
    pub fn Login_Cache();
    pub fn UI_LoginMenu();
}

//
// ui_signup.c
//
extern "C" {
    pub fn Signup_Cache();
    pub fn UI_SignupMenu();
}

//
// ui_rankstatus.c
//
extern "C" {
    pub fn RankStatus_Cache();
    pub fn UI_RankStatusMenu();
}

// new ui

pub const ASSET_BACKGROUND: &[u8] = b"uiBackground";

// for tracking sp game info in Team Arena
#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct postGameInfo_s {
    pub score: c_int,
    pub redScore: c_int,
    pub blueScore: c_int,
    pub perfects: c_int,
    pub accuracy: c_int,
    pub impressives: c_int,
    pub excellents: c_int,
    pub defends: c_int,
    pub assists: c_int,
    pub gauntlets: c_int,
    pub captures: c_int,
    pub time: c_int,
    pub timeBonus: c_int,
    pub shutoutBonus: c_int,
    pub skillBonus: c_int,
    pub baseScore: c_int,
}

pub type postGameInfo_t = postGameInfo_s;
