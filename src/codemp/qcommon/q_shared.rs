// Copyright (C) 1999-2000 Id Software, Inc.
//
// q_shared.c -- stateless support routines that are included in each code dll

use core::ffi::{c_char, c_int, c_void};

extern "C" {
    // From q_shared.h - declarations for external functions
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strncpy(dest: *mut c_char, src: *const c_char, count: usize) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn atoi(nptr: *const c_char) -> c_int;
    fn atof(nptr: *const c_char) -> f32;
    fn tolower(c: c_int) -> c_int;
    fn toupper(c: c_int) -> c_int;
    fn vsprintf(buf: *mut c_char, fmt: *const c_char, ap: core::ffi::VaList) -> c_int;
    fn vsnprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ap: core::ffi::VaList) -> c_int;

    fn Com_Error(level: c_int, error: *const c_char, ...);
    fn Com_Printf(msg: *const c_char, ...);
}

// -------------------------
// GetIDForString
// -------------------------

#[repr(C)]
pub struct stringID_table_t {
    pub name: *const c_char,
    pub id: c_int,
}

pub fn GetIDForString(table: *mut stringID_table_t, string: *const c_char) -> c_int {
    let mut index = 0;

    unsafe {
        while !(*table.add(index)).name.is_null() && *(*table.add(index)).name != 0 {
            if Q_stricmp((*table.add(index)).name, string) == 0 {
                return (*table.add(index)).id;
            }

            index += 1;
        }
    }

    -1
}

// -------------------------
// GetStringForID
// -------------------------

pub fn GetStringForID(table: *mut stringID_table_t, id: c_int) -> *const c_char {
    let mut index = 0;

    unsafe {
        while !(*table.add(index)).name.is_null() && *(*table.add(index)).name != 0 {
            if (*table.add(index)).id == id {
                return (*table.add(index)).name;
            }

            index += 1;
        }
    }

    core::ptr::null()
}

