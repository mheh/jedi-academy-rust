// client.h -- primary header for client

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint, c_uchar, c_short, c_ushort};

// Forward declarations of external types from ported headers
// These types are defined in other modules but referenced here
// Types from q_shared.h:
extern "C" {
    pub type vec3_t;
    pub type vec4_t;
    pub type qboolean;
    pub type e_status;
}

// Types from qcommon.h:
extern "C" {
    pub type netadr_t;
    pub type netchan_t;
    pub type msg_t;
    pub type vm_t;
    pub type cvar_t;
    pub type connstate_t;
    pub type fileHandle_t;
}

// Types from tr_public.h:
extern "C" {
    pub type glconfig_t;
    pub type qhandle_t;
    pub type refexport_t;
    pub type stereoFrame_t;
}

// Types from game/bg_public.h, game/g_public.h:
extern "C" {
    pub type playerState_t;
    pub type gameState_t;
    pub type entityState_t;
    pub type usercmd_t;
}

// Constants from q_shared.h and qcommon.h
// Rust requires compile-time constants for array lengths; using Q3 standard values
pub const MAX_MAP_AREA_BYTES: usize = 32;
pub const MAX_QPATH: usize = 64;
pub const MAX_STRING_TOKENS: usize = 1024;
pub const MAX_RELIABLE_COMMANDS: usize = 128;
pub const MAX_STRING_CHARS: usize = 256;
pub const MAX_OSPATH: usize = 256;
pub const MAX_INFO_STRING: usize = 1024;
pub const MAX_JOYSTICK_AXIS: usize = 16;
pub const CMD_BACKUP: usize = 64;
pub const PACKET_BACKUP: usize = 32;
pub const MAX_GENTITIES: usize = 2048;
pub const MAX_OTHER_SERVERS: usize = 32;
pub const MAX_GLOBAL_SERVERS: usize = 256;
pub const MAX_TOKEN_CHARS: usize = 256;
pub const MAX_NAME_LENGTH: usize = 64;

// Platform-specific constants
#[cfg(target_pointer_width = "32")]
pub const MAX_PARSE_ENTITIES: usize = 1024; // _XBOX
#[cfg(not(target_pointer_width = "32"))]
pub const MAX_PARSE_ENTITIES: usize = 2048;

#[cfg(target_pointer_width = "32")]
pub const CON_TEXTSIZE: usize = 256; // _XBOX
#[cfg(not(target_pointer_width = "32"))]
pub const CON_TEXTSIZE: usize = 32768;

pub const MAX_HEIGHTMAP_SIZE: usize = 16000;
pub const MAX_AUTOMAP_SYMBOLS: usize = 512;
pub const NUM_CON_TIMES: usize = 4;

pub const RETRANSMIT_TIMEOUT: c_int = 3000; // time between connection packet retransmits

// Wind
extern "C" {
    pub static mut cl_windVec: vec3_t;
}

// snapshots are a view of the server at a given time
#[repr(C)]
pub struct clSnapshot_t {
    pub valid: qboolean,            // cleared if delta parsing was invalid
    pub snapFlags: c_int,           // rate delayed and dropped commands

    pub serverTime: c_int,          // server time the message is valid for (in msec)

    pub messageNum: c_int,          // copied from netchan->incoming_sequence
    pub deltaNum: c_int,            // messageNum the delta is from
    pub ping: c_int,                // time from when cmdNum-1 was sent to time packet was reeceived
    pub areamask: [c_uchar; MAX_MAP_AREA_BYTES], // portalarea visibility bits

    pub cmdNum: c_int,              // the next cmdNum the server is expecting
    pub ps: playerState_t,          // complete information about the current player at this time
    pub vps: playerState_t,         // vehicle I'm riding's playerstate (if applicable) -rww

    pub numEntities: c_int,         // all of the entities that need to be presented
    pub parseEntitiesNum: c_int,    // at the time of this snapshot

    pub serverCommandNum: c_int,    // execute all commands up to this before
                                    // making the snapshot current
}

//
// =============================================================================
//
// the clientActive_t structure is wiped completely at every
// new gamestate_t, potentially several times during an established connection
//
// =============================================================================
//

