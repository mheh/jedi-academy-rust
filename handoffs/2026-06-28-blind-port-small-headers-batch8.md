# Blind Port Handoff: small headers batch 8

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1067
- oracle/code: 646
- oracle/codemp: 421

## Delegated

- `oracle/codemp/client/snd_mp3.h` -> `src/codemp/client/snd_mp3_h.rs`
- `oracle/codemp/client/snd_local_console.h` -> `src/codemp/client/snd_local_console_h.rs`
- `oracle/codemp/qcommon/GenericParser2.h` -> `src/codemp/qcommon/GenericParser2_h.rs`
- `oracle/codemp/ui/ui_public.h` -> `src/codemp/ui/ui_public_h.rs`
- `oracle/codemp/client/FxSystem.h` -> `src/codemp/client/FxSystem_h.rs`

## Committed In This Batch

- `ce671a5 port oracle/codemp/client/snd_mp3.h`
- `02170dc port oracle/codemp/client/snd_local_console.h`
- `32774f2 port oracle/codemp/ui/ui_public.h`
- `640aa79 port oracle/codemp/qcommon/GenericParser2.h`
- `7f44af4 port oracle/codemp/client/FxSystem.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- `snd_mp3_h.rs` uses local `sboolean`, `sfx_t`, and `channel_t` stubs for unported sound-local dependencies; C++ defaults are explicit required parameters.
- `snd_local_console_h.rs` uses local `ALuint` and `cvar_t` stubs.
- `ui_public_h.rs` added no local stubs; enum values are represented as `c_int` constants.
- `GenericParser2_h.rs` represents C++ inheritance with `base` fields, overloads with suffixed Rust names, and non-inline bodies from `GenericParser2.cpp` as explicit `todo!` stubs.
- `FxSystem_h.rs` uses local opaque/client renderer stubs and preserves C++ inline helper behavior; non-inline methods from `FxSystem.cpp` are explicit `todo!` stubs.

## Next Recommended Batch

Use cheaper workers for the next batch unless a file looks unusually risky:

- small headers: `gpt-5.4-mini`, `reasoning_effort: "medium"`
- medium C++ headers: `gpt-5.4-mini`, `reasoning_effort: "high"` or `gpt-5.4`
- large `.c` / `.cpp` files or dependency-heavy ports: inherited/current model or `gpt-5.4`

Good next candidates:

- `oracle/codemp/client/FXExport.h` -> `src/codemp/client/FXExport_h.rs`
- `oracle/codemp/client/FxScheduler.h` -> `src/codemp/client/FxScheduler_h.rs`
- `oracle/codemp/client/FxPrimitives.h` -> `src/codemp/client/FxPrimitives_h.rs`
- `oracle/codemp/qcommon/cm_local.h` -> `src/codemp/qcommon/cm_local_h.rs`
- `oracle/codemp/client/snd_local.h` -> `src/codemp/client/snd_local_h.rs`

Possible but heavier follow-ups:

- `oracle/codemp/client/FxSystem.cpp` -> `src/codemp/client/FxSystem.rs`
- `oracle/codemp/qcommon/GenericParser2.cpp` -> `src/codemp/qcommon/GenericParser2.rs`
- `oracle/codemp/client/snd_mp3.cpp` -> `src/codemp/client/snd_mp3.rs`

Continue avoiding broad renderer/client/source files and large dependency-heavy headers such as `server.h`, `client.h`, `qfiles.h`, `sparc.h`, `hstring.h`, and `ui_shared.h`.

## State

- Branch: `full-port`
- HEAD before this handoff commit: `7f44af4 port oracle/codemp/client/FxSystem.h`
- Working tree was clean before this handoff file was created.
