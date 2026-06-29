// cl.input.c  -- builds an intended movement command to send to the server

// leave this as first line for PCH reasons...
//

use core::ffi::{c_char, c_int, c_void};

// External dependencies from client.h and related headers
// These are imported/stubbed from the game engine

extern "C" {
    pub static mut frame_msec: core::ffi::c_uint;
    pub static mut old_com_frameTime: c_int;

    pub static mut cl_mPitchOverride: f32;
    pub static mut cl_mYawOverride: f32;

    // External types and functions - stubs for dependencies
    pub fn Cmd_Argv(n: c_int) -> *const c_char;
    pub fn Com_Printf(format: *const c_char, ...);
    pub fn Com_Error(code: c_int, format: *const c_char, ...);
    pub fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *const cvar_t;
    pub fn Cvar_Set(name: *const c_char, value: *const c_char);
    pub fn Cvar_VariableIntegerValue(name: *const c_char) -> c_int;
    pub fn Cmd_AddCommand(name: *const c_char, cmd: unsafe extern "C" fn());
    pub fn Netchan_Transmit(chan: *mut c_void, length: c_int, data: *const u8);
    pub fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);
    pub fn MSG_WriteByte(buf: *mut msg_t, c: u8);
    pub fn MSG_WriteLong(buf: *mut msg_t, c: c_int);
    pub fn MSG_WriteString(buf: *mut msg_t, string: *const c_char);
    pub fn MSG_WriteDeltaUsercmd(buf: *mut msg_t, from: *const usercmd_t, to: *const usercmd_t);
    pub fn IN_CenterView();
    pub fn SCR_DebugGraph(value: c_int, color: c_int);
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
    pub fn ClampChar(i: c_int) -> c_char;
    pub fn Sys_Milliseconds() -> c_int;
    pub fn CL_IsRunningInGameCinematic() -> bool;
    pub fn SQRTFAST(x: f32) -> f32;
    pub fn SHORT2ANGLE(x: c_int) -> f32;
    pub fn ANGLE2SHORT(x: f32) -> c_int;
    pub fn Q_fabs(x: f32) -> f32;

    pub static mut cl: client_t;
    pub static mut cls: clientState_t;
    pub static mut clc: clientConnection_t;
    pub static mut com_frameTime: c_int;
    pub static mut com_sv_running: *const cvar_t;
    pub static mut sv_paused: *const cvar_t;
    pub static mut cl_paused: *const cvar_t;
    pub static mut cl_freelook: *const cvar_t;
    pub static mut m_pitch: *const cvar_t;
    pub static mut m_filter: *const cvar_t;
    pub static mut m_yaw: *const cvar_t;
    pub static mut m_side: *const cvar_t;
    pub static mut m_forward: *const cvar_t;
    pub static mut cl_sensitivity: *const cvar_t;
    pub static mut cl_mouseAccel: *const cvar_t;
    pub static mut cl_showMouseRate: *const cvar_t;
    pub static mut cl_debugMove: *const cvar_t;
    pub static mut cl_nodelta: *const cvar_t;
    pub static mut cl_packetdup: *const cvar_t;

    pub fn _UI_MouseEvent(dx: c_int, dy: c_int);
}

// Type stubs for external dependencies
#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub string: *const c_char,
    pub resetString: *const c_char,
    pub latchedString: *const c_char,
    pub flags: c_int,
    pub modified: bool,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
}

#[repr(C)]
pub struct kbutton_t {
    pub down: [c_int; 2],
    pub downtime: c_int,
    pub msec: c_int,
    pub active: bool,
    pub wasPressed: bool,
}

#[repr(C)]
pub struct usercmd_t {
    pub serverTime: c_int,
    pub buttons: c_int,
    pub weapon: c_char,
    pub forwardmove: c_char,
    pub rightmove: c_char,
    pub upmove: c_char,
    pub angles: [c_int; 3],
    pub generic_cmd: c_int,
}

#[repr(C)]
pub struct msg_t {
    pub allowoverflow: bool,
    pub overflowed: bool,
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub delta_angles: [c_int; 3],
    pub groundEntityNum: c_int,
}

#[repr(C)]
pub struct frame_t {
    pub ps: playerState_t,
    pub valid: bool,
    pub messageNum: c_int,
}

#[repr(C)]
pub struct client_t {
    pub viewangles: [f32; 3],
    pub mouseDx: [c_int; 2],
    pub mouseDy: [c_int; 2],
    pub mouseIndex: c_int,
    pub joystickAxis: [c_int; 4],
    pub frame: frame_t,
    pub serverTime: c_int,
    pub cgameUserCmdValue: c_char,
    pub gcmdSendValue: bool,
    pub gcmdValue: c_int,
    pub cgameSensitivity: f32,
    pub cmdNumber: c_int,
    pub cmds: [usercmd_t; 256],
    pub packetTime: [c_int; 256],
    pub packetCmdNumber: [c_int; 256],
    pub serverId: c_int,
}

#[repr(C)]
pub struct netchan_t {
    pub outgoingSequence: c_int,
}

#[repr(C)]
pub struct clientConnection_t {
    pub netchan: netchan_t,
    pub lastPacketSentTime: c_int,
    pub reliableSequence: c_int,
    pub reliableAcknowledge: c_int,
    pub serverCommandSequence: c_int,
    pub reliableCommands: [[c_char; 1024]; 128],
}

#[repr(C)]
pub struct clientState_t {
    pub keyCatchers: c_int,
    pub realtime: c_int,
    pub frametime: c_int,
    pub state: c_int,
}

// Constants
pub const PITCH: usize = 0;
pub const YAW: usize = 1;
pub const ROLL: usize = 2;

pub const AXIS_SIDE: usize = 0;
pub const AXIS_FORWARD: usize = 1;
pub const AXIS_UP: usize = 2;

pub const MAX_JOYSTICK_AXIS: c_int = 4;
pub const KEYCATCH_UI: c_int = 1;
pub const MAX_MSGLEN: c_int = 16384;
pub const MAX_RELIABLE_COMMANDS: c_int = 128;
pub const MAX_PACKET_USERCMDS: c_int = 32;
pub const CMD_MASK: c_int = 255;
pub const PACKET_MASK: c_int = 255;

pub const ENTITYNUM_NONE: c_int = 1023;

pub const FP_DRAIN: c_int = 0;
pub const FP_PUSH: c_int = 1;
pub const FP_SPEED: c_int = 2;
pub const FP_PULL: c_int = 3;
pub const FP_TELEPATHY: c_int = 4;
pub const FP_GRIP: c_int = 5;
pub const FP_LIGHTNING: c_int = 6;
pub const FP_RAGE: c_int = 7;
pub const FP_PROTECT: c_int = 8;
pub const FP_ABSORB: c_int = 9;
pub const FP_SEE: c_int = 10;
pub const FP_HEAL: c_int = 11;

