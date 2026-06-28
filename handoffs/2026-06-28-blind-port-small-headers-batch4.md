# Blind Port Handoff: small headers batch 4

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1086
- oracle/code: 646
- oracle/codemp: 440

## Delegated

- `oracle/codemp/qcommon/stringed_ingame.h` -> `src/codemp/qcommon/stringed_ingame_h.rs`
- `oracle/codemp/qcommon/cm_patch.h` -> `src/codemp/qcommon/cm_patch_h.rs`
- `oracle/codemp/qcommon/cm_randomterrain.h` -> `src/codemp/qcommon/cm_randomterrain_h.rs`
- `oracle/codemp/qcommon/unzip.h` -> `src/codemp/qcommon/unzip_h.rs`
- `oracle/codemp/qcommon/files.h` -> `src/codemp/qcommon/files_h.rs`

## Committed In This Batch

- `9072c42 update blind port handoff workflow`
- `8075aa2 port oracle/codemp/qcommon/stringed_ingame.h`
- `8688ec9 port oracle/codemp/qcommon/cm_patch.h`
- `69a5276 port oracle/codemp/qcommon/cm_randomterrain.h`
- `0a38da8 port oracle/codemp/qcommon/unzip.h`
- `f6f1700 port oracle/codemp/qcommon/files.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- `stringed_ingame_h.rs` uses a local `cvar_t` layout stub because no shared qcommon `cvar_t` module is available yet.
- `cm_randomterrain_h.rs` reuses the opaque `CCMLandScape` from `cm_terrainmap_h.rs` and preserves C++ member declarations as ordered comments.
- `cm_patch_h.rs` preserves Xbox-only structures under `#[cfg(feature = "xbox")]`; default path mirrors non-Xbox.
- `unzip_h.rs` uses `z_stream` / `EStatus` from the existing zlib zip header port and treats strict unzip typedefs as undefined.
- `files_h.rs` keeps local fallbacks for `cvar_t`, `MAX_OSPATH`, and `MAX_FILE_HANDLES`; Xbox/GOB and `FS_MISSING` branches remain unported.

## Next Recommended Batch

Good next candidates:

- `oracle/codemp/qcommon/cm_draw.h` -> `src/codemp/qcommon/cm_draw_h.rs`
- `oracle/codemp/server/exe_headers.h` -> `src/codemp/server/exe_headers_h.rs`
- `oracle/codemp/ui/keycodes.h` -> `src/codemp/ui/keycodes_h.rs`
- `oracle/codemp/client/keycodes.h` -> `src/codemp/client/keycodes_h.rs`
- `oracle/codemp/qcommon/hstring.h` -> `src/codemp/qcommon/hstring_h.rs` only if handled solo because it is STL-heavy.

Continue avoiding broad renderer/client/source files and large dependency-heavy headers such as `server.h`, `qfiles.h`, and `sparc.h`.

## State

- Branch: `full-port`
- HEAD before this handoff commit: `f6f1700 port oracle/codemp/qcommon/files.h`
- Working tree was clean before this handoff file was created.
