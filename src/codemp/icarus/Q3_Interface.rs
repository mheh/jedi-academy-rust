#![allow(non_snake_case)]

// ICARUS Engine Interface File
//
//	This file is the only section of the ICARUS systems that
//	is not directly portable from engine to engine.
//
//	-- jweier

use core::ffi::{c_int, c_char, c_void, c_uint};

// External declarations
extern "C" {
    // From g_icarus.cpp
    fn ICARUS_GetScript(name: *const c_char, buf: *mut *mut c_char) -> c_int;

    // From qcommon
    fn Q_flrand(min: f32, max: f32) -> f32;
    fn COM_ParseString(data: *mut *mut c_char, s: *mut *mut c_char) -> c_int;
    fn va(format: *const c_char, ...) -> *const c_char;
    fn vsprintf(text: *mut c_char, format: *const c_char, argptr: *mut core::ffi::c_void) -> c_int;
    fn atof(s: *const c_char) -> f64;
    fn atoi(s: *const c_char) -> c_int;
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn stricmp(a: *const c_char, b: *const c_char) -> c_int;
    fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;

    // Server functions
    fn SV_SendServerCommand(ent: *mut c_void, cmd: *const c_char, ...);
    fn SV_GentityNum(num: c_int) -> *mut sharedEntity_t;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Q_strupr(str: *mut c_char) -> *mut c_char;
    fn VectorCompare(v1: *const [f32; 3], v2: *const [f32; 3]) -> c_int;
    fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    fn VM_Call(vm: *mut c_void, cmd: c_int, ...) -> c_int;

    // ICARUS functions
    fn ICARUS_LinkEntity(entID: c_int, sequencer: *mut CSequencer, taskManager: *mut CTaskManager) -> c_int;

    // Global state
    pub static mut sv: serverState_t;
    pub static mut svs: serverStatic_t;
    pub static mut gvm: *mut c_void;
    pub static mut com_developer: *mut cvar_s;
    pub static mut com_timescale: *mut cvar_s;
    pub static ICARUS_entFilter: c_int;
}

// Constants and type definitions (from headers)
const MAX_GENTITIES: usize = 4096;
const TID_CHAN_VOICE: c_int = 0;
const TK_FLOAT: c_int = 1;
const TK_INT: c_int = 2;
const TK_VECTOR: c_int = 3;
const TK_STRING: c_int = 4;
const TK_IDENTIFIER: c_int = 5;
const TK_EQUALS: c_int = 6;
const TK_GREATER_THAN: c_int = 7;
const TK_LESS_THAN: c_int = 8;
const TK_NOT: c_int = 9;
const WL_ERROR: c_int = 0;
const WL_WARNING: c_int = 1;
const WL_DEBUG: c_int = 2;
const WL_VERBOSE: c_int = 3;
const VTYPE_NONE: c_int = 0;
const VTYPE_FLOAT: c_int = 1;
const VTYPE_STRING: c_int = 2;
const VTYPE_VECTOR: c_int = 3;
const GAME_ICARUS_PLAYSOUND: c_int = 1;
const GAME_ICARUS_SET: c_int = 2;
const GAME_ICARUS_LERP2POS: c_int = 3;
const GAME_ICARUS_LERP2ORIGIN: c_int = 4;
const GAME_ICARUS_LERP2ANGLES: c_int = 5;
const GAME_ICARUS_GETTAG: c_int = 6;
const GAME_ICARUS_LERP2START: c_int = 7;
const GAME_ICARUS_LERP2END: c_int = 8;
const GAME_ICARUS_USE: c_int = 9;
const GAME_ICARUS_KILL: c_int = 10;
const GAME_ICARUS_REMOVE: c_int = 11;
const GAME_ICARUS_PLAY: c_int = 12;
const GAME_ICARUS_GETFLOAT: c_int = 13;
const GAME_ICARUS_GETVECTOR: c_int = 14;
const GAME_ICARUS_GETSTRING: c_int = 15;
const S_COLOR_RED: &str = "^1";
const S_COLOR_YELLOW: &str = "^3";
const S_COLOR_BLUE: &str = "^4";
const S_COLOR_GREEN: &str = "^2";

// Forward declarations of types
#[repr(C)]
pub struct sharedEntity_t {
    // Placeholder - actual structure would be defined elsewhere
    pub s: entityState_t,
    pub script_targetname: [c_char; 256],
    pub taskID: [c_int; 10], // NUM_TIDS approximate
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    // Other fields...
}

#[repr(C)]
pub struct CSequencer {
    // Placeholder
}

#[repr(C)]
pub struct CTaskManager {
    // Placeholder
}

#[repr(C)]
pub struct cvar_s {
    pub value: f32,
    pub integer: c_int,
    // Other fields...
}

#[repr(C)]
pub struct serverState_t {
    pub mSharedMemory: *mut c_void,
    // Other fields...
}

#[repr(C)]
pub struct serverStatic_t {
    pub time: c_int,
    // Other fields...
}

#[repr(C)]
pub struct entlist_t {
    // Placeholder - this is actually a map/hash in C++
}

