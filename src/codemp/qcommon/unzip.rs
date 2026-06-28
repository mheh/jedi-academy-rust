#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

/*****************************************************************************
 * name:		unzip.c
 *
 * desc:		IO on .zip files using portions of zlib
 *
 *****************************************************************************/

use core::ffi::{c_char, c_int, c_void};

// Type declarations
/* Type declarations */

type Byte = u8;  // 8 bits
type uInt = c_int;  // 16 bits or more (using c_int for 32-bit consistency)
type uLong = u32;  // 32 bits or more
type voidp = *mut Byte;

const SEEK_SET: i32 = 0;  // Seek from beginning of file.
const SEEK_CUR: i32 = 1;  // Seek from current position.
const SEEK_END: i32 = 2;  // Set file pointer to EOF plus "offset"

type gzFile = voidp;

type ZIP_FILE = c_void;  // FILE type

// Macros from the C code
const UNZ_BUFSIZE: uLong = 65536;
const UNZ_MAXFILENAMEINZIP: uLong = 256;

const SIZECENTRALDIRITEM: uLong = 0x2e;
const SIZEZIPLOCALHEADER: uLong = 0x1e;

const BUFREADCOMMENT: uLong = 0x400;

const CASESENSITIVITYDEFAULT_NO: i32 = 1;
const CASESENSITIVITYDEFAULTVALUE: i32 = 2;

// Constants from unzip.h
const UNZ_OK: c_int = 0;
const UNZ_END_OF_LIST_OF_FILE: c_int = -100;
const UNZ_EOF: c_int = 0;
const UNZ_PARAMERROR: c_int = -102;
const UNZ_BADZIPFILE: c_int = -103;
const UNZ_INTERNALERROR: c_int = -104;
const UNZ_CRCERROR: c_int = -105;

const ZF_DEFLATED: c_int = 8;
const Z_OK: c_int = 0;
const Z_STREAM_END: c_int = 1;
const Z_DATA_ERROR: c_int = -3;
const Z_ERRNO: c_int = (-1);
const Z_SYNC_FLUSH: c_int = 0;

// UNZ_ERRNO maps to Z_DATA_ERROR
const UNZ_ERRNO: c_int = Z_DATA_ERROR;

// Structures from unzip.h

/* tm_unz contain date/time info */
#[repr(C)]
pub struct tm_unz {
    pub tm_sec: u32,   // seconds after the minute - [0,59]
    pub tm_min: u32,   // minutes after the hour - [0,59]
    pub tm_hour: u32,  // hours since midnight - [0,23]
    pub tm_mday: u32,  // day of the month - [1,31]
    pub tm_mon: u32,   // months since January - [0,11]
    pub tm_year: u32,  // years - [1980..2044]
}

/* unz_global_info structure contain global data about the ZIPfile
   These data comes from the end of central dir */
#[repr(C)]
pub struct unz_global_info {
    pub number_entry: uLong,    // total number of entries in the central dir on this disk
    pub size_comment: uLong,    // size of the global comment of the zipfile
}

/* unz_file_info contain information about a file in the zipfile */
#[repr(C)]
pub struct unz_file_info {
    pub version: uLong,              // version made by                 2 unsigned chars
    pub version_needed: uLong,       // version needed to extract       2 unsigned chars
    pub flag: uLong,                 // general purpose bit flag        2 unsigned chars
    pub compression_method: uLong,   // compression method              2 unsigned chars
    pub dosDate: uLong,              // last mod file date in Dos fmt   4 unsigned chars
    pub crc: uLong,                  // crc-32                          4 unsigned chars
    pub compressed_size: uLong,      // compressed size                 4 unsigned chars
    pub uncompressed_size: uLong,    // uncompressed size               4 unsigned chars
    pub size_filename: uLong,        // filename length                 2 unsigned chars
    pub size_file_extra: uLong,      // extra field length              2 unsigned chars
    pub size_file_comment: uLong,    // file comment length             2 unsigned chars
    pub disk_num_start: uLong,       // disk number start               2 unsigned chars
    pub internal_fa: uLong,          // internal file attributes        2 unsigned chars
    pub external_fa: uLong,          // external file attributes        4 unsigned chars
    pub tmu_date: tm_unz,
}

/* unz_file_info_interntal contain internal info about a file in zipfile*/
#[repr(C)]
pub struct unz_file_info_internal {
    pub offset_curfile: uLong, // relative offset of static header 4 unsigned chars
}

// z_stream structure - simplified for our purposes
#[repr(C)]
pub struct z_stream {
    pub next_in: *mut Byte,    // next input byte
    pub avail_in: uInt,        // number of bytes available at next_in
    pub total_in: u32,         // total number of input bytes read so far
    pub next_out: *mut Byte,   // next output byte will go here
    pub avail_out: uInt,       // remaining free space at next_out
    pub total_out: u32,        // total number of bytes output so far
    pub msg: *mut c_char,      // last error message, NULL if no error
    pub state: *mut c_void,    // not visible by applications
    pub zalloc: *mut c_void,   // used to allocate the internal state
    pub zfree: *mut c_void,    // used to free the internal state
    pub opaque: *mut c_void,   // private data object passed to zalloc and zfree
    pub data_type: c_int,      // best guess about the data type: binary or text
    pub adler: u32,            // adler-32 or crc-32 value of the uncompressed data
    pub reserved: u32,         // reserved for future use
}

