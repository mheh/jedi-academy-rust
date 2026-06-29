// q_shared.c -- stateless support routines that are included in each code dll

use core::ffi::{c_char, c_int, c_void, c_uint, CStr};
use core::ptr;

// Note: These types and functions are expected to be imported from q_shared_h module
// Local stub declarations for types that cross this module boundary
#[repr(C)]
pub struct growList_t {
    pub maxElements: c_int,
    pub currentElements: c_int,
    pub elements: *mut *mut c_void,
}

// These constants are assumed to be defined elsewhere in q_shared.h
// Importing conceptually but providing local stubs for compilation
pub const MAX_TOKEN_CHARS: usize = 1024;
pub const MAX_INFO_STRING: usize = 1024;
pub const MAX_INFO_KEY: usize = 1024;
pub const MAX_INFO_VALUE: usize = 1024;
pub const FILE_HASH_SIZE: usize = 1024;
pub const MAX_QPATH: usize = 256;
pub const ERR_FATAL: c_int = 3;
pub const ERR_DROP: c_int = 2;
pub type qboolean = c_int;
pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

// External functions
extern "C" {
    pub fn Com_Allocate(bytes: c_int) -> *mut c_void;
    pub fn Com_Dealloc(ptr: *mut c_void);
    pub fn Com_Error(error_level: c_int, fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Parse(data: *mut *const c_char) -> *const c_char;
    pub fn Com_ParseOnLine(data: *mut *const c_char) -> *const c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn tolower(c: c_int) -> c_int;
    pub fn toupper(c: c_int) -> c_int;
    pub fn vsprintf(str: *mut c_char, format: *const c_char, ap: core::ffi::VaList) -> c_int;
}

/*
============================================================================

GROWLISTS

============================================================================
*/

// malloc / free all in one place for debugging

pub fn Com_InitGrowList(list: *mut growList_t, maxElements: c_int) {
    unsafe {
        (*list).maxElements = maxElements;
        (*list).currentElements = 0;
        (*list).elements = Com_Allocate(maxElements * (core::mem::size_of::<*mut c_void>() as c_int)) as *mut *mut c_void;
    }
}

pub fn Com_AddToGrowList(list: *mut growList_t, data: *mut c_void) -> c_int {
    unsafe {
        let mut old: *mut *mut c_void;

        if (*list).currentElements != (*list).maxElements {
            *(*list).elements.add((*list).currentElements as usize) = data;
            let result = (*list).currentElements;
            (*list).currentElements += 1;
            return result;
        }

        // grow, reallocate and move
        old = (*list).elements;

        if (*list).maxElements < 0 {
            Com_Error(ERR_FATAL, b"Com_AddToGrowList: maxElements = %i\0".as_ptr() as *const c_char, (*list).maxElements);
        }

        if (*list).maxElements == 0 {
            // initialize the list to hold 100 elements
            Com_InitGrowList(list, 100);
            return Com_AddToGrowList(list, data);
        }

        (*list).maxElements *= 2;

        Com_DPrintf(b"Resizing growlist to %i maxElements\n\0".as_ptr() as *const c_char, (*list).maxElements);

        (*list).elements = Com_Allocate((*list).maxElements * (core::mem::size_of::<*mut c_void>() as c_int)) as *mut *mut c_void;

        if (*list).elements.is_null() {
            Com_Error(ERR_DROP, b"Growlist alloc failed\0".as_ptr() as *const c_char);
        }

        memcpy(
            (*list).elements as *mut c_void,
            old as *const c_void,
            ((*list).currentElements as usize) * core::mem::size_of::<*mut c_void>(),
        );

        Com_Dealloc(old as *mut c_void);

        return Com_AddToGrowList(list, data);
    }
}

pub fn Com_GrowListElement(list: *const growList_t, index: c_int) -> *mut c_void {
    unsafe {
        if index < 0 || index >= (*list).currentElements {
            Com_Error(ERR_DROP, b"Com_GrowListElement: %i out of range of %i\0".as_ptr() as *const c_char,
                index, (*list).currentElements);
        }
        return *(*list).elements.add(index as usize);
    }
}

pub fn Com_IndexForGrowListElement(list: *const growList_t, element: *const c_void) -> c_int {
    unsafe {
        let mut i: c_int;

        i = 0;
        while i < (*list).currentElements {
            if *(*list).elements.add(i as usize) == element as *mut c_void {
                return i;
            }
            i += 1;
        }
        return -1;
    }
}

//============================================================================


pub fn Com_Clamp(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    return value;
}

/*
============
Com_StringContains
============
*/
pub fn Com_StringContains(str1: *const c_char, str2: *const c_char, casesensitive: c_int) -> *const c_char {
    unsafe {
        let mut len: c_int;
        let mut i: c_int;
        let mut j: c_int;
        let mut str1_ptr: *const c_char = str1;

        len = (strlen(str1) as c_int) - (strlen(str2) as c_int);
        i = 0;
        while i <= len {
            j = 0;
            while *str2.add(j as usize) != 0 {
                if casesensitive != 0 {
                    if *str1_ptr.add(j as usize) != *str2.add(j as usize) {
                        break;
                    }
                } else {
                    if toupper(*str1_ptr.add(j as usize) as c_int) != toupper(*str2.add(j as usize) as c_int) {
                        break;
                    }
                }
                j += 1;
            }
            if *str2.add(j as usize) == 0 {
                return str1_ptr;
            }
            i += 1;
            str1_ptr = str1_ptr.add(1);
        }
        return ptr::null();
    }
}

/*
============
Com_Filter
============
*/
pub fn Com_Filter(filter: *const c_char, name: *const c_char, casesensitive: c_int) -> c_int {
    unsafe {
        let mut buf: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
        let mut ptr: *const c_char;
        let mut i: c_int;
        let mut found: c_int;
        let mut filter_ptr: *const c_char = filter;
        let mut name_ptr: *const c_char = name;

        while *filter_ptr != 0 {
            if *filter_ptr as u8 == b'*' {
                filter_ptr = filter_ptr.add(1);
                i = 0;
                while *filter_ptr != 0 {
                    if *filter_ptr as u8 == b'*' || *filter_ptr as u8 == b'?' {
                        break;
                    }
                    buf[i as usize] = *filter_ptr;
                    filter_ptr = filter_ptr.add(1);
                    i += 1;
                }
                buf[i as usize] = 0;
                if strlen(buf.as_ptr()) > 0 {
                    ptr = Com_StringContains(name_ptr, buf.as_ptr(), casesensitive);
                    if ptr.is_null() {
                        return qfalse;
                    }
                    name_ptr = ptr.add(strlen(buf.as_ptr()));
                }
            } else if *filter_ptr as u8 == b'?' {
                filter_ptr = filter_ptr.add(1);
                name_ptr = name_ptr.add(1);
            } else if *filter_ptr as u8 == b'[' && *filter_ptr.add(1) as u8 == b'[' {
                filter_ptr = filter_ptr.add(1);
            } else if *filter_ptr as u8 == b'[' {
                filter_ptr = filter_ptr.add(1);
                found = qfalse;
                while *filter_ptr != 0 && found == 0 {
                    if *filter_ptr as u8 == b']' && *filter_ptr.add(1) as u8 != b']' {
                        break;
                    }
                    if *filter_ptr.add(1) as u8 == b'-' && *filter_ptr.add(2) != 0 && (*filter_ptr.add(2) as u8 != b']' || *filter_ptr.add(3) as u8 == b']') {
                        if casesensitive != 0 {
                            if *name_ptr as u8 >= *filter_ptr as u8 && *name_ptr as u8 <= *filter_ptr.add(2) as u8 {
                                found = qtrue;
                            }
                        } else {
                            if toupper(*name_ptr as c_int) as u8 >= toupper(*filter_ptr as c_int) as u8 &&
                                toupper(*name_ptr as c_int) as u8 <= toupper(*filter_ptr.add(2) as c_int) as u8 {
                                found = qtrue;
                            }
                        }
                        filter_ptr = filter_ptr.add(3);
                    } else {
                        if casesensitive != 0 {
                            if *filter_ptr == *name_ptr {
                                found = qtrue;
                            }
                        } else {
                            if toupper(*filter_ptr as c_int) == toupper(*name_ptr as c_int) {
                                found = qtrue;
                            }
                        }
                        filter_ptr = filter_ptr.add(1);
                    }
                }
                if found == 0 {
                    return qfalse;
                }
                while *filter_ptr != 0 {
                    if *filter_ptr as u8 == b']' && *filter_ptr.add(1) as u8 != b']' {
                        break;
                    }
                    filter_ptr = filter_ptr.add(1);
                }
                filter_ptr = filter_ptr.add(1);
                name_ptr = name_ptr.add(1);
            } else {
                if casesensitive != 0 {
                    if *filter_ptr != *name_ptr {
                        return qfalse;
                    }
                } else {
                    if toupper(*filter_ptr as c_int) != toupper(*name_ptr as c_int) {
                        return qfalse;
                    }
                }
                filter_ptr = filter_ptr.add(1);
                name_ptr = name_ptr.add(1);
            }
        }
        return qtrue;
    }
}


/*
================
Com_HashString

================
*/
pub fn Com_HashString(fname: *const c_char) -> c_int {
    unsafe {
        let mut i: c_int = 0;
        let mut hash: i64 = 0;
        let mut letter: c_char;

        while *fname.add(i as usize) != 0 {
            letter = tolower(*fname.add(i as usize) as c_int) as c_char;
            if letter as u8 == b'.' {
                break;  // don't include extension
            }
            if letter as u8 == b'\\' {
                letter = '/' as c_char;  // damn path names
            }
            hash += (letter as i64) * ((i + 119) as i64);
            i += 1;
        }
        hash &= (FILE_HASH_SIZE as i64 - 1);
        return hash as c_int;
    }
}


/*
============
Com_SkipPath
============
*/
pub fn Com_SkipPath(pathname: *mut c_char) -> *mut c_char {
    unsafe {
        let mut last: *mut c_char;

        last = pathname;
        let mut pathname_ptr = pathname;
        while *pathname_ptr != 0 {
            if *pathname_ptr as u8 == b'/' {
                last = pathname_ptr.add(1);
            }
            pathname_ptr = pathname_ptr.add(1);
        }
        return last;
    }
}

/*
============
Com_StripExtension
============
*/
pub fn Com_StripExtension(r#in: *const c_char, out: *mut c_char) {
    unsafe {
        let mut in_ptr = r#in;
        let mut out_ptr = out;
        while *in_ptr != 0 && *in_ptr as u8 != b'.' {
            *out_ptr = *in_ptr;
            out_ptr = out_ptr.add(1);
            in_ptr = in_ptr.add(1);
        }
        *out_ptr = 0;
    }
}


/*
==================
Com_DefaultExtension
==================
*/
pub fn Com_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char) {
    unsafe {
        let mut oldPath: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut src: *mut c_char;

        //
        // if path doesn't have a .EXT, append extension
        // (extension should include the .)
        //
        src = path.add(strlen(path) - 1);

        while *src as u8 != b'/' && src != path {
            if *src as u8 == b'.' {
                return;  // it has an extension
            }
            src = src.add(-1);
        }

        Q_strncpyz(oldPath.as_mut_ptr(), path, core::mem::size_of_val(&oldPath) as c_int);
        Com_sprintf(path, maxSize, b"%s%s\0".as_ptr() as *const c_char, oldPath.as_ptr(), extension);
    }
}

