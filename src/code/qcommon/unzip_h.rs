use core::ffi::c_int;

#[cfg(any(feature = "STRICTUNZIP", feature = "STRICTZIPUNZIP"))]
mod unz_file_impl {
    /* like the STRICT of WIN32, we define a pointer that cannot be converted
        from (void*) without cast */
    #[repr(C)]
    pub struct TagunzFile__ {
        unused: i32,
    }
    pub type unzFile = *mut TagunzFile__;
}

#[cfg(not(any(feature = "STRICTUNZIP", feature = "STRICTZIPUNZIP")))]
mod unz_file_impl {
    pub type unzFile = *mut core::ffi::c_void;
}

pub use unz_file_impl::unzFile;

/* ZIP_FILE is an alias for FILE */
pub type ZIP_FILE = libc::FILE;

/* tm_unz contain date/time info */
#[repr(C)]
pub struct tm_unz_s {
    pub tm_sec: core::ffi::c_uint,     /* seconds after the minute - [0,59] */
    pub tm_min: core::ffi::c_uint,     /* minutes after the hour - [0,59] */
    pub tm_hour: core::ffi::c_uint,    /* hours since midnight - [0,23] */
    pub tm_mday: core::ffi::c_uint,    /* day of the month - [1,31] */
    pub tm_mon: core::ffi::c_uint,     /* months since January - [0,11] */
    pub tm_year: core::ffi::c_uint,    /* years - [1980..2044] */
}

/* unz_global_info structure contain global data about the ZIPfile
   These data comes from the end of central dir */
#[repr(C)]
pub struct unz_global_info_s {
    pub number_entry: core::ffi::c_ulong,  /* total number of entries in the central dir on this disk */
    pub size_comment: core::ffi::c_ulong,  /* size of the global comment of the zipfile */
}

/* unz_file_info contain information about a file in the zipfile */
#[repr(C)]
pub struct unz_file_info_s {
    pub version: core::ffi::c_ulong,             /* version made by                 2 unsigned chars */
    pub version_needed: core::ffi::c_ulong,      /* version needed to extract       2 unsigned chars */
    pub flag: core::ffi::c_ulong,                /* general purpose bit flag        2 unsigned chars */
    pub compression_method: core::ffi::c_ulong,  /* compression method              2 unsigned chars */
    pub dosDate: core::ffi::c_ulong,             /* last mod file date in Dos fmt   4 unsigned chars */
    pub crc: core::ffi::c_ulong,                 /* crc-32                          4 unsigned chars */
    pub compressed_size: core::ffi::c_ulong,     /* compressed size                 4 unsigned chars */
    pub uncompressed_size: core::ffi::c_ulong,   /* uncompressed size               4 unsigned chars */
    pub size_filename: core::ffi::c_ulong,       /* filename length                 2 unsigned chars */
    pub size_file_extra: core::ffi::c_ulong,     /* extra field length              2 unsigned chars */
    pub size_file_comment: core::ffi::c_ulong,   /* file comment length             2 unsigned chars */
    pub disk_num_start: core::ffi::c_ulong,      /* disk number start               2 unsigned chars */
    pub internal_fa: core::ffi::c_ulong,         /* internal file attributes        2 unsigned chars */
    pub external_fa: core::ffi::c_ulong,         /* external file attributes        4 unsigned chars */
    pub tmu_date: tm_unz_s,
}

/* unz_file_info_interntal contain internal info about a file in zipfile*/
#[repr(C)]
pub struct unz_file_info_internal_s {
    pub offset_curfile: core::ffi::c_ulong, /* relative offset of static header 4 unsigned chars */
}

/* file_in_zip_read_info_s contain internal information about a file in zipfile,
    when reading and decompress it */
#[repr(C)]
pub struct file_in_zip_read_info_s {
    pub read_buffer: *mut core::ffi::c_char,        /* internal buffer for compressed data */
    pub stream: z_stream,                           /* zLib stream structure for inflate */
    pub pos_in_zipfile: core::ffi::c_ulong,         /* position in unsigned char on the zipfile, for fseek*/
    pub stream_initialised: core::ffi::c_ulong,     /* flag set if stream structure is initialised*/
    pub offset_local_extrafield: core::ffi::c_ulong, /* offset of the static extra field */
    pub size_local_extrafield: core::ffi::c_uint,   /* size of the static extra field */
    pub pos_local_extrafield: core::ffi::c_ulong,   /* position in the static extra field in read*/
    pub crc32: core::ffi::c_ulong,                  /* crc32 of all data uncompressed */
    pub crc32_wait: core::ffi::c_ulong,             /* crc32 we must obtain after decompress all */
    pub rest_read_compressed: core::ffi::c_ulong,   /* number of unsigned char to be decompressed */
    pub rest_read_uncompressed: core::ffi::c_ulong, /* number of unsigned char to be obtained after decomp*/
    pub file: *mut ZIP_FILE,                        /* io structore of the zipfile */
    pub compression_method: core::ffi::c_ulong,     /* compression method (0==store) */
    pub byte_before_the_zipfile: core::ffi::c_ulong, /* unsigned char before the zipfile, (>0 for sfx)*/
}

