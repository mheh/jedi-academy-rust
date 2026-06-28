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

use core::ffi::c_int;

// MB #include "../client/cl_data.h"
// Include stubs for external symbols
extern "C" {
    fn G_ActivePlayerNormal() -> bool;
    fn Cvar_Set(name: *const core::ffi::c_char, value: *const core::ffi::c_char);
    fn Cvar_VariableIntegerValue(name: *const core::ffi::c_char) -> c_int;
    fn Cvar_Get(
        name: *const core::ffi::c_char,
        value: *const core::ffi::c_char,
        flags: c_int,
    ) -> *mut cvar_t;
    fn IN_RumbleAdjust(controller: c_int, left: c_int, right: c_int) -> bool;
    fn IN_GetMainController() -> c_int;

    // Global cvar
    static cl_paused: *mut cvar_t;
    // Global from cg_local.h
    static cg: cg_t;
}

// Stub types for external dependencies
#[repr(C)]
pub struct cvar_t {
    pub name: *const core::ffi::c_char,
    pub string: *const core::ffi::c_char,
    pub resetString: *const core::ffi::c_char,
    pub latched: *const core::ffi::c_char,
    pub flags: c_int,
    pub modified: bool,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
}

#[repr(C)]
pub struct cg_t {
    pub time: c_int,
    // ... other fields omitted, we only need time
}

// Constants from win_input.h
const IN_CMD_GOTO_XTIMES: c_int = -5;
const IN_CMD_GOTO: c_int = -6;
const IN_CMD_DEC_ARG2: c_int = -7;
const IN_CMD_INC_ARG2: c_int = -8;
const IN_CMD_DEC_ARG1: c_int = -9;
const IN_CMD_INC_ARG1: c_int = -10;
const IN_CMD_DEC_LEFT: c_int = -70;
const IN_CMD_DEC_RIGHT: c_int = -71;
const IN_CMD_INC_LEFT: c_int = -72;
const IN_CMD_INC_RIGHT: c_int = -73;

#[repr(C)]
pub struct rumblestate_t {
    pub timeToStop: c_int,
    // Right motor speed on Xbox, action type on Gamecube
    pub arg1: c_int,
    // Left motor speed on Xbox, secondary action type on Gamecube
    pub arg2: c_int,
}

