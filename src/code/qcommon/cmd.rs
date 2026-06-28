// cmd.c -- Quake script command processing module

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

const MAX_CMD_BUFFER: usize = 8192;
const MAX_STRING_CHARS: usize = 1024;
const MAX_STRING_TOKENS: usize = 256;
const MAX_QPATH: usize = 64;
const CMD_MAX_NUM: usize = 256;
const CMD_MAX_NAME: usize = 32;

// Opaque types from headers
#[repr(C)]
pub struct msg_t {
    pub allowoverflow: c_int,
    pub overflowed: c_int,
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // Other fields not used in this file
}

pub type xcommand_t = Option<unsafe extern "C" fn()>;

#[repr(C)]
pub struct cmd_function_s {
    pub name: [c_char; CMD_MAX_NAME],
    pub function: xcommand_t,
}
pub type cmd_function_t = cmd_function_s;

// Globals
static mut cmd_wait: c_int = 0;
static mut cmd_text: msg_t = msg_t {
    allowoverflow: 0,
    overflowed: 0,
    data: core::ptr::null_mut(),
    maxsize: 0,
    cursize: 0,
    readcount: 0,
    bit: 0,
};
static mut cmd_text_buf: [u8; MAX_CMD_BUFFER] = [0; MAX_CMD_BUFFER];
static mut cmd_defer_text_buf: [c_char; MAX_CMD_BUFFER] = [0 as c_char; MAX_CMD_BUFFER];

// File-local static variables for command execution
static mut cmd_argc: c_int = 0;
static mut cmd_argv: [*mut c_char; MAX_STRING_TOKENS] = [core::ptr::null_mut(); MAX_STRING_TOKENS];
static mut cmd_tokenized: [c_char; MAX_STRING_CHARS + MAX_STRING_TOKENS] = [0 as c_char; MAX_STRING_CHARS + MAX_STRING_TOKENS];

static mut cmd_functions: [cmd_function_t; CMD_MAX_NUM] = [cmd_function_t {
    name: [0 as c_char; CMD_MAX_NAME],
    function: None,
}; CMD_MAX_NUM];

// External C functions
extern "C" {
    fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);
    fn MSG_WriteData(buf: *mut msg_t, data: *const c_void, length: c_int);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn atoi(s: *const c_char) -> c_int;
    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int, ...);
    fn COM_DefaultExtension(path: *mut c_char, length: c_int, extension: *const c_char);
    fn Cvar_VariableString(var_name: *const c_char) -> *mut c_char;
    fn Cvar_Command() -> c_int;
    fn CL_GameCommand() -> c_int;
    fn SV_GameCommand() -> c_int;
    fn UI_GameCommand() -> c_int;
    fn CL_ForwardCommandToServer();
    fn Com_Filter(filter: *mut c_char, name: *mut c_char, casesensitive: c_int) -> c_int;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn va(format: *const c_char, ...) -> *mut c_char;

    pub static mut com_cl_running: *mut cvar_t;
    pub static mut com_sv_running: *mut cvar_t;
}

//=============================================================================

/*
============
Cmd_Wait_f

Causes execution of the remainder of the command buffer to be delayed until
next frame.  This allows commands like:
bind g "cmd use rocket ; +attack ; wait ; -attack ; cmd use blaster"
============
*/
pub unsafe fn Cmd_Wait_f() {
    if Cmd_Argc() == 2 {
        cmd_wait = atoi(Cmd_Argv(1));
    } else {
        cmd_wait = 1;
    }
}

//=============================================================================

/*
=============================================================================

						COMMAND BUFFER

=============================================================================
*/

/*
============
Cbuf_Init
============
*/
pub unsafe fn Cbuf_Init() {
    MSG_Init(
        core::ptr::addr_of_mut!(cmd_text),
        core::ptr::addr_of_mut!(cmd_text_buf[0]),
        MAX_CMD_BUFFER as c_int,
    );
}

/*
============
Cbuf_AddText

Adds command text at the end of the buffer, does NOT add a final \n
============
*/
pub unsafe fn Cbuf_AddText(text: *const c_char) {
    let l = strlen(text);

    if cmd_text.cursize as usize + l >= cmd_text.maxsize as usize {
        Com_Printf(b"Cbuf_AddText: overflow\n" as *const u8 as *const c_char);
        return;
    }
    MSG_WriteData(
        core::ptr::addr_of_mut!(cmd_text),
        text as *const c_void,
        strlen(text) as c_int,
    );
}

