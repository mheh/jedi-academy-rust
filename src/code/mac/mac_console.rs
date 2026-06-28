#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// Mac OS SDK types - stubs for compatibility with <DriverServices.h> and <console.h>
#[repr(C)]
pub struct EventRecord {
    pub what: c_int,
    pub message: c_int,
    // Rest of EventRecord fields are opaque to this module
}

#[repr(C)]
pub struct SIOUX_Settings {
    pub initializeTB: c_int,
    pub standalone: c_int,
    pub setupmenus: c_int,
    pub autocloseonquit: c_int,
    pub asktosaveonclose: c_int,
    pub toppixel: c_int,
    pub leftpixel: c_int,
}

// Extern C interface to Mac OS SDK functions
extern "C" {
    // From <console.h>
    pub static mut SIOUXSettings: SIOUX_Settings;
    fn SIOUXHandleOneEvent(event: *mut EventRecord) -> c_int;
    fn printf(fmt: *const c_char, ...) -> c_int;
}

// Mac OS event type constants from <DriverServices.h>
const keyDown: c_int = 3;
const charCodeMask: c_int = 0xFF;

#[allow(non_snake_case)]
const CONSOLE_MASK: c_int = 1023;

static mut consoleChars: [c_char; 1024] = [0; 1024];
static mut consoleHead: c_int = 0;
static mut consoleTail: c_int = 0;
static mut consoleDisplayed: c_int = 0;

// Static buffer for return value of Sys_ConsoleInput
static mut console_input_string: [c_char; 1024] = [0; 1024];

/*
==================
Sys_InitConsole
==================
*/
pub unsafe fn Sys_InitConsole() {
    (*core::ptr::addr_of_mut!(SIOUXSettings)).initializeTB = 0;
    (*core::ptr::addr_of_mut!(SIOUXSettings)).standalone = 0;
    (*core::ptr::addr_of_mut!(SIOUXSettings)).setupmenus = 0;
    (*core::ptr::addr_of_mut!(SIOUXSettings)).autocloseonquit = 1;
    (*core::ptr::addr_of_mut!(SIOUXSettings)).asktosaveonclose = 0;
    (*core::ptr::addr_of_mut!(SIOUXSettings)).toppixel = 40;
    (*core::ptr::addr_of_mut!(SIOUXSettings)).leftpixel = 10;

    //	Sys_ShowConsole( 1, qfalse );
}

/*
==================
Sys_ShowConsole
==================
*/
pub unsafe fn Sys_ShowConsole(level: c_int, _quitOnClose: c_int) {

    if level != 0 {
        *core::ptr::addr_of_mut!(consoleDisplayed) = 1;
        printf(b"\n\0".as_ptr() as *const c_char);
    } else {
        // FIXME: I don't know how to hide this window...
        *core::ptr::addr_of_mut!(consoleDisplayed) = 0;
    }
}


/*
================
Sys_Print

This is called for all console output, even if the game is running
full screen and the dedicated console window is hidden.
================
*/
pub unsafe fn Sys_Print(text: *const c_char) {
    if *core::ptr::addr_of!(consoleDisplayed) == 0 {
        return;
    }
    printf(b"%s\0".as_ptr() as *const c_char, text);
}


/*
==================
Sys_ConsoleEvent
==================
*/
pub unsafe fn Sys_ConsoleEvent(event: *mut EventRecord) -> c_int {
    let flag: c_int = SIOUXHandleOneEvent(event);

    // track keyboard events so we can do console input,
    // because SIOUX doesn't offer a polled read as far
    // as I can tell...
    if flag != 0 && (*event).what == keyDown {
        let myCharCode: c_int;

        myCharCode = (*event).message & charCodeMask;
        if myCharCode == 8 || myCharCode == 28 {
            if *core::ptr::addr_of!(consoleHead) > *core::ptr::addr_of!(consoleTail) {
                *core::ptr::addr_of_mut!(consoleHead) -= 1;
            }
        } else if myCharCode >= 32 || myCharCode == 13 {
            let head = *core::ptr::addr_of!(consoleHead);
            consoleChars[(head & CONSOLE_MASK) as usize] = myCharCode as c_char;
            *core::ptr::addr_of_mut!(consoleHead) += 1;
        }
    }

    flag
}


/*
================
Sys_ConsoleInput

Checks for a complete line of text typed in at the console.
Return NULL if a complete line is not ready.
================
*/
pub unsafe fn Sys_ConsoleInput() -> *const c_char {
    if *core::ptr::addr_of!(consoleTail) == *core::ptr::addr_of!(consoleHead) {
        return core::ptr::null();
    }

    let mut i: c_int = 0;
    while i + *core::ptr::addr_of!(consoleTail) < *core::ptr::addr_of!(consoleHead) {
        let tail = *core::ptr::addr_of!(consoleTail);
        console_input_string[i as usize] = consoleChars[((tail + i) & CONSOLE_MASK) as usize];
        if console_input_string[i as usize] == 13 {
            *core::ptr::addr_of_mut!(consoleTail) += i + 1;
            console_input_string[i as usize] = 0;
            return console_input_string.as_ptr() as *const c_char;
        }
        i += 1;
    }

    core::ptr::null()
}