impl Default for rumblestate_t {
    fn default() -> Self {
        rumblestate_t {
            timeToStop: 0,
            arg1: 0,
            arg2: 0,
        }
    }
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

impl Default for rumblescript_t {
    fn default() -> Self {
        rumblescript_t {
            nextStateAt: 0,
            controller: 0,
            currentState: 0,
            usedStates: 0,
            numStates: 0,
            autoDelete: false,
            states: core::ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct rumblestatus_t {
    pub changed: bool,
    pub killed: bool,
    pub paused: bool,
    pub timePaused: c_int,
}

impl Default for rumblestatus_t {
    fn default() -> Self {
        rumblestatus_t {
            changed: false,
            killed: false,
            paused: false,
            timePaused: 0,
        }
    }
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
    };
    MAX_RUMBLE_CONTROLLERS
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
    };
    MAX_RUMBLE_SCRIPTS
];

static mut in_useRumble: *mut cvar_t = core::ptr::null_mut();

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
    unsafe {
        if ActiveClientNum() == 0 {
            Cvar_Set(
                b"in_useRumble\0".as_ptr() as *const core::ffi::c_char,
                b"1\0".as_ptr() as *const core::ffi::c_char,
            );
        } else {
            Cvar_Set(
                b"in_useRumble2\0".as_ptr() as *const core::ffi::c_char,
                b"1\0".as_ptr() as *const core::ffi::c_char,
            );
        }
    }
}

pub fn IN_disableRumble() {
    unsafe {
        if ActiveClientNum() == 0 {
            Cvar_Set(
                b"in_useRumble\0".as_ptr() as *const core::ffi::c_char,
                b"0\0".as_ptr() as *const core::ffi::c_char,
            );
        } else {
            Cvar_Set(
                b"in_useRumble2\0".as_ptr() as *const core::ffi::c_char,
                b"0\0".as_ptr() as *const core::ffi::c_char,
            );
        }
    }
}

pub fn IN_usingRumble() -> bool {
    unsafe {
        if ActiveClientNum() == 0 {
            //return Cvar_VariableIntegerValue( "in_useRumble");
            return (*in_useRumble).integer != 0;
        } else {
            //return Cvar_VariableIntegerValue( "in_useRumble2");
            return (*in_useRumble).integer != 0;
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

    unsafe {
        let mut i: usize;
        for i_val in 0..MAX_RUMBLE_SCRIPTS {
            i = i_val;
            if rumbleScripts[i].states.is_null() {
                break;
            }
        }

        if i == MAX_RUMBLE_SCRIPTS {
            return -1; // Ran out of scripts
        }

        rumbleScripts[i].autoDelete = deleteWhenFinished;
        rumbleScripts[i].controller = controller;
        rumbleScripts[i].currentState = 0;
        rumbleScripts[i].nextStateAt = 0;
        rumbleScripts[i].numStates = numStates;
        rumbleScripts[i].usedStates = 0;

        // Allocate array
        let boxed: Box<[rumblestate_t]> = vec![rumblestate_t::default(); numStates as usize]
            .into_boxed_slice();
        rumbleScripts[i].states = Box::into_raw(boxed) as *mut rumblestate_t;

        // memset equivalent - already initialized via vec::default()
        return i as c_int;
    }
}

// A negative time will last until you kill it explicitly
// Returns index, used to kill or change a state in a script
pub fn IN_AddRumbleStateFull(whichScript: c_int, arg1: c_int, arg2: c_int, timeInMs: c_int) -> c_int {
    if !IN_usingRumble() {
        return -1;
    }

    unsafe {
        assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);
        assert!(
            rumbleScripts[whichScript as usize].usedStates
                < rumbleScripts[whichScript as usize].numStates
        );

        // Get the current state
        let curScript = &mut rumbleScripts[whichScript as usize];
        let curState = &mut (*curScript.states.add(curScript.usedStates as usize));

        curState.arg1 = arg1;
        curState.arg2 = arg2;

        curState.timeToStop = timeInMs;
        curScript.usedStates += 1;
        return curScript.usedStates - 1;
    }
}

pub fn IN_AddRumbleState(whichScript: c_int, leftSpeed: c_int, rightSpeed: c_int, timeInMs: c_int) -> c_int {
    IN_AddRumbleStateFull(whichScript, leftSpeed, rightSpeed, timeInMs)
}

pub fn IN_AddRumbleStateSpecial(whichScript: c_int, action: c_int, arg1: c_int, arg2: c_int) -> c_int {
    if !IN_usingRumble() {
        return -1;
    }

    unsafe {
        assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);
        assert!(
            rumbleScripts[whichScript as usize].usedStates
                < rumbleScripts[whichScript as usize].numStates
        );

        // Get the current state
        let curScript = &mut rumbleScripts[whichScript as usize];
        let curState = &mut *(
            curScript.states.add(curScript.usedStates as usize) as *mut rumblestate_special_t
        );

        curState.code = action;
        curState.arg1 = arg1;
        curState.arg2 = arg2;
        curScript.usedStates += 1;
        return curScript.usedStates - 1;
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

    return IN_AddRumbleStateSpecial(whichScript, IN_CMD_GOTO_XTIMES, e, timeInMs / fadeSmoothness);
}

pub fn IN_AddEffectFadeExp6(
    whichScript: c_int,
    startLeft: c_int,
    startRight: c_int,
    endLeft: c_int,
    endRight: c_int,
    factor: core::ffi::c_char,
    timeInMs: c_int,
) -> c_int {
    const fadeSmoothness: c_int = 10; // number of ms between updates, smaller is smoother

    let state = IN_AddRumbleState(whichScript, startLeft, startRight, fadeSmoothness); // Lasts for fadeSmoothness ms

    if startLeft < endLeft {
        // Fade increases
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_INC_LEFT,
            state,
            (endLeft - startLeft) * fadeSmoothness / timeInMs
                - (factor as c_int / 2) * (1 - timeInMs / fadeSmoothness),
        );
    } else {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_DEC_LEFT,
            state,
            (startLeft - endLeft) * fadeSmoothness / timeInMs
                - (factor as c_int / 2) * (1 - timeInMs / fadeSmoothness),
        );
    }

