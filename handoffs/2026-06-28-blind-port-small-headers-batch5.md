# Blind Port Handoff: small headers batch 5

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1082
- oracle/code: 646
- oracle/codemp: 436

## Delegated

- `oracle/codemp/qcommon/cm_draw.h` -> `src/codemp/qcommon/cm_draw_h.rs`
- `oracle/codemp/server/exe_headers.h` -> `src/codemp/server/exe_headers_h.rs`
- `oracle/codemp/ui/keycodes.h` -> `src/codemp/ui/keycodes_h.rs`
- `oracle/codemp/client/keycodes.h` -> `src/codemp/client/keycodes_h.rs`

## Committed In This Batch

- `57a1ad9 port oracle/codemp/server/exe_headers.h`
- `7f7df6c port oracle/codemp/client/keycodes.h`
- `5e1a010 port oracle/codemp/ui/keycodes.h`
- `0546fdc port oracle/codemp/qcommon/cm_draw.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- `exe_headers_h.rs` is a PCH include shim only; original include order is preserved as comments.
- `client/keycodes_h.rs` and `ui/keycodes_h.rs` port matching `fakeAscii_t` enum values as `c_int` constants.
- `cm_draw_h.rs` adds a local `POINT` layout stub and preserves non-inline C++ member declarations as ordered comments.
- `cm_draw_h.rs` uses typed Rust helpers for C macros where exact macro genericity is not available.
- `cm_draw_h.rs` does not free `row_off` in `CleanUp`; allocation ownership remains unported and is documented in the file.

## Next Recommended Batch

Good next candidates:

- `oracle/codemp/ui/ui_force.h` -> `src/codemp/ui/ui_force_h.rs`
- `oracle/codemp/client/keys.h` -> `src/codemp/client/keys_h.rs`
- `oracle/codemp/client/fffx.h` -> `src/codemp/client/fffx_h.rs`
- `oracle/codemp/qcommon/vm_local.h` -> `src/codemp/qcommon/vm_local_h.rs`
- `oracle/codemp/qcommon/cm_landscape.h` -> `src/codemp/qcommon/cm_landscape_h.rs`

Continue avoiding broad renderer/client/source files and large dependency-heavy headers such as `server.h`, `qfiles.h`, `sparc.h`, and `hstring.h`.

## State

- Branch: `full-port`
- HEAD before this handoff commit: `0546fdc port oracle/codemp/qcommon/cm_draw.h`
- Working tree was clean before this handoff file was created.