/*
============================================================================

					BYTE ORDER FUNCTIONS

============================================================================
*/

// can't just use function pointers, or dll linkage can
// mess up when qcommon is included in multiple places
static mut _BigShort: Option<fn(c_short) -> c_short> = None;
static mut _LittleShort: Option<fn(c_short) -> c_short> = None;
static mut _BigLong: Option<fn(c_int) -> c_int> = None;
static mut _LittleLong: Option<fn(c_int) -> c_int> = None;
static mut _BigFloat: Option<fn(f32) -> f32> = None;
static mut _LittleFloat: Option<fn(f32) -> f32> = None;

pub fn BigShort(l: c_short) -> c_short {
    unsafe {
        match _BigShort {
            Some(f) => f(l),
            None => l,
        }
    }
}

pub fn LittleShort(l: c_short) -> c_short {
    unsafe {
        match _LittleShort {
            Some(f) => f(l),
            None => l,
        }
    }
}

pub fn BigLong(l: c_int) -> c_int {
    unsafe {
        match _BigLong {
            Some(f) => f(l),
            None => l,
        }
    }
}

pub fn LittleLong(l: c_int) -> c_int {
    unsafe {
        match _LittleLong {
            Some(f) => f(l),
            None => l,
        }
    }
}

pub fn BigFloat(l: f32) -> f32 {
    unsafe {
        match _BigFloat {
            Some(f) => f(l),
            None => l,
        }
    }
}

