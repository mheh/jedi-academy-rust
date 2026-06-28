//! Mechanical port of `code/ff/ff_system.cpp`.
//!
//! Porting note: The original file was wrapped in `#ifdef _IMMERSION`, but this
//! code is translated unconditionally since _IMMERSION is not yet a Cargo feature.

// //#include "common_headers.h"
//
// #ifdef _IMMERSION
//
// //#include "ff.h"
// //#include "ff_ffset.h"
// //#include "ff_compound.h"
// //#include "ff_system.h"

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int};
use std::collections::BTreeMap;

// LOCAL STUB: qboolean type from engine
pub type qboolean = c_int;

// LOCAL STUB: ffHandle_t type from ff_public.h
pub type ffHandle_t = c_int;

// LOCAL STUB: TNameTable type from ff_utils.h
pub type TNameTable = Vec<String>;

// LOCAL STUB: Constants from ff_public.h
pub const FF_HANDLE_NULL: c_int = 0;
pub const FF_CHANNEL_BODY: c_int = 4;
pub const FF_CHANNEL_MAX: c_int = 7;
pub const FF_MAX_PATH: c_int = 256;

// LOCAL STUB: qboolean constants
pub const QTRUE: c_int = 1;
pub const QFALSE: c_int = 0;

// LOCAL STUB: Forward declaration stubs for unported dependencies
// These will be completed when the respective headers are ported

// FFConfigParser from ff_ConfigParser.h
pub struct FFConfigParser {
    _private: (),
}

impl FFConfigParser {
    pub fn Init(&mut self, _config_file: *const c_char) -> qboolean {
        // Stub: implementation will be in .cpp file
        QFALSE
    }

    pub fn RightOfSet(&self, _name: *const c_char) -> *const c_char {
        // Stub: implementation will be in .cpp file
        core::ptr::null()
    }

    pub fn Clear(&mut self) {
        // Stub: implementation will be in .cpp file
    }
}

// FFSet type from ff_ffset.h - used in FFMultiSet
pub struct FFSet {
    _private: (),
}

impl FFSet {
    pub fn GetDevice(&mut self) -> *mut CImmDevices {
        // Stub: implementation will be in .cpp file
        core::ptr::null_mut()
    }

    pub fn GetRegisteredNames(&self, _names: &mut TNameTable) {
        // Stub: implementation will be in .cpp file
    }
}

// CImmDevices from IFC - used in FFMultiSet
pub struct CImmDevices {
    _private: (),
}

impl CImmDevices {
    pub fn GetProductName(&self, _name: *mut c_char, _len: c_int) {
        // Stub: implementation will be in .cpp file
    }
}

// FFMultiSet from ff_MultiSet.h - base class for FFChannelSet
#[repr(C)]
pub struct FFMultiSet {
    mConfig: *mut FFConfigParser,
    // typedef vector<FFSet*> Set;
    // Set mSet;
    mSet: Vec<*mut FFSet>,
    mDevices: *mut CImmDevices,
}

// ChannelCompound from ff_ChannelCompound.h
#[repr(C)]
pub struct ChannelCompound {
    _private: (),
}

pub type Channel = BTreeMap<c_int, c_int>;

// FFChannelSet from ff_ChannelSet.h
pub struct FFChannelSet {
    mConfig: *mut FFConfigParser,
    mSet: Vec<*mut FFSet>,
    mDevices: *mut CImmDevices,
    mChannel: Channel,
}

impl FFChannelSet {
    pub fn Init(&mut self, _config: &mut FFConfigParser, _channels: *const c_char) -> qboolean {
        // Stub: implementation will be in .cpp file
        QFALSE
    }

    pub fn GetRegisteredNames(&self, _names: &mut TNameTable) {
        // Stub: implementation will be in .cpp file
    }

    pub fn GetSets(&mut self) -> &mut Vec<*mut FFSet> {
        // Stub: implementation will be in .cpp file
        &mut self.mSet
    }

    pub fn Display(&mut self, _unprocessed: &mut TNameTable, _processed: &mut TNameTable) {
        // Stub: implementation will be in .cpp file
    }

