/*****************************************************************************
 * name:		l_log.h
 *
 * desc:		log file
 *
 * $Archive: /source/code/botlib/l_log.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::c_char;

extern "C" {
    // open a log file
    pub fn Log_Open(filename: *const c_char);
    // close the current log file
    pub fn Log_Close();
    // close log file if present
    pub fn Log_Shutdown();
    // write to the current opened log file
    pub fn Log_Write(fmt: *const c_char, ...);
    // write to the current opened log file with a time stamp
    pub fn Log_WriteTimeStamped(fmt: *const c_char, ...);
    // returns a pointer to the log file
    pub fn Log_FilePointer() -> *mut core::ffi::c_void;
    // flush log file
    pub fn Log_Flush();
}
