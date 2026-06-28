// cvar.c -- dynamic variable tracking

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

type qboolean = c_int;
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

const MAX_CVARS: usize = 1024;
const MAX_CVAR_VALUE_STRING: usize = 256;
const MAX_STRING_TOKENS: usize = 256;
const MAX_INFO_STRING: usize = 1024;

// CVAR flags
const CVAR_TEMP: c_int = 0;
const CVAR_ARCHIVE: c_int = 1;
const CVAR_USERINFO: c_int = 2;
const CVAR_SERVERINFO: c_int = 4;
const CVAR_SYSTEMINFO: c_int = 8;
const CVAR_INIT: c_int = 16;
const CVAR_LATCH: c_int = 32;
const CVAR_ROM: c_int = 64;
const CVAR_USER_CREATED: c_int = 128;
const CVAR_SAVEGAME: c_int = 256;
const CVAR_CHEAT: c_int = 512;
const CVAR_NORESTART: c_int = 1024;

// Error codes
const ERR_FATAL: c_int = 0;
const ERR_DROP: c_int = 1;

#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
}

impl cvar_t {
    const fn new() -> Self {
        cvar_t {
            name: core::ptr::null_mut(),
            string: core::ptr::null_mut(),
            resetString: core::ptr::null_mut(),
            latchedString: core::ptr::null_mut(),
            flags: 0,
            modified: qfalse,
            modificationCount: 0,
            value: 0.0,
            integer: 0,
            next: core::ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct vmCvar_t {
    pub handle: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub string: [c_char; MAX_CVAR_VALUE_STRING],
}

// External C functions
extern "C" {
    fn Z_Free(ptr: *mut c_void);
    fn Z_Malloc(size: usize, tag: c_int, clear: qboolean) -> *mut c_void;
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn Com_Filter(filter: *mut c_char, name: *mut c_char, casesensitive: c_int) -> c_int;
    fn CopyString(in_: *const c_char) -> *mut c_char;
    fn Cmd_Argv(arg: c_int) -> *mut c_char;
    fn Cmd_Argc() -> c_int;
    fn Cmd_AddCommand(cmd_name: *const c_char, function: Option<unsafe extern "C" fn()>);
    fn FS_Printf(f: c_int, fmt: *const c_char, ...) -> c_int;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int, ...);
    fn strlen(s: *const c_char) -> usize;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn atof(s: *const c_char) -> f32;
    fn atoi(s: *const c_char) -> c_int;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn sprintf(s: *mut c_char, fmt: *const c_char, ...) -> c_int;
    fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);
}

pub static mut cvar_vars: *mut cvar_t = core::ptr::null_mut();
pub static mut cvar_cheats: *mut cvar_t = core::ptr::null_mut();
pub static mut cvar_modifiedFlags: c_int = 0;

static mut cvar_indexes: [cvar_t; MAX_CVARS] = [cvar_t::new(); MAX_CVARS];
static mut cvar_numIndexes: c_int = 0;

static mut lastMemPool: *mut c_char = core::ptr::null_mut();
static mut memPoolSize: c_int = 0;

// If the string came from the memory pool, don't really free it.  The entire
// memory pool will be wiped during the next level load.
unsafe fn Cvar_FreeString(string: *mut c_char) {
    if !lastMemPool.is_null()
        && (string as usize) >= (lastMemPool as usize)
        && (string as usize) < ((lastMemPool as usize) + (memPoolSize as usize))
    {
        // String is in the memory pool, don't free it
    } else {
        Z_Free(string as *mut c_void);
    }
}

/*
============
Cvar_ValidateString
============
*/
unsafe fn Cvar_ValidateString(s: *const c_char) -> qboolean {
    if s.is_null() {
        return qfalse;
    }
    if !strchr(s, '\\' as c_int).is_null() {
        return qfalse;
    }
    if !strchr(s, '"' as c_int).is_null() {
        return qfalse;
    }
    if !strchr(s, ';' as c_int).is_null() {
        return qfalse;
    }
    qtrue
}

/*
============
Cvar_FindVar
============
*/
unsafe fn Cvar_FindVar(var_name: *const c_char) -> *mut cvar_t {
    let mut var = cvar_vars;

    while !var.is_null() {
        if Q_stricmp(var_name, (*var).name) == 0 {
            return var;
        }
        var = (*var).next;
    }

    core::ptr::null_mut()
}

/*
============
Cvar_VariableValue
============
*/
pub unsafe extern "C" fn Cvar_VariableValue(var_name: *const c_char) -> f32 {
    let var = Cvar_FindVar(var_name);
    if var.is_null() {
        return 0.0;
    }
    (*var).value
}

/*
============
Cvar_VariableIntegerValue
============
*/
pub unsafe extern "C" fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int {
    let var = Cvar_FindVar(var_name);
    if var.is_null() {
        return 0;
    }
    (*var).integer
}

/*
============
Cvar_VariableString
============
*/
pub unsafe extern "C" fn Cvar_VariableString(var_name: *const c_char) -> *mut c_char {
    let var = Cvar_FindVar(var_name);
    if var.is_null() {
        return b"" as *const u8 as *mut c_char;
    }
    (*var).string
}

/*
============
Cvar_VariableStringBuffer
============
*/
pub unsafe extern "C" fn Cvar_VariableStringBuffer(
    var_name: *const c_char,
    buffer: *mut c_char,
    bufsize: c_int,
) {
    let var = Cvar_FindVar(var_name);
    if var.is_null() {
        *buffer = 0;
    } else {
        Q_strncpyz(buffer, (*var).string, bufsize);
    }
}

/*
============
Cvar_CompleteVariable
============
*/
pub unsafe extern "C" fn Cvar_CompleteVariable(partial: *const c_char) -> *mut c_char {
    let len = strlen(partial);

    if len == 0 {
        return core::ptr::null_mut();
    }

    // check partial match
    let mut cvar = cvar_vars;
    while !cvar.is_null() {
        if Q_stricmpn(partial, (*cvar).name, len) == 0 {
            if ((*cvar).flags & CVAR_CHEAT) != 0 && (*cvar_cheats).integer == 0 {
                // continue
            } else {
                return (*cvar).name;
            }
        }
        cvar = (*cvar).next;
    }

    core::ptr::null_mut()
}

/*
============
Cvar_CompleteVariableNext - get the next cvar in alphabetical order.

============
*/
pub unsafe extern "C" fn Cvar_CompleteVariableNext(partial: *mut c_char, last: *mut c_char) -> *mut c_char {
    let len = strlen(partial as *const c_char);

    if len == 0 {
        return core::ptr::null_mut();
    }

    // this check needed since cvars may be resetting from cmd searches
    let mut base: *mut cvar_t = core::ptr::null_mut();
    if !last.is_null() {
        let mut cvar = cvar_vars;
        while !cvar.is_null() {
            if Q_stricmp(last as *const c_char, (*cvar).name) == 0 {
                base = (*cvar).next;
                break;
            }
            cvar = (*cvar).next;
        }
        if base.is_null() {
            //not found, either error or at end of list
            return core::ptr::null_mut();
        }
    } else {
        base = cvar_vars;
    }

    // check partial match
    let mut cvar = base;
    while !cvar.is_null() {
        if Q_stricmpn(partial as *const c_char, (*cvar).name, len) == 0 {
            if ((*cvar).flags & CVAR_CHEAT) != 0 && (*cvar_cheats).integer == 0 {
                // continue
            } else {
                return (*cvar).name;
            }
        }
        cvar = (*cvar).next;
    }

    core::ptr::null_mut()
}

// Forward declaration
unsafe fn Cvar_Set2(var_name: *const c_char, value: *const c_char, force: qboolean) -> *mut cvar_t;

/*
============
Cvar_Get

If the variable already exists, the value will not be set unless CVAR_ROM
The flags will be or'ed in if the variable exists.
============
*/
pub unsafe extern "C" fn Cvar_Get(
    var_name: *const c_char,
    var_value: *const c_char,
    flags: c_int,
) -> *mut cvar_t {
    if var_name.is_null() || var_value.is_null() {
        Com_Error(ERR_FATAL, b"Cvar_Get: NULL parameter\0" as *const u8 as *const c_char);
    }

    let mut var_name_local = var_name;
    if Cvar_ValidateString(var_name) == qfalse {
        Com_Printf(
            b"invalid cvar name string: %s\n\0" as *const u8 as *const c_char,
            var_name,
        );
        var_name_local = b"BADNAME\0" as *const u8 as *const c_char;
    }

    // #if 0		// FIXME: values with backslash happen
    // 	if ( !Cvar_ValidateString( var_value ) ) {
    // 		Com_Printf("invalid cvar value string: %s\n", var_value );
    // 		var_value = "BADVALUE";
    // 	}
    // #endif

    let mut var = Cvar_FindVar(var_name_local);
    if !var.is_null() {
        // if the C code is now specifying a variable that the user already
        // set a value for, take the new value as the reset value
        if ((*var).flags & CVAR_USER_CREATED) != 0 && (flags & CVAR_USER_CREATED) == 0
            && *var_value != 0
        {
            (*var).flags &= !CVAR_USER_CREATED;
            Cvar_FreeString((*var).resetString);
            (*var).resetString = CopyString(var_value);

            // ZOID--needs to be set so that cvars the game sets as
            // SERVERINFO get sent to clients
            cvar_modifiedFlags |= flags;
        }

        (*var).flags |= flags;
        // only allow one non-empty reset string without a warning
        if *(*var).resetString == 0 {
            // we don't have a reset string yet
            Cvar_FreeString((*var).resetString);
            (*var).resetString = CopyString(var_value);
        } else if *var_value != 0 && strcmp((*var).resetString, var_value) != 0 {
            Com_Printf(
                b"Warning: cvar \"%s\" given initial values: \"%s\" and \"%s\"\n\0"
                    as *const u8 as *const c_char,
                var_name_local,
                (*var).resetString,
                var_value,
            );
        }
        // if we have a latched string, take that value now
        if !(*var).latchedString.is_null() {
            let s = (*var).latchedString;

            (*var).latchedString = core::ptr::null_mut(); // otherwise cvar_set2 would free it
            Cvar_Set2(var_name_local, s, qtrue);
            Cvar_FreeString(s);
        }

        // use a CVAR_SET for rom sets, get won't override
        // #if 0
        //		// CVAR_ROM always overrides
        //		if ( flags & CVAR_ROM ) {
        //			Cvar_Set2( var_name, var_value, qtrue );
        //		}
        // #endif
        return var;
    }

    //
    // allocate a new cvar
    //
    if cvar_numIndexes as usize == MAX_CVARS {
        Com_Error(ERR_FATAL, b"MAX_CVARS\0" as *const u8 as *const c_char);
    }
    var = &mut cvar_indexes[cvar_numIndexes as usize];
    cvar_numIndexes += 1;
    (*var).name = CopyString(var_name_local);
    (*var).string = CopyString(var_value);
    (*var).modified = qtrue;
    (*var).modificationCount = 1;
    (*var).value = atof((*var).string);
    (*var).integer = atoi((*var).string);
    (*var).resetString = CopyString(var_value);

    // link the variable in
    (*var).next = cvar_vars;
    cvar_vars = var;

    (*var).flags = flags;

    var
}

/*
============
Cvar_Set2
============
*/
unsafe fn Cvar_Set2(var_name: *const c_char, value: *const c_char, force: qboolean) -> *mut cvar_t {
    let mut var_name_local = var_name;

    Com_DPrintf(b"Cvar_Set2: %s %s\n\0" as *const u8 as *const c_char, var_name, value);

    if Cvar_ValidateString(var_name) == qfalse {
        Com_Printf(
            b"invalid cvar name string: %s\n\0" as *const u8 as *const c_char,
            var_name,
        );
        var_name_local = b"BADNAME\0" as *const u8 as *const c_char;
    }

    // #if 0	// FIXME
    // 	if ( value && !Cvar_ValidateString( value ) ) {
    // 		Com_Printf("invalid cvar value string: %s\n", value );
    // 		var_value = "BADVALUE";
    // 	}
    // #endif

    let mut var = Cvar_FindVar(var_name_local);
    if var.is_null() {
        if value.is_null() {
            return core::ptr::null_mut();
        }
        // create it
        if force == qfalse {
            return Cvar_Get(var_name_local, value, CVAR_USER_CREATED);
        } else {
            return Cvar_Get(var_name_local, value, 0);
        }
    }

    let mut value_local = value;
    if value_local.is_null() {
        value_local = (*var).resetString;
    }

    // note what types of cvars have been modified (userinfo, archive, serverinfo, systeminfo)
    cvar_modifiedFlags |= (*var).flags;

    if force == qfalse {
        if ((*var).flags & CVAR_ROM) != 0 {
            Com_Printf(
                b"%s is read only.\n\0" as *const u8 as *const c_char,
                var_name_local,
            );
            return var;
        }

        if ((*var).flags & CVAR_INIT) != 0 {
            Com_Printf(
                b"%s is write protected.\n\0" as *const u8 as *const c_char,
                var_name_local,
            );
            return var;
        }

        if ((*var).flags & CVAR_LATCH) != 0 {
            if !(*var).latchedString.is_null() {
                if strcmp(value_local, (*var).latchedString) == 0 {
                    return var;
                }
                Cvar_FreeString((*var).latchedString);
            } else {
                if strcmp(value_local, (*var).string) == 0 {
                    return var;
                }
            }

            Com_Printf(
                b"%s will be changed upon restarting.\n\0" as *const u8 as *const c_char,
                var_name_local,
            );
            (*var).latchedString = CopyString(value_local);
            (*var).modified = qtrue;
            (*var).modificationCount += 1;
            return var;
        }

        if ((*var).flags & CVAR_CHEAT) != 0 && (*cvar_cheats).integer == 0 {
            Com_Printf(
                b"%s is cheat protected.\n\0" as *const u8 as *const c_char,
                var_name_local,
            );
            return var;
        }
    } else {
        if !(*var).latchedString.is_null() {
            Cvar_FreeString((*var).latchedString);
            (*var).latchedString = core::ptr::null_mut();
        }
    }

    if strcmp(value_local, (*var).string) == 0 {
        return var; // not changed
    }

    (*var).modified = qtrue;
    (*var).modificationCount += 1;

    Cvar_FreeString((*var).string); // free the old value string

    (*var).string = CopyString(value_local);
    (*var).value = atof((*var).string);
    (*var).integer = atoi((*var).string);

    var
}

/*
============
Cvar_Set
============
*/
pub unsafe extern "C" fn Cvar_Set(var_name: *const c_char, value: *const c_char) {
    Cvar_Set2(var_name, value, qtrue);
}

/*
============
Cvar_SetValue
============
*/
pub unsafe extern "C" fn Cvar_SetValue(var_name: *const c_char, value: f32) {
    let mut val: [c_char; 32] = [0; 32];

    if value == value as c_int as f32 {
        Com_sprintf(
            val.as_mut_ptr(),
            32,
            b"%i\0" as *const u8 as *const c_char,
            value as c_int,
        );
    } else {
        Com_sprintf(
            val.as_mut_ptr(),
            32,
            b"%f\0" as *const u8 as *const c_char,
            value,
        );
    }
    Cvar_Set(var_name, val.as_ptr());
}

/*
============
Cvar_Reset
============
*/
pub unsafe extern "C" fn Cvar_Reset(var_name: *const c_char) {
    Cvar_Set2(var_name, core::ptr::null(), qfalse);
}

/*
============
Cvar_SetCheatState

Any testing variables will be reset to the safe values
============
*/
pub unsafe extern "C" fn Cvar_SetCheatState() {
    // set all default vars to the safe value
    let mut var = cvar_vars;
    while !var.is_null() {
        if ((*var).flags & CVAR_CHEAT) != 0 {
            Cvar_Set((*var).name, (*var).resetString);
        }
        var = (*var).next;
    }
}

/*
============
Cvar_Command

Handles variable inspection and changing from the console
============
*/
pub unsafe extern "C" fn Cvar_Command() -> qboolean {
    // check variables
    let v = Cvar_FindVar(Cmd_Argv(0));
    if v.is_null() {
        return qfalse;
    }

    // perform a variable print or set
    if Cmd_Argc() == 1 {
        Com_Printf(
            b"\"%s\" is:\"%s" b"\x1b[0m" b"\" default:\"%s" b"\x1b[0m" b"\"\n\0"
                as *const u8 as *const c_char,
            (*v).name,
            (*v).string,
            (*v).resetString,
        );
        if !(*v).latchedString.is_null() {
            Com_Printf(
                b"latched: \"%s\"\n\0" as *const u8 as *const c_char,
                (*v).latchedString,
            );
        }
        return qtrue;
    }

    //JFM toggle test
    let value = Cmd_Argv(1);
    if *value as u8 as char == '!' {
        //toggle
        let mut buff: [c_char; 5] = [0; 5];
        sprintf(
            buff.as_mut_ptr(),
            b"%i\0" as *const u8 as *const c_char,
            if (*v).value != 0.0 { 0 } else { 1 },
        );
        Cvar_Set2((*v).name, buff.as_ptr(), qfalse); // toggle the value
    } else {
        Cvar_Set2((*v).name, value, qfalse); // set the value if forcing isn't required
    }

    qtrue
}

/*
============
Cvar_Toggle_f

Toggles a cvar for easy single key binding
============
*/
pub unsafe extern "C" fn Cvar_Toggle_f() {
    if Cmd_Argc() != 2 {
        Com_Printf(b"usage: toggle <variable>\n\0" as *const u8 as *const c_char);
        return;
    }

    let mut v = Cvar_VariableIntegerValue(Cmd_Argv(1));
    v = if v != 0 { 0 } else { 1 };

    Cvar_Set2(Cmd_Argv(1), va(b"%i\0" as *const u8 as *const c_char, v), qfalse);
}

/*
============
Cvar_Set_f

Allows setting and defining of arbitrary cvars from console, even if they
weren't declared in C code.
============
*/
pub unsafe extern "C" fn Cvar_Set_f() {
    let mut combined: [c_char; MAX_STRING_TOKENS] = [0; MAX_STRING_TOKENS];

    let c = Cmd_Argc();
    if c < 3 {
        Com_Printf(b"usage: set <variable> <value>\n\0" as *const u8 as *const c_char);
        return;
    }

    combined[0] = 0;
    let mut l: usize = 0;
    let mut i = 2;
    while i < c as usize {
        let len = strlen(Cmd_Argv(i as c_int)) + 1;
        if l + len >= MAX_STRING_TOKENS - 2 {
            break;
        }
        strcat(combined.as_mut_ptr(), Cmd_Argv(i as c_int));
        if i != (c as usize) - 1 {
            strcat(combined.as_mut_ptr(), b" \0" as *const u8 as *const c_char);
        }
        l += len;
        i += 1;
    }
    Cvar_Set2(Cmd_Argv(1), combined.as_ptr(), qfalse);
}

/*
============
Cvar_SetU_f

As Cvar_Set, but also flags it as userinfo
============
*/
pub unsafe extern "C" fn Cvar_SetU_f() {
    if Cmd_Argc() != 3 {
        Com_Printf(b"usage: setu <variable> <value>\n\0" as *const u8 as *const c_char);
        return;
    }
    Cvar_Set_f();
    let v = Cvar_FindVar(Cmd_Argv(1));
    if v.is_null() {
        return;
    }
    (*v).flags |= CVAR_USERINFO;
}

/*
============
Cvar_SetS_f

As Cvar_Set, but also flags it as userinfo
============
*/
pub unsafe extern "C" fn Cvar_SetS_f() {
    if Cmd_Argc() != 3 {
        Com_Printf(b"usage: sets <variable> <value>\n\0" as *const u8 as *const c_char);
        return;
    }
    Cvar_Set_f();
    let v = Cvar_FindVar(Cmd_Argv(1));
    if v.is_null() {
        return;
    }
    (*v).flags |= CVAR_SERVERINFO;
}

/*
============
Cvar_SetA_f

As Cvar_Set, but also flags it as archived
============
*/
pub unsafe extern "C" fn Cvar_SetA_f() {
    if Cmd_Argc() != 3 {
        Com_Printf(b"usage: seta <variable> <value>\n\0" as *const u8 as *const c_char);
        return;
    }
    Cvar_Set_f();
    let v = Cvar_FindVar(Cmd_Argv(1));
    if v.is_null() {
        return;
    }
    (*v).flags |= CVAR_ARCHIVE;
}

/*
============
Cvar_Reset_f
============
*/
pub unsafe extern "C" fn Cvar_Reset_f() {
    if Cmd_Argc() != 2 {
        Com_Printf(b"usage: reset <variable>\n\0" as *const u8 as *const c_char);
        return;
    }
    Cvar_Reset(Cmd_Argv(1));
}

/*
============
Cvar_WriteVariables

Appends lines containing "set variable value" for all variables
with the archive flag set to qtrue.
============
*/
pub unsafe extern "C" fn Cvar_WriteVariables(f: c_int) {
    // #ifndef _XBOX
    let mut buffer: [c_char; 1024] = [0; 1024];

    let mut var = cvar_vars;
    while !var.is_null() {
        if ((*var).flags & CVAR_ARCHIVE) != 0 {
            // write the latched value, even if it hasn't taken effect yet
            if !(*var).latchedString.is_null() {
                Com_sprintf(
                    buffer.as_mut_ptr(),
                    1024,
                    b"seta %s \"%s\"\n\0" as *const u8 as *const c_char,
                    (*var).name,
                    (*var).latchedString,
                );
            } else {
                Com_sprintf(
                    buffer.as_mut_ptr(),
                    1024,
                    b"seta %s \"%s\"\n\0" as *const u8 as *const c_char,
                    (*var).name,
                    (*var).string,
                );
            }
            FS_Printf(f, b"%s\0" as *const u8 as *const c_char, buffer.as_ptr());
        }
        var = (*var).next;
    }
    // #endif
}

/*
============
Cvar_List_f

============
*/
pub unsafe extern "C" fn Cvar_List_f() {
    let match_: *mut c_char;

    if Cmd_Argc() > 1 {
        match_ = Cmd_Argv(1);
    } else {
        match_ = core::ptr::null_mut();
    }

    let mut i: c_int = 0;
    let mut var = cvar_vars;
    while !var.is_null() {
        if !match_.is_null() && Com_Filter(match_, (*var).name, qfalse) == qfalse {
            var = (*var).next;
            continue;
        }

        if ((*var).flags & CVAR_SERVERINFO) != 0 {
            Com_Printf(b"S\0" as *const u8 as *const c_char);
        } else {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }
        if ((*var).flags & CVAR_USERINFO) != 0 {
            Com_Printf(b"U\0" as *const u8 as *const c_char);
        } else {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }
        if ((*var).flags & CVAR_ROM) != 0 {
            Com_Printf(b"R\0" as *const u8 as *const c_char);
        } else {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }
        if ((*var).flags & CVAR_INIT) != 0 {
            Com_Printf(b"I\0" as *const u8 as *const c_char);
        } else {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }
        if ((*var).flags & CVAR_ARCHIVE) != 0 {
            Com_Printf(b"A\0" as *const u8 as *const c_char);
        } else {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }
        if ((*var).flags & CVAR_LATCH) != 0 {
            Com_Printf(b"L\0" as *const u8 as *const c_char);
        } else {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }
        if ((*var).flags & CVAR_CHEAT) != 0 {
            if (*cvar_cheats).integer == 0 {
                i -= 1;
                var = (*var).next;
                continue;
            }
            Com_Printf(b"C\0" as *const u8 as *const c_char);
        } else {
            Com_Printf(b" \0" as *const u8 as *const c_char);
        }

        Com_Printf(
            b" %s \"%s\"\n\0" as *const u8 as *const c_char,
            (*var).name,
            (*var).string,
        );

        var = (*var).next;
        i += 1;
    }

    Com_Printf(
        b"\n%i total cvars\n\0" as *const u8 as *const c_char,
        i,
    );
}

/*
============
Cvar_Restart_f

Resets all cvars to their hardcoded values
============
*/
pub unsafe extern "C" fn Cvar_Restart_f() {
    let mut prev = &mut cvar_vars;
    loop {
        let var = *prev;
        if var.is_null() {
            break;
        }

        // don't mess with rom values, or some inter-module
        // communication will get broken (com_cl_running, etc)
        if ((*var).flags & (CVAR_ROM | CVAR_INIT | CVAR_NORESTART)) != 0 {
            prev = &mut (*var).next;
            continue;
        }

        // throw out any variables the user created
        if ((*var).flags & CVAR_USER_CREATED) != 0 {
            *prev = (*var).next;
            if !(*var).name.is_null() {
                Cvar_FreeString((*var).name);
            }
            if !(*var).string.is_null() {
                Cvar_FreeString((*var).string);
            }
            if !(*var).latchedString.is_null() {
                Cvar_FreeString((*var).latchedString);
            }
            if !(*var).resetString.is_null() {
                Cvar_FreeString((*var).resetString);
            }
            // clear the var completely, since we
            // can't remove the index from the list
            memset(var as *mut c_void, 0, core::mem::size_of_val(&*var));
            continue;
        }
        Cvar_Set((*var).name, (*var).resetString);
        prev = &mut (*var).next;
    }
}

/*
=====================
Cvar_InfoString
=====================
*/
pub unsafe extern "C" fn Cvar_InfoString(bit: c_int) -> *mut c_char {
    static mut info: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];