    pub fn StopAll(&mut self) -> qboolean {
        // Stub: implementation will be in .cpp file
        QFALSE
    }

    pub fn Register(&mut self, _compound: &mut ChannelCompound, _name: *const c_char, _create: qboolean) -> qboolean {
        // Stub: implementation will be in .cpp file
        QFALSE
    }
}

// FFChannelSet static method
#[cfg(feature = "FF_CONSOLECOMMAND")]
impl FFChannelSet {
    pub fn GetDisplayTokens(tokens: &mut TNameTable) {
        // Stub: implementation will be in .cpp file
    }
}

// FFHandleTable from ff_HandleTable.h
pub struct FFHandleTable {
    _private: (),
}

impl FFHandleTable {
    pub fn Init(&mut self) {
        // Stub: implementation will be in .cpp file
    }

    pub fn size(&self) -> usize {
        // Stub: implementation will be in .cpp file
        0
    }

    pub fn GetFailedNames(&self, _names: &mut TNameTable) -> qboolean {
        // Stub: implementation will be in .cpp file
        QFALSE
    }

    pub fn GetChannels(&self, _channels: &mut Vec<c_int>) {
        // Stub: implementation will be in .cpp file
    }

    pub fn clear(&mut self) {
        // Stub: implementation will be in .cpp file
    }

    pub fn index(&self, _i: usize) -> &ChannelCompound {
        // Stub: implementation will be in .cpp file
        unsafe { &*(core::ptr::null::<ChannelCompound>()) }
    }

    pub fn Convert(&mut self, _compound: &ChannelCompound, _name: *const c_char, _notfound: qboolean) -> ffHandle_t {
        // Stub: implementation will be in .cpp file
        FF_HANDLE_NULL
    }
}

impl ChannelCompound {
    pub fn GetChannel(&self) -> c_int {
        // Stub: implementation will be in .cpp file
        0
    }
}

// cvar_t stub for ff_developer
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

//===[FFSystem]===========================================================/////////////
//
//	The main system for a single user with multiple channels for
//	multiple simultaneous devices. All this is factored and 'classy'
//	with the intent to make it more readable and easy to track bugs.
//
//	That's the intent, at least.
//
//====================================================================/////////////

pub struct FFSystem {
    pub mConfig: FFConfigParser,
    pub mChannel: FFChannelSet,
    pub mHandle: FFHandleTable,
    pub mInitialized: qboolean,
    pub ffShake: ffHandle_t,
}

