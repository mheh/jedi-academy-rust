// Copyright (C) 1999-2000 Id Software, Inc.
//
//! Mechanical port of `code/ui/ui_public.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_void};

// Type definitions for C handle types and common types
pub type qhandle_t = c_int;
pub type fileHandle_t = c_int;
pub type fsMode_t = c_int;
pub type clipHandle_t = c_int;
pub type sfxHandle_t = c_int;
pub type qboolean = c_int;
pub type byte = u8;

// Type definition for vec3_t
pub type vec3_t = [c_float; 3];

// Stub declarations for external types used in function pointer signatures
// These are complex renderer/sound/system types that are defined in other modules
#[repr(C)]
pub struct refEntity_t {
    // PORTING: Stub - full definition not needed for ui_public.h interface
}

#[repr(C)]
pub struct polyVert_t {
    // PORTING: Stub - full definition not needed for ui_public.h interface
}

#[repr(C)]
pub struct refdef_t {
    // PORTING: Stub - full definition not needed for ui_public.h interface
}

#[repr(C)]
pub struct orientation_t {
    // PORTING: Stub - full definition not needed for ui_public.h interface
}

#[repr(C)]
pub struct glconfig_t {
    // PORTING: Stub - full definition not needed for ui_public.h interface
}

// connstate_t is imported as a type alias (c_int)
pub type connstate_t = c_int;

pub const UI_API_VERSION: c_int = 3;

// ============= uiimport_t struct =============
// Contains function pointers for engine callbacks available to the UI module

#[repr(C)]
pub struct uiimport_t {
    // ============== general Quake services ==================

    // print message on the local console
    pub Printf: unsafe extern "C" fn(*const c_char, ...) -> (),

    // abort the game
    pub Error: unsafe extern "C" fn(c_int, *const c_char, ...) -> (),

    // console variable interaction
    pub Cvar_Set: unsafe extern "C" fn(*const c_char, *const c_char) -> (),
    pub Cvar_VariableValue: unsafe extern "C" fn(*const c_char) -> c_float,
    pub Cvar_VariableStringBuffer:
        unsafe extern "C" fn(*const c_char, *mut c_char, c_int) -> (),
    pub Cvar_SetValue: unsafe extern "C" fn(*const c_char, c_float) -> (),
    pub Cvar_Reset: unsafe extern "C" fn(*const c_char) -> (),
    pub Cvar_Create: unsafe extern "C" fn(*const c_char, *const c_char, c_int) -> (),
    pub Cvar_InfoStringBuffer: unsafe extern "C" fn(c_int, *mut c_char, c_int) -> (),

    // console command interaction
    pub Argc: unsafe extern "C" fn() -> c_int,
    pub Argv: unsafe extern "C" fn(c_int, *mut c_char, c_int) -> (),
    pub Cmd_ExecuteText: unsafe extern "C" fn(c_int, *const c_char) -> (),
    pub Cmd_TokenizeString: unsafe extern "C" fn(*const c_char) -> (),

    // filesystem access
    pub FS_FOpenFile:
        unsafe extern "C" fn(*const c_char, *mut fileHandle_t, fsMode_t) -> c_int,
    pub FS_Read: unsafe extern "C" fn(*mut c_void, c_int, fileHandle_t) -> c_int,
    pub FS_Write: unsafe extern "C" fn(*const c_void, c_int, fileHandle_t) -> c_int,
    pub FS_FCloseFile: unsafe extern "C" fn(fileHandle_t) -> (),
    pub FS_GetFileList:
        unsafe extern "C" fn(*const c_char, *const c_char, *mut c_char, c_int) -> c_int,
    pub FS_ReadFile: unsafe extern "C" fn(*const c_char, *mut *mut c_void) -> c_int,
    pub FS_FreeFile: unsafe extern "C" fn(*mut c_void) -> (),

    // =========== renderer function calls ================

    pub R_RegisterModel: unsafe extern "C" fn(*const c_char) -> qhandle_t, // returns rgb axis if not found
    pub R_RegisterSkin: unsafe extern "C" fn(*const c_char) -> qhandle_t, // returns all white if not found
    pub R_RegisterShader: unsafe extern "C" fn(*const c_char) -> qhandle_t, // returns white if not found
    pub R_RegisterShaderNoMip: unsafe extern "C" fn(*const c_char) -> qhandle_t, // returns white if not found
    pub R_RegisterFont: unsafe extern "C" fn(*const c_char) -> qhandle_t, // returns 0 for bad font
    pub R_Font_StrLenPixels:
        unsafe extern "C" fn(*const c_char, c_int, c_float) -> c_int,
    pub R_Font_HeightPixels: unsafe extern "C" fn(c_int, c_float) -> c_int,
    pub R_Font_DrawString: unsafe extern "C" fn(
        c_int,
        c_int,
        *const c_char,
        *const c_float,
        c_int,
        c_int,
        c_float,
    ) -> (),
    pub R_Font_StrLenChars: unsafe extern "C" fn(*const c_char) -> c_int,
    pub Language_IsAsian: unsafe extern "C" fn() -> qboolean,
    pub Language_UsesSpaces: unsafe extern "C" fn() -> qboolean,
    pub AnyLanguage_ReadCharFromString: unsafe extern "C" fn(
        *const c_char,
        *mut c_int,
        *mut qboolean,
    ) -> c_int,

