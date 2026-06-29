use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr;
use core::mem;

// System includes translated to extern declarations
extern "C" {
    // From sys/time.h
    fn gettimeofday(tv: *mut libc::timeval, tz: *mut libc::timezone) -> c_int;

    // From sys/stat.h / unistd.h
    fn mkdir(path: *const c_char, mode: c_int) -> c_int;

    // From dirent.h
    fn opendir(name: *const c_char) -> *mut libc::DIR;
    fn readdir(dir: *mut libc::DIR) -> *mut libc::dirent;
    fn closedir(dir: *mut libc::DIR) -> c_int;

    // From sys/stat.h
    fn stat(path: *const c_char, buf: *mut libc::stat) -> c_int;

    // From unistd.h
    fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char;
    fn getuid() -> libc::uid_t;

    // From stdlib.h
    fn getenv(name: *const c_char) -> *mut c_char;

    // From string.h
    fn strerror(errnum: c_int) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;

    // From errno.h - errno is a function that returns &mut c_int on POSIX
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn __errno_location() -> *mut c_int;

    // From pwd.h
    fn getpwuid(uid: libc::uid_t) -> *mut libc::passwd;

    // From ctype.h
    fn tolower(c: c_int) -> c_int;

    // From math.h
    fn rint(x: f64) -> f64;

    // Game/engine functions from qcommon and q_shared
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn Q_stricmp(s0: *const c_char, s1: *const c_char) -> c_int;
    fn Com_FilterPath(filter: *const c_char, name: *const c_char, casesensitive: c_int) -> c_int;
    fn CopyString(in_: *const c_char) -> *mut c_char;
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void) -> ();
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize) -> ();
    fn Q_strcat(dest: *mut c_char, size: usize, fmt: *const c_char, ...) -> ();
    fn Sys_Error(fmt: *const c_char, ...) -> !;
}

const MAX_OSPATH: usize = 256;

const MAX_FOUND_FILES: usize = 0x1000;
const TAG_FILESYS: c_int = 0; // TAG_FILESYS placeholder

//=============================================================================

// Used to determine CD Path
static mut cdPath: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

// Used to determine local installation path
static mut installPath: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

// Used to determine where to store user-specific files
static mut homePath: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

//DWORD timeGetTime(void)
//{
//	return Sys_Milliseconds();
//}

/*
================
Sys_Milliseconds
================
*/
static mut curtime: c_int = 0;
static mut sys_timeBase: c_int = 0;

#[allow(non_snake_case)]
pub unsafe fn Sys_Milliseconds(baseTime: c_int) -> c_int {
    let mut tp: libc::timeval = mem::zeroed();
    let mut tzp: libc::timezone = mem::zeroed();

    gettimeofday(&mut tp, &mut tzp);

    if sys_timeBase == 0 {
        sys_timeBase = tp.tv_sec as c_int;
        return (tp.tv_usec / 1000) as c_int;
    }

    curtime = ((tp.tv_sec - sys_timeBase as libc::time_t) as c_int * 1000) + (tp.tv_usec / 1000) as c_int;

    static mut sys_timeBase_local: c_int = 0;
    sys_timeBase_local = curtime;
    if baseTime == 0 {
        curtime -= sys_timeBase_local;
    }

    return curtime;

}


// #if 0 // bk001215 - see snapvector.nasm for replacement
#[cfg(target_os = "macos")] // rcg010206 - using this for PPC builds...
#[allow(non_snake_case)]
pub fn fastftol(f: f32) -> c_long { // bk001213 - from win32/win_shared.c
  //static int tmp;
  //	__asm fld f
  //__asm fistp tmp
  //__asm mov eax, tmp
  return f as c_long;
}

#[cfg(target_os = "macos")]
#[allow(non_snake_case)]
pub unsafe fn Sys_SnapVector3(v: *mut f32) { // bk001213 - see win32/win_shared.c
  // bk001213 - old linux
  *v.offset(0) = rint(*v.offset(0) as f64) as f32;
  *v.offset(1) = rint(*v.offset(1) as f64) as f32;
  *v.offset(2) = rint(*v.offset(2) as f64) as f32;
}
// #endif


