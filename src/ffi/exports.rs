//! The two C symbols the engine calls into. These are the only `#[no_mangle]`
//! entry points the module exposes.

use core::ffi::c_int;

use crate::codemp::game::g_main;
use crate::ffi::syscalls::{self, SyscallFn};

/// `dllEntry` — engine → game, called once at load to hand over the syscall
/// pointer the module uses for every callback into the engine.
#[no_mangle]
pub unsafe extern "C" fn dllEntry(syscallptr: SyscallFn) {
    syscalls::set_syscall(syscallptr);
}

/// `vmMain` — engine → game, the primary entry point.
///
/// `command` is a [`crate::ffi::GameExport`]; the twelve `argN` are
/// `intptr_t`-width (so a 64-bit engine can pass pointers through them) and the
/// return value is likewise `intptr_t`. On a 32-bit build `isize` collapses to
/// `i32`, matching the original Raven JKA signature exactly.
///
/// Any Rust panic is caught here: unwinding across the FFI boundary into the C
/// engine is undefined behaviour, so a panic becomes a `0` return.
#[no_mangle]
pub unsafe extern "C" fn vmMain(
    command: c_int,
    arg0: isize,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
    arg5: isize,
    arg6: isize,
    arg7: isize,
    arg8: isize,
    arg9: isize,
    arg10: isize,
    arg11: isize,
) -> isize {
    let args = [
        arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11,
    ];
    std::panic::catch_unwind(|| g_main::vm_main(command, &args)).unwrap_or(0)
}