pub const GENCMD_FORCE_DRAIN: c_int = 1;
pub const GENCMD_FORCE_THROW: c_int = 2;
pub const GENCMD_FORCE_SPEED: c_int = 3;
pub const GENCMD_FORCE_PULL: c_int = 4;
pub const GENCMD_FORCE_DISTRACT: c_int = 5;
pub const GENCMD_FORCE_GRIP: c_int = 6;
pub const GENCMD_FORCE_LIGHTNING: c_int = 7;
pub const GENCMD_FORCE_RAGE: c_int = 8;
pub const GENCMD_FORCE_PROTECT: c_int = 9;
pub const GENCMD_FORCE_ABSORB: c_int = 10;
pub const GENCMD_FORCE_SEEING: c_int = 11;
pub const GENCMD_FORCE_HEAL: c_int = 12;

pub const BUTTON_WALKING: c_int = 0x40;

pub const CA_CINEMATIC: c_int = 2;
pub const CA_PRIMED: c_int = 3;
pub const CA_ACTIVE: c_int = 4;
pub const CA_CONNECTED: c_int = 1;

pub const ERR_DROP: c_int = 0;

pub const clc_clientCommand: u8 = 1;
pub const clc_move: u8 = 2;

pub fn atoi(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }
    unsafe {
        let mut result = 0;
        let mut i = 0;
        let bytes = s as *const u8;
        while *bytes.add(i) != 0 {
            let c = *bytes.add(i);
            if c >= b'0' && c <= b'9' {
                result = result * 10 + (c - b'0') as c_int;
            } else {
                break;
            }
            i += 1;
        }
        result
    }
}

/*
===============================================================================

KEY BUTTONS

Continuous button event tracking is complicated by the fact that two different
input sources (say, mouse button 1 and the control key) can both press the
same button, but the button should only be released when both of the
pressing key have been released.

When a key event issues a button command (+forward, +attack, etc), it appends
its key number as argv(1) so it can be matched up with the release.

argv(2) will be set to the time the event happened, which allows exact
control even at low framerates when the down and up events may both get qued
at the same time.

===============================================================================
*/

pub static mut in_left: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_right: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_forward: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_back: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_lookup: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_lookdown: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_moveleft: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_moveright: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_strafe: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_speed: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_up: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_down: kbutton_t = kbutton_t {
    down: [0, 0],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};

pub static mut in_buttons: [kbutton_t; 9] = [
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
    kbutton_t {
        down: [0, 0],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    },
];

pub static mut in_mlooking: bool = false;

#[cfg(target_os = "windows")]
mod xbox {
    use super::*;

    // HotSwapManager C++ class stubs
    pub struct HotSwapManager {
        id: c_int,
    }

    pub const HOTSWAP_ID_WHITE: c_int = 0;
    pub const HOTSWAP_ID_BLACK: c_int = 1;

    impl HotSwapManager {
        pub fn new(id: c_int) -> Self {
            HotSwapManager { id }
        }

        pub fn SetDown(&mut self) {}
        pub fn SetUp(&mut self) {}
        pub fn Update(&mut self) {}
        pub fn ButtonDown(&self) -> bool {
            false
        }
    }

    pub static mut swapMan1: HotSwapManager = HotSwapManager { id: HOTSWAP_ID_WHITE };
    pub static mut swapMan2: HotSwapManager = HotSwapManager { id: HOTSWAP_ID_BLACK };

    pub unsafe fn IN_HotSwap1On() {
        swapMan1.SetDown();
    }

    pub unsafe fn IN_HotSwap2On() {
        swapMan2.SetDown();
    }

    pub unsafe fn IN_HotSwap1Off() {
        swapMan1.SetUp();
    }

    pub unsafe fn IN_HotSwap2Off() {
        swapMan2.SetUp();
    }

    pub unsafe fn CL_UpdateHotSwap() {
        swapMan1.Update();
        swapMan2.Update();
    }

    pub unsafe fn CL_ExtendSelectTime() -> bool {
        swapMan1.ButtonDown() || swapMan2.ButtonDown()
    }
}

unsafe fn IN_UseGivenForce() {
    let c = Cmd_Argv(1);
    let mut forceNum = -1;
    let mut genCmdNum = 0;

    if !c.is_null() && *c != 0 {
        forceNum = atoi(c);
    } else {
        return;
    }

    match forceNum {
        FP_DRAIN => {
            genCmdNum = GENCMD_FORCE_DRAIN;
        }
        FP_PUSH => {
            genCmdNum = GENCMD_FORCE_THROW;
        }
        FP_SPEED => {
            genCmdNum = GENCMD_FORCE_SPEED;
        }
        FP_PULL => {
            genCmdNum = GENCMD_FORCE_PULL;
        }
        FP_TELEPATHY => {
            genCmdNum = GENCMD_FORCE_DISTRACT;
        }
        FP_GRIP => {
            genCmdNum = GENCMD_FORCE_GRIP;
        }
        FP_LIGHTNING => {
            genCmdNum = GENCMD_FORCE_LIGHTNING;
        }
        FP_RAGE => {
            genCmdNum = GENCMD_FORCE_RAGE;
        }
        FP_PROTECT => {
            genCmdNum = GENCMD_FORCE_PROTECT;
        }
        FP_ABSORB => {
            genCmdNum = GENCMD_FORCE_ABSORB;
        }
        FP_SEE => {
            genCmdNum = GENCMD_FORCE_SEEING;
        }
        FP_HEAL => {
            genCmdNum = GENCMD_FORCE_HEAL;
        }
        _ => {
            panic!("Invalid force number");
        }
    }

    if genCmdNum != 0 {
        (*core::ptr::addr_of_mut!(cl)).gcmdSendValue = true;
        (*core::ptr::addr_of_mut!(cl)).gcmdValue = genCmdNum;
    }
}

pub unsafe fn IN_MLookDown() {
    in_mlooking = true;
}

pub unsafe fn IN_MLookUp() {
    in_mlooking = false;
    if (*cl_freelook).integer == 0 {
        IN_CenterView();
    }
}

