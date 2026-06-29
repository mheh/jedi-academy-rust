// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_consolecmds.c -- text commands typed in at the local console, or
// executed by a key binding

use core::ffi::c_int;

// External function declarations
extern "C" {
    fn CG_CrosshairPlayer() -> c_int;
    fn trap_Argv(arg: c_int, buffer: *mut u8, buf_size: c_int);
    fn trap_SendConsoleCommand(cmd: *const u8);
    fn va(format: *const u8, ...) -> *const u8;
    fn atoi(s: *const u8) -> c_int;
    fn trap_Cvar_Set(name: *const u8, value: *const u8);
    fn CG_Printf(format: *const u8, ...);
    fn CG_BuildSpectatorString();
    fn trap_SendClientCommand(cmd: *const u8);
    fn Menu_Reset();
    fn Menu_ScrollFeeder(menu: *mut core::ffi::c_void, feeder: c_int, down: c_int);
    fn CG_AddBufferedSound(handle: c_int);
    fn CG_CenterPrint(text: *const u8, height: c_int, time: c_int);
    fn CG_GetStringEdString(table: *const u8, key: *const u8) -> *const u8;
    fn trap_Args(buffer: *mut u8, buf_size: c_int);
    fn Com_sprintf(buffer: *mut u8, buf_size: c_int, format: *const u8, ...);
    fn trap_Cvar_VariableStringBuffer(name: *const u8, buffer: *mut u8, buf_size: c_int);
    fn CG_SiegeBriefingDisplay(team: c_int, dontshow: c_int);
    fn CG_Argv(arg: c_int) -> *const u8;
    fn Q_stricmp(s1: *const u8, s2: *const u8) -> c_int;
    fn trap_AddCommand(cmd: *const u8);
    fn CG_LastAttacker() -> c_int;
    fn CG_NextWeapon_f();
    fn CG_PrevWeapon_f();
    fn CG_Weapon_f();
    fn CG_WeaponClean_f();
    fn CG_NextForcePower_f();
    fn CG_PrevForcePower_f();
    fn CG_NextInventory_f();
    fn CG_PrevInventory_f();
    fn CG_TestGun_f();
    fn CG_TestModel_f();
    fn CG_TestModelNextFrame_f();
    fn CG_TestModelPrevFrame_f();
    fn CG_TestModelNextSkin_f();
    fn CG_TestModelPrevSkin_f();
    fn CG_LoadDeferredPlayers();

    // Opaque type definitions for external structs
    static mut cg_viewsize: cvar_t;
    static mut cg_cameraOrbit: cvar_t;
    static mut cgs: cgs_t;
    static mut cg: cg_t;
    pub static mut menuScoreboard: *mut core::ffi::c_void;
}

#[repr(C)]
struct cvar_t {
    // Opaque placeholder for cvar_t structure
}

#[repr(C)]
struct cgs_t {
    // Opaque placeholder for cgs_t structure
}

#[repr(C)]
struct cg_t {
    // Opaque placeholder for cg_t structure
}

fn CG_TargetCommand_f() {
    let mut targetNum: c_int;
    let mut test: [u8; 4] = [0; 4];

    unsafe {
        targetNum = CG_CrosshairPlayer();
        if targetNum == 0 {
            return;
        }

        trap_Argv(1, test.as_mut_ptr(), 4);
        trap_SendConsoleCommand(va(
            b"gc %i %i\0".as_ptr(),
            targetNum,
            atoi(test.as_ptr()),
        ));
    }
}



/*
=================
CG_SizeUp_f

Keybinding command
=================
*/
fn CG_SizeUp_f() {
    unsafe {
        trap_Cvar_Set(
            b"cg_viewsize\0".as_ptr(),
            va(
                b"%i\0".as_ptr(),
                (cg_viewsize.integer + 10) as c_int,
            ),
        );
    }
}


/*
=================
CG_SizeDown_f

Keybinding command
=================
*/
fn CG_SizeDown_f() {
    unsafe {
        trap_Cvar_Set(
            b"cg_viewsize\0".as_ptr(),
            va(
                b"%i\0".as_ptr(),
                (cg_viewsize.integer - 10) as c_int,
            ),
        );
    }
}


/*
=============
CG_Viewpos_f

Debugging command to print the current position
=============
*/
fn CG_Viewpos_f() {
    unsafe {
        CG_Printf(
            b"%s (%i %i %i) : %i\n\0".as_ptr(),
            cgs.mapname as *const u8,
            cg.refdef.vieworg[0] as c_int,
            cg.refdef.vieworg[1] as c_int,
            cg.refdef.vieworg[2] as c_int,
            cg.refdef.viewangles[2] as c_int,  // YAW = 2
        );
    }
}


