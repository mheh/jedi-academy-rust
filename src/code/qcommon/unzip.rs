/*****************************************************************************
 * name:		unzip.c
 *
 * desc:		IO on .zip files using portions of zlib
 *
 * $Archive: /StarTrek/Code-Single/qcommon/unzip.cpp $
 * $Author: Jmonroe $
 * $Revision: 5 $
 * $Modtime: 4/10/01 6:47p $
 * $Date: 4/10/01 7:28p $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::mem;
use core::ptr::{addr_of, addr_of_mut};

// ZIP_fopen, ZIP_fclose, ZIP_fseek, ZIP_fread, ZIP_ftell are mapped to C stdio functions
// which will be declared as externs below

// unzip.h -- IO for uncompress .zip files using zlib
//   Version 0.15 beta, Mar 19th, 1998,
//
//   Copyright (C) 1998 Gilles Vollant
//
//   This unzip package allow extract file from .ZIP file, compatible with PKZip 2.04g
//     WinZip, InfoZip tools and compatible.
//   Encryption and multi volume ZipFile (span) are not supported.
//   Old compressions used by old PKZip 1.x are not supported
//
//   THIS IS AN ALPHA VERSION. AT THIS STAGE OF DEVELOPPEMENT, SOMES API OR STRUCTURE
//   CAN CHANGE IN FUTURE VERSION !!
//   I WAIT FEEDBACK at mail info@winimage.com
//   Visit also http://www.winimage.com/zLibDll/unzip.htm for evolution
//
//   Condition of use and distribution are the same than zlib :
//
//  This software is provided 'as-is', without any express or implied
//  warranty.  In no event will the authors be held liable for any damages
//  arising from the use of this software.
//
//  Permission is granted to anyone to use this software for any purpose,
//  including commercial applications, and to alter it and redistribute it
//  freely, subject to the following restrictions:
//
//  1. The origin of this software must not be misrepresented; you must not
//     claim that you wrote the original software. If you use this software
//     in a product, an acknowledgment in the product documentation would be
//     appreciated but is not required.
//  2. Altered source versions must be plainly marked as such, and must not be
//     misrepresented as being the original software.
//  3. This notice may not be removed or altered from any source distribution.


// Type declarations

// function prototypes
// (OF macro equivalent not needed in Rust)

type Byte = u8;  // 8 bits
type uInt = u32;  // 16 bits or more
type uLong = u32; // 32 bits or more
type voidp = *mut Byte;

// Seek constants
const SEEK_SET: c_int = 0;       // Seek from beginning of file.
const SEEK_CUR: c_int = 1;       // Seek from current position.
const SEEK_END: c_int = 2;       // Set file pointer to EOF plus "offset"

type gzFile = voidp;

// gzopen
//     Opens a gzip (.gz) file for reading or writing. The mode parameter
//   is as in fopen ("rb" or "wb") but can also include a compression level
//   ("wb9") or a strategy: 'f' for filtered data as in "wb6f", 'h' for
//   Huffman only compression as in "wb1h". (See the description
//   of deflateInit2 for more information about the strategy parameter.)
//
//     gzopen can be used to read a file which is not in gzip format; in this
//   case gzread will directly read from the file without decompression.
//
//     gzopen returns NULL if the file could not be opened or if there was
//   insufficient memory to allocate the (de)compression state; errno
//   can be checked to distinguish the two cases (if errno is zero, the
//   zlib error is Z_MEM_ERROR).

extern "C" {
    fn gzopen(path: *const c_char, mode: *const c_char) -> gzFile;

    // gzdopen() associates a gzFile with the file descriptor fd.  File
    // descriptors are obtained from calls like open, dup, creat, pipe or
    // fileno (in the file has been previously opened with fopen).
    // The mode parameter is as in gzopen.
    //   The next call of gzclose on the returned gzFile will also close the
    // file descriptor fd, just like fclose(fdopen(fd), mode) closes the file
    // descriptor fd. If you want to keep fd open, use gzdopen(dup(fd), mode).
    //   gzdopen returns NULL if there was insufficient memory to allocate
    // the (de)compression state.
    fn gzdopen(fd: c_int, mode: *const c_char) -> gzFile;

    // Dynamically update the compression level or strategy. See the description
    // of deflateInit2 for the meaning of these parameters.
    //   gzsetparams returns Z_OK if success, or Z_STREAM_ERROR if the file was not
    // opened for writing.
    fn gzsetparams(file: gzFile, level: c_int, strategy: c_int) -> c_int;

    // Reads the given number of uncompressed bytes from the compressed file.
    // If the input file was not in gzip format, gzread copies the given number
    // of bytes into the buffer.
    //   gzread returns the number of uncompressed bytes actually read (0 for
    // end of file, -1 for error).
    fn gzread(file: gzFile, buf: voidp, len: c_int) -> c_int;

    // Writes the given number of uncompressed bytes into the compressed file.
    // gzwrite returns the number of uncompressed bytes actually written
    // (0 in case of error).
    fn gzwrite(file: gzFile, buf: *const c_void, len: c_int) -> c_int;

    // Converts, formats, and writes the args to the compressed file under
    // control of the format string, as in fprintf. gzprintf returns the number of
    // uncompressed bytes actually written (0 in case of error).
    // fn gzprintf(file: gzFile, format: *const c_char, ...) -> c_int;

    // Writes the given null-terminated string to the compressed file, excluding
    // the terminating null character.
    //   gzputs returns the number of characters written, or -1 in case of error.
    fn gzputs(file: gzFile, s: *const c_char) -> c_int;

    // Reads bytes from the compressed file until len-1 characters are read, or
    // a newline character is read and transferred to buf, or an end-of-file
    // condition is encountered.  The string is then terminated with a null
    // character.
    //   gzgets returns buf, or Z_NULL in case of error.
    fn gzgets(file: gzFile, buf: *mut c_char, len: c_int) -> *mut c_char;

    // Writes c, converted to an unsigned char, into the compressed file.
    // gzputc returns the value that was written, or -1 in case of error.
    fn gzputc(file: gzFile, c: c_int) -> c_int;

    // Reads one byte from the compressed file. gzgetc returns this byte
    // or -1 in case of end of file or error.
    fn gzgetc(file: gzFile) -> c_int;

    // Flushes all pending output into the compressed file. The parameter
    // flush is as in the deflate() function. The return value is the zlib
    // error number (see function gzerror below). gzflush returns Z_OK if
    // the flush parameter is Z_FINISH and all output could be flushed.
    //   gzflush should be called only when strictly necessary because it can
    // degrade compression.
    fn gzflush(file: gzFile, flush: c_int) -> c_int;

    // Sets the starting position for the next gzread or gzwrite on the
    // given compressed file. The offset represents a number of bytes in the
    // uncompressed data stream. The whence parameter is defined as in lseek(2);
    // the value SEEK_END is not supported.
    //   If the file is opened for reading, this function is emulated but can be
    // extremely slow. If the file is opened for writing, only forward seeks are
    // supported; gzseek then compresses a sequence of zeroes up to the new
    // starting position.
    //
    //   gzseek returns the resulting offset location as measured in bytes from
    // the beginning of the uncompressed stream, or -1 in case of error, in
    // particular if the file is opened for writing and the new starting position
    // would be before the current position.
    fn gzseek(file: gzFile, offset: c_int, whence: c_int) -> c_int;

    // Rewinds the given file. This function is supported only for reading.
    //
    // gzrewind(file) is equivalent to (int)gzseek(file, 0L, SEEK_SET)
    fn gzrewind(file: gzFile) -> c_int;

    // Returns the starting position for the next gzread or gzwrite on the
    // given compressed file. This position represents a number of bytes in the
    // uncompressed data stream.
    //
    // gztell(file) is equivalent to gzseek(file, 0L, SEEK_CUR)
    fn gztell(file: gzFile) -> c_int;

    // Returns 1 when EOF has previously been detected reading the given
    // input stream, otherwise zero.
    fn gzeof(file: gzFile) -> c_int;

    // Flushes all pending output if necessary, closes the compressed file
    // and deallocates all the (de)compression state. The return value is the zlib
    // error number (see function gzerror below).
    fn gzclose(file: gzFile) -> c_int;

    // Returns the error message for the last error which occurred on the
    // given compressed file. errnum is set to zlib error number. If an
    // error occurred in the file system and not in the compression library,
    // errnum is set to Z_ERRNO and the application may consult errno
    // to get the exact error code.
    fn gzerror(file: gzFile, errnum: *mut c_int) -> *const c_char;

    // C stdio functions via macros
    fn fopen(path: *const c_char, mode: *const c_char) -> *mut c_void;
    fn fclose(file: *mut c_void) -> c_int;
    fn fseek(file: *mut c_void, offset: c_int, whence: c_int) -> c_int;
    fn fread(buf: *mut c_void, size: usize, count: usize, file: *mut c_void) -> usize;
    fn ftell(file: *mut c_void) -> c_int;

    fn Z_Malloc(size: c_int, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(buf: *mut c_void);

    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    fn LittleShort(v: i16) -> uLong;
    fn LittleLong(v: c_int) -> uLong;

    fn crc32(crc: uLong, buf: *const Byte, len: uInt) -> uLong;

    fn inflateInit(stream: *mut z_stream, method: c_int, wbits: c_int) -> c_int;
    fn inflate(stream: *mut z_stream) -> c_int;
    fn inflateEnd(stream: *mut z_stream) -> c_int;
}

// ZIP file macros map to stdio functions
type ZIP_FILE = c_void;

// Case sensitivity defaults
#[cfg(target_os = "unix")]
const CASESENSITIVITYDEFAULT_YES: c_int = 1;

#[cfg(not(target_os = "unix"))]
const CASESENSITIVITYDEFAULTVALUE: c_int = 2;

#[cfg(target_os = "unix")]
const CASESENSITIVITYDEFAULTVALUE: c_int = 1;

#[cfg(not(target_os = "unix"))]
const CASESENSITIVITYDEFAULTVALUE: c_int = 2;

const UNZ_BUFSIZE: uInt = 65536;
const UNZ_MAXFILENAMEINZIP: usize = 256;

const SIZECENTRALDIRITEM: uInt = 0x2e;
const SIZEZIPLOCALHEADER: uInt = 0x1e;

// Error codes
const UNZ_OK: c_int = 0;
const UNZ_EOF: c_int = -1;
const UNZ_ERRNO: c_int = -100;
const UNZ_PARAMERROR: c_int = -102;
const UNZ_BADZIPFILE: c_int = -103;
const UNZ_CRCERROR: c_int = -104;
const UNZ_END_OF_LIST_OF_FILE: c_int = -105;

// Compression method constants
const ZF_DEFLATED: uLong = 8;

// zlib constants
const Z_OK: c_int = 0;
const Z_STREAM_END: c_int = 1;
const Z_SYNC_FLUSH: c_int = 3;
const Z_FINISH: c_int = 4;

// Memory tags
const TAG_FILESYS: c_int = 0;

// Buffer sizes for central dir search
const BUFREADCOMMENT: uInt = 0x400;

// Structure definitions from zlib/unzip headers
#[repr(C)]
pub struct z_stream {
    pub next_in: *mut Byte,     // next input byte
    pub avail_in: uInt,         // number of bytes available at next_in
    pub total_in: uLong,        // total nb of input bytes read so far

    pub next_out: *mut Byte,    // next output byte will go here
    pub avail_out: uInt,        // remaining free space at next_out
    pub total_out: uLong,       // total nb of bytes output so far

    pub msg: *mut c_char,       // last error message, NULL if no error
    // struct internal_state *state; // not exposed by zlib
    pub state: *mut c_void,     // internal state

    pub zalloc: *mut c_void,    // alloc function
    pub zfree: *mut c_void,     // free function
    pub opaque: *mut c_void,    // private data object passed to zalloc and zfree
}

#[repr(C)]
pub struct tm_unz {
    pub tm_sec: uInt,   // seconds after minute
    pub tm_min: uInt,   // minutes after hour
    pub tm_hour: uInt,  // hours since midnight
    pub tm_mday: uInt,  // day of month
    pub tm_mon: uInt,   // months since January
    pub tm_year: uInt,  // years since 1980
}

#[repr(C)]
pub struct unz_file_info {
    pub version: uLong,                // version made by
    pub version_needed: uLong,         // version needed to extract
    pub flag: uLong,                   // general purpose bit flag
    pub compression_method: uLong,     // compression method
    pub dosDate: uLong,                // last mod file date in Dos fmt
    pub crc: uLong,                    // crc-32
    pub compressed_size: uLong,        // compressed size
    pub uncompressed_size: uLong,      // uncompressed size
    pub size_filename: uLong,          // filename length
    pub size_file_extra: uLong,        // extra field length
    pub size_file_comment: uLong,      // file comment length
    pub disk_num_start: uLong,         // disk number start
    pub internal_fa: uLong,            // internal file attributes
    pub external_fa: uLong,            // external file attributes
    pub tmu_date: tm_unz,
}

#[repr(C)]
pub struct unz_file_info_internal {
    pub offset_curfile: uLong,         // relative offset of local header
}

#[repr(C)]
pub struct unz_global_info {
    pub number_entry: uLong,           // total number of entries in the central dir on this disk
    pub size_comment: uLong,           // zipfile comment length
}

#[repr(C)]
pub struct file_in_zip_read_info_s {
    pub read_buffer: *mut c_char,
    pub stream: z_stream,
    pub pos_in_zipfile: uLong,         // position in byte on the zipfile, for fseek
    pub stream_initialised: c_int,     // flag set if stream structure is initialised
    pub offset_local_extrafield: uLong,
    pub size_local_extrafield: uInt,
    pub pos_local_extrafield: uLong,
    pub file: *mut ZIP_FILE,
    pub byte_before_the_zipfile: uLong,
    pub rest_read_compressed: uLong,
    pub rest_read_uncompressed: uLong,
    pub crc32: uLong,
    pub crc32_wait: uLong,
    pub compression_method: uLong,
}

pub type unzFile = *mut unz_s;

#[repr(C)]
pub struct unz_s {
    pub file: *mut ZIP_FILE,
    pub gi: unz_global_info,
    pub byte_before_the_zipfile: uLong,
    pub central_pos: uLong,
    pub offset_central_dir: uLong,
    pub size_central_dir: uLong,
    pub pos_in_central_dir: uLong,
    pub num_file: uLong,
    pub cur_file_info: unz_file_info,
    pub cur_file_info_internal: unz_file_info_internal,
    pub current_file_ok: c_int,
    pub pfile_in_zip_read: *mut file_in_zip_read_info_s,
}

// ALLOC macro
#[inline]
fn ALLOC(size: usize) -> *mut c_void {
    unsafe { Z_Malloc(size as c_int, TAG_FILESYS, 0) }
}

// TRYFREE macro
#[inline]
fn TRYFREE(p: *mut c_void) {
    if !p.is_null() {
        unsafe { Z_Free(p); }
    }
}

fn UNZ_Malloc(size: c_int) -> *mut c_void {
    let buf: *mut c_void;

    buf = unsafe { Z_Malloc(size, TAG_FILESYS, 0) };
    buf
}

// Reads a long in LSB order from the given gz_stream. Sets
fn unzlocal_getShort(fin: *mut ZIP_FILE, pX: *mut uLong) -> c_int {
    let mut v: i16 = 0;

    unsafe {
        fread(
            addr_of_mut!(v) as *mut c_void,
            mem::size_of::<i16>(),
            1,
            fin,
        );

        *pX = LittleShort(v);
    }
    UNZ_OK
}

fn unzlocal_getLong(fin: *mut ZIP_FILE, pX: *mut uLong) -> c_int {
    let mut v: c_int = 0;

    unsafe {
        fread(
            addr_of_mut!(v) as *mut c_void,
            mem::size_of::<c_int>(),
            1,
            fin,
        );

        *pX = LittleLong(v);
    }
    UNZ_OK
}

// My own strcmpi / strcasecmp
fn strcmpcasenosensitive_internal(fileName1: *const c_char, fileName2: *const c_char) -> c_int {
    unsafe {
        let mut p1 = fileName1;
        let mut p2 = fileName2;

        loop {
            let mut c1: u8 = *p1 as u8;
            let mut c2: u8 = *p2 as u8;

            if c1 >= b'a' && c1 <= b'z' {
                c1 -= 0x20;
            }
            if c2 >= b'a' && c2 <= b'z' {
                c2 -= 0x20;
            }
            if c1 == b'\0' {
                return if c2 == b'\0' { 0 } else { -1 };
            }
            if c2 == b'\0' {
                return 1;
            }
            if (c1 as i32) < (c2 as i32) {
                return -1;
            }
            if (c1 as i32) > (c2 as i32) {
                return 1;
            }

            p1 = p1.add(1);
            p2 = p2.add(1);
        }
    }
}

// Compare two filename (fileName1,fileName2).
// If iCaseSenisivity = 1, comparision is case sensitivity (like strcmp)
// If iCaseSenisivity = 2, comparision is not case sensitivity (like strcmpi
//                                                              or strcasecmp)
// If iCaseSenisivity = 0, case sensitivity is defaut of your operating system
//      (like 1 on Unix, 2 on Windows)

pub extern "C" fn unzStringFileNameCompare(
    fileName1: *const c_char,
    fileName2: *const c_char,
    mut iCaseSensitivity: c_int,
) -> c_int {
    if iCaseSensitivity == 0 {
        iCaseSensitivity = CASESENSITIVITYDEFAULTVALUE;
    }

    if iCaseSensitivity == 1 {
        return unsafe { strcmp(fileName1, fileName2) };
    }

    strcmpcasenosensitive_internal(fileName1, fileName2)
}

// Locate the Central directory of a zipfile (at the end, just before
//   the global comment)
fn unzlocal_SearchCentralDir(fin: *mut ZIP_FILE) -> uLong {
    let mut buf: *mut Byte;
    let mut uSizeFile: uLong;
    let mut uBackRead: uLong;
    let uMaxBack: uLong = 0xffff; /* maximum size of global comment */
    let mut uPosFound: uLong = 0;

    unsafe {
        if fseek(fin, 0, SEEK_END) != 0 {
            return 0;
        }

        uSizeFile = ftell(fin) as uLong;

        let mut uMaxBackMut = uMaxBack;
        if uMaxBackMut > uSizeFile {
            uMaxBackMut = uSizeFile;
        }

        buf = ALLOC((BUFREADCOMMENT + 4) as usize) as *mut Byte;

        uBackRead = 4;
        while uBackRead < uMaxBackMut {
            let uReadSize: uLong;
            let uReadPos: uLong;
            let mut i: c_int;

            let mut uBackReadMut = uBackRead;
            if uBackReadMut + BUFREADCOMMENT > uMaxBackMut {
                uBackReadMut = uMaxBackMut;
            } else {
                uBackReadMut += BUFREADCOMMENT;
            }
            uBackRead = uBackReadMut;
            uReadPos = uSizeFile - uBackRead;

            uReadSize = if (BUFREADCOMMENT + 4) < (uSizeFile - uReadPos) {
                BUFREADCOMMENT + 4
            } else {
                uSizeFile - uReadPos
            };

            if fseek(fin, uReadPos as c_int, SEEK_SET) != 0 {
                break;
            }

            if fread(buf as *mut c_void, uReadSize as usize, 1, fin) != 1 {
                break;
            }

            i = (uReadSize as c_int) - 3;
            while {
                i -= 1;
                i > 0
            } {
                if (*buf.add(i as usize) == 0x50)
                    && (*buf.add((i + 1) as usize) == 0x4b)
                    && (*buf.add((i + 2) as usize) == 0x05)
                    && (*buf.add((i + 3) as usize) == 0x06)
                {
                    uPosFound = uReadPos + i as uLong;
                    break;
                }
            }

            if uPosFound != 0 {
                break;
            }
        }
        TRYFREE(buf as *mut c_void);
    }
    uPosFound
}

