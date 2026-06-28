//
// name:		be_ai_char.c
//
// desc:		bot characters
//
// $Archive: /MissionPack/code/botlib/be_ai_char.c $
// $Author: Ttimo $
// $Revision: 6 $
// $Modtime: 4/22/01 8:52a $
// $Date: 4/22/01 8:52a $
//

use core::ffi::{c_int, c_char, c_void};
use core::mem;

const MAX_CHARACTERISTICS: c_int = 80;

const CT_INTEGER: c_char = 1;
const CT_FLOAT: c_char = 2;
const CT_STRING: c_char = 3;

const DEFAULT_CHARACTER: &[u8] = b"bots/default_c.c";

//characteristic value
#[repr(C)]
union cvalue {
    integer: c_int,
    _float: f32,
    string: *mut c_char,
}

//a characteristic
#[repr(C)]
struct bot_characteristic_s {
    type_: c_char, //characteristic type
    value: cvalue, //characteristic value
}
pub type bot_characteristic_t = bot_characteristic_s;

//a bot character
#[repr(C)]
struct bot_character_s {
    filename: [c_char; 260usize], // MAX_QPATH
    skill: f32,
    c: [bot_characteristic_t; 1], //variable sized
}
pub type bot_character_t = bot_character_s;

pub static mut botcharacters: [*mut bot_character_t; 65usize] = [core::ptr::null_mut(); 65]; // MAX_CLIENTS + 1

// External declarations for engine functions and imports

extern "C" {
    static mut botimport: BotImportStruct;
    fn PC_SetBaseFolder(folder: *const c_char);
    fn LoadSourceFile(filename: *const c_char) -> *mut source_t;
    fn FreeSource(source: *mut source_t);
    fn PC_ReadToken(source: *mut source_t, token: *mut token_t) -> c_int;
    fn PC_ExpectTokenType(source: *mut source_t, ttype: c_int, subtype: c_int, token: *mut token_t) -> c_int;
    fn PC_ExpectTokenString(source: *mut source_t, string: *const c_char) -> c_int;
    fn PC_ExpectAnyToken(source: *mut source_t, token: *mut token_t) -> c_int;
    fn StripDoubleQuotes(string: *mut c_char);
    fn SourceError(source: *mut source_t, format: *const c_char, ...);
    fn GetClearedMemory(size: usize) -> *mut c_void;
    fn GetMemory(size: usize) -> *mut c_void;
    fn FreeMemory(ptr: *mut c_void);
    fn LibVarGetValue(varname: *const c_char) -> c_int;
    fn Sys_MilliSeconds() -> c_int;
    fn Log_Write(format: *const c_char, ...);
}

// Stub types for external dependencies
#[repr(C)]
struct BotImportStruct {
    // Fields not used in this file
}

#[repr(C)]
struct source_t {
    // Opaque type
}

#[repr(C)]
struct token_t {
    string: [c_char; 1024usize],
    type_: c_int,
    subtype: c_int,
    intvalue: c_int,
    floatvalue: f32,
}

// Constants
const qfalse: c_int = 0;
const qtrue: c_int = 1;

//========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//========================================================================
unsafe fn BotCharacterFromHandle(handle: c_int) -> *mut bot_character_t {
    if handle <= 0 || handle > 64 {
        // MAX_CLIENTS = 64
        (*core::ptr::addr_of_mut!(botimport)).Print(2, // PRT_FATAL
            b"character handle %d out of range\n\0".as_ptr() as *const c_char, handle);
        return core::ptr::null_mut();
    } //end if
    if botcharacters[handle as usize].is_null() {
        (*core::ptr::addr_of_mut!(botimport)).Print(2, // PRT_FATAL
            b"invalid character %d\n\0".as_ptr() as *const c_char, handle);
        return core::ptr::null_mut();
    } //end if
    return botcharacters[handle as usize];
} //end of the function BotCharacterFromHandle

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotDumpCharacter(ch: *mut bot_character_t) {
    let mut i: c_int;

    Log_Write(b"%s\0".as_ptr() as *const c_char, (*ch).filename.as_ptr());
    Log_Write(b"skill %d\n\0".as_ptr() as *const c_char, (*ch).skill as c_int);
    Log_Write(b"{\n\0".as_ptr() as *const c_char);
    i = 0;
    while i < MAX_CHARACTERISTICS {
        let c_ptr = (*ch).c.as_ptr().add(i as usize);
        match (*c_ptr).type_ {
            CT_INTEGER => {
                Log_Write(b" %4d %d\n\0".as_ptr() as *const c_char, i, (*c_ptr).value.integer);
            }
            CT_FLOAT => {
                Log_Write(b" %4d %f\n\0".as_ptr() as *const c_char, i, (*c_ptr).value._float);
            }
            CT_STRING => {
                Log_Write(b" %4d %s\n\0".as_ptr() as *const c_char, i, (*c_ptr).value.string);
            }
            _ => {}
        } //end case
        i += 1;
    } //end for
    Log_Write(b"}\n\0".as_ptr() as *const c_char);
} //end of the function BotDumpCharacter

