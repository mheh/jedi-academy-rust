//Anything above this include will be ignored by the compiler

use core::ffi::{c_char, c_int, c_void};

/*
===============================================================================

OPERATOR CONSOLE ONLY COMMANDS

These commands can only be entered from stdin or by a remote operator datagram
===============================================================================
*/

extern "C" {
    fn trap_SP_GetStringTextString(
        va: *const c_char,
        text: *mut c_char,
        size: c_int,
    );
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Cmd_Argc() -> c_int;
    fn Cmd_Argv(i: c_int) -> *mut c_char;
    fn Cmd_Args() -> *mut c_char;
    fn Cmd_AddCommand(name: *const c_char, func: *const c_void);
    fn Cmd_RemoveCommand(name: *const c_char);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: c_int);
    fn Q_CleanStr(string: *mut c_char);
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn sprintf(dest: *mut c_char, fmt: *const c_char, ...) -> c_int;
    fn atoi(s: *const c_char) -> c_int;

    static mut com_sv_running: *const crate::cvar_s;
    static mut sv_maxclients: *const crate::cvar_s;
    static mut sv_gametype: *const crate::cvar_s;
    static mut com_dedicated: *const crate::cvar_s;
    static mut sv_mapname: *const crate::cvar_s;
    static mut svs: crate::server::svs_t;
    static mut sv: crate::server::server_t;
    static mut gvm: *mut c_void;
    static mut com_frameTime: c_int;
}

extern "C" {
    fn Cvar_Get(
        var_name: *const c_char,
        var_value: *const c_char,
        flags: c_int,
    ) -> *const crate::cvar_s;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cvar_SetValue(var_name: *const c_char, value: f32);
    fn Cvar_SetLatched(var_name: *const c_char, value: *const c_char);
    fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
    fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    fn Cvar_InfoString(bit: c_int) -> *const c_char;
    fn Info_Print(s: *const c_char);
    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn NET_StringToAdr(s: *const c_char, a: *mut crate::netadr_t) -> c_int;
    fn NET_AdrToString(a: crate::netadr_t) -> *const c_char;
    fn NET_OutOfBandPrint(sock: c_int, adr: crate::netadr_t, format: *const c_char, ...);
    fn BigShort(l: c_int) -> c_int;
    fn SE_GetString(table_name: *const c_char) -> *const c_char;
    fn va(fmt: *const c_char, ...) -> *const c_char;

    fn SV_SpawnServer(mapname: *const c_char, killBots: c_int, eForceReload: c_int);
    fn SV_MapRestart_f();
    fn SV_Shutdown(finalize: *const c_char);
    fn SV_RestartGameProgs();
    fn SV_GameClientNum(num: c_int) -> *mut crate::playerState_t;
    fn SV_AddServerCommand(client: *mut crate::client_t, cmd: *const c_char);
    fn SV_DropClient(client: *mut crate::client_t, reason: *const c_char);
    fn SV_ClientEnterWorld(client: *mut crate::client_t, cmd: *const c_char);
    fn SV_SetConfigstring(index: c_int, val: *const c_char);
    fn SV_SendServerCommand(client: *mut c_void, fmt: *const c_char, ...);
    fn SV_SectorList_f();

    fn VM_Call(vm: *mut c_void, call: c_int, ...) -> c_int;
    fn VM_ExplicitArgPtr(vm: *mut c_void, arg: c_int) -> *mut c_void;
}

const MAX_QPATH: usize = 256;
const CVAR_SERVERINFO: c_int = 4;
const CVAR_LATCH: c_int = 32;
const CVAR_SYSTEMINFO: c_int = 2;
const AUTHORIZE_SERVER_NAME: *const c_char = b"auth.raven.com\0" as *const _ as *const c_char;
const PORT_AUTHORIZE: c_int = 1024;
const NS_SERVER: c_int = 2;
const SNAPFLAG_SERVERCOUNT: c_int = 1;
const CS_CONNECTED: c_int = 0;
const CS_ZOMBIE: c_int = 1;
const CS_ACTIVE: c_int = 2;
const SS_LOADING: c_int = 0;
const SS_GAME: c_int = 1;
const GT_SINGLE_PLAYER: c_int = 0;
const GT_FFA: c_int = 1;
const NUM_FORCE_POWERS: c_int = 18;
const GAME_RUN_FRAME: c_int = 1;
const GAME_CLIENT_CONNECT: c_int = 2;
const PERS_SCORE: c_int = 0;
const NA_BOT: c_int = 1;
const NA_LOOPBACK: c_int = 2;
const NA_BAD: c_int = 0;
const eForceReload_NOTHING: c_int = 0;
const eForceReload_BSP: c_int = 1;
const eForceReload_MODELS: c_int = 2;
const eForceReload_ALL: c_int = 3;

fn SV_GetStringEdString(refSection: *mut c_char, refName: *mut c_char) -> *const c_char {
    /*
    static char text[1024]={0};
    trap_SP_GetStringTextString(va("%s_%s", refSection, refName), text, sizeof(text));
    return text;
    */

    //Well, it would've been lovely doing it the above way, but it would mean mixing
    //languages for the client depending on what the server is. So we'll mark this as
    //a stringed reference with @@@ and send the refname to the client, and when it goes
    //to print it will get scanned for the stringed reference indication and dealt with
    //properly.
    static mut text: [c_char; 1024] = [0; 1024];
    unsafe {
        Com_sprintf(
            text.as_mut_ptr(),
            1024 as c_int,
            b"@@@%s\0" as *const _ as *const c_char,
            refName,
        );
        text.as_ptr()
    }
}

