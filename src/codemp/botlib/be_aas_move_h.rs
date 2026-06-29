/*****************************************************************************
 * name:		be_aas_move.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_move.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_float};

// Opaque type stubs
#[repr(C)]
pub struct aas_clientmove_s {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct aas_reachability_s {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct aas_settings_t {
    _opaque: [u8; 0],
}

pub type vec3_t = [c_float; 3];

#[cfg(feature = "AASINTERN")]
extern "C" {
    pub static mut aassettings: aas_settings_t;
}

extern "C" {
    //movement prediction
    pub fn AAS_PredictClientMovement(
        r#move: *mut aas_clientmove_s,
        entnum: c_int,
        origin: vec3_t,
        presencetype: c_int,
        onground: c_int,
        velocity: vec3_t,
        cmdmove: vec3_t,
        cmdframes: c_int,
        maxframes: c_int,
        frametime: c_float,
        stopevent: c_int,
        stopareanum: c_int,
        visualize: c_int,
    ) -> c_int;

    //predict movement until bounding box is hit
    pub fn AAS_ClientMovementHitBBox(
        r#move: *mut aas_clientmove_s,
        entnum: c_int,
        origin: vec3_t,
        presencetype: c_int,
        onground: c_int,
        velocity: vec3_t,
        cmdmove: vec3_t,
        cmdframes: c_int,
        maxframes: c_int,
        frametime: c_float,
        mins: vec3_t,
        maxs: vec3_t,
        visualize: c_int,
    ) -> c_int;

    //returns true if on the ground at the given origin
    pub fn AAS_OnGround(origin: vec3_t, presencetype: c_int, passent: c_int) -> c_int;

    //returns true if swimming at the given origin
    pub fn AAS_Swimming(origin: vec3_t) -> c_int;

    //returns the jump reachability run start point
    pub fn AAS_JumpReachRunStart(reach: *mut aas_reachability_s, runstart: vec3_t);

    //returns true if against a ladder at the given origin
    pub fn AAS_AgainstLadder(origin: vec3_t) -> c_int;

    //rocket jump Z velocity when rocket-jumping at origin
    pub fn AAS_RocketJumpZVelocity(origin: vec3_t) -> c_float;

    //bfg jump Z velocity when bfg-jumping at origin
    pub fn AAS_BFGJumpZVelocity(origin: vec3_t) -> c_float;

    //calculates the horizontal velocity needed for a jump and returns true this velocity could be calculated
    pub fn AAS_HorizontalVelocityForJump(
        zvel: c_float,
        start: vec3_t,
        end: vec3_t,
        velocity: *mut c_float,
    ) -> c_int;

    //
    pub fn AAS_SetMovedir(angles: vec3_t, movedir: vec3_t);

    //
    pub fn AAS_DropToFloor(origin: vec3_t, mins: vec3_t, maxs: vec3_t) -> c_int;

    //
    pub fn AAS_InitSettings();
}
