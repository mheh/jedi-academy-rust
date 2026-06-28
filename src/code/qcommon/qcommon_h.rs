// qcommon.h -- definitions common between client and server, but not game.or ref modules

use core::ffi::{c_int, c_char, c_void, c_short, c_float};

// Type definitions needed early
pub type qboolean = c_int;
pub type fileHandle_t = c_int;
pub type entityState_t = c_void;
pub type cvar_t = c_void;
pub type vmCvar_t = c_void;
pub type memtag_t = c_int;
pub type fsMode_t = c_int;

// some zone mem debugging stuff
// #ifndef FINAL_BUILD
//	#ifdef _DEBUG
//	//
//	// both of these should be REM'd unless you specifically need them...
//	//
//	//#define DEBUG_ZONE_ALLOCS			// adds __FILE__ and __LINE__ info to zone blocks, to see who's leaking
//	//#define DETAILED_ZONE_DEBUG_CODE	// this slows things down a LOT, and is only for tracking nasty double-freeing Z_Malloc bugs
//	#endif
// #endif


//============================================================================

//
// msg.c
//
#[repr(C)]
pub struct msg_t {
	pub allowoverflow: qboolean,	// if false, do a Com_Error
	pub overflowed: qboolean,		// set to true if the buffer size failed (with allowoverflow set)
	pub data: *mut u8,
	pub maxsize: c_int,
	pub cursize: c_int,
	pub readcount: c_int,
	pub bit: c_int,				// for bitwise reads and writes
}

extern "C" {
	pub fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);
	pub fn MSG_Clear(buf: *mut msg_t);
	pub fn MSG_GetSpace(buf: *mut msg_t, length: c_int) -> *mut c_void;
	pub fn MSG_WriteData(buf: *mut msg_t, data: *const c_void, length: c_int);
}


// Forward declarations for opaque structs
// struct usercmd_s;
// struct entityState_s;
// struct playerState_s;

extern "C" {
	pub fn MSG_WriteBits(msg: *mut msg_t, value: c_int, bits: c_int);

	pub fn MSG_WriteByte(sb: *mut msg_t, c: c_int);
	pub fn MSG_WriteShort(sb: *mut msg_t, c: c_int);
	pub fn MSG_WriteLong(sb: *mut msg_t, c: c_int);
	pub fn MSG_WriteString(sb: *mut msg_t, s: *const c_char);

	pub fn MSG_BeginReading(sb: *mut msg_t);

	pub fn MSG_ReadBits(msg: *mut msg_t, bits: c_int) -> c_int;

	pub fn MSG_ReadByte(sb: *mut msg_t) -> c_int;
	pub fn MSG_ReadShort(sb: *mut msg_t) -> c_int;
	pub fn MSG_ReadLong(sb: *mut msg_t) -> c_int;
	pub fn MSG_ReadString(sb: *mut msg_t) -> *mut c_char;
	pub fn MSG_ReadStringLine(sb: *mut msg_t) -> *mut c_char;
	pub fn MSG_ReadData(sb: *mut msg_t, buffer: *mut c_void, size: c_int);


	pub fn MSG_WriteDeltaUsercmd(msg: *mut msg_t, from: *mut c_void, to: *mut c_void);
	pub fn MSG_ReadDeltaUsercmd(msg: *mut msg_t, from: *mut c_void, to: *mut c_void);

	pub fn MSG_WriteDeltaEntity(msg: *mut msg_t, from: *mut c_void, to: *mut c_void, force: qboolean);
	pub fn MSG_ReadDeltaEntity(msg: *mut msg_t, from: *mut c_void, to: *mut c_void, number: c_int);
	pub fn MSG_ReadEntity(msg: *mut msg_t, to: *mut c_void);
	pub fn MSG_WriteEntity(msg: *mut msg_t, to: *mut c_void, removeNum: c_int);

	pub fn MSG_WriteDeltaPlayerstate(msg: *mut msg_t, from: *mut c_void, to: *mut c_void);
	pub fn MSG_ReadDeltaPlayerstate(msg: *mut msg_t, from: *mut c_void, to: *mut c_void);
}


