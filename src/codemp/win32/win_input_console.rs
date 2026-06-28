// #include "../server/exe_headers.h"

use core::ffi::{c_int, c_void};

// #include "../client/client.h"
// #include "../qcommon/qcommon.h"
// #ifdef _JK2MP
// #include "../ui/keycodes.h"
// #else
// #include "../client/keycodes.h"
// #endif

// #include "win_local.h"
// #include "win_input.h"

// Stub type declarations
#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // Placeholder; actual structure would come from header files
}

#[repr(C)]
pub struct clientActive_t {
    pub state: c_int,
    pub mainGamepad: c_int,
    // Other fields not relevant to this file
}

// Stub type for PadInfo
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

// Type alias for keyboard input codes
type fakeAscii_t = c_int;

// Stub declarations for external functions
extern "C" {
    fn Key_GetCatcher() -> c_int;
    fn Sys_Milliseconds() -> c_int;
    fn Sys_QueEvent(
        time: c_int,
        type_: c_int,
        value: c_int,
        value2: c_int,
        ptrLength: c_int,
        ptr: *mut c_void,
    );
    fn Cvar_VariableIntegerValue(var_name: *const core::ffi::c_char) -> c_int;
    fn Cvar_Get(
        var_name: *const core::ffi::c_char,
        var_value: *const core::ffi::c_char,
        flags: c_int,
    ) -> *mut cvar_t;
    fn Cvar_Set(var_name: *const core::ffi::c_char, value: *const core::ffi::c_char);
    fn Cvar_SetValue(var_name: *const core::ffi::c_char, value: f32);
    fn Cvar_VariableValue(var_name: *const core::ffi::c_char) -> f32;
    fn Com_Printf(format: *const core::ffi::c_char, ...);
    fn Cbuf_ExecuteText(exec_when: c_int, text: *const core::ffi::c_char);
    fn va(format: *const core::ffi::c_char, ...) -> *const core::ffi::c_char;
    fn Z_CompactStats();

    // Global struct
    #[allow(non_upper_case_globals)]
    pub static mut cls: clientActive_t;
}

fn HandleDebugJoystickPress(button: fakeAscii_t);

static mut _UIRunning: bool = false;

fn IN_ControllerMustBePlugged(controller: c_int) -> bool;

// #ifdef _DEBUG
// bool cheatPadEnabled = 1;
// #else
// bool cheatPadEnabled = 1;
// #endif
static mut cheatPadEnabled: bool = true;

// Controller connection globals
static mut uiControllerNotification: i8 = -1;
static mut noControllersConnected: bool = false;
static mut wasPlugged: [bool; 4] = [false; 4];

pub static mut _padInfo: PadInfo = PadInfo {
    padId: 0,
    joyInfo: [JoyInfo { x: 0.0, y: 0.0 }, JoyInfo { x: 0.0, y: 0.0 }],
}; // gamepad thumbstick buffer

//If the Xbox white or black button was held for less than this amount of
//time while a selection bar was up, the user wants to use the button rather
//than reassign it.
const MAX_WB_HOLD_TIME: c_int = 500;

