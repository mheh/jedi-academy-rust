// Port of oracle/code/game/common_headers.h

// #pragma once
// #if !defined(COMMON_HEADERS_H_INC)
// #define COMMON_HEADERS_H_INC

// #if !defined(__Q_SHARED_H)
//     #include "../game/q_shared.h"
// #endif

pub use crate::code::game::q_shared;

// //#if !defined(Q_SHAREDBASIC_H_INC)
// //	#include "../game/q_sharedbasic.h"			// fileHandle_t
// //#endif

// //#if !defined(Q_MATH_H_INC)
// //	#include "../game/q_math.h"
// //#endif

// #ifdef _XBOX
// #define GAME_INCLUDE
#[cfg(target_os = "xbox")]
pub const GAME_INCLUDE: bool = true;

// 	#include "../game/b_local.h"
#[cfg(target_os = "xbox")]
pub use crate::code::game::b_local_h;
// 	#include "../cgame/cg_local.h"
#[cfg(target_os = "xbox")]
pub use crate::cgame::cg_local_h;
// 	#include "../game/g_navigator.h"
#[cfg(target_os = "xbox")]
pub use crate::code::game::g_navigator_h;
// 	#include "../game/g_shared.h"
#[cfg(target_os = "xbox")]
pub use crate::code::game::g_shared_h;
// 	#include "../game/g_functions.h"
#[cfg(target_os = "xbox")]
pub use crate::code::game::g_functions_h;
// #endif

// #endif // COMMON_HEADERS_H_INC
