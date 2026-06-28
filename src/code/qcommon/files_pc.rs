//! Mechanical port of `code/qcommon/files_pc.cpp`.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void, c_ulong};
use crate::codemp::qcommon::files_h::{
    fileHandle_t, qboolean, FILE, cvar_t, searchpath_t, pack_t, fileInPack_t,
    directory_t, fileHandleData_t, unzFile, qfile_gut,
    MAX_SEARCH_PATHS, MAX_FILEHASH_SIZE, MAX_OSPATH, MAX_ZPATH, MAX_FILE_HANDLES,
    BASEGAME, DEMOGAME, DEMO_PAK_CHECKSUM,
};
use crate::ffi::types::{QFALSE, QTRUE};

// PORT: We need extern "C" declarations for libc functions and other engine functions
extern "C" {
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Z_Malloc(size: usize, tag: c_int, clear: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Z_Label(ptr: *mut c_void, label: *const c_char);
    pub fn Cvar_Get(varName: *const c_char, varValue: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Cvar_Set(varName: *const c_char, varValue: *const c_char);
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_Argv(arg: c_int) -> *const c_char;
    pub fn Cmd_AddCommand(cmdName: *const c_char, function: extern "C" fn());
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    pub fn Q_strlwr(string: *mut c_char) -> *mut c_char;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn S_ClearSoundBuffer();
    pub fn Sys_ListFiles(directory: *const c_char, extension: *const c_char, numfiles: *mut c_int, wantsubs: qboolean) -> *mut *mut c_char;
    pub fn Sys_FreeFileList(list: *mut *mut c_char);
    pub fn Sys_FileOutOfDate(source: *const c_char, dest: *const c_char) -> qboolean;
    pub fn Sys_CopyFile(ospath: *const c_char, copypath: *const c_char, qbSilent: qboolean) -> qboolean;
    pub fn Sys_DefaultCDPath() -> *const c_char;
    pub fn Sys_DefaultBasePath() -> *const c_char;
    pub fn FS_BuildOSPath(base: *const c_char, game: *const c_char, qpath: *const c_char) -> *mut c_char;
    pub fn FS_CreatePath(OSPath: *mut c_char) -> qboolean;
    pub fn FS_HandleForFile() -> fileHandle_t;
    pub fn FS_FilenameCompare(s1: *const c_char, s2: *const c_char) -> qboolean;
    pub fn FS_SV_FOpenFileRead(filename: *const c_char, fp: *mut fileHandle_t) -> c_int;
    pub fn FS_FOpenFileAppend(filename: *const c_char) -> fileHandle_t;
    pub fn FS_Shutdown();
    pub fn Com_BlockChecksum(buffer: *const c_void, length: c_int) -> c_int;
    pub fn LittleLong(l: c_int) -> c_int;
    pub fn tolower(c: c_int) -> c_int;
    pub fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE;
    pub fn fclose(file: *mut FILE) -> c_int;
    pub fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut FILE) -> usize;
    pub fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut FILE) -> usize;
    pub fn fseek(stream: *mut FILE, offset: c_int, whence: c_int) -> c_int;
    pub fn ftell(stream: *mut FILE) -> c_int;
    pub fn setvbuf(stream: *mut FILE, buf: *mut c_char, mode: c_int, size: usize) -> c_int;
    pub fn fflush(stream: *mut FILE) -> c_int;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    pub fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn remove(path: *const c_char) -> c_int;
    pub fn rename(oldpath: *const c_char, newpath: *const c_char) -> c_int;
    pub fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: extern "C" fn(*const c_void, *const c_void) -> c_int);
    pub fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn unzOpen(path: *const c_char) -> unzFile;
    pub fn unzGetGlobalInfo(file: unzFile, pglobal_info: *mut c_void) -> c_int;
    pub fn unzGoToFirstFile(file: unzFile) -> c_int;
    pub fn unzGetCurrentFileInfo(file: unzFile, pfile_info: *mut c_void, szFileName: *mut c_char, fileNameBufferSize: c_ulong, extraField: *mut c_void, extraFieldBufferSize: c_ulong, szComment: *mut c_char, commentBufferSize: c_ulong) -> c_int;
    pub fn unzGoToNextFile(file: unzFile) -> c_int;
    pub fn unzReOpen(path: *const c_char, file: unzFile) -> unzFile;
    pub fn unzSetCurrentFileInfoPosition(file: unzFile, pos: c_ulong) -> c_int;
    pub fn unzOpenCurrentFile(file: unzFile) -> c_int;
    pub fn unzReadCurrentFile(file: unzFile, buf: *mut c_void, len: c_int) -> c_int;
    pub fn unzCloseCurrentFile(file: unzFile) -> c_int;
    pub fn unzClose(file: unzFile) -> c_int;
    pub fn unzGetCurrentFileInfoPosition(file: unzFile, pos: *mut c_ulong) -> c_int;
    pub fn unztell(file: unzFile) -> c_int;
    pub fn CopyString(str: *const c_char) -> *mut c_char;
}

// PORT: Use the imported globals from files_h
use crate::codemp::qcommon::files_h::{
    fs_gamedir, fs_debug, fs_basepath, fs_cdpath, fs_copyfiles, fs_gamedirvar, fs_restrict,
    fs_searchpaths, fs_readCount, fs_loadCount, fs_packFiles, fsh,
};

const MAX_ZPATH_VAL: usize = MAX_ZPATH;
const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;
const FS_SEEK_CUR: c_int = 0;
const FS_SEEK_END: c_int = 1;
const FS_SEEK_SET: c_int = 2;
const SEEK_CUR_VAL: c_int = 1;
const SEEK_END_VAL: c_int = 2;
const SEEK_SET_VAL: c_int = 0;
const _IONBF: c_int = 2;
const ERR_DROP: c_int = 0;
const ERR_FATAL: c_int = 1;
const TAG_FILESYS: c_int = 8;
const CVAR_INIT: c_int = 0x0001;
const CVAR_SERVERINFO: c_int = 0x0002;
const CA_ACTIVE: c_int = 3;
const FS_READ: c_int = 0;
const FS_WRITE: c_int = 1;
const FS_APPEND: c_int = 2;
const FS_APPEND_SYNC: c_int = 3;
const PATH_SEP: c_int = '/' as c_int;
const UNZ_OK: c_int = 0;
const DEMO_PAK_MAXFILES: u32 = 5174u;
const S_COLOR_CYAN: &str = "";
const S_COLOR_MAGENTA: &str = "";

