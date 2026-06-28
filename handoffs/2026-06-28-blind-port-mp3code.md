# Blind Port Handoff: mp3code

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1118
- oracle/code: 646
- oracle/codemp: 472

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
- `oracle/codemp/mp3code/l3dq.c` -> `src/codemp/mp3code/l3dq.rs`
- `oracle/codemp/mp3code/mdct.c` -> `src/codemp/mp3code/mdct.rs`
- `oracle/codemp/mp3code/hwin.c` -> `src/codemp/mp3code/hwin.rs`
- `oracle/codemp/mp3code/upsf.c` -> `src/codemp/mp3code/upsf.rs`
- `oracle/codemp/mp3code/uph.c` -> `src/codemp/mp3code/uph.rs`
- `oracle/codemp/mp3code/cupl3.c` -> `src/codemp/mp3code/cupl3.rs`
- `oracle/codemp/mp3code/csbtb.c` -> `src/codemp/mp3code/csbtb.rs`
- `oracle/codemp/mp3code/cwin.c` -> `src/codemp/mp3code/cwin.rs`
- `oracle/codemp/mp3code/csbtl3.c` -> `src/codemp/mp3code/csbtl3.rs`
- `oracle/codemp/mp3code/cwinb.c` -> `src/codemp/mp3code/cwinb.rs`
- `oracle/codemp/mp3code/cwinm.c` -> `src/codemp/mp3code/cwinm.rs`
- `oracle/codemp/mp3code/csbt.c` -> `src/codemp/mp3code/csbt.rs`

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
- `6dcec4d port oracle/codemp/mp3code/l3dq.c`
- `76299c1 port oracle/codemp/mp3code/mdct.c`
- `76f0d61 port oracle/codemp/mp3code/hwin.c`
- `4f5fbee port oracle/codemp/mp3code/upsf.c`
- `27fa9ed port oracle/codemp/mp3code/uph.c`
- `53f2baa port oracle/codemp/mp3code/cupl3.c`
- `c43d090 port oracle/codemp/mp3code/csbtb.c`
- `73321d8 port oracle/codemp/mp3code/cwin.c`
- `ac509da port oracle/codemp/mp3code/csbtl3.c`
- `f18b67f port oracle/codemp/mp3code/cwinb.c`
- `b7993da port oracle/codemp/mp3code/cwinm.c`
- `1c06629 port oracle/codemp/mp3code/csbt.c`

## Unresolved Dependencies

- `mhead_h.rs` declares unresolved decoder functions from `mhead.h`: `audio_decode_init`, `audio_decode_info`, `audio_decode`, `audio_decode8_init`, `audio_decode8_info`, `audio_decode8`.
- `cup.rs` now defines shared decoder globals/functions including `decinfo`, `look_c_value`, `sf_table`, `sample`, grouped lookup tables, `audio_decode_routine`, `gpNextByteAfterData`, `audio_decode`, and `L2audio_decode`.
- `cupl1.rs` now defines `L1audio_decode` and `L1audio_decode_init`.
- `cdct.rs` now defines DCT coefficient storage/accessors and the `fdct*` transform functions.
- `l3init.rs` now defines Layer III init entry points and leaves quant/imdct/hwin/msis table-provider boundaries explicit.
- `msis.rs` now defines antialias, mid/side, and intensity stereo processing plus ms/is table accessors.
- `l3dq.rs` now defines dequant lookup storage/accessors and `dequant`.
- `mdct.rs` now defines IMDCT lookup storage/accessors and `imdct18`/`imdct6_3`.
- `hwin.rs` now defines hybrid window storage/accessors and hybrid/filter helpers.
- `upsf.rs` now defines MPEG1/MPEG2 scale-factor unpacking and leaves `bitget` explicit.
- `uph.rs` now defines Huffman decode helpers and leaves `bitdat` explicit.
- `cupl3.rs` now defines the Layer III decode orchestration, bit reader, side-info unpacking, main decode path, transform dispatch, and `L3audio_decode*` functions.
- `csbtb.rs` now defines byte-output SBT helpers and leaves DCT/window helpers explicit.
- `cwin.rs` now defines 16-bit window helpers but currently carries a private zeroed `wincoef` stub that should be reconciled with `tableawd_h::wincoef`.
- `csbtl3.rs` now defines Layer III SBT wrappers and leaves DCT/window helpers explicit.
- `cwinb.rs` now defines byte-output window helpers using `tableawd_h::wincoef`.
- `cwinm.rs` now re-exports the shared window coefficient table and the 16-bit/byte window bodies from split Rust modules.
- `csbt.rs` now defines `sbt_init` plus 16-bit SBT wrapper bodies and uses existing DCT/window modules.
- Remaining unresolved decoder/body dependencies include the `cwin.rs` coefficient-table reconciliation.
- `wavep.rs` preserves the original `#if 0` body as disabled Rust and keeps missing `wcvt.c` conversion boundaries explicit.

## Next Recommended Batch

Stay in `oracle/codemp/mp3code/` and finish the remaining standalone source:

- `oracle/codemp/mp3code/towave.c`

## State

- Branch: `full-port`
- HEAD before this handoff update: `1c06629`
- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