unsafe fn IN_KeyDown(b: *mut kbutton_t) {
    let mut k: c_int;
    let c = Cmd_Argv(1);

    if !c.is_null() && *c != 0 {
        k = atoi(c);
    } else {
        k = -1; // typed manually at the console for continuous down
    }

    if k == (*b).down[0] || k == (*b).down[1] {
        return; // repeating key
    }

    if (*b).down[0] == 0 {
        (*b).down[0] = k;
    } else if (*b).down[1] == 0 {
        (*b).down[1] = k;
    } else {
        Com_Printf("Three keys down for a button!\n" as *const c_char);
        return;
    }

    if (*b).active {
        return; // still down
    }

    // save timestamp for partial frame summing
    let c = Cmd_Argv(2);
    (*b).downtime = atoi(c);

    (*b).active = true;
    (*b).wasPressed = true;
}

unsafe fn IN_KeyUp(b: *mut kbutton_t) {
    let mut k: c_int;
    let c = Cmd_Argv(1);
    let mut uptime: core::ffi::c_uint;

    let c = Cmd_Argv(1);
    if !c.is_null() && *c != 0 {
        k = atoi(c);
    } else {
        // typed manually at the console, assume for unsticking, so clear all
        (*b).down[0] = 0;
        (*b).down[1] = 0;
        (*b).active = false;
        return;
    }

    if (*b).down[0] == k {
        (*b).down[0] = 0;
    } else if (*b).down[1] == k {
        (*b).down[1] = 0;
    } else {
        return; // key up without coresponding down (menu pass through)
    }
    if (*b).down[0] != 0 || (*b).down[1] != 0 {
        return; // some other key is still holding it down
    }

    (*b).active = false;

    // save timestamp for partial frame summing
    let c = Cmd_Argv(2);
    uptime = atoi(c) as core::ffi::c_uint;
    if uptime != 0 {
        (*b).msec += (uptime as c_int) - (*b).downtime;
    } else {
        (*b).msec += (frame_msec / 2) as c_int;
    }

    (*b).active = false;
}

/*
===============
CL_KeyState

Returns the fraction of the frame that the key was down
===============
*/
pub unsafe fn CL_KeyState(key: *mut kbutton_t) -> f32 {
    let mut val: f32;
    let mut msec: c_int;

    msec = (*key).msec;
    (*key).msec = 0;

    if (*key).active {
        // still down
        if (*key).downtime == 0 {
            msec = com_frameTime;
        } else {
            msec += com_frameTime - (*key).downtime;
        }
        (*key).downtime = com_frameTime;
    }

    #[cfg(disabled_debug)]
    {
        if msec != 0 {
            Com_Printf("%i " as *const c_char, msec);
        }
    }

    val = msec as f32 / frame_msec as f32;
    if val < 0.0 {
        val = 0.0;
    }
    if val > 1.0 {
        val = 1.0;
    }

    val
}

pub unsafe fn IN_UpDown() {
    IN_KeyDown(&mut in_up);
}
pub unsafe fn IN_UpUp() {
    IN_KeyUp(&mut in_up);
}
pub unsafe fn IN_DownDown() {
    IN_KeyDown(&mut in_down);
}
pub unsafe fn IN_DownUp() {
    IN_KeyUp(&mut in_down);
}
pub unsafe fn IN_LeftDown() {
    IN_KeyDown(&mut in_left);
}
pub unsafe fn IN_LeftUp() {
    IN_KeyUp(&mut in_left);
}
pub unsafe fn IN_RightDown() {
    IN_KeyDown(&mut in_right);
}
pub unsafe fn IN_RightUp() {
    IN_KeyUp(&mut in_right);
}
pub unsafe fn IN_ForwardDown() {
    IN_KeyDown(&mut in_forward);
}
pub unsafe fn IN_ForwardUp() {
    IN_KeyUp(&mut in_forward);
}
pub unsafe fn IN_BackDown() {
    IN_KeyDown(&mut in_back);
}
pub unsafe fn IN_BackUp() {
    IN_KeyUp(&mut in_back);
}
pub unsafe fn IN_LookupDown() {
    IN_KeyDown(&mut in_lookup);
}
pub unsafe fn IN_LookupUp() {
    IN_KeyUp(&mut in_lookup);
}
pub unsafe fn IN_LookdownDown() {
    IN_KeyDown(&mut in_lookdown);
}
pub unsafe fn IN_LookdownUp() {
    IN_KeyUp(&mut in_lookdown);
}
pub unsafe fn IN_MoveleftDown() {
    IN_KeyDown(&mut in_moveleft);
}
pub unsafe fn IN_MoveleftUp() {
    IN_KeyUp(&mut in_moveleft);
}
pub unsafe fn IN_MoverightDown() {
    IN_KeyDown(&mut in_moveright);
}
pub unsafe fn IN_MoverightUp() {
    IN_KeyUp(&mut in_moveright);
}

pub unsafe fn IN_SpeedDown() {
    IN_KeyDown(&mut in_speed);
}
pub unsafe fn IN_SpeedUp() {
    IN_KeyUp(&mut in_speed);
}
pub unsafe fn IN_StrafeDown() {
    IN_KeyDown(&mut in_strafe);
}
pub unsafe fn IN_StrafeUp() {
    IN_KeyUp(&mut in_strafe);
}

pub unsafe fn IN_Button0Down() {
    IN_KeyDown(&mut in_buttons[0]);
}
pub unsafe fn IN_Button0Up() {
    IN_KeyUp(&mut in_buttons[0]);
}
pub unsafe fn IN_Button1Down() {
    IN_KeyDown(&mut in_buttons[1]);
}
pub unsafe fn IN_Button1Up() {
    IN_KeyUp(&mut in_buttons[1]);
}
pub unsafe fn IN_Button2Down() {
    IN_KeyDown(&mut in_buttons[2]);
}
pub unsafe fn IN_Button2Up() {
    IN_KeyUp(&mut in_buttons[2]);
}
pub unsafe fn IN_Button3Down() {
    IN_KeyDown(&mut in_buttons[3]);
}
pub unsafe fn IN_Button3Up() {
    IN_KeyUp(&mut in_buttons[3]);
}
pub unsafe fn IN_Button4Down() {
    IN_KeyDown(&mut in_buttons[4]);
}
pub unsafe fn IN_Button4Up() {
    IN_KeyUp(&mut in_buttons[4]);
}
pub unsafe fn IN_Button5Down() {
    IN_KeyDown(&mut in_buttons[5]);
}
pub unsafe fn IN_Button5Up() {
    IN_KeyUp(&mut in_buttons[5]);
}
pub unsafe fn IN_Button6Down() {
    IN_KeyDown(&mut in_buttons[6]);
}
pub unsafe fn IN_Button6Up() {
    IN_KeyUp(&mut in_buttons[6]);
}
pub unsafe fn IN_Button7Down() {
    IN_KeyDown(&mut in_buttons[7]);
}
pub unsafe fn IN_Button7Up() {
    IN_KeyUp(&mut in_buttons[7]);
}
pub unsafe fn IN_Button8Down() {
    IN_KeyDown(&mut in_buttons[8]);
}
pub unsafe fn IN_Button8Up() {
    IN_KeyUp(&mut in_buttons[8]);
}