/*
============
Cbuf_InsertText

Adds command text immediately after the current command
Adds a \n to the text
============
*/
pub unsafe fn Cbuf_InsertText(text: *const c_char) {
    let len = strlen(text) + 1;
    if len as c_int + cmd_text.cursize > cmd_text.maxsize {
        Com_Printf(b"Cbuf_InsertText overflowed\n" as *const u8 as *const c_char);
        return;
    }

    // move the existing command text
    let mut i = cmd_text.cursize - 1;
    loop {
        if i < 0 {
            break;
        }
        *cmd_text.data.offset((i + len as c_int) as isize) =
            *cmd_text.data.offset(i as isize);
        i -= 1;
    }

    // copy the new text in
    memcpy(
        cmd_text.data as *mut c_void,
        text as *const c_void,
        len - 1,
    );

    // add a \n
    *cmd_text.data.offset((len - 1) as isize) = b'\n' as u8;

    cmd_text.cursize += len as c_int;
}

/*
============
Cbuf_ExecuteText
============
*/
const EXEC_NOW: c_int = 0;
const EXEC_INSERT: c_int = 1;
const EXEC_APPEND: c_int = 2;

pub unsafe fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char) {
    match exec_when {
        EXEC_NOW => {
            Cmd_ExecuteString(text);
        }
        EXEC_INSERT => {
            Cbuf_InsertText(text);
        }
        EXEC_APPEND => {
            Cbuf_AddText(text);
        }
        _ => {
            Com_Error(
                1, // ERR_FATAL
                b"Cbuf_ExecuteText: bad exec_when\0" as *const u8 as *const c_char,
            );
        }
    }
}

/*
============
Cbuf_Execute
============
*/
pub unsafe fn Cbuf_Execute() {
    let mut i: c_int;
    let text: *mut c_char;
    let mut line: [c_char; MAX_CMD_BUFFER] = [0 as c_char; MAX_CMD_BUFFER];
    let mut quotes: c_int;

    while cmd_text.cursize > 0 {
        if cmd_wait != 0 {
            // skip out while text still remains in buffer, leaving it
            // for next frame
            cmd_wait -= 1;
            break;
        }

        // find a \n or ; line break
        text = cmd_text.data as *mut c_char;

        quotes = 0;
        i = 0;
        while i < cmd_text.cursize {
            if *text.offset(i as isize) as u8 == b'"' {
                quotes += 1;
            }
            if (quotes & 1) == 0 && *text.offset(i as isize) as u8 == b';' {
                break; // don't break if inside a quoted string
            }
            if *text.offset(i as isize) as u8 == b'\n'
                || *text.offset(i as isize) as u8 == b'\r'
            {
                break;
            }
            i += 1;
        }

        memcpy(
            line.as_mut_ptr() as *mut c_void,
            text as *const c_void,
            i as usize,
        );
        line[i as usize] = 0 as c_char;

        // delete the text from the command buffer and move remaining commands down
        // this is necessary because commands (exec) can insert data at the
        // beginning of the text buffer

        if i == cmd_text.cursize {
            cmd_text.cursize = 0;
        } else {
            i += 1;
            cmd_text.cursize -= i;
            memmove(
                text as *mut c_void,
                text.offset(i as isize) as *const c_void,
                cmd_text.cursize as usize,
            );
        }

        // execute the command line
        Cmd_ExecuteString(line.as_ptr());
    }
}

/*
==============================================================================

						SCRIPT COMMANDS

==============================================================================
*/

/*
===============
Cmd_Exec_f
===============
*/
pub unsafe fn Cmd_Exec_f() {
    let mut f: *mut c_void = core::ptr::null_mut();
    let len: c_int;
    let mut filename: [c_char; MAX_QPATH] = [0 as c_char; MAX_QPATH];

    if Cmd_Argc() != 2 {
        Com_Printf(b"exec <filename> : execute a script file\n" as *const u8 as *const c_char);
        return;
    }

    Q_strncpyz(
        filename.as_mut_ptr(),
        Cmd_Argv(1),
        MAX_QPATH as c_int,
    );
    COM_DefaultExtension(
        filename.as_mut_ptr(),
        MAX_QPATH as c_int,
        b".cfg\0" as *const u8 as *const c_char,
    );
    len = FS_ReadFile(filename.as_ptr(), core::ptr::addr_of_mut!(f));
    if f.is_null() {
        Com_Printf(
            b"couldn't exec %s\n\0" as *const u8 as *const c_char,
            Cmd_Argv(1),
        );
        return;
    }
    Com_Printf(
        b"execing %s\n\0" as *const u8 as *const c_char,
        Cmd_Argv(1),
    );

    Cbuf_InsertText(f as *const c_char);

    FS_FreeFile(f);
}