#[repr(C)]
#[derive(Copy, Clone)]
struct unz_s {
    file: *mut c_void,
    cur_file_info: unz_file_info,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct unz_file_info {
    size_filename: c_ulong,
    uncompressed_size: c_int,
    crc: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct unz_global_info {
    number_entry: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ZIP_FILE {
    _unused: c_int,
}

//static int		fs_numServerPaks;
//static int		fs_serverPaks[MAX_SEARCH_PATHS];

// productId: This file is copyright 2003 Raven Software, and may not be duplicated except during a licensed installation of the full commercial version of Star Wars: Jedi Academy
static fs_scrambledProductId: &[u8] = &[
    42, 143, 149, 190,  10, 197, 225, 133, 243,  63, 189, 182, 226,  56, 143,  17, 215,  37, 197, 218,  50, 103,  24, 235, 246, 191, 183, 149, 160, 170,
    230,  52, 176, 231,  15, 194, 236, 247, 159, 168, 132, 154,  24, 133,  67,  85,  36,  97,  99,  86, 117, 189, 212, 156, 236, 153,  68,  10, 196, 241,
    39, 219, 156,  88,  93, 198, 200, 232, 142,  67,  45, 209,  53, 186, 228, 241, 162, 127, 213,  83,   7, 121,  11,  93, 123, 243, 148, 240, 229,  42,
    42,   6, 215, 239, 112, 120, 240, 244, 104,  12,  38,  47, 201, 253, 223, 208, 154,  69, 141, 157,  32, 117, 166, 146, 236,  59,  15, 223,  52,  89,
    133,  64, 201,  56, 119,  25, 211, 152, 159,  11,  92,  59, 207,  81, 123,   0, 121, 241, 116,  42,  36, 251,  51, 149,  79, 165,  12, 106, 187, 225,
    203,  99, 102,  69,  97,  81,  27, 107,  81, 178,  63,  35, 185,  64, 115
];

// ================
// return a hash value for the filename
// ================
pub fn FS_HashFileName(fname: *const c_char, hashSize: c_int) -> c_int {
    let mut i: c_int = 0;
    let mut hash: c_int = 0;
    let mut letter: c_char;

    hash = 0;
    i = 0;
    unsafe {
        loop {
            if *fname.offset(i as isize) == '\0' as c_char {
                break;
            }
            letter = tolower(*fname.offset(i as isize) as c_int) as c_char;
            if letter == '.' as c_char { break; }  // don't include extension
            if letter == '\\' as c_char { letter = '/' as c_char; }  // damn path names
            if letter == '/' as c_char { letter = '/' as c_char; }  // damn path names	//mac and unix are different
            hash += (letter as c_int) * (i + 119);
            i += 1;
        }
    }
    hash = hash ^ (hash >> 10) ^ (hash >> 20);
    hash &= (hashSize - 1);
    hash
}

fn FS_FileForHandle(f: fileHandle_t) -> *mut FILE {
    unsafe {
        if f < 0 || f > MAX_FILE_HANDLES as c_int {
            Com_Error( ERR_DROP, "FS_FileForHandle: out of reange\0".as_ptr() as *const c_char );
        }
        if (*core::ptr::addr_of!(fsh[f as usize])).zipFile == QTRUE {
            Com_Error( ERR_DROP, "FS_FileForHandle: can\'t get FILE on zip file\0".as_ptr() as *const c_char );
        }
        if (*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.o.is_null() {
            Com_Error( ERR_DROP, "FS_FileForHandle: NULL\0".as_ptr() as *const c_char );
        }

        return (*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.o;
    }
}

pub fn FS_ForceFlush( f: fileHandle_t ) {
    let file: *mut FILE;

    unsafe {
        file = FS_FileForHandle(f);
        setvbuf( file, core::ptr::null_mut(), _IONBF, 0 );
    }
}

// ================
// FS_filelength
//
// If this is called on a non-unique FILE (from a pak file),
// it will return the size of the pak file, not the expected
// size of the file.
// ================
pub fn FS_filelength( f: fileHandle_t ) -> c_int {
    let mut pos: c_int;
    let mut end: c_int;
    let mut h: *mut FILE;

    unsafe {
        h = FS_FileForHandle(f);
        pos = ftell (h);
        fseek (h, 0, SEEK_END);
        end = ftell (h);
        fseek (h, pos, SEEK_SET);

        return end;
    }
}

// =================
// FS_CopyFile
//
// Copy a fully specified file from one place to another
// =================
// added extra param so behind-the-scenes copying in savegames doesn't clutter up the screen -slc
pub fn FS_CopyFile( fromOSPath: *mut c_char, toOSPath: *mut c_char, qbSilent: qboolean ) -> qboolean {
    let mut f: *mut FILE;
    let mut len: c_int;
    let mut buf: *mut u8;

    unsafe {
        if qbSilent == QFALSE {
            Com_Printf( "copy %s to %s\n\0".as_ptr() as *const c_char, fromOSPath, toOSPath );
        }
        f = fopen( fromOSPath, "rb\0".as_ptr() as *const c_char );
        if f.is_null() {
            return QFALSE;
        }
        fseek (f, 0, SEEK_END);
        len = ftell (f);
        fseek (f, 0, SEEK_SET);

        buf = Z_Malloc( len as usize, TAG_FILESYS, QFALSE) as *mut u8;
        if fread( buf as *mut c_void, 1, len as usize, f ) != (len as usize) {
            Z_Free( buf as *mut c_void );
            fclose(f);
            if qbSilent == QTRUE {
                return QFALSE;
            }
            Com_Error( ERR_FATAL, "Short read in FS_Copyfiles()\n\0".as_ptr() as *const c_char );
        }
        fclose( f );

        FS_CreatePath( toOSPath );
        f = fopen( toOSPath, "wb\0".as_ptr() as *const c_char );
        if f.is_null() {
            Z_Free( buf as *mut c_void );
            return QFALSE;
        }
        if fwrite( buf as *const c_void, 1, len as usize, f ) != (len as usize) {
            Z_Free( buf as *mut c_void );
            fclose(f);
            if qbSilent == QTRUE {
                return QFALSE;
            }
            Com_Error( ERR_FATAL, "Short write in FS_Copyfiles()\n\0".as_ptr() as *const c_char );
        }
        fclose( f );
        Z_Free( buf as *mut c_void );

        return QTRUE;
    }
}

// ==============
// FS_FCloseFile
//
// If the FILE pointer is an open pak file, leave it open.
//
// For some reason, other dll's can't just call fclose()
// on files returned by FS_FOpenFile...
// ==============
pub fn FS_FCloseFile( f: fileHandle_t ) {
    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if (*core::ptr::addr_of!(fsh[f as usize])).zipFile == QTRUE {
            unzCloseCurrentFile( (*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.z );
            if (*core::ptr::addr_of!(fsh[f as usize])).handleFiles.unique != 0 {
                unzClose( (*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.z );
            }
            memset( core::ptr::addr_of_mut!(fsh[f as usize]) as *mut c_void, 0, core::mem::size_of::<fileHandleData_t>() );
            return;
        }

        // we didn't find it as a pak, so close it as a unique file
        fclose ((*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.o);
        memset( core::ptr::addr_of_mut!(fsh[f as usize]) as *mut c_void, 0, core::mem::size_of::<fileHandleData_t>() );
    }
}

// The following functions with "UserGen" in them were added for savegame handling,
//	since outside functions aren't supposed to know about full paths/dirs

// "filename" is local to the current gamedir (eg "saves/blah.sav")
//
pub fn FS_DeleteUserGenFile( filename: *const c_char ) {
    let mut ospath: *mut c_char;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        ospath = FS_BuildOSPath( (*fs_basepath).string, core::ptr::addr_of!(fs_gamedir[0]) as *const c_char, filename );

        if (*(*fs_debug)).integer != 0 {
            Com_Printf( "FS_DeleteUserGenFile: %s\n\0".as_ptr() as *const c_char, ospath );
        }

        remove ( ospath );
    }
}

// filenames are local (eg "saves/blah.sav")
//
// return: qtrue = OK
//
pub fn FS_MoveUserGenFile( filename_src: *const c_char, filename_dst: *const c_char ) -> qboolean {
    let mut ospath_src: *mut c_char;
    let mut ospath_dst: *mut c_char;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        ospath_src = FS_BuildOSPath( (*fs_basepath).string, core::ptr::addr_of!(fs_gamedir[0]) as *const c_char, filename_src );
        ospath_dst = FS_BuildOSPath( (*fs_basepath).string, core::ptr::addr_of!(fs_gamedir[0]) as *const c_char, filename_dst );

        if (*(*fs_debug)).integer != 0 {
            Com_Printf( "FS_MoveUserGenFile: %s to %s\n\0".as_ptr() as *const c_char, ospath_src, ospath_dst );
        }

    /*	int iSlashes1=0;
        int iSlashes2=0;
        char *p;
        for (p = strchr(filename_src,'/'); p; iSlashes1++)
        {
            p = strchr(p+1,'/');
        }
        for (p = strchr(filename_dst,'/'); p; iSlashes2++)
        {
            p = strchr(p+1,'/');
        }

        if (iSlashes1 != iSlashes2)
        {
            int ret = FS_CopyFile( ospath_src, ospath_dst, qtrue );
            remove(ospath_src);
            return ret;
        }
        else
    */
        {
            remove(ospath_dst);
            return if 0 == rename (ospath_src, ospath_dst ) { QTRUE } else { QFALSE };
        }
    }
}

// ===========
// FS_FOpenFileWrite
//
// ===========
pub fn FS_FOpenFileWrite( filename: *const c_char ) -> fileHandle_t {
    let mut ospath: *mut c_char;
    let mut f: fileHandle_t;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        f = FS_HandleForFile();
        (*core::ptr::addr_of_mut!(fsh[f as usize])).zipFile = QFALSE;

        ospath = FS_BuildOSPath( (*fs_basepath).string, core::ptr::addr_of!(fs_gamedir[0]) as *const c_char, filename );

        if (*(*fs_debug)).integer != 0 {
            Com_Printf( "FS_FOpenFileWrite: %s\n\0".as_ptr() as *const c_char, ospath );
        }

        //Com_DPrintf( "writing to: %s\n", ospath );
        FS_CreatePath( ospath );
        (*core::ptr::addr_of_mut!(fsh[f as usize])).handleFiles.file.o = fopen( ospath, "wb\0".as_ptr() as *const c_char );

        Q_strncpyz( core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(fsh[f as usize])).name[0]), filename, core::mem::size_of::<[c_char; MAX_ZPATH]>() );

        (*core::ptr::addr_of_mut!(fsh[f as usize])).handleSync = QFALSE;
        if (*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.o.is_null() {
            f = 0;
        }
        return f;
    }
}

extern "C" {
    pub static mut com_buildScript: *mut cvar_t;
}

fn FS_FileCacheable(filename: *const c_char) -> bool {
    unsafe {
        if !com_buildScript.is_null() && (*com_buildScript).integer != 0 {
            return true;
        }
        return !strchr(filename, '/' as c_int).is_null();
    }
}

// ===========
// FS_FOpenFileRead
//
// Finds the file in the search path.
// Returns filesize and an open FILE pointer.
// Used for streaming data out of either a
// separate file or a ZIP file.
// ===========
pub fn FS_FOpenFileRead( filename: *const c_char, file: *mut fileHandle_t, uniqueFILE: qboolean ) -> c_int {
    let mut search: *mut searchpath_t;
    let mut netpath: *mut c_char;
    let mut pak: *mut pack_t;
    let mut pakFile: *mut fileInPack_t;
    let mut dir: *mut directory_t;
    let mut hash: c_int = 0;
    let mut zfi: *mut unz_s;
    let mut temp: *mut ZIP_FILE;
    //	int				i;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if file.is_null() {
            Com_Error( ERR_FATAL, "FS_FOpenFileRead: NULL \'file\' parameter passed\n\0".as_ptr() as *const c_char );
        }

        if filename.is_null() {
            Com_Error( ERR_FATAL, "FS_FOpenFileRead: NULL \'filename\' parameter passed\n\0".as_ptr() as *const c_char );
        }

        // qpaths are not supposed to have a leading slash
        if *filename == '/' as c_char || *filename == '\\' as c_char {
            // SAFETY: filename is a valid C string at this point; we skip the first char
        }

        // make absolutely sure that it can't back up the path.
        // The searchpaths do guarantee that something will always
        // be prepended, so we don't need to worry about "c:" or "//limbo"
        if !strstr( filename, "..\0".as_ptr() as *const c_char ).is_null() || !strstr( filename, "::\0".as_ptr() as *const c_char ).is_null() {
            *file = 0;
            return -1;
        }

        //
        // search through the path, one element at a time
        //

        *file = FS_HandleForFile();
        (*core::ptr::addr_of_mut!(fsh[*file as usize])).handleFiles.unique = uniqueFILE;

        // this new bool is in for an optimisation, if you (eg) opened a BSP file under fs_copyfiles==2,
        //	then it triggered a copy operation to update your local HD version, then this will re-open the
        //	file handle on your local version, not the net build. This uses a bit more CPU to re-do the loop
        //	logic, but should read faster than accessing the net version a second time.
        //
        let mut bFasterToReOpenUsingNewLocalFile: qboolean = QFALSE;

        loop {
            bFasterToReOpenUsingNewLocalFile = QFALSE;

            search = fs_searchpaths;
            while !search.is_null() {
                //
                if !(*search).pack.is_null() {
                    hash = FS_HashFileName(filename, (*(*search).pack).hashSize);
                }
                // is the element a pak file?
                if !(*search).pack.is_null() && !(*(*(*search).pack).hashTable.add(hash as usize)).is_null() {
                    // disregard if it doesn't match one of the allowed pure pak files
        /*			if ( !FS_PakIsPure(search->pack) ) {
                        continue;
                    }
        */
                    // look through all the pak file elements
                    pak = (*search).pack;
                    pakFile = *(*pak).hashTable.add(hash as usize);
                    loop {
                        // case and separator insensitive comparisons
                        if FS_FilenameCompare( (*pakFile).name, filename ) == QFALSE {
                            // found it!
                            if uniqueFILE == QTRUE {
                                // open a new file on the pakfile
                                (*core::ptr::addr_of_mut!(fsh[*file as usize])).handleFiles.file.z = unzReOpen ((*pak).pakFilename.as_ptr() as *const c_char, (*pak).handle);
                                if (*core::ptr::addr_of!(fsh[*file as usize])).handleFiles.file.z.is_null() {
                                    Com_Error (ERR_FATAL, "Couldn\'t reopen %s\0".as_ptr() as *const c_char, (*pak).pakFilename.as_ptr() as *const c_char);
                                }
                            } else {
                                (*core::ptr::addr_of_mut!(fsh[*file as usize])).handleFiles.file.z = (*pak).handle;
                            }
                            Q_strncpyz( core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(fsh[*file as usize])).name[0]), filename, core::mem::size_of::<[c_char; MAX_ZPATH]>() );
                            (*core::ptr::addr_of_mut!(fsh[*file as usize])).zipFile = QTRUE;
                            zfi = (*core::ptr::addr_of!(fsh[*file as usize])).handleFiles.file.z as *mut unz_s;
                            // in case the file was new
                            temp = (*zfi).file as *mut ZIP_FILE;
                            // set the file position in the zip file (also sets the current file info)
                            unzSetCurrentFileInfoPosition((*pak).handle, (*pakFile).pos);
                            // copy the file info into the unzip structure
                            memcpy( zfi as *mut c_void, (*pak).handle as *const c_void, core::mem::size_of::<unz_s>());
                            // we copy this back into the structure
                            (*zfi).file = temp as *mut c_void;
                            // open the file in the zip
                            unzOpenCurrentFile( (*core::ptr::addr_of!(fsh[*file as usize])).handleFiles.file.z );
                            (*core::ptr::addr_of_mut!(fsh[*file as usize])).zipFilePos = (*pakFile).pos as c_int;

                            if (*(*fs_debug)).integer != 0 {
                                Com_Printf( "FS_FOpenFileRead: %s (found in \'%s\')\n\0".as_ptr() as *const c_char,
                                    filename, (*pak).pakFilename.as_ptr() as *const c_char );
                            }
                            return (*zfi).cur_file_info.uncompressed_size;
                        }
                        if !(*pakFile).next.is_null() {
                            pakFile = (*pakFile).next;
                        } else {
                            break;
                        }
                    }
                } else if !(*search).dir.is_null() {
                    // check a file in the directory tree

                    // if we are running restricted, the only files we
                    // will allow to come from the directory are .cfg files
                    if (*(*fs_restrict)).integer != 0 /*|| fs_numServerPaks*/ {
                        let mut l: c_int;

                        l = strlen( filename ) as c_int;

                        if stricmp( filename.add(l as usize - 4), ".cfg\0".as_ptr() as *const c_char) != 0  // for config files
                            && stricmp( filename.add(l as usize - 4), ".sav\0".as_ptr() as *const c_char) != 0  // for save games
                            && stricmp( filename.add(l as usize - 4), ".dat\0".as_ptr() as *const c_char) != 0 {  // for journal files
                            search = (*search).next;
                            continue;
                        }
                    }

                    dir = (*search).dir;

                    netpath = FS_BuildOSPath( (*dir).path.as_ptr(), (*dir).gamedir.as_ptr(), filename );

                    (*core::ptr::addr_of_mut!(fsh[*file as usize])).handleFiles.file.o = fopen (netpath, "rb\0".as_ptr() as *const c_char);
                    if (*core::ptr::addr_of!(fsh[*file as usize])).handleFiles.file.o.is_null() {
                        search = (*search).next;
                        continue;
                    }

                    // if running with fs_copyfiles 2, and search path == local, then we need to fail to open
                    //	if the time/date stamp != the network version (so it'll loop round again and use the network path,
                    //	which comes later in the search order)
                    //
                    if (*(*fs_copyfiles)).integer == 2 && (*(*fs_cdpath)).string[0] as c_int != 0 && Q_stricmp( (*dir).path.as_ptr(), (*(*fs_basepath)).string ) == 0
                        && FS_FileCacheable(filename) {
                        if Sys_FileOutOfDate( netpath, FS_BuildOSPath( (*(*fs_cdpath)).string, (*dir).gamedir.as_ptr(), filename ) ) == QTRUE {
                            fclose((*core::ptr::addr_of!(fsh[*file as usize])).handleFiles.file.o);
                            (*core::ptr::addr_of_mut!(fsh[*file as usize])).handleFiles.file.o = core::ptr::null_mut();
                            search = (*search).next;
                            continue;	//carry on to find the cdpath version.
                        }
                    }

                    Q_strncpyz( core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(fsh[*file as usize])).name[0]), filename, core::mem::size_of::<[c_char; MAX_ZPATH]>() );
                    (*core::ptr::addr_of_mut!(fsh[*file as usize])).zipFile = QFALSE;
                    if (*(*fs_debug)).integer != 0 {
                        Com_Printf( "FS_FOpenFileRead: %s (found in \'%s/%s\')\n\0".as_ptr() as *const c_char, filename,
                            (*dir).path.as_ptr(), (*dir).gamedir.as_ptr() );
                    }

                    // if we are getting it from the cdpath, optionally copy it
                    //  to the basepath
                    if (*(*fs_copyfiles)).integer != 0 && stricmp( (*dir).path.as_ptr(), (*(*fs_cdpath)).string ) == 0 {
                        let mut copypath: *mut c_char;

                        copypath = FS_BuildOSPath( (*(*fs_basepath)).string, (*dir).gamedir.as_ptr(), filename );

                        match (*(*fs_copyfiles)).integer {
                            1 => {
                                FS_CopyFile( netpath, copypath, QFALSE );
                            }

                            2 => {

                                if FS_FileCacheable(filename) {
                                    // maybe change this to Com_DPrintf?   On the other hand...
                                    //
                                    Com_Printf( "fs_copyfiles(2), Copying: %s to %s\n\0".as_ptr() as *const c_char, netpath, copypath );

                                    FS_CreatePath( copypath );

                                    if Sys_CopyFile( netpath, copypath, QTRUE ) == QTRUE {
                                        // clear this handle and setup for re-opening of the new local copy...
                                        //
                                        bFasterToReOpenUsingNewLocalFile = QTRUE;
                                        fclose((*core::ptr::addr_of!(fsh[*file as usize])).handleFiles.file.o);
                                        (*core::ptr::addr_of_mut!(fsh[*file as usize])).handleFiles.file.o = core::ptr::null_mut();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    if bFasterToReOpenUsingNewLocalFile == QTRUE {
                        break;	// and re-read the local copy, not the net version
                    }

                    return FS_filelength (*file);
                }
                search = (*search).next;
            }

            if bFasterToReOpenUsingNewLocalFile == QFALSE {
                break;
            }
        }

        Com_DPrintf ("Can\'t find %s\n\0".as_ptr() as *const c_char, filename);

        *file = 0;
        return -1;
    }
}

// =================
// FS_Read
//
// Properly handles partial reads
// =================
pub fn FS_Read( buffer: *mut c_void, len: c_int, f: fileHandle_t ) -> c_int {
    let mut block: c_int;
    let mut remaining: c_int;
    let mut read: c_int;
    let mut buf: *mut u8;
    let mut tries: c_int;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if f == 0 {
            return 0;
        }
        if f <= 0 || f >= MAX_FILE_HANDLES as c_int {
            Com_Error( ERR_FATAL, "FS_Read: Invalid handle %d\n\0".as_ptr() as *const c_char, f );
        }

        buf = buffer as *mut u8;
        *core::ptr::addr_of_mut!(fs_readCount) += len;

        if (*core::ptr::addr_of!(fsh[f as usize])).zipFile == QFALSE {
            remaining = len;
            tries = 0;
            while remaining != 0 {
                block = remaining;
                read = fread (buf as *mut c_void, 1, block as usize, (*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.o) as c_int;
                if read == 0 {
                    // we might have been trying to read from a CD, which
                    // sometimes returns a 0 read on windows
                    if tries == 0 {
                        tries = 1;
                    } else {
                        return len - remaining;	//Com_Error (ERR_FATAL, "FS_Read: 0 bytes read");
                    }
                }

                if read == -1 {
                    Com_Error (ERR_FATAL, "FS_Read: -1 bytes read\0".as_ptr() as *const c_char);
                }

                remaining -= read;
                buf = buf.add(read as usize);
            }
            return len;
        } else {
            return unzReadCurrentFile((*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.z, buffer, len);
        }
    }
}

// =================
// FS_Write
//
// Properly handles partial writes
// =================
pub fn FS_Write( buffer: *const c_void, len: c_int, h: fileHandle_t ) -> c_int {
    let mut block: c_int;
    let mut remaining: c_int;
    let mut written: c_int;
    let mut buf: *mut u8;
    let mut tries: c_int;
    let mut f: *mut FILE;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if h == 0 {
            return 0;
        }

        f = FS_FileForHandle(h);
        buf = buffer as *mut u8;

        remaining = len;
        tries = 0;
        while remaining != 0 {
            block = remaining;
            written = fwrite (buf as *const c_void, 1, block as usize, f) as c_int;
            if written == 0 {
                if tries == 0 {
                    tries = 1;
                } else {
                    Com_Printf( "FS_Write: 0 bytes written\n\0".as_ptr() as *const c_char );
                    return 0;
                }
            }

            if written == -1 {
                Com_Printf( "FS_Write: -1 bytes written\n\0".as_ptr() as *const c_char );
                return 0;
            }

            remaining -= written;
            buf = buf.add(written as usize);
        }
        if (*core::ptr::addr_of!(fsh[h as usize])).handleSync != 0 {
            fflush( f );
        }
        return len;
    }
}

// =================
// FS_Seek
//
// =================
pub fn FS_Seek( f: fileHandle_t, offset: c_int, origin: c_int ) -> c_int {
    let mut _origin: c_int;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
            return -1;
        }

        if (*core::ptr::addr_of!(fsh[f as usize])).zipFile == QTRUE {
            let mut foo: [c_char; 65536] = [0; 65536];
            if offset == 0 && origin == FS_SEEK_SET {
                // set the file position in the zip file (also sets the current file info)
                unzSetCurrentFileInfoPosition((*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.z, (*core::ptr::addr_of!(fsh[f as usize])).zipFilePos as c_ulong);
                return unzOpenCurrentFile((*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.z);
            } else if offset < 65536 {
                // set the file position in the zip file (also sets the current file info)
                unzSetCurrentFileInfoPosition((*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.z, (*core::ptr::addr_of!(fsh[f as usize])).zipFilePos as c_ulong);
                unzOpenCurrentFile((*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.z);
                return FS_Read(foo.as_mut_ptr() as *mut c_void, offset, f);
            } else {
                Com_Error( ERR_FATAL, "ZIP FILE FSEEK NOT YET IMPLEMENTED for big offsets(%s)\n\0".as_ptr() as *const c_char, (*core::ptr::addr_of!(fsh[f as usize])).name.as_ptr());
                return -1;
            }
        } else {
            let mut file: *mut FILE;
            file = FS_FileForHandle(f);
            match origin {
                FS_SEEK_CUR => {
                    _origin = SEEK_CUR;
                }
                FS_SEEK_END => {
                    _origin = SEEK_END;
                }
                FS_SEEK_SET => {
                    _origin = SEEK_SET;
                }
                _ => {
                    _origin = SEEK_CUR;
                    Com_Error( ERR_FATAL, "Bad origin in FS_Seek\n\0".as_ptr() as *const c_char );
                }
            }

            return fseek( file, offset, _origin );
        }
    }
}

// ======================================================================================
//
// CONVENIENCE FUNCTIONS FOR ENTIRE FILES
//
// ======================================================================================

pub fn FS_FileIsInPAK(filename: *const c_char ) -> c_int {
    let mut search: *mut searchpath_t;
    let mut pak: *mut pack_t;
    let mut pakFile: *mut fileInPack_t;
    let mut hash: c_int = 0;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if filename.is_null() {
            Com_Error( ERR_FATAL, "FS_FOpenFileRead: NULL \'filename\' parameter passed\n\0".as_ptr() as *const c_char );
        }

        // qpaths are not supposed to have a leading slash
        if *filename == '/' as c_char || *filename == '\\' as c_char {
            // SAFETY: filename is a valid C string; skip the first char
        }

        // make absolutely sure that it can't back up the path.
        // The searchpaths do guarantee that something will always
        // be prepended, so we don't need to worry about "c:" or "//limbo"
        if !strstr( filename, "..\0".as_ptr() as *const c_char ).is_null() || !strstr( filename, "::\0".as_ptr() as *const c_char ).is_null() {
            return -1;
        }

        //
        // search through the path, one element at a time
        //

        search = fs_searchpaths;
        while !search.is_null() {
            //
            if !(*search).pack.is_null() {
                hash = FS_HashFileName(filename, (*(*search).pack).hashSize);
            }
            // is the element a pak file?
            if !(*search).pack.is_null() && !(*(*(*search).pack).hashTable.add(hash as usize)).is_null() {
                // disregard if it doesn't match one of the allowed pure pak files
    /*			if ( !FS_PakIsPure(search->pack) ) {
                    continue;
                }
    */
                // look through all the pak file elements
                pak = (*search).pack;
                pakFile = *(*pak).hashTable.add(hash as usize);
                loop {
                    // case and separator insensitive comparisons
                    if FS_FilenameCompare( (*pakFile).name, filename ) == QFALSE {
                        return 1;
                    }
                    if !(*pakFile).next.is_null() {
                        pakFile = (*pakFile).next;
                    } else {
                        break;
                    }
                }
            }
            search = (*search).next;
        }
        return -1;
    }
}

// ============
// FS_ReadFile
//
// Filename are relative to the quake search path
// a null buffer will just return the file length without loading
// ============
pub fn FS_ReadFile( qpath: *const c_char, buffer: *mut *mut c_void ) -> c_int {
    let mut h: fileHandle_t;
    let mut buf: *mut u8;
    let mut len: c_int;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if qpath.is_null() || *qpath == 0 as c_char {
            Com_Error( ERR_FATAL, "FS_ReadFile with empty name\n\0".as_ptr() as *const c_char );
        }

        // stop sounds from repeating
        S_ClearSoundBuffer();

        // look for it in the filesystem or pack files
        len = FS_FOpenFileRead( qpath, &mut h, QFALSE );
        if h == 0 {
            if !buffer.is_null() {
                *buffer = core::ptr::null_mut();
            }
            return -1;
        }

        if buffer.is_null() {
            FS_FCloseFile( h);
            return len;
        }

        *core::ptr::addr_of_mut!(fs_loadCount) += 1;

        buf = Z_Malloc( (len + 1) as usize, TAG_FILESYS, QFALSE) as *mut u8;
        *buffer = buf as *mut c_void;

        Z_Label(buf as *mut c_void, qpath);

        // PRECACE CHECKER!
        // PORT: Commented out precache checker that requires external symbols
        // #ifndef FINAL_BUILD
        //	if (com_sv_running && com_sv_running->integer && cls.state >= CA_ACTIVE) {	//com_cl_running
        //		if (strncmp(qpath,"menu/",5) ) {
        //			Com_Printf( S_COLOR_MAGENTA"FS_ReadFile: %s NOT PRECACHED!\n", qpath );
        //		}
        //	}
        // #endif

        FS_Read (buf as *mut c_void, len, h);

        // guarantee that it will have a trailing 0 for string operations
        *buf.add(len as usize) = 0;
        FS_FCloseFile( h );
        return len;
    }
}

// =============
// FS_FreeFile
// =============
pub fn FS_FreeFile( buffer: *mut c_void ) {

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }
        if buffer.is_null() {
            Com_Error( ERR_FATAL, "FS_FreeFile( NULL )\0".as_ptr() as *const c_char );
        }

        Z_Free( buffer );
    }
}

