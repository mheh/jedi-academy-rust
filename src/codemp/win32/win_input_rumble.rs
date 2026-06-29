/*
 * UNPUBLISHED -- Rights  reserved  under  the  copyright  laws  of the
 * United States.  Use  of a copyright notice is precautionary only and
 * does not imply publication or disclosure.
 *
 * THIS DOCUMENTATION CONTAINS CONFIDENTIAL AND PROPRIETARY INFORMATION
 * OF    VICARIOUS   VISIONS,  INC.    ANY  DUPLICATION,  MODIFICATION,
 * DISTRIBUTION, OR DISCLOSURE IS STRICTLY PROHIBITED WITHOUT THE PRIOR
 * EXPRESS WRITTEN PERMISSION OF VICARIOUS VISIONS, INC.
 */

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Extern C functions and types
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn IN_RumbleAdjust(controller: c_int, left: c_int, right: c_int) -> bool;
    fn IN_GetMainController() -> c_int;

    fn G_ActivePlayerNormal() -> bool;

    // External globals
    pub static cg: cg_t;
    pub static cl_paused: cvar_t;
}

// Forward declarations for external types
#[repr(C)]
pub struct cvar_t {
    pub value: f32,
    pub integer: c_int,
}

/// Stub for cg_t struct to allow access to cg.time.
#[repr(C)]
pub struct cg_t {
    pub _pad: [u8; 316],
    pub time: c_int,
}

#[repr(C)]
pub struct rumblestate_t {
    pub timeToStop: c_int,
    // Right motor speed on Xbox, action type on Gamecube
    pub arg1: c_int,
    // Left motor speed on Xbox, secondary action type on Gamecube
    pub arg2: c_int,
}

#[repr(C)]
pub struct rumblestate_special_t {
    pub code: c_int,
    pub arg1: c_int,
    pub arg2: c_int,
}

#[repr(C)]
pub struct rumblescript_t {
    pub nextStateAt: c_int,
    pub controller: c_int,
    pub currentState: c_int,
    pub usedStates: c_int,
    pub numStates: c_int,
    pub autoDelete: bool,
    pub states: *mut rumblestate_t,
}

#[repr(C)]
pub struct rumblestatus_t {
    pub changed: bool,
    pub killed: bool,
    pub paused: bool,
    pub timePaused: c_int,
}

const MAX_RUMBLE_STATES: usize = 10;
const MAX_RUMBLE_SCRIPTS: usize = 10;
const MAX_RUMBLE_CONTROLLERS: usize = 4;

// In rumblestate, highest speed for each side takes precidence
// Number of rumble states is fairly small, so a plain array will work fine
static mut rumbleStatus: [rumblestatus_t; MAX_RUMBLE_CONTROLLERS] = [
    rumblestatus_t {
        changed: false,
        killed: false,
        paused: false,
        timePaused: 0,
    }; MAX_RUMBLE_CONTROLLERS
];

static mut rumbleScripts: [rumblescript_t; MAX_RUMBLE_SCRIPTS] = [
    rumblescript_t {
        nextStateAt: 0,
        controller: 0,
        currentState: 0,
        usedStates: 0,
        numStates: 0,
        autoDelete: false,
        states: core::ptr::null_mut(),
    }; MAX_RUMBLE_SCRIPTS
];

static mut in_useRumble: *mut cvar_t = core::ptr::null_mut();

// Rumble command constants (from win_input.h)
const IN_CMD_GOTO_XTIMES: c_int = -5;
const IN_CMD_GOTO: c_int = -6;
const IN_CMD_DEC_ARG2: c_int = -7;
const IN_CMD_INC_ARG2: c_int = -8;
const IN_CMD_DEC_ARG1: c_int = -9;
const IN_CMD_INC_ARG1: c_int = -10;
// Xbox-specific commands
const IN_CMD_DEC_LEFT: c_int = -70;
const IN_CMD_DEC_RIGHT: c_int = -71;
const IN_CMD_INC_LEFT: c_int = -72;
const IN_CMD_INC_RIGHT: c_int = -73;

/***** FIXME Some functions that would be found in a client manager *****/
/***** BEGIN FILLER *****/

// Always return 0 because we have only one client (right now anyway)
fn ActiveClientNum() -> c_int {
    0
}

