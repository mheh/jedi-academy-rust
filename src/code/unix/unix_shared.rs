#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use std::ptr;

// Constants from included headers
const MAX_OSPATH: usize = 256;
const MAX_FOUND_FILES: usize = 0x1000;
const S_IFDIR: c_uint = 0o040000;

// Stub struct declarations for C interop matching Unix ABI
// timeval from sys/time.h
#[repr(C)]
pub struct timeval {
    pub tv_sec: i32,
    pub tv_usec: i32,
}

// timezone from sys/time.h
#[repr(C)]
pub struct timezone {
    pub tz_minuteswest: c_int,
    pub tz_dsttime: c_int,
}

// dirent from dirent.h - simplified for macOS
#[repr(C)]
pub struct dirent {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [c_char; 256],
}

// DIR is opaque
#[repr(C)]
pub struct DIR {
    _unused: [u8; 0],
}

// stat from sys/stat.h - simplified for macOS/Unix
#[repr(C)]
pub struct stat {
    pub st_dev: u64,
    pub st_mode: u32,
    pub st_ino: u64,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub st_atimespec: timespec,
    pub st_mtimespec: timespec,
    pub st_ctimespec: timespec,
    pub st_birthtimespec: timespec,
    pub st_size: i64,
    pub st_blocks: i64,
    pub st_blksize: i32,
    pub st_flags: u32,
    pub st_gen: u32,
    pub st_lspare: i32,
    pub st_qspare: [i64; 2],
}

#[repr(C)]
pub struct timespec {
    pub tv_sec: i32,
    pub tv_nsec: i32,
}

// External C library functions
extern "C" {
    pub fn gettimeofday(tv: *mut timeval, tz: *mut timezone) -> c_int;
    pub fn mkdir(path: *const c_char, mode: c_uint) -> c_int;
    pub fn tolower(c: c_int) -> c_int;
    pub fn opendir(dirname: *const c_char) -> *mut DIR;
    pub fn readdir(dirp: *mut DIR) -> *mut dirent;
    pub fn closedir(dirp: *mut DIR) -> c_int;
    pub fn stat(path: *const c_char, buf: *mut stat) -> c_int;
    pub fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char;
    pub fn getenv(name: *const c_char) -> *mut c_char;
    pub fn strerror(errnum: c_int) -> *mut c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;
}

// Game engine functions (from q_shared.h, qcommon.h)
extern "C" {
    pub fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...) -> usize;
    pub fn CopyString(in_: *const c_char) -> *mut c_char;
    pub fn Z_Malloc(size: usize) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    pub fn Q_strcat(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    pub fn Sys_Error(fmt: *const c_char, ...) -> !;
}

// Type aliases matching C
type qboolean = c_int;
const qtrue: c_int = 1;

// Constant from processor detection
const CPUID_GENERIC: c_int = 0;

// Used to determine CD Path
static mut programpath: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

//===============================================================================

/*
================
Sys_Milliseconds
================
*/
pub static mut curtime: c_int = 0;
pub static mut sys_timeBase: c_int = 0;

pub extern "C" fn Sys_Milliseconds() -> c_int {
    let mut tp: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut tzp: timezone = timezone {
        tz_minuteswest: 0,
        tz_dsttime: 0,
    };

    unsafe {
        gettimeofday(&mut tp, &mut tzp);

        if sys_timeBase == 0 {
            sys_timeBase = tp.tv_sec;
            return tp.tv_usec / 1000;
        }

        curtime = (tp.tv_sec - sys_timeBase) * 1000 + tp.tv_usec / 1000;

        return curtime;
    }
}

pub extern "C" fn Sys_Mkdir(path: *const c_char) {
    unsafe {
        mkdir(path, 0o777);
    }
}

pub extern "C" fn strlwr(mut s: *mut c_char) -> *mut c_char {
    unsafe {
        let start = s;
        while *s != 0 {
            *s = tolower(*s as c_int) as c_char;
            s = s.offset(1);
        }
        start
    }
}

//============================================

/* Like glob_match, but match PATTERN against any final segment of TEXT.  */
unsafe fn glob_match_after_star(mut pattern: *mut c_char, mut text: *mut c_char) -> c_int {
    let mut c: c_char;
    let mut c1: c_char;

    loop {
        c = *pattern;
        pattern = pattern.offset(1);
        if (c as u8) != b'?' && (c as u8) != b'*' {
            break;
        }
        if (c as u8) == b'?' && *text == 0 {
            return 0;
        }
        if (c as u8) == b'?' {
            text = text.offset(1);
        }
    }

    if c == 0 {
        return 1;
    }

    if (c as u8) == b'\\' {
        c1 = *pattern;
    } else {
        c1 = c;
    }

    loop {
        if ((c as u8) == b'[' || *text == c1) && glob_match(pattern.offset(-1), text) != 0 {
            return 1;
        }
        if *text == 0 {
            return 0;
        }
        text = text.offset(1);
    }
}

