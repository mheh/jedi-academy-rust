//Anything above this #include will be ignored by the compiler

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

//rww - RAGDOLL_BEGIN
//rww - RAGDOLL_END

// persistant server info
pub static mut svs: serverStatic_t = serverStatic_t {
    ips: [0; 4],
    time: 0,
    nextHeartbeatTime: 0,
    nextSnapshotEntities: 0,
    numSnapshotEntities: 0,
    clients: 0 as *mut client_t,
    clientSize: 0,
    redirectAddress: netadr_t { type_: 0, ip: [0; 4], port: 0 },
};

// local server
pub static mut sv: server_t = server_t {
    state: 0,
    time: 0,
    timeResidual: 0,
    snapFlagServerBit: 0,
    configstrings: [0 as *mut c_char; 65536],
    entityBaselines: 0 as *mut c_void,
    restartTime: 0,
    nextmap: [0; 256],
    mapChecksum: 0,
    demoState: 0,
    demoFile: 0 as *mut c_void,
};

// game virtual machine // bk001212 init
pub static mut gvm: *mut vm_t = 0 as *mut vm_t;

pub static mut sv_fps: *mut cvar_t = 0 as *mut cvar_t;				// time rate for running non-clients
pub static mut sv_timeout: *mut cvar_t = 0 as *mut cvar_t;			// seconds without any message
pub static mut sv_zombietime: *mut cvar_t = 0 as *mut cvar_t;			// seconds to sink messages after disconnect
pub static mut sv_rconPassword: *mut cvar_t = 0 as *mut cvar_t;		// password for remote server commands
pub static mut sv_privatePassword: *mut cvar_t = 0 as *mut cvar_t;	// password for the privateClient slots
pub static mut sv_maxclients: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_privateClients: *mut cvar_t = 0 as *mut cvar_t;		// number of clients reserved for password
pub static mut sv_hostname: *mut cvar_t = 0 as *mut cvar_t;
#[cfg(not(feature = "xbox"))]  // No master or downloads on Xbox
pub static mut sv_allowDownload: *mut cvar_t = 0 as *mut cvar_t;
#[cfg(not(feature = "xbox"))]
pub static mut sv_master: [*mut cvar_t; 4] = [0 as *mut cvar_t; 4];		// master server ip address
pub static mut sv_reconnectlimit: *mut cvar_t = 0 as *mut cvar_t;		// minimum seconds between connect messages
pub static mut sv_showghoultraces: *mut cvar_t = 0 as *mut cvar_t;	// report ghoul2 traces
pub static mut sv_showloss: *mut cvar_t = 0 as *mut cvar_t;			// report when usercmds are lost
pub static mut sv_padPackets: *mut cvar_t = 0 as *mut cvar_t;			// add nop bytes to messages
pub static mut sv_killserver: *mut cvar_t = 0 as *mut cvar_t;			// menu system can set to 1 to shut server down
pub static mut sv_mapname: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_mapChecksum: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_serverid: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_maxRate: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_minPing: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_maxPing: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_gametype: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_pure: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_floodProtect: *mut cvar_t = 0 as *mut cvar_t;
pub static mut sv_needpass: *mut cvar_t = 0 as *mut cvar_t;
#[cfg(feature = "USE_CD_KEY")]
pub static mut sv_allowAnonymous: *mut cvar_t = 0 as *mut cvar_t;

/*
=============================================================================

EVENT MESSAGES

=============================================================================
*/

/*
===============
SV_ExpandNewlines

Converts newlines to "\n" so a line prints nicer
===============
*/
pub fn SV_ExpandNewlines(in_str: *mut c_char) -> *mut c_char {
    static mut string: [c_char; 1024] = [0; 1024];
    let mut l: c_int = 0;

    let mut in_ptr = in_str;
    unsafe {
        while *in_ptr != 0 && l < 1024 - 3 {
            if *in_ptr == b'\n' as c_char {
                string[l as usize] = b'\\' as c_char;
                l += 1;
                string[l as usize] = b'n' as c_char;
                l += 1;
            } else {
                string[l as usize] = *in_ptr;
                l += 1;
            }
            in_ptr = in_ptr.offset(1);
        }
        string[l as usize] = 0;

        addr_of_mut!(string) as *mut c_char
    }
}

/*
======================
SV_ReplacePendingServerCommands

  This is ugly
======================
*/
pub fn SV_ReplacePendingServerCommands(client: *mut client_t, cmd: *const c_char) -> c_int {
    let mut i: c_int;
    let mut index: c_int;
    let mut csnum1: c_int = 0;
    let mut csnum2: c_int = 0;

    unsafe {
        i = (*client).reliableSent + 1;
        while i <= (*client).reliableSequence {
            index = i & (MAX_RELIABLE_COMMANDS - 1);
            //
            if Q_strncmp(cmd, (*client).reliableCommands[index as usize].as_ptr(), strlen(b"cs\0".as_ptr() as *const c_char)) == 0 {
                sscanf(cmd, b"cs %i\0".as_ptr() as *const c_char, addr_of_mut!(csnum1));
                sscanf((*client).reliableCommands[index as usize].as_ptr(), b"cs %i\0".as_ptr() as *const c_char, addr_of_mut!(csnum2));
                if csnum1 == csnum2 {
                    Q_strncpyz(
                        (*client).reliableCommands[index as usize].as_mut_ptr(),
                        cmd,
                        std::mem::size_of_val(&(*client).reliableCommands[index as usize]) as c_int
                    );
                    /*
                    if ( client->netchan.remoteAddress.type != NA_BOT ) {
                        Com_Printf( "WARNING: client %i removed double pending config string %i: %s\n", client-svs.clients, csnum1, cmd );
                    }
                    */
                    return 1; // qtrue
                }
            }
            i += 1;
        }
        return 0; // qfalse
    }
}

