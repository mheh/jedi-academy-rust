#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint};

// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #pragma warning( disable : 4786)
// #include "client.h"
// #include <windows.h>
// #include "..\smartheap\smrtheap.h"
// #if !defined(__Q_SHARED_H)
// #include "../game/q_shared.h"
// #endif
// #if !defined(_QCOMMON_H_)
// #include "../qcommon/qcommon.h"
// #endif
// #include <stdio.h>
// #include <map>
// using namespace std;

// ============================================================================
// Windows API (Platform-specific)
// ============================================================================

extern "C" {
    fn Sleep(dwMilliseconds: c_uint);
    fn OutputDebugString(lpOutputString: *const c_char);
}

// ============================================================================
// SmartHeap API (Opaque external types and functions)
// ============================================================================

pub enum MEM_POOL {}
#[repr(C)]
pub struct MEM_ERROR_INFO {
    pub objectCreationInfo: *mut MEM_ERROR_INFO,
    pub checkpoint: c_int,
    pub argSize: c_int,
    // ... other fields we don't need to fully define
}
pub type MEM_BOOL = c_int;

extern "C" {
    // Memory pool initialization and configuration
    fn MemInitDefaultPool() -> *mut MEM_POOL;
    fn MemPoolSetSmallBlockAllocator(pool: *mut MEM_POOL, allocator: c_int);
    fn MemSetErrorHandler(handler: unsafe extern "C" fn(*mut MEM_ERROR_INFO) -> MEM_BOOL);
    fn MemDefaultErrorHandler() -> unsafe extern "C" fn(*mut MEM_ERROR_INFO) -> MEM_BOOL;

    // Debug memory functions
    fn dbgMemSetGuardSize(size: c_int);
    fn dbgMemTotalCount() -> c_int;
    fn dbgMemTotalSize() -> c_int;
    fn dbgMemReportLeakage(arg1: *mut c_void, arg2: c_int, arg3: c_int);
    fn dbgMemFormatCall(info: *mut MEM_ERROR_INFO, buffer: *mut c_char, size: c_int);
    fn dbgMemSetSafetyLevel(level: c_int);
    fn dbgMemPoolSetCheckFrequency(pool: *mut MEM_POOL, freq: c_int);
    fn dbgMemSetCheckFrequency(freq: c_int);
    fn dbgMemDeferFreeing(defer: c_int);
    fn dbgMemSetDeferQueueLen(len: c_int);
    fn dbgMemSetDefaultErrorOutput(arg1: c_int, arg2: *const c_char);
}

// SmartHeap constants
const MEM_SMALL_BLOCK_SH3: c_int = 3;
const MEM_SAFETY_DEBUG: c_int = 2;
const MEM_SAFETY_SOME: c_int = 1;
const DBGMEM_OUTPUT_FILE: c_int = 0;
const DBGMEM_OUTPUT_PROMPT: c_int = 1;
const TRUE: c_int = 1;
const FALSE: c_int = 0;

// ============================================================================
// Game/Engine API
// ============================================================================

// cvar_t is mostly opaque; it has an integer field accessed in the destructor
// For now, we define it with enough of the layout to access the integer field
#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub string: *const c_char,
    pub resetString: *const c_char,
    pub latched_string: *const c_char,
    pub flags: c_int,
    pub modified: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
}

extern "C" {
    fn Cmd_AddCommand(name: *const c_char, func: extern "C" fn());
    fn Cmd_Argc() -> c_int;
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn Com_Printf(msg: *const c_char, ...);
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;

    // C standard library
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strcmpi(s1: *const c_char, s2: *const c_char) -> c_int;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn atol(nptr: *const c_char) -> i64;
}

// ============================================================================
// #if MEM_DEBUG static globals
// ============================================================================

const maxStack: c_int = 2048;

