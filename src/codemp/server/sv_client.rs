// sv_client.rs -- server code for dealing with clients

use core::ffi::{c_int, c_char, c_void};
use core::ptr;
use core::mem;

// PORTING_STUB: Forward declarations and external types that would normally come from server.h
// These are declared to satisfy the type system; actual definitions belong in other modules.

#[repr(C)]
pub struct client_t {
    // PORTING_STUB: Full definition expected from server.h
}

#[repr(C)]
pub struct netadr_t {
    // PORTING_STUB: Full definition expected from server.h
}

#[repr(C)]
pub struct challenge_t {
    // PORTING_STUB: Full definition expected from server.h
}

#[repr(C)]
pub struct sharedEntity_t {
    // PORTING_STUB: Full definition expected from server.h
}

#[repr(C)]
pub struct entityState_t {
    // PORTING_STUB: Full definition expected from server.h
}

#[repr(C)]
pub struct msg_t {
    // PORTING_STUB: Full definition expected from msg.h
}

#[repr(C)]
pub struct usercmd_t {
    // PORTING_STUB: Full definition expected from usercmd.h
}

#[repr(C)]
pub struct cvar_t {
    // PORTING_STUB: Full definition expected from cvar.h
}

#[repr(C)]
pub struct z_stream {
    // PORTING_STUB: Full definition expected from zlib.h
}

#[repr(C)]
pub struct XBPlayerInfo {
    // PORTING_STUB: Full definition expected for Xbox build
}

#[repr(C)]
pub struct XBOnlineInfo {
    // PORTING_STUB: Full definition expected for Xbox build
}

#[repr(C)]
pub struct rmAutomapSymbol_t {
    // PORTING_STUB: Full definition expected from RMG headers
}

// External function declarations
extern "C" {
    fn SV_CloseDownload(cl: *mut client_t);
    fn Cvar_VariableValue(name: *const c_char) -> f32;
    fn Sys_IsLANAddress(adr: netadr_t) -> c_int;
    fn NET_OutOfBandPrint(sock: c_int, adr: netadr_t, fmt: *const c_char, ...);
    fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> c_int;
    fn BigShort(s: c_int) -> c_int;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn NET_AdrToString(a: netadr_t) -> *const c_char;
    fn NET_CompareAdr(a: netadr_t, b: netadr_t) -> c_int;
    fn NET_CompareBaseAdr(a: netadr_t, b: netadr_t) -> c_int;
    fn NET_IsLocalAddress(adr: netadr_t) -> c_int;
    fn Com_Memset(ptr: *mut c_void, c: c_int, size: usize);
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    fn atoi(str: *const c_char) -> c_int;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);
    fn SE_GetString(table: *const c_char, label: *const c_char) -> *const c_char;
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn VM_Call(vm: *mut c_void, call: c_int, ...) -> *mut c_void;
    fn VM_ExplicitArgPtr(vm: *mut c_void, arg: c_int) -> *mut c_void;
    fn Netchan_Setup(sock: c_int, chan: *mut c_void, from: netadr_t, qport: c_int);
    fn SV_UserinfoChanged(cl: *mut client_t);
    fn SV_GentityNum(num: c_int) -> *mut sharedEntity_t;
    fn SV_Heartbeat_f();
    fn SV_SendServerCommand(cl: *mut client_t, fmt: *const c_char, ...);
    fn SV_DropClient(drop: *mut client_t, reason: *const c_char);
    fn SV_BotFreeClient(num: c_int);
    fn SV_SetUserinfo(client: c_int, s: *const c_char);
    fn MSG_Init(msg: *mut msg_t, data: *mut u8, len: c_int);
    fn MSG_WriteLong(msg: *mut msg_t, c: c_int);
    fn MSG_WriteShort(msg: *mut msg_t, c: c_int);
    fn MSG_WriteByte(msg: *mut msg_t, c: c_int);
    fn MSG_WriteBigString(msg: *mut msg_t, s: *const c_char);
    fn MSG_WriteDeltaEntity(msg: *mut msg_t, from: *const entityState_t, to: *const entityState_t, force: c_int);
    fn MSG_WriteData(msg: *mut msg_t, data: *const c_void, len: c_int);
    fn MSG_WriteBits(msg: *mut msg_t, value: c_int, bits: c_int);
    fn MSG_WriteString(msg: *mut msg_t, s: *const c_char);
    fn MSG_Bitstream(msg: *mut msg_t);
    fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    fn MSG_ReadByte(msg: *mut msg_t) -> c_int;
    fn MSG_ReadString(msg: *mut msg_t) -> *const c_char;
    fn MSG_ReadDeltaUsercmdKey(msg: *mut msg_t, key: c_int, from: *const usercmd_t, to: *mut usercmd_t);
    fn SV_SendMessageToClient(msg: *mut msg_t, client: *mut client_t);
    fn SV_SendClientGameState(client: *mut client_t);
    fn SV_Netchan_TransmitNextFragment(chan: *mut c_void);
    fn SV_UpdateServerCommandsToClient(client: *mut client_t, msg: *mut msg_t);
    fn SV_SendClientSnapshot(cl: *mut client_t);
    fn SV_ClientEnterWorld(client: *mut client_t, cmd: *mut usercmd_t);
    fn deflateInit(strm: *mut z_stream, level: c_int) -> c_int;
    fn deflate(strm: *mut z_stream, flush: c_int) -> c_int;
    fn deflateEnd(strm: *mut z_stream) -> c_int;
    fn Cmd_TokenizeString(text: *const c_char);
    fn Cmd_Argc() -> c_int;
    fn FS_LoadedPakPureChecksums() -> *const c_char;
    fn FS_FileIsInPAK(file: *const c_char, checksum: *mut c_int) -> c_int;
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn sprintf(s: *mut c_char, fmt: *const c_char, ...) -> c_int;
    fn Com_HashKey(data: *const c_char, len: c_int) -> c_int;
    fn FS_SV_FOpenFileRead(filename: *const c_char, file: *mut *mut c_void) -> c_int;
    fn FS_FCloseFile(f: *mut c_void);
    fn FS_Read(buffer: *mut c_void, len: c_int, f: *mut c_void) -> c_int;
    fn FS_idPak(pak: *const c_char, base: *const c_char) -> c_int;
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
}

// Global server state (would normally come from sv_main.c)
// PORTING_STUB: These should reference actual global svs and sv objects
#[allow(non_upper_case_globals)]
pub static mut svs: *mut c_void = ptr::null_mut();

#[allow(non_upper_case_globals)]
pub static mut sv: *mut c_void = ptr::null_mut();

#[allow(non_upper_case_globals)]
pub static mut gvm: *mut c_void = ptr::null_mut();

// Xbox-specific globals (referenced only in Xbox builds)
#[allow(non_upper_case_globals)]
#[cfg(feature = "xbox")]
pub static mut xbOnlineInfo: XBOnlineInfo = XBOnlineInfo {};

#[allow(non_upper_case_globals)]
#[cfg(feature = "xbox")]
pub static mut logged_on: c_int = 0;

#[allow(non_upper_case_globals)]
#[cfg(feature = "xbox")]
pub static mut TheRandomMissionManager: *mut c_void = ptr::null_mut();

// Forward declaration of SV_CloseDownload (static)
#[cfg(not(feature = "xbox"))]
unsafe fn SV_CloseDownload(cl: *mut client_t);