#[repr(C)]
pub struct outPacket_t {
    pub p_cmdNumber: c_int,     // cl.cmdNumber when packet was sent
    pub p_serverTime: c_int,    // usercmd->serverTime when packet was sent
    pub p_realtime: c_int,      // cls.realtime when packet was sent
}

// the parseEntities array must be large enough to hold PACKET_BACKUP frames of
// entities, so that when a delta compressed message arives from the server
// it can be un-deltad from the original

extern "C" {
    pub static g_console_field_width: c_int;
}

#[repr(C)]
pub struct clientActive_t {
    pub timeoutcount: c_int,        // it requres several frames in a timeout condition
                                    // to disconnect, preventing debugging breaks from
                                    // causing immediate disconnects on continue
    pub snap: clSnapshot_t,         // latest received from server

    pub serverTime: c_int,          // may be paused during play
    pub oldServerTime: c_int,       // to prevent time from flowing bakcwards
    pub oldFrameServerTime: c_int,  // to check tournament restarts
    pub serverTimeDelta: c_int,     // cl.serverTime = cls.realtime + cl.serverTimeDelta
                                    // this value changes as net lag varies
    pub extrapolatedSnapshot: qboolean, // set if any cgame frame has been forced to extrapolate
                                    // cleared when CL_AdjustTimeDelta looks at it
    pub newSnapshots: qboolean,     // set on parse of any valid packet

    pub gameState: gameState_t,     // configstrings
    pub mapname: [c_char; MAX_QPATH], // extracted from CS_SERVERINFO

    pub parseEntitiesNum: c_int,    // index (not anded off) into cl_parse_entities[]

    pub mouseDx: [c_int; 2],        // added to by mouse events
    pub mouseDy: [c_int; 2],        // added to by mouse events
    pub mouseIndex: c_int,
    pub joystickAxis: [c_int; MAX_JOYSTICK_AXIS], // set by joystick events

    // cgame communicates a few values to the client system
    pub cgameUserCmdValue: c_int,   // current weapon to add to usercmd_t
    pub cgameViewAngleForce: vec3_t,
    pub cgameViewAngleForceTime: c_int,
    pub cgameSensitivity: f32,

    pub cgameForceSelection: c_int,
    pub cgameInvenSelection: c_int,

    pub gcmdSendValue: qboolean,
    pub gcmdSentValue: qboolean,
    pub gcmdValue: c_uchar,

    // cmds[cmdNumber] is the predicted command, [cmdNumber-1] is the last
    // properly generated command
    pub cmds: [usercmd_t; CMD_BACKUP], // each mesage will send several old cmds
    pub cmdNumber: c_int,       // incremented each frame, because multiple
                                // frames may need to be packed into a single packet

    pub outPackets: [outPacket_t; PACKET_BACKUP], // information about each packet we have sent out

    // the client maintains its own idea of view angles, which are
    // sent to the server each frame.  It is cleared to 0 upon entering each level.
    // the server sends a delta each frame which is added to the locally
    // tracked view angles to account for standing on rotating objects,
    // and teleport direction changes
    pub viewangles: vec3_t,

    pub serverId: c_int,        // included in each client message so the server
                                // can tell if it is for a prior map_restart
    // big stuff at end of structure so most offsets are 15 bits or less
    pub snapshots: [clSnapshot_t; PACKET_BACKUP],

    pub entityBaselines: [entityState_t; MAX_GENTITIES], // for delta compression when not in previous frame

    pub parseEntities: [entityState_t; MAX_PARSE_ENTITIES],

    pub mSharedMemory: *mut c_char,
}

extern "C" {
    pub static mut cl: clientActive_t;
}

#[repr(C)]
pub struct rmAutomapSymbol_t {
    pub mType: c_int,
    pub mSide: c_int,
    pub mOrigin: vec3_t,
}

//
// =============================================================================
//
// the clientConnection_t structure is wiped when disconnecting from a server,
// either to go to a full screen console, play a demo, or connect to a different server
//
// A connection can be to either a server through the network layer or a
// demo through a file.
//
// =============================================================================
//

#[repr(C)]
pub struct clientConnection_t {
    pub clientNum: c_int,
    pub lastPacketSentTime: c_int,  // for retransmits during connection
    pub lastPacketTime: c_int,      // for timeouts

    pub serverAddress: netadr_t,
    pub connectTime: c_int,         // for connection retransmits
    pub connectPacketCount: c_int,  // for display on connection dialog
    pub serverMessage: [c_char; MAX_STRING_TOKENS], // for display on connection dialog

