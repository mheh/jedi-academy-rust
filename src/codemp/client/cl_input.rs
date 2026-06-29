#![allow(non_snake_case)]

// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// cl.input.c  -- builds an intended movement command to send to the server

// #include "client.h"
// #ifdef _XBOX
// #include "../../code/client/cl_input_hotswap.h"
// #include "../xbox/XBVoice.h"
// #endif

use core::ffi::{c_int, c_char, c_void};
use std::ffi::CStr;

pub static mut frame_msec: core::ffi::c_uint = 0;
pub static mut old_com_frameTime: c_int = 0;

pub static mut cl_mPitchOverride: f32 = 0.0;
pub static mut cl_mYawOverride: f32 = 0.0;
pub static mut cl_mSensitivityOverride: f32 = 0.0;
pub static mut cl_bUseFighterPitch: bool = false;
pub static mut cl_crazyShipControls: bool = false;

// #ifdef VEH_CONTROL_SCHEME_4
// #define	OVERRIDE_MOUSE_SENSITIVITY 5.0f//20.0f = 180 degree turn in one mouse swipe across keyboard
// #else// VEH_CONTROL_SCHEME_4
const OVERRIDE_MOUSE_SENSITIVITY: f32 = 10.0; //20.0f = 180 degree turn in one mouse swipe across keyboard
// #endif// VEH_CONTROL_SCHEME_4

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

#[repr(C)]
pub struct kbutton_t {
    pub down: [c_int; 2],
    pub downtime: c_int,
    pub msec: c_int,
    pub active: bool,
    pub wasPressed: bool,
}

pub static mut in_left: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_right: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_forward: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_back: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_lookup: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_lookdown: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_moveleft: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_moveright: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_strafe: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_speed: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_up: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};
pub static mut in_down: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: false,
    wasPressed: false,
};

pub static mut in_buttons: [kbutton_t; 16] = [
    kbutton_t {
        down: [0; 2],
        downtime: 0,
        msec: 0,
        active: false,
        wasPressed: false,
    }; 16
];

pub static mut in_mlooking: bool = false;

// #ifdef _XBOX
// HotSwapManager swapMan1(HOTSWAP_ID_WHITE);
// HotSwapManager swapMan2(HOTSWAP_ID_BLACK);
//
//
// void IN_HotSwap1On(void)
// {
//	swapMan1.SetDown();
// }
//
//
// void IN_HotSwap2On(void)
// {
//	swapMan2.SetDown();
// }
//
//
// void IN_HotSwap1Off(void)
// {
//	swapMan1.SetUp();
// }
//
//
// void IN_HotSwap2Off(void)
// {
//	swapMan2.SetUp();
// }
//
//
// void CL_UpdateHotSwap(void)
// {
//	swapMan1.Update();
//	swapMan2.Update();
// }
//
//
// bool CL_ExtendSelectTime(void)
// {
//	return swapMan1.ButtonDown() || swapMan2.ButtonDown();
// }
//
// #endif

extern "C" {
    fn IN_Button11Down();
    fn IN_Button11Up();
    fn IN_Button10Down();
    fn IN_Button10Up();
    fn IN_Button6Down();
    fn IN_Button6Up();
    fn Cmd_Argv(n: c_int) -> *const c_char;
    fn atoi(s: *const c_char) -> c_int;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Cvar_VariableIntegerValue(name: *const c_char) -> c_int;
    fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
    fn IN_CenterView();
    fn VM_Call(vm: *mut c_void, command: c_int, ...) -> c_int;
    fn Cvar_Get(
        var_name: *const c_char,
        var_value: *const c_char,
        flags: c_int,
    ) -> *mut c_void;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Sys_Milliseconds() -> c_int;
    fn Com_Memset(dst: *mut c_void, c: c_int, count: usize) -> *mut c_void;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn Cmd_AddCommand(cmd_name: *const c_char, function: unsafe extern "C" fn());
    fn Com_HashKey(key: *const c_char, length: c_int) -> c_int;
    fn ClampChar(i: c_int) -> c_char;
    fn SQRTFAST(x: f32) -> f32;
    fn AngleSubtract(a1: f32, a2: f32) -> f32;
    fn AngleNormalize180(angle: f32) -> f32;
    fn AngleNormalize360(angle: f32) -> f32;
    fn SHORT2ANGLE(x: i16) -> f32;
    fn ANGLE2SHORT(x: f32) -> i16;
    fn Q_fabs(x: f32) -> f32;
    fn MSG_Init(buf: *mut c_void, data: *mut c_void, length: c_int);
    fn MSG_Bitstream(buf: *mut c_void);
    fn MSG_WriteLong(buf: *mut c_void, c: c_int);
    fn MSG_WriteByte(buf: *mut c_void, c: c_int);
    fn MSG_WriteString(buf: *mut c_void, s: *const c_char);
    fn MSG_WriteDeltaUsercmdKey(
        buf: *mut c_void,
        key: c_int,
        from: *mut c_void,
        to: *mut c_void,
    );
    fn CL_Netchan_Transmit(netchan: *mut c_void, msg: *mut c_void);
    fn CL_Netchan_TransmitNextFragment(netchan: *mut c_void);
    fn SCR_DebugGraph(value: c_int, color: c_int);
    fn Sys_IsLANAddress(adr: *mut c_void) -> bool;
}

pub unsafe extern "C" fn IN_UseGivenForce() {
    let c = Cmd_Argv(1);
    let mut forceNum: c_int = -1;
    let mut genCmdNum: c_int = 0;

    if !c.is_null() && *c as c_char != 0 {
        forceNum = atoi(c);
    } else {
        return;
    }

    match forceNum {
        121 => {
            // FP_DRAIN
            IN_Button11Down();
            IN_Button11Up();
        }
        122 => {
            // FP_PUSH
            genCmdNum = 3; // GENCMD_FORCE_THROW
        }
        123 => {
            // FP_SPEED
            genCmdNum = 4; // GENCMD_FORCE_SPEED
        }
        124 => {
            // FP_PULL
            genCmdNum = 5; // GENCMD_FORCE_PULL
        }
        125 => {
            // FP_TELEPATHY
            genCmdNum = 6; // GENCMD_FORCE_DISTRACT
        }
        126 => {
            // FP_GRIP
            IN_Button6Down();
            IN_Button6Up();
        }
        127 => {
            // FP_LIGHTNING
            IN_Button10Down();
            IN_Button10Up();
        }
        128 => {
            // FP_RAGE
            genCmdNum = 7; // GENCMD_FORCE_RAGE
        }
        129 => {
            // FP_PROTECT
            genCmdNum = 8; // GENCMD_FORCE_PROTECT
        }
        130 => {
            // FP_ABSORB
            genCmdNum = 9; // GENCMD_FORCE_ABSORB
        }
        131 => {
            // FP_SEE
            genCmdNum = 10; // GENCMD_FORCE_SEEING
        }
        132 => {
            // FP_HEAL
            genCmdNum = 11; // GENCMD_FORCE_HEAL
        }
        133 => {
            // FP_TEAM_HEAL
            genCmdNum = 12; // GENCMD_FORCE_HEALOTHER
        }
        134 => {
            // FP_TEAM_FORCE
            genCmdNum = 13; // GENCMD_FORCE_FORCEPOWEROTHER
        }
        _ => {
            // assert(0);
        }
    }

    if genCmdNum != 0 {
        // cl.gcmdSendValue = qtrue;
        // cl.gcmdValue = genCmdNum;
    }
}

pub unsafe extern "C" fn IN_MLookDown() {
    in_mlooking = true;
}

pub unsafe extern "C" fn IN_MLookUp() {
    in_mlooking = false;
    // if (!cl_freelook->integer) {
    //	IN_CenterView();
    // }
}

pub unsafe extern "C" fn IN_GenCMD1() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_SABERSWITCH;
}

pub unsafe extern "C" fn IN_GenCMD2() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_ENGAGE_DUEL;
}

pub unsafe extern "C" fn IN_GenCMD3() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_HEAL;
}

pub unsafe extern "C" fn IN_GenCMD4() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_SPEED;
}

pub unsafe extern "C" fn IN_GenCMD5() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_PULL;
}

pub unsafe extern "C" fn IN_GenCMD6() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_DISTRACT;
}

pub unsafe extern "C" fn IN_GenCMD7() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_RAGE;
}

pub unsafe extern "C" fn IN_GenCMD8() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_PROTECT;
}

pub unsafe extern "C" fn IN_GenCMD9() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_ABSORB;
}

pub unsafe extern "C" fn IN_GenCMD10() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_HEALOTHER;
}

pub unsafe extern "C" fn IN_GenCMD11() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_FORCEPOWEROTHER;
}

pub unsafe extern "C" fn IN_GenCMD12() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_SEEING;
}

pub unsafe extern "C" fn IN_GenCMD13() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_SEEKER;
}

