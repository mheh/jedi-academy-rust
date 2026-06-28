// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "client.h"
// #include "../game/botlib.h"
// #include "../qcommon/stringed_ingame.h"

/*
Ghoul2 Insert Start
*/

// #if !defined(G2_H_INC)
// #include "../ghoul2/G2_local.h"
// #endif

/*
Ghoul2 Insert End
*/

// #ifdef VV_LIGHTING
// #include "../renderer/tr_lightmanager.h"
// #endif

use core::ffi::{c_char, c_int, c_void};

// Stub types and extern declarations for unresolved dependencies
extern "C" {
    pub static mut botlib_export: *mut botlib_export_t;
    pub fn SP_Register(Package: *const c_char);
}

// Stub for botlib_export_t
#[repr(C)]
pub struct botlib_export_t {
    // placeholder for actual struct
}

// Stub for vm_t
pub enum vm_t {}

// Stub for uiClientState_t
#[repr(C)]
pub struct uiClientState_t {
    // placeholder
}

// Stub for serverInfo_t
#[repr(C)]
pub struct serverInfo_t {
    // placeholder
}

// Stub for netadr_t
#[repr(C)]
pub struct netadr_t {
    // placeholder
}

// Stub for glconfig_t
#[repr(C)]
pub struct glconfig_t {
    // placeholder
}

// Stub for cvar_t
#[repr(C)]
pub struct cvar_t {
    // placeholder
}

// Stub for vmCvar_t
#[repr(C)]
pub struct vmCvar_t {
    // placeholder
}

// Stub for pc_token_s
#[repr(C)]
pub struct pc_token_s {
    // placeholder
}

// Stub for qtime_s
#[repr(C)]
pub struct qtime_s {
    // placeholder
}

// Stub for refEntity_t
#[repr(C)]
pub struct refEntity_t {
    // placeholder
}

// Stub for polyVert_t
#[repr(C)]
pub struct polyVert_t {
    // placeholder
}

// Stub for refdef_t
#[repr(C)]
pub struct refdef_t {
    // placeholder
}

// Stub for orientation_t
#[repr(C)]
pub struct orientation_t {
    // placeholder
}

// Stub for mdxaBone_t
#[repr(C)]
pub struct mdxaBone_t {
    // placeholder
}

// Stub for CGhoul2Info
pub enum CGhoul2Info {}

// Stub for CGhoul2Info_v
pub enum CGhoul2Info_v {}

// Stub for SSkinGoreData
#[repr(C)]
pub struct SSkinGoreData {
    // placeholder
}

// Stub for sharedSetBoneIKStateParams_t
#[repr(C)]
pub struct sharedSetBoneIKStateParams_t {
    // placeholder
}

// Stub for sharedIKMoveParams_t
#[repr(C)]
pub struct sharedIKMoveParams_t {
    // placeholder
}

// Stub for Eorientations
pub enum Eorientations {}

// Stub for fileHandle_t
pub type fileHandle_t = c_int;

// Stub for fsMode_t
pub type fsMode_t = c_int;

// Stub for qhandle_t
pub type qhandle_t = c_int;

// Stub for qboolean
pub type qboolean = c_int;

pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

// Stub for NA_BAD and other address types
pub const NA_BAD: c_int = 0;

// Stub for server list constants
pub const MAX_OTHER_SERVERS: c_int = 128;
pub const MAX_GLOBAL_SERVERS: c_int = 2048;
pub const MAX_STRING_CHARS: c_int = 1024;

// Stub for address source constants
pub const AS_LOCAL: c_int = 0;
pub const AS_MPLAYER: c_int = 1;
pub const AS_GLOBAL: c_int = 2;
pub const AS_FAVORITES: c_int = 3;

// Stub for sort key constants
pub const SORT_HOST: c_int = 0;
pub const SORT_MAP: c_int = 1;
pub const SORT_CLIENTS: c_int = 2;
pub const SORT_GAME: c_int = 3;
pub const SORT_PING: c_int = 4;

// Stub for trap IDs
pub const TRAP_MEMSET: c_int = 100;
pub const TRAP_MEMCPY: c_int = 101;
pub const TRAP_STRNCPY: c_int = 102;
pub const TRAP_SIN: c_int = 103;
pub const TRAP_COS: c_int = 104;
pub const TRAP_ATAN2: c_int = 105;
pub const TRAP_SQRT: c_int = 106;
pub const TRAP_MATRIXMULTIPLY: c_int = 107;
pub const TRAP_ANGLEVECTORS: c_int = 108;
pub const TRAP_PERPENDICULARVECTOR: c_int = 109;
pub const TRAP_FLOOR: c_int = 110;
pub const TRAP_CEIL: c_int = 111;
pub const TRAP_TESTPRINTINT: c_int = 112;
pub const TRAP_TESTPRINTFLOAT: c_int = 113;
pub const TRAP_ACOS: c_int = 114;
pub const TRAP_ASIN: c_int = 115;