// The active controller will always be number 0 for now
fn ActiveController() -> c_int {
    0
}
/***** END FILLER *****/

pub fn IN_enableRumble() {
    if ActiveClientNum() == 0 {
        unsafe {
            Cvar_Set(b"in_useRumble\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);
        }
    } else {
        unsafe {
            Cvar_Set(b"in_useRumble2\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);
        }
    }
}

pub fn IN_disableRumble() {
    if ActiveClientNum() == 0 {
        unsafe {
            Cvar_Set(b"in_useRumble\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
        }
    } else {
        unsafe {
            Cvar_Set(b"in_useRumble2\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
        }
    }
}

pub fn IN_usingRumble() -> bool {
    if ActiveClientNum() == 0 {
        unsafe {
            Cvar_VariableIntegerValue(b"in_useRumble\0".as_ptr() as *const c_char) != 0
        }
    } else {
        unsafe {
            Cvar_VariableIntegerValue(b"in_useRumble2\0".as_ptr() as *const c_char) != 0
        }
    }
}

// Creates a rumble script with numStates
// Returns -1 on no more room, otherwise an identifier to use for scripts
pub fn IN_CreateRumbleScript(controller: c_int, numStates: c_int, deleteWhenFinished: bool) -> c_int {
    if !IN_usingRumble() {
        return -1;
    }

    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return -1;
    }
    assert!(numStates > 0 && numStates < MAX_RUMBLE_STATES as c_int);

    let mut i: c_int = 0;
    while i < MAX_RUMBLE_SCRIPTS as c_int {
        unsafe {
            if (*addr_of_mut!(rumbleScripts).add(i as usize)).states.is_null() {
                break;
            }
        }
        i += 1;
    }

    if i == MAX_RUMBLE_SCRIPTS as c_int {
        return -1; // Ran out of scripts
    }

    unsafe {
        let idx = i as usize;
        let states_size = core::mem::size_of::<rumblestate_t>() * numStates as usize;
        let states_ptr = malloc(states_size) as *mut rumblestate_t;

        (*addr_of_mut!(rumbleScripts).add(idx)).autoDelete = deleteWhenFinished;
        (*addr_of_mut!(rumbleScripts).add(idx)).controller = controller;
        (*addr_of_mut!(rumbleScripts).add(idx)).currentState = 0;
        (*addr_of_mut!(rumbleScripts).add(idx)).nextStateAt = 0;
        (*addr_of_mut!(rumbleScripts).add(idx)).numStates = numStates;
        (*addr_of_mut!(rumbleScripts).add(idx)).usedStates = 0;
        (*addr_of_mut!(rumbleScripts).add(idx)).states = states_ptr;

        memset(
            states_ptr as *mut c_void,
            0,
            core::mem::size_of::<rumblestate_t>() * numStates as usize,
        );
    }

    i
}

// A negative time will last until you kill it explicitly
// Returns index, used to kill or change a state in a script
pub fn IN_AddRumbleStateFull(whichScript: c_int, arg1: c_int, arg2: c_int, timeInMs: c_int) -> c_int {
    if !IN_usingRumble() {
        return -1;
    }

    assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);
    unsafe {
        assert!(
            (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).usedStates
                < (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).numStates
        );

        // Get the current state
        let curScript = addr_of_mut!(rumbleScripts).add(whichScript as usize);
        let cur_used_states = (*curScript).usedStates as usize;
        let curState = (*curScript).states.add(cur_used_states);

        (*curState).arg1 = arg1;
        (*curState).arg2 = arg2;
        (*curState).timeToStop = timeInMs;

        let result = (*curScript).usedStates;
        (*curScript).usedStates += 1;
        result
    }
}

pub fn IN_AddRumbleState(whichScript: c_int, leftSpeed: c_int, rightSpeed: c_int, timeInMs: c_int) -> c_int {
    IN_AddRumbleStateFull(whichScript, leftSpeed, rightSpeed, timeInMs)
}

pub fn IN_AddRumbleStateSpecial(whichScript: c_int, action: c_int, arg1: c_int, arg2: c_int) -> c_int {
    if !IN_usingRumble() {
        return -1;
    }

    assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);
    unsafe {
        assert!(
            (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).usedStates
                < (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).numStates
        );

        // Get the current state
        let curScript = addr_of_mut!(rumbleScripts).add(whichScript as usize);
        let cur_used_states = (*curScript).usedStates as usize;
        let curState = (*curScript).states.add(cur_used_states) as *mut rumblestate_special_t;

        (*curState).code = action;
        (*curState).arg1 = arg1;
        (*curState).arg2 = arg2;

        let result = (*curScript).usedStates;
        (*curScript).usedStates += 1;
        result
    }
}

pub fn IN_AddEffectFade4(
    whichScript: c_int,
    startLeft: c_int,
    startRight: c_int,
    endLeft: c_int,
    endRight: c_int,
    timeInMs: c_int,
) -> c_int {
    const fadeSmoothness: c_int = 50; // number of ms between updates, smaller is smoother

    let e = IN_AddRumbleState(whichScript, startLeft, startRight, fadeSmoothness); // Lasts for fadeSmoothness ms

    if startLeft < endLeft {
        // Fade increases
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_INC_LEFT,
            e,
            (endLeft - startLeft) * fadeSmoothness / timeInMs,
        );
    } else {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_DEC_LEFT,
            e,
            (startLeft - endLeft) * fadeSmoothness / timeInMs,
        );
    }

    if startRight < endRight {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_INC_RIGHT,
            e,
            (endRight - startRight) * fadeSmoothness / timeInMs,
        );
    } else {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_DEC_RIGHT,
            e,
            (startRight - endRight) * fadeSmoothness / timeInMs,
        );
    }

    IN_AddRumbleStateSpecial(whichScript, IN_CMD_GOTO_XTIMES, e, timeInMs / fadeSmoothness)
}

