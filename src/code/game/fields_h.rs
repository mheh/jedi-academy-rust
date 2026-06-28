#![allow(non_snake_case)]

// Filename:-	fields.h
//

//
// fields are needed for spawning from the entity string
// and saving / loading games
//

use core::ffi::c_char;

pub const FFL_SPAWNTEMP: i32 = 1;
pub const MAX_GHOULINST_SIZE: i32 = 16384;

//
// Offset calculation macros (preserved from C header):
// These macros require type definitions in scope: gentity_t, spawn_temp_t, level_locals_t, gclient_t, gNPC_t, Vehicle_t
//
// #ifndef FOFS
// #define	FOFS(x) ((int)&(((gentity_t *)0)->x))	// usually already defined in qshared.h
// #endif
// #define	STOFS(x) ((int)&(((spawn_temp_t *)0)->x))
// #define	LLOFS(x) ((int)&(((level_locals_t *)0)->x))
// #define	CLOFS(x) ((int)&(((gclient_t *)0)->x))
// #define NPCOFS(x) ((int)&(((gNPC_t *)0)->x))
// #define VHOFS(x) ((int)&(((Vehicle_t *)0)->x))
//
// #define strFOFS(x)	 #x,FOFS(x)
// #define	strSTOFS(x)  #x,STOFS(x)
// #define	strLLOFS(x)	 #x,LLOFS(x)
// #define	strCLOFS(x)  #x,CLOFS(x)
// #define strNPCOFS(x) #x,NPCOFS(x)
// #define strVHICOFS(x) #x,VHOFS(x)
//

#[repr(C)]
pub enum fieldtypeSAVE_t {
    //	F_INT,
    //	F_SHORT,
    //	F_FLOAT,
    F_STRING = 0,       // string
    //	F_VECTOR,
    F_NULL = 1,         // A ptr to null out
    F_ITEM = 2,         // Item pointer handling
    //	F_MMOVE,			// Mmove pointer handling
    F_GCLIENT = 3,      // Client pointer handling
    F_GENTITY = 4,      // gentity_t ptr handling
    F_BOOLPTR = 5,      // Generic pointer that is recreated later, could be left alone, but clearer if only 0/1 rather than 0/alloc

    F_BEHAVIORSET = 6,  // special scripting string ptr array handler
    F_ALERTEVENT = 7,   // special handler for alertevent struct in level_locals_t
    F_AIGROUPS = 8,     // some AI grouping stuff of Mike's
    F_ANIMFILESETS = 9, // animfileset animevent strings

    F_GROUP = 10,
    F_VEHINFO = 11,
    F_IGNORE = 12,
}

#[repr(C)]
pub struct save_field_t {
    pub psName: *const c_char,    // char	*psName;
    pub iOffset: i32,             // int		iOffset;
    pub eFieldType: fieldtypeSAVE_t, // fieldtypeSAVE_t	eFieldType;
}

//////////////////////// eof //////////////////////////
