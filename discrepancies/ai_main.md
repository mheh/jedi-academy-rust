# Discrepancies: src/codemp/game/ai_main.rs

- **Rust:** src/codemp/game/ai_main.rs
- **PC oracle:** oracle/codemp/game/ai_main.c
- **Xbox grayj:** scratchpad/grayj/codemp/game/ai_main.c
- **Verdict:** 6 divergences (6 xbox-residue, 0 port-bug, 0 unsure)

## Findings

| Function | Rust line(s) | PC line(s) | Xbox line(s) | First divergence | Class |
|----------|--------------|-----------|--------------|------------------|-------|
| `BotOrder` | 4129–4151 | 240–269 | 242–273 | `ai_main.rs:4134` — single-client path sets `state_Forced` with no BotDoChat | xbox-residue |
| `BotLovedOneDied` | 4423–4442 | 5355–5386 | 5331–5369 | `ai_main.rs:4423` — PassLovedOneCheck branch returns without BotDoChat("LovedOneKilledLovedOne") | xbox-residue |
| `StandardBotAI` — death-notification | 6588–6607 | 6109–6119 | 6094–6107 | `ai_main.rs:6591` — BotDoChat("Died") and BotDoChat("KilledOnPurposeByLove") inside `/* */` | xbox-residue |
| `StandardBotAI` — enemy-clear | 7130–7148 | 6438–6451 | 6425–6442 | `ai_main.rs:7133` — BotDoChat("KilledHatedOne") and BotDoChat("Killed") inside `/* */` | xbox-residue |
| `StandardBotAI` — chat-dispatch | 7734–7762 | 7074–7100 | 7065–7093 | `ai_main.rs:7734` — entire doChat/trap_EA_Say/BotReplyGreetings block inside `/* */` | xbox-residue |
| `BotAISetup` | 2945–2948 | 7575–7615 | 7596–7599 | `ai_main.rs:2945` — eFlagRed/eFlagBlue/droppedRedFlag/droppedBlueFlag set to null, absent from PC oracle | xbox-residue |

### Detail

#### `BotOrder` — OrderAccepted chat (xbox-residue)

PC oracle (lines 240–247, single-client; lines 264–269, team-loop) — **active**:
```c
BotStraightTPOrderCheck(ent, ordernum, botstates[clientnum]);
botstates[clientnum]->state_Forced = ordernum;
botstates[clientnum]->chatObject = ent;
botstates[clientnum]->chatAltObject = NULL;
if (BotDoChat(botstates[clientnum], "OrderAccepted", 1))
{
    botstates[clientnum]->chatTeam = 1;
}
```

Xbox grayj (lines 242–249, single-client) — **commented out** (`/* ... */`); similarly lines 266–273 for the team-loop instance.

Rust (lines 4134–4135 / 4145–4146) — no BotDoChat in either path:
```rust
BotStraightTPOrderCheck(ent, ordernum, botstates[clientnum as usize]);
(*botstates[clientnum as usize]).state_Forced = ordernum;
```
Rust matches Xbox grayj (both calls elided); PC oracle has both active.

---

#### `BotLovedOneDied` — attachment-revenge chat (xbox-residue)

PC oracle (lines 5355–5384) — **three active BotDoChat calls**:
```c
// PassLovedOneCheck branch:
BotDoChat(bs, "LovedOneKilledLovedOne", 0);  // line 5359, then return
// revengeHateLevel == loved_death_thresh branch:
BotDoChat(bs, "Hatred", 1);                  // line 5375
// else: switch-hatred branch:
BotDoChat(bs, "BelovedKilled", 0);           // line 5384
```

Xbox grayj (lines 5333–5366) — all three calls inside `/* ... */` comment blocks; `return` and `revengeHateLevel`/`revengeEnemy` assignments remain active.

Rust (lines 4423–4442) — no BotDoChat calls; only Rust line-comments (`//CHAT: Hatred section`, `//CHAT: BelovedKilled section`) mark the spots:
```rust
if PassLovedOneCheck(bs, (*loved).lastHurt) == 0 {
    //a loved one killed a loved one.. you cannot hate them
    return;  // BotDoChat("LovedOneKilledLovedOne") absent
}
```
Rust matches Xbox grayj for all three missing chat calls.

---

#### `StandardBotAI` — death-notification chat (xbox-residue)

PC oracle (lines 6109–6119) — **active**:
```c
BotDoChat(bs, "Died", 0);
// ...
BotDoChat(bs, "KilledOnPurposeByLove", 0);
```

Xbox grayj (lines 6094–6107) — both calls inside `/* ... */`.

Rust (lines 6588–6607) — both calls preserved as a block comment:
```rust
/*
    bs->chatObject = bs->lastHurt;
    bs->chatAltObject = NULL;
    BotDoChat(bs, "Died", 0);
    ...
    BotDoChat(bs, "KilledOnPurposeByLove", 0);
*/
```
Rust matches Xbox grayj.

---

#### `StandardBotAI` — enemy-clear chat (xbox-residue)

PC oracle (lines 6438–6451) — **active**:
```c
BotDoChat(bs, "KilledHatedOne", 1);
// ...
BotDoChat(bs, "Killed", 0);
```

Xbox grayj (lines 6425–6442) — both calls inside `/* ... */`.

Rust (lines 7130–7148) — both calls inside a block comment:
```rust
/*
    bs->chatObject = bs->revengeEnemy;
    bs->chatAltObject = NULL;
    BotDoChat(bs, "KilledHatedOne", 1);
    ...
    BotDoChat(bs, "Killed", 0);
*/
```
Rust matches Xbox grayj.

