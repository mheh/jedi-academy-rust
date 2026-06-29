#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Copyright (C) 1994-2000, RAD Game Tools, Inc.

// Platform detection guide (from original):
//  __RAD16__ means 16 bit code (Win16)
//  __RAD32__ means 32 bit code (DOS, Win386, Win32s, Mac)
//
//  __RADDOS__ means DOS code (16 or 32 bit)
//  __RADWIN__ means Windows code (Win16, Win386, Win32s)
//  __RADWINEXT__ means Windows 386 extender (Win386)
//  __RADNT__ means Win32s code
//  __RADMAC__ means Macintosh
//
//  __RADX86__ means Intel x86
//  __RADMMX__ means Intel x86 MMX instructions are allowed
//  __RAD68K__ means 68K
//  __RADPPC__ means PowerPC
//
// __RADLITTLEENDIAN__ means processor is little-endian (x86)
// __RADBIGENDIAN__ means processor is big-endian (680x0, PPC)
//
// __RADALLOWINLINES__ means this compiler allows inline function declarations
//                     use RADINLINE for the appropriate keyword

// Type definitions

pub type s8 = i8;
pub type u8 = u8;
pub type u32 = u32;
pub type s32 = i32;
pub type f32 = f32;
pub type f64 = f64;

// Compiler and platform detection would normally set these defines.
// For Rust targets, we assume:
// - 32-bit code (__RAD32__)
// - x86/x86_64 architecture (__RADX86__)
// - little-endian (__RADLITTLEENDIAN__)
// - Windows/Win32 (__RADNT__)

pub type u16 = u16;
pub type s16 = i16;

pub type u64 = u64;
pub type s64 = i64;

// const ptr4_defined = true; // 32-bit implementations

// Type aliases for compatibility

// Calling convention aliases (not directly representable in Rust, kept for doc purposes)
// RADLINK -> __stdcall on Win32, __pascal elsewhere
// RADEXPLINK -> __stdcall on Win32, __far __pascal elsewhere
// RADEXPFUNC -> extern "C" on C++ side, empty on C
// RADASMLINK -> __cdecl

// String and memory operation declarations for platform-specific implementations.
// These would originally be inline assembly or pragma aux declarations for different
// compilers (Watcom, MWERKS, MSC, etc.). In Rust, we declare them as extern "C"
// to be linked against platform-specific implementations or replaced with Rust equivalents.

#[cfg(target_os = "macos")]
pub use mac_builtins::*;

#[cfg(not(target_os = "macos"))]
pub use generic_builtins::*;

#[cfg(target_os = "macos")]
mod mac_builtins {
    use super::*;

    // Mac-specific inline function declarations that would be implemented
    // via intrinsics.h or assembly on the actual platform

    extern "C" {
        pub fn radconv32a(p: *mut core::ffi::c_void, n: u32);
        pub fn radloadu32(a: u32) -> u32;
        pub fn radloadu32ptr(p: *const u32) -> u32;
        pub fn radsqr(a: u32) -> u32;
        pub fn mult64anddiv(m1: u32, m2: u32, d: u32) -> u32;
        pub fn radabs(ab: i32) -> i32;
        pub fn radmemset16(dest: *mut core::ffi::c_void, value: u16, size: u32);
        pub fn RADCycleTimerAvail() -> u32;
        pub fn RADCycleTimerStartAddr(addr: *mut u32);
        pub fn RADCycleTimerDeltaAddr(addr: *mut u32) -> u32;
        pub fn RADCycleTimerStartAddr64(addr: *mut u64);
        pub fn RADCycleTimerDeltaAddr64(addr: *mut u64);
    }

    pub use libc::{
        strcpy as radstrcpy,
        strcat as radstrcat,
        memcpy as radmemcpy,
        memcmp as radmemcmp,
        memset as radmemset,
        strlen as radstrlen,
        strchr as radstrchr,
        toupper as radtoupper,
        strcmp as radstrcmp,
    };

    // radstru32 and radmemcpydb (BlockMoveData) need custom handling

    #[inline]
    pub fn radstru32(s: *const core::ffi::c_char) -> u32 {
        // Would call atol in C; need to parse string to u32
        // Stub for now - actual implementation converts C string to u32
        0
    }

    #[inline]
    pub fn radmemcpydb(
        dest: *mut core::ffi::c_void,
        source: *const core::ffi::c_void,
        size: u32,
    ) {
        // BlockMoveData on Mac; equivalent to memcpy but handles overlaps
        unsafe {
            core::ptr::copy_nonoverlapping(source as *const u8, dest as *mut u8, size as usize);
        }
    }

