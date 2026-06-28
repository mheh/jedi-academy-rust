//! Mechanical port of `codemp/ui/ui_gameinfo.c`.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::addr_of_mut;

// ================================================================
// Stubs for external dependencies not yet ported.
// ================================================================

// From ui_shared.h / ui_local.h
pub type fileHandle_t = c_int;
pub type qhandle_t = c_int;

#[repr(C)]
pub struct mapInfo {
    pub cinematic: c_int,
    pub mapLoadName: *const c_char,
    pub mapName: *const c_char,
    pub levelShot: c_int,
    pub imageName: *const c_char,
    pub typeBits: c_int,
}

#[repr(C)]
pub struct uiInfo_s {
    pub mapCount: c_int,
    pub mapList: [mapInfo; 128], // MAX_MAPS = 128
    // Remaining fields from the full uiInfo_t (partial stub)
}

extern "C" {
    // From ui_local.h
    pub static mut uiInfo: uiInfo_s;

    // From q_shared / common
    pub fn COM_Parse(data: *const *const c_char) -> *const c_char;
    pub fn COM_ParseExt(data: *const *const c_char, allowLineBreak: c_int) -> *const c_char;
    pub fn COM_Compress(data: *mut c_char);

    // From common/printng
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn trap_Print(string: *const c_char);

    // From q_shared / string functions
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: c_int);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Standard C string functions
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;

    // UI allocation
    pub fn UI_Alloc(size: usize) -> *mut c_void;
    pub fn UI_OutOfMemory() -> c_int;
    pub fn String_Alloc(str: *const c_char) -> *const c_char;
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);

    // Filesystem
    pub fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut fileHandle_t, mode: c_int) -> c_int;
    pub fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t);
    pub fn trap_FS_FCloseFile(f: fileHandle_t);
    pub fn trap_FS_GetFileList(
        path: *const c_char,
        extension: *const c_char,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int;

    // Cvar
    pub fn trap_Cvar_Register(vmCvar: *mut vmCvar_t, varName: *const c_char, defaultValue: *const c_char, flags: c_int);
    pub fn trap_Cvar_VariableValue(var_name: *const c_char) -> f32;

    // Utility
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
}

// From ui_public.h / common
#[repr(C)]
pub struct vmCvar_t {
    pub handle: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub string: [c_char; 256],
}

// Constants from q_shared.h / ui_local.h
pub const MAX_BOTS: c_int = 64;
pub const MAX_ARENAS: c_int = 128;
pub const MAX_BOTS_TEXT: c_int = 32000;
pub const MAX_ARENAS_TEXT: c_int = 32000;
pub const MAX_TOKEN_CHARS: c_int = 1024;
pub const MAX_INFO_STRING: c_int = 1024;
pub const MAX_MAPS: c_int = 128;

pub const FS_READ: c_int = 0;
pub const CVAR_INIT: c_int = 4;
pub const CVAR_ROM: c_int = 16;

pub const S_COLOR_RED: &[u8] = b"^1\0";
pub const S_COLOR_YELLOW: &[u8] = b"^3\0";

// Game type enums
pub const GT_FFA: c_int = 0;
pub const GT_TOURNAMENT: c_int = 1;
pub const GT_SINGLE_PLAYER: c_int = 2;
pub const GT_TEAM: c_int = 3;
pub const GT_CTF: c_int = 4;
pub const GT_1FCTF: c_int = 5;
pub const GT_ONEFLAG: c_int = 6;
pub const GT_OBELISK: c_int = 7;
pub const GT_HARVESTER: c_int = 8;
pub const GT_HOLOCRON: c_int = 9;
pub const GT_JEDIMASTER: c_int = 10;
pub const GT_DUEL: c_int = 11;
pub const GT_POWERDUEL: c_int = 12;
pub const GT_SIEGE: c_int = 13;
pub const GT_CTY: c_int = 14;

// ================================================================
// Global variables (static, matching C semantics).
// ================================================================

//
// arena and bot info
//

pub static mut ui_numBots: c_int = 0;
static mut ui_botInfos: [*const c_char; MAX_BOTS as usize] = [core::ptr::null(); MAX_BOTS as usize];

static mut ui_numArenas: c_int = 0;
static mut ui_arenaInfos: [*const c_char; MAX_ARENAS as usize] = [core::ptr::null(); MAX_ARENAS as usize];

// ================================================================
// Functions
// ================================================================

