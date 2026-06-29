
/*****************************************************************************
 * name:		be_ai_weight.c
 *
 * desc:		fuzzy logic
 *
 * $Archive: /MissionPack/code/botlib/be_ai_weight.c $
 * $Author: Mrelusive $
 * $Revision: 3 $
 * $Modtime: 8/06/00 5:25p $
 * $Date: 8/06/00 11:07p $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_char, c_void};
use core::mem;

#[path = "be_ai_weight_h.rs"]
mod be_ai_weight_h;
use be_ai_weight_h::*;

const MAX_INVENTORYVALUE: c_int = 999999;

const MAX_WEIGHT_FILES: usize = 128;
static mut weightFileList: [*mut weightconfig_t; 128] = [core::ptr::null_mut(); 128];

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
extern "C" {
    static mut botimport: BotImportStruct;
    fn PC_SetBaseFolder(folder: *const c_char);
    fn LoadSourceFile(filename: *const c_char) -> *mut source_t;
    fn FreeSource(source: *mut source_t);
    fn PC_ReadToken(source: *mut source_t, token: *mut token_t) -> c_int;
    fn PC_ExpectTokenType(source: *mut source_t, ttype: c_int, subtype: c_int, token: *mut token_t) -> c_int;
    fn PC_ExpectTokenString(source: *mut source_t, string: *const c_char) -> c_int;
    fn PC_ExpectAnyToken(source: *mut source_t, token: *mut token_t) -> c_int;
    fn PC_CheckTokenString(source: *mut source_t, string: *const c_char) -> c_int;
    fn StripDoubleQuotes(string: *mut c_char);
    fn SourceError(source: *mut source_t, format: *const c_char, ...);
    fn SourceWarning(source: *mut source_t, format: *const c_char, ...);
    fn GetClearedMemory(size: usize) -> *mut c_void;
    fn FreeMemory(ptr: *mut c_void);
    fn LibVarGetValue(varname: *const c_char) -> c_int;
    fn Sys_MilliSeconds() -> c_int;
}

// Stub types for external dependencies
#[repr(C)]
struct BotImportStruct {
    Print: extern "C" fn(c_int, *const c_char, ...) -> c_int,
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

// Stub for Q_strncpyz
unsafe fn Q_strncpyz(dest: *mut c_char, src: *const c_char, len: usize) {
    let mut i: usize = 0;
    while i < len - 1 && *src.add(i) != 0 {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    *dest.add(i) = 0;
}

// Stubs for libc functions
unsafe fn c_strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    let mut i: usize = 0;
    loop {
        let c1 = *s1.add(i) as u8;
        let c2 = *s2.add(i) as u8;
        if c1 != c2 {
            return (c1 as i32) - (c2 as i32);
        }
        if c1 == 0 {
            return 0;
        }
        i += 1;
    }
}

unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut len: usize = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

unsafe fn c_strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    let mut i: usize = 0;
    loop {
        *dest.add(i) = *src.add(i);
        if *src.add(i) == 0 {
            break;
        }
        i += 1;
    }
    dest
}

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ReadValue(source: *mut source_t, value: *mut f32) -> c_int {
    let mut token: token_t = mem::zeroed();

    if PC_ExpectAnyToken(source, &mut token) == 0 { return qfalse; }
    if c_strcmp(token.string.as_ptr(), b"-\0".as_ptr() as *const c_char) == 0 {
        SourceWarning(source, b"negative value set to zero\n\0".as_ptr() as *const c_char);
        if PC_ExpectTokenType(source, 1, 0, &mut token) == 0 { return qfalse; } // TT_NUMBER
    } //end if
    if token.type_ != 1 { // TT_NUMBER
        SourceError(source, b"invalid return value %s\n\0".as_ptr() as *const c_char, token.string.as_ptr());
        return qfalse;
    } //end if
    *value = token.floatvalue;
    return qtrue;
} //end of the function ReadValue

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ReadFuzzyWeight(source: *mut source_t, fs: *mut fuzzyseperator_t) -> c_int {
    if PC_CheckTokenString(source, b"balance\0".as_ptr() as *const c_char) != 0 {
        (*fs).r#type = WT_BALANCE;
        if PC_ExpectTokenString(source, b"(\0".as_ptr() as *const c_char) == 0 { return qfalse; }
        if ReadValue(source, &mut (*fs).weight) == 0 { return qfalse; }
        if PC_ExpectTokenString(source, b",\0".as_ptr() as *const c_char) == 0 { return qfalse; }
        if ReadValue(source, &mut (*fs).minweight) == 0 { return qfalse; }
        if PC_ExpectTokenString(source, b",\0".as_ptr() as *const c_char) == 0 { return qfalse; }
        if ReadValue(source, &mut (*fs).maxweight) == 0 { return qfalse; }
        if PC_ExpectTokenString(source, b")\0".as_ptr() as *const c_char) == 0 { return qfalse; }
    } //end if
    else {
        (*fs).r#type = 0;
        if ReadValue(source, &mut (*fs).weight) == 0 { return qfalse; }
        (*fs).minweight = (*fs).weight;
        (*fs).maxweight = (*fs).weight;
    } //end if
    if PC_ExpectTokenString(source, b";\0".as_ptr() as *const c_char) == 0 { return qfalse; }
    return qtrue;
} //end of the function ReadFuzzyWeight

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn FreeFuzzySeperators_r(fs: *mut fuzzyseperator_t) {
    if fs.is_null() { return; }
    if !(*fs).child.is_null() { FreeFuzzySeperators_r((*fs).child); }
    if !(*fs).next.is_null() { FreeFuzzySeperators_r((*fs).next); }
    FreeMemory(fs as *mut c_void);
} //end of the function FreeFuzzySeperators

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn FreeWeightConfig2(config: *mut weightconfig_t) {
    let mut i: c_int;

    i = 0;
    while i < (*config).numweights {
        FreeFuzzySeperators_r((*config).weights[i as usize].firstseperator);
        if !(*config).weights[i as usize].name.is_null() { FreeMemory((*config).weights[i as usize].name as *mut c_void); }
        i += 1;
    } //end for
    FreeMemory(config as *mut c_void);
} //end of the function FreeWeightConfig2

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn FreeWeightConfig(config: *mut weightconfig_t) {
    if LibVarGetValue(b"bot_reloadcharacters\0".as_ptr() as *const c_char) == 0 { return; }
    FreeWeightConfig2(config);
} //end of the function FreeWeightConfig

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
unsafe fn ReadFuzzySeperators_r(source: *mut source_t) -> *mut fuzzyseperator_t {
    let mut newindent: c_int;
    let mut index: c_int;
    let mut def: c_int;
    let mut founddefault: c_int;
    let mut token: token_t = mem::zeroed();
    let mut fs: *mut fuzzyseperator_t;
    let mut lastfs: *mut fuzzyseperator_t;
    let mut firstfs: *mut fuzzyseperator_t;

    founddefault = qfalse;
    firstfs = core::ptr::null_mut();
    lastfs = core::ptr::null_mut();
    if PC_ExpectTokenString(source, b"(\0".as_ptr() as *const c_char) == 0 { return core::ptr::null_mut(); }
    if PC_ExpectTokenType(source, 1, 2, &mut token) == 0 { return core::ptr::null_mut(); } // TT_NUMBER, TT_INTEGER
    index = token.intvalue;
    if PC_ExpectTokenString(source, b")\0".as_ptr() as *const c_char) == 0 { return core::ptr::null_mut(); }
    if PC_ExpectTokenString(source, b"{\0".as_ptr() as *const c_char) == 0 { return core::ptr::null_mut(); }
    if PC_ExpectAnyToken(source, &mut token) == 0 { return core::ptr::null_mut(); }
    loop {
        def = if c_strcmp(token.string.as_ptr(), b"default\0".as_ptr() as *const c_char) == 0 { qtrue } else { qfalse };
        if def != 0 || c_strcmp(token.string.as_ptr(), b"case\0".as_ptr() as *const c_char) == 0 {
            fs = GetClearedMemory(mem::size_of::<fuzzyseperator_t>()) as *mut fuzzyseperator_t;
            (*fs).index = index;
            if !lastfs.is_null() { (*lastfs).next = fs; }
            else { firstfs = fs; }
            lastfs = fs;
            if def != 0 {
                if founddefault != 0 {
                    SourceError(source, b"switch already has a default\n\0".as_ptr() as *const c_char);
                    FreeFuzzySeperators_r(firstfs);
                    return core::ptr::null_mut();
                } //end if
                (*fs).value = MAX_INVENTORYVALUE;
                founddefault = qtrue;
            } //end if
            else {
                if PC_ExpectTokenType(source, 1, 2, &mut token) == 0 { // TT_NUMBER, TT_INTEGER
                    FreeFuzzySeperators_r(firstfs);
                    return core::ptr::null_mut();
                } //end if
                (*fs).value = token.intvalue;
            } //end else
            if PC_ExpectTokenString(source, b":\0".as_ptr() as *const c_char) == 0 || PC_ExpectAnyToken(source, &mut token) == 0 {
                FreeFuzzySeperators_r(firstfs);
                return core::ptr::null_mut();
            } //end if
            newindent = qfalse;
            if c_strcmp(token.string.as_ptr(), b"{\0".as_ptr() as *const c_char) == 0 {
                newindent = qtrue;
                if PC_ExpectAnyToken(source, &mut token) == 0 {
                    FreeFuzzySeperators_r(firstfs);
                    return core::ptr::null_mut();
                } //end if
            } //end if
            if c_strcmp(token.string.as_ptr(), b"return\0".as_ptr() as *const c_char) == 0 {
                if ReadFuzzyWeight(source, fs) == 0 {
                    FreeFuzzySeperators_r(firstfs);
                    return core::ptr::null_mut();
                } //end if
            } //end if
            else if c_strcmp(token.string.as_ptr(), b"switch\0".as_ptr() as *const c_char) == 0 {
                (*fs).child = ReadFuzzySeperators_r(source);
                if (*fs).child.is_null() {
                    FreeFuzzySeperators_r(firstfs);
                    return core::ptr::null_mut();
                } //end if
            } //end else if
            else {
                SourceError(source, b"invalid name %s\n\0".as_ptr() as *const c_char, token.string.as_ptr());
                return core::ptr::null_mut();
            } //end else
            if newindent != 0 {
                if PC_ExpectTokenString(source, b"}\0".as_ptr() as *const c_char) == 0 {
                    FreeFuzzySeperators_r(firstfs);
                    return core::ptr::null_mut();
                } //end if
            } //end if
        } //end if
        else {
            FreeFuzzySeperators_r(firstfs);
            SourceError(source, b"invalid name %s\n\0".as_ptr() as *const c_char, token.string.as_ptr());
            return core::ptr::null_mut();
        } //end else
        if PC_ExpectAnyToken(source, &mut token) == 0 {
            FreeFuzzySeperators_r(firstfs);
            return core::ptr::null_mut();
        } //end if
        if c_strcmp(token.string.as_ptr(), b"}\0".as_ptr() as *const c_char) == 0 { break; }
    }
    //
    if founddefault == 0 {
        SourceWarning(source, b"switch without default\n\0".as_ptr() as *const c_char);
        fs = GetClearedMemory(mem::size_of::<fuzzyseperator_t>()) as *mut fuzzyseperator_t;
        (*fs).index = index;
        (*fs).value = MAX_INVENTORYVALUE;
        (*fs).weight = 0.0;
        (*fs).next = core::ptr::null_mut();
        (*fs).child = core::ptr::null_mut();
        if !lastfs.is_null() { (*lastfs).next = fs; }
        else { firstfs = fs; }
        lastfs = fs;
    } //end if
    //
    return firstfs;
} //end of the function ReadFuzzySeperators_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn ReadWeightConfig(filename: *mut c_char) -> *mut weightconfig_t {
    let mut newindent: c_int;
    let mut avail: c_int = 0;
    let mut n: c_int;
    let mut token: token_t = mem::zeroed();
    let mut source: *mut source_t;
    let mut fs: *mut fuzzyseperator_t;
    let mut config: *mut weightconfig_t = core::ptr::null_mut();
    #[cfg(feature = "debug")]
    let mut starttime: c_int = 0;

    #[cfg(feature = "debug")]
    {
        starttime = Sys_MilliSeconds();
    }

    if LibVarGetValue(b"bot_reloadcharacters\0".as_ptr() as *const c_char) == 0 {
        avail = -1;
        n = 0;
        while n < MAX_WEIGHT_FILES as c_int {
            config = *core::ptr::addr_of!(weightFileList[n as usize]);
            if config.is_null() {
                if avail == -1 {
                    avail = n;
                } //end if
                n += 1;
                continue;
            } //end if
            if c_strcmp(filename, (*config).filename.as_ptr()) == 0 {
                //botimport.Print( PRT_MESSAGE, "retained %s\n", filename );
                return config;
            } //end if
            n += 1;
        } //end for

        if avail == -1 {
            (*core::ptr::addr_of_mut!(botimport)).Print(2, b"weightFileList was full trying to load %s\n\0".as_ptr() as *const c_char, filename);
            return core::ptr::null_mut();
        } //end if
    } //end if

    PC_SetBaseFolder(b"botfiles\0".as_ptr() as *const c_char); // BOTFILESBASEFOLDER
    source = LoadSourceFile(filename);
    if source.is_null() {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, b"counldn't load %s\n\0".as_ptr() as *const c_char, filename);
        return core::ptr::null_mut();
    } //end if
    //
    config = GetClearedMemory(mem::size_of::<weightconfig_t>()) as *mut weightconfig_t;
    (*config).numweights = 0;
    Q_strncpyz((*config).filename.as_mut_ptr(), filename, mem::size_of_val(&(*config).filename));
    //parse the item config file
    while PC_ReadToken(source, &mut token) != 0 {
        if c_strcmp(token.string.as_ptr(), b"weight\0".as_ptr() as *const c_char) == 0 {
            if (*config).numweights >= MAX_WEIGHTS {
                SourceWarning(source, b"too many fuzzy weights\n\0".as_ptr() as *const c_char);
                break;
            } //end if
            if PC_ExpectTokenType(source, 4, 0, &mut token) == 0 { // TT_STRING
                FreeWeightConfig(config);
                FreeSource(source);
                return core::ptr::null_mut();
            } //end if
            StripDoubleQuotes(token.string.as_mut_ptr());
            (*config).weights[(*config).numweights as usize].name = GetClearedMemory(c_strlen(token.string.as_ptr()) + 1) as *mut c_char;
            c_strcpy((*config).weights[(*config).numweights as usize].name, token.string.as_ptr());
            if PC_ExpectAnyToken(source, &mut token) == 0 {
                FreeWeightConfig(config);
                FreeSource(source);
                return core::ptr::null_mut();
            } //end if
            newindent = qfalse;
            if c_strcmp(token.string.as_ptr(), b"{\0".as_ptr() as *const c_char) == 0 {
                newindent = qtrue;
                if PC_ExpectAnyToken(source, &mut token) == 0 {
                    FreeWeightConfig(config);
                    FreeSource(source);
                    return core::ptr::null_mut();
                } //end if
            } //end if
            if c_strcmp(token.string.as_ptr(), b"switch\0".as_ptr() as *const c_char) == 0 {
                fs = ReadFuzzySeperators_r(source);
                if fs.is_null() {
                    FreeWeightConfig(config);
                    FreeSource(source);
                    return core::ptr::null_mut();
                } //end if
                (*config).weights[(*config).numweights as usize].firstseperator = fs;
            } //end if
            else if c_strcmp(token.string.as_ptr(), b"return\0".as_ptr() as *const c_char) == 0 {
                fs = GetClearedMemory(mem::size_of::<fuzzyseperator_t>()) as *mut fuzzyseperator_t;
                (*fs).index = 0;
                (*fs).value = MAX_INVENTORYVALUE;
                (*fs).next = core::ptr::null_mut();
                (*fs).child = core::ptr::null_mut();
                if ReadFuzzyWeight(source, fs) == 0 {
                    FreeMemory(fs as *mut c_void);
                    FreeWeightConfig(config);
                    FreeSource(source);
                    return core::ptr::null_mut();
                } //end if
                (*config).weights[(*config).numweights as usize].firstseperator = fs;
            } //end else if
            else {
                SourceError(source, b"invalid name %s\n\0".as_ptr() as *const c_char, token.string.as_ptr());
                FreeWeightConfig(config);
                FreeSource(source);
                return core::ptr::null_mut();
            } //end else
            if newindent != 0 {
                if PC_ExpectTokenString(source, b"}\0".as_ptr() as *const c_char) == 0 {
                    FreeWeightConfig(config);
                    FreeSource(source);
                    return core::ptr::null_mut();
                } //end if
            } //end if
            (*config).numweights += 1;
        } //end if
        else {
            SourceError(source, b"invalid name %s\n\0".as_ptr() as *const c_char, token.string.as_ptr());
            FreeWeightConfig(config);
            FreeSource(source);
            return core::ptr::null_mut();
        } //end else
    } //end while
    //free the source at the end of a pass
    FreeSource(source);
    //if the file was located in a pak file
    (*core::ptr::addr_of_mut!(botimport)).Print(0, b"loaded %s\n\0".as_ptr() as *const c_char, filename);
    #[cfg(feature = "debug")]
    {
        if bot_developer != 0 {
            (*core::ptr::addr_of_mut!(botimport)).Print(0, b"weights loaded in %d msec\n\0".as_ptr() as *const c_char, Sys_MilliSeconds() - starttime);
        } //end if
    }
    //
    if LibVarGetValue(b"bot_reloadcharacters\0".as_ptr() as *const c_char) == 0 {
        *core::ptr::addr_of_mut!(weightFileList[avail as usize]) = config;
    } //end if
    //
    return config;
} //end of the function ReadWeightConfig

#[cfg(feature = "unused")]
unsafe fn WriteFuzzyWeight(fp: *mut core::ffi::c_void, fs: *mut fuzzyseperator_t) -> c_int {
    if (*fs).r#type == WT_BALANCE {
        if libc_fprintf(fp, b" return balance(\0".as_ptr() as *const c_char) < 0 { return qfalse; }
        if WriteFloat(fp, (*fs).weight) == 0 { return qfalse; }
        if libc_fprintf(fp, b",\0".as_ptr() as *const c_char) < 0 { return qfalse; }
        if WriteFloat(fp, (*fs).minweight) == 0 { return qfalse; }
        if libc_fprintf(fp, b",\0".as_ptr() as *const c_char) < 0 { return qfalse; }
        if WriteFloat(fp, (*fs).maxweight) == 0 { return qfalse; }
        if libc_fprintf(fp, b");\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
    } //end if
    else {
        if libc_fprintf(fp, b" return \0".as_ptr() as *const c_char) < 0 { return qfalse; }
        if WriteFloat(fp, (*fs).weight) == 0 { return qfalse; }
        if libc_fprintf(fp, b";\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
    } //end else
    return qtrue;
} //end of the function WriteFuzzyWeight

#[cfg(feature = "unused")]
unsafe fn WriteFuzzySeperators_r(fp: *mut core::ffi::c_void, fs: *mut fuzzyseperator_t, indent: c_int) -> c_int {
    let mut indent = indent;
    if WriteIndent(fp, indent) == 0 { return qfalse; }
    if libc_fprintf(fp, b"switch(%d)\n\0".as_ptr() as *const c_char, (*fs).index) < 0 { return qfalse; }
    if WriteIndent(fp, indent) == 0 { return qfalse; }
    if libc_fprintf(fp, b"{\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
    indent += 1;
    loop {
        if WriteIndent(fp, indent) == 0 { return qfalse; }
        if !(*fs).next.is_null() {
            if libc_fprintf(fp, b"case %d:\0".as_ptr() as *const c_char, (*fs).value) < 0 { return qfalse; }
        } //end if
        else {
            if libc_fprintf(fp, b"default:\0".as_ptr() as *const c_char) < 0 { return qfalse; }
        } //end else
        if !(*fs).child.is_null() {
            if libc_fprintf(fp, b"\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
            if WriteIndent(fp, indent) == 0 { return qfalse; }
            if libc_fprintf(fp, b"{\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
            if WriteFuzzySeperators_r(fp, (*fs).child, indent + 1) == 0 { return qfalse; }
            if WriteIndent(fp, indent) == 0 { return qfalse; }
            if !(*fs).next.is_null() {
                if libc_fprintf(fp, b"} //end case\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
            } //end if
            else {
                if libc_fprintf(fp, b"} //end default\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
            } //end else
        } //end if
        else {
            if WriteFuzzyWeight(fp, fs) == 0 { return qfalse; }
        } //end else
        fs = (*fs).next;
        if fs.is_null() { break; }
    }
    indent -= 1;
    if WriteIndent(fp, indent) == 0 { return qfalse; }
    if libc_fprintf(fp, b"} //end switch\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
    return qtrue;
} //end of the function WriteItemFuzzyWeights_r

#[cfg(feature = "unused")]
pub unsafe fn WriteWeightConfig(filename: *mut c_char, config: *mut weightconfig_t) -> c_int {
    let mut i: c_int;
    let mut fp: *mut core::ffi::c_void;
    let mut ifw: *mut weight_t;

    fp = libc_fopen(filename, b"wb\0".as_ptr() as *const c_char);
    if fp.is_null() { return qfalse; }

    i = 0;
    while i < (*config).numweights {
        ifw = &mut (*config).weights[i as usize] as *mut weight_t;
        if libc_fprintf(fp, b"\nweight \"%s\"\n\0".as_ptr() as *const c_char, (*ifw).name) < 0 { return qfalse; }
        if libc_fprintf(fp, b"{\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
        if (*(*ifw).firstseperator).index > 0 {
            if WriteFuzzySeperators_r(fp, (*ifw).firstseperator, 1) == 0 { return qfalse; }
        } //end if
        else {
            if WriteIndent(fp, 1) == 0 { return qfalse; }
            if WriteFuzzyWeight(fp, (*ifw).firstseperator) == 0 { return qfalse; }
        } //end else
        if libc_fprintf(fp, b"} //end weight\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
        i += 1;
    } //end for
    libc_fclose(fp);
    return qtrue;
} //end of the function WriteWeightConfig

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn FindFuzzyWeight(wc: *mut weightconfig_t, name: *mut c_char) -> c_int {
    let mut i: c_int;

    i = 0;
    while i < (*wc).numweights {
        if c_strcmp((*wc).weights[i as usize].name, name) == 0 {
            return i;
        } //end if
        i += 1;
    } //end for
    return -1;
} //end of the function FindFuzzyWeight

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn FuzzyWeight_r(inventory: *mut c_int, fs: *mut fuzzyseperator_t) -> f32 {
    let mut scale: f32;
    let mut w1: f32;
    let mut w2: f32;

    if *inventory.add((*fs).index as usize) < (*fs).value {
        if !(*fs).child.is_null() { return FuzzyWeight_r(inventory, (*fs).child); }
        else { return (*fs).weight; }
    } //end if
    else if !(*fs).next.is_null() {
        if *inventory.add((*fs).index as usize) < (*(*fs).next).value {
            //first weight
            if !(*fs).child.is_null() { w1 = FuzzyWeight_r(inventory, (*fs).child); }
            else { w1 = (*fs).weight; }
            //second weight
            if !(*(*fs).next).child.is_null() { w2 = FuzzyWeight_r(inventory, (*(*fs).next).child); }
            else { w2 = (*(*fs).next).weight; }
            //the scale factor
            scale = (*inventory.add((*fs).index as usize) - (*fs).value) as f32 / ((*(*fs).next).value - (*fs).value) as f32;
            //scale between the two weights
            return scale * w1 + (1.0 - scale) * w2;
        } //end if
        return FuzzyWeight_r(inventory, (*fs).next);
    } //end else if
    return (*fs).weight;
} //end of the function FuzzyWeight_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn FuzzyWeightUndecided_r(inventory: *mut c_int, fs: *mut fuzzyseperator_t) -> f32 {
    let mut scale: f32;
    let mut w1: f32;
    let mut w2: f32;

    if *inventory.add((*fs).index as usize) < (*fs).value {
        if !(*fs).child.is_null() { return FuzzyWeightUndecided_r(inventory, (*fs).child); }
        else { return (*fs).minweight + random() * ((*fs).maxweight - (*fs).minweight); }
    } //end if
    else if !(*fs).next.is_null() {
        if *inventory.add((*fs).index as usize) < (*(*fs).next).value {
            //first weight
            if !(*fs).child.is_null() { w1 = FuzzyWeightUndecided_r(inventory, (*fs).child); }
            else { w1 = (*fs).minweight + random() * ((*fs).maxweight - (*fs).minweight); }
            //second weight
            if !(*(*fs).next).child.is_null() { w2 = FuzzyWeight_r(inventory, (*(*fs).next).child); }
            else { w2 = (*(*fs).next).minweight + random() * ((*(*fs).next).maxweight - (*(*fs).next).minweight); }
            //the scale factor
            scale = (*inventory.add((*fs).index as usize) - (*fs).value) as f32 / ((*(*fs).next).value - (*fs).value) as f32;
            //scale between the two weights
            return scale * w1 + (1.0 - scale) * w2;
        } //end if
        return FuzzyWeightUndecided_r(inventory, (*fs).next);
    } //end else if
    return (*fs).weight;
} //end of the function FuzzyWeightUndecided_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn FuzzyWeight(inventory: *mut c_int, wc: *mut weightconfig_t, weightnum: c_int) -> f32 {
    return FuzzyWeight_r(inventory, (*wc).weights[weightnum as usize].firstseperator);
} //end of the function FuzzyWeight

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn FuzzyWeightUndecided(inventory: *mut c_int, wc: *mut weightconfig_t, weightnum: c_int) -> f32 {
    return FuzzyWeightUndecided_r(inventory, (*wc).weights[weightnum as usize].firstseperator);
} //end of the function FuzzyWeightUndecided

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn EvolveFuzzySeperator_r(fs: *mut fuzzyseperator_t) {
    if !(*fs).child.is_null() {
        EvolveFuzzySeperator_r((*fs).child);
    } //end if
    else if (*fs).r#type == WT_BALANCE {
        //every once in a while an evolution leap occurs, mutation
        if random() < 0.01 { (*fs).weight += crandom() * ((*fs).maxweight - (*fs).minweight); }
        else { (*fs).weight += crandom() * ((*fs).maxweight - (*fs).minweight) * 0.5; }
        //modify bounds if necesary because of mutation
        if (*fs).weight < (*fs).minweight { (*fs).minweight = (*fs).weight; }
        else if (*fs).weight > (*fs).maxweight { (*fs).maxweight = (*fs).weight; }
    } //end else if
    if !(*fs).next.is_null() { EvolveFuzzySeperator_r((*fs).next); }
} //end of the function EvolveFuzzySeperator_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn EvolveWeightConfig(config: *mut weightconfig_t) {
    let mut i: c_int;

    i = 0;
    while i < (*config).numweights {
        EvolveFuzzySeperator_r((*config).weights[i as usize].firstseperator);
        i += 1;
    } //end for
} //end of the function EvolveWeightConfig

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ScaleFuzzySeperator_r(fs: *mut fuzzyseperator_t, scale: f32) {
    if !(*fs).child.is_null() {
        ScaleFuzzySeperator_r((*fs).child, scale);
    } //end if
    else if (*fs).r#type == WT_BALANCE {
        //
        (*fs).weight = ((*fs).maxweight + (*fs).minweight) * scale;
        //get the weight between bounds
        if (*fs).weight < (*fs).minweight { (*fs).weight = (*fs).minweight; }
        else if (*fs).weight > (*fs).maxweight { (*fs).weight = (*fs).maxweight; }
    } //end else if
    if !(*fs).next.is_null() { ScaleFuzzySeperator_r((*fs).next, scale); }
} //end of the function ScaleFuzzySeperator_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn ScaleWeight(config: *mut weightconfig_t, name: *mut c_char, mut scale: f32) {
    let mut i: c_int;

    if scale < 0.0 { scale = 0.0; }
    else if scale > 1.0 { scale = 1.0; }
    i = 0;
    while i < (*config).numweights {
        if c_strcmp(name, (*config).weights[i as usize].name) == 0 {
            ScaleFuzzySeperator_r((*config).weights[i as usize].firstseperator, scale);
            break;
        } //end if
        i += 1;
    } //end for
} //end of the function ScaleWeight

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ScaleFuzzySeperatorBalanceRange_r(fs: *mut fuzzyseperator_t, scale: f32) {
    if !(*fs).child.is_null() {
        ScaleFuzzySeperatorBalanceRange_r((*fs).child, scale);
    } //end if
    else if (*fs).r#type == WT_BALANCE {
        let mid: f32 = ((*fs).minweight + (*fs).maxweight) * 0.5;
        //get the weight between bounds
        (*fs).maxweight = mid + ((*fs).maxweight - mid) * scale;
        (*fs).minweight = mid + ((*fs).minweight - mid) * scale;
        if (*fs).maxweight < (*fs).minweight {
            (*fs).maxweight = (*fs).minweight;
        } //end if
    } //end else if
    if !(*fs).next.is_null() { ScaleFuzzySeperatorBalanceRange_r((*fs).next, scale); }
} //end of the function ScaleFuzzySeperatorBalanceRange_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn ScaleFuzzyBalanceRange(config: *mut weightconfig_t, mut scale: f32) {
    let mut i: c_int;

    if scale < 0.0 { scale = 0.0; }
    else if scale > 100.0 { scale = 100.0; }
    i = 0;
    while i < (*config).numweights {
        ScaleFuzzySeperatorBalanceRange_r((*config).weights[i as usize].firstseperator, scale);
        i += 1;
    } //end for
} //end of the function ScaleFuzzyBalanceRange

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn InterbreedFuzzySeperator_r(fs1: *mut fuzzyseperator_t, fs2: *mut fuzzyseperator_t,
                                    fsout: *mut fuzzyseperator_t) -> c_int {
    if !(*fs1).child.is_null() {
        if (*fs2).child.is_null() || (*fsout).child.is_null() {
            (*core::ptr::addr_of_mut!(botimport)).Print(1, b"cannot interbreed weight configs, unequal child\n\0".as_ptr() as *const c_char);
            return qfalse;
        } //end if
        if InterbreedFuzzySeperator_r((*fs2).child, (*fs2).child, (*fsout).child) == 0 {
            return qfalse;
        } //end if
    } //end if
    else if (*fs1).r#type == WT_BALANCE {
        if (*fs2).r#type != WT_BALANCE || (*fsout).r#type != WT_BALANCE {
            (*core::ptr::addr_of_mut!(botimport)).Print(1, b"cannot interbreed weight configs, unequal balance\n\0".as_ptr() as *const c_char);
            return qfalse;
        } //end if
        (*fsout).weight = ((*fs1).weight + (*fs2).weight) / 2.0;
        if (*fsout).weight > (*fsout).maxweight { (*fsout).maxweight = (*fsout).weight; }
        if (*fsout).weight > (*fsout).minweight { (*fsout).minweight = (*fsout).weight; }
    } //end else if
    if !(*fs1).next.is_null() {
        if (*fs2).next.is_null() || (*fsout).next.is_null() {
            (*core::ptr::addr_of_mut!(botimport)).Print(1, b"cannot interbreed weight configs, unequal next\n\0".as_ptr() as *const c_char);
            return qfalse;
        } //end if
        if InterbreedFuzzySeperator_r((*fs1).next, (*fs2).next, (*fsout).next) == 0 {
            return qfalse;
        } //end if
    } //end if
    return qtrue;
} //end of the function InterbreedFuzzySeperator_r

//===========================================================================
// config1 and config2 are interbreeded and stored in configout
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn InterbreedWeightConfigs(config1: *mut weightconfig_t, config2: *mut weightconfig_t,
                                    configout: *mut weightconfig_t) {
    let mut i: c_int;

    if (*config1).numweights != (*config2).numweights ||
        (*config1).numweights != (*configout).numweights {
        (*core::ptr::addr_of_mut!(botimport)).Print(1, b"cannot interbreed weight configs, unequal numweights\n\0".as_ptr() as *const c_char);
        return;
    } //end if
    i = 0;
    while i < (*config1).numweights {
        InterbreedFuzzySeperator_r((*config1).weights[i as usize].firstseperator,
                                    (*config2).weights[i as usize].firstseperator,
                                    (*configout).weights[i as usize].firstseperator);
        i += 1;
    } //end for
} //end of the function InterbreedWeightConfigs

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn BotShutdownWeights() {
    let mut i: c_int;

    i = 0;
    while i < MAX_WEIGHT_FILES as c_int {
        if !*core::ptr::addr_of!(weightFileList[i as usize]).is_null() {
            FreeWeightConfig2(*core::ptr::addr_of!(weightFileList[i as usize]));
            *core::ptr::addr_of_mut!(weightFileList[i as usize]) = core::ptr::null_mut();
        } //end if
        i += 1;
    } //end for
} //end of the function BotShutdownWeights

// Stubs for functions referenced in disabled code
#[cfg(feature = "unused")]
unsafe fn random() -> f32 { 0.0 }

#[cfg(feature = "unused")]
unsafe fn crandom() -> f32 { 0.0 }

// Stubs for external functions needed
extern "C" {
    fn random() -> f32;
    fn crandom() -> f32;
}

#[cfg(feature = "debug")]
extern "C" {
    static bot_developer: c_int;
}
