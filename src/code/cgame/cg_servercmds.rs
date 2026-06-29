// cg_servercmds.c -- text commands sent by the server

// this line must stay at top so the whole PCH thing works...
// (note: PCH concept not directly applicable to Rust, but preserving comment for reference)

use core::ffi::c_int;

// External declarations for functions/data from other modules
extern "C" {
    fn CG_ConfigString(index: c_int) -> *const core::ffi::c_char;
    fn atoi(nptr: *const core::ffi::c_char) -> c_int;
    fn Info_ValueForKey(s: *const core::ffi::c_char, key: *const core::ffi::c_char) -> *const core::ffi::c_char;
    fn Com_sprintf(dest: *mut core::ffi::c_char, size: usize, fmt: *const core::ffi::c_char, ...) -> c_int;
    fn strrchr(s: *const core::ffi::c_char, c: c_int) -> *mut core::ffi::c_char;
    fn strcpy(dest: *mut core::ffi::c_char, src: *const core::ffi::c_char) -> *mut core::ffi::c_char;
    fn strupr(s: *mut core::ffi::c_char) -> *mut core::ffi::c_char;
    fn stricmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char) -> c_int;
    fn strcmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char) -> c_int;
    fn cgi_GetGameState(gameState: *mut core::ffi::c_void) -> ();
    fn CG_NewClientinfo(clientNum: c_int) -> ();
    fn CG_RegisterItemSounds(itemNum: c_int) -> ();
    fn CG_RegisterItemVisuals(itemNum: c_int) -> ();
    fn CG_StartMusic(restart: c_int) -> ();
    fn CG_Printf(fmt: *const core::ffi::c_char, ...) -> ();
    fn CG_Argv(arg: c_int) -> *const core::ffi::c_char;
    fn cgi_R_RegisterModel(name: *const core::ffi::c_char) -> *mut core::ffi::c_void;
    fn cgi_R_RegisterSkin(name: *const core::ffi::c_char) -> *mut core::ffi::c_void;
    fn cgi_S_RegisterSound(name: *const core::ffi::c_char) -> c_int;
    fn CG_ScrollText(text: *const core::ffi::c_char, width: c_int) -> ();
    fn CG_CenterPrint(text: *const core::ffi::c_char, y: c_int) -> ();
    fn CG_CaptionText(text: *const core::ffi::c_char, handle: c_int, y: c_int) -> ();
    fn CG_CaptionTextStop() -> ();
    fn CG_SetLightstyle(index: c_int) -> ();
    fn cgi_R_WorldEffectCommand(command: *const core::ffi::c_char) -> ();
    fn cgi_GetServerCommand(sequenceNum: c_int) -> c_int;

    // Global data
    static mut cgs: CGameStaticData;
    static bg_numItems: c_int;
    static bg_itemlist: *const BgItemData;
    static mut cg: CGameData;
    static mut theFxScheduler: FxSchedulerData;
}

// Local stub types for layout/ABI matching - these would need actual definitions
// from their respective header files for correctness
#[repr(C)]
pub struct CGameStaticData {
    // Partial definition - would need complete definition from cg_local.h
    pub dmflags: c_int,
    pub teamflags: c_int,
    pub timelimit: c_int,
    pub maxclients: c_int,
    pub mapname: [core::ffi::c_char; 128],
    pub stripLevelName: [[core::ffi::c_char; 128]; 4],
    pub model_draw: [*mut core::ffi::c_void; 256],  // MAX_MODELS assumed 256
    pub skins: [*mut core::ffi::c_void; 256],  // MAX_CHARSKINS assumed 256
    pub sound_precache: [c_int; 256],  // MAX_SOUNDS assumed 256
    pub gameState: [core::ffi::c_char; 8192],  // rough estimate
    pub serverCommandSequence: c_int,
    // ... additional fields would follow
}

#[repr(C)]
pub struct BgItemData {
    pub classname: *const core::ffi::c_char,
    // ... additional fields would follow
}

#[repr(C)]
pub struct CGameData {
    pub levelShot: c_int,
    pub snap: *mut core::ffi::c_void,
    // ... additional fields would follow
}

#[repr(C)]
pub struct FxSchedulerData {
    // Partial definition for theFxScheduler
}

