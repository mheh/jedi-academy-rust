// leave this as first line for PCH reasons...
//
// #pragma warning( disable : 4786)
// #pragma warning( disable : 4100)
// #pragma warning( disable : 4663)

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void};

// Stubs for SmartHeap library types and functions (Windows-specific)
// These would be linked via Windows SmartHeap library in the actual build
type MEM_BOOL = c_int;
type MEM_POOL = *mut c_void;
type MEM_ERROR_INFO = c_void;

extern "C" {
    fn MemInitDefaultPool() -> MEM_POOL;
    fn MemPoolSetSmallBlockAllocator(pool: MEM_POOL, arg: c_int) -> ();
    fn dbgMemSetGuardSize(size: c_int) -> ();
    fn EnableChecking(x: c_int) -> ();
    fn dbgMemFormatCall(info: *mut MEM_ERROR_INFO, buffer: *mut c_char, size: c_int) -> ();
    fn dbgMemTotalCount() -> c_int;
    fn dbgMemTotalSize() -> c_int;
    fn MemSetErrorHandler(handler: extern "C" fn(*mut MEM_ERROR_INFO) -> MEM_BOOL) -> ();
    fn dbgMemReportLeakage(arg1: *mut c_void, arg2: c_int, arg3: c_int) -> ();
    fn MemDefaultErrorHandler(info: *mut MEM_ERROR_INFO) -> MEM_BOOL;
    fn dbgMemSetDefaultErrorOutput(output_type: c_int, filename: *const c_char) -> ();
    fn dbgMemSetSafetyLevel(level: c_int) -> ();
    fn dbgMemPoolSetCheckFrequency(pool: MEM_POOL, freq: c_int) -> ();
    fn dbgMemSetCheckFrequency(freq: c_int) -> ();
    fn dbgMemDeferFreeing(defer: c_int) -> ();
    fn dbgMemSetDeferQueueLen(len: c_int) -> ();

    // Windows API functions
    fn OutputDebugStringA(lpOutputString: *const c_char) -> ();
    fn Sleep(dwMilliseconds: u32) -> ();
    fn strlen(s: *const c_char) -> usize;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmpi(s1: *const c_char, s2: *const c_char) -> c_int;
    fn sprintf(buffer: *mut c_char, format: *const c_char, ...) -> c_int;

    // Q3 engine functions
    fn Cmd_Argc() -> c_int;
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn Cmd_AddCommand(name: *const c_char, func: extern "C" fn() -> ()) -> ();
    fn Cvar_Get(
        var_name: *const c_char,
        var_value: *const c_char,
        flags: c_int,
    ) -> *mut cvar_t;
    fn Com_Printf(fmt: *const c_char, ...) -> ();
    fn atol(nptr: *const c_char) -> c_int;
}

#[repr(C)]
pub struct cvar_t {
    // Stub structure for cvar_t
    // Full definition would come from q_shared.h
    integer: c_int,
    // ... other fields omitted for brevity
}

const MEM_SMALL_BLOCK_SH3: c_int = 3;
const DBGMEM_OUTPUT_FILE: c_int = 1;
const DBGMEM_OUTPUT_PROMPT: c_int = 0;
const MEM_SAFETY_DEBUG: c_int = 2;
const MEM_SAFETY_SOME: c_int = 1;

#[cfg(feature = "MEM_DEBUG")]
mod mem_debug {
    use super::*;

    const maxStack: usize = 2048;

    static mut TotalMem: c_int = 0;
    static mut TotalBlocks: c_int = 0;
    static mut nStack: c_int = 0;
    static mut StackNames: [[c_char; 256]; 2048] = [[0; 256]; 2048];
    static mut StackSize: [c_int; 2048] = [0; 2048];
    static mut StackCount: [c_int; 2048] = [0; 2048];
    static mut StackCache: [c_int; 48] = [0; 48];
    static mut StackCacheAt: c_int = 0;
    static mut CheckpointSize: [c_int; 3000] = [0; 3000];
    static mut CheckpointCount: [c_int; 3000] = [0; 3000];