/*
==================
SV_GetPlayerByName

Returns the player with name from Cmd_Argv(1)
==================
*/
unsafe fn SV_GetPlayerByName() -> *mut crate::client_t {
    let mut cl: *mut crate::client_t;
    let mut i: c_int;
    let mut s: *mut c_char;
    let mut cleanName: [c_char; 64] = [0; 64];

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        return core::ptr::null_mut();
    }

    if Cmd_Argc() < 2 {
        Com_Printf(b"No player specified.\n\0" as *const _ as *const c_char);
        return core::ptr::null_mut();
    }

    s = Cmd_Argv(1);

    // check for a name match
    i = 0;
    cl = svs.clients;
    while i < (*sv_maxclients).integer {
        if (*cl).state == 0 {
            i += 1;
            cl = cl.add(1);
            continue;
        }
        if Q_stricmp((*cl).name.as_ptr(), s) == 0 {
            return cl;
        }

        Q_strncpyz(
            cleanName.as_mut_ptr(),
            (*cl).name.as_ptr(),
            64 as c_int,
        );
        Q_CleanStr(cleanName.as_mut_ptr());
        if Q_stricmp(cleanName.as_ptr(), s) == 0 {
            return cl;
        }
        i += 1;
        cl = cl.add(1);
    }

    Com_Printf(
        b"Player %s is not on the server\n\0" as *const _ as *const c_char,
        s,
    );

    core::ptr::null_mut()
}

/*
==================
SV_GetPlayerByNum

Returns the player with idnum from Cmd_Argv(1)
==================
*/
unsafe fn SV_GetPlayerByNum() -> *mut crate::client_t {
    let mut cl: *mut crate::client_t;
    let mut i: c_int;
    let mut idnum: c_int;
    let mut s: *mut c_char;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        return core::ptr::null_mut();
    }

    if Cmd_Argc() < 2 {
        Com_Printf(b"No player specified.\n\0" as *const _ as *const c_char);
        return core::ptr::null_mut();
    }

    s = Cmd_Argv(1);

    i = 0;
    while *s.add(i as usize) as c_int != 0 {
        if (*s.add(i as usize) as c_int) < ('0' as c_int) || (*s.add(i as usize) as c_int) > ('9' as c_int) {
            Com_Printf(
                b"Bad slot number: %s\n\0" as *const _ as *const c_char,
                s,
            );
            return core::ptr::null_mut();
        }
        i += 1;
    }
    idnum = atoi(s);
    if idnum < 0 || idnum >= (*sv_maxclients).integer {
        Com_Printf(
            b"Bad client slot: %i\n\0" as *const _ as *const c_char,
            idnum,
        );
        return core::ptr::null_mut();
    }

    cl = svs.clients.add(idnum as usize);
    if (*cl).state == 0 {
        Com_Printf(
            b"Client %i is not active\n\0" as *const _ as *const c_char,
            idnum,
        );
        return core::ptr::null_mut();
    }
    cl
}

//=========================================================

/*
==================
SV_Map_f

Restart the server on a different map
==================
*/
unsafe fn SV_Map_f() {
    let mut cmd: *mut c_char;
    let mut map: *mut c_char;
    let mut killBots: c_int;
    let mut cheat: c_int;
    let mut expanded: [c_char; 256] = [0; 256];
    let mut mapname: [c_char; 256] = [0; 256];

    map = Cmd_Argv(1);
    if map.is_null() {
        return;
    }

    // make sure the level exists before trying to change, so that
    // a typo at the server console won't end the game
    if !strchr(map, '\\' as c_int).is_null() {
        Com_Printf(b"Can\'t have mapnames with a \\\n\0" as *const _ as *const c_char);
        return;
    }

    #[cfg(not(target_os = "xbox"))]
    {
        Com_sprintf(
            expanded.as_mut_ptr(),
            256 as c_int,
            b"maps/%s.bsp\0" as *const _ as *const c_char,
            map,
        );
        if FS_ReadFile(expanded.as_ptr(), core::ptr::null_mut() as *mut *mut c_void) == -1 {
            Com_Printf(
                b"Can\'t find map %s\n\0" as *const _ as *const c_char,
                expanded.as_ptr(),
            );
            return;
        }
    }

    // force latched values to get set
    Cvar_Get(
        b"g_gametype\0" as *const _ as *const c_char,
        b"0\0" as *const _ as *const c_char,
        (CVAR_SERVERINFO | CVAR_LATCH) as c_int,
    );

    cmd = Cmd_Argv(0);
    if Q_stricmpn(cmd, b"sp\0" as *const _ as *const c_char, 2) == 0 {
        Cvar_SetValue(
            b"g_gametype\0" as *const _ as *const c_char,
            GT_SINGLE_PLAYER as f32,
        );
        Cvar_SetValue(b"g_doWarmup\0" as *const _ as *const c_char, 0.0);
        // may not set sv_maxclients directly, always set latched
        Cvar_SetLatched(
            b"sv_maxclients\0" as *const _ as *const c_char,
            b"8\0" as *const _ as *const c_char,
        );
        cmd = cmd.add(2);
        cheat = 0;
        killBots = 1;
    } else {
        if Q_stricmpn(cmd, b"devmap\0" as *const _ as *const c_char, 6) == 0
            || Q_stricmp(cmd, b"spdevmap\0" as *const _ as *const c_char) == 0
        {
            cheat = 1;
            killBots = 1;
        } else {
            cheat = 0;
            killBots = 0;
        }
        /*
        if( sv_gametype->integer == GT_SINGLE_PLAYER ) {
            Cvar_SetValue( "g_gametype", GT_FFA );
        }
        */
    }

    // save the map name here cause on a map restart we reload the jampconfig.cfg
    // and thus nuke the arguments of the map command
    Q_strncpyz(mapname.as_mut_ptr(), map, 256 as c_int);

    let mut eForceReload: c_int = eForceReload_NOTHING; // default for normal load

    //	if ( !Q_stricmp( cmd, "devmapbsp") ) {	// not relevant in MP codebase
    //		eForceReload = eForceReload_BSP;
    //	}
    //	else
    if Q_stricmp(cmd, b"devmapmdl\0" as *const _ as *const c_char) == 0 {
        eForceReload = eForceReload_MODELS;
    } else if Q_stricmp(cmd, b"devmapall\0" as *const _ as *const c_char) == 0 {
        eForceReload = eForceReload_ALL;
    }

    // start up the map
    SV_SpawnServer(mapname.as_ptr(), killBots, eForceReload);

    // set the cheat value
    // if the level was started with "map <levelname>", then
    // cheats will not be allowed.  If started with "devmap <levelname>"
    // then cheats will be allowed
    if cheat != 0 {
        Cvar_Set(b"sv_cheats\0" as *const _ as *const c_char, b"1\0" as *const _ as *const c_char);
    } else {
        Cvar_Set(b"sv_cheats\0" as *const _ as *const c_char, b"0\0" as *const _ as *const c_char);
    }
}

