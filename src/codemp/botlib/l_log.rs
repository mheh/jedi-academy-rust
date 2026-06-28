
/*****************************************************************************
 * name:		l_log.c
 *
 * desc:		log file
 *
 * $Archive: /MissionPack/CODE/botlib/l_log.c $
 * $Author: Raduffy $
 * $Revision: 1 $
 * $Modtime: 12/20/99 8:43p $
 * $Date: 3/08/00 11:28a $
 *
 *****************************************************************************/

use core::ffi::{c_char, c_int};
use std::ptr::{addr_of, addr_of_mut, null_mut};

const MAX_LOGFILENAMESIZE: usize = 1024;

#[repr(C)]
pub struct logfile_s {
    pub filename: [c_char; MAX_LOGFILENAMESIZE],
    pub fp: *mut libc::FILE,
    pub numwrites: c_int,
}

static mut logfile: logfile_s = logfile_s {
    filename: [0; MAX_LOGFILENAMESIZE],
    fp: null_mut(),
    numwrites: 0,
};

extern "C" {
    fn LibVarValue(name: *const c_char, default: *const c_char) -> *const c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut libc::FILE;
    fn fclose(fp: *mut libc::FILE) -> c_int;
    fn fprintf(fp: *mut libc::FILE, fmt: *const c_char, ...) -> c_int;
    fn vfprintf(fp: *mut libc::FILE, fmt: *const c_char, ap: *mut libc::va_list) -> c_int;
    fn fflush(fp: *mut libc::FILE) -> c_int;
}

// External engine dependencies
extern "C" {
    // Placeholder stubs for external dependencies that would be defined elsewhere
    static botimport: bot_import_t;
    static mut botlibglobals: botlib_globals_t;
}

#[repr(C)]
pub struct bot_import_t {
    // Stub structure - actual definition would be in be_interface.h equivalent
    // Contains Print function pointer and other import functions
}

#[repr(C)]
pub struct botlib_globals_t {
    pub time: f32,
    // Other fields would be defined in actual header
}

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn Log_Open(filename: *mut c_char) {
    if LibVarValue(b"log\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char).is_null() {
        return;
    }
    if filename.is_null() || strlen(filename) == 0 {
        // botimport.Print(PRT_MESSAGE, "openlog <filename>\n");
        return;
    } //end if
    if !(*addr_of!(logfile).fp).is_null() {
        // botimport.Print(PRT_ERROR, "log file %s is already opened\n", logfile.filename);
        return;
    } //end if

    let logfile_mut = addr_of_mut!(logfile);
    (*logfile_mut).fp = fopen(filename, b"wb\0".as_ptr() as *const c_char);
    if (*logfile_mut).fp.is_null() {
        // botimport.Print(PRT_ERROR, "can't open the log file %s\n", filename);
        return;
    } //end if
    strncpy((*logfile_mut).filename.as_mut_ptr(), filename, MAX_LOGFILENAMESIZE);
    // botimport.Print(PRT_MESSAGE, "Opened log %s\n", logfile.filename);
} //end of the function Log_Create

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn Log_Close() {
    if (*addr_of!(logfile).fp).is_null() {
        return;
    }
    if fclose((*addr_of!(logfile).fp)) != 0 {
        // botimport.Print(PRT_ERROR, "can't close log file %s\n", logfile.filename);
        return;
    } //end if
    let logfile_mut = addr_of_mut!(logfile);
    (*logfile_mut).fp = null_mut();
    // botimport.Print(PRT_MESSAGE, "Closed log %s\n", logfile.filename);
} //end of the function Log_Close

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn Log_Shutdown() {
    if !(*addr_of!(logfile).fp).is_null() {
        Log_Close();
    }
} //end of the function Log_Shutdown

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
// NOTE: Rust does not support defining variadic functions in stable Rust.
// This function signature preserves the C ABI intent but requires C code
// or nightly c_variadic feature for full varargs implementation.
// The original C code used va_start/va_end with vfprintf.
pub unsafe extern "C" fn Log_Write(fmt: *const c_char) {
    if (*addr_of!(logfile).fp).is_null() {
        return;
    }
    // Port limitation: varargs handling requires nightly c_variadic feature
    // Actual implementation would use va_list/vfprintf as in original C code
} //end of the function Log_Write

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
// NOTE: Rust does not support defining variadic functions in stable Rust.
// This function signature preserves the C ABI intent but requires C code
// or nightly c_variadic feature for full varargs implementation.
// The original C code:
// - fprintf(logfile.fp, "%d   %02d:%02d:%02d:%02d   ",
//            logfile.numwrites,
//            (int) (botlibglobals.time / 60 / 60),
//            (int) (botlibglobals.time / 60),
//            (int) (botlibglobals.time),
//            (int) ((int) (botlibglobals.time * 100)) -
//                  ((int) botlibglobals.time) * 100);
// - va_start(ap, fmt)
// - vfprintf(logfile.fp, fmt, ap)
// - va_end(ap)
// - fprintf(logfile.fp, "\r\n")
// - logfile.numwrites++
// - fflush(logfile.fp)
pub unsafe extern "C" fn Log_WriteTimeStamped(fmt: *const c_char) {
    if (*addr_of!(logfile).fp).is_null() {
        return;
    }
    // Port limitation: varargs handling requires nightly c_variadic feature
    // Actual implementation would compute timestamps and call vfprintf as in original C code
} //end of the function Log_Write

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn Log_FilePointer() -> *mut libc::FILE {
    (*addr_of!(logfile)).fp
} //end of the function Log_FilePointer

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn Log_Flush() {
    if !(*addr_of!(logfile).fp).is_null() {
        fflush((*addr_of!(logfile)).fp);
    }
} //end of the function Log_Flush
