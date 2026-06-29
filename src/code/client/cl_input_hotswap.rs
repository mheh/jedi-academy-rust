/*
	TODO: finalize item support

	1) Make ItemSelectUp() work.
	2) Change cg.itemSelect to whatever var is used to store selected item.
	3) Make sure commands in itemCommands work in both multi & single player.
*/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Note: These includes represent the original C header dependencies.
// Actual implementations would need to be ported separately or linked via FFI.

extern "C" {
	pub static mut cg: cg_t;
	pub static mut cls: clientStatic_t;
	pub static mut showPowers: [c_int; 16]; // MAX_SHOWPOWERS

	pub fn Cvar_Get(var_name: *const c_char, default_value: *const c_char, flags: c_int) -> *mut cvar_t;
	pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);
	pub fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
	pub fn va(fmt: *const c_char, ...) -> *const c_char;
	pub fn S_RegisterSound(name: *const c_char) -> c_int;
	pub fn S_StartLocalSound(sfxHandle: c_int, channelNum: c_int);
	pub fn cgi_S_RegisterSound(name: *const c_char) -> c_int;
	pub fn cgi_S_StartLocalSound(sfxHandle: c_int, channelNum: c_int);
	pub fn sprintf(s: *mut c_char, fmt: *const c_char, ...) -> c_int;
}

// Forward declarations of external types
#[repr(C)]
pub struct cvar_t {
	pub name: *mut c_char,
	pub string: *mut c_char,
	pub resetString: *mut c_char,
	pub latchedString: *mut c_char,
	pub flags: c_int,
	pub modified: c_int,
	pub modificationCount: c_int,
	pub value: f32,
	pub integer: c_int,
	pub next: *mut cvar_t,
}

#[cfg(feature = "jk2mp")]
#[repr(C)]
pub struct cg_t {
	// JK2MP version of cg_t
	// Minimal placeholder - actual structure would be much larger
	pub snap: *mut c_void,
	pub weaponSelect: c_int,
	pub weaponSelectTime: c_int,
	pub forceSelect: c_int,
	pub forceSelectTime: f32,
	pub itemSelect: c_int,
	pub invenSelectTime: f32,
	pub time: c_int,
	// ... more fields would follow
	_opaque: [u8; 0],
}

#[cfg(not(feature = "jk2mp"))]
#[repr(C)]
pub struct cg_t {
	// Non-JK2MP version of cg_t
	// Minimal placeholder - actual structure would be much larger
	pub snap: *mut c_void,
	pub weaponSelect: c_int,
	pub weaponSelectTime: c_int,
	pub forcepowerSelect: c_int,
	pub forcepowerSelectTime: c_int,
	pub inventorySelect: c_int,
	pub inventorySelectTime: c_int,
	pub time: c_int,
	// ... more fields would follow
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct clientStatic_t {
	pub frametime: c_int,
	// ... more fields would follow
	_opaque: [u8; 0],
}

// Conditional compilation mapping to the original C code
#[cfg(feature = "jk2mp")]
const WEAPONBINDSTR: &[u8] = b"weaponclean";

#[cfg(not(feature = "jk2mp"))]
const WEAPONBINDSTR: &[u8] = b"weapon";

const BIND_TIME: c_int = 3000; //number of milliseconds button is held before binding
const EXEC_TIME: c_int = 500;  //max ms button can be held to execute in bind mode
const EXEC_APPEND: c_int = 2;

#[cfg(feature = "jk2mp")]
const ITEMCOMMANDS_LEN: usize = 12; // HI_NUM_HOLDABLE
#[cfg(not(feature = "jk2mp"))]
const ITEMCOMMANDS_LEN: usize = 7;  // INV_MAX

const HOTSWAP_CAT_WEAPON: c_int = 0;
const HOTSWAP_CAT_ITEM: c_int = 1;
const HOTSWAP_CAT_FORCE: c_int = 2;

const WEAPON_SELECT_TIME: c_int = 1400;

#[cfg(feature = "jk2mp")]
const ITEMCOMMANDS: &[Option<&[u8]>] = &[
	None,						//HI_NONE
	Some(b"use_seeker\n"),
	Some(b"use_field\n"),
	Some(b"use_bacta\n"),
	Some(b"use_bactabig\n"),
	Some(b"use_electrobinoculars\n"),
	Some(b"use_sentry\n"),
	Some(b"use_jetpack\n"),
	None,						//ammo dispenser
	None,						//health dispenser
	Some(b"use_eweb\n"),
	Some(b"use_cloak\n"),
];

#[cfg(not(feature = "jk2mp"))]
const ITEMCOMMANDS: &[Option<&[u8]>] = &[
	Some(b"use_electrobinoculars\n"),
	Some(b"use_bacta\n"),
	Some(b"use_seeker\n"),
	Some(b"use_goggles\n"),
	Some(b"use_sentry\n"),
	None,						//goodie key
	None,						//security key
];

#[repr(C)]
pub struct HotSwapManager {
	down: bool,		//Is the button down?
	noExec: bool,	//Don't execute the button's bind.
	noBind: bool,	//Don't bind the button.
	forceBound: bool,//Is a force power currently bound?
	downTime: c_int,	//How long the button has been held down.
	bindTime: c_int,	//How long the button has been down with the selection up.
	uniqueID: c_int,	//Unique ID for this button.
}

impl HotSwapManager {
	pub fn new(uniqueID: c_int) -> Self {
		let mut manager = HotSwapManager {
			uniqueID,
			forceBound: false,
			down: false,
			noExec: false,
			noBind: false,
			downTime: 0,
			bindTime: 0,
		};
		manager.Reset();
		manager
	}