pub extern "C" fn unzReOpen(path: *const c_char, file: unzFile) -> unzFile {
    let mut s: *mut unz_s;
    let fin: *mut ZIP_FILE;

    unsafe {
        fin = fopen(path, b"rb\0".as_ptr() as *const c_char) as *mut ZIP_FILE;
        if fin.is_null() {
            return core::ptr::null_mut();
        }

        s = ALLOC(mem::size_of::<unz_s>()) as *mut unz_s;
        memcpy(
            s as *mut c_void,
            file as *const c_void,
            mem::size_of::<unz_s>(),
        );

        (*s).file = fin;
        s as unzFile
    }
}

// Open a Zip file. path contain the full pathname (by example,
//    on a Windows NT computer "c:\\test\\zlib109.zip" or on an Unix computer
//    "zlib/zlib109.zip".
//    If the zipfile cannot be opened (file don't exist or in not valid), the
//      return value is NULL.
//    Else, the return value is a unzFile Handle, usable with other function
//    of this unzip package.
pub extern "C" fn unzOpen(path: *const c_char) -> unzFile {
    let mut us: unz_s = unsafe { mem::zeroed() };
    let mut s: *mut unz_s;
    let mut central_pos: uLong;
    let mut uL: uLong;
    let fin: *mut ZIP_FILE;

    let mut number_disk: uLong;          // number of the current dist, used for
                                          //   spaning ZIP, unsupported, always 0
    let mut number_disk_with_CD: uLong;  // number the the disk with central dir, used
                                          //   for spaning ZIP, unsupported, always 0
    let mut number_entry_CD: uLong;      // total number of entries in
                                          //   the central dir
                                          //   (same than number_entry on nospan)

    let mut err: c_int = UNZ_OK;

    unsafe {
        fin = fopen(path, b"rb\0".as_ptr() as *const c_char) as *mut ZIP_FILE;
        if fin.is_null() {
            return core::ptr::null_mut();
        }

        central_pos = unzlocal_SearchCentralDir(fin);
        if central_pos == 0 {
            err = UNZ_ERRNO;
        }

        if fseek(fin, central_pos as c_int, SEEK_SET) != 0 {
            err = UNZ_ERRNO;
        }

        // the signature, already checked
        if unzlocal_getLong(fin, addr_of_mut!(uL)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        // number of this disk
        if unzlocal_getShort(fin, addr_of_mut!(number_disk)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        // number of the disk with the start of the central directory
        if unzlocal_getShort(fin, addr_of_mut!(number_disk_with_CD)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        // total number of entries in the central dir on this disk
        if unzlocal_getShort(fin, addr_of_mut!(us.gi.number_entry)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        // total number of entries in the central dir
        if unzlocal_getShort(fin, addr_of_mut!(number_entry_CD)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if (number_entry_CD != us.gi.number_entry)
            || (number_disk_with_CD != 0)
            || (number_disk != 0)
        {
            err = UNZ_BADZIPFILE;
        }

        // size of the central directory
        if unzlocal_getLong(fin, addr_of_mut!(us.size_central_dir)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        // offset of start of central directory with respect to the
        //       starting disk number
        if unzlocal_getLong(fin, addr_of_mut!(us.offset_central_dir)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        // zipfile comment length
        if unzlocal_getShort(fin, addr_of_mut!(us.gi.size_comment)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if (central_pos < us.offset_central_dir + us.size_central_dir) && (err == UNZ_OK) {
            err = UNZ_BADZIPFILE;
        }

        if err != UNZ_OK {
            fclose(fin as *mut c_void);
            return core::ptr::null_mut();
        }

        us.file = fin;
        us.byte_before_the_zipfile = central_pos - (us.offset_central_dir + us.size_central_dir);
        us.central_pos = central_pos;
        us.pfile_in_zip_read = core::ptr::null_mut();

        s = ALLOC(mem::size_of::<unz_s>()) as *mut unz_s;
        *s = us;
        //	unzGoToFirstFile((unzFile)s);
        s as unzFile
    }
}

// Close a ZipFile opened with unzipOpen.
// If there is files inside the .Zip opened with unzipOpenCurrentFile (see later),
//   these files MUST be closed with unzipCloseCurrentFile before call unzipClose.
// return UNZ_OK if there is no problem.
pub extern "C" fn unzClose(file: unzFile) -> c_int {
    let s: *mut unz_s;
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;

    unsafe {
        if !(*s).pfile_in_zip_read.is_null() {
            unzCloseCurrentFile(file);
        }

        fclose((*s).file as *mut c_void);
        TRYFREE(s as *mut c_void);
    }
    UNZ_OK
}

// Write info about the ZipFile in the *pglobal_info structure.
// No preparation of the structure is needed
// return UNZ_OK if there is no problem.
pub extern "C" fn unzGetGlobalInfo(
    file: unzFile,
    pglobal_info: *mut unz_global_info,
) -> c_int {
    let s: *mut unz_s;
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;
    unsafe {
        *pglobal_info = (*s).gi;
    }
    UNZ_OK
}

// Translate date/time from Dos format to tm_unz (readable more easilty)
fn unzlocal_DosDateToTmuDate(ulDosDate: uLong, ptm: *mut tm_unz) {
    let uDate: uLong;
    uDate = ulDosDate >> 16;

    unsafe {
        (*ptm).tm_mday = (uDate & 0x1f);
        (*ptm).tm_mon = ((((uDate) & 0x1E0) / 0x20) - 1);
        (*ptm).tm_year = (((uDate & 0x0FE00) / 0x0200) + 1980);

        (*ptm).tm_hour = ((ulDosDate & 0xF800) / 0x800);
        (*ptm).tm_min = ((ulDosDate & 0x7E0) / 0x20);
        (*ptm).tm_sec = (2 * (ulDosDate & 0x1f));
    }
}

// Get Info about the current file in the zipfile, with internal only info
fn unzlocal_GetCurrentFileInfoInternal(
    file: unzFile,
    pfile_info: *mut unz_file_info,
    pfile_info_internal: *mut unz_file_info_internal,
    szFileName: *mut c_char,
    fileNameBufferSize: uLong,
    extraField: *mut c_void,
    extraFieldBufferSize: uLong,
    szComment: *mut c_char,
    commentBufferSize: uLong,
) -> c_int {
    let s: *mut unz_s;
    let mut file_info: unz_file_info = unsafe { mem::zeroed() };
    let mut file_info_internal: unz_file_info_internal = unsafe { mem::zeroed() };
    let mut err: c_int = UNZ_OK;
    let mut uMagic: uLong;
    let mut lSeek: c_int = 0;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        if fseek(
            (*s).file,
            ((*s).pos_in_central_dir + (*s).byte_before_the_zipfile) as c_int,
            SEEK_SET,
        ) != 0
        {
            err = UNZ_ERRNO;
        }

        // we check the magic
        if err == UNZ_OK {
            if unzlocal_getLong((*s).file, addr_of_mut!(uMagic)) != UNZ_OK {
                err = UNZ_ERRNO;
            } else if uMagic != 0x02014b50 {
                err = UNZ_BADZIPFILE;
            }
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.version)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.version_needed)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.flag)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.compression_method)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(file_info.dosDate)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        unzlocal_DosDateToTmuDate(file_info.dosDate, addr_of_mut!(file_info.tmu_date));

        if unzlocal_getLong((*s).file, addr_of_mut!(file_info.crc)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(file_info.compressed_size)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(file_info.uncompressed_size)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.size_filename)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.size_file_extra)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.size_file_comment)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.disk_num_start)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(file_info.internal_fa)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(file_info.external_fa)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(file_info_internal.offset_curfile))
            != UNZ_OK
        {
            err = UNZ_ERRNO;
        }

        lSeek += file_info.size_filename as c_int;
        if (err == UNZ_OK) && !szFileName.is_null() {
            let uSizeRead: uLong;
            if file_info.size_filename < fileNameBufferSize {
                *szFileName.add(file_info.size_filename as usize) = b'\0' as c_char;
                uSizeRead = file_info.size_filename;
            } else {
                uSizeRead = fileNameBufferSize;
            }

            if (file_info.size_filename > 0) && (fileNameBufferSize > 0) {
                if fread(
                    szFileName as *mut c_void,
                    uSizeRead as usize,
                    1,
                    (*s).file,
                ) != 1
                {
                    err = UNZ_ERRNO;
                }
            }
            lSeek -= uSizeRead as c_int;
        }

        if (err == UNZ_OK) && !extraField.is_null() {
            let uSizeRead: uLong;
            if file_info.size_file_extra < extraFieldBufferSize {
                uSizeRead = file_info.size_file_extra;
            } else {
                uSizeRead = extraFieldBufferSize;
            }

            if lSeek != 0 {
                if fseek((*s).file, lSeek, SEEK_CUR) == 0 {
                    lSeek = 0;
                } else {
                    err = UNZ_ERRNO;
                }
            }
            if (file_info.size_file_extra > 0) && (extraFieldBufferSize > 0) {
                if fread(extraField, uSizeRead as usize, 1, (*s).file) != 1 {
                    err = UNZ_ERRNO;
                }
            }
            lSeek += (file_info.size_file_extra - uSizeRead) as c_int;
        } else {
            lSeek += file_info.size_file_extra as c_int;
        }

        if (err == UNZ_OK) && !szComment.is_null() {
            let uSizeRead: uLong;
            if file_info.size_file_comment < commentBufferSize {
                *szComment.add(file_info.size_file_comment as usize) = b'\0' as c_char;
                uSizeRead = file_info.size_file_comment;
            } else {
                uSizeRead = commentBufferSize;
            }

            if lSeek != 0 {
                if fseek((*s).file, lSeek, SEEK_CUR) == 0 {
                    lSeek = 0;
                } else {
                    err = UNZ_ERRNO;
                }
            }
            if (file_info.size_file_comment > 0) && (commentBufferSize > 0) {
                if fread(szComment as *mut c_void, uSizeRead as usize, 1, (*s).file) != 1 {
                    err = UNZ_ERRNO;
                }
            }
            lSeek += (file_info.size_file_comment - uSizeRead) as c_int;
        } else {
            lSeek += file_info.size_file_comment as c_int;
        }

        if (err == UNZ_OK) && !pfile_info.is_null() {
            *pfile_info = file_info;
        }

        if (err == UNZ_OK) && !pfile_info_internal.is_null() {
            *pfile_info_internal = file_info_internal;
        }
    }

    err
}

