//! Mechanical port of `code/ff/ff_ChannelSet.cpp`.
//!
//! Porting note: The original file was wrapped in `#ifdef _IMMERSION`, but this
//! code is translated unconditionally since _IMMERSION is not yet a Cargo feature.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};
use std::collections::BTreeMap;

// LOCAL STUB: qboolean type from engine
pub type qboolean = c_int;

// LOCAL STUB: TNameTable type from ff_utils.h (vector<string>)
pub type TNameTable = Vec<String>;

// LOCAL STUB: FFSet from ff_ffset.h
pub struct FFSet {
    _private: (),
}

impl FFSet {
    pub fn Register(&mut self, name: *const c_char, create: qboolean) -> *mut MultiEffect {
        // Stub: implementation will be in ff_ffset.rs
        core::ptr::null_mut()
    }
}

// LOCAL STUB: FFConfigParser from ff_ConfigParser.h
pub struct FFConfigParser {
    _private: (),
}

// LOCAL STUB: CImmDevices from IFC
pub struct CImmDevices {
    _private: (),
}

impl CImmDevices {
    pub fn GetNumDevices(&self) -> c_int {
        // Stub: implementation will be in IFC
        0
    }

    pub fn GetDevice(&mut self, i: c_int) -> *mut DeviceInfo {
        // Stub: implementation will be in IFC
        core::ptr::null_mut()
    }
}

pub struct DeviceInfo {
    _private: (),
}

impl DeviceInfo {
    pub fn GetProductName(&self, name: *mut c_char, maxlen: c_int) {
        // Stub: implementation will be in IFC
    }
}

// LOCAL STUB: MultiEffect from ff_MultiEffect.h
pub struct MultiEffect {
    _private: (),
}

impl MultiEffect {
    pub fn GetName(&self) -> *const c_char {
        // Stub: implementation will be in ff_MultiEffect.rs
        core::ptr::null()
    }
}

// LOCAL STUB: FFMultiSet from ff_MultiSet.h - base class for FFChannelSet
#[repr(C)]
pub struct FFMultiSet {
    mConfig: *mut FFConfigParser,
    mSet: Vec<*mut FFSet>,
    mDevices: *mut CImmDevices,
}

impl FFMultiSet {
    pub fn GetDisplayTokens(Tokens: &mut TNameTable) {
        // Stub: implementation will be in ff_MultiSet.rs
    }

    pub fn Display(&mut self, Unprocessed: &mut TNameTable, Processed: &mut TNameTable) {
        // Stub: implementation will be in ff_MultiSet.rs
    }
}

// LOCAL STUB: ChannelCompound from ff_ChannelCompound.h
pub struct ChannelCompound {
    _private: (),
}

impl ChannelCompound {
    pub fn GetChannel(&self) -> c_int {
        // Stub: implementation will be in ff_ChannelCompound_h.rs
        0
    }

    pub fn GetSet(&mut self) -> &mut std::collections::BTreeSet<*mut MultiEffect> {
        // Stub: implementation will be in ff_ChannelCompound_h.rs
        unsafe {
            // This is a stub; the real ChannelCompound contains a BTreeSet
            static mut EMPTY_SET: std::collections::BTreeSet<*mut MultiEffect> = std::collections::BTreeSet::new();
            &mut EMPTY_SET
        }
    }
}

pub type Channel = BTreeMap<c_int, c_int>;

// NOTE: ChannelIterator is a wrapper around BTreeMap range iteration
// In C++, it's multimapIterator<FFChannelSet::Channel> which iterates
// over entries with a specific key
pub struct ChannelIterator {
    // Implementation would need access to the map and key for proper iteration
    // For now, this is a stub
    _private: (),
}

impl ChannelIterator {
    pub fn new(map: &mut Channel, channel: c_int) -> Self {
        ChannelIterator {
            _private: (),
        }
    }

