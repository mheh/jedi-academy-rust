# Blind Port Handoff: mp3code

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1136
- oracle/code: 646
- oracle/codemp: 490

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

## Unresolved Dependencies

- `mhead_h.rs` declares unresolved decoder functions from `mhead.h`: `audio_decode_init`, `audio_decode_info`, `audio_decode`, `audio_decode8_init`, `audio_decode8_info`, `audio_decode8`.
- `cupini.rs` now defines `audio_decode_init`, `audio_decode_info`, and `decode_table_init`.
- Remaining unresolved decoder/body dependencies include `decinfo`, `look_c_value`, `sf_table`, `sample`, grouped lookup tables, `audio_decode_routine`, `sbt_*`, `sbtB_*`, `L1audio_decode_init`, `L3audio_decode_init`, `L1audio_decode`, `L2audio_decode`, `L3audio_decode`, and `sbt_init`.

## Next Recommended Batch

Stay in `oracle/codemp/mp3code/` and continue decoder bodies in dependency-shaped batches.

Good next batch:

- `oracle/codemp/mp3code/cup.c`
- `oracle/codemp/mp3code/cupl1.c`
- `oracle/codemp/mp3code/cupl3.c`

Then split transform/window files:

- `oracle/codemp/mp3code/csbt.c`
- `oracle/codemp/mp3code/csbtb.c`
- `oracle/codemp/mp3code/csbtl3.c`
- `oracle/codemp/mp3code/cwin.c`
- `oracle/codemp/mp3code/cwinb.c`
- `oracle/codemp/mp3code/cwinm.c`

Smaller standalone candidates remain:

- `oracle/codemp/mp3code/wavep.c`
- `oracle/codemp/mp3code/towave.c`

## State

- Branch: `full-port`
- HEAD before this handoff update: `810371b`
- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
