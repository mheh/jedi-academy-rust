use core::ffi::{c_int, c_char, c_void};
use std::ptr;

// Declarations for external dependencies - need to be resolved from other modules
// For now, these are stubs to match the C file structure
extern "C" {
    // From files.h and related modules
    fn FS_CheckInit();
    fn FS_HandleForFile() -> u32;
    fn FS_BuildOSPath(filename: *const c_char) -> *mut c_char;
    fn FS_BuildGOBPath(qpath: *const c_char) -> *mut c_char;
    fn FS_ReplaceSeparators(path: *mut c_char);

    // From qcommon/qcommon.h
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut c_void;
    fn Cvar_Set(name: *const c_char, value: *const c_char);

    // From game/q_shared.h
    fn S_ClearSoundBuffer();

    // From win32/win_file.h
    fn WF_Open(name: *const c_char, read: bool, asset: bool) -> i32;
    fn WF_Close(handle: i32);
    fn WF_Read(buffer: *mut c_void, len: c_int, handle: i32) -> c_int;
    fn WF_Write(buffer: *const c_void, len: c_int, handle: i32) -> c_int;
    fn WF_Seek(offset: c_int, seek_type: c_int, handle: i32) -> c_int;
    fn WF_Tell(handle: i32) -> c_int;
    fn WF_Resize(size: u32, handle: i32) -> bool;

    // From zlib
    fn inflateInit(stream: *mut z_stream) -> c_int;
    fn inflate(stream: *mut z_stream, flush: c_int) -> c_int;
    fn inflateEnd(stream: *mut z_stream) -> c_int;

    // From qcommon/z_memman.h
    fn Z_Malloc(size: u32, tag: c_int, qfalse: bool, lineno: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    // From sys/sys_main.c
    fn Sys_GetFileCode(filename: *const c_char) -> c_int;
    fn Sys_ListFiles(
        path: *mut c_char,
        extension: *const c_char,
        numfiles: *mut c_int,
        sort: bool,
    ) -> *mut *mut c_char;
    fn Sys_ListFiles_MP(
        path: *mut c_char,
        extension: *const c_char,
        filter: *const c_char,
        numfiles: *mut c_int,
        sort: bool,
    ) -> *mut *mut c_char;
    fn Sys_FreeFileList(filelist: *mut *mut c_char);
    fn Sys_DefaultCDPath() -> *const c_char;
    fn Sys_DefaultBasePath() -> *const c_char;
    fn Sys_Log(name: *const c_char, buffer: *const c_void, len: c_int, append: bool);

    // GOB library functions
    fn GOBInit(
        mem: *const GOBMemoryFuncSet,
        file: *const GOBFileSysFuncSet,
        codec: *const GOBCodecFuncSet,
        cache: *const GOBCacheFileFuncSet,
    ) -> c_int;
    fn GOBArchiveOpen(
        archive: *mut c_char,
        access: c_int,
        flag1: u32,
        flag2: u32,
    ) -> c_int;
    fn GOBOpen(name: *const c_char, handle: *mut u32) -> c_int;
    fn GOBClose(handle: u32) -> u32;
    fn GOBAccess(name: *const c_char, status: *mut u32) -> c_int;
    fn GOBRead(buffer: *mut c_void, size: c_int, handle: u32) -> u32;
    fn GOBWrite(buffer: *const c_void, size: c_int, handle: u32) -> u32;
    fn GOBSeek(handle: u32, offset: i32, seek_type: c_int, pos: *mut u32) -> u32;
    fn GOBSetCacheSize(size: c_int);
    fn GOBSetReadBufferSize(size: c_int);
    fn GOBSetProfileFuncs(profile: *const GOBProfileFuncSet);
    fn GOBStartProfile();
    fn LittleLong(l: u32) -> u32;
}

// Stub for strstr if needed
extern "C" {
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// Constants from zlib/z_stream
const Z_OK: c_int = 0;
const Z_STREAM_END: c_int = 1;
const Z_BUF_ERROR: c_int = -5;
const Z_FINISH: c_int = 4;

const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;

const ZI_STACKSIZE: usize = 64 * 1024;

const MAX_FILE_HANDLES: usize = 64;
const MAX_OSPATH: usize = 256;
const MAX_FOUND_FILES: usize = 0x1000;

// GOB constants
const GOBERR_OK: c_int = 0;
const GOBACCESS_READ: c_int = 1;
const GOBSEEK_START: c_int = 0;
const GOBSEEK_CURRENT: c_int = 1;
const GOBSEEK_END: c_int = 2;
const GOB_FALSE: u32 = 0;
const GOB_TRUE: u32 = 1;
const GOB_INVALID_SIZE: u32 = 0xFFFFFFFF;
const GOB_INFINITE_RATIO: f32 = 0.0;

// Tags
const TAG_FILESYS: c_int = 1;
const TAG_TEMP_WORKSPACE: c_int = 2;
const TAG_LISTFILES: c_int = 3;

// CVAR flags
const CVAR_INIT: c_int = 1;
const CVAR_SERVERINFO: c_int = 2;

// Error codes
const ERR_FATAL: c_int = 1;

// Filesystem modes
const FS_READ: c_int = 0;

// Filesystem seek modes
const FS_SEEK_SET: c_int = 0;
const FS_SEEK_CUR: c_int = 1;
const FS_SEEK_END: c_int = 2;

// GOB file handle type
type GOBFSHandle = u32;
type GOBChar = c_char;
type GOBInt32 = c_int;
type GOBUInt32 = u32;
type GOBVoid = c_void;
type GOBBool = u32;
type GOBAccessType = c_int;
type GOBSeekType = c_int;
type wfhandle_t = i32;
type fileHandle_t = u32;
type qboolean = bool;
type fsMode_t = c_int;

// zlib types
type uInt = u32;
type uLong = u32;
type Bytef = u8;
type voidpf = *mut c_void;

#[repr(C)]
struct z_stream {
    next_in: *mut Bytef,
    avail_in: uInt,
    total_in: uLong,
    next_out: *mut Bytef,
    avail_out: uInt,
    total_out: uLong,
    msg: *const c_char,
    state: *mut c_void,
    zalloc: Option<unsafe extern "C" fn(voidpf, uInt, uInt) -> voidpf>,
    zfree: Option<unsafe extern "C" fn(voidpf, voidpf) -> ()>,
    opaque: voidpf,
    data_type: c_int,
    adler: uLong,
    reserved: uLong,
}

#[repr(C)]
struct gi_handleTable {
    file: wfhandle_t,
    used: bool,
}

#[repr(C)]
struct GOBMemoryFuncSet {
    alloc: Option<unsafe extern "C" fn(GOBUInt32) -> *mut GOBVoid>,
    free: Option<unsafe extern "C" fn(*mut GOBVoid) -> ()>,
}

#[repr(C)]
struct GOBFileSysFuncSet {
    open: Option<unsafe extern "C" fn(*const GOBChar, GOBAccessType) -> GOBFSHandle>,
    close: Option<unsafe extern "C" fn(*mut GOBFSHandle) -> GOBBool>,
    read: Option<unsafe extern "C" fn(GOBFSHandle, *mut GOBVoid, GOBInt32) -> GOBInt32>,
    write: Option<unsafe extern "C" fn(GOBFSHandle, *const GOBVoid, GOBInt32) -> GOBInt32>,
    seek: Option<unsafe extern "C" fn(GOBFSHandle, GOBInt32, GOBSeekType) -> GOBInt32>,
}

#[repr(C)]
struct GOBCacheFileFuncSet {
    open: Option<unsafe extern "C" fn(GOBUInt32) -> GOBBool>,
    close: Option<unsafe extern "C" fn() -> GOBBool>,
    read: Option<unsafe extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32>,
    write: Option<unsafe extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32>,
    seek: Option<unsafe extern "C" fn(GOBInt32) -> GOBInt32>,
}

#[repr(C)]
struct GOBCodec {
    tag: c_char,
    ratio: f32,
    compress: *const c_void,
    decompress: *const c_void,
}

#[repr(C)]
struct GOBCodecFuncSet {
    numCodecs: c_int,
    codecs: [GOBCodec; 2],
}

#[repr(C)]
struct GOBProfileFuncSet {
    profileread: Option<unsafe extern "C" fn(GOBUInt32) -> ()>,
}

// File system handle table - external declaration
// This should be declared in another module (files_common.rs or similar)
extern "C" {
    static mut fsh: [filehandle_s; MAX_FILE_HANDLES];
}

#[repr(C)]
struct filehandle_s {
    used: qboolean,
    gob: qboolean,
    whandle: wfhandle_t,
    ghandle: GOBFSHandle,
}

static mut fs_openorder: *mut c_void = ptr::null_mut();
static mut fs_debug: *mut c_void = ptr::null_mut();
static mut fs_copyfiles: *mut c_void = ptr::null_mut();
static mut fs_cdpath: *mut c_void = ptr::null_mut();
static mut fs_basepath: *mut c_void = ptr::null_mut();
static mut fs_gamedirvar: *mut c_void = ptr::null_mut();
static mut fs_restrict: *mut c_void = ptr::null_mut();

// Zlib Tech Ref says decompression should use about 44kb.  I'll
// go with 64kb as a safety factor...
static mut zi_stackTop: *mut c_char = ptr::null_mut();
static mut zi_stackBase: *mut c_char = ptr::null_mut();

//GOB stuff
//===========================================================================

static mut gi_handles: *mut gi_handleTable = ptr::null_mut();
static mut gi_cacheHandle: c_int = 0;

unsafe extern "C" fn gi_open(name: *mut GOBChar, type_: GOBAccessType) -> GOBFSHandle {
    if type_ != GOBACCESS_READ {
        return 0xFFFFFFFF;
    }

    let mut f: c_int = 0;
    loop {
        if f >= MAX_FILE_HANDLES as c_int {
            break;
        }
        if !(*gi_handles.add(f as usize)).used {
            break;
        }
        f += 1;
    }

    if f == MAX_FILE_HANDLES as c_int {
        return 0xFFFFFFFF;
    }

    (*gi_handles.add(f as usize)).file =
        WF_Open(name, true, if !strstr(name, "assets.gob\0".as_ptr() as *const c_char).is_null() { true } else { false });
    if (*gi_handles.add(f as usize)).file < 0 {
        return 0xFFFFFFFF;
    }
    (*gi_handles.add(f as usize)).used = true;

    f as GOBFSHandle
}

unsafe extern "C" fn gi_close(handle: *mut GOBFSHandle) -> GOBBool {
    WF_Close((*gi_handles.add(*handle as usize)).file);
    (*gi_handles.add(*handle as usize)).used = false;
    GOB_TRUE
}

unsafe extern "C" fn gi_read(handle: GOBFSHandle, buffer: *mut GOBVoid, size: GOBInt32) -> GOBInt32 {
    WF_Read(buffer, size, (*gi_handles.add(handle as usize)).file)
}

unsafe extern "C" fn gi_seek(handle: GOBFSHandle, offset: GOBInt32, type_: GOBSeekType) -> GOBInt32 {
    let _type_ = match type_ {
        GOBSEEK_START => SEEK_SET,
        GOBSEEK_CURRENT => SEEK_CUR,
        GOBSEEK_END => SEEK_END,
        _ => {
            debug_assert!(false);
            SEEK_SET
        }
    };

    WF_Seek(offset, _type_, (*gi_handles.add(handle as usize)).file)
}

unsafe extern "C" fn gi_alloc(size: GOBUInt32) -> *mut GOBVoid {
    Z_Malloc(size, TAG_FILESYS, false, 32)
}

unsafe extern "C" fn gi_free(ptr: *mut GOBVoid) {
    Z_Free(ptr);
}

unsafe extern "C" fn cache_open(size: GOBUInt32) -> GOBBool {
    loop {
        if gi_cacheHandle >= MAX_FILE_HANDLES as c_int {
            break;
        }
        if !(*gi_handles.add(gi_cacheHandle as usize)).used {
            break;
        }
        gi_cacheHandle += 1;
    }

    if gi_cacheHandle == MAX_FILE_HANDLES as c_int {
        return GOB_FALSE;
    }

    (*gi_handles.add(gi_cacheHandle as usize)).file = WF_Open("z:\\jedi.swap\0".as_ptr() as *const c_char, false, true);
    if (*gi_handles.add(gi_cacheHandle as usize)).file < 0 {
        return GOB_FALSE;
    }

    if !WF_Resize(size, (*gi_handles.add(gi_cacheHandle as usize)).file) {
        WF_Close((*gi_handles.add(gi_cacheHandle as usize)).file);
        return GOB_FALSE;
    }

    (*gi_handles.add(gi_cacheHandle as usize)).used = true;

    GOB_TRUE
}

unsafe extern "C" fn cache_close() -> GOBBool {
    WF_Close((*gi_handles.add(gi_cacheHandle as usize)).file);
    (*gi_handles.add(gi_cacheHandle as usize)).used = false;
    GOB_TRUE
}

unsafe extern "C" fn cache_read(buffer: *mut GOBVoid, size: GOBInt32) -> GOBInt32 {
    WF_Read(buffer, size, (*gi_handles.add(gi_cacheHandle as usize)).file)
}

unsafe extern "C" fn cache_write(buffer: *mut GOBVoid, size: GOBInt32) -> GOBInt32 {
    WF_Write(buffer, size, (*gi_handles.add(gi_cacheHandle as usize)).file)
}

unsafe extern "C" fn cache_seek(offset: GOBInt32) -> GOBInt32 {
    WF_Seek(offset, SEEK_SET, (*gi_handles.add(gi_cacheHandle as usize)).file)
}

unsafe extern "C" fn zi_alloc(opaque: voidpf, items: uInt, size: uInt) -> voidpf {
    let ret = zi_stackTop;

    zi_stackTop = zi_stackTop.add((items * size) as usize);
    debug_assert!((zi_stackTop as usize) < (zi_stackBase as usize) + ZI_STACKSIZE);

    ret as voidpf
}

unsafe extern "C" fn zi_free(_opaque: voidpf, _address: voidpf) {}

unsafe extern "C" fn gi_decompress_zlib(
    source: *mut GOBVoid,
    sourceLen: GOBUInt32,
    dest: *mut GOBVoid,
    destLen: *mut GOBUInt32,
) -> GOBInt32 {
    // Copied and modified version of zlib's uncompress()...

    let mut stream: z_stream = core::mem::zeroed();
    let mut err: c_int;

    stream.next_in = source as *mut Bytef;
    stream.avail_in = sourceLen as uInt;

    stream.next_out = dest as *mut Bytef;
    stream.avail_out = *destLen as uInt;
    if stream.avail_out as uLong != *destLen as uLong {
        return Z_BUF_ERROR;
    }

    stream.zalloc = Some(zi_alloc);
    stream.zfree = Some(zi_free);
    zi_stackTop = zi_stackBase;

    err = inflateInit(&mut stream);
    if err != Z_OK {
        return err;
    }

    err = inflate(&mut stream, Z_FINISH);
    if err != Z_STREAM_END {
        inflateEnd(&mut stream);
        return if err == Z_OK { Z_BUF_ERROR } else { err };
    }
    *destLen = stream.total_out as GOBUInt32;

    err = inflateEnd(&mut stream);
    err
}

pub unsafe extern "C" fn gi_decompress_null(
    source: *mut GOBVoid,
    sourceLen: GOBUInt32,
    dest: *mut GOBVoid,
    destLen: *mut GOBUInt32,
) -> GOBInt32 {
    if sourceLen > *destLen {
        return -1;
    }
    *destLen = sourceLen;

    memcpy(dest, source, sourceLen as usize);
    0
}

#[cfg(feature = "GOB_PROFILE")]
unsafe extern "C" fn gi_profileread(code: GOBUInt32) {
    let code = LittleLong(code);
    Sys_Log("gob-prof.dat\0".as_ptr() as *const c_char, &code as *const _ as *const c_void, core::mem::size_of::<GOBUInt32>() as c_int, true);
}

//===========================================================================

unsafe extern "C" fn FS_CheckUsed(f: fileHandle_t) {
    if !(*addr_of_mut!(fsh).add(f as usize)).used {
        Com_Error(
            ERR_FATAL,
            "Filesystem call attempting to use invalid handle\n\0".as_ptr() as *const c_char,
        );
    }
}

pub unsafe extern "C" fn FS_filelength(f: fileHandle_t) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if (*addr_of_mut!(fsh).add(f as usize)).gob {
        let mut cur: GOBUInt32 = 0;
        let mut end: GOBUInt32 = 0;
        let mut crap: GOBUInt32 = 0;
        GOBSeek((*addr_of_mut!(fsh).add(f as usize)).ghandle, 0, GOBSEEK_CURRENT, &mut cur);
        GOBSeek((*addr_of_mut!(fsh).add(f as usize)).ghandle, 0, GOBSEEK_END, &mut end);
        GOBSeek((*addr_of_mut!(fsh).add(f as usize)).ghandle, cur as i32, GOBSEEK_START, &mut crap);

        end as c_int
    } else {
        let pos = WF_Tell((*addr_of_mut!(fsh).add(f as usize)).whandle);
        WF_Seek(0, SEEK_END, (*addr_of_mut!(fsh).add(f as usize)).whandle);
        let end = WF_Tell((*addr_of_mut!(fsh).add(f as usize)).whandle);
        WF_Seek(pos, SEEK_SET, (*addr_of_mut!(fsh).add(f as usize)).whandle);

        end
    }
}

pub unsafe extern "C" fn FS_FCloseFile(f: fileHandle_t) {
    FS_CheckInit();
    FS_CheckUsed(f);

    if (*addr_of_mut!(fsh).add(f as usize)).gob {
        GOBClose((*addr_of_mut!(fsh).add(f as usize)).ghandle);
    } else {
        WF_Close((*addr_of_mut!(fsh).add(f as usize)).whandle);
    }

    (*addr_of_mut!(fsh).add(f as usize)).used = false;
}

pub unsafe extern "C" fn FS_FOpenFileWrite(filename: *const c_char) -> fileHandle_t {
    FS_CheckInit();

    let f = FS_HandleForFile();

    let osname = FS_BuildOSPath(filename);
    (*addr_of_mut!(fsh).add(f as usize)).whandle = WF_Open(osname, false, false);
    if (*addr_of_mut!(fsh).add(f as usize)).whandle >= 0 {
        (*addr_of_mut!(fsh).add(f as usize)).used = true;
        (*addr_of_mut!(fsh).add(f as usize)).gob = false;
        return f;
    }

    0
}

/*
===========
FS_FOpenFileRead

Finds the file in the search path.
Returns filesize and an open FILE pointer.
Used for streaming data out of either a
separate file or a ZIP file.
===========
*/

unsafe extern "C" fn FS_FOpenFileReadOS(filename: *const c_char, f: fileHandle_t) -> c_int {
    if Sys_GetFileCode(filename) != -1 {
        let osname = FS_BuildOSPath(filename);
        (*addr_of_mut!(fsh).add(f as usize)).whandle = WF_Open(osname, true, false);
        if (*addr_of_mut!(fsh).add(f as usize)).whandle >= 0 {
            (*addr_of_mut!(fsh).add(f as usize)).used = true;
            (*addr_of_mut!(fsh).add(f as usize)).gob = false;
            return FS_filelength(f);
        }
    }
    -1
}

/*
===================
FS_BuildGOBPath

Qpath may have either forward or backwards slashes
===================
*/
unsafe extern "C" fn FS_BuildGOBPath(qpath: *const c_char) -> *mut c_char {
    static mut path: [[c_char; MAX_OSPATH]; 2] = [[0; MAX_OSPATH]; 2];
    static mut toggle: c_int = 0;

    toggle ^= 1; // flip-flop to allow two returns without clash

    if *qpath as u8 == b'\\' || *qpath as u8 == b'/' {
        Com_sprintf(
            path[toggle as usize].as_mut_ptr(),
            MAX_OSPATH as c_int,
            ".%s\0".as_ptr() as *const c_char,
            qpath,
        );
    } else {
        Com_sprintf(
            path[toggle as usize].as_mut_ptr(),
            MAX_OSPATH as c_int,
            ".\\\%s\0".as_ptr() as *const c_char,
            qpath,
        );
    }

    //	FS_ReplaceSeparators( path[toggle], '\\' );
    FS_ReplaceSeparators(path[toggle].as_mut_ptr());

    path[toggle].as_mut_ptr()
}

unsafe extern "C" fn FS_FOpenFileReadGOB(filename: *const c_char, f: fileHandle_t) -> c_int {
    let gobname = FS_BuildGOBPath(filename);
    if GOBOpen(gobname, &mut (*addr_of_mut!(fsh).add(f as usize)).ghandle) == GOBERR_OK {
        (*addr_of_mut!(fsh).add(f as usize)).used = true;
        (*addr_of_mut!(fsh).add(f as usize)).gob = true;
        return FS_filelength(f);
    }
    -1
}

/*
===========
FS_FOpenFileRead

Finds the file in the search path.
Returns filesize and an open FILE pointer.
Used for streaming data out of either a
separate file or a ZIP file.
===========
*/
pub unsafe extern "C" fn FS_FOpenFileRead(
    filename: *const c_char,
    file: *mut fileHandle_t,
    uniqueFILE: qboolean,
) -> c_int {
    FS_CheckInit();

    if file.is_null() {
        Com_Error(
            ERR_FATAL,
            "FS_FOpenFileRead: NULL 'file' parameter passed\n\0".as_ptr() as *const c_char,
        );
    }

    if filename.is_null() {
        Com_Error(
            ERR_FATAL,
            "FS_FOpenFileRead: NULL 'filename' parameter passed\n\0".as_ptr() as *const c_char,
        );
    }

    *file = FS_HandleForFile();

    let mut len: c_int;

    if (*(fs_openorder as *mut cvar_s)).integer == 0 {
        // Release mode -- read from GOB first
        len = FS_FOpenFileReadGOB(filename, *file);
        if len < 0 {
            len = FS_FOpenFileReadOS(filename, *file);
        }
    } else {
        // Debug mode -- external files override GOB
        len = FS_FOpenFileReadOS(filename, *file);
        if len < 0 {
            len = FS_FOpenFileReadGOB(filename, *file);
        }
    }

    if len >= 0 {
        return len;
    }

    Com_DPrintf("Can't find %s\n\0".as_ptr() as *const c_char, filename);

    *file = 0;
    -1
}

/*
=================
FS_Read

Properly handles partial reads
=================
*/
pub unsafe extern "C" fn FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if f == 0 {
        return 0;
    }

    if f <= 0 || f >= MAX_FILE_HANDLES as fileHandle_t {
        Com_Error(ERR_FATAL, "FS_Read: Invalid handle %d\n\0".as_ptr() as *const c_char, f);
    }

    if (*addr_of_mut!(fsh).add(f as usize)).gob {
        let size = GOBRead(buffer, len, (*addr_of_mut!(fsh).add(f as usize)).ghandle);
        if size == GOB_INVALID_SIZE {
            #[cfg(feature = "FINAL_BUILD")]
            {
                extern "C" {
                    fn ERR_DiscFail(flag: bool);
                }
                ERR_DiscFail(false);
            }
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                Com_Error(ERR_FATAL, "Failed to read from GOB\0".as_ptr() as *const c_char);
            }
        }
        size as c_int
    } else {
        WF_Read(buffer, len, (*addr_of_mut!(fsh).add(f as usize)).whandle)
    }
}

