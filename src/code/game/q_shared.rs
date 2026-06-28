// q_shared.c -- stateless support routines that are included in each code dll

#![allow(non_snake_case)]
#![allow(static_mut_refs)]

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// External functions and types (defined elsewhere in the codebase)
// ============================================================================

extern "C" {
    fn Com_Printf(fmt: *const c_char, ...) -> c_int;
    fn Com_Error(code: c_int, fmt: *const c_char, ...) -> !;
    fn strlen(s: *const c_char) -> usize;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn atof(nptr: *const c_char) -> f32;
    fn atoi(nptr: *const c_char) -> c_int;
    fn vsprintf(str: *mut c_char, format: *const c_char, ap: *mut core::ffi::c_void) -> c_int;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_IsColorString(p: *const c_char) -> c_int;
}

// ============================================================================
// Constants
// ============================================================================

const MAX_TOKEN_CHARS: usize = 1024;
const MAX_INFO_KEY: usize = 64;
const MAX_INFO_VALUE: usize = 64;
const MAX_INFO_STRING: usize = 512;

const ERR_FATAL: c_int = 1;
const ERR_DROP: c_int = 2;

// ============================================================================
// Type definitions (stubs for types used in this file)
// ============================================================================

pub type qboolean = c_int;
pub type byte = u8;
pub type vec4_t = [f32; 4];

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

#[repr(C)]
pub struct parseData_t {
    pub com_lines: c_int,
}

#[repr(C)]
pub struct stringID_table_t {
    pub name: *const c_char,
    pub id: c_int,
}

// ============================================================================
// CLAMP FUNCTION
// ============================================================================