/*
======================
SV_AddServerCommand

The given command will be transmitted to the client, and is guaranteed to
not have future snapshot_t executed before it is executed
======================
*/
pub fn SV_AddServerCommand(client: *mut client_t, cmd: *const c_char) {
    let mut index: c_int;
    let mut i: c_int;

    // this is very ugly but it's also a waste to for instance send multiple config string updates
    // for the same config string index in one snapshot
//	if ( SV_ReplacePendingServerCommands( client, cmd ) ) {
//		return;
//	}

    unsafe {
        (*client).reliableSequence += 1;
        // if we would be losing an old command that hasn't been acknowledged,
        // we must drop the connection
        // we check == instead of >= so a broadcast print added by SV_DropClient()
        // doesn't cause a recursive drop client
        if (*client).reliableSequence - (*client).reliableAcknowledge == MAX_RELIABLE_COMMANDS + 1 {
            Com_Printf(b"===== pending server commands =====\n\0".as_ptr() as *const c_char);
            i = (*client).reliableAcknowledge + 1;
            while i <= (*client).reliableSequence {
                Com_Printf(b"cmd %5d: %s\n\0".as_ptr() as *const c_char, i, (*client).reliableCommands[(i & (MAX_RELIABLE_COMMANDS - 1)) as usize].as_ptr());
                i += 1;
            }
            Com_Printf(b"cmd %5d: %s\n\0".as_ptr() as *const c_char, i, cmd);
            SV_DropClient(client, b"Server command overflow\0".as_ptr() as *const c_char);
            return;
        }
        index = (*client).reliableSequence & (MAX_RELIABLE_COMMANDS - 1);
        Q_strncpyz(
            (*client).reliableCommands[index as usize].as_mut_ptr(),
            cmd,
            std::mem::size_of_val(&(*client).reliableCommands[index as usize]) as c_int
        );
    }
}


/*
=================
SV_SendServerCommand

Sends a reliable command string to be interpreted by
the client game module: "cp", "print", "chat", etc
A NULL client will broadcast to all clients
=================
*/
pub unsafe extern "C" fn SV_SendServerCommand(cl: *mut client_t, fmt: *const c_char, ...) {
    let mut message: [u8; 65536] = [0; 65536]; // MAX_MSGLEN
    let mut client: *mut client_t;
    let mut j: c_int;

    if !cl.is_null() {
        SV_AddServerCommand(cl, fmt);
        return;
    }

    // hack to echo broadcast prints to console
    if !com_dedicated.is_null() && (*com_dedicated).integer != 0 && libc_strncmp(message.as_ptr() as *const c_char, b"print\0".as_ptr() as *const c_char, 5) == 0 {
        Com_Printf(b"broadcast: %s\n\0".as_ptr() as *const c_char, SV_ExpandNewlines(message.as_mut_ptr() as *mut c_char));
    }

    // send the data to all relevent clients
    j = 0;
    client = (*addr_of!(svs)).clients;
    while j < (*addr_of!(sv_maxclients)).integer {
        if (*client).state < 2 { // CS_PRIMED
            j += 1;
            client = client.offset(1);
            continue;
        }
        SV_AddServerCommand(client, message.as_ptr() as *const c_char);
        j += 1;
        client = client.offset(1);
    }
}


/*
==============================================================================

MASTER SERVER FUNCTIONS

==============================================================================
*/
#[cfg(not(feature = "xbox"))]	// No master on Xbox
const NEW_RESOLVE_DURATION: c_int = 86400000; //24 hours
#[cfg(not(feature = "xbox"))]
static mut g_lastResolveTime: [c_int; 4] = [0; 4]; // MAX_MASTER_SERVERS

#[cfg(not(feature = "xbox"))]
#[inline]
pub fn SV_MasterNeedsResolving(server: c_int, time: c_int) -> bool {
    //refresh every so often regardless of if the actual address was modified -rww
    unsafe {
        if g_lastResolveTime[server as usize] > time {
            //time flowed backwards?
            return true;
        }

        if (time - g_lastResolveTime[server as usize]) > NEW_RESOLVE_DURATION {
            //it's time again
            return true;
        }

        return false;
    }
}

/*
================
SV_MasterHeartbeat

Send a message to the masters every few minutes to
let it know we are alive, and log information.
We will also have a heartbeat sent when a server
changes from empty to non-empty, and full to non-full,
but not on every player enter or exit.
================
*/
#[cfg(not(feature = "xbox"))]
const HEARTBEAT_MSEC: c_int = 300*1000;
#[cfg(not(feature = "xbox"))]
const HEARTBEAT_GAME: &[u8] = b"QuakeArena-1\0";

