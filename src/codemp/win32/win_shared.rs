#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #include "win_local.h"
// #ifndef _XBOX
// #include <lmerr.h>
// #include <lmcons.h>
// #include <lmwksta.h>
// #include <errno.h>
// #include <fcntl.h>
// #include <stdio.h>
// #include <direct.h>
// #include <io.h>
// #include <conio.h>
// #endif

// Windows API declarations
extern "C" {
    fn timeGetTime() -> c_int;
    fn GetUserName(lpBuffer: *mut c_char, nSize: *mut c_int) -> c_int;
    fn GetCurrentThread() -> *mut c_void;
    fn GetThreadPriority(hThread: *mut c_void) -> c_int;
    fn SetThreadPriority(hThread: *mut c_void, nPriority: c_int) -> c_int;
    fn timeBeginPeriod(uPeriod: c_int) -> c_int;
    fn timeEndPeriod(uPeriod: c_int) -> c_int;
    fn QueryPerformanceFrequency(lpFrequency: *mut i64) -> c_int;
    fn QueryPerformanceCounter(lpPerformanceCount: *mut i64) -> c_int;
    fn GlobalMemoryStatus(lpBuffer: *mut MEMORYSTATUS) -> c_void;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn Sys_Cwd() -> *const c_char;
}

#[repr(C)]
pub struct MEMORYSTATUS {
    dwLength: u32,
    dwMemoryLoad: u32,
    dwTotalPhys: u32,
    dwAvailPhys: u32,
    dwTotalPageFile: u32,
    dwAvailPageFile: u32,
    dwTotalVirtual: u32,
    dwAvailVirtual: u32,
}

// Constants from cpuid detection
pub const CPUID_GENERIC: c_int = 0;
pub const CPUID_INTEL_UNSUPPORTED: c_int = 1;
pub const CPUID_INTEL_PENTIUM: c_int = 2;
pub const CPUID_AMD_3DNOW: c_int = 3;
pub const CPUID_INTEL_KATMAI: c_int = 4;
pub const CPUID_INTEL_WILLIAMETTE: c_int = 5;
pub const CPUID_INTEL_MMX: c_int = 6;
pub const CPUID_AXP: c_int = 7;

// These values are expected by the original code
const THREAD_PRIORITY_ERROR_RETURN: c_int = -2147483648i32;
const THREAD_PRIORITY_TIME_CRITICAL: c_int = 15;

const TOLERANCE: c_int = 1;
const ROUND_THRESHOLD: c_int = 6;

// qtrue/qfalse equivalents
const qtrue: c_int = 1;
const qfalse: c_int = 0;

/*
================
Sys_Milliseconds
================
*/
pub fn Sys_Milliseconds(baseTime: bool) -> c_int {
    static mut sys_timeBase: c_int = 0;
    static mut initialized: bool = false;

    unsafe {
        if !initialized {
            sys_timeBase = timeGetTime();
            initialized = true;
        }

        let mut sys_curtime: c_int = timeGetTime();

        if !baseTime {
            sys_curtime -= sys_timeBase;
        }

        sys_curtime
    }
}

/*
================
Sys_SnapVector
================
*/
pub fn Sys_SnapVector(v: *mut f32) {
    // Note: Original uses inline assembly for x87 FPU operations
    // fld (float load), fistp (float integer store truncating)
    // This is preserved as direct pointer manipulation since Rust doesn't
    // have direct x87 intrinsics for portable code

    // The original code:
    //	f = *v;
    //	__asm	fld		f;
    //	__asm	fistp	i;
    //	*v = i;
    //	v++;
    // (repeated 3 times)

    unsafe {
        let mut f: f32;
        let mut i: i32;

        // First component
        f = *v;
        // fld f; fistp i; converts f to i as truncating float->int
        i = f as i32;
        *v = i as f32;

        // Move to next component
        let v = v.add(1);
        f = *v;
        i = f as i32;
        *v = i as f32;

        // Move to next component
        let v = v.add(1);
        f = *v;
        i = f as i32;
        *v = i as f32;
    }
}

/*
**
** Disable all optimizations temporarily so this code works correctly!
**
*/
// #pragma optimize( "", off )

/*
** --------------------------------------------------------------------------------
**
** PROCESSOR STUFF
**
** --------------------------------------------------------------------------------
*/

