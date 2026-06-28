// Copyright (C) 1999-2000 Id Software, Inc.
//
// leave this at the top of all UI_xxxx files for PCH reasons...
//

// Stub for exe_headers.h - platform/executable headers (not needed in Rust)
// Stub for ui_local.h - local UI declarations

use core::ffi::{c_char, c_int, c_void};

// Conditional compilation stub for _IMMERSION
// #ifdef _IMMERSION
// #include "../ff/ff.h"
// #endif // _IMMERSION

// this file is only included when building a dll
// syscalls.asm is included instead when building a qvm

// PORTING: Static function pointer for syscalls, mirroring C's variadic signature
static mut syscall: unsafe extern "C" fn(c_int, ...) -> c_int =
    unsafe { core::mem::transmute(-1isize) };

pub unsafe extern "C" fn dllEntry(syscallptr: unsafe extern "C" fn(c_int, ...) -> c_int) {
    syscall = syscallptr;
    //	CG_PreInit();
}

// Porting: Convert float to int bit-representation
#[inline(always)]
pub fn PASSFLOAT(x: f32) -> c_int {
    let floatTemp: f32 = x;
    let ptr = &floatTemp as *const f32 as *const c_int;
    unsafe { *ptr }
}

// Stub declarations for external types
#[repr(C)]
pub struct refEntity_t {
    // PORTING: Stub - full definition not needed for syscalls.rs
}

#[repr(C)]
pub struct refdef_t {
    // PORTING: Stub - full definition not needed for syscalls.rs
}

#[repr(C)]
pub struct glconfig_t {
    // PORTING: Stub - full definition not needed for syscalls.rs
}

// Type aliases for C handle types
pub type qhandle_t = c_int;
pub type clipHandle_t = c_int;
pub type sfxHandle_t = c_int;
pub type qboolean = c_int;
pub type ffHandle_t = c_int;

// Stub declarations for external functions
extern "C" {
    pub fn FloatAsInt(f: f32) -> c_int;
    pub fn Cvar_VariableValue(var_name: *const c_char) -> f32;
}

extern "C" {
    pub fn CL_UISystemCalls(args: *mut c_int) -> c_int;
}

extern "C" {
    pub fn S_StartLocalSound(sfx: sfxHandle_t, channelNum: c_int);
    pub fn S_StopSounds();
    pub fn S_RegisterSound(sample: *const c_char) -> sfxHandle_t;
}

extern "C" {
    pub fn Key_SetBinding(keynum: c_int, binding: *const c_char);
    pub fn Key_GetOverstrikeMode() -> qboolean;
    pub fn Key_SetOverstrikeMode(state: qboolean);
    pub fn Key_ClearStates();
    pub fn Key_GetCatcher() -> c_int;
    pub fn Key_SetCatcher(catcher: c_int);
}

extern "C" {
    pub fn CL_GetGlconfig(glconfig: *mut glconfig_t);
}

// Conditional declarations for _IMMERSION
#[cfg(feature = "immersion")]
extern "C" {
    pub fn FF_AddForce(ff: ffHandle_t);
    pub fn FF_Register(name: *const c_char, channel: c_int, qtrue: qboolean) -> ffHandle_t;
}

// Stub for ui object - this appears to be a struct with function pointers in C
// PORTING: Stub - full definition in ui_local.h
pub struct UI_t {
    // PORTING: Members would include function pointers like R_ClearScene, R_AddRefEntityToScene, etc.
}

// Global ui instance - PORTING: Stub, defined elsewhere
extern "C" {
    pub static mut ui: UI_t;
}

// PORTING: Methods on UI struct would be called via function pointers
// For now, declaring stubs for the method wrapper calls
extern "C" {
    pub fn ui_R_ClearScene();
    pub fn ui_R_AddRefEntityToScene(re: *const refEntity_t);
    pub fn ui_R_RenderScene(fd: *const refdef_t);
    pub fn ui_R_SetColor(rgba: *const f32);
    pub fn ui_R_DrawStretchPic(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        hShader: qhandle_t,
    );
    pub fn ui_R_ModelBounds(model: clipHandle_t, mins: *mut [f32; 3], maxs: *mut [f32; 3]);
}

pub fn trap_Cvar_VariableValue(var_name: *const c_char) -> f32 {
    let temp: c_int;
    //	temp = syscall( UI_CVAR_VARIABLEVALUE, var_name );
    unsafe {
        temp = FloatAsInt(Cvar_VariableValue(var_name));
        *((&temp as *const c_int) as *const f32)
    }
}

pub fn trap_R_ClearScene() {
    unsafe {
        ui_R_ClearScene();
    }
}

