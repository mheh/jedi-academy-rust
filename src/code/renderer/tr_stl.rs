// leave this as first line for PCH reasons...
//

// Filename:-	tr_stl.rs
//
// I mainly made this file because I was getting sick of all the stupid error messages in MS's STL implementation,
//	and didn't want them showing up in the renderer files they were used in. This way keeps them more or less invisible
//	because of minimal dependancies
//

use std::collections::HashMap;
use core::ffi::{c_char, c_int};

// Type alias: map<sstring_t, const char *>
type ShaderEntryPtrs_t = HashMap<String, *const c_char>;

// Global shader entry pointers map, zero-initialized
static mut ShaderEntryPtrs: Option<ShaderEntryPtrs_t> = None;


pub fn ShaderEntryPtrs_Clear() {
	unsafe {
		match ShaderEntryPtrs {
			Some(ref mut map) => map.clear(),
			None => {
				ShaderEntryPtrs = Some(HashMap::new());
			}
		}
	}
}


pub fn ShaderEntryPtrs_Size() -> c_int {
	unsafe {
		match &ShaderEntryPtrs {
			Some(map) => map.len() as c_int,
			None => 0,
		}
	}
}


pub fn ShaderEntryPtrs_Insert(token: *const c_char, p: *const c_char) {
	unsafe {
		if ShaderEntryPtrs.is_none() {
			ShaderEntryPtrs = Some(HashMap::new());
		}

		if let Some(ref mut map) = ShaderEntryPtrs {
			let key_str = std::ffi::CStr::from_ptr(token).to_string_lossy().to_string();

			if map.get(&key_str).is_none() {
				map.insert(key_str, p);
			} else {
				VID_Printf( PRINT_DEVELOPER, c"Duplicate shader entry %s!\n".as_ptr(), token );
			}
		}
	}
}



// returns NULL if not found...
//
pub fn ShaderEntryPtrs_Lookup(psShaderName: *const c_char) -> *const c_char {
	unsafe {
		match &ShaderEntryPtrs {
			Some(map) => {
				let key_str = std::ffi::CStr::from_ptr(psShaderName).to_string_lossy().to_string();
				match map.get(&key_str) {
					Some(&p) => p,
					None => std::ptr::null(),
				}
			}
			None => std::ptr::null(),
		}
	}
}



// Stub for external renderer function
extern "C" {
	pub fn VID_Printf( level: c_int, fmt: *const c_char, ... );
}

const PRINT_DEVELOPER: c_int = 1;
