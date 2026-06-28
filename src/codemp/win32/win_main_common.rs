// win_main_common.cpp

use core::ffi::{c_int, c_char, c_void};

// External functions from qcommon and filesystem modules
extern "C" {
    fn Com_Printf(fmt: *const c_char, ...) -> ();
    fn Sys_Milliseconds() -> c_int;
    fn Z_Free(ptr: *mut c_void) -> ();
    fn Sys_Cwd() -> *const c_char;
    fn FS_Read(buffer: *mut c_void, size: c_int, f: i32) -> c_int;
    fn FS_Seek(f: i32, offset: c_int, origin: c_int) -> ();
    fn Com_BlockChecksum(ptr: *const u8, len: c_int) -> c_int;
}

// External constants (assumed from oracle dependencies)
// MAX_QUED_EVENTS and MASK_QUED_EVENTS would come from qcommon.h
// MAX_MSGLEN would come from qcommon.h
// We declare them as unresolved stubs; actual values defined in qcommon module

// External types from qcommon
#[repr(C)]
pub struct sysEvent_t {
    pub evTime: c_int,
    pub evType: c_int,      // sysEventType_t (int enum)
    pub evValue: c_int,
    pub evValue2: c_int,
    pub evPtrLength: c_int,
    pub evPtr: *mut c_void,
}