/*
================
SV_MapRestart_f

Completely restarts a level, but doesn't send a new gamestate to the clients.
This allows fair starts with variable load times.
================
*/
unsafe fn SV_MapRestart_f_impl() {
    let mut i: c_int;
    let mut client: *mut crate::client_t;
    let mut denied: *const c_char;
    let mut isBot: c_int;
    let mut delay: c_int;

    // make sure we aren't restarting twice in the same frame
    if com_frameTime == sv.serverId {
        return;
    }

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const _ as *const c_char);
        return;
    }

    if sv.restartTime != 0 {
        return;
    }

    if Cmd_Argc() > 1 {
        delay = atoi(Cmd_Argv(1));
    } else {
        delay = 5;
    }
    if delay != 0 {
        sv.restartTime = svs.time + (delay * 1000) as i32;
        SV_SetConfigstring(5, va(b"%i\0" as *const _ as *const c_char, sv.restartTime));
        return;
    }

    // check for changes in variables that can't just be restarted
    // check for maxclients change
    if (*sv_maxclients).modified != 0 || (*sv_gametype).modified != 0 {
        let mut mapname: [c_char; 256] = [0; 256];

        Com_Printf(b"variable change -- restarting.\n\0" as *const _ as *const c_char);
        // restart the map the slow way
        Q_strncpyz(
            mapname.as_mut_ptr(),
            Cvar_VariableString(b"mapname\0" as *const _ as *const c_char),
            256 as c_int,
        );

        SV_SpawnServer(mapname.as_ptr(), 0, eForceReload_NOTHING);
        return;
    }

    // toggle the server bit so clients can detect that a
    // map_restart has happened
    svs.snapFlagServerBit ^= SNAPFLAG_SERVERCOUNT;

    // generate a new serverid
    sv.restartedServerId = sv.serverId;
    sv.serverId = com_frameTime;
    Cvar_Set(b"sv_serverid\0" as *const _ as *const c_char, va(b"%i\0" as *const _ as *const c_char, sv.serverId));

    // reset all the vm data in place without changing memory allocation
    // note that we do NOT set sv.state = SS_LOADING, so configstrings that
    // had been changed from their default values will generate broadcast updates
    sv.state = SS_LOADING;
    sv.restarting = 1;

    SV_RestartGameProgs();

    // run a few frames to allow everything to settle
    i = 0;
    while i < 3 {
        VM_Call(gvm, GAME_RUN_FRAME, svs.time);
        svs.time += 100;
        i += 1;
    }

    sv.state = SS_GAME;
    sv.restarting = 0;

    // connect and begin all the clients
    i = 0;
    while i < (*sv_maxclients).integer {
        client = svs.clients.add(i as usize);

        // send the new gamestate to all connected clients
        if (*client).state < CS_CONNECTED {
            i += 1;
            continue;
        }

        if (*client).netchan.remoteAddress.type_ == NA_BOT {
            isBot = 1;
        } else {
            isBot = 0;
        }

        // add the map_restart command
        SV_AddServerCommand(client, b"map_restart\n\0" as *const _ as *const c_char);

        // connect the client again, without the firstTime flag
        denied = VM_ExplicitArgPtr(gvm, VM_Call(gvm, GAME_CLIENT_CONNECT, i, 0, isBot)) as *const c_char;
        if !denied.is_null() {
            // this generally shouldn't happen, because the client
            // was connected before the level change
            SV_DropClient(client, denied);
            Com_Printf(
                b"SV_MapRestart_f(%d): dropped client %i - denied!\n\0" as *const _ as *const c_char,
                delay,
                i,
            ); // bk010125
            i += 1;
            continue;
        }

        (*client).state = CS_ACTIVE;

        SV_ClientEnterWorld(client, &(*client).lastUsercmd as *const _ as *mut c_char);
        i += 1;
    }

    // run another frame to allow things to look at all the players
    VM_Call(gvm, GAME_RUN_FRAME, svs.time);
    svs.time += 100;
}