    pub fn is_end(&self, channel_map: &Channel) -> bool {
        // Stub: Needs proper implementation with reference to actual iterator state
        false
    }

    pub fn get_device(&self) -> c_int {
        // Stub: Would return (**itChannel).second from C++ code
        0
    }

    pub fn next(&mut self) {
        // Stub: Advances iterator
    }
}

#[repr(C)]
pub struct FFChannelSet {
    // Base class FFMultiSet fields
    mConfig: *mut FFConfigParser,
    mSet: Vec<*mut FFSet>,
    mDevices: *mut CImmDevices,
    // Derived class member
    mChannel: Channel,
}

impl FFChannelSet {
    ////---------------------------
    /// FFChannelSet::ParseChannels
    //-------------------------------
    //	This is the worst hack of a parser ever devised.
    //
    pub fn ParseChannels(&mut self, channels: *const c_char) -> qboolean {
        if channels.is_null() {
            return 0; // qfalse
        }

        let mut channel: c_int = 0;
        let mut pos: *const c_char = channels;

        // for (	pos = channels
        // ;	pos && sscanf( pos, "%d", &channel ) == 1
        // ;)
        loop {
            if pos.is_null() {
                break;
            }

            // sscanf( pos, "%d", &channel ) == 1
            // SAFETY: sscanf call to parse integer from string
            let sscanf_result = unsafe {
                let mut temp_channel: c_int = 0;
                let fmt = b"%d\0".as_ptr() as *const c_char;
                let result = libc::sscanf(pos, fmt, &mut temp_channel as *mut c_int);
                if result == 1 {
                    channel = temp_channel;
                    1
                } else {
                    0
                }
            };

            if sscanf_result != 1 {
                break;
            }

            let mut device: c_int = 0;
            let mut endpos: *mut c_char;

            // endpos = strchr( pos, ';' );
            // SAFETY: strchr returns pointer or null
            endpos = unsafe {
                libc::strchr(pos as *mut c_char, b';' as c_int) as *mut c_char
            };

            if channel >= 0 && channel < 7 {
                // FF_CHANNEL_MAX = 7, from ff_ChannelCompound.h

                // for (	pos = strchr( pos, ',' )
                // ;	pos && ( !endpos || pos < endpos ) && sscanf( pos, " ,%d", &device ) == 1
                // ;	pos = strchr( pos + 1, ',' )
                // )

                // SAFETY: strchr call to find comma
                pos = unsafe {
                    libc::strchr(pos as *mut c_char, b',' as c_int) as *const c_char
                };

                loop {
                    if pos.is_null() {
                        break;
                    }

                    if !endpos.is_null() && pos >= endpos as *const c_char {
                        break;
                    }

                    // sscanf( pos, " ,%d", &device ) == 1
                    // SAFETY: sscanf call to parse device number
                    let sscanf_result = unsafe {
                        let mut temp_device: c_int = 0;
                        let fmt = b" ,%d\0".as_ptr() as *const c_char;
                        let result = libc::sscanf(pos, fmt, &mut temp_device as *mut c_int);
                        if result == 1 {
                            device = temp_device;
                            1
                        } else {
                            0
                        }
                    };

                    if sscanf_result != 1 {
                        break;
                    }

                    if device >= 0 && (device as usize) < self.mSet.len() {
                        // for (	ChannelIterator itChannel( mChannel, channel )
                        // ;	itChannel != mChannel.end()
                        // &&	(**itChannel).second != device	// found duplicate
                        // ;	++itChannel
                        // );

                        // Look for existing entry with this (channel, device) pair
                        let range_iter = self.mChannel.range(channel..=channel);
                        let mut found_duplicate = false;
                        for (_, &dev) in range_iter {
                            if dev == device {
                                found_duplicate = true;
                                break;
                            }
                        }

                        // Don't allow duplicates
                        if !found_duplicate {
                            // FFChannelSet::Channel::value_type Value( channel, device );
                            // Value.second = device;
                            // mChannel.insert( Value );
                            self.mChannel.insert(channel, device);
                        }
                    }

                    // pos = strchr( pos + 1, ',' )
                    // SAFETY: strchr call
                    pos = unsafe {
                        let next_pos = (pos as *mut c_char).add(1);
                        libc::strchr(next_pos, b',' as c_int) as *const c_char
                    };
                }
            }

            // pos = ( endpos ? endpos + 1 : NULL);
            pos = if !endpos.is_null() {
                unsafe { (endpos as *const c_char).add(1) }
            } else {
                core::ptr::null()
            };
        }

        // FIX ME -- return qfalse if there is a parse error
        1 // qtrue
    }

