# Blind Port Handoff: small headers batch 6

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1077
- oracle/code: 646
- oracle/codemp: 431

## Delegated

- `oracle/codemp/ui/ui_force.h` -> `src/codemp/ui/ui_force_h.rs`
- `oracle/codemp/client/keys.h` -> `src/codemp/client/keys_h.rs`
- `oracle/codemp/client/fffx.h` -> `src/codemp/client/fffx_h.rs`
- `oracle/codemp/qcommon/vm_local.h` -> `src/codemp/qcommon/vm_local_h.rs`
- `oracle/codemp/qcommon/cm_landscape.h` -> `src/codemp/qcommon/cm_landscape_h.rs`

## Committed In This Batch

- `9d42ff2 port oracle/codemp/ui/ui_force.h`
- `f1373fa port oracle/codemp/client/keys.h`
- `db2fe31 port oracle/codemp/client/fffx.h`
- `06c28ac port oracle/codemp/qcommon/vm_local.h`
- `4f24c40 port oracle/codemp/qcommon/cm_landscape.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- `ui_force_h.rs` uses a local opaque `rectDef_t` alias because `ui_shared.h` is not ported yet.
- `keys_h.rs` uses a local `word = u16` typedef and imports `MAX_KEYS` from the existing UI keycodes port.
- `fffx_h.rs` mirrors `CGAME_ONLY` with `feature = "cgame_only"` and uses thin Rust functions for C force-feedback macros.
- `vm_local_h.rs` uses a local opaque `vmHeader_t` stub for `qfiles.h` and opaque `std::map` stubs for non-Xbox symbol maps.
- `cm_landscape_h.rs` uses local stubs for terrain types including `thandle_t`, `cbrush_s`, `cbrushside_s`, `CCMShader`, `CRandomTerrain`, and opaque STL list storage.

## Next Recommended Batch

Good next candidates:

- `oracle/codemp/client/snd_public.h` -> `src/codemp/client/snd_public_h.rs`
- `oracle/codemp/client/snd_music.h` -> `src/codemp/client/snd_music_h.rs`
- `oracle/codemp/client/FxUtil.h` -> `src/codemp/client/FxUtil_h.rs`
- `oracle/codemp/client/snd_ambient.h` -> `src/codemp/client/snd_ambient_h.rs`
- `oracle/codemp/qcommon/RoffSystem.h` -> `src/codemp/qcommon/RoffSystem_h.rs`

Possible but heavier follow-ups:

- `oracle/codemp/qcommon/GenericParser2.h` -> `src/codemp/qcommon/GenericParser2_h.rs`
- `oracle/codemp/ui/ui_public.h` -> `src/codemp/ui/ui_public_h.rs`
- `oracle/codemp/qcommon/cm_local.h` -> `src/codemp/qcommon/cm_local_h.rs`

Continue avoiding broad renderer/client/source files and large dependency-heavy headers such as `server.h`, `qfiles.h`, `sparc.h`, `hstring.h`, and `ui_shared.h`.

## State

- Branch: `full-port`
- HEAD before this handoff commit: `4f24c40 port oracle/codemp/qcommon/cm_landscape.h`
- Working tree was clean before this handoff file was created.