    pub challenge: c_int,           // from the server to use for connecting
    pub checksumFeed: c_int,        // from the server for checksum calculations

    // these are our reliable messages that go to the server
    pub reliableSequence: c_int,
    pub reliableAcknowledge: c_int, // the last one the server has executed
    pub reliableCommands: [[c_char; MAX_STRING_CHARS]; MAX_RELIABLE_COMMANDS],

    // server message (unreliable) and command (reliable) sequence
    // numbers are NOT cleared at level changes, but continue to
    // increase as long as the connection is valid

    // message sequence is used by both the network layer and the
    // delta compression layer
    pub serverMessageSequence: c_int,

    // reliable messages received from server
    pub serverCommandSequence: c_int,
    pub lastExecutedServerCommand: c_int, // last server command grabbed or executed with CL_GetServerCommand
    pub serverCommands: [[c_char; MAX_STRING_CHARS]; MAX_RELIABLE_COMMANDS],

    // Platform-specific: download/demo fields
    // #ifndef _XBOX	// No downloading or demos on Xbox
    // file transfer from server
    pub download: fileHandle_t,
    pub downloadTempName: [c_char; MAX_OSPATH],
    pub downloadName: [c_char; MAX_OSPATH],
    pub downloadNumber: c_int,
    pub downloadBlock: c_int,       // block we are waiting for
    pub downloadCount: c_int,       // how many bytes we got
    pub downloadSize: c_int,        // how many bytes we got
    pub downloadList: [c_char; MAX_INFO_STRING], // list of paks we need to download
    pub downloadRestart: qboolean,  // if true, we need to do another FS_Restart because we downloaded a pak

    // demo information
    pub demoName: [c_char; MAX_QPATH],
    pub spDemoRecording: qboolean,
    pub demorecording: qboolean,
    pub demoplaying: qboolean,
    pub demowaiting: qboolean,      // don't record until a non-delta message is received
    pub firstDemoFrameSkipped: qboolean,
    pub demofile: fileHandle_t,

    pub timeDemoFrames: c_int,      // counter of rendered frames
    pub timeDemoStart: c_int,       // cls.realtime before first frame
    pub timeDemoBaseTime: c_int,    // each frame will be at this time + frameNum * 50
    // #endif

    // big stuff at end of structure so most offsets are 15 bits or less
    pub netchan: netchan_t,

    // rwwRMG - added:
    pub rmgSeed: c_int,
    pub rmgHeightMapSize: c_int,
    pub rmgHeightMap: [c_uchar; MAX_HEIGHTMAP_SIZE],
    pub rmgFlattenMap: [c_uchar; MAX_HEIGHTMAP_SIZE],
    pub rmgAutomapSymbols: [rmAutomapSymbol_t; MAX_AUTOMAP_SYMBOLS],
    pub rmgAutomapSymbolCount: c_int,
}

extern "C" {
    pub static mut clc: clientConnection_t;
}

//
// ==================================================================
//
// the clientStatic_t structure is never wiped, and is used even when
// no client connection is active at all
//
// ==================================================================
//

#[repr(C)]
pub struct ping_t {
    pub adr: netadr_t,
    pub start: c_int,
    pub time: c_int,
    pub info: [c_char; MAX_INFO_STRING],
    // #ifdef _XBOX
    // pub xnaddr: XNADDR,
    // #endif
}

#[repr(C)]
pub struct serverInfo_t {
    pub adr: netadr_t,
    pub hostName: [c_char; MAX_NAME_LENGTH],
    pub mapName: [c_char; MAX_NAME_LENGTH],
    pub game: [c_char; MAX_NAME_LENGTH],
    // #ifndef _XBOX
    pub netType: c_int,
    // #endif
    pub gameType: c_int,
    pub clients: c_int,
    pub maxClients: c_int,
    // #ifndef _XBOX
    pub minPing: c_int,
    pub maxPing: c_int,
    // #endif
    pub ping: c_int,
    pub visible: qboolean,
    // int			allowAnonymous;
    // #ifndef _XBOX
    pub needPassword: qboolean,
    pub trueJedi: c_int,
    pub weaponDisable: c_int,
    pub forceDisable: c_int,
    // #endif
    // qboolean	pure;
    // #ifdef _XBOX
    // pub saberOnly: qboolean,       // Not the same as weaponDisable!
    // pub SessionID: XNKID,
    // pub KeyExchangeKey: XNKEY,
    // pub HostAddress: XNADDR,
    // #endif
}

