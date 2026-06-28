#![allow(non_snake_case)]

#[no_mangle]
pub extern "C" fn IN_Init() {}

#[no_mangle]
pub extern "C" fn IN_Frame() {}

#[no_mangle]
pub extern "C" fn IN_Shutdown() {}

#[no_mangle]
pub extern "C" fn Sys_SendKeyEvents() {}
