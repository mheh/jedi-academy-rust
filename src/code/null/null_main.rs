// sys_null.h -- null system driver to aid porting efforts

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint, VaList};

// Declared from qcommon/qcommon.h and libc
extern "C" {
    // From qcommon/qcommon.h
    fn FS_Read(buffer: *mut c_void, size: c_int, count: c_int, f: fileHandle_t) -> c_int;
    fn FS_Seek(f: fileHandle_t, offset: c_int, origin: c_int);
    fn Com_Init(argc: c_int, argv: *const *const c_char);
    fn Com_Frame();

    // From libc
    fn printf(format: *const c_char, ...) -> c_int;
    fn vprintf(format: *const c_char, ap: VaList) -> c_int;
    fn exit(status: c_int) -> !;
}

pub type fileHandle_t = c_int;

pub static mut sys_curtime: c_int = 0;


//===================================================================

pub fn Sys_BeginStreamedFile(f: fileHandle_t, readAhead: c_int) {
}

pub fn Sys_EndStreamedFile(f: fileHandle_t) {
}

pub fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: fileHandle_t) -> c_int {
    unsafe { FS_Read(buffer, size, count, f) }
}

pub fn Sys_StreamSeek(f: fileHandle_t, offset: c_int, origin: c_int) {
    unsafe { FS_Seek(f, offset, origin) };
}


//===================================================================


pub fn Sys_mkdir(path: *const c_char) {
}

pub unsafe extern "C" fn Sys_Error(error: *const c_char, mut args: VaList) {
    printf(b"Sys_Error: \0".as_ptr() as *const c_char);
    vprintf(error, args.as_mut_ptr());
    printf(b"\n\0".as_ptr() as *const c_char);

    exit(1);
}

pub fn Sys_Quit() -> ! {
    unsafe { exit(0) }
}

pub fn Sys_UnloadGame() {
}

pub fn Sys_GetGameAPI(parms: *mut c_void) -> *mut c_void {
    core::ptr::null_mut()
}

pub fn Sys_GetClipboardData() -> *const c_char {
    core::ptr::null()
}

pub fn Sys_Milliseconds() -> c_int {
    0
}

pub fn Sys_Mkdir(path: *const c_char) {
}

pub fn Sys_FindFirst(path: *const c_char, musthave: c_uint, canthave: c_uint) -> *const c_char {
    core::ptr::null()
}

pub fn Sys_FindNext(musthave: c_uint, canthave: c_uint) -> *const c_char {
    core::ptr::null()
}

pub fn Sys_FindClose() {
}

pub fn Sys_Init() {
}


pub fn Sys_EarlyOutput(string: *const c_char) {
    unsafe { printf(b"%s\0".as_ptr() as *const c_char, string) };
}


#[no_mangle]
pub extern "C" fn main(argc: c_int, argv: *const *const c_char) -> ! {
    unsafe {
        Com_Init(argc, argv);

        loop {
            Com_Frame();
        }
    }
}