pub fn LittleFloat(l: f32) -> f32 {
    unsafe {
        match _LittleFloat {
            Some(f) => f(l),
            None => l,
        }
    }
}

pub fn ShortSwap(l: c_short) -> c_short {
    let b1: u8;
    let b2: u8;

    b1 = (l as u16 & 255) as u8;
    b2 = ((l as u16 >> 8) & 255) as u8;

    return (((b1 as c_short) << 8) + (b2 as c_short)) as c_short;
}

pub fn ShortNoSwap(l: c_short) -> c_short {
    return l;
}

pub fn LongSwap(l: c_int) -> c_int {
    let b1: u8;
    let b2: u8;
    let b3: u8;
    let b4: u8;

    b1 = (l as u32 & 255) as u8;
    b2 = ((l as u32 >> 8) & 255) as u8;
    b3 = ((l as u32 >> 16) & 255) as u8;
    b4 = ((l as u32 >> 24) & 255) as u8;

    return (((b1 as c_int) << 24) + ((b2 as c_int) << 16) + ((b3 as c_int) << 8) + (b4 as c_int)) as c_int;
}

pub fn LongNoSwap(l: c_int) -> c_int {
    return l;
}

pub fn FloatSwap(f: f32) -> f32 {
    #[repr(C)]
    union FloatBytes {
        f: f32,
        b: [u8; 4],
    }

    unsafe {
        let mut dat1: FloatBytes = FloatBytes { f: f };
        let mut dat2: FloatBytes = FloatBytes { b: [0; 4] };

        dat2.b[0] = dat1.b[3];
        dat2.b[1] = dat1.b[2];
        dat2.b[2] = dat1.b[1];
        dat2.b[3] = dat1.b[0];
        return dat2.f;
    }
}