#[repr(C)]
pub struct interface_export_t {
    pub I_LoadFile: Option<extern "C" fn(*const c_char, *mut *mut c_void) -> c_int>,
    pub I_CenterPrint: Option<extern "C" fn(*const c_char, ...)>,
    pub I_DPrintf: Option<extern "C" fn(c_int, *const c_char, ...)>,
    pub I_GetEntityByName: Option<extern "C" fn(*const c_char) -> *mut sharedEntity_t>,
    pub I_GetTime: Option<extern "C" fn() -> u32>,
    pub I_GetTimeScale: Option<extern "C" fn() -> u32>,
    pub I_PlaySound: Option<extern "C" fn(c_int, c_int, *const c_char, *const c_char) -> c_int>,
    pub I_Lerp2Pos: Option<extern "C" fn(c_int, c_int, *mut [f32; 3], *mut [f32; 3], f32)>,
    pub I_Lerp2Origin: Option<extern "C" fn(c_int, c_int, *mut [f32; 3], f32)>,
    pub I_Lerp2Angles: Option<extern "C" fn(c_int, c_int, *mut [f32; 3], f32)>,
    pub I_GetTag: Option<extern "C" fn(c_int, *const c_char, c_int, *mut [f32; 3]) -> c_int>,
    pub I_Lerp2Start: Option<extern "C" fn(c_int, c_int, f32)>,
    pub I_Lerp2End: Option<extern "C" fn(c_int, c_int, f32)>,
    pub I_Use: Option<extern "C" fn(c_int, *const c_char)>,
    pub I_Kill: Option<extern "C" fn(c_int, *const c_char)>,
    pub I_Remove: Option<extern "C" fn(c_int, *const c_char)>,
    pub I_Set: Option<extern "C" fn(c_int, c_int, *const c_char, *const c_char)>,
    pub I_Random: Option<extern "C" fn(f32, f32) -> f32>,
    pub I_Play: Option<extern "C" fn(c_int, c_int, *const c_char, *const c_char)>,
    pub I_CameraEnable: Option<extern "C" fn()>,
    pub I_CameraDisable: Option<extern "C" fn()>,
    pub I_CameraZoom: Option<extern "C" fn(f32, f32)>,
    pub I_CameraMove: Option<extern "C" fn(*mut [f32; 3], f32)>,
    pub I_CameraPan: Option<extern "C" fn(*mut [f32; 3], *mut [f32; 3], f32)>,
    pub I_CameraRoll: Option<extern "C" fn(f32, f32)>,
    pub I_CameraTrack: Option<extern "C" fn(*const c_char, f32, f32)>,
    pub I_CameraFollow: Option<extern "C" fn(*const c_char, f32, f32)>,
    pub I_CameraDistance: Option<extern "C" fn(f32, f32)>,
    pub I_CameraShake: Option<extern "C" fn(f32, c_int)>,
    pub I_CameraFade: Option<extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, f32)>,
    pub I_CameraPath: Option<extern "C" fn(*const c_char)>,
    pub I_GetFloat: Option<extern "C" fn(c_int, c_int, *const c_char, *mut f32) -> c_int>,
    pub I_GetVector: Option<extern "C" fn(c_int, c_int, *const c_char, *mut [f32; 3]) -> c_int>,
    pub I_GetString: Option<extern "C" fn(c_int, c_int, *const c_char, *mut *mut c_char) -> c_int>,
    pub I_Evaluate: Option<extern "C" fn(c_int, *const c_char, c_int, *const c_char, c_int) -> c_int>,
    pub I_DeclareVariable: Option<extern "C" fn(*const c_char, c_int) -> c_int>,
    pub I_FreeVariable: Option<extern "C" fn(*const c_char)>,
    pub I_WriteSaveData: Option<extern "C" fn(c_int, *mut c_void, c_int) -> c_int>,
    pub I_ReadSaveData: Option<extern "C" fn(c_int, *mut c_void, c_int) -> c_int>,
    pub I_LinkEntity: Option<extern "C" fn(c_int, *mut CSequencer, *mut CTaskManager) -> c_int>,
}

// Shared memory structures for VM interface
#[repr(C)]
pub struct T_G_ICARUS_PLAYSOUND {
    pub taskID: c_int,
    pub entID: c_int,
    pub name: [c_char; 256],
    pub channel: [c_char; 256],
}

#[repr(C)]
pub struct T_G_ICARUS_SET {
    pub taskID: c_int,
    pub entID: c_int,
    pub type_name: [c_char; 256],
    pub data: [c_char; 256],
}

#[repr(C)]
pub struct T_G_ICARUS_LERP2POS {
    pub taskID: c_int,
    pub entID: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub nullAngles: c_int,
    pub duration: f32,
}

#[repr(C)]
pub struct T_G_ICARUS_LERP2ORIGIN {
    pub taskID: c_int,
    pub entID: c_int,
    pub origin: [f32; 3],
    pub duration: f32,
}

#[repr(C)]
pub struct T_G_ICARUS_LERP2ANGLES {
    pub taskID: c_int,
    pub entID: c_int,
    pub angles: [f32; 3],
    pub duration: f32,
}

#[repr(C)]
pub struct T_G_ICARUS_GETTAG {
    pub entID: c_int,
    pub name: [c_char; 256],
    pub lookup: c_int,
    pub info: [f32; 3],
}

#[repr(C)]
pub struct T_G_ICARUS_LERP2START {
    pub taskID: c_int,
    pub entID: c_int,
    pub duration: f32,
}

#[repr(C)]
pub struct T_G_ICARUS_LERP2END {
    pub taskID: c_int,
    pub entID: c_int,
    pub duration: f32,
}

#[repr(C)]
pub struct T_G_ICARUS_USE {
    pub entID: c_int,
    pub target: [c_char; 256],
}

#[repr(C)]
pub struct T_G_ICARUS_KILL {
    pub entID: c_int,
    pub name: [c_char; 256],
}