pub fn Com_Clampi(min: c_int, max: c_int, value: c_int) -> c_int {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

pub fn Com_Clamp(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

// ============
// COM_SkipPath
// ============
pub fn COM_SkipPath(mut pathname: *mut c_char) -> *mut c_char {
    let mut last: *mut c_char;

    last = pathname;
    unsafe {
        while *pathname != 0 {
            if *pathname == b'/' as c_char {
                last = pathname.add(1);
            }
            pathname = pathname.add(1);
        }
    }
    last
}

// ============
// COM_StripExtension
// ============
pub fn COM_StripExtension(input: *const c_char, output: *mut c_char) {
    let mut in_ptr = input;
    let mut out_ptr = output;

    unsafe {
        while *in_ptr != 0 && *in_ptr != b'.' as c_char {
            *out_ptr = *in_ptr;
            out_ptr = out_ptr.add(1);
            in_ptr = in_ptr.add(1);
        }
        *out_ptr = 0;
    }
}

// ==================
// COM_DefaultExtension
// ==================
const MAX_QPATH: usize = 64;

pub fn COM_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char) {
    let mut oldPath: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut src: *mut c_char;

    // if path doesn't have a .EXT, append extension
    // (extension should include the .)
    unsafe {
        src = path.add(strlen(path).saturating_sub(1));

        while *src != b'/' as c_char && src != path {
            if *src == b'.' as c_char {
                return; // it has an extension
            }
            src = src.sub(1);
        }

        Q_strncpyz(oldPath.as_mut_ptr(), path, core::mem::size_of_val(&oldPath) as c_int);
        Com_sprintf(
            path,
            maxSize,
            b"%s%s\0".as_ptr() as *const c_char,
            oldPath.as_ptr(),
            extension,
        );
    }
}

// ============================================================================
//
// BYTE ORDER FUNCTIONS
//
// ============================================================================

/*
// can't just use function pointers, or dll linkage can
// mess up when qcommon is included in multiple places
static short	(*_BigShort) (short l);
static short	(*_LittleShort) (short l);
static int		(*_BigLong) (int l);
static int		(*_LittleLong) (int l);
static qint64	(*_BigLong64) (qint64 l);
static qint64	(*_LittleLong64) (qint64 l);
static float	(*_BigFloat) (const float *l);
static float	(*_LittleFloat) (const float *l);

short	BigShort(short l){return _BigShort(l);}
short	LittleShort(short l) {return _LittleShort(l);}
int		BigLong (int l) {return _BigLong(l);}
int		LittleLong (int l) {return _LittleLong(l);}
qint64 	BigLong64 (qint64 l) {return _BigLong64(l);}
qint64 	LittleLong64 (qint64 l) {return _LittleLong64(l);}
float	BigFloat (const float *l) {return _BigFloat(l);}
float	LittleFloat (const float *l) {return _LittleFloat(l);}
*/

pub fn ShortSwap(mut l: i16) -> i16 {
    let b1: u8;
    let b2: u8;

    b1 = (l as u16 & 255) as u8;
    b2 = ((l as u16 >> 8) & 255) as u8;

    ((b1 as i16) << 8) + b2 as i16
}

pub fn ShortNoSwap(l: i16) -> i16 {
    l
}

pub fn LongSwap(mut l: c_int) -> c_int {
    let b1: u8;
    let b2: u8;
    let b3: u8;
    let b4: u8;

    b1 = (l as u32 & 255) as u8;
    b2 = ((l as u32 >> 8) & 255) as u8;
    b3 = ((l as u32 >> 16) & 255) as u8;
    b4 = ((l as u32 >> 24) & 255) as u8;

    ((b1 as u32) << 24) as c_int + ((b2 as u32) << 16) as c_int + ((b3 as u32) << 8) as c_int + b4 as c_int
}

pub fn LongNoSwap(l: c_int) -> c_int {
    l
}

#[repr(C)]
pub struct qint64 {
    pub b0: u8,
    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
    pub b4: u8,
    pub b5: u8,
    pub b6: u8,
    pub b7: u8,
}

pub fn Long64Swap(mut ll: qint64) -> qint64 {
    let result = qint64 {
        b0: ll.b7,
        b1: ll.b6,
        b2: ll.b5,
        b3: ll.b4,
        b4: ll.b3,
        b5: ll.b2,
        b6: ll.b1,
        b7: ll.b0,
    };

    result
}

pub fn Long64NoSwap(ll: qint64) -> qint64 {
    ll
}

#[repr(C)]
pub union FloatByteUnion {
    pub f: f32,
    pub i: u32,
}

pub fn FloatSwap(f: *const f32) -> f32 {
    unsafe {
        let in_val = f as *const FloatByteUnion;
        let mut out: FloatByteUnion = FloatByteUnion { i: 0 };

        out.i = LongSwap((*in_val).i as c_int) as u32;

        out.f
    }
}

pub fn FloatNoSwap(f: *const f32) -> f32 {
    unsafe { *f }
}

// ================
// Swap_Init
// ================
/*
void Swap_Init (void)
{
    byte	swaptest[2] = {1,0};

// set the byte swapping variables in a portable manner
    if ( *(short *)swaptest == 1)
    {
        _BigShort = ShortSwap;
        _LittleShort = ShortNoSwap;
        _BigLong = LongSwap;
        _LittleLong = LongNoSwap;
        _BigLong64 = Long64Swap;
        _LittleLong64 = Long64NoSwap;
        _BigFloat = FloatSwap;
        _LittleFloat = FloatNoSwap;
    }
    else
    {
        _BigShort = ShortNoSwap;
        _LittleShort = ShortSwap;
        _BigLong = LongNoSwap;
        _LittleLong = LongSwap;
        _BigLong64 = Long64NoSwap;
        _LittleLong64 = Long64Swap;
        _BigFloat = FloatNoSwap;
        _LittleFloat = FloatSwap;
    }

}
*/

// ============================================================================
//
// PARSING
//
// ============================================================================

const MAX_TOKEN_CHARS: usize = 1024;

static mut com_token: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
static mut com_parsename: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
static mut com_lines: c_int = 0;

pub fn COM_BeginParseSession(name: *const c_char) {
    unsafe {
        com_lines = 0;
        Com_sprintf(
            com_parsename.as_mut_ptr(),
            core::mem::size_of_val(&com_parsename) as c_int,
            b"%s\0".as_ptr() as *const c_char,
            name,
        );
    }
}

pub fn COM_GetCurrentParseLine() -> c_int {
    unsafe { com_lines }
}

pub fn COM_Parse(data_p: *mut *const c_char) -> *mut c_char {
    unsafe { COM_ParseExt(data_p, 1) }
}

pub fn COM_ParseError(format: *mut c_char, _args: ...) {
    unsafe {
        let mut argptr: core::ffi::VaList;
        let mut string: [c_char; 4096] = [0; 4096];

        core::arch::x86_64::_va_arg();
    }
}

pub fn COM_ParseWarning(format: *mut c_char, _args: ...) {
    unsafe {
        let mut string: [c_char; 4096] = [0; 4096];
    }
}

// ==============
// COM_Parse
//
// Parse a token out of a string
// Will never return NULL, just empty strings
//
// If "allowLineBreaks" is qtrue then an empty
// string will be returned if the next token is
// a newline.
// ==============

pub fn SkipWhitespace(mut data: *const c_char, hasNewLines: *mut c_int) -> *const c_char {
    unsafe {
        loop {
            let c = *data as u32;
            if c > b' ' as u32 {
                break;
            }
            if c == 0 {
                return core::ptr::null();
            }
            if c == b'\n' as u32 {
                com_lines += 1;
                *hasNewLines = 1;
            }
            data = data.add(1);
        }
        data
    }
}

pub fn COM_Compress(data_p: *mut c_char) -> c_int {
    let mut in_ptr: *mut c_char;
    let mut out_ptr: *mut c_char;
    let mut c: c_int;
    let mut newline: c_int = 0;
    let mut whitespace: c_int = 0;

    in_ptr = data_p;
    out_ptr = data_p;
    unsafe {
        if !in_ptr.is_null() {
            loop {
                c = *in_ptr as c_int & 0xff;
                if c == 0 {
                    break;
                }
                // skip double slash comments
                if c == b'/' as c_int && *in_ptr.add(1) as c_int == b'/' as c_int {
                    loop {
                        if *in_ptr == 0 || *in_ptr == b'\n' as c_char {
                            break;
                        }
                        in_ptr = in_ptr.add(1);
                    }
                    // skip /* */ comments
                } else if c == b'/' as c_int && *in_ptr.add(1) as c_int == b'*' as c_int {
                    loop {
                        if *in_ptr == 0 || (*in_ptr == b'*' as c_char && *in_ptr.add(1) == b'/' as c_char) {
                            break;
                        }
                        in_ptr = in_ptr.add(1);
                    }
                    if *in_ptr != 0 {
                        in_ptr = in_ptr.add(2);
                    }
                    // record when we hit a newline
                } else if c == b'\n' as c_int || c == b'\r' as c_int {
                    newline = 1;
                    in_ptr = in_ptr.add(1);
                    // record when we hit whitespace
                } else if c == b' ' as c_int || c == b'\t' as c_int {
                    whitespace = 1;
                    in_ptr = in_ptr.add(1);
                    // an actual token
                } else {
                    // if we have a pending newline, emit it (and it counts as whitespace)
                    if newline != 0 {
                        *out_ptr = b'\n' as c_char;
                        out_ptr = out_ptr.add(1);
                        newline = 0;
                        whitespace = 0;
                    }
                    if whitespace != 0 {
                        *out_ptr = b' ' as c_char;
                        out_ptr = out_ptr.add(1);
                        whitespace = 0;
                    }

                    // copy quoted strings unmolested
                    if c == b'"' as c_int {
                        *out_ptr = c as c_char;
                        out_ptr = out_ptr.add(1);
                        in_ptr = in_ptr.add(1);
                        loop {
                            c = *in_ptr as c_int & 0xff;
                            if c != 0 && c != b'"' as c_int {
                                *out_ptr = c as c_char;
                                out_ptr = out_ptr.add(1);
                                in_ptr = in_ptr.add(1);
                            } else {
                                break;
                            }
                        }
                        if c == b'"' as c_int {
                            *out_ptr = c as c_char;
                            out_ptr = out_ptr.add(1);
                            in_ptr = in_ptr.add(1);
                        }
                    } else {
                        *out_ptr = c as c_char;
                        out_ptr = out_ptr.add(1);
                        in_ptr = in_ptr.add(1);
                    }
                }
            }
        }
        *out_ptr = 0;
        out_ptr as usize - data_p as usize
    }
}

pub fn COM_ParseExt(data_p: *mut *const c_char, allowLineBreaks: c_int) -> *mut c_char {
    let mut c: c_int = 0;
    let mut len: c_int;
    let mut hasNewLines: c_int = 0;
    let mut data: *const c_char;

    unsafe {
        data = *data_p;
        len = 0;
        com_token[0] = 0;

        // make sure incoming data is valid
        if data.is_null() {
            *data_p = core::ptr::null();
            return com_token.as_mut_ptr();
        }

        loop {
            // skip whitespace
            data = SkipWhitespace(data, &mut hasNewLines);
            if data.is_null() {
                *data_p = core::ptr::null();
                return com_token.as_mut_ptr();
            }
            if hasNewLines != 0 && allowLineBreaks == 0 {
                *data_p = data;
                return com_token.as_mut_ptr();
            }

            c = *data as c_int & 0xff;

            // skip double slash comments
            if c == b'/' as c_int && *data.add(1) as c_int == b'/' as c_int {
                data = data.add(2);
                loop {
                    if *data == 0 || *data == b'\n' as c_char {
                        break;
                    }
                    data = data.add(1);
                }
            }
            // skip /* */ comments
            else if c == b'/' as c_int && *data.add(1) as c_int == b'*' as c_int {
                data = data.add(2);
                loop {
                    if *data == 0 || (*data == b'*' as c_char && *data.add(1) == b'/' as c_char) {
                        break;
                    }
                    data = data.add(1);
                }
                if *data != 0 {
                    data = data.add(2);
                }
            } else {
                break;
            }
        }

        // handle quoted strings
        if c == b'"' as c_int {
            data = data.add(1);
            loop {
                c = *data as c_int & 0xff;
                data = data.add(1);
                if c == b'"' as c_int || c == 0 {
                    com_token[len as usize] = 0;
                    *data_p = data;
                    return com_token.as_mut_ptr();
                }
                if len < MAX_TOKEN_CHARS as c_int - 1 {
                    com_token[len as usize] = c as c_char;
                    len += 1;
                }
            }
        }

        // parse a regular word
        loop {
            if len < MAX_TOKEN_CHARS as c_int - 1 {
                com_token[len as usize] = c as c_char;
                len += 1;
            }
            data = data.add(1);
            c = *data as c_int & 0xff;
            if c == b'\n' as c_int {
                com_lines += 1;
            }
            if c <= 32 {
                break;
            }
        }

        if len == MAX_TOKEN_CHARS as c_int - 1 {
            //		Com_Printf ("Token exceeded %i chars, discarded.\n", MAX_TOKEN_CHARS);
            len = 0;
        }
        com_token[len as usize] = 0;

        *data_p = data;
        com_token.as_mut_ptr()
    }
}

// #if 0
// // no longer used
// /*
// ===============
// COM_ParseInfos
// ===============
// */
// int COM_ParseInfos( char *buf, int max, char infos[][MAX_INFO_STRING] ) {
// ...
// }
// #endif

// ===============
// COM_ParseString
// ===============
pub fn COM_ParseString(data: *mut *const c_char, s: *mut *const c_char) -> c_int {
    unsafe {
        //	*s = COM_ParseExt( data, qtrue );
        *s = COM_ParseExt(data, 0);
        if (**s) as u32 == 0 {
            Com_Printf(b"unexpected EOF\n\0".as_ptr() as *const c_char);
            return 1;
        }
        0
    }
}

// ===============
// COM_ParseInt
// ===============
pub fn COM_ParseInt(data: *mut *const c_char, i: *mut c_int) -> c_int {
    let token: *const c_char;

    unsafe {
        token = COM_ParseExt(data, 0);
        if *token as u32 == 0 {
            Com_Printf(b"unexpected EOF\n\0".as_ptr() as *const c_char);
            return 1;
        }

        *i = atoi(token);
        0
    }
}

// ===============
// COM_ParseFloat
// ===============
pub fn COM_ParseFloat(data: *mut *const c_char, f: *mut f32) -> c_int {
    let token: *const c_char;

    unsafe {
        token = COM_ParseExt(data, 0);
        if *token as u32 == 0 {
            Com_Printf(b"unexpected EOF\n\0".as_ptr() as *const c_char);
            return 1;
        }

        *f = atof(token);
        0
    }
}

// ===============
// COM_ParseVec4
// ===============
pub fn COM_ParseVec4(buffer: *mut *const c_char, c: *mut [f32; 4]) -> c_int {
    let mut i: c_int;
    let mut f: f32;

    i = 0;
    while i < 4 {
        if COM_ParseFloat(buffer, &mut f) != 0 {
            return 1;
        }
        unsafe {
            (*c)[i as usize] = f;
        }
        i += 1;
    }
    0
}

// ==================
// COM_MatchToken
// ==================
pub fn COM_MatchToken(buf_p: *mut *const c_char, match_str: *mut c_char) {
    let token: *const c_char;

    unsafe {
        token = COM_Parse(buf_p);
        if strcmp(token as *const c_char, match_str as *const c_char) != 0 {
            Com_Error(
                2, // ERR_DROP
                b"MatchToken: %s != %s\0".as_ptr() as *const c_char,
                token,
                match_str,
            );
        }
    }
}

// simple strcmp for extern context
fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut p1 = s1;
        let mut p2 = s2;
        loop {
            let c1 = *p1 as u8;
            let c2 = *p2 as u8;
            if c1 != c2 {
                return (c1 as c_int) - (c2 as c_int);
            }
            if c1 == 0 {
                break;
            }
            p1 = p1.add(1);
            p2 = p2.add(1);
        }
        0
    }
}

