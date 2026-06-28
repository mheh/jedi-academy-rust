// Filename:-	tr_jpeg_interface.h
//
// pragma warning (disable: 4100)	//unreferenced formal parameter
// pragma warning (disable: 4127)	//conditional expression is constant
// pragma warning (disable: 4244)	//int to unsigned short

use core::ffi::{c_int, c_char, c_uchar};

pub type LPCSTR = *const c_char;

extern "C" {
    pub fn LoadJPG(filename: *const c_char, pic: *mut *mut c_uchar, width: *mut c_int, height: *mut c_int) -> c_int;
    pub fn SaveJPG(filename: *const c_char, quality: c_int, image_width: c_int, image_height: c_int, image_buffer: *mut c_uchar);
    pub fn JPG_ErrorThrow(message: LPCSTR);
    pub fn JPG_MessageOut(message: LPCSTR);
}

#[inline]
pub fn ERROR_STRING_NO_RETURN(message: LPCSTR) {
    unsafe { JPG_ErrorThrow(message) }
}

#[inline]
pub fn MESSAGE_STRING(message: LPCSTR) {
    unsafe { JPG_MessageOut(message) }
}

////////////////// eof //////////////////
