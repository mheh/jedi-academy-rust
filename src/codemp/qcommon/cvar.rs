// cvar.c -- dynamic variable tracking

//Anything above this #include will be ignored by the compiler

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// Externs and type definitions
// ============================================================================

extern "C" {
    // From exe_headers.h and other modules
    fn Z_Free(ptr: *mut c_void);
    fn Z_Malloc(size: c_int, tag: c_int, clear: c_int) -> *mut c_void;
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_sprintf(buffer: *mut c_char, bufsize: c_int, fmt: *const c_char, ...);
    fn Com_Memset(ptr: *mut c_void, c: c_int, size: usize);
    fn Com_Filter(filter: *const c_char, name: *const c_char, casesensitive: c_int) -> c_int;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn Cmd_Argc() -> c_int;
    fn Cmd_AddCommand(name: *const c_char, func: unsafe extern "C" fn());
    fn Info_SetValueForKey(info: *mut c_char, key: *const c_char, value: *const c_char);
    fn Info_SetValueForKey_Big(info: *mut c_char, key: *const c_char, value: *const c_char);
    fn FS_Printf(f: c_int, fmt: *const c_char, ...);
    fn Cvar_Update(vmCvar: *mut vmCvar_t);
}

// Type stubs - these should match the C definitions
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

#[repr(C)]
pub struct vmCvar_t {
    pub handle: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub string: [c_char; 256],
}

const MAX_CVARS: usize = 1224;
const FILE_HASH_SIZE: usize = 256;
const MAX_INFO_STRING: usize = 512;
const BIG_INFO_STRING: usize = 8192;
const MAX_STRING_TOKENS: usize = 1024;
const MAX_CVAR_VALUE_STRING: usize = 256;

// Cvar flag constants (from q_shared.h)
const CVAR_ARCHIVE: c_int = 1;
const CVAR_USERINFO: c_int = 2;
const CVAR_SERVERINFO: c_int = 4;
const CVAR_SYSTEMINFO: c_int = 8;
const CVAR_INIT: c_int = 16;
const CVAR_LATCH: c_int = 32;
const CVAR_ROM: c_int = 64;
const CVAR_USER_CREATED: c_int = 128;
const CVAR_TEMP: c_int = 256;
const CVAR_CHEAT: c_int = 512;
const CVAR_NORESTART: c_int = 1024;
const CVAR_INTERNAL: c_int = 2048;

const ERR_FATAL: c_int = 3;
const ERR_DROP: c_int = 2;

const TAG_SMALL: c_int = 3;

// Memory pool constants
const S_COLOR_WHITE: &[u8] = b"";

// ============================================================================
// Global variables
// ============================================================================

pub static mut cvar_vars: *mut cvar_t = core::ptr::null_mut();
pub static mut cvar_cheats: *mut cvar_t = core::ptr::null_mut();
pub static mut cvar_modifiedFlags: c_int = 0;

pub static mut cvar_indexes: [cvar_t; MAX_CVARS] = [cvar_t {
    name: core::ptr::null_mut(),
    string: core::ptr::null_mut(),
    resetString: core::ptr::null_mut(),
    latchedString: core::ptr::null_mut(),
    flags: 0,
    modified: 0,
    modificationCount: 0,
    value: 0.0,
    integer: 0,
    next: core::ptr::null_mut(),
    hashNext: core::ptr::null_mut(),
}; MAX_CVARS];

pub static mut cvar_numIndexes: c_int = 0;

static mut hashTable: [*mut cvar_t; FILE_HASH_SIZE] = [core::ptr::null_mut(); FILE_HASH_SIZE];

static mut lastMemPool: *mut c_char = core::ptr::null_mut();
static mut memPoolSize: c_int = 0;

// Forward declaration
fn Cvar_Set2(var_name: *const c_char, value: *const c_char, force: c_int) -> *mut cvar_t;

// ============================================================================
// CopyString stub - creates a copy of a string
// ============================================================================
fn CopyString(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return core::ptr::null_mut();
    }

    unsafe {
        let len = libc::strlen(s) as c_int + 1;
        let ptr = Z_Malloc(len, TAG_SMALL, 0) as *mut c_char;
        if !ptr.is_null() {
            libc::strcpy(ptr, s);
        }
        ptr
    }
}

// ============================================================================
// Cvar_FreeString
// ============================================================================

//If the string came from the memory pool, don't really free it.  The entire
//memory pool will be wiped during the next level load.
unsafe fn Cvar_FreeString(string: *mut c_char) {
    if lastMemPool.is_null() || string < lastMemPool || string >= lastMemPool.add(memPoolSize as usize) {
        Z_Free(string as *mut c_void);
    }
}

// ============================================================================
// generateHashValue
// ============================================================================