#[repr(C)]
pub struct T_G_ICARUS_REMOVE {
    pub entID: c_int,
    pub name: [c_char; 256],
}

#[repr(C)]
pub struct T_G_ICARUS_PLAY {
    pub taskID: c_int,
    pub entID: c_int,
    pub r#type: [c_char; 256],
    pub name: [c_char; 256],
}

#[repr(C)]
pub struct T_G_ICARUS_GETFLOAT {
    pub entID: c_int,
    pub r#type: c_int,
    pub name: [c_char; 256],
    pub value: f32,
}

#[repr(C)]
pub struct T_G_ICARUS_GETVECTOR {
    pub entID: c_int,
    pub r#type: c_int,
    pub name: [c_char; 256],
    pub value: [f32; 3],
}

#[repr(C)]
pub struct T_G_ICARUS_GETSTRING {
    pub entID: c_int,
    pub r#type: c_int,
    pub name: [c_char; 256],
    pub value: [c_char; 256],
}

pub static mut interface_export: interface_export_t = interface_export_t {
    I_LoadFile: None,
    I_CenterPrint: None,
    I_DPrintf: None,
    I_GetEntityByName: None,
    I_GetTime: None,
    I_GetTimeScale: None,
    I_PlaySound: None,
    I_Lerp2Pos: None,
    I_Lerp2Origin: None,
    I_Lerp2Angles: None,
    I_GetTag: None,
    I_Lerp2Start: None,
    I_Lerp2End: None,
    I_Use: None,
    I_Kill: None,
    I_Remove: None,
    I_Set: None,
    I_Random: None,
    I_Play: None,
    I_CameraEnable: None,
    I_CameraDisable: None,
    I_CameraZoom: None,
    I_CameraMove: None,
    I_CameraPan: None,
    I_CameraRoll: None,
    I_CameraTrack: None,
    I_CameraFollow: None,
    I_CameraDistance: None,
    I_CameraShake: None,
    I_CameraFade: None,
    I_CameraPath: None,
    I_GetFloat: None,
    I_GetVector: None,
    I_GetString: None,
    I_Evaluate: None,
    I_DeclareVariable: None,
    I_FreeVariable: None,
    I_WriteSaveData: None,
    I_ReadSaveData: None,
    I_LinkEntity: None,
};

// Forward declarations for internal stubs
extern "C" {
    fn Q3_VariableDeclared(name: *const c_char) -> c_int;
    fn Q3_GetFloatVariable(name: *const c_char, val: *mut f32) -> c_int;
    fn Q3_SetFloatVariable(name: *const c_char, val: f32);
    fn Q3_SetStringVariable(name: *const c_char, val: *const c_char);
    fn Q3_SetVectorVariable(name: *const c_char, val: *const c_char);
    fn Q3_DeclareVariable(type_name: *const c_char, var_type: c_int) -> c_int;
    fn Q3_FreeVariable(name: *const c_char);
}

/*
============
Q3_ReadScript
  Description	: Reads in a file and attaches the script directory properly
  Return type	: static int
  Argument		: const char *name
  Argument		: void **buf
============
*/
fn Q3_ReadScript(name: *const c_char, buf: *mut *mut c_void) -> c_int {
    unsafe {
        let formatted = va(b"scripts/%s\0" as *const u8 as *const c_char, name);
        ICARUS_GetScript(formatted, buf as *mut *mut c_char)	//get a (hopefully) cached file
    }
}

/*
============
Q3_CenterPrint
  Description	: Prints a message in the center of the screen
  Return type	: static void
  Argument		:  const char *format
  Argument		: ...
============
*/
// Note: In Rust, implementing variadic functions requires platform-specific va_list handling.
// This is a stub that preserves the C interface but would need unsafe FFI details to implement fully.
fn Q3_CenterPrint(format: *const c_char) {
    unsafe {
        let mut text: [c_char; 1024] = [0; 1024];

        // FIXME: added '!' so you can print something that's hasn't been precached, '@' searches only for precache text
        // this is just a TEMPORARY placeholder until objectives are in!!!  -- dmv 11/26/01

        if !text.is_empty() && ((text[0] as u8 == b'@') || text[0] as u8 == b'!')	// It's a key
        {
            if text[0] as u8 == b'!'
            {
                SV_SendServerCommand(std::ptr::null_mut(), b"cp \"%s\"\0" as *const u8 as *const c_char, text.as_ptr().add(1));
                return;
            }

            SV_SendServerCommand(std::ptr::null_mut(), b"cp \"%s\"\0" as *const u8 as *const c_char, text.as_ptr());
        }

        // Just a developers note
        return;
    }
}


/*
-------------------------
void Q3_ClearTaskID( int *taskID )

WARNING: Clearing a taskID will make that task never finish unless you intend to
			return the same taskID from somewhere else.
-------------------------
*/
#[cfg(not(target_os = "xbox"))]
pub fn Q3_TaskIDClear(taskID: *mut c_int) {
    unsafe {
        *taskID = -1;
    }
}

/*
-------------------------
qboolean Q3_TaskIDPending( sharedEntity_t *ent, taskID_t taskType )
-------------------------
*/
pub fn Q3_TaskIDPending(ent: *mut sharedEntity_t, taskType: c_int) -> c_int {
    unsafe {
        if ent.is_null() {
            return 0; // qfalse
        }

        let ent_ref = &*ent;

        if taskType < TID_CHAN_VOICE || taskType >= 10 // NUM_TIDS
        {
            return 0; // qfalse
        }

        if taskType < 0 || taskType >= ent_ref.taskID.len() as c_int {
            return 0; // qfalse
        }

        if ent_ref.taskID[taskType as usize] >= 0 {
            ///-1 is none
            return 1; // qtrue
        }

        0 // qfalse
    }
}

