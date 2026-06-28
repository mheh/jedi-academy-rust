/*****************************************
 *
 * GOB File System
 *
 * Here's what Merriam-Webster says about "gob":  --Chuck
 *		Entry:     gob
 *		Function:  noun
 *		Etymology: Middle English gobbe, from Middle French gobe large piece of food,
 *		           back-formation from gobet
 *		Date:      14th century
 *		1 : LUMP
 *		2 : a large amount -- usually used in plural <gobs of money>
 *
 * Purpose: Provide fast, efficient disk access on a variety of platforms.
 *
 * Implementation:
 *		The GOB system maintains two files -- GOB and GFC.  The GOB file is actually
 *		an archive of many files split into variable size, compressed blocks.  The GFC,
 *		GOB File Control, contains 3 tables -- a block table, basic file table, and
 *		extended file table.  The block table is analogous to a DOS FAT.  The basic
 *		file table contains a minimal set of file information to handle basic reading
 *		tasks.  The extended file table is optionally loaded and contains additional
 *		file information.  File names are case insensitive.
 *
 *		Files can be read in a normal manner.  Open, read, seek and close
 *		operations are all provided.  Files can only be written in a single
 *		contiguous chunk of blocks at the end of an archive.  Reads are processed
 *		through a configurable number of read ahead buffers to in an effort to
 *		minimize both reads and seeks.  Other operations including delete, verify,
 *		access, and get size are also supported on files inside an archive.
 *
 *		The system supports read profiling.  By supplying a file read callback
 *		function, the library will output the block number of each read.  This can
 *		be used rearrange block in the archive to minimize seek times.  The
 *		GOBRearrange sorts files in an archive.
 *
 *		Supports block based caching.  Primarily aimed at caching files off a DVD/CD
 *		to a faster hard disk.
 *
 * Future Work:
 *
 * Dependencies: vvInt, snprintf, zlib
 * Owner: Chris McEvoy
 * History:
 *     09/23/2001 Original version
 *     10/28/2002 Merged into vvtech
 *
 * Copyright (C) 2002, Vicarious Visions, Inc.  All Rights Reserved.
 *
 * UNPUBLISHED -- Rights  reserved  under  the  copyright  laws  of the
 * United States.  Use  of a copyright notice is precautionary only and
 * does not imply publication or disclosure.
 *
 * THIS DOCUMENTATION CONTAINS CONFIDENTIAL AND PROPRIETARY INFORMATION
 * OF    VICARIOUS   VISIONS,  INC.    ANY  DUPLICATION,  MODIFICATION,
 * DISTRIBUTION, OR DISCLOSURE IS STRICTLY PROHIBITED WITHOUT THE PRIOR
 * EXPRESS WRITTEN PERMISSION OF VICARIOUS VISIONS, INC.
 *
 *****************************************/

/*
    This is an unofficial branch of GOB, for Jedi Academy
    Maintainer: Brian Osman
*/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_ulong};

pub const GOB_MAGIC_IDENTIFIER: u32 = 0x8008;
pub const GOB_MAX_FILE_NAME_LEN: usize = 96;
pub const GOB_MAX_OPEN_FILES: usize = 16;
pub const GOB_MAX_CODECS: usize = 2;
pub const GOB_INFINITE_RATIO: u32 = 1000;
pub const GOB_READ_RETRYS: usize = 3;

pub const GOB_MAX_FILES: usize = 16 * 1024;
pub const GOB_MAX_BLOCKS: i32 = 32767;

pub const GOB_BLOCK_SIZE: usize = 64 * 1024;
pub const GOB_BLOCK_ALIGNMENT: usize = 2048;
pub const GOB_MEM_ALIGNMENT: usize = 64;
pub const GOB_COMPRESS_OVERHEAD: usize = 1024;

pub const GOB_INVALID_SIZE: u32 = 0xFFFFFFFF;
pub const GOB_INVALID_BLOCK: u32 = 0xFFFFFFFF;

pub const GOB_TRUE: i32 = 1;
pub const GOB_FALSE: i32 = 0;

pub const GOBERR_OK: i32 = 0;
pub const GOBERR_NOT_INIT: i32 = 1;
pub const GOBERR_FILE_NOT_FOUND: i32 = 2;
pub const GOBERR_FILE_READ: i32 = 3;
pub const GOBERR_FILE_WRITE: i32 = 4;
pub const GOBERR_NO_MEMORY: i32 = 5;
pub const GOBERR_ALREADY_INIT: i32 = 6;
pub const GOBERR_ALREADY_OPEN: i32 = 7;
pub const GOBERR_INVALID_ACCESS: i32 = 8;
pub const GOBERR_NOT_GOB_FILE: i32 = 9;
pub const GOBERR_NOT_OPEN: i32 = 10;
pub const GOBERR_CANNOT_CREATE: i32 = 11;
pub const GOBERR_TOO_MANY_OPEN: i32 = 12;
pub const GOBERR_INVALID_SEEK: i32 = 13;
pub const GOBERR_TOO_MANY_FILES: i32 = 14;
pub const GOBERR_FILE_RENAME: i32 = 15;
pub const GOBERR_PROFILE_OFF: i32 = 16;
pub const GOBERR_PROFILE_ON: i32 = 17;
pub const GOBERR_NO_EXTENDED: i32 = 18;
pub const GOBERR_DUP_HASH: i32 = 19;
pub const GOBERR_TOO_MANY_BLOCKS: i32 = 20;
pub const GOBERR_COMPRESS_FAIL: i32 = 21;
pub const GOBERR_NO_SUITABLE_CODEC: i32 = 22;

