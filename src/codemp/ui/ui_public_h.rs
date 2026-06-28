// Copyright (C) 1999-2000 Id Software, Inc.
//
//! Mechanical port of `codemp/ui/ui_public.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{connstate_t, MAX_STRING_CHARS};
use core::ffi::{c_char, c_int};

pub const UI_API_VERSION: c_int = 7;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct uiClientState_t {
    pub connState: connstate_t,
    pub connectPacketCount: c_int,
    pub clientNum: c_int,
    pub servername: [c_char; MAX_STRING_CHARS],
    pub updateInfoString: [c_char; MAX_STRING_CHARS],
    pub messageString: [c_char; MAX_STRING_CHARS],
}

pub type uiImport_t = c_int;

pub const UI_ERROR: uiImport_t = 0;
pub const UI_PRINT: uiImport_t = 1;
pub const UI_MILLISECONDS: uiImport_t = 2;
pub const UI_CVAR_SET: uiImport_t = 3;
pub const UI_CVAR_VARIABLEVALUE: uiImport_t = 4;
pub const UI_CVAR_VARIABLESTRINGBUFFER: uiImport_t = 5;
pub const UI_CVAR_SETVALUE: uiImport_t = 6;
pub const UI_CVAR_RESET: uiImport_t = 7;
pub const UI_CVAR_CREATE: uiImport_t = 8;
pub const UI_CVAR_INFOSTRINGBUFFER: uiImport_t = 9;
pub const UI_ARGC: uiImport_t = 10;
pub const UI_ARGV: uiImport_t = 11;
pub const UI_CMD_EXECUTETEXT: uiImport_t = 12;
pub const UI_FS_FOPENFILE: uiImport_t = 13;
pub const UI_FS_READ: uiImport_t = 14;
pub const UI_FS_WRITE: uiImport_t = 15;
pub const UI_FS_FCLOSEFILE: uiImport_t = 16;
pub const UI_FS_GETFILELIST: uiImport_t = 17;
pub const UI_R_REGISTERMODEL: uiImport_t = 18;
pub const UI_R_REGISTERSKIN: uiImport_t = 19;
pub const UI_R_REGISTERSHADERNOMIP: uiImport_t = 20;
pub const UI_R_SHADERNAMEFROMINDEX: uiImport_t = 21;
pub const UI_R_CLEARSCENE: uiImport_t = 22;
pub const UI_R_ADDREFENTITYTOSCENE: uiImport_t = 23;
pub const UI_R_ADDPOLYTOSCENE: uiImport_t = 24;
pub const UI_R_ADDLIGHTTOSCENE: uiImport_t = 25;
pub const UI_R_RENDERSCENE: uiImport_t = 26;
pub const UI_R_SETCOLOR: uiImport_t = 27;
pub const UI_R_DRAWSTRETCHPIC: uiImport_t = 28;
pub const UI_UPDATESCREEN: uiImport_t = 29;
pub const UI_CM_LERPTAG: uiImport_t = 30;
pub const UI_CM_LOADMODEL: uiImport_t = 31;
pub const UI_S_REGISTERSOUND: uiImport_t = 32;
pub const UI_S_STARTLOCALSOUND: uiImport_t = 33;
pub const UI_KEY_KEYNUMTOSTRINGBUF: uiImport_t = 34;
pub const UI_KEY_GETBINDINGBUF: uiImport_t = 35;
pub const UI_KEY_SETBINDING: uiImport_t = 36;
pub const UI_KEY_ISDOWN: uiImport_t = 37;
pub const UI_KEY_GETOVERSTRIKEMODE: uiImport_t = 38;
pub const UI_KEY_SETOVERSTRIKEMODE: uiImport_t = 39;
pub const UI_KEY_CLEARSTATES: uiImport_t = 40;
pub const UI_KEY_GETCATCHER: uiImport_t = 41;
pub const UI_KEY_SETCATCHER: uiImport_t = 42;
pub const UI_GETCLIPBOARDDATA: uiImport_t = 43;
pub const UI_GETGLCONFIG: uiImport_t = 44;
pub const UI_GETCLIENTSTATE: uiImport_t = 45;
pub const UI_GETCONFIGSTRING: uiImport_t = 46;
pub const UI_LAN_GETPINGQUEUECOUNT: uiImport_t = 47;
pub const UI_LAN_CLEARPING: uiImport_t = 48;
pub const UI_LAN_GETPING: uiImport_t = 49;
pub const UI_LAN_GETPINGINFO: uiImport_t = 50;
pub const UI_CVAR_REGISTER: uiImport_t = 51;
pub const UI_CVAR_UPDATE: uiImport_t = 52;
pub const UI_MEMORY_REMAINING: uiImport_t = 53;
pub const UI_GET_CDKEY: uiImport_t = 54;
pub const UI_SET_CDKEY: uiImport_t = 55;
pub const UI_VERIFY_CDKEY: uiImport_t = 56;
pub const UI_R_REGISTERFONT: uiImport_t = 57;
pub const UI_R_FONT_STRLENPIXELS: uiImport_t = 58;
pub const UI_R_FONT_STRLENCHARS: uiImport_t = 59;
pub const UI_R_FONT_STRHEIGHTPIXELS: uiImport_t = 60;
pub const UI_R_FONT_DRAWSTRING: uiImport_t = 61;
pub const UI_LANGUAGE_ISASIAN: uiImport_t = 62;
pub const UI_LANGUAGE_USESSPACES: uiImport_t = 63;
pub const UI_ANYLANGUAGE_READCHARFROMSTRING: uiImport_t = 64;
pub const UI_R_MODELBOUNDS: uiImport_t = 65;
pub const UI_PC_ADD_GLOBAL_DEFINE: uiImport_t = 66;
pub const UI_PC_LOAD_SOURCE: uiImport_t = 67;
pub const UI_PC_FREE_SOURCE: uiImport_t = 68;
pub const UI_PC_READ_TOKEN: uiImport_t = 69;
pub const UI_PC_SOURCE_FILE_AND_LINE: uiImport_t = 70;
pub const UI_PC_LOAD_GLOBAL_DEFINES: uiImport_t = 71;
pub const UI_PC_REMOVE_ALL_GLOBAL_DEFINES: uiImport_t = 72;