// Write info about the ZipFile in the *pglobal_info structure.
// No preparation of the structure is needed
// return UNZ_OK if there is no problem.
pub extern "C" fn unzGetCurrentFileInfo(
    file: unzFile,
    pfile_info: *mut unz_file_info,
    szFileName: *mut c_char,
    fileNameBufferSize: uLong,
    extraField: *mut c_void,
    extraFieldBufferSize: uLong,
    szComment: *mut c_char,
    commentBufferSize: uLong,
) -> c_int {
    unzlocal_GetCurrentFileInfoInternal(
        file,
        pfile_info,
        core::ptr::null_mut(),
        szFileName,
        fileNameBufferSize,
        extraField,
        extraFieldBufferSize,
        szComment,
        commentBufferSize,
    )
}

// Set the current file of the zipfile to the first file.
// return UNZ_OK if there is no problem
pub extern "C" fn unzGoToFirstFile(file: unzFile) -> c_int {
    let mut err: c_int = UNZ_OK;
    let s: *mut unz_s;
    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        (*s).pos_in_central_dir = (*s).offset_central_dir;
        (*s).num_file = 0;
        err = unzlocal_GetCurrentFileInfoInternal(
            file,
            addr_of_mut!((*s).cur_file_info),
            addr_of_mut!((*s).cur_file_info_internal),
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        (*s).current_file_ok = if err == UNZ_OK { 1 } else { 0 };
    }
    err
}