    // #define _FASTRPT_
    // When enabled, uses hmap (C++ hash map) which is not available in Rust without std
    // This would require std::collections::HashMap or similar implementation

    pub static mut mem_leakfile: *mut cvar_t = core::ptr::null_mut();
    pub static mut mem_leakreport: *mut cvar_t = core::ptr::null_mut();

    extern "C" pub fn MyMemReporter2(info: *mut MEM_ERROR_INFO) -> MEM_BOOL {
        unsafe {
            static mut buffer: [c_char; 10000] = [0; 10000];

            if info.is_null() || (*info as *const _).is_null() {
                return 1;
            }

            let info_ptr = if let Some(creation_info) = (*info as *mut MEM_ERROR_INFO).as_mut() {
                creation_info as *mut _
            } else {
                return 1;
            };

            // SAFETY: Assuming info->objectCreationInfo is a valid pointer in this context
            let mut idx: c_int = 0; // checkpoint field - would be in MEM_ERROR_INFO
            if idx < 0 || idx >= 1000 {
                idx = 0;
            }

            CheckpointCount[idx as usize] += 1;
            // CheckpointSize[idx as usize] += info->argSize;

            // dbgMemFormatCall(info_ptr, buffer.as_mut_ptr(), 9999);

            // Stub - Windows SmartHeap function not available in Rust
            // Would need linking to Windows SmartHeap library

            if !strstr(buffer.as_ptr(), b"ntdll\0".as_ptr() as *const c_char).is_null() {
                return 1;
            }
            if !strstr(buffer.as_ptr(), b"CLBCATQ\0".as_ptr() as *const c_char).is_null() {
                return 1;
            }

            let mut i: c_int;
            TotalBlocks += 1;
            if TotalBlocks % 1000 == 0 {
                let mut mess: [c_char; 1000] = [0; 1000];
                sprintf(
                    mess.as_mut_ptr(),
                    b"%d blocks processed\n\0".as_ptr() as *const c_char,
                    TotalBlocks,
                );
                OutputDebugStringA(mess.as_ptr());
            }

            i = strlen(buffer.as_ptr()) as c_int;
            while i > 0 {
                if buffer[i as usize] == b'\n' as c_char {
                    break;
                }
                i -= 1;
            }

            if i == 0 {
                return 1;
            }

            buffer[i as usize] = 0;
            let mut buf: *mut c_char = buffer.as_mut_ptr();

            while *buf != 0 {
                if *buf == b'\n' as c_char {
                    buf = buf.offset(1);
                    break;
                }
                buf = buf.offset(1);
            }

            let mut start: *mut c_char = core::ptr::null_mut();
            let mut altName: *mut c_char = core::ptr::null_mut();

            while *buf != 0 {
                while *buf == b' ' as c_char {
                    buf = buf.offset(1);
                }
                start = buf;
                while *buf != 0 && *buf != b'\n' as c_char {
                    buf = buf.offset(1);
                }

                if *start != 0 {
                    if *buf != 0 {
                        *buf = 0;
                        buf = buf.offset(1);
                    }

                    if strlen(start) > 255 {
                        *start.offset(255) = 0;
                    }

                    if !strstr(start, b"std::\0".as_ptr() as *const c_char).is_null() {
                        altName = b"std::??\0".as_ptr() as *mut c_char;
                        //				start=0;
                        continue;
                    }
                    if !strstr(start, b"Malloc\0".as_ptr() as *const c_char).is_null() {
                        altName = b"Malloc??\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    if !strstr(start, b"G_Alloc\0".as_ptr() as *const c_char).is_null() {
                        altName = b"G_Alloc\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    if !strstr(start, b"Hunk_Alloc\0".as_ptr() as *const c_char).is_null() {
                        altName = b"Hunk_Alloc\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    if !strstr(start, b"FS_LoadFile\0".as_ptr() as *const c_char).is_null() {
                        altName = b"FS_LoadFile\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    if !strstr(start, b"CopyString\0".as_ptr() as *const c_char).is_null() {
                        altName = b"CopyString\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    break;
                }
            }

            if start.is_null() || *start == 0 {
                start = altName;
                if start.is_null() || *start == 0 {
                    start = b"UNKNOWN\0".as_ptr() as *mut c_char;
                }
            }

            // #ifdef _FASTRPT_
            // Would use hmap (C++ hash map) here - not ported to Rust without std
            // #else

            for i in 0..48 {
                if StackCache[i] < 0 || StackCache[i] >= nStack {
                    continue;
                }
                if strcmpi(
                    start,
                    StackNames[StackCache[i as usize] as usize].as_ptr(),
                ) == 0
                {
                    break;
                }
            }

            if i < 48 {
                StackSize[StackCache[i] as usize] += 0; // info->argSize
                StackCount[StackCache[i] as usize] += 1;
            } else {
                let mut i: c_int = 0;
                while i < nStack {
                    if strcmpi(start, StackNames[i as usize].as_ptr()) == 0 {
                        break;
                    }
                    i += 1;
                }

                if i < nStack {
                    StackSize[i as usize] += 0; // info->argSize
                    StackCount[i as usize] += 1;
                    StackCache[StackCacheAt as usize] = i;
                    StackCacheAt += 1;
                    if StackCacheAt >= 48 {
                        StackCacheAt = 0;
                    }
                } else if i < maxStack as c_int {
                    strcpy(
                        StackNames[i as usize].as_mut_ptr(),
                        start,
                    );
                    StackSize[i as usize] = 0; // info->argSize
                    StackCount[i as usize] = 1;
                    nStack += 1;
                } else if nStack < maxStack as c_int {
                    nStack += 1;
                    strcpy(
                        StackNames[(maxStack - 1) as usize].as_mut_ptr(),
                        b"*****OTHER*****\0".as_ptr() as *mut c_char,
                    );
                    StackSize[maxStack - 1] = 0; // info->argSize
                    StackCount[maxStack - 1] = 1;
                } else {
                    StackSize[maxStack - 1] += 0; // info->argSize
                    StackCount[maxStack - 1] += 1;
                }
            }
            // #endif

            TotalMem += 0; // info->argSize
            return 1;
        }
    }

    extern "C" pub fn MyMemReporter3(info: *mut MEM_ERROR_INFO) -> MEM_BOOL {
        unsafe {
            static mut buffer: [c_char; 10000] = [0; 10000];

            if info.is_null() || (*info as *const _).is_null() {
                return 1;
            }

            let info_ptr = if let Some(creation_info) = (*info as *mut MEM_ERROR_INFO).as_mut() {
                creation_info as *mut _
            } else {
                return 1;
            };

            let mut idx: c_int = 0; // checkpoint field
            if idx < 0 || idx >= 3000 {
                idx = 0;
            }
            CheckpointCount[idx as usize] += 1;
            // CheckpointSize[idx as usize] += info->argSize;

            // dbgMemFormatCall(info_ptr, buffer.as_mut_ptr(), 9999);
            // Stub - Windows SmartHeap function not available in Rust

            let mut i: c_int;
            TotalBlocks += 1;
            //	if (TotalBlocks%1000==0)
            //	{
            //		char mess[1000];
            //		sprintf(mess,"%d blocks processed\n",TotalBlocks);
            //		OutputDebugString(mess);
            //	}

            i = strlen(buffer.as_ptr()) as c_int;
            while i > 0 {
                if buffer[i as usize] == b'\n' as c_char {
                    break;
                }
                i -= 1;
            }

            if i == 0 {
                return 1;
            }

            buffer[i as usize] = 0;
            let mut buf: *mut c_char = buffer.as_mut_ptr();

            while *buf != 0 {
                if *buf == b'\n' as c_char {
                    buf = buf.offset(1);
                    break;
                }
                buf = buf.offset(1);
            }

            let mut start: *mut c_char = core::ptr::null_mut();
            let mut altName: *mut c_char = core::ptr::null_mut();

            while *buf != 0 {
                while *buf == b' ' as c_char {
                    buf = buf.offset(1);
                }
                start = buf;
                while *buf != 0 && *buf != b'\n' as c_char {
                    buf = buf.offset(1);
                }

                if *start != 0 {
                    if *buf != 0 {
                        *buf = 0;
                        buf = buf.offset(1);
                    }

                    if strlen(start) > 255 {
                        *start.offset(255) = 0;
                    }

                    if !strstr(start, b"SV_AreaEntities\0".as_ptr() as *const c_char).is_null() {
                        altName = b"SV_AreaEntities??\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    if !strstr(start, b"SV_Trace\0".as_ptr() as *const c_char).is_null() {
                        altName = b"SV_Trace??\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    if !strstr(start, b"SV_PointContents\0".as_ptr() as *const c_char).is_null() {
                        altName = b"SV_PointContents??\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    if !strstr(start, b"CG_Trace\0".as_ptr() as *const c_char).is_null() {
                        altName = b"??\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    if !strstr(start, b"CG_PointContents\0".as_ptr() as *const c_char).is_null() {
                        altName = b"??\0".as_ptr() as *mut c_char;
                        start = core::ptr::null_mut();
                        continue;
                    }
                    /*
                    if !strstr(start, b"").is_null()
                    {
                        altName="??";
                        start=0;
                        continue;
                    }
                    if !strstr(start, b"").is_null()
                    {
                        altName="??";
                        start=0;
                        continue;
                    }
                    */
                    break;
                }
            }

            if start.is_null() || *start == 0 {
                start = altName;
                if start.is_null() || *start == 0 {
                    start = b"UNKNOWN\0".as_ptr() as *mut c_char;
                }
            }

            // #ifdef _FASTRPT_
            // Would use hmap (C++ hash map) here - not ported to Rust without std
            // #else

            for i in 0..48 {
                if StackCache[i] < 0 || StackCache[i] >= nStack {
                    continue;
                }
                if strcmpi(
                    start,
                    StackNames[StackCache[i as usize] as usize].as_ptr(),
                ) == 0
                {
                    break;
                }
            }

            if i < 48 {
                StackSize[StackCache[i] as usize] += 0; // info->argSize
                StackCount[StackCache[i] as usize] += 1;
            } else {
                let mut i: c_int = 0;
                while i < nStack {
                    if strcmpi(start, StackNames[i as usize].as_ptr()) == 0 {
                        break;
                    }
                    i += 1;
                }

                if i < nStack {
                    StackSize[i as usize] += 0; // info->argSize
                    StackCount[i as usize] += 1;
                    StackCache[StackCacheAt as usize] = i;
                    StackCacheAt += 1;
                    if StackCacheAt >= 48 {
                        StackCacheAt = 0;
                    }
                } else if i < maxStack as c_int {
                    strcpy(
                        StackNames[i as usize].as_mut_ptr(),
                        start,
                    );
                    StackSize[i as usize] = 0; // info->argSize
                    StackCount[i as usize] = 1;
                    nStack += 1;
                } else if nStack < maxStack as c_int {
                    nStack += 1;
                    strcpy(
                        StackNames[(maxStack - 1) as usize].as_mut_ptr(),
                        b"*****OTHER*****\0".as_ptr() as *mut c_char,
                    );
                    StackSize[maxStack - 1] = 0; // info->argSize
                    StackCount[maxStack - 1] = 1;
                } else {
                    StackSize[maxStack - 1] += 0; // info->argSize
                    StackCount[maxStack - 1] += 1;
                }
            }
            // #endif

            TotalMem += 0; // info->argSize
            return 1;
        }
    }

    pub extern "C" fn SH_Checking_f() {}
}

pub struct Leakage {
    MyPool: MEM_POOL,
}

impl Leakage {
    pub fn new() -> Self {
        unsafe {
            let pool = MemInitDefaultPool();
            //		MemPoolSetSmallBlockSize(MyPool, 16);
            MemPoolSetSmallBlockAllocator(pool, MEM_SMALL_BLOCK_SH3);
            #[cfg(feature = "MEM_DEBUG")]
            {
                dbgMemSetGuardSize(2);
                // EnableChecking(100000);
            }
            Leakage { MyPool: pool }
        }
    }