pub unsafe extern "C" fn IN_GenCMD14() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_FIELD;
}

pub unsafe extern "C" fn IN_GenCMD15() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_BACTA;
}

pub unsafe extern "C" fn IN_GenCMD16() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_ELECTROBINOCULARS;
}

pub unsafe extern "C" fn IN_GenCMD17() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_ZOOM;
}

pub unsafe extern "C" fn IN_GenCMD18() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_SENTRY;
}

pub unsafe extern "C" fn IN_GenCMD19() {
    // #ifdef _XBOX
    //	if (cl.snap.ps.weapon != WP_SABER)
    //	{
    //		Cbuf_ExecuteText(EXEC_APPEND, "weapon 1");
    //		return;
    //	}
    // #endif
    //
    //	if (Cvar_VariableIntegerValue("d_saberStanceDebug"))
    //	{
    //		Com_Printf("SABERSTANCEDEBUG: Gencmd on client set successfully.\n");
    //	}
    //	cl.gcmdSendValue = qtrue;
    //	cl.gcmdValue = GENCMD_SABERATTACKCYCLE;
}

pub unsafe extern "C" fn IN_GenCMD20() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FORCE_THROW;
}

pub unsafe extern "C" fn IN_GenCMD21() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_JETPACK;
}

pub unsafe extern "C" fn IN_GenCMD22() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_BACTABIG;
}

pub unsafe extern "C" fn IN_GenCMD23() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_HEALTHDISP;
}

pub unsafe extern "C" fn IN_GenCMD24() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_AMMODISP;
}

pub unsafe extern "C" fn IN_GenCMD25() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_EWEB;
}

pub unsafe extern "C" fn IN_GenCMD26() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_USE_CLOAK;
}

pub unsafe extern "C" fn IN_GenCMD27() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_TAUNT;
}

pub unsafe extern "C" fn IN_GenCMD28() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_BOW;
}

pub unsafe extern "C" fn IN_GenCMD29() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_MEDITATE;
}

pub unsafe extern "C" fn IN_GenCMD30() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_FLOURISH;
}

pub unsafe extern "C" fn IN_GenCMD31() {
    // cl.gcmdSendValue = qtrue;
    // cl.gcmdValue = GENCMD_GLOAT;
}

//toggle automap view mode
static mut g_clAutoMapMode: bool = false;

pub unsafe extern "C" fn IN_AutoMapButton() {
    g_clAutoMapMode = !g_clAutoMapMode;
}

//toggle between automap, radar, nothing
// extern cvar_t *r_autoMap;
pub unsafe extern "C" fn IN_AutoMapToggle() {
    if Cvar_VariableIntegerValue(b"cg_drawRadar\0".as_ptr() as *const c_char) != 0 {
        Cvar_Set(b"cg_drawRadar\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
    } else {
        Cvar_Set(b"cg_drawRadar\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);
    }
    /*
    if (r_autoMap && r_autoMap->integer)
    { //automap off, radar on
        Cvar_Set("r_autoMap", "0");
        Cvar_Set("cg_drawRadar", "1");
    }
    else if (Cvar_VariableIntegerValue("cg_drawRadar"))
    { //radar off, automap should be off too
        Cvar_Set("cg_drawRadar", "0");
    }
    else
    { //turn automap on
        Cvar_Set("r_autoMap", "1");
    }
    */
}

pub unsafe extern "C" fn IN_VoiceChatButton() {
    // if (!uivm) { //ui not loaded so this command is useless
    //	return;
    // }
    // VM_Call( uivm, UI_SET_ACTIVE_MENU, UIMENU_VOICECHAT );
}

pub unsafe extern "C" fn IN_KeyDown(b: *mut kbutton_t) {
    let mut k: c_int;
    let c = Cmd_Argv(1);

    if !c.is_null() && *c as c_char != 0 {
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
        Com_Printf(b"Three keys down for a button!\n\0".as_ptr() as *const c_char);
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

pub unsafe extern "C" fn IN_KeyUp(b: *mut kbutton_t) {
    let mut k: c_int;
    let c = Cmd_Argv(1);
    let mut uptime: core::ffi::c_uint;

    if !c.is_null() && *c as c_char != 0 {
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
        (*b).msec += (uptime - (*b).downtime as core::ffi::c_uint) as c_int;
    } else {
        (*b).msec += frame_msec as c_int / 2;
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
        // if (!(*key).downtime) {
        //	msec = com_frameTime;
        // } else {
        //	msec += com_frameTime - (*key).downtime;
        // }
        // (*key).downtime = com_frameTime;
    }

    // #if 0
    //	if (msec) {
    //		Com_Printf ("%i ", msec);
    //	}
    // #endif

    val = msec as f32 / frame_msec as f32;
    if val < 0.0 {
        val = 0.0;
    }
    if val > 1.0 {
        val = 1.0;
    }

    val
}

const AUTOMAP_KEY_FORWARD: c_int = 1;
const AUTOMAP_KEY_BACK: c_int = 2;
const AUTOMAP_KEY_YAWLEFT: c_int = 3;
const AUTOMAP_KEY_YAWRIGHT: c_int = 4;
const AUTOMAP_KEY_PITCHUP: c_int = 5;
const AUTOMAP_KEY_PITCHDOWN: c_int = 6;
const AUTOMAP_KEY_DEFAULTVIEW: c_int = 7;

#[repr(C)]
pub struct autoMapInput_t {
    pub up: f32,
    pub down: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub goToDefaults: bool,
}

static mut g_clAutoMapInput: autoMapInput_t = autoMapInput_t {
    up: 0.0,
    down: 0.0,
    yaw: 0.0,
    pitch: 0.0,
    goToDefaults: false,
};

//intercept certain keys during automap mode
unsafe fn CL_AutoMapKey(autoMapKey: c_int, up: bool) {
    // autoMapInput_t *data = (autoMapInput_t *)cl.mSharedMemory;

    match autoMapKey {
        AUTOMAP_KEY_FORWARD => {
            if up {
                g_clAutoMapInput.up = 0.0;
            } else {
                g_clAutoMapInput.up = 16.0;
            }
        }
        AUTOMAP_KEY_BACK => {
            if up {
                g_clAutoMapInput.down = 0.0;
            } else {
                g_clAutoMapInput.down = 16.0;
            }
        }
        AUTOMAP_KEY_YAWLEFT => {
            if up {
                g_clAutoMapInput.yaw = 0.0;
            } else {
                g_clAutoMapInput.yaw = -4.0;
            }
        }
        AUTOMAP_KEY_YAWRIGHT => {
            if up {
                g_clAutoMapInput.yaw = 0.0;
            } else {
                g_clAutoMapInput.yaw = 4.0;
            }
        }
        AUTOMAP_KEY_PITCHUP => {
            if up {
                g_clAutoMapInput.pitch = 0.0;
            } else {
                g_clAutoMapInput.pitch = -4.0;
            }
        }
        AUTOMAP_KEY_PITCHDOWN => {
            if up {
                g_clAutoMapInput.pitch = 0.0;
            } else {
                g_clAutoMapInput.pitch = 4.0;
            }
        }
        AUTOMAP_KEY_DEFAULTVIEW => {
            memset(
                &mut g_clAutoMapInput as *mut _ as *mut c_void,
                0,
                std::mem::size_of::<autoMapInput_t>(),
            );
            g_clAutoMapInput.goToDefaults = true;
        }
        _ => {}
    }

    // memcpy(data, &g_clAutoMapInput, sizeof(autoMapInput_t));

    // if (cgvm) {
    //	VM_Call(cgvm, CG_AUTOMAP_INPUT, 0);
    // }

    g_clAutoMapInput.goToDefaults = false;
}

pub unsafe extern "C" fn IN_UpDown() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_PITCHUP, false);
    } else {
        IN_KeyDown(&mut in_up);
    }
}

pub unsafe extern "C" fn IN_UpUp() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_PITCHUP, true);
    } else {
        IN_KeyUp(&mut in_up);
    }
}

pub unsafe extern "C" fn IN_DownDown() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_PITCHDOWN, false);
    } else {
        IN_KeyDown(&mut in_down);
    }
}

pub unsafe extern "C" fn IN_DownUp() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_PITCHDOWN, true);
    } else {
        IN_KeyUp(&mut in_down);
    }
}

pub unsafe extern "C" fn IN_LeftDown() {
    IN_KeyDown(&mut in_left);
}

pub unsafe extern "C" fn IN_LeftUp() {
    IN_KeyUp(&mut in_left);
}

pub unsafe extern "C" fn IN_RightDown() {
    IN_KeyDown(&mut in_right);
}

pub unsafe extern "C" fn IN_RightUp() {
    IN_KeyUp(&mut in_right);
}

pub unsafe extern "C" fn IN_ForwardDown() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_FORWARD, false);
    } else {
        IN_KeyDown(&mut in_forward);
    }
}

