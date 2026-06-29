// Filename: stringed_interface.rs
//
// This file contains functions that StringEd wants to call to do things like load/save, they can be modified
// for use ingame, but must remain functionally the same...
//
// Please try and put modifications for whichever games this is used for inside #defines, so I can copy the same file
// into each project.
//

use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr::{addr_of_mut, null_mut};

// Opaque type representing C++ std::string
// Full definition requires C++ runtime; used for signature compatibility
#[repr(C)]
pub struct CppString {
    _opaque: [u8; 0],
}

// Extern C declarations for game filesystem
extern "C" {
    fn FS_ReadFile(name: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn FS_ListFiles(path: *const c_char, extension: *const c_char, numfiles: *mut c_int) -> *mut *mut c_char;
    fn FS_FreeFileList(fileList: *mut *mut c_char);
    fn va(fmt: *const c_char, ...) -> *mut c_char;
}

// C stdlib externs for _STRINGED mode
#[cfg(feature = "_STRINGED")]
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn fopen(path: *const c_char, mode: *const c_char) -> *mut c_void;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    fn fclose(stream: *mut c_void) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn filesize(fh: *mut c_void) -> c_long;
    fn sprintf(str: *mut c_char, format: *const c_char, ...) -> c_int;
}

// Global variable for file counting in quake-style recursion
#[cfg(not(feature = "_STRINGED"))]
static mut giFilesFound: c_int = 0;

// Constants (stub - actual values from headers)
#[allow(dead_code)]
const MAX_QPATH: usize = 256;

// this just gets the binary of the file into memory, so I can parse it. Called by main SGE loader
//
// returns either char * of loaded file, else NULL for failed-to-open...
//
pub unsafe extern "C" fn SE_LoadFileData(
    psFileName: *const c_char,
    piLoadedLength: *mut c_int,
) -> *mut u8 {
    let mut psReturn: *mut u8 = null_mut();
    if !piLoadedLength.is_null() {
        *piLoadedLength = 0;
    }

    #[cfg(feature = "_STRINGED")]
    {
        if !psFileName.is_null() && *psFileName.add(1) as c_int == b':' as c_int {
            // full-path filename...
            //
            let fh = fopen(psFileName, b"rb\0".as_ptr() as *const c_char);
            if !fh.is_null() {
                let lLength = filesize(fh);

                if lLength > 0 {
                    psReturn = malloc((lLength + 1) as usize) as *mut u8;
                    if !psReturn.is_null() {
                        let iBytesRead = fread(psReturn as *mut c_void, 1, lLength as usize, fh);
                        if iBytesRead as c_long != lLength {
                            // error reading file!!!...
                            //
                            free(psReturn as *mut c_void);
                            psReturn = null_mut();
                        } else {
                            *psReturn.add(lLength as usize) = b'\0';
                            if !piLoadedLength.is_null() {
                                *piLoadedLength = iBytesRead as c_int;
                            }
                        }
                        fclose(fh);
                    }
                }
            }
        } else {
            // Fallthrough to non-_STRINGED path
            let mut pvLoadedData: *mut c_void = null_mut();
            let iLen = FS_ReadFile(psFileName, addr_of_mut!(pvLoadedData));

            if iLen > 0 {
                psReturn = pvLoadedData as *mut u8;
                if !piLoadedLength.is_null() {
                    *piLoadedLength = iLen;
                }
            }
        }
    }
    #[cfg(not(feature = "_STRINGED"))]
    {
        // local filename, so prepend the base dir etc according to game and load it however (from PAK?)
        let mut pvLoadedData: *mut c_void = null_mut();
        let iLen = FS_ReadFile(psFileName, addr_of_mut!(pvLoadedData));

        if iLen > 0 {
            psReturn = pvLoadedData as *mut u8;
            if !piLoadedLength.is_null() {
                *piLoadedLength = iLen;
            }
        }
    }

    psReturn
}