#[repr(C)]
pub struct serverAddress_t {
    pub ip: [c_uchar; 4],
    pub port: c_ushort,
}

#[repr(C)]
pub struct clientStatic_t {
    pub state: connstate_t,         // connection status
    pub keyCatchers: c_int,         // bit flags

    pub servername: [c_char; MAX_OSPATH], // name of server from original connect (used by reconnect)

    // when the server clears the hunk, all of these must be restarted
    pub rendererStarted: qboolean,
    pub soundStarted: qboolean,
    pub soundRegistered: qboolean,
    pub uiStarted: qboolean,
    pub cgameStarted: qboolean,

    pub framecount: c_int,
    pub frametime: c_int,           // msec since last frame

    pub realtime: c_int,            // ignores pause
    pub realFrametime: c_int,       // ignoring pause, so console always works

    pub numlocalservers: c_int,
    pub localServers: [serverInfo_t; MAX_OTHER_SERVERS],

    pub numglobalservers: c_int,
    pub globalServers: [serverInfo_t; MAX_GLOBAL_SERVERS],
    // additional global servers
    pub numGlobalServerAddresses: c_int,
    pub globalServerAddresses: [serverAddress_t; MAX_GLOBAL_SERVERS],

    pub numfavoriteservers: c_int,
    pub favoriteServers: [serverInfo_t; MAX_OTHER_SERVERS],

    pub nummplayerservers: c_int,
    pub mplayerServers: [serverInfo_t; MAX_OTHER_SERVERS],

    pub pingUpdateSource: c_int,    // source currently pinging or updating

    pub masterNum: c_int,

    // update server info
    pub updateServer: netadr_t,
    pub updateChallenge: [c_char; MAX_TOKEN_CHARS],
    pub updateInfoString: [c_char; MAX_INFO_STRING],

    pub authorizeServer: netadr_t,

    // rendering info
    pub glconfig: glconfig_t,
    pub charSetShader: qhandle_t,
    pub whiteShader: qhandle_t,
    pub consoleShader: qhandle_t,

    // #ifdef _XBOX
    // pub mainGamepad: c_short,
    // #endif
}

#[repr(C)]
pub struct console_t {
    pub initialized: qboolean,

    pub text: [c_short; CON_TEXTSIZE],
    pub current: c_int,         // line where next message will be printed
    pub x: c_int,               // offset in current line for next print
    pub display: c_int,         // bottom of console displays this line

    pub linewidth: c_int,       // characters across screen
    pub totallines: c_int,      // total lines in console scrollback

    pub xadjust: f32,           // for wide aspect screens
    pub yadjust: f32,           // for wide aspect screens

    pub displayFrac: f32,       // aproaches finalFrac at scr_conspeed
    pub finalFrac: f32,         // 0.0 to 1.0 lines of console to display

    pub vislines: c_int,        // in scanlines

    pub times: [c_int; NUM_CON_TIMES], // cls.realtime time the line was generated
                                // for transparent notify lines
    pub color: vec4_t,
}

extern "C" {
    pub static mut cls: clientStatic_t;
}

//=============================================================================

extern "C" {
    pub static mut cgvm: *mut vm_t;   // interface to cgame dll or vm
    pub static mut uivm: *mut vm_t;   // interface to ui dll or vm
    pub static mut re: refexport_t;   // interface to refresh .dll
}