/*
================
return a hash value for the filename
================
*/
unsafe fn generateHashValue(fname: *const c_char) -> c_int {
    let mut i = 0;
    let mut hash: c_int = 0;

    loop {
        let ch = *fname.add(i) as u8;
        if ch == 0 {
            break;
        }
        let letter = (ch as c_char).to_ascii_lowercase() as u8 as c_int;
        hash += letter * (i as c_int + 119);
        i += 1;
    }
    hash &= (FILE_HASH_SIZE as c_int - 1);
    hash
}

// ============================================================================
// Cvar_ValidateString
// ============================================================================

/*
============
Cvar_ValidateString
============
*/
unsafe fn Cvar_ValidateString(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0; // qfalse
    }
    if !libc::strchr(s, b'\\' as c_int).is_null() {
        return 0; // qfalse
    }
    if !libc::strchr(s, b'"' as c_int).is_null() {
        return 0; // qfalse
    }
    if !libc::strchr(s, b';' as c_int).is_null() {
        return 0; // qfalse
    }
    1 // qtrue
}

// ============================================================================
// Cvar_FindVar
// ============================================================================

/*
============
Cvar_FindVar
============
*/
unsafe fn Cvar_FindVar(var_name: *const c_char) -> *mut cvar_t {
    let hash = generateHashValue(var_name) as usize;

    let mut var = hashTable[hash];
    while !var.is_null() {
        if Q_stricmp(var_name, (*var).name) == 0 {
            return var;
        }
        var = (*var).hashNext;
    }

    core::ptr::null_mut()
}

// ============================================================================
// Cvar_VariableValue
// ============================================================================

/*
============
Cvar_VariableValue
============
*/
pub unsafe fn Cvar_VariableValue(var_name: *const c_char) -> f32 {
    let var = Cvar_FindVar(var_name);

    if var.is_null() {
        return 0.0;
    }
    (*var).value
}

// ============================================================================
// Cvar_VariableIntegerValue
// ============================================================================

/*
============
Cvar_VariableIntegerValue
============
*/
pub unsafe fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int {
    let var = Cvar_FindVar(var_name);

    if var.is_null() {
        return 0;
    }
    (*var).integer
}

// ============================================================================
// Cvar_VariableString
// ============================================================================

/*
============
Cvar_VariableString
============
*/
pub unsafe fn Cvar_VariableString(var_name: *const c_char) -> *mut c_char {
    let var = Cvar_FindVar(var_name);

    if var.is_null() {
        return b"\0".as_ptr() as *mut c_char;
    }
    (*var).string
}

// ============================================================================
// Cvar_VariableStringBuffer
// ============================================================================

/*
============
Cvar_VariableStringBuffer
============
*/
pub unsafe fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int) {
    let var = Cvar_FindVar(var_name);

    if var.is_null() {
        *buffer = 0;
    } else {
        Q_strncpyz(buffer, (*var).string, bufsize);
    }
}

// ============================================================================
// Cvar_CommandCompletion
// ============================================================================

/*
============
Cvar_CommandCompletion
============
*/
pub unsafe fn Cvar_CommandCompletion(callback: unsafe extern "C" fn(*const c_char)) {
    let mut cvar = cvar_vars;

    while !cvar.is_null() {
        // Dont show internal cvars
        if (*cvar).flags & CVAR_INTERNAL == 0 {
            callback((*cvar).name);
        }
        cvar = (*cvar).next;
    }
}

// ============================================================================
// Cvar_Get
// ============================================================================

