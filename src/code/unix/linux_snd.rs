#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::ptr::{self, addr_of, addr_of_mut};

// ioctl constants from linux/soundcard.h
const SNDCTL_DSP_RESET: c_int = 0x00005000;
const SNDCTL_DSP_GETCAPS: c_int = 0x0000500f;
const SNDCTL_DSP_GETFMTS: c_int = 0x0000500b;
const SNDCTL_DSP_STEREO: c_int = 0x00005003;
const SNDCTL_DSP_SPEED: c_int = 0x00005002;
const SNDCTL_DSP_SETFMT: c_int = 0x00005005;
const SNDCTL_DSP_GETOSPACE: c_int = 0x00001801;
const SNDCTL_DSP_GETOPTR: c_int = 0x00001800;
const SNDCTL_DSP_SETTRIGGER: c_int = 0x00005007;

const AFMT_S16_LE: c_int = 0x00000010;
const AFMT_U8: c_int = 0x00000008;

const DSP_CAP_TRIGGER: c_int = 0x00000100;
const DSP_CAP_MMAP: c_int = 0x00010000;

const PCM_ENABLE_OUTPUT: c_int = 0x00000001;

// Open flags from fcntl.h
const O_RDWR: c_int = 0x00000002;

// mmap flags
const PROT_WRITE: c_int = 0x00000002;
const MAP_FILE: c_int = 0x00000000;
const MAP_SHARED: c_int = 0x00000001;

// Cvar flags from q_shared.h / cvar.h
const CVAR_ARCHIVE: c_int = 0x00000040;

// Stub types from snd_local.h with repr(C) for ABI compatibility
#[repr(C)]
pub struct dma_t {
    pub samplebits: c_int,
    pub speed: c_int,
    pub channels: c_int,
    pub samples: c_int,
    pub submission_chunk: c_int,
    pub buffer: *mut c_void,
    // Additional fields that might exist in the actual struct
}

#[repr(C)]
pub struct cvar_t {
    pub value: f32,
    pub string: *mut c_char,
    // Additional fields that might exist in the actual struct
}

// Stub types from linux/soundcard.h with repr(C) for ABI compatibility
#[repr(C)]
pub struct audio_buf_info {
    pub fragstotal: c_int,
    pub fragsize: c_int,
    pub bytes: c_int,
    pub fragments: c_int,
}

#[repr(C)]
pub struct count_info {
    pub ptr: c_int,
    pub blocks: c_int,
    pub bytes: c_int,
}

// External functions from libc
extern "C" {
    pub fn open(path: *const c_char, flags: c_int, ...) -> c_int;
    pub fn close(fd: c_int) -> c_int;
    pub fn seteuid(euid: u32) -> c_int;
    pub fn getuid() -> u32;
    pub fn ioctl(fd: c_int, request: c_int, ...) -> c_int;
    pub fn mmap(addr: *mut c_void, len: usize, prot: c_int, flags: c_int, fd: c_int, offset: i64) -> *mut c_void;
    pub fn perror(s: *const c_char);
}

// External game functions
extern "C" {
    pub fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Com_Printf(format: *const c_char, ...);
}

// External global from elsewhere
extern "C" {
    pub static saved_euid: u32;
}

// External DMA buffer from snd_local.h
extern "C" {
    pub static mut dma: dma_t;
}

// Global audio device file descriptor
pub static mut audio_fd: c_int = 0;
pub static mut snd_inited: c_int = 0;

pub static mut sndbits: *mut cvar_t = ptr::null_mut();
pub static mut sndspeed: *mut cvar_t = ptr::null_mut();
pub static mut sndchannels: *mut cvar_t = ptr::null_mut();

pub static mut snddevice: *mut cvar_t = ptr::null_mut();

/* Some devices may work only with 48000 */
static tryrates: [c_int; 5] = [22050, 11025, 44100, 48000, 8000];