/*
===============
Cmd_Vstr_f

Inserts the current value of a variable as command text
===============
*/
pub unsafe fn Cmd_Vstr_f() {
    let v: *mut c_char;

    if Cmd_Argc() != 2 {
        Com_Printf(
            b"vstr <variablename> : execute a variable command\n" as *const u8 as *const c_char,
        );
        return;
    }

    v = Cvar_VariableString(Cmd_Argv(1));
    Cbuf_InsertText(va(b"%s\n\0" as *const u8 as *const c_char, v));
}

/*
===============
Cmd_Echo_f

Just prints the rest of the line to the console
===============
*/
pub unsafe fn Cmd_Echo_f() {
    let mut i: c_int = 1;

    while i < Cmd_Argc() {
        Com_Printf(b"%s \0" as *const u8 as *const c_char, Cmd_Argv(i));
        i += 1;
    }
    Com_Printf(b"\n\0" as *const u8 as *const c_char);
}

/*
=============================================================================

					COMMAND EXECUTION

=============================================================================
*/

/*
============
Cmd_Argc
============
*/
pub unsafe fn Cmd_Argc() -> c_int {
    cmd_argc
}

/*
============
Cmd_Argv
============
*/
pub unsafe fn Cmd_Argv(arg: c_int) -> *mut c_char {
    if (arg as usize) >= cmd_argc as usize {
        return b"\0" as *const u8 as *mut c_char;
    }
    cmd_argv[arg as usize]
}

/*
============
Cmd_ArgvBuffer

The interpreted versions use this because
they can't have pointers returned to them
============
*/
pub unsafe fn Cmd_ArgvBuffer(arg: c_int, buffer: *mut c_char, bufferLength: c_int) {
    Q_strncpyz(buffer, Cmd_Argv(arg), bufferLength);
}

/*
============
Cmd_Args

Returns a single string containing argv(1) to argv(argc()-1)
============
*/
pub unsafe fn Cmd_Args() -> *mut c_char {
    static mut cmd_args: [c_char; MAX_STRING_CHARS] = [0 as c_char; MAX_STRING_CHARS];
    let mut i: c_int;

    cmd_args[0] = 0 as c_char;
    i = 1;
    while i < cmd_argc {
        strcat(cmd_args.as_mut_ptr(), Cmd_Argv(i));
        if i != cmd_argc {
            strcat(cmd_args.as_mut_ptr(), b" \0" as *const u8 as *const c_char);
        }
        i += 1;
    }

    cmd_args.as_mut_ptr()
}

/*
============
Cmd_ArgsBuffer

The interpreted versions use this because
they can't have pointers returned to them
============
*/
pub unsafe fn Cmd_ArgsBuffer(buffer: *mut c_char, bufferLength: c_int) {
    Q_strncpyz(buffer, Cmd_Args(), bufferLength);
}

/*
============
Cmd_TokenizeString

Parses the given string into command line tokens.
The text is copied to a seperate buffer and 0 characters
are inserted in the apropriate place, The argv array
will point into this temporary buffer.
============
*/
pub unsafe fn Cmd_TokenizeString(text_in: *const c_char) {
    let mut text: *const u8;
    let mut textOut: *mut c_char;

    // clear previous args
    cmd_argc = 0;

    if text_in.is_null() {
        return;
    }

    text = text_in as *const u8;
    textOut = cmd_tokenized.as_mut_ptr();

    loop {
        if cmd_argc == MAX_STRING_TOKENS as c_int {
            return; // this is usually something malicious
        }

        loop {
            // skip whitespace
            while !(*text).is_null() && (*text) <= b' ' {
                text = text.offset(1);
            }
            if (*text).is_null() {
                return; // all tokens parsed
            }

            // skip // comments
            if *text == b'/' && *text.offset(1) == b'/' {
                return; // all tokens parsed
            }

            // skip /* */ comments
            if *text == b'/' && *text.offset(1) == b'*' {
                while !(*text).is_null() && !(*text == b'*' && *text.offset(1) == b'/') {
                    text = text.offset(1);
                }
                if (*text).is_null() {
                    return; // all tokens parsed
                }
                text = text.offset(2);
            } else {
                break; // we are ready to parse a token
            }
        }

        // handle quoted strings
        if *text == b'"' {
            cmd_argv[cmd_argc as usize] = textOut;
            cmd_argc += 1;
            text = text.offset(1);
            while !(*text).is_null() && *text != b'"' {
                *textOut = *text as c_char;
                textOut = textOut.offset(1);
                text = text.offset(1);
            }
            *textOut = 0 as c_char;
            textOut = textOut.offset(1);
            if (*text).is_null() {
                return; // all tokens parsed
            }
            text = text.offset(1);
            continue;
        }

        // regular token
        cmd_argv[cmd_argc as usize] = textOut;
        cmd_argc += 1;

        // skip until whitespace, quote, or command
        while *text > b' ' {
            if *text == b'"' {
                break;
            }

            if *text == b'/' && *text.offset(1) == b'/' {
                break;
            }

            // skip /* */ comments
            if *text == b'/' && *text.offset(1) == b'*' {
                break;
            }

            *textOut = *text as c_char;
            textOut = textOut.offset(1);
            text = text.offset(1);
        }

        *textOut = 0 as c_char;
        textOut = textOut.offset(1);

        if (*text).is_null() {
            return; // all tokens parsed
        }
    }
}