pub fn Com_Clamp(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

// ============================================================================
// COM_SkipPath
// ============================================================================
pub fn COM_SkipPath(pathname: *mut c_char) -> *mut c_char {
    let mut last: *mut c_char;
    let mut pathname = pathname;

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

// ============================================================================
// COM_StripExtension
// ============================================================================
pub fn COM_StripExtension(in_: *const c_char, out: *mut c_char) {
    let mut in_ptr = in_;
    let mut out_ptr = out;
    unsafe {
        while *in_ptr != 0 && *in_ptr != b'.' as c_char {
            *out_ptr = *in_ptr;
            out_ptr = out_ptr.add(1);
            in_ptr = in_ptr.add(1);
        }
        *out_ptr = 0;
    }
}

// ============================================================================
// COM_DefaultExtension
// ============================================================================
pub fn COM_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char) {
    unsafe {
        if *path != 0 {
            //
            // if path doesn't have a .EXT, append extension
            // (extension should include the .)
            //
            let mut src = path.add(strlen(path).wrapping_sub(1));

            while *src != b'/' as c_char && src != path {
                if *src == b'.' as c_char {
                    return;                 // it has an extension
                }
                src = src.sub(1);
            }
        }

        if strlen(path).wrapping_add(strlen(extension)) >= maxSize as usize {
            Com_Printf(b"COM_DefaultExtension: overflow adding %s to %s\n\0".as_ptr() as *const c_char, extension, path);
        } else {
            strcat(path, extension);
        }
    }
}

// ============================================================================
//
//                          BYTE ORDER FUNCTIONS
//
// ============================================================================

// can't just use function pointers, or dll linkage can
// mess up when qcommon is included in multiple places
static mut _BigShort: Option<fn(short) -> short> = None;
static mut _LittleShort: Option<fn(short) -> short> = None;
static mut _BigLong: Option<fn(c_int) -> c_int> = None;
static mut _LittleLong: Option<fn(c_int) -> c_int> = None;
static mut _BigFloat: Option<fn(f32) -> f32> = None;
static mut _LittleFloat: Option<fn(f32) -> f32> = None;

#[cfg(target_arch = "x86")]
pub fn BigShort(l: c_int) -> c_int {
    unsafe { _BigShort.unwrap()(l as c_int) as c_int }
}

#[cfg(target_arch = "x86")]
pub fn BigLong(l: c_int) -> c_int {
    unsafe { _BigLong.unwrap()(l) }
}

#[cfg(target_arch = "x86")]
pub fn BigFloat(l: f32) -> f32 {
    unsafe { _BigFloat.unwrap()(l) }
}

#[cfg(not(target_arch = "x86"))]
//
// standard smart-swap code...
//
pub fn BigShort(l: c_int) -> c_int {
    unsafe { _BigShort.unwrap()(l as c_int) as c_int }
}

#[cfg(not(target_arch = "x86"))]
pub fn LittleShort(l: c_int) -> c_int {
    unsafe { _LittleShort.unwrap()(l as c_int) as c_int }
}

#[cfg(not(target_arch = "x86"))]
pub fn BigLong(l: c_int) -> c_int {
    unsafe { _BigLong.unwrap()(l) }
}

#[cfg(not(target_arch = "x86"))]
pub fn LittleLong(l: c_int) -> c_int {
    unsafe { _LittleLong.unwrap()(l) }
}

#[cfg(not(target_arch = "x86"))]
pub fn BigFloat(l: f32) -> f32 {
    unsafe { _BigFloat.unwrap()(l) }
}

#[cfg(not(target_arch = "x86"))]
pub fn LittleFloat(l: f32) -> f32 {
    unsafe { _LittleFloat.unwrap()(l) }
}

pub fn ShortSwap(l: c_int) -> c_int {
    let b1 = (l & 255) as u8;
    let b2 = ((l >> 8) & 255) as u8;

    (((b1 as c_int) << 8) + (b2 as c_int))
}

pub fn ShortNoSwap(l: c_int) -> c_int {
    l
}

pub fn LongSwap(l: c_int) -> c_int {
    let b1 = (l & 255) as u8;
    let b2 = ((l >> 8) & 255) as u8;
    let b3 = ((l >> 16) & 255) as u8;
    let b4 = ((l >> 24) & 255) as u8;

    ((((b1 as c_int) << 24) + (((b2 as c_int) << 16) + (((b3 as c_int) << 8) + (b4 as c_int)))))
}

pub fn LongNoSwap(l: c_int) -> c_int {
    l
}

pub fn FloatSwap(f: f32) -> f32 {
    #[repr(C)]
    union FloatBytes {
        f: f32,
        b: [u8; 4],
    }

    let mut dat1: FloatBytes = FloatBytes { f };
    let mut dat2: FloatBytes = FloatBytes { b: [0; 4] };

    unsafe {
        dat2.b[0] = dat1.b[3];
        dat2.b[1] = dat1.b[2];
        dat2.b[2] = dat1.b[1];
        dat2.b[3] = dat1.b[0];
        dat2.f
    }
}

pub fn FloatNoSwap(f: f32) -> f32 {
    f
}

// ============================================================================
// Swap_Init
// ============================================================================
pub fn Swap_Init() {
    let swaptest: [u8; 2] = [1, 0];

    // set the byte swapping variables in a portable manner
    unsafe {
        if *(swaptest.as_ptr() as *const c_int) == 1 {
            _BigShort = Some(ShortSwap);
            _LittleShort = Some(ShortNoSwap);
            _BigLong = Some(LongSwap);
            _LittleLong = Some(LongNoSwap);
            _BigFloat = Some(FloatSwap);
            _LittleFloat = Some(FloatNoSwap);
        } else {
            _BigShort = Some(ShortNoSwap);
            _LittleShort = Some(ShortSwap);
            _BigLong = Some(LongNoSwap);
            _LittleLong = Some(LongSwap);
            _BigFloat = Some(FloatNoSwap);
            _LittleFloat = Some(FloatSwap);
        }
    }
}

// ============================================================================
//
// PARSING
//
// ============================================================================

static mut com_token: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
//JLFCALLOUT MPNOTUSED
//#include functionality for files
static mut parseDataCount: c_int = -1;
static mut parseData: [parseData_t; 2] = [
    parseData_t { com_lines: 0 },
    parseData_t { com_lines: 0 },
];

pub fn COM_ParseInit() {
    unsafe {
        core::ptr::write_bytes(core::ptr::addr_of_mut!(parseData[0]) as *mut u8, 0, core::mem::size_of::<parseData_t>());
        core::ptr::write_bytes(core::ptr::addr_of_mut!(parseData[1]) as *mut u8, 0, core::mem::size_of::<parseData_t>());
        COM_BeginParseSession();
    }
}

#[cfg(target_os = "windows")]
pub fn COM_BeginParseSession(nested: bool) {
    unsafe {
        if nested {
            parseDataCount = 1;
        } else {
            parseDataCount = 0;
        }
        parseData[parseDataCount as usize].com_lines = 1;
    }
}

#[cfg(not(target_os = "windows"))]
pub fn COM_BeginParseSession() {
    unsafe {
        parseDataCount = 0;
        parseData[parseDataCount as usize].com_lines = 1;
    }
}

pub fn COM_GetCurrentParseLine(_index: c_int) -> c_int {
    unsafe { parseData[parseDataCount as usize].com_lines }
}

pub fn COM_Parse(data_p: *mut *const c_char) -> *mut c_char {
    unsafe { COM_ParseExt(data_p, qtrue) }
}

// ============================================================================
// COM_Parse
//
// Parse a token out of a string
// Will never return NULL, just empty strings
//
// If "allowLineBreaks" is qtrue then an empty
// string will be returned if the next token is
// a newline.
// ============================================================================
unsafe fn SkipWhitespace(data: *const c_char, hasNewLines: *mut qboolean) -> *const c_char {
    let mut data_ptr = data;
    loop {
        let c = *data_ptr;
        if (c as u32) <= b' ' as u32 {
            if c == 0 {
                return core::ptr::null();
            }
            if c == b'\n' as c_char {
                parseData[parseDataCount as usize].com_lines += 1;
                *hasNewLines = qtrue;
            }
            data_ptr = data_ptr.add(1);
        } else {
            break;
        }
    }

    data_ptr
}

pub fn COM_ParseExt(data_p: *mut *const c_char, allowLineBreaks: qboolean) -> *mut c_char {
    let mut c: c_int = 0;
    let mut len: c_int;
    let mut hasNewLines: qboolean = qfalse;
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
            data = SkipWhitespace(data, core::ptr::addr_of_mut!(hasNewLines));
            if data.is_null() {
                *data_p = core::ptr::null();
                return com_token.as_mut_ptr();
            }
            if hasNewLines != 0 && allowLineBreaks == 0 {
                *data_p = data;
                return com_token.as_mut_ptr();
            }

            c = *data as c_int;

            // skip double slash comments
            if c == b'/' as c_int && *data.add(1) == b'/' as c_char {
                while *data != 0 && *data != b'\n' as c_char {
                    // Advance to the end of the line
                    data = data.add(1);
                }
            }
            // skip /* */ comments
            else if c == b'/' as c_int && *data.add(1) == b'*' as c_char {
                while *data != 0 && (*data != b'*' as c_char || *data.add(1) != b'/' as c_char) {
                    // Advance to the */ characters
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
                c = *data as c_int;
                data = data.add(1);
                if c == b'"' as c_int || c == 0 {
                    com_token[len as usize] = 0;
                    *data_p = data as *mut *const c_char;
                    return com_token.as_mut_ptr();
                }
                if len < MAX_TOKEN_CHARS as c_int {
                    com_token[len as usize] = c as c_char;
                    len += 1;
                }
            }
        }

        // parse a regular word
        loop {
            if len < MAX_TOKEN_CHARS as c_int {
                com_token[len as usize] = c as c_char;
                len += 1;
            }
            data = data.add(1);
            c = *data as c_int;
            if c == b'\n' as c_int {
                parseData[parseDataCount as usize].com_lines += 1;
            }
            if c <= 32 {
                break;
            }
        }

        if len == MAX_TOKEN_CHARS as c_int {
            Com_Printf(b"Token exceeded %i chars, discarded.\n\0".as_ptr() as *const c_char, MAX_TOKEN_CHARS as c_int);
            len = 0;
        }
        com_token[len as usize] = 0;

        *data_p = data as *mut *const c_char;
        com_token.as_mut_ptr()
    }
}