#[cfg(feature = "mem_debug")]
static mut TotalMem: c_int = 0;
#[cfg(feature = "mem_debug")]
static mut TotalBlocks: c_int = 0;
#[cfg(feature = "mem_debug")]
static mut nStack: c_int = 0;
#[cfg(feature = "mem_debug")]
static mut StackNames: [[c_char; 256]; 2048] = [[0; 256]; 2048];
#[cfg(feature = "mem_debug")]
static mut StackSize: [c_int; 2048] = [0; 2048];
#[cfg(feature = "mem_debug")]
static mut StackCount: [c_int; 2048] = [0; 2048];
#[cfg(feature = "mem_debug")]
static mut StackCache: [c_int; 48] = [0; 48];
#[cfg(feature = "mem_debug")]
static mut StackCacheAt: c_int = 0;
#[cfg(feature = "mem_debug")]
static mut CheckpointSize: [c_int; 1000] = [0; 1000];
#[cfg(feature = "mem_debug")]
static mut CheckpointCount: [c_int; 1000] = [0; 1000];

pub static mut mem_leakfile: *mut cvar_t = core::ptr::null_mut();
pub static mut mem_leakreport: *mut cvar_t = core::ptr::null_mut();

// ============================================================================
// Memory reporter callback (MEM_DEBUG)
// ============================================================================

#[cfg(feature = "mem_debug")]
unsafe extern "C" fn MyMemReporter2(info: *mut MEM_ERROR_INFO) -> MEM_BOOL {
    static mut buffer: [c_char; 10000] = [0; 10000];
    if (*info).objectCreationInfo.is_null() {
        return 1;
    }
    let mut info = (*info).objectCreationInfo;
    let idx = (*info).checkpoint;
    if idx < 0 || idx >= 1000 {
        let idx = 0;
        CheckpointCount[idx as usize] += 1;
        CheckpointSize[idx as usize] += (*info).argSize;
    } else {
        CheckpointCount[idx as usize] += 1;
        CheckpointSize[idx as usize] += (*info).argSize;
    }
    // return 1;
    dbgMemFormatCall(info, buffer.as_mut_ptr(), 9999);
    if !strstr(buffer.as_ptr(), c"ntdll".as_ptr() as *const c_char).is_null() {
        return 1;
    }
    if !strstr(buffer.as_ptr(), c"CLBCATQ".as_ptr() as *const c_char).is_null() {
        return 1;
    }
    let mut i = strlen(buffer.as_ptr());
    while i > 0 {
        i -= 1;
        if buffer[i] == b'\n' as c_char {
            break;
        }
    }
    if i == 0 {
        return 1;
    }
    buffer[i as usize] = 0;
    let mut buf = buffer.as_mut_ptr();
    while *buf != 0 {
        if *buf == b'\n' as c_char {
            buf = buf.offset(1);
            break;
        }
        buf = buf.offset(1);
    }
    let mut start: *mut c_char = core::ptr::null_mut();
    while *buf != 0 {
        while *buf == b' ' as c_char {
            buf = buf.offset(1);
        }
        start = buf;
        while *buf != 0 && *buf != b'\n' as c_char {
            buf = buf.offset(1);
        }
        if !start.is_null() {
            if *buf != 0 {
                *buf = 0;
                buf = buf.offset(1);
            }
            if strlen(start) > 255 {
                *start.offset(255) = 0;
            }
            if strstr(start, c"std::".as_ptr() as *const c_char).is_null() {
                if strstr(start, c"Malloc".as_ptr() as *const c_char).is_null() {
                    if strstr(start, c"FS_LoadFile".as_ptr() as *const c_char).is_null() {
                        if strstr(start, c"CopyString".as_ptr() as *const c_char).is_null() {
                            // break out of loop
                            break;
                        } else {
                            start = core::ptr::null_mut();
                        }
                    } else {
                        start = core::ptr::null_mut();
                    }
                } else {
                    start = core::ptr::null_mut();
                }
            }
        }
    }
    if start.is_null() || *start == 0 {
        start = c"UNKNOWN".as_ptr() as *mut c_char;
    }

    let mut i = 0;
    while i < 48 {
        if !(StackCache[i as usize] < 0 || StackCache[i as usize] >= nStack) {
            if strcmpi(start, StackNames[StackCache[i as usize] as usize].as_ptr()) == 0 {
                break;
            }
        }
        i += 1;
    }

    if i < 48 {
        let cache_idx = StackCache[i as usize] as usize;
        StackSize[cache_idx] += (*info).argSize;
        StackCount[cache_idx] += 1;
    } else {
        let mut found = false;
        let mut i = 0;
        while i < nStack {
            if strcmpi(start, StackNames[i as usize].as_ptr()) == 0 {
                StackSize[i as usize] += (*info).argSize;
                StackCount[i as usize] += 1;
                StackCache[StackCacheAt as usize] = i;
                StackCacheAt += 1;
                if StackCacheAt >= 48 {
                    StackCacheAt = 0;
                }
                found = true;
                break;
            }
            i += 1;
        }

        if !found && nStack < maxStack {
            strcpy(StackNames[nStack as usize].as_mut_ptr(), start);
            StackSize[nStack as usize] = (*info).argSize;
            StackCount[nStack as usize] = 1;
            nStack += 1;
        } else if nStack >= maxStack {
            StackSize[(maxStack - 1) as usize] += (*info).argSize;
            StackCount[(maxStack - 1) as usize] += 1;
        }
    }

    TotalMem += (*info).argSize;
    return 1;
}