// ==========================================================================
//
// ZIP FILE LOADING
//
// ==========================================================================

// =================
// FS_LoadZipFile
//
// Creates a new pak_t in the search chain for the contents
// of a zip file.
// =================
fn FS_LoadZipFile( zipfile: *mut c_char ) -> *mut pack_t {
    let mut buildBuffer: *mut fileInPack_t;
    let mut pack: *mut pack_t;
    let mut uf: unzFile;
    let mut err: c_int;
    let mut gi: unz_global_info;
    let mut filename_inzip: [c_char; MAX_ZPATH] = [0; MAX_ZPATH];
    let mut file_info: unz_file_info;
    let mut i: c_int;
    let mut len: c_int;
    let mut hash: c_int;
    let mut fs_numHeaderLongs: c_int;
    let mut fs_headerLongs: *mut c_int;
    let mut namePtr: *mut c_char;

    unsafe {
        fs_numHeaderLongs = 0;

        uf = unzOpen(zipfile);
        err = unzGetGlobalInfo (uf, &mut gi as *mut c_void as *mut c_void);

        if err != UNZ_OK {
            return core::ptr::null_mut();
        }

        *core::ptr::addr_of_mut!(fs_packFiles) += gi.number_entry;

        len = 0;	//find the length of all filenames
        unzGoToFirstFile(uf);
        i = 0;
        while i < gi.number_entry {
            err = unzGetCurrentFileInfo(uf, &mut file_info as *mut c_void as *mut c_void, filename_inzip.as_mut_ptr(), MAX_ZPATH as c_int, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0);
            if err != UNZ_OK {
                break;
            }
            if file_info.size_filename > MAX_OSPATH as c_ulong {
                Com_Error(ERR_FATAL, "ERROR: filename length > MAX_QPATH ( strlen(%s) = %d) \n\0".as_ptr() as *const c_char, filename_inzip.as_ptr(), file_info.size_filename );
            }
            len += strlen(filename_inzip.as_ptr()) as c_int + 1;
            unzGoToNextFile(uf);
            i += 1;
        }

        buildBuffer = Z_Malloc( (gi.number_entry as usize * core::mem::size_of::<fileInPack_t>() + len as usize) as usize, TAG_FILESYS, QTRUE ) as *mut fileInPack_t;
        namePtr = (buildBuffer as *mut c_char).add(gi.number_entry as usize * core::mem::size_of::<fileInPack_t>());
        fs_headerLongs = Z_Malloc( (gi.number_entry as usize * core::mem::size_of::<c_int>()) as usize, TAG_FILESYS, QTRUE ) as *mut c_int;

        // get the hash table size from the number of files in the zip
        // because lots of custom pk3 files have less than 32 or 64 files
        i = 1;
        while i <= MAX_FILEHASH_SIZE as c_int {
            if i > gi.number_entry {
                break;
            }
            i <<= 1;
        }

        pack = Z_Malloc( (core::mem::size_of::<pack_t>() + i as usize * core::mem::size_of::<*mut fileInPack_t>()) as usize, TAG_FILESYS, QTRUE ) as *mut pack_t;
        memset (pack as *mut c_void, 0, core::mem::size_of::<pack_t>() + i as usize * core::mem::size_of::<*mut fileInPack_t>());
        (*pack).hashSize = i;
        (*pack).hashTable = (pack as *mut c_char).add(core::mem::size_of::<pack_t>()) as *mut *mut fileInPack_t;

        Q_strncpyz( (*pack).pakFilename.as_mut_ptr(), zipfile, core::mem::size_of::<[c_char; MAX_OSPATH]>() );

        (*pack).handle = uf;
        (*pack).numfiles = gi.number_entry;
        unzGoToFirstFile(uf);

        i = 0;
        while i < gi.number_entry {
            err = unzGetCurrentFileInfo(uf, &mut file_info as *mut c_void as *mut c_void, filename_inzip.as_mut_ptr(), MAX_ZPATH as c_int, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0);
            if err != UNZ_OK {
                break;
            }
            if file_info.uncompressed_size > 0 {
                *fs_headerLongs.add(fs_numHeaderLongs as usize) = LittleLong(file_info.crc);
                fs_numHeaderLongs += 1;
            }
            Q_strlwr( filename_inzip.as_mut_ptr() );
            hash = FS_HashFileName(filename_inzip.as_ptr(), (*pack).hashSize);
            (*buildBuffer.add(i as usize)).name = namePtr;
            strcpy( namePtr, filename_inzip.as_ptr() );
            namePtr = namePtr.add(strlen(filename_inzip.as_ptr()) + 1);
            // store the file position in the zip
            unzGetCurrentFileInfoPosition(uf, (*buildBuffer.add(i as usize)).pos as *mut c_ulong);
            //
            (*buildBuffer.add(i as usize)).next = *(*pack).hashTable.add(hash as usize);
            *(*pack).hashTable.add(hash as usize) = buildBuffer.add(i as usize);
            unzGoToNextFile(uf);
            i += 1;
        }

        (*pack).checksum = Com_BlockChecksum( fs_headerLongs as *const c_void, 4 * fs_numHeaderLongs );
        (*pack).checksum = LittleLong( (*pack).checksum );

        Z_Free(fs_headerLongs as *mut c_void);

        (*pack).buildBuffer = buildBuffer;
        return pack;
    }
}