// =================
// SkipBracedSection
//
// The next token should be an open brace.
// Skips until a matching close brace is found.
// Internal brace depths are properly skipped.
// =================
pub fn SkipBracedSection(program: *mut *const c_char) {
    let mut token: *mut c_char;
    let mut depth: c_int;

    depth = 0;
    loop {
        unsafe {
            token = COM_ParseExt(program, 1);
            if *token.add(1) as u32 == 0 {
                if *token as u32 == b'{' as u32 {
                    depth += 1;
                } else if *token as u32 == b'}' as u32 {
                    depth -= 1;
                }
            }
            if depth == 0 || (*program).is_null() {
                break;
            }
        }
    }
}

// =================
// SkipRestOfLine
// =================
pub fn SkipRestOfLine(data: *mut *const c_char) {
    let mut p: *const c_char;
    let mut c: c_int;

    unsafe {
        p = *data;
        loop {
            c = *p as c_int & 0xff;
            p = p.add(1);
            if c == 0 {
                break;
            }
            if c == b'\n' as c_int {
                com_lines += 1;
                break;
            }
        }

        *data = p;
    }
}

pub fn Parse1DMatrix(buf_p: *mut *const c_char, x: c_int, m: *mut f32) {
    let mut token: *const c_char;
    let mut i: c_int;

    unsafe {
        COM_MatchToken(buf_p, b"(\0".as_ptr() as *mut c_char);

        i = 0;
        while i < x {
            token = COM_Parse(buf_p);
            *m.add(i as usize) = atof(token);
            i += 1;
        }

        COM_MatchToken(buf_p, b")\0".as_ptr() as *mut c_char);
    }
}

