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

use core::ffi::{c_char, c_int, c_uint, c_void};
use std::ptr::{addr_of, addr_of_mut};

// Constants
const GOB_MAGIC_IDENTIFIER: u32 = 0x8008;
const GOB_MAX_FILE_NAME_LEN: usize = 96;
const GOB_MAX_OPEN_FILES: usize = 16;
const GOB_MAX_CODECS: usize = 2;
const GOB_INFINITE_RATIO: u32 = 1000;
const GOB_READ_RETRYS: usize = 3;

const GOB_MAX_FILES: usize = 16 * 1024;
const GOB_MAX_BLOCKS: u32 = 32767;

const GOB_BLOCK_SIZE: u32 = 64 * 1024;
const GOB_BLOCK_ALIGNMENT: u32 = 2048;
const GOB_MEM_ALIGNMENT: u32 = 64;
const GOB_COMPRESS_OVERHEAD: u32 = 1024;

const GOB_INVALID_SIZE: u32 = 0xFFFFFFFF;
const GOB_INVALID_BLOCK: u32 = 0xFFFFFFFF;

const GOB_TRUE: u32 = 1;
const GOB_FALSE: u32 = 0;

const GOBERR_OK: i32 = 0;
const GOBERR_NOT_INIT: i32 = 1;
const GOBERR_FILE_NOT_FOUND: i32 = 2;
const GOBERR_FILE_READ: i32 = 3;
const GOBERR_FILE_WRITE: i32 = 4;
const GOBERR_NO_MEMORY: i32 = 5;
const GOBERR_ALREADY_INIT: i32 = 6;
const GOBERR_ALREADY_OPEN: i32 = 7;
const GOBERR_INVALID_ACCESS: i32 = 8;
const GOBERR_NOT_GOB_FILE: i32 = 9;
const GOBERR_NOT_OPEN: i32 = 10;
const GOBERR_CANNOT_CREATE: i32 = 11;
const GOBERR_TOO_MANY_OPEN: i32 = 12;
const GOBERR_INVALID_SEEK: i32 = 13;
const GOBERR_TOO_MANY_FILES: i32 = 14;
const GOBERR_FILE_RENAME: i32 = 15;
const GOBERR_PROFILE_OFF: i32 = 16;
const GOBERR_PROFILE_ON: i32 = 17;
const GOBERR_NO_EXTENDED: i32 = 18;
const GOBERR_DUP_HASH: i32 = 19;
const GOBERR_TOO_MANY_BLOCKS: i32 = 20;
const GOBERR_COMPRESS_FAIL: i32 = 21;
const GOBERR_NO_SUITABLE_CODEC: i32 = 22;

const GOBACCESS_READ: i32 = 0;
const GOBACCESS_WRITE: i32 = 1;
const GOBACCESS_RW: i32 = 2;

const GOBSEEK_START: i32 = 0;
const GOBSEEK_CURRENT: i32 = 1;
const GOBSEEK_END: i32 = 2;

fn GOB_CODEC_MASK(n: u32) -> u32 {
	1u32 << n
}
const GOB_CODEC_MASK_ANY: u32 = u32::MAX;

const GOBMARKER_STARTBLOCK: u32 = 'L' as u32 | ('B' as u32) << 8 | ('T' as u32) << 16 | ('S' as u32) << 24;
const GOBMARKER_ENDBLOCK: u32 = 'L' as u32 | ('B' as u32) << 8 | ('N' as u32) << 16 | ('E' as u32) << 24;

// Type aliases
type GOBInt32 = i32;
type GOBUInt32 = u32;
type GOBChar = c_char;
type GOBBool = u32;
type GOBError = i32;
type GOBSeekType = i32;
type GOBHandle = i32;
type GOBAccessType = i32;
type GOBFSHandle = *mut c_void;
type GOBVoid = c_void;

// Function pointer types
type GOBFileSysOpenFunc = unsafe extern "C" fn(*mut GOBChar, GOBAccessType) -> GOBFSHandle;
type GOBFileSysCloseFunc = unsafe extern "C" fn(*mut GOBFSHandle) -> GOBBool;
type GOBFileSysReadFunc = unsafe extern "C" fn(GOBFSHandle, *mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBFileSysWriteFunc = unsafe extern "C" fn(GOBFSHandle, *mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBFileSysSeekFunc = unsafe extern "C" fn(GOBFSHandle, GOBInt32, GOBSeekType) -> GOBInt32;
type GOBFileSysRenameFunc = unsafe extern "C" fn(*mut GOBChar, *mut GOBChar) -> GOBInt32;

type GOBMemAllocFunc = unsafe extern "C" fn(GOBUInt32) -> *mut GOBVoid;
type GOBMemFreeFunc = unsafe extern "C" fn(*mut GOBVoid);

type GOBCompressFunc = unsafe extern "C" fn(*mut GOBVoid, GOBUInt32, *mut GOBVoid, *mut GOBUInt32) -> GOBInt32;
type GOBDecompressFunc = unsafe extern "C" fn(*mut GOBVoid, GOBUInt32, *mut GOBVoid, *mut GOBUInt32) -> GOBInt32;

type GOBCacheFileOpenFunc = unsafe extern "C" fn(GOBUInt32) -> GOBBool;
type GOBCacheFileCloseFunc = unsafe extern "C" fn() -> GOBBool;
type GOBCacheFileReadFunc = unsafe extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBCacheFileWriteFunc = unsafe extern "C" fn(*mut GOBVoid, GOBInt32) -> GOBInt32;
type GOBCacheFileSeekFunc = unsafe extern "C" fn(GOBInt32) -> GOBInt32;

// Structure definitions
#[repr(C)]
struct GOBBlockTableEntry {
	size: GOBUInt32,  // compressed size
	offset: GOBUInt32,
	next: GOBUInt32,
}

#[repr(C)]
struct GOBFileTableBasicEntry {
	hash: GOBUInt32,
	size: GOBUInt32,  // decompressed size
	block: GOBUInt32,
}

#[repr(C)]
struct GOBFileTableExtEntry {
	name: [GOBChar; GOB_MAX_FILE_NAME_LEN],
	crc: GOBUInt32,
	time: GOBUInt32,
}

#[repr(C)]
struct GOBMemoryFuncSet {
	alloc: Option<GOBMemAllocFunc>,
	free: Option<GOBMemFreeFunc>,
}

#[repr(C)]
struct GOBSingleCodecDesc {
	tag: GOBChar,
	max_ratio: GOBInt32,
	compress: Option<GOBCompressFunc>,
	decompress: Option<GOBDecompressFunc>,
}

#[repr(C)]
struct GOBCodecFuncSet {
	codecs: GOBInt32,
	codec: [GOBSingleCodecDesc; GOB_MAX_CODECS],
}

#[repr(C)]
struct GOBFileSysFuncSet {
	open: Option<GOBFileSysOpenFunc>,
	close: Option<GOBFileSysCloseFunc>,
	read: Option<GOBFileSysReadFunc>,
	write: Option<GOBFileSysWriteFunc>,
	seek: Option<GOBFileSysSeekFunc>,
}

#[repr(C)]
struct GOBCacheFileFuncSet {
	open: Option<GOBCacheFileOpenFunc>,
	close: Option<GOBCacheFileCloseFunc>,
	read: Option<GOBCacheFileReadFunc>,
	write: Option<GOBCacheFileWriteFunc>,
	seek: Option<GOBCacheFileSeekFunc>,
}

#[repr(C)]
struct GOBReadStats {
	bufferUsed: GOBUInt32,
	bytesRead: GOBUInt32,
	cacheBytesRead: GOBUInt32,
	cacheBytesWrite: GOBUInt32,
	totalSeeks: GOBUInt32,
	farSeeks: GOBUInt32,
	filesOpened: GOBUInt32,
}

type GOBProfileReadFunc = unsafe extern "C" fn(GOBUInt32);

#[repr(C)]
struct GOBProfileFuncSet {
	read: Option<GOBProfileReadFunc>,
}

// Profiling data
static mut ProfileReadCallback: Option<GOBProfileReadFunc> = None;
static mut ProfileEnabled: GOBBool = GOB_FALSE;

// Indicates whether or not the library has been initialized
static mut LibraryInit: GOBBool = GOB_FALSE;

// Callbacks for handling low-level compression/decompression
static mut CodecFuncs: GOBCodecFuncSet = GOBCodecFuncSet {
	codecs: 0,
	codec: [
		GOBSingleCodecDesc {
			tag: 0,
			max_ratio: 0,
			compress: None,
			decompress: None,
		};
		GOB_MAX_CODECS
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
#[repr(C)]
struct GOBBlockCache {
	data: *mut GOBChar,
	block: GOBUInt32,
	time: GOBUInt32,
	size: GOBUInt32,
}

static mut CacheBlocks: *mut GOBBlockCache = std::ptr::null_mut();
static mut NumCacheBlocks: GOBUInt32 = 0;
static mut CacheBlockCounter: GOBUInt32 = 0;

// Read ahead buffer
#[repr(C)]
struct GOBReadBuffer {
	data: *mut GOBChar,
	dataStart: *mut GOBChar,
	pos: GOBUInt32,
	size: GOBUInt32,
}

static mut ReadBuffer: GOBReadBuffer = GOBReadBuffer {
	data: std::ptr::null_mut(),
	dataStart: std::ptr::null_mut(),
	pos: 0,
	size: 0,
};

// Decompression buffer
static mut DecompBuffer: *mut GOBChar = std::ptr::null_mut();

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
static mut FileTableBasic: *mut GOBFileTableBasicEntry = std::ptr::null_mut();
static mut FileTableExt: *mut GOBFileTableExtEntry = std::ptr::null_mut();

// Block tables (from the GFC)
static mut BlockTable: *mut GOBBlockTableEntry = std::ptr::null_mut();
static mut BlockCRC: *mut GOBUInt32 = std::ptr::null_mut();
static mut CacheFileTable: *mut GOBUInt32 = std::ptr::null_mut();

// Do the tables need to be written?
static mut FileTableDirty: GOBBool = GOB_FALSE;

// Information about open files
#[repr(C)]
struct OpenFileInfo {
	valid: GOBBool,
	startBlock: GOBUInt32,
	block: GOBUInt32,
	offset: GOBUInt32,

	pos: GOBUInt32,
	size: GOBUInt32,
}

// Open file table -- indices in this array are passed
// back to the caller as pseudo file handles.
static mut OpenFiles: [OpenFileInfo; GOB_MAX_OPEN_FILES] = [
	OpenFileInfo {
		valid: GOB_FALSE,
		startBlock: 0,
		block: 0,
		offset: 0,
		pos: 0,
		size: 0,
	};
	GOB_MAX_OPEN_FILES
];

// Converting text to lower case -- this isn't very
// clean.  A common buffer is used to store lower case
// text.  So its not thread safe... among other things. ;)
static mut LowerCaseBuffer: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];

unsafe fn LowerCase(name: *const GOBChar) -> *mut GOBChar {
	let mut i = 0;
	loop {
		let ch = *name.add(i);
		if ch == 0 {
			break;
		}
		let buf = addr_of_mut!(LowerCaseBuffer);
		*(*buf).as_mut_ptr().add(i) = (ch as u8).to_ascii_lowercase() as GOBChar;
		i += 1;
	}
	let buf = addr_of_mut!(LowerCaseBuffer);
	*(*buf).as_mut_ptr().add(i) = 0;

	addr_of_mut!(LowerCaseBuffer) as *mut GOBChar
}

// Checks if a file handle is invalid
fn InvalidHandle(h: GOBFSHandle) -> bool {
	(h as u32) == 0xFFFFFFFF
}

// Endian conversion
#[cfg(target_endian = "little")]
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

// External C functions from zlib and libc
extern "C" {
	fn crc32(crc: u32, buf: *const u8, len: u32) -> u32;
	fn adler32(adler: u32, buf: *const u8, len: u32) -> u32;
	fn strlen(s: *const c_char) -> usize;
	fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
	fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
	fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
	fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
	fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: *const c_void);
	fn tolower(c: c_int) -> c_int;
}

// Given a file name, get its index in the FileTable
unsafe fn GetFileTableEntry(file: *const GOBChar) -> GOBInt32 {
	let mut entry: GOBUInt32;
	let mut hash: GOBUInt32;

	// hash the file name
	hash = crc32(0, std::ptr::null(), 0);
	hash = crc32(hash, file as *const u8, strlen(file) as u32);

	// linear search for matching a matching hash
	for entry in 0..(*addr_of!(ArchiveNumFiles)) {
		if (*(*addr_of!(FileTableBasic)).add(entry as usize)).block != GOB_INVALID_BLOCK &&
			(*(*addr_of!(FileTableBasic)).add(entry as usize)).hash == hash
		{
			return entry as GOBInt32;
		}
	}

	return -1;
}

// Mark the contents of cache and read buffer invalid
unsafe fn InvalidateCache() {
	let mut i: GOBUInt32;
	for i in 0..(*addr_of!(NumCacheBlocks)) {
		(*(*addr_of_mut!(CacheBlocks)).add(i as usize)).block = 0xFFFFFFFF;
	}
	(*addr_of_mut!(ReadBuffer)).pos = 0xFFFFFFFF;
}

// Deallocate memory used by cache and read buffer
unsafe fn FreeCache() {
	let mut i: GOBUInt32;

	if !(*addr_of!(CacheBlocks)).is_null() {
		for i in 0..(*addr_of!(NumCacheBlocks)) {
			if !(*(*addr_of_mut!(CacheBlocks)).add(i as usize)).data.is_null() {
				if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
					free_fn((*(*addr_of_mut!(CacheBlocks)).add(i as usize)).data as *mut GOBVoid);
				}
				(*(*addr_of_mut!(CacheBlocks)).add(i as usize)).data = std::ptr::null_mut();
			}
		}

		if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
			free_fn(*addr_of_mut!(CacheBlocks) as *mut GOBVoid);
		}
		*addr_of_mut!(NumCacheBlocks) = 0;
		*addr_of_mut!(CacheBlocks) = std::ptr::null_mut();
	}
}

