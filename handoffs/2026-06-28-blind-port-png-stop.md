# Blind Port Handoff: png stop

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1116
- oracle/code: 646
- oracle/codemp: 470

## Delegated

- `oracle/codemp/mp3code/towave.c` -> `src/codemp/mp3code/towave.rs`
- `oracle/codemp/png/png.cpp` -> `src/codemp/png/png.rs`

## Committed

- `aac7d2c port oracle/codemp/mp3code/towave.c`
- `2a0a0fd update blind port mp3code handoff`
- `cbffde2 port oracle/codemp/png/png.cpp`

## Unresolved Dependencies

- `oracle/codemp/mp3code/` now has paired Rust files for every source/header in that directory.
- `towave.rs` defines MP3 stream globals plus validation/header/unpack and streaming decode/rewind entry points.
- `png.rs` defines PNG load/save/filter/unfilter/pack/unpack entry points and keeps engine dependencies explicit: `Z_Malloc`, `Z_Free`, `FS_FOpenFileWrite`, `FS_Write`, `FS_FCloseFile`, `FS_ReadFile`, `FS_FreeFile`, `Com_Printf`, plus libc `memcmp`/`memcpy`/`memset`.
- `png.rs` routes zlib calls through existing `src/codemp/zlib32/zip_h.rs`.
- `png.rs` uses a local `BigLong` helper and hardcodes `TAG_TEMP_PNG = 32` from the non-Xbox `tags.h` order.
- No agent failures.

## Next Recommended Batch

Good small standalone candidates:

- `oracle/codemp/qcommon/chash.h` -> `src/codemp/qcommon/chash_h.rs`
- `oracle/codemp/qcommon/fixedmap.h` -> `src/codemp/qcommon/fixedmap_h.rs`
- `oracle/codemp/game/bg_strap.h` -> `src/codemp/game/bg_strap_h.rs`
- `oracle/codemp/game/chars.h` -> `src/codemp/game/chars_h.rs`

Avoid broad `qcommon/*.cpp` and renderer/client files until smaller headers around them are paired.

## State

- Branch: `full-port`
- HEAD before this handoff commit: `cbffde2 port oracle/codemp/png/png.cpp`
- Working tree clean before handoff file creation.
- No builds, tests, `cargo check`, or formatting were run.