pub fn Parse2DMatrix(buf_p: *mut *const c_char, y: c_int, x: c_int, m: *mut f32) {
    let mut i: c_int;

    unsafe {
        COM_MatchToken(buf_p, b"(\0".as_ptr() as *mut c_char);

        i = 0;
        while i < y {
            Parse1DMatrix(buf_p, x, m.add((i as usize) * (x as usize)));
            i += 1;
        }

        COM_MatchToken(buf_p, b")\0".as_ptr() as *mut c_char);
    }
}

pub fn Parse3DMatrix(buf_p: *mut *const c_char, z: c_int, y: c_int, x: c_int, m: *mut f32) {
    let mut i: c_int;

    unsafe {
        COM_MatchToken(buf_p, b"(\0".as_ptr() as *mut c_char);

        i = 0;
        while i < z {
            Parse2DMatrix(buf_p, y, x, m.add((i as usize) * (x as usize) * (y as usize)));
            i += 1;
        }

        COM_MatchToken(buf_p, b")\0".as_ptr() as *mut c_char);
    }
}

// ============================================================================
//
// LIBRARY REPLACEMENT FUNCTIONS
//
// ============================================================================

pub fn Q_isprint(c: c_int) -> c_int {
    if c >= 0x20 && c <= 0x7E {
        return 1;
    }
    0
}

