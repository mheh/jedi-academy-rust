// cl_cgame.c  -- client system interaction with client game

// leave this as first line for PCH reasons...
//

// Includes translated from ../server/exe_headers.h, ../ui/ui_shared.h, ../RMG/RM_Headers.h, etc.
// In Rust, we'll declare externs as needed below

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr;

// Extern function declarations from libc and engine

extern "C" {
    // From server/exe_headers.h and other headers
    pub static mut cgvm: vm_t;

    // FIXME: Temp - Sound functions
    pub fn S_UpdateAmbientSet(name: *const c_char, origin: *const [f32; 3]);
    pub fn S_AddLocalSet(
        name: *const c_char,
        listener_origin: *const [f32; 3],
        origin: *const [f32; 3],
        entID: c_int,
        time: c_int,
    ) -> sfxHandle_t;
    pub fn AS_ParseSets();
    pub fn AS_GetBModelSound(name: *const c_char, stage: c_int) -> sfxHandle_t;
    pub fn AS_AddPrecacheEntry(name: *const c_char);
    pub fn Menus_FindByName(p: *const c_char) -> *mut menuDef_t;

    pub fn R_inPVS(p1: *const [f32; 3], p2: *const [f32; 3]) -> qboolean;

    pub fn UI_SetActiveMenu(menuname: *const c_char, menuID: *const c_char);

    // From client.h
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    pub fn Com_EventLoop();
    pub fn Com_TouchMemory();
    pub fn Sys_Milliseconds() -> c_int;
    pub fn Sys_BeginProfiling();
    pub fn Sys_LowPhysicalMemory() -> qboolean;

    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_Argv(arg: c_int) -> *const c_char;
    pub fn Cmd_ArgvBuffer(arg: c_int, buffer: *mut c_char, bufsize: c_int);
    pub fn Cmd_ArgsBuffer(buffer: *mut c_char, bufsize: c_int);
    pub fn Cmd_AddCommand(cmd_name: *const c_char, function: *const c_void);
    pub fn Cmd_TokenizeString(text: *const c_char);

    pub fn Cvar_Register(
        vmCvar: *mut vmCvar_t,
        varName: *const c_char,
        defaultValue: *const c_char,
        flags: c_int,
    );
    pub fn Cvar_Update(vmCvar: *mut vmCvar_t);
    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);

    pub fn FS_FOpenFileByMode(
        qpath: *const c_char,
        f: *mut c_int,
        mode: fsMode_t,
    ) -> c_int;
    pub fn FS_Read(buffer: *mut c_void, len: c_int, f: c_int);
    pub fn FS_Write(buffer: *const c_void, len: c_int, f: c_int);
    pub fn FS_FCloseFile(f: c_int);

    pub fn Cbuf_AddText(text: *const c_char);

    pub fn SCR_UpdateScreen();
    pub fn Con_Close();
    pub fn Con_ClearNotify();

    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;

    pub fn CM_LoadMap(mapname: *const c_char, clientLoad: qboolean, checksum: *mut c_int, subBSP: qboolean);
    pub fn CM_NumInlineModels() -> c_int;
    pub fn CM_InlineModel(index: c_int) -> c_int;
    pub fn CM_TempBoxModel(mins: *const [f32; 3], maxs: *const [f32; 3]) -> c_int;
    pub fn CM_PointContents(p: *const [f32; 3], model: c_int) -> c_int;
    pub fn CM_TransformedPointContents(
        p: *const [f32; 3],
        model: c_int,
        origin: *const [f32; 3],
        angles: *const [f32; 3],
    ) -> c_int;
    pub fn CM_BoxTrace(
        results: *mut trace_t,
        start: *const [f32; 3],
        end: *const [f32; 3],
        mins: *const [f32; 3],
        maxs: *const [f32; 3],
        model: c_int,
        brushmask: c_int,
    );
    pub fn CM_TransformedBoxTrace(
        results: *mut trace_t,
        start: *const [f32; 3],
        end: *const [f32; 3],
        mins: *const [f32; 3],
        maxs: *const [f32; 3],
        model: c_int,
        brushmask: c_int,
        origin: *const [f32; 3],
        angles: *const [f32; 3],
    );
    pub fn CM_SnapPVS(origin: *const [f32; 3], buffer: *mut u8);

    pub fn S_StopSounds();
    pub fn S_StartSound(origin: *const [f32; 3], entnum: c_int, entchannel: soundChannel_t, sfxHandle: c_int);
    pub fn S_StartLocalSound(sfxHandle: c_int, channelNum: c_int);
    pub fn S_ClearLoopingSounds();
    pub fn S_AddLoopingSound(
        entityNum: c_int,
        origin: *const [f32; 3],
        velocity: *const [f32; 3],
        sfxHandle: c_int,
        channel: soundChannel_t,
    );
    pub fn S_UpdateEntityPosition(entityNum: c_int, origin: *const [f32; 3]);
    pub fn S_Respatialize(
        entityNum: c_int,
        origin: *const [f32; 3],
        axis: *const [[f32; 3]; 3],
        inwater: c_int,
    );
    pub fn S_RegisterSound(sample: *const c_char) -> sfxHandle_t;
    pub fn S_StartBackgroundTrack(intro: *const c_char, loop_: *const c_char, fadeupTime: c_int);
    pub fn S_GetSampleLengthInMilliSeconds(sfxHandle: c_int) -> c_int;

    pub fn Z_Malloc(size: c_int, tag: memtag_t, zeroIt: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);

    pub fn CL_AddReliableCommand(cmd: *const c_char);

    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);

    // VM_Call - system call dispatcher
    pub fn VM_Call(callnum: c_int, ...) -> c_int;

    pub fn RM_CreateRandomModels(terrain: c_int, info: *const c_char);
    pub fn RM_ShutdownTerrain();

    pub fn RE_RegisterMedia_LevelLoadEnd();

    // Renderer functions via re struct - typically accessed through global re object
    // (These would normally be in a struct, but we'll declare needed functions)

    pub fn Com_SetOrgAngles(org: *mut [f32; 3], angles: *mut [f32; 3]);

    #[cfg(feature = "immersion")]
    pub fn CL_FF_Start(ffHandle: ffHandle_t, duration: c_int);
    #[cfg(feature = "immersion")]
    pub fn CL_FF_Stop(ffHandle: ffHandle_t, duration: c_int);
    #[cfg(feature = "immersion")]
    pub fn FF_StopAll();
    #[cfg(feature = "immersion")]
    pub fn FF_Shake(intensity: c_int, duration: c_int);
    #[cfg(feature = "immersion")]
    pub fn FF_Register(file: *const c_char, tag: c_int) -> ffHandle_t;
    #[cfg(feature = "immersion")]
    pub fn CL_FF_AddLoopingForce(ffHandle: ffHandle_t, intensity: c_int);

    pub fn CIN_PlayCinematic(
        arg0: *const c_char,
        arg1: c_int,
        arg2: c_int,
        arg3: c_int,
        arg4: c_int,
        arg5: c_int,
        arg6: *const c_char,
    ) -> c_int;
    pub fn CIN_StopCinematic(handle: c_int) -> c_int;
    pub fn CIN_RunCinematic(handle: c_int) -> c_int;
    pub fn CIN_DrawCinematic(handle: c_int);
    pub fn CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int);

    pub fn Menus_OpenByName(name: *const c_char);
    pub fn Menu_Reset();
    pub fn Menu_New(buffer: *mut c_char);
    pub fn Menu_FindItemByName(menu: *const menuDef_t, name: *const c_char) -> *mut itemDef_t;
    pub fn Menu_PaintAll();
    pub fn Menus_CloseAll();
    pub fn PC_StartParseSession(data: *mut c_char, outdata: *mut *mut c_char) -> c_int;
    pub fn PC_EndParseSession(data: *mut c_char);
    pub fn PC_ParseExt() -> *mut c_char;
    pub fn PC_ParseInt(value: *mut c_int);
    pub fn PC_ParseString(s: *mut *const c_char);
    pub fn PC_ParseFloat(value: *mut f32);
    pub fn String_Init();

    pub fn SE_GetString(label: *const c_char) -> *const c_char;

    // G2 functions
    #[cfg(feature = "ghoul2")]
    pub fn G2API_ListSurfaces(ghlInfo: *mut CGhoul2Info);
    #[cfg(feature = "ghoul2")]
    pub fn G2API_ListBones(ghlInfo: *mut CGhoul2Info, model: c_int);
    #[cfg(feature = "ghoul2")]
    pub fn G2API_HaveWeGhoul2Models(ghoul2: CGhoul2Info_v) -> qboolean;
    #[cfg(feature = "ghoul2")]
    pub fn G2API_SetGhoul2ModelIndexes(
        ghoul2: CGhoul2Info_v,
        modelIndexes: *mut qhandle_t,
        skinIndexes: *mut qhandle_t,
    );

    #[cfg(feature = "ghoul2")]
    pub fn G2API_SetTime(time: c_int, timeType: c_int);

    pub fn cmg_landscape(idx: c_int) -> *mut CCMLandScape;

    // Global variables from client
    pub static mut cl: clientState_t;
    pub static mut cls: clientStatic_t;
    pub static mut clc: clientConnection_t;
    pub static mut sv: server_t;

    pub static mut com_sv_running: *const cvar_t;
    pub static mut cl_activeAction: *const cvar_t;
    pub static mut cl_showTimeDelta: *const cvar_t;
    pub static mut cl_timeNudge: *const cvar_t;
    pub static mut sv_paused: *const cvar_t;
    pub static mut cl_paused: *const cvar_t;
    pub static mut com_timescale: *const cvar_t;

    pub static mut cl_mPitchOverride: f32;
    pub static mut cl_mYawOverride: f32;
    pub static mut cl_overriddenAngles: [f32; 3];
    pub static mut cl_overrideAngles: qboolean;

    pub static mut tr_distortionAlpha: f32;
    pub static mut tr_distortionStretch: f32;
    pub static mut tr_distortionPrePost: qboolean;
    pub static mut tr_distortionNegate: qboolean;

    pub static mut tr: trGlobals_t;
    pub static mut re: refexport_t;
    pub static mut cmg: cmodelsGlobals_t;
}