pub const UI_ERROR: c_int = 1;
pub const UI_PRINT: c_int = 2;
pub const UI_MILLISECONDS: c_int = 3;
pub const UI_CVAR_REGISTER: c_int = 4;
pub const UI_CVAR_UPDATE: c_int = 5;
pub const UI_CVAR_SET: c_int = 6;
pub const UI_CVAR_VARIABLEVALUE: c_int = 7;
pub const UI_CVAR_VARIABLESTRINGBUFFER: c_int = 8;
pub const UI_CVAR_SETVALUE: c_int = 9;
pub const UI_CVAR_RESET: c_int = 10;
pub const UI_CVAR_CREATE: c_int = 11;
pub const UI_CVAR_INFOSTRINGBUFFER: c_int = 12;
pub const UI_ARGC: c_int = 13;
pub const UI_ARGV: c_int = 14;
pub const UI_CMD_EXECUTETEXT: c_int = 15;
pub const UI_FS_FOPENFILE: c_int = 16;
pub const UI_FS_READ: c_int = 17;
pub const UI_FS_WRITE: c_int = 18;
pub const UI_FS_FCLOSEFILE: c_int = 19;
pub const UI_FS_GETFILELIST: c_int = 20;
pub const UI_R_REGISTERMODEL: c_int = 21;
pub const UI_R_REGISTERSKIN: c_int = 22;
pub const UI_R_REGISTERSHADERNOMIP: c_int = 23;
pub const UI_R_SHADERNAMEFROMINDEX: c_int = 24;
pub const UI_R_CLEARSCENE: c_int = 25;
pub const UI_R_ADDREFENTITYTOSCENE: c_int = 26;
pub const UI_R_ADDPOLYTOSCENE: c_int = 27;
pub const UI_R_ADDLIGHTTOSCENE: c_int = 28;
pub const UI_R_RENDERSCENE: c_int = 29;
pub const UI_R_SETCOLOR: c_int = 30;
pub const UI_R_DRAWSTRETCHPIC: c_int = 31;
pub const UI_R_MODELBOUNDS: c_int = 32;
pub const UI_UPDATESCREEN: c_int = 33;
pub const UI_CM_LERPTAG: c_int = 34;
pub const UI_S_REGISTERSOUND: c_int = 35;
pub const UI_S_STARTLOCALSOUND: c_int = 36;
pub const UI_KEY_KEYNUMTOSTRINGBUF: c_int = 37;
pub const UI_KEY_GETBINDINGBUF: c_int = 38;
pub const UI_KEY_SETBINDING: c_int = 39;
pub const UI_KEY_ISDOWN: c_int = 40;
pub const UI_KEY_GETOVERSTRIKEMODE: c_int = 41;
pub const UI_KEY_SETOVERSTRIKEMODE: c_int = 42;
pub const UI_KEY_CLEARSTATES: c_int = 43;
pub const UI_KEY_GETCATCHER: c_int = 44;
pub const UI_KEY_SETCATCHER: c_int = 45;
pub const UI_GETCLIPBOARDDATA: c_int = 46;
pub const UI_GETCLIENTSTATE: c_int = 47;
pub const UI_GETGLCONFIG: c_int = 48;
pub const UI_GETCONFIGSTRING: c_int = 49;
pub const UI_LAN_LOADCACHEDSERVERS: c_int = 50;
pub const UI_LAN_SAVECACHEDSERVERS: c_int = 51;
pub const UI_LAN_ADDSERVER: c_int = 52;
pub const UI_LAN_REMOVESERVER: c_int = 53;
pub const UI_LAN_GETPINGQUEUECOUNT: c_int = 54;
pub const UI_LAN_CLEARPING: c_int = 55;
pub const UI_LAN_GETPING: c_int = 56;
pub const UI_LAN_GETPINGINFO: c_int = 57;
pub const UI_LAN_GETSERVERCOUNT: c_int = 58;
pub const UI_LAN_GETSERVERADDRESSSTRING: c_int = 59;
pub const UI_LAN_GETSERVERINFO: c_int = 60;
pub const UI_LAN_GETSERVERPING: c_int = 61;
pub const UI_LAN_MARKSERVERVISIBLE: c_int = 62;
pub const UI_LAN_SERVERISVISIBLE: c_int = 63;
pub const UI_LAN_UPDATEVISIBLEPINGS: c_int = 64;
pub const UI_LAN_RESETPINGS: c_int = 65;
pub const UI_LAN_SERVERSTATUS: c_int = 66;
pub const UI_LAN_COMPARESERVERS: c_int = 67;
pub const UI_MEMORY_REMAINING: c_int = 68;
pub const UI_GET_CDKEY: c_int = 69;
pub const UI_SET_CDKEY: c_int = 70;
pub const UI_R_REGISTERFONT: c_int = 71;
pub const UI_R_FONT_STRLENPIXELS: c_int = 72;
pub const UI_R_FONT_STRLENCHARS: c_int = 73;
pub const UI_R_FONT_STRHEIGHTPIXELS: c_int = 74;
pub const UI_R_FONT_DRAWSTRING: c_int = 75;
pub const UI_LANGUAGE_ISASIAN: c_int = 76;
pub const UI_LANGUAGE_USESSPACES: c_int = 77;
pub const UI_ANYLANGUAGE_READCHARFROMSTRING: c_int = 78;
pub const UI_PC_ADD_GLOBAL_DEFINE: c_int = 79;
pub const UI_PC_LOAD_SOURCE: c_int = 80;
pub const UI_PC_FREE_SOURCE: c_int = 81;
pub const UI_PC_READ_TOKEN: c_int = 82;
pub const UI_PC_SOURCE_FILE_AND_LINE: c_int = 83;
pub const UI_PC_LOAD_GLOBAL_DEFINES: c_int = 84;
pub const UI_PC_REMOVE_ALL_GLOBAL_DEFINES: c_int = 85;
pub const UI_S_STOPBACKGROUNDTRACK: c_int = 86;
pub const UI_S_STARTBACKGROUNDTRACK: c_int = 87;
pub const UI_REAL_TIME: c_int = 88;
pub const UI_CIN_PLAYCINEMATIC: c_int = 89;
pub const UI_CIN_STOPCINEMATIC: c_int = 90;
pub const UI_CIN_RUNCINEMATIC: c_int = 91;
pub const UI_CIN_DRAWCINEMATIC: c_int = 92;
pub const UI_CIN_SETEXTENTS: c_int = 93;
pub const UI_R_REMAP_SHADER: c_int = 94;
pub const UI_VERIFY_CDKEY: c_int = 95;
pub const UI_SP_GETNUMLANGUAGES: c_int = 96;
pub const UI_SP_GETLANGUAGENAME: c_int = 97;
pub const UI_SP_GETSTRINGTEXTSTRING: c_int = 98;
pub const UI_G2_LISTSURFACES: c_int = 99;
pub const UI_G2_LISTBONES: c_int = 100;
pub const UI_G2_HAVEWEGHOULMODELS: c_int = 101;
pub const UI_G2_SETMODELS: c_int = 102;
pub const UI_G2_GETBOLT: c_int = 103;
pub const UI_G2_GETBOLT_NOREC: c_int = 104;
pub const UI_G2_GETBOLT_NOREC_NOROT: c_int = 105;
pub const UI_G2_INITGHOUL2MODEL: c_int = 106;
pub const UI_G2_COLLISIONDETECT: c_int = 107;
pub const UI_G2_COLLISIONDETECTCACHE: c_int = 108;
pub const UI_G2_ANGLEOVERRIDE: c_int = 109;
pub const UI_G2_CLEANMODELS: c_int = 110;
pub const UI_G2_PLAYANIM: c_int = 111;
pub const UI_G2_GETBONEANIM: c_int = 112;
pub const UI_G2_GETBONEFRAME: c_int = 113;
pub const UI_G2_GETGLANAME: c_int = 114;
pub const UI_G2_COPYGHOUL2INSTANCE: c_int = 115;
pub const UI_G2_COPYSPECIFICGHOUL2MODEL: c_int = 116;
pub const UI_G2_DUPLICATEGHOUL2INSTANCE: c_int = 117;
pub const UI_G2_HASGHOUL2MODELONINDEX: c_int = 118;
pub const UI_G2_REMOVEGHOUL2MODEL: c_int = 119;
pub const UI_G2_ADDBOLT: c_int = 120;
pub const UI_G2_SETBOLTON: c_int = 121;
pub const UI_G2_ADDSKINGORE: c_int = 122;
pub const UI_G2_SETROOTSURFACE: c_int = 123;
pub const UI_G2_SETSURFACEONOFF: c_int = 124;
pub const UI_G2_SETNEWORIGIN: c_int = 125;
pub const UI_G2_GETTIME: c_int = 126;
pub const UI_G2_SETTIME: c_int = 127;
pub const UI_G2_SETRAGDOLL: c_int = 128;
pub const UI_G2_ANIMATEG2MODELS: c_int = 129;
pub const UI_G2_SETBONEIKSTATE: c_int = 130;
pub const UI_G2_IKMOVE: c_int = 131;
pub const UI_G2_GETSURFACENAME: c_int = 132;
pub const UI_G2_SETSKIN: c_int = 133;
pub const UI_G2_ATTACHG2MODEL: c_int = 134;

pub const UI_SHUTDOWN: c_int = 1;
pub const UI_MENU_RESET: c_int = 2;
pub const UI_INIT: c_int = 3;
pub const UI_CONSOLE_COMMAND: c_int = 4;
pub const UI_HASUNIQUECDKEY: c_int = 5;
pub const UI_GETAPIVERSION: c_int = 6;
pub const UI_API_VERSION: c_int = 4;

pub const CA_AUTHORIZING: c_int = 4;
pub const CA_ACTIVE: c_int = 5;

pub const ERR_FATAL: c_int = 0;
pub const ERR_DROP: c_int = 1;

pub const CVAR_INIT: c_int = 1;
pub const CVAR_SYSTEMINFO: c_int = 16;
pub const CVAR_ARCHIVE: c_int = 32;

pub const KEYCATCH_UI: c_int = 2;

// Stub structures
#[repr(C)]
pub struct gameState_t {
    // placeholder
}

#[repr(C)]
pub struct playerState_t {
    pub clientNum: c_int,
    // placeholder for rest
}

#[repr(C)]
pub struct snapshot_t {
    pub ps: playerState_t,
    // placeholder for rest
}

#[repr(C)]
pub struct clientActive_t {
    pub snap: snapshot_t,
    pub gameState: gameState_t,
    // placeholder for rest
}

#[repr(C)]
pub struct clientStatic_t {
    // placeholder
}

// Extern "C" functions
extern "C" {
    pub static mut cl: clientActive_t;
    pub static mut cls: clientStatic_t;
    pub static mut clc: clientStatic_t;
    pub static mut cvar_modifiedFlags: c_int;

    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Com_Memset(buf: *mut c_void, c: c_int, size: c_int) -> *mut c_void;
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, size: c_int) -> *mut c_void;
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_RealTime(qtime: *mut qtime_s) -> c_int;

    pub fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Cvar_Register(vmCvar: *mut vmCvar_t, var_name: *const c_char, var_value: *const c_char, flags: c_int);
    pub fn Cvar_Update(vmCvar: *mut vmCvar_t);
    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn Cvar_SetValue(var_name: *const c_char, value: f32);
    pub fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    pub fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
    pub fn Cvar_Reset(var_name: *const c_char);
    pub fn Cvar_InfoStringBuffer(bit: c_int, buffer: *mut c_char, bufsize: c_int);

    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_ArgvBuffer(arg: c_int, buffer: *mut c_char, bufsize: c_int);

    pub fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);

    pub fn FS_FOpenFileByMode(qpath: *const c_char, f: *mut c_int, mode: fsMode_t) -> c_int;
    pub fn FS_Read2(buffer: *mut c_void, len: c_int, f: c_int) -> c_int;
    pub fn FS_Write(buffer: *const c_void, len: c_int, f: c_int) -> c_int;
    pub fn FS_FCloseFile(f: c_int);
    pub fn FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int;
    pub fn FS_SV_FOpenFileRead(filename: *const c_char, fp: *mut fileHandle_t) -> c_int;
    pub fn FS_SV_FOpenFileWrite(filename: *const c_char) -> fileHandle_t;
    pub fn FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int;
    pub fn FS_FCloseFile(f: fileHandle_t);

    pub fn Sys_Milliseconds() -> c_int;
    pub fn Sys_GetClipboardData() -> *mut c_char;

    pub fn Z_Free(ptr: *mut c_void);

    pub fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean;
    pub fn NET_CompareAdr(a: netadr_t, b: netadr_t) -> qboolean;
    pub fn NET_AdrToString(a: netadr_t) -> *const c_char;

    pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);

    pub fn va(fmt: *const c_char, ...) -> *const c_char;

    pub fn Key_KeynumToString(keynum: c_int) -> *const c_char;
    pub fn SE_GetString(label: *const c_char) -> *const c_char;
    pub fn Key_GetBinding(keynum: c_int) -> *const c_char;
    pub fn Key_SetBinding(keynum: c_int, binding: *const c_char);
    pub fn Key_IsDown(keynum: c_int) -> qboolean;
    pub fn Key_GetOverstrikeMode() -> qboolean;
    pub fn Key_SetOverstrikeMode(state: qboolean);
    pub fn Key_ClearStates();

    pub fn SCR_UpdateScreen();

    pub fn S_RegisterSound(name: *const c_char) -> qhandle_t;
    pub fn S_StartLocalSound(sfxHandle: qhandle_t, channelNum: c_int);
    pub fn S_StopBackgroundTrack();
    pub fn S_StartBackgroundTrack(intro: *const c_char, loop_: *const c_char, fadeupTime: qboolean);

    pub fn CL_GetPingQueueCount() -> c_int;
    pub fn CL_ClearPing(n: c_int);
    pub fn CL_GetPing(n: c_int, buf: *mut c_char, buflen: c_int, pingtime: *mut c_int);
    pub fn CL_GetPingInfo(n: c_int, buf: *mut c_char, buflen: c_int);
    pub fn CL_UpdateVisiblePings_f(source: c_int) -> qboolean;
    pub fn CL_ServerStatus(serverAddress: *mut c_char, serverStatus: *mut c_char, maxLen: c_int) -> c_int;
    pub fn CL_CDKeyValidate(key: *const c_char, checksum: *const c_char) -> c_int;

    pub fn Hunk_MemoryRemaining() -> c_int;

    pub fn SE_GetNumLanguages() -> c_int;
    pub fn SE_GetLanguageName(languageIndex: c_int) -> *const c_char;

    pub fn VM_Create(module: *const c_char, systemCalls: extern "C" fn(*mut c_int) -> c_int, interpret: vmInterpret_t) -> *mut vm_t;
    pub fn VM_Free(vm: *mut vm_t);
    pub fn VM_Call(vm: *mut vm_t, commandId: c_int, ...) -> c_int;
    pub fn VM_ArgPtr(intValue: c_int) -> *mut c_void;

    pub fn MatrixMultiply(in1: *const [f32; 3], in2: *const [f32; 3], out: *mut [f32; 3]);
    pub fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);
    pub fn PerpendicularVector(dst: *mut f32, src: *const f32);
    pub fn Q_acos(c: f32) -> f32;
    pub fn Q_asin(s: f32) -> f32;

    pub fn CIN_PlayCinematic(name: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: c_int) -> c_int;
    pub fn CIN_StopCinematic(handle: c_int) -> c_int;
    pub fn CIN_RunCinematic(handle: c_int) -> c_int;
    pub fn CIN_DrawCinematic(handle: c_int);
    pub fn CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int);

    // Renderer stubs
    pub struct renderer_t {
        // placeholder
    }
    pub static mut re: renderer_t;
}

