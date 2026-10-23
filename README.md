# Kowari

> A systems language with **affine resource types**, explicit allocators,
> function-scoped references, and first-class WASM hosting.
>
> **Status: pre-alpha — M0 (scaffolding).** The compiler does not yet compile
> programs. See [Current status](#current-status) below.

**Kowari** is named after the [kowari](https://en.wikipedia.org/wiki/Kowari)
(*Dasyuroides byrnei*), a small carnivorous marsupial.

---

## Design in one paragraph

**Kowari** makes resource leaks, double-frees, and dangling-references **compile
errors** rather than runtime bugs. Every `resource` type must be consumed
exactly once (`deinit`); every reference (`&T`, `&mut T`) is valid only for
the enclosing function. There is **no global allocator** — every allocation
goes through an allocator you pass explicitly. WASM modules are hosted
in-process via [Wasmtime](https://wasmtime.dev/), so a Kowari binary can call
into a `.wasm` guest with the same safety guarantees.

The full design is in [`SPEC.md`](Spec.md). The development plan — scope,
milestones, testing strategy — is in [`PLANNING.md`](Planning.md). **The spec
is authoritative where the two documents differ.**

## Current status

**Milestone 0 (scaffolding), in progress.** What works today:

- ✅ Rust workspace with 11 crates ([PLANNING.md §4.1](Planning.md))
- ✅ CI: `fmt --check`, `clippy -D warnings`, `test`, `build --release` on Linux + Windows
- ✅ Skeleton of the lexer, parser, typeck, IR, codegen, driver, and test harness
- ⏳ Minimal lexer and golden-test harness (last piece of M0)
- ❌ Everything else

**You cannot compile a `.kowari` program yet.** The first milestone that
produces a running binary is M1a (see [PLANNING.md §5](Planning.md)).
When M0 is done, this section will say so.

## Repository layout

```
crates/          Compiler, one crate per phase (lexer → parser → typeck → IR → codegen)
stdlib/          Standard library, written in .kowari (empty until M2)
tests/           Golden tests: compile_pass/ and compile_fail/ (see PLANNING.md §6)
examples/        Runnable demos (empty until M1a)
benchmarks/      Criterion benchmarks (empty until M2)
docs/            User and contributor documentation
scripts/         ci.sh (local CI mirror), build-llvm.sh (placeholder until M1a)
```

See [PLANNING.md §4.1](Planning.md) for the full annotated tree.

## Building

Prerequisites:

- **Rust** — the pinned toolchain in `rust-toolchain.toml` is installed
  automatically by `rustup`. No manual setup required.
- **LLVM** — **not required yet.** M0 uses no LLVM. `scripts/build-llvm.sh`
  is a placeholder that will install the pinned LLVM version at M1a, when
  [inkwell](https://github.com/TheDan64/inkwell) integration begins.

```bash
git clone https://github.com/kowari-lang/kowari
cd kowari
cargo build --workspace
```

The compiler binary lands at `target/debug/kowaric`:

```bash
cargo run -p kowari-driver -- --help
```

## Testing

```bash
cargo test --workspace      # unit + golden tests
cargo fmt --all --check     # formatting
cargo clippy --workspace --all-targets -- -D warnings
```

Or run the full CI locally:

```bash
scripts/ci.sh               # bash — Linux, macOS, WSL, Git Bash on Windows
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the workflow, and
[PLANNING.md §14](Planning.md) for a concrete "first contribution" recipe.
The short version:

1. Read `SPEC.md` (§3–§6 especially) and `PLANNING.md` (§3, §4, §6).
2. Find a diagnostic code in `docs/internals/diagnostics.md` without a
   golden test, write one, open a PR. This exercises the full toolchain
   without needing compiler-internal knowledge.
3. **Do not change `SPEC.md` without opening an issue first.** The spec is
   the contract; design changes go through discussion.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option. Unless you explicitly state otherwise, any contribution you
submit for inclusion in the project (as defined in the Apache-2.0 license)
shall be dual-licensed as above, without any additional terms or conditions.
