# Discrepancies: src/codemp/game/g_client.rs vs oracle/codemp/game/g_client.c

- **Rust:** src/codemp/game/g_client.rs
- **Oracle:** oracle/codemp/game/g_client.c
- **Verdict:** 1 behavioral divergence — missing saber-style fallback in `ClientSpawn`

## Findings

| Function | Rust line(s) | Oracle line(s) | First divergence | Confidence |
|----------|--------------|----------------|------------------|------------|
| `ClientSpawn` | 3261 | 3062–3070 | g_client.rs:3261 — closing `}` of `changedSaber` block without the preceding `WP_SaberStyleValidForSaber` / `WP_UseFirstValidSaberStyle` guard | likely-bug |

### Detail

Inside the `if changedSaber` block, the oracle (lines 3062–3070) validates the newly-chosen saber style before the block closes:

```c
if ( g_gametype.integer != GT_SIEGE )
{
    if ( !WP_SaberStyleValidForSaber( &ent->client->saber[0], &ent->client->saber[1],
                                       ent->client->ps.saberHolstered,
                                       ent->client->ps.fd.saberAnimLevel ) )
    {
        WP_UseFirstValidSaberStyle( &ent->client->saber[0], &ent->client->saber[1],
                                    ent->client->ps.saberHolstered,
                                    &ent->client->ps.fd.saberAnimLevel );
        ent->client->ps.fd.saberAnimLevelBase =
            ent->client->saberCycleQueue =
                ent->client->ps.fd.saberAnimLevel;
    }
}
```

The Rust port's `changedSaber` block ends at line 3261 without this guard. When a client's saber changes at spawn time outside of Siege mode, the Rust version leaves `saberAnimLevel` at whatever value was chosen by the saber-setup path; if that level is incompatible with the actual saber type (e.g., a staff/dual style with a single-blade hilt), the oracle would silently correct it via `WP_UseFirstValidSaberStyle`, but the Rust port would not. `saberCycleQueue` is also never updated in the Rust path.

`WP_SaberStyleValidForSaber` and `WP_UseFirstValidSaberStyle` are not yet imported in `g_client.rs`, so this was almost certainly deferred with the rest of the saber-style validation infrastructure rather than deliberately dropped.