// Type definitions
pub type qboolean = c_int;
pub type qhandle_t = c_int;
pub type sfxHandle_t = c_int;
pub type memtag_t = c_int;
pub type fsMode_t = c_int;
pub type soundChannel_t = c_int;
pub type ffHandle_t = c_int;
pub type stereoFrame_t = c_int;

pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

// Constants from various headers
pub const MAX_CONFIGSTRINGS: c_int = 1024;
pub const MAX_GAMESTATE_CHARS: c_int = 16000;
pub const MAX_GENTITIES: c_int = 4096;
pub const MAX_ENTITIES_IN_SNAPSHOT: c_int = 256;
pub const MAX_PARSE_ENTITIES: c_int = 4096;
pub const PACKET_BACKUP: c_int = 32;
pub const PACKET_MASK: c_int = (PACKET_BACKUP - 1);
pub const CMD_BACKUP: c_int = 64;
pub const CMD_MASK: c_int = (CMD_BACKUP - 1);
pub const MAX_RELIABLE_COMMANDS: c_int = 128;

pub const CS_SERVERINFO: c_int = 0;

pub const ERR_DROP: c_int = 1;
pub const ERR_FATAL: c_int = 3;
pub const ERR_DISCONNECT: c_int = 2;

pub const CA_UNINITIALIZED: c_int = 0;
pub const CA_DISCONNECTED: c_int = 1;
pub const CA_AUTHORIZING: c_int = 2;
pub const CA_CONNECTING: c_int = 3;
pub const CA_CHALLENGING: c_int = 4;
pub const CA_CONNECTED: c_int = 5;
pub const CA_LOADING: c_int = 6;
pub const CA_PRIMED: c_int = 7;
pub const CA_ACTIVE: c_int = 8;

pub const PITCH: usize = 0;
pub const YAW: usize = 1;
pub const ROLL: usize = 2;

pub const CG_PRINT: c_int = 0;
pub const CG_ERROR: c_int = 1;
pub const CG_MILLISECONDS: c_int = 2;
pub const CG_CVAR_REGISTER: c_int = 3;
pub const CG_CVAR_UPDATE: c_int = 4;
pub const CG_CVAR_SET: c_int = 5;
pub const CG_ARGC: c_int = 6;
pub const CG_ARGV: c_int = 7;
pub const CG_ARGS: c_int = 8;
pub const CG_FS_FOPENFILE: c_int = 9;
pub const CG_FS_READ: c_int = 10;
pub const CG_FS_WRITE: c_int = 11;
pub const CG_FS_FCLOSEFILE: c_int = 12;
pub const CG_SENDCONSOLECOMMAND: c_int = 13;
pub const CG_ADDCOMMAND: c_int = 14;
pub const CG_SENDCLIENTCOMMAND: c_int = 15;
pub const CG_UPDATESCREEN: c_int = 16;
pub const CG_RMG_INIT: c_int = 17;
pub const CG_CM_REGISTER_TERRAIN: c_int = 18;
pub const CG_RE_INIT_RENDERER_TERRAIN: c_int = 19;
pub const CG_CM_LOADMAP: c_int = 20;
pub const CG_CM_NUMINLINEMODELS: c_int = 21;
pub const CG_CM_INLINEMODEL: c_int = 22;
pub const CG_CM_TEMPBOXMODEL: c_int = 23;
pub const CG_CM_POINTCONTENTS: c_int = 24;
pub const CG_CM_TRANSFORMEDPOINTCONTENTS: c_int = 25;
pub const CG_CM_BOXTRACE: c_int = 26;
pub const CG_CM_TRANSFORMEDBOXTRACE: c_int = 27;
pub const CG_CM_MARKFRAGMENTS: c_int = 28;
pub const CG_CM_SNAPPVS: c_int = 29;
pub const CG_S_STOPSOUNDS: c_int = 30;
pub const CG_S_STARTSOUND: c_int = 31;
pub const CG_S_UPDATEAMBIENTSET: c_int = 32;
pub const CG_S_ADDLOCALSET: c_int = 33;
pub const CG_AS_PARSESETS: c_int = 34;
pub const CG_AS_ADDENTRY: c_int = 35;
pub const CG_AS_GETBMODELSOUND: c_int = 36;
pub const CG_S_STARTLOCALSOUND: c_int = 37;
pub const CG_S_CLEARLOOPINGSOUNDS: c_int = 38;
pub const CG_S_ADDLOOPINGSOUND: c_int = 39;
pub const CG_S_UPDATEENTITYPOSITION: c_int = 40;
pub const CG_S_RESPATIALIZE: c_int = 41;
pub const CG_S_REGISTERSOUND: c_int = 42;
pub const CG_S_STARTBACKGROUNDTRACK: c_int = 43;
pub const CG_S_GETSAMPLELENGTH: c_int = 44;
pub const CG_FF_START: c_int = 45;
pub const CG_FF_STOP: c_int = 46;
pub const CG_FF_STOPALL: c_int = 47;
pub const CG_FF_SHAKE: c_int = 48;
pub const CG_FF_REGISTER: c_int = 49;
pub const CG_FF_ADDLOOPINGFORCE: c_int = 50;
pub const CG_FF_STARTFX: c_int = 51;
pub const CG_FF_ENSUREFX: c_int = 52;
pub const CG_FF_STOPFX: c_int = 53;
pub const CG_FF_STOPALLFX: c_int = 54;
pub const CG_FF_XBOX_SHAKE: c_int = 55;
pub const CG_FF_XBOX_DAMAGE: c_int = 56;
pub const CG_R_LOADWORLDMAP: c_int = 57;
pub const CG_R_REGISTERMODEL: c_int = 58;
pub const CG_R_REGISTERSKIN: c_int = 59;
pub const CG_R_REGISTERSHADER: c_int = 60;
pub const CG_R_REGISTERSHADERNOMIP: c_int = 61;
pub const CG_R_REGISTERFONT: c_int = 62;
pub const CG_R_FONTSTRLENPIXELS: c_int = 63;
pub const CG_R_FONTSTRLENCHARS: c_int = 64;
pub const CG_R_FONTHEIGHTPIXELS: c_int = 65;
pub const CG_R_FONTDRAWSTRING: c_int = 66;
pub const CG_LANGUAGE_ISASIAN: c_int = 67;
pub const CG_LANGUAGE_USESSPACES: c_int = 68;
pub const CG_ANYLANGUAGE_READFROMSTRING: c_int = 69;
pub const CG_R_SETREFRACTIONPROP: c_int = 70;
pub const CG_R_CLEARSCENE: c_int = 71;
pub const CG_R_ADDREFENTITYTOSCENE: c_int = 72;
pub const CG_R_INPVS: c_int = 73;
pub const CG_R_GETLIGHTING: c_int = 74;
pub const CG_R_ADDPOLYTOSCENE: c_int = 75;
pub const CG_R_ADDLIGHTTOSCENE: c_int = 76;
pub const CG_R_RENDERSCENE: c_int = 77;
pub const CG_R_SETCOLOR: c_int = 78;
pub const CG_R_DRAWSTRETCHPIC: c_int = 79;
pub const CG_R_MODELBOUNDS: c_int = 80;
pub const CG_R_LERPTAG: c_int = 81;
pub const CG_R_DRAWROTATEPIC: c_int = 82;
pub const CG_R_DRAWROTATEPIC2: c_int = 83;
pub const CG_R_SETRANGEFOG: c_int = 84;
pub const CG_R_LA_GOGGLES: c_int = 85;
pub const CG_R_SCISSOR: c_int = 86;
pub const CG_GETGLCONFIG: c_int = 87;
pub const CG_GETGAMESTATE: c_int = 88;
pub const CG_GETCURRENTSNAPSHOTNUMBER: c_int = 89;
pub const CG_GETSNAPSHOT: c_int = 90;
pub const CG_GETDEFAULTSTATE: c_int = 91;
pub const CG_GETSERVERCOMMAND: c_int = 92;
pub const CG_GETCURRENTCMDNUMBER: c_int = 93;
pub const CG_GETUSERCMD: c_int = 94;
pub const CG_SETUSERCMDVALUE: c_int = 95;
pub const CG_SETUSERCMDANGLES: c_int = 96;
pub const COM_SETORGANGLES: c_int = 97;
pub const CG_G2_LISTSURFACES: c_int = 98;
pub const CG_G2_LISTBONES: c_int = 99;
pub const CG_G2_HAVEWEGHOULMODELS: c_int = 100;
pub const CG_G2_SETMODELS: c_int = 101;
pub const CG_R_GET_LIGHT_STYLE: c_int = 102;
pub const CG_R_SET_LIGHT_STYLE: c_int = 103;
pub const CG_R_GET_BMODEL_VERTS: c_int = 104;
pub const CG_R_WORLD_EFFECT_COMMAND: c_int = 105;
pub const CG_CIN_PLAYCINEMATIC: c_int = 106;
pub const CG_CIN_STOPCINEMATIC: c_int = 107;
pub const CG_CIN_RUNCINEMATIC: c_int = 108;
pub const CG_CIN_DRAWCINEMATIC: c_int = 109;
pub const CG_CIN_SETEXTENTS: c_int = 110;
pub const CG_Z_MALLOC: c_int = 111;
pub const CG_Z_FREE: c_int = 112;
pub const CG_UI_SETACTIVE_MENU: c_int = 113;
pub const CG_UI_MENU_OPENBYNAME: c_int = 114;
pub const CG_UI_MENU_RESET: c_int = 115;
pub const CG_UI_MENU_NEW: c_int = 116;
pub const CG_UI_PARSE_INT: c_int = 117;
pub const CG_UI_PARSE_STRING: c_int = 118;
pub const CG_UI_PARSE_FLOAT: c_int = 119;
pub const CG_UI_STARTPARSESESSION: c_int = 120;
pub const CG_UI_ENDPARSESESSION: c_int = 121;
pub const CG_UI_PARSEEXT: c_int = 122;
pub const CG_UI_MENUCLOSE_ALL: c_int = 123;
pub const CG_UI_MENUPAINT_ALL: c_int = 124;
pub const CG_UI_STRING_INIT: c_int = 125;
pub const CG_UI_GETMENUINFO: c_int = 126;
pub const CG_UI_GETITEMTEXT: c_int = 127;
pub const CG_UI_GETITEMINFO: c_int = 128;
pub const CG_SP_GETSTRINGTEXTSTRING: c_int = 129;
pub const CG_CONSOLE_COMMAND: c_int = 130;
pub const CG_INIT: c_int = 131;
pub const CG_SHUTDOWN: c_int = 132;
pub const CG_DRAW_ACTIVE_FRAME: c_int = 133;

