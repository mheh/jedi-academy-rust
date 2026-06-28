# Blind Port Handoff: small headers batch 7

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1072
- oracle/code: 646
- oracle/codemp: 426

## Delegated

- `oracle/codemp/client/snd_public.h` -> `src/codemp/client/snd_public_h.rs`
- `oracle/codemp/client/snd_music.h` -> `src/codemp/client/snd_music_h.rs`
- `oracle/codemp/client/FxUtil.h` -> `src/codemp/client/FxUtil_h.rs`
- `oracle/codemp/client/snd_ambient.h` -> `src/codemp/client/snd_ambient_h.rs`
- `oracle/codemp/qcommon/RoffSystem.h` -> `src/codemp/qcommon/RoffSystem_h.rs`

## Committed In This Batch

- `b1f140d port oracle/codemp/client/snd_music.h`
- `38df23d port oracle/codemp/client/snd_public.h`
- `b6c0005 port oracle/codemp/client/FxUtil.h`
- `e562b3a port oracle/codemp/client/snd_ambient.h`
- `ed0bfc2 port oracle/codemp/qcommon/RoffSystem.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- `snd_music_h.rs` uses a local `sboolean = c_int` alias for the unported sound-local typedef and documents C++ default arguments as explicit parameters.
- `snd_public_h.rs` uses a local `sfxHandle_t = c_int` alias and maps `_XBOX` sound overloads to `feature = "xbox"`.
- `FxUtil_h.rs` uses opaque stubs for `FxPrimitives.h` classes, `refdef_t`, and `EMatImpactEffect`; C++ defaults are captured as `FX_DEFAULT_*` constants.
- `snd_ambient_h.rs` uses a local `sfxHandle_t` alias and opaque STL vector/map stubs for `CSetGroup`.
- `RoffSystem_h.rs` uses opaque STL map/vector stubs; non-inline C++ method bodies from `RoffSystem.cpp` remain explicit `todo!` stubs, while inline `NewID` is implemented.

## Next Recommended Batch

Good next candidates:

- `oracle/codemp/client/snd_mp3.h` -> `src/codemp/client/snd_mp3_h.rs`
- `oracle/codemp/client/snd_local_console.h` -> `src/codemp/client/snd_local_console_h.rs`
- `oracle/codemp/qcommon/GenericParser2.h` -> `src/codemp/qcommon/GenericParser2_h.rs`
- `oracle/codemp/ui/ui_public.h` -> `src/codemp/ui/ui_public_h.rs`
- `oracle/codemp/client/FxSystem.h` -> `src/codemp/client/FxSystem_h.rs`

Possible but heavier follow-ups:

- `oracle/codemp/qcommon/cm_local.h` -> `src/codemp/qcommon/cm_local_h.rs`
- `oracle/codemp/client/FxScheduler.h` -> `src/codemp/client/FxScheduler_h.rs`
- `oracle/codemp/client/FxPrimitives.h` -> `src/codemp/client/FxPrimitives_h.rs`

Continue avoiding broad renderer/client/source files and large dependency-heavy headers such as `server.h`, `client.h`, `qfiles.h`, `sparc.h`, `hstring.h`, `snd_local.h`, and `ui_shared.h`.

## State

- Branch: `full-port`
- HEAD before this handoff commit: `ed0bfc2 port oracle/codemp/qcommon/RoffSystem.h`
- Working tree was clean before this handoff file was created.