//===============================================================

/*
==================
SV_GetPlayerByName

Returns the player with name from Cmd_Argv(1)
==================
*/
unsafe fn SV_GetPlayerByFedName(name: *const c_char) -> *mut crate::client_t {
    let mut cl: *mut crate::client_t;
    let mut i: c_int;
    let mut cleanName: [c_char; 64] = [0; 64];

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        return core::ptr::null_mut();
    }

    // check for a name match
    i = 0;
    cl = svs.clients;
    while i < (*sv_maxclients).integer {
        if (*cl).state == 0 {
            i += 1;
            cl = cl.add(1);
            continue;
        }
        if Q_stricmp((*cl).name.as_ptr(), name) == 0 {
            return cl;
        }

        Q_strncpyz(
            cleanName.as_mut_ptr(),
            (*cl).name.as_ptr(),
            64 as c_int,
        );
        Q_CleanStr(cleanName.as_mut_ptr());
        if Q_stricmp(cleanName.as_ptr(), name) == 0 {
            return cl;
        }
        i += 1;
        cl = cl.add(1);
    }

    core::ptr::null_mut()
}

unsafe fn SV_KickByName(name: *const c_char) {
    let mut cl: *mut crate::client_t;
    let mut i: c_int;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        return;
    }

    cl = SV_GetPlayerByFedName(name);
    if cl.is_null() {
        if Q_stricmp(name, b"all\0" as *const _ as *const c_char) == 0 {
            i = 0;
            cl = svs.clients;
            while i < (*sv_maxclients).integer {
                if (*cl).state == 0 {
                    i += 1;
                    cl = cl.add(1);
                    continue;
                }
                if (*cl).netchan.remoteAddress.type_ == NA_LOOPBACK {
                    i += 1;
                    cl = cl.add(1);
                    continue;
                }
                SV_DropClient(
                    cl,
                    SV_GetStringEdString(
                        b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                        b"WAS_KICKED\0" as *const _ as *const c_char as *mut c_char,
                    ),
                ); // "was kicked" );
                (*cl).lastPacketTime = svs.time; // in case there is a funny zombie
                i += 1;
                cl = cl.add(1);
            }
        } else if Q_stricmp(name, b"allbots\0" as *const _ as *const c_char) == 0 {
            i = 0;
            cl = svs.clients;
            while i < (*sv_maxclients).integer {
                if (*cl).state == 0 {
                    i += 1;
                    cl = cl.add(1);
                    continue;
                }
                if (*cl).netchan.remoteAddress.type_ != NA_BOT {
                    i += 1;
                    cl = cl.add(1);
                    continue;
                }
                SV_DropClient(
                    cl,
                    SV_GetStringEdString(
                        b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                        b"WAS_KICKED\0" as *const _ as *const c_char as *mut c_char,
                    ),
                ); // "was kicked" );
                (*cl).lastPacketTime = svs.time; // in case there is a funny zombie
                i += 1;
                cl = cl.add(1);
            }
        }
        return;
    }
    if (*cl).netchan.remoteAddress.type_ == NA_LOOPBACK {
        //		SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        SV_SendServerCommand(
            core::ptr::null_mut(),
            b"print \"%s\"\0" as *const _ as *const c_char,
            SV_GetStringEdString(
                b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                b"CANNOT_KICK_HOST\0" as *const _ as *const c_char as *mut c_char,
            ),
        );
        return;
    }

    SV_DropClient(
        cl,
        SV_GetStringEdString(
            b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
            b"WAS_KICKED\0" as *const _ as *const c_char as *mut c_char,
        ),
    ); // "was kicked" );
    (*cl).lastPacketTime = svs.time; // in case there is a funny zombie
}

/*
==================
SV_Kick_f

Kick a user off of the server  FIXME: move to game
==================
*/
unsafe fn SV_Kick_f() {
    let mut cl: *mut crate::client_t;
    let mut i: c_int;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const _ as *const c_char);
        return;
    }

    if Cmd_Argc() != 2 {
        Com_Printf(
            b"Usage: kick <player name>\nkick all = kick everyone\nkick allbots = kick all bots\n\0" as *const _ as *const c_char
        );
        return;
    }

    if Q_stricmp(Cmd_Argv(1), b"Padawan\0" as *const _ as *const c_char) == 0 {
        //if you try to kick the default name, also try to kick ""
        SV_KickByName(b"\0" as *const _ as *const c_char);
    }

    cl = SV_GetPlayerByName();
    if cl.is_null() {
        if Q_stricmp(Cmd_Argv(1), b"all\0" as *const _ as *const c_char) == 0 {
            i = 0;
            cl = svs.clients;
            while i < (*sv_maxclients).integer {
                if (*cl).state == 0 {
                    i += 1;
                    cl = cl.add(1);
                    continue;
                }
                if (*cl).netchan.remoteAddress.type_ == NA_LOOPBACK {
                    i += 1;
                    cl = cl.add(1);
                    continue;
                }
                SV_DropClient(
                    cl,
                    SV_GetStringEdString(
                        b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                        b"WAS_KICKED\0" as *const _ as *const c_char as *mut c_char,
                    ),
                ); // "was kicked" );
                (*cl).lastPacketTime = svs.time; // in case there is a funny zombie
                i += 1;
                cl = cl.add(1);
            }
        } else if Q_stricmp(Cmd_Argv(1), b"allbots\0" as *const _ as *const c_char) == 0 {
            i = 0;
            cl = svs.clients;
            while i < (*sv_maxclients).integer {
                if (*cl).state == 0 {
                    i += 1;
                    cl = cl.add(1);
                    continue;
                }
                if (*cl).netchan.remoteAddress.type_ != NA_BOT {
                    i += 1;
                    cl = cl.add(1);
                    continue;
                }
                SV_DropClient(
                    cl,
                    SV_GetStringEdString(
                        b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                        b"WAS_KICKED\0" as *const _ as *const c_char as *mut c_char,
                    ),
                ); // "was kicked" );
                (*cl).lastPacketTime = svs.time; // in case there is a funny zombie
                i += 1;
                cl = cl.add(1);
            }
        }
        return;
    }
    if (*cl).netchan.remoteAddress.type_ == NA_LOOPBACK {
        //		SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        SV_SendServerCommand(
            core::ptr::null_mut(),
            b"print \"%s\"\0" as *const _ as *const c_char,
            SV_GetStringEdString(
                b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                b"CANNOT_KICK_HOST\0" as *const _ as *const c_char as *mut c_char,
            ),
        );
        return;
    }

    SV_DropClient(
        cl,
        SV_GetStringEdString(
            b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
            b"WAS_KICKED\0" as *const _ as *const c_char as *mut c_char,
        ),
    ); // "was kicked" );
    (*cl).lastPacketTime = svs.time; // in case there is a funny zombie
}

