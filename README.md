# Kowari

**Kowari** is a statically-typed systems programming language for
server-side native executables. It combines compiler-enforced safety
(without a borrow checker), explicit allocators, errors-as-values, and
first-class WebAssembly hosting.

- **Status:** v0 — pre-alpha. The design is specified; the compiler is
  under construction. **Not yet usable.**
- **Design:** [SPEC.md](./SPEC.md) — the authoritative record of all
  language decisions.
- **Plan:** [PLANNING.md](./PLANNING.md) — scope, milestones, roadmap.

## What makes Kowari different

- **Compiler-enforced safety without a borrow checker** — affine resource
  types (`deinit` / `defer` / `errdefer`), function-scoped references,
  and a narrow `unsafe` escape hatch. See [SPEC.md §4](./SPEC.md#4-safety-model).
- **Explicit allocators** — no hidden global allocator, no GC.
  See [SPEC.md §6](./SPEC.md#6-allocators).
- **Errors as values** — `Result<T, E>` and `?`. No exceptions.
  See [SPEC.md §7](./SPEC.md#7-error-handling--panic).
- **Hosts WebAssembly** — your programs call WASM components via an
  embedded Wasmtime runtime. Interop is the point, not an afterthought.
  See [SPEC.md §12](./SPEC.md#12-ffi--wasm-interoperability).

## Building

Requires a Rust toolchain and LLVM. See [CONTRIBUTING.md](./CONTRIBUTING.md)
for the full setup, or:

```sh
scripts/build-llvm.sh   # installs the pinned LLVM
cargo build
cargo test
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.