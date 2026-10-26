# Kowari

> **Status: pre‑alpha — M0 (scaffolding), closing out.**
> The lexer is functional; the remaining M0 work is test coverage and the
> golden‑test harness. The compiler does not yet compile programs. See
> [Current status](#current-status) below.

**Kowari** is a systems language with **affine resource types**, explicit
allocators, function‑scoped references, and first‑class WASM hosting.
The full design is in [`Spec.md`](Spec.md); the development plan is in
[`Planning.md`](Planning.md).

## Current status

**Milestone 0 (scaffolding), closing out.**

What works today:
* ✅ Rust workspace with 11 crates (see `Planning.md` §4.1).
* ✅ CI: `fmt --check`, `clippy -D warnings`, `test`, `build --release` on Linux + Windows.
* ✅ **Hand‑written lexer** with the full M0 token set:
    * All single‑ and multi‑character operators, including overflow‑policy
      operators (`+%`, `+?`, `+|`, etc.).
    * Keywords, identifiers, integers (decimal, `0x`, `0b`, `0o`),
      floats, strings, chars, line comments, and nested block comments.
    * Non‑panicking error recovery with stable diagnostic codes (`E0001`–`E0008`).
    * Unit tests for operators, numeric literals, spans, and diagnostics.
* ✅ `kowari-diagnostics` code registry with descriptions.
* ⏳ **Remaining M0 work:** extend the lexer test suite (see
  [Missing tests](#missing-tests)) and finish the golden‑test harness.
* ❌ Everything else.

**You cannot compile a `.kowari` program yet.** The first milestone that
produces a running binary is M1a (see `Planning.md` §5).

## Lexer notes (M0)

The lexer is **hand‑written**, not derive‑based. It lives in
`crates/kowari-lexer/src/lib.rs` and follows a single‑pass,
`CharIndices`‑based design. Key behaviours to be aware of at M0:

* **Zero‑copy tokens.** Literal and identifier tokens borrow from the source
  (`&'src str`), so no per‑token `String` allocation happens.
* **Diagnostics, not panics.** Malformed input produces a `Vec<Diag>` with
  code, span, and message; the lexer continues after recoverable errors.
* **Provisional edge cases.** Unicode identifiers, escape validation, and
  newlines inside strings are currently accepted without extra diagnostics.
  These behaviours are pinned by tests and may be tightened in later
  milestones.

## Repository layout

```aiignore
crates/ Compiler, one crate per phase (lexer → parser → typeck → IR → codegen)
stdlib/ Standard library (empty until M2)
tests/ Golden tests: compile_pass/ and compile_fail/
examples/ Runnable demos (empty until M1a)
benchmarks/ Criterion benchmarks (empty until M2)
docs/ User and contributor documentation
scripts/ ci.sh, build-llvm.sh (placeholder until M1a)
```

See `Planning.md` §4.1 for the full annotated tree.

## Building

Prerequisites:
* **Rust** — the pinned toolchain in `rust-toolchain.toml` is installed
  automatically by `rustup`.
* **LLVM** — **not required yet.** M0 uses no LLVM.

```bash
git clone https://github.com/kowari-lang/kowari
cd kowari
cargo build --workspace
```

The compiler binary lands at target/debug/kowaric:
bash

```bash
cargo run -p kowari-driver -- --help
```


## Testing
```bash
cargo test --workspace       # unit + golden tests
cargo fmt --all --check      # formatting
cargo clippy --workspace --all-targets -- -D warnings
```
Or run the full CI locally:
```bash
scripts/ci.sh   # bash — Linux, macOS, WSL, Git Bash on Windows
```

## Contributing

See CONTRIBUTING.md for the workflow, and
Planning.md §14 for a concrete “first contribution” recipe.

The short version:

    Read Spec.md (§3–§6 especially) and Planning.md (§3, §4, §6).

    Find a diagnostic code in docs/internals/diagnostics.md without a
    golden test, write one, open a PR. This exercises the full toolchain
    without needing compiler‑internal knowledge.

    Do not change Spec.md without opening an issue first. The spec
    is the contract; design changes go through discussion.

## License

Dual‑licensed under either of:

    Apache License, Version 2.0 (LICENSE-APACHE)

    MIT License (LICENSE-MIT)

at your option.

Unless you explicitly state otherwise, any contribution you submit for
inclusion in the project (as defined in the Apache‑2.0 license) shall be
dual‑licensed as above, without any additional terms or conditions.