/* file_in_zip_read_info_s contain internal information about a file in zipfile,
    when reading and decompress it */
#[repr(C)]
pub struct file_in_zip_read_info_s {
    pub read_buffer: *mut c_char,     // internal buffer for compressed data
    pub stream: z_stream,             // zLib stream structure for inflate
    pub pos_in_zipfile: uLong,        // position in unsigned char on the zipfile, for fseek
    pub stream_initialised: uLong,    // flag set if stream structure is initialised
    pub offset_local_extrafield: uLong, // offset of the static extra field
    pub size_local_extrafield: uInt,  // size of the static extra field
    pub pos_local_extrafield: uLong,  // position in the static extra field in read
    pub crc32: uLong,                 // crc32 of all data uncompressed
    pub crc32_wait: uLong,            // crc32 we must obtain after decompress all
    pub rest_read_compressed: uLong,  // number of unsigned char to be decompressed
    pub rest_read_uncompressed: uLong, // number of unsigned char to be obtained after decomp
    pub file: *mut ZIP_FILE,          // io structore of the zipfile
    pub compression_method: uLong,    // compression method (0==store)
    pub byte_before_the_zipfile: uLong, // unsigned char before the zipfile, (>0 for sfx)
}

/* unz_s contain internal information about the zipfile
*/
#[repr(C)]
pub struct unz_s {
    pub file: *mut ZIP_FILE,        // io structore of the zipfile
    pub gi: unz_global_info,        // public global information
    pub byte_before_the_zipfile: uLong, // unsigned char before the zipfile, (>0 for sfx)
    pub num_file: uLong,            // number of the current file in the zipfile
    pub pos_in_central_dir: uLong,  // pos of the current file in the central dir
    pub current_file_ok: uLong,     // flag about the usability of the current file
    pub central_pos: uLong,         // position of the beginning of the central dir
    pub size_central_dir: uLong,    // size of the central directory
    pub offset_central_dir: uLong,  // offset of start of central directory with respect to the starting disk number
    pub cur_file_info: unz_file_info, // public info about the current file in zip
    pub cur_file_info_internal: unz_file_info_internal, // private info about it
    pub pfile_in_zip_read: *mut file_in_zip_read_info_s, // structure about the current file if we are decompressing it
    pub tmpFile: *mut u8,
    pub tmpPos: c_int,
    pub tmpSize: c_int,
}

pub type unzFile = *mut c_void;

// Extern declarations for C/zlib functions and memory functions
extern "C" {
    fn ZIP_fopen(path: *const c_char, mode: *const c_char) -> *mut ZIP_FILE;
    fn ZIP_fclose(file: *mut ZIP_FILE) -> c_int;
    fn ZIP_fseek(file: *mut ZIP_FILE, offset: i32, whence: i32) -> c_int;
    fn ZIP_fread(buf: *mut c_void, size: usize, nmemb: usize, file: *mut ZIP_FILE) -> usize;
    fn ZIP_ftell(file: *mut ZIP_FILE) -> i32;

    fn Z_Malloc(size: uLong, tag: c_int, clear: i32) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    fn LittleShort(x: i16) -> uLong;
    fn LittleLong(x: i32) -> uLong;

    fn inflateInit(
        strm: *mut z_stream,
        flush: c_int,
        dummy: c_int,
    ) -> c_int;
    fn inflate(strm: *mut z_stream) -> c_int;
    fn inflateEnd(strm: *mut z_stream) -> c_int;

    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, size: uLong);
    fn strlen(s: *const c_char) -> uLong;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// Macros translated to functions or constants

/* ===========================================================================
   Reads a long in LSB order from the given gz_stream. Sets
*/
unsafe fn unzlocal_getShort(fin: *mut ZIP_FILE, pX: *mut uLong) -> c_int {
    let mut v: i16 = 0;

    ZIP_fread(
        &mut v as *mut _ as *mut c_void,
        core::mem::size_of_val(&v),
        1,
        fin,
    );

    *pX = LittleShort(v);
    return UNZ_OK;
}

unsafe fn unzlocal_getLong(fin: *mut ZIP_FILE, pX: *mut uLong) -> c_int {
    let mut v: i32 = 0;

    ZIP_fread(
        &mut v as *mut _ as *mut c_void,
        core::mem::size_of_val(&v),
        1,
        fin,
    );

    *pX = LittleLong(v);
    return UNZ_OK;
}

