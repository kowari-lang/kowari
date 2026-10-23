# SPEC.md — Kowari Language Specification

> **Kowari.** Status: **design, v0.** This document is the authoritative record of design decisions. Planning, milestones, and implementation sequencing live in `PLANNING.md`.

---

## 1. Vision

**Kowari** is a statically-typed systems programming language targeting **server-side native executables**, implemented in **Rust**, that achieves **compiler-enforced safety without a borrow checker** by combining **affine resource types**, **explicit allocators**, **errors-as-values**, and **structural rules that prevent aliasing and dangling references** — with **LLVM** as the native backend and **WebAssembly hosting** as the primary cross-language interop mechanism.

---

## 2. Non-Goals (v1)

These are explicitly *out* of scope. They may be revisited later but are not part of v1.

- No borrow checker (affine types instead — see §4).
- No garbage collector.
- No exceptions, no `catch`/`try`/`finally`.
- No unwinding on panic (panic = abort).
- No user-defined operator overloading.
- No function overloading.
- No self-hosting.
- No async/await in v1 (concurrency model: threads — see §10).
- No comptime / staged execution.
- No declarative or procedural macros in v1 (a `format` compiler intrinsic covers string formatting — see §9).
- No package manager / registry (single-package builds; module system designed to allow one later).
- No incremental compilation (designed-for, not built).
- No WIT / Component Model in v1 (raw core-WASM imports only; WIT is v2 — see §12.2).
- No reflection.

---

## 3. Core Design Decisions

| Area | Decision |
| :--- | :--- |
| Implementation language | Rust |
| Typing | Static, strong, no implicit conversions, no `void*` |
| Safety model | Compiler-enforced via **affine resource types** + restricted references + explicit `unsafe` |
| Memory management | **Explicit allocators** (vtable interface); no global allocator, no GC |
| Cleanup | **`deinit` + `defer`/`errdefer`** (Zig-style). No `Drop`. |
| Errors | `Result<T, E>` values; `?` propagation; **exact-match only** (no `From`-conversion in v1) |
| Nullability | `Option<T>`; references are non-nullable |
| Panic | `panic = abort` (no unwinding) |
| Overflow | Panics; wrapping operators (`+%`, `*%`, …) for explicit opt-out |
| Native backend | LLVM |
| WASM role | **Host** WASM components via embedded runtime (Wasmtime) — not emit WASM as primary output |
| Compilation model | **Per-module** with serialized interface metadata; whole-program build initially, structure ready for incremental |
| Generics | **Monomorphization** |
| Traits | Rust-style (`struct`/`trait`/`impl`); **no** operator overloading, **no** function overloading |
| Concurrency | OS threads + `Mutex<T>` + channels + structured scoping; async deferred |
| Mutability | Immutable by default; `let` / `let mut` |
| References | Function-scoped; non-nullable; never stored in aggregates; never returned |
| Strings | `String` (owned) + `&str` (borrowed, UTF-8 invariant) |
| FFI | Both **C ABI** and **WASM hosting**, layered |
| Primary platform | Server-side native executables |

---

## 4. Safety Model

The language's safety story without a borrow checker rests on five structural rules that *together* eliminate the classic memory-safety bug classes.

### 4.1 Affine resource types

A type may be marked as a **resource type** (any owning type: `Vec`, `String`, `File`, `MutexGuard`, …). The compiler enforces that every resource value is **consumed exactly once** on every control-flow path.

- **Consumption** is by-value use: passing to a function taking `T`, assigning out of it, calling `deinit(self)`, or registering it in a `defer`.
- Leaking a resource (letting it go out of scope without consuming it) is a **compile error** (`E0142`).
- `deinit(self)` **consumes `self`**, so calling it twice is a compile error.
- `mem::forget(x)` consumes `x` without running `deinit`; it is `unsafe` (programmer takes responsibility for the leak).

This is the single largest safety mechanism: it statically catches leaks, double-frees, and use-after-`deinit`.

### 4.2 References are restricted

