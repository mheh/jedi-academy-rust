#[repr(C)]
pub struct timing_c {
    start: i64,
    end: i64,

    reset: i32,
}

impl timing_c {
    pub fn new() -> Self {
        timing_c {
            start: 0,
            end: 0,
            reset: 0,
        }
    }

    pub fn Start(&mut self) {
        unsafe {
            use core::arch::asm;

            let s = &mut self.start as *mut i64;

            asm!(
                "rdtsc",
                "mov [{}], eax",
                "mov [{}+4], edx",
                in(reg) s,
                out("eax") _,
                out("edx") _,
            );
        }
    }

    pub fn End(&mut self) -> i32 {
        unsafe {
            use core::arch::asm;

            let e = &mut self.end as *mut i64;

            asm!(
                "rdtsc",
                "mov [{}], eax",
                "mov [{}+4], edx",
                in(reg) e,
                out("eax") _,
                out("edx") _,
            );
        }

        let time = self.end - self.start;
        if time < 0 {
            0 as i32
        } else {
            time as i32
        }
    }
}

// end