// =================================================================================
//
// DIRECTORY SCANNING FUNCTIONS
//
// =================================================================================

const MAX_FOUND_FILES: usize = 0x1000;

fn FS_ReturnPath( zname: *const c_char, zpath: *mut c_char, depth: *mut c_int ) -> c_int {
    let mut len: c_int;
    let mut at: c_int;
    let mut newdep: c_int;

    unsafe {
        newdep = 0;
        *zpath = 0;
        len = 0;
        at = 0;

        while *zname.add(at as usize) != 0 {
            if *zname.add(at as usize) == '/' as c_char || *zname.add(at as usize) == '\\' as c_char {
                len = at;
                newdep += 1;
            }
            at += 1;
        }
        strcpy(zpath, zname);
        *zpath.add(len as usize) = 0;
        *depth = newdep;

        return len;
    }
}

// ==================
// FS_AddFileToList
// ==================
fn FS_AddFileToList( name: *mut c_char, list: *mut *mut c_char, nfiles: c_int ) -> c_int {
    let mut i: c_int;

    unsafe {
        if nfiles == MAX_FOUND_FILES as c_int - 1 {
            return nfiles;
        }
        i = 0;
        while i < nfiles {
            if stricmp( name, *list.add(i as usize) ) == 0 {
                return nfiles;		// allready in list
            }
            i += 1;
        }
        *list.add(nfiles as usize) = CopyString( name );
        return nfiles + 1;
    }
}