	fn GetBinding(&self) -> *mut c_char {
		unsafe {
			let mut buf: [c_char; 64] = [0; 64];

			sprintf(buf.as_mut_ptr(), b"hotswap%d\0".as_ptr() as *const c_char, self.uniqueID);
			let cvar = Cvar_Get(buf.as_ptr(), b"\0".as_ptr() as *const c_char, 1); // CVAR_ARCHIVE

			if !cvar.is_null() && !(*cvar).string.is_null() {
				if *(*cvar).string != 0 {
					return (*cvar).string;
				}
			}
			core::ptr::null_mut()
		}
	}

	fn ForceSelectUp(&self) -> bool {
		unsafe {
			let force_select_time = self.get_force_select_time();
			force_select_time != 0 &&
				(force_select_time + WEAPON_SELECT_TIME >= cg.time)
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
			let inv_select_time = self.get_inv_select_time();
			inv_select_time != 0 &&
				(inv_select_time + WEAPON_SELECT_TIME >= cg.time)
		}
	}

	fn HUDInBindState(&self) -> bool {
		self.ForceSelectUp() || self.WeaponSelectUp() || self.ItemSelectUp()
	}

	fn Bind(&mut self) {
		self.forceBound = false;

		if self.WeaponSelectUp() {
			unsafe {
				HotSwapBind(self.uniqueID, HOTSWAP_CAT_WEAPON, cg.weaponSelect);
			}
		} else if self.ForceSelectUp() {
			self.forceBound = true;
			let force_idx = unsafe { self.get_force_select_value() };
			HotSwapBind(self.uniqueID, HOTSWAP_CAT_FORCE, force_idx);
		} else if self.ItemSelectUp() {
			let inv_select = unsafe { self.get_inv_select_value() };
			HotSwapBind(self.uniqueID, HOTSWAP_CAT_ITEM, inv_select);
		} else {
			assert!(false, "HotSwapManager::Bind: No selection active");
		}

		self.noExec = true;
		self.noBind = true;
		unsafe {
			let sfx = Self::register_sound(b"sound/interface/update\0".as_ptr() as *const c_char);
			Self::start_sound(sfx, 0);
		}
	}

	fn Execute(&mut self) {
		let binding = self.GetBinding();
		if !binding.is_null() && !self.noExec {
			if !self.forceBound {
				self.noExec = true;
			}
			unsafe {
				Cbuf_ExecuteText(EXEC_APPEND, binding);
			}
		}
	}