// Set the current file of the zipfile to the next file.
// return UNZ_OK if there is no problem
// return UNZ_END_OF_LIST_OF_FILE if the actual file was the latest.
pub extern "C" fn unzGoToNextFile(file: unzFile) -> c_int {
    let s: *mut unz_s;
    let mut err: c_int;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        if (*s).current_file_ok == 0 {
            return UNZ_END_OF_LIST_OF_FILE;
        }
        if (*s).num_file + 1 == (*s).gi.number_entry {
            return UNZ_END_OF_LIST_OF_FILE;
        }

        (*s).pos_in_central_dir = (*s).pos_in_central_dir
            + SIZECENTRALDIRITEM
            + (*s).cur_file_info.size_filename
            + (*s).cur_file_info.size_file_extra
            + (*s).cur_file_info.size_file_comment;
        (*s).num_file += 1;
        err = unzlocal_GetCurrentFileInfoInternal(
            file,
            addr_of_mut!((*s).cur_file_info),
            addr_of_mut!((*s).cur_file_info_internal),
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        (*s).current_file_ok = if err == UNZ_OK { 1 } else { 0 };
    }
    err
}

// Get the position of the info of the current file in the zip.
// return UNZ_OK if there is no problem
pub extern "C" fn unzGetCurrentFileInfoPosition(file: unzFile, pos: *mut uLong) -> c_int {
    let s: *mut unz_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;

        *pos = (*s).pos_in_central_dir;
    }
    UNZ_OK
}