    // a scene is built up by calls to R_ClearScene and the various R_Add functions.
    // Nothing is drawn until R_RenderScene is called.
    pub R_ClearScene: unsafe extern "C" fn() -> (),
    pub R_AddRefEntityToScene: unsafe extern "C" fn(*const refEntity_t) -> (),
    pub R_AddPolyToScene:
        unsafe extern "C" fn(qhandle_t, c_int, *const polyVert_t) -> (),
    pub R_AddLightToScene: unsafe extern "C" fn(*const vec3_t, c_float, c_float, c_float, c_float) -> (),
    pub R_RenderScene: unsafe extern "C" fn(*const refdef_t) -> (),

    pub R_ModelBounds: unsafe extern "C" fn(qhandle_t, *mut vec3_t, *mut vec3_t) -> (),

    pub R_SetColor: unsafe extern "C" fn(*const c_float) -> (),  // NULL = 1,1,1,1
    pub R_DrawStretchPic: unsafe extern "C" fn(
        c_float,
        c_float,
        c_float,
        c_float,
        c_float,
        c_float,
        c_float,
        c_float,
        qhandle_t,
    ) -> (),  // 0 = white
    pub R_ScissorPic: unsafe extern "C" fn(
        c_float,
        c_float,
        c_float,
        c_float,
        c_float,
        c_float,
        c_float,
        c_float,
        qhandle_t,
    ) -> (),  // 0 = white

    // force a screen update, only used during gamestate load
    pub UpdateScreen: unsafe extern "C" fn() -> (),

    // stuff for savegame screenshots...
    pub PrecacheScreenshot: unsafe extern "C" fn() -> (),

    // ========= model collision ===============

    // R_LerpTag is only valid for md3 models
    pub R_LerpTag: unsafe extern "C" fn(
        *mut orientation_t,
        clipHandle_t,
        c_int,
        c_int,
        c_float,
        *const c_char,
    ) -> (),

    // =========== sound function calls ===============

    pub S_StartLocalSound: unsafe extern "C" fn(sfxHandle_t, c_int) -> (),
    pub S_RegisterSound: unsafe extern "C" fn(*const c_char) -> sfxHandle_t,
    pub S_StartLocalLoopingSound: unsafe extern "C" fn(sfxHandle_t) -> (),
    pub S_StopSounds: unsafe extern "C" fn() -> (),

    // =========== getting save game picture ===============
    pub DrawStretchRaw: unsafe extern "C" fn(
        c_int,
        c_int,
        c_int,
        c_int,
        c_int,
        c_int,
        *const byte,
        c_int,
        qboolean,
    ) -> (),
    //qboolean(*SG_GetSaveImage)( const char *psPathlessBaseName, void *pvAddress );
    pub SG_GetSaveGameComment:
        unsafe extern "C" fn(*const c_char, *mut c_char, *mut c_char) -> c_int,
    pub SG_GameAllowedToSaveHere: unsafe extern "C" fn(qboolean) -> qboolean,
    pub SG_StoreSaveGameComment: unsafe extern "C" fn(*const c_char) -> (),
    //byte *(*SCR_GetScreenshot)(qboolean *);

    // =========== data shared with the client system =============

    // keyboard and key binding interaction
    pub Key_KeynumToStringBuf: unsafe extern "C" fn(c_int, *mut c_char, c_int) -> (),
    pub Key_GetBindingBuf: unsafe extern "C" fn(c_int, *mut c_char, c_int) -> (),
    pub Key_SetBinding: unsafe extern "C" fn(c_int, *const c_char) -> (),
    pub Key_IsDown: unsafe extern "C" fn(c_int) -> qboolean,
    pub Key_GetOverstrikeMode: unsafe extern "C" fn() -> qboolean,
    pub Key_SetOverstrikeMode: unsafe extern "C" fn(qboolean) -> (),
    pub Key_ClearStates: unsafe extern "C" fn() -> (),
    pub Key_GetCatcher: unsafe extern "C" fn() -> c_int,
    pub Key_SetCatcher: unsafe extern "C" fn(c_int) -> (),

    pub GetClipboardData: unsafe extern "C" fn(*mut c_char, c_int) -> (),

    pub GetGlconfig: unsafe extern "C" fn(*mut glconfig_t) -> (),

    pub GetClientState: unsafe extern "C" fn() -> connstate_t,

    pub GetConfigString: unsafe extern "C" fn(c_int, *mut c_char, c_int) -> (),

