// qcommon.h -- definitions common between client and server, but not game.or ref modules

use core::ffi::{c_char, c_int, c_uint, c_void};

// Forward declarations from included headers
// #include "../qcommon/cm_public.h"
// #include "../game/q_shared.h"

// Note: qboolean, byte, usercmd_t, entityState_t, playerState_t, cvar_t,
// fileHandle_t, fsMode_t, vmCvar_t, qtime_t, memtag_t, and CGenericParser2
// are defined in imported modules (q_shared.h, cm_public.h, etc.)

//#define	PRE_RELEASE_DEMO

//#define		USE_CD_KEY

//============================================================================

//
// msg.c
//
#[repr(C)]
pub struct msg_t {
    pub allowoverflow: c_int,  // if false, do a Com_Error
    pub overflowed: c_int,     // set to true if the buffer size failed (with allowoverflow set)
    pub oob: c_int,            // set to true if the buffer size failed (with allowoverflow set)
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,            // for bitwise reads and writes
}

extern "C" {
    pub fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);
    pub fn MSG_InitOOB(buf: *mut msg_t, data: *mut u8, length: c_int);
    pub fn MSG_Clear(buf: *mut msg_t);
    pub fn MSG_WriteData(buf: *mut msg_t, data: *const c_void, length: c_int);
    pub fn MSG_Bitstream(buf: *mut msg_t);
}

// struct usercmd_s;
// struct entityState_s;
// struct playerState_s;

extern "C" {
    pub fn MSG_WriteBits(msg: *mut msg_t, value: c_int, bits: c_int);

    pub fn MSG_WriteChar(sb: *mut msg_t, c: c_int);
    pub fn MSG_WriteByte(sb: *mut msg_t, c: c_int);
    pub fn MSG_WriteShort(sb: *mut msg_t, c: c_int);
    pub fn MSG_WriteLong(sb: *mut msg_t, c: c_int);
    pub fn MSG_WriteFloat(sb: *mut msg_t, f: f32);
    pub fn MSG_WriteString(sb: *mut msg_t, s: *const c_char);
    pub fn MSG_WriteBigString(sb: *mut msg_t, s: *const c_char);
    pub fn MSG_WriteAngle16(sb: *mut msg_t, f: f32);

    pub fn MSG_BeginReading(sb: *mut msg_t);
    pub fn MSG_BeginReadingOOB(sb: *mut msg_t);

    pub fn MSG_ReadBits(msg: *mut msg_t, bits: c_int) -> c_int;

    pub fn MSG_ReadChar(sb: *mut msg_t) -> c_int;
    pub fn MSG_ReadByte(sb: *mut msg_t) -> c_int;
    pub fn MSG_ReadShort(sb: *mut msg_t) -> c_int;
    pub fn MSG_ReadLong(sb: *mut msg_t) -> c_int;
    pub fn MSG_ReadFloat(sb: *mut msg_t) -> f32;
    pub fn MSG_ReadString(sb: *mut msg_t) -> *mut c_char;
    pub fn MSG_ReadBigString(sb: *mut msg_t) -> *mut c_char;
    pub fn MSG_ReadStringLine(sb: *mut msg_t) -> *mut c_char;
    pub fn MSG_ReadAngle16(sb: *mut msg_t) -> f32;
    pub fn MSG_ReadData(sb: *mut msg_t, buffer: *mut c_void, size: c_int);
}

// Forward declarations for structs - define only the type names here
// The actual definitions are in their respective modules
pub struct usercmd_s;
pub struct entityState_s;
pub struct playerState_s;

// Type aliases for convenience (these should be defined in the q_shared module)
type usercmd_t = usercmd_s;
type entityState_t = entityState_s;
type playerState_t = playerState_s;

extern "C" {
    pub fn MSG_WriteDeltaUsercmd(
        msg: *mut msg_t,
        from: *mut usercmd_s,
        to: *mut usercmd_s,
    );
    pub fn MSG_ReadDeltaUsercmd(msg: *mut msg_t, from: *mut usercmd_s, to: *mut usercmd_s);

    pub fn MSG_WriteDeltaUsercmdKey(msg: *mut msg_t, key: c_int, from: *mut usercmd_t, to: *mut usercmd_t);
    pub fn MSG_ReadDeltaUsercmdKey(msg: *mut msg_t, key: c_int, from: *mut usercmd_t, to: *mut usercmd_t);

    pub fn MSG_WriteDeltaEntity(
        msg: *mut msg_t,
        from: *mut entityState_s,
        to: *mut entityState_s,
        force: c_int,
    );
    pub fn MSG_ReadDeltaEntity(msg: *mut msg_t, from: *mut entityState_t, to: *mut entityState_t, number: c_int);
}

#[cfg(feature = "_ONEBIT_COMBO")]
extern "C" {
    pub fn MSG_WriteDeltaPlayerstate(
        msg: *mut msg_t,
        from: *mut playerState_s,
        to: *mut playerState_s,
        bitComboDelta: *mut c_int,
        bitNumDelta: *mut c_int,
        isVehiclePS: c_int,
    );
}

#[cfg(not(feature = "_ONEBIT_COMBO"))]
extern "C" {
    pub fn MSG_WriteDeltaPlayerstate(
        msg: *mut msg_t,
        from: *mut playerState_s,
        to: *mut playerState_s,
        isVehiclePS: c_int,
    );
}

extern "C" {
    pub fn MSG_ReadDeltaPlayerstate(
        msg: *mut msg_t,
        from: *mut playerState_s,
        to: *mut playerState_s,
        isVehiclePS: c_int,
    );

    pub fn MSG_ReportChangeVectors_f();
}

//============================================================================

/*
==============================================================

NET

==============================================================
*/

pub const PACKET_BACKUP: c_int = 32;   // number of old messages that must be kept on client and
                                       // server for delta comrpession and ping estimation
pub const PACKET_MASK: c_int = PACKET_BACKUP - 1;

