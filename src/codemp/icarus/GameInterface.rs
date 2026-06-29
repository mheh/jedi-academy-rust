// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// ICARUS Utility functions
// rww - mangled to work in server exe setting.

// #include "Q3_Interface.h"
// #include "g_roff.h"

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_void};
use std::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::g_public_h::{NUM_BSETS, MAX_GENTITIES};
use crate::codemp::server::server_h::{sharedEntity_t, SV_GentityNum, gvm, sv, server_t};
use crate::codemp::icarus::GameInterface_h::{
    ICARUS_Instance, iICARUS, ICARUS_BufferList, ICARUS_EntList, pscript_t, pscript_s,
    bufferlist_t, entlist_t, interface_export_t, interface_export, Interface_Init,
    ICARUS_RegisterScript,
};
use crate::codemp::icarus::Q3_Interface_h::{
    ICARUS_GetIDForString, GET_ID_FOR_STRING, SET_SPAWNSCRIPT, SET_USESCRIPT, SET_AWAKESCRIPT,
    SET_ANGERSCRIPT, SET_ATTACKSCRIPT, SET_VICTORYSCRIPT, SET_LOSTENEMYSCRIPT, SET_PAINSCRIPT,
    SET_FLEESCRIPT, SET_DEATHSCRIPT, SET_DELAYEDSCRIPT, SET_BLOCKEDSCRIPT, SET_FFIRESCRIPT,
    SET_FFDEATHSCRIPT, SET_MINDTRICKSCRIPT, SET_CINEMATIC_SKIPSCRIPT, SET_LOOPSOUND,
    SET_VIDEO_PLAY, SET_ADDRHANDBOLT_MODEL, SET_ADDLHANDBOLT_MODEL,
};
use crate::codemp::icarus::interpreter_h::{
    ID_CAMERA, ID_PLAY, ID_RUN, ID_SOUND, ID_SET, TYPE_PATH,
};
use crate::codemp::icarus::blockstream_h::{
    CBlockStream, CBlockMember, CBlock, IBI_EXT,
};
use crate::oracle::{
    Com_Error, Com_Printf, FS_ReadFile, FS_FreeFile, Q_strupr, Q_stricmp,
    GetIDForString, COM_StripExtension, stringID_table_t,
};
use crate::ffi::game_export::{GAME_ICARUS_SOUNDINDEX, GAME_ICARUS_GETSETIDFORSTRING};

