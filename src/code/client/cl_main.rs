// cl_main.rs  -- client main loop

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Note: These includes represent the original C header dependencies.
// Actual implementations would need to be ported separately or linked via FFI.
// For now, we declare the externs and types needed by this module.

const RETRANSMIT_TIMEOUT: c_int = 3000; // time between connection packet retransmits

// Extern cvar declarations
extern "C" {
    pub static mut cl_nodelta: *mut cvar_t;
    pub static mut cl_debugMove: *mut cvar_t;
    pub static mut cl_noprint: *mut cvar_t;
    pub static mut cl_timeout: *mut cvar_t;
    pub static mut cl_maxpackets: *mut cvar_t;
    pub static mut cl_packetdup: *mut cvar_t;
    pub static mut cl_timeNudge: *mut cvar_t;
    pub static mut cl_showTimeDelta: *mut cvar_t;
    pub static mut cl_newClock: *mut cvar_t;
    pub static mut cl_shownet: *mut cvar_t;
    pub static mut cl_avidemo: *mut cvar_t;
    pub static mut cl_pano: *mut cvar_t;
    pub static mut cl_panoNumShots: *mut cvar_t;
    pub static mut cl_skippingcin: *mut cvar_t;
    pub static mut cl_endcredits: *mut cvar_t;
    pub static mut cl_freelook: *mut cvar_t;
    pub static mut cl_sensitivity: *mut cvar_t;
    pub static mut cl_mouseAccel: *mut cvar_t;
    pub static mut cl_showMouseRate: *mut cvar_t;
    pub static mut cl_VideoQuality: *mut cvar_t;
    pub static mut cl_VidFadeUp: *mut cvar_t; // deliberately kept as "Vid" rather than "Video" so tab-matching matches only VideoQuality
    pub static mut cl_VidFadeDown: *mut cvar_t;
    pub static mut cl_framerate: *mut cvar_t;
    pub static mut m_pitch: *mut cvar_t;
    pub static mut m_yaw: *mut cvar_t;
    pub static mut m_forward: *mut cvar_t;
    pub static mut m_side: *mut cvar_t;
    pub static mut m_filter: *mut cvar_t;
    pub static mut cl_activeAction: *mut cvar_t;
    pub static mut cl_updateInfoString: *mut cvar_t;
    pub static mut cl_ingameVideo: *mut cvar_t;
    pub static mut cl_thumbStickMode: *mut cvar_t;
}

// Xbox-specific cvar
#[cfg(target_os = "xbox")]
extern "C" {
    pub static mut cl_mapname: *mut cvar_t;
    pub static mut vidRestartReloadMap: qboolean;
}

// Extern struct declarations (these would be fully defined in their respective modules)
#[repr(C)]
pub struct cvar_t {
    // Placeholder - full definition would come from cvar header
}

#[repr(C)]
pub struct clientActive_t {
    // Placeholder
}

#[repr(C)]
pub struct clientConnection_t {
    // Placeholder
}

#[repr(C)]
pub struct clientStatic_t {
    // Placeholder
}

#[repr(C)]
pub struct refexport_t {
    // Placeholder
}

#[repr(C)]
pub struct ping_t {
    // Placeholder
}

pub type qboolean = c_int;

// Global structures
extern "C" {
    pub static mut cl: clientActive_t;
    pub static mut clc: clientConnection_t;
    pub static mut cls: clientStatic_t;

    // Structure containing functions exported from refresh DLL
    pub static mut re: refexport_t;

    pub static mut cl_pinglist: [ping_t; 0]; // MAX_PINGREQUESTS - size determined by external constant
}

