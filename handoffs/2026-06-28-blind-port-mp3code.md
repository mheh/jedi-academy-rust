# Blind Port Handoff: mp3code

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1130
- oracle/code: 646
- oracle/codemp: 484

## Delegated

- `oracle/codemp/mp3code/small_header.h` -> `src/codemp/mp3code/small_header_h.rs`
- `oracle/codemp/mp3code/copyright.h` -> `src/codemp/mp3code/copyright_h.rs`
- `oracle/codemp/mp3code/jdw.h` -> `src/codemp/mp3code/jdw_h.rs`
- `oracle/codemp/mp3code/config.h` -> `src/codemp/mp3code/config_h.rs`
- `oracle/codemp/mp3code/port.h` -> `src/codemp/mp3code/port_h.rs`
- `oracle/codemp/mp3code/mhead.h` -> `src/codemp/mp3code/mhead_h.rs`
- `oracle/codemp/mp3code/mhead.c` -> `src/codemp/mp3code/mhead.rs`
- `oracle/codemp/mp3code/l3.h` -> `src/codemp/mp3code/l3_h.rs`
- `oracle/codemp/mp3code/mp3struct.h` -> `src/codemp/mp3code/mp3struct_h.rs`
- `oracle/codemp/mp3code/htable.h` -> `src/codemp/mp3code/htable_h.rs`
- `oracle/codemp/mp3code/tableawd.h` -> `src/codemp/mp3code/tableawd_h.rs`
- `oracle/codemp/mp3code/cupini.c` -> `src/codemp/mp3code/cupini.rs`
- `oracle/codemp/mp3code/wavep.c` -> `src/codemp/mp3code/wavep.rs`
- `oracle/codemp/mp3code/cupl1.c` -> `src/codemp/mp3code/cupl1.rs`
- `oracle/codemp/mp3code/cup.c` -> `src/codemp/mp3code/cup.rs`
- `oracle/codemp/mp3code/cdct.c` -> `src/codemp/mp3code/cdct.rs`
- `oracle/codemp/mp3code/l3init.c` -> `src/codemp/mp3code/l3init.rs`
- `oracle/codemp/mp3code/msis.c` -> `src/codemp/mp3code/msis.rs`

## Committed

- `20985cb port oracle/codemp/mp3code/small_header.h`
- `ebab31a port oracle/codemp/mp3code/copyright.h`
- `329f3c5 port oracle/codemp/mp3code/jdw.h`
- `5eded4b port oracle/codemp/mp3code/config.h`
- `f0209b9 port oracle/codemp/mp3code/port.h`
- `ac36a91 port oracle/codemp/mp3code/mhead.c`
- `860a98c port oracle/codemp/mp3code/l3.h`
- `f0ee1e7 port oracle/codemp/mp3code/mp3struct.h`
- `d33ddcd port oracle/codemp/mp3code/htable.h`
- `c836a4a port oracle/codemp/mp3code/tableawd.h`
- `810371b port oracle/codemp/mp3code/cupini.c`
- `28b3c0b port oracle/codemp/mp3code/wavep.c`
- `8285269 port oracle/codemp/mp3code/cupl1.c`
- `8007a8c port oracle/codemp/mp3code/cup.c`
- `825abc6 port oracle/codemp/mp3code/cdct.c`
- `ec5e771 port oracle/codemp/mp3code/l3init.c`
- `ba5abbb port oracle/codemp/mp3code/msis.c`

## Unresolved Dependencies

- `mhead_h.rs` declares unresolved decoder functions from `mhead.h`: `audio_decode_init`, `audio_decode_info`, `audio_decode`, `audio_decode8_init`, `audio_decode8_info`, `audio_decode8`.
- `cup.rs` now defines shared decoder globals/functions including `decinfo`, `look_c_value`, `sf_table`, `sample`, grouped lookup tables, `audio_decode_routine`, `gpNextByteAfterData`, `audio_decode`, and `L2audio_decode`.
- `cupl1.rs` now defines `L1audio_decode` and `L1audio_decode_init`.
- `cdct.rs` now defines DCT coefficient storage/accessors and the `fdct*` transform functions.
- `l3init.rs` now defines Layer III init entry points and leaves quant/imdct/hwin/msis table-provider boundaries explicit.
- `msis.rs` now defines antialias, mid/side, and intensity stereo processing plus ms/is table accessors.
- Remaining unresolved decoder/body dependencies include `sbt_*`, `sbtB_*`, `sbt_init`, `L3audio_decode_init`, `L3audio_decode`, and remaining Layer III support/transform bodies.
- `wavep.rs` preserves the original `#if 0` body as disabled Rust and keeps missing `wcvt.c` conversion boundaries explicit.

## Next Recommended Batch

Stay in `oracle/codemp/mp3code/` and continue decoder bodies in dependency-shaped batches.

Good next batch:

- `oracle/codemp/mp3code/l3dq.c`
- `oracle/codemp/mp3code/mdct.c`
- `oracle/codemp/mp3code/hwin.c`
- `oracle/codemp/mp3code/uph.c`
- `oracle/codemp/mp3code/upsf.c`

Then `oracle/codemp/mp3code/cupl3.c` as a dedicated large-file port once more Layer III helpers are present.

Then transform/window files:

- `oracle/codemp/mp3code/csbt.c`
- `oracle/codemp/mp3code/csbtb.c`
- `oracle/codemp/mp3code/csbtl3.c`
- `oracle/codemp/mp3code/cwin.c`
- `oracle/codemp/mp3code/cwinb.c`
- `oracle/codemp/mp3code/cwinm.c`

Standalone candidates remain:

- `oracle/codemp/mp3code/towave.c`

## State

- Branch: `full-port`
- HEAD before this handoff update: `ba5abbb`
- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