    info[0] = 0;

    let mut var = cvar_vars;
    while !var.is_null() {
        if ((*var).flags & bit) != 0 {
            Info_SetValueForKey(
                info.as_mut_ptr(),
                (*var).name,
                (*var).string,
            );
        }
        var = (*var).next;
    }
    info.as_mut_ptr()
}

/*
=====================
Cvar_InfoStringBuffer
=====================
*/
pub unsafe extern "C" fn Cvar_InfoStringBuffer(bit: c_int, buff: *mut c_char, buffsize: c_int) {
    Q_strncpyz(buff, Cvar_InfoString(bit), buffsize);
}

/*
=====================
Cvar_Register

basically a slightly modified Cvar_Get for the interpreted modules
=====================
*/
pub unsafe extern "C" fn Cvar_Register(
    vmCvar: *mut vmCvar_t,
    varName: *const c_char,
    defaultValue: *const c_char,
    flags: c_int,
) {
    let cv = Cvar_Get(varName, defaultValue, flags);
    if vmCvar.is_null() {
        return;
    }
    (*vmCvar).handle = (cv as usize - cvar_indexes.as_ptr() as usize) as c_int;
    (*vmCvar).modificationCount = -1;
    Cvar_Update(vmCvar);
}

/*
=====================
Cvar_Register

updates an interpreted modules' version of a cvar
=====================
*/
pub unsafe extern "C" fn Cvar_Update(vmCvar: *mut vmCvar_t) {
    if ((*vmCvar).handle as usize) >= (cvar_numIndexes as usize) {
        Com_Error(ERR_DROP, b"Cvar_Update: handle out of range\0" as *const u8 as *const c_char);
    }

    let cv = cvar_indexes.as_mut_ptr().add((*vmCvar).handle as usize);

    if (*cv).modificationCount == (*vmCvar).modificationCount {
        return;
    }
    if (*cv).string.is_null() {
        return; // variable might have been cleared by a cvar_restart
    }
    (*vmCvar).modificationCount = (*cv).modificationCount;
    Q_strncpyz(
        (*vmCvar).string.as_mut_ptr(),
        (*cv).string,
        MAX_CVAR_VALUE_STRING as c_int,
    );
    (*vmCvar).value = (*cv).value;
    (*vmCvar).integer = (*cv).integer;
}

