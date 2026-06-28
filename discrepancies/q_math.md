# Discrepancies: src/codemp/game/q_math.rs

- **Rust:** src/codemp/game/q_math.rs
- **PC oracle:** oracle/codemp/game/q_math.c
- **Xbox grayj:** /private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/8aa39250-5453-41a6-8515-4d0d90e61f9c/scratchpad/grayj/codemp/game/q_math.c
- **Verdict:** 1 divergence (1 xbox-residue)

## Findings

| Function | Rust line(s) | PC line(s) | Xbox line(s) | First divergence | Class |
|----------|--------------|-----------|--------------|------------------|-------|
| `flrand` | 1016 | 1441–1449 | 1462–1473 | `q_math.rs:1016 — \`debug_assert!((max - min) < 32768.0);\`` | xbox-residue |

### Detail

**`flrand`** — PC oracle omits the range assert entirely; Xbox grayj has `assert((max - min) < 32768);`; Rust has `debug_assert!((max - min) < 32768.0);`.  In release builds the `debug_assert!` compiles away so there is no observable difference, but in debug builds the Rust will abort on out-of-range input where the PC silently continues.

```rust
// Rust (q_math.rs:1015–1016)
pub fn flrand(min: f32, max: f32) -> f32 {
    debug_assert!((max - min) < 32768.0);
```

```c
// PC oracle (q_math.c:1441–1444) — no assert
float flrand(float min, float max)
{
    float	result;
    holdrand = (holdrand * 214013L) + 2531011L;
```

```c
// Xbox grayj (q_math.c:1462–1467) — has assert
float flrand(float min, float max)
{
    float	result;
    assert((max - min) < 32768);
    holdrand = (holdrand * 214013L) + 2531011L;
```
