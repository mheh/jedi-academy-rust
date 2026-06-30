// Filename:-	stringed_interface.cpp
//
// This file contains functions that StringEd wants to call to do things like load/save, they can be modified
//	for use ingame, but must remain functionally the same...
//
//  Please try and put modifications for whichever games this is used for inside #defines, so I can copy the same file
//		into each project.
//

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

//////////////////////////////////////////////////
//
// stuff common to all qcommon files...
use crate::codemp::server::server_h::*;
use crate::codemp::game::q_shared_h::*;
use crate::codemp::qcommon::qcommon_h::*;
//
//////////////////////////////////////////////////

// #pragma warning ( disable : 4511 )			// copy constructor could not be generated
// #pragma warning ( disable : 4512 )			// assignment operator could not be generated
// #pragma warning ( disable : 4663 )			// C++ language change: blah blah template crap blah blah
use crate::codemp::qcommon::stringed_interface_h::*;
use crate::codemp::qcommon::stringed_ingame_h::*;

// <string> / using namespace std; — std_string (= String) is imported from stringed_interface_h

#[cfg(feature = "stringed")]
use crate::codemp::qcommon::generic_h::*;

use core::ffi::{c_char, c_int, c_long, c_uchar, c_void};
use core::ptr::{addr_of, addr_of_mut, null_mut};

// sprintf and strlen — used in SE_R_ListFiles (non-_STRINGED path); from C stdlib
extern "C" {
    fn sprintf(str_: *mut c_char, format: *const c_char, ...) -> c_int;
    fn strlen(s: *const c_char) -> usize;
}

// malloc, free, fopen, fread, fclose, filesize — used only in _STRINGED path; from C stdlib
#[cfg(feature = "stringed")]
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn fopen(path: *const c_char, mode: *const c_char) -> *mut c_void;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    fn fclose(stream: *mut c_void) -> c_int;
    fn filesize(fh: *mut c_void) -> c_long;
}

// 'extern string strResult;' declared inside SE_BuildFileList in C++ — lifted to module scope as Rust
// does not allow extern blocks inside function bodies.
// Porting note: std_string (= String) is not FFI-safe; this is a best-effort faithful translation.
#[cfg(feature = "stringed")]
extern "C" {
    static strResult: std_string;
}

#[cfg(not(feature = "stringed"))]
// quake-style method of doing things since their file-list code doesn't have a 'recursive' flag...
//
pub static mut giFilesFound: c_int = 0;


// this just gets the binary of the file into memory, so I can parse it. Called by main SGE loader
//
//  returns either char * of loaded file, else NULL for failed-to-open...
//
pub unsafe extern "C" fn SE_LoadFileData(
    psFileName: *const c_char,
    piLoadedLength: *mut c_int, /* = 0 */
) -> *mut c_uchar {
    let mut psReturn: *mut c_uchar = null_mut();
    if !piLoadedLength.is_null() {
        *piLoadedLength = 0;
    }

    // Porting note: C's '#ifdef _STRINGED ... else' cannot be expressed as #[cfg] on an else arm.
    // Duplicated into two cfg blocks (one for each feature state) per AGENT_PROMPT section 3.
    #[cfg(feature = "stringed")]
    {
        if *psFileName.add(1) == b':' as c_char {
            // full-path filename...
            //
            let fh = fopen(psFileName, b"rb\0".as_ptr() as *const c_char);
            if !fh.is_null() {
                let lLength: c_long = filesize(fh);

                if lLength > 0 {
                    psReturn = malloc((lLength + 1) as usize) as *mut c_uchar;
                    if !psReturn.is_null() {
                        let iBytesRead: c_int =
                            fread(psReturn as *mut c_void, 1, lLength as usize, fh) as c_int;
                        if iBytesRead as c_long != lLength {
                            // error reading file!!!...
                            //
                            free(psReturn as *mut c_void);
                                 psReturn = null_mut();
                        } else {
                            *psReturn.add(lLength as usize) = b'\0';
                            if !piLoadedLength.is_null() {
                                *piLoadedLength = iBytesRead;
                            }
                        }
                        fclose(fh);
                    }
                }
            }
        } else {
            // local filename, so prepend the base dir etc according to game and load it however (from PAK?)
            //
            let mut pvLoadedData: *mut c_uchar = null_mut();
            let iLen: c_int =
                FS_ReadFile(psFileName, addr_of_mut!(pvLoadedData) as *mut *mut c_void);

            if iLen > 0 {
                psReturn = pvLoadedData;
                if !piLoadedLength.is_null() {
                    *piLoadedLength = iLen;
                }
            }
        }
    }
    #[cfg(not(feature = "stringed"))]
    {
        // local filename, so prepend the base dir etc according to game and load it however (from PAK?)
        //
        let mut pvLoadedData: *mut c_uchar = null_mut();
        let iLen: c_int =
            FS_ReadFile(psFileName, addr_of_mut!(pvLoadedData) as *mut *mut c_void);

        if iLen > 0 {
            psReturn = pvLoadedData;
            if !piLoadedLength.is_null() {
                *piLoadedLength = iLen;
            }
        }
    }

    psReturn
}


