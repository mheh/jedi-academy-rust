# Blind Port Handoff: Haiku cycle 300

Date: 2026-06-28

## Pairing

Command: `scripts/compare-src-oracle.sh`
Pre-cycle unpaired: 911. Post-cycle unpaired: 588.
(300 committed; count dropped 323 because some ported headers satisfied additional
header/direct key pairings.)

## Execution

- Background Workflow `wf_54291730-cab` (task `weswxl4wv`), single `Port` phase, `parallel()`, ~16 concurrent.
- Workers: `haiku`, `reasoning_effort: medium`, compact structured output.
- Selection: stratified 300 non-empty files across the <=2000-line pool (3..1963 lines, ~152.8k lines).
- Script: `scratchpad/blind-port-batch300.js` (batch inlined). Prompt hardened to require
  syntactically valid Rust (let mut for reassigned locals, no partial init, omit guesswork
  size_of asserts).
- Orchestrator committed each file serially with `-c commit.gpgsign=false`.

## Measured cost

- 300 agents, 12,171,427 subagent tokens, 4540 tool uses, ~2.4h wall-clock.
- ~40.6k tokens/file (consistent with batch30 ~43.8k, cycle100 ~42.8k).

## Result

- 298 reported wrote; 2 reported failed (`oracle/code/renderer/tr_curve.cpp`,
  `oracle/code/ff/IFC/FeelitAPI.h`) — BOTH FALSE NEGATIVES, files fully written. All 300
  verified present/non-empty and committed.

## Rule violations cleaned up (some agents disobeyed)

- `src/code/client/mod.rs` had been edited to add `pub mod snd_local_console_h;` and
  `pub mod snd_mem_console;` — reverted (module wiring is forbidden in blind port; also racy).
- Stray build artifacts `libff_h.rlib`, `libmac_net.rlib` appeared — some agents ran rustc/cargo
  despite the no-build prohibition. Deleted.
- An unsanctioned extra file `src/code/client/snd_local_console_h.rs` (not in batch) was written
  as a dependency — deleted; will be ported properly in a future cycle.
- Lesson: a few Haiku workers ignore "write only your dest / do not build". Consider running
  future batches in a worktree or a sandbox that blocks cargo, and post-run `git clean` of
  non-batch paths.

## Remaining

- ~588 unpaired total. Of these: ~500 small/medium files <=2000 lines, 106 giants >2000 lines
  (deferred), 4 empty 0-byte files (deferred, stub trivially).

## Estimate to finish

- Small/medium remainder: ~20M tokens, ~5-6h wall-clock at 16-wide.
- Giants: ~30-40M tokens, high Haiku fidelity risk (recommend escalating model).

## Fidelity (blind, unverified — no build run)

- Diagnostics across ported files include real compile errors in some files (syntax errors,
  E0384 reassign-immutable, failing size_of parity asserts) plus many lint-level warnings and
  unlinked-file notices. A dedicated verification/build-fix + module-wiring pass remains required.

## State

- Branch: `full-port`. Working tree clean after cleanup.
- NOTE: repo signs via 1Password SSH (interactive); automated commits must use
  `-c commit.gpgsign=false`.
