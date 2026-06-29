#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// External dependencies - these are declared from other modules
extern "C" {
    fn FS_CheckInit();
    fn FS_HandleForFile() -> i32;  // fileHandle_t
    fn FS_BuildOSPath(filename: *const c_char) -> *mut c_char;
    fn FS_BuildGOBPath(qpath: *const c_char) -> *mut c_char;
    fn FS_ReplaceSeparators(path: *mut c_char);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, destsize: usize, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Z_Malloc(size: usize, tag: c_int, clear: bool, alignment: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn S_ClearSoundBuffer();
    fn Sys_GetFileCode(filename: *const c_char) -> c_int;
    fn Sys_DefaultCDPath() -> *const c_char;
    fn Sys_DefaultBasePath() -> *const c_char;
    #[cfg(not(feature = "_JK2MP"))]
    fn Sys_ListFiles(path: *const c_char, extension: *const c_char, numfiles: *mut c_int, sort: bool) -> *mut *mut c_char;
    #[cfg(feature = "_JK2MP")]
    fn Sys_ListFiles(path: *const c_char, extension: *const c_char, unk: *mut c_void, numfiles: *mut c_int, sort: bool) -> *mut *mut c_char;
    fn Sys_FreeFileList(filelist: *mut *mut c_char);
    fn WF_Open(filename: *const c_char, read: bool, cacheable: bool) -> i32;
    fn WF_Close(handle: i32);
    fn WF_Read(buffer: *mut c_void, len: c_int, handle: i32) -> c_int;
    fn WF_Write(buffer: *const c_void, len: c_int, handle: i32) -> c_int;
    fn WF_Seek(offset: c_int, origin: c_int, handle: i32) -> c_int;
    fn WF_Tell(handle: i32) -> c_int;
    fn WF_Resize(size: c_int, handle: i32) -> bool;
    fn GOBOpen(name: *const c_char, handle: *mut i32) -> c_int;
    fn GOBClose(handle: i32) -> c_int;
    fn GOBRead(buffer: *mut c_void, len: c_int, handle: i32) -> c_int;
    fn GOBSeek(handle: i32, offset: c_int, origin: c_int, pos: *mut c_int) -> c_int;
    fn GOBAccess(name: *mut c_char, status: *mut bool) -> c_int;
    fn GOBInit(mem: *const GOBMemoryFuncSet, file: *const GOBFileSysFuncSet, codec: *const GOBCodecFuncSet, cache: *const GOBCacheFileFuncSet) -> c_int;
    fn GOBArchiveOpen(name: *mut c_char, access: c_int, unk1: bool, unk2: bool) -> c_int;
    fn GOBSetCacheSize(size: c_int);
    fn GOBSetReadBufferSize(size: c_int);
    fn GOBSetProfileFuncs(profile: *const GOBProfileFuncSet);
    fn GOBStartProfile();
    fn inflateInit(stream: *mut z_stream) -> c_int;
    fn inflate(stream: *mut z_stream, flush: c_int) -> c_int;
    fn inflateEnd(stream: *mut z_stream) -> c_int;
    fn LittleLong(l: u32) -> u32;
    fn Sys_Log(name: *const c_char, buffer: *const c_void, size: usize, append: bool);
}

// C string functions as extern
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn stricmp(a: *const c_char, b: *const c_char) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// Type definitions
#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // Other fields defined in other modules
}

type fileHandle_t = i32;
type wfhandle_t = i32;
type GOBFSHandle = u32;
type GOBChar = c_char;
type GOBAccessType = c_int;
type GOBBool = c_int;
type GOBInt32 = c_int;
type GOBUInt32 = u32;
type GOBVoid = c_void;
type GOBSeekType = c_int;
type voidpf = *mut c_void;
type uInt = u32;

// GOB Constants
const GOBACCESS_READ: c_int = 0;
const GOBSEEK_START: c_int = 0;
const GOBSEEK_CURRENT: c_int = 1;
const GOBSEEK_END: c_int = 2;
const GOB_TRUE: GOBBool = 1;
const GOB_FALSE: GOBBool = 0;
const GOBERR_OK: c_int = 0;
const GOB_INVALID_SIZE: u32 = 0xFFFFFFFF;
const GOB_INFINITE_RATIO: c_int = 9999;

// File seek constants
const FS_SEEK_SET: c_int = 0;
const FS_SEEK_CUR: c_int = 1;
const FS_SEEK_END: c_int = 2;

// Seek constants
const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;