// Constants
const STRIPED_LEVELNAME_VARIATIONS: usize = 4;
const SCREEN_HEIGHT: c_int = 480;  // typical value
const SCREEN_WIDTH: c_int = 640;   // typical value
const MAX_MODELS: c_int = 256;
const MAX_CHARSKINS: c_int = 256;
const MAX_SOUNDS: c_int = 256;
const MAX_CLIENTS: c_int = 64;
const MAX_FX: c_int = 4000;
const MAX_LIGHT_STYLES: c_int = 256;
const MAX_WORLD_FX: c_int = 256;
const MAX_QPATH: usize = 256;
const CHAN_AUTO: c_int = 3;
const CHAN_LOCAL_SOUND: c_int = 2;

// Config string indices
const CS_SERVERINFO: c_int = 0;
const CS_ITEMS: c_int = 2;
const CS_MODELS: c_int = 20;
const CS_SOUNDS: c_int = 320;
const CS_CHARSKINS: c_int = 576;
const CS_EFFECTS: c_int = 832;
const CS_PLAYERS: c_int = 840;
const CS_MUSIC: c_int = 906;
const CS_LIGHT_STYLES: c_int = 910;
const CS_WORLD_FX: c_int = 966;

const qtrue: c_int = 1;
const qfalse: c_int = 0;

// Forward declaration - defined elsewhere
extern "C" {
    fn CG_RegisterClientModels(entityNum: c_int) -> ();
}

/*
================
CG_ParseServerinfo

This is called explicitly when the gamestate is first received,
and whenever the server updates any serverinfo flagged cvars
================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CG_ParseServerinfo() {
    let info: *const core::ffi::c_char;
    let mapname: *const core::ffi::c_char;

    info = CG_ConfigString(CS_SERVERINFO);
    (*core::ptr::addr_of_mut!(cgs)).dmflags = atoi(Info_ValueForKey(info, b"dmflags\0".as_ptr() as *const core::ffi::c_char));
    (*core::ptr::addr_of_mut!(cgs)).teamflags = atoi(Info_ValueForKey(info, b"teamflags\0".as_ptr() as *const core::ffi::c_char));
    (*core::ptr::addr_of_mut!(cgs)).timelimit = atoi(Info_ValueForKey(info, b"timelimit\0".as_ptr() as *const core::ffi::c_char));
    (*core::ptr::addr_of_mut!(cgs)).maxclients = 1;
    mapname = Info_ValueForKey(info, b"mapname\0".as_ptr() as *const core::ffi::c_char);
    Com_sprintf(
        (*core::ptr::addr_of_mut!(cgs)).mapname.as_mut_ptr(),
        core::mem::size_of_val(&(*core::ptr::addr_of!(cgs)).mapname),
        b"maps/%s.bsp\0".as_ptr() as *const core::ffi::c_char,
        mapname,
    );
    let p: *mut core::ffi::c_char = strrchr(mapname, '/' as c_int);
    strcpy(
        (*core::ptr::addr_of_mut!(cgs)).stripLevelName[0].as_mut_ptr(),
        if !p.is_null() { p.add(1) } else { mapname },
    );
    strupr((*core::ptr::addr_of_mut!(cgs)).stripLevelName[0].as_mut_ptr());
    let mut i: c_int = 1;
    while (i as usize) < STRIPED_LEVELNAME_VARIATIONS {
        // clear retry-array
        *(*core::ptr::addr_of_mut!(cgs)).stripLevelName[i as usize].as_mut_ptr() = '\0' as core::ffi::c_char;
        i += 1;
    }
    // be careful with the []-numbers here. Currently I use 0,1,2 for replacements or substitution, and [3] for "INGAME"
    //	I know, if I'd known there was going to be this much messing about I'd have subroutinised it all and done it
    //	neater, but it kinda evolved...   Feel free to bug me if you want to add to it... ?  -Ste.
    //

    //FIXME: a better way to handle sound-matched strings from other levels (currently uses levelname+sound as key)

    // additional String files needed for some levels...
    //
    // JKA...
    if stricmp(
        (*core::ptr::addr_of!(cgs)).stripLevelName[0].as_ptr(),
        b"YAVIN1B\0".as_ptr() as *const core::ffi::c_char,
    ) == 0
    {
        strcpy(
            (*core::ptr::addr_of_mut!(cgs)).stripLevelName[1].as_mut_ptr(),
            b"YAVIN1\0".as_ptr() as *const core::ffi::c_char,
        );
    }

    /*	// JK2...
    if (!stricmp(cgs.stripLevelName[0],"KEJIM_BASE") ||
        !stricmp(cgs.stripLevelName[0],"KEJIM_POST")
        )
    {
        strcpy( cgs.stripLevelName[1], "ARTUS_MINE" );
    }
    if (!stricmp(cgs.stripLevelName[0],"DOOM_DETENTION") ||
        !stricmp(cgs.stripLevelName[0],"DOOM_SHIELDS")
        )
    {
        strcpy( cgs.stripLevelName[1], "DOOM_COMM" );
    }
    if (!stricmp(cgs.stripLevelName[0],"DOOM_COMM"))
    {
        strcpy( cgs.stripLevelName[1], "CAIRN_BAY" );
    }
    if (!stricmp(cgs.stripLevelName[0],"NS_STARPAD"))
    {
        strcpy( cgs.stripLevelName[1], "ARTUS_TOPSIDE" );	// for dream sequence...

        strcpy( cgs.stripLevelName[2], "BESPIN_UNDERCITY" );	// for dream sequence...
    }
    if (!stricmp(cgs.stripLevelName[0],"BESPIN_PLATFORM"))
    {
        strcpy( cgs.stripLevelName[1], "BESPIN_UNDERCITY" );
    }
    */
}


