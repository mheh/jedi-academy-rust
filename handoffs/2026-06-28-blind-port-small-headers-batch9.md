# Blind Port Handoff: small headers batch 9

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1062
- oracle/code: 646
- oracle/codemp: 416

## Delegated

- `oracle/codemp/client/FXExport.h` -> `src/codemp/client/FXExport_h.rs`
- `oracle/codemp/client/FxScheduler.h` -> `src/codemp/client/FxScheduler_h.rs`
- `oracle/codemp/client/FxPrimitives.h` -> `src/codemp/client/FxPrimitives_h.rs`
- `oracle/codemp/qcommon/cm_local.h` -> `src/codemp/qcommon/cm_local_h.rs`
- `oracle/codemp/client/snd_local.h` -> `src/codemp/client/snd_local_h.rs`

## Model Policy Used

- All file workers used `gpt-5.4-mini`.
- `FXExport.h`, `FxScheduler.h`, `FxPrimitives.h`, and `cm_local.h` used `reasoning_effort: "high"`.
- `snd_local.h` used `reasoning_effort: "medium"`.

## Committed In This Batch

- `fbb6db6 port oracle/codemp/client/FXExport.h`
- `b6689d7 port oracle/codemp/client/snd_local.h`
- `33b772b port oracle/codemp/qcommon/cm_local.h`
- `08fcb73 port oracle/codemp/client/FxScheduler.h`
- `afe3872 port oracle/codemp/client/FxPrimitives.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- `FXExport_h.rs` uses a local opaque `refdef_t` stub.
- `snd_local_h.rs` uses local `cvar_t`, `ALuint`, and `sboolean` aliases/stubs; earlier sound headers still keep their own local sound stubs.
- `cm_local_h.rs` uses feature-gated local `NotSoShort` and `SPARC<T>` stubs for unported Xbox `sparc.h`; `CM_GetShaderInfo` overloads are split into suffixed Rust names. The Xbox `cNode_t::planeNum` field was corrected to `c_int` before commit.
- `FxScheduler_h.rs` uses Vec-backed local STL container stand-ins for inline methods, suffixes C++ overloads, and leaves `.cpp`-backed methods as `todo!` stubs.
- `FxPrimitives_h.rs` uses local support stubs for `CGhoul2Info_v`, `miniRefEntity_t`, `SFxHelper`, and `theFxHelper`; these overlap conceptually with `FxSystem_h.rs` but were left local for blind single-file isolation.

## Next Recommended Batch

Continue using cheaper workers unless a file looks unusually risky:

- small headers: `gpt-5.4-mini`, `reasoning_effort: "medium"`
- medium C++ headers: `gpt-5.4-mini`, `reasoning_effort: "high"` or `gpt-5.4`
- large `.c` / `.cpp` files or dependency-heavy ports: inherited/current model or `gpt-5.4`

Good next candidates:

- `oracle/codemp/client/OpenAL/al.h` -> `src/codemp/client/OpenAL/al_h.rs`
- `oracle/codemp/client/OpenAL/alc.h` -> `src/codemp/client/OpenAL/alc_h.rs`
- `oracle/codemp/client/OpenAL/altypes.h` -> `src/codemp/client/OpenAL/altypes_h.rs`
- `oracle/codemp/client/OpenAL/alctypes.h` -> `src/codemp/client/OpenAL/alctypes_h.rs`
- `oracle/codemp/client/OpenAL/alu.h` -> `src/codemp/client/OpenAL/alu_h.rs`

Possible but heavier follow-ups:

- `oracle/codemp/client/FXExport.cpp` -> `src/codemp/client/FXExport.rs`
- `oracle/codemp/client/FxPrimitives.cpp` -> `src/codemp/client/FxPrimitives.rs`
- `oracle/codemp/client/FxScheduler.cpp` -> `src/codemp/client/FxScheduler.rs`
- `oracle/codemp/client/FxSystem.cpp` -> `src/codemp/client/FxSystem.rs`
- `oracle/codemp/client/FxUtil.cpp` -> `src/codemp/client/FxUtil.rs`

Continue avoiding broad renderer/client/source files and large dependency-heavy headers such as `server.h`, `client.h`, `qfiles.h`, `sparc.h`, `hstring.h`, and `ui_shared.h`.

## State

- Branch: `full-port`
- HEAD before this handoff commit: `afe3872 port oracle/codemp/client/FxPrimitives.h`
- Working tree was clean before this handoff file was created.
