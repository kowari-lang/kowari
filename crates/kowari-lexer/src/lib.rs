//! Lexer: source text → tokens.
mod tests;
pub mod token;

use kowari_diagnostics::{Diag, ErrorCode, Span};
use std::iter::Peekable;
use std::str::CharIndices;
pub use token::{NUMERIC_SUFFIXES, Token, TokenKind};

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
                &[
                    ('&', TokenKind::DoubleAnd),
                    ('=', TokenKind::BitwiseAndEquals),
                ],
            )),
            '|' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Pipe,
                &[
                    ('|', TokenKind::DoublePipe),
                    ('=', TokenKind::BitwiseOrEquals),
                ],
            )),
            ':' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Colon,
                &[(':', TokenKind::DoubleColon)],
            )),
            '^' => Some(lex_operator(
                &mut iter,
                src.len(),
                TokenKind::Caret,
                &[('=', TokenKind::BitwiseXorEquals)],
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
                        TokenKind::LessThan,
                        &[('=', TokenKind::LessOrEquals)],
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
                        TokenKind::GreaterThan,
                        &[('=', TokenKind::GreaterOrEquals)],
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
            ';' => Some((TokenKind::Semicolon, start + c.len_utf8())),
            '#' => Some((TokenKind::HashTag, start + 1)),
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
                        span: Span { start, end },
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
                            start,
                            end: start + 1,
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
                            start,
                            end: start + 2,
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
                            start,
                            end: src.len(),
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
                    let mut found_closing_quote = false;
                    while let Some((i, ch)) = iter.peek().copied() {
                        if ch == '\n' || ch == '\r' {
                            recovered_end = i;
                            break;
                        }
                        if ch == '\'' {
                            iter.next();
                            recovered_end = i + 1;
                            found_closing_quote = true;
                            break;
                        }
                        iter.next();
                        recovered_end = i + ch.len_utf8();
                    }

                    let (code, message) = if found_closing_quote {
                        (
                            ErrorCode::MultiChar,
                            "Char literal must contain exactly one character",
                        )
                    } else {
                        (ErrorCode::UnterminatedChar, "Unterminated char literal")
                    };

                    errors.push(Diag {
                        code: code.into(),
                        span: Span {
                            start,
                            end: recovered_end,
                        },
                        message: message.to_string(),
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
                        start,
                        end: start + c.len_utf8(),
                    },
                    message: format!("Unexpected token '{c}'"),
                });
                None
            }
        };
        if let Some((kind, end)) = opt {
            tokens.push(Token {
                kind,
                span: Span { start, end },
            })
        }
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span {
            start: src.len(),
            end: src.len(),
        },
    });

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(tokens)
}

fn lex_number<'src>(
    iter: &mut Peekable<CharIndices<'src>>,
    src: &'src str,
    start: usize,
    errors: &mut Vec<Diag>,
) -> (TokenKind<'src>, usize) {
    // --- Radix-prefixed integers ------------------------------------------
    if src.as_bytes()[start] == b'0' {
        if let Some(result) = parse_radix_integer(iter, src, start, errors) {
            return result;
        }
    }

    // --- Decimal integer part ---------------------------------------------
    parse_decimal_integer(iter);

    // --- Fractional part ---------------------------------------------------
    let mut is_float = parse_fraction(iter, src);

    // --- Exponent ----------------------------------------------------------
    is_float |= parse_exponent(iter, src);

    let end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());
    finish_number(iter, src, start, end, is_float, errors)
}

fn parse_decimal_integer(iter: &mut Peekable<CharIndices>) {
    while iter
        .peek()
        .is_some_and(|(_, c)| c.is_ascii_digit() || *c == '_')
    {
        iter.next();
    }
}

fn parse_fraction(iter: &mut Peekable<CharIndices>, src: &str) -> bool {
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
    is_float
}

fn parse_exponent(iter: &mut Peekable<CharIndices>, src: &str) -> bool {
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
            return true;
        }
    }
    false
}

fn parse_radix_integer<'src>(
    iter: &mut Peekable<CharIndices<'src>>,
    src: &'src str,
    start: usize,
    errors: &mut Vec<Diag>,
) -> Option<(TokenKind<'src>, usize)> {
    let radix = match iter.peek().map(|&(_, c)| c) {
        Some('x') | Some('X') => 16u32,
        Some('b') | Some('B') => 2,
        Some('o') | Some('O') => 8,
        _ => return None,
    };
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
            span: Span { start, end },
            message: "Expected digits after radix prefix".to_string(),
        });
        return Some((TokenKind::Integer(&src[start..end]), end));
    }

    Some(finish_number(iter, src, start, end, false, errors))
}

/// Shared tail for every numeric literal: consume an optional `u32`/`f64`/…
/// suffix, validate `_` separators, and build the token.
fn finish_number<'src>(
    iter: &mut Peekable<CharIndices<'src>>,
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
                start: end - 1,
                end,
            },
            message: "Numeric literal cannot end with '_'".to_string(),
        });
    } else if let Some(rel) = text.find("__") {
        let abs = base + rel;
        errors.push(Diag {
            code: ErrorCode::BadNumericSeparator.into(),
            span: Span {
                start: abs,
                end: abs + 2,
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
        "crate" => TokenKind::CrateToken,
        "use" => TokenKind::UseToken,
        "self" => TokenKind::SelfValueToken,
        "Self" => TokenKind::SelfTypeToken,
        "const" => TokenKind::ConstToken,
        "static" => TokenKind::StaticToken,
        "type" => TokenKind::TypeDefToken,
        "where" => TokenKind::WhereToken,
        "mod" => TokenKind::ModuleToken,
        "dyn" => TokenKind::DynToken,
        "union" => TokenKind::UnionToken,
        _ => TokenKind::Identifier(text),
    }
}

fn peek_char(iter: &mut Peekable<CharIndices>) -> Option<char> {
    iter.peek().map(|(_, c)| *c)
}

/// Consume the next char if it equals `expected`. Returns true if consumed.
fn try_consume(iter: &mut Peekable<CharIndices>, expected: char) -> bool {
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
    iter: &mut Peekable<CharIndices<'src>>,
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
fn skip_line_comment(iter: &mut Peekable<CharIndices>) {
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
    iter: &mut Peekable<CharIndices>,
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
        start,
        end: src_len,
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
fn consume_suffix(iter: &mut Peekable<CharIndices>, src: &str, end: usize) -> usize {
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

pub fn dump_tokens(src: &str) -> Result<String, Vec<Diag>> {
    let tokens = lex(src)?;
    let mut out = String::new();
    for tok in tokens {
        use std::fmt::Write;
        writeln!(out, "{tok}").unwrap();
    }
    Ok(out)
}
