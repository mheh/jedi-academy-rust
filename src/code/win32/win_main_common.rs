// win_main_common.rs
// Translated from oracle/code/win32/win_main_common.cpp
// win_main.h
//
// #include "../game/q_shared.h"
// #include "../qcommon/qcommon.h"
// #include "../client/client.h"
// #include "win_local.h"
// #include "resource.h"
// #ifndef _GAMECUBE
// #include <errno.h>
// #include <float.h>
// #include <fcntl.h>
// #include <stdio.h>
// #include <direct.h>
// #include <io.h>
// #include <conio.h>
// #endif

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// External C functions
extern "C" {
    fn Com_Printf(fmt: *const c_char, ...);
    fn Z_Free(ptr: *mut c_void);
    fn Sys_Milliseconds() -> c_int;
    fn Sys_Cwd() -> *mut c_char;
    fn Com_BlockChecksum(ptr: *mut c_void, len: c_int) -> c_int;
    fn FS_Read(buffer: *mut c_void, size: c_int, f: c_int) -> c_int;
    fn FS_Seek(f: c_int, offset: c_int, origin: c_int);
}

// Stub type declarations for types defined in other headers
// These need to be imported from their actual modules
#[repr(C)]
pub struct sysEvent_t {
    pub evTime: c_int,
    pub evType: c_int, // sysEventType_t
    pub evValue: c_int,
    pub evValue2: c_int,
    pub evPtrLength: c_int,
    pub evPtr: *mut c_void,
}

// Constants
pub const MAX_QUED_EVENTS: usize = 256;
pub const MASK_QUED_EVENTS: usize = MAX_QUED_EVENTS - 1;
pub const MAX_MSGLEN: usize = 16384;

//#define SPANK_MONKEYS	//----(SA)	commented out for running net developer release builds
pub static mut sys_monkeySpank: c_int = 0;

/*
==================
Sys_MonkeyShouldBeSpanked
==================
*/
#[no_mangle]
pub extern "C" fn Sys_MonkeyShouldBeSpanked() -> c_int {
    unsafe { sys_monkeySpank }
}

/*
==================
Sys_FunctionCmp
==================
*/
#[no_mangle]
pub extern "C" fn Sys_FunctionCmp(f1: *mut c_void, f2: *mut c_void) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut l: c_int;
    let mut func_end: [u8; 32] = [0xC3, 0x90, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut ptr: *mut u8;
    let mut ptr2: *mut u8;
    let mut f1_ptr: *mut u8;
    let mut f2_ptr: *mut u8;

    unsafe {
        ptr = f1 as *mut u8;
        if *(ptr as *const u8) == 0xE9 {
            //Com_Printf("f1 %p1 jmp %d\n", (int *) f1, *(int*)(ptr+1));
            let offset = (*(ptr.add(1) as *const i32) + 5) as isize;
            f1_ptr = ((f1 as isize).wrapping_add(offset)) as *mut u8;
        } else {
            f1_ptr = ptr;
        }
        //Com_Printf("f1 ptr %p\n", f1_ptr);

        ptr = f2 as *mut u8;
        if *(ptr as *const u8) == 0xE9 {
            //Com_Printf("f2 %p jmp %d\n", (int *) f2, *(int*)(ptr+1));
            let offset = (*(ptr.add(1) as *const i32) + 5) as isize;
            f2_ptr = ((f2 as isize).wrapping_add(offset)) as *mut u8;
        } else {
            f2_ptr = ptr;
        }
        //Com_Printf("f2 ptr %p\n", f2_ptr);

        #[cfg(debug_assertions)]
        {
            // sprintf((char *)func_end, "%c%c%c%c%c%c%c", 0x5F, 0x5E, 0x5B, 0x8B, 0xE5, 0x5D, 0xC3);
            func_end[0] = 0x5F;
            func_end[1] = 0x5E;
            func_end[2] = 0x5B;
            func_end[3] = 0x8B;
            func_end[4] = 0xE5;
            func_end[5] = 0x5D;
            func_end[6] = 0xC3;
        }

        i = 0;
        while i < 1024 {
            j = 0;
            while j < 32 && func_end[j as usize] != 0 {
                if *f1_ptr.add((i + j) as usize) != func_end[j as usize] {
                    break;
                }
                j += 1;
            }
            if j < 32 && func_end[j as usize] == 0 {
                break;
            }
            i += 1;
        }

        #[cfg(debug_assertions)]
        {
            l = i + 7;
        }
        #[cfg(not(debug_assertions))]
        {
            l = i + 2;
        }
        //Com_Printf("function length = %d\n", l);

        i = 0;
        loop {
            if i >= l {
                break;
            }

            // check for a potential function call
            if *f1_ptr.add(i as usize) == 0xE8 {
                // get the function pointers in case this really is a function call
                let target_offset1 = (*(f1_ptr.add((i + 1) as usize) as *const i32) + 5) as isize;
                let target_offset2 = (*(f2_ptr.add((i + 1) as usize) as *const i32) + 5) as isize;
                ptr = ((f1_ptr.add(i as usize) as isize).wrapping_add(target_offset1)) as *mut u8;
                ptr2 = ((f2_ptr.add(i as usize) as isize).wrapping_add(target_offset2)) as *mut u8;
                // if it was a function call and both f1 and f2 call the same function
                if ptr == ptr2 {
                    i += 4;
                    continue;
                }
            }
            if *f1_ptr.add(i as usize) != *f2_ptr.add(i as usize) {
                return 0; // qfalse
            }
            i += 1;
        }
    }

    1 // qtrue
}

