// #include "../server/exe_headers.h"

// #include "../client/client.h"
// #include "../qcommon/qcommon.h"
// #ifdef _JK2MP
// #include "../ui/keycodes.h"
// #else
// #include "../client/keycodes.h"
// #endif

// #include "win_local.h"
// #include "win_input.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// EXTERNAL FUNCTION DECLARATIONS
extern "C" {
    // From qcommon/qcommon.h
    static mut cls: ClientState;

    // From client/client.h or similar
    static mut cl_thumbStickMode: *mut Cvar_t;

    fn Sys_Milliseconds() -> c_int;
    fn Sys_QueEvent(time: c_int, event_type: c_int, value: c_int, value2: c_int, len: c_int, data: *mut c_void);
    fn Cvar_VariableIntegerValue(cvar: *const c_char) -> c_int;
    fn Cvar_Get(cvar: *const c_char, default_val: *const c_char, flags: c_int) -> *mut Cvar_t;
    fn Cvar_Set(cvar: *const c_char, value: *const c_char);
    fn Cvar_SetValue(cvar: *const c_char, value: f32);
    fn Cvar_VariableValue(cvar: *const c_char) -> f32;
    fn Cbuf_ExecuteText(exec_mode: c_int, text: *const c_char);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Key_GetCatcher() -> c_int;
    fn Z_CompactStats();
    fn Sys_Reboot(reason: *const c_char);
    fn va(fmt: *const c_char, ...) -> *const c_char;
}

// LOCAL TYPE STUBS
#[repr(C)]
pub struct ClientState {
    // Stub: minimal fields for this translation
    pub state: c_int,
    pub mainGamepad: c_int,
}

#[repr(C)]
pub struct Cvar_t {
    // Stub: minimal fields
    pub integer: c_int,
}

// TYPES FROM win_input.h
pub type fakeAscii_t = c_int;