/*
============
Cvar_Init

Reads in all archived cvars
============
*/
pub unsafe extern "C" fn Cvar_Init() {
    cvar_cheats = Cvar_Get(
        b"helpUsObi\0" as *const u8 as *const c_char,
        b"0\0" as *const u8 as *const c_char,
        CVAR_SYSTEMINFO,
    );

    Cmd_AddCommand(b"toggle\0" as *const u8 as *const c_char, Some(Cvar_Toggle_f));
    Cmd_AddCommand(b"set\0" as *const u8 as *const c_char, Some(Cvar_Set_f));
    Cmd_AddCommand(b"sets\0" as *const u8 as *const c_char, Some(Cvar_SetS_f));
    Cmd_AddCommand(b"setu\0" as *const u8 as *const c_char, Some(Cvar_SetU_f));
    Cmd_AddCommand(b"seta\0" as *const u8 as *const c_char, Some(Cvar_SetA_f));
    Cmd_AddCommand(b"reset\0" as *const u8 as *const c_char, Some(Cvar_Reset_f));
    Cmd_AddCommand(b"cvarlist\0" as *const u8 as *const c_char, Some(Cvar_List_f));
    Cmd_AddCommand(
        b"cvar_restart\0" as *const u8 as *const c_char,
        Some(Cvar_Restart_f),
    );
}