/*
==================
SV_Ban_f

Ban a user from being able to play on this server through the auth
server
==================
*/
#[cfg(feature = "USE_CD_KEY")]
unsafe fn SV_Ban_f() {
    let mut cl: *mut crate::client_t;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const _ as *const c_char);
        return;
    }

    if Cmd_Argc() != 2 {
        Com_Printf(b"Usage: banUser <player name>\n\0" as *const _ as *const c_char);
        return;
    }

    cl = SV_GetPlayerByName();

    if cl.is_null() {
        return;
    }

    if (*cl).netchan.remoteAddress.type_ == NA_LOOPBACK {
        //		SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        SV_SendServerCommand(
            core::ptr::null_mut(),
            b"print \"%s\"\0" as *const _ as *const c_char,
            SV_GetStringEdString(
                b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                b"CANNOT_KICK_HOST\0" as *const _ as *const c_char as *mut c_char,
            ),
        );
        return;
    }

    // look up the authorize server's IP
    if svs.authorizeAddress.ip[0] == 0 && svs.authorizeAddress.type_ != NA_BAD {
        Com_Printf(
            b"Resolving %s\n\0" as *const _ as *const c_char,
            AUTHORIZE_SERVER_NAME,
        );
        if NET_StringToAdr(
            AUTHORIZE_SERVER_NAME,
            &mut svs.authorizeAddress as *mut crate::netadr_t,
        ) == 0
        {
            Com_Printf(b"Couldn\'t resolve address\n\0" as *const _ as *const c_char);
            return;
        }
        svs.authorizeAddress.port = BigShort(PORT_AUTHORIZE);
        Com_Printf(
            b"%s resolved to %i.%i.%i.%i:%i\n\0" as *const _ as *const c_char,
            AUTHORIZE_SERVER_NAME,
            svs.authorizeAddress.ip[0],
            svs.authorizeAddress.ip[1],
            svs.authorizeAddress.ip[2],
            svs.authorizeAddress.ip[3],
            BigShort(svs.authorizeAddress.port),
        );
    }

    // otherwise send their ip to the authorize server
    if svs.authorizeAddress.type_ != NA_BAD {
        NET_OutOfBandPrint(
            NS_SERVER,
            svs.authorizeAddress,
            b"banUser %i.%i.%i.%i\0" as *const _ as *const c_char,
            (*cl).netchan.remoteAddress.ip[0],
            (*cl).netchan.remoteAddress.ip[1],
            (*cl).netchan.remoteAddress.ip[2],
            (*cl).netchan.remoteAddress.ip[3],
        );
        Com_Printf(
            b"%s was banned from coming back\n\0" as *const _ as *const c_char,
            (*cl).name.as_ptr(),
        );
    }
}

