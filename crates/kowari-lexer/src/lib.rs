//! Lexer: source text → tokens.

use kowari_diagnostics::{Diag, ErrorCode, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind<'src> {
    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LSquareBracket, // [
    RSquareBracket, // ]
    Pipe,           // |
    Ampersand,      // &
    Caret,          // ^
    HashTag,        // #
    AtSign,         // @
    BiggerThen,     // >
    SmallerThen,    // <
    Exclamation,    // !
    Colon,          // :
    Semicolon,      // ;
    Dot,            // .
    QuestionMark,   // ?
    Tilde,          // ~

    // Multi-Operators
    LShift,             // <<
    RShift,             // >>
    LShiftEquals,       // <<=
    RShiftEquals,       // >>=
    Equals,             // =
    DoubleEquals,       // ==
    NotEquals,          // !=
    BiggerOrEquals,     // >=
    SmallerOrEquals,    // <=
    DoubleAnd,          // &&
    DoublePipe,         // ||
    PlusEquals,         // +=
    MinusEquals,        // -=
    StarEquals,         // *=
    SlashEquals,        // /=
    PercentEquals,      // %=
    DoubleColon,        // ::
    DoubleDot,          // ..
    RangeInclusive,     // ..=
    SmallArrow,         // ->
    BigArrow,           // =>
    DoubleQuestionMark, // ??

    // Overflow-policy operators (see Spec §11.2)

    // Wrapping (explicit wrap-around, modulo 2^N)
    PlusWrapping,   // +%
    MinusWrapping,  // -%
    StarWrapping,   // *%
    SlashWrapping,  // /%
    LShiftWrapping, // <<%
    RShiftWrapping, // >>%

    // Checked (returns Option<T>, None on overflow)
    PlusChecked,   // +?
    MinusChecked,  // -?
    StarChecked,   // *?
    SlashChecked,  // /?
    LShiftChecked, // <<?
    RShiftChecked, // >>?

    // Saturating (clamps to min/max of the type)
    PlusSaturating,  // +|
    MinusSaturating, // -|
    StarSaturating,  // *|

    // Keywords
    Function,
    Identifier(&'src str),
    LetToken,
    MutToken,
    IfToken,
    ElseToken,
    MatchToken,
    ForToken,
    WhileToken,
    LoopToken,
    ReturnToken,
    Defer,
    ErrDefer,
    StructToken,
    EnumToken,
    TraitToken,
    ImplToken,
    PubToken,
    UnsafeToken,
    ExternToken,
    InToken,
    RangeToken,
    AsToken,
    BreakToken,
    ContinueToken,

    // Literals
    Integer(&'src str),
    Float(&'src str),
    String(&'src str),
    Char(&'src str),
    TrueToken,
    FalseToken,

    // End Of File
    Eof,
}

/// Type suffixes valid on numeric literals. Keep in sync with the primitive
/// types in the Spec (§3).
const NUMERIC_SUFFIXES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64",
];

pub fn lex(src: &'_ str) -> Result<Vec<Token<'_>>, Vec<Diag>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut errors = Vec::new();
    let mut iter = src.char_indices().peekable();

    while let Some((start, c)) = iter.next() {
        if c.is_whitespace() {
            continue;
        }

        if c == '/' {
            match iter.peek().map(|(_, c)| *c) {
                Some('/') => {
                    iter.next();
                    skip_line_comment(&mut iter);
                    continue;
                }
                Some('*') => {
                    iter.next();
                    if let Err(span) = skip_block_comment(&mut iter, start, src.len()) {
                        errors.push(Diag {
                            code: ErrorCode::UnterminatedBlockComment.into(),
                            span,
                            message: "Unterminated block comment".to_string(),
                        });
                    }
                    continue;
                }
                _ => {} // `/`, `/=`, `/%`, `/?` fall through to the operator arm
            }
        }

        let opt: Option<(TokenKind, usize)> = match c {
            '+' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Plus,
                &[
                    ('=', TokenKind::PlusEquals),
                    ('%', TokenKind::PlusWrapping),
                    ('?', TokenKind::PlusChecked),
                    ('|', TokenKind::PlusSaturating),
                ],
            )),
            '-' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Minus,
                &[
                    ('=', TokenKind::MinusEquals),
                    ('%', TokenKind::MinusWrapping),
                    ('?', TokenKind::MinusChecked),
                    ('|', TokenKind::MinusSaturating),
                    ('>', TokenKind::SmallArrow),
                ],
            )),
            '*' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Star,
                &[
                    ('=', TokenKind::StarEquals),
                    ('%', TokenKind::StarWrapping),
                    ('?', TokenKind::StarChecked),
                    ('|', TokenKind::StarSaturating),
                ],
            )),
            '/' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Slash,
                &[
                    ('=', TokenKind::SlashEquals),
                    ('%', TokenKind::SlashWrapping),
                    ('?', TokenKind::SlashChecked),
                ],
            )),
            '%' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Percent,
                &[('=', TokenKind::PercentEquals)],
            )),
            '!' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Exclamation,
                &[('=', TokenKind::NotEquals)],
            )),
            '?' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::QuestionMark,
                &[('?', TokenKind::DoubleQuestionMark)],
            )),
            '=' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Equals,
                &[('=', TokenKind::DoubleEquals), ('>', TokenKind::BigArrow)],
            )),
            '&' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Ampersand,
                &[('&', TokenKind::DoubleAnd)],
            )),
            '|' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Pipe,
                &[('|', TokenKind::DoublePipe)],
            )),
            ':' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Colon,
                &[(':', TokenKind::DoubleColon)],
            )),
            '<' => {
                // Two-stage: `<<` has its own suffix set; otherwise `<` vs `<=`.
                if try_consume(&mut iter, '<') {
                    Some(lex_operator(
                        &mut iter,
                        src.len(),
                        TokenKind::LShift,
                        &[
                            ('=', TokenKind::LShiftEquals),
                            ('%', TokenKind::LShiftWrapping),
                            ('?', TokenKind::LShiftChecked),
                        ],
                    ))
                } else {
                    Some(lex_operator(
                        &mut iter,
                        src.len(),
                        TokenKind::SmallerThen,
                        &[('=', TokenKind::SmallerOrEquals)],
                    ))
                }
            }
            '>' => {
                if try_consume(&mut iter, '>') {
                    Some(lex_operator(
                        &mut iter,
                        src.len(),
                        TokenKind::RShift,
                        &[
                            ('=', TokenKind::RShiftEquals),
                            ('%', TokenKind::RShiftWrapping),
                            ('?', TokenKind::RShiftChecked),
                        ],
                    ))
                } else {
                    Some(lex_operator(
                        &mut iter,
                        src.len(),
                        TokenKind::BiggerThen,
                        &[('=', TokenKind::BiggerOrEquals)],
                    ))
                }
            }
            '(' => Some((TokenKind::LParen, start + c.len_utf8())),
            ')' => Some((TokenKind::RParen, start + c.len_utf8())),
            '{' => Some((TokenKind::LBrace, start + c.len_utf8())),
            '}' => Some((TokenKind::RBrace, start + c.len_utf8())),
            '[' => Some((TokenKind::LSquareBracket, start + c.len_utf8())),
            ']' => Some((TokenKind::RSquareBracket, start + c.len_utf8())),
            '~' => Some((TokenKind::Tilde, start + c.len_utf8())),
            '@' => Some((TokenKind::AtSign, start + c.len_utf8())),
            '.' => {
                if try_consume(&mut iter, '.') {
                    if try_consume(&mut iter, '=') {
                        Some((TokenKind::RangeInclusive, start + 3))
                    } else {
                        Some((TokenKind::DoubleDot, start + 2))
                    }
                } else {
                    Some((TokenKind::Dot, start + 1))
                }
            }
            '"' => {
                // Consume until unescaped closing " or EOF.
                let mut terminated = false;
                loop {
                    match iter.next() {
                        Some((_, '\\')) => {
                            // Skip the escaped char. Unterminated if EOF right after '\'.
                            if iter.next().is_none() {
                                break;
                            }
                        }
                        Some((_, '"')) => {
                            terminated = true;
                            break;
                        }
                        Some(_) => {} // any other char is fine
                        None => break,
                    }
                }

                let end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());

                if !terminated {
                    errors.push(Diag {
                        code: ErrorCode::UnterminatedString.into(),
                        span: Span {
                            start: start as u32,
                            end: end as u32,
                        },
                        message: "Unterminated string literal".to_string(),
                    });
                    None
                } else {
                    // `end` is one byte past the closing `"`; content is between the
                    // delimiters, so exclude one byte on each side.
                    let content_end = end - 1;
                    debug_assert!(content_end > start);
                    let content = &src[start + 1..content_end];
                    Some((TokenKind::String(content), end))
                }
            }
            '\'' => {
                let content_start = start + 1;

                // Read the single content unit.
                let Some((_, first)) = iter.next() else {
                    // `'` at EOF
                    errors.push(Diag {
                        code: ErrorCode::UnterminatedChar.into(),
                        span: Span {
                            start: start as u32,
                            end: (start + 1) as u32,
                        },
                        message: "Unterminated char literal".to_string(),
                    });
                    continue;
                };

                // Empty char: `''`
                if first == '\'' {
                    errors.push(Diag {
                        code: ErrorCode::EmptyChar.into(),
                        span: Span {
                            start: start as u32,
                            end: (start + 2) as u32,
                        },
                        message: "Empty char literal".to_string(),
                    });
                    continue;
                }

                // Escape: must be followed by exactly one char.
                if first == '\\' && iter.next().is_none() {
                    errors.push(Diag {
                        code: ErrorCode::UnterminatedChar.into(),
                        span: Span {
                            start: start as u32,
                            end: src.len() as u32,
                        },
                        message: "Unterminated char literal".to_string(),
                    });
                    continue; // diverges — fixes the discarded Option
                }

                // Enforce the closing quote.
                if !try_consume(&mut iter, '\'') {
                    // Malformed: too long (`'ab'`) or unterminated (`'a`).
                    // Recover by consuming up to and including the next `'`,
                    // stopping at newline if we hit one first.
                    let mut recovered_end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());
                    while let Some((i, ch)) = iter.peek().copied() {
                        if ch == '\n' || ch == '\r' {
                            recovered_end = i;
                            break;
                        }
                        if ch == '\'' {
                            iter.next();
                            recovered_end = i + 1;
                            break;
                        }
                        iter.next();
                        recovered_end = i + ch.len_utf8();
                    }
                    errors.push(Diag {
                        code: ErrorCode::MultiChar.into(),
                        span: Span {
                            start: start as u32,
                            end: recovered_end as u32,
                        },
                        message: "Char literal must contain exactly one character".to_string(),
                    });
                    continue;
                }

                // Success path only: `end` sits just past the closing `'`.
                let end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());
                let content = &src[content_start..end - 1];
                Some((TokenKind::Char(content), end))
            }
            // Numbers
            c if c.is_ascii_digit() => Some(lex_number(&mut iter, src, start, &mut errors)),
            // Identifiers/Key-Words
            c if c.is_alphabetic() || c == '_' => {
                while iter
                    .peek()
                    .is_some_and(|(_, c)| c.is_alphanumeric() || *c == '_')
                {
                    iter.next();
                }

                let end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());
                let text = &src[start..end];
                let kind = check_for_keyword(text);
                Some((kind, end))
            }
            _ => {
                errors.push(Diag {
                    code: ErrorCode::UnexpectedToken.into(),
                    span: Span {
                        start: start as u32,
                        end: (start + c.len_utf8()) as u32,
                    },
                    message: format!("Unexpected token '{c}'"),
                });
                None
            }
        };
        if let Some((kind, end)) = opt {
            tokens.push(Token {
                kind,
                span: Span {
                    start: start as u32,
                    end: end as u32,
                },
            })
        }
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span {
            start: src.len() as u32,
            end: src.len() as u32,
        },
    });

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(tokens)
}