// External declarations
extern "C" {
    pub static mut ff_developer: *mut cvar_t;

    // LOCAL STUB: Com_Printf from engine
    pub fn Com_Printf(fmt: *const c_char, ...) -> c_int;

    // LOCAL STUB: stricmp from libc
    pub fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

////--------------
/// FFSystem::Init
//------------------
//
//
impl FFSystem {
    pub fn Init(&mut self, channels: *const c_char) -> qboolean {
        // kludgy restart mechanism
        #[repr(C)]
        struct TRestartInfo {
            name: TNameTable,
            channel: Vec<c_int>,
        }

        let mut restart = TRestartInfo {
            name: TNameTable::new(),
            channel: Vec::new(),
        };

        if self.mInitialized != QFALSE {
            restart.name.resize(self.mHandle.size(), String::new());
            restart.channel.resize(self.mHandle.size(), FF_CHANNEL_MAX);

            self.mChannel.GetRegisteredNames(&mut restart.name);
            self.mHandle.GetFailedNames(&mut restart.name);
            self.mHandle.GetChannels(&mut restart.channel);

            self.Shutdown();
        }

        self.mHandle.Init();

        if self.mConfig.Init(b"fffx/fffx.cfg\0".as_ptr() as *const c_char) != QFALSE	// Process config file
            && self.mChannel.Init(&mut self.mConfig, channels) != QFALSE		// Init devices
        {
            if restart.name.len() > 1 {
                for i in 1..restart.name.len() {
                    // ignore leading device-specific set name -- (may be switching devices)
                    unsafe {
                        let name_ptr = if restart.name[i].is_empty() {
                            core::ptr::null()
                        } else {
                            restart.name[i].as_ptr() as *const c_char
                        };
                        let name_from_config = self.mConfig.RightOfSet(name_ptr);
                        self.Register(name_from_config, restart.channel[i]);
                    }
                }
            } else {
                self.ffShake = self.Register(b"fffx/player/shake\0".as_ptr() as *const c_char, FF_CHANNEL_BODY);
            }

            self.mInitialized = QTRUE;
        }

        self.mInitialized
    }

    pub fn Shutdown(&mut self) {
        self.mInitialized = QFALSE;
        self.mHandle.clear();
        self.mChannel.Display(&mut TNameTable::new(), &mut TNameTable::new());
        self.mConfig.Clear();
    }

    pub fn Register(&mut self, name: *const c_char, channel: c_int) -> ffHandle_t {
        let mut result = FF_HANDLE_NULL;
        if !name.is_null() {
            unsafe {
                if *name != 0 {
                    let mut compound = ChannelCompound {
                        _private: (),
                    };
                    self.mChannel.Register(&mut compound, name, QTRUE);
                    result = self.mHandle.Convert(&compound, name, QTRUE);
                }
            }
        }
        result
    }
}

#[cfg(feature = "FF_CONSOLECOMMAND")]
impl FFSystem {
    pub fn GetDisplayTokens(&mut self, tokens: &mut TNameTable) {
        FFChannelSet::GetDisplayTokens(tokens);
        unsafe {
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                tokens.push("effects".to_string());
            }
        }
    }

    pub fn Display(&mut self, unprocessed: &mut TNameTable, processed: &mut TNameTable) {
        let mut itName = 0;
        while itName < unprocessed.len() {
            unsafe {
                if stricmp(
                    b"effects\0".as_ptr() as *const c_char,
                    unprocessed[itName].as_ptr() as *const c_char,
                ) == 0
                {
                    if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                        Com_Printf(b"[registered effects]\n\0".as_ptr() as *const c_char);

                        let mut EffectNames = TNameTable::new();
                        let mut total = 0;
                        let ffSet = self.mChannel.GetSets();

                        for i in 0..ffSet.len() {
                            let mut ProductName: [c_char; 256] = [0; 256];
                            ProductName[0] = 0;
                            if !ffSet[i].is_null() {
                                (*(*ffSet[i])).GetDevice();
                                // NOTE: GetProductName call omitted for stub
                                // ProductName is left as zero-terminated empty string
                                Com_Printf(b"%s...\n\0".as_ptr() as *const c_char, ProductName.as_ptr());

                                EffectNames.clear();
                                EffectNames.resize(self.mHandle.size(), String::new());
                                (*ffSet[i]).GetRegisteredNames(&mut EffectNames);

                                for j in 1..EffectNames.len() {
                                    if !EffectNames[j].is_empty() {
                                        Com_Printf(
                                            b"%3d) \"%s\" channel=%d\n\0".as_ptr() as *const c_char,
                                            total,
                                            EffectNames[j].as_ptr() as *const c_char,
                                            self.mHandle.index(j).GetChannel(),
                                        );
                                        total += 1;
                                    }
                                }
                            }
                        }

                        EffectNames.clear();
                        EffectNames.resize(self.mHandle.size(), String::new());

                        if self.mHandle.GetFailedNames(&mut EffectNames) != QFALSE {
                            Com_Printf(b"Failed Registrants...\n\0".as_ptr() as *const c_char);
                            for j in 1..EffectNames.len() {
                                if !EffectNames[j].is_empty() {
                                    Com_Printf(
                                        b"%3d) \"%s\" channel=%d\n\0".as_ptr() as *const c_char,
                                        total,
                                        EffectNames[j].as_ptr() as *const c_char,
                                        self.mHandle.index(j).GetChannel(),
                                    );
                                    total += 1;
                                }
                            }
                        }
                    }
                    //else
                    //{
                    //	Com_Printf( "\"effects\" only available when ff_developer is set\n" );
                    //}

                    processed.push(unprocessed[itName].clone());
                    unprocessed.remove(itName);
                } else {
                    itName += 1;
                }
            }
        }

        self.mChannel.Display(unprocessed, processed);
    }
}

// #endif // FF_CONSOLECOMMAND

// #endif // _IMMERSION