//============================================================================

// #ifdef _M_IX86
// //
// // optimised stuff for Intel, since most of our data is in that format anyway...
// //
extern "C" {
	pub fn BigShort(l: c_short) -> c_short;
	pub fn BigLong(l: c_int) -> c_int;
	pub fn BigFloat(l: c_float) -> c_float;
}
// #define LittleShort(l) l
// #define LittleLong(l) l
// #define LittleFloat(l) l
// //
// #else
// //
// // standard smart-swap code...
// //
// extern	short	BigShort (short l);
// extern	short	LittleShort (short l);
// extern	int		BigLong (int l);
// extern	int		LittleLong (int l);
// extern	float	BigFloat (float l);
// extern	float	LittleFloat (float l);
// //
// #endif

#[inline]
pub fn LittleShort(l: c_short) -> c_short {
	l
}

#[inline]
pub fn LittleLong(l: c_int) -> c_int {
	l
}

#[inline]
pub fn LittleFloat(l: c_float) -> c_float {
	l
}


/*
==============================================================

NET

==============================================================
*/

// #ifdef _XBOX
// #define PACKET_BACKUP	2
// #else
pub const PACKET_BACKUP: c_int = 16;	// number of old messages that must be kept on client and
// #endif						// server for delta comrpession and ping estimation
pub const PACKET_MASK: c_int = PACKET_BACKUP - 1;

pub const MAX_PACKET_USERCMDS: c_int = 32;		// max number of usercmd_t in a packet

pub const PORT_ANY: c_int = -1;