fn lex_number<'src>(
    iter: &mut std::iter::Peekable<std::str::CharIndices<'src>>,
    src: &'src str,
    start: usize,
    errors: &mut Vec<Diag>,
) -> (TokenKind<'src>, usize) {
    // --- Radix-prefixed integers ------------------------------------------
    if src.as_bytes()[start] == b'0' {
        let radix = match iter.peek().map(|&(_, c)| c) {
            Some('x') | Some('X') => Some(16u32),
            Some('b') | Some('B') => Some(2),
            Some('o') | Some('O') => Some(8),
            _ => None,
        };
        if let Some(radix) = radix {
            iter.next();
            let digits_start = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());
            while iter
                .peek()
                .is_some_and(|(_, c)| c.is_digit(radix) || *c == '_')
            {
                iter.next();
            }
            let end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());

            let has_digit = src[digits_start..end].chars().any(|c| c.is_digit(radix));
            if !has_digit {
                errors.push(Diag {
                    code: ErrorCode::RadixWithoutDigits.into(),
                    span: Span {
                        start: start as u32,
                        end: end as u32,
                    },
                    message: "Expected digits after radix prefix".to_string(),
                });
                return (TokenKind::Integer(&src[start..end]), end);
            }
            return finish_number(iter, src, start, end, false, errors);
        }
    }

    // --- Decimal integer part ---------------------------------------------
    while iter
        .peek()
        .is_some_and(|(_, c)| c.is_ascii_digit() || *c == '_')
    {
        iter.next();
    }

    // --- Fractional part ---------------------------------------------------
    let mut is_float = false;
    if let Some(&(dot_idx, '.')) = iter.peek() {
        let after_dot = src[dot_idx + 1..].chars().next();
        if after_dot.is_some_and(|c| c.is_ascii_digit()) {
            iter.next();
            while iter
                .peek()
                .is_some_and(|(_, c)| c.is_ascii_digit() || *c == '_')
            {
                iter.next();
            }
            is_float = true;
        }
    }

    // --- Exponent ----------------------------------------------------------
    if let Some(&(e_idx, 'e' | 'E')) = iter.peek() {
        let mut chars = src[e_idx..].chars();
        chars.next();
        let mut sign_seen = false;
        let mut next = chars.next();
        if next == Some('+') || next == Some('-') {
            sign_seen = true;
            next = chars.next();
        }
        if next.is_some_and(|c| c.is_ascii_digit()) {
            iter.next();
            if sign_seen {
                iter.next();
            }
            while iter
                .peek()
                .is_some_and(|(_, c)| c.is_ascii_digit() || *c == '_')
            {
                iter.next();
            }
            is_float = true;
        }
    }

    let end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());
    finish_number(iter, src, start, end, is_float, errors)
}