//========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//========================================================================
unsafe fn BotFreeCharacterStrings(ch: *mut bot_character_t) {
    let mut i: c_int;

    i = 0;
    while i < MAX_CHARACTERISTICS {
        let c_ptr = (*ch).c.as_ptr().add(i as usize);
        if (*c_ptr).type_ == CT_STRING {
            FreeMemory((*c_ptr).value.string as *mut c_void);
        } //end if
        i += 1;
    } //end for
} //end of the function BotFreeCharacterStrings

//========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//========================================================================
unsafe fn BotFreeCharacter2(handle: c_int) {
    if handle <= 0 || handle > 64 {
        // MAX_CLIENTS = 64
        (*core::ptr::addr_of_mut!(botimport)).Print(2, // PRT_FATAL
            b"character handle %d out of range\n\0".as_ptr() as *const c_char, handle);
        return;
    } //end if
    if botcharacters[handle as usize].is_null() {
        (*core::ptr::addr_of_mut!(botimport)).Print(2, // PRT_FATAL
            b"invalid character %d\n\0".as_ptr() as *const c_char, handle);
        return;
    } //end if
    BotFreeCharacterStrings(botcharacters[handle as usize]);
    FreeMemory(botcharacters[handle as usize] as *mut c_void);
    botcharacters[handle as usize] = core::ptr::null_mut();
} //end of the function BotFreeCharacter2