/*
============
Cvar_Get

If the variable already exists, the value will not be set unless CVAR_ROM
The flags will be or'ed in if the variable exists.
============
*/
pub unsafe fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t {
    let mut var: *mut cvar_t;
    let hash: c_int;

    if var_name.is_null() || var_value.is_null() {
        Com_Error(ERR_FATAL, b"Cvar_Get: NULL parameter\0".as_ptr() as *const c_char);
    }

    if Cvar_ValidateString(var_name) == 0 {
        Com_Printf(b"invalid cvar name string: %s\n\0".as_ptr() as *const c_char, var_name);
        let bad_name = b"BADNAME\0".as_ptr() as *const c_char;
        var = Cvar_FindVar(var_name);
        if var.is_null() {
            // Will create with new name below
        }
    }

    // #if 0 -- commented out in original
    // if ( !Cvar_ValidateString( var_value ) ) {
    //     Com_Printf("invalid cvar value string: %s\n", var_value );
    //     var_value = "BADVALUE";
    // }
    // #endif

    var = Cvar_FindVar(var_name);
    if !var.is_null() {
        // if the C code is now specifying a variable that the user already
        // set a value for, take the new value as the reset value
        if ((*var).flags & CVAR_USER_CREATED != 0) && (flags & CVAR_USER_CREATED == 0) && !(*var_value) == 0 {
            (*var).flags &= !CVAR_USER_CREATED;
            Cvar_FreeString((*var).resetString);
            (*var).resetString = CopyString(var_value);

            // ZOID--needs to be set so that cvars the game sets as
            // SERVERINFO get sent to clients
            cvar_modifiedFlags |= flags;
        }

        (*var).flags |= flags;
        // only allow one non-empty reset string without a warning
        if (*(*var).resetString) == 0 {
            // we don't have a reset string yet
            Cvar_FreeString((*var).resetString);
            (*var).resetString = CopyString(var_value);
        } else if *var_value != 0 && libc::strcmp((*var).resetString, var_value) != 0 {
            Com_DPrintf(
                b"Warning: cvar \"%s\" given initial values: \"%s\" and \"%s\"\n\0".as_ptr() as *const c_char,
                var_name,
                (*var).resetString,
                var_value,
            );
        }
        // if we have a latched string, take that value now
        if !(*var).latchedString.is_null() {
            let s = (*var).latchedString;

            (*var).latchedString = core::ptr::null_mut(); // otherwise cvar_set2 would free it
            Cvar_Set2(var_name, s, 1); // qtrue
            Cvar_FreeString(s);
        }

        // #if 0 -- commented out in original
        // // CVAR_ROM always overrides
        // if ( flags & CVAR_ROM ) {
        //     Cvar_Set2( var_name, var_value, qtrue );
        // }
        // #endif
        return var;
    }

    //
    // allocate a new cvar
    //
    if cvar_numIndexes >= MAX_CVARS as c_int {
        Com_Error(ERR_FATAL, b"MAX_CVARS\0".as_ptr() as *const c_char);
    }
    var = core::ptr::addr_of_mut!(cvar_indexes[cvar_numIndexes as usize]);
    cvar_numIndexes += 1;
    (*var).name = CopyString(var_name);
    (*var).string = CopyString(var_value);
    (*var).modified = 1; // qtrue
    (*var).modificationCount = 1;
    (*var).value = libc::atof((*var).string) as f32;
    (*var).integer = libc::atoi((*var).string);
    (*var).resetString = CopyString(var_value);

    // link the variable in
    (*var).next = cvar_vars;
    cvar_vars = var;

    (*var).flags = flags;

    hash = generateHashValue(var_name);
    (*var).hashNext = hashTable[hash as usize];
    hashTable[hash as usize] = var;

    var
}

// ============================================================================
// Cvar_Set2
// ============================================================================

/*
============
Cvar_Set2
============
*/
fn Cvar_Set2(var_name: *const c_char, value: *const c_char, force: c_int) -> *mut cvar_t {
    unsafe {
        let mut var: *mut cvar_t;

        if Cvar_ValidateString(var_name) == 0 {
            Com_Printf(b"invalid cvar name string: %s\n\0".as_ptr() as *const c_char, var_name);
            // Reassigning var_name is unsafe in Rust, so we skip that part
            // The original C code does this, but we'll handle it differently
        }

        // #if 0	// FIXME
        // if ( value && !Cvar_ValidateString( value ) ) {
        //     Com_Printf("invalid cvar value string: %s\n", value );
        //     var_value = "BADVALUE";
        // }
        // #endif

        var = Cvar_FindVar(var_name);
        if var.is_null() {
            if value.is_null() {
                return core::ptr::null_mut();
            }
            // create it
            if force == 0 {
                return Cvar_Get(var_name, value, CVAR_USER_CREATED);
            } else {
                return Cvar_Get(var_name, value, 0);
            }
        }

        // Dont display the update when its internal
        if (*var).flags & CVAR_INTERNAL == 0 {
            Com_DPrintf(b"Cvar_Set2: %s %s\n\0".as_ptr() as *const c_char, var_name, value);
        }

        let mut val = value;
        if val.is_null() {
            val = (*var).resetString;
        }

        if libc::strcmp(val, (*var).string) == 0 {
            return var;
        }
        // note what types of cvars have been modified (userinfo, archive, serverinfo, systeminfo)
        cvar_modifiedFlags |= (*var).flags;

        if force == 0 {
            if (*var).flags & CVAR_ROM != 0 {
                Com_Printf(b"%s is read only.\n\0".as_ptr() as *const c_char, var_name);
                return var;
            }

            if (*var).flags & CVAR_INIT != 0 {
                Com_Printf(b"%s is write protected.\n\0".as_ptr() as *const c_char, var_name);
                return var;
            }

            if (*var).flags & CVAR_LATCH != 0 {
                if !(*var).latchedString.is_null() {
                    if libc::strcmp(val, (*var).latchedString) == 0 {
                        return var;
                    }
                    Cvar_FreeString((*var).latchedString);
                } else {
                    if libc::strcmp(val, (*var).string) == 0 {
                        return var;
                    }
                }

                Com_Printf(b"%s will be changed upon restarting.\n\0".as_ptr() as *const c_char, var_name);
                (*var).latchedString = CopyString(val);
                (*var).modified = 1; // qtrue
                (*var).modificationCount += 1;
                return var;
            }

            if ((*var).flags & CVAR_CHEAT != 0) && (*cvar_cheats).integer == 0 {
                Com_Printf(b"%s is cheat protected.\n\0".as_ptr() as *const c_char, var_name);
                return var;
            }
        } else {
            if !(*var).latchedString.is_null() {
                Cvar_FreeString((*var).latchedString);
                (*var).latchedString = core::ptr::null_mut();
            }
        }

        if libc::strcmp(val, (*var).string) == 0 {
            return var; // not changed
        }

        (*var).modified = 1; // qtrue
        (*var).modificationCount += 1;

        Cvar_FreeString((*var).string); // free the old value string

        (*var).string = CopyString(val);
        (*var).value = libc::atof((*var).string) as f32;
        (*var).integer = libc::atoi((*var).string);

        var
    }
}

