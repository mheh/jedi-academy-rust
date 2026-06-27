# Discrepancies: src/codemp/game/g_combat.rs vs oracle/codemp/game/g_combat.c

- **Rust:** src/codemp/game/g_combat.rs
- **Oracle:** oracle/codemp/game/g_combat.c
- **Verdict:** 14 behavioral divergences across `G_Damage` (3) and `player_die` (11) — all likely-bug

## Findings

| Function | Rust line(s) | Oracle line(s) | First divergence | Confidence |
|----------|--------------|----------------|------------------|------------|
| `G_Damage` | 3736 | 5413 | g_combat.rs:3736 — `if take != 0 && !(*targ).client.is_null() && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0 {` | likely-bug |
| `G_Damage` | 3856 | 5549 | g_combat.rs:3856 — `if !(*targ).client.is_null() && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0` | likely-bug |
| `G_Damage` | 3875 | 5570 | g_combat.rs:3875 — `if !(*targ).client.is_null() && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0` | likely-bug |
| `player_die` | 4842 | 2186–2197 | g_combat.rs:4842 — `murderer = murderPilot;` without following `actualMOD = self->client->otherKillerMOD` or `tempInflictorEnt` spawn | likely-bug |
| `player_die` | 4887–4943 | 2256–2280 | g_combat.rs:4887 — `G_Damage(killEnt, murderer, murderer, ..., MOD_BLASTER)` | likely-bug |
| `player_die` | 4879–4880 | 2239–2248 | g_combat.rs:4880 — `murderer = self_;` without `if !attacker { self } else { attacker }` branch | likely-bug |
| `player_die` | — | 2459–2461 | g_combat.c:2459 — `BG_ClearRocketLock(&self->client->ps);` (absent in Rust) | likely-bug |
| `player_die` | 4789 | 2470–2473 | g_combat.rs:4789 — `let wasJediMaster: qboolean = QFALSE;` with update block commented out at 5145–5150 | likely-bug |
| `player_die` | 5158–5162 | 2482–2501 | g_combat.rs:5162 — condition ends at `MOD_UNKNOWN` without `\|\| meansOfDeath == MOD_SUICIDE`; also missing `actualMOD`/`tempInflictorEnt` setup from lines 2487–2501 | likely-bug |
| `player_die` | 5270 | 2580–2582 | g_combat.rs:5270 — `(*(*attacker).client).lastkilled_client = (*self_).s.number;` without `if self->s.number < MAX_CLIENTS` guard | likely-bug |
| `player_die` | 5274 | 2587–2596 | g_combat.rs:5274 — `if attacker == self_ \|\| OnSameTeam(...)` with no preceding "droid-in-vehicle no-credit" or `MOD_COLLISION`/`MOD_VEH_EXPLOSION` checks | likely-bug |
| `player_die` | 5365–5400 | 2702–2705 | g_combat.rs:5365 — non-client `else` branch falls through to GT_DUEL without `else if (MOD_COLLISION \|\| MOD_VEH_EXPLOSION)` early-exit | likely-bug |
| `player_die` | 5251–5259 | 2563 | g_combat.rs:5251 — inline `G_TempEntity(EV_OBITUARY)` block instead of `G_BroadcastObit(self, inflictor, attacker, killer, actualMOD, wasInVehicle, wasJediMaster)` | likely-bug |
| `player_die` | — | 2638–2713 | g_combat.c:2638 — `GT_JEDIMASTER` scoring branches (ThrowSaberToAttacker, score-to-jmEnt) commented out entirely in Rust | likely-bug |

### Detail

#### Finding 1 — G_Damage: FP_PROTECT not gated by DAMAGE_NO_PROTECTION

Oracle (line 5413):
```c
if ( !(dflags & DAMAGE_NO_PROTECTION) )
{//protect overridden by no_protection
    if (take && targ->client && (targ->client->ps.fd.forcePowersActive & (1 << FP_PROTECT)))
```

Rust (line 3736):
```rust
if take != 0
    && !(*targ).client.is_null()
    && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0
```

The outer `!(dflags & DAMAGE_NO_PROTECTION)` guard is absent. Any caller that passes `DAMAGE_NO_PROTECTION` (e.g., the vehicle-occupant G_Damage calls, telefrag, crush) can still have its damage reduced by force-protect in the Rust port.

---

#### Finding 2 — G_Damage: FP_RAGE damage reduction not gated by DAMAGE_NO_PROTECTION

Oracle (line 5549):
```c
if ( !(dflags & DAMAGE_NO_PROTECTION) )
{//rage overridden by no_protection
    if (targ->client && (targ->client->ps.fd.forcePowersActive & (1 << FP_RAGE)) && (inflictor->client || attacker->client))
        take /= (targ->client->ps.fd.forcePowerLevel[FP_RAGE]+1);
}
```

Rust (line 3856):
```rust
if !(*targ).client.is_null()
    && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
    && (!(*inflictor).client.is_null() || !(*attacker).client.is_null())
{
    take /= (*(*targ).client).ps.fd.forcePowerLevel[FP_RAGE as usize] + 1;
}
```

