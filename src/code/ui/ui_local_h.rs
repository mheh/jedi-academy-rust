// ui_local.h - UI system local definitions
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// === Type definitions from included headers ===
// Type stubs for structural coherence; full definitions in respective modules:
pub type qboolean = c_int;
pub type vec3_t = [f32; 3];
pub type vec4_t = [f32; 4];
pub type qhandle_t = c_int;
pub type sfxHandle_t = c_int;
pub type clipHandle_t = c_int;
pub type ffHandle_t = c_int;

// Opaque struct declarations for types from other modules
#[repr(C)]
pub struct glconfig_t {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct displayContextDef_t {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct itemDef_t {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct refEntity_t {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct refdef_t {
    _unused: [u8; 0],
}
#[repr(C)]
pub struct uiimport_t {
    _unused: [u8; 0],
}

// Constants from included headers (should be imported from respective modules in full integration)
pub const NUM_FORCE_POWERS: usize = 7;

// === ui_qmenu.c ===

pub const MAX_PLAYERMODELS: usize = 32;
pub const MAX_DEFERRED_SCRIPT: usize = 1024;

pub const MAX_EDIT_LINE: usize = 256;

#[repr(C)]
pub struct uifield_t {
    pub cursor: c_int,
    pub scroll: c_int,
    pub widthInChars: c_int,
    pub buffer: [c_char; MAX_EDIT_LINE],
    pub maxchars: c_int,
    pub style: c_int,
    pub textEnum: c_int,        // Label
    pub textcolor: c_int,       // Normal color
    pub textcolor2: c_int,      // Highlight color
}

extern "C" {
    pub fn Menu_Cache();
}

// === ui_field.c ===

extern "C" {
    pub fn Field_Clear(edit: *mut uifield_t);
    pub fn Field_CharEvent(edit: *mut uifield_t, ch: c_int);
    pub fn Field_Draw(
        edit: *mut uifield_t,
        x: c_int,
        y: c_int,
        width: c_int,
        size: c_int,
        color: c_int,
        color2: c_int,
        showCursor: qboolean,
    );
}

// === ui_menu.c ===

extern "C" {
    pub fn UI_MainMenu();
    pub fn UI_InGameMenu(holoFlag: *const c_char);
    pub fn AssetCache();
    pub fn UI_DataPadMenu();
}

// === ui_connect.c ===

extern "C" {
    pub fn UI_DrawConnect(servername: *const c_char, updateInfoString: *const c_char);
    pub fn UI_UpdateConnectionString(string: *mut c_char);
    pub fn UI_UpdateConnectionMessageString(string: *mut c_char);
}

// === ui_atoms.c ===

pub const UI_FADEOUT: c_int = 0;
pub const UI_FADEIN: c_int = 1;

#[repr(C)]
pub struct uiStatic_t {
    pub frametime: c_int,
    pub realtime: c_int,
    pub cursorx: c_int,
    pub cursory: c_int,

    pub glconfig: glconfig_t,
    pub debugMode: qboolean,
    pub whiteShader: qhandle_t,
    pub menuBackShader: qhandle_t,
    pub cursor: qhandle_t,
    pub scalex: f32,
    pub scaley: f32,
    //pub bias: f32,
    pub firstdraw: qboolean,
}

extern "C" {
    pub fn UI_FillRect(x: f32, y: f32, width: f32, height: f32, color: *const f32);
    pub fn UI_DrawString(x: c_int, y: c_int, str: *const c_char, style: c_int, color: vec4_t);
    pub fn UI_DrawHandlePic(x: f32, y: f32, w: f32, h: f32, hShader: qhandle_t);
    pub fn UI_UpdateScreen();
    pub fn UI_RegisterFont(fontName: *const c_char) -> c_int;
    pub fn UI_SetColor(rgba: *const f32);
    pub fn UI_Cvar_VariableString(var_name: *const c_char) -> *mut c_char;
}

extern "C" {
    pub static mut uis: uiStatic_t;
    pub static mut ui: uiimport_t;
}

pub const MAX_MOVIES: usize = 256;
pub const MAX_MODS: usize = 64;

#[repr(C)]
pub struct modInfo_t {
    pub modName: *const c_char,
    pub modDescr: *const c_char,
}

#[repr(C)]
pub struct playerSpeciesInfo_t {
    pub Name: [c_char; 64],
    pub SkinHeadCount: c_int,
    //pub SkinHeadIcons: [qhandle_t; MAX_PLAYERMODELS],
    pub SkinHeadNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub SkinTorsoCount: c_int,
    //pub SkinTorsoIcons: [qhandle_t; MAX_PLAYERMODELS],
    pub SkinTorsoNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub SkinLegCount: c_int,
    //pub SkinLegIcons: [qhandle_t; MAX_PLAYERMODELS],
    pub SkinLegNames: [[c_char; 16]; MAX_PLAYERMODELS],
    pub ColorShader: [[c_char; 64]; MAX_PLAYERMODELS],
    pub ColorCount: c_int,
    pub ColorActionText: [[c_char; 128]; MAX_PLAYERMODELS],
}

#[repr(C)]
pub struct uiInfo_t {
    pub uiDC: displayContextDef_t,