/* unz_s contain internal information about the zipfile
*/
#[repr(C)]
pub struct unz_s {
    pub file: *mut ZIP_FILE,                           /* io structore of the zipfile */
    pub gi: unz_global_info_s,                         /* public global information */
    pub byte_before_the_zipfile: core::ffi::c_ulong,   /* unsigned char before the zipfile, (>0 for sfx)*/
    pub num_file: core::ffi::c_ulong,                  /* number of the current file in the zipfile*/
    pub pos_in_central_dir: core::ffi::c_ulong,        /* pos of the current file in the central dir*/
    pub current_file_ok: core::ffi::c_ulong,           /* flag about the usability of the current file*/
    pub central_pos: core::ffi::c_ulong,               /* position of the beginning of the central dir*/
    pub size_central_dir: core::ffi::c_ulong,          /* size of the central directory  */
    pub offset_central_dir: core::ffi::c_ulong,        /* offset of start of central directory with
                                                           respect to the starting disk number */
    pub cur_file_info: unz_file_info_s,                /* public info about the current file in zip*/
    pub cur_file_info_internal: unz_file_info_internal_s, /* private info about it*/
    pub pfile_in_zip_read: *mut file_in_zip_read_info_s, /* structure about the current
                                                             file if we are decompressing it */
}

/* Error codes */
pub const UNZ_OK: core::ffi::c_int = 0;
pub const UNZ_END_OF_LIST_OF_FILE: core::ffi::c_int = -100;
pub const UNZ_ERRNO: core::ffi::c_int = -3; /* Z_DATA_ERROR value */
pub const UNZ_EOF: core::ffi::c_int = 0;
pub const UNZ_PARAMERROR: core::ffi::c_int = -102;
pub const UNZ_BADZIPFILE: core::ffi::c_int = -103;
pub const UNZ_INTERNALERROR: core::ffi::c_int = -104;
pub const UNZ_CRCERROR: core::ffi::c_int = -105;

pub const UNZ_CASESENSITIVE: core::ffi::c_int = 1;
pub const UNZ_NOTCASESENSITIVE: core::ffi::c_int = 2;
pub const UNZ_OSDEFAULTCASE: core::ffi::c_int = 0;

