# `PLANNING.md` — Kowari Development Plan

> Companion to `SPEC.md`. This document covers **how** Kowari v1 is built: scope, project structure, milestones, testing, tooling, and future plans. It does **not** re-state design decisions — see `SPEC.md` for those. Where the two documents touch, `SPEC.md` is authoritative.

---

## 1. Purpose & Scope

**This document defines:**

- What "v1" means as a deliverable (and what "v1.0" means as a release).
- The structure of the compiler repository.
- The milestones that lead to v1, each with a concrete "definition of done."
- Development workflow, testing strategy, and quality gates.
- A roadmap of future work (v2+, no dates, no versions beyond "v2" placeholder).

**Out of scope:** design decisions (→ `SPEC.md`), marketing, user documentation (→ `docs/` folder once it exists), hiring, or anything time-bound.

---

## 2. Version Strategy

The project uses three version axes that should not be conflated:

| Axis | Meaning | Example |
| :--- | :--- | :--- |
| **Language version** | Semantic version of the *language itself* (what users depend on) | `v1.0`, `v2.0` |
| **Compiler version** | SemVer of the compiler binary implementing a language version | `kowaric 0.3.2` implementing language `v0.9` |
| **Release cut** | A tested snapshot | "v1.0-rc1" |

**Rules:**

- **Language `v1.0`** is declared when §12 (Definition of Done) is met. Before that, the language is `v0.x` and **breaking changes are expected**.
- **Compiler `v0.x`** may track the language closely; when the language reaches `v1.0`, the compiler becomes `kowaric 1.0` in the same release.
- **Post-`v1.0`, language versions follow semver strictly.** Breaking language changes ship only in a `v2.0`.
- The `SPEC.md` document is tagged with the language version it describes: `SPEC.md (v0)` → `SPEC.md (v1.0)` at release.

---

## 3. V1 Scope

V1 delivers **the full v1 feature set described in `SPEC.md`**, implemented to a quality level suitable for server-side use *by the project authors* — not yet for external production use. The distinction matters: v1 is the point where **the design is validated end-to-end**, not where the ecosystem is complete.

### 3.1 In Scope for V1

Everything in `SPEC.md` §3–§14, **except** items listed in §3.2. Highlights:

- Lexer, parser, AST, typechecker (including affine resource check), IR, LLVM codegen, driver.
- Affine resource types, `deinit`/`defer`/`errdefer`, function-scoped references, `unsafe`.
- `Result` + `?` (exact-match + `Option`), `Option`, exhaustive `match` with `if let`/`let else`.
- Explicit allocators: `GeneralPurposeAllocator`, `ArenaAllocator`, `FixedBufferAllocator`.
- Traits, monomorphized generics, no overloading.
- `String` / `&str` / `&[u8]`.
- OS threads, `Mutex`, channels, structured concurrency.
- C FFI (`extern "C"`, `repr(C)`).
- WASM hosting via Wasmtime (`wasm` stdlib module).
- Unified `Error` type at FFI boundaries.
- Per-module compilation with metadata (whole-program build initially).
- Golden test harness + fuzzing + DWARF debug info.
- `format` intrinsic (for `println`-style ergonomics).

### 3.2 Explicitly Deferred from V1

Per `SPEC.md` §2 (non-goals) and §15 (open questions). Not in scope:

- Borrowed return values, borrow-checked functions, full borrow checker (→ v2+).
- `From`-conversion for `?` (→ v2).
- Async/await, M:N green threads (→ v2+).
- WIT / Component Model (→ v2).
- Declarative macros, `#[test]` framework (→ v2).
- Package manager / registry (→ v2+; see Pademelon note in §4.4).
- Incremental compilation, LSP, formatter, linter, doc generator (→ v2+).
- Self-hosting (→ v3+).
- WASM *output* (`wasm32` target) (→ v2+).
- Cranelift debug backend (→ v2).
- `RwLock`, `RefCell` (→ v2, only if the v1 signals justify them; see `SPEC.md` §15.11).

---

## 4. Project Structure

### 4.1 Repository Layout

The compiler is a Rust workspace. Each phase of compilation is its own crate, with a thin driver binary. **References are kept first-class through all IR layers** (per `SPEC.md` §15.11) so that later borrow-checker work remains incremental.