// Write the file table to disk if the form of a GFC
unsafe fn CommitFileTable() -> GOBError {
	let mut num: GOBUInt32;
	let mut basic: GOBFileTableBasicEntry;
	let mut ext: GOBFileTableExtEntry;
	let mut block: GOBBlockTableEntry;

	// open the GFC
	let handle = if let Some(open_fn) = (*addr_of!(FSFuncs)).open {
		open_fn(addr_of_mut!(ControlFileName) as *mut GOBChar, GOBACCESS_WRITE)
	} else {
		return GOBERR_FILE_WRITE;
	};
	if InvalidHandle(handle) { return GOBERR_FILE_WRITE; }

	// write the magic identifier
	num = SwapBytes(GOB_MAGIC_IDENTIFIER);
	if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
		if write_fn(handle, &mut num as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 {
			return GOBERR_FILE_WRITE;
		}
	} else {
		return GOBERR_FILE_WRITE;
	}

	// write the size of the GOB
	num = SwapBytes(*addr_of!(ArchiveSize));
	if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
		if write_fn(handle, &mut num as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 {
			return GOBERR_FILE_WRITE;
		}
	} else {
		return GOBERR_FILE_WRITE;
	}

	// write number of blocks in archive
	num = SwapBytes(*addr_of!(ArchiveNumBlocks));
	if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
		if write_fn(handle, &mut num as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 {
			return GOBERR_FILE_WRITE;
		}
	} else {
		return GOBERR_FILE_WRITE;
	}

	// write number of file in archive
	num = SwapBytes(*addr_of!(ArchiveNumFiles));
	if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
		if write_fn(handle, &mut num as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 {
			return GOBERR_FILE_WRITE;
		}
	} else {
		return GOBERR_FILE_WRITE;
	}

	// write block table -- with endian conversion
	for num in 0..(*addr_of!(ArchiveNumBlocks)) {
		block.next = SwapBytes((*(*addr_of!(BlockTable)).add(num as usize)).next);
		block.offset = SwapBytes((*(*addr_of!(BlockTable)).add(num as usize)).offset);
		block.size = SwapBytes((*(*addr_of!(BlockTable)).add(num as usize)).size);

		if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
			if write_fn(handle, &mut block as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBBlockTableEntry>() as GOBInt32) == 0 {
				return GOBERR_FILE_WRITE;
			}
		} else {
			return GOBERR_FILE_WRITE;
		}
	}

	// write block CRCs -- with endian conversion
	for num in 0..(*addr_of!(ArchiveNumBlocks)) {
		*(*addr_of_mut!(BlockCRC)).add(num as usize) = SwapBytes(*(*addr_of!(BlockCRC)).add(num as usize));
		if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
			if write_fn(handle, (*addr_of_mut!(BlockCRC)).add(num as usize) as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 {
				return GOBERR_FILE_WRITE;
			}
		} else {
			return GOBERR_FILE_WRITE;
		}
	}

	// write each basic table entry -- with endian conversion
	for num in 0..(*addr_of!(ArchiveNumFiles)) {
		basic.hash = SwapBytes((*(*addr_of!(FileTableBasic)).add(num as usize)).hash);
		basic.block = SwapBytes((*(*addr_of!(FileTableBasic)).add(num as usize)).block);
		basic.size = SwapBytes((*(*addr_of!(FileTableBasic)).add(num as usize)).size);

		if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
			if write_fn(handle, &mut basic as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBFileTableBasicEntry>() as GOBInt32) == 0 {
				return GOBERR_FILE_WRITE;
			}
		} else {
			return GOBERR_FILE_WRITE;
		}
	}

	// write each extended table entry -- with endian conversion
	for num in 0..(*addr_of!(ArchiveNumFiles)) {
		strcpy(ext.name.as_mut_ptr(), (*(*addr_of!(FileTableExt)).add(num as usize)).name.as_ptr());
		ext.crc = SwapBytes((*(*addr_of!(FileTableExt)).add(num as usize)).crc);
		ext.time = SwapBytes((*(*addr_of!(FileTableExt)).add(num as usize)).time);

		if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
			if write_fn(handle, &mut ext as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBFileTableExtEntry>() as GOBInt32) == 0 {
				return GOBERR_FILE_WRITE;
			}
		} else {
			return GOBERR_FILE_WRITE;
		}
	}

	// all done
	let mut h = handle;
	if let Some(close_fn) = (*addr_of!(FSFuncs)).close {
		close_fn(&mut h);
	}
	*addr_of_mut!(FileTableDirty) = GOB_FALSE;

	return GOBERR_OK;
}


unsafe fn DeallocTables() {
	if !(*addr_of!(BlockTable)).is_null() {
		// free the block table
		if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
			free_fn(*addr_of_mut!(BlockTable) as *mut GOBVoid);
		}
		*addr_of_mut!(BlockTable) = std::ptr::null_mut();
	}

	if !(*addr_of!(BlockCRC)).is_null() {
		// free the block crc table
		if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
			free_fn(*addr_of_mut!(BlockCRC) as *mut GOBVoid);
		}
		*addr_of_mut!(BlockCRC) = std::ptr::null_mut();
	}

	if !(*addr_of!(CacheFileTable)).is_null()
	{
		// free the block cache table
		if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
			free_fn(*addr_of_mut!(CacheFileTable) as *mut GOBVoid);
		}
		*addr_of_mut!(CacheFileTable) = std::ptr::null_mut();
	}

	if !(*addr_of!(FileTableBasic)).is_null() {
		// free the basic file table
		if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
			free_fn(*addr_of_mut!(FileTableBasic) as *mut GOBVoid);
		}
		*addr_of_mut!(FileTableBasic) = std::ptr::null_mut();
	}

	if !(*addr_of!(FileTableExt)).is_null() {
		// free the extended file table
		if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
			free_fn(*addr_of_mut!(FileTableExt) as *mut GOBVoid);
		}
		*addr_of_mut!(FileTableExt) = std::ptr::null_mut();
	}
}

unsafe fn AllocTables(num_blocks: GOBUInt32, num_files: GOBUInt32,
	extended: GOBBool, safe: GOBBool) -> GOBError {
	let mut num: GOBUInt32;

	// dump any old tables
	DeallocTables();

	// allocate the block table
	if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		*addr_of_mut!(BlockTable) = alloc_fn((num_blocks as usize * std::mem::size_of::<GOBBlockTableEntry>()) as GOBUInt32) as *mut GOBBlockTableEntry;
	}
	if (*addr_of!(BlockTable)).is_null() { return GOBERR_NO_MEMORY; }

	if safe == GOB_TRUE {
		// allocate the block crc table for verifying data validity
		if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
			*addr_of_mut!(BlockCRC) = alloc_fn((num_blocks as usize * std::mem::size_of::<GOBUInt32>()) as GOBUInt32) as *mut GOBUInt32;
		}
		if (*addr_of!(BlockCRC)).is_null() { return GOBERR_NO_MEMORY; }
	}
	else {
		*addr_of_mut!(BlockCRC) = std::ptr::null_mut();
	}

	if *addr_of!(CacheFileActive) != GOB_FALSE
	{
		// allocate the block cache bitfield
		if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
			*addr_of_mut!(CacheFileTable) = alloc_fn(((num_blocks / 32 + 1) * 4) as GOBUInt32) as *mut GOBUInt32;
		}
		if (*addr_of!(CacheFileTable)).is_null() { return GOBERR_NO_MEMORY; }
	}

	// allocate the basic file table
	if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		*addr_of_mut!(FileTableBasic) = alloc_fn((num_files as usize * std::mem::size_of::<GOBFileTableBasicEntry>()) as GOBUInt32) as *mut GOBFileTableBasicEntry;
	}
	if (*addr_of!(FileTableBasic)).is_null() { return GOBERR_NO_MEMORY; }

	if extended != GOB_FALSE {
		// allocate the extended file table
		if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
			*addr_of_mut!(FileTableExt) = alloc_fn((num_files as usize * std::mem::size_of::<GOBFileTableExtEntry>()) as GOBUInt32) as *mut GOBFileTableExtEntry;
		}
		if (*addr_of!(FileTableExt)).is_null() { return GOBERR_NO_MEMORY; }
	}
	else {
		*addr_of_mut!(FileTableExt) = std::ptr::null_mut();
	}

	// clear the tables
	for num in 0..num_files {
		(*(*addr_of_mut!(FileTableBasic)).add(num as usize)).block = GOB_INVALID_BLOCK;
		if !(*addr_of!(FileTableExt)).is_null() {
			(*(*addr_of_mut!(FileTableExt)).add(num as usize)).name[0] = 0;
		}
	}

	for num in 0..num_blocks {
		(*(*addr_of_mut!(BlockTable)).add(num as usize)).next = GOB_INVALID_BLOCK;
		(*(*addr_of_mut!(BlockTable)).add(num as usize)).size = GOB_INVALID_SIZE;
	}

	return GOBERR_OK;
}