// ============================================================================
// COM_Compress
// remove blank space and comments from source
// ============================================================================

pub fn COM_Compress(data_p: *mut c_char) -> c_int {
    let mut in_ptr: *mut c_char;
    let mut out: *mut c_char;
    let mut c: c_int;
    let mut newline: qboolean = qfalse;
    let mut whitespace: qboolean = qfalse;

    unsafe {
        in_ptr = out = data_p;
        if !in_ptr.is_null() {
            while (*in_ptr) as c_int != 0 {
                c = *in_ptr as c_int;
                // skip double slash comments
                if c == b'/' as c_int && *in_ptr.add(1) == b'/' as c_char {
                    while *in_ptr != 0 && *in_ptr != b'\n' as c_char {
                        in_ptr = in_ptr.add(1);
                    }
                    // skip /* */ comments
                } else if c == b'/' as c_int && *in_ptr.add(1) == b'*' as c_char {
                    while *in_ptr != 0 && (*in_ptr != b'*' as c_char || *in_ptr.add(1) != b'/' as c_char) {
                        in_ptr = in_ptr.add(1);
                    }
                    if *in_ptr != 0 {
                        in_ptr = in_ptr.add(2);
                    }
                    // record when we hit a newline
                } else if c == b'\n' as c_int || c == b'\r' as c_int {
                    newline = qtrue;
                    in_ptr = in_ptr.add(1);
                    // record when we hit whitespace
                } else if c == b' ' as c_int || c == b'\t' as c_int {
                    whitespace = qtrue;
                    in_ptr = in_ptr.add(1);
                    // an actual token
                } else {
                    // if we have a pending newline, emit it (and it counts as whitespace)
                    if newline != 0 {
                        *out = b'\n' as c_char;
                        out = out.add(1);
                        newline = qfalse;
                        whitespace = qfalse;
                    }
                    if whitespace != 0 {
                        *out = b' ' as c_char;
                        out = out.add(1);
                        whitespace = qfalse;
                    }

                    // copy quoted strings unmolested
                    if c == b'"' as c_int {
                        *out = c as c_char;
                        out = out.add(1);
                        in_ptr = in_ptr.add(1);
                        loop {
                            c = *in_ptr as c_int;
                            if c != 0 && c != b'"' as c_int {
                                *out = c as c_char;
                                out = out.add(1);
                                in_ptr = in_ptr.add(1);
                            } else {
                                break;
                            }
                        }
                        if c == b'"' as c_int {
                            *out = c as c_char;
                            out = out.add(1);
                            in_ptr = in_ptr.add(1);
                        }
                    } else {
                        *out = c as c_char;
                        out = out.add(1);
                        in_ptr = in_ptr.add(1);
                    }
                }
            }
        }
        *out = 0;
        out as isize - data_p as isize
    }
}