//
// cvars
//
extern "C" {
    pub static mut cl_nodelta: *mut cvar_t;
    pub static mut cl_debugMove: *mut cvar_t;
    pub static mut cl_noprint: *mut cvar_t;
    pub static mut cl_timegraph: *mut cvar_t;
    pub static mut cl_maxpackets: *mut cvar_t;
    pub static mut cl_packetdup: *mut cvar_t;
    pub static mut cl_shownet: *mut cvar_t;
    pub static mut cl_showSend: *mut cvar_t;
    pub static mut cl_timeNudge: *mut cvar_t;
    pub static mut cl_showTimeDelta: *mut cvar_t;
    pub static mut cl_freezeDemo: *mut cvar_t;

    pub static mut cl_yawspeed: *mut cvar_t;
    pub static mut cl_pitchspeed: *mut cvar_t;
    pub static mut cl_run: *mut cvar_t;
    pub static mut cl_anglespeedkey: *mut cvar_t;

    pub static mut cl_sensitivity: *mut cvar_t;
    pub static mut cl_freelook: *mut cvar_t;

    pub static mut cl_mouseAccel: *mut cvar_t;
    pub static mut cl_showMouseRate: *mut cvar_t;

    pub static mut m_pitchVeh: *mut cvar_t;
    pub static mut m_pitch: *mut cvar_t;
    pub static mut m_yaw: *mut cvar_t;
    pub static mut m_forward: *mut cvar_t;
    pub static mut m_side: *mut cvar_t;
    pub static mut m_filter: *mut cvar_t;

    pub static mut cl_timedemo: *mut cvar_t;

    pub static mut cl_activeAction: *mut cvar_t;

    // #ifndef _XBOX
    pub static mut cl_allowDownload: *mut cvar_t;
    pub static mut cl_allowAltEnter: *mut cvar_t;
    // #endif
    pub static mut cl_conXOffset: *mut cvar_t;
    pub static mut cl_inGameVideo: *mut cvar_t;
}

//=================================================

//
// cl_main
//
extern "C" {
    pub fn CL_Init();
    pub fn CL_FlushMemory();
    pub fn CL_ShutdownAll();
    pub fn CL_AddReliableCommand(cmd: *const c_char);

    pub fn CL_StartHunkUsers();

    pub fn CL_Disconnect_f();
    pub fn CL_GetChallengePacket();
    pub fn CL_Vid_Restart_f();
    pub fn CL_Snd_Restart_f();
    pub fn CL_StartDemoLoop();
    pub fn CL_NextDemo();
    pub fn CL_ReadDemoMessage();

    pub fn CL_InitDownloads();
    pub fn CL_NextDownload();

    pub fn CL_GetPing(n: c_int, buf: *mut c_char, buflen: c_int, pingtime: *mut c_int);
    pub fn CL_GetPingInfo(n: c_int, buf: *mut c_char, buflen: c_int);
    pub fn CL_ClearPing(n: c_int);
    pub fn CL_GetPingQueueCount() -> c_int;

    pub fn CL_ShutdownRef();
    pub fn CL_InitRef();

    #[cfg(feature = "USE_CD_KEY")]
    pub fn CL_CDKeyValidate(key: *const c_char, checksum: *const c_char) -> qboolean;

    pub fn CL_ServerStatus(serverAddress: *mut c_char, serverStatusString: *mut c_char, maxLen: c_int) -> c_int;
}

//
// cl_input
//
#[repr(C)]
pub struct kbutton_t {
    pub down: [c_int; 2],       // key nums holding it down
    pub downtime: c_uint,       // msec timestamp
    pub msec: c_uint,           // msec down this frame if both a down and up happened
    pub active: qboolean,       // current state
    pub wasPressed: qboolean,   // set when down, not cleared when up
}

extern "C" {
    pub static mut in_mlook: kbutton_t;
    pub static mut in_klook: kbutton_t;
    pub static mut in_strafe: kbutton_t;
    pub static mut in_speed: kbutton_t;
}

extern "C" {
    pub fn CL_InitInput();
    pub fn CL_SendCmd();
    pub fn CL_ClearState();
    pub fn CL_ReadPackets();

    pub fn CL_WritePacket();
    pub fn IN_CenterView();

    pub fn CL_VerifyCode();

    pub fn CL_KeyState(key: *mut kbutton_t) -> f32;
    pub fn Key_KeynumToString(keynum: c_int) -> *const c_char; // note: translate is only called for menu display not configs
}

//
// cl_parse.c
//
extern "C" {
    pub static cl_connectedToPureServer: c_int;
    pub static cl_connectedGAME: c_int;
    pub static cl_connectedCGAME: c_int;
    pub static cl_connectedUI: c_int;

    pub fn CL_SystemInfoChanged();
    pub fn CL_ParseServerMessage(msg: *mut msg_t);
}

//====================================================================