fn UIJoy2Key(button: fakeAscii_t) -> fakeAscii_t {
    // Placeholder button values - these come from keycodes.h
    const A_JOY7: c_int = 7;
    const A_CURSOR_DOWN: c_int = 1001;
    const A_JOY5: c_int = 5;
    const A_CURSOR_UP: c_int = 1002;
    const A_JOY6: c_int = 6;
    const A_CURSOR_RIGHT: c_int = 1003;
    const A_JOY8: c_int = 8;
    const A_CURSOR_LEFT: c_int = 1004;
    const A_JOY15: c_int = 15;
    const A_MOUSE1: c_int = 1;
    const A_JOY16: c_int = 16;
    const A_ESCAPE: c_int = 27;
    const A_JOY14: c_int = 14;
    const A_DELETE: c_int = 330;
    const A_JOY13: c_int = 13;
    const A_BACKSPACE: c_int = 8;
    const A_JOY11: c_int = 11;
    const A_PAGE_UP: c_int = 1007;
    const A_JOY12: c_int = 12;
    const A_PAGE_DOWN: c_int = 1008;
    const A_JOY1: c_int = 1;
    const A_JOY2: c_int = 2;
    const A_JOY4: c_int = 4;
    const A_JOY3: c_int = 3;
    const A_SPACE: c_int = 32;

    match button {
        A_JOY7 => A_CURSOR_DOWN,
        A_JOY5 => A_CURSOR_UP,
        A_JOY6 => A_CURSOR_RIGHT,
        A_JOY8 => A_CURSOR_LEFT,
        A_JOY15 => A_MOUSE1,
        // #ifdef _GAMECUBE
        A_JOY16 => A_ESCAPE,
        A_JOY14 => A_DELETE,
        // #else
        // A_JOY14 => A_ESCAPE,
        // A_JOY16 => A_DELETE,
        // // Arbitrary choice for X button - need it for passcodes.
        // A_JOY13 => A_BACKSPACE,
        // #endif

        //left and right trigger for scrolling
        A_JOY11 => A_PAGE_UP,
        A_JOY12 => A_PAGE_DOWN,

        // start and back button on xbox
        A_JOY1 => {
            //JLF MPMOVED
            A_ESCAPE
        }
        A_JOY2 | A_JOY4 => {
            //JLF MPMOVED
            A_MOUSE1
            //return button;
        }

        A_JOY3 => A_MOUSE1,
        _ => A_SPACE, //Invalid button.
    }
}

#[repr(C)]
struct UIKeyQueueEntry {
    button: c_int,
    pressed: bool,
}

static mut uiKeyQueue: [[UIKeyQueueEntry; 5]; 2] = [[UIKeyQueueEntry {
    button: 0,
    pressed: false,
}; 5]; 2];
static mut uiQueueLen: [c_int; 2] = [0; 2];
static mut uiLastKeyUpDown: [c_int; 2] = [0; 2];
static mut uiLastKeyLeftRight: [c_int; 2] = [0; 2];

