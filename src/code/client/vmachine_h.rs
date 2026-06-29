// vmachine.h -- virtual machine header for client

use core::ffi::{c_int, c_char, c_void};
use core::ptr::addr_of_mut;

/*
==================================================================

functions exported to the main executable

==================================================================
*/

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum cgameExport_t {
	CG_INIT = 0,
	CG_SHUTDOWN = 1,
	CG_CONSOLE_COMMAND = 2,
	CG_DRAW_ACTIVE_FRAME = 3,
	CG_CROSSHAIR_PLAYER = 4,
	CG_CAMERA_POS = 5,
	CG_CAMERA_ANG = 6,
	/*
	Ghoul2 Insert Start
	*/

	CG_RESIZE_G2_BOLT = 7,
	CG_RESIZE_G2 = 8,
	CG_RESIZE_G2_BONE = 9,
	CG_RESIZE_G2_SURFACE = 10,
	CG_RESIZE_G2_TEMPBONE = 11,
	/*
	Ghoul2 Insert End
	*/
	CG_DRAW_DATAPAD_HUD = 12,
	CG_DRAW_DATAPAD_OBJECTIVES = 13,
	CG_DRAW_DATAPAD_WEAPONS = 14,
	CG_DRAW_DATAPAD_INVENTORY = 15,
	CG_DRAW_DATAPAD_FORCEPOWERS = 16,
}

/*
==============================================================

VIRTUAL MACHINE

==============================================================
*/
#[repr(C)]
pub struct vm_s {
	pub entryPoint: extern "C" fn(c_int, ...) -> c_int,
}

pub type vm_t = vm_s;

extern "C" {
	pub static mut cgvm: vm_t;	// interface to cgame dll or vm
	pub static mut uivm: vm_t;	// interface to ui dll or vm

	pub fn VM_Call(callnum: c_int, ...) -> c_int;
	pub fn VM_DllSyscall(arg: c_int, ...) -> c_int;
	pub fn CL_ShutdownCGame();

	/*
	================
	VM_Create

	it will attempt to load as a system dll
	================
	*/
	pub fn Sys_LoadCgame(
		entryPoint: *mut extern "C" fn(c_int, ...) -> c_int,
		systemcalls: extern "C" fn(c_int, ...) -> c_int,
	) -> *mut c_void;
}

// Note: #include "../game/q_shared.h" from original is handled via module dependencies

#[inline]
pub fn VM_Create(module: *const c_char) -> *mut c_void {
	let mut res: *mut c_void;
	// try to load as a system dll
	unsafe {
		if libc::strcmp(module, b"cl\0".as_ptr() as *const i8) == 0 {
			res = Sys_LoadCgame(
				addr_of_mut!(cgvm.entryPoint),
				VM_DllSyscall,
			);
			if res.is_null() {
				//Com_Printf( "Failed.\n" );
				return core::ptr::null_mut();
			}
		} else {
			res = core::ptr::null_mut();
		}
	}

	res
}