pub fn FloatNoSwap(f: f32) -> f32 {
    return f;
}

/*
================
Swap_Init
================
*/
pub fn Swap_Init() {
    unsafe {
        let swaptest: [u8; 2] = [1, 0];

        // set the byte swapping variables in a portable manner
        if *(swaptest.as_ptr() as *const c_short) == 1 {
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

/*
===============
Com_ParseInfos
===============
*/
pub fn Com_ParseInfos(buf: *const c_char, max: c_int, infos: *mut [c_char; MAX_INFO_STRING]) -> c_int {
    unsafe {
        let mut token: *const c_char;
        let mut count: c_int = 0;
        let mut key: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
        let mut buf_ptr: *const c_char = buf;

        loop {
            token = Com_Parse(&mut buf_ptr);
            if *token == 0 {
                break;
            }
            if strcmp(token, b"{\0".as_ptr() as *const c_char) != 0 {
                Com_Printf(b"Missing { in info file\n\0".as_ptr() as *const c_char);
                break;
            }

            if count == max {
                Com_Printf(b"Max infos exceeded\n\0".as_ptr() as *const c_char);
                break;
            }

            *(*infos.add(count as usize)).as_mut_ptr() = 0;
            loop {
                token = Com_Parse(&mut buf_ptr);
                if *token == 0 {
                    Com_Printf(b"Unexpected end of info file\n\0".as_ptr() as *const c_char);
                    break;
                }
                if strcmp(token, b"}\0".as_ptr() as *const c_char) == 0 {
                    break;
                }
                Q_strncpyz(key.as_mut_ptr(), token, core::mem::size_of_val(&key) as c_int);

                token = Com_ParseOnLine(&mut buf_ptr);
                if *token == 0 {
                    token = b"<NULL>\0".as_ptr() as *const c_char;
                }
                Info_SetValueForKey((*infos.add(count as usize)).as_mut_ptr(), key.as_ptr(), token);
            }
            count += 1;
        }

        return count;
    }
}



/*
============================================================================

					LIBRARY REPLACEMENT FUNCTIONS

============================================================================
*/

pub fn Q_isprint(c: c_int) -> c_int {
    if c >= 0x20 && c <= 0x7E {
        return 1;
    }
    return 0;
}

pub fn Q_islower(c: c_int) -> c_int {
    if c >= ('a' as c_int) && c <= ('z' as c_int) {
        return 1;
    }
    return 0;
}

pub fn Q_isupper(c: c_int) -> c_int {
    if c >= ('A' as c_int) && c <= ('Z' as c_int) {
        return 1;
    }
    return 0;
}

pub fn Q_isalpha(c: c_int) -> c_int {
    if (c >= ('a' as c_int) && c <= ('z' as c_int)) || (c >= ('A' as c_int) && c <= ('Z' as c_int)) {
        return 1;
    }
    return 0;
}

pub fn Q_strrchr(string: *const c_char, c: c_int) -> *mut c_char {
    unsafe {
        let cc: c_char = c as c_char;
        let mut s: *mut c_char;
        let mut sp: *mut c_char = ptr::null_mut();

        s = string as *mut c_char;

        while *s != 0 {
            if *s == cc {
                sp = s;
            }
            s = s.add(1);
        }
        if cc as c_int == 0 {
            sp = s;
        }

        return sp;
    }
}

/*
=============
Q_strncpyz

Safe strncpy that ensures a trailing zero
=============
*/
pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int) {
    unsafe {
        if src.is_null() {
            Com_Error(ERR_FATAL, b"Q_strncpyz: NULL src\0".as_ptr() as *const c_char);
        }
        if destsize < 1 {
            Com_Error(ERR_FATAL, b"Q_strncpyz: destsize < 1\0".as_ptr() as *const c_char);
        }

        strncpy(dest, src, (destsize - 1) as usize);
        *dest.add((destsize - 1) as usize) = 0;
    }
}

pub fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int {
    unsafe {
        let mut c1: c_int;
        let mut c2: c_int;
        let mut s1_ptr = s1;
        let mut s2_ptr = s2;
        let mut n_mut = n;

        loop {
            c1 = *s1_ptr as c_int;
            s1_ptr = s1_ptr.add(1);
            c2 = *s2_ptr as c_int;
            s2_ptr = s2_ptr.add(1);

            if n_mut == 0 {
                return 0;  // strings are equal until end point
            }
            n_mut -= 1;

            if c1 != c2 {
                if c1 >= ('a' as c_int) && c1 <= ('z' as c_int) {
                    c1 -= ('a' as c_int) - ('A' as c_int);
                }
                if c2 >= ('a' as c_int) && c2 <= ('z' as c_int) {
                    c2 -= ('a' as c_int) - ('A' as c_int);
                }
                if c1 != c2 {
                    return if c1 < c2 { -1 } else { 1 };
                }
            }
            if c1 == 0 {
                break;
            }
        }

        return 0;  // strings are equal
    }
}

pub fn Q_strncmp(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int {
    unsafe {
        let mut c1: c_int;
        let mut c2: c_int;
        let mut s1_ptr = s1;
        let mut s2_ptr = s2;
        let mut n_mut = n;

        loop {
            c1 = *s1_ptr as c_int;
            s1_ptr = s1_ptr.add(1);
            c2 = *s2_ptr as c_int;
            s2_ptr = s2_ptr.add(1);

            if n_mut == 0 {
                return 0;  // strings are equal until end point
            }
            n_mut -= 1;

            if c1 != c2 {
                return if c1 < c2 { -1 } else { 1 };
            }
            if c1 == 0 {
                break;
            }
        }

        return 0;  // strings are equal
    }
}

pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    return Q_stricmpn(s1, s2, 99999);
}


pub fn Q_strlwr(s1: *mut c_char) -> *mut c_char {
    unsafe {
        let mut s: *mut c_char;

        s = s1;
        while *s != 0 {
            *s = tolower(*s as c_int) as c_char;
            s = s.add(1);
        }
        return s1;
    }
}

pub fn Q_strupr(s1: *mut c_char) -> *mut c_char {
    unsafe {
        let mut s: *mut c_char;

        s = s1;
        while *s != 0 {
            *s = toupper(*s as c_int) as c_char;
            s = s.add(1);
        }
        return s1;
    }
}


// never goes past bounds or leaves without a terminating 0
pub fn Q_strcat(dest: *mut c_char, size: c_int, src: *const c_char) {
    unsafe {
        let mut l1: c_int;

        l1 = strlen(dest) as c_int;
        if l1 >= size {
            Com_Error(ERR_FATAL, b"Q_strcat: already overflowed\0".as_ptr() as *const c_char);
        }
        Q_strncpyz(dest.add(l1 as usize), src, size - l1);
    }
}


pub fn Q_PrintStrlen(string: *const c_char) -> c_int {
    unsafe {
        let mut len: c_int;
        let mut p: *const c_char;

        if string.is_null() {
            return 0;
        }

        len = 0;
        p = string;
        while *p != 0 {
            if Q_IsColorString(p) != 0 {
                p = p.add(2);
                continue;
            }
            p = p.add(1);
            len += 1;
        }

        return len;
    }
}

// Note: Q_IsColorString is a macro function expected to be defined elsewhere
pub fn Q_IsColorString(p: *const c_char) -> c_int {
    // Placeholder - this should be imported or defined from q_shared.h
    0
}

pub fn Q_CleanStr(string: *mut c_char) -> *mut c_char {
    unsafe {
        let mut d: *mut c_char;
        let mut s: *mut c_char;
        let mut c: c_int;

        s = string;
        d = string;
        while {
            c = *s as c_int;
            c != 0
        } {
            if Q_IsColorString(s) != 0 {
                s = s.add(1);
            }
            else if c >= 0x20 && c <= 0x7E {
                *d = c as c_char;
                d = d.add(1);
            }
            s = s.add(1);
        }
        *d = '\0' as c_char;

        return string;
    }
}


// Varargs functions must be provided via C FFI: Rust cannot implement varargs functions
// that use va_list in a portable way. These are declared as provided by a C module.
// Structural translation preserved for reference:
//
// Original C:
//   void Com_sprintf( char *dest, int size, const char *fmt, ...) {
//       va_list argptr;
//       char bigbuffer[32000];
//       va_start(argptr, fmt);
//       len = vsprintf(bigbuffer, fmt, argptr);
//       va_end(argptr);
//       ... error checking ...
//       Q_strncpyz(dest, bigbuffer, size);
//   }

extern "C" {
    pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);

    /*
    ============
    va

    does a varargs printf into a temp buffer, so I don't need to have
    varargs versions of all text functions.
    FIXME: make this buffer size safe someday
    ============
    */
    pub fn va(format: *const c_char, ...) -> *mut c_char;
}


/*
=====================================================================

  INFO STRINGS

=====================================================================
*/

/*
===============
Info_ValueForKey

Searches the string for the given
key and returns the associated value, or an empty string.
FIXME: overflow check?
===============
*/
pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char {
    unsafe {
        let mut pkey: [c_char; MAX_INFO_KEY] = [0; MAX_INFO_KEY];
        static mut value: [[c_char; MAX_INFO_VALUE]; 2] = [[0; MAX_INFO_VALUE]; 2];  // use two buffers so compares
                                                    // work without stomping on each other
        static mut valueindex: c_int = 0;
        let mut o: *mut c_char;
        let mut s_ptr: *const c_char;

        if s.is_null() || key.is_null() {
            return b"\0".as_ptr() as *mut c_char;
        }

        if strlen(s) >= MAX_INFO_STRING {
            Com_Error(ERR_DROP, b"Info_ValueForKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        valueindex ^= 1;
        s_ptr = s;
        if *s_ptr as u8 == b'\\' {
            s_ptr = s_ptr.add(1);
        }
        loop {
            o = pkey.as_mut_ptr();
            while *s_ptr as u8 != b'\\' {
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

            while *s_ptr as u8 != b'\\' && *s_ptr != 0 {
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

        return b"\0".as_ptr() as *mut c_char;
    }
}


/*
===================
Info_NextPair

Used to itterate through all the key/value pairs in an info string
===================
*/
pub fn Info_NextPair(head: *mut *const c_char, key: *mut c_char, value: *mut c_char) {
    unsafe {
        let mut o: *mut c_char;
        let mut s: *const c_char;

        s = *head;

        if *s as u8 == b'\\' {
            s = s.add(1);
        }
        *key = 0;
        *value = 0;

        o = key;
        while *s as u8 != b'\\' {
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
        while *s as u8 != b'\\' && *s != 0 {
            *o = *s;
            o = o.add(1);
            s = s.add(1);
        }
        *o = 0;

        *head = s;
    }
}


/*
===================
Info_RemoveKey
===================
*/
pub fn Info_RemoveKey(s: *mut c_char, key: *const c_char) {
    unsafe {
        let mut start: *mut c_char;
        let mut pkey: [c_char; MAX_INFO_KEY] = [0; MAX_INFO_KEY];
        let mut value: [c_char; MAX_INFO_VALUE] = [0; MAX_INFO_VALUE];
        let mut o: *mut c_char;
        let mut s_ptr: *mut c_char = s;

        if strlen(s) >= MAX_INFO_STRING {
            Com_Error(ERR_DROP, b"Info_RemoveKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        if !strchr(key, '\\' as c_int).is_null() {
            return;
        }

        loop {
            start = s_ptr;
            if *s_ptr as u8 == b'\\' {
                s_ptr = s_ptr.add(1);
            }
            o = pkey.as_mut_ptr();
            while *s_ptr as u8 != b'\\' {
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
            while *s_ptr as u8 != b'\\' && *s_ptr != 0 {
                if *s_ptr == 0 {
                    return;
                }
                *o = *s_ptr;
                o = o.add(1);
                s_ptr = s_ptr.add(1);
            }
            *o = 0;

            if strcmp(key, pkey.as_ptr()) == 0 {
                strcpy(start, s_ptr);  // remove this part
                return;
            }

            if *s_ptr == 0 {
                return;
            }
        }
    }
}


/*
==================
Info_Validate

Some characters are illegal in info strings because they
can mess up the server's parsing
==================
*/
pub fn Info_Validate(s: *const c_char) -> qboolean {
    unsafe {
        if !strchr(s, '\"' as c_int).is_null() {
            return qfalse;
        }
        if !strchr(s, ';' as c_int).is_null() {
            return qfalse;
        }
        return qtrue;
    }
}

/*
==================
Info_SetValueForKey

Changes or adds a key/value pair
==================
*/
pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char) {
    unsafe {
        let mut newi: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];

        if strlen(s) >= MAX_INFO_STRING {
            Com_Error(ERR_DROP, b"Info_SetValueForKey: oversize infostring\0".as_ptr() as *const c_char);
        }

        if !strchr(key, '\\' as c_int).is_null() || !strchr(value, '\\' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a \\\n\0".as_ptr() as *const c_char);
            return;
        }

        if !strchr(key, ';' as c_int).is_null() || !strchr(value, ';' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a semicolon\n\0".as_ptr() as *const c_char);
            return;
        }

        if !strchr(key, '\"' as c_int).is_null() || !strchr(value, '\"' as c_int).is_null() {
            Com_Printf(b"Can't use keys or values with a \"\n\0".as_ptr() as *const c_char);
            return;
        }

        Info_RemoveKey(s, key);
        if value.is_null() || strlen(value) == 0 {
            return;
        }

        Com_sprintf(newi.as_mut_ptr(), core::mem::size_of_val(&newi) as c_int, b"\\%s\\%s\0".as_ptr() as *const c_char, key, value);

        if strlen(newi.as_ptr()) + strlen(s) > MAX_INFO_STRING {
            Com_Printf(b"Info string length exceeded\n\0".as_ptr() as *const c_char);
            return;
        }

        strcat(s, newi.as_ptr());
    }
}

//====================================================================


/*
===============
ParseHex
===============
*/
pub fn ParseHex(text: *const c_char) -> c_int {
    unsafe {
        let mut value: c_int = 0;
        let mut c: c_int;
        let mut text_ptr = text;

        loop {
            c = *text_ptr as c_int;
            text_ptr = text_ptr.add(1);
            if c == 0 {
                break;
            }
            if c >= ('0' as c_int) && c <= ('9' as c_int) {
                value = value * 16 + c - ('0' as c_int);
                continue;
            }
            if c >= ('a' as c_int) && c <= ('f' as c_int) {
                value = value * 16 + 10 + c - ('a' as c_int);
                continue;
            }
            if c >= ('A' as c_int) && c <= ('F' as c_int) {
                value = value * 16 + 10 + c - ('A' as c_int);
                continue;
            }
        }

        return value;
    }
}