//==========================================================================

pub static mut cl_upspeed: *const cvar_t = core::ptr::null();
pub static mut cl_forwardspeed: *const cvar_t = core::ptr::null();
pub static mut cl_sidespeed: *const cvar_t = core::ptr::null();

pub static mut cl_yawspeed: *const cvar_t = core::ptr::null();
pub static mut cl_pitchspeed: *const cvar_t = core::ptr::null();

pub static mut cl_run: *const cvar_t = core::ptr::null();

pub static mut cl_anglespeedkey: *const cvar_t = core::ptr::null();

/*
================
CL_AdjustAngles

Moves the local angle positions
================
*/
pub unsafe fn CL_AdjustAngles() {
    let mut speed: f32;

    if (*core::ptr::addr_of!(in_speed)).active {
        speed = 0.001 * cls.frametime as f32 * (*cl_anglespeedkey).value;
    } else {
        speed = 0.001 * cls.frametime as f32;
    }

    if !(*core::ptr::addr_of!(in_strafe)).active {
        if cl_mYawOverride != 0.0 {
            (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] -=
                cl_mYawOverride * 5.0 * speed * (*cl_yawspeed).value * CL_KeyState(&mut in_right);
            (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] +=
                cl_mYawOverride * 5.0 * speed * (*cl_yawspeed).value * CL_KeyState(&mut in_left);
        } else {
            (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] -=
                speed * (*cl_yawspeed).value * CL_KeyState(&mut in_right);
            (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] +=
                speed * (*cl_yawspeed).value * CL_KeyState(&mut in_left);
        }
    }

    if cl_mPitchOverride != 0.0 {
        (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] -=
            cl_mPitchOverride * 5.0 * speed * (*cl_pitchspeed).value * CL_KeyState(&mut in_lookup);
        (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] +=
            cl_mPitchOverride * 5.0 * speed * (*cl_pitchspeed).value * CL_KeyState(&mut in_lookdown);
    } else {
        (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] -=
            speed * (*cl_pitchspeed).value * CL_KeyState(&mut in_lookup);
        (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] +=
            speed * (*cl_pitchspeed).value * CL_KeyState(&mut in_lookdown);
    }
}

/*
================
CL_KeyMove

Sets the usercmd_t based on key states
================
*/
pub unsafe fn CL_KeyMove(cmd: *mut usercmd_t) {
    let mut movespeed: c_int;
    let mut forward: c_int;
    let mut side: c_int;
    let mut up: c_int;

    //
    // adjust for speed key / running
    // the walking flag is to keep animations consistant
    // even during acceleration and develeration
    //
    if ((*core::ptr::addr_of!(in_speed)).active as c_int) ^ (*cl_run).integer != 0 {
        movespeed = 127;
        (*cmd).buttons &= !BUTTON_WALKING;
    } else {
        (*cmd).buttons |= BUTTON_WALKING;
        movespeed = 64;
    }

    forward = 0;
    side = 0;
    up = 0;
    if (*core::ptr::addr_of!(in_strafe)).active {
        side += (movespeed as f32 * CL_KeyState(&mut in_right)) as c_int;
        side -= (movespeed as f32 * CL_KeyState(&mut in_left)) as c_int;
    }

    side += (movespeed as f32 * CL_KeyState(&mut in_moveright)) as c_int;
    side -= (movespeed as f32 * CL_KeyState(&mut in_moveleft)) as c_int;

    up += (movespeed as f32 * CL_KeyState(&mut in_up)) as c_int;
    up -= (movespeed as f32 * CL_KeyState(&mut in_down)) as c_int;

    forward += (movespeed as f32 * CL_KeyState(&mut in_forward)) as c_int;
    forward -= (movespeed as f32 * CL_KeyState(&mut in_back)) as c_int;

    (*cmd).forwardmove = ClampChar(forward);
    (*cmd).rightmove = ClampChar(side);
    (*cmd).upmove = ClampChar(up);
}

/*
=================
CL_MouseEvent
=================
*/
pub unsafe fn CL_MouseEvent(dx: c_int, dy: c_int, time: c_int) {
    if cls.keyCatchers & KEYCATCH_UI != 0 {
        _UI_MouseEvent(dx, dy);
    } else {
        (*core::ptr::addr_of_mut!(cl)).mouseDx[(*core::ptr::addr_of!(cl)).mouseIndex as usize] +=
            dx;
        (*core::ptr::addr_of_mut!(cl)).mouseDy[(*core::ptr::addr_of!(cl)).mouseIndex as usize] +=
            dy;
    }
}

/*
=================
CL_JoystickEvent

Joystick values stay set until changed
=================
*/
pub unsafe fn CL_JoystickEvent(axis: c_int, value: c_int, time: c_int) {
    if axis < 0 || axis >= MAX_JOYSTICK_AXIS {
        Com_Error(ERR_DROP, "CL_JoystickEvent: bad axis %i\0" as *const c_char, axis);
    }
    (*core::ptr::addr_of_mut!(cl)).joystickAxis[axis as usize] = value;
}

