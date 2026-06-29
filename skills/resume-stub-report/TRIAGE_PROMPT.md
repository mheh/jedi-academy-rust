# Stub Re-Port Triage — Per-Batch Agent Prompt

Canonical prompt for ONE read-only agent that triages a batch of stubbed files before re-port. Fill in
the file list, then pass verbatim. The agent edits nothing.

---

You are triaging Rust files in the Jedi Academy C++->Rust port to decide whether each needs a faithful
re-port. READ-ONLY analysis: do NOT edit, write, build, or test anything.

Repo root: /Users/milohehmsoth/Developer/Milo/jedi-academy-rust

BACKGROUND: An old "blind-port" process translated each C/C++ file but FABRICATED placeholder
definitions for every external symbol it didn't define locally — opaque `#[repr(C)] struct Foo { _x: [u8; 0] }`
stubs, marker structs, and sometimes wrongly-named or extra "shadow" structs. The correct approach
("import-trust") never defines external types: it imports them via `use crate::<mirrored-dir>::<header>_h::*;`
derived from the C `#include` directives, trusting them to exist. See PORTING_STYLE.md and
skills/faithful-port/SKILL.md.

A file "SUFFERS from the issue" (needs RE-PORT) if it contains FABRICATED definitions — `[u8; 0]` opaque
stubs, marker structs, or `#[repr(C)]`/`extern` placeholder defs — for types/functions/globals NOT defined
in its paired oracle SOURCE file but living in OTHER files (so they should instead be glob-imported).

A file DOES NOT suffer (SKIP) if every `[u8; 0]` corresponds to a struct that is GENUINELY empty/zero-sized
in the oracle source itself, or the type is actually defined in that same source file.

WATCH FOR (these have all occurred and must be reported so the re-port agent can fix them):
- **Invented type names**: a stub named differently from the real oracle type (e.g. `CCRMMission` for the
  real `CRMMission`; `level_t` for `level_locals_t`; `game_interface_t` for `game_import_t`). Report the
  wrong name -> correct name.
- **Shadow/expansion structs**: extra fabricated structs with no oracle counterpart (e.g. `*_full`
  variants layered on top of an opaque stub). Report them for removal.
- **Partially-legitimate headers**: files where SOME structs are genuinely defined in the oracle source
  and must be KEPT, while others are external and must be imported. List which to keep.
- **libjpeg note**: jpeg `.cpp` files defining `JPEG_INTERNALS` pull in jpegint.h, so most jpeg struct
  types are external (jpeglib.h / jpegint.h) — but a few small ones may be locally defined in the `.cpp`.
  Check where each stubbed type is actually defined before ruling.

FILES TO TRIAGE (each Rust file is paired with its oracle source):
{{FILE_LIST}}   <!-- e.g. "1. src/a/b.rs <- oracle/a/b.cpp" lines -->

FOR EACH FILE:
- Read the current Rust file and grep its `[u8; 0]` / fabricated-stub lines.
- Read the paired oracle source's top (its `#include` directives) and find where each stubbed symbol is
  actually defined (THIS source, or external?).
- Decide RE-PORT or SKIP, with a one-line reason and the count of fabricated external stubs. For RE-PORT,
  list the headers the stubbed types should be imported from (the glob targets), plus any invented-name
  corrections, shadow structs to remove, and locally-defined structs to KEEP.

Return a markdown table: File | Verdict | Fabricated external stubs (count) | Import sources (headers) | Reason.
Then a short notes section for any file with invented names, shadow structs, kept-local structs, or a
non-obvious oracle pairing. Be concise; do not dump file contents.
