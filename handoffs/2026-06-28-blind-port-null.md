# Blind Port Handoff: codemp/null

Date: 2026-06-28

Pairing command:
- `scripts/compare-src-oracle.sh`

Latest pairing result:
- Oracle files without paired Rust files: 1157
- `oracle/code`: 646
- `oracle/codemp`: 511

Delegated files:
- `oracle/codemp/null/null_renderer.cpp` -> `src/codemp/null/null_renderer.rs`
- `oracle/codemp/null/null_net.c` -> `src/codemp/null/null_net.rs`
- `oracle/codemp/null/null_client.cpp` -> `src/codemp/null/null_client.rs`

Committed files:
- `1537a2b port oracle/codemp/null/null_renderer.cpp`
- `cf3768c port oracle/codemp/null/null_net.c`
- `b91bad3 port oracle/codemp/null/null_client.cpp`

Returned with unresolved dependencies:
- None blocked.
- `null_net.rs` and `null_client.rs` include local ABI stubs for unported qcommon/client types (`netadr_t`, `msg_t`, `cvar_t`) where shared Rust definitions do not exist yet.

Next recommended batch:
- `oracle/codemp/null/null_main.c` -> `src/codemp/null/null_main.rs`
- `oracle/codemp/null/mac_net.c` -> `src/codemp/null/mac_net.rs`
- `oracle/codemp/null/null_renderer.cpp` is done; remaining small null files include `null_glimp.cpp`, `null_input.cpp`, `null_snddma.cpp`, and `win_main.cpp`.

Current branch and HEAD:
- Branch: `full-port`
- HEAD: `b91bad3 port oracle/codemp/null/null_client.cpp`

Agent failures:
- None.

Notes:
- No build, test, cargo check, or cargo fmt was run, per blind-port rules.
- `oracle/` was not modified.