pub unsafe extern "C" fn IN_ForwardUp() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_FORWARD, true);
    } else {
        IN_KeyUp(&mut in_forward);
    }
}

pub unsafe extern "C" fn IN_BackDown() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_BACK, false);
    } else {
        IN_KeyDown(&mut in_back);
    }
}

pub unsafe extern "C" fn IN_BackUp() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_BACK, true);
    } else {
        IN_KeyUp(&mut in_back);
    }
}

pub unsafe extern "C" fn IN_LookupDown() {
    IN_KeyDown(&mut in_lookup);
}

pub unsafe extern "C" fn IN_LookupUp() {
    IN_KeyUp(&mut in_lookup);
}

pub unsafe extern "C" fn IN_LookdownDown() {
    IN_KeyDown(&mut in_lookdown);
}

pub unsafe extern "C" fn IN_LookdownUp() {
    IN_KeyUp(&mut in_lookdown);
}

pub unsafe extern "C" fn IN_MoveleftDown() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_YAWLEFT, false);
    } else {
        IN_KeyDown(&mut in_moveleft);
    }
}

pub unsafe extern "C" fn IN_MoveleftUp() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_YAWLEFT, true);
    } else {
        IN_KeyUp(&mut in_moveleft);
    }
}

pub unsafe extern "C" fn IN_MoverightDown() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_YAWRIGHT, false);
    } else {
        IN_KeyDown(&mut in_moveright);
    }
}

pub unsafe extern "C" fn IN_MoverightUp() {
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_YAWRIGHT, true);
    } else {
        IN_KeyUp(&mut in_moveright);
    }
}

// #ifdef _XBOX
// // BTO - State for auto-level usage. This REALLY ought to be in cl[sc ].
// // Honestly, I don't want to figure out which one, especially anticipating
// // split-screen. All I want for Christmas is to not do split-screen.
// static unsigned long sLastFireTime = 0;
// #endif

pub unsafe extern "C" fn IN_SpeedDown() {
    IN_KeyDown(&mut in_speed);
}

pub unsafe extern "C" fn IN_SpeedUp() {
    IN_KeyUp(&mut in_speed);
}

pub unsafe extern "C" fn IN_StrafeDown() {
    IN_KeyDown(&mut in_strafe);
}

pub unsafe extern "C" fn IN_StrafeUp() {
    IN_KeyUp(&mut in_strafe);
}

pub unsafe extern "C" fn IN_Button0Down() {
    IN_KeyDown(&mut in_buttons[0]);
}

pub unsafe extern "C" fn IN_Button0Up() {
    IN_KeyUp(&mut in_buttons[0]);
    // #ifdef _XBOX	// Auto-level. Thought this was nasty, but now I sort of like it.
    //	sLastFireTime = Sys_Milliseconds();
    // #endif
}

pub unsafe extern "C" fn IN_Button1Down() {
    IN_KeyDown(&mut in_buttons[1]);
}

pub unsafe extern "C" fn IN_Button1Up() {
    IN_KeyUp(&mut in_buttons[1]);
}

pub unsafe extern "C" fn IN_Button2Down() {
    IN_KeyDown(&mut in_buttons[2]);
}

pub unsafe extern "C" fn IN_Button2Up() {
    IN_KeyUp(&mut in_buttons[2]);
}

pub unsafe extern "C" fn IN_Button3Down() {
    IN_KeyDown(&mut in_buttons[3]);
}

pub unsafe extern "C" fn IN_Button3Up() {
    IN_KeyUp(&mut in_buttons[3]);
}

pub unsafe extern "C" fn IN_Button4Down() {
    IN_KeyDown(&mut in_buttons[4]);
}

pub unsafe extern "C" fn IN_Button4Up() {
    IN_KeyUp(&mut in_buttons[4]);
}

pub unsafe extern "C" fn IN_Button5Down() {
    //use key
    if g_clAutoMapMode {
        CL_AutoMapKey(AUTOMAP_KEY_DEFAULTVIEW, false);
    } else {
        IN_KeyDown(&mut in_buttons[5]);
    }
}

pub unsafe extern "C" fn IN_Button5Up() {
    IN_KeyUp(&mut in_buttons[5]);
}

pub unsafe extern "C" fn IN_Button6Down() {
    IN_KeyDown(&mut in_buttons[6]);
}

pub unsafe extern "C" fn IN_Button6Up() {
    IN_KeyUp(&mut in_buttons[6]);
}

pub unsafe extern "C" fn IN_Button7Down() {
    IN_KeyDown(&mut in_buttons[7]);
}

pub unsafe extern "C" fn IN_Button7Up() {
    IN_KeyUp(&mut in_buttons[7]);
    // #ifdef _XBOX	// Auto-level. Thought this was nasty, but now I sort of like it.
    //	sLastFireTime = Sys_Milliseconds();
    // #endif
}

pub unsafe extern "C" fn IN_Button8Down() {
    IN_KeyDown(&mut in_buttons[8]);
}

pub unsafe extern "C" fn IN_Button8Up() {
    IN_KeyUp(&mut in_buttons[8]);
}

pub unsafe extern "C" fn IN_Button9Down() {
    IN_KeyDown(&mut in_buttons[9]);
}

pub unsafe extern "C" fn IN_Button9Up() {
    IN_KeyUp(&mut in_buttons[9]);
}

pub unsafe extern "C" fn IN_Button10Down() {
    IN_KeyDown(&mut in_buttons[10]);
}

pub unsafe extern "C" fn IN_Button10Up() {
    IN_KeyUp(&mut in_buttons[10]);
}

pub unsafe extern "C" fn IN_Button11Down() {
    IN_KeyDown(&mut in_buttons[11]);
}

pub unsafe extern "C" fn IN_Button11Up() {
    IN_KeyUp(&mut in_buttons[11]);
}

pub unsafe extern "C" fn IN_Button12Down() {
    IN_KeyDown(&mut in_buttons[12]);
}

pub unsafe extern "C" fn IN_Button12Up() {
    IN_KeyUp(&mut in_buttons[12]);
}

pub unsafe extern "C" fn IN_Button13Down() {
    IN_KeyDown(&mut in_buttons[13]);
}

pub unsafe extern "C" fn IN_Button13Up() {
    IN_KeyUp(&mut in_buttons[13]);
}

pub unsafe extern "C" fn IN_Button14Down() {
    IN_KeyDown(&mut in_buttons[14]);
}

pub unsafe extern "C" fn IN_Button14Up() {
    IN_KeyUp(&mut in_buttons[14]);
}

pub unsafe extern "C" fn IN_Button15Down() {
    IN_KeyDown(&mut in_buttons[15]);
}

pub unsafe extern "C" fn IN_Button15Up() {
    IN_KeyUp(&mut in_buttons[15]);
}

pub unsafe extern "C" fn IN_ButtonDown() {
    IN_KeyDown(&mut in_buttons[1]);
}

pub unsafe extern "C" fn IN_ButtonUp() {
    IN_KeyUp(&mut in_buttons[1]);
}

pub unsafe extern "C" fn IN_CenterViewLocal() {
    // cl.viewangles[PITCH] = -SHORT2ANGLE(cl.snap.ps.delta_angles[PITCH]);
}

// #ifdef _XBOX
// void IN_VoiceToggleDown(void) { g_Voice.SetChannel( CHAN_ALL ); }
// void IN_VoiceToggleUp(void) { g_Voice.SetChannel( CHAN_TEAM ); }
// #endif

//==========================================================================

// cvar_t	*cl_upspeed;
// cvar_t	*cl_forwardspeed;
// cvar_t	*cl_sidespeed;
//
// cvar_t	*cl_yawspeed;
// cvar_t	*cl_pitchspeed;
//
// cvar_t	*cl_run;
//
// cvar_t	*cl_anglespeedkey;

