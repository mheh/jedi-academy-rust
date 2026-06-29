#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

//! Mechanical port of `code/qcommon/files.h`.
//!
//! Structures local to the files_* modules.

use crate::code::qcommon::unzip_h::unzFile;
use crate::ffi::types::{fileHandle_t, qboolean};
use core::ffi::{c_char, c_float, c_int, c_ulong, c_void};

pub type FILE = c_void;

// Type definitions for Xbox-specific types
#[cfg(target_os = "xbox")]
pub type wfhandle_t = c_int;

#[cfg(target_os = "xbox")]
pub type GOBHandle = c_int;

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
}

pub const MAX_ZPATH: usize = 256;
pub const BASEGAME: &str = "base";

// Constants from q_shared.h
pub const MAX_OSPATH: usize = 260;
pub const MAX_QPATH: usize = 64;

/*
   Structures local to the files_* modules.
*/

#[repr(C)]
pub struct fileInPack_s {
    pub name: *mut c_char,          // name of the file
    pub pos: c_ulong,               // file info position in zip
    pub next: *mut fileInPack_s,    // next file in the hash
}

pub type fileInPack_t = fileInPack_s;

#[repr(C)]
pub struct pack_t {
    pub pakFilename: [c_char; MAX_OSPATH], // c:\quake3\base\asset0.pk3
    #[cfg(not(target_os = "xbox"))]
    pub handle: unzFile,
    pub checksum: c_int,
    pub numfiles: c_int,
    pub hashSize: c_int,                   // hash table size (power of 2)
    pub hashTable: *mut *mut fileInPack_t, // hash table
    pub buildBuffer: *mut fileInPack_t,    // buffer with the filenames etc.
}

#[repr(C)]
pub struct directory_t {
    pub path: [c_char; MAX_OSPATH],    // c:\stvoy
    pub gamedir: [c_char; MAX_OSPATH], // base
}

#[repr(C)]
pub struct searchpath_s {
    pub next: *mut searchpath_s,

    pub pack: *mut pack_t,      // only one of pack / dir will be non NULL
    pub dir: *mut directory_t,
}

pub type searchpath_t = searchpath_s;

pub const MAX_FILE_HANDLES: usize = 16;

#[repr(C)]
pub union qfile_gus {
    pub o: *mut FILE,
    pub z: unzFile,
}

pub type qfile_gut = qfile_gus;

#[repr(C)]
pub struct qfile_us {
    pub file: qfile_gut,
    pub unique: qboolean,
}

pub type qfile_ut = qfile_us;

#[repr(C)]
pub struct fileHandleData_t {
    pub handleFiles: qfile_ut,
    pub handleSync: qboolean,
    pub baseOffset: c_int,
    pub fileSize: c_int,
    pub zipFilePos: c_int,
    pub zipFile: qboolean,
    pub name: [c_char; MAX_QPATH],

    #[cfg(target_os = "xbox")]
    pub ghandle: GOBHandle,
    #[cfg(target_os = "xbox")]
    pub gob: qboolean,
    #[cfg(target_os = "xbox")]
    pub used: qboolean,
    #[cfg(target_os = "xbox")]
    pub whandle: wfhandle_t,
}

unsafe extern "C" {
    pub static mut fsh: [fileHandleData_t; MAX_FILE_HANDLES];

    pub static mut fs_searchpaths: *mut searchpath_t;
    pub static mut fs_gamedir: [c_char; MAX_OSPATH]; // this will be a single file name with no separators
    pub static fs_debug: *mut cvar_t;
    pub static fs_basepath: *mut cvar_t;
    pub static fs_cdpath: *mut cvar_t;
    pub static fs_copyfiles: *mut cvar_t;
    pub static fs_gamedirvar: *mut cvar_t;
    pub static fs_restrict: *mut cvar_t;
    pub static mut fs_readCount: c_int;    // total bytes read
    pub static mut fs_loadCount: c_int;    // total files read
    pub static mut fs_packFiles: c_int;    // total number of files in packs

    pub fn FS_Startup(gameName: *const c_char);
    pub fn FS_CreatePath(OSPath: *mut c_char);
    pub fn FS_BuildOSPath(
        base: *const c_char,
        game: *const c_char,
        qpath: *const c_char,
    ) -> *mut c_char;
    #[link_name = "FS_BuildOSPath"]
    pub fn FS_BuildOSPath_v2(qpath: *const c_char) -> *mut c_char;
    pub fn FS_HandleForFile() -> fileHandle_t;
    pub fn FS_FilenameCompare(s1: *const c_char, s2: *const c_char) -> qboolean;
    pub fn FS_SV_FOpenFileRead(filename: *const c_char, fp: *mut fileHandle_t) -> c_int;
    pub fn FS_Shutdown();
    pub fn FS_SetRestrictions();
    pub fn FS_CheckInit();
    pub fn FS_ReplaceSeparators(path: *mut c_char);
}
