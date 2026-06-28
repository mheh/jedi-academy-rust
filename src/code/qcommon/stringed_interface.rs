// Filename:-	stringed_interface.cpp
//
// This file contains functions that StringEd wants to call to do things like load/save, they can be modified
//	for use ingame, but must remain functionally the same...
//
//  Please try and put modifications for whichever games this is used for inside #defines, so I can copy the same file
//		into each project.
//

use core::ffi::{c_int, c_char, c_void};

// stuff common to all qcommon files...
// #include "../server/server.h"
// #include "../game/q_shared.h"
// #include "qcommon.h"

// #pragma warning ( disable : 4511 )			// copy constructor could not be generated
// #pragma warning ( disable : 4512 )			// assignment operator could not be generated
// #pragma warning ( disable : 4663 )			// C++ language change: blah blah template crap blah blah
// #include "stringed_interface.h"
// #include "stringed_ingame.h"

// #include <string>
// using namespace std;

// Stub for c_uchar type
type c_uchar = u8;

// Conditional compilation: _STRINGED
// #ifdef _STRINGED
// #include <stdlib.h>
// #include <memory.h>
// #include "generic.h"
// #endif

// External functions declared for the game engine and C standard library
extern "C" {
	// File system functions (game engine)
	fn FS_ReadFile(filename: *const c_char, buffer: *mut *mut c_void) -> c_int;
	fn FS_FreeFile(buffer: *mut c_void);
	fn FS_ListFiles(path: *const c_char, extension: *const c_char, numfiles: *mut c_int) -> *mut *mut c_char;
	fn FS_FreeFileList(list: *mut *mut c_char);

	// Standard C library functions (always available, but only used conditionally)
	fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
	fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
	fn fclose(stream: *mut c_void) -> c_int;
	fn malloc(size: usize) -> *mut c_void;
	fn free(ptr: *mut c_void);
	fn filesize(stream: *mut c_void) -> c_int;

	// String functions
	fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
	fn strlen(s: *const c_char) -> usize;

	// Utility functions
	fn va(format: *const c_char, ...) -> *mut c_char;
}

// this just gets the binary of the file into memory, so I can parse it. Called by main SGE loader
//
//  returns either char * of loaded file, else NULL for failed-to-open...
//
#[no_mangle]
pub unsafe extern "C" fn SE_LoadFileData(psFileName: *const c_char, piLoadedLength: *mut c_int) -> *mut c_uchar {
	let mut psReturn: *mut c_uchar = std::ptr::null_mut();
	if !piLoadedLength.is_null() {
		*piLoadedLength = 0;
	}

	// #ifdef _STRINGED
	// if (psFileName[1] == ':') { ... } else { ... }
	// #else
	// { ... }
	// #endif
	#[cfg(feature = "_STRINGED")]
	{
		if !psFileName.is_null() && *psFileName.offset(1) as u8 as char == ':' {
			// full-path filename...
			//
			let fh = fopen(psFileName, "rb\0".as_ptr() as *const c_char);
			if !fh.is_null() {
				let lLength = filesize(fh as *mut c_void);

				if lLength > 0 {
					psReturn = malloc((lLength as usize) + 1) as *mut c_uchar;
					if !psReturn.is_null() {
						let iBytesRead = fread(psReturn as *mut c_void, 1, lLength as usize, fh);
						if iBytesRead as c_int != lLength {
							// error reading file!!!...
							//
							free(psReturn as *mut c_void);
							psReturn = std::ptr::null_mut();
						} else {
							*psReturn.offset(lLength as isize) = 0; // '\0'
							if !piLoadedLength.is_null() {
								*piLoadedLength = iBytesRead as c_int;
							}
						}
						fclose(fh);
					}
				}
			}
		} else {
			// local filename, so prepend the base dir etc according to game and load it however (from PAK?)
			//
			let mut pvLoadedData: *mut c_void = std::ptr::null_mut();
			let iLen = FS_ReadFile(psFileName, &mut pvLoadedData);

			if iLen > 0 {
				psReturn = pvLoadedData as *mut c_uchar;
				if !piLoadedLength.is_null() {
					*piLoadedLength = iLen;
				}
			}
		}
	}

	#[cfg(not(feature = "_STRINGED"))]
	{
		// local filename, so prepend the base dir etc according to game and load it however (from PAK?)
		//
		let mut pvLoadedData: *mut c_void = std::ptr::null_mut();
		let iLen = FS_ReadFile(psFileName, &mut pvLoadedData);

		if iLen > 0 {
			psReturn = pvLoadedData as *mut c_uchar;
			if !piLoadedLength.is_null() {
				*piLoadedLength = iLen;
			}
		}
	}

	psReturn
}


