# Discrepancies: src/codemp/game/bg_saber.rs vs oracle/codemp/game/bg_saber.c

- **Rust:** src/codemp/game/bg_saber.rs
- **Oracle:** oracle/codemp/game/bg_saber.c
- **Verdict:** 12 behavioral divergences — all likely-bug; primary pattern is systematic omission of per-saber override checks (`BG_MySaber` + `jumpAtkFwdMove`/`jumpAtkBackMove`/`lungeAtkMove`/`kataMove`/`jumpAtkRightMove`/`jumpAtkLeftMove`) and unconditional force-drain where C drains only on a valid (non-cancelled) move

## Findings

| Function | Rust line(s) | Oracle line(s) | First divergence | Confidence |
|----------|--------------|----------------|------------------|------------|
| `PM_SaberLockBreak` | 2095 | 1240 | bg_saber.rs:2095 — `> 0` instead of `> Q_irand(2,4)` | likely-bug |
| `PM_SaberFlipOverAttackMove` | 1249–1323 | 1664–1691 | bg_saber.rs:1259 — `VectorCopy(&(*ps).viewangles` (first physics stmt; no `BG_MySaber` / `jumpAtkFwdMove` override check precedes it) | likely-bug |
| `PM_SaberBackflipAttackMove` | 1325–1332 | 1757–1787 | bg_saber.rs:1327 — `(*pmv).cmd.upmove = 127` (no `BG_MySaber` / `jumpAtkBackMove` override check precedes it) | likely-bug |
| `PM_SaberLungeAttackMove` | 1404–1420 | 1835–1889 | bg_saber.rs:1404 — signature `PM_SaberLungeAttackMove()` (missing `noSpecials` param; no `lungeAtkMove` override check; always does lunge physics) | likely-bug |
| `PM_SaberJumpAttackMove` | 1427–1447 | 1947–1993 | bg_saber.rs:1432 — `VectorCopy(&(*ps).viewangles` (first physics stmt; no `BG_MySaber` / `jumpAtkFwdMove` override check precedes it) | likely-bug |
| `PM_CanDoDualDoubleAttacks` | 941–951 | 2033–2056 | bg_saber.rs:945 — `if BG_SaberInSpecialAttack((*ps).torsoAnim)` (no `WP_SABER` weapon check or `SFL_NO_MIRROR_ATTACKS` flag test precedes it) | likely-bug |
| `PM_CanDoKata` | 1929 | 2733–2744 | bg_saber.rs:1929 — `return QTRUE;` (no `BG_MySaber` / `kataMove == LS_NONE` veto check before returning) | likely-bug |
| `PM_CheckAltKickAttack` | 920–932 | 2750–2775 | bg_saber.rs:924 — `if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0` (no `WP_SABER` weapon check or `SFL_NO_KICKS` flag test precedes it) | likely-bug |
| `PM_WeaponLightsaber` (roll-stab) | 2707–2716 | 2862–2871 | bg_saber.rs:2707 — `if BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != QFALSE` (no enclosing `PM_CanDoRollStab()` guard) | likely-bug |
| `PM_WeaponLightsaber` (kata dispatch) | 3062–3076 | 3341–3416 | bg_saber.rs:3064 — `match (*ps).fd.saberAnimLevel` (no `BG_MySaber` / `kataMove` override dispatch; cancelled-kata fallthrough absent) | likely-bug |
| `PM_SaberAttackForMovement` (cartwheel/jump-right/jump-left) | 2383–2420 | 2241–2310 | bg_saber.rs:2385 — `if noSpecials == QFALSE` (missing `overrideJumpRightAttackMove != LS_NONE` guard; no `jumpAtkRightMove`/`jumpAtkLeftMove`/`SFL_NO_CARTWHEELS` override logic) | likely-bug |
| `PM_SaberAttackForMovement` (force-drain timing) | 2529, 2543–2544 | ~2472–2495, ~2506–2511 | bg_saber.rs:2529 — `BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB)` (unconditional; C drains only when `newmove != LS_A_T2B && newmove != LS_NONE`) | likely-bug |

### Detail

#### Finding 1 — PM_SaberLockBreak: superBreak threshold replaced with `> 0`

Oracle (line 1240):
```c
qboolean superBreak = (strength+pm->ps->saberLockHits > Q_irand(2,4));
```

Rust (line 2095):
```rust
let superBreak: qboolean = if strength + (*ps).saberLockHits > 0 { QTRUE } else { QFALSE }; //Q_irand(2,4))
```