/*
-------------------------
void Q3_TaskIDComplete( sharedEntity_t *ent, taskID_t taskType )
-------------------------
*/
pub fn Q3_TaskIDComplete(ent: *mut sharedEntity_t, taskType: c_int) {
    unsafe {
        if ent.is_null() {
            return;
        }

        if taskType < TID_CHAN_VOICE || taskType >= 10 // NUM_TIDS
        {
            return;
        }

        let ent_ref = &mut *ent;

        if taskType < 0 || taskType >= ent_ref.taskID.len() as c_int {
            return;
        }

        if Q3_TaskIDPending(ent, taskType) != 0 {
            //Complete it
            // gTaskManagers[ent->s.number]->Completed( ent->taskID[taskType] );

            //See if any other tasks have the name number and clear them so we don't complete more than once
            let clearTask = ent_ref.taskID[taskType as usize];
            for tid in 0..ent_ref.taskID.len() {
                if ent_ref.taskID[tid] == clearTask {
                    Q3_TaskIDClear(&mut ent_ref.taskID[tid]);
                }
            }

            //clear it - should be cleared in for loop above
            //Q3_TaskIDClear( &ent->taskID[taskType] );
        }
        //otherwise, wasn't waiting for a task to complete anyway
    }
}

/*
-------------------------
void Q3_SetTaskID( sharedEntity_t *ent, taskID_t taskType, int taskID )
-------------------------
*/

pub fn Q3_TaskIDSet(ent: *mut sharedEntity_t, taskType: c_int, taskID: c_int) {
    unsafe {
        if ent.is_null() {
            return;
        }

        if taskType < TID_CHAN_VOICE || taskType >= 10 // NUM_TIDS
        {
            return;
        }

        //Might be stomping an old task, so complete and clear previous task if there was one
        Q3_TaskIDComplete(ent, taskType);

        let ent_ref = &mut *ent;
        if taskType >= 0 && taskType < ent_ref.taskID.len() as c_int {
            ent_ref.taskID[taskType as usize] = taskID;
        }
    }
}


/*
============
Q3_CheckStringCounterIncrement
  Description	:
  Return type	: static float
  Argument		: const char *string
============
*/
fn Q3_CheckStringCounterIncrement(string: *const c_char) -> f32 {
    unsafe {
        let mut numString: *mut c_char = std::ptr::null_mut();
        let mut val: f32 = 0.0;

        if *(string as *const u8) as char == '+' {
            //We want to increment whatever the value is by whatever follows the +
            if *(string.add(1) as *const u8) as char != '\0' {
                numString = string.add(1) as *mut c_char;
                val = atof(numString) as f32;
            }
        } else if *(string as *const u8) as char == '-' {
            //we want to decrement
            if *(string.add(1) as *const u8) as char != '\0' {
                numString = string.add(1) as *mut c_char;
                val = (atof(numString) as f32) * -1.0;
            }
        }

        val
    }
}

/*
=============
Q3_GetEntityByName

Returns the sequencer of the entity by the given name
=============
*/
fn Q3_GetEntityByName(name: *const c_char) -> *mut sharedEntity_t {
    unsafe {
        let mut ent: *mut sharedEntity_t;
        let mut temp: [c_char; 1024] = [0; 1024];

        if name.is_null() || *(name as *const u8) as char == '\0' {
            return std::ptr::null_mut();
        }

        strncpy(temp.as_mut_ptr(), name, std::mem::size_of_val(&temp));
        *(temp.as_mut_ptr().add(std::mem::size_of_val(&temp) - 1)) = 0;

        // ei = ICARUS_EntList.find( Q_strupr( (char *) temp ) );
        //
        // if ( ei == ICARUS_EntList.end() )
        //     return NULL;
        //
        // ent = SV_GentityNum((*ei).second);

        // return ent;
        // this now returns the ent instead of the sequencer -- dmv 06/27/01
        //	if (ent == NULL)
        //		return NULL;
        //	return gSequencers[ent->s.number];

        std::ptr::null_mut()
    }
}

/*
=============
Q3_GetTime

Get the current game time
=============
*/
fn Q3_GetTime() -> u32 {
    unsafe {
        svs.time as u32
    }
}

/*
=============
G_AddSexToMunroString

Take any string, look for "kyle/" replace with "kyla/" based on "sex"
And: Take any string, look for "/mr_" replace with "/ms_" based on "sex"
returns qtrue if changed to ms
=============
*/
/*
static qboolean G_AddSexToMunroString ( char *string, qboolean qDoBoth )
{
    char *start;

    if VALIDSTRING( string ) {
        if ( g_sex->string[0] == 'f' ) {
            start = strstr( string, "kyle/" );
            if ( start != NULL ) {
                strncpy( start, "kyla", 5 );
                return qtrue;
            } else {
                start = strrchr( string, '/' );		//get the last slash before the wav
                if (start != NULL) {
                    if (!strncmp( start, "/mr_", 4) ) {
                        if (qDoBoth) {	//we want to change mr to ms
                            start[2] = 's';	//change mr to ms
                            return qtrue;
                        } else {	//IF qDoBoth
                            return qfalse;	//don't want this one
                        }
                    }
                }	//IF found slash
            }
        }	//IF Female
        else {	//i'm male
            start = strrchr( string, '/' );		//get the last slash before the wav
            if (start != NULL) {
                if (!strncmp( start, "/ms_", 4) ) {
                    return qfalse;	//don't want this one
                }
            }	//IF found slash
        }
    }	//if VALIDSTRING
    return qtrue;
}
*/

