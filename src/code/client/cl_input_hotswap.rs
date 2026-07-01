/*
	TODO: finalize item support

	1) Make ItemSelectUp() work.
	2) Change cg.itemSelect to whatever var is used to store selected item.
	3) Make sure commands in itemCommands work in both multi & single player.
*/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int};

#[cfg(feature = "jk2mp")]
use crate::codemp::client::client_h::*;
#[cfg(feature = "jk2mp")]
use crate::codemp::cgame::cg_local_h::*;

#[cfg(not(feature = "jk2mp"))]
use crate::code::client::client_h::*;
#[cfg(not(feature = "jk2mp"))]
use crate::code::cgame::cg_local_h::*;

use crate::code::client::cl_input_hotswap_h::*;

// <stdio.h> is not included directly by this file, but sprintf() is used below;
// declare it as a trusted system libc function per porting rules.
extern "C" {
	fn sprintf(s: *mut c_char, fmt: *const c_char, ...) -> c_int;
}


// #ifdef _JK2MP
// #define FORCESELECTTIME forceSelectTime
// #define FORCESELECT		forceSelect
// #define INVSELECTTIME	invenSelectTime
// #define INVSELECT		itemSelect
// #define REGISTERSOUND	S_RegisterSound
// #define STARTSOUND		S_StartLocalSound
// #define WEAPONBINDSTR	"weaponclean"
// #else
// #define FORCESELECTTIME forcepowerSelectTime
// #define FORCESELECT		forcepowerSelect
// #define INVSELECTTIME	inventorySelectTime
// #define INVSELECT		inventorySelect
// #define REGISTERSOUND	cgi_S_RegisterSound
// #define STARTSOUND		cgi_S_StartLocalSound
// #define WEAPONBINDSTR	"weapon"
// #endif
//
// The FORCESELECT*/INVSELECT* macros stand for differing cg_t member names between
// _JK2MP and the single-player build; translated below as small cfg-gated accessor
// functions (Section 3: macros -> const/type/#[inline] fns) since Rust has no
// member-name macros. REGISTERSOUND/STARTSOUND stand for differing engine function
// names and are translated the same way. WEAPONBINDSTR stands for differing string
// literals and is translated as a cfg-gated byte-string const.

#[cfg(feature = "jk2mp")]
const WEAPONBINDSTR: &[u8] = b"weaponclean\0";
#[cfg(not(feature = "jk2mp"))]
const WEAPONBINDSTR: &[u8] = b"weapon\0";

#[cfg(feature = "jk2mp")]
#[inline]
unsafe fn FORCESELECTTIME() -> c_int {
	cg.forceSelectTime
}
#[cfg(not(feature = "jk2mp"))]
#[inline]
unsafe fn FORCESELECTTIME() -> c_int {
	cg.forcepowerSelectTime
}

#[cfg(feature = "jk2mp")]
#[inline]
unsafe fn FORCESELECT() -> c_int {
	cg.forceSelect
}
#[cfg(not(feature = "jk2mp"))]
#[inline]
unsafe fn FORCESELECT() -> c_int {
	showPowers[cg.forcepowerSelect as usize]
}

#[cfg(feature = "jk2mp")]
#[inline]
unsafe fn INVSELECTTIME() -> c_int {
	cg.invenSelectTime
}
#[cfg(not(feature = "jk2mp"))]
#[inline]
unsafe fn INVSELECTTIME() -> c_int {
	cg.inventorySelectTime
}

#[cfg(feature = "jk2mp")]
#[inline]
unsafe fn INVSELECT() -> c_int {
	cg.itemSelect
}
#[cfg(not(feature = "jk2mp"))]
#[inline]
unsafe fn INVSELECT() -> c_int {
	cg.inventorySelect
}

#[cfg(feature = "jk2mp")]
#[inline]
unsafe fn REGISTERSOUND(name: *const c_char) -> c_int {
	S_RegisterSound(name)
}
#[cfg(not(feature = "jk2mp"))]
#[inline]
unsafe fn REGISTERSOUND(name: *const c_char) -> c_int {
	cgi_S_RegisterSound(name)
}