    // Macros as const or inline functions:
    #[inline(always)]
    pub fn BreakPoint() {
        // DebugStr("\pBreakPoint() was called") - platform-specific debugger break
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::x86_64::__debugbreak();
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            panic!("BreakPoint");
        }
    }

    // Macro emulations for RADCycleTimer operations
    // These would be preprocessor macros calling the Addr functions:
    // #define RADCycleTimerStart(var) RADCycleTimerStartAddr(&var)
    // #define RADCycleTimerDelta(var) RADCycleTimerDeltaAddr(&var)
    // etc. - handled via explicit function calls in Rust code
}

#[cfg(not(target_os = "macos"))]
mod generic_builtins {
    use super::*;

    // Generic/Win32 implementations

    #[inline(always)]
    pub fn radconv32a(_p: *mut core::ffi::c_void, _n: u32) {
        // No-op for non-Mac platforms in this fallback
    }

    #[inline(always)]
    pub fn radloadu32(a: u32) -> u32 {
        // Simple identity for little-endian x86
        a
    }

    #[inline(always)]
    pub fn radloadu32ptr(p: *const u32) -> u32 {
        unsafe { *p }
    }

    pub use libc::{
        strcpy as radstrcpy,
        strcat as radstrcat,
        memcpy as radmemcpy,
        memmove as radmemcpydb,
        memcmp as radmemcmp,
        memset as radmemset,
        strlen as radstrlen,
        strchr as radstrchr,
        toupper as radtoupper,
        strcmp as radstrcmp,
    };

    #[inline]
    pub fn radstru32(s: *const core::ffi::c_char) -> u32 {
        // Stub: parse C string to u32 (would use atol in C)
        0
    }

    #[inline]
    pub fn radmemset16(dest: *mut core::ffi::c_void, value: u16, size: u32) {
        // Set 16-bit values across memory
        unsafe {
            let ptr = dest as *mut u16;
            for i in 0..(size as usize / 2) {
                *ptr.add(i) = value;
            }
        }
    }

    #[inline]
    pub fn radsqr(a: u32) -> u32 {
        a.wrapping_mul(a)
    }

    #[inline]
    pub fn mult64anddiv(m1: u32, m2: u32, d: u32) -> u32 {
        // 64-bit multiply then divide: (m1 * m2) / d
        let prod = (m1 as u64).wrapping_mul(m2 as u64);
        (prod / d as u64) as u32
    }

    #[inline]
    pub fn radabs(ab: i32) -> i32 {
        if ab < 0 { -ab } else { ab }
    }

    pub fn RADCycleTimerAvail() -> u32 {
        // rdtsc availability check - would use CPUID in actual code
        1 // Assume available on modern systems
    }

    pub fn RADCycleTimerStartAddr(addr: *mut u32) {
        // Read RDTSC and store at addr
        // Stub: Would read TSC register
        unsafe { *addr = 0; }
    }

    pub fn RADCycleTimerDeltaAddr(addr: *mut u32) -> u32 {
        // Read RDTSC, compute delta from stored value, store new value
        // Stub: Would read TSC register
        unsafe {
            let delta = 0u32.wrapping_sub(*addr);
            *addr = delta;
            delta
        }
    }

    pub fn RADCycleTimerStartAddr64(addr: *mut u64) {
        // 64-bit version of RADCycleTimerStartAddr
        unsafe { *addr = 0; }
    }

    pub fn RADCycleTimerDeltaAddr64(addr: *mut u64) {
        // 64-bit version of RADCycleTimerDeltaAddr
        unsafe { *addr = 0; }
    }

    #[inline]
    pub fn BreakPoint() {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::x86_64::__debugbreak();
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            panic!("BreakPoint");
        }
    }
}

// Memory allocation interface
pub type RADMEMALLOC = extern "C" fn(bytes: u32) -> *mut core::ffi::c_void;
pub type RADMEMFREE = extern "C" fn(ptr: *mut core::ffi::c_void);

// Memory management functions
extern "C" {
    pub fn RADSetMemory(a: RADMEMALLOC, f: RADMEMFREE);
    pub fn radmalloc(numbytes: u32) -> *mut core::ffi::c_void;
    pub fn radfree(ptr: *mut core::ffi::c_void);
}

// DOS-specific timer functions (conditional on platform)
#[cfg(all(target_os = "windows", target_pointer_width = "16"))]
pub mod dos_timers {
    use super::*;

    extern "C" {
        pub static mut RADTimerSetupAddr: *mut core::ffi::c_void;
        pub static mut RADTimerReadAddr: *mut core::ffi::c_void;
        pub static mut RADTimerDoneAddr: *mut core::ffi::c_void;
    }