/*
=============
Q3_PlaySound

Plays a sound from an entity
=============
*/
fn Q3_PlaySound(taskID: c_int, entID: c_int, name: *const c_char, channel: *const c_char) -> c_int {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_PLAYSOUND;

        (*sharedMem).taskID = taskID;
        (*sharedMem).entID = entID;
        strcpy((*sharedMem).name.as_mut_ptr(), name);
        strcpy((*sharedMem).channel.as_mut_ptr(), channel);

        VM_Call(gvm, GAME_ICARUS_PLAYSOUND)
    }
}


/*
============
Q3_SetVar
  Description	:
  Return type	: static void
  Argument		:  int taskID
  Argument		: int entID
  Argument		: const char *type_name
  Argument		: const char *data
============
*/
pub fn Q3_SetVar(taskID: c_int, entID: c_int, type_name: *const c_char, data: *const c_char) {
    unsafe {
        let vret = Q3_VariableDeclared(type_name);
        let mut float_data: f32;
        let mut val: f32 = 0.0;

        if vret != VTYPE_NONE {
            match vret {
                VTYPE_FLOAT => {
                    //Check to see if increment command
                    if (val = Q3_CheckStringCounterIncrement(data)) != 0.0 {
                        Q3_GetFloatVariable(type_name, &mut float_data);
                        float_data += val;
                    } else {
                        float_data = atof(data) as f32;
                    }
                    Q3_SetFloatVariable(type_name, float_data);
                }

                VTYPE_STRING => {
                    Q3_SetStringVariable(type_name, data);
                }

                VTYPE_VECTOR => {
                    Q3_SetVectorVariable(type_name, data);
                }

                _ => {}
            }

            return;
        }

        Q3_DebugPrint(WL_ERROR, b"%s variable or field not found!\n\0" as *const u8 as *const c_char, type_name);
    }
}

/*
============
Q3_Set
  Description	:
  Return type	: void
  Argument		:  int taskID
  Argument		: int entID
  Argument		: const char *type_name
  Argument		: const char *data
============
*/
fn Q3_Set(taskID: c_int, entID: c_int, type_name: *const c_char, data: *const c_char) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_SET;

        (*sharedMem).taskID = taskID;
        (*sharedMem).entID = entID;
        strcpy((*sharedMem).type_name.as_mut_ptr(), type_name);
        strcpy((*sharedMem).data.as_mut_ptr(), data);

        if VM_Call(gvm, GAME_ICARUS_SET) != 0 {
            // gTaskManagers[entID]->Completed( taskID );
        }
    }
}