The random threshold `Q_irand(2,4)` (range 2–4) was replaced with a hard-coded `0`, making `superBreak` true whenever `strength + saberLockHits > 0` — i.e., almost always. The commented-out `Q_irand(2,4)` in the Rust source confirms this was noticed but not fixed. This makes the saber-lock super-break fire far more readily than the PC game intends.

---

#### Finding 2 — PM_SaberFlipOverAttackMove: missing jumpAtkFwdMove override check

Oracle (lines 1664–1691): before computing flip physics, retrieves `BG_MySaber(pm->ps->clientNum, 0)` and `BG_MySaber(..., 1)`, checks each saber's `jumpAtkFwdMove` field; if the override is valid (not `LS_INVALID`) the function returns that override move directly (or `LS_A_T2B` if cancelled with `LS_NONE`).

Rust (line 1259): proceeds directly to `VectorCopy(&(*ps).viewangles, ...)` physics setup with no `BG_MySaber` call. Custom saber `jumpAtkFwdMove` overrides are never consulted; the flip always fires regardless of saber type.

---

#### Finding 3 — PM_SaberBackflipAttackMove: missing jumpAtkBackMove override check

Oracle (lines 1757–1787): checks `saber1->jumpAtkBackMove` and `saber2->jumpAtkBackMove` overrides; returns the override directly if valid, or `LS_A_T2B` if `LS_NONE`, before ever computing backflip physics.

Rust (lines 1327–1329): immediately executes `(*pmv).cmd.upmove = 127; (*ps).velocity[2] = 500.0;` and returns `LS_A_BACKFLIP_ATK` with no override check. Custom saber backflip overrides and cancellations are ignored.

---

#### Finding 4 — PM_SaberLungeAttackMove: missing noSpecials param, override check, and style dispatch

Oracle (lines 1835–1889): signature `PM_SaberLungeAttackMove(qboolean noSpecials)`. Checks `lungeAtkMove` override on both sabers. If `saberAnimLevel == SS_FAST`, performs lunge physics and returns `LS_A_LUNGE`. If `SS_STAFF && !noSpecials`, returns `LS_SPINATTACK`. If `!noSpecials` (else), returns `LS_SPINATTACK_DUAL`. Fallback returns `LS_A_T2B`.

Rust (lines 1404–1420): signature `PM_SaberLungeAttackMove()` — no `noSpecials` parameter. No `lungeAtkMove` override check. Always executes lunge physics and always returns `LS_A_LUNGE`. The `SS_STAFF` / `SS_DUAL` spin-attack dispatch in the oracle is replicated ad-hoc in the `PM_SaberAttackForMovement` caller in Rust but the `LS_A_T2B` fallback for `noSpecials=true` with dual/staff style is missing there (newmove stays `LS_NONE` instead).

---

#### Finding 5 — PM_SaberJumpAttackMove: missing jumpAtkFwdMove override check

Oracle (lines 1947–1993): before jump physics, retrieves both sabers via `BG_MySaber` and checks `jumpAtkFwdMove`; returns the override (or `LS_A_T2B` on cancellation) before doing any velocity/angle work.

Rust (line 1432): proceeds immediately to `VectorCopy(&(*ps).viewangles, ...)` with no override check. Mirrors Finding 2 for the non-aerial forward jump attack path.

---

#### Finding 6 — PM_CanDoDualDoubleAttacks: missing WP_SABER + SFL_NO_MIRROR_ATTACKS guard

Oracle (lines 2033–2056): opens with:
```c
if ( pm->ps->weapon == WP_SABER ) {
    saberInfo_t *saber = BG_MySaber( pm->ps->clientNum, 0 );
    if ( saber && (saber->saberFlags & SFL_NO_MIRROR_ATTACKS) ) { return qfalse; }
}
```
Only then checks `BG_SaberInSpecialAttack`.

Rust (lines 941–951): opens directly with `if BG_SaberInSpecialAttack((*ps).torsoAnim)`. Neither the `WP_SABER` weapon check nor the `SFL_NO_MIRROR_ATTACKS` flag test is present. Saber types that explicitly disable mirror attacks will incorrectly be allowed to perform dual double attacks.

---

#### Finding 7 — PM_CanDoKata: missing kataMove == LS_NONE veto