    if startRight < endRight {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_INC_RIGHT,
            state,
            (endRight - startRight) * fadeSmoothness / timeInMs
                - (factor as c_int / 2) * (1 - timeInMs / fadeSmoothness),
        );
    } else {
        IN_AddRumbleStateSpecial(
            whichScript,
            IN_CMD_DEC_RIGHT,
            state,
            (startRight - endRight) * fadeSmoothness / timeInMs
                - (factor as c_int / 2) * (1 - timeInMs / fadeSmoothness),
        );
    }

    IN_AddRumbleStateSpecial(whichScript, IN_CMD_INC_ARG2, state + 1, factor as c_int);
    IN_AddRumbleStateSpecial(whichScript, IN_CMD_INC_ARG2, state + 2, factor as c_int);
    return IN_AddRumbleStateSpecial(whichScript, IN_CMD_GOTO_XTIMES, state, timeInMs / fadeSmoothness);
}

// Kills a rumble state based on index
pub fn IN_KillRumbleState(whichScript: c_int, index: c_int) {
    if !IN_usingRumble() {
        return;
    }

    unsafe {
        assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);
        assert!(index < rumbleScripts[whichScript as usize].numStates);

        rumbleScripts[whichScript as usize].states[index as usize].timeToStop = 0;
        rumbleStatus[rumbleScripts[whichScript as usize].controller as usize].changed = true;
    }
}

// Stops the script, if script has autodelete on then it will get deleted, otherwise it will only stop
pub fn IN_KillRumbleScript(whichScript: c_int) {
    if !IN_usingRumble() {
        return;
    }

    unsafe {
        assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);

        rumbleScripts[whichScript as usize].nextStateAt = 0;
        if rumbleScripts[whichScript as usize].autoDelete {
            if !rumbleScripts[whichScript as usize].states.is_null() {
                let _ = Box::from_raw(core::slice::from_raw_parts_mut(
                    rumbleScripts[whichScript as usize].states,
                    rumbleScripts[whichScript as usize].numStates as usize,
                ));
            }
            rumbleScripts[whichScript as usize].states = core::ptr::null_mut();
        }

        rumbleStatus[rumbleScripts[whichScript as usize].controller as usize].changed = true;
    }
}

// Stops Rumbling for specific controller
pub fn IN_KillRumbleScripts_controller(controller: c_int) {
    if !IN_usingRumble() {
        return;
    }
    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return;
    }

    unsafe {
        if rumbleStatus[controller as usize].killed == true {
            return;
        }

        for i in 0..MAX_RUMBLE_SCRIPTS {
            if rumbleScripts[i].controller == controller {
                IN_KillRumbleScript(i as c_int);
            }
        }

        rumbleStatus[controller as usize].killed = IN_RumbleAdjust(controller, 0, 0);
    }
}

// Stops Rumbling on all controllers
pub fn IN_KillRumbleScripts() {
    if !IN_usingRumble() {
        return;
    }

    unsafe {
        for i in 0..MAX_RUMBLE_SCRIPTS {
            IN_KillRumbleScript(i as c_int);
        }

        for j in 0..MAX_RUMBLE_CONTROLLERS {
            if !rumbleStatus[j].killed {
                rumbleStatus[j].killed = IN_RumbleAdjust(j as c_int, 0, 0);
            }
        }
    }
}

pub fn IN_DeleteRumbleScript(whichScript: c_int) {
    if !IN_usingRumble() {
        return;
    }

    unsafe {
        assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);

        if !rumbleScripts[whichScript as usize].states.is_null() {
            let _ = Box::from_raw(core::slice::from_raw_parts_mut(
                rumbleScripts[whichScript as usize].states,
                rumbleScripts[whichScript as usize].numStates as usize,
            ));
        }
        rumbleScripts[whichScript as usize].nextStateAt = 0;
        rumbleScripts[whichScript as usize].states = core::ptr::null_mut();

        rumbleStatus[rumbleScripts[whichScript as usize].controller as usize].changed = true;
    }
}

