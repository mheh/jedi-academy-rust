# Discrepancies: src/codemp/game/ai_wpnav.rs vs oracle/codemp/game/ai_wpnav.c

- **Rust:** src/codemp/game/ai_wpnav.rs
- **Oracle:** oracle/codemp/game/ai_wpnav.c
- **Verdict:** 1 behavioral divergence — translation bug in `GetFlagStr`

## Findings

| Function | Rust line(s) | Oracle line(s) | First divergence | Confidence |
|----------|--------------|----------------|------------------|------------|
| `GetFlagStr` | 678–681 | 201–205 | `ai_wpnav.rs:678 — \`put(&mut i, b'\0');\`` | likely-bug |

### Detail

The `put` closure (Rust lines 597–600) is defined as `flagstr[i] = c; i += 1` — it always increments `i` after writing.  The C places the NUL terminator as `flagstr[i] = '\0';` (oracle line 201) **without** a subsequent `i++`, so the trailing `if (i == 0)` guard (oracle line 203) correctly detects the "no flags matched" case and writes `"unknown"` into the buffer.

In the Rust translation `put(&mut i, b'\0')` at line 678 writes the NUL and increments `i` from `0` to `1`; the subsequent `if i == 0` (line 680) is therefore unreachable.  When `flags` is non-zero but contains no recognised bit, the C returns the string `"unknown"` while the Rust returns an empty string (NUL at offset 0).