/// Shared tail for every numeric literal: consume an optional `u32`/`f64`/…
/// suffix, validate `_` separators, and build the token.
fn finish_number<'src>(
    iter: &mut std::iter::Peekable<std::str::CharIndices<'src>>,
    src: &'src str,
    start: usize,
    end: usize,
    is_float: bool,
    errors: &mut Vec<Diag>,
) -> (TokenKind<'src>, usize) {
    let end = consume_suffix(iter, src, end);
    let text = &src[start..end];
    validate_separators(text, start, errors);
    let kind = if is_float {
        TokenKind::Float(text)
    } else {
        TokenKind::Integer(text)
    };
    (kind, end)
}

/// Reject `1_` / `1__2`-style separators. `base` is the byte offset of `text`
/// in `src` so diagnostics point at the right column.
fn validate_separators(text: &str, base: usize, errors: &mut Vec<Diag>) {
    if text.ends_with('_') {
        let end = base + text.len();
        errors.push(Diag {
            code: ErrorCode::BadNumericSeparator.into(),
            span: Span {
                start: (end - 1) as u32,
                end: end as u32,
            },
            message: "Numeric literal cannot end with '_'".to_string(),
        });
    } else if let Some(rel) = text.find("__") {
        let abs = base + rel;
        errors.push(Diag {
            code: ErrorCode::BadNumericSeparator.into(),
            span: Span {
                start: abs as u32,
                end: (abs + 2) as u32,
            },
            message: "Numeric literal cannot contain consecutive '_'".to_string(),
        });
    }
}