// ============================================================================
// COM_MatchToken
// ============================================================================
pub fn COM_MatchToken(buf_p: *mut *const c_char, match_: *const c_char) {
    unsafe {
        let token = COM_Parse(buf_p);
        if strcmp(token, match_) != 0 {
            Com_Error(ERR_DROP, b"MatchToken: %s != %s\0".as_ptr() as *const c_char, token, match_);
        }
    }
}

// ============================================================================
// SkipBracedSection
//
// The next token should be an open brace.
// Skips until a matching close brace is found.
// Internal brace depths are properly skipped.
// ============================================================================
pub fn SkipBracedSection(program: *mut *const c_char) {
    let mut token: *mut c_char;
    let mut depth: c_int = 0;

    unsafe {
        if com_token[0] == b'{' as c_char {
            //for tr_shader which just ate the brace
            depth = 1;
        }
        loop {
            token = COM_ParseExt(program, qtrue);
            if *token.add(1) == 0 {
                if *token == b'{' as c_char {
                    depth += 1;

                } else if *token == b'}' as c_char {
                    depth -= 1;
                }
            }

            if depth == 0 || *(*program) == 0 {
                break;
            }
        }
    }
}

// ============================================================================
// SkipRestOfLine
// ============================================================================
pub fn SkipRestOfLine(data: *mut *const c_char) {
    unsafe {
        let mut p: *const c_char = *data;
        loop {
            let c = *p as c_int;
            p = p.add(1);
            if c == 0 {
                break;
            }
            if c == b'\n' as c_int {
                parseData[parseDataCount as usize].com_lines += 1;
                break;
            }
        }

        *data = p;
    }
}

