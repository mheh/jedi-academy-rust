// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"
// #include "../game/q_shared.h"
// #include "../qcommon/qcommon.h"
// #include "win_local.h"

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_ulong};
use core::ptr::{addr_of, addr_of_mut};

// External Windows API and standard library functions
extern "C" {
    fn timeGetTime() -> c_uint;
    fn GetUserName(lpBuffer: *mut c_char, nSize: *mut c_ulong) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

// Constants from included headers (win_local.h, qcommon.h)
// Processor IDs (from win_local.h)
#[allow(dead_code)]
mod constants {
    use core::ffi::c_int;
    pub const CPUID_GENERIC: c_int = 0;
    pub const CPUID_INTEL_UNSUPPORTED: c_int = 1;
    pub const CPUID_INTEL_PENTIUM: c_int = 2;
    pub const CPUID_INTEL_MMX: c_int = 3;
    pub const CPUID_AMD_3DNOW: c_int = 4;
    pub const CPUID_INTEL_KATMAI: c_int = 5;
    pub const CPUID_INTEL_WILLIAMETTE: c_int = 6;
    pub const CPUID_AXP: c_int = 7;

    // Game constants (from q_shared.h)
    pub const qtrue: c_int = 1;
    pub const qfalse: c_int = 0;
}

use constants::*;

/*
================
Sys_Milliseconds
================
*/
pub fn Sys_Milliseconds() -> c_int {
    static mut sys_timeBase: c_uint = 0;
    let mut sys_curtime: c_int;

    unsafe {
        if sys_timeBase == 0 {
            sys_timeBase = timeGetTime();
        }
        sys_curtime = (timeGetTime().wrapping_sub(sys_timeBase)) as c_int;
    }

    sys_curtime
}

/*
** --------------------------------------------------------------------------------
**
** PROCESSOR STUFF
**
** --------------------------------------------------------------------------------
*/

#[inline]
unsafe fn CPUID(func: c_int, regs: *mut [c_uint; 4]) {
    #[cfg(target_arch = "x86")]
    {
        use core::arch::x86::*;

        let mut regEAX: c_uint;
        let mut regEBX: c_uint;
        let mut regECX: c_uint;
        let mut regEDX: c_uint;

        core::arch::asm!(
            "mov eax, {0:e}",
            "cpuid",
            "mov {1:e}, eax",
            "mov {2:e}, ebx",
            "mov {3:e}, ecx",
            "mov {4:e}, edx",
            in(reg) func,
            out(reg) regEAX,
            out(reg) regEBX,
            out(reg) regECX,
            out(reg) regEDX,
            options(nomem, nostack)
        );

        (*regs)[0] = regEAX;
        (*regs)[1] = regEBX;
        (*regs)[2] = regECX;
        (*regs)[3] = regEDX;
    }

    #[cfg(target_arch = "x86_64")]
    {
        use core::arch::x86_64::*;

        let mut regEAX: c_uint;
        let mut regEBX: c_uint;
        let mut regECX: c_uint;
        let mut regEDX: c_uint;

        core::arch::asm!(
            "mov eax, {0:e}",
            "cpuid",
            "mov {1:e}, eax",
            "mov {2:e}, ebx",
            "mov {3:e}, ecx",
            "mov {4:e}, edx",
            in(reg) func,
            out(reg) regEAX,
            out(reg) regEBX,
            out(reg) regECX,
            out(reg) regEDX,
            options(nomem, nostack)
        );

        (*regs)[0] = regEAX;
        (*regs)[1] = regEBX;
        (*regs)[2] = regECX;
        (*regs)[3] = regEDX;
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
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // Test whether CPUID instruction is supported by checking if bit 21 (ID bit) of EFLAGS can be toggled
        // pushfd - save eflags
        // pop eax - eflags into eax
        // test eax, 0x00200000 - check ID bit
        // jz set21 - if not set, jump to set21
        // and eax, 0xffdfffff - clear bit 21
        // push eax - save modified flags
        // popfd - restore to EFLAGS
        // pushfd - save current EFLAGS
        // pop eax - check if bit was cleared
        // test eax, 0x00200000 - check ID bit
        // jz good - if clear, CPUID not supported
        // jmp err - bit wasn't cleared, error
        // set21: or eax, 0x00200000 - set bit 21
        // ... similar test sequence
        // If bit can be toggled, CPU is Pentium or later with CPUID; otherwise error

        let mut flags: u32;
        let result: c_int;

        core::arch::asm!(
            "pushfd",
            "pop eax",
            "mov ecx, eax",
            "xor eax, 0x00200000",
            "push eax",
            "popfd",
            "pushfd",
            "pop eax",
            "xor eax, ecx",
            "shr eax, 21",
            "and eax, 1",
            out("eax") result,
            out("ecx") flags,
            options(nomem, nostack)
        );

        result
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    qfalse
}

unsafe fn Is3DNOW() -> c_int {
    let mut regs: [c_uint; 4] = [0; 4];
    let mut pstring: [c_uint; 4] = [0; 4];
    let mut processorString: [c_char; 13] = [0; 13];

    // get name of processor
    CPUID(0, &mut pstring);
    // Cast pstring (uint array) as char array to extract bytes
    let pstring_bytes = pstring.as_ptr() as *const [c_char; 16];
    let pstring_chars = &*pstring_bytes;

    processorString[0] = pstring_chars[4];
    processorString[1] = pstring_chars[5];
    processorString[2] = pstring_chars[6];
    processorString[3] = pstring_chars[7];
    processorString[4] = pstring_chars[12];
    processorString[5] = pstring_chars[13];
    processorString[6] = pstring_chars[14];
    processorString[7] = pstring_chars[15];
    processorString[8] = pstring_chars[8];
    processorString[9] = pstring_chars[9];
    processorString[10] = pstring_chars[10];
    processorString[11] = pstring_chars[11];
    processorString[12] = 0;

    //  REMOVED because you can have 3DNow! on non-AMD systems
    //	if ( strcmp( processorString, "AuthenticAMD" ) )
    //		return qfalse;

    // check AMD-specific functions
    CPUID(0x80000000 as c_int, &mut regs);
    if regs[0] < 0x80000000 {
        return qfalse;
    }

    // bit 31 of EDX denotes 3DNOW! support
    CPUID(0x80000001 as c_int, &mut regs);
    if (regs[3] & (1 << 31)) != 0 {
        return qtrue;
    }

    qfalse
}

unsafe fn IsKNI() -> c_int {
    let mut regs: [c_uint; 4] = [0; 4];

    // get CPU feature bits
    CPUID(1, &mut regs);

    // bit 25 of EDX denotes KNI existence
    if (regs[3] & (1 << 25)) != 0 {
        // Ok, CPU supports this instruction, but does the OS?
        //
        // Test a KNI instruction and make sure you don't get an exception...
        //
        // __try
        // {
        // 	pushad
        // 	orps xmm1,xmm1
        // 	popad
        // }
        // __except(EXCEPTION_EXECUTE_HANDLER)
        // {
        // 	return qfalse
        // }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // Attempt to execute orps xmm1,xmm1 via inline asm
            // If this doesn't panic/segfault, assume OS supports it
            core::arch::asm!(
                "pushad",
                "orps xmm1, xmm1",
                "popad",
                options(nostack)
            );
            return qtrue;
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        return qfalse;
    }

    qfalse
}

unsafe fn IsWIL() -> c_int {
    let mut regs: [c_uint; 4] = [0; 4];

    // get CPU feature bits
    CPUID(1, &mut regs);

    // bit 26 of EDX denotes WIL existence
    if (regs[3] & (1 << 26)) != 0 {
        // Ok, CPU supports this instruction, but does the OS?
        //
        // Test a WIL instruction and make sure you don't get an exception...
        //
        // __try
        // {
        // 	pushad
        // 	xorpd xmm0,xmm0  (Willamette New Instructions)
        // 	popad
        // }
        // __except(EXCEPTION_EXECUTE_HANDLER)
        // {
        // 	return qfalse  (Willamette New Instructions not supported)
        // }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // Attempt to execute xorpd xmm0,xmm0 (Willamette New Instructions)
            // If this doesn't panic/segfault, assume OS supports it
            core::arch::asm!(
                "pushad",
                "xorpd xmm0, xmm0",
                "popad",
                options(nostack)
            );
            return qtrue;  // Williamette/P4 instructions available
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        return qfalse;
    }

    qfalse
}

unsafe fn IsMMX() -> c_int {
    let mut regs: [c_uint; 4] = [0; 4];

    // get CPU feature bits
    CPUID(1, &mut regs);

    // bit 23 of EDX denotes MMX existence
    if (regs[3] & (1 << 23)) != 0 {
        return qtrue;
    }
    qfalse
}

pub fn Sys_GetProcessorId() -> c_int {
    #[cfg(target_arch = "aarch64")]
    {
        // ARM 64-bit - use generic ID
        return CPUID_GENERIC;
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        // Non-x86 platforms - use generic ID
        return CPUID_GENERIC;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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

//============================================

pub fn Sys_GetCurrentUser() -> *const c_char {
    #[cfg(target_os = "xbox")]
    {
        return core::ptr::null();
    }

    #[cfg(not(target_os = "xbox"))]
    unsafe {
        static mut s_userName: [c_char; 1024] = [0; 1024];
        let mut size: c_ulong = core::mem::size_of_val(&s_userName) as c_ulong;

        if GetUserName(
            addr_of_mut!(s_userName) as *mut c_char,
            &mut size,
        ) == 0
        {
            strcpy(
                addr_of_mut!(s_userName) as *mut c_char,
                b"player\0".as_ptr() as *const c_char,
            );
        }

        if *addr_of!(s_userName) == 0 {
            strcpy(
                addr_of_mut!(s_userName) as *mut c_char,
                b"player\0".as_ptr() as *const c_char,
            );
        }

        addr_of!(s_userName) as *const c_char
    }
}