pub fn IN_AddEffectFadeExp6(
    whichScript: c_int,
    startLeft: c_int,
    startRight: c_int,
    endLeft: c_int,
    endRight: c_int,
    factor: c_char,
    timeInMs: c_int,
) -> c_int {
    const fadeSmoothness: c_int = 10; // number of ms between updates, smaller is smoother

    let state = IN_AddRumbleState(whichScript, startLeft, startRight, fadeSmoothness); // Lasts for fadeSmoothness ms

    let factor_i = factor as c_int;

    if startLeft < endLeft {
        // Fade increases
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_INC_LEFT,
            state,
            (endLeft - startLeft) * fadeSmoothness / timeInMs - (factor_i / 2) * (1 - timeInMs / fadeSmoothness),
        );
    } else {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_DEC_LEFT,
            state,
            (startLeft - endLeft) * fadeSmoothness / timeInMs - (factor_i / 2) * (1 - timeInMs / fadeSmoothness),
        );
    }

    if startRight < endRight {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_INC_RIGHT,
            state,
            (endRight - startRight) * fadeSmoothness / timeInMs - (factor_i / 2) * (1 - timeInMs / fadeSmoothness),
        );
    } else {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_DEC_RIGHT,
            state,
            (startRight - endRight) * fadeSmoothness / timeInMs - (factor_i / 2) * (1 - timeInMs / fadeSmoothness),
        );
    }

    IN_AddRumbleStateSpecial(whichScript, IN_CMD_INC_ARG2, state + 1, factor_i);
    IN_AddRumbleStateSpecial(whichScript, IN_CMD_INC_ARG2, state + 2, factor_i);
    IN_AddRumbleStateSpecial(whichScript, IN_CMD_GOTO_XTIMES, state, timeInMs / fadeSmoothness)
}

// Kills a rumble state based on index
pub fn IN_KillRumbleState(whichScript: c_int, index: c_int) {
    if !IN_usingRumble() {
        return;
    }

    assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);
    assert!(index < unsafe { (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).numStates });

    unsafe {
        let script_idx = whichScript as usize;
        let idx = index as usize;
        (*addr_of_mut!(rumbleScripts).add(script_idx))
            .states
            .add(idx)
            .write(rumblestate_t {
                timeToStop: 0,
                arg1: (*(*addr_of_mut!(rumbleScripts).add(script_idx)).states.add(idx)).arg1,
                arg2: (*(*addr_of_mut!(rumbleScripts).add(script_idx)).states.add(idx)).arg2,
            });

        let controller = (*addr_of_mut!(rumbleScripts).add(script_idx)).controller as usize;
        (*addr_of_mut!(rumbleStatus).add(controller)).changed = true;
    }
}