    #[cfg(feature = "MEM_DEBUG")]
    pub fn LeakReport(&mut self) {
        unsafe {
            use core::ptr::addr_of_mut;

            // This just makes sure we have map nodes available without allocation
            // during the heap walk (which could be bad).
            let mut i: c_int;
            // #ifdef _FASTRPT_
            // hlist<int> makeSureWeHaveNodes;
            // for(i=0;i<5000;i++)
            // {
            //     makeSureWeHaveNodes.push_back(0);
            // }
            // makeSureWeHaveNodes.clear();
            // Lookup.clear();
            // #endif

            let mut mess: [c_char; 1000] = [0; 1000];
            let blocks = dbgMemTotalCount();
            let mem = dbgMemTotalSize() / 1024;
            sprintf(
                mess.as_mut_ptr(),
                b"Final Memory Summary %d blocks %d K\n\0".as_ptr() as *const c_char,
                blocks,
                mem,
            );
            OutputDebugStringA(mess.as_ptr());

            i = 0;
            while i < 3000 {
                mem_debug::CheckpointSize[i as usize] = 0;
                mem_debug::CheckpointCount[i as usize] = 0;
                i += 1;
            }

            mem_debug::TotalMem = 0;
            mem_debug::TotalBlocks = 0;
            mem_debug::nStack = 0;
            MemSetErrorHandler(mem_debug::MyMemReporter2);
            dbgMemReportLeakage(core::ptr::null_mut(), 1, 1000);
            MemSetErrorHandler(MemDefaultErrorHandler);

            // multimap<int,pair<int,char *> > sortit;
            // multimap<int,pair<int,char *> >::iterator j;

            if mem_debug::TotalBlocks != 0 {
                // Sort by size.
                Sleep(100);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"**********Memory Leak Report**********\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"*************** By Size **************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                sprintf(
                    mess.as_mut_ptr(),
                    b"Actual leakage %d blocks %d K\n\0".as_ptr() as *const c_char,
                    mem_debug::TotalBlocks,
                    mem_debug::TotalMem / 1024,
                );
                OutputDebugStringA(mess.as_ptr());

                // sortit.clear();
                i = 0;
                while i < mem_debug::nStack {
                    // sortit.insert(pair<int,pair<int,char *> >(-StackSize[i],pair<int,char *>(StackCount[i],StackNames[i])));
                    i += 1;
                }

                Sleep(5);
                // for (j=sortit.begin();j!=sortit.end();j++)
                // {
                //     sprintf(mess,"%5d KB %6d cnt  %s\n",-(*j).first/1024,(*j).second.first,(*j).second.second);
                //     Sleep(5);
                //     OutputDebugString(mess);
                // }

                // Sort by count.
                Sleep(100);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"**********Memory Leak Report**********\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"************** By Count **************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                sprintf(
                    mess.as_mut_ptr(),
                    b"Actual leakage %d blocks %d K\n\0".as_ptr() as *const c_char,
                    mem_debug::TotalBlocks,
                    mem_debug::TotalMem / 1024,
                );
                OutputDebugStringA(mess.as_ptr());

                // sortit.clear();
                i = 0;
                while i < mem_debug::nStack {
                    // sortit.insert(pair<int,pair<int,char *> >(-StackCount[i],pair<int,char *>(StackSize[i],StackNames[i])));
                    i += 1;
                }

                Sleep(5);
                // for (j=sortit.begin();j!=sortit.end();j++)
                // {
                //     sprintf(mess,"%5d KB %6d cnt  %s\n",(*j).second.first/1024,-(*j).first,(*j).second.second);
                //     Sleep(5);
                //     OutputDebugString(mess);
                // }
            } else {
                OutputDebugStringA(b"No Memory Leaks\n\0".as_ptr() as *const c_char);
            }

            mem_debug::TotalMem = 0;
            mem_debug::TotalBlocks = 0;
            mem_debug::nStack = 0;
            MemSetErrorHandler(mem_debug::MyMemReporter3);
            dbgMemReportLeakage(core::ptr::null_mut(), 2001, 2001);
            MemSetErrorHandler(MemDefaultErrorHandler);
            if mem_debug::TotalBlocks != 0 {
                // Sort by count.
                Sleep(100);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"SV_PointContents     \0".as_ptr() as *const c_char);
                sprintf(
                    mess.as_mut_ptr(),
                    b"%d Calls.\n\0".as_ptr() as *const c_char,
                    mem_debug::TotalBlocks,
                );
                OutputDebugStringA(mess.as_ptr());
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                // sortit.clear();
                i = 0;
                while i < mem_debug::nStack {
                    // sortit.insert(pair<int,pair<int,char *> >(-StackCount[i],pair<int,char *>(StackSize[i],StackNames[i])));
                    i += 1;
                }
                Sleep(5);
                // for (j=sortit.begin();j!=sortit.end();j++)
                // {
                //     sprintf(mess,"%7d cnt  %s\n",-(*j).first,(*j).second.second);
                //     Sleep(5);
                //     OutputDebugString(mess);
                // }
            }

            mem_debug::TotalMem = 0;
            mem_debug::TotalBlocks = 0;
            mem_debug::nStack = 0;
            MemSetErrorHandler(mem_debug::MyMemReporter3);
            dbgMemReportLeakage(core::ptr::null_mut(), 2002, 2002);
            MemSetErrorHandler(MemDefaultErrorHandler);
            if mem_debug::TotalBlocks != 0 {
                // Sort by count.
                Sleep(100);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"SV_Trace     \0".as_ptr() as *const c_char);
                sprintf(
                    mess.as_mut_ptr(),
                    b"%d Calls.\n\0".as_ptr() as *const c_char,
                    mem_debug::TotalBlocks,
                );
                OutputDebugStringA(mess.as_ptr());
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                // sortit.clear();
                i = 0;
                while i < mem_debug::nStack {
                    // sortit.insert(pair<int,pair<int,char *> >(-StackCount[i],pair<int,char *>(StackSize[i],StackNames[i])));
                    i += 1;
                }
                Sleep(5);
                // for (j=sortit.begin();j!=sortit.end();j++)
                // {
                //     sprintf(mess,"%7d cnt  %s\n",-(*j).first,(*j).second.second);
                //     Sleep(5);
                //     OutputDebugString(mess);
                // }
            }

