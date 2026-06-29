// Simple header file to dispatch to the relevant platform API headers
#![allow(non_snake_case)]

// #if defined(_XBOX)
// #include <xtl.h>
// #endif

// #ifdef _WIN32
#[cfg(windows)]
pub const WIN32_LEAN_AND_MEAN: i32 = 1;
// #define WIN32_LEAN_AND_MEAN 1
// #endif

// #if defined(_WINDOWS)
// #include <windows.h>
// #endif