```
kowari/                            # repo root
├── Cargo.toml                     # workspace manifest
├── rust-toolchain.toml            # pin nightly (LLVM bindings often need it)
├── SPEC.md                        # language specification
├── PLANNING.md                    # this document
├── README.md                      # orientation for newcomers
├── CHANGELOG.md                   # per-release notes
├── LICENSE
│
├── crates/                        # Rust crates
│   ├── kowari-lexer/              # tokens + logos-backed lexer
│   ├── kowari-parser/             # chumsky-based parser → AST
│   ├── kowari-ast/                # AST types, shared with parser + typeck
│   ├── kowari-diagnostics/        # diagnostic codes (E0001…), rendering, spans
│   ├── kowari-typeck/             # type checker + affine resource analysis
│   ├── kowari-ir/                 # language IR (HIR + MIR-equivalent) — refs first-class
│   ├── kowari-codegen-llvm/       # IR → LLVM IR (inkwell) → object file
│   ├── kowari-metadata/           # module interface metadata (serialization)
│   ├── kowari-wasm-host/          # Wasmtime wrapper — stdlib runtime support
│   ├── kowari-driver/             # the `kowaric` binary; orchestrates the pipeline
│   └── kowari-test-harness/       # golden-test runner (used by `cargo test`)
│
├── stdlib/                        # the language's standard library (written in .kowari)
│   ├── alloc/                     # Allocator trait + implementations
│   ├── collections/               # Vec, HashMap, BTreeMap, …
│   ├── string/                    # String, &str helpers
│   ├── fmt/                       # Format trait, format intrinsic declarations
│   ├── io/                        # file, stdout/stderr
│   ├── thread/                    # spawn, scope, channels
│   ├── sync/                      # Mutex, Guard, atomics
│   ├── ffi/                       # C ABI helpers, CStr
│   └── wasm/                      # Instance, Memory, call — backed by kowari-wasm-host
│
├── tests/                         # golden test suite (see §6)
│   ├── compile_pass/              # *.kowari — must compile + run + match expected output
│   ├── compile_fail/              # *.kowari — must fail with a specific diagnostic code
│   └── fuzz/                      # cargo-fuzz targets
│
├── examples/                      # user-facing example programs (compiled in CI)
│   ├── hello_world/
│   ├── file_io/
│   ├── wasm_add/                  # WASM host demo (v1 acceptance test)
│   └── concurrent_counter/
│
├── benchmarks/                    # criterion benchmarks for compiler phases + example runtime
│
├── docs/
│   ├── SPEC.md → (symlink to root)
│   ├── tutorial.md                # end-user tutorial (written during milestone 5)
│   ├── reference.md               # end-user language reference
│   └── internals/                 # contributor-facing compiler docs
│
└── scripts/
    ├── ci.sh                      # mirrors what CI runs
    └── build-llvm.sh              # helper for getting LLVM installed
```

### 4.2 Crate Responsibilities (brief)

| Crate | Responsibility | Public API (roughly) |
| :--- | :--- | :--- |
| `kowari-lexer` | Source → `Vec<Token>` | `lex(src) -> Result<Vec<Token>, Vec<Diag>>` |
| `kowari-parser` | Tokens → AST | `parse(tokens) -> Result<Ast, Vec<Diag>>` |
| `kowari-ast` | Shared AST type definitions | — (data only) |
| `kowari-diagnostics` | Diagnostic codes, spans, rendering | `Diag { code, span, message }` |
| `kowari-typeck` | AST → typed HIR, affine check, trait resolution | `check(ast) -> Result<Hir, Vec<Diag>>` |
| `kowari-ir` | HIR → MIR-like IR (refs preserved) | `lower(hir) -> Mir` |
| `kowari-codegen-llvm` | IR → LLVM → object | `compile(mir, opts) -> Result<Object, Diag>` |
| `kowari-metadata` | Module interface metadata serialization | `serialize(mod) -> Vec<u8>`, `deserialize(&[u8]) -> Mod` |
| `kowari-wasm-host` | Wasmtime embedding for stdlib `wasm` module | `load(path) -> Instance`, `call(...)` |
| `kowari-driver` | CLI (`kowaric`), orchestrates the pipeline | `kowaric main.kowari -o prog` |
| `kowari-test-harness` | Runs the golden suite as `cargo test` | `run_all()` |

**Dependency direction is strictly one-way** (`lexer → parser → typeck → ir → codegen`); shared types live in `kowari-ast` / `kowari-diagnostics`. Do **not** let `kowari-parser` depend on `kowari-typeck`.

### 4.3 User's Project Layout (what a user's project looks like)

**Single-file program (v1):**
```
hello.kowari
```
Compile: `kowaric hello.kowari -o hello`

**Directory project (v1, no package manager):**
```
myproject/
├── project.toml      # minimal: name, version, entry
├── src/
│   ├── main.kowari
│   └── util.kowari   # module = file; visible via `use util;`
└── (optional) data files
```

`project.toml` is **minimal** in v1 — no dependencies section; dependencies are not a v1 concept. This matches `SPEC.md` §9.3.

### 4.4 Reserved Names

- **`Pademelon`** — held for a future tool (candidate: the package manager, formatter, or linter). Do not use for anything else; it will be claimed in a future planning revision.

---

## 5. Milestones

Milestones are **vertical slices**: each ends with something runnable end-to-end and a hardened test gate. No milestone ships on a schedule; each ships when its Definition of Done (DoD) is met.

### Milestone 0 — Foundation

**Goal:** Scaffolding, CI, and the golden-test harness so that no future milestone regresses silently.

**Deliverables:**

