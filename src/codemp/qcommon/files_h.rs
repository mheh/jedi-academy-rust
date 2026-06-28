//! Mechanical port of `codemp/qcommon/files.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::qcommon::unzip_h::unzFile;
use crate::ffi::types::{fileHandle_t, qboolean};
use core::ffi::{c_char, c_float, c_int, c_ulong, c_void};

pub type FILE = c_void;

// Header-local stubs for q_shared.h/qcommon.h symbols not yet ported here.
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

pub const BASEGAME: &str = "base";
pub const DEMOGAME: &str = "demo";

// every time a new demo pk3 file is built, this checksum must be updated.
// the easiest way to get it is to just run the game and see what it spits out
pub const DEMO_PAK_CHECKSUM: c_ulong = 437558517u32 as c_ulong;

// if this is defined, the executable positively won't work with any paks other
// than the demo pak, even if productid is present.  This is only used for our
// last demo release to prevent the mac and linux users from using the demo
// executable with the production windows pak before the mac/linux products
// hit the shelves a little later
// NOW defined in build files
//#define PRE_RELEASE_TADEMO

pub const MAX_OSPATH: usize = 256; // q_shared.h fallback when PATH_MAX is not defined
pub const MAX_ZPATH: usize = 256;
pub const MAX_SEARCH_PATHS: usize = 4096;
pub const MAX_FILEHASH_SIZE: usize = 1024;
pub const MAX_FILE_HANDLES: usize = 64; // qcommon.h non-Xbox value

#[repr(C)]
#[derive(Debug)]
pub struct fileInPack_s {
    pub name: *mut c_char,          // name of the file
    pub pos: c_ulong,               // file info position in zip
    pub next: *mut fileInPack_s,    // next file in the hash
}

pub type fileInPack_t = fileInPack_s;

#[repr(C)]
#[derive(Debug)]
pub struct pack_t {
    pub pakFilename: [c_char; MAX_OSPATH], // c:\quake3\base\pak0.pk3
    pub pakBasename: [c_char; MAX_OSPATH], // pak0
    pub pakGamename: [c_char; MAX_OSPATH], // base
    pub handle: unzFile,                   // handle to zip file
    pub checksum: c_int,                   // regular checksum
    pub pure_checksum: c_int,              // checksum for pure
    pub numfiles: c_int,                   // number of files in pk3
    pub referenced: c_int,                 // referenced file flags
    pub hashSize: c_int,                   // hash table size (power of 2)
    pub hashTable: *mut *mut fileInPack_t, // hash table
    pub buildBuffer: *mut fileInPack_t,    // buffer with the filenames etc.
}

#[repr(C)]
#[derive(Debug)]
pub struct directory_t {
    pub path: [c_char; MAX_OSPATH],    // c:\jk2
    pub gamedir: [c_char; MAX_OSPATH], // base
}

#[repr(C)]
#[derive(Debug)]
pub struct searchpath_s {
    pub next: *mut searchpath_s,

    pub pack: *mut pack_t,      // only one of pack / dir will be non NULL
    pub dir: *mut directory_t,
}

pub type searchpath_t = searchpath_s;

#[repr(C)]
#[derive(Copy, Clone)]
pub union qfile_gus {
    pub o: *mut FILE,
    pub z: unzFile,
}

pub type qfile_gut = qfile_gus;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct qfile_us {
    pub file: qfile_gut,
    pub unique: qboolean,
}

pub type qfile_ut = qfile_us;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct fileHandleData_t {
    pub handleFiles: qfile_ut,
    pub handleSync: qboolean,
    pub baseOffset: c_int,
    pub fileSize: c_int,
    pub zipFilePos: c_int,
    pub zipFile: qboolean,
    pub streamed: qboolean,
    pub name: [c_char; MAX_ZPATH],
}

unsafe extern "C" {
    pub static mut fs_gamedir: [c_char; MAX_OSPATH]; // this will be a single file name with no separators
    pub static mut fs_debug: *mut cvar_t;
    pub static mut fs_homepath: *mut cvar_t;
    pub static mut fs_basepath: *mut cvar_t;
    pub static mut fs_basegame: *mut cvar_t;
    pub static mut fs_cdpath: *mut cvar_t;
    pub static mut fs_copyfiles: *mut cvar_t;
    pub static mut fs_gamedirvar: *mut cvar_t;
    pub static mut fs_restrict: *mut cvar_t;
    pub static mut fs_dirbeforepak: *mut cvar_t; //rww - when building search path, keep directories at top and insert pk3's under them
    pub static mut fs_searchpaths: *mut searchpath_t;
    pub static mut fs_readCount: c_int; // total bytes read
    pub static mut fs_loadCount: c_int; // total files read
    pub static mut fs_loadStack: c_int; // total files in memory
    pub static mut fs_packFiles: c_int; // total number of files in packs

    pub static mut fs_fakeChkSum: c_int;
    pub static mut fs_checksumFeed: c_int;

    pub static mut fsh: [fileHandleData_t; MAX_FILE_HANDLES];

    pub static mut initialized: qboolean;

    // never load anything from pk3 files that are not present at the server when pure
    pub static mut fs_numServerPaks: c_int;
    pub static mut fs_serverPaks: [c_int; MAX_SEARCH_PATHS]; // checksums
    pub static mut fs_serverPakNames: [*mut c_char; MAX_SEARCH_PATHS]; // pk3 names

    // only used for autodownload, to make sure the client has at least
    // all the pk3 files that are referenced at the server side
    pub static mut fs_numServerReferencedPaks: c_int;
    pub static mut fs_serverReferencedPaks: [c_int; MAX_SEARCH_PATHS]; // checksums
    pub static mut fs_serverReferencedPakNames: [*mut c_char; MAX_SEARCH_PATHS]; // pk3 names

    // last valid game folder used
    pub static mut lastValidBase: [c_char; MAX_OSPATH];
    pub static mut lastValidGame: [c_char; MAX_OSPATH];

    pub fn FS_Startup(gameName: *const c_char);
    pub fn FS_CreatePath(OSPath: *mut c_char) -> qboolean;
    pub fn FS_BuildOSPath(
        base: *const c_char,
        game: *const c_char,
        qpath: *const c_char,
    ) -> *mut c_char;
    #[link_name = "FS_BuildOSPath"]
    pub fn FS_BuildOSPath_qpath(qpath: *const c_char) -> *mut c_char;
    pub fn FS_HandleForFile() -> fileHandle_t;
    pub fn FS_FilenameCompare(s1: *const c_char, s2: *const c_char) -> qboolean;
    pub fn FS_SV_FOpenFileRead(filename: *const c_char, fp: *mut fileHandle_t) -> c_int;
    pub fn FS_Shutdown();
    pub fn FS_SetRestrictions();
    pub fn FS_CheckInit();
    pub fn FS_ReplaceSeparators(path: *mut c_char);
}