            mem_debug::TotalMem = 0;
            mem_debug::TotalBlocks = 0;
            mem_debug::nStack = 0;
            MemSetErrorHandler(mem_debug::MyMemReporter3);
            dbgMemReportLeakage(core::ptr::null_mut(), 2003, 2003);
            MemSetErrorHandler(MemDefaultErrorHandler);
            if mem_debug::TotalBlocks != 0 {
                // Sort by count.
                Sleep(100);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"SV_AreaEntities     \0".as_ptr() as *const c_char);
                sprintf(
                    mess.as_mut_ptr(),
                    b"%d Calls.\n\0".as_ptr() as *const c_char,
                    mem_debug::TotalBlocks,
                );
                OutputDebugStringA(mess.as_ptr());
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                // sortit.clear();
                i = 0;
                while i < mem_debug::nStack {
                    // sortit.insert(pair<int,pair<int,char *> >(-StackCount[i],pair<int,char *>(StackSize[i],StackNames[i])));
                    i += 1;
                }
                Sleep(5);
                // for (j=sortit.begin();j!=sortit.end();j++)
                // {
                //     sprintf(mess,"%7d cnt  %s\n",-(*j).first,(*j).second.second);
                //     Sleep(5);
                //     OutputDebugString(mess);
                // }
            }