// called by main SGE code after loaded data has been parsedinto internal structures...
//
#[no_mangle]
pub unsafe extern "C" fn SE_FreeFileDataAfterLoad(psLoadedFile: *mut c_uchar) {
	// #ifdef _STRINGED
	#[cfg(feature = "_STRINGED")]
	{
		if !psLoadedFile.is_null() {
			free(psLoadedFile as *mut c_void);
		}
	}
	// #else
	#[cfg(not(feature = "_STRINGED"))]
	{
		if !psLoadedFile.is_null() {
			FS_FreeFile(psLoadedFile as *mut c_void);
		}
	}
	// #endif
}

// #ifndef _STRINGED
// quake-style method of doing things since their file-list code doesn't have a 'recursive' flag...
//
#[cfg(not(feature = "_STRINGED"))]
pub static mut giFilesFound: c_int = 0;

// Porting note: SE_R_ListFiles is a local static function, only used in !_STRINGED builds
#[cfg(not(feature = "_STRINGED"))]
unsafe fn SE_R_ListFiles(psExtension: *const c_char, psDir: *const c_char, strResults: &mut String) {
	//	Com_Printf(va("Scanning Dir: %s\n",psDir));

	let mut sysFiles: *mut *mut c_char;
	let mut dirFiles: *mut *mut c_char;
	let mut numSysFiles: c_int = 0;
	let mut i: c_int = 0;
	let mut numdirs: c_int = 0;

	dirFiles = FS_ListFiles(psDir, "/\0".as_ptr() as *const c_char, &mut numdirs);
	i = 0;
	while i < numdirs {
		if !(*dirFiles.offset(i as isize)).is_null() && *(*dirFiles.offset(i as isize)) as u8 as char != '\0' && *(*dirFiles.offset(i as isize)) as u8 as char != '.' {
			// skip blanks, plus ".", ".." etc
			let mut sDirName: [c_char; 260] = [0; 260]; // MAX_QPATH = 260
			sprintf(
				sDirName.as_mut_ptr(),
				"%s/%s\0".as_ptr() as *const c_char,
				psDir,
				*dirFiles.offset(i as isize)
			);
			//
			// for some reason the quake filesystem in this game now returns an extra slash on the end,
			//	didn't used to. Sigh...
			//
			let sDirNameLen = strlen(sDirName.as_ptr());
			if sDirNameLen > 0 && sDirName[sDirNameLen - 1] as u8 as char == '/' {
				sDirName[sDirNameLen - 1] = 0; // '\0'
			}
			SE_R_ListFiles(psExtension, sDirName.as_ptr(), strResults);
		}
		i += 1;
	}

	sysFiles = FS_ListFiles(psDir, psExtension, &mut numSysFiles);
	i = 0;
	while i < numSysFiles {
		let mut sFilename: [c_char; 260] = [0; 260]; // MAX_QPATH = 260
		sprintf(
			sFilename.as_mut_ptr(),
			"%s/%s\0".as_ptr() as *const c_char,
			psDir,
			*sysFiles.offset(i as isize)
		);

		//		Com_Printf("%sFound file: %s",!i?"\n":"",sFilename);

		// Porting note: Converting C string to Rust String for concatenation
		if let Ok(filename_cstr) = std::ffi::CStr::from_ptr(sFilename.as_ptr()).to_str() {
			strResults.push_str(filename_cstr);
			strResults.push(';');
			giFilesFound += 1;
		}

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
// #endif

// replace this with a call to whatever your own code equivalent is.
//
// expected result is a ';'-delineated string (including last one) containing file-list search results
//
#[no_mangle]
pub unsafe extern "C" fn SE_BuildFileList(psStartDir: *const c_char, strResults: &mut String) -> c_int {
	// #ifndef _STRINGED
	#[cfg(not(feature = "_STRINGED"))]
	{
		giFilesFound = 0;
		strResults.clear();

		// Porting note: Rust String used instead of std::string
		// The extern "C" signature accepts a mutable reference; in FFI this is handled carefully
		SE_R_ListFiles(
			".str\0".as_ptr() as *const c_char, // sSE_INGAME_FILE_EXTENSION
			psStartDir,
			strResults
		);

		return giFilesFound;
	}
	// #else
	#[cfg(feature = "_STRINGED")]
	{
		// .ST files...
		//
		let _iFilesFound = 0; // Porting note: BuildFileList is not declared; this is a stub

		// extern string strResult;
		// strResults = strResult;
		// Porting note: In _STRINGED mode, this would call external BuildFileList
		// which is not available in this port context

		return _iFilesFound;
	}
	// #endif
}

/////////////////////// eof ///////////////////////
