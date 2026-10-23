# Contributing to Kowari

Thanks for your interest. This project is pre-alpha (M0); the most useful
contributions right now are **tests** and **diagnostics**, not features.

## Getting set up

1. Install Rust via [rustup](https://rustup.rs/). The pinned toolchain in
   `rust-toolchain.toml` installs automatically on first `cargo` command.
2. `cargo build --workspace` — should be green.
3. `cargo test --workspace` — should be green.

LLVM is **not** required until M1a. `scripts/build-llvm.sh` is a placeholder.

## Before you open a PR

- [ ] `cargo fmt --all --check` is clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean
- [ ] `cargo test --workspace` passes
- [ ] Tests added or coverage preserved per [PLANNING.md §6.4](Planning.md)
- [ ] If a new diagnostic code: it has an entry in
  `docs/internals/diagnostics.md` and a `tests/compile_fail/` golden test
- [ ] If a design decision is affected: `SPEC.md` is updated **in the same PR**

## Design changes

Do **not** change `SPEC.md` without opening an issue first. The spec is the
contract; see [PLANNING.md §14](Planning.md).

## First contribution

The easiest first PR is a new golden test for a diagnostic code that doesn't
have one yet. See [PLANNING.md §14](Planning.md) for the recipe.