// client.h -- primary header for client

use core::ffi::{c_char, c_int, c_short, c_uchar, c_uint, c_void};

// Re-export dependencies from other modules
use crate::code::game::q_shared::*;
use crate::code::qcommon::qcommon::*;
use crate::code::renderer::tr_public::*;
// Stub imports for keys and snd_public which would come from other modules
// use crate::code::client::keys::*;
// use crate::code::client::snd_public::*;
// use crate::code::cgame::cg_public::*;

// snapshots are a view of the server at a given time
#[repr(C)]
pub struct clSnapshot_t {
	pub valid: qboolean,			// cleared if delta parsing was invalid
	pub snapFlags: c_int,		// rate delayed and dropped commands

	pub serverTime: c_int,		// server time the message is valid for (in msec)

	pub messageNum: c_int,		// copied from netchan->incoming_sequence
	pub deltaNum: c_int,		// messageNum the delta is from
	pub ping: c_int,			// time from when cmdNum-1 was sent to time packet was reeceived
	pub areamask: [c_uchar; MAX_MAP_AREA_BYTES],		// portalarea visibility bits

	pub cmdNum: c_int,			// the next cmdNum the server is expecting
	pub ps: playerState_t,						// complete information about the current player at this time

	pub numEntities: c_int,			// all of the entities that need to be presented
	pub parseEntitiesNum: c_int,		// at the time of this snapshot

	pub serverCommandNum: c_int,		// execute all commands up to this before
												// making the snapshot current
}

/*
=============================================================================

the clientActive_t structure is wiped completely at every
new gamestate_t, potentially several times during an established connection

=============================================================================
*/

// the parseEntities array must be large enough to hold PACKET_BACKUP frames of
// entities, so that when a delta compressed message arives from the server
// it can be un-deltad from the original
pub const MAX_PARSE_ENTITIES: usize = 512;

pub extern "C" {
	pub static mut g_console_field_width: c_int;
}

#[repr(C)]
pub struct clientActive_t {
	pub timeoutcount: c_int,

	pub frame: clSnapshot_t,			// latest received from server

	pub serverTime: c_int,
	pub oldServerTime: c_int,		// to prevent time from flowing bakcwards
	pub oldFrameServerTime: c_int,	// to check tournament restarts
	pub serverTimeDelta: c_int,	// cl.serverTime = cls.realtime + cl.serverTimeDelta
									// this value changes as net lag varies
	pub extrapolatedSnapshot: qboolean,	// set if any cgame frame has been forced to extrapolate
									// cleared when CL_AdjustTimeDelta looks at it
	pub newSnapshots: qboolean,		// set on parse, cleared when CL_AdjustTimeDelta looks at it

	pub gameState: gameState_t,			// configstrings
	pub mapname: [c_char; MAX_QPATH],	// extracted from CS_SERVERINFO

	pub parseEntitiesNum: c_int,	// index (not anded off) into cl_parse_entities[]

	pub mouseDx: [c_int; 2], pub mouseDy: [c_int; 2],	// added to by mouse events
	pub mouseIndex: c_int,
	pub joystickAxis: [c_int; MAX_JOYSTICK_AXIS],	// set by joystick events

	pub cgameUserCmdValue: c_int,	// current weapon to add to usercmd_t
	pub cgameSensitivity: f32,

	// cmds[cmdNumber] is the predicted command, [cmdNumber-1] is the last
	// properly generated command
	pub cmds: [usercmd_t; CMD_BACKUP],	// each mesage will send several old cmds
	pub cmdNumber: c_int,			// incremented each frame, because multiple
									// frames may need to be packed into a single packet

	pub packetTime: [c_int; PACKET_BACKUP],	// cls.realtime sent, for calculating pings
	pub packetCmdNumber: [c_int; PACKET_BACKUP],	// cmdNumber when packet was sent

	// the client maintains its own idea of view angles, which are
	// sent to the server each frame.  It is cleared to 0 upon entering each level.
	// the server sends a delta each frame which is added to the locally
	// tracked view angles to account for standing on rotating objects,
	// and teleport direction changes
	pub viewangles: vec3_t,

	// these are just parsed out of the configstrings for convenience
	pub serverId: c_int,

	// non-gameserver infornamtion
	pub cinematictime: c_int,		// cls.realtime for first cinematic frame (FIXME: NO LONGER USED!, but I wasn't sure if I could remove it because of struct sizes assumed elsewhere? -Ste)

	// big stuff at end of structure so most offsets are 15 bits or less
	pub frames: [clSnapshot_t; PACKET_BACKUP],

	pub parseEntities: [entityState_t; MAX_PARSE_ENTITIES],