// ============================================================================
// Cvar_Set
// ============================================================================

/*
============
Cvar_Set
============
*/
pub unsafe fn Cvar_Set(var_name: *const c_char, value: *const c_char) {
    Cvar_Set2(var_name, value, 1); // qtrue
}

// ============================================================================
// Cvar_SetLatched
// ============================================================================

/*
============
Cvar_SetLatched
============
*/
pub unsafe fn Cvar_SetLatched(var_name: *const c_char, value: *const c_char) {
    Cvar_Set2(var_name, value, 0); // qfalse
}

// ============================================================================
// Cvar_SetValue
// ============================================================================

/*
============
Cvar_SetValue
============
*/
pub unsafe fn Cvar_SetValue(var_name: *const c_char, value: f32) {
    let mut val: [c_char; 32] = [0; 32];

    if value == value as c_int as f32 {
        Com_sprintf(
            val.as_mut_ptr(),
            32,
            b"%i\0".as_ptr() as *const c_char,
            value as c_int,
        );
    } else {
        Com_sprintf(val.as_mut_ptr(), 32, b"%f\0".as_ptr() as *const c_char, value);
    }
    Cvar_Set(var_name, val.as_ptr());
}

// ============================================================================
// Cvar_Reset
// ============================================================================

/*
============
Cvar_Reset
============
*/
pub unsafe fn Cvar_Reset(var_name: *const c_char) {
    Cvar_Set2(var_name, core::ptr::null(), 0); // qfalse
}

// ============================================================================
// Cvar_SetCheatState
// ============================================================================

/*
============
Cvar_SetCheatState

Any testing variables will be reset to the safe values
============
*/
pub unsafe fn Cvar_SetCheatState() {
    let mut var: *mut cvar_t;

    // set all default vars to the safe value
    var = cvar_vars;
    while !var.is_null() {
        if (*var).flags & CVAR_CHEAT != 0 {
            // the CVAR_LATCHED|CVAR_CHEAT vars might escape the reset here
            // because of a different var->latchedString
            if !(*var).latchedString.is_null() {
                Cvar_FreeString((*var).latchedString);
                (*var).latchedString = core::ptr::null_mut();
            }
            if libc::strcmp((*var).resetString, (*var).string) != 0 {
                Cvar_Set((*var).name, (*var).resetString);
            }
        }
        var = (*var).next;
    }
}

// ============================================================================
// Cvar_Command
// ============================================================================

/*
============
Cvar_Command

Handles variable inspection and changing from the console
============
*/
pub unsafe fn Cvar_Command() -> c_int {
    let v: *mut cvar_t;

    // check variables
    v = Cvar_FindVar(Cmd_Argv(0));
    if v.is_null() {
        return 0; // qfalse
    }

    // perform a variable print or set
    if Cmd_Argc() == 1 {
        /*		if (v->flags & CVAR_INTERNAL) // don't display
            {
                return qtrue;
            }
        */
        Com_Printf(
            b"\"%s\" is:\"%s\x19\" default:\"%s\x19\"\n\0".as_ptr() as *const c_char,
            (*v).name,
            (*v).string,
            (*v).resetString,
        );
        if !(*v).latchedString.is_null() {
            Com_Printf(b"latched: \"%s\"\n\0".as_ptr() as *const c_char, (*v).latchedString);
        }
        return 1; // qtrue
    }

    //JFM toggle test
    let value: *const c_char;
    value = Cmd_Argv(1);
    if *value as u8 == b'!' {
        //toggle
        let mut buff: [c_char; 5] = [0; 5];
        libc::sprintf(buff.as_mut_ptr(), b"%i\0".as_ptr() as *const c_char, if (*v).value != 0.0 { 0 } else { 1 });
        Cvar_Set2((*v).name, buff.as_ptr(), 0); // qfalse - toggle the value
    } else {
        Cvar_Set2((*v).name, value, 0); // qfalse - set the value if forcing isn't required
    }

    1 // qtrue
}

