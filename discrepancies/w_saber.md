# Discrepancies: src/codemp/game/w_saber.rs vs oracle/codemp/game/w_saber.c

- **Rust:** src/codemp/game/w_saber.rs
- **Oracle:** oracle/codemp/game/w_saber.c
- **Verdict:** 1 behavioral divergence

## Findings

| Function | Rust line(s) | Oracle line(s) | First divergence | Confidence |
|----------|--------------|----------------|------------------|------------|
| `CheckSaberDamage` | 7323–7330 | 4076–4082 | `w_saber.rs:7323 — missing \`\|\| (WP_SaberBladeDoTransitionDamage(...)&&BG_SaberInTransitionAny(...))\`` | likely-bug |

### Detail

`CheckSaberDamage` decides whether a blade deals full damage on a given frame. The PC oracle (lines 4076–4082) gates the damage branch on five OR-conditions (wrapped by the `saberAttackWound` time-guard):

```c
if ( self->client->ps.saberAttackWound < level.time
    && (SaberAttacking(self)
        || BG_SuperBreakWinAnim(self->client->ps.torsoAnim)
        || (d_saberSPStyleDamage.integer&&self->client->ps.saberInFlight&&rSaberNum==0)
        || (WP_SaberBladeDoTransitionDamage( &self->client->saber[rSaberNum], rBladeNum )
            &&BG_SaberInTransitionAny(self->client->ps.saberMove))   // ← 4th clause
        || (self->client->ps.m_iVehicleNum && self->client->ps.saberMove > LS_READY) )
   )
```

The Rust port (lines 7323–7330) contains only four OR-conditions; the 4th clause is entirely absent:

```rust
if (SaberAttacking(self_) != QFALSE
    || BG_SuperBreakWinAnim((*(*self_).client).ps.torsoAnim) != QFALSE
    || ((*addr_of!(d_saberSPStyleDamage)).integer != 0
        && (*(*self_).client).ps.saberInFlight != QFALSE
        && rSaberNum == 0)
    // ← WP_SaberBladeDoTransitionDamage(...) && BG_SaberInTransitionAny(...) MISSING
    || ((*(*self_).client).ps.m_iVehicleNum != 0
        && (*(*self_).client).ps.saberMove > LS_READY))
    && (*(*self_).client).ps.saberAttackWound < (*addr_of!(level)).time
```

The missing condition is: when a blade's `doTransitionDamage` flag is set (queried by `WP_SaberBladeDoTransitionDamage`) **and** the current saber move is a transition animation (queried by `BG_SaberInTransitionAny`), the blade should enter the full damage branch. Without this clause, no blade ever triggers damage via this path in the Rust port regardless of its `doTransitionDamage` flag — a silent gameplay behaviour regression for any saber type that uses that flag.

This is not Xbox-vs-PC residue; it is missing logic that was present in the PC source from the start and appears to have been dropped during the porting pass on this function.

## Non-divergences examined

The following items were scrutinised during the audit but are NOT behavioral divergences:

- **`CheckSaberDamage` condition order** — C checks `saberAttackWound < level.time` first (short-circuit); Rust evaluates the OR-group first and then ANDs the timer check. Both have the same final boolean result since no sub-expression has visible side-effects prior to the AND.

- **`G_SaberAttackPower` broken-limb multiply** — C: `baseLevel *= 0.3;` (implicit `int`×`float`, result truncated into `int`). Rust: `(baseLevel as f64 * 0.3) as c_int`. Identical truncation for all integer inputs in range.

- **`G_SaberAttackPower` `assert`** — C has `assert(ent && ent->client)` (debug only); Rust omits it. No production behavior difference.

- **`saberCheckKnockdown_BrokenParry` `saber[1].model` null guard** — C checks `other->client->saber[1].model` as a pointer before `model[0]`; since `model` is `char[MAX_QPATH]` (fixed inline array), the pointer is always non-null. Rust skips straight to `model[0] != 0`. Equivalent. The body of that branch is a no-op expression (`other->client->saber[1].disarmBonus;`) in C; Rust preserves it as `let _ = ...`.

- **`WP_GetSaberDeflectionAngle` `quadDiff` abs** — C: `int quadDiff = fabs((float)(defQuad-attQuadStart))`. Rust: `((defQuad - attQuadStart) as f32).abs() as c_int`. Identical for all small-integer inputs.

- **`WP_SaberCanBlock` `!point` null guard** — C has `if (!point) return 0` (absent in Rust). The Rust signature takes `&vec3_t` (non-nullable reference), so the guard is subsumed by the type system. No behavioral difference for valid callers.

- **`RandFloat` RAND_MAX** — C uses `(float)RAND_MAX` (32767 on MSVC); Rust uses `32767.0f32`. Identical.

- **`G_PowerLevelForSaberAnim` `BOTH_FORCELONGLEAP_ATTACK` / `BOTH_STABDOWN*` fall-through** — C `break`s out of the switch and falls to `return FORCE_LEVEL_0` after the switch. Rust returns `FORCE_LEVEL_0` as the arm's value expression. Same result.

### Functions audited

Every function in both files was compared against the PC oracle. The full list:

`RandFloat`, `G_DebugBoxLines`, `G_CanBeEnemy`, `HasSetSaberOnly`, `UpdateClientRenderBolts`,
`UpdateClientRenderinfo`, `VectorCompare2`, `WPDEBUG_SaberColor`, `G_PrettyCloseIGuess`,
`MakeDeadSaber`, `WP_SaberStartMissileBlockCheck`, `saberMoveBack`, `CheckThrownSaberDamaged`,
`saberCheckRadiusDamage`, `saberKnockDown`, `saberKnockOutOfHand`, `saberReactivate`,
`WP_SaberRemoveG2Model`, `WP_SaberAddG2Model`, `WP_SaberInitBladeData`, `WP_SaberBladeLength`,
`WP_SaberLength`, `WP_DeactivateSaber`, `WP_ActivateSaber`, `G_G2TraceCollide`,
`G_SaberInBackAttack`, `SaberAttacking`, `WP_SaberClearDamage`, `WP_SaberDamageAdd`,
`WP_SaberApplyDamage`, `WP_SaberDoHit`, `WP_SaberRadiusDamage`, `G_KickDownable`,
`G_TossTheMofo`, `WP_MissileBlockForBlock`, `G_GetParryForBlock`, `WP_SaberBlockNonRandom`,
`WP_SaberBlock`, `WP_SaberCanBlock`, `G_KnockawayForParry`, `G_SaberLockAnim`,
`WP_SabersCheckLock2`, `WP_SabersCheckLock`, `G_BuildSaberFaces`, `G_SabCol_CalcPlaneEq`,
`G_SabCol_PointRelativeToPlane`, `G_SaberFaceCollisionCheck`, `G_SaberCollide`,
`G_SaberAttackPower`, `G_GetAttackDamage`, `G_GetAnimPoint`, `G_ClientIdleInWorld`,
`WP_GetSaberDeflectionAngle`, `WP_SabersIntersect`, `WP_SaberDoClash`, `WP_SaberBounceSound`,
`G_SPSaberDamageTraceLerped`, `G_CheckLookTarget`, `G_G2NPCAngles`, `G_G2PlayerAngles`,
`G_KickTrace`, `G_KickSomeMofos`, `G_GrabSomeMofos`, `WP_SaberPositionUpdate`,
`CheckSaberDamage`, `G_PowerLevelForSaberAnim`, `saberCheckKnockdown_DuelLoss`,
`saberCheckKnockdown_BrokenParry`, `saberCheckKnockdown_Smashed`, `saberCheckKnockdown_Thrown`