/*
==================
Sys_FunctionCheckSum
==================
*/
#[no_mangle]
pub extern "C" fn Sys_FunctionCheckSum(f1: *mut c_void) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut l: c_int;
    let mut shermcrap: c_int;
    let mut func_end: [u8; 32] = [0xC3, 0x90, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut ptr: *mut u8;
    let mut f1_ptr: *mut u8;

    unsafe {
        ptr = f1 as *mut u8;
        if *(ptr as *const u8) == 0xE9 {
            //Com_Printf("f1 %p1 jmp %d\n", (int *) f1, *(int*)(ptr+1));
            let offset = (*(ptr.add(1) as *const i32) + 5) as isize;
            f1_ptr = ((f1 as isize).wrapping_add(offset)) as *mut u8;
        } else {
            f1_ptr = ptr;
        }
        //Com_Printf("f1 ptr %p\n", f1_ptr);

        #[cfg(debug_assertions)]
        {
            // sprintf((char *)func_end, "%c%c%c%c%c%c%c", 0x5F, 0x5E, 0x5B, 0x8B, 0xE5, 0x5D, 0xC3);
            func_end[0] = 0x5F;
            func_end[1] = 0x5E;
            func_end[2] = 0x5B;
            func_end[3] = 0x8B;
            func_end[4] = 0xE5;
            func_end[5] = 0x5D;
            func_end[6] = 0xC3;
        }

        i = 0;
        while i < 1024 {
            j = 0;
            while j < 32 && func_end[j as usize] != 0 {
                if *f1_ptr.add((i + j) as usize) != func_end[j as usize] {
                    break;
                }
                j += 1;
            }
            if j < 32 && func_end[j as usize] == 0 {
                break;
            }
            i += 1;
        }

        #[cfg(debug_assertions)]
        {
            l = i + 7;
        }
        #[cfg(not(debug_assertions))]
        {
            l = i + 2;
        }
        //Com_Printf("function length = %d\n", l);

        shermcrap = Com_BlockChecksum(f1_ptr as *mut c_void, l);
    }

    shermcrap
}

//NOTE TTimo: heavily NON PORTABLE, PLZ DON'T USE
//  https://zerowing.idsoftware.com/bugzilla/show_bug.cgi?id=447
#[cfg(feature = "shellexecute")]
#[no_mangle]
pub extern "C" fn Sys_ShellExecute(op: *mut c_char, file: *mut c_char, doexit: c_int, params: *mut c_char, dir: *mut c_char) -> c_int {
    //----(SA) added
    /*
    ==============
    Sys_ShellExecute

    -	Windows only

    	Performs an operation on a specified file.

    	See info on ShellExecute() for details

    ==============
    */
    // This function is not currently available
    // See the original code in the oracle/code/win32/win_main_common.cpp for details
    0
}