/*
================
CL_AdjustAngles

Moves the local angle positions
================
*/
pub unsafe fn CL_AdjustAngles() {
    // float	speed;
    //
    // if ( in_speed.active ) {
    //	speed = 0.001 * cls.frametime * cl_anglespeedkey->value;
    // } else {
    //	speed = 0.001 * cls.frametime;
    // }
    //
    // if ( !in_strafe.active ) {
    //	if ( cl_mYawOverride )
    //	{
    //		if ( cl_mSensitivityOverride )
    //		{
    //			cl.viewangles[YAW] -= cl_mYawOverride*cl_mSensitivityOverride*speed*cl_yawspeed->value*CL_KeyState (&in_right);
    //			cl.viewangles[YAW] += cl_mYawOverride*cl_mSensitivityOverride*speed*cl_yawspeed->value*CL_KeyState (&in_left);
    //		}
    //		else
    //		{
    //			cl.viewangles[YAW] -= cl_mYawOverride*OVERRIDE_MOUSE_SENSITIVITY*speed*cl_yawspeed->value*CL_KeyState (&in_right);
    //			cl.viewangles[YAW] += cl_mYawOverride*OVERRIDE_MOUSE_SENSITIVITY*speed*cl_yawspeed->value*CL_KeyState (&in_left);
    //		}
    //	}
    //	else
    //	{
    //		cl.viewangles[YAW] -= speed*cl_yawspeed->value*CL_KeyState (&in_right);
    //		cl.viewangles[YAW] += speed*cl_yawspeed->value*CL_KeyState (&in_left);
    //	}
    // }
    //
    // if ( cl_mPitchOverride )
    // {
    //	if ( cl_mSensitivityOverride )
    //	{
    //		cl.viewangles[PITCH] -= cl_mPitchOverride*cl_mSensitivityOverride*speed*cl_pitchspeed->value * CL_KeyState (&in_lookup);
    //		cl.viewangles[PITCH] += cl_mPitchOverride*cl_mSensitivityOverride*speed*cl_pitchspeed->value * CL_KeyState (&in_lookdown);
    //	}
    //	else
    //	{
    //		cl.viewangles[PITCH] -= cl_mPitchOverride*OVERRIDE_MOUSE_SENSITIVITY*speed*cl_pitchspeed->value * CL_KeyState (&in_lookup);
    //		cl.viewangles[PITCH] += cl_mPitchOverride*OVERRIDE_MOUSE_SENSITIVITY*speed*cl_pitchspeed->value * CL_KeyState (&in_lookdown);
    //	}
    // }
    // else
    // {
    //	cl.viewangles[PITCH] -= speed*cl_pitchspeed->value * CL_KeyState (&in_lookup);
    //	cl.viewangles[PITCH] += speed*cl_pitchspeed->value * CL_KeyState (&in_lookdown);
    // }
}

/*
================
CL_KeyMove

Sets the usercmd_t based on key states
================
*/
pub unsafe fn CL_KeyMove(cmd: *mut c_void) {
    // int		movespeed;
    // int		forward, side, up;
    //
    // //
    // // adjust for speed key / running
    // // the walking flag is to keep animations consistant
    // // even during acceleration and develeration
    // //
    // if ( in_speed.active ^ cl_run->integer ) {
    //	movespeed = 127;
    //	cmd->buttons &= ~BUTTON_WALKING;
    // } else {
    //	cmd->buttons |= BUTTON_WALKING;
    //	movespeed = 46;
    // }
    //
    // forward = 0;
    // side = 0;
    // up = 0;
    // if ( in_strafe.active ) {
    //	side += movespeed * CL_KeyState (&in_right);
    //	side -= movespeed * CL_KeyState (&in_left);
    // }
    //
    // side += movespeed * CL_KeyState (&in_moveright);
    // side -= movespeed * CL_KeyState (&in_moveleft);
    //
    //
    // up += movespeed * CL_KeyState (&in_up);
    // up -= movespeed * CL_KeyState (&in_down);
    //
    // forward += movespeed * CL_KeyState (&in_forward);
    // forward -= movespeed * CL_KeyState (&in_back);
    //
    // cmd->forwardmove = ClampChar( forward );
    // cmd->rightmove = ClampChar( side );
    // cmd->upmove = ClampChar( up );
}

/*
=================
CL_MouseEvent
=================
*/
pub unsafe extern "C" fn CL_MouseEvent(dx: c_int, dy: c_int, time: c_int) {
    // if (g_clAutoMapMode && cgvm)
    // { //automap input
    //	autoMapInput_t *data = (autoMapInput_t *)cl.mSharedMemory;
    //
    //	g_clAutoMapInput.yaw = dx;
    //	g_clAutoMapInput.pitch = dy;
    //	memcpy(data, &g_clAutoMapInput, sizeof(autoMapInput_t));
    //	VM_Call(cgvm, CG_AUTOMAP_INPUT, 1);
    //
    //	g_clAutoMapInput.yaw = 0.0f;
    //	g_clAutoMapInput.pitch = 0.0f;
    // }
    // else if ( cls.keyCatchers & KEYCATCH_UI ) {
    //	VM_Call( uivm, UI_MOUSE_EVENT, dx, dy );
    // } else if (cls.keyCatchers & KEYCATCH_CGAME) {
    //	VM_Call (cgvm, CG_MOUSE_EVENT, dx, dy);
    // } else {
    //	cl.mouseDx[cl.mouseIndex] += dx;
    //	cl.mouseDy[cl.mouseIndex] += dy;
    // }
}

/*
=================
CL_JoystickEvent

Joystick values stay set until changed
=================
*/
pub unsafe extern "C" fn CL_JoystickEvent(axis: c_int, value: c_int, time: c_int) {
    // if ( axis < 0 || axis >= MAX_JOYSTICK_AXIS ) {
    //	Com_Error( ERR_DROP, "CL_JoystickEvent: bad axis %i", axis );
    // }
    // cl.joystickAxis[axis] = value;
}

/*
=================
CL_JoystickMove
=================
*/
// extern cvar_t *in_joystick;
pub unsafe fn CL_JoystickMove(cmd: *mut c_void) {
    // #ifndef _XBOX	// We always have a joystick, won't bother adding another cvar
    //	if ( !in_joystick->integer )
    //	{
    //		return;
    //	}
    // #endif
    //
    //	int		movespeed;
    //	float	anglespeed;
    //
    //	if ( in_speed.active ^ cl_run->integer ) {
    //		movespeed = 2;
    //	} else {
    //		movespeed = 1;
    //		cmd->buttons |= BUTTON_WALKING;
    //	}
    //
    //	if ( in_speed.active ) {
    //		anglespeed = 0.001 * cls.frametime * cl_anglespeedkey->value;
    //	} else {
    //		anglespeed = 0.001 * cls.frametime;
    //	}
    //
    // #ifndef _XBOX
    //	if ( !in_strafe.active ) {
    //		if ( cl_mYawOverride )
    //		{
    //			if ( cl_mSensitivityOverride )
    //			{
    //				cl.viewangles[YAW] += cl_mYawOverride * cl_mSensitivityOverride * cl.joystickAxis[AXIS_SIDE]/2.0f;
    //			}
    //			else
    //			{
    //				cl.viewangles[YAW] += cl_mYawOverride * OVERRIDE_MOUSE_SENSITIVITY * cl.joystickAxis[AXIS_SIDE]/2.0f;
    //			}
    //		}
    //		else
    //		{
    //			cl.viewangles[YAW] += anglespeed * (cl_yawspeed->value / 100.0f) * cl.joystickAxis[AXIS_SIDE];
    //		}
    //	} else
    // #endif
    //	{
    //		cmd->rightmove = ClampChar( cmd->rightmove + cl.joystickAxis[AXIS_SIDE] );
    //	}
    //
    // #ifndef _XBOX
    //	if ( in_mlooking || cl_freelook->integer ) {
    //		if ( cl_mPitchOverride )
    //		{
    //			if ( cl_mSensitivityOverride )
    //			{
    //				cl.viewangles[PITCH] += cl_mPitchOverride * cl_mSensitivityOverride * cl.joystickAxis[AXIS_FORWARD]/2.0f;
    //			}
    //			else
    //			{
    //				cl.viewangles[PITCH] += cl_mPitchOverride * OVERRIDE_MOUSE_SENSITIVITY * cl.joystickAxis[AXIS_FORWARD]/2.0f;
    //			}
    //		}
    //		else
    //		{
    //			cl.viewangles[PITCH] += anglespeed * (cl_pitchspeed->value / 100.0f) * cl.joystickAxis[AXIS_FORWARD];
    //		}
    //	} else
    // #endif
    //	{
    //		cmd->forwardmove = ClampChar( cmd->forwardmove + cl.joystickAxis[AXIS_FORWARD] );
    //	}
    //
    //	cmd->upmove = ClampChar( cmd->upmove + cl.joystickAxis[AXIS_UP] );
}

/*
=================
CL_MouseMove
=================
*/
// #ifdef _XBOX
// void CL_MouseClamp(int *x, int *y)
// {
//	float ax = Q_fabs(*x);
//	float ay = Q_fabs(*y);
//
//	ax = (ax-10)*(3.0f/45.0f) * (ax-10) * (Q_fabs(*x) > 10);
//	ay = (ay-10)*(3.0f/45.0f) * (ay-10) * (Q_fabs(*y) > 10);
//	if (*x < 0)
//		*x = -ax;
//	else
//		*x = ax;
//	if (*y < 0)
//		*y = -ay;
//	else
//		*y = ay;
// }
// #endif

