#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables
)]

use core::ffi::{c_char, c_float, c_int, c_uint, c_ulong, c_void};
use core::ptr::{addr_of_mut, null_mut};

use crate::ffi::types::{qboolean, QFALSE, QTRUE};

type uid_t = c_uint;

const CVAR_ARCHIVE: c_int = 0x00000001;

const O_RDWR: c_int = 2;
const PROT_WRITE: c_int = 0x2;
const MAP_SHARED: c_int = 0x01;
const MAP_FILE: c_int = 0;

const AFMT_U8: c_int = 0x00000008;
const AFMT_S16_LE: c_int = 0x00000010;
const DSP_CAP_TRIGGER: c_int = 0x00001000;
const DSP_CAP_MMAP: c_int = 0x00002000;
const PCM_ENABLE_OUTPUT: c_int = 0x00000002;

const IOC_NRBITS: c_ulong = 8;
const IOC_TYPEBITS: c_ulong = 8;
const IOC_SIZEBITS: c_ulong = 14;
const IOC_DIRBITS: c_ulong = 2;

const IOC_NRSHIFT: c_ulong = 0;
const IOC_TYPESHIFT: c_ulong = IOC_NRSHIFT + IOC_NRBITS;
const IOC_SIZESHIFT: c_ulong = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT: c_ulong = IOC_SIZESHIFT + IOC_SIZEBITS;

const IOC_WRITE: c_ulong = 1;
const IOC_READ: c_ulong = 2;

const fn IOC(dir: c_ulong, type_: c_ulong, nr: c_ulong, size: c_ulong) -> c_ulong {
    (dir << IOC_DIRSHIFT) | (type_ << IOC_TYPESHIFT) | (nr << IOC_NRSHIFT) | (size << IOC_SIZESHIFT)
}

const fn IOR<T>(type_: c_ulong, nr: c_ulong) -> c_ulong {
    IOC(IOC_READ, type_, nr, core::mem::size_of::<T>() as c_ulong)
}

const fn IOW<T>(type_: c_ulong, nr: c_ulong) -> c_ulong {
    IOC(IOC_WRITE, type_, nr, core::mem::size_of::<T>() as c_ulong)
}

const fn IOWR<T>(type_: c_ulong, nr: c_ulong) -> c_ulong {
    IOC(IOC_READ | IOC_WRITE, type_, nr, core::mem::size_of::<T>() as c_ulong)
}

const SNDCTL_DSP_SPEED: c_ulong = IOWR::<c_int>(b'P' as c_ulong, 2);
const SNDCTL_DSP_STEREO: c_ulong = IOWR::<c_int>(b'P' as c_ulong, 3);
const SNDCTL_DSP_SETFMT: c_ulong = IOWR::<c_int>(b'P' as c_ulong, 5);
const SNDCTL_DSP_GETFMTS: c_ulong = IOR::<c_int>(b'P' as c_ulong, 11);
const SNDCTL_DSP_GETOSPACE: c_ulong = IOR::<audio_buf_info>(b'P' as c_ulong, 12);
const SNDCTL_DSP_GETCAPS: c_ulong = IOR::<c_int>(b'P' as c_ulong, 15);
const SNDCTL_DSP_SETTRIGGER: c_ulong = IOW::<c_int>(b'P' as c_ulong, 16);
const SNDCTL_DSP_GETOPTR: c_ulong = IOR::<count_info>(b'P' as c_ulong, 18);

#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

#[repr(C)]
pub struct dma_t {
    pub channels: c_int,
    pub samples: c_int,
    pub submission_chunk: c_int,
    pub samplebits: c_int,
    pub speed: c_int,
    pub buffer: *mut u8,
}

#[repr(C)]
struct audio_buf_info {
    fragments: c_int,
    fragstotal: c_int,
    fragsize: c_int,
    bytes: c_int,
}

#[repr(C)]
struct count_info {
    bytes: c_int,
    blocks: c_int,
    ptr: c_int,
}

unsafe extern "C" {
    static mut saved_euid: uid_t;
    static mut dma: dma_t;

    fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Com_Printf(fmt: *const c_char, ...);

    fn close(fd: c_int) -> c_int;
    fn geteuid() -> uid_t;
    fn getuid() -> uid_t;
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    fn mmap(
        addr: *mut c_void,
        length: usize,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: isize,
    ) -> *mut c_void;
    fn open(path: *const c_char, oflag: c_int, ...) -> c_int;
    fn perror(s: *const c_char);
    fn seteuid(euid: uid_t) -> c_int;
}