// Forward declarations
extern "C" {
    pub fn CL_ShutdownRef();
    pub fn CL_InitRef();
    pub fn CL_CheckForResend();
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Z_Free(ptr: *mut c_void);
    pub fn CopyString(str: *const c_char) -> *mut c_char;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    pub fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn Cbuf_AddText(text: *const c_char);
    pub fn Cbuf_Execute();
    pub fn CL_ShutdownCGame();
    pub fn CL_ShutdownUI();
    pub fn S_DisableSounds();
    pub fn Con_Close();
    pub fn S_StopAllSounds();
    pub fn NET_StringToAdr(string: *const c_char, adr: *mut c_void);
    pub fn NET_CompareAdr(a: c_void, b: c_void) -> qboolean;
    pub fn NET_AdrToString(a: c_void) -> *const c_char;
    pub fn NET_CompareBaseAdr(a: c_void, b: c_void) -> qboolean;
    pub fn NET_OutOfBandPrint(sock: c_int, adr: c_void, fmt: *const c_char, ...);
    pub fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;
    pub fn Cvar_InfoString(flags: c_int) -> *const c_char;
    pub fn Info_SetValueForKey(info: *mut c_char, key: *const c_char, value: *const c_char);
    pub fn Cmd_Argv(arg: c_int) -> *const c_char;
    pub fn Cmd_Args() -> *const c_char;
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_TokenizeString(text: *const c_char);
    pub fn Cmd_AddCommand(cmd_name: *const c_char, function: *const c_void);
    pub fn Cmd_RemoveCommand(cmd_name: *const c_char);
    pub fn SCR_UpdateScreen();
    pub fn SCR_StopCinematic();
    pub fn SCR_DebugGraph(value: f32, color: c_int);
    pub fn SCR_RunCinematic();
    pub fn SCR_Init();
    pub fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
    pub fn S_ClearSoundBuffer();
    pub fn CL_ClearState();
    pub fn CL_FreeReliableCommands();
    pub fn CL_StartHunkUsers();
    pub fn CL_InitUI();
    pub fn CL_InitCGame();
    pub fn Netchan_Setup(sock: c_int, chan: *mut c_void, from: c_void, qport: c_int);
    pub fn Netchan_Process(chan: *mut c_void, msg: *mut c_void) -> qboolean;
    pub fn MSG_BeginReading(msg: *mut c_void);
    pub fn MSG_ReadLong(msg: *mut c_void) -> c_int;
    pub fn MSG_ReadStringLine(msg: *mut c_void) -> *const c_char;
    pub fn MSG_ReadString(msg: *mut c_void) -> *const c_char;
    pub fn CL_ParseServerMessage(msg: *mut c_void);
    pub fn CL_SendCmd();
    pub fn CL_SetCGameTime();
    pub fn CL_WritePacket();
    pub fn CL_InitInput();
    pub fn CL_IsRunningInGameCinematic() -> qboolean;
    pub fn CL_CheckPendingCinematic() -> qboolean;
    pub fn CL_PlayCinematic_f();
    pub fn CL_PlayInGameCinematic_f();
    pub fn CL_GenericMenu_f();
    pub fn CL_DataPad_f();
    pub fn CL_EndScreenDissolve_f();
    pub fn UI_SetActiveMenu(menu: *const c_char, arg: *const c_void);
    pub fn UI_UpdateConnectionString(string: *mut c_char);
    pub fn UI_UpdateConnectionMessageString(string: *const c_char);
    pub fn Info_Print(info: *const c_char);
    pub fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    pub fn S_Init();
    pub fn S_BeginRegistration();
    pub fn S_Shutdown();
    pub fn S_RestartMusic();
    pub fn S_ReloadAllUsedSounds();
    pub fn S_Update();
    pub fn S_StopSounds();
    pub fn Con_RunConsole();
    pub fn GetRefAPI(version: c_int) -> *mut refexport_t;
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
    pub fn Cvar_Get(var_name: *const c_char, default_value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Cvar_SetValue(var_name: *const c_char, value: f32);
    pub fn Cvar_ModifiedFlags() -> c_int;
    pub fn SE_GetString(string_name: *const c_char) -> *const c_char;
    pub fn RM_InitTerrain();
}

// Xbox-specific externs
#[cfg(target_os = "xbox")]
extern "C" {
    pub fn CL_UpdateHotSwap();
    pub fn R_DeleteTextures();
    pub fn CM_LoadMap(mapname: *const c_char, clientload: qboolean, checksum: *mut c_int);
    pub fn RE_LoadWorldMap(mapname: *const c_char);
    pub fn SP_DisplayLogos();
    pub fn CIN_Init();
}

// Immersion (force feedback) externs
#[cfg(feature = "immersion")]
extern "C" {
    pub fn CL_ShutdownFF();
    pub fn CL_InitFF();
    pub fn FF_IsInitialized() -> qboolean;
    pub fn FF_Init() -> qboolean;
    pub fn FF_Shutdown();
    pub fn FF_Update();
    pub fn AS_ParseSets();
}

// Sound system externs
extern "C" {
    pub static mut s_soundMuted: qboolean;
    pub fn Sys_In_Restart_f();
    pub fn CL_FreeServerCommands();
}

// Global variables
pub static mut cvar_modifiedFlags: c_int = 0;
pub static mut com_cl_running: *mut cvar_t = core::ptr::null_mut();
pub static mut com_sv_running: *mut cvar_t = core::ptr::null_mut();
pub static mut com_developer: *mut cvar_t = core::ptr::null_mut();
pub static mut cl_paused: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_paused: *mut cvar_t = core::ptr::null_mut();
pub static mut cl_timegraph: *mut cvar_t = core::ptr::null_mut();

// Local statics for CL_Frame
pub static mut frameCount: u32 = 0;
pub static mut avgFrametime: f32 = 0.0;

const MAX_RELIABLE_COMMANDS: usize = 64; // Standard Q3 value, adjust if needed
const MAX_PINGREQUESTS: usize = 32;
const MAX_CONFIGSTRINGS: usize = 1024;
const MAX_STRING_CHARS: usize = 1024;
const MAX_INFO_STRING: usize = 1024;
const PROTOCOL_VERSION: c_int = 15;
const CVAR_USERINFO: c_int = 1 << 1;
const CVAR_TEMP: c_int = 1 << 2;
const CVAR_ROM: c_int = 1 << 3;
const CVAR_ARCHIVE: c_int = 1 << 4;
const CVAR_SAVEGAME: c_int = 1 << 6;
const CVAR_NORESTART: c_int = 1 << 7;
const CA_UNINITIALIZED: c_int = 0;
const CA_DISCONNECTED: c_int = 1;
const CA_CONNECTING: c_int = 2;
const CA_CHALLENGING: c_int = 3;
const CA_CONNECTED: c_int = 4;
const CA_PRIMED: c_int = 5;
const CA_ACTIVE: c_int = 6;
const CA_CINEMATIC: c_int = 7;
const ERR_FATAL: c_int = 1;
const ERR_DROP: c_int = 2;
const ERR_DISCONNECT: c_int = 3;
const KEYCATCH_CONSOLE: c_int = 1;
const KEYCATCH_UI: c_int = 2;
const SMALLCHAR_WIDTH: usize = 8;
const EXEC_NOW: c_int = 0;
const NS_CLIENT: c_int = 0;
const PRINT_ALL: c_int = 0;
const PRINT_WARNING: c_int = 1;
const PRINT_DEVELOPER: c_int = 2;
const S_COLOR_YELLOW: &str = "^3";
const S_COLOR_RED: &str = "^1";
const REF_API_VERSION: c_int = 12;

/*
=======================================================================

CLIENT RELIABLE COMMAND COMMUNICATION

=======================================================================
*/

/*
======================
CL_AddReliableCommand

The given command will be transmitted to the server, and is gauranteed to
not have future usercmd_t executed before it is executed
======================
*/
pub extern "C" fn CL_AddReliableCommand(cmd: *const c_char) {
    let mut index: c_int;

    // if we would be losing an old command that hasn't been acknowledged,
    // we must drop the connection
    if unsafe { (*core::ptr::addr_of_mut!(clc)).reliableSequence - (*core::ptr::addr_of!(clc)).reliableAcknowledge > MAX_RELIABLE_COMMANDS as c_int } {
        unsafe {
            Com_Error(ERR_DROP, b"Client command overflow\0".as_ptr() as *const c_char);
        }
    }
    unsafe {
        (*core::ptr::addr_of_mut!(clc)).reliableSequence += 1;
    }
    index = unsafe { (*core::ptr::addr_of!(clc)).reliableSequence & ((MAX_RELIABLE_COMMANDS as c_int) - 1) };
    if unsafe { !(*core::ptr::addr_of!(clc)).reliableCommands[index as usize].is_null() } {
        unsafe {
            Z_Free((*core::ptr::addr_of!(clc)).reliableCommands[index as usize] as *mut c_void);
        }
    }
    unsafe {
        (*core::ptr::addr_of_mut!(clc)).reliableCommands[index as usize] = CopyString(cmd);
    }
}

//======================================================================

/*
==================
CL_NextDemo

Called when a demo or cinematic finishes
If the "nextdemo" cvar is set, that command will be issued
==================
*/
pub extern "C" fn CL_NextDemo() {
    let mut v: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

    unsafe {
        Q_strncpyz(v.as_mut_ptr(), Cvar_VariableString(b"nextdemo\0".as_ptr() as *const c_char), v.len());
    }
    v[MAX_STRING_CHARS - 1] = 0;
    unsafe {
        Com_DPrintf(b"CL_NextDemo: %s\n\0".as_ptr() as *const c_char, v.as_ptr());
    }
    if v[0] == 0 {
        return;
    }

    unsafe {
        Cvar_Set(b"nextdemo\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
        Cbuf_AddText(v.as_ptr() as *const c_char);
        Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
        Cbuf_Execute();
    }
}

//======================================================================

/*
=================
CL_FlushMemory

Called by CL_MapLoading, CL_Connect_f, and CL_ParseGamestate the only
ways a client gets into a game
Also called by Com_Error
=================
*/
pub extern "C" fn CL_FlushMemory() {
    // clear sounds (moved higher up within this func to avoid the odd sound stutter)
    unsafe {
        S_DisableSounds();
    }

    // unload the old VM
    unsafe {
        CL_ShutdownCGame();
    }

    unsafe {
        CL_ShutdownUI();
    }

    if unsafe { !(*core::ptr::addr_of!(re)).Shutdown.is_null() } {
        unsafe {
            (*core::ptr::addr_of!(re)).Shutdown.unwrap()(0);		// don't destroy window or context
        }
    }

    //rwwFIXMEFIXME: The game server appears to continue running, so clearing common bsp data causes crashing and other bad things
    /*
    CM_ClearMap();
    */

    unsafe {
        (*core::ptr::addr_of_mut!(cls)).soundRegistered = 0;
        (*core::ptr::addr_of_mut!(cls)).rendererStarted = 0;
    }
    #[cfg(feature = "immersion")]
    unsafe {
        CL_ShutdownFF();
        (*core::ptr::addr_of_mut!(cls)).forceStarted = 0;
    }
}

/*
=====================
CL_MapLoading

A local server is starting to load a map, so update the
screen to let the user know about it, then dump all client
memory on the hunk from cgame, ui, and renderer
=====================
*/
pub extern "C" fn CL_MapLoading() {
    if unsafe { (*com_cl_running).integer == 0 } {
        return;
    }

    unsafe {
        Con_Close();
        (*core::ptr::addr_of_mut!(cls)).keyCatchers = 0;
    }

    // if we are already connected to the local host, stay connected
    if unsafe { (*core::ptr::addr_of!(cls)).state >= CA_CONNECTED } {
        let servername_ptr = unsafe { core::ptr::addr_of!((*core::ptr::addr_of!(cls)).servername) as *const c_char };
        if unsafe { libc::strcmp(servername_ptr, b"localhost\0".as_ptr() as *const c_char) == 0 } {
            unsafe {
                (*core::ptr::addr_of_mut!(cls)).state = CA_CONNECTED;		// so the connect screen is drawn
                libc::memset((*core::ptr::addr_of_mut!(cls)).updateInfoString.as_mut_ptr() as *mut c_void, 0, (*core::ptr::addr_of!(cls)).updateInfoString.len());
//		memset( clc.serverMessage, 0, sizeof( clc.serverMessage ) );
                libc::memset(core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(cl)).gameState) as *mut c_void, 0, core::mem::size_of_val(&(*core::ptr::addr_of!(cl)).gameState));
                (*core::ptr::addr_of_mut!(clc)).lastPacketSentTime = -9999;
                SCR_UpdateScreen();
            }
            return;
        }
    }

    // clear nextmap so the cinematic shutdown doesn't execute it
    unsafe {
        Cvar_Set(b"nextmap\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
    }
    #[cfg(target_os = "xbox")]
    {
        let oldState: c_int = unsafe { (*core::ptr::addr_of!(cls)).state };
        unsafe {
            (*core::ptr::addr_of_mut!(cls)).state = CA_CHALLENGING;
            SCR_UpdateScreen();
            (*core::ptr::addr_of_mut!(cls)).state = oldState;
        }
    }

    unsafe {
        CL_Disconnect();
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(cls)).servername.as_mut_ptr(),
            b"localhost\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(cls)).servername.len()
        );
        (*core::ptr::addr_of_mut!(cls)).state = CA_CHALLENGING;		// so the connect screen is drawn
        (*core::ptr::addr_of_mut!(cls)).keyCatchers = 0;
    }
    #[cfg(not(target_os = "xbox"))]
    unsafe {
        SCR_UpdateScreen();
    }

    unsafe {
        (*core::ptr::addr_of_mut!(clc)).connectTime = -RETRANSMIT_TIMEOUT;
        NET_StringToAdr(
            (*core::ptr::addr_of!(cls)).servername.as_ptr() as *const c_char,
            core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(clc)).serverAddress) as *mut c_void
        );
        // we don't need a challenge on the localhost

        CL_CheckForResend();
    }

    CL_FlushMemory();
}