pub fn IN_UIEmptyQueue() {
    const A_CURSOR_DOWN: c_int = 1001;
    const A_CURSOR_UP: c_int = 1002;
    const A_CURSOR_RIGHT: c_int = 1003;
    const A_CURSOR_LEFT: c_int = 1004;
    const SE_KEY: c_int = 1;

    /// If the ui is not running then this doesn't have any effect
    unsafe {
        if !_UIRunning {
            uiQueueLen[0] = 0;
            uiQueueLen[1] = 0;
            return;
        }

        // BTO - No CM, bypass that logic.
        //	for (int i = 0; i < ClientManager::NumClients(); i++)
        for i in 0..1 {
            //		ClientManager::ActivateClient(i);
            let mut found = 0;
            let mut bCancel = 0;
            for j in 0..(uiQueueLen[i] as usize) {
                match uiKeyQueue[i][j].button {
                    A_CURSOR_DOWN | A_CURSOR_UP => {
                        if (found & 2) != 0 {
                            // Was a left/right key pressed already?
                            bCancel = 1;
                        }
                        found |= 1;
                    }
                    A_CURSOR_RIGHT | A_CURSOR_LEFT => {
                        if (found & 1) != 0 {
                            // Was an up/down key already pressed?
                            bCancel = 1;
                        }
                        found |= 2;
                    }
                    _ => {}
                }
            }

            if bCancel == 0 {
                // was it cancelled?
                for j in 0..(uiQueueLen[i] as usize) {
                    let time = Sys_Milliseconds();
                    match uiKeyQueue[i][j].button {
                        A_CURSOR_DOWN | A_CURSOR_UP => {
                            if uiLastKeyLeftRight[i] != 0 {
                                if uiLastKeyLeftRight[i] > time {
                                    // don't allow up/down till left/right has enough leway time
                                    continue;
                                }
                            }
                            uiLastKeyUpDown[i] = time + 150; /// 250 ms sound right?
                        }
                        A_CURSOR_LEFT | A_CURSOR_RIGHT => {
                            if uiLastKeyUpDown[i] != 0 {
                                if uiLastKeyUpDown[i] > time {
                                    // don't allow up/down till left/right has enough leway time
                                    continue;
                                }
                            }
                            uiLastKeyLeftRight[i] = time + 150; /// 250 ms sound right?
                        }
                        _ => {}
                    }
                    Sys_QueEvent(
                        0,
                        SE_KEY,
                        uiKeyQueue[i][j].button,
                        uiKeyQueue[i][j].pressed as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
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
    const A_JOY11: c_int = 11;
    const A_JOY12: c_int = 12;
    const A_JOY4: c_int = 4;
    const A_F1: c_int = 282;
    const A_F2: c_int = 283;
    const A_F3: c_int = 284;
    const A_JOY1: c_int = 1;
    const A_JOY2: c_int = 2;
    const A_JOY13: c_int = 13;
    const CA_CINEMATIC: c_int = 6;
    const SE_KEY: c_int = 1;
    const A_ESCAPE: c_int = 27;

    unsafe {
        // Check for special cases for map hack
        // #ifndef FINAL_BUILD
        if Cvar_VariableIntegerValue(b"cl_maphack\0".as_ptr() as *const _) != 0 {
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
        // #endif

        if IN_GetMainController() == controller || _UIRunning {
            // Always map start button to ESCAPE
            if !_UIRunning && button == A_JOY4 && cls.state != CA_CINEMATIC {
                Sys_QueEvent(
                    0,
                    SE_KEY,
                    A_ESCAPE,
                    pressed as c_int,
                    0,
                    core::ptr::null_mut(),
                );
            }

            // #ifndef FINAL_BUILD
            if controller != 3 || !cheatPadEnabled {
                // #endif
                let key = if _UIRunning {
                    UIJoy2Key(button)
                } else {
                    button
                };
                Sys_QueEvent(0, SE_KEY, key, pressed as c_int, 0, core::ptr::null_mut());
            }
        }

        /*
        if (pressed)
        {
            G_DemoKeypress();
        }

        //Hacky!  Skip the credits if start is pressed.
        if(button == K_JOY4 && pressed) {
            CG_SkipCredits();
        }
        */

        // #ifndef FINAL_BUILD
        if controller == 3 && pressed && cheatPadEnabled {
            HandleDebugJoystickPress(button);
            return;
        }
        // #endif

        /*
        extern int player1ControllerId;
        extern int player2ControllerId;
        extern bool checkForPlayerControllers;
        extern bool controllerUnplugged;


        // If the game isn't started yet
        if ((ClientManager::Shared().cls.cgameStarted == qfalse) &&
            (!controllerUnplugged))
        {
            // and player1 doesnt have a controllerid, then assign client 1 to this controller
            if (player1ControllerId == -1)
            {
                ClientManager::ActivateClient(0);
                if (ClientManager::ActiveController() != controller)
                    ClientManager::SetActiveController(controller);
            }
            // player1 has a controller that is different then input recieved, and player2 doesnt have a controller and there are 2 clients
            else if (player1ControllerId != controller && player2ControllerId == -1 && ClientManager::NumClients() > 1)
            {
                ClientManager::ActivateClient(1);
                if (ClientManager::ActiveController() != controller)
                    ClientManager::SetActiveController(controller);
            }
        }

        if (ClientManager::ActivateByControllerId(controller))
        {

        #ifdef _XBOX
            //Check the status of the white or black buttons.
            if (button == K_JOY9) {
                ClientManager::ActiveClient().whiteButtonDown = pressed;
            } else if(button == K_JOY10) {
                ClientManager::ActiveClient().blackButtonDown = pressed;
            }

            //Ignore white/black button presses if inv/force/weap select is up.
            //This is ugly.  It basically says return if the UI isn't running, if
            //we got a white or black button, and if the inv/force/weapon select is
            //running.
            if(!_UIRunning &&
                (button == K_JOY9 ||
                button == K_JOY10) &&
                (ClientManager::ActiveClient().cg.inventorySelectTime +
                WEAPON_SELECT_TIME > ClientManager::ActiveClient().cg.time ||
                ClientManager::ActiveClient().cg.forcepowerSelectTime +
                WEAPON_SELECT_TIME > ClientManager::ActiveClient().cg.time ||
                ClientManager::ActiveClient().cg.weaponSelectTime +
                WEAPON_SELECT_TIME > ClientManager::ActiveClient().cg.time)) {

                if(!pressed) {
                    //And it just gets hackier!  Is that a word?  It is now!
                    //If we've released the button and it wasn't down too long...
                    if((button == K_JOY9 &&
                            ClientManager::ActiveClient().whiteButtonHoldTime <
                            MAX_WB_HOLD_TIME) ||
                        (button == K_JOY10 &&
                            ClientManager::ActiveClient().whiteButtonHoldTime <
                            MAX_WB_HOLD_TIME)) {
                        //If we've already let the button press through previously,
                        //just send a release message.  Otherwise send both a press
                        //and release.
                        if(ClientManager::ActiveClient().keys[button].down) {
                            Sys_QueEvent( 0, SE_KEY, button, false, 0, NULL );
                        } else {
                            Sys_QueEvent( 0, SE_KEY, button, true, 0, NULL );
                            Sys_QueEvent( 0, SE_KEY, button, false, 0, NULL );
                        }
                    }
                }
                return;
            }
        #endif

            if (!_UIRunning)
            {
                Sys_QueEvent( 0, SE_KEY, button, pressed, 0, NULL );
            }
            else
            {
        //		int clientNum = ClientManager::ActiveClientNum();
                int clientNum = 0; // VVFIXME
                int qL = uiQueueLen[clientNum];
                if(qL < 5) {
                    uiKeyQueue[clientNum][qL].button = UIJoy2Key(button);
                    uiKeyQueue[clientNum][qL].pressed = pressed;
                    uiQueueLen[clientNum]++;
                }
                //Sys_QueEvent(0, SE_KEY, UIJoy2Key(button), pressed, 0, NULL);
            }
        */
        /*
        }
        */
    }
}

static mut g_noCheckAxis: bool = false;

/**********
IN_CommonUpdate
Updates thumbstick events based on _padInfo and ui_thumbStickMode
**********/
pub fn IN_CommonUpdate() {
    const KEYCATCH_UI: c_int = 1;
    const AXIS_SIDE: c_int = 0;
    const AXIS_FORWARD: c_int = 1;
    const SE_MOUSE: c_int = 2;
    const SE_JOYSTICK_AXIS: c_int = 3;

    unsafe {
        _UIRunning = Key_GetCatcher() == KEYCATCH_UI;

        // if the UI is running, then let all gamepad sticks work, else only main controller
        if _UIRunning {
            Sys_QueEvent(
                0,
                SE_MOUSE,
                (_padInfo.joyInfo[1].x * 4.0) as c_int,
                (_padInfo.joyInfo[1].y * -4.0) as c_int,
                0,
                core::ptr::null_mut(),
            );
        } else if _padInfo.padId == IN_GetMainController() {
            // Find out how to configure the thumbsticks
            let thumbStickMode = {
                let cvar_ptr = Cvar_Get(
                    b"ui_thumbStickMode\0".as_ptr() as *const _,
                    b"0\0".as_ptr() as *const _,
                    0,
                );
                if !cvar_ptr.is_null() {
                    (*cvar_ptr).integer
                } else {
                    0
                }
            };

            match thumbStickMode {
                0 => {
                    // Configure left thumbstick to move forward/back & strafe left/right
                    Sys_QueEvent(
                        0,
                        SE_JOYSTICK_AXIS,
                        AXIS_SIDE,
                        (_padInfo.joyInfo[0].x * 127.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                    Sys_QueEvent(
                        0,
                        SE_JOYSTICK_AXIS,
                        AXIS_FORWARD,
                        (_padInfo.joyInfo[0].y * 127.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );

                    // Configure right thumbstick for freelook
                    Sys_QueEvent(
                        0,
                        SE_MOUSE,
                        (_padInfo.joyInfo[1].x * 48.0) as c_int,
                        (_padInfo.joyInfo[1].y * 48.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                }
                1 => {
                    // Configure left thumbstick for freelook
                    Sys_QueEvent(
                        0,
                        SE_MOUSE,
                        (_padInfo.joyInfo[0].x * 48.0) as c_int,
                        (_padInfo.joyInfo[0].y * 48.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );

                    // Configure right thumbstick to move forward/back & strafe left/right
                    Sys_QueEvent(
                        0,
                        SE_JOYSTICK_AXIS,
                        AXIS_SIDE,
                        (_padInfo.joyInfo[1].x * 127.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                    Sys_QueEvent(
                        0,
                        SE_JOYSTICK_AXIS,
                        AXIS_FORWARD,
                        (_padInfo.joyInfo[1].y * 127.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                }
                2 => {
                    // Configure left thumbstick to move forward/back & turn left/right
                    Sys_QueEvent(
                        0,
                        SE_JOYSTICK_AXIS,
                        AXIS_FORWARD,
                        (_padInfo.joyInfo[0].y * 127.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                    Sys_QueEvent(
                        0,
                        SE_MOUSE,
                        (_padInfo.joyInfo[0].x * 48.0) as c_int,
                        0,
                        0,
                        core::ptr::null_mut(),
                    );

                    // Configure right thumbstick to look up/down & strafe left/right
                    Sys_QueEvent(
                        0,
                        SE_JOYSTICK_AXIS,
                        AXIS_SIDE,
                        (_padInfo.joyInfo[1].x * 127.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                    Sys_QueEvent(
                        0,
                        SE_MOUSE,
                        0,
                        (_padInfo.joyInfo[1].y * 48.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                }
                3 => {
                    // Configure left thumbstick to look up/down & strafe left/right
                    Sys_QueEvent(
                        0,
                        SE_JOYSTICK_AXIS,
                        AXIS_SIDE,
                        (_padInfo.joyInfo[0].x * 127.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                    Sys_QueEvent(
                        0,
                        SE_MOUSE,
                        0,
                        (_padInfo.joyInfo[0].y * 48.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );

                    // Configure right thumbstick to move forward/back & turn left/right
                    Sys_QueEvent(
                        0,
                        SE_JOYSTICK_AXIS,
                        AXIS_FORWARD,
                        (_padInfo.joyInfo[1].y * 127.0) as c_int,
                        0,
                        core::ptr::null_mut(),
                    );
                    Sys_QueEvent(
                        0,
                        SE_MOUSE,
                        (_padInfo.joyInfo[1].x * 48.0) as c_int,
                        0,
                        0,
                        core::ptr::null_mut(),
                    );
                }
                _ => {}
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
    }

    //TODO Add a call to the UI that draws a controller disconnected message
    // on the screen.
    //	VM_Call( uivm, UI_CONTROLLER_UNPLUGGED, true, controller);
}

/*********
IN_ClearControllerUnplugged
*********/
fn IN_ClearControllerUnplugged() {
    unsafe {
        uiControllerNotification = -1;
    }

    //TODO Add a call to the UI that removes the controller disconnected
    // message from the screen.
    //	VM_Call( uivm, UI_CONTROLLER_UNPLUGGED, false, 0);
}

/*********
IN_ControllerMustBePlugged
*********/
fn IN_ControllerMustBePlugged(controller: c_int) -> bool {
    const CA_LOADING: c_int = 0;
    const CA_CONNECTING: c_int = 1;
    const CA_CONNECTED: c_int = 2;
    const CA_CHALLENGING: c_int = 3;
    const CA_PRIMED: c_int = 4;
    const CA_CINEMATIC: c_int = 6;

    unsafe {
        if cls.state == CA_LOADING
            || cls.state == CA_CONNECTING
            || cls.state == CA_CONNECTED
            || cls.state == CA_CHALLENGING
            || cls.state == CA_PRIMED
            || cls.state == CA_CINEMATIC
        {
            return false;
        }

        if !_UIRunning && controller == IN_GetMainController() {
            return true;
        }

        if noControllersConnected {
            return true;
        }

        return false;
    }
}

/*********
IN_PadUnplugged
*********/
pub fn IN_PadUnplugged(controller: c_int) {
    unsafe {
        if wasPlugged[controller as usize] {
            Com_Printf(
                b"\tController %d unplugged\n\0".as_ptr() as *const _,
                controller,
            );
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
            Com_Printf(
                b"\tController %d plugged\n\0".as_ptr() as *const _,
                controller,
            );
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
fn IN_GetMainController() -> c_int {
    unsafe { cls.mainGamepad }
}

/*********
IN_SetMainController
*********/
fn IN_SetMainController(id: c_int) {
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
        let format_str = va(b"%i\0".as_ptr() as *const _, configValue);
        Cvar_Set(b"ui_thumbStickMode\0".as_ptr() as *const _, format_str);
    }
}

/*********
IN_SetButtonConfig
Execs a button configuration script based on configValue
*********/
pub fn IN_SetButtonConfig(configValue: c_int) {
    const EXEC_NOW: c_int = 0;

    unsafe {
        // Set the cvar
        let format_str = va(b"%i\0".as_ptr() as *const _, configValue);
        Cvar_Set(b"ui_buttonMode\0".as_ptr() as *const _, format_str);

        // Exec the script
        let exec_str = va(
            b"exec cfg\\buttonConfig%i.cfg\n\0".as_ptr() as *const _,
            configValue,
        );
        Cbuf_ExecuteText(EXEC_NOW, exec_str);
    }
}

/*********
IN_SetDpadConfig
Execs a dpad configuration script based on configValue
*********/
pub fn IN_SetDpadConfig(configValue: c_int) {
    const EXEC_NOW: c_int = 0;

    unsafe {
        // Set the cvar
        let format_str = va(b"%i\0".as_ptr() as *const _, configValue);
        Cvar_Set(b"ui_dpadMode\0".as_ptr() as *const _, format_str);

        // Exec the script
        let exec_str = va(
            b"exec cfg\\dpadConfig%i.cfg\n\0".as_ptr() as *const _,
            configValue,
        );
        Cbuf_ExecuteText(EXEC_NOW, exec_str);
    }
}

/**********************************************************
*
* DEBUGGING CODE
*
**********************************************************/
static mut debugSoundOff: bool = false;

// #ifndef FINAL_BUILD
fn HandleDebugJoystickPress(button: fakeAscii_t) {
    const EXEC_APPEND: c_int = 1;

    const A_JOY13: c_int = 13;
    const A_JOY16: c_int = 16;
    const A_JOY14: c_int = 14;
    const A_JOY15: c_int = 15;
    const A_JOY4: c_int = 4;
    const A_JOY1: c_int = 1;
    const A_JOY2: c_int = 2;
    const A_JOY12: c_int = 12;
    const A_JOY8: c_int = 8;
    const A_JOY6: c_int = 6;
    const A_JOY5: c_int = 5;
    const A_JOY7: c_int = 7;
    const A_JOY11: c_int = 11;
    const A_JOY9: c_int = 9;
    const A_JOY10: c_int = 10;

    // Super hackalicious crap used below. Please remove this at some point.
    static mut curSaberSet: c_int = 0;
    static mut curPlayerSet: c_int = 0;
    static mut dpadmode: i16 = 0;
    static mut buttonmode: i16 = 0;
    static mut thumbmode: i16 = 0;

    unsafe {
        match button {
            A_JOY13 => {
                // Right pad up
                Cbuf_ExecuteText(EXEC_APPEND, b"give all\n\0".as_ptr() as *const _);
            }
            A_JOY16 => {
                // Right pad left
                Cbuf_ExecuteText(EXEC_APPEND, b"viewpos\n\0".as_ptr() as *const _);
            }
            A_JOY14 => {
                // Right pad right
                Cbuf_ExecuteText(EXEC_APPEND, b"noclip\n\0".as_ptr() as *const _);
            }
            A_JOY15 => {
                // Right pad down
                Cbuf_ExecuteText(EXEC_APPEND, b"god\n\0".as_ptr() as *const _);
            }
            A_JOY4 => {
                // Start
                let m_pitch_val = Cvar_VariableValue(b"m_pitch\0".as_ptr() as *const _);
                Cvar_SetValue(b"m_pitch\0".as_ptr() as *const _, -m_pitch_val);
            }
            A_JOY1 => {
                // back
                let cl_autolevel_val =
                    Cvar_VariableIntegerValue(b"cl_autolevel\0".as_ptr() as *const _);
                Cvar_SetValue(
                    b"cl_autolevel\0".as_ptr() as *const _,
                    (if cl_autolevel_val != 0 { 0 } else { 1 }) as f32,
                );
            }
            A_JOY2 => {
                // Left thumbstick
                Z_CompactStats();
            }
            A_JOY12 => {
                // Upper right trigger
                Cbuf_ExecuteText(EXEC_APPEND, b"load current\n\0".as_ptr() as *const _);
            }
            A_JOY8 => {
                // Left pad left
                thumbmode += 1;
                if thumbmode == 4 {
                    thumbmode = 0;
                }
                IN_SetThumbStickConfig(thumbmode as c_int);
            }
            A_JOY6 => {
                // Left pad right
                dpadmode += 1;
                if dpadmode == 4 {
                    dpadmode = 0;
                }
                IN_SetDpadConfig(0);
            }
            A_JOY5 => {
                // Left pad up
                buttonmode += 1;
                if buttonmode == 4 {
                    buttonmode = 0;
                }
                IN_SetButtonConfig(buttonmode as c_int);
            }
            A_JOY7 => {
                // Left pad down
                Cbuf_ExecuteText(EXEC_APPEND, b"vid_restart\n\0".as_ptr() as *const _);
            }
            A_JOY11 => {
                // Upper left trigger
                // VVFIXME : This is totally bootleg. The above loads current, because the
                // current SG system writes out to "current" and then uses FS to move the file
                // to the name that we specify. Which we don't support. But we can't write to
                // "current" as the SG system doesn't allow it. So we write to some arbitary
                // name, and our game ends up in current.
                Cbuf_ExecuteText(EXEC_APPEND, b"save foo\n\0".as_ptr() as *const _);
                // #if 0	// VVFIXME
                // extern cvar_t *cl_safezonemask;
                // if(cl_safezonemask) {
                //     if(cl_safezonemask->integer) {
                //         Cbuf_ExecuteText(EXEC_APPEND, "safezonemask 0\n");
                //     } else {
                //         Cbuf_ExecuteText(EXEC_APPEND, "safezonemask 1\n");
                //     }
                // }
                // #endif
            }
            A_JOY9 => {
                // White button
                // Hacky. Really hacky. No, hackier than that.
                curSaberSet = (curSaberSet + 1) % 3; // Number of xsaber strings in config file
                let cmd = va(
                    b"vstr xsaber%d\n\0".as_ptr() as *const _,
                    curSaberSet,
                );
                Cbuf_ExecuteText(EXEC_APPEND, cmd);
            }
            A_JOY10 => {
                // Black button
                curPlayerSet = (curPlayerSet + 1) % 6; // Number of xplayer strings in config file
                let cmd = va(
                    b"vstr xplayer%d\n\0".as_ptr() as *const _,
                    curPlayerSet,
                );
                Cbuf_ExecuteText(EXEC_APPEND, cmd);
            }
            _ => {}
        }
    }
}

// #endif