// GOBInit
// Public function.  Initialize the library.
pub unsafe extern "C" fn GOBInit(mem: *mut std::ffi::c_void,
	file: *mut std::ffi::c_void,
	codec: *mut std::ffi::c_void,
	cache: *mut std::ffi::c_void) -> GOBError
{
	let mem = mem as *mut GOBMemoryFuncSet;
	let file = file as *mut GOBFileSysFuncSet;
	let codec = codec as *mut GOBCodecFuncSet;
	let cache_ptr = cache as *mut GOBCacheFileFuncSet;

	let mut i: GOBInt32;
	let mut err: GOBError;

	if *addr_of!(LibraryInit) != GOB_FALSE { return GOBERR_ALREADY_INIT; }

	// setup the callbacks
	*addr_of_mut!(MemFuncs) = *mem;
	*addr_of_mut!(FSFuncs) = *file;
	*addr_of_mut!(CodecFuncs) = *codec;
	if !cache_ptr.is_null() {
		*addr_of_mut!(CacheFileFuncs) = *cache_ptr;
		*addr_of_mut!(CacheFileActive) = GOB_TRUE;
	} else {
		*addr_of_mut!(CacheFileActive) = GOB_FALSE;
	}

	// allocate decompression buffer
	if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		*addr_of_mut!(DecompBuffer) = alloc_fn(GOB_BLOCK_SIZE + GOB_COMPRESS_OVERHEAD) as *mut GOBChar;
	}
	if (*addr_of!(DecompBuffer)).is_null() { return GOBERR_NO_MEMORY; }

	// clear open table
	for i in 0..GOB_MAX_OPEN_FILES as GOBInt32 {
		(*addr_of_mut!(OpenFiles))[i as usize].valid = GOB_FALSE;
	}

	*addr_of_mut!(LibraryInit) = GOB_TRUE;

	err = GOBSetCacheSize(1);
	if err != GOBERR_OK {
		*addr_of_mut!(LibraryInit) = GOB_FALSE;
		return err;
	}

	(*addr_of_mut!(ReadBuffer)).data = std::ptr::null_mut();
	err = GOBSetReadBufferSize(128*1024);
	if err != GOBERR_OK {
		*addr_of_mut!(LibraryInit) = GOB_FALSE;
		return err;
	}

	return GOBERR_OK;
}

// GOBShutdown
// Public function.  Close the library.
pub unsafe extern "C" fn GOBShutdown() -> GOBError
{
	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }

	// if we have an open archive, close it
	if !InvalidHandle(*addr_of!(ArchiveHandle)) { GOBArchiveClose(); }

	FreeCache();

	// free read ahead buffer
	if !(*addr_of!(ReadBuffer)).data.is_null() {
		if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
			free_fn((*addr_of_mut!(ReadBuffer)).data as *mut GOBVoid);
		}
		(*addr_of_mut!(ReadBuffer)).data = std::ptr::null_mut();
	}

	// free decompression buffer
	if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
		free_fn(*addr_of_mut!(DecompBuffer) as *mut GOBVoid);
	}

	// free the file and block tables
	DeallocTables();

	*addr_of_mut!(LibraryInit) = GOB_FALSE;
	return GOBERR_OK;
}


// GOBArchiveCreate
// Public function.  Create an empty GFC and GOB.
pub unsafe extern "C" fn GOBArchiveCreate(file: *const GOBChar) -> GOBError
{
	let mut fname: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];
	let mut handle: GOBFSHandle;
	let mut error: GOBError;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if !InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_ALREADY_OPEN; }

	// Allocate the max space for tables
	error = AllocTables(GOB_MAX_BLOCKS, GOB_MAX_FILES as GOBUInt32, GOB_TRUE, GOB_TRUE);
	if GOBERR_OK != error {
		return error;
	}

	// create an empty GFC
	snprintf(addr_of_mut!(ControlFileName) as *mut GOBChar, GOB_MAX_FILE_NAME_LEN, "%s.gfc\0".as_ptr() as *const c_char, file);

	*addr_of_mut!(ArchiveSize) = 0;
	*addr_of_mut!(ArchiveNumBlocks) = 0;
	*addr_of_mut!(ArchiveNumFiles) = 0;
	*addr_of_mut!(CacheFileActive) = GOB_FALSE;

	CommitFileTable();

	// create an empty GOB
	snprintf(fname.as_mut_ptr(), GOB_MAX_FILE_NAME_LEN, "%s.gob\0".as_ptr() as *const c_char, file);
	handle = if let Some(open_fn) = (*addr_of!(FSFuncs)).open {
		open_fn(fname.as_mut_ptr(), GOBACCESS_WRITE)
	} else {
		return GOBERR_CANNOT_CREATE;
	};
	if InvalidHandle(handle) { return GOBERR_CANNOT_CREATE; }

	let mut h = handle;
	if let Some(close_fn) = (*addr_of!(FSFuncs)).close {
		close_fn(&mut h);
	}

	return GOBERR_OK;
}

// GOBArchiveOpen
// Public function.  Open a GOB file and cache file tables.
pub unsafe extern "C" fn GOBArchiveOpen(file: *const GOBChar, atype: GOBAccessType,
	extended: GOBBool, safe: GOBBool) -> GOBError
{
	let mut fname: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];
	let mut handle: GOBFSHandle;
	let mut magic: GOBUInt32;
	let mut i: GOBUInt32;
	let mut error: GOBError;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if !InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_ALREADY_OPEN; }

	// open the GFC
	snprintf(addr_of_mut!(ControlFileName) as *mut GOBChar, GOB_MAX_FILE_NAME_LEN, "%s.gfc\0".as_ptr() as *const c_char, file);
	handle = if let Some(open_fn) = (*addr_of!(FSFuncs)).open {
		open_fn(addr_of_mut!(ControlFileName) as *mut GOBChar, atype)
	} else {
		return GOBERR_FILE_NOT_FOUND;
	};
	if InvalidHandle(handle) { return GOBERR_FILE_NOT_FOUND; }

	// read and check the magic
	if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
		if read_fn(handle, &mut magic as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_READ; }
	} else {
		return GOBERR_FILE_READ;
	}
	if SwapBytes(magic) != GOB_MAGIC_IDENTIFIER { return GOBERR_NOT_GOB_FILE; }

	// read the GOB archive size
	if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
		if read_fn(handle, addr_of_mut!(ArchiveSize) as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_READ; }
	} else {
		return GOBERR_FILE_READ;
	}
	*addr_of_mut!(ArchiveSize) = SwapBytes(*addr_of!(ArchiveSize));

	// read the number of blocks
	if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
		if read_fn(handle, addr_of_mut!(ArchiveNumBlocks) as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_READ; }
	} else {
		return GOBERR_FILE_READ;
	}
	*addr_of_mut!(ArchiveNumBlocks) = SwapBytes(*addr_of!(ArchiveNumBlocks));

	// read the number of files
	if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
		if read_fn(handle, addr_of_mut!(ArchiveNumFiles) as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == 0 { return GOBERR_FILE_READ; }
	} else {
		return GOBERR_FILE_READ;
	}
	*addr_of_mut!(ArchiveNumFiles) = SwapBytes(*addr_of!(ArchiveNumFiles));

	// Allocate the space for tables
	if atype == GOBACCESS_READ {
		error = AllocTables(*addr_of!(ArchiveNumBlocks), *addr_of!(ArchiveNumFiles), extended, safe);
	}
	else {
		error = AllocTables(GOB_MAX_BLOCKS, GOB_MAX_FILES as GOBUInt32, extended, safe);
	}
	if GOBERR_OK != error {
		return error;
	}

	// read the block table
	if *addr_of!(ArchiveNumBlocks) != 0 {
		if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
			if read_fn(handle, *addr_of_mut!(BlockTable) as *mut GOBVoid,
				(std::mem::size_of::<GOBBlockTableEntry>() as GOBUInt32 * *addr_of!(ArchiveNumBlocks)) as GOBInt32) == 0
			{
				return GOBERR_FILE_READ;
			}
		} else {
			return GOBERR_FILE_READ;
		}
	}

	if !(*addr_of!(BlockCRC)).is_null() {
		// read the block CRCs
		if *addr_of!(ArchiveNumBlocks) != 0 {
			if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
				if read_fn(handle, *addr_of_mut!(BlockCRC) as *mut GOBVoid,
					(std::mem::size_of::<GOBUInt32>() as GOBUInt32 * *addr_of!(ArchiveNumBlocks)) as GOBInt32) == 0
				{
					return GOBERR_FILE_READ;
				}
			} else {
				return GOBERR_FILE_READ;
			}
		}
	}
	else {
		// skip block CRCs
		if let Some(seek_fn) = (*addr_of!(FSFuncs)).seek {
			seek_fn(handle, (std::mem::size_of::<GOBUInt32>() as GOBUInt32 * *addr_of!(ArchiveNumBlocks)) as GOBInt32,
				GOBSEEK_CURRENT);
		}
	}

	if *addr_of!(CacheFileActive) != GOB_FALSE
	{
		// clear the block cache table
		for i in 0..(*addr_of!(ArchiveNumBlocks) / 32) {
			*(*addr_of_mut!(CacheFileTable)).add(i as usize) = 0;
		}
	}

	// open the cache file
	if *addr_of!(CacheFileActive) != GOB_FALSE {
		if let Some(cache_open_fn) = (*addr_of!(CacheFileFuncs)).open {
			if cache_open_fn(*addr_of!(ArchiveSize)) == GOB_FALSE {
				*addr_of_mut!(CacheFileActive) = GOB_FALSE;
			}
		}
	}

	// endian convert the table
	for i in 0..(*addr_of!(ArchiveNumBlocks)) {
		(*(*addr_of_mut!(BlockTable)).add(i as usize)).next = SwapBytes((*(*addr_of!(BlockTable)).add(i as usize)).next);
		(*(*addr_of_mut!(BlockTable)).add(i as usize)).offset = SwapBytes((*(*addr_of!(BlockTable)).add(i as usize)).offset);
		(*(*addr_of_mut!(BlockTable)).add(i as usize)).size = SwapBytes((*(*addr_of!(BlockTable)).add(i as usize)).size);

		if !(*addr_of!(BlockCRC)).is_null() {
			*(*addr_of_mut!(BlockCRC)).add(i as usize) = SwapBytes(*(*addr_of!(BlockCRC)).add(i as usize));
		}
	}

	// read the basic file table
	if *addr_of!(ArchiveNumFiles) != 0 {
		if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
			if read_fn(handle, *addr_of_mut!(FileTableBasic) as *mut GOBVoid,
				(std::mem::size_of::<GOBFileTableBasicEntry>() as GOBUInt32 * *addr_of!(ArchiveNumFiles)) as GOBInt32) == 0
			{
				return GOBERR_FILE_READ;
			}
		} else {
			return GOBERR_FILE_READ;
		}
	}

	// endian convert the table
	for i in 0..(*addr_of!(ArchiveNumFiles)) {
		(*(*addr_of_mut!(FileTableBasic)).add(i as usize)).hash = SwapBytes((*(*addr_of!(FileTableBasic)).add(i as usize)).hash);
		(*(*addr_of_mut!(FileTableBasic)).add(i as usize)).block = SwapBytes((*(*addr_of!(FileTableBasic)).add(i as usize)).block);
		(*(*addr_of_mut!(FileTableBasic)).add(i as usize)).size = SwapBytes((*(*addr_of!(FileTableBasic)).add(i as usize)).size);
	}

	// if we have memory for the extended file table
	if !(*addr_of!(FileTableExt)).is_null() {
		// read the table
		if *addr_of!(ArchiveNumFiles) != 0 {
			if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
				if read_fn(handle, *addr_of_mut!(FileTableExt) as *mut GOBVoid,
					(std::mem::size_of::<GOBFileTableExtEntry>() as GOBUInt32 * *addr_of!(ArchiveNumFiles)) as GOBInt32) == 0
				{
					return GOBERR_FILE_READ;
				}
			} else {
				return GOBERR_FILE_READ;
			}
		}

		// endian convert the table
		for i in 0..(*addr_of!(ArchiveNumFiles)) {
			(*(*addr_of_mut!(FileTableExt)).add(i as usize)).crc = SwapBytes((*(*addr_of!(FileTableExt)).add(i as usize)).crc);
			(*(*addr_of_mut!(FileTableExt)).add(i as usize)).time = SwapBytes((*(*addr_of!(FileTableExt)).add(i as usize)).time);
		}
	}

	let mut h = handle;
	if let Some(close_fn) = (*addr_of!(FSFuncs)).close {
		close_fn(&mut h);
	}

	// open the GOB
	snprintf(fname.as_mut_ptr(), GOB_MAX_FILE_NAME_LEN, "%s.gob\0".as_ptr() as *const c_char, file);
	*addr_of_mut!(ArchiveHandle) = if let Some(open_fn) = (*addr_of!(FSFuncs)).open {
		open_fn(fname.as_mut_ptr(), atype)
	} else {
		return GOBERR_FILE_NOT_FOUND;
	};
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_FILE_NOT_FOUND; }

	// initialize stats gathering
	*addr_of_mut!(CurrentArchivePos) = 0;
	(*addr_of_mut!(ReadStats)).bufferUsed = 0;
	(*addr_of_mut!(ReadStats)).bytesRead = 0;
	(*addr_of_mut!(ReadStats)).cacheBytesRead = 0;
	(*addr_of_mut!(ReadStats)).cacheBytesWrite = 0;
	(*addr_of_mut!(ReadStats)).totalSeeks = 0;
	(*addr_of_mut!(ReadStats)).farSeeks = 0;
	(*addr_of_mut!(ReadStats)).filesOpened = 0;

	return GOBERR_OK;
}

