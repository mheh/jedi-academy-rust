#![allow(non_snake_case, non_camel_case_types, dead_code, unused_variables)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr::null_mut;

pub type FILE = c_void;

static SYS_ERROR_PREFIX: [c_char; 12] = [
    b'S' as c_char,
    b'y' as c_char,
    b's' as c_char,
    b'_' as c_char,
    b'E' as c_char,
    b'r' as c_char,
    b'r' as c_char,
    b'o' as c_char,
    b'r' as c_char,
    b':' as c_char,
    b' ' as c_char,
    0,
];
static NEWLINE: [c_char; 2] = [b'\n' as c_char, 0];
static STRING_FORMAT: [c_char; 3] = [b'%' as c_char, b's' as c_char, 0];

unsafe extern "C" {
    fn fread(buffer: *mut c_void, size: usize, count: usize, stream: *mut FILE) -> usize;
    fn fseek(stream: *mut FILE, offset: c_int, origin: c_int) -> c_int;
    fn printf(format: *const c_char, ...) -> c_int;
    fn exit(status: c_int) -> !;

    fn Com_Init(argc: c_int, argv: *mut *mut c_char);
    fn Com_Frame();
}

#[no_mangle]
pub static mut sys_curtime: c_int = 0;

//===================================================================

#[no_mangle]
pub unsafe extern "C" fn Sys_BeginStreamedFile(f: *mut FILE, readAhead: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn Sys_EndStreamedFile(f: *mut FILE) {}

#[no_mangle]
pub unsafe extern "C" fn Sys_StreamedRead(
    buffer: *mut c_void,
    size: c_int,
    count: c_int,
    f: *mut FILE,
) -> c_int {
    fread(buffer, size as usize, count as usize, f) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn Sys_StreamSeek(f: *mut FILE, offset: c_int, origin: c_int) {
    fseek(f, offset, origin);
}

//===================================================================

#[no_mangle]
pub unsafe extern "C" fn Sys_mkdir(path: *const c_char) {}

#[no_mangle]
pub unsafe extern "C" fn Sys_Error(error: *mut c_char) {
    // DEVIATION: Rust cannot define C variadic functions on stable; this preserves
    // the null driver's visible prefix/newline/exit behavior but cannot consume
    // additional format arguments from the original `char *error, ...` signature.
    printf(SYS_ERROR_PREFIX.as_ptr());
    printf(STRING_FORMAT.as_ptr(), error);
    printf(NEWLINE.as_ptr());

    exit(1);
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Quit() {
    exit(0);
}

#[no_mangle]
pub unsafe extern "C" fn Sys_UnloadGame() {}

#[no_mangle]
pub unsafe extern "C" fn Sys_GetGameAPI(parms: *mut c_void) -> *mut c_void {
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn Sys_GetClipboardData() -> *mut c_char {
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Milliseconds() -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn Sys_Mkdir(path: *mut c_char) {}

#[no_mangle]
pub unsafe extern "C" fn Sys_FindFirst(
    path: *mut c_char,
    musthave: c_uint,
    canthave: c_uint,
) -> *mut c_char {
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn Sys_FindNext(musthave: c_uint, canthave: c_uint) -> *mut c_char {
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn Sys_FindClose() {}

#[no_mangle]
pub unsafe extern "C" fn Sys_Init() {}

#[no_mangle]
pub unsafe extern "C" fn Sys_EarlyOutput(string: *mut c_char) {
    printf(STRING_FORMAT.as_ptr(), string);
}

#[no_mangle]
pub unsafe extern "C" fn main(argc: c_int, argv: *mut *mut c_char) {
    Com_Init(argc, argv);

    loop {
        Com_Frame();
    }
}
