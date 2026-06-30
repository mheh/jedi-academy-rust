// Filename:-	stringed_interface.h
//
// These are the functions that get replaced by game-specific ones (or StringEd code) so SGE can access files etc
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// #pragma warning ( disable : 4786 )			// disable the usual stupid and pointless STL warning
// #include <string>
// using namespace std;

use core::ffi::{c_char, c_int, c_uchar};

// C++ std::string — canonical home of this type for the StringEd port.
pub type std_string = String;

unsafe extern "C" {
    pub fn SE_LoadFileData(psFileName: *const c_char, piLoadedLength: *mut c_int) -> *mut c_uchar;
    pub fn SE_FreeFileDataAfterLoad(psLoadedFile: *mut c_uchar);
    pub fn SE_BuildFileList(psStartDir: *const c_char, strResults: *mut std_string) -> c_int;
}

// ////////////////// eof ///////////////////