/*
=================
CL_JoystickMove
=================
*/
pub unsafe fn CL_JoystickMove(cmd: *mut usercmd_t) {
    let mut movespeed: c_int;
    let mut anglespeed: f32;

    if ((*core::ptr::addr_of!(in_speed)).active as c_int) ^ (*cl_run).integer != 0 {
        movespeed = 2;
    } else {
        movespeed = 1;
        (*cmd).buttons |= BUTTON_WALKING;
    }

    if (*core::ptr::addr_of!(in_speed)).active {
        anglespeed = 0.001 * cls.frametime as f32 * (*cl_anglespeedkey).value;
    } else {
        anglespeed = 0.001 * cls.frametime as f32;
    }

    #[cfg(not(target_os = "windows"))]
    {
        if !(*core::ptr::addr_of!(in_strafe)).active {
            if cl_mYawOverride != 0.0 {
                (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] +=
                    5.0 * cl_mYawOverride * (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_SIDE] as f32;
            } else {
                (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] += anglespeed
                    * ((*cl_yawspeed).value / 100.0)
                    * (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_SIDE] as f32;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if !(*core::ptr::addr_of!(in_strafe)).active {
            if cl_mYawOverride != 0.0 {
                (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] +=
                    5.0 * cl_mYawOverride * (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_SIDE] as f32;
            } else {
                (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] += anglespeed
                    * ((*cl_yawspeed).value / 100.0)
                    * (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_SIDE] as f32;
            }
        }
    }

    (*cmd).rightmove = ClampChar(
        (*cmd).rightmove as c_int + (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_SIDE],
    );

    if in_mlooking {
        if cl_mPitchOverride != 0.0 {
            (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] +=
                5.0 * cl_mPitchOverride * (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_FORWARD] as f32;
        } else {
            (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] += anglespeed
                * ((*cl_pitchspeed).value / 100.0)
                * (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_FORWARD] as f32;
        }
    } else {
        (*cmd).forwardmove = ClampChar(
            (*cmd).forwardmove as c_int + (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_FORWARD],
        );
    }

    (*cmd).upmove = ClampChar(
        (*cmd).upmove as c_int + (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_UP],
    );
}

/*
=================
CL_MouseMove
=================
*/

#[cfg(target_os = "windows")]
pub unsafe fn CL_MouseClamp(x: *mut c_int, y: *mut c_int) {
    let mut ax = Q_fabs(*x as f32);
    let mut ay = Q_fabs(*y as f32);

    ax = (ax - 10.0) * (3.0 / 45.0) * (ax - 10.0) * if Q_fabs(*x as f32) > 10.0 { 1.0 } else { 0.0 };
    ay = (ay - 10.0) * (3.0 / 45.0) * (ay - 10.0) * if Q_fabs(*y as f32) > 10.0 { 1.0 } else { 0.0 };
    if *x < 0 {
        *x = -ax as c_int;
    } else {
        *x = ax as c_int;
    }
    if *y < 0 {
        *y = -ay as c_int;
    } else {
        *y = ay as c_int;
    }
}

pub unsafe fn CL_MouseMove(cmd: *mut usercmd_t) {
    let mut mx: f32;
    let mut my: f32;
    let mut accelSensitivity: f32;
    let mut rate: f32;
    let speed = frame_msec as f32;
    let pitch = (*m_pitch).value;

    #[cfg(target_os = "windows")]
    {
        const mouseSpeedX: f32 = 0.06;
        const mouseSpeedY: f32 = 0.05;

        extern "C" {
            static cg_crossHairStatus: c_int;
            static g_lastFireTime: c_int;
        }

        // allow mouse smoothing
        if (*m_filter).integer != 0 {
            mx = ((*core::ptr::addr_of!(cl)).mouseDx[0] + (*core::ptr::addr_of!(cl)).mouseDx[1])
                as f32
                * 0.5
                * frame_msec as f32
                * mouseSpeedX;
            my = ((*core::ptr::addr_of!(cl)).mouseDy[0] + (*core::ptr::addr_of!(cl)).mouseDy[1])
                as f32
                * 0.5
                * frame_msec as f32
                * mouseSpeedY;
        } else {
            let mut ax = (*core::ptr::addr_of!(cl)).mouseDx
                [(*core::ptr::addr_of!(cl)).mouseIndex as usize];
            let mut ay = (*core::ptr::addr_of!(cl)).mouseDy
                [(*core::ptr::addr_of!(cl)).mouseIndex as usize];
            CL_MouseClamp(&mut ax, &mut ay);

            mx = ax as f32 * speed * mouseSpeedX;
            my = ay as f32 * speed * mouseSpeedY;
        }

        let m_hoverSensitivity = 0.4;
        if cg_crossHairStatus == 1 {
            mx *= m_hoverSensitivity;
            my *= m_hoverSensitivity;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // allow mouse smoothing
        if (*m_filter).integer != 0 {
            mx = ((*core::ptr::addr_of!(cl)).mouseDx[0] + (*core::ptr::addr_of!(cl)).mouseDx[1])
                as f32
                * 0.5;
            my = ((*core::ptr::addr_of!(cl)).mouseDy[0] + (*core::ptr::addr_of!(cl)).mouseDy[1])
                as f32
                * 0.5;
        } else {
            mx = (*core::ptr::addr_of!(cl)).mouseDx[(*core::ptr::addr_of!(cl)).mouseIndex as usize]
                as f32;
            my = (*core::ptr::addr_of!(cl)).mouseDy[(*core::ptr::addr_of!(cl)).mouseIndex as usize]
                as f32;
        }
    }

    (*core::ptr::addr_of_mut!(cl)).mouseIndex ^= 1;
    (*core::ptr::addr_of_mut!(cl)).mouseDx[(*core::ptr::addr_of!(cl)).mouseIndex as usize] = 0;
    (*core::ptr::addr_of_mut!(cl)).mouseDy[(*core::ptr::addr_of!(cl)).mouseIndex as usize] = 0;

    rate = SQRTFAST(mx * mx + my * my) / speed;
    accelSensitivity = (*cl_sensitivity).value + rate * (*cl_mouseAccel).value;

    // scale by FOV
    accelSensitivity *= (*core::ptr::addr_of!(cl)).cgameSensitivity;

    if rate != 0.0 && (*cl_showMouseRate).integer != 0 {
        Com_Printf("%f : %f\n" as *const c_char, rate, accelSensitivity);
    }

    mx *= accelSensitivity;
    my *= accelSensitivity;

    if mx == 0.0 && my == 0.0 {
        #[cfg(target_os = "windows")]
        {
            extern "C" {
                static cg_crossHairStatus: c_int;
                static g_lastFireTime: c_int;
            }

            // If there was a movement but no change in angles then start auto-leveling the camera
            let autolevelSpeed = 0.03;

            if cg_crossHairStatus != 1
                && // Not looking at an enemy
                (*core::ptr::addr_of!(cl)).joystickAxis[AXIS_FORWARD] != 0
                && // Moving forward/backward
                (*core::ptr::addr_of!(cl)).frame.ps.groundEntityNum != ENTITYNUM_NONE
                && // Not in the air
                Cvar_VariableIntegerValue("cl_autolevel\0" as *const c_char) != 0
                && // Autolevel is turned on
                g_lastFireTime < (Sys_Milliseconds() - 1000)
            // Haven't fired recently
            {
                let mut normAngle = -SHORT2ANGLE((*core::ptr::addr_of!(cl)).frame.ps.delta_angles[PITCH]);
                // The adjustment to normAngle below is meant to add or remove some multiple
                // of 360, so that normAngle is within 180 of viewangles[PITCH]. It should
                // be correct.
                let diff = (*core::ptr::addr_of!(cl)).viewangles[PITCH] as c_int - normAngle as c_int;
                if diff > 180 {
                    normAngle += 360.0 * ((diff + 180) as f32 / 360.0);
                } else if diff < -180 {
                    normAngle -= 360.0 * ((-diff + 180) as f32 / 360.0);
                }

                if Cvar_VariableIntegerValue("cg_thirdperson\0" as *const c_char) == 1 {
                    //				normAngle += 10;	// Removed by BTO, 2003/05/14, I hate it
                    // autolevelSpeed *= 1.5;
                }
                if (*core::ptr::addr_of!(cl)).viewangles[PITCH] > normAngle {
                    (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] -= autolevelSpeed * speed;
                    if (*core::ptr::addr_of!(cl)).viewangles[PITCH] < normAngle {
                        (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] = normAngle;
                    }
                } else if (*core::ptr::addr_of!(cl)).viewangles[PITCH] < normAngle {
                    (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] += autolevelSpeed * speed;
                    if (*core::ptr::addr_of!(cl)).viewangles[PITCH] > normAngle {
                        (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] = normAngle;
                    }
                }
            }
        }
        return;
    }

    // add mouse X/Y movement to cmd
    if (*core::ptr::addr_of!(in_strafe)).active {
        (*cmd).rightmove = ClampChar((*cmd).rightmove as c_int + ((*m_side).value * mx) as c_int);
    } else {
        if cl_mYawOverride != 0.0 {
            (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] -= cl_mYawOverride * mx;
        } else {
            (*core::ptr::addr_of_mut!(cl)).viewangles[YAW] -= (*m_yaw).value * mx;
        }
    }

    if (in_mlooking || (*cl_freelook).integer != 0) && !(*core::ptr::addr_of!(in_strafe)).active {
        // VVFIXME - This is supposed to be a CVAR
        #[cfg(target_os = "windows")]
        const cl_pitchSensitivity: f32 = 0.5;
        #[cfg(not(target_os = "windows"))]
        const cl_pitchSensitivity: f32 = 1.0;

        if cl_mPitchOverride != 0.0 {
            if pitch > 0.0 {
                (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] +=
                    cl_mPitchOverride * my * cl_pitchSensitivity;
            } else {
                (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] -=
                    cl_mPitchOverride * my * cl_pitchSensitivity;
            }
        } else {
            (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] += pitch * my * cl_pitchSensitivity;
        }
    } else {
        (*cmd).forwardmove =
            ClampChar((*cmd).forwardmove as c_int - ((*m_forward).value * my) as c_int);
    }
}

/*
==============
CL_CmdButtons
==============
*/
pub unsafe fn CL_CmdButtons(cmd: *mut usercmd_t) {
    let mut i: c_int;

    //
    // figure button bits
    // send a button bit even if the key was pressed and released in
    // less than a frame
    //
    for i in 0..9 {
        if (*core::ptr::addr_of!(in_buttons[i])).active
            || (*core::ptr::addr_of!(in_buttons[i])).wasPressed
        {
            (*cmd).buttons |= 1 << i;
        }
        (*core::ptr::addr_of_mut!(in_buttons[i])).wasPressed = false;
    }

    if cls.keyCatchers != 0 {
        //cmd->buttons |= BUTTON_TALK;
    }

    // allow the game to know if any key at all is
    // currently pressed, even if it isn't bound to anything
    /*
    if ( kg.anykeydown && !cls.keyCatchers ) {
        cmd->buttons |= BUTTON_ANY;
    }
    */
}

/*
==============
CL_FinishMove
==============
*/
pub unsafe fn CL_FinishMove(cmd: *mut usercmd_t) {
    let mut i: c_int;

    // copy the state that the cgame is currently sending
    (*cmd).weapon = (*core::ptr::addr_of!(cl)).cgameUserCmdValue;

    if (*core::ptr::addr_of!(cl)).gcmdSendValue {
        (*cmd).generic_cmd = (*core::ptr::addr_of!(cl)).gcmdValue;
        (*core::ptr::addr_of_mut!(cl)).gcmdSendValue = false;
    } else {
        (*cmd).generic_cmd = 0;
    }

    // send the current server time so the amount of movement
    // can be determined without allowing cheating
    (*cmd).serverTime = (*core::ptr::addr_of!(cl)).serverTime;

    for i in 0..3 {
        (*cmd).angles[i as usize] = ANGLE2SHORT((*core::ptr::addr_of!(cl)).viewangles[i]);
    }
}

/*
=================
CL_CreateCmd
=================
*/
pub static mut cl_overriddenAngles: [f32; 3] = [0.0, 0.0, 0.0];
pub static mut cl_overrideAngles: bool = false;

pub unsafe fn CL_CreateCmd() -> usercmd_t {
    let mut cmd: usercmd_t = usercmd_t {
        serverTime: 0,
        buttons: 0,
        weapon: 0,
        forwardmove: 0,
        rightmove: 0,
        upmove: 0,
        angles: [0, 0, 0],
        generic_cmd: 0,
    };
    let mut oldAngles: [f32; 3] = [0.0, 0.0, 0.0];

    VectorCopy((*core::ptr::addr_of!(cl)).viewangles.as_ptr(), oldAngles.as_mut_ptr());

    // keyboard angle adjustment
    CL_AdjustAngles();

    // CL_KeyMove (&cmd);
    CL_KeyMove(&mut cmd);

    // get basic movement from mouse
    CL_MouseMove(&mut cmd);

    // get basic movement from joystick
    CL_JoystickMove(&mut cmd);

    // check to make sure the angles haven't wrapped
    if (*core::ptr::addr_of!(cl)).viewangles[PITCH] - oldAngles[PITCH] > 90.0 {
        (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] = oldAngles[PITCH] + 90.0;
    } else if oldAngles[PITCH] - (*core::ptr::addr_of!(cl)).viewangles[PITCH] > 90.0 {
        (*core::ptr::addr_of_mut!(cl)).viewangles[PITCH] = oldAngles[PITCH] - 90.0;
    }

    if cl_overrideAngles {
        VectorCopy(cl_overriddenAngles.as_ptr(), (*core::ptr::addr_of_mut!(cl)).viewangles.as_mut_ptr());
        cl_overrideAngles = false;
    }
    // store out the final values
    CL_FinishMove(&mut cmd);

    // draw debug graphs of turning for mouse testing
    #[cfg(not(target_os = "windows"))]
    {
        if (*cl_debugMove).integer != 0 {
            if (*cl_debugMove).integer == 1 {
                SCR_DebugGraph(
                    ((*core::ptr::addr_of!(cl)).viewangles[YAW] - oldAngles[YAW]).abs() as c_int,
                    0,
                );
            }
            if (*cl_debugMove).integer == 2 {
                SCR_DebugGraph(
                    ((*core::ptr::addr_of!(cl)).viewangles[PITCH] - oldAngles[PITCH]).abs() as c_int,
                    0,
                );
            }
        }
    }

    cmd
}

/*
=================
CL_CreateNewCommands

Create a new usercmd_t structure for this frame
=================
*/
pub unsafe fn CL_CreateNewCommands() {
    let mut cmd: *mut usercmd_t;
    let mut cmdNum: c_int;

    // no need to create usercmds until we have a gamestate
    //	if ( cls.state < CA_PRIMED ) {
    //		return;
    //	}

    frame_msec = (com_frameTime - old_com_frameTime) as core::ffi::c_uint;

    // if running less than 5fps, truncate the extra time to prevent
    // unexpected moves after a hitch
    if frame_msec > 200 {
        frame_msec = 200;
    }
    old_com_frameTime = com_frameTime;

    // generate a command for this frame
    (*core::ptr::addr_of_mut!(cl)).cmdNumber += 1;
    cmdNum = (*core::ptr::addr_of!(cl)).cmdNumber & CMD_MASK;
    (*core::ptr::addr_of_mut!(cl)).cmds[cmdNum as usize] = CL_CreateCmd();
    cmd = &mut (*core::ptr::addr_of_mut!(cl)).cmds[cmdNum as usize];
}

/*
=================
CL_ReadyToSendPacket

Returns qfalse if we are over the maxpackets limit
and should choke back the bandwidth a bit by not sending
a packet this frame.  All the commands will still get
delivered in the next packet, but saving a header and
getting more delta compression will reduce total bandwidth.
=================
*/
pub unsafe fn CL_ReadyToSendPacket() -> bool {
    // don't send anything if playing back a demo
    //	if ( cls.state == CA_CINEMATIC )
    if cls.state == CA_CINEMATIC || CL_IsRunningInGameCinematic() {
        return false;
    }

    // if we don't have a valid gamestate yet, only send
    // one packet a second
    if cls.state != CA_ACTIVE
        && cls.state != CA_PRIMED
        && cls.realtime - clc.lastPacketSentTime < 1000
    {
        return false;
    }

    // send every frame for loopbacks
    true
}

/*
===================
CL_WritePacket

Create and send the command packet to the server
Including both the reliable commands and the usercmds

During normal gameplay, a client packet will contain something like:

4	sequence number
2	qport
4	serverid
4	acknowledged sequence number
4	clc.serverCommandSequence
<optional reliable commands>
1	clc_move or clc_moveNoDelta
1	command count
<count * usercmds>

===================
*/
pub unsafe fn CL_WritePacket() {
    let mut buf: msg_t = msg_t {
        allowoverflow: false,
        overflowed: false,
        data: core::ptr::null_mut(),
        maxsize: 0,
        cursize: 0,
        readcount: 0,
        bit: 0,
    };
    let mut data: [u8; MAX_MSGLEN as usize] = [0; MAX_MSGLEN as usize];
    let mut i: c_int;
    let mut j: c_int;
    let mut cmd: *const usercmd_t;
    let mut oldcmd: *const usercmd_t;
    let mut nullcmd: usercmd_t = usercmd_t {
        serverTime: 0,
        buttons: 0,
        weapon: 0,
        forwardmove: 0,
        rightmove: 0,
        upmove: 0,
        angles: [0, 0, 0],
        generic_cmd: 0,
    };
    let mut packetNum: c_int;
    let mut oldPacketNum: c_int;
    let mut count: c_int;

    // don't send anything if playing back a demo
    //	if ( cls.state == CA_CINEMATIC )
    if cls.state == CA_CINEMATIC || CL_IsRunningInGameCinematic() {
        return;
    }

    MSG_Init(&mut buf, data.as_mut_ptr(), MAX_MSGLEN);

    // write any unacknowledged clientCommands
    i = clc.reliableAcknowledge + 1;
    while i <= clc.reliableSequence {
        MSG_WriteByte(&mut buf, clc_clientCommand);
        MSG_WriteLong(&mut buf, i);
        MSG_WriteString(
            &mut buf,
            clc.reliableCommands[(i & (MAX_RELIABLE_COMMANDS - 1)) as usize].as_ptr() as *const c_char,
        );
        i += 1;
    }

    // we want to send all the usercmds that were generated in the last
    // few packet, so even if a couple packets are dropped in a row,
    // all the cmds will make it to the server
    if (*cl_packetdup).integer < 0 {
        Cvar_Set("cl_packetdup\0" as *const c_char, "0\0" as *const c_char);
    } else if (*cl_packetdup).integer > 5 {
        Cvar_Set("cl_packetdup\0" as *const c_char, "5\0" as *const c_char);
    }
    oldPacketNum = (clc.netchan.outgoingSequence - 1 - (*cl_packetdup).integer) & PACKET_MASK;
    count = (*core::ptr::addr_of!(cl)).cmdNumber
        - (*core::ptr::addr_of!(cl)).packetCmdNumber[oldPacketNum as usize];
    if count > MAX_PACKET_USERCMDS {
        count = MAX_PACKET_USERCMDS;
        Com_Printf("MAX_PACKET_USERCMDS\n" as *const c_char);
    }
    if count >= 1 {
        // begin a client move command
        MSG_WriteByte(&mut buf, clc_move);

        // write the last reliable message we received
        MSG_WriteLong(&mut buf, clc.serverCommandSequence);

        // write the current serverId so the server
        // can tell if this is from the current gameState
        MSG_WriteLong(&mut buf, (*core::ptr::addr_of!(cl)).serverId);

        // write the current time
        MSG_WriteLong(&mut buf, cls.realtime);

        // let the server know what the last messagenum we
        // got was, so the next message can be delta compressed
        // FIXME: this could just be a bit flag, with the message implicit
        // from the unreliable ack of the netchan
        if (*cl_nodelta).integer != 0 || !(*core::ptr::addr_of!(cl)).frame.valid {
            MSG_WriteLong(&mut buf, -1); // no compression
        } else {
            MSG_WriteLong(&mut buf, (*core::ptr::addr_of!(cl)).frame.messageNum);
        }

        // write the cmdNumber so the server can determine which ones it
        // has already received
        MSG_WriteLong(&mut buf, (*core::ptr::addr_of!(cl)).cmdNumber);

        // write the command count
        MSG_WriteByte(&mut buf, count as u8);

        // write all the commands, including the predicted command
        oldcmd = &nullcmd;
        i = 0;
        while i < count {
            j = ((*core::ptr::addr_of!(cl)).cmdNumber - count + i + 1) & CMD_MASK;
            cmd = &(*core::ptr::addr_of!(cl)).cmds[j as usize];
            MSG_WriteDeltaUsercmd(&mut buf, oldcmd, cmd);
            oldcmd = cmd;
            i += 1;
        }
    }

    //
    // deliver the message
    //
    packetNum = clc.netchan.outgoingSequence & PACKET_MASK;
    (*core::ptr::addr_of_mut!(cl)).packetTime[packetNum as usize] = cls.realtime;
    (*core::ptr::addr_of_mut!(cl)).packetCmdNumber[packetNum as usize] =
        (*core::ptr::addr_of!(cl)).cmdNumber;
    clc.lastPacketSentTime = cls.realtime;
    Netchan_Transmit(&mut clc.netchan, buf.cursize, buf.data);
}

/*
=================
CL_SendCmd

Called every frame to builds and sends a command packet to the server.
=================
*/
pub unsafe fn CL_SendCmd() {
    // don't send any message if not connected
    if cls.state < CA_CONNECTED {
        return;
    }

    // don't send commands if paused
    if (*com_sv_running).integer != 0
        && (*sv_paused).integer != 0
        && (*cl_paused).integer != 0
    {
        return;
    }

    // we create commands even if a demo is playing,
    CL_CreateNewCommands();

    // don't send a packet if the last packet was sent too recently
    if !CL_ReadyToSendPacket() {
        return;
    }

    CL_WritePacket();
}

/*
============
CL_InitInput
============
*/
pub unsafe fn CL_InitInput() {
    Cmd_AddCommand("centerview\0" as *const c_char, IN_CenterView);

    Cmd_AddCommand("+moveup\0" as *const c_char, IN_UpDown);
    Cmd_AddCommand("-moveup\0" as *const c_char, IN_UpUp);
    Cmd_AddCommand("+movedown\0" as *const c_char, IN_DownDown);
    Cmd_AddCommand("-movedown\0" as *const c_char, IN_DownUp);
    Cmd_AddCommand("+left\0" as *const c_char, IN_LeftDown);
    Cmd_AddCommand("-left\0" as *const c_char, IN_LeftUp);
    Cmd_AddCommand("+right\0" as *const c_char, IN_RightDown);
    Cmd_AddCommand("-right\0" as *const c_char, IN_RightUp);
    Cmd_AddCommand("+forward\0" as *const c_char, IN_ForwardDown);
    Cmd_AddCommand("-forward\0" as *const c_char, IN_ForwardUp);
    Cmd_AddCommand("+back\0" as *const c_char, IN_BackDown);
    Cmd_AddCommand("-back\0" as *const c_char, IN_BackUp);
    Cmd_AddCommand("+lookup\0" as *const c_char, IN_LookupDown);
    Cmd_AddCommand("-lookup\0" as *const c_char, IN_LookupUp);
    Cmd_AddCommand("+lookdown\0" as *const c_char, IN_LookdownDown);
    Cmd_AddCommand("-lookdown\0" as *const c_char, IN_LookdownUp);
    Cmd_AddCommand("+strafe\0" as *const c_char, IN_StrafeDown);
    Cmd_AddCommand("-strafe\0" as *const c_char, IN_StrafeUp);
    Cmd_AddCommand("+moveleft\0" as *const c_char, IN_MoveleftDown);
    Cmd_AddCommand("-moveleft\0" as *const c_char, IN_MoveleftUp);
    Cmd_AddCommand("+moveright\0" as *const c_char, IN_MoverightDown);
    Cmd_AddCommand("-moveright\0" as *const c_char, IN_MoverightUp);
    Cmd_AddCommand("+speed\0" as *const c_char, IN_SpeedDown);
    Cmd_AddCommand("-speed\0" as *const c_char, IN_SpeedUp);
    //xbox hot swappable buttons
    #[cfg(target_os = "windows")]
    {
        Cmd_AddCommand("+hotswap1\0" as *const c_char, xbox::IN_HotSwap1On);
        Cmd_AddCommand("+hotswap2\0" as *const c_char, xbox::IN_HotSwap2On);
        Cmd_AddCommand("-hotswap1\0" as *const c_char, xbox::IN_HotSwap1Off);
        Cmd_AddCommand("-hotswap2\0" as *const c_char, xbox::IN_HotSwap2Off);
    }
    Cmd_AddCommand("useGivenForce\0" as *const c_char, IN_UseGivenForce);
    //buttons
    Cmd_AddCommand("+attack\0" as *const c_char, IN_Button0Down); //attack
    Cmd_AddCommand("-attack\0" as *const c_char, IN_Button0Up);
    Cmd_AddCommand("+force_lightning\0" as *const c_char, IN_Button1Down); //force lightning
    Cmd_AddCommand("-force_lightning\0" as *const c_char, IN_Button1Up);
    Cmd_AddCommand("+useforce\0" as *const c_char, IN_Button2Down); //use current force power
    Cmd_AddCommand("-useforce\0" as *const c_char, IN_Button2Up);
    Cmd_AddCommand("+force_drain\0" as *const c_char, IN_Button3Down); //force drain
    Cmd_AddCommand("-force_drain\0" as *const c_char, IN_Button3Up);
    Cmd_AddCommand("+walk\0" as *const c_char, IN_Button4Down); //walking
    Cmd_AddCommand("-walk\0" as *const c_char, IN_Button4Up);
    Cmd_AddCommand("+use\0" as *const c_char, IN_Button5Down); //use object
    Cmd_AddCommand("-use\0" as *const c_char, IN_Button5Up);
    Cmd_AddCommand("+force_grip\0" as *const c_char, IN_Button6Down); //force jump
    Cmd_AddCommand("-force_grip\0" as *const c_char, IN_Button6Up);
    Cmd_AddCommand("+altattack\0" as *const c_char, IN_Button7Down); //altattack
    Cmd_AddCommand("-altattack\0" as *const c_char, IN_Button7Up);
    Cmd_AddCommand("+forcefocus\0" as *const c_char, IN_Button8Down); //special saber attacks
    Cmd_AddCommand("-forcefocus\0" as *const c_char, IN_Button8Up);
    Cmd_AddCommand("+block\0" as *const c_char, IN_Button8Down); //manual blocking
    Cmd_AddCommand("-block\0" as *const c_char, IN_Button8Up);
    //end buttons
    Cmd_AddCommand("+mlook\0" as *const c_char, IN_MLookDown);
    Cmd_AddCommand("-mlook\0" as *const c_char, IN_MLookUp);

    cl_nodelta = Cvar_Get("cl_nodelta\0" as *const c_char, "0\0" as *const c_char, 0);
    cl_debugMove = Cvar_Get("cl_debugMove\0" as *const c_char, "0\0" as *const c_char, 0);
}
