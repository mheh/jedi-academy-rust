//! `g_timer.c` — string-keyed per-entity timers (the SP "timer map" rewritten by
//! rww into a static free-list pool so the SP AI/saber code ports cleanly).
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_timer.c`. The C file's own header:
//!
//! > //rww - rewrite from C++ SP version.
//! > //This is here only to make porting from SP easier, it's really sort of nasty (being static
//! > //now). Basically it's slower and takes more memory.
//!
//! Storage mirrors the three C globals exactly: a fixed pool of `gtimer_t`
//! (`g_timerPool`), a per-entity head array (`g_timers`) indexed by `ent->s.number`,
//! and the singly-linked free list (`g_timerFreeList`). They live in zeroed BSS in C;
//! here they are zero-initialised mutable statics, taken only via
//! `addr_of_mut!`/`addr_of!` (never `&`/`&mut` to a `static mut`).

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)] // C global names (`g_timerPool`, ...) kept verbatim
#![allow(non_camel_case_types)] // C type names (`gtimer_t`) kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::level;
use crate::codemp::game::q_shared::Q_stricmp;
use crate::codemp::game::q_shared_h::MAX_GENTITIES;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};

/// `#define MAX_GTIMERS 16384` (g_timer.c:8).
pub const MAX_GTIMERS: usize = 16384;

/// `typedef struct gtimer_s { const char *name; int time; struct gtimer_s *next; } gtimer_t;`
/// (g_timer.c:10). Field order matches the C struct exactly.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gtimer_t {
    pub name: *const c_char,
    pub time: c_int,
    pub next: *mut gtimer_t, // In either free list or current list
}

/// `gtimer_t g_timerPool[ MAX_GTIMERS ];` (g_timer.c:17). Zeroed BSS in C.
static mut g_timerPool: [gtimer_t; MAX_GTIMERS] = [gtimer_t {
    name: null_mut(),
    time: 0,
    next: null_mut(),
}; MAX_GTIMERS];

/// `gtimer_t *g_timers[ MAX_GENTITIES ];` (g_timer.c:18). Per-entity list heads.
static mut g_timers: [*mut gtimer_t; MAX_GENTITIES] = [null_mut(); MAX_GENTITIES];

/// `gtimer_t *g_timerFreeList;` (g_timer.c:19).
static mut g_timerFreeList: *mut gtimer_t = null_mut();

/*
-------------------------
TIMER_Clear
-------------------------
*/
pub unsafe fn TIMER_Clear() {
    let mut i: usize;
    i = 0;
    while i < MAX_GENTITIES {
        g_timers[i] = null_mut();
        i += 1;
    }

    let pool = addr_of_mut!(g_timerPool) as *mut gtimer_t;
    i = 0;
    while i < MAX_GTIMERS - 1 {
        (*pool.add(i)).next = pool.add(i + 1);
        i += 1;
    }
    (*pool.add(MAX_GTIMERS - 1)).next = null_mut();
    g_timerFreeList = pool;
}

/*
-------------------------
TIMER_Clear
-------------------------
*/
pub unsafe fn TIMER_Clear2(ent: *mut gentity_t) {
    // rudimentary safety checks, might be other things to check?
    if !ent.is_null() && (*ent).s.number > 0 && ((*ent).s.number as usize) < MAX_GENTITIES {
        let num = (*ent).s.number as usize;
        let mut p: *mut gtimer_t = g_timers[num];

        // No timers at all -> do nothing
        if p.is_null() {
            return;
        }

        // Find the end of this ents timer list
        while !(*p).next.is_null() {
            p = (*p).next;
        }

        // Splice the lists
        (*p).next = g_timerFreeList;
        g_timerFreeList = g_timers[num];
        g_timers[num] = null_mut();
    }
}

