# Discrepancies: src/codemp/game/ai_wpnav.rs

- **Rust:** src/codemp/game/ai_wpnav.rs
- **PC oracle:** oracle/codemp/game/ai_wpnav.c
- **Xbox grayj:** /private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/8aa39250-5453-41a6-8515-4d0d90e61f9c/scratchpad/grayj/codemp/game/ai_wpnav.c
- **Verdict:** Clean — no behavioral divergences from PC oracle

## Findings

No behavioral discrepancies found.

### Notes on audit scope

Every ported function was compared body-by-body against both C files. Key areas checked:

**`LoadPathData`** — The Xbox grayj file allocates `B_TempAlloc(len + 1)` for `fileString` (Xbox line 2041) and frees `B_TempFree(len + 1)` (Xbox line 2231). The PC oracle uses the fixed size `524288` (oracle lines 2041/2231). The Rust uses `524288`, matching the PC oracle. No Xbox residue.

**`LoadPath_ThisLevel`** — The Xbox build would hit `assert(0)` in the `g_RMG` branch (oracle line 3357, inside `#ifdef _XBOX`). The Rust omits the assert and follows the PC `#else` path. Correct.

**`AcceptBotCommand`** — The `bot_wp_save` handler is wrapped in `#ifndef _XBOX` in both C files (oracle line 3802). The Rust includes it unconditionally, targeting PC. Correct.

**`ConnectTrail`, `RepairPaths`, `G_NearestNodeToPoint`, `G_NodeClearForNext`, `G_NodeClearFlags`, `G_NodeMatchingXY`, `G_NodeMatchingXY_BA`, `G_BackwardAttachment`, `G_RMGPathing`, `BeginAutoPathRoutine`, `SavePathData`** — All guarded by `#ifndef _XBOX` in both C files. Absent from Xbox builds. The Rust implements all of them following the PC oracle. No Xbox residue present in any body.

**`BotWaypointRender` render gate** — The PC/Xbox both use `goto checkprint` when `gWPRenderTime > level.time`. The Rust inverts to an `if gWPRenderTime <= level.time` block that falls through to the checkprint section. Semantically identical.

**Trace calls with `&vec3_origin` in place of C's `NULL` mins/maxs** — Consistent throughout the port (DoorBlockingSection, ConnectTrail, G_RecursiveConnection, etc.). A zero-vector is a degenerate bounding box effectively equivalent to a point trace, and the pattern is applied uniformly across the file. Treated as intentional FFI idiom.

**`SavePathData` `storeString[0] = 0` initialisation** — Explicitly documented as a DEVIATION in the function's doc comment (Rust line 2652). The C reads uninitialised `storeString` via `%s` for the first waypoint's neighbor loop — undefined behaviour in C. The Rust zeroes the buffer first, producing identical output on well-formed runs. Intentional, behaviour-preserving.

**`ConnectTrail` debug-print guard** — The PC oracle guards the "Could not link" error print with `#ifndef _DEBUG` so debug builds always print it. The Rust omits this guard, making the condition purely `behindTheScenes == 0`. Release-build behaviour is identical; debug-build behaviour differs, but this does not affect game logic.

**Shared functions with identical PC/Xbox bodies** — `GetFlagStr`, `G_TestLine`, `TransferWPData`, `CreateNewWP`, `CreateNewWP_FromObject`, `RemoveWP`, `RemoveAllWP`, `RemoveWP_InTrail`, `CreateNewWP_InTrail`, `CreateNewWP_InsertUnder`, `TeleportToWP`, `WPFlagsModify`, `NotWithinRange`, `CanGetToVector`, `CanGetToVectorTravel`, `OrgVisibleCurve`, `CanForceJumpTo`, `OpposingEnds`, `DoorBlockingSection`, `CalculatePaths`, `CalculateJumpRoutes`, `CalculateSiegeGoals`, `CalculateWeightGoals`, `GetObjectThatTargets`, `GetNearestVisibleWPToItem`, `FlagObjects`, `GetClosestSpawn`, `GetNextSpawnInIndex`, `G_RecursiveConnection` — the Xbox grayj and PC oracle bodies are identical for all of these. The Rust translations match the PC oracle faithfully in all cases examined.