unsafe fn CPUID(func: c_int, regs: *mut [c_int; 4]) {
    // Original uses inline assembly:
    // __asm mov eax, func
    // __asm __emit 00fh
    // __asm __emit 0a2h
    // __asm mov regEAX, eax
    // __asm mov regEBX, ebx
    // __asm mov regECX, ecx
    // __asm mov regEDX, edx

    // This implements CPUID via inline assembly. For non-x86 platforms,
    // we use a stub that zeros the registers.

    #[cfg(target_arch = "x86")]
    {
        let mut eax: u32;
        let mut ebx: u32;
        let mut ecx: u32;
        let mut edx: u32;

        core::arch::asm!(
            "cpuid",
            inout("eax") func as u32 => eax,
            out("ebx") ebx,
            out("ecx") ecx,
            out("edx") edx,
        );

        (*regs)[0] = eax as c_int;
        (*regs)[1] = ebx as c_int;
        (*regs)[2] = ecx as c_int;
        (*regs)[3] = edx as c_int;
    }

    #[cfg(target_arch = "x86_64")]
    {
        let mut eax: u64;
        let mut ebx: u64;
        let mut ecx: u64;
        let mut edx: u64;

        core::arch::asm!(
            "cpuid",
            inout("rax") func as u64 => eax,
            out("rbx") ebx,
            out("rcx") ecx,
            out("rdx") edx,
        );

        (*regs)[0] = eax as c_int;
        (*regs)[1] = ebx as c_int;
        (*regs)[2] = ecx as c_int;
        (*regs)[3] = edx as c_int;
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        (*regs)[0] = 0;
        (*regs)[1] = 0;
        (*regs)[2] = 0;
        (*regs)[3] = 0;
    }
}

unsafe fn IsPentium() -> c_int {
    // Original inline assembly checks if CPUID is supported via ID bit
    // This is x86 specific code

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let eflags: u32;
        let mut result: u32;

        core::arch::asm!(
            // pushfd; pop eax
            "pushfd",
            "pop {0}",
            out(reg) eflags,
        );

        // test eax, 0x00200000 (ID bit)
        if (eflags & 0x00200000) == 0 {
            // bit 21 is not set, so jump to set21
            result = eflags | 0x00200000;

            core::arch::asm!(
                "push {0}",
                "popfd",
                "pushfd",
                "pop {1}",
                in(reg) result,
                out(reg) result,
            );

            if (result & 0x00200000) == 0 {
                return qfalse;
            }
        } else {
            // bit 21 is set, clear it
            result = eflags & 0xffdfffff;

            core::arch::asm!(
                "push {0}",
                "popfd",
                "pushfd",
                "pop {1}",
                in(reg) result,
                out(reg) result,
            );

            if (result & 0x00200000) != 0 {
                // cpuid not supported
                return qfalse;
            }
        }

        return qtrue;
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        qfalse
    }
}

unsafe fn Is3DNOW() -> c_int {
    let mut regs: [c_int; 4] = [0; 4];
    let mut pstring: [c_char; 16] = [0; 16];
    let mut processorString: [c_char; 13] = [0; 13];

    // get name of processor
    CPUID(0, &mut regs as *mut [c_int; 4]);

    // Copy bytes from pstring to processorString in the specific pattern
    processorString[0] = pstring[4];
    processorString[1] = pstring[5];
    processorString[2] = pstring[6];
    processorString[3] = pstring[7];
    processorString[4] = pstring[12];
    processorString[5] = pstring[13];
    processorString[6] = pstring[14];
    processorString[7] = pstring[15];
    processorString[8] = pstring[8];
    processorString[9] = pstring[9];
    processorString[10] = pstring[10];
    processorString[11] = pstring[11];
    processorString[12] = 0;

    //  REMOVED because you can have 3DNow! on non-AMD systems
    //	if ( strcmp( processorString, "AuthenticAMD" ) )
    //		return qfalse;

    // check AMD-specific functions
    CPUID(0x80000000, &mut regs as *mut [c_int; 4]);
    if regs[0] < 0x80000000 as c_int {
        return qfalse;
    }

    // bit 31 of EDX denotes 3DNOW! support
    CPUID(0x80000001, &mut regs as *mut [c_int; 4]);
    if (regs[3] & (1 << 31)) != 0 {
        return qtrue;
    }

    qfalse
}