pub const GOBACCESS_READ: i32 = 0;
pub const GOBACCESS_WRITE: i32 = 1;
pub const GOBACCESS_RW: i32 = 2;

pub const GOBSEEK_START: i32 = 0;
pub const GOBSEEK_CURRENT: i32 = 1;
pub const GOBSEEK_END: i32 = 2;

// GOB_CODEC_MASK(n) ((GOBUInt32)(1u<<(n)))
#[inline]
pub const fn GOB_CODEC_MASK(n: u32) -> u32 {
    1u32 << n
}

pub const GOB_CODEC_MASK_ANY: u32 = u32::MAX;

// GOBMARKER_STARTBLOCK ('L' | 'B' << 8 | 'T' << 16 | 'S' << 24)
pub const GOBMARKER_STARTBLOCK: u32 = (('L' as u32) | (('B' as u32) << 8) | (('T' as u32) << 16) | (('S' as u32) << 24));

// GOBMARKER_ENDBLOCK ('L' | 'B' << 8 | 'N' << 16 | 'E' << 24)
pub const GOBMARKER_ENDBLOCK: u32 = (('L' as u32) | (('B' as u32) << 8) | (('N' as u32) << 16) | (('E' as u32) << 24));

pub type int32 = i32;
pub type uint32 = u32;
pub type ulong = c_ulong;
pub type byte = u8;

pub type GOBInt32 = i32;
pub type GOBUInt32 = u32;
pub type GOBChar = c_char;
pub type GOBBool = bool;
pub type GOBError = i32;
pub type GOBSeekType = i32;
pub type GOBHandle = i32;
pub type GOBAccessType = i32;
pub type GOBFSHandle = *mut c_void;
pub type GOBVoid = c_void;

pub type GOBFileSysOpenFunc = unsafe extern "C" fn(*mut GOBChar, GOBAccessType) -> GOBFSHandle;
pub type GOBFileSysCloseFunc = unsafe extern "C" fn(*mut GOBFSHandle) -> GOBBool;
pub type GOBFileSysReadFunc = unsafe extern "C" fn(GOBFSHandle, *mut GOBVoid, GOBInt32) -> GOBInt32;
pub type GOBFileSysWriteFunc = unsafe extern "C" fn(GOBFSHandle, *mut GOBVoid, GOBInt32) -> GOBInt32;
pub type GOBFileSysSeekFunc = unsafe extern "C" fn(GOBFSHandle, GOBInt32, GOBSeekType) -> GOBInt32;
pub type GOBFileSysRenameFunc = unsafe extern "C" fn(*mut GOBChar, *mut GOBChar) -> GOBInt32;

pub type GOBMemAllocFunc = unsafe extern "C" fn(GOBUInt32) -> *mut GOBVoid;
pub type GOBMemFreeFunc = unsafe extern "C" fn(*mut GOBVoid);

pub type GOBCompressFunc = unsafe extern "C" fn(*mut GOBVoid, GOBUInt32, *mut GOBVoid, *mut GOBUInt32) -> GOBInt32;
pub type GOBDecompressFunc = unsafe extern "C" fn(*mut GOBVoid, GOBUInt32, *mut GOBVoid, *mut GOBUInt32) -> GOBInt32;

pub type GOBCacheFileOpenFunc = unsafe extern "C" fn(GOBUInt32) -> GOBBool;
pub type GOBCacheFileCloseFunc = unsafe extern "C" fn() -> GOBBool;
pub type GOBCacheFileReadFunc = unsafe extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32;
pub type GOBCacheFileWriteFunc = unsafe extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32;
pub type GOBCacheFileSeekFunc = unsafe extern "C" fn(GOBInt32) -> GOBInt32;

#[repr(C)]
pub struct GOBBlockTableEntry {
    pub size: GOBUInt32,    // compressed size
    pub offset: GOBUInt32,
    pub next: GOBUInt32,
}

#[repr(C)]
pub struct GOBFileTableBasicEntry {
    pub hash: GOBUInt32,
    pub size: GOBUInt32,    // decompressed size
    pub block: GOBUInt32,
}

#[repr(C)]
pub struct GOBFileTableExtEntry {
    pub name: [GOBChar; GOB_MAX_FILE_NAME_LEN],
    pub crc: GOBUInt32,
    pub time: GOBUInt32,
}

#[repr(C)]
pub struct GOBMemoryFuncSet {
    pub alloc: GOBMemAllocFunc,
    pub free: GOBMemFreeFunc,
}

