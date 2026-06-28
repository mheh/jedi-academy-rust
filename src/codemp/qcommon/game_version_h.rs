#![allow(non_snake_case)]

// C origin: `codemp/qcommon/game_version.h`.
// Includes `../win32/AutoVersion.h`.

pub use crate::codemp::win32::AutoVersion_h::{
    VERSION_BUILD_NUMBER, VERSION_EXTERNAL_BUILD, VERSION_INTERNAL_BUILD, VERSION_MAJOR_RELEASE,
    VERSION_MINOR_RELEASE, VERSION_STRING, VERSION_STRING_DOTTED,
};

// Current version of the multi player game
#[cfg(debug_assertions)]
pub const Q3_VERSION: &str = "(debug)JAmp: v1.0.1.0";

// `FINAL_BUILD` has no crate feature in this port yet; the default non-debug path
// mirrors the original `#else` branch.
#[cfg(not(debug_assertions))]
pub const Q3_VERSION: &str = "(internal)JAmp: v1.0.1.0";

//end