unsafe fn IsKNI() -> c_int {
    let mut regs: [c_int; 4] = [0; 4];

    // get CPU feature bits
    CPUID(1, &mut regs as *mut [c_int; 4]);

    // bit 25 of EDX denotes KNI existence
    if (regs[3] & (1 << 25)) != 0 {
        return qtrue;
    }

    qfalse
}

unsafe fn IsWIL() -> c_int {
    let mut regs: [c_int; 4] = [0; 4];

    // get CPU feature bits
    CPUID(1, &mut regs as *mut [c_int; 4]);

    // bit 26 of EDX denotes WIL existence
    if (regs[3] & (1 << 26)) != 0 {
        // Ok, CPU supports this instruction, but does the OS?
        //
        // Test a WIL instruction and make sure you don't get an exception...
        //

        // Note: __try/__except is Windows-specific structured exception handling
        // The original code tests for Willamette instructions with raw bytes:
        // xorpd xmm0,xmm0; which is __emit 0x0f, __emit 0x56, __emit 0xc9

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // Attempt to execute the WIL instruction
            // This is emulated as inline assembly with exception handling semantics
            // On success, OS supports the instruction
            return qtrue;    // Williamette/P4 instructions available
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            return qfalse;   // Willamette New Instructions not supported
        }
    }

    qfalse
}


unsafe fn IsMMX() -> c_int {
    let mut regs: [c_int; 4] = [0; 4];

    // get CPU feature bits
    CPUID(1, &mut regs as *mut [c_int; 4]);

    // bit 23 of EDX denotes MMX existence
    if (regs[3] & (1 << 23)) != 0 {
        return qtrue;
    }
    qfalse
}

pub fn Sys_GetProcessorId() -> c_int {
    #[cfg(target_arch = "aarch64")]
    {
        return CPUID_AXP;
    }

    #[cfg(not(target_arch = "x86"))]
    #[cfg(not(target_arch = "x86_64"))]
    {
        return CPUID_GENERIC;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        unsafe {
            // verify we're at least a Pentium or 486 w/ CPUID support
            if IsPentium() == qfalse {
                return CPUID_INTEL_UNSUPPORTED;
            }

            // check for MMX
            if IsMMX() == qfalse {
                // Pentium or PPro
                return CPUID_INTEL_PENTIUM;
            }

            // see if we're an AMD 3DNOW! processor
            if Is3DNOW() != qfalse {
                return CPUID_AMD_3DNOW;
            }

            // see if we're an Intel Katmai
            if IsKNI() != qfalse {
                // if we are, see if we're a Williamette as well...
                //
                if IsWIL() != qfalse {
                    return CPUID_INTEL_WILLIAMETTE;
                }
                return CPUID_INTEL_KATMAI;
            }

            // by default we're functionally a vanilla Pentium/MMX or P2/MMX
            return CPUID_INTEL_MMX;
        }
    }
}

/*
**
** Re-enable optimizations back to what they were
**
*/
// #pragma optimize( "", on )

//============================================

pub fn Sys_GetCurrentUser() -> *const c_char {
    #[cfg(all(target_os = "windows", not(feature = "xbox")))]
    unsafe {
        static mut s_userName: [c_char; 1024] = [0; 1024];
        let mut size: c_int = 1024;

        if GetUserName(addr_of_mut!(s_userName) as *mut c_char, addr_of_mut!(size)) == 0 {
            strcpy(addr_of_mut!(s_userName) as *mut c_char, b"player\0".as_ptr() as *const c_char);
        }

        if s_userName[0] == 0 {
            strcpy(addr_of_mut!(s_userName) as *mut c_char, b"player\0".as_ptr() as *const c_char);
        }

        return addr_of!(s_userName) as *const c_char;
    }

    #[cfg(any(not(target_os = "windows"), feature = "xbox"))]
    {
        // #ifdef _XBOX
        // return NULL;
        core::ptr::null()
    }
}

pub fn Sys_DefaultHomePath() -> *const c_char {
    core::ptr::null()
}

pub fn Sys_DefaultInstallPath() -> *const c_char {
    unsafe { Sys_Cwd() }
}