pub fn Parse1DMatrix(buf_p: *mut *const c_char, x: c_int, m: *mut f32) {
    unsafe {
        let mut token: *mut c_char;
        let mut i: c_int;

        COM_MatchToken(buf_p, b"(\0".as_ptr() as *const c_char);

        for i in 0..x {
            token = COM_Parse(buf_p);
            *m.add(i as usize) = atof(token);
        }

        COM_MatchToken(buf_p, b")\0".as_ptr() as *const c_char);
    }
}

pub fn Parse2DMatrix(buf_p: *mut *const c_char, y: c_int, x: c_int, m: *mut f32) {
    unsafe {
        COM_MatchToken(buf_p, b"(\0".as_ptr() as *const c_char);

        for i in 0..y {
            Parse1DMatrix(buf_p, x, m.add((i * x) as usize));
        }

        COM_MatchToken(buf_p, b")\0".as_ptr() as *const c_char);
    }
}

pub fn Parse3DMatrix(buf_p: *mut *const c_char, z: c_int, y: c_int, x: c_int, m: *mut f32) {
    unsafe {
        COM_MatchToken(buf_p, b"(\0".as_ptr() as *const c_char);

        for i in 0..z {
            Parse2DMatrix(buf_p, y, x, m.add((i * x * y) as usize));
        }

        COM_MatchToken(buf_p, b")\0".as_ptr() as *const c_char);
    }
}

// ============================================================================
//
//                          LIBRARY REPLACEMENT FUNCTIONS
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

/*
char* Q_strrchr( const char* string, int c )
{
	char cc = c;
	char *s;
	char *sp=(char *)0;

	s = (char*)string;

	while (*s)
	{
		if (*s == cc)
			sp = s;
		s++;
	}
	if (cc == 0)
		sp = s;

	return sp;
}
*/
// ============================================================================
// Q_strncpyz
//
// Safe strncpy that ensures a trailing zero
// ============================================================================
pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int, bBarfIfTooLong: qboolean) {
    unsafe {
        if src.is_null() {
            Com_Error(ERR_FATAL, b"Q_strncpyz: NULL src\0".as_ptr() as *const c_char);
        }
        if destsize < 1 {
            Com_Error(ERR_FATAL, b"Q_strncpyz: destsize < 1\0".as_ptr() as *const c_char);
        }

        if bBarfIfTooLong != 0 {
            if strlen(src).wrapping_add(1) > destsize as usize {
                Com_Error(ERR_FATAL, b"String dest buffer too small to hold string \"%s\" %d > %d\n(source addr = %x, dest addr = %x\0".as_ptr() as *const c_char, src, strlen(src).wrapping_add(1), destsize, src, dest);
            }
        }
        strncpy(dest, src, (destsize - 1) as usize);
        *dest.add((destsize - 1) as usize) = 0;
    }
}
/*
int Q_stricmpn (const char *s1, const char *s2, int n) {
	int		c1, c2;

	do {
		c1 = *s1++;
		c2 = *s2++;

		if (!n--) {
			return 0;		// strings are equal until end point
		}

		if (c1 != c2) {
			if (c1 >= 'a' && c1 <= 'z') {
				c1 -= ('a' - 'A');
			}
			if (c2 >= 'a' && c2 <= 'z') {
				c2 -= ('a' - 'A');
			}
			if (c1 != c2) {
				return c1 < c2 ? -1 : 1;
			}
		}
	} while (c1);

	return 0;		// strings are equal
}

int Q_strncmp (const char *s1, const char *s2, int n) {
	int		c1, c2;

	do {
		c1 = *s1++;
		c2 = *s2++;

		if (!n--) {
			return 0;		// strings are equal until end point
		}

		if (c1 != c2) {
			return c1 < c2 ? -1 : 1;
		}
	} while (c1);

	return 0;		// strings are equal
}



char *Q_strlwr( char *s1 ) {
    char	*s;

    s = s1;
	while ( *s ) {
		*s = tolower(*s);
		s++;
	}
    return s1;
}

char *Q_strupr( char *s1 ) {
    char	*s;

    s = s1;
	while ( *s ) {
		*s = toupper(*s);
		s++;
	}
    return s1;
}
*/