/* My own strcmpi / strcasecmp */
unsafe fn strcmpcasenosensitive_internal(fileName1: *const c_char, fileName2: *const c_char) -> c_int {
    let mut p1 = fileName1;
    let mut p2 = fileName2;

    loop {
        let mut c1 = *p1 as c_int;
        let mut c2 = *p2 as c_int;
        p1 = p1.add(1);
        p2 = p2.add(1);

        if (c1 >= 'a' as c_int) && (c1 <= 'z' as c_int) {
            c1 -= 0x20;
        }
        if (c2 >= 'a' as c_int) && (c2 <= 'z' as c_int) {
            c2 -= 0x20;
        }

        if c1 == '\0' as c_int {
            return if c2 == '\0' as c_int { 0 } else { -1 };
        }
        if c2 == '\0' as c_int {
            return 1;
        }
        if c1 < c2 {
            return -1;
        }
        if c1 > c2 {
            return 1;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn unzStringFileNameCompare(
    fileName1: *const c_char,
    fileName2: *const c_char,
    mut iCaseSensitivity: c_int,
) -> c_int {
    if iCaseSensitivity == 0 {
        iCaseSensitivity = CASESENSITIVITYDEFAULTVALUE;
    }

    if iCaseSensitivity == 1 {
        return strcmp(fileName1, fileName2);
    }

    return strcmpcasenosensitive_internal(fileName1, fileName2);
}

/*
  Locate the Central directory of a zipfile (at the end, just before
    the global comment)
*/
unsafe fn unzlocal_SearchCentralDir(fin: *mut ZIP_FILE) -> uLong {
    let mut buf: *mut u8;
    let mut uSizeFile: uLong;
    let mut uBackRead: uLong;
    let mut uMaxBack: uLong = 0xffff; // maximum size of global comment
    let mut uPosFound: uLong = 0;

    if ZIP_fseek(fin, 0, SEEK_END) != 0 {
        return 0;
    }

    uSizeFile = ZIP_ftell(fin) as uLong;

    if uMaxBack > uSizeFile {
        uMaxBack = uSizeFile;
    }

    buf = Z_Malloc(BUFREADCOMMENT + 4, 0, 0 as c_int) as *mut u8;

    uBackRead = 4;
    while uBackRead < uMaxBack {
        let mut uReadSize: uLong;
        let mut uReadPos: uLong;
        let mut i: i32;

        if uBackRead + BUFREADCOMMENT > uMaxBack {
            uBackRead = uMaxBack;
        } else {
            uBackRead += BUFREADCOMMENT;
        }

        uReadPos = uSizeFile - uBackRead;

        uReadSize = if (BUFREADCOMMENT + 4) < (uSizeFile - uReadPos) {
            BUFREADCOMMENT + 4
        } else {
            uSizeFile - uReadPos
        };

        if ZIP_fseek(fin, uReadPos as i32, SEEK_SET) != 0 {
            break;
        }

        if ZIP_fread(buf as *mut c_void, uReadSize as usize, 1, fin) != 1 {
            break;
        }

        i = (uReadSize as i32) - 3;
        while i > 0 {
            i -= 1;
            if (*(buf.add(i as usize)) == 0x50)
                && (*(buf.add(i as usize + 1)) == 0x4b)
                && (*(buf.add(i as usize + 2)) == 0x05)
                && (*(buf.add(i as usize + 3)) == 0x06)
            {
                uPosFound = uReadPos + (i as uLong);
                break;
            }
        }

        if uPosFound != 0 {
            break;
        }
    }

    if !buf.is_null() {
        Z_Free(buf as *mut c_void);
    }

    return uPosFound;
}

#[no_mangle]
pub unsafe extern "C" fn unzReOpen(path: *const c_char, file: unzFile) -> unzFile {
    let mut s: *mut unz_s;
    let mut fin: *mut ZIP_FILE;

    fin = ZIP_fopen(path, b"rb\0".as_ptr() as *const c_char);
    if fin.is_null() {
        return core::ptr::null_mut();
    }

    s = Z_Malloc(core::mem::size_of::<unz_s>() as uLong, 0, 0) as *mut unz_s;
    Com_Memcpy(
        s as *mut c_void,
        file as *const c_void,
        core::mem::size_of::<unz_s>() as uLong,
    );

    (*s).file = fin;
    return s as unzFile;
}

/*
  Open a Zip file. path contain the full pathname (by example,
     on a Windows NT computer "c:\\test\\zlib109.zip" or on an Unix computer
     "zlib/zlib109.zip".
     If the zipfile cannot be opened (file don't exist or in not valid), the
       return value is NULL.
     Else, the return value is a unzFile Handle, usable with other function
       of this unzip package.
*/
#[no_mangle]
pub unsafe extern "C" fn unzOpen(path: *const c_char) -> unzFile {
    let mut us: unz_s = core::mem::zeroed();
    let mut s: *mut unz_s;
    let mut central_pos: uLong;
    let mut uL: uLong;
    let mut fin: *mut ZIP_FILE;

    let mut number_disk: uLong; // number of the current dist, used for spaning ZIP, unsupported, always 0
    let mut number_disk_with_CD: uLong; // number the the disk with central dir, used for spaning ZIP, unsupported, always 0
    let mut number_entry_CD: uLong; // total number of entries in the central dir (same than number_entry on nospan)

    let mut err: c_int = UNZ_OK;

    fin = ZIP_fopen(path, b"rb\0".as_ptr() as *const c_char);
    if fin.is_null() {
        return core::ptr::null_mut();
    }

    central_pos = unzlocal_SearchCentralDir(fin);
    if central_pos == 0 {
        err = UNZ_ERRNO;
    }

    if ZIP_fseek(fin, central_pos as i32, SEEK_SET) != 0 {
        err = UNZ_ERRNO;
    }

    // the signature, already checked
    if unzlocal_getLong(fin, &mut uL) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    // number of this disk
    if unzlocal_getShort(fin, &mut number_disk) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    // number of the disk with the start of the central directory
    if unzlocal_getShort(fin, &mut number_disk_with_CD) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    // total number of entries in the central dir on this disk
    if unzlocal_getShort(fin, &mut us.gi.number_entry) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    // total number of entries in the central dir
    if unzlocal_getShort(fin, &mut number_entry_CD) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if (number_entry_CD != us.gi.number_entry) || (number_disk_with_CD != 0) || (number_disk != 0) {
        err = UNZ_BADZIPFILE;
    }

    // size of the central directory
    if unzlocal_getLong(fin, &mut us.size_central_dir) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    // offset of start of central directory with respect to the starting disk number
    if unzlocal_getLong(fin, &mut us.offset_central_dir) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    // zipfile comment length
    if unzlocal_getShort(fin, &mut us.gi.size_comment) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if (central_pos < us.offset_central_dir + us.size_central_dir) && (err == UNZ_OK) {
        err = UNZ_BADZIPFILE;
    }

    if err != UNZ_OK {
        ZIP_fclose(fin);
        return core::ptr::null_mut();
    }

    us.file = fin;
    us.byte_before_the_zipfile = central_pos - (us.offset_central_dir + us.size_central_dir);
    us.central_pos = central_pos;
    us.pfile_in_zip_read = core::ptr::null_mut();

    s = Z_Malloc(core::mem::size_of::<unz_s>() as uLong, 0, 0) as *mut unz_s;
    *s = us;
    return s as unzFile;
}

/*
  Close a ZipFile opened with unzipOpen.
  If there is files inside the .Zip opened with unzipOpenCurrentFile (see later),
    these files MUST be closed with unzipCloseCurrentFile before call unzipClose.
  return UNZ_OK if there is no problem. */
#[no_mangle]
pub unsafe extern "C" fn unzClose(file: unzFile) -> c_int {
    let mut s: *mut unz_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;

    if !(*s).pfile_in_zip_read.is_null() {
        unzCloseCurrentFile(file);
    }

    ZIP_fclose((*s).file);
    if !s.is_null() {
        Z_Free(s as *mut c_void);
    }
    return UNZ_OK;
}

/*
  Write info about the ZipFile in the *pglobal_info structure.
  No preparation of the structure is needed
  return UNZ_OK if there is no problem. */
#[no_mangle]
pub unsafe extern "C" fn unzGetGlobalInfo(
    file: unzFile,
    pglobal_info: *mut unz_global_info,
) -> c_int {
    let s: *mut unz_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;
    *pglobal_info = (*s).gi;
    return UNZ_OK;
}

/*
   Translate date/time from Dos format to tm_unz (readable more easilty)
*/
unsafe fn unzlocal_DosDateToTmuDate(ulDosDate: uLong, ptm: *mut tm_unz) {
    let mut uDate: uLong;

    uDate = ulDosDate >> 16;
    (*ptm).tm_mday = (uDate & 0x1f) as u32;
    (*ptm).tm_mon = ((((uDate) & 0x1E0) / 0x20) - 1) as u32;
    (*ptm).tm_year = (((uDate & 0x0FE00) / 0x0200) + 1980) as u32;

    (*ptm).tm_hour = ((ulDosDate & 0xF800) / 0x800) as u32;
    (*ptm).tm_min = ((ulDosDate & 0x7E0) / 0x20) as u32;
    (*ptm).tm_sec = (2 * (ulDosDate & 0x1f)) as u32;
}

/*
  Get Info about the current file in the zipfile, with internal only info
*/
unsafe fn unzlocal_GetCurrentFileInfoInternal(
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
    let mut file_info: unz_file_info = core::mem::zeroed();
    let mut file_info_internal: unz_file_info_internal = core::mem::zeroed();
    let mut err: c_int = UNZ_OK;
    let mut uMagic: uLong;
    let mut lSeek: i32 = 0;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;

    if ZIP_fseek(
        (*s).file,
        ((*s).pos_in_central_dir + (*s).byte_before_the_zipfile) as i32,
        SEEK_SET,
    ) != 0
    {
        err = UNZ_ERRNO;
    }

    // we check the magic
    if err == UNZ_OK {
        if unzlocal_getLong((*s).file, &mut uMagic) != UNZ_OK {
            err = UNZ_ERRNO;
        } else if uMagic != 0x02014b50 {
            err = UNZ_BADZIPFILE;
        }
    }

    if unzlocal_getShort((*s).file, &mut file_info.version) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut file_info.version_needed) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut file_info.flag) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut file_info.compression_method) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getLong((*s).file, &mut file_info.dosDate) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    unzlocal_DosDateToTmuDate(file_info.dosDate, &mut file_info.tmu_date);

    if unzlocal_getLong((*s).file, &mut file_info.crc) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getLong((*s).file, &mut file_info.compressed_size) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getLong((*s).file, &mut file_info.uncompressed_size) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut file_info.size_filename) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut file_info.size_file_extra) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut file_info.size_file_comment) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut file_info.disk_num_start) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut file_info.internal_fa) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getLong((*s).file, &mut file_info.external_fa) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getLong((*s).file, &mut file_info_internal.offset_curfile) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    lSeek += file_info.size_filename as i32;

    if (err == UNZ_OK) && !szFileName.is_null() {
        let mut uSizeRead: uLong;

        if file_info.size_filename < fileNameBufferSize {
            *(szFileName.add(file_info.size_filename as usize)) = '\0' as c_char;
            uSizeRead = file_info.size_filename;
        } else {
            uSizeRead = fileNameBufferSize;
        }

        if (file_info.size_filename > 0) && (fileNameBufferSize > 0) {
            if ZIP_fread(szFileName as *mut c_void, uSizeRead as usize, 1, (*s).file) != 1 {
                err = UNZ_ERRNO;
            }
        }
        lSeek -= uSizeRead as i32;
    }

    if (err == UNZ_OK) && !extraField.is_null() {
        let mut uSizeRead: uLong;

        if file_info.size_file_extra < extraFieldBufferSize {
            uSizeRead = file_info.size_file_extra;
        } else {
            uSizeRead = extraFieldBufferSize;
        }

        if lSeek != 0 {
            if ZIP_fseek((*s).file, lSeek, SEEK_CUR) == 0 {
                lSeek = 0;
            } else {
                err = UNZ_ERRNO;
            }
        }

        if (file_info.size_file_extra > 0) && (extraFieldBufferSize > 0) {
            if ZIP_fread(extraField, uSizeRead as usize, 1, (*s).file) != 1 {
                err = UNZ_ERRNO;
            }
        }
        lSeek += file_info.size_file_extra as i32 - uSizeRead as i32;
    } else {
        lSeek += file_info.size_file_extra as i32;
    }

    if (err == UNZ_OK) && !szComment.is_null() {
        let mut uSizeRead: uLong;

        if file_info.size_file_comment < commentBufferSize {
            *(szComment.add(file_info.size_file_comment as usize)) = '\0' as c_char;
            uSizeRead = file_info.size_file_comment;
        } else {
            uSizeRead = commentBufferSize;
        }

        if lSeek != 0 {
            if ZIP_fseek((*s).file, lSeek, SEEK_CUR) == 0 {
                lSeek = 0;
            } else {
                err = UNZ_ERRNO;
            }
        }

        if (file_info.size_file_comment > 0) && (commentBufferSize > 0) {
            if ZIP_fread(szComment as *mut c_void, uSizeRead as usize, 1, (*s).file) != 1 {
                err = UNZ_ERRNO;
            }
        }
        lSeek += file_info.size_file_comment as i32 - uSizeRead as i32;
    } else {
        lSeek += file_info.size_file_comment as i32;
    }

    if (err == UNZ_OK) && !pfile_info.is_null() {
        *pfile_info = file_info;
    }

    if (err == UNZ_OK) && !pfile_info_internal.is_null() {
        *pfile_info_internal = file_info_internal;
    }

    return err;
}

