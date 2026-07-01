# Workspace scaffold — multi-module build (like Raven / OpenJK)

## Why

Raven's original build (`oracle/codemp/JKA_mp.sln`, `oracle/code/JediAcademy.sln`)
and OpenJK's CMake produce **separate artifacts per engine module**: jampgamex86.dll,
cgamex86.dll, uix86.dll, jagamex86.dll, and the jamp / jampded / jasp executables.
Two hard constraints mean one Cargo crate cannot reproduce that:

1. A Cargo **package emits at most one library**, so the four cdylibs must be four crates.
2. game / cgame / ui each define their own `#[no_mangle] vmMain` / `dllEntry` / `trap_*`
   — three of those in one crate is a duplicate-symbol link error.

So the repo is now a **Cargo workspace**, one crate per module.

## How it's structured

- `src/` is unchanged — a single crate-agnostic **source pool** mirroring `oracle/`.
  No ported file was moved or edited for this scaffold.
- Each `modules/<name>/` crate is a **generated manifest** that mounts only its own
  transitive `use crate::…` closure from the pool via `#[path]`. Because every mount
  is rooted the same way, `crate::codemp::game::…` resolves identically inside every
  crate — the faithful analogue of Raven compiling shared files (bg_*, q_shared, G2)
  into each module target.
  - **Seed dirs** (a module's own directory, e.g. `codemp/game`) are mounted wholesale
    by pointing at the pool's existing `mod.rs`.
  - **Dependency files** reached through the closure (e.g. `cgame/animtable`,
    `ghoul2/g2_h`) get a trimmed generated `mod.rs` with `#[path]` leaves.

Generated files carry a `// GENERATED … do not edit by hand` header. Regenerate with
the generator below — never hand-edit `modules/*/src/**`.

## Generator

`scripts/genmod/genmod.py` + one JSON config per crate in `scripts/genmod/`.
Config: `{ "crate", "seed_dirs", ["kind":"bin"], ["reexport":[...]] }`.

```
python3 scripts/genmod/genmod.py scripts/genmod/mp-game.json
```

It prints the closure size, dep files, and any `missing` (referenced-but-absent) modules.

## Members & status (as of this commit)

| Crate | Package | Artifact | Kind | Builds? |
|-------|---------|----------|------|---------|
| modules/mp-game   | jampgame | jampgamex86 | cdylib | **YES — canary** |
| modules/mp-cgame  | cgame    | cgamex86    | cdylib | no (unported deps) |
| modules/mp-ui     | ui       | uix86       | cdylib | no (unported deps) |
| modules/sp-game   | jagame   | jagamex86   | cdylib | no (unported deps) |
| modules/mp-engine | jamp     | jamp        | bin    | no — WIP scaffold |
| modules/mp-ded    | jampded  | jampded     | bin    | no — WIP scaffold |
| modules/sp-engine | jasp     | jasp        | bin    | no — WIP scaffold |

**Canary:** `cargo build -p jampgame` → `target/debug/libjampgame.dylib`. Use this to
confirm the game module still builds as porting proceeds. `cargo build` (whole
workspace) will fail until the other modules' closures are fully ported — expected.

The non-jampgame crates fail only because their closures still contain blind-port
`[u8; 0]` / broken files (the same worklist in `handoffs/stub-report-worklist.md`).
As those files get re-ported, each module's build turns green with no scaffold change.

## Notes / follow-ups

- `src/lib.rs` is now **orphaned** (the old single-crate root); it is not compiled by
  any crate. Left in place as documentation; safe to delete later.
- `build.rs` (workspace root) is jampgame-specific (MSVC `legacy_stdio` link + `oracle`
  parity-C). `modules/mp-game/build.rs` `include!`s it. Its header was changed from
  `//!` to `//` so it is valid at an include site. Under `--features oracle`, the root
  script's relative `oracle_c` path resolves against the mp-game manifest dir — that
  parity build needs the path made workspace-relative (not yet done; default no-oracle
  build is unaffected).
- **Phase 2 (executables):** the seed dirs for jamp/jampded/jasp are first-guess
  (whole platform/engine dirs, `unix` chosen over `win32`/`mac`). Refine against
  Raven's `jk2mp.vcproj` / `WinDed.vcproj` / `starwars.vcproj` file lists, then wire the
  C `main`/`WinMain` from the mounted platform module into each crate's stub `fn main`.
- sp-game / sp-engine closures currently pull some `crate::codemp::…` files — a
  blind-port cross-engine contamination in `code/` sources; fix during SP re-port.
