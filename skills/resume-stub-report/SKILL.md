---
name: resume-stub-report
description: Resume the stub re-port worklist — replace old blind-port files that fabricated `[u8; 0]` opaque stubs with faithful import-trust ports, one batch at a time. Drives the loop triage -> re-port (Sonnet, per skills/faithful-port) -> verify (0 stubs + rustfmt parse-gate) -> commit -> check off, reading state from handoffs/stub-report-worklist.md. Use when continuing the stub cleanup, when asked to "do the next batch" of re-ports, or to regenerate the worklist.
---

# Resume Stub Re-Port

## Purpose

Many `src/` files were produced by the old **blind-port** process, which fabricated opaque
placeholder definitions (`#[repr(C)] struct Foo { _x: [u8; 0] }`, marker structs, sometimes
**wrongly-named** ones) for every external symbol. This skill drives the orchestration that replaces
those files, in worst-first batches, with faithful **import-trust** ports (no stubs — external symbols
are glob-imported from their `#include` headers and trusted to exist).

The per-file porting itself is owned by `skills/faithful-port`. This skill is the **orchestrator**: it
selects work, gates it with triage, fans out re-port agents, verifies, commits, and tracks progress.

## State: the worklist

`handoffs/stub-report-worklist.md` is the durable checklist — every file that still contains
`[u8; 0]` stubs, sorted worst-first, `- [ ]` pending / `- [x]` done. Resume = read it, take the next
unchecked batch.

Regenerate / re-sync it from ground truth at any time:

```sh
# files still containing opaque stubs, with counts, worst-first
git grep -c '\[u8; 0\]' -- 'src/**/*.rs' | awk -F: '{printf "%s\t%s\n",$2,$1}' | sort -rn
```

A file is **done** when `grep -c '\[u8; 0\]' <file>` returns 0. Mark `[x]` only after it is committed.

## The loop (one batch ≈ 6–8 files)

### 1. Pick the batch

Take the next 6–8 unchecked worst-offenders from the worklist. Map each `src/` path to its oracle source:
`src/a/b.rs` -> `oracle/a/b.cpp` (or `.c`); `src/a/b_h.rs` -> `oracle/a/b.h`. Confirm the oracle file
exists and get its line count.

GOTCHA (zsh): the shell is zsh, which does **not** word-split unquoted variables. Resolve `.cpp`/`.c`
with an explicit fallback, not a `for x in $cand` loop:

```sh
if [[ $f == *_h.rs ]]; then ora="oracle/${rel%_h.rs}.h"
else ora="oracle/${rel%.rs}.cpp"; [ -f "$ora" ] || ora="oracle/${rel%.rs}.c"; fi
```

### 2. Triage gate (read-only, ONE agent for the whole batch)

Before spending re-port effort, confirm each file genuinely suffers. Hand the batch to one read-only
agent using `skills/resume-stub-report/TRIAGE_PROMPT.md`. It returns, per file: **RE-PORT** or **SKIP**,
the fabricated-stub count, the headers each stubbed type should be imported from, any **invented type
names** to correct, and any **locally-defined structs to keep**. Only RE-PORT files proceed.

This gate is not optional — it has repeatedly caught fabrication beyond opaque stubs (wrong invented
names like `CCRMMission` for `CRMMission`, `level_t` for `level_locals_t`) that a naive re-port would
copy forward, and files where some structs are legitimately local and must be preserved.

### 3. Re-port (one background Sonnet agent per RE-PORT file)

For each file, build the agent prompt from `skills/faithful-port/AGENT_PROMPT.md` — but instead of
transcribing it, instruct the agent to **read and obey** `PORTING_STYLE.md` and
`skills/faithful-port/AGENT_PROMPT.md` exactly, then supply:

- the placeholder values `{{ORACLE_PATH}}`, `{{DEST_PATH}}`, `{{LINES}}`;
- an **OVERWRITE NOTE**: the destination already exists as a blind-port stub file and must be replaced
  entirely, keeping none of its fabricated definitions (large files >1000 lines: Write first portion,
  Edit-append the rest, no elision);
- the **triage cautions** for that file verbatim: correct any invented names to the real oracle names,
  keep the named locally-defined structs, remove any extra fabricated "shadow" structs.

Default model **Sonnet**. Run agents in the background (`run_in_background: true`), small batches.

### 4. Verify gate (per returned file, AFTER its completion notification)

Do NOT verify a file while its agent is still running — mid-incremental-write files show transient
syntax/`[u8; 0]` states. Wait for the completion notification, then:

- `grep -c '\[u8; 0\]' <dest>` is **0**.
- `grep -c 'r#self' <dest>` is **0** (`self` must be `self_`, never `r#self`).
- Triage-specific: the invented names are gone; any shadow structs are gone; the keep-these structs are present.
- **Doc-comment leak scan**: `grep -nE '//!|/\*!|/\*\*[^*]|///([^/]|$)' <dest>`. These are the real
  hazards. NOT hazards (ignore): `/***...` banners and `////...` dividers — extra stars/slashes make
  them ordinary comments.
- **Parse-gate**: copy to a temp path and `rustfmt --edition 2021 <tmp>` (exit 0 = parses). Do NOT write
  the formatted result back — rustfmt is a syntax check only; preserve port style.

If the parse-gate fails, apply the cheap safety-net fixes and re-check:

- doc-comment leak `//!x` -> `// !x`; `/**x*/` -> `/* x */`; `/*!` -> `/* !`.
- **nested block comment** (`error: unterminated block comment`): Rust block comments NEST, so a `/*`
  inside a preserved/commented C block (or even inside an agent's own port-note, e.g. `cinfo/*mut`) opens
  a nested comment the outer `*/` never closes. Balance the delimiters, or — if it's an agent-authored
  porting note — reword it to remove the stray `/*`.

EXPECTED, not failures: rust-analyzer `unlinked-file` ("not included in the module tree") and
`unresolved import` for not-yet-ported `_h` modules. Import-trust deliberately trusts modules that don't
exist yet; module wiring is a separate later phase. `unexpected_cfgs` warnings (e.g. `feature = "xbox"`)
are also fine — those features aren't declared in Cargo.toml yet.

### 5. Commit & check off

One commit per file, signing disabled (1Password SSH blocks non-interactive signing):

```sh
git -c commit.gpgsign=false commit -m "port <oracle-relative-path>"
```

Then flip the worklist line `- [ ]` -> `- [x]`. Commit worklist updates **separately** with a
non-`port` message (e.g. `worklist: mark batch N done`).

## Shell gotchas

- zsh does not word-split unquoted variables (see step 1).
- `${PIPESTATUS[0]}` after `... | head` is unreliable; check `rustfmt` exit with a separate
  `rustfmt ...; echo $?` rather than reading it through a pipe.

## Hard prohibitions (inherited from faithful-port)

Do not build or test (rustfmt as a throwaway parse-gate only). Do not modify `oracle/`. Do not fabricate
or stub external types. Do not let any agent touch `mod.rs`/`lib.rs`/`main.rs`. No coauthor/generated-by
trailers. Prefer small background batches over one long mega-run.

## Handoff

When pausing mid-batch, the worklist already records progress. If useful, note in `handoffs/` the batch
in flight, which agents have/haven't reported, and the next worst-offenders.