| Rule | Consequence |
| :--- | :--- |
| `&T` / `&mut T` are **function-scoped** — valid only within the enclosing function body | No dangling references possible |
| References **cannot be stored in structs, enums, or arrays** | No self-referential types; no view types in aggregates |
| References **cannot be returned** from a function | No escaping borrows; return owned values instead |
| References **cannot be captured by escaping closures** | Closures that escape capture by move |
| `&T` is **always read-only** (no interior mutability through `&`) | See §4.3 |
| `&mut T` is **unique** within its scope; a second `&mut` to the same local is a compile error; aliasing `&mut` **across a function call** is `unsafe` | Prevents aliasing UB, with an honest escape hatch |
| A `defer` body may only capture references to locals declared **before** the `defer` | Prevents use-after-`deinit` |

### 4.3 Mutability

- Values are **immutable by default**; `let mut x = …` opts into mutation.
- `&T` is read-only. Shared mutable state **must** go through `Mutex<T>`.
- For single-threaded interior mutability of `Copy` types, `Cell<T>` provides `get`/`set`/`swap` by value — no borrows, no lifetimes.
- `MutexGuard<T>` (see §10) is the only way to obtain a `&mut T` from shared state.

### 4.4 The `unsafe` escape hatch

`unsafe` relaxes compiler checks. It does **not** allow undefined behavior — it is a promise that the programmer has upheld the language's invariants.

**Operations requiring `unsafe` in v1:**
- Dereferencing raw pointers (`*const T`, `*mut T`).
- Calling `extern "C"` / `extern "wasm"` functions.
- `mem::transmute` / bit-casting.
- `mem::forget`.
- Creating aliasing `&mut` across a call boundary.
- Unchecked indexing (`get_unchecked`).
- Accessing `union` fields.

**Invariants `unsafe` code must uphold:** no dangling references, no aliasing `&mut`, no use-after-free, no double-free, no data races, no out-of-bounds access, no uninitialized reads, valid bit patterns for the type.

**Convention:** every `unsafe` block must be preceded by a `// SAFETY: …` comment on the line above (or at end-of-line for one-liners) stating *why* the block upholds the invariants above. `unsafe fn` requires a `// SAFETY:` comment above the function explaining its caller-side preconditions.

**The `// SAFETY:` marker is a comment, not code.** It is parsed by the linter for adjacency and presence, but has **no semantic effect** — it does not affect type checking, codegen, or optimization. Removing all `// SAFETY:` comments leaves the program's behavior identical; only the linter's verdict changes.

**Severity:** warn-by-default; **deny-in-stdlib** (all stdlib `unsafe` must be documented); projects may promote to deny globally for safety-critical code.

### 4.5 Summary of invariants

Safe code guarantees:
1. No null dereferences (`&T` non-nullable, `Option<T>` explicit).
2. No dangling references (function-scoped refs).
3. No use-after-free or double-free (affine resources + consuming `deinit`).
4. No leaks (affine check).
5. No out-of-bounds access (bounds-checked, panic).
6. No uninitialized reads (guaranteed initialization).
7. No data races (shared mutation only via `Mutex`).
8. No aliasing `&mut` (unique, function-scoped; cross-call aliasing is `unsafe`).

---

## 5. Resource Management: `deinit` / `defer` / `errdefer`

No `Drop`. Cleanup is explicit and structural.

```kowari
let v = Vec::init(alloc);
defer v.deinit();          // runs at scope exit: normal return, ?, break, continue
v.append(1);
v.append(2);
```

### 5.1 Semantics

| Keyword | When it runs |
| :--- | :--- |
| `defer expr` | On **any** scope exit: normal return, `?` early return, `break`/`continue` out of a block |
| `errdefer expr` | Only on **error-path** exit (`?` returning `Err`) |

- Deferred expressions run in **LIFO order**.
- A `defer`/`errdefer` body **must not** `return`, `break`, `continue`, or `?` (compile error).
- On `panic` (abort), `defer` does **not** run — the process dies.
- Every resource-typed local must be consumed on every path; a `defer` covering it counts as consumption.

### 5.2 `deinit` signature