// ============================================================================
// Cvar_Toggle_f
// ============================================================================

/*
============
Cvar_Toggle_f

Toggles a cvar for easy single key binding
============
*/
pub unsafe extern "C" fn Cvar_Toggle_f() {
    let mut v: c_int;

    if Cmd_Argc() != 2 {
        Com_Printf(b"usage: toggle <variable>\n\0".as_ptr() as *const c_char);
        return;
    }

    v = Cvar_VariableValue(Cmd_Argv(1)) as c_int;
    v = if v != 0 { 0 } else { 1 };

    let mut buf: [c_char; 32] = [0; 32];
    libc::sprintf(buf.as_mut_ptr(), b"%i\0".as_ptr() as *const c_char, v);
    Cvar_Set2(Cmd_Argv(1), buf.as_ptr(), 0); // qfalse
}

// ============================================================================
// Cvar_Set_f
// ============================================================================

/*
============
Cvar_Set_f

Allows setting and defining of arbitrary cvars from console, even if they
weren't declared in C code.
============
*/
pub unsafe extern "C" fn Cvar_Set_f() {
    let mut i: c_int;
    let mut c: c_int;
    let mut l: c_int;
    let mut len: c_int;
    let mut combined: [c_char; 1024] = [0; 1024];

    c = Cmd_Argc();
    if c < 3 {
        Com_Printf(b"usage: set <variable> <value>\n\0".as_ptr() as *const c_char);
        return;
    }

    combined[0] = 0;
    l = 0;
    i = 2;
    while i < c {
        len = libc::strlen(Cmd_Argv(i)) as c_int + 1;
        if l + len >= MAX_STRING_TOKENS as c_int - 2 {
            break;
        }
        libc::strcat(combined.as_mut_ptr(), Cmd_Argv(i));
        if i != c - 1 {
            libc::strcat(combined.as_mut_ptr(), b" \0".as_ptr() as *const c_char);
        }
        l += len;
        i += 1;
    }
    Cvar_Set2(Cmd_Argv(1), combined.as_ptr(), 0); // qfalse
}

// ============================================================================
// Cvar_SetU_f
// ============================================================================

/*
============
Cvar_SetU_f

As Cvar_Set, but also flags it as userinfo
============
*/
pub unsafe extern "C" fn Cvar_SetU_f() {
    let v: *mut cvar_t;

    if Cmd_Argc() != 3 {
        Com_Printf(b"usage: setu <variable> <value>\n\0".as_ptr() as *const c_char);
        return;
    }
    Cvar_Set_f();
    v = Cvar_FindVar(Cmd_Argv(1));
    if v.is_null() {
        return;
    }
    (*v).flags |= CVAR_USERINFO;
}

// ============================================================================
// Cvar_SetS_f
// ============================================================================

/*
============
Cvar_SetS_f

As Cvar_Set, but also flags it as serverinfo
============
*/
pub unsafe extern "C" fn Cvar_SetS_f() {
    let v: *mut cvar_t;

    if Cmd_Argc() != 3 {
        Com_Printf(b"usage: sets <variable> <value>\n\0".as_ptr() as *const c_char);
        return;
    }
    Cvar_Set_f();
    v = Cvar_FindVar(Cmd_Argv(1));
    if v.is_null() {
        return;
    }
    (*v).flags |= CVAR_SERVERINFO;
}

// ============================================================================
// Cvar_SetA_f
// ============================================================================

/*
============
Cvar_SetA_f

As Cvar_Set, but also flags it as archived
============
*/
pub unsafe extern "C" fn Cvar_SetA_f() {
    let v: *mut cvar_t;

    if Cmd_Argc() != 3 {
        Com_Printf(b"usage: seta <variable> <value>\n\0".as_ptr() as *const c_char);
        return;
    }
    Cvar_Set_f();
    v = Cvar_FindVar(Cmd_Argv(1));
    if v.is_null() {
        return;
    }
    (*v).flags |= CVAR_ARCHIVE;
}

// ============================================================================
// Cvar_Reset_f
// ============================================================================