/*
MP has FS_Read2 which is supposed to do some extra logic.
We don't care, and just call FS_Read()
*/
pub unsafe extern "C" fn FS_Read2(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int {
    FS_Read(buffer, len, f)
}

/*
=================
FS_Write
=================
*/
pub unsafe extern "C" fn FS_Write(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if f == 0 {
        return 0;
    }

    if f <= 0 || f >= MAX_FILE_HANDLES as fileHandle_t {
        Com_Error(ERR_FATAL, "FS_Read: Invalid handle %d\n\0".as_ptr() as *const c_char, f);
    }

    if (*addr_of_mut!(fsh).add(f as usize)).gob {
        Com_Error(
            ERR_FATAL,
            "FS_Write: Cannot write to GOB files %d\n\0".as_ptr() as *const c_char,
            f,
        );
    } else {
        return WF_Write(buffer, len, (*addr_of_mut!(fsh).add(f as usize)).whandle);
    }

    0
}

/*
=================
FS_Seek

=================
*/
pub unsafe extern "C" fn FS_Seek(f: fileHandle_t, offset: c_int, origin: c_int) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if (*addr_of_mut!(fsh).add(f as usize)).gob {
        let _origin = match origin {
            FS_SEEK_CUR => GOBSEEK_CURRENT,
            FS_SEEK_END => GOBSEEK_END,
            FS_SEEK_SET => GOBSEEK_START,
            _ => {
                Com_Error(ERR_FATAL, "Bad origin in FS_Seek\n\0".as_ptr() as *const c_char);
                GOBSEEK_CURRENT
            }
        };

        let mut pos: GOBUInt32 = 0;
        GOBSeek((*addr_of_mut!(fsh).add(f as usize)).ghandle, offset, _origin, &mut pos);
        pos as c_int
    } else {
        let _origin = match origin {
            FS_SEEK_CUR => SEEK_CUR,
            FS_SEEK_END => SEEK_END,
            FS_SEEK_SET => SEEK_SET,
            _ => {
                Com_Error(ERR_FATAL, "Bad origin in FS_Seek\n\0".as_ptr() as *const c_char);
                SEEK_CUR
            }
        };

        WF_Seek(offset, _origin, (*addr_of_mut!(fsh).add(f as usize)).whandle)
    }
}