extern "C" {
    fn ICARUS_Malloc(size: c_int) -> *mut c_void;
    fn ICARUS_Free(ptr: *mut c_void);
    fn Q3_DebugPrint(level: c_int, format: *const c_char, ...);
    fn ICARUS_SoundPrecache(filename: *const c_char);
    fn ICARUS_InterrogateScript(filename: *const c_char);
    fn ICARUS_PrecacheEnt(ent: *mut sharedEntity_t);
    fn Q3_TaskIDClear(taskID: *mut c_int);
    fn VM_Call(vm: *mut c_void, callnum: c_int, ...) -> c_int;
    fn sprintf(buf: *mut c_char, format: *const c_char, ...) -> c_int;
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

pub static mut ICARUS_entFilter: c_int = -1;

/*
=============
ICARUS_GetScript

gets the named script from the cache or disk if not already loaded
=============
*/
pub unsafe extern "C" fn ICARUS_GetScript(name: *const c_char, buf: *mut *mut c_char) -> c_int {
    // Make sure the caller is valid

    // Attempt to retrieve a precached script
    // Note: ICARUS_BufferList is a std::map<string, pscript_t*>, simulated as opaque C struct
    // We need to use a C interface to search the map

    // Not found, check the disk
    if ICARUS_RegisterScript(name, 0) == 0 {
        return 0;
    }

    // Script is now inserted, retrieve it and pass through
    // TODO: We need a helper function to search the map since it's C++ in the original
    // For now, this is a structural stub that needs the C++ map implementation

    *buf = core::ptr::null_mut();
    return 0;
}

/*
=============
ICARUS_RunScript

Runs the script by the given name
=============
*/
pub unsafe extern "C" fn ICARUS_RunScript(ent: *mut sharedEntity_t, name: *const c_char) -> c_int {
    let mut buf: *mut c_char;
    let mut len: c_int;

    // Make sure the caller is valid
    let ent_num = (*ent).s.number;
    if ent_num >= MAX_GENTITIES as c_int || ent_num < 0 {
        return 0;
    }

    // Note: gSequencers is defined in Instance.rs
    // if gSequencers[ent_num] == NULL {
    //     return 0;
    // }

    // Only handle the simple case for now
    #[cfg(feature = "_HACK_FOR_TESTING_ONLY_1")]
    {
        let mut namex: [c_char; 1024] = [0; 1024];
        let blah = strstr(name, "stu/\0".as_ptr() as *const c_char);
        if !blah.is_null() {
            let r = (blah as usize) - (name as usize);
            let mut i = 0;
            while i < r {
                namex[i] = *(name.add(i));
                i += 1;
            }
            namex[i] = 0;
            strcat(namex.as_mut_ptr(), "ignorethisfolder/\0".as_ptr() as *const c_char);
            i = strlen(namex.as_ptr() as *const c_char) as usize;
            let mut r = r;
            while *(name.add(r)) != '/' as c_char {
                r += 1;
            }
            r += 1;

            while *(name.add(r)) != 0 {
                namex[i] = *(name.add(r));
                r += 1;
                i += 1;
            }
            namex[i] = 0;
        } else {
            strcpy(namex.as_mut_ptr(), name);
        }

        len = ICARUS_GetScript(namex.as_ptr(), &mut buf);
    }

    #[cfg(not(feature = "_HACK_FOR_TESTING_ONLY_1"))]
    {
        len = ICARUS_GetScript(name, &mut buf);
    }

    if len == 0 {
        return 0;
    }

    // Attempt to run the script
    // Note: gSequencers[ent->s.number]->Run() needs proper implementation
    // if S_FAILED(gSequencers[ent_num as usize]->Run(buf, len)) {
    //     return 0;
    // }

    if (ICARUS_entFilter == -1) || (ICARUS_entFilter == ent_num) {
        Q3_DebugPrint(
            3, // WL_VERBOSE
            "%d Script %s executed by %s %s\n\0".as_ptr() as *const c_char,
            (*sv).time,
            name,
            (*ent).classname,
            (*ent).targetname,
        );
    }

    return 1; // true
}

/*
=================
ICARUS_Init

Allocates a new ICARUS instance
=================
*/
pub unsafe extern "C" fn ICARUS_Init() {
    // Link all interface functions
    Interface_Init(addr_of_mut!(interface_export) as *mut interface_export_t);

    // Create the ICARUS instance for this session
    // iICARUS = ICARUS_Instance::Create(&interface_export);
    // Note: This needs proper C++ binding
    // if iICARUS == NULL {
    //     Com_Error(ERR_DROP, "Unable to initialize ICARUS instance\n");
    //     return;
    // }
}

/*
=================
ICARUS_Shutdown

Frees up ICARUS resources from all entities
=================
*/
pub unsafe extern "C" fn ICARUS_Shutdown() {
    let mut ent: *mut sharedEntity_t;

    // Release all ICARUS resources from the entities
    for i in 0..MAX_GENTITIES {
        ent = SV_GentityNum(i as c_int);

        // Note: gSequencers needs to be accessed here
        // if gSequencers[i] {
        //     if ent->s.number >= MAX_GENTITIES || ent->s.number < 0 {
        //         ent->s.number = i as c_int;
        //         assert(0);
        //     }
        //     ICARUS_FreeEnt(ent);
        // }
    }

    // Clear out all precached scripts
    // Note: ICARUS_BufferList is a C++ std::map, needs proper C interface
    // for (ei = ICARUS_BufferList.begin(); ei != ICARUS_BufferList.end(); ei++) {
    //     ICARUS_Free((*ei).second->buffer);
    //     delete (*ei).second;
    // }

    // ICARUS_BufferList.clear();

    // Clear the name map
    // ICARUS_EntList.clear();

    // Free this instance
    // if iICARUS {
    //     iICARUS->Delete();
    //     iICARUS = NULL;
    // }
}

/*
==============
ICARUS_FreeEnt

Frees all ICARUS resources on an entity

WARNING!!! DO NOT DO THIS WHILE RUNNING A SCRIPT, ICARUS WILL CRASH!!!
FIXME: shouldn't ICARUS handle this internally?

==============
*/
pub unsafe extern "C" fn ICARUS_FreeEnt(ent: *mut sharedEntity_t) {
    // assert(iICARUS);

    let ent_num = (*ent).s.number;
    if ent_num >= MAX_GENTITIES as c_int || ent_num < 0 {
        // assert(0);
        return;
    }

    // Make sure the ent is valid
    // Note: gSequencers needs to be accessed
    // if gSequencers[ent_num as usize] == NULL {
    //     return;
    // }

    // Remove them from the ICARUSE_EntList list so that when their g_entity index is reused,
    // ICARUS doesn't try to affect the new (incorrect) ent.
    if VALIDSTRING((*ent).script_targetname) {
        let mut temp: [c_char; 1024] = [0; 1024];

        strncpy(temp.as_mut_ptr(), (*ent).script_targetname, 1023);
        temp[1023] = 0;

        // Note: ICARUS_EntList is a C++ std::map
        // entlist_t::iterator it = ICARUS_EntList.find(Q_strupr(temp));
        // if (it != ICARUS_EntList.end()) {
        //     ICARUS_EntList.erase(it);
        // }
    }

    // Delete the sequencer and the task manager
    // Note: iICARUS->DeleteSequencer() needs proper C++ binding
    // iICARUS->DeleteSequencer(gSequencers[ent_num as usize]);

    // Clean up the pointers
    // gSequencers[ent_num as usize] = NULL;
    // gTaskManagers[ent_num as usize] = NULL;
}

/*
==============
ICARUS_ValidEnt

Determines whether or not an entity needs ICARUS information
==============
*/
pub unsafe extern "C" fn ICARUS_ValidEnt(ent: *mut sharedEntity_t) -> c_int {
    let mut i: c_int;

    // Targeted by a script
    if VALIDSTRING((*ent).script_targetname) {
        return 1; // true
    }

    // Potentially able to call a script
    for i in 0..NUM_BSETS {
        if VALIDSTRING((*ent).behaviorSet[i]) {
            // Com_Printf("WARNING: Entity %d (%s) has behaviorSet but no script_targetname -- using targetname\n",
            //            ent->s.number, ent->targetname);

            // ent->script_targetname = ent->targetname;
            // rww - You CANNOT do things like this now. We're switching memory around to be able to read this memory from vm land,
            // and while this allows us to read it on our "fake" entity here, we can't modify pointers like this. We can however do
            // something completely hackish such as the following.

            let ent_num = (*ent).s.number;
            // assert(ent_num >= 0 && ent_num < MAX_GENTITIES);
            let trueEntity: *mut sharedEntity_t = SV_GentityNum(ent_num);
            // This works because we're modifying the actual shared game vm data and turning one pointer into another.
            // While these pointers both look like garbage to us in here, they are not.
            (*trueEntity).script_targetname = (*trueEntity).targetname;
            return 1; // true
        }
    }

    return 0; // false
}

/*
==============
ICARUS_AssociateEnt

Associate the entity's id and name so that it can be referenced later
==============
*/
pub unsafe extern "C" fn ICARUS_AssociateEnt(ent: *mut sharedEntity_t) {
    let mut temp: [c_char; 1024] = [0; 1024];

    if VALIDSTRING((*ent).script_targetname) == 0 {
        return;
    }

    strncpy(temp.as_mut_ptr(), (*ent).script_targetname, 1023);
    temp[1023] = 0;

    // Note: ICARUS_EntList is a C++ std::map
    // ICARUS_EntList[Q_strupr(temp)] = ent->s.number;
}

/*
==============
ICARUS_RegisterScript

Loads and caches a script
==============
*/
pub unsafe extern "C" fn ICARUS_RegisterScript(
    name: *const c_char,
    bCalledDuringInterrogate: c_int, // qboolean
) -> c_int {
    let mut pscript: *mut pscript_t;
    let mut newname: [c_char; 256] = [0; 256]; // MAX_FILENAME_LENGTH
    let mut buffer: *mut c_char = core::ptr::null_mut();
    let mut length: c_long;

    // Make sure this isn't already cached
    // Note: ICARUS_BufferList is a C++ std::map
    // ei = ICARUS_BufferList.find(name);

    // note special return condition here, if doing interrogate and we already have this file then we MUST return
    // false (which stops the interrogator proceeding), this not only saves some time, but stops a potential
    // script recursion bug which could lock the program in an infinite loop... Return TRUE for normal though!
    //
    // if (ei != ICARUS_BufferList.end()) {
    //     return (bCalledDuringInterrogate) ? 0 : 1;
    // }

    sprintf(newname.as_mut_ptr(), "%s%s\0".as_ptr() as *const c_char, name, IBI_EXT.as_ptr() as *const c_char);

    // small update here, if called during interrogate, don't let gi.FS_ReadFile() complain because it can't
    // find stuff like BS_RUN_AND_SHOOT as scriptname... During FINALBUILD the message won't appear anyway, hence
    // the ifndef, this just cuts down on internal error reports while testing release mode...
    //
    let mut qbIgnoreFileRead: c_int = 0; // qfalse

    // NOTENOTE: For the moment I've taken this back out, to avoid doubling the number of fopen()'s per file.
    // #if 0//#ifndef FINAL_BUILD
    // if (bCalledDuringInterrogate) {
    //     fileHandle_t file;
    //     gi.FS_FOpenFile(newname, &file, FS_READ);
    //     if (file == NULL) {
    //         qbIgnoreFileRead = 1; // qtrue
    //     } else {
    //         gi.FS_FCloseFile(file);
    //     }
    // }
    // #endif

    length = if qbIgnoreFileRead != 0 { -1 } else { FS_ReadFile(newname.as_ptr(), &mut buffer as *mut *mut c_char) as c_long };

    if length <= 0 {
        // File not found, but keep quiet during interrogate stage, because of stuff like BS_RUN_AND_SHOOT as scriptname
        //
        if bCalledDuringInterrogate == 0 {
            Com_Printf("^1Could not open file '%s'\n\0".as_ptr() as *const c_char, newname.as_ptr());
        }
        return 0; // false
    }

    pscript = ICARUS_Malloc(core::mem::size_of::<pscript_t>() as c_int) as *mut pscript_t;

    (*pscript).buffer = ICARUS_Malloc(length as c_int) as *mut c_char; // gi.Malloc(length, TAG_ICARUS, qfalse);
    memcpy((*pscript).buffer as *mut c_void, buffer as *const c_void, length as usize);
    (*pscript).length = length;

    FS_FreeFile(buffer);

    // Note: ICARUS_BufferList is a C++ std::map
    // ICARUS_BufferList[name] = pscript;

    return 1; // true
}

pub unsafe extern "C" fn ICARUS_SoundPrecache(filename: *const c_char) {
    // Note: T_G_ICARUS_SOUNDINDEX is defined in shared memory structures
    // T_G_ICARUS_SOUNDINDEX *sharedMem = (T_G_ICARUS_SOUNDINDEX *)sv.mSharedMemory;
    // strcpy(sharedMem->filename, filename);
    // VM_Call(gvm, GAME_ICARUS_SOUNDINDEX);
}

pub unsafe extern "C" fn ICARUS_GetIDForString(string: *const c_char) -> c_int {
    // Note: T_G_ICARUS_GETSETIDFORSTRING is defined in shared memory structures
    // T_G_ICARUS_GETSETIDFORSTRING *sharedMem = (T_G_ICARUS_GETSETIDFORSTRING *)sv.mSharedMemory;
    // strcpy(sharedMem->string, string);
    // return VM_Call(gvm, GAME_ICARUS_GETSETIDFORSTRING);
    return 0;
}

/*
-------------------------
ICARUS_InterrogateScript
-------------------------
*/

// at this point the filename should have had the "scripts" (Q3_SCRIPT_DIR) added to it (but not the IBI extension)
//
pub unsafe extern "C" fn ICARUS_InterrogateScript(filename: *const c_char) {
    let mut stream: CBlockStream;
    let mut blockMember: *mut CBlockMember;
    let mut block: CBlock;

    if Q_stricmp(filename, "NULL\0".as_ptr() as *const c_char) == 0 || Q_stricmp(filename, "default\0".as_ptr() as *const c_char) == 0 {
        return;
    }

    //////////////////////////////////
    //
    // ensure "scripts" (Q3_SCRIPT_DIR), which will be missing if this was called recursively...
    //
    let mut sFilename: [c_char; 1024] = [0; 1024]; // should really be MAX_QPATH (and 64 bytes instead of 1024), but this fits the rest of the code

    if Q_stricmpn(filename, "scripts/\0".as_ptr() as *const c_char, strlen("scripts/\0".as_ptr() as *const c_char)) == 0 {
        // strcpy(sFilename, filename);
        // Q_strncpyz not needed here, use strncpy
    } else {
        // Build path with va() or sprintf
        // For now, manually construct the path
        strcpy(sFilename.as_mut_ptr(), "scripts/\0".as_ptr() as *const c_char);
        strcat(sFilename.as_mut_ptr(), filename);
    }
    //
    //////////////////////////////////

    // Attempt to register this script
    if ICARUS_RegisterScript(sFilename.as_ptr(), 1) == 0 { // true = bCalledDuringInterrogate
        return;
    }

    let mut buf: *mut c_char;
    let mut len: c_int;

    // Attempt to retrieve the new script data
    if (len = ICARUS_GetScript(sFilename.as_ptr(), &mut buf)) == 0 {
        return;
    }

    // Open the stream
    // if stream.Open(buf, len) == 0 { // qfalse
    //     return;
    // }

    let mut sVal1: *const c_char;
    let mut sVal2: *const c_char;
    let mut temp: [c_char; 1024] = [0; 1024];
    let mut setID: c_int;

    // Now iterate through all blocks of the script, searching for keywords
    // while stream.BlockAvailable() {
    //     // Get a block
    //     if stream.ReadBlock(&block) == 0 { // qfalse
    //         return;
    //     }

    //     // Determine what type of block this is
    //     match block.GetBlockID() {
    //         ID_CAMERA => { // to cache ROFF files
    //             let f = *(block.GetMemberData(0) as *const f32);
    //             if f == TYPE_PATH as f32 {
    //                 sVal1 = block.GetMemberData(1) as *const c_char;
    //                 // we can do this I guess since the roff is loaded on the server.
    //                 // theROFFSystem.Cache((char *)sVal1, qfalse);
    //             }
    //         }

    //         ID_PLAY => { // to cache ROFF files
    //             sVal1 = block.GetMemberData(0) as *const c_char;
    //             if Q_stricmp(sVal1, "PLAY_ROFF\0".as_ptr() as *const c_char) == 0 {
    //                 sVal1 = block.GetMemberData(1) as *const c_char;
    //                 // we can do this I guess since the roff is loaded on the server.
    //                 // theROFFSystem.Cache((char *)sVal1, qfalse);
    //             }
    //         }

    //         ID_RUN => {
    //             sVal1 = block.GetMemberData(0) as *const c_char;
    //             COM_StripExtension(sVal1, temp.as_mut_ptr());
    //             ICARUS_InterrogateScript(temp.as_ptr());
    //         }

    //         ID_SOUND => {
    //             // We can't just call over to S_RegisterSound or whatever because this is on the server.
    //             sVal1 = block.GetMemberData(1) as *const c_char; // 0 is channel, 1 is filename
    //             ICARUS_SoundPrecache(sVal1);
    //         }

    //         ID_SET => {
    //             blockMember = block.GetMember(0);

    //             // NOTENOTE: This will not catch special case get() inlines! (There's not really a good way to do that)

    //             // Make sure we're testing against strings
    //             if (*blockMember).GetID() == TK_STRING {
    //                 sVal1 = block.GetMemberData(0) as *const c_char;
    //                 sVal2 = block.GetMemberData(1) as *const c_char;

    //                 // Get the id for this set identifier
    //                 setID = ICARUS_GetIDForString(sVal1);

    //                 // Check against valid types
    //                 match setID {
    //                     SET_SPAWNSCRIPT | SET_USESCRIPT | SET_AWAKESCRIPT | SET_ANGERSCRIPT | SET_ATTACKSCRIPT |
    //                     SET_VICTORYSCRIPT | SET_LOSTENEMYSCRIPT | SET_PAINSCRIPT | SET_FLEESCRIPT | SET_DEATHSCRIPT |
    //                     SET_DELAYEDSCRIPT | SET_BLOCKEDSCRIPT | SET_FFIRESCRIPT | SET_FFDEATHSCRIPT |
    //                     SET_MINDTRICKSCRIPT | SET_CINEMATIC_SKIPSCRIPT => {
    //                         // Recursively obtain all embedded scripts
    //                         ICARUS_InterrogateScript(sVal2);
    //                     }
    //                     SET_LOOPSOUND => { // like ID_SOUND, but set's looping
    //                         ICARUS_SoundPrecache(sVal2);
    //                     }
    //                     SET_VIDEO_PLAY => { // in game cinematic
    //                         // do nothing for MP.
    //                     }
    //                     SET_ADDRHANDBOLT_MODEL | SET_ADDLHANDBOLT_MODEL => {
    //                         // do nothing for MP
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //         }

    //         _ => {}
    //     }

    //     // Clean out the block for the next pass
    //     block.Free();
    // }

    // // All done
    // stream.Free();
}

#[cfg(not(feature = "_XBOX"))]
pub static BSTable: [stringID_table_t; 11] = [
    stringID_table_t { string: "BS_DEFAULT\0".as_ptr() as *const c_char, id: 0 }, // default behavior for that NPC
    stringID_table_t { string: "BS_ADVANCE_FIGHT\0".as_ptr() as *const c_char, id: 1 }, // Advance to captureGoal and shoot enemies if you can
    stringID_table_t { string: "BS_SLEEP\0".as_ptr() as *const c_char, id: 2 }, // Play awake script when startled by sound
    stringID_table_t { string: "BS_FOLLOW_LEADER\0".as_ptr() as *const c_char, id: 3 }, // Follow your leader and shoot any enemies you come across
    stringID_table_t { string: "BS_JUMP\0".as_ptr() as *const c_char, id: 4 }, // Face navgoal and jump to it.
    stringID_table_t { string: "BS_SEARCH\0".as_ptr() as *const c_char, id: 5 }, // Using current waypoint as a base), search the immediate branches of waypoints for enemies
    stringID_table_t { string: "BS_WANDER\0".as_ptr() as *const c_char, id: 6 }, // Wander down random waypoint paths
    stringID_table_t { string: "BS_NOCLIP\0".as_ptr() as *const c_char, id: 7 }, // Moves through walls), etc.
    stringID_table_t { string: "BS_REMOVE\0".as_ptr() as *const c_char, id: 8 }, // Waits for player to leave PVS then removes itself
    stringID_table_t { string: "BS_CINEMATIC\0".as_ptr() as *const c_char, id: 9 }, // Does nothing but face it's angles and move to a goal if it has one
    stringID_table_t { string: "\0".as_ptr() as *const c_char, id: -1 }, // the rest are internal only
];

/*
==============
ICARUS_PrecacheEnt

Precache all scripts being used by the entity
==============
*/
pub unsafe extern "C" fn ICARUS_PrecacheEnt(ent: *mut sharedEntity_t) {
    let mut newname: [c_char; 256] = [0; 256]; // MAX_FILENAME_LENGTH
    let mut i: usize;

    for i in 0..NUM_BSETS {
        if (*ent).behaviorSet[i].is_null() {
            continue;
        }

        if GetIDForString(BSTable.as_ptr(), (*ent).behaviorSet[i]) == -1 {
            // not a behavior set
            sprintf(
                newname.as_mut_ptr(),
                "%s/%s\0".as_ptr() as *const c_char,
                "scripts\0".as_ptr() as *const c_char,
                (*ent).behaviorSet[i],
            );

            // Precache this, and all internally referenced scripts
            ICARUS_InterrogateScript(newname.as_ptr());
        }
    }
}

/*
==============
ICARUS_InitEnt

Allocates a sequencer and task manager only if an entity is a potential script user
==============
*/
pub unsafe extern "C" fn ICARUS_InitEnt(ent: *mut sharedEntity_t) {
    // Make sure this is a fresh ent
    // assert(iICARUS);
    // assert(gTaskManagers[ent->s.number] == NULL);
    // assert(gSequencers[ent->s.number] == NULL);

    let ent_num = (*ent).s.number as usize;

    // Note: These checks need gSequencers and gTaskManagers which are defined elsewhere
    // if gSequencers[ent_num] != NULL {
    //     return;
    // }

    // if gTaskManagers[ent_num] != NULL {
    //     return;
    // }

    // Create the sequencer and setup the task manager
    // Note: iICARUS->GetSequencer() and GetTaskManager() need proper C++ binding
    // gSequencers[ent_num] = iICARUS->GetSequencer(ent->s.number);
    // gTaskManagers[ent_num] = gSequencers[ent_num]->GetTaskManager();

    // Initialize all taskIDs to -1
    memset(&mut (*ent).taskID as *mut _ as *mut c_void, -1, core::mem::size_of_val(&(*ent).taskID));

    // Add this entity to a map of valid associated ents for quick retrieval later
    ICARUS_AssociateEnt(ent);

    // Precache all the entity's scripts
    ICARUS_PrecacheEnt(ent);
}

/*
-------------------------
ICARUS_LinkEntity
-------------------------
*/
pub unsafe extern "C" fn ICARUS_LinkEntity(
    entID: c_int,
    sequencer: *mut c_void, // CSequencer*
    taskManager: *mut c_void, // CTaskManager*
) -> c_int {
    let ent: *mut sharedEntity_t = SV_GentityNum(entID);

    if ent.is_null() {
        return 0; // false
    }

    let ent_num = (*ent).s.number as usize;

    // Note: gSequencers and gTaskManagers need proper implementation
    // gSequencers[ent_num] = sequencer;
    // gTaskManagers[ent_num] = taskManager;

    ICARUS_AssociateEnt(ent);

    return 1; // true
}

/*
-------------------------
Svcmd_ICARUS_f
-------------------------
*/
pub unsafe extern "C" fn Svcmd_ICARUS_f() {
    // rwwFIXMEFIXME: Do something with this for debugging purposes at some point.
    /*
    char	*cmd = Cmd_Argv(1);

    if (Q_stricmp(cmd, "log") == 0) {
        // g_ICARUSDebug->integer = WL_DEBUG;
        if (VALIDSTRING(Cmd_Argv(2))) {
            sharedEntity_t	*ent = G_Find(NULL, FOFS(script_targetname), gi.argv(2));

            if (ent == NULL) {
                Com_Printf("Entity \"%s\" not found!\n", gi.argv(2));
                return;
            }

            // Start logging
            Com_Printf("Logging ICARUS info for entity %s\n", gi.argv(2));

            ICARUS_entFilter = (ent->s.number == ICARUS_entFilter) ? -1 : ent->s.number;

            return;
        }

        Com_Printf("Logging ICARUS info for all entities\n");

        return;
    }
    */
    return;
}

// Helper function
#[inline]
unsafe fn VALIDSTRING(a: *const c_char) -> c_int {
    if a.is_null() || *a == 0 {
        0
    } else {
        1
    }
}

#[inline]
unsafe fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int {
    // Stub: This should compare first n characters case-insensitively
    // For now, return 0 (equal)
    0
}

#[inline]
unsafe fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char {
    // Stub: This should find the substring
    core::ptr::null()
}