fn IN_RunSpecialScript(whichScript: c_int) -> c_int {
    unsafe {
        let sp = &mut *(
            rumbleScripts[whichScript as usize]
                .states
                .add(rumbleScripts[whichScript as usize].currentState as usize)
                as *mut rumblestate_special_t
        );
        match sp.code {
            // updates the current state pointer
            // uses arg1
            IN_CMD_GOTO => {
                rumbleScripts[whichScript as usize].currentState = sp.arg1;
                return rumbleScripts[whichScript as usize].states[sp.arg1 as usize].timeToStop;
            }
            // does a goto, and decreases count of arg2, until 0
            IN_CMD_GOTO_XTIMES => {
                sp.arg2 -= 1;
                if sp.arg2 >= 0 {
                    rumbleScripts[whichScript as usize].currentState = sp.arg1;
                    return rumbleScripts[whichScript as usize].states[sp.arg1 as usize].timeToStop;
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
                let temp = &mut *(
                    rumbleScripts[whichScript as usize].states.add(sp.arg1 as usize)
                        as *mut rumblestate_special_t
                );
                temp.arg2 -= sp.arg2;
            }

            // Increase Arg2 of a State,		sp->arg1 = state, sp->arg2 = amount to increase arg2 of state by
            IN_CMD_INC_ARG2 => {
                let temp = &mut *(
                    rumbleScripts[whichScript as usize].states.add(sp.arg1 as usize)
                        as *mut rumblestate_special_t
                );
                temp.arg2 += sp.arg2;
            }

            // Decreasae Arg1 of a State,		sp->arg1 = state, sp->arg2 = amount to decrease arg1 of state by
            IN_CMD_DEC_ARG1 => {
                let temp = &mut *(
                    rumbleScripts[whichScript as usize].states.add(sp.arg1 as usize)
                        as *mut rumblestate_special_t
                );
                temp.arg1 -= sp.arg2;
            }

            // Increase Arg2 of a State,		sp->arg1 = state, sp->arg2 = amount to increase arg1 of state by
            IN_CMD_INC_ARG1 => {
                let temp = &mut *(
                    rumbleScripts[whichScript as usize].states.add(sp.arg1 as usize)
                        as *mut rumblestate_special_t
                );
                temp.arg1 += sp.arg2;
            }

            IN_CMD_DEC_LEFT => {
                rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg2 -= sp.arg2;
                if rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg2 < 0 {
                    rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg2 = 0;
                }
                if rumbleScripts[whichScript as usize].currentState
                    >= rumbleScripts[whichScript as usize].usedStates - 1
                {
                    return -2; // Done
                }
                rumbleScripts[whichScript as usize].currentState += 1;
                return rumbleScripts[whichScript as usize].states
                    [rumbleScripts[whichScript as usize].currentState as usize]
                    .timeToStop;
            }

            IN_CMD_DEC_RIGHT => {
                rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg1 -= sp.arg2;
                if rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg1 < 0 {
                    rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg1 = 0;
                }
                if rumbleScripts[whichScript as usize].currentState
                    >= rumbleScripts[whichScript as usize].usedStates - 1
                {
                    return -2; // Done
                }
                rumbleScripts[whichScript as usize].currentState += 1;
                return rumbleScripts[whichScript as usize].states
                    [rumbleScripts[whichScript as usize].currentState as usize]
                    .timeToStop;
            }

            IN_CMD_INC_LEFT => {
                rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg2 += sp.arg2;
                if rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg2 > 65534 {
                    rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg2 = 65534;
                }
                if rumbleScripts[whichScript as usize].currentState
                    >= rumbleScripts[whichScript as usize].usedStates - 1
                {
                    return -2; // Done
                }
                rumbleScripts[whichScript as usize].currentState += 1;
                return rumbleScripts[whichScript as usize].states
                    [rumbleScripts[whichScript as usize].currentState as usize]
                    .timeToStop;
            }

            IN_CMD_INC_RIGHT => {
                rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg1 += sp.arg2;
                if rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg1 > 65534 {
                    rumbleScripts[whichScript as usize].states[sp.arg1 as usize].arg1 = 65534;
                }
                if rumbleScripts[whichScript as usize].currentState
                    >= rumbleScripts[whichScript as usize].usedStates - 1
                {
                    return -2; // Done
                }
                rumbleScripts[whichScript as usize].currentState += 1;
                return rumbleScripts[whichScript as usize].states
                    [rumbleScripts[whichScript as usize].currentState as usize]
                    .timeToStop;
            }
            _ => return 0,
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

    unsafe {
        assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);

        // Can't execute an empty script???
        assert!(rumbleScripts[whichScript as usize].usedStates > 0);

        rumbleScripts[whichScript as usize].currentState = 0;
        let mut cmd = rumbleScripts[whichScript as usize].states
            [rumbleScripts[whichScript as usize].currentState as usize]
            .timeToStop;
        if cmd < 0 {
            cmd = IN_RunSpecialScript(whichScript);
        }

        rumbleScripts[whichScript as usize].nextStateAt = IN_Time() + cmd;

        rumbleStatus[rumbleScripts[whichScript as usize].controller as usize].changed = true;
        rumbleStatus[rumbleScripts[whichScript as usize].controller as usize].killed = false;
    }
}

pub fn IN_PauseRumbling_controller(controller: c_int) {
    if !IN_usingRumble() {
        return;
    }
    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return;
    }

    unsafe {
        if rumbleStatus[controller as usize].paused == true {
            return;
        }

        rumbleStatus[controller as usize].timePaused = IN_Time();
        rumbleStatus[controller as usize].paused = IN_RumbleAdjust(controller, 0, 0);
    }
}

pub fn IN_UnPauseRumbling_controller(controller: c_int) {
    if !IN_usingRumble() {
        return;
    }
    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return;
    }

    unsafe {
        // can't unpause a control that wasn't paused
        if rumbleStatus[controller as usize].paused == false {
            return;
        }

        let cur_time = IN_Time();
        for i in 0..MAX_RUMBLE_SCRIPTS {
            if rumbleScripts[i].controller == controller {
                if rumbleScripts[i].nextStateAt == 0 {
                    continue;
                }
                // update the time to stop based on how long it was paused
                rumbleScripts[i].nextStateAt +=
                    cur_time - rumbleStatus[controller as usize].timePaused;
            }
        }

        rumbleStatus[controller as usize].paused = false;
        rumbleStatus[controller as usize].changed = true;
        rumbleStatus[controller as usize].killed = false;
    }
}