/*
  Write info about the ZipFile in the *pglobal_info structure.
  No preparation of the structure is needed
  return UNZ_OK if there is no problem.
*/
#[no_mangle]
pub unsafe extern "C" fn unzGetCurrentFileInfo(
    file: unzFile,
    pfile_info: *mut unz_file_info,
    szFileName: *mut c_char,
    fileNameBufferSize: uLong,
    extraField: *mut c_void,
    extraFieldBufferSize: uLong,
    szComment: *mut c_char,
    commentBufferSize: uLong,
) -> c_int {
    return unzlocal_GetCurrentFileInfoInternal(
        file,
        pfile_info,
        core::ptr::null_mut(),
        szFileName,
        fileNameBufferSize,
        extraField,
        extraFieldBufferSize,
        szComment,
        commentBufferSize,
    );
}

/*
  Set the current file of the zipfile to the first file.
  return UNZ_OK if there is no problem
*/
#[no_mangle]
pub unsafe extern "C" fn unzGoToFirstFile(file: unzFile) -> c_int {
    let mut err: c_int = UNZ_OK;
    let s: *mut unz_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;
    (*s).pos_in_central_dir = (*s).offset_central_dir;
    (*s).num_file = 0;
    err = unzlocal_GetCurrentFileInfoInternal(
        file,
        &mut (*s).cur_file_info,
        &mut (*s).cur_file_info_internal,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );
    (*s).current_file_ok = if err == UNZ_OK { 1 } else { 0 };
    return err;
}