fn CG_ScoresDown_f() {
    unsafe {
        CG_BuildSpectatorString();
        if cg.scoresRequestTime + 2000 < cg.time {
            // the scores are more than two seconds out of data,
            // so request new ones
            cg.scoresRequestTime = cg.time;
            trap_SendClientCommand(b"score\0".as_ptr());

            // leave the current scores up if they were already
            // displayed, but if this is the first hit, clear them out
            if cg.showScores == 0 {
                cg.showScores = 1; // qtrue
                cg.numScores = 0;
            }
        } else {
            // show the cached contents even if they just pressed if it
            // is within two seconds
            cg.showScores = 1; // qtrue
        }
    }
}

fn CG_ScoresUp_f() {
    unsafe {
        if cg.showScores != 0 {
            cg.showScores = 0; // qfalse
            cg.scoreFadeTime = cg.time;
        }
    }
}

extern "C" {
    pub static mut menuScoreboard: *mut core::ffi::c_void;
    fn Menu_Reset();            // FIXME: add to right include file
}

fn CG_scrollScoresDown_f() {
    unsafe {
        if !menuScoreboard.is_null() && cg.scoreBoardShowing != 0 {
            let menu_ptr = menuScoreboard;
            Menu_ScrollFeeder(menu_ptr, 1, 1); // FEEDER_SCOREBOARD = 1
            Menu_ScrollFeeder(menu_ptr, 4, 1); // FEEDER_REDTEAM_LIST = 4
            Menu_ScrollFeeder(menu_ptr, 5, 1); // FEEDER_BLUETEAM_LIST = 5
        }
    }
}


fn CG_scrollScoresUp_f() {
    unsafe {
        if !menuScoreboard.is_null() && cg.scoreBoardShowing != 0 {
            let menu_ptr = menuScoreboard;
            Menu_ScrollFeeder(menu_ptr, 1, 0); // FEEDER_SCOREBOARD = 1
            Menu_ScrollFeeder(menu_ptr, 4, 0); // FEEDER_REDTEAM_LIST = 4
            Menu_ScrollFeeder(menu_ptr, 5, 0); // FEEDER_BLUETEAM_LIST = 5
        }
    }
}


fn CG_spWin_f() {
    unsafe {
        trap_Cvar_Set(b"cg_cameraOrbit\0".as_ptr(), b"2\0".as_ptr());
        trap_Cvar_Set(b"cg_cameraOrbitDelay\0".as_ptr(), b"35\0".as_ptr());
        trap_Cvar_Set(b"cg_thirdPerson\0".as_ptr(), b"1\0".as_ptr());
        trap_Cvar_Set(b"cg_thirdPersonAngle\0".as_ptr(), b"0\0".as_ptr());
        trap_Cvar_Set(b"cg_thirdPersonRange\0".as_ptr(), b"100\0".as_ptr());
        CG_AddBufferedSound(cgs.media.winnerSound);
        //trap_S_StartLocalSound(cgs.media.winnerSound, CHAN_ANNOUNCER);
        CG_CenterPrint(
            CG_GetStringEdString(b"MP_INGAME\0".as_ptr(), b"YOU_WIN\0".as_ptr()),
            (480.0 * 0.30) as c_int,  // SCREEN_HEIGHT * .30
            0,
        );
    }
}

fn CG_spLose_f() {
    unsafe {
        trap_Cvar_Set(b"cg_cameraOrbit\0".as_ptr(), b"2\0".as_ptr());
        trap_Cvar_Set(b"cg_cameraOrbitDelay\0".as_ptr(), b"35\0".as_ptr());
        trap_Cvar_Set(b"cg_thirdPerson\0".as_ptr(), b"1\0".as_ptr());
        trap_Cvar_Set(b"cg_thirdPersonAngle\0".as_ptr(), b"0\0".as_ptr());
        trap_Cvar_Set(b"cg_thirdPersonRange\0".as_ptr(), b"100\0".as_ptr());
        CG_AddBufferedSound(cgs.media.loserSound);
        //trap_S_StartLocalSound(cgs.media.loserSound, CHAN_ANNOUNCER);
        CG_CenterPrint(
            CG_GetStringEdString(b"MP_INGAME\0".as_ptr(), b"YOU_LOSE\0".as_ptr()),
            (480.0 * 0.30) as c_int,  // SCREEN_HEIGHT * .30
            0,
        );
    }
}


