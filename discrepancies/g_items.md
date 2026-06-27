# Discrepancies: src/codemp/game/g_items.rs

- **Rust:** src/codemp/game/g_items.rs
- **PC oracle:** oracle/codemp/game/g_items.c
- **Xbox grayj:** /private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/8aa39250-5453-41a6-8515-4d0d90e61f9c/scratchpad/grayj/codemp/game/g_items.c
- **Verdict:** 2 divergences (1 port-bug, 1 intentional)

## Findings

| Function | Rust line(s) | PC line(s) | Xbox line(s) | First divergence | Class |
|----------|--------------|------------|--------------|------------------|-------|
| `Touch_Item` | 3717 | 2465 | 2465 | `g_items.rs:3717 — \`(*ent).s.eType == ET_NPC\`` | port-bug |
| `ItemUse_Seeker` | 959–965 | 1096–1125 | 1096–1125 | `g_items.rs:959 — GT_SIEGE/d_siegeSeekerNPC branch absent` | intentional |

### Detail

#### `ItemUse_Seeker` (intentional)

The PC oracle (and Xbox, identically) conditionally spawns a real NPC remote for siege:

```c
// PC oracle / Xbox g_items.c:1098-1124
if ( g_gametype.integer == GT_SIEGE && d_siegeSeekerNPC.integer )
{
    gentity_t *remote = NPC_SpawnType( ent, "remote", NULL, qfalse );
    // ... team assignment ...
}
else
{
    ent->client->ps.eFlags |= EF_SEEKERDRONE;
    ent->client->ps.droneExistTime = level.time + 30000;
    ent->client->ps.droneFireTime  = level.time + 1500;
}
```

The Rust port omits the `if` branch entirely and always executes the `else` logic:

```rust
// g_items.rs:962-965
(*(*ent).client).ps.eFlags |= EF_SEEKERDRONE;
(*(*ent).client).ps.droneExistTime = ((*addr_of!(level)).time + 30000) as f32;
(*(*ent).client).ps.droneFireTime  = ((*addr_of!(level)).time + 1500) as f32;
```

The Rust docstring explicitly acknowledges this: "the GT_SIEGE / d_siegeSeekerNPC branch (NPC_SpawnType 'remote') is not yet ported, pending the NPC subsystem … See DEVIATIONS."