```kowari
fn deinit(self);         // consumes self; allocator is stored inside self (see §6)
```

### 5.3 Allocator lives inside the collection

Owning collections store their allocator handle (a small `Copy` vtable pair). This means `deinit(self)` needs no allocator argument — unlike Zig — at the cost of ~2 words per collection value. This removes the largest source of allocator-threading pain.

---

## 6. Allocators

Explicit, vtable-based (Zig-inspired), threaded by explicit parameter or stored inside owning types.

```kowari
trait Allocator {
    fn alloc(self, size: usize, align: usize) -> Result<*mut u8, AllocError>;
    fn realloc(self, ptr: *mut u8, old_size: usize, new_size: usize, align: usize)
        -> Result<*mut u8, AllocError>;
    fn free(self, ptr: *mut u8, size: usize, align: usize);
    fn remap(...) -> ...;      // optional
}
```

### 6.1 Fallible core, panic-y convenience

- `alloc`/`realloc` **return `Result`** — allocation failure is recoverable.
- Convenience methods that panic on OOM (`alloc_or_panic`) exist for cases where failure is genuinely unrecoverable.
- This composes with "recoverable errors are values" (§7): OOM is a `Result`, programmer errors are panics.

### 6.2 Shipping allocators (stdlib)

- `GeneralPurposeAllocator` (backed by the system allocator; supports leak detection in debug).
- `ArenaAllocator` (bump; bulk-free).
- `FixedBufferAllocator` (stack/static buffer).
- `DebugAllocator` (tracks allocations, poison on free — for testing).

### 6.3 Threading rule

The allocator is always **explicit** in an API — either as a parameter, or stored inside an owning type at `init` and consulted by its methods. **No implicit/global/context allocator.** This is the Zig model, and it keeps allocations visible at call sites.

---

## 7. Error Handling & Panic

### 7.1 `Result<T, E>`

```kowari
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

- Recoverable errors **are values** (`Result`).
- `Result` is `#[must_use]`-annotated: discarding one is a **warning**, not a panic.
- No exceptions, no `catch`/`try`/`finally`.

### 7.2 The `?` operator

`expr?` means: if `Ok(v)`, evaluate to `v`; if `Err(e)`, return `Err(e)` from the enclosing function.

- **Exact-match only** in v1: the enclosing function's error type must equal the callee's. (Multi-error functions use a user-defined error `enum` + `match`; `From`-conversion is a v2 consideration.)
- `?` also works on `Option<T>` (returns `None` early).
- When `?` returns early, **all registered `defer`s and `errdefer`s run** (LIFO).

```kowari
fn read_config() -> Result<Config, ConfigError> {
    let text = read_file("config.toml")?;
    let parsed = parse(text)?;
    Ok(parsed)
}
```

### 7.3 Panics

Programmer errors — bounds-check failure, integer overflow, division by zero, `panic!`, `unreachable!`, `assert!` failure — **panic**.

- **Panic = abort.** No unwinding. `defer`/`errdefer` do **not** run on panic. The process dies with a diagnostic.
- **No recovery from panic.** There is no `catch`.
- All checks (bounds, overflow) are **on in both debug and release builds** for predictable behavior.

### 7.4 What is *not* a panic

Allocation failure (`Result`), I/O errors (`Result`), parsing failures (`Result`), user-defined recoverable conditions (`Result`).

---

## 8. Pattern Matching

Rust-style `match`, simplified for the no-borrow-checker world.

- **No fallthrough.** Every arm is a pattern → expression; exhaustive by construction.
- **Exhaustiveness is mandatory.** Enums, booleans, tuples checked fully; integer matches require a `_` wildcard.
- **Pattern forms:** literals, ranges (`1..=5`), tuples, struct destructuring, enum variants with bindings, or-patterns (`A | B`), guards (`if x > 0`), `_`, `@` bindings. Slice patterns optional.
- **No `ref`/`ref mut` binding modes.** Binding by value moves non-`Copy` values. To match non-destructively, match on `&value`; bindings then borrow. This is *simpler* than Rust's binding modes.
- **`match` returns a value;** arms must agree on type; `!` (never) for diverging arms (`return`, `panic`, `continue`).
- **`if let` and `let else`** as sugar — both supported.
- **`let` destructuring is irrefutable only;** refutable patterns require `match` / `if let` / `let else`.
- **No matching on arbitrary user types.** Patterns apply to structs, enums, tuples, and built-ins.
- Exhaustive matches lower to jump tables / decision trees; no bounds-check overhead.

