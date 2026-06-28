# bg_saber divergence report

Files compared:
- **Rust port**: `src/codemp/game/bg_saber.rs`
- **PC oracle** (source of truth): `oracle/codemp/game/bg_saber.c`
- **Xbox grayj**: `grayj/codemp/game/bg_saber.c`

Audit scope: full file, function-by-function.
Classification key: **xbox-residue** = Rust matches Xbox, diverges from PC oracle; **port-bug** = Rust matches neither; **intentional** = deliberate, semantics-preserving; **unsure** = ambiguous.

## Summary table

| # | Function | Rust location | PC oracle location | Xbox location | Class | Short description |
|---|----------|---------------|--------------------|---------------|-------|-------------------|
| 1 | `PM_SaberLockBreak` | rs:2095 | c:1240 | grayj:1226 | xbox-residue | `superBreak` threshold is `> 0`; PC uses `> Q_irand(2,4)` |
| 2 | `PM_CanDoDualDoubleAttacks` | rs:941 | c:2033 | grayj:1826 | xbox-residue | Missing `SFL_NO_MIRROR_ATTACKS` saber-flag guard |
| 3 | `PM_SaberBackflipAttackMove` | rs:1325 | c:1755 | grayj:1707 | xbox-residue | Missing `jumpAtkBackMove` saber override / cancel checks |
| 4 | `PM_SaberLungeAttackMove` | rs:1404 | c:1835 | grayj:1756 | xbox-residue | Missing `lungeAtkMove` saber override / cancel checks |
| 5 | `PM_SaberJumpAttackMove` | rs:1427 | c:1947 | grayj:1770 | xbox-residue | Missing `jumpAtkFwdMove` saber override / cancel checks |
| 6 | `PM_SaberFlipOverAttackMove` | rs:1249 | c:1657 | grayj:1640 | xbox-residue | Missing `jumpAtkFwdMove` saber override / cancel checks |
| 7 | `PM_SaberAttackForMovement` | rs:2376 | c:2241 | grayj:2019 | xbox-residue | Missing `overrideJumpRight/LeftAttackMove` setup and `allowCartwheels` (`SFL_NO_CARTWHEELS`) guard |

**Total: 7 divergences (7 xbox-residue, 0 port-bug, 0 intentional, 0 unsure)**

---

## Detail

### Finding 1 — `PM_SaberLockBreak`: `superBreak` threshold

**Rust** (`bg_saber.rs:2095`):
```rust
let superBreak: qboolean = if strength + (*ps).saberLockHits > 0 { QTRUE } else { QFALSE };
```

**PC oracle** (`bg_saber.c:1240`):
```c
qboolean superBreak = (strength+pm->ps->saberLockHits > Q_irand(2,4));
```

**Xbox grayj** (`bg_saber.c:1226`):
```c
qboolean superBreak = (strength+pm->ps->saberLockHits > 0);//Q_irand(2,4));
```

The Xbox commented out the random threshold `Q_irand(2,4)` and replaced it with the literal `0`. The Rust copied this change. Effect: saber lock breaks are much easier to trigger — any positive hit advantage triggers a super-break, whereas the PC requires a surplus of 2-4 above a random roll.

---

### Finding 2 — `PM_CanDoDualDoubleAttacks`: missing `SFL_NO_MIRROR_ATTACKS` guard

**Rust** (`bg_saber.rs:941`): function body only checks `BG_SaberInSpecialAttack`.

**PC oracle** (`bg_saber.c:2033–2056`): before calling `BG_SaberInSpecialAttack`, the function first checks `pm->ps->weapon == WP_SABER` and then tests `SFL_NO_MIRROR_ATTACKS` on both `saber1` and `saber2`, returning `qfalse` if either flag is set.

**Xbox grayj** (`bg_saber.c:1826–1834`): only checks `BG_SaberInSpecialAttack`; no weapon or saber-flag checks.