// Set the position of the info of the current file in the zip.
// return UNZ_OK if there is no problem
pub extern "C" fn unzSetCurrentFileInfoPosition(file: unzFile, pos: uLong) -> c_int {
    let s: *mut unz_s;
    let mut err: c_int;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;

        (*s).pos_in_central_dir = pos;
        err = unzlocal_GetCurrentFileInfoInternal(
            file,
            addr_of_mut!((*s).cur_file_info),
            addr_of_mut!((*s).cur_file_info_internal),
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        (*s).current_file_ok = if err == UNZ_OK { 1 } else { 0 };
    }
    UNZ_OK
}

// Try locate the file szFileName in the zipfile.
// For the iCaseSensitivity signification, see unzipStringFileNameCompare
//
// return value :
// UNZ_OK if the file is found. It becomes the current file.
// UNZ_END_OF_LIST_OF_FILE if the file is not found
pub extern "C" fn unzLocateFile(
    file: unzFile,
    szFileName: *const c_char,
    iCaseSensitivity: c_int,
) -> c_int {
    let s: *mut unz_s;
    let mut err: c_int;

    let mut num_fileSaved: uLong;
    let mut pos_in_central_dirSaved: uLong;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        if strlen(szFileName) >= UNZ_MAXFILENAMEINZIP {
            return UNZ_PARAMERROR;
        }

        s = file as *mut unz_s;
        if (*s).current_file_ok == 0 {
            return UNZ_END_OF_LIST_OF_FILE;
        }

        num_fileSaved = (*s).num_file;
        pos_in_central_dirSaved = (*s).pos_in_central_dir;

        err = unzGoToFirstFile(file);

        while err == UNZ_OK {
            let mut szCurrentFileName: [c_char; UNZ_MAXFILENAMEINZIP + 1] =
                [0 as c_char; UNZ_MAXFILENAMEINZIP + 1];
            unzGetCurrentFileInfo(
                file,
                core::ptr::null_mut(),
                szCurrentFileName.as_mut_ptr(),
                (UNZ_MAXFILENAMEINZIP - 1) as uLong,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                0,
            );
            if unzStringFileNameCompare(szCurrentFileName.as_ptr(), szFileName, iCaseSensitivity)
                == 0
            {
                return UNZ_OK;
            }
            err = unzGoToNextFile(file);
        }

        (*s).num_file = num_fileSaved;
        (*s).pos_in_central_dir = pos_in_central_dirSaved;
    }
    err
}