/*
=====================
CL_ClearState

Called before parsing a gamestate
=====================
*/
pub extern "C" fn CL_ClearState() {
    unsafe {
        CL_ShutdownCGame();
    }

    unsafe {
        S_StopAllSounds();
    }

    unsafe {
        libc::memset(core::ptr::addr_of_mut!(cl) as *mut c_void, 0, core::mem::size_of_val(&cl));
    }
}

/*
=====================
CL_FreeReliableCommands

Wipes all reliableCommands strings from clc
=====================
*/
pub extern "C" fn CL_FreeReliableCommands() {
    // wipe the client connection
    for i in 0..MAX_RELIABLE_COMMANDS {
        if unsafe { !(*core::ptr::addr_of!(clc)).reliableCommands[i].is_null() } {
            unsafe {
                Z_Free((*core::ptr::addr_of!(clc)).reliableCommands[i] as *mut c_void);
                (*core::ptr::addr_of_mut!(clc)).reliableCommands[i] = core::ptr::null_mut();
            }
        }
    }
}


/*
=====================
CL_Disconnect

Called when a connection, or cinematic is being terminated.
Goes from a connected state to either a menu state or a console state
Sends a disconnect message to the server
This is also called on Com_Error and Com_Quit, so it shouldn't cause any errors
=====================
*/
pub extern "C" fn CL_Disconnect() {
    if unsafe { com_cl_running.is_null() || (*com_cl_running).integer == 0 } {
        return;
    }

    #[cfg(target_os = "xbox")]
    unsafe {
        Cvar_Set(b"r_norefresh\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
    }

    if unsafe { (*core::ptr::addr_of!(cls)).uiStarted != 0 } {
        unsafe {
            UI_SetActiveMenu(core::ptr::null(), core::ptr::null());
        }
    }

    unsafe {
        SCR_StopCinematic();
        S_ClearSoundBuffer();
    }

    #[cfg(target_os = "xbox")]
    unsafe {
//	extern qboolean RE_RegisterImages_LevelLoadEnd(void);
//	RE_RegisterImages_LevelLoadEnd();
        R_DeleteTextures();
    }

    // send a disconnect message to the server
    // send it a few times in case one is dropped
    if unsafe { (*core::ptr::addr_of!(cls)).state >= CA_CONNECTED } {
        unsafe {
            CL_AddReliableCommand(b"disconnect\0".as_ptr() as *const c_char);
            CL_WritePacket();
            CL_WritePacket();
            CL_WritePacket();
        }
    }

    unsafe {
        CL_ClearState();
    }

    unsafe {
        CL_FreeReliableCommands();
    }

    extern "C" {
        pub fn CL_FreeServerCommands();
    }
    unsafe {
        CL_FreeServerCommands();
    }

    unsafe {
        libc::memset(core::ptr::addr_of_mut!(clc) as *mut c_void, 0, core::mem::size_of_val(&clc));
    }

    unsafe {
        (*core::ptr::addr_of_mut!(cls)).state = CA_DISCONNECTED;
    }

    // allow cheats locally
    unsafe {
        Cvar_Set(b"timescale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);//jic we were skipping
        Cvar_Set(b"skippingCinematic\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);//jic we were skipping
    }
}


/*
===================
CL_ForwardCommandToServer

adds the current command line as a clientCommand
things like godmode, noclip, etc, are commands directed to the server,
so when they are typed in at the console, they will need to be forwarded.
===================
*/
pub extern "C" fn CL_ForwardCommandToServer() {
    let mut cmd: *const c_char;
    let mut string: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

    unsafe {
        cmd = Cmd_Argv(0);
    }

    // ignore key up commands
    if unsafe { *cmd as u8 as char } == '-' {
        return;
    }

    if unsafe { (*core::ptr::addr_of!(cls)).state != CA_ACTIVE || *cmd as u8 as char } == '+' {
        unsafe {
            Com_Printf(b"Unknown command \"%s\"\n\0".as_ptr() as *const c_char, cmd);
        }
        return;
    }

    if unsafe { Cmd_Argc() > 1 } {
        unsafe {
            Com_sprintf(string.as_mut_ptr(), string.len(), b"%s %s\0".as_ptr() as *const c_char, cmd, Cmd_Args());
        }
    } else {
        unsafe {
            Q_strncpyz(string.as_mut_ptr(), cmd, string.len());
        }
    }

    unsafe {
        CL_AddReliableCommand(string.as_ptr() as *const c_char);
    }
}


/*
======================================================================

CONSOLE COMMANDS

======================================================================
*/

/*
==================
CL_ForwardToServer_f
==================
*/
pub extern "C" fn CL_ForwardToServer_f() {
    if unsafe { (*core::ptr::addr_of!(cls)).state != CA_ACTIVE } {
        unsafe {
            Com_Printf(b"Not connected to a server.\n\0".as_ptr() as *const c_char);
        }
        return;
    }

    // don't forward the first argument
    if unsafe { Cmd_Argc() > 1 } {
        unsafe {
            CL_AddReliableCommand(Cmd_Args());
        }
    }
}

/*
==================
CL_Disconnect_f
==================
*/
pub extern "C" fn CL_Disconnect_f() {
    unsafe {
        SCR_StopCinematic();
    }

    //FIXME:
    // TA codebase added additional CA_CINEMATIC check below, presumably so they could play cinematics
    //	in the menus when disconnected, although having the SCR_StopCinematic() call above is weird.
    // Either there's a bug, or the new version of that function conditionally-doesn't stop cinematics...
    //
    if unsafe { (*core::ptr::addr_of!(cls)).state != CA_DISCONNECTED && (*core::ptr::addr_of!(cls)).state != CA_CINEMATIC } {
        unsafe {
            Com_Error(ERR_DISCONNECT, b"Disconnected from server\0".as_ptr() as *const c_char);
        }
    }
}


/*
=================
CL_Vid_Restart_f

Restart the video subsystem
=================
*/
pub extern "C" fn CL_Vid_Restart_f() {
    unsafe {
        S_StopAllSounds();		// don't let them loop during the restart
        S_BeginRegistration();	// all sound handles are now invalid
        CL_ShutdownRef();
        CL_ShutdownUI();
        CL_ShutdownCGame();
    }

    //rww - sof2mp does this here, but it seems to cause problems in this codebase.
//	CM_ClearMap();

    unsafe {
        CL_InitRef();
    }

    unsafe {
        (*core::ptr::addr_of_mut!(cls)).rendererStarted = 0;
        (*core::ptr::addr_of_mut!(cls)).uiStarted = 0;
        (*core::ptr::addr_of_mut!(cls)).cgameStarted = 0;
        (*core::ptr::addr_of_mut!(cls)).soundRegistered = 0;
    }

    #[cfg(feature = "immersion")]
    unsafe {
        CL_ShutdownFF();
        (*core::ptr::addr_of_mut!(cls)).forceStarted = 0;
    }

    #[cfg(target_os = "xbox")]
    unsafe {
        vidRestartReloadMap = 1;
    }

    // unpause so the cgame definately gets a snapshot and renders a frame
    unsafe {
        Cvar_Set(b"cl_paused\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
    }
}

/*
=================
CL_Snd_Restart_f

Restart the sound subsystem
The cgame and game must also be forced to restart because
handles will be invalid
=================
*/
pub extern "C" fn CL_Snd_Restart_f() {
    unsafe {
        S_Shutdown();
    }

    unsafe {
        S_Init();
    }

//	CL_Vid_Restart_f();

    unsafe {
        s_soundMuted = 0;		// we can play again

        S_RestartMusic();

        S_ReloadAllUsedSounds();

        AS_ParseSets();
    }
}

#[cfg(feature = "immersion")]
/*
=================
CL_FF_Restart_f
=================
*/
pub extern "C" fn CL_FF_Restart_f() {
    unsafe {
        if FF_IsInitialized() != 0 {
            // Apply cvar changes w/o losing registered effects
            // Allows changing devices in-game without restarting the map
            if FF_Init() == 0 {
                FF_Shutdown();	// error (shouldn't happen)
            }
        } else if (*core::ptr::addr_of!(cls)).state >= CA_PRIMED {	// maybe > CA_DISCONNECTED
            // Restart map or menu
            CL_Vid_Restart_f();
        } else if (*core::ptr::addr_of!(cls)).uiStarted != 0 {
            // Restart menu
            CL_ShutdownUI();
            (*core::ptr::addr_of_mut!(cls)).forceStarted = 0;
        }
    }
}

/*
==================
CL_Configstrings_f
==================
*/
pub extern "C" fn CL_Configstrings_f() {
    let mut i: c_int;
    let mut ofs: c_int;

    if unsafe { (*core::ptr::addr_of!(cls)).state != CA_ACTIVE } {
        unsafe {
            Com_Printf(b"Not connected to a server.\n\0".as_ptr() as *const c_char);
        }
        return;
    }

    for i in 0..MAX_CONFIGSTRINGS as c_int {
        unsafe {
            ofs = (*core::ptr::addr_of!(cl)).gameState.stringOffsets[i as usize];
        }
        if ofs == 0 {
            continue;
        }
        unsafe {
            Com_Printf(
                b"%4i: %s\n\0".as_ptr() as *const c_char,
                i,
                (*core::ptr::addr_of!(cl)).gameState.stringData.as_ptr().add(ofs as usize)
            );
        }
    }
}

/*
==============
CL_Clientinfo_f
==============
*/
pub extern "C" fn CL_Clientinfo_f() {
    unsafe {
        Com_Printf(b"--------- Client Information ---------\n\0".as_ptr() as *const c_char);
        Com_Printf(b"state: %i\n\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(cls)).state);
        Com_Printf(b"Server: %s\n\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(cls)).servername.as_ptr());
        Com_Printf(b"User info settings:\n\0".as_ptr() as *const c_char);
        Info_Print(Cvar_InfoString(CVAR_USERINFO));
        Com_Printf(b"--------------------------------------\n\0".as_ptr() as *const c_char);
    }
}


//====================================================================

extern "C" {
    pub fn UI_UpdateConnectionString(string: *mut c_char);
}

/*
=================
CL_CheckForResend

Resend a connect message if the last one has timed out
=================
*/
pub extern "C" fn CL_CheckForResend() {
    let mut port: c_int;
    let mut info: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];

//	if ( cls.state == CA_CINEMATIC )
    if unsafe { (*core::ptr::addr_of!(cls)).state == CA_CINEMATIC || CL_IsRunningInGameCinematic() != 0 } {
        return;
    }

    // resend if we haven't gotten a reply yet
    if unsafe { (*core::ptr::addr_of!(cls)).state < CA_CONNECTING || (*core::ptr::addr_of!(cls)).state > CA_CHALLENGING } {
        return;
    }

    if unsafe { (*core::ptr::addr_of!(cls)).realtime - (*core::ptr::addr_of!(clc)).connectTime < RETRANSMIT_TIMEOUT } {
        return;
    }

    unsafe {
        (*core::ptr::addr_of_mut!(clc)).connectTime = (*core::ptr::addr_of!(cls)).realtime;	// for retransmit requests
        (*core::ptr::addr_of_mut!(clc)).connectPacketCount += 1;
    }

    // requesting a challenge
    unsafe {
        match (*core::ptr::addr_of!(cls)).state {
        CA_CONNECTING => {
            UI_UpdateConnectionString(va(b"(%i)\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(clc)).connectPacketCount) as *mut c_char);

            NET_OutOfBandPrint(NS_CLIENT, (*core::ptr::addr_of!(clc)).serverAddress, b"getchallenge\0".as_ptr() as *const c_char);
        }

        CA_CHALLENGING => {
        // sending back the challenge
            port = Cvar_VariableIntegerValue(b"qport\0".as_ptr() as *const c_char);

            UI_UpdateConnectionString(va(b"(%i)\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(clc)).connectPacketCount) as *mut c_char);

            Q_strncpyz(info.as_mut_ptr(), Cvar_InfoString(CVAR_USERINFO), info.len());
            Info_SetValueForKey(info.as_mut_ptr(), b"protocol\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, PROTOCOL_VERSION));
            Info_SetValueForKey(info.as_mut_ptr(), b"qport\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, port));
            Info_SetValueForKey(info.as_mut_ptr(), b"challenge\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(clc)).challenge));
            NET_OutOfBandPrint(NS_CLIENT, (*core::ptr::addr_of!(clc)).serverAddress, b"connect \"%s\"\0".as_ptr() as *const c_char, info.as_ptr());
            // the most current userinfo has been sent, so watch for any
            // newer changes to userinfo variables
            cvar_modifiedFlags &= !CVAR_USERINFO;
        }

        _ => {
            Com_Error(ERR_FATAL, b"CL_CheckForResend: bad cls.state\0".as_ptr() as *const c_char);
        }
        }
    }
}


/*
===================
CL_DisconnectPacket

Sometimes the server can drop the client and the netchan based
disconnect can be lost.  If the client continues to send packets
to the server, the server will send out of band disconnect packets
to the client so it doesn't have to wait for the full timeout period.
===================
*/
pub extern "C" fn CL_DisconnectPacket(from: c_void) {
    if unsafe { (*core::ptr::addr_of!(cls)).state != CA_ACTIVE } {
        return;
    }

    // if not from our server, ignore it
    if unsafe { NET_CompareAdr(from, (*core::ptr::addr_of!(clc)).netchan.remoteAddress) == 0 } {
        return;
    }

    // if we have received packets within three seconds, ignore it
    // (it might be a malicious spoof)
    if unsafe { (*core::ptr::addr_of!(cls)).realtime - (*core::ptr::addr_of!(clc)).lastPacketTime < 3000 } {
        return;
    }

    // drop the connection (FIXME: connection dropped dialog)
    unsafe {
        Com_Printf(b"Server disconnected for unknown reason\n\0".as_ptr() as *const c_char);
        CL_Disconnect();
    }
}


/*
=================
CL_ConnectionlessPacket

Responses to broadcasts, etc
=================
*/
pub extern "C" fn CL_ConnectionlessPacket(from: c_void, msg: *mut c_void) {
    let mut s: *const c_char;
    let mut c: *const c_char;

    unsafe {
        MSG_BeginReading(msg);
        MSG_ReadLong(msg);	// skip the -1

        s = MSG_ReadStringLine(msg);

        Cmd_TokenizeString(s);

        c = Cmd_Argv(0);

        Com_DPrintf(b"CL packet %s: %s\n\0".as_ptr() as *const c_char, NET_AdrToString(from), c);

        // challenge from the server we are connecting to
        if libc::strcmp(c, b"challengeResponse\0".as_ptr() as *const c_char) == 0 {
            if (*core::ptr::addr_of!(cls)).state != CA_CONNECTING {
                Com_Printf(b"Unwanted challenge response received.  Ignored.\n\0".as_ptr() as *const c_char);
            } else {
                // start sending challenge repsonse instead of challenge request packets
                (*core::ptr::addr_of_mut!(clc)).challenge = libc::atoi(Cmd_Argv(1)) as c_int;
                (*core::ptr::addr_of_mut!(cls)).state = CA_CHALLENGING;
                (*core::ptr::addr_of_mut!(clc)).connectPacketCount = 0;
                (*core::ptr::addr_of_mut!(clc)).connectTime = -99999;

                // take this address as the new server address.  This allows
                // a server proxy to hand off connections to multiple servers
                (*core::ptr::addr_of_mut!(clc)).serverAddress = from;
            }
            return;
        }

        // server connection
        if libc::strcmp(c, b"connectResponse\0".as_ptr() as *const c_char) == 0 {
            if (*core::ptr::addr_of!(cls)).state >= CA_CONNECTED {
                Com_Printf(b"Dup connect received.  Ignored.\n\0".as_ptr() as *const c_char);
                return;
            }
            if (*core::ptr::addr_of!(cls)).state != CA_CHALLENGING {
                Com_Printf(b"connectResponse packet while not connecting.  Ignored.\n\0".as_ptr() as *const c_char);
                return;
            }
            if NET_CompareBaseAdr(from, (*core::ptr::addr_of!(clc)).serverAddress) == 0 {
                Com_Printf(b"connectResponse from a different address.  Ignored.\n\0".as_ptr() as *const c_char);
                Com_Printf(b"%s should have been %s\n\0".as_ptr() as *const c_char, NET_AdrToString(from),
                    NET_AdrToString((*core::ptr::addr_of!(clc)).serverAddress));
                return;
            }
            Netchan_Setup(NS_CLIENT, core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(clc)).netchan) as *mut c_void, from, Cvar_VariableIntegerValue(b"qport\0".as_ptr() as *const c_char));
            (*core::ptr::addr_of_mut!(cls)).state = CA_CONNECTED;
            (*core::ptr::addr_of_mut!(clc)).lastPacketSentTime = -9999;		// send first packet immediately
            return;
        }

        // a disconnect message from the server, which will happen if the server
        // dropped the connection but it is still getting packets from us
        if libc::strcmp(c, b"disconnect\0".as_ptr() as *const c_char) == 0 {
            CL_DisconnectPacket(from);
            return;
        }

        // echo request from server
        if libc::strcmp(c, b"echo\0".as_ptr() as *const c_char) == 0 {
            NET_OutOfBandPrint(NS_CLIENT, from, b"%s\0".as_ptr() as *const c_char, Cmd_Argv(1));
            return;
        }

        // print request from server
        if libc::strcmp(c, b"print\0".as_ptr() as *const c_char) == 0 {
            s = MSG_ReadString(msg);
            UI_UpdateConnectionMessageString(s);
            Com_Printf(b"%s\0".as_ptr() as *const c_char, s);
            return;
        }


        Com_DPrintf(b"Unknown connectionless packet command.\n\0".as_ptr() as *const c_char);
    }
}


/*
=================
CL_PacketEvent

A packet has arrived from the main event loop
=================
*/
pub extern "C" fn CL_PacketEvent(from: c_void, msg: *mut c_void) {
    let mut headerBytes: c_int;

    unsafe {
        (*core::ptr::addr_of_mut!(clc)).lastPacketTime = (*core::ptr::addr_of!(cls)).realtime;
    }

    if unsafe { (*(msg as *mut c_int)).unsigned_abs() >= 4 && *(msg as *mut *mut c_int) as i32 == -1 } {
        unsafe {
            CL_ConnectionlessPacket(from, msg);
        }
        return;
    }

    if unsafe { (*core::ptr::addr_of!(cls)).state < CA_CONNECTED } {
        return;		// can't be a valid sequenced packet
    }

    if unsafe { (*(msg as *mut c_int)) < 8 } {
        unsafe {
            Com_Printf(b"%s: Runt packet\n\0".as_ptr() as *const c_char, NET_AdrToString(from));
        }
        return;
    }

    //
    // packet from server
    //
    if unsafe { NET_CompareAdr(from, (*core::ptr::addr_of!(clc)).netchan.remoteAddress) == 0 } {
        unsafe {
            Com_DPrintf(b"%s:sequenced packet without connection\n\0".as_ptr() as *const c_char, NET_AdrToString(from));
            // FIXME: send a client disconnect?
        }
        return;
    }

    if unsafe { Netchan_Process(core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(clc)).netchan) as *mut c_void, msg) == 0 } {
        return;		// out of order, duplicated, etc
    }

    // the header is different lengths for reliable and unreliable messages
    unsafe {
        headerBytes = (*(msg as *mut c_int));
    }

    unsafe {
        (*core::ptr::addr_of_mut!(clc)).lastPacketTime = (*core::ptr::addr_of!(cls)).realtime;
        CL_ParseServerMessage(msg);
    }
}

/*
==================
CL_CheckTimeout

==================
*/
pub extern "C" fn CL_CheckTimeout() {
    //
    // check timeout
    //
    if unsafe {
        ((*cl_paused).integer == 0 || (*sv_paused).integer == 0)
//		&& cls.state >= CA_CONNECTED && cls.state != CA_CINEMATIC
            && (*core::ptr::addr_of!(cls)).state >= CA_CONNECTED && ((*core::ptr::addr_of!(cls)).state != CA_CINEMATIC && CL_IsRunningInGameCinematic() == 0)
            && (*core::ptr::addr_of!(cls)).realtime - (*core::ptr::addr_of!(clc)).lastPacketTime > ((*cl_timeout).value * 1000.0) as c_int
    } {
        unsafe {
            (*core::ptr::addr_of_mut!(cl)).timeoutcount += 1;
            if (*core::ptr::addr_of!(cl)).timeoutcount > 5 {	// timeoutcount saves debugger
                Com_Printf(b"\nServer connection timed out.\n\0".as_ptr() as *const c_char);
                CL_Disconnect();
                return;
            }
        }
    } else {
        unsafe {
            (*core::ptr::addr_of_mut!(cl)).timeoutcount = 0;
        }
    }
}


//============================================================================

/*
==================
CL_CheckUserinfo

==================
*/
pub extern "C" fn CL_CheckUserinfo() {
    if unsafe { (*core::ptr::addr_of!(cls)).state < CA_CHALLENGING } {
        return;
    }

    // send a reliable userinfo update if needed
    if unsafe { (cvar_modifiedFlags & CVAR_USERINFO) != 0 } {
        unsafe {
            cvar_modifiedFlags &= !CVAR_USERINFO;
            CL_AddReliableCommand(va(b"userinfo \"%s\"\0".as_ptr() as *const c_char, Cvar_InfoString(CVAR_USERINFO)));
        }
    }

}


/*
==================
CL_Frame

==================
*/
extern "C" {
    pub static mut cl_newClock: *mut cvar_t;
}
pub static mut frameCount: u32 = 0;
pub static mut avgFrametime: f32 = 0.0;

pub extern "C" fn CL_Frame(msec: c_int, fractionMsec: f32) {
    let mut msec = msec;

    if unsafe { (*com_cl_running).integer == 0 } {
        return;
    }

    // load the ref / cgame if needed
    unsafe {
        CL_StartHunkUsers();
    }

    #[cfg(all(target_os = "xbox", not(debug_assertions)))]
    {
        // Play the intro movies once
        static mut firstRun: bool = true;
        if unsafe { firstRun } {
        //	SP_DoLicense();
            unsafe {
                SP_DisplayLogos();
            }
        }
    }

    #[cfg(target_os = "xbox")]
    {
        // load ui if needed
        if unsafe { (*core::ptr::addr_of!(cls)).uiStarted == 0 && (*core::ptr::addr_of!(cls)).state != CA_CINEMATIC } {
            unsafe {
                (*core::ptr::addr_of_mut!(cls)).uiStarted = 1;
                SCR_StopCinematic();
                CL_InitUI();
            }
        }
    }

    if unsafe { (*core::ptr::addr_of!(cls)).state == CA_DISCONNECTED && ((*core::ptr::addr_of!(cls)).keyCatchers & KEYCATCH_UI) == 0
        && (*com_sv_running).integer == 0 } {
        // if disconnected, bring up the menu
        if unsafe { CL_CheckPendingCinematic() == 0 } {	// this avoid having the menu flash for one frame before pending cinematics
            #[cfg(target_os = "xbox")]
            {
                static mut firstRun: bool = true;
                if unsafe { firstRun } {
                    unsafe {
                        UI_SetActiveMenu(b"splashMenu\0".as_ptr() as *const c_char, core::ptr::null());
                    }
                } else {
                    unsafe {
                        UI_SetActiveMenu(b"mainMenu\0".as_ptr() as *const c_char, core::ptr::null());
                    }
                }
                unsafe {
                    firstRun = false;
                }
            }
            #[cfg(not(target_os = "xbox"))]
            unsafe {
                UI_SetActiveMenu(b"mainMenu\0".as_ptr() as *const c_char, core::ptr::null());
            }
        }
    }

    #[cfg(target_os = "xbox")]
    {
        // handled above in firstRun handling
    }


    // if recording an avi, lock to a fixed fps
    if unsafe { (*cl_avidemo).integer != 0 } {
        // save the current screen
        if unsafe { (*core::ptr::addr_of!(cls)).state == CA_ACTIVE } {
            if unsafe { (*cl_avidemo).integer > 0 } {
                unsafe {
                    Cbuf_ExecuteText(EXEC_NOW, b"screenshot silent\n\0".as_ptr() as *const c_char);
                }
            } else {
                unsafe {
                    Cbuf_ExecuteText(EXEC_NOW, b"screenshot_tga silent\n\0".as_ptr() as *const c_char);
                }
            }
        }
        // fixed time for next frame
        if unsafe { (*cl_avidemo).integer > 0 } {
            msec = 1000 / unsafe { (*cl_avidemo).integer };
        } else {
            msec = 1000 / -unsafe { (*cl_avidemo).integer };
        }
    }

    // save the msec before checking pause
    unsafe {
        (*core::ptr::addr_of_mut!(cls)).realFrametime = msec;
    }

    // decide the simulation time
    unsafe {
        (*core::ptr::addr_of_mut!(cls)).frametime = msec;
    }
    if unsafe { (*cl_framerate).integer != 0 } {
        unsafe {
            avgFrametime += msec as f32;
            let mut mess: [c_char; 256] = [0; 256];
            if (frameCount & 0x1f) == 0 {
                let msg_str = format!("Frame rate={}\n\n", 1000.0f32 * (1.0 / (avgFrametime / 32.0f32)));
                // Note: This sprintf call needs proper handling
                avgFrametime = 0.0f32;
            }
            frameCount += 1;
        }
    }
    unsafe {
        (*core::ptr::addr_of_mut!(cls)).frametimeFraction = fractionMsec;
        (*core::ptr::addr_of_mut!(cls)).realtime += msec;
        (*core::ptr::addr_of_mut!(cls)).realtimeFraction += fractionMsec;
        if (*core::ptr::addr_of!(cls)).realtimeFraction >= 1.0f32 {
            if !cl_newClock.is_null() && (*cl_newClock).integer != 0 {
                (*core::ptr::addr_of_mut!(cls)).realtime += 1;
            }
            (*core::ptr::addr_of_mut!(cls)).realtimeFraction -= 1.0f32;
        }
    }
    #[cfg(not(target_os = "xbox"))]
    if unsafe { (*cl_timegraph).integer != 0 } {
        unsafe {
            SCR_DebugGraph((*core::ptr::addr_of!(cls)).realFrametime as f32 * 0.25, 0);
        }
    }

    #[cfg(target_os = "xbox")]
    {
        //Check on the hot swappable button states.
        unsafe {
            CL_UpdateHotSwap();
        }
    }

    // see if we need to update any userinfo
    unsafe {
        CL_CheckUserinfo();
    }

    // if we haven't gotten a packet in a long time,
    // drop the connection
    unsafe {
        CL_CheckTimeout();
    }

    // send intentions now
    unsafe {
        CL_SendCmd();
    }

    // resend a connection request if necessary
    unsafe {
        CL_CheckForResend();
    }

    // decide on the serverTime to render
    unsafe {
        CL_SetCGameTime();
    }

    if unsafe { (*cl_pano).integer != 0 && (*core::ptr::addr_of!(cls)).state == CA_ACTIVE } {	//grab some panoramic shots
        let mut i = 1;
        let pref = unsafe { (*cl_pano).integer };
        let oldnoprint = unsafe { (*cl_noprint).integer };
        unsafe {
            Con_Close();
            (*core::ptr::addr_of_mut!(cl_noprint)).integer = 1;	//hide the screen shot msgs
        }
        while i <= unsafe { (*cl_panoNumShots).integer } {
            unsafe {
                Cvar_SetValue(b"pano\0".as_ptr() as *const c_char, i as f32);
                SCR_UpdateScreen();// update the screen
                Cbuf_ExecuteText(EXEC_NOW, va(b"screenshot %dpano%02d\n\0".as_ptr() as *const c_char, pref, i));	//grab this screen
            }
            i += 1;
        }
        unsafe {
            Cvar_SetValue(b"pano\0".as_ptr() as *const c_char, 0.0f32);	//done
            (*core::ptr::addr_of_mut!(cl_noprint)).integer = oldnoprint;
        }
    }

    if unsafe { (*cl_skippingcin).integer != 0 && (*cl_endcredits).integer == 0 && (*com_developer).integer == 0 } {
        if unsafe { (*cl_skippingcin).modified != 0 } {
            unsafe {
                S_StopSounds();		//kill em all but music
                (*core::ptr::addr_of_mut!(cl_skippingcin)).modified = 0;
                Com_Printf(va(b"%s%s\0".as_ptr() as *const c_char, S_COLOR_YELLOW.as_ptr(), SE_GetString(b"CON_TEXT_SKIPPING\0".as_ptr() as *const c_char)));
                SCR_UpdateScreen();
            }
        }
    } else {
        // update the screen
        unsafe {
            SCR_UpdateScreen();
        }

        #[cfg(all(target_os = "xbox", not(debug_assertions)))]
        {
            // Note: D3DPERF_QueryRepeatFrame() is a Windows/Xbox specific function
            // Keeping the condition structure for clarity but would need Windows specific handling
        }
    }
    // update audio
    unsafe {
        S_Update();
    }

    #[cfg(feature = "immersion")]
    unsafe {
        FF_Update();
    }
    // advance local effects for next frame
    unsafe {
        SCR_RunCinematic();
    }

    unsafe {
        Con_RunConsole();
    }

    unsafe {
        (*core::ptr::addr_of_mut!(cls)).framecount += 1;
    }
}


//============================================================================

/*
================
VID_Printf

DLL glue
================
*/
const MAXPRINTMSG: usize = 4096;
pub extern "C" fn VID_Printf(print_level: c_int, fmt: *const c_char, ...) {
    // Note: Variadic function handling in Rust requires special consideration
    // This is a simplified stub that would need proper implementation
    unsafe {
        if print_level == PRINT_ALL {
            Com_Printf(b"%s\0".as_ptr() as *const c_char, fmt);
        } else if print_level == PRINT_WARNING {
            Com_Printf(b"%s%s\0".as_ptr() as *const c_char, S_COLOR_YELLOW.as_ptr(), fmt);		// yellow
        } else if print_level == PRINT_DEVELOPER {
            Com_DPrintf(b"%s%s\0".as_ptr() as *const c_char, S_COLOR_RED.as_ptr(), fmt);
        }
    }
}



/*
============
CL_ShutdownRef
============
*/
pub extern "C" fn CL_ShutdownRef() {
    if unsafe { (*core::ptr::addr_of!(re)).Shutdown.is_null() } {
        return;
    }
    unsafe {
        (*core::ptr::addr_of!(re)).Shutdown.unwrap()(1);
        libc::memset(core::ptr::addr_of_mut!(re) as *mut c_void, 0, core::mem::size_of_val(&re));
    }
}

/*
============================
CL_StartSound

Convenience function for the sound system to be started
REALLY early on Xbox, helps with memory fragmentation.
============================
*/
pub extern "C" fn CL_StartSound() {
    if unsafe { (*core::ptr::addr_of!(cls)).soundStarted == 0 } {
        unsafe {
            (*core::ptr::addr_of_mut!(cls)).soundStarted = 1;
            S_Init();
        }
    }

    if unsafe { (*core::ptr::addr_of!(cls)).soundRegistered == 0 } {
        unsafe {
            (*core::ptr::addr_of_mut!(cls)).soundRegistered = 1;
            S_BeginRegistration();
        }
    }
}

/*
============================
CL_StartHunkUsers

After the server has cleared the hunk, these will need to be restarted
This is the only place that any of these functions are called from
============================
*/
pub extern "C" fn CL_StartHunkUsers() {
    if unsafe { (*com_cl_running).integer == 0 } {
        return;
    }

    if unsafe { (*core::ptr::addr_of!(cls)).rendererStarted == 0 } {
        #[cfg(target_os = "xbox")]
        {
            //if ((!com_sv_running->integer || com_errorEntered) && !vidRestartReloadMap)
            //{
            //	// free up some memory
            //	extern void SV_ClearLastLevel(void);
            //	SV_ClearLastLevel();
            //}
        }

        unsafe {
            (*core::ptr::addr_of_mut!(cls)).rendererStarted = 1;
            // Note: BeginRegistration call needs proper glconfig pointer

            // load character sets
    //		cls.charSetShader = re.RegisterShaderNoMip( "gfx/2d/bigchars" );
            // cls.charSetShader = re.RegisterShaderNoMip( "gfx/2d/charsgrid_med" );
            // cls.whiteShader = re.RegisterShader( "white" );
            // cls.consoleShader = re.RegisterShader( "console" );
            // g_console_field_width = cls.glconfig.vidWidth / SMALLCHAR_WIDTH - 2;
            // kg.g_consoleField.widthInChars = g_console_field_width;
        }
        #[cfg(not(feature = "immersion"))]
        {
            //-------
            //	The latest Immersion Force Feedback system initializes here, not through
            //	win32 input system. Therefore, the window handle is valid :)
            //-------

            // now that the renderer has started up we know that the global hWnd is now valid,
            //	so we can now go ahead and (re)setup the input stuff that needs hWnds for DI...
            //  (especially Force feedback)...
            //
            static mut bOnceOnly: bool = false;	// only do once, not every renderer re-start
            if !unsafe { bOnceOnly } {
                unsafe {
                    bOnceOnly = true;
                    Sys_In_Restart_f();
                }
            }

            #[cfg(target_os = "xbox")]
            unsafe {
                if vidRestartReloadMap != 0 {
                    let mut checksum: c_int = 0;
                    CM_LoadMap(
                        va(b"maps/%s.bsp\0".as_ptr() as *const c_char, (*cl_mapname).string) as *const c_char,
                        0,
                        &mut checksum
                    );
                    RE_LoadWorldMap(va(b"maps/%s.bsp\0".as_ptr() as *const c_char, (*cl_mapname).string) as *const c_char);
                    vidRestartReloadMap = 0;
                }
            }

        }
    }

    if unsafe { (*core::ptr::addr_of!(cls)).soundStarted == 0 } {
        unsafe {
            (*core::ptr::addr_of_mut!(cls)).soundStarted = 1;
            S_Init();
        }
    }

    if unsafe { (*core::ptr::addr_of!(cls)).soundRegistered == 0 } {
        unsafe {
            (*core::ptr::addr_of_mut!(cls)).soundRegistered = 1;
            S_BeginRegistration();
        }
    }

    #[cfg(feature = "immersion")]
    if unsafe { (*core::ptr::addr_of!(cls)).forceStarted == 0 } {
        unsafe {
            (*core::ptr::addr_of_mut!(cls)).forceStarted = 1;
            CL_InitFF();
        }
    }

    #[cfg(not(target_os = "xbox"))]	//i guess xbox doesn't want the ui loaded all the time?
    //we require the ui to be loaded here or else it crashes trying to access the ui on command line map loads
    if unsafe { (*core::ptr::addr_of!(cls)).uiStarted == 0 } {
        unsafe {
            (*core::ptr::addr_of_mut!(cls)).uiStarted = 1;
            CL_InitUI();
        }
    }

//	if ( !cls.cgameStarted && cls.state > CA_CONNECTED && cls.state != CA_CINEMATIC ) {
    if unsafe { (*core::ptr::addr_of!(cls)).cgameStarted == 0 && (*core::ptr::addr_of!(cls)).state > CA_CONNECTED && ((*core::ptr::addr_of!(cls)).state != CA_CINEMATIC && CL_IsRunningInGameCinematic() == 0) } {
        unsafe {
            (*core::ptr::addr_of_mut!(cls)).cgameStarted = 1;
            CL_InitCGame();
        }
    }
}

/*
============
CL_InitRef
============
*/
pub extern "C" fn CL_InitRef() {
    let mut ret: *mut refexport_t;

    unsafe {
        Com_Printf(b"----- Initializing Renderer ----\n\0".as_ptr() as *const c_char);
    }

    // cinematic stuff

    unsafe {
        ret = GetRefAPI(REF_API_VERSION);
    }

    unsafe {
        Com_Printf(b"-------------------------------\n\0".as_ptr() as *const c_char);
    }

    if unsafe { ret.is_null() } {
        unsafe {
            Com_Error(ERR_FATAL, b"Couldn't initialize refresh\0".as_ptr() as *const c_char);
        }
    }

    unsafe {
        re = *ret;
    }

    // unpause so the cgame definately gets a snapshot and renders a frame
    unsafe {
        Cvar_Set(b"cl_paused\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
    }
}


//===========================================================================================

/*
====================
CL_Init
====================
*/
pub extern "C" fn CL_Init() {
    unsafe {
        Com_Printf(b"----- Client Initialization -----\n\0".as_ptr() as *const c_char);
    }

    unsafe {
        // Con_Init();
    }

    unsafe {
        CL_ClearState();
    }

    unsafe {
        (*core::ptr::addr_of_mut!(cls)).state = CA_DISCONNECTED;	// no longer CA_UNINITIALIZED
        (*core::ptr::addr_of_mut!(cls)).keyCatchers = KEYCATCH_CONSOLE;
        (*core::ptr::addr_of_mut!(cls)).realtime = 0;
        (*core::ptr::addr_of_mut!(cls)).realtimeFraction = 0.0f32;	// fraction of a msec accumulated
    }

    unsafe {
        CL_InitInput();
    }

    #[cfg(not(target_os = "xbox"))]	// No terrain on Xbox
    unsafe {
        RM_InitTerrain();
    }

    //
    // register our variables
    //
    unsafe {
        cl_noprint = Cvar_Get(b"cl_noprint\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

        cl_timeout = Cvar_Get(b"cl_timeout\0".as_ptr() as *const c_char, b"125\0".as_ptr() as *const c_char, 0);

        cl_timeNudge = Cvar_Get(b"cl_timeNudge\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_TEMP);
        cl_shownet = Cvar_Get(b"cl_shownet\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_TEMP);
        cl_showTimeDelta = Cvar_Get(b"cl_showTimeDelta\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_TEMP);
        cl_newClock = Cvar_Get(b"cl_newClock\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
        cl_activeAction = Cvar_Get(b"activeAction\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char, CVAR_TEMP);

        cl_avidemo = Cvar_Get(b"cl_avidemo\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        cl_pano = Cvar_Get(b"pano\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        cl_panoNumShots = Cvar_Get(b"panoNumShots\0".as_ptr() as *const c_char, b"10\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        cl_skippingcin = Cvar_Get(b"skippingCinematic\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ROM);
        cl_endcredits = Cvar_Get(b"cg_endcredits\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

        // Note: cl_yawspeed, cl_pitchspeed, cl_anglespeedkey, cl_maxpackets, cl_packetdup, cl_run are not used in this file
        // but would be initialized in CL_InitInput or similar

        cl_sensitivity = Cvar_Get(b"sensitivity\0".as_ptr() as *const c_char, b"5\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        cl_mouseAccel = Cvar_Get(b"cl_mouseAccel\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        cl_freelook = Cvar_Get(b"cl_freelook\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

        cl_showMouseRate = Cvar_Get(b"cl_showmouserate\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);

        cl_ingameVideo = Cvar_Get(b"cl_ingameVideo\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        cl_VideoQuality = Cvar_Get(b"cl_VideoQuality\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        cl_VidFadeUp = Cvar_Get(b"cl_VidFadeUp\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_TEMP);
        cl_VidFadeDown = Cvar_Get(b"cl_VidFadeDown\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_TEMP);
        cl_framerate = Cvar_Get(b"cl_framerate\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_TEMP);

        cl_thumbStickMode = Cvar_Get(b"ui_thumbStickMode\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

        // init autoswitch so the ui will have it correctly even
        // if the cgame hasn't been started
        Cvar_Get(b"cg_autoswitch\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

        m_pitch = Cvar_Get(b"m_pitch\0".as_ptr() as *const c_char, b"0.022\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        m_yaw = Cvar_Get(b"m_yaw\0".as_ptr() as *const c_char, b"0.022\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        m_forward = Cvar_Get(b"m_forward\0".as_ptr() as *const c_char, b"0.25\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        m_side = Cvar_Get(b"m_side\0".as_ptr() as *const c_char, b"0.25\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
        m_filter = Cvar_Get(b"m_filter\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

        #[cfg(target_os = "xbox")]
        {
            cl_mapname = Cvar_Get(b"cl_mapname\0".as_ptr() as *const c_char, b"t3_bounty\0".as_ptr() as *const c_char, CVAR_TEMP);
        }

        cl_updateInfoString = Cvar_Get(b"cl_updateInfoString\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char, CVAR_ROM);

        // userinfo
        Cvar_Get(b"name\0".as_ptr() as *const c_char, b"Jaden\0".as_ptr() as *const c_char, CVAR_USERINFO | CVAR_ARCHIVE);
        Cvar_Get(b"snaps\0".as_ptr() as *const c_char, b"20\0".as_ptr() as *const c_char, CVAR_USERINFO | CVAR_ARCHIVE);

        Cvar_Get(b"sex\0".as_ptr() as *const c_char, b"f\0".as_ptr() as *const c_char, CVAR_USERINFO | CVAR_ARCHIVE | CVAR_SAVEGAME | CVAR_NORESTART);
        Cvar_Get(b"snd\0".as_ptr() as *const c_char, b"jaden_fmle\0".as_ptr() as *const c_char, CVAR_USERINFO | CVAR_ARCHIVE | CVAR_SAVEGAME | CVAR_NORESTART);//UI_SetSexandSoundForModel changes to match sounds.cfg for model
        Cvar_Get(b"handicap\0".as_ptr() as *const c_char, b"100\0".as_ptr() as *const c_char, CVAR_USERINFO | CVAR_SAVEGAME | CVAR_NORESTART);

        //
        // register our commands
        //
        Cmd_AddCommand(b"cmd\0".as_ptr() as *const c_char, CL_ForwardToServer_f as *const c_void);
        Cmd_AddCommand(b"configstrings\0".as_ptr() as *const c_char, CL_Configstrings_f as *const c_void);
        Cmd_AddCommand(b"clientinfo\0".as_ptr() as *const c_char, CL_Clientinfo_f as *const c_void);
        Cmd_AddCommand(b"snd_restart\0".as_ptr() as *const c_char, CL_Snd_Restart_f as *const c_void);
        Cmd_AddCommand(b"vid_restart\0".as_ptr() as *const c_char, CL_Vid_Restart_f as *const c_void);
        Cmd_AddCommand(b"disconnect\0".as_ptr() as *const c_char, CL_Disconnect_f as *const c_void);
        Cmd_AddCommand(b"cinematic\0".as_ptr() as *const c_char, CL_PlayCinematic_f as *const c_void);
        Cmd_AddCommand(b"ingamecinematic\0".as_ptr() as *const c_char, CL_PlayInGameCinematic_f as *const c_void);
        Cmd_AddCommand(b"uimenu\0".as_ptr() as *const c_char, CL_GenericMenu_f as *const c_void);
        Cmd_AddCommand(b"datapad\0".as_ptr() as *const c_char, CL_DataPad_f as *const c_void);
        Cmd_AddCommand(b"endscreendissolve\0".as_ptr() as *const c_char, CL_EndScreenDissolve_f as *const c_void);
        #[cfg(feature = "immersion")]
        {
            Cmd_AddCommand(b"ff_restart\0".as_ptr() as *const c_char, CL_FF_Restart_f as *const c_void);
        }

        CL_InitRef();

        CL_StartHunkUsers();

        SCR_Init();

        Cbuf_Execute();

        Cvar_Set(b"cl_running\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);

        #[cfg(target_os = "xbox")]
        {
            Com_Printf(b"Initializing Cinematics...\n\0".as_ptr() as *const c_char);
            CIN_Init();
        }

        Com_Printf(b"----- Client Initialization Complete -----\n\0".as_ptr() as *const c_char);
    }
}


/*
===============
CL_Shutdown

===============
*/
pub extern "C" fn CL_Shutdown() {
    static mut recursive: qboolean = 0;

    if unsafe { com_cl_running.is_null() || (*com_cl_running).integer == 0 } {
        return;
    }

    unsafe {
        Com_Printf(b"----- CL_Shutdown -----\n\0".as_ptr() as *const c_char);
    }

    if unsafe { recursive != 0 } {
        unsafe {
            libc::printf(b"recursive shutdown\n\0".as_ptr() as *const c_char);
        }
        return;
    }
    unsafe {
        recursive = 1;
    }

    unsafe {
        CL_ShutdownUI();
        CL_Disconnect();

        S_Shutdown();
        CL_ShutdownRef();
    }

    #[cfg(feature = "immersion")]
    unsafe {
        CL_ShutdownFF();
    }

    unsafe {
        Cmd_RemoveCommand(b"cmd\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"configstrings\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"clientinfo\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"snd_restart\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"vid_restart\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"disconnect\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"cinematic\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"ingamecinematic\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand(b"pause\0".as_ptr() as *const c_char);

        Cvar_Set(b"cl_running\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);

        recursive = 0;

        libc::memset(core::ptr::addr_of_mut!(cls) as *mut c_void, 0, core::mem::size_of_val(&cls));

        Com_Printf(b"-----------------------\n\0".as_ptr() as *const c_char);
    }
}


/*
==================
CL_GetPing
==================
*/
pub extern "C" fn CL_GetPing(n: c_int, adrstr: *mut c_char, pingtime: *mut c_int) {
    let mut str: *const c_char;
    let mut time: c_int;

    if unsafe { cl_pinglist[n as usize].adr.port == 0 } {
        // empty slot
        unsafe {
            *adrstr = b'\0' as c_char;
        }
        unsafe {
            *pingtime = 0;
        }
        return;
    }

    unsafe {
        str = NET_AdrToString(cl_pinglist[n as usize].adr);
        libc::strcpy(adrstr, str);

        time = cl_pinglist[n as usize].time;
    }
    if time == 0 {
        // check for timeout
        time = unsafe { (*core::ptr::addr_of!(cls)).realtime - cl_pinglist[n as usize].start };
        if time < 500 {
            // not timed out yet
            time = 0;
        }
    }

    unsafe {
        *pingtime = time;
    }
}

/*
==================
CL_ClearPing
==================
*/
pub extern "C" fn CL_ClearPing(n: c_int) {
    if n < 0 || n >= MAX_PINGREQUESTS as c_int {
        return;
    }

    unsafe {
        cl_pinglist[n as usize].adr.port = 0;
    }
}

/*
==================
CL_GetPingQueueCount
==================
*/
pub extern "C" fn CL_GetPingQueueCount() -> c_int {
    let mut i: c_int;
    let mut count: c_int;

    count = 0;
    for i in 0..MAX_PINGREQUESTS as c_int {
        if unsafe { cl_pinglist[i as usize].adr.port != 0 } {
            count += 1;
        }
    }

    return count;
}

/*
==================
CL_GetFreePing
==================
*/
pub extern "C" fn CL_GetFreePing() -> *mut ping_t {
    let mut pingptr: *mut ping_t;
    let mut best: *mut ping_t;
    let mut oldest: c_int;
    let mut i: c_int;
    let mut time: c_int;

    for i in 0..MAX_PINGREQUESTS as c_int {
        unsafe {
            pingptr = &mut cl_pinglist[i as usize];
        }
        if unsafe { (*pingptr).adr.port != 0 } {
            if unsafe { (*pingptr).time == 0 } {
                if unsafe { (*core::ptr::addr_of!(cls)).realtime - (*pingptr).start < 500 } {
                    // still waiting for response
                    continue;
                }
            } else if unsafe { (*pingptr).time < 500 } {
                // results have not been queried
                continue;
            }
        }

        // clear it
        unsafe {
            (*pingptr).adr.port = 0;
        }
        return pingptr;
    }

    // use oldest entry
    unsafe {
        pingptr = &mut cl_pinglist[0];
        best = &mut cl_pinglist[0];
    }
    oldest = i32::MIN;
    for i in 0..MAX_PINGREQUESTS as c_int {
        unsafe {
            pingptr = &mut cl_pinglist[i as usize];
        }
        // scan for oldest
        time = unsafe { (*core::ptr::addr_of!(cls)).realtime - (*pingptr).start };
        if time > oldest {
            oldest = time;
            best = pingptr;
        }
    }

    return best;
}