- Workspace laid out as §4.1; empty crates with stub APIs.
- `rust-toolchain.toml` pinned; LLVM installation script.
- `kowari-diagnostics` with a code registry (`E0001`–`E0999` reserved for compiler; `W0001`– for warnings).
- **`kowari-test-harness`** that discovers `tests/compile_pass/*.kowari` and `tests/compile_fail/*.kowari`, runs them, and reports pass/fail. Initially tested against a trivial "identity" pipeline (lex only).
- CI workflow: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo build --release`, and a check that `examples/` compiles once the driver exists.
- `README.md` that orients a new contributor in <10 minutes.

**DoD:** CI green on a repo that lexes one file and returns its tokens.

**Estimate:** ~1 week.

---

### Milestone 1a — Language-Only Vertical Slice

**Goal:** One `.kowari` program using **every safety mechanism** compiles to native and runs, without WASM. This is the *only* milestone whose demo is non-negotiable: it exercises affine types, `defer`, `Result`+`?`, `Option`+`match`, traits (monomorphized), and explicit allocators. (Math/string demo is a *subset* of this — see `SPEC.md` discussion.)

**Deliverables:**

- Lexer (`logos`) with full token set for the v1 grammar (see §5.1 for the required grammar coverage at this milestone).
- Parser (`chumsky`) with recovery; AST.
- Typechecker: functions, primitives (`i32`/`i64`/`u64`/`bool`/`f32`/`f64`), `struct`, `enum`, `trait`, `impl`, monomorphized generics, `Option<T>`, `Result<T,E>`, `?` on both, `match` with exhaustiveness, `let else`.
- **Affine resource check:** every `struct`/`enum` can be marked `resource`; dataflow pass verifies consumption on every path; `deinit(self)` is the only consumer for resource `deinit`.
- `defer` / `errdefer` — full semantics per `SPEC.md` §5, including LIFO, no early-exit inside, and running on `?`.
- Reference rules per `SPEC.md` §4.2 (function-scoped, not in aggregates, not returned, `// SAFETY:` lint).
- Allocators: `Allocator` trait; `GeneralPurposeAllocator` (backed by a Rust-side `extern "C"` shim, since it wraps libc); `FixedBufferAllocator` (stack-buffer, in-language).
- IR (HIR + MIR) with **references first-class** (§15.11).
- LLVM codegen via **inkwell** (safer than raw `llvm-sys`), producing object files; link step via `cc` or `ld` (whatever `cc` uses).
- Driver CLI: `kowaric path/to/main.kowari -o prog`.
- Minimal DWARF: source-line info on every instruction; function names; local variable names. Enough to set a breakpoint and inspect locals in LLDB.
- **Golden tests:** at minimum
  - `compile_pass/affine_counter.kowari` — resource counter, `defer deinit`, prints 1.
  - `compile_pass/result_propagation.kowari` — nested `?` + `defer` + `errdefer`.
  - `compile_pass/trait_format.kowari` — `Show`/`Format` trait, two impls.
  - `compile_pass/match_exhaustive.kowari` — enum + match.
  - `compile_fail/missing_deinit.kowari` — `E0142`.
  - `compile_fail/ref_in_struct.kowari` — new code for §4.2's "refs in aggregates" rule.
  - `compile_fail/use_after_deinit.kowari` — new code.

**DoD:** The affine-counter demo compiles, runs, prints `1`, exits 0; all golden tests pass in CI; the demo can be debugged in LLDB with `break main` and `print c`.

**Estimate:** ~6–10 weeks (this is the hardest milestone; most of the compiler infrastructure lands here).

---

### Milestone 1b — WASM Hosting

**Goal:** Prove the interop thesis: a `.kowari` binary hosts a `.wasm` module and calls into it.

**Deliverables:**

- `kowari-wasm-host` crate wrapping Wasmtime (embedded runtime).
- `wasm` stdlib module: `Instance::load`, `Instance::call`, `Memory` (read/write/borrow), `Trap` mapping.
- `Error::from_wasm_trap` (§12.4).
- A `calc.wasm` produced outside this repo (from Rust or C, using its own toolchain) — this is the *guest* we test against.
- Example `examples/wasm_add/`: loads `calc.wasm`, calls `add(1.0, 2.0)`, prints `3.0`.
- Golden test `compile_pass/wasm_add.kowari` (with the `.wasm` fixture checked in).

**DoD:** `examples/wasm_add` runs and prints `3.0` on all supported platforms in CI; a trap in the guest surfaces as `Error { kind: Wasm, … }` at the call site.

**Estimate:** ~2–3 weeks.

---

### Milestone 2 — Standard Library Core

**Goal:** Make the language useful for *non-trivial* programs by shipping collections, strings, IO, and formatting.

**Deliverables:**

- `alloc`: `GeneralPurposeAllocator`, `ArenaAllocator`, `FixedBufferAllocator`, `DebugAllocator` (leak-checking, for tests).
- `collections`: `Vec<T>` (full resource-type semantics), `HashMap<K,V>` (with the ergonomic caveat from `SPEC.md` §15.11 — `get` returns `Option<V>` by value at this stage), `BTreeMap<K,V>`, small-vector `SmallVec` optional.
- `string`: `String`, `&str` (function-scoped, UTF-8 invariant), `&[u8]`.
- `fmt`: `Format` trait, `format` intrinsic, `println` / `print` / `eprintln`.
- `io`: `File` (open/read/write/close, all `Result`), `stdin`/`stdout`/`stderr`.
- `ffi`: `extern "C"` bindings for what stdlib needs (`malloc`, `free`, `read`, `write`, `open`, `close`); `CStr` for C-string interop.
- **Golden test coverage** expands to every stdlib module.
- **Benchmarks** (criterion, in `benchmarks/`) for `Vec::push`, `HashMap::insert`/`get`, `String::append`.