// Stops the script, if script has autodelete on then it will get deleted, otherwise it will only stop
pub fn IN_KillRumbleScript(whichScript: c_int) {
    if !IN_usingRumble() {
        return;
    }

    assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);

    unsafe {
        let script_idx = whichScript as usize;
        (*addr_of_mut!(rumbleScripts).add(script_idx)).nextStateAt = 0;

        if (*addr_of_mut!(rumbleScripts).add(script_idx)).autoDelete {
            if !(*addr_of_mut!(rumbleScripts).add(script_idx)).states.is_null() {
                free((*addr_of_mut!(rumbleScripts).add(script_idx)).states as *mut c_void);
            }
            (*addr_of_mut!(rumbleScripts).add(script_idx)).states = core::ptr::null_mut();
        }

        let controller = (*addr_of_mut!(rumbleScripts).add(script_idx)).controller as usize;
        (*addr_of_mut!(rumbleStatus).add(controller)).changed = true;
    }
}

// Stops Rumbling for specific controller
pub fn IN_KillRumbleScripts_Controller(controller: c_int) {
    if !IN_usingRumble() {
        return;
    }
    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return;
    }
    unsafe {
        if (*addr_of_mut!(rumbleStatus).add(controller as usize)).killed {
            return;
        }
    }

    for i in 0..MAX_RUMBLE_SCRIPTS as c_int {
        unsafe {
            if (*addr_of_mut!(rumbleScripts).add(i as usize)).controller == controller {
                IN_KillRumbleScript(i);
            }
        }
    }

    unsafe {
        let controller_idx = controller as usize;
        (*addr_of_mut!(rumbleStatus).add(controller_idx)).killed = IN_RumbleAdjust(controller, 0, 0);
    }
}

// Stops Rumbling on all controllers
pub fn IN_KillRumbleScripts_All() {
    if !IN_usingRumble() {
        return;
    }

    for i in 0..MAX_RUMBLE_SCRIPTS as c_int {
        IN_KillRumbleScript(i);
    }

    for j in 0..MAX_RUMBLE_CONTROLLERS as c_int {
        unsafe {
            if !(*addr_of_mut!(rumbleStatus).add(j as usize)).killed {
                (*addr_of_mut!(rumbleStatus).add(j as usize)).killed =
                    IN_RumbleAdjust(j, 0, 0);
            }
        }
    }
}

pub fn IN_DeleteRumbleScript(whichScript: c_int) {
    if !IN_usingRumble() {
        return;
    }

    assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);

    unsafe {
        let script_idx = whichScript as usize;
        if !(*addr_of_mut!(rumbleScripts).add(script_idx)).states.is_null() {
            free((*addr_of_mut!(rumbleScripts).add(script_idx)).states as *mut c_void);
        }
        (*addr_of_mut!(rumbleScripts).add(script_idx)).nextStateAt = 0;
        (*addr_of_mut!(rumbleScripts).add(script_idx)).states = core::ptr::null_mut();

        let controller = (*addr_of_mut!(rumbleScripts).add(script_idx)).controller as usize;
        (*addr_of_mut!(rumbleStatus).add(controller)).changed = true;
    }
}

