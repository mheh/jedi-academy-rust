//! BotLib core syscall wrappers — `trap_BotLib*` setup/var/test, bot client/snapshot/
//! usercmd access, the `trap_PC_*` precompiler-token traps, and
//! `trap_Bot_UpdateWaypoints/CalculatePaths`.
//!
//! 1:1 with `refs/raven-jediacademy/codemp/game/g_syscalls.c`; thin `syscall!` thunks.

use core::ffi::{c_char, c_void};

use super::cstr;
use crate::codemp::game::q_shared_h::{pc_token_t, usercmd_t, vec3_t};
use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport::*;

/// `trap_BotAllocateClient` — reserve a bot client slot, returning its client number.
pub fn BotAllocateClient() -> i32 {
    unsafe { syscall!(G_BOT_ALLOCATE_CLIENT) as i32 }
}

/// `trap_BotFreeClient` — release the bot client slot `client_num`.
pub fn BotFreeClient(client_num: i32) {
    unsafe {
        syscall!(G_BOT_FREE_CLIENT, client_num);
    }
}

/// `trap_BotGetServerCommand` — pull the next pending server command for `client_num`
/// into `message`. Returns nonzero if a command was retrieved.
pub fn BotGetServerCommand(client_num: i32, message: &mut [c_char]) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_GET_CONSOLE_MESSAGE,
            client_num,
            message.as_mut_ptr(),
            message.len() as i32
        ) as i32
    }
}

/// `trap_BotGetSnapshotEntity` — entity number at snapshot slot `sequence` for `client_num`.
pub fn BotGetSnapshotEntity(client_num: i32, sequence: i32) -> i32 {
    unsafe { syscall!(BOTLIB_GET_SNAPSHOT_ENTITY, client_num, sequence) as i32 }
}

/// `trap_BotLibDefine` — add a global precompiler `#define` from `string`.
pub fn BotLibDefine(string: &str) -> i32 {
    let s = cstr(string);
    unsafe { syscall!(BOTLIB_PC_ADD_GLOBAL_DEFINE, s.as_ptr()) as i32 }
}

/// `trap_BotLibLoadMap` — load the AAS/bot data for `mapname`.
pub fn BotLibLoadMap(mapname: &str) -> i32 {
    let m = cstr(mapname);
    unsafe { syscall!(BOTLIB_LOAD_MAP, m.as_ptr()) as i32 }
}

/// `trap_BotLibSetup` — initialise the bot library.
pub fn BotLibSetup() -> i32 {
    unsafe { syscall!(BOTLIB_SETUP) as i32 }
}

/// `trap_BotLibShutdown` — tear down the bot library.
pub fn BotLibShutdown() -> i32 {
    unsafe { syscall!(BOTLIB_SHUTDOWN) as i32 }
}

/// `trap_BotLibStartFrame` — advance the bot library's clock to `time`.
pub fn BotLibStartFrame(time: f32) -> i32 {
    unsafe { syscall!(BOTLIB_START_FRAME, pass_float(time)) as i32 }
}

/// `trap_BotLibTest` — bot-library debug hook.
pub fn BotLibTest(parm0: i32, parm1: &mut [c_char], parm2: &vec3_t, parm3: &vec3_t) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_TEST,
            parm0,
            parm1.as_mut_ptr(),
            parm2.as_ptr(),
            parm3.as_ptr()
        ) as i32
    }
}

/// `trap_BotLibUpdateEntity` — push entity `ent`'s updated state (`bue`, an engine
/// `bot_updateentity_s`) into the bot library. The struct is opaque to the module.
pub fn BotLibUpdateEntity(ent: i32, bue: *mut c_void) -> i32 {
    unsafe { syscall!(BOTLIB_UPDATENTITY, ent, bue) as i32 }
}

/// `trap_BotLibVarGet` — read bot-library variable `var_name` into `value`.
pub fn BotLibVarGet(var_name: &str, value: &mut [c_char]) -> i32 {
    let name = cstr(var_name);
    unsafe {
        syscall!(
            BOTLIB_LIBVAR_GET,
            name.as_ptr(),
            value.as_mut_ptr(),
            value.len() as i32
        ) as i32
    }
}

/// `trap_BotLibVarSet` — set bot-library variable `var_name` to `value`.
pub fn BotLibVarSet(var_name: &str, value: &str) -> i32 {
    let name = cstr(var_name);
    let val = cstr(value);
    unsafe { syscall!(BOTLIB_LIBVAR_SET, name.as_ptr(), val.as_ptr()) as i32 }
}

/// `trap_BotUserCommand` — feed the bot at `client_num` the movement command `ucmd`.
pub fn BotUserCommand(client_num: i32, ucmd: *mut usercmd_t) {
    unsafe {
        syscall!(BOTLIB_USER_COMMAND, client_num, ucmd);
    }
}

/// `trap_Bot_CalculatePaths` — (re)compute the navigation paths; `rmg` flags RMG maps.
pub fn Bot_CalculatePaths(rmg: i32) {
    unsafe {
        syscall!(G_BOT_CALCULATEPATHS, rmg);
    }
}

/// `trap_Bot_UpdateWaypoints` — hand the engine `wpnum` waypoint objects (`wps`, an
/// array of `wpobject_t *`). `wpobject_t` lives in the not-yet-ported `ai_main.h`, so
/// the array is passed through opaquely as `void **`.
pub fn Bot_UpdateWaypoints(wpnum: i32, wps: *mut *mut c_void) {
    unsafe {
        syscall!(G_BOT_UPDATEWAYPOINTS, wpnum, wps);
    }
}

/// `trap_PC_FreeSource` — release the precompiler source `handle`.
pub fn PC_FreeSource(handle: i32) -> i32 {
    unsafe { syscall!(BOTLIB_PC_FREE_SOURCE, handle) as i32 }
}

/// `trap_PC_LoadSource` — open `filename` for precompiler tokenisation; returns a handle.
pub fn PC_LoadSource(filename: &str) -> i32 {
    let f = cstr(filename);
    unsafe { syscall!(BOTLIB_PC_LOAD_SOURCE, f.as_ptr()) as i32 }
}

/// `trap_PC_ReadToken` — read the next token from source `handle` into `pc_token`.
pub fn PC_ReadToken(handle: i32, pc_token: &mut pc_token_t) -> i32 {
    unsafe { syscall!(BOTLIB_PC_READ_TOKEN, handle, pc_token as *mut pc_token_t) as i32 }
}

/// `trap_PC_SourceFileAndLine` — current source `filename`/`line` for `handle`.
pub fn PC_SourceFileAndLine(handle: i32, filename: &mut [c_char], line: *mut i32) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_PC_SOURCE_FILE_AND_LINE,
            handle,
            filename.as_mut_ptr(),
            line
        ) as i32
    }
}
