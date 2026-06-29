#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, CStr};

// #include "common_headers.h"
// #include "ff.h"
// #include "ff_ffset.h"
// #include "ff_compound.h"
// #include "ff_system.h"

// ============================================================================
// Type declarations (from common headers)
// ============================================================================

pub type qboolean = c_int;
pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

pub type ffHandle_t = c_int;
pub const FF_HANDLE_NULL: ffHandle_t = -1;

#[repr(C)]
pub struct cvar_s {
    pub name: *const c_char,
    pub string: *const c_char,
    pub integer: c_int,
    // Opaque beyond these common fields
}

pub type cvar_t = cvar_s;

pub type TNameTable = Vec<String>;

// Forward declare FFSystem (C++ class)
#[repr(C)]
pub struct FFSystem;

// ============================================================================
// External functions
// ============================================================================

extern "C" {
    pub fn Cmd_AddCommand(cmd_name: *const c_char, function: extern "C" fn());
    pub fn Cmd_RemoveCommand(cmd_name: *const c_char);
    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_Argv(argv: c_int) -> *const c_char;
    pub fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Com_Printf(fmt: *const c_char, ...);

    // FFSystem C++ methods (extern wrapper functions or mangled names)
    fn FFSystem_IsInitialized(this: *mut FFSystem) -> qboolean;
    fn FFSystem_Init(this: *mut FFSystem, channels: *const c_char) -> qboolean;
    fn FFSystem_Shutdown(this: *mut FFSystem);
    fn FFSystem_Register(this: *mut FFSystem, name: *const c_char, channel: c_int, notfound: qboolean) -> ffHandle_t;
    fn FFSystem_EnsurePlaying(this: *mut FFSystem, ff: ffHandle_t) -> qboolean;
    fn FFSystem_Play(this: *mut FFSystem, ff: ffHandle_t) -> qboolean;
    fn FFSystem_Stop(this: *mut FFSystem, ff: ffHandle_t) -> qboolean;
    fn FFSystem_StopAll(this: *mut FFSystem) -> qboolean;
    fn FFSystem_Shake(this: *mut FFSystem, intensity: c_int, duration: c_int, ensure_shake: qboolean) -> qboolean;
    fn FFSystem_GetName(this: *const FFSystem, ff: ffHandle_t) -> *const c_char;
    fn FFSystem_Display(this: *mut FFSystem, unprocessed: *mut TNameTable, processed: *mut TNameTable);
}

// ============================================================================
// FFSystem impl wrapper methods
// ============================================================================

impl FFSystem {
    pub unsafe fn IsInitialized(&self) -> qboolean {
        FFSystem_IsInitialized(self as *const _ as *mut _)
    }

    pub unsafe fn Init(&mut self, channels: *const c_char) -> qboolean {
        FFSystem_Init(self as *mut _, channels)
    }

    pub unsafe fn Shutdown(&mut self) {
        FFSystem_Shutdown(self as *mut _)
    }

    pub unsafe fn Register(&mut self, name: *const c_char, channel: c_int, notfound: qboolean) -> ffHandle_t {
        FFSystem_Register(self as *mut _, name, channel, notfound)
    }

    pub unsafe fn EnsurePlaying(&mut self, ff: ffHandle_t) -> qboolean {
        FFSystem_EnsurePlaying(self as *mut _, ff)
    }

    pub unsafe fn Play(&mut self, ff: ffHandle_t) -> qboolean {
        FFSystem_Play(self as *mut _, ff)
    }

    pub unsafe fn Stop(&mut self, ff: ffHandle_t) -> qboolean {
        FFSystem_Stop(self as *mut _, ff)
    }

    pub unsafe fn StopAll(&mut self) -> qboolean {
        FFSystem_StopAll(self as *mut _)
    }

    pub unsafe fn Shake(&mut self, intensity: c_int, duration: c_int, ensure_shake: qboolean) -> qboolean {
        FFSystem_Shake(self as *mut _, intensity, duration, ensure_shake)
    }

    pub unsafe fn GetName(&self, ff: ffHandle_t) -> *const c_char {
        FFSystem_GetName(self as *const _, ff)
    }

    pub unsafe fn Display(&mut self, unprocessed: *mut TNameTable, processed: *mut TNameTable) {
        FFSystem_Display(self as *mut _, unprocessed, processed)
    }
}

// ============================================================================
// Global variables
// ============================================================================

pub static mut gFFSystem: FFSystem = FFSystem;