pub type vmInterpret_t = c_int;

// Stub for renderer methods (would need to be implemented)
impl renderer_t {
    pub fn RegisterModel(&self, name: *const c_char) -> qhandle_t { 0 }
    pub fn RegisterSkin(&self, name: *const c_char) -> qhandle_t { 0 }
    pub fn RegisterShaderNoMip(&self, name: *const c_char) -> qhandle_t { 0 }
    pub fn ShaderNameFromIndex(&self, index: c_int) -> *const c_char { core::ptr::null() }
    pub fn ClearScene(&self) {}
    pub fn AddRefEntityToScene(&self, entity: *const refEntity_t) {}
    pub fn AddPolyToScene(&self, numverts: c_int, numpolys: c_int, polys: *const polyVert_t, numassets: c_int) {}
    pub fn AddLightToScene(&self, org: *const f32, intensity: f32, r: f32, g: f32, b: f32) {}
    pub fn RenderScene(&self, fd: *const refdef_t) {}
    pub fn SetColor(&self, rgba: *const f32) {}
    pub fn DrawStretchPic(&self, x: f32, y: f32, w: f32, h: f32, s1: f32, t1: f32, s2: f32, t2: f32, hShader: qhandle_t) {}
    pub fn ModelBounds(&self, model: qhandle_t, mins: *mut f32, maxs: *mut f32) {}
    pub fn LerpTag(&self, tag: *mut orientation_t, model: qhandle_t, start: c_int, end: c_int, frac: f32, tagName: *const c_char) {}
    pub fn RegisterFont(&self, fontName: *const c_char) -> qhandle_t { 0 }
    pub fn Font_StrLenPixels(&self, text: *const c_char, fontHandle: qhandle_t, scale: f32) -> c_int { 0 }
    pub fn Font_StrLenChars(&self, text: *const c_char) -> c_int { 0 }
    pub fn Font_HeightPixels(&self, fontHandle: qhandle_t, scale: f32) -> c_int { 0 }
    pub fn Font_DrawString(&self, ox: c_int, oy: c_int, text: *const c_char, color: *const f32, fontHandle: c_int, iCharLimit: c_int, fScale: f32) {}
    pub fn Language_IsAsian(&self) -> qboolean { 0 }
    pub fn Language_UsesSpaces(&self) -> qboolean { 0 }
    pub fn AnyLanguage_ReadCharFromString(&self, psText: *const c_char, piAdvanceCount: *mut c_int, pbIsTrailingPunctuation: *mut qboolean) -> c_int { 0 }
    pub fn RemapShader(&self, oldShader: *const c_char, newShader: *const c_char, timeOffset: *const c_char) {}
}

#[cfg(feature = "vv_lighting")]
extern "C" {
    pub struct VVLightManager {
        // placeholder
    }
    pub static VVLightMan: VVLightManager;
}

#[cfg(feature = "vv_lighting")]
impl VVLightManager {
    pub fn RE_AddLightToScene(&self, org: *const f32, intensity: f32, r: f32, g: f32, b: f32) {}
}

// G2 API stubs
extern "C" {
    #[cfg(feature = "full_g2_leak_checking")]
    pub static mut g_G2AllocServer: c_int;

    pub static mut gG2_GBMNoReconstruct: qboolean;
    pub static mut gG2_GBMUseSPMethod: qboolean;

    pub fn G2API_ListSurfaces(ghlInfo: *mut CGhoul2Info);
    pub fn G2API_ListBones(ghlInfo: *mut CGhoul2Info, modelIndex: c_int);
    pub fn G2API_HaveWeGhoul2Models(ghoul2: CGhoul2Info_v) -> qboolean;
    pub fn G2API_SetGhoul2ModelIndexes(ghoul2: CGhoul2Info_v, modelHandles: *mut qhandle_t, skinHandles: *mut qhandle_t);
    pub fn G2API_GetBoltMatrix(ghoul2: CGhoul2Info_v, modelIndex: c_int, boltIndex: c_int, boltMatrix: *mut mdxaBone_t,
                               scale: *const f32, angles: *const f32, position: c_int, usedModel: *mut qhandle_t, modelScale: *mut f32) -> c_int;
    pub fn G2API_InitGhoul2Model(ghoul2: *mut *mut CGhoul2Info_v, ghoul2Skin: *const c_char, modelIndex: c_int,
                                  customSkin: qhandle_t, customShader: qhandle_t, modelFlags: c_int, lodBias: c_int) -> c_int;
    pub fn G2API_SetBoneAngles(ghoul2: CGhoul2Info_v, modelIndex: c_int, boneName: *const c_char, angles: *mut f32, flags: c_int,
                               up: Eorientations, right: Eorientations, forward: Eorientations,
                               modelList: *mut qhandle_t, blendTime: c_int, currentTime: c_int) -> qboolean;
    pub fn G2API_CleanGhoul2Models(ghoul2: *mut *mut CGhoul2Info_v);
    pub fn G2API_SetBoneAnim(ghoul2: CGhoul2Info_v, modelIndex: c_int, boneName: *const c_char, startFrame: c_int, endFrame: c_int,
                             flags: c_int, animSpeed: f32, currentTime: c_int, setFrame: f32, blendTime: c_int) -> qboolean;
    pub fn G2API_GetBoneAnim(ghoul2: *mut CGhoul2Info_v, boneName: *const c_char, currentTime: c_int, bone_frame: *mut f32, start_frame: *mut c_int,
                             end_frame: *mut c_int, flags: *mut c_int, retAnimSpeed: *mut f32, modelIndex: *mut c_int) -> qboolean;
    pub fn G2API_GetGLAName(ghoul2: CGhoul2Info_v, modelIndex: c_int) -> *mut c_char;
    pub fn G2API_CopyGhoul2Instance(ghoul2From: CGhoul2Info_v, ghoul2To: CGhoul2Info_v, modelIndex: c_int) -> c_int;
    pub fn G2API_CopySpecificG2Model(ghoul2From: CGhoul2Info_v, modelFrom: c_int, ghoul2To: CGhoul2Info_v, modelTo: c_int);
    pub fn G2API_DuplicateGhoul2Instance(ghoul2: CGhoul2Info_v, ghoul2Dup: *mut *mut CGhoul2Info_v);
    pub fn G2API_HasGhoul2ModelOnIndex(ghoul2: *mut *mut CGhoul2Info_v, modelIndex: c_int) -> qboolean;
    pub fn G2API_RemoveGhoul2Model(ghoul2: *mut *mut CGhoul2Info_v, modelIndex: c_int) -> qboolean;
    pub fn G2API_AddBolt(ghoul2: CGhoul2Info_v, modelIndex: c_int, boneName: *const c_char) -> c_int;
    pub fn G2API_SetBoltInfo(ghoul2: CGhoul2Info_v, modelIndex: c_int, boltIndex: c_int);
    pub fn G2API_SetRootSurface(ghoul2: CGhoul2Info_v, modelIndex: c_int, surfaceName: *const c_char) -> c_int;
    pub fn G2API_SetSurfaceOnOff(ghoul2: CGhoul2Info_v, surfaceName: *const c_char, flags: c_int) -> c_int;
    pub fn G2API_SetNewOrigin(ghoul2: CGhoul2Info_v, boneIndex: c_int) -> c_int;
    pub fn G2API_GetTime(argTime: c_int) -> c_int;
    pub fn G2API_SetTime(currentTime: c_int, clock: c_int);
    pub fn G2API_SetBoneIKState(ghoul2: CGhoul2Info_v, modelIndex: c_int, boneName: *const c_char, ikState: c_int, params: *mut sharedSetBoneIKStateParams_t) -> c_int;
    pub fn G2API_IKMove(ghoul2: CGhoul2Info_v, modelIndex: c_int, params: *mut sharedIKMoveParams_t) -> c_int;
    pub fn G2API_GetSurfaceName(ghoul2: *mut CGhoul2Info_v, surfaceNumber: c_int) -> *mut c_char;
    pub fn G2API_SetSkin(ghoul2: *mut CGhoul2Info_v, modelIndex: c_int, customSkin: qhandle_t, customShader: qhandle_t) -> qboolean;
    pub fn G2API_AttachG2Model(ghoul2From: CGhoul2Info_v, modelFrom: c_int, ghoul2To: CGhoul2Info_v, toBoltIndex: c_int, toModel: c_int) -> c_int;
}

