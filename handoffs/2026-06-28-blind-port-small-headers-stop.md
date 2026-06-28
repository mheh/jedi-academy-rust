# Blind Port Handoff: small headers

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1099
- oracle/code: 646
- oracle/codemp: 453

## Delegated

- `oracle/codemp/qcommon/chash.h` -> `src/codemp/qcommon/chash_h.rs`
- `oracle/codemp/qcommon/fixedmap.h` -> `src/codemp/qcommon/fixedmap_h.rs`

## Committed Since PNG Handoff

- `2221f29 port oracle/codemp/game/chars.h`
- `325d7b7 port oracle/codemp/game/bg_strap.h`
- `294038a port oracle/codemp/qcommon/chash.h`
- `c8bf2ae port oracle/codemp/qcommon/fixedmap.h`
- `b0d73a7 port oracle/codemp/game/g_headers.h`
- `df54925 port oracle/codemp/game/be_ai_gen.h`
- `c8a417b port oracle/codemp/game/syn.h`
- `a3344cb port oracle/codemp/game/say.h`
- `f121453 port oracle/codemp/game/be_ai_char.h`
- `c9c13bb port oracle/codemp/qcommon/exe_headers.h`
- `504c8ba port oracle/codemp/game/npc_headers.h`
- `e455e0c port oracle/codemp/game/g_ICARUScb.h`
- `ecdc63a port oracle/codemp/qcommon/INetProfile.h`
- `3cb40dc port oracle/codemp/qcommon/stringed_interface.h`
- `48ac9e1 port oracle/codemp/qcommon/platform.h`
- `e7d5d3c port oracle/codemp/game/be_ea.h`
- `77caeb4 port oracle/codemp/qcommon/cm_polylib.h`
- `0bc8348 port oracle/codemp/game/g_team.h`
- `7b63d78 port oracle/codemp/qcommon/MiniHeap.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- Several new headers are declaration-only mirrors and keep C++ or engine dependencies explicit/opaque, notably `INetProfile_h.rs`, `stringed_interface_h.rs`, and `MiniHeap_h.rs`.
- `MiniHeap_h.rs` declares libc `malloc`/`free` and extern `G2VertSpaceServer` / `G2VertSpaceClient`.

## Next Recommended Batch

Good next small candidates:

- `oracle/codemp/game/bg_lib.h` -> `src/codemp/game/bg_lib_h.rs`
- `oracle/codemp/qcommon/cm_public.h` -> `src/codemp/qcommon/cm_public_h.rs`
- `oracle/codemp/qcommon/tags.h` -> `src/codemp/qcommon/tags_h.rs`
- `oracle/codemp/game/g_nav.h` -> `src/codemp/game/g_nav_h.rs`
- `oracle/codemp/qcommon/cm_terrainmap.h` -> `src/codemp/qcommon/cm_terrainmap_h.rs`

Avoid broad renderer/client/source files until the small qcommon/game headers around them are paired.

## State

- Branch: `full-port`
- HEAD: `7b63d78 port oracle/codemp/qcommon/MiniHeap.h`
- Working tree was clean before handoff file creation.
