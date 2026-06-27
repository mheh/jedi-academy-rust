//! The game→engine syscall pointer and the primitives that use it.

use core::sync::atomic::{AtomicUsize, Ordering};

/// The engine syscall entry point: `intptr_t (QDECL *)(intptr_t arg, ...)`.
///
/// It is genuinely **variadic** and cdecl on every platform JKA targets, so the
/// type must be declared variadic. On x86_64 SysV and AArch64 the calling
/// convention for variadic functions differs from fixed-arity ones; declaring a
/// fixed-arity pointer here would corrupt 64-bit calls. `isize` == C `intptr_t`,
/// which is why this same type is correct for both 32- and 64-bit engines.
pub type SyscallFn = unsafe extern "C" fn(arg: isize, ...) -> isize;

/// The engine syscall pointer, stored as `usize` (0 == not yet set). The engine
/// calls `dllEntry` exactly once, before any `vmMain` command, to install it.
static SYSCALL: AtomicUsize = AtomicUsize::new(0);

/// Record the engine syscall pointer. Called from `dllEntry`.
pub(crate) fn set_syscall(f: SyscallFn) {
    SYSCALL.store(f as usize, Ordering::SeqCst);
}

/// Fetch the engine syscall pointer.
///
/// # Safety
/// Only valid after `dllEntry` has installed the pointer (i.e. when called from
/// within a `vmMain` dispatch). The returned value is a variadic C function;
/// calling it is itself `unsafe` and the argument list must match the trap.
#[inline]
pub(crate) fn raw_syscall() -> SyscallFn {
    let p = SYSCALL.load(Ordering::SeqCst);
    debug_assert!(p != 0, "engine syscall pointer used before dllEntry()");
    // SAFETY: `set_syscall` only ever stores a valid `SyscallFn` cast to `usize`.
    unsafe { core::mem::transmute::<usize, SyscallFn>(p) }
}

/// Reinterpret an `f32` as an integer-width syscall argument, mirroring the C
/// `PASSFLOAT` helper, so float arguments survive the integer-only syscall ABI.
#[inline]
pub fn pass_float(f: f32) -> isize {
    f.to_bits() as i32 as isize
}