unsafe fn Cvar_Realloc(string: *mut *mut c_char, memPool: *mut c_char, memPoolUsed: &mut c_int) {
    if !string.is_null() && !(*string).is_null() {
        let temp = memPool.add(*memPoolUsed as usize);
        strcpy(temp, *string);
        *memPoolUsed += strlen(*string) as c_int + 1;
        Cvar_FreeString(*string);
        *string = temp;
    }
}

//Turns many small allocation blocks into one big one.
pub unsafe extern "C" fn Cvar_Defrag() {
    let mut totalMem: c_int = 0;

    let mut var = cvar_vars;
    while !var.is_null() {
        if !(*var).name.is_null() {
            totalMem += strlen((*var).name) as c_int + 1;
        }
        if !(*var).string.is_null() {
            totalMem += strlen((*var).string) as c_int + 1;
        }
        if !(*var).resetString.is_null() {
            totalMem += strlen((*var).resetString) as c_int + 1;
        }
        if !(*var).latchedString.is_null() {
            totalMem += strlen((*var).latchedString) as c_int + 1;
        }
        var = (*var).next;
    }

    let mem = Z_Malloc(totalMem as usize, 0, qfalse) as *mut c_char;
    let nextMemPoolSize = totalMem;
    totalMem = 0;

    var = cvar_vars;
    while !var.is_null() {
        Cvar_Realloc(&mut (*var).name, mem, &mut totalMem);
        Cvar_Realloc(&mut (*var).string, mem, &mut totalMem);
        Cvar_Realloc(&mut (*var).resetString, mem, &mut totalMem);
        Cvar_Realloc(&mut (*var).latchedString, mem, &mut totalMem);
        var = (*var).next;
    }

    if !lastMemPool.is_null() {
        Z_Free(lastMemPool as *mut c_void);
    }
    lastMemPool = mem;
    memPoolSize = nextMemPoolSize;
}