**DoD:** A `wc -l`-style program in `.kowari` reads stdin, counts lines, prints to stdout, and compiles+runs from a single `.kowari` file. Benchmarks run in CI and record numbers (not gated on them yet).

**Estimate:** ~4–6 weeks.

---

### Milestone 3 — Concurrency

**Goal:** v1's threaded, structured-concurrency model works end-to-end.

**Deliverables:**

- `thread`: `spawn` (move-capture closures), `scope` (structured join), `JoinHandle`.
- `sync`: `Mutex<T>` + `MutexGuard<T>` (resource type, `deinit` unlocks, `Deref`/`DerefMut`), atomics (`AtomicU8/16/32/64`, `AtomicBool`, `AtomicPtr` — the last `unsafe`), `Ordering`.
- `sync` channels: `mpsc` (`channel`), `spmc`.
- **Data-race safety proof (documented):** why `MutexGuard` + function-scoped refs + spawn-by-move means no data races without `Send`/`Sync` (this is the argument in `SPEC.md` §10.2, formalized as a document in `docs/internals/data-race-safety.md`).
- Example `examples/concurrent_counter/`: N threads, each increments a shared `Mutex<u64>` M times; final count = N×M; runs deterministically in output (only final value printed).
- Golden test `compile_pass/concurrent_counter.kowari`.

**DoD:** `examples/concurrent_counter` runs with `--threads 8 --increments 10000` and prints `80000`; ThreadSanitizer on a nightly build of the *compiler* itself (not the language output) shows no races in the runtime shim.

**Estimate:** ~3–4 weeks.

---

### Milestone 4 — C FFI and Module System

**Goal:** Programs can call into C and can span multiple modules — the last pieces before "use it for real work."

**Deliverables:**

- `extern "C"` declarations with `repr(C)` structs; opaque `*mut c_void` for foreign types.
- A `#[link("libfoo")]`-style annotation (or equivalent in the language — decide during the milestone; the simplest form is a flag on `extern` blocks) to specify linking.
- `kowari-metadata`: interface metadata serialization for modules (`bincode` initially — swap later if needed), consumed by the typechecker in multi-module builds.
- Driver supports multi-file builds: `kowaric src/main.kowari -o prog` finds and typechecks `use`d modules, compiles each separately, links.
- Example `examples/file_io/` uses the FFI-backed `io` module (or, if `io` is fully in-language by now, demonstrates *user* FFI with a small custom C library).
- **`// SAFETY:` lint** implemented in the driver (warn by default; deny in stdlib via an attribute or a build flag).

**DoD:** A multi-file program using a user-defined `extern "C"` function from a small C object file compiles and runs. Metadata round-trips through disk correctly (a `.kowari` module compiled independently can be linked into a second binary without recompiling the source).

**Estimate:** ~3–4 weeks.

---

### Milestone 5 — Polish & Acceptance

**Goal:** Everything the user-facing side of v1 needs to be trustworthy: good errors, docs, tutorial, and a final acceptance suite.

**Deliverables:**

- **Diagnostics quality pass:** every `E####` code has a documentation entry in `docs/internals/diagnostics.md`; error messages reviewed for helpfulness; `chumsky`'s recovery tuned so common typos don't cascade.
- **Tutorial** (`docs/tutorial.md`): "Build a small HTTP client in `.kowari`" — uses `wasm` to load a WASI-compiled HTTP component, or (fallback) a minimal `std::io`-based HTTP client.
- **Language reference** (`docs/reference.md`): one section per grammar production and every stdlib item, generated where possible.
- **`--verbose`, `--emit=llvm-ir`, `--emit=asm`, `--emit=obj`, `--emit=metadata`** flags on the driver for debugging.
- **DWARF completeness:** variable types, inlined frames (at least the basics), so debugger output is *usable*.
- **Examples** all build and run in CI: `hello_world`, `file_io`, `wasm_add`, `concurrent_counter`.
- **Benchmark snapshots** recorded; a `PERFORMANCE.md` records baseline numbers.
- **Release process documented** in `docs/internals/release.md`: version bump, changelog, `SPEC.md` tag, tag in git, build artifacts.
- **`SPEC.md` promoted to `v1.0`** with the version line updated.

**DoD:** All §12 criteria (below) are met.

**Estimate:** ~2–3 weeks.

---

### 5.1 Grammar coverage by milestone

Not every grammar construct needs to work at every milestone. To avoid surprises, the parser's *required* coverage at each milestone:

| Construct | M0 | M1a | M1b | M2 | M3 | M4 | M5 |
| :--- | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| `fn`, `let`, `let mut` | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `if`/`else`, `while`, `loop`, `for` (range) | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Primitives, arithmetic, comparisons | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `struct`, `enum`, `match`, `if let`, `let else` | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `trait`, `impl`, generics (monomorphized) | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `resource`, `deinit`, `defer`, `errdefer` | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `unsafe`, raw pointers, `extern "C"` | — | ✓ (parsed, minimal use) | ✓ | ✓ | ✓ | ✓ | ✓ |
| `use` (single file) | — | — | ✓ | ✓ | ✓ | ✓ | ✓ |
| `use` (multi-file) | — | — | — | — | — | ✓ | ✓ |
| `extern "wasm"` | — | — | ✓ | ✓ | ✓ | ✓ | ✓ |
| Attributes (`#[…]`, incl. `// SAFETY:`-adjacent lints) | — | — | — | partial | ✓ | ✓ | ✓ |

Legend: `✓` = fully works; `partial` = parses, minimal typecheck; `—` = not required.

---

## 6. Testing & Quality Strategy

### 6.1 The three test layers (per `SPEC.md` §14)

1. **Golden harness (compiler)** — `tests/compile_pass` and `tests/compile_fail`; runs as `cargo test`. This is the *primary* correctness gate for every milestone.
2. **Fuzzing (lexer/parser)** — `tests/fuzz/*.rs` (cargo-fuzz); nightly CI; must not panic on any input.
3. **Language-level `#[test]`** — v2. Until then, stdlib modules include `#[test]`-shaped functions (with a comment placeholder) that a small Rust shim invokes; this scaffolding migrates to real `#[test]` in v2.

### 6.2 Golden test conventions

- Each `.kowari` file may include trailing expectation comments:
  ```kowari
  // EXPECT_STDOUT: 1
  // EXPECT_EXIT: 0
  // EXPECT_STDERR: (empty)
  ```
- `compile_fail` files use:
  ```kowari
  // ERROR E0142
  ```
  (A single file may declare multiple `// ERROR` lines; the harness asserts all and only those.)
- **Diagnostic codes are the contract**, not messages. Messages may evolve.
- **Determinism:** golden tests never depend on timing, thread scheduling, or filesystem layout. Where a test needs files, they are `tests/fixtures/` and copied to a temp dir by the harness.
- **Naming:** `snake_case.kowari`, prefixed by a coarse domain (`affine_`, `result_`, `trait_`, `wasm_`, `thread_`, `ffi_`, `match_`), so a `cargo test -p kowari-test-harness -- affine` runs just that domain.

### 6.3 CI

| Check | On PR | Nightly | On Release |
| :--- | :-: | :-: | :-: |
| `cargo fmt --check` | ✓ | ✓ | ✓ |
| `cargo clippy -D warnings` | ✓ | ✓ | ✓ |
| `cargo test` (unit + golden) | ✓ | ✓ | ✓ |
| `cargo test --release` (golden, release mode) | — | ✓ | ✓ |
| `cargo fuzz` (short run, each target) | — | ✓ | ✓ |
| `cargo bench` (record numbers) | — | ✓ | ✓ |
| Build+run `examples/*` | ✓ | ✓ | ✓ |
| Miri on `kowari-wasm-host` (unsafe shim) | — | ✓ | — |
| ThreadSanitizer on `kowari-wasm-host` + runtime shims | — | ✓ | — |

**Platforms in CI:** Linux x86-64 (mandatory), Linux aarch64 (mandatory from M2), macOS aarch64 (mandatory from M4), Windows x86-64 (best-effort until M5; must be green by v1.0). LLVM version pinned in CI.

### 6.4 Quality gates per milestone

- **A milestone does not ship** until its golden tests are in `main` and green.
- **A PR may not merge** if it *reduces* coverage in any domain (tracked by a small script counting `compile_pass`/`compile_fail` per domain).
- **`unsafe` in the Rust compiler** (not the language) must have a `// SAFETY:` comment (same convention as the language, enforced by a clippy-style lint or a `rg` check in CI).

---

## 7. Diagnostics & Error Reporting

Diagnostics are how the compiler speaks to the user; they deserve their own strategy.

- **Codes:** `E####` for errors, `W####` for warnings. Allocated once in `kowari-diagnostics/src/codes.rs`, documented in `docs/internals/diagnostics.md`.
- **Every code has a stable slug** (e.g., `E0142 = missing_deinit`). Tests assert on the slug, not the number, so renumbering is possible but discouraged.
- **Message style:** "what went wrong" + "where" + "how to fix" (three-part). Primary span first; secondary spans where helpful (e.g., "help: this local was declared here").
- **Suggestions** (`help:` and `note:`) are encouraged. Applying a fix (machine-applicable suggestions) is a v2 tooling concern.
- **`--error-format=json`** on the driver from M1a so future tooling (LSP, IDE) can consume structured diagnostics.
- **No panics during error recovery.** The compiler must always produce *some* diagnostic for bad input; a compiler panic on user input is a bug.

---

## 8. Stdlib Strategy

The stdlib is written **in the language itself** wherever possible, with thin Rust-side shims for what the language can't yet express (e.g., the `GeneralPurposeAllocator`'s `malloc`/`free` calls; `wasm`'s Wasmtime embedding).

