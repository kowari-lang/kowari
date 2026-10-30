# Kowari Language Specification

Version: 0.1.0-draft
Status: Draft

This document specifies the Kowari programming language, version 0.1. It is the normative reference for the lexer,
parser, and semantic analyzer. Features marked v1 are present in the grammar but rejected by the compiler with
diagnostic E-V1-001; they are included here so that source written today remains syntactically stable as the language
grows.

> **Editorial note.** This copy normalizes Markdown structure, headings, lists, tables, and code fences. It does not
> resolve or change normative language rules. See the separate review report for contradictions, grammar gaps, and
> decisions still required.

## Contents

- [1 Introduction](#1-introduction)
- [2 Non-goals](#2-non-goals)
- [3 Lexical Structure](#3-lexical-structure)
- [4 Types](#4-types)
- [5 Expressions](#5-expressions)
- [6 Declarations](#6-declarations)
- [7 Statements](#7-statements)
- [8 Patterns](#8-patterns)
- [9 Modules and visibility](#9-modules-and-visibility)
- [10 Integer Literal Typing](#10-integer-literal-typing)
- [11 Strings and Formatting](#11-strings-and-formatting)
- [12 v1 Constructs](#12-v1-constructs-parsed-rejected-in-v0)
- [13 FFI in v0](#13-ffi-in-v0)
- [14 Open Questions](#14-open-questions)
- [Appendix A — EBNF Grammar](#appendix-a--ebnf-grammar)
- [Appendix B — Diagnostic Registry](#appendix-b--diagnostic-registry)

## Changes from v0

- Formal grammar. Full EBNF in Appendix A. Previously: prose only.

- Formal diagnostic registry. Namespaced codes (E-LEX-001 etc.) in Appendix B. Previously: example codes only.

- Name resolution. Defined in §9: modules, visibility, shadowing, glob imports.

- Closure semantics. Defined in §5.9: capture modes, move, escaping.

- Trait rules. Defined in §6.6: default methods, supertraits, coherence, associated types.

- Pattern lowering. §8.5 defines the decision-tree algorithm.

- Integer typing. §10.1 defines unsuffixed-literal inference and context.

- Format string syntax. §11 formalizes format!-style grammar and the Format trait.

- Non-goals split. §2 now distinguishes "out of scope for v0" (with target version) from "out of scope permanently".

- Versioning. Every section carries an implicit version; constructs v1-or-later are marked inline.

- Rationale callouts. Non-normative > **Rationale.** blocks accompany rules where the why is worth recording.

## 1 Introduction

### 1.1 What Kowari is

Kowari is a statically typed, compiled systems language. It targets native code via LLVM with an optional Cranelift
debug backend. Its core identity is a single safety story — affine resource types with compile-time leak checking —
replacing what other systems languages split across a borrow checker, lifetime annotations, and manual free discipline.

Kowari is not trying to be a general-purpose application language. It is a language for programs where memory layout,
concurrency safety, and FFI correctness are load-bearing. That identity governs every non-goal in §2.

### 1.2 Conformance

The key words shall, shall not, should, and may in this document are to be interpreted as described in RFC 2119. An
implementation that rejects a conforming program or accepts a non-conforming one is non-conforming.

## 2 Non-goals

### 2.1 Out of scope for v0 (target version)

| Feature                                                                    | Target | Notes                                     |
|----------------------------------------------------------------------------|--------|-------------------------------------------|
| Affine resource types (`deinit`, `consume self`, leak-is-error)            | v1     | Grammar reserved; E-V1-001 today          |
| References (`&T`, `&mut T`), function-scoped                               | v1     | Migration to borrow checker in v2 per §15 |
| Structured concurrency (`spawn`, `sync`, `async`, `Mutex`)                 | v1     | OS threads; no async runtime              |
| `unsafe`, FFI (`extern "C"`)                                               | v1     | No raw pointer manipulation in v0         |
| Allocators (user-provided)                                                 | v1     | v0 uses a single global allocator         |
| `comptime fn` and `comptime` items                                         | v1     | `comptime` reserved                       |
| `#[derive(...)]`                                                           | v1     | Attribute parsed; no expansion            |
| `dyn Trait`                                                                | v1     | `dyn` reserved                            |
| `From` conversion integrated into `?`                                      | v1     | v0 `?` requires exact error type          |
| Tail-call guarantee                                                        | v1     | Lowering decision deferred to v1          |
| `@capabilities` module annotation                                          | v1     | Reserved                                  |
| Formatter, documentation generator, LSP, user test runner, package manager | v1+    | —                                         |

### 2.2 Out of scope permanently

These will not be added, at any version, without a new specification document:

- Garbage collection. No tracing GC, no reference counting. Memory is managed by affine ownership.

- Exceptions and unwinding. panic = abort. No catch. No finally.

- Default laziness. Evaluation is eager.

- Operator overloading. Beyond the fixed primitive-operator set in §5.3.

- Custom operators. Only the tokens in §3.2 are operators.

- Dependent types. No proof obligations in the type system.

- Runtime reflection. No type introspection at runtime. comptime in v1 is compile-time only.

- Implicit conversions beyond integer-literal defaulting (§10.1) and v1 From-on-?.

## 3 Lexical Structure

### 3.1 Source encoding

Source files are UTF-8. A leading byte-order mark (U+FEFF) shall be ignored. Identifiers are ASCII by default; Unicode
identifiers are a v1-or-later design space (§ Appendix D).

### 3.2 Tokens

Kowari has the following token categories: keywords, identifiers, literals, operators/punctuation, delimiters, and
reserved tokens.

Keywords (v0 active): as, async, break, const, continue, defer, dyn, else, enum, extern, false, fn, for, if, impl, in,
let, loop, match, mod, move, mut, pub, return, self, Self, spawn, struct, sync, trait, true, type, union, unsafe, use,
where, while

Keywords (v1 active, currently parse-only — marked *): async, dyn, extern, spawn, sync, unsafe

Reserved tokens (no current use; documented in §3.5): ComptimeToken (comptime), ConstToken (const in modifier position —
currently active as a keyword), DynToken (dyn — reserved for v1 object safety), StaticToken (static)

> **Rationale.** Reserving syntax is cheap. Shipping a keyword you later rename is expensive. Every name in the reserved
> set has a defined purpose; nothing is reserved speculatively.

Operators and punctuation: + - * / % & | ^ ! = == != < <= > >= << >> && || ? . , ; : :: -> => @ # _ ( ) [ ] { }

### 3.3 Literals

Integer literals. Decimal (123), hexadecimal (0xFF), octal (0o777), binary (0b1010). An underscore may appear between
digits (1_000_000). An optional suffix selects the type: i8, i16, i32, i64, u8, u16, u32, u64, usize, isize. An
unsuffixed integer literal has an inferred type (§10.1). Integer literals with an invalid suffix shall be lexed as the
integer followed by the identifier (1u65 → integer 1, identifier u65), matching §10.1's rule that unsuffixed contexts
defer to inference.

Float literals. Decimal floating-point with mandatory fractional part or exponent (1.5, 1e10, 1.5e-10). Suffixes f32,
f64. No hexadecimal floats in v0.

Boolean literals. true, false.

Character literals. 'a', '\n', '\u{1F600}', '\''. Single-byte or Unicode scalar; type char.

String literals. "hello", "line\n", "quote: \"". UTF-8, with the same escape set as characters. Adjacent string literals
do not concatenate.

Byte string literals. b"bytes", type [u8; N]. Byte strings support only \xNN escapes.

Raw strings. r"...", r#"..."#, r##"..."##, … No escape processing.

### 3.4 Comments

- Line comments: // to end of line.

- Block comments: /* ... */. Nesting shall be supported.

- Doc comments: /// (outer, attaches to the following item) and //! (inner, attaches to the enclosing module).

> **Rationale.** Nesting block comments avoids the C-style footgun where commenting-out a block breaks on a comment
> inside it. The cost is zero.

### 3.5 Reserved tokens

The following tokens are reserved for future use. Their presence outside their target context produces E-LEX-013.
| Token | Reserved for | Target |
|---|---|---|
| `comptime` | Compile-time evaluation marker (`comptime fn`, `comptime { }`) | v1 |
| `static` | Module-scoped mutable state (if added) | v1+ |
| `dyn` | Trait objects (`dyn Trait`) | v1 |
| `const` in modifier position | Predecessor spelling for `comptime fn`; retained for tooling compatibility | v1 |

> **Rationale.** comptime and static have defined homes. dyn is reserved for the v1 trait-object feature. const in
> modifier position is currently accepted as an alias for comptime's future spelling, so source remains
> forward-compatible
> either way.

### 3.6 Lexical diagnostics

See Appendix B, category E-LEX.

### 3.7 Lexical diagnostic behavior

The token grammar was defined in Appendix A. §3 normatively fixes the diagnostic behavior of the lexer and the fine
points the grammar leaves open.

#### 3.7.1 Comments

Block comments nest. /* /* */ */ is one comment. The tokenizer shall count nesting depth and emit E-LEX-003 on
unterminated nesting at EOF.

Doc comments /// and //! attach to the next item or the enclosing module respectively. A doc comment not adjacent to a
declarable item produces E-LEX-012.

> **Rationale.** Nesting comments are the cheapest way to make "comment out this region" work reliably; no mainstream
> language has regretted supporting them.

#### 3.7.2 Token edge cases

| Situation                      | Behavior                | Diagnostic                 |
|--------------------------------|-------------------------|----------------------------|
| `1u65`                         | Lexes as `1` then `u65` | None — suffix rule in §3.3 |
| `0x` alone                     | Invalid                 | `E-LEX-008`                |
| `0xFF_FF_FF`                   | Valid                   | None                       |
| `"abc` at EOF                  | Unterminated            | `E-LEX-001`                |
| `'ab'`                         | Multi-character         | `E-LEX-007`                |
| `''`                           | Empty                   | `E-LEX-006`                |
| `'\q'`                         | Invalid escape          | `E-LEX-004`                |
| `b"abc"`                       | Byte string             | None                       |
| `r"abc"` / `r#"..."#`          | Raw string              | None                       |
| Non-ASCII in identifier        | Rejected in v0          | `E-LEX-011`                |
| Reserved token outside context | Rejected                | `E-LEX-013`                |

#### 3.7.3 Integer suffixes

The full suffix set is: i8, i16, i32, i64, isize, u8, u16, u32, u64, usize. A literal of the form DIGITS SUFFIX with
SUFFIX a valid suffix has that type. Any other trailing identifier is not consumed as a suffix — the lexer stops at the
first non-suffix character (which is why 1u65 is 1 followed by an identifier, and why the parser reports the actual
error, not the lexer).

> **Rationale.** Making the suffix set a fixed, small, closed list keeps 1u65 from becoming a hard lexer error. The
> error the user sees is E-INT-001 at the use site of the unsuffixed 1, which is more informative than a lexer tantrum.

#### 3.7.4 Character and string escapes

The escape set is identical for char and "…" string literals: \', \", \\, \n, \r, \t, \0, \xNN (byte), \u{…} (Unicode
scalar). b"…" byte strings accept only \\, \", \n, \r, \t, \0, \xNN. Any other escape is E-LEX-004.

#### 3.7.5 Lexical diagnostics (normative list)

| Code        | Trigger                                                |
|-------------|--------------------------------------------------------|
| `E-LEX-001` | unterminated string literal                            |
| `E-LEX-002` | unterminated character literal                         |
| `E-LEX-003` | unterminated block comment (nesting-aware)             |
| `E-LEX-004` | invalid escape sequence                                |
| `E-LEX-005` | invalid byte in source                                 |
| `E-LEX-006` | empty character literal ''                             |
| `E-LEX-007` | multi-character character literal                      |
| `E-LEX-008` | invalid integer literal                                |
| `E-LEX-009` | integer literal out of range for suffix                |
| `E-LEX-010` | invalid float literal                                  |
| `E-LEX-011` | non-ASCII identifier (Unicode identifiers: Appendix D) |
| `E-LEX-012` | doc comment not attached to an item                    |
| `E-LEX-013` | reserved token used outside target context             |

## 4 Types

Kowari's type system is static, nominal (except tuples and function types, which are structural), with no implicit
coercions except integer-literal defaulting (§10.1) and, in v1, From-on-?. All values have a single, inferred or
annotated type.

### 4.1 Primitive types

| Type                      |                Size | Range / representation                                  |
|---------------------------|--------------------:|---------------------------------------------------------|
| `bool`                    |              1 byte | `true` / `false`                                        |
| `char`                    |             4 bytes | Unicode scalar (`0x0`–`0x10FFFF`, excluding surrogates) |
| `i8`, `i16`, `i32`, `i64` | 1 / 2 / 4 / 8 bytes | Two’s complement                                        |
| `u8`, `u16`, `u32`, `u64` | 1 / 2 / 4 / 8 bytes | Unsigned                                                |
| `isize`, `usize`          |       Pointer width | Signed / unsigned                                       |
| `f32`, `f64`              |         4 / 8 bytes | IEEE 754 binary32 / binary64                            |
| `!`                       |             0 bytes | Never type (bottom)                                     |
| `()`                      |             0 bytes | Unit type                                               |

! is the type of return, break, continue, panic!, and any expression that does not produce a value. It is assignable to
every type (§4.6).

### 4.2 Compound types

- Tuple (T₁, T₂, …, Tₙ) — structural, fields named .0, .1, …; n ≥ 0; () is the empty tuple.

- Array [T; N] — fixed size, inline storage, N a usize constant.

- Slice [T] — unsized; usable only behind a reference in v0 (&[T]).

- Struct — nominal product type, fields named, layout per §4.7.

- Union — nominal, fields overlap, unsafe to access in v1; in v0 only usable via #[repr (C)] for FFI and rejected at
  construction sites otherwise.

- Enum — nominal sum type; each variant may carry a tuple or struct payload.

- Function pointer fn (A, B) -> C — pointer to a function with the given signature; non-capturing.

### 4.3 Generic types

Types may be parameterized by type parameters: Vec<T>, Result<T, E>, HashMap<K, V>. Generic arguments are invariant in
v0 — no variance annotations, no higher-kinded types. A generic parameter may carry trait bounds (§6.5).

### 4.4 Special types from the prelude

- Option<T> — defined in §5.4, prelude-only.

- Result<T, E> — defined in §5.5, prelude-only.

- String — owned UTF-8, §11.2.

- Box<T> — owning pointer; v1 (requires allocators to be user-pluggable; in v0 the prelude provides Box backed by the
  global allocator, but user code cannot implement Allocator).

### 4.5 Reference types (v0, scoped)

Kowari v0 supports shared (&T) and mutable (&mut T) references, with the following restrictions:

- Non-storable. A reference may not appear as a struct field, enum payload, array element, or tuple field that outlives
  its enclosing expression.

- Non-returnable. A function shall not return a reference whose referent lives in its caller or callee. fn f (x: &T) ->
  &T is rejected with E-REF-001.

- Function-scoped. A reference may not be captured by a closure that escapes the function that created it.

Formally, references are usable only as function parameters, local bindings within a single function body, and
temporaries. They shall be rejected at any position that would require them to outlive a single function activation.

Aliasing is not checked in v0. &mut T is advisory: it declares the referent may be mutated, but the compiler does not
prevent two &mut T to the same place, nor a &T overlapping a &mut T. E-REF-003 is reserved for v1.

Dereference syntax. *expr dereferences a reference; on a place expression it produces a place. Reading through *r where
r: &T moves out of the referent if T: Copy, and is a compile error (E-REF-002) otherwise — copy or borrow, never move
from a shared reference.

No auto-deref, no Deref trait in v0. Method calls shall not auto-dereference. s.pretty () on s: &Settings is rejected
with E-REF-004 ("method pretty not found on &Settings; dereference explicitly with (*s)"). Auto-deref is a v1 ergonomic.

Constructing references. &place and &mut place construct a reference to a place expression. &mut place requires place to
be a mutable place (binding declared mut, or a field of one).

Method receivers. A method's first parameter may be self, &self, or &mut self — all three are usable in v0. consume self
(§4.9) is v1.

Reference diagnostics:
| Code | Trigger |
|---|---|
| `E-REF-001` | function returns a reference |
| `E-REF-002` | move out of a shared reference |
| `E-REF-003` | reserved: aliasing violation (v1) |
| `E-REF-004` | auto-deref required but not permitted in v0 |
| `E-REF-005` | reference used in non-function-scoped position |

> **Rationale.** Function-scoped references eliminate every dangling-reference bug class by scoping, without
> implementing the borrow checker. Aliasing remains unchecked; that is what the borrow checker is for, and it lands in
> v1
> with the same surface syntax. When v1 ships, the function-scope restriction is relaxed, never tightened — code that
> compiled on v0 continues to compile.

### 4.6 Subtyping and coercion

Kowari's subtyping lattice is shallow by design:

- ! is a subtype of every type.

- References are covariant in the referent type: &'a T is a subtype of &'a U when T <: U. (Lifetimes are not first-class
  in v0; this is a statement about the rules that will be added in v1, and v0 accepts the same variance structurally.)

- No other subtyping. No u8 <: u16. No structural subtyping between structs.

Explicit coercions (as) are permitted only between primitive numeric types, from char to u32, from u8 to char, and from
a zero-sized unit struct to (). as is a widening-or-truncating numeric cast and never changes representation silently —
it is checked at the call site, not inferred.

### 4.7 Layout and repr

Without a #[repr (...)] attribute, a struct's field order and padding are unspecified — the compiler may reorder for
size. With #[repr (C)], fields are laid out in declaration order, each at its natural alignment, and the struct's size
is the next multiple of its alignment. #[repr (Rust)] is the default and equals "unspecified."

`#[repr (C)]` is required for any struct or union passed across an FFI boundary. #[repr (packed)] is not in v0 (it
interacts badly with references; will be specified in v1).

Layout of enums is unspecified without #[repr (C)] or #[repr (u8)] (etc.). #[repr (u8)] fixes the tag to a u8 and lays
out the payload after it. Zero-payload enums are guaranteed to have a tag and no payload.

Alignment is at least the natural alignment of every field. Empty structs have size 0 and alignment 1.

> **Rationale.** Rust's experience: default-layout reorder is a real memory win, and #[repr (C)] is a cheap opt-in. Do
> not promise a layout the compiler cannot always achieve.

### 4.8 Option narrowing

Option<T> is laid out using a niche: for Option<&T>, Option<Box<T>>, Option<NonNull<T>>, and any other type T where T
has an unused valid value (the "niche"), Option<T> is guaranteed to be the same size as T. Option<()> is 1 byte (not 2).
This is a guarantee, not an optimization — it is documented so users can rely on it for FFI and for exact-size structs.

### 4.9 consume self — reserved (v1)

The consume parameter modifier, which marks self (or any parameter) as consumed by the function, is parsed in v0 but
produces E-V1-001 at every use site. Its semantics (§4.1 in the v0 rationale) are unchanged and will activate in v1.

### 4.10 Type diagnostics

| Code         | Trigger                            |
|--------------|------------------------------------|
| `E-TYPE-001` | type mismatch                      |
| `E-TYPE-002` | cannot infer type                  |
| `E-TYPE-003` | unknown type name                  |
| `E-TYPE-004` | wrong number of generic arguments  |
| `E-TYPE-005` | missing ? where Option expected    |
| `E-TYPE-006` | recursive type without indirection |
| `E-TYPE-007` | invalid as cast                    |
| `E-TYPE-008` | ambiguous integer type at use site |

## 5 Expressions

Every expression has a value category: place, value, or type. Places are assignable, borrowable, and readable; values
are readable, movable, and (if Copy) copyable; type expressions appear in type position only. The categories exist so
that the reference rules of §4.5 can be stated precisely.

### 5.1 Operator precedence

From tightest to loosest:

| Level          | Operators                  | Notes                             |
|----------------|----------------------------|-----------------------------------|
| Postfix        | `.`, `()`, `[]`, `?`       | Postfix operators                 |
| Unary prefix   | `-`, `!`, `&`, `&mut`, `*` | Prefix operators                  |
| Multiplicative | `*`, `/`, `%`              |                                   |
| Additive       | `+`, `-`                   |                                   |
| Shift          | `<<`, `>>`                 |                                   |
| Comparison     | `<`, `<=`, `>`, `>=`       |                                   |
| Equality       | `==`, `!=`                 |                                   |
| Bitwise AND    | `&`                        |                                   |
| Bitwise XOR    | `^`                        |                                   |
| Bitwise OR     | \`                         | \`                                |
| Logical AND    | `&&`                       | Short-circuit                     |
| Logical OR     | \`                         | Short-circuit                     |
| Assignment     | `=`, `+=`, `-=`, …         | Right-associative; statement only |

? binds to the entire preceding expression (expr? is (expr)?), not just the last primary — this is why foo ()?.bar is
(foo ()?).bar.

### 5.2 Arithmetic

The operators `+`, `-`, `*`, `/`, and `%` on integers are checked in debug builds and wrap in release builds. Wrap
explicitly with `.wrapping_add`, `.checked_add`, `.saturating_add`, or `.overflowing_add` (prelude). Division by zero
panics in both modes. Float division by zero follows IEEE 754 (`±∞`, `NaN`).

Unary - on an unsigned integer is a compile error. Unary - on iN::MIN panics in debug, wraps in release.

### 5.3 Bitwise, comparison, logical

Bitwise operators require identical integer types. Comparison operators (< <= > >=) require Ord. Equality operators
(== !=) require PartialEq. All are non-overloadable in v0 — the trait bound is not user-implementable in v0 (that's a v1
feature with the rest of the prelude-implementation system).

&& and || short-circuit; the right operand is evaluated only if needed.

### 5.4 Option<T>

```kowari
enum Option<T> {
Some (T),
None,
}
```

Methods in the prelude: .is_some (), .is_none (), .unwrap (), .unwrap_or (default), .map (f), .and_then (f), .ok_or
(err). unwrap panics on None with panic!("called Option::unwrap on None").

T? is sugar for Option<T> in type position only.

### 5.5 Result<T, E>

```kowari
enum Result<T, E> {
Ok (T),
Err (E),
}
```

The ? operator: expr? evaluates expr: Result<T, E>. If Ok (v), evaluates to v. If Err (e), returns Err (e) from the
enclosing function. ? requires the enclosing function to return Result<_, E'> where E = E' in v0. In v1, E: Into<E'> via
From will be permitted.

? on Option is also supported: expr? on expr: Option<T> returns None from the enclosing function, which must return
Option<U>.

> **Rationale.** v0's exact-error-type ? is a v1-relaxable rule (it accepts less than v1 will; it never accepts more).
> This is the safe direction for a spec.

### 5.6 Calls

f (a, b, c) — function call. Argument evaluation is left-to-right, and function arguments are evaluated before the
callee's body begins. Method calls x.m (a, b) are sugar for m (x, a, b) (with self bound to x); x.m () on x: &T requires
explicit (*x).m () in v0 (no auto-deref, §4.5).

### 5.7 Closures

```kowari
|x| x + 1 // infer param and return types
|a: i32, b: i32| -> i32 { a + b } // explicit types
move |x| x + captured // capture by value
```

Capture mode. By default, a closure captures by reference, borrowing the smallest amount needed (shared, or mutable).
move before the pipe captures by value (moving the captured variable into the closure). This is Rust's rule.

Escaping. A non-move closure shall not escape the function that created it (it captures references, which are
function-scoped per §4.5). A move closure may escape — it owns its captures — but it cannot capture a &T because that
would smuggle a function-scoped reference past its scope (E-REF-005).

Traits. All closures implement Fn, FnMut, or FnOnce depending on how they use their captures. This distinction is on the
type system side; the surface syntax is the same.

Diagnostics:
| Code | Trigger |
|---|---|
| `E-CLS-001` | non-move closure escapes its function |
| `E-CLS-002` | move closure captures a function-scoped reference |

### 5.8 Method syntax and UFCS

x.m (a) is sugar for T::m (x, a) where T is the type of x. There is no universal function-call syntax; T::m (x, a) is
written to disambiguate or to call a method as a free function. Method resolution shall consider inherent impls on the
receiver's type, then trait impls in scope, in that order; a candidate that is ambiguous produces E-NAME-004.

### 5.9 Blocks and statements

A block is { stmts…; tail }. The trailing expression (if any) is the block's value; a block with no trailing expression
has type (). Statements are terminated by ; or are items. A block's value is not implicitly converted to () — a block
that ends in an expression of type T and is used in a () context produces E-TYPE-001.

defer { … } schedules the block to run when the enclosing scope exits, in reverse order of registration. On panic, defer
blocks do not run (panic = abort).

### 5.10 if, match, loop, while, for

All of these are expressions. if cond { a } else { b } has type T where both branches have type T; an if without else
has type ().

match is an expression; arms share the match's type. Exhaustiveness is checked (§8.4).

loop { } has type ! unless it contains a break value, in which case it has the type of the values.

### 5.11 panic!

panic!(msg: fmt, args…) aborts the process (no unwinding; defer does not run). It has type !. The format! argument
grammar is in §11. Panic messages are implementation-defined in v0 but shall include the file and line of the panic!.

### 5.12 Expression diagnostics

| Code        | Trigger                                                 |
|-------------|---------------------------------------------------------|
| `E-EXP-001` | cannot use ? outside function returning Result / Option |
| `E-EXP-002` | if branches have incompatible types                     |
| `E-EXP-003` | match arms have incompatible types                      |
| `E-EXP-004` | block's tail type does not match context                |
| `E-EXP-005` | division by zero in constant expression                 |
| `E-EXP-006` | overflow in constant expression                         |
| `E-CLS-001` | non-move closure escapes                                |
| `E-CLS-002` | move closure captures a function-scoped reference       |

## 6 Declarations

### 6.1 Functions

A function is declared with fn, an identifier, a parameter list, and a body. Functions are items, so they have a name
and are visible via §9's visibility rules.

```kowari
fn add (a: i32, b: i32) -> i32 { a + b } // block body
fn log (msg: String) { eprintln!(msg) } // implicit `-> ()`
fn identity<T>(x: T) -> T { x } // generic
```

Return type. If omitted, the return type is (). The body's trailing expression shall have a type equal to the return
type (§4.6 subtyping applies — ! is a valid body of any return type).

Parameters. Parameter names are patterns; irrefutable patterns are required. A parameter whose pattern is refutable
produces E-PAT-003.

Recursion. Functions may call themselves and each other. The compiler shall reject cycles of const fn evaluation;
function calls (including mutual recursion) are not constrained.

Tail-call guarantee. Not in v0. Lowering is an LLVM decision in v0; musttail guarantee is a v1 spec addition.

### 6.2 Methods and receivers

A method is a function declared in an impl block. Its first parameter is one of:
| Receiver | Meaning |
|---|---|
| `self` | Consumes the value; caller moves it |
| `&self` | Borrows immutably; caller keeps ownership |
| `&mut self` | Borrows mutably; caller keeps ownership and cannot access it during the call |
| `consume self` | v1; parsed, then rejected with `E-V1-001` |
| No receiver | Associated function; called as `Type::f(...)` |

```kowari
impl Vec<i32> {
fn new () -> Self { … } // associated, no receiver
fn len (&self) -> usize { … } // shared borrow
fn push (&mut self, x: i32) { … } // mutable borrow
fn into_parts (self) -> (…) { … } // consumes
}
```

Calling. v.push (1) requires v to be a mutable place. v.len () takes &v; if v is already borrowed mutably for the
duration of an outer expression, v0 does not check this (§4.5 — aliasing is advisory). Explicit dereference is required:

```kowari
let v = get_vec (); (*v).len ()                             // if v: &Vec<i32>
// v.len ()                             // E-REF-004
```

Method resolution order. For a call x.m (args):

- Inherent impls on T = typeof (x), considering T, &T, &mut T in that order.

- If T is a reference &U or &mut U, the same, on U.

- Trait impls in scope for T (or &U/&mut U).

- If a match is found in an earlier step, no later step is consulted. Ambiguity within a step produces E-NAME-004.

Receiver diagnostics:
| Code | Trigger |
|---|---|
| `E-METH-001` | method called on immutable place with &mut self receiver |
| `E-METH-002` | method called with self receiver on a place already borrowed |
| `E-METH-003` | receiver type does not match the method's expected type |

### 6.3 Structs

A struct declares a nominal product type. Three syntactic forms:

```kowari
struct Point { x: i32, y: i32 } // named fields
struct Meters (f64); // tuple struct; field `.0`
struct Unit; // unit struct, size 0
```

Construction. Named-field structs: Point { x: 0, y: 0 }. Field shorthand: Point { x, y } where x and y are bindings in
scope. Tuple structs: Meters (5.0). Unit structs: Unit (no parentheses).

Missing fields produce E-STRCT-001. Extra fields produce E-STRCT-002. Duplicate field initializers produce E-STRCT-003.
Struct update (..other) is not in v0; it is a v1 ergonomic.

Mutability. Fields share the mutability of their binding. let p = Point{…} gives immutable fields; let mut p = … gives
mutable fields. Field-level mut on the declaration is not supported (§14.3 of the design rationale: mutability belongs
to the binding, not the type).

Visibility. Each field may carry its own pub (§9.2). A private field makes the struct non-constructible outside its
module — construction, pattern matching, and field access outside the module are rejected with E-MOD-001.

Derived traits (v1). #[derive (Eq, Ord, Hash, Format)] is parsed in v0 but does not expand; use of the attribute in v0
produces E-V1-001. Until v1, derive must be implemented by hand.

### 6.4 Enums

An enum declares a nominal sum type. Each variant is a name with optional payload:

```kowari
enum Shape {
Empty, // no payload
Circle (f64), // tuple payload
Rect { w: f64, h: f64 }, // struct payload
}
```

Construction. Shape::Empty, Shape::Circle (1.0), Shape::Rect { w: 1.0, h: 2.0 }. A variant with no payload is a value,
not a function; Shape::Empty is the value.

Pattern matching. Any variant shape is matchable (§8). Exhaustiveness is checked (§8.4).

Discriminants. May not be assigned explicitly in v0 (Shape::Circle = 4 is E-V1-001). Layout of the tag is
implementation-defined unless #[repr (...)] is applied (§4.7).

Duplicate variant names produce E-ENUM-001. Recursive enum with no indirection produces E-TYPE-006:

```kowari
enum Bad { Cons (Bad) } // E-TYPE-006
enum Ok { Cons (Box<Bad>) } // Box is v1 (indirection)
```

> **Rationale.** Enum tags, like struct layout, are unspecified without #[repr]. Rust's experience is that guaranteeing
> layout kills optimizations (e.g., niche packing for Option<&T>); the spec should be silent unless asked.

### 6.5 Unions

A union declares a nominal type whose fields overlap in memory:

```kowari
#[repr (C)]
union Bytes {
b: [u8; 4],
w: u32,
}
```

Construction (Bytes { w: 5 }) is a v1 feature: it requires unsafe context because reading a union field whose written
variant doesn't match is undefined. In v0, unions may be declared (for FFI planning) but not constructed — every
construction site produces E-V1-001. Unions are usable in v0 only through extern "C" bindings, which are also v1.

> **Rationale.** Unions are FFI infrastructure. FFI is v1. Shipping construction without unsafe would be unsound;
> shipping it with unsafe requires unsafe, which is v1. So: v0 parses, does not permit use.

### 6.6 Traits

A trait declares a set of items that implementors must provide:

```kowari
trait Shape {
fn area (&self) -> f64; // required
fn perimeter (&self) -> f64 { // default method
0.0
}
type Output; // associated type
const MIN: f64; // associated const
}
```

Default methods. A method with a body in the trait declaration is a default: implementors may override it, or omit it
and inherit. A default method may call required methods and other defaults via self.

Supertraits. trait Drawable: Shape requires every Drawable implementor to also implement Shape. Supertrait bounds are
checked at impl time; calling a Shape method on a T: Drawable is allowed.

Associated types. type Output; in a trait is a placeholder the impl fills in. Referenced as <T as Trait>::Output or
(when unambiguous) T::Output.

Associated consts. const MIN: f64; in a trait; each impl supplies a value.

Trait bounds. A generic parameter may be bounded: fn print<T: Format>(x: T), or with where:

```kowari
fn pretty<T>(x: T) -> String
where T: Format + Clone
{ format!("{}", x) }
```

Coherence (orphan rule). An impl Trait for Type shall be permitted only if Trait or Type is local to the current crate.
No blanket impls of foreign traits for foreign types. Violations produce E-TRAIT-002.

> **Rationale.** Coherence without orphan rules leads to overlapping impls that the compiler cannot disambiguate (the
> old Haskell "orphan instances" problem, and the reason Rust adopted coherence). Coherence is a spec rule, not a
> checker
> heuristic — accept the ergonomic cost to get soundness.

Object safety. Not in v0 (dyn Trait is v1). A trait shall not be used as dyn Trait in v0; dyn is parsed and produces
E-V1-001. Object safety rules for v1 are in Appendix E.

Required item missing in impl produces E-TRAIT-003. Extra method in impl (not on trait) produces E-TRAIT-004. Duplicate
method name across supertraits produces E-TRAIT-005 at the use site. Associated type count mismatch produces
E-TRAIT-006.

### 6.7 Impls

Two kinds:

```kowari
impl Vec<i32> { … } // inherent impl
impl Format for Point { … } // trait impl
```

Inherent impls. Add methods to a type without a trait. A type may have multiple inherent impls; method names must be
unique across all of them (no shadowing, no ambiguity). Duplicates produce E-TRAIT-004.

Trait impls. Provide the trait's required items. Missing items produce E-TRAIT-003. Extra items produce E-TRAIT-004.
Wrong generic parameters produce E-TRAIT-007.

Where clauses may appear before the impl body to constrain generic parameters.

Overlapping impls. Two impls of the same trait for overlapping types produce E-TRAIT-008. The compiler shall check
overlaps (either structurally or by a documented conservative approximation — v0 may use the exact structure, since no
specialization is permitted).

Coherence is checked at trait-impl boundaries. An impl whose trait and type are both foreign is rejected at declaration
time, not at use.

### 6.8 type and const

Type aliases (type Foo = Bar;) are transparent: Foo and Bar are the same type to the checker. Aliases may have generic
parameters: type Pair<T> = (T, T);. Aliases are not newtypes — creating one requires struct Foo (Bar), which is nominal.

Constants (const NAME: T = expr;) are compile-time values evaluated at declaration. A const may be used in any
expression context where a value of type T is expected. Const evaluation is total: division by zero, overflow, and
array-index out-of-bounds in a const produce E-EXP-005 / E-EXP-006 / E-INT-003. A const may reference other consts;
cycles produce E-NAME-002.

static (mutable module state) is reserved for v1+ (§3.5) and produces E-LEX-013 on use.

### 6.9 Attributes

Attributes are `#[...]` markers before an item. In v0 the following attributes are active:
| Attribute | Applies to | Effect |
|---|---|---|
| `#[repr(C)]` | Struct, enum, union | Fixed C layout (§4.7) |
| `#[repr(Rust)]` | Struct, enum | Default unspecified layout (explicit) |
| `#[repr(u8)]`, `#[repr(u16)]`, … | Enum | Fixed tag width |
| `#[repr(packed)]` | — | Not in v0; `E-V1-001` |
| `#[must_use]` | Function, type | Warns on discarded return (`W-EXP-001`) |
| `#[allow(diagnostic)]` | Item, statement | Suppresses a warning |
| `#[deprecated]` | Item | Warns on use (`W-NAME-001`) |

The following are reserved (parsed, E-V1-001 at
use): #[derive (...)], #[inline], #[cold], #[target_feature], #[no_mangle], #[export_name], #[link (...)], #[global_allocator], #[test], #[bench], #[cfg (...)].

An unknown attribute name produces E-ATTR-001. An attribute in an invalid position produces E-ATTR-002. #[repr] on an
invalid target produces E-ATTR-003.

> **Rationale.** Every reserved attribute has a defined target. Silently ignoring an attribute (as some languages do) is
> worse than rejecting it, because it makes builds mysteriously differ between compilers.

### 6.10 Declaration diagnostics

| Code          | Trigger                                          |
|---------------|--------------------------------------------------|
| `E-METH-001`  | &mut self on immutable place                     |
| `E-METH-002`  | self receiver on a place already borrowed        |
| `E-METH-003`  | receiver type mismatch                           |
| `E-STRCT-001` | missing struct field                             |
| `E-STRCT-002` | extra struct field                               |
| `E-STRCT-003` | duplicate struct field initializer               |
| `E-ENUM-001`  | duplicate enum variant name                      |
| `E-TRAIT-001` | trait bound not satisfied                        |
| `E-TRAIT-002` | orphan rule violation                            |
| `E-TRAIT-003` | missing trait member in impl                     |
| `E-TRAIT-004` | extra member in impl / duplicate inherent method |
| `E-TRAIT-005` | ambiguity across supertraits                     |
| `E-TRAIT-006` | associated type count mismatch                   |
| `E-TRAIT-007` | generic parameter mismatch in impl               |
| `E-TRAIT-008` | overlapping impls                                |
| `E-ATTR-001`  | unknown attribute                                |
| `E-ATTR-002`  | attribute in invalid position                    |
| `E-ATTR-003`  | #[repr] on invalid target                        |

## 7 Statements

### 7.1 let statements

```kowari
let x = 5; // inferred type
let y: i32 = 5; // explicit type
let mut z = 10; // mutable binding
let (a, b) = pair; // destructuring
let Point { x, y } = p; // struct destructuring
let Some (v) = maybe else { return; }; // refutable, needs else
```

Initialization. Every let shall initialize its binding. let x: i32; without an initializer is a declaration of an
uninitialized binding; it may be initialized later, but reading an uninitialized binding produces E-LET-001. This
differs from "definite assignment analysis" — v0 requires initialization on the let or at some subsequent statement on
every path, checked by a forward dataflow pass.

Shadowing. A let may shadow a previous binding in the same scope. The new binding's type may differ from the old.
Shadowing is not mutation: it creates a new binding, and the old one is inaccessible by name. There is no E-NAME code
for this — shadowing is a feature.

Refutable patterns. A refutable pattern (one that can fail: Some (x), [a, b], or-patterns) in a let shall have an else
block that diverges (return, break, continue, panic!, or otherwise has type !). Without else, the pattern is rejected
with E-PAT-003.

Mutable bindings. let mut x allows x (and, transitively, fields of x) to be assigned to, moved out of, or borrowed
mutably (&mut x).

### 7.2 Expression statements

An expression followed by ; is an expression statement. The expression's value is discarded, which produces W-EXP-001 if
the expression's type carries #[must_use]. An expression statement is a statement, not a tail expression — it does not
contribute to the block's value.

A block with a trailing expression T in a () context (e.g., the last block of an if without else) produces E-TYPE-001
(§5.9).

### 7.3 defer

```kowari
fn process (path: &Path) {
let file = open (path);
defer { file.close () };
// … work …
// file.close () runs here, on return or normal end.
}
```

A defer block schedules the enclosed block to run when the enclosing scope exits. Multiple defers run in reverse order
of registration. defer runs on return, on break/continue, and on normal exit. defer does not run on panic (§5.11 —
panic = abort, no unwinding).

defer may not itself contain a return (§14.2, reasons). Attempting to return from inside a defer produces E-DEFER-001.
defer may not contain a break that would exit a loop outside its containing scope (§7.4); produce E-DEFER-002.

### 7.4 break and continue

break and continue target the innermost enclosing loop. In v0 there is no labeled break — you cannot exit an outer loop
from an inner loop. This is a v1 ergonomic ('outer: loop { break 'outer; }); in v0 it is a design hole that may be
closed in v1 by labeled-loop syntax.

### 7.5 Block-internal items

fn, struct, enum, const, type, mod, and trait may appear at the top level of a block. A block-internal item is visible
only within its block (and its nested blocks), and does not participate in the module's namespace.

> **Rationale.** This makes small helper functions local without requiring a let f = |...| closure. It also composes
> with impl inside functions for one-off trait impls, though that pattern is rare enough not to need dedicated support.

### 7.6 Statement diagnostics

| Code          | Trigger                                                                |
|---------------|------------------------------------------------------------------------|
| `E-LET-001`   | use of an uninitialized binding                                        |
| `E-DEFER-001` | return inside defer                                                    |
| `E-DEFER-002` | break/continue crosses a defer boundary                                |
| `E-STMT-001`  | item declaration with invalid visibility (e.g., pub at function scope) |
| `W-EXP-001`   | discarded value of #[must_use] type                                    |

## 8 Patterns

### 8.1 Pattern categories

Patterns are either irrefutable (always match — _, x, (a, b) of irrefutable parts) or refutable (may fail — Some (x),
literals, or-patterns). The distinction is checked: an irrefutable pattern is required in let, function parameters,
closure parameters, and for bindings. A refutable pattern in those positions produces E-PAT-003.

Patterns appear in: let, match arms, function parameters, closure parameters, for loops, and if let (not in v0 — match
covers it).

### 8.2 Pattern kinds

| Kind         | Example                             | Notes                                                                     |
|--------------|-------------------------------------|---------------------------------------------------------------------------|
| Wildcard     | `_`                                 | Matches any value; binds nothing                                          |
| Binding      | `x`, `mut x`                        | Binds by value (copies if `Copy`, moves otherwise)                        |
| Ref binding  | `ref x`, `ref mut x`                | v1                                                                        |
| Literal      | `0`, `"abc"`, `true`                | Matches exactly                                                           |
| Range        | `0..=10`                            | Inclusive; `..` is exclusive; endpoints must be literals of the same type |
| Tuple        | `(a, b)`, `(a, ..)`                 | `..` absorbs the rest                                                     |
| Array        | `[a, b, c]`, `[first, .., last]`    | Fixed length; `..` absorbs the middle                                     |
| Tuple struct | `Meters(v)`                         | Unwraps the payload                                                       |
| Struct       | `Point { x, y }`, `Point { x, .. }` | Field patterns; `..` absorbs the rest                                     |
| Enum variant | `Shape::Circle(r)`                  | Variant with payload                                                      |
| Or-pattern   | `A(x)                               | B(x)`, `Some(1)                                                           | Some(2)` | At least two alternatives |
| Guarded      | `Some(x) if x > 0`                  | Match arm only; not allowed in `let`                                      |
| `@` binding  | `x @ 1..=10`                        | Binds a variable to a matched subpattern                                  |

Range patterns. In v0, range endpoints must be literals of the same integer or char type; end must be ≥ start.
Non-literal endpoints produce E-PAT-006. Ranges on floats are rejected (E-PAT-007).

Or-pattern bindings. Every variable bound in one alternative shall be bound in every alternative, with the same type. A
(x) | B (x) is fine; A (x) | B (y) produces E-PAT-004. Bindings must have the same mutability across alternatives; mixed
x / mut x produces E-PAT-004.

### 8.3 Guards

A guard if cond may follow a pattern in a match arm. The guard is evaluated only if the pattern matches. Guards are not
permitted in let, function parameters, or for bindings.

A guard may reference any bindings made by the pattern. A guard may call functions and perform effects.

### 8.4 Exhaustiveness

The compiler shall prove that a match covers every value of its scrutinee's type. Failure produces E-PAT-001. The
compiler shall also prove that no arm is unreachable, producing W-PAT-002 for each unreachable arm.

Exhaustiveness is checked on the type, not on values. For a bool, true and false are the only cases. For an enum, every
variant must be covered (or an _ arm must exist). For integer types, _ is required unless every value is covered (which
is infeasible without a range analyzer; v0 accepts _ and rejects nothing for integers — no exhaustiveness complaints on
integers, but also no useful coverage analysis).

Nested exhaustiveness. A nested enum Option<Result<T, E>> requires covering None, Some (Ok (_)), Some (Err (_)) (or a
wildcard). The compiler shall perform exhaustiveness check recursively over payloads, so that Some (Ok (_)) and Some (
Err (_)) together cover Some (_), but not Some (_) alone.

### 8.5 Usefulness checking (unreachable arms)

The compiler shall perform a usefulness check: starting from the "unreachable space" (initially empty), each arm is
subtracted from the uncovered space and must cover at least one value; an arm covering nothing is unreachable and
produces W-PAT-002. This is the standard Maranget algorithm.

> **Rationale.** Exhaustiveness says "you have missed a case"; usefulness says "this case can never occur". The second
> is more valuable — it catches typos, dead code from API churn, and arms that were shadowed by a broader earlier arm.
> Rust has shipped it for years and it is a genuine quality-of-life win.

### 8.6 Decision-tree lowering

Pattern matching is compiled to a decision tree:

- If all arms match on the same constructor with the same shape, descend to subpatterns.

- If arms differ, branch on the scrutinee's discriminant (enum tag) or comparison (literal / range).

- Guards evaluate after the pattern matches at a leaf.

- Or-patterns are flattened before lowering (each alternative becomes a separate path through the tree).

- The tree is compiled to a switch when the branches are on integers or enums, and to a chain of comparisons when they
  are not.

Order of evaluation. Scrutinee first, then discriminant tests in source order within each branch level, then guards in
source order. Bindings inside an arm are evaluated lazily — a binding is only established if its pattern actually
matches. This is observable only through side effects in guards and through panic messages, but is specified for
consistency.

Irrefutable patterns are not lowered to trees — they are direct destructuring with no runtime check. let (a, b) = pair;
compiles to two field accesses, not a comparison.

The compiler shall not evaluate the scrutinee more than once, regardless of how many discriminant tests it performs.
Subpattern bindings that are not used may be eliminated (dead-binding elimination).

### 8.7 Pattern diagnostics

| Code        | Trigger                                              |
|-------------|------------------------------------------------------|
| `E-PAT-001` | non-exhaustive match                                 |
| `W-PAT-002` | unreachable match arm or pattern                     |
| `E-PAT-003` | refutable pattern where irrefutable required         |
| `E-PAT-004` | inconsistent bindings across or-pattern alternatives |
| `E-PAT-005` | guard in a non-match context                         |
| `E-PAT-006` | non-literal range endpoint                           |
| `E-PAT-007` | range on non-integer, non-char type                  |
| `E-PAT-008` | duplicate binding name within one pattern            |

## 9 Modules and visibility

### 9.1 The module tree

A crate is rooted at a module named crate. Modules are declared with mod:

```kowari
mod geometry { // inline module
mod point { … }
}
mod io; // file-backed module: `io.ko` or `io/mod.ko`
```

Inline vs. file-backed. A mod name { … } declares an inline module. A mod name; loads from disk. The compiler shall
resolve mod foo; to foo.ko in the directory of the file containing the declaration; if foo.ko is absent and foo/mod.ko
is present, that is used instead. Both being present produces E-MOD-004.

Duplicate modules. Two mod name; in the same parent produce E-NAME-002 regardless of file.

Module paths. Every item has a canonical path from crate, e.g., crate::geometry::point::Point. Paths are ::-separated
segments; the leading crate:: may be replaced by self:: (current module) or super:: (parent module).
| Prefix | Meaning |
|---|---|
| `crate::` | Crate root |
| `self::` | Current module |
| `super::` | Parent module (none at the crate root); resolution uses §9.4 |

### 9.2 Visibility

Private by default. Every item is visible only within the module that declares it, unless it carries a pub visibility
modifier.
| Modifier | Visible to |
|---|---|
| (none) | Declaring module only |
| `pub` | Entire crate, plus downstream crates (if the module path is public) |
| `pub(crate)` | Entire crate, not downstream |
| `pub(package)` | Synonym for `pub(crate)`; may be unified with `pub(crate)` in v1 |

Field visibility. A struct's fields may have individual visibility. A private field makes the whole struct
unconstructible outside its module — a destructurable-by-construction rule, i.e., you can destructure Point { x, .. }
outside the module only if you have access to x and .. is permitted. Attempting to construct or match on a private field
produces E-MOD-001.

Enum variant visibility. Enum variants inherit the enum's visibility. Enum fields (payloads) inherit the variant's
visibility. There is no way to make an enum's payload private — that would require the enum to be unusable at all. If an
enum is pub, all its variants are pub, and all their payload fields are pub. This is a v0 restriction; v1 may relax it,
but the workaround (private struct wrapper) already exists.

Trait item visibility. Trait methods and associated items are as public as the trait itself. There is no pub on trait
members.

pub exceeding parent visibility. An item may not be more visible than its parent module. mod m { pub fn f () {} } in a
pub (crate) module m produces E-MOD-002.

### 9.3 use imports

```kowari
use crate::geometry::Point; // one item
use crate::geometry::{Point, Shape}; // several
use crate::geometry::{Point as P, Shape}; // renamed
use crate::geometry::*; // glob
use super::sibling::helper; // relative
```

Imports bring a name into scope. use crate::a::b::c; brings c into the current module's namespace. The imported name may
be shadowed by a later use, a mod, or an item of the same name — the most recent declaration wins, and a duplicate use
of the same name at the same level produces E-NAME-002.

Glob imports bring every pub item from the module into scope. Two glob imports that both bring in the same name produce
E-NAME-004 at any use of the ambiguous name — not at import time. A single name resolving ambiguously across glob and
non-glob imports is resolved in favor of the non-glob.

Re-exports. pub use crate::geometry::Point; re-exports the imported name with the visibility specified. A re-export of a
private item produces E-MOD-001.

Import cycles. Module import cycles are permitted; item-level cycles (a const referencing a const that references it
back) produce E-MOD-003.

Unused imports. An import that is not referenced anywhere in the module produces W-MOD-001. This is a warning in v0; it
can be promoted to an error via #[deny (unused_import)] on the module or crate.

### 9.4 Name resolution

Resolution of a path segment proceeds in this order:

- Items in the current scope (functions, structs, enums, traits, consts, types, mods, and used names from earlier in the
  same block).

- Parent modules' items, walking outward until crate.

- The prelude (§9.5).

- If nothing matches, E-NAME-001.

Shadowing. A binding in an inner scope shadows an item from an outer scope (and a use in an inner block shadows the
outer name). A let cannot shadow a type or module with a value — that would create a namespace collision; it produces
E-NAME-003.

Type vs. value namespaces. Kowari has two namespaces: types (structs, enums, traits, type aliases, generic parameters)
and values (functions, consts, let bindings, enum variants). A name may exist in both without conflict: struct Foo; and
const Foo: i32 = 1; coexist. Foo in a value position refers to the const; in a type position, the struct. Tuple struct
constructors are values; the struct name is a type.

Resolution across modules. A path crate::a::b::c resolves c in module a::b, regardless of the current module. If a::b
re-exports c from elsewhere, the path resolves through the re-export.

### 9.5 The prelude

The prelude is the implicit top-of-scope imports available to every module. It contains:

- Option, Some, None

- Result, Ok, Err

- String

- Vec, HashMap, BTreeMap, HashSet, BTreeSet, VecDeque

- Format (trait)

- Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Default

- Into, From (v1-activatable; v0 declares Into/From but only .into () / .from () inherent methods exist)

- Iterator, IntoIterator, Extend

- format!, println!, eprintln!, debug_assert!, assert!, panic!

> **Rationale.** Preludes are the largest silent API surface in any language. Enumerating it in the spec — rather than
> leaving it to library documentation — means users can reason about name resolution exactly.

### 9.6 Module diagnostics

| Code         | Trigger                                         |
|--------------|-------------------------------------------------|
| `E-MOD-001`  | item or field is private                        |
| `E-MOD-002`  | pub exceeds parent module's visibility          |
| `E-MOD-003`  | const / type definition cycle                   |
| `E-MOD-004`  | mod foo; where both foo.ko and foo/mod.ko exist |
| `E-NAME-001` | unresolved name                                 |
| `E-NAME-002` | duplicate definition in one scope               |
| `E-NAME-003` | let shadows a type or module                    |
| `E-NAME-004` | ambiguous glob import                           |
| `E-NAME-005` | self outside method                             |
| `E-NAME-006` | super at crate root                             |
| `W-MOD-001`  | unused import                                   |
| `W-NAME-001` | use of a #[deprecated] item                     |

## 10 Integer Literal Typing

### 10.1 Unsuffixed integer literals

An unsuffixed integer literal (5, 0xFF, 0b1010) has no inherent type. Its type is determined by inference from context:

- If the literal appears in an expression whose expected type is an integer type T, the literal has type T.

- If inference reaches the end of the enclosing function without a type constraint, the literal defaults to i32.

- If inference encounters multiple constraints that do not unify, produce E-TYPE-001.

```kowari
let x = 5; // 5: i32 (default)
let y: u8 = 5; // 5: u8 (expected)
let z = 5u8; // 5: u8 (suffixed)
let w = x + 1; // 1: i32 (unified with x)
let v = take_u8 (5); // 5: u8 (parameter type)
let u: u8 = 5i32; // E-TYPE-001: i32 vs u8
```

Contexts that constrain. Function arguments (parameter types), let annotations, struct field initializers, pattern
positions, return expressions, binary-operator operands (unified with the other operand), array initializers, and as
casts.

Contexts that do not constrain. if conditions (must be bool, so integer literals are rejected with E-TYPE-001 unless
compared), match scrutinees without an inferred type (default to i32), statement-level expressions whose value is
discarded (default to i32).

> **Rationale.** Two policies compete for unsuffixed literals: default-to-i32 (C, Rust) and always-infer-or-error
> (Haskell). The former is what most systems code expects — let x = 5 just works — and i32 is the lowest-surprise
> default.
> The latter is stricter but surprises everyone who has never seen it. v0 adopts the former, with the strictness of
> E-TYPE-001 on conflicting constraints.

Diagnostics:
| Code | Trigger |
|---|---|
| `E-INT-001` | integer literal does not fit in its inferred type |
| `E-INT-002` | ambiguous integer type at use site (reserved for v1; not produced in v0) |
| `E-INT-003` | integer overflow in constant evaluation |

E-INT-001 example: let x: u8 = 300; — 300 does not fit in u8, and no context lowers it. Produced at the let, with the
span of the literal.

### 10.2 Float literal typing

Unsuffixed float literals (1.5, 3.0e10) default to f64. let x: f32 = 1.5; gives f32. Mixing f32 and f64 in an expression
produces E-TYPE-001 — there is no implicit float widening, unlike C.

### 10.3 Integer operators

Binary arithmetic (+ - * / %) on two integer operands of the same type produces the same type. Mixed-type operands
produce E-TYPE-001 unless explicitly cast with as. The same rule applies to bitwise and shift operators. << and >>
require both operands to be the same integer type.

Comparison operators (< <= > >=) require the same type on both operands; the result is bool.

== and != require the same type; the result is bool.

### 10.4 Overflow semantics

| Build mode | `+`, `-`, `*`, unary `-` overflow | Division by zero |
|------------|-----------------------------------|------------------|
| Debug      | Panic                             | Panic            |
| Release    | Wrap                              | Panic            |

panic here means panic!("integer overflow") — process abort, no unwinding (§5.11). The compiler shall not optimize on
the assumption that overflow does not occur, because it does occur (wrapping) in release builds.

Checked variants. checked_add returns Option<T>. wrapping_add returns T. saturating_add returns T, clamped.
overflowing_add returns (T, bool). All four exist for each of + - * / % on each integer type, in the prelude.

### 10.5 Shift semantics

x << n and x >> n shall panic (in all build modes) if n >= bit_width (T). This is stricter than C (undefined behavior)
and Rust (n is checked only in debug builds) — the v0 choice is to panic in all modes because the alternative is silent,
mode-dependent results.

### 10.6 Casts (as)

x as U converts between primitive types. Permitted casts in v0:
| From | To | Behavior |
|---|---|---|
| `iN` | `iM` (`M ≤ N`) | Truncate high bits |
| `iN` | `iM` (`M > N`) | Sign-extend |
| `uN` | `uM` | Zero-extend or truncate |
| `iN` | `uM` | Reinterpret bits; truncate or zero-extend |
| `uN` | `iM` | Reinterpret bits; truncate or sign-extend |
| `iN`, `uN` | `f32`, `f64` | Nearest representable; overflow to `±∞` |
| `f32`, `f64` | `iN`, `uN` | Truncate toward zero; out-of-range saturates |
| `char` | `u32` | Unicode scalar value |
| `u8` | `char` | Check valid scalar; panic if not |
| `bool` | `u8` | `0` or `1` |
| `u8` | `bool` | `0` is false, `1` is true; other values panic |
| `&T` | `*const T`, `*mut T` | v1 (requires `unsafe`) |

as is the only way to convert between primitive numeric types. There is no implicit widening, no implicit narrowing, no
integer-to-float coercion.

Invalid casts produce E-TYPE-007:

- Between non-primitive types

- From T to T (no-op; use nothing)

- From integer to String

- From &T to &U (use a conversion trait in v1)

- From fn (...) to a non-function type

### 10.7 Integer diagnostics

| Code         | Trigger                                       |
|--------------|-----------------------------------------------|
| `E-INT-001`  | integer literal does not fit in inferred type |
| `E-INT-003`  | integer overflow in constant evaluation       |
| `E-TYPE-001` | mixed integer types in one operator           |
| `E-TYPE-007` | invalid as cast                               |

## 11 Strings and Formatting

### 11.1 String types

| Type            | Description                            |
|-----------------|----------------------------------------|
| `&str`          | Borrowed UTF-8 slice; unowned; `Copy`  |
| `String`        | Owned, heap-allocated, UTF-8, growable |
| `[u8]`, `&[u8]` | Raw byte slice (not necessarily UTF-8) |
| `char`          | One Unicode scalar                     |

&str is a fat pointer (ptr + len). String is a struct wrapping (ptr, len, cap). The prelude provides String::new (),
String::from (&str), String::with_capacity (n), and the usual push, push_str, pop, len, is_empty, chars, bytes, lines,
split, trim, parse<T>, as_str.

&str and String are distinct types. &String auto-derefs to &str only in function argument position (a narrow, documented
exception to the "no auto-deref" rule of §4.5):

```kowari
fn take (s: &str) { }
let owned: String = …;
take (&owned)                            // OK: &String coerces to &str in argument position
```

This is the only coercion-by-reference in v0. Everywhere else, explicit deref/borrow is required.

### 11.2 Format string syntax

The format! family takes a format string literal followed by arguments:

```kowari
format!("hello")                        // "hello"
format!("x = {}", x)                    // x: any impl Format
format!("{x} and {y}")                  // named (identifier shorthand)
format!("{0} then {1}", a, b)           // positional
format!("{a} then {b}", a = x, b = y)   // named explicit
format!("{:>10}", x)                    // width, right-align
format!("{:<10}", x)                    // left-align
format!("{:^10}", x)                    // center
format!("{:08.3}", f)                   // zero-pad, width 8, precision 3
format!("{:#x}", n)                     // hex with 0x prefix
format!("{:#b}", n)                     // binary with 0b prefix
format!("{:*^20}", s)                   // fill '*', center, width 20
format!("{{}}")                         // literal "{}"
```

**Grammar**

```text
format_string  ::= FORMAT_LITERAL
argument       ::= {[ arg_name "=" ] expr}
arg_spec       ::= "{" [ arg_ref ] [ ":" format_spec ] "}"
arg_ref        ::= INT | IDENT | (* empty = next positional *)
format_spec    ::= [ fill ] [ align ] [ sign ] [ "#" ] [ "0" ]
[ width ] [ "." precision ] [ type ]
fill           ::= any char except "{" or "}"
align          ::= "<" | ">" | "^"
sign           ::= "+" | "-"
width          ::= INT | "{" IDENT "}"
precision      ::= INT | "{" IDENT "}"
type           ::= "b" | "o" | "x" | "X" | "e" | "E" | "p" | "?"
```

Argument references

- {} — next positional argument

- {0}, {1} — explicit positional

- {name} — named argument (format!("{x}", x = y) or identifier shorthand format!("{x}") where x is a binding)

- {x:>10} — named with spec

Positional and named arguments may be mixed, but positional indices are counted across all explicit and implicit
positional arguments, in source order.
Type specifiers
| Specifier | Meaning | Constraint |
|---|---|---|
| (none) | User-defined display formatting | `T: Format` |
| `?` | Debug representation | `T: Debug` |
| `x`, `X` | Lowercase / uppercase hexadecimal | Integer |
| `b` | Binary | Integer |
| `o` | Octal | Integer |
| `e`, `E` | Scientific notation | Float |
| `p` | Pointer address | `&T`, `*const T`, `*mut T` |

The default ({}) uses Format, a trait in the prelude:

```kowari
trait Format {
fn fmt (&self, f: &mut Formatter) -> Result<(), FormatError>;
}

trait Debug {
fn fmt (&self, f: &mut Formatter) -> Result<(), FormatError>;
}
```

Formatter exposes the parsed spec (fill, align, width, precision, type) and a write_str method. Debug is what {:?} uses;
the two traits are separate because display and debug formatting are semantically distinct (e.g., String displays
without quotes but debugs with them).
Width and precision from arguments

Width and precision may be taken from a named argument: {:>width$} with width = 10 in the arguments. The $ suffix marks
the reference as a width/precision source rather than an argument to format.

### 11.3 Argument evaluation order

Arguments are evaluated left-to-right, before the format string is parsed. This matches §5.6 (function call evaluation).
Argument evaluation happens even if the corresponding format spec is unreachable, because the format string is known at
compile time but the argument expressions are not.

### 11.4 Compile-time verification

The format string shall be checked at compile time:

- Positional index out of range → E-STR-003

- Named argument not supplied → E-STR-003

- Type spec incompatible with argument type → E-STR-002

- Unbalanced {/} → E-STR-001

- Unknown type spec → E-STR-001

- Missing required trait (Format or Debug) → E-STR-002

A format! whose string is not a literal is not permitted in v0; use Formatter directly for runtime-format-string
scenarios. This is a v1 ergonomic to be revisited.

### 11.5 Related macros

The prelude provides:
| Macro | Effect |
|---|---|
| `format!(...)` | Returns `String` |
| `println!(...)` | Writes to stdout with newline; returns `()` |
| `eprintln!(...)` | Writes to stderr with newline |
| `print!`, `eprint!` | As above, without newline |
| `panic!(...)` | Aborts with message (see §5.11) |
| `assert!(cond)`, `assert!(cond, "...", ...)` | Panics if false |
| `debug_assert!(...)` | Like `assert!` in debug builds; compiles away in release |

assert! and debug_assert! expand to if !cond { panic!(...) }; the message is formatted with the same syntax.

> **Rationale.** Rust's format syntax is well-tested and familiar. The main departure here is stricter compile-time
> checking (Rust also checks, but the spec makes it a normative requirement rather than an implementation quality) and
> the
> decision not to permit runtime format strings in v0 (which simplifies the checker and makes the format a macro rather
> than a function).

### 11.6 String diagnostics

| Code         | Trigger                                                |
|--------------|--------------------------------------------------------|
| `E-STR-001`  | invalid format string syntax                           |
| `E-STR-002`  | format argument type does not implement required trait |
| `E-STR-003`  | format argument count / name mismatch                  |
| `E-STR-004`  | non-literal format string (v0)                         |
| `E-TYPE-001` | String vs. &str mismatch outside argument position     |

## 12 v1 Constructs (Parsed, Rejected in v0)

Every construct in this section is accepted by the parser (Appendix A) but produces E-V1-001 during semantic analysis,
with the span of the construct's v1 token or its first token. The purpose is forward compatibility: a v1 source file is
today a valid v0 parse, so tools (formatters, syntax highlighters, LSPs) can be written once.

### 12.1 unsafe { ... }

A block marker that, in v1, permits operations the type system cannot verify (raw pointer dereference, union field
reads, calls to unsafe fn, transmutes). In v0 the parser accepts it; the checker rejects it. unsafe fn declarations are
likewise parsed and rejected.

v1 semantics summary (for forward reference): unsafe blocks do not turn off the affine checker; they widen the permitted
operations. An unsafe fn requires an unsafe block at every call site. A // SAFETY: comment is recommended before each
unsafe block and required by lint inside std (warn-by-default elsewhere). Raw pointers *const T, *mut T are constructed
via &x as *const T and dereferenced only inside unsafe.

### 12.2 extern "C" { ... } and extern "C" fn

FFI declarations. Parsed in v0; rejected at semantic analysis.

v1 semantics summary: extern "C" { fn foo (x: i32) -> i32; } declares a foreign function. extern "C" fn bar
(...) { ... } exports a Rust function under a C ABI. All types crossing the boundary shall be #[repr (C)]-compatible or
primitives; violations produce E-FFI-001 in v1. Function pointers to foreign functions are permitted; calls require
unsafe. #[no_mangle] and #[export_name] (reserved attributes in v0) control exported symbols.

### 12.3 spawn, sync { ... }, async { ... }

Structured concurrency primitives. Parsed; rejected.

v1 semantics summary (forward reference — see the design rationale in the current Spec.md §10): scope { spawn { ... } }
runs the inner blocks concurrently on OS threads and joins at scope exit. async { ... } is a lighter-weight form that
runs on the same thread cooperative scheduler. sync { ... } provides a critical section (a Mutex around a block). All
three are lexically scoped — you cannot spawn a task that outlives its scope. This is what makes structured concurrency
eliminate the Send/Sync auto-trait complexity.

### 12.4 comptime

Compile-time evaluation. Parsed; rejected.

v1 semantics summary: comptime fn evaluates its body at compile time. The body shall be pure (no Store, no Time, no
allocations beyond compile-time arena) and shall terminate (checked by a conservative termination analysis: recursion
only through structural subterms). comptime { ... } blocks evaluate expressions at compile time and splice the result.
comptime if, comptime for provide compile-time branching and iteration. No type-level computation in v1 — that is
reserved for a v2 addition if warranted.

### 12.5 #[derive (...)]

Attribute form for automatic trait derivation. Parsed in v0 (with the attribute text preserved); rejected at expansion.

v1 semantics summary: #[derive (Eq, Ord, Hash, Debug, Clone, Default)] generates structural implementations. The derive
set is closed — no user-defined derives in v1 (that would require a macro system, which is out of scope forever per
§2.2). Derives operate on all fields; a #[derive (Clone)] on a struct requires all field types to be Clone, checked at
expansion.

### 12.6 dyn Trait

Trait object syntax. Parsed; rejected.

v1 semantics summary: dyn Trait is an unsized type usable behind a pointer: &dyn Trait, Box<dyn Trait>. It uses a
vtable-based representation. Object safety rules: a trait is object-safe if all its methods have receivers (self, &self,
&mut self), it has no associated types without a where clause on Self, no generic methods, and no associated consts.
Object safety violations produce E-TRAIT-005 in v1. v0 reserves dyn as a keyword and parses the syntax; the semantic
checker rejects every use.

### 12.7 consume self and consume parameters

Affine resource consumption. Parsed; rejected. This is the v0→v1 migration path described in the current Spec.md §4.1:
the affine type system (with deinit, consume, and leak checking) is v1, but the syntax is fixed today so that v1 source
files parse correctly.

### 12.8 ? with From-conversion

In v0, ? requires the error type to match exactly (E-EXP-001 on mismatch). v1 relaxes to E: Into<E'> via From. The v0
rule is a narrowing, so v1 code that compiles will compile today; today's code will not silently change meaning.

### 12.9 @capabilities { ... } module attribute

Parsed; rejected. v1 will require modules to declare the capabilities they use (fs.read, net.connect, clock.now, …) and
reject modules whose bodies perform I/O outside their declared set. This is a v1 addition, not a v0 concern.

### 12.10 Other v1 syntax

The following tokens and constructs parse in v0 but are reserved for later activation:

- async, await (v1)

- const fn — parsed as synonym for comptime fn (v1)

- static — reserved, target v1+

- ref, ref mut in patterns (v1)

- Labeled loops ('label: loop { ... }) and labeled break (break 'label) — not in the v0 grammar; planned for v1. Source
  using them produces E-PARSE-001, not E-V1-001, because the grammar does not include them.

### 12.11 E-V1-001 details

E-V1-001 is emitted with a message of the form:

- E-V1-001: <construct> is reserved for a future language version (see §2.1); this program is syntactically valid Kowari
  0.x but requires <target> to compile.

The <target> field is filled in from the §2.1 table. Users may suppress the error at the declaration level
with #[allow (v1_001)] if they are writing shared v0/v1 libraries, but not at the use site — a v0 program that executes
a v1 construct is not permitted.

## 13 FFI in v0

### 13.1 What works in v0

Only two things:

- #[repr (C)] on structs, enums, and unions — the layout attribute is active (§4.7). This means a struct can be laid out
  to match a C ABI even in v0, ready for v1 FFI.

- Primitive and compound types with defined layouts — i32, f64, u8, tuples of #[repr (C)] structs, arrays of primitives,
  and similar. These are all FFI-safe by type.

### 13.2 What does not work in v0

- extern "C" { ... } blocks — parsed, E-V1-001.

- extern "C" fn declarations — parsed, E-V1-001.

- unsafe { ... } blocks — parsed, E-V1-001.

- #[no_mangle], #[link (...)], #[export_name] — parsed, E-V1-001.

- Raw pointers *const T, *mut T — the type syntax is reserved, and no raw pointer is constructible in v0. &T as *const T
  produces E-V1-001.

### 13.3 Why ship #[repr (C)] before FFI?

Layout attributes are free: they change nothing about the type checker and are a strict improvement over "field order is
unspecified." Shipping them in v0 means:

- Users can define #[repr (C)]-compatible types today, for later FFI.

- The compiler's layout algorithm is exercised by tests long before FFI is landed.

- Debugging layout issues in v0 doesn't require shipping FFI first.

Conversely, shipping unsafe or FFI before the affine type system is fully implemented would allow type-unsafe operations
with no way to check them. That's the wrong order.

### 13.4 Types with defined FFI layout

The spec shall guarantee the following types have the same layout as their C counterparts, at every optimization level
and on every supported target:
| Kowari type | C equivalent |
|---|---|
| `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` | `int8_t` … `uint64_t` |
| `isize`, `usize` | `intptr_t`, `uintptr_t` |
| `f32`, `f64` | `float`, `double` |
| `bool` | `_Bool` (size and alignment only; see below) |
| `#[repr(C)] struct { … }` | Matching C `struct { … }` |
| `#[repr(C)] union { … }` | Matching C `union { … }` |
| `#[repr(u8)] enum { … }` | `uint8_t` tag plus payload (payload order implementation-defined) |
| `*const T`, `*mut T` | `const T*`, `T*` |
| `Option<&T>` | `T*` (null-optimized; §4.8) |
| `Option<fn(...)>` | Function pointer or null |

bool is not guaranteed to be _Bool-compatible for values (Kowari bool is 1 byte, false = 0, true = 1, no other values).
Passing a bool across FFI requires it to hold exactly 0 or 1; C's _Bool guarantees this; a C function that writes an
arbitrary byte into a bool produces a bool that violates the invariant, and any subsequent use is undefined behavior in
v1. In v0 no such write is possible (no FFI), so the guarantee is purely forward-looking.

### 13.5 Padding and zero-initialization

`#[repr (C)]` structs have the same padding rules as the platform's C compiler (C11 §6.7.2.1). Zero-sized types have
size
0; arrays of zero-sized types have size 0 (not 1, unlike C++ empty-base optimization). This is a layout guarantee, not a
suggestion.

## 14 Open Questions

These are known spec holes. They are documented so implementations do not diverge, but the spec does not yet fix an
answer. Each entry describes the question, the options considered, and the current best guess.

### 14.1 Labeled loops and labeled break

Question: Does v0 need 'outer: loop { break 'outer; }?

Options:

- **(a)** Keep as v1; users write loop-with-flag or restructure.

- **(b)** Add in v0: requires grammar change ('label: before loop/while/for, break 'label).

- **(c)** Add a labels extension via macros (rejected — no macros in v0).

Current position: v0 omits. Adding it is a backwards-compatible grammar addition (no existing program becomes
ambiguous), so it can be added in v0.1 or v1 without breaking anything. Not a blocker for v0.

Recommendation: Ship v0 without labels; add in v0.1 if early adopters complain.

### 14.2 return inside defer

Question: Should return inside a defer block be permitted, and if so, should it override the outer return?

Options:

- **(a)** E-DEFER-001 (current spec §7.3) — defer may not return.

- **(b)** Permitted; the defer's return overrides any value the enclosing function was about to return (Go's named
  return semantics, Ruby's ensure).

- **(c)** Permitted, but only if it returns the same type, and the outer return is silently overwritten (C# finally with
  return inside).

Current position: (a). return inside defer is rejected with E-DEFER-001.

Why the spec picks (a): defer runs during unwinding-by-return, which is a state the type system cannot reason about. The
options that permit it all require the compiler to track a "pending return value" that the defer may mutate, and this
interacts badly with ?, with early returns inside loops, and with break. Rejecting it entirely keeps defer simple and
predictable. Users who need this write an explicit let result = ...; defer { cleanup () }; result at the end.

Reconsider in v1 if a strong use case appears.

### 14.3 mut on struct fields

Question: Should fields have independent mutability (struct S { mut x: i32 })?

Options:

- **(a)** No field-level mut (current spec §6.3). Mutability belongs to the binding.

- **(b)** Yes, per-field mut; the binding's mutability does not propagate.

- **(c)** Yes, per-field mut is the only mutability; bindings are always mutable.

Current position: (a). This matches Rust's design, and the rationale is that a binding can be borrowed partially, so a
struct with mutable and immutable fields can be modeled as two structs or as Cell-like wrappers.

Reconsider in v1 if use cases emerge (interior mutability patterns are the most likely).

### 14.4 Coherence and negative reasoning

Question: What happens when a downstream crate adds an impl that would conflict with one in the current crate?

Current position: Coherence is checked at impl declaration (§6.6). A program that compiles today cannot be broken by a
downstream crate adding an impl — because downstream impls cannot exist for local types with local traits, and any impl
of a local trait for a foreign type that would conflict is caught at the local boundary. This is the "closed world"
assumption and it holds for v0 because there are no blanket impls (impl<T> Trait for T).

Open sub-question: Should blanket impls be permitted in v1? If they are, coherence must handle overlapping blanket impls
and the "negative reasoning" problem (knowing that no impl could exist). Rust's approach is to forbid blanket impls
except through specialization (still unstable). The spec defers this to a v1 design discussion.

### 14.5 Object safety rules (v1)

Question: What makes a trait usable as dyn Trait?

Current proposal (v1 draft in §12.6): A trait is object-safe if all its methods have receivers, it has no associated
types without a where Self: Sized escape, no generic methods, and no associated consts.

Open questions:

- Should where Self: Sized allow non-object-safe methods to be present but not dispatchable through the vtable? (Rust
  says yes.)

- Should associated types be allowed with a where clause that names them? (Rust says yes, via dyn Trait<Assoc = T>.)

- Should dyn dispatch require an explicit dyn keyword, or should Box<Trait> be interpreted as Box<dyn Trait>? (Rust
  chose dyn Trait explicitly; the v0 spec follows.)

Not a v0 blocker.

### 14.6 Integer literal inference edge cases

Question: Are there contexts where the "default to i32" rule of §10.1 surprises users?

Known edge cases:

- let x = 0; x = some_u64; — the second assignment must unify with i32, producing E-TYPE-001. Users may expect x to
  widen. The spec keeps the strict rule (unification, not widening).

- [0; 10] — the array's element type is i32 unless annotation says otherwise.

- fn f () { 0 } where f's return type is () — 0 is i32, discarded, produces W-EXP-001 because it's not () and not !.
  This is correct but surprising on first encounter.

Current position: Keep the strict rule. Document the edge cases in user documentation, not the spec.

### 14.7 Format string argument counting

Question: In format!("{0} {0}", x), is x evaluated once or twice?

Current position (§11.3): Arguments are evaluated left-to-right, before formatting begins. x is evaluated exactly once;
the value is used twice.

Open question: Should argument evaluation be lazy per-use (so a format string that never invokes {0} doesn't evaluate
x)? The spec says no, because (a) it complicates the compiler, (b) it makes side effects in arguments non-deterministic,
and (c) no mainstream language does this.

### 14.8 #[must_use] propagation

Question: Should a struct carrying #[must_use] propagate the warning when a field of it is discarded?

Current position (§6.9): #[must_use] applies to the type as a whole. Discarding a Result produces the warning;
discarding a field of a Result does not (the field was extracted by a match, which is a use).

Not a v0 blocker. Reconsider if users report false positives.

### 14.9 The &String → &str coercion

Question: Why does §11.1 permit &String → &str in argument position, when §4.5 says no auto-deref?

Current position: It's a narrow, documented exception to the no-auto-deref rule, justified by (a) the ergonomic value
being enormous — every string-taking function would otherwise require .as_str () — and (b) the coercion being trivially
sound (it's a subtyping relation, not a deref).

Open question: Should the same exception apply to &Vec<T> → &[T]? The spec says no in v0, because there's no Deref trait
to express the general form, and special-casing two types is where the slope starts. &String → &str is accepted because
it's the lowest-level, highest-frequency case; if &Vec<T> → &[T] is added later, it will need a broader justification.

### 14.10 panic! message stability

Question: Should the exact text of panic messages be stable?

Current position (§5.11): Panic messages include file and line, but the specific wording of framework panics (unwrap on
None, integer overflow, etc.) is not guaranteed stable. User panic!("...") messages are stable — they are exactly the
format string.

Open question: Should framework panic messages be stable for test assertions (assert_eq! relies on this)? The spec says
no in v0 because it constrains the standard library's ability to improve messages. Reconsider if users report test
fragility.

### 14.11 unreachable! and exhaustiveness

Question: Should unreachable!() exist as a way to silence exhaustiveness warnings?

Current position: Not in v0's prelude. The ! type (never) is a type, and a match arm that returns ! satisfies
exhaustiveness because ! is a subtype of every type. But unreachable!() is a macro, and v0's macro set is closed
(§11.5). Users write panic!("unreachable") if they need it; that does not silence exhaustiveness (the match still must
be exhaustive), which is the correct behavior.

Reconsider in v1 if a need emerges.

---

## Appendix A — EBNF Grammar

The grammar below defines every production accepted by the v0 parser. Constructs marked (* v1 *) are parsed but not
semantically accepted in v0; they produce E-V1-001. EBNF terminals in double quotes are literal; bare identifiers are
non-terminals. { X } is repetition, [ X ] is optional, | is alternation.

```
program            ::= item*

item               ::= { attribute } { visibility } item_kind

item_kind          ::= fn_item
| struct_item
| enum_item
| union_item
| trait_item
| impl_item
| mod_item
| use_item
| const_item
| type_alias_item
| extern_block            (* v1 *)
| comptime_item           (* v1 *)

visibility         ::= "pub"
| "pub" "(" ( "package" | "crate" ) ")"

attribute          ::= "#" "[" attr_body "]"
attr_body          ::= path [ attr_args ]
attr_args          ::= "(" attr_arg { "," attr_arg } [ "," ] ")"
attr_arg           ::= literal | path

# ── Functions ────────────────────────────────────────────────────────
fn_item            ::= "fn" IDENT [ generic_params ] "(" [ param_list ] ")"
[ "->" type ] [ where_clause ] block
param_list         ::= param { "," param } [ "," ]
param              ::= pattern ":" type
| "consume" pattern ":" type          (* v1 *)
generic_params     ::= "<" generic_param { "," generic_param } [ "," ] ">"
generic_param      ::= IDENT [ ":" trait_bounds ]
trait_bounds       ::= trait_bound { "+" trait_bound }
trait_bound        ::= path
where_clause       ::= "where" where_pred { "," where_pred } [ "," ]
where_pred         ::= type ":" trait_bounds

# ── Types in item position ───────────────────────────────────────────
struct_item        ::= "struct" IDENT [ generic_params ] struct_body
struct_body        ::= "{" [ field_list ] "}"
| "(" type_list ")" ";"
field_list         ::= field { "," field } [ "," ]
field              ::= { visibility } IDENT ":" type

enum_item          ::= "enum" IDENT [ generic_params ] "{" [ variant_list ] "}"
variant_list       ::= variant { "," variant } [ "," ]
variant            ::= IDENT [ variant_body ]
variant_body       ::= "(" type_list ")"
| "{" field_list "}"

union_item         ::= "union" IDENT "{" field_list "}"

trait_item         ::= "trait" IDENT [ generic_params ]
[ ":" trait_bounds ] "{" { trait_member } "}"
trait_member       ::= fn_sig
| fn_item_with_body                 (* default method *)
| type_assoc
| const_item

impl_item          ::= "impl" [ generic_params ] impl_target
[ where_clause ] "{" { item_kind } "}"
impl_target        ::= type                                  (* inherent impl *)
| trait_bound "for" type                (* trait impl *)

type_assoc         ::= "type" IDENT [ generic_params ] [ ":" trait_bounds ] ";"

# ── Modules ──────────────────────────────────────────────────────────
mod_item           ::= "mod" IDENT ( "{" { item } "}" | ";" )
use_item           ::= "use" use_tree ";"
use_tree           ::= path [ "::" "*" ]
| path "::" "{" use_tree { "," use_tree } [ "," ] "}"

const_item         ::= "const" IDENT ":" type "=" expr ";"
type_alias_item    ::= "type" IDENT [ generic_params ] "=" type ";"

# ── v1-only item forms (parsed; E-V1-001) ────────────────────────────
extern_block       ::= "extern" STRING "{" { fn_sig | item_kind } "}"
comptime_item      ::= "comptime" item_kind
fn_sig             ::= "fn" IDENT [ generic_params ] "(" [ param_list ] ")"
[ "->" type ] [ where_clause ] ";"

# ── Types ────────────────────────────────────────────────────────────
type               ::= type_primary { type_suffix }
type_primary       ::= path [ generic_args ]
| "(" [ type { "," type } [ "," ] ] ")"     (* unit or tuple *)
| "[" type ";" expr "]"                      (* array *)
| "[" type "]"                                (* slice *)
| "&" [ "mut" ] type
| "dyn" trait_bounds                          (* v1 *)
| "!"                                         (* never *)
| "_"                                         (* inferred *)
type_suffix        ::= "?"                                    (* Option sugar *)
generic_args       ::= "<" type { "," type } [ "," ] ">"
type_list          ::= type { "," type } [ "," ]

# ── Statements and blocks ────────────────────────────────────────────
block              ::= "{" { stmt } [ expr ] "}"
stmt               ::= let_stmt
| expr_stmt
| item_kind
| defer_stmt
| ";"
let_stmt           ::= "let" pattern [ ":" type ] [ "=" expr ] [ "else" block ] ";"
expr_stmt          ::= expr ";"
defer_stmt         ::= "defer" block

# ── Expressions ──────────────────────────────────────────────────────
expr               ::= or_expr [ "?" ]                        (* postfix try *)

or_expr            ::= and_expr { "||" and_expr }
and_expr           ::= bitor_expr { "&&" bitor_expr }
bitor_expr         ::= bitxor_expr { "|" bitxor_expr }
bitxor_expr        ::= bitand_expr { "^" bitand_expr }
bitand_expr        ::= eq_expr { "&" eq_expr }
eq_expr            ::= cmp_expr { ( "==" | "!=" ) cmp_expr }
cmp_expr           ::= shift_expr { ( "<" | "<=" | ">" | ">=" ) shift_expr }
shift_expr         ::= add_expr { ( "<<" | ">>" ) add_expr }
add_expr           ::= mul_expr { ( "+" | "-" ) mul_expr }
mul_expr           ::= unary_expr { ( "*" | "/" | "%" ) unary_expr }
unary_expr         ::= ( "-" | "!" | "&" [ "mut" ] | "*" ) unary_expr
| postfix_expr
postfix_expr       ::= primary_expr { postfix_op }
postfix_op         ::= "." IDENT [ "(" [ arg_list ] ")" ]     (* field or method *)
| "." INT                                 (* tuple index *)
| "[" expr "]"                            (* index *)
| "(" [ arg_list ] ")"                    (* call *)
primary_expr       ::= literal
| path
| "(" ")"                                 (* unit *)
| "(" expr ")"                            (* grouping *)
| "(" expr { "," expr } [ "," ] ")"       (* tuple *)
| "[" [ expr { "," expr } [ "," ] ] "]"   (* array *)
| "[" expr ";" expr "]"                   (* array repeat *)
| block
| if_expr
| match_expr
| loop_expr
| while_expr
| for_expr
| closure_expr
| "break" [ expr ]
| "continue"
| "return" [ expr ]
| "unsafe" block                          (* v1 *)
| "async" block                           (* v1 *)
| "spawn" block                           (* v1 *)
| "sync" block                            (* v1 *)
arg_list           ::= expr { "," expr } [ "," ]
closure_expr       ::= [ "move" ] "|" [ closure_params ] "|" expr
closure_params     ::= closure_param { "," closure_param } [ "," ]
closure_param      ::= pattern [ ":" type ]

if_expr            ::= "if" expr block [ "else" ( if_expr | block ) ]
match_expr         ::= "match" expr "{" { match_arm } "}"
match_arm          ::= pattern [ "if" expr ] "=>" expr [ "," ]
loop_expr          ::= "loop" block
while_expr         ::= "while" expr block
for_expr           ::= "for" pattern "in" expr block

# ── Patterns ─────────────────────────────────────────────────────────
pattern            ::= or_pattern
or_pattern         ::= primary_pattern { "|" primary_pattern }
primary_pattern    ::= "_"
| literal
| IDENT [ "@" pattern ]                   (* binding *)
| "ref" [ "mut" ] IDENT                   (* v1 *)
| path [ "(" [ pattern_list ] ")" ]       (* tuple variant *)
| path "{" [ field_pat_list ] "}"         (* struct variant *)
| "(" [ pattern { "," pattern } [ "," ] ] ")"
| "[" [ pattern { "," pattern } [ "," ] ] "]"
| ".."                                    (* rest, in nested *)
pattern_list       ::= pattern { "," pattern } [ "," ]
field_pat_list     ::= field_pat { "," field_pat } [ "," ]
field_pat          ::= IDENT [ ":" pattern ]
| ".."

# ── Paths and literals ───────────────────────────────────────────────
path               ::= path_seg { "::" path_seg }
path_seg           ::= IDENT | "self" | "Self" | "super" | "crate"

literal            ::= INT | FLOAT | STRING | RAW_STRING | CHAR
| BYTE_STRING | "true" | "false"
```

## Appendix B — Diagnostic Registry

Diagnostic codes are the stability contract. Once a code is published, its meaning shall not change. New codes are
additive; retired codes remain reserved. Each code follows the pattern E-CATEGORY-NNN (errors) or W-CATEGORY-NNN
(warnings). Categories are stable.

Category prefix legend:

| Prefix    | Domain                                      | v0 status |
|-----------|---------------------------------------------|-----------|
| `E-LEX`   | Lexical analysis                            | Active    |
| `E-PARSE` | Parsing                                     | Active    |
| `E-NAME`  | Name resolution                             | Active    |
| `E-TYPE`  | Type checking                               | Active    |
| `E-PAT`   | Pattern matching                            | Active    |
| `E-MOD`   | Modules and visibility                      | Active    |
| `E-TRAIT` | Traits                                      | Active    |
| `E-INT`   | Integer literal typing                      | Active    |
| `E-STR`   | Strings and formatting                      | Active    |
| `E-BUILD` | Build system                                | Active    |
| `E-TEST`  | Test framework                              | Active    |
| `E-V1`    | v1-only construct used in v0                | Active    |
| `E-OWN`   | Ownership / affine                          | v1        |
| `E-CONC`  | Concurrency                                 | v1        |
| `E-FFI`   | FFI / unsafe                                | v1        |
| `E-INC`   | Internal compiler error (never user-facing) | Active    |

Representative codes (full enumeration accompanies each section as it is written):

**E-LEX**

- `E-LEX-001` — unterminated string literal

- `E-LEX-002` — unterminated character literal

- `E-LEX-003` — unterminated block comment

- `E-LEX-004` — invalid escape sequence

- `E-LEX-005` — invalid character in source

- `E-LEX-006` — empty character literal

- `E-LEX-007` — multi-character character literal

- `E-LEX-008` — invalid integer literal

- `E-LEX-009` — integer literal out of range for suffix

- `E-LEX-010` — invalid float literal

- `E-LEX-011` — non-ASCII identifier (reserved; v1 will define Unicode identifiers)

- `E-LEX-012` — doc comment not attached to item

- `E-LEX-013` — reserved token used outside its target context

**E-PARSE**

- `E-PARSE-001` — unexpected token

- `E-PARSE-002` — expected closing delimiter

- `E-PARSE-003` — expected expression

- `E-PARSE-004` — expected type

- `E-PARSE-005` — expected pattern

- `E-PARSE-006` — mismatched delimiter

- `E-PARSE-007` — missing item name

- `E-PARSE-008` — unexpected attribute position

**E-NAME**

- `E-NAME-001` — unresolved name

- `E-NAME-002` — duplicate definition

- `E-NAME-003` — cannot shadow item with non-item

- `E-NAME-004` — ambiguous glob import

- `E-NAME-005` — self outside method

- `E-NAME-006` — super at crate root

**E-TYPE**

- `E-TYPE-001` — type mismatch

- `E-TYPE-002` — cannot infer type

- `E-TYPE-003` — unknown type name

- `E-TYPE-004` — wrong number of generic arguments

- `E-TYPE-005` — missing ? where Option expected

- `E-TYPE-006` — recursive type without indirection

**E-PAT**

- `E-PAT-001` — non-exhaustive match

- `E-PAT-002` — unreachable arm

- `E-PAT-003` — refutable pattern in let without else

- `E-PAT-004` — binding used inconsistently across or-patterns

- `E-PAT-005` — guard may not be arbitrary expression in non-match context

**E-MOD**

- `E-MOD-001` — item is private

- `E-MOD-002` — pub exceeds parent module's visibility

- `E-MOD-003` — import cycle

**E-TRAIT**

- `E-TRAIT-001` — trait bound not satisfied

- `E-TRAIT-002` — orphan rule violation

- `E-TRAIT-003` — missing trait member in impl

- `E-TRAIT-004` — method name collision in impl

- `E-TRAIT-005` — dyn trait is not object-safe (v1)

**E-INT**

- `E-INT-001` — integer literal does not fit in its inferred type

- `E-INT-002` — ambiguous integer type at use site

- `E-INT-003` — integer overflow in constant evaluation

**E-STR**

- `E-STR-001` — invalid format string

- `E-STR-002` — format argument type does not implement Format

- `E-STR-003` — positional argument count mismatch

**E-V1**

- E-V1-001 — construct is reserved for a future language version; see §2.1

**E-INC**

- `E-INC-001` — internal compiler error (please report)