---

## 9. Generics, Traits, Modules

### 9.1 Generics

- **Monomorphized** (Rust-style). Slower compiles in exchange for predictable performance and no dispatch cost.
- Slow *clean* builds are acceptable; incremental builds should stay fast (design constraint on the compiler architecture, not the language).

### 9.2 Traits

- Rust-style `trait` / `impl`.
- **No operator overloading.** `+`, `*`, `==`, etc. are only defined for built-in types, *with one exception*:
  - **Generic arithmetic via type-set constraints.** Generic code may use `+`, `<`, `==` when `T` is constrained to a built-in operator set: `fn sum<T: ~int | ~float>(xs: &[T]) -> T`. This is **not** user-defined operator overloading — the built-in operators apply only to built-in numeric types.
- **No function overloading.**
- **No specialization** in v1.
- **Associated types** supported.
- **Coherence/orphan rules** enforced (needed for monomorphization).
- **Formatting** via a `Format`-like trait, with a compiler intrinsic `format(fmt, args…)` for `println`-style ergonomics. Declarative macros are v2.

### 9.3 Modules & compilation

- **Module = file; package = directory with a manifest.**
- **Visibility:** private-to-module by default; `pub`, `pub(package)`, `pub(crate)`-style modifiers.
- **Per-module compilation with serialized interface metadata.** Each module compiles to an object file plus a versioned, self-describing binary metadata blob (public items + signatures). The typechecker consumes metadata, not source.
- **Whole-program build initially** (single package, in-memory), but the compiler is structured as per-module from day one — incremental is a storage-format addition later, not an architecture rewrite.
- **No package manager in v1.** A trivial manifest is acceptable; a registry is v2+.

---

## 10. Concurrency

**v1 model: OS threads + `Mutex<T>` + channels + structured scoping.**

- Threads are the unit of concurrency; a thread is a scope.
- **`Mutex<T>`** is the *only* sanctioned cross-thread shared mutation mechanism.
- **`MutexGuard<T>`** is a resource type: `deinit` releases the lock, `Deref`/`DerefMut` expose `&T`/`&mut T` for the guard's (function-scoped) lifetime. Cannot be stored; cannot cross thread boundaries.
- **Channels** (`mpsc`, `spmc`) for message passing.
- **Atomics** (`AtomicU64` etc.) for lock-free patterns; typed and safe.
- **`Cell<T>`** for single-threaded interior mutability of `Copy` types.

### 10.1 Structured concurrency

Threads are spawned within a **structured scope**: a `thread::scope` block spawns children and joins them at exit. This:
- Makes `defer` work naturally across threads.
- Ensures every thread has a defined lifetime.
- Allows the compiler to guarantee `MutexGuard`s and borrows cannot cross thread boundaries (they're function-scoped; a structured scope's captures are moves).

### 10.2 No `Send`/`Sync` traits needed

Because references are function-scoped and cannot be stored, and because shared mutation requires `Mutex`, the language does **not** need Rust's `Send`/`Sync` auto-traits. Spawned closures capture by move; guards are function-scoped; the type system naturally prevents data races.

### 10.3 Deferred: async/await and M:N

Not in v1. If added later:
- **async/await** with a runtime, with structured concurrency from day one (cancellation runs `defer`/`errdefer`).
- **M:N green threads** would require task stacks to come from an allocator — a natural fit with §6 but a serious runtime project.

The v1 thread model is chosen so that async can be added as a library on top, without changing the language's safety story.

---

## 11. Strings, Integers, Operators

### 11.1 Strings

