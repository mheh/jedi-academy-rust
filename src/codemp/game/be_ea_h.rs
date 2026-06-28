//! `be_ea.h` — elementary action declarations.

#![allow(non_snake_case)]

use crate::codemp::game::botlib_h::bot_input_t;
use crate::codemp::game::q_shared_h::vec3_t;
use core::ffi::{c_char, c_int};

unsafe extern "C" {
    // ClientCommand elementary actions
    pub fn EA_Say(client: c_int, str: *mut c_char);
    pub fn EA_SayTeam(client: c_int, str: *mut c_char);
    pub fn EA_Command(client: c_int, command: *mut c_char);

    pub fn EA_Action(client: c_int, action: c_int);
    pub fn EA_Crouch(client: c_int);
    pub fn EA_Walk(client: c_int);
    pub fn EA_MoveUp(client: c_int);
    pub fn EA_MoveDown(client: c_int);
    pub fn EA_MoveForward(client: c_int);
    pub fn EA_MoveBack(client: c_int);
    pub fn EA_MoveLeft(client: c_int);
    pub fn EA_MoveRight(client: c_int);
    pub fn EA_Attack(client: c_int);
    pub fn EA_Alt_Attack(client: c_int);
    pub fn EA_ForcePower(client: c_int);
    pub fn EA_Respawn(client: c_int);
    pub fn EA_Talk(client: c_int);
    pub fn EA_Gesture(client: c_int);
    pub fn EA_Use(client: c_int);

    // regular elementary actions
    pub fn EA_SelectWeapon(client: c_int, weapon: c_int);
    pub fn EA_Jump(client: c_int);
    pub fn EA_DelayedJump(client: c_int);
    pub fn EA_Move(client: c_int, dir: *mut vec3_t, speed: f32);
    pub fn EA_View(client: c_int, viewangles: *mut vec3_t);

    // send regular input to the server
    pub fn EA_EndRegular(client: c_int, thinktime: f32);
    pub fn EA_GetInput(client: c_int, thinktime: f32, input: *mut bot_input_t);
    pub fn EA_ResetInput(client: c_int);
    // setup and shutdown routines
    pub fn EA_Setup() -> c_int;
    pub fn EA_Shutdown();
}