// ============================================================================
// Leakage class (C++ class translated to Rust struct)
// ============================================================================

pub struct Leakage {
    MyPool: *mut MEM_POOL,
}

impl Leakage {
    pub fn new() -> Self {
        unsafe {
            let pool = MemInitDefaultPool();
            MemPoolSetSmallBlockAllocator(pool, MEM_SMALL_BLOCK_SH3);

            #[cfg(feature = "mem_debug")]
            {
                dbgMemSetGuardSize(2);
            }

            let mut leak = Leakage { MyPool: pool };
            leak.EnableChecking(100000);
            leak
        }
    }

    #[cfg(feature = "mem_debug")]
    pub fn LeakReport(&self) {
        unsafe {
            let mut mess: [c_char; 1000] = [0; 1000];
            let blocks = dbgMemTotalCount();
            let mem = dbgMemTotalSize() / 1024;
            sprintf(mess.as_mut_ptr(), c"Final Memory Summary %d blocks %d K\n".as_ptr() as *const c_char, blocks, mem);
            OutputDebugString(mess.as_ptr());

            let mut i = 0;
            while i < 1000 {
                CheckpointSize[i as usize] = 0;
                CheckpointCount[i as usize] = 0;
                i += 1;
            }

            TotalMem = 0;
            TotalBlocks = 0;
            nStack = 0;
            MemSetErrorHandler(MyMemReporter2);
            dbgMemReportLeakage(core::ptr::null_mut(), 1, 1000);
            MemSetErrorHandler(MemDefaultErrorHandler());

            if TotalBlocks != 0 {
                // Sort by size.
                Sleep(100);
                OutputDebugString(c"**************************************\n".as_ptr() as *const c_char);
                OutputDebugString(c"**********Memory Leak Report**********\n".as_ptr() as *const c_char);
                OutputDebugString(c"*************** By Size **************\n".as_ptr() as *const c_char);
                OutputDebugString(c"**************************************\n".as_ptr() as *const c_char);
                sprintf(mess.as_mut_ptr(), c"Actual leakage %d blocks %d K\n".as_ptr() as *const c_char, TotalBlocks, TotalMem / 1024);
                OutputDebugString(mess.as_ptr());

                // Use a simple Vec-based sorting instead of multimap for mechanical translation
                // multimap<int,pair<int,char *> > sortit;
                // for (i=0;i<nStack;i++)
                //     sortit.insert(pair<int,pair<int,char *> >(-StackSize[i],pair<int,char *>(StackCount[i],StackNames[i])));

                let mut sortit: Vec<(c_int, (c_int, *const c_char))> = Vec::new();
                let mut i = 0;
                while i < nStack {
                    sortit.push((-StackSize[i as usize], (StackCount[i as usize], StackNames[i as usize].as_ptr())));
                    i += 1;
                }
                sortit.sort_by_key(|&(k, _)| k);

                Sleep(5);
                for (k, (cnt, name)) in sortit.iter() {
                    sprintf(mess.as_mut_ptr(), c"%5d KB %6d cnt  %s\n".as_ptr() as *const c_char, -k / 1024, cnt, name);
                    Sleep(5);
                    OutputDebugString(mess.as_ptr());
                }

                // Sort by count.
                Sleep(100);
                OutputDebugString(c"**************************************\n".as_ptr() as *const c_char);
                OutputDebugString(c"**********Memory Leak Report**********\n".as_ptr() as *const c_char);
                OutputDebugString(c"************** By Count **************\n".as_ptr() as *const c_char);
                OutputDebugString(c"**************************************\n".as_ptr() as *const c_char);
                sprintf(mess.as_mut_ptr(), c"Actual leakage %d blocks %d K\n".as_ptr() as *const c_char, TotalBlocks, TotalMem / 1024);
                OutputDebugString(mess.as_ptr());
                sortit.clear();
                let mut i = 0;
                while i < nStack {
                    sortit.push((-StackCount[i as usize], (StackSize[i as usize], StackNames[i as usize].as_ptr())));
                    i += 1;
                }
                sortit.sort_by_key(|&(k, _)| k);

                Sleep(5);
                for (k, (size, name)) in sortit.iter() {
                    sprintf(mess.as_mut_ptr(), c"%5d KB %6d cnt  %s\n".as_ptr() as *const c_char, size / 1024, -k, name);
                    Sleep(5);
                    OutputDebugString(mess.as_ptr());
                }
            } else {
                OutputDebugString(c"No Memory Leaks\n".as_ptr() as *const c_char);
            }

            // Sort by size.
            Sleep(5);
            OutputDebugString(c"***************************************\n".as_ptr() as *const c_char);
            OutputDebugString(c"By Tag, sort: size ********************\n".as_ptr() as *const c_char);
            OutputDebugString(c"size(K)   count  name  \n".as_ptr() as *const c_char);
            OutputDebugString(c"-----------------------\n".as_ptr() as *const c_char);
            Sleep(5);
            let mut sorted: Vec<(c_int, c_int)> = Vec::new();
            let mut i = 0;
            while i < 1000 {
                if CheckpointCount[i as usize] != 0 {
                    sorted.push((-CheckpointSize[i as usize], i));
                }
                i += 1;
            }
            sorted.sort_by_key(|&(k, _)| k);
            for (k, i) in sorted.iter() {
                sprintf(mess.as_mut_ptr(), c"%8d %8d %s\n".as_ptr() as *const c_char, CheckpointSize[*i as usize] / 1024, CheckpointCount[*i as usize], c"unknown".as_ptr() as *const c_char);
                Sleep(5);
                OutputDebugString(mess.as_ptr());
            }

            // Sort by count.
            Sleep(5);
            OutputDebugString(c"***************************************\n".as_ptr() as *const c_char);
            OutputDebugString(c"By Tag, sort: count *******************\n".as_ptr() as *const c_char);
            OutputDebugString(c"size(K)   count  name  \n".as_ptr() as *const c_char);
            OutputDebugString(c"-----------------------\n".as_ptr() as *const c_char);
            Sleep(5);
            sorted.clear();
            let mut i = 0;
            while i < 1000 {
                if CheckpointCount[i as usize] != 0 {
                    sorted.push((-CheckpointCount[i as usize], i));
                }
                i += 1;
            }
            sorted.sort_by_key(|&(k, _)| k);
            for (k, i) in sorted.iter() {
                sprintf(mess.as_mut_ptr(), c"%8d %8d %s\n".as_ptr() as *const c_char, CheckpointSize[*i as usize] / 1024, CheckpointCount[*i as usize], c"unknown".as_ptr() as *const c_char);
                Sleep(5);
                OutputDebugString(mess.as_ptr());
            }
        }
    }

