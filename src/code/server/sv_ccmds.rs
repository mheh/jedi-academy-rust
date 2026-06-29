// leave this as first line for PCH reasons...
//

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

//
// OPERATOR CONSOLE ONLY COMMANDS
//
// These commands can only be entered from stdin or by a remote operator datagram
//

pub static mut qbLoadTransition: bool = false;

//
// ==================
// SV_SetPlayer
//
// Returns the player
// ==================
//
unsafe fn SV_SetPlayer() -> *mut client_t {
    let cl = addr_of_mut!((*addr_of!(svs)).clients[0]);
    if (*cl).state == 0 {
        Com_Printf(b"Client is not active\n" as *const u8 as *const c_char);
        return core::ptr::null_mut();
    }
    cl
}

//
// =========================================================
// don't call this directly, it should only be called from SV_Map_f() or SV_MapTransition_f()
//
unsafe fn SV_Map_(eForceReload: ForceReload_e) -> bool {
    let map = Cmd_Argv(1);
    if *map == 0 as c_char {
        Com_Printf(b"no map specified\n" as *const u8 as *const c_char);
        return false;
    }

    // make sure the level exists before trying to change, so that
    // a typo at the server console won't end the game
    if strchr(map, b'\\' as c_char) != core::ptr::null() {
        Com_Printf(b"Can\'t have mapnames with a \\\n" as *const u8 as *const c_char);
        return false;
    }

    // #ifndef _XBOX	// Could check for maps/%s/brushes.mle or something...
    let mut expanded: [c_char; 260] = [0; 260]; // MAX_QPATH = 260
    Com_sprintf(
        addr_of_mut!(expanded) as *mut c_char,
        260,
        b"maps/%s.bsp\0" as *const u8 as *const c_char,
        map,
    );
    if FS_ReadFile(addr_of!(expanded) as *const c_char, core::ptr::null_mut()) == -1 {
        Com_Printf(
            b"Can\'t find map %s\n" as *const u8 as *const c_char,
            addr_of!(expanded) as *const c_char,
        );
        // extern	cvar_t	*com_buildScript;
        if !com_buildScript.is_null() && (*com_buildScript).integer != 0 {
            //yes, it's happened, someone deleted a map during my build...
            Com_Error(
                1, // ERR_FATAL
                b"Can\'t find map %s\n" as *const u8 as *const c_char,
                addr_of!(expanded) as *const c_char,
            );
        }
        return false;
    }
    // #endif

    if *map as u8 != b'_' {
        SG_WipeSavegame(b"auto\0" as *const u8 as *const c_char);
    }

    SV_SpawnServer(addr_of!(expanded) as *const c_char, eForceReload, true); // start up the map
    true
}