/*
  Set the current file of the zipfile to the next file.
  return UNZ_OK if there is no problem
  return UNZ_END_OF_LIST_OF_FILE if the actual file was the latest.
*/
#[no_mangle]
pub unsafe extern "C" fn unzGoToNextFile(file: unzFile) -> c_int {
    let s: *mut unz_s;
    let mut err: c_int;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
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
    (*s).num_file = (*s).num_file + 1;

    err = unzlocal_GetCurrentFileInfoInternal(
        file,
        &mut (*s).cur_file_info,
        &mut (*s).cur_file_info_internal,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );
    (*s).current_file_ok = if err == UNZ_OK { 1 } else { 0 };
    return err;
}

/*
  Get the position of the info of the current file in the zip.
  return UNZ_OK if there is no problem
*/
#[no_mangle]
pub unsafe extern "C" fn unzGetCurrentFileInfoPosition(file: unzFile, pos: *mut uLong) -> c_int {
    let s: *mut unz_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;

    *pos = (*s).pos_in_central_dir;
    return UNZ_OK;
}

/*
  Set the position of the info of the current file in the zip.
  return UNZ_OK if there is no problem
*/
#[no_mangle]
pub unsafe extern "C" fn unzSetCurrentFileInfoPosition(file: unzFile, pos: uLong) -> c_int {
    let s: *mut unz_s;
    let mut err: c_int;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;

    (*s).pos_in_central_dir = pos;
    err = unzlocal_GetCurrentFileInfoInternal(
        file,
        &mut (*s).cur_file_info,
        &mut (*s).cur_file_info_internal,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );
    (*s).current_file_ok = if err == UNZ_OK { 1 } else { 0 };
    return UNZ_OK;
}