// called by main SGE code after loaded data has been parsedinto internal structures...
//
pub unsafe extern "C" fn SE_FreeFileDataAfterLoad(psLoadedFile: *mut u8) {
    #[cfg(feature = "_STRINGED")]
    {
        if !psLoadedFile.is_null() {
            free(psLoadedFile as *mut c_void);
        }
    }
    #[cfg(not(feature = "_STRINGED"))]
    {
        if !psLoadedFile.is_null() {
            FS_FreeFile(psLoadedFile as *mut c_void);
        }
    }
}

#[cfg(not(feature = "_STRINGED"))]
// quake-style method of doing things since their file-list code doesn't have a 'recursive' flag...
//
unsafe fn SE_R_ListFiles(
    psExtension: *const c_char,
    psDir: *const c_char,
    strResults: *mut CppString,
) {
    // Com_Printf(va("Scanning Dir: %s\n",psDir));

    let mut sysFiles: *mut *mut c_char;
    let mut dirFiles: *mut *mut c_char;
    let mut numSysFiles: c_int;
    let mut numdirs: c_int = 0;

    dirFiles = FS_ListFiles(psDir, b"/\0".as_ptr() as *const c_char, addr_of_mut!(numdirs));
    let mut i = 0;
    while i < numdirs {
        if !(*dirFiles.add(i as usize)).is_null()
            && *(*dirFiles.add(i as usize)) as c_int != 0
            && *(*dirFiles.add(i as usize)) as c_int != '.' as c_int
        {
            // skip blanks, plus ".", ".." etc
            let mut sDirName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            sprintf(
                sDirName.as_mut_ptr(),
                b"%s/%s\0".as_ptr() as *const c_char,
                psDir,
                *dirFiles.add(i as usize),
            );
            //
            // for some reason the quake filesystem in this game now returns an extra slash on the end,
            // didn't used to. Sigh...
            //
            let slen = strlen(sDirName.as_ptr());
            if slen > 0 && sDirName[slen - 1] as u8 == b'/' {
                sDirName[slen - 1] = 0;
            }
            SE_R_ListFiles(psExtension, sDirName.as_ptr(), strResults);
        }
        i += 1;
    }

    numSysFiles = 0;
    sysFiles = FS_ListFiles(psDir, psExtension, addr_of_mut!(numSysFiles));
    i = 0;
    while i < numSysFiles {
        let mut sFilename: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        sprintf(
            sFilename.as_mut_ptr(),
            b"%s/%s\0".as_ptr() as *const c_char,
            psDir,
            *sysFiles.add(i as usize),
        );

        // Com_Printf("%sFound file: %s",!i?"\n":"",sFilename);

        // strResults += sFilename;
        // strResults += ';';
        // giFilesFound++;

        // read it in...
        //
        /*		byte *pbData = NULL;
        int iSize = FS_ReadFile( sFilename, (void **)&pbData);

        if (pbData)
        {

            FS_FreeFile( pbData );
        }
        */
        i += 1;
    }
    FS_FreeFileList(sysFiles);
    FS_FreeFileList(dirFiles);
}

// replace this with a call to whatever your own code equivalent is.
//
// expected result is a ';'-delineated string (including last one) containing file-list search results
//
#[allow(non_snake_case)]
pub unsafe fn SE_BuildFileList(psStartDir: *const c_char, strResults: *mut CppString) -> c_int {
    #[cfg(not(feature = "_STRINGED"))]
    {
        // giFilesFound = 0;
        // strResults = "";

        // SE_R_ListFiles( sSE_INGAME_FILE_EXTENSION, psStartDir, strResults );

        // return giFilesFound;

        // Stub: full implementation requires access to sSE_INGAME_FILE_EXTENSION constant
        // and C++ std::string mutation via reference
        0
    }
    #[cfg(feature = "_STRINGED")]
    {
        // .ST files...
        //
        // int iFilesFound = BuildFileList(	va("%s\\*%s",psStartDir, sSE_INGAME_FILE_EXTENSION),	// LPCSTR psPathAndFilter,
        //										true					// bool bRecurseSubDirs
        //										);

        // extern string strResult;
        // strResults = strResult;
        // return iFilesFound;

        // Stub: full implementation requires BuildFileList from generic.h and external strResult
        0
    }
}

/////////////////////// eof ///////////////////////
