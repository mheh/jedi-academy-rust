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

use core::ffi::{c_char, c_int, c_void};
use core::mem;
use core::ptr;

// Constants from goblib.h
const GOB_MAGIC_IDENTIFIER: u32 = 0x8008;
const GOB_MAX_FILE_NAME_LEN: usize = 96;
const GOB_MAX_OPEN_FILES: usize = 16;
const GOB_MAX_CODECS: usize = 2;
const GOB_READ_RETRYS: usize = 3;
const GOB_MAX_FILES: usize = 16*1024;
const GOB_MAX_BLOCKS: usize = 32767;
const GOB_BLOCK_SIZE: usize = 64*1024;
const GOB_BLOCK_ALIGNMENT: usize = 2048;
const GOB_MEM_ALIGNMENT: usize = 64;
const GOB_COMPRESS_OVERHEAD: usize = 1024;

const GOB_INVALID_SIZE: u32 = 0xFFFFFFFF;
const GOB_INVALID_BLOCK: u32 = 0xFFFFFFFF;

const GOB_TRUE: c_int = 1;
const GOB_FALSE: c_int = 0;

const GOBERR_OK: c_int = 0;
const GOBERR_NOT_INIT: c_int = 1;
const GOBERR_FILE_NOT_FOUND: c_int = 2;
const GOBERR_FILE_READ: c_int = 3;
const GOBERR_FILE_WRITE: c_int = 4;
const GOBERR_NO_MEMORY: c_int = 5;
const GOBERR_ALREADY_INIT: c_int = 6;
const GOBERR_ALREADY_OPEN: c_int = 7;
const GOBERR_INVALID_ACCESS: c_int = 8;
const GOBERR_NOT_GOB_FILE: c_int = 9;
const GOBERR_NOT_OPEN: c_int = 10;
const GOBERR_CANNOT_CREATE: c_int = 11;
const GOBERR_TOO_MANY_OPEN: c_int = 12;
const GOBERR_INVALID_SEEK: c_int = 13;
const GOBERR_TOO_MANY_FILES: c_int = 14;
const GOBERR_FILE_RENAME: c_int = 15;
const GOBERR_PROFILE_OFF: c_int = 16;
const GOBERR_PROFILE_ON: c_int = 17;
const GOBERR_NO_EXTENDED: c_int = 18;
const GOBERR_DUP_HASH: c_int = 19;
const GOBERR_TOO_MANY_BLOCKS: c_int = 20;
const GOBERR_COMPRESS_FAIL: c_int = 21;
const GOBERR_NO_SUITABLE_CODEC: c_int = 22;

const GOBACCESS_READ: c_int = 0;
const GOBACCESS_WRITE: c_int = 1;
const GOBACCESS_RW: c_int = 2;

const GOBSEEK_START: c_int = 0;
const GOBSEEK_CURRENT: c_int = 1;
const GOBSEEK_END: c_int = 2;

fn GOB_CODEC_MASK(n: u32) -> u32 { 1u32 << n }

const GOBMARKER_STARTBLOCK: u32 = ('L' as u32) | (('B' as u32) << 8) | (('T' as u32) << 16) | (('S' as u32) << 24);
const GOBMARKER_ENDBLOCK: u32 = ('L' as u32) | (('B' as u32) << 8) | (('N' as u32) << 16) | (('E' as u32) << 24);

// Type aliases from goblib.h
type GOBInt32 = c_int;
type GOBUInt32 = u32;
type GOBChar = c_char;
type GOBBool = c_int;
type GOBError = c_int;
type GOBSeekType = c_int;
type GOBHandle = c_int;
type GOBAccessType = c_int;
type GOBFSHandle = *mut c_void;
type GOBVoid = c_void;