/*
==================
SV_BanNum_f

Ban a user from being able to play on this server through the auth
server
==================
*/
#[cfg(feature = "USE_CD_KEY")]
unsafe fn SV_BanNum_f() {
    let mut cl: *mut crate::client_t;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const _ as *const c_char);
        return;
    }

    if Cmd_Argc() != 2 {
        Com_Printf(b"Usage: banClient <client number>\n\0" as *const _ as *const c_char);
        return;
    }

    cl = SV_GetPlayerByNum();
    if cl.is_null() {
        return;
    }
    if (*cl).netchan.remoteAddress.type_ == NA_LOOPBACK {
        //		SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        SV_SendServerCommand(
            core::ptr::null_mut(),
            b"print \"%s\"\0" as *const _ as *const c_char,
            SV_GetStringEdString(
                b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                b"CANNOT_KICK_HOST\0" as *const _ as *const c_char as *mut c_char,
            ),
        );
        return;
    }

    // look up the authorize server's IP
    if svs.authorizeAddress.ip[0] == 0 && svs.authorizeAddress.type_ != NA_BAD {
        Com_Printf(
            b"Resolving %s\n\0" as *const _ as *const c_char,
            AUTHORIZE_SERVER_NAME,
        );
        if NET_StringToAdr(
            AUTHORIZE_SERVER_NAME,
            &mut svs.authorizeAddress as *mut crate::netadr_t,
        ) == 0
        {
            Com_Printf(b"Couldn\'t resolve address\n\0" as *const _ as *const c_char);
            return;
        }
        svs.authorizeAddress.port = BigShort(PORT_AUTHORIZE);
        Com_Printf(
            b"%s resolved to %i.%i.%i.%i:%i\n\0" as *const _ as *const c_char,
            AUTHORIZE_SERVER_NAME,
            svs.authorizeAddress.ip[0],
            svs.authorizeAddress.ip[1],
            svs.authorizeAddress.ip[2],
            svs.authorizeAddress.ip[3],
            BigShort(svs.authorizeAddress.port),
        );
    }

    // otherwise send their ip to the authorize server
    if svs.authorizeAddress.type_ != NA_BAD {
        NET_OutOfBandPrint(
            NS_SERVER,
            svs.authorizeAddress,
            b"banUser %i.%i.%i.%i\0" as *const _ as *const c_char,
            (*cl).netchan.remoteAddress.ip[0],
            (*cl).netchan.remoteAddress.ip[1],
            (*cl).netchan.remoteAddress.ip[2],
            (*cl).netchan.remoteAddress.ip[3],
        );
        Com_Printf(
            b"%s was banned from coming back\n\0" as *const _ as *const c_char,
            (*cl).name.as_ptr(),
        );
    }
}

/*
==================
SV_KickNum_f

Kick a user off of the server  FIXME: move to game
==================
*/
unsafe fn SV_KickNum_f() {
    let mut cl: *mut crate::client_t;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const _ as *const c_char);
        return;
    }

    if Cmd_Argc() != 2 {
        Com_Printf(b"Usage: kicknum <client number>\n\0" as *const _ as *const c_char);
        return;
    }

    cl = SV_GetPlayerByNum();
    if cl.is_null() {
        return;
    }
    if (*cl).netchan.remoteAddress.type_ == NA_LOOPBACK {
        //		SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        SV_SendServerCommand(
            core::ptr::null_mut(),
            b"print \"%s\"\0" as *const _ as *const c_char,
            SV_GetStringEdString(
                b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
                b"CANNOT_KICK_HOST\0" as *const _ as *const c_char as *mut c_char,
            ),
        );
        return;
    }

    SV_DropClient(
        cl,
        SV_GetStringEdString(
            b"MP_SVGAME\0" as *const _ as *const c_char as *mut c_char,
            b"WAS_KICKED\0" as *const _ as *const c_char as *mut c_char,
        ),
    ); // "was kicked" );
    (*cl).lastPacketTime = svs.time; // in case there is a funny zombie
}

/*
================
SV_Status_f
================
*/
unsafe fn SV_Status_f() {
    let mut i: c_int;
    let mut cl: *mut crate::client_t;
    let mut ps: *mut crate::playerState_t;
    let mut s: *const c_char;
    let mut ping: c_int;
    let mut state: [c_char; 32] = [0; 32];
    let mut avoidTruncation: c_int = 0;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"%s\0" as *const _ as *const c_char, SE_GetString(b"STR_SERVER_SERVER_NOT_RUNNING\0" as *const _ as *const c_char));
        return;
    }

    if Cmd_Argc() > 1 {
        if Q_stricmp(b"notrunc\0" as *const _ as *const c_char, Cmd_Argv(1)) == 0 {
            avoidTruncation = 1;
        }
    }

    Com_Printf(b"map: %s\n\0" as *const _ as *const c_char, (*sv_mapname).string.as_ptr());

    Com_Printf(
        b"num score ping name            lastmsg address               qport rate\n\0" as *const _ as *const c_char
    );
    Com_Printf(
        b"--- ----- ---- --------------- ------- --------------------- ----- -----\n\0" as *const _ as *const c_char
    );
    i = 0;
    cl = svs.clients;
    while i < (*sv_maxclients).integer {
        if (*cl).state == 0 {
            i += 1;
            cl = cl.add(1);
            continue;
        }

        if (*cl).state == CS_CONNECTED {
            strcpy(state.as_mut_ptr(), b"CNCT \0" as *const _ as *const c_char);
        } else if (*cl).state == CS_ZOMBIE {
            strcpy(state.as_mut_ptr(), b"ZMBI \0" as *const _ as *const c_char);
        } else {
            ping = if (*cl).ping < 9999 { (*cl).ping } else { 9999 };
            sprintf(
                state.as_mut_ptr(),
                b"%4i\0" as *const _ as *const c_char,
                ping,
            );
        }

        ps = SV_GameClientNum(i);
        s = NET_AdrToString((*cl).netchan.remoteAddress);

        if avoidTruncation == 0 {
            Com_Printf(
                b"%3i %5i %s %-15.15s %7i %21s %5i %5i\n\0" as *const _ as *const c_char,
                i,
                (*ps).persistant[PERS_SCORE as usize],
                state.as_ptr(),
                (*cl).name.as_ptr(),
                svs.time - (*cl).lastPacketTime,
                s,
                (*cl).netchan.qport,
                (*cl).rate,
            );
        } else {
            Com_Printf(
                b"%3i %5i %s %s %7i %21s %5i %5i\n\0" as *const _ as *const c_char,
                i,
                (*ps).persistant[PERS_SCORE as usize],
                state.as_ptr(),
                (*cl).name.as_ptr(),
                svs.time - (*cl).lastPacketTime,
                s,
                (*cl).netchan.qport,
                (*cl).rate,
            );
        }
        i += 1;
        cl = cl.add(1);
    }
    Com_Printf(b"\n\0" as *const _ as *const c_char);
}