pub static mut use_ff: *mut cvar_t = core::ptr::null_mut();
pub static mut ensureShake: *mut cvar_t = core::ptr::null_mut();
pub static mut ff_developer: *mut cvar_t = core::ptr::null_mut();

#[cfg(feature = "FF_DELAY")]
pub static mut ff_delay: *mut cvar_t = core::ptr::null_mut();

pub static mut ff_channels: *mut cvar_t = core::ptr::null_mut();

// ============================================================================
// Static string constants
// ============================================================================

static _pass: &[u8] = b"SUCCEEDED";
static _fail: &[u8] = b"FAILED";

// ============================================================================
// Channel name table
// ============================================================================

pub static gChannelName: [&[u8]; 7] = [
    b"FF_CHANNEL_WEAPON",
    b"FF_CHANNEL_MENU",
    b"FF_CHANNEL_TOUCH",
    b"FF_CHANNEL_DAMAGE",
    b"FF_CHANNEL_BODY",
    b"FF_CHANNEL_FORCE",
    b"FF_CHANNEL_FOOT",
];

// ============================================================================
// Macros
// ============================================================================

// Enable/Disable Com_Printf in FF_* functions
#[cfg(feature = "FF_PRINT")]
macro_rules! FF_PROLOGUE {
    ($name:ident, $string:expr) => {
        let mut result: qboolean = qfalse;
        if FF_IsAvailable() != 0 {
            unsafe {
                if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                    Com_Printf(b"%s: \"%s\" \0".as_ptr() as *const c_char, stringify!($name).as_ptr(), $string.as_ptr());
                }
            }
        }
    };
}

#[cfg(feature = "FF_PRINT")]
macro_rules! FF_PROLOGUE_NOQUOTE {
    ($name:ident, $string:expr) => {
        let mut result: qboolean = qfalse;
        if FF_IsAvailable() != 0 {
            unsafe {
                if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                    Com_Printf(b"%s: %s \0".as_ptr() as *const c_char, stringify!($name).as_ptr(), $string.as_ptr());
                }
            }
        }
    };
}

#[cfg(feature = "FF_PRINT")]
macro_rules! FF_EPILOGUE {
    () => {
        FF_EPILOGUE_NORETURN!();
        return result;
    };
}

#[cfg(feature = "FF_PRINT")]
macro_rules! FF_EPILOGUE_NORETURN {
    () => {
        unsafe {
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                let status_str = if result != 0 { _pass.as_ptr() } else { _fail.as_ptr() };
                Com_Printf(b"[%s]\n\0".as_ptr() as *const c_char, status_str);
            }
        }
    };
}

#[cfg(feature = "FF_PRINT")]
macro_rules! FF_RESULT {
    ($function:expr) => {
        result = if ($function) != 0 { qtrue } else { qfalse };
    };
}

#[cfg(not(feature = "FF_PRINT"))]
macro_rules! FF_PROLOGUE {
    ($name:ident, $string:expr) => {
        let mut result: qboolean = qfalse;
        if FF_IsAvailable() != 0 {
            // Prologue without printing
        }
    };
}

#[cfg(not(feature = "FF_PRINT"))]
macro_rules! FF_PROLOGUE_NOQUOTE {
    ($name:ident, $string:expr) => {
        let mut result: qboolean = qfalse;
        if FF_IsAvailable() != 0 {
            // Prologue without printing
        }
    };
}

#[cfg(not(feature = "FF_PRINT"))]
macro_rules! FF_EPILOGUE {
    () => {
        FF_EPILOGUE_NORETURN!();
        return result;
    };
}

#[cfg(not(feature = "FF_PRINT"))]
macro_rules! FF_EPILOGUE_NORETURN {
    () => {
        // No output
    };
}

#[cfg(not(feature = "FF_PRINT"))]
macro_rules! FF_RESULT {
    ($function:expr) => {
        result = if ($function) != 0 { qtrue } else { qfalse };
    };
}

// ============================================================================
// Functions
// ============================================================================

/// FF_IsAvailable
///
/// Test to see if force feedback is currently operating. This is almost useless.
/// The only good it does currently is temporarily toggle effects on/off for users
/// amusement... feedback on, feedback off, feedback on, feedback off. Results are
/// instantaneous. FF_* calls basically skip themselves harmlessly.
///
/// Assumptions:
/// *	External system unloads this module if no device is present.
/// *	External system unloads this module if feedback is disabled when system (re)starts
///
/// Parameters:
/// 	None
///
/// Returns:
/// -	true: feedback currently enabled
/// -	false: feedback currently disabled
///
pub fn FF_IsAvailable() -> qboolean {
    unsafe {
        let gff_sys = core::ptr::addr_of_mut!(gFFSystem);
        if !use_ff.is_null() && (*use_ff).integer != 0 && (*gff_sys).IsInitialized() != 0 {
            qtrue
        } else {
            qfalse
        }
    }
}