/*
============
Cvar_Reset_f
============
*/
pub unsafe extern "C" fn Cvar_Reset_f() {
    if Cmd_Argc() != 2 {
        Com_Printf(b"usage: reset <variable>\n\0".as_ptr() as *const c_char);
        return;
    }
    Cvar_Reset(Cmd_Argv(1));
}

// ============================================================================
// Cvar_WriteVariables
// ============================================================================

/*
============
Cvar_WriteVariables

Appends lines containing "set variable value" for all variables
with the archive flag set to qtrue.
============
*/
pub unsafe fn Cvar_WriteVariables(f: c_int) {
    let mut var: *mut cvar_t;
    let mut buffer: [c_char; 1024] = [0; 1024];

    var = cvar_vars;
    while !var.is_null() {
        // #ifdef USE_CD_KEY
        // if( Q_stricmp( var->name, "cl_cdkey" ) == 0 ) {
        //     continue;
        // }
        // #endif // USE_CD_KEY

        if (*var).flags & CVAR_ARCHIVE != 0 {
            // write the latched value, even if it hasn't taken effect yet
            if !(*var).latchedString.is_null() {
                Com_sprintf(
                    buffer.as_mut_ptr(),
                    1024,
                    b"seta %s \"%s\"\n\0".as_ptr() as *const c_char,
                    (*var).name,
                    (*var).latchedString,
                );
            } else {
                Com_sprintf(
                    buffer.as_mut_ptr(),
                    1024,
                    b"seta %s \"%s\"\n\0".as_ptr() as *const c_char,
                    (*var).name,
                    (*var).string,
                );
            }
            FS_Printf(f, b"%s\0".as_ptr() as *const c_char, buffer.as_ptr());
        }
        var = (*var).next;
    }
}

// ============================================================================
// Cvar_List_f
// ============================================================================

