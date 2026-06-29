# Faithful Port — Per-File Agent Prompt

This is the canonical prompt handed to ONE worker agent to port ONE oracle file.
Substitute the placeholders, then pass the result verbatim as the agent prompt.

Placeholders:
- `{{ORACLE_PATH}}` — repo-relative C/C++ source, e.g. `oracle/code/game/NPC.cpp`
- `{{DEST_PATH}}`   — repo-relative Rust destination, e.g. `src/code/game/NPC.rs`
- `{{LINES}}`       — line count of the source file

---

You are doing a faithful, blind (no-compile) C/C++ -> Rust port of one file for the Jedi Academy Rust port.

Repo root: /Users/milohehmsoth/Developer/Milo/jedi-academy-rust

TASK: translate the source file
  {{ORACLE_PATH}}   ({{LINES}} lines)
into a new Rust file
  {{DEST_PATH}}     (create it).

SECTION 1 — read first:
- Read PORTING_STYLE.md (repo root) — binding rules.
- Read the whole source: {{ORACLE_PATH}}.
- Do NOT read from files other than src/codemp/game for examples of symbol use in a live environment - for
  calling conventions and usage patterns only. NOT to copy or verify type definitions and NOT as a style
  source for stubs.

SECTION 2 — imports & unknown symbols (do NOT define or stub anything not in this source file):
- Do NOT define or stub types, functions, globals, or macros that are not defined in this source file.
  No #[repr(C)] structs for external types, no opaque `[u8; 0]` placeholders, no marker structs — nothing.
- Imports and unknown types should be trusted to exist and be importable relative to the paths declared in
  the #include directives at the top of the file you are porting. You should not go look for them or verify
  what their actual structures are. You are to translate exactly what is in your source file, without building.
- Translate each include into a glob `use`, mapping the header `foo.h` to the module `foo_h`, mirroring the
  include's directory under the `crate::` root (the crate mirrors the source tree under src/, with `/` -> `::`
  and `.h` -> `_h`). Resolve each include relative to the directory of the file you are porting, then write it
  as an absolute `crate::...::foo_h` path and import with a glob `::*`. Examples for a file in code/game/:
    #include "g_local.h"              -> use crate::code::game::g_local_h::*;
    #include "b_local.h"              -> use crate::code::game::b_local_h::*;
    #include "../qcommon/q_shared.h"  -> use crate::code::qcommon::q_shared_h::*;
    #include "../game/anims.h"        -> use crate::code::game::anims_h::*;
  If an include path is awkward, mirror it as best you can and trust it resolves; do not verify.
- System/C-library includes (e.g. <stdio.h>, <math.h>) are not modules: use core::ffi / extern "C" as needed,
  not a `_h` import.

SECTION 3 — faithfulness:
- Preserve C symbol names, table/field order, control flow, raw pointers, casts, wrapping arithmetic, integer/
  float promotion, and dangerous behavior (OOB indexing, null patterns). Use core::ffi types; access static mut
  globals via addr_of!/addr_of_mut!.
- Translate the file's OWN definitions (its static globals, file-local structs/enums, tables, and all of its
  functions) fully and faithfully from the source.
- Preserve ALL original source comments verbatim (Raven/id comments, trailing, block, TODO, warnings, table
  notes). Translate only comment syntax. NEVER let a preserved comment become a Rust doc comment: do not start a
  comment with //!, ///, /*! or /** ; if the text begins with '!' or '*', add a space after the // or /*.
- Rust block comments NEST but C block comments do NOT. When you preserve a commented-out C block that itself
  contains inner `/* ... */` pairs, every `/*` must be matched by a `*/` in Rust or the whole rest of the file
  is swallowed. C closes the OUTER `/*` at the FIRST `*/`, leaving a trailing unbalanced `/*`; to keep the same
  text commented out and parseable in Rust, balance the delimiters (add the closing `*/` the C compiler implied).
- A C identifier named `self` becomes `self_` (NEVER r#self — `self` is an illegal raw identifier). Same for any
  other reserved word that cannot be a raw identifier.
- A C file-local `static` FUNCTION becomes a plain `unsafe fn` (no `pub`, NO `static` keyword — `static fn` is
  not valid Rust). Only `static` DATA (globals) becomes a Rust `static`.
- If the same external symbol is declared more than once in the C source, declare it once in Rust (duplicate
  `extern` items do not parse). Note such dedups in your report.
- Produce syntactically VALID Rust: `let mut` for reassigned locals, correct char/string escapes, no partial
  initialization, omit size_of asserts that would be guesswork.

SECTION 4 — size / output-cap handling (large files):
- A response cannot exceed the output token cap (~32k), so a file over ~1000 lines cannot be written in one tool
  call. Build the file incrementally: Write the first portion, then use Edit to APPEND each subsequent portion in
  order, until the ENTIRE file is translated. Do not stop early, summarize, or elide any function. Every item in
  the source must be fully translated.

SECTION 5 — prohibitions / report:
- Do NOT build, test, run cargo, cargo check, rustc, or cargo fmt.
- Do NOT modify oracle/. Do NOT edit any file other than {{DEST_PATH}} (you may READ any file).
- Do not attempt to register in the module. You are constricted only to your file. Do NOT touch any
  mod.rs, lib.rs, main.rs, or any parent module file to declare/register {{DEST_PATH}} — write only your file.
- FINAL REPORT: lines written; the list of `use` imports derived from the #include directives; any external
  symbols declared once due to duplicate C declarations; any notable faithful-translation decisions; and confirm
  the WHOLE file was translated end to end (no elided functions) and that the file contains no `[u8; 0]` stubs.