	//DJC added - making force powers in single player work like those in
	//multiplayer.  This makes hot swapping code more portable.
	pub gcmdSendValue: qboolean,
	pub gcmdValue: c_uchar,
}

pub extern "C" {
	pub static mut cl: clientActive_t;
}

/*
=============================================================================

the clientConnection_t structure is wiped when disconnecting from a server,
either to go to a full screen console, or connect to a different server

A connection can be to either a server through the network layer,
or just a streaming cinematic.

=============================================================================
*/


#[repr(C)]
pub struct clientConnection_t {
	pub lastPacketSentTime: c_int,			// for retransmits
	pub lastPacketTime: c_int,
	pub servername: [c_char; MAX_OSPATH],		// name of server from original connect
	pub serverAddress: netadr_t,
	pub connectTime: c_int,		// for connection retransmits
	pub connectPacketCount: c_int,	// for display on connection dialog

	pub challenge: c_int,			// from the server to use for connecting

	pub reliableSequence: c_int,
	pub reliableAcknowledge: c_int,
	pub reliableCommands: [*mut c_char; MAX_RELIABLE_COMMANDS],

	// reliable messages received from server
	pub serverCommandSequence: c_int,
	pub serverCommands: [*mut c_char; MAX_RELIABLE_COMMANDS],

	// big stuff at end of structure so most offsets are 15 bits or less
	pub netchan: netchan_t,
}

pub extern "C" {
	pub static mut clc: clientConnection_t;
}

/*
==================================================================

the clientStatic_t structure is never wiped, and is used even when
no client connection is active at all

==================================================================
*/

#[repr(C)]
pub enum exitTo_t {
	EXIT_CONSOLE,
	EXIT_ARENAS,
	EXIT_SERVERS,
	EXIT_LAUNCH,			// quit all the way out of the game on disconnect
}

#[cfg(feature = "xbox")]
pub const MAX_LOCAL_SERVERS: usize = 1;
#[cfg(feature = "xbox")]
pub const MAX_GLOBAL_SERVERS: usize = 1;
#[cfg(feature = "xbox")]
pub const MAX_PINGREQUESTS: usize = 1;

#[cfg(not(feature = "xbox"))]
pub const MAX_LOCAL_SERVERS: usize = 16;
#[cfg(not(feature = "xbox"))]
pub const MAX_GLOBAL_SERVERS: usize = 256;
#[cfg(not(feature = "xbox"))]
pub const MAX_PINGREQUESTS: usize = 16;

#[repr(C)]
pub struct ping_t {
	pub adr: netadr_t,
	pub start: c_int,
	pub time: c_int,
}

#[repr(C)]
pub struct serverInfoResponse_t {
	pub netadr: netadr_t,
	pub info: [c_char; MAX_INFO_STRING],
}

#[repr(C)]
pub struct getserversResponse_t {
	pub netadr: netadr_t,
	pub info: [c_char; MAX_INFO_STRING],
}

#[repr(C)]
pub struct clientStatic_t {
	pub state: connstate_t,				// connection status
	pub keyCatchers: c_int,		// bit flags

	pub servername: [c_char; MAX_OSPATH],		// name of server from original connect (used by reconnect)

	// when the server clears the hunk, all of these must be restarted
	pub rendererStarted: qboolean,
	pub soundStarted: qboolean,
	pub soundRegistered: qboolean,
	pub uiStarted: qboolean,
	pub cgameStarted: qboolean,
	#[cfg(feature = "immersion")]
	pub forceStarted: qboolean,

	pub framecount: c_int,
	pub frametime: c_int,			// msec since last frame
	pub frametimeFraction: f32,	// fraction of a msec since last frame

	pub realtime: c_int,			// ignores pause
	pub realtimeFraction: f32,	// fraction of a msec accumulated
	pub realFrametime: c_int,		// ignoring pause, so console always works

	// update server info
	pub updateInfoString: [c_char; MAX_INFO_STRING],

	// rendering info
	pub glconfig: glconfig_t,
	pub charSetShader: qhandle_t,
	pub whiteShader: qhandle_t,
	pub consoleShader: qhandle_t,

	#[cfg(feature = "xbox")]
	pub mainGamepad: c_short,
}

#[cfg(feature = "xbox")]
pub const CON_TEXTSIZE: usize = 256;
#[cfg(not(target_os = "xbox"))]
pub const CON_TEXTSIZE: usize = 32768;
pub const NUM_CON_TIMES: usize = 4;

#[repr(C)]
pub struct console_t {
	pub initialized: qboolean,

	pub text: [c_short; CON_TEXTSIZE],
	pub current: c_int,		// line where next message will be printed
	pub x: c_int,				// offset in current line for next print
	pub display: c_int,		// bottom of console displays this line