    ////----------------------
    /// FFChannelSet::Register
    //--------------------------
    //
    //	Assumptions:
    //	*	'compound' is empty of effects and contains the desired channel prior to entry.
    //
    //	Parameters:
    //	*	compound: its channel parameter is an input. its effect set is filled with registered
    //		- effects. 'compound' should not contain any effects prior to this function call.
    //	*	name: effect name to register in each FFSet on the channel
    //	*	create: qtrue if FFSet should create the effect, qfalse if it should just look it up.
    //
    pub fn Register(&mut self, compound: &mut ChannelCompound, name: *const c_char, create: qboolean) -> qboolean {
        let channel = compound.GetChannel();

        // for (	ChannelIterator itChannel( mChannel, compound.GetChannel() )
        // ;	itChannel != mChannel.end()
        // ;	++itChannel
        // )
        let range_iter = self.mChannel.range(channel..=channel);
        for (_, &device_index) in range_iter {
            if (device_index as usize) < self.mSet.len() {
                let device_set = self.mSet[device_index as usize];
                if !device_set.is_null() {
                    // MultiEffect *Effect;
                    // Effect = mSet[ (**itChannel).second ]->Register( name, create );
                    // SAFETY: dereference mSet[device_index]
                    let effect = unsafe {
                        (*device_set).Register(name, create)
                    };

                    if !effect.is_null() {
                        // compound.GetSet().insert( Effect );
                        let effect_set = compound.GetSet();
                        effect_set.insert(effect);
                    }
                }
            }
        }

        // return qboolean( compound.GetSet().size() != 0 );
        let effect_set = compound.GetSet();
        if effect_set.len() != 0 { 1 } else { 0 }
    }
}

// #ifdef FF_CONSOLECOMMAND
#[cfg(feature = "FF_CONSOLECOMMAND")]
impl FFChannelSet {
    pub fn GetDisplayTokens(Tokens: &mut TNameTable) {
        FFMultiSet::GetDisplayTokens(Tokens);
        Tokens.push("channels".to_string());
        Tokens.push("devices".to_string());
    }

