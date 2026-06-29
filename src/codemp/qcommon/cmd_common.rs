// cmd.c -- Quake script command processing module

use core::ffi::{c_char, c_int, c_void};
use core::ptr::addr_of_mut;

const MAX_CMD_BUFFER: usize = 16384;
const MAX_CMD_LINE: usize = 1024;

// Local stubs for constants that must be defined for array sizing
// These should be verified against the header definitions
const MAX_STRING_TOKENS: usize = 1024;
const BIG_INFO_STRING: usize = 8192;
const MAX_STRING_CHARS: usize = 8192;
const MAX_QPATH: usize = 64;

#[repr(C)]
struct cmd_t {
    data: *mut u8,
    maxsize: c_int,
    cursize: c_int,
}

static mut cmd_wait: c_int = 0;
static mut cmd_text: cmd_t = cmd_t {
    data: core::ptr::null_mut(),
    maxsize: 0,
    cursize: 0,
};
static mut cmd_text_buf: [u8; MAX_CMD_BUFFER] = [0; MAX_CMD_BUFFER];

// External C functions
extern "C" {
    fn Com_Printf(format: *const c_char, ...);
    fn Com_Memcpy(dst: *mut c_void, src: *const c_void, len: usize);
    fn Com_Error(code: c_int, format: *const c_char, ...);
    fn Cmd_ExecuteString(text: *const c_char);
    fn strlen(s: *const c_char) -> usize;
    fn atoi(s: *const c_char) -> c_int;
    fn Q_strncpyz(dst: *mut c_char, src: *const c_char, len: usize);
    fn COM_DefaultExtension(path: *mut c_char, pathsize: usize, ext: *const c_char);
    fn FS_ReadFile(name: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buf: *mut c_void);
    fn Cvar_VariableString(name: *const c_char) -> *const c_char;
    fn va(format: *const c_char, ...) -> *const c_char;
    fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void;
    fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn Cmd_AddCommand(name: *const c_char, func: unsafe extern "C" fn());
    fn Cmd_List_f();
}

static mut cmd_argc: c_int = 0;
static mut cmd_argv: [*const c_char; MAX_STRING_TOKENS] = [core::ptr::null(); MAX_STRING_TOKENS];
static mut cmd_tokenized: [c_char; BIG_INFO_STRING + MAX_STRING_TOKENS] = [0; BIG_INFO_STRING + MAX_STRING_TOKENS];

//=============================================================================

/*
============
Cmd_Wait_f

Causes execution of the remainder of the command buffer to be delayed until
next frame.  This allows commands like:
bind g "cmd use rocket ; +attack ; wait ; -attack ; cmd use blaster"
============
*/
pub unsafe extern "C" fn Cmd_Wait_f() {
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
pub unsafe extern "C" fn Cbuf_Init() {
    cmd_text.data = addr_of_mut!(cmd_text_buf) as *mut u8;
    cmd_text.maxsize = MAX_CMD_BUFFER as c_int;
    cmd_text.cursize = 0;
}

/*
============
Cbuf_AddText

Adds command text at the end of the buffer, does NOT add a final \n
============
*/
pub unsafe extern "C" fn Cbuf_AddText(text: *const c_char) {
    let l: usize;

    l = strlen(text);

    if cmd_text.cursize as usize + l >= cmd_text.maxsize as usize
    {
        Com_Printf(c"Cbuf_AddText: overflow\n".as_ptr());
        return;
    }
    Com_Memcpy(
        cmd_text.data.add(cmd_text.cursize as usize) as *mut c_void,
        text as *const c_void,
        l,
    );
    cmd_text.cursize += l as c_int;
}

/*
============
Cbuf_InsertText

Adds command text immediately after the current command
Adds a \n to the text
============
*/
pub unsafe extern "C" fn Cbuf_InsertText(text: *const c_char) {
    let len: usize;

    len = strlen(text) + 1;
    if len as c_int + cmd_text.cursize > cmd_text.maxsize {
        Com_Printf(c"Cbuf_InsertText overflowed\n".as_ptr());
        return;
    }

    // move the existing command text
    let mut i = cmd_text.cursize - 1;
    while i >= 0 {
        *cmd_text.data.add((i as usize) + len) = *cmd_text.data.add(i as usize);
        i -= 1;
    }

    // copy the new text in
    Com_Memcpy(
        cmd_text.data as *mut c_void,
        text as *const c_void,
        len - 1,
    );

    // add a \n
    *cmd_text.data.add(len - 1) = b'\n' as u8;

    cmd_text.cursize += len as c_int;
}

/*
============
Cbuf_ExecuteText
============
*/
pub unsafe extern "C" fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char) {
    match exec_when {
        0 => {
            // EXEC_NOW
            if !text.is_null() && strlen(text) > 0 {
                Cmd_ExecuteString(text);
            } else {
                Cbuf_Execute();
            }
        }
        1 => {
            // EXEC_INSERT
            Cbuf_InsertText(text);
        }
        2 => {
            // EXEC_APPEND
            Cbuf_AddText(text);
        }
        _ => {
            Com_Error(3, c"Cbuf_ExecuteText: bad exec_when".as_ptr());
        }
    }
}