fn IN_RunSpecialScript(whichScript: c_int) -> c_int {
    unsafe {
        let script_idx = whichScript as usize;
        let cur_state_idx = (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState as usize;
        let sp = (*addr_of_mut!(rumbleScripts).add(script_idx))
            .states
            .add(cur_state_idx) as *mut rumblestate_special_t;

        match (*sp).code {
            // updates the current state pointer
            // uses arg1
            IN_CMD_GOTO => {
                (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState = (*sp).arg1;
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize)
                    .read()
                    .timeToStop
            }
            // does a goto, and decreases count of arg2, until 0
            IN_CMD_GOTO_XTIMES => {
                (*sp).arg2 -= 1;
                if (*sp).arg2 >= 0 {
                    (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState = (*sp).arg1;
                    (*addr_of_mut!(rumbleScripts).add(script_idx))
                        .states
                        .add((*sp).arg1 as usize)
                        .read()
                        .timeToStop
                } else {
                    // Go onto next cmd
                    if !IN_AdvanceToNextState(whichScript) {
                        return -2; // Done
                    }
                    return -1;
                }
            }

            // Decreasae Arg2 of a State,		sp->arg1 = state, sp->arg2 = amount to decrease arg2 of state by
            IN_CMD_DEC_ARG2 => {
                let temp = (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize) as *mut rumblestate_special_t;
                (*temp).arg2 -= (*sp).arg2;
                0
            }

            // Increase Arg2 of a State,		sp->arg1 = state, sp->arg2 = amount to increase arg2 of state by
            IN_CMD_INC_ARG2 => {
                let temp = (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize) as *mut rumblestate_special_t;
                (*temp).arg2 += (*sp).arg2;
                0
            }

            // Decreasae Arg1 of a State,		sp->arg1 = state, sp->arg2 = amount to decrease arg1 of state by
            IN_CMD_DEC_ARG1 => {
                let temp = (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize) as *mut rumblestate_special_t;
                (*temp).arg1 -= (*sp).arg2;
                0
            }

            // Increase Arg2 of a State,		sp->arg1 = state, sp->arg2 = amount to increase arg1 of state by
            IN_CMD_INC_ARG1 => {
                let temp = (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize) as *mut rumblestate_special_t;
                (*temp).arg1 += (*sp).arg2;
                0
            }

            IN_CMD_DEC_LEFT => {
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize)
                    .write(rumblestate_t {
                        timeToStop: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .timeToStop,
                        arg1: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .arg1,
                        arg2: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .arg2
                            - (*sp).arg2,
                    });

                if (*addr_of_mut!(rumbleScripts).add(script_idx)).states.add((*sp).arg1 as usize).read().arg2 < 0 {
                    (*addr_of_mut!(rumbleScripts).add(script_idx))
                        .states
                        .add((*sp).arg1 as usize)
                        .write(rumblestate_t {
                            timeToStop: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                                .states
                                .add((*sp).arg1 as usize))
                            .timeToStop,
                            arg1: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                                .states
                                .add((*sp).arg1 as usize))
                            .arg1,
                            arg2: 0,
                        });
                }

                if (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState
                    >= (*addr_of_mut!(rumbleScripts).add(script_idx)).usedStates - 1
                {
                    return -2; // Done
                }
                (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState += 1;
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*addr_of_mut!(rumbleScripts).add(script_idx)).currentState as usize)
                    .read()
                    .timeToStop
            }

            IN_CMD_DEC_RIGHT => {
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize)
                    .write(rumblestate_t {
                        timeToStop: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .timeToStop,
                        arg1: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .arg1
                            - (*sp).arg2,
                        arg2: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .arg2,
                    });

                if (*addr_of_mut!(rumbleScripts).add(script_idx)).states.add((*sp).arg1 as usize).read().arg1 < 0 {
                    (*addr_of_mut!(rumbleScripts).add(script_idx))
                        .states
                        .add((*sp).arg1 as usize)
                        .write(rumblestate_t {
                            timeToStop: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                                .states
                                .add((*sp).arg1 as usize))
                            .timeToStop,
                            arg1: 0,
                            arg2: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                                .states
                                .add((*sp).arg1 as usize))
                            .arg2,
                        });
                }

                if (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState
                    >= (*addr_of_mut!(rumbleScripts).add(script_idx)).usedStates - 1
                {
                    return -2; // Done
                }
                (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState += 1;
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*addr_of_mut!(rumbleScripts).add(script_idx)).currentState as usize)
                    .read()
                    .timeToStop
            }

            IN_CMD_INC_LEFT => {
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize)
                    .write(rumblestate_t {
                        timeToStop: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .timeToStop,
                        arg1: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .arg1,
                        arg2: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .arg2
                            + (*sp).arg2,
                    });

                if (*addr_of_mut!(rumbleScripts).add(script_idx)).states.add((*sp).arg1 as usize).read().arg2 > 65534 {
                    (*addr_of_mut!(rumbleScripts).add(script_idx))
                        .states
                        .add((*sp).arg1 as usize)
                        .write(rumblestate_t {
                            timeToStop: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                                .states
                                .add((*sp).arg1 as usize))
                            .timeToStop,
                            arg1: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                                .states
                                .add((*sp).arg1 as usize))
                            .arg1,
                            arg2: 65534,
                        });
                }

                if (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState
                    >= (*addr_of_mut!(rumbleScripts).add(script_idx)).usedStates - 1
                {
                    return -2; // Done
                }
                (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState += 1;
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*addr_of_mut!(rumbleScripts).add(script_idx)).currentState as usize)
                    .read()
                    .timeToStop
            }

            IN_CMD_INC_RIGHT => {
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*sp).arg1 as usize)
                    .write(rumblestate_t {
                        timeToStop: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .timeToStop,
                        arg1: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .arg1
                            + (*sp).arg2,
                        arg2: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                            .states
                            .add((*sp).arg1 as usize))
                        .arg2,
                    });

                if (*addr_of_mut!(rumbleScripts).add(script_idx)).states.add((*sp).arg1 as usize).read().arg1 > 65534 {
                    (*addr_of_mut!(rumbleScripts).add(script_idx))
                        .states
                        .add((*sp).arg1 as usize)
                        .write(rumblestate_t {
                            timeToStop: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                                .states
                                .add((*sp).arg1 as usize))
                            .timeToStop,
                            arg1: 65534,
                            arg2: (*(*addr_of_mut!(rumbleScripts).add(script_idx))
                                .states
                                .add((*sp).arg1 as usize))
                            .arg2,
                        });
                }

                if (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState
                    >= (*addr_of_mut!(rumbleScripts).add(script_idx)).usedStates - 1
                {
                    return -2; // Done
                }
                (*addr_of_mut!(rumbleScripts).add(script_idx)).currentState += 1;
                (*addr_of_mut!(rumbleScripts).add(script_idx))
                    .states
                    .add((*addr_of_mut!(rumbleScripts).add(script_idx)).currentState as usize)
                    .read()
                    .timeToStop
            }

            _ => 0,
        }
    }
}

