// leave this line at the top for all g_xxxx.cpp files...
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use crate::code::game::g_local_h::{gentity_t, MAX_GENTITIES};
use crate::rufl::hstring::hstring;

// External engine interface and level globals
// These are defined elsewhere in the codebase
extern "C" {
    pub static mut level: crate::code::game::g_local_h::level_locals_t;
    pub static ENTITYNUM_MAX_NORMAL: c_int;

    // Engine interface functions for save/load
    pub fn gi_AppendToSaveGame(tag: c_int, data: *const c_void, size: c_int);
    pub fn gi_ReadFromSaveGame(tag: c_int, data: *mut c_void, size: c_int);
}

// Local wrappers for the engine interface (matching the gi. prefix pattern)
mod gi {
    use super::*;

    #[inline]
    pub unsafe fn AppendToSaveGame(tag: c_int, data: *const c_void, size: c_int) {
        gi_AppendToSaveGame(tag, data, size);
    }

    #[inline]
    pub unsafe fn ReadFromSaveGame(tag: c_int, data: *mut c_void, size: c_int) {
        gi_ReadFromSaveGame(tag, data, size);
    }
}

const MAX_GTIMERS: usize = 16384;

#[repr(C)]
struct gtimer_s {
    id: hstring,                    // Use handle strings, so that things work after loading
    time: c_int,
    next: *mut gtimer_s,            // In either free list or current list
}

type gtimer_t = gtimer_s;

static mut g_timerPool: [gtimer_s; MAX_GTIMERS] = [const { gtimer_s {
    id: unsafe { core::mem::zeroed() },
    time: 0,
    next: core::ptr::null_mut(),
}}; MAX_GTIMERS];

static mut g_timers: [*mut gtimer_t; MAX_GENTITIES as usize] = [core::ptr::null_mut(); MAX_GENTITIES as usize];
static mut g_timerFreeList: *mut gtimer_t = core::ptr::null_mut();


static unsafe fn TIMER_GetCount(num: c_int) -> c_int
{
    let mut p = g_timers[num as usize];
    let mut count = 0;

    while !p.is_null() {
        count += 1;
        p = (*p).next;
    }

    count
}


