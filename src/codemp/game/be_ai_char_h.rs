//! `be_ai_char.h` — bot character declarations.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

unsafe extern "C" {
    // loads a bot character from a file
    pub fn BotLoadCharacter(charfile: *mut c_char, skill: f32) -> c_int;
    // frees a bot character
    pub fn BotFreeCharacter(character: c_int);
    // returns a float characteristic
    pub fn Characteristic_Float(character: c_int, index: c_int) -> f32;
    // returns a bounded float characteristic
    pub fn Characteristic_BFloat(character: c_int, index: c_int, min: f32, max: f32) -> f32;
    // returns an integer characteristic
    pub fn Characteristic_Integer(character: c_int, index: c_int) -> c_int;
    // returns a bounded integer characteristic
    pub fn Characteristic_BInteger(
        character: c_int,
        index: c_int,
        min: c_int,
        max: c_int,
    ) -> c_int;
    // returns a string characteristic
    pub fn Characteristic_String(character: c_int, index: c_int, buf: *mut c_char, size: c_int);
    // free cached bot characters
    pub fn BotShutdownCharacters();
}