/*
============
Q3_Evaluate
  Description	:
  Return type	: int
  Argument		:  int p1Type
  Argument		: const char *p1
  Argument		: int p2Type
  Argument		: const char *p2
  Argument		: int operatorType
============
*/
fn Q3_Evaluate(p1Type: c_int, p1: *const c_char, p2Type: c_int, p2: *const c_char, operatorType: c_int) -> c_int {
    unsafe {
        let mut f1: f32 = 0.0;
        let mut f2: f32 = 0.0;
        let mut v1: [f32; 3] = [0.0; 3];
        let mut v2: [f32; 3] = [0.0; 3];
        let mut c1: *const c_char = std::ptr::null();
        let mut c2: *const c_char = std::ptr::null();
        let mut i1: c_int = 0;
        let mut i2: c_int = 0;
        let mut p1Type = p1Type;
        let mut p2Type = p2Type;

        //Always demote to int on float to integer comparisons
        if (p1Type == TK_FLOAT && p2Type == TK_INT) || (p1Type == TK_INT && p2Type == TK_FLOAT) {
            p1Type = TK_INT;
            p2Type = TK_INT;
        }

        //Cannot compare two disimilar types
        if p1Type != p2Type {
            Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate comparing two disimilar types!\n\0" as *const u8 as *const c_char);
            return 0; // false
        }

        //Format the parameters
        match p1Type {
            TK_FLOAT => {
                sscanf(p1, b"%f\0" as *const u8 as *const c_char, &mut f1);
                sscanf(p2, b"%f\0" as *const u8 as *const c_char, &mut f2);
            }

            TK_INT => {
                sscanf(p1, b"%d\0" as *const u8 as *const c_char, &mut i1);
                sscanf(p2, b"%d\0" as *const u8 as *const c_char, &mut i2);
            }

            TK_VECTOR => {
                sscanf(p1, b"%f %f %f\0" as *const u8 as *const c_char, &mut v1[0], &mut v1[1], &mut v1[2]);
                sscanf(p2, b"%f %f %f\0" as *const u8 as *const c_char, &mut v2[0], &mut v2[1], &mut v2[2]);
            }

            TK_STRING | TK_IDENTIFIER => {
                c1 = p1;
                c2 = p2;
            }

            _ => {
                Q3_DebugPrint(WL_WARNING, b"Q3_Evaluate unknown type used!\n\0" as *const u8 as *const c_char);
                return 0; // false
            }
        }

        //Compare them and return the result

        //FIXME: YUCK!!!  Better way to do this?

        match operatorType {
            //
            //	EQUAL TO
            //

            TK_EQUALS => {
                match p1Type {
                    TK_FLOAT => {
                        return if f1 == f2 { 1 } else { 0 };
                    }

                    TK_INT => {
                        return if i1 == i2 { 1 } else { 0 };
                    }

                    TK_VECTOR => {
                        return VectorCompare(&v1, &v2);
                    }

                    TK_STRING | TK_IDENTIFIER => {
                        return if stricmp(c1, c2) == 0 { 1 } else { 0 };	//NOTENOTE: The script uses proper string comparison logic (ex. ( a == a ) == true )
                    }

                    _ => {
                        Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate unknown type used!\n\0" as *const u8 as *const c_char);
                        return 0; // false
                    }
                }
            }

            //
            //	GREATER THAN
            //

            TK_GREATER_THAN => {
                match p1Type {
                    TK_FLOAT => {
                        return if f1 > f2 { 1 } else { 0 };
                    }

                    TK_INT => {
                        return if i1 > i2 { 1 } else { 0 };
                    }

                    TK_VECTOR => {
                        Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate vector comparisons of type GREATER THAN cannot be performed!\0" as *const u8 as *const c_char);
                        return 0; // false
                    }

                    TK_STRING | TK_IDENTIFIER => {
                        Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate string comparisons of type GREATER THAN cannot be performed!\0" as *const u8 as *const c_char);
                        return 0; // false
                    }

                    _ => {
                        Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate unknown type used!\n\0" as *const u8 as *const c_char);
                        return 0; // false
                    }
                }
            }

            //
            //	LESS THAN
            //

            TK_LESS_THAN => {
                match p1Type {
                    TK_FLOAT => {
                        return if f1 < f2 { 1 } else { 0 };
                    }

                    TK_INT => {
                        return if i1 < i2 { 1 } else { 0 };
                    }

                    TK_VECTOR => {
                        Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate vector comparisons of type LESS THAN cannot be performed!\0" as *const u8 as *const c_char);
                        return 0; // false
                    }

                    TK_STRING | TK_IDENTIFIER => {
                        Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate string comparisons of type LESS THAN cannot be performed!\0" as *const u8 as *const c_char);
                        return 0; // false
                    }

                    _ => {
                        Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate unknown type used!\n\0" as *const u8 as *const c_char);
                        return 0; // false
                    }
                }
            }

            //
            //	NOT
            //

            TK_NOT => {	//NOTENOTE: Implied "NOT EQUAL TO"
                match p1Type {
                    TK_FLOAT => {
                        return if f1 != f2 { 1 } else { 0 };
                    }

                    TK_INT => {
                        return if i1 != i2 { 1 } else { 0 };
                    }

                    TK_VECTOR => {
                        return if VectorCompare(&v1, &v2) == 0 { 1 } else { 0 };
                    }

                    TK_STRING | TK_IDENTIFIER => {
                        return stricmp(c1, c2);
                    }

                    _ => {
                        Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate unknown type used!\n\0" as *const u8 as *const c_char);
                        return 0; // false
                    }
                }
            }

            _ => {
                Q3_DebugPrint(WL_ERROR, b"Q3_Evaluate unknown operator used!\n\0" as *const u8 as *const c_char);
            }
        }

        0 // false
    }
}

/*
-------------------------
Q3_CameraFade
-------------------------
*/
fn Q3_CameraFade(sr: f32, sg: f32, sb: f32, sa: f32, dr: f32, dg: f32, db: f32, da: f32, duration: f32) {
    Q3_DebugPrint(WL_WARNING, b"Q3_CameraFade: NOT SUPPORTED IN MP\n\0" as *const u8 as *const c_char);
}

/*
-------------------------
Q3_CameraPath
-------------------------
*/
fn Q3_CameraPath(name: *const c_char) {
    Q3_DebugPrint(WL_WARNING, b"Q3_CameraPath: NOT SUPPORTED IN MP\n\0" as *const u8 as *const c_char);
}

/*
-------------------------
Q3_DebugPrint
-------------------------
*/
// Note: In Rust, implementing variadic functions requires platform-specific va_list handling.
// This is a stub that preserves the C interface but would need unsafe FFI details to implement fully.
pub fn Q3_DebugPrint(level: c_int, format: *const c_char) {
    unsafe {
        //Don't print messages they don't want to see
        //if ( g_ICARUSDebug->integer < level )
        if com_developer.is_null() || (*com_developer).integer == 0 {
            return;
        }

        //Add the color formatting
        match level {
            WL_ERROR => {
                Com_Printf(b"%sERROR\0" as *const u8 as *const c_char, S_COLOR_RED.as_ptr());
            }

            WL_WARNING => {
                Com_Printf(b"%sWARNING\0" as *const u8 as *const c_char, S_COLOR_YELLOW.as_ptr());
            }

            WL_DEBUG => {
                Com_Printf(b"%sDEBUG\0" as *const u8 as *const c_char, S_COLOR_BLUE.as_ptr());
            }

            _ => { // WL_VERBOSE and default
                Com_Printf(b"%sINFO\0" as *const u8 as *const c_char, S_COLOR_GREEN.as_ptr());
            }
        }
    }
}

fn CGCam_Anything() {
    Q3_DebugPrint(WL_WARNING, b"Camera functions NOT SUPPORTED IN MP\n\0" as *const u8 as *const c_char);
}

//These are useless for MP. Just taking it for now since I don't want to remove all calls to this in ICARUS.
pub fn AppendToSaveGame(chid: c_int, data: *mut c_void, length: c_int) -> c_int {
    1
}