            mem_debug::TotalMem = 0;
            mem_debug::TotalBlocks = 0;
            mem_debug::nStack = 0;
            MemSetErrorHandler(mem_debug::MyMemReporter3);
            dbgMemReportLeakage(core::ptr::null_mut(), 2004, 2004);
            MemSetErrorHandler(MemDefaultErrorHandler);
            if mem_debug::TotalBlocks != 0 {
                // Sort by count.
                Sleep(100);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"CG_Trace     \0".as_ptr() as *const c_char);
                sprintf(
                    mess.as_mut_ptr(),
                    b"%d Calls.\n\0".as_ptr() as *const c_char,
                    mem_debug::TotalBlocks,
                );
                OutputDebugStringA(mess.as_ptr());
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                // sortit.clear();
                i = 0;
                while i < mem_debug::nStack {
                    // sortit.insert(pair<int,pair<int,char *> >(-StackCount[i],pair<int,char *>(StackSize[i],StackNames[i])));
                    i += 1;
                }
                Sleep(5);
                // for (j=sortit.begin();j!=sortit.end();j++)
                // {
                //     sprintf(mess,"%7d cnt  %s\n",-(*j).first,(*j).second.second);
                //     Sleep(5);
                //     OutputDebugString(mess);
                // }
            }

            mem_debug::TotalMem = 0;
            mem_debug::TotalBlocks = 0;
            mem_debug::nStack = 0;
            MemSetErrorHandler(mem_debug::MyMemReporter3);
            dbgMemReportLeakage(core::ptr::null_mut(), 2005, 2005);
            MemSetErrorHandler(MemDefaultErrorHandler);
            if mem_debug::TotalBlocks != 0 {
                // Sort by count.
                Sleep(100);
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                OutputDebugStringA(b"CG_PointContents     \0".as_ptr() as *const c_char);
                sprintf(
                    mess.as_mut_ptr(),
                    b"%d Calls.\n\0".as_ptr() as *const c_char,
                    mem_debug::TotalBlocks,
                );
                OutputDebugStringA(mess.as_ptr());
                OutputDebugStringA(b"**************************************\n\0".as_ptr() as *const c_char);
                // sortit.clear();
                i = 0;
                while i < mem_debug::nStack {
                    // sortit.insert(pair<int,pair<int,char *> >(-StackCount[i],pair<int,char *>(StackSize[i],StackNames[i])));
                    i += 1;
                }
                Sleep(5);
                // for (j=sortit.begin();j!=sortit.end();j++)
                // {
                //     sprintf(mess,"%7d cnt  %s\n",-(*j).first,(*j).second.second);
                //     Sleep(5);
                //     OutputDebugString(mess);
                // }
            }

            // #if 0 //sw doesn't have the tag stuff
            // // Sort by size.
            // Sleep(5);
            // OutputDebugString("***************************************\n");
            // OutputDebugString("By Tag, sort: size ********************\n");
            // OutputDebugString("size(K)   count  name  \n");
            // OutputDebugString("-----------------------\n");
            // Sleep(5);
            // multimap<int,int> sorted;
            // for (i=0;i<1000;i++)
            // {
            //     if (CheckpointCount[i])
            //     {
            //         sorted.insert(pair<int,int>(-CheckpointSize[i],i));
            //     }
            // }
            // multimap<int,int>::iterator k;
            // for (k=sorted.begin();k!=sorted.end();k++)
            // {
            //     sprintf(mess,"%8d %8d %s\n",CheckpointSize[(*k).second]/1024,CheckpointCount[(*k).second],(*k).second>=2?tagDefs[(*k).second-2]:"unknown");
            //     Sleep(5);
            //     OutputDebugString(mess);
            // }
            //
            // // Sort by count.
            // Sleep(5);
            // OutputDebugString("By Tag, sort: count *******************\n");
            // OutputDebugString("size(K)   count  name  \n");
            // OutputDebugString("-----------------------\n");
            // Sleep(5);
            // sorted.clear();
            // for (i=0;i<1000;i++)
            // {
            //     if (CheckpointCount[i])
            //     {
            //         sorted.insert(pair<int,int>(-CheckpointCount[i],i));
            //     }
            // }
            // for (k=sorted.begin();k!=sorted.end();k++)
            // {
            //     sprintf(mess,"%8d %8d %s\n",CheckpointSize[(*k).second]/1024,CheckpointCount[(*k).second],(*k).second>=2?tagDefs[(*k).second-2]:"unknown");
            //     Sleep(5);
            //     OutputDebugString(mess);
            // }
            // #endif
        }
    }
}

