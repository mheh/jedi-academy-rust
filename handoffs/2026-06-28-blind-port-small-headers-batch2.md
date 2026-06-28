# Blind Port Handoff: small headers batch 2

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1096
- oracle/code: 646
- oracle/codemp: 450

## Delegated

- `oracle/codemp/game/bg_lib.h` -> `src/codemp/game/bg_lib_h.rs`
- `oracle/codemp/qcommon/cm_public.h` -> `src/codemp/qcommon/cm_public_h.rs`
- `oracle/codemp/qcommon/tags.h` -> `src/codemp/qcommon/tags_h.rs`
- `oracle/codemp/game/g_nav.h` -> `src/codemp/game/g_nav_h.rs`
- `oracle/codemp/qcommon/cm_terrainmap.h` -> `src/codemp/qcommon/cm_terrainmap_h.rs`

## Committed In This Batch

- `5ba6f12 port oracle/codemp/game/bg_lib.h`
- `b90658c port oracle/codemp/game/g_nav.h`
- `69150ec port oracle/codemp/qcommon/cm_public.h`
- `ed98dac port oracle/codemp/qcommon/cm_terrainmap.h`
- `05736a5 port oracle/codemp/qcommon/tags.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- `cm_public_h.rs` keeps `traceWork_s` and `CCMPatch` as opaque stubs, and defines local `clipHandle_t`, `markFragment_t`, and `orientation_t` because those are not yet in `q_shared_h.rs`.
- `cm_terrainmap_h.rs` keeps `CCMLandScape` opaque and preserves `CTerrainMap` C++ member declarations as an ordered comment block.
- `g_nav_h.rs` declares `NAV_CalculateSquadPaths` and `NPC_MoveToGoalExt`; no Rust definitions were found by the worker.
- `tags_h.rs` ports the active non-`_XBOX` tag list only.

## Next Recommended Batch

Good next small candidates:

- `oracle/codemp/encryption/encryption.h` -> `src/codemp/encryption/encryption_h.rs`
- `oracle/codemp/game/be_ai_chat.h` -> `src/codemp/game/be_ai_chat_h.rs`
- `oracle/codemp/game/inv.h` -> `src/codemp/game/inv_h.rs`
- `oracle/codemp/game/match.h` -> `src/codemp/game/match_h.rs`
- `oracle/codemp/qcommon/sstring.h` -> `src/codemp/qcommon/sstring_h.rs`

Avoid broad renderer/client/source files and large qcommon headers such as `sparc.h` / `qfiles.h` until more small surrounding headers are paired.

## State

- Branch: `full-port`
- HEAD: `05736a5 port oracle/codemp/qcommon/tags.h`
- Working tree was clean before this handoff file was created.