### 8.1 Bootstrapping order

1. `alloc` (needs FFI to `malloc`/`free`)
2. `string` (needs `alloc`)
3. `collections::Vec` (needs `alloc`)
4. `fmt` (`format` intrinsic; `Format` trait; `println`)
5. `io` (needs FFI to `open`/`read`/`write`/`close`)
6. `collections::HashMap`, `BTreeMap`
7. `sync` (needs atomics + futex/`pthread_mutex` FFI initially)
8. `thread` (needs `pthread_create`-style FFI)
9. `wasm` (needs `kowari-wasm-host`)
10. `ffi` (CStr, common C types)

### 8.2 Stdlib conventions

- Every `pub` item is documented (see §8.3).
- Every module is `resource`-aware: resource types carry their allocator; `deinit` consumes `self`.
- No global allocator, ever. `Vec::new(alloc)` / `Vec::with_capacity(alloc, n)`; no `Vec::new()`.
- `unsafe` in stdlib is documented with `// SAFETY:` and validated in CI (deny-by-default for stdlib).
- **Stability:** pre-`v1.0`, stdlib APIs may break freely; `v1.0` freezes the *documented* surface (per §11 below).

### 8.3 Documentation

Stdlib docs are extracted from source doc-comments by a small tool (initially `kowaric --doc`, later a dedicated doc generator). Format: markdown-in-comments:

```kowari
/// Returns the element at `index`, or `None` if out of bounds.
/// 
/// # Panics
/// Never — this method is bounds-checked.
fn get(&self, index: usize) -> Option<V> { … }
```

**`# Panics`, `# Errors`, `# Safety`** sections are conventions, not enforced in v1.

---

## 9. Dependencies

Rust crates used by the compiler. Versions are chosen during M0; this table fixes the *choice* of crate, not the version.

### 9.1 Required in v1

| Purpose | Crate | Rationale |
| :--- | :--- | :--- |
| Lexing | `logos` | Fast, derive-based, per SPEC §Appendix. |
| Parsing | `chumsky` | Combinators + strong error recovery; §Appendix recommends. |
| LLVM bindings | **`inkwell`** | Safer than raw `llvm-sys`; wrap it, don't expose it. |
| Diagnostic rendering | `codespan-reporting` | Battle-tested span rendering (or write our own — decide in M0). |
| CLI | `clap` | Standard, well-documented. |
| Compiler-internal error handling | `thiserror` (library errors) + `anyhow` (driver) | Idiomatic Rust. |
| WASM hosting | **`wasmtime`** | Per SPEC §12.2; recommended Rust embedding runtime. |
| WASM component (v2 readiness) | `wasmtime` `component` feature flag | Off in v1, on in v2. |
| Metadata serialization | **`bincode`** | Simple, versioned by hand if needed. Evaluate `rkyv` in M4 if `bincode` bottlenecks. |
| Fuzzing | `cargo-fuzz` (libFuzzer backend) | Standard. |
| Benchmarking | `criterion` | Standard for Rust. |
| Temporary directories (tests) | `tempfile` | Standard. |

### 9.2 Explicitly avoided

- **No `syn`/`quote`/`proc-macro2` for the language's parser.** Those are for Rust macros; our parser is `chumsky`. (Confusing them is a common trap.)
- **No `rustc` internals.** We do not embed or reuse rustc's IR, typechecker, or codegen; we only follow its *design*.
- **No `llvm-sys` in the driver.** Only `kowari-codegen-llvm` may depend on the LLVM crate (and only via `inkwell`).
- **No async runtime in the compiler** unless benchmarking shows a need. The driver is single-threaded initially.

---

## 10. Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
| :--- | :--- | :--- | :--- |
| `inkwell`/`llvm-sys` version drift (each LLVM release breaks bindings) | High | High | Pin LLVM version in `rust-toolchain.toml` + CI; wrap LLVM calls behind a narrow `kowari-codegen-llvm::backend` trait so a swap to `llvm-sys` (or a different LLVM major) is a localized change. |
| Affine resource analysis edge cases (loops, early exits, `break`/`continue`, `match` moves) | High | Medium | Design the analysis on paper first (`docs/internals/affine-check.md`); test exhaustively in `compile_fail/`; when in doubt, reject conservatively and document. |
| DWARF is fiddly and under-documented | Medium | High | Start minimal (source lines only) in M1a; harden in M5; accept that *perfect* debug info is a v2 polish item. |
| WASM hosting overhead in hot paths | Medium | Medium | Document expected granularity (`SPEC.md` §12.2) — never claim "WASM at native speed." Provide a native C-ABI fallback path. |
| Slow compiles from monomorphization | Medium | Medium | Track compile times per example in benchmarks. Mitigate by keeping stdlib generics small, considering `dyn` in stdlib where perf allows. |
| Compiler complexity explosion (typeck) | High | Medium | Strict crate boundaries (§4.2); no back-edges; treat `kowari-typeck` as the only place the type system lives. |
| "Learning project vs production" scope creep | High | Medium | The **non-goals in `SPEC.md` §2 are the contract.** Any addition requires a corresponding SPEC change. |
| Golden test suite rots | Medium | Medium | Every PR must add or preserve coverage per §6.4; diagnostic codes as test assertions. |
| Contributing friction (LLVM install) | Low | High | Documented `scripts/build-llvm.sh`; pre-built LLVM images in CI; a `kowaric --version` check that tells users exactly which LLVM to install. |
| No package manager means no ecosystem (v1) | Medium | Medium | Accepted for v1. Design `kowari-metadata` (§4.2, §5.4) so a package manager is an additive change in v2. |