Oracle (lines 2733–2744): after all preconditions pass, checks:
```c
saberInfo_t *saber1 = BG_MySaber(pm->ps->clientNum, 0);
if ( saber1 && saber1->kataMove == LS_NONE ) { return qfalse; }
saberInfo_t *saber2 = BG_MySaber(pm->ps->clientNum, 1);
if ( saber2 && saber2->kataMove == LS_NONE ) { return qfalse; }
```

Rust (line 1929): returns `QTRUE` immediately with no `BG_MySaber` / `kataMove` check. Saber types that explicitly disable kata moves via `kataMove == LS_NONE` will still trigger kata attacks.

---

#### Finding 8 — PM_CheckAltKickAttack: missing WP_SABER + SFL_NO_KICKS guard

Oracle (lines 2750–2775): opens with:
```c
if ( pm->ps->weapon == WP_SABER ) {
    saberInfo_t *saber = BG_MySaber( pm->ps->clientNum, 0 );
    if ( saber && (saber->saberFlags & SFL_NO_KICKS) ) { return qfalse; }
}
```

Rust (line 924): opens directly with `if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0`. No weapon or saber-flag check. Saber types that disable kicks will still allow kick attacks.

---

#### Finding 9 — PM_WeaponLightsaber (roll-stab): missing PM_CanDoRollStab() guard

Oracle (lines 2862–2871):
```c
if ( PM_CanDoRollStab() ) {
    if ( BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) ) {
        ...roll-stab setup...
    }
}
```

Rust (line 2707): the outer `PM_CanDoRollStab()` call is absent; the inner `BG_EnoughForcePowerForMove` check is the outermost gate. `PM_CanDoRollStab` inspects `WP_SABER` and the `SFL_NO_ROLL_STAB` saber flag; without it, saber types that disable roll-stab will still be able to perform the move.

---

#### Finding 10 — PM_WeaponLightsaber (kata dispatch): missing kataMove override dispatch

Oracle (lines 3341–3416): after `PM_CanDoKata()` returns true, retrieves `kataMove` overrides from both sabers. If an override is valid (`!= LS_INVALID && != LS_NONE`), uses `overrideMove` as the kata. If `overrideMove == LS_NONE` (explicit cancellation), skips the entire kata block (falls through without returning). Only if no override (`== LS_INVALID`) uses the default `saberAnimLevel`-based dispatch.

Rust (lines 3062–3076): directly dispatches on `saberAnimLevel` with `match` and always returns. No `BG_MySaber` / `kataMove` override check, and no cancelled-kata fallthrough path.

---

#### Finding 11 — PM_SaberAttackForMovement (cartwheel / jump-right / jump-left): missing override and SFL_NO_CARTWHEELS

Oracle (lines 2241–2310): computes `overrideJumpRightAttackMove` and `overrideJumpLeftAttackMove` from `jumpAtkRightMove`/`jumpAtkLeftMove` on both sabers; computes `allowCartwheels` from `!(saber->saberFlags & SFL_NO_CARTWHEELS)`. The cartwheel path is guarded by `overrideJumpRightAttackMove != LS_NONE`; if a valid override exists it is returned directly. Non-staff non-dual cartwheel is further gated by `allowCartwheels`.

Rust (line 2385): the first branch is `if noSpecials == QFALSE` with no `overrideJumpRightAttackMove` guard, no `BG_MySaber` calls, and no `allowCartwheels` variable. Cartwheel attacks fire regardless of saber type; saber override moves for right/left jump attacks are never applied.

---

#### Finding 12 — PM_SaberAttackForMovement (force-drain timing): unconditional vs conditional

Oracle (lines ~2472–2495 and ~2506–2511): for the flip-over and DFA paths:
```c
newmove = PM_SaberFlipOverAttackMove();
if ( newmove != LS_A_T2B && newmove != LS_NONE ) {
    BG_ForcePowerDrain(pm->ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
}
```
Force is drained only when the move helper returns a live move (not a cancelled/fallback value). The same pattern applies to the DUAL/STAFF jump-attack path via `PM_SaberJumpAttackMove2`.

Rust (lines 2529 and 2543–2544): `BG_ForcePowerDrain` is called unconditionally immediately after `PM_SaberFlipOverAttackMove()`, regardless of whether the returned move is `LS_A_T2B`, `LS_NONE`, or a valid attack. The DUAL/STAFF jump-attack path similarly drains force before the move is determined. This causes spurious force drain when saber overrides cancel a move.