/*
=================
SV_GetChallenge

A "getchallenge" OOB command has been received
Returns a challenge number that can be used
in a subsequent connectResponse command.
We do this to prevent denial of service attacks that
flood the server with invalid connection IPs.  With a
challenge, they must give a valid IP address.

If we are authorizing, a challenge request will cause a packet
to be sent to the authorize server.

When an authorizeip is returned, a challenge response will be
sent to that ip.
=================
*/
pub unsafe fn SV_GetChallenge(from: netadr_t) {
    let mut i: c_int;
    let mut oldest: c_int;
    let mut oldestTime: c_int;
    let mut challenge: *mut challenge_t;

    // ignore if we are in single player
    /*
    if ( Cvar_VariableValue( "g_gametype" ) == GT_SINGLE_PLAYER || Cvar_VariableValue("ui_singlePlayerActive")) {
        return;
    }
    */
    if Cvar_VariableValue(b"ui_singlePlayerActive\0".as_ptr() as *const c_char) != 0.0
    {
        return;
    }

    oldest = 0;
    oldestTime = 0x7fffffff;

    // see if we already have a challenge for this ip
    // PORTING_STUB: svs.challenges[0] needs proper type casting
    // challenge = &svs.challenges[0];
    // for (i = 0 ; i < MAX_CHALLENGES ; i++, challenge++) {
    //     if ( !challenge->connected && NET_CompareAdr( from, challenge->adr ) ) {
    //         break;
    //     }
    //     if ( challenge->time < oldestTime ) {
    //         oldestTime = challenge->time;
    //         oldest = i;
    //     }
    // }

    // if (i == MAX_CHALLENGES) {
    //     // this is the first time this client has asked for a challenge
    //     challenge = &svs.challenges[oldest];

    //     challenge->challenge = ( (rand() << 16) ^ rand() ) ^ svs.time;
    //     challenge->adr = from;
    //     challenge->firstTime = svs.time;
    //     challenge->time = svs.time;
    //     challenge->connected = qfalse;
    //     i = oldest;
    // }

    // // if they are on a lan address, send the challengeResponse immediately
    // if ( Sys_IsLANAddress( from ) ) {
    //     challenge->pingTime = svs.time;
    //     NET_OutOfBandPrint( NS_SERVER, from, "challengeResponse %i", challenge->challenge );
    //     return;
    // }

    // #ifdef USE_CD_KEY
    //     // look up the authorize server's IP
    //     if ( !svs.authorizeAddress.ip[0] && svs.authorizeAddress.type != NA_BAD ) {
    //         Com_Printf( "Resolving %s\n", AUTHORIZE_SERVER_NAME );
    //         if ( !NET_StringToAdr( AUTHORIZE_SERVER_NAME, &svs.authorizeAddress ) ) {
    //             Com_Printf( "Couldn't resolve address\n" );
    //             return;
    //         }
    //         svs.authorizeAddress.port = BigShort( PORT_AUTHORIZE );
    //         Com_Printf( "%s resolved to %i.%i.%i.%i:%i\n", AUTHORIZE_SERVER_NAME,
    //             svs.authorizeAddress.ip[0], svs.authorizeAddress.ip[1],
    //             svs.authorizeAddress.ip[2], svs.authorizeAddress.ip[3],
    //             BigShort( svs.authorizeAddress.port ) );
    //     }

    //     // if they have been challenging for a long time and we
    //     // haven't heard anything from the authoirze server, go ahead and
    //     // let them in, assuming the id server is down
    //     if ( svs.time - challenge->firstTime > AUTHORIZE_TIMEOUT ) {
    //         Com_DPrintf( "authorize server timed out\n" );

    //         challenge->pingTime = svs.time;
    //         NET_OutOfBandPrint( NS_SERVER, challenge->adr,
    //             "challengeResponse %i", challenge->challenge );
    //         return;
    //     }

    //     // otherwise send their ip to the authorize server
    //     if ( svs.authorizeAddress.type != NA_BAD ) {
    //         let fs: *mut cvar_t;
    //         let game: [c_char; 1024] = [0; 1024];

    //         game[0] = 0;
    //         fs = Cvar_Get ("fs_game", "", CVAR_INIT|CVAR_SYSTEMINFO );
    //         if (fs && fs->string[0] != 0) {
    //             strcpy(game, fs->string);
    //         }
    //         Com_DPrintf( "sending getIpAuthorize for %s\n", NET_AdrToString( from ));
    //         fs = Cvar_Get ("sv_allowAnonymous", "0", CVAR_SERVERINFO);

    //         NET_OutOfBandPrint( NS_SERVER, svs.authorizeAddress,
    //             "getIpAuthorize %i %i.%i.%i.%i %s %s",  svs.challenges[i].challenge,
    //             from.ip[0], from.ip[1], from.ip[2], from.ip[3], game, fs->integer );
    //     }
    // #else
    //     challenge->pingTime = svs.time;
    //     NET_OutOfBandPrint( NS_SERVER, challenge->adr, "challengeResponse %i", challenge->challenge );
    // #endif	// USE_CD_KEY
}

/*
====================
SV_AuthorizeIpPacket

A packet has been returned from the authorize server.
If we have a challenge adr for that ip, send the
challengeResponse to it
====================
*/
#[cfg(not(feature = "xbox"))]
pub unsafe fn SV_AuthorizeIpPacket(from: netadr_t) {
    // let mut challenge: c_int;
    // let mut i: c_int;
    // let mut s: *mut c_char;
    // let mut r: *mut c_char;
    // let mut ret: [c_char; 1024] = [0; 1024];

    // if ( !NET_CompareBaseAdr( from, svs.authorizeAddress ) ) {
    //     Com_Printf( "SV_AuthorizeIpPacket: not from authorize server\n" );
    //     return;
    // }

    // challenge = atoi( Cmd_Argv( 1 ) );

    // for (i = 0 ; i < MAX_CHALLENGES ; i++) {
    //     if ( svs.challenges[i].challenge == challenge ) {
    //         break;
    //     }
    // }
    // if ( i == MAX_CHALLENGES ) {
    //     Com_Printf( "SV_AuthorizeIpPacket: challenge not found\n" );
    //     return;
    // }

    // // send a packet back to the original client
    // svs.challenges[i].pingTime = svs.time;
    // s = Cmd_Argv( 2 );
    // r = Cmd_Argv( 3 );			// reason

    // if ( !Q_stricmp( s, "demo" ) ) {
    //     if ( Cvar_VariableValue( "fs_restrict" ) ) {
    //         // a demo client connecting to a demo server
    //         NET_OutOfBandPrint( NS_SERVER, svs.challenges[i].adr,
    //             "challengeResponse %i", svs.challenges[i].challenge );
    //         return;
    //     }
    //     // they are a demo client trying to connect to a real server
    //     NET_OutOfBandPrint( NS_SERVER, svs.challenges[i].adr, "print\nServer is not a demo server\n" );
    //     // clear the challenge record so it won't timeout and let them through
    //     Com_Memset( &svs.challenges[i], 0, mem::size_of::<challenge_t>() as c_int );
    //     return;
    // }
    // if ( !Q_stricmp( s, "accept" ) ) {
    //     NET_OutOfBandPrint( NS_SERVER, svs.challenges[i].adr,
    //         "challengeResponse %i", svs.challenges[i].challenge );
    //     return;
    // }
    // if ( !Q_stricmp( s, "unknown" ) ) {
    //     if (!r) {
    //         NET_OutOfBandPrint( NS_SERVER, svs.challenges[i].adr, "print\nAwaiting CD key authorization\n" );
    //     } else {
    //         sprintf(ret, "print\n%s\n", r);
    //         NET_OutOfBandPrint( NS_SERVER, svs.challenges[i].adr, ret );
    //     }
    //     // clear the challenge record so it won't timeout and let them through
    //     Com_Memset( &svs.challenges[i], 0, mem::size_of::<challenge_t>() as c_int );
    //     return;
    // }

    // // authorization failed
    // if (!r) {
    //     NET_OutOfBandPrint( NS_SERVER, svs.challenges[i].adr, "print\nSomeone is using this CD Key\n" );
    // } else {
    //     sprintf(ret, "print\n%s\n", r);
    //     NET_OutOfBandPrint( NS_SERVER, svs.challenges[i].adr, ret );
    // }

    // // clear the challenge record so it won't timeout and let them through
    // Com_Memset( &svs.challenges[i], 0, mem::size_of::<challenge_t>() as c_int );
}

/*
==================
SV_DirectConnect

A "connect" OOB command has been received
==================
*/
pub unsafe fn SV_DirectConnect(from: netadr_t) {
    // let mut userinfo: [c_char; 1024]; // MAX_INFO_STRING
    // let mut i: c_int;
    // let mut cl: *mut client_t;
    // let mut newcl: *mut client_t;
    // let mut temp: client_t;
    // let mut ent: *mut sharedEntity_t;
    // let mut clientNum: c_int;
    // let mut version: c_int;
    // let mut qport: c_int;
    // let mut challenge: c_int;
    // let mut password: *const c_char;
    // let mut startIndex: c_int;
    // let mut denied: *const c_char;
    // let mut count: c_int;
    // let mut reconnect: bool = false;

    // Com_DPrintf ("SVC_DirectConnect ()\n");

    // Q_strncpyz( userinfo, Cmd_Argv(1), mem::size_of_val(&userinfo) as c_int );

    // version = atoi( Info_ValueForKey( userinfo, "protocol" ) );
    // if ( version != PROTOCOL_VERSION ) {
    //     NET_OutOfBandPrint( NS_SERVER, from, "print\nServer uses protocol version %i.\n", PROTOCOL_VERSION );
    //     Com_DPrintf ("    rejected connect from version %i\n", version);
    //     return;
    // }

    // challenge = atoi( Info_ValueForKey( userinfo, "challenge" ) );
    // qport = atoi( Info_ValueForKey( userinfo, "qport" ) );

    // // quick reject
    // i = 0;
    // cl = ptr::addr_of_mut!((*svs).clients).cast::<*mut client_t>();
    // loop {
    //     if i >= sv_maxclients->integer {
    //         break;
    //     }

    //     if ( NET_CompareBaseAdr( from, cl->netchan.remoteAddress )
    //         && ( cl->netchan.qport == qport
    //         || from.port == cl->netchan.remoteAddress.port ) ) {
    //         if (( svs.time - cl->lastConnectTime)
    //             < (sv_reconnectlimit->integer * 1000)) {
    //             NET_OutOfBandPrint( NS_SERVER, from, "print\nReconnect rejected : too soon\n" );
    //             Com_DPrintf ("%s:reconnect rejected : too soon\n", NET_AdrToString (from));
    //             return;
    //         }
    //         break;
    //     }
    //     i += 1;
    //     cl = cl.add(1);
    // }

    // rest of SV_DirectConnect...
}

