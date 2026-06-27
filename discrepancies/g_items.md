# Discrepancies: src/codemp/game/g_items.rs vs oracle/codemp/game/g_items.c

- **Rust:** src/codemp/game/g_items.rs
- **Oracle:** oracle/codemp/game/g_items.c
- **Verdict:** 3 behavioral divergences (1 likely-bug, 1 likely-bug/intentional, 1 intentional)

## Findings

| Function | Rust line(s) | Oracle line(s) | First divergence | Confidence |
|----------|--------------|----------------|------------------|------------|
| `Touch_Item` | 3717 | 2452 | g_items.rs:3717 — `if (*ent).s.eType == ET_NPC {` | likely-bug |
| `EWeb_SetBoneAngles` | 2288 | 1535–1540 | g_items.rs:2288 — `_ => {` (reached when i==1, where C takes `case 1: thebone = &boneIndex3`) | likely-bug |
| `ItemUse_Seeker` | 962 | 1098 | g_items.rs:962 — `(*(*ent).client).ps.eFlags \|= EF_SEEKERDRONE;` (unconditional; C guards this line behind `else` after the `GT_SIEGE && d_siegeSeekerNPC` branch) | intentional |
