//! `g_ICARUScb.h` — ICARUS callback declarations.

#![allow(non_snake_case)]

use crate::codemp::game::q_shared_h::{qboolean, vec3_t};
use core::ffi::{c_char, c_int};

unsafe extern "C" {
    pub fn Q3_PlaySound(
        taskID: c_int,
        entID: c_int,
        name: *const c_char,
        channel: *const c_char,
    ) -> c_int;
    pub fn Q3_Set(
        taskID: c_int,
        entID: c_int,
        type_name: *const c_char,
        data: *const c_char,
    ) -> qboolean;
    pub fn Q3_Lerp2Pos(
        taskID: c_int,
        entID: c_int,
        origin: *mut vec3_t,
        angles: *mut vec3_t,
        duration: f32,
    );
    pub fn Q3_Lerp2Origin(taskID: c_int, entID: c_int, origin: *mut vec3_t, duration: f32);
    pub fn Q3_Lerp2Angles(taskID: c_int, entID: c_int, angles: *mut vec3_t, duration: f32);
    pub fn Q3_GetTag(
        entID: c_int,
        name: *const c_char,
        lookup: c_int,
        info: *mut vec3_t,
    ) -> c_int;
    pub fn Q3_Lerp2Start(entID: c_int, taskID: c_int, duration: f32);
    pub fn Q3_Lerp2End(entID: c_int, taskID: c_int, duration: f32);
    pub fn Q3_Use(entID: c_int, target: *const c_char);
    pub fn Q3_Kill(entID: c_int, name: *const c_char);
    pub fn Q3_Remove(entID: c_int, name: *const c_char);
    pub fn Q3_Play(taskID: c_int, entID: c_int, r#type: *const c_char, name: *const c_char);
    pub fn Q3_GetFloat(
        entID: c_int,
        r#type: c_int,
        name: *const c_char,
        value: *mut f32,
    ) -> c_int;
    pub fn Q3_GetVector(
        entID: c_int,
        r#type: c_int,
        name: *const c_char,
        value: *mut vec3_t,
    ) -> c_int;
    pub fn Q3_GetString(
        entID: c_int,
        r#type: c_int,
        name: *const c_char,
        value: *mut *mut c_char,
    ) -> c_int;
}
