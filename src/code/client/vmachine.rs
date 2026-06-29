// vmachine.rs -- wrapper to fake virtual machine for client

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use core::ffi::c_int;
use std::ptr::addr_of_mut;

/*
==================================================================

functions exported to the main executable

==================================================================
*/

#[repr(C)]
#[derive(Clone, Copy)]
pub enum cgameExport_t {
	CG_INIT,
	CG_SHUTDOWN,
	CG_CONSOLE_COMMAND,
	CG_DRAW_ACTIVE_FRAME,
	CG_CROSSHAIR_PLAYER,
	CG_CAMERA_POS,
	CG_CAMERA_ANG,
	/*
	Ghoul2 Insert Start
	*/

	CG_RESIZE_G2_BOLT,
	CG_RESIZE_G2,
	CG_RESIZE_G2_BONE,
	CG_RESIZE_G2_SURFACE,
	CG_RESIZE_G2_TEMPBONE,
	/*
	Ghoul2 Insert End
	*/
	CG_DRAW_DATAPAD_HUD,
	CG_DRAW_DATAPAD_OBJECTIVES,
	CG_DRAW_DATAPAD_WEAPONS,
	CG_DRAW_DATAPAD_INVENTORY,
	CG_DRAW_DATAPAD_FORCEPOWERS,
}

/*
==============================================================

VIRTUAL MACHINE

==============================================================
*/

#[repr(C)]
pub struct vm_s {
	pub entryPoint: Option<extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int) -> c_int>,
}

pub type vm_t = vm_s;

// interface to cgame dll or vm
pub static mut cgvm: vm_t = vm_s {
	entryPoint: None,
};

// interface to ui dll or vm
pub static mut uivm: vm_t = vm_s {
	entryPoint: None,
};

// #pragma warning (disable : 4514)
/*
==============================================================

VIRTUAL MACHINE

==============================================================
*/

pub fn VM_Call(callnum: c_int, arg1: c_int, arg2: c_int, arg3: c_int, arg4: c_int, arg5: c_int, arg6: c_int, arg7: c_int, arg8: c_int, arg9: c_int) -> c_int {
	//	assert (cgvm.entryPoint);

	unsafe {
		if let Some(entry) = (*addr_of_mut!(cgvm)).entryPoint {
			return entry(callnum, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
		}
	}

	-1
}

/*
============
VM_DllSyscall

we pass this to the cgame dll to call back into the client
============
*/
extern "C" {
	pub fn CL_CgameSystemCalls(args: *mut c_int) -> c_int;
	pub fn CL_UISystemCalls(args: *mut c_int) -> c_int;
	pub fn CL_ShutdownCGame();
}

pub fn VM_DllSyscall(arg: c_int, arg1: c_int, arg2: c_int, arg3: c_int, arg4: c_int, arg5: c_int, arg6: c_int, arg7: c_int, arg8: c_int, arg9: c_int) -> c_int {
	//	return cgvm->systemCall( &arg );
	unsafe {
		CL_CgameSystemCalls((&arg as *const c_int) as *mut c_int)
	}
}