	pub linewidth: c_int,		// characters across screen
	pub totallines: c_int,		// total lines in console scrollback

	pub xadjust: f32,		// for wide aspect screens
	pub yadjust: f32,

	pub displayFrac: f32,	// aproaches finalFrac at scr_conspeed
	pub finalFrac: f32,		// 0.0 to 1.0 lines of console to display

	pub vislines: c_int,		// in scanlines

	pub times: [c_int; NUM_CON_TIMES],	// cls.realtime time the line was generated
								// for transparent notify lines
	pub color: vec4_t,
}

pub extern "C" {
	pub static mut cls: clientStatic_t;
}

//=============================================================================

pub extern "C" {
	pub static re: refexport_t;		// interface to refresh .dll
}


//
// cvars
//
pub extern "C" {
	pub static mut cl_nodelta: *mut cvar_t;
	pub static mut cl_debugMove: *mut cvar_t;
	pub static mut cl_noprint: *mut cvar_t;
	pub static mut cl_timegraph: *mut cvar_t;
	pub static mut cl_maxpackets: *mut cvar_t;
	pub static mut cl_packetdup: *mut cvar_t;
	pub static mut cl_shownet: *mut cvar_t;
	pub static mut cl_timeNudge: *mut cvar_t;
	pub static mut cl_showTimeDelta: *mut cvar_t;

	pub static mut cl_yawspeed: *mut cvar_t;
	pub static mut cl_pitchspeed: *mut cvar_t;
	pub static mut cl_run: *mut cvar_t;
	pub static mut cl_anglespeedkey: *mut cvar_t;

	pub static mut cl_sensitivity: *mut cvar_t;
	pub static mut cl_freelook: *mut cvar_t;

	pub static mut cl_mouseAccel: *mut cvar_t;
	pub static mut cl_showMouseRate: *mut cvar_t;

	pub static mut cl_ingameVideo: *mut cvar_t;
	pub static mut cl_VideoQuality: *mut cvar_t;
	pub static mut cl_VidFadeUp: *mut cvar_t;
	pub static mut cl_VidFadeDown: *mut cvar_t;

	pub static mut m_pitch: *mut cvar_t;
	pub static mut m_yaw: *mut cvar_t;
	pub static mut m_forward: *mut cvar_t;
	pub static mut m_side: *mut cvar_t;
	pub static mut m_filter: *mut cvar_t;

	pub static mut cl_activeAction: *mut cvar_t;

	pub static mut cl_thumbStickMode: *mut cvar_t;
}

//=================================================

//
// cl_main
//

pub extern "C" {
	pub fn CL_Init ();

	pub fn CL_AddReliableCommand( cmd: *const c_char );

	pub fn CL_Disconnect_f ();
	pub fn CL_GetChallengePacket ();
	pub fn CL_Vid_Restart_f( );
	pub fn CL_Snd_Restart_f ();

	pub fn CL_NextDemo( );

	pub fn CL_GetPing( n: c_int, adrstr: *mut c_char, pingtime: *mut c_int );
	pub fn CL_ClearPing( n: c_int );
	pub fn CL_GetPingQueueCount( ) -> c_int;
}

//
// cl_input
//
#[repr(C)]
pub struct kbutton_t {
	pub down: [c_int; 2],		// key nums holding it down
	pub downtime: c_uint,		// msec timestamp
	pub msec: c_uint,			// msec down this frame if both a down and up happened
	pub active: qboolean,			// current state
	pub wasPressed: qboolean,		// set when down, not cleared when up
}

pub extern "C" {
	pub static mut in_mlook: kbutton_t;
	pub static mut in_klook: kbutton_t;
	pub static mut in_strafe: kbutton_t;
	pub static mut in_speed: kbutton_t;
}

pub extern "C" {
	pub fn CL_InitInput ();
	pub fn CL_SendCmd ();
	pub fn CL_ClearState ();
	pub fn CL_ReadPackets ();
	pub fn CL_UpdateHotSwap();
	pub fn CL_ExtendSelectTime() -> bool;

	pub fn CL_WritePacket( );
	pub fn IN_CenterView ();

	pub fn CL_KeyState (key: *mut kbutton_t) -> f32;
	pub fn Key_KeynumToString( keynum: c_int/*bTranslate: qboolean*/ ) -> *const c_char; //note: translate is only called for menu display not configs
}

//
// cl_parse.c
//
pub extern "C" {
	pub fn CL_SystemInfoChanged( );
	pub fn CL_ParseServerMessage( msg: *mut msg_t );
}

//====================================================================

