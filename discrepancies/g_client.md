# Discrepancies: src/codemp/game/g_client.rs

- **Rust:** src/codemp/game/g_client.rs
- **PC oracle:** oracle/codemp/game/g_client.c
- **Xbox grayj:** /private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/8aa39250-5453-41a6-8515-4d0d90e61f9c/scratchpad/grayj/codemp/game/g_client.c
- **Verdict:** 1 divergence (1 xbox-residue)

## Findings

| Function | Rust line(s) | PC line(s) | Xbox line(s) | First divergence | Class |
|----------|--------------|-----------|--------------|------------------|-------|
| ClientSpawn | 3191–3261 | 3002–3071 | 2999–3059 | `g_client.rs:3261 — premature } closes changedSaber block; WP_SaberStyleValidForSaber/WP_UseFirstValidSaberStyle block absent` | xbox-residue |

---

### Detail

#### Finding 1 — ClientSpawn: saber-style validation fallback absent from changedSaber block

**Rust (3191–3261)** — `changedSaber` block ends immediately after saberAnimLevel clamping; no style check:
```rust
        // ... saberAnimLevel clamping (lines 3231-3260) ...
        }
    }
    // changedSaber block closes at line 3261; WP_SaberStyleValidForSaber never called
```

**PC oracle (3062–3070)** — appended inside `changedSaber` before closing `}`:
```c
if ( g_gametype.integer != GT_SIEGE )
{
    //let's just make sure the styles we chose are cool
    if ( !WP_SaberStyleValidForSaber( &ent->client->saber[0], &ent->client->saber[1],
             ent->client->ps.saberHolstered, ent->client->ps.fd.saberAnimLevel ) )
    {
        WP_UseFirstValidSaberStyle( &ent->client->saber[0], &ent->client->saber[1],
            ent->client->ps.saberHolstered, &ent->client->ps.fd.saberAnimLevel );
        ent->client->ps.fd.saberAnimLevelBase =
            ent->client->saberCycleQueue = ent->client->ps.fd.saberAnimLevel;
    }
}
```

**Xbox grayj (2999–3059)** — `changedSaber` block also closes without the style-validation block, matching the Rust exactly.