// Changed by BTO (VV) - Visual C++ 7.1 doesn't allow default args on funcion pointers
pub fn ReadFromSaveGame(chid: c_int, pvAddress: *mut c_void, iLength: c_int) -> c_int {
    1
}

pub fn CGCam_Enable() {
    CGCam_Anything();
}

pub fn CGCam_Disable() {
    CGCam_Anything();
}

pub fn CGCam_Zoom(FOV: f32, duration: f32) {
    CGCam_Anything();
}

pub fn CGCam_Pan(dest: *mut [f32; 3], panDirection: *mut [f32; 3], duration: f32) {
    CGCam_Anything();
}

pub fn CGCam_Move(dest: *mut [f32; 3], duration: f32) {
    CGCam_Anything();
}

#[cfg(not(target_os = "xbox"))]
pub fn CGCam_Shake(intensity: f32, duration: c_int) {
    CGCam_Anything();
}

pub fn CGCam_Follow(cameraGroup: *const c_char, speed: f32, initLerp: f32) {
    CGCam_Anything();
}

pub fn CGCam_Track(trackName: *const c_char, speed: f32, initLerp: f32) {
    CGCam_Anything();
}

pub fn CGCam_Distance(distance: f32, initLerp: f32) {
    CGCam_Anything();
}

pub fn CGCam_Roll(dest: f32, duration: f32) {
    CGCam_Anything();
}

fn Q3_GetTimeScale() -> u32 {
    unsafe {
        com_timescale.as_ref().map(|cv| cv.value as u32).unwrap_or(1)
    }
}

fn Q3_Lerp2Pos(taskID: c_int, entID: c_int, origin: *mut [f32; 3], angles: *mut [f32; 3], duration: f32) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_LERP2POS;

        (*sharedMem).taskID = taskID;
        (*sharedMem).entID = entID;
        VectorCopy(origin, &mut (*sharedMem).origin);

        if !angles.is_null() {
            VectorCopy(angles, &mut (*sharedMem).angles);
            (*sharedMem).nullAngles = 0; // qfalse
        } else {
            (*sharedMem).nullAngles = 1; // qtrue
        }
        (*sharedMem).duration = duration;

        VM_Call(gvm, GAME_ICARUS_LERP2POS);
        //We do this in case the values are modified in the game. It would be expected by icarus that
        //the values passed in here are modified equally.
        VectorCopy(&(*sharedMem).origin, origin);

        if !angles.is_null() {
            VectorCopy(&(*sharedMem).angles, angles);
        }
    }
}

fn Q3_Lerp2Origin(taskID: c_int, entID: c_int, origin: *mut [f32; 3], duration: f32) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_LERP2ORIGIN;

        (*sharedMem).taskID = taskID;
        (*sharedMem).entID = entID;
        VectorCopy(origin, &mut (*sharedMem).origin);
        (*sharedMem).duration = duration;

        VM_Call(gvm, GAME_ICARUS_LERP2ORIGIN);
        VectorCopy(&(*sharedMem).origin, origin);
    }
}

fn Q3_Lerp2Angles(taskID: c_int, entID: c_int, angles: *mut [f32; 3], duration: f32) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_LERP2ANGLES;

        (*sharedMem).taskID = taskID;
        (*sharedMem).entID = entID;
        VectorCopy(angles, &mut (*sharedMem).angles);
        (*sharedMem).duration = duration;

        VM_Call(gvm, GAME_ICARUS_LERP2ANGLES);
        VectorCopy(&(*sharedMem).angles, angles);
    }
}

fn Q3_GetTag(entID: c_int, name: *const c_char, lookup: c_int, info: *mut [f32; 3]) -> c_int {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_GETTAG;

        (*sharedMem).entID = entID;
        strcpy((*sharedMem).name.as_mut_ptr(), name);
        (*sharedMem).lookup = lookup;
        VectorCopy(info, &mut (*sharedMem).info);

        let r = VM_Call(gvm, GAME_ICARUS_GETTAG);
        VectorCopy(&(*sharedMem).info, info);
        r
    }
}

fn Q3_Lerp2Start(entID: c_int, taskID: c_int, duration: f32) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_LERP2START;

        (*sharedMem).taskID = taskID;
        (*sharedMem).entID = entID;
        (*sharedMem).duration = duration;

        VM_Call(gvm, GAME_ICARUS_LERP2START);
    }
}

fn Q3_Lerp2End(entID: c_int, taskID: c_int, duration: f32) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_LERP2END;

        (*sharedMem).taskID = taskID;
        (*sharedMem).entID = entID;
        (*sharedMem).duration = duration;

        VM_Call(gvm, GAME_ICARUS_LERP2END);
    }
}

fn Q3_Use(entID: c_int, target: *const c_char) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_USE;

        (*sharedMem).entID = entID;
        strcpy((*sharedMem).target.as_mut_ptr(), target);

        VM_Call(gvm, GAME_ICARUS_USE);
    }
}

fn Q3_Kill(entID: c_int, name: *const c_char) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_KILL;

        (*sharedMem).entID = entID;
        strcpy((*sharedMem).name.as_mut_ptr(), name);

        VM_Call(gvm, GAME_ICARUS_KILL);
    }
}

fn Q3_Remove(entID: c_int, name: *const c_char) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_REMOVE;

        (*sharedMem).entID = entID;
        strcpy((*sharedMem).name.as_mut_ptr(), name);

        VM_Call(gvm, GAME_ICARUS_REMOVE);
    }
}

