/*
** linux_joystick.c
**
** This file contains ALL Linux specific stuff having to do with the
** Joystick input.  When a port is being made the following functions
** must be implemented by the port:
**
** Authors: mkv, bk
**
*/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_assignments)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::ffi::types::{qboolean, QFALSE, QTRUE};

const PATH_MAX: usize = 4096;
const O_RDONLY: c_int = 0;
const O_NONBLOCK: c_int = 0x00000800;

const JS_EVENT_BUTTON: u8 = 0x01;
const JS_EVENT_AXIS: u8 = 0x02;
const JS_EVENT_INIT: u8 = 0x80;

const JSIOCGAXES: c_uint = 0x80016a11;
const JSIOCGBUTTONS: c_uint = 0x80016a12;

const fn JSIOCGNAME(len: usize) -> c_uint {
    0x80006a13 | ((len as c_uint) << 16)
}

pub type sysEventType_t = c_int;
pub const SE_KEY: sysEventType_t = 1;

pub const K_LEFTARROW: c_int = 178;
pub const K_RIGHTARROW: c_int = 179;
pub const K_UPARROW: c_int = 176;
pub const K_DOWNARROW: c_int = 177;
pub const K_JOY1: c_int = 266;
pub const K_JOY16: c_int = 281;
pub const K_JOY17: c_int = 282;
pub const K_JOY18: c_int = 283;
pub const K_JOY19: c_int = 284;
pub const K_JOY20: c_int = 285;
pub const K_JOY21: c_int = 286;
pub const K_JOY22: c_int = 287;
pub const K_JOY23: c_int = 288;
pub const K_JOY24: c_int = 289;
pub const K_JOY25: c_int = 290;
pub const K_JOY26: c_int = 291;
pub const K_JOY27: c_int = 292;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct js_event {
    time: c_uint,
    value: i16,
    r#type: u8,
    number: u8,
}

unsafe extern "C" {
    fn open(pathname: *const c_char, flags: c_int, ...) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn ioctl(fd: c_int, request: c_uint, ...) -> c_int;
    fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;

    pub static mut in_joystick: *mut cvar_t;
    pub static mut in_joystickDebug: *mut cvar_t;
    pub static mut joy_threshold: *mut cvar_t;

    fn Com_Printf(fmt: *const c_char, ...);
    fn Sys_QueEvent(
        time: c_int,
        type_: sysEventType_t,
        value: c_int,
        value2: c_int,
        ptrLength: c_int,
        ptr: *mut c_void,
    );
}

/* We translate axes movement into keypresses. */
#[no_mangle]
pub static mut joy_keys: [c_int; 16] = [
    K_LEFTARROW,
    K_RIGHTARROW,
    K_UPARROW,
    K_DOWNARROW,
    K_JOY16,
    K_JOY17,
    K_JOY18,
    K_JOY19,
    K_JOY20,
    K_JOY21,
    K_JOY22,
    K_JOY23,
    K_JOY24,
    K_JOY25,
    K_JOY26,
    K_JOY27,
];

/* Our file descriptor for the joystick device. */
static mut joy_fd: c_int = -1;

// bk001130 - from linux_glimp.c
// extern cvar_t *  in_joystick;
// extern cvar_t *  in_joystickDebug;
// extern cvar_t *  joy_threshold;