//
// Save out some player data for later restore if this is a spawn point with KEEP_PREV (spawnflags&1) set...
//
// (now also called by auto-save code to setup the cvars correctly
pub unsafe fn SV_Player_EndOfLevelSave() {
    // I could just call GetClientState() but that's in sv_bot.cpp, and I'm not sure if that's going to be deleted for
    //	the single player build, so here's the guts again...
    //
    let cl = addr_of_mut!((*addr_of!(svs)).clients[0]); // 0 because only ever us as a player

    if !cl.is_null()
        && !(*cl).gentity.is_null()
        && !(*(*cl).gentity).client.is_null()
    // crash fix for voy4->brig transition when you kill Foster.
    //	Shouldn't happen, but does sometimes...
    {
        Cvar_Set(
            b"playersave\0" as *const u8 as *const c_char,
            b"\0" as *const u8 as *const c_char,
        ); // default to blank

        //		clientSnapshot_t*	pFrame = &cl->frames[cl->netchan.outgoingSequence & PACKET_MASK];
        let pState = (*(*cl).gentity).client;
        //				|general info				  |-force powers |-saber 1										   |-saber 2										  |-general saber
        let s = va(
            b"%i %i %i %i %i %i %i %f %f %f %i %i %i %i %i %s %i %i %i %i %i %i %i %i %i %i %i %i %i %i %i %i %s %i %i %i %i %i %i %i %i %i %i %i %i %i %i %i %i %i %i %i %i\0" as *const u8 as *const c_char,
            (*pState).stats[0],
            (*pState).stats[1],
            (*pState).stats[2],
            (*pState).stats[3],
            (*pState).weapon,
            (*pState).weaponstate,
            (*pState).batteryCharge,
            (*pState).viewangles[0],
            (*pState).viewangles[1],
            (*pState).viewangles[2],
            //force power data
            (*pState).forcePowersKnown,
            (*pState).forcePower,
            (*pState).forcePowerMax,
            (*pState).forcePowerRegenRate,
            (*pState).forcePowerRegenAmount,
            //saber 1 data
            (*pState).saber[0].name,
            (*pState).saber[0].blade[0].active,
            (*pState).saber[0].blade[1].active,
            (*pState).saber[0].blade[2].active,
            (*pState).saber[0].blade[3].active,
            (*pState).saber[0].blade[4].active,
            (*pState).saber[0].blade[5].active,
            (*pState).saber[0].blade[6].active,
            (*pState).saber[0].blade[7].active,
            (*pState).saber[0].blade[0].color,
            (*pState).saber[0].blade[1].color,
            (*pState).saber[0].blade[2].color,
            (*pState).saber[0].blade[3].color,
            (*pState).saber[0].blade[4].color,
            (*pState).saber[0].blade[5].color,
            (*pState).saber[0].blade[6].color,
            (*pState).saber[0].blade[7].color,
            //saber 2 data
            (*pState).saber[1].name,
            (*pState).saber[1].blade[0].active,
            (*pState).saber[1].blade[1].active,
            (*pState).saber[1].blade[2].active,
            (*pState).saber[1].blade[3].active,
            (*pState).saber[1].blade[4].active,
            (*pState).saber[1].blade[5].active,
            (*pState).saber[1].blade[6].active,
            (*pState).saber[1].blade[7].active,
            (*pState).saber[1].blade[0].color,
            (*pState).saber[1].blade[1].color,
            (*pState).saber[1].blade[2].color,
            (*pState).saber[1].blade[3].color,
            (*pState).saber[1].blade[4].color,
            (*pState).saber[1].blade[5].color,
            (*pState).saber[1].blade[6].color,
            (*pState).saber[1].blade[7].color,
            //general saber data
            (*pState).saberStylesKnown,
            (*pState).saberAnimLevel,
            (*pState).saberLockEnemy,
            (*pState).saberLockTime,
        );
        Cvar_Set(
            b"playersave\0" as *const u8 as *const c_char,
            s,
        );

        //ammo
        let mut s2 = b"\0" as *const u8 as *const c_char;
        for i in 0..16 {
            // AMMO_MAX = 16
            s2 = va(
                b"%s %i\0" as *const u8 as *const c_char,
                s2,
                (*pState).ammo[i],
            );
        }
        Cvar_Set(b"playerammo\0" as *const u8 as *const c_char, s2);

        //inventory
        s2 = b"\0" as *const u8 as *const c_char;
        for i in 0..14 {
            // INV_MAX = 14
            s2 = va(
                b"%s %i\0" as *const u8 as *const c_char,
                s2,
                (*pState).inventory[i],
            );
        }
        Cvar_Set(b"playerinv\0" as *const u8 as *const c_char, s2);

        // the new JK2 stuff - force powers, etc...
        //
        s2 = b"\0" as *const u8 as *const c_char;
        for i in 0..15 {
            // NUM_FORCE_POWERS = 15
            s2 = va(
                b"%s %i\0" as *const u8 as *const c_char,
                s2,
                (*pState).forcePowerLevel[i],
            );
        }
        Cvar_Set(b"playerfplvl\0" as *const u8 as *const c_char, s2);
    }
}

//
// Restart the server on a different map
//
// extern void	SCR_PrecacheScreenshot();  //scr_scrn.cpp
unsafe fn SV_MapTransition_f() {
    //	SCR_PrecacheScreenshot();
    SV_Player_EndOfLevelSave();

    let spawntarget = Cmd_Argv(2);
    if *spawntarget != 0 as c_char {
        Cvar_Set(b"spawntarget\0" as *const u8 as *const c_char, spawntarget);
    } else {
        Cvar_Set(
            b"spawntarget\0" as *const u8 as *const c_char,
            b"\0" as *const u8 as *const c_char,
        );
    }

    SV_Map_(0); // eForceReload_NOTHING
}

