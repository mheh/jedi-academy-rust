//! Internal helper macros.

/// Invoke an engine syscall through the pointer the engine handed us in `dllEntry`.
///
/// Every argument is widened to `isize` (== C `intptr_t`) to match the engine's
/// `intptr_t (QDECL *)(intptr_t arg, ...)` cdecl-variadic syscall ABI. This is the
/// single point through which the game module calls back into the engine.
///
/// # Safety
/// Must be invoked inside an `unsafe` block. The caller is responsible for passing
/// argument types and a count that match the trap identified by `$id` — see the
/// trap signatures in `refs/raven-jediacademy/codemp/game/g_syscalls.c`.
macro_rules! syscall {
    ($id:expr $(, $arg:expr )* $(,)?) => {
        $crate::ffi::syscalls::raw_syscall()( $id as isize $(, $arg as isize )* )
    };
}