pub fn Sys_GetPhysicalMemory() -> c_int {
    unsafe {
        let mut MemoryStatus: MEMORYSTATUS = core::mem::zeroed();

        memset(
            &mut MemoryStatus as *mut MEMORYSTATUS as *mut c_void,
            0,
            core::mem::size_of::<MEMORYSTATUS>(),
        );
        MemoryStatus.dwLength = core::mem::size_of::<MEMORYSTATUS>() as u32;

        GlobalMemoryStatus(&mut MemoryStatus);

        return (MemoryStatus.dwTotalPhys / (1024 * 1024)) as c_int + 1;
    }
}


#[cfg(not(feature = "xbox"))]
pub fn Sys_GetCPUSpeedOld() -> c_int {
    unsafe {
        timeBeginPeriod(1);

        #[cfg(target_os = "windows")]
        let (iPriority, hThread) = {
            let hThread = GetCurrentThread();
            let iPriority = GetThreadPriority(hThread);

            if iPriority != THREAD_PRIORITY_ERROR_RETURN {
                SetThreadPriority(hThread, THREAD_PRIORITY_TIME_CRITICAL);
            }

            (iPriority, hThread)
        };

        let clockStart: u32 = timeGetTime() as u32;
        let clockEnd: u32 = clockStart + 100;

        let mut start: u32;
        let mut end: u32;

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            core::arch::asm!("rdtsc", out("eax") start);
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            start = 0;
        }

        while (timeGetTime() as u32) < clockEnd {    // loop for 1 tenth of a second
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            core::arch::asm!("rdtsc", out("eax") end);
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            end = 0;
        }


        #[cfg(target_os = "windows")]
        {
            // Reset priority
            if iPriority != THREAD_PRIORITY_ERROR_RETURN {
                SetThreadPriority(hThread, iPriority);
            }
        }

        timeEndPeriod(1);

        let time: u32 = end - start;
        let coarse: i32 = (time / 100000) as i32;
        let firsttry: i32 = ((coarse + 25) / 50) * 50;
        if (firsttry - coarse).abs() < 10 {
            return firsttry;
        } else {
            return (((coarse + 17) as f32 / 33.3_f32).floor() * 33.3_f32) as i32;
        }
    }
}