//
// ==================
// SV_Map_f
//
// Restart the server on a different map, but clears a cvar so that typing "map blah" doesn't try and preserve
// player weapons/ammo/etc from the previous level that you haven't really exited (ie ignores KEEP_PREV on spawn points)
// ==================
//
// void SCR_UnprecacheScreenshot();	//scr_scrn.cpp
unsafe fn SV_Map_f() {
    Cvar_Set(
        b"playersave\0" as *const u8 as *const c_char,
        b"\0" as *const u8 as *const c_char,
    );
    Cvar_Set(
        b"spawntarget\0" as *const u8 as *const c_char,
        b"\0" as *const u8 as *const c_char,
    );
    Cvar_Set(b"tier_storyinfo\0" as *const u8 as *const c_char, b"0\0" as *const u8 as *const c_char);
    Cvar_Set(
        b"tiers_complete\0" as *const u8 as *const c_char,
        b"\0" as *const u8 as *const c_char,
    );
    //	SCR_UnprecacheScreenshot();

    let mut eForceReload: ForceReload_e = 0; // eForceReload_NOTHING // default for normal load

    if Q_stricmp(Cmd_Argv(0), b"devmapbsp\0" as *const u8 as *const c_char) == 0 {
        eForceReload = 1; // eForceReload_BSP
    } else if Q_stricmp(Cmd_Argv(0), b"devmapmdl\0" as *const u8 as *const c_char) == 0 {
        eForceReload = 2; // eForceReload_MODELS
    } else if Q_stricmp(Cmd_Argv(0), b"devmapall\0" as *const u8 as *const c_char) == 0 {
        eForceReload = 3; // eForceReload_ALL
    }

    if SV_Map_(eForceReload) {
        // set the cheat value
        // if the level was started with "map <levelname>", then
        // cheats will not be allowed.  If started with "devmap <levelname>"
        // then cheats will be allowed
        if Q_stricmpn(Cmd_Argv(0), b"devmap\0" as *const u8 as *const c_char, 6) == 0 {
            Cvar_Set(b"helpUsObi\0" as *const u8 as *const c_char, b"1\0" as *const u8 as *const c_char);
        } else {
            // #ifdef _XBOX
            //				Cvar_Set( "helpUsObi", "1" );
            // #else
            Cvar_Set(
                b"helpUsObi\0" as *const u8 as *const c_char,
                b"0\0" as *const u8 as *const c_char,
            );
            // #endif
        }
    }
}

//
// ==================
// SV_LoadTransition_f
// ==================
//
pub unsafe fn SV_LoadTransition_f() {
    let map = Cmd_Argv(1);
    if *map == 0 as c_char {
        return;
    }

    qbLoadTransition = true;

    //	SCR_PrecacheScreenshot();
    SV_Player_EndOfLevelSave();

    //Save the full current state of the current map so we can return to it later
    SG_WriteSavegame(
        va(
            b"hub/%s\0" as *const u8 as *const c_char,
            sv_mapname,
        ),
        false,
    );

    //set the spawntarget if there is one
    let spawntarget = Cmd_Argv(2);
    if *spawntarget != 0 as c_char {
        Cvar_Set(b"spawntarget\0" as *const u8 as *const c_char, spawntarget);
    } else {
        Cvar_Set(
            b"spawntarget\0" as *const u8 as *const c_char,
            b"\0" as *const u8 as *const c_char,
        );
    }

    if !SV_TryLoadTransition(map) {
        //couldn't load a savegame
        SV_Map_(0); // eForceReload_NOTHING
    }
    qbLoadTransition = false;
}

//
// ===============================================================
//