/*
  Try locate the file szFileName in the zipfile.
  For the iCaseSensitivity signification, see unzipStringFileNameCompare

  return value :
  UNZ_OK if the file is found. It becomes the current file.
  UNZ_END_OF_LIST_OF_FILE if the file is not found
*/
#[no_mangle]
pub unsafe extern "C" fn unzLocateFile(
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
        let mut szCurrentFileName: [c_char; UNZ_MAXFILENAMEINZIP as usize + 1] = [0; 257];

        unzGetCurrentFileInfo(
            file,
            core::ptr::null_mut(),
            szCurrentFileName.as_mut_ptr(),
            (core::mem::size_of_val(&szCurrentFileName) - 1) as uLong,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );

        if unzStringFileNameCompare(szCurrentFileName.as_ptr(), szFileName, iCaseSensitivity) == 0 {
            return UNZ_OK;
        }
        err = unzGoToNextFile(file);
    }

    (*s).num_file = num_fileSaved;
    (*s).pos_in_central_dir = pos_in_central_dirSaved;
    return err;
}

/*
  Read the static header of the current zipfile
  Check the coherency of the static header and info in the end of central
        directory about this file
  store in *piSizeVar the size of extra info in static header
        (filename and size of extra field data)
*/
unsafe fn unzlocal_CheckCurrentFileCoherencyHeader(
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

    *piSizeVar = 0;
    *poffset_local_extrafield = 0;
    *psize_local_extrafield = 0;

    if ZIP_fseek(
        (*s).file,
        ((*s).cur_file_info_internal.offset_curfile + (*s).byte_before_the_zipfile) as i32,
        SEEK_SET,
    ) != 0
    {
        return UNZ_ERRNO;
    }

    if err == UNZ_OK {
        if unzlocal_getLong((*s).file, &mut uMagic) != UNZ_OK {
            err = UNZ_ERRNO;
        } else if uMagic != 0x04034b50 {
            err = UNZ_BADZIPFILE;
        }
    }

    if unzlocal_getShort((*s).file, &mut uData) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut uFlags) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    if unzlocal_getShort((*s).file, &mut uData) != UNZ_OK {
        err = UNZ_ERRNO;
    } else if (err == UNZ_OK) && (uData != (*s).cur_file_info.compression_method) {
        err = UNZ_BADZIPFILE;
    }

    if (err == UNZ_OK) && ((*s).cur_file_info.compression_method != 0)
        && ((*s).cur_file_info.compression_method != ZF_DEFLATED as uLong)
    {
        err = UNZ_BADZIPFILE;
    }

    if unzlocal_getLong((*s).file, &mut uData) != UNZ_OK {
        err = UNZ_ERRNO;
    } // date/time

    if unzlocal_getLong((*s).file, &mut uData) != UNZ_OK {
        err = UNZ_ERRNO;
    } // crc
    else if (err == UNZ_OK) && (uData != (*s).cur_file_info.crc) && ((uFlags & 8) == 0) {
        err = UNZ_BADZIPFILE;
    }

    if unzlocal_getLong((*s).file, &mut uData) != UNZ_OK {
        err = UNZ_ERRNO;
    } // size compr
    else if (err == UNZ_OK) && (uData != (*s).cur_file_info.compressed_size) && ((uFlags & 8) == 0) {
        err = UNZ_BADZIPFILE;
    }

    if unzlocal_getLong((*s).file, &mut uData) != UNZ_OK {
        err = UNZ_ERRNO;
    } // size uncompr
    else if (err == UNZ_OK) && (uData != (*s).cur_file_info.uncompressed_size) && ((uFlags & 8) == 0) {
        err = UNZ_BADZIPFILE;
    }

    if unzlocal_getShort((*s).file, &mut size_filename) != UNZ_OK {
        err = UNZ_ERRNO;
    } else if (err == UNZ_OK) && (size_filename != (*s).cur_file_info.size_filename) {
        err = UNZ_BADZIPFILE;
    }

    *piSizeVar += size_filename as uInt;

    if unzlocal_getShort((*s).file, &mut size_extra_field) != UNZ_OK {
        err = UNZ_ERRNO;
    }

    *poffset_local_extrafield =
        (*s).cur_file_info_internal.offset_curfile + SIZEZIPLOCALHEADER + size_filename;
    *psize_local_extrafield = size_extra_field as uInt;

    *piSizeVar += size_extra_field as uInt;

    return err;
}