// GOBArchiveClose
// Public function.  Close an open GOB archive.
pub unsafe extern "C" fn GOBArchiveClose() -> GOBError
{
	let mut i: GOBInt32;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// close any open files
	for i in 0..GOB_MAX_OPEN_FILES as GOBInt32 {
		GOBClose(i);
	}

	// close the GOB
	let mut h = *addr_of!(ArchiveHandle);
	if let Some(close_fn) = (*addr_of!(FSFuncs)).close {
		close_fn(&mut h);
	}
	*addr_of_mut!(ArchiveHandle) = 0xFFFFFFFF as *mut c_void;

	// commit the file table if we're updated it
	if *addr_of!(FileTableDirty) != GOB_FALSE {
		CommitFileTable();
	}

	// close the cache file
	if *addr_of!(CacheFileActive) != GOB_FALSE {
		if let Some(cache_close_fn) = (*addr_of!(CacheFileFuncs)).close {
			cache_close_fn();
		}
		*addr_of_mut!(CacheFileActive) = GOB_FALSE;
	}

	return GOBERR_OK;
}

extern "C" fn SortBlockDescsCallback(elem1: *const c_void, elem2: *const c_void) -> c_int {
	let e1 = elem1 as *const GOBBlockTableEntry;
	let e2 = elem2 as *const GOBBlockTableEntry;
	((*e1).offset as i32) - ((*e2).offset as i32)
}

// GOBArchiveCheckMarkers
// Public function.  Check start/end markers to check approximate validity of GOB file
pub unsafe extern "C" fn GOBArchiveCheckMarkers() -> GOBError
{
	let mut i: GOBUInt32;
	let mut valid_blocks: GOBUInt32;
	let mut blocks: *mut GOBBlockTableEntry;
	let mut block: GOBUInt32;
	let mut start_marker: GOBUInt32;
	let mut end_marker: GOBUInt32;
	let mut ok: GOBBool;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// count valid blocks
	valid_blocks = 0;
	for i in 0..(*addr_of!(ArchiveNumBlocks))
	{
		if (*(*addr_of!(BlockTable)).add(i as usize)).size != GOB_INVALID_SIZE &&
			(*(*addr_of!(BlockTable)).add(i as usize)).next != GOB_INVALID_BLOCK
		{
			valid_blocks += 1;
		}
	}

	// arcvive is empty
	if valid_blocks == 0
	{
		return GOBERR_OK;
	}

	// alloc mem for valid block list
	blocks = if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		alloc_fn(std::mem::size_of::<GOBBlockTableEntry>() as GOBUInt32 * valid_blocks) as *mut GOBBlockTableEntry
	} else {
		std::ptr::null_mut()
	};
	if blocks.is_null()
	{
		return GOBERR_NO_MEMORY;
	}

	// copy valid blocks descriptions
	block = 0;
	for i in 0..(*addr_of!(ArchiveNumBlocks))
	{
		if (*(*addr_of!(BlockTable)).add(i as usize)).size != GOB_INVALID_SIZE &&
			(*(*addr_of!(BlockTable)).add(i as usize)).next != GOB_INVALID_BLOCK
		{
			*blocks.add(block as usize) = *(*addr_of!(BlockTable)).add(i as usize);
			block += 1;
		}
	}

	// and sort 'em
	qsort(blocks as *mut c_void, valid_blocks as usize, std::mem::size_of::<GOBBlockTableEntry>(),
		SortBlockDescsCallback as *const c_void);

	// suppress some warnings
	start_marker = 0;
	end_marker = 0;

	// now scan entire archive for start-of-block and end-of-block markers
	for i in 0..valid_blocks
	{
		ok = GOB_TRUE;
		ok = if ok == GOB_FALSE { GOB_FALSE } else { if let Some(seek_fn) = (*addr_of!(FSFuncs)).seek { if seek_fn(*addr_of!(ArchiveHandle), (*blocks.add(i as usize)).offset as GOBInt32, GOBSEEK_START) != 0 { GOB_FALSE } else { GOB_TRUE } } else { GOB_FALSE } };
		ok = if ok == GOB_FALSE { GOB_FALSE } else { if let Some(read_fn) = (*addr_of!(FSFuncs)).read { if read_fn(*addr_of!(ArchiveHandle), &mut start_marker as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == std::mem::size_of::<GOBUInt32>() as GOBInt32 { GOB_TRUE } else { GOB_FALSE } } else { GOB_FALSE } };
		ok = if ok == GOB_FALSE { GOB_FALSE } else { if let Some(seek_fn) = (*addr_of!(FSFuncs)).seek { if seek_fn(*addr_of!(ArchiveHandle), ((*blocks.add(i as usize)).offset as GOBInt32 + (*blocks.add(i as usize)).size as GOBInt32 - std::mem::size_of::<GOBUInt32>() as GOBInt32), GOBSEEK_START) != 0 { GOB_FALSE } else { GOB_TRUE } } else { GOB_FALSE } };
		ok = if ok == GOB_FALSE { GOB_FALSE } else { if let Some(read_fn) = (*addr_of!(FSFuncs)).read { if read_fn(*addr_of!(ArchiveHandle), &mut end_marker as *mut _ as *mut GOBVoid, std::mem::size_of::<GOBUInt32>() as GOBInt32) == std::mem::size_of::<GOBUInt32>() as GOBInt32 { GOB_TRUE } else { GOB_FALSE } } else { GOB_FALSE } };
		if ok == GOB_FALSE ||
			SwapBytes(start_marker) != GOBMARKER_STARTBLOCK ||
			SwapBytes(end_marker) != GOBMARKER_ENDBLOCK
		{
			if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
				free_fn(blocks as *mut GOBVoid);
			}

			return GOBERR_NOT_GOB_FILE;
		}
	}

	if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
		free_fn(blocks as *mut GOBVoid);
	}

	return GOBERR_OK;
}

// GOBArchiveCreate
// Public function.  Create an empty GFC and GOB.
pub unsafe extern "C" fn GOBGetSlack(x: GOBUInt32) -> GOBUInt32
{
	let mut align = x % GOB_BLOCK_ALIGNMENT;
	if align != 0 { return GOB_BLOCK_ALIGNMENT - align; }
	return 0;
}

// GOBOpen
// Public function.  Open a file inside a GOB.
pub unsafe extern "C" fn GOBOpen(file: *mut GOBChar, handle: *mut GOBHandle) -> GOBError
{
	let mut entry: GOBInt32;
	let mut lfile: *mut GOBChar;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// find a free handle
	*handle = 0;
	while *handle < GOB_MAX_OPEN_FILES as GOBHandle {
		if (*addr_of_mut!(OpenFiles))[*handle as usize].valid == GOB_FALSE { break; }
		*handle += 1;
	}

	if *handle >= GOB_MAX_OPEN_FILES as GOBHandle { return GOBERR_TOO_MANY_OPEN; }

	// find the file in the table
	lfile = LowerCase(file);

	entry = GetFileTableEntry(lfile);

	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	// setup the open file
	(*addr_of_mut!(OpenFiles))[*handle as usize].startBlock = (*(*addr_of!(FileTableBasic)).add(entry as usize)).block;
	(*addr_of_mut!(OpenFiles))[*handle as usize].block = (*(*addr_of!(FileTableBasic)).add(entry as usize)).block;
	(*addr_of_mut!(OpenFiles))[*handle as usize].size = (*(*addr_of!(FileTableBasic)).add(entry as usize)).size;
	(*addr_of_mut!(OpenFiles))[*handle as usize].offset = 0;
	(*addr_of_mut!(OpenFiles))[*handle as usize].pos = 0;

	(*addr_of_mut!(OpenFiles))[*handle as usize].valid = GOB_TRUE;

	(*addr_of_mut!(ReadStats)).filesOpened += 1;

	return GOBERR_OK;
}