//
// ================
// SV_Status_f
// ================
//
unsafe fn SV_Status_f() {
    let mut ping: c_int;

    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n" as *const u8 as *const c_char);
        return;
    }

    Com_Printf(
        b"map: %s\n\0" as *const u8 as *const c_char,
        (*sv_mapname).string,
    );

    Com_Printf(b"num score ping name            lastmsg address               qport rate\n\0" as *const u8 as *const c_char);
    Com_Printf(b"--- ----- ---- --------------- ------- --------------------- ----- -----\n\0" as *const u8 as *const c_char);
    for i in 0..1 {
        let cl = addr_of_mut!((*addr_of!(svs)).clients[i]);
        if (*cl).state == 0 {
            continue;
        }
        Com_Printf(b"%3i \0" as *const u8 as *const c_char, i);
        Com_Printf(
            b"%5i \0" as *const u8 as *const c_char,
            (*(*cl).gentity).client as *mut c_void as c_int, // PERS_SCORE index
        );

        if (*cl).state == 1 {
            // CS_CONNECTED
            Com_Printf(b"CNCT \0" as *const u8 as *const c_char);
        } else if (*cl).state == 2 {
            // CS_ZOMBIE
            Com_Printf(b"ZMBI \0" as *const u8 as *const c_char);
        } else {
            ping = if (*cl).ping < 9999 {
                (*cl).ping
            } else {
                9999
            };
            Com_Printf(b"%4i \0" as *const u8 as *const c_char, ping);
        }

        Com_Printf(b"%s\0" as *const u8 as *const c_char, (*cl).name);
        let l = 16 - strlen((*cl).name) as c_int;
        for _j in 0..l {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }

        Com_Printf(
            b"%7i \0" as *const u8 as *const c_char,
            (*addr_of!(sv)).time - (*cl).lastPacketTime,
        );

        let s = NET_AdrToString((*cl).netchan.remoteAddress);
        Com_Printf(b"%s\0" as *const u8 as *const c_char, s);
        let l = 22 - strlen(s) as c_int;
        for _j in 0..l {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }

        Com_Printf(b"%5i\0" as *const u8 as *const c_char, (*cl).netchan.qport);

        Com_Printf(b" %5i\0" as *const u8 as *const c_char, (*cl).rate);

        Com_Printf(b"\n\0" as *const u8 as *const c_char);
    }
    Com_Printf(b"\n\0" as *const u8 as *const c_char);
}

//
// ===========
// SV_Serverinfo_f
//
// Examine the serverinfo string
// ===========
//
unsafe fn SV_Serverinfo_f() {
    Com_Printf(b"Server info settings:\n\0" as *const u8 as *const c_char);
    Info_Print(Cvar_InfoString(4)); // CVAR_SERVERINFO
}

//
// ===========
// SV_Systeminfo_f
//
// Examine or change the serverinfo string
// ===========
//
unsafe fn SV_Systeminfo_f() {
    Com_Printf(b"System info settings:\n\0" as *const u8 as *const c_char);
    Info_Print(Cvar_InfoString(8)); // CVAR_SYSTEMINFO
}

//
// ===========
// SV_DumpUser_f
//
// Examine all a users info strings FIXME: move to game
// ===========
//
unsafe fn SV_DumpUser_f() {
    // make sure server is running
    if (*com_sv_running).integer == 0 {
        Com_Printf(b"Server is not running.\n\0" as *const u8 as *const c_char);
        return;
    }

    if Cmd_Argc() != 2 {
        Com_Printf(b"Usage: info <userid>\n\0" as *const u8 as *const c_char);
        return;
    }

    let cl = SV_SetPlayer();
    if cl.is_null() {
        return;
    }

    Com_Printf(b"userinfo\n\0" as *const u8 as *const c_char);
    Com_Printf(b"--------\n\0" as *const u8 as *const c_char);
    Info_Print((*cl).userinfo);
}

//
// ===========================================================
//

//
// ==================
// SV_AddOperatorCommands
// ==================
//
pub fn SV_AddOperatorCommands() {
    static mut initialized: bool = false;

    unsafe {
        if initialized {
            return;
        }
        initialized = true;

        Cmd_AddCommand(b"status\0" as *const u8 as *const c_char, SV_Status_f);
        Cmd_AddCommand(b"serverinfo\0" as *const u8 as *const c_char, SV_Serverinfo_f);
        Cmd_AddCommand(b"systeminfo\0" as *const u8 as *const c_char, SV_Systeminfo_f);
        Cmd_AddCommand(b"dumpuser\0" as *const u8 as *const c_char, SV_DumpUser_f);
        Cmd_AddCommand(b"sectorlist\0" as *const u8 as *const c_char, SV_SectorList_f);
        Cmd_AddCommand(b"map\0" as *const u8 as *const c_char, SV_Map_f);
        Cmd_AddCommand(b"devmap\0" as *const u8 as *const c_char, SV_Map_f);
        Cmd_AddCommand(b"devmapbsp\0" as *const u8 as *const c_char, SV_Map_f);
        Cmd_AddCommand(b"devmapmdl\0" as *const u8 as *const c_char, SV_Map_f);
        Cmd_AddCommand(b"devmapsnd\0" as *const u8 as *const c_char, SV_Map_f);
        Cmd_AddCommand(b"devmapall\0" as *const u8 as *const c_char, SV_Map_f);
        Cmd_AddCommand(b"maptransition\0" as *const u8 as *const c_char, SV_MapTransition_f);
        Cmd_AddCommand(b"load\0" as *const u8 as *const c_char, SV_LoadGame_f);
        Cmd_AddCommand(b"loadtransition\0" as *const u8 as *const c_char, SV_LoadTransition_f);
        Cmd_AddCommand(b"save\0" as *const u8 as *const c_char, SV_SaveGame_f);
        Cmd_AddCommand(b"wipe\0" as *const u8 as *const c_char, SV_WipeGame_f);

        // #ifdef _DEBUG
        // extern void UI_Dump_f(void);
        // Cmd_AddCommand ("ui_dump", UI_Dump_f);
        // #endif
    }
}

