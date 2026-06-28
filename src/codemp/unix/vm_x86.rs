#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;

#[repr(C)]
pub struct vm_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct vmHeader_t {
    _unused: [u8; 0],
}

#[no_mangle]
pub extern "C" fn VM_Compile(_vm: *mut vm_t, _header: *mut vmHeader_t) {}

#[no_mangle]
pub extern "C" fn VM_CallCompiled(_vm: *mut vm_t, _args: *mut c_int) -> c_int {
    unsafe { core::hint::unreachable_unchecked() }
}