/*
  Open for reading data the current file in the zipfile.
  If there is no error and the file is opened, the return value is UNZ_OK.
*/
#[no_mangle]
pub unsafe extern "C" fn unzOpenCurrentFile(file: unzFile) -> c_int {
    let mut err: c_int = UNZ_OK;
    let Store: i32;
    let mut iSizeVar: uInt = 0;
    let s: *mut unz_s;
    let mut pfile_in_zip_read_info: *mut file_in_zip_read_info_s;
    let mut offset_local_extrafield: uLong = 0; // offset of the static extra field
    let mut size_local_extrafield: uInt = 0; // size of the static extra field

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;
    if (*s).current_file_ok == 0 {
        return UNZ_PARAMERROR;
    }

    if !(*s).pfile_in_zip_read.is_null() {
        unzCloseCurrentFile(file);
    }

    if unzlocal_CheckCurrentFileCoherencyHeader(
        s,
        &mut iSizeVar,
        &mut offset_local_extrafield,
        &mut size_local_extrafield,
    ) != UNZ_OK
    {
        return UNZ_BADZIPFILE;
    }

    pfile_in_zip_read_info = Z_Malloc(
        core::mem::size_of::<file_in_zip_read_info_s>() as uLong,
        0,
        0,
    ) as *mut file_in_zip_read_info_s;

    (*pfile_in_zip_read_info).read_buffer =
        Z_Malloc(UNZ_BUFSIZE, 0, 0) as *mut c_char;
    (*pfile_in_zip_read_info).offset_local_extrafield = offset_local_extrafield;
    (*pfile_in_zip_read_info).size_local_extrafield = size_local_extrafield;
    (*pfile_in_zip_read_info).pos_local_extrafield = 0;

    (*pfile_in_zip_read_info).stream_initialised = 0;

    if ((*s).cur_file_info.compression_method != 0)
        && ((*s).cur_file_info.compression_method != ZF_DEFLATED as uLong)
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
        err = inflateInit(&mut (*pfile_in_zip_read_info).stream, Z_SYNC_FLUSH, 1);
        if err == Z_OK {
            (*pfile_in_zip_read_info).stream_initialised = 1;
        }
        /* windowBits is passed < 0 to tell that there is no zlib header.
         * Note that in this case inflate *requires* an extra "dummy" byte
         * after the compressed stream in order to complete decompression and
         * return Z_STREAM_END.
         * In unzip, i don't wait absolutely Z_STREAM_END because I known the
         * size of both compressed and uncompressed data
         */
    }

    (*pfile_in_zip_read_info).rest_read_compressed = (*s).cur_file_info.compressed_size;
    (*pfile_in_zip_read_info).rest_read_uncompressed = (*s).cur_file_info.uncompressed_size;

    (*pfile_in_zip_read_info).pos_in_zipfile =
        (*s).cur_file_info_internal.offset_curfile + SIZEZIPLOCALHEADER + iSizeVar as uLong;

    (*pfile_in_zip_read_info).stream.avail_in = 0;

    (*s).pfile_in_zip_read = pfile_in_zip_read_info;
    return UNZ_OK;
}

