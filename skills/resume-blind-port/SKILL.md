---
name: resume-blind-port
description: Resume blind Jedi Academy porting from the newest saved handoff before doing any new work. Use when starting a fresh chat, clearing context, continuing unattended porting, or asking to pick back up from the latest handoff.
---

# Resume Blind Port

## Purpose

Restart the blind porting loop from durable repo state, not conversation memory.

Use this skill at the start of a fresh chat after the prior orchestrator wrote a handoff.

## First action

Run `scripts/latest-handoff.sh`.

If it prints a handoff path, read that handoff before reading broad repo context or delegating agents.

If no handoff exists, say so and continue with `skills/blind-port-orchestrator/SKILL.md` from a fresh pairing pass.

## Resume order

1. Read the newest handoff printed by `scripts/latest-handoff.sh`.
2. Read `PORTING_STYLE.md`.
3. Read `skills/blind-port-orchestrator/SKILL.md`.
4. Run `scripts/compare-src-oracle.sh`.
5. Continue from the handoff's next recommended batch, adjusting only for files already committed since the handoff.

## Rules

Do not rely on prior chat context.
Do not build.
Do not test.
Do not run `cargo check`.
Do not run `cargo fmt`.
Do not modify `oracle/`.
Do not add coauthor trailers to commits.

## If context grows

Before the orchestrator context becomes unwieldy, write a new compact handoff under `handoffs/` and stop.