extern "C" {
    pub fn unzStringFileNameCompare(
        fileName1: *const core::ffi::c_char,
        fileName2: *const core::ffi::c_char,
        iCaseSensitivity: core::ffi::c_int,
    ) -> core::ffi::c_int;

    /*
       Compare two filename (fileName1,fileName2).
       If iCaseSenisivity = 1, comparision is case sensitivity (like strcmp)
       If iCaseSenisivity = 2, comparision is not case sensitivity (like strcmpi
                                or strcasecmp)
       If iCaseSenisivity = 0, case sensitivity is defaut of your operating system
        (like 1 on Unix, 2 on Windows)
    */

    pub fn unzOpen(path: *const core::ffi::c_char) -> unzFile;

    /*
      Open a Zip file. path contain the full pathname (by example,
         on a Windows NT computer "c:\\zlib\\zlib111.zip" or on an Unix computer
         "zlib/zlib111.zip".
         If the zipfile cannot be opened (file don't exist or in not valid), the
           return value is NULL.
         Else, the return value is a unzFile Handle, usable with other function
           of this unzip package.
    */

    pub fn unzReOpen(path: *const core::ffi::c_char, file: unzFile) -> unzFile;

    pub fn unzClose(file: unzFile) -> core::ffi::c_int;

    /*
      Close a ZipFile opened with unzipOpen.
      If there is files inside the .Zip opened with unzOpenCurrentFile (see later),
        these files MUST be closed with unzipCloseCurrentFile before call unzipClose.
      return UNZ_OK if there is no problem. */

    pub fn unzGetGlobalInfo(file: unzFile, pglobal_info: *mut unz_global_info_s) -> core::ffi::c_int;

    /*
      Write info about the ZipFile in the *pglobal_info structure.
      No preparation of the structure is needed
      return UNZ_OK if there is no problem. */

    pub fn unzGetGlobalComment(
        file: unzFile,
        szComment: *mut core::ffi::c_char,
        uSizeBuf: core::ffi::c_ulong,
    ) -> core::ffi::c_int;

    /*
      Get the global comment string of the ZipFile, in the szComment buffer.
      uSizeBuf is the size of the szComment buffer.
      return the number of unsigned char copied or an error code <0
    */

    /* Unzip package allow you browse the directory of the zipfile */

    pub fn unzGoToFirstFile(file: unzFile) -> core::ffi::c_int;

    /*
      Set the current file of the zipfile to the first file.
      return UNZ_OK if there is no problem
    */

    pub fn unzGoToNextFile(file: unzFile) -> core::ffi::c_int;

    /*
      Set the current file of the zipfile to the next file.
      return UNZ_OK if there is no problem
      return UNZ_END_OF_LIST_OF_FILE if the actual file was the latest.
    */

    pub fn unzGetCurrentFileInfoPosition(file: unzFile, pos: *mut core::ffi::c_ulong)
        -> core::ffi::c_int;

    /*
      Get the position of the info of the current file in the zip.
      return UNZ_OK if there is no problem
    */

    pub fn unzSetCurrentFileInfoPosition(file: unzFile, pos: core::ffi::c_ulong) -> core::ffi::c_int;

    /*
      Set the position of the info of the current file in the zip.
      return UNZ_OK if there is no problem
    */

    pub fn unzLocateFile(
        file: unzFile,
        szFileName: *const core::ffi::c_char,
        iCaseSensitivity: core::ffi::c_int,
    ) -> core::ffi::c_int;

    /*
      Try locate the file szFileName in the zipfile.
      For the iCaseSensitivity signification, see unzStringFileNameCompare

      return value :
      UNZ_OK if the file is found. It becomes the current file.
      UNZ_END_OF_LIST_OF_FILE if the file is not found
    */

    pub fn unzGetCurrentFileInfo(
        file: unzFile,
        pfile_info: *mut unz_file_info_s,
        szFileName: *mut core::ffi::c_char,
        fileNameBufferSize: core::ffi::c_ulong,
        extraField: *mut core::ffi::c_void,
        extraFieldBufferSize: core::ffi::c_ulong,
        szComment: *mut core::ffi::c_char,
        commentBufferSize: core::ffi::c_ulong,
    ) -> core::ffi::c_int;

    /*
      Get Info about the current file
      if pfile_info!=NULL, the *pfile_info structure will contain somes info about
             the current file
      if szFileName!=NULL, the filemane string will be copied in szFileName
                 (fileNameBufferSize is the size of the buffer)
      if extraField!=NULL, the extra field information will be copied in extraField
                 (extraFieldBufferSize is the size of the buffer).
                 This is the Central-header version of the extra field
      if szComment!=NULL, the comment string of the file will be copied in szComment
                 (commentBufferSize is the size of the buffer)
    */

    /* for reading the content of the current zipfile, you can open it, read data
       from it, and close it (you can close it before reading all the file)
    */

    pub fn unzOpenCurrentFile(file: unzFile) -> core::ffi::c_int;

    /*
      Open for reading data the current file in the zipfile.
      If there is no error, the return value is UNZ_OK.
    */

    pub fn unzCloseCurrentFile(file: unzFile) -> core::ffi::c_int;

    /*
      Close the file in zip opened with unzOpenCurrentFile
      Return UNZ_CRCERROR if all the file was read but the CRC is not good
    */

    pub fn unzReadCurrentFile(file: unzFile, buf: *mut core::ffi::c_void, len: core::ffi::c_uint)
        -> core::ffi::c_int;

    /*
      Read unsigned chars from the current file (opened by unzOpenCurrentFile)
      buf contain buffer where data must be copied
      len the size of buf.

      return the number of unsigned char copied if somes unsigned chars are copied
      return 0 if the end of file was reached
      return <0 with error code if there is an error
        (UNZ_ERRNO for IO error, or zLib error for uncompress error)
    */

    pub fn unztell(file: unzFile) -> core::ffi::c_long;

    /*
      Give the current position in uncompressed data
    */

    pub fn unzeof(file: unzFile) -> core::ffi::c_int;

    /*
      return 1 if the end of file was reached, 0 elsewhere
    */

    pub fn unzGetLocalExtrafield(file: unzFile, buf: *mut core::ffi::c_void, len: core::ffi::c_uint)
        -> core::ffi::c_int;

    /*
      Read extra field from the current file (opened by unzOpenCurrentFile)
      This is the local-header version of the extra field (sometimes, there is
        more info in the local-header version than in the central-header)

      if buf==NULL, it return the size of the local extra field

      if buf!=NULL, len is the size of the buffer, the extra header is copied in
         buf.
      the return value is the number of unsigned chars copied in buf, or (if <0)
         the error code
    */
}

/* z_stream is from zlib - declared as external opaque type */
extern "C" {
    #[repr(C)]
    pub struct z_stream {
        _opaque: core::ffi::c_void,
    }
}