// Read the static header of the current zipfile
// Check the coherency of the static header and info in the end of central
//       directory about this file
// store in *piSizeVar the size of extra info in static header
//       (filename and size of extra field data)
fn unzlocal_CheckCurrentFileCoherencyHeader(
    s: *mut unz_s,
    piSizeVar: *mut uInt,
    poffset_local_extrafield: *mut uLong,
    psize_local_extrafield: *mut uInt,
) -> c_int {
    let mut uMagic: uLong;
    let mut uData: uLong;
    let mut uFlags: uLong;
    let mut size_filename: uLong;
    let mut size_extra_field: uLong;
    let mut err: c_int = UNZ_OK;

    unsafe {
        *piSizeVar = 0;
        *poffset_local_extrafield = 0;
        *psize_local_extrafield = 0;

        if fseek(
            (*s).file,
            ((*s).cur_file_info_internal.offset_curfile + (*s).byte_before_the_zipfile) as c_int,
            SEEK_SET,
        ) != 0
        {
            return UNZ_ERRNO;
        }

        if err == UNZ_OK {
            if unzlocal_getLong((*s).file, addr_of_mut!(uMagic)) != UNZ_OK {
                err = UNZ_ERRNO;
            } else if uMagic != 0x04034b50 {
                err = UNZ_BADZIPFILE;
            }
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(uData)) != UNZ_OK {
            err = UNZ_ERRNO;
        }
        //
        // else if ((err==UNZ_OK) && (uData!=s->cur_file_info.wVersion))
        //	err=UNZ_BADZIPFILE;
        //

        if unzlocal_getShort((*s).file, addr_of_mut!(uFlags)) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(uData)) != UNZ_OK {
            err = UNZ_ERRNO;
        } else if (err == UNZ_OK) && (uData != (*s).cur_file_info.compression_method) {
            err = UNZ_BADZIPFILE;
        }

        if (err == UNZ_OK) && ((*s).cur_file_info.compression_method != 0)
            && ((*s).cur_file_info.compression_method != ZF_DEFLATED)
        {
            err = UNZ_BADZIPFILE;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(uData)) != UNZ_OK {
            /* date/time */
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(uData)) != UNZ_OK {
            /* crc */
            err = UNZ_ERRNO;
        } else if (err == UNZ_OK) && (uData != (*s).cur_file_info.crc)
            && ((uFlags & 8) == 0)
        {
            err = UNZ_BADZIPFILE;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(uData)) != UNZ_OK {
            /* size compr */
            err = UNZ_ERRNO;
        } else if (err == UNZ_OK) && (uData != (*s).cur_file_info.compressed_size)
            && ((uFlags & 8) == 0)
        {
            err = UNZ_BADZIPFILE;
        }

        if unzlocal_getLong((*s).file, addr_of_mut!(uData)) != UNZ_OK {
            /* size uncompr */
            err = UNZ_ERRNO;
        } else if (err == UNZ_OK) && (uData != (*s).cur_file_info.uncompressed_size)
            && ((uFlags & 8) == 0)
        {
            err = UNZ_BADZIPFILE;
        }

        if unzlocal_getShort((*s).file, addr_of_mut!(size_filename)) != UNZ_OK {
            err = UNZ_ERRNO;
        } else if (err == UNZ_OK) && (size_filename != (*s).cur_file_info.size_filename) {
            err = UNZ_BADZIPFILE;
        }

        *piSizeVar += size_filename as uInt;

        if unzlocal_getShort((*s).file, addr_of_mut!(size_extra_field)) != UNZ_OK {
            err = UNZ_ERRNO;
        }
        *poffset_local_extrafield = (*s).cur_file_info_internal.offset_curfile
            + SIZEZIPLOCALHEADER as uLong
            + size_filename;
        *psize_local_extrafield = size_extra_field as uInt;

        *piSizeVar += size_extra_field as uInt;

        err
    }
}