// GOBOpenCode
// Public function.  Open file with a code inside a GOB.
pub unsafe extern "C" fn GOBOpenCode(code: GOBInt32, handle: *mut GOBHandle) -> GOBError
{
	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// find a free handle
	*handle = 0;
	while *handle < GOB_MAX_OPEN_FILES as GOBHandle {
		if (*addr_of_mut!(OpenFiles))[*handle as usize].valid == GOB_FALSE { break; }
		*handle += 1;
	}

	if *handle >= GOB_MAX_OPEN_FILES as GOBHandle { return GOBERR_TOO_MANY_OPEN; }

	// setup the open file
	(*addr_of_mut!(OpenFiles))[*handle as usize].startBlock = (*(*addr_of!(FileTableBasic)).add(code as usize)).block;
	(*addr_of_mut!(OpenFiles))[*handle as usize].block = (*(*addr_of!(FileTableBasic)).add(code as usize)).block;
	(*addr_of_mut!(OpenFiles))[*handle as usize].size = (*(*addr_of!(FileTableBasic)).add(code as usize)).size;
	(*addr_of_mut!(OpenFiles))[*handle as usize].offset = 0;
	(*addr_of_mut!(OpenFiles))[*handle as usize].pos = 0;

	(*addr_of_mut!(OpenFiles))[*handle as usize].valid = GOB_TRUE;

	(*addr_of_mut!(ReadStats)).filesOpened += 1;

	return GOBERR_OK;
}

// GOBClose
// Public function.  Close a file.
pub unsafe extern "C" fn GOBClose(handle: GOBHandle) -> GOBError
{
	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if (*addr_of_mut!(OpenFiles))[handle as usize].valid == GOB_FALSE { return GOBERR_NOT_OPEN; }

	// close the file by simply invalidating the open
	// file table entry
	(*addr_of_mut!(OpenFiles))[handle as usize].valid = GOB_FALSE;

	return GOBERR_OK;
}

unsafe fn RawRead(buffer: *mut GOBVoid, size: GOBUInt32, pos: GOBUInt32) -> GOBUInt32
{
	let mut bytes: GOBUInt32;

	// Reads _must_ be aligned otherwise things get very slow
	if pos % GOB_BLOCK_ALIGNMENT != 0 {
		return 0;
	}
	if (buffer as u32) % GOB_MEM_ALIGNMENT != 0 {
		return 0;
	}

	// seek
	if let Some(seek_fn) = (*addr_of!(FSFuncs)).seek {
		if seek_fn(*addr_of!(ArchiveHandle), pos as GOBInt32, GOBSEEK_START) != 0 { return 0; }
	} else {
		return 0;
	}

	if *addr_of!(CurrentArchivePos) != pos { (*addr_of_mut!(ReadStats)).totalSeeks += 1; }
	if pos > *addr_of!(CurrentArchivePos) + GOB_BLOCK_ALIGNMENT ||
		*addr_of!(CurrentArchivePos) > pos + GOB_BLOCK_ALIGNMENT
	{
		(*addr_of_mut!(ReadStats)).farSeeks += 1;
	}

	// read
	bytes = if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
		read_fn(*addr_of!(ArchiveHandle), buffer, size as GOBInt32) as GOBUInt32
	} else {
		0
	};

	(*addr_of_mut!(ReadStats)).bytesRead += bytes;
	*addr_of_mut!(CurrentArchivePos) = pos + bytes;

	return bytes;
}

unsafe fn CacheRawRead(buffer: *mut GOBVoid, size: GOBUInt32, pos: GOBUInt32) -> GOBUInt32
{
	let mut bytes: GOBUInt32;

	// Reads _must_ be aligned otherwise things get very slow
	if pos % GOB_BLOCK_ALIGNMENT != 0 {
		return 0;
	}
	if (buffer as u32) % GOB_MEM_ALIGNMENT != 0 {
		return 0;
	}

	// seek
	if let Some(seek_fn) = (*addr_of!(CacheFileFuncs)).seek {
		if seek_fn(pos as GOBInt32) != 0 { return 0; }
	} else {
		return 0;
	}

	// read
	bytes = if let Some(read_fn) = (*addr_of!(CacheFileFuncs)).read {
		read_fn(buffer, size as GOBInt32) as GOBUInt32
	} else {
		0
	};
	(*addr_of_mut!(ReadStats)).cacheBytesRead += bytes;

	return bytes;
}

unsafe fn CacheRawWrite(buffer: *mut GOBVoid, size: GOBUInt32, pos: GOBUInt32) -> GOBUInt32
{
	let mut bytes: GOBUInt32;

	// Writes _must_ be aligned otherwise things get very slow
	if pos % GOB_BLOCK_ALIGNMENT != 0 {
		return 0;
	}
	if (buffer as u32) % GOB_MEM_ALIGNMENT != 0 {
		return 0;
	}

	// seek
	if let Some(seek_fn) = (*addr_of!(CacheFileFuncs)).seek {
		if seek_fn(pos as GOBInt32) != 0 { return 0; }
	} else {
		return 0;
	}

	// write
	bytes = if let Some(write_fn) = (*addr_of!(CacheFileFuncs)).write {
		write_fn(buffer, size as GOBInt32) as GOBUInt32
	} else {
		0
	};
	(*addr_of_mut!(ReadStats)).cacheBytesWrite += bytes;

	return bytes;
}