fn CG_TellTarget_f() {
    let mut clientNum: c_int;
    let mut command: [u8; 128] = [0; 128];
    let mut message: [u8; 128] = [0; 128];

    unsafe {
        clientNum = CG_CrosshairPlayer();
        if clientNum == -1 {
            return;
        }

        trap_Args(message.as_mut_ptr(), 128);
        Com_sprintf(
            command.as_mut_ptr(),
            128,
            b"tell %i %s\0".as_ptr(),
            clientNum,
            message.as_ptr(),
        );
        trap_SendClientCommand(command.as_ptr());
    }
}

fn CG_TellAttacker_f() {
    let mut clientNum: c_int;
    let mut command: [u8; 128] = [0; 128];
    let mut message: [u8; 128] = [0; 128];

    unsafe {
        clientNum = CG_LastAttacker();
        if clientNum == -1 {
            return;
        }

        trap_Args(message.as_mut_ptr(), 128);
        Com_sprintf(
            command.as_mut_ptr(),
            128,
            b"tell %i %s\0".as_ptr(),
            clientNum,
            message.as_ptr(),
        );
        trap_SendClientCommand(command.as_ptr());
    }
}


/*
==================
CG_StartOrbit_f
==================
*/

fn CG_StartOrbit_f() {
    let mut var: [u8; 64] = [0; 64];  // MAX_TOKEN_CHARS = 64

    unsafe {
        trap_Cvar_VariableStringBuffer(b"developer\0".as_ptr(), var.as_mut_ptr(), 64);
        if atoi(var.as_ptr()) == 0 {
            return;
        }
        if cg_cameraOrbit.value != 0.0 {
            trap_Cvar_Set(b"cg_cameraOrbit\0".as_ptr(), b"0\0".as_ptr());
            trap_Cvar_Set(b"cg_thirdPerson\0".as_ptr(), b"0\0".as_ptr());
        } else {
            trap_Cvar_Set(b"cg_cameraOrbit\0".as_ptr(), b"5\0".as_ptr());
            trap_Cvar_Set(b"cg_thirdPerson\0".as_ptr(), b"1\0".as_ptr());
            trap_Cvar_Set(b"cg_thirdPersonAngle\0".as_ptr(), b"0\0".as_ptr());
            trap_Cvar_Set(b"cg_thirdPersonRange\0".as_ptr(), b"100\0".as_ptr());
        }
    }
}

extern "C" {
    fn CG_SiegeBriefingDisplay(team: c_int, dontshow: c_int);
}

fn CG_SiegeBriefing_f() {
    let mut team: c_int;

    unsafe {
        if cgs.gametype != 4 {  // GT_SIEGE = 4
            // Cannot be displayed unless in this gametype
            return;
        }

        team = cg.predictedPlayerState.persistant[6];  // PERS_TEAM = 6

        if team != 0 &&  // SIEGETEAM_TEAM1 = 0
            team != 1  // SIEGETEAM_TEAM2 = 1
        {
            // cannot be displayed if not on a valid team
            return;
        }

        CG_SiegeBriefingDisplay(team, 0);
    }
}

fn CG_SiegeCvarUpdate_f() {
    let mut team: c_int;

    unsafe {
        if cgs.gametype != 4 {  // GT_SIEGE = 4
            // Cannot be displayed unless in this gametype
            return;
        }

        team = cg.predictedPlayerState.persistant[6];  // PERS_TEAM = 6

        if team != 0 &&  // SIEGETEAM_TEAM1 = 0
            team != 1  // SIEGETEAM_TEAM2 = 1
        {
            // cannot be displayed if not on a valid team
            return;
        }

        CG_SiegeBriefingDisplay(team, 1);
    }
}

fn CG_SiegeCompleteCvarUpdate_f() {
    unsafe {
        if cgs.gametype != 4 {  // GT_SIEGE = 4
            // Cannot be displayed unless in this gametype
            return;
        }

        // Set up cvars for both teams
        CG_SiegeBriefingDisplay(0, 1);  // SIEGETEAM_TEAM1 = 0
        CG_SiegeBriefingDisplay(1, 1);  // SIEGETEAM_TEAM2 = 1
    }
}