//========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//========================================================================
unsafe fn BotFreeCharacter(handle: c_int) {
    if LibVarGetValue(b"bot_reloadcharacters\0".as_ptr() as *const c_char) == 0 { return; }
    BotFreeCharacter2(handle);
} //end of the function BotFreeCharacter

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotDefaultCharacteristics(ch: *mut bot_character_t, defaultch: *mut bot_character_t) {
    let mut i: c_int;

    i = 0;
    while i < MAX_CHARACTERISTICS {
        let c_ptr = (*ch).c.as_ptr().add(i as usize) as *mut bot_characteristic_t;
        let default_c_ptr = (*defaultch).c.as_ptr().add(i as usize);
        if (*c_ptr).type_ != 0 {
            i += 1;
            continue;
        }
        //
        if (*default_c_ptr).type_ == CT_FLOAT {
            (*c_ptr).type_ = CT_FLOAT;
            (*c_ptr).value._float = (*default_c_ptr).value._float;
        } //end if
        else if (*default_c_ptr).type_ == CT_INTEGER {
            (*c_ptr).type_ = CT_INTEGER;
            (*c_ptr).value.integer = (*default_c_ptr).value.integer;
        } //end else if
        else if (*default_c_ptr).type_ == CT_STRING {
            (*c_ptr).type_ = CT_STRING;
            let len = libc_strlen((*default_c_ptr).value.string) + 1;
            (*c_ptr).value.string = GetMemory(len) as *mut c_char;
            libc_strcpy((*c_ptr).value.string, (*default_c_ptr).value.string);
        } //end else if
        i += 1;
    } //end for
} //end of the function BotDefaultCharacteristics

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotLoadCharacterFromFile(charfile: *mut c_char, skill: c_int) -> *mut bot_character_t {
    let mut indent: c_int;
    let mut index: c_int;
    let mut foundcharacter: c_int;
    let mut ch: *mut bot_character_t;
    let mut source: *mut source_t;
    let mut token: token_t = mem::zeroed();

    foundcharacter = qfalse;
    //a bot character is parsed in two phases
    PC_SetBaseFolder(b"botfiles\0".as_ptr() as *const c_char); // BOTFILESBASEFOLDER
    source = LoadSourceFile(charfile);
    if source.is_null() {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, // PRT_ERROR
            b"counldn\'t load %s\n\0".as_ptr() as *const c_char, charfile);
        return core::ptr::null_mut();
    } //end if
    ch = GetClearedMemory(mem::size_of::<bot_character_t>() +
                    MAX_CHARACTERISTICS as usize * mem::size_of::<bot_characteristic_t>()) as *mut bot_character_t;
    libc_strcpy((*ch).filename.as_mut_ptr(), charfile);
    while PC_ReadToken(source, &mut token) != 0 {
        if c_strcmp(token.string.as_ptr(), b"skill\0".as_ptr() as *const c_char) == 0 {
            if PC_ExpectTokenType(source, 1, 0, &mut token) == 0 { // TT_NUMBER
                FreeSource(source);
                BotFreeCharacterStrings(ch);
                FreeMemory(ch as *mut c_void);
                return core::ptr::null_mut();
            } //end if
            if PC_ExpectTokenString(source, b"{\0".as_ptr() as *const c_char) == 0 {
                FreeSource(source);
                BotFreeCharacterStrings(ch);
                FreeMemory(ch as *mut c_void);
                return core::ptr::null_mut();
            } //end if
            //if it's the correct skill
            if skill < 0 || token.intvalue == skill {
                foundcharacter = qtrue;
                (*ch).skill = token.intvalue as f32;
                while PC_ExpectAnyToken(source, &mut token) != 0 {
                    if c_strcmp(token.string.as_ptr(), b"}\0".as_ptr() as *const c_char) == 0 { break; }
                    if token.type_ != 1 || (token.subtype & 2) == 0 { // TT_NUMBER, TT_INTEGER
                        SourceError(source, b"expected integer index, found %s\n\0".as_ptr() as *const c_char, token.string.as_ptr());
                        FreeSource(source);
                        BotFreeCharacterStrings(ch);
                        FreeMemory(ch as *mut c_void);
                        return core::ptr::null_mut();
                    } //end if
                    index = token.intvalue;
                    if index < 0 || index > MAX_CHARACTERISTICS {
                        SourceError(source, b"characteristic index out of range [0, %d]\n\0".as_ptr() as *const c_char, MAX_CHARACTERISTICS);
                        FreeSource(source);
                        BotFreeCharacterStrings(ch);
                        FreeMemory(ch as *mut c_void);
                        return core::ptr::null_mut();
                    } //end if
                    let c_ptr = (*ch).c.as_ptr().add(index as usize) as *mut bot_characteristic_t;
                    if (*c_ptr).type_ != 0 {
                        SourceError(source, b"characteristic %d already initialized\n\0".as_ptr() as *const c_char, index);
                        FreeSource(source);
                        BotFreeCharacterStrings(ch);
                        FreeMemory(ch as *mut c_void);
                        return core::ptr::null_mut();
                    } //end if
                    if PC_ExpectAnyToken(source, &mut token) == 0 {
                        FreeSource(source);
                        BotFreeCharacterStrings(ch);
                        FreeMemory(ch as *mut c_void);
                        return core::ptr::null_mut();
                    } //end if
                    if token.type_ == 1 { // TT_NUMBER
                        if (token.subtype & 1) != 0 { // TT_FLOAT
                            (*c_ptr).value._float = token.floatvalue;
                            (*c_ptr).type_ = CT_FLOAT;
                        } //end if
                        else {
                            (*c_ptr).value.integer = token.intvalue;
                            (*c_ptr).type_ = CT_INTEGER;
                        } //end else
                    } //end if
                    else if token.type_ == 2 { // TT_STRING
                        StripDoubleQuotes(token.string.as_mut_ptr());
                        let len = libc_strlen(token.string.as_ptr()) + 1;
                        (*c_ptr).value.string = GetMemory(len) as *mut c_char;
                        libc_strcpy((*c_ptr).value.string, token.string.as_ptr());
                        (*c_ptr).type_ = CT_STRING;
                    } //end else if
                    else {
                        SourceError(source, b"expected integer, float or string, found %s\n\0".as_ptr() as *const c_char, token.string.as_ptr());
                        FreeSource(source);
                        BotFreeCharacterStrings(ch);
                        FreeMemory(ch as *mut c_void);
                        return core::ptr::null_mut();
                    } //end else
                } //end if
                break;
            } //end if
            else {
                indent = 1;
                while indent != 0 {
                    if PC_ExpectAnyToken(source, &mut token) == 0 {
                        FreeSource(source);
                        BotFreeCharacterStrings(ch);
                        FreeMemory(ch as *mut c_void);
                        return core::ptr::null_mut();
                    } //end if
                    if c_strcmp(token.string.as_ptr(), b"{\0".as_ptr() as *const c_char) == 0 { indent += 1; }
                    else if c_strcmp(token.string.as_ptr(), b"}\0".as_ptr() as *const c_char) == 0 { indent -= 1; }
                } //end while
            } //end else
        } //end if
        else {
            SourceError(source, b"unknown definition %s\n\0".as_ptr() as *const c_char, token.string.as_ptr());
            FreeSource(source);
            BotFreeCharacterStrings(ch);
            FreeMemory(ch as *mut c_void);
            return core::ptr::null_mut();
        } //end else
    } //end while
    FreeSource(source);
    //
    if foundcharacter == 0 {
        BotFreeCharacterStrings(ch);
        FreeMemory(ch as *mut c_void);
        return core::ptr::null_mut();
    } //end if
    return ch;
} //end of the function BotLoadCharacterFromFile

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotFindCachedCharacter(charfile: *mut c_char, skill: f32) -> c_int {
    let mut handle: c_int;

    handle = 1;
    while handle <= 64 { // MAX_CLIENTS
        if botcharacters[handle as usize].is_null() {
            handle += 1;
            continue;
        }
        if c_strcmp((*botcharacters[handle as usize]).filename.as_ptr(), charfile) == 0 &&
            (skill < 0.0 || ((*botcharacters[handle as usize]).skill - skill).abs() < 0.01) {
            return handle;
        } //end if
        handle += 1;
    } //end for
    return 0;
} //end of the function BotFindCachedCharacter

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotLoadCachedCharacter(charfile: *mut c_char, skill: f32, reload: c_int) -> c_int {
    let mut handle: c_int;
    let mut cachedhandle: c_int;
    let mut intskill: c_int;
    let mut ch: *mut bot_character_t = core::ptr::null_mut();
    #[cfg(debug_assertions)]
    let mut starttime: c_int;

    #[cfg(debug_assertions)]
    {
        starttime = Sys_MilliSeconds();
    }

    //find a free spot for a character
    handle = 1;
    while handle <= 64 { // MAX_CLIENTS
        if botcharacters[handle as usize].is_null() { break; }
        handle += 1;
    } //end for
    if handle > 64 { return 0; } // MAX_CLIENTS
    //try to load a cached character with the given skill
    if reload == 0 {
        cachedhandle = BotFindCachedCharacter(charfile, skill);
        if cachedhandle != 0 {
            (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
                b"loaded cached skill %f from %s\n\0".as_ptr() as *const c_char, skill, charfile);
            return cachedhandle;
        } //end if
    } //end else
    //
    intskill = (skill + 0.5) as c_int;
    //try to load the character with the given skill
    ch = BotLoadCharacterFromFile(charfile, intskill);
    if !ch.is_null() {
        botcharacters[handle as usize] = ch;
        //
        (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
            b"loaded skill %d from %s\n\0".as_ptr() as *const c_char, intskill, charfile);
        #[cfg(debug_assertions)]
        {
            if bot_developer != 0 {
                (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
                    b"skill %d loaded in %d msec from %s\n\0".as_ptr() as *const c_char, intskill, Sys_MilliSeconds() - starttime, charfile);
            } //end if
        }
        return handle;
    } //end if
    //
    (*core::ptr::addr_of_mut!(botimport)).Print(3, // PRT_WARNING
        b"couldn\'t find skill %d in %s\n\0".as_ptr() as *const c_char, intskill, charfile);
    //
    if reload == 0 {
        //try to load a cached default character with the given skill
        cachedhandle = BotFindCachedCharacter(DEFAULT_CHARACTER.as_ptr() as *mut c_char, skill);
        if cachedhandle != 0 {
            (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
                b"loaded cached default skill %d from %s\n\0".as_ptr() as *const c_char, intskill, charfile);
            return cachedhandle;
        } //end if
    } //end if
    //try to load the default character with the given skill
    ch = BotLoadCharacterFromFile(DEFAULT_CHARACTER.as_ptr() as *mut c_char, intskill);
    if !ch.is_null() {
        botcharacters[handle as usize] = ch;
        (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
            b"loaded default skill %d from %s\n\0".as_ptr() as *const c_char, intskill, charfile);
        return handle;
    } //end if
    //
    if reload == 0 {
        //try to load a cached character with any skill
        cachedhandle = BotFindCachedCharacter(charfile, -1.0);
        if cachedhandle != 0 {
            (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
                b"loaded cached skill %f from %s\n\0".as_ptr() as *const c_char, (*botcharacters[cachedhandle as usize]).skill, charfile);
            return cachedhandle;
        } //end if
    } //end if
    //try to load a character with any skill
    ch = BotLoadCharacterFromFile(charfile, -1);
    if !ch.is_null() {
        botcharacters[handle as usize] = ch;
        (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
            b"loaded skill %f from %s\n\0".as_ptr() as *const c_char, (*ch).skill, charfile);
        return handle;
    } //end if
    //
    if reload == 0 {
        //try to load a cached character with any skill
        cachedhandle = BotFindCachedCharacter(DEFAULT_CHARACTER.as_ptr() as *mut c_char, -1.0);
        if cachedhandle != 0 {
            (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
                b"loaded cached default skill %f from %s\n\0".as_ptr() as *const c_char, (*botcharacters[cachedhandle as usize]).skill, charfile);
            return cachedhandle;
        } //end if
    } //end if
    //try to load a character with any skill
    ch = BotLoadCharacterFromFile(DEFAULT_CHARACTER.as_ptr() as *mut c_char, -1);
    if !ch.is_null() {
        botcharacters[handle as usize] = ch;
        (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
            b"loaded default skill %f from %s\n\0".as_ptr() as *const c_char, (*ch).skill, charfile);
        return handle;
    } //end if
    //
    (*core::ptr::addr_of_mut!(botimport)).Print(3, // PRT_WARNING
        b"couldn\'t load any skill from %s\n\0".as_ptr() as *const c_char, charfile);
    //couldn't load any character
    return 0;
} //end of the function BotLoadCachedCharacter

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotLoadCharacterSkill(charfile: *mut c_char, skill: f32) -> c_int {
    let mut ch: c_int;
    let mut defaultch: c_int;

    defaultch = BotLoadCachedCharacter(DEFAULT_CHARACTER.as_ptr() as *mut c_char, skill, qfalse);
    ch = BotLoadCachedCharacter(charfile, skill, LibVarGetValue(b"bot_reloadcharacters\0".as_ptr() as *const c_char));

    if defaultch != 0 && ch != 0 {
        BotDefaultCharacteristics(botcharacters[ch as usize], botcharacters[defaultch as usize]);
    } //end if

    return ch;
} //end of the function BotLoadCharacterSkill

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotInterpolateCharacters(handle1: c_int, handle2: c_int, desiredskill: f32) -> c_int {
    let mut ch1: *mut bot_character_t;
    let mut ch2: *mut bot_character_t;
    let mut out: *mut bot_character_t;
    let mut i: c_int;
    let mut handle: c_int;
    let mut scale: f32;

    ch1 = BotCharacterFromHandle(handle1);
    ch2 = BotCharacterFromHandle(handle2);
    if ch1.is_null() || ch2.is_null() {
        return 0;
    }
    //find a free spot for a character
    handle = 1;
    while handle <= 64 { // MAX_CLIENTS
        if botcharacters[handle as usize].is_null() { break; }
        handle += 1;
    } //end for
    if handle > 64 { return 0; } // MAX_CLIENTS
    out = GetClearedMemory(mem::size_of::<bot_character_t>() +
                    MAX_CHARACTERISTICS as usize * mem::size_of::<bot_characteristic_t>()) as *mut bot_character_t;
    (*out).skill = desiredskill;
    libc_strcpy((*out).filename.as_mut_ptr(), (*ch1).filename.as_ptr());
    botcharacters[handle as usize] = out;

    scale = (desiredskill - (*ch1).skill) / ((*ch2).skill - (*ch1).skill);
    i = 0;
    while i < MAX_CHARACTERISTICS {
        //
        let ch1_c_ptr = (*ch1).c.as_ptr().add(i as usize);
        let ch2_c_ptr = (*ch2).c.as_ptr().add(i as usize);
        let out_c_ptr = (*out).c.as_ptr().add(i as usize) as *mut bot_characteristic_t;
        if (*ch1_c_ptr).type_ == CT_FLOAT && (*ch2_c_ptr).type_ == CT_FLOAT {
            (*out_c_ptr).type_ = CT_FLOAT;
            (*out_c_ptr).value._float = (*ch1_c_ptr).value._float +
                                ((*ch2_c_ptr).value._float - (*ch1_c_ptr).value._float) * scale;
        } //end if
        else if (*ch1_c_ptr).type_ == CT_INTEGER {
            (*out_c_ptr).type_ = CT_INTEGER;
            (*out_c_ptr).value.integer = (*ch1_c_ptr).value.integer;
        } //end else if
        else if (*ch1_c_ptr).type_ == CT_STRING {
            (*out_c_ptr).type_ = CT_STRING;
            let len = libc_strlen((*ch1_c_ptr).value.string) + 1;
            (*out_c_ptr).value.string = GetMemory(len) as *mut c_char;
            libc_strcpy((*out_c_ptr).value.string, (*ch1_c_ptr).value.string);
        } //end else if
        i += 1;
    } //end for
    return handle;
} //end of the function BotInterpolateCharacters

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotLoadCharacter(charfile: *mut c_char, mut skill: f32) -> c_int {
    let mut firstskill: c_int;
    let mut secondskill: c_int;
    let mut handle: c_int;

    //make sure the skill is in the valid range
    if skill < 1.0 { skill = 1.0; }
    else if skill > 5.0 { skill = 5.0; }
    //skill 1, 4 and 5 should be available in the character files
    if skill == 1.0 || skill == 4.0 || skill == 5.0 {
        return BotLoadCharacterSkill(charfile, skill);
    } //end if
    //check if there's a cached skill
    handle = BotFindCachedCharacter(charfile, skill);
    if handle != 0 {
        (*core::ptr::addr_of_mut!(botimport)).Print(0, // PRT_MESSAGE
            b"loaded cached skill %f from %s\n\0".as_ptr() as *const c_char, skill, charfile);
        return handle;
    } //end if
    if skill < 4.0 {
        //load skill 1 and 4
        firstskill = BotLoadCharacterSkill(charfile, 1.0);
        if firstskill == 0 { return 0; }
        secondskill = BotLoadCharacterSkill(charfile, 4.0);
        if secondskill == 0 { return firstskill; }
    } //end if
    else {
        //load skill 4 and 5
        firstskill = BotLoadCharacterSkill(charfile, 4.0);
        if firstskill == 0 { return 0; }
        secondskill = BotLoadCharacterSkill(charfile, 5.0);
        if secondskill == 0 { return firstskill; }
    } //end else
    //interpolate between the two skills
    handle = BotInterpolateCharacters(firstskill, secondskill, skill);
    if handle == 0 { return 0; }
    //write the character to the log file
    BotDumpCharacter(botcharacters[handle as usize]);
    //
    return handle;
} //end of the function BotLoadCharacter

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn CheckCharacteristicIndex(character: c_int, index: c_int) -> c_int {
    let mut ch: *mut bot_character_t;

    ch = BotCharacterFromHandle(character);
    if ch.is_null() { return qfalse; }
    if index < 0 || index >= MAX_CHARACTERISTICS {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, // PRT_ERROR
            b"characteristic %d does not exist\n\0".as_ptr() as *const c_char, index);
        return qfalse;
    } //end if
    let c_ptr = (*ch).c.as_ptr().add(index as usize);
    if (*c_ptr).type_ == 0 {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, // PRT_ERROR
            b"characteristic %d is not initialized\n\0".as_ptr() as *const c_char, index);
        return qfalse;
    } //end if
    return qtrue;
} //end of the function CheckCharacteristicIndex

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn Characteristic_Float(character: c_int, index: c_int) -> f32 {
    let mut ch: *mut bot_character_t;

    ch = BotCharacterFromHandle(character);
    if ch.is_null() { return 0.0; }
    //check if the index is in range
    if CheckCharacteristicIndex(character, index) == 0 { return 0.0; }
    let c_ptr = (*ch).c.as_ptr().add(index as usize);
    //an integer will be converted to a float
    if (*c_ptr).type_ == CT_INTEGER {
        return (*c_ptr).value.integer as f32;
    } //end if
    //floats are just returned
    else if (*c_ptr).type_ == CT_FLOAT {
        return (*c_ptr).value._float;
    } //end else if
    //cannot convert a string pointer to a float
    else {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, // PRT_ERROR
            b"characteristic %d is not a float\n\0".as_ptr() as *const c_char, index);
        return 0.0;
    } //end else if
    //	return 0;
} //end of the function Characteristic_Float

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn Characteristic_BFloat(character: c_int, index: c_int, min: f32, max: f32) -> f32 {
    let mut value: f32;
    let mut ch: *mut bot_character_t;

    ch = BotCharacterFromHandle(character);
    if ch.is_null() { return 0.0; }
    if min > max {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, // PRT_ERROR
            b"cannot bound characteristic %d between %f and %f\n\0".as_ptr() as *const c_char, index, min, max);
        return 0.0;
    } //end if
    value = Characteristic_Float(character, index);
    if value < min { return min; }
    if value > max { return max; }
    return value;
} //end of the function Characteristic_BFloat

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn Characteristic_Integer(character: c_int, index: c_int) -> c_int {
    let mut ch: *mut bot_character_t;

    ch = BotCharacterFromHandle(character);
    if ch.is_null() { return 0; }
    //check if the index is in range
    if CheckCharacteristicIndex(character, index) == 0 { return 0; }
    let c_ptr = (*ch).c.as_ptr().add(index as usize);
    //an integer will just be returned
    if (*c_ptr).type_ == CT_INTEGER {
        return (*c_ptr).value.integer;
    } //end if
    //floats are casted to integers
    else if (*c_ptr).type_ == CT_FLOAT {
        return (*c_ptr).value._float as c_int;
    } //end else if
    else {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, // PRT_ERROR
            b"characteristic %d is not a integer\n\0".as_ptr() as *const c_char, index);
        return 0;
    } //end else if
    //	return 0;
} //end of the function Characteristic_Integer

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn Characteristic_BInteger(character: c_int, index: c_int, min: c_int, max: c_int) -> c_int {
    let mut value: c_int;
    let mut ch: *mut bot_character_t;

    ch = BotCharacterFromHandle(character);
    if ch.is_null() { return 0; }
    if min > max {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, // PRT_ERROR
            b"cannot bound characteristic %d between %d and %d\n\0".as_ptr() as *const c_char, index, min, max);
        return 0;
    } //end if
    value = Characteristic_Integer(character, index);
    if value < min { return min; }
    if value > max { return max; }
    return value;
} //end of the function Characteristic_BInteger

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn Characteristic_String(character: c_int, index: c_int, buf: *mut c_char, size: c_int) {
    let mut ch: *mut bot_character_t;

    ch = BotCharacterFromHandle(character);
    if ch.is_null() { return; }
    //check if the index is in range
    if CheckCharacteristicIndex(character, index) == 0 { return; }
    let c_ptr = (*ch).c.as_ptr().add(index as usize);
    //an integer will be converted to a float
    if (*c_ptr).type_ == CT_STRING {
        libc_strncpy(buf, (*c_ptr).value.string, size - 1);
        *buf.add((size - 1) as usize) = 0;
        return;
    } //end if
    else {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, // PRT_ERROR
            b"characteristic %d is not a string\n\0".as_ptr() as *const c_char, index);
        return;
    } //end else if
    return;
} //end of the function Characteristic_String

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn BotShutdownCharacters() {
    let mut handle: c_int;

    handle = 1;
    while handle <= 64 { // MAX_CLIENTS
        if !botcharacters[handle as usize].is_null() {
            BotFreeCharacter2(handle);
        } //end if
        handle += 1;
    } //end for
} //end of the function BotShutdownCharacters

// Local libc stubs

fn c_strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut i = 0;
        loop {
            let c1 = *s1.add(i);
            let c2 = *s2.add(i);
            if c1 != c2 {
                return (c1 as c_int) - (c2 as c_int);
            }
            if c1 == 0 {
                return 0;
            }
            i += 1;
        }
    }
}

fn libc_strlen(s: *const c_char) -> usize {
    unsafe {
        let mut len = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        len
    }
}

fn libc_strcpy(dest: *mut c_char, src: *const c_char) {
    unsafe {
        let mut i = 0;
        loop {
            let c = *src.add(i);
            *dest.add(i) = c;
            if c == 0 {
                break;
            }
            i += 1;
        }
    }
}

fn libc_strncpy(dest: *mut c_char, src: *const c_char, n: c_int) {
    unsafe {
        let mut i = 0;
        while i < n as usize {
            let c = *src.add(i);
            *dest.add(i) = c;
            if c == 0 {
                break;
            }
            i += 1;
        }
    }
}

#[cfg(debug_assertions)]
static mut bot_developer: c_int = 0;
