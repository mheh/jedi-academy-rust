# Discrepancies: src/codemp/game/ai_main.rs vs oracle/codemp/game/ai_main.c

- **Rust:** `src/codemp/game/ai_main.rs` (8 491 lines) — AI-assisted port originally from the grayj Xbox codebase, later re-targeted at the PC oracle.
- **Oracle:** `oracle/codemp/game/ai_main.c` (7 642 lines) — Raven PC release source, the authoritative target.
- **Verdict:** 5 behavioral divergences found, all in the bot chat subsystem. Every `chatObject`/`chatAltObject` field assignment paired with a `BotDoChat` call has been either silently dropped (`BotOrder`, `BotLovedOneDied`) or left as a `/* … */` block comment (`StandardBotAI`). Most critically, the entire `doChat` dispatch block in `StandardBotAI` — which includes an early-return guard that halts the bot's AI frame when a chat is pending and no enemy is visible — is also commented out, producing different control flow every frame a `doChat` would have fired. No Xbox-residue divergences were found; all diffs trace to the same deferred-chat root cause.

## Findings

| Function | Rust line(s) | Oracle line(s) | First divergence | Confidence |
|---|---|---|---|---|
| `BotOrder` | 4134–4135, 4144–4145 | 242–247, 264–268 | rs:4134 — `state_Forced = ordernum` is the last statement; C continues with `chatObject = ent` | `likely-bug` |
| `StandardBotAI` — death block | 6590–6594, 6603–6607 | 6109–6111, 6117–6119 | rs:6590 — `//CHAT: Died` comment, then `/* bs->chatObject = bs->lastHurt; … BotDoChat(bs, "Died", 0); */` | `likely-bug` |
| `StandardBotAI` — kill block | 7131–7135, 7144–7148 | 6439–6441, 6449–6451 | rs:7131 — `//CHAT: Destroyed hated one` comment, then `/* bs->chatObject = bs->revengeEnemy; … BotDoChat(bs, "KilledHatedOne", 1); */` | `likely-bug` |
| `StandardBotAI` — doChat dispatch | 7734–7762 | 7074–7100 | rs:7734 — entire `if (bs->doChat && bs->chatTime > level.time …) { return; } …` block is inside one `/* … */` comment, removing the AI early-return guard and all `EA_Say`/`EA_SayTeam`/`BotReplyGreetings` calls | `likely-bug` |
| `BotLovedOneDied` | 4423–4441 | 5355–5387 | rs:4425 — early-return after `PassLovedOneCheck == 0` skips `chatObject`/`chatAltObject` assignments and `BotDoChat("LovedOneKilledLovedOne")`; rs:4434 has only `//CHAT: Hatred section` where C calls `BotDoChat("Hatred", 1)`; rs:4439 has only `//CHAT: BelovedKilled section` where C calls `BotDoChat("BelovedKilled", 0)` | `likely-bug` |

---

### Detail: `BotOrder` (rs:4087 / oracle:184)

In both the single-client branch (`clientnum != -1`) and the all-clients broadcast loop, the C oracle does:

```c
BotStraightTPOrderCheck(ent, ordernum, botstates[clientnum]);
botstates[clientnum]->state_Forced = ordernum;
botstates[clientnum]->chatObject = ent;          // rs: absent
botstates[clientnum]->chatAltObject = NULL;      // rs: absent
if (BotDoChat(botstates[clientnum], "OrderAccepted", 1))  // rs: absent
{
    botstates[clientnum]->chatTeam = 1;          // rs: absent
}
```

The Rust stops after `state_Forced = ordernum` (rs:4134 single-client, rs:4145 broadcast). There is no comment or stub. The doc-comment on `BotOrder` (rs:4079) acknowledges "The commented-out `BotDoChat` blocks are not ported", but in the oracle these calls are **active code**, not commented out.

---

### Detail: `StandardBotAI` — death block (rs:6571 / oracle:6088)

When a bot dies with a valid `lastHurt`, the C calls `BotDeathNotify(bs)` then:

```c
if (PassLovedOneCheck(bs, bs->lastHurt)) {
    bs->chatObject = bs->lastHurt;
    bs->chatAltObject = NULL;
    BotDoChat(bs, "Died", 0);
} else if (/* hurtByLove check */) {
    bs->chatObject = bs->lastHurt;
    bs->chatAltObject = NULL;
    BotDoChat(bs, "KilledOnPurposeByLove", 0);
}
```

The Rust preserves the outer `if/else-if` structure and calls `BotDeathNotify`, but wraps the field assignments and `BotDoChat` calls in `/* … */` block comments (rs:6590–6594, rs:6603–6607). `BotDeathNotify` itself is fully implemented.

---

### Detail: `StandardBotAI` — kill block (rs:7114 / oracle:6420)

When the current enemy dies or is lost, the C oracle:

```c
if (/* revengeEnemy dead & was hated */) {
    bs->chatObject = bs->revengeEnemy;
    bs->chatAltObject = NULL;
    BotDoChat(bs, "KilledHatedOne", 1);
    bs->revengeEnemy = NULL;
    bs->revengeHateLevel = 0;
} else if (/* simple kill */) {
    bs->chatObject = bs->currentEnemy;
    bs->chatAltObject = NULL;
    BotDoChat(bs, "Killed", 0);
}
```

The Rust preserves `revengeEnemy = null_mut()` and `revengeHateLevel = 0` but wraps the three-line chat setup + call in `/* … */` in each branch (rs:7131–7135, rs:7144–7148). State mutations that affect future AI (revenge clearing) are correct; only the outgoing chat is lost.

---

### Detail: `StandardBotAI` — doChat dispatch block (rs:7734 / oracle:7074)

This is the most impactful divergence because it affects control flow, not just chat output. The C oracle block:

```c
if (bs->doChat && bs->chatTime > level.time && (!bs->currentEnemy || !bs->frame_Enemy_Vis))
{
    return;   // <-- halts ALL remaining AI for this frame
}
else if (bs->doChat && bs->currentEnemy && bs->frame_Enemy_Vis)
{
    bs->doChat = 0;
    bs->chatTeam = 0;
}
else if (bs->doChat && bs->chatTime <= level.time)
{
    if (bs->chatTeam) { trap_EA_SayTeam(bs->client, bs->currentChat); bs->chatTeam = 0; }
    else              { trap_EA_Say(bs->client, bs->currentChat); }
    if (bs->doChat == 2) { BotReplyGreetings(bs); }
    bs->doChat = 0;
}
```

In Rust, the entire block (rs:7735–7761) is inside a single `/* … */` block comment. As a result:
- The early-return guard is never evaluated: bots always proceed through the full AI frame even when `doChat` is set and no enemy is visible.
- `doChat` and `chatTeam` state are never reset mid-frame.
- `EA_SayTeam`, `EA_Say`, and `BotReplyGreetings` are never called from this site (though `BotReplyGreetings` is otherwise fully implemented and reachable).

---

### Detail: `BotLovedOneDied` (rs:4385 / oracle:5315)

Three call sites are silently dropped:

**Site A** — oracle:5355–5360, rs:4423–4426  
When `!PassLovedOneCheck(bs, loved->lastHurt)` (a loved one killed a loved one), C sets `chatObject`/`chatAltObject` and calls `BotDoChat("LovedOneKilledLovedOne", 0)` before returning. Rust returns immediately without the field assignments or chat call. Control flow (the `return`) is preserved; the chat invocation is not.

**Site B** — oracle:5371–5375, rs:4432–4435  
When `revengeHateLevel` first reaches `loved_death_thresh`, C sets `chatObject`/`chatAltObject` and calls `BotDoChat("Hatred", 1)`. Rust has only a `//CHAT: Hatred section` comment inside the `if` block.

**Site C** — oracle:5380–5385, rs:4437–4441  
In the `else if (revengeHateLevel < loved_death_thresh - 1)` branch, C calls `BotDoChat("BelovedKilled", 0)` then updates `revengeHateLevel` and `revengeEnemy`. Rust has only `//CHAT: BelovedKilled section` and performs the state updates. The revenge-state mutations are correct; only the outgoing chat is lost.