---

#### `StandardBotAI` — chat-dispatch block (xbox-residue)

PC oracle (lines 7074–7100) — **active**: the per-frame chat check that sends queued bot speech via `trap_EA_SayTeam` / `trap_EA_Say` and, when `doChat == 2`, calls `BotReplyGreetings(bs)`.

Xbox grayj (lines 7065–7093) — the entire three-branch `if (bs->doChat …)` block is wrapped in `/* ... */`.

Rust (lines 7734–7762) — the same block is preserved verbatim as a Rust block comment:
```rust
/*
    if (bs->doChat && bs->chatTime > level.time && (!bs->currentEnemy || !bs->frame_Enemy_Vis))
    { return; }
    else if (bs->doChat && bs->currentEnemy && bs->frame_Enemy_Vis)
    { bs->doChat = 0; bs->chatTeam = 0; }
    else if (bs->doChat && bs->chatTime <= level.time)
    {
        if (bs->chatTeam) { trap_EA_SayTeam(bs->client, bs->currentChat); ... }
        else              { trap_EA_Say(bs->client, bs->currentChat); }
        if (bs->doChat == 2) { BotReplyGreetings(bs); }
        bs->doChat = 0;
    }
*/
```
Consequence: bots never actually deliver chat to the game (no `trap_EA_Say*` calls fire), even when other active code sets `doChat`/`currentChat`. Rust matches Xbox grayj.

---

#### `BotAISetup` — flag-pointer initialization (xbox-residue)

PC oracle `BotAISetup` (lines 7575–7615) — registers cvars, calls `trap_BotLibSetup()`; **no** eFlagRed / eFlagBlue / droppedRedFlag / droppedBlueFlag assignments.

Xbox grayj `BotAISetup` (lines 7569–7615) — adds four extra assignments before the restart guard:
```c
eFlagRed = NULL;
eFlagBlue = NULL;
droppedRedFlag = NULL;
droppedBlueFlag = NULL;
```

Rust `BotAISetup` (lines 2945–2948) — includes the same four assignments:
```rust
eFlagRed = null_mut();
eFlagBlue = null_mut();
droppedRedFlag = null_mut();
droppedBlueFlag = null_mut();
```
Rust matches Xbox grayj; PC oracle lacks these assignments in `BotAISetup` (the PC only zeroes them implicitly at module load via BSS).

## Audit notes

Full function-by-function three-way comparison performed across the Rust port (8,491 lines),
PC oracle (7,642 lines), and Xbox grayj (7,649 lines). All divergences are xbox-residue; no
port-bugs or unsure cases were found.

**Functions confirmed clean (Rust matches PC oracle):**

`BotMindTricked`, `IsTeamplay`, `BotReportStatus`, `BotStraightTPOrderCheck`, `BotAI_GetClientState`,
`BotAI_GetEntityState`, `BotEntityInfo`, `NumBots`, `AngleDifference`, `BotChangeViewAngle`,
`BotAIRegularUpdate`, `RemoveColorEscapeSequences`, `PlayersInGame`, `OrgVisible`, `WPOrgVisible`,
`BotPVSCheck`, `OrgVisibleBox`, `GetNearestVisibleWP`, `CheckForFunc`, `InFieldOfVision`,
`EntityVisibleBox`, `BotWeaponBlockable`, `MoveTowardIdealAngles`, `BotGetWeaponRange`,
`BotIsAChickenWuss`, `PassStandardEnemyChecks`, `PassLovedOneCheck`, `GetLoveLevel`,
`ScanForEnemies`, `BotDamageNotification`, `CheckForFriendInLOF`, `GetNewFlagPoint`,
`BotWeaponCanLead`, `BotWeaponSelectable`, `PrimFiring`, `KeepPrimFromFiring`, `AltFiring`,
`KeepAltFromFiring`, `UpdateEventTracker`, `BotSurfaceNear`, `WaitingForNow`, `BotCanHear`,
`BotHasAssociated`, `PassWayCheck`, `TotalTrailDistance`, `CheckForShorterRoutes`,
`WPConstantRoutine`, `BotCTFGuardDuty`, `WPTouchRoutine`, `BotScheduleBotThink`,
`BotResetState`, `BotAILoadMap`, `MeleeCombatHandling`, `SaberCombatHandling`, `CombatBotAI`,
`BotAISetupClient`, `BotAIShutdownClient`, `BotDeathNotify`, `StrafeTracing`, `BotScanForLeader`,
`BotReplyGreetings` (function body is active; call-site is the finding above), `CTFFlagMovement`,
`BotCheckDetPacks`, `BotUseInventoryItem`, `Bot_SetForcedMovement`, `BotFallbackNavigation`,
`GetNearestBadThing`, `BotDefendFlag`, `BotGetEnemyFlag`, `BotGetFlagBack`, `BotGuardFlagCarrier`,
`BotGetFlagHome`, `Siege_TargetClosestObjective`, `Siege_DefendFromAttackers`, `Siege_CountDefenders`,
`Siege_CountTeammates`, `CommanderBotCTFAI`, `CommanderBotSiegeAI`, `CommanderBotTeamplayAI`,
`CommanderBotAI`, `GetIdealDestination`, `BotAI`, `BotAIShutdown`, `BotAIStartFrame`,
`StandardBotAI` (all remaining code outside the three commented blocks).