/*
-------------------------
TIMER_RemoveHelper

Scans an entities timer list to remove a given
timer from the list and put it on the free list

Doesn't do much error checking, only called below
-------------------------
*/
unsafe fn TIMER_RemoveHelper(num: c_int, timer: *mut gtimer_t)
{
    let mut p = g_timers[num as usize];

    // Special case: first timer in list
    if p == timer {
        g_timers[num as usize] = (*g_timers[num as usize]).next;
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
TIMER_Clear
-------------------------
*/

pub unsafe fn TIMER_Clear()
{
    for i in 0..MAX_GENTITIES as usize {
        g_timers[i] = core::ptr::null_mut();
    }

    for i in 0..(MAX_GTIMERS - 1) {
        g_timerPool[i].next = &mut g_timerPool[i+1];
    }
    g_timerPool[MAX_GTIMERS-1].next = core::ptr::null_mut();
    g_timerFreeList = &mut g_timerPool[0];
}

/*
-------------------------
TIMER_Clear
-------------------------
*/

pub unsafe fn TIMER_Clear_idx(idx: c_int)
{
    // rudimentary safety checks, might be other things to check?
    if idx >= 0 && idx < MAX_GENTITIES {
        let mut p = g_timers[idx as usize];

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
        g_timerFreeList = g_timers[idx as usize];
        g_timers[idx as usize] = core::ptr::null_mut();
    }
}


/*
-------------------------
TIMER_Save
-------------------------
*/

pub unsafe fn TIMER_Save()
{
    let mut ent = &mut g_entities[0];

    for j in 0..MAX_GENTITIES {
        let numTimers = TIMER_GetCount(j as c_int) as u8;

        if !(*ent).inuse && numTimers != 0 {
//          Com_Printf( "WARNING: ent with timers not inuse\n" );
            debug_assert!(numTimers != 0);
            TIMER_Clear_idx(j as c_int);
        }

        //Write out the timer information
        gi.AppendToSaveGame('TIME' as c_int, &numTimers as *const _ as *const c_void, core::mem::size_of_val(&numTimers) as c_int);

        let mut p = g_timers[j as usize];
        debug_assert!(
            (numTimers != 0 && !p.is_null()) || (numTimers == 0 && p.is_null())
        );

        while !p.is_null() {
            let timerID = (*p).id.c_str();
            let length = crate::code::game::q_shared::strlen(timerID) + 1;
            let time = (*p).time - level.time;	//convert this back to delta so we can use SET after loading

            debug_assert!(length < 1024);//This will cause problems when loading the timer if longer

            //Write out the id string
            gi.AppendToSaveGame('TMID' as c_int, timerID as *const c_void, length as c_int);

            //Write out the timer data
            gi.AppendToSaveGame('TDTA' as c_int, &time as *const _ as *const c_void, core::mem::size_of_val(&time) as c_int);
            p = (*p).next;
        }

        ent = ent.add(1);
    }
}

/*
-------------------------
TIMER_Load
-------------------------
*/

pub unsafe fn TIMER_Load()
{
    let mut ent = &mut g_entities[0];

    for j in 0..MAX_GENTITIES {
        let mut numTimers: u8 = 0;

        gi.ReadFromSaveGame('TIME' as c_int, &mut numTimers as *mut _ as *mut c_void, core::mem::size_of_val(&numTimers) as c_int);

        //Read back all entries
        for i in 0..(numTimers as c_int) {
            let mut time: c_int = 0;
            let mut tempBuffer: [c_char; 1024] = [0; 1024];  // Still ugly. Setting ourselves up for 007 AUF all over again. =)

            debug_assert!(core::mem::size_of::<c_int>() == core::mem::size_of_val(&time)); //make sure we're reading the same size as we wrote

            //Read the id string and time
            gi.ReadFromSaveGame('TMID' as c_int, &mut tempBuffer[0] as *mut _ as *mut c_void, 0);
            gi.ReadFromSaveGame('TDTA' as c_int, &mut time as *mut _ as *mut c_void, core::mem::size_of_val(&time) as c_int);

            //this is odd, we saved all the timers in the autosave, but not all the ents are spawned yet from an auto load, so skip it
            if (*ent).inuse {
                //Restore it
                TIMER_Set(ent, &tempBuffer[0] as *const _ as *const c_char, time);
            }
        }

        ent = ent.add(1);
    }
}


unsafe fn TIMER_GetNew(num: c_int, identifier: *const c_char) -> *mut gtimer_t
{
    debug_assert!(num < ENTITYNUM_MAX_NORMAL); //don't want timers on NONE or the WORLD
    let mut p = g_timers[num as usize];

    // Search for an existing timer with this name
    while !p.is_null() {
        if (*p).id == identifier {
            // Found it
            return p;
        }

        p = (*p).next;
    }

    // No existing timer with this name was found, so grab one from the free list
    if g_timerFreeList.is_null() {
        //oh no, none free!
        debug_assert!(!g_timerFreeList.is_null());
        return core::ptr::null_mut();
    }

    p = g_timerFreeList;
    g_timerFreeList = (*g_timerFreeList).next;
    (*p).next = g_timers[num as usize];
    g_timers[num as usize] = p;
    p
}


pub unsafe fn TIMER_GetExisting(num: c_int, identifier: *const c_char) -> *mut gtimer_t
{
    let mut p = g_timers[num as usize];

    while !p.is_null() {
        if (*p).id == identifier {
            // Found it
            return p;
        }

        p = (*p).next;
    }

    core::ptr::null_mut()
}



/*
-------------------------
TIMER_Set
-------------------------
*/

pub unsafe fn TIMER_Set(ent: *mut gentity_t, identifier: *const c_char, duration: c_int)
{
    debug_assert!((*ent).inuse);
    let timer = TIMER_GetNew((*ent).s.number, identifier);

    if !timer.is_null() {
        (*timer).id = identifier;
        (*timer).time = level.time + duration;
    }
}

/*
-------------------------
TIMER_Get
-------------------------
*/

pub unsafe fn TIMER_Get(ent: *mut gentity_t, identifier: *const c_char) -> c_int
{
    let timer = TIMER_GetExisting((*ent).s.number, identifier);

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

pub unsafe fn TIMER_Done(ent: *mut gentity_t, identifier: *const c_char) -> bool
{
    let timer = TIMER_GetExisting((*ent).s.number, identifier);

    if timer.is_null() {
        return true;
    }

    (*timer).time < level.time
}

/*
-------------------------
TIMER_Done2

Returns false if timer has been
started but is not done...or if
timer was never started
-------------------------
*/

pub unsafe fn TIMER_Done2(ent: *mut gentity_t, identifier: *const c_char, remove: bool) -> bool
{
    let timer = TIMER_GetExisting((*ent).s.number, identifier);

    if timer.is_null() {
        return false;
    }

    let res = (*timer).time < level.time;

    if res && remove {
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
pub unsafe fn TIMER_Exists(ent: *mut gentity_t, identifier: *const c_char) -> bool
{
    !TIMER_GetExisting((*ent).s.number, identifier).is_null()
}



/*
-------------------------
TIMER_Remove
Utility to get rid of any timer
-------------------------
*/
pub unsafe fn TIMER_Remove(ent: *mut gentity_t, identifier: *const c_char)
{
    let timer = TIMER_GetExisting((*ent).s.number, identifier);

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

pub unsafe fn TIMER_Start(self_: *mut gentity_t, identifier: *const c_char, duration: c_int) -> bool
{
    if TIMER_Done(self_, identifier) {
        TIMER_Set(self_, identifier, duration);
        return true;
    }
    false
}