unsafe fn BlockReadLow(block: GOBUInt32) -> GOBInt32
{
	let mut pos: GOBUInt32;
	let mut bytes: GOBUInt32;
	let mut cache_read: GOBBool;
	let mut cache_write: GOBBool;
	let mut cache_fail: GOBBool;

	pos = 0;
	cache_read = GOB_FALSE;
	cache_write = GOB_FALSE;
	cache_fail = GOB_FALSE;

	loop {
		// is the block in the read ahead buffer?
		if (*addr_of!(ReadBuffer)).pos <= (*(*addr_of!(BlockTable)).add(block as usize)).offset + pos &&
			(*addr_of!(ReadBuffer)).pos + (*addr_of!(ReadBuffer)).size > (*(*addr_of!(BlockTable)).add(block as usize)).offset + pos
		{
			let mut buffer_offset: GOBUInt32;
			let mut buffer_size: GOBUInt32;

			// use data in the read buffer
			buffer_offset = (*(*addr_of!(BlockTable)).add(block as usize)).offset + pos - (*addr_of!(ReadBuffer)).pos;
			buffer_size = (*addr_of!(ReadBuffer)).size - buffer_offset;

			// clamp size
			if buffer_size > (*(*addr_of!(BlockTable)).add(block as usize)).size - pos {
				buffer_size = (*(*addr_of!(BlockTable)).add(block as usize)).size - pos;
			}

			memcpy(
				(*addr_of_mut!(DecompBuffer)).add(pos as usize) as *mut c_void,
				(*addr_of!(ReadBuffer)).dataStart.add(buffer_offset as usize) as *const c_void,
				buffer_size as usize
			);

			pos += buffer_size;
		}

		// got enough data
		if pos == (*(*addr_of!(BlockTable)).add(block as usize)).size { break; }

		// refill read buffer
		(*addr_of_mut!(ReadBuffer)).pos = (*(*addr_of!(BlockTable)).add(block as usize)).offset + pos;
		(*addr_of_mut!(ReadBuffer)).pos -= (*addr_of!(ReadBuffer)).pos % GOB_BLOCK_ALIGNMENT;

		// check if block is in the external cache system
		if *addr_of!(CacheFileActive) != GOB_FALSE &&
			(*(*addr_of!(CacheFileTable)).add((block / 32) as usize)) & (1 << (block % 32)) != 0
		{
			if CacheRawRead((*addr_of_mut!(ReadBuffer)).dataStart,
				(*addr_of!(ReadBuffer)).size, (*addr_of!(ReadBuffer)).pos) != 0
			{
				cache_read = GOB_TRUE;
				continue;
			}
		}

		// read block from archive
		bytes = RawRead((*addr_of_mut!(ReadBuffer)).dataStart, (*addr_of!(ReadBuffer)).size, (*addr_of!(ReadBuffer)).pos);
		if bytes != (*addr_of!(ReadBuffer)).size &&
			bytes != *addr_of!(ArchiveSize) - (*addr_of!(ReadBuffer)).pos
		{
			return -1; // Main read fail error code
		}

		// write block to cache file
		if *addr_of!(CacheFileActive) != GOB_FALSE
		{
			if CacheRawWrite((*addr_of_mut!(ReadBuffer)).dataStart, bytes,
				(*addr_of!(ReadBuffer)).pos) == bytes
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

unsafe fn BlockReadWithCache(block: GOBUInt32) -> GOBBool
{
	let mut i: GOBInt32;

	for i in 0..GOB_READ_RETRYS as GOBInt32 {
		let mut result: GOBInt32;

		// read the data
		result = BlockReadLow(block);
		if result >= 0
		{
			if !(*addr_of!(BlockCRC)).is_null() {
				// crc check
				let mut crc: GOBUInt32;

				crc = adler32(0, std::ptr::null(), 0);
				crc = adler32(crc, *addr_of_mut!(DecompBuffer) as *const u8,
					(*(*addr_of!(BlockTable)).add(block as usize)).size);

				if *(*addr_of!(BlockCRC)).add(block as usize) != crc {
					// crc mismatch, we must have got bad data --
					// try invalidating the cache and retrying...
					if *addr_of!(CacheFileActive) != GOB_FALSE {
						*(*addr_of_mut!(CacheFileTable)).add((block / 32) as usize) &= !(1 << (block % 32));
					}
					(*addr_of_mut!(ReadBuffer)).pos = 0xFFFFFFFF;
					continue;
				}
			}

			// if cache write occurred -- mark block as cached
			if result == 2 {
				*(*addr_of_mut!(CacheFileTable)).add((block / 32) as usize) |= (1 << (block % 32));
			}

			// read success, crc success (or no check performed)
			return GOB_TRUE;
		}
	}

	// multiple read/crc failures
	return GOB_FALSE;
}

unsafe fn BlockRead(buffer: *mut GOBVoid, block: GOBUInt32) -> GOBUInt32
{
	let mut size: GOBUInt32;
	let mut codec_index: GOBInt32;
	let mut compressed_data: *mut GOBChar;

	// read block from cache or archive
	if !BlockReadWithCache(block)
	{
		return GOB_INVALID_SIZE;
	}

	// decompress
	codec_index = 0;
	size = 0; // Initialize to satisfy compiler
	compressed_data = (*addr_of_mut!(DecompBuffer)).add(std::mem::size_of::<GOBUInt32>()); // skip start-of-block marker
	while codec_index < (*addr_of!(CodecFuncs)).codecs {
		// Check if codec matches
		if *compressed_data == (*addr_of!(CodecFuncs)).codec[codec_index as usize].tag {
			size = GOB_BLOCK_SIZE;
			if let Some(decomp_fn) = (*addr_of!(CodecFuncs)).codec[codec_index as usize].decompress {
				if decomp_fn(compressed_data.add(1) as *mut GOBVoid,
					(*(*addr_of!(BlockTable)).add(block as usize)).size - 1 - (std::mem::size_of::<GOBUInt32>() * 2) as GOBUInt32,
					buffer, &mut size) != 0 {
					return GOB_INVALID_SIZE;
				}
			} else {
				return GOB_INVALID_SIZE;
			}
			break;
		}
		codec_index += 1;
	}

	// If no suitable codecs were found, we're screwed
	if codec_index == (*addr_of!(CodecFuncs)).codecs {
		return GOB_INVALID_SIZE;
	}

	if (*addr_of!(ProfileReadCallback)).is_some() && *addr_of!(ProfileEnabled) != GOB_FALSE {
		// register current read command
		if let Some(callback_fn) = *addr_of!(ProfileReadCallback) {
			callback_fn(block);
		}
	}

	return size;
}

unsafe fn FillCacheBlock(block: GOBUInt32, index: GOBUInt32) {
	(*(*addr_of_mut!(CacheBlocks)).add(index as usize)).time = *addr_of!(CacheBlockCounter);
	*addr_of_mut!(CacheBlockCounter) += 1;
	(*(*addr_of_mut!(CacheBlocks)).add(index as usize)).block = block;
	(*(*addr_of_mut!(CacheBlocks)).add(index as usize)).size = BlockRead((*(*addr_of_mut!(CacheBlocks)).add(index as usize)).data as *mut GOBVoid, block);
}

unsafe fn FindBestCacheBlock(block: GOBUInt32) -> GOBInt32 {
	let mut i: GOBInt32;
	let mut oldest_time: GOBUInt32;
	let mut oldest_index: GOBInt32;

	oldest_time = 0xFFFFFFFF;
	oldest_index = -1;

	for i in 0..(*addr_of!(NumCacheBlocks)) as GOBInt32 {
		if (*(*addr_of_mut!(CacheBlocks)).add(i as usize)).block == block {
			// if block is in this read buffer, use it
			return i;
		}

		// find the buffer that hasn't been accessed
		// for the longest time
		if (*(*addr_of_mut!(CacheBlocks)).add(i as usize)).time < oldest_time {
			oldest_time = (*(*addr_of_mut!(CacheBlocks)).add(i as usize)).time;
			oldest_index = i;
		}
	}

	// use the buffer that hasn't been accessed
	// in the longest time
	return oldest_index;
}

// GOBRead
// Public function.  Read from an open file using
// a funky read-ahead buffer system.
pub unsafe extern "C" fn GOBRead(buffer: *mut GOBVoid, size: GOBUInt32, handle: GOBHandle) -> GOBUInt32
{
	let mut pos: GOBUInt32;
	let mut cache_id: GOBInt32;

	if *addr_of!(LibraryInit) == GOB_FALSE { return 0; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return 0; }
	if (*addr_of_mut!(OpenFiles))[handle as usize].valid == GOB_FALSE { return 0; }

	// make sure we're reading within the file
	let mut size = size;
	if (*addr_of_mut!(OpenFiles))[handle as usize].pos + size > (*addr_of_mut!(OpenFiles))[handle as usize].size {
		size = (*addr_of_mut!(OpenFiles))[handle as usize].size - (*addr_of_mut!(OpenFiles))[handle as usize].pos;
		if size == 0 { return 0; }
	}

	cache_id = FindBestCacheBlock((*addr_of_mut!(OpenFiles))[handle as usize].block);
	if cache_id < 0 { return GOB_INVALID_SIZE; }

	pos = (*addr_of_mut!(OpenFiles))[handle as usize].pos;

	loop {
		// are looking for data inside the read buffer?
		if (*(*addr_of_mut!(CacheBlocks)).add(cache_id as usize)).block == (*addr_of_mut!(OpenFiles))[handle as usize].block {
			// move any relevant data from the read buffer to the target buffer
			let mut buffer_size: GOBUInt32;

			// calc size of data we want from current buffer
			buffer_size = (*(*addr_of_mut!(CacheBlocks)).add(cache_id as usize)).size - (*addr_of_mut!(OpenFiles))[handle as usize].offset;
			if buffer_size > size { buffer_size = size; }

			// move from read buffer into output buffer
			memcpy(
				(buffer as *mut u8).add(((*addr_of_mut!(OpenFiles))[handle as usize].pos - pos) as usize) as *mut c_void,
				(*(*addr_of_mut!(CacheBlocks)).add(cache_id as usize)).data.add((*addr_of_mut!(OpenFiles))[handle as usize].offset as usize) as *const c_void,
				buffer_size as usize
			);

			// update file position
			(*addr_of_mut!(OpenFiles))[handle as usize].pos += buffer_size;
			(*addr_of_mut!(OpenFiles))[handle as usize].offset += buffer_size;

			// if we've completed this block -- move to next
			if (*addr_of_mut!(OpenFiles))[handle as usize].offset == (*(*addr_of_mut!(CacheBlocks)).add(cache_id as usize)).size {
				(*addr_of_mut!(OpenFiles))[handle as usize].block = (*(*addr_of!(BlockTable)).add((*addr_of_mut!(OpenFiles))[handle as usize].block as usize)).next;
				(*addr_of_mut!(OpenFiles))[handle as usize].offset = 0;
			}

			(*(*addr_of_mut!(CacheBlocks)).add(cache_id as usize)).time = *addr_of!(CacheBlockCounter);
			*addr_of_mut!(CacheBlockCounter) += 1;

			(*addr_of_mut!(ReadStats)).bufferUsed += buffer_size;
			size -= buffer_size;
			if size == 0 { break; }
		}

		// refill the buffer
		FillCacheBlock((*addr_of_mut!(OpenFiles))[handle as usize].block, cache_id as GOBUInt32);
		if (*(*addr_of_mut!(CacheBlocks)).add(cache_id as usize)).size == GOB_INVALID_SIZE {
			(*(*addr_of_mut!(CacheBlocks)).add(cache_id as usize)).block = GOB_INVALID_BLOCK;
			return GOB_INVALID_SIZE;
		}

		// reading off the end of the archive
		if (*(*addr_of_mut!(CacheBlocks)).add(cache_id as usize)).block != (*addr_of_mut!(OpenFiles))[handle as usize].block { break; }
	}

	return (*addr_of_mut!(OpenFiles))[handle as usize].pos - pos;
}

// GOBSeek
// Public function.  Seek to a position in an open file.
pub unsafe extern "C" fn GOBSeek(handle: GOBHandle, offset: GOBUInt32, type_: GOBSeekType, pos: *mut GOBUInt32) -> GOBError
{
	let mut blocks: GOBUInt32;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if (*addr_of_mut!(OpenFiles))[handle as usize].valid == GOB_FALSE { return GOBERR_NOT_OPEN; }

	// find a new position based on the seek type
	match type_ {
	GOBSEEK_START => {
		*pos = offset;
	},

	GOBSEEK_CURRENT => {
		*pos = (*addr_of_mut!(OpenFiles))[handle as usize].pos + offset;
	},

	GOBSEEK_END => {
		*pos = (*addr_of_mut!(OpenFiles))[handle as usize].size + offset;
	},

	_ => {
		return GOBERR_INVALID_SEEK;
	}
	}

	// check to make sure we're still in the file
	if *pos > (*addr_of_mut!(OpenFiles))[handle as usize].size {
		return GOBERR_INVALID_SEEK;
	}

	// update the file position
	(*addr_of_mut!(OpenFiles))[handle as usize].pos = *pos;

	// update block
	blocks = *pos / GOB_BLOCK_SIZE;
	(*addr_of_mut!(OpenFiles))[handle as usize].block = (*addr_of_mut!(OpenFiles))[handle as usize].startBlock;
	while blocks != 0 {
		(*addr_of_mut!(OpenFiles))[handle as usize].block = (*(*addr_of!(BlockTable)).add((*addr_of_mut!(OpenFiles))[handle as usize].block as usize)).next;
		blocks -= 1;
	}

	// update position inside block
	(*addr_of_mut!(OpenFiles))[handle as usize].offset = *pos % GOB_BLOCK_SIZE;

	return GOBERR_OK;
}


unsafe fn FindFreeBlock() -> GOBUInt32
{
	let mut i: GOBInt32;
	for i in 0..GOB_MAX_BLOCKS as GOBInt32 {
		if (*(*addr_of_mut!(BlockTable)).add(i as usize)).next == GOB_INVALID_BLOCK { return i as GOBUInt32; }
	}
	return GOB_MAX_BLOCKS;
}

// GOBWrite
// Public function.  Write an entire file.  The file should not be open!
pub unsafe extern "C" fn GOBWrite(buffer: *mut GOBVoid, size: GOBUInt32, mtime: GOBUInt32, file: *const GOBChar, codec_mask: GOBUInt32) -> GOBError
{
	let mut handle: GOBHandle;
	let mut slack: GOBInt32;
	let mut lfile: *mut GOBChar;
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

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if (*addr_of!(FileTableExt)).is_null() { return GOBERR_NO_EXTENDED; }
	if (*addr_of!(BlockCRC)).is_null() { return GOBERR_NO_EXTENDED; }

	InvalidateCache();

	// delete the file if it exists
	GOBDelete(file);

	// find a free entry in the file table
	handle = 0;
	while handle < GOB_MAX_FILES as GOBHandle {
		if (*(*addr_of_mut!(FileTableBasic)).add(handle as usize)).block == GOB_INVALID_BLOCK { break; }
		handle += 1;
	}

	if handle >= GOB_MAX_FILES as GOBHandle { return GOBERR_TOO_MANY_FILES; }
	if handle >= (*addr_of!(ArchiveNumFiles)) as GOBHandle { *addr_of_mut!(ArchiveNumFiles) = (handle + 1) as GOBUInt32; }

	// move to the end of the GOB
	if let Some(seek_fn) = (*addr_of!(FSFuncs)).seek {
		if seek_fn(*addr_of!(ArchiveHandle), 0, GOBSEEK_END) != 0 {
			return GOBERR_FILE_WRITE;
		}
	} else {
		return GOBERR_FILE_WRITE;
	}

	// alloc compression buffer
	out = if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		alloc_fn(GOB_BLOCK_SIZE + GOB_COMPRESS_OVERHEAD) as *mut GOBChar
	} else {
		std::ptr::null_mut()
	};

	last_block = GOB_MAX_BLOCKS - 1;

	pos = 0;
	while pos < size {
		let mut block: GOBUInt32;
		let mut in_size: GOBUInt32;

		// get a free block
		block = FindFreeBlock();
		if block >= GOB_MAX_BLOCKS { return GOBERR_TOO_MANY_BLOCKS; }
		if block >= *addr_of!(ArchiveNumBlocks) { *addr_of_mut!(ArchiveNumBlocks) = block + 1; }

		// if this is not the first block, mark next block for the last block
		// else assign the first block in file table
		if pos != 0 { (*(*addr_of_mut!(BlockTable)).add(last_block as usize)).next = block; }
		else { (*(*addr_of_mut!(FileTableBasic)).add(handle as usize)).block = block; }

		// invalidate the next block
		(*(*addr_of_mut!(BlockTable)).add(block as usize)).next = GOB_MAX_BLOCKS;

		// compute the decompressed block size
		in_size = size - pos;
		if in_size > GOB_BLOCK_SIZE { in_size = GOB_BLOCK_SIZE; }

		// compress block

		codec_index = 0;
		while codec_index < (*addr_of!(CodecFuncs)).codecs {
			if (GOB_CODEC_MASK(codec_index as u32) & codec_mask) == 0 {
				// skip if this codec is not listed as one of the allowed ones
				codec_index += 1;
				continue;
			}
			(*(*addr_of_mut!(BlockTable)).add(block as usize)).size = GOB_BLOCK_SIZE + GOB_COMPRESS_OVERHEAD;
			out_data = out;
			*(out_data as *mut GOBUInt32) = SwapBytes(GOBMARKER_STARTBLOCK);
			out_data = out_data.add(std::mem::size_of::<GOBUInt32>());
			*out_data = (*addr_of!(CodecFuncs)).codec[codec_index as usize].tag;
			out_data = out_data.add(1);
			if let Some(comp_fn) = (*addr_of!(CodecFuncs)).codec[codec_index as usize].compress {
				if comp_fn((buffer as *mut u8).add(pos as usize) as *mut GOBVoid,
					in_size, out_data as *mut GOBVoid, &mut (*(*addr_of_mut!(BlockTable)).add(block as usize)).size) != 0
				{
					return GOBERR_COMPRESS_FAIL;
				}
			} else {
				return GOBERR_COMPRESS_FAIL;
			}
			out_data = out_data.add((*(*addr_of_mut!(BlockTable)).add(block as usize)).size as usize);
			*(out_data as *mut GOBUInt32) = SwapBytes(GOBMARKER_ENDBLOCK);
			out_data = out_data.add(std::mem::size_of::<GOBUInt32>());

			// Adjust for the prefixed start-of-block marker and codec tag and trailing end-of-block marker
			compressed_size = (*(*addr_of_mut!(BlockTable)).add(block as usize)).size;
			(*(*addr_of_mut!(BlockTable)).add(block as usize)).size = compressed_size + 1 + (std::mem::size_of::<GOBUInt32>() * 2) as GOBUInt32;

			// Check compression result
			compression_ratio = (compressed_size * 100 / in_size) as GOBInt32;
			if compression_ratio <= (*addr_of!(CodecFuncs)).codec[codec_index as usize].max_ratio
			{
				// Compressed result is under par.  Let's go with it
				break;
			}

			// Otherwise, try the next compressor
			codec_index += 1;
		}

		// If no suitable codecs were found, take our ball and go home
		if codec_index == (*addr_of!(CodecFuncs)).codecs { return GOBERR_NO_SUITABLE_CODEC; }

		// compute and store the CRC
		*(*addr_of_mut!(BlockCRC)).add(block as usize) = adler32(0, std::ptr::null(), 0);
		*(*addr_of_mut!(BlockCRC)).add(block as usize) = adler32(*(*addr_of!(BlockCRC)).add(block as usize), out as *const u8,
					(*(*addr_of_mut!(BlockTable)).add(block as usize)).size);

		// write block
		if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
			if write_fn(*addr_of!(ArchiveHandle), out as *mut GOBVoid, (*(*addr_of_mut!(BlockTable)).add(block as usize)).size as GOBInt32) !=
				(*(*addr_of_mut!(BlockTable)).add(block as usize)).size as GOBInt32
			{
				return GOBERR_FILE_WRITE;
			}
		} else {
			return GOBERR_FILE_WRITE;
		}

		// compute the slack (to keep alignment)
		slack = GOBGetSlack((*(*addr_of_mut!(BlockTable)).add(block as usize)).size) as GOBInt32;

		// write the slack space
		memset(out as *mut c_void, 0, slack as usize);
		if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
			if write_fn(*addr_of!(ArchiveHandle), out as *mut GOBVoid, slack) != slack {
				return GOBERR_FILE_WRITE;
			}
		} else {
			return GOBERR_FILE_WRITE;
		}

		(*(*addr_of_mut!(BlockTable)).add(block as usize)).offset = *addr_of!(ArchiveSize);
		*addr_of_mut!(ArchiveSize) += (*(*addr_of_mut!(BlockTable)).add(block as usize)).size + slack as GOBUInt32;

		last_block = block;
		pos += GOB_BLOCK_SIZE;
	}

	if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
		free_fn(out as *mut GOBVoid);
	}

	lfile = LowerCase(file);

	// calculate file name hash
	hash = crc32(0, std::ptr::null(), 0);
	hash = crc32(hash, lfile as *const u8, strlen(lfile) as u32);

	// make sure hash is unique
	for i in 0..GOB_MAX_FILES as GOBInt32 {
		if i != handle &&
			(*(*addr_of_mut!(FileTableBasic)).add(i as usize)).block != GOB_INVALID_BLOCK &&
			(*(*addr_of_mut!(FileTableBasic)).add(i as usize)).hash == hash
		{
			return GOBERR_DUP_HASH;
		}
	}

	// update the file tables
	(*(*addr_of_mut!(FileTableBasic)).add(handle as usize)).hash = hash;
	(*(*addr_of_mut!(FileTableBasic)).add(handle as usize)).size = size;

	strcpy((*(*addr_of_mut!(FileTableExt)).add(handle as usize)).name.as_mut_ptr(), lfile);

	crc = crc32(0, std::ptr::null(), 0);
	crc = crc32(crc, buffer as *const u8, size);
	(*(*addr_of_mut!(FileTableExt)).add(handle as usize)).crc = crc;

	(*(*addr_of_mut!(FileTableExt)).add(handle as usize)).time = mtime;

	*addr_of_mut!(FileTableDirty) = GOB_TRUE;
	return GOBERR_OK;
}