/*
============
Cmd_AddCommand
============
*/
pub unsafe fn Cmd_AddCommand(cmd_name: *const c_char, function: xcommand_t) {
    let mut cmd: *mut cmd_function_t;
    let mut add: *mut cmd_function_t = core::ptr::null_mut();
    let mut c: c_int;

    // fail if the command already exists
    c = 0;
    while c < CMD_MAX_NUM as c_int {
        cmd = core::ptr::addr_of_mut!(cmd_functions[c as usize]);
        if strcmp(cmd_name, (*cmd).name.as_ptr()) == 0 {
            // allow completion-only commands to be silently doubled
            if function.is_some() {
                Com_Printf(
                    b"Cmd_AddCommand: %s already defined\n\0" as *const u8 as *const c_char,
                    cmd_name,
                );
            }
            return;
        }

        if add.is_null() && (*cmd).name[0] == 0 as c_char {
            add = cmd;
        }
        c += 1;
    }

    if add.is_null() {
        Com_Printf(
            b"Cmd_AddCommand: Too many commands registered\n\0" as *const u8 as *const c_char,
            cmd_name,
        );
        return;
    }

    Q_strncpyz(
        (*add).name.as_mut_ptr(),
        cmd_name,
        CMD_MAX_NAME as c_int,
    );
    (*add).function = function;
}

/*
============
Cmd_RemoveCommand
============
*/
pub unsafe fn Cmd_RemoveCommand(cmd_name: *const c_char) {
    let mut cmd: *mut cmd_function_t;
    let mut c: c_int = 0;

    while c < CMD_MAX_NUM as c_int {
        cmd = core::ptr::addr_of_mut!(cmd_functions[c as usize]);
        if strcmp(cmd_name, (*cmd).name.as_ptr()) == 0 {
            (*cmd).name[0] = 0 as c_char;
            return;
        }
        c += 1;
    }
}

pub unsafe fn Cmd_CompleteCommandNext(partial: *mut c_char, last: *mut c_char) -> *mut c_char {
    let mut cmd: *mut cmd_function_t;
    let mut base: *mut cmd_function_t;
    let len: usize;
    let mut c: c_int;

    len = strlen(partial);

    if len == 0 {
        return core::ptr::null_mut();
    }

    // start past last match
    base = core::ptr::null_mut();
    if !last.is_null() {
        c = 0;
        while c < CMD_MAX_NUM as c_int {
            cmd = core::ptr::addr_of_mut!(cmd_functions[c as usize]);
            if strcmp(last, (*cmd).name.as_ptr()) == 0 {
                base = core::ptr::addr_of_mut!(cmd_functions[(c as usize + 1)]);
                break;
            }
            c += 1;
        }
        if base.is_null() {
            // not found, either error or at end of list
            return core::ptr::null_mut();
        }
    } else {
        base = core::ptr::addr_of_mut!(cmd_functions[0]);
    }

    let base_idx = base.offset_from(core::ptr::addr_of!(cmd_functions[0]));
    c = base_idx as c_int;
    while c < CMD_MAX_NUM as c_int {
        cmd = core::ptr::addr_of_mut!(cmd_functions[c as usize]);
        if strcmp(partial, (*cmd).name.as_ptr()) == 0 {
            return (*cmd).name.as_mut_ptr();
        }
        c += 1;
    }

    // check for partial match
    c = base_idx as c_int;
    while c < CMD_MAX_NUM as c_int {
        cmd = core::ptr::addr_of_mut!(cmd_functions[c as usize]);
        if strncmp(partial, (*cmd).name.as_ptr(), len) == 0 {
            return (*cmd).name.as_mut_ptr();
        }
        c += 1;
    }

    core::ptr::null_mut()
}