    #[cfg(not(feature = "mem_debug"))]
    pub fn LeakReport(&self) {
        // Empty implementation when MEM_DEBUG is not enabled
    }

    pub fn drop_impl(&self) {
        #[cfg(feature = "mem_debug")]
        unsafe {
            // #if 0
            // if (mem_leakfile && mem_leakfile->integer)
            // {
            //     dbgMemSetDefaultErrorOutput(DBGMEM_OUTPUT_FILE,"leakage.out");
            //     dbgMemReportLeakage(NULL,1,1);
            //     dbgMemSetDefaultErrorOutput(DBGMEM_OUTPUT_PROMPT,NULL);
            // }
            // #endif

            if !mem_leakreport.is_null() && (*mem_leakreport).integer != 0 {
                self.LeakReport();
            }
        }
    }

    pub fn EnableChecking(&mut self, x: c_int) {
        #[cfg(feature = "mem_debug")]
        unsafe {
            if x != 0 {
                dbgMemSetSafetyLevel(MEM_SAFETY_DEBUG);
                dbgMemPoolSetCheckFrequency(self.MyPool, x);
                dbgMemSetCheckFrequency(x);
                dbgMemDeferFreeing(TRUE);
                if x > 50000 {
                    dbgMemSetDeferQueueLen(x + 5000);
                } else {
                    dbgMemSetDeferQueueLen(50000);
                }
            } else {
                dbgMemSetSafetyLevel(MEM_SAFETY_SOME);
                dbgMemDeferFreeing(FALSE);
            }
        }
    }
}