// Open for reading data the current file in the zipfile.
// If there is no error and the file is opened, the return value is UNZ_OK.
pub extern "C" fn unzOpenCurrentFile(file: unzFile) -> c_int {
    let mut err: c_int = UNZ_OK;
    let Store: c_int;
    let mut iSizeVar: uInt = 0;
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;
    let mut offset_local_extrafield: uLong = 0;  /* offset of the static extra field */
    let mut size_local_extrafield: uInt = 0;    /* size of the static extra field */

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        if (*s).current_file_ok == 0 {
            return UNZ_PARAMERROR;
        }

        if !(*s).pfile_in_zip_read.is_null() {
            unzCloseCurrentFile(file);
        }

        if unzlocal_CheckCurrentFileCoherencyHeader(
            s,
            addr_of_mut!(iSizeVar),
            addr_of_mut!(offset_local_extrafield),
            addr_of_mut!(size_local_extrafield),
        ) != UNZ_OK
        {
            return UNZ_BADZIPFILE;
        }

        pfile_in_zip_read_info =
            ALLOC(mem::size_of::<file_in_zip_read_info_s>()) as *mut file_in_zip_read_info_s;

        (*pfile_in_zip_read_info).read_buffer =
            ALLOC(UNZ_BUFSIZE as usize) as *mut c_char;
        (*pfile_in_zip_read_info).offset_local_extrafield = offset_local_extrafield;
        (*pfile_in_zip_read_info).size_local_extrafield = size_local_extrafield;
        (*pfile_in_zip_read_info).pos_local_extrafield = 0;

        (*pfile_in_zip_read_info).stream_initialised = 0;

        if ((*s).cur_file_info.compression_method != 0)
            && ((*s).cur_file_info.compression_method != ZF_DEFLATED)
        {
            err = UNZ_BADZIPFILE;
        }
        Store = if (*s).cur_file_info.compression_method == 0 { 1 } else { 0 };

        (*pfile_in_zip_read_info).crc32_wait = (*s).cur_file_info.crc;
        (*pfile_in_zip_read_info).crc32 = 0;
        (*pfile_in_zip_read_info).compression_method = (*s).cur_file_info.compression_method;
        (*pfile_in_zip_read_info).file = (*s).file;
        (*pfile_in_zip_read_info).byte_before_the_zipfile = (*s).byte_before_the_zipfile;

        (*pfile_in_zip_read_info).stream.total_out = 0;

        if Store == 0 {
            err = inflateInit(
                addr_of_mut!((*pfile_in_zip_read_info).stream),
                Z_SYNC_FLUSH,
                1,
            );
            if err == Z_OK {
                (*pfile_in_zip_read_info).stream_initialised = 1;
            }
            // windowBits is passed < 0 to tell that there is no zlib header.
            //  * Note that in this case inflate *requires* an extra "dummy" byte
            //  * after the compressed stream in order to complete decompression and
            //  * return Z_STREAM_END.
            //  * In unzip, i don't wait absolutely Z_STREAM_END because I known the
            //  * size of both compressed and uncompressed data
        }
        (*pfile_in_zip_read_info).rest_read_compressed = (*s).cur_file_info.compressed_size;
        (*pfile_in_zip_read_info).rest_read_uncompressed =
            (*s).cur_file_info.uncompressed_size;

        (*pfile_in_zip_read_info).pos_in_zipfile = (*s).cur_file_info_internal.offset_curfile
            + SIZEZIPLOCALHEADER as uLong
            + iSizeVar as uLong;

        (*pfile_in_zip_read_info).stream.avail_in = 0;

        (*s).pfile_in_zip_read = pfile_in_zip_read_info;
    }
    UNZ_OK
}

// Read bytes from the current file.
// buf contain buffer where data must be copied
// len the size of buf.
//
// return the number of byte copied if somes bytes are copied
// return 0 if the end of file was reached
// return <0 with error code if there is an error
//   (UNZ_ERRNO for IO error, or zLib error for uncompress error)
pub extern "C" fn unzReadCurrentFile(file: unzFile, buf: *mut c_void, len: uInt) -> c_int {
    let mut err: c_int = UNZ_OK;
    let mut iRead: uInt = 0;
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR;
        }

        if (*pfile_in_zip_read_info).read_buffer.is_null() {
            return UNZ_END_OF_LIST_OF_FILE;
        }
        if len == 0 {
            return 0;
        }

        (*pfile_in_zip_read_info).stream.next_out = buf as *mut Byte;

        (*pfile_in_zip_read_info).stream.avail_out = len;

        if len > (*pfile_in_zip_read_info).rest_read_uncompressed {
            (*pfile_in_zip_read_info).stream.avail_out =
                (*pfile_in_zip_read_info).rest_read_uncompressed as uInt;
        }

        while (*pfile_in_zip_read_info).stream.avail_out > 0 {
            if ((*pfile_in_zip_read_info).stream.avail_in == 0)
                && ((*pfile_in_zip_read_info).rest_read_compressed > 0)
            {
                let mut uReadThis: uInt = UNZ_BUFSIZE;
                if (*pfile_in_zip_read_info).rest_read_compressed < uReadThis {
                    uReadThis = (*pfile_in_zip_read_info).rest_read_compressed as uInt;
                }
                if uReadThis == 0 {
                    return UNZ_EOF;
                }
                if (*s).cur_file_info.compressed_size
                    == (*pfile_in_zip_read_info).rest_read_compressed
                {
                    if fseek(
                        (*pfile_in_zip_read_info).file,
                        ((*pfile_in_zip_read_info).pos_in_zipfile
                            + (*pfile_in_zip_read_info).byte_before_the_zipfile)
                            as c_int,
                        SEEK_SET,
                    ) != 0
                    {
                        return UNZ_ERRNO;
                    }
                }
                if fread(
                    (*pfile_in_zip_read_info).read_buffer as *mut c_void,
                    uReadThis as usize,
                    1,
                    (*pfile_in_zip_read_info).file,
                ) != 1
                {
                    return UNZ_ERRNO;
                }
                (*pfile_in_zip_read_info).pos_in_zipfile += uReadThis as uLong;

                (*pfile_in_zip_read_info).rest_read_compressed -= uReadThis as uLong;

                (*pfile_in_zip_read_info).stream.next_in =
                    (*pfile_in_zip_read_info).read_buffer as *mut Byte;
                (*pfile_in_zip_read_info).stream.avail_in = uReadThis;
            }

            if (*pfile_in_zip_read_info).compression_method == 0 {
                let mut uDoCopy: uInt;
                let mut i: uInt;
                if (*pfile_in_zip_read_info).stream.avail_out
                    < (*pfile_in_zip_read_info).stream.avail_in
                {
                    uDoCopy = (*pfile_in_zip_read_info).stream.avail_out;
                } else {
                    uDoCopy = (*pfile_in_zip_read_info).stream.avail_in;
                }

                i = 0;
                while i < uDoCopy {
                    *(*pfile_in_zip_read_info)
                        .stream
                        .next_out
                        .add(i as usize) = *(*pfile_in_zip_read_info)
                        .stream
                        .next_in
                        .add(i as usize);
                    i += 1;
                }

                (*pfile_in_zip_read_info).crc32 = crc32(
                    (*pfile_in_zip_read_info).crc32,
                    (*pfile_in_zip_read_info).stream.next_out,
                    uDoCopy,
                );
                (*pfile_in_zip_read_info).rest_read_uncompressed -= uDoCopy as uLong;
                (*pfile_in_zip_read_info).stream.avail_in -= uDoCopy;
                (*pfile_in_zip_read_info).stream.avail_out -= uDoCopy;
                (*pfile_in_zip_read_info).stream.next_out =
                    (*pfile_in_zip_read_info).stream.next_out.add(uDoCopy as usize);
                (*pfile_in_zip_read_info).stream.next_in =
                    (*pfile_in_zip_read_info).stream.next_in.add(uDoCopy as usize);
                (*pfile_in_zip_read_info).stream.total_out += uDoCopy as uLong;
                iRead += uDoCopy;
            } else {
                let mut uTotalOutBefore: uLong;
                let mut uTotalOutAfter: uLong;
                let bufBefore: *const Byte;
                let uOutThis: uLong;

                uTotalOutBefore = (*pfile_in_zip_read_info).stream.total_out;
                bufBefore = (*pfile_in_zip_read_info).stream.next_out;

                //
                // if ((pfile_in_zip_read_info->rest_read_uncompressed ==
                //          pfile_in_zip_read_info->stream.avail_out) &&
                //	(pfile_in_zip_read_info->rest_read_compressed == 0))
                //	flush = Z_FINISH;
                //
                err = inflate(addr_of_mut!((*pfile_in_zip_read_info).stream));

                uTotalOutAfter = (*pfile_in_zip_read_info).stream.total_out;
                uOutThis = uTotalOutAfter - uTotalOutBefore;

                (*pfile_in_zip_read_info).crc32 =
                    crc32((*pfile_in_zip_read_info).crc32, bufBefore, uOutThis as uInt);

                (*pfile_in_zip_read_info).rest_read_uncompressed -= uOutThis;

                iRead += (uTotalOutAfter - uTotalOutBefore) as uInt;

                if err == Z_STREAM_END {
                    return if iRead == 0 { UNZ_EOF } else { iRead as c_int };
                }
                if err != Z_OK {
                    break;
                }
            }
        }

        if err == Z_OK {
            iRead as c_int
        } else {
            err
        }
    }
}