/*
============
Cbuf_Execute
============
*/
pub unsafe extern "C" fn Cbuf_Execute() {
    let mut line: [u8; MAX_CMD_LINE] = [0; MAX_CMD_LINE];

    while cmd_text.cursize > 0
    {
        if cmd_wait > 0 {
            // skip out while text still remains in buffer, leaving it
            // for next frame
            cmd_wait -= 1;
            break;
        }

        // find a \n or ; line break
        let text = cmd_text.data as *const u8;

        let mut quotes = 0;
        let mut i = 0;
        while (i as usize) < cmd_text.cursize as usize
        {
            if *text.add(i as usize) == b'"' {
                quotes += 1;
            }
            if (quotes & 1) == 0 && *text.add(i as usize) == b';' {
                break; // don't break if inside a quoted string
            }
            if *text.add(i as usize) == b'\n' || *text.add(i as usize) == b'\r' {
                break;
            }
            i += 1;
        }

        let i = if i >= MAX_CMD_LINE - 1 {
            MAX_CMD_LINE - 1
        } else {
            i as usize
        };

        Com_Memcpy(
            line.as_mut_ptr() as *mut c_void,
            text as *const c_void,
            i,
        );
        line[i] = 0;

        // delete the text from the command buffer and move remaining commands down
        // this is necessary because commands (exec) can insert data at the
        // beginning of the text buffer

        if i as c_int == cmd_text.cursize {
            cmd_text.cursize = 0;
        } else {
            cmd_text.cursize -= (i + 1) as c_int;
            memmove(
                text as *mut c_void,
                text.add(i + 1) as *const c_void,
                cmd_text.cursize as usize,
            );
        }

        // execute the command line

        Cmd_ExecuteString(line.as_ptr() as *const c_char);
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
pub unsafe extern "C" fn Cmd_Exec_f() {
    let len: c_int;
    let mut filename: [c_char; MAX_QPATH] = [0; MAX_QPATH];

    if Cmd_Argc() != 2 {
        Com_Printf(c"exec <filename> : execute a script file\n".as_ptr());
        return;
    }

    Q_strncpyz(filename.as_mut_ptr(), Cmd_Argv(1), MAX_QPATH);
    COM_DefaultExtension(filename.as_mut_ptr(), MAX_QPATH, c".cfg".as_ptr());
    let mut f: *mut c_void = core::ptr::null_mut();
    len = FS_ReadFile(filename.as_ptr(), &mut f);
    if f.is_null() {
        Com_Printf(c"couldn't exec %s\n".as_ptr(), Cmd_Argv(1));
        return;
    }
    Com_Printf(c"execing %s\n".as_ptr(), Cmd_Argv(1));

    Cbuf_InsertText(f as *const c_char);

    FS_FreeFile(f);
}

/*
===============
Cmd_Vstr_f

Inserts the current value of a variable as command text
===============
*/
pub unsafe extern "C" fn Cmd_Vstr_f() {
    let v: *const c_char;

    if Cmd_Argc() != 2 {
        Com_Printf(c"vstr <variablename> : execute a variable command\n".as_ptr());
        return;
    }

    v = Cvar_VariableString(Cmd_Argv(1));
    Cbuf_InsertText(va(c"%s\n".as_ptr(), v));
}

/*
===============
Cmd_Echo_f

Just prints the rest of the line to the console
===============
*/
pub unsafe extern "C" fn Cmd_Echo_f() {
    let mut i: c_int = 1;

    while i < Cmd_Argc() {
        Com_Printf(c"%s ".as_ptr(), Cmd_Argv(i));
        i += 1;
    }
    Com_Printf(c"\n".as_ptr());
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
pub fn Cmd_Argc() -> c_int {
    unsafe { cmd_argc }
}

/*
============
Cmd_Argv
============
*/
pub fn Cmd_Argv(arg: c_int) -> *const c_char {
    unsafe {
        if (arg as u32) >= cmd_argc as u32 {
            c"".as_ptr()
        } else {
            cmd_argv[arg as usize]
        }
    }
}

/*
============
Cmd_ArgvBuffer

The interpreted versions use this because
they can't have pointers returned to them
============
*/
pub unsafe extern "C" fn Cmd_ArgvBuffer(arg: c_int, buffer: *mut c_char, bufferLength: c_int) {
    Q_strncpyz(buffer, Cmd_Argv(arg), bufferLength as usize);
}

/*
============
Cmd_Args

Returns a single string containing argv(1) to argv(argc()-1)
============
*/
pub unsafe extern "C" fn Cmd_Args() -> *const c_char {
    static mut cmd_args: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

    cmd_args[0] = 0;
    let mut i = 1;
    while i < cmd_argc {
        strcat(cmd_args.as_mut_ptr(), Cmd_Argv(i));
        if i != cmd_argc - 1 {
            strcat(cmd_args.as_mut_ptr(), c" ".as_ptr());
        }
        i += 1;
    }

    cmd_args.as_ptr()
}

/*
============
Cmd_Args

Returns a single string containing argv(arg) to argv(argc()-1)
============
*/
pub unsafe extern "C" fn Cmd_ArgsFrom(mut arg: c_int) -> *const c_char {
    static mut cmd_args: [c_char; BIG_INFO_STRING] = [0; BIG_INFO_STRING];

    cmd_args[0] = 0;
    if arg < 0 {
        arg = 0;
    }
    let mut i = arg;
    while i < cmd_argc {
        strcat(cmd_args.as_mut_ptr(), Cmd_Argv(i));
        if i != cmd_argc - 1 {
            strcat(cmd_args.as_mut_ptr(), c" ".as_ptr());
        }
        i += 1;
    }

    cmd_args.as_ptr()
}

/*
============
Cmd_ArgsBuffer

The interpreted versions use this because
they can't have pointers returned to them
============
*/
pub unsafe extern "C" fn Cmd_ArgsBuffer(buffer: *mut c_char, bufferLength: c_int) {
    Q_strncpyz(buffer, Cmd_Args(), bufferLength as usize);
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
pub unsafe extern "C" fn Cmd_TokenizeString(text_in: *const c_char) {
    // clear previous args
    cmd_argc = 0;

    if text_in.is_null() {
        return;
    }

    let mut text = text_in;
    let mut textOut = addr_of_mut!(cmd_tokenized) as *mut c_char;

    loop {
        if cmd_argc == MAX_STRING_TOKENS as c_int {
            return; // this is usually something malicious
        }

        loop {
            // skip whitespace
            while !(*text).is_null() && (*text as u8) <= b' ' {
                text = text.add(1);
            }
            if (*text).is_null() {
                return; // all tokens parsed
            }

            // skip // comments
            if *text == b'/' as c_char && *text.add(1) == b'/' as c_char {
                return; // all tokens parsed
            }

            // skip /* */ comments
            if *text == b'/' as c_char && *text.add(1) == b'*' as c_char {
                while !(*text).is_null() && !(*text == b'*' as c_char && *text.add(1) == b'/' as c_char) {
                    text = text.add(1);
                }
                if (*text).is_null() {
                    return; // all tokens parsed
                }
                text = text.add(2);
            } else {
                break; // we are ready to parse a token
            }
        }

        // handle quoted strings
        if *text == b'"' as c_char {
            cmd_argv[cmd_argc as usize] = textOut;
            cmd_argc += 1;
            text = text.add(1);
            while !(*text).is_null() && *text != b'"' as c_char {
                *textOut = *text;
                textOut = textOut.add(1);
                text = text.add(1);
            }
            *textOut = 0;
            textOut = textOut.add(1);
            if (*text).is_null() {
                return; // all tokens parsed
            }
            text = text.add(1);
            continue;
        }

        // regular token
        cmd_argv[cmd_argc as usize] = textOut;
        cmd_argc += 1;

        // skip until whitespace, quote, or command
        while (*(text as *const u8) /*eurofix*/) > b' '
        {
            if *text == b'"' as c_char {
                break;
            }

            if *text == b'/' as c_char && *text.add(1) == b'/' as c_char {
                break;
            }

            // skip /* */ comments
            if *text == b'/' as c_char && *text.add(1) == b'*' as c_char {
                break;
            }

            *textOut = *text;
            textOut = textOut.add(1);
            text = text.add(1);
        }

        *textOut = 0;
        textOut = textOut.add(1);

        if (*text).is_null() {
            return; // all tokens parsed
        }
    }
}

/*
============
Cmd_Init
============
*/
pub unsafe extern "C" fn Cmd_Init() {
    Cmd_AddCommand(c"cmdlist".as_ptr(), Cmd_List_f);
    Cmd_AddCommand(c"exec".as_ptr(), Cmd_Exec_f);
    Cmd_AddCommand(c"vstr".as_ptr(), Cmd_Vstr_f);
    Cmd_AddCommand(c"echo".as_ptr(), Cmd_Echo_f);
    Cmd_AddCommand(c"wait".as_ptr(), Cmd_Wait_f);
}