impl Drop for Leakage {
    fn drop(&mut self) {
        #[cfg(feature = "MEM_DEBUG")]
        unsafe {
            if !mem_debug::mem_leakfile.is_null()
                && (*mem_debug::mem_leakfile).integer != 0
            {
                dbgMemSetDefaultErrorOutput(DBGMEM_OUTPUT_FILE, b"leakage.out\0".as_ptr() as *const c_char);
                dbgMemReportLeakage(core::ptr::null_mut(), 1, 1);
                dbgMemSetDefaultErrorOutput(DBGMEM_OUTPUT_PROMPT, core::ptr::null());
            }
            if !mem_debug::mem_leakreport.is_null()
                && (*mem_debug::mem_leakreport).integer != 0
            {
                let mut leakage = self;
                leakage.LeakReport();
            }
        }
    }
}

#[cfg(feature = "MEM_DEBUG")]
impl Leakage {
    pub fn EnableChecking(&mut self, x: c_int) {
        unsafe {
            if x != 0 {
                dbgMemSetSafetyLevel(MEM_SAFETY_DEBUG);
                dbgMemPoolSetCheckFrequency(self.MyPool, x);
                dbgMemSetCheckFrequency(x);
                dbgMemDeferFreeing(1);
                dbgMemSetDeferQueueLen(50000);
            } else {
                dbgMemSetSafetyLevel(MEM_SAFETY_SOME);
                dbgMemDeferFreeing(0);
            }
        }
    }
}