// never goes past bounds or leaves without a terminating 0
pub fn Q_strcat(dest: *mut c_char, size: c_int, src: *const c_char) {
    unsafe {
        let l1 = strlen(dest);

        if l1 >= size as usize {
            Com_Error(ERR_FATAL, b"Q_strcat: already overflowed\0".as_ptr() as *const c_char);
        }
        if strlen(src).wrapping_add(1) > (size as usize).wrapping_sub(l1) {
            //do the error here instead of in Q_strncpyz to get a meaningful msg
            Com_Error(ERR_FATAL, b"Q_strcat: cannot append \"%s\" to \"%s\"\0".as_ptr() as *const c_char, src, dest);
        }
        Q_strncpyz(dest.add(l1), src, size - (l1 as c_int), qfalse);
    }
}

pub fn Q_PrintStrlen(string: *const c_char) -> c_int {
    unsafe {
        if string.is_null() {
            return 0;
        }

        let mut len = 0;
        let mut p = string;
        while *p != 0 {
            if Q_IsColorString(p as *const c_char) != 0 {
                p = p.add(2);
                continue;
            }
            p = p.add(1);
            len += 1;
        }

        len
    }
}

pub fn Q_CleanStr(string: *mut c_char) -> *mut c_char {
    unsafe {
        let mut d: *mut c_char = string;
        let mut s: *mut c_char = string;

        while (*s) as c_int != 0 {
            let c = *s as c_int;
            if Q_IsColorString(s as *const c_char) != 0 {
                s = s.add(1);
            } else if c >= 0x20 && c <= 0x7E {
                *d = c as c_char;
                d = d.add(1);
            }
            s = s.add(1);
        }
        *d = b'\0' as c_char;

        string
    }
}

pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, args: ...) {
    unsafe {
        let mut bigbuffer: [c_char; 1024] = [0; 1024];

        let mut ap = args;
        let len = vsprintf(bigbuffer.as_mut_ptr(), fmt, core::ptr::addr_of_mut!(ap) as *mut c_void);
        if len >= 1024 {
            Com_Error(ERR_FATAL, b"Com_sprintf: overflowed bigbuffer\0".as_ptr() as *const c_char);
        }
        if len >= size {
            Com_Printf(b"Com_sprintf: overflow of %i in %i\n\0".as_ptr() as *const c_char, len, size);
        }
        Q_strncpyz(dest, bigbuffer.as_ptr(), size, qfalse);
    }
}

// ============================================================================
// va
//
// does a varargs printf into a temp buffer, so I don't need to have
// varargs versions of all text functions.
// FIXME: make this buffer size safe someday
// ============================================================================
pub fn va(format: *const c_char, args: ...) -> *mut c_char {
    unsafe {
        static mut buffers: [[c_char; 1024]; 4] = [[0; 1024]; 4];
        static mut index: c_int = 0;

        let buf = buffers[(index as usize) % 4].as_mut_ptr();
        index += 1;

        let mut ap = args;
        let len = vsprintf(buf, format, core::ptr::addr_of_mut!(ap) as *mut c_void);

        debug_assert!(len < 1024 as c_int);

        buf
    }
}