// Tags
const TAG_FILESYS: c_int = 7;
const TAG_TEMP_WORKSPACE: c_int = 5;
const TAG_LISTFILES: c_int = 13;

// FS Mode
type fsMode_t = c_int;
const FS_READ: fsMode_t = 0;

// zlib constants
const Z_OK: c_int = 0;
const Z_STREAM_END: c_int = 1;
const Z_BUF_ERROR: c_int = -5;
const Z_FINISH: c_int = 4;

// Other constants
const ERR_FATAL: c_int = 0;
const CVAR_INIT: c_int = 1;
const CVAR_SERVERINFO: c_int = 2;
const MAX_FILE_HANDLES: usize = 16;
const MAX_OSPATH: usize = 260;
const MAX_FOUND_FILES: usize = 0x1000;

// Function pointers for GOB callbacks
type GOBOpenFn = unsafe extern "C" fn(*const GOBChar, GOBAccessType) -> GOBFSHandle;
type GOBCloseFn = unsafe extern "C" fn(*mut GOBFSHandle) -> GOBBool;
type GOBReadFn = unsafe extern "C" fn(GOBFSHandle, *mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBSeekFn = unsafe extern "C" fn(GOBFSHandle, GOBInt32, GOBSeekType) -> GOBInt32;
type GOBAllocFn = unsafe extern "C" fn(GOBUInt32) -> *mut GOBVoid;
type GOBFreeFn = unsafe extern "C" fn(*mut GOBVoid);
type ZiAllocFn = unsafe extern "C" fn(voidpf, uInt, uInt) -> voidpf;
type ZiFreeFn = unsafe extern "C" fn(voidpf, voidpf);
type GOBDecompressFn = unsafe extern "C" fn(*mut GOBVoid, GOBUInt32, *mut GOBVoid, *mut GOBUInt32) -> GOBInt32;
type GOBCacheOpenFn = unsafe extern "C" fn(GOBUInt32) -> GOBBool;
type GOBCacheCloseFn = unsafe extern "C" fn() -> GOBBool;
type GOBCacheReadFn = unsafe extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBCacheWriteFn = unsafe extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBCacheSeekFn = unsafe extern "C" fn(GOBInt32) -> GOBInt32;
type GOBProfileReadFn = unsafe extern "C" fn(GOBUInt32);

#[repr(C)]
pub struct z_stream {
    pub next_in: *mut u8,
    pub avail_in: uInt,
    pub total_in: u32,
    pub next_out: *mut u8,
    pub avail_out: uInt,
    pub total_out: u32,
    pub msg: *mut c_char,
    pub state: *mut c_void,
    pub zalloc: Option<unsafe extern "C" fn(voidpf, uInt, uInt) -> voidpf>,
    pub zfree: Option<unsafe extern "C" fn(voidpf, voidpf)>,
    pub opaque: voidpf,
    pub data_type: c_int,
    pub adler: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct GOBMemoryFuncSet {
    pub alloc: Option<GOBAllocFn>,
    pub free: Option<GOBFreeFn>,
}

#[repr(C)]
pub struct GOBFileSysFuncSet {
    pub open: Option<GOBOpenFn>,
    pub close: Option<GOBCloseFn>,
    pub read: Option<GOBReadFn>,
    pub seek: Option<GOBSeekFn>,
    pub write: *mut c_void,
}

#[repr(C)]
pub struct GOBCacheFileFuncSet {
    pub open: Option<GOBCacheOpenFn>,
    pub close: Option<GOBCacheCloseFn>,
    pub read: Option<GOBCacheReadFn>,
    pub write: Option<GOBCacheWriteFn>,
    pub seek: Option<GOBCacheSeekFn>,
}

#[repr(C)]
pub struct GOBCodec {
    pub tag: c_char,
    pub ratio: c_int,
    pub compress: *mut c_void,
    pub decompress: Option<GOBDecompressFn>,
}

#[repr(C)]
pub struct GOBCodecFuncSet {
    pub num_codecs: c_int,
    pub codecs: [GOBCodec; 2],
}

#[repr(C)]
pub struct GOBProfileFuncSet {
    pub read: Option<GOBProfileReadFn>,
}

#[repr(C)]
pub struct gi_handleTable {
    pub file: wfhandle_t,
    pub used: bool,
}

#[repr(C)]
pub struct fileSystemHandle_t {
    pub used: bool,
    pub gob: bool,
    pub whandle: wfhandle_t,
    pub ghandle: i32,
}

// File system handle array - stub declaration
// The actual array would be defined in files.h
// We'll declare it as extern
extern "C" {
    static mut fsh: [fileSystemHandle_t; 16];  // MAX_FILE_HANDLES
}

static mut fs_openorder: *mut cvar_t = core::ptr::null_mut();

// Zlib Tech Ref says decompression should use about 44kb.  I'll
// go with 64kb as a safety factor...
const ZI_STACKSIZE: usize = 64 * 1024;

static mut zi_stackTop: *mut c_char = core::ptr::null_mut();
static mut zi_stackBase: *mut c_char = core::ptr::null_mut();

//GOB stuff
//===========================================================================

static mut gi_handles: *mut gi_handleTable = core::ptr::null_mut();
static mut gi_cacheHandle: c_int = 0;

unsafe extern "C" fn gi_open(name: *mut GOBChar, typ: GOBAccessType) -> GOBFSHandle {
    if typ != GOBACCESS_READ {
        return 0xFFFFFFFF;
    }

    let mut f: c_int = 0;
    while f < MAX_FILE_HANDLES as c_int {
        if (*gi_handles.add(f as usize)).used == false {
            break;
        }
        f += 1;
    }

    if f == MAX_FILE_HANDLES as c_int {
        return 0xFFFFFFFF;
    }

    (*gi_handles.add(f as usize)).file = WF_Open(name, true, if strstr(name, "assets.gob\0".as_ptr() as *const c_char) != core::ptr::null_mut() { true } else { false });
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

unsafe extern "C" fn gi_seek(handle: GOBFSHandle, offset: GOBInt32, typ: GOBSeekType) -> GOBInt32 {
    let _type = match typ {
        GOBSEEK_START => SEEK_SET,
        GOBSEEK_CURRENT => SEEK_CUR,
        GOBSEEK_END => SEEK_END,
        _ => {
            assert!(false, "Invalid seek type");
            SEEK_SET
        }
    };

    WF_Seek(offset, _type, (*gi_handles.add(handle as usize)).file)
}

unsafe extern "C" fn gi_alloc(size: GOBUInt32) -> *mut GOBVoid {
    Z_Malloc(size as usize, TAG_FILESYS, false, 32)
}

unsafe extern "C" fn gi_free(ptr: *mut GOBVoid) {
    Z_Free(ptr);
}

unsafe extern "C" fn cache_open(size: GOBUInt32) -> GOBBool {
    gi_cacheHandle = 0;
    while gi_cacheHandle < MAX_FILE_HANDLES as c_int {
        if (*gi_handles.add(gi_cacheHandle as usize)).used == false {
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

    if !WF_Resize(size as c_int, (*gi_handles.add(gi_cacheHandle as usize)).file) {
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

unsafe extern "C" fn zi_alloc(_opaque: voidpf, items: uInt, size: uInt) -> voidpf {
    let ret = zi_stackTop;

    zi_stackTop = zi_stackTop.add((items * size) as usize);
    assert!(zi_stackTop < zi_stackBase.add(ZI_STACKSIZE), "zi_stackTop overflow");

    ret as voidpf
}

unsafe extern "C" fn zi_free(_opaque: voidpf, _address: voidpf) {
}

pub unsafe extern "C" fn gi_decompress_zlib(
    source: *mut GOBVoid,
    sourceLen: GOBUInt32,
    dest: *mut GOBVoid,
    destLen: *mut GOBUInt32,
) -> GOBInt32 {
    // Copied and modified version of zlib's uncompress()...

    let mut stream: z_stream = core::mem::zeroed();
    let mut err: c_int;

    stream.next_in = source as *mut u8;
    stream.avail_in = sourceLen as uInt;

    stream.next_out = dest as *mut u8;
    stream.avail_out = *destLen as uInt;
    if (stream.avail_out as GOBUInt32) != *destLen {
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
    *destLen = stream.total_out;

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
    let code_le = LittleLong(code);
    Sys_Log("gob-prof.dat\0".as_ptr() as *const c_char, &code_le as *const _ as *const c_void, core::mem::size_of_val(&code_le), true);
}

//===========================================================================

static mut fs_debug: *mut cvar_t = core::ptr::null_mut();
static mut fs_copyfiles: *mut cvar_t = core::ptr::null_mut();
static mut fs_cdpath: *mut cvar_t = core::ptr::null_mut();
static mut fs_basepath: *mut cvar_t = core::ptr::null_mut();
static mut fs_gamedirvar: *mut cvar_t = core::ptr::null_mut();
static mut fs_restrict: *mut cvar_t = core::ptr::null_mut();

unsafe fn FS_CheckUsed(f: fileHandle_t) {
    if (*fsh.as_mut_ptr().add(f as usize)).used == false {
        Com_Error(ERR_FATAL, "Filesystem call attempting to use invalid handle\n\0".as_ptr() as *const c_char);
    }
}

pub unsafe fn FS_filelength(f: fileHandle_t) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if (*fsh.as_mut_ptr().add(f as usize)).gob {
        let mut cur: GOBUInt32 = 0;
        let mut end: GOBUInt32 = 0;
        let mut crap: GOBUInt32 = 0;
        GOBSeek((*fsh.as_mut_ptr().add(f as usize)).ghandle, 0, GOBSEEK_CURRENT, &mut (cur as c_int));
        GOBSeek((*fsh.as_mut_ptr().add(f as usize)).ghandle, 0, GOBSEEK_END, &mut (end as c_int));
        GOBSeek((*fsh.as_mut_ptr().add(f as usize)).ghandle, cur as c_int, GOBSEEK_START, &mut (crap as c_int));

        end as c_int
    } else {
        let pos = WF_Tell((*fsh.as_mut_ptr().add(f as usize)).whandle);
        WF_Seek(0, SEEK_END, (*fsh.as_mut_ptr().add(f as usize)).whandle);
        let end = WF_Tell((*fsh.as_mut_ptr().add(f as usize)).whandle);
        WF_Seek(pos, SEEK_SET, (*fsh.as_mut_ptr().add(f as usize)).whandle);

        end
    }
}

pub unsafe fn FS_FCloseFile(f: fileHandle_t) {
    FS_CheckInit();
    FS_CheckUsed(f);

    if (*fsh.as_mut_ptr().add(f as usize)).gob {
        GOBClose((*fsh.as_mut_ptr().add(f as usize)).ghandle);
    } else {
        WF_Close((*fsh.as_mut_ptr().add(f as usize)).whandle);
    }

    (*fsh.as_mut_ptr().add(f as usize)).used = false;
}

pub unsafe fn FS_FOpenFileWrite(filename: *const c_char) -> fileHandle_t {
    FS_CheckInit();

    let f = FS_HandleForFile();

    let osname = FS_BuildOSPath(filename);
    (*fsh.as_mut_ptr().add(f as usize)).whandle = WF_Open(osname, false, false);
    if (*fsh.as_mut_ptr().add(f as usize)).whandle >= 0 {
        (*fsh.as_mut_ptr().add(f as usize)).used = true;
        (*fsh.as_mut_ptr().add(f as usize)).gob = false;
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

unsafe fn FS_FOpenFileReadOS(filename: *const c_char, f: fileHandle_t) -> c_int {
    if Sys_GetFileCode(filename) != -1 {
        let osname = FS_BuildOSPath(filename);
        (*fsh.as_mut_ptr().add(f as usize)).whandle = WF_Open(osname, true, false);
        if (*fsh.as_mut_ptr().add(f as usize)).whandle >= 0 {
            (*fsh.as_mut_ptr().add(f as usize)).used = true;
            (*fsh.as_mut_ptr().add(f as usize)).gob = false;
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
unsafe fn FS_BuildGOBPath(qpath: *const c_char) -> *mut c_char {
    static mut path: [[c_char; MAX_OSPATH]; 2] = [[0; MAX_OSPATH]; 2];
    static mut toggle: c_int = 0;

    toggle ^= 1;  // flip-flop to allow two returns without clash

    let toggle_idx = (toggle & 1) as usize;
    if *qpath as u8 == b'\\' as u8 || *qpath as u8 == b'/' as u8 {
        // ".%s"
        path[toggle_idx][0] = b'.' as c_char;
        strcpy(path[toggle_idx].as_mut_ptr().add(1), qpath);
    } else {
        // ".\\%s"
        path[toggle_idx][0] = b'.' as c_char;
        path[toggle_idx][1] = b'\\' as c_char;
        strcpy(path[toggle_idx].as_mut_ptr().add(2), qpath);
    }

    //	FS_ReplaceSeparators( path[toggle], '\\' );
    FS_ReplaceSeparators(path[toggle_idx].as_mut_ptr());

    path[toggle_idx].as_mut_ptr()
}

unsafe fn FS_FOpenFileReadGOB(filename: *const c_char, f: fileHandle_t) -> c_int {
    let gobname = FS_BuildGOBPath(filename);
    if GOBOpen(gobname, &mut (*fsh.as_mut_ptr().add(f as usize)).ghandle) == GOBERR_OK {
        (*fsh.as_mut_ptr().add(f as usize)).used = true;
        (*fsh.as_mut_ptr().add(f as usize)).gob = true;
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
pub unsafe fn FS_FOpenFileRead(
    filename: *const c_char,
    file: *mut fileHandle_t,
    _uniqueFILE: bool,
) -> c_int {
    FS_CheckInit();

    if file == core::ptr::null_mut() {
        Com_Error(ERR_FATAL, "FS_FOpenFileRead: NULL 'file' parameter passed\n\0".as_ptr() as *const c_char);
    }

    if filename == core::ptr::null() {
        Com_Error(ERR_FATAL, "FS_FOpenFileRead: NULL 'filename' parameter passed\n\0".as_ptr() as *const c_char);
    }

    *file = FS_HandleForFile();

    let len = if (*fs_openorder).integer == 0 {
        // Release mode -- read from GOB first
        let mut len = FS_FOpenFileReadGOB(filename, *file);
        if len < 0 {
            len = FS_FOpenFileReadOS(filename, *file);
        }
        len
    } else {
        // Debug mode -- external files override GOB
        let mut len = FS_FOpenFileReadOS(filename, *file);
        if len < 0 {
            len = FS_FOpenFileReadGOB(filename, *file);
        }
        len
    };

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
pub unsafe fn FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if f == 0 {
        return 0;
    }

    if f <= 0 || f >= MAX_FILE_HANDLES as c_int {
        Com_Error(ERR_FATAL, "FS_Read: Invalid handle %d\n\0".as_ptr() as *const c_char, f);
    }

    if (*fsh.as_mut_ptr().add(f as usize)).gob {
        let size = GOBRead(buffer, len, (*fsh.as_mut_ptr().add(f as usize)).ghandle) as GOBUInt32;
        if size == GOB_INVALID_SIZE {
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                Com_Error(ERR_FATAL, "Failed to read from GOB\0".as_ptr() as *const c_char);
            }
            #[cfg(feature = "FINAL_BUILD")]
            {
                /*
                extern void ERR_DiscFail(bool);
                ERR_DiscFail(false);
                */
            }
        }
        return size as c_int;
    } else {
        return WF_Read(buffer, len, (*fsh.as_mut_ptr().add(f as usize)).whandle);
    }
}

/*
    MP has FS_Read2 which is supposed to do some extra logic.
    We don't care, and just call FS_Read()
*/
pub unsafe fn FS_Read2(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int {
    FS_Read(buffer, len, f)
}

/*
=================
FS_Write
=================
*/
pub unsafe fn FS_Write(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if f == 0 {
        return 0;
    }

    if f <= 0 || f >= MAX_FILE_HANDLES as c_int {
        Com_Error(ERR_FATAL, "FS_Read: Invalid handle %d\n\0".as_ptr() as *const c_char, f);
    }

    if (*fsh.as_mut_ptr().add(f as usize)).gob {
        Com_Error(ERR_FATAL, "FS_Write: Cannot write to GOB files %d\n\0".as_ptr() as *const c_char, f);
    } else {
        return WF_Write(buffer, len, (*fsh.as_mut_ptr().add(f as usize)).whandle);
    }

    0
}

/*
=================
FS_Seek

=================
*/
pub unsafe fn FS_Seek(f: fileHandle_t, offset: c_int, origin: c_int) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if (*fsh.as_mut_ptr().add(f as usize)).gob {
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
        GOBSeek((*fsh.as_mut_ptr().add(f as usize)).ghandle, offset, _origin, &mut (pos as c_int));
        return pos as c_int;
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

        return WF_Seek(offset, _origin, (*fsh.as_mut_ptr().add(f as usize)).whandle);
    }
}

/*
=================
FS_Access
=================
*/
pub unsafe fn FS_Access(filename: *const c_char) -> bool {
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
pub unsafe fn FS_FileIsInPAK(filename: *const c_char, pChecksum: *mut c_int) -> c_int {
    FS_CheckInit();

    if filename == core::ptr::null() {
        Com_Error(ERR_FATAL, "FS_FOpenFileRead: NULL 'filename' parameter passed\n\0".as_ptr() as *const c_char);
    }

    let mut exists: GOBBool = 0;
    GOBAccess(filename as *mut GOBChar, &mut exists);

    *pChecksum = 0;

    return if exists != 0 { 1 } else { -1 };
}

#[cfg(not(feature = "_JK2MP"))]
pub unsafe fn FS_FileIsInPAK(filename: *const c_char) -> c_int {
    FS_CheckInit();

    if filename == core::ptr::null() {
        Com_Error(ERR_FATAL, "FS_FOpenFileRead: NULL 'filename' parameter passed\n\0".as_ptr() as *const c_char);
    }

    let mut exists: GOBBool = 0;
    GOBAccess(filename as *mut GOBChar, &mut exists);

    return if exists != 0 { 1 } else { -1 };
}

/*
============
FS_ReadFile

Filename are relative to the quake search path
a null buffer will just return the file length without loading
============
*/
pub unsafe fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int {
    FS_CheckInit();

    if qpath == core::ptr::null() || *qpath as u8 == 0 {
        Com_Error(ERR_FATAL, "FS_ReadFile with empty name\n\0".as_ptr() as *const c_char);
    }

    // stop sounds from repeating
    S_ClearSoundBuffer();

    let mut h: fileHandle_t = 0;
    let len = FS_FOpenFileRead(qpath, &mut h, false);
    if h == 0 {
        if buffer != core::ptr::null_mut() {
            *buffer = core::ptr::null_mut();
        }
        return -1;
    }

    if buffer == core::ptr::null_mut() {
        FS_FCloseFile(h);
        return len;
    }

    // assume temporary....
    let buf = Z_Malloc((len + 1) as usize, TAG_TEMP_WORKSPACE, false, 32) as *mut u8;
    *(buf.add(len as usize)) = 0;

    //	Z_Label(buf, qpath);

    FS_Read(buf as *mut c_void, len, h);

    // guarantee that it will have a trailing 0 for string operations
    *(buf.add(len as usize)) = 0;
    FS_FCloseFile(h);

    *buffer = buf as *mut c_void;
    len
}

/*
=============
FS_FreeFile
=============
*/
pub unsafe fn FS_FreeFile(buffer: *mut c_void) {
    FS_CheckInit();

    if buffer == core::ptr::null_mut() {
        Com_Error(ERR_FATAL, "FS_FreeFile( NULL )\0".as_ptr() as *const c_char);
    }

    Z_Free(buffer);
}

pub unsafe fn FS_FOpenFileByMode(qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t) -> c_int {
    FS_CheckInit();

    if mode != FS_READ {
        Com_Error(ERR_FATAL, "FSH_FOpenFile: bad mode\0".as_ptr() as *const c_char);
        return -1;
    }

    FS_FOpenFileRead(qpath, f, true)
}

pub unsafe fn FS_FTell(f: fileHandle_t) -> c_int {
    FS_CheckInit();
    FS_CheckUsed(f);

    if (*fsh.as_mut_ptr().add(f as usize)).gob {
        let mut pos: GOBUInt32 = 0;
        GOBSeek((*fsh.as_mut_ptr().add(f as usize)).ghandle, 0, GOBSEEK_CURRENT, &mut (pos as c_int));
        return pos as c_int;
    } else {
        return WF_Tell((*fsh.as_mut_ptr().add(f as usize)).whandle);
    }
}

/*
================
FS_Startup
================
*/
pub unsafe fn FS_Startup(gameName: *const c_char) {
    Com_Printf("----- FS_Startup -----\n\0".as_ptr() as *const c_char);

    fs_openorder = Cvar_Get("fs_openorder\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, 0);
    fs_debug = Cvar_Get("fs_debug\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, 0);
    fs_copyfiles = Cvar_Get("fs_copyfiles\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_INIT);
    fs_cdpath = Cvar_Get("fs_cdpath\0".as_ptr() as *const c_char, Sys_DefaultCDPath(), CVAR_INIT);
    fs_basepath = Cvar_Get("fs_basepath\0".as_ptr() as *const c_char, Sys_DefaultBasePath(), CVAR_INIT);
    fs_gamedirvar = Cvar_Get("fs_game\0".as_ptr() as *const c_char, "base\0".as_ptr() as *const c_char, CVAR_INIT | CVAR_SERVERINFO);
    fs_restrict = Cvar_Get("fs_restrict\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, CVAR_INIT);

    gi_handles = Z_Malloc(core::mem::size_of::<gi_handleTable>() * MAX_FILE_HANDLES, TAG_FILESYS, true, 1) as *mut gi_handleTable;
    for f in 0..MAX_FILE_HANDLES {
        (*fsh.as_mut_ptr().add(f)).used = false;
        (*gi_handles.add(f)).used = false;
    }

    zi_stackBase = Z_Malloc(ZI_STACKSIZE, TAG_FILESYS, false, 1) as *mut c_char;

    let mut mem: GOBMemoryFuncSet = core::mem::zeroed();
    mem.alloc = Some(gi_alloc);
    mem.free = Some(gi_free);

    let mut file: GOBFileSysFuncSet = core::mem::zeroed();
    file.close = Some(gi_close);
    file.open = Some(gi_open);
    file.read = Some(gi_read);
    file.seek = Some(gi_seek);
    file.write = core::ptr::null_mut();

    let mut cache: GOBCacheFileFuncSet = core::mem::zeroed();
    cache.close = Some(cache_close);
    cache.open = Some(cache_open);
    cache.read = Some(cache_read);
    cache.seek = Some(cache_seek);
    cache.write = Some(cache_write);

    let mut codec: GOBCodecFuncSet = core::mem::zeroed();
    codec.num_codecs = 2;  // codecs
    codec.codecs[0].tag = b'z' as c_char;
    codec.codecs[0].ratio = GOB_INFINITE_RATIO;
    codec.codecs[0].compress = core::ptr::null_mut();
    codec.codecs[0].decompress = Some(gi_decompress_zlib);
    codec.codecs[1].tag = b'0' as c_char;
    codec.codecs[1].ratio = GOB_INFINITE_RATIO;
    codec.codecs[1].compress = core::ptr::null_mut();
    codec.codecs[1].decompress = Some(gi_decompress_null);

    #[cfg(target_os = "windows")]
    let gob_result = GOBInit(&mem, &file, &codec, &cache);
    #[cfg(not(target_os = "windows"))]
    let gob_result = GOBInit(&mem, &file, &codec, core::ptr::null());

    if gob_result != GOBERR_OK {
        Com_Error(ERR_FATAL, "Could not initialize GOB\0".as_ptr() as *const c_char);
    }

    let archive = FS_BuildOSPath("assets\0".as_ptr() as *const c_char);
    if GOBArchiveOpen(archive, GOBACCESS_READ, GOB_FALSE != 0, GOB_TRUE != 0) != GOBERR_OK {
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
        profile.read = Some(gi_profileread);
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
unsafe fn FS_AddFileToList(
    name: *mut c_char,
    list: *mut *mut c_char,
    nfiles: c_int,
) -> c_int {
    if nfiles == MAX_FOUND_FILES_SIZE as c_int - 1 {
        return nfiles;
    }
    for i in 0..nfiles {
        if stricmp(name, *list.add(i as usize)) == 0 {
            return nfiles;  // allready in list
        }
    }
    //	list[nfiles] = CopyString( name );
    *list.add(nfiles as usize) = Z_Malloc(strlen(name) + 1, TAG_LISTFILES, false, 1) as *mut c_char;
    strcpy(*list.add(nfiles as usize), name);
    nfiles + 1
}

/*
===============
FS_ListFiles

Returns a uniqued list of files that match the given criteria
from all search paths
===============
*/
pub unsafe fn FS_ListFiles(
    path: *const c_char,
    extension: *const c_char,
    numfiles: *mut c_int,
) -> *mut *mut c_char {
    let mut netpath: *mut c_char;
    let mut numSysFiles: c_int = 0;
    let mut sysFiles: *mut *mut c_char;
    let mut name: *mut c_char;
    let mut nfiles: c_int = 0;
    let mut listCopy: *mut *mut c_char;
    let mut list: [*mut c_char; MAX_FOUND_FILES] = [core::ptr::null_mut(); MAX_FOUND_FILES];

    FS_CheckInit();

    if path == core::ptr::null() {
        *numfiles = 0;
        return core::ptr::null_mut();
    }

    // We don't do any fancy searchpath magic here, it's all in the meta-file
    // that Sys_ListFiles will return
    netpath = FS_BuildOSPath(path);
    #[cfg(feature = "_JK2MP")]
    {
        sysFiles = Sys_ListFiles(netpath, extension, core::ptr::null_mut(), &mut numSysFiles, false);
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        sysFiles = Sys_ListFiles(netpath, extension, &mut numSysFiles, false);
    }

    for i in 0..numSysFiles {
        // unique the match
        name = *sysFiles.add(i as usize);
        nfiles = FS_AddFileToList(name, list.as_mut_ptr(), nfiles);
    }
    Sys_FreeFileList(sysFiles);

    // return a copy of the list
    *numfiles = nfiles;

    if nfiles == 0 {
        return core::ptr::null_mut();
    }

    listCopy = Z_Malloc(((nfiles + 1) as usize) * core::mem::size_of::<*mut c_char>(), TAG_LISTFILES, false, 1) as *mut *mut c_char;
    for i in 0..nfiles {
        *listCopy.add(i as usize) = list[i as usize];
    }
    *listCopy.add(nfiles as usize) = core::ptr::null_mut();

    listCopy
}

/*
=================
FS_FreeFileList
=================
*/
pub unsafe fn FS_FreeFileList(filelist: *mut *mut c_char) {
    FS_CheckInit();

    if filelist == core::ptr::null_mut() {
        return;
    }

    let mut i = 0;
    while !(*filelist.add(i)).is_null() {
        Z_Free(*filelist.add(i) as *mut c_void);
        i += 1;
    }

    Z_Free(filelist as *mut c_void);
}

/*
===============
FS_AddFileToListBuf
===============
*/
unsafe fn FS_AddFileToListBuf(
    name: *mut c_char,
    listbuf: *mut c_char,
    bufsize: c_int,
    nfiles: c_int,
) -> c_int {
    let mut p: *mut c_char;

    if nfiles == MAX_FOUND_FILES as c_int - 1 {
        return nfiles;
    }

    let mut name_ptr = name;
    if *name as u8 == b'/' as u8 || *name as u8 == b'\\' as u8 {
        name_ptr = name.add(1);
    }

    p = listbuf;
    loop {
        if *p as u8 == 0 {
            break;
        }
        if stricmp(name_ptr, p) == 0 {
            return nfiles;  // already in list
        }
        p = p.add(strlen(p) + 1);
    }

    if ((p as usize - listbuf as usize) + strlen(name_ptr) + 2) > bufsize as usize {
        return nfiles;  // list is full
    }

    strcpy(p, name_ptr);
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
pub unsafe fn FS_GetFileList(
    path: *const c_char,
    mut extension: *const c_char,
    listbuf: *mut c_char,
    bufsize: c_int,
) -> c_int {
    let mut nfiles: c_int = 0;
    let mut i: c_int;
    let mut netpath: *mut c_char;
    let mut numSysFiles: c_int = 0;
    let mut sysFiles: *mut *mut c_char;
    let mut name: *mut c_char;

    FS_CheckInit();

    if path == core::ptr::null() {
        return 0;
    }
    if extension == core::ptr::null() {
        extension = "\0".as_ptr() as *const c_char;
    }

    // Prime the file list buffer
    *listbuf = 0;
    netpath = FS_BuildOSPath(path);
    #[cfg(feature = "_JK2MP")]
    {
        sysFiles = Sys_ListFiles(netpath, extension, core::ptr::null_mut(), &mut numSysFiles, false);
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        sysFiles = Sys_ListFiles(netpath, extension, &mut numSysFiles, false);
    }

    i = 0;
    while i < numSysFiles {
        // unique the match
        name = *sysFiles.add(i as usize);
        nfiles = FS_AddFileToListBuf(name, listbuf, bufsize, nfiles);
        i += 1;
    }
    Sys_FreeFileList(sysFiles);

    nfiles
}

/*
=================
 Filesytem STUBS
=================
*/

pub unsafe fn FS_ConditionalRestart(_checksumFeed: c_int) -> bool {
    false
}

pub unsafe fn FS_ClearPakReferences(_flags: c_int) {
}

pub unsafe fn FS_LoadedPakNames() -> *const c_char {
    "\0".as_ptr() as *const c_char
}

pub unsafe fn FS_ReferencedPakNames() -> *const c_char {
    "\0".as_ptr() as *const c_char
}

pub unsafe fn FS_SetRestrictions() {
}

#[cfg(feature = "_JK2MP")]
pub unsafe fn FS_Restart(_checksumFeed: c_int) {
}

#[cfg(not(feature = "_JK2MP"))]
pub unsafe fn FS_Restart() {
}

pub unsafe fn FS_FileExists(_file: *const c_char) -> bool {
    assert!(false, "FS_FileExists not implemented on Xbox");
    false
}

pub unsafe fn FS_UpdateGamedir() {
}

pub unsafe fn FS_PureServerSetReferencedPaks(_pakSums: *const c_char, _pakNames: *const c_char) {
}

pub unsafe fn FS_PureServerSetLoadedPaks(_pakSums: *const c_char, _pakNames: *const c_char) {
}

pub unsafe fn FS_ReferencedPakChecksums() -> *const c_char {
    "\0".as_ptr() as *const c_char
}

pub unsafe fn FS_LoadedPakChecksums() -> *const c_char {
    "\0".as_ptr() as *const c_char
}