pub fn Q_islower(c: c_int) -> c_int {
    if c >= b'a' as c_int && c <= b'z' as c_int {
        return 1;
    }
    0
}

pub fn Q_isupper(c: c_int) -> c_int {
    if c >= b'A' as c_int && c <= b'Z' as c_int {
        return 1;
    }
    0
}

pub fn Q_isalpha(c: c_int) -> c_int {
    if (c >= b'a' as c_int && c <= b'z' as c_int) || (c >= b'A' as c_int && c <= b'Z' as c_int) {
        return 1;
    }
    0
}

pub fn Q_strrchr(string: *const c_char, c: c_int) -> *mut c_char {
    let cc = c as c_char;
    let mut s: *mut c_char;
    let mut sp: *mut c_char;

    s = string as *mut c_char;
    sp = core::ptr::null_mut();

    unsafe {
        while *s != 0 {
            if *s == cc {
                sp = s;
            }
            s = s.add(1);
        }
        if cc as c_int == 0 {
            sp = s;
        }
    }

    sp
}

// =============
// Q_strncpyz
//
// Safe strncpy that ensures a trailing zero
// =============
pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int) {
    // bk001129 - also NULL dest
    if dest.is_null() {
        unsafe {
            Com_Error(1, b"Q_strncpyz: NULL dest\0".as_ptr() as *const c_char);
        }
    }
    if src.is_null() {
        unsafe {
            Com_Error(1, b"Q_strncpyz: NULL src\0".as_ptr() as *const c_char);
        }
    }
    if destsize < 1 {
        unsafe {
            Com_Error(1, b"Q_strncpyz: destsize < 1\0".as_ptr() as *const c_char);
        }
    }

    unsafe {
        strncpy(dest, src, (destsize - 1) as usize);
        *dest.add((destsize - 1) as usize) = 0;
    }
}

pub fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int {
    let mut c1: c_int;
    let mut c2: c_int;
    let mut n_mut = n;

    // bk001129 - moved in 1.17 fix not in id codebase
    unsafe {
        if s1.is_null() {
            if s2.is_null() {
                return 0;
            } else {
                return -1;
            }
        } else if s2.is_null() {
            return 1;
        }

        loop {
            c1 = *s1 as c_int & 0xff;
            s1 = s1.add(1);
            c2 = *s2 as c_int & 0xff;
            s2 = s2.add(1);

            n_mut -= 1;
            if n_mut == 0 {
                return 0; // strings are equal until end point
            }

            if c1 != c2 {
                if c1 >= b'a' as c_int && c1 <= b'z' as c_int {
                    c1 -= b'a' as c_int - b'A' as c_int;
                }
                if c2 >= b'a' as c_int && c2 <= b'z' as c_int {
                    c2 -= b'a' as c_int - b'A' as c_int;
                }
                if c1 != c2 {
                    return if c1 < c2 { -1 } else { 1 };
                }
            }
            if c1 == 0 {
                break;
            }
        }

        0 // strings are equal
    }
}