fn IN_Time() -> c_int {
    //mb return ClientManager::ActiveClient().cg.time;
    unsafe { cg.time }
}

pub fn IN_ExecuteRumbleScript(whichScript: c_int) {
    if !IN_usingRumble() {
        return;
    }

    assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);

    unsafe {
        // Can't execute an empty script???
        assert!((*addr_of_mut!(rumbleScripts).add(whichScript as usize)).usedStates > 0);

        (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).currentState = 0;
        let cmd = (*addr_of_mut!(rumbleScripts).add(whichScript as usize))
            .states
            .read()
            .timeToStop;

        let mut final_cmd = cmd;
        if cmd < 0 {
            final_cmd = IN_RunSpecialScript(whichScript);
        }

        (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).nextStateAt = IN_Time() + final_cmd;

        let controller = (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).controller as usize;
        (*addr_of_mut!(rumbleStatus).add(controller)).changed = true;
        (*addr_of_mut!(rumbleStatus).add(controller)).killed = false;
    }
}

pub fn IN_PauseRumbling_Controller(controller: c_int) {
    if !IN_usingRumble() {
        return;
    }
    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return;
    }
    unsafe {
        if (*addr_of_mut!(rumbleStatus).add(controller as usize)).paused {
            return;
        }

        (*addr_of_mut!(rumbleStatus).add(controller as usize)).timePaused = IN_Time();
        (*addr_of_mut!(rumbleStatus).add(controller as usize)).paused = IN_RumbleAdjust(controller, 0, 0);
    }
}

pub fn IN_UnPauseRumbling_Controller(controller: c_int) {
    if !IN_usingRumble() {
        return;
    }
    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return;
    }

    unsafe {
        // can't unpause a control that wasn't paused
        if !(*addr_of_mut!(rumbleStatus).add(controller as usize)).paused {
            return;
        }

        let cur_time = IN_Time();
        for i in 0..MAX_RUMBLE_SCRIPTS as c_int {
            if (*addr_of_mut!(rumbleScripts).add(i as usize)).controller == controller {
                if (*addr_of_mut!(rumbleScripts).add(i as usize)).nextStateAt == 0 {
                    continue;
                }
                // update the time to stop based on how long it was paused
                (*addr_of_mut!(rumbleScripts).add(i as usize)).nextStateAt +=
                    cur_time - (*addr_of_mut!(rumbleStatus).add(controller as usize)).timePaused;
            }
        }

        (*addr_of_mut!(rumbleStatus).add(controller as usize)).paused = false;
        (*addr_of_mut!(rumbleStatus).add(controller as usize)).changed = true;
        (*addr_of_mut!(rumbleStatus).add(controller as usize)).killed = false;
    }
}