#[allow(non_snake_case)]
pub unsafe fn Sys_Mkdir(path: *const c_char) {
    mkdir(path, 0o777);
}

#[allow(non_snake_case)]
pub unsafe fn strlwr(mut s: *mut c_char) -> *mut c_char {
  if s.is_null() { // bk001204 - paranoia
    assert!(false);
    return s;
  }
  while *s != 0 {
    *s = tolower(*s as c_int) as c_char;
    s = s.offset(1);
  }
  return s; // bk001204 - duh
}

//============================================

#[allow(non_snake_case)]
const MAX_FOUND_FILES_CONST: usize = 0x1000;

// bk001129 - new in 1.26
#[allow(non_snake_case)]
pub unsafe fn Sys_ListFilteredFiles(basedir: *const c_char, subdirs: *mut c_char, filter: *const c_char, list: *mut *mut c_char, numfiles: *mut c_int) {
    let mut search: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut newsubdirs: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut filename: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut fdir: *mut libc::DIR;
    let mut d: *mut libc::dirent;
    let mut st: libc::stat = mem::zeroed();

    if *numfiles >= MAX_FOUND_FILES as c_int - 1 {
        return;
    }

    if strlen(subdirs) != 0 {
        Com_sprintf(search.as_mut_ptr(), mem::size_of_val(&search), b"%s/%s\0".as_ptr() as *const c_char, basedir, subdirs);
    }
    else {
        Com_sprintf(search.as_mut_ptr(), mem::size_of_val(&search), b"%s\0".as_ptr() as *const c_char, basedir);
    }

    fdir = opendir(search.as_ptr());
    if fdir.is_null() {
        return;
    }

    loop {
        d = readdir(fdir);
        if d.is_null() {
            break;
        }
        Com_sprintf(filename.as_mut_ptr(), mem::size_of_val(&filename), b"%s/%s\0".as_ptr() as *const c_char, search.as_ptr(), (*d).d_name.as_ptr());
        if stat(filename.as_ptr(), &mut st) == -1 {
            continue;
        }

        if ((*st).st_mode & libc::S_IFDIR as libc::mode_t) != 0 {
            if Q_stricmp((*d).d_name.as_ptr(), b".\0".as_ptr() as *const c_char) != 0 && Q_stricmp((*d).d_name.as_ptr(), b"..\0".as_ptr() as *const c_char) != 0 {
                if strlen(subdirs) != 0 {
                    Com_sprintf(newsubdirs.as_mut_ptr(), mem::size_of_val(&newsubdirs), b"%s/%s\0".as_ptr() as *const c_char, subdirs, (*d).d_name.as_ptr());
                }
                else {
                    Com_sprintf(newsubdirs.as_mut_ptr(), mem::size_of_val(&newsubdirs), b"%s\0".as_ptr() as *const c_char, (*d).d_name.as_ptr());
                }
                Sys_ListFilteredFiles(basedir, newsubdirs.as_mut_ptr(), filter, list, numfiles);
            }
        }
        if *numfiles >= MAX_FOUND_FILES as c_int - 1 {
            break;
        }
        Com_sprintf(filename.as_mut_ptr(), mem::size_of_val(&filename), b"%s/%s\0".as_ptr() as *const c_char, subdirs, (*d).d_name.as_ptr());
        if Com_FilterPath(filter, filename.as_ptr(), 0) == 0 {
            continue;
        }
        *list.offset(*numfiles as isize) = CopyString(filename.as_ptr());
        *numfiles += 1;
    }

    closedir(fdir);
}

