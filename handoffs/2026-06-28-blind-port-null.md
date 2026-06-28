# Blind Port Handoff: codemp/null

Date: 2026-06-28

Pairing command:
- `scripts/compare-src-oracle.sh`

Latest pairing result:
- Oracle files without paired Rust files: 1152
- `oracle/code`: 646
- `oracle/codemp`: 506

Delegated files:
- `oracle/codemp/null/null_renderer.cpp` -> `src/codemp/null/null_renderer.rs`
- `oracle/codemp/null/null_net.c` -> `src/codemp/null/null_net.rs`
- `oracle/codemp/null/null_client.cpp` -> `src/codemp/null/null_client.rs`
- `oracle/codemp/null/mac_net.c` -> `src/codemp/null/mac_net.rs`
- `oracle/codemp/null/null_main.c` -> `src/codemp/null/null_main.rs`
- `oracle/codemp/null/null_input.cpp` -> `src/codemp/null/null_input.rs`
- `oracle/codemp/null/null_snddma.cpp` -> `src/codemp/null/null_snddma.rs`
- `oracle/codemp/null/null_glimp.cpp` -> `src/codemp/null/null_glimp.rs`

Committed files:
- `1537a2b port oracle/codemp/null/null_renderer.cpp`
- `cf3768c port oracle/codemp/null/null_net.c`
- `b91bad3 port oracle/codemp/null/null_client.cpp`
- `6a27de5 port oracle/codemp/null/mac_net.c`
- `c7b3910 port oracle/codemp/null/null_main.c`
- `ee49cd3 port oracle/codemp/null/null_input.cpp null_snddma.cpp`
- `97d09be port oracle/codemp/null/null_glimp.cpp`

Returned with unresolved dependencies:
- None blocked.
- `null_net.rs` and `null_client.rs` include local ABI stubs for unported qcommon/client types (`netadr_t`, `msg_t`, `cvar_t`) where shared Rust definitions do not exist yet.
- `mac_net.rs` duplicates the scoped network ABI stubs used by `null_net.rs`.
- `null_main.rs` uses extern declarations for libc/engine calls (`fread`, `fseek`, `printf`, `exit`, `Com_Init`, `Com_Frame`) and documents the stable-Rust limitation for C variadic `Sys_Error`.

Next recommended batch:
- `oracle/codemp/null/win_main.cpp` -> `src/codemp/null/win_main.rs` remains in `codemp/null/`; it is large and should be delegated alone.
- After that, move to another small platform directory such as `oracle/codemp/unix/` or `oracle/codemp/win32/` single-file stubs.

Current branch and HEAD:
- Branch: `full-port`
- HEAD: `97d09be port oracle/codemp/null/null_glimp.cpp`

Agent failures:
- None.

Notes:
- No build, test, cargo check, or cargo fmt was run, per blind-port rules.
- `oracle/` was not modified.