fn check_for_keyword(text: &'_ str) -> TokenKind<'_> {
    match text {
        "fn" => TokenKind::Function,
        "let" => TokenKind::LetToken,
        "mut" => TokenKind::MutToken,
        "if" => TokenKind::IfToken,
        "else" => TokenKind::ElseToken,
        "match" => TokenKind::MatchToken,
        "for" => TokenKind::ForToken,
        "while" => TokenKind::WhileToken,
        "return" => TokenKind::ReturnToken,
        "defer" => TokenKind::Defer,
        "errdefer" => TokenKind::ErrDefer,
        "struct" => TokenKind::StructToken,
        "enum" => TokenKind::EnumToken,
        "trait" => TokenKind::TraitToken,
        "impl" => TokenKind::ImplToken,
        "pub" => TokenKind::PubToken,
        "unsafe" => TokenKind::UnsafeToken,
        "extern" => TokenKind::ExternToken,
        "true" => TokenKind::TrueToken,
        "false" => TokenKind::FalseToken,
        "loop" => TokenKind::LoopToken,
        "in" => TokenKind::InToken,
        "range" => TokenKind::RangeToken,
        "break" => TokenKind::BreakToken,
        "continue" => TokenKind::ContinueToken,
        "as" => TokenKind::AsToken,
        _ => TokenKind::Identifier(text),
    }
}

fn peek_char(iter: &mut std::iter::Peekable<std::str::CharIndices>) -> Option<char> {
    iter.peek().map(|(_, c)| *c)
}

/// Consume the next char if it equals `expected`. Returns true if consumed.
fn try_consume(iter: &mut std::iter::Peekable<std::str::CharIndices>, expected: char) -> bool {
    if peek_char(iter) == Some(expected) {
        iter.next();
        true
    } else {
        false
    }
}

/// Lex an operator whose kind is determined by an optional one-character
/// suffix. `single` is the kind when no listed suffix matches.
fn lex_operator<'src>(
    iter: &mut std::iter::Peekable<std::str::CharIndices<'src>>,
    src_len: usize,
    single: TokenKind<'src>,
    suffixes: &[(char, TokenKind<'src>)],
) -> (TokenKind<'src>, usize) {
    let kind = match iter.peek() {
        Some(&(_, c)) => match suffixes.iter().find(|(s, _)| *s == c) {
            Some(&(_, k)) => {
                iter.next();
                k
            }
            None => single,
        },
        None => single,
    };
    let end = iter.peek().map(|(i, _)| *i).unwrap_or(src_len);
    (kind, end)
}

/// Skip to end-of-line (or EOF). The initial `//` has already been consumed.
fn skip_line_comment(iter: &mut std::iter::Peekable<std::str::CharIndices>) {
    for (_, ch) in iter.by_ref() {
        if ch == '\n' {
            break;
        }
    }
}

/// Skip a `/* ... */` block comment, honoring nesting. The initial `/*`
/// has already been consumed. Returns `Err(span)` on EOF before the
/// matching `*/`.
fn skip_block_comment(
    iter: &mut std::iter::Peekable<std::str::CharIndices>,
    start: usize,
    src_len: usize,
) -> Result<(), Span> {
    let mut depth: usize = 1;
    let mut prev = '\0';
    for (_, ch) in iter.by_ref() {
        if prev == '*' && ch == '/' {
            depth -= 1;
            if depth == 0 {
                return Ok(());
            }
            prev = '\0'; // consume the pair, don't chain into `*/*`
        } else if prev == '/' && ch == '*' {
            depth += 1;
            prev = '\0';
        } else {
            prev = ch;
        }
    }
    Err(Span {
        start: start as u32,
        end: src_len as u32,
    })
}

/// If `src[end..]` begins with a known numeric suffix that ends on a word
/// boundary, return its byte length. The boundary check is what makes
/// `1u32x` lex as `1` + `u32x` rather than `1u32` + `x`.
fn match_suffix(src: &str, end: usize) -> Option<usize> {
    for suf in NUMERIC_SUFFIXES {
        if src[end..].starts_with(suf) {
            let after = end + suf.len();
            let boundary = src[after..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric() && c != '_');
            if boundary {
                return Some(suf.len());
            }
        }
    }
    None
}

/// Consume the suffix in `iter` and return the new end offset, or `end`
/// unchanged if no suffix follows. All suffix chars are ASCII, so byte
/// length equals char count.
fn consume_suffix(
    iter: &mut std::iter::Peekable<std::str::CharIndices>,
    src: &str,
    end: usize,
) -> usize {
    match match_suffix(src, end) {
        Some(len) => {
            for _ in 0..len {
                iter.next();
            }
            end + len
        }
        None => end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexes_fn_main() {
        let tokens = lex("fn main() { 1 + 2 }").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Function,
                TokenKind::Identifier("main"),
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::Integer("1"),
                TokenKind::Plus,
                TokenKind::Integer("2"),
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
        // Spot-check a span
        assert_eq!(tokens[1].span, Span { start: 3, end: 7 }); // "main"
    }

    #[test]
    fn lex_unknown_token_errors() {
        let mut result = lex("fn main() { ♥ }");
        assert!(result.is_err(), "expected lexing to fail on '♥'");

        let mut diags = result.unwrap_err();
        assert_eq!(
            diags.len(),
            1,
            "expected exactly one diagnostic, got {diags:?}"
        );

        let diag = &diags[0];
        assert_eq!(diag.span, Span { start: 12, end: 15 });
        assert_eq!(
            diags,
            vec![Diag {
                code: ErrorCode::UnexpectedToken.into(),
                span: Span { start: 12, end: 15 },
                message: "Unexpected token '♥'".to_string(),
            }]
        );

        result = lex("fn main() { ♥☻○ }");
        diags = result.unwrap_err();
        assert_eq!(
            diags.len(),
            3,
            "expected exactly 3 diagnostic, got {diags:?}"
        );
    }
    #[test]
    fn lexes_operators_without_whitespace() {
        let kinds: Vec<_> = lex("1+2").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1"),
                TokenKind::Plus,
                TokenKind::Integer("2"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_multi_char_operators() {
        let tokens = lex("== != <= >= && || << >>")
            .expect("lexer should accept all valid two-char operators");

        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::DoubleEquals,
                TokenKind::NotEquals,
                TokenKind::SmallerOrEquals,
                TokenKind::BiggerOrEquals,
                TokenKind::DoubleAnd,
                TokenKind::DoublePipe,
                TokenKind::LShift,
                TokenKind::RShift,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_shift_assign_disambiguation() {
        // `<<=` must not lex as `<<`, `=`
        let kinds: Vec<_> = lex("<<= >>= <<").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::LShiftEquals,
                TokenKind::RShiftEquals,
                TokenKind::LShift,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn span_of_two_char_operator() {
        let tokens = lex("==").unwrap();
        assert_eq!(tokens[0].span, Span { start: 0, end: 2 });
    }

    #[test]
    fn lexes_single_char_operator_at_eof() {
        let tokens = lex("+").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[0].span, Span { start: 0, end: 1 });
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }

    // ----- Wrapping ------------------------------------------------------------

    #[test]
    fn lexes_wrapping_operators() {
        let kinds: Vec<_> = lex("+% -% *% /% <<% >>%")
            .expect("wrapping operators should lex")
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::PlusWrapping,
                TokenKind::MinusWrapping,
                TokenKind::StarWrapping,
                TokenKind::SlashWrapping,
                TokenKind::LShiftWrapping,
                TokenKind::RShiftWrapping,
                TokenKind::Eof,
            ]
        );
    }

    // ----- Checked -------------------------------------------------------------

    #[test]
    fn lexes_checked_operators() {
        let kinds: Vec<_> = lex("+? -? *? /? <<? >>?")
            .expect("checked operators should lex")
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::PlusChecked,
                TokenKind::MinusChecked,
                TokenKind::StarChecked,
                TokenKind::SlashChecked,
                TokenKind::LShiftChecked,
                TokenKind::RShiftChecked,
                TokenKind::Eof,
            ]
        );
    }

    // ----- Saturating ----------------------------------------------------------

    #[test]
    fn lexes_saturating_operators() {
        let kinds: Vec<_> = lex("+| -| *|")
            .expect("saturating operators should lex")
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::PlusSaturating,
                TokenKind::MinusSaturating,
                TokenKind::StarSaturating,
                TokenKind::Eof,
            ]
        );
    }

    // ----- Disambiguation ------------------------------------------------------

    #[test]
    fn compound_and_wrapping_are_distinct() {
        // "+=" (compound assign) must not be confused with "+%" (wrapping).
        let kinds: Vec<_> = lex("+= +%").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::PlusEquals,
                TokenKind::PlusWrapping,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn saturating_vs_pipe_and_double_pipe() {
        // "+|" must not lex as "+", "|".
        // Standalone "|" and "||" must still lex correctly.
        let kinds: Vec<_> = lex("+| | ||").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::PlusSaturating,
                TokenKind::Pipe,
                TokenKind::DoublePipe,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn saturating_requires_adjacency() {
        // With a space: "+ |" is Plus then Pipe (bitwise-or).
        let kinds: Vec<_> = lex("a + | b").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("a"),
                TokenKind::Plus,
                TokenKind::Pipe,
                TokenKind::Identifier("b"),
                TokenKind::Eof,
            ]
        );

        // Without a space: "+|" is one token.
        let kinds: Vec<_> = lex("a+|b").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("a"),
                TokenKind::PlusSaturating,
                TokenKind::Identifier("b"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn checked_is_a_single_token() {
        // "a +? b" — "+?" must be one token.
        let tokens = lex("a+?b").unwrap();
        assert_eq!(tokens[1].kind, TokenKind::PlusChecked);
        // "+?" is 2 bytes at offset 1..3
        assert_eq!(tokens[1].span, Span { start: 1, end: 3 });
    }

    // ----- Spans ---------------------------------------------------------------

    #[test]
    fn spans_of_overflow_policy_operators() {
        for (src, offset, expected_kind, expected_len) in [
            ("a+%b", 1, TokenKind::PlusWrapping, 2),
            ("a+?b", 1, TokenKind::PlusChecked, 2),
            ("a+|b", 1, TokenKind::PlusSaturating, 2),
            ("a-%b", 1, TokenKind::MinusWrapping, 2),
            ("a-?b", 1, TokenKind::MinusChecked, 2),
            ("a-|b", 1, TokenKind::MinusSaturating, 2),
            ("a*%b", 1, TokenKind::StarWrapping, 2),
            ("a*?b", 1, TokenKind::StarChecked, 2),
            ("a*|b", 1, TokenKind::StarSaturating, 2),
            ("a/%b", 1, TokenKind::SlashWrapping, 2),
            ("a/?b", 1, TokenKind::SlashChecked, 2),
            ("a<<%b", 1, TokenKind::LShiftWrapping, 3),
            ("a<<?b", 1, TokenKind::LShiftChecked, 3),
            ("a>>%b", 1, TokenKind::RShiftWrapping, 3),
            ("a>>?b", 1, TokenKind::RShiftChecked, 3),
        ] {
            let tokens = lex(src).unwrap();
            assert_eq!(tokens[1].kind, expected_kind, "input: {src:?}");
            assert_eq!(
                tokens[1].span,
                Span {
                    start: offset,
                    end: offset + expected_len
                },
                "span mismatch for {src:?}"
            );
        }
    }

    // ----- Edge cases ----------------------------------------------------------

    #[test]
    fn overflow_policy_operators_at_eof() {
        for (src, expected_kind) in [
            ("+%", TokenKind::PlusWrapping),
            ("+?", TokenKind::PlusChecked),
            ("+|", TokenKind::PlusSaturating),
            ("<<%", TokenKind::LShiftWrapping),
            (">>?", TokenKind::RShiftChecked),
        ] {
            let tokens = lex(src).unwrap();
            assert_eq!(tokens[0].kind, expected_kind, "input: {src:?}");
            assert_eq!(
                tokens[0].span,
                Span {
                    start: 0,
                    end: src.len() as u32
                },
                "input: {src:?}"
            );
            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }
    }

    #[test]
    fn percent_compound_assignment_is_one_token() {
        // Regression: the original lexer passed Percent for both kinds,
        // silently swallowing the `=`.
        let kinds: Vec<_> = lex("%= %").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![TokenKind::PercentEquals, TokenKind::Percent, TokenKind::Eof,]
        );
    }

    // ----- End-to-end demo -----------------------------------------------------

    #[test]
    fn lexes_spec_overflow_examples() {
        // The three overflow-policy examples from Spec §11.2.
        let kinds: Vec<_> = lex("250 +% 10  250 +? 10  250 +| 10")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("250"),
                TokenKind::PlusWrapping,
                TokenKind::Integer("10"),
                TokenKind::Integer("250"),
                TokenKind::PlusChecked,
                TokenKind::Integer("10"),
                TokenKind::Integer("250"),
                TokenKind::PlusSaturating,
                TokenKind::Integer("10"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn dot_and_range() {
        let kinds: Vec<_> = lex(". .. ..= in range")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Dot,
                TokenKind::DoubleDot,
                TokenKind::RangeInclusive,
                TokenKind::InToken,
                TokenKind::RangeToken,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn integer_separators_are_preserved() {
        let kinds: Vec<_> = lex("1_000_000").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![TokenKind::Integer("1_000_000"), TokenKind::Eof]);
    }

    #[test]
    fn underscore_leading_identifiers_are_not_integers() {
        // A leading '_' must produce an Identifier, not an Integer.
        let kinds: Vec<_> = lex("_foo _ _1").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("_foo"),
                TokenKind::Identifier("_"),
                TokenKind::Identifier("_1"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn trailing_underscore_is_rejected() {
        // "1_" — a trailing separator is not valid in any reasonable grammar.
        let result = lex("1_");
        assert!(result.is_err(), "trailing '_' should be a diagnostic");
        let diags = result.unwrap_err();
        assert_eq!(diags[0].code, ErrorCode::BadNumericSeparator.into(),);
        assert_eq!(diags[0].span, Span { start: 1, end: 2 });
    }

    #[test]
    fn double_underscore_is_rejected() {
        let result = lex("1__2");
        assert!(result.is_err(), "double '_' should be a diagnostic");
        let diags = result.unwrap_err();
        assert_eq!(diags[0].code, ErrorCode::BadNumericSeparator.into(),);
    }

    #[test]
    fn lexes_simple_char() {
        let tokens = lex("'a'").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Char("a"));
        assert_eq!(tokens[0].span, Span { start: 0, end: 3 });
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }

    #[test]
    fn lexes_escaped_char() {
        let tokens = lex(r"'\n'").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Char(r"\n"));
        assert_eq!(tokens[0].span, Span { start: 0, end: 4 });
    }

    #[test]
    fn lexes_escaped_quote_char() {
        // '\'' — escape is \' , closing quote is the final '
        let tokens = lex(r"'\''").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Char(r"\'"));
        assert_eq!(tokens[0].span, Span { start: 0, end: 4 });
    }

    #[test]
    fn empty_char_is_an_error() {
        let diags = lex("''").unwrap_err();
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].code, ErrorCode::EmptyChar.into(),);
        assert_eq!(diags[0].span, Span { start: 0, end: 2 });
    }

    #[test]
    fn multi_char_is_an_error_and_recovers() {
        // 'ab' — should consume through the closing quote and produce one diag.
        let diags = lex("'ab' + 1").unwrap_err();
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].code, ErrorCode::MultiChar.into());
        assert_eq!(diags[0].span, Span { start: 0, end: 4 });
    }

    #[test]
    fn unterminated_char_is_an_error() {
        let diags = lex("'a").unwrap_err();
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].code, ErrorCode::MultiChar.into());
    }

    #[test]
    fn bare_quote_at_eof_is_an_error() {
        let diags = lex("'").unwrap_err();
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].code, ErrorCode::UnterminatedChar.into());
    }

    #[test]
    fn unterminated_escape_is_an_error() {
        let diags = lex(r"'\").unwrap_err();
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert_eq!(diags[0].code, ErrorCode::UnterminatedChar.into());
    }

    #[test]
    fn string_content_is_correct_with_multibyte_chars() {
        // é is 2 bytes; the slice must not land mid-codepoint.
        let tokens = lex("\"héllo\"").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::String("héllo"));
        assert_eq!(tokens[0].span, Span { start: 0, end: 8 });

        let tokens = lex("\"é\"").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::String("é"));
        assert_eq!(tokens[0].span, Span { start: 0, end: 4 });
    }

    #[test]
    fn string_span_includes_both_quotes() {
        let tokens = lex("\"abc\"").unwrap();
        assert_eq!(tokens[0].span, Span { start: 0, end: 5 });
        assert_eq!(tokens[0].kind, TokenKind::String("abc"));
    }

    #[test]
    fn lexes_arrows() {
        let kinds: Vec<_> = lex("-> =>").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![TokenKind::SmallArrow, TokenKind::BigArrow, TokenKind::Eof]
        );
    }

    #[test]
    fn arrow_spans() {
        let tokens = lex("->").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::SmallArrow);
        assert_eq!(tokens[0].span, Span { start: 0, end: 2 });

        let tokens = lex("=>").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::BigArrow);
        assert_eq!(tokens[0].span, Span { start: 0, end: 2 });
    }

    #[test]
    fn arrow_is_not_minus_then_greater() {
        // Regression: without the '>' suffix on '-', this used to lex as
        // [Minus, BiggerThen].
        let kinds: Vec<_> = lex("->").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![TokenKind::SmallArrow, TokenKind::Eof],
            "-> must be one token, not Minus + BiggerThen"
        );
    }

    #[test]
    fn fat_arrow_is_not_equals_then_greater() {
        let kinds: Vec<_> = lex("=>").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![TokenKind::BigArrow, TokenKind::Eof],
            "=> must be one token, not Equals + BiggerThen"
        );
    }

    #[test]
    fn minus_arrow_does_not_break_arithmetic() {
        // The '-' arm now has five suffixes; make sure none shadow each other.
        let kinds: Vec<_> = lex("-= -% -? -| ->")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::MinusEquals,
                TokenKind::MinusWrapping,
                TokenKind::MinusChecked,
                TokenKind::MinusSaturating,
                TokenKind::SmallArrow,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn equals_arrow_does_not_break_equality() {
        let kinds: Vec<_> = lex("= == =>").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Equals,
                TokenKind::DoubleEquals,
                TokenKind::BigArrow,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn fn_signature_with_return_arrow() {
        // End-to-end: the exact shape every parser test will need.
        let kinds: Vec<_> = lex("fn f() -> i32")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Function,
                TokenKind::Identifier("f"),
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::SmallArrow,
                TokenKind::Identifier("i32"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn match_arm_with_fat_arrow() {
        let kinds: Vec<_> = lex("1 => 2").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1"),
                TokenKind::BigArrow,
                TokenKind::Integer("2"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_floats() {
        let kinds: Vec<_> = lex("3.14 1e10 1.5e-3 2E+8 0.0")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Float("3.14"),
                TokenKind::Float("1e10"),
                TokenKind::Float("1.5e-3"),
                TokenKind::Float("2E+8"),
                TokenKind::Float("0.0"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn range_is_not_a_float() {
        let kinds: Vec<_> = lex("1..2").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1"),
                TokenKind::DoubleDot,
                TokenKind::Integer("2"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn float_then_dot_recovers() {
        let kinds: Vec<_> = lex("1.5.6").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Float("1.5"),
                TokenKind::Dot,
                TokenKind::Integer("6"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn radix_literals() {
        let kinds: Vec<_> = lex("0xFF 0b1010 0o777 0xFF_FF")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("0xFF"),
                TokenKind::Integer("0b1010"),
                TokenKind::Integer("0o777"),
                TokenKind::Integer("0xFF_FF"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn radix_without_digits_is_an_error() {
        let diags = lex("0x").unwrap_err();
        assert_eq!(diags[0].code, ErrorCode::RadixWithoutDigits.into());
    }

    // ----- Comments ----------------------------------------------------------

    #[test]
    fn line_comment_skips_to_eol() {
        let kinds: Vec<_> = lex("1 // c\n2").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1"),
                TokenKind::Integer("2"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn line_comment_at_eof() {
        let kinds: Vec<_> = lex("1 // x").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![TokenKind::Integer("1"), TokenKind::Eof]);
    }

    #[test]
    fn block_comment_inline() {
        let kinds: Vec<_> = lex("1 /* x */ 2").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1"),
                TokenKind::Integer("2"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn nested_block_comment() {
        let kinds: Vec<_> = lex("1 /* a /* b */ c */ 2")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1"),
                TokenKind::Integer("2"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn adjacent_block_comments() {
        let kinds: Vec<_> = lex("/*a*//*b*/1").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![TokenKind::Integer("1"), TokenKind::Eof]);
    }

    #[test]
    fn unterminated_block_comment_is_error() {
        let diags = lex("1 /* never ends").unwrap_err();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, ErrorCode::UnterminatedBlockComment.into());
        assert_eq!(diags[0].span, Span { start: 2, end: 15 });
    }

    #[test]
    fn division_still_works_next_to_comments() {
        // `/` not followed by `/` or `*` must still be a Slash token.
        let kinds: Vec<_> = lex("a / b // c").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("a"),
                TokenKind::Slash,
                TokenKind::Identifier("b"),
                TokenKind::Eof,
            ]
        );
    }

    // ----- Numeric suffixes --------------------------------------------------

    #[test]
    fn integer_suffixes() {
        let kinds: Vec<_> = lex("1u32 42i64 0xFFu8 0b1010usize")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1u32"),
                TokenKind::Integer("42i64"),
                TokenKind::Integer("0xFFu8"),
                TokenKind::Integer("0b1010usize"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn float_suffixes() {
        let kinds: Vec<_> = lex("1.5f32 3.14f64 1e10f32 2E+8f64")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Float("1.5f32"),
                TokenKind::Float("3.14f64"),
                TokenKind::Float("1e10f32"),
                TokenKind::Float("2E+8f64"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn suffix_separated_by_underscore() {
        let kinds: Vec<_> = lex("1_u32 1_000_u32")
            .unwrap()
            .iter()
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1_u32"),
                TokenKind::Integer("1_000_u32"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn incomplete_suffix_is_not_absorbed() {
        // `u32x` is not a suffix — number must end at `1`, ident starts at `u`.
        let kinds: Vec<_> = lex("1u32x").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1"),
                TokenKind::Identifier("u32x"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn unknown_suffix_is_not_absorbed() {
        let kinds: Vec<_> = lex("1foo").unwrap().iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer("1"),
                TokenKind::Identifier("foo"),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn trailing_underscore_still_errors_with_suffix_following() {
        // `1__u32` — the `__` is caught before the suffix is considered.
        let diags = lex("1__u32").unwrap_err();
        assert_eq!(diags[0].code, ErrorCode::BadNumericSeparator.into());
    }

    #[test]
    fn empty_radix_is_still_an_error_even_with_underscore() {
        // Regression: `0x_` used to slip past the "no digits" check because
        // the span was non-empty (it contained the `_`).
        let diags = lex("0x_").unwrap_err();
        assert_eq!(diags[0].code, ErrorCode::RadixWithoutDigits.into());
    }
}
