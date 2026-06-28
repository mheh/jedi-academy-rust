// this line must stay at top so the whole PCH thing works...

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

// Precompiled header file for the client game

// No PCH at all on Xbox build, we just include everything. Does this slow
// down builds? Somewhat. But then again, if I do change headers, I have to
// tolerate VS.net's piss poor dependency system that requires my to manually
// delete the .pch for the PC version to work at all. So, I'll live.

// PORTING: cg_local_h is unported
// use crate::code::cgame::cg_local_h;

// #[cfg(feature = "xbox")]
// PORTING: Xbox-only includes from game module (unported)
// use crate::code::game::g_local_h;
// use crate::code::game::g_functions_h;
// use crate::code::game::b_local_h;

// //#include "CGEntity.h"
// //#include "../game/SpawnSystem.h"
// //#include "../game/EntitySystem.h"
// //#include "../game/CScheduleSystem.h"

// end