pub fn FF_IsInitialized() -> qboolean {
    unsafe {
        (*core::ptr::addr_of_mut!(gFFSystem)).IsInitialized()
    }
}

/// FF_Init
///
/// Initializes the force feedback system.
///
/// This function may be called multiple times to apply changes in cvars.
///
/// Assumptions:
/// *	If FF_Init returns qfalse, caller calls FF_Shutdown
///
/// Parameters:
/// 	None
///
/// Returns:
/// -	qtrue: module initialized properly.
/// -	qfalse: module experienced an error. Caller MUST call FF_Shutdown.
///
pub fn FF_Init() -> qboolean {
    unsafe {
        let gff_sys = core::ptr::addr_of_mut!(gFFSystem);
        if (*gff_sys).IsInitialized() == 0 {
            // Console variable setup

            #[cfg(feature = "FF_CONSOLECOMMAND")]
            {
                Cmd_AddCommand(b"ff_stopall\0".as_ptr() as *const c_char, CMD_FF_StopAll);
                Cmd_AddCommand(b"ff_info\0".as_ptr() as *const c_char, CMD_FF_Info);
            }

            use_ff = Cvar_Get(b"use_ff\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0); // CVAR_ARCHIVE
            ensureShake = Cvar_Get(b"ff_ensureShake\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0); // CVAR_TEMP
            ff_developer = Cvar_Get(b"ff_developer\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0); // CVAR_TEMP
            ff_channels = Cvar_Get(b"ff_channels\0".as_ptr() as *const c_char, b"FF_CHANNEL\0".as_ptr() as *const c_char, 0); // CVAR_ARCHIVE

            #[cfg(feature = "FF_DELAY")]
            {
                ff_delay = Cvar_Get(b"ff_delay\0".as_ptr() as *const c_char, b"FF_DELAY\0".as_ptr() as *const c_char, 0); // CVAR_ARCHIVE
            }
        }

        // assumes external system will call FF_Shutdown in case of failure
        if !ff_channels.is_null() && (*gff_sys).Init((*ff_channels).string) != 0 {
            qtrue
        } else {
            qfalse
        }
    }
}

/// FF_Shutdown
///
/// Shut force feedback system down and free resources.
///
/// Assumptions:
/// *	Always called if FF_Init returns qfalse. ALWAYS. (Or memory leaks occur)
/// *	Never called twice in succession. (always in response to previous assumption)
///
/// Parameters:
/// 	None
///
/// Returns:
/// 	None
///
pub fn FF_Shutdown() {
    #[cfg(feature = "FF_CONSOLECOMMAND")]
    {
        unsafe {
            Cmd_RemoveCommand(b"ff_stopall\0".as_ptr() as *const c_char);
            Cmd_RemoveCommand(b"ff_info\0".as_ptr() as *const c_char);
        }
    }

    unsafe {
        (*core::ptr::addr_of_mut!(gFFSystem)).Shutdown();
    }
}

/// FF_Register
///
/// Loads a named effect from an .ifr file through the game's file system. The handle
/// returned is not tied to any particular device. The feedback system may even change
/// which device receives the effect without the need to restart the external system.
/// The is ONE EXCEPTION. If this module is not loaded when the registration phase
/// passes, the external system will need to be restarted to register effects properly.
///
/// Parameters:
/// *	name: effect name from .ifr (case-sensitive)
/// *	channel: channel to output effect. A channel may play on 0+ devices.
/// *	notfound: return a valid handle even if effect is not found
/// 	- Allows temporary disabling of a channel in-game without losing effects
/// 	- Only use with trusted effect names, not user input. See CMD_FF_Play.
///
/// Returns:
/// 	Handle to loaded effect
///
pub fn FF_Register(name: *const c_char, channel: c_int, notfound: qboolean) -> ffHandle_t {
    let mut ff: ffHandle_t = FF_HANDLE_NULL;

    // Removed console print... too much spam with AddLoopingForce.
    // FF_PROLOGUE( FF_Register, ( name ? name : "" ) );
    // ff = gFFSystem.Register( name, channel, notfound );
    // FF_RESULT( ff != FF_HANDLE_NULL );
    // FF_EPILOGUE_NORETURN;

    if FF_IsAvailable() != 0 {
        unsafe {
            ff = (*core::ptr::addr_of_mut!(gFFSystem)).Register(name, channel, notfound);
        }
    }

    ff
}

/// FF_EnsurePlaying
///
/// Starts an effect if the effect is not currently playing.
/// Does not restart currently playing effects.
///
/// Parameters:
/// *	ff: handle of an effect
///
/// Returns:
/// -	qtrue: effect started
/// -	qfalse: effect was not started
///
pub fn FF_EnsurePlaying(ff: ffHandle_t) -> qboolean {
    unsafe {
        let mut result: qboolean = qfalse;
        let gff_sys = core::ptr::addr_of_mut!(gFFSystem);
        if FF_IsAvailable() != 0 {
            #[cfg(feature = "FF_PRINT")]
            {
                if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                    Com_Printf(b"%s: \"%s\" \0".as_ptr() as *const c_char, b"FF_EnsurePlaying\0".as_ptr() as *const c_char, (*gff_sys).GetName(ff));
                }
            }
            result = if (*gff_sys).EnsurePlaying(ff) != 0 { qtrue } else { qfalse };
        }
        #[cfg(feature = "FF_PRINT")]
        {
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                let status_str = if result != 0 { _pass.as_ptr() } else { _fail.as_ptr() };
                Com_Printf(b"[%s]\n\0".as_ptr() as *const c_char, status_str);
            }
        }
        result
    }
}

/// FF_Play
///
/// Start an effect on its registered channel.
///
/// Parameters
/// *	ff: handle to an effect
///
/// Returns:
/// -	qtrue: effect started
/// -	qfalse: effect was not started
///
pub fn FF_Play(ff: ffHandle_t) -> qboolean {
    unsafe {
        let mut result: qboolean = qfalse;
        let gff_sys = core::ptr::addr_of_mut!(gFFSystem);
        if FF_IsAvailable() != 0 {
            #[cfg(feature = "FF_PRINT")]
            {
                if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                    Com_Printf(b"%s: \"%s\" \0".as_ptr() as *const c_char, b"FF_Play\0".as_ptr() as *const c_char, (*gff_sys).GetName(ff));
                }
            }
            result = if (*gff_sys).Play(ff) != 0 { qtrue } else { qfalse };
        }
        #[cfg(feature = "FF_PRINT")]
        {
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                let status_str = if result != 0 { _pass.as_ptr() } else { _fail.as_ptr() };
                Com_Printf(b"[%s]\n\0".as_ptr() as *const c_char, status_str);
            }
        }
        result
    }
}

