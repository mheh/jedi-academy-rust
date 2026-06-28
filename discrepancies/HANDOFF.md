# Handoff: validate-raven-source (Xbox-residue audit, GitHub issue #1)

## Goal
Working GitHub issue **#1** "Strain out leftover Xbox (grayj) code in favor of Raven PC
source" in repo `jedi-academy-rust` (an AI-assisted C++→Rust port of Jedi Academy).

Approach (from issue #1's comment): fan out **Sonnet** agents, one per ported game file,
each comparing the Rust port against its reference C source and writing a per-file
discrepancy markdown into `discrepancies/`. The orchestrator only manages which files go
to which agent — Sonnet does all reading and writing.

## Current state
- On branch **`validate-raven-source`** (created off `master`; not pushed).
- `discrepancies/` directory created and contains:
  - **10 report files** from a FIRST pilot run (two-way Rust-vs-PC only):
    `w_saber, bg_pmove, ai_wpnav, w_force, ai_main, q_math, g_combat, g_client,
    g_items, bg_saber`. Roll-up: 37 findings across 7 files; `bg_pmove`/`w_force`/`q_math`
    clean. All labeled `likely-bug`, **0 xbox-residue** (no Xbox source was available yet).
  - `validate-raven.mjs` — the workflow script (now updated to the THREE-WAY version).
  - `pairs.json` — the 86-file args list (rust + oracle C + xbox grayj path + out path).
  - `FORMAT.md` — documents the current three-way report format and classification rules.
- **Nothing committed yet.** Working tree has the new `discrepancies/` files untracked.

## Key finding so far (validated)
The `g_combat.md` report was spot-checked against real source (findings 1, 10, 11, 14) —
**all accurate with correct line pointers**. The divergences are **incompletely-ported PC
features** (commented-out `/* */` blocks, missing guards like the `DAMAGE_NO_PROTECTION`
gate and the `self->s.number < MAX_CLIENTS` check), NOT Xbox logic substituted in. That is
why the pilot found 0 xbox-residue — without the Xbox source you can't attribute, and the
evidence looked like port omissions. The user did NOT want g_combat fixed yet.

## The reframe (IN PROGRESS — this is where we paused)
To make `xbox-residue` attribution actually work, we added a THIRD reference: the grayj
**Xbox** source.
- Cloned shallow to: `<SCRATCHPAD>/grayj` (i.e.
  `/private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/6add34ac-6754-4edf-8438-31fcc00fc3f4/scratchpad/grayj`).
  Source: https://github.com/grayj/Jedi-Academy . **This is a temp clone — it will be gone
  in a fresh session and must be re-cloned.** Layout matches oracle: `codemp/game/*.c`.
- 82/86 files have a grayj counterpart; 4 are PC-only (no Xbox): `npc_ai_galakmech`,
  `npc_ai_interrogator`, `npc_ai_mark1`, `npc_ai_mark2` → two-way only.
- Verified grayj `g_combat.c` (5651 lines) genuinely differs from oracle (5943 lines).
- `validate-raven.mjs` and `pairs.json` were UPDATED for the three-way comparison.
  Classification: Rust≠PC & Rust==Xbox → `xbox-residue`; Rust≠PC & Rust≠Xbox → `port-bug`;
  else `intentional`/`unsure`. See `discrepancies/FORMAT.md`.

## Next steps
1. **Re-clone grayj** (temp dir is ephemeral) and confirm `pairs.json` xbox paths still
   resolve (the absolute scratchpad path will differ in a new session — regenerate
   `pairs.json` paths if so). Build script for paths is in the conversation; mapping is
   case-insensitive basename match `src/codemp/game/<n>.rs` ↔ `oracle/codemp/game/<N>.c`
   ↔ `grayj/codemp/game/<N>.c`.
2. **Decide billing/scope** — the user paused after the 10-file pilot to check whether
   Sonnet-agent usage counts against their plan (it does, at Sonnet rates — still their
   usage, just cheaper than Opus). The 10 heaviest files cost ~1M Sonnet tokens / ~15 min;
   full 86 likely ~2–4M tokens.
3. **Re-run the pilot 10 with the three-way workflow** so their reports get proper
   xbox-residue attribution and the new column format (the existing 10 use the old format).
4. **Run the remaining 76** (or full 86) once scope is approved.
5. After reports land, triage `xbox-residue` rows first (issue #1's actual target); the
   `port-bug` rows feed issue #2-style correctness work. The user does NOT want fixes
   applied until reports are reviewed.

## How to run the workflow
The workflow is a background `Workflow` tool script. Resend via
`{scriptPath: "<SCRATCHPAD>/scratchpad/validate-raven.mjs"}` (or the copy in
`discrepancies/validate-raven.mjs`) with `args` = the JSON array from `pairs.json`
(pass as an ACTUAL JSON array value, not a stringified string — the script does guard
with `JSON.parse` if it arrives as a string). For a smaller batch, pass a subset of the
array. Each agent uses `model: 'sonnet'`. Reports are written by the agents directly.

## Gotchas
- `Workflow` `args` may arrive as a string; the script already handles both.
- Concurrency cap is ~min(16, cores-2); 10 ran concurrently fine.
- The 4 PC-only files must use `xbox: null` in their pair object (script branches on it).
- Don't commit the temp grayj clone into the repo. The `oracle/` submodule is PC source
  (`https://code.idtech.space/raven/jediacademy`).

## Suggested skills
- **review** — once reports exist, to review the branch's discrepancy findings against the
  originating issue #1.
- **triage** / **to-issues** — to convert confirmed `xbox-residue` and `port-bug` findings
  into tracked GitHub issues/sub-tasks.
- **diagnose** — when investigating any single confirmed divergence deeply before fixing.

## Reference artifacts
- GitHub issue #1 (and its comment) — `gh issue view 1 --comments`
- GitHub issue #2 (array OOB) — related correctness work the `port-bug` findings feed.
- `discrepancies/FORMAT.md`, `discrepancies/validate-raven.mjs`, `discrepancies/pairs.json`
- Pilot reports in `discrepancies/*.md` (old two-way format — to be regenerated).