/*
=================
FS_Access
=================
*/
pub unsafe extern "C" fn FS_Access(filename: *const c_char) -> qboolean {
    let mut status: GOBBool = 0;

    FS_CheckInit();

    let gobname = FS_BuildGOBPath(filename);
    if GOBAccess(gobname, &mut status) != GOBERR_OK || status != GOB_TRUE {
        return Sys_GetFileCode(filename) != -1;
    }

    true
}

/*
======================================================================================

CONVENIENCE FUNCTIONS FOR ENTIRE FILES

======================================================================================
*/

#[cfg(feature = "_JK2MP")]
pub unsafe extern "C" fn FS_FileIsInPAK(filename: *const c_char, pChecksum: *mut c_int) -> c_int
#[cfg(not(feature = "_JK2MP"))]
pub unsafe extern "C" fn FS_FileIsInPAK(filename: *const c_char) -> c_int
{
    FS_CheckInit();

    if filename.is_null() {
        Com_Error(
            ERR_FATAL,
            "FS_FOpenFileRead: NULL 'filename' parameter passed\n\0".as_ptr() as *const c_char,
        );
    }

    let mut exists: GOBBool = 0;
    GOBAccess(filename as *const c_char, &mut exists);

    #[cfg(feature = "_JK2MP")]
    {
        *pChecksum = 0;
    }

    if exists != 0 { 1 } else { -1 }
}