#[cfg(feature = "jk2mp")]
#[inline]
unsafe fn STARTSOUND(sfxHandle: c_int, channelNum: c_int) {
	S_StartLocalSound(sfxHandle, channelNum)
}
#[cfg(not(feature = "jk2mp"))]
#[inline]
unsafe fn STARTSOUND(sfxHandle: c_int, channelNum: c_int) {
	cgi_S_StartLocalSound(sfxHandle, channelNum)
}

const BIND_TIME: c_int = 3000; //number of milliseconds button is held before binding
const EXEC_TIME: c_int = 500; //max ms button can be held to execute in bind mode


#[cfg(feature = "jk2mp")]
const itemCommands: [*const c_char; 12] = [
	core::ptr::null(),						//HI_NONE
	b"use_seeker\n\0".as_ptr() as *const c_char,
	b"use_field\n\0".as_ptr() as *const c_char,
	b"use_bacta\n\0".as_ptr() as *const c_char,
	b"use_bactabig\n\0".as_ptr() as *const c_char,
	b"use_electrobinoculars\n\0".as_ptr() as *const c_char,
	b"use_sentry\n\0".as_ptr() as *const c_char,
	b"use_jetpack\n\0".as_ptr() as *const c_char,
	core::ptr::null(),						//ammo dispenser
	core::ptr::null(),						//health dispenser
	b"use_eweb\n\0".as_ptr() as *const c_char,
	b"use_cloak\n\0".as_ptr() as *const c_char,
];
#[cfg(not(feature = "jk2mp"))]
const itemCommands: [*const c_char; 7] = [
	b"use_electrobinoculars\n\0".as_ptr() as *const c_char,
	b"use_bacta\n\0".as_ptr() as *const c_char,
	b"use_seeker\n\0".as_ptr() as *const c_char,
	b"use_goggles\n\0".as_ptr() as *const c_char,
	b"use_sentry\n\0".as_ptr() as *const c_char,
	core::ptr::null(),						//goodie key
	core::ptr::null(),						//security key
];



// Note: HotSwapManager (with its private fields down/noExec/noBind/forceBound/
// downTime/bindTime/uniqueID) is declared in cl_input_hotswap_h.rs (glob-imported
// above) rather than in this file, mirroring the C++ split between class
// declaration (.h) and out-of-line member definitions (.cpp).

impl HotSwapManager {
	// HotSwapManager::HotSwapManager(int uniqueID)
	pub fn new(uniqueID: c_int) -> Self {
		let mut manager = HotSwapManager {
			down: false, //not in the C++ member-init list; overwritten by Reset() below
			noExec: false, //not in the C++ member-init list; overwritten by Reset() below
			noBind: false, //not in the C++ member-init list; overwritten by Reset() below
			forceBound: false,
			downTime: 0, //not in the C++ member-init list; overwritten by Reset() below
			bindTime: 0, //not in the C++ member-init list; overwritten by Reset() below
			uniqueID,
		};
		manager.Reset();
		manager
	}