pub fn IN_TogglePauseRumbling_Controller(controller: c_int) {
    if !IN_usingRumble() {
        return;
    }
    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return;
    }
    unsafe {
        if (*addr_of_mut!(rumbleStatus).add(controller as usize)).paused {
            IN_UnPauseRumbling_Controller(controller);
        } else {
            IN_PauseRumbling_Controller(controller);
        }
    }
}

// Pauses rumbling on all controllers
pub fn IN_PauseRumbling_All() {
    if !IN_usingRumble() {
        return;
    }
    for i in 0..MAX_RUMBLE_CONTROLLERS as c_int {
        IN_PauseRumbling_Controller(i);
    }
}

// UnPauses rumbling on all controllers
pub fn IN_UnPauseRumbling_All() {
    if !IN_usingRumble() {
        return;
    }
    for i in 0..MAX_RUMBLE_CONTROLLERS as c_int {
        IN_UnPauseRumbling_Controller(i);
    }
}

// Toggles Pausing on all controllers
pub fn IN_TogglePauseRumbling_All() {
    if !IN_usingRumble() {
        return;
    }
    for i in 0..MAX_RUMBLE_CONTROLLERS as c_int {
        IN_TogglePauseRumbling_Controller(i);
    }
}

// Returns false when the end of the script is reached
pub fn IN_AdvanceToNextState(whichScript: c_int) -> bool {
    assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);

    unsafe {
        if (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).currentState
            >= (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).usedStates - 1
        {
            // Script is at its end, so kill it( which deletes only if autodelete
            IN_KillRumbleScript(whichScript);
            return false;
        }

        // Advance a state
        (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).currentState += 1;

        let mut cmd = (*addr_of_mut!(rumbleScripts).add(whichScript as usize))
            .states
            .add((*addr_of_mut!(rumbleScripts).add(whichScript as usize)).currentState as usize)
            .read()
            .timeToStop;

        while cmd < 0 {
            cmd = IN_RunSpecialScript(whichScript);
            if cmd == -1 {
                return true;
            }
            if cmd == -2 {
                return false;
            }
        }

        (*addr_of_mut!(rumbleScripts).add(whichScript as usize)).nextStateAt = IN_Time() + cmd;
        true
    }
}