fn Q3_Play(taskID: c_int, entID: c_int, r#type: *const c_char, name: *const c_char) {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_PLAY;

        (*sharedMem).taskID = taskID;
        (*sharedMem).entID = entID;
        strcpy((*sharedMem).r#type.as_mut_ptr(), r#type);
        strcpy((*sharedMem).name.as_mut_ptr(), name);

        VM_Call(gvm, GAME_ICARUS_PLAY);
    }
}

fn Q3_GetFloat(entID: c_int, r#type: c_int, name: *const c_char, value: *mut f32) -> c_int {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_GETFLOAT;

        (*sharedMem).entID = entID;
        (*sharedMem).r#type = r#type;
        strcpy((*sharedMem).name.as_mut_ptr(), name);
        (*sharedMem).value = 0.0; //*value;

        let r = VM_Call(gvm, GAME_ICARUS_GETFLOAT);
        *value = (*sharedMem).value;
        r
    }
}

fn Q3_GetVector(entID: c_int, r#type: c_int, name: *const c_char, value: *mut [f32; 3]) -> c_int {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_GETVECTOR;

        (*sharedMem).entID = entID;
        (*sharedMem).r#type = r#type;
        strcpy((*sharedMem).name.as_mut_ptr(), name);
        VectorCopy(value, &mut (*sharedMem).value);

        let r = VM_Call(gvm, GAME_ICARUS_GETVECTOR);
        VectorCopy(&(*sharedMem).value, value);
        r
    }
}

fn Q3_GetString(entID: c_int, r#type: c_int, name: *const c_char, value: *mut *mut c_char) -> c_int {
    unsafe {
        let sharedMem = sv.mSharedMemory as *mut T_G_ICARUS_GETSTRING;

        (*sharedMem).entID = entID;
        (*sharedMem).r#type = r#type;
        strcpy((*sharedMem).name.as_mut_ptr(), name);

        let r = VM_Call(gvm, GAME_ICARUS_GETSTRING);
        //rww - careful with this, next time shared memory is altered this will get stomped
        *value = (*sharedMem).value.as_mut_ptr();
        r
    }
}


/*
============
Interface_Init
  Description	: Inits the interface for the game
  Return type	: void
  Argument		: interface_export_t *pe
============
*/
pub fn Interface_Init(pe: *mut interface_export_t) {
    unsafe {
        //TODO: This is where you link up all your functions to the engine

        //General
        (*pe).I_LoadFile				=	Some(Q3_ReadScript);
        (*pe).I_CenterPrint			=	None;	// Variadic function - requires platform-specific va_list handling
        (*pe).I_DPrintf				=	None;	// Variadic function - requires platform-specific va_list handling
        (*pe).I_GetEntityByName		=	Some(Q3_GetEntityByName);
        (*pe).I_GetTime				=	Some(Q3_GetTime);
        (*pe).I_GetTimeScale			=	Some(Q3_GetTimeScale);
        (*pe).I_PlaySound				=	Some(Q3_PlaySound);
        (*pe).I_Lerp2Pos				=	Some(Q3_Lerp2Pos);
        (*pe).I_Lerp2Origin			=	Some(Q3_Lerp2Origin);
        (*pe).I_Lerp2Angles			=	Some(Q3_Lerp2Angles);
        (*pe).I_GetTag				=	Some(Q3_GetTag);
        (*pe).I_Lerp2Start			=	Some(Q3_Lerp2Start);
        (*pe).I_Lerp2End				=	Some(Q3_Lerp2End);
        (*pe).I_Use					=	Some(Q3_Use);
        (*pe).I_Kill					=	Some(Q3_Kill);
        (*pe).I_Remove				=	Some(Q3_Remove);
        (*pe).I_Set					=	Some(Q3_Set);
        (*pe).I_Random				=	Some(Q_flrand);
        (*pe).I_Play					=	Some(Q3_Play);

        //Camera functions
        (*pe).I_CameraEnable			=	Some(CGCam_Enable);
        (*pe).I_CameraDisable			=	Some(CGCam_Disable);
        (*pe).I_CameraZoom			=	Some(CGCam_Zoom);
        (*pe).I_CameraMove			=	Some(CGCam_Move);
        (*pe).I_CameraPan				=	Some(CGCam_Pan);
        (*pe).I_CameraRoll			=	Some(CGCam_Roll);
        (*pe).I_CameraTrack			=	Some(CGCam_Track);
        (*pe).I_CameraFollow			=	Some(CGCam_Follow);
        (*pe).I_CameraDistance		=	Some(CGCam_Distance);
        (*pe).I_CameraShake			=	Some(CGCam_Shake);
        (*pe).I_CameraFade			=	Some(Q3_CameraFade);
        (*pe).I_CameraPath			=	Some(Q3_CameraPath);

        //Variable information
        (*pe).I_GetFloat				=	Some(Q3_GetFloat);
        (*pe).I_GetVector				=	Some(Q3_GetVector);
        (*pe).I_GetString				=	Some(Q3_GetString);

        (*pe).I_Evaluate				=	Some(Q3_Evaluate);

        (*pe).I_DeclareVariable		=	Some(Q3_DeclareVariable);
        (*pe).I_FreeVariable			=	Some(Q3_FreeVariable);

        //Save / Load functions
        (*pe).I_WriteSaveData			=	Some(AppendToSaveGame);
        (*pe).I_ReadSaveData			=	Some(ReadFromSaveGame);
        (*pe).I_LinkEntity			=	Some(ICARUS_LinkEntity);
    }
}