/*
==================
SV_ConSay_f
==================
*/
unsafe fn SV_ConSay_f() {
    let mut p: *mut c_char;
    let mut text: [c_char; 1024] = [0; 1024];

    if (*com_dedicated).integer == 0 {
        Com_Printf(b"Server is not dedicated.\n\0" as *const _ as *const c_char);
        return;
    }

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const _ as *const c_char);
        return;
    }

    if Cmd_Argc() < 2 {
        return;
    }

    strcpy(text.as_mut_ptr(), b"Server: \0" as *const _ as *const c_char);
    p = Cmd_Args();

    if *p as c_int == ('"' as c_int) {
        p = p.add(1);
        *p.add(strlen(p) - 1) = 0 as c_char;
    }

    strcat(text.as_mut_ptr(), p);

    SV_SendServerCommand(
        core::ptr::null_mut(),
        b"chat \"%s\n\"\0" as *const _ as *const c_char,
        text.as_ptr(),
    );
}

static forceToggleNamePrints: [*const c_char; 19] = [
    b"HEAL\0" as *const _ as *const c_char,                //FP_HEAL
    b"JUMP\0" as *const _ as *const c_char,                //FP_LEVITATION
    b"SPEED\0" as *const _ as *const c_char,               //FP_SPEED
    b"PUSH\0" as *const _ as *const c_char,                //FP_PUSH
    b"PULL\0" as *const _ as *const c_char,                //FP_PULL
    b"MINDTRICK\0" as *const _ as *const c_char,           //FP_TELEPTAHY
    b"GRIP\0" as *const _ as *const c_char,                //FP_GRIP
    b"LIGHTNING\0" as *const _ as *const c_char,           //FP_LIGHTNING
    b"DARK RAGE\0" as *const _ as *const c_char,           //FP_RAGE
    b"PROTECT\0" as *const _ as *const c_char,             //FP_PROTECT
    b"ABSORB\0" as *const _ as *const c_char,              //FP_ABSORB
    b"TEAM HEAL\0" as *const _ as *const c_char,           //FP_TEAM_HEAL
    b"TEAM REPLENISH\0" as *const _ as *const c_char,      //FP_TEAM_FORCE
    b"DRAIN\0" as *const _ as *const c_char,               //FP_DRAIN
    b"SEEING\0" as *const _ as *const c_char,              //FP_SEE
    b"SABER OFFENSE\0" as *const _ as *const c_char,       //FP_SABER_OFFENSE
    b"SABER DEFENSE\0" as *const _ as *const c_char,       //FP_SABER_DEFENSE
    b"SABER THROW\0" as *const _ as *const c_char,         //FP_SABERTHROW
    core::ptr::null(),
];

/*
==================
SV_ForceToggle_f
==================
*/
unsafe fn SV_ForceToggle_f() {
    let mut i: c_int = 0;
    let mut fpDisabled: c_int = Cvar_VariableValue(b"g_forcePowerDisable\0" as *const _ as *const c_char) as c_int;
    let mut targetPower: c_int = 0;
    let mut powerDisabled: *const c_char = b"Enabled\0" as *const _ as *const c_char;

    if Cmd_Argc() < 2 {
        //no argument supplied, spit out a list of force powers and their numbers
        while i < NUM_FORCE_POWERS {
            if (fpDisabled & (1 << i)) != 0 {
                powerDisabled = b"Disabled\0" as *const _ as *const c_char;
            } else {
                powerDisabled = b"Enabled\0" as *const _ as *const c_char;
            }

            Com_Printf(
                b"%s\0" as *const _ as *const c_char,
                va(
                    b"%i - %s - Status: %s\n\0" as *const _ as *const c_char,
                    i,
                    forceToggleNamePrints[i as usize],
                    powerDisabled,
                ),
            );
            i += 1;
        }

        Com_Printf(b"Example usage: forcetoggle 3\n(toggles PUSH)\n\0" as *const _ as *const c_char);
        return;
    }

    targetPower = atoi(Cmd_Argv(1));

    if targetPower < 0 || targetPower >= NUM_FORCE_POWERS {
        Com_Printf(b"Specified a power that does not exist.\nExample usage: forcetoggle 3\n(toggles PUSH)\n\0" as *const _ as *const c_char);
        return;
    }

    if (fpDisabled & (1 << targetPower)) != 0 {
        powerDisabled = b"enabled\0" as *const _ as *const c_char;
        fpDisabled &= !(1 << targetPower);
    } else {
        powerDisabled = b"disabled\0" as *const _ as *const c_char;
        fpDisabled |= 1 << targetPower;
    }

    Cvar_Set(
        b"g_forcePowerDisable\0" as *const _ as *const c_char,
        va(b"%i\0" as *const _ as *const c_char, fpDisabled),
    );

    Com_Printf(
        b"%s has been %s.\n\0" as *const _ as *const c_char,
        forceToggleNamePrints[targetPower as usize],
        powerDisabled,
    );
}

/*
==================
SV_Heartbeat_f

Also called by SV_DropClient, SV_DirectConnect, and SV_SpawnServer
==================
*/
unsafe fn SV_Heartbeat_f() {
    svs.nextHeartbeatTime = -9999999;
}