/*
static void CG_Camera_f( void ) {
    char name[1024];
    trap_Argv( 1, name, sizeof(name));
    if (trap_loadCamera(name)) {
        cg.cameraMode = qtrue;
        trap_startCamera(cg.time);
    } else {
        CG_Printf ("Unable to load camera %s\n",name);
    }
}
*/


#[repr(C)]
struct consoleCommand_t {
    cmd: *const u8,
    function: extern "C" fn(),
}

static commands: [consoleCommand_t; 32] = [
    consoleCommand_t {
        cmd: b"testgun\0".as_ptr(),
        function: CG_TestGun_f,
    },
    consoleCommand_t {
        cmd: b"testmodel\0".as_ptr(),
        function: CG_TestModel_f,
    },
    consoleCommand_t {
        cmd: b"nextframe\0".as_ptr(),
        function: CG_TestModelNextFrame_f,
    },
    consoleCommand_t {
        cmd: b"prevframe\0".as_ptr(),
        function: CG_TestModelPrevFrame_f,
    },
    consoleCommand_t {
        cmd: b"nextskin\0".as_ptr(),
        function: CG_TestModelNextSkin_f,
    },
    consoleCommand_t {
        cmd: b"prevskin\0".as_ptr(),
        function: CG_TestModelPrevSkin_f,
    },
    consoleCommand_t {
        cmd: b"viewpos\0".as_ptr(),
        function: CG_Viewpos_f,
    },
    consoleCommand_t {
        cmd: b"+scores\0".as_ptr(),
        function: CG_ScoresDown_f,
    },
    consoleCommand_t {
        cmd: b"-scores\0".as_ptr(),
        function: CG_ScoresUp_f,
    },
    consoleCommand_t {
        cmd: b"sizeup\0".as_ptr(),
        function: CG_SizeUp_f,
    },
    consoleCommand_t {
        cmd: b"sizedown\0".as_ptr(),
        function: CG_SizeDown_f,
    },
    consoleCommand_t {
        cmd: b"weapnext\0".as_ptr(),
        function: CG_NextWeapon_f,
    },
    consoleCommand_t {
        cmd: b"weapprev\0".as_ptr(),
        function: CG_PrevWeapon_f,
    },
    consoleCommand_t {
        cmd: b"weapon\0".as_ptr(),
        function: CG_Weapon_f,
    },
    consoleCommand_t {
        cmd: b"weaponclean\0".as_ptr(),
        function: CG_WeaponClean_f,
    },
    consoleCommand_t {
        cmd: b"tell_target\0".as_ptr(),
        function: CG_TellTarget_f,
    },
    consoleCommand_t {
        cmd: b"tell_attacker\0".as_ptr(),
        function: CG_TellAttacker_f,
    },
    consoleCommand_t {
        cmd: b"tcmd\0".as_ptr(),
        function: CG_TargetCommand_f,
    },
    consoleCommand_t {
        cmd: b"spWin\0".as_ptr(),
        function: CG_spWin_f,
    },
    consoleCommand_t {
        cmd: b"spLose\0".as_ptr(),
        function: CG_spLose_f,
    },
    consoleCommand_t {
        cmd: b"scoresDown\0".as_ptr(),
        function: CG_scrollScoresDown_f,
    },
    consoleCommand_t {
        cmd: b"scoresUp\0".as_ptr(),
        function: CG_scrollScoresUp_f,
    },
    consoleCommand_t {
        cmd: b"startOrbit\0".as_ptr(),
        function: CG_StartOrbit_f,
    },
    //consoleCommand_t { cmd: b"camera\0".as_ptr(), function: CG_Camera_f },
    consoleCommand_t {
        cmd: b"loaddeferred\0".as_ptr(),
        function: CG_LoadDeferredPlayers,
    },
    consoleCommand_t {
        cmd: b"invnext\0".as_ptr(),
        function: CG_NextInventory_f,
    },
    consoleCommand_t {
        cmd: b"invprev\0".as_ptr(),
        function: CG_PrevInventory_f,
    },
    consoleCommand_t {
        cmd: b"forcenext\0".as_ptr(),
        function: CG_NextForcePower_f,
    },
    consoleCommand_t {
        cmd: b"forceprev\0".as_ptr(),
        function: CG_PrevForcePower_f,
    },
    consoleCommand_t {
        cmd: b"briefing\0".as_ptr(),
        function: CG_SiegeBriefing_f,
    },
    consoleCommand_t {
        cmd: b"siegeCvarUpdate\0".as_ptr(),
        function: CG_SiegeCvarUpdate_f,
    },
    consoleCommand_t {
        cmd: b"siegeCompleteCvarUpdate\0".as_ptr(),
        function: CG_SiegeCompleteCvarUpdate_f,
    },
    // Padding entry to match array size
    consoleCommand_t {
        cmd: core::ptr::null(),
        function: CG_TargetCommand_f,  // Dummy function, not used
    },
];