Risks reviewed at the start of every milestone; updates go in the milestone PR, not in this document.

---

## 11. Documentation & Stability

### 11.1 Docs to write (and *when*)

| Doc | When | Audience |
| :--- | :--- | :--- |
| `README.md` (orientation) | M0 | New contributors |
| `docs/internals/architecture.md` | M1a | Contributors |
| `docs/internals/affine-check.md` (the analysis, in detail) | M1a | Contributors |
| `docs/internals/data-race-safety.md` (formal argument) | M3 | Contributors |
| `docs/internals/diagnostics.md` (all `E`/`W` codes) | Continuously from M0 | Users + contributors |
| `docs/reference.md` (language + stdlib) | M5 (with extraction from source) | Users |
| `docs/tutorial.md` | M5 | Users |
| `docs/internals/release.md` | M5 | Maintainers |
| `CHANGELOG.md` | Every release | Everyone |

### 11.2 Stability policy

- **Pre-`v1.0`:** anything may break. Semver applies to the *compiler binary* only (so a pinned `kowaric 0.3.1` keeps working). No source-compatibility promises.
- **`v1.0`:** language semantics and the *documented* stdlib surface are frozen for `v1.x`. A `v2.0` may break.
- **Deprecation:** `v1.0` onward, a removal requires at least one minor version of `#[deprecated]`-style warning first (the attribute exists in v1 already, used internally; user-facing use is a M5 polish item).

---

## 12. Definition of Done for v1.0

`v1.0` ships when **all** of the following hold. Each is a literal checklist item; none is negotiable.

### 12.1 Language features

- [ ] All §3.1 features work as specified in `SPEC.md`.
- [ ] Every §4.4 `unsafe`-required operation is rejected outside `unsafe` and works inside it.
- [ ] Affine check rejects every case in `tests/compile_fail/affine_*` and accepts every case in `tests/compile_pass/affine_*`.
- [ ] Reference rules (§4.2) are enforced: refs in aggregates, returned refs, and refs escaping closures are compile errors.
- [ ] `?` runs `defer` and `errdefer` correctly on early return; panic aborts without running them.

### 12.2 End-to-end demos

- [ ] `examples/hello_world` compiles and runs.
- [ ] `examples/file_io` compiles and runs (reads and writes a file).
- [ ] `examples/wasm_add` compiles and runs (hosts a `.wasm` guest, prints `3.0`).
- [ ] `examples/concurrent_counter` compiles and runs (prints correct total; no data races under TSan).

### 12.3 Quality

- [ ] `tests/compile_pass/` and `tests/compile_fail/` pass on Linux x86-64, Linux aarch64, macOS aarch64, Windows x86-64.
- [ ] `cargo fuzz` runs nightly green on lexer and parser.
- [ ] `cargo bench` produces a baseline in `PERFORMANCE.md`.
- [ ] No compiler panics on any test input (fuzzing + golden suite).

### 12.4 Documentation

- [ ] `docs/tutorial.md` takes a reader from zero to a working `.kowari` program without external help.
- [ ] `docs/reference.md` covers every grammar production and every public stdlib item.
- [ ] `docs/internals/diagnostics.md` documents every diagnostic code.
- [ ] `SPEC.md` is tagged `v1.0` and matches the shipped compiler.

### 12.5 Tooling

- [ ] Debugger can set a breakpoint, print locals, and step source lines in LLDB and GDB.
- [ ] `kowaric --emit=llvm-ir` produces valid LLVM IR for every test.
- [ ] Multi-module projects (M4) build correctly; metadata round-trips through disk.
- [ ] CI green on all platforms.
- [ ] Release process documented and one dry run performed.

### 12.6 Rejection criteria (what would *not* let v1.0 ship)

- Any affine-check unsoundness (accepting a leaking/double-freeing program).
- Any reference rule unsoundness (accepting a dangling-reference program).
- Any safety invariant (§4.5) violated by safe code accepted by the compiler, *unless* the violation is documented as `unsafe` — in which case it is not a bug, but a documentation task.
- A demo in §12.2 that doesn't run.
- A platform in §12.3 where the golden suite doesn't pass.

---

## 13. Future Plans (v2+, no dates, no ordering)

These are **planned directions** — not committed milestones. Each will be specified in a future planning revision, with its own DoD, when it becomes active.

### 13.1 Version 2 candidates