pub fn trap_R_AddRefEntityToScene(re: *const refEntity_t) {
    unsafe {
        ui_R_AddRefEntityToScene(re);
    }
}

pub fn trap_R_RenderScene(fd: *const refdef_t) {
    //	syscall( UI_R_RENDERSCENE, fd );
    unsafe {
        ui_R_RenderScene(fd);
    }
}

pub fn trap_R_SetColor(rgba: *const f32) {
    //	syscall( UI_R_SETCOLOR, rgba );
    //	re.SetColor( rgba );
    unsafe {
        ui_R_SetColor(rgba);
    }
}

pub fn trap_R_DrawStretchPic(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    hShader: qhandle_t,
) {
    //	syscall( UI_R_DRAWSTRETCHPIC, PASSFLOAT(x), PASSFLOAT(y), PASSFLOAT(w), PASSFLOAT(h), PASSFLOAT(s1), PASSFLOAT(t1), PASSFLOAT(s2), PASSFLOAT(t2), hShader );
    //	re.DrawStretchPic( x, y, w, h, s1, t1, s2, t2, hShader  );

    unsafe {
        ui_R_DrawStretchPic(x, y, w, h, s1, t1, s2, t2, hShader);
    }
}

pub fn trap_R_ModelBounds(model: clipHandle_t, mins: *mut [f32; 3], maxs: *mut [f32; 3]) {
    //	syscall( UI_R_MODELBOUNDS, model, mins, maxs );
    unsafe {
        ui_R_ModelBounds(model, mins, maxs);
    }
}

pub fn trap_S_StartLocalSound(sfx: sfxHandle_t, channelNum: c_int) {
    //	syscall( UI_S_STARTLOCALSOUND, sfx, channelNum );
    unsafe {
        S_StartLocalSound(sfx, channelNum);
    }
}

pub fn trap_S_StopSounds() {
    unsafe {
        S_StopSounds();
    }
}

pub fn trap_S_RegisterSound(sample: *const c_char, _compressed: qboolean) -> sfxHandle_t {
    unsafe { S_RegisterSound(sample) }
}

#[cfg(feature = "immersion")]
pub fn trap_FF_Start(ff: ffHandle_t) {
    unsafe {
        FF_AddForce(ff);
    }
}

#[cfg(feature = "immersion")]
pub fn trap_FF_Register(name: *const c_char, channel: c_int) -> ffHandle_t {
    unsafe { FF_Register(name, channel, 1) }
}

pub fn trap_Key_SetBinding(keynum: c_int, binding: *const c_char) {
    unsafe {
        Key_SetBinding(keynum, binding);
    }
}

pub fn trap_Key_GetOverstrikeMode() -> qboolean {
    unsafe { Key_GetOverstrikeMode() }
}

pub fn trap_Key_SetOverstrikeMode(state: qboolean) {
    unsafe {
        Key_SetOverstrikeMode(state);
    }
}

pub fn trap_Key_ClearStates() {
    unsafe {
        Key_ClearStates();
    }
}

pub fn trap_Key_GetCatcher() -> c_int {
    unsafe { Key_GetCatcher() }
}

pub fn trap_Key_SetCatcher(catcher: c_int) {
    unsafe {
        Key_SetCatcher(catcher);
    }
}

/*
void trap_GetClipboardData( char *buf, int bufsize ) {
	syscall( UI_GETCLIPBOARDDATA, buf, bufsize );
}

void trap_GetClientState( uiClientState_t *state ) {
	syscall( UI_GETCLIENTSTATE, state );
}
*/

pub fn trap_GetGlconfig(glconfig: *mut glconfig_t) {
    //	syscall( UI_GETGLCONFIG, glconfig );
    unsafe {
        CL_GetGlconfig(glconfig);
    }
}

#[cfg(not(target_os = "xbox"))]
// this returns a handle.  arg0 is the name in the format "idlogo.roq", set arg1 to NULL, alteredstates to qfalse (do not alter gamestate)
pub fn trap_CIN_PlayCinematic(
    arg0: *const c_char,
    xpos: c_int,
    ypos: c_int,
    width: c_int,
    height: c_int,
    bits: c_int,
    psAudioFile: *const c_char,
) -> c_int {
    unsafe { syscall(256, arg0, xpos, ypos, width, height, bits, psAudioFile) }
}

// stops playing the cinematic and ends it.  should always return FMV_EOF
// cinematics must be stopped in reverse order of when they are started
pub fn trap_CIN_StopCinematic(handle: c_int) -> c_int {
    unsafe { syscall(257, handle) }
}