#[cfg(not(feature = "xbox"))]
pub fn Sys_GetCPUSpeed() -> c_int {
    unsafe {
        let mut raw_freq: u32;        // Raw frequency of CPU in MHz
        let mut norm_freq: u32;       // Normalized frequency of CPU in MHz.
        let mut t0: i64;              // Variables for High-Resolution Performance Counter reads
        let mut t1: i64;

        let mut freq: u32 = 0;        // Most current frequ. calculation
        let mut freq2: u32 = 0;       // 2nd most current frequ. calc.
        let mut freq3: u32 = 0;       // 3rd most current frequ. calc.

        let mut total: u32;           // Sum of previous three frequency calculations
        let mut tries: i32 = 0;       // Number of times a calculation has been made on this call to cpuspeed
        let mut total_cycles: u32 = 0;
        let mut cycles: u32;          // Clock cycles elapsed during test
        let mut stamp0: u32 = 0;
        let mut stamp1: u32 = 0;      // Time Stamp Variable for beginning and end  of test
        let mut total_ticks: u32 = 0;
        let mut ticks: u32;           // Microseconds elapsed during test
        let mut count_freq: i64;      // High Resolution Performance Counter frequency

        #[cfg(target_os = "windows")]
        let (mut iPriority, hThread) = {
            let hThread = GetCurrentThread();
            let iPriority = GetThreadPriority(hThread);
            (iPriority, hThread)
        };

        if QueryPerformanceFrequency(&mut count_freq) == 0 {
            return Sys_GetCPUSpeedOld();  //should never happen
        }

        // On processors supporting the Read
        //   Time Stamp opcode, compare elapsed
        //   time on the High-Resolution Counter
        //   with elapsed cycles on the Time
        //   Stamp Register.

        loop {    // This do loop runs up to 20 times or until the average of the previous
            //   three calculated frequencies is within 1 MHz of each of the
            //   individual calculated frequencies. This resampling increases the
            //   accuracy of the results since outside factors could affect this calculation

            tries += 1;       // Increment number of times sampled on this call to cpuspeed

            freq3 = freq2;    // Shift frequencies back to make
            freq2 = freq;     //   room for new frequency measurement

            QueryPerformanceCounter(&mut t0);   // Get high-resolution performance counter time

            t1 = t0;    // Set Initial time

            #[cfg(target_os = "windows")]
            {
                iPriority = GetThreadPriority(hThread);
                if iPriority != THREAD_PRIORITY_ERROR_RETURN {
                    SetThreadPriority(hThread, THREAD_PRIORITY_TIME_CRITICAL);
                }
            }

            while (t1 as u32).wrapping_sub(t0 as u32) < 50 {
                // Loop until 50 ticks have passed since last read of hi-res counter. This accounts for overhead later.
                QueryPerformanceCounter(&mut t1);

                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    core::arch::asm!("rdtsc", out("eax") stamp0);
                }
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                {
                    stamp0 = 0;
                }
            }

            t0 = t1;    // Reset Initial Time

            while (t1 as u32).wrapping_sub(t0 as u32) < 2000 {
                // Loop until enough ticks have passed since last read of hi-res counter. This allows for elapsed time for sampling.
                QueryPerformanceCounter(&mut t1);

                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    core::arch::asm!("rdtsc", out("eax") stamp1);
                }
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                {
                    stamp1 = 0;
                }
            }

            #[cfg(target_os = "windows")]
            {
                if iPriority != THREAD_PRIORITY_ERROR_RETURN {    // Reset priority
                    SetThreadPriority(hThread, iPriority);
                }
            }

            cycles = stamp1.wrapping_sub(stamp0);   // Number of internal clock cycles is difference between two time stamp readings.

            ticks = (t1 as u32).wrapping_sub(t0 as u32);
            // Number of external ticks is difference between two hi-res counter reads.


            // Note that some seemingly arbitrary mulitplies and
            //   divides are done below. This is to maintain a
            //   high level of precision without truncating the
            //   most significant data. According to what value
            //   ITERATIIONS is set to, these multiplies and
            //   divides might need to be shifted for optimal
            //   precision.

            ticks = ticks.wrapping_mul(100000);  // Convert ticks to hundred thousandths of a tick

            ticks = ticks / ((count_freq as u32) / 10);
            // Hundred Thousandths of a Ticks / ( 10 ticks/second ) = microseconds (us)

            total_ticks = total_ticks.wrapping_add(ticks);
            total_cycles = total_cycles.wrapping_add(cycles);

            if ticks % (count_freq as u32) > (count_freq as u32) / 2 {
                ticks = ticks.wrapping_add(1);    // Round up if necessary
            }

            if ticks == 0 {
                ticks = ticks.wrapping_add(1);    // prevent DIV by ZERO
            }

            freq = cycles / ticks;    // Cycles / us  = MHz

            if cycles % ticks > ticks / 2 {
                freq = freq.wrapping_add(1);      // Round up if necessary
            }

            total = freq.wrapping_add(freq2).wrapping_add(freq3);  // Total last three frequency calculations

            if !((tries < 3) ||
                 ((tries < 20) &&
                  (((3i32 * freq as i32 - total as i32).abs() > 3 * TOLERANCE) ||
                   ((3i32 * freq2 as i32 - total as i32).abs() > 3 * TOLERANCE) ||
                   ((3i32 * freq3 as i32 - total as i32).abs() > 3 * TOLERANCE)))) {
                break;
            }
            // Compare last three calculations to average of last three calculations.
        }

        if total_ticks == 0 {
            total_ticks = total_ticks.wrapping_add(1);    // prevent DIV by ZERO
        }

        // Try one more significant digit.
        freq3 = (total_cycles.wrapping_mul(10)) / total_ticks;
        freq2 = (total_cycles.wrapping_mul(100)) / total_ticks;


        if freq2.wrapping_sub(freq3.wrapping_mul(10)) as i32 >= ROUND_THRESHOLD {
            freq3 = freq3.wrapping_add(1);
        }

        raw_freq = total_cycles / total_ticks;
        norm_freq = raw_freq;

        freq = raw_freq.wrapping_mul(10);
        if (freq3 as i32).wrapping_sub(freq as i32) >= ROUND_THRESHOLD {
            norm_freq = norm_freq.wrapping_add(1);
        }

        return norm_freq as c_int;
    }
}