//New C "lookup" func.
//Returns existing timer in array if
pub unsafe fn TIMER_GetNew(num: c_int, identifier: *const c_char) -> *mut gtimer_t {
    let num = num as usize;
    let mut p: *mut gtimer_t = g_timers[num];

    // Search for an existing timer with this name
    while !p.is_null() {
        if Q_stricmp((*p).name, identifier) == 0 {
            // Found it
            return p;
        }

        p = (*p).next;
    }

    // No existing timer with this name was found, so grab one from the free list
    if g_timerFreeList.is_null() {
        return null_mut();
    }

    p = g_timerFreeList;
    g_timerFreeList = (*g_timerFreeList).next;
    (*p).next = g_timers[num];
    g_timers[num] = p;
    p
}

//don't return the first free if it doesn't already exist, return null.
pub unsafe fn TIMER_GetExisting(num: c_int, identifier: *const c_char) -> *mut gtimer_t {
    let num = num as usize;
    let mut p: *mut gtimer_t = g_timers[num];

    while !p.is_null() {
        if Q_stricmp((*p).name, identifier) == 0 {
            // Found it
            return p;
        }

        p = (*p).next;
    }

    null_mut()
}

/*
-------------------------
TIMER_Set
-------------------------
*/
pub unsafe fn TIMER_Set(ent: *mut gentity_t, identifier: *const c_char, duration: c_int) {
    let timer: *mut gtimer_t = TIMER_GetNew((*ent).s.number, identifier);

    if timer.is_null() {
        return;
    }
    (*timer).name = identifier;
    (*timer).time = (*addr_of!(level)).time + duration;
}

/*
-------------------------
TIMER_Get
-------------------------
*/
pub unsafe fn TIMER_Get(ent: *mut gentity_t, identifier: *const c_char) -> c_int {
    let timer: *mut gtimer_t = TIMER_GetExisting((*ent).s.number, identifier);

    if timer.is_null() {
        return -1;
    }

    (*timer).time
}

/*
-------------------------
TIMER_Done
-------------------------
*/
pub unsafe fn TIMER_Done(ent: *mut gentity_t, identifier: *const c_char) -> qboolean {
    let timer: *mut gtimer_t = TIMER_GetExisting((*ent).s.number, identifier);

    if timer.is_null() {
        return QTRUE;
    }

    ((*timer).time < (*addr_of!(level)).time) as qboolean
}

/*
-------------------------
TIMER_RemoveHelper

Scans an entities timer list to remove a given
timer from the list and put it on the free list

Doesn't do much error checking, only called below
-------------------------
*/
pub unsafe fn TIMER_RemoveHelper(num: c_int, timer: *mut gtimer_t) {
    let num = num as usize;
    let mut p: *mut gtimer_t = g_timers[num];

    // Special case: first timer in list
    if p == timer {
        g_timers[num] = (*g_timers[num]).next;
        (*p).next = g_timerFreeList;
        g_timerFreeList = p;
        return;
    }

    // Find the predecessor
    while (*p).next != timer {
        p = (*p).next;
    }

    // Rewire
    (*p).next = (*(*p).next).next;
    (*timer).next = g_timerFreeList;
    g_timerFreeList = timer;
}

/*
-------------------------
TIMER_Done2

Returns false if timer has been
started but is not done...or if
timer was never started
-------------------------
*/
pub unsafe fn TIMER_Done2(
    ent: *mut gentity_t,
    identifier: *const c_char,
    remove: qboolean,
) -> qboolean {
    let timer: *mut gtimer_t = TIMER_GetExisting((*ent).s.number, identifier);
    let res: qboolean;

    if timer.is_null() {
        return QFALSE;
    }

    res = ((*timer).time < (*addr_of!(level)).time) as qboolean;

    if res != QFALSE && remove != QFALSE {
        // Put it back on the free list
        TIMER_RemoveHelper((*ent).s.number, timer);
    }

    res
}

/*
-------------------------
TIMER_Exists
-------------------------
*/
pub unsafe fn TIMER_Exists(ent: *mut gentity_t, identifier: *const c_char) -> qboolean {
    let timer: *mut gtimer_t = TIMER_GetExisting((*ent).s.number, identifier);

    if timer.is_null() {
        return QFALSE;
    }

    QTRUE
}