// GOBDelete
// Public function.  Delete a file from a GOB.  The file should not be open!
pub unsafe extern "C" fn GOBDelete(file: *const GOBChar) -> GOBError
{
	let mut entry: GOBInt32;
	let mut lfile: *mut GOBChar;
	let mut block: GOBUInt32;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if (*addr_of!(FileTableExt)).is_null() { return GOBERR_NO_EXTENDED; }
	if (*addr_of!(BlockCRC)).is_null() { return GOBERR_NO_EXTENDED; }

	// find the file in the table
	lfile = LowerCase(file);

	entry = GetFileTableEntry(lfile);
	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	// invalidate blocks
	block = (*(*addr_of_mut!(FileTableBasic)).add(entry as usize)).block;
	loop {
		let mut next: GOBUInt32;
		next = (*(*addr_of_mut!(BlockTable)).add(block as usize)).next;
		(*(*addr_of_mut!(BlockTable)).add(block as usize)).next = GOB_INVALID_BLOCK;
		block = next;
		if block == GOB_MAX_BLOCKS { break; }
	}

	// invalidate the file
	(*(*addr_of_mut!(FileTableBasic)).add(entry as usize)).block = GOB_INVALID_BLOCK;

	*addr_of_mut!(FileTableDirty) = GOB_TRUE;

	return GOBERR_OK;
}

// GOBRearrange
// Public function.  Sorts the blocks in an archive.
pub unsafe extern "C" fn GOBRearrange(file: *const GOBChar, xlat: *const GOBUInt32, _rename: Option<GOBFileSysRenameFunc>) -> GOBError
{
	let mut err: GOBError;
	let mut buffer: *mut GOBVoid;
	let mut slack: GOBInt32;
	let mut slack_buf: *mut GOBVoid;
	let mut i: GOBUInt32;
	let mut size: GOBUInt32;
	let mut temp_handle: GOBFSHandle;
	let mut full_name: [GOBChar; GOB_MAX_FILE_NAME_LEN] = [0; GOB_MAX_FILE_NAME_LEN];

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if !InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_ALREADY_OPEN; }
	if (*addr_of!(FileTableExt)).is_null() { return GOBERR_NO_EXTENDED; }
	if (*addr_of!(BlockCRC)).is_null() { return GOBERR_NO_EXTENDED; }

	// start things up
	err = GOBArchiveOpen(file, GOBACCESS_READ, GOB_TRUE, GOB_TRUE);
	if err != GOBERR_OK { return err; }

	// create temporary file
	temp_handle = if let Some(open_fn) = (*addr_of!(FSFuncs)).open {
		open_fn("~temp.tmp\0".as_ptr() as *mut GOBChar, GOBACCESS_WRITE)
	} else {
		return GOBERR_FILE_WRITE;
	};
	if InvalidHandle(temp_handle) { return GOBERR_FILE_WRITE; }

	size = 0;

	// create an empty buffer for slack
	slack_buf = if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		alloc_fn(GOB_BLOCK_ALIGNMENT)
	} else {
		std::ptr::null_mut()
	};
	if slack_buf.is_null() { return GOBERR_NO_MEMORY; }
	memset(slack_buf, 0, GOB_BLOCK_ALIGNMENT as usize);

	// get memory for block
	buffer = if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		alloc_fn(GOB_BLOCK_SIZE + GOB_COMPRESS_OVERHEAD)
	} else {
		std::ptr::null_mut()
	};
	if buffer.is_null() { return GOBERR_NO_MEMORY; }

	// copy files in new order to end of archive
	for i in 0..(*addr_of!(ArchiveNumBlocks)) {
		if (*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).next != GOB_INVALID_BLOCK {
			// seek to the block
			if let Some(seek_fn) = (*addr_of!(FSFuncs)).seek {
				if seek_fn(*addr_of!(ArchiveHandle),
					(*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).offset as GOBInt32, GOBSEEK_START) != 0
				{
					return GOBERR_FILE_READ;
				}
			} else {
				return GOBERR_FILE_READ;
			}

			// read the block
			if let Some(read_fn) = (*addr_of!(FSFuncs)).read {
				if read_fn(*addr_of!(ArchiveHandle), buffer, (*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).size as GOBInt32) !=
					(*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).size as GOBInt32
				{
					return GOBERR_FILE_READ;
				}
			} else {
				return GOBERR_FILE_READ;
			}

			// write block
			if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
				if write_fn(temp_handle, buffer, (*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).size as GOBInt32) !=
					(*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).size as GOBInt32
				{
					return GOBERR_FILE_WRITE;
				}
			} else {
				return GOBERR_FILE_WRITE;
			}

			// write the slack
			slack = GOBGetSlack((*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).size) as GOBInt32;
			if let Some(write_fn) = (*addr_of!(FSFuncs)).write {
				if write_fn(temp_handle, slack_buf, slack) != slack {
					return GOBERR_FILE_WRITE;
				}
			} else {
				return GOBERR_FILE_WRITE;
			}

			// update block pos
			(*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).offset = size;
			size += (*(*addr_of_mut!(BlockTable)).add(*xlat.add(i as usize) as usize)).size + slack as GOBUInt32;
		}
	}

	if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
		free_fn(buffer);
	}
	if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
		free_fn(slack_buf);
	}

	// close the archive
	err = GOBArchiveClose();
	if err != GOBERR_OK { return err; }

	// close temp file
	let mut h = temp_handle;
	if let Some(close_fn) = (*addr_of!(FSFuncs)).close {
		close_fn(&mut h);
	}

	// overrwrite archive with temp file
	snprintf(full_name.as_mut_ptr(), GOB_MAX_FILE_NAME_LEN, "%s.gob\0".as_ptr() as *const c_char, file);
	if let Some(rename_fn) = _rename {
		if rename_fn("~temp.tmp\0".as_ptr() as *mut GOBChar, full_name.as_mut_ptr()) != 0 { return GOBERR_FILE_RENAME; }
	} else {
		return GOBERR_FILE_RENAME;
	}

	*addr_of_mut!(ArchiveSize) = size;

	CommitFileTable();

	return GOBERR_OK;
}