type GOBFileSysOpenFunc = extern "C" fn(*mut GOBChar, GOBAccessType) -> GOBFSHandle;
type GOBFileSysCloseFunc = extern "C" fn(*mut GOBFSHandle) -> GOBBool;
type GOBFileSysReadFunc = extern "C" fn(GOBFSHandle, *mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBFileSysWriteFunc = extern "C" fn(GOBFSHandle, *mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBFileSysSeekFunc = extern "C" fn(GOBFSHandle, GOBInt32, GOBSeekType) -> GOBInt32;
type GOBFileSysRenameFunc = extern "C" fn(*mut GOBChar, *mut GOBChar) -> GOBInt32;

type GOBMemAllocFunc = extern "C" fn(GOBUInt32) -> *mut GOBVoid;
type GOBMemFreeFunc = extern "C" fn(*mut GOBVoid) -> ();

type GOBCompressFunc = extern "C" fn(*mut GOBVoid, GOBUInt32, *mut GOBVoid, *mut GOBUInt32) -> GOBInt32;
type GOBDecompressFunc = extern "C" fn(*mut GOBVoid, GOBUInt32, *mut GOBVoid, *mut GOBUInt32) -> GOBInt32;

type GOBCacheFileOpenFunc = extern "C" fn(GOBUInt32) -> GOBBool;
type GOBCacheFileCloseFunc = extern "C" fn() -> GOBBool;
type GOBCacheFileReadFunc = extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBCacheFileWriteFunc = extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBCacheFileSeekFunc = extern "C" fn(GOBInt32) -> GOBInt32;

type GOBProfileReadFunc = extern "C" fn(GOBUInt32) -> ();

#[repr(C)]
pub struct GOBBlockTableEntry {
	pub next: GOBUInt32,
	pub offset: GOBUInt32,
	pub size: GOBUInt32,
}

#[repr(C)]
pub struct GOBFileTableBasicEntry {
	pub block: GOBUInt32,
	pub hash: GOBUInt32,
	pub size: GOBUInt32,
}

#[repr(C)]
pub struct GOBFileTableExtEntry {
	pub name: [GOBChar; GOB_MAX_FILE_NAME_LEN],
	pub crc: GOBUInt32,
	pub time: GOBUInt32,
}

#[repr(C)]
pub struct GOBMemoryFuncSet {
	pub alloc: Option<GOBMemAllocFunc>,
	pub free: Option<GOBMemFreeFunc>,
}

#[repr(C)]
pub struct GOBSingleCodecDesc {
	pub tag: GOBChar,
	pub max_ratio: GOBInt32,
	pub compress: Option<GOBCompressFunc>,
	pub decompress: Option<GOBDecompressFunc>,
}

#[repr(C)]
pub struct GOBCodecFuncSet {
	pub codecs: GOBInt32,
	pub codec: [GOBSingleCodecDesc; GOB_MAX_CODECS],
}

#[repr(C)]
pub struct GOBFileSysFuncSet {
	pub open: Option<GOBFileSysOpenFunc>,
	pub close: Option<GOBFileSysCloseFunc>,
	pub read: Option<GOBFileSysReadFunc>,
	pub write: Option<GOBFileSysWriteFunc>,
	pub seek: Option<GOBFileSysSeekFunc>,
}

#[repr(C)]
pub struct GOBCacheFileFuncSet {
	pub open: Option<GOBCacheFileOpenFunc>,
	pub close: Option<GOBCacheFileCloseFunc>,
	pub read: Option<GOBCacheFileReadFunc>,
	pub write: Option<GOBCacheFileWriteFunc>,
	pub seek: Option<GOBCacheFileSeekFunc>,
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

#[repr(C)]
pub struct GOBBlockCache {
	pub data: *mut GOBChar,
	pub block: GOBUInt32,
	pub time: GOBUInt32,
	pub size: GOBUInt32,
}

#[repr(C)]
pub struct GOBReadBuffer {
	pub data: *mut GOBChar,
	pub dataStart: *mut GOBChar,
	pub pos: GOBUInt32,
	pub size: GOBUInt32,
}

pub struct OpenFileInfo {
	pub valid: GOBBool,
	pub startBlock: GOBUInt32,
	pub block: GOBUInt32,
	pub offset: GOBUInt32,
	pub pos: GOBUInt32,
	pub size: GOBUInt32,
}

type GOBProfileFuncSet = GOBProfileReadFunc;

// Profiling data
static mut ProfileReadCallback: Option<GOBProfileReadFunc> = None;
static mut ProfileEnabled: GOBBool = GOB_FALSE;

// Indicates whether or not the library has been initialized
static mut LibraryInit: GOBBool = GOB_FALSE;

// Callbacks for handling low-level compression/decompression
static mut CodecFuncs: GOBCodecFuncSet = GOBCodecFuncSet {
	codecs: 0,
	codec: [
		GOBSingleCodecDesc { tag: 0, max_ratio: 0, compress: None, decompress: None },
		GOBSingleCodecDesc { tag: 0, max_ratio: 0, compress: None, decompress: None },
	],
};

// Callbacks for handling low-level memory alloc and free
static mut MemFuncs: GOBMemoryFuncSet = GOBMemoryFuncSet {
	alloc: None,
	free: None,
};

// Callbacks for handling low-level file access
static mut FSFuncs: GOBFileSysFuncSet = GOBFileSysFuncSet {
	open: None,
	close: None,
	read: None,
	write: None,
	seek: None,
};

// Callbacks for handling block caching (ie Xbox temp space)
static mut CacheFileFuncs: GOBCacheFileFuncSet = GOBCacheFileFuncSet {
	open: None,
	close: None,
	read: None,
	write: None,
	seek: None,
};
static mut CacheFileActive: GOBBool = GOB_FALSE;

// Name of the GFC file
static mut ControlFileName: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];

// Handle to the GOB archive
static mut ArchiveHandle: GOBFSHandle = 0xFFFFFFFF as *mut c_void;

// Size of the active GOB archive
static mut ArchiveSize: GOBUInt32 = 0;
static mut ArchiveNumBlocks: GOBUInt32 = 0;
static mut ArchiveNumFiles: GOBUInt32 = 0;

// Cached blocks
static mut CacheBlocks: *mut GOBBlockCache = ptr::null_mut();
static mut NumCacheBlocks: GOBUInt32 = 0;
static mut CacheBlockCounter: GOBUInt32 = 0;

// Read ahead buffer
static mut ReadBuffer: GOBReadBuffer = GOBReadBuffer {
	data: ptr::null_mut(),
	dataStart: ptr::null_mut(),
	pos: 0xFFFFFFFF,
	size: 0,
};

// Decompression buffer
static mut DecompBuffer: *mut GOBChar = ptr::null_mut();

// Stats gathering
static mut ReadStats: GOBReadStats = GOBReadStats {
	bufferUsed: 0,
	bytesRead: 0,
	cacheBytesRead: 0,
	cacheBytesWrite: 0,
	totalSeeks: 0,
	farSeeks: 0,
	filesOpened: 0,
};
static mut CurrentArchivePos: GOBUInt32 = 0;

// File tables (from the GFC)
static mut FileTableBasic: *mut GOBFileTableBasicEntry = ptr::null_mut();
static mut FileTableExt: *mut GOBFileTableExtEntry = ptr::null_mut();

// Block tables (from the GFC)
static mut BlockTable: *mut GOBBlockTableEntry = ptr::null_mut();
static mut BlockCRC: *mut GOBUInt32 = ptr::null_mut();
static mut CacheFileTable: *mut GOBUInt32 = ptr::null_mut();

// Do the tables need to be written?
static mut FileTableDirty: GOBBool = GOB_FALSE;

// Information about open files
static mut OpenFiles: [OpenFileInfo; GOB_MAX_OPEN_FILES] = [
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
	OpenFileInfo { valid: GOB_FALSE, startBlock: 0, block: 0, offset: 0, pos: 0, size: 0 },
];

// Converting text to lower case -- this isn't very
// clean.  A common buffer is used to store lower case
// text.  So its not thread safe... among other things. ;)
static mut LowerCaseBuffer: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];

extern "C" {
	fn strlen(s: *const c_char) -> usize;
	fn tolower(c: c_int) -> c_int;
	fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
	fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
	fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
	fn crc32(crc: u32, buf: *const c_char, len: usize) -> u32;
	fn adler32(adler: u32, buf: *const c_char, len: usize) -> u32;
	fn qsort(base: *mut c_void, nitems: usize, size: usize, compar: extern "C" fn(*const c_void, *const c_void) -> c_int) -> ();
	fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
	fn _snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
}

// Platform-specific snprintf handling: provide a Z_NULL constant for zlib
const Z_NULL: *const c_char = ptr::null();

unsafe fn LowerCase(name: *const GOBChar) -> *const GOBChar {
	let mut i: GOBInt32 = 0;
	while *name.offset(i as isize) != 0 {
		*LowerCaseBuffer.as_mut_ptr().offset(i as isize) = tolower(*name.offset(i as isize) as c_int) as GOBChar;
		i += 1;
	}
	*LowerCaseBuffer.as_mut_ptr().offset(i as isize) = 0;

	LowerCaseBuffer.as_ptr() as *const GOBChar
}

// Checks if a file handle is invalid
fn InvalidHandle(h: GOBFSHandle) -> bool {
	(h as usize) == 0xFFFFFFFF
}

// Endian conversion
#[cfg(any(target_endian = "little", all()))]
fn SwapBytes(x: GOBUInt32) -> GOBUInt32 {
	(x >> 24) |
	((x >> 8) & 0xFF00) |
	((x << 8) & 0xFF0000) |
	(x << 24)
}

#[cfg(target_endian = "big")]
fn SwapBytes(x: GOBUInt32) -> GOBUInt32 {
	x
}

// Given a file name, get its index in the FileTable
unsafe fn GetFileTableEntry(file: *const GOBChar) -> GOBInt32 {
	let mut entry: GOBUInt32;
	let mut hash: u32;

	// hash the file name
	hash = crc32(0, Z_NULL, 0);
	hash = crc32(hash, file, strlen(file));

	// linear search for matching a matching hash
	entry = 0;
	while entry < *core::ptr::addr_of!(ArchiveNumFiles) {
		if (*BlockTable.add(entry as usize)).next != GOB_INVALID_BLOCK &&
			(*FileTableBasic.add(entry as usize)).hash == hash
		{
			return entry as GOBInt32;
		}
		entry += 1;
	}

	return -1;
}

// Mark the contents of cache and read buffer invalid
unsafe fn InvalidateCache() {
	let mut i: GOBUInt32 = 0;
	while i < *core::ptr::addr_of!(NumCacheBlocks) {
		(*CacheBlocks.add(i as usize)).block = 0xFFFFFFFF;
		i += 1;
	}
	(*core::ptr::addr_of_mut!(ReadBuffer)).pos = 0xFFFFFFFF;
}

// Deallocate memory used by cache and read buffer
unsafe fn FreeCache() {
	let mut i: GOBUInt32 = 0;

	if !CacheBlocks.is_null() {
		while i < *core::ptr::addr_of!(NumCacheBlocks) {
			let data = (*CacheBlocks.add(i as usize)).data;
			if !data.is_null() {
				(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(data as *mut GOBVoid);
			}
			(*CacheBlocks.add(i as usize)).data = ptr::null_mut();
			i += 1;
		}

		(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(CacheBlocks as *mut GOBVoid);
		*core::ptr::addr_of_mut!(NumCacheBlocks) = 0;
		CacheBlocks = ptr::null_mut();
	}
}

// Write the file table to disk if the form of a GFC
unsafe fn CommitFileTable() -> GOBError {
	let mut num: GOBUInt32;
	let mut basic: GOBFileTableBasicEntry;
	let mut ext: GOBFileTableExtEntry;
	let mut block: GOBBlockTableEntry;

	// open the GFC
	let handle = (*core::ptr::addr_of_mut!(FSFuncs)).open.unwrap()(*core::ptr::addr_of_mut!(ControlFileName) as *mut GOBChar, GOBACCESS_WRITE);
	if InvalidHandle(handle) { return GOBERR_FILE_WRITE; }

	// write the magic identifier
	num = SwapBytes(GOB_MAGIC_IDENTIFIER);
	if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(handle, &num as *const _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_WRITE; }

	// write the size of the GOB
	num = SwapBytes(*core::ptr::addr_of!(ArchiveSize));
	if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(handle, &num as *const _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_WRITE; }

	// write number of blocks in archive
	num = SwapBytes(*core::ptr::addr_of!(ArchiveNumBlocks));
	if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(handle, &num as *const _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_WRITE; }

	// write number of file in archive
	num = SwapBytes(*core::ptr::addr_of!(ArchiveNumFiles));
	if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(handle, &num as *const _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_WRITE; }

	// write block table -- with endian conversion
	num = 0;
	while num < *core::ptr::addr_of!(ArchiveNumBlocks) {
		block.next = SwapBytes((*BlockTable.add(num as usize)).next);
		block.offset = SwapBytes((*BlockTable.add(num as usize)).offset);
		block.size = SwapBytes((*BlockTable.add(num as usize)).size);

		if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(handle, &block as *const _ as *mut GOBVoid, mem::size_of::<GOBBlockTableEntry>() as GOBInt32) == 0 { return GOBERR_FILE_WRITE; }
		num += 1;
	}

	// write block CRCs -- with endian conversion
	num = 0;
	while num < *core::ptr::addr_of!(ArchiveNumBlocks) {
		*BlockCRC.add(num as usize) = SwapBytes(*BlockCRC.add(num as usize));
		if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(handle, &*BlockCRC.add(num as usize) as *const _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 {
			return GOBERR_FILE_WRITE;
		}
		num += 1;
	}

	// write each basic table entry -- with endian conversion
	num = 0;
	while num < *core::ptr::addr_of!(ArchiveNumFiles) {
		basic.hash = SwapBytes((*FileTableBasic.add(num as usize)).hash);
		basic.block = SwapBytes((*FileTableBasic.add(num as usize)).block);
		basic.size = SwapBytes((*FileTableBasic.add(num as usize)).size);

		if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(handle, &basic as *const _ as *mut GOBVoid, mem::size_of::<GOBFileTableBasicEntry>() as GOBInt32) == 0 { return GOBERR_FILE_WRITE; }
		num += 1;
	}

	// write each extended table entry -- with endian conversion
	num = 0;
	while num < *core::ptr::addr_of!(ArchiveNumFiles) {
		strcpy(&mut ext.name as *mut [GOBChar; GOB_MAX_FILE_NAME_LEN] as *mut GOBChar, (*FileTableExt.add(num as usize)).name.as_ptr());
		ext.crc = SwapBytes((*FileTableExt.add(num as usize)).crc);
		ext.time = SwapBytes((*FileTableExt.add(num as usize)).time);

		if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(handle, &ext as *const _ as *mut GOBVoid, mem::size_of::<GOBFileTableExtEntry>() as GOBInt32) == 0 { return GOBERR_FILE_WRITE; }
		num += 1;
	}

	// all done
	(*core::ptr::addr_of_mut!(FSFuncs)).close.unwrap()(&mut handle.cast_mut());
	*core::ptr::addr_of_mut!(FileTableDirty) = GOB_FALSE;

	return GOBERR_OK;
}


unsafe fn DeallocTables() {
	if !BlockTable.is_null() {
		// free the block table
		(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(BlockTable as *mut GOBVoid);
		BlockTable = ptr::null_mut();
	}

	if !BlockCRC.is_null() {
		// free the block crc table
		(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(BlockCRC as *mut GOBVoid);
		BlockCRC = ptr::null_mut();
	}

	if !CacheFileTable.is_null()
	{
		// free the block cache table
		(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(CacheFileTable as *mut GOBVoid);
		CacheFileTable = ptr::null_mut();
	}

	if !FileTableBasic.is_null() {
		// free the basic file table
		(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(FileTableBasic as *mut GOBVoid);
		FileTableBasic = ptr::null_mut();
	}

	if !FileTableExt.is_null() {
		// free the extended file table
		(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(FileTableExt as *mut GOBVoid);
		FileTableExt = ptr::null_mut();
	}
}

unsafe fn AllocTables(num_blocks: GOBUInt32, num_files: GOBUInt32,
	extended: GOBBool, safe: GOBBool) -> GOBError {
	let mut num: GOBUInt32;

	// dump any old tables
	DeallocTables();

	// allocate the block table
	BlockTable = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((num_blocks as usize * mem::size_of::<GOBBlockTableEntry>()) as GOBUInt32) as *mut GOBBlockTableEntry;
	if BlockTable.is_null() { return GOBERR_NO_MEMORY; }

	if safe != GOB_FALSE {
		// allocate the block crc table for verifying data validity
		BlockCRC = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((num_blocks as usize * mem::size_of::<GOBUInt32>()) as GOBUInt32) as *mut GOBUInt32;
		if BlockCRC.is_null() { return GOBERR_NO_MEMORY; }
	}
	else {
		BlockCRC = ptr::null_mut();
	}

	if *core::ptr::addr_of!(CacheFileActive) != GOB_FALSE
	{
		// allocate the block cache bitfield
		CacheFileTable = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap(((num_blocks as usize / 32 + 1) * 4) as GOBUInt32) as *mut GOBUInt32;
		if CacheFileTable.is_null() { return GOBERR_NO_MEMORY; }
	}

	// allocate the basic file table
	FileTableBasic = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((num_files as usize * mem::size_of::<GOBFileTableBasicEntry>()) as GOBUInt32) as *mut GOBFileTableBasicEntry;
	if FileTableBasic.is_null() { return GOBERR_NO_MEMORY; }

	if extended != GOB_FALSE {
		// allocate the extended file table
		FileTableExt = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((num_files as usize * mem::size_of::<GOBFileTableExtEntry>()) as GOBUInt32) as *mut GOBFileTableExtEntry;
		if FileTableExt.is_null() { return GOBERR_NO_MEMORY; }
	}
	else {
		FileTableExt = ptr::null_mut();
	}

	// clear the tables
	num = 0;
	while num < num_files {
		(*FileTableBasic.add(num as usize)).block = GOB_INVALID_BLOCK;
		if !FileTableExt.is_null() { (*FileTableExt.add(num as usize)).name[0] = 0; }
		num += 1;
	}

	num = 0;
	while num < num_blocks {
		(*BlockTable.add(num as usize)).next = GOB_INVALID_BLOCK;
		(*BlockTable.add(num as usize)).size = GOB_INVALID_SIZE;
		num += 1;
	}

	return GOBERR_OK;
}


// GOBInit
// Public function.  Initialize the library.
#[no_mangle]
pub unsafe extern "C" fn GOBInit(mem: *mut GOBMemoryFuncSet,
	file: *mut GOBFileSysFuncSet,
	codec: *mut GOBCodecFuncSet,
	cache: *mut GOBCacheFileFuncSet) -> GOBError {
	let mut i: GOBInt32;
	let mut err: GOBError;

	if *core::ptr::addr_of!(LibraryInit) != GOB_FALSE { return GOBERR_ALREADY_INIT; }

	// setup the callbacks
	*core::ptr::addr_of_mut!(MemFuncs) = *mem;
	*core::ptr::addr_of_mut!(FSFuncs) = *file;
	*core::ptr::addr_of_mut!(CodecFuncs) = *codec;
	if !cache.is_null() {
		*core::ptr::addr_of_mut!(CacheFileFuncs) = *cache;
		*core::ptr::addr_of_mut!(CacheFileActive) = GOB_TRUE;
	} else {
		*core::ptr::addr_of_mut!(CacheFileActive) = GOB_FALSE;
	}

	// allocate decompression buffer
	DecompBuffer = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((GOB_BLOCK_SIZE + GOB_COMPRESS_OVERHEAD) as GOBUInt32) as *mut GOBChar;
	if DecompBuffer.is_null() { return GOBERR_NO_MEMORY; }

	// clear open table
	i = 0;
	while i < GOB_MAX_OPEN_FILES as GOBInt32 {
		OpenFiles[i as usize].valid = GOB_FALSE;
		i += 1;
	}

	*core::ptr::addr_of_mut!(LibraryInit) = GOB_TRUE;

	err = GOBSetCacheSize(1);
	if err != GOBERR_OK {
		*core::ptr::addr_of_mut!(LibraryInit) = GOB_FALSE;
		return err;
	}

	(*core::ptr::addr_of_mut!(ReadBuffer)).data = ptr::null_mut();
	err = GOBSetReadBufferSize(128*1024);
	if err != GOBERR_OK {
		*core::ptr::addr_of_mut!(LibraryInit) = GOB_FALSE;
		return err;
	}

	return GOBERR_OK;
}

// GOBShutdown
// Public function.  Close the library.
#[no_mangle]
pub unsafe extern "C" fn GOBShutdown() -> GOBError {
	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }

	// if we have an open archive, close it
	if !InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { GOBArchiveClose(); }

	FreeCache();

	// free read ahead buffer
	let rb_data = (*core::ptr::addr_of!(ReadBuffer)).data;
	if !rb_data.is_null() {
		(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(rb_data as *mut GOBVoid);
		(*core::ptr::addr_of_mut!(ReadBuffer)).data = ptr::null_mut();
	}

	// free decompression buffer
	(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(DecompBuffer as *mut GOBVoid);

	// free the file and block tables
	DeallocTables();

	*core::ptr::addr_of_mut!(LibraryInit) = GOB_FALSE;
	return GOBERR_OK;
}


// GOBArchiveCreate
// Public function.  Create an empty GFC and GOB.
#[no_mangle]
pub unsafe extern "C" fn GOBArchiveCreate(file: *const GOBChar) -> GOBError {
	let mut fname: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];
	let mut handle: GOBFSHandle;
	let mut error: GOBError;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if !InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_ALREADY_OPEN; }

	// Allocate the max space for tables
	error = AllocTables(GOB_MAX_BLOCKS as GOBUInt32, GOB_MAX_FILES as GOBUInt32, GOB_TRUE, GOB_TRUE);
	if GOBERR_OK != error {
		return error;
	}

	// create an empty GFC
	_snprintf(fname.as_mut_ptr(), GOB_MAX_FILE_NAME_LEN, "%s.gfc\0".as_ptr() as *const c_char, file);
	strcpy(ControlFileName.as_mut_ptr(), fname.as_ptr());

	*core::ptr::addr_of_mut!(ArchiveSize) = 0;
	*core::ptr::addr_of_mut!(ArchiveNumBlocks) = 0;
	*core::ptr::addr_of_mut!(ArchiveNumFiles) = 0;
	*core::ptr::addr_of_mut!(CacheFileActive) = GOB_FALSE;

	CommitFileTable();

	// create an empty GOB
	_snprintf(fname.as_mut_ptr(), GOB_MAX_FILE_NAME_LEN, "%s.gob\0".as_ptr() as *const c_char, file);
	handle = (*core::ptr::addr_of_mut!(FSFuncs)).open.unwrap()(fname.as_mut_ptr(), GOBACCESS_WRITE);
	if InvalidHandle(handle) { return GOBERR_CANNOT_CREATE; }

	(*core::ptr::addr_of_mut!(FSFuncs)).close.unwrap()(&mut handle.cast_mut());

	return GOBERR_OK;
}

// GOBArchiveOpen
// Public function.  Open a GOB file and cache file tables.
#[no_mangle]
pub unsafe extern "C" fn GOBArchiveOpen(file: *const GOBChar, atype: GOBAccessType,
	extended: GOBBool, safe: GOBBool) -> GOBError {
	let mut fname: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];
	let mut handle: GOBFSHandle;
	let mut magic: GOBUInt32;
	let mut i: GOBUInt32;
	let mut error: GOBError;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if !InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_ALREADY_OPEN; }

	// open the GFC
	_snprintf(ControlFileName.as_mut_ptr(), GOB_MAX_FILE_NAME_LEN, "%s.gfc\0".as_ptr() as *const c_char, file);
	handle = (*core::ptr::addr_of_mut!(FSFuncs)).open.unwrap()(ControlFileName.as_mut_ptr(), atype);
	if InvalidHandle(handle) { return GOBERR_FILE_NOT_FOUND; }

	// read and check the magic
	if (*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(handle, &mut magic as *mut _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_READ; }
	if SwapBytes(magic) != GOB_MAGIC_IDENTIFIER { return GOBERR_NOT_GOB_FILE; }

	// read the GOB archive size
	if (*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(handle, &mut *core::ptr::addr_of_mut!(ArchiveSize) as *mut _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_READ; }
	*core::ptr::addr_of_mut!(ArchiveSize) = SwapBytes(*core::ptr::addr_of!(ArchiveSize));

	// read the number of blocks
	if (*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(handle, &mut *core::ptr::addr_of_mut!(ArchiveNumBlocks) as *mut _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_READ; }
	*core::ptr::addr_of_mut!(ArchiveNumBlocks) = SwapBytes(*core::ptr::addr_of!(ArchiveNumBlocks));

	// read the number of files
	if (*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(handle, &mut *core::ptr::addr_of_mut!(ArchiveNumFiles) as *mut _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_READ; }
	*core::ptr::addr_of_mut!(ArchiveNumFiles) = SwapBytes(*core::ptr::addr_of!(ArchiveNumFiles));

	// Allocate the space for tables
	if atype == GOBACCESS_READ {
		error = AllocTables(*core::ptr::addr_of!(ArchiveNumBlocks), *core::ptr::addr_of!(ArchiveNumFiles), extended, safe);
	}
	else {
		error = AllocTables(GOB_MAX_BLOCKS as GOBUInt32, GOB_MAX_FILES as GOBUInt32, extended, safe);
	}
	if GOBERR_OK != error {
		return error;
	}

	// read the block table
	if *core::ptr::addr_of!(ArchiveNumBlocks) != 0 &&
		(*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(handle, BlockTable as *mut GOBVoid,
		(mem::size_of::<GOBBlockTableEntry>() as GOBUInt32 * *core::ptr::addr_of!(ArchiveNumBlocks)) as GOBInt32) == 0
	{
		return GOBERR_FILE_READ;
	}

	if !BlockCRC.is_null() {
		// read the block CRCs
		if *core::ptr::addr_of!(ArchiveNumBlocks) != 0 &&
			(*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(handle, BlockCRC as *mut GOBVoid,
			(mem::size_of::<GOBUInt32>() as GOBUInt32 * *core::ptr::addr_of!(ArchiveNumBlocks)) as GOBInt32) == 0
		{
			return GOBERR_FILE_READ;
		}
	}
	else {
		// skip block CRCs
		(*core::ptr::addr_of_mut!(FSFuncs)).seek.unwrap()(handle, (mem::size_of::<GOBUInt32>() as GOBUInt32 * *core::ptr::addr_of!(ArchiveNumBlocks)) as GOBInt32,
			GOBSEEK_CURRENT);
	}

	if *core::ptr::addr_of!(CacheFileActive) != GOB_FALSE
	{
		// clear the block cache table
		i = 0;
		while i < *core::ptr::addr_of!(ArchiveNumBlocks) / 32 {
			*CacheFileTable.add(i as usize) = 0;
			i += 1;
		}
	}

	// open the cache file
	if *core::ptr::addr_of!(CacheFileActive) != GOB_FALSE && !(*core::ptr::addr_of_mut!(CacheFileFuncs)).open.unwrap()(*core::ptr::addr_of!(ArchiveSize)) {
		*core::ptr::addr_of_mut!(CacheFileActive) = GOB_FALSE;
	}

	// endian convert the table
	i = 0;
	while i < *core::ptr::addr_of!(ArchiveNumBlocks) {
		(*BlockTable.add(i as usize)).next = SwapBytes((*BlockTable.add(i as usize)).next);
		(*BlockTable.add(i as usize)).offset = SwapBytes((*BlockTable.add(i as usize)).offset);
		(*BlockTable.add(i as usize)).size = SwapBytes((*BlockTable.add(i as usize)).size);

		if !BlockCRC.is_null() {
			*BlockCRC.add(i as usize) = SwapBytes(*BlockCRC.add(i as usize));
		}
		i += 1;
	}

	// read the basic file table
	if *core::ptr::addr_of!(ArchiveNumFiles) != 0 &&
		(*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(handle, FileTableBasic as *mut GOBVoid,
		(mem::size_of::<GOBFileTableBasicEntry>() as GOBUInt32 * *core::ptr::addr_of!(ArchiveNumFiles)) as GOBInt32) == 0
	{
		return GOBERR_FILE_READ;
	}

	// endian convert the table
	i = 0;
	while i < *core::ptr::addr_of!(ArchiveNumFiles) {
		(*FileTableBasic.add(i as usize)).hash = SwapBytes((*FileTableBasic.add(i as usize)).hash);
		(*FileTableBasic.add(i as usize)).block = SwapBytes((*FileTableBasic.add(i as usize)).block);
		(*FileTableBasic.add(i as usize)).size = SwapBytes((*FileTableBasic.add(i as usize)).size);
		i += 1;
	}

	// if we have memory for the extended file table
	if !FileTableExt.is_null() {
		// read the table
		if *core::ptr::addr_of!(ArchiveNumFiles) != 0 &&
			(*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(handle, FileTableExt as *mut GOBVoid,
			(mem::size_of::<GOBFileTableExtEntry>() as GOBUInt32 * *core::ptr::addr_of!(ArchiveNumFiles)) as GOBInt32) == 0
		{
			return GOBERR_FILE_READ;
		}

		// endian convert the table
		i = 0;
		while i < *core::ptr::addr_of!(ArchiveNumFiles) {
			(*FileTableExt.add(i as usize)).crc = SwapBytes((*FileTableExt.add(i as usize)).crc);
			(*FileTableExt.add(i as usize)).time = SwapBytes((*FileTableExt.add(i as usize)).time);
			i += 1;
		}
	}

	(*core::ptr::addr_of_mut!(FSFuncs)).close.unwrap()(&mut handle.cast_mut());

	// open the GOB
	_snprintf(fname.as_mut_ptr(), GOB_MAX_FILE_NAME_LEN, "%s.gob\0".as_ptr() as *const c_char, file);
	*core::ptr::addr_of_mut!(ArchiveHandle) = (*core::ptr::addr_of_mut!(FSFuncs)).open.unwrap()(fname.as_mut_ptr(), atype);
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_FILE_NOT_FOUND; }

	// initialize stats gathering
	*core::ptr::addr_of_mut!(CurrentArchivePos) = 0;
	(*core::ptr::addr_of_mut!(ReadStats)).bufferUsed = 0;
	(*core::ptr::addr_of_mut!(ReadStats)).bytesRead = 0;
	(*core::ptr::addr_of_mut!(ReadStats)).cacheBytesRead = 0;
	(*core::ptr::addr_of_mut!(ReadStats)).cacheBytesWrite = 0;
	(*core::ptr::addr_of_mut!(ReadStats)).totalSeeks = 0;
	(*core::ptr::addr_of_mut!(ReadStats)).farSeeks = 0;
	(*core::ptr::addr_of_mut!(ReadStats)).filesOpened = 0;

	return GOBERR_OK;
}

// GOBArchiveClose
// Public function.  Close an open GOB archive.
#[no_mangle]
pub unsafe extern "C" fn GOBArchiveClose() -> GOBError {
	let mut i: GOBInt32;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// close any open files
	i = 0;
	while i < GOB_MAX_OPEN_FILES as GOBInt32 {
		GOBClose(i);
		i += 1;
	}

	// close the GOB
	(*core::ptr::addr_of_mut!(FSFuncs)).close.unwrap()(&mut *core::ptr::addr_of_mut!(ArchiveHandle));
	*core::ptr::addr_of_mut!(ArchiveHandle) = 0xFFFFFFFF as *mut c_void;

	// commit the file table if we're updated it
	if *core::ptr::addr_of!(FileTableDirty) != GOB_FALSE {
		CommitFileTable();
	}

	// close the cache file
	if *core::ptr::addr_of!(CacheFileActive) != GOB_FALSE {
		(*core::ptr::addr_of_mut!(CacheFileFuncs)).close.unwrap();
		*core::ptr::addr_of_mut!(CacheFileActive) = GOB_FALSE;
	}

	return GOBERR_OK;
}

extern "C" fn SortBlockDescsCallback(elem1: *const c_void, elem2: *const c_void) -> c_int {
	return ((*(elem1 as *const GOBBlockTableEntry)).offset as c_int) -
		((*(elem2 as *const GOBBlockTableEntry)).offset as c_int);
}

// GOBArchiveCheckMarkers
// Public function.  Check start/end markers to check approximate validity of GOB file
#[no_mangle]
pub unsafe extern "C" fn GOBArchiveCheckMarkers() -> GOBError {
	let mut i: GOBUInt32;
	let mut valid_blocks: GOBUInt32;
	let mut blocks: *mut GOBBlockTableEntry;
	let mut block: GOBUInt32;
	let mut start_marker: GOBUInt32;
	let mut end_marker: GOBUInt32;
	let mut ok: GOBBool;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// count valid blocks
	valid_blocks = 0;
	i = 0;
	while i < *core::ptr::addr_of!(ArchiveNumBlocks) {
		if (*BlockTable.add(i as usize)).size != GOB_INVALID_SIZE &&
			(*BlockTable.add(i as usize)).next != GOB_INVALID_BLOCK
		{
			valid_blocks += 1;
		}
		i += 1;
	}

	// arcvive is empty
	if valid_blocks == 0
	{
		return GOBERR_OK;
	}

	// alloc mem for valid block list
	blocks = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((mem::size_of::<GOBBlockTableEntry>() as usize * valid_blocks as usize) as GOBUInt32) as *mut GOBBlockTableEntry;
	if blocks.is_null()
	{
		return GOBERR_NO_MEMORY;
	}

	// copy valid blocks descriptions
	block = 0;
	i = 0;
	while i < *core::ptr::addr_of!(ArchiveNumBlocks) {
		if (*BlockTable.add(i as usize)).size != GOB_INVALID_SIZE &&
			(*BlockTable.add(i as usize)).next != GOB_INVALID_BLOCK
		{
			*blocks.add(block as usize) = *BlockTable.add(i as usize);
			block += 1;
		}
		i += 1;
	}

	// and sort 'em
	qsort(blocks as *mut c_void, valid_blocks as usize, mem::size_of::<GOBBlockTableEntry>(), SortBlockDescsCallback);

	// suppress some warnings
	start_marker = 0;
	end_marker = 0;

	// now scan entire archive for start-of-block and end-of-block markers
	i = 0;
	while i < valid_blocks {
		ok = GOB_TRUE;
		if (*core::ptr::addr_of_mut!(FSFuncs)).seek.unwrap()(*core::ptr::addr_of!(ArchiveHandle), (*blocks.add(i as usize)).offset as GOBInt32, GOBSEEK_START) != 0 { ok = GOB_FALSE; }
		if ok != GOB_FALSE && (*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(*core::ptr::addr_of!(ArchiveHandle), &mut start_marker as *mut _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) != mem::size_of::<GOBUInt32>() as GOBInt32 { ok = GOB_FALSE; }
		if ok != GOB_FALSE && (*core::ptr::addr_of_mut!(FSFuncs)).seek.unwrap()(*core::ptr::addr_of!(ArchiveHandle), ((*blocks.add(i as usize)).offset as i64 + (*blocks.add(i as usize)).size as i64 - mem::size_of::<GOBUInt32>() as i64) as GOBInt32, GOBSEEK_START) != 0 { ok = GOB_FALSE; }
		if ok != GOB_FALSE && (*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(*core::ptr::addr_of!(ArchiveHandle), &mut end_marker as *mut _ as *mut GOBVoid, mem::size_of::<GOBUInt32>() as GOBInt32) != mem::size_of::<GOBUInt32>() as GOBInt32 { ok = GOB_FALSE; }
		if ok == GOB_FALSE ||
			SwapBytes(start_marker) != GOBMARKER_STARTBLOCK ||
			SwapBytes(end_marker) != GOBMARKER_ENDBLOCK
		{
			(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(blocks as *mut GOBVoid);

			return GOBERR_NOT_GOB_FILE;
		}
		i += 1;
	}

	(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(blocks as *mut GOBVoid);

	return GOBERR_OK;
}

// GOBArchiveCreate
// Public function.  Create an empty GFC and GOB.
#[no_mangle]
pub unsafe extern "C" fn GOBGetSlack(x: GOBUInt32) -> GOBUInt32 {
	let align = x % GOB_BLOCK_ALIGNMENT as GOBUInt32;
	if align != 0 { return GOB_BLOCK_ALIGNMENT as GOBUInt32 - align; }
	return 0;
}

// GOBOpen
// Public function.  Open a file inside a GOB.
#[no_mangle]
pub unsafe extern "C" fn GOBOpen(file: *mut GOBChar, handle: *mut GOBHandle) -> GOBError {
	let mut entry: GOBInt32;
	let lfile: *const GOBChar;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// find a free handle
	*handle = 0;
	while *handle < GOB_MAX_OPEN_FILES as GOBHandle {
		if OpenFiles[*handle as usize].valid == GOB_FALSE { break; }
		*handle += 1;
	}

	if *handle >= GOB_MAX_OPEN_FILES as GOBHandle { return GOBERR_TOO_MANY_OPEN; }

	// find the file in the table
	lfile = LowerCase(file);

	entry = GetFileTableEntry(lfile);

	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	// setup the open file
	OpenFiles[*handle as usize].startBlock = (*FileTableBasic.add(entry as usize)).block;
	OpenFiles[*handle as usize].block = (*FileTableBasic.add(entry as usize)).block;
	OpenFiles[*handle as usize].size = (*FileTableBasic.add(entry as usize)).size;
	OpenFiles[*handle as usize].offset = 0;
	OpenFiles[*handle as usize].pos = 0;

	OpenFiles[*handle as usize].valid = GOB_TRUE;

	*core::ptr::addr_of_mut!(ReadStats).filesOpened += 1;

	return GOBERR_OK;
}

// GOBOpenCode
// Public function.  Open file with a code inside a GOB.
#[no_mangle]
pub unsafe extern "C" fn GOBOpenCode(code: GOBInt32, handle: *mut GOBHandle) -> GOBError {
	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// find a free handle
	*handle = 0;
	while *handle < GOB_MAX_OPEN_FILES as GOBHandle {
		if OpenFiles[*handle as usize].valid == GOB_FALSE { break; }
		*handle += 1;
	}

	if *handle >= GOB_MAX_OPEN_FILES as GOBHandle { return GOBERR_TOO_MANY_OPEN; }

	// setup the open file
	OpenFiles[*handle as usize].startBlock = (*FileTableBasic.add(code as usize)).block;
	OpenFiles[*handle as usize].block = (*FileTableBasic.add(code as usize)).block;
	OpenFiles[*handle as usize].size = (*FileTableBasic.add(code as usize)).size;
	OpenFiles[*handle as usize].offset = 0;
	OpenFiles[*handle as usize].pos = 0;

	OpenFiles[*handle as usize].valid = GOB_TRUE;

	*core::ptr::addr_of_mut!(ReadStats).filesOpened += 1;

	return GOBERR_OK;
}

// GOBClose
// Public function.  Close a file.
#[no_mangle]
pub unsafe extern "C" fn GOBClose(handle: GOBHandle) -> GOBError {
	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if OpenFiles[handle as usize].valid == GOB_FALSE { return GOBERR_NOT_OPEN; }

	// close the file by simply invalidating the open
	// file table entry
	OpenFiles[handle as usize].valid = GOB_FALSE;

	return GOBERR_OK;
}

unsafe fn RawRead(buffer: *mut GOBVoid, size: GOBUInt32, pos: GOBUInt32) -> GOBUInt32 {
	let mut bytes: GOBUInt32;

	// Reads _must_ be aligned otherwise things get very slow
	if pos % GOB_BLOCK_ALIGNMENT as GOBUInt32 != 0 {
		return 0;
	}
	if (buffer as usize) % GOB_MEM_ALIGNMENT != 0 {
		return 0;
	}

	// seek
	if (*core::ptr::addr_of_mut!(FSFuncs)).seek.unwrap()(*core::ptr::addr_of!(ArchiveHandle), pos as GOBInt32, GOBSEEK_START) != 0 { return 0; }

	if *core::ptr::addr_of!(CurrentArchivePos) != pos { *core::ptr::addr_of_mut!(ReadStats).totalSeeks += 1; }
	if pos > *core::ptr::addr_of!(CurrentArchivePos) + GOB_BLOCK_ALIGNMENT as GOBUInt32 ||
		*core::ptr::addr_of!(CurrentArchivePos) > pos + GOB_BLOCK_ALIGNMENT as GOBUInt32
	{
		*core::ptr::addr_of_mut!(ReadStats).farSeeks += 1;
	}

	// read
	bytes = (*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(*core::ptr::addr_of!(ArchiveHandle), buffer, size as GOBInt32) as GOBUInt32;

	*core::ptr::addr_of_mut!(ReadStats).bytesRead += bytes;
	*core::ptr::addr_of_mut!(CurrentArchivePos) = pos + bytes;

	return bytes;
}

unsafe fn CacheRawRead(buffer: *mut GOBVoid, size: GOBUInt32, pos: GOBUInt32) -> GOBUInt32 {
	let mut bytes: GOBUInt32;

	// Reads _must_ be aligned otherwise things get very slow
	if pos % GOB_BLOCK_ALIGNMENT as GOBUInt32 != 0 {
		return 0;
	}
	if (buffer as usize) % GOB_MEM_ALIGNMENT != 0 {
		return 0;
	}

	// seek
	if (*core::ptr::addr_of_mut!(CacheFileFuncs)).seek.unwrap()(pos as GOBInt32) != 0 { return 0; }

	// read
	bytes = (*core::ptr::addr_of_mut!(CacheFileFuncs)).read.unwrap()(buffer, size as GOBInt32) as GOBUInt32;
	*core::ptr::addr_of_mut!(ReadStats).cacheBytesRead += bytes;

	return bytes;
}

unsafe fn CacheRawWrite(buffer: *mut GOBVoid, size: GOBUInt32, pos: GOBUInt32) -> GOBUInt32 {
	let mut bytes: GOBUInt32;

	// Writes _must_ be aligned otherwise things get very slow
	if pos % GOB_BLOCK_ALIGNMENT as GOBUInt32 != 0 {
		return 0;
	}
	if (buffer as usize) % GOB_MEM_ALIGNMENT != 0 {
		return 0;
	}

	// seek
	if (*core::ptr::addr_of_mut!(CacheFileFuncs)).seek.unwrap()(pos as GOBInt32) != 0 { return 0; }

	// write
	bytes = (*core::ptr::addr_of_mut!(CacheFileFuncs)).write.unwrap()(buffer, size as GOBInt32) as GOBUInt32;
	*core::ptr::addr_of_mut!(ReadStats).cacheBytesWrite += bytes;

	return bytes;
}

unsafe fn BlockReadLow(block: GOBUInt32) -> GOBInt32 {
	let mut pos: GOBUInt32 = 0;
	let mut bytes: GOBUInt32;
	let mut cache_read: GOBBool = GOB_FALSE;
	let mut cache_write: GOBBool = GOB_FALSE;
	let mut cache_fail: GOBBool = GOB_FALSE;

	loop {
		// is the block in the read ahead buffer?
		if (*core::ptr::addr_of!(ReadBuffer)).pos <= (*BlockTable.add(block as usize)).offset + pos &&
			(*core::ptr::addr_of!(ReadBuffer)).pos + (*core::ptr::addr_of!(ReadBuffer)).size > (*BlockTable.add(block as usize)).offset + pos
		{
			let mut buffer_offset: GOBUInt32;
			let mut buffer_size: GOBUInt32;

			// use data in the read buffer
			buffer_offset = (*BlockTable.add(block as usize)).offset + pos - (*core::ptr::addr_of!(ReadBuffer)).pos;
			buffer_size = (*core::ptr::addr_of!(ReadBuffer)).size - buffer_offset;

			// clamp size
			if buffer_size > (*BlockTable.add(block as usize)).size - pos {
				buffer_size = (*BlockTable.add(block as usize)).size - pos;
			}

			memcpy(&mut *DecompBuffer.add(pos as usize) as *mut GOBChar as *mut c_void, &*(*core::ptr::addr_of!(ReadBuffer)).dataStart.add(buffer_offset as usize) as *const GOBChar as *const c_void, buffer_size as usize);

			pos += buffer_size;
		}

		// got enough data
		if pos == (*BlockTable.add(block as usize)).size { break; }

		// refill read buffer
		(*core::ptr::addr_of_mut!(ReadBuffer)).pos = (*BlockTable.add(block as usize)).offset + pos;
		(*core::ptr::addr_of_mut!(ReadBuffer)).pos -= (*core::ptr::addr_of!(ReadBuffer)).pos % GOB_BLOCK_ALIGNMENT as GOBUInt32;

		// check if block is in the external cache system
		if *core::ptr::addr_of!(CacheFileActive) != GOB_FALSE &&
			(*CacheFileTable.add((block / 32) as usize)) & (1 << (block % 32)) != 0
		{
			if CacheRawRead((*core::ptr::addr_of!(ReadBuffer)).dataStart as *mut GOBVoid,
				(*core::ptr::addr_of!(ReadBuffer)).size, (*core::ptr::addr_of!(ReadBuffer)).pos) != 0
			{
				cache_read = GOB_TRUE;
				continue;
			}
		}

		// read block from archive
		bytes = RawRead((*core::ptr::addr_of!(ReadBuffer)).dataStart as *mut GOBVoid, (*core::ptr::addr_of!(ReadBuffer)).size, (*core::ptr::addr_of!(ReadBuffer)).pos);
		if bytes != (*core::ptr::addr_of!(ReadBuffer)).size &&
			bytes != *core::ptr::addr_of!(ArchiveSize) - (*core::ptr::addr_of!(ReadBuffer)).pos
		{
			return -1; // Main read fail error code
		}

		// write block to cache file
		if *core::ptr::addr_of!(CacheFileActive) != GOB_FALSE
		{
			if CacheRawWrite((*core::ptr::addr_of!(ReadBuffer)).dataStart as *mut GOBVoid, bytes,
				(*core::ptr::addr_of!(ReadBuffer)).pos) == bytes
			{
				cache_write = GOB_TRUE;
			}
			else
			{
				cache_fail = GOB_TRUE;
			}
		}
	}

	if cache_write != GOB_FALSE {
		if cache_fail == GOB_FALSE { return 2; }
		return 0;
	}

	if cache_read != GOB_FALSE { return 1; }
	return 0;
}

unsafe fn BlockReadWithCache(block: GOBUInt32) -> GOBBool {
	let mut i: GOBInt32 = 0;

	while i < GOB_READ_RETRYS as GOBInt32 {
		let mut result: GOBInt32;

		// read the data
		result = BlockReadLow(block);
		if result >= 0
		{
			if !BlockCRC.is_null() {
				// crc check
				let mut crc: GOBUInt32;

				crc = adler32(0, Z_NULL, 0);
				crc = adler32(crc, DecompBuffer, (*BlockTable.add(block as usize)).size as usize);

				if *BlockCRC.add(block as usize) != crc {
					// crc mismatch, we must have got bad data --
					// try invalidating the cache and retrying...
					if *core::ptr::addr_of!(CacheFileActive) != GOB_FALSE {
						*CacheFileTable.add((block / 32) as usize) &= !(1 << (block % 32));
					}
					(*core::ptr::addr_of_mut!(ReadBuffer)).pos = 0xFFFFFFFF;
					i += 1;
					continue;
				}
			}

			// if cache write occurred -- mark block as cached
			if result == 2 {
				*CacheFileTable.add((block / 32) as usize) |= 1 << (block % 32);
			}

			// read success, crc success (or no check performed)
			return GOB_TRUE;
		}
		i += 1;
	}

	// multiple read/crc failures
	return GOB_FALSE;
}

unsafe fn BlockRead(buffer: *mut GOBVoid, block: GOBUInt32) -> GOBUInt32 {
	let mut size: GOBUInt32;
	let mut codec_index: GOBInt32;
	let mut compressed_data: *mut GOBChar;

	// read block from cache or archive
	if BlockReadWithCache(block) == GOB_FALSE
	{
		return GOB_INVALID_SIZE;
	}

	// decompress
	codec_index = 0;
	size = 0; // Initialize to satisfy compiler
	compressed_data = DecompBuffer.add(mem::size_of::<GOBUInt32>()); // skip start-of-block marker
	while codec_index < (*core::ptr::addr_of!(CodecFuncs)).codecs {
		// Check if codec matches
		if *compressed_data == (*core::ptr::addr_of!(CodecFuncs)).codec[codec_index as usize].tag {
			size = GOB_BLOCK_SIZE as GOBUInt32;
			if (*core::ptr::addr_of!(CodecFuncs)).codec[codec_index as usize].decompress.unwrap()(
				compressed_data.add(1) as *mut GOBVoid,
				(*BlockTable.add(block as usize)).size - 1 - (mem::size_of::<GOBUInt32>() * 2) as GOBUInt32,
				buffer, &mut size) != 0 {
				return GOB_INVALID_SIZE;
			}
			break;
		}
		codec_index += 1;
	}

	// If no suitable codecs were found, we're screwed
	if codec_index == (*core::ptr::addr_of!(CodecFuncs)).codecs {
		return GOB_INVALID_SIZE;
	}

	if let Some(cb) = *core::ptr::addr_of!(ProfileReadCallback) {
		if *core::ptr::addr_of!(ProfileEnabled) != GOB_FALSE {
			// register current read command
			cb(block);
		}
	}

	return size;
}

unsafe fn FillCacheBlock(block: GOBUInt32, index: GOBUInt32) {
	(*CacheBlocks.add(index as usize)).time = *core::ptr::addr_of!(CacheBlockCounter);
	*core::ptr::addr_of_mut!(CacheBlockCounter) += 1;
	(*CacheBlocks.add(index as usize)).block = block;
	(*CacheBlocks.add(index as usize)).size = BlockRead((*CacheBlocks.add(index as usize)).data as *mut GOBVoid, block);
}

unsafe fn FindBestCacheBlock(block: GOBUInt32) -> GOBInt32 {
	let mut i: GOBInt32 = 0;
	let mut oldest_time: GOBUInt32 = 0xFFFFFFFF;
	let mut oldest_index: GOBInt32 = -1;

	while i < *core::ptr::addr_of!(NumCacheBlocks) as GOBInt32 {
		if (*CacheBlocks.add(i as usize)).block == block {
			// if block is in this read buffer, use it
			return i;
		}

		// find the buffer that hasn't been accessed
		// for the longest time
		if (*CacheBlocks.add(i as usize)).time < oldest_time {
			oldest_time = (*CacheBlocks.add(i as usize)).time;
			oldest_index = i;
		}
		i += 1;
	}

	// use the buffer that hasn't been accessed
	// in the longest time
	return oldest_index;
}

// GOBRead
// Public function.  Read from an open file using
// a funky read-ahead buffer system.
#[no_mangle]
pub unsafe extern "C" fn GOBRead(buffer: *mut GOBVoid, size: GOBUInt32, handle: GOBHandle) -> GOBUInt32 {
	let mut pos: GOBUInt32;
	let mut cache_id: GOBInt32;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return 0; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return 0; }
	if OpenFiles[handle as usize].valid == GOB_FALSE { return 0; }

	// make sure we're reading within the file
	let mut sz = size;
	if OpenFiles[handle as usize].pos + sz > OpenFiles[handle as usize].size {
		sz = OpenFiles[handle as usize].size - OpenFiles[handle as usize].pos;
		if sz == 0 { return 0; }
	}

	cache_id = FindBestCacheBlock(OpenFiles[handle as usize].block);
	if cache_id < 0 { return GOB_INVALID_SIZE; }

	pos = OpenFiles[handle as usize].pos;

	loop {
		// are looking for data inside the read buffer?
		if (*CacheBlocks.add(cache_id as usize)).block == OpenFiles[handle as usize].block {
			// move any relevant data from the read buffer to the target buffer
			let mut buffer_size: GOBUInt32;

			// calc size of data we want from current buffer
			buffer_size = (*CacheBlocks.add(cache_id as usize)).size - OpenFiles[handle as usize].offset;
			if buffer_size > sz { buffer_size = sz; }

			// move from read buffer into output buffer
			memcpy(&mut *(buffer as *mut u8).add((OpenFiles[handle as usize].pos - pos) as usize) as *mut u8 as *mut c_void,
				&*(*CacheBlocks.add(cache_id as usize)).data.add(OpenFiles[handle as usize].offset as usize) as *const GOBChar as *const c_void,
				buffer_size as usize);

			// update file position
			OpenFiles[handle as usize].pos += buffer_size;
			OpenFiles[handle as usize].offset += buffer_size;

			// if we've completed this block -- move to next
			if OpenFiles[handle as usize].offset == (*CacheBlocks.add(cache_id as usize)).size {
				OpenFiles[handle as usize].block = (*BlockTable.add(OpenFiles[handle as usize].block as usize)).next;
				OpenFiles[handle as usize].offset = 0;
			}

			(*CacheBlocks.add(cache_id as usize)).time = *core::ptr::addr_of!(CacheBlockCounter);
			*core::ptr::addr_of_mut!(CacheBlockCounter) += 1;

			*core::ptr::addr_of_mut!(ReadStats).bufferUsed += buffer_size;
			sz -= buffer_size;
			if sz == 0 { break; }
		}

		// refill the buffer
		FillCacheBlock(OpenFiles[handle as usize].block, cache_id as GOBUInt32);
		if (*CacheBlocks.add(cache_id as usize)).size == GOB_INVALID_SIZE {
			(*CacheBlocks.add(cache_id as usize)).block = GOB_INVALID_BLOCK;
			return GOB_INVALID_SIZE;
		}

		// reading off the end of the archive
		if (*CacheBlocks.add(cache_id as usize)).block != OpenFiles[handle as usize].block { break; }
	}

	return OpenFiles[handle as usize].pos - pos;
}

// GOBSeek
// Public function.  Seek to a position in an open file.
#[no_mangle]
pub unsafe extern "C" fn GOBSeek(handle: GOBHandle, offset: GOBUInt32, type_: GOBSeekType, pos: *mut GOBUInt32) -> GOBError {
	let mut blocks: GOBUInt32;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if OpenFiles[handle as usize].valid == GOB_FALSE { return GOBERR_NOT_OPEN; }

	// find a new position based on the seek type
	match type_ {
	GOBSEEK_START => {
		*pos = offset;
	},

	GOBSEEK_CURRENT => {
		*pos = OpenFiles[handle as usize].pos + offset;
	},

	GOBSEEK_END => {
		*pos = OpenFiles[handle as usize].size + offset;
	},

	_ => {
		return GOBERR_INVALID_SEEK;
	}
	}

	// check to make sure we're still in the file
	if *pos > OpenFiles[handle as usize].size {
		return GOBERR_INVALID_SEEK;
	}

	// update the file position
	OpenFiles[handle as usize].pos = *pos;

	// update block
	blocks = *pos / GOB_BLOCK_SIZE as GOBUInt32;
	OpenFiles[handle as usize].block = OpenFiles[handle as usize].startBlock;
	while blocks != 0 {
		OpenFiles[handle as usize].block = (*BlockTable.add(OpenFiles[handle as usize].block as usize)).next;
		blocks -= 1;
	}

	// update position inside block
	OpenFiles[handle as usize].offset = *pos % GOB_BLOCK_SIZE as GOBUInt32;

	return GOBERR_OK;
}


unsafe fn FindFreeBlock() -> GOBUInt32 {
	let mut i: GOBInt32 = 0;
	while i < GOB_MAX_BLOCKS as GOBInt32 {
		if (*BlockTable.add(i as usize)).next == GOB_INVALID_BLOCK { return i as GOBUInt32; }
		i += 1;
	}
	return GOB_MAX_BLOCKS as GOBUInt32;
}

// GOBWrite
// Public function.  Write an entire file.  The file should not be open!
#[no_mangle]
pub unsafe extern "C" fn GOBWrite(buffer: *mut GOBVoid, size: GOBUInt32, mtime: GOBUInt32, file: *const GOBChar, codec_mask: GOBUInt32) -> GOBError {
	let mut handle: GOBUInt32;
	let mut slack: GOBInt32;
	let lfile: *const GOBChar;
	let mut hash: GOBUInt32;
	let mut crc: GOBUInt32;
	let mut i: GOBInt32;
	let mut out: *mut GOBChar;
	let mut pos: GOBUInt32;
	let mut last_block: GOBUInt32;
	let mut codec_index: GOBInt32;
	let mut compression_ratio: GOBInt32;
	let mut out_data: *mut GOBChar;
	let mut compressed_size: GOBUInt32;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if FileTableExt.is_null() { return GOBERR_NO_EXTENDED; }
	if BlockCRC.is_null() { return GOBERR_NO_EXTENDED; }

	InvalidateCache();

	// delete the file if it exists
	GOBDelete(file);

	// find a free entry in the file table
	handle = 0;
	while handle < GOB_MAX_FILES as GOBUInt32 {
		if (*FileTableBasic.add(handle as usize)).block == GOB_INVALID_BLOCK { break; }
		handle += 1;
	}

	if handle >= GOB_MAX_FILES as GOBUInt32 { return GOBERR_TOO_MANY_FILES; }
	if handle >= *core::ptr::addr_of!(ArchiveNumFiles) { *core::ptr::addr_of_mut!(ArchiveNumFiles) = handle + 1; }

	// move to the end of the GOB
	if (*core::ptr::addr_of_mut!(FSFuncs)).seek.unwrap()(*core::ptr::addr_of!(ArchiveHandle), 0, GOBSEEK_END) != 0 {
		return GOBERR_FILE_WRITE;
	}

	// alloc compression buffer
	out = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((GOB_BLOCK_SIZE + GOB_COMPRESS_OVERHEAD) as GOBUInt32) as *mut GOBChar;

	last_block = GOB_MAX_BLOCKS as GOBUInt32 - 1;

	pos = 0;
	while pos < size {
		let mut block: GOBUInt32;
		let mut in_size: GOBUInt32;

		// get a free block
		block = FindFreeBlock();
		if block >= GOB_MAX_BLOCKS as GOBUInt32 { return GOBERR_TOO_MANY_BLOCKS; }
		if block >= *core::ptr::addr_of!(ArchiveNumBlocks) { *core::ptr::addr_of_mut!(ArchiveNumBlocks) = block + 1; }

		// if this is not the first block, mark next block for the last block
		// else assign the first block in file table
		if pos != 0 { (*BlockTable.add(last_block as usize)).next = block; }
		else { (*FileTableBasic.add(handle as usize)).block = block; }

		// invalidate the next block
		(*BlockTable.add(block as usize)).next = GOB_MAX_BLOCKS as GOBUInt32;

		// compute the decompressed block size
		in_size = size - pos;
		if in_size > GOB_BLOCK_SIZE as GOBUInt32 { in_size = GOB_BLOCK_SIZE as GOBUInt32; }

		// compress block

		codec_index = 0;
		while codec_index < (*core::ptr::addr_of!(CodecFuncs)).codecs {
			if (GOB_CODEC_MASK(codec_index as u32) & codec_mask) == 0 {
				// skip if this codec is not listed as one of the allowed ones
				codec_index += 1;
				continue;
			}
			(*BlockTable.add(block as usize)).size = (GOB_BLOCK_SIZE + GOB_COMPRESS_OVERHEAD) as GOBUInt32;
			out_data = out;
			*(out_data as *mut GOBUInt32) = SwapBytes(GOBMARKER_STARTBLOCK);
			out_data = out_data.add(mem::size_of::<GOBUInt32>());
			*out_data = (*core::ptr::addr_of!(CodecFuncs)).codec[codec_index as usize].tag;
			out_data = out_data.add(1);
			if (*core::ptr::addr_of!(CodecFuncs)).codec[codec_index as usize].compress.unwrap()(&mut *(buffer as *mut u8).add(pos as usize) as *mut u8 as *mut GOBVoid,
				in_size, out_data as *mut GOBVoid, &mut (*BlockTable.add(block as usize)).size) != 0 {
				return GOBERR_COMPRESS_FAIL;
			}
			out_data = out_data.add((*BlockTable.add(block as usize)).size as usize);
			*(out_data as *mut GOBUInt32) = SwapBytes(GOBMARKER_ENDBLOCK);
			out_data = out_data.add(mem::size_of::<GOBUInt32>());

			// Adjust for the prefixed start-of-block marker and codec tag and trailing end-of-block marker
			compressed_size = (*BlockTable.add(block as usize)).size;
			(*BlockTable.add(block as usize)).size += (1 + mem::size_of::<GOBUInt32>() * 2) as GOBUInt32;

			// Check compression result
			compression_ratio = (compressed_size as GOBInt32 * 100 / in_size as GOBInt32);
			if compression_ratio <= (*core::ptr::addr_of!(CodecFuncs)).codec[codec_index as usize].max_ratio {
				// Compressed result is under par.  Let's go with it
				break;
			}

			// Otherwise, try the next compressor
			codec_index += 1;
		}

		// If no suitable codecs were found, take our ball and go home
		if codec_index == (*core::ptr::addr_of!(CodecFuncs)).codecs { return GOBERR_NO_SUITABLE_CODEC; }

		// compute and store the CRC
		*BlockCRC.add(block as usize) = adler32(0, Z_NULL, 0);
		*BlockCRC.add(block as usize) = adler32(*BlockCRC.add(block as usize), out,
					(*BlockTable.add(block as usize)).size as usize);

		// write block
		if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(*core::ptr::addr_of!(ArchiveHandle), out as *mut GOBVoid, (*BlockTable.add(block as usize)).size as GOBInt32) !=
			(*BlockTable.add(block as usize)).size as GOBInt32 {
			return GOBERR_FILE_WRITE;
		}

		// compute the slack (to keep alignment)
		slack = GOBGetSlack((*BlockTable.add(block as usize)).size) as GOBInt32;

		// write the slack space
		memset(out as *mut c_void, 0, slack as usize);
		if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(*core::ptr::addr_of!(ArchiveHandle), out as *mut GOBVoid, slack as GOBInt32) != slack as GOBInt32 {
			return GOBERR_FILE_WRITE;
		}

		(*BlockTable.add(block as usize)).offset = *core::ptr::addr_of!(ArchiveSize);
		*core::ptr::addr_of_mut!(ArchiveSize) = *core::ptr::addr_of!(ArchiveSize) + (*BlockTable.add(block as usize)).size + slack as GOBUInt32;

		last_block = block;
		pos += GOB_BLOCK_SIZE as GOBUInt32;
	}

	(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(out as *mut GOBVoid);

	lfile = LowerCase(file);

	// calculate file name hash
	hash = crc32(0, Z_NULL, 0);
	hash = crc32(hash, lfile, strlen(lfile));

	// make sure hash is unique
	i = 0;
	while i < GOB_MAX_FILES as GOBInt32 {
		if i as GOBUInt32 != handle &&
			(*FileTableBasic.add(i as usize)).block != GOB_INVALID_BLOCK &&
			(*FileTableBasic.add(i as usize)).hash == hash {
			return GOBERR_DUP_HASH;
		}
		i += 1;
	}

	// update the file tables
	(*FileTableBasic.add(handle as usize)).hash = hash;
	(*FileTableBasic.add(handle as usize)).size = size;

	strcpy(&mut (*FileTableExt.add(handle as usize)).name as *mut [GOBChar; GOB_MAX_FILE_NAME_LEN] as *mut GOBChar, lfile);

	crc = crc32(0, Z_NULL, 0);
	crc = crc32(crc, buffer as *const c_char, size as usize);
	(*FileTableExt.add(handle as usize)).crc = crc;

	(*FileTableExt.add(handle as usize)).time = mtime;

	*core::ptr::addr_of_mut!(FileTableDirty) = GOB_TRUE;
	return GOBERR_OK;
}

// GOBDelete
// Public function.  Delete a file from a GOB.  The file should not be open!
#[no_mangle]
pub unsafe extern "C" fn GOBDelete(file: *const GOBChar) -> GOBError {
	let mut entry: GOBInt32;
	let lfile: *const GOBChar;
	let mut block: GOBUInt32;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if FileTableExt.is_null() { return GOBERR_NO_EXTENDED; }
	if BlockCRC.is_null() { return GOBERR_NO_EXTENDED; }

	// find the file in the table
	lfile = LowerCase(file);

	entry = GetFileTableEntry(lfile);
	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	// invalidate blocks
	block = (*FileTableBasic.add(entry as usize)).block;
	loop {
		let next: GOBUInt32;
		next = (*BlockTable.add(block as usize)).next;
		(*BlockTable.add(block as usize)).next = GOB_INVALID_BLOCK;
		block = next;
		if block == GOB_MAX_BLOCKS as GOBUInt32 { break; }
	}

	// invalidate the file
	(*FileTableBasic.add(entry as usize)).block = GOB_INVALID_BLOCK;

	*core::ptr::addr_of_mut!(FileTableDirty) = GOB_TRUE;

	return GOBERR_OK;
}

// GOBRearrange
// Public function.  Sorts the blocks in an archive.
#[no_mangle]
pub unsafe extern "C" fn GOBRearrange(file: *const GOBChar, xlat: *const GOBUInt32, _rename: GOBFileSysRenameFunc) -> GOBError {
	let mut err: GOBError;
	let mut buffer: *mut GOBVoid;
	let mut slack: GOBInt32;
	let mut slack_buf: *mut GOBVoid;
	let mut i: GOBUInt32;
	let mut size: GOBUInt32;
	let mut temp_handle: GOBFSHandle;
	let mut full_name: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if !InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_ALREADY_OPEN; }
	if FileTableExt.is_null() { return GOBERR_NO_EXTENDED; }
	if BlockCRC.is_null() { return GOBERR_NO_EXTENDED; }

	// start things up
	err = GOBArchiveOpen(file, GOBACCESS_READ, GOB_TRUE, GOB_TRUE);
	if err != GOBERR_OK { return err; }

	// create temporary file
	temp_handle = (*core::ptr::addr_of_mut!(FSFuncs)).open.unwrap()("~temp.tmp\0".as_ptr() as *mut c_char, GOBACCESS_WRITE);
	if InvalidHandle(temp_handle) { return GOBERR_FILE_WRITE; }

	size = 0;

	// create an empty buffer for slack
	slack_buf = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap(GOB_BLOCK_ALIGNMENT as GOBUInt32);
	if slack_buf.is_null() { return GOBERR_NO_MEMORY; }
	memset(slack_buf, 0, GOB_BLOCK_ALIGNMENT);

	// get memory for block
	buffer = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((GOB_BLOCK_SIZE + GOB_COMPRESS_OVERHEAD) as GOBUInt32);
	if buffer.is_null() { return GOBERR_NO_MEMORY; }

	// copy files in new order to end of archive
	i = 0;
	while i < *core::ptr::addr_of!(ArchiveNumBlocks) {
		if (*BlockTable.add(*xlat.add(i as usize) as usize)).next != GOB_INVALID_BLOCK {
			// seek to the block
			if (*core::ptr::addr_of_mut!(FSFuncs)).seek.unwrap()(*core::ptr::addr_of!(ArchiveHandle),
				(*BlockTable.add(*xlat.add(i as usize) as usize)).offset as GOBInt32, GOBSEEK_START) != 0 {
				return GOBERR_FILE_READ;
			}

			// read the block
			if (*core::ptr::addr_of_mut!(FSFuncs)).read.unwrap()(*core::ptr::addr_of!(ArchiveHandle), buffer, (*BlockTable.add(*xlat.add(i as usize) as usize)).size as GOBInt32) !=
				(*BlockTable.add(*xlat.add(i as usize) as usize)).size as GOBInt32 {
				return GOBERR_FILE_READ;
			}

			// write block
			if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(temp_handle, buffer, (*BlockTable.add(*xlat.add(i as usize) as usize)).size as GOBInt32) !=
				(*BlockTable.add(*xlat.add(i as usize) as usize)).size as GOBInt32 {
				return GOBERR_FILE_WRITE;
			}

			// write the slack
			slack = GOBGetSlack((*BlockTable.add(*xlat.add(i as usize) as usize)).size) as GOBInt32;
			if (*core::ptr::addr_of_mut!(FSFuncs)).write.unwrap()(temp_handle, slack_buf, slack) != slack {
				return GOBERR_FILE_WRITE;
			}

			// update block pos
			(*BlockTable.add(*xlat.add(i as usize) as usize)).offset = size;
			size = size + (*BlockTable.add(*xlat.add(i as usize) as usize)).size + slack as GOBUInt32;
		}
		i += 1;
	}

	(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(buffer);
	(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(slack_buf);

	// close the archive
	err = GOBArchiveClose();
	if err != GOBERR_OK { return err; }

	// close temp file
	(*core::ptr::addr_of_mut!(FSFuncs)).close.unwrap()(&mut temp_handle);

	// overrwrite archive with temp file
	_snprintf(full_name.as_mut_ptr(), GOB_MAX_FILE_NAME_LEN, "%s.gob\0".as_ptr() as *const c_char, file);
	if _rename("~temp.tmp\0".as_ptr() as *mut c_char, full_name.as_mut_ptr()) != 0 { return GOBERR_FILE_RENAME; }

	*core::ptr::addr_of_mut!(ArchiveSize) = size;

	CommitFileTable();

	return GOBERR_OK;
}


// GOBVerify
// Public function.  Verifies the integrity of a file.
#[no_mangle]
pub unsafe extern "C" fn GOBVerify(file: *const GOBChar, status: *mut GOBBool) -> GOBError {
	let mut handle: GOBHandle;
	let mut err: GOBError;
	let mut buffer: *mut GOBVoid;
	let mut size: GOBUInt32 = 0; // assign to avoid compiler warning
	let mut junk: GOBUInt32 = 0;
	let mut crc: GOBUInt32;
	let mut entry: GOBInt32;
	let lfile: *const GOBChar;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if FileTableExt.is_null() { return GOBERR_NO_EXTENDED; }
	if BlockCRC.is_null() { return GOBERR_NO_EXTENDED; }

	// get the file size
	err = GOBGetSize(file, &mut size, &mut junk, &mut junk);
	if err != GOBERR_OK { return err; }

	// open the file
	err = GOBOpen(file as *mut GOBChar, &mut handle);
	if err != GOBERR_OK { return err; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);

	// alloc space for the file
	buffer = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()(size);
	if buffer.is_null() { return GOBERR_NO_MEMORY; }

	// read it into the buffer
	crc = GOBRead(buffer, size, handle);
	if crc != size { return GOBERR_FILE_READ; }

	// calc the crc
	crc = crc32(0, Z_NULL, 0);
	crc = crc32(crc, buffer as *const c_char, size as usize);

	(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(buffer);

	// verify the crc matches
	if crc != (*FileTableExt.add(entry as usize)).crc { *status = GOB_FALSE; }
	else { *status = GOB_TRUE; }

	err = GOBClose(handle);
	if err != GOBERR_OK { return err; }

	return GOBERR_OK;
}

// GOBGetSize
// Public function.  Get a file compressed, decompressed, slack sizes.
#[no_mangle]
pub unsafe extern "C" fn GOBGetSize(file: *const GOBChar,
	decomp: *mut GOBUInt32, comp: *mut GOBUInt32, slack: *mut GOBUInt32) -> GOBError {
	let mut entry: GOBInt32;
	let lfile: *const GOBChar;
	let mut block: GOBUInt32;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// get file table entry
	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);
	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	// decompressed size from file table
	*decomp = (*FileTableBasic.add(entry as usize)).size;

	// compressed size is sum of block sizes
	*comp = 0;
	*slack = 0;
	block = (*FileTableBasic.add(entry as usize)).block;
	while block != GOB_MAX_BLOCKS as GOBUInt32 {
		*comp = *comp + (*BlockTable.add(block as usize)).size;
		*slack = *slack + GOBGetSlack((*BlockTable.add(block as usize)).size);
		block = (*BlockTable.add(block as usize)).next;
	}

	return GOBERR_OK;
}

// GOBGetTime
// Public function.  Get a file modification time.
#[no_mangle]
pub unsafe extern "C" fn GOBGetTime(file: *const GOBChar, time: *mut GOBUInt32) -> GOBError {
	let mut entry: GOBInt32;
	let lfile: *const GOBChar;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if FileTableExt.is_null() { return GOBERR_NO_EXTENDED; }
	if BlockCRC.is_null() { return GOBERR_NO_EXTENDED; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);
	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	*time = (*FileTableExt.add(entry as usize)).time;
	return GOBERR_OK;
}

// GOBGetCRC
// Public function.  Get a file CRC.
#[no_mangle]
pub unsafe extern "C" fn GOBGetCRC(file: *const GOBChar, crc: *mut GOBUInt32) -> GOBError {
	let mut entry: GOBInt32;
	let lfile: *const GOBChar;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if FileTableExt.is_null() { return GOBERR_NO_EXTENDED; }
	if BlockCRC.is_null() { return GOBERR_NO_EXTENDED; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);
	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	*crc = (*FileTableExt.add(entry as usize)).crc;
	return GOBERR_OK;
}

// GOBAccess
// Public function.  Determine if a file exists in the archive.
#[no_mangle]
pub unsafe extern "C" fn GOBAccess(file: *const GOBChar, status: *mut GOBBool) -> GOBError {
	let mut entry: GOBInt32;
	let lfile: *const GOBChar;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);
	if entry == -1 { *status = GOB_FALSE; }
	else { *status = GOB_TRUE; }

	return GOBERR_OK;
}

// GOBGetFileCode
// Public function.  Find the index into the file table of a file.
#[no_mangle]
pub unsafe extern "C" fn GOBGetFileCode(file: *const GOBChar) -> GOBInt32 {
	let mut entry: GOBInt32;
	let lfile: *const GOBChar;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return -1; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return -1; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);

	return entry;
}

// GOBGetFileTables
// Public function.  Return the active file tables.
#[no_mangle]
pub unsafe extern "C" fn GOBGetFileTables(basic: *mut *mut GOBFileTableBasicEntry,
	ext: *mut *mut GOBFileTableExtEntry) -> GOBError {
	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	*basic = FileTableBasic;
	*ext = FileTableExt;
	return GOBERR_OK;
}

// GOBGetBlockTable
// Public function.  Return the active block table.
#[no_mangle]
pub unsafe extern "C" fn GOBGetBlockTable(table: *mut *mut GOBBlockTableEntry, num: *mut GOBUInt32) -> GOBError {
	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*core::ptr::addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	*table = BlockTable;
	*num = *core::ptr::addr_of!(ArchiveNumBlocks);
	return GOBERR_OK;
}

// GOBSetCacheSize
// Public function.  Allocates buffers to cache blocks.
#[no_mangle]
pub unsafe extern "C" fn GOBSetCacheSize(num: GOBUInt32) -> GOBError {
	let mut i: GOBUInt32;

	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }

	// only continue if we actually need to resize
	if num == *core::ptr::addr_of!(NumCacheBlocks) { return GOBERR_OK; }

	// free old cache buffers
	FreeCache();

	*core::ptr::addr_of_mut!(NumCacheBlocks) = 0;

	CacheBlocks = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()(
			(mem::size_of::<GOBBlockCache>() as usize * num as usize) as GOBUInt32) as *mut GOBBlockCache;
	if CacheBlocks.is_null() { return GOBERR_NO_MEMORY; }

	// allocate cache blocks and initialize
	i = 0;
	while i < num {
		(*CacheBlocks.add(i as usize)).data = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()(GOB_BLOCK_SIZE as GOBUInt32) as *mut GOBChar;
		if (*CacheBlocks.add(i as usize)).data.is_null() { return GOBERR_NO_MEMORY; }

		(*CacheBlocks.add(i as usize)).size = 0;
		(*CacheBlocks.add(i as usize)).time = 0;
		(*CacheBlocks.add(i as usize)).block = 0xFFFFFFFF;

		*core::ptr::addr_of_mut!(NumCacheBlocks) += 1;
		i += 1;
	}

	return GOBERR_OK;
}

// GOBSetReadBufferSize
// Public function.  Allocate a read ahead buffer.
#[no_mangle]
pub unsafe extern "C" fn GOBSetReadBufferSize(size: GOBUInt32) -> GOBError {
	if *core::ptr::addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }

	// only continue if we actually need to resize
	if size == (*core::ptr::addr_of!(ReadBuffer)).size { return GOBERR_OK; }

	// remove old buffer
	let rb_data = (*core::ptr::addr_of!(ReadBuffer)).data;
	if !rb_data.is_null() {
		(*core::ptr::addr_of_mut!(MemFuncs)).free.unwrap()(rb_data as *mut GOBVoid);
	}

	// allocate new buffer
	(*core::ptr::addr_of_mut!(ReadBuffer)).data = (*core::ptr::addr_of_mut!(MemFuncs)).alloc.unwrap()((size as usize + GOB_MEM_ALIGNMENT) as GOBUInt32) as *mut GOBChar;
	if (*core::ptr::addr_of!(ReadBuffer)).data.is_null() { return GOB_INVALID_SIZE as GOBError; }

	// set aligned pointer
	(*core::ptr::addr_of_mut!(ReadBuffer)).dataStart =
		&mut *(*core::ptr::addr_of!(ReadBuffer)).data.add(GOB_MEM_ALIGNMENT -
		((*core::ptr::addr_of!(ReadBuffer)).data as usize % GOB_MEM_ALIGNMENT));

	(*core::ptr::addr_of_mut!(ReadBuffer)).pos = 0xFFFFFFFF;
	(*core::ptr::addr_of_mut!(ReadBuffer)).size = size;

	return GOBERR_OK;
}

// GOBGetReadStats
// Public function.  Get file read statistics (seeks, sizes).
#[no_mangle]
pub unsafe extern "C" fn GOBGetReadStats() -> GOBReadStats {
	return *core::ptr::addr_of!(ReadStats);
}


#[no_mangle]
pub unsafe extern "C" fn GOBSetProfileFuncs(fset: *mut GOBProfileFuncSet) {
	if !fset.is_null() {
		*core::ptr::addr_of_mut!(ProfileReadCallback) = Some(*fset);
	}
}

#[no_mangle]
pub unsafe extern "C" fn GOBStartProfile() -> GOBError {
	if *core::ptr::addr_of!(ProfileEnabled) != GOB_FALSE { return GOBERR_PROFILE_ON; }
	*core::ptr::addr_of_mut!(ProfileEnabled) = GOB_TRUE;
	return GOBERR_OK;
}

#[no_mangle]
pub unsafe extern "C" fn GOBStopProfile() -> GOBError {
	if *core::ptr::addr_of!(ProfileEnabled) == GOB_FALSE { return GOBERR_PROFILE_OFF; }
	*core::ptr::addr_of_mut!(ProfileEnabled) = GOB_FALSE;
	return GOBERR_OK;
}
