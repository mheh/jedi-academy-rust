// Current version of the single player game
// Ported from: oracle/code/qcommon/stv_version.h

// Original C includes: ../win32/autoversion.h
// PORTING NOTE: VERSION_STRING_DOTTED originally provided by autoversion.h.
// Using stub value pending version information from build system.
const VERSION_STRING_DOTTED: &str = "1.0.0";

#[cfg(debug_assertions)]
pub const Q3_VERSION: &str = "(debug)JA: v1.0.0";

#[cfg(all(not(debug_assertions), feature = "final_build"))]
pub const Q3_VERSION: &str = "JA: v1.0.0";

#[cfg(all(not(debug_assertions), not(feature = "final_build")))]
pub const Q3_VERSION: &str = "(internal)JA: v1.0.0";

// end
