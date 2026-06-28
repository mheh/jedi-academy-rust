#![allow(non_snake_case)]

// C origin: `codemp/namespace_begin.h`.
// On Xbox builds this opened a C++ namespace selected by _GAME, _CGAME, or _UI.
// Rust modules already provide namespace boundaries, so this compatibility port
// intentionally declares no items.
