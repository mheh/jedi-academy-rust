//! Mechanical port of `codemp/qcommon/unzip.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::zlib32::zip_h::{z_stream, EStatus};
use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};

pub type FILE = c_void;
pub type ZIP_FILE = FILE;

// STRICTUNZIP / STRICTZIPUNZIP are not defined in this port configuration.
pub type unzFile = *mut c_void;

/* tm_unz contain date/time info */
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct tm_unz_s {
    pub tm_sec: c_uint,  /* seconds after the minute - [0,59] */
    pub tm_min: c_uint,  /* minutes after the hour - [0,59] */
    pub tm_hour: c_uint, /* hours since midnight - [0,23] */
    pub tm_mday: c_uint, /* day of the month - [1,31] */
    pub tm_mon: c_uint,  /* months since January - [0,11] */
    pub tm_year: c_uint, /* years - [1980..2044] */
}

pub type tm_unz = tm_unz_s;

/* unz_global_info structure contain global data about the ZIPfile
   These data comes from the end of central dir */
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct unz_global_info_s {
    pub number_entry: c_ulong, /* total number of entries in the central dir on this disk */
    pub size_comment: c_ulong, /* size of the global comment of the zipfile */
}

pub type unz_global_info = unz_global_info_s;

/* unz_file_info contain information about a file in the zipfile */
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct unz_file_info_s {
    pub version: c_ulong,              /* version made by                 2 unsigned chars */
    pub version_needed: c_ulong,       /* version needed to extract       2 unsigned chars */
    pub flag: c_ulong,                 /* general purpose bit flag        2 unsigned chars */
    pub compression_method: c_ulong,   /* compression method              2 unsigned chars */
    pub dosDate: c_ulong,              /* last mod file date in Dos fmt   4 unsigned chars */
    pub crc: c_ulong,                  /* crc-32                          4 unsigned chars */
    pub compressed_size: c_ulong,      /* compressed size                 4 unsigned chars */
    pub uncompressed_size: c_ulong,    /* uncompressed size               4 unsigned chars */
    pub size_filename: c_ulong,        /* filename length                 2 unsigned chars */
    pub size_file_extra: c_ulong,      /* extra field length              2 unsigned chars */
    pub size_file_comment: c_ulong,    /* file comment length             2 unsigned chars */
    pub disk_num_start: c_ulong,       /* disk number start               2 unsigned chars */
    pub internal_fa: c_ulong,          /* internal file attributes        2 unsigned chars */
    pub external_fa: c_ulong,          /* external file attributes        4 unsigned chars */
    pub tmu_date: tm_unz,
}

pub type unz_file_info = unz_file_info_s;

/* unz_file_info_interntal contain internal info about a file in zipfile*/
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct unz_file_info_internal_s {
    pub offset_curfile: c_ulong, /* relative offset of static header 4 unsigned chars */
}

pub type unz_file_info_internal = unz_file_info_internal_s;

/* file_in_zip_read_info_s contain internal information about a file in zipfile,
    when reading and decompress it */
#[repr(C)]
#[derive(Debug)]
pub struct file_in_zip_read_info_s {
    pub read_buffer: *mut c_char, /* internal buffer for compressed data */
    pub stream: z_stream,         /* zLib stream structure for inflate */

    pub pos_in_zipfile: c_ulong,     /* position in unsigned char on the zipfile, for fseek*/
    pub stream_initialised: c_ulong, /* flag set if stream structure is initialised*/

    pub offset_local_extrafield: c_ulong, /* offset of the static extra field */
    pub size_local_extrafield: c_uint,    /* size of the static extra field */
    pub pos_local_extrafield: c_ulong,    /* position in the static extra field in read*/

    pub crc32: c_ulong,                  /* crc32 of all data uncompressed */
    pub crc32_wait: c_ulong,             /* crc32 we must obtain after decompress all */
    pub rest_read_compressed: c_ulong,   /* number of unsigned char to be decompressed */
    pub rest_read_uncompressed: c_ulong, /*number of unsigned char to be obtained after decomp*/
    pub file: *mut ZIP_FILE,             /* io structore of the zipfile */
    pub compression_method: c_ulong,     /* compression method (0==store) */
    pub byte_before_the_zipfile: c_ulong, /* unsigned char before the zipfile, (>0 for sfx)*/
}

/* unz_s contain internal information about the zipfile
 */