/*
=====================
SV_DropClient

Called when the player is totally leaving the server, either willingly
or unwillingly.  This is NOT called if the entire server is quiting
or crashing -- SV_FinalMessage() will handle that
=====================
*/
pub unsafe fn SV_DropClient(drop: *mut client_t, reason: *const c_char) {
    // let mut i: c_int;
    // let mut challenge: *mut challenge_t;

    // if ( drop->state == CS_ZOMBIE ) {
    //     return;		// already dropped
    // }

    // if ( !drop->gentity || !(drop->gentity->r.svFlags & SVF_BOT) ) {
    //     // see if we already have a challenge for this ip
    //     challenge = &svs.challenges[0];

    //     for (i = 0 ; i < MAX_CHALLENGES ; i++, challenge++) {
    //         if ( NET_CompareAdr( drop->netchan.remoteAddress, challenge->adr ) ) {
    //             challenge->connected = qfalse;
    //             break;
    //         }
    //     }
    // }

    // #ifdef _XBOX
    //     // Tells all clients to remove the dropped player from their list, if not a bot
    //     if ( drop->netchan.remoteAddress.type != NA_BOT )
    //     {
    //         let index: c_int = drop - svs.clients;
    //         for (let j: c_int = 0; j < sv_maxclients->integer ; j++) {
    //             if ( svs.clients[j].state < CS_PRIMED ) {
    //                 continue;
    //             }
    //             if ( svs.clients[j].netchan.remoteAddress.type == NA_BOT ) {
    //                 continue;
    //             }
    //             SV_SendClientRemovePeer(&svs.clients[j], index);
    //         }

    //         // update the advertised session
    //         XBL_MM_RemovePlayer( drop->usePrivateSlot );
    //     }
    // #endif

    // // Kill any download
    // #ifndef _XBOX	// No downloads on Xbox
    //     SV_CloseDownload( drop );
    // #endif

    // // tell everyone why they got dropped
    // SV_SendServerCommand( NULL, "print \"%s" S_COLOR_WHITE " %s\n\"", drop->name, reason );

    // Com_DPrintf( "Going to CS_ZOMBIE for %s\n", drop->name );
    // drop->state = CS_ZOMBIE;		// become free in a few seconds

    // #ifndef _XBOX	// No downloads on Xbox
    //     if (drop->download)	{
    //         FS_FCloseFile( drop->download );
    //         drop->download = 0;
    //     }
    // #endif

    // // call the prog function for removing a client
    // // this will remove the body, among other things
    // VM_Call( gvm, GAME_CLIENT_DISCONNECT, drop - svs.clients );

    // // add the disconnect command
    // SV_SendServerCommand( drop, va("disconnect \"%s\"", reason ) );

    // if ( drop->netchan.remoteAddress.type == NA_BOT ) {
    //     SV_BotFreeClient( drop - svs.clients );
    // }

    // // nuke user info
    // SV_SetUserinfo( drop - svs.clients, "" );

    // // if this was the last client on the server, send a heartbeat
    // // to the master so it is known the server is empty
    // // send a heartbeat now so the master will get up to date info
    // // if there is already a slot for this ip, reuse it
    // #ifndef _XBOX	// No master on Xbox
    //     for (i=0 ; i < sv_maxclients->integer ; i++ ) {
    //         if ( svs.clients[i].state >= CS_CONNECTED ) {
    //             break;
    //         }
    //     }
    //     if ( i == sv_maxclients->integer ) {
    //         SV_Heartbeat_f();
    //     }
    // #endif
}

pub unsafe fn SV_WriteRMGAutomapSymbols(msg: *mut msg_t) {
    // let mut count: c_int = TheRandomMissionManager->GetAutomapSymbolCount ( );
    // let mut i: c_int;

    // MSG_WriteShort ( msg, count );

    // for ( i = 0; i < count; i ++ )
    // {
    //     let symbol: *mut rmAutomapSymbol_t = TheRandomMissionManager->GetAutomapSymbol ( i );

    //     MSG_WriteByte ( msg, symbol->mType );
    //     MSG_WriteByte ( msg, symbol->mSide );
    //     MSG_WriteLong ( msg, (long)symbol->mOrigin[0] );
    //     MSG_WriteLong ( msg, (long)symbol->mOrigin[1] );
    // }
}

/*
================
SV_SendClientGameState

Sends the first message from the server to a connected client.
This will be sent on the initial connection and upon each new map load.

It will be resent if the client acknowledges a later message but has
the wrong gamestate.
================
*/
pub unsafe fn SV_SendClientGameState(client: *mut client_t) {
    // let mut start: c_int;
    // let mut base: *mut entityState_t;
    // let mut nullstate: entityState_t;
    // let mut msg: msg_t;
    // let mut msgBuffer: [u8; MAX_MSGLEN];

    // // MW - my attempt to fix illegible server message errors caused by
    // // packet fragmentation of initial snapshot.
    // while((*client).state && (*client).netchan.unsentFragments)
    // {
    //     // send additional message fragments if the last message
    //     // was too large to send at once

    //     Com_Printf ("[ISM]SV_SendClientGameState() [2] for %s, writing out old fragments\n", (*client).name);
    //     SV_Netchan_TransmitNextFragment(&(*client).netchan);
    // }

    // Com_DPrintf ("SV_SendClientGameState() for %s\n", (*client).name);
    // Com_DPrintf( "Going from CS_CONNECTED to CS_PRIMED for %s\n", (*client).name );
    // (*client).state = CS_PRIMED;
    // (*client).pureAuthentic = 0;

    // // when we receive the first packet from the client, we will
    // // notice that it is from a different serverid and that the
    // // gamestate message was not just sent, forcing a retransmit
    // (*client).gamestateMessageNum = (*client).netchan.outgoingSequence;

    // MSG_Init( &msg, msgBuffer.as_mut_ptr(), mem::size_of_val(&msgBuffer) as c_int );

    // // NOTE, MRE: all server->client messages now acknowledge
    // // let the client know which reliable clientCommands we have received
    // MSG_WriteLong( &msg, (*client).lastClientCommand );

    // // send any server commands waiting to be sent first.
    // // we have to do this cause we send the client->reliableSequence
    // // with a gamestate and it sets the clc.serverCommandSequence at
    // // the client side
    // SV_UpdateServerCommandsToClient( client, &msg );

    // // send the gamestate
    // MSG_WriteByte( &msg, svc_gamestate );
    // MSG_WriteLong( &msg, (*client).reliableSequence );

    // // write the configstrings
    // start = 0;
    // loop {
    //     if start >= MAX_CONFIGSTRINGS {
    //         break;
    //     }
    //     if (sv.configstrings[start][0] != 0) {
    //         MSG_WriteByte( &msg, svc_configstring );
    //         MSG_WriteShort( &msg, start );
    //         MSG_WriteBigString( &msg, sv.configstrings[start] );
    //     }
    //     start += 1;
    // }

    // // write the baselines
    // Com_Memset( &nullstate, 0, mem::size_of::<entityState_t>() as c_int );
    // start = 0;
    // loop {
    //     if start >= MAX_GENTITIES {
    //         break;
    //     }
    //     base = &sv.svEntities[start].baseline;
    //     if ( !(*base).number ) {
    //         start += 1;
    //         continue;
    //     }
    //     MSG_WriteByte( &msg, svc_baseline );
    //     MSG_WriteDeltaEntity( &msg, &nullstate, base, qtrue );
    //     start += 1;
    // }

    // MSG_WriteByte( &msg, svc_EOF );

    // MSG_WriteLong( &msg, client.offset_from(svs.clients) as c_int);

    // // write the checksum feed
    // MSG_WriteLong( &msg, sv.checksumFeed);

    // //rwwRMG - send info for the terrain
    // if ( TheRandomMissionManager )
    // {
    //     let mut zdata: z_stream;

    //     // Send the height map
    //     mem::memset(&zdata, 0, mem::size_of::<z_stream>());
    //     deflateInit ( &zdata, Z_MAX_COMPRESSION );

    //     let heightmap: [c_uchar; 15000];
    //     zdata.next_out = heightmap.as_mut_ptr();
    //     zdata.avail_out = 15000;
    //     zdata.next_in = TheRandomMissionManager->GetLandScape()->GetHeightMap();
    //     zdata.avail_in = TheRandomMissionManager->GetLandScape()->GetRealArea();
    //     deflate(&zdata, Z_SYNC_FLUSH);

    //     MSG_WriteShort ( &msg, zdata.total_out as u16 );
    //     MSG_WriteBits ( &msg, 1, 1 );
    //     MSG_WriteData ( &msg, heightmap.as_ptr() as *const c_void, zdata.total_out as c_int);

    //     deflateEnd(&zdata);

    //     // Send the flatten map
    //     mem::memset(&zdata, 0, mem::size_of::<z_stream>());
    //     deflateInit ( &zdata, Z_MAX_COMPRESSION );

    //     zdata.next_out = heightmap.as_mut_ptr();
    //     zdata.avail_out = 15000;
    //     zdata.next_in = TheRandomMissionManager->GetLandScape()->GetFlattenMap();
    //     zdata.avail_in = TheRandomMissionManager->GetLandScape()->GetRealArea();
    //     deflate(&zdata, Z_SYNC_FLUSH);

    //     MSG_WriteShort ( &msg, zdata.total_out as u16 );
    //     MSG_WriteBits ( &msg, 1, 1 );
    //     MSG_WriteData ( &msg, heightmap.as_ptr() as *const c_void, zdata.total_out as c_int);

    //     deflateEnd(&zdata);

    //     // Seed is needed for misc ents and noise
    //     MSG_WriteLong ( &msg, TheRandomMissionManager->GetLandScape()->get_rand_seed ( ) );

    //     SV_WriteRMGAutomapSymbols ( &msg );
    // }
    // else
    // {
    //     MSG_WriteShort ( &msg, 0 );
    // }

    // // deliver this to the client
    // SV_SendMessageToClient( &msg, client );
}

