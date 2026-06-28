# Discrepancies: src/codemp/game/g_combat.rs

- **Rust:** src/codemp/game/g_combat.rs
- **PC oracle:** oracle/codemp/game/g_combat.c
- **Xbox grayj:** /private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/8aa39250-5453-41a6-8515-4d0d90e61f9c/scratchpad/grayj/codemp/game/g_combat.c
- **Verdict:** 7 divergences (6 xbox-residue, 1 port-bug)

## Findings

| Function | Rust line(s) | PC line(s) | Xbox line(s) | First divergence | Class |
|----------|--------------|-----------|--------------|------------------|-------|
| G_GetJediMaster | 560–573 | 1744–1761 | 1743–1761 | `g_combat.rs:563 — if (*ent).inuse != 0 && !(*ent).client.is_null() && (*(*ent).client).ps.isJediMaster != 0` | port-bug |
| player_die | 5131–5133 | 2459–2461 | 2376–2378 | `g_combat.rs:5134 — (BG_ClearRocketLock call absent)` | xbox-residue |
| player_die | 5145–5150 | 2470–2473 | 2386–2391 | `g_combat.rs:5145 — /* if (self->client && self->client->ps.isJediMaster) { wasJediMaster = qtrue; } */` | xbox-residue |
| player_die | 5158–5166 | 2482–2501 | 2399–2404 | `g_combat.rs:5162 — meansOfDeath == MOD_UNKNOWN) (MOD_SUICIDE absent; otherKillerMOD/tempInflictorEnt block absent)` | xbox-residue |
| player_die | 4887–4943 | 2256/2269/2279 | 2184/2198/2208 | `g_combat.rs:4895 — MOD_BLASTER (should be actualMOD; murderer used as inflictor)` | xbox-residue |
| player_die | 5252–5259 | 2563 | 2466–2474 | `g_combat.rs:5252 — ent = G_TempEntity(...EV_OBITUARY) (inline instead of G_BroadcastObit call)` | xbox-residue |
| player_die | 5269–5300 | 2578–2636 | 2485–2516 | `g_combat.rs:5270 — lastkilled_client = self->s.number (no MAX_CLIENTS guard; MOD_COLLISION/VEH_EXPLOSION branch absent; G_CheckTKAutoKickBan absent)` | xbox-residue |

---

### Detail

#### Finding 2 — player_die: missing BG_ClearRocketLock / isHacking / hackingTime resets

**Rust (5131–5133)** — three resets only:
```rust
(*(*self_).client).ps.heldByClient = 0;
(*(*self_).client).beingThrown = 0;
(*(*self_).client).doingThrow = 0;
// BG_ClearRocketLock / isHacking / hackingTime absent
```

**PC oracle (2459–2461)** — adds three more:
```c
BG_ClearRocketLock( &self->client->ps );
self->client->isHacking = 0;
self->client->ps.hackingTime = 0;
```

**Xbox grayj (2376–2378)** — same as Rust; the three extra PC lines are absent.

---

#### Finding 3 — player_die: wasJediMaster assignment commented out

**Rust (5145–5150)** — commented block, wasJediMaster always stays QFALSE:
```rust
/*
if (self->client && self->client->ps.isJediMaster)
{
    wasJediMaster = qtrue;
}
*/
```

**PC oracle (2470–2473)** — block is active:
```c
if (self->client && self->client->ps.isJediMaster)
{
    wasJediMaster = qtrue;
}
```

**Xbox grayj (2386–2391)** — same block is commented out with `/* ... */`, matching the Rust.

---

#### Finding 4 — player_die: otherKillerTime block missing MOD_SUICIDE and vehicle-weapon re-attribution

**Rust (5158–5166)** — condition missing MOD_SUICIDE; body only reassigns attacker:
```rust
if (self_ == attacker || (*attacker).client.is_null())
    && (meansOfDeath == MOD_CRUSH
        || meansOfDeath == MOD_FALLING
        || meansOfDeath == MOD_TRIGGER_HURT
        || meansOfDeath == MOD_UNKNOWN)   // MOD_SUICIDE absent
    && (*(*self_).client).ps.otherKillerTime > (*addr_of!(level)).time
{
    attacker = g_entities + otherKiller;
    // actualMOD / tempInflictorEnt tracking absent
}
```

**PC oracle (2482–2501)** — includes MOD_SUICIDE; body also copies otherKillerMOD into actualMOD and optionally spawns a fake tempInflictorEnt to carry vehicle-weapon info:
```c
if ((self == attacker || !attacker->client) &&
    (meansOfDeath == MOD_CRUSH || meansOfDeath == MOD_FALLING ||
     meansOfDeath == MOD_TRIGGER_HURT || meansOfDeath == MOD_UNKNOWN ||
     meansOfDeath == MOD_SUICIDE) &&
    self->client->ps.otherKillerTime > level.time)
{
    attacker = &g_entities[self->client->ps.otherKiller];
    if ( self->client->otherKillerMOD != MOD_UNKNOWN )
        actualMOD = self->client->otherKillerMOD;
    if ( self->client->otherKillerVehWeapon > 0 )
    {
        tempInflictorEnt = G_Spawn();
        if ( tempInflictorEnt ) { tempInflictor = qtrue; ... }
    }
}
```