static mut TheLeakage: Option<Leakage> = None;

pub fn init_the_leakage() {
    unsafe {
        TheLeakage = Some(Leakage::new());
    }
}

#[cfg(feature = "MEM_DEBUG")]
pub extern "C" fn MEM_Checking_f() {
    unsafe {
        if Cmd_Argc() != 2 {
            Com_Printf(b"mem_checking <frequency>\n\0".as_ptr() as *const c_char);
            return;
        }

        let arg = atol(Cmd_Argv(1));
        if arg > 0 && arg < 100 {
            Com_Printf(
                b"mem_checking frequency is too low ( < 100 )\n\0".as_ptr() as *const c_char,
            );
            return;
        }

        if let Some(ref mut leakage) = TheLeakage {
            leakage.EnableChecking(arg);
        }
    }
}

#[cfg(feature = "MEM_DEBUG")]
pub extern "C" fn MEM_Report_f() {
    unsafe {
        if false {
            dbgMemSetDefaultErrorOutput(DBGMEM_OUTPUT_FILE, b"leakage.out\0".as_ptr() as *const c_char);
            dbgMemReportLeakage(core::ptr::null_mut(), 1, 1);
            dbgMemSetDefaultErrorOutput(DBGMEM_OUTPUT_PROMPT, core::ptr::null());
        }
        if let Some(ref mut leakage) = TheLeakage {
            leakage.LeakReport();
        }
    }
}

/*
void myexit(void)
{
    TheLeakage.LeakReport();
}
*/

#[cfg(feature = "MEM_DEBUG")]
pub extern "C" fn SH_Register() {
    unsafe {
        Cmd_AddCommand(
            b"mem_checking\0".as_ptr() as *const c_char,
            MEM_Checking_f,
        );
        Cmd_AddCommand(b"mem_report\0".as_ptr() as *const c_char, MEM_Report_f);

        mem_debug::mem_leakfile = Cvar_Get(b"mem_leakfile\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        mem_debug::mem_leakreport = Cvar_Get(
            b"mem_leakreport\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            0,
        );
        //	atexit(myexit);
    }
}