// bk001129 - in 1.17 this used to be
// char **Sys_ListFiles( const char *directory, const char *extension, int *numfiles, qboolean wantsubs )
#[allow(non_snake_case)]
pub unsafe fn Sys_ListFiles(directory: *const c_char, extension: *const c_char, filter: *mut c_char, numfiles: *mut c_int, wantsubs: c_int) -> *mut *mut c_char {
    let mut d: *mut libc::dirent;
    // char *p; // bk001204 - unused
    let mut fdir: *mut libc::DIR;
    let mut dironly: c_int = wantsubs;
    let mut search: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut nfiles: c_int;
    let mut listCopy: *mut *mut c_char;
    let mut list: [*mut c_char; MAX_FOUND_FILES] = [ptr::null_mut(); MAX_FOUND_FILES];
    //int			flag; // bk001204 - unused
    let mut i: c_int;
    let mut st: libc::stat = mem::zeroed();

    let mut extLen: c_int;

    if !filter.is_null() {

        nfiles = 0;
        Sys_ListFilteredFiles(directory, b"\0".as_ptr() as *mut c_char, filter, list.as_mut_ptr(), &mut nfiles);

        *list.as_mut_ptr().offset(nfiles as isize) = ptr::null_mut();
        *numfiles = nfiles;

        if nfiles == 0 {
            return ptr::null_mut();
        }

        listCopy = Z_Malloc(((nfiles + 1) * mem::size_of::<*mut c_char>()) as usize, TAG_FILESYS, 0) as *mut *mut c_char;
        i = 0;
        while i < nfiles {
            *listCopy.offset(i as isize) = list[i as usize];
            i += 1;
        }
        *listCopy.offset(i as isize) = ptr::null_mut();

        return listCopy;
    }

    let mut ext_ptr: *const c_char = extension;

    if ext_ptr.is_null() {
        ext_ptr = b"\0".as_ptr() as *const c_char;
    }

    if *ext_ptr as u8 == b'/' as u8 && *ext_ptr.offset(1) as u8 == 0 {
        ext_ptr = b"\0".as_ptr() as *const c_char;
        dironly = 1;
    }

    extLen = strlen(ext_ptr) as c_int;

    // search
    nfiles = 0;

    fdir = opendir(directory);
    if fdir.is_null() {
        *numfiles = 0;
        return ptr::null_mut();
    }

    loop {
        d = readdir(fdir);
        if d.is_null() {
            break;
        }
        Com_sprintf(search.as_mut_ptr(), mem::size_of_val(&search), b"%s/%s\0".as_ptr() as *const c_char, directory, (*d).d_name.as_ptr());
        if stat(search.as_ptr(), &mut st) == -1 {
            continue;
        }
        if (dironly != 0 && ((*st).st_mode & libc::S_IFDIR as libc::mode_t) == 0) ||
            (dironly == 0 && ((*st).st_mode & libc::S_IFDIR as libc::mode_t) != 0) {
            continue;
        }

        if *ext_ptr as u8 != 0 {
            if strlen((*d).d_name.as_ptr()) < strlen(ext_ptr) ||
                Q_stricmp(
                    (*d).d_name.as_ptr().offset(strlen((*d).d_name.as_ptr()) as isize - strlen(ext_ptr) as isize),
                    ext_ptr) != 0 {
                continue; // didn't match
            }
        }

        if nfiles == MAX_FOUND_FILES as c_int - 1 {
            break;
        }
        list[nfiles as usize] = CopyString((*d).d_name.as_ptr());
        nfiles += 1;
    }

    *list.as_mut_ptr().offset(nfiles as isize) = ptr::null_mut();

    closedir(fdir);

    // return a copy of the list
    *numfiles = nfiles;

    if nfiles == 0 {
        return ptr::null_mut();
    }

    listCopy = Z_Malloc(((nfiles + 1) * mem::size_of::<*mut c_char>()) as usize, TAG_FILESYS, 0) as *mut *mut c_char;
    i = 0;
    while i < nfiles {
        *listCopy.offset(i as isize) = list[i as usize];
        i += 1;
    }
    *listCopy.offset(i as isize) = ptr::null_mut();

    return listCopy;
}