pub unsafe fn SV_SendClientMapChange(client: *mut client_t) {
    // let mut msg: msg_t;
    // let mut msgBuffer: [u8; MAX_MSGLEN];

    // MSG_Init( &msg, msgBuffer.as_mut_ptr(), mem::size_of_val(&msgBuffer) as c_int );

    // // NOTE, MRE: all server->client messages now acknowledge
    // // let the client know which reliable clientCommands we have received
    // MSG_WriteLong( &msg, (*client).lastClientCommand );

    // // send any server commands waiting to be sent first.
    // // we have to do this cause we send the client->reliableSequence
    // // with a gamestate and it sets the clc.serverCommandSequence at
    // // the client side
    // SV_UpdateServerCommandsToClient( client, &msg );

    // // send the gamestate
    // MSG_WriteByte( &msg, svc_mapchange );

    // // deliver this to the client
    // SV_SendMessageToClient( &msg, client );
}

#[cfg(feature = "xbox")]
pub unsafe fn SV_SendClientNewPeer(client: *mut client_t, info: *mut XBPlayerInfo) {
    // let mut msg: msg_t;
    // let mut msgBuffer: [u8; MAX_MSGLEN];

    // MSG_Init( &msg, msgBuffer.as_mut_ptr(), mem::size_of_val(&msgBuffer) as c_int );

    // // NOTE, MRE: all server->client messages now acknowledge
    // // let the client know which reliable clientCommands we have received
    // MSG_WriteLong( &msg, (*client).lastClientCommand );

    // // send any server commands waiting to be sent first.
    // // we have to do this cause we send the client->reliableSequence
    // // with a gamestate and it sets the clc.serverCommandSequence at
    // // the client side
    // SV_UpdateServerCommandsToClient( client, &msg );

    // // send the command
    // MSG_WriteByte( &msg, svc_newpeer );

    // // We now write the specific player number as well, so the clients know where
    // // to put this info. (That keeps cgs.clientinfo in sync with xbPlayerList)
    // MSG_WriteLong( &msg, info.offset_from(xbOnlineInfo.xbPlayerList.as_ptr()) as c_int );
    // MSG_WriteData(&msg, info as *const c_void, mem::size_of::<XBPlayerInfo>() as c_int);

    // // deliver this to the client
    // SV_SendMessageToClient( &msg, client );
}

#[cfg(feature = "xbox")]
pub unsafe fn SV_SendClientRemovePeer(client: *mut client_t, index: c_int) {
    // let mut msg: msg_t;
    // let mut msgBuffer: [u8; MAX_MSGLEN];

    // MSG_Init( &msg, msgBuffer.as_mut_ptr(), mem::size_of_val(&msgBuffer) as c_int );

    // // NOTE, MRE: all server->client messages now acknowledge
    // // let the client know which reliable clientCommands we have received
    // MSG_WriteLong( &msg, (*client).lastClientCommand );

    // // send any server commands waiting to be sent first.
    // // we have to do this cause we send the client->reliableSequence
    // // with a gamestate and it sets the clc.serverCommandSequence at
    // // the client side
    // SV_UpdateServerCommandsToClient( client, &msg );

    // // send the command
    // MSG_WriteByte( &msg, svc_removepeer );

    // // All clients have IDENTICAL ordering within xbPlayerList, so just
    // // send the index (rather than the XUID, like we did before).
    // MSG_WriteLong( &msg, index );

    // // deliver this to the client
    // SV_SendMessageToClient( &msg, client );
}

#[cfg(feature = "xbox")]
pub unsafe fn SV_SendClientXbInfo(client: *mut client_t) {
    // let mut msg: msg_t;
    // let mut msgBuffer: [u8; MAX_MSGLEN];

    // MSG_Init( &msg, msgBuffer.as_mut_ptr(), mem::size_of_val(&msgBuffer) as c_int );

    // // NOTE, MRE: all server->client messages now acknowledge
    // // let the client know which reliable clientCommands we have received
    // MSG_WriteLong( &msg, (*client).lastClientCommand );

    // // send any server commands waiting to be sent first.
    // // we have to do this cause we send the client->reliableSequence
    // // with a gamestate and it sets the clc.serverCommandSequence at
    // // the client side
    // SV_UpdateServerCommandsToClient( client, &msg );

    // // send the command
    // MSG_WriteByte( &msg, svc_xbInfo );

    // MSG_WriteData(&msg, addr_of!(xbOnlineInfo) as *const c_void, mem::size_of::<XBOnlineInfo>() as c_int);

    // // deliver this to the client
    // SV_SendMessageToClient( &msg, client );
}

/*
==================
SV_ClientEnterWorld
==================
*/
pub unsafe fn SV_ClientEnterWorld(client: *mut client_t, cmd: *mut usercmd_t) {
    // let mut clientNum: c_int;
    // let mut ent: *mut sharedEntity_t;

    // Com_DPrintf( "Going from CS_PRIMED to CS_ACTIVE for %s\n", (*client).name );
    // (*client).state = CS_ACTIVE;

    // #ifdef _XBOX
    //     //update XbOnlineInfo with client
    //     SV_SendClientXbInfo(client);
    // #endif

    // // set up the entity for the client
    // clientNum = client.offset_from(svs.clients) as c_int;
    // ent = SV_GentityNum( clientNum );
    // (*ent).s.number = clientNum;
    // (*client).gentity = ent;

    // (*client).lastUserInfoChange = 0; //reset the delay
    // (*client).lastUserInfoCount = 0; //reset the count

    // (*client).deltaMessage = -1;
    // (*client).nextSnapshotTime = svs.time;	// generate a snapshot immediately
    // (*client).lastUsercmd = *cmd;

    // // call the game begin function
    // VM_Call( gvm, GAME_CLIENT_BEGIN, clientNum );
}

/*
==================
SV_CloseDownload

clear/free any download vars
==================
*/
#[cfg(not(feature = "xbox"))]
unsafe fn SV_CloseDownload(cl: *mut client_t) {
    // let mut i: c_int;

    // // EOF
    // if ((*cl).download) {
    //     FS_FCloseFile( (*cl).download );
    // }
    // (*cl).download = ptr::null_mut();
    // (*cl).downloadName[0] = 0;

    // // Free the temporary buffer space
    // for (i = 0; i < MAX_DOWNLOAD_WINDOW; i++) {
    //     if ((*cl).downloadBlocks[i]) {
    //         Z_Free( (*cl).downloadBlocks[i] );
    //         (*cl).downloadBlocks[i] = ptr::null_mut();
    //     }
    // }
}

/*
==================
SV_StopDownload_f

Abort a download if in progress
==================
*/
#[cfg(not(feature = "xbox"))]
pub unsafe fn SV_StopDownload_f(cl: *mut client_t) {
    // if (*cl).downloadName[0] != 0 {
    //     Com_DPrintf( "clientDownload: %d : file \"%s\" aborted\n", cl.offset_from(svs.clients) as c_int, (*cl).downloadName );
    // }

    // SV_CloseDownload( cl );
}

/*
==================
SV_DoneDownload_f

Downloads are finished
==================
*/
#[cfg(not(feature = "xbox"))]
pub unsafe fn SV_DoneDownload_f(cl: *mut client_t) {
    // Com_DPrintf( "clientDownload: %s Done\n", (*cl).name);
    // // resend the game state to update any clients that entered during the download
    // SV_SendClientGameState(cl);
}