pub extern "C" {
	pub fn VID_MenuInit( );
	pub fn VID_MenuDraw( );
	pub fn VID_MenuKey( keynum: c_int ) -> *const c_char;
	pub fn VID_Printf (print_level: c_int, fmt: *const c_char, ...);
}


//
// console
//
pub extern "C" {
	pub fn Con_DrawCharacter (cx: c_int, line: c_int, num: c_int);

	pub fn Con_CheckResize ();
	pub fn Con_Init ();
	pub fn Con_Clear_f ();
	pub fn Con_ToggleConsole_f ();
	pub fn Con_DrawNotify ();
	pub fn Con_ClearNotify ();
	pub fn Con_RunConsole ();
	pub fn Con_DrawConsole ();
	pub fn Con_PageUp( );
	pub fn Con_PageDown( );
	pub fn Con_Top( );
	pub fn Con_Bottom( );
	pub fn Con_Close( );
}


//
// cl_scrn.c
//
pub extern "C" {
	pub fn SCR_Init ();
	pub fn SCR_UpdateScreen ();

	pub fn SCR_DebugGraph (value: f32, color: c_int);

	pub fn SCR_GetBigStringWidth( str: *const c_char ) -> c_int;	// returns in virtual 640x480 coordinates

	pub fn SCR_FillRect( x: f32, y: f32, width: f32, height: f32,
					 color: *const f32 );
	pub fn SCR_DrawPic( x: f32, y: f32, width: f32, height: f32, hShader: qhandle_t );
	pub fn SCR_DrawNamedPic( x: f32, y: f32, width: f32, height: f32, picname: *const c_char );

	pub fn SCR_DrawBigString( x: c_int, y: c_int, s: *const c_char, alpha: f32 );			// draws a string with embedded color control characters with fade
	pub fn SCR_DrawBigStringColor( x: c_int, y: c_int, s: *const c_char, color: vec4_t );	// ignores embedded color control characters
	pub fn SCR_DrawSmallString( x: c_int, y: c_int, s: *const c_char, alpha: f32 );			// draws a string with embedded color control characters with fade
	pub fn SCR_DrawSmallStringColor( x: c_int, y: c_int, s: *const c_char, color: vec4_t );	// ignores embedded color control characters
	pub fn SCR_DrawBigChar( x: c_int, y: c_int, ch: c_int );
	pub fn SCR_DrawSmallChar( x: c_int, y: c_int, ch: c_int );

	#[cfg(feature = "xbox")]
	pub fn SCR_PrecacheScreenshot();
}


//
// cl_cin.c
//
pub extern "C" {
	pub fn CL_PlayCinematic_f( );
	pub fn CL_PlayInGameCinematic_f();
	pub fn CL_CheckPendingCinematic() -> qboolean;
	pub fn CL_IsRunningInGameCinematic() -> qboolean;
	pub fn CL_InGameCinematicOnStandBy() -> qboolean;
	pub fn SCR_DrawCinematic ();
	pub fn SCR_RunCinematic ();
	pub fn SCR_StopCinematic( bAllowRefusal: qboolean ); // = qfalse );

	pub fn CIN_PlayCinematic( arg0: *const c_char, xpos: c_int, ypos: c_int, width: c_int, height: c_int, bits: c_int, psAudioFile: *const c_char /* = NULL */);
	pub fn CIN_StopCinematic(handle: c_int) -> e_status;
	pub fn CIN_RunCinematic (handle: c_int) -> e_status;
	pub fn CIN_DrawCinematic (handle: c_int);
	pub fn CIN_SetExtents (handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int);
	pub fn CIN_SetLooping (handle: c_int, loop: qboolean);
	pub fn CIN_UploadCinematic(handle: c_int);
	pub fn CIN_CloseAllVideos();

	#[cfg(feature = "xbox")]
	pub fn CIN_Init();
	#[cfg(feature = "xbox")]
	pub fn CIN_PlayAllFrames( arg: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, systemBits: c_int, keyBreakAllowed: bool ) -> bool;
}


//
// cl_cgame.c
//
pub extern "C" {
	pub fn CL_InitCGame( );
	pub fn CL_ShutdownCGame( );
	pub fn CL_GameCommand( ) -> qboolean;
	pub fn CL_CGameRendering( stereo: stereoFrame_t );
	pub fn CL_SetCGameTime( );
	pub fn CL_FirstSnapshot( );
}


//
// cl_ui.c
//
pub extern "C" {
	pub fn CL_InitUI( );
	pub fn CL_ShutdownUI( );
	pub fn CL_GenericMenu_f();
	pub fn CL_DataPad_f();
	pub fn CL_EndScreenDissolve_f();
}