pub static mut uivm: *mut vm_t = core::ptr::null_mut();

#[cfg(feature = "use_cd_key")]
extern "C" {
    pub static mut cl_cdkey: [c_char; 34];
}

// ====================
// GetClientState
// ====================
unsafe extern "C" fn GetClientState(state: *mut uiClientState_t) {
    // Stub implementation - needs actual data
}

// ====================
// LAN_LoadCachedServers
// ====================
pub unsafe extern "C" fn LAN_LoadCachedServers() {
    #[cfg(not(any(target_os = "xbox")))]
    {
        let mut size: c_int = 0;
        let mut fileIn: fileHandle_t = 0;
        // cls.numglobalservers = cls.nummplayerservers = cls.numfavoriteservers = 0;
        // cls.numGlobalServerAddresses = 0;
        if FS_SV_FOpenFileRead(b"servercache.dat\0".as_ptr() as *const c_char, &mut fileIn) != 0 {
            FS_Read(&mut (*core::ptr::addr_of!(cls)).numglobalservers as *mut _ as *mut c_void, core::mem::size_of::<c_int>() as c_int, fileIn);
            // Additional FS_Read calls would go here following the C code pattern
            FS_FCloseFile(fileIn);
        }
    }
}

// ====================
// LAN_SaveServersToCache
// ====================
pub unsafe extern "C" fn LAN_SaveServersToCache() {
    #[cfg(not(any(target_os = "xbox")))]
    {
        let size: c_int = 0;
        let fileOut = FS_SV_FOpenFileWrite(b"servercache.dat\0".as_ptr() as *const c_char);
        // FS_Write operations would follow the C code pattern
        FS_FCloseFile(fileOut);
    }
}

// ====================
// LAN_ResetPings
// ====================
unsafe extern "C" fn LAN_ResetPings(source: c_int) {
    // Stub implementation
}

// ====================
// LAN_AddServer
// ====================
unsafe extern "C" fn LAN_AddServer(source: c_int, name: *const c_char, address: *const c_char) -> c_int {
    // Stub implementation
    -1
}

// ====================
// LAN_RemoveServer
// ====================
unsafe extern "C" fn LAN_RemoveServer(source: c_int, addr: *const c_char) {
    // Stub implementation
}

// ====================
// LAN_GetServerCount
// ====================
unsafe extern "C" fn LAN_GetServerCount(source: c_int) -> c_int {
    // Stub implementation
    0
}

// ====================
// LAN_GetServerAddressString
// ====================
unsafe extern "C" fn LAN_GetServerAddressString(source: c_int, n: c_int, buf: *mut c_char, buflen: c_int) {
    // Stub implementation
}

// ====================
// LAN_GetServerInfo
// ====================
unsafe extern "C" fn LAN_GetServerInfo(source: c_int, n: c_int, buf: *mut c_char, buflen: c_int) {
    // Stub implementation
}

// ====================
// LAN_GetServerPing
// ====================
unsafe extern "C" fn LAN_GetServerPing(source: c_int, n: c_int) -> c_int {
    // Stub implementation
    -1
}

// ====================
// LAN_GetServerPtr
// ====================
unsafe extern "C" fn LAN_GetServerPtr(source: c_int, n: c_int) -> *mut serverInfo_t {
    // Stub implementation
    core::ptr::null_mut()
}

// ====================
// LAN_CompareServers
// ====================
unsafe extern "C" fn LAN_CompareServers(source: c_int, sortKey: c_int, sortDir: c_int, s1: c_int, s2: c_int) -> c_int {
    // Stub implementation
    0
}

// ====================
// LAN_GetPingQueueCount
// ====================
unsafe extern "C" fn LAN_GetPingQueueCount() -> c_int {
    CL_GetPingQueueCount()
}

// ====================
// LAN_ClearPing
// ====================
unsafe extern "C" fn LAN_ClearPing(n: c_int) {
    CL_ClearPing(n);
}

// ====================
// LAN_GetPing
// ====================
unsafe extern "C" fn LAN_GetPing(n: c_int, buf: *mut c_char, buflen: c_int, pingtime: *mut c_int) {
    CL_GetPing(n, buf, buflen, pingtime);
}

// ====================
// LAN_GetPingInfo
// ====================
unsafe extern "C" fn LAN_GetPingInfo(n: c_int, buf: *mut c_char, buflen: c_int) {
    CL_GetPingInfo(n, buf, buflen);
}

// ====================
// LAN_MarkServerVisible
// ====================
unsafe extern "C" fn LAN_MarkServerVisible(source: c_int, n: c_int, visible: qboolean) {
    // Stub implementation
}

// =======================
// LAN_ServerIsVisible
// =======================
unsafe extern "C" fn LAN_ServerIsVisible(source: c_int, n: c_int) -> c_int {
    // Stub implementation
    qfalse
}

// =======================
// LAN_UpdateVisiblePings
// =======================
pub unsafe extern "C" fn LAN_UpdateVisiblePings(source: c_int) -> qboolean {
    CL_UpdateVisiblePings_f(source)
}

// ====================
// LAN_GetServerStatus
// ====================
pub unsafe extern "C" fn LAN_GetServerStatus(serverAddress: *mut c_char, serverStatus: *mut c_char, maxLen: c_int) -> c_int {
    CL_ServerStatus(serverAddress, serverStatus, maxLen)
}

// ====================
// CL_GetGlconfig
// ====================
unsafe extern "C" fn CL_GetGlconfig(config: *mut glconfig_t) {
    // Stub implementation - would copy cls.glconfig to config
}

// ====================
// GetClipboardData
// ====================
unsafe extern "C" fn GetClipboardData(buf: *mut c_char, buflen: c_int) {
    let cbd = Sys_GetClipboardData();

    if cbd.is_null() {
        *buf = 0;
        return;
    }

    Q_strncpyz(buf, cbd, buflen);
    Z_Free(cbd as *mut c_void);
}

// ====================
// Key_KeynumToStringBuf
// ====================
// only ever called by binding-display code, therefore returns non-technical "friendly" names
// in any language that don't necessarily match those in the config file...
pub unsafe extern "C" fn Key_KeynumToStringBuf(keynum: c_int, buf: *mut c_char, buflen: c_int) {
    let psKeyName = Key_KeynumToString(keynum);

    // see if there's a more friendly (or localised) name...
    let psKeyNameFriendly = SE_GetString(va(b"KEYNAMES_KEYNAME_%s\0".as_ptr() as *const c_char, psKeyName));

    let name_to_use = if !psKeyNameFriendly.is_null() && *psKeyNameFriendly != 0 {
        psKeyNameFriendly
    } else {
        psKeyName
    };

    Q_strncpyz(buf, name_to_use, buflen);
}

// ====================
// Key_GetBindingBuf
// ====================
unsafe extern "C" fn Key_GetBindingBuf(keynum: c_int, buf: *mut c_char, buflen: c_int) {
    let value = Key_GetBinding(keynum);
    if !value.is_null() {
        Q_strncpyz(buf, value, buflen);
    } else {
        *buf = 0;
    }
}

// ====================
// Key_GetCatcher
// ====================
pub unsafe extern "C" fn Key_GetCatcher() -> c_int {
    (*core::ptr::addr_of!(cls)).keyCatchers
}

// ====================
// Key_SetCatcher
// ====================
pub unsafe extern "C" fn Key_SetCatcher(catcher: c_int) {
    (*core::ptr::addr_of_mut!(cls)).keyCatchers = catcher;
}