// Give the current position in uncompressed data
pub extern "C" fn unztell(file: unzFile) -> c_int {
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR;
        }

        (*pfile_in_zip_read_info).stream.total_out as c_int
    }
}

// return 1 if the end of file was reached, 0 elsewhere
pub extern "C" fn unzeof(file: unzFile) -> c_int {
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR;
        }

        if (*pfile_in_zip_read_info).rest_read_uncompressed == 0 {
            1
        } else {
            0
        }
    }
}

// Read extra field from the current file (opened by unzOpenCurrentFile)
// This is the static-header version of the extra field (sometimes, there is
//   more info in the static-header version than in the central-header)
//
// if buf==NULL, it return the size of the static extra field that can be read
//
// if buf!=NULL, len is the size of the buffer, the extra header is copied in
//	buf.
// the return value is the number of bytes copied in buf, or (if <0)
//	the error code
pub extern "C" fn unzGetLocalExtrafield(file: unzFile, buf: *mut c_void, len: uInt) -> c_int {
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;
    let mut read_now: uInt;
    let size_to_read: uLong;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR;
        }

        size_to_read = (*pfile_in_zip_read_info).size_local_extrafield as uLong
            - (*pfile_in_zip_read_info).pos_local_extrafield;

        if buf.is_null() {
            return size_to_read as c_int;
        }

        if len > size_to_read as uInt {
            read_now = size_to_read as uInt;
        } else {
            read_now = len;
        }

        if read_now == 0 {
            return 0;
        }

        if fseek(
            (*pfile_in_zip_read_info).file,
            ((*pfile_in_zip_read_info).offset_local_extrafield
                + (*pfile_in_zip_read_info).pos_local_extrafield)
                as c_int,
            SEEK_SET,
        ) != 0
        {
            return UNZ_ERRNO;
        }

        if fread(buf, size_to_read as usize, 1, (*pfile_in_zip_read_info).file) != 1 {
            return UNZ_ERRNO;
        }

        read_now as c_int
    }
}

// Close the file in zip opened with unzipOpenCurrentFile
// Return UNZ_CRCERROR if all the file was read but the CRC is not good
pub extern "C" fn unzCloseCurrentFile(file: unzFile) -> c_int {
    let mut err: c_int = UNZ_OK;

    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;
        pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR;
        }

        if (*pfile_in_zip_read_info).rest_read_uncompressed == 0 {
            if (*pfile_in_zip_read_info).crc32 != (*pfile_in_zip_read_info).crc32_wait {
                err = UNZ_CRCERROR;
            }
        }

        TRYFREE((*pfile_in_zip_read_info).read_buffer as *mut c_void);
        (*pfile_in_zip_read_info).read_buffer = core::ptr::null_mut();
        if (*pfile_in_zip_read_info).stream_initialised != 0 {
            inflateEnd(addr_of_mut!((*pfile_in_zip_read_info).stream));
        }

        (*pfile_in_zip_read_info).stream_initialised = 0;
        TRYFREE(pfile_in_zip_read_info as *mut c_void);

        (*s).pfile_in_zip_read = core::ptr::null_mut();
    }

    err
}

// Get the global comment string of the ZipFile, in the szComment buffer.
// uSizeBuf is the size of the szComment buffer.
// return the number of byte copied or an error code <0
pub extern "C" fn unzGetGlobalComment(
    file: unzFile,
    szComment: *mut c_char,
    uSizeBuf: uLong,
) -> c_int {
    let s: *mut unz_s;
    let mut uReadThis: uLong;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        s = file as *mut unz_s;

        uReadThis = uSizeBuf;
        if uReadThis > (*s).gi.size_comment {
            uReadThis = (*s).gi.size_comment;
        }

        if fseek((*s).file, ((*s).central_pos + 22) as c_int, SEEK_SET) != 0 {
            return UNZ_ERRNO;
        }

        if uReadThis > 0 {
            *szComment = b'\0' as c_char;
            if fread(szComment as *mut c_void, uReadThis as usize, 1, (*s).file) != 1 {
                return UNZ_ERRNO;
            }
        }

        if !szComment.is_null() && (uSizeBuf > (*s).gi.size_comment) {
            *szComment.add((*s).gi.size_comment as usize) = b'\0' as c_char;
        }
        uReadThis as c_int
    }
}

// end