#[no_mangle]
pub static mut audio_fd: c_int = 0;
#[no_mangle]
pub static mut snd_inited: c_int = 0;

#[no_mangle]
pub static mut sndbits: *mut cvar_t = null_mut();
#[no_mangle]
pub static mut sndspeed: *mut cvar_t = null_mut();
#[no_mangle]
pub static mut sndchannels: *mut cvar_t = null_mut();

#[no_mangle]
pub static mut snddevice: *mut cvar_t = null_mut();

/* Some devices may work only with 48000 */
static mut tryrates: [c_int; 5] = [22050, 11025, 44100, 48000, 8000];

#[no_mangle]
pub unsafe extern "C" fn SNDDMA_Init() -> qboolean {
    let mut rc: c_int;
    let mut fmt: c_int = 0;
    let mut tmp: c_int;
    let mut i: usize;
    // char *s; // bk001204 - unused
    let mut info: audio_buf_info = core::mem::zeroed();
    let mut caps: c_int = 0;

    if snd_inited != 0 {
        return QTRUE;
    }

    if snddevice.is_null() {
        sndbits = Cvar_Get(c"sndbits".as_ptr(), c"16".as_ptr(), CVAR_ARCHIVE);
        sndspeed = Cvar_Get(c"sndspeed".as_ptr(), c"0".as_ptr(), CVAR_ARCHIVE);
        sndchannels = Cvar_Get(c"sndchannels".as_ptr(), c"2".as_ptr(), CVAR_ARCHIVE);
        snddevice = Cvar_Get(c"snddevice".as_ptr(), c"/dev/dsp".as_ptr(), CVAR_ARCHIVE);
    }

    // open /dev/dsp, confirm capability to mmap, and get size of dma buffer
    if audio_fd == 0 {
        seteuid(saved_euid);

        audio_fd = open((*snddevice).string, O_RDWR);

        seteuid(getuid());

        if audio_fd < 0 {
            perror((*snddevice).string);
            Com_Printf(c"Could not open %s\n".as_ptr(), (*snddevice).string);
            return QFALSE;
        }
    }

    if ioctl(audio_fd, SNDCTL_DSP_GETCAPS, addr_of_mut!(caps)) == -1 {
        perror((*snddevice).string);
        Com_Printf(c"Sound driver too old\n".as_ptr());
        close(audio_fd);
        return QFALSE;
    }

    if (caps & DSP_CAP_TRIGGER) == 0 || (caps & DSP_CAP_MMAP) == 0 {
        Com_Printf(c"Sorry but your soundcard can't do this\n".as_ptr());
        close(audio_fd);
        return QFALSE;
    }

    /* SNDCTL_DSP_GETOSPACE moved to be called later */

    // set sample bits & speed
    dma.samplebits = (*sndbits).value as c_int;
    if dma.samplebits != 16 && dma.samplebits != 8 {
        ioctl(audio_fd, SNDCTL_DSP_GETFMTS, addr_of_mut!(fmt));
        if (fmt & AFMT_S16_LE) != 0 {
            dma.samplebits = 16;
        } else if (fmt & AFMT_U8) != 0 {
            dma.samplebits = 8;
        }
    }

    dma.speed = (*sndspeed).value as c_int;
    if dma.speed == 0 {
        i = 0;
        while i < core::mem::size_of::<[c_int; 5]>() / 4 {
            if ioctl(audio_fd, SNDCTL_DSP_SPEED, addr_of_mut!(tryrates).cast::<c_int>().add(i)) == 0 {
                break;
            }
            i += 1;
        }
        dma.speed = *addr_of_mut!(tryrates).cast::<c_int>().add(i);
    }

    dma.channels = (*sndchannels).value as c_int;
    if dma.channels < 1 || dma.channels > 2 {
        dma.channels = 2;
    }

    /*  mmap() call moved forward */

    tmp = 0;
    if dma.channels == 2 {
        tmp = 1;
    }
    rc = ioctl(audio_fd, SNDCTL_DSP_STEREO, addr_of_mut!(tmp));
    if rc < 0 {
        perror((*snddevice).string);
        Com_Printf(c"Could not set %s to stereo=%d".as_ptr(), (*snddevice).string, dma.channels);
        close(audio_fd);
        return QFALSE;
    }

    if tmp != 0 {
        dma.channels = 2;
    } else {
        dma.channels = 1;
    }

    rc = ioctl(audio_fd, SNDCTL_DSP_SPEED, addr_of_mut!(dma.speed));
    if rc < 0 {
        perror((*snddevice).string);
        Com_Printf(c"Could not set %s speed to %d".as_ptr(), (*snddevice).string, dma.speed);
        close(audio_fd);
        return QFALSE;
    }

    if dma.samplebits == 16 {
        rc = AFMT_S16_LE;
        rc = ioctl(audio_fd, SNDCTL_DSP_SETFMT, addr_of_mut!(rc));
        if rc < 0 {
            perror((*snddevice).string);
            Com_Printf(c"Could not support 16-bit data.  Try 8-bit.\n".as_ptr());
            close(audio_fd);
            return QFALSE;
        }
    } else if dma.samplebits == 8 {
        rc = AFMT_U8;
        rc = ioctl(audio_fd, SNDCTL_DSP_SETFMT, addr_of_mut!(rc));
        if rc < 0 {
            perror((*snddevice).string);
            Com_Printf(c"Could not support 8-bit data.\n".as_ptr());
            close(audio_fd);
            return QFALSE;
        }
    } else {
        perror((*snddevice).string);
        Com_Printf(c"%d-bit sound not supported.".as_ptr(), dma.samplebits);
        close(audio_fd);
        return QFALSE;
    }

    if ioctl(audio_fd, SNDCTL_DSP_GETOSPACE, addr_of_mut!(info)) == -1 {
        perror(c"GETOSPACE".as_ptr());
        Com_Printf(c"Um, can't do GETOSPACE?\n".as_ptr());
        close(audio_fd);
        return QFALSE;
    }

    dma.samples = info.fragstotal * info.fragsize / (dma.samplebits / 8);
    dma.submission_chunk = 1;

    // memory map the dma buffer

    if dma.buffer.is_null() {
        dma.buffer = mmap(
            null_mut(),
            (info.fragstotal * info.fragsize) as usize,
            PROT_WRITE,
            MAP_FILE | MAP_SHARED,
            audio_fd,
            0,
        ) as *mut u8;
    }

    if dma.buffer.is_null() {
        perror((*snddevice).string);
        Com_Printf(c"Could not mmap %s\n".as_ptr(), (*snddevice).string);
        close(audio_fd);
        return QFALSE;
    }

    // toggle the trigger & start her up

    tmp = 0;
    rc = ioctl(audio_fd, SNDCTL_DSP_SETTRIGGER, addr_of_mut!(tmp));
    if rc < 0 {
        perror((*snddevice).string);
        Com_Printf(c"Could not toggle.\n".as_ptr());
        close(audio_fd);
        return QFALSE;
    }

    tmp = PCM_ENABLE_OUTPUT;
    rc = ioctl(audio_fd, SNDCTL_DSP_SETTRIGGER, addr_of_mut!(tmp));
    if rc < 0 {
        perror((*snddevice).string);
        Com_Printf(c"Could not toggle.\n".as_ptr());
        close(audio_fd);

        return QFALSE;
    }

    snd_inited = 1;
    QTRUE
}

#[no_mangle]
pub unsafe extern "C" fn SNDDMA_GetDMAPos() -> c_int {
    let mut count: count_info = core::mem::zeroed();

    if snd_inited == 0 {
        return 0;
    }

    if ioctl(audio_fd, SNDCTL_DSP_GETOPTR, addr_of_mut!(count)) == -1 {
        perror((*snddevice).string);
        Com_Printf(c"Uh, sound dead.\n".as_ptr());
        close(audio_fd);
        snd_inited = 0;
        return 0;
    }
    count.ptr / (dma.samplebits / 8)
}

#[no_mangle]
pub extern "C" fn SNDDMA_Shutdown() {}

/*
==============
SNDDMA_Submit

Send sound to device if buffer isn't really the dma buffer
===============
*/
#[no_mangle]
pub extern "C" fn SNDDMA_Submit() {}

#[no_mangle]
pub extern "C" fn SNDDMA_BeginPainting() {}