//#define SPANK_MONKEYS	//----(SA)	commented out for running net developer release builds
static mut sys_monkeySpank: c_int = 0;

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
    let func_end: [u8; 32] = [0xC3, 0x90, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let ptr: *mut u8;
    let ptr2: *mut u8;
    let f1_ptr: *mut u8;
    let f2_ptr: *mut u8;

    ptr = f1 as *mut u8;
    if unsafe { *(ptr as *const u8) } == 0xE9 {
        //Com_Printf("f1 %p1 jmp %d\n", (int *) f1, *(int*)(ptr+1));
        f1_ptr = unsafe {
            (f1 as *mut u8)
                .add(*(((f1 as *mut u8).add(1)) as *const c_int) as usize)
                .add(5)
        };
    } else {
        f1_ptr = ptr;
    }
    //Com_Printf("f1 ptr %p\n", f1_ptr);

    ptr = f2 as *mut u8;
    if unsafe { *(ptr as *const u8) } == 0xE9 {
        //Com_Printf("f2 %p jmp %d\n", (int *) f2, *(int*)(ptr+1));
        f2_ptr = unsafe {
            (f2 as *mut u8)
                .add(*(((f2 as *mut u8).add(1)) as *const c_int) as usize)
                .add(5)
        };
    } else {
        f2_ptr = ptr;
    }
    //Com_Printf("f2 ptr %p\n", f2_ptr);

    // func_end is mutable in _DEBUG but we preserve the structure as-is
    let mut func_end_mutable = func_end;
    #[cfg(debug_assertions)]
    {
        // In _DEBUG builds, this would use sprintf to write specific bytes
        // Simulating sprintf write of: "%c%c%c%c%c%c%c", 0x5F, 0x5E, 0x5B, 0x8B, 0xE5, 0x5D, 0xC3
        func_end_mutable[0] = 0x5F;
        func_end_mutable[1] = 0x5E;
        func_end_mutable[2] = 0x5B;
        func_end_mutable[3] = 0x8B;
        func_end_mutable[4] = 0xE5;
        func_end_mutable[5] = 0x5D;
        func_end_mutable[6] = 0xC3;
    }

    i = 0;
    while i < 1024 {
        j = 0;
        while func_end_mutable[j as usize] != 0 {
            if unsafe { f1_ptr.add(i as usize).read() } != func_end_mutable[j as usize] {
                break;
            }
            j += 1;
        }
        if func_end_mutable[j as usize] == 0 {
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
    while i < l {
        // check for a potential function call
        if unsafe { *((f1_ptr.add(i as usize)) as *const u8) } == 0xE8 {
            // get the function pointers in case this really is a function call
            ptr = unsafe {
                ((f1_ptr.add(i as usize)) as *const c_int)
                    .add(1)
                    .read() as usize as *mut u8
            };
            ptr = unsafe { ptr.add((f1_ptr.add(i as usize) as *const c_int).add(1).read() as usize).add(5) };
            ptr2 = unsafe {
                ((f2_ptr.add(i as usize)) as *const c_int)
                    .add(1)
                    .read() as usize as *mut u8
            };
            ptr2 = unsafe { ptr2.add((f2_ptr.add(i as usize) as *const c_int).add(1).read() as usize).add(5) };
            // if it was a function call and both f1 and f2 call the same function
            if ptr == ptr2 {
                i += 4;
                continue;
            }
        }
        if unsafe { f1_ptr.add(i as usize).read() } != unsafe { f2_ptr.add(i as usize).read() } {
            return 0; // qfalse
        }
        i += 1;
    }
    return 1; // qtrue
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
    let func_end: [u8; 32] = [0xC3, 0x90, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let ptr: *mut u8;
    let f1_ptr: *mut u8;

    ptr = f1 as *mut u8;
    if unsafe { *(ptr as *const u8) } == 0xE9 {
        //Com_Printf("f1 %p1 jmp %d\n", (int *) f1, *(int*)(ptr+1));
        f1_ptr = unsafe {
            (f1 as *mut u8)
                .add(*(((f1 as *mut u8).add(1)) as *const c_int) as usize)
                .add(5)
        };
    } else {
        f1_ptr = ptr;
    }
    //Com_Printf("f1 ptr %p\n", f1_ptr);

    let mut func_end_mutable = func_end;
    #[cfg(debug_assertions)]
    {
        // In _DEBUG builds, this would use sprintf to write specific bytes
        // Simulating sprintf write of: "%c%c%c%c%c%c%c", 0x5F, 0x5E, 0x5B, 0x8B, 0xE5, 0x5D, 0xC3
        func_end_mutable[0] = 0x5F;
        func_end_mutable[1] = 0x5E;
        func_end_mutable[2] = 0x5B;
        func_end_mutable[3] = 0x8B;
        func_end_mutable[4] = 0xE5;
        func_end_mutable[5] = 0x5D;
        func_end_mutable[6] = 0xC3;
    }

    i = 0;
    while i < 1024 {
        j = 0;
        while func_end_mutable[j as usize] != 0 {
            if unsafe { f1_ptr.add(i as usize).read() } != func_end_mutable[j as usize] {
                break;
            }
            j += 1;
        }
        if func_end_mutable[j as usize] == 0 {
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
    shermcrap = unsafe { Com_BlockChecksum(f1_ptr, l) };
    return shermcrap;
}

//NOTE TTimo: heavily NON PORTABLE, PLZ DON'T USE
//  https://zerowing.idsoftware.com/bugzilla/show_bug.cgi?id=447
// Commented out in original: #if 0
//----(SA) added
/*
==============
Sys_ShellExecute

-	Windows only

	Performs an operation on a specified file.

	See info on ShellExecute() for details

==============
*/
// int Sys_ShellExecute(char *op, char *file, qboolean doexit, char *params, char *dir ) {
// 	unsigned int retval;
// 	char *se_op;
//
// 	// set default operation to "open"
// 	if(op)	se_op = op;
// 	else	se_op = "open";
//
//
// 	// probably need to protect this some in the future so people have
// 	// less chance of system invasion with this powerful interface
// 	// (okay, not so invasive, but could be annoying/rude)
//
//
// 	retval = (UINT)ShellExecute(NULL, se_op, file, params, dir, SW_NORMAL);	// only option forced by game is 'sw_normal'
//
// 	if( retval <= 32) {	// ERROR
// 		Com_DPrintf("Sys_ShellExecuteERROR: %d\n", retval);
// 		return retval;
// 	}
//
// 	if ( doexit ) {
// 		// (SA) this works better for exiting cleanly...
// 		Cbuf_ExecuteText( EXEC_APPEND, "quit" );
// 	}
//
// 	return 999;	// success
// }
//----(SA) end

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
pub extern "C" fn Sys_DefaultCDPath() -> *const c_char {
    b"\0".as_ptr() as *const c_char
}

/*
==============
Sys_DefaultBasePath
==============
*/
#[no_mangle]
pub extern "C" fn Sys_DefaultBasePath() -> *const c_char {
    unsafe { Sys_Cwd() }
}

/*
========================================================================

EVENT LOOP

========================================================================
*/

// These constants are assumed to come from qcommon.h
// MAX_QUED_EVENTS and MASK_QUED_EVENTS are used but not defined here
// Assuming they are defined in the qcommon module or via #[link] attributes
// For now we'll use reasonable defaults that should match the C header
const MAX_QUED_EVENTS: usize = 256;
const MASK_QUED_EVENTS: c_int = 255;
const MAX_MSGLEN: usize = 16384;

static mut eventQue: [sysEvent_t; MAX_QUED_EVENTS] = [sysEvent_t {
    evTime: 0,
    evType: 0,
    evValue: 0,
    evValue2: 0,
    evPtrLength: 0,
    evPtr: core::ptr::null_mut(),
}; MAX_QUED_EVENTS];
static mut eventHead: c_int = 0;
static mut eventTail: c_int = 0;
static mut sys_packetReceived: [u8; MAX_MSGLEN] = [0; MAX_MSGLEN];

/*
================
Sys_QueEvent

A time of 0 will get the current time
Ptr should either be null, or point to a block of data that can
be freed by the game later.
================
*/
#[no_mangle]
pub extern "C" fn Sys_QueEvent(
    time: c_int,
    type_: c_int,
    value: c_int,
    value2: c_int,
    ptrLength: c_int,
    ptr: *mut c_void,
) {
    let mut ev: *mut sysEvent_t;
    let mut time_mut = time;

    unsafe {
        ev = &mut eventQue[(eventHead & MASK_QUED_EVENTS) as usize];
        if eventHead - eventTail >= MAX_QUED_EVENTS as c_int {
            Com_Printf(b"Sys_QueEvent: overflow\n\0".as_ptr() as *const c_char);
            // we are discarding an event, but don't leak memory
            if !(*ev).evPtr.is_null() {
                Z_Free((*ev).evPtr);
            }
            eventTail += 1;
        }

        eventHead += 1;

        if time_mut == 0 {
            time_mut = Sys_Milliseconds();
        }

        (*ev).evTime = time_mut;
        (*ev).evType = type_;
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
pub extern "C" fn Sys_BeginStreamedFile(f: i32, readAhead: c_int) {}

#[no_mangle]
pub extern "C" fn Sys_EndStreamedFile(f: i32) {}

#[no_mangle]
pub extern "C" fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, f: i32) -> c_int {
    unsafe { FS_Read(buffer, size * count, f) }
}

#[no_mangle]
pub extern "C" fn Sys_StreamSeek(f: i32, offset: c_int, origin: c_int) {
    unsafe {
        FS_Seek(f, offset, origin);
    }
}

#[no_mangle]
pub extern "C" fn Sys_InitializeCriticalSection() -> *mut c_void {
    (-1 as isize) as *mut c_void
}

#[no_mangle]
pub extern "C" fn Sys_EnterCriticalSection(_ptr: *mut c_void) {}

#[no_mangle]
pub extern "C" fn Sys_LeaveCriticalSection(_ptr: *mut c_void) {}