pub const MAX_PACKET_USERCMDS: c_int = 32;  // max number of usercmd_t in a packet

pub const PORT_ANY: c_int = -1;

pub const MAX_RELIABLE_COMMANDS: c_int = 128;  // max string commands buffered for restransmit

#[repr(C)]
#[derive(Clone, Copy)]
pub enum netadrtype_t {
    NA_BOT,
    NA_BAD,         // an address lookup failed
    NA_LOOPBACK,
    NA_BROADCAST,
    NA_IP,
    NA_IPX,
    NA_BROADCAST_IPX,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum netsrc_t {
    NS_CLIENT,
    NS_SERVER,
}

#[repr(C)]
pub struct netadr_t {
    pub r#type: netadrtype_t,

    pub ip: [u8; 4],
    pub ipx: [u8; 10],

    pub port: u16,
}

extern "C" {
    pub fn NET_Init();
    pub fn NET_Shutdown();
    pub fn NET_Restart();
    pub fn NET_Config(enableNetworking: c_int);

    pub fn NET_SendPacket(sock: netsrc_t, length: c_int, data: *const c_void, to: netadr_t);
    // NET_OutOfBandPrint and NET_OutOfBandData are variadic, need special handling
    // pub fn NET_OutOfBandPrint(net_socket: netsrc_t, adr: netadr_t, format: *const c_char, ...);
    // pub fn NET_OutOfBandData(sock: netsrc_t, adr: netadr_t, format: *mut u8, len: c_int);

    pub fn NET_CompareAdr(a: netadr_t, b: netadr_t) -> c_int;
    pub fn NET_CompareBaseAdr(a: netadr_t, b: netadr_t) -> c_int;
    pub fn NET_IsLocalAddress(adr: netadr_t) -> c_int;
    pub fn NET_AdrToString(a: netadr_t) -> *const c_char;
    pub fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> c_int;
    pub fn NET_GetLoopPacket(sock: netsrc_t, net_from: *mut netadr_t, net_message: *mut msg_t) -> c_int;
    pub fn NET_Sleep(msec: c_int);
}

pub const MAX_MSGLEN: c_int = 49152;   // max length of a message, which may
                                        // be fragmented into multiple packets

//rww - 6/28/02 - Changed from 16384 to match sof2's. This does seem rather huge, but I guess it doesn't really hurt anything.

pub const MAX_DOWNLOAD_WINDOW: c_int = 8;        // max of eight download frames
pub const MAX_DOWNLOAD_BLKSIZE: c_int = 2048;    // 2048 byte block chunks

/*
Netchan handles packet fragmentation and out of order / duplicate suppression
*/

#[repr(C)]
pub struct netchan_t {
    pub sock: netsrc_t,

    pub dropped: c_int,  // between last packet and previous

    pub remoteAddress: netadr_t,
    pub qport: c_int,    // qport value to write when transmitting

    // sequencing variables
    pub incomingSequence: c_int,
    pub outgoingSequence: c_int,

    // incoming fragment assembly buffer
    pub fragmentSequence: c_int,
    pub fragmentLength: c_int,
    pub fragmentBuffer: [u8; MAX_MSGLEN as usize],