pub const MAX_RELIABLE_COMMANDS: c_int = 64;			// max string commands buffered for restransmit

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum netadrtype_t {
	NA_BAD = 0,					// an address lookup failed
	NA_LOOPBACK = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum netsrc_t {
	NS_CLIENT = 0,
	NS_SERVER = 1,
}

#[repr(C)]
pub struct netadr_t {
	pub type_: netadrtype_t,

	pub port: u16,
}

extern "C" {
	pub fn NET_SendPacket(sock: netsrc_t, length: c_int, data: *const c_void, to: netadr_t);
	pub fn NET_OutOfBandPrint(net_socket: netsrc_t, adr: netadr_t, format: *const c_char, ...);

	pub fn NET_CompareAdr(a: netadr_t, b: netadr_t) -> qboolean;
	pub fn NET_CompareBaseAdr(a: netadr_t, b: netadr_t) -> qboolean;
	pub fn NET_IsLocalAddress(adr: netadr_t) -> qboolean;
	pub fn NET_IsLANAddress(adr: netadr_t) -> qboolean;
	pub fn NET_AdrToString(a: netadr_t) -> *const c_char;
	pub fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean;
	pub fn NET_GetLoopPacket(sock: netsrc_t, net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean;
}


pub const MAX_MSGLEN: c_int = 1 * 17408;		// max length of a message, which may
//#define	MAX_MSGLEN				(3*16384)		// max length of a message, which may
												// be fragmented into multiple packets


/*
Netchan handles packet fragmentation and out of order / duplicate suppression
*/

#[repr(C)]
pub struct netchan_t {
	pub sock: netsrc_t,

	pub dropped: c_int,			// between last packet and previous

	pub remoteAddress: netadr_t,
	pub qport: c_int,				// qport value to write when transmitting

	// sequencing variables
	pub incomingSequence: c_int,
	pub incomingAcknowledged: c_int,

	pub outgoingSequence: c_int,

	// incoming fragment assembly buffer
	pub fragmentSequence: c_int,
	pub fragmentLength: c_int,
	pub fragmentBuffer: [u8; 17408],	// MAX_MSGLEN
}

extern "C" {
	pub fn Netchan_Init(qport: c_int);
	pub fn Netchan_Setup(sock: netsrc_t, chan: *mut netchan_t, adr: netadr_t, qport: c_int);

	pub fn Netchan_Transmit(chan: *mut netchan_t, length: c_int, data: *const u8);
	pub fn Netchan_Process(chan: *mut netchan_t, msg: *mut msg_t) -> qboolean;
}


/*
==============================================================

PROTOCOL

==============================================================
*/

pub const PROTOCOL_VERSION: c_int = 40;

pub const PORT_SERVER: c_int = 27960;

// the svc_strings[] array in cl_parse.c should mirror this
//
// server to client
//
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum svc_ops_e {
	svc_bad = 0,
	svc_nop = 1,
	svc_gamestate = 2,
	svc_configstring = 3,			// [short] [string] only in gamestate messages
	svc_baseline = 4,				// only in gamestate messages
	svc_serverCommand = 5,			// [string] to be executed by client game module
	svc_download = 6,				// [short] size [size bytes]
	svc_snapshot = 7,
}


//
// client to server
//
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum clc_ops_e {
	clc_bad = 0,
	clc_nop = 1,
	clc_move = 2,				// [[usercmd_t]
	clc_clientCommand = 3,		// [string] message
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

pub type xcommand_t = extern "C" fn();

extern "C" {
	pub fn Cmd_Init();

	pub fn Cmd_AddCommand(cmd_name: *const c_char, function: Option<xcommand_t>);
	// called by the init functions of other parts of the program to
	// register commands and functions to call for them.
	// The cmd_name is referenced later, so it should not be in temp memory
	// if function is NULL, the command will be forwarded to the server
	// as a clc_clientCommand instead of executed locally

	pub fn Cmd_RemoveCommand(cmd_name: *const c_char);

	pub fn Cmd_CompleteCommand(partial: *const c_char) -> *mut c_char;
	// attempts to match a partial command for automatic command line completion
	// returns NULL if nothing fits

	pub fn Cmd_Argc() -> c_int;
	pub fn Cmd_Argv(arg: c_int) -> *mut c_char;
	pub fn Cmd_ArgvBuffer(arg: c_int, buffer: *mut c_char, bufferLength: c_int);
	pub fn Cmd_Args() -> *mut c_char;
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

	pub fn Cvar_SetValue(var_name: *const c_char, value: c_float);
	// expands value to a string and calls Cvar_Set

	pub fn Cvar_VariableValue(var_name: *const c_char) -> c_float;
	pub fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;
	// returns 0 if not defined or non numeric

	pub fn Cvar_VariableString(var_name: *const c_char) -> *mut c_char;
	pub fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
	// returns an empty string if not defined

	pub fn Cvar_CompleteVariable(partial: *const c_char) -> *mut c_char;
	// attempts to match a partial variable name for command line completion
	// returns NULL if nothing fits

	pub fn Cvar_Reset(var_name: *const c_char);

	pub fn Cvar_SetCheatState();
	// reset all testing vars to a safe value

	pub fn Cvar_Command() -> qboolean;
	// called by Cmd_ExecuteString when Cmd_Argv(0) doesn't match a known
	// command.  Returns true if the command was a variable reference that
	// was handled. (print or change)

	pub fn Cvar_WriteVariables(f: fileHandle_t);
	// writes lines containing "set variable value" for all variables
	// with the archive flag set to true.

	pub fn Cvar_Init();

	pub fn Cvar_InfoString(bit: c_int) -> *mut c_char;
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

extern "C" {
	pub fn FS_Initialized() -> qboolean;

	pub fn FS_InitFilesystem();

	pub fn FS_ListFiles(directory: *const c_char, extension: *const c_char, numfiles: *mut c_int) -> *mut *mut c_char;
	// directory should not have either a leading or trailing /
	// if extension is "/", only subdirectories will be returned
	// the returned files will not include any directories or /

	pub fn FS_FreeFileList(filelist: *mut *mut c_char);

	pub fn FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int;

	pub fn FS_FOpenFileWrite(qpath: *const c_char) -> fileHandle_t;
	// will properly create any needed paths and deal with seperater character issues

	pub fn FS_FOpenFileAppend(filename: *const c_char) -> fileHandle_t;	// this was present already, but no public proto

	pub fn FS_GetExtendedInfo_FOpenFileRead(filename: *const c_char, ppsFilename: *mut *mut c_char, piOffset: *mut c_int) -> qboolean;
	//return value is success of opening file, then ppsFilename and piOffset are valid

	pub fn FS_FOpenFileRead(qpath: *const c_char, file: *mut fileHandle_t, uniqueFILE: qboolean) -> c_int;
	// if uniqueFILE is true, then a new FILE will be fopened even if the file
	// is found in an already open pak file.  If uniqueFILE is false, you must call
	// FS_FCloseFile instead of fclose, otherwise the pak FILE would be improperly closed
	// It is generally safe to always set uniqueFILE to true, because the majority of
	// file IO goes through FS_ReadFile, which Does The Right Thing already.

	pub fn FS_FileIsInPAK(filename: *const c_char) -> c_int;
	// returns 1 if a file is in the PAK file, otherwise -1

	pub fn FS_Write(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int;

	pub fn FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int;
	// properly handles partial reads and reads from other dlls

	pub fn FS_FCloseFile(f: fileHandle_t);
	// note: you can't just fclose from another DLL, due to MS libc issues

	pub fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
	// returns the length of the file
	// a null buffer will just return the file length without loading
	// as a quick check for existance. -1 length == not present
	// A 0 byte will always be appended at the end, so string ops are safe.
	// the buffer should be considered read-only, because it may be cached
	// for other uses.

	pub fn FS_ForceFlush(f: fileHandle_t);
	// forces flush on files we're writing to.

	pub fn FS_FreeFile(buffer: *mut c_void);
	// frees the memory returned by FS_ReadFile

	pub fn FS_WriteFile(qpath: *const c_char, buffer: *const c_void, size: c_int);
	// writes a complete file, creating any subdirectories needed

	pub fn FS_filelength(f: fileHandle_t) -> c_int;
	// doesn't work for files that are opened from a pack file

	pub fn FS_FTell(f: fileHandle_t) -> c_int;
	// where are we?

	pub fn FS_Flush(f: fileHandle_t);

	pub fn FS_Printf(f: fileHandle_t, fmt: *const c_char, ...);
	// like fprintf

	pub fn FS_FOpenFileByMode(qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t) -> c_int;
	// opens a file for reading, writing, or appending depending on the value of mode

	pub fn FS_Seek(f: fileHandle_t, offset: i64, origin: c_int) -> c_int;
	// seek on a file (doesn't work for zip files!!!!!!!!)


	// These 2 are generally only used by the save games, filenames are local (eg "saves/blah.sav")
	//
	pub fn FS_DeleteUserGenFile(filename: *const c_char);
	pub fn FS_MoveUserGenFile(filename_src: *const c_char, filename_dst: *const c_char) -> qboolean;
}

/*
==============================================================

MISC

==============================================================
*/

//==========================================================
//
// NOTE NOTE NOTE!!!!!!!!!!!!!
//
// Any CPUID_XXXX defined as higher than CPUID_INTEL_MMX *must* have MMX support (eg like CPUID_AMD_3DNOW (0x30) has),
//	this allows convenient MMX capability checking. If you for some reason want to support some new processor that does
//	*NOT* have MMX (yeah, right), then define it as a lower number. -slc
//
// ( These values are returned by Sys_GetProcessorId )
//
pub const CPUID_GENERIC: c_int = 0;			// any unrecognized processor

pub const CPUID_AXP: c_int = 0x10;

pub const CPUID_INTEL_UNSUPPORTED: c_int = 0x20;			// Intel 386/486
pub const CPUID_INTEL_PENTIUM: c_int = 0x21;			// Intel Pentium or PPro
pub const CPUID_INTEL_MMX: c_int = 0x22;			// Intel Pentium/MMX or P2/MMX
pub const CPUID_INTEL_KATMAI: c_int = 0x23;			// Intel Katmai
pub const CPUID_INTEL_WILLIAMETTE: c_int = 0x24;			// Intel Williamette

pub const CPUID_AMD_3DNOW: c_int = 0x30;			// AMD K6 3DNOW!
//
//==========================================================

#[inline]
pub fn RoundUp(N: c_int, M: c_int) -> c_int {
	N + ((M as u32 - (N as u32 % M as u32)) as c_int)
}

#[inline]
pub fn RoundDown(N: c_int, M: c_int) -> c_int {
	N - ((N as u32 % M as u32) as c_int)
}

extern "C" {
	pub fn CopyString(in_str: *const c_char) -> *mut c_char;
	pub fn Info_Print(s: *const c_char);

	pub fn Com_BeginRedirect(buffer: *mut c_char, buffersize: c_int, flush: Option<extern "C" fn(*mut c_char)>);
	pub fn Com_EndRedirect();
	pub fn Com_Printf(fmt: *const c_char, ...);
	pub fn Com_DPrintf(fmt: *const c_char, ...);
	pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
	pub fn Com_Quit_f();
	pub fn Com_EventLoop() -> c_int;
	pub fn Com_Milliseconds() -> c_int;	// will be journaled properly
	pub fn Com_BlockChecksum(buffer: *const c_void, length: c_int) -> c_int;
	pub fn Com_Filter(filter: *mut c_char, name: *mut c_char, casesensitive: c_int) -> c_int;

	pub fn Com_StartupVariable(match_str: *const c_char);
	// checks for and removes command line "+set var arg" constructs
	// if match is NULL, all set commands will be executed, otherwise
	// only a set with the exact name.  Only used during startup.
}


extern "C" {
	pub static mut com_developer: *mut cvar_t;
	pub static mut com_speeds: *mut cvar_t;
	pub static mut com_timescale: *mut cvar_t;
	pub static mut com_sv_running: *mut cvar_t;
	pub static mut com_cl_running: *mut cvar_t;
	pub static mut com_viewlog: *mut cvar_t;			// 0 = hidden, 1 = visible, 2 = minimized
	pub static mut com_version: *mut cvar_t;

	// both client and server must agree to pause
	pub static mut cl_paused: *mut cvar_t;
	pub static mut sv_paused: *mut cvar_t;

	// com_speeds times
	pub static mut time_game: c_int;
	pub static mut time_frontend: c_int;
	pub static mut time_backend: c_int;		// renderer backend time

	pub static mut timeInTrace: c_int;
	pub static mut timeInPVSCheck: c_int;
	pub static mut numTraces: c_int;

	pub static mut com_frameTime: c_int;
	pub static mut com_frameMsec: c_int;

	pub static mut com_errorEntered: qboolean;


	// #ifndef _XBOX
	pub static mut com_journalFile: fileHandle_t;
	pub static mut com_journalDataFile: fileHandle_t;
	// #endif
}

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
	pub fn Z_Validate() -> c_int;			// also used to insure all of these are paged in
	pub fn Z_MemSize(eTag: memtag_t) -> c_int;
	pub fn Z_TagFree(eTag: memtag_t);
	pub fn Z_Free(ptr: *mut c_void) -> c_int;	//returns bytes freed
	pub fn Z_Size(pvAddress: *mut c_void) -> c_int;
	pub fn Z_MorphMallocTag(pvAddress: *mut c_void, eDesiredTag: memtag_t);
	pub fn Z_IsFromZone(pvAddress: *mut c_void, eTag: memtag_t) -> qboolean;	//returns size if true

	// #ifdef DEBUG_ZONE_ALLOCS
	//	void *_D_Z_Malloc ( int iSize, memtag_t eTag, qboolean bZeroit, const char *psFile, int iLine );
	//	void *_D_S_Malloc ( int iSize, const char *psFile, int iLine );
	//	void  _D_Z_Label  ( const void *pvAddress, const char *pslabel );
	// #else
	pub fn Z_Malloc(iSize: c_int, eTag: memtag_t, bZeroit: qboolean, iAlign: c_int) -> *mut c_void;	// return memory NOT zero-filled by default
	pub fn S_Malloc(iSize: c_int) -> *mut c_void;									// NOT 0 filled memory only for small allocations
	// #endif
}


extern "C" {
	pub fn Hunk_Clear();
	pub fn Hunk_ClearToMark();
	pub fn Hunk_SetMark();
}
// note the opposite default for 'bZeroIt' in Hunk_Alloc to Z_Malloc, since Hunk_Alloc always used to memset(0)...
//
#[inline]
pub fn Hunk_Alloc(size: c_int, bZeroIt: qboolean) -> *mut c_void {
	unsafe {
		Z_Malloc(size, 0, bZeroIt, 4)
	}
}


extern "C" {
	pub fn Com_TouchMemory();

	// commandLine should not include the executable name (argv[0])
	pub fn Com_SetOrgAngles(org: *mut c_void, angles: *mut c_void);
	pub fn Com_Init(commandLine: *mut c_char);
	pub fn Com_Frame();
	pub fn Com_Shutdown();
	pub fn Com_ShutdownZoneMemory();
	pub fn Com_ShutdownHunkMemory();

	// bool Com_ParseTextFile(const char *file, class CGenericParser2 &parser, bool cleanFirst = true);
	// CGenericParser2 *Com_ParseTextFile(const char *file, bool cleanFirst, bool writeable);
	// void Com_ParseTextFileDestroy(class CGenericParser2 &parser);
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
	pub fn CL_Disconnect();
	pub fn CL_Shutdown();
	pub fn CL_Frame(msec: c_int, fractionMsec: c_float);
	pub fn CL_GameCommand() -> qboolean;
	pub fn CL_KeyEvent(key: c_int, down: qboolean, time: c_int);

	pub fn CL_CharEvent(key: c_int);
	// char events are for field typing, not game control

	pub fn CL_MouseEvent(dx: c_int, dy: c_int, time: c_int);

	pub fn CL_JoystickEvent(axis: c_int, value: c_int, time: c_int);

	pub fn CL_PacketEvent(from: netadr_t, msg: *mut msg_t);

	pub fn CL_ConsolePrint(text: *mut c_char);

	pub fn CL_MapLoading();
	// do a screen update before starting to load a map
	// when the server is going to load a new map, the entire hunk
	// will be cleared, so the client must shutdown cgame, ui, and
	// the renderer

	pub fn CL_ForwardCommandToServer();
	// adds the current command line as a clc_clientCommand to the client message.
	// things like godmode, noclip, etc, are commands directed to the server,
	// so when they are typed in at the console, they will need to be forwarded.

	pub fn CL_FlushMemory();
	// dump all memory on an error

	pub fn CL_StartHunkUsers();

	pub fn Key_WriteBindings(f: fileHandle_t);
	// for writing the config files

	pub fn S_ClearSoundBuffer();
	// call before filesystem access

	pub fn SCR_DebugGraph(value: c_float, color: c_int);	// FIXME: move logging to common?


	//
	// server interface
	//
	pub fn SV_Init();
	pub fn SV_Shutdown(finalmsg: *mut c_char);
	pub fn SV_Frame(msec: c_int, fractionMsec: c_float);
	pub fn SV_PacketEvent(from: netadr_t, msg: *mut msg_t);
	pub fn SV_GameCommand() -> qboolean;


	//
	// UI interface
	//
	pub fn UI_GameCommand() -> qboolean;
}


/*
==============================================================

NON-PORTABLE SYSTEM SERVICES

==============================================================
*/

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum joystickAxis_t {
	AXIS_SIDE = 0,
	AXIS_FORWARD = 1,
	AXIS_UP = 2,
	AXIS_ROLL = 3,
	AXIS_YAW = 4,
	AXIS_PITCH = 5,
	MAX_JOYSTICK_AXIS = 6,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum sysEventType_t {
	SE_NONE = 0,	// evTime is still valid
	SE_KEY = 1,		// evValue is a key code, evValue2 is the down flag
	SE_CHAR = 2,	// evValue is an ascii char
	SE_MOUSE = 3,	// evValue and evValue2 are reletive signed x / y moves
	SE_JOYSTICK_AXIS = 4,	// evValue is an axis number and evValue2 is the current state (-127 to 127)
	SE_CONSOLE = 5,	// evPtr is a char*
	SE_PACKET = 6,	// evPtr is a netadr_t followed by data bytes to evPtrLength
}

#[repr(C)]
pub struct sysEvent_t {
	pub evTime: c_int,
	pub evType: sysEventType_t,
	pub evValue: c_int,
	pub evValue2: c_int,
	pub evPtrLength: c_int,	// bytes of data pointed to by evPtr, for journaling
	pub evPtr: *mut c_void,			// this must be manually freed if not NULL
}

extern "C" {
	pub fn Sys_GetEvent() -> sysEvent_t;

	pub fn Sys_Init();

	pub fn Sys_GetCurrentUser() -> *mut c_char;

	pub fn Sys_Error(error: *const c_char, ...);
	pub fn Sys_Quit();
	pub fn Sys_GetClipboardData() -> *mut c_char;	// note that this isn't journaled...

	pub fn Sys_Print(msg: *const c_char);
	// #ifdef _XBOX
	// void	Sys_Log( const char *file, const char *msg );
	// void	Sys_Log( const char *file, const void *buffer, int size, bool flush );
	// #endif

	// Sys_Milliseconds should only be used for profiling purposes,
	// any game related timing information should come from event timestamps
	pub fn Sys_Milliseconds() -> c_int;


	// the system console is shown when a dedicated server is running
	pub fn Sys_DisplaySystemConsole(show: qboolean);

	pub fn Sys_GetProcessorId() -> c_int;

	pub fn Sys_BeginStreamedFile(f: fileHandle_t, readahead: c_int);
	pub fn Sys_EndStreamedFile(f: fileHandle_t);
	pub fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: fileHandle_t) -> c_int;
	pub fn Sys_StreamSeek(f: fileHandle_t, offset: c_int, origin: c_int);

	pub fn Sys_ShowConsole(level: c_int, quitOnClose: qboolean);
	pub fn Sys_SetErrorText(text: *const c_char);

	pub fn Sys_CheckCD() -> qboolean;

	pub fn Sys_Mkdir(path: *const c_char);
	pub fn Sys_Cwd() -> *mut c_char;
	pub fn Sys_DefaultCDPath() -> *mut c_char;
	pub fn Sys_DefaultBasePath() -> *mut c_char;

	pub fn Sys_ListFiles(directory: *const c_char, extension: *const c_char, numfiles: *mut c_int, wantsubs: qboolean) -> *mut *mut c_char;
	pub fn Sys_FreeFileList(filelist: *mut *mut c_char);

	pub fn Sys_BeginProfiling();
	pub fn Sys_EndProfiling();

	pub fn Sys_LowPhysicalMemory() -> qboolean;
	// qboolean Sys_FileOutOfDate( LPCSTR psFinalFileName /* dest */, LPCSTR psDataFileName /* src */ );
	// qboolean Sys_CopyFile(LPCSTR lpExistingFileName, LPCSTR lpNewFileName, qboolean bOverwrite);
}


//byte*	SCR_GetScreenshot(qboolean *qValid);
//void	SCR_SetScreenshot(const byte *pbData, int w, int h);
//byte*	SCR_TempRawImage_ReadFromFile(const char *psLocalFilename, int *piWidth, int *piHeight, byte *pbReSampleBuffer, qboolean qbVertFlip);
//void	SCR_TempRawImage_CleanUp();

#[inline]
pub fn Round(value: c_float) -> c_int {
	(value + 0.5).floor() as c_int
}


// #ifdef _XBOX
// //////////////////////////////
// //
// // Map Lump Loader
// //
// struct Lump
// {
//	void* data;
//	int len;
//
//	Lump() : data(NULL), len(0) {}
//	~Lump() { clear(); }
//
//	void load(const char* map, const char* lump)
//	{
//		clear();
//
//		char path[MAX_QPATH];
//		Com_sprintf(path, MAX_QPATH, "%s/%s.mle", map, lump);
//
//		len = FS_ReadFile(path, &data);
//		if (len < 0) len = 0;
//	}
//
//	void clear(void)
//	{
//		if (data)
//		{
//			FS_FreeFile(data);
//			data = NULL;
//		}
//	}
// };
// #endif _XBOX
