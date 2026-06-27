# Discrepancies: src/codemp/game/w_force.rs vs oracle/codemp/game/w_force.c

- **Rust:** src/codemp/game/w_force.rs
- **Oracle:** oracle/codemp/game/w_force.c
- **Verdict:** Clean

## Findings

No behavioral discrepancies found.

### Functions audited

Every function in both files was compared line by line against the PC oracle. The full list:

`G_PreDefSound`, `WP_ForcePowerStop`, `WP_InitForcePowers`, `WP_SpawnInitForcePowers`,
`WP_ForcePowerRegenerate`, `WP_ForcePowerAvailable`, `WP_ForcePowerInUse`,
`ForcePowerUsableOn`, `WP_ForcePowerUsable`, `WP_AbsorbConversion`, `G_IsMindTricked`,
`HolocronUpdate`, `JediMasterUpdate`, `WP_HasForcePowers`, `WP_AddToClientBitflags`,
`WP_AddAsMindtricked`, `RemoveTrickedEnt`, `G_InGetUpAnim`, `G_SpecialRollGetup`,
`ForceJumpCharge`, `WP_GetVelocityForForceJump`, `ForceJump`, `WP_ForcePowerStart`,
`ForceHeal`, `ForceSpeed`, `ForceSeeing`, `ForceProtect`, `ForceAbsorb`, `ForceRage`,
`ForceLightning`, `ForceDrain`, `ForceDrainDamage`, `ForceLightningDamage`,
`ForceShootLightning`, `ForceShootDrain`, `ForceGrip`, `ForceTeamHeal`,
`ForceTeamForceReplenish`, `ForceTelepathyCheckDirectNPCTarget`, `ForceTelepathy`,
`ForceThrow`, `GEntity_UseFunc`, `CanCounterThrow`, `G_LetGoOfWall`, `DoGripAction`,
`WP_UpdateMindtrickEnts`, `SeekerDroneUpdate`, `FindGenericEnemyIndex`,
`Jedi_DodgeEvasion`, `WP_ForcePowerRun`, `WP_DoSpecificPower`, `WP_ForcePowersUpdate`

### Notes

The following items were examined closely but are NOT behavioral divergences:

- **`WP_ForcePowerUsable` null-model guard** — C checks `self->client->modelindex && model && model[0]`; Rust checks only the array index (Rust's `model` is a fixed inline array, so pointer-null is impossible). Equivalent.

- **`WP_ForcePowerStart` speed-drain cast** — C passes `overrideAmt * 0.025` as a float that gets truncated to the `int` parameter of `BG_ForcePowerDrain`; Rust does `(override_amt as f64 * 0.025) as c_int`. Identical truncation.

- **`ForceTelepathyCheckDirectNPCTarget` `mindTrickDone`** — C declares `qboolean mindTrickDone = qfalse` and sets it to `qtrue` once, but never reads it after assignment. The Rust port comments the assignment out as vestigial and omits the variable entirely. No behavioral effect.

- **`DoGripAction` / `ForceGrip` `forceGripBeingGripped` type** — The Rust struct stores `forceGripBeingGripped` and related drone/seeker times as `f32`; C stores them as `int`. The code bodies cast appropriately on both sides. No behavioral divergence in the logic; the type difference lives in the struct definition, not the function bodies.

- **`SeekerDroneUpdate` `droneExistTime` arithmetic** — `droneExistTime` is `f32` in Rust vs `int` in C; comparisons and arithmetic are cast accordingly. The formulas and branch conditions are structurally identical.

- **`ForceGrip` end-of-function `return`** — C has an explicit `return;` in an `else` branch at the end of the function; Rust omits it because the `else` is the last arm and falls off the end. Equivalent.

- **`FP_DRAIN` fallthrough in `WP_ForcePowerStop`** — C's `default:` fallthrough after the `FP_DRAIN` label is faithfully modeled: both paths reach the same code.

- **`WP_InitForcePowers` `GT_SIEGE` path, bot personality, force-power parsing** — all match the PC oracle.

- **`ForceDrainDamage` dead g2animent check** — C has a dead `g2animent` read inside a client-non-null block; Rust mirrors it identically.

- **Cosine/sine precision in `SeekerDroneUpdate` orbit** — C calls `cos(angle)` / `sin(angle)` (promoting `float` to `double`); Rust does `(angle as f64).cos() as f32`. Equivalent double-precision intermediate.
