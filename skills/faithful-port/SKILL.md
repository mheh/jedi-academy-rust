---
name: faithful-port
description: Faithful single-agent C/C++ -> Rust port of one unported oracle file using import-trust (no stubs). Each file is ported by one capable agent (Sonnet) that translates #include directives into glob `use crate::...::foo_h::*;` imports, trusts external symbols to exist without verifying them, preserves comments verbatim, and writes large files incrementally. Use when porting an oracle/code or oracle/codemp source file to src/ faithfully without fabricating stubs, when continuing the import-trust port, or when the blind-port stub approach produced opaque placeholders that need replacing.
---

# Faithful Port

## Purpose

Port unported `oracle/` files to Rust **faithfully** with **one capable agent per file** and an
**import-trust** model: the agent never defines or stubs external types — it `use`-imports them on faith
from the `#include` paths and translates the source body exactly. No build, no test.

This supersedes the stub-based guidance in `skills/blind-port-orchestrator/SKILL.md` and the
"unresolved dependency stubs" line in `PORTING_STYLE.md`. Where they say "stub", this skill says
"trust the import".

## Why this exists

The cheap fan-out blind port had each worker fabricate its own opaque stubs
(`#[repr(C)] struct gentity_t { _p: [u8; 0] }`), so every file invented incompatible placeholder types
and nothing could ever cohere. This skill instead produces files that read like the finished article:
real translated bodies plus glob imports of the real (eventual) modules.

## Required reading

- `PORTING_STYLE.md` — the base rules (faithfulness, comments, repr(C), core::ffi, static mut, etc.).
- `skills/faithful-port/AGENT_PROMPT.md` — the canonical per-file agent prompt. This is the artifact you
  hand to each worker; do not improvise a different prompt.

Use `oracle/` as read-only source truth. Existing `src/` files are considered complete.

## Pair discovery

Run `scripts/compare-src-oracle.sh` to list `oracle/` files with no paired `src/` file.
Mappings: `.c`/`.cpp` -> `.rs`; `.h` -> `_h.rs`; directory structure mirrored 1:1.

## The import-trust rule (the core idea)

For every symbol used but not defined in the file being ported, the agent does NOT define or stub it.
Instead it imports it, trusting it exists:

- Each `#include "dir/foo.h"` becomes `use crate::<mirrored-dir>::foo_h::*;` (glob), with the directory
  mirrored under `crate::` (src tree), `/` -> `::`, and `.h` -> `_h`. Includes are resolved relative to the
  ported file's directory, then written as absolute `crate::...` paths.
- The agent never opens those modules to verify their contents and never fabricates them.
- System/libc includes (`<...>`) are not modules — use `core::ffi` / `extern "C"` instead.

Result invariant: a finished file contains **zero** `[u8; 0]` placeholders and **zero** fabricated
`#[repr(C)]` definitions of external engine types.

## Delegation

One worker agent per file. Default model: **Sonnet** (reliable at faithful translation and at the
incremental multi-write needed for large files). Use a stronger model only for unusually gnarly files.

For each file:
1. Compute `{{ORACLE_PATH}}`, `{{DEST_PATH}}`, `{{LINES}}`.
2. Fill those into `AGENT_PROMPT.md` and pass the result verbatim as the agent prompt.
3. Run agents in the background; small batches, not one long-running mega-run.

### Large files and the output cap

A single response cannot exceed ~32k output tokens, so any file over ~1000 lines must be written
incrementally (Write the first portion, then Edit-append the rest in order). The prompt already instructs
this. Do not chunk such files across multiple agents unless a single agent genuinely cannot finish — the
single-agent whole-file path keeps imports and definitions coherent.

## Verify before commit

For each returned file:
- `grep -c '\[u8; 0\]' <dest>` must be 0 (no opaque stubs).
- `grep -c 'r#self' <dest>` must be 0.
- Confirm each `#include` in the source has a matching `use crate::...::*_h::*;` (or a justified omission).
- Parse-gate: copy to a temp path and run `rustfmt --edition 2021 <tmp>`; exit 0 means it parses. rustfmt
  is a syntax check only — do NOT write the formatted result back (preserve port style). Apply the cheap
  safety-net fixes if needed: `r#self` -> `self_`, `//!` -> `// `, `/**` -> `/* `.
- Spot-check completeness: every function/item in the source is present (no elision).

## Commit policy

One commit per file: `port <oracle-relative-path>`. No coauthor/generated-by trailers.

IMPORTANT: this repo signs commits via interactive 1Password SSH (`commit.gpgsign=true`,
`gpg.format=ssh`), which blocks non-interactive commits. Commit with signing disabled:
`git -c commit.gpgsign=false commit -m "port <path>"`.

Commit any `handoffs/` updates separately with a non-`port` message.

## Handoff

When pausing, write a short handoff under `handoffs/`: pairing command + counts, files ported/committed,
any files that failed or need a stronger model, current branch/HEAD, and the next recommended batch.

## Hard prohibitions

Do not build. Do not test. Do not run `cargo check`/`cargo fmt` as part of porting (rustfmt is allowed
ONLY as a throwaway parse-gate during verify). Do not modify `oracle/`. Do not fabricate or stub external
types. Do not add coauthor trailers. Do not start a long-running mega-run; prefer small batches.