	fn GetBinding(&self) -> *mut c_char {
		unsafe {
			let mut buf: [c_char; 64] = [0; 64];

			sprintf(buf.as_mut_ptr(), b"hotswap%d\0".as_ptr() as *const c_char, self.uniqueID);
			let cvar = Cvar_Get(buf.as_ptr(), b"\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

			if !cvar.is_null() && *(*cvar).string != 0 {
				(*cvar).string
			} else {
				core::ptr::null_mut()
			}
		}
	}


	fn Bind(&mut self) {
		self.forceBound = false;

		unsafe {
			if self.WeaponSelectUp() {
				HotSwapBind(self.uniqueID, HOTSWAP_CAT_WEAPON, cg.weaponSelect);
			} else if self.ForceSelectUp() {
				self.forceBound = true;
				HotSwapBind(self.uniqueID, HOTSWAP_CAT_FORCE, FORCESELECT());
			} else if self.ItemSelectUp() {
				HotSwapBind(self.uniqueID, HOTSWAP_CAT_ITEM, INVSELECT());
			} else {
				assert!(false);
			}

			self.noExec = true;
			self.noBind = true;
			STARTSOUND(REGISTERSOUND(b"sound/interface/update\0".as_ptr() as *const c_char), 0);
		}
	}


	fn ForceSelectUp(&self) -> bool {
		unsafe {
			FORCESELECTTIME() != 0 &&
				(FORCESELECTTIME() + WEAPON_SELECT_TIME >= cg.time)
		}
	}


	fn WeaponSelectUp(&self) -> bool {
		unsafe {
			cg.weaponSelectTime != 0 &&
				(cg.weaponSelectTime + WEAPON_SELECT_TIME >= cg.time)
		}
	}


	fn ItemSelectUp(&self) -> bool {
		unsafe {
			INVSELECTTIME() != 0 &&
				(INVSELECTTIME() + WEAPON_SELECT_TIME >= cg.time)
		}
	}


	fn HUDInBindState(&self) -> bool {
		self.ForceSelectUp() || self.WeaponSelectUp() || self.ItemSelectUp()
	}


	pub fn Update(&mut self) {
		unsafe {
			if self.down {
				//Increment bindTime only if HUD is in select mode.
				if self.HUDInBindState() {
					self.bindTime += cls.frametime;
				} else {

					//Clear bind time.
					self.bindTime = 0;

					//If a force power is bound, want to execute whenever the button
					//is down to handle powers which can be held.
					if self.forceBound {
						self.Execute();
					}
				}
				self.downTime += cls.frametime;
			}

			//Down long enough, bind button.
			if !self.noBind && self.bindTime >= BIND_TIME {
				self.Bind();
			}
		}
	}


	fn Execute(&mut self) {
		let binding = self.GetBinding();
		if !binding.is_null() && !self.noExec {
			if !self.forceBound {
				self.noExec = true;
			}
			unsafe {
				Cbuf_ExecuteText(EXEC_APPEND, binding as *const c_char);
			}
		}
	}


	pub fn SetDown(&mut self) {
		//Set the down flag.
		self.down = true;

		//Execute the bind if the HUD isn't up.
		if !self.HUDInBindState() {
			self.Execute();
		}
	}


	pub fn SetUp(&mut self) {
		//Execute the bind if the button was held down for long enough.
		if self.downTime <= EXEC_TIME {
			self.Execute();
		}

		self.Reset();
	}


	fn Reset(&mut self) {
		self.down = false;
		self.downTime = 0;
		self.bindTime = 0;
		self.noExec = false;
		self.noBind = false;
	}
}


// static void HotSwapBind(const char *uniqueID, const char *value)
// Renamed from HotSwapBind to HotSwapBind_internal: C++ overload resolution (this
// 2-arg static/file-local function vs. the public 3-arg HotSwapBind below) has no
// direct Rust equivalent since Rust does not support free-function overloading.
unsafe fn HotSwapBind_internal(uniqueID: *const c_char, value: *const c_char) {
	Cvar_Set(uniqueID, value);
}


pub fn HotSwapBind(buttonID: c_int, category: c_int, value: c_int) {
	unsafe {
		let mut uniqueID: [c_char; 64] = [0; 64];
		sprintf(uniqueID.as_mut_ptr(), b"hotswap%d\0".as_ptr() as *const c_char, buttonID);

		match category {
			HOTSWAP_CAT_WEAPON => {
				HotSwapBind_internal(uniqueID.as_ptr(), va(b"%s %d\n\0".as_ptr() as *const c_char, WEAPONBINDSTR.as_ptr() as *const c_char, value));
			}
			HOTSWAP_CAT_ITEM => {
				// Unchecked indexing preserved to match the original C array subscript
				// itemCommands[value] (no bounds check in the source).
				assert!(!(*itemCommands.as_ptr().add(value as usize)).is_null());
				HotSwapBind_internal(uniqueID.as_ptr(), *itemCommands.as_ptr().add(value as usize));
			}
			HOTSWAP_CAT_FORCE => {
				HotSwapBind_internal(uniqueID.as_ptr(), va(b"useGivenForce %d\n\0".as_ptr() as *const c_char, value));
			}
			_ => {
				assert!(false);
			}
		}
	}
}