// ===============
// FS_ListFiles
//
// Returns a uniqued list of files that match the given criteria
// from all search paths
// ===============
pub fn FS_ListFiles( path: *const c_char, extension: *const c_char, numfiles: *mut c_int ) -> *mut *mut c_char {
    let mut nfiles: c_int;
    let mut listCopy: *mut *mut c_char;
    let mut list: [*mut c_char; MAX_FOUND_FILES] = [core::ptr::null_mut(); MAX_FOUND_FILES];
    let mut search: *mut searchpath_t;
    let mut i: c_int;
    let mut pathLength: c_int;
    let mut extensionLength: c_int;
    let mut length: c_int;
    let mut pathDepth: c_int;
    let mut pak: *mut pack_t;
    let mut buildBuffer: *mut fileInPack_t;
    let mut zpath: [c_char; MAX_ZPATH] = [0; MAX_ZPATH];

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if path.is_null() {
            *numfiles = 0;
            return core::ptr::null_mut();
        }

        let ext_ptr = if extension.is_null() { "\0".as_ptr() as *const c_char } else { extension };

        pathLength = strlen( path ) as c_int;
        extensionLength = strlen( ext_ptr ) as c_int;
        nfiles = 0;
        FS_ReturnPath(path, zpath.as_mut_ptr(), &mut pathDepth);

        //
        // search through the path, one element at a time, adding to list
        //
        search = fs_searchpaths;
        while !search.is_null() {
            // is the element a pak file?
            if !(*search).pack.is_null() {
                // look through all the pak file elements
                pak = (*search).pack;
                buildBuffer = (*pak).buildBuffer;
                i = 0;
                while i < (*pak).numfiles {
                    let mut name: *mut c_char;
                    let mut zpathLen: c_int;
                    let mut depth: c_int = 0;

                    // check for directory match
                    name = (*buildBuffer.add(i as usize)).name;
                    zpathLen = FS_ReturnPath(name, zpath.as_mut_ptr(), &mut depth);

                    if (depth - pathDepth) > 2 || pathLength > zpathLen || Q_stricmpn( name, path, pathLength as usize ) != 0 {
                        i += 1;
                        continue;
                    }

                    // check for extension match
                    length = strlen( name ) as c_int;
                    if length < extensionLength {
                        i += 1;
                        continue;
                    }

                    if stricmp( name.add((length - extensionLength) as usize), ext_ptr ) != 0 {
                        i += 1;
                        continue;
                    }

                    // unique the match
                    nfiles = FS_AddFileToList( name.add((pathLength + 1) as usize), list.as_mut_ptr(), nfiles );
                    i += 1;
                }
            } else if !(*search).dir.is_null() { // scan for files in the filesystem
                let mut netpath: *mut c_char;
                let mut numSysFiles: c_int = 0;
                let mut sysFiles: *mut *mut c_char;
                let mut name: *mut c_char;

                netpath = FS_BuildOSPath( (*(*search).dir).path.as_ptr(), (*(*search).dir).gamedir.as_ptr(), path );
                sysFiles = Sys_ListFiles( netpath, ext_ptr, &mut numSysFiles, QFALSE );
                i = 0;
                while i < numSysFiles {
                    // unique the match
                    name = *sysFiles.add(i as usize);
                    nfiles = FS_AddFileToList( name, list.as_mut_ptr(), nfiles );
                    i += 1;
                }
                Sys_FreeFileList( sysFiles );
            }
            search = (*search).next;
        }

        // return a copy of the list
        *numfiles = nfiles;

        if nfiles == 0 {
            return core::ptr::null_mut();
        }

        listCopy = Z_Malloc( ((nfiles + 1) as usize * core::mem::size_of::<*mut c_char>()) as usize, TAG_FILESYS, QFALSE) as *mut *mut c_char;
        i = 0;
        while i < nfiles {
            *listCopy.add(i as usize) = list[i as usize];
            i += 1;
        }
        *listCopy.add(i as usize) = core::ptr::null_mut();

        return listCopy;
    }
}