    pub effectsColor: c_int,
    pub currentCrosshair: c_int,

    pub modList: [modInfo_t; MAX_MODS],
    pub modIndex: c_int,
    pub modCount: c_int,

    pub playerSpeciesCount: c_int,
    pub playerSpecies: [playerSpeciesInfo_t; MAX_PLAYERMODELS],
    pub playerSpeciesIndex: c_int,

    pub deferredScript: [c_char; MAX_DEFERRED_SCRIPT],
    pub deferredScriptItem: *mut itemDef_t,

    pub runScriptItem: *mut itemDef_t,

    pub inGameLoad: qboolean,
    // Used by Force Power allocation screen
    pub forcePowerUpdated: i16,                     // Enum of which power had the point allocated
    // Used by Weapon allocation screen
    pub selectedWeapon1: i16,                       // 1st weapon chosen
    pub selectedWeapon1ItemName: [c_char; 64],      // Item name of weapon chosen
    pub selectedWeapon1AmmoIndex: c_int,            // Holds index to ammo
    pub selectedWeapon2: i16,                       // 2nd weapon chosen
    pub selectedWeapon2ItemName: [c_char; 64],      // Item name of weapon chosen
    pub selectedWeapon2AmmoIndex: c_int,            // Holds index to ammo
    pub selectedThrowWeapon: i16,                   // throwable weapon chosen
    pub selectedThrowWeaponItemName: [c_char; 64],  // Item name of weapon chosen
    pub selectedThrowWeaponAmmoIndex: c_int,        // Holds index to ammo

    pub weapon1ItemButton: *mut itemDef_t,
    pub litWeapon1Icon: qhandle_t,
    pub unlitWeapon1Icon: qhandle_t,
    pub weapon2ItemButton: *mut itemDef_t,
    pub litWeapon2Icon: qhandle_t,
    pub unlitWeapon2Icon: qhandle_t,

    pub weaponThrowButton: *mut itemDef_t,
    pub litThrowableIcon: qhandle_t,
    pub unlitThrowableIcon: qhandle_t,
    pub movesTitleIndex: i16,
    pub movesBaseAnim: *mut c_char,
    pub moveAnimTime: c_int,
    pub languageCount: c_int,
    pub languageCountIndex: c_int,

    pub forcePowerLevel: [c_int; NUM_FORCE_POWERS],
}

extern "C" {
    pub static mut uiInfo: uiInfo_t;
}

// === ui_main.c ===

extern "C" {
    pub fn _UI_Init(inGameLoad: qboolean);
    pub fn _UI_DrawRect(x: f32, y: f32, width: f32, height: f32, size: f32, color: *const f32);
    pub fn _UI_MouseEvent(dx: c_int, dy: c_int);
    pub fn _UI_KeyEvent(key: c_int, down: qboolean);
    pub fn UI_Report();
}

extern "C" {
    pub static mut GoToMenu: [c_char; 0];  // Incomplete array type (unsized)
}

// === ui_syscalls.c ===

extern "C" {
    pub fn trap_CIN_PlayCinematic(
        arg0: *const c_char,
        xpos: c_int,
        ypos: c_int,
        width: c_int,
        height: c_int,
        bits: c_int,
        psAudioFile: *const c_char, /* = NULL */
    ) -> c_int;
    pub fn trap_CIN_StopCinematic(handle: c_int) -> c_int;
    pub fn trap_Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn trap_Cvar_VariableValue(var_name: *const c_char) -> f32;
    pub fn trap_GetGlconfig(glconfig: *mut glconfig_t);
    pub fn trap_Key_ClearStates();
    pub fn trap_Key_GetCatcher() -> c_int;
    pub fn trap_Key_GetOverstrikeMode() -> qboolean;
    pub fn trap_Key_SetBinding(keynum: c_int, binding: *const c_char);
    pub fn trap_Key_SetCatcher(catcher: c_int);
    pub fn trap_Key_SetOverstrikeMode(state: qboolean);
    pub fn trap_R_DrawStretchPic(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        hShader: qhandle_t,
    );
    pub fn trap_R_ModelBounds(model: clipHandle_t, mins: *mut vec3_t, maxs: *mut vec3_t);
    pub fn trap_R_SetColor(rgba: *const f32);
    pub fn trap_R_ClearScene();
    pub fn trap_R_AddRefEntityToScene(re: *const refEntity_t);
    pub fn trap_R_RenderScene(fd: *const refdef_t);
    pub fn trap_S_StopSounds();
    pub fn trap_S_RegisterSound(sample: *const c_char, compressed: qboolean) -> sfxHandle_t;
    pub fn trap_S_StartLocalSound(sfx: sfxHandle_t, channelNum: c_int);

    #[cfg(feature = "_IMMERSION")]
    pub fn trap_FF_Register(name: *const c_char, channel: c_int) -> ffHandle_t;
    #[cfg(feature = "_IMMERSION")]
    pub fn trap_FF_Start(ff: ffHandle_t);

    #[cfg(not(target_os = "windows"))]
    pub fn PASSFLOAT(x: f32) -> c_int;
}

extern "C" {
    pub fn _UI_Refresh(realtime: c_int);
}