/*
============
FS_ReadFile

Filename are relative to the quake search path
a null buffer will just return the file length without loading
============
*/
pub unsafe extern "C" fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int {
    FS_CheckInit();

    if qpath.is_null() || *qpath == 0 {
        Com_Error(ERR_FATAL, "FS_ReadFile with empty name\n\0".as_ptr() as *const c_char);
    }

    // stop sounds from repeating
    S_ClearSoundBuffer();

    let mut h: fileHandle_t = 0;
    let len = FS_FOpenFileRead(qpath, &mut h, false);
    if h == 0 {
        if !buffer.is_null() {
            *buffer = ptr::null_mut();
        }
        return -1;
    }

    if buffer.is_null() {
        FS_FCloseFile(h);
        return len;
    }

    // assume temporary....
    let buf = Z_Malloc((len + 1) as u32, TAG_TEMP_WORKSPACE, false, 32) as *mut u8;
    *buf.add(len as usize) = 0;

    //	Z_Label(buf, qpath);

    FS_Read(buf as *mut c_void, len, h);

    // guarantee that it will have a trailing 0 for string operations
    *buf.add(len as usize) = 0;
    FS_FCloseFile(h);

    *buffer = buf as *mut c_void;
    len
}

/*
=============
FS_FreeFile
=============
*/
pub unsafe extern "C" fn FS_FreeFile(buffer: *mut c_void) {
    FS_CheckInit();

    if buffer.is_null() {
        Com_Error(ERR_FATAL, "FS_FreeFile( NULL )\0".as_ptr() as *const c_char);
    }

    Z_Free(buffer);
}

