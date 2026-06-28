//! Mechanical port of `codemp/qcommon/cmd_pc.cpp`.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};
use crate::ffi::types::qboolean;
use crate::codemp::qcommon::files_h::cvar_t;
use std::ptr::{addr_of, addr_of_mut};

// PORT: Type alias for command function pointer: void (*xcommand_t)(void)
pub type xcommand_t = extern "C" fn();

#[repr(C)]
struct cmd_function_s {
    next: *mut cmd_function_s,
    name: *mut c_char,
    function: Option<xcommand_t>,
}

type cmd_function_t = cmd_function_s;

static mut cmd_functions: *mut cmd_function_t = core::ptr::null_mut();

extern "C" {
    pub fn S_Malloc(size: usize) -> *mut c_void;
    pub fn CopyString(str: *const c_char) -> *mut c_char;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Cmd_TokenizeString(text: *const c_char);
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_Argv(arg: c_int) -> *const c_char;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Cvar_Command() -> qboolean;
    pub fn CL_GameCommand() -> qboolean;
    pub fn SV_GameCommand() -> qboolean;
    pub fn UI_GameCommand() -> qboolean;
    pub fn CL_ForwardCommandToServer(text: *const c_char);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Filter(filter: *const c_char, name: *const c_char, casesensitive: qboolean) -> qboolean;

    // Global cvar pointers for com_cl_running and com_sv_running
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

    // fail if the command already exists
    cmd = unsafe { cmd_functions };
    while !cmd.is_null() {
        unsafe {
            if Q_stricmp(cmd_name, (*cmd).name) == 0 {
                // allow completion-only commands to be silently doubled
                if function.is_some() {
                    Com_Printf(
                        b"Cmd_AddCommand: %s already defined\n\0".as_ptr() as *const c_char,
                        cmd_name,
                    );
                }
                return;
            }
            cmd = (*cmd).next;
        }
    }

    // use a small malloc to avoid zone fragmentation
    unsafe {
        cmd = S_Malloc(core::mem::size_of::<cmd_function_t>()) as *mut cmd_function_t;
        (*cmd).name = CopyString(cmd_name);
        (*cmd).function = function;
        (*cmd).next = cmd_functions;
        cmd_functions = cmd;
    }
}

/*
============
Cmd_RemoveCommand
============
*/
pub extern "C" fn Cmd_RemoveCommand(cmd_name: *const c_char) {
    let mut cmd: *mut cmd_function_t;
    let mut back: *mut *mut cmd_function_t;

    unsafe {
        back = addr_of_mut!(cmd_functions);
        loop {
            cmd = *back;
            if cmd.is_null() {
                // command wasn't active
                return;
            }
            if Q_stricmp(cmd_name, (*cmd).name) == 0 {
                *back = (*cmd).next;
                if !(*cmd).name.is_null() {
                    Z_Free((*cmd).name as *mut c_void);
                }
                Z_Free(cmd as *mut c_void);
                return;
            }
            back = addr_of_mut!((*cmd).next);
        }
    }
}

/*
============
Cmd_CommandCompletion
============
*/
pub extern "C" fn Cmd_CommandCompletion(callback: extern "C" fn(*const c_char)) {
    let mut cmd: *mut cmd_function_t;

    unsafe {
        cmd = cmd_functions;
        while !cmd.is_null() {
            callback((*cmd).name);
            cmd = (*cmd).next;
        }
    }
}

/*
============
Cmd_ExecuteString

A complete command line has been parsed, so try to execute it
============
*/
pub extern "C" fn Cmd_ExecuteString(text: *const c_char) {
    let mut cmd: *mut cmd_function_t;
    let mut prev: *mut *mut cmd_function_t;

    unsafe {
        // execute the command line
        Cmd_TokenizeString(text);
        if Cmd_Argc() == 0 {
            return; // no tokens
        }

        // check registered command functions
        prev = addr_of_mut!(cmd_functions);
        while !(*prev).is_null() {
            cmd = *prev;
            if Q_stricmp(Cmd_Argv(0), (*cmd).name) == 0 {
                // rearrange the links so that the command will be
                // near the head of the list next time it is used
                *prev = (*cmd).next;
                (*cmd).next = cmd_functions;
                cmd_functions = cmd;

                // perform the action
                if let Some(func) = (*cmd).function {
                    func();
                } else {
                    // let the cgame or game handle it
                    break;
                }
                return;
            }
            prev = addr_of_mut!((*cmd).next);
        }

        // check cvars
        if Cvar_Command() != 0 {
            return;
        }

        // check client game commands
        if !com_cl_running.is_null() && (*com_cl_running).integer != 0 && CL_GameCommand() != 0 {
            return;
        }

        // check server game commands
        if !com_sv_running.is_null() && (*com_sv_running).integer != 0 && SV_GameCommand() != 0 {
            return;
        }

        // check ui commands
        if !com_cl_running.is_null() && (*com_cl_running).integer != 0 && UI_GameCommand() != 0 {
            return;
        }

        // send it as a server command if we are connected
        // this will usually result in a chat message
        //CL_ForwardCommandToServer ( text );
        CL_ForwardCommandToServer(text);
    }
}

/*
============
Cmd_List_f
============
*/
pub extern "C" fn Cmd_List_f() {
    let mut cmd: *mut cmd_function_t;
    let mut i: c_int;
    let mut match_: *const c_char;

    unsafe {
        if Cmd_Argc() > 1 {
            match_ = Cmd_Argv(1);
        } else {
            match_ = core::ptr::null();
        }

        i = 0;
        cmd = cmd_functions;
        while !cmd.is_null() {
            if !match_.is_null() && Com_Filter(match_, (*cmd).name, 0) == 0 {
                cmd = (*cmd).next;
                continue;
            }

            Com_Printf(b"%s\n\0".as_ptr() as *const c_char, (*cmd).name);
            i += 1;
            cmd = (*cmd).next;
        }
        Com_Printf(b"%i commands\n\0".as_ptr() as *const c_char, i);
    }
}