**Xbox grayj (2399–2404)** — no MOD_SUICIDE; body only reassigns attacker, matching the Rust.

---

#### Finding 5 — player_die: vehicle occupant G_Damage calls use MOD_BLASTER / murderer as inflictor

**Rust (4887–4896, 4915–4924, 4934–4943)** — three calls (pilot, passengers, droid unit) all use murderer as inflictor and hardcode MOD_BLASTER:
```rust
G_Damage(killEnt, murderer, murderer, null_mut(),
         addr_of_mut!((*(*killEnt).client).ps.origin),
         99999, DAMAGE_NO_PROTECTION, MOD_BLASTER);
```

**PC oracle (2256, 2269, 2279)** — same three call sites use tempInflictorEnt (carries vehicle-weapon info) and actualMOD:
```c
G_Damage(killEnt, tempInflictorEnt, murderer, NULL,
         killEnt->client->ps.origin, 99999, DAMAGE_NO_PROTECTION, actualMOD);
```

**Xbox grayj (2184, 2198, 2208)** — same as Rust: murderer as inflictor, MOD_BLASTER hardcoded.

---

#### Finding 6 — player_die: inline EV_OBITUARY instead of G_BroadcastObit, missing vehicle/missile metadata

**Rust (5252–5259)** — inline broadcast with 4 fields; isJediMaster and all vehicle/missile fields absent:
```rust
if (*self_).s.eType != ET_NPC && g_noPDuelCheck == QFALSE {
    ent = G_TempEntity(&(*self_).r.currentOrigin, EV_OBITUARY);
    (*ent).s.eventParm = meansOfDeath;
    (*ent).s.otherEntityNum = (*self_).s.number;
    (*ent).s.otherEntityNum2 = killer;
    (*ent).r.svFlags = SVF_BROADCAST;
    //		ent->s.isJediMaster = wasJediMaster;
}
```

**PC oracle (2563)** — calls the dedicated helper which additionally sets `s.eventParm = MOD_VEHICLE`, `s.weapon`, `s.generic1`, `s.lookTarget` (wasInVehicle), `s.brokenLimbs`, `s.saberInFlight`, and `s.isJediMaster`:
```c
G_BroadcastObit( self, inflictor, attacker, killer, actualMOD, wasInVehicle, wasJediMaster );
```

**Xbox grayj (2466–2474)** — same inline pattern as the Rust, with `//ent->s.isJediMaster = wasJediMaster;` likewise commented.

Note: `G_BroadcastObit` is faithfully translated in Rust (lines 669–717) but is never called by `player_die`.

---

#### Finding 7 — player_die: scoring section missing MAX_CLIENTS guard, MOD_COLLISION/VEH_EXPLOSION no-credit branches, and G_CheckTKAutoKickBan

**Rust (5269–5300)** — unconditional lastkilled_client write; jumps straight to OnSameTeam check; no G_CheckTKAutoKickBan:
```rust
(*(*attacker).client).lastkilled_client = (*self_).s.number; // no MAX_CLIENTS guard
G_CheckVictoryScript(attacker);
if attacker == self_ || OnSameTeam(self_, attacker) != QFALSE {
    // ...
    AddScore(attacker, &(*self_).r.currentOrigin, -1);
    // G_CheckTKAutoKickBan absent
} else {
    AddScore(attacker, &(*self_).r.currentOrigin, 1);
}
// no outer else-if (MOD_COLLISION || MOD_VEH_EXPLOSION) branch
```

**PC oracle (2578–2636)** — guards lastkilled_client to real players; skips scoring for droid-in-vehicle kills and veh-veh collisions; calls G_CheckTKAutoKickBan on team-kills; adds a MOD_FALLING+vehicle no-penalty guard:
```c
if ( self->s.number < MAX_CLIENTS )
    attacker->client->lastkilled_client = self->s.number;
G_CheckVictoryScript(attacker);
if ( self->s.number >= MAX_CLIENTS && self->client->NPC_class != CLASS_VEHICLE && self->s.m_iVehicleNum )
{ /* no credit for droid */ }
else if ( meansOfDeath == MOD_COLLISION || meansOfDeath == MOD_VEH_EXPLOSION )
{ /* no credit */ }
else if ( attacker == self || OnSameTeam(self, attacker) ) {
    if ( meansOfDeath == MOD_FALLING && attacker != self && attacker->s.m_iVehicleNum )
    { /* no penalty */ }
    else { AddScore(-1); G_CheckTKAutoKickBan(attacker); }
} else {
    AddScore(+1);
}
// outer else if (MOD_COLLISION || MOD_VEH_EXPLOSION) for no-attacker path
```

**Xbox grayj (2485–2516)** — no MAX_CLIENTS guard on lastkilled_client, no droid/MOD_COLLISION branches, no G_CheckTKAutoKickBan; structure matches Rust.