| Type | Meaning |
| :--- | :--- |
| `String` | Owned, mutable, UTF-8, allocator-stored; resource type |
| `&str` | Borrowed view `(ptr, len)`, UTF-8 invariant; function-scoped (per §4.2) |
| `&[u8]` | Borrowed byte slice; no UTF-8 invariant |

- All string types are UTF-8 guaranteed where marked.
- Slicing returns `&str` when boundaries are on char boundaries (checked; panic otherwise) or `&[u8]` otherwise.
- **FFI representation:** `(ptr, len)` pair — never NUL-terminated. A `CStr`-style wrapper is provided for C interop only.

### 11.2 Integers

- Fixed-width: `i8`/`i16`/`i32`/`i64`/`i128`, `u8`/`u16`/`u32`/`u64`/`u128`, `usize`/`isize`.
- **Overflow panics** in both debug and release.
- **Wrapping operators** for explicit opt-out: `+%`, `-%`, `*%`, `<<%`, etc.
- **Checked operators**: `+?` returns `Option<T>` (`None` on overflow).
- **Saturating operators**: `+|` / `-|` / `*|`.
- **Division by zero** panics.

### 11.3 Operators

- **No user-defined operator overloading** (see §9.2).
- `+`, `-`, `*`, `/`, `%`, `<<`, `>>`, `&`, `|`, `^`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`, `!` are defined for **built-in types only**.
- `==` and ordering are structural for built-in types; user types use trait methods `eq` / `cmp` (not the operators).
- `[]` indexing is built-in for arrays and slices only.
- `->` for function return types; `.` for method/field access.
- Assignment `=`; compound assignment `+=` etc. (for built-in types).

---

## 12. FFI & WASM Interoperability

Interop is **layered**. Each layer is additive; you can stop at any layer.

| Layer | Mechanism | Default |
| :--- | :--- | :--- |
| **Raw C ABI** | `extern "C"` fns, `repr(C)` structs, opaque pointers | `unsafe` |
| **Raw core-WASM imports** | `extern "wasm"` fns, memory/table imports, `i32`-encoded ABI | `unsafe` |
| **Safe stdlib wrappers** | Rust-facing APIs around each boundary | Safe |
| **WASM Component Model / WIT** | Typed bindings from WIT definitions | **v2** |

### 12.1 C ABI (v1)

- `extern "C"` declarations; `repr(C)` on structs; opaque `*mut c_void` for foreign types.
- Error convention: return an `int32` code + out-parameter for the value, or a tagged struct.
- Strings cross as `(ptr, len)`.
- All C calls are `unsafe`.

### 12.2 WASM hosting (v1)

The language **hosts** WASM modules via an embedded runtime (**Wasmtime**). It does **not** primarily emit WASM.

- Stdlib module `wasm` provides: `Instance::load`, `Instance::call`, `Memory`, `Global`, `Table`.
- WASM calls return `Result<T, Error>`; traps map to `Error::from_wasm_trap` (see §12.4).
- The host and guest share linear memory via an explicit memory handle; string/struct marshalling is `(ptr, len)` over that memory.
- **WIT / Component Model is v2.** v1 uses raw core-WASM imports, which are stable and tooling-light. WIT is an additive layer later.

### 12.3 WASM output (not v1)

LLVM's `wasm32`/`wasm64` target is available if the language later wants to compile *to* WASM (for browser targeting). This is **independent** of hosting and is not a v1 goal.

### 12.4 Unified error representation

All FFI wrappers return the language's own `Result<T, Error>`. `Error` is a single concrete struct:

```kowari
struct Error {
    kind: ErrorKind,   // Io, Network, Wasm, Ffi, Timeout, Other
    code: i32,         // raw errno / WASM error code, 0 if n/a
    message: String,   // strerror / trap text
}
```

Boundary wrappers do the translation once (`Error::from_errno`, `Error::from_wasm_trap`, …). Application code never branches on whether an error came from C or WASM.

**Relationship to `From`-conversion (v2).** The unified `Error` and `?`'s `From`-conversion (deferred to v2 per §7.2) are orthogonal mechanisms:

- **Unified `Error`** is a **boundary convention**: every FFI wrapper translates foreign errors (errno, WASM trap, …) into the one concrete `Error` type. Translation happens **once**, inside the wrapper, independent of `?`.
- **`From`-conversion** is an **operator semantics** feature: `?` inserts `E1::from(e)` so a richer error type can absorb `Error` (and others) as it bubbles up.

In v1 the unified `Error` **eliminates most of the pressure for `From`**: applications that use `Result<T, Error>` throughout need only exact-match `?`. Applications defining a richer multi-error enum use `.map_err(Enum::Variant)?` — slightly more verbose, no language machinery. `From`-conversion in v2 removes that verbosity without changing anything about the unified `Error`.

---

## 13. `unsafe`

See §4.4. In brief:

- `unsafe` **blocks** relax compiler checks inside; `unsafe fn` moves the burden to the caller.
- **Allowed inside `unsafe` (v1):** pointer dereference, `extern` calls, `transmute`, `mem::forget`, aliasing `&mut`, unchecked indexing, `union` access.
- **Not allowed even inside `unsafe`:** UB. `unsafe` is a promise not to trigger it; the invariants listed in §4.4 still apply.
- **Comment convention:** `// SAFETY: …` precedes every `unsafe` block (warning if absent).