    // outgoing fragment buffer
    // we need to space out the sending of large fragmented messages
    pub unsentFragments: c_int,
    pub unsentFragmentStart: c_int,
    pub unsentLength: c_int,
    pub unsentBuffer: [u8; MAX_MSGLEN as usize],
}

extern "C" {
    pub fn Netchan_Init(qport: c_int);
    pub fn Netchan_Setup(sock: netsrc_t, chan: *mut netchan_t, adr: netadr_t, qport: c_int);

    pub fn Netchan_Transmit(chan: *mut netchan_t, length: c_int, data: *const u8);
    pub fn Netchan_TransmitNextFragment(chan: *mut netchan_t);

    pub fn Netchan_Process(chan: *mut netchan_t, msg: *mut msg_t) -> c_int;
}

/*
==============================================================

PROTOCOL

==============================================================
*/

pub const PROTOCOL_VERSION: c_int = 26;

#[cfg(not(feature = "_XBOX"))]
pub const UPDATE_SERVER_NAME: &[u8] = b"updatejk3.ravensoft.com\0";
#[cfg(not(feature = "_XBOX"))]
pub const MASTER_SERVER_NAME: &[u8] = b"masterjk3.ravensoft.com\0";

#[cfg(all(not(feature = "_XBOX"), feature = "USE_CD_KEY"))]
pub const AUTHORIZE_SERVER_NAME: &[u8] = b"authorizejk3.ravensoft.com\0";

#[cfg(feature = "_XBOX")]
pub const PORT_SERVER: c_int = 1000;
#[cfg(feature = "_XBOX")]
pub const NUM_SERVER_PORTS: c_int = 1;

#[cfg(not(feature = "_XBOX"))]
pub const PORT_MASTER: c_int = 29060;
#[cfg(not(feature = "_XBOX"))]
pub const PORT_UPDATE: c_int = 29061;
//#define	PORT_AUTHORIZE		29062
#[cfg(not(feature = "_XBOX"))]
pub const PORT_SERVER: c_int = 29070;  //...+9 more for multiple servers
#[cfg(not(feature = "_XBOX"))]
pub const NUM_SERVER_PORTS: c_int = 4;  // broadcast scan this many ports after
                                        // PORT_SERVER so a single machine can
                                        // run multiple servers

// the svc_strings[] array in cl_parse.c should mirror this
//
// server to client
//
#[repr(C)]
#[derive(Clone, Copy)]
pub enum svc_ops_e {
    svc_bad,
    svc_nop,
    svc_gamestate,
    svc_configstring,  // [short] [string] only in gamestate messages
    svc_baseline,      // only in gamestate messages
    svc_serverCommand, // [string] to be executed by client game module
    svc_download,      // [short] size [size bytes]
    svc_snapshot,
    svc_setgame,
    svc_mapchange,
    #[cfg(feature = "_XBOX")]
    svc_newpeer,       //jsw//inform current clients about new player
    #[cfg(feature = "_XBOX")]
    svc_removepeer,    //jsw//inform current clients about dying player
    #[cfg(feature = "_XBOX")]
    svc_xbInfo,        //jsw//update client with current server xbOnlineInfo
    svc_EOF,
}

//
// client to server
//
#[repr(C)]
#[derive(Clone, Copy)]
pub enum clc_ops_e {
    clc_bad,
    clc_nop,
    clc_move,          // [[usercmd_t]
    clc_moveNoDelta,   // [[usercmd_t]
    clc_clientCommand, // [string] message
    clc_EOF,
}

/*
==============================================================

VIRTUAL MACHINE

==============================================================
*/

pub struct vm_s;
pub type vm_t = vm_s;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum vmInterpret_t {
    VMI_NATIVE,
    VMI_BYTECODE,
    VMI_COMPILED,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum sharedTraps_t {
    TRAP_MEMSET = 100,
    TRAP_MEMCPY,
    TRAP_STRNCPY,
    TRAP_SIN,
    TRAP_COS,
    TRAP_ATAN2,
    TRAP_SQRT,
    TRAP_MATRIXMULTIPLY,
    TRAP_ANGLEVECTORS,
    TRAP_PERPENDICULARVECTOR,
    TRAP_FLOOR,
    TRAP_CEIL,

    TRAP_TESTPRINTINT,
    TRAP_TESTPRINTFLOAT,

    TRAP_ACOS,
    TRAP_ASIN,
}

extern "C" {
    pub fn VM_Init();
    pub fn VM_Create(
        module: *const c_char,
        systemCalls: Option<extern "C" fn(*mut c_int) -> c_int>,
        interpret: vmInterpret_t,
    ) -> *mut vm_t;
    // module should be bare: "cgame", not "cgame.dll" or "vm/cgame.qvm"

    pub fn VM_Free(vm: *mut vm_t);
    pub fn VM_Clear();
    pub fn VM_Restart(vm: *mut vm_t) -> *mut vm_t;

    // pub fn VM_Call(vm: *mut vm_t, callNum: c_int, ...) -> c_int;

    pub fn VM_Debug(level: c_int);

    pub fn VM_Shifted_Alloc(ptr: *mut *mut c_void, size: c_int);
    pub fn VM_Shifted_Free(ptr: *mut *mut c_void);

    pub fn VM_ArgPtr(intValue: c_int) -> *mut c_void;
    pub fn VM_ExplicitArgPtr(vm: *mut vm_t, intValue: c_int) -> *mut c_void;
}

/*
==============================================================

CMD

Command text buffering and command execution

==============================================================
*/

/*

Any number of commands can be added in a frame, from several different sources.
Most commands come from either keybindings or console line input, but entire text
files can be execed.

*/

extern "C" {
    pub fn Cbuf_Init();
    // allocates an initial text buffer that will grow as needed

    pub fn Cbuf_AddText(text: *const c_char);
    // Adds command text at the end of the buffer, does NOT add a final \n

    pub fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
    // this can be used in place of either Cbuf_AddText or Cbuf_InsertText

    pub fn Cbuf_Execute();
    // Pulls off \n terminated lines of text from the command buffer and sends
    // them through Cmd_ExecuteString.  Stops when the buffer is empty.
    // Normally called once per frame, but may be explicitly invoked.
    // Do not call inside a command function, or current args will be destroyed.
}

//===========================================================================

/*

Command execution takes a null terminated string, breaks it into tokens,
then searches for a command or variable that matches the first token.

*/

pub type xcommand_t = Option<extern "C" fn()>;

extern "C" {
    pub fn Cmd_Init();

    pub fn Cmd_AddCommand(cmd_name: *const c_char, function: xcommand_t);
    // called by the init functions of other parts of the program to
    // register commands and functions to call for them.
    // The cmd_name is referenced later, so it should not be in temp memory
    // if function is NULL, the command will be forwarded to the server
    // as a clc_clientCommand instead of executed locally

    pub fn Cmd_RemoveCommand(cmd_name: *const c_char);

    pub fn Cmd_CommandCompletion(callback: Option<extern "C" fn(*const c_char)>);
    // callback with each valid string

    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_Argv(arg: c_int) -> *mut c_char;
    pub fn Cmd_ArgvBuffer(arg: c_int, buffer: *mut c_char, bufferLength: c_int);
    pub fn Cmd_Args() -> *mut c_char;
    pub fn Cmd_ArgsFrom(arg: c_int) -> *mut c_char;
    pub fn Cmd_ArgsBuffer(buffer: *mut c_char, bufferLength: c_int);
    // The functions that execute commands get their parameters with these
    // functions. Cmd_Argv () will return an empty string, not a NULL
    // if arg > argc, so string operations are allways safe.

    pub fn Cmd_TokenizeString(text: *const c_char);
    // Takes a null terminated string.  Does not need to be /n terminated.
    // breaks the string up into arg tokens.

    pub fn Cmd_ExecuteString(text: *const c_char);
    // Parses a single line of text into arguments and tries to execute it
    // as if it was typed at the console
}

/*
==============================================================

CVAR

==============================================================
*/

/*

cvar_t variables are used to hold scalar or string variables that can be changed
or displayed at the console or prog code as well as accessed directly
in C code.

The user can access cvars from the console in three ways:
r_draworder			prints the current value
r_draworder 0		sets the current value to 0
set r_draworder 0	as above, but creates the cvar if not present

Cvars are restricted from having the same names as commands to keep this
interface from being ambiguous.

The are also occasionally used to communicated information between different
modules of the program.

*/

// Forward declaration - cvar_t is defined elsewhere
pub struct cvar_t;
pub struct vmCvar_t;

extern "C" {
    pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    // creates the variable if it doesn't exist, or returns the existing one
    // if it exists, the value will not be changed, but flags will be ORed in
    // that allows variables to be unarchived without needing bitflags
    // if value is "", the value will not override a previously set value.

    pub fn Cvar_Register(vmCvar: *mut vmCvar_t, varName: *const c_char, defaultValue: *const c_char, flags: c_int);
    // basically a slightly modified Cvar_Get for the interpreted modules

    pub fn Cvar_Update(vmCvar: *mut vmCvar_t);
    // updates an interpreted modules' version of a cvar

    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    // will create the variable with no flags if it doesn't exist

    pub fn Cvar_SetLatched(var_name: *const c_char, value: *const c_char);
    // don't set the cvar immediately

    pub fn Cvar_SetValue(var_name: *const c_char, value: f32);
    // expands value to a string and calls Cvar_Set

    pub fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    pub fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;
    // returns 0 if not defined or non numeric

    pub fn Cvar_VariableString(var_name: *const c_char) -> *mut c_char;
    pub fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
    // returns an empty string if not defined

    pub fn Cvar_CommandCompletion(callback: Option<extern "C" fn(*const c_char)>);
    // callback with each valid string

    pub fn Cvar_Reset(var_name: *const c_char);

    pub fn Cvar_SetCheatState();
    // reset all testing vars to a safe value

    pub fn Cvar_Command() -> c_int;
    // called by Cmd_ExecuteString when Cmd_Argv(0) doesn't match a known
    // command.  Returns true if the command was a variable reference that
    // was handled. (print or change)
}

// Forward declaration for fileHandle_t
pub struct fileHandle_t;

extern "C" {
    pub fn Cvar_WriteVariables(f: *mut fileHandle_t);
    // writes lines containing "set variable value" for all variables
    // with the archive flag set to true.

    pub fn Cvar_Init();

    pub fn Cvar_InfoString(bit: c_int) -> *mut c_char;
    pub fn Cvar_InfoString_Big(bit: c_int) -> *mut c_char;
    // returns an info string containing all the cvars that have the given bit set
    // in their flags ( CVAR_USERINFO, CVAR_SERVERINFO, CVAR_SYSTEMINFO, etc )
    pub fn Cvar_InfoStringBuffer(bit: c_int, buff: *mut c_char, buffsize: c_int);

    pub fn Cvar_Restart_f();
}

extern "C" {
    pub static mut cvar_modifiedFlags: c_int;
}
// whenever a cvar is modifed, its flags will be OR'd into this, so
// a single check can determine if any CVAR_USERINFO, CVAR_SERVERINFO,
// etc, variables have been modified since the last check.  The bit
// can then be cleared to allow another change detection.

/*
==============================================================

FILESYSTEM

No stdio calls should be used by any part of the game, because
we need to deal with all sorts of directory and seperator char
issues.
==============================================================
*/

// referenced flags
// these are in loop specific order so don't change the order
pub const FS_GENERAL_REF: c_int = 0x01;
pub const FS_UI_REF: c_int = 0x02;
pub const FS_CGAME_REF: c_int = 0x04;
pub const FS_QAGAME_REF: c_int = 0x08;
// number of id paks that will never be autodownloaded from base
pub const NUM_ID_PAKS: c_int = 9;

#[cfg(feature = "_XBOX")]
pub const MAX_FILE_HANDLES: c_int = 16;
#[cfg(not(feature = "_XBOX"))]
pub const MAX_FILE_HANDLES: c_int = 64;

extern "C" {
    pub fn FS_Initialized() -> c_int;

    pub fn FS_InitFilesystem();
    pub fn FS_Shutdown(closemfp: c_int);

    pub fn FS_ConditionalRestart(checksumFeed: c_int) -> c_int;
    pub fn FS_Restart(checksumFeed: c_int);
    // shutdown and restart the filesystem so changes to fs_gamedir can take effect

    pub fn FS_ListFiles(
        directory: *const c_char,
        extension: *const c_char,
        numfiles: *mut c_int,
    ) -> *mut *mut c_char;
    // directory should not have either a leading or trailing /
    // if extension is "/", only subdirectories will be returned
    // the returned files will not include any directories or /

    pub fn FS_FreeFileList(fileList: *mut *mut c_char);
    //rwwRMG - changed to fileList to not conflict with list type

    pub fn FS_FileExists(file: *const c_char) -> c_int;

    pub fn FS_LoadStack() -> c_int;

    pub fn FS_GetFileList(
        path: *const c_char,
        extension: *const c_char,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int;
    pub fn FS_GetModList(listbuf: *mut c_char, bufsize: c_int) -> c_int;

    pub fn FS_FOpenFileWrite(qpath: *const c_char) -> *mut fileHandle_t;
    // will properly create any needed paths and deal with seperater character issues

    pub fn FS_filelength(f: *mut fileHandle_t) -> c_int;
    pub fn FS_SV_FOpenFileWrite(filename: *const c_char) -> *mut fileHandle_t;
    pub fn FS_SV_FOpenFileRead(filename: *const c_char, fp: *mut *mut fileHandle_t) -> c_int;
    pub fn FS_SV_Rename(from: *const c_char, to: *const c_char);
    pub fn FS_FOpenFileRead(
        qpath: *const c_char,
        file: *mut *mut fileHandle_t,
        uniqueFILE: c_int,
    ) -> c_int;
    // if uniqueFILE is true, then a new FILE will be fopened even if the file
    // is found in an already open pak file.  If uniqueFILE is false, you must call
    // FS_FCloseFile instead of fclose, otherwise the pak FILE would be improperly closed
    // It is generally safe to always set uniqueFILE to true, because the majority of
    // file IO goes through FS_ReadFile, which Does The Right Thing already.

    pub fn FS_FileIsInPAK(filename: *const c_char, pChecksum: *mut c_int) -> c_int;
    // returns 1 if a file is in the PAK file, otherwise -1

    pub fn FS_Write(buffer: *const c_void, len: c_int, f: *mut fileHandle_t) -> c_int;

    pub fn FS_Read2(buffer: *mut c_void, len: c_int, f: *mut fileHandle_t) -> c_int;
    pub fn FS_Read(buffer: *mut c_void, len: c_int, f: *mut fileHandle_t) -> c_int;
    // properly handles partial reads and reads from other dlls

    pub fn FS_FCloseFile(f: *mut fileHandle_t);
    // note: you can't just fclose from another DLL, due to MS libc issues

    pub fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    // returns the length of the file
    // a null buffer will just return the file length without loading
    // as a quick check for existance. -1 length == not present
    // A 0 byte will always be appended at the end, so string ops are safe.
    // the buffer should be considered read-only, because it may be cached
    // for other uses.

    pub fn FS_ForceFlush(f: *mut fileHandle_t);
    // forces flush on files we're writing to.

    pub fn FS_FreeFile(buffer: *mut c_void);
    // frees the memory returned by FS_ReadFile

    pub fn FS_WriteFile(qpath: *const c_char, buffer: *const c_void, size: c_int);
    // writes a complete file, creating any subdirectories needed

    pub fn FS_FTell(f: *mut fileHandle_t) -> c_int;
    // where are we?

    pub fn FS_Flush(f: *mut fileHandle_t);

    // pub fn FS_Printf(f: *mut fileHandle_t, fmt: *const c_char, ...);
    // like fprintf

    // fsMode_t is defined elsewhere
    pub fn FS_FOpenFileByMode(qpath: *const c_char, f: *mut *mut fileHandle_t, mode: c_int) -> c_int;
    // opens a file for reading, writing, or appending depending on the value of mode

    pub fn FS_Seek(f: *mut fileHandle_t, offset: libc::c_long, origin: c_int) -> c_int;
    // seek on a file (doesn't work for zip files!!!!!!!!)

    pub fn FS_FilenameCompare(s1: *const c_char, s2: *const c_char) -> c_int;

    pub fn FS_GamePureChecksum() -> *const c_char;
    // Returns the checksum of the pk3 from which the server loaded the qagame.qvm

    pub fn FS_LoadedPakNames() -> *const c_char;
    pub fn FS_LoadedPakChecksums() -> *const c_char;
    pub fn FS_LoadedPakPureChecksums() -> *const c_char;
    // Returns a space separated string containing the checksums of all loaded pk3 files.
    // Servers with sv_pure set will get this string and pass it to clients.

    pub fn FS_ReferencedPakNames() -> *const c_char;
    pub fn FS_ReferencedPakChecksums() -> *const c_char;
    pub fn FS_ReferencedPakPureChecksums() -> *const c_char;
    // Returns a space separated string containing the checksums of all loaded
    // AND referenced pk3 files. Servers with sv_pure set will get this string
    // back from clients for pure validation

    pub fn FS_ClearPakReferences(flags: c_int);
    // clears referenced booleans on loaded pk3s

    pub fn FS_PureServerSetReferencedPaks(pakSums: *const c_char, pakNames: *const c_char);
    pub fn FS_PureServerSetLoadedPaks(pakSums: *const c_char, pakNames: *const c_char);
    // If the string is empty, all data sources will be allowed.
    // If not empty, only pk3 files that match one of the space
    // separated checksums will be checked for files, with the
    // sole exception of .cfg files.

    pub fn FS_idPak(pak: *mut c_char, base: *mut c_char) -> c_int;
    pub fn FS_ComparePaks(neededpaks: *mut c_char, len: c_int, dlstring: c_int) -> c_int;
    pub fn FS_Rename(from: *const c_char, to: *const c_char);
}

/*
==============================================================

MISC

==============================================================
*/

// NOTE NOTE NOTE!!!!!!!!!!!!!
//
// Any CPUID_XXXX defined as higher than CPUID_INTEL_MMX *must* have MMX support (eg like CPUID_AMD_3DNOW (0x30) has),
//	this allows convenient MMX capability checking. If you for some reason want to support some new processor that does
//	*NOT* have MMX (yeah, right), then define it as a lower number. -slc
//
// ( These values are returned by Sys_GetProcessorId )
//
pub const CPUID_GENERIC: c_int = 0;  // any unrecognized processor

pub const CPUID_AXP: c_int = 0x10;

pub const CPUID_INTEL_UNSUPPORTED: c_int = 0x20;  // Intel 386/486
pub const CPUID_INTEL_PENTIUM: c_int = 0x21;     // Intel Pentium or PPro
pub const CPUID_INTEL_MMX: c_int = 0x22;         // Intel Pentium/MMX or P2/MMX
pub const CPUID_INTEL_KATMAI: c_int = 0x23;      // Intel Katmai
pub const CPUID_INTEL_WILLIAMETTE: c_int = 0x24; // Intel Williamette

pub const CPUID_AMD_3DNOW: c_int = 0x30;  // AMD K6 3DNOW!
//
//==========================================================

#[inline]
pub fn RoundUp(n: c_int, m: c_int) -> c_int {
    n + ((m as u32 - (n as u32) % (m as u32)) as c_int)
}

#[inline]
pub fn RoundDown(n: c_int, m: c_int) -> c_int {
    n - ((n as u32 % m as u32) as c_int)
}

extern "C" {
    pub fn CopyString(in_: *const c_char) -> *mut c_char;
    pub fn Info_Print(s: *const c_char);

    pub fn Com_BeginRedirect(buffer: *mut c_char, buffersize: c_int, flush: Option<extern "C" fn(*mut c_char)>);
    pub fn Com_EndRedirect();
    // pub fn Com_Printf(fmt: *const c_char, ...);
    // pub fn Com_DPrintf(fmt: *const c_char, ...);
    // pub fn Com_OPrintf(fmt: *const c_char, ...); // Outputs to the VC / Windows Debug window (only in debug compile)
    // pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Com_Quit_f();
    pub fn Com_EventLoop() -> c_int;
    pub fn Com_Milliseconds() -> c_int;  // will be journaled properly
    pub fn Com_BlockChecksum(buffer: *const c_void, length: c_int) -> c_uint;
    pub fn Com_BlockChecksumKey(buffer: *mut c_void, length: c_int, key: c_int) -> c_uint;
    pub fn Com_HashKey(string: *mut c_char, maxlen: c_int) -> c_int;
    pub fn Com_Filter(filter: *mut c_char, name: *mut c_char, casesensitive: c_int) -> c_int;
    pub fn Com_FilterPath(filter: *mut c_char, name: *mut c_char, casesensitive: c_int) -> c_int;
    pub fn Com_RealTime(qtime: *mut qtime_t) -> c_int;
    pub fn Com_SafeMode() -> c_int;

    pub fn Com_StartupVariable(match_: *const c_char);
    // checks for and removes command line "+set var arg" constructs
    // if match is NULL, all set commands will be executed, otherwise
    // only a set with the exact name.  Only used during startup.
}

extern "C" {
    pub static mut com_developer: *mut cvar_t;
    pub static mut com_vmdebug: *mut cvar_t;
    pub static mut com_dedicated: *mut cvar_t;
    pub static mut com_speeds: *mut cvar_t;
    pub static mut com_timescale: *mut cvar_t;
    pub static mut com_sv_running: *mut cvar_t;
    pub static mut com_cl_running: *mut cvar_t;
    pub static mut com_viewlog: *mut cvar_t;  // 0 = hidden, 1 = visible, 2 = minimized
    pub static mut com_version: *mut cvar_t;
    pub static mut com_blood: *mut cvar_t;
    pub static mut com_buildScript: *mut cvar_t;  // for building release pak files
    pub static mut com_journal: *mut cvar_t;
    pub static mut com_cameraMode: *mut cvar_t;

    pub static mut com_optvehtrace: *mut cvar_t;

    #[cfg(feature = "G2_PERFORMANCE_ANALYSIS")]
    pub static mut com_G2Report: *mut cvar_t;

    pub static mut com_RMG: *mut cvar_t;

    // both client and server must agree to pause
    pub static mut cl_paused: *mut cvar_t;
    pub static mut sv_paused: *mut cvar_t;

    // com_speeds times
    pub static mut time_game: c_int;
    pub static mut time_frontend: c_int;
    pub static mut time_backend: c_int;  // renderer backend time

    pub static mut com_frameTime: c_int;
    pub static mut com_frameMsec: c_int;

    pub static mut com_errorEntered: c_int;

    #[cfg(not(feature = "_XBOX"))]
    pub static mut logfile: *mut fileHandle_t;
    #[cfg(not(feature = "_XBOX"))]
    pub static mut com_journalFile: *mut fileHandle_t;
    #[cfg(not(feature = "_XBOX"))]
    pub static mut com_journalDataFile: *mut fileHandle_t;
}

// Forward declarations for memory and type definitions
pub struct memtag_t;
pub struct qtime_t;
pub struct CGenericParser2;

#[cfg(debug_assertions)]
pub const DEBUG_ZONE_ALLOCS: bool = true;
#[cfg(not(debug_assertions))]
pub const DEBUG_ZONE_ALLOCS: bool = false;

/*
--- low memory ----
server vm
server clipmap
---mark---
renderer initialization (shaders, etc)
UI vm
cgame vm
renderer map
renderer models

---free---

temp file loading
--- high memory ---

*/

extern "C" {
    // pub fn Z_Malloc(iSize: c_int, eTag: memtag_t, bZeroit: qboolean, iAlign: c_int) -> *mut c_void;
    // pub fn S_Malloc(iSize: c_int) -> *mut c_void;
    pub fn Z_MorphMallocTag(pvBuffer: *mut c_void, eDesiredTag: c_int);
    pub fn Z_Validate();
    pub fn Z_MemSize(eTag: c_int) -> c_int;
    pub fn Z_TagFree(eTag: c_int);
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Z_Size(pvAddress: *mut c_void) -> c_int;
    pub fn Com_InitZoneMemory();
    pub fn Com_InitHunkMemory();
    pub fn Com_ShutdownZoneMemory();
    pub fn Com_ShutdownHunkMemory();

    pub fn Hunk_Clear();
    pub fn Hunk_ClearToMark();
    pub fn Hunk_SetMark();
    pub fn Hunk_CheckMark() -> c_int;
    pub fn Hunk_ClearTempMemory();
    pub fn Hunk_AllocateTempMemory(size: c_int) -> *mut c_void;
    pub fn Hunk_FreeTempMemory(buf: *mut c_void);
    pub fn Hunk_MemoryRemaining() -> c_int;
    pub fn Hunk_Log();
    pub fn Hunk_Trash();

    pub fn Com_TouchMemory();

    // commandLine should not include the executable name (argv[0])
    pub fn Com_Init(commandLine: *mut c_char);
    pub fn Com_Frame();
    pub fn Com_Shutdown();
    //rwwRMG: Inserted:
    pub fn Com_ParseTextFile(file: *const c_char, parser: *mut CGenericParser2, cleanFirst: c_int) -> c_int;
    pub fn Com_ParseTextFileDestroy(parser: *mut CGenericParser2);
}

/*
==============================================================

CLIENT / SERVER SYSTEMS

==============================================================
*/

//
// client interface
//
extern "C" {
    pub fn CL_InitKeyCommands();
    // the keyboard binding interface must be setup before execing
    // config files, but the rest of client startup will happen later

    pub fn CL_Init();
    pub fn CL_Disconnect(showMainMenu: c_int);
    pub fn CL_Shutdown();
    pub fn CL_Frame(msec: c_int);
    pub fn CL_GameCommand() -> c_int;
    pub fn CL_KeyEvent(key: c_int, down: c_int, time: c_uint);

    pub fn CL_CharEvent(key: c_int);
    // char events are for field typing, not game control

    pub fn CL_MouseEvent(dx: c_int, dy: c_int, time: c_int);

    pub fn CL_JoystickEvent(axis: c_int, value: c_int, time: c_int);

    pub fn CL_PacketEvent(from: netadr_t, msg: *mut msg_t);

    pub fn CL_ConsolePrint(text: *const c_char, silent: c_int);

    pub fn CL_MapLoading();
    // do a screen update before starting to load a map
    // when the server is going to load a new map, the entire hunk
    // will be cleared, so the client must shutdown cgame, ui, and
    // the renderer

    pub fn CL_ForwardCommandToServer(string: *const c_char);
    // adds the current command line as a clc_clientCommand to the client message.
    // things like godmode, noclip, etc, are commands directed to the server,
    // so when they are typed in at the console, they will need to be forwarded.

    pub fn CL_ShutdownAll();
    // shutdown all the client stuff

    pub fn CL_FlushMemory();
    // dump all memory on an error

    pub fn CL_StartHunkUsers();
    // start all the client stuff using the hunk

    pub fn Key_WriteBindings(f: *mut fileHandle_t);
    // for writing the config files

    pub fn S_ClearSoundBuffer();
    // call before filesystem access

    pub fn SCR_DebugGraph(value: f32, color: c_int);  // FIXME: move logging to common?
}

//
// server interface
//
extern "C" {
    pub fn SV_Init();
    pub fn SV_Shutdown(finalmsg: *mut c_char);
    pub fn SV_Frame(msec: c_int);
    pub fn SV_PacketEvent(from: netadr_t, msg: *mut msg_t);
    pub fn SV_GameCommand() -> c_int;
}

//
// UI interface
//
extern "C" {
    pub fn UI_GameCommand() -> c_int;
    pub fn UI_usesUniqueCDKey() -> c_int;
}

/*
==============================================================

NON-PORTABLE SYSTEM SERVICES

==============================================================
*/

#[repr(C)]
#[derive(Clone, Copy)]
pub enum joystickAxis_t {
    AXIS_SIDE,
    AXIS_FORWARD,
    AXIS_UP,
    AXIS_ROLL,
    AXIS_YAW,
    AXIS_PITCH,
    MAX_JOYSTICK_AXIS,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum sysEventType_t {
    // bk001129 - make sure SE_NONE is zero
    SE_NONE = 0,           // evTime is still valid
    SE_KEY,                // evValue is a key code, evValue2 is the down flag
    SE_CHAR,               // evValue is an ascii char
    SE_MOUSE,              // evValue and evValue2 are reletive signed x / y moves
    SE_JOYSTICK_AXIS,      // evValue is an axis number and evValue2 is the current state (-127 to 127)
    SE_CONSOLE,            // evPtr is a char*
    SE_PACKET,             // evPtr is a netadr_t followed by data bytes to evPtrLength
}

#[repr(C)]
pub struct sysEvent_t {
    pub evTime: c_int,
    pub evType: sysEventType_t,
    pub evValue: c_int,
    pub evValue2: c_int,
    pub evPtrLength: c_int,  // bytes of data pointed to by evPtr, for journaling
    pub evPtr: *mut c_void,  // this must be manually freed if not NULL
}

extern "C" {
    pub fn Sys_GetEvent() -> sysEvent_t;

    pub fn Sys_Init();

    // general development dll loading for virtual machine testing
    pub fn Sys_LoadDll(
        name: *const c_char,
        entryPoint: *mut Option<extern "C" fn(c_int) -> c_int>,
        systemcalls: Option<extern "C" fn(c_int) -> c_int>,
    ) -> *mut c_void;
    pub fn Sys_UnloadDll(dllHandle: *mut c_void);

    pub fn Sys_UnloadGame();
    pub fn Sys_GetGameAPI(parms: *mut c_void) -> *mut c_void;

    pub fn Sys_UnloadCGame();
    pub fn Sys_GetCGameAPI() -> *mut c_void;

    pub fn Sys_UnloadUI();
    pub fn Sys_GetUIAPI() -> *mut c_void;

    //bot libraries
    pub fn Sys_UnloadBotLib();
    pub fn Sys_GetBotLibAPI(parms: *mut c_void) -> *mut c_void;

    pub fn Sys_GetCurrentUser() -> *mut c_char;

    // pub fn Sys_Error(error: *const c_char, ...);
    pub fn Sys_Quit();
    pub fn Sys_GetClipboardData() -> *mut c_char;  // note that this isn't journaled...

    pub fn Sys_Print(msg: *const c_char);
    #[cfg(feature = "_XBOX")]
    pub fn Sys_Log(file: *const c_char, msg: *const c_char);
    #[cfg(feature = "_XBOX")]
    pub fn Sys_Log_buf(file: *const c_char, buffer: *const c_void, size: c_int, flush: c_int);

    // Sys_Milliseconds should only be used for profiling purposes,
    // any game related timing information should come from event timestamps
    pub fn Sys_Milliseconds(baseTime: c_int) -> c_int;

    #[cfg(target_os = "linux")]
    extern "C" {
        pub fn Sys_SnapVector(v: *mut f32);
    }

    #[cfg(not(target_os = "linux"))]
    pub fn Sys_SnapVector(v: *mut f32);

    // the system console is shown when a dedicated server is running
    pub fn Sys_DisplaySystemConsole(show: c_int);

    pub fn Sys_GetProcessorId() -> c_int;
    pub fn Sys_GetCPUSpeed() -> c_int;
    pub fn Sys_GetPhysicalMemory() -> c_int;

    pub fn Sys_BeginStreamedFile(f: *mut fileHandle_t, readahead: c_int);
    pub fn Sys_EndStreamedFile(f: *mut fileHandle_t);
    pub fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: *mut fileHandle_t) -> c_int;
    pub fn Sys_StreamSeek(f: *mut fileHandle_t, offset: c_int, origin: c_int);

    pub fn Sys_ShowConsole(level: c_int, quitOnClose: c_int);
    pub fn Sys_SetErrorText(text: *const c_char);

    pub fn Sys_SendPacket(length: c_int, data: *const c_void, to: netadr_t);
    #[cfg(feature = "_XBOX")]
    pub fn Sys_SendVoicePacket(length: c_int, data: *const c_void, to: netadr_t);

    pub fn Sys_StringToAdr(s: *const c_char, a: *mut netadr_t) -> c_int;
    //Does NOT parse port numbers, only base addresses.

    pub fn Sys_IsLANAddress(adr: netadr_t) -> c_int;
    pub fn Sys_ShowIP();

    pub fn Sys_CheckCD() -> c_int;

    pub fn Sys_Mkdir(path: *const c_char);
    pub fn Sys_Cwd() -> *mut c_char;
    pub fn Sys_SetDefaultCDPath(path: *const c_char);
    pub fn Sys_DefaultCDPath() -> *mut c_char;
    pub fn Sys_SetDefaultInstallPath(path: *const c_char);
    pub fn Sys_DefaultInstallPath() -> *mut c_char;
    pub fn Sys_SetDefaultHomePath(path: *const c_char);
    pub fn Sys_DefaultHomePath() -> *mut c_char;
    pub fn Sys_DefaultBasePath() -> *mut c_char;

    pub fn Sys_ListFiles(
        directory: *const c_char,
        extension: *const c_char,
        filter: *mut c_char,
        numfiles: *mut c_int,
        wantsubs: c_int,
    ) -> *mut *mut c_char;
    pub fn Sys_FreeFileList(fileList: *mut *mut c_char);
    //rwwRMG - changed to fileList to not conflict with list type

    pub fn Sys_BeginProfiling();
    pub fn Sys_EndProfiling();

    pub fn Sys_FunctionCmp(f1: *mut c_void, f2: *mut c_void) -> c_int;
    pub fn Sys_FunctionCheckSum(f1: *mut c_void) -> c_int;

    pub fn Sys_LowPhysicalMemory() -> c_int;
    pub fn Sys_ProcessorCount() -> c_uint;

    pub fn Sys_MonkeyShouldBeSpanked() -> c_int;
}

/* This is based on the Adaptive Huffman algorithm described in Sayood's Data
 * Compression book.  The ranks are not actually stored, but implicitly defined
 * by the location of a node within a doubly-linked list */

pub const NYT: c_int = 256;  /* NYT = Not Yet Transmitted; HMAX = 256 */
pub const INTERNAL_NODE: c_int = 257;  /* HMAX+1 */

#[repr(C)]
pub struct nodetype {
    pub left: *mut nodetype,
    pub right: *mut nodetype,
    pub parent: *mut nodetype,  /* tree structure */
    pub next: *mut nodetype,
    pub prev: *mut nodetype,  /* doubly-linked list */
    pub head: *mut *mut nodetype,  /* highest ranked node in block */
    pub weight: c_int,
    pub symbol: c_int,
}

pub type node_t = nodetype;

pub const HMAX: c_int = 256;  /* Maximum symbol */

#[repr(C)]
pub struct huff_t {
    pub blocNode: c_int,
    pub blocPtrs: c_int,

    pub tree: *mut node_t,
    pub lhead: *mut node_t,
    pub ltail: *mut node_t,
    pub loc: [*mut node_t; 257],  /* HMAX+1 */
    pub freelist: *mut *mut node_t,

    pub nodeList: [node_t; 768],
    pub nodePtrs: [*mut node_t; 768],
}

#[repr(C)]
pub struct huffman_t {
    pub compressor: huff_t,
    pub decompressor: huff_t,
}

extern "C" {
    pub fn Huff_Compress(buf: *mut msg_t, offset: c_int);
    pub fn Huff_Decompress(buf: *mut msg_t, offset: c_int);
    pub fn Huff_Init(huff: *mut huffman_t);
    pub fn Huff_addRef(huff: *mut huff_t, ch: u8);
    pub fn Huff_Receive(node: *mut node_t, ch: *mut c_int, fin: *mut u8) -> c_int;
    pub fn Huff_transmit(huff: *mut huff_t, ch: c_int, fout: *mut u8);
    pub fn Huff_offsetReceive(node: *mut node_t, ch: *mut c_int, fin: *mut u8, offset: *mut c_int) -> c_int;
    pub fn Huff_offsetTransmit(huff: *mut huff_t, ch: c_int, fout: *mut u8, offset: *mut c_int);
    pub fn Huff_putBit(bit: c_int, fout: *mut u8, offset: *mut c_int);
    pub fn Huff_getBit(fout: *mut u8, offset: *mut c_int) -> c_int;
}

extern "C" {
    pub static mut clientHuffTables: huffman_t;
}

pub const SV_ENCODE_START: c_int = 4;
pub const SV_DECODE_START: c_int = 12;
pub const CL_ENCODE_START: c_int = 12;
pub const CL_DECODE_START: c_int = 4;

#[inline]
pub fn Round(value: f32) -> c_int {
    (value + 0.5).floor() as c_int
}

#[cfg(feature = "_XBOX")]
pub struct Lump {
    pub data: *mut c_void,
    pub len: c_int,
}

#[cfg(feature = "_XBOX")]
impl Lump {
    // Lump() : data(NULL), len(0) {}
    pub fn new() -> Self {
        Lump {
            data: core::ptr::null_mut(),
            len: 0,
        }
    }

    // void load(const char* map, const char* lump)
    pub fn load(&mut self, _map: *const c_char, _lump: *const c_char) {
        // self.clear();
        // char path[MAX_QPATH];
        // Com_sprintf(path, MAX_QPATH, "%s/%s.mle", map, lump);
        // len = FS_ReadFile(path, &data);
        // if (len < 0) len = 0;
    }

    // void clear(void)
    pub fn clear(&mut self) {
        if !self.data.is_null() {
            // FS_FreeFile(self.data);
            self.data = core::ptr::null_mut();
        }
    }
}
