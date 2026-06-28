//! Mechanical port of `codemp/qcommon/cmd_console.cpp`.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};
use crate::ffi::types::qboolean;
use crate::codemp::qcommon::files_h::cvar_t;
use std::ptr::{addr_of, addr_of_mut};

const CMD_MAX_NUM: usize = 512;
const CMD_MAX_NAME: usize = 32;

// PORT: Type alias for command function pointer: void (*xcommand_t)(void)
pub type xcommand_t = extern "C" fn();

#[repr(C)]
pub struct cmd_function_s {
    name: [c_char; CMD_MAX_NAME],
    function: Option<xcommand_t>,
}

type cmd_function_t = cmd_function_s;

static mut cmd_functions: [cmd_function_t; CMD_MAX_NUM] = unsafe {
    // Initialize with zero-filled array (matching C BSS semantics)
    [cmd_function_s {
        name: [0; CMD_MAX_NAME],
        function: None,
    }; CMD_MAX_NUM]
};

extern "C" {
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Cmd_TokenizeString(text: *const c_char);
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_Argv(arg: c_int) -> *const c_char;
    pub fn Cvar_Command() -> qboolean;
    pub fn CL_GameCommand() -> qboolean;
    pub fn SV_GameCommand() -> qboolean;
    pub fn UI_GameCommand() -> qboolean;
    pub fn CL_ForwardCommandToServer(text: *const c_char);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Filter(filter: *const c_char, name: *const c_char, casesensitive: qboolean) -> qboolean;

    // Global cvar pointers
    pub static mut com_cl_running: *mut cvar_t;
    pub static mut com_sv_running: *mut cvar_t;
}

/*
============
Cmd_AddCommand
============
*/
pub extern "C" fn Cmd_AddCommand(cmd_name: *const c_char, function: Option<xcommand_t>) {
    let mut cmd: *mut cmd_function_t;
    let mut add: *mut cmd_function_t = core::ptr::null_mut();
    let mut i: usize;

    // fail if the command already exists
    i = 0;
    while i < CMD_MAX_NUM {
        cmd = unsafe { addr_of_mut!(cmd_functions[i]) };
        if unsafe { strcmp(cmd_name, (*cmd).name.as_ptr()) == 0 } {
            // allow completion-only commands to be silently doubled
            if function.is_some() {
                unsafe {
                    Com_Printf(
                        b"Cmd_AddCommand: %s already defined\n\0".as_ptr() as *const c_char,
                        cmd_name,
                    );
                }
            }
            return;
        }

        if add.is_null() && unsafe { (*cmd).name[0] == 0 } {
            add = cmd;
        }

        i += 1;
    }

    if add.is_null() {
        unsafe {
            Com_Printf(b"Cmd_AddCommand: Too many commands registered\n\0".as_ptr() as *const c_char);
        }
        return;
    }

    if unsafe { strlen(cmd_name) >= (CMD_MAX_NAME - 1) as usize } {
        unsafe {
            Com_Printf(b"Cmd_AddCommand: Excessively long command name\n\0".as_ptr() as *const c_char);
        }
    } else {
        unsafe {
            Q_strncpyz((*add).name.as_mut_ptr(), cmd_name, CMD_MAX_NAME as c_int);
            (*add).function = function;
        }
    }
}

/*
============
Cmd_RemoveCommand
============
*/
pub extern "C" fn Cmd_RemoveCommand(cmd_name: *const c_char) {
    let mut cmd: *mut cmd_function_t;
    let mut i: usize;

    i = 0;
    while i < CMD_MAX_NUM {
        cmd = unsafe { addr_of_mut!(cmd_functions[i]) };
        if unsafe { strcmp(cmd_name, (*cmd).name.as_ptr()) == 0 } {
            unsafe { (*cmd).name[0] = 0 };
            return;
        }

        i += 1;
    }
}


/*
============
Cmd_ExecuteString

A complete command line has been parsed, so try to execute it
============
*/
pub extern "C" fn Cmd_ExecuteString(text: *const c_char) {
    let mut i: c_int;

    // execute the command line
    unsafe { Cmd_TokenizeString(text) };
    if unsafe { Cmd_Argc() } == 0 {
        return;		// no tokens
    }

    // check registered command functions
    i = 0;
    while i < CMD_MAX_NUM as c_int {
        if unsafe { Q_stricmp(Cmd_Argv(0), cmd_functions[i as usize].name.as_ptr()) == 0 } {
            // rearrange the links so that the command will be
            // near the head of the list next time it is used
            let temp = unsafe { cmd_functions[i as usize] };
            unsafe {
                cmd_functions[i as usize] = cmd_functions[0];
                cmd_functions[0] = temp;
            }

            // perform the action
            unsafe {
                if let Some(func) = temp.function {
                    func();
                }
                // let the cgame or game handle it
            }
            return;
        }

        i += 1;
    }

    // check cvars
    if unsafe { Cvar_Command() != 0 } {
        return;
    }

    // check client game commands
    if unsafe {
        !com_cl_running.is_null()
            && (*com_cl_running).integer != 0
            && CL_GameCommand() != 0
    } {
        return;
    }

    // check server game commands
    if unsafe {
        !com_sv_running.is_null()
            && (*com_sv_running).integer != 0
            && SV_GameCommand() != 0
    } {
        return;
    }

    // check ui commands
    if unsafe {
        !com_cl_running.is_null()
            && (*com_cl_running).integer != 0
            && UI_GameCommand() != 0
    } {
        return;
    }

    // send it as a server command if we are connected
    // this will usually result in a chat message
    //CL_ForwardCommandToServer ( text );
    unsafe { CL_ForwardCommandToServer(text) };
}


/*
============
Cmd_List_f
============
*/
pub extern "C" fn Cmd_List_f() {
    let mut cmd: *mut cmd_function_t;
    let mut i: c_int;
    let match_str: *const c_char;

    if unsafe { Cmd_Argc() } > 1 {
        match_str = unsafe { Cmd_Argv(1) };
    } else {
        match_str = core::ptr::null();
    }

    i = 0;
    let mut c: c_int = 0;
    while c < CMD_MAX_NUM as c_int {
        cmd = unsafe { addr_of_mut!(cmd_functions[c as usize]) };
        if !match_str.is_null()
            && unsafe { Com_Filter(match_str, (*cmd).name.as_ptr(), 0) == 0 }
        {
            c += 1;
            continue;
        }

        unsafe { Com_Printf(b"%s\n\0".as_ptr() as *const c_char, (*cmd).name.as_ptr()) };
        i += 1;

        c += 1;
    }
    unsafe { Com_Printf(b"%i commands\n\0".as_ptr() as *const c_char, i) };
}