/// FF_StopAll
///
/// Stop all currently playing effects.
///
/// Parameters:
/// 	None
///
/// Returns:
/// -	qtrue: no errors occurred
/// -	qfalse: an error occurred
///
pub fn FF_StopAll() -> qboolean {
    unsafe {
        let mut result: qboolean = qfalse;
        let gff_sys = core::ptr::addr_of_mut!(gFFSystem);
        if FF_IsAvailable() != 0 {
            #[cfg(feature = "FF_PRINT")]
            {
                if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                    Com_Printf(b"%s: \"%s\" \0".as_ptr() as *const c_char, b"FF_StopAll\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
                }
            }
            result = if (*gff_sys).StopAll() != 0 { qtrue } else { qfalse };
        }
        #[cfg(feature = "FF_PRINT")]
        {
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                let status_str = if result != 0 { _pass.as_ptr() } else { _fail.as_ptr() };
                Com_Printf(b"[%s]\n\0".as_ptr() as *const c_char, status_str);
            }
        }
        result
    }
}

/// FF_Stop
///
/// Stop an effect. Only returns qfalse if there's an error
///
/// Parameters:
/// *	ff: handle to a playing effect
///
/// Returns:
/// -	qtrue: no errors occurred
/// -	qfalse: an error occurred
///
pub fn FF_Stop(ff: ffHandle_t) -> qboolean {
    unsafe {
        let mut result: qboolean = qfalse;
        let gff_sys = core::ptr::addr_of_mut!(gFFSystem);
        if FF_IsAvailable() != 0 {
            #[cfg(feature = "FF_PRINT")]
            {
                if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                    Com_Printf(b"%s: \"%s\" \0".as_ptr() as *const c_char, b"FF_Stop\0".as_ptr() as *const c_char, (*gff_sys).GetName(ff));
                }
            }
            result = if (*gff_sys).Stop(ff) != 0 { qtrue } else { qfalse };
        }
        #[cfg(feature = "FF_PRINT")]
        {
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                let status_str = if result != 0 { _pass.as_ptr() } else { _fail.as_ptr() };
                Com_Printf(b"[%s]\n\0".as_ptr() as *const c_char, status_str);
            }
        }
        result
    }
}