Effect: sabers with `SFL_NO_MIRROR_ATTACKS` set can still trigger dual double-attacks in the Rust port; the PC oracle would suppress them.

---

### Finding 3 — `PM_SaberBackflipAttackMove`: missing `jumpAtkBackMove` overrides

**Rust** (`bg_saber.rs:1325–1330`): sets `upmove = 127`, `velocity[2] = 500`, returns `LS_A_BACKFLIP_ATK` unconditionally.

**PC oracle** (`bg_saber.c:1755–1791`): before performing the backflip physics, resolves `saber1->jumpAtkBackMove` and `saber2->jumpAtkBackMove` via `BG_MySaber`. If a saber has a non-`LS_INVALID` override, the override move is returned instead; if the override is `LS_NONE`, `LS_A_T2B` is returned (cancelling the special). The backflip physics only executes if no saber vetoes it.

**Xbox grayj** (`bg_saber.c:1707–1712`): bare three-line body — no override checks (matches Rust).

Effect: custom saber types cannot redirect or cancel the backflip attack move in the Rust port.

---

### Finding 4 — `PM_SaberLungeAttackMove`: missing `lungeAtkMove` overrides

**Rust** (`bg_saber.rs:1404–1420`): bare lunge physics; always returns `LS_A_LUNGE`. The style dispatch (SS_STAFF → `LS_SPINATTACK`, SS_DUAL → `LS_SPINATTACK_DUAL`) is performed by the caller in `PM_SaberAttackForMovement`.

**PC oracle** (`bg_saber.c:1835–1889`): takes a `noSpecials` parameter. At the top it resolves `saber1->lungeAtkMove` and `saber2->lungeAtkMove` via `BG_MySaber`; an override returns the override move, `LS_NONE` returns `LS_A_T2B` (cancel). Only after these checks does it dispatch by style: `SS_FAST` executes the lunge physics and returns `LS_A_LUNGE`; staff/dual return `LS_SPINATTACK`/`LS_SPINATTACK_DUAL` respecting `noSpecials`; otherwise `LS_A_T2B`. The caller conditionally drains force only if the returned move is not `LS_A_T2B`/`LS_NONE`.

**Xbox grayj** (`bg_saber.c:1756–1768`): bare lunge body; style dispatch in caller (matches Rust structure). Xbox caller drains force unconditionally before dispatch.

Additional Xbox-matching behavior: force power is drained unconditionally before the dispatch in both Xbox and Rust; the PC oracle drains only if the move is not cancelled.

Effect: saber-type `lungeAtkMove` overrides and cancellations are ignored; force is always drained even when the move is vetoed by a saber type.

---

### Finding 5 — `PM_SaberJumpAttackMove`: missing `jumpAtkFwdMove` overrides

**Rust** (`bg_saber.rs:1427–1447`): executes forward-jump DFA physics unconditionally; returns `LS_A_JUMP_T__B_` with no override checks.

**PC oracle** (`bg_saber.c:1947–1993`): at the top resolves `saber1->jumpAtkFwdMove` and `saber2->jumpAtkFwdMove` via `BG_MySaber`; an override returns the override move, `LS_NONE` returns `LS_A_T2B`. The DFA physics only runs if no saber vetoes it. The caller conditionally drains force only when the result is not `LS_A_T2B`/`LS_NONE`.

**Xbox grayj** (`bg_saber.c:1770–1786`): no override checks; DFA physics unconditional (matches Rust). Xbox caller drains force unconditionally.

Effect: saber-type `jumpAtkFwdMove` overrides/cancels are ignored for the SS_STRONG DFA. Note that `PM_SaberJumpAttackMove2` (used for dual/staff forward attacks) correctly implements the override checks and is tested against the C oracle — this finding is specifically for the SS_STRONG path.

---

### Finding 6 — `PM_SaberFlipOverAttackMove`: missing `jumpAtkFwdMove` overrides

