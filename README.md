# jedi-academy-rust

An AI-assisted port of Star Wars Jedi Knight: Jedi Academy from C++ to Rust.

## What is this

Mostly, this started as fun — pointing an AI at Raven's original Jedi Academy
source and watching it translate decades of C++ into Rust. It worked, so here
it is.

It's trying to be a faithful port for the time being. At odds with rust idioms
as a language to preserve faithfulness.

## Where it could go

A few directions that would be nice:

- **Rust-idiomatic fixes** — the port still carries a lot of C habits.
  Out-of-bounds array access, raw pointer juggling, manual memory management,
  `unsafe` that doesn't need to be unsafe. PRs that make this more idiomatic
  (and safer) are welcome.
- **Rethinking how the game works** — nothing here is sacred. If reorganizing
  systems makes the game easier to understand and mod, that's ok. I'd like to
  find direction in the way faithfulness and 1:1 continuity is preserved as it
  changes

## `oracle`

The [`oracle/`](oracle) submodule holds Raven's original Jedi Academy C++
source. It's the reference the port is compared against when something looks
wrong.

## Building & testing

```sh
# clone with the oracle submodule (or run: git submodule update --init)
git clone --recurse-submodules <repo-url>

cargo build          # builds the jampgame cdylib (pure Rust)
cargo test           # runs the unit tests
```

### Parity tests against the original C

The `oracle` feature compiles extracted Raven C functions with headers from the
oracle subdir. Primarily for math and RNG.

```sh
cargo test --features oracle -- --test-threads=1
```

`--test-threads=1` is required because of single global C state.

## Contributing

Agents are OK — a lot of this was written by one. But only clear, small commits
are accepted. Each commit should do one understandable thing so it can be
reviewed and checked against the `oracle` reference.

## Status

Early and unstructured. Things move, break, and get rewritten. Treat it as a
playground, not a release.

## License

See [LICENSE](LICENSE).