    pub Milliseconds: unsafe extern "C" fn() -> c_int,
    pub Draw_DataPad: unsafe extern "C" fn(c_int) -> (),
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum dpTypes_t {
    DP_HUD = 0,
    DP_OBJECTIVES,
    DP_WEAPONS,
    DP_INVENTORY,
    DP_FORCEPOWERS,
}

// uiImport_t enum - syscall IDs for trap function indices
#[repr(C)]
#[derive(Copy, Clone)]
pub enum uiImport_t {
    UI_ERROR = 0,
    UI_PRINT,
    UI_MILLISECONDS,
    UI_CVAR_SET,
    UI_CVAR_VARIABLEVALUE,
    UI_CVAR_VARIABLESTRINGBUFFER,
    UI_CVAR_SETVALUE,
    UI_CVAR_RESET,
    UI_CVAR_CREATE,
    UI_CVAR_INFOSTRINGBUFFER,
    UI_ARGC = 10,                            // 10
    UI_ARGV,
    UI_CMD_EXECUTETEXT,
    UI_FS_FOPENFILE,
    UI_FS_READ,
    UI_FS_WRITE,
    UI_FS_FCLOSEFILE,
    UI_FS_GETFILELIST,
    UI_R_REGISTERMODEL,
    UI_R_REGISTERSKIN,
    UI_R_REGISTERSHADERNOMIP = 20,            // 20
    UI_R_CLEARSCENE,
    UI_R_ADDREFENTITYTOSCENE,
    UI_R_ADDPOLYTOSCENE,
    UI_R_ADDLIGHTTOSCENE,
    UI_R_RENDERSCENE,
    UI_R_SETCOLOR,
    UI_R_DRAWSTRETCHPIC,
    UI_UPDATESCREEN,
    UI_CM_LERPTAG,
    UI_CM_LOADMODEL = 30,                    // 30
    UI_S_REGISTERSOUND,
    UI_S_STARTLOCALSOUND,
    UI_KEY_KEYNUMTOSTRINGBUF,
    UI_KEY_GETBINDINGBUF,
    UI_KEY_SETBINDING,
    UI_KEY_ISDOWN,
    UI_KEY_GETOVERSTRIKEMODE,
    UI_KEY_SETOVERSTRIKEMODE,
    UI_KEY_CLEARSTATES,
    UI_KEY_GETCATCHER = 40,                  // 40
    UI_KEY_SETCATCHER,
    UI_GETCLIPBOARDDATA,
    UI_GETGLCONFIG,
    UI_GETCLIENTSTATE,
    UI_GETCONFIGSTRING,
    UI_LAN_GETPINGQUEUECOUNT,
    UI_LAN_CLEARPING,
    UI_LAN_GETPING,
    UI_LAN_GETPINGINFO,
    UI_CVAR_REGISTER = 50,                   // 50
    UI_CVAR_UPDATE,
    UI_MEMORY_REMAINING,
    UI_GET_CDKEY,
    UI_SET_CDKEY,
    UI_R_REGISTERFONT,
    UI_R_MODELBOUNDS,
    UI_PC_ADD_GLOBAL_DEFINE,
    UI_PC_LOAD_SOURCE,
    UI_PC_FREE_SOURCE,
    UI_PC_READ_TOKEN = 60,                   // 60
    UI_PC_SOURCE_FILE_AND_LINE,
    UI_S_STOPBACKGROUNDTRACK,
    UI_S_STARTBACKGROUNDTRACK,
    UI_REAL_TIME,
    UI_LAN_GETSERVERCOUNT,
    UI_LAN_GETSERVERADDRESSSTRING,
    UI_LAN_GETSERVERINFO,
    UI_LAN_MARKSERVERVISIBLE,
    UI_LAN_UPDATEVISIBLEPINGS,
    UI_LAN_RESETPINGS = 70,                  // 70
    UI_LAN_LOADCACHEDSERVERS,
    UI_LAN_SAVECACHEDSERVERS,
    UI_LAN_ADDSERVER,
    UI_LAN_REMOVESERVER,
    UI_CIN_PLAYCINEMATIC,
    UI_CIN_STOPCINEMATIC,
    UI_CIN_RUNCINEMATIC,
    UI_CIN_DRAWCINEMATIC,
    UI_CIN_SETEXTENTS,
    UI_R_REMAP_SHADER = 80,                  // 80
    UI_VERIFY_CDKEY,
    UI_LAN_SERVERSTATUS,
    UI_LAN_GETSERVERPING,
    UI_LAN_SERVERISVISIBLE,
    UI_LAN_COMPARESERVERS,

    UI_MEMSET = 100,
    UI_MEMCPY,
    UI_STRNCPY,
    UI_SIN,
    UI_COS,
    UI_ATAN2,
    UI_SQRT,
    UI_FLOOR,
    UI_CEIL,
}
