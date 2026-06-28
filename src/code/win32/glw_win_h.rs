use core::ffi::{c_int, c_void};

#[repr(C)]
#[cfg(target_os = "windows")]
pub struct glwstate_t {
    pub wndproc: *mut c_void,

    pub hDC: *mut c_void,           // handle to device context
    pub hGLRC: *mut c_void,         // handle to GL rendering context

    pub hinstOpenGL: *mut c_void,   // HINSTANCE for the OpenGL library

    pub allowdisplaydepthchange: c_int,
    pub pixelFormatSet: c_int,

    pub desktopBitsPixel: c_int,
    pub desktopWidth: c_int,
    pub desktopHeight: c_int,

    pub cdsFullscreen: c_int,

    pub log_fp: *mut c_void,
}

#[cfg(target_os = "windows")]
pub static mut glw_state: glwstate_t = glwstate_t {
    wndproc: std::ptr::null_mut(),
    hDC: std::ptr::null_mut(),
    hGLRC: std::ptr::null_mut(),
    hinstOpenGL: std::ptr::null_mut(),
    allowdisplaydepthchange: 0,
    pixelFormatSet: 0,
    desktopBitsPixel: 0,
    desktopWidth: 0,
    desktopHeight: 0,
    cdsFullscreen: 0,
    log_fp: std::ptr::null_mut(),
};