/*
==================
SV_NextDownload_f

The argument will be the last acknowledged block from the client, it should be
the same as cl->downloadClientBlock
==================
*/
#[cfg(not(feature = "xbox"))]
pub unsafe fn SV_NextDownload_f(cl: *mut client_t) {
    // let mut block: c_int = atoi( Cmd_Argv(1) );

    // if (block == (*cl).downloadClientBlock) {
    //     Com_DPrintf( "clientDownload: %d : client acknowledge of block %d\n", cl.offset_from(svs.clients) as c_int, block );

    //     // Find out if we are done.  A zero-length block indicates EOF
    //     if ((*cl).downloadBlockSize[((*cl).downloadClientBlock % MAX_DOWNLOAD_WINDOW) as usize] == 0) {
    //         Com_Printf( "clientDownload: %d : file \"%s\" completed\n", cl.offset_from(svs.clients) as c_int, (*cl).downloadName );
    //         SV_CloseDownload( cl );
    //         return;
    //     }

    //     (*cl).downloadSendTime = svs.time;
    //     (*cl).downloadClientBlock += 1;
    //     return;
    // }
    // // We aren't getting an acknowledge for the correct block, drop the client
    // // FIXME: this is bad... the client will never parse the disconnect message
    // //			because the cgame isn't loaded yet
    // SV_DropClient( cl, "broken download" );
}

/*
==================
SV_BeginDownload_f
==================
*/
#[cfg(not(feature = "xbox"))]
pub unsafe fn SV_BeginDownload_f(cl: *mut client_t) {
    // // Kill any existing download
    // SV_CloseDownload( cl );

    // // cl->downloadName is non-zero now, SV_WriteDownloadToClient will see this and open
    // // the file itself
    // Q_strncpyz( (*cl).downloadName, Cmd_Argv(1), mem::size_of_val(&(*cl).downloadName) as c_int );
}

/*
==================
SV_WriteDownloadToClient

Check to see if the client wants a file, open it if needed and start pumping the client
Fill up msg with data
==================
*/
#[cfg(not(feature = "xbox"))]
pub unsafe fn SV_WriteDownloadToClient(cl: *mut client_t, msg: *mut msg_t) {
    // let mut curindex: c_int;
    // let mut rate: c_int;
    // let mut blockspersnap: c_int;
    // let mut idPack: c_int;
    // let mut missionPack: c_int;
    // let mut errorMessage: [c_char; 1024];

    // if (*cl).downloadName[0] == 0 {
    //     return;	// Nothing being downloaded
    // }

    // if (*cl).download == ptr::null_mut() {
    //     // We open the file here

    //     Com_Printf( "clientDownload: %d : begining \"%s\"\n", cl.offset_from(svs.clients) as c_int, (*cl).downloadName );

    //     missionPack = FS_idPak((*cl).downloadName, b"missionpack\0".as_ptr() as *const c_char);
    //     idPack = missionPack != 0 || FS_idPak((*cl).downloadName, b"base\0".as_ptr() as *const c_char) != 0;

    //     if ( !sv_allowDownload->integer || idPack != 0 ||
    //         ( (*cl).downloadSize = FS_SV_FOpenFileRead( (*cl).downloadName, &mut (*cl).download ) ) <= 0 ) {
    //         // cannot auto-download file
    //         if idPack != 0 {
    //             Com_Printf(b"clientDownload: %d : \"%s\" cannot download id pk3 files\n\0".as_ptr() as *const c_char, cl.offset_from(svs.clients) as c_int, (*cl).downloadName);
    //             if missionPack != 0 {
    //                 Com_sprintf(errorMessage.as_mut_ptr(), mem::size_of_val(&errorMessage), b"Cannot autodownload Team Arena file \"%s\"\nThe Team Arena mission pack can be found in your local game store.\0".as_ptr() as *const c_char, (*cl).downloadName);
    //             } else {
    //                 Com_sprintf(errorMessage.as_mut_ptr(), mem::size_of_val(&errorMessage), b"Cannot autodownload id pk3 file \"%s\"\0".as_ptr() as *const c_char, (*cl).downloadName);
    //             }
    //         } else if ( !sv_allowDownload->integer ) {
    //             Com_Printf(b"clientDownload: %d : \"%s\" download disabled\0".as_ptr() as *const c_char, cl.offset_from(svs.clients) as c_int, (*cl).downloadName);
    //             if sv_pure->integer != 0 {
    //                 Com_sprintf(errorMessage.as_mut_ptr(), mem::size_of_val(&errorMessage), b"Could not download \"%s\" because autodownloading is disabled on the server.\n\nYou will need to get this file elsewhere before you can connect to this pure server.\n\0".as_ptr() as *const c_char, (*cl).downloadName);
    //             } else {
    //                 Com_sprintf(errorMessage.as_mut_ptr(), mem::size_of_val(&errorMessage), b"Could not download \"%s\" because autodownloading is disabled on the server.\n\nSet autodownload to No in your settings and you might be able to connect if you do have the file.\n\0".as_ptr() as *const c_char, (*cl).downloadName);
    //             }
    //         } else {
    //             Com_Printf(b"clientDownload: %d : \"%s\" file not found on server\n\0".as_ptr() as *const c_char, cl.offset_from(svs.clients) as c_int, (*cl).downloadName);
    //             Com_sprintf(errorMessage.as_mut_ptr(), mem::size_of_val(&errorMessage), b"File \"%s\" not found on server for autodownloading.\n\0".as_ptr() as *const c_char, (*cl).downloadName);
    //         }
    //         MSG_WriteByte( msg, svc_download );
    //         MSG_WriteShort( msg, 0 ); // client is expecting block zero
    //         MSG_WriteLong( msg, -1 ); // illegal file size
    //         MSG_WriteString( msg, errorMessage.as_ptr() );

    //         (*cl).downloadName[0] = 0;
    //         return;
    //     }

    //     // Init
    //     (*cl).downloadCurrentBlock = (*cl).downloadClientBlock = (*cl).downloadXmitBlock = 0;
    //     (*cl).downloadCount = 0;
    //     (*cl).downloadEOF = qfalse;
    // }

    // // Perform any reads that we need to
    // while ((*cl).downloadCurrentBlock - (*cl).downloadClientBlock < MAX_DOWNLOAD_WINDOW &&
    //     (*cl).downloadSize != (*cl).downloadCount) {

    //     curindex = (((*cl).downloadCurrentBlock % MAX_DOWNLOAD_WINDOW) as usize);

    //     if (*cl).downloadBlocks[curindex] == ptr::null_mut() {
    //         (*cl).downloadBlocks[curindex] = Z_Malloc( MAX_DOWNLOAD_BLKSIZE as usize, TAG_DOWNLOAD, qtrue as c_int ) as *mut c_uchar;
    //     }

    //     (*cl).downloadBlockSize[curindex] = FS_Read( (*cl).downloadBlocks[curindex] as *mut c_void, MAX_DOWNLOAD_BLKSIZE, (*cl).download );

    //     if ((*cl).downloadBlockSize[curindex] < 0) {
    //         // EOF right now
    //         (*cl).downloadCount = (*cl).downloadSize;
    //         break;
    //     }

    //     (*cl).downloadCount += (*cl).downloadBlockSize[curindex];

    //     // Load in next block
    //     (*cl).downloadCurrentBlock += 1;
    // }

    // // Check to see if we have eof condition and add the EOF block
    // if ((*cl).downloadCount == (*cl).downloadSize &&
    //     !(*cl).downloadEOF &&
    //     (*cl).downloadCurrentBlock - (*cl).downloadClientBlock < MAX_DOWNLOAD_WINDOW) {

    //     (*cl).downloadBlockSize[((*cl).downloadCurrentBlock % MAX_DOWNLOAD_WINDOW) as usize] = 0;
    //     (*cl).downloadCurrentBlock += 1;

    //     (*cl).downloadEOF = qtrue;  // We have added the EOF block
    // }

    // // Loop up to window size times based on how many blocks we can fit in the
    // // client snapMsec and rate

    // // based on the rate, how many bytes can we fit in the snapMsec time of the client
    // // normal rate / snapshotMsec calculation
    // rate = (*cl).rate;
    // if ( sv_maxRate->integer != 0 ) {
    //     if ( sv_maxRate->integer < 1000 ) {
    //         Cvar_Set( b"sv_MaxRate\0".as_ptr() as *const c_char, b"1000\0".as_ptr() as *const c_char );
    //     }
    //     if ( sv_maxRate->integer < rate ) {
    //         rate = sv_maxRate->integer;
    //     }
    // }

    // if rate == 0 {
    //     blockspersnap = 1;
    // } else {
    //     blockspersnap = ( (rate * (*cl).snapshotMsec) / 1000 + MAX_DOWNLOAD_BLKSIZE ) /
    //         MAX_DOWNLOAD_BLKSIZE;
    // }

    // if (blockspersnap < 0) {
    //     blockspersnap = 1;
    // }

    // while (blockspersnap > 0) {
    //     blockspersnap -= 1;

    //     // Write out the next section of the file, if we have already reached our window,
    //     // automatically start retransmitting

    //     if ((*cl).downloadClientBlock == (*cl).downloadCurrentBlock) {
    //         return; // Nothing to transmit
    //     }

    //     if ((*cl).downloadXmitBlock == (*cl).downloadCurrentBlock) {
    //         // We have transmitted the complete window, should we start resending?

    //         //FIXME:  This uses a hardcoded one second timeout for lost blocks
    //         //the timeout should be based on client rate somehow
    //         if (svs.time - (*cl).downloadSendTime > 1000) {
    //             (*cl).downloadXmitBlock = (*cl).downloadClientBlock;
    //         } else {
    //             return;
    //         }
    //     }

    //     // Send current block
    //     curindex = (((*cl).downloadXmitBlock % MAX_DOWNLOAD_WINDOW) as usize);

    //     MSG_WriteByte( msg, svc_download );
    //     MSG_WriteShort( msg, (*cl).downloadXmitBlock );

    //     // block zero is special, contains file size
    //     if ( (*cl).downloadXmitBlock == 0 ) {
    //         MSG_WriteLong( msg, (*cl).downloadSize );
    //     }

    //     MSG_WriteShort( msg, (*cl).downloadBlockSize[curindex] );

    //     // Write the block
    //     if ( (*cl).downloadBlockSize[curindex] != 0 ) {
    //         MSG_WriteData( msg, (*cl).downloadBlocks[curindex] as *const c_void, (*cl).downloadBlockSize[curindex] );
    //     }

    //     Com_DPrintf( "clientDownload: %d : writing block %d\n", cl.offset_from(svs.clients) as c_int, (*cl).downloadXmitBlock );

    //     // Move on to the next block
    //     // It will get sent with next snap shot.  The rate will keep us in line.
    //     (*cl).downloadXmitBlock += 1;

    //     (*cl).downloadSendTime = svs.time;
    // }
}