pub const RESET_TIME: c_int = 300;

//bg_public.h won't cooperate in here
pub const EF_PERMANENT: c_int = 0x00080000;

// Stub types for declarations
#[repr(C)]
pub struct vm_t {
    pub entryPoint: c_int,
    // ... other fields omitted in this stub
}

#[repr(C)]
pub struct gameState_t {
    pub stringOffsets: [c_int; MAX_CONFIGSTRINGS as usize],
    pub stringData: [c_char; MAX_GAMESTATE_CHARS as usize],
    pub dataCount: c_int,
}

#[repr(C)]
pub struct glconfig_t {
    // Stub
}

#[repr(C)]
pub struct usercmd_t {
    pub serverTime: c_int,
    // ... other fields
}

#[repr(C)]
pub struct snapshot_t {
    pub snapFlags: c_int,
    pub serverCommandSequence: c_int,
    pub ping: c_int,
    pub serverTime: c_int,
    pub areamask: [u8; 32],
    pub cmdNum: c_int,
    pub ps: playerState_t,
    pub numEntities: c_int,
    pub entities: [entityState_t; MAX_ENTITIES_IN_SNAPSHOT as usize],
}

#[repr(C)]
pub struct clSnapshot_t {
    pub snapFlags: c_int,
    pub serverCommandNum: c_int,
    pub ping: c_int,
    pub serverTime: c_int,
    pub areamask: [u8; 32],
    pub cmdNum: c_int,
    pub ps: playerState_t,
    pub numEntities: c_int,
    pub parseEntitiesNum: c_int,
    pub valid: qboolean,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eFlags: c_int,
    // ... other fields
}

#[repr(C)]
pub struct playerState_t {
    // Stub
}

#[repr(C)]
pub struct clientState_t {
    pub gameState: gameState_t,
    pub cmdNumber: c_int,
    pub cmds: [usercmd_t; CMD_BACKUP as usize],
    pub frame: clSnapshot_t,
    pub frames: [clSnapshot_t; PACKET_BACKUP as usize],
    pub parseEntities: [entityState_t; MAX_PARSE_ENTITIES as usize],
    pub parseEntitiesNum: c_int,
    pub serverTime: c_int,
    pub oldServerTime: c_int,
    pub serverTimeDelta: c_int,
    pub cgameUserCmdValue: c_int,
    pub cgameSensitivity: f32,
    pub mapname: [c_char; 128],
    pub newSnapshots: qboolean,
    pub extrapolatedSnapshot: qboolean,
    // ... other fields
}

#[repr(C)]
pub struct clientStatic_t {
    pub state: c_int,
    pub cgameStarted: qboolean,
    pub realtime: c_int,
    pub glconfig: glconfig_t,
    // ... other fields
}

#[repr(C)]
pub struct clientConnection_t {
    pub serverCommandSequence: c_int,
    pub serverCommands: [[c_char; 1024]; MAX_RELIABLE_COMMANDS as usize],
    // ... other fields
}

#[repr(C)]
pub struct server_t {
    pub svEntities: [svEntity_t; MAX_GENTITIES as usize],
    // ... other fields
}

#[repr(C)]
pub struct svEntity_t {
    pub baseline: entityState_t,
    // ... other fields
}

#[repr(C)]
pub struct vmCvar_t {
    pub handle: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub string: [c_char; 256],
}

#[repr(C)]
pub struct trace_t {
    // Stub
}

#[repr(C)]
pub struct menuDef_t {
    pub window: windowDef_t,
    // ... other fields
}

#[repr(C)]
pub struct windowDef_t {
    pub rect: rectDef_t,
    pub foreColor: [f32; 4],
    pub background: qhandle_t,
    // ... other fields
}