#[repr(C)]
#[derive(Debug)]
pub struct unz_s {
    pub file: *mut ZIP_FILE,       /* io structore of the zipfile */
    pub gi: unz_global_info,       /* public global information */
    pub byte_before_the_zipfile: c_ulong, /* unsigned char before the zipfile, (>0 for sfx)*/
    pub num_file: c_ulong,         /* number of the current file in the zipfile*/
    pub pos_in_central_dir: c_ulong, /* pos of the current file in the central dir*/
    pub current_file_ok: c_ulong,  /* flag about the usability of the current file*/
    pub central_pos: c_ulong,      /* position of the beginning of the central dir*/

    pub size_central_dir: c_ulong, /* size of the central directory  */
    pub offset_central_dir: c_ulong, /* offset of start of central directory with
                                      respect to the starting disk number */

    pub cur_file_info: unz_file_info, /* public info about the current file in zip*/
    pub cur_file_info_internal: unz_file_info_internal, /* private info about it*/
    pub pfile_in_zip_read: *mut file_in_zip_read_info_s, /* structure about the current
                                                          file if we are decompressing it */
    pub tmpFile: *mut u8,
    pub tmpPos: c_int,
    pub tmpSize: c_int,
}

pub const UNZ_OK: c_int = 0;
pub const UNZ_END_OF_LIST_OF_FILE: c_int = -100;
pub const UNZ_ERRNO: c_int = EStatus::Z_DATA_ERROR as c_int;
pub const UNZ_EOF: c_int = 0;
pub const UNZ_PARAMERROR: c_int = -102;
pub const UNZ_BADZIPFILE: c_int = -103;
pub const UNZ_INTERNALERROR: c_int = -104;
pub const UNZ_CRCERROR: c_int = -105;

pub const UNZ_CASESENSITIVE: c_int = 1;
pub const UNZ_NOTCASESENSITIVE: c_int = 2;
pub const UNZ_OSDEFAULTCASE: c_int = 0;

unsafe extern "C" {
    pub fn unzStringFileNameCompare(
        fileName1: *const c_char,
        fileName2: *const c_char,
        iCaseSensitivity: c_int,
    ) -> c_int;

    /*
       Compare two filename (fileName1,fileName2).
       If iCaseSenisivity = 1, comparision is case sensitivity (like strcmp)
       If iCaseSenisivity = 2, comparision is not case sensitivity (like strcmpi
                                    or strcasecmp)
       If iCaseSenisivity = 0, case sensitivity is defaut of your operating system
        (like 1 on Unix, 2 on Windows)
    */

    pub fn unzOpen(path: *const c_char) -> unzFile;
    pub fn unzReOpen(path: *const c_char, file: unzFile) -> unzFile;

    pub fn unzClose(file: unzFile) -> c_int;
    pub fn unzGetGlobalInfo(file: unzFile, pglobal_info: *mut unz_global_info) -> c_int;
    pub fn unzGetGlobalComment(file: unzFile, szComment: *mut c_char, uSizeBuf: c_ulong)
        -> c_int;

    /***************************************************************************/
    /* Unzip package allow you browse the directory of the zipfile */

    pub fn unzGoToFirstFile(file: unzFile) -> c_int;
    pub fn unzGoToNextFile(file: unzFile) -> c_int;
    pub fn unzGetCurrentFileInfoPosition(file: unzFile, pos: *mut c_ulong) -> c_int;
    pub fn unzSetCurrentFileInfoPosition(file: unzFile, pos: c_ulong) -> c_int;
    pub fn unzLocateFile(
        file: unzFile,
        szFileName: *const c_char,
        iCaseSensitivity: c_int,
    ) -> c_int;

    pub fn unzGetCurrentFileInfo(
        file: unzFile,
        pfile_info: *mut unz_file_info,
        szFileName: *mut c_char,
        fileNameBufferSize: c_ulong,
        extraField: *mut c_void,
        extraFieldBufferSize: c_ulong,
        szComment: *mut c_char,
        commentBufferSize: c_ulong,
    ) -> c_int;

    /***************************************************************************/
    /* for reading the content of the current zipfile, you can open it, read data
       from it, and close it (you can close it before reading all the file)
       */

    pub fn unzOpenCurrentFile(file: unzFile) -> c_int;
    pub fn unzCloseCurrentFile(file: unzFile) -> c_int;
    pub fn unzReadCurrentFile(file: unzFile, buf: *mut c_void, len: c_uint) -> c_int;
    pub fn unztell(file: unzFile) -> core::ffi::c_long;
    pub fn unzeof(file: unzFile) -> c_int;
    pub fn unzGetLocalExtrafield(file: unzFile, buf: *mut c_void, len: c_uint) -> c_int;
}