// called by main SGE code after loaded data has been parsedinto internal structures...
//
pub unsafe extern "C" fn SE_FreeFileDataAfterLoad(psLoadedFile: *mut c_uchar) {
    #[cfg(feature = "stringed")]
    {
        if !psLoadedFile.is_null() {
            free(psLoadedFile as *mut c_void);
        }
    }
    #[cfg(not(feature = "stringed"))]
    {
        if !psLoadedFile.is_null() {
            FS_FreeFile(psLoadedFile as *mut c_void);
        }
    }
}


#[cfg(not(feature = "stringed"))]
// quake-style method of doing things since their file-list code doesn't have a 'recursive' flag...
//
unsafe fn SE_R_ListFiles(
    psExtension: *const c_char,
    psDir: *const c_char,
    strResults: *mut std_string,
) {
//	Com_Printf(va("Scanning Dir: %s\n",psDir));

    let mut sysFiles: *mut *mut c_char = null_mut();
    let mut dirFiles: *mut *mut c_char = null_mut();
    let mut numSysFiles: c_int = 0;
    let mut i: c_int;
    let mut numdirs: c_int = 0;

    dirFiles = FS_ListFiles(psDir, b"/\0".as_ptr() as *const c_char, addr_of_mut!(numdirs));
    i = 0;
    while i < numdirs {
        if *(*dirFiles.add(i as usize)) != 0 as c_char
            && *(*dirFiles.add(i as usize)) != b'.' as c_char // skip blanks, plus ".", ".." etc
        {
            let mut sDirName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            sprintf(
                sDirName.as_mut_ptr(),
                b"%s/%s\0".as_ptr() as *const c_char,
                psDir,
                *dirFiles.add(i as usize),
            );
            //
            // for some reason the quake filesystem in this game now returns an extra slash on the end,
            //	didn't used to. Sigh...
            //
            if sDirName[strlen(sDirName.as_ptr()) - 1] == b'/' as c_char {
                sDirName[strlen(sDirName.as_ptr()) - 1] = 0;
            }
            SE_R_ListFiles(psExtension, sDirName.as_ptr(), strResults);
        }
        i += 1;
    }

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

//		Com_Printf("%sFound file: %s",!i?"\n":"",sFilename);

        // Porting note: C++ 'strResults += sFilename' appends a const char* to a std::string.
        // With std_string = String, we convert via CStr then push_str.
        (*strResults).push_str(
            core::ffi::CStr::from_ptr(sFilename.as_ptr())
                .to_string_lossy()
                .as_ref(),
        );
        // strResults += ';'
        (*strResults).push(';');
        *addr_of_mut!(giFilesFound) += 1;

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
pub unsafe extern "C" fn SE_BuildFileList(
    psStartDir: *const c_char,
    strResults: *mut std_string,
) -> c_int {
    #[cfg(not(feature = "stringed"))]
    {
        *addr_of_mut!(giFilesFound) = 0;
        // strResults = ""
        *strResults = std_string::new();

        // Porting note: sSE_INGAME_FILE_EXTENSION is &str; as_ptr() passes the string data as
        // *const c_char. This relies on the string literal being followed by a null byte in storage.
        SE_R_ListFiles(
            sSE_INGAME_FILE_EXTENSION.as_ptr() as *const c_char,
            psStartDir,
            strResults,
        );

        return *addr_of!(giFilesFound);
    }
    #[cfg(feature = "stringed")]
    {
        // .ST files...
        //
        let iFilesFound: c_int = BuildFileList(
            va(
                b"%s\\*%s\0".as_ptr() as *const c_char, // LPCSTR psPathAndFilter,
                psStartDir,
                sSE_INGAME_FILE_EXTENSION.as_ptr() as *const c_char,
            ),
            true, // bool bRecurseSubDirs
        );

        // 'extern string strResult;' (C++ local extern declaration) is lifted to module scope above.
        // Porting note: core::ptr::read of a non-FFI-safe String — best-effort faithful translation.
        *strResults = core::ptr::read(addr_of!(strResult));
        return iFilesFound;
    }
}

/////////////////////// eof ///////////////////////
