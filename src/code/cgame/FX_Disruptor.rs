// Disruptor Weapon

// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"

// #include "cg_local.h"
// #include "cg_media.h"
// #include "FxScheduler.h"

use core::ffi::{c_int, c_char};
use core::ptr::addr_of;

type vec3_t = [f32; 3];

// Stubs for external constants
extern "C" {
    static FX_SIZE_LINEAR: c_int;
    static FX_ALPHA_LINEAR: c_int;
    static FX_ALPHA_WAVE: c_int;
    static vec3_origin: vec3_t;
}

// External functions
extern "C" {
    fn FX_AddLine(
        handle: c_int,
        start: *const f32,
        end: *const f32,
        f1: f32,
        f2: f32,
        f3: f32,
        f4: f32,
        f5: f32,
        f6: f32,
        color1: *const f32,
        color2: *const f32,
        f7: f32,
        duration: c_int,
        shader: c_int,
        flags1: c_int,
        flags2: c_int,
    );

    fn FX_AddBezier(
        start: *const f32,
        end: *const f32,
        c1: *const f32,
        c1b: *const f32,
        c2: *const f32,
        c2b: *const f32,
        f1: f32,
        f2: f32,
        f3: f32,
        f4: f32,
        f5: f32,
        f6: f32,
        color1: *const f32,
        color2: *const f32,
        f7: f32,
        duration: c_int,
        shader: c_int,
        flags: c_int,
    );

    fn cgi_R_RegisterShader(name: *const c_char) -> c_int;

    fn VectorMA(v: *const f32, scale: f32, dir: *const f32, dest: *mut f32);
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorAdd(v1: *const f32, v2: *const f32, dest: *mut f32);
}

// Stub for external C++ method call
extern "C" {
    fn theFxScheduler_PlayEffect(name: *const c_char, origin: *const f32, normal: *const f32);
}

/*
---------------------------
FX_DisruptorMainShot
---------------------------
*/
static mut WHITE: vec3_t = [1.0f32, 1.0f32, 1.0f32];

fn FX_DisruptorMainShot(start: *const f32, end: *const f32) {
    unsafe {
        FX_AddLine(
            -1,
            start,
            end,
            0.1f32,
            4.0f32,
            0.0f32,
            1.0f32,
            0.0f32,
            0.0f32,
            addr_of!(WHITE) as *const f32,
            addr_of!(WHITE) as *const f32,
            0.0f32,
            120,
            cgi_R_RegisterShader(b"gfx/effects/redLine\0".as_ptr() as *const c_char),
            0,
            FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        );
    }
}


/*
---------------------------
FX_DisruptorAltShot
---------------------------
*/
fn FX_DisruptorAltShot(start: *const f32, end: *const f32, fullCharge: c_int) {
    unsafe {
        FX_AddLine(
            -1,
            start,
            end,
            0.1f32,
            10.0f32,
            0.0f32,
            1.0f32,
            0.0f32,
            0.0f32,
            addr_of!(WHITE) as *const f32,
            addr_of!(WHITE) as *const f32,
            0.0f32,
            175,
            cgi_R_RegisterShader(b"gfx/effects/redLine\0".as_ptr() as *const c_char),
            0,
            FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        );

        if fullCharge != 0 {
            let YELLER: vec3_t = [0.8f32, 0.7f32, 0.0f32];

            // add some beef
            FX_AddLine(
                -1,
                start,
                end,
                0.1f32,
                7.0f32,
                0.0f32,
                1.0f32,
                0.0f32,
                0.0f32,
                addr_of!(YELLER) as *const f32,
                addr_of!(YELLER) as *const f32,
                0.0f32,
                150,
                cgi_R_RegisterShader(b"gfx/misc/whiteline2\0".as_ptr() as *const c_char),
                0,
                FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
            );
        }
    }
}

/*
---------------------------
FX_DisruptorAltMiss
---------------------------
*/

fn FX_DisruptorAltMiss(origin: *const f32, normal: *const f32) {
    unsafe {
        let mut pos: vec3_t = [0.0f32; 3];
        let mut c1: vec3_t = [0.0f32; 3];
        let mut c2: vec3_t = [0.0f32; 3];

        VectorMA(origin, 4.0f32, normal, c1.as_mut_ptr());
        VectorCopy(c1.as_ptr(), c2.as_mut_ptr());
        c1[2] += 4f32;
        c2[2] += 12f32;

        VectorAdd(origin, normal, pos.as_mut_ptr());
        pos[2] += 28f32;

        FX_AddBezier(
            origin,
            pos.as_ptr(),
            c1.as_ptr(),
            vec3_origin.as_ptr(),
            c2.as_ptr(),
            vec3_origin.as_ptr(),
            6.0f32,
            6.0f32,
            0.0f32,
            0.0f32,
            0.2f32,
            0.5f32,
            addr_of!(WHITE) as *const f32,
            addr_of!(WHITE) as *const f32,
            0.0f32,
            4000,
            cgi_R_RegisterShader(b"gfx/effects/smokeTrail\0".as_ptr() as *const c_char),
            FX_ALPHA_WAVE,
        );

        theFxScheduler_PlayEffect(
            b"disruptor/alt_miss\0".as_ptr() as *const c_char,
            origin,
            normal,
        );
    }
}

/*
---------------------------
FX_KothosBeam
---------------------------
*/
fn FX_KothosBeam(start: *const f32, end: *const f32) {
    unsafe {
        FX_AddLine(
            -1,
            start,
            end,
            0.1f32,
            10.0f32,
            0.0f32,
            1.0f32,
            0.0f32,
            0.0f32,
            addr_of!(WHITE) as *const f32,
            addr_of!(WHITE) as *const f32,
            0.0f32,
            175,
            cgi_R_RegisterShader(b"gfx/misc/dr1\0".as_ptr() as *const c_char),
            0,
            FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        );

        let YELLER: vec3_t = [0.8f32, 0.7f32, 0.0f32];

        // add some beef
        FX_AddLine(
            -1,
            start,
            end,
            0.1f32,
            7.0f32,
            0.0f32,
            1.0f32,
            0.0f32,
            0.0f32,
            addr_of!(YELLER) as *const f32,
            addr_of!(YELLER) as *const f32,
            0.0f32,
            150,
            cgi_R_RegisterShader(b"gfx/misc/whiteline2\0".as_ptr() as *const c_char),
            0,
            FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        );
    }
}