/*
================
CG_ConfigStringModified

================
*/
#[allow(non_snake_case)]
unsafe extern "C" fn CG_ConfigStringModified() {
    let str: *const core::ffi::c_char;
    let mut num: c_int;

    num = atoi(CG_Argv(1));

    // get the gamestate from the client system, which will have the
    // new configstring already integrated
    cgi_GetGameState(core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(cgs)).gameState) as *mut core::ffi::c_void);

    // look up the individual string that was modified
    str = CG_ConfigString(num);

    // do something with it if necessary
    if num == CS_ITEMS {
        let mut i: c_int = 1;
        while i < bg_numItems {
            if *str.add(i as usize) as core::ffi::c_char == '1' as core::ffi::c_char {
                if !(*bg_itemlist.add(i as usize)).classname.is_null() {
                    CG_RegisterItemSounds(i);
                    CG_RegisterItemVisuals(i);
                }
            }
            i += 1;
        }
    } else if num == CS_MUSIC {
        CG_StartMusic(qtrue);
    } else if num == CS_SERVERINFO {
        CG_ParseServerinfo();
    } else if num >= CS_MODELS && num < CS_MODELS + MAX_MODELS {
        (*core::ptr::addr_of_mut!(cgs)).model_draw[(num - CS_MODELS) as usize] = cgi_R_RegisterModel(str);
        //		OutputDebugString(va("### CG_ConfigStringModified(): cgs.model_draw[%d] = \"%s\"\n",num-CS_MODELS,str));
        // GHOUL2 Insert start
    } else if num >= CS_CHARSKINS && num < CS_CHARSKINS + MAX_CHARSKINS {
        (*core::ptr::addr_of_mut!(cgs)).skins[(num - CS_CHARSKINS) as usize] = cgi_R_RegisterSkin(str);
        // Ghoul2 Insert end
    } else if num >= CS_SOUNDS && num < CS_SOUNDS + MAX_SOUNDS {
        if *str as core::ffi::c_char != '*' as core::ffi::c_char {
            (*core::ptr::addr_of_mut!(cgs)).sound_precache[(num - CS_SOUNDS) as usize] = cgi_S_RegisterSound(str);
        }
    } else if num >= CS_EFFECTS && num < CS_EFFECTS + MAX_FX {
        (*core::ptr::addr_of_mut!(theFxScheduler)).RegisterEffect(str);
    } else if num >= CS_PLAYERS && num < CS_PLAYERS + MAX_CLIENTS {
        CG_NewClientinfo(num - CS_PLAYERS);
        CG_RegisterClientModels(num - CS_PLAYERS);
    } else if num >= CS_LIGHT_STYLES && num < CS_LIGHT_STYLES + (MAX_LIGHT_STYLES * 3) {
        CG_SetLightstyle(num - CS_LIGHT_STYLES);
    } else if num >= CS_WORLD_FX && num < CS_WORLD_FX + MAX_WORLD_FX {
        cgi_R_WorldEffectCommand(str);
    }
}

