# Discrepancies: src/codemp/game/w_force.rs

- **Rust:** src/codemp/game/w_force.rs
- **PC oracle:** oracle/codemp/game/w_force.c
- **Xbox grayj:** /private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/8aa39250-5453-41a6-8515-4d0d90e61f9c/scratchpad/grayj/codemp/game/w_force.c
- **Verdict:** Clean — 0 divergences

## Findings

| Function | Rust line(s) | PC line(s) | Xbox line(s) | First divergence | Class |
|----------|--------------|------------|--------------|------------------|-------|

No behavioral discrepancies found.

## Audit notes

All ~50 functions were compared function-by-function. Every known Xbox grayj divergence
from the PC oracle was checked and the Rust was confirmed to follow the PC version in
each case:

- **Loop-sound init guards** (`if (!speedLoopSound)` / `if (!seeingLoopSound)` etc.) —
  Rust has the guards (PC); Xbox omits them.
- **`else if (maxRank >= NUM_FORCE_MASTERY_LEVELS)` check** in
  `WP_SpawnInitForcePowers` — present in Rust (PC); absent in Xbox grayj.
- **`warnClient ||` in the `setForce` condition** — Rust includes it (PC); Xbox has it
  commented out as `/*warnClient ||*/`.
- **`GT_HOLOCRON` / `GT_JEDIMASTER` gametype filter** inside the `warnClient` block —
  Rust has the filter (PC); Xbox has the entire block commented out.
- **`!didEvent` NFR send** — Rust has no extra duel-gametype guard (PC); Xbox adds
  `&& g_gametype.integer != GT_DUEL && g_gametype.integer != GT_POWERDUEL`.
- **`holocronBits` / `GT_HOLOCRON` block** in `WP_SpawnInitForcePowers` — Rust has it
  as live code (PC); Xbox wraps it in `/* ... */`.
- **`saberFlags & SFL_TWO_HANDED`** in `WP_ForcePowerUsable` — Rust uses
  `SFL_TWO_HANDED` (PC); Xbox uses `saber[0].twoHanded`.
- **`otherKillerMOD` / `otherKillerVehWeapon` / `otherKillerWeaponType` writes** in
  `ForceThrow` and `DoGripAction` — Rust has them (PC has them at oracle lines 3675–3677,
  4045–4047, 4081–4083); Xbox grayj omits all nine assignments entirely.