/*
============
Cmd_CompleteCommand
============
*/
pub unsafe fn Cmd_CompleteCommand(partial: *const c_char) -> *mut c_char {
    let mut cmd: *mut cmd_function_t;
    let len: usize;
    let mut c: c_int;

    len = strlen(partial);

    if len == 0 {
        return core::ptr::null_mut();
    }

    // check for exact match
    c = 0;
    while c < CMD_MAX_NUM as c_int {
        cmd = core::ptr::addr_of_mut!(cmd_functions[c as usize]);
        if Q_stricmp(partial, (*cmd).name.as_ptr()) == 0 {
            return (*cmd).name.as_mut_ptr();
        }
        c += 1;
    }

    // check for partial match
    c = 0;
    while c < CMD_MAX_NUM as c_int {
        cmd = core::ptr::addr_of_mut!(cmd_functions[c as usize]);
        if Q_stricmpn(partial, (*cmd).name.as_ptr(), len) == 0 {
            return (*cmd).name.as_mut_ptr();
        }
        c += 1;
    }

    core::ptr::null_mut()
}

/*
============
Cmd_ExecuteString

A complete command line has been parsed, so try to execute it
============
*/
pub unsafe fn Cmd_ExecuteString(text: *const c_char) {
    // execute the command line
    Cmd_TokenizeString(text);
    if Cmd_Argc() == 0 {
        return; // no tokens
    }

    // check registered command functions
    let mut c: c_int = 0;
    while c < CMD_MAX_NUM as c_int {
        if Q_stricmp(cmd_argv[0], cmd_functions[c as usize].name.as_ptr()) == 0 {
            // rearrange the links so that the command will be
            // near the head of the list next time it is used
            let temp = cmd_functions[c as usize];
            cmd_functions[c as usize] = cmd_functions[0];
            cmd_functions[0] = temp;

            // perform the action
            if temp.function.is_none() {
                // let the cgame or game handle it
                break;
            } else {
                if let Some(f) = temp.function {
                    f();
                }
            }
            return;
        }
        c += 1;
    }

    // check cvars
    if Cvar_Command() != 0 {
        return;
    }

    // check client game commands
    if !com_cl_running.is_null()
        && (*com_cl_running).integer != 0
        && CL_GameCommand() != 0
    {
        return;
    }

    // check server game commands
    if !com_sv_running.is_null()
        && (*com_sv_running).integer != 0
        && SV_GameCommand() != 0
    {
        return;
    }

    // check ui commands
    if !com_cl_running.is_null()
        && (*com_cl_running).integer != 0
        && UI_GameCommand() != 0
    {
        return;
    }

    // send it as a server command if we are connected
    // this will usually result in a chat message
    CL_ForwardCommandToServer();
}

/*
============
Cmd_List_f
============
*/
pub unsafe fn Cmd_List_f() {
    let mut cmd: *mut cmd_function_t;
    let mut i: c_int;
    let match_: *mut c_char;

    if Cmd_Argc() > 1 {
        match_ = Cmd_Argv(1);
    } else {
        match_ = core::ptr::null_mut();
    }

    i = 0;
    let mut c: c_int = 0;
    while c < CMD_MAX_NUM as c_int {
        cmd = core::ptr::addr_of_mut!(cmd_functions[c as usize]);
        if !match_.is_null()
            && Com_Filter(match_, (*cmd).name.as_mut_ptr(), 0) == 0
        {
            c += 1;
            continue;
        }
        Com_Printf(
            b"%s\n\0" as *const u8 as *const c_char,
            (*cmd).name.as_ptr(),
        );
        i += 1;
        c += 1;
    }
    Com_Printf(b"%i commands\n\0" as *const u8 as *const c_char, i);
}

/*
============
Cmd_Init
============
*/
pub unsafe fn Cmd_Init() {
    // register our commands
    Cmd_AddCommand(b"cmdlist\0" as *const u8 as *const c_char, Some(Cmd_List_f));
    Cmd_AddCommand(b"exec\0" as *const u8 as *const c_char, Some(Cmd_Exec_f));
    Cmd_AddCommand(b"vstr\0" as *const u8 as *const c_char, Some(Cmd_Vstr_f));
    Cmd_AddCommand(b"echo\0" as *const u8 as *const c_char, Some(Cmd_Echo_f));
    Cmd_AddCommand(b"wait\0" as *const u8 as *const c_char, Some(Cmd_Wait_f));
}