pub unsafe fn SNDDMA_Init() -> c_int {
    let mut rc: c_int;
    let mut fmt: c_int;
    let mut tmp: c_int;
    let mut i: usize;
    let mut info: audio_buf_info = audio_buf_info {
        fragstotal: 0,
        fragsize: 0,
        bytes: 0,
        fragments: 0,
    };

    if snd_inited != 0 {
        return 0;
    }

    if sndbits.is_null() {
        sndbits = Cvar_Get(
            b"sndbits\0".as_ptr() as *const c_char,
            b"16\0".as_ptr() as *const c_char,
            CVAR_ARCHIVE,
        );
        sndspeed = Cvar_Get(
            b"sndspeed\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_ARCHIVE,
        );
        sndchannels = Cvar_Get(
            b"sndchannels\0".as_ptr() as *const c_char,
            b"2\0".as_ptr() as *const c_char,
            CVAR_ARCHIVE,
        );
        snddevice = Cvar_Get(
            b"snddevice\0".as_ptr() as *const c_char,
            b"/dev/dsp\0".as_ptr() as *const c_char,
            CVAR_ARCHIVE,
        );
    }

    // open /dev/dsp, confirm capability to mmap, and get size of dma buffer
    if audio_fd == 0 {
        seteuid(saved_euid);

        audio_fd = open((*snddevice).string, O_RDWR);

        seteuid(getuid());

        if audio_fd < 0 {
            perror((*snddevice).string);
            Com_Printf(
                b"Could not open %s\n\0".as_ptr() as *const c_char,
                (*snddevice).string,
            );
            return 0;
        }
    }

    // #if 0
    // /* Not applicable here */
    // rc = ioctl(audio_fd, SNDCTL_DSP_RESET, 0);
    // if (rc < 0) {
    //     perror(snddevice->string);
    //     Com_Printf("Could not reset %s\n", snddevice->string);
    //     close(audio_fd);
    //
    //     return 0;
    // }
    // #endif

    let mut caps: c_int = 0;
    if ioctl(audio_fd, SNDCTL_DSP_GETCAPS, &mut caps as *mut c_int as *mut c_void) == -1 {
        perror((*snddevice).string);
        Com_Printf(b"Sound driver too old\n\0".as_ptr() as *const c_char);
        close(audio_fd);
        return 0;
    }

    if (caps & DSP_CAP_TRIGGER) == 0 || (caps & DSP_CAP_MMAP) == 0 {
        Com_Printf(b"Sorry but your soundcard can\'t do this\n\0".as_ptr() as *const c_char);
        close(audio_fd);
        return 0;
    }

    /* SNDCTL_DSP_GETOSPACE moved to be called later */

    // set sample bits & speed
    (*addr_of_mut!(dma)).samplebits = (*sndbits).value as c_int;
    if (*addr_of_mut!(dma)).samplebits != 16 && (*addr_of_mut!(dma)).samplebits != 8 {
        fmt = 0;
        ioctl(
            audio_fd,
            SNDCTL_DSP_GETFMTS,
            &mut fmt as *mut c_int as *mut c_void,
        );
        if (fmt & AFMT_S16_LE) != 0 {
            (*addr_of_mut!(dma)).samplebits = 16;
        } else if (fmt & AFMT_U8) != 0 {
            (*addr_of_mut!(dma)).samplebits = 8;
        }
    }

    (*addr_of_mut!(dma)).speed = (*sndspeed).value as c_int;
    if (*addr_of_mut!(dma)).speed == 0 {
        i = 0;
        while i < 5 {
            let mut rate: c_int = tryrates[i];
            if ioctl(
                audio_fd,
                SNDCTL_DSP_SPEED,
                &mut rate as *mut c_int as *mut c_void,
            ) == 0
            {
                break;
            }
            i += 1;
        }
        (*addr_of_mut!(dma)).speed = tryrates[i];
    }

    (*addr_of_mut!(dma)).channels = (*sndchannels).value as c_int;
    if (*addr_of_mut!(dma)).channels < 1 || (*addr_of_mut!(dma)).channels > 2 {
        (*addr_of_mut!(dma)).channels = 2;
    }

    /*  mmap() call moved forward */

    tmp = 0;
    if (*addr_of_mut!(dma)).channels == 2 {
        tmp = 1;
    }
    rc = ioctl(
        audio_fd,
        SNDCTL_DSP_STEREO,
        &mut tmp as *mut c_int as *mut c_void,
    );
    if rc < 0 {
        perror((*snddevice).string);
        Com_Printf(
            b"Could not set %s to stereo=%d\0".as_ptr() as *const c_char,
            (*snddevice).string,
            (*addr_of_mut!(dma)).channels,
        );
        close(audio_fd);
        return 0;
    }

    if tmp != 0 {
        (*addr_of_mut!(dma)).channels = 2;
    } else {
        (*addr_of_mut!(dma)).channels = 1;
    }

    rc = ioctl(
        audio_fd,
        SNDCTL_DSP_SPEED,
        &mut (*addr_of_mut!(dma)).speed as *mut c_int as *mut c_void,
    );
    if rc < 0 {
        perror((*snddevice).string);
        Com_Printf(
            b"Could not set %s speed to %d\0".as_ptr() as *const c_char,
            (*snddevice).string,
            (*addr_of_mut!(dma)).speed,
        );
        close(audio_fd);
        return 0;
    }

    if (*addr_of_mut!(dma)).samplebits == 16 {
        rc = AFMT_S16_LE;
        rc = ioctl(
            audio_fd,
            SNDCTL_DSP_SETFMT,
            &mut rc as *mut c_int as *mut c_void,
        );
        if rc < 0 {
            perror((*snddevice).string);
            Com_Printf(b"Could not support 16-bit data.  Try 8-bit.\n\0".as_ptr() as *const c_char);
            close(audio_fd);
            return 0;
        }
    } else if (*addr_of_mut!(dma)).samplebits == 8 {
        rc = AFMT_U8;
        rc = ioctl(
            audio_fd,
            SNDCTL_DSP_SETFMT,
            &mut rc as *mut c_int as *mut c_void,
        );
        if rc < 0 {
            perror((*snddevice).string);
            Com_Printf(b"Could not support 8-bit data.\n\0".as_ptr() as *const c_char);
            close(audio_fd);
            return 0;
        }
    } else {
        perror((*snddevice).string);
        Com_Printf(
            b"%d-bit sound not supported.\0".as_ptr() as *const c_char,
            (*addr_of_mut!(dma)).samplebits,
        );
        close(audio_fd);
        return 0;
    }

    if ioctl(
        audio_fd,
        SNDCTL_DSP_GETOSPACE,
        &mut info as *mut audio_buf_info as *mut c_void,
    ) == -1
    {
        perror(b"GETOSPACE\0".as_ptr() as *const c_char);
        Com_Printf(b"Um, can\'t do GETOSPACE?\n\0".as_ptr() as *const c_char);
        close(audio_fd);
        return 0;
    }

    (*addr_of_mut!(dma)).samples =
        info.fragstotal * info.fragsize / ((*addr_of_mut!(dma)).samplebits / 8);
    (*addr_of_mut!(dma)).submission_chunk = 1;

    // memory map the dma buffer

    if (*addr_of_mut!(dma)).buffer.is_null() {
        (*addr_of_mut!(dma)).buffer = mmap(
            ptr::null_mut(),
            (info.fragstotal * info.fragsize) as usize,
            PROT_WRITE,
            MAP_FILE | MAP_SHARED,
            audio_fd,
            0,
        );
    }

    if (*addr_of_mut!(dma)).buffer.is_null() {
        perror((*snddevice).string);
        Com_Printf(
            b"Could not mmap %s\n\0".as_ptr() as *const c_char,
            (*snddevice).string,
        );
        close(audio_fd);
        return 0;
    }

    // toggle the trigger & start her up

    tmp = 0;
    rc = ioctl(
        audio_fd,
        SNDCTL_DSP_SETTRIGGER,
        &mut tmp as *mut c_int as *mut c_void,
    );
    if rc < 0 {
        perror((*snddevice).string);
        Com_Printf(b"Could not toggle.\n\0".as_ptr() as *const c_char);
        close(audio_fd);
        return 0;
    }

    tmp = PCM_ENABLE_OUTPUT;
    rc = ioctl(
        audio_fd,
        SNDCTL_DSP_SETTRIGGER,
        &mut tmp as *mut c_int as *mut c_void,
    );
    if rc < 0 {
        perror((*snddevice).string);
        Com_Printf(b"Could not toggle.\n\0".as_ptr() as *const c_char);
        close(audio_fd);

        return 0;
    }

    snd_inited = 1;
    return 1;
}

pub unsafe fn SNDDMA_GetDMAPos() -> c_int {
    let mut count: count_info = count_info {
        ptr: 0,
        blocks: 0,
        bytes: 0,
    };

    if snd_inited == 0 {
        return 0;
    }

    if ioctl(
        audio_fd,
        SNDCTL_DSP_GETOPTR,
        &mut count as *mut count_info as *mut c_void,
    ) == -1
    {
        perror((*addr_of!(snddevice)).string);
        Com_Printf(b"Uh, sound dead.\n\0".as_ptr() as *const c_char);
        close(audio_fd);
        snd_inited = 0;
        return 0;
    }
    return count.ptr / ((*addr_of!(dma)).samplebits / 8);
}

pub unsafe fn SNDDMA_Shutdown() {
}

/*
==============
SNDDMA_Submit

Send sound to device if buffer isn't really the dma buffer
===============
*/
pub unsafe fn SNDDMA_Submit() {
}

pub unsafe fn SNDDMA_BeginPainting() {
}