Same `!(dflags & DAMAGE_NO_PROTECTION)` gate missing as finding 1.

---

#### Finding 3 — G_Damage: FP_RAGE health floor not gated by DAMAGE_NO_PROTECTION

Oracle (line 5570):
```c
if ( !(dflags & DAMAGE_NO_PROTECTION) )
{//rage overridden by no_protection
    if (targ->client && (targ->client->ps.fd.forcePowersActive & (1 << FP_RAGE)) && ...)
    { if (targ->health <= 0) targ->health = 1; ... }
}
```

Rust (line 3875):
```rust
if !(*targ).client.is_null()
    && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
    ...
{ if (*targ).health <= 0 { (*targ).health = 1; } }
```

Gate missing as with findings 1–2. A DAMAGE_NO_PROTECTION hit that would kill a raging target still gets its health clamped to 1.

---

#### Finding 4 — player_die: vehicle kill path missing actualMOD update and fake inflictor

Oracle (lines 2183–2199), inside the `murderPilot` branch of the vehicle-is-self path:
```c
murderer = murderPilot;
actualMOD = self->client->otherKillerMOD;
if ( self->client->otherKillerVehWeapon > 0 )
{
    tempInflictorEnt = G_Spawn();
    if ( tempInflictorEnt )
    {
        tempInflictor = qtrue;
        tempInflictorEnt->classname = "vehicle_proj";
        tempInflictorEnt->s.otherEntityNum2 = self->client->otherKillerVehWeapon-1;
        tempInflictorEnt->s.weapon = self->client->otherKillerWeaponType;
    }
}
```

Rust (line 4842):
```rust
murderer = murderPilot;
// actualMOD never captured; tempInflictorEnt never spawned
```

`actualMOD` stays `meansOfDeath` and `tempInflictorEnt` stays `inflictor`. This propagates wrong MOD and inflictor into every subsequent G_Damage call and the obituary.

---

#### Finding 5 — player_die: G_Damage calls on vehicle occupants use wrong inflictor and MOD

Oracle (lines 2256, 2269, 2279):
```c
G_Damage(killEnt, tempInflictorEnt, murderer, NULL, killEnt->client->ps.origin, 99999, DAMAGE_NO_PROTECTION, actualMOD);
```

Rust (lines 4887–4895, 4915–4924, 4934–4943):
```rust
G_Damage(killEnt, murderer, murderer, null_mut(), ..., 99999, DAMAGE_NO_PROTECTION, MOD_BLASTER);
```

Both the inflictor argument (`murderer` instead of `tempInflictorEnt`) and the MOD (`MOD_BLASTER` instead of `actualMOD`) are wrong. This affects the pilot, all passengers, and the droid unit.

---

#### Finding 6 — player_die: "no valid murderer" fallback ignores attacker

Oracle (lines 2239–2248):
```c
if (!murderer)
{
    if ( !attacker )
        murderer = self;
    else
        murderer = attacker;
}
```

Rust (lines 4879–4880):
```rust
if murderer.is_null() {
    murderer = self_;
}
```

When there is a valid (non-null) `attacker` but no `murderer` derived from the vehicle pilot chain, the Rust port credits `self_` as the murderer instead of `attacker`. Kill credit for vehicle passengers is wrong in this scenario.

---

#### Finding 7 — player_die: BG_ClearRocketLock / isHacking / hackingTime not reset

Oracle (lines 2459–2461):
```c
BG_ClearRocketLock( &self->client->ps );
self->client->isHacking = 0;
self->client->ps.hackingTime = 0;
```

Rust: all three lines are absent. A player who dies while hacking or with an active rocket lock retains those states, which can cause stale UI artifacts after respawn.

---

#### Finding 8 — player_die: wasJediMaster never set to true

Oracle (lines 2470–2473):
```c
if (self->client && self->client->ps.isJediMaster)
{
    wasJediMaster = qtrue;
}
```

Rust (line 4789):
```rust
let wasJediMaster: qboolean = QFALSE;
```

The update block is commented out at lines 5145–5150. `wasJediMaster` is always `QFALSE`. This feeds into the obituary (finding 13), where `ent->s.isJediMaster` would never be set (the assignment is also commented out at line 5258).

---

#### Finding 9 — player_die: MOD_SUICIDE missing from otherKiller condition; actualMOD/tempInflictorEnt setup absent

Oracle (lines 2482–2501):
```c
if ((self == attacker || !attacker->client) &&
    (... || meansOfDeath == MOD_SUICIDE) &&
    self->client->ps.otherKillerTime > level.time)
{
    attacker = &g_entities[self->client->ps.otherKiller];
    if ( self->client->otherKillerMOD != MOD_UNKNOWN )
        actualMOD = self->client->otherKillerMOD;
    if ( self->client->otherKillerVehWeapon > 0 )
    { /* spawn tempInflictorEnt */ }
}
```