#[cfg(feature = "use_cd_key")]
// ====================
// CLUI_GetCDKey
// ====================
unsafe extern "C" fn CLUI_GetCDKey(buf: *mut c_char, buflen: c_int) {
    let fs = Cvar_Get(b"fs_game\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char, CVAR_INIT | CVAR_SYSTEMINFO);
    if UI_usesUniqueCDKey() != 0 && !fs.is_null() && *(*fs).string as c_char != 0 {
        Com_Memcpy(buf as *mut c_void, (cl_cdkey.as_ptr() as *const u8).offset(16) as *const c_void, 16);
        *buf.offset(16) = 0;
    } else {
        Com_Memcpy(buf as *mut c_void, cl_cdkey.as_ptr() as *const c_void, 16);
        *buf.offset(16) = 0;
    }
}

#[cfg(feature = "use_cd_key")]
// ====================
// CLUI_SetCDKey
// ====================
unsafe extern "C" fn CLUI_SetCDKey(buf: *mut c_char) {
    let fs = Cvar_Get(b"fs_game\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char, CVAR_INIT | CVAR_SYSTEMINFO);
    if UI_usesUniqueCDKey() != 0 && !fs.is_null() && *(*fs).string as c_char != 0 {
        Com_Memcpy((cl_cdkey.as_mut_ptr() as *mut u8).offset(16) as *mut c_void, buf as *const c_void, 16);
        *cl_cdkey.as_mut_ptr().offset(32) = 0;
        // set the flag so the fle will be written at the next opportunity
        cvar_modifiedFlags |= CVAR_ARCHIVE;
    } else {
        Com_Memcpy(cl_cdkey.as_mut_ptr() as *mut c_void, buf as *const c_void, 16);
        // set the flag so the fle will be written at the next opportunity
        cvar_modifiedFlags |= CVAR_ARCHIVE;
    }
}

// ====================
// GetConfigString
// ====================
unsafe extern "C" fn GetConfigString(index: c_int, buf: *mut c_char, size: c_int) -> c_int {
    let offset: c_int;

    if index < 0 || index >= MAX_CONFIGSTRINGS {
        return qfalse;
    }

    offset = (*core::ptr::addr_of!(cl)).gameState.stringOffsets[index as usize];
    if offset == 0 {
        if size != 0 {
            *buf = 0;
        }
        return qfalse;
    }

    Q_strncpyz(buf, ((*core::ptr::addr_of!(cl)).gameState.stringData as *mut c_char).offset(offset as isize), size);

    return qtrue;
}

// ====================
// FloatAsInt
// ====================
unsafe extern "C" fn FloatAsInt(f: f32) -> c_int {
    let temp: c_int;

    *((&temp) as *mut c_int as *mut f32) = f;

    return temp;
}

unsafe extern "C" fn VM_ArgPtr(intValue: c_int) -> *mut c_void;
// #define VMA(x) VM_ArgPtr(args[x])
// #define VMF(x) ((float *)args)[x]

// ====================
// CL_UISystemCalls
//
// The ui module is making a system call
// ====================
pub unsafe extern "C" fn CL_UISystemCalls(args: *mut c_int) -> c_int {
    let args = core::slice::from_raw_parts(args, 100); // conservative upper bound

    // rww - alright, DO NOT EVER add a GAME/CGAME/UI generic call without adding a trap to match, and
    // all of these traps must be shared and have cases in sv_game, cl_cgame, and cl_ui. They must also
    // all be in the same order, and start at 100.

    match args[0] {
        TRAP_MEMSET => {
            Com_Memset(VM_ArgPtr(args[1]), args[2], args[3]);
            return 0;
        }
        TRAP_MEMCPY => {
            Com_Memcpy(VM_ArgPtr(args[1]), VM_ArgPtr(args[2]), args[3]);
            return 0;
        }
        TRAP_STRNCPY => {
            return libc::strncpy(VM_ArgPtr(args[1]) as *mut c_char, VM_ArgPtr(args[2]) as *const c_char, args[3] as usize) as c_int;
        }
        TRAP_SIN => {
            return FloatAsInt(libm::sinf(*(VM_ArgPtr(args[1]) as *const f32)));
        }
        TRAP_COS => {
            return FloatAsInt(libm::cosf(*(VM_ArgPtr(args[1]) as *const f32)));
        }
        TRAP_ATAN2 => {
            return FloatAsInt(libm::atan2f(*(VM_ArgPtr(args[1]) as *const f32), *(VM_ArgPtr(args[2]) as *const f32)));
        }
        TRAP_SQRT => {
            return FloatAsInt(libm::sqrtf(*(VM_ArgPtr(args[1]) as *const f32)));
        }
        TRAP_MATRIXMULTIPLY => {
            MatrixMultiply(VM_ArgPtr(args[1]) as *const [f32; 3], VM_ArgPtr(args[2]) as *const [f32; 3], VM_ArgPtr(args[3]) as *mut [f32; 3]);
            return 0;
        }
        TRAP_ANGLEVECTORS => {
            AngleVectors(VM_ArgPtr(args[1]) as *const f32, VM_ArgPtr(args[2]) as *mut f32, VM_ArgPtr(args[3]) as *mut f32, VM_ArgPtr(args[4]) as *mut f32);
            return 0;
        }
        TRAP_PERPENDICULARVECTOR => {
            PerpendicularVector(VM_ArgPtr(args[1]) as *mut f32, VM_ArgPtr(args[2]) as *const f32);
            return 0;
        }
        TRAP_FLOOR => {
            return FloatAsInt(libm::floorf(*(VM_ArgPtr(args[1]) as *const f32)));
        }
        TRAP_CEIL => {
            return FloatAsInt(libm::ceilf(*(VM_ArgPtr(args[1]) as *const f32)));
        }
        TRAP_TESTPRINTINT => {
            return 0;
        }
        TRAP_TESTPRINTFLOAT => {
            return 0;
        }
        TRAP_ACOS => {
            return FloatAsInt(Q_acos(*(VM_ArgPtr(args[1]) as *const f32)));
        }
        TRAP_ASIN => {
            return FloatAsInt(Q_asin(*(VM_ArgPtr(args[1]) as *const f32)));
        }

        UI_ERROR => {
            Com_Error(ERR_DROP, b"%s\0".as_ptr() as *const c_char, VM_ArgPtr(args[1]));
            return 0;
        }

        UI_PRINT => {
            Com_Printf(b"%s\0".as_ptr() as *const c_char, VM_ArgPtr(args[1]));
            return 0;
        }

        UI_MILLISECONDS => {
            return Sys_Milliseconds();
        }

        UI_CVAR_REGISTER => {
            Cvar_Register(VM_ArgPtr(args[1]) as *mut vmCvar_t, VM_ArgPtr(args[2]) as *const c_char, VM_ArgPtr(args[3]) as *const c_char, args[4]);
            return 0;
        }

        UI_CVAR_UPDATE => {
            Cvar_Update(VM_ArgPtr(args[1]) as *mut vmCvar_t);
            return 0;
        }

        UI_CVAR_SET => {
            Cvar_Set(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *const c_char);
            return 0;
        }

        UI_CVAR_VARIABLEVALUE => {
            return FloatAsInt(Cvar_VariableValue(VM_ArgPtr(args[1]) as *const c_char));
        }

        UI_CVAR_VARIABLESTRINGBUFFER => {
            Cvar_VariableStringBuffer(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *mut c_char, args[3]);
            return 0;
        }

        UI_CVAR_SETVALUE => {
            Cvar_SetValue(VM_ArgPtr(args[1]) as *const c_char, *(VM_ArgPtr(args[2]) as *const f32));
            return 0;
        }

        UI_CVAR_RESET => {
            Cvar_Reset(VM_ArgPtr(args[1]) as *const c_char);
            return 0;
        }

        UI_CVAR_CREATE => {
            Cvar_Get(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *const c_char, args[3]);
            return 0;
        }

        UI_CVAR_INFOSTRINGBUFFER => {
            Cvar_InfoStringBuffer(args[1], VM_ArgPtr(args[2]) as *mut c_char, args[3]);
            return 0;
        }

        UI_ARGC => {
            return Cmd_Argc();
        }

        UI_ARGV => {
            Cmd_ArgvBuffer(args[1], VM_ArgPtr(args[2]) as *mut c_char, args[3]);
            return 0;
        }

        UI_CMD_EXECUTETEXT => {
            Cbuf_ExecuteText(args[1], VM_ArgPtr(args[2]) as *const c_char);
            return 0;
        }

        UI_FS_FOPENFILE => {
            return FS_FOpenFileByMode(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *mut c_int, args[3] as fsMode_t);
        }

        UI_FS_READ => {
            FS_Read2(VM_ArgPtr(args[1]), args[2], args[3]);
            return 0;
        }

        UI_FS_WRITE => {
            FS_Write(VM_ArgPtr(args[1]), args[2], args[3]);
            return 0;
        }

        UI_FS_FCLOSEFILE => {
            FS_FCloseFile(args[1]);
            return 0;
        }

        UI_FS_GETFILELIST => {
            return FS_GetFileList(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *const c_char, VM_ArgPtr(args[3]) as *mut c_char, args[4]);
        }

        UI_R_REGISTERMODEL => {
            return re.RegisterModel(VM_ArgPtr(args[1]) as *const c_char);
        }

        UI_R_REGISTERSKIN => {
            return re.RegisterSkin(VM_ArgPtr(args[1]) as *const c_char);
        }

        UI_R_REGISTERSHADERNOMIP => {
            return re.RegisterShaderNoMip(VM_ArgPtr(args[1]) as *const c_char);
        }

        UI_R_SHADERNAMEFROMINDEX => {
            {
                let gameMem = VM_ArgPtr(args[1]) as *mut c_char;
                let retMem = re.ShaderNameFromIndex(args[2]);
                if !retMem.is_null() {
                    libc::strcpy(gameMem, retMem);
                } else {
                    *gameMem = 0;
                }
            }
            return 0;
        }

        UI_R_CLEARSCENE => {
            re.ClearScene();
            return 0;
        }

        UI_R_ADDREFENTITYTOSCENE => {
            re.AddRefEntityToScene(VM_ArgPtr(args[1]) as *const refEntity_t);
            return 0;
        }

        UI_R_ADDPOLYTOSCENE => {
            re.AddPolyToScene(args[1], args[2], VM_ArgPtr(args[3]) as *const polyVert_t, 1);
            return 0;
        }

        UI_R_ADDLIGHTTOSCENE => {
            #[cfg(feature = "vv_lighting")]
            {
                VVLightMan.RE_AddLightToScene(VM_ArgPtr(args[1]) as *const f32, *(VM_ArgPtr(args[2]) as *const f32), *(VM_ArgPtr(args[3]) as *const f32), *(VM_ArgPtr(args[4]) as *const f32), *(VM_ArgPtr(args[5]) as *const f32));
            }
            #[cfg(not(feature = "vv_lighting"))]
            {
                re.AddLightToScene(VM_ArgPtr(args[1]) as *const f32, *(VM_ArgPtr(args[2]) as *const f32), *(VM_ArgPtr(args[3]) as *const f32), *(VM_ArgPtr(args[4]) as *const f32), *(VM_ArgPtr(args[5]) as *const f32));
            }
            return 0;
        }

        UI_R_RENDERSCENE => {
            re.RenderScene(VM_ArgPtr(args[1]) as *const refdef_t);
            return 0;
        }

        UI_R_SETCOLOR => {
            re.SetColor(VM_ArgPtr(args[1]) as *const f32);
            return 0;
        }

        UI_R_DRAWSTRETCHPIC => {
            re.DrawStretchPic(*(VM_ArgPtr(args[1]) as *const f32), *(VM_ArgPtr(args[2]) as *const f32), *(VM_ArgPtr(args[3]) as *const f32), *(VM_ArgPtr(args[4]) as *const f32), *(VM_ArgPtr(args[5]) as *const f32), *(VM_ArgPtr(args[6]) as *const f32), *(VM_ArgPtr(args[7]) as *const f32), *(VM_ArgPtr(args[8]) as *const f32), args[9]);
            return 0;
        }

        UI_R_MODELBOUNDS => {
            re.ModelBounds(args[1], VM_ArgPtr(args[2]) as *mut f32, VM_ArgPtr(args[3]) as *mut f32);
            return 0;
        }

        UI_UPDATESCREEN => {
            SCR_UpdateScreen();
            return 0;
        }

        UI_CM_LERPTAG => {
            re.LerpTag(VM_ArgPtr(args[1]) as *mut orientation_t, args[2], args[3], args[4], *(VM_ArgPtr(args[5]) as *const f32), VM_ArgPtr(args[6]) as *const c_char);
            return 0;
        }

        UI_S_REGISTERSOUND => {
            return S_RegisterSound(VM_ArgPtr(args[1]) as *const c_char);
        }

        UI_S_STARTLOCALSOUND => {
            S_StartLocalSound(args[1], args[2]);
            return 0;
        }

        UI_KEY_KEYNUMTOSTRINGBUF => {
            Key_KeynumToStringBuf(args[1], VM_ArgPtr(args[2]) as *mut c_char, args[3]);
            return 0;
        }

        UI_KEY_GETBINDINGBUF => {
            Key_GetBindingBuf(args[1], VM_ArgPtr(args[2]) as *mut c_char, args[3]);
            return 0;
        }

        UI_KEY_SETBINDING => {
            Key_SetBinding(args[1], VM_ArgPtr(args[2]) as *const c_char);
            return 0;
        }

        UI_KEY_ISDOWN => {
            return Key_IsDown(args[1]);
        }

        UI_KEY_GETOVERSTRIKEMODE => {
            return Key_GetOverstrikeMode();
        }

        UI_KEY_SETOVERSTRIKEMODE => {
            Key_SetOverstrikeMode(args[1] as qboolean);
            return 0;
        }

        UI_KEY_CLEARSTATES => {
            Key_ClearStates();
            return 0;
        }

        UI_KEY_GETCATCHER => {
            return Key_GetCatcher();
        }

        UI_KEY_SETCATCHER => {
            Key_SetCatcher(args[1]);
            return 0;
        }

        UI_GETCLIPBOARDDATA => {
            GetClipboardData(VM_ArgPtr(args[1]) as *mut c_char, args[2]);
            return 0;
        }

        UI_GETCLIENTSTATE => {
            GetClientState(VM_ArgPtr(args[1]) as *mut uiClientState_t);
            return 0;
        }

        UI_GETGLCONFIG => {
            CL_GetGlconfig(VM_ArgPtr(args[1]) as *mut glconfig_t);
            return 0;
        }

        UI_GETCONFIGSTRING => {
            return GetConfigString(args[1], VM_ArgPtr(args[2]) as *mut c_char, args[3]);
        }

        UI_LAN_LOADCACHEDSERVERS => {
            LAN_LoadCachedServers();
            return 0;
        }

        UI_LAN_SAVECACHEDSERVERS => {
            LAN_SaveServersToCache();
            return 0;
        }

        UI_LAN_ADDSERVER => {
            return LAN_AddServer(args[1], VM_ArgPtr(args[2]) as *const c_char, VM_ArgPtr(args[3]) as *const c_char);
        }

        UI_LAN_REMOVESERVER => {
            LAN_RemoveServer(args[1], VM_ArgPtr(args[2]) as *const c_char);
            return 0;
        }

        UI_LAN_GETPINGQUEUECOUNT => {
            return LAN_GetPingQueueCount();
        }

        UI_LAN_CLEARPING => {
            LAN_ClearPing(args[1]);
            return 0;
        }

        UI_LAN_GETPING => {
            LAN_GetPing(args[1], VM_ArgPtr(args[2]) as *mut c_char, args[3], VM_ArgPtr(args[4]) as *mut c_int);
            return 0;
        }

        UI_LAN_GETPINGINFO => {
            LAN_GetPingInfo(args[1], VM_ArgPtr(args[2]) as *mut c_char, args[3]);
            return 0;
        }

        UI_LAN_GETSERVERCOUNT => {
            return LAN_GetServerCount(args[1]);
        }

        UI_LAN_GETSERVERADDRESSSTRING => {
            LAN_GetServerAddressString(args[1], args[2], VM_ArgPtr(args[3]) as *mut c_char, args[4]);
            return 0;
        }

        UI_LAN_GETSERVERINFO => {
            LAN_GetServerInfo(args[1], args[2], VM_ArgPtr(args[3]) as *mut c_char, args[4]);
            return 0;
        }

        UI_LAN_GETSERVERPING => {
            return LAN_GetServerPing(args[1], args[2]);
        }

        UI_LAN_MARKSERVERVISIBLE => {
            LAN_MarkServerVisible(args[1], args[2], args[3] as qboolean);
            return 0;
        }

        UI_LAN_SERVERISVISIBLE => {
            return LAN_ServerIsVisible(args[1], args[2]);
        }

        UI_LAN_UPDATEVISIBLEPINGS => {
            return LAN_UpdateVisiblePings(args[1]);
        }

        UI_LAN_RESETPINGS => {
            LAN_ResetPings(args[1]);
            return 0;
        }

        UI_LAN_SERVERSTATUS => {
            return LAN_GetServerStatus(VM_ArgPtr(args[1]) as *mut c_char, VM_ArgPtr(args[2]) as *mut c_char, args[3]);
        }

        UI_LAN_COMPARESERVERS => {
            return LAN_CompareServers(args[1], args[2], args[3], args[4], args[5]);
        }

        UI_MEMORY_REMAINING => {
            return Hunk_MemoryRemaining();
        }

        #[cfg(feature = "use_cd_key")]
        UI_GET_CDKEY => {
            CLUI_GetCDKey(VM_ArgPtr(args[1]) as *mut c_char, args[2]);
            return 0;
        }

        #[cfg(feature = "use_cd_key")]
        UI_SET_CDKEY => {
            CLUI_SetCDKey(VM_ArgPtr(args[1]) as *mut c_char);
            return 0;
        }

        UI_R_REGISTERFONT => {
            return re.RegisterFont(VM_ArgPtr(args[1]) as *const c_char);
        }

        UI_R_FONT_STRLENPIXELS => {
            return re.Font_StrLenPixels(VM_ArgPtr(args[1]) as *const c_char, args[2], *(VM_ArgPtr(args[3]) as *const f32));
        }

        UI_R_FONT_STRLENCHARS => {
            return re.Font_StrLenChars(VM_ArgPtr(args[1]) as *const c_char);
        }

        UI_R_FONT_STRHEIGHTPIXELS => {
            return re.Font_HeightPixels(args[1], *(VM_ArgPtr(args[2]) as *const f32));
        }

        UI_R_FONT_DRAWSTRING => {
            re.Font_DrawString(args[1], args[2], VM_ArgPtr(args[3]) as *const c_char, VM_ArgPtr(args[4]) as *const f32, args[5], args[6], *(VM_ArgPtr(args[7]) as *const f32));
            return 0;
        }

        UI_LANGUAGE_ISASIAN => {
            return re.Language_IsAsian();
        }

        UI_LANGUAGE_USESSPACES => {
            return re.Language_UsesSpaces();
        }

        UI_ANYLANGUAGE_READCHARFROMSTRING => {
            return re.AnyLanguage_ReadCharFromString(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *mut c_int, VM_ArgPtr(args[3]) as *mut qboolean);
        }

        UI_PC_ADD_GLOBAL_DEFINE => {
            return (*botlib_export).PC_AddGlobalDefine(VM_ArgPtr(args[1]) as *mut c_char);
        }
        UI_PC_LOAD_SOURCE => {
            return (*botlib_export).PC_LoadSourceHandle(VM_ArgPtr(args[1]) as *const c_char);
        }
        UI_PC_FREE_SOURCE => {
            return (*botlib_export).PC_FreeSourceHandle(args[1]);
        }
        UI_PC_READ_TOKEN => {
            return (*botlib_export).PC_ReadTokenHandle(args[1], VM_ArgPtr(args[2]) as *mut pc_token_s);
        }
        UI_PC_SOURCE_FILE_AND_LINE => {
            return (*botlib_export).PC_SourceFileAndLine(args[1], VM_ArgPtr(args[2]) as *mut c_char, VM_ArgPtr(args[3]) as *mut c_int);
        }
        UI_PC_LOAD_GLOBAL_DEFINES => {
            return (*botlib_export).PC_LoadGlobalDefines(VM_ArgPtr(args[1]) as *mut c_char);
        }
        UI_PC_REMOVE_ALL_GLOBAL_DEFINES => {
            (*botlib_export).PC_RemoveAllGlobalDefines();
            return 0;
        }

        UI_S_STOPBACKGROUNDTRACK => {
            S_StopBackgroundTrack();
            return 0;
        }
        UI_S_STARTBACKGROUNDTRACK => {
            S_StartBackgroundTrack(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *const c_char, qfalse);
            return 0;
        }

        UI_REAL_TIME => {
            return Com_RealTime(VM_ArgPtr(args[1]) as *mut qtime_s);
        }

        UI_CIN_PLAYCINEMATIC => {
            Com_DPrintf(b"UI_CIN_PlayCinematic\n\0".as_ptr() as *const c_char);
            return CIN_PlayCinematic(VM_ArgPtr(args[1]) as *const c_char, args[2], args[3], args[4], args[5], args[6]);
        }

        UI_CIN_STOPCINEMATIC => {
            return CIN_StopCinematic(args[1]);
        }

        UI_CIN_RUNCINEMATIC => {
            return CIN_RunCinematic(args[1]);
        }

        UI_CIN_DRAWCINEMATIC => {
            CIN_DrawCinematic(args[1]);
            return 0;
        }

        UI_CIN_SETEXTENTS => {
            CIN_SetExtents(args[1], args[2], args[3], args[4], args[5]);
            return 0;
        }

        UI_R_REMAP_SHADER => {
            re.RemapShader(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *const c_char, VM_ArgPtr(args[3]) as *const c_char);
            return 0;
        }

        #[cfg(feature = "use_cd_key")]
        UI_VERIFY_CDKEY => {
            return CL_CDKeyValidate(VM_ArgPtr(args[1]) as *const c_char, VM_ArgPtr(args[2]) as *const c_char);
        }

        UI_SP_GETNUMLANGUAGES => {
            return SE_GetNumLanguages();
        }

        UI_SP_GETLANGUAGENAME => {
            let languageName: *const c_char;
            let holdName = VM_ArgPtr(args[2]) as *mut c_char;

            languageName = SE_GetLanguageName(VM_ArgPtr(args[1]) as c_int);
            Q_strncpyz(holdName, languageName, 128);
            return 0;
        }

        UI_SP_GETSTRINGTEXTSTRING => {
            let text: *const c_char;

            debug_assert!(!VM_ArgPtr(args[1]).is_null());
            debug_assert!(!VM_ArgPtr(args[2]).is_null());
            text = SE_GetString(VM_ArgPtr(args[1]) as *const c_char);
            Q_strncpyz(VM_ArgPtr(args[2]) as *mut c_char, text, args[3]);
            return qtrue;
        }

        /*
        Ghoul2 Insert Start
        */
        /*
        Ghoul2 Insert Start
        */

        UI_G2_LISTSURFACES => {
            G2API_ListSurfaces(args[1] as *mut CGhoul2Info);
            return 0;
        }

        UI_G2_LISTBONES => {
            G2API_ListBones(args[1] as *mut CGhoul2Info, args[2]);
            return 0;
        }

        UI_G2_HAVEWEGHOULMODELS => {
            return G2API_HaveWeGhoul2Models(*((args[1] as *mut CGhoul2Info_v).read()));
        }

        UI_G2_SETMODELS => {
            G2API_SetGhoul2ModelIndexes(*((args[1] as *mut CGhoul2Info_v).read()), VM_ArgPtr(args[2]) as *mut qhandle_t, VM_ArgPtr(args[3]) as *mut qhandle_t);
            return 0;
        }

        UI_G2_GETBOLT => {
            return G2API_GetBoltMatrix(*((args[1] as *mut CGhoul2Info_v).read()), args[2], args[3], VM_ArgPtr(args[4]) as *mut mdxaBone_t, VM_ArgPtr(args[5]) as *const f32, VM_ArgPtr(args[6]) as *const f32, args[7], VM_ArgPtr(args[8]) as *mut qhandle_t, VM_ArgPtr(args[9]) as *mut f32);
        }

        UI_G2_GETBOLT_NOREC => {
            gG2_GBMNoReconstruct = qtrue;
            return G2API_GetBoltMatrix(*((args[1] as *mut CGhoul2Info_v).read()), args[2], args[3], VM_ArgPtr(args[4]) as *mut mdxaBone_t, VM_ArgPtr(args[5]) as *const f32, VM_ArgPtr(args[6]) as *const f32, args[7], VM_ArgPtr(args[8]) as *mut qhandle_t, VM_ArgPtr(args[9]) as *mut f32);
        }

        UI_G2_GETBOLT_NOREC_NOROT => {
            gG2_GBMNoReconstruct = qtrue;
            gG2_GBMUseSPMethod = qtrue;
            return G2API_GetBoltMatrix(*((args[1] as *mut CGhoul2Info_v).read()), args[2], args[3], VM_ArgPtr(args[4]) as *mut mdxaBone_t, VM_ArgPtr(args[5]) as *const f32, VM_ArgPtr(args[6]) as *const f32, args[7], VM_ArgPtr(args[8]) as *mut qhandle_t, VM_ArgPtr(args[9]) as *mut f32);
        }

        UI_G2_INITGHOUL2MODEL => {
            #[cfg(feature = "full_g2_leak_checking")]
            {
                g_G2AllocServer = 0;
            }
            return G2API_InitGhoul2Model(VM_ArgPtr(args[1]) as *mut *mut CGhoul2Info_v, VM_ArgPtr(args[2]) as *const c_char, args[3], args[4] as qhandle_t,
                                         args[5] as qhandle_t, args[6], args[7]);
        }

        UI_G2_COLLISIONDETECT => {
            return 0; //not supported for ui
        }

        UI_G2_COLLISIONDETECTCACHE => {
            return 0; //not supported for ui
        }

        UI_G2_ANGLEOVERRIDE => {
            return G2API_SetBoneAngles(*((args[1] as *mut CGhoul2Info_v).read()), args[2], VM_ArgPtr(args[3]) as *const c_char, VM_ArgPtr(args[4]) as *mut f32, args[5],
                                       core::mem::transmute(args[6] as Eorientations), core::mem::transmute(args[7] as Eorientations), core::mem::transmute(args[8] as Eorientations),
                                       VM_ArgPtr(args[9]) as *mut qhandle_t, args[10], args[11]);
        }

        UI_G2_CLEANMODELS => {
            #[cfg(feature = "full_g2_leak_checking")]
            {
                g_G2AllocServer = 0;
            }
            G2API_CleanGhoul2Models(VM_ArgPtr(args[1]) as *mut *mut CGhoul2Info_v);
            return 0;
        }

        UI_G2_PLAYANIM => {
            return G2API_SetBoneAnim(*((args[1] as *mut CGhoul2Info_v).read()), args[2], VM_ArgPtr(args[3]) as *const c_char, args[4], args[5],
                                     args[6], *(VM_ArgPtr(args[7]) as *const f32), args[8], *(VM_ArgPtr(args[9]) as *const f32), args[10]);
        }

        UI_G2_GETBONEANIM => {
            {
                let g2 = *((args[1] as *mut CGhoul2Info_v).read());
                let modelIndex = args[10];

                return G2API_GetBoneAnim((&g2 as *const _ as *mut CGhoul2Info_v), VM_ArgPtr(args[2]) as *const c_char, args[3], VM_ArgPtr(args[4]) as *mut f32, VM_ArgPtr(args[5]) as *mut c_int,
                                         VM_ArgPtr(args[6]) as *mut c_int, VM_ArgPtr(args[7]) as *mut c_int, VM_ArgPtr(args[8]) as *mut f32, VM_ArgPtr(args[9]) as *mut c_int);
            }
        }

        UI_G2_GETBONEFRAME => {
            { //rwwFIXMEFIXME: Just make a G2API_GetBoneFrame func too. This is dirty.
                let g2 = *((args[1] as *mut CGhoul2Info_v).read());
                let modelIndex = args[6];
                let mut iDontCare1: c_int = 0;
                let mut iDontCare2: c_int = 0;
                let mut iDontCare3: c_int = 0;
                let mut fDontCare1: f32 = 0.0;

                return G2API_GetBoneAnim((&g2 as *const _ as *mut CGhoul2Info_v), VM_ArgPtr(args[2]) as *const c_char, args[3], VM_ArgPtr(args[4]) as *mut f32, &mut iDontCare1,
                                         &mut iDontCare2, &mut iDontCare3, &mut fDontCare1, VM_ArgPtr(args[5]) as *mut c_int);
            }
        }

        UI_G2_GETGLANAME => {
            {
                let point = VM_ArgPtr(args[3]) as *mut c_char;
                let local: *mut c_char;
                local = G2API_GetGLAName(*((args[1] as *mut CGhoul2Info_v).read()), args[2]);
                if !local.is_null() {
                    libc::strcpy(point, local);
                }
            }
            return 0;
        }

        UI_G2_COPYGHOUL2INSTANCE => {
            return G2API_CopyGhoul2Instance(*((args[1] as *mut CGhoul2Info_v).read()), *((args[2] as *mut CGhoul2Info_v).read()), args[3]) as c_int;
        }

        UI_G2_COPYSPECIFICGHOUL2MODEL => {
            G2API_CopySpecificG2Model(*((args[1] as *mut CGhoul2Info_v).read()), args[2], *((args[3] as *mut CGhoul2Info_v).read()), args[4]);
            return 0;
        }

        UI_G2_DUPLICATEGHOUL2INSTANCE => {
            #[cfg(feature = "full_g2_leak_checking")]
            {
                g_G2AllocServer = 0;
            }
            G2API_DuplicateGhoul2Instance(*((args[1] as *mut CGhoul2Info_v).read()), VM_ArgPtr(args[2]) as *mut *mut CGhoul2Info_v);
            return 0;
        }

        UI_G2_HASGHOUL2MODELONINDEX => {
            return G2API_HasGhoul2ModelOnIndex(VM_ArgPtr(args[1]) as *mut *mut CGhoul2Info_v, args[2]) as c_int;
        }

        UI_G2_REMOVEGHOUL2MODEL => {
            #[cfg(feature = "full_g2_leak_checking")]
            {
                g_G2AllocServer = 0;
            }
            return G2API_RemoveGhoul2Model(VM_ArgPtr(args[1]) as *mut *mut CGhoul2Info_v, args[2]) as c_int;
        }

        UI_G2_ADDBOLT => {
            return G2API_AddBolt(*((args[1] as *mut CGhoul2Info_v).read()), args[2], VM_ArgPtr(args[3]) as *const c_char);
        }

        UI_G2_SETBOLTON => {
            G2API_SetBoltInfo(*((args[1] as *mut CGhoul2Info_v).read()), args[2], args[3]);
            return 0;
        }

        /*
        Ghoul2 Insert End
        */
        UI_G2_SETROOTSURFACE => {
            return G2API_SetRootSurface(*((args[1] as *mut CGhoul2Info_v).read()), args[2], VM_ArgPtr(args[3]) as *const c_char);
        }

        UI_G2_SETSURFACEONOFF => {
            return G2API_SetSurfaceOnOff(*((args[1] as *mut CGhoul2Info_v).read()), VM_ArgPtr(args[2]) as *const c_char, args[3]);
        }

        UI_G2_SETNEWORIGIN => {
            return G2API_SetNewOrigin(*((args[1] as *mut CGhoul2Info_v).read()), args[2]);
        }

        UI_G2_GETTIME => {
            return G2API_GetTime(0);
        }

        UI_G2_SETTIME => {
            G2API_SetTime(args[1], args[2]);
            return 0;
        }

        UI_G2_SETRAGDOLL => {
            return 0; //not supported for ui
        }

        UI_G2_ANIMATEG2MODELS => {
            return 0; //not supported for ui
        }

        UI_G2_SETBONEIKSTATE => {
            return G2API_SetBoneIKState(*((args[1] as *mut CGhoul2Info_v).read()), args[2], VM_ArgPtr(args[3]) as *const c_char, args[4], VM_ArgPtr(args[5]) as *mut sharedSetBoneIKStateParams_t);
        }

        UI_G2_IKMOVE => {
            return G2API_IKMove(*((args[1] as *mut CGhoul2Info_v).read()), args[2], VM_ArgPtr(args[3]) as *mut sharedIKMoveParams_t);
        }

        UI_G2_GETSURFACENAME => {
            { //Since returning a pointer in such a way to a VM seems to cause MASSIVE FAILURE<tm>, we will shove data into the pointer the vm passes instead
                let point = VM_ArgPtr(args[4]) as *mut c_char;
                let local: *mut c_char;
                let modelindex = args[3];

                let g2 = *((args[1] as *mut CGhoul2Info_v).read());

                local = G2API_GetSurfaceName((&g2 as *const _ as *mut CGhoul2Info_v), args[2]);
                if !local.is_null() {
                    libc::strcpy(point, local);
                }
            }

            return 0;
        }

        UI_G2_SETSKIN => {
            {
                let g2 = *((args[1] as *mut CGhoul2Info_v).read());
                let modelIndex = args[2];

                return G2API_SetSkin((&g2 as *const _ as *mut CGhoul2Info_v), args[3], args[4]) as c_int;
            }
        }

        UI_G2_ATTACHG2MODEL => {
            {
                let g2From = args[1] as *mut CGhoul2Info_v;
                let g2To = args[3] as *mut CGhoul2Info_v;

                return G2API_AttachG2Model((*g2From.read()), args[2], (*g2To.read()), args[4], args[5]);
            }
        }
        /*
        Ghoul2 Insert End
        */

        _ => {
            Com_Error(ERR_DROP, b"Bad UI system trap: %i\0".as_ptr() as *const c_char, args[0]);
        }
    }

    return 0;
}

// ====================
// CL_ShutdownUI
// ====================
pub unsafe extern "C" fn CL_ShutdownUI() {
    (*core::ptr::addr_of_mut!(cls)).keyCatchers &= !KEYCATCH_UI;
    (*core::ptr::addr_of_mut!(cls)).uiStarted = qfalse;
    if uivm.is_null() {
        return;
    }
    VM_Call(uivm, UI_SHUTDOWN);
    VM_Call(uivm, UI_MENU_RESET);
    VM_Free(uivm);
    uivm = core::ptr::null_mut();
}

// ====================
// CL_InitUI
// ====================

pub unsafe extern "C" fn CL_InitUI() {
    let v: c_int;
    let interpret: vmInterpret_t;

    // load the dll or bytecode
    if (*core::ptr::addr_of!(cl_connectedToPureServer)) != 0 {
        // #if 0
        // if sv_pure is set we only allow qvms to be loaded
        // interpret = VMI_COMPILED;
        // #else //load the module type based on what the server is doing -rww
        interpret = (*core::ptr::addr_of!(cl_connectedUI)) as vmInterpret_t;
        // #endif
    } else {
        interpret = Cvar_VariableValue(b"vm_ui\0".as_ptr() as *const c_char) as vmInterpret_t;
    }
    uivm = VM_Create(b"ui\0".as_ptr() as *const c_char, CL_UISystemCalls, interpret);
    if uivm.is_null() {
        Com_Error(ERR_FATAL, b"VM_Create on UI failed\0".as_ptr() as *const c_char);
    }

    // sanity check
    v = VM_Call(uivm, UI_GETAPIVERSION);
    if v != UI_API_VERSION {
        Com_Error(ERR_DROP, b"User Interface is version %d, expected %d\0".as_ptr() as *const c_char, v, UI_API_VERSION);
        (*core::ptr::addr_of_mut!(cls)).uiStarted = qfalse;
    } else {
        // init for this gamestate
        //rww - changed to <= CA_ACTIVE, because that is the state when we did a vid_restart
        //ingame (was just < CA_ACTIVE before, resulting in ingame menus getting wiped and
        //not reloaded on vid restart from ingame menu)
        VM_Call(uivm, UI_INIT, ((*core::ptr::addr_of!(cls)).state >= CA_AUTHORIZING && (*core::ptr::addr_of!(cls)).state <= CA_ACTIVE) as c_int);
    }
}

pub unsafe extern "C" fn UI_usesUniqueCDKey() -> qboolean {
    if !uivm.is_null() {
        return (VM_Call(uivm, UI_HASUNIQUECDKEY) == qtrue) as qboolean;
    } else {
        return qfalse;
    }
}

// ====================
// UI_GameCommand
//
// See if the current console command is claimed by the ui
// ====================
pub unsafe extern "C" fn UI_GameCommand() -> qboolean {
    if uivm.is_null() {
        return qfalse;
    }

    return (VM_Call(uivm, UI_CONSOLE_COMMAND, (*core::ptr::addr_of!(cls)).realtime) != 0) as qboolean;
}

// Stubs for extern C functions we need to declare but can't fully implement
extern "C" {
    pub static mut cl_connectedToPureServer: c_int;
    pub static mut cl_connectedUI: c_int;
}

// Stub implementations for botlib_export methods
impl botlib_export_t {
    pub unsafe fn PC_AddGlobalDefine(&self, define: *mut c_char) -> c_int { 0 }
    pub unsafe fn PC_LoadSourceHandle(&self, filename: *const c_char) -> c_int { 0 }
    pub unsafe fn PC_FreeSourceHandle(&self, handle: c_int) -> c_int { 0 }
    pub unsafe fn PC_ReadTokenHandle(&self, handle: c_int, pc_token: *mut pc_token_s) -> c_int { 0 }
    pub unsafe fn PC_SourceFileAndLine(&self, handle: c_int, filename: *mut c_char, line: *mut c_int) -> c_int { 0 }
    pub unsafe fn PC_LoadGlobalDefines(&self, filename: *mut c_char) -> c_int { 0 }
    pub unsafe fn PC_RemoveAllGlobalDefines(&self) { }
}
