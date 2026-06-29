// snddma_null.c
// all other sound mixing is portable

use core::ffi::c_int;

// Local stub: qboolean is typically typedef'd to int in Quake engine
type qboolean = c_int;

#[allow(non_snake_case)]
pub extern "C" fn SNDDMA_Init() -> qboolean {
	0
}

#[allow(non_snake_case)]
pub extern "C" fn SNDDMA_GetDMAPos() -> c_int {
	0
}

#[allow(non_snake_case)]
pub extern "C" fn SNDDMA_Shutdown() {
}

#[allow(non_snake_case)]
pub extern "C" fn SNDDMA_BeginPainting() {
}

#[allow(non_snake_case)]
pub extern "C" fn SNDDMA_Submit() {
}