/* Return nonzero if PATTERN has any special globbing chars in it.  */
unsafe fn glob_pattern_p(mut pattern: *mut c_char) -> c_int {
    let mut c: c_char;
    let mut open: c_int = 0;

    loop {
        c = *pattern;
        pattern = pattern.offset(1);
        if c == 0 {
            break;
        }
        match c as u8 {
            b'?' | b'*' => {
                return 1;
            }

            b'[' => {
                open += 1; /* Only accept an open brace if there is a close */
                           /* brace to match it.  Bracket expressions must be */
                           /* complete, according to Posix.2 */
            }
            b']' => {
                if open != 0 {
                    return 1;
                }
            }

            b'\\' => {
                if *pattern == 0 {
                    return 0;
                }
                pattern = pattern.offset(1);
            }
            _ => {}
        }
    }

    return 0;
}

/* Match the pattern PATTERN against the string TEXT;
   return 1 if it matches, 0 otherwise.

   A match means the entire string TEXT is used up in matching.

   In the pattern string, `*' matches any sequence of characters,
   `?' matches any character, [SET] matches any character in the specified set,
   [!SET] matches any character not in the specified set.

   A set is composed of characters or ranges; a range looks like
   character hyphen character (as in 0-9 or A-Z).
   [0-9a-zA-Z_] is the set of characters allowed in C identifiers.
   Any other character in the pattern must be matched exactly.

   To suppress the special syntactic significance of any of `[]*?!-\',
   and match the character exactly, precede it with a `\'.
*/

pub extern "C" fn glob_match(mut pattern: *mut c_char, mut text: *mut c_char) -> c_int {
    unsafe {
        let mut c: c_char;

        loop {
            c = *pattern;
            pattern = pattern.offset(1);
            if c == 0 {
                break;
            }
            match c as u8 {
                b'?' => {
                    if *text == 0 {
                        return 0;
                    } else {
                        text = text.offset(1);
                    }
                }

                b'\\' => {
                    if *pattern != *text {
                        return 0;
                    }
                    pattern = pattern.offset(1);
                    text = text.offset(1);
                }

                b'*' => {
                    return glob_match_after_star(pattern, text);
                }

                b'[' => {
                    let c1 = *text;
                    text = text.offset(1);
                    if c1 == 0 {
                        return 0;
                    }

                    let invert = ((*pattern as u8) == b'!' || (*pattern as u8) == b'^') as c_int;
                    if invert != 0 {
                        pattern = pattern.offset(1);
                    }

                    let mut c = *pattern;
                    pattern = pattern.offset(1);
                    loop {
                        let mut cstart = c;
                        let mut cend = c;

                        if (c as u8) == b'\\' {
                            cstart = *pattern;
                            pattern = pattern.offset(1);
                            cend = cstart;
                        }
                        if c == 0 {
                            return 0;
                        }

                        c = *pattern;
                        pattern = pattern.offset(1);
                        if (c as u8) == b'-' && (*pattern as u8) != b']' {
                            cend = *pattern;
                            pattern = pattern.offset(1);
                            if (cend as u8) == b'\\' {
                                cend = *pattern;
                                pattern = pattern.offset(1);
                            }
                            if cend == 0 {
                                return 0;
                            }
                            c = *pattern;
                            pattern = pattern.offset(1);
                        }
                        if c1 >= cstart && c1 <= cend {
                            // Skip the rest of the [...] construct that already matched.
                            while (c as u8) != b']' {
                                if c == 0 {
                                    return 0;
                                }
                                c = *pattern;
                                pattern = pattern.offset(1);
                                if c == 0 {
                                    return 0;
                                } else if (c as u8) == b'\\' {
                                    pattern = pattern.offset(1);
                                }
                            }
                            if invert != 0 {
                                return 0;
                            }
                            break;
                        }
                        if (c as u8) == b']' {
                            break;
                        }
                    }
                    if invert == 0 {
                        return 0;
                    }
                }

                _ => {
                    if c != *text {
                        return 0;
                    }
                    text = text.offset(1);
                }
            }
        }

        return (*text == 0) as c_int;
    }
}

//============================================

