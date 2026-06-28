---
name: blind-port-orchestrator
description: Orchestrate blind high-throughput Jedi Academy source porting from oracle/code and oracle/codemp to src/ without building or testing. Use when porting many remaining oracle/code or oracle/codemp files, finding unpaired source files, delegating file batches to agents, context approaches 150k tokens, or committing one translated file at a time.
---

# Blind Port Orchestrator

## Purpose

Drive fast mechanical translation of unported `oracle/` files into Rust by delegating small file sets to fresh agents.

This skill is for orchestration only. File agents port blindly: no build, no test, no cargo check, no global formatting.

## Required context

Read `PORTING_STYLE.md` first. It is the bible.

Use `oracle/` as read-only source truth. Do not modify `oracle/`.

Existing `src/` files are considered complete. Missing `oracle/code/` and `oracle/codemp/` files are valid full-port work.

## Pair discovery

Run `scripts/compare-src-oracle.sh` before delegation to find `oracle/` files without paired `src/` files.

The script compares both source roots:

- `oracle/codemp/...` against `src/codemp/...`
- `oracle/code/...` against `src/code/...`

Expected mappings:

- `.c` / `.cpp` source files become `.rs`
- `.h` headers become `_h.rs` unless an existing Rust file establishes another mapping
- directory structure is preserved 1:1
- awkward original directory names may remain on disk and be exposed with Rust-friendly module aliases

Use the pairing output to create batches. Prefer files with few includes and few downstream dependencies first.

## Delegation loop

Delegate work to fresh agents in small batches.

Each agent receives only:

- `PORTING_STYLE.md`
- the assigned `oracle/...` file(s)
- the destination `src/...` path(s)
- directly required neighboring Rust modules or stubs
- the instruction: "port blindly; do not build or test"

Batch size defaults:

- one large file per agent
- two to five small files per agent
- headers may be batched only when they are tightly coupled

When orchestrator context reaches roughly 150k tokens, stop delegating and run this skill again from a compact handoff.

## Agent instructions

Tell each file agent:

- Translate mechanically, preserving C names, table order, control flow, globals, raw pointers, casts, and dangerous behavior.
- Do not make idiomatic Rust improvements.
- Do not build, test, run cargo check, run cargo fmt, or edit unrelated files.
- Do not modify `oracle/`.
- Keep unported dependencies as explicit stubs only when needed to make the translated file structurally coherent.
- Report the exact files changed, symbols ported, missing dependencies, and any intentional deviations.

## Commit policy

Commit at a per-file level.

One commit should contain:

- one translated source/header file, or
- one tightly coupled source/header pair, or
- the minimum module-wiring needed for that file

Commit message format:

`port <oracle-relative-path>`

Examples:

- `port oracle/code/qcommon/q_math.cpp`
- `port oracle/codemp/server/sv_game.cpp`
- `port oracle/code/game/g_local.h`

Do not combine unrelated files in one commit.

Do not include coauthor trailers or generated-by trailers in commit messages.

## Handoff

When stopping, write a compact handoff under `handoffs/`.

Include:

- last pairing output location or command
- files already delegated
- files committed
- files returned with unresolved dependencies
- next recommended batch
- current branch and HEAD
- any agent failures or files that should be retried smaller

Keep handoff factual and short. Point to files and commits; do not paste source.

## Hard prohibitions

Do not build.
Do not test.
Do not run `cargo check`.
Do not run `cargo fmt`.
Do not modify `oracle/`.
Do not refactor for idiomatic Rust.
Do not add coauthor trailers to commits.
Do not let orchestrator context grow past the handoff threshold just to finish a batch.