// =================
// FS_FreeFileList
// =================
pub fn FS_FreeFileList( filelist: *mut *mut c_char ) {
    let mut i: c_int = 0;

    unsafe {
        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if filelist.is_null() {
            return;
        }

        while !(*filelist.add(i as usize)).is_null() {
            Z_Free( *filelist.add(i as usize) as *mut c_void );
            i += 1;
        }

        Z_Free( filelist as *mut c_void );
    }
}

// ===============
// FS_AddFileToListBuf
// ===============
fn FS_AddFileToListBuf( name: *mut c_char, listbuf: *mut c_char, bufsize: c_int, nfiles: c_int ) -> c_int {
    let mut p: *mut c_char;

    unsafe {
        if nfiles == MAX_FOUND_FILES as c_int - 1 {
            return nfiles;
        }

        if *name == '/' as c_char || *name == '\\' as c_char {
            // SAFETY: name is a valid C string; skip the first char
        }

        p = listbuf;
        while *p != 0 {
            if stricmp( name, p ) == 0 {
                return nfiles;		// already in list
            }
            p = p.add(strlen( p ) + 1);
        }

        if ((p.add(strlen( name ) + 2) as isize - listbuf as isize) as c_int) > bufsize {
            return nfiles;		// list is full
        }

        strcpy( p, name );
        p = p.add(strlen( p ) + 1);
        *p = 0;

        return nfiles + 1;
    }
}