/*
=================
CG_ServerCommand

The string has been tokenized and can be retrieved with
Cmd_Argc() / Cmd_Argv()
=================
*/
#[allow(non_snake_case)]
unsafe extern "C" fn CG_ServerCommand() {
    let cmd: *const core::ffi::c_char;

    cmd = CG_Argv(0);

    if strcmp(cmd, b"cp\0".as_ptr() as *const core::ffi::c_char) == 0 {
        CG_CenterPrint(CG_Argv(1), SCREEN_HEIGHT * 0.25 as c_int);
        return;
    }

    if strcmp(cmd, b"cs\0".as_ptr() as *const core::ffi::c_char) == 0 {
        CG_ConfigStringModified();
        return;
    }

    if strcmp(cmd, b"print\0".as_ptr() as *const core::ffi::c_char) == 0 {
        CG_Printf(b"%s\0".as_ptr() as *const core::ffi::c_char, CG_Argv(1));
        return;
    }

    if strcmp(cmd, b"chat\0".as_ptr() as *const core::ffi::c_char) == 0 {
        //		cgi_S_StartLocalSound ( cgs.media.talkSound, CHAN_LOCAL_SOUND );
        CG_Printf(b"%s\n\0".as_ptr() as *const core::ffi::c_char, CG_Argv(1));
        return;
    }


    // Scroll text
    if strcmp(cmd, b"st\0".as_ptr() as *const core::ffi::c_char) == 0 {
        CG_ScrollText(CG_Argv(1), SCREEN_WIDTH - 16);
        return;
    }

    // Cinematic text
    if strcmp(cmd, b"ct\0".as_ptr() as *const core::ffi::c_char) == 0 {
        CG_CaptionText(
            CG_Argv(1),
            (*core::ptr::addr_of!(cgs)).sound_precache[atoi(CG_Argv(2)) as usize],
            SCREEN_HEIGHT * 0.25 as c_int,
        );
        return;
    }

    // Text stop
    if strcmp(cmd, b"cts\0".as_ptr() as *const core::ffi::c_char) == 0 {
        CG_CaptionTextStop();
        return;
    }


    // Text to appear in center of screen with an LCARS frame around it.
    if strcmp(cmd, b"lt\0".as_ptr() as *const core::ffi::c_char) == 0 {
        CG_Printf(
            b"CG_LCARSText() being called. Tell Ste\nString: \"%s\"\n\0".as_ptr() as *const core::ffi::c_char,
            CG_Argv(1),
        );
        return;
    }

    // clientLevelShot is sent before taking a special screenshot for
    // the menu system during development
    if strcmp(cmd, b"clientLevelShot\0".as_ptr() as *const core::ffi::c_char) == 0 {
        (*core::ptr::addr_of_mut!(cg)).levelShot = qtrue;
        return;
    }

    if strcmp(cmd, b"vmsg\0".as_ptr() as *const core::ffi::c_char) == 0 {
        #[cfg(feature = "vm_debug")]
        {
            let mut snd: [core::ffi::c_char; MAX_QPATH] = [0; MAX_QPATH];

            Com_sprintf(
                snd.as_mut_ptr(),
                core::mem::size_of_val(&snd),
                b"sound/teamplay/vmsg/%s.wav\0".as_ptr() as *const core::ffi::c_char,
                CG_Argv(1),
            );
            // cgi_S_StartSound (NULL, cg.snap->ps.clientNum, CHAN_AUTO,
            //	cgi_S_RegisterSound (snd) );
        }
        return;
    }

    CG_Printf(
        b"Unknown client game command: %s\n\0".as_ptr() as *const core::ffi::c_char,
        cmd,
    );
}


/*
====================
CG_ExecuteNewServerCommands

Execute all of the server commands that were received along
with this this snapshot.
====================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CG_ExecuteNewServerCommands(latestSequence: c_int) {
    while (*core::ptr::addr_of!(cgs)).serverCommandSequence < latestSequence {
        (*core::ptr::addr_of_mut!(cgs)).serverCommandSequence += 1;
        if cgi_GetServerCommand((*core::ptr::addr_of!(cgs)).serverCommandSequence) != 0 {
            CG_ServerCommand();
        }
    }
}