	fn Reset(&mut self) {
		self.down = false;
		self.downTime = 0;
		self.bindTime = 0;
		self.noExec = false;
		self.noBind = false;
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

	pub fn ButtonDown(&self) -> bool {
		self.down
	}

	// Helper method to get force select time based on JK2MP config
	#[cfg(feature = "jk2mp")]
	unsafe fn get_force_select_time(&self) -> c_int {
		cg.forceSelectTime as c_int
	}

	#[cfg(not(feature = "jk2mp"))]
	unsafe fn get_force_select_time(&self) -> c_int {
		cg.forcepowerSelectTime
	}

	// Helper method to get inventory select time based on JK2MP config
	#[cfg(feature = "jk2mp")]
	unsafe fn get_inv_select_time(&self) -> c_int {
		cg.invenSelectTime as c_int
	}

	#[cfg(not(feature = "jk2mp"))]
	unsafe fn get_inv_select_time(&self) -> c_int {
		cg.inventorySelectTime
	}

	// Helper method to get force select value based on JK2MP config
	#[cfg(feature = "jk2mp")]
	unsafe fn get_force_select_value(&self) -> c_int {
		cg.forceSelect
	}

	#[cfg(not(feature = "jk2mp"))]
	unsafe fn get_force_select_value(&self) -> c_int {
		showPowers[cg.forcepowerSelect as usize]
	}

	// Helper method to get inventory select value based on JK2MP config
	#[cfg(feature = "jk2mp")]
	unsafe fn get_inv_select_value(&self) -> c_int {
		cg.itemSelect
	}

	#[cfg(not(feature = "jk2mp"))]
	unsafe fn get_inv_select_value(&self) -> c_int {
		cg.inventorySelect
	}

	// Helper methods for sound functions based on JK2MP config
	#[cfg(feature = "jk2mp")]
	unsafe fn register_sound(name: *const c_char) -> c_int {
		S_RegisterSound(name)
	}

	#[cfg(not(feature = "jk2mp"))]
	unsafe fn register_sound(name: *const c_char) -> c_int {
		cgi_S_RegisterSound(name)
	}

	#[cfg(feature = "jk2mp")]
	unsafe fn start_sound(sfxHandle: c_int, channelNum: c_int) {
		S_StartLocalSound(sfxHandle, channelNum)
	}

	#[cfg(not(feature = "jk2mp"))]
	unsafe fn start_sound(sfxHandle: c_int, channelNum: c_int) {
		cgi_S_StartLocalSound(sfxHandle, channelNum)
	}
}

fn HotSwapBind_internal(uniqueID: *const c_char, value: *const c_char) {
	unsafe {
		Cvar_Set(uniqueID, value);
	}
}

pub fn HotSwapBind(buttonID: c_int, category: c_int, value: c_int) {
	unsafe {
		let mut uniqueID: [c_char; 64] = [0; 64];
		sprintf(uniqueID.as_mut_ptr(), b"hotswap%d\0".as_ptr() as *const c_char, buttonID);

		match category {
		HOTSWAP_CAT_WEAPON => {
			let cmd_str = va(
				b"%s %d\n\0".as_ptr() as *const c_char,
				WEAPONBINDSTR.as_ptr() as *const c_char,
				value,
			);
			HotSwapBind_internal(uniqueID.as_ptr(), cmd_str);
		}
		HOTSWAP_CAT_ITEM => {
			if value >= 0 && (value as usize) < ITEMCOMMANDS.len() {
				if let Some(cmd) = ITEMCOMMANDS[value as usize] {
					HotSwapBind_internal(uniqueID.as_ptr(), cmd.as_ptr() as *const c_char);
				} else {
					assert!(false, "itemCommands[value] is NULL");
				}
			}
		}
		HOTSWAP_CAT_FORCE => {
			let cmd_str = va(
				b"useGivenForce %d\n\0".as_ptr() as *const c_char,
				value,
			);
			HotSwapBind_internal(uniqueID.as_ptr(), cmd_str);
		}
		_ => {
			assert!(false, "HotSwapBind: invalid category");
		}
		}
	}
}
