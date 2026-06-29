use core::ffi::{c_char, c_void};

extern "C" {
    pub fn SP_DisplayLogos();
    pub fn SP_DoLicense();
    pub fn SP_LoadFile(name: *const c_char) -> *mut c_void;
    pub fn SP_LoadFileWithLanguage(name: *const c_char) -> *mut c_void;
    pub fn SP_GetLanguageExt() -> *mut c_char;
    pub fn SP_DrawTexture(
        pixels: *mut c_void,
        width: f32,
        height: f32,
        vShift: f32,
    );
}
