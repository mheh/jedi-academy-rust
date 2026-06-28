# Blind Port Handoff: small headers batch 3

Date: 2026-06-28

## Pairing

Last pairing command:

`scripts/compare-src-oracle.sh`

Latest post-batch summary:

- Oracle files without a paired Rust file: 1091
- oracle/code: 646
- oracle/codemp: 445

## Delegated

- `oracle/codemp/encryption/encryption.h` -> `src/codemp/encryption/encryption_h.rs`
- `oracle/codemp/game/be_ai_chat.h` -> `src/codemp/game/be_ai_chat_h.rs`
- `oracle/codemp/game/inv.h` -> `src/codemp/game/inv_h.rs`
- `oracle/codemp/game/match.h` -> `src/codemp/game/match_h.rs`
- `oracle/codemp/qcommon/sstring.h` -> `src/codemp/qcommon/sstring_h.rs`

## Committed In This Batch

- `8da9408 port oracle/codemp/encryption/encryption.h`
- `7cc7180 port oracle/codemp/game/be_ai_chat.h`
- `64864ea port oracle/codemp/game/inv.h`
- `ae4d3c0 port oracle/codemp/game/match.h`
- `183ccb4 port oracle/codemp/qcommon/sstring.h`

## Unresolved Dependencies

- No agent failures.
- No builds, tests, `cargo check`, or formatting were run.
- `be_ai_chat_h.rs` declares chat/match helper functions as extern only; implementations remain unported.
- `sstring_h.rs` ports the C++ template as a const-generic Rust struct, with overload/operator names made explicit (`sstring_copy`, `sstring_char`, `operator_eq`, etc.).
- `encryption_h.rs` preserves the commented-out `_ENCRYPTION_` define as comments only.

## Next Recommended Batch

Good next small candidates:

- `oracle/codemp/qcommon/stringed_ingame.h` -> `src/codemp/qcommon/stringed_ingame_h.rs`
- `oracle/codemp/qcommon/cm_randomterrain.h` -> `src/codemp/qcommon/cm_randomterrain_h.rs`
- `oracle/codemp/qcommon/cm_patch.h` -> `src/codemp/qcommon/cm_patch_h.rs`
- `oracle/codemp/qcommon/files.h` -> `src/codemp/qcommon/files_h.rs`
- `oracle/codemp/qcommon/unzip.h` -> `src/codemp/qcommon/unzip_h.rs`

Avoid large or dependency-heavy headers such as `server.h`, `qfiles.h`, `sparc.h`, and STL-heavy `hstring.h` until more small qcommon/game headers are paired.

## State

- Branch: `full-port`
- HEAD: `183ccb4 port oracle/codemp/qcommon/sstring.h`
- Working tree has untracked handoff files only: `handoffs/2026-06-28-blind-port-small-headers-batch2.md` and this file.
