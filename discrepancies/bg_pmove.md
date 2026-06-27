# Discrepancies: src/codemp/game/bg_pmove.rs vs oracle/codemp/game/bg_pmove.c

- **Rust:** src/codemp/game/bg_pmove.rs
- **Oracle:** oracle/codemp/game/bg_pmove.c
- **Verdict:** Clean

## Findings

No behavioral discrepancies found.

### Notes

Every function was audited against the PC oracle line by line across the full 12 720-line Rust file and 11 218-line C oracle. The items below were examined closely but are NOT behavioral divergences:

- **`VectorSet(mins, pm->mins[0], pm->mins[0], 0)` bug in `PM_CheckJump`** — both the oracle (lines 2356–2357 and 2427–2428) and the Rust port (corresponding lines in the wall-run-in-air section) use `mins[0]` for both the X and Y component, silently clobbering the correct Y. This is a faithful bug preservation, not a divergence.

- **`pml.frametime = pml.msec * 0.001`** — the C `0.001` is a double literal, so the product promotes to `f64` before narrowing to the `f32` field. The Rust reproduces this as `(pml.msec as f64 * 0.001) as f32` throughout (e.g. `PmoveSingle`, `PM_Friction`).

- **`pm->ps->gravity *= 0.5` (slow-fall)** — `gravity` is `int`; multiplying by the double literal `0.5` yields a double product that is then truncated back to `int`. The Rust uses `(gravity as f64 * 0.5) as c_int`, preserving the truncation semantics.

- **`addTime *= 0.75` / `addTime *= 1.5` (rage in `PM_Weapon`)** — `addTime` is `int` in C; the bare double literals cause double-promotion before truncation. Rust correctly uses `(addTime as f64 * 0.75) as c_int` and `(addTime as f64 * 1.5) as c_int`.

- **`BG_AdjustClientSpeed` double-promoted multiplies** — the C uses bare `0.75`, `0.4`, `1.7`, etc. (all doubles) on a `float speed`, so the product is double before narrowing. Rust carries these as explicit `(speed as f64 * …) as f32` casts exactly where the C literal has no `f` suffix.

- **`PM_NoclipMove` dual-button turbo boost** — both `BUTTON_ATTACK` and `BUTTON_ALT_ATTACK` trigger the speed boost (oracle lines 3576–3581, Rust lines 4196–4201). `PM_FlyMove` uses only `BUTTON_ALT_ATTACK` for its turbo; both files agree on this asymmetry.

- **`pm->cmd.forcesel != -1` unsigned-byte compare** — `forcesel` / `invensel` are `byte` (u8). C compares `byte != -1`; in C the byte promotes to int and the comparison is true unless the byte holds `0xFF`. The Rust reproduces this as `forcesel as c_int != -1` and uses `1i32.wrapping_shl(forcesel as u32)` to match the x86 masked-shift behaviour when the sentinel `0xFF` value reaches the shift. Both paths are behaviorally identical on the PC target.

- **`#ifdef QAGAME` / `#else` branches** — the Rust is always the QAGAME build. All `#ifdef QAGAME` branches are included and all `#else` / cgame-only branches are correctly dropped. This includes `PM_UpdateViewAngles` (oracle has two copies, the Rust uses only the QAGAME version), `PM_VehFaceHyperspacePoint` / `PM_VehForcedTurning` (VEH_CONTROL_SCHEME_4 vs default), and the vehicle-physics `#else` block in `PmoveSingle`.

- **`#if 0` dead blocks** — several C blocks guarded by `#if 0` (e.g. BOTH_KISSER1LOOP stiffenedUp check, the old disruptor-movement-cancel approach, the experimental saber-off grapple in `PM_Weapon`) are correctly dropped in the Rust; the code paths are documented in comments.