pub fn Q_strncmp(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int {
    let mut c1: c_int;
    let mut c2: c_int;
    let mut n_mut = n;

    unsafe {
        loop {
            c1 = *s1 as c_int & 0xff;
            s1 = s1.add(1);
            c2 = *s2 as c_int & 0xff;
            s2 = s2.add(1);

            n_mut -= 1;
            if n_mut == 0 {
                return 0; // strings are equal until end point
            }

            if c1 != c2 {
                return if c1 < c2 { -1 } else { 1 };
            }
            if c1 == 0 {
                break;
            }
        }

        0 // strings are equal
    }
}

pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    if !s1.is_null() && !s2.is_null() {
        Q_stricmpn(s1, s2, 99999)
    } else {
        -1
    }
}

pub fn Q_strlwr(s1: *mut c_char) -> *mut c_char {
    let mut s: *mut c_char;

    s = s1;
    unsafe {
        while *s != 0 {
            *s = tolower(*s as c_int) as c_char;
            s = s.add(1);
        }
    }
    s1
}

pub fn Q_strupr(s1: *mut c_char) -> *mut c_char {
    let mut s: *mut c_char;

    s = s1;
    unsafe {
        while *s != 0 {
            *s = toupper(*s as c_int) as c_char;
            s = s.add(1);
        }
    }
    s1
}

// never goes past bounds or leaves without a terminating 0
pub fn Q_strcat(dest: *mut c_char, size: c_int, src: *const c_char) {
    let l1: usize;

    unsafe {
        l1 = strlen(dest);
        if l1 as c_int >= size {
            Com_Error(1, b"Q_strcat: already overflowed\0".as_ptr() as *const c_char);
        }
        Q_strncpyz(dest.add(l1), src, size - l1 as c_int);
    }
}

pub fn Q_PrintStrlen(string: *const c_char) -> c_int {
    let mut len: c_int;
    let mut p: *const c_char;

    if string.is_null() {
        return 0;
    }

    len = 0;
    p = string;
    unsafe {
        while *p != 0 {
            if Q_IsColorString(p) != 0 {
                p = p.add(2);
                continue;
            }
            p = p.add(1);
            len += 1;
        }
    }

    len
}

pub fn Q_CleanStr(string: *mut c_char) -> *mut c_char {
    let mut d: *mut c_char;
    let mut s: *mut c_char;
    let mut c: c_int;

    s = string;
    d = string;
    unsafe {
        loop {
            c = *s as c_int & 0xff;
            if c == 0 {
                break;
            }
            if Q_IsColorString(s) != 0 {
                s = s.add(1);
            } else if c >= 0x20 && c <= 0x7E {
                *d = c as c_char;
                d = d.add(1);
            }
            s = s.add(1);
        }
        *d = 0 as c_char;
    }

    string
}

// Macro stub for Q_IsColorString - check if this is a color string token
fn Q_IsColorString(p: *const c_char) -> c_int {
    // Q_COLOR_ESCAPE is '^'
    // #define Q_IsColorString(p)	( p && *(p) == Q_COLOR_ESCAPE && *((p)+1) && *((p)+1) != Q_COLOR_ESCAPE && *((p)+1) <= '7' && *((p)+1) >= '0' )
    unsafe {
        if p.is_null() {
            return 0;
        }
        let c1 = *p;
        if c1 != b'^' as c_char {
            return 0;
        }
        let c2 = *p.add(1);
        if c2 as u32 == 0 {
            return 0;
        }
        if c2 == b'^' as c_char {
            return 0;
        }
        if (c2 as u8) <= b'7' && (c2 as u8) >= b'0' {
            return 1;
        }
        0
    }
}

pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, _args: ...) {
    // Note: This is a simplified stub - actual implementation would need varargs handling
    // For now we just set dest to empty to avoid undefined behavior
    unsafe {
        *dest = 0;
    }
}

// ============
// va
//
// does a varargs printf into a temp buffer, so I don't need to have
// varargs versions of all text functions.
// FIXME: make this buffer size safe someday
// ============