//
// ==================
// SV_RemoveOperatorCommands
// ==================
//
pub fn SV_RemoveOperatorCommands() {
    // #if 0
    // // removing these won't let the server start again
    // Cmd_RemoveCommand ("status");
    // Cmd_RemoveCommand ("serverinfo");
    // Cmd_RemoveCommand ("systeminfo");
    // Cmd_RemoveCommand ("dumpuser");
    // Cmd_RemoveCommand ("serverrecord");
    // Cmd_RemoveCommand ("serverstop");
    // Cmd_RemoveCommand ("sectorlist");
    // #endif
}

// ============================================================================
// External function declarations (stubs for linking)
// ============================================================================

// Types
#[repr(C)]
pub struct client_t {
    state: c_int,
    gentity: *mut c_void,
    name: *const c_char,
    lastPacketTime: c_int,
    netchan: netchan_t,
    userinfo: *const c_char,
    ping: c_int,
    rate: c_int,
}

#[repr(C)]
pub struct netchan_t {
    remoteAddress: *const c_void,
    qport: c_int,
}

#[repr(C)]
pub struct server_t {
    time: c_int,
}

#[repr(C)]
pub struct serverStatic_t {
    clients: [client_t; 1],
}

#[repr(C)]
pub struct cvar_t {
    integer: c_int,
    string: *const c_char,
}

type ForceReload_e = c_int;

// Global variables (stubs)
pub static mut svs: serverStatic_t = serverStatic_t {
    clients: [client_t {
        state: 0,
        gentity: core::ptr::null_mut(),
        name: core::ptr::null(),
        lastPacketTime: 0,
        netchan: netchan_t {
            remoteAddress: core::ptr::null(),
            qport: 0,
        },
        userinfo: core::ptr::null(),
        ping: 0,
        rate: 0,
    }; 1],
};

pub static mut sv: server_t = server_t { time: 0 };

pub static mut sv_mapname: cvar_t = cvar_t {
    integer: 0,
    string: core::ptr::null(),
};

pub static mut com_sv_running: cvar_t = cvar_t {
    integer: 0,
    string: core::ptr::null(),
};

pub static mut com_buildScript: *mut cvar_t = core::ptr::null_mut();

// External function stubs
extern "C" {
    pub fn Cmd_Argv(argc: c_int) -> *const c_char;
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_AddCommand(cmd_name: *const c_char, function: unsafe extern "C" fn() -> ()) -> ();
    pub fn Cmd_RemoveCommand(cmd_name: *const c_char) -> ();
    pub fn Com_Printf(fmt: *const c_char, ...) -> ();
    pub fn Com_sprintf(buffer: *mut c_char, bufsize: usize, fmt: *const c_char, ...) -> ();
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...) -> ();
    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char) -> ();
    pub fn Cvar_InfoString(bit: c_int) -> *const c_char;
    pub fn Info_Print(info: *const c_char) -> ();
    pub fn FS_ReadFile(filename: *const c_char, buf: *mut *mut c_void) -> c_int;
    pub fn NET_AdrToString(a: *const c_void) -> *const c_char;
    pub fn SV_SpawnServer(mapname: *const c_char, force_reload: c_int, spawn_progs: bool) -> ();
    pub fn SV_LoadGame_f() -> ();
    pub fn SV_SaveGame_f() -> ();
    pub fn SV_WipeGame_f() -> ();
    pub fn SV_SectorList_f() -> ();
    pub fn SV_TryLoadTransition(mapname: *const c_char) -> bool;
    pub fn SG_WriteSavegame(dir: *const c_char, autosave: bool) -> ();
    pub fn SG_WipeSavegame(dirname: *const c_char) -> ();
    pub fn strchr(s: *const c_char, c: c_char) -> *const c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
}
