// ICARUS Public Header File

// #pragma warning ( disable : 4786 )	//NOTENOTE: STL Debug name length warning

// #ifndef	__ICARUS__
// #define __ICARUS__

// #include "../game/g_public.h"

// #pragma warning( disable : 4786 )  // identifier was truncated
// #pragma warning( disable : 4514 )  // unreferenced inline was removed
// #pragma warning( disable : 4710 )  // not inlined

// #pragma warning( push, 3 )	//save current state and change to 3

use core::ffi::{c_int, c_void};

// #define STL_ITERATE( a, b )		for ( a = b.begin(); a != b.end(); a++ )
// #define STL_INSERT( a, b )		a.insert( a.end(), b );

// #include "tokenizer.h"
// #include "blockstream.h"
// #include "interpreter.h"
// #include "sequencer.h"
// #include "taskmanager.h"
// #include "instance.h"

// #pragma warning( pop )	//restore

// External memory allocation functions used by ICARUS
extern "C" {
    pub fn ICARUS_Malloc(iSize: c_int) -> *mut c_void;
    pub fn ICARUS_Free(pMem: *mut c_void);
}

// #endif	//__ICARUS__