/*
=================
SV_Disconnect_f

The client is going to disconnect, so remove the connection immediately  FIXME: move to game?
=================
*/
extern "C" {
    pub fn SV_GetStringEdString(refSection: *const c_char, refName: *const c_char) -> *const c_char;
}

unsafe fn SV_Disconnect_f(cl: *mut client_t) {
    // SV_DropClient( cl, "disconnected" );
    // SV_DropClient( cl, SV_GetStringEdString(b"MP_SVGAME\0".as_ptr() as *const c_char, b"DISCONNECTED\0".as_ptr() as *const c_char) );
}

/*
=================
SV_VerifyPaks_f

If we are pure, disconnect the client if they do no meet the following conditions:

1. the first two checksums match our view of cgame and ui
2. there are no any additional checksums that we do not have

This routine would be a bit simpler with a goto but i abstained

=================
*/
#[cfg(not(feature = "xbox"))]
unsafe fn SV_VerifyPaks_f(cl: *mut client_t) {
    // let mut nChkSum1: c_int;
    // let mut nChkSum2: c_int;
    // let mut nClientPaks: c_int;
    // let mut nServerPaks: c_int;
    // let mut i: c_int;
    // let mut j: c_int;
    // let mut nCurArg: c_int;
    // let mut nClientChkSum: [c_int; 1024];
    // let mut nServerChkSum: [c_int; 1024];
    // let mut pPaks: *const c_char;
    // let mut pArg: *const c_char;
    // let mut bGood: c_int = qtrue as c_int;

    // // if we are pure, we "expect" the client to load certain things from
    // // certain pk3 files, namely we want the client to have loaded the
    // // ui and cgame that we think should be loaded based on the pure setting
    // //
    // if ( sv_pure->integer != 0 ) {

    //     bGood = qtrue as c_int;
    //     nChkSum1 = 0;
    //     nChkSum2 = 0;
    //     // we run the game, so determine which cgame and ui the client "should" be running
    //     //dlls are valid too now -rww
    //     if (Cvar_VariableValue( b"vm_cgame\0".as_ptr() as *const c_char ) != 0.0)
    //     {
    //         bGood = (FS_FileIsInPAK(b"vm/cgame.qvm\0".as_ptr() as *const c_char, &mut nChkSum1) == 1) as c_int;
    //     }
    //     else
    //     {
    //         bGood = (FS_FileIsInPAK(b"cgamex86.dll\0".as_ptr() as *const c_char, &mut nChkSum1) == 1) as c_int;
    //     }

    //     if (bGood != 0)
    //     {
    //         if (Cvar_VariableValue( b"vm_ui\0".as_ptr() as *const c_char ) != 0.0)
    //         {
    //             bGood = (FS_FileIsInPAK(b"vm/ui.qvm\0".as_ptr() as *const c_char, &mut nChkSum2) == 1) as c_int;
    //         }
    //         else
    //         {
    //             bGood = (FS_FileIsInPAK(b"uix86.dll\0".as_ptr() as *const c_char, &mut nChkSum2) == 1) as c_int;
    //         }
    //     }

    //     nClientPaks = Cmd_Argc();

    //     // start at arg 1 ( skip cl_paks )
    //     nCurArg = 1;

    //     // we basically use this while loop to avoid using 'goto' :)
    //     loop {
    //         if !bGood != 0 {
    //             break;
    //         }

    //         // must be at least 6: "cl_paks cgame ui @ firstref ... numChecksums"
    //         // numChecksums is encoded
    //         if (nClientPaks < 6) {
    //             bGood = qfalse as c_int;
    //             break;
    //         }
    //         // verify first to be the cgame checksum
    //         pArg = Cmd_Argv(nCurArg);
    //         nCurArg += 1;
    //         if (pArg == ptr::null() || *pArg == b'@' as c_char || atoi(pArg) != nChkSum1 ) {
    //             bGood = qfalse as c_int;
    //             break;
    //         }
    //         // verify the second to be the ui checksum
    //         pArg = Cmd_Argv(nCurArg);
    //         nCurArg += 1;
    //         if (pArg == ptr::null() || *pArg == b'@' as c_char || atoi(pArg) != nChkSum2 ) {
    //             bGood = qfalse as c_int;
    //             break;
    //         }
    //         // should be sitting at the delimeter now
    //         pArg = Cmd_Argv(nCurArg);
    //         nCurArg += 1;
    //         if (*pArg != b'@' as c_char) {
    //             bGood = qfalse as c_int;
    //             break;
    //         }
    //         // store checksums since tokenization is not re-entrant
    //         i = 0;
    //         loop {
    //             if nCurArg >= nClientPaks {
    //                 break;
    //             }
    //             nClientChkSum[i as usize] = atoi(Cmd_Argv(nCurArg));
    //             nCurArg += 1;
    //             i += 1;
    //         }

    //         // store number to compare against (minus one cause the last is the number of checksums)
    //         nClientPaks = i - 1;

    //         // make sure none of the client check sums are the same
    //         // so the client can't send 5 the same checksums
    //         i = 0;
    //         loop {
    //             if i >= nClientPaks {
    //                 break;
    //             }
    //             j = 0;
    //             loop {
    //                 if j >= nClientPaks {
    //                     break;
    //                 }
    //                 if (i == j) {
    //                     j += 1;
    //                     continue;
    //                 }
    //                 if (nClientChkSum[i as usize] == nClientChkSum[j as usize]) {
    //                     bGood = qfalse as c_int;
    //                     break;
    //                 }
    //                 j += 1;
    //             }
    //             if bGood == qfalse as c_int {
    //                 break;
    //             }
    //             i += 1;
    //         }
    //         if bGood == qfalse as c_int {
    //             break;
    //         }

    //         // get the pure checksums of the pk3 files loaded by the server
    //         pPaks = FS_LoadedPakPureChecksums();
    //         Cmd_TokenizeString( pPaks );
    //         nServerPaks = Cmd_Argc();
    //         if (nServerPaks > 1024) {
    //             nServerPaks = 1024;
    //         }

    //         i = 0;
    //         loop {
    //             if i >= nServerPaks {
    //                 break;
    //             }
    //             nServerChkSum[i as usize] = atoi(Cmd_Argv(i));
    //             i += 1;
    //         }

    //         // check if the client has provided any pure checksums of pk3 files not loaded by the server
    //         i = 0;
    //         loop {
    //             if i >= nClientPaks {
    //                 break;
    //             }
    //             j = 0;
    //             loop {
    //                 if j >= nServerPaks {
    //                     break;
    //                 }
    //                 if (nClientChkSum[i as usize] == nServerChkSum[j as usize]) {
    //                     break;
    //                 }
    //                 j += 1;
    //             }
    //             if j >= nServerPaks {
    //                 bGood = qfalse as c_int;
    //                 break;
    //             }
    //             i += 1;
    //         }
    //         if ( bGood == qfalse as c_int ) {
    //             break;
    //         }

    //         // check if the number of checksums was correct
    //         nChkSum1 = sv.checksumFeed;
    //         i = 0;
    //         loop {
    //             if i >= nClientPaks {
    //                 break;
    //             }
    //             nChkSum1 ^= nClientChkSum[i as usize];
    //             i += 1;
    //         }
    //         nChkSum1 ^= nClientPaks;
    //         if (nChkSum1 != nClientChkSum[nClientPaks as usize]) {
    //             bGood = qfalse as c_int;
    //             break;
    //         }

    //         // break out
    //         break;
    //     }

    //     if bGood != 0 {
    //         (*cl).pureAuthentic = 1;
    //     }
    //     else {
    //         (*cl).pureAuthentic = 0;
    //         (*cl).nextSnapshotTime = -1;
    //         (*cl).state = CS_ACTIVE;
    //         SV_SendClientSnapshot( cl );
    //         SV_DropClient( cl, b"Unpure client detected. Invalid .PK3 files referenced!\0".as_ptr() as *const c_char );
    //     }
    // }
}