/*
============
Cvar_List_f
============
*/
pub unsafe extern "C" fn Cvar_List_f() {
    let mut var: *mut cvar_t;
    let mut i: c_int;
    let match_str: *const c_char;

    if Cmd_Argc() > 1 {
        match_str = Cmd_Argv(1);
    } else {
        match_str = core::ptr::null();
    }

    i = 0;
    var = cvar_vars;
    while !var.is_null() {
        // Dont show internal cvars
        if (*var).flags & CVAR_INTERNAL != 0 {
            var = (*var).next;
            continue;
        }

        if !match_str.is_null() && Com_Filter(match_str, (*var).name, 0) == 0 {
            var = (*var).next;
            continue;
        }

        if (*var).flags & CVAR_SERVERINFO != 0 {
            Com_Printf(b"S\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b" \0".as_ptr() as *const c_char);
        }
        if (*var).flags & CVAR_USERINFO != 0 {
            Com_Printf(b"U\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b" \0".as_ptr() as *const c_char);
        }
        if (*var).flags & CVAR_ROM != 0 {
            Com_Printf(b"R\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b" \0".as_ptr() as *const c_char);
        }
        if (*var).flags & CVAR_INIT != 0 {
            Com_Printf(b"I\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b" \0".as_ptr() as *const c_char);
        }
        if (*var).flags & CVAR_ARCHIVE != 0 {
            Com_Printf(b"A\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b" \0".as_ptr() as *const c_char);
        }
        if (*var).flags & CVAR_LATCH != 0 {
            Com_Printf(b"L\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b" \0".as_ptr() as *const c_char);
        }
        if (*var).flags & CVAR_CHEAT != 0 {
            Com_Printf(b"C\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b" \0".as_ptr() as *const c_char);
        }

        Com_Printf(b" %s \"%s\"\n\0".as_ptr() as *const c_char, (*var).name, (*var).string);

        var = (*var).next;
        i += 1;
    }

    Com_Printf(b"\n%i total cvars\n\0".as_ptr() as *const c_char, i);
    Com_Printf(b"%i cvar indexes\n\0".as_ptr() as *const c_char, cvar_numIndexes);
}

// ============================================================================
// Cvar_Restart_f
// ============================================================================

/*
============
Cvar_Restart_f

Resets all cvars to their hardcoded values
============
*/
pub unsafe extern "C" fn Cvar_Restart_f() {
    let mut var: *mut cvar_t;
    let mut prev: *mut *mut cvar_t;

    prev = core::ptr::addr_of_mut!(cvar_vars);
    loop {
        var = *prev;
        if var.is_null() {
            break;
        }

        // don't mess with rom values, or some inter-module
        // communication will get broken (com_cl_running, etc)
        if (*var).flags & (CVAR_ROM | CVAR_INIT | CVAR_NORESTART) != 0 {
            prev = core::ptr::addr_of_mut!((*var).next);
            continue;
        }

        // throw out any variables the user created
        if (*var).flags & CVAR_USER_CREATED != 0 {
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
            Com_Memset(var as *mut c_void, 0, core::mem::size_of::<cvar_t>());
            continue;
        }

        Cvar_Set((*var).name, (*var).resetString);

        prev = core::ptr::addr_of_mut!((*var).next);
    }
}

// ============================================================================
// Cvar_InfoString
// ============================================================================

/*
=====================
Cvar_InfoString
=====================
*/
pub unsafe fn Cvar_InfoString(bit: c_int) -> *mut c_char {
    static mut info: [c_char; 512] = [0; 512];
    let mut var: *mut cvar_t;

    info[0] = 0;

    var = cvar_vars;
    while !var.is_null() {
        if ((*var).flags & CVAR_INTERNAL) == 0 && ((*var).flags & bit) != 0 {
            Info_SetValueForKey(info.as_mut_ptr(), (*var).name, (*var).string);
        }
        var = (*var).next;
    }

    /*
    for (var = cvar_vars ; var ; var = var->next)
    {
        if ((var->flags & CVAR_INTERNAL) &&
            (var->flags & bit) &&
            !Q_stricmp(var->name, "g_debugMelee"))
        { //this one must go first
            Info_SetValueForKey (info, var->name, var->string);
            kungFuSafety = true;
            break;
        }
    }
    if (!kungFuSafety)
    { //even if it was not found, it must be in the info string
        Info_SetValueForKey (info, "g_debugMelee", "1");
    }
    */

    info.as_mut_ptr()
}

// ============================================================================
// Cvar_InfoString_Big
// ============================================================================

/*
=====================
Cvar_InfoString_Big

  handles large info strings ( CS_SYSTEMINFO )
=====================
*/
pub unsafe fn Cvar_InfoString_Big(bit: c_int) -> *mut c_char {
    static mut info: [c_char; 8192] = [0; 8192];
    let mut var: *mut cvar_t;

    info[0] = 0;

    var = cvar_vars;
    while !var.is_null() {
        if ((*var).flags & CVAR_INTERNAL) == 0 && ((*var).flags & bit) != 0 {
            Info_SetValueForKey_Big(info.as_mut_ptr(), (*var).name, (*var).string);
        }
        var = (*var).next;
    }
    info.as_mut_ptr()
}

// ============================================================================
// Cvar_InfoStringBuffer
// ============================================================================

/*
=====================
Cvar_InfoStringBuffer
=====================
*/
pub unsafe fn Cvar_InfoStringBuffer(bit: c_int, buff: *mut c_char, buffsize: c_int) {
    Q_strncpyz(buff, Cvar_InfoString(bit), buffsize);
}

// ============================================================================
// Cvar_Register
// ============================================================================

/*
=====================
Cvar_Register

basically a slightly modified Cvar_Get for the interpreted modules
=====================
*/
pub unsafe fn Cvar_Register(vmCvar: *mut vmCvar_t, varName: *const c_char, defaultValue: *const c_char, flags: c_int) {
    let cv: *mut cvar_t;

    cv = Cvar_Get(varName, defaultValue, flags);
    if vmCvar.is_null() {
        return;
    }
    (*vmCvar).handle = (cv as usize - core::ptr::addr_of!(cvar_indexes[0]) as usize) as c_int / core::mem::size_of::<cvar_t>() as c_int;
    (*vmCvar).modificationCount = -1;
    Cvar_Update(vmCvar);
}

// ============================================================================
// Cvar_Update
// ============================================================================

/*
=====================
Cvar_Update

updates an interpreted modules' version of a cvar
=====================
*/
pub unsafe fn Cvar_Update(vmCvar: *mut vmCvar_t) {
    let mut cv: *mut cvar_t = core::ptr::null_mut(); // bk001129

    debug_assert!(!vmCvar.is_null()); // bk

    if ((*vmCvar).handle as c_int) as u32 >= cvar_numIndexes as u32 {
        Com_Error(ERR_DROP, b"Cvar_Update: handle out of range\0".as_ptr() as *const c_char);
    }

    cv = core::ptr::addr_of_mut!(cvar_indexes[(*vmCvar).handle as usize]);

    if (*cv).modificationCount == (*vmCvar).modificationCount {
        return;
    }
    if (*cv).string.is_null() {
        return; // variable might have been cleared by a cvar_restart
    }
    (*vmCvar).modificationCount = (*cv).modificationCount;
    // bk001129 - mismatches.
    if libc::strlen((*cv).string) as c_int + 1 > MAX_CVAR_VALUE_STRING as c_int {
        Com_Error(
            ERR_DROP,
            b"Cvar_Update: src %s length %d exceeds MAX_CVAR_VALUE_STRING\0".as_ptr() as *const c_char,
            (*cv).string,
            libc::strlen((*cv).string) as c_int,
        );
    }
    // bk001212 - Q_strncpyz guarantees zero padding and dest[MAX_CVAR_VALUE_STRING-1]==0
    // bk001129 - paranoia. Never trust the destination string.
    // bk001129 - beware, sizeof(char*) is always 4 (for cv->string).
    //            sizeof(vmCvar->string) always MAX_CVAR_VALUE_STRING
    //Q_strncpyz( vmCvar->string, cv->string, sizeof( vmCvar->string ) ); // id
    Q_strncpyz((*vmCvar).string.as_mut_ptr(), (*cv).string, MAX_CVAR_VALUE_STRING as c_int);

    (*vmCvar).value = (*cv).value;
    (*vmCvar).integer = (*cv).integer;
}

// ============================================================================
// Cvar_Init
// ============================================================================

/*
============
Cvar_Init

Reads in all archived cvars
============
*/
pub unsafe fn Cvar_Init() {
    cvar_cheats = Cvar_Get(b"sv_cheats\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ROM | CVAR_SYSTEMINFO);

    Cmd_AddCommand(b"toggle\0".as_ptr() as *const c_char, Cvar_Toggle_f);
    Cmd_AddCommand(b"set\0".as_ptr() as *const c_char, Cvar_Set_f);
    Cmd_AddCommand(b"sets\0".as_ptr() as *const c_char, Cvar_SetS_f);
    Cmd_AddCommand(b"setu\0".as_ptr() as *const c_char, Cvar_SetU_f);
    Cmd_AddCommand(b"seta\0".as_ptr() as *const c_char, Cvar_SetA_f);
    Cmd_AddCommand(b"reset\0".as_ptr() as *const c_char, Cvar_Reset_f);
    Cmd_AddCommand(b"cvarlist\0".as_ptr() as *const c_char, Cvar_List_f);
    Cmd_AddCommand(b"cvar_restart\0".as_ptr() as *const c_char, Cvar_Restart_f);
}

// ============================================================================
// Cvar_Realloc
// ============================================================================

unsafe fn Cvar_Realloc(string: *mut *mut c_char, memPool: *mut c_char, memPoolUsed: &mut c_int) {
    if !(*string).is_null() {
        let temp: *mut c_char = memPool.add(*memPoolUsed as usize);
        libc::strcpy(temp, *string);
        *memPoolUsed += libc::strlen(*string) as c_int + 1;
        Cvar_FreeString(*string);
        *string = temp;
    }
}

// ============================================================================
// Cvar_Defrag
// ============================================================================

//Turns many small allocation blocks into one big one.
pub unsafe fn Cvar_Defrag() {
    let mut var: *mut cvar_t;
    let mut totalMem: c_int = 0;
    let nextMemPoolSize: c_int;

    var = cvar_vars;
    while !var.is_null() {
        if !(*var).name.is_null() {
            totalMem += libc::strlen((*var).name) as c_int + 1;
        }
        if !(*var).string.is_null() {
            totalMem += libc::strlen((*var).string) as c_int + 1;
        }
        if !(*var).resetString.is_null() {
            totalMem += libc::strlen((*var).resetString) as c_int + 1;
        }
        if !(*var).latchedString.is_null() {
            totalMem += libc::strlen((*var).latchedString) as c_int + 1;
        }
        var = (*var).next;
    }

    let mem: *mut c_char = Z_Malloc(totalMem, TAG_SMALL, 0) as *mut c_char;
    nextMemPoolSize = totalMem;
    totalMem = 0;

    var = cvar_vars;
    while !var.is_null() {
        Cvar_Realloc(core::ptr::addr_of_mut!((*var).name), mem, &mut totalMem);
        Cvar_Realloc(core::ptr::addr_of_mut!((*var).string), mem, &mut totalMem);
        Cvar_Realloc(core::ptr::addr_of_mut!((*var).resetString), mem, &mut totalMem);
        Cvar_Realloc(core::ptr::addr_of_mut!((*var).latchedString), mem, &mut totalMem);
        var = (*var).next;
    }

    if !lastMemPool.is_null() {
        Z_Free(lastMemPool as *mut c_void);
    }
    lastMemPool = mem;
    memPoolSize = nextMemPoolSize;
}

// ============================================================================
// libc stubs for C standard library functions used in this module
// ============================================================================

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn atof(s: *const c_char) -> f64;
    fn atoi(s: *const c_char) -> c_int;
    fn sprintf(s: *mut c_char, fmt: *const c_char, ...) -> c_int;
}

mod libc {
    use core::ffi::c_char;

    pub use super::{atof, atoi, strcat, strchr, strcpy, strcmp, strlen, sprintf};
}