#[repr(C)]
pub struct rectDef_t {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[repr(C)]
pub struct itemDef_t {
    pub window: windowDef_t,
    pub text: [c_char; 256],
    // ... other fields
}

#[repr(C)]
pub struct cvar_t {
    // Stub
}

#[repr(C)]
pub struct refexport_t {
    // Stub - would contain function pointers
}

#[repr(C)]
pub struct trGlobals_t {
    pub rangedFog: f32,
    // ... other fields
}

#[repr(C)]
pub struct cmodelsGlobals_t {
    pub landScape: *mut CCMLandScape,
    pub landScapes: [*mut CCMLandScape; 4],
}

pub struct CCMLandScape {
    // Stub
}

#[repr(C)]
pub struct refEntity_t {
    // Stub
}

#[repr(C)]
pub struct polyVert_t {
    // Stub
}

#[repr(C)]
pub struct refdef_t {
    // Stub
}

#[repr(C)]
pub struct orientation_t {
    // Stub
}

#[repr(C)]
pub struct markFragment_t {
    // Stub
}

#[repr(C)]
pub struct CGhoul2Info {
    // Stub
}

pub type CGhoul2Info_v = *mut CGhoul2Info;

// ====================
// CL_GetGameState
// ====================
#[no_mangle]
pub extern "C" fn CL_GetGameState(gs: *mut gameState_t) {
    unsafe {
        *gs = cl.gameState;
    }
}

// ====================
// CL_GetGlconfig
// ====================
#[no_mangle]
pub extern "C" fn CL_GetGlconfig(glconfig: *mut glconfig_t) {
    unsafe {
        *glconfig = cls.glconfig;
    }
}

// ====================
// CL_GetUserCmd
// ====================
#[no_mangle]
pub extern "C" fn CL_GetUserCmd(cmdNumber: c_int, ucmd: *mut usercmd_t) -> qboolean {
    unsafe {
        // cmds[cmdNumber] is the last properly generated command

        // can't return anything that we haven't created yet
        if cmdNumber > cl.cmdNumber {
            Com_Error(
                ERR_DROP,
                "CL_GetUserCmd: %i >= %i\0".as_ptr() as *const c_char,
                cmdNumber,
                cl.cmdNumber,
            );
        }

        // the usercmd has been overwritten in the wrapping
        // buffer because it is too far out of date
        if cmdNumber <= cl.cmdNumber - CMD_BACKUP {
            return qfalse;
        }

        *ucmd = cl.cmds[((cmdNumber & CMD_MASK) as usize)];

        return qtrue;
    }
}

#[no_mangle]
pub extern "C" fn CL_GetCurrentCmdNumber() -> c_int {
    unsafe { cl.cmdNumber }
}

// ====================
// CL_GetCurrentSnapshotNumber
// ====================
#[no_mangle]
pub extern "C" fn CL_GetCurrentSnapshotNumber(
    snapshotNumber: *mut c_int,
    serverTime: *mut c_int,
) {
    unsafe {
        *snapshotNumber = cl.frame.messageNum;
        *serverTime = cl.frame.serverTime;
    }
}

// ====================
// CL_GetSnapshot
// ====================
#[no_mangle]
pub extern "C" fn CL_GetSnapshot(snapshotNumber: c_int, snapshot: *mut snapshot_t) -> qboolean {
    unsafe {
        let clSnap: *mut clSnapshot_t;
        let mut i: c_int;
        let mut count: c_int;

        if snapshotNumber > cl.frame.messageNum {
            Com_Error(
                ERR_DROP,
                "CL_GetSnapshot: snapshotNumber > cl.frame.messageNum\0".as_ptr() as *const c_char,
            );
        }

        // if the frame has fallen out of the circular buffer, we can't return it
        if cl.frame.messageNum - snapshotNumber >= PACKET_BACKUP {
            return qfalse;
        }

        // if the frame is not valid, we can't return it
        clSnap = &mut cl.frames[((snapshotNumber & PACKET_MASK) as usize)];
        if (*clSnap).valid == 0 {
            return qfalse;
        }

        // if the entities in the frame have fallen out of their
        // circular buffer, we can't return it
        if cl.parseEntitiesNum - (*clSnap).parseEntitiesNum >= MAX_PARSE_ENTITIES {
            return qfalse;
        }

        // write the snapshot
        (*snapshot).snapFlags = (*clSnap).snapFlags;
        (*snapshot).serverCommandSequence = (*clSnap).serverCommandNum;
        (*snapshot).ping = (*clSnap).ping;
        (*snapshot).serverTime = (*clSnap).serverTime;
        core::ptr::copy_nonoverlapping(
            (*clSnap).areamask.as_ptr(),
            (*snapshot).areamask.as_mut_ptr(),
            (*snapshot).areamask.len(),
        );
        (*snapshot).cmdNum = (*clSnap).cmdNum;
        (*snapshot).ps = (*clSnap).ps;
        count = (*clSnap).numEntities;
        if count > MAX_ENTITIES_IN_SNAPSHOT {
            Com_DPrintf(
                "CL_GetSnapshot: truncated %i entities to %i\n\0".as_ptr() as *const c_char,
                count,
                MAX_ENTITIES_IN_SNAPSHOT,
            );
            count = MAX_ENTITIES_IN_SNAPSHOT;
        }
        (*snapshot).numEntities = count;
        // Ghoul2 Insert Start
        i = 0;
        while i < count {
            let entNum: c_int = ((*clSnap).parseEntitiesNum + i) & (MAX_PARSE_ENTITIES - 1);
            (*snapshot).entities[i as usize] = cl.parseEntities[(entNum as usize)];
            i += 1;
        }
        // Ghoul2 Insert End

        // FIXME: configstring changes and server commands!!!

        return qtrue;
    }
}

#[no_mangle]
pub extern "C" fn CL_GetDefaultState(index: c_int, state: *mut entityState_t) -> qboolean {
    unsafe {
        if index < 0 || index >= MAX_GENTITIES {
            return qfalse;
        }

        // Is this safe? I think so. But it's still ugly as sin.
        if (sv.svEntities[index as usize].baseline.eFlags & EF_PERMANENT) == 0 {
            //	if (!(cl.entityBaselines[index].eFlags & EF_PERMANENT))
            {
                return qfalse;
            }
        }

        *state = sv.svEntities[index as usize].baseline;
        //	*state = cl.entityBaselines[index];

        return qtrue;
    }
}

#[no_mangle]
pub extern "C" fn CL_SetUserCmdValue(
    userCmdValue: c_int,
    sensitivityScale: f32,
    mPitchOverride: f32,
    mYawOverride: f32,
) {
    unsafe {
        cl.cgameUserCmdValue = userCmdValue;
        cl.cgameSensitivity = sensitivityScale;
        cl_mPitchOverride = mPitchOverride;
        cl_mYawOverride = mYawOverride;
    }
}

#[no_mangle]
pub extern "C" fn CL_SetUserCmdAngles(
    pitchOverride: f32,
    yawOverride: f32,
    rollOverride: f32,
) {
    unsafe {
        cl_overriddenAngles[PITCH] = pitchOverride;
        cl_overriddenAngles[YAW] = yawOverride;
        cl_overriddenAngles[ROLL] = rollOverride;
        cl_overrideAngles = qtrue;
    }
}

#[no_mangle]
pub extern "C" fn CL_AddCgameCommand(cmdName: *const c_char) {
    unsafe {
        Cmd_AddCommand(cmdName, ptr::null());
    }
}

#[no_mangle]
pub extern "C" fn CL_CgameError(string: *const c_char) {
    unsafe {
        Com_Error(ERR_DROP, "%s\0".as_ptr() as *const c_char, string);
    }
}

// =====================
// CL_ConfigstringModified
// =====================
#[no_mangle]
pub extern "C" fn CL_ConfigstringModified() {
    unsafe {
        let mut old: *const c_char;
        let s: *const c_char;
        let mut i: c_int;
        let mut index: c_int;
        let mut dup: *const c_char;
        let oldGs: gameState_t;
        let len: usize;

        index = core::ffi::CStr::from_ptr(Cmd_Argv(1))
            .to_bytes()
            .iter()
            .fold(0, |acc, &b| acc * 10 + (b as c_int - '0' as c_int));
        if index < 0 || index >= MAX_CONFIGSTRINGS {
            Com_Error(
                ERR_DROP,
                "configstring > MAX_CONFIGSTRINGS\0".as_ptr() as *const c_char,
            );
        }
        s = Cmd_Argv(2);

        old = (cl.gameState.stringData.as_ptr() as *const c_char)
            .add(cl.gameState.stringOffsets[index as usize] as usize);
        if unsafe { core::ffi::CStr::from_ptr(old) == core::ffi::CStr::from_ptr(s) } {
            return; // unchanged
        }

        // build the new gameState_t
        oldGs = cl.gameState;

        core::ptr::write_bytes(&mut cl.gameState, 0, 1);

        // leave the first 0 for uninitialized strings
        cl.gameState.dataCount = 1;

        i = 0;
        while i < MAX_CONFIGSTRINGS {
            if i == index {
                dup = s;
            } else {
                dup = (oldGs.stringData.as_ptr() as *const c_char)
                    .add(oldGs.stringOffsets[i as usize] as usize);
            }
            if *dup == 0 {
                i += 1;
                continue; // leave with the default empty string
            }

            len = core::ffi::CStr::from_ptr(dup).len();

            if len + 1 + (cl.gameState.dataCount as usize) > MAX_GAMESTATE_CHARS as usize {
                Com_Error(
                    ERR_DROP,
                    "MAX_GAMESTATE_CHARS exceeded\0".as_ptr() as *const c_char,
                );
            }

            // append it to the gameState string buffer
            cl.gameState.stringOffsets[i as usize] = cl.gameState.dataCount;
            core::ptr::copy_nonoverlapping(
                dup as *const u8,
                (cl.gameState.stringData.as_mut_ptr() as *mut u8)
                    .add(cl.gameState.dataCount as usize),
                len + 1,
            );
            cl.gameState.dataCount += len as c_int + 1;

            i += 1;
        }

        if index == CS_SERVERINFO {
            // parse serverId and other cvars
            CL_SystemInfoChanged();
        }
    }
}

extern "C" {
    fn CL_SystemInfoChanged();
}

// ===================
// CL_GetServerCommand
//
// Set up argc/argv for the given command
// ===================
#[no_mangle]
pub extern "C" fn CL_GetServerCommand(serverCommandNumber: c_int) -> qboolean {
    unsafe {
        let s: *const c_char;
        let cmd: *const c_char;

        // if we have irretrievably lost a reliable command, drop the connection
        if serverCommandNumber <= clc.serverCommandSequence - MAX_RELIABLE_COMMANDS {
            Com_Error(
                ERR_DROP,
                "CL_GetServerCommand: a reliable command was cycled out\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }

        if serverCommandNumber > clc.serverCommandSequence {
            Com_Error(
                ERR_DROP,
                "CL_GetServerCommand: requested a command not received\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }

        s = clc.serverCommands[(serverCommandNumber & (MAX_RELIABLE_COMMANDS - 1)) as usize]
            .as_ptr() as *const c_char;

        Com_DPrintf(
            "serverCommand: %i : %s\n\0".as_ptr() as *const c_char,
            serverCommandNumber,
            s,
        );

        Cmd_TokenizeString(s);
        cmd = Cmd_Argv(0);

        if core::ffi::CStr::from_ptr(cmd) == core::ffi::CStr::from_ptr("disconnect\0".as_ptr() as *const c_char) {
            Com_Error(
                ERR_DISCONNECT,
                "Server disconnected\n\0".as_ptr() as *const c_char,
            );
        }

        if core::ffi::CStr::from_ptr(cmd) == core::ffi::CStr::from_ptr("cs\0".as_ptr() as *const c_char) {
            CL_ConfigstringModified();
            // reparse the string, because CL_ConfigstringModified may have done another Cmd_TokenizeString()
            Cmd_TokenizeString(s);
            return qtrue;
        }

        // the clientLevelShot command is used during development
        // to generate 128*128 screenshots from the intermission
        // point of levels for the menu system to use
        // we pass it along to the cgame to make apropriate adjustments,
        // but we also clear the console and notify lines here
        if core::ffi::CStr::from_ptr(cmd) == core::ffi::CStr::from_ptr("clientLevelShot\0".as_ptr() as *const c_char) {
            // don't do it if we aren't running the server locally,
            // otherwise malicious remote servers could overwrite
            // the existing thumbnails
            if (*com_sv_running).integer == 0 {
                return qfalse;
            }
            // close the console
            Con_Close();
            // take a special screenshot next frame
            Cbuf_AddText("wait ; wait ; wait ; wait ; screenshot levelshot\n\0".as_ptr() as *const c_char);
            return qtrue;
        }

        // we may want to put a "connect to other server" command here

        // cgame can now act on the command
        return qtrue;
    }
}

// ====================
// CL_CM_LoadMap
//
// Just adds default parameters that cgame doesn't need to know about
// ====================
#[no_mangle]
pub extern "C" fn CL_CM_LoadMap(mapname: *const c_char, subBSP: qboolean) {
    unsafe {
        let mut checksum: c_int = 0;
        CM_LoadMap(mapname, qtrue, &mut checksum, subBSP);
    }
}

// ====================
// CL_ShutdownCGame
//
// ====================
#[no_mangle]
pub extern "C" fn CL_ShutdownCGame() {
    unsafe {
        cls.cgameStarted = qfalse;

        if cgvm.entryPoint == 0 {
            return;
        }
        VM_Call(CG_SHUTDOWN);
        #[cfg(not(feature = "xbox"))]
        {
            RM_ShutdownTerrain();
        }

        //	VM_Free( cgvm );
        //	cgvm = NULL;
    }
}

//RMG
extern "C" {
    pub fn CM_RegisterTerrain(config: *const c_char, server: bool) -> *mut CCMLandScape;
    pub fn RE_InitRendererTerrain(info: *const c_char);
}
//RMG

pub static mut g_oldRangedFog: f32 = 0.0;

// ====================
// CL_CgameSystemCalls
//
// The cgame module is making a system call
// ====================
extern "C" {
    fn VM_ArgPtr(intValue: c_int) -> *mut c_void;
}

macro_rules! VMA {
    ($x:expr) => {
        (*args.get_unchecked($x as usize)) as *const c_void
    };
}

macro_rules! VMF {
    ($x:expr) => {
        *((&args as *const c_int as *const f32).add($x as usize))
    };
}

#[no_mangle]
pub extern "C" fn CL_CgameSystemCalls(args: *mut c_int) -> c_int {
    unsafe {
        let args = core::slice::from_raw_parts(args, 256); // Safety: assuming args is a large enough array

        match args[0] {
            CG_PRINT => {
                Com_Printf("%s\0".as_ptr() as *const c_char, VMA!(1));
                return 0;
            }
            CG_ERROR => {
                Com_Error(
                    ERR_DROP,
                    "\x1b[31m%s\0".as_ptr() as *const c_char,
                    VMA!(1),
                );
                return 0;
            }
            CG_MILLISECONDS => {
                return Sys_Milliseconds();
            }
            CG_CVAR_REGISTER => {
                Cvar_Register(
                    VMA!(1) as *mut vmCvar_t,
                    VMA!(2) as *const c_char,
                    VMA!(3) as *const c_char,
                    args[4],
                );
                return 0;
            }
            CG_CVAR_UPDATE => {
                Cvar_Update(VMA!(1) as *mut vmCvar_t);
                return 0;
            }
            CG_CVAR_SET => {
                Cvar_Set(VMA!(1) as *const c_char, VMA!(2) as *const c_char);
                return 0;
            }
            CG_ARGC => {
                return Cmd_Argc();
            }
            CG_ARGV => {
                Cmd_ArgvBuffer(args[1], VMA!(2) as *mut c_char, args[3]);
                return 0;
            }
            CG_ARGS => {
                Cmd_ArgsBuffer(VMA!(1) as *mut c_char, args[2]);
                return 0;
            }
            CG_FS_FOPENFILE => {
                return FS_FOpenFileByMode(
                    VMA!(1) as *const c_char,
                    VMA!(2) as *mut c_int,
                    args[3] as fsMode_t,
                );
            }
            CG_FS_READ => {
                FS_Read(VMA!(1), args[2], args[3]);
                return 0;
            }
            CG_FS_WRITE => {
                FS_Write(VMA!(1), args[2], args[3]);
                return 0;
            }
            CG_FS_FCLOSEFILE => {
                FS_FCloseFile(args[1]);
                return 0;
            }
            CG_SENDCONSOLECOMMAND => {
                Cbuf_AddText(VMA!(1) as *const c_char);
                return 0;
            }
            CG_ADDCOMMAND => {
                CL_AddCgameCommand(VMA!(1) as *const c_char);
                return 0;
            }
            CG_SENDCLIENTCOMMAND => {
                CL_AddReliableCommand(VMA!(1) as *const c_char);
                return 0;
            }
            CG_UPDATESCREEN => {
                // this is used during lengthy level loading, so pump message loop
                Com_EventLoop(); // FIXME: if a server restarts here, BAD THINGS HAPPEN!
                SCR_UpdateScreen();
                return 0;
            }

            CG_RMG_INIT => {
                #[cfg(feature = "xbox")]
                {
                    Com_Error(
                        ERR_FATAL,
                        "ERROR: Terrain unsupported on Xbox.\n\0".as_ptr() as *const c_char,
                    );
                }
                #[cfg(not(feature = "xbox"))]
                {
                    RM_CreateRandomModels(args[1], VMA!(2) as *const c_char);
                    cmg.landScape = *cmg.landScapes.get_unchecked(0);
                    if !cmg.landScape.is_null() {
                        (*cmg.landScape).rand_seed((*cmg.landScape).get_rand_seed());
                    }
                }
                return 0;
            }
            CG_CM_REGISTER_TERRAIN => {
                #[cfg(feature = "xbox")]
                {
                    Com_Error(
                        ERR_FATAL,
                        "ERROR: Terrain unsupported on Xbox.\n\0".as_ptr() as *const c_char,
                    );
                }
                #[cfg(not(feature = "xbox"))]
                {
                    let terrain = CM_RegisterTerrain(VMA!(1) as *const c_char, false);
                    if !terrain.is_null() {
                        return (*terrain).GetTerrainId();
                    }
                }
                return 0;
            }
            CG_RE_INIT_RENDERER_TERRAIN => {
                #[cfg(feature = "xbox")]
                {
                    Com_Error(
                        ERR_FATAL,
                        "ERROR: Terrain unsupported on Xbox.\n\0".as_ptr() as *const c_char,
                    );
                }
                #[cfg(not(feature = "xbox"))]
                {
                    RE_InitRendererTerrain(VMA!(1) as *const c_char);
                }
                return 0;
            }

            CG_CM_LOADMAP => {
                #[cfg(feature = "xbox")]
                {
                    CL_CM_LoadMap(VMA!(1) as *const c_char, 0);
                }
                #[cfg(not(feature = "xbox"))]
                {
                    CL_CM_LoadMap(VMA!(1) as *const c_char, args[2]);
                }
                return 0;
            }
            CG_CM_NUMINLINEMODELS => {
                return CM_NumInlineModels();
            }
            CG_CM_INLINEMODEL => {
                return CM_InlineModel(args[1]);
            }
            CG_CM_TEMPBOXMODEL => {
                return CM_TempBoxModel(VMA!(1) as *const [f32; 3], VMA!(2) as *const [f32; 3]);
            }
            CG_CM_POINTCONTENTS => {
                return CM_PointContents(VMA!(1) as *const [f32; 3], args[2]);
            }
            CG_CM_TRANSFORMEDPOINTCONTENTS => {
                return CM_TransformedPointContents(
                    VMA!(1) as *const [f32; 3],
                    args[2],
                    VMA!(3) as *const [f32; 3],
                    VMA!(4) as *const [f32; 3],
                );
            }
            CG_CM_BOXTRACE => {
                CM_BoxTrace(
                    VMA!(1) as *mut trace_t,
                    VMA!(2) as *const [f32; 3],
                    VMA!(3) as *const [f32; 3],
                    VMA!(4) as *const [f32; 3],
                    VMA!(5) as *const [f32; 3],
                    args[6],
                    args[7],
                );
                return 0;
            }
            CG_CM_TRANSFORMEDBOXTRACE => {
                CM_TransformedBoxTrace(
                    VMA!(1) as *mut trace_t,
                    VMA!(2) as *const [f32; 3],
                    VMA!(3) as *const [f32; 3],
                    VMA!(4) as *const [f32; 3],
                    VMA!(5) as *const [f32; 3],
                    args[6],
                    args[7],
                    VMA!(8) as *const [f32; 3],
                    VMA!(9) as *const [f32; 3],
                );
                return 0;
            }
            CG_CM_MARKFRAGMENTS => {
                return re.MarkFragments(
                    args[1],
                    VMA!(2) as *const [[f32; 3]],
                    VMA!(3) as *const [f32; 3],
                    args[4],
                    VMA!(5) as *mut f32,
                    args[6],
                    VMA!(7) as *mut markFragment_t,
                );
            }
            CG_CM_SNAPPVS => {
                CM_SnapPVS(VMA!(1) as *const [f32; 3], VMA!(2) as *mut u8);
                return 0;
            }
            CG_S_STOPSOUNDS => {
                S_StopSounds();
                return 0;
            }

            CG_S_STARTSOUND => {
                // stops an ERR_DROP internally if called illegally from game side, but note that it also gets here
                //	legally during level start where normally the internal s_soundStarted check would return. So ok to hit this.
                if cls.cgameStarted == 0 {
                    return 0;
                }
                S_StartSound(VMA!(1) as *const [f32; 3], args[2], args[3] as soundChannel_t, args[4]);
                return 0;
            }
            CG_S_UPDATEAMBIENTSET => {
                // stops an ERR_DROP internally if called illegally from game side, but note that it also gets here
                //	legally during level start where normally the internal s_soundStarted check would return. So ok to hit this.
                if cls.cgameStarted == 0 {
                    return 0;
                }
                S_UpdateAmbientSet(VMA!(1) as *const c_char, VMA!(2) as *const [f32; 3]);
                return 0;
            }
            CG_S_ADDLOCALSET => {
                return S_AddLocalSet(
                    VMA!(1) as *const c_char,
                    VMA!(2) as *const [f32; 3],
                    VMA!(3) as *const [f32; 3],
                    args[4],
                    args[5],
                );
            }
            CG_AS_PARSESETS => {
                AS_ParseSets();
                return 0;
            }
            CG_AS_ADDENTRY => {
                AS_AddPrecacheEntry(VMA!(1) as *const c_char);
                return 0;
            }
            CG_AS_GETBMODELSOUND => {
                return AS_GetBModelSound(VMA!(1) as *const c_char, args[2]);
            }
            CG_S_STARTLOCALSOUND => {
                // stops an ERR_DROP internally if called illegally from game side, but note that it also gets here
                //	legally during level start where normally the internal s_soundStarted check would return. So ok to hit this.
                if cls.cgameStarted == 0 {
                    return 0;
                }
                S_StartLocalSound(args[1], args[2]);
                return 0;
            }
            CG_S_CLEARLOOPINGSOUNDS => {
                S_ClearLoopingSounds();
                return 0;
            }
            CG_S_ADDLOOPINGSOUND => {
                // stops an ERR_DROP internally if called illegally from game side, but note that it also gets here
                //	legally during level start where normally the internal s_soundStarted check would return. So ok to hit this.
                if cls.cgameStarted == 0 {
                    return 0;
                }
                S_AddLoopingSound(
                    args[1],
                    VMA!(2) as *const [f32; 3],
                    VMA!(3) as *const [f32; 3],
                    args[4],
                    args[5] as soundChannel_t,
                );
                return 0;
            }
            CG_S_UPDATEENTITYPOSITION => {
                S_UpdateEntityPosition(args[1], VMA!(2) as *const [f32; 3]);
                return 0;
            }
            CG_S_RESPATIALIZE => {
                S_Respatialize(
                    args[1],
                    VMA!(2) as *const [f32; 3],
                    VMA!(3) as *const [[f32; 3]; 3],
                    args[4],
                );
                return 0;
            }
            CG_S_REGISTERSOUND => {
                return S_RegisterSound(VMA!(1) as *const c_char);
            }
            CG_S_STARTBACKGROUNDTRACK => {
                S_StartBackgroundTrack(VMA!(1) as *const c_char, VMA!(2) as *const c_char, args[3]);
                return 0;
            }
            CG_S_GETSAMPLELENGTH => {
                return S_GetSampleLengthInMilliSeconds(args[1]);
            }
            #[cfg(feature = "immersion")]
            CG_FF_START => {
                CL_FF_Start(args[1] as ffHandle_t, args[2] as c_int);
                return 0;
            }
            #[cfg(feature = "immersion")]
            CG_FF_STOP => {
                CL_FF_Stop(args[1] as ffHandle_t, args[2] as c_int);
                return 0;
            }
            #[cfg(feature = "immersion")]
            CG_FF_STOPALL => {
                FF_StopAll();
                return 0;
            }
            #[cfg(feature = "immersion")]
            CG_FF_SHAKE => {
                FF_Shake(args[1] as c_int, args[2] as c_int);
                return 0;
            }
            #[cfg(feature = "immersion")]
            CG_FF_REGISTER => {
                return FF_Register(VMA!(1) as *const c_char, args[2] as c_int);
            }
            #[cfg(feature = "immersion")]
            CG_FF_ADDLOOPINGFORCE => {
                CL_FF_AddLoopingForce(args[1] as ffHandle_t, args[2] as c_int);
                return 0;
            }
            #[cfg(not(feature = "immersion"))]
            CG_FF_STARTFX => {
                FFFX_START(args[1] as c_int);
                return 0;
            }
            #[cfg(not(feature = "immersion"))]
            CG_FF_ENSUREFX => {
                FFFX_ENSURE(args[1] as c_int);
                return 0;
            }
            #[cfg(not(feature = "immersion"))]
            CG_FF_STOPFX => {
                FFFX_STOP(args[1] as c_int);
                return 0;
            }
            #[cfg(not(feature = "immersion"))]
            CG_FF_STOPALLFX => {
                FFFX_STOPALL;
                return 0;
            }
            #[cfg(feature = "xbox")]
            CG_FF_XBOX_SHAKE => {
                FF_XboxShake(VMF!(1), args[2] as c_int);
                return 0;
            }
            #[cfg(feature = "xbox")]
            CG_FF_XBOX_DAMAGE => {
                FF_XboxDamage(args[1] as c_int, VMF!(2));
                return 0;
            }
            CG_R_LOADWORLDMAP => {
                re.LoadWorld(VMA!(1) as *const c_char);
                return 0;
            }
            CG_R_REGISTERMODEL => {
                return re.RegisterModel(VMA!(1) as *const c_char);
            }
            CG_R_REGISTERSKIN => {
                return re.RegisterSkin(VMA!(1) as *const c_char);
            }
            CG_R_REGISTERSHADER => {
                return re.RegisterShader(VMA!(1) as *const c_char);
            }
            CG_R_REGISTERSHADERNOMIP => {
                return re.RegisterShaderNoMip(VMA!(1) as *const c_char);
            }
            CG_R_REGISTERFONT => {
                return re.RegisterFont(VMA!(1) as *const c_char);
            }
            CG_R_FONTSTRLENPIXELS => {
                return re.Font_StrLenPixels(VMA!(1) as *const c_char, args[2], VMF!(3));
            }
            CG_R_FONTSTRLENCHARS => {
                return re.Font_StrLenChars(VMA!(1) as *const c_char);
            }
            CG_R_FONTHEIGHTPIXELS => {
                return re.Font_HeightPixels(args[1], VMF!(2));
            }
            CG_R_FONTDRAWSTRING => {
                re.Font_DrawString(args[1], args[2], VMA!(3) as *const c_char, VMA!(4) as *mut f32, args[5], args[6], VMF!(7));
                return 0;
            }
            CG_LANGUAGE_ISASIAN => {
                return re.Language_IsAsian();
            }
            CG_LANGUAGE_USESSPACES => {
                return re.Language_UsesSpaces();
            }
            CG_ANYLANGUAGE_READFROMSTRING => {
                return re.AnyLanguage_ReadCharFromString(
                    VMA!(1) as *const c_char,
                    VMA!(2) as *mut c_int,
                    VMA!(3) as *mut qboolean,
                );
            }
            CG_R_SETREFRACTIONPROP => {
                tr_distortionAlpha = VMF!(1);
                tr_distortionStretch = VMF!(2);
                tr_distortionPrePost = args[3];
                tr_distortionNegate = args[4];
                return 0;
            }
            CG_R_CLEARSCENE => {
                re.ClearScene();
                return 0;
            }
            CG_R_ADDREFENTITYTOSCENE => {
                re.AddRefEntityToScene(VMA!(1) as *const refEntity_t);
                return 0;
            }

            CG_R_INPVS => {
                return R_inPVS(VMA!(1) as *const [f32; 3], VMA!(2) as *const [f32; 3]);
            }

            CG_R_GETLIGHTING => {
                return re.GetLighting(
                    VMA!(1) as *const f32,
                    VMA!(2) as *mut f32,
                    VMA!(3) as *mut f32,
                    VMA!(4) as *mut f32,
                );
            }
            CG_R_ADDPOLYTOSCENE => {
                re.AddPolyToScene(args[1], args[2], VMA!(3) as *const polyVert_t);
                return 0;
            }
            CG_R_ADDLIGHTTOSCENE => {
                #[cfg(feature = "vv_lighting")]
                {
                    VVLightMan.RE_AddLightToScene(
                        VMA!(1) as *const f32,
                        VMF!(2),
                        VMF!(3),
                        VMF!(4),
                        VMF!(5),
                    );
                }
                #[cfg(not(feature = "vv_lighting"))]
                {
                    re.AddLightToScene(
                        VMA!(1) as *const f32,
                        VMF!(2),
                        VMF!(3),
                        VMF!(4),
                        VMF!(5),
                    );
                }
                return 0;
            }
            CG_R_RENDERSCENE => {
                re.RenderScene(VMA!(1) as *const refdef_t);
                return 0;
            }
            CG_R_SETCOLOR => {
                re.SetColor(VMA!(1) as *const f32);
                return 0;
            }
            CG_R_DRAWSTRETCHPIC => {
                re.DrawStretchPic(VMF!(1), VMF!(2), VMF!(3), VMF!(4), VMF!(5), VMF!(6), VMF!(7), VMF!(8), args[9]);
                return 0;
            }
            CG_R_MODELBOUNDS => {
                re.ModelBounds(args[1], VMA!(2) as *mut f32, VMA!(3) as *mut f32);
                return 0;
            }
            CG_R_LERPTAG => {
                re.LerpTag(
                    VMA!(1) as *mut orientation_t,
                    args[2],
                    args[3],
                    args[4],
                    VMF!(5),
                    VMA!(6) as *const c_char,
                );
                return 0;
            }
            CG_R_DRAWROTATEPIC => {
                re.DrawRotatePic(
                    VMF!(1), VMF!(2), VMF!(3), VMF!(4), VMF!(5), VMF!(6), VMF!(7), VMF!(8), VMF!(9), args[10],
                );
                return 0;
            }
            CG_R_DRAWROTATEPIC2 => {
                re.DrawRotatePic2(
                    VMF!(1), VMF!(2), VMF!(3), VMF!(4), VMF!(5), VMF!(6), VMF!(7), VMF!(8), VMF!(9), args[10],
                );
                return 0;
            }
            CG_R_SETRANGEFOG => {
                if tr.rangedFog <= 0.0 {
                    g_oldRangedFog = tr.rangedFog;
                }
                tr.rangedFog = VMF!(1);
                if tr.rangedFog == 0.0 && g_oldRangedFog != 0.0 {
                    // restore to previous state if applicable
                    tr.rangedFog = g_oldRangedFog;
                }
                return 0;
            }
            CG_R_LA_GOGGLES => {
                re.LAGoggles();
                return 0;
            }
            CG_R_SCISSOR => {
                re.Scissor(VMF!(1), VMF!(2), VMF!(3), VMF!(4));
                return 0;
            }
            CG_GETGLCONFIG => {
                CL_GetGlconfig(VMA!(1) as *mut glconfig_t);
                return 0;
            }
            CG_GETGAMESTATE => {
                CL_GetGameState(VMA!(1) as *mut gameState_t);
                return 0;
            }
            CG_GETCURRENTSNAPSHOTNUMBER => {
                CL_GetCurrentSnapshotNumber(VMA!(1) as *mut c_int, VMA!(2) as *mut c_int);
                return 0;
            }
            CG_GETSNAPSHOT => {
                return CL_GetSnapshot(args[1], VMA!(2) as *mut snapshot_t);
            }

            CG_GETDEFAULTSTATE => {
                return CL_GetDefaultState(args[1], VMA!(2) as *mut entityState_t);
            }

            CG_GETSERVERCOMMAND => {
                return CL_GetServerCommand(args[1]);
            }
            CG_GETCURRENTCMDNUMBER => {
                return CL_GetCurrentCmdNumber();
            }
            CG_GETUSERCMD => {
                return CL_GetUserCmd(args[1], VMA!(2) as *mut usercmd_t);
            }
            CG_SETUSERCMDVALUE => {
                CL_SetUserCmdValue(args[1], VMF!(2), VMF!(3), VMF!(4));
                return 0;
            }
            CG_SETUSERCMDANGLES => {
                CL_SetUserCmdAngles(VMF!(1), VMF!(2), VMF!(3));
                return 0;
            }
            COM_SETORGANGLES => {
                Com_SetOrgAngles(VMA!(1) as *mut [f32; 3], VMA!(2) as *mut [f32; 3]);
                return 0;
            }
            // Ghoul2 Insert Start

            CG_G2_LISTSURFACES => {
                #[cfg(feature = "ghoul2")]
                G2API_ListSurfaces(VMA!(1) as *mut CGhoul2Info);
                return 0;
            }

            CG_G2_LISTBONES => {
                #[cfg(feature = "ghoul2")]
                G2API_ListBones(VMA!(1) as *mut CGhoul2Info, args[2]);
                return 0;
            }

            CG_G2_HAVEWEGHOULMODELS => {
                #[cfg(feature = "ghoul2")]
                return G2API_HaveWeGhoul2Models(*(VMA!(1) as *const CGhoul2Info_v));
                #[cfg(not(feature = "ghoul2"))]
                return 0;
            }

            CG_G2_SETMODELS => {
                #[cfg(feature = "ghoul2")]
                G2API_SetGhoul2ModelIndexes(
                    *(VMA!(1) as *const CGhoul2Info_v),
                    VMA!(2) as *mut qhandle_t,
                    VMA!(3) as *mut qhandle_t,
                );
                return 0;
            }

            // Ghoul2 Insert End

            CG_R_GET_LIGHT_STYLE => {
                re.GetLightStyle(args[1], VMA!(2) as *mut u8);
                return 0;
            }
            CG_R_SET_LIGHT_STYLE => {
                re.SetLightStyle(args[1], args[2]);
                return 0;
            }

            CG_R_GET_BMODEL_VERTS => {
                re.GetBModelVerts(
                    args[1],
                    VMA!(2) as *mut [[f32; 3]],
                    VMA!(3) as *mut f32,
                );
                return 0;
            }

            CG_R_WORLD_EFFECT_COMMAND => {
                re.WorldEffectCommand(VMA!(1) as *const c_char);
                return 0;
            }

            CG_CIN_PLAYCINEMATIC => {
                return CIN_PlayCinematic(
                    VMA!(1) as *const c_char,
                    args[2],
                    args[3],
                    args[4],
                    args[5],
                    args[6],
                    VMA!(7) as *const c_char,
                );
            }

            CG_CIN_STOPCINEMATIC => {
                return CIN_StopCinematic(args[1]);
            }

            CG_CIN_RUNCINEMATIC => {
                return CIN_RunCinematic(args[1]);
            }

            CG_CIN_DRAWCINEMATIC => {
                CIN_DrawCinematic(args[1]);
                return 0;
            }

            CG_CIN_SETEXTENTS => {
                CIN_SetExtents(args[1], args[2], args[3], args[4], args[5]);
                return 0;
            }

            CG_Z_MALLOC => {
                return Z_Malloc(args[1], args[2] as memtag_t, qfalse) as c_int;
            }

            CG_Z_FREE => {
                Z_Free(VMA!(1) as *mut c_void);
                return 0;
            }

            CG_UI_SETACTIVE_MENU => {
                UI_SetActiveMenu(VMA!(1) as *const c_char, ptr::null());
                return 0;
            }

            CG_UI_MENU_OPENBYNAME => {
                Menus_OpenByName(VMA!(1) as *const c_char);
                return 0;
            }

            CG_UI_MENU_RESET => {
                Menu_Reset();
                return 0;
            }

            CG_UI_MENU_NEW => {
                Menu_New(VMA!(1) as *mut c_char);
                return 0;
            }

            CG_UI_PARSE_INT => {
                PC_ParseInt(VMA!(1) as *mut c_int);
                return 0;
            }

            CG_UI_PARSE_STRING => {
                PC_ParseString(VMA!(1) as *mut *const c_char);
                return 0;
            }

            CG_UI_PARSE_FLOAT => {
                PC_ParseFloat(VMA!(1) as *mut f32);
                return 0;
            }

            CG_UI_STARTPARSESESSION => {
                return PC_StartParseSession(VMA!(1) as *mut c_char, VMA!(2) as *mut *mut c_char);
            }

            CG_UI_ENDPARSESESSION => {
                PC_EndParseSession(VMA!(1) as *mut c_char);
                return 0;
            }

            CG_UI_PARSEEXT => {
                let holdPtr: *mut *mut c_char = VMA!(1) as *mut *mut c_char;
                *holdPtr = PC_ParseExt();
                return 0;
            }

            CG_UI_MENUCLOSE_ALL => {
                Menus_CloseAll();
                return 0;
            }

            CG_UI_MENUPAINT_ALL => {
                Menu_PaintAll();
                return 0;
            }

            CG_UI_STRING_INIT => {
                String_Init();
                return 0;
            }

            CG_UI_GETMENUINFO => {
                let menu: *mut menuDef_t = Menus_FindByName(VMA!(1) as *const c_char);
                let xPos: *mut c_int = VMA!(2) as *mut c_int;
                let yPos: *mut c_int = VMA!(3) as *mut c_int;
                let w: *mut c_int = VMA!(4) as *mut c_int;
                let h: *mut c_int = VMA!(5) as *mut c_int;
                let result: c_int;

                if !menu.is_null() {
                    *xPos = (*menu).window.rect.x as c_int;
                    *yPos = (*menu).window.rect.y as c_int;
                    *w = (*menu).window.rect.w as c_int;
                    *h = (*menu).window.rect.h as c_int;
                    result = qtrue;
                } else {
                    result = qfalse;
                }

                return result;
            }

            CG_UI_GETITEMTEXT => {
                let menu: *mut menuDef_t = Menus_FindByName(VMA!(1) as *const c_char);
                let item: *mut itemDef_t;
                let result: c_int;

                if !menu.is_null() {
                    item = Menu_FindItemByName(
                        menu as *const menuDef_t,
                        VMA!(2) as *const c_char,
                    );
                    if !item.is_null() {
                        Q_strncpyz(VMA!(3) as *mut c_char, (*item).text.as_ptr(), 256);
                        result = qtrue;
                    } else {
                        result = qfalse;
                    }
                } else {
                    result = qfalse;
                }

                return result;
            }

            CG_UI_GETITEMINFO => {
                let menu: *mut menuDef_t = Menus_FindByName(VMA!(1) as *const c_char);

                if !menu.is_null() {
                    let item: *mut itemDef_t =
                        Menu_FindItemByName(menu as *const menuDef_t, VMA!(2) as *const c_char);
                    if !item.is_null() {
                        let xPos: *mut c_int = VMA!(3) as *mut c_int;
                        let yPos: *mut c_int = VMA!(4) as *mut c_int;
                        let w: *mut c_int = VMA!(5) as *mut c_int;
                        let h: *mut c_int = VMA!(6) as *mut c_int;
                        let color: *mut [f32; 4] = VMA!(7) as *mut [f32; 4];
                        let background: *mut qhandle_t = VMA!(8) as *mut qhandle_t;

                        *xPos = (*item).window.rect.x as c_int;
                        *yPos = (*item).window.rect.y as c_int;
                        *w = (*item).window.rect.w as c_int;
                        *h = (*item).window.rect.h as c_int;

                        if color.is_null() {
                            return qfalse;
                        }

                        (*color)[0] = (*item).window.foreColor[0];
                        (*color)[1] = (*item).window.foreColor[1];
                        (*color)[2] = (*item).window.foreColor[2];
                        (*color)[3] = (*item).window.foreColor[3];

                        if background.is_null() {
                            return qfalse;
                        }
                        *background = (*item).window.background;

                        return qtrue;
                    } else {
                        return qfalse;
                    }
                } else {
                    return qfalse;
                }
            }

            CG_SP_GETSTRINGTEXTSTRING => {
                let text: *const c_char = SE_GetString(VMA!(1) as *const c_char);

                if !VMA!(2).is_null() {
                    // only if dest buffer supplied...
                    if *text != 0 {
                        Q_strncpyz(VMA!(2) as *mut c_char, text, args[3]);
                    } else {
                        Com_sprintf(
                            VMA!(2) as *mut c_char,
                            args[3] as usize,
                            "??%s\0".as_ptr() as *const c_char,
                            VMA!(1),
                        );
                    }
                }
                return core::ffi::CStr::from_ptr(text).len() as c_int;
            }
            _ => {
                Com_Error(
                    ERR_DROP,
                    "Bad cgame system trap: %i\0".as_ptr() as *const c_char,
                    args[0],
                );
            }
        }
        return 0;
    }
}

extern "C" {
    fn FFFX_START(effect: c_int);
    fn FFFX_ENSURE(effect: c_int);
    fn FFFX_STOP(effect: c_int);
    fn FFFX_STOPALL();
    #[cfg(feature = "xbox")]
    fn FF_XboxShake(intensity: f32, duration: c_int);
    #[cfg(feature = "xbox")]
    fn FF_XboxDamage(damage: c_int, intensity: f32);
}

// ====================
// CL_InitCGame
//
// Should only be called by CL_StartHunkUsers
// ====================
#[no_mangle]
pub extern "C" fn CL_InitCGame() {
    unsafe {
        let info: *const c_char;
        let mapname: *const c_char;
        let mut t1: c_int;
        let mut t2: c_int;

        t1 = Sys_Milliseconds();

        // put away the console
        Con_Close();

        // find the current mapname
        info =
            (cl.gameState.stringData.as_ptr() as *const c_char)
                .add(cl.gameState.stringOffsets[CS_SERVERINFO as usize] as usize);
        mapname = Info_ValueForKey(info, "mapname\0".as_ptr() as *const c_char);
        Com_sprintf(
            cl.mapname.as_mut_ptr(),
            cl.mapname.len(),
            "maps/%s.bsp\0".as_ptr() as *const c_char,
            mapname,
        );

        cls.state = CA_LOADING;

        // init for this gamestate
        VM_Call(CG_INIT, clc.serverCommandSequence);

        // we will send a usercmd this frame, which
        // will cause the server to send us the first snapshot
        cls.state = CA_PRIMED;

        t2 = Sys_Milliseconds();

        //Com_Printf( "CL_InitCGame: %5.2f seconds\n", (t2-t1)/1000.0 );
        // have the renderer touch all its images, so they are present
        // on the card even if the driver does deferred loading
        re.EndRegistration();

        // make sure everything is paged in
        //	if (!Sys_LowPhysicalMemory())
        {
            Com_TouchMemory();
        }

        // clear anything that got printed
        Con_ClearNotify();
    }
}

// ====================
// CL_GameCommand
//
// See if the current console command is claimed by the cgame
// ====================
#[no_mangle]
pub extern "C" fn CL_GameCommand() -> qboolean {
    unsafe {
        if cls.state != CA_ACTIVE {
            return qfalse;
        }

        return VM_Call(CG_CONSOLE_COMMAND);
    }
}

// =====================
// CL_CGameRendering
// =====================
#[no_mangle]
pub extern "C" fn CL_CGameRendering(stereo: stereoFrame_t) {
    unsafe {
        let mut timei: c_int = cl.serverTime;
        if timei > 60 {
            timei -= 0;
        }
        G2API_SetTime(cl.serverTime, 2); // G2T_CG_TIME = 2
        VM_Call(CG_DRAW_ACTIVE_FRAME, timei, stereo, qfalse);
        //	VM_Debug( 0 );
    }
}

// =================
// CL_AdjustTimeDelta
//
// Adjust the clients view of server time.
//
// We attempt to have cl.serverTime exactly equal the server's view
// of time plus the timeNudge, but with variable latencies over
// the internet it will often need to drift a bit to match conditions.
//
// Our ideal time would be to have the adjusted time aproach, but not pass,
// the very latest snapshot.
//
// Adjustments are only made when a new snapshot arrives, which keeps the
// adjustment process framerate independent and prevents massive overadjustment
// during times of significant packet loss.
// =================

#[no_mangle]
pub extern "C" fn CL_AdjustTimeDelta() {
    unsafe {
        let mut resetTime: c_int;
        let mut newDelta: c_int;
        let mut deltaDelta: c_int;

        cl.newSnapshots = qfalse;

        // if the current time is WAY off, just correct to the current value
        if (*com_sv_running).integer != 0 {
            resetTime = 100;
        } else {
            resetTime = RESET_TIME;
        }

        newDelta = cl.frame.serverTime - cls.realtime;
        deltaDelta = (newDelta - cl.serverTimeDelta).abs();

        if deltaDelta > RESET_TIME {
            cl.serverTimeDelta = newDelta;
            cl.oldServerTime = cl.frame.serverTime; // FIXME: is this a problem for cgame?
            cl.serverTime = cl.frame.serverTime;
            if (*cl_showTimeDelta).integer != 0 {
                Com_Printf("<RESET> \0".as_ptr() as *const c_char);
            }
        } else if deltaDelta > 100 {
            // fast adjust, cut the difference in half
            if (*cl_showTimeDelta).integer != 0 {
                Com_Printf("<FAST> \0".as_ptr() as *const c_char);
            }
            cl.serverTimeDelta = (cl.serverTimeDelta + newDelta) >> 1;
        } else {
            // slow drift adjust, only move 1 or 2 msec

            // if any of the frames between this and the previous snapshot
            // had to be extrapolated, nudge our sense of time back a little
            // the granularity of +1 / -2 is too high for timescale modified frametimes
            if (*com_timescale).value == 0.0 || (*com_timescale).value == 1.0 {
                if cl.extrapolatedSnapshot != 0 {
                    cl.extrapolatedSnapshot = qfalse;
                    cl.serverTimeDelta -= 2;
                } else {
                    // otherwise, move our sense of time forward to minimize total latency
                    cl.serverTimeDelta += 1;
                }
            }
        }

        if (*cl_showTimeDelta).integer != 0 {
            Com_Printf("%i \0".as_ptr() as *const c_char, cl.serverTimeDelta);
        }
    }
}

// ==================
// CL_FirstSnapshot
// ==================
#[no_mangle]
pub extern "C" fn CL_FirstSnapshot() {
    unsafe {
        RE_RegisterMedia_LevelLoadEnd();

        cls.state = CA_ACTIVE;

        // set the timedelta so we are exactly on this first frame
        cl.serverTimeDelta = cl.frame.serverTime - cls.realtime;
        cl.oldServerTime = cl.frame.serverTime;

        // if this is the first frame of active play,
        // execute the contents of activeAction now
        // this is to allow scripting a timedemo to start right
        // after loading
        if *(*cl_activeAction).string.as_ptr() as c_int != 0 {
            Cbuf_AddText((*cl_activeAction).string.as_ptr() as *const c_char);
            Cvar_Set("activeAction\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char);
        }

        Sys_BeginProfiling();

        #[cfg(feature = "xbox")]
        {
            // turn vsync back on - tearing is ugly
            qglEnable(0x2001); // GL_VSYNC
        }
    }
}

// ==================
// CL_SetCGameTime
// ==================
#[no_mangle]
pub extern "C" fn CL_SetCGameTime() {
    unsafe {
        // getting a valid frame message ends the connection process
        if cls.state != CA_ACTIVE {
            if cls.state != CA_PRIMED {
                return;
            }
            if cl.newSnapshots != 0 {
                cl.newSnapshots = qfalse;
                CL_FirstSnapshot();
            }

            if cls.state != CA_ACTIVE {
                return;
            }
        }

        // if we have gotten to this point, cl.frame is guaranteed to be valid
        if cl.frame.valid == 0 {
            Com_Error(
                ERR_DROP,
                "CL_SetCGameTime: !cl.snap.valid\0".as_ptr() as *const c_char,
            );
        }

        // allow pause in single player
        if (*sv_paused).integer != 0 && (*cl_paused).integer != 0 && (*com_sv_running).integer != 0 {
            // paused
            return;
        }

        if cl.frame.serverTime < cl.oldFrameServerTime {
            Com_Error(
                ERR_DROP,
                "cl.frame.serverTime < cl.oldFrameServerTime\0".as_ptr() as *const c_char,
            );
        }
        cl.oldFrameServerTime = cl.frame.serverTime;

        // get our current view of time

        // cl_timeNudge is a user adjustable cvar that allows more
        // or less latency to be added in the interest of better
        // smoothness or better responsiveness.
        cl.serverTime = cls.realtime + cl.serverTimeDelta - (*cl_timeNudge).integer;

        // guarantee that time will never flow backwards, even if
        // serverTimeDelta made an adjustment or cl_timeNudge was changed
        if cl.serverTime < cl.oldServerTime {
            cl.serverTime = cl.oldServerTime;
        }
        cl.oldServerTime = cl.serverTime;

        // note if we are almost past the latest frame (without timeNudge),
        // so we will try and adjust back a bit when the next snapshot arrives
        if cls.realtime + cl.serverTimeDelta >= cl.frame.serverTime - 5 {
            cl.extrapolatedSnapshot = qtrue;
        }

        // if we have gotten new snapshots, drift serverTimeDelta
        // don't do this every frame, or a period of packet loss would
        // make a huge adjustment
        if cl.newSnapshots != 0 {
            CL_AdjustTimeDelta();
        }
    }
}

extern "C" {
    fn qglEnable(cap: c_int);
}
