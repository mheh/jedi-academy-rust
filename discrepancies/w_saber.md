# Discrepancies: src/codemp/game/w_saber.rs

- **Rust:** src/codemp/game/w_saber.rs
- **PC oracle:** oracle/codemp/game/w_saber.c
- **Xbox grayj:** /private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/8aa39250-5453-41a6-8515-4d0d90e61f9c/scratchpad/grayj/codemp/game/w_saber.c
- **Verdict:** 1 divergence (1 xbox-residue)

## Findings

| Function | Rust line(s) | PC line(s) | Xbox line(s) | First divergence | Class |
|---|---|---|---|---|---|
| `CheckSaberDamage` | 7323–7330 | 4076–4082 | 3791–3795 | Damage-trigger condition missing `WP_SaberBladeDoTransitionDamage` branch | xbox-residue |

### Detail

#### `CheckSaberDamage` — xbox-residue

PC oracle adds a fifth `||` arm to the damage-trigger `if` that enables damage during
transition animations when the blade is configured for it:

```c
// oracle/codemp/game/w_saber.c:4076–4082
if ( self->client->ps.saberAttackWound < level.time
    && (SaberAttacking(self)
        || BG_SuperBreakWinAnim(self->client->ps.torsoAnim)
        || (d_saberSPStyleDamage.integer&&self->client->ps.saberInFlight&&rSaberNum==0)
        || (WP_SaberBladeDoTransitionDamage( &self->client->saber[rSaberNum], rBladeNum )
            && BG_SaberInTransitionAny(self->client->ps.saberMove))
        || (self->client->ps.m_iVehicleNum && self->client->ps.saberMove > LS_READY) ))
```

Xbox grayj omits this arm entirely (the function
`WP_SaberBladeDoTransitionDamage` does not exist in that codebase):

```c
// grayj/codemp/game/w_saber.c:3791–3795
if ( (SaberAttacking(self)
        || BG_SuperBreakWinAnim(self->client->ps.torsoAnim)
        || (d_saberSPStyleDamage.integer&&self->client->ps.saberInFlight&&rSaberNum==0)
        || (self->client->ps.m_iVehicleNum && self->client->ps.saberMove > LS_READY) ) &&
    self->client->ps.saberAttackWound < level.time )
```

The Rust port reproduces the Xbox logic — four `||` arms, no transition-damage
branch — making it behaviorally identical to grayj rather than to the PC oracle.
Sabers with the `SFL2_NO_TRANSITION_DAMAGE`-off flag will not deal damage during
transition frames as they should on PC.

```rust
// src/codemp/game/w_saber.rs:7323–7330
if (SaberAttacking(self_) != QFALSE
    || BG_SuperBreakWinAnim((*(*self_).client).ps.torsoAnim) != QFALSE
    || ((*addr_of!(d_saberSPStyleDamage)).integer != 0
        && (*(*self_).client).ps.saberInFlight != QFALSE
        && rSaberNum == 0)
    || ((*(*self_).client).ps.m_iVehicleNum != 0
        && (*(*self_).client).ps.saberMove > LS_READY))
    && (*(*self_).client).ps.saberAttackWound < (*addr_of!(level)).time
```

**Fix:** add the missing arm after the `d_saberSPStyleDamage` branch:
```rust
|| (WP_SaberBladeDoTransitionDamage(
        addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
        rBladeNum,
    ) != QFALSE
    && BG_SaberInTransitionAny((*(*self_).client).ps.saberMove) != QFALSE)
```

## Functions audited with no finding

The following functions were compared three-way and found to match the PC oracle
semantically (idiomatic Rust translation, no behavioral divergence):

| Function | Notes |
|---|---|
| `RandFloat` | Rust uses 32767.0 = RAND_MAX for bg_lib LCG, matching PC semantics |
| `SetSaberBoxSize` | Has PC's full `dualSabers`/`alwaysBlock`/`forceBlock` logic |
| `WP_SabersCheckLock` | Uses `saberFlags & SFL_NOT_LOCKABLE` (PC-style, not Xbox `.lockable` field) |
| `WP_SaberStartMissileBlockCheck` | Uses `saberFlags & SFL_NOT_ACTIVE_BLOCKING` (PC-style) |
| `G_PowerLevelForSaberAnim` | Identical across all three |
| `CheckThrownSaberDamaged` | Has PC's `isJediMaster` check, `saberFlags2 & SFL2_NO_DISMEMBERMENT`, sets `te->s.weapon`/`legsAnim` |
| `saberCheckKnockdown_DuelLoss` | Identical across all three |
| `saberCheckKnockdown_BrokenParry` | Identical across all three |
| `G_KickSomeMofos` | Identical across all three |
| `WP_SaberDoHit` | PC-only function (absent from Xbox); Rust matches PC signature and body |
| `WP_SaberDoClash` | Rust uses PC signature `(self, saberNum, bladeNum)`, sets `te->s.weapon`/`legsAnim` (Xbox had no-arg version) |
| `WP_GetSaberDeflectionAngle` | Identical across all three |
| `saberMoveBack` | Identical across all three |
| `MakeDeadSaber` | Identical across all three |
| `WP_SaberPositionUpdate` | Trail access uses `.trail.lastTime`/`.base`/`.tip` (PC struct, not Xbox array-by-client-num); `lastTime` validity guard present; `WP_SaberDoHit` called per-blade per PC |
