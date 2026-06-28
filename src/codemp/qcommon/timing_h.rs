#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::c_int;

// C origin: `codemp/qcommon/timing.h`.

#[repr(C)]
pub struct timing_c {
    start: i64,
    end: i64,

    reset: c_int,
}

impl timing_c {
    pub fn Start(&mut self) {
        self.start = rdtsc_i64();
    }

    pub fn End(&mut self) -> c_int {
        #[cfg(not(target_os = "linux"))]
        {
            self.end = rdtsc_i64();
        }

        let mut time = self.end.wrapping_sub(self.start);
        if time < 0 {
            time = 0;
        }
        time as c_int
    }
}

#[inline]
fn rdtsc_i64() -> i64 {
    #[cfg(target_arch = "x86")]
    {
        // SAFETY: `_rdtsc` mirrors the original inline assembly `rdtsc` instruction.
        unsafe { core::arch::x86::_rdtsc() as i64 }
    }

    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: `_rdtsc` mirrors the original inline assembly `rdtsc` instruction.
        unsafe { core::arch::x86_64::_rdtsc() as i64 }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        0
    }
}