#[repr(C)]
pub struct GOBSingleCodecDesc {
    pub tag: GOBChar,
    pub max_ratio: GOBInt32,
    pub compress: GOBCompressFunc,
    pub decompress: GOBDecompressFunc,
}

#[repr(C)]
pub struct GOBCodecFuncSet {
    pub codecs: GOBInt32,
    pub codec: [GOBSingleCodecDesc; GOB_MAX_CODECS],
}

#[repr(C)]
pub struct GOBFileSysFuncSet {
    pub open: GOBFileSysOpenFunc,
    pub close: GOBFileSysCloseFunc,
    pub read: GOBFileSysReadFunc,
    pub write: GOBFileSysWriteFunc,
    pub seek: GOBFileSysSeekFunc,
}

#[repr(C)]
pub struct GOBCacheFileFuncSet {
    pub open: GOBCacheFileOpenFunc,
    pub close: GOBCacheFileCloseFunc,
    pub read: GOBCacheFileReadFunc,
    pub write: GOBCacheFileWriteFunc,
    pub seek: GOBCacheFileSeekFunc,
}

#[repr(C)]
pub struct GOBReadStats {
    pub bufferUsed: GOBUInt32,
    pub bytesRead: GOBUInt32,
    pub cacheBytesRead: GOBUInt32,
    pub cacheBytesWrite: GOBUInt32,
    pub totalSeeks: GOBUInt32,
    pub farSeeks: GOBUInt32,
    pub filesOpened: GOBUInt32,
}

extern "C" {
    pub fn GOBInit(mem: *mut GOBMemoryFuncSet,
        file: *mut GOBFileSysFuncSet,
        codec: *mut GOBCodecFuncSet,
        cache: *mut GOBCacheFileFuncSet) -> GOBError;
    pub fn GOBShutdown() -> GOBError;

    pub fn GOBArchiveCreate(file: *const GOBChar) -> GOBError;
    pub fn GOBArchiveOpen(file: *const GOBChar, atype: GOBAccessType,
        extended: GOBBool, safe: GOBBool) -> GOBError;
    pub fn GOBArchiveClose() -> GOBError;
    pub fn GOBArchiveCheckMarkers() -> GOBError;

    pub fn GOBOpen(file: *mut GOBChar, handle: *mut GOBHandle) -> GOBError;
    pub fn GOBOpenCode(code: GOBInt32, handle: *mut GOBHandle) -> GOBError;
    pub fn GOBClose(handle: GOBHandle) -> GOBError;

    pub fn GOBRead(buffer: *mut GOBVoid, size: GOBUInt32, handle: GOBHandle) -> GOBUInt32;
    pub fn GOBSeek(handle: GOBHandle, offset: GOBUInt32, seektype: GOBSeekType, pos: *mut GOBUInt32) -> GOBError;

    pub fn GOBWrite(buffer: *mut GOBVoid, size: GOBUInt32, mtime: GOBUInt32, file: *const GOBChar, codec_mask: GOBUInt32) -> GOBError;
    pub fn GOBDelete(file: *const GOBChar) -> GOBError;

    pub fn GOBRearrange(file: *const GOBChar, xlat: *const GOBUInt32, rename: GOBFileSysRenameFunc) -> GOBError;

    pub fn GOBVerify(file: *const GOBChar, status: *mut GOBBool) -> GOBError;

    pub fn GOBGetSize(file: *const GOBChar, decomp: *mut GOBUInt32, comp: *mut GOBUInt32, slack: *mut GOBUInt32) -> GOBError;
    pub fn GOBGetTime(file: *const GOBChar, time: *mut GOBUInt32) -> GOBError;
    pub fn GOBGetCRC(file: *const GOBChar, crc: *mut GOBUInt32) -> GOBError;

    pub fn GOBAccess(file: *const GOBChar, status: *mut GOBBool) -> GOBError;
    pub fn GOBGetFileCode(file: *const GOBChar) -> GOBInt32;

    pub fn GOBGetFileTables(basic: *mut *mut GOBFileTableBasicEntry, ext: *mut *mut GOBFileTableExtEntry) -> GOBError;
    pub fn GOBGetBlockTable(table: *mut *mut GOBBlockTableEntry, num: *mut GOBUInt32) -> GOBError;
    pub fn GOBGetSlack(x: GOBUInt32) -> GOBUInt32;

    pub fn GOBSetCacheSize(num: GOBUInt32) -> GOBError;
    pub fn GOBSetReadBufferSize(size: GOBUInt32) -> GOBError;

    pub fn GOBGetReadStats() -> GOBReadStats;


    pub fn GOBSetProfileFuncs(fset: *mut GOBProfileFuncSet);

    pub fn GOBStartProfile() -> GOBError;
    pub fn GOBStopProfile() -> GOBError;
}

pub type GOBProfileReadFunc = unsafe extern "C" fn(GOBUInt32);

#[repr(C)]
pub struct GOBProfileFuncSet {
    pub read: GOBProfileReadFunc,
}