/*
=================
SV_ResetPureClient_f
=================
*/
unsafe fn SV_ResetPureClient_f(cl: *mut client_t) {
    // (*cl).pureAuthentic = 0;
}

/*
=================
SV_UserinfoChanged

Pull specific info from a newly changed userinfo string
into a more C friendly form.
=================
*/
pub unsafe fn SV_UserinfoChanged(cl: *mut client_t) {
    // let mut val: *mut c_char;
    // let mut i: c_int;

    // // name for C code
    // Q_strncpyz( (*cl).name, Info_ValueForKey ((*cl).userinfo, b"name\0".as_ptr() as *const c_char), mem::size_of_val(&(*cl).name) as c_int );

    // // rate command

    // // if the client is on the same subnet as the server and we aren't running an
    // // internet public server, assume they don't need a rate choke
    // if ( Sys_IsLANAddress( (*cl).netchan.remoteAddress ) && com_dedicated->integer != 2 ) {
    //     (*cl).rate = 99999;	// lans should not rate limit
    // } else {
    //     val = Info_ValueForKey ((*cl).userinfo, b"rate\0".as_ptr() as *const c_char);
    //     if (strlen(val) > 0) {
    //         i = atoi(val);
    //         (*cl).rate = i;
    //         if ((*cl).rate < 1000) {
    //             (*cl).rate = 1000;
    //         } else if ((*cl).rate > 90000) {
    //             (*cl).rate = 90000;
    //         }
    //     } else {
    //         (*cl).rate = 3000;
    //     }
    // }
    // val = Info_ValueForKey ((*cl).userinfo, b"handicap\0".as_ptr() as *const c_char);
    // if (strlen(val) > 0) {
    //     i = atoi(val);
    //     if (i<=0 || i>100 || strlen(val) > 4) {
    //         Info_SetValueForKey( (*cl).userinfo, b"handicap\0".as_ptr() as *const c_char, b"100\0".as_ptr() as *const c_char );
    //     }
    // }

    // // snaps command
    // val = Info_ValueForKey ((*cl).userinfo, b"snaps\0".as_ptr() as *const c_char);
    // if (strlen(val) > 0) {
    //     i = atoi(val);
    //     if ( i < 1 ) {
    //         i = 1;
    //     } else if ( i > 30 ) {
    //         i = 30;
    //     }
    //     (*cl).snapshotMsec = 1000/i;
    // } else {
    //     (*cl).snapshotMsec = 50;
    // }
}

const INFO_CHANGE_MIN_INTERVAL: c_int = 6000; //6 seconds is reasonable I suppose
const INFO_CHANGE_MAX_COUNT: c_int = 3; //only allow 3 changes within the 6 seconds

/*
==================
SV_UpdateUserinfo_f
==================
*/
unsafe fn SV_UpdateUserinfo_f(cl: *mut client_t) {
    // Q_strncpyz( (*cl).userinfo, Cmd_Argv(1), mem::size_of_val(&(*cl).userinfo) as c_int );

    // #ifdef FINAL_BUILD
    //     if ((*cl).lastUserInfoChange > svs.time)
    //     {
    //         (*cl).lastUserInfoCount += 1;

    //         if ((*cl).lastUserInfoCount >= INFO_CHANGE_MAX_COUNT)
    //         {
    //         //	SV_SendServerCommand(cl, "print \"Warning: Too many info changes, last info ignored\n\"\n");
    //             SV_SendServerCommand(cl, b"print \"@@@TOO_MANY_INFO\n\"\n\0".as_ptr() as *const c_char);
    //             return;
    //         }
    //     }
    //     else
    // #endif
    //     {
    //         (*cl).lastUserInfoCount = 0;
    //         (*cl).lastUserInfoChange = svs.time + INFO_CHANGE_MIN_INTERVAL;
    //     }

    //     SV_UserinfoChanged( cl );
    //     // call prog code to allow overrides
    //     VM_Call( gvm, GAME_CLIENT_USERINFO_CHANGED, cl.offset_from(svs.clients) as c_int );
}

#[repr(C)]
pub struct ucmd_t {
    pub name: *const c_char,
    pub func: unsafe extern "C" fn(*mut client_t),
}

#[cfg(feature = "xbox")]
static ucmds: &[ucmd_t] = &[
    ucmd_t {
        name: b"userinfo\0".as_ptr() as *const c_char,
        func: SV_UpdateUserinfo_f,
    },
    ucmd_t {
        name: b"disconnect\0".as_ptr() as *const c_char,
        func: SV_Disconnect_f,
    },
    ucmd_t {
        name: b"cp\0".as_ptr() as *const c_char,
        func: SV_VerifyPaks_f,
    },
    ucmd_t {
        name: b"vdr\0".as_ptr() as *const c_char,
        func: SV_ResetPureClient_f,
    },
    ucmd_t {
        name: ptr::null(),
        func: unsafe extern "C" fn(*mut client_t) {},
    },
];

#[cfg(not(feature = "xbox"))]
static ucmds: &[ucmd_t] = &[
    ucmd_t {
        name: b"userinfo\0".as_ptr() as *const c_char,
        func: SV_UpdateUserinfo_f,
    },
    ucmd_t {
        name: b"disconnect\0".as_ptr() as *const c_char,
        func: SV_Disconnect_f,
    },
    ucmd_t {
        name: b"cp\0".as_ptr() as *const c_char,
        func: SV_VerifyPaks_f,
    },
    ucmd_t {
        name: b"vdr\0".as_ptr() as *const c_char,
        func: SV_ResetPureClient_f,
    },
    ucmd_t {
        name: b"download\0".as_ptr() as *const c_char,
        func: SV_BeginDownload_f,
    },
    ucmd_t {
        name: b"nextdl\0".as_ptr() as *const c_char,
        func: SV_NextDownload_f,
    },
    ucmd_t {
        name: b"stopdl\0".as_ptr() as *const c_char,
        func: SV_StopDownload_f,
    },
    ucmd_t {
        name: b"donedl\0".as_ptr() as *const c_char,
        func: SV_DoneDownload_f,
    },
    ucmd_t {
        name: ptr::null(),
        func: unsafe extern "C" fn(*mut client_t) {},
    },
];

/*
==================
SV_ExecuteClientCommand

Also called by bot code
==================
*/
pub unsafe fn SV_ExecuteClientCommand(cl: *mut client_t, s: *const c_char, clientOK: c_int) {
    // let mut u: *mut ucmd_t;

    // Cmd_TokenizeString( s );

    // // see if it is a server level command
    // u = ucmds.as_ptr() as *mut ucmd_t;
    // loop {
    //     if (*u).name == ptr::null() {
    //         break;
    //     }
    //     if strcmp (Cmd_Argv(0), (*u).name) == 0 {
    //         ((*u).func)( cl );
    //         break;
    //     }
    //     u = u.add(1);
    // }

    // if clientOK != 0 {
    //     // pass unknown strings to the game
    //     if (*u).name == ptr::null() && sv.state == SS_GAME {
    //         VM_Call( gvm, GAME_CLIENT_COMMAND, cl.offset_from(svs.clients) as c_int );
    //     }
    // }
}

/*
===============
SV_ClientCommand
===============
*/
unsafe fn SV_ClientCommand(cl: *mut client_t, msg: *mut msg_t) -> c_int {
    // let mut seq: c_int;
    // let mut s: *const c_char;
    // let mut clientOk: c_int = qtrue as c_int;

    // seq = MSG_ReadLong( msg );
    // s = MSG_ReadString( msg );

    // // see if we have already executed it
    // if ( (*cl).lastClientCommand >= seq ) {
    //     return qtrue as c_int;
    // }

    // Com_DPrintf( "clientCommand: %s : %i : %s\n", (*cl).name, seq, s );

    // // drop the connection if we have somehow lost commands
    // if ( seq > (*cl).lastClientCommand + 1 ) {
    //     Com_Printf( "Client %s lost %i clientCommands\n", (*cl).name,
    //         seq - (*cl).lastClientCommand + 1 );
    //     SV_DropClient( cl, b"Lost reliable commands\0".as_ptr() as *const c_char );
    //     return qfalse as c_int;
    // }

    // // malicious users may try using too many string commands
    // // to lag other players.  If we decide that we want to stall
    // // the command, we will stop processing the rest of the packet,
    // // including the usercmd.  This causes flooders to lag themselves
    // // but not other people
    // // We don't do this when the client hasn't been active yet since its
    // // normal to spam a lot of commands when downloading
    // if ( !com_cl_running->integer &&
    //     (*cl).state >= CS_ACTIVE &&
    //     sv_floodProtect->integer &&
    //     svs.time < (*cl).nextReliableTime ) {
    //     // ignore any other text messages from this client but let them keep playing
    //     clientOk = qfalse as c_int;
    //     Com_DPrintf( "client text ignored for %s\n", (*cl).name );
    //     //return qfalse as c_int;	// stop processing
    // }

    // // don't allow another command for one second
    // (*cl).nextReliableTime = svs.time + 1000;

    // SV_ExecuteClientCommand( cl, s, clientOk );

    // (*cl).lastClientCommand = seq;
    // Com_sprintf((*cl).lastClientCommandString, mem::size_of_val(&(*cl).lastClientCommandString), "%s", s);

    return 1; // continue procesing
}