/*
-------------------------
TIMER_Remove
Utility to get rid of any timer
-------------------------
*/
pub unsafe fn TIMER_Remove(ent: *mut gentity_t, identifier: *const c_char) {
    let timer: *mut gtimer_t = TIMER_GetExisting((*ent).s.number, identifier);

    if timer.is_null() {
        return;
    }

    // Put it back on the free list
    TIMER_RemoveHelper((*ent).s.number, timer);
}

/*
-------------------------
TIMER_Start
-------------------------
*/
pub unsafe fn TIMER_Start(
    self_: *mut gentity_t,
    identifier: *const c_char,
    duration: c_int,
) -> qboolean {
    if TIMER_Done(self_, identifier) != QFALSE {
        TIMER_Set(self_, identifier, duration);
        return QTRUE;
    }
    QFALSE
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::codemp::game::g_main::{level, level_lock};
    use crate::oracle;
    use core::ptr::addr_of_mut;

    // A zeroed gentity_t whose only field the timer code touches is `s.number`. The full
    // struct is large but `MaybeUninit::zeroed` is valid for it (all-bytes-zero is a legal
    // gentity_t per its C BSS lifetime), and we only ever read `s.number`.
    unsafe fn ent_with_number(num: c_int) -> Box<gentity_t> {
        let mut e: Box<gentity_t> = Box::new(core::mem::MaybeUninit::zeroed().assume_init());
        e.s.number = num;
        e
    }

    /// `TIMER_Set` then `TIMER_Done` must agree with the C oracle bit-for-bit across a range
    /// of clocks/durations/keys — the timer is "done" iff its stored `level.time + duration`
    /// is strictly less than the current `level.time`, and a never-set key reports done.
    #[test]
    fn TIMER_Set_and_Done_match_oracle() {
        let _g = level_lock();
        unsafe {
            // (set_time, num, key, duration, query_time) scenarios spanning expired/pending/
            // exact-boundary, distinct keys colliding on the same entity, and a never-set key.
            let cases: &[(c_int, c_int, &core::ffi::CStr, c_int, c_int)] = &[
                (1000, 7, c"saberBlock", 500, 1400), // pending (1500 not < 1400)
                (1000, 7, c"saberBlock", 500, 1600), // expired (1500 < 1600)
                (1000, 7, c"saberBlock", 500, 1500), // exact boundary (1500 < 1500 false)
                (0, 3, c"attackDelay", 0, 0),        // zero duration at t=0
                (0, 3, c"attackDelay", 0, 1),        // expired by 1ms
                (10000, 42, c"DASH", 250, 9000),     // query before set -> pending
                (-500, 5, c"mixedCase", 100, 0),     // negative base time
            ];

            for &(set_time, num, key, duration, query_time) in cases {
                // ---- oracle side ----
                oracle::jka_TIMER_oracle_reset(set_time);
                oracle::jka_TIMER_Set_idx(num, key.as_ptr(), duration);
                oracle::jka_TIMER_oracle_set_time(query_time);
                let c_done = oracle::jka_TIMER_Done_idx(num, key.as_ptr());

                // ---- rust side ----
                TIMER_Clear();
                (*addr_of_mut!(level)).time = set_time;
                let mut ent = ent_with_number(num);
                TIMER_Set(&mut *ent, key.as_ptr(), duration);
                (*addr_of_mut!(level)).time = query_time;
                let r_done = TIMER_Done(&mut *ent, key.as_ptr());

                assert_eq!(
                    r_done, c_done,
                    "TIMER_Done mismatch: set_time={set_time} num={num} key={key:?} \
                     duration={duration} query_time={query_time}"
                );

                // Also assert the never-set key reports the C oracle's value (qtrue).
                let other = c"neverSet";
                oracle::jka_TIMER_oracle_set_time(query_time);
                let c_missing = oracle::jka_TIMER_Done_idx(num, other.as_ptr());
                let r_missing = TIMER_Done(&mut *ent, other.as_ptr());
                assert_eq!(
                    r_missing, c_missing,
                    "TIMER_Done(never-set) mismatch for num={num} query_time={query_time}"
                );
            }
        }
    }
}