pub fn IN_TogglePauseRumbling_controller(controller: c_int) {
    if !IN_usingRumble() {
        return;
    }
    if controller <= -1 || controller >= MAX_RUMBLE_CONTROLLERS as c_int {
        return;
    }

    unsafe {
        if rumbleStatus[controller as usize].paused {
            IN_UnPauseRumbling_controller(controller);
        } else {
            IN_PauseRumbling_controller(controller);
        }
    }
}

// Pauses rumbling on all controllers
pub fn IN_PauseRumbling() {
    if !IN_usingRumble() {
        return;
    }
    for i in 0..MAX_RUMBLE_CONTROLLERS {
        IN_PauseRumbling_controller(i as c_int);
    }
}

// UnPauses rumbling on all controllers
pub fn IN_UnPauseRumbling() {
    if !IN_usingRumble() {
        return;
    }
    for i in 0..MAX_RUMBLE_CONTROLLERS {
        IN_UnPauseRumbling_controller(i as c_int);
    }
}

// Toggles Pausing on all controllers
pub fn IN_TogglePauseRumbling() {
    if !IN_usingRumble() {
        return;
    }
    for i in 0..MAX_RUMBLE_CONTROLLERS {
        IN_TogglePauseRumbling_controller(i as c_int);
    }
}