    pub fn Display(&mut self, Unprocessed: &mut TNameTable, Processed: &mut TNameTable) {
        FFMultiSet::Display(self, Unprocessed, Processed);

        // for (	TNameTable::iterator itName = Unprocessed.begin()
        // ;	itName != Unprocessed.end()
        // ;)
        let mut i = 0;
        while i < Unprocessed.len() {
            // if ( stricmp( "channels", (*itName).c_str() ) == 0 )
            // SAFETY: C string comparison
            let is_channels = unsafe {
                let cstr_channels = b"channels\0".as_ptr() as *const c_char;
                let name_ptr = Unprocessed[i].as_ptr() as *const c_char;
                libc::strcasecmp(cstr_channels, name_ptr) == 0
            };

            if is_channels {
                // Com_Printf( "[available channels]\n" );
                unsafe {
                    Com_Printf(b"[available channels]\n\0".as_ptr() as *const c_char);
                }

                // for (	int i = 0
                // ;	i < FF_CHANNEL_MAX
                // ;	i++)
                for ch_idx in 0..7 {
                    // FF_CHANNEL_MAX = 7
                    // Com_Printf( "%d) %s  devices:", i, gChannelName[ i ] );
                    unsafe {
                        Com_Printf(
                            b"%d) %s  devices:\0".as_ptr() as *const c_char,
                            ch_idx,
                            gChannelName[ch_idx as usize],
                        );
                    }

                    // for (	ChannelIterator itChannel( mChannel, i )
                    // ;	itChannel != mChannel.end()
                    // ;	++itChannel
                    // )
                    let range_iter = self.mChannel.range(ch_idx..=ch_idx);
                    for (_, &device_idx) in range_iter {
                        // Com_Printf( " %d", (**itChannel).second );
                        unsafe {
                            Com_Printf(b" %d\0".as_ptr() as *const c_char, device_idx);
                        }
                    }

                    // Com_Printf( "\n" );
                    unsafe {
                        Com_Printf(b"\n\0".as_ptr() as *const c_char);
                    }
                }

                // Processed.push_back( *itName );
                Processed.push(Unprocessed[i].clone());

                // itName = Unprocessed.erase( itName );
                Unprocessed.remove(i);
                // Note: don't increment i after erase, as we want to check the next element
                // which is now at position i
            } else if {
                // else if ( stricmp( "devices", (*itName).c_str() ) == 0 )
                unsafe {
                    let cstr_devices = b"devices\0".as_ptr() as *const c_char;
                    let name_ptr = Unprocessed[i].as_ptr() as *const c_char;
                    libc::strcasecmp(cstr_devices, name_ptr) == 0
                }
            } {
                // Com_Printf( "[initialized devices]\n" );
                unsafe {
                    Com_Printf(b"[initialized devices]\n\0".as_ptr() as *const c_char);
                }

                // for (	int i = 0
                // ;	i < mDevices->GetNumDevices()
                // ;	i++)
                if !self.mDevices.is_null() {
                    let num_devices = unsafe {
                        (*self.mDevices).GetNumDevices()
                    };

                    for dev_idx in 0..num_devices {
                        // char ProductName[ FF_MAX_PATH ];
                        // ProductName[ 0 ] = 0;
                        // mDevices->GetDevice( i )->GetProductName( ProductName, FF_MAX_PATH - 1 );
                        let mut product_name = [0u8; 256]; // FF_MAX_PATH = 256 (assuming)
                        product_name[0] = 0;

                        unsafe {
                            let device = (*self.mDevices).GetDevice(dev_idx);
                            if !device.is_null() {
                                (*device).GetProductName(
                                    product_name.as_mut_ptr() as *mut c_char,
                                    255, // FF_MAX_PATH - 1
                                );
                            }
                        }

                        // Com_Printf( "%d) %s\n", i, ProductName );
                        unsafe {
                            Com_Printf(
                                b"%d) %s\n\0".as_ptr() as *const c_char,
                                dev_idx,
                                product_name.as_ptr() as *const c_char,
                            );
                        }
                    }
                }

                // Processed.push_back( *itName );
                Processed.push(Unprocessed[i].clone());

                // itName = Unprocessed.erase( itName );
                Unprocessed.remove(i);
                // Note: don't increment i after erase
            } else {
                // itName++;
                i += 1;
            }
        }
    }
}
// #endif // FF_CONSOLECOMMAND

// extern const char *gChannelName[];
// SAFETY: This external array is defined elsewhere in the codebase
extern "C" {
    pub static gChannelName: *const *const c_char;
}

// LOCAL STUB: Com_Printf function from engine
// SAFETY: This is a C stdio-like function from the engine
extern "C" {
    pub fn Com_Printf(fmt: *const c_char, ...) -> c_int;
}

// Re-export stubs for libc functions we're using
// (In actual code, these would come from libc crate)
mod libc {
    use core::ffi::c_char;
    use core::ffi::c_int;

    extern "C" {
        pub fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
        pub fn strchr(s: *const c_char, c: c_int) -> *const c_char;
        pub fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    }
}