/*
===============
UI_ParseInfos
===============
*/
pub unsafe extern "C" fn UI_ParseInfos(
    mut buf: *mut c_char,
    max: c_int,
    infos: *mut *mut c_char,
) -> c_int {
    let mut token: *const c_char;
    let mut count: c_int = 0;
    let mut key: [c_char; MAX_TOKEN_CHARS as usize] = [0; MAX_TOKEN_CHARS as usize];
    let mut info: [c_char; MAX_INFO_STRING as usize] = [0; MAX_INFO_STRING as usize];

    count = 0;

    loop {
        token = COM_Parse((&mut buf as *mut *mut c_char) as *const *const c_char);
        if (*token.offset(0)) == 0 {
            break;
        }
        if strcmp(token, "{" as *const u8 as *const c_char) != 0 {
            Com_Printf("Missing { in info file\n" as *const u8 as *const c_char);
            break;
        }

        if count == max {
            Com_Printf("Max infos exceeded\n" as *const u8 as *const c_char);
            break;
        }

        info[0] = 0;
        loop {
            token = COM_ParseExt((&mut buf as *mut *mut c_char) as *const *const c_char, 1);
            if (*token.offset(0)) == 0 {
                Com_Printf("Unexpected end of info file\n" as *const u8 as *const c_char);
                break;
            }
            if strcmp(token, "}" as *const u8 as *const c_char) == 0 {
                break;
            }
            Q_strncpyz(key.as_mut_ptr(), token, MAX_TOKEN_CHARS);

            token = COM_ParseExt((&mut buf as *mut *mut c_char) as *const *const c_char, 0);
            if (*token.offset(0)) == 0 {
                strcpy(token as *mut c_char, "<NULL>" as *const u8 as *const c_char);
            }
            Info_SetValueForKey(info.as_mut_ptr(), key.as_ptr(), token);
        }
        // NOTE: extra space for arena number
        let alloc_size = strlen(info.as_ptr())
            + strlen("\\" as *const u8 as *const c_char)
            + strlen("num" as *const u8 as *const c_char)
            + strlen("\\" as *const u8 as *const c_char)
            + strlen(va("%d" as *const u8 as *const c_char, MAX_ARENAS as c_int))
            + 1;
        *infos.offset(count as isize) = UI_Alloc(alloc_size) as *mut c_char;
        if !(*infos.offset(count as isize)).is_null() {
            strcpy(
                *infos.offset(count as isize),
                info.as_ptr() as *const c_char,
            );
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                if trap_Cvar_VariableValue("com_buildScript" as *const u8 as *const c_char) != 0.0 {
                    let botFile: *const c_char =
                        Info_ValueForKey(info.as_ptr() as *const c_char, "personality" as *const u8 as *const c_char);
                    if !botFile.is_null() && (*botFile.offset(0)) != 0 {
                        let mut fh: fileHandle_t = 0;
                        trap_FS_FOpenFile(botFile, &mut fh, FS_READ);
                        if fh != 0 {
                            trap_FS_FCloseFile(fh);
                        }
                    }
                }
            }
            count += 1;
        }
    }
    count
}

/*
===============
UI_LoadArenasFromFile
===============
*/
unsafe fn UI_LoadArenasFromFile(filename: *mut c_char) {
    let mut len: c_int;
    let mut f: fileHandle_t = 0;
    let mut buf: [c_char; MAX_ARENAS_TEXT as usize] = [0; MAX_ARENAS_TEXT as usize];

    len = trap_FS_FOpenFile(filename, &mut f, FS_READ);
    if f == 0 {
        trap_Print(va(
            "{}\0" as *const u8 as *const c_char,
            "file not found: %s\n" as *const u8 as *const c_char,
            filename,
        ));
        return;
    }
    if len >= MAX_ARENAS_TEXT {
        trap_Print(va(
            "{}\0" as *const u8 as *const c_char,
            "file too large: %s is %i, max allowed is %i" as *const u8 as *const c_char,
            filename,
            len,
            MAX_ARENAS_TEXT,
        ));
        trap_FS_FCloseFile(f);
        return;
    }

    trap_FS_Read(buf.as_mut_ptr() as *mut c_void, len, f);
    buf[len as usize] = 0;
    trap_FS_FCloseFile(f);

    ui_numArenas += UI_ParseInfos(
        buf.as_mut_ptr(),
        MAX_ARENAS - ui_numArenas,
        &mut ui_arenaInfos[(ui_numArenas) as usize] as *mut *const c_char as *mut *mut c_char,
    );
}