// Max rumble takes precidence
// Other possibility is some kind of sum of all the speeds
// Call this once a frame, to update the controller based on the rumble states
pub fn IN_UpdateRumbleFromStates() {
    //if (!IN_usingRumble()) return;
    /*mb	extern int G_ShouldBeRumbling();
    if (!G_ShouldBeRumbling())
        return;
    */

    unsafe {
        let mut usingRumble: [c_int; 2] = [0; 2];
        usingRumble[0] = Cvar_VariableIntegerValue(b"in_useRumble\0".as_ptr() as *const c_char);
        usingRumble[1] = Cvar_VariableIntegerValue(b"in_useRumble2\0".as_ptr() as *const c_char);

        let mut value: [[c_int; 2]; MAX_RUMBLE_CONTROLLERS] = [[0; 2]; MAX_RUMBLE_CONTROLLERS];
        let cur_time = IN_Time();

        for i in 0..MAX_RUMBLE_SCRIPTS as c_int {
            // If rumble is paused on current controller than skip this rumble state
            if (*addr_of_mut!(rumbleStatus).add(
                (*addr_of_mut!(rumbleScripts).add(i as usize)).controller as usize,
            ))
            .paused
            {
                continue;
            }

            //*mb	ClientManager::ActivateByControllerId(rumbleScripts[i].controller);
            if usingRumble[ActiveClientNum() as usize] == 0 {
                IN_KillRumbleScript(i);
                continue;
            }
            /*mb
            if (!ClientManager::ActiveGentity() || !G_ActivePlayerNormal())
            {
                IN_KillRumbleScript(i);
                continue;
            }
            */

            // Unset state so skip
            if (*addr_of_mut!(rumbleScripts).add(i as usize)).nextStateAt == 0 {
                continue;
            }

            // Time is up on this rumble state
            if (*addr_of_mut!(rumbleScripts).add(i as usize)).nextStateAt < cur_time {
                // If timeToStop is < cur_time and > 0 then end this state otherwise (negative number) always rumble
                if (*addr_of_mut!(rumbleScripts).add(i as usize)).nextStateAt > 0 {
                    let controller = (*addr_of_mut!(rumbleScripts).add(i as usize)).controller as usize;
                    (*addr_of_mut!(rumbleStatus).add(controller)).changed = true;
                    (*addr_of_mut!(rumbleStatus).add(controller)).killed = false;
                    if !IN_AdvanceToNextState(i) {
                        // Returns false if reached the end of script
                        continue;
                    }
                }
            }

            let curScript = addr_of_mut!(rumbleScripts).add(i as usize);
            let controller_idx = (*curScript).controller as usize;
            let cur_state_idx = (*curScript).currentState as usize;
            let arg2 = (*curScript).states.add(cur_state_idx).read().arg2;
            let arg1 = (*curScript).states.add(cur_state_idx).read().arg1;

            if value[controller_idx][0] < arg2 {
                value[controller_idx][0] = arg2;
            }
            if value[controller_idx][1] < arg1 {
                value[controller_idx][1] = arg1;
            }
        }

        // Go through the 4 controller ports
        for i in 0..MAX_RUMBLE_CONTROLLERS as c_int {
            // paused, so do nothing for this controller
            if (*addr_of_mut!(rumbleStatus).add(i as usize)).paused {
                continue;
            }

            // Only update the actual hardware if a state has changed
            if !(*addr_of_mut!(rumbleStatus).add(i as usize)).changed {
                continue;
            }

            IN_RumbleAdjust(i, value[i as usize][0], value[i as usize][1]);

            // State has changed
            (*addr_of_mut!(rumbleStatus).add(i as usize)).changed = false;
        }
    }
}

/*
==================
IN_RumbleInit
==================
*/
pub fn IN_RumbleInit() {
    unsafe {
        memset(
            addr_of_mut!(rumbleStatus) as *mut c_void,
            0,
            core::mem::size_of::<rumblestatus_t>() * MAX_RUMBLE_CONTROLLERS,
        );
        memset(
            addr_of_mut!(rumbleScripts) as *mut c_void,
            0,
            core::mem::size_of::<rumblescript_t>() * MAX_RUMBLE_SCRIPTS,
        );

        in_useRumble = Cvar_Get(b"in_useRumble\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
        Cvar_Get(b"in_useRumble2\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
    }
}

/*
==================
IN_RumbleShutdown
==================
*/
pub fn IN_RumbleShutdown() {
    unsafe {
        for i in 0..MAX_RUMBLE_SCRIPTS as c_int {
            if !(*addr_of_mut!(rumbleScripts).add(i as usize)).states.is_null() {
                free((*addr_of_mut!(rumbleScripts).add(i as usize)).states as *mut c_void);
            }
            (*addr_of_mut!(rumbleScripts).add(i as usize)).states = core::ptr::null_mut();
            (*addr_of_mut!(rumbleScripts).add(i as usize)).nextStateAt = 0;
        }
    }
}

/*
==================
IN_RumbleFrame
==================
*/
pub fn IN_RumbleFrame() {
    unsafe {
        // Check to see if we need to pause rumbling
        let main_controller = IN_GetMainController();
        if cl_paused.integer != 0 && !(*addr_of_mut!(rumbleStatus).add(main_controller as usize)).paused {
            IN_PauseRumbling_Controller(main_controller);
        } else if cl_paused.integer == 0 && (*addr_of_mut!(rumbleStatus).add(main_controller as usize)).paused {
            IN_UnPauseRumbling_Controller(main_controller);
        }

        // Update the states
        IN_UpdateRumbleFromStates();
    }
}
