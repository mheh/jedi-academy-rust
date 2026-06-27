//! Elementary-action (EA) trap wrappers — the bot's command/movement/input traps
//! (`trap_EA_*`, BOTLIB_EA_* trap numbers 400-425).
//!
//! 1:1 with `refs/raven-jediacademy/codemp/game/be_ea.h` (signatures) + `g_syscalls.c`
//! (the `Q_syscall(BOTLIB_EA_*, ...)` thunks); thin `syscall!` wrappers mirroring
//! `BotUserCommand` in `botlib_core.rs`.

use core::ffi::c_void;

use super::cstr;
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport::*;

/// `trap_EA_Say` — bot `client` says `str` to everyone.
pub fn EA_Say(client: i32, str: &str) {
    let s = cstr(str);
    unsafe {
        syscall!(BOTLIB_EA_SAY, client, s.as_ptr());
    }
}

/// `trap_EA_SayTeam` — bot `client` says `str` to its team.
pub fn EA_SayTeam(client: i32, str: &str) {
    let s = cstr(str);
    unsafe {
        syscall!(BOTLIB_EA_SAY_TEAM, client, s.as_ptr());
    }
}

/// `trap_EA_Command` — bot `client` issues console `command`.
pub fn EA_Command(client: i32, command: &str) {
    let c = cstr(command);
    unsafe {
        syscall!(BOTLIB_EA_COMMAND, client, c.as_ptr());
    }
}

/// `trap_EA_Action` — bot `client` performs elementary `action`.
pub fn EA_Action(client: i32, action: i32) {
    unsafe {
        syscall!(BOTLIB_EA_ACTION, client, action);
    }
}

/// `trap_EA_Gesture` — bot `client` gestures.
pub fn EA_Gesture(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_GESTURE, client);
    }
}

/// `trap_EA_Talk` — bot `client` enters talk mode.
pub fn EA_Talk(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_TALK, client);
    }
}

/// `trap_EA_Attack` — bot `client` attacks (primary fire).
pub fn EA_Attack(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_ATTACK, client);
    }
}

/// `trap_EA_Alt_Attack` — bot `client` alt-attacks (secondary fire).
pub fn EA_Alt_Attack(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_ALT_ATTACK, client);
    }
}

/// `trap_EA_ForcePower` — bot `client` uses a force power.
pub fn EA_ForcePower(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_FORCEPOWER, client);
    }
}

/// `trap_EA_Use` — bot `client` presses use.
pub fn EA_Use(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_USE, client);
    }
}

/// `trap_EA_Respawn` — bot `client` respawns.
pub fn EA_Respawn(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_RESPAWN, client);
    }
}

/// `trap_EA_Crouch` — bot `client` crouches.
pub fn EA_Crouch(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_CROUCH, client);
    }
}

/// `trap_EA_MoveUp` — bot `client` moves up.
pub fn EA_MoveUp(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_MOVE_UP, client);
    }
}

/// `trap_EA_MoveDown` — bot `client` moves down.
pub fn EA_MoveDown(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_MOVE_DOWN, client);
    }
}

/// `trap_EA_MoveForward` — bot `client` moves forward.
pub fn EA_MoveForward(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_MOVE_FORWARD, client);
    }
}

/// `trap_EA_MoveBack` — bot `client` moves backward.
pub fn EA_MoveBack(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_MOVE_BACK, client);
    }
}

/// `trap_EA_MoveLeft` — bot `client` strafes left.
pub fn EA_MoveLeft(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_MOVE_LEFT, client);
    }
}

/// `trap_EA_MoveRight` — bot `client` strafes right.
pub fn EA_MoveRight(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_MOVE_RIGHT, client);
    }
}

/// `trap_EA_SelectWeapon` — bot `client` selects `weapon`.
pub fn EA_SelectWeapon(client: i32, weapon: i32) {
    unsafe {
        syscall!(BOTLIB_EA_SELECT_WEAPON, client, weapon);
    }
}

/// `trap_EA_Jump` — bot `client` jumps.
pub fn EA_Jump(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_JUMP, client);
    }
}

/// `trap_EA_DelayedJump` — bot `client` queues a delayed jump.
pub fn EA_DelayedJump(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_DELAYED_JUMP, client);
    }
}

/// `trap_EA_Move` — bot `client` moves in `dir` at `speed`.
pub fn EA_Move(client: i32, dir: &vec3_t, speed: f32) {
    unsafe {
        syscall!(BOTLIB_EA_MOVE, client, dir.as_ptr(), pass_float(speed));
    }
}

/// `trap_EA_View` — bot `client` sets its view angles to `viewangles`.
pub fn EA_View(client: i32, viewangles: &vec3_t) {
    unsafe {
        syscall!(BOTLIB_EA_VIEW, client, viewangles.as_ptr());
    }
}

/// `trap_EA_EndRegular` — bot `client` ends the regular-input frame at `thinktime`.
pub fn EA_EndRegular(client: i32, thinktime: f32) {
    unsafe {
        syscall!(BOTLIB_EA_END_REGULAR, client, pass_float(thinktime));
    }
}

/// `trap_EA_GetInput` — read bot `client`'s accumulated `input` for `thinktime`.
///
/// `input` is a `bot_input_t *` in C (be_ea.h:48). `bot_input_t` is not yet ported,
/// so the buffer is passed through opaquely as `void *` (mirroring OpenJK's
/// `trap_EA_GetInput(int, float, void *)`).
pub fn EA_GetInput(client: i32, thinktime: f32, input: *mut c_void) {
    unsafe {
        syscall!(BOTLIB_EA_GET_INPUT, client, pass_float(thinktime), input);
    }
}

/// `trap_EA_ResetInput` — reset bot `client`'s accumulated input.
pub fn EA_ResetInput(client: i32) {
    unsafe {
        syscall!(BOTLIB_EA_RESET_INPUT, client);
    }
}