/*
  Read bytes from the current file.
  buf contain buffer where data must be copied
  len the size of buf.

  return the number of byte copied if somes bytes are copied
  return 0 if the end of file was reached
  return <0 with error code if there is an error
    (UNZ_ERRNO for IO error, or zLib error for uncompress error)
*/
#[no_mangle]
pub unsafe extern "C" fn unzReadCurrentFile(file: unzFile, buf: *mut c_void, len: u32) -> c_int {
    let mut err: c_int = UNZ_OK;
    let mut iRead: uInt = 0;
    let s: *mut unz_s;
    let mut pfile_in_zip_read_info: *mut file_in_zip_read_info_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
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
    (*pfile_in_zip_read_info).stream.avail_out = len as uInt;

    if len as uLong > (*pfile_in_zip_read_info).rest_read_uncompressed {
        (*pfile_in_zip_read_info).stream.avail_out =
            (*pfile_in_zip_read_info).rest_read_uncompressed as uInt;
    }

    while (*pfile_in_zip_read_info).stream.avail_out > 0 {
        if ((*pfile_in_zip_read_info).stream.avail_in == 0)
            && ((*pfile_in_zip_read_info).rest_read_compressed > 0)
        {
            let mut uReadThis: uInt = UNZ_BUFSIZE as uInt;
            if ((*pfile_in_zip_read_info).rest_read_compressed as uInt) < uReadThis {
                uReadThis = (*pfile_in_zip_read_info).rest_read_compressed as uInt;
            }
            if uReadThis == 0 {
                return UNZ_EOF;
            }
            if (*s).cur_file_info.compressed_size == (*pfile_in_zip_read_info).rest_read_compressed {
                if ZIP_fseek(
                    (*pfile_in_zip_read_info).file,
                    ((*pfile_in_zip_read_info).pos_in_zipfile
                        + (*pfile_in_zip_read_info).byte_before_the_zipfile) as i32,
                    SEEK_SET,
                ) != 0
                {
                    return UNZ_ERRNO;
                }
            }
            if ZIP_fread(
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
            let mut i: uInt = 0;

            if (*pfile_in_zip_read_info).stream.avail_out
                < (*pfile_in_zip_read_info).stream.avail_in
            {
                uDoCopy = (*pfile_in_zip_read_info).stream.avail_out;
            } else {
                uDoCopy = (*pfile_in_zip_read_info).stream.avail_in;
            }

            while i < uDoCopy {
                *((*pfile_in_zip_read_info).stream.next_out.add(i as usize)) =
                    *((*pfile_in_zip_read_info).stream.next_in.add(i as usize));
                i += 1;
            }

            (*pfile_in_zip_read_info).rest_read_uncompressed -= uDoCopy as uLong;
            (*pfile_in_zip_read_info).stream.avail_in -= uDoCopy;
            (*pfile_in_zip_read_info).stream.avail_out -= uDoCopy;
            (*pfile_in_zip_read_info).stream.next_out =
                (*pfile_in_zip_read_info).stream.next_out.add(uDoCopy as usize);
            (*pfile_in_zip_read_info).stream.next_in =
                (*pfile_in_zip_read_info).stream.next_in.add(uDoCopy as usize);
            (*pfile_in_zip_read_info).stream.total_out += uDoCopy as u32;
            iRead += uDoCopy;
        } else {
            let mut uTotalOutBefore: uLong;
            let mut bufBefore: *const Byte;
            let mut uOutThis: uLong;
            let mut uTotalOutAfter: uLong;

            uTotalOutBefore = (*pfile_in_zip_read_info).stream.total_out as uLong;
            bufBefore = (*pfile_in_zip_read_info).stream.next_out;

            err = inflate(&mut (*pfile_in_zip_read_info).stream);

            uTotalOutAfter = (*pfile_in_zip_read_info).stream.total_out as uLong;
            uOutThis = uTotalOutAfter - uTotalOutBefore;

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
        return iRead as c_int;
    }
    return err;
}

/*
  Give the current position in uncompressed data
*/
#[no_mangle]
pub unsafe extern "C" fn unztell(file: unzFile) -> i32 {
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;
    pfile_in_zip_read_info = (*s).pfile_in_zip_read;

    if pfile_in_zip_read_info.is_null() {
        return UNZ_PARAMERROR;
    }

    return (*pfile_in_zip_read_info).stream.total_out as i32;
}

/*
  return 1 if the end of file was reached, 0 elsewhere
*/
#[no_mangle]
pub unsafe extern "C" fn unzeof(file: unzFile) -> c_int {
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;
    pfile_in_zip_read_info = (*s).pfile_in_zip_read;

    if pfile_in_zip_read_info.is_null() {
        return UNZ_PARAMERROR;
    }

    if (*pfile_in_zip_read_info).rest_read_uncompressed == 0 {
        return 1;
    } else {
        return 0;
    }
}

/*
  Read extra field from the current file (opened by unzOpenCurrentFile)
  This is the static-header version of the extra field (sometimes, there is
    more info in the static-header version than in the central-header)

  if buf==NULL, it return the size of the static extra field that can be read

  if buf!=NULL, len is the size of the buffer, the extra header is copied in
    buf.
  the return value is the number of bytes copied in buf, or (if <0)
    the error code
*/
#[no_mangle]
pub unsafe extern "C" fn unzGetLocalExtrafield(file: unzFile, buf: *mut c_void, len: u32) -> c_int {
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;
    let mut read_now: uInt = 0;
    let mut size_to_read: uLong;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
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

    if (len as uLong) > size_to_read {
        read_now = size_to_read as uInt;
    } else {
        read_now = len as uInt;
    }

    if read_now == 0 {
        return 0;
    }

    if ZIP_fseek(
        (*pfile_in_zip_read_info).file,
        ((*pfile_in_zip_read_info).offset_local_extrafield
            + (*pfile_in_zip_read_info).pos_local_extrafield) as i32,
        SEEK_SET,
    ) != 0
    {
        return UNZ_ERRNO;
    }

    if ZIP_fread(buf, size_to_read as usize, 1, (*pfile_in_zip_read_info).file) != 1 {
        return UNZ_ERRNO;
    }

    return read_now as c_int;
}

/*
  Close the file in zip opened with unzipOpenCurrentFile
  Return UNZ_CRCERROR if all the file was read but the CRC is not good
*/
#[no_mangle]
pub unsafe extern "C" fn unzCloseCurrentFile(file: unzFile) -> c_int {
    let mut err: c_int = UNZ_OK;
    let s: *mut unz_s;
    let pfile_in_zip_read_info: *mut file_in_zip_read_info_s;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;
    pfile_in_zip_read_info = (*s).pfile_in_zip_read;

    if pfile_in_zip_read_info.is_null() {
        return UNZ_PARAMERROR;
    }

    if !(*pfile_in_zip_read_info).read_buffer.is_null() {
        Z_Free((*pfile_in_zip_read_info).read_buffer as *mut c_void);
    }
    (*pfile_in_zip_read_info).read_buffer = core::ptr::null_mut();

    if (*pfile_in_zip_read_info).stream_initialised != 0 {
        inflateEnd(&mut (*pfile_in_zip_read_info).stream);
    }

    (*pfile_in_zip_read_info).stream_initialised = 0;
    if !pfile_in_zip_read_info.is_null() {
        Z_Free(pfile_in_zip_read_info as *mut c_void);
    }

    (*s).pfile_in_zip_read = core::ptr::null_mut();

    return err;
}

/*
  Get the global comment string of the ZipFile, in the szComment buffer.
  uSizeBuf is the size of the szComment buffer.
  return the number of byte copied or an error code <0
*/
#[no_mangle]
pub unsafe extern "C" fn unzGetGlobalComment(
    file: unzFile,
    szComment: *mut c_char,
    uSizeBuf: uLong,
) -> c_int {
    let s: *mut unz_s;
    let mut uReadThis: uLong;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    s = file as *mut unz_s;

    uReadThis = uSizeBuf;
    if uReadThis > (*s).gi.size_comment {
        uReadThis = (*s).gi.size_comment;
    }

    if ZIP_fseek((*s).file, ((*s).central_pos + 22) as i32, SEEK_SET) != 0 {
        return UNZ_ERRNO;
    }

    if uReadThis > 0 {
        *szComment = '\0' as c_char;
        if ZIP_fread(szComment as *mut c_void, uReadThis as usize, 1, (*s).file) != 1 {
            return UNZ_ERRNO;
        }
    }

    if !szComment.is_null() && (uSizeBuf > (*s).gi.size_comment) {
        *(szComment.add((*s).gi.size_comment as usize)) = '\0' as c_char;
    }
    return uReadThis as c_int;
}

// end