// GOBVerify
// Public function.  Verifies the integrity of a file.
pub unsafe extern "C" fn GOBVerify(file: *const GOBChar, status: *mut GOBBool) -> GOBError
{
	let mut handle: GOBHandle;
	let mut err: GOBError;
	let mut buffer: *mut GOBVoid;
	let mut size: GOBUInt32 = 0;
	let mut junk: GOBUInt32;
	let mut crc: GOBUInt32;
	let mut entry: GOBInt32;
	let mut lfile: *mut GOBChar;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if (*addr_of!(FileTableExt)).is_null() { return GOBERR_NO_EXTENDED; }
	if (*addr_of!(BlockCRC)).is_null() { return GOBERR_NO_EXTENDED; }

	// get the file size
	err = GOBGetSize(file, &mut size, &mut junk, &mut junk);
	if err != GOBERR_OK { return err; }

	// open the file
	err = GOBOpen(file as *mut GOBChar, &mut handle);
	if err != GOBERR_OK { return err; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);

	// alloc space for the file
	buffer = if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		alloc_fn(size)
	} else {
		std::ptr::null_mut()
	};
	if buffer.is_null() { return GOBERR_NO_MEMORY; }

	// read it into the buffer
	crc = GOBRead(buffer, size, handle);
	if crc != size { return GOBERR_FILE_READ; }

	// calc the crc
	crc = crc32(0, std::ptr::null(), 0);
	crc = crc32(crc, buffer as *const u8, size);

	if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
		free_fn(buffer);
	}

	// verify the crc matches
	if crc != (*(*addr_of_mut!(FileTableExt)).add(entry as usize)).crc { *status = GOB_FALSE; }
	else { *status = GOB_TRUE; }

	err = GOBClose(handle);
	if err != GOBERR_OK { return err; }

	return GOBERR_OK;
}

// GOBGetSize
// Public function.  Get a file compressed, decompressed, slack sizes.
pub unsafe extern "C" fn GOBGetSize(file: *const GOBChar,
	decomp: *mut GOBUInt32, comp: *mut GOBUInt32, slack: *mut GOBUInt32) -> GOBError
{
	let mut entry: GOBInt32;
	let mut lfile: *mut GOBChar;
	let mut block: GOBUInt32;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	// get file table entry
	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);
	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	// decompressed size from file table
	*decomp = (*(*addr_of_mut!(FileTableBasic)).add(entry as usize)).size;

	// compressed size is sum of block sizes
	*comp = 0;
	*slack = 0;
	block = (*(*addr_of_mut!(FileTableBasic)).add(entry as usize)).block;
	while block != GOB_MAX_BLOCKS {
		*comp += (*(*addr_of_mut!(BlockTable)).add(block as usize)).size;
		*slack += GOBGetSlack((*(*addr_of_mut!(BlockTable)).add(block as usize)).size);
		block = (*(*addr_of_mut!(BlockTable)).add(block as usize)).next;
	}

	return GOBERR_OK;
}

// GOBGetTime
// Public function.  Get a file modification time.
pub unsafe extern "C" fn GOBGetTime(file: *const GOBChar, time: *mut GOBUInt32) -> GOBError
{
	let mut entry: GOBInt32;
	let mut lfile: *mut GOBChar;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if (*addr_of!(FileTableExt)).is_null() { return GOBERR_NO_EXTENDED; }
	if (*addr_of!(BlockCRC)).is_null() { return GOBERR_NO_EXTENDED; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);
	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	*time = (*(*addr_of_mut!(FileTableExt)).add(entry as usize)).time;
	return GOBERR_OK;
}

// GOBGetCRC
// Public function.  Get a file CRC.
pub unsafe extern "C" fn GOBGetCRC(file: *const GOBChar, crc: *mut GOBUInt32) -> GOBError
{
	let mut entry: GOBInt32;
	let mut lfile: *mut GOBChar;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	if (*addr_of!(FileTableExt)).is_null() { return GOBERR_NO_EXTENDED; }
	if (*addr_of!(BlockCRC)).is_null() { return GOBERR_NO_EXTENDED; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);
	if entry == -1 { return GOBERR_FILE_NOT_FOUND; }

	*crc = (*(*addr_of_mut!(FileTableExt)).add(entry as usize)).crc;
	return GOBERR_OK;
}

// GOBAccess
// Public function.  Determine if a file exists in the archive.
pub unsafe extern "C" fn GOBAccess(file: *const GOBChar, status: *mut GOBBool) -> GOBError
{
	let mut entry: GOBInt32;
	let mut lfile: *mut GOBChar;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);
	if entry == -1 { *status = GOB_FALSE; }
	else { *status = GOB_TRUE; }

	return GOBERR_OK;
}

// GOBGetFileCode
// Public function.  Find the index into the file table of a file.
pub unsafe extern "C" fn GOBGetFileCode(file: *const GOBChar) -> GOBInt32
{
	let mut entry: GOBInt32;
	let mut lfile: *mut GOBChar;

	if *addr_of!(LibraryInit) == GOB_FALSE { return -1; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return -1; }

	lfile = LowerCase(file);
	entry = GetFileTableEntry(lfile);

	return entry;
}

// GOBGetFileTables
// Public function.  Return the active file tables.
pub unsafe extern "C" fn GOBGetFileTables(basic: *mut *mut GOBFileTableBasicEntry,
	ext: *mut *mut GOBFileTableExtEntry) -> GOBError
{
	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	*basic = *addr_of_mut!(FileTableBasic);
	*ext = *addr_of_mut!(FileTableExt);
	return GOBERR_OK;
}

// GOBGetBlockTable
// Public function.  Return the active block table.
pub unsafe extern "C" fn GOBGetBlockTable(table: *mut *mut GOBBlockTableEntry, num: *mut GOBUInt32) -> GOBError
{
	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }
	if InvalidHandle(*addr_of!(ArchiveHandle)) { return GOBERR_NOT_OPEN; }
	*table = *addr_of_mut!(BlockTable);
	*num = *addr_of!(ArchiveNumBlocks);
	return GOBERR_OK;
}

// GOBSetCacheSize
// Public function.  Allocates buffers to cache blocks.
pub unsafe extern "C" fn GOBSetCacheSize(num: GOBUInt32) -> GOBError
{
	let mut i: GOBUInt32;

	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }

	// only continue if we actually need to resize
	if num == *addr_of!(NumCacheBlocks) { return GOBERR_OK; }

	// free old cache buffers
	FreeCache();

	*addr_of_mut!(NumCacheBlocks) = 0;

	if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		*addr_of_mut!(CacheBlocks) = alloc_fn(
				(std::mem::size_of::<GOBBlockCache>() as GOBUInt32 * num)) as *mut GOBBlockCache;
	}
	if (*addr_of!(CacheBlocks)).is_null() { return GOBERR_NO_MEMORY; }

	// allocate cache blocks and initialize
	for i in 0..num {
		if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
			(*(*addr_of_mut!(CacheBlocks)).add(i as usize)).data = alloc_fn(GOB_BLOCK_SIZE) as *mut GOBChar;
		}
		if (*(*addr_of_mut!(CacheBlocks)).add(i as usize)).data.is_null() { return GOBERR_NO_MEMORY; }

		(*(*addr_of_mut!(CacheBlocks)).add(i as usize)).size = 0;
		(*(*addr_of_mut!(CacheBlocks)).add(i as usize)).time = 0;
		(*(*addr_of_mut!(CacheBlocks)).add(i as usize)).block = 0xFFFFFFFF;

		*addr_of_mut!(NumCacheBlocks) += 1;
	}

	return GOBERR_OK;
}

// GOBSetReadBufferSize
// Public function.  Allocate a read ahead buffer.
pub unsafe extern "C" fn GOBSetReadBufferSize(size: GOBUInt32) -> GOBError
{
	if *addr_of!(LibraryInit) == GOB_FALSE { return GOBERR_NOT_INIT; }

	// only continue if we actually need to resize
	if size == (*addr_of!(ReadBuffer)).size { return GOBERR_OK; }

	// remove old buffer
	if !(*addr_of!(ReadBuffer)).data.is_null() {
		if let Some(free_fn) = (*addr_of!(MemFuncs)).free {
			free_fn((*addr_of_mut!(ReadBuffer)).data as *mut GOBVoid);
		}
	}

	// allocate new buffer
	if let Some(alloc_fn) = (*addr_of!(MemFuncs)).alloc {
		(*addr_of_mut!(ReadBuffer)).data = alloc_fn(size + GOB_MEM_ALIGNMENT) as *mut GOBChar;
	}
	if (*addr_of!(ReadBuffer)).data.is_null() { return GOB_INVALID_SIZE as GOBError; }

	// set aligned pointer
	(*addr_of_mut!(ReadBuffer)).dataStart =
		(*addr_of_mut!(ReadBuffer)).data.add((GOB_MEM_ALIGNMENT as usize) -
		((*addr_of_mut!(ReadBuffer)).data as u32 % GOB_MEM_ALIGNMENT) as usize);

	(*addr_of_mut!(ReadBuffer)).pos = 0xFFFFFFFF;
	(*addr_of_mut!(ReadBuffer)).size = size;

	return GOBERR_OK;
}

// GOBGetReadStats
// Public function.  Get file read statistics (seeks, sizes).
pub unsafe extern "C" fn GOBGetReadStats() -> GOBReadStats
{
	return *addr_of!(ReadStats);
}


pub unsafe extern "C" fn GOBSetProfileFuncs(fset: *mut GOBProfileFuncSet)
{
	*addr_of_mut!(ProfileReadCallback) = (*fset).read;
}

pub unsafe extern "C" fn GOBStartProfile() -> GOBError
{
	if *addr_of!(ProfileEnabled) != GOB_FALSE { return GOBERR_PROFILE_ON; }
	*addr_of_mut!(ProfileEnabled) = GOB_TRUE;
	return GOBERR_OK;
}

pub unsafe extern "C" fn GOBStopProfile() -> GOBError
{
	if *addr_of!(ProfileEnabled) == GOB_FALSE { return GOBERR_PROFILE_OFF; }
	*addr_of_mut!(ProfileEnabled) = GOB_FALSE;
	return GOBERR_OK;
}
