/*****************************************************************************
 * name:		be_ai_goal.c
 *
 * desc:		goal AI
 *
 * $Archive: /MissionPack/code/botlib/be_ai_goal.c $
 * $Author: Ttimo $
 * $Revision: 14 $
 * $Modtime: 4/13/01 4:45p $
 * $Date: 4/13/01 4:45p $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_char, c_void};

// Stub types for external dependencies - these would be defined in actual modules
// These are forward declarations to allow structural coherence of this file

#[repr(C)]
pub struct weightconfig_s {
    _dummy: c_int,
}

#[repr(C)]
pub struct token_s {
    _dummy: c_int,
}

pub type token_t = token_s;

#[repr(C)]
pub struct source_s {
    _dummy: c_int,
}

pub type source_t = source_s;

#[repr(C)]
pub struct bot_goal_s {
    pub areanum: c_int,
    pub origin: [f32; 3],
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub entitynum: c_int,
    pub number: c_int,
    pub flags: c_int,
    pub iteminfo: c_int,
}

pub type bot_goal_t = bot_goal_s;

#[repr(C)]
pub struct aas_entityinfo_s {
    _dummy: c_int,
}

pub type aas_entityinfo_t = aas_entityinfo_s;

#[repr(C)]
pub struct bsp_trace_s {
    pub fraction: f32,
    _dummy: [c_int; 20],
}

pub type bsp_trace_t = bsp_trace_s;

#[repr(C)]
pub struct fielddef_s {
    _dummy: c_int,
}

pub type fielddef_t = fielddef_s;

#[repr(C)]
pub struct structdef_s {
    _dummy: c_int,
}

pub type structdef_t = structdef_s;

#[repr(C)]
pub struct libvar_s {
    pub value: f32,
}

pub type libvar_t = libvar_s;

// Constants from be_interface.h (stub)
pub const MAX_CLIENTS: usize = 64;
pub const MAX_EPAIRKEY: usize = 256;
pub const MAX_STRINGFIELD: usize = 256;
pub const MAX_GOALSTACK: usize = 32;
pub const MAX_AVOIDGOALS: usize = 32;
pub const MAX_PATH: usize = 256;
pub const BOTFILESBASEFOLDER: &[u8; 1] = b"";
pub const PRESENCE_NORMAL: c_int = 0;
pub const CONTENTS_WATER: c_int = 16;
pub const CONTENTS_SOLID: c_int = 1;
pub const CONTENTS_PLAYERCLIP: c_int = 65536;

// Error codes (stub)
pub const BLERR_NOERROR: c_int = 0;
pub const BLERR_CANNOTLOADITEMWEIGHTS: c_int = 1;
pub const BLERR_CANNOTLOADITEMCONFIG: c_int = 2;

// Print types (stub)
pub const PRT_MESSAGE: c_int = 0;
pub const PRT_WARNING: c_int = 1;
pub const PRT_ERROR: c_int = 2;
pub const PRT_FATAL: c_int = 3;

// Goal flags (stub)
pub const GFL_ITEM: c_int = 1;
pub const GFL_DROPPED: c_int = 2;
pub const GFL_ROAM: c_int = 4;

// Gametype (stub)
pub const GT_FFA: c_int = 0;
pub const GT_HOLOCRON: c_int = 1;
pub const GT_JEDIMASTER: c_int = 2;
pub const GT_DUEL: c_int = 3;
pub const GT_POWERDUEL: c_int = 4;
pub const GT_SINGLE_PLAYER: c_int = 5;
pub const GT_TEAM: c_int = 6;
pub const GT_SIEGE: c_int = 7;
pub const GT_CTF: c_int = 8;
pub const GT_CTY: c_int = 9;
pub const GT_MAX_GAME_TYPE: c_int = 10;

// Conditional compilation
// #define DEBUG_AI_GOAL
// #ifdef RANDOMIZE
// #define UNDECIDEDFUZZY
// #endif //RANDOMIZE
// #define DROPPEDWEIGHT

//minimum avoid goal time
pub const AVOID_MINIMUM_TIME: c_int = 10;
//default avoid goal time
pub const AVOID_DEFAULT_TIME: c_int = 30;
//avoid dropped goal time
pub const AVOID_DROPPED_TIME: c_int = 10;
//
pub const TRAVELTIME_SCALE: f32 = 0.01;
//item flags
pub const IFL_NOTFREE: c_int = 1;
//not in free for all
pub const IFL_NOTTEAM: c_int = 2;
//not in team play
pub const IFL_NOTSINGLE: c_int = 4;
//not in single player
pub const IFL_NOTBOT: c_int = 8;
//bot should never go for this
pub const IFL_ROAM: c_int = 16;
//bot roam goal

//location in the map "target_location"
#[repr(C)]
pub struct maplocation_s {
    pub origin: [f32; 3],
    pub areanum: c_int,
    pub name: [c_char; MAX_EPAIRKEY],
    pub next: *mut maplocation_s,
}

pub type maplocation_t = maplocation_s;

//camp spots "info_camp"
#[repr(C)]
pub struct campspot_s {
    pub origin: [f32; 3],
    pub areanum: c_int,
    pub name: [c_char; MAX_EPAIRKEY],
    pub range: f32,
    pub weight: f32,
    pub wait: f32,
    pub random: f32,
    pub next: *mut campspot_s,
}

pub type campspot_t = campspot_s;

//FIXME: these are game specific
#[repr(C)]
pub struct levelitem_s {
    pub number: c_int,
    //number of the level item
    pub iteminfo: c_int,
    //index into the item info
    pub flags: c_int,
    //item flags
    pub weight: f32,
    //fixed roam weight
    pub origin: [f32; 3],
    //origin of the item
    pub goalareanum: c_int,
    //area the item is in
    pub goalorigin: [f32; 3],
    //goal origin within the area
    pub entitynum: c_int,
    //entity number
    pub timeout: f32,
    //item is removed after this time
    pub prev: *mut levelitem_s,
    pub next: *mut levelitem_s,
}

pub type levelitem_t = levelitem_s;

#[repr(C)]
pub struct iteminfo_s {
    pub classname: [c_char; 32],
    //classname of the item
    pub name: [c_char; MAX_STRINGFIELD],
    //name of the item
    pub model: [c_char; MAX_STRINGFIELD],
    //model of the item
    pub modelindex: c_int,
    //model index
    pub type_: c_int,
    //item type (renamed from 'type' to avoid Rust keyword)
    pub index: c_int,
    //index in the inventory
    pub respawntime: f32,
    //respawn time
    pub mins: [f32; 3],
    //mins of the item
    pub maxs: [f32; 3],
    //maxs of the item
    pub number: c_int,
    //number of the item info
}

pub type iteminfo_t = iteminfo_s;

// macro-like function for offset calculation
pub fn ITEMINFO_OFS(field_offset: usize) -> c_int {
    field_offset as c_int
}

// iteminfo_fields array - preserved in order
pub static ITEMINFO_FIELDS: &[()] = &[];

#[repr(C)]
pub struct itemconfig_s {
    pub numiteminfo: c_int,
    pub iteminfo: *mut iteminfo_t,
}

pub type itemconfig_t = itemconfig_s;

//goal state
#[repr(C)]
pub struct bot_goalstate_s {
    pub itemweightconfig: *mut weightconfig_s,
    //weight config
    pub itemweightindex: *mut c_int,
    //index from item to weight
    //
    pub client: c_int,
    //client using this goal state
    pub lastreachabilityarea: c_int,
    //last area with reachabilities the bot was in
    //
    pub goalstack: [bot_goal_t; MAX_GOALSTACK],
    //goal stack
    pub goalstacktop: c_int,
    //the top of the goal stack
    //
    pub avoidgoals: [c_int; MAX_AVOIDGOALS],
    //goals to avoid
    pub avoidgoaltimes: [f32; MAX_AVOIDGOALS],
    //times to avoid the goals
}

pub type bot_goalstate_t = bot_goalstate_s;

pub static mut BOTGOALSTATES: [*mut bot_goalstate_t; MAX_CLIENTS + 1] = [core::ptr::null_mut(); MAX_CLIENTS + 1];
// bk001206 - FIXME: init?
//item configuration
pub static mut ITEMCONFIG: *mut itemconfig_t = core::ptr::null_mut();
// bk001206 - init
//level items
pub static mut LEVELITEMHEAP: *mut levelitem_t = core::ptr::null_mut();
// bk001206 - init
pub static mut FREELEVELITEMS: *mut levelitem_t = core::ptr::null_mut();
// bk001206 - init
pub static mut LEVELITEMS: *mut levelitem_t = core::ptr::null_mut();
// bk001206 - init
pub static mut NUMLEVELITEMS: c_int = 0;
//map locations
pub static mut MAPLOCATIONS: *mut maplocation_t = core::ptr::null_mut();
// bk001206 - init
//camp spots
pub static mut CAMPSPOTS: *mut campspot_t = core::ptr::null_mut();
// bk001206 - init
//the game type
pub static mut G_GAMETYPE: c_int = 0;
// bk001206 - init
//additional dropped item weight
pub static mut DROPPEDWEIGHT: *mut libvar_t = core::ptr::null_mut();
// bk001206 - init

// External C function stubs - these need to be declared as extern "C"
extern "C" {
    pub fn botimport_Print(print_type: c_int, fmt: *const c_char, ...);
    pub fn BotReachabilityArea(origin: *const f32, client: c_int) -> c_int;
    pub fn LibVarValue(name: *const c_char, default: *const c_char) -> f32;
    pub fn LibVarSet(name: *const c_char, value: *const c_char);
    pub fn LibVarString(name: *const c_char, default: *const c_char) -> *const c_char;
    pub fn LibVar(name: *const c_char, default: *const c_char) -> *mut libvar_t;
    pub fn PC_SetBaseFolder(folder: *const c_char);
    pub fn LoadSourceFile(filename: *const c_char) -> *mut source_t;
    pub fn PC_ReadToken(source: *mut source_t, token: *mut token_t) -> c_int;
    pub fn PC_ExpectTokenType(source: *mut source_t, type_: c_int, subtype: c_int, token: *mut token_t) -> c_int;
    pub fn StripDoubleQuotes(str: *mut c_char);
    pub fn ReadStructure(source: *mut source_t, structdef: *const structdef_t, ent: *mut c_char) -> c_int;
    pub fn SourceError(source: *mut source_t, fmt: *const c_char, ...);
    pub fn FreeMemory(mem: *mut c_void);
    pub fn FreeSource(source: *mut source_t);
    pub fn GetClearedHunkMemory(size: usize) -> *mut c_void;
    pub fn GetClearedMemory(size: usize) -> *mut c_void;
    pub fn Com_Memset(dest: *mut c_void, c: c_int, count: usize) -> *mut c_void;
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;
    pub fn Log_Write(fmt: *const c_char, ...);
    pub fn AAS_NextBSPEntity(ent: c_int) -> c_int;
    pub fn AAS_ValueForBSPEpairKey(ent: c_int, key: *const c_char, value: *mut c_char, size: usize) -> c_int;
    pub fn AAS_VectorForBSPEpairKey(ent: c_int, key: *const c_char, value: *mut f32) -> c_int;
    pub fn AAS_IntForBSPEpairKey(ent: c_int, key: *const c_char, value: *mut c_int) -> c_int;
    pub fn AAS_FloatForBSPEpairKey(ent: c_int, key: *const c_char, value: *mut f32) -> c_int;
    pub fn AAS_PointAreaNum(point: *const f32) -> c_int;
    pub fn AAS_PointContents(point: *const f32) -> c_int;
    pub fn AAS_Trace(start: *const f32, mins: *const f32, maxs: *const f32, end: *const f32, passent: c_int, contentmask: c_int) -> bsp_trace_t;
    pub fn AAS_BestReachableFromJumpPadArea(origin: *const f32, mins: *const f32, maxs: *const f32) -> c_int;
    pub fn AAS_DropToFloor(origin: *mut f32, mins: *const f32, maxs: *const f32) -> c_int;
    pub fn AAS_BestReachableArea(origin: *const f32, mins: *const f32, maxs: *const f32, goalorigin: *mut f32) -> c_int;
    pub fn AAS_Loaded() -> c_int;
    pub fn AAS_Time() -> f32;
    pub fn AAS_AreaReachability(areanum: c_int) -> c_int;
    pub fn AAS_NextEntity(ent: c_int) -> c_int;
    pub fn AAS_EntityModelindex(ent: c_int) -> c_int;
    pub fn AAS_EntityInfo(ent: c_int, entinfo: *mut aas_entityinfo_t);
    pub fn AAS_EntityType(ent: c_int) -> c_int;
    pub fn AAS_AreaJumpPad(areanum: c_int) -> c_int;
    pub fn AAS_AreaTravelTimeToGoalArea(areanum: c_int, origin: *const f32, goalareanum: c_int, travelflags: c_int) -> c_int;
    pub fn AAS_PresenceTypeBoundingBox(type_: c_int, mins: *mut f32, maxs: *mut f32);
    pub fn FindFuzzyWeight(iwc: *mut weightconfig_s, name: *const c_char) -> c_int;
    pub fn InterbreedWeightConfigs(config1: *mut weightconfig_s, config2: *mut weightconfig_s, config3: *mut weightconfig_s);
    pub fn EvolveWeightConfig(config: *mut weightconfig_s);
    pub fn FuzzyWeightUndecided(inventory: *mut c_int, config: *mut weightconfig_s, weightnum: c_int) -> f32;
    pub fn FuzzyWeight(inventory: *mut c_int, config: *mut weightconfig_s, weightnum: c_int) -> f32;
    pub fn ReadWeightConfig(filename: *const c_char) -> *mut weightconfig_s;
    pub fn FreeWeightConfig(config: *mut weightconfig_s);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub static bot_developer: c_int;
}

// Utility macros for pointer operations
macro_rules! addr_of_mut {
    ($x:expr) => {
        &mut $x as *mut _
    };
}

//========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//========================================================================
pub unsafe fn BotGoalStateFromHandle(handle: c_int) -> *mut bot_goalstate_t {
    if handle <= 0 || (handle as usize) > MAX_CLIENTS {
        botimport_Print(PRT_FATAL, b"goal state handle %d out of range\n\0".as_ptr() as *const c_char, handle);
        return core::ptr::null_mut();
    } //end if
    if BOTGOALSTATES[handle as usize].is_null() {
        botimport_Print(PRT_FATAL, b"invalid goal state %d\n\0".as_ptr() as *const c_char, handle);
        return core::ptr::null_mut();
    } //end if
    return BOTGOALSTATES[handle as usize];
} //end of the function BotGoalStateFromHandle
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotInterbreedGoalFuzzyLogic(parent1: c_int, parent2: c_int, child: c_int) {
    let p1: *mut bot_goalstate_t;
    let p2: *mut bot_goalstate_t;
    let c: *mut bot_goalstate_t;

    p1 = BotGoalStateFromHandle(parent1);
    p2 = BotGoalStateFromHandle(parent2);
    c = BotGoalStateFromHandle(child);

    InterbreedWeightConfigs((*p1).itemweightconfig, (*p2).itemweightconfig, (*c).itemweightconfig);
} //end of the function BotInterbreedingGoalFuzzyLogic
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotSaveGoalFuzzyLogic(goalstate: c_int, filename: *mut c_char) {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);

    //WriteWeightConfig(filename, gs->itemweightconfig);
} //end of the function BotSaveGoalFuzzyLogic
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotMutateGoalFuzzyLogic(goalstate: c_int, range: f32) {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);

    EvolveWeightConfig((*gs).itemweightconfig);
} //end of the function BotMutateGoalFuzzyLogic
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn LoadItemConfig(filename: *mut c_char) -> *mut itemconfig_t {
    let mut max_iteminfo: c_int;
    let mut token: token_t = core::mem::zeroed();
    let mut path: [c_char; MAX_PATH] = [0; MAX_PATH];
    let mut source: *mut source_t;
    let ic: *mut itemconfig_t;
    let ii: *mut iteminfo_t;

    max_iteminfo = LibVarValue(b"max_iteminfo\0".as_ptr() as *const c_char, b"256\0".as_ptr() as *const c_char) as c_int;
    if max_iteminfo < 0 {
        botimport_Print(PRT_ERROR, b"max_iteminfo = %d\n\0".as_ptr() as *const c_char, max_iteminfo);
        max_iteminfo = 256;
        LibVarSet(b"max_iteminfo\0".as_ptr() as *const c_char, b"256\0".as_ptr() as *const c_char);
    }

    // strncpy equivalent
    let mut i = 0;
    while i < MAX_PATH && *filename.add(i) != 0 {
        path[i] = *filename.add(i);
        i += 1;
    }
    if i < MAX_PATH {
        path[i] = 0;
    }

    PC_SetBaseFolder(BOTFILESBASEFOLDER.as_ptr() as *const c_char);
    source = LoadSourceFile(path.as_ptr() as *const c_char);
    if source.is_null() {
        botimport_Print(PRT_ERROR, b"counldn\'t load %s\n\0".as_ptr() as *const c_char, path.as_ptr());
        return core::ptr::null_mut();
    } //end if
    //initialize item config
    ic = GetClearedHunkMemory(core::mem::size_of::<itemconfig_t>() + (max_iteminfo as usize) * core::mem::size_of::<iteminfo_t>()) as *mut itemconfig_t;
    (*ic).iteminfo = (ic as *mut c_char).add(core::mem::size_of::<itemconfig_t>()) as *mut iteminfo_t;
    (*ic).numiteminfo = 0;
    //parse the item config file
    while PC_ReadToken(source, addr_of_mut!(token)) != 0 {
        if Q_stricmp(token.string.as_ptr(), b"iteminfo\0".as_ptr() as *const c_char) == 0 {
            if (*ic).numiteminfo >= max_iteminfo {
                SourceError(source, b"more than %d item info defined\n\0".as_ptr() as *const c_char, max_iteminfo);
                FreeMemory(ic as *mut c_void);
                FreeSource(source);
                return core::ptr::null_mut();
            } //end if
            ii = (*ic).iteminfo.add((*ic).numiteminfo as usize);
            Com_Memset(ii as *mut c_void, 0, core::mem::size_of::<iteminfo_t>());
            if PC_ExpectTokenType(source, 0, 0, addr_of_mut!(token)) == 0 {
                FreeMemory(ic as *mut c_void);
                FreeMemory(source as *mut c_void);
                return core::ptr::null_mut();
            } //end if
            StripDoubleQuotes((*token).string.as_mut_ptr());
            // strncpy for classname
            let mut j = 0;
            while j < 31 && *(*token).string.as_ptr().add(j) != 0 {
                (*ii).classname[j] = *(*token).string.as_ptr().add(j);
                j += 1;
            }
            if j < 31 {
                (*ii).classname[j] = 0;
            }
            if ReadStructure(source, &iteminfo_struct, ii as *mut c_char) == 0 {
                FreeMemory(ic as *mut c_void);
                FreeSource(source);
                return core::ptr::null_mut();
            } //end if
            (*ii).number = (*ic).numiteminfo;
            (*ic).numiteminfo += 1;
        } //end if
        else {
            SourceError(source, b"unknown definition %s\n\0".as_ptr() as *const c_char, (*token).string.as_ptr());
            FreeMemory(ic as *mut c_void);
            FreeSource(source);
            return core::ptr::null_mut();
        } //end else
    } //end while
    FreeSource(source);
    //
    if (*ic).numiteminfo == 0 {
        botimport_Print(PRT_WARNING, b"no item info loaded\n\0".as_ptr() as *const c_char);
    }
    botimport_Print(PRT_MESSAGE, b"loaded %s\n\0".as_ptr() as *const c_char, path.as_ptr());
    return ic;
} //end of the function LoadItemConfig
//===========================================================================
// index to find the weight function of an iteminfo
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn ItemWeightIndex(iwc: *mut weightconfig_s, ic: *mut itemconfig_t) -> *mut c_int {
    let index: *mut c_int;
    let mut i: c_int;

    //initialize item weight index
    index = GetClearedMemory(core::mem::size_of::<c_int>() * ((*ic).numiteminfo as usize)) as *mut c_int;

    i = 0;
    while i < (*ic).numiteminfo {
        *index.add(i as usize) = FindFuzzyWeight(iwc, (*(*ic).iteminfo.add(i as usize)).classname.as_ptr());
        if *index.add(i as usize) < 0 {
            Log_Write(b"item info %d \"%s\" has no fuzzy weight\r\n\0".as_ptr() as *const c_char, i, (*(*ic).iteminfo.add(i as usize)).classname.as_ptr());
        } //end if
        i += 1;
    } //end for
    return index;
} //end of the function ItemWeightIndex
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn InitLevelItemHeap() {
    let mut i: c_int;
    let mut max_levelitems: c_int;

    if !LEVELITEMHEAP.is_null() {
        FreeMemory(LEVELITEMHEAP as *mut c_void);
    }

    max_levelitems = LibVarValue(b"max_levelitems\0".as_ptr() as *const c_char, b"256\0".as_ptr() as *const c_char) as c_int;
    LEVELITEMHEAP = GetClearedMemory((max_levelitems as usize) * core::mem::size_of::<levelitem_t>()) as *mut levelitem_t;

    i = 0;
    while i < max_levelitems - 1 {
        (*LEVELITEMHEAP.add(i as usize)).next = LEVELITEMHEAP.add((i + 1) as usize);
        i += 1;
    } //end for
    (*LEVELITEMHEAP.add((max_levelitems - 1) as usize)).next = core::ptr::null_mut();
    //
    FREELEVELITEMS = LEVELITEMHEAP;
} //end of the function InitLevelItemHeap
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AllocLevelItem() -> *mut levelitem_t {
    let li: *mut levelitem_t;

    li = FREELEVELITEMS;
    if li.is_null() {
        botimport_Print(PRT_FATAL, b"out of level items\n\0".as_ptr() as *const c_char);
        return core::ptr::null_mut();
    } //end if
    //
    FREELEVELITEMS = (*FREELEVELITEMS).next;
    Com_Memset(li as *mut c_void, 0, core::mem::size_of::<levelitem_t>());
    return li;
} //end of the function AllocLevelItem
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn FreeLevelItem(li: *mut levelitem_t) {
    (*li).next = FREELEVELITEMS;
    FREELEVELITEMS = li;
} //end of the function FreeLevelItem
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AddLevelItemToList(li: *mut levelitem_t) {
    if !LEVELITEMS.is_null() {
        (*LEVELITEMS).prev = li;
    }
    (*li).prev = core::ptr::null_mut();
    (*li).next = LEVELITEMS;
    LEVELITEMS = li;
} //end of the function AddLevelItemToList
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn RemoveLevelItemFromList(li: *mut levelitem_t) {
    if !(*li).prev.is_null() {
        (*(*li).prev).next = (*li).next;
    } else {
        LEVELITEMS = (*li).next;
    }
    if !(*li).next.is_null() {
        (*(*li).next).prev = (*li).prev;
    }
} //end of the function RemoveLevelItemFromList
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotFreeInfoEntities() {
    let mut ml: *mut maplocation_t;
    let mut nextml: *mut maplocation_t;
    let mut cs: *mut campspot_t;
    let mut nextcs: *mut campspot_t;

    ml = MAPLOCATIONS;
    while !ml.is_null() {
        nextml = (*ml).next;
        FreeMemory(ml as *mut c_void);
        ml = nextml;
    } //end for
    MAPLOCATIONS = core::ptr::null_mut();
    cs = CAMPSPOTS;
    while !cs.is_null() {
        nextcs = (*cs).next;
        FreeMemory(cs as *mut c_void);
        cs = nextcs;
    } //end for
    CAMPSPOTS = core::ptr::null_mut();
} //end of the function BotFreeInfoEntities
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotInitInfoEntities() {
    let mut classname: [c_char; MAX_EPAIRKEY] = [0; MAX_EPAIRKEY];
    let ml: *mut maplocation_t;
    let cs: *mut campspot_t;
    let mut ent: c_int;
    let mut numlocations: c_int = 0;
    let mut numcampspots: c_int = 0;

    BotFreeInfoEntities();
    //
    numlocations = 0;
    numcampspots = 0;
    ent = AAS_NextBSPEntity(0);
    while ent != 0 {
        if AAS_ValueForBSPEpairKey(ent, b"classname\0".as_ptr() as *const c_char, classname.as_mut_ptr(), MAX_EPAIRKEY) == 0 {
            ent = AAS_NextBSPEntity(ent);
            continue;
        }

        //map locations
        if Q_stricmp(classname.as_ptr(), b"target_location\0".as_ptr() as *const c_char) == 0 {
            let ml = GetClearedMemory(core::mem::size_of::<maplocation_t>()) as *mut maplocation_t;
            AAS_VectorForBSPEpairKey(ent, b"origin\0".as_ptr() as *const c_char, (*ml).origin.as_mut_ptr());
            AAS_ValueForBSPEpairKey(ent, b"message\0".as_ptr() as *const c_char, (*ml).name.as_mut_ptr(), core::mem::size_of_val(&(*ml).name));
            (*ml).areanum = AAS_PointAreaNum((*ml).origin.as_ptr());
            (*ml).next = MAPLOCATIONS;
            MAPLOCATIONS = ml;
            numlocations += 1;
        } //end if
        //camp spots
        else if Q_stricmp(classname.as_ptr(), b"info_camp\0".as_ptr() as *const c_char) == 0 {
            let cs = GetClearedMemory(core::mem::size_of::<campspot_t>()) as *mut campspot_t;
            AAS_VectorForBSPEpairKey(ent, b"origin\0".as_ptr() as *const c_char, (*cs).origin.as_mut_ptr());
            //cs->origin[2] += 16;
            AAS_ValueForBSPEpairKey(ent, b"message\0".as_ptr() as *const c_char, (*cs).name.as_mut_ptr(), core::mem::size_of_val(&(*cs).name));
            AAS_FloatForBSPEpairKey(ent, b"range\0".as_ptr() as *const c_char, addr_of_mut!((*cs).range));
            AAS_FloatForBSPEpairKey(ent, b"weight\0".as_ptr() as *const c_char, addr_of_mut!((*cs).weight));
            AAS_FloatForBSPEpairKey(ent, b"wait\0".as_ptr() as *const c_char, addr_of_mut!((*cs).wait));
            AAS_FloatForBSPEpairKey(ent, b"random\0".as_ptr() as *const c_char, addr_of_mut!((*cs).random));
            (*cs).areanum = AAS_PointAreaNum((*cs).origin.as_ptr());
            if (*cs).areanum == 0 {
                botimport_Print(PRT_MESSAGE, b"camp spot at %1.1f %1.1f %1.1f in solid\n\0".as_ptr() as *const c_char, (*cs).origin[0], (*cs).origin[1], (*cs).origin[2]);
                FreeMemory(cs as *mut c_void);
                ent = AAS_NextBSPEntity(ent);
                continue;
            } //end if
            (*cs).next = CAMPSPOTS;
            CAMPSPOTS = cs;
            //AAS_DrawPermanentCross(cs->origin, 4, LINECOLOR_YELLOW);
            numcampspots += 1;
        } //end else if
        ent = AAS_NextBSPEntity(ent);
    } //end for
    if bot_developer != 0 {
        botimport_Print(PRT_MESSAGE, b"%d map locations\n\0".as_ptr() as *const c_char, numlocations);
        botimport_Print(PRT_MESSAGE, b"%d camp spots\n\0".as_ptr() as *const c_char, numcampspots);
    } //end if
} //end of the function BotInitInfoEntities
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotInitLevelItems() {
    let mut i: c_int;
    let mut spawnflags: c_int;
    let mut value: c_int;
    let mut classname: [c_char; MAX_EPAIRKEY] = [0; MAX_EPAIRKEY];
    let mut origin: [f32; 3] = [0.0; 3];
    let mut end: [f32; 3] = [0.0; 3];
    let mut ent: c_int;
    let mut goalareanum: c_int;
    let ic: *mut itemconfig_t;
    let li: *mut levelitem_t;
    let mut trace: bsp_trace_t;

    //initialize the map locations and camp spots
    BotInitInfoEntities();

    //initialize the level item heap
    InitLevelItemHeap();
    LEVELITEMS = core::ptr::null_mut();
    NUMLEVELITEMS = 0;
    //
    ic = ITEMCONFIG;
    if ic.is_null() {
        return;
    }

    //if there's no AAS file loaded
    if AAS_Loaded() == 0 {
        return;
    }

    //update the modelindexes of the item info
    i = 0;
    while i < (*ic).numiteminfo {
        //ic->iteminfo[i].modelindex = AAS_IndexFromModel(ic->iteminfo[i].model);
        if (*(*ic).iteminfo.add(i as usize)).modelindex == 0 {
            Log_Write(b"item %s has modelindex 0\0".as_ptr() as *const c_char, (*(*ic).iteminfo.add(i as usize)).classname.as_ptr());
        } //end if
        i += 1;
    } //end for

    ent = AAS_NextBSPEntity(0);
    while ent != 0 {
        if AAS_ValueForBSPEpairKey(ent, b"classname\0".as_ptr() as *const c_char, classname.as_mut_ptr(), MAX_EPAIRKEY) == 0 {
            ent = AAS_NextBSPEntity(ent);
            continue;
        }
        //
        spawnflags = 0;
        AAS_IntForBSPEpairKey(ent, b"spawnflags\0".as_ptr() as *const c_char, addr_of_mut!(spawnflags));
        //
        i = 0;
        while i < (*ic).numiteminfo {
            if Q_stricmp(classname.as_ptr(), (*(*ic).iteminfo.add(i as usize)).classname.as_ptr()) == 0 {
                break;
            }
            i += 1;
        } //end for
        if i >= (*ic).numiteminfo {
            Log_Write(b"entity %s unknown item\r\n\0".as_ptr() as *const c_char, classname.as_ptr());
            ent = AAS_NextBSPEntity(ent);
            continue;
        } //end if
        //get the origin of the item
        if AAS_VectorForBSPEpairKey(ent, b"origin\0".as_ptr() as *const c_char, origin.as_mut_ptr()) == 0 {
            botimport_Print(PRT_ERROR, b"item %s without origin\n\0".as_ptr() as *const c_char, classname.as_ptr());
            ent = AAS_NextBSPEntity(ent);
            continue;
        } //end else
        //
        goalareanum = 0;
        //if it is a floating item
        if (spawnflags & 1) != 0 {
            //if the item is not floating in water
            if (AAS_PointContents(origin.as_ptr()) & CONTENTS_WATER) == 0 {
                end[0] = origin[0];
                end[1] = origin[1];
                end[2] = origin[2] - 32.0;
                trace = AAS_Trace(origin.as_ptr(), (*(*ic).iteminfo.add(i as usize)).mins.as_ptr(), (*(*ic).iteminfo.add(i as usize)).maxs.as_ptr(), end.as_ptr(), -1, CONTENTS_SOLID | CONTENTS_PLAYERCLIP);
                //if the item not near the ground
                if trace.fraction >= 1.0 {
                    //if the item is not reachable from a jumppad
                    goalareanum = AAS_BestReachableFromJumpPadArea(origin.as_ptr(), (*(*ic).iteminfo.add(i as usize)).mins.as_ptr(), (*(*ic).iteminfo.add(i as usize)).maxs.as_ptr());
                    Log_Write(b"item %s reachable from jumppad area %d\r\n\0".as_ptr() as *const c_char, (*(*ic).iteminfo.add(i as usize)).classname.as_ptr(), goalareanum);
                    //botimport.Print(PRT_MESSAGE, "item %s reachable from jumppad area %d\r\n", ic->iteminfo[i].classname, goalareanum);
                    if goalareanum == 0 {
                        ent = AAS_NextBSPEntity(ent);
                        continue;
                    }
                } //end if
            } //end if
        } //end if

        li = AllocLevelItem();
        if li.is_null() {
            return;
        }
        //
        NUMLEVELITEMS += 1;
        (*li).number = NUMLEVELITEMS;
        (*li).timeout = 0.0;
        (*li).entitynum = 0;
        //
        (*li).flags = 0;
        value = 0;
        AAS_IntForBSPEpairKey(ent, b"notfree\0".as_ptr() as *const c_char, addr_of_mut!(value));
        if value != 0 {
            (*li).flags |= IFL_NOTFREE;
        }
        value = 0;
        AAS_IntForBSPEpairKey(ent, b"notteam\0".as_ptr() as *const c_char, addr_of_mut!(value));
        if value != 0 {
            (*li).flags |= IFL_NOTTEAM;
        }
        value = 0;
        AAS_IntForBSPEpairKey(ent, b"notsingle\0".as_ptr() as *const c_char, addr_of_mut!(value));
        if value != 0 {
            (*li).flags |= IFL_NOTSINGLE;
        }
        value = 0;
        AAS_IntForBSPEpairKey(ent, b"notbot\0".as_ptr() as *const c_char, addr_of_mut!(value));
        if value != 0 {
            (*li).flags |= IFL_NOTBOT;
        }
        if Q_stricmp(classname.as_ptr(), b"item_botroam\0".as_ptr() as *const c_char) == 0 {
            (*li).flags |= IFL_ROAM;
            AAS_FloatForBSPEpairKey(ent, b"weight\0".as_ptr() as *const c_char, addr_of_mut!((*li).weight));
        } //end if
        //if not a stationary item
        if (spawnflags & 1) == 0 {
            if AAS_DropToFloor(origin.as_mut_ptr(), (*(*ic).iteminfo.add(i as usize)).mins.as_ptr(), (*(*ic).iteminfo.add(i as usize)).maxs.as_ptr()) == 0 {
                botimport_Print(PRT_MESSAGE, b"%s in solid at (%1.1f %1.1f %1.1f)\n\0".as_ptr() as *const c_char,
                                classname.as_ptr(), origin[0], origin[1], origin[2]);
            } //end if
        } //end if
        //item info of the level item
        (*li).iteminfo = i;
        //origin of the item
        (*li).origin[0] = origin[0];
        (*li).origin[1] = origin[1];
        (*li).origin[2] = origin[2];
        //
        if goalareanum != 0 {
            (*li).goalareanum = goalareanum;
            (*li).goalorigin[0] = origin[0];
            (*li).goalorigin[1] = origin[1];
            (*li).goalorigin[2] = origin[2];
        } //end if
        else {
            //get the item goal area and goal origin
            (*li).goalareanum = AAS_BestReachableArea(origin.as_ptr(),
                            (*(*ic).iteminfo.add(i as usize)).mins.as_ptr(), (*(*ic).iteminfo.add(i as usize)).maxs.as_ptr(),
                            (*li).goalorigin.as_mut_ptr());
            if (*li).goalareanum == 0 {
                botimport_Print(PRT_MESSAGE, b"%s not reachable for bots at (%1.1f %1.1f %1.1f)\n\0".as_ptr() as *const c_char,
                                classname.as_ptr(), origin[0], origin[1], origin[2]);
            } //end if
        } //end else
        //
        AddLevelItemToList(li);
        ent = AAS_NextBSPEntity(ent);
    } //end for
    botimport_Print(PRT_MESSAGE, b"found %d level items\n\0".as_ptr() as *const c_char, NUMLEVELITEMS);
} //end of the function BotInitLevelItems
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotGoalName(number: c_int, name: *mut c_char, size: c_int) {
    let li: *mut levelitem_t;

    if ITEMCONFIG.is_null() {
        return;
    }
    //
    li = LEVELITEMS;
    while !li.is_null() {
        if (*li).number == number {
            // strncpy equivalent
            let mut j = 0;
            while j < (size - 1) as usize && *(*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).name.as_ptr().add(j) != 0 {
                *name.add(j) = *(*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).name.as_ptr().add(j);
                j += 1;
            }
            *name.add(j) = 0;
            return;
        } //end for
        li = (*li).next;
    } //end for
    *name = 0;
    return;
} //end of the function BotGoalName
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotResetAvoidGoals(goalstate: c_int) {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    Com_Memset((*gs).avoidgoals.as_mut_ptr() as *mut c_void, 0, MAX_AVOIDGOALS * core::mem::size_of::<c_int>());
    Com_Memset((*gs).avoidgoaltimes.as_mut_ptr() as *mut c_void, 0, MAX_AVOIDGOALS * core::mem::size_of::<f32>());
} //end of the function BotResetAvoidGoals
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotDumpAvoidGoals(goalstate: c_int) {
    let mut i: c_int;
    let gs: *mut bot_goalstate_t;
    let mut name: [c_char; 32] = [0; 32];

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    i = 0;
    while i < MAX_AVOIDGOALS as c_int {
        if (*gs).avoidgoaltimes[i as usize] >= AAS_Time() {
            BotGoalName((*gs).avoidgoals[i as usize], name.as_mut_ptr(), 32);
            Log_Write(b"avoid goal %s, number %d for %f seconds\0".as_ptr() as *const c_char, name.as_ptr(),
                (*gs).avoidgoals[i as usize], (*gs).avoidgoaltimes[i as usize] - AAS_Time());
        } //end if
        i += 1;
    } //end for
} //end of the function BotDumpAvoidGoals
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotAddToAvoidGoals(gs: *mut bot_goalstate_t, number: c_int, avoidtime: f32) {
    let mut i: c_int;

    i = 0;
    while i < MAX_AVOIDGOALS as c_int {
        //if the avoid goal is already stored
        if (*gs).avoidgoals[i as usize] == number {
            (*gs).avoidgoals[i as usize] = number;
            (*gs).avoidgoaltimes[i as usize] = AAS_Time() + avoidtime;
            return;
        } //end if
        i += 1;
    } //end for

    i = 0;
    while i < MAX_AVOIDGOALS as c_int {
        //if this avoid goal has expired
        if (*gs).avoidgoaltimes[i as usize] < AAS_Time() {
            (*gs).avoidgoals[i as usize] = number;
            (*gs).avoidgoaltimes[i as usize] = AAS_Time() + avoidtime;
            return;
        } //end if
        i += 1;
    } //end for
} //end of the function BotAddToAvoidGoals
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotRemoveFromAvoidGoals(goalstate: c_int, number: c_int) {
    let mut i: c_int;
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    //don't use the goals the bot wants to avoid
    i = 0;
    while i < MAX_AVOIDGOALS as c_int {
        if (*gs).avoidgoals[i as usize] == number && (*gs).avoidgoaltimes[i as usize] >= AAS_Time() {
            (*gs).avoidgoaltimes[i as usize] = 0.0;
            return;
        } //end if
        i += 1;
    } //end for
} //end of the function BotRemoveFromAvoidGoals
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotAvoidGoalTime(goalstate: c_int, number: c_int) -> f32 {
    let mut i: c_int;
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return 0.0;
    }
    //don't use the goals the bot wants to avoid
    i = 0;
    while i < MAX_AVOIDGOALS as c_int {
        if (*gs).avoidgoals[i as usize] == number && (*gs).avoidgoaltimes[i as usize] >= AAS_Time() {
            return (*gs).avoidgoaltimes[i as usize] - AAS_Time();
        } //end if
        i += 1;
    } //end for
    return 0.0;
} //end of the function BotAvoidGoalTime
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotSetAvoidGoalTime(goalstate: c_int, number: c_int, mut avoidtime: f32) {
    let gs: *mut bot_goalstate_t;
    let li: *mut levelitem_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    if avoidtime < 0.0 {
        if ITEMCONFIG.is_null() {
            return;
        }
        //
        li = LEVELITEMS;
        while !li.is_null() {
            if (*li).number == number {
                avoidtime = (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).respawntime;
                if avoidtime == 0.0 {
                    avoidtime = AVOID_DEFAULT_TIME as f32;
                }
                if avoidtime < AVOID_MINIMUM_TIME as f32 {
                    avoidtime = AVOID_MINIMUM_TIME as f32;
                }
                BotAddToAvoidGoals(gs, number, avoidtime);
                return;
            } //end for
            li = (*li).next;
        } //end for
        return;
    } //end if
    else {
        BotAddToAvoidGoals(gs, number, avoidtime);
    } //end else
} //end of the function BotSetAvoidGoalTime
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotGetLevelItemGoal(index: c_int, name: *const c_char, goal: *mut bot_goal_t) -> c_int {
    let mut li: *mut levelitem_t;

    if ITEMCONFIG.is_null() {
        return -1;
    }
    li = LEVELITEMS;
    if index >= 0 {
        while !li.is_null() {
            if (*li).number == index {
                li = (*li).next;
                break;
            } //end if
            li = (*li).next;
        } //end for
    } //end for
    while !li.is_null() {
        //
        if G_GAMETYPE == GT_SINGLE_PLAYER {
            if ((*li).flags & IFL_NOTSINGLE) != 0 {
                li = (*li).next;
                continue;
            }
        } else if G_GAMETYPE >= GT_TEAM {
            if ((*li).flags & IFL_NOTTEAM) != 0 {
                li = (*li).next;
                continue;
            }
        } else {
            if ((*li).flags & IFL_NOTFREE) != 0 {
                li = (*li).next;
                continue;
            }
        }
        if ((*li).flags & IFL_NOTBOT) != 0 {
            li = (*li).next;
            continue;
        }
        //
        if Q_stricmp(name, (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).name.as_ptr()) == 0 {
            (*goal).areanum = (*li).goalareanum;
            (*goal).origin[0] = (*li).goalorigin[0];
            (*goal).origin[1] = (*li).goalorigin[1];
            (*goal).origin[2] = (*li).goalorigin[2];
            (*goal).entitynum = (*li).entitynum;
            (*goal).mins[0] = (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).mins[0];
            (*goal).mins[1] = (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).mins[1];
            (*goal).mins[2] = (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).mins[2];
            (*goal).maxs[0] = (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).maxs[0];
            (*goal).maxs[1] = (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).maxs[1];
            (*goal).maxs[2] = (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).maxs[2];
            (*goal).number = (*li).number;
            (*goal).flags = GFL_ITEM;
            if (*li).timeout != 0.0 {
                (*goal).flags |= GFL_DROPPED;
            }
            //botimport.Print(PRT_MESSAGE, "found li %s\n", itemconfig->iteminfo[li->iteminfo].name);
            return (*li).number;
        } //end if
        li = (*li).next;
    } //end for
    return -1;
} //end of the function BotGetLevelItemGoal
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotGetMapLocationGoal(name: *const c_char, goal: *mut bot_goal_t) -> c_int {
    let mut ml: *mut maplocation_t;
    let mins: [f32; 3] = [-8.0, -8.0, -8.0];
    let maxs: [f32; 3] = [8.0, 8.0, 8.0];

    ml = MAPLOCATIONS;
    while !ml.is_null() {
        if Q_stricmp((*ml).name.as_ptr(), name) == 0 {
            (*goal).areanum = (*ml).areanum;
            (*goal).origin[0] = (*ml).origin[0];
            (*goal).origin[1] = (*ml).origin[1];
            (*goal).origin[2] = (*ml).origin[2];
            (*goal).entitynum = 0;
            (*goal).mins[0] = mins[0];
            (*goal).mins[1] = mins[1];
            (*goal).mins[2] = mins[2];
            (*goal).maxs[0] = maxs[0];
            (*goal).maxs[1] = maxs[1];
            (*goal).maxs[2] = maxs[2];
            return 1;
        } //end if
        ml = (*ml).next;
    } //end for
    return 0;
} //end of the function BotGetMapLocationGoal
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotGetNextCampSpotGoal(num: c_int, goal: *mut bot_goal_t) -> c_int {
    let mut i: c_int;
    let mut cs: *mut campspot_t;
    let mins: [f32; 3] = [-8.0, -8.0, -8.0];
    let maxs: [f32; 3] = [8.0, 8.0, 8.0];
    let mut num_mut = num;

    if num_mut < 0 {
        num_mut = 0;
    }
    i = num_mut;
    cs = CAMPSPOTS;
    while !cs.is_null() {
        i -= 1;
        if i < 0 {
            (*goal).areanum = (*cs).areanum;
            (*goal).origin[0] = (*cs).origin[0];
            (*goal).origin[1] = (*cs).origin[1];
            (*goal).origin[2] = (*cs).origin[2];
            (*goal).entitynum = 0;
            (*goal).mins[0] = mins[0];
            (*goal).mins[1] = mins[1];
            (*goal).mins[2] = mins[2];
            (*goal).maxs[0] = maxs[0];
            (*goal).maxs[1] = maxs[1];
            (*goal).maxs[2] = maxs[2];
            return num_mut + 1;
        } //end if
        cs = (*cs).next;
    } //end for
    return 0;
} //end of the function BotGetNextCampSpotGoal
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotFindEntityForLevelItem(li: *mut levelitem_t) {
    let mut ent: c_int;
    let mut modelindex: c_int;
    let ic: *mut itemconfig_t;
    let mut entinfo: aas_entityinfo_t = core::mem::zeroed();
    let mut dir: [f32; 3] = [0.0; 3];

    ic = ITEMCONFIG;
    if ITEMCONFIG.is_null() {
        return;
    }
    ent = AAS_NextEntity(0);
    while ent != 0 {
        //get the model index of the entity
        modelindex = AAS_EntityModelindex(ent);
        //
        if modelindex == 0 {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //get info about the entity
        AAS_EntityInfo(ent, addr_of_mut!(entinfo));
        //if the entity is still moving
        if entinfo.origin[0] != entinfo.lastvisorigin[0] ||
                entinfo.origin[1] != entinfo.lastvisorigin[1] ||
                entinfo.origin[2] != entinfo.lastvisorigin[2] {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //
        if (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).modelindex == modelindex {
            //check if the entity is very close
            dir[0] = (*li).origin[0] - entinfo.origin[0];
            dir[1] = (*li).origin[1] - entinfo.origin[1];
            dir[2] = (*li).origin[2] - entinfo.origin[2];
            let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            if len < 30.0 {
                //found an entity for this level item
                (*li).entitynum = ent;
            } //end if
        } //end if
        ent = AAS_NextEntity(ent);
    } //end for
} //end of the function BotFindEntityForLevelItem
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================

//NOTE: enum entityType_t in bg_public.h
pub const ET_ITEM: c_int = 2;

pub unsafe fn BotUpdateEntityItems() {
    let mut ent: c_int;
    let mut i: c_int;
    let mut modelindex: c_int;
    let mut dir: [f32; 3];
    let mut li: *mut levelitem_t;
    let mut nextli: *mut levelitem_t;
    let mut entinfo: aas_entityinfo_t = core::mem::zeroed();
    let ic: *mut itemconfig_t;

    //timeout current entity items if necessary
    li = LEVELITEMS;
    while !li.is_null() {
        nextli = (*li).next;
        //if it is a item that will time out
        if (*li).timeout != 0.0 {
            //timeout the item
            if (*li).timeout < AAS_Time() {
                RemoveLevelItemFromList(li);
                FreeLevelItem(li);
            } //end if
        } //end if
        li = nextli;
    } //end for
    //find new entity items
    ic = ITEMCONFIG;
    if ITEMCONFIG.is_null() {
        return;
    }
    //
    ent = AAS_NextEntity(0);
    while ent != 0 {
        if AAS_EntityType(ent) != ET_ITEM {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //get the model index of the entity
        modelindex = AAS_EntityModelindex(ent);
        //
        if modelindex == 0 {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //get info about the entity
        AAS_EntityInfo(ent, addr_of_mut!(entinfo));
        //FIXME: don't do this
        //skip all floating items for now
        //if (entinfo.groundent != ENTITYNUM_WORLD) continue;
        //if the entity is still moving
        if entinfo.origin[0] != entinfo.lastvisorigin[0] ||
                entinfo.origin[1] != entinfo.lastvisorigin[1] ||
                entinfo.origin[2] != entinfo.lastvisorigin[2] {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //check if the entity is already stored as a level item
        li = LEVELITEMS;
        while !li.is_null() {
            //if the level item is linked to an entity
            if (*li).entitynum != 0 && (*li).entitynum == ent {
                //the entity is re-used if the models are different
                if (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).modelindex != modelindex {
                    //remove this level item
                    RemoveLevelItemFromList(li);
                    FreeLevelItem(li);
                    li = core::ptr::null_mut();
                    break;
                } //end if
                else {
                    if entinfo.origin[0] != (*li).origin[0] ||
                        entinfo.origin[1] != (*li).origin[1] ||
                        entinfo.origin[2] != (*li).origin[2]
                    {
                        (*li).origin[0] = entinfo.origin[0];
                        (*li).origin[1] = entinfo.origin[1];
                        (*li).origin[2] = entinfo.origin[2];
                        //also update the goal area number
                        (*li).goalareanum = AAS_BestReachableArea((*li).origin.as_ptr(),
                                        (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).mins.as_ptr(), (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).maxs.as_ptr(),
                                        (*li).goalorigin.as_mut_ptr());
                    } //end if
                    break;
                } //end else
            } //end if
            li = (*li).next;
        } //end for
        if !li.is_null() {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //try to link the entity to a level item
        li = LEVELITEMS;
        while !li.is_null() {
            //if this level item is already linked
            if (*li).entitynum != 0 {
                li = (*li).next;
                continue;
            }
            //
            if G_GAMETYPE == GT_SINGLE_PLAYER {
                if ((*li).flags & IFL_NOTSINGLE) != 0 {
                    li = (*li).next;
                    continue;
                }
            } else if G_GAMETYPE >= GT_TEAM {
                if ((*li).flags & IFL_NOTTEAM) != 0 {
                    li = (*li).next;
                    continue;
                }
            } else {
                if ((*li).flags & IFL_NOTFREE) != 0 {
                    li = (*li).next;
                    continue;
                }
            }
            //if the model of the level item and the entity are the same
            if (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).modelindex == modelindex {
                //check if the entity is very close
                dir[0] = (*li).origin[0] - entinfo.origin[0];
                dir[1] = (*li).origin[1] - entinfo.origin[1];
                dir[2] = (*li).origin[2] - entinfo.origin[2];
                let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                if len < 30.0 {
                    //found an entity for this level item
                    (*li).entitynum = ent;
                    //if the origin is different
                    if entinfo.origin[0] != (*li).origin[0] ||
                        entinfo.origin[1] != (*li).origin[1] ||
                        entinfo.origin[2] != (*li).origin[2]
                    {
                        //update the level item origin
                        (*li).origin[0] = entinfo.origin[0];
                        (*li).origin[1] = entinfo.origin[1];
                        (*li).origin[2] = entinfo.origin[2];
                        //also update the goal area number
                        (*li).goalareanum = AAS_BestReachableArea((*li).origin.as_ptr(),
                                        (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).mins.as_ptr(), (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).maxs.as_ptr(),
                                        (*li).goalorigin.as_mut_ptr());
                    } //end if
                    // #ifdef DEBUG
                    Log_Write(b"linked item %s to an entity\0".as_ptr() as *const c_char, (*(*ITEMCONFIG).iteminfo.add((*li).iteminfo as usize)).classname.as_ptr());
                    // #endif //DEBUG
                    break;
                } //end if
            } //end else
            li = (*li).next;
        } //end for
        if !li.is_null() {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //check if the model is from a known item
        i = 0;
        while i < (*ITEMCONFIG).numiteminfo {
            if (*(*ITEMCONFIG).iteminfo.add(i as usize)).modelindex == modelindex {
                break;
            } //end if
            i += 1;
        } //end for
        //if the model is not from a known item
        if i >= (*ITEMCONFIG).numiteminfo {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //allocate a new level item
        li = AllocLevelItem();
        //
        if li.is_null() {
            ent = AAS_NextEntity(ent);
            continue;
        }
        //entity number of the level item
        (*li).entitynum = ent;
        //number for the level item
        (*li).number = NUMLEVELITEMS + ent;
        //set the item info index for the level item
        (*li).iteminfo = i;
        //origin of the item
        (*li).origin[0] = entinfo.origin[0];
        (*li).origin[1] = entinfo.origin[1];
        (*li).origin[2] = entinfo.origin[2];
        //get the item goal area and goal origin
        (*li).goalareanum = AAS_BestReachableArea((*li).origin.as_ptr(),
                                    (*(*ITEMCONFIG).iteminfo.add(i as usize)).mins.as_ptr(), (*(*ITEMCONFIG).iteminfo.add(i as usize)).maxs.as_ptr(),
                                    (*li).goalorigin.as_mut_ptr());
        //never go for items dropped into jumppads
        if AAS_AreaJumpPad((*li).goalareanum) != 0 {
            FreeLevelItem(li);
            ent = AAS_NextEntity(ent);
            continue;
        } //end if
        //time this item out after 30 seconds
        //dropped items disappear after 30 seconds
        (*li).timeout = AAS_Time() + 30.0;
        //add the level item to the list
        AddLevelItemToList(li);
        //botimport.Print(PRT_MESSAGE, "found new level item %s\n", ic->iteminfo[i].classname);
        ent = AAS_NextEntity(ent);
    } //end for
    /*
    for (li = levelitems; li; li = li->next)
    {
        if (!li->entitynum)
        {
            BotFindEntityForLevelItem(li);
        } //end if
    } //end for*/
} //end of the function BotUpdateEntityItems
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotDumpGoalStack(goalstate: c_int) {
    let mut i: c_int;
    let gs: *mut bot_goalstate_t;
    let mut name: [c_char; 32] = [0; 32];

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    i = 1;
    while i <= (*gs).goalstacktop {
        BotGoalName((*gs).goalstack[i as usize].number, name.as_mut_ptr(), 32);
        Log_Write(b"%d: %s\0".as_ptr() as *const c_char, i, name.as_ptr());
        i += 1;
    } //end for
} //end of the function BotDumpGoalStack
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotPushGoal(goalstate: c_int, goal: *const bot_goal_t) {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    if (*gs).goalstacktop >= (MAX_GOALSTACK - 1) as c_int {
        botimport_Print(PRT_ERROR, b"goal heap overflow\n\0".as_ptr() as *const c_char);
        BotDumpGoalStack(goalstate);
        return;
    } //end if
    (*gs).goalstacktop += 1;
    Com_Memcpy(
        (*gs).goalstack.as_mut_ptr().add((*gs).goalstacktop as usize) as *mut c_void,
        goal as *const c_void,
        core::mem::size_of::<bot_goal_t>(),
    );
} //end of the function BotPushGoal
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotPopGoal(goalstate: c_int) {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    if (*gs).goalstacktop > 0 {
        (*gs).goalstacktop -= 1;
    }
} //end of the function BotPopGoal
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotEmptyGoalStack(goalstate: c_int) {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    (*gs).goalstacktop = 0;
} //end of the function BotEmptyGoalStack
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotGetTopGoal(goalstate: c_int, goal: *mut bot_goal_t) -> c_int {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return 0;
    }
    if (*gs).goalstacktop == 0 {
        return 0;
    }
    Com_Memcpy(
        goal as *mut c_void,
        &(*gs).goalstack[(*gs).goalstacktop as usize] as *const _ as *const c_void,
        core::mem::size_of::<bot_goal_t>(),
    );
    return 1;
} //end of the function BotGetTopGoal
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotGetSecondGoal(goalstate: c_int, goal: *mut bot_goal_t) -> c_int {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return 0;
    }
    if (*gs).goalstacktop <= 1 {
        return 0;
    }
    Com_Memcpy(
        goal as *mut c_void,
        &(*gs).goalstack[((*gs).goalstacktop - 1) as usize] as *const _ as *const c_void,
        core::mem::size_of::<bot_goal_t>(),
    );
    return 1;
} //end of the function BotGetSecondGoal
//===========================================================================
// pops a new long term goal on the goal stack in the goalstate
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotChooseLTGItem(
    goalstate: c_int,
    origin: *const f32,
    inventory: *mut c_int,
    travelflags: c_int,
) -> c_int {
    let mut areanum: c_int;
    let mut t: c_int;
    let mut weightnum: c_int;
    let mut weight: f32;
    let mut bestweight: f32;
    let mut avoidtime: f32;
    let iteminfo: *mut iteminfo_t;
    let ic: *mut itemconfig_t;
    let mut li: *mut levelitem_t;
    let mut bestitem: *mut levelitem_t;
    let mut goal: bot_goal_t = core::mem::zeroed();
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return 0;
    }
    if (*gs).itemweightconfig.is_null() {
        return 0;
    }
    //get the area the bot is in
    areanum = BotReachabilityArea(origin, (*gs).client);
    //if the bot is in solid or if the area the bot is in has no reachability links
    if areanum == 0 || AAS_AreaReachability(areanum) == 0 {
        //use the last valid area the bot was in
        areanum = (*gs).lastreachabilityarea;
    } //end if
    //remember the last area with reachabilities the bot was in
    (*gs).lastreachabilityarea = areanum;
    //if still in solid
    if areanum == 0 {
        return 0;
    }
    //the item configuration
    ic = ITEMCONFIG;
    if ITEMCONFIG.is_null() {
        return 0;
    }
    //best weight and item so far
    bestweight = 0.0;
    bestitem = core::ptr::null_mut();
    Com_Memset(&mut goal as *mut _ as *mut c_void, 0, core::mem::size_of::<bot_goal_t>());
    //go through the items in the level
    li = LEVELITEMS;
    while !li.is_null() {
        if G_GAMETYPE == GT_SINGLE_PLAYER {
            if ((*li).flags & IFL_NOTSINGLE) != 0 {
                li = (*li).next;
                continue;
            }
        } else if G_GAMETYPE >= GT_TEAM {
            if ((*li).flags & IFL_NOTTEAM) != 0 {
                li = (*li).next;
                continue;
            }
        } else {
            if ((*li).flags & IFL_NOTFREE) != 0 {
                li = (*li).next;
                continue;
            }
        }
        if ((*li).flags & IFL_NOTBOT) != 0 {
            li = (*li).next;
            continue;
        }
        //if the item is not in a possible goal area
        if (*li).goalareanum == 0 {
            li = (*li).next;
            continue;
        }
        //FIXME: is this a good thing? added this for items that never spawned into the game (f.i. CTF flags in obelisk)
        if (*li).entitynum == 0 && ((*li).flags & IFL_ROAM) == 0 {
            li = (*li).next;
            continue;
        }
        //get the fuzzy weight function for this item
        iteminfo = &mut (*ic).iteminfo[(*li).iteminfo as usize];
        weightnum = *(*gs).itemweightindex.add((*iteminfo).number as usize);
        if weightnum < 0 {
            li = (*li).next;
            continue;
        }

        // #ifdef UNDECIDEDFUZZY
        weight = FuzzyWeightUndecided(inventory, (*gs).itemweightconfig, weightnum);
        // #else
        // weight = FuzzyWeight(inventory, gs->itemweightconfig, weightnum);
        // #endif //UNDECIDEDFUZZY
        // #ifdef DROPPEDWEIGHT
        //HACK: to make dropped items more attractive
        if (*li).timeout != 0.0 {
            weight += (*DROPPEDWEIGHT).value;
        }
        // #endif //DROPPEDWEIGHT
        //use weight scale for item_botroam
        if ((*li).flags & IFL_ROAM) != 0 {
            weight *= (*li).weight;
        }
        //
        if weight > 0.0 {
            //get the travel time towards the goal area
            t = AAS_AreaTravelTimeToGoalArea(areanum, origin, (*li).goalareanum, travelflags);
            //if the goal is reachable
            if t > 0 {
                //if this item won't respawn before we get there
                avoidtime = BotAvoidGoalTime(goalstate, (*li).number);
                if avoidtime - (t as f32) * 0.009 > 0.0 {
                    li = (*li).next;
                    continue;
                }
                //
                weight /= (t as f32) * TRAVELTIME_SCALE;
                //
                if weight > bestweight {
                    bestweight = weight;
                    bestitem = li;
                } //end if
            } //end if
        } //end if
        li = (*li).next;
    } //end for
    //if no goal item found
    if bestitem.is_null() {
        /*
        //if not in lava or slime
        if (!AAS_AreaLava(areanum) && !AAS_AreaSlime(areanum))
        {
            if (AAS_RandomGoalArea(areanum, travelflags, &goal.areanum, goal.origin))
            {
                VectorSet(goal.mins, -15, -15, -15);
                VectorSet(goal.maxs, 15, 15, 15);
                goal.entitynum = 0;
                goal.number = 0;
                goal.flags = GFL_ROAM;
                goal.iteminfo = 0;
                //push the goal on the stack
                BotPushGoal(goalstate, &goal);
                //
#ifdef DEBUG
                botimport.Print(PRT_MESSAGE, "chosen roam goal area %d\n", goal.areanum);
#endif //DEBUG
                return qtrue;
            } //end if
        } //end if
        */
        return 0;
    } //end if
    //create a bot goal for this item
    iteminfo = &mut (*ic).iteminfo[(*bestitem).iteminfo as usize];
    goal.origin[0] = (*bestitem).goalorigin[0];
    goal.origin[1] = (*bestitem).goalorigin[1];
    goal.origin[2] = (*bestitem).goalorigin[2];
    goal.mins[0] = (*iteminfo).mins[0];
    goal.mins[1] = (*iteminfo).mins[1];
    goal.mins[2] = (*iteminfo).mins[2];
    goal.maxs[0] = (*iteminfo).maxs[0];
    goal.maxs[1] = (*iteminfo).maxs[1];
    goal.maxs[2] = (*iteminfo).maxs[2];
    goal.areanum = (*bestitem).goalareanum;
    goal.entitynum = (*bestitem).entitynum;
    goal.number = (*bestitem).number;
    goal.flags = GFL_ITEM;
    if (*bestitem).timeout != 0.0 {
        goal.flags |= GFL_DROPPED;
    }
    if ((*bestitem).flags & IFL_ROAM) != 0 {
        goal.flags |= GFL_ROAM;
    }
    goal.iteminfo = (*bestitem).iteminfo;
    //if it's a dropped item
    if (*bestitem).timeout != 0.0 {
        avoidtime = AVOID_DROPPED_TIME as f32;
    } //end if
    else {
        avoidtime = (*iteminfo).respawntime;
        if avoidtime == 0.0 {
            avoidtime = AVOID_DEFAULT_TIME as f32;
        }
        if avoidtime < AVOID_MINIMUM_TIME as f32 {
            avoidtime = AVOID_MINIMUM_TIME as f32;
        }
    } //end else
    //add the chosen goal to the goals to avoid for a while
    BotAddToAvoidGoals(gs, (*bestitem).number, avoidtime);
    //push the goal on the stack
    BotPushGoal(goalstate, &goal);
    //
    return 1;
} //end of the function BotChooseLTGItem
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotChooseNBGItem(
    goalstate: c_int,
    origin: *const f32,
    inventory: *mut c_int,
    travelflags: c_int,
    ltg: *mut bot_goal_t,
    maxtime: f32,
) -> c_int {
    let mut areanum: c_int;
    let mut t: c_int;
    let mut weightnum: c_int;
    let mut ltg_time: c_int;
    let mut weight: f32;
    let mut bestweight: f32;
    let mut avoidtime: f32;
    let iteminfo: *mut iteminfo_t;
    let ic: *mut itemconfig_t;
    let mut li: *mut levelitem_t;
    let mut bestitem: *mut levelitem_t;
    let mut goal: bot_goal_t = core::mem::zeroed();
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return 0;
    }
    if (*gs).itemweightconfig.is_null() {
        return 0;
    }
    //get the area the bot is in
    areanum = BotReachabilityArea(origin, (*gs).client);
    //if the bot is in solid or if the area the bot is in has no reachability links
    if areanum == 0 || AAS_AreaReachability(areanum) == 0 {
        //use the last valid area the bot was in
        areanum = (*gs).lastreachabilityarea;
    } //end if
    //remember the last area with reachabilities the bot was in
    (*gs).lastreachabilityarea = areanum;
    //if still in solid
    if areanum == 0 {
        return 0;
    }
    //
    if !ltg.is_null() {
        ltg_time = AAS_AreaTravelTimeToGoalArea(areanum, origin, (*ltg).areanum, travelflags);
    } else {
        ltg_time = 99999;
    }
    //the item configuration
    ic = ITEMCONFIG;
    if ITEMCONFIG.is_null() {
        return 0;
    }
    //best weight and item so far
    bestweight = 0.0;
    bestitem = core::ptr::null_mut();
    Com_Memset(&mut goal as *mut _ as *mut c_void, 0, core::mem::size_of::<bot_goal_t>());
    //go through the items in the level
    li = LEVELITEMS;
    while !li.is_null() {
        if G_GAMETYPE == GT_SINGLE_PLAYER {
            if ((*li).flags & IFL_NOTSINGLE) != 0 {
                li = (*li).next;
                continue;
            }
        } else if G_GAMETYPE >= GT_TEAM {
            if ((*li).flags & IFL_NOTTEAM) != 0 {
                li = (*li).next;
                continue;
            }
        } else {
            if ((*li).flags & IFL_NOTFREE) != 0 {
                li = (*li).next;
                continue;
            }
        }
        if ((*li).flags & IFL_NOTBOT) != 0 {
            li = (*li).next;
            continue;
        }
        //if the item is in a possible goal area
        if (*li).goalareanum == 0 {
            li = (*li).next;
            continue;
        }
        //FIXME: is this a good thing? added this for items that never spawned into the game (f.i. CTF flags in obelisk)
        if (*li).entitynum == 0 && ((*li).flags & IFL_ROAM) == 0 {
            li = (*li).next;
            continue;
        }
        //get the fuzzy weight function for this item
        iteminfo = &mut (*ic).iteminfo[(*li).iteminfo as usize];
        weightnum = *(*gs).itemweightindex.add((*iteminfo).number as usize);
        if weightnum < 0 {
            li = (*li).next;
            continue;
        }
        //
        // #ifdef UNDECIDEDFUZZY
        weight = FuzzyWeightUndecided(inventory, (*gs).itemweightconfig, weightnum);
        // #else
        // weight = FuzzyWeight(inventory, gs->itemweightconfig, weightnum);
        // #endif //UNDECIDEDFUZZY
        // #ifdef DROPPEDWEIGHT
        //HACK: to make dropped items more attractive
        if (*li).timeout != 0.0 {
            weight += (*DROPPEDWEIGHT).value;
        }
        // #endif //DROPPEDWEIGHT
        //use weight scale for item_botroam
        if ((*li).flags & IFL_ROAM) != 0 {
            weight *= (*li).weight;
        }
        //
        if weight > 0.0 {
            //get the travel time towards the goal area
            t = AAS_AreaTravelTimeToGoalArea(areanum, origin, (*li).goalareanum, travelflags);
            //if the goal is reachable
            if t > 0 && (t as f32) < maxtime {
                //if this item won't respawn before we get there
                avoidtime = BotAvoidGoalTime(goalstate, (*li).number);
                if avoidtime - (t as f32) * 0.009 > 0.0 {
                    li = (*li).next;
                    continue;
                }
                //
                weight /= (t as f32) * TRAVELTIME_SCALE;
                //
                if weight > bestweight {
                    t = 0;
                    if !ltg.is_null() && (*li).timeout == 0.0 {
                        //get the travel time from the goal to the long term goal
                        t = AAS_AreaTravelTimeToGoalArea((*li).goalareanum, (*li).goalorigin.as_ptr(), (*ltg).areanum, travelflags);
                    } //end if
                    //if the travel back is possible and doesn't take too long
                    if t <= ltg_time {
                        bestweight = weight;
                        bestitem = li;
                    } //end if
                } //end if
            } //end if
        } //end if
        li = (*li).next;
    } //end for
    //if no goal item found
    if bestitem.is_null() {
        return 0;
    }
    //create a bot goal for this item
    iteminfo = &mut (*ic).iteminfo[(*bestitem).iteminfo as usize];
    goal.origin[0] = (*bestitem).goalorigin[0];
    goal.origin[1] = (*bestitem).goalorigin[1];
    goal.origin[2] = (*bestitem).goalorigin[2];
    goal.mins[0] = (*iteminfo).mins[0];
    goal.mins[1] = (*iteminfo).mins[1];
    goal.mins[2] = (*iteminfo).mins[2];
    goal.maxs[0] = (*iteminfo).maxs[0];
    goal.maxs[1] = (*iteminfo).maxs[1];
    goal.maxs[2] = (*iteminfo).maxs[2];
    goal.areanum = (*bestitem).goalareanum;
    goal.entitynum = (*bestitem).entitynum;
    goal.number = (*bestitem).number;
    goal.flags = GFL_ITEM;
    if (*bestitem).timeout != 0.0 {
        goal.flags |= GFL_DROPPED;
    }
    if ((*bestitem).flags & IFL_ROAM) != 0 {
        goal.flags |= GFL_ROAM;
    }
    goal.iteminfo = (*bestitem).iteminfo;
    //if it's a dropped item
    if (*bestitem).timeout != 0.0 {
        avoidtime = AVOID_DROPPED_TIME as f32;
    } //end if
    else {
        avoidtime = (*iteminfo).respawntime;
        if avoidtime == 0.0 {
            avoidtime = AVOID_DEFAULT_TIME as f32;
        }
        if avoidtime < AVOID_MINIMUM_TIME as f32 {
            avoidtime = AVOID_MINIMUM_TIME as f32;
        }
    } //end else
    //add the chosen goal to the goals to avoid for a while
    BotAddToAvoidGoals(gs, (*bestitem).number, avoidtime);
    //push the goal on the stack
    BotPushGoal(goalstate, &goal);
    //
    return 1;
} //end of the function BotChooseNBGItem
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotTouchingGoal(origin: *const f32, goal: *mut bot_goal_t) -> c_int {
    let mut i: c_int;
    let mut boxmins: [f32; 3] = [0.0; 3];
    let mut boxmaxs: [f32; 3] = [0.0; 3];
    let mut absmins: [f32; 3];
    let mut absmaxs: [f32; 3];
    let safety_maxs: [f32; 3] = [0.0, 0.0, 0.0];
    //{4, 4, 10};
    let safety_mins: [f32; 3] = [0.0, 0.0, 0.0];
    //{-4, -4, 0};

    AAS_PresenceTypeBoundingBox(PRESENCE_NORMAL, boxmins.as_mut_ptr(), boxmaxs.as_mut_ptr());
    absmins[0] = (*goal).mins[0] - boxmaxs[0];
    absmins[1] = (*goal).mins[1] - boxmaxs[1];
    absmins[2] = (*goal).mins[2] - boxmaxs[2];
    absmaxs[0] = (*goal).maxs[0] - boxmins[0];
    absmaxs[1] = (*goal).maxs[1] - boxmins[1];
    absmaxs[2] = (*goal).maxs[2] - boxmins[2];
    absmins[0] += (*goal).origin[0];
    absmins[1] += (*goal).origin[1];
    absmins[2] += (*goal).origin[2];
    absmaxs[0] += (*goal).origin[0];
    absmaxs[1] += (*goal).origin[1];
    absmaxs[2] += (*goal).origin[2];
    //make the box a little smaller for safety
    absmaxs[0] -= safety_maxs[0];
    absmaxs[1] -= safety_maxs[1];
    absmaxs[2] -= safety_maxs[2];
    absmins[0] -= safety_mins[0];
    absmins[1] -= safety_mins[1];
    absmins[2] -= safety_mins[2];

    i = 0;
    while i < 3 {
        if *origin.add(i as usize) < absmins[i as usize] || *origin.add(i as usize) > absmaxs[i as usize] {
            return 0;
        }
        i += 1;
    } //end for
    return 1;
} //end of the function BotTouchingGoal
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotItemGoalInVisButNotVisible(viewer: c_int, eye: *const f32, viewangles: *const f32, goal: *mut bot_goal_t) -> c_int {
    let mut entinfo: aas_entityinfo_t = core::mem::zeroed();
    let trace: bsp_trace_t;
    let mut middle: [f32; 3];

    if ((*goal).flags & GFL_ITEM) == 0 {
        return 0;
    }
    //
    middle[0] = (*goal).mins[0] + (*goal).mins[0];
    middle[1] = (*goal).mins[1] + (*goal).mins[1];
    middle[2] = (*goal).mins[2] + (*goal).mins[2];
    middle[0] *= 0.5;
    middle[1] *= 0.5;
    middle[2] *= 0.5;
    middle[0] += (*goal).origin[0];
    middle[1] += (*goal).origin[1];
    middle[2] += (*goal).origin[2];
    //
    trace = AAS_Trace(eye, core::ptr::null(), core::ptr::null(), middle.as_ptr(), viewer, CONTENTS_SOLID);
    //if the goal middle point is visible
    if trace.fraction >= 1.0 {
        //the goal entity number doesn't have to be valid
        //just assume it's valid
        if (*goal).entitynum <= 0 {
            return 0;
        }
        //
        //if the entity data isn't valid
        AAS_EntityInfo((*goal).entitynum, addr_of_mut!(entinfo));
        //NOTE: for some wacko reason entities are sometimes
        // not updated
        //if (!entinfo.valid) return qtrue;
        if entinfo.ltime < AAS_Time() - 0.5 {
            return 1;
        }
    } //end if
    return 0;
} //end of the function BotItemGoalInVisButNotVisible
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotResetGoalState(goalstate: c_int) {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    Com_Memset((*gs).goalstack.as_mut_ptr() as *mut c_void, 0, MAX_GOALSTACK * core::mem::size_of::<bot_goal_t>());
    (*gs).goalstacktop = 0;
    BotResetAvoidGoals(goalstate);
} //end of the function BotResetGoalState
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotLoadItemWeights(goalstate: c_int, filename: *mut c_char) -> c_int {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return BLERR_CANNOTLOADITEMWEIGHTS;
    }
    //load the weight configuration
    (*gs).itemweightconfig = ReadWeightConfig(filename as *const c_char);
    if (*gs).itemweightconfig.is_null() {
        botimport_Print(PRT_FATAL, b"couldn\'t load weights\n\0".as_ptr() as *const c_char);
        return BLERR_CANNOTLOADITEMWEIGHTS;
    } //end if
    //if there's no item configuration
    if ITEMCONFIG.is_null() {
        return BLERR_CANNOTLOADITEMWEIGHTS;
    }
    //create the item weight index
    (*gs).itemweightindex = ItemWeightIndex((*gs).itemweightconfig, ITEMCONFIG);
    //everything went ok
    return BLERR_NOERROR;
} //end of the function BotLoadItemWeights
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotFreeItemWeights(goalstate: c_int) {
    let gs: *mut bot_goalstate_t;

    gs = BotGoalStateFromHandle(goalstate);
    if gs.is_null() {
        return;
    }
    if !(*gs).itemweightconfig.is_null() {
        FreeWeightConfig((*gs).itemweightconfig);
    }
    if !(*gs).itemweightindex.is_null() {
        FreeMemory((*gs).itemweightindex as *mut c_void);
    }
} //end of the function BotFreeItemWeights
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotAllocGoalState(client: c_int) -> c_int {
    let mut i: c_int;

    i = 1;
    while (i as usize) <= MAX_CLIENTS {
        if BOTGOALSTATES[i as usize].is_null() {
            BOTGOALSTATES[i as usize] = GetClearedMemory(core::mem::size_of::<bot_goalstate_t>()) as *mut bot_goalstate_t;
            (*BOTGOALSTATES[i as usize]).client = client;
            return i;
        } //end if
        i += 1;
    } //end for
    return 0;
} //end of the function BotAllocGoalState
//========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//========================================================================
pub unsafe fn BotFreeGoalState(handle: c_int) {
    if handle <= 0 || (handle as usize) > MAX_CLIENTS {
        botimport_Print(PRT_FATAL, b"goal state handle %d out of range\n\0".as_ptr() as *const c_char, handle);
        return;
    } //end if
    if BOTGOALSTATES[handle as usize].is_null() {
        botimport_Print(PRT_FATAL, b"invalid goal state handle %d\n\0".as_ptr() as *const c_char, handle);
        return;
    } //end if
    BotFreeItemWeights(handle);
    FreeMemory(BOTGOALSTATES[handle as usize] as *mut c_void);
    BOTGOALSTATES[handle as usize] = core::ptr::null_mut();
} //end of the function BotFreeGoalState
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotSetupGoalAI() -> c_int {
    let filename: *const c_char;

    //check if teamplay is on
    G_GAMETYPE = LibVarValue(b"g_gametype\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char) as c_int;
    //item configuration file
    filename = LibVarString(b"itemconfig\0".as_ptr() as *const c_char, b"items.c\0".as_ptr() as *const c_char);
    //load the item configuration
    ITEMCONFIG = LoadItemConfig(filename as *mut c_char);
    if ITEMCONFIG.is_null() {
        botimport_Print(PRT_FATAL, b"couldn\'t load item config\n\0".as_ptr() as *const c_char);
        return BLERR_CANNOTLOADITEMCONFIG;
    } //end if
    //
    DROPPEDWEIGHT = LibVar(b"droppedweight\0".as_ptr() as *const c_char, b"1000\0".as_ptr() as *const c_char);
    //everything went ok
    return BLERR_NOERROR;
} //end of the function BotSetupGoalAI
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotShutdownGoalAI() {
    let mut i: c_int;

    if !ITEMCONFIG.is_null() {
        FreeMemory(ITEMCONFIG as *mut c_void);
    }
    ITEMCONFIG = core::ptr::null_mut();
    if !LEVELITEMHEAP.is_null() {
        FreeMemory(LEVELITEMHEAP as *mut c_void);
    }
    LEVELITEMHEAP = core::ptr::null_mut();
    FREELEVELITEMS = core::ptr::null_mut();
    LEVELITEMS = core::ptr::null_mut();
    NUMLEVELITEMS = 0;

    BotFreeInfoEntities();

    i = 1;
    while (i as usize) <= MAX_CLIENTS {
        if !BOTGOALSTATES[i as usize].is_null() {
            BotFreeGoalState(i);
        } //end if
        i += 1;
    } //end for
} //end of the function BotShutdownGoalAI

// Stub for iteminfo_struct
pub static ITEMINFO_STRUCT: structdef_t = structdef_s { _dummy: 0 };