#[repr(C)]
pub struct JoyInfo {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct PadInfo {
    pub padId: c_int,
    pub joyInfo: [JoyInfo; 2],
}

// GLOBALS
static mut _UIRunning: bool = false;

// Comment out next line to turn off debug controller. This should be
// forced off in FINAL_BUILD, but for now, I want to send builds to
// Activision made with FINAL_BUILD but that support the cheatpad.
#[cfg(feature = "DEBUG_CONTROLLER")]
const _DEBUG_CONTROLLER: bool = true;

// Controller connection globals
static mut uiControllerNotification: i8 = -1;
pub static mut noControllersConnected: bool = false;
pub static mut wasPlugged: [bool; 4] = [false; 4];

pub static mut _padInfo: PadInfo = PadInfo {
    padId: 0,
    joyInfo: [JoyInfo { x: 0.0, y: 0.0 }; 2],
}; // gamepad thumbstick buffer



//If the Xbox white or black button was held for less than this amount of
//time while a selection bar was up, the user wants to use the button rather
//than reassign it.
const MAX_WB_HOLD_TIME: c_int = 500;

// Keycode constants for UIJoy2Key mapping
const A_JOY1: c_int = 1;
const A_JOY2: c_int = 2;
const A_JOY3: c_int = 3;
const A_JOY4: c_int = 4;
const A_JOY5: c_int = 5;
const A_JOY6: c_int = 6;
const A_JOY7: c_int = 7;
const A_JOY8: c_int = 8;
const A_JOY11: c_int = 11;
const A_JOY12: c_int = 12;
const A_JOY14: c_int = 14;
const A_JOY15: c_int = 15;
const A_JOY16: c_int = 16;
const A_CURSOR_DOWN: c_int = 0x100;
const A_CURSOR_UP: c_int = 0x101;
const A_CURSOR_LEFT: c_int = 0x102;
const A_CURSOR_RIGHT: c_int = 0x103;
const A_PAGE_UP: c_int = 0x104;
const A_PAGE_DOWN: c_int = 0x105;
const A_MOUSE1: c_int = 1;
const A_ESCAPE: c_int = 27;
const A_DELETE: c_int = 127;
const A_SPACE: c_int = 32;
const A_F1: c_int = 0x70;
const A_F2: c_int = 0x71;
const A_F3: c_int = 0x72;

// Event type constants
const SE_KEY: c_int = 0;
const SE_MOUSE: c_int = 1;
const SE_JOYSTICK_AXIS: c_int = 2;

// Axis constants
const AXIS_SIDE: c_int = 0;
const AXIS_FORWARD: c_int = 1;

// Client state constants
const CA_LOADING: c_int = 0;
const CA_CONNECTING: c_int = 1;
const CA_CONNECTED: c_int = 2;
const CA_CHALLENGING: c_int = 3;
const CA_PRIMED: c_int = 4;
const CA_CINEMATIC: c_int = 5;

// Key catcher constants
const KEYCATCH_UI: c_int = 1;

// Cvar exec mode constants
const EXEC_NOW: c_int = 0;
const EXEC_APPEND: c_int = 1;

fn UIJoy2Key(button: fakeAscii_t) -> fakeAscii_t {
    match button {
        A_JOY7 => A_CURSOR_DOWN,
        A_JOY5 => A_CURSOR_UP,
        A_JOY6 => A_CURSOR_RIGHT,
        A_JOY8 => A_CURSOR_LEFT,
        A_JOY15 => A_MOUSE1,
        #[cfg(target_os = "gamecube")]
        A_JOY16 => A_ESCAPE,
        #[cfg(target_os = "gamecube")]
        A_JOY14 => A_DELETE,
        #[cfg(not(target_os = "gamecube"))]
        A_JOY14 => A_ESCAPE,
        #[cfg(not(target_os = "gamecube"))]
        A_JOY16 => A_DELETE,

        //left and right trigger for scrolling
        A_JOY11 => A_PAGE_UP,
        A_JOY12 => A_PAGE_DOWN,

        // start and back button on xbox
        A_JOY1 => {
            //JLF
            A_ESCAPE
        },
        A_JOY2 | A_JOY4 => {
            //JLF
            A_MOUSE1
            //return button;
        },

        A_JOY3 => A_MOUSE1,
        _ => A_SPACE, //Invalid button.
    }
}

#[repr(C)]
struct UIKeyQueueEntry {
    button: c_int,
    pressed: bool,
}

static mut uiKeyQueue: [[UIKeyQueueEntry; 5]; 2] = [[UIKeyQueueEntry { button: 0, pressed: false }; 5]; 2];
static mut uiQueueLen: [c_int; 2] = [0; 2];
static mut uiLastKeyUpDown: [c_int; 2] = [0; 2];
static mut uiLastKeyLeftRight: [c_int; 2] = [0; 2];

pub fn IN_UIEmptyQueue() {
    unsafe {
        /// If the ui is not running then this doesn't have any effect
        if !_UIRunning {
            uiQueueLen[0] = 0;
            uiQueueLen[1] = 0;
            return;
        }

        // BTO - No CM, bypass that logic.
        //	for (int i = 0; i < ClientManager::NumClients(); i++)
        for i in 0..1 {
            //		ClientManager::ActivateClient(i);
            let mut found: c_int = 0;
            let mut bCancel: c_int = 0;
            for j in 0..(uiQueueLen[i] as usize) {
                match uiKeyQueue[i][j].button {
                    A_CURSOR_DOWN | A_CURSOR_UP => {
                        if (found & 2) != 0 { // Was a left/right key pressed already?
                            bCancel = 1;
                        }
                        found |= 1;
                    },
                    A_CURSOR_RIGHT | A_CURSOR_LEFT => {
                        if (found & 1) != 0 { // Was an up/down key already pressed?
                            bCancel = 1;
                        }
                        found |= 2;
                    },
                    _ => {},
                }
            }

            if bCancel == 0 { // was it cancelled?
                for j in 0..(uiQueueLen[i] as usize) {
                    let time: c_int = Sys_Milliseconds();
                    match uiKeyQueue[i][j].button {
                        A_CURSOR_DOWN | A_CURSOR_UP => {
                            if uiLastKeyLeftRight[i] != 0 {
                                if uiLastKeyLeftRight[i] > time { // don't allow up/down till left/right has enough leway time
                                    continue;
                                }
                            }
                            uiLastKeyUpDown[i] = time + 150; // 250 ms sound right?
                        },
                        A_CURSOR_LEFT | A_CURSOR_RIGHT => {
                            if uiLastKeyUpDown[i] != 0 {
                                if uiLastKeyUpDown[i] > time { // don't allow up/down till left/right has enough leway time
                                    continue;
                                }
                            }
                            uiLastKeyLeftRight[i] = time + 150; // 250 ms sound right?
                        },
                        _ => {},
                    }
                    Sys_QueEvent(0, SE_KEY, uiKeyQueue[i][j].button, uiKeyQueue[i][j].pressed as c_int, 0, core::ptr::null_mut());
                }
            }
        }

        // Reset the queue
        uiQueueLen[0] = 0;
        uiQueueLen[1] = 0;
    }
}

// extern void G_DemoKeypress();
// extern void CG_SkipCredits(void);
pub fn IN_CommonJoyPress(controller: c_int, button: fakeAscii_t, pressed: bool) {
    unsafe {
        // Check for special cases for map hack
        // This should be #ifdef'd out in FINAL_BUILD, but I really don't care.
        // If someone wants to copy the retail version to their modded xbox and
        // edit the config file to turn on maphack, let them.
        if Cvar_VariableIntegerValue(b"cl_maphack\0".as_ptr() as *const c_char) != 0 {
            if _UIRunning && button == A_JOY11 && pressed {
                // Left trigger -> F1
                Sys_QueEvent(0, SE_KEY, A_F1, pressed as c_int, 0, core::ptr::null_mut());
                return;
            } else if _UIRunning && button == A_JOY12 && pressed {
                // Right trigger -> F2
                Sys_QueEvent(0, SE_KEY, A_F2, pressed as c_int, 0, core::ptr::null_mut());
                return;
            } else if _UIRunning && button == A_JOY4 && pressed {
                // Start button -> F3
                IN_SetMainController(controller);
                Sys_QueEvent(0, SE_KEY, A_F3, pressed as c_int, 0, core::ptr::null_mut());
                return;
            }
        }


        if IN_GetMainController() == controller || _UIRunning {
            // Always map start button to ESCAPE
            if !_UIRunning && button == A_JOY4 && cls.state != CA_CINEMATIC {
                Sys_QueEvent(0, SE_KEY, A_ESCAPE, pressed as c_int, 0, core::ptr::null_mut());
            }

            #[cfg(feature = "DEBUG_CONTROLLER")]
            if controller != 3 {
                Sys_QueEvent(0, SE_KEY, if _UIRunning { UIJoy2Key(button) } else { button }, pressed as c_int, 0, core::ptr::null_mut());
            }
            #[cfg(not(feature = "DEBUG_CONTROLLER"))]
            {
                Sys_QueEvent(0, SE_KEY, if _UIRunning { UIJoy2Key(button) } else { button }, pressed as c_int, 0, core::ptr::null_mut());
            }
        }

        #[cfg(feature = "DEBUG_CONTROLLER")]
        if controller == 3 && pressed {
            HandleDebugJoystickPress(button);
            return;
        }
    }
}

pub static mut g_noCheckAxis: bool = false;

/**********
IN_CommonUpdate
Updates thumbstick events based on _padInfo and ui_thumbStickMode
**********/
pub fn IN_CommonUpdate() {
    unsafe {
        _UIRunning = Key_GetCatcher() == KEYCATCH_UI;

        // if the UI is running, then let all gamepad sticks work, else only main controller
        if _UIRunning {
            Sys_QueEvent(0, SE_MOUSE, (_padInfo.joyInfo[1].x * 4.0) as c_int, (_padInfo.joyInfo[1].y * -4.0) as c_int, 0, core::ptr::null_mut());
        } else if _padInfo.padId == IN_GetMainController() {
            // Find out how to configure the thumbsticks
            //int thumbStickMode = Cvar_Get("ui_thumbStickMode", "0" , 0)->integer;
            let thumbStickMode: c_int = (*cl_thumbStickMode).integer;

            match thumbStickMode {
                0 => {
                    // Configure left thumbstick to move forward/back & strafe left/right
                    Sys_QueEvent(0, SE_JOYSTICK_AXIS, AXIS_SIDE, (_padInfo.joyInfo[0].x * 127.0) as c_int, 0, core::ptr::null_mut());
                    Sys_QueEvent(0, SE_JOYSTICK_AXIS, AXIS_FORWARD, (_padInfo.joyInfo[0].y * 127.0) as c_int, 0, core::ptr::null_mut());

                    // Configure right thumbstick for freelook
                    Sys_QueEvent(0, SE_MOUSE, (_padInfo.joyInfo[1].x * 48.0) as c_int, (_padInfo.joyInfo[1].y * 48.0) as c_int, 0, core::ptr::null_mut());
                },
                1 => {
                    // Configure left thumbstick for freelook
                    Sys_QueEvent(0, SE_MOUSE, (_padInfo.joyInfo[0].x * 48.0) as c_int, (_padInfo.joyInfo[0].y * 48.0) as c_int, 0, core::ptr::null_mut());

                    // Configure right thumbstick to move forward/back & strafe left/right
                    Sys_QueEvent(0, SE_JOYSTICK_AXIS, AXIS_SIDE, (_padInfo.joyInfo[1].x * 127.0) as c_int, 0, core::ptr::null_mut());
                    Sys_QueEvent(0, SE_JOYSTICK_AXIS, AXIS_FORWARD, (_padInfo.joyInfo[1].y * 127.0) as c_int, 0, core::ptr::null_mut());
                },
                2 => {
                    // Configure left thumbstick to move forward/back & turn left/right
                    Sys_QueEvent(0, SE_JOYSTICK_AXIS, AXIS_FORWARD, (_padInfo.joyInfo[0].y * 127.0) as c_int, 0, core::ptr::null_mut());
                    Sys_QueEvent(0, SE_MOUSE, (_padInfo.joyInfo[0].x * 48.0) as c_int, 0, 0, core::ptr::null_mut());

                    // Configure right thumbstick to look up/down & strafe left/right
                    Sys_QueEvent(0, SE_JOYSTICK_AXIS, AXIS_SIDE, (_padInfo.joyInfo[1].x * 127.0) as c_int, 0, core::ptr::null_mut());
                    Sys_QueEvent(0, SE_MOUSE, 0, (_padInfo.joyInfo[1].y * 48.0) as c_int, 0, core::ptr::null_mut());
                },
                3 => {
                    // Configure left thumbstick to look up/down & strafe left/right
                    Sys_QueEvent(0, SE_JOYSTICK_AXIS, AXIS_SIDE, (_padInfo.joyInfo[0].x * 127.0) as c_int, 0, core::ptr::null_mut());
                    Sys_QueEvent(0, SE_MOUSE, 0, (_padInfo.joyInfo[0].y * 48.0) as c_int, 0, core::ptr::null_mut());

                    // Configure right thumbstick to move forward/back & turn left/right
                    Sys_QueEvent(0, SE_JOYSTICK_AXIS, AXIS_FORWARD, (_padInfo.joyInfo[1].y * 127.0) as c_int, 0, core::ptr::null_mut());
                    Sys_QueEvent(0, SE_MOUSE, (_padInfo.joyInfo[1].x * 48.0) as c_int, 0, 0, core::ptr::null_mut());
                },
                _ => {},
            }
        }
    }
}

/*********
IN_DisplayControllerUnplugged
*********/
fn IN_DisplayControllerUnplugged(controller: c_int) {
    unsafe {
        uiControllerNotification = controller as i8;

        //TODO Add a call to the UI that draws a controller disconnected message
        // on the screen.
        //	VM_Call( uivm, UI_CONTROLLER_UNPLUGGED, true, controller);
    }
}

/*********
IN_ClearControllerUnplugged
*********/
fn IN_ClearControllerUnplugged() {
    unsafe {
        uiControllerNotification = -1;

        //TODO Add a call to the UI that removes the controller disconnected
        // message from the screen.
        //	VM_Call( uivm, UI_CONTROLLER_UNPLUGGED, false, 0);
    }
}

/*********
IN_ControllerMustBePlugged
*********/
fn IN_ControllerMustBePlugged(controller: c_int) -> bool {
    unsafe {
        if cls.state == CA_LOADING ||
            cls.state == CA_CONNECTING ||
            cls.state == CA_CONNECTED ||
            cls.state == CA_CHALLENGING ||
            cls.state == CA_PRIMED ||
            cls.state == CA_CINEMATIC {
            return false;
        }

        if !_UIRunning && controller == IN_GetMainController() {
            return true;
        }

        if noControllersConnected {
            return true;
        }

        false
    }
}

/*********
IN_PadUnplugged
*********/
pub fn IN_PadUnplugged(controller: c_int) {
    unsafe {
        if wasPlugged[controller as usize] {
            Com_Printf(b"\tController %d unplugged\n\0".as_ptr() as *const c_char, controller);
        }

        if IN_ControllerMustBePlugged(controller) {
            //If UI isn't busy, inform it about controller loss.
            if uiControllerNotification == -1 {
                IN_DisplayControllerUnplugged(controller);
            }
        }
        wasPlugged[controller as usize] = false;
    }
}

/*********
IN_PadPlugged
*********/
pub fn IN_PadPlugged(controller: c_int) {
    unsafe {
        if !wasPlugged[controller as usize] {
            Com_Printf(b"\tController %d plugged\n\0".as_ptr() as *const c_char, controller);
        }

        if IN_ControllerMustBePlugged(controller) {
            //If UI is dealing with this controller, tell it to stop.
            if uiControllerNotification == controller as i8 {
                IN_ClearControllerUnplugged();
            }
        }
        wasPlugged[controller as usize] = true;
        noControllersConnected = false;
    }
}

/*********
IN_GetMainController
*********/
pub fn IN_GetMainController() -> c_int {
    unsafe {
        cls.mainGamepad
    }
}

/*********
IN_SetMainController
*********/
pub fn IN_SetMainController(id: c_int) {
    unsafe {
        cls.mainGamepad = id;
    }
}

/*********
IN_SetThumbStickConfig
Sets the thumbstick configuration value
*********/
pub fn IN_SetThumbStickConfig(configValue: c_int) {
    unsafe {
        let config_str = va(b"%i\0".as_ptr() as *const c_char, configValue);
        Cvar_Set(b"ui_thumbStickMode\0".as_ptr() as *const c_char, config_str);
    }
}

/*********
IN_SetButtonConfig
Execs a button configuration script based on configValue
*********/
pub fn IN_SetButtonConfig(configValue: c_int) {
    unsafe {
        // Set the cvar
        let config_str = va(b"%i\0".as_ptr() as *const c_char, configValue);
        Cvar_Set(b"ui_buttonMode\0".as_ptr() as *const c_char, config_str);

        // Exec the script
        let exec_string = va(b"exec cfg\\buttonConfig%i.cfg\n\0".as_ptr() as *const c_char, configValue);
        Cbuf_ExecuteText(EXEC_NOW, exec_string);
    }
}

/*********
IN_SetDpadConfig
Execs a dpad configuration script based on configValue
*********/
pub fn IN_SetDpadConfig(configValue: c_int) {
    unsafe {
        // Set the cvar
        let config_str = va(b"%i\0".as_ptr() as *const c_char, configValue);
        Cvar_Set(b"ui_dpadMode\0".as_ptr() as *const c_char, config_str);

        // Exec the script
        let exec_string = va(b"exec cfg\\dpadConfig%i.cfg\n\0".as_ptr() as *const c_char, configValue);
        Cbuf_ExecuteText(EXEC_NOW, exec_string);
    }
}

/**********************************************************
*
* DEBUGGING CODE
*
**********************************************************/

#[cfg(feature = "DEBUG_CONTROLLER")]
fn HandleDebugJoystickPress(button: fakeAscii_t) {
    unsafe {
        // Super hackalicious crap used below. Please remove this at some point.
        static mut curSaberSet: c_int = 0;
        static mut curPlayerSet: c_int = 0;
        static mut dpadmode: i16 = 0;
        static mut buttonmode: i16 = 0;
        static mut thumbmode: i16 = 0;

        const A_JOY9: c_int = 9;   // White button
        const A_JOY10: c_int = 10; // Black button
        const A_JOY13: c_int = 13; // Right pad up

        match button {
            A_JOY13 => { // Right pad up
                Cbuf_ExecuteText(EXEC_APPEND, b"give all\n\0".as_ptr() as *const c_char);
            },
            A_JOY16 => { // Right pad left
                Cbuf_ExecuteText(EXEC_APPEND, b"viewpos\n\0".as_ptr() as *const c_char);
            },
            A_JOY14 => { // Right pad right
                Cbuf_ExecuteText(EXEC_APPEND, b"noclip\n\0".as_ptr() as *const c_char);
            },
            A_JOY15 => { // Right pad down
                Cbuf_ExecuteText(EXEC_APPEND, b"god\n\0".as_ptr() as *const c_char);
            },
            A_JOY4 => { // Start
                Cvar_SetValue(b"m_pitch\0".as_ptr() as *const c_char, -Cvar_VariableValue(b"m_pitch\0".as_ptr() as *const c_char));
            },
            A_JOY1 => { // back
                Cvar_SetValue(b"cl_autolevel\0".as_ptr() as *const c_char, if Cvar_VariableIntegerValue(b"cl_autolevel\0".as_ptr() as *const c_char) != 0 { 0.0 } else { 1.0 });
            },
            A_JOY2 => { // Left thumbstick
                Z_CompactStats();
            },
            A_JOY12 => { // Upper right trigger
                Cbuf_ExecuteText(EXEC_APPEND, b"load dbg-game\n\0".as_ptr() as *const c_char);
            },
            A_JOY8 => { // Left pad left
                thumbmode += 1;
                if thumbmode == 4 {
                    thumbmode = 0;
                }
                IN_SetThumbStickConfig(thumbmode as c_int);
            },
            A_JOY6 => { // Left pad right
                dpadmode += 1;
                if dpadmode == 4 {
                    dpadmode = 0;
                }
                IN_SetDpadConfig(0);
            },
            A_JOY5 => { // Left pad up
                buttonmode += 1;
                if buttonmode == 4 {
                    buttonmode = 0;
                }
                IN_SetButtonConfig(buttonmode as c_int);
            },
            A_JOY7 => { // Left pad down
                //		Cbuf_ExecuteText(EXEC_APPEND, "vid_restart\n");
                Sys_Reboot(b"multiplayer\0".as_ptr() as *const c_char);
            },
            A_JOY11 => { // Upper left trigger
                Cbuf_ExecuteText(EXEC_APPEND, b"save dbg-game\n\0".as_ptr() as *const c_char);
            },
            A_JOY9 => { // White button
                // Hacky. Really hacky. No, hackier than that.
                curSaberSet = (curSaberSet + 1) % 3; // Number of xsaber strings in config file
                let cmd_str = va(b"vstr xsaber%d\n\0".as_ptr() as *const c_char, curSaberSet);
                Cbuf_ExecuteText(EXEC_APPEND, cmd_str);
            },
            A_JOY10 => { // Black button
                curPlayerSet = (curPlayerSet + 1) % 6; // Number of xplayer strings in config file
                let cmd_str = va(b"vstr xplayer%d\n\0".as_ptr() as *const c_char, curPlayerSet);
                Cbuf_ExecuteText(EXEC_APPEND, cmd_str);
            },
            _ => {},
        }
    }
}

#[cfg(not(feature = "DEBUG_CONTROLLER"))]
fn HandleDebugJoystickPress(_button: fakeAscii_t) {
    // Stub when DEBUG_CONTROLLER is not defined
}
