# Kowari

> **Status: pre-alpha — M0 complete, M1a (parser) starting.**
> The lexer and golden-test harness are done. The compiler does not yet
> compile programs. See [Current status](#current-status) below.

**Kowari** is a systems language with **affine resource types**, explicit
allocators, function-scoped references, and first-class WASM hosting.
The full design is in [`Spec.md`](Spec.md); the development plan is in
[`Planning.md`](Planning.md).

## Current status

**Milestone 0 (scaffolding), complete.**

What works today:

* ✅ Rust workspace with 11 crates (see `Planning.md` §4.1).
* ✅ CI: `fmt --check`, `clippy -D warnings`, `test`, `build --release` on Linux + Windows.
* ✅ **Hand-written lexer** (`crates/kowari-lexer`) with the full M0 token set:
    * All single- and multi-character operators, including overflow-policy
      operators (`+%`, `+?`, `+|`, etc.) and `??` for option defaults.
    * Keywords, identifiers, integers (decimal, `0x`, `0b`, `0o`, with
      separators and type suffixes), floats, strings, chars, line comments,
      and nested block comments.
    * Non-panicking error recovery with stable diagnostic codes.
    * Zero-copy tokens (literals and identifiers borrow from the source).
    * `Display` impls and a `dump_tokens` helper for golden-file output.
* ✅ `kowari-diagnostics` code registry with descriptions.
* ✅ **Golden-test harness** (`crates/kowari-lexer/tests/golden.rs`):
  directory-driven, LF-normalised, with `UPDATE` / `--ignored` regeneration.
  16 seed cases covering every literal kind, every diagnostic, operators,
  keywords, comments, and edge cases.
* ⏳ **Next:** M1a (parser). See `Planning.md` §5.
* ❌ Everything else.

**You cannot compile a `.kowari` program yet.** The first milestone that
produces a running binary is M1a.

## Lexer notes

The lexer is **hand-written**, not derive-based. It lives in
`crates/kowari-lexer/src/` (`lib.rs` for the scanner, `token.rs` for the
token type). It is a single-pass, `CharIndices`-based design.

**Guarantees**

* **Zero-copy.** `Integer`, `Float`, `String`, `Char`, and `Identifier`
  borrow from the source (`&'src str`); no per-token allocation.
* **No panics.** Malformed input produces a `Vec<Diag>`; the lexer
  continues after recoverable errors and never unwinds on user input.
* **Stable output.** `dump_tokens(src)` returns a canonical,
  one-token-per-line string used by the golden suite. Span format:
  `Kind @ start..end`.

**Provisional behaviours (pinned by tests, may tighten later)**

* Unicode identifiers are accepted (`is_alphabetic()` / `is_alphanumeric()`).
* String escapes are not validated (`\"` works; `\q` is silently accepted).
* Raw newlines inside string literals are accepted.
* Float and char literals are lexed but not yet specified in `Spec.md` §11.

**Reserved tokens**

* `~` (Tilde) is lexed but reserved for future use.

**Primitive types are identifiers.** `i32`, `usize`, `f64`, etc. lex as
`Identifier`, matching Rust/Go/Swift/Zig. `1i32` (with a numeric prefix)
lexes as a single `Integer` token via suffix absorption. This is the
intended design; a golden test (`primitives.kowari`) pins it.

## Repository layout

```
crates/ Compiler, one crate per phase (lexer → parser → typeck → IR → codegen)
stdlib/ Standard library (empty until M2)
tests/ Workspace-level golden tests (populated from M1a)
examples/ Runnable demos (empty until M1a)
benchmarks/ Criterion benchmarks (empty until M2)
docs/ User and contributor documentation
scripts/ ci.sh, build-llvm.sh (placeholder until M1a)
```

Lexer golden tests live at `crates/kowari-lexer/tests/golden/`.

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

### Golden tests

The lexer golden suite compares lex () output against checked-in
.expected files. To regenerate after an intentional change:

```bash
# Linux / macOS / Git Bash
UPDATE=1 cargo test -p kowari-lexer --test golden

# PowerShell
$env:UPDATE = "1"; cargo test -p kowari-lexer --test golden; Remove-Item Env:\UPDATE

# Cross-platform alternative
cargo test -p kowari-lexer --test golden -- --ignored
```

Always review the git diff of .expected files before committing —
regeneration without review turns the golden suite into a tautology.

## Contributing

See CONTRIBUTING.md for the workflow, and
Planning.md §14 for a concrete "first contribution" recipe.

The short version:

    Read Spec.md (§3–§6 especially) and Planning.md (§3, §4, §6).

    Find a diagnostic code in docs/internals/diagnostics.md without a
    golden test, write one, open a PR.

    Do not change Spec.md without opening an issue first. The spec
    is the contract; design changes go through discussion.

## License

Dual-licensed under either of:

    Apache License, Version 2.0 (LICENSE-APACHE)

    MIT License (LICENSE-MIT)

at your option.

Unless you explicitly state otherwise, any contribution you submit for
inclusion in the project (as defined in the Apache-2.0 license) shall be
dual-licensed as above, without any additional terms or conditions.