#[cfg(not(feature = "xbox"))]
pub fn SV_MasterHeartbeat() {
    static mut adr: [netadr_t; 4] = [netadr_t { ip: [0; 4], port: 0, type_: 0 }; 4]; // MAX_MASTER_SERVERS
    let mut i: c_int;
    let mut time: c_int;

    // "dedicated 1" is for lan play, "dedicated 2" is for inet public play
    unsafe {
        if com_dedicated.is_null() || (*com_dedicated).integer != 2 {
            return;		// only dedicated servers send heartbeats
        }

        // if not time yet, don't send anything
        if (*addr_of!(svs)).time < (*addr_of!(svs)).nextHeartbeatTime {
            return;
        }
        (*addr_of_mut!(svs)).nextHeartbeatTime = (*addr_of!(svs)).time + HEARTBEAT_MSEC;

        //we need to use this instead of svs.time since svs.time resets over map changes (or rather
        //every time the game restarts), and we don't really need to resolve every map change
        time = Com_Milliseconds();

        // send to group masters
        i = 0;
        while i < 4 { // MAX_MASTER_SERVERS
            if (*(*addr_of!(sv_master[i as usize]))).string[0] == 0 {
                i += 1;
                continue;
            }

            // see if we haven't already resolved the name
            // resolving usually causes hitches on win95, so only
            // do it when needed
            if (*(*addr_of!(sv_master[i as usize]))).modified != 0 || SV_MasterNeedsResolving(i, time) {
                (*(*addr_of_mut!(sv_master[i as usize]))).modified = 0;

                g_lastResolveTime[i as usize] = time;

                Com_Printf(b"Resolving %s\n\0".as_ptr() as *const c_char, (*(*addr_of!(sv_master[i as usize]))).string.as_ptr() as *const c_char);
                if NET_StringToAdr((*(*addr_of!(sv_master[i as usize]))).string.as_ptr() as *const c_char, addr_of_mut!(adr[i as usize])) == 0 {
                    // if the address failed to resolve, clear it
                    // so we don't take repeated dns hits
                    Com_Printf(b"Couldn't resolve address: %s\n\0".as_ptr() as *const c_char, (*(*addr_of!(sv_master[i as usize]))).string.as_ptr() as *const c_char);
                    Cvar_Set((*(*addr_of!(sv_master[i as usize]))).name.as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
                    (*(*addr_of_mut!(sv_master[i as usize]))).modified = 0;
                    i += 1;
                    continue;
                }
                if strstr(b":\0".as_ptr() as *const c_char, (*(*addr_of!(sv_master[i as usize]))).string.as_ptr() as *const c_char) == std::ptr::null() {
                    adr[i as usize].port = BigShort(10066); // PORT_MASTER
                }
                Com_Printf(b"%s resolved to %i.%i.%i.%i:%i\n\0".as_ptr() as *const c_char, (*(*addr_of!(sv_master[i as usize]))).string.as_ptr() as *const c_char,
                    adr[i as usize].ip[0] as c_int, adr[i as usize].ip[1] as c_int, adr[i as usize].ip[2] as c_int, adr[i as usize].ip[3] as c_int,
                    BigShort(adr[i as usize].port) as c_int);
            }


            Com_Printf(b"Sending heartbeat to %s\n\0".as_ptr() as *const c_char, (*(*addr_of!(sv_master[i as usize]))).string.as_ptr() as *const c_char);
            // this command should be changed if the server info / status format
            // ever incompatably changes
            NET_OutOfBandPrint(0, adr[i as usize], b"heartbeat %s\n\0".as_ptr() as *const c_char, HEARTBEAT_GAME.as_ptr() as *const c_char);
            i += 1;
        }
    }
}

/*
=================
SV_MasterShutdown

Informs all masters that this server is going down
=================
*/
#[cfg(not(feature = "xbox"))]
pub fn SV_MasterShutdown() {
    // send a hearbeat right now
    unsafe {
        (*addr_of_mut!(svs)).nextHeartbeatTime = -9999;
        SV_MasterHeartbeat();

        // send it again to minimize chance of drops
        (*addr_of_mut!(svs)).nextHeartbeatTime = -9999;
        SV_MasterHeartbeat();

        // when the master tries to poll the server, it won't respond, so
        // it will be removed from the list
    }
}


/*
==============================================================================

CONNECTIONLESS COMMANDS

==============================================================================
*/

/*
================
SVC_Status

Responds with all the info that qplug or qspy can see about the server
and all connected players.  Used for getting detailed information after
the simple info query.
================
*/
pub fn SVC_Status(from: netadr_t) {
    let mut player: [c_char; 1024] = [0; 1024];
    let mut status: [c_char; 65536] = [0; 65536]; // MAX_MSGLEN
    let mut i: c_int;
    let mut cl: *mut client_t;
    let mut ps: *mut playerState_t;
    let mut statusLength: c_int;
    let mut playerLength: c_int;
    let mut infostring: [c_char; 512] = [0; 512]; // MAX_INFO_STRING

    unsafe {
        // ignore if we are in single player
        /*
        if ( Cvar_VariableValue( "g_gametype" ) == GT_SINGLE_PLAYER ) {
            return;
        }
        */

        libc_strcpy(infostring.as_mut_ptr(), Cvar_InfoString(0x00000001) as *const c_char); // CVAR_SERVERINFO

        // echo back the parameter to status. so master servers can use it as a challenge
        // to prevent timed spoofed reply packets that add ghost servers
        Info_SetValueForKey(infostring.as_mut_ptr(), b"challenge\0".as_ptr() as *const c_char, Cmd_Argv(1));

        // add "demo" to the sv_keywords if restricted
        if Cvar_VariableValue(b"fs_restrict\0".as_ptr() as *const c_char) != 0.0 {
            let mut keywords: [c_char; 512] = [0; 512]; // MAX_INFO_STRING

            Com_sprintf(keywords.as_mut_ptr(), std::mem::size_of_val(&keywords) as c_int, b"demo %s\0".as_ptr() as *const c_char,
                Info_ValueForKey(infostring.as_ptr(), b"sv_keywords\0".as_ptr() as *const c_char));
            Info_SetValueForKey(infostring.as_mut_ptr(), b"sv_keywords\0".as_ptr() as *const c_char, keywords.as_ptr());
        }

        status[0] = 0;
        statusLength = 0;

        i = 0;
        while i < (*addr_of!(sv_maxclients)).integer {
            cl = (*addr_of!(svs)).clients.offset(i as isize);
            if (*cl).state >= 1 { // CS_CONNECTED
                ps = SV_GameClientNum(i);
                Com_sprintf(player.as_mut_ptr(), std::mem::size_of_val(&player) as c_int, b"%i %i \"%s\"\n\0".as_ptr() as *const c_char,
                    (*ps).persistant[0] as c_int, (*cl).ping, (*cl).name.as_ptr() as *const c_char); // PERS_SCORE
                playerLength = strlen(player.as_ptr()) as c_int;
                if statusLength + playerLength >= std::mem::size_of_val(&status) as c_int {
                    break;		// can't hold any more
                }
                libc_strcpy(status.as_mut_ptr().offset(statusLength as isize), player.as_ptr());
                statusLength += playerLength;
            }
            i += 1;
        }

        NET_OutOfBandPrint(0, from, b"statusResponse\n%s\n%s\0".as_ptr() as *const c_char, infostring.as_ptr(), status.as_ptr());
    }
}

/*
================
SVC_Info

Responds with a short info message that should be enough to determine
if a user is interested in a server to do a full status
================
*/
pub fn SVC_Info(from: netadr_t) {
    let mut i: c_int;
    let mut count: c_int;
    let mut wDisable: c_int;
    let mut gamedir: *const c_char;
    let mut infostring: [c_char; 512] = [0; 512]; // MAX_INFO_STRING

    unsafe {
        // ignore if we are in single player
        /*
        if ( Cvar_VariableValue( "g_gametype" ) == GT_SINGLE_PLAYER || Cvar_VariableValue("ui_singlePlayerActive")) {
            return;
        }
        */

        #[cfg(feature = "xbox")]
        {
            // don't send system link info if in Xbox Live
            // if (logged_on)
            //     return;
        }

        if Cvar_VariableValue(b"ui_singlePlayerActive\0".as_ptr() as *const c_char) != 0.0 {
            return;
        }

        // don't count privateclients
        count = 0;
        i = (*addr_of!(sv_privateClients)).integer;
        while i < (*addr_of!(sv_maxclients)).integer {
            if (*(*addr_of!(svs)).clients.offset(i as isize)).state >= 1 { // CS_CONNECTED
                count += 1;
            }
            i += 1;
        }

        infostring[0] = 0;

        // echo back the parameter to status. so servers can use it as a challenge
        // to prevent timed spoofed reply packets that add ghost servers
        Info_SetValueForKey(infostring.as_mut_ptr(), b"challenge\0".as_ptr() as *const c_char, Cmd_Argv(1));

        Info_SetValueForKey(infostring.as_mut_ptr(), b"protocol\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, 0));  // PROTOCOL_VERSION
        Info_SetValueForKey(infostring.as_mut_ptr(), b"hostname\0".as_ptr() as *const c_char, (*(*addr_of!(sv_hostname))).string.as_ptr() as *const c_char);
        Info_SetValueForKey(infostring.as_mut_ptr(), b"mapname\0".as_ptr() as *const c_char, (*(*addr_of!(sv_mapname))).string.as_ptr() as *const c_char);
        Info_SetValueForKey(infostring.as_mut_ptr(), b"clients\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, count));
        Info_SetValueForKey(infostring.as_mut_ptr(), b"sv_maxclients\0".as_ptr() as *const c_char,
            va(b"%i\0".as_ptr() as *const c_char, (*(*addr_of!(sv_maxclients)).integer - (*(*addr_of!(sv_privateClients))).integer)));
        Info_SetValueForKey(infostring.as_mut_ptr(), b"gametype\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*(*addr_of!(sv_gametype))).integer));
        Info_SetValueForKey(infostring.as_mut_ptr(), b"needpass\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*(*addr_of!(sv_needpass))).integer));
        Info_SetValueForKey(infostring.as_mut_ptr(), b"truejedi\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, Cvar_VariableIntegerValue(b"g_jediVmerc\0".as_ptr() as *const c_char)));
        if (*(*addr_of!(sv_gametype))).integer == 3 || (*(*addr_of!(sv_gametype))).integer == 4 { // GT_DUEL || GT_POWERDUEL
            wDisable = Cvar_VariableIntegerValue(b"g_duelWeaponDisable\0".as_ptr() as *const c_char);
        } else {
            wDisable = Cvar_VariableIntegerValue(b"g_weaponDisable\0".as_ptr() as *const c_char);
        }
        Info_SetValueForKey(infostring.as_mut_ptr(), b"wdisable\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, wDisable));
        Info_SetValueForKey(infostring.as_mut_ptr(), b"fdisable\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, Cvar_VariableIntegerValue(b"g_forcePowerDisable\0".as_ptr() as *const c_char)));
        //Info_SetValueForKey( infostring, "pure", va("%i", sv_pure->integer ) );

        if (*(*addr_of!(sv_minPing))).integer != 0 {
            Info_SetValueForKey(infostring.as_mut_ptr(), b"minPing\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*(*addr_of!(sv_minPing))).integer));
        }
        if (*(*addr_of!(sv_maxPing))).integer != 0 {
            Info_SetValueForKey(infostring.as_mut_ptr(), b"maxPing\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*(*addr_of!(sv_maxPing))).integer));
        }
        gamedir = Cvar_VariableString(b"fs_game\0".as_ptr() as *const c_char);
        if *gamedir != 0 {
            Info_SetValueForKey(infostring.as_mut_ptr(), b"game\0".as_ptr() as *const c_char, gamedir);
        }
        #[cfg(feature = "USE_CD_KEY")]
        Info_SetValueForKey(infostring.as_mut_ptr(), b"sv_allowAnonymous\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*(*addr_of!(sv_allowAnonymous))).integer));

        #[cfg(feature = "xbox")]
        {
            // // Include Xbox specific networking info
            // char sxnkid[XNKID_STRING_LEN];
            // XNKIDToString(SysLink_GetXNKID(), sxnkid);
            // Info_SetValueForKey(infostring, "xnkid", sxnkid);

            // char sxnkey[XNKEY_STRING_LEN];
            // XNKEYToString(SysLink_GetXNKEY(), sxnkey);
            // Info_SetValueForKey(infostring, "xnkey", sxnkey);

            // char sxnaddr[XNADDR_STRING_LEN];
            // XnAddrToString(Net_GetXNADDR(), sxnaddr);
            // Info_SetValueForKey(infostring, "xnaddr", sxnaddr);
        }

        NET_OutOfBandPrint(0, from, b"infoResponse\n%s\0".as_ptr() as *const c_char, infostring.as_ptr());
    }
}

/*
================
SVC_FlushRedirect

================
*/
pub fn SV_FlushRedirect(outputbuf: *mut c_char) {
    unsafe {
        NET_OutOfBandPrint(0, (*addr_of!(svs)).redirectAddress, b"print\n%s\0".as_ptr() as *const c_char, outputbuf);
    }
}

/*
===============
SVC_RemoteCommand

An rcon packet arrived from the network.
Shift down the remaining args
Redirect all printfs
===============
*/
pub fn SVC_RemoteCommand(from: netadr_t, msg: *mut msg_t) {
    let mut valid: c_int;
    let mut i: u32;
    let mut time: u32;
    let mut remaining: [c_char; 1024] = [0; 1024];
    const SV_OUTPUTBUF_LENGTH: usize = 65536 - 16; // MAX_MSGLEN - 16
    let mut sv_outputbuf: [c_char; SV_OUTPUTBUF_LENGTH] = [0; SV_OUTPUTBUF_LENGTH];
    static mut lasttime: u32 = 0;

    unsafe {
        time = Com_Milliseconds() as u32;
        if time < (lasttime + 500) {
            return;
        }
        lasttime = time;

        if strlen((*(*addr_of!(sv_rconPassword))).string.as_ptr()) == 0 ||
            libc_strcmp(Cmd_Argv(1), (*(*addr_of!(sv_rconPassword))).string.as_ptr()) != 0 {
            valid = 0; // qfalse
            Com_DPrintf(b"Bad rcon from %s:\n%s\n\0".as_ptr() as *const c_char, NET_AdrToString(from), Cmd_Argv(2));
        } else {
            valid = 1; // qtrue
            Com_DPrintf(b"Rcon from %s:\n%s\n\0".as_ptr() as *const c_char, NET_AdrToString(from), Cmd_Argv(2));
        }

        // start redirecting all print outputs to the packet
        (*addr_of_mut!(svs)).redirectAddress = from;
        Com_BeginRedirect(sv_outputbuf.as_mut_ptr(), SV_OUTPUTBUF_LENGTH as c_int, SV_FlushRedirect);

        if strlen((*(*addr_of!(sv_rconPassword))).string.as_ptr()) == 0 {
            Com_Printf(b"No rconpassword set.\n\0".as_ptr() as *const c_char);
        } else if valid == 0 {
            Com_Printf(b"Bad rconpassword.\n\0".as_ptr() as *const c_char);
        } else {
            remaining[0] = 0;

            i = 2;
            while i < Cmd_Argc() as u32 {
                libc_strcat(remaining.as_mut_ptr(), Cmd_Argv(i as c_int));
                libc_strcat(remaining.as_mut_ptr(), b" \0".as_ptr() as *const c_char);
                i += 1;
            }

            Cmd_ExecuteString(remaining.as_mut_ptr());
        }

        Com_EndRedirect();
    }
}

/*
=================
SV_ConnectionlessPacket

A connectionless packet has four leading 0xff
characters to distinguish it from a game channel.
Clients that are in the game can still send
connectionless packets.
=================
*/
pub fn SV_ConnectionlessPacket(from: netadr_t, msg: *mut msg_t) {
    let mut s: *const c_char;
    let mut c: *const c_char;

    unsafe {
        MSG_BeginReadingOOB(msg);
        MSG_ReadLong(msg);		// skip the -1 marker

        if Q_strncmp(b"connect\0".as_ptr() as *const c_char, (*msg).data.as_ptr().offset(4) as *const c_char, 7) == 0 {
            Huff_Decompress(msg, 12);
        }

        s = MSG_ReadStringLine(msg);
        Cmd_TokenizeString(s);

        c = Cmd_Argv(0);
        Com_DPrintf(b"SV packet %s : %s\n\0".as_ptr() as *const c_char, NET_AdrToString(from), c);

        if Q_stricmp(c, b"getstatus\0".as_ptr() as *const c_char) == 0 {
            SVC_Status(from);
        } else if Q_stricmp(c, b"getinfo\0".as_ptr() as *const c_char) == 0 {
            SVC_Info(from);
        } else if Q_stricmp(c, b"getchallenge\0".as_ptr() as *const c_char) == 0 {
            SV_GetChallenge(from);
        } else if Q_stricmp(c, b"connect\0".as_ptr() as *const c_char) == 0 {
            SV_DirectConnect(from);
        #[cfg(not(feature = "xbox"))]	// No authorization on Xbox
        } else if Q_stricmp(c, b"ipAuthorize\0".as_ptr() as *const c_char) == 0 {
            SV_AuthorizeIpPacket(from);
        #[cfg(feature = "xbox")]
        } else {
            // dummy branch for Xbox build
        }
        if Q_stricmp(c, b"rcon\0".as_ptr() as *const c_char) == 0 {
            SVC_RemoteCommand(from, msg);
        } else if Q_stricmp(c, b"disconnect\0".as_ptr() as *const c_char) == 0 {
            // if a client starts up a local server, we may see some spurious
            // server disconnect messages when their new server sees our final
            // sequenced messages to the old client
        } else {
            Com_DPrintf(b"bad connectionless packet from %s:\n%s\n\0".as_ptr() as *const c_char, NET_AdrToString(from), s);
        }
    }
}


//============================================================================

/*
=================
SV_ReadPackets
=================
*/
pub fn SV_PacketEvent(from: netadr_t, msg: *mut msg_t) {
    let mut i: c_int;
    let mut cl: *mut client_t;
    let mut qport: c_int;

    unsafe {
        // check for connectionless packet (0xffffffff) first
        if (*msg).cursize >= 4 && *((*msg).data.as_ptr() as *const c_int) == -1 {
            SV_ConnectionlessPacket(from, msg);
            return;
        }

        // read the qport out of the message so we can fix up
        // stupid address translating routers
        MSG_BeginReadingOOB(msg);
        MSG_ReadLong(msg);				// sequence number
        qport = MSG_ReadShort(msg) & 0xffff;

        // find which client the message is from
        i = 0;
        cl = (*addr_of!(svs)).clients;
        while i < (*addr_of!(sv_maxclients)).integer {
            if (*cl).state == 0 { // CS_FREE
                i += 1;
                cl = cl.offset(1);
                continue;
            }
            if NET_CompareBaseAdr(from, (*cl).netchan.remoteAddress) == 0 {
                i += 1;
                cl = cl.offset(1);
                continue;
            }
            // it is possible to have multiple clients from a single IP
            // address, so they are differentiated by the qport variable
            if (*cl).netchan.qport != qport {
                i += 1;
                cl = cl.offset(1);
                continue;
            }

            // the IP port can't be used to differentiate them, because
            // some address translating routers periodically change UDP
            // port assignments
            if (*cl).netchan.remoteAddress.port != from.port {
                Com_Printf(b"SV_ReadPackets: fixing up a translated port\n\0".as_ptr() as *const c_char);
                (*cl).netchan.remoteAddress.port = from.port;
            }

            // make sure it is a valid, in sequence packet
            if SV_Netchan_Process(cl, msg) != 0 {
                // zombie clients still need to do the Netchan_Process
                // to make sure they don't need to retransmit the final
                // reliable message, but they don't do any other processing
                if (*cl).state != 5 { // CS_ZOMBIE
                    (*cl).lastPacketTime = (*addr_of!(svs)).time;	// don't timeout
                    SV_ExecuteClientMessage(cl, msg);
                }
            }
            return;
        }

        // if we received a sequenced packet from an address we don't reckognize,
        // send an out of band disconnect packet to it
        NET_OutOfBandPrint(0, from, b"disconnect\0".as_ptr() as *const c_char);
    }
}


/*
===================
SV_CalcPings

Updates the cl->ping variables
===================
*/
pub fn SV_CalcPings() {
    let mut i: c_int;
    let mut j: c_int;
    let mut cl: *mut client_t;
    let mut total: c_int;
    let mut count: c_int;
    let mut delta: c_int;
    let mut ps: *mut playerState_t;

    unsafe {
        i = 0;
        while i < (*addr_of!(sv_maxclients)).integer {
            cl = (*addr_of!(svs)).clients.offset(i as isize);
            if (*cl).state != 4 { // CS_ACTIVE
                (*cl).ping = 999;
                i += 1;
                continue;
            }
            if (*cl).gentity.is_null() {
                (*cl).ping = 999;
                i += 1;
                continue;
            }
            if (*(*cl).gentity).r.svFlags & 1 != 0 { // SVF_BOT
                (*cl).ping = 0;
                i += 1;
                continue;
            }

            total = 0;
            count = 0;
            j = 0;
            while j < 4 { // PACKET_BACKUP
                if (*cl).frames[j as usize].messageAcked <= 0 {
                    j += 1;
                    continue;
                }
                delta = (*cl).frames[j as usize].messageAcked - (*cl).frames[j as usize].messageSent;
                count += 1;
                total += delta;
                j += 1;
            }
            if count == 0 {
                (*cl).ping = 999;
            } else {
                (*cl).ping = total / count;
                if (*cl).ping > 999 {
                    (*cl).ping = 999;
                }
            }

            // let the game dll know about the ping
            ps = SV_GameClientNum(i);
            (*ps).ping = (*cl).ping;
            i += 1;
        }
    }
}

/*
==================
SV_CheckTimeouts

If a packet has not been received from a client for timeout->integer
seconds, drop the conneciton.  Server time is used instead of
realtime to avoid dropping the local client while debugging.

When a client is normally dropped, the client_t goes into a zombie state
for a few seconds to make sure any final reliable message gets resent
if necessary
==================
*/
pub fn SV_CheckTimeouts() {
    let mut i: c_int;
    let mut cl: *mut client_t;
    let mut droppoint: c_int;
    let mut zombiepoint: c_int;

    unsafe {
        droppoint = (*addr_of!(svs)).time - 1000 * (*(*addr_of!(sv_timeout))).integer;
        zombiepoint = (*addr_of!(svs)).time - 1000 * (*(*addr_of!(sv_zombietime))).integer;

        i = 0;
        cl = (*addr_of!(svs)).clients;
        while i < (*addr_of!(sv_maxclients)).integer {
            // message times may be wrong across a changelevel
            if (*cl).lastPacketTime > (*addr_of!(svs)).time {
                (*cl).lastPacketTime = (*addr_of!(svs)).time;
            }

            if (*cl).state == 5 && (*cl).lastPacketTime < zombiepoint { // CS_ZOMBIE
                Com_DPrintf(b"Going from CS_ZOMBIE to CS_FREE for %s\n\0".as_ptr() as *const c_char, (*cl).name.as_ptr() as *const c_char);
                (*cl).state = 0;	// can now be reused // CS_FREE
                i += 1;
                cl = cl.offset(1);
                continue;
            }
            if (*cl).state >= 1 && (*cl).lastPacketTime < droppoint { // CS_CONNECTED
                // wait several frames so a debugger session doesn't
                // cause a timeout
                (*cl).timeoutCount += 1;
                if (*cl).timeoutCount > 5 {
                    SV_DropClient(cl, b"timed out\0".as_ptr() as *const c_char);
                    (*cl).state = 0;	// don't bother with zombie state // CS_FREE
                }
            } else {
                (*cl).timeoutCount = 0;
            }
            i += 1;
            cl = cl.offset(1);
        }
    }
}


/*
==================
SV_CheckPaused
==================
*/
pub fn SV_CheckPaused() -> c_int {
    let mut count: c_int;
    let mut cl: *mut client_t;
    let mut i: c_int;

    unsafe {
        if (*addr_of!(cl_paused)).integer == 0 {
            return 0; // qfalse
        }

        // only pause if there is just a single client connected
        count = 0;
        i = 0;
        cl = (*addr_of!(svs)).clients;
        while i < (*addr_of!(sv_maxclients)).integer {
            if (*cl).state >= 1 && (*cl).netchan.remoteAddress.type_ != 0 { // CS_CONNECTED && NA_BOT
                count += 1;
            }
            i += 1;
            cl = cl.offset(1);
        }

        if count > 1 {
            // don't pause
            (*addr_of_mut!(sv_paused)).integer = 0;
            return 0; // qfalse
        }

        (*addr_of_mut!(sv_paused)).integer = 1;
        return 1; // qtrue
    }
}

/*
==================
SV_CheckCvars
==================
*/
pub fn SV_CheckCvars() {
    static mut lastMod: c_int = -1;
    let mut changed: c_int = 0;

    unsafe {
        if (*(*addr_of!(sv_hostname))).modificationCount != lastMod {
            let mut hostname: [c_char; 512] = [0; 512]; // MAX_INFO_STRING
            let mut c: *mut c_char = hostname.as_mut_ptr();
            lastMod = (*(*addr_of!(sv_hostname))).modificationCount;

            libc_strcpy(hostname.as_mut_ptr(), (*(*addr_of!(sv_hostname))).string.as_ptr() as *const c_char);
            while *c != 0 {
                if (*c == b'\\' as c_char) || (*c == b';' as c_char) || (*c == b'"' as c_char) {
                    *c = b'.' as c_char;
                    changed = 1;
                }
                c = c.offset(1);
            }
            if changed != 0 {
                Cvar_Set(b"sv_hostname\0".as_ptr() as *const c_char, hostname.as_ptr());
            }
        }
    }
}

/*
==================
SV_Frame

Player movement occurs as a result of packet events, which
happen before SV_Frame is called
==================
*/
pub fn SV_Frame(msec: c_int) {
    let mut frameMsec: c_int;
    let mut startTime: c_int;

    unsafe {
        // the menu kills the server with this cvar
        if (*(*addr_of!(sv_killserver))).integer != 0 {
            SV_Shutdown(b"Server was killed.\n\0".as_ptr() as *const c_char);
            Cvar_Set(b"sv_killserver\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
            return;
        }

        if (*addr_of!(com_sv_running)).integer == 0 {
            return;
        }

        // allow pause if only the local client is connected
        if SV_CheckPaused() != 0 {
            return;
        }

        // if it isn't time for the next frame, do nothing
        if (*(*addr_of!(sv_fps))).integer < 1 {
            Cvar_Set(b"sv_fps\0".as_ptr() as *const c_char, b"10\0".as_ptr() as *const c_char);
        }
        frameMsec = 1000 / (*(*addr_of!(sv_fps))).integer;

        sv.timeResidual += msec;

        if (*addr_of!(com_dedicated)).integer == 0 {
            SV_BotFrame((*addr_of!(svs)).time + sv.timeResidual);
        }

        if (*addr_of!(com_dedicated)).integer != 0 && sv.timeResidual < frameMsec && (addr_of!(com_timescale).is_null() || (*addr_of!(com_timescale)).value >= 1.0) {
            // NET_Sleep will give the OS time slices until either get a packet
            // or time enough for a server frame has gone by
            NET_Sleep(frameMsec - sv.timeResidual);
            return;
        }

        // if time is about to hit the 32nd bit, kick all clients
        // and clear sv.time, rather
        // than checking for negative time wraparound everywhere.
        // 2giga-milliseconds = 23 days, so it won't be too often
        if (*addr_of!(svs)).time > 0x70000000 {
            SV_Shutdown(b"Restarting server due to time wrapping\0".as_ptr() as *const c_char);
            //Cbuf_AddText( "vstr nextmap\n" );
            Cbuf_AddText(b"map_restart 0\n\0".as_ptr() as *const c_char);
            return;
        }
        // this can happen considerably earlier when lots of clients play and the map doesn't change
        if (*addr_of!(svs)).nextSnapshotEntities >= 0x7FFFFFFE - (*addr_of!(svs)).numSnapshotEntities {
            SV_Shutdown(b"Restarting server due to numSnapshotEntities wrapping\0".as_ptr() as *const c_char);
            //Cbuf_AddText( "vstr nextmap\n" );
            Cbuf_AddText(b"map_restart 0\n\0".as_ptr() as *const c_char);
            return;
        }

        if sv.restartTime != 0 && (*addr_of!(svs)).time >= sv.restartTime {
            sv.restartTime = 0;
            Cbuf_AddText(b"map_restart 0\n\0".as_ptr() as *const c_char);
            return;
        }

        // update infostrings if anything has been changed
        if cvar_modifiedFlags & 0x00000001 != 0 { // CVAR_SERVERINFO
            SV_SetConfigstring(0, Cvar_InfoString(0x00000001)); // CS_SERVERINFO, CVAR_SERVERINFO
            cvar_modifiedFlags &= !0x00000001;
        }
        if cvar_modifiedFlags & 0x00000002 != 0 { // CVAR_SYSTEMINFO
            SV_SetConfigstring(1, Cvar_InfoString_Big(0x00000002)); // CS_SYSTEMINFO, CVAR_SYSTEMINFO
            cvar_modifiedFlags &= !0x00000002;
        }

        if (*addr_of!(com_speeds)).integer != 0 {
            startTime = Sys_Milliseconds();
        } else {
            startTime = 0;	// quite a compiler warning
        }

        // update ping based on the all received frames
        SV_CalcPings();

        if (*addr_of!(com_dedicated)).integer != 0 {
            SV_BotFrame((*addr_of!(svs)).time);
        }

        // run the game simulation in chunks
        while sv.timeResidual >= frameMsec {
            sv.timeResidual -= frameMsec;
            (*addr_of_mut!(svs)).time += frameMsec;

            // let everything in the world think and move
            VM_Call(gvm, 1, (*addr_of!(svs)).time); // GAME_RUN_FRAME
        }

        //rww - RAGDOLL_BEGIN
        G2API_SetTime((*addr_of!(svs)).time, 0);
        //rww - RAGDOLL_END

        if (*addr_of!(com_speeds)).integer != 0 {
            time_game = Sys_Milliseconds() - startTime;
        }

        // check timeouts
        SV_CheckTimeouts();

        // send messages back to the clients
        SV_SendClientMessages();

        SV_CheckCvars();

        // send a heartbeat to the master if needed
        #[cfg(not(feature = "xbox"))]	// No master on Xbox
        SV_MasterHeartbeat();
    }
}

//============================================================================

// Stub types and external declarations - these would need actual definitions
// in the linked code or imported from other modules
#[repr(C)]
pub struct serverStatic_t {
    pub ips: [u8; 4],
    pub time: c_int,
    pub nextHeartbeatTime: c_int,
    pub nextSnapshotEntities: c_int,
    pub numSnapshotEntities: c_int,
    pub clients: *mut client_t,
    pub clientSize: c_int,
    pub redirectAddress: netadr_t,
}

#[repr(C)]
pub struct server_t {
    pub state: c_int,
    pub time: c_int,
    pub timeResidual: c_int,
    pub snapFlagServerBit: c_int,
    pub configstrings: [*mut c_char; 65536],
    pub entityBaselines: *mut c_void,
    pub restartTime: c_int,
    pub nextmap: [c_char; 256],
    pub mapChecksum: c_int,
    pub demoState: c_int,
    pub demoFile: *mut c_void,
}

#[repr(C)]
pub struct vm_t {
    pub dummy: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub string: [c_char; 256],
    pub name: [c_char; 64],
    pub modified: c_int,
    pub modificationCount: c_int,
    pub integer: c_int,
    pub value: f32,
}

#[repr(C)]
pub struct netadr_t {
    pub type_: c_int,
    pub ip: [u8; 4],
    pub port: u16,
}

#[repr(C)]
pub struct client_t {
    pub state: c_int,
    pub name: [c_char; 32],
    pub ping: c_int,
    pub netchan: netchan_t,
    pub reliableSequence: c_int,
    pub reliableSent: c_int,
    pub reliableAcknowledge: c_int,
    pub reliableCommands: [[c_char; 128]; 64], // MAX_RELIABLE_COMMANDS
    pub gentity: *mut gentity_t,
    pub frames: [clientFrame_t; 4], // PACKET_BACKUP
    pub lastPacketTime: c_int,
    pub timeoutCount: c_int,
}

#[repr(C)]
pub struct netchan_t {
    pub remoteAddress: netadr_t,
    pub qport: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub persistant: [c_int; 16],
    pub ping: c_int,
}

#[repr(C)]
pub struct gentity_t {
    pub r: gentity_r_t,
}

#[repr(C)]
pub struct gentity_r_t {
    pub svFlags: c_int,
}

#[repr(C)]
pub struct clientFrame_t {
    pub messageSent: c_int,
    pub messageAcked: c_int,
}

#[repr(C)]
pub struct msg_t {
    pub data: [c_char; 65536],
    pub cursize: c_int,
}

// External function declarations
extern "C" {
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_sprintf(buf: *mut c_char, size: c_int, fmt: *const c_char, ...);
    pub fn Com_Milliseconds() -> c_int;
    pub fn Com_BeginRedirect(buf: *mut c_char, size: c_int, flush: extern "C" fn(*mut c_char));
    pub fn Com_EndRedirect();
    pub fn Cvar_Set(name: *const c_char, value: *const c_char);
    pub fn Cvar_InfoString(flags: c_int) -> *const c_char;
    pub fn Cvar_InfoString_Big(flags: c_int) -> *const c_char;
    pub fn Cvar_VariableValue(name: *const c_char) -> f32;
    pub fn Cvar_VariableString(name: *const c_char) -> *const c_char;
    pub fn Cvar_VariableIntegerValue(name: *const c_char) -> c_int;
    pub fn Cmd_Argv(arg: c_int) -> *const c_char;
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_TokenizeString(text: *const c_char);
    pub fn Cmd_ExecuteString(text: *mut c_char);
    pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    pub fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> c_int;
    pub fn NET_CompareBaseAdr(a: netadr_t, b: netadr_t) -> c_int;
    pub fn NET_OutOfBandPrint(socket: c_int, adr: netadr_t, fmt: *const c_char, ...);
    pub fn NET_AdrToString(a: netadr_t) -> *const c_char;
    pub fn NET_Sleep(time: c_int);
    pub fn SV_GameClientNum(i: c_int) -> *mut playerState_t;
    pub fn SV_DropClient(client: *mut client_t, reason: *const c_char);
    pub fn SV_Shutdown(finalmsg: *const c_char);
    pub fn SV_SetConfigstring(index: c_int, val: *const c_char);
    pub fn SV_Netchan_Process(client: *mut client_t, msg: *mut msg_t) -> c_int;
    pub fn SV_ExecuteClientMessage(client: *mut client_t, msg: *mut msg_t);
    pub fn SV_SendClientMessages();
    pub fn SV_BotFrame(time: c_int);
    pub fn SV_GetChallenge(from: netadr_t);
    pub fn SV_DirectConnect(from: netadr_t);
    pub fn SV_AuthorizeIpPacket(from: netadr_t);
    pub fn VM_Call(vm: *mut vm_t, call_type: c_int, ...) -> c_int;
    pub fn G2API_SetTime(time: c_int, arg: c_int);
    pub fn Sys_Milliseconds() -> c_int;
    pub fn Cbuf_AddText(text: *const c_char);
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
    pub fn BigShort(val: u16) -> u16;
    pub fn Q_strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: c_int);
    pub fn Huff_Decompress(msg: *mut msg_t, start: c_int);
    pub fn MSG_BeginReadingOOB(msg: *mut msg_t);
    pub fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    pub fn MSG_ReadShort(msg: *mut msg_t) -> c_int;
    pub fn MSG_ReadStringLine(msg: *mut msg_t) -> *const c_char;

    pub fn strlen(s: *const c_char) -> usize;
    pub fn libc_strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn libc_strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn libc_strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn libc_strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    pub fn vsprintf(buf: *mut c_char, fmt: *const c_char, ap: *mut c_void) -> c_int;
    pub fn sscanf(s: *const c_char, fmt: *const c_char, ...) -> c_int;

    pub static mut com_dedicated: *mut cvar_t;
    pub static mut com_sv_running: *mut cvar_t;
    pub static mut com_speeds: *mut cvar_t;
    pub static mut com_timescale: *mut cvar_t;
    pub static mut cl_paused: *mut cvar_t;
    pub static mut sv_paused: *mut cvar_t;
    pub static mut cvar_modifiedFlags: c_int;
    pub static mut time_game: c_int;
}

// Note: The MAX_* constants are defined in headers but used here
const MAX_MSGLEN: usize = 65536;
const MAX_RELIABLE_COMMANDS: c_int = 64;
const MAX_MASTER_SERVERS: usize = 4;
const MAX_INFO_STRING: usize = 512;