pub const UI_S_STOPBACKGROUNDTRACK: uiImport_t = 73;
pub const UI_S_STARTBACKGROUNDTRACK: uiImport_t = 74;
pub const UI_REAL_TIME: uiImport_t = 75;
pub const UI_LAN_GETSERVERCOUNT: uiImport_t = 76;
pub const UI_LAN_GETSERVERADDRESSSTRING: uiImport_t = 77;
pub const UI_LAN_GETSERVERINFO: uiImport_t = 78;
pub const UI_LAN_MARKSERVERVISIBLE: uiImport_t = 79;
pub const UI_LAN_UPDATEVISIBLEPINGS: uiImport_t = 80;
pub const UI_LAN_RESETPINGS: uiImport_t = 81;
pub const UI_LAN_LOADCACHEDSERVERS: uiImport_t = 82;
pub const UI_LAN_SAVECACHEDSERVERS: uiImport_t = 83;
pub const UI_LAN_ADDSERVER: uiImport_t = 84;
pub const UI_LAN_REMOVESERVER: uiImport_t = 85;
pub const UI_CIN_PLAYCINEMATIC: uiImport_t = 86;
pub const UI_CIN_STOPCINEMATIC: uiImport_t = 87;
pub const UI_CIN_RUNCINEMATIC: uiImport_t = 88;
pub const UI_CIN_DRAWCINEMATIC: uiImport_t = 89;
pub const UI_CIN_SETEXTENTS: uiImport_t = 90;
pub const UI_R_REMAP_SHADER: uiImport_t = 91;
pub const UI_LAN_SERVERSTATUS: uiImport_t = 92;
pub const UI_LAN_GETSERVERPING: uiImport_t = 93;
pub const UI_LAN_SERVERISVISIBLE: uiImport_t = 94;
pub const UI_LAN_COMPARESERVERS: uiImport_t = 95;

pub const UI_MEMSET: uiImport_t = 100;
pub const UI_MEMCPY: uiImport_t = 101;
pub const UI_STRNCPY: uiImport_t = 102;
pub const UI_SIN: uiImport_t = 103;
pub const UI_COS: uiImport_t = 104;
pub const UI_ATAN2: uiImport_t = 105;
pub const UI_SQRT: uiImport_t = 106;
pub const UI_MATRIXMULTIPLY: uiImport_t = 107;
pub const UI_ANGLEVECTORS: uiImport_t = 108;
pub const UI_PERPENDICULARVECTOR: uiImport_t = 109;
pub const UI_FLOOR: uiImport_t = 110;
pub const UI_CEIL: uiImport_t = 111;

pub const UI_TESTPRINTINT: uiImport_t = 112;
pub const UI_TESTPRINTFLOAT: uiImport_t = 113;

pub const UI_ACOS: uiImport_t = 114;
pub const UI_ASIN: uiImport_t = 115;

pub const UI_SP_GETNUMLANGUAGES: uiImport_t = 116;
pub const UI_SP_GETLANGUAGENAME: uiImport_t = 117;
pub const UI_SP_GETSTRINGTEXTSTRING: uiImport_t = 200;