**Rust** (`bg_saber.rs:1249–1318`): immediately sets up flip-forward physics; returns `LS_A_FLIP_SLASH` unconditionally.

**PC oracle** (`bg_saber.c:1657–1753`): at the top resolves `saber1->jumpAtkFwdMove` and `saber2->jumpAtkFwdMove` via `BG_MySaber`; an override returns the override move, `LS_NONE` returns `LS_A_T2B`. The flip physics only runs if no saber vetoes it. The caller conditionally drains force only when the result is not `LS_A_T2B`/`LS_NONE`.

**Xbox grayj** (`bg_saber.c:1640–1705`): no override checks; flip physics unconditional (matches Rust). Xbox caller drains force unconditionally.

Effect: saber-type `jumpAtkFwdMove` overrides/cancels are ignored for the SS_MEDIUM forward-flip attack. Uses the same field as Finding 5 but for a distinct code path.

---

### Finding 7 — `PM_SaberAttackForMovement`: missing cartwheel-right/left overrides and `allowCartwheels` guard

**Rust** (`bg_saber.rs:2376–2658`): function body begins directly with the `rightmove`/`leftmove` dispatch. No saber override variables are computed; the cartwheel-right/left conditions have no override gate; the non-staff cartwheel branch has no `allowCartwheels` check.

**PC oracle** (`bg_saber.c:2241–2373`): before the movement dispatch, the function computes:
- `overrideJumpRightAttackMove` — set to `saber1->jumpAtkRightMove` or `saber2->jumpAtkRightMove` if either is not `LS_INVALID`
- `overrideJumpLeftAttackMove` — same for `jumpAtkLeftMove`
- `allowCartwheels` — set to `qfalse` if either saber has `SFL_NO_CARTWHEELS`

The cartwheel-right condition adds `overrideJumpRightAttackMove != LS_NONE` as a gate (preventing the cartwheel if a saber explicitly cancels it). Inside the block, if `overrideJumpRightAttackMove != LS_INVALID`, the override move is returned instead of performing the cartwheel. Otherwise, the non-staff branch checks `allowCartwheels` before doing the arial/cart move; if false, no aerial attack is set. The same logic mirrors for cartwheel-left.

**Xbox grayj** (`bg_saber.c:2019–2131`): no override variables computed; cartwheel condition has no gate; non-staff branch has no `allowCartwheels` check (matches Rust).

Effect: custom saber types cannot redirect or cancel cartwheel attacks via `jumpAtkRightMove`/`jumpAtkLeftMove`, and `SFL_NO_CARTWHEELS` is silently ignored.

---

## Non-findings (verified correct)

- **`PM_CheckStabDown`** (`rs:1087`): Rust DOES have the `SFL_NO_STABDOWN` saber-flag guard, matching the PC oracle. Xbox grayj lacks it. No divergence from PC.
- **`PM_SaberJumpAttackMove2`** (`rs:1459`): Rust correctly implements `jumpAtkFwdMove` override checks for the dual/staff forward-jump path; tested by oracle parity test at `rs:3941`. No divergence from PC.
- **`PM_CanDoRollStab`**: Rust has `SFL_NO_ROLL_STAB` checks, verified by oracle parity test at `rs:3879`. No divergence from PC.
- **Data tables** (`saberMoveData`, `transitionMove`, `saberMoveTransitionAngle`, `bg_parryDebounce`): verified byte-for-byte against PC oracle by tests at `rs:3649`. No divergence.
- **`PM_SaberLockWinAnim`/`LoseAnim`/`ResultAnim`/`AnimTransitionAnim`**: match both PC oracle and Xbox grayj.
- **`PM_SaberLocked`**: matches both PC oracle and Xbox grayj.
- **`PM_CheckAltKickAttack`**, **`PM_CanDoKata`**, **`PM_SaberMoveOkayForKata`**: functionally identical across all three sources.