/**********************************************/
/* Joystick routines.                         */
/**********************************************/
// bk001130 - from cvs1.17 (mkv), removed from linux_glimp.c
#[no_mangle]
pub unsafe extern "C" fn IN_StartupJoystick() {
    let mut i: c_int = 0;

    joy_fd = -1;

    if (*in_joystick).integer == 0 {
        Com_Printf(c"Joystick is not active.\n".as_ptr());
        return;
    }

    while i < 4 {
        let mut filename: [c_char; PATH_MAX] = [0; PATH_MAX];

        snprintf(
            filename.as_mut_ptr(),
            PATH_MAX,
            c"/dev/js%d".as_ptr(),
            i,
        );

        joy_fd = open(filename.as_ptr(), O_RDONLY | O_NONBLOCK);

        if joy_fd != -1 {
            let mut event: js_event = js_event {
                time: 0,
                value: 0,
                r#type: 0,
                number: 0,
            };
            let mut axes: c_char = 0;
            let mut buttons: c_char = 0;
            let mut name: [c_char; 128] = [0; 128];
            let mut n: c_int = -1;

            Com_Printf(c"Joystick %s found\n".as_ptr(), filename.as_ptr());

            /* Get rid of initialization messages. */
            loop {
                n = read(
                    joy_fd,
                    addr_of_mut!(event).cast::<c_void>(),
                    size_of::<js_event>(),
                ) as c_int;

                if n == -1 {
                    break;
                }

                if (event.r#type & JS_EVENT_INIT) == 0 {
                    break;
                }
            }

            /* Get joystick statistics. */
            ioctl(joy_fd, JSIOCGAXES, addr_of_mut!(axes));
            ioctl(joy_fd, JSIOCGBUTTONS, addr_of_mut!(buttons));

            if ioctl(joy_fd, JSIOCGNAME(size_of::<[c_char; 128]>()), name.as_mut_ptr()) < 0 {
                strncpy(name.as_mut_ptr(), c"Unknown".as_ptr(), size_of::<[c_char; 128]>());
            }

            Com_Printf(c"Name:    %s\n".as_ptr(), name.as_ptr());
            Com_Printf(c"Axes:    %d\n".as_ptr(), axes as c_int);
            Com_Printf(c"Buttons: %d\n".as_ptr(), buttons as c_int);

            /* Our work here is done. */
            return;
        }

        i += 1;
    }

    /* No soup for you. */
    if joy_fd == -1 {
        Com_Printf(c"No joystick found.\n".as_ptr());
        return;
    }
}

#[no_mangle]
pub unsafe extern "C" fn IN_JoyMove() {
    /* Store instantaneous joystick state. Hack to get around
     * event model used in Linux joystick driver.
     */
    static mut axes_state: [c_int; 16] = [0; 16];
    /* Old bits for Quake-style input compares. */
    static mut old_axes: c_uint = 0;
    /* Our current goodies. */
    let mut axes: c_uint = 0;
    let mut i: c_int = 0;

    if joy_fd == -1 {
        return;
    }

    /* Empty the queue, dispatching button presses immediately
     * and updating the instantaneous state for the axes.
     */
    loop {
        let mut n: c_int = -1;
        let mut event: js_event = js_event {
            time: 0,
            value: 0,
            r#type: 0,
            number: 0,
        };

        n = read(
            joy_fd,
            addr_of_mut!(event).cast::<c_void>(),
            size_of::<js_event>(),
        ) as c_int;

        if n == -1 {
            /* No error, we're non-blocking. */
            break;
        }

        if event.r#type & JS_EVENT_BUTTON != 0 {
            Sys_QueEvent(
                0,
                SE_KEY,
                K_JOY1 + event.number as c_int,
                event.value as c_int,
                0,
                null_mut(),
            );
        } else if event.r#type & JS_EVENT_AXIS != 0 {
            if event.number >= 16 {
                continue;
            }

            *addr_of_mut!(axes_state)
                .cast::<c_int>()
                .add(event.number as usize) = event.value as c_int;
        } else {
            Com_Printf(c"Unknown joystick event type\n".as_ptr());
        }
    }

    /* Translate our instantaneous state to bits. */
    while i < 16 {
        let f: f32 = (*addr_of!(axes_state).cast::<c_int>().add(i as usize) as f32) / 32767.0f32;

        if f < -(*joy_threshold).value {
            axes |= 1 << (i * 2);
        } else if f > (*joy_threshold).value {
            axes |= 1 << ((i * 2) + 1);
        }

        i += 1;
    }

    /* Time to update axes state based on old vs. new. */
    i = 0;
    while i < 16 {
        if (axes & (1 << i)) != 0 && (old_axes & (1 << i)) == 0 {
            Sys_QueEvent(
                0,
                SE_KEY,
                *addr_of!(joy_keys).cast::<c_int>().add(i as usize),
                QTRUE,
                0,
                null_mut(),
            );
        }

        if (axes & (1 << i)) == 0 && (old_axes & (1 << i)) != 0 {
            Sys_QueEvent(
                0,
                SE_KEY,
                *addr_of!(joy_keys).cast::<c_int>().add(i as usize),
                QFALSE,
                0,
                null_mut(),
            );
        }

        i += 1;
    }

    /* Save for future generations. */
    old_axes = axes;
}