// Returns false when the end of the script is reached
pub fn IN_AdvanceToNextState(whichScript: c_int) -> bool {
    unsafe {
        assert!(whichScript >= 0 && whichScript < MAX_RUMBLE_SCRIPTS as c_int);

        if rumbleScripts[whichScript as usize].currentState
            >= rumbleScripts[whichScript as usize].usedStates - 1
        {
            // Script is at its end, so kill it( which deletes only if autodelete
            IN_KillRumbleScript(whichScript);
            return false;
        }

        // Advance a state
        rumbleScripts[whichScript as usize].currentState += 1;

        let mut cmd = rumbleScripts[whichScript as usize].states
            [rumbleScripts[whichScript as usize].currentState as usize]
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

        rumbleScripts[whichScript as usize].nextStateAt = IN_Time() + cmd;
        return true;
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
        usingRumble[0] = Cvar_VariableIntegerValue(b"in_useRumble\0".as_ptr() as *const core::ffi::c_char);
        usingRumble[1] = Cvar_VariableIntegerValue(b"in_useRumble2\0".as_ptr() as *const core::ffi::c_char);

        let mut value: [[c_int; 2]; MAX_RUMBLE_CONTROLLERS] = [[0; 2]; MAX_RUMBLE_CONTROLLERS];
        let cur_time = IN_Time();

        for i in 0..MAX_RUMBLE_SCRIPTS {
            // If rumble is paused on current controller than skip this rumble state
            if rumbleStatus[rumbleScripts[i].controller as usize].paused {
                continue;
            }

            //*mb	ClientManager::ActivateByControllerId(rumbleScripts[i].controller);
            if usingRumble[ActiveClientNum() as usize] == 0 {
                IN_KillRumbleScript(i as c_int);
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
            if rumbleScripts[i].nextStateAt == 0 {
                continue;
            }

            // Time is up on this rumble state
            if rumbleScripts[i].nextStateAt < cur_time {
                // If timeToStop is < cur_time and > 0 then end this state otherwise (negative number) always rumble
                if rumbleScripts[i].nextStateAt > 0 {
                    rumbleStatus[rumbleScripts[i].controller as usize].changed = true;
                    rumbleStatus[rumbleScripts[i].controller as usize].killed = false;
                    if !IN_AdvanceToNextState(i as c_int) {
                        // Returns false if reached the end of script
                        continue;
                    }
                }
            }

            let curScript = &rumbleScripts[i];

            if value[curScript.controller as usize][0] < curScript.states[curScript.currentState as usize].arg2
            {
                value[curScript.controller as usize][0] =
                    curScript.states[curScript.currentState as usize].arg2;
            }
            if value[curScript.controller as usize][1] < curScript.states[curScript.currentState as usize].arg1
            {
                value[curScript.controller as usize][1] =
                    curScript.states[curScript.currentState as usize].arg1;
            }
        }

        // Go through the 4 controller ports
        for i in 0..MAX_RUMBLE_CONTROLLERS {
            // paused, so do nothing for this controller
            if rumbleStatus[i].paused {
                continue;
            }

            // Only update the actual hardware if a state has changed
            if !rumbleStatus[i].changed {
                continue;
            }

            IN_RumbleAdjust(i as c_int, value[i][0], value[i][1]);

            // State has changed
            rumbleStatus[i].changed = false;
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
        // memset equivalent - already zero-initialized by static mut
        for i in 0..MAX_RUMBLE_CONTROLLERS {
            rumbleStatus[i] = rumblestatus_t::default();
        }
        for i in 0..MAX_RUMBLE_SCRIPTS {
            rumbleScripts[i] = rumblescript_t::default();
        }

        in_useRumble = Cvar_Get(
            b"in_useRumble\0".as_ptr() as *const core::ffi::c_char,
            b"1\0".as_ptr() as *const core::ffi::c_char,
            0,
        );
        Cvar_Get(
            b"in_useRumble2\0".as_ptr() as *const core::ffi::c_char,
            b"1\0".as_ptr() as *const core::ffi::c_char,
            0,
        );
    }
}

/*
==================
IN_RumbleShutdown
==================
*/
pub fn IN_RumbleShutdown() {
    unsafe {
        for i in 0..MAX_RUMBLE_SCRIPTS {
            if !rumbleScripts[i].states.is_null() {
                let _ = Box::from_raw(core::slice::from_raw_parts_mut(
                    rumbleScripts[i].states,
                    rumbleScripts[i].numStates as usize,
                ));
            }
            rumbleScripts[i].states = core::ptr::null_mut();
            rumbleScripts[i].nextStateAt = 0;
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
        if (*cl_paused).integer != 0
            && !rumbleStatus[IN_GetMainController() as usize].paused
        {
            IN_PauseRumbling_controller(IN_GetMainController());
        } else if (*cl_paused).integer == 0 && rumbleStatus[IN_GetMainController() as usize].paused
        {
            IN_UnPauseRumbling_controller(IN_GetMainController());
        }

        // Update the states
        IN_UpdateRumbleFromStates();
    }
}
