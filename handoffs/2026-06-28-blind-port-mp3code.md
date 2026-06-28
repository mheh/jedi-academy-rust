# Blind Port Handoff: mp3code

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Post-batch summary:

- Oracle files without a paired Rust file: 1141
- oracle/code: 646
- oracle/codemp: 495

## Delegated

- `oracle/codemp/mp3code/small_header.h` -> `src/codemp/mp3code/small_header_h.rs`
- `oracle/codemp/mp3code/copyright.h` -> `src/codemp/mp3code/copyright_h.rs`
- `oracle/codemp/mp3code/jdw.h` -> `src/codemp/mp3code/jdw_h.rs`
- `oracle/codemp/mp3code/config.h` -> `src/codemp/mp3code/config_h.rs`
- `oracle/codemp/mp3code/port.h` -> `src/codemp/mp3code/port_h.rs`
- `oracle/codemp/mp3code/mhead.h` -> `src/codemp/mp3code/mhead_h.rs`
- `oracle/codemp/mp3code/mhead.c` -> `src/codemp/mp3code/mhead.rs`

## Committed

- `20985cb port oracle/codemp/mp3code/small_header.h`
- `ebab31a port oracle/codemp/mp3code/copyright.h`
- `329f3c5 port oracle/codemp/mp3code/jdw.h`
- `5eded4b port oracle/codemp/mp3code/config.h`
- `f0209b9 port oracle/codemp/mp3code/port.h`
- `ac36a91 port oracle/codemp/mp3code/mhead.c`

## Unresolved Dependencies

- `mhead_h.rs` declares unresolved decoder functions from `mhead.h`: `audio_decode_init`, `audio_decode_info`, `audio_decode`, `audio_decode8_init`, `audio_decode8_info`, `audio_decode8`.
- These are expected to be supplied by later MP3 decoder body ports.

## Next Recommended Batch

Stay in `oracle/codemp/mp3code/` and port the small dependency/table headers before decoder bodies:

- `oracle/codemp/mp3code/mp3struct.h`
- `oracle/codemp/mp3code/l3.h`
- `oracle/codemp/mp3code/htable.h`
- `oracle/codemp/mp3code/tableawd.h`

After those, split decoder C files into small batches. Good candidates:

- `oracle/codemp/mp3code/towave.c`
- `oracle/codemp/mp3code/wavep.c`
- `oracle/codemp/mp3code/cupini.c`

## State

- Branch: `full-port`
- HEAD before this handoff commit: `ac36a91`
- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