/*
Ghoul2 Insert Start
*/
pub const UI_G2_LISTSURFACES: uiImport_t = 201;
pub const UI_G2_LISTBONES: uiImport_t = 202;
pub const UI_G2_SETMODELS: uiImport_t = 203;
pub const UI_G2_HAVEWEGHOULMODELS: uiImport_t = 204;
pub const UI_G2_GETBOLT: uiImport_t = 205;
pub const UI_G2_GETBOLT_NOREC: uiImport_t = 206;
pub const UI_G2_GETBOLT_NOREC_NOROT: uiImport_t = 207;
pub const UI_G2_INITGHOUL2MODEL: uiImport_t = 208;
pub const UI_G2_COLLISIONDETECT: uiImport_t = 209;
pub const UI_G2_COLLISIONDETECTCACHE: uiImport_t = 210;
pub const UI_G2_CLEANMODELS: uiImport_t = 211;
pub const UI_G2_ANGLEOVERRIDE: uiImport_t = 212;
pub const UI_G2_PLAYANIM: uiImport_t = 213;
pub const UI_G2_GETBONEANIM: uiImport_t = 214;
// trimmed down version of GBA, so I don't have to pass all those unused args across the VM-exe border
pub const UI_G2_GETBONEFRAME: uiImport_t = 215;
pub const UI_G2_GETGLANAME: uiImport_t = 216;
pub const UI_G2_COPYGHOUL2INSTANCE: uiImport_t = 217;
pub const UI_G2_COPYSPECIFICGHOUL2MODEL: uiImport_t = 218;
pub const UI_G2_DUPLICATEGHOUL2INSTANCE: uiImport_t = 219;
pub const UI_G2_HASGHOUL2MODELONINDEX: uiImport_t = 220;
pub const UI_G2_REMOVEGHOUL2MODEL: uiImport_t = 221;
pub const UI_G2_ADDBOLT: uiImport_t = 222;
pub const UI_G2_SETBOLTON: uiImport_t = 223;
pub const UI_G2_SETROOTSURFACE: uiImport_t = 224;
pub const UI_G2_SETSURFACEONOFF: uiImport_t = 225;
pub const UI_G2_SETNEWORIGIN: uiImport_t = 226;

pub const UI_G2_GETTIME: uiImport_t = 227;
pub const UI_G2_SETTIME: uiImport_t = 228;

/*
    //rww - RAGDOLL_BEGIN
*/
pub const UI_G2_SETRAGDOLL: uiImport_t = 229;
pub const UI_G2_ANIMATEG2MODELS: uiImport_t = 230;
/*
    //rww - RAGDOLL_END
*/

// rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
// by using the majority of gil's existing bone angling stuff from the ragdoll code.
pub const UI_G2_SETBONEIKSTATE: uiImport_t = 231;
pub const UI_G2_IKMOVE: uiImport_t = 232;

pub const UI_G2_GETSURFACENAME: uiImport_t = 233;
pub const UI_G2_SETSKIN: uiImport_t = 234;
pub const UI_G2_ATTACHG2MODEL: uiImport_t = 235;
/*
Ghoul2 Insert End
*/

pub type uiMenuCommand_t = c_int;

pub const UIMENU_NONE: uiMenuCommand_t = 0;
pub const UIMENU_MAIN: uiMenuCommand_t = 1;
pub const UIMENU_INGAME: uiMenuCommand_t = 2;
pub const UIMENU_PLAYERCONFIG: uiMenuCommand_t = 3;
pub const UIMENU_TEAM: uiMenuCommand_t = 4;
pub const UIMENU_POSTGAME: uiMenuCommand_t = 5;
pub const UIMENU_PLAYERFORCE: uiMenuCommand_t = 6;
pub const UIMENU_SIEGEMESSAGE: uiMenuCommand_t = 7;
pub const UIMENU_SIEGEOBJECTIVES: uiMenuCommand_t = 8;
pub const UIMENU_VOICECHAT: uiMenuCommand_t = 9;
pub const UIMENU_CLOSEALL: uiMenuCommand_t = 10;
pub const UIMENU_CLASSSEL: uiMenuCommand_t = 11;

pub const SORT_HOST: c_int = 0;
pub const SORT_MAP: c_int = 1;
pub const SORT_CLIENTS: c_int = 2;
pub const SORT_GAME: c_int = 3;
pub const SORT_PING: c_int = 4;

pub type uiExport_t = c_int;

pub const UI_GETAPIVERSION: uiExport_t = 0; // system reserved

pub const UI_INIT: uiExport_t = 1;
// void UI_Init( void );

pub const UI_SHUTDOWN: uiExport_t = 2;
// void UI_Shutdown( void );

pub const UI_KEY_EVENT: uiExport_t = 3;
// void UI_KeyEvent( int key );

pub const UI_MOUSE_EVENT: uiExport_t = 4;
// void UI_MouseEvent( int dx, int dy );

pub const UI_REFRESH: uiExport_t = 5;
// void UI_Refresh( int time );

pub const UI_IS_FULLSCREEN: uiExport_t = 6;
// qboolean UI_IsFullscreen( void );

pub const UI_SET_ACTIVE_MENU: uiExport_t = 7;
// void UI_SetActiveMenu( uiMenuCommand_t menu );

pub const UI_CONSOLE_COMMAND: uiExport_t = 8;
// qboolean UI_ConsoleCommand( int realTime );

pub const UI_DRAW_CONNECT_SCREEN: uiExport_t = 9;
// void UI_DrawConnectScreen( qboolean overlay );
pub const UI_HASUNIQUECDKEY: uiExport_t = 10;
// if !overlay, the background will be drawn, otherwise it will be
// overlayed over whatever the cgame has drawn.
// a GetClientState syscall will be made to get the current strings

pub const UI_MENU_RESET: uiExport_t = 11;