/*
==================
SV_ClientThink

Also called by bot code
==================
*/
pub unsafe fn SV_ClientThink(cl: *mut client_t, cmd: *mut usercmd_t) {
    // (*cl).lastUsercmd = *cmd;

    // if ( (*cl).state != CS_ACTIVE ) {
    //     return;		// may have been kicked during the last usercmd
    // }

    // VM_Call( gvm, GAME_CLIENT_THINK, cl.offset_from(svs.clients) as c_int );
}

/*
==================
SV_UserMove

The message usually contains all the movement commands
that were in the last three packets, so that the information
in dropped packets can be recovered.

On very fast clients, there may be multiple usercmd packed into
each of the backup packets.
==================
*/
unsafe fn SV_UserMove(cl: *mut client_t, msg: *mut msg_t, delta: c_int) {
    // let mut i: c_int;
    // let mut key: c_int;
    // let mut cmdCount: c_int;
    // let mut nullcmd: usercmd_t;
    // let mut cmds: [usercmd_t; MAX_PACKET_USERCMDS];
    // let mut cmd: *mut usercmd_t;
    // let mut oldcmd: *mut usercmd_t;

    // if ( delta != 0 ) {
    //     (*cl).deltaMessage = (*cl).messageAcknowledge;
    // } else {
    //     (*cl).deltaMessage = -1;
    // }

    // cmdCount = MSG_ReadByte( msg ) as c_int;

    // if ( cmdCount < 1 ) {
    //     Com_Printf( "cmdCount < 1\n" );
    //     return;
    // }

    // if ( cmdCount > MAX_PACKET_USERCMDS ) {
    //     Com_Printf( "cmdCount > MAX_PACKET_USERCMDS\n" );
    //     return;
    // }

    // // use the checksum feed in the key
    // key = sv.checksumFeed;
    // // also use the message acknowledge
    // key ^= (*cl).messageAcknowledge;
    // // also use the last acknowledged server command in the key
    // key ^= Com_HashKey((*cl).reliableCommands[ ((*cl).reliableAcknowledge & (MAX_RELIABLE_COMMANDS-1)) as usize ], 32);

    // Com_Memset( &nullcmd, 0, mem::size_of::<usercmd_t>() as c_int );
    // oldcmd = &nullcmd;
    // i = 0;
    // loop {
    //     if i >= cmdCount {
    //         break;
    //     }
    //     cmd = &mut cmds[i as usize];
    //     MSG_ReadDeltaUsercmdKey( msg, key, oldcmd, cmd );
    //     oldcmd = cmd;
    //     i += 1;
    // }

    // // save time for ping calculation
    // (*cl).frames[ ((*cl).messageAcknowledge & PACKET_MASK) as usize ].messageAcked = svs.time;

    // // if this is the first usercmd we have received
    // // this gamestate, put the client into the world
    // if ( (*cl).state == CS_PRIMED ) {
    //     SV_ClientEnterWorld( cl, &mut cmds[0] );
    //     // the moves can be processed normaly
    // }
    // //
    // #ifndef _XBOX	// No pure on Xbox
    //     if (sv_pure->integer != 0 && (*cl).pureAuthentic == 0) {
    //         SV_DropClient( cl, b"Cannot validate pure client!\0".as_ptr() as *const c_char );
    //         return;
    //     }
    // #endif

    // if ( (*cl).state != CS_ACTIVE ) {
    //     (*cl).deltaMessage = -1;
    //     return;
    // }

    // // usually, the first couple commands will be duplicates
    // // of ones we have previously received, but the servertimes
    // // in the commands will cause them to be immediately discarded
    // i = 0;
    // loop {
    //     if i >= cmdCount {
    //         break;
    //     }
    //     // if this is a cmd from before a map_restart ignore it
    //     if ( cmds[i as usize].serverTime > cmds[(cmdCount-1) as usize].serverTime ) {
    //         i += 1;
    //         continue;
    //     }
    //     // extremely lagged or cmd from before a map_restart
    //     //if ( cmds[i].serverTime > svs.time + 3000 ) {
    //     //	continue;
    //     //}
    //     // don't execute if this is an old cmd which is already executed
    //     // these old cmds are included when cl_packetdup > 0
    //     if ( cmds[i as usize].serverTime <= (*cl).lastUsercmd.serverTime ) {
    //         i += 1;
    //         continue;
    //     }
    //     SV_ClientThink (cl, &mut cmds[ i as usize ]);
    //     i += 1;
    // }
}

/*
===================
SV_ExecuteClientMessage

Parse a client packet
===================
*/
pub unsafe fn SV_ExecuteClientMessage(cl: *mut client_t, msg: *mut msg_t) {
    // let mut c: c_int;
    // let mut serverId: c_int;

    // MSG_Bitstream(msg);

    // serverId = MSG_ReadLong( msg );
    // (*cl).messageAcknowledge = MSG_ReadLong( msg );

    // if ((*cl).messageAcknowledge < 0) {
    //     // usually only hackers create messages like this
    //     // it is more annoying for them to let them hanging
    //     //SV_DropClient( cl, "illegible client message" );
    //     return;
    // }

    // (*cl).reliableAcknowledge = MSG_ReadLong( msg );

    // // NOTE: when the client message is fux0red the acknowledgement numbers
    // // can be out of range, this could cause the server to send thousands of server
    // // commands which the server thinks are not yet acknowledged in SV_UpdateServerCommandsToClient
    // if ((*cl).reliableAcknowledge < (*cl).reliableSequence - MAX_RELIABLE_COMMANDS) {
    //     // usually only hackers create messages like this
    //     // it is more annoying for them to let them hanging
    //     //SV_DropClient( cl, "illegible client message" );
    //     (*cl).reliableAcknowledge = (*cl).reliableSequence;
    //     return;
    // }
    // // if this is a usercmd from a previous gamestate,
    // // ignore it or retransmit the current gamestate
    // //
    // // if the client was downloading, let it stay at whatever serverId and
    // // gamestate it was at.  This allows it to keep downloading even when
    // // the gamestate changes.  After the download is finished, we'll
    // // notice and send it a new game state
    // #ifdef _XBOX	// No downloads on Xbox
    //     if ( serverId != sv.serverId ) {
    // #else
    //     if ( serverId != sv.serverId && !(*cl).downloadName[0] ) {
    // #endif
    //         if ( serverId == sv.restartedServerId ) {
    //             // they just haven't caught the map_restart yet
    //             return;
    //         }
    //         // if we can tell that the client has dropped the last
    //         // gamestate we sent them, resend it
    //         if ( (*cl).messageAcknowledge > (*cl).gamestateMessageNum ) {
    //             Com_DPrintf( "%s : dropped gamestate, resending\n", (*cl).name );
    //             SV_SendClientGameState( cl );
    //         }
    //         return;
    //     }

    //     // read optional clientCommand strings
    //     loop {
    //         c = MSG_ReadByte( msg );
    //         if ( c == clc_EOF ) {
    //             break;
    //         }
    //         if ( c != clc_clientCommand ) {
    //             break;
    //         }
    //         if ( !SV_ClientCommand( cl, msg ) != 0 ) {
    //             return;	// we couldn't execute it because of the flood protection
    //         }
    //         if ((*cl).state == CS_ZOMBIE) {
    //             return;	// disconnect command
    //         }
    //     }

    //     // read the usercmd_t
    //     if ( c == clc_move ) {
    //         SV_UserMove( cl, msg, qtrue as c_int );
    //     } else if ( c == clc_moveNoDelta ) {
    //         SV_UserMove( cl, msg, qfalse as c_int );
    //     } else if ( c != clc_EOF ) {
    //         Com_Printf( "WARNING: bad command byte for client %i\n", cl.offset_from(svs.clients) as c_int );
    //     }
    //     //	if ( msg->readcount != msg->cursize ) {
    //     //		Com_Printf( "WARNING: Junk at end of packet for client %i\n", cl - svs.clients );
    //     //	}
}
