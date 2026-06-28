// client_ui.h -- header for client access to ui funcs

use core::ffi::{c_int, c_char};

// Type alias for qboolean (from ui_public.h)
pub type qboolean = c_int;

#[allow(non_snake_case)]
pub extern "C" {
    pub fn _UI_KeyEvent(key: c_int, down: qboolean);
    pub fn UI_SetActiveMenu(menuname: *const c_char, menuID: *const c_char);
    pub fn UI_UpdateConnectionMessageString(string: *mut c_char);
    pub fn UI_ConsoleCommand() -> qboolean;
    pub fn _UI_IsFullscreen() -> qboolean;
}
