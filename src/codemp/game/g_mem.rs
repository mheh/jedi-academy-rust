//! `g_mem.c` — the game module's fixed-pool zone allocator.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_mem.c`. A simple linear (bump)
//! allocator over one fixed `static` pool: `G_Alloc` hands out 32-byte-aligned
//! slices and never frees individually; `G_InitMemory` resets the pool at each map
//! load. (OpenJK enlarges `POOLSIZE` to 4 MB and adds a zero-size guard; the ABI
//! target is original JKA, whose PC source uses a 256 KB pool, so that size and the
//! JKA control flow are kept verbatim.)
//!
//! No C oracle: like the trap layer / `G_InitWorldSession`, the behaviour is
//! engine-I/O (`G_Printf`/`G_Error`) plus mutation of a file-private static pool,
//! not a computable data table. The only arithmetic — the `(size + 31) & ~31`
//! 32-byte round-up and the bounds check — is a verbatim transcription.

#![allow(non_snake_case)] // C function names (`G_Alloc`, ...) kept verbatim
#![allow(non_upper_case_globals)] // C static names (`memoryPool`, `allocPoint`) kept

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::g_main::{g_debugAlloc, G_Error, G_Printf};

const POOLSIZE: c_int = 256 * 1024;

static mut memoryPool: [c_char; POOLSIZE as usize] = [0; POOLSIZE as usize];
static mut allocPoint: c_int = 0;

pub fn G_Alloc(size: c_int) -> *mut c_void {
    // SAFETY: the module is single-threaded; `memoryPool`/`allocPoint` are this
    // file's own statics, taken by raw pointer (never `&mut`) to stay sound.
    unsafe {
        if (*addr_of!(g_debugAlloc)).integer != 0 {
            G_Printf(&format!(
                "G_Alloc of {} bytes ({} left)\n",
                size,
                POOLSIZE - *addr_of!(allocPoint) - ((size + 31) & !31)
            ));
        }

        if *addr_of!(allocPoint) + size > POOLSIZE {
            // bk010103 - was %u, but is signed
            G_Error(&format!(
                "G_Alloc: failed on allocation of {} bytes\n",
                size
            ));
            // return NULL;  -- unreachable: G_Error (-> !) does not return.
        }

        let p = (addr_of_mut!(memoryPool) as *mut c_char).add(*addr_of!(allocPoint) as usize)
            as *mut c_void;

        *addr_of_mut!(allocPoint) += (size + 31) & !31;

        p
    }
}

pub fn G_InitMemory() {
    // SAFETY: single-threaded module init.
    unsafe {
        *addr_of_mut!(allocPoint) = 0;
    }
}

/// Test-only: serializes every oracle test that drives the single file-private `G_Alloc`
/// pool (currently `g_spawn::G_NewString` and `bg_misc::BG_ParseField`'s `F_LSTRING`
/// path). `cargo test` runs tests on parallel threads, so they share this one lock and
/// reset the pool with [`G_InitMemory`] after acquiring it — otherwise one test could
/// reset the bump pointer out from under another's live allocations.
#[cfg(all(test, feature = "oracle"))]
pub static POOL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn Svcmd_GameMem_f() {
    // SAFETY: single-threaded module; reads `allocPoint`.
    unsafe {
        G_Printf(&format!(
            "Game memory status: {} out of {} bytes allocated\n",
            *addr_of!(allocPoint),
            POOLSIZE
        ));
    }
}
