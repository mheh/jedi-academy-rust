# Discrepancies: src/codemp/game/q_math.rs vs oracle/codemp/game/q_math.c

- **Rust:** src/codemp/game/q_math.rs
- **Oracle:** oracle/codemp/game/q_math.c
- **Verdict:** Clean

## Findings

No behavioral discrepancies found.

### Notes

Every function was audited against the PC oracle line by line. The items below were examined closely but are NOT behavioral divergences:

- **`Q_rsqrt` / `Q_fabs` bit-hacks** — C used strict-aliasing-UB type-puns (`*(long*)&y`, `*(int*)&f`). Rust uses `f32::to_bits`/`from_bits`, which is the identical safe encoding. On a 32-bit target the C `long` is 32 bits, matching `i32` in Rust. The oracle parity test confirms bit-exact output.

- **`holdrand` integer width** — C declares `static unsigned long holdrand`. On the 32-bit Windows PC target `unsigned long` is 32 bits; Rust uses `AtomicU32` (+ wrapping arithmetic), reproducing that exact 32-bit stream. The Rust comment acknowledges this as an intentional width fix mirroring OpenJK's `uint32_t holdrand`.

- **`PlaneFromPoints` temporaries** — C writes the cross product directly into `plane[0..2]` then normalizes in-place; Rust uses a separate `normal` temp and copies back. Both paths produce identical `plane[0..2]` and `plane[3]` values on both degenerate and non-degenerate input (confirmed by oracle parity test).

- **`PerpendicularVector` `fabs` precision** — C promotes `src[i]` to `double` via `fabs()`; Rust widens to `f64` via `(src[i] as f64).abs()`. Identical result.

- **`AngleSubtract` `fmod`** — C's `fmod(a, 360)` promotes float to double; Rust does `(a as f64 % 360.0) as f32`. Identical result.

- **`flrand` extra `debug_assert!`** — C has no assert in `flrand`; Rust adds `debug_assert!((max - min) < 32768.0)`. This is a non-release defensive check only and does not change production behavior.

- **`BoxOnPlaneSide` x86 asm path** — The oracle also contains a Windows `__declspec(naked)` x86 asm fast path behind `#if !(__LCC__ || C_ONLY || !id386)`. Rust implements the portable C branch, which is the same branch used on Linux/Mac/non-x86 PC builds. No behavioral difference for any non-x86-Windows caller.

- **`NormalToLatLong` singularity check** — C uses `!normal[0] && !normal[1]` (C truthiness of float); Rust uses `normal[0] == 0.0 && normal[1] == 0.0`. Equivalent.

- **`DirToByte` null guard** — C has `if (!dir) return 0` for a null pointer; Rust's non-nullable `&vec3_t` subsumes this without changing behavior for valid calls.

- **`ColorBytes3` uninitialized fourth byte** — Rust zeroes the fourth byte of the `u32`; C leaves it indeterminate (stack garbage). Noted as a documented deviation; parity test ignores that byte.