pub unsafe extern "C" fn FS_FOpenFileByMode(
    qpath: *const c_char,
    f: *mut fileHandle_t,
    mode: fsMode_t,
) -> c_int {
    FS_CheckInit();

    if mode != FS_READ {
        Com_Error(ERR_FATAL, "FSH_FOpenFile: bad mode\0".as_ptr() as *const c_char);
        return -1;
    }

    FS_FOpenFileRead(qpath, f, true)
}

pub unsafe extern "C" fn FS_FTell(f: fileHandle_t) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if (*addr_of_mut!(fsh).add(f as usize)).gob {
        let mut pos: GOBUInt32 = 0;
        GOBSeek((*addr_of_mut!(fsh).add(f as usize)).ghandle, 0, GOBSEEK_CURRENT, &mut pos);
        pos as c_int
    } else {
        WF_Tell((*addr_of_mut!(fsh).add(f as usize)).whandle)
    }
}

/*
================
FS_Startup
================
*/
pub unsafe extern "C" fn FS_Startup(gameName: *const c_char) {
    Com_Printf("----- FS_Startup -----\n\0".as_ptr() as *const c_char);

    fs_openorder = Cvar_Get("fs_openorder\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, 0);
    fs_debug = Cvar_Get("fs_debug\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, 0);
    fs_copyfiles = Cvar_Get("fs_copyfiles\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_INIT);
    fs_cdpath = Cvar_Get(
        "fs_cdpath\0".as_ptr() as *const c_char,
        Sys_DefaultCDPath(),
        CVAR_INIT,
    );
    fs_basepath = Cvar_Get(
        "fs_basepath\0".as_ptr() as *const c_char,
        Sys_DefaultBasePath(),
        CVAR_INIT,
    );
    fs_gamedirvar = Cvar_Get(
        "fs_game\0".as_ptr() as *const c_char,
        "base\0".as_ptr() as *const c_char,
        CVAR_INIT | CVAR_SERVERINFO,
    );
    fs_restrict = Cvar_Get("fs_restrict\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, CVAR_INIT);

    gi_handles = Z_Malloc((MAX_FILE_HANDLES as u32) * (core::mem::size_of::<gi_handleTable>() as u32), TAG_FILESYS, false, 32)
        as *mut gi_handleTable;
    for f in 0..MAX_FILE_HANDLES {
        (*addr_of_mut!(fsh).add(f)).used = false;
        (*gi_handles.add(f)).used = false;
    }

    zi_stackBase = Z_Malloc(ZI_STACKSIZE as u32, TAG_FILESYS, false, 0) as *mut c_char;

    let mut mem: GOBMemoryFuncSet = core::mem::zeroed();
    mem.alloc = Some(gi_alloc);
    mem.free = Some(gi_free);

    let mut file: GOBFileSysFuncSet = core::mem::zeroed();
    file.close = Some(gi_close);
    file.open = Some(gi_open);
    file.read = Some(gi_read);
    file.seek = Some(gi_seek);
    file.write = None;

    let mut cache: GOBCacheFileFuncSet = core::mem::zeroed();
    cache.close = Some(cache_close);
    cache.open = Some(cache_open);
    cache.read = Some(cache_read);
    cache.seek = Some(cache_seek);
    cache.write = Some(cache_write);

    let mut codec: GOBCodecFuncSet = core::mem::zeroed();
    codec.numCodecs = 2;
    // Codec 0 - zlib
    codec.codecs[0].tag = 'z' as c_char;
    codec.codecs[0].ratio = GOB_INFINITE_RATIO;
    codec.codecs[0].compress = ptr::null();
    codec.codecs[0].decompress = gi_decompress_zlib as *const c_void;
    // Codec 1 - null
    codec.codecs[1].tag = '0' as c_char;
    codec.codecs[1].ratio = GOB_INFINITE_RATIO;
    codec.codecs[1].compress = ptr::null();
    codec.codecs[1].decompress = gi_decompress_null as *const c_void;

    #[cfg(target_os = "windows")]
    let gob_init_result = {
        GOBInit(&mem, &file, &codec, &cache)
    };
    #[cfg(not(target_os = "windows"))]
    let gob_init_result = {
        GOBInit(&mem, &file, &codec, ptr::null())
    };

    if gob_init_result != GOBERR_OK {
        Com_Error(ERR_FATAL, "Could not initialize GOB\0".as_ptr() as *const c_char);
    }

    let archive = FS_BuildOSPath("assets\0".as_ptr() as *const c_char);
    if GOBArchiveOpen(archive, GOBACCESS_READ, GOB_FALSE, GOB_TRUE) != GOBERR_OK {
        #[cfg(feature = "FINAL_BUILD")]
        {
            /*
            extern void ERR_DiscFail(bool);
            ERR_DiscFail(false);
            */
        }
        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            //Com_Error( ERR_FATAL, "Could not initialize GOB" );
            Cvar_Set("fs_openorder\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char);
        }
    }

    GOBSetCacheSize(1);
    GOBSetReadBufferSize(32 * 1024);

    #[cfg(feature = "GOB_PROFILE")]
    {
        let mut profile: GOBProfileFuncSet = core::mem::zeroed();
        profile.profileread = Some(gi_profileread);
        GOBSetProfileFuncs(&profile);
        GOBStartProfile();
    }

    Com_Printf("----------------------\n\0".as_ptr() as *const c_char);
}

/*
============================

DIRECTORY SCANNING FUCNTIONS

============================
*/

/*
==================
FS_AddFileToList
==================
*/
unsafe extern "C" fn FS_AddFileToList(
    name: *mut c_char,
    list: *mut *mut c_char,
    mut nfiles: c_int,
) -> c_int {
    if nfiles == MAX_FOUND_FILES as c_int - 1 {
        return nfiles;
    }
    for i in 0..nfiles {
        if stricmp(name, *list.add(i as usize)) == 0 {
            return nfiles; // allready in list
        }
    }
    //	list[nfiles] = CopyString( name );
    *list.add(nfiles as usize) =
        Z_Malloc((strlen(name) + 1) as u32, TAG_LISTFILES, false, 0) as *mut c_char;
    strcpy(*list.add(nfiles as usize), name);
    nfiles += 1;

    nfiles
}

/*
===============
FS_ListFiles

Returns a uniqued list of files that match the given criteria
from all search paths
===============
*/
pub unsafe extern "C" fn FS_ListFiles(
    path: *const c_char,
    extension: *const c_char,
    numfiles: *mut c_int,
) -> *mut *mut c_char {
    let mut netpath: *mut c_char;
    let mut numSysFiles: c_int = 0;
    let mut sysFiles: *mut *mut c_char;
    let name: *mut c_char;
    let mut nfiles: c_int = 0;
    let mut listCopy: *mut *mut c_char;
    let mut list: [*mut c_char; MAX_FOUND_FILES] = [ptr::null_mut(); MAX_FOUND_FILES];

    FS_CheckInit();

    if path.is_null() {
        *numfiles = 0;
        return ptr::null_mut();
    }

    // We don't do any fancy searchpath magic here, it's all in the meta-file
    // that Sys_ListFiles will return
    netpath = FS_BuildOSPath(path);
    #[cfg(feature = "_JK2MP")]
    {
        sysFiles = Sys_ListFiles_MP(netpath, extension, ptr::null(), &mut numSysFiles, false);
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        sysFiles = Sys_ListFiles(netpath, extension, &mut numSysFiles, false);
    }

    for i in 0..numSysFiles as usize {
        // unique the match
        nfiles = FS_AddFileToList(*sysFiles.add(i), list.as_mut_ptr(), nfiles);
    }
    Sys_FreeFileList(sysFiles);

    // return a copy of the list
    *numfiles = nfiles;

    if nfiles == 0 {
        return ptr::null_mut();
    }

    listCopy = Z_Malloc(
        ((nfiles + 1) as u32) * (core::mem::size_of::<*mut c_char>() as u32),
        TAG_LISTFILES,
        false,
        0,
    ) as *mut *mut c_char;
    for i in 0..nfiles as usize {
        *listCopy.add(i) = list[i];
    }
    *listCopy.add(nfiles as usize) = ptr::null_mut();

    listCopy
}

/*
=================
FS_FreeFileList
=================
*/
pub unsafe extern "C" fn FS_FreeFileList(filelist: *mut *mut c_char) {
    FS_CheckInit();

    if filelist.is_null() {
        return;
    }

    let mut i: c_int = 0;
    while !(*filelist.add(i as usize)).is_null() {
        Z_Free(*filelist.add(i as usize) as *mut c_void);
        i += 1;
    }

    Z_Free(filelist as *mut c_void);
}

/*
===============
FS_AddFileToListBuf
===============
*/
unsafe extern "C" fn FS_AddFileToListBuf(
    mut name: *mut c_char,
    listbuf: *mut c_char,
    bufsize: c_int,
    nfiles: c_int,
) -> c_int {
    let mut p: *mut c_char;

    if nfiles == MAX_FOUND_FILES as c_int - 1 {
        return nfiles;
    }

    if *name as u8 == b'/' || *name as u8 == b'\\' {
        name = name.add(1);
    }

    p = listbuf;
    while *p != 0 {
        if stricmp(name, p) == 0 {
            return nfiles; // already in list
        }
        p = p.add(strlen(p) + 1);
    }

    if (p as usize + strlen(name) + 2 - listbuf as usize) > bufsize as usize {
        return nfiles; // list is full
    }

    strcpy(p, name);
    p = p.add(strlen(p) + 1);
    *p = 0;

    nfiles + 1
}

/*
================
FS_GetFileList

Returns a uniqued list of files that match the given criteria
from all search paths
================
*/
pub unsafe extern "C" fn FS_GetFileList(
    path: *const c_char,
    extension: *const c_char,
    listbuf: *mut c_char,
    bufsize: c_int,
) -> c_int {
    let mut nfiles: c_int = 0;
    let mut netpath: *mut c_char;
    let mut numSysFiles: c_int = 0;
    let mut sysFiles: *mut *mut c_char;
    let name: *mut c_char;

    FS_CheckInit();

    if path.is_null() {
        return 0;
    }
    let extension = if extension.is_null() {
        "\0".as_ptr() as *const c_char
    } else {
        extension
    };

    // Prime the file list buffer
    *listbuf = 0;
    netpath = FS_BuildOSPath(path);
    #[cfg(feature = "_JK2MP")]
    {
        sysFiles = Sys_ListFiles_MP(netpath, extension, ptr::null(), &mut numSysFiles, false);
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        sysFiles = Sys_ListFiles(netpath, extension, &mut numSysFiles, false);
    }

    for i in 0..numSysFiles as usize {
        // unique the match
        name = *sysFiles.add(i);
        nfiles = FS_AddFileToListBuf(name, listbuf, bufsize, nfiles);
    }
    Sys_FreeFileList(sysFiles);

    nfiles
}

/*
=================
 Filesytem STUBS
=================
*/

pub unsafe extern "C" fn FS_ConditionalRestart(checksumFeed: c_int) -> qboolean {
    false
}

pub unsafe extern "C" fn FS_ClearPakReferences(flags: c_int) {}

pub unsafe extern "C" fn FS_LoadedPakNames() -> *const c_char {
    "\0".as_ptr() as *const c_char
}

pub unsafe extern "C" fn FS_ReferencedPakNames() -> *const c_char {
    "\0".as_ptr() as *const c_char
}

pub unsafe extern "C" fn FS_SetRestrictions() {}

#[cfg(feature = "_JK2MP")]
pub unsafe extern "C" fn FS_Restart(checksumFeed: c_int) {}
#[cfg(not(feature = "_JK2MP"))]
pub unsafe extern "C" fn FS_Restart() {}

pub unsafe extern "C" fn FS_FileExists(file: *const c_char) -> qboolean {
    debug_assert!(false, "FS_FileExists not implemented on Xbox");
    false
}

pub unsafe extern "C" fn FS_UpdateGamedir() {}

pub unsafe extern "C" fn FS_PureServerSetReferencedPaks(pakSums: *const c_char, pakNames: *const c_char) {}

pub unsafe extern "C" fn FS_PureServerSetLoadedPaks(pakSums: *const c_char, pakNames: *const c_char) {}

pub unsafe extern "C" fn FS_ReferencedPakChecksums() -> *const c_char {
    "\0".as_ptr() as *const c_char
}

pub unsafe extern "C" fn FS_LoadedPakChecksums() -> *const c_char {
    "\0".as_ptr() as *const c_char
}

// Stub cvar_s struct for accessing integer field
#[repr(C)]
struct cvar_s {
    name: *const c_char,
    string: *const c_char,
    resetString: *const c_char,
    latched_string: *const c_char,
    flags: c_int,
    modified: bool,
    modificationCount: c_int,
    value: f32,
    integer: c_int,
    next: *mut cvar_s,
    prev: *mut cvar_s,
}