#[allow(non_snake_case)]
pub unsafe fn Sys_FreeFileList(list: *mut *mut c_char) {
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

#[allow(non_snake_case)]
pub unsafe fn Sys_Cwd() -> *mut c_char {
    static mut cwd: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

    getcwd(cwd.as_mut_ptr(), mem::size_of_val(&cwd) - 1);
    cwd[MAX_OSPATH-1] = 0;

    return cwd.as_mut_ptr();
}

#[allow(non_snake_case)]
pub unsafe fn Sys_SetDefaultCDPath(path: *const c_char) {
    Q_strncpyz(cdPath.as_mut_ptr(), path, mem::size_of_val(&cdPath));
}

#[allow(non_snake_case)]
pub unsafe fn Sys_DefaultCDPath() -> *mut c_char {
    return cdPath.as_mut_ptr();
}

#[allow(non_snake_case)]
pub unsafe fn Sys_SetDefaultInstallPath(path: *const c_char) {
    Q_strncpyz(installPath.as_mut_ptr(), path, mem::size_of_val(&installPath));
}

#[allow(non_snake_case)]
pub unsafe fn Sys_DefaultInstallPath() -> *mut c_char {
    if *installPath.as_ptr() as u8 != 0 {
        return installPath.as_mut_ptr();
    } else {
        return Sys_Cwd();
    }
}

#[allow(non_snake_case)]
pub unsafe fn Sys_SetDefaultHomePath(path: *const c_char) {
    Q_strncpyz(homePath.as_mut_ptr(), path, mem::size_of_val(&homePath));
}

#[allow(non_snake_case)]
pub unsafe fn Sys_DefaultHomePath() -> *mut c_char {
    let mut p: *mut c_char;

    if *homePath.as_ptr() as u8 != 0 {
        return homePath.as_mut_ptr();
    }

    p = getenv(b"HOME\0".as_ptr() as *const c_char);
    if !p.is_null() {
        Q_strncpyz(homePath.as_mut_ptr(), p, mem::size_of_val(&homePath));
        #[cfg(target_os = "macos")]
        {
            Q_strcat(homePath.as_mut_ptr(), mem::size_of_val(&homePath), b"/Library/Application Support/Quake3\0".as_ptr() as *const c_char);
        }
        #[cfg(not(target_os = "macos"))]
        {
            Q_strcat(homePath.as_mut_ptr(), mem::size_of_val(&homePath), b"/.ja\0".as_ptr() as *const c_char);
        }
        if mkdir(homePath.as_ptr(), 0o777) != 0 {
            // Get errno value
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            {
                let err = unsafe { *__errno_location() };
                if err != libc::EEXIST {
                    Sys_Error(b"Unable to create directory \"%s\", error is %s(%d)\n\0".as_ptr() as *const c_char, homePath.as_ptr(), strerror(err), err);
                }
            }
            #[cfg(not(any(target_os = "linux", target_os = "macos")))]
            {
                // For other platforms, just assume error and report
                Sys_Error(b"Unable to create directory \"%s\"\n\0".as_ptr() as *const c_char, homePath.as_ptr());
            }
        }
        return homePath.as_mut_ptr();
    }
    return b"\0".as_ptr() as *mut c_char; // assume current dir
}

//============================================

#[allow(non_snake_case)]
pub fn Sys_GetProcessorId() -> c_int {
    return 0; // CPUID_GENERIC
}

#[allow(non_snake_case)]
pub fn Sys_ShowConsole(visLevel: c_int, quitOnClose: c_int) {
}

#[allow(non_snake_case)]
pub unsafe fn Sys_GetCurrentUser() -> *mut c_char {
    let mut p: *mut libc::passwd;

    p = getpwuid(getuid());
    if p.is_null() {
        return b"player\0".as_ptr() as *mut c_char;
    }
    return (*p).pw_name.as_mut_ptr();
}