// ============================================================================
//
//   INFO STRINGS
//
// ============================================================================

// ============================================================================
// Info_ValueForKey
//
// Searches the string for the given
// key and returns the associated value, or an empty string.
// FIXME: overflow check?
// ============================================================================
pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char {
    unsafe {
        let mut pkey: [c_char; MAX_INFO_KEY] = [0; MAX_INFO_KEY];
        static mut value: [[c_char; MAX_INFO_VALUE]; 2] = [[0; MAX_INFO_VALUE]; 2];
        // use two buffers so compares
        // work without stomping on each other
        static mut valueindex: c_int = 0;
        let mut o: *mut c_char;

        if s.is_null() || key.is_null() {
            return b"\0".as_ptr() as *mut c_char;
        }

        if strlen(s) >= MAX_INFO_STRING {
            Com_Error(ERR_DROP, b"Info_ValueForKey: oversize infostring\0".as_ptr() as *const c_char);
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

// ============================================================================
// Info_NextPair
//
// Used to itterate through all the key/value pairs in an info string
// ============================================================================
pub fn Info_NextPair(head: *mut *const c_char, key: *mut c_char, value: *mut c_char) {
    unsafe {
        let mut o: *mut c_char;
        let mut s: *const c_char;

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

// ============================================================================
// Info_RemoveKey
// ============================================================================
pub fn Info_RemoveKey(s: *mut c_char, key: *const c_char) {
    unsafe {
        let mut start: *mut c_char;
        let mut pkey: [c_char; MAX_INFO_KEY] = [0; MAX_INFO_KEY];
        let mut value: [c_char; MAX_INFO_VALUE] = [0; MAX_INFO_VALUE];
        let mut o: *mut c_char;

        if strlen(s) >= MAX_INFO_STRING {
            Com_Error(ERR_DROP, b"Info_RemoveKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        if !strchr(key, b'\\' as c_int).is_null() {
            return;
        }

        let mut s_ptr = s;
        loop {
            start = s_ptr;
            if *s_ptr == b'\\' as c_char {
                s_ptr = s_ptr.add(1);
            }
            o = pkey.as_mut_ptr();
            while *s_ptr != b'\\' as c_char {
                if *s_ptr == 0 {
                    return;
                }
                *o = *s_ptr;
                o = o.add(1);
                s_ptr = s_ptr.add(1);
            }
            *o = 0;
            s_ptr = s_ptr.add(1);

            o = value.as_mut_ptr();
            while *s_ptr != b'\\' as c_char && *s_ptr != 0 {
                if *s_ptr == 0 {
                    return;
                }
                *o = *s_ptr;
                o = o.add(1);
                s_ptr = s_ptr.add(1);
            }
            *o = 0;

            if strcmp(key, pkey.as_ptr()) == 0 {
                strcpy(start, s_ptr);	// remove this part
                return;
            }

            if *s_ptr == 0 {
                return;
            }
            s_ptr = s_ptr.add(1);
        }
    }
}

// ============================================================================
// Info_Validate
//
// Some characters are illegal in info strings because they
// can mess up the server's parsing
// ============================================================================
pub fn Info_Validate(s: *const c_char) -> qboolean {
    unsafe {
        if !strchr(s, b'"' as c_int).is_null() {
            return qfalse;
        }
        if !strchr(s, b';' as c_int).is_null() {
            return qfalse;
        }
        qtrue
    }
}

// ============================================================================
// Info_SetValueForKey
//
// Changes or adds a key/value pair
// ============================================================================
pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char) {
    unsafe {
        let mut newi: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];

        if strlen(s as *const c_char) >= MAX_INFO_STRING {
            Com_Error(ERR_DROP, b"Info_SetValueForKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        if !strchr(key, b'\\' as c_int).is_null() || !strchr(value, b'\\' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a \\(%s, %s)\n\0".as_ptr() as *const c_char, key, value);
            return;
        }

        if !strchr(key, b';' as c_int).is_null() || !strchr(value, b';' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a semicolon(%s, %s)\n\0".as_ptr() as *const c_char, key, value);
            return;
        }

        if !strchr(key, b'"' as c_int).is_null() || !strchr(value, b'"' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a \"(%s, %s)\n\0".as_ptr() as *const c_char, key, value);
            return;
        }

        Info_RemoveKey(s, key);
        if value.is_null() || strlen(value) == 0 {
            return;
        }

        Com_sprintf(newi.as_mut_ptr(), newi.len() as c_int, b"\\%s\\%s\0".as_ptr() as *const c_char, key, value);

        if strlen(newi.as_ptr()).wrapping_add(strlen(s as *const c_char)) > MAX_INFO_STRING {
            Com_Printf(b"Info string length exceeded\n\0".as_ptr() as *const c_char);
            return;
        }

        strcat(s, newi.as_ptr());
    }
}

// ============================================================================
//
// String ID Tables
//
// ============================================================================

// ============================================================================
// GetIDForString
// ============================================================================

#[allow(non_upper_case_globals)]
const VALIDSTRING: unsafe fn(*const c_char) -> bool = |a: *const c_char| {
    !a.is_null() && !(*a as u8 == 0)
};

pub fn GetIDForString(table: *const stringID_table_t, string: *const c_char) -> c_int {
    unsafe {
        let mut index: c_int = 0;

        while !(*table.add(index as usize)).name.is_null() && !(*(*table.add(index as usize)).name as u8 == 0) {
            if Q_stricmp((*table.add(index as usize)).name, string) == 0 {
                return (*table.add(index as usize)).id;
            }

            index += 1;
        }

        -1
    }
}

// ============================================================================
// GetStringForID
// ============================================================================

pub fn GetStringForID(table: *const stringID_table_t, id: c_int) -> *const c_char {
    unsafe {
        let mut index: c_int = 0;

        while !(*table.add(index as usize)).name.is_null() && !(*(*table.add(index as usize)).name as u8 == 0) {
            if (*table.add(index as usize)).id == id {
                return (*table.add(index as usize)).name;
            }

            index += 1;
        }

        core::ptr::null()
    }
}

// ============================================================================
// COM_ParseString
// ============================================================================
pub fn COM_ParseString(data: *mut *const c_char, s: *mut *const c_char) -> qboolean {
    unsafe {
        //	*s = COM_ParseExt( data, qtrue );
        *s = COM_ParseExt(data, qfalse);
        if *(*s) == 0 {
            Com_Printf(b"unexpected EOF in COM_ParseString\n\0".as_ptr() as *const c_char);
            return qtrue;
        }
        qfalse
    }
}

// ============================================================================
// COM_ParseInt
// ============================================================================
pub fn COM_ParseInt(data: *mut *const c_char, i: *mut c_int) -> qboolean {
    unsafe {
        let token: *const c_char;

        token = COM_ParseExt(data, qfalse);
        if *token == 0 {
            Com_Printf(b"unexpected EOF in COM_ParseInt\n\0".as_ptr() as *const c_char);
            return qtrue;
        }

        *i = atoi(token);
        qfalse
    }
}

// ============================================================================
// COM_ParseFloat
// ============================================================================
pub fn COM_ParseFloat(data: *mut *const c_char, f: *mut f32) -> qboolean {
    unsafe {
        let token: *const c_char;

        token = COM_ParseExt(data, qfalse);
        if *token == 0 {
            Com_Printf(b"unexpected EOF in COM_ParseFloat\n\0".as_ptr() as *const c_char);
            return qtrue;
        }

        *f = atof(token);
        qfalse
    }
}

// ============================================================================
// COM_ParseVec4
// ============================================================================
pub fn COM_ParseVec4(buffer: *mut *const c_char, c: *mut vec4_t) -> qboolean {
    unsafe {
        let mut f: f32;

        for i in 0..4 {
            if COM_ParseFloat(buffer, core::ptr::addr_of_mut!(f)) != 0 {
                return qtrue;
            }
            (*c)[i] = f;
        }
        qfalse
    }
}

// end
