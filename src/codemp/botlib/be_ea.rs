/*****************************************************************************
 * name:		be_ea.c
 *
 * desc:		elementary actions
 *
 * $Archive: /MissionPack/code/botlib/be_ea.c $
 * $Author: Zaphod $
 * $Revision: 5 $
 * $Modtime: 11/22/00 8:50a $
 * $Date: 11/22/00 8:55a $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};

const MAX_USERMOVE: c_int = 400;
const MAX_COMMANDARGUMENTS: c_int = 10;
const ACTION_JUMPEDLASTFRAME: c_int = 0x0800000; // 128

// External types used by this module
// Minimal structure declaration for bot_input_t based on fields accessed in be_ea.c
// Full definition in botlib.h with potentially additional fields
#[repr(C)]
pub struct bot_input_t {
    pub actionflags: c_int,
    pub weapon: c_int,
    pub dir: [f32; 3],
    pub speed: f32,
    pub viewangles: [f32; 3],
    pub thinktime: f32,
}

// External structures and globals
// These are defined in other modules and linked at runtime
#[repr(C)]
pub struct botimport_t {
    pub BotClientCommand: Option<unsafe extern "C" fn(c_int, *const c_char)>,
    pub BotInput: Option<unsafe extern "C" fn(c_int, *mut bot_input_t)>,
    // Additional fields omitted - only those needed for this file
}

#[repr(C)]
pub struct botlibglobals_t {
    pub maxclients: c_int,
    // Additional fields omitted
}

extern "C" {
    pub static botimport: botimport_t;
    pub static botlibglobals: botlibglobals_t;

    // External functions from other modules
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn GetClearedHunkMemory(size: usize) -> *mut c_void;
    fn FreeMemory(ptr: *mut c_void);
    fn Com_Memcpy(dst: *mut c_void, src: *const c_void, size: usize) -> *mut c_void;

    // Vector operations - assumed to be defined in shared math module
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorClear(v: *mut f32);
}

// Global bot input array
static mut botinputs: *mut bot_input_t = 0 as *mut bot_input_t;

// Constants imported from q_shared.h / botlib.h
// These would typically come from included headers
const ACTION_GESTURE: c_int = 0;		// Placeholder - actual values in q_shared.h
const ACTION_ATTACK: c_int = 0;			// Placeholder
const ACTION_ALT_ATTACK: c_int = 0;		// Placeholder
const ACTION_FORCEPOWER: c_int = 0;		// Placeholder
const ACTION_TALK: c_int = 0;			// Placeholder
const ACTION_USE: c_int = 0;			// Placeholder
const ACTION_RESPAWN: c_int = 0;		// Placeholder
const ACTION_JUMP: c_int = 0;			// Placeholder
const ACTION_DELAYEDJUMP: c_int = 0;	// Placeholder
const ACTION_CROUCH: c_int = 0;			// Placeholder
const ACTION_WALK: c_int = 0;			// Placeholder
const ACTION_MOVEUP: c_int = 0;			// Placeholder
const ACTION_MOVEDOWN: c_int = 0;		// Placeholder
const ACTION_MOVEFORWARD: c_int = 0;	// Placeholder
const ACTION_MOVEBACK: c_int = 0;		// Placeholder
const ACTION_MOVELEFT: c_int = 0;		// Placeholder
const ACTION_MOVERIGHT: c_int = 0;		// Placeholder

const qfalse: bool = false;

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn EA_Say(client: c_int, str: *mut c_char) {
    unsafe {
        if let Some(cmd_fn) = botimport.BotClientCommand {
            let fmt = b"say %s\0".as_ptr() as *const c_char;
            let cmd = va(fmt, str);
            cmd_fn(client, cmd);
        }
    }
} //end of the function EA_Say
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn EA_SayTeam(client: c_int, str: *mut c_char) {
    unsafe {
        if let Some(cmd_fn) = botimport.BotClientCommand {
            let fmt = b"say_team %s\0".as_ptr() as *const c_char;
            let cmd = va(fmt, str);
            cmd_fn(client, cmd);
        }
    }
} //end of the function EA_SayTeam
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn EA_Tell(client: c_int, clientto: c_int, str: *mut c_char) {
    unsafe {
        if let Some(cmd_fn) = botimport.BotClientCommand {
            let fmt = b"tell %d, %s\0".as_ptr() as *const c_char;
            let cmd = va(fmt, clientto, str);
            cmd_fn(client, cmd);
        }
    }
} //end of the function EA_SayTeam
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn EA_UseItem(client: c_int, it: *mut c_char) {
    unsafe {
        if let Some(cmd_fn) = botimport.BotClientCommand {
            let fmt = b"use %s\0".as_ptr() as *const c_char;
            let cmd = va(fmt, it);
            cmd_fn(client, cmd);
        }
    }
} //end of the function EA_UseItem
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn EA_DropItem(client: c_int, it: *mut c_char) {
    unsafe {
        if let Some(cmd_fn) = botimport.BotClientCommand {
            let fmt = b"drop %s\0".as_ptr() as *const c_char;
            let cmd = va(fmt, it);
            cmd_fn(client, cmd);
        }
    }
} //end of the function EA_DropItem
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn EA_UseInv(client: c_int, inv: *mut c_char) {
    unsafe {
        if let Some(cmd_fn) = botimport.BotClientCommand {
            let fmt = b"invuse %s\0".as_ptr() as *const c_char;
            let cmd = va(fmt, inv);
            cmd_fn(client, cmd);
        }
    }
} //end of the function EA_UseInv
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_DropInv(client: c_int, inv: *mut c_char) {
    unsafe {
        if let Some(cmd_fn) = botimport.BotClientCommand {
            let fmt = b"invdrop %s\0".as_ptr() as *const c_char;
            let cmd = va(fmt, inv);
            cmd_fn(client, cmd);
        }
    }
} //end of the function EA_DropInv
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Gesture(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_GESTURE;
    }
} //end of the function EA_Gesture
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn EA_Command(client: c_int, command: *mut c_char) {
    unsafe {
        if let Some(cmd_fn) = botimport.BotClientCommand {
            cmd_fn(client, command);
        }
    }
} //end of the function EA_Command
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_SelectWeapon(client: c_int, weapon: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).weapon = weapon;
    }
} //end of the function EA_SelectWeapon
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Attack(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_ATTACK;
    }
} //end of the function EA_Attack
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Alt_Attack(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_ALT_ATTACK;
    }
} //end of the function EA_Alt_Attack
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_ForcePower(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_FORCEPOWER;
    }
} //end of the function EA_ForcePower
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Talk(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_TALK;
    }
} //end of the function EA_Talk
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Use(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_USE;
    }
} //end of the function EA_Use
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Respawn(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_RESPAWN;
    }
} //end of the function EA_Respawn
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Jump(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        if (*bi).actionflags & ACTION_JUMPEDLASTFRAME != 0 {
            (*bi).actionflags &= !ACTION_JUMP;
        } //end if
        else {
            (*bi).actionflags |= ACTION_JUMP;
        } //end if
    }
} //end of the function EA_Jump
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_DelayedJump(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        if (*bi).actionflags & ACTION_JUMPEDLASTFRAME != 0 {
            (*bi).actionflags &= !ACTION_DELAYEDJUMP;
        } //end if
        else {
            (*bi).actionflags |= ACTION_DELAYEDJUMP;
        } //end if
    }
} //end of the function EA_DelayedJump
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Crouch(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_CROUCH;
    }
} //end of the function EA_Crouch
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Walk(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_WALK;
    }
} //end of the function EA_Walk
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Action(client: c_int, action: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= action;
    }
} //end of function EA_Action
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_MoveUp(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_MOVEUP;
    }
} //end of the function EA_MoveUp
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_MoveDown(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_MOVEDOWN;
    }
} //end of the function EA_MoveDown
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_MoveForward(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_MOVEFORWARD;
    }
} //end of the function EA_MoveForward
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_MoveBack(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_MOVEBACK;
    }
} //end of the function EA_MoveBack
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_MoveLeft(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_MOVELEFT;
    }
} //end of the function EA_MoveLeft
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_MoveRight(client: c_int) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags |= ACTION_MOVERIGHT;
    }
} //end of the function EA_MoveRight
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Move(client: c_int, dir: *const f32, speed: f32) {
    let bi: *mut bot_input_t;
    let mut speed = speed;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        VectorCopy(dir, &mut (*bi).dir[0] as *mut f32);
        // cap speed
        if speed > MAX_USERMOVE as f32 {
            speed = MAX_USERMOVE as f32;
        } else if speed < -(MAX_USERMOVE as f32) {
            speed = -(MAX_USERMOVE as f32);
        }
        (*bi).speed = speed;
    }
} //end of the function EA_Move
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_View(client: c_int, viewangles: *const f32) {
    let bi: *mut bot_input_t;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        VectorCopy(viewangles, &mut (*bi).viewangles[0] as *mut f32);
    }
} //end of the function EA_View
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_EndRegular(client: c_int, thinktime: f32) {
    /*
    let bi: *mut bot_input_t;
    let mut jumped: c_int = qfalse as c_int;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags &= !ACTION_JUMPEDLASTFRAME;

        (*bi).thinktime = thinktime;
        if let Some(input_fn) = botimport.BotInput {
            input_fn(client, bi);
        }

        (*bi).thinktime = 0.0;
        VectorClear(&mut (*bi).dir[0] as *mut f32);
        (*bi).speed = 0.0;
        jumped = ((*bi).actionflags & ACTION_JUMP) as c_int;
        (*bi).actionflags = 0;
        if jumped != 0 {
            (*bi).actionflags |= ACTION_JUMPEDLASTFRAME;
        }
    }
    */
} //end of the function EA_EndRegular
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_GetInput(client: c_int, thinktime: f32, input: *mut bot_input_t) {
    let bi: *mut bot_input_t;
    // let mut jumped: bool = qfalse;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    // (*bi).actionflags &= !ACTION_JUMPEDLASTFRAME;

    unsafe {
        (*bi).thinktime = thinktime;
        Com_Memcpy(input as *mut c_void, bi as *const c_void, core::mem::size_of::<bot_input_t>());

        /*
        (*bi).thinktime = 0.0;
        VectorClear(&mut (*bi).dir[0] as *mut f32);
        (*bi).speed = 0.0;
        jumped = ((*bi).actionflags & ACTION_JUMP) != 0;
        (*bi).actionflags = 0;
        if jumped {
            (*bi).actionflags |= ACTION_JUMPEDLASTFRAME;
        }
        */
    }
} //end of the function EA_GetInput
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_ResetInput(client: c_int) {
    let bi: *mut bot_input_t;
    let mut jumped: c_int = qfalse as c_int;

    bi = unsafe { addr_of_mut!(botinputs).read().add(client as usize) };

    unsafe {
        (*bi).actionflags &= !ACTION_JUMPEDLASTFRAME;

        (*bi).thinktime = 0.0;
        VectorClear(&mut (*bi).dir[0] as *mut f32);
        (*bi).speed = 0.0;
        jumped = if ((*bi).actionflags & ACTION_JUMP) != 0 { 1 } else { 0 };
        (*bi).actionflags = 0;
        if jumped != 0 {
            (*bi).actionflags |= ACTION_JUMPEDLASTFRAME;
        }
    }
} //end of the function EA_ResetInput
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Setup() -> c_int {
    // initialize the bot inputs
    unsafe {
        let ptr = addr_of_mut!(botinputs);
        *ptr = GetClearedHunkMemory(botlibglobals.maxclients as usize * core::mem::size_of::<bot_input_t>()) as *mut bot_input_t;
    }
    return 0; // BLERR_NOERROR
} //end of the function EA_Setup
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub fn EA_Shutdown() {
    unsafe {
        FreeMemory(*addr_of_mut!(botinputs) as *mut c_void);
        *addr_of_mut!(botinputs) = null_mut();
    }
} //end of the function EA_Shutdown
