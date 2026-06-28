#![allow(non_snake_case)]

/*****************************************************************************
 * name:		be_aas_file.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_file.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::{c_char, c_int};

// qboolean is typically typedef'd to int in C
pub type qboolean = c_int;

#[cfg(feature = "aasintern")]
pub extern "C" {
    // loads the AAS file with the given name
    pub fn AAS_LoadAASFile(filename: *mut c_char) -> c_int;
    // writes an AAS file with the given name
    pub fn AAS_WriteAASFile(filename: *mut c_char) -> qboolean;
    // dumps the loaded AAS data
    pub fn AAS_DumpAASData();
    // print AAS file information
    pub fn AAS_FileInfo();
}