/*
=================
CG_ConsoleCommand

The string has been tokenized and can be retrieved with
Cmd_Argc() / Cmd_Argv()
=================
*/
pub extern "C" fn CG_ConsoleCommand() -> c_int {
    let cmd: *const u8;
    let mut i: c_int;

    unsafe {
        cmd = CG_Argv(0);

        i = 0;
        while i < (commands.len() as c_int) {
            if Q_stricmp(cmd, commands[i as usize].cmd) == 0 {
                (commands[i as usize].function)();
                return 1;  // qtrue
            }
            i += 1;
        }
    }

    return 0;  // qfalse
}


/*
=================
CG_InitConsoleCommands

Let the client system know about all of our commands
so it can perform tab completion
=================
*/
pub extern "C" fn CG_InitConsoleCommands() {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < (commands.len() as c_int) {
            if !commands[i as usize].cmd.is_null() {
                trap_AddCommand(commands[i as usize].cmd);
            }
            i += 1;
        }

        //
        // the game server will interpret these commands, which will be automatically
        // forwarded to the server after they are not recognized locally
        //
        trap_AddCommand(b"forcechanged\0".as_ptr());
        trap_AddCommand(b"sv_invnext\0".as_ptr());
        trap_AddCommand(b"sv_invprev\0".as_ptr());
        trap_AddCommand(b"sv_forcenext\0".as_ptr());
        trap_AddCommand(b"sv_forceprev\0".as_ptr());
        trap_AddCommand(b"sv_saberswitch\0".as_ptr());
        trap_AddCommand(b"engage_duel\0".as_ptr());
        trap_AddCommand(b"force_heal\0".as_ptr());
        trap_AddCommand(b"force_speed\0".as_ptr());
        trap_AddCommand(b"force_throw\0".as_ptr());
        trap_AddCommand(b"force_pull\0".as_ptr());
        trap_AddCommand(b"force_distract\0".as_ptr());
        trap_AddCommand(b"force_rage\0".as_ptr());
        trap_AddCommand(b"force_protect\0".as_ptr());
        trap_AddCommand(b"force_absorb\0".as_ptr());
        trap_AddCommand(b"force_healother\0".as_ptr());
        trap_AddCommand(b"force_forcepowerother\0".as_ptr());
        trap_AddCommand(b"force_seeing\0".as_ptr());
        trap_AddCommand(b"use_seeker\0".as_ptr());
        trap_AddCommand(b"use_field\0".as_ptr());
        trap_AddCommand(b"use_bacta\0".as_ptr());
        trap_AddCommand(b"use_electrobinoculars\0".as_ptr());
        trap_AddCommand(b"zoom\0".as_ptr());
        trap_AddCommand(b"use_sentry\0".as_ptr());
        trap_AddCommand(b"bot_order\0".as_ptr());
        trap_AddCommand(b"saberAttackCycle\0".as_ptr());
        trap_AddCommand(b"kill\0".as_ptr());
        trap_AddCommand(b"say\0".as_ptr());
        trap_AddCommand(b"say_team\0".as_ptr());
        trap_AddCommand(b"tell\0".as_ptr());
        trap_AddCommand(b"give\0".as_ptr());
        trap_AddCommand(b"god\0".as_ptr());
        trap_AddCommand(b"notarget\0".as_ptr());
        trap_AddCommand(b"noclip\0".as_ptr());
        trap_AddCommand(b"team\0".as_ptr());
        trap_AddCommand(b"follow\0".as_ptr());
        trap_AddCommand(b"levelshot\0".as_ptr());
        trap_AddCommand(b"addbot\0".as_ptr());
        trap_AddCommand(b"setviewpos\0".as_ptr());
        trap_AddCommand(b"callvote\0".as_ptr());
        trap_AddCommand(b"vote\0".as_ptr());
        trap_AddCommand(b"callteamvote\0".as_ptr());
        trap_AddCommand(b"teamvote\0".as_ptr());
        trap_AddCommand(b"stats\0".as_ptr());
        trap_AddCommand(b"teamtask\0".as_ptr());
        trap_AddCommand(b"loaddefered\0".as_ptr());  // spelled wrong, but not changing for demo
    }
}