extern "C" {
    pub fn CL_ServerInfoPacket(from: netadr_t, msg: *mut msg_t);
    pub fn CL_LocalServers_f();
    // #ifndef _XBOX
    pub fn CL_GlobalServers_f();
    pub fn CL_FavoriteServers_f();
    // #endif
    pub fn CL_Ping_f();
    pub fn CL_UpdateVisiblePings_f(source: c_int) -> qboolean;
}

//
// console
//
extern "C" {
    pub fn Con_DrawCharacter(cx: c_int, line: c_int, num: c_int);

    pub fn Con_CheckResize();
    pub fn Con_Init();
    pub fn Con_Clear_f();
    pub fn Con_ToggleConsole_f();
    pub fn Con_DrawNotify();
    pub fn Con_ClearNotify();
    pub fn Con_RunConsole();
    pub fn Con_DrawConsole();
    pub fn Con_PageUp();
    pub fn Con_PageDown();
    pub fn Con_Top();
    pub fn Con_Bottom();
    pub fn Con_Close();
}

//
// cl_scrn.c
//
extern "C" {
    pub fn SCR_Init();
    pub fn SCR_UpdateScreen();

    pub fn SCR_DebugGraph(value: f32, color: c_int);

    pub fn SCR_GetBigStringWidth(str: *const c_char) -> c_int; // returns in virtual 640x480 coordinates

    pub fn SCR_FillRect(x: f32, y: f32, width: f32, height: f32, color: *const f32);
    pub fn SCR_DrawPic(x: f32, y: f32, width: f32, height: f32, hShader: qhandle_t);
    pub fn SCR_DrawNamedPic(x: f32, y: f32, width: f32, height: f32, picname: *const c_char);

    pub fn SCR_DrawBigString(x: c_int, y: c_int, s: *const c_char, alpha: f32); // draws a string with embedded color control characters with fade
    pub fn SCR_DrawBigStringColor(x: c_int, y: c_int, s: *const c_char, color: vec4_t); // ignores embedded color control characters
    pub fn SCR_DrawSmallStringExt(x: c_int, y: c_int, string: *const c_char, setColor: *mut f32, forceColor: qboolean);
    pub fn SCR_DrawSmallChar(x: c_int, y: c_int, ch: c_int);
}

//
// cl_cin.c
//
extern "C" {
    pub fn CL_PlayCinematic_f();
    pub fn SCR_DrawCinematic();
    pub fn SCR_RunCinematic();
    pub fn SCR_StopCinematic();
    pub fn CIN_PlayCinematic(arg0: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int, bits: c_int) -> c_int;
    pub fn CIN_StopCinematic(handle: c_int) -> e_status;
    pub fn CIN_RunCinematic(handle: c_int) -> e_status;
    pub fn CIN_DrawCinematic(handle: c_int);
    pub fn CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int);
    pub fn CIN_SetLooping(handle: c_int, loop: qboolean);
    pub fn CIN_UploadCinematic(handle: c_int);
    pub fn CIN_CloseAllVideos();

    pub fn CL_UpdateHotSwap();

    // #ifdef _XBOX
    // pub fn CIN_Init();
    // pub fn CIN_PlayAllFrames(arg: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, systemBits: c_int, keyBreakAllowed: bool) -> bool;
    // #endif
}

//
// cl_cgame.c
//
extern "C" {
    pub fn CL_InitCGame();
    pub fn CL_ShutdownCGame();
    pub fn CL_GameCommand() -> qboolean;
    pub fn CL_CGameRendering(stereo: stereoFrame_t);
    pub fn CL_SetCGameTime();
    pub fn CL_FirstSnapshot();
    pub fn CL_ShaderStateChanged();
}

//
// cl_ui.c
//
extern "C" {
    pub fn CL_InitUI();
    pub fn CL_ShutdownUI();
    pub fn Key_GetCatcher() -> c_int;
    pub fn Key_SetCatcher(catcher: c_int);
    pub fn LAN_LoadCachedServers();
    pub fn LAN_SaveServersToCache();
}

//
// cl_net_chan.c
//
extern "C" {
    pub fn CL_Netchan_Transmit(chan: *mut netchan_t, msg: *mut msg_t);
    pub fn CL_Netchan_TransmitNextFragment(chan: *mut netchan_t);
    pub fn CL_Netchan_Process(chan: *mut netchan_t, msg: *mut msg_t) -> qboolean;
}
