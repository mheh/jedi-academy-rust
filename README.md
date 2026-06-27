# jedi-academy-rust

An AI-assisted port of Star Wars Jedi Knight: Jedi Academy from C++ to Rust.

## What is this

Mostly, this started as fun — pointing an AI at Raven's original Jedi Academy
source and watching it translate decades of C++ into Rust. It turned into a
surprisingly large pile of working code, so here it is.

It's not trying to be a faithful preservation project or a grand foundation.
It's a port that happens to compile, and a nice excuse to drag an old codebase
into a safer, more modern language.

## Where it could go

A few directions that would be genuinely nice:

- **Rust-idiomatic fixes** — the port still carries a lot of C habits.
  Out-of-bounds array access, raw pointer juggling, manual memory management,
  `unsafe` that doesn't need to be unsafe. PRs that make this more idiomatic
  (and safer) are very welcome.
- **Rethinking how the game works** — nothing here is sacred. If reorganizing
  systems makes the game easier to understand and mod, that's a feature, not a
  betrayal of the original.
- **Easier modding for newcomers** — the original is powerful but intimidating
  to get into. A long-term goal is making this something a curious person can
  actually crack open and change without years of engine archaeology.

## `oracle`

The [`oracle/`](oracle) submodule holds Raven's original Jedi Academy C++
source. It's the reference the port is compared against when something looks
wrong.

## Status

Early and unstructured. Things move, break, and get rewritten. Treat it as a
playground, not a release.

## License

See [LICENSE](LICENSE).