/// FF_Shake
///
/// Shake the mouse (play the special "shake" effect) at a given strength
/// for a given duration. The shake effect can be a compound containing
/// multiple component effects, but each component effect's magnitude and
/// duration will be set to the parameters passed in this function.
///
/// Parameters:
/// *	intensity [0..10000] - Magnitude of effect
/// *	duration [0..MAXINT] - Length of shake in milliseconds
///
/// Returns:
/// -	qtrue: shake started
/// -	qfalse: shake did not start
///
pub fn FF_Shake(intensity: c_int, duration: c_int) -> qboolean {
    unsafe {
        let mut result: qboolean = qfalse;
        let gff_sys = core::ptr::addr_of_mut!(gFFSystem);
        if FF_IsAvailable() != 0 {
            #[cfg(feature = "FF_PRINT")]
            {
                if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                    Com_Printf(b"%s: %s \0".as_ptr() as *const c_char, b"FF_Shake\0".as_ptr() as *const c_char, b"intensity/duration message\0".as_ptr() as *const c_char);
                }
            }
            let ensure_shake_val = if !ensureShake.is_null() && (*ensureShake).integer != 0 {
                qtrue
            } else {
                qfalse
            };
            result = if (*gff_sys).Shake(intensity, duration, ensure_shake_val) != 0 { qtrue } else { qfalse };
        }
        #[cfg(feature = "FF_PRINT")]
        {
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                let status_str = if result != 0 { _pass.as_ptr() } else { _fail.as_ptr() };
                Com_Printf(b"[%s]\n\0".as_ptr() as *const c_char, status_str);
            }
        }
        result
    }
}

#[cfg(feature = "FF_CONSOLECOMMAND")]
/// CMD_FF_StopAll
///
/// Console function which stops all currently playing effects
///
/// Parameters:
/// 	None
///
/// Returns:
/// 	None
///
extern "C" fn CMD_FF_StopAll() {
    // Display messages
    if FF_StopAll() != 0 {
        unsafe {
            Com_Printf(b"stopping all effects\n\0".as_ptr() as *const c_char);
        }
    } else {
        unsafe {
            Com_Printf(b"failed to stop all effects\n\0".as_ptr() as *const c_char);
        }
    }
}

#[cfg(feature = "FF_CONSOLECOMMAND")]
/// CMD_FF_Info
///
/// Console function which displays various ff-system information.
///
/// Parameters:
/// *	'devices'	display list of ff devices currently connected
/// *	'channels'	display list of support ff channels
/// *	'order'		display search order used by ff name-resolution system (ff_ffset.cpp)
/// *	'files'		display currently loaded .ifr files sorted by device
/// *	'effects'	display currently registered effects sorted by device
///
/// Returns:
/// 	None
///
extern "C" fn CMD_FF_Info() {
    unsafe {
        let mut unprocessed: TNameTable = Vec::new();
        let mut processed: TNameTable = Vec::new();

        let argc = Cmd_Argc();
        for i in 1..argc {
            let arg = Cmd_Argv(i);
            if !arg.is_null() {
                // Convert c_char pointer to String
                let c_str = CStr::from_ptr(arg);
                if let Ok(s) = c_str.to_str() {
                    unprocessed.push(s.to_string());
                }
            }
        }

        if unprocessed.is_empty() {
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                Com_Printf(b"Usage: ff_info [devices] [channels] [order] [files] [effects]\n\0".as_ptr() as *const c_char);
            } else {
                Com_Printf(b"Usage: ff_info [devices] [channels]\n\0".as_ptr() as *const c_char);
            }

            return;
        }

        (*core::ptr::addr_of_mut!(gFFSystem)).Display(&mut unprocessed, &mut processed);

        if !unprocessed.is_empty() {
            Com_Printf(b"invalid parameters:\0".as_ptr() as *const c_char);
            for param in &unprocessed {
                Com_Printf(b" %s\0".as_ptr() as *const c_char, param.as_ptr() as *const c_char);
            }

            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                Com_Printf(b"Usage: ff_info [devices] [channels] [order] [files] [effects]\n\0".as_ptr() as *const c_char);
            } else {
                Com_Printf(b"Usage: ff_info [devices] [channels]\n\0".as_ptr() as *const c_char);
            }
        }
    }
}

// #endif // FF_CONSOLECOMMAND
// #endif // _IMMERSION