/*
===========
SV_Serverinfo_f

Examine the serverinfo string
===========
*/
unsafe fn SV_Serverinfo_f() {
    Com_Printf(b"Server info settings:\n\0" as *const _ as *const c_char);
    Info_Print(Cvar_InfoString(CVAR_SERVERINFO));
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const _ as *const c_char);
    }
}

/*
===========
SV_Systeminfo_f

Examine or change the serverinfo string
===========
*/
unsafe fn SV_Systeminfo_f() {
    Com_Printf(b"System info settings:\n\0" as *const _ as *const c_char);
    Info_Print(Cvar_InfoString(CVAR_SYSTEMINFO));
}

/*
===========
SV_DumpUser_f

Examine all a users info strings FIXME: move to game
===========
*/
unsafe fn SV_DumpUser_f() {
    let mut cl: *mut crate::client_t;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const _ as *const c_char);
        return;
    }

    if Cmd_Argc() != 2 {
        Com_Printf(b"Usage: info <userid>\n\0" as *const _ as *const c_char);
        return;
    }

    cl = SV_GetPlayerByName();
    if cl.is_null() {
        return;
    }

    Com_Printf(b"userinfo\n\0" as *const _ as *const c_char);
    Com_Printf(b"--------\n\0" as *const _ as *const c_char);
    Info_Print((*cl).userinfo.as_ptr());
}

/*
=================
SV_KillServer
=================
*/
unsafe fn SV_KillServer_f() {
    SV_Shutdown(b"killserver\0" as *const _ as *const c_char);
}

//===========================================================

/*
==================
SV_AddOperatorCommands
==================
*/
unsafe fn SV_AddOperatorCommands() {
    static mut initialized: c_int = 0;

    if initialized != 0 {
        return;
    }
    initialized = 1;

    Cmd_AddCommand(
        b"heartbeat\0" as *const _ as *const c_char,
        SV_Heartbeat_f as *const c_void,
    );
    Cmd_AddCommand(
        b"kick\0" as *const _ as *const c_char,
        SV_Kick_f as *const c_void,
    );
    #[cfg(feature = "USE_CD_KEY")]
    {
        Cmd_AddCommand(
            b"banUser\0" as *const _ as *const c_char,
            SV_Ban_f as *const c_void,
        );
        Cmd_AddCommand(
            b"banClient\0" as *const _ as *const c_char,
            SV_BanNum_f as *const c_void,
        );
    }

    Cmd_AddCommand(
        b"clientkick\0" as *const _ as *const c_char,
        SV_KickNum_f as *const c_void,
    );
    Cmd_AddCommand(
        b"status\0" as *const _ as *const c_char,
        SV_Status_f as *const c_void,
    );
    Cmd_AddCommand(
        b"serverinfo\0" as *const _ as *const c_char,
        SV_Serverinfo_f as *const c_void,
    );
    Cmd_AddCommand(
        b"systeminfo\0" as *const _ as *const c_char,
        SV_Systeminfo_f as *const c_void,
    );
    Cmd_AddCommand(
        b"dumpuser\0" as *const _ as *const c_char,
        SV_DumpUser_f as *const c_void,
    );
    Cmd_AddCommand(
        b"map_restart\0" as *const _ as *const c_char,
        SV_MapRestart_f_impl as *const c_void,
    );
    Cmd_AddCommand(
        b"sectorlist\0" as *const _ as *const c_char,
        SV_SectorList_f as *const c_void,
    );
    Cmd_AddCommand(
        b"map\0" as *const _ as *const c_char,
        SV_Map_f as *const c_void,
    );
    #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
    {
        Cmd_AddCommand(
            b"devmap\0" as *const _ as *const c_char,
            SV_Map_f as *const c_void,
        );
        Cmd_AddCommand(
            b"spmap\0" as *const _ as *const c_char,
            SV_Map_f as *const c_void,
        );
        Cmd_AddCommand(
            b"spdevmap\0" as *const _ as *const c_char,
            SV_Map_f as *const c_void,
        );
        //	Cmd_AddCommand ("devmapbsp", SV_Map_f);	// not used in MP codebase, no server BSP_cacheing
        Cmd_AddCommand(
            b"devmapmdl\0" as *const _ as *const c_char,
            SV_Map_f as *const c_void,
        );
        Cmd_AddCommand(
            b"devmapall\0" as *const _ as *const c_char,
            SV_Map_f as *const c_void,
        );
    }
    Cmd_AddCommand(
        b"killserver\0" as *const _ as *const c_char,
        SV_KillServer_f as *const c_void,
    );
    //	if( com_dedicated->integer )
    {
        Cmd_AddCommand(
            b"svsay\0" as *const _ as *const c_char,
            SV_ConSay_f as *const c_void,
        );
    }

    Cmd_AddCommand(
        b"forcetoggle\0" as *const _ as *const c_char,
        SV_ForceToggle_f as *const c_void,
    );
}

/*
==================
SV_RemoveOperatorCommands
==================
*/
unsafe fn SV_RemoveOperatorCommands() {
    #[cfg(any())]
    {
        // removing these won't let the server start again
        Cmd_RemoveCommand(b"heartbeat\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"kick\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"banUser\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"banClient\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"status\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"serverinfo\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"systeminfo\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"dumpuser\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"map_restart\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"sectorlist\0" as *const _ as *const c_char);
        Cmd_RemoveCommand(b"svsay\0" as *const _ as *const c_char);
    }
}
