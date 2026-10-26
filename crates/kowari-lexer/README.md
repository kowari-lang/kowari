# Lexer internals

*`crates/kowari-lexer` — source text → tokens.*

## Design

Single-pass, `CharIndices`-based scanner. The entry point is
`pub fn lex(src: &str) -> Result<Vec<Token<'_>>, Vec<Diag>>`. On success,
the token vector always ends with a single `Eof` token. On failure, the
`Err` variant contains every diagnostic accumulated during the scan —
the lexer does not stop at the first error.

Tokens are zero-copy: `Identifier(&'src str)`, `Integer(&'src str)`,
`Float(&'src str)`, `String(&'src str)`, and `Char(&'src str)` all borrow
from the input. `String` and `Char` content slices exclude the
delimiters; `String` content includes escapes verbatim (`"a\"b"` →
`String(r#"a\"b"#)`).

## Files

| File                 | Responsibility                                            |
|:---------------------|:----------------------------------------------------------|
| `src/lib.rs`         | The scanner (`lex`), helpers, `dump_tokens`               |
| `src/token.rs`       | `Token`, `TokenKind`, `NUMERIC_SUFFIXES`, `Display` impls |
| `src/tests.rs`       | Unit tests                                                |
| `tests/golden.rs`    | Golden-test runner                                        |
| `tests/golden/pass/` | Inputs that must lex cleanly + `.expected`                |
| `tests/golden/fail/` | Inputs that must produce diagnostics + `.expected`        |

## Adding a token

1. Add the variant to `TokenKind` in `src/token.rs`.
2. Add the arm to `TokenKind::Display` — **use the enum variant name**
   (e.g. `Impl`, not `Implement`) so golden files are greppable against
   the enum.
3. Add the dispatch arm in `src/lib.rs` (`lex` for single-char,
   `lex_operator` for suffixed operators).
4. Add a golden pass case under `tests/golden/pass/`.
5. `UPDATE=1 cargo test -p kowari-lexer --test golden`.
6. Review the `git diff` of the new `.expected` file line-by-line.
7. If the token is spec-backed, cite the section in `Spec.md`. If not,
   add it to the **Provisional tokens** table in `Spec.md` §15.

## Adding a diagnostic

1. Add the code to `ErrorCode` in `kowari-diagnostics`.
2. Emit it from the lexer with a precise `Span`.
3. Add a golden fail case under `tests/golden/fail/`.
4. Regenerate + review as above.
5. Document the code in `docs/internals/diagnostics.md`.

## Non-panicking guarantee

Every arm of the main `match` in `lex` must return a value or push a
`Diag` and continue. No `unwrap`, no `expect`, no indexing that could
panic on invalid UTF-8 boundaries. The golden fail suite is the
regression test for this guarantee.

## Conventions

* **Spans are byte offsets**, not char offsets. For multi-byte chars,
  `start + c.len_utf8()` is the correct end; `start + 1` is a bug.
* **Display names match enum variants.** `PubToken` → `"Pub"`, not
  `"Public"`. This keeps `.expected` files greppable.
* **Suffix boundary.** A numeric suffix is only absorbed if the
  character after it is not alphanumeric and not `_`. This is what makes
  `1u32x` lex as `Integer("1")` + `Identifier("u32x")`.
* **`_` in identifiers.** A leading `_` starts an identifier, not a
  number. `_1` is `Identifier("_1")`, `1_` is a `BadNumericSeparator`
  diagnostic.