// ================
// FS_GetFileList
//
// Returns a uniqued list of files that match the given criteria
// from all search paths
// ================
pub fn FS_GetFileList(  path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int ) -> c_int {
    let mut nfiles: c_int;
    let mut search: *mut searchpath_t;
    let mut i: c_int;
    let mut pathLength: c_int;
    let mut extensionLength: c_int;
    let mut length: c_int;
    let mut pathDepth: c_int;
    let mut pak: *mut pack_t;
    let mut buildBuffer: *mut fileInPack_t;
    let mut zpath: [c_char; MAX_ZPATH] = [0; MAX_ZPATH];

    unsafe {
        if Q_stricmp(path, "$modlist\0".as_ptr() as *const c_char) == 0 {
            return FS_GetModList(listbuf, bufsize);
        }

        if fs_searchpaths.is_null() {
            Com_Error( ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
        }

        if path.is_null() {
            return 0;
        }

        let ext_ptr = if extension.is_null() { "\0".as_ptr() as *const c_char } else { extension };

        pathLength = strlen( path ) as c_int;
        extensionLength = strlen( ext_ptr ) as c_int;
        nfiles = 0;
        *listbuf = 0;
        FS_ReturnPath(path, zpath.as_mut_ptr(), &mut pathDepth);
        //
        // search through the path, one element at a time, adding to list
        //
        search = fs_searchpaths;
        while !search.is_null() {
            // is the element a pak file?
            if !(*search).pack.is_null() {
                // look through all the pak file elements
                pak = (*search).pack;
                buildBuffer = (*pak).buildBuffer;
                i = 0;
                while i < (*pak).numfiles {
                    let mut name: *mut c_char;
                    let mut zpathLen: c_int;
                    let mut depth: c_int = 0;

                    // check for directory match
                    name = (*buildBuffer.add(i as usize)).name;
                    zpathLen = FS_ReturnPath(name, zpath.as_mut_ptr(), &mut depth);

                    if (depth - pathDepth) > 2 || pathLength > zpathLen || Q_stricmpn( name, path, pathLength as usize ) != 0 {
                        i += 1;
                        continue;
                    }

                    // check for extension match
                    length = strlen( name ) as c_int;
                    if length < extensionLength || (length == (extensionLength + pathLength)) {
                        i += 1;
                        continue;
                    }

                    if stricmp( name.add((length - extensionLength) as usize), ext_ptr ) != 0 {
                        i += 1;
                        continue;
                    }

                    // unique the match
                    nfiles = FS_AddFileToListBuf( name.add(pathLength as usize), listbuf, bufsize, nfiles );
                    i += 1;
                }
            } else if !(*search).dir.is_null() { // scan for files in the filesystem
                let mut netpath: *mut c_char;
                let mut numSysFiles: c_int = 0;
                let mut sysFiles: *mut *mut c_char;
                let mut name: *mut c_char;

                netpath = FS_BuildOSPath( (*(*search).dir).path.as_ptr(), (*(*search).dir).gamedir.as_ptr(), path );
                sysFiles = Sys_ListFiles( netpath, ext_ptr, &mut numSysFiles, QFALSE );
                i = 0;
                while i < numSysFiles {
                    // unique the match
                    name = *sysFiles.add(i as usize);
                    nfiles = FS_AddFileToListBuf( name, listbuf, bufsize, nfiles );
                    i += 1;
                }
                Sys_FreeFileList( sysFiles );
            }
            search = (*search).next;
        }

        return nfiles;
    }
}

// ================
// FS_GetModList
//
// Returns a list of mod directory names
// A mod directory is a peer to base with a pk3 in it
//
// ================
pub fn FS_GetModList( listbuf: *mut c_char, bufsize: c_int ) -> c_int {
    let mut nMods: c_int;
    let mut i: c_int;
    let mut nTotal: c_int;
    let mut nLen: c_int;
    let mut nPaks: c_int;
    let mut nPotential: c_int;
    let mut nDescLen: c_int;
    let mut pFiles: *mut *mut c_char = core::ptr::null_mut();
    let mut pPaks: *mut *mut c_char = core::ptr::null_mut();
    let mut name: *mut c_char;
    let mut path: *mut c_char;
    let mut descPath: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut descHandle: fileHandle_t;

    unsafe {
        *listbuf = 0;
        nMods = 0;
        nPotential = 0;
        nTotal = 0;

        pFiles = Sys_ListFiles( (*fs_basepath).string, ".*\0".as_ptr() as *const c_char, &mut nPotential, QTRUE );
        i = 0;
        while i < nPotential {
            name = *pFiles.add(i as usize);
            if Q_stricmp(name, "base\0".as_ptr() as *const c_char) != 0 && Q_stricmpn(name, ".\0".as_ptr() as *const c_char, 1) != 0 {
                // ignore base
                path = FS_BuildOSPath( (*fs_basepath).string, name, "\0".as_ptr() as *const c_char );
                nPaks = 0;
                pPaks = Sys_ListFiles(path, ".pk3\0".as_ptr() as *const c_char, &mut nPaks, QFALSE);
                if nPaks > 0 {
                    nLen = strlen(name) as c_int + 1;
                    // nLen is the length of the mod path
                    // we need to see if there is a description available
                    descPath[0] = '\0' as c_char;
                    strcpy(descPath.as_mut_ptr(), name);
                    strcat(descPath.as_mut_ptr(), "/description.txt\0".as_ptr() as *const c_char);
                    nDescLen = FS_SV_FOpenFileRead( descPath.as_ptr(), &mut descHandle);
                    if nDescLen > 0 && descHandle != 0 {
                        let mut file: *mut FILE;
                        file = FS_FileForHandle(descHandle);
                        memset( descPath.as_mut_ptr() as *mut c_void, 0, core::mem::size_of::<[c_char; MAX_OSPATH]>() );
                        nDescLen = fread(descPath.as_mut_ptr() as *mut c_void, 1, 48, file) as c_int;
                        if nDescLen >= 0 {
                            *descPath.as_mut_ptr().add(nDescLen as usize) = '\0' as c_char;
                        }
                        FS_FCloseFile(descHandle);
                    } else {
                        strcpy(descPath.as_mut_ptr(), name);
                    }
                    nDescLen = strlen(descPath.as_ptr()) as c_int + 1;

                    if nTotal + nLen + 1 + nDescLen + 1 < bufsize {
                        strcpy(listbuf, name);
                        listbuf = listbuf.add(nLen as usize);
                        strcpy(listbuf, descPath.as_ptr());
                        listbuf = listbuf.add(nDescLen as usize);
                        nTotal += nLen + nDescLen;
                        nMods += 1;
                    } else {
                        break;
                    }
                }
                Sys_FreeFileList( pPaks );
            }
            i += 1;
        }
        Sys_FreeFileList( pFiles );

        return nMods;
    }
}

//============================================================================

// ================
// FS_Dir_f
// ================
pub fn FS_Dir_f( ) {
    let mut path: *const c_char;
    let mut extension: *const c_char;
    let mut dirnames: *mut *mut c_char;
    let mut ndirs: c_int = 0;
    let mut i: c_int;

    unsafe {
        if Cmd_Argc() < 2 || Cmd_Argc() > 3 {
            Com_Printf( "usage: dir <directory> [extension]\n\0".as_ptr() as *const c_char );
            return;
        }

        if Cmd_Argc() == 2 {
            path = Cmd_Argv( 1 );
            extension = "\0".as_ptr() as *const c_char;
        } else {
            path = Cmd_Argv( 1 );
            extension = Cmd_Argv( 2 );
        }

        Com_Printf( "Directory of %s %s\n\0".as_ptr() as *const c_char, path, extension );
        Com_Printf( "---------------\n\0".as_ptr() as *const c_char );

        dirnames = FS_ListFiles( path, extension, &mut ndirs );

        i = 0;
        while i < ndirs {
            Com_Printf( "%s\n\0".as_ptr() as *const c_char, *dirnames.add(i as usize) );
            i += 1;
        }
        FS_FreeFileList( dirnames );
    }
}

// ============
// FS_Path_f
//
// ============
pub fn FS_Path_f( ) {
    let mut s: *mut searchpath_t;
    let mut i: c_int;

    unsafe {
        Com_Printf ("Current search path:\n\0".as_ptr() as *const c_char);
        s = fs_searchpaths;
        while !s.is_null() {
            if !(*s).pack.is_null() {
                Com_Printf ("%s (%i files)\n\0".as_ptr() as *const c_char, (*(*s).pack).pakFilename.as_ptr(), (*(*s).pack).numfiles);
    /*			if ( fs_numServerPaks ) {
                    for ( i = 0 ; i < fs_numServerPaks ; i++ ) {
                        if ( s->pack->checksum == fs_serverPaks[i] ) {
                            break;		// on the aproved list
                        }
                    }
                    if ( i == fs_numServerPaks ) {
                        Com_Printf( "    not on the pure list\n" );
                    } else {
                        Com_Printf( "    on the pure list\n" );
                    }
                }
    */
            } else {
                Com_Printf ("%s/%s\n\0".as_ptr() as *const c_char, (*(*s).dir).path.as_ptr(), (*(*s).dir).gamedir.as_ptr() );
            }
            s = (*s).next;
        }


        Com_Printf( "\n\0".as_ptr() as *const c_char );
        i = 1;
        while i < MAX_FILE_HANDLES as c_int {
            if !(*core::ptr::addr_of!(fsh[i as usize])).handleFiles.file.o.is_null() {
                Com_Printf( "handle %i: %s\n\0".as_ptr() as *const c_char, i, (*core::ptr::addr_of!(fsh[i as usize])).name.as_ptr() );
            }
            i += 1;
        }
    }
}

// ============
// FS_TouchFile_f
//
// The only purpose of this function is to allow game script files to copy
// arbitrary files during an "fs_copyfiles 1" run.
// ============
pub fn FS_TouchFile_f( ) {
    let mut f: fileHandle_t;
    let count: c_int;

    unsafe {
        count = Cmd_Argc();

        if (count == 2) || (count == 3) {
            FS_FOpenFileRead( Cmd_Argv( 1 ), &mut f, QFALSE );
            if f != 0 {
                FS_FCloseFile( f );
            }
            if count == 3 {
                FS_FOpenFileRead( Cmd_Argv( 2 ), &mut f, QFALSE );
                if f != 0 {
                    FS_FCloseFile( f );
                }
            }
        } else {
            Com_Printf( "Usage: touchFile <file> [file2] -- You gave %d args!\n\0".as_ptr() as *const c_char, Cmd_Argc() );
        }
    }
}

//===========================================================================



fn paksort( a: *const c_void, b: *const c_void ) -> c_int {
    let mut aa: *const c_char;
    let mut bb: *const c_char;

    unsafe {
        aa = *(a as *const *const c_char);
        bb = *(b as *const *const c_char);

        return stricmp( aa, bb );
    }
}


// ================
// FS_AddGameDirectory
//
// Sets fs_gamedir, adds the directory to the head of the path,
// then loads the zip headers
// ================
const MAX_PAKFILES: usize = 1024;
fn FS_AddGameDirectory( path: *const c_char, dir: *const c_char ) {
    let mut i: c_int;
    let mut search: *mut searchpath_t;
    let mut pak: *mut pack_t;
    let mut pakfile: *mut c_char;
    let mut numfiles: c_int = 0;
    let mut pakfiles: *mut *mut c_char;
    let mut sorted: [*mut c_char; MAX_PAKFILES] = [core::ptr::null_mut(); MAX_PAKFILES];

    unsafe {
        Q_strncpyz( core::ptr::addr_of_mut!(fs_gamedir[0]), dir, core::mem::size_of::<[c_char; MAX_OSPATH]>() );

        //
        // add the directory to the search path
        //
        search = Z_Malloc (core::mem::size_of::<searchpath_t>(), TAG_FILESYS, QTRUE ) as *mut searchpath_t;
        (*search).dir = Z_Malloc( core::mem::size_of::<directory_t>(), TAG_FILESYS, QTRUE ) as *mut directory_t;
        (*search).pack = core::ptr::null_mut();
        Q_strncpyz( (*(*search).dir).path.as_mut_ptr(), path, core::mem::size_of::<[c_char; MAX_OSPATH]>() );
        Q_strncpyz( (*(*search).dir).gamedir.as_mut_ptr(), dir, core::mem::size_of::<[c_char; MAX_OSPATH]>() );
        (*search).next = fs_searchpaths;
        fs_searchpaths = search;

        Z_Label(search as *mut c_void, path);
        Z_Label((*search).dir as *mut c_void, dir);

        // find all pak files in this directory
        pakfile = FS_BuildOSPath( path, dir, "\0".as_ptr() as *const c_char );
        *pakfile.add(strlen(pakfile) - 1) = 0;	// strip the trailing slash

    #[cfg(feature = "PRE_RELEASE_DEMO")]
        {
            pakfile = FS_BuildOSPath( path, dir, "asset0.pksp\0".as_ptr() as *const c_char );
            if ( ( pak = FS_LoadZipFile( pakfile ) ) == core::ptr::null_mut() )
                {return;}
            if ( ((*pak).numfiles ^ 0x84268436u as c_int) != (DEMO_PAK_MAXFILES^ 0x84268436u as c_int)) {	//don't let them use the full version, even if renamed!
                return;}
            search = Z_Malloc(core::mem::size_of::<searchpath_t>(), TAG_FILESYS, QTRUE ) as *mut searchpath_t;
            (*search).pack = pak;
            (*search).dir = core::ptr::null_mut();
            (*search).next = fs_searchpaths;
            fs_searchpaths = search;
        }
    #[cfg(not(feature = "PRE_RELEASE_DEMO"))]
        {
            pakfiles = Sys_ListFiles( pakfile, ".pk3\0".as_ptr() as *const c_char, &mut numfiles, QFALSE );

            // sort them so that later alphabetic matches override
            // earlier ones.  This makes pak1.pk3 override asset0.pk3
            if numfiles > MAX_PAKFILES as c_int {
                numfiles = MAX_PAKFILES as c_int;
            }
            i = 0;
            while i < numfiles {
                sorted[i as usize] = *pakfiles.add(i as usize);
                i += 1;
            }

            qsort( sorted.as_mut_ptr() as *mut c_void, numfiles as usize, core::mem::size_of::<*mut c_char>(), paksort );

            i = 0;
            while i < numfiles {
                pakfile = FS_BuildOSPath( path, dir, sorted[i as usize] );
                if ( ( pak = FS_LoadZipFile( pakfile ) ) == core::ptr::null_mut() ) {
                    i += 1;
                    continue;
                }
                search = Z_Malloc(core::mem::size_of::<searchpath_t>(), TAG_FILESYS, QTRUE ) as *mut searchpath_t;
                (*search).pack = pak;
                (*search).dir = core::ptr::null_mut();
                (*search).next = fs_searchpaths;
                fs_searchpaths = search;
                i += 1;
            }

            // done
            Sys_FreeFileList( pakfiles );
        }
    }
}

// ================
// FS_Startup
// ================
pub fn FS_Startup( gameName: *const c_char ) {
    unsafe {
        Com_Printf( "----- FS_Startup -----\n\0".as_ptr() as *const c_char );

        fs_debug = Cvar_Get( "fs_debug\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, 0 );
        fs_copyfiles = Cvar_Get( "fs_copyfiles\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, CVAR_INIT );
        fs_cdpath = Cvar_Get ("fs_cdpath\0".as_ptr() as *const c_char, Sys_DefaultCDPath(), CVAR_INIT );
        fs_basepath = Cvar_Get ("fs_basepath\0".as_ptr() as *const c_char, Sys_DefaultBasePath(), CVAR_INIT );

        fs_gamedirvar = Cvar_Get ("fs_game\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, CVAR_INIT|CVAR_SERVERINFO );
        fs_restrict = Cvar_Get ("fs_restrict\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, CVAR_INIT );
        Cvar_Get( "com_demo\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, CVAR_INIT );

        // set up cdpath
        if (*fs_cdpath).string[0] as c_int != 0 {
            FS_AddGameDirectory ( (*fs_cdpath).string, gameName );
        }

        // set up basepath
        FS_AddGameDirectory ( (*fs_basepath).string, gameName );

        // check for game override
        if (*fs_gamedirvar).string[0] as c_int != 0 &&
            Q_stricmp( gameName, "base\0".as_ptr() as *const c_char ) == 0 &&
            Q_stricmp( (*fs_gamedirvar).string, gameName ) != 0 {
            if (*fs_cdpath).string[0] as c_int != 0 {
                FS_AddGameDirectory( (*fs_cdpath).string, (*fs_gamedirvar).string );
            }
            FS_AddGameDirectory( (*fs_basepath).string, (*fs_gamedirvar).string );
        }

        // add our commands
        Cmd_AddCommand ("path\0".as_ptr() as *const c_char, FS_Path_f);
        Cmd_AddCommand ("dir\0".as_ptr() as *const c_char, FS_Dir_f);
        Cmd_AddCommand ("touchFile\0".as_ptr() as *const c_char, FS_TouchFile_f);

        // print the current search paths
        FS_Path_f();

        Com_Printf( "----------------------\n\0".as_ptr() as *const c_char );
        Com_Printf( "%d files in pk3 files\n\0".as_ptr() as *const c_char, *core::ptr::addr_of!(fs_packFiles) );
    }
}

// ===================
// FS_SetRestrictions
//
// Looks for product keys and restricts media add on ability
// if the full version is not found
// ===================
pub fn FS_SetRestrictions( ) {
    let mut path: *mut searchpath_t;

#[cfg(not(feature = "PRE_RELEASE_DEMO"))]
    {
        let mut productId: *mut u8;

        unsafe {
            // if fs_restrict is set, don't even look for the id file,
            // which allows the demo release to be tested even if
            // the full game is present
            if (*(*fs_restrict)).integer == 0 {
                // look for the full game id
                FS_ReadFile( "productid.txt\0".as_ptr() as *const c_char, &mut (productId as *mut c_void) as *mut *mut c_void );
                if !productId.is_null() {
                    // check against the hardcoded string
                    let mut seed: u32;
                    let mut i: usize;

                    seed = 102270;
                    i = 0;
                    while i < fs_scrambledProductId.len() {
        #[cfg(debug_assertions)]
                        {
                            // fs_scrambledProductId[i]  = productId[i] ^ (seed&255);
                            // Com_Printf("%3i, ", fs_scrambledProductId[i]);
                        }
                        if ( ( fs_scrambledProductId[i] ^ ((seed & 255) as u8) ) != *productId.add(i) ) {
                            break;
                        }
                        seed = (69069 * seed + 1);
                        i += 1;
                    }

                    FS_FreeFile( productId as *mut c_void );

                    if i == fs_scrambledProductId.len() {
                        return;	// no restrictions
                    }
                    Com_Error( ERR_FATAL, "Invalid product identification\0".as_ptr() as *const c_char );
                }
            }
        }
    }

    unsafe {
        Cvar_Set( "fs_restrict\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char );
        Cvar_Set( "com_demo\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char );

        Com_Printf( "\nRunning in restricted demo mode.\n\n\0".as_ptr() as *const c_char );

        // restart the filesystem with just the demo directory
        FS_Shutdown();
        FS_Startup( "demo\0".as_ptr() as *const c_char );

        // make sure that the pak file has the header checksum we expect
        path = fs_searchpaths;
        while !path.is_null() {
            if !(*path).pack.is_null() {
                // a tiny attempt to keep the checksum from being scannable from the exe
                if ( ((*(*path).pack).checksum ^ 0x10228436i32) != (DEMO_PAK_CHECKSUM as c_int ^ 0x10228436i32) ) {
                    Com_Error( ERR_FATAL, "Corrupted pk3: %u\0".as_ptr() as *const c_char, (*(*path).pack).checksum );
                }
            }
            path = (*path).next;
        }
    }
}

// ================
// FS_Restart
// ================

pub fn FS_Restart( ) {
    unsafe {
        // free anything we currently have loaded
        FS_Shutdown();

        // try to start up normally
        FS_Startup( "base\0".as_ptr() as *const c_char );

        // see if we are going to allow add-ons
        FS_SetRestrictions();

        // if we can't find default.cfg, assume that the paths are
        // busted and error out now, rather than getting an unreadable
        // graphics screen when the font fails to load
        if FS_ReadFile( "default.cfg\0".as_ptr() as *const c_char, core::ptr::null_mut() ) <= 0 {
            Com_Error( ERR_FATAL, "Couldn\'t load default.cfg\0".as_ptr() as *const c_char );
        }
    }
}

// ========================================================================================
//
// Handle based file calls for virtual machines
//
// ========================================================================================

pub fn FS_FOpenFileByMode( qpath: *const c_char, f: *mut fileHandle_t, mode: c_int ) -> c_int {
    let mut r: c_int = 0;
    let mut sync: qboolean;

    sync = QFALSE;

    unsafe {
        match mode {
            FS_READ => {
                r = FS_FOpenFileRead( qpath, f, QTRUE );
            }
            FS_WRITE => {
                *f = FS_FOpenFileWrite( qpath );
                r = 0;
            }
            FS_APPEND_SYNC => {
                sync = QTRUE;
                *f = FS_FOpenFileAppend( qpath );
                r = 0;
            }
            FS_APPEND => {
                *f = FS_FOpenFileAppend( qpath );
                r = 0;
            }
            _ => {
                Com_Error( ERR_FATAL, "FSH_FOpenFile: bad mode\0".as_ptr() as *const c_char );
                return -1;
            }
        }

        if *f != 0 {
            if (*core::ptr::addr_of!(fsh[*f as usize])).zipFile == QTRUE {
                (*core::ptr::addr_of_mut!(fsh[*f as usize])).baseOffset = unztell((*core::ptr::addr_of!(fsh[*f as usize])).handleFiles.file.z);
            } else {
                (*core::ptr::addr_of_mut!(fsh[*f as usize])).baseOffset = ftell((*core::ptr::addr_of!(fsh[*f as usize])).handleFiles.file.o);
            }
            (*core::ptr::addr_of_mut!(fsh[*f as usize])).fileSize = r;
        }
        (*core::ptr::addr_of_mut!(fsh[*f as usize])).handleSync = sync;

        return r;
    }
}

pub fn FS_FTell( f: fileHandle_t ) -> c_int {
    let mut pos: c_int;
    unsafe {
        if (*core::ptr::addr_of!(fsh[f as usize])).zipFile == QTRUE {
            pos = unztell((*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.z);
        } else {
            pos = ftell((*core::ptr::addr_of!(fsh[f as usize])).handleFiles.file.o);
        }
    }
    return pos;
}
