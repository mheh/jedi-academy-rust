# Blind Port Handoff: Haiku cycle 100

Date: 2026-06-28

## Pairing

Command: `scripts/compare-src-oracle.sh`
Pre-cycle unpaired: 1026. Post-cycle unpaired: 911 (100 ported this cycle + 30 prior batch).

## Execution

- Background Workflow `wf_46a85fb0-6a5` (task `wf6pwv7bi`), single `Port` phase, `parallel()`, ~16 concurrent.
- Workers: `haiku`, `reasoning_effort: medium`, structured output (compact: wrote/destPath/notes).
- Selection: stratified 100 non-empty files across the <=2000-line pool (3..1926 lines, ~49.5k lines).
- Script: `scratchpad/blind-port-batch100.js` (batch inlined; args do not reach workflow scripts here).
- Agents wrote only their dest `.rs`; orchestrator committed each serially with `-c commit.gpgsign=false`.
- Prompt hardened vs batch30: forbids inventing magic struct offsets and reimplementing libc (e.g. strlen).

## Measured cost

- 100 agents, 4,278,235 subagent tokens, 1855 tool uses, ~60 min wall-clock.
- ~42.8k tokens/file (matches batch30's ~43.8k). Consistent calibration.

## Result

- 99 reported wrote; 1 reported failed (`oracle/code/game/NPC_misc.cpp`) — FALSE NEGATIVE:
  the file was fully written (228-line faithful port), the agent just failed to emit its final
  structured return. All 100 dest files verified present/non-empty and committed.

## Committed

100 commits `port <oracle-path>`. Verify with `git log --oneline -101`.

## Deferred

- 4 empty (0-byte) unpaired oracle files still pending (e.g. `oracle/code/Rufl/random.cpp`,
  `random.h`, `oracle/codemp/cgame/cg_media.h`, `cg_playeranimate.c`-class). Handle trivially
  (write a path-comment stub `.rs`) in a cleanup pass; not worth an agent each.
- 106 giants (>2000 lines) still deferred pending model decision.

## Remaining estimate (unchanged, validated)

- Small/medium pool now ~811 files <=2000 lines remaining (~915 non-empty - 100 - prior) :
  budget ~35M tokens, ~9h wall-clock at 16-wide.
- Giants: 106 files, ~30-40M tokens, high Haiku fidelity risk.

## Fidelity notes (blind, unverified — no build)

- Diagnostics surfacing on ported/other files are expected: unused `core::ffi` imports,
  `unused_mut`, unknown cfg features (`xbox`, `cgame_only`), unlinked-file (no mod.rs wiring,
  by design). All warnings; nothing built.
- A later verification/build-fix + module-wiring pass is required across the whole port.

## State

- Branch: `full-port`. HEAD before this handoff: last `port ...` commit of the 100.
- NOTE: repo signs via 1Password SSH (interactive); automated batches must use
  `-c commit.gpgsign=false`.