pub unsafe fn CL_MouseMove(cmd: *mut c_void) {
    // float	mx, my;
    // float	accelSensitivity;
    // float	rate;
    // const float	speed = static_cast<float>(frame_msec);
    // const float pitch = cl_bUseFighterPitch?m_pitchVeh->value:m_pitch->value;
    //
    // #ifdef _XBOX
    //	const float mouseSpeedX = 0.06f;
    //	const float mouseSpeedY = 0.05f;
    //
    //	// allow mouse smoothing
    //	if ( m_filter->integer ) {
    //		mx = ( cl.mouseDx[0] + cl.mouseDx[1] ) * 0.5f * frame_msec * mouseSpeedX;
    //		my = ( cl.mouseDy[0] + cl.mouseDy[1] ) * 0.5f * frame_msec * mouseSpeedY;
    //	} else {
    //		int ax = cl.mouseDx[cl.mouseIndex];
    //		int ay = cl.mouseDy[cl.mouseIndex];
    //		CL_MouseClamp(&ax, &ay);
    //
    //		mx = ax * speed * mouseSpeedX;
    //		my = ay * speed * mouseSpeedY;
    //	}
    //
    //	extern int cg_crossHairStatus;
    //	const float m_hoverSensitivity = 0.4f;
    //	if (cg_crossHairStatus)
    //	{
    //		mx *= m_hoverSensitivity;
    //		my *= m_hoverSensitivity;
    //	}
    // #else
    //	// allow mouse smoothing
    //	if ( m_filter->integer ) {
    //		mx = ( cl.mouseDx[0] + cl.mouseDx[1] ) * 0.5;
    //		my = ( cl.mouseDy[0] + cl.mouseDy[1] ) * 0.5;
    //	} else {
    //		mx = cl.mouseDx[cl.mouseIndex];
    //		my = cl.mouseDy[cl.mouseIndex];
    //	}
    // #endif
    //
    //	cl.mouseIndex ^= 1;
    //	cl.mouseDx[cl.mouseIndex] = 0;
    //	cl.mouseDy[cl.mouseIndex] = 0;
    //
    //	rate = SQRTFAST( mx * mx + my * my ) / speed;
    //	if ( cl_mYawOverride || cl_mPitchOverride )
    //	{//FIXME: different people have different speed mouses,
    //		if ( cl_mSensitivityOverride )
    //		{
    //			//this will fuck things up for them, need to clamp
    //			//max input?
    //			accelSensitivity = cl_mSensitivityOverride;
    //		}
    //		else
    //		{
    //			accelSensitivity = cl_sensitivity->value + rate * cl_mouseAccel->value;
    //			// scale by FOV
    //			accelSensitivity *= cl.cgameSensitivity;
    //		}
    //	}
    //	else
    //	{
    //		accelSensitivity = cl_sensitivity->value + rate * cl_mouseAccel->value;
    //		// scale by FOV
    //		accelSensitivity *= cl.cgameSensitivity;
    //	}
    //
    //	if ( rate && cl_showMouseRate->integer ) {
    //		Com_Printf( "%f : %f\n", rate, accelSensitivity );
    //	}
    //
    //	mx *= accelSensitivity;
    //	my *= accelSensitivity;
    //
    //	if (!mx && !my) {
    // #ifdef _XBOX
    //		// If there was a movement but no change in angles then start auto-leveling the camera
    //		float autolevelSpeed = 0.03f;
    //
    //		if (!cg_crossHairStatus &&								// Not looking at an enemy
    //			cl.joystickAxis[AXIS_FORWARD] &&					// Moving forward/backward
    //			cl.snap.ps.groundEntityNum != ENTITYNUM_NONE &&		// Not in the air
    //			Cvar_VariableIntegerValue("cl_autolevel") &&		// Autolevel is turned on
    //			!(in_buttons[0].active || in_buttons[7].active) &&	// Not firing a weapon
    //			sLastFireTime < Sys_Milliseconds() - 1000)			// Haven't fired recently
    //		{
    //			float normAngle = -SHORT2ANGLE(cl.snap.ps.delta_angles[PITCH]);
    //			// The adjustment to normAngle below is meant to add or remove some multiple
    //			// of 360, so that normAngle is within 180 of viewangles[PITCH]. It should
    //			// be correct.
    //			int diff = (int)(cl.viewangles[PITCH] - normAngle);
    //			if (diff > 180)
    //				normAngle += 360.0f * ((diff+180) / 360);
    //			else if (diff < -180)
    //				normAngle -= 360.0f * ((-diff+180) / 360);
    //
    //			if (Cvar_VariableIntegerValue("cg_thirdperson") == 1)
    //			{
    //				normAngle += 10;
    //				autolevelSpeed *= 1.5f;
    //			}
    //			if (cl.viewangles[PITCH] > normAngle)
    //			{
    //				cl.viewangles[PITCH] -= autolevelSpeed * speed;
    //				if (cl.viewangles[PITCH] < normAngle) cl.viewangles[PITCH] = normAngle;
    //			}
    //			else if (cl.viewangles[PITCH] < normAngle)
    //			{
    //				cl.viewangles[PITCH] += autolevelSpeed * speed;
    //				if (cl.viewangles[PITCH] > normAngle) cl.viewangles[PITCH] = normAngle;
    //			}
    //		}
    // #endif
    //		return;
    //	}
    //
    //	// add mouse X/Y movement to cmd
    //	if ( in_strafe.active ) {
    //		cmd->rightmove = ClampChar( cmd->rightmove + m_side->value * mx );
    //	} else {
    //		if ( cl_mYawOverride )
    //		{
    //			cl.viewangles[YAW] -= cl_mYawOverride * mx;
    //		}
    //		else
    //		{
    //			cl.viewangles[YAW] -= m_yaw->value * mx;
    //		}
    //	}
    //
    //	if ( (in_mlooking || cl_freelook->integer) && !in_strafe.active ) {
    //		// VVFIXME - This is supposed to be a CVAR
    // #ifdef _XBOX
    //		const float cl_pitchSensitivity = 0.5f;
    // #else
    //		const float cl_pitchSensitivity = 1.0f;
    // #endif
    //		if ( cl_mPitchOverride )
    //		{
    //			if ( pitch > 0 )
    //			{
    //				cl.viewangles[PITCH] += cl_mPitchOverride * my * cl_pitchSensitivity;
    //			}
    //			else
    //			{
    //				cl.viewangles[PITCH] -= cl_mPitchOverride * my * cl_pitchSensitivity;
    //			}
    //		}
    //		else
    //		{
    //			cl.viewangles[PITCH] += pitch * my * cl_pitchSensitivity;
    //		}
    //	} else {
    //		cmd->forwardmove = ClampChar( cmd->forwardmove - m_forward->value * my );
    //	}
}

pub unsafe fn CL_NoUseableForce() -> bool {
    // if (!cgvm) { //ahh, no cgame loaded
    //	return false;
    // }
    //
    // return (qboolean)VM_Call(cgvm, CG_GET_USEABLE_FORCE);
    false
}

/*
==============
CL_CmdButtons
==============
*/
pub unsafe fn CL_CmdButtons(cmd: *mut c_void) {
    // int		i;
    //
    // //
    // // figure button bits
    // // send a button bit even if the key was pressed and released in
    // // less than a frame
    // //
    // for (i = 0 ; i < 15 ; i++) {
    //	if ( in_buttons[i].active || in_buttons[i].wasPressed ) {
    //		cmd->buttons |= 1 << i;
    //	}
    //	in_buttons[i].wasPressed = qfalse;
    // }
    //
    // if (cmd->buttons & BUTTON_FORCEPOWER)
    // { //check for transferring a use force to a use inventory...
    //	if ((cmd->buttons & BUTTON_USE) || CL_NoUseableForce())
    //	{ //it's pushed, remap it!
    //		cmd->buttons &= ~BUTTON_FORCEPOWER;
    //		cmd->buttons |= BUTTON_USE_HOLDABLE;
    //	}
    // }
    //
    // if ( cls.keyCatchers ) {
    //	cmd->buttons |= BUTTON_TALK;
    // }
    //
    // // allow the game to know if any key at all is
    // // currently pressed, even if it isn't bound to anything
    // if ( kg.anykeydown && !cls.keyCatchers ) {
    //	cmd->buttons |= BUTTON_ANY;
    // }
}

/*
==============
CL_FinishMove
==============
*/
static mut cl_sendAngles: [f32; 3] = [0.0; 3];
static mut cl_lastViewAngles: [f32; 3] = [0.0; 3];