    pub type RADTimerSetupType = extern "C" fn();
    pub type RADTimerReadType = extern "C" fn() -> u32;
    pub type RADTimerDoneType = extern "C" fn();

    #[inline]
    pub fn RADTimerSetup() {
        unsafe {
            if !RADTimerSetupAddr.is_null() {
                let f: RADTimerSetupType = core::mem::transmute(RADTimerSetupAddr);
                f();
            }
        }
    }

    #[inline]
    pub fn RADTimerRead() -> u32 {
        unsafe {
            if !RADTimerReadAddr.is_null() {
                let f: RADTimerReadType = core::mem::transmute(RADTimerReadAddr);
                f()
            } else {
                0
            }
        }
    }

    #[inline]
    pub fn RADTimerDone() {
        unsafe {
            if !RADTimerDoneAddr.is_null() {
                let f: RADTimerDoneType = core::mem::transmute(RADTimerDoneAddr);
                f();
            }
        }
    }
}

// Non-DOS timer functions
#[cfg(not(all(target_os = "windows", target_pointer_width = "16")))]
pub mod timers {
    use super::*;

    #[inline]
    pub fn RADTimerSetup() {
        // No-op on non-DOS platforms
    }

    #[inline]
    pub fn RADTimerDone() {
        // No-op on non-DOS platforms
    }

    // RADTimerRead depends on platform:
    // On Win16/RADWINEXT: defined as timeGetTime
    // Otherwise: declared as extern function
    #[cfg(any(target_pointer_width = "16"))]
    extern "C" {
        pub fn timeGetTime() -> u32;
    }

    #[cfg(not(any(target_pointer_width = "16")))]
    extern "C" {
        pub fn RADTimerRead() -> u32;
    }
}

// Helper macros/consts for alignment operations
// #define u32neg1 ((u32)(s32)-1)
pub const U32NEG1: u32 = ((-1i32) as u32);

// #define RAD_align(var) var; u8 junk##var[4-(sizeof(var)&3)];
// These are handled at the struct/data definition site in Rust using
// #[repr(C, align(4))] or manual padding fields.

// Platform-specific keyboard input (Watcom)
#[cfg(all(target_os = "windows", target_pointer_width = "16"))]
pub mod kbd_watcom {
    use super::*;

    extern "C" {
        pub fn bkbhit() -> u8;
        pub fn bgetch() -> u8;
        pub fn radinp(p: u16) -> u8;
        pub fn radoutp(p: u16, v: u8);
        pub fn radtoupper(p: u8) -> u8;
    }
}

// Lock operations for multi-processor machines
#[cfg(target_os = "windows")]
pub mod locked_ops {
    use super::*;

    // for multi-processor machines
    // lock inc [var] (x86 specific)
    #[inline]
    pub fn LockedIncrement(_var: &mut i32) {
        // Platform-specific atomic increment; x86 asm: lock inc [var]
        // Would use atomic operations or asm on actual platform
    }

    // lock dec [var] (x86 specific)
    #[inline]
    pub fn LockedDecrement(_var: &mut i32) {
        // Platform-specific atomic decrement; x86 asm: lock dec [var]
        // Would use atomic operations or asm on actual platform
    }

    #[inline]
    pub fn LockedIncrementFunc(_var: *mut core::ffi::c_void) {
        // Platform-specific atomic increment of pointed value
        // x86 asm: mov eax,[var]; lock inc [eax]
    }

    #[inline]
    pub fn LockedDecrementFunc(_var: *mut core::ffi::c_void) {
        // Platform-specific atomic decrement of pointed value
        // x86 asm: mov eax,[var]; lock dec [eax]
    }
}


// Additional string operation declarations for Watcom 32-bit
#[cfg(all(target_os = "windows", target_pointer_width = "32", not(target_env = "msvc")))]
extern "C" {
    pub fn radstrcpy(dest: *mut core::ffi::c_char, source: *const core::ffi::c_char) -> *mut core::ffi::c_char;
    pub fn radstpcpy(dest: *mut core::ffi::c_char, source: *const core::ffi::c_char) -> *mut core::ffi::c_char;
    pub fn radstpcpyrs(dest: *mut core::ffi::c_char, source: *const core::ffi::c_char) -> *mut core::ffi::c_char;
    pub fn radmemset32(dest: *mut core::ffi::c_void, value: u32, size: u32);
    pub fn radstricmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char) -> i8;
    pub fn radstrnicmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char, len: u32) -> i8;
    pub fn radstrupr(s1: *mut core::ffi::c_char) -> *mut core::ffi::c_char;
    pub fn radstrlwr(s1: *mut core::ffi::c_char) -> *mut core::ffi::c_char;
}