/*
==================
Sys_BeginProfiling
==================
*/
#[no_mangle]
pub extern "C" fn Sys_BeginProfiling() {
    // this is just used on the mac build
}

/*
==============
Sys_DefaultCDPath
==============
*/
#[no_mangle]
pub extern "C" fn Sys_DefaultCDPath() -> *mut c_char {
    b"\0".as_ptr() as *mut c_char
}

/*
==============
Sys_DefaultBasePath
==============
*/
#[no_mangle]
pub extern "C" fn Sys_DefaultBasePath() -> *mut c_char {
    Sys_Cwd()
}

/*
========================================================================

EVENT LOOP

========================================================================
*/

pub static mut eventQue: [sysEvent_t; MAX_QUED_EVENTS] = [
    sysEvent_t {
        evTime: 0,
        evType: 0,
        evValue: 0,
        evValue2: 0,
        evPtrLength: 0,
        evPtr: core::ptr::null_mut(),
    };
    MAX_QUED_EVENTS
];
pub static mut eventHead: c_int = 0;
pub static mut eventTail: c_int = 0;
pub static mut sys_packetReceived: [u8; MAX_MSGLEN] = [0u8; MAX_MSGLEN];

/*
================
Sys_QueEvent

A time of 0 will get the current time
Ptr should either be null, or point to a block of data that can
be freed by the game later.
================
*/
#[no_mangle]
pub extern "C" fn Sys_QueEvent(mut time: c_int, r#type: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void) {
    let mut ev: *mut sysEvent_t;

    unsafe {
        ev = &mut eventQue[(eventHead as usize) & MASK_QUED_EVENTS];
        if eventHead - eventTail >= MAX_QUED_EVENTS as c_int {
            Com_Printf(b"Sys_QueEvent: overflow\n\0".as_ptr() as *const c_char);
            // we are discarding an event, but don't leak memory
            if !(*ev).evPtr.is_null() {
                Z_Free((*ev).evPtr);
            }
            eventTail += 1;
        }

        eventHead += 1;

        if time == 0 {
            time = Sys_Milliseconds();
        }

        (*ev).evTime = time;
        (*ev).evType = r#type;
        (*ev).evValue = value;
        (*ev).evValue2 = value2;
        (*ev).evPtrLength = ptrLength;
        (*ev).evPtr = ptr;
    }
}

//================================================================

/*
=================
Sys_Net_Restart_f

Restart the network subsystem
=================
*/
#[no_mangle]
pub extern "C" fn Sys_Net_Restart_f() {
    //	NET_Restart();
}

//=======================================================================

#[no_mangle]
pub extern "C" fn Sys_InitStreamThread() {}

#[no_mangle]
pub extern "C" fn Sys_ShutdownStreamThread() {}

#[no_mangle]
pub extern "C" fn Sys_BeginStreamedFile(_f: c_int, _readAhead: c_int) {}

#[no_mangle]
pub extern "C" fn Sys_EndStreamedFile(_f: c_int) {}

#[no_mangle]
pub extern "C" fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: c_int) -> c_int {
    unsafe { FS_Read(buffer, size * count, f) }
}

#[no_mangle]
pub extern "C" fn Sys_StreamSeek(f: c_int, offset: c_int, origin: c_int) {
    unsafe {
        FS_Seek(f, offset, origin);
    }
}

#[no_mangle]
pub extern "C" fn Sys_InitializeCriticalSection() -> *mut c_void {
    (-1_isize) as *mut c_void
}

#[no_mangle]
pub extern "C" fn Sys_EnterCriticalSection(_ptr: *mut c_void) {}

#[no_mangle]
pub extern "C" fn Sys_LeaveCriticalSection(_ptr: *mut c_void) {}