pub unsafe fn CL_FinishMove(cmd: *mut c_void) {
    // int		i;
    //
    // // copy the state that the cgame is currently sending
    // cmd->weapon = cl.cgameUserCmdValue;
    // cmd->forcesel = cl.cgameForceSelection;
    // cmd->invensel = cl.cgameInvenSelection;
    //
    // if (cl.gcmdSendValue)
    // {
    //	cmd->generic_cmd = cl.gcmdValue;
    //	//cl.gcmdSendValue = qfalse;
    //	cl.gcmdSentValue = qtrue;
    // }
    // else
    // {
    //	cmd->generic_cmd = 0;
    // }
    //
    // // send the current server time so the amount of movement
    // // can be determined without allowing cheating
    // cmd->serverTime = cl.serverTime;
    //
    // if (cl.cgameViewAngleForceTime > cl.serverTime)
    // {
    //	cl.cgameViewAngleForce[YAW] -= SHORT2ANGLE(cl.snap.ps.delta_angles[YAW]);
    //
    //	cl.viewangles[YAW] = cl.cgameViewAngleForce[YAW];
    //	cl.cgameViewAngleForceTime = 0;
    // }
    //
    // if ( cl_crazyShipControls )
    // {
    //	float pitchSubtract, pitchDelta, yawDelta;
    //
    //	yawDelta = AngleSubtract(cl.viewangles[YAW],cl_lastViewAngles[YAW]);
    //	//yawDelta *= (4.0f*pVeh->m_fTimeModifier);
    //	cl_sendAngles[ROLL] -= yawDelta;
    //
    //	float nRoll = fabs(cl_sendAngles[ROLL]);
    //
    //	pitchDelta = AngleSubtract(cl.viewangles[PITCH],cl_lastViewAngles[PITCH]);
    //	//pitchDelta *= (2.0f*pVeh->m_fTimeModifier);
    //	pitchSubtract = pitchDelta * (nRoll/90.0f);
    //	cl_sendAngles[PITCH] += pitchDelta-pitchSubtract;
    //
    //	//yaw-roll calc should be different
    //	if (nRoll > 90.0f)
    //	{
    //		nRoll -= 180.0f;
    //	}
    //	if (nRoll < 0.0f)
    //	{
    //		nRoll = -nRoll;
    //	}
    //	pitchSubtract = pitchDelta * (nRoll/90.0f);
    //	if ( cl_sendAngles[ROLL] > 0.0f )
    //	{
    //		cl_sendAngles[YAW] += pitchSubtract;
    //	}
    //	else
    //	{
    //		cl_sendAngles[YAW] -= pitchSubtract;
    //	}
    //
    //	cl_sendAngles[PITCH] = AngleNormalize180( cl_sendAngles[PITCH] );
    //	cl_sendAngles[YAW] = AngleNormalize360( cl_sendAngles[YAW] );
    //	cl_sendAngles[ROLL] = AngleNormalize180( cl_sendAngles[ROLL] );
    //
    //	for (i=0 ; i<3 ; i++) {
    //		cmd->angles[i] = ANGLE2SHORT(cl_sendAngles[i]);
    //	}
    // }
    // else
    // {
    //	for (i=0 ; i<3 ; i++) {
    //		cmd->angles[i] = ANGLE2SHORT(cl.viewangles[i]);
    //	}
    //	//in case we switch to the cl_crazyShipControls
    //	VectorCopy( cl.viewangles, cl_sendAngles );
    // }
    // //always needed in for the cl_crazyShipControls
    // VectorCopy( cl.viewangles, cl_lastViewAngles );
}

/*
=================
CL_CreateCmd
=================
*/
pub unsafe fn CL_CreateCmd() -> *mut c_void {
    // usercmd_t	cmd;
    // vec3_t		oldAngles;
    //
    // VectorCopy( cl.viewangles, oldAngles );
    //
    // // keyboard angle adjustment
    // CL_AdjustAngles ();
    //
    // Com_Memset( &cmd, 0, sizeof( cmd ) );
    //
    // CL_CmdButtons( &cmd );
    //
    // // get basic movement from keyboard
    // CL_KeyMove( &cmd );
    //
    // // get basic movement from mouse
    // CL_MouseMove( &cmd );
    //
    // // get basic movement from joystick
    // CL_JoystickMove( &cmd );
    //
    // // check to make sure the angles haven't wrapped
    // if ( cl.viewangles[PITCH] - oldAngles[PITCH] > 90 ) {
    //	cl.viewangles[PITCH] = oldAngles[PITCH] + 90;
    // } else if ( oldAngles[PITCH] - cl.viewangles[PITCH] > 90 ) {
    //	cl.viewangles[PITCH] = oldAngles[PITCH] - 90;
    // }
    //
    // // store out the final values
    // CL_FinishMove( &cmd );
    //
    // // draw debug graphs of turning for mouse testing
    // if ( cl_debugMove->integer ) {
    //	if ( cl_debugMove->integer == 1 ) {
    //		SCR_DebugGraph( abs(cl.viewangles[YAW] - oldAngles[YAW]), 0 );
    //	}
    //	if ( cl_debugMove->integer == 2 ) {
    //		SCR_DebugGraph( abs(cl.viewangles[PITCH] - oldAngles[PITCH]), 0 );
    //	}
    // }
    //
    // return cmd;
    std::ptr::null_mut()
}

/*
=================
CL_CreateNewCommands

Create a new usercmd_t structure for this frame
=================
*/
pub unsafe fn CL_CreateNewCommands() {
    // usercmd_t	*cmd;
    // int			cmdNum;
    //
    // // no need to create usercmds until we have a gamestate
    // if ( cls.state < CA_PRIMED ) {
    //	return;
    // }
    //
    // frame_msec = com_frameTime - old_com_frameTime;
    //
    // // if running less than 5fps, truncate the extra time to prevent
    // // unexpected moves after a hitch
    // if ( frame_msec > 200 ) {
    //	frame_msec = 200;
    // }
    // old_com_frameTime = com_frameTime;
    //
    //
    // // generate a command for this frame
    // cl.cmdNumber++;
    // cmdNum = cl.cmdNumber & CMD_MASK;
    // cl.cmds[cmdNum] = CL_CreateCmd ();
    // cmd = &cl.cmds[cmdNum];
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
    // int		oldPacketNum;
    // int		delta;
    //
    // // don't send anything if playing back a demo
    // #ifdef _XBOX	// No demos on Xbox
    //	if ( cls.state == CA_CINEMATIC ) {
    // #else
    //	if ( clc.demoplaying || cls.state == CA_CINEMATIC ) {
    // #endif
    //		return qfalse;
    //	}
    //
    // // If we are downloading, we send no less than 50ms between packets
    // #ifndef _XBOX	// No downloads on Xbox
    //	if ( *clc.downloadTempName &&
    //		cls.realtime - clc.lastPacketSentTime < 50 ) {
    //		return qfalse;
    //	}
    // #endif
    //
    // // if we don't have a valid gamestate yet, only send
    // // one packet a second
    // if ( cls.state != CA_ACTIVE &&
    //	cls.state != CA_PRIMED &&
    // #ifndef _XBOX	// No downloads on Xbox
    //	!*clc.downloadTempName &&
    // #endif
    //	cls.realtime - clc.lastPacketSentTime < 1000 ) {
    //	return qfalse;
    // }
    //
    // // send every frame for loopbacks
    // if ( clc.netchan.remoteAddress.type == NA_LOOPBACK ) {
    //	return qtrue;
    // }
    //
    // // send every frame for LAN
    // if ( Sys_IsLANAddress( clc.netchan.remoteAddress ) ) {
    //	return qtrue;
    // }
    //
    // // check for exceeding cl_maxpackets
    // if ( cl_maxpackets->integer < 15 ) {
    //	Cvar_Set( "cl_maxpackets", "15" );
    // } else if ( cl_maxpackets->integer > 100 ) {
    //	Cvar_Set( "cl_maxpackets", "100" );
    // }
    // oldPacketNum = (clc.netchan.outgoingSequence - 1) & PACKET_MASK;
    // delta = cls.realtime -  cl.outPackets[ oldPacketNum ].p_realtime;
    // if ( delta < 1000 / cl_maxpackets->integer ) {
    //	// the accumulated commands will go out in the next packet
    //	return qfalse;
    // }
    //
    // return qtrue;
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
    // msg_t		buf;
    // byte		data[MAX_MSGLEN];
    // int			i, j;
    // usercmd_t	*cmd, *oldcmd;
    // usercmd_t	nullcmd;
    // int			packetNum;
    // int			oldPacketNum;
    // int			count, key;
    //
    // // don't send anything if playing back a demo
    // #ifdef _XBOX	// No demos on Xbox
    //	if ( cls.state == CA_CINEMATIC ) {
    // #else
    //	if ( clc.demoplaying || cls.state == CA_CINEMATIC ) {
    // #endif
    //		return;
    //	}
    //
    // Com_Memset( &nullcmd, 0, sizeof(nullcmd) );
    // oldcmd = &nullcmd;
    //
    // MSG_Init( &buf, data, sizeof(data) );
    //
    // MSG_Bitstream( &buf );
    // // write the current serverId so the server
    // // can tell if this is from the current gameState
    // MSG_WriteLong( &buf, cl.serverId );
    //
    // // write the last message we received, which can
    // // be used for delta compression, and is also used
    // // to tell if we dropped a gamestate
    // MSG_WriteLong( &buf, clc.serverMessageSequence );
    //
    // // write the last reliable message we received
    // MSG_WriteLong( &buf, clc.serverCommandSequence );
    //
    // // write any unacknowledged clientCommands
    // for ( i = clc.reliableAcknowledge + 1 ; i <= clc.reliableSequence ; i++ ) {
    //	MSG_WriteByte( &buf, clc_clientCommand );
    //	MSG_WriteLong( &buf, i );
    //	MSG_WriteString( &buf, clc.reliableCommands[ i & (MAX_RELIABLE_COMMANDS-1) ] );
    // }
    //
    // // we want to send all the usercmds that were generated in the last
    // // few packet, so even if a couple packets are dropped in a row,
    // // all the cmds will make it to the server
    // if ( cl_packetdup->integer < 0 ) {
    //	Cvar_Set( "cl_packetdup", "0" );
    // } else if ( cl_packetdup->integer > 5 ) {
    //	Cvar_Set( "cl_packetdup", "5" );
    // }
    // oldPacketNum = (clc.netchan.outgoingSequence - 1 - cl_packetdup->integer) & PACKET_MASK;
    // count = cl.cmdNumber - cl.outPackets[ oldPacketNum ].p_cmdNumber;
    // if ( count > MAX_PACKET_USERCMDS ) {
    //	count = MAX_PACKET_USERCMDS;
    //	Com_Printf("MAX_PACKET_USERCMDS\n");
    // }
    // if ( count >= 1 ) {
    //	if ( cl_showSend->integer ) {
    //		Com_Printf( "(%i)", count );
    //	}
    //
    //	// begin a client move command
    //	if ( cl_nodelta->integer || !cl.snap.valid
    // #ifndef _XBOX	// No demos on Xbox
    //		|| clc.demowaiting
    // #endif
    //		|| clc.serverMessageSequence != cl.snap.messageNum ) {
    //		MSG_WriteByte (&buf, clc_moveNoDelta);
    //	} else {
    //		MSG_WriteByte (&buf, clc_move);
    //	}
    //
    //	// write the command count
    //	MSG_WriteByte( &buf, count );
    //
    //	// use the checksum feed in the key
    //	key = clc.checksumFeed;
    //	// also use the message acknowledge
    //	key ^= clc.serverMessageSequence;
    //	// also use the last acknowledged server command in the key
    //	key ^= Com_HashKey(clc.serverCommands[ clc.serverCommandSequence & (MAX_RELIABLE_COMMANDS-1) ], 32);
    //
    //	// write all the commands, including the predicted command
    //	for ( i = 0 ; i < count ; i++ ) {
    //		j = (cl.cmdNumber - count + i + 1) & CMD_MASK;
    //		cmd = &cl.cmds[j];
    //		MSG_WriteDeltaUsercmdKey (&buf, key, oldcmd, cmd);
    //		oldcmd = cmd;
    //	}
    //
    //	if (cl.gcmdSentValue)
    //	{ //hmm, just clear here, I guess.. hoping it will resolve issues with gencmd values sometimes not going through.
    //		cl.gcmdSendValue = qfalse;
    //		cl.gcmdSentValue = qfalse;
    //		cl.gcmdValue = 0;
    //	}
    // }
    //
    // //
    // // deliver the message
    // //
    // packetNum = clc.netchan.outgoingSequence & PACKET_MASK;
    // cl.outPackets[ packetNum ].p_realtime = cls.realtime;
    // cl.outPackets[ packetNum ].p_serverTime = oldcmd->serverTime;
    // cl.outPackets[ packetNum ].p_cmdNumber = cl.cmdNumber;
    // clc.lastPacketSentTime = cls.realtime;
    //
    // if ( cl_showSend->integer ) {
    //	Com_Printf( "%i ", buf.cursize );
    // }
    //
    // CL_Netchan_Transmit (&clc.netchan, &buf);
    //
    // // clients never really should have messages large enough
    // // to fragment, but in case they do, fire them all off
    // // at once
    // while ( clc.netchan.unsentFragments ) {
    //	CL_Netchan_TransmitNextFragment( &clc.netchan );
    // }
}