/*
===============
UI_LoadArenas
===============
*/
pub unsafe extern "C" fn UI_LoadArenas() {
    let mut numdirs: c_int;
    let mut filename: [c_char; 128] = [0; 128];
    let mut dirlist: [c_char; 1024] = [0; 1024];
    let mut dirptr: *mut c_char;
    let mut i: c_int;
    let mut n: c_int;
    let mut dirlen: c_int;
    let mut type_: *const c_char;

    ui_numArenas = 0;
    uiInfo.mapCount = 0;

    // get all arenas from .arena files
    numdirs = trap_FS_GetFileList(
        "scripts" as *const u8 as *const c_char,
        ".arena" as *const u8 as *const c_char,
        dirlist.as_mut_ptr(),
        1024,
    );
    dirptr = dirlist.as_mut_ptr();
    i = 0;
    while i < numdirs {
        dirlen = strlen(dirptr) as c_int;
        strcpy(filename.as_mut_ptr(), "scripts/" as *const u8 as *const c_char);
        strcat(filename.as_mut_ptr(), dirptr);
        UI_LoadArenasFromFile(filename.as_mut_ptr());
        dirptr = dirptr.offset((dirlen + 1) as isize);
        i += 1;
    }
    // trap_Print( va( "%i arenas parsed\n", ui_numArenas ) );
    if UI_OutOfMemory() != 0 {
        trap_Print("WARNING: not anough memory in pool to load all arenas\n" as *const u8 as *const c_char);
    }

    n = 0;
    while n < ui_numArenas {
        // determine type

        (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).cinematic = -1;
        (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).mapLoadName =
            String_Alloc(Info_ValueForKey(
                ui_arenaInfos[n as usize],
                "map" as *const u8 as *const c_char,
            ));
        (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).mapName =
            String_Alloc(Info_ValueForKey(
                ui_arenaInfos[n as usize],
                "longname" as *const u8 as *const c_char,
            ));
        (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).levelShot = -1;
        (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).imageName =
            String_Alloc(va(
                "levelshots/%s" as *const u8 as *const c_char,
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).mapLoadName,
            ));
        (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits = 0;

        type_ = Info_ValueForKey(
            ui_arenaInfos[n as usize],
            "type" as *const u8 as *const c_char,
        );
        // if no type specified, it will be treated as "ffa"
        if *type_ != 0 {
            if !strstr(type_, "ffa" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_FFA;
            }
            if !strstr(type_, "team" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_TEAM;
            }
            if !strstr(type_, "holocron" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_HOLOCRON;
            }
            if !strstr(type_, "jedimaster" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_JEDIMASTER;
            }
            if !strstr(type_, "duel" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_DUEL;
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_POWERDUEL;
            }
            if !strstr(type_, "powerduel" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_DUEL;
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_POWERDUEL;
            }
            if !strstr(type_, "siege" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_SIEGE;
            }
            if !strstr(type_, "ctf" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_CTF;
            }
            if !strstr(type_, "cty" as *const u8 as *const c_char).is_null() {
                (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                    1 << GT_CTY;
            }
        } else {
            (*addr_of_mut!(uiInfo).mapList.as_mut_ptr().offset(uiInfo.mapCount as isize)).typeBits |=
                1 << GT_FFA;
        }

        uiInfo.mapCount += 1;
        if uiInfo.mapCount >= MAX_MAPS {
            break;
        }
        n += 1;
    }
}

/*
===============
UI_LoadBotsFromFile
===============
*/
unsafe fn UI_LoadBotsFromFile(filename: *mut c_char) {
    let mut len: c_int;
    let mut f: fileHandle_t = 0;
    let mut buf: [c_char; MAX_BOTS_TEXT as usize] = [0; MAX_BOTS_TEXT as usize];
    let mut stopMark: *const c_char;

    len = trap_FS_FOpenFile(filename, &mut f, FS_READ);
    if f == 0 {
        trap_Print(va(
            "{}\0" as *const u8 as *const c_char,
            "file not found: %s\n" as *const u8 as *const c_char,
            filename,
        ));
        return;
    }
    if len >= MAX_BOTS_TEXT {
        trap_Print(va(
            "{}\0" as *const u8 as *const c_char,
            "file too large: %s is %i, max allowed is %i" as *const u8 as *const c_char,
            filename,
            len,
            MAX_BOTS_TEXT,
        ));
        trap_FS_FCloseFile(f);
        return;
    }

    trap_FS_Read(buf.as_mut_ptr() as *mut c_void, len, f);
    buf[len as usize] = 0;

    stopMark = strstr(buf.as_ptr(), "@STOPHERE" as *const u8 as *const c_char);

    // This bot is in place as a mark for modview's bot viewer.
    // If we hit it just stop and trace back to the beginning of the bot define and cut the string off.
    // This is only done in the UI and not the game so that "test" bots can be added manually and still
    // not show up in the menu.
    if !stopMark.is_null() {
        let mut startPoint: isize = stopMark as isize - buf.as_ptr() as isize;

        while buf[startPoint as usize] as u8 != b'{' {
            startPoint -= 1;
        }

        buf[startPoint as usize] = 0;
    }

    trap_FS_FCloseFile(f);

    COM_Compress(buf.as_mut_ptr());

    ui_numBots += UI_ParseInfos(
        buf.as_mut_ptr(),
        MAX_BOTS - ui_numBots,
        &mut ui_botInfos[(ui_numBots) as usize] as *mut *const c_char as *mut *mut c_char,
    );
}

/*
===============
UI_LoadBots
===============
*/
pub unsafe extern "C" fn UI_LoadBots() {
    let mut botsFile: vmCvar_t = core::mem::zeroed();
    let mut numdirs: c_int;
    let mut filename: [c_char; 128] = [0; 128];
    let mut dirlist: [c_char; 1024] = [0; 1024];
    let mut dirptr: *mut c_char;
    let mut i: c_int;
    let mut dirlen: c_int;

    ui_numBots = 0;

    trap_Cvar_Register(
        &mut botsFile,
        "g_botsFile" as *const u8 as *const c_char,
        "" as *const u8 as *const c_char,
        CVAR_INIT | CVAR_ROM,
    );
    if botsFile.string[0] != 0 {
        UI_LoadBotsFromFile(botsFile.string.as_mut_ptr());
    } else {
        UI_LoadBotsFromFile("botfiles/bots.txt" as *const u8 as *const c_char as *mut c_char);
    }

    // get all bots from .bot files
    numdirs = trap_FS_GetFileList(
        "scripts" as *const u8 as *const c_char,
        ".bot" as *const u8 as *const c_char,
        dirlist.as_mut_ptr(),
        1024,
    );
    dirptr = dirlist.as_mut_ptr();
    i = 0;
    while i < numdirs {
        dirlen = strlen(dirptr) as c_int;
        strcpy(filename.as_mut_ptr(), "scripts/" as *const u8 as *const c_char);
        strcat(filename.as_mut_ptr(), dirptr);
        UI_LoadBotsFromFile(filename.as_mut_ptr());
        dirptr = dirptr.offset((dirlen + 1) as isize);
        i += 1;
    }
    // trap_Print( va( "%i bots parsed\n", ui_numBots ) );
}

/*
===============
UI_GetBotInfoByNumber
===============
*/
pub unsafe extern "C" fn UI_GetBotInfoByNumber(num: c_int) -> *const c_char {
    if num < 0 || num >= ui_numBots {
        trap_Print(va(
            "{}\0" as *const u8 as *const c_char,
            "Invalid bot number: %i\n" as *const u8 as *const c_char,
            num,
        ));
        return core::ptr::null();
    }
    ui_botInfos[num as usize]
}

/*
===============
UI_GetBotInfoByName
===============
*/
pub unsafe extern "C" fn UI_GetBotInfoByName(name: *const c_char) -> *const c_char {
    let mut n: c_int;
    let mut value: *const c_char;

    n = 0;
    while n < ui_numBots {
        value = Info_ValueForKey(ui_botInfos[n as usize], "name" as *const u8 as *const c_char);
        if Q_stricmp(value, name) == 0 {
            return ui_botInfos[n as usize];
        }
        n += 1;
    }

    core::ptr::null()
}

pub unsafe extern "C" fn UI_GetNumBots() -> c_int {
    ui_numBots
}

pub unsafe extern "C" fn UI_GetBotNameByNumber(num: c_int) -> *const c_char {
    let info: *const c_char = UI_GetBotInfoByNumber(num);
    if !info.is_null() {
        return Info_ValueForKey(info, "name" as *const u8 as *const c_char);
    }
    "Kyle" as *const u8 as *const c_char
}