- **`From`-conversion for `?`.** Add `From` trait + coherence rules; `?` gains `into()` semantics. Removes `.map_err(Enum::Variant)?` verbosity.
- **`#[borrows(self)]` borrowed returns** (`SPEC.md` §15.11 path 1). Local dataflow pass; enables `fn get(&self) -> Option<&V>`.
- **WIT / Component Model.** Add `wasmtime` component support; generate bindings from `.wit` files. The interop layer's natural upgrade from raw core-WASM.
- **Declarative macros** (`macro_rules!`-style). Token-tree matching; replaces the `format` intrinsic once mature.
- **Language-level `#[test]` framework.** Attribute + a `kowaric test` runner; replaces the golden-harness shim for user code.
- **`RwLock`, `RefCell`** — only if v1 signals (§15.11, §15.5 in `SPEC.md`) demand them.
- **WASM *output*** (`wasm32`/`wasm64` target). Compile the language to WASM for browser or edge; independent of hosting.
- **Cranelift debug backend.** Fast-compile backend for iteration; opt-in via `--backend=cranelift`.
- **Formatter** (`kowarifmt`) and **linter** (`kowarilint`), matching Rust's `rustfmt`/`clippy` ergonomics. *(Pademelon is a candidate name for one of these tools — see `SPEC.md` Appendix B.)*

### 13.2 Version 3+ candidates

- **Gradual borrow checking** (`SPEC.md` §15.11 path 2): `#[borrow_checked]` on individual functions.
- **Package manager / registry** (`cargo`-equivalent). Requires `kowari-metadata` maturity from v1. *(Pademelon is a candidate name — see `SPEC.md` Appendix B.)*
- **Incremental compilation.** Requires `kowari-metadata` + driver support for per-module caching.
- **LSP server.** Structured diagnostics (`--error-format=json`) exist from v1; LSP is a layer on top.
- **Doc generator.** Replaces `kowaric --doc`.
- **Async/await** or **M:N green threads** (one, not both; whichever the ecosystem needs). Requires substantial runtime work; do not commit before v2's stabilization.
- **Self-hosting** (compiler written in the language). Compiler must be expressive enough; no design changes anticipated, but a large engineering effort.
- **Full borrow checker** (`SPEC.md` §15.11 path 3). Only if the language's market fit clearly requires it.

### 13.3 Explicitly *not* on the roadmap

- A general-purpose GC mode.
- Exceptions or `catch`/`try`/`finally`.
- User-defined operator overloading or function overloading (this is a *design decision*, not a deferral).
- Comptime / staged execution (rejected in `SPEC.md` §2).
- Reflection (rejected in `SPEC.md` §2).

---

## 14. Onboarding (for future contributors)

If you are new to the project and want to contribute:

1. Read `SPEC.md` (§3–§6 especially) and this document (§3, §4, §6).
2. Skim `docs/internals/architecture.md` (created in M1a).
3. Build the compiler: `scripts/build-llvm.sh && cargo build`.
4. Run the tests: `cargo test`.
5. Pick a diagnostic-code entry in `docs/internals/diagnostics.md` that lacks a golden test, write one, and open a PR. This is the easiest first contribution — it exercises the full toolchain without requiring compiler-internal knowledge.
6. Do **not** open a PR that changes `SPEC.md` without discussing it in an issue first; the spec is the contract.

Contribution rules (PR template):

- [ ] Tests added (or coverage preserved) per §6.4.
- [ ] `cargo fmt` and `cargo clippy -D warnings` clean.
- [ ] If a new diagnostic code was added: it has an entry in `docs/internals/diagnostics.md` and a `compile_fail/` golden test.
- [ ] If a SPEC decision is affected: `SPEC.md` updated in the same PR.
- [ ] If a milestone DoD is affected: noted in the PR description.

---

## 15. Glossary (quick reference)

| Term | Definition |
| :--- | :--- |
| **Kowari** | The language. Named after the small carnivorous marsupial (*Dasyuroides byrnei*). |
| **`kowaric`** | The compiler binary. |
| **`.kowari`** | Source file extension. |
| **Affine resource type** | A type whose values must be consumed exactly once; leaks and double-consumption are compile errors. |
| **Consumption** | By-value use of a resource value (pass, move, `deinit`). |
| **`defer` / `errdefer`** | Scope-exit / error-path-exit cleanup, LIFO. |
| **Function-scoped reference** | A `&T` or `&mut T` valid only within the enclosing function; cannot be stored, returned, or escape. |
| **Golden test** | `.kowari` file with inline `// EXPECT_*` / `// ERROR` assertions. |
| **Diagnostic code** | Stable identifier (e.g. `E0142`) attached to every diagnostic; the test contract. |
| **Vertical slice** | A milestone that produces a runnable program using every relevant language feature, rather than building the compiler bottom-up. |
| **DoD** | Definition of Done; the acceptance criteria for a milestone (§5, §12). |
| **`// SAFETY:`** | Linter-parsed comment preceding every `unsafe` block/fn; no semantic effect. |
| **Pademelon** | Reserved name for a future tool (TBD). |

---

*End of planning document (v0). Update this file — and only this file — when scope, milestones, or future plans change. Design decisions change `SPEC.md`.*