/*
=================
CL_SendCmd

Called every frame to builds and sends a command packet to the server.
=================
*/
pub unsafe fn CL_SendCmd() {
    // // don't send any message if not connected
    // if ( cls.state < CA_CONNECTED ) {
    //	return;
    // }
    //
    // // don't send commands if paused
    // if ( com_sv_running->integer && sv_paused->integer && cl_paused->integer ) {
    //	return;
    // }
    //
    // // we create commands even if a demo is playing,
    // CL_CreateNewCommands();
    //
    // // don't send a packet if the last packet was sent too recently
    // if ( !CL_ReadyToSendPacket() ) {
    //	if ( cl_showSend->integer ) {
    //		Com_Printf( ". " );
    //	}
    //	return;
    // }
    //
    // CL_WritePacket();
}

/*
============
CL_InitInput
============
*/
pub unsafe fn CL_InitInput() {
    Cmd_AddCommand(b"centerview\0".as_ptr() as *const c_char, IN_CenterViewLocal);

    //Cmd_AddCommand ("+taunt", IN_Button3Down);//gesture
    //Cmd_AddCommand ("-taunt", IN_Button3Up);
    Cmd_AddCommand(b"+moveup\0".as_ptr() as *const c_char, IN_UpDown);
    Cmd_AddCommand(b"-moveup\0".as_ptr() as *const c_char, IN_UpUp);
    Cmd_AddCommand(b"+movedown\0".as_ptr() as *const c_char, IN_DownDown);
    Cmd_AddCommand(b"-movedown\0".as_ptr() as *const c_char, IN_DownUp);
    Cmd_AddCommand(b"+left\0".as_ptr() as *const c_char, IN_LeftDown);
    Cmd_AddCommand(b"-left\0".as_ptr() as *const c_char, IN_LeftUp);
    Cmd_AddCommand(b"+right\0".as_ptr() as *const c_char, IN_RightDown);
    Cmd_AddCommand(b"-right\0".as_ptr() as *const c_char, IN_RightUp);
    Cmd_AddCommand(b"+forward\0".as_ptr() as *const c_char, IN_ForwardDown);
    Cmd_AddCommand(b"-forward\0".as_ptr() as *const c_char, IN_ForwardUp);
    Cmd_AddCommand(b"+back\0".as_ptr() as *const c_char, IN_BackDown);
    Cmd_AddCommand(b"-back\0".as_ptr() as *const c_char, IN_BackUp);
    Cmd_AddCommand(b"+lookup\0".as_ptr() as *const c_char, IN_LookupDown);
    Cmd_AddCommand(b"-lookup\0".as_ptr() as *const c_char, IN_LookupUp);
    Cmd_AddCommand(b"+lookdown\0".as_ptr() as *const c_char, IN_LookdownDown);
    Cmd_AddCommand(b"-lookdown\0".as_ptr() as *const c_char, IN_LookdownUp);
    Cmd_AddCommand(b"+strafe\0".as_ptr() as *const c_char, IN_StrafeDown);
    Cmd_AddCommand(b"-strafe\0".as_ptr() as *const c_char, IN_StrafeUp);
    Cmd_AddCommand(b"+moveleft\0".as_ptr() as *const c_char, IN_MoveleftDown);
    Cmd_AddCommand(b"-moveleft\0".as_ptr() as *const c_char, IN_MoveleftUp);
    Cmd_AddCommand(b"+moveright\0".as_ptr() as *const c_char, IN_MoverightDown);
    Cmd_AddCommand(b"-moveright\0".as_ptr() as *const c_char, IN_MoverightUp);
    Cmd_AddCommand(b"+speed\0".as_ptr() as *const c_char, IN_SpeedDown);
    Cmd_AddCommand(b"-speed\0".as_ptr() as *const c_char, IN_SpeedUp);
    Cmd_AddCommand(b"+attack\0".as_ptr() as *const c_char, IN_Button0Down);
    Cmd_AddCommand(b"-attack\0".as_ptr() as *const c_char, IN_Button0Up);
    //Cmd_AddCommand ("+force_jump", IN_Button1Down);//force jump
    //Cmd_AddCommand ("-force_jump", IN_Button1Up);
    Cmd_AddCommand(b"+use\0".as_ptr() as *const c_char, IN_Button5Down);
    Cmd_AddCommand(b"-use\0".as_ptr() as *const c_char, IN_Button5Up);
    Cmd_AddCommand(b"+force_grip\0".as_ptr() as *const c_char, IN_Button6Down); //force grip
    Cmd_AddCommand(b"-force_grip\0".as_ptr() as *const c_char, IN_Button6Up);
    Cmd_AddCommand(b"+altattack\0".as_ptr() as *const c_char, IN_Button7Down); //altattack
    Cmd_AddCommand(b"-altattack\0".as_ptr() as *const c_char, IN_Button7Up);
    Cmd_AddCommand(b"+useforce\0".as_ptr() as *const c_char, IN_Button9Down); //active force power
    Cmd_AddCommand(b"-useforce\0".as_ptr() as *const c_char, IN_Button9Up);
    Cmd_AddCommand(
        b"+force_lightning\0".as_ptr() as *const c_char,
        IN_Button10Down,
    ); //active force power
    Cmd_AddCommand(
        b"-force_lightning\0".as_ptr() as *const c_char,
        IN_Button10Up,
    );
    Cmd_AddCommand(b"+force_drain\0".as_ptr() as *const c_char, IN_Button11Down); //active force power
    Cmd_AddCommand(b"-force_drain\0".as_ptr() as *const c_char, IN_Button11Up);
    //buttons
    Cmd_AddCommand(b"+button0\0".as_ptr() as *const c_char, IN_Button0Down); //attack
    Cmd_AddCommand(b"-button0\0".as_ptr() as *const c_char, IN_Button0Up);
    Cmd_AddCommand(b"+button1\0".as_ptr() as *const c_char, IN_Button1Down); //force jump
    Cmd_AddCommand(b"-button1\0".as_ptr() as *const c_char, IN_Button1Up);
    Cmd_AddCommand(b"+button2\0".as_ptr() as *const c_char, IN_Button2Down); //use holdable (not used - change to use jedi power?)
    Cmd_AddCommand(b"-button2\0".as_ptr() as *const c_char, IN_Button2Up);
    Cmd_AddCommand(b"+button3\0".as_ptr() as *const c_char, IN_Button3Down); //gesture
    Cmd_AddCommand(b"-button3\0".as_ptr() as *const c_char, IN_Button3Up);
    Cmd_AddCommand(b"+button4\0".as_ptr() as *const c_char, IN_Button4Down); //walking
    Cmd_AddCommand(b"-button4\0".as_ptr() as *const c_char, IN_Button4Up);
    Cmd_AddCommand(b"+button5\0".as_ptr() as *const c_char, IN_Button5Down); //use object
    Cmd_AddCommand(b"-button5\0".as_ptr() as *const c_char, IN_Button5Up);
    Cmd_AddCommand(b"+button6\0".as_ptr() as *const c_char, IN_Button6Down); //force grip
    Cmd_AddCommand(b"-button6\0".as_ptr() as *const c_char, IN_Button6Up);
    Cmd_AddCommand(b"+button7\0".as_ptr() as *const c_char, IN_Button7Down); //altattack
    Cmd_AddCommand(b"-button7\0".as_ptr() as *const c_char, IN_Button7Up);
    Cmd_AddCommand(b"+button8\0".as_ptr() as *const c_char, IN_Button8Down);
    Cmd_AddCommand(b"-button8\0".as_ptr() as *const c_char, IN_Button8Up);
    Cmd_AddCommand(b"+button9\0".as_ptr() as *const c_char, IN_Button9Down); //active force power
    Cmd_AddCommand(b"-button9\0".as_ptr() as *const c_char, IN_Button9Up);
    Cmd_AddCommand(b"+button10\0".as_ptr() as *const c_char, IN_Button10Down); //force lightning
    Cmd_AddCommand(b"-button10\0".as_ptr() as *const c_char, IN_Button10Up);
    Cmd_AddCommand(b"+button11\0".as_ptr() as *const c_char, IN_Button11Down); //force drain
    Cmd_AddCommand(b"-button11\0".as_ptr() as *const c_char, IN_Button11Up);
    Cmd_AddCommand(b"+button12\0".as_ptr() as *const c_char, IN_Button12Down);
    Cmd_AddCommand(b"-button12\0".as_ptr() as *const c_char, IN_Button12Up);
    Cmd_AddCommand(b"+button13\0".as_ptr() as *const c_char, IN_Button13Down);
    Cmd_AddCommand(b"-button13\0".as_ptr() as *const c_char, IN_Button13Up);
    Cmd_AddCommand(b"+button14\0".as_ptr() as *const c_char, IN_Button14Down);
    Cmd_AddCommand(b"-button14\0".as_ptr() as *const c_char, IN_Button14Up);
    Cmd_AddCommand(b"+mlook\0".as_ptr() as *const c_char, IN_MLookDown);
    Cmd_AddCommand(b"-mlook\0".as_ptr() as *const c_char, IN_MLookUp);

    Cmd_AddCommand(b"sv_saberswitch\0".as_ptr() as *const c_char, IN_GenCMD1);
    Cmd_AddCommand(b"engage_duel\0".as_ptr() as *const c_char, IN_GenCMD2);
    Cmd_AddCommand(b"force_heal\0".as_ptr() as *const c_char, IN_GenCMD3);
    Cmd_AddCommand(b"force_speed\0".as_ptr() as *const c_char, IN_GenCMD4);
    Cmd_AddCommand(b"force_pull\0".as_ptr() as *const c_char, IN_GenCMD5);
    Cmd_AddCommand(b"force_distract\0".as_ptr() as *const c_char, IN_GenCMD6);
    Cmd_AddCommand(b"force_rage\0".as_ptr() as *const c_char, IN_GenCMD7);
    Cmd_AddCommand(b"force_protect\0".as_ptr() as *const c_char, IN_GenCMD8);
    Cmd_AddCommand(b"force_absorb\0".as_ptr() as *const c_char, IN_GenCMD9);
    Cmd_AddCommand(b"force_healother\0".as_ptr() as *const c_char, IN_GenCMD10);
    Cmd_AddCommand(
        b"force_forcepowerother\0".as_ptr() as *const c_char,
        IN_GenCMD11,
    );
    Cmd_AddCommand(b"force_seeing\0".as_ptr() as *const c_char, IN_GenCMD12);
    Cmd_AddCommand(b"use_seeker\0".as_ptr() as *const c_char, IN_GenCMD13);
    Cmd_AddCommand(b"use_field\0".as_ptr() as *const c_char, IN_GenCMD14);
    Cmd_AddCommand(b"use_bacta\0".as_ptr() as *const c_char, IN_GenCMD15);
    Cmd_AddCommand(
        b"use_electrobinoculars\0".as_ptr() as *const c_char,
        IN_GenCMD16,
    );
    Cmd_AddCommand(b"zoom\0".as_ptr() as *const c_char, IN_GenCMD17);
    Cmd_AddCommand(b"use_sentry\0".as_ptr() as *const c_char, IN_GenCMD18);
    Cmd_AddCommand(b"use_jetpack\0".as_ptr() as *const c_char, IN_GenCMD21);
    Cmd_AddCommand(b"use_bactabig\0".as_ptr() as *const c_char, IN_GenCMD22);
    Cmd_AddCommand(b"use_healthdisp\0".as_ptr() as *const c_char, IN_GenCMD23);
    Cmd_AddCommand(b"use_ammodisp\0".as_ptr() as *const c_char, IN_GenCMD24);
    Cmd_AddCommand(b"use_eweb\0".as_ptr() as *const c_char, IN_GenCMD25);
    Cmd_AddCommand(b"use_cloak\0".as_ptr() as *const c_char, IN_GenCMD26);
    Cmd_AddCommand(b"taunt\0".as_ptr() as *const c_char, IN_GenCMD27);
    Cmd_AddCommand(b"bow\0".as_ptr() as *const c_char, IN_GenCMD28);
    Cmd_AddCommand(b"meditate\0".as_ptr() as *const c_char, IN_GenCMD29);
    Cmd_AddCommand(b"flourish\0".as_ptr() as *const c_char, IN_GenCMD30);
    Cmd_AddCommand(b"gloat\0".as_ptr() as *const c_char, IN_GenCMD31);
    Cmd_AddCommand(b"saberAttackCycle\0".as_ptr() as *const c_char, IN_GenCMD19);
    Cmd_AddCommand(b"force_throw\0".as_ptr() as *const c_char, IN_GenCMD20);
    // #ifdef _XBOX
    //	Cmd_AddCommand ("+hotswap1", IN_HotSwap1On);
    //	Cmd_AddCommand ("+hotswap2", IN_HotSwap2On);
    //	Cmd_AddCommand ("-hotswap1", IN_HotSwap1Off);
    //	Cmd_AddCommand ("-hotswap2", IN_HotSwap2Off);
    //
    //	Cmd_AddCommand ("+voicetoggle", IN_VoiceToggleDown);
    //	Cmd_AddCommand ("-voicetoggle", IN_VoiceToggleUp);
    // #endif
    Cmd_AddCommand(b"useGivenForce\0".as_ptr() as *const c_char, IN_UseGivenForce);

    Cmd_AddCommand(b"automap_button\0".as_ptr() as *const c_char, IN_AutoMapButton);
    Cmd_AddCommand(b"automap_toggle\0".as_ptr() as *const c_char, IN_AutoMapToggle);
    Cmd_AddCommand(b"voicechat\0".as_ptr() as *const c_char, IN_VoiceChatButton);

    // cl_nodelta = Cvar_Get ("cl_nodelta", "0", 0);
    // cl_debugMove = Cvar_Get ("cl_debugMove", "0", 0);
}