Rust (lines 5158–5166):
```rust
if (self_ == attacker || (*attacker).client.is_null())
    && (meansOfDeath == MOD_CRUSH
        || meansOfDeath == MOD_FALLING
        || meansOfDeath == MOD_TRIGGER_HURT
        || meansOfDeath == MOD_UNKNOWN)  // MOD_SUICIDE missing
    && (*(*self_).client).ps.otherKillerTime > (*addr_of!(level)).time
{
    attacker = ...; // actualMOD and tempInflictorEnt setup absent
}
```

Three problems in the same block: (a) `MOD_SUICIDE` not in the condition, so `/kill` deaths never get rerouted to the real last attacker; (b) `actualMOD` is never updated from `otherKillerMOD`; (c) `tempInflictorEnt` is never constructed from `otherKillerVehWeapon`.

---

#### Finding 10 — player_die: lastkilled_client written without MAX_CLIENTS guard

Oracle (lines 2580–2582):
```c
if ( self->s.number < MAX_CLIENTS )
{//only remember real clients
    attacker->client->lastkilled_client = self->s.number;
}
```

Rust (line 5270):
```rust
(*(*attacker).client).lastkilled_client = (*self_).s.number;
```

The guard is absent. NPC entity numbers (>= MAX_CLIENTS) also update `lastkilled_client`, which can corrupt attacker-tracking logic that assumes only real player indices are stored there.

---

#### Finding 11 — player_die: missing "no credit for droid in vehicle" and MOD_COLLISION/VEH_EXPLOSION scoring skips (client-attacker branch)

Oracle (lines 2587–2596):
```c
if ( self->s.number >= MAX_CLIENTS
    && self->client
    && self->client->NPC_class != CLASS_VEHICLE
    && self->s.m_iVehicleNum )
{//no credit for droid, you do get credit for the vehicle kill and the pilot (2 points)
}
else if ( meansOfDeath == MOD_COLLISION
    || meansOfDeath == MOD_VEH_EXPLOSION )
{//no credit for veh-veh collisions?
}
else if ( attacker == self || OnSameTeam (self, attacker ) )
```

Rust (line 5274):
```rust
if attacker == self_ || OnSameTeam(self_, attacker) != QFALSE {
```

Both the droid-in-vehicle no-credit check and the MOD_COLLISION/MOD_VEH_EXPLOSION no-credit check are absent. Vehicle collision kills erroneously award score, and killing a droid that is riding another vehicle also awards score.

---

#### Finding 12 — player_die: missing MOD_COLLISION/VEH_EXPLOSION scoring skip in the non-client-attacker branch

Oracle (lines 2702–2705):
```c
else if ( meansOfDeath == MOD_COLLISION
    || meansOfDeath == MOD_VEH_EXPLOSION )
{//no credit for veh-veh collisions?
}
else
{ /* GT_DUEL etc. */ }
```

Rust (lines 5365–5400, the `else` block when `attacker.is_null() || (*attacker).client.is_null()`):

The `MOD_COLLISION`/`MOD_VEH_EXPLOSION` early exit is absent; the code falls through directly to the commented-out JediMaster check and the GT_DUEL/`AddScore(self_, -1)` path. Vehicle-collision suicides incorrectly apply a -1 score.

---

#### Finding 13 — player_die: inline EV_OBITUARY instead of calling G_BroadcastObit

Oracle (line 2563):
```c
G_BroadcastObit( self, inflictor, attacker, killer, actualMOD, wasInVehicle, wasJediMaster );
```

Rust (lines 5251–5259):
```rust
if (*self_).s.eType != ET_NPC && g_noPDuelCheck == QFALSE {
    ent = G_TempEntity(&(*self_).r.currentOrigin, EV_OBITUARY);
    (*ent).s.eventParm = meansOfDeath;           // should be actualMOD
    (*ent).s.otherEntityNum = (*self_).s.number;
    (*ent).s.otherEntityNum2 = killer;
    (*ent).r.svFlags = SVF_BROADCAST;
    //		ent->s.isJediMaster = wasJediMaster;
}
```

The ported `G_BroadcastObit` function exists in the same file and handles: resolving `wasInVehicle` to `lookTarget`, overriding MOD to `MOD_VEHICLE` for `vehicle_proj` inflictors, setting `brokenLimbs` when attacker is in a vehicle, setting `saberInFlight` when the attacker is a `func_rotating`, and propagating `wasJediMaster` → `isJediMaster`. The Rust `player_die` bypasses all of that with a stripped-down inline block that also uses `meansOfDeath` instead of `actualMOD`.

---

#### Finding 14 — player_die: GT_JEDIMASTER scoring branches commented out

Oracle (lines 2638–2647, 2651–2673):

Two full `if (g_gametype.integer == GT_JEDIMASTER)` blocks handle score-to-jmEnt (enemy kill path) and ThrowSaberToAttacker + `isJediMaster = qfalse` (self-kill and enemy-kill paths). Both are active code in the C oracle.

Rust (lines 5301–5312, 5314–5338): both blocks are enclosed in `/* ... */` comments. `GT_JEDIMASTER` is not imported. All Jedi-Master-mode scoring — including points awarded to the active Jedi Master for non-participants' kills, and saber-return on JM death — is silently skipped.