static mut TheLeakage: Option<Leakage> = None;

// ============================================================================
// #if MEM_DEBUG - Command handlers and registration
// ============================================================================

#[cfg(feature = "mem_debug")]
extern "C" fn MEM_Checking_f() {
    unsafe {
        if Cmd_Argc() != 2 {
            Com_Printf(c"mem_checking <frequency>\n".as_ptr() as *const c_char);
            return;
        }

        let freq = atol(Cmd_Argv(1));
        if freq > 0 && freq < 100 {
            Com_Printf(c"mem_checking frequency is too low ( < 100 )\n".as_ptr() as *const c_char);
            return;
        }

        match &mut TheLeakage {
            Some(leak) => {
                leak.EnableChecking(freq as c_int);
            }
            None => {}
        }
    }
}

#[cfg(feature = "mem_debug")]
extern "C" fn MEM_Report_f() {
    unsafe {
        match &TheLeakage {
            Some(leak) => {
                leak.LeakReport();
            }
            None => {}
        }
    }
}

// void myexit(void)
// {
//     TheLeakage.LeakReport();
// }

#[cfg(feature = "mem_debug")]
pub extern "C" fn SH_Register() {
    unsafe {
        Cmd_AddCommand(c"mem_checking".as_ptr() as *const c_char, MEM_Checking_f);
        Cmd_AddCommand(c"mem_report".as_ptr() as *const c_char, MEM_Report_f);

        mem_leakfile = Cvar_Get(c"mem_leakfile".as_ptr() as *const c_char, c"1".as_ptr() as *const c_char, 0);
        mem_leakreport = Cvar_Get(c"mem_leakreport".as_ptr() as *const c_char, c"1".as_ptr() as *const c_char, 0);

        // atexit(myexit);

        // Initialize TheLeakage singleton
        TheLeakage = Some(Leakage::new());
    }
}

#[cfg(not(feature = "mem_debug"))]
pub extern "C" fn SH_Register() {
    // Empty implementation when MEM_DEBUG is not enabled
}