pub fn va(format: *const c_char, _args: ...) -> *mut c_char {
    // Note: This is a simplified stub
    unsafe {
        // Return a static buffer; in real implementation this would handle varargs
        static mut string: [[c_char; 32000]; 2] = [[0; 32000]; 2];
        static mut index: c_int = 0;
        let buf: *mut c_char;

        buf = string[(index & 1) as usize].as_mut_ptr();
        index += 1;

        // In a real implementation, we'd do:
        // va_start(argptr, format);
        // vsprintf(buf, format, argptr);
        // va_end(argptr);

        buf
    }
}

// =====================================================================
//
// INFO STRINGS
//
// =====================================================================

// ===============
// Info_ValueForKey
//
// Searches the string for the given
// key and returns the associated value, or an empty string.
// FIXME: overflow check?
// ===============

const BIG_INFO_KEY: usize = 8192;
const BIG_INFO_VALUE: usize = 8192;

pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char {
    let mut pkey: [c_char; BIG_INFO_KEY] = [0; BIG_INFO_KEY];
    static mut value: [[c_char; BIG_INFO_VALUE]; 2] = [[0; BIG_INFO_VALUE]; 2];
    static mut valueindex: c_int = 0;
    let mut o: *mut c_char;

    unsafe {
        if s.is_null() || key.is_null() {
            return b"\0".as_ptr() as *mut c_char;
        }

        if strlen(s) >= 8192 {
            // BIG_INFO_STRING
            Com_Error(2, b"Info_ValueForKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        valueindex ^= 1;
        let mut s_ptr = s;
        if *s_ptr == b'\\' as c_char {
            s_ptr = s_ptr.add(1);
        }
        loop {
            o = pkey.as_mut_ptr();
            while *s_ptr != b'\\' as c_char {
                if *s_ptr == 0 {
                    return b"\0".as_ptr() as *mut c_char;
                }
                *o = *s_ptr;
                o = o.add(1);
                s_ptr = s_ptr.add(1);
            }
            *o = 0;
            s_ptr = s_ptr.add(1);

            o = value[valueindex as usize].as_mut_ptr();

            while *s_ptr != b'\\' as c_char && *s_ptr != 0 {
                *o = *s_ptr;
                o = o.add(1);
                s_ptr = s_ptr.add(1);
            }
            *o = 0;

            if Q_stricmp(key, pkey.as_ptr()) == 0 {
                return value[valueindex as usize].as_mut_ptr();
            }

            if *s_ptr == 0 {
                break;
            }
            s_ptr = s_ptr.add(1);
        }

        b"\0".as_ptr() as *mut c_char
    }
}

// ===================
// Info_NextPair
//
// Used to itterate through all the key/value pairs in an info string
// ===================
pub fn Info_NextPair(head: *mut *const c_char, key: *mut c_char, value: *mut c_char) {
    let mut o: *mut c_char;
    let mut s: *const c_char;

    unsafe {
        s = *head;

        if *s == b'\\' as c_char {
            s = s.add(1);
        }
        *key = 0;
        *value = 0;

        o = key;
        while *s != b'\\' as c_char {
            if *s == 0 {
                *o = 0;
                *head = s;
                return;
            }
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;
        s = s.add(1);

        o = value;
        while *s != b'\\' as c_char && *s != 0 {
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;

        *head = s;
    }
}

// ===================
// Info_RemoveKey
// ===================

const MAX_INFO_STRING: usize = 1024;
const MAX_INFO_KEY: usize = 1024;
const MAX_INFO_VALUE: usize = 1024;

pub fn Info_RemoveKey(s: *mut c_char, key: *const c_char) {
    let mut start: *mut c_char;
    let mut pkey: [c_char; MAX_INFO_KEY] = [0; MAX_INFO_KEY];
    let mut value: [c_char; MAX_INFO_VALUE] = [0; MAX_INFO_VALUE];
    let mut o: *mut c_char;

    unsafe {
        if strlen(s) >= MAX_INFO_STRING {
            Com_Error(2, b"Info_RemoveKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        if !strchr(key, b'\\' as c_int).is_null() {
            return;
        }

        loop {
            start = s;
            if *s == b'\\' as c_char {
                s = s.add(1);
            }
            o = pkey.as_mut_ptr();
            while *s != b'\\' as c_char {
                if *s == 0 {
                    return;
                }
                *o = *s;
                o = o.add(1);
                s = s.add(1);
            }
            *o = 0;
            s = s.add(1);

            o = value.as_mut_ptr();
            while *s != b'\\' as c_char && *s != 0 {
                if *s == 0 {
                    return;
                }
                *o = *s;
                o = o.add(1);
                s = s.add(1);
            }
            *o = 0;

            if strcmp(key as *const c_char, pkey.as_ptr()) == 0 {
                strcpy(start, s); // remove this part
                return;
            }

            if *s == 0 {
                return;
            }
        }
    }
}

// ===================
// Info_RemoveKey_Big
// ===================
pub fn Info_RemoveKey_Big(s: *mut c_char, key: *const c_char) {
    let mut start: *mut c_char;
    let mut pkey: [c_char; BIG_INFO_KEY] = [0; BIG_INFO_KEY];
    let mut value: [c_char; BIG_INFO_VALUE] = [0; BIG_INFO_VALUE];
    let mut o: *mut c_char;

    unsafe {
        if strlen(s) >= 8192 {
            // BIG_INFO_STRING
            Com_Error(2, b"Info_RemoveKey_Big: oversize infostring\0".as_ptr() as *const c_char);
        }

        if !strchr(key, b'\\' as c_int).is_null() {
            return;
        }

        loop {
            start = s;
            if *s == b'\\' as c_char {
                s = s.add(1);
            }
            o = pkey.as_mut_ptr();
            while *s != b'\\' as c_char {
                if *s == 0 {
                    return;
                }
                *o = *s;
                o = o.add(1);
                s = s.add(1);
            }
            *o = 0;
            s = s.add(1);

            o = value.as_mut_ptr();
            while *s != b'\\' as c_char && *s != 0 {
                if *s == 0 {
                    return;
                }
                *o = *s;
                o = o.add(1);
                s = s.add(1);
            }
            *o = 0;

            if strcmp(key as *const c_char, pkey.as_ptr()) == 0 {
                strcpy(start, s); // remove this part
                return;
            }

            if *s == 0 {
                return;
            }
        }
    }
}

// ==================
// Info_Validate
//
// Some characters are illegal in info strings because they
// can mess up the server's parsing
// ==================
pub fn Info_Validate(s: *const c_char) -> c_int {
    unsafe {
        if !strchr(s, b'"' as c_int).is_null() {
            return 0;
        }
        if !strchr(s, b';' as c_int).is_null() {
            return 0;
        }
        1
    }
}

// ==================
// Info_SetValueForKey
//
// Changes or adds a key/value pair
// ==================
pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char) {
    let mut newi: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];

    unsafe {
        if strlen(s) >= MAX_INFO_STRING {
            Com_Error(2, b"Info_SetValueForKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        if !strchr(key, b'\\' as c_int).is_null() || !strchr(value, b'\\' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a \\\n\0".as_ptr() as *const c_char);
            return;
        }

        if !strchr(key, b';' as c_int).is_null() || !strchr(value, b';' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a semicolon\n\0".as_ptr() as *const c_char);
            return;
        }

        if !strchr(key, b'"' as c_int).is_null() || !strchr(value, b'"' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a \"\n\0".as_ptr() as *const c_char);
            return;
        }

        Info_RemoveKey(s, key);
        if value.is_null() || strlen(value) == 0 {
            return;
        }

        Com_sprintf(
            newi.as_mut_ptr(),
            core::mem::size_of_val(&newi) as c_int,
            b"\\%s\\%s\0".as_ptr() as *const c_char,
            key,
            value,
        );

        if strlen(newi.as_ptr()) + strlen(s) > MAX_INFO_STRING {
            Com_Printf(b"Info string length exceeded\n\0".as_ptr() as *const c_char);
            return;
        }

        strcat(newi.as_mut_ptr(), s);
        strcpy(s, newi.as_ptr());
    }
}

// ==================
// Info_SetValueForKey_Big
//
// Changes or adds a key/value pair
// ==================
pub fn Info_SetValueForKey_Big(s: *mut c_char, key: *const c_char, value: *const c_char) {
    let mut newi: [c_char; 8192] = [0; 8192]; // BIG_INFO_STRING

    unsafe {
        if strlen(s) >= 8192 {
            // BIG_INFO_STRING
            Com_Error(2, b"Info_SetValueForKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        if !strchr(key, b'\\' as c_int).is_null() || !strchr(value, b'\\' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a \\\n\0".as_ptr() as *const c_char);
            return;
        }

        if !strchr(key, b';' as c_int).is_null() || !strchr(value, b';' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a semicolon\n\0".as_ptr() as *const c_char);
            return;
        }

        if !strchr(key, b'"' as c_int).is_null() || !strchr(value, b'"' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a \"\n\0".as_ptr() as *const c_char);
            return;
        }

        Info_RemoveKey_Big(s, key);
        if value.is_null() || strlen(value) == 0 {
            return;
        }

        Com_sprintf(
            newi.as_mut_ptr(),
            core::mem::size_of_val(&newi) as c_int,
            b"\\%s\\%s\0".as_ptr() as *const c_char,
            key,
            value,
        );

        if strlen(newi.as_ptr()) + strlen(s) > 8192 {
            // BIG_INFO_STRING
            Com_Printf(b"BIG Info string length exceeded\n\0".as_ptr() as *const c_char);
            return;
        }

        strcat(s, newi.as_ptr());
    }
}

// ====================================================================