pub extern "C" fn Sys_ListFiles(
    directory: *const c_char,
    extension: *const c_char,
    numfiles: *mut c_int,
    wantsubs: qboolean,
) -> *mut *mut c_char {
    unsafe {
        let mut d: *mut dirent;
        let mut dironly: qboolean = wantsubs;
        let mut search: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
        let mut nfiles: c_int = 0;
        let mut listCopy: *mut *mut c_char;
        let mut list: [*mut c_char; MAX_FOUND_FILES] = [ptr::null_mut(); MAX_FOUND_FILES];
        let mut i: c_int;
        let mut st: stat = std::mem::zeroed();

        let mut extLen: usize;

        let mut extension = extension;
        if extension.is_null() {
            extension = b"\0".as_ptr() as *const c_char;
        }

        if *extension as u8 == b'/' && *extension.offset(1) == 0 {
            extension = b"\0".as_ptr() as *const c_char;
            dironly = qtrue;
        }

        extLen = strlen(extension);

        // search
        nfiles = 0;

        let fdir = opendir(directory);
        if fdir.is_null() {
            *numfiles = 0;
            return ptr::null_mut();
        }

        loop {
            d = readdir(fdir);
            if d.is_null() {
                break;
            }
            Com_sprintf(
                search.as_mut_ptr(),
                std::mem::size_of_val(&search),
                b"%s/%s\0".as_ptr() as *const c_char,
                directory,
                (*d).d_name.as_ptr(),
            );
            if stat(search.as_mut_ptr(), &mut st) == -1 {
                continue;
            }
            if (dironly != 0 && (st.st_mode & S_IFDIR) == 0)
                || (dironly == 0 && (st.st_mode & S_IFDIR) != 0)
            {
                continue;
            }

            if *extension != 0 {
                if strlen((*d).d_name.as_ptr()) < extLen
                    || Q_stricmp(
                        ((*d).d_name.as_ptr() as *mut c_char)
                            .offset(strlen((*d).d_name.as_ptr()) as isize - extLen as isize),
                        extension,
                    ) != 0
                {
                    continue; // didn't match
                }
            }

            if nfiles == (MAX_FOUND_FILES - 1) as c_int {
                break;
            }
            list[nfiles as usize] = CopyString((*d).d_name.as_ptr());
            nfiles += 1;
        }

        list[nfiles as usize] = ptr::null_mut();

        closedir(fdir);

        // return a copy of the list
        *numfiles = nfiles;

        if nfiles == 0 {
            return ptr::null_mut();
        }

        listCopy = Z_Malloc(((nfiles + 1) as usize) * std::mem::size_of::<*mut c_char>())
            as *mut *mut c_char;
        i = 0;
        while i < nfiles {
            *listCopy.offset(i as isize) = list[i as usize];
            i += 1;
        }
        *listCopy.offset(i as isize) = ptr::null_mut();

        return listCopy;
    }
}

pub extern "C" fn Sys_FreeFileList(list: *mut *mut c_char) {
    unsafe {
        let mut i: c_int = 0;

        if list.is_null() {
            return;
        }

        while !(*list.offset(i as isize)).is_null() {
            Z_Free(*list.offset(i as isize) as *mut c_void);
            i += 1;
        }

        Z_Free(list as *mut c_void);
    }
}

pub extern "C" fn Sys_Cwd() -> *mut c_char {
    static mut cwd: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

    unsafe {
        getcwd(cwd.as_mut_ptr(), std::mem::size_of_val(&cwd) - 1);
        cwd[MAX_OSPATH - 1] = 0;

        return cwd.as_mut_ptr();
    }
}

pub extern "C" fn SetProgramPath(path: *mut c_char) {
    unsafe {
        let mut p: *mut c_char;

        Q_strncpyz(
            programpath.as_mut_ptr(),
            path,
            std::mem::size_of_val(&programpath),
        );
        p = strrchr(programpath.as_mut_ptr(), '/' as c_int);
        if !p.is_null() {
            *p = 0; // remove program name, leave only path
        }
    }
}

pub extern "C" fn Sys_DefaultCDPath() -> *mut c_char {
    unsafe {
        if programpath[0] != 0 {
            return programpath.as_mut_ptr();
        } else {
            return Sys_Cwd();
        }
    }
}

pub extern "C" fn Sys_DefaultBasePath() -> *mut c_char {
    unsafe {
        let mut p: *mut c_char;
        static mut basepath: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

        p = getenv(b"HOME\0".as_ptr() as *const c_char);
        if !p.is_null() {
            Q_strncpyz(
                basepath.as_mut_ptr(),
                p,
                std::mem::size_of_val(&basepath),
            );
            Q_strcat(
                basepath.as_mut_ptr(),
                std::mem::size_of_val(&basepath),
                b"/.q3a\0".as_ptr() as *const c_char,
            );
            if mkdir(basepath.as_mut_ptr(), 0o777) != 0 {
                if 0 != 17 {
                    // if (errno != EEXIST)
                    Sys_Error(
                        b"Unable to create directory \"%s\", error is %s(%d)\n\0".as_ptr()
                            as *const c_char,
                        basepath.as_mut_ptr(),
                        strerror(0),
                        0,
                    );
                }
            }
            return basepath.as_mut_ptr();
        }
        return b"\0".as_ptr() as *mut c_char; // assume current dir
    }
}

//============================================

pub extern "C" fn Sys_GetProcessorId() -> c_int {
    return CPUID_GENERIC;
}

pub extern "C" fn Sys_ShowConsole(_visLevel: c_int, _quitOnClose: qboolean) {}
