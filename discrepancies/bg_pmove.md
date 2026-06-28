# Discrepancies: src/codemp/game/bg_pmove.rs

- **Rust:** src/codemp/game/bg_pmove.rs
- **PC oracle:** oracle/codemp/game/bg_pmove.c
- **Xbox grayj:** scratchpad/grayj/codemp/game/bg_pmove.c
- **Verdict:** Clean — 0 behavioral divergences found (0 xbox-residue, 0 port-bug, 0 unsure)

## Findings

No behavioral discrepancies found.

## Audit notes

A full function-by-function three-way comparison was performed across the Rust port
(12,720 lines), PC oracle (11,217 lines), and Xbox grayj (10,880 lines). Functions
examined (all confirmed PC-matching):

`PM_GetSaberStance`, `PM_Friction`, `PM_CmdScale`, `PM_CheckJump`, `PM_WalkMove`,
`PM_AirMove`, `PM_CrashLand`, `PM_GroundTraceMissed`, `PM_GroundTrace`, `PM_CheckDuck`,
`PM_Footsteps`, `PM_Animate`, `PM_DropTimers`, `BG_UnrestrainedPitchRoll`,
`PM_VehFaceHyperspacePoint`, `PmoveSingle`, `Pmove`, `PM_WaterJumpMove`, `PM_WaterMove`,
`PM_FlyVehicleMove`, `PM_FlyMove`, `PM_RocketLock`, `PM_DoChargedWeapons`, `PM_Weapon`,
`PM_AdjustAttackStates`, `BG_CmdForRoll`, `BG_AdjustClientSpeed`, `PM_CmdForSaberMoves`,
`BG_G2PlayerAngles`, `BG_G2ClientNeckAngles`, `BG_G2ClientSpineAngles`, `BG_SwingAngles`,
`BG_G2ATSTAngles`, and all intermediate helpers.

### Intentional / non-finding patterns observed

| Pattern | Description |
|---------|-------------|
| f64 arithmetic promotions | Several expressions use `as f64 * … as f32` to replicate C's implicit double-promotion (e.g. `PM_CmdScale`, `PM_Friction`, `PM_CrashLand` quadratic, `PM_WalkMove` waterScale, `BG_AdjustClientSpeed` speed multipliers, `PmoveSingle` gravity × 0.5). All match the PC oracle's intended precision. |
| `#ifdef QAGAME` branches | The Rust port is a QAGAME build; QAGAME-only paths are taken and the `#else` (CGAME) paths are dropped. Observed in `PM_Weapon` (vehicle button masking, `G_CheapWeaponFire` call), and the QAGAME NPC no-weapon torso-sync block. Both the PC and Xbox grayj have identical `#ifdef` structure; behavior of the active branch matches. |
| `METROID_JUMP = 1` dead branch | The `#else` branch in `PM_AirMove`/`PM_CheckJump` is never built. Rust takes the active `#if METROID_JUMP` path, matching PC behavior. |
| `VEH_CONTROL_SCHEME_4` dead branch | Not defined in either PC or Xbox build. `PM_VehFaceHyperspacePoint` takes the non-VEH_CONTROL_SCHEME_4 path, matching the active PC build. |
| `BONE_BASED_LEG_ANGLES` undefined | The `legBoneYaw` block in `BG_G2PlayerAngles` is dropped (never defined in either PC or Xbox). |
| `PM_GetSaberStance` — key Xbox vs PC structural difference | Xbox grayj uses `FORCE_LEVEL_1`/`FORCE_LEVEL_5`/`FORCE_LEVEL_3`/etc. in the switch and lacks the `saber1->readyAnim` / `saber2->readyAnim` guard checks and the dual-saber holster check. The PC oracle and the Rust port both use `SS_FAST`/`SS_TAVION`/`SS_STRONG`/`SS_NONE`/`SS_MEDIUM`/`SS_DESANN` with the full saber-object guards. Rust matches PC — not a finding. |