---

## 14. Tooling & Testing (v1 commitments)

### 14.1 Compiler-internal testing harness

Two golden-test suites, in place **before** the first vertical slice:

| Suite | Asserts |
| :--- | :--- |
| `tests/compile_pass/*.kowari` | Compiles, runs, matches expected stdout / exit code / stderr |
| `tests/compile_fail/*.kowari` | Fails to compile with a specific, stable diagnostic code (`E0142`-style) |

- Expected output recorded inline (`// EXPECT_STDOUT: …`) or in a sidecar `.expected` file.
- **Diagnostics have stable codes.** Messages may evolve; codes are the contract.

### 14.2 Fuzzing

- `cargo-fuzz` targets for lexer and parser. Neither may panic on any input. Run nightly in CI.

### 14.3 Debug info

- **DWARF + LLDB/GDB support** is a stated goal from the first vertical slice (minimal source-line and variable info). Server-side code is debugged with a debugger; retrofitting this is painful.

### 14.4 `#[test]` (language-level)

Deferred to v2 (requires attribute/macro infrastructure). v1 tests use the golden harness and (in stdlib code) plain functions driven by a Rust test runner.

### 14.5 Out of scope for v1 (tooling)

- Formatter, linter, doc generator, LSP, package manager, built-in test framework. All v2+.

---

## 15. Open Questions

Items still open at v0:

1. **Exact surface grammar.** Rust-like direction is set; token set, declaration syntax, and lifetime-adjacent syntax are TBD.
2. **`Result`/`Error` ergonomics in practice.** Whether a `From`-conversion for `?` is added in v2 (and whether it takes the form of a `From` trait or a dedicated `Try` trait).
3. **`Mutex<T>` API shape.** Whether `lock()` returns `Result<Guard<T>, Poisoned>` (Rust) or panics on poison. Recommendation: return `Result` — but decide before writing the `Mutex` stdlib type.
4. **Format intrinsic details.** Whether `format(fmt, args…)` is a variadic-like compiler intrinsic (v1) or a declarative macro (v2). Recommendation: intrinsic v1, macro v2.
5. **`Cell<T>` vs. `RefCell<T>`.** Whether single-threaded interior mutability needs the runtime-checked `RefCell` variant, or whether `Cell` + `mem::replace` suffices. Recommendation: start with `Cell` only; add `RefCell` if it proves necessary.
6. **WIT-first FFI.** Whether the Component Model layer is WIT-generated from day one, or built as raw core-WASM imports first and WIT added later. **Recommendation: raw core-WASM first; WIT in v2.**
7. **Cranelift debug backend.** Whether to add Cranelift as a fast-compile *debug* backend to avoid gating iteration on LLVM. Recommendation: revisit after the LLVM path works.
8. **Module metadata format.** Concrete serialization format for interface metadata (custom binary, `rkyv`, `bincode`, …). Decide when the second module is needed.
9. **Self-hosting.** Not a v1 goal; language should remain expressive enough for compiler construction, but self-hosting is not a v1 constraint.
10. **Macro system design.** Declarative macros (`macro_rules!`-style) is the recommended v2 direction; token-stream quoting/unquoting and hygiene strategy TBD.
11. **Borrow-checker reconsideration.** The v1 design (function-scoped references, no refs in aggregates, structural `&mut` uniqueness) is a deliberate trade: it gives ~80% of Rust's safety at ~15% of the borrow checker's complexity, but forfeits zero-copy view APIs (`fn get(&self) -> Option<&V>`), borrow-carrying iterators (`for x in &mut vec`), struct borrow-splitting, and `RwLock` read concurrency. **Signals to revisit:** (a) stdlib `get`-style APIs that want to return references recur; (b) iteration becomes closure-driven by default; (c) `unsafe` appears for ergonomic rather than genuinely unsafe reasons (the `// SAFETY:` lint will surface this); (d) `RwLock` is requested for read-heavy workloads; (e) benchmarks show copy overhead dominates. **Migration paths (in order of effort):** (1) borrowed return values gated by a `#[borrows(self)]` marker (local dataflow pass, not full borrowck); (2) opt-in `#[borrow_checked]` on individual functions (gradual borrow checking); (3) full borrow checker (v3-level rearchitecture). **Design constraint that keeps these open:** the compiler IR must keep references as first-class types — do not desugar references away early, or paths (1) and (2) become full rewrites.

---

## Appendix A — Resources

### Language implementation in Rust

- **`rust-langdev`** — curated list of compiler-construction crates.
- **Lexers:** `logos`.
- **Parsers:** `chumsky` (combinators, strong error recovery — recommended), `lalrpop`, `nom`.
- **Diagnostics / LSP:** crates listed in `rust-langdev`.

### Learning compiler construction

- **"Crafting Interpreters"** (Nystrom) — tree-walking + bytecode VMs.
- **`codetesla51/how-to-build-a-programming-language`** — interpreters, bytecode, JIT, AOT with LLVM.

### LLVM

- **LLVM Kaleidoscope tutorial.**
- **"Learn LLVM 17."**

### Rust-style semantics (background for §4–§5)

- **The Rust Reference** — type/ownership model.
- **The Rustonomicon** — ownership, lifetimes, `unsafe`.

### Affine/linear type systems (background for §4.1)

- **Austral** — a minimal language with linear types and `unsafe`; closest existing analog to this design.
- **Linear Haskell / Clean uniqueness types** — theory.

### Allocators (Zig-inspired)

- **Zig `std.mem.Allocator` docs** — vtable interface blueprint.
- **`zigzag_alloc`** (Rust crate) — Zig-inspired allocator model.
- **Rusholme** — compiler whose runtime is written in Zig.

### WASM hosting

- **Wasmtime** (`wasmtime`, incl. `component` module) — recommended Rust runtime.
- **Wasmer** (`wasmer-engine-native`) — WASM-to-native-shared-object approach.
- **WAMR**, **WasmEdge** — C-API runtimes.
- **WebAssembly Component Model docs**, **WASI.dev**, **`wit-bindgen`**, **`waclay`** — v2 reference material.
- **"Server-Side WebAssembly"** (O'Reilly).
- **wasmCloud** — Component Model in production.

---

## Appendix B — Naming

- **Language name:** Kowari (a small carnivorous marsupial — `Dasyuroides byrnei`).
- **Compiler binary:** `kowaric`.
- **File extension:** `.kowari`.
- **Crates:** `kowari-*` (e.g. `kowari-lexer`, `kowari-typeck`, `kowari-codegen-llvm`).
- **Namespaces:** `kowari-lang` (GitHub org, crates.io).
- **Reserved:** **Pademelon** — held for a future tool (candidate: the future package manager, or the formatter/linter). Do not use it for anything else.

*End of spec (v0).*

---
