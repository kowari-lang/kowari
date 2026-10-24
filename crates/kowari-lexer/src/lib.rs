//! Lexer: source text → tokens.

use kowari_diagnostics::{Diag, DiagCode, Span};

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
    DoubleSlash,        // //

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

pub fn lex(src: &'_ str) -> Result<Vec<Token<'_>>, Vec<Diag>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut errors = Vec::new();
    let mut iter = src.char_indices().peekable();

    while let Some((start, c)) = iter.next() {
        if c.is_whitespace() {
            continue;
        }

        let opt: Option<(TokenKind, usize)> = match c {
            '+' => Some(lex_arith(
                &mut iter,
                src.len(),
                TokenKind::Plus,
                TokenKind::PlusEquals,
                TokenKind::PlusWrapping,
                TokenKind::PlusChecked,
                Some(TokenKind::PlusSaturating),
            )),
            '-' => Some(lex_arith(
                &mut iter,
                src.len(),
                TokenKind::Minus,
                TokenKind::MinusEquals,
                TokenKind::MinusWrapping,
                TokenKind::MinusChecked,
                Some(TokenKind::MinusSaturating),
            )),
            '*' => Some(lex_arith(
                &mut iter,
                src.len(),
                TokenKind::Star,
                TokenKind::StarEquals,
                TokenKind::StarWrapping,
                TokenKind::StarChecked,
                Some(TokenKind::StarSaturating),
            )),
            '/' => Some(lex_arith(
                &mut iter,
                src.len(),
                TokenKind::Slash,
                TokenKind::SlashEquals,
                TokenKind::SlashWrapping,
                TokenKind::SlashChecked,
                None,
            )),
            '%' => Some(lex_optional_suffix(
                &mut iter,
                src.len(),
                '=',
                TokenKind::Percent,
                TokenKind::PercentEquals,
            )),
            '!' => Some(lex_optional_suffix(
                &mut iter,
                src.len(),
                '=',
                TokenKind::Exclamation,
                TokenKind::NotEquals,
            )),
            '(' => Some((TokenKind::LParen, start + c.len_utf8())),
            ')' => Some((TokenKind::RParen, start + c.len_utf8())),
            '{' => Some((TokenKind::LBrace, start + c.len_utf8())),
            '}' => Some((TokenKind::RBrace, start + c.len_utf8())),
            '[' => Some((TokenKind::LSquareBracket, start + c.len_utf8())),
            ']' => Some((TokenKind::RSquareBracket, start + c.len_utf8())),
            ';' => Some((TokenKind::Semicolon, start + c.len_utf8())),
            '=' => Some(lex_optional_suffix(
                &mut iter,
                src.len(),
                '=',
                TokenKind::Equals,
                TokenKind::DoubleEquals,
            )),
            '<' => {
                if try_consume(&mut iter, '<') {
                    Some(lex_arith(
                        &mut iter,
                        src.len(),
                        TokenKind::LShift,
                        TokenKind::LShiftEquals,
                        TokenKind::LShiftWrapping,
                        TokenKind::LShiftChecked,
                        None,
                    ))
                } else if try_consume(&mut iter, '=') {
                    Some((TokenKind::SmallerOrEquals, start + 2))
                } else {
                    Some((TokenKind::SmallerThen, start + 1))
                }
            }
            '>' => {
                if try_consume(&mut iter, '>') {
                    Some(lex_arith(
                        &mut iter,
                        src.len(),
                        TokenKind::RShift,
                        TokenKind::RShiftEquals,
                        TokenKind::RShiftWrapping,
                        TokenKind::RShiftChecked,
                        None,
                    ))
                } else if try_consume(&mut iter, '=') {
                    Some((TokenKind::BiggerOrEquals, start + 2))
                } else {
                    Some((TokenKind::BiggerThen, start + 1))
                }
            }
            '&' => Some(lex_optional_suffix(
                &mut iter,
                src.len(),
                '&',
                TokenKind::Ampersand,
                TokenKind::DoubleAnd,
            )),
            '|' => Some(lex_optional_suffix(
                &mut iter,
                src.len(),
                '|',
                TokenKind::Pipe,
                TokenKind::DoublePipe,
            )),
            '~' => Some((TokenKind::Tilde, start + c.len_utf8())),
            '.' => {
                if try_consume(&mut iter, '.') {
                    if try_consume(&mut iter, '=') {
                        Some((TokenKind::RangeInclusive, start + 2))
                    } else {
                        Some((TokenKind::DoubleDot, start + 2))
                    }
                } else {
                    Some((TokenKind::Dot, start + 1))
                }
            },
            c if c.is_ascii_digit() => {
                // Inner loop: consume trailing digits.
                while iter.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
                    iter.next();
                }

                let end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());
                Some((TokenKind::Integer(&src[start..end]), end))
            }
            c if c.is_alphabetic() || c == '_' => {
                while iter
                    .peek()
                    .is_some_and(|(_, c)| c.is_alphanumeric() || *c == '_')
                {
                    iter.next();
                }

                let end = iter.peek().map(|(i, _)| *i).unwrap_or(src.len());
                let text = &src[start..end];
                let kind = match text {
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
                    _ => TokenKind::Identifier(text),
                };
                Some((kind, end))
            }
            _ => {
                errors.push(Diag {
                    code: DiagCode("E0001"),
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

/// Lex a punctuator that may be one or two characters (e.g. `+` or `+=`).
/// Consumes the second character only if it matches `suffix`, then reports
/// whichever token applies along with the byte offset just past it.
fn lex_optional_suffix<'src>(
    iter: &mut std::iter::Peekable<std::str::CharIndices<'src>>,
    src_len: usize,
    suffix: char,
    single: TokenKind<'src>,
    combined: TokenKind<'src>,
) -> (TokenKind<'src>, usize) {
    let kind = if try_consume(iter, suffix) {
        combined
    } else {
        single
    };
    // Peek *after* consuming: `end` must sit past the suffix when present.
    let end = iter.peek().map(|(i, _)| *i).unwrap_or(src_len);
    (kind, end)
}

fn lex_arith<'src>(
    iter: &mut std::iter::Peekable<std::str::CharIndices<'src>>,
    src_len: usize,
    single: TokenKind<'src>,
    eq: TokenKind<'src>,
    wrapping: TokenKind<'src>,
    checked: TokenKind<'src>,
    saturating: Option<TokenKind<'src>>,
) -> (TokenKind<'src>, usize) {
    let kind = match peek_char(iter) {
        Some('=') => {
            iter.next();
            eq
        }
        Some('%') => {
            iter.next();
            wrapping
        }
        Some('?') => {
            iter.next();
            checked
        }
        Some('|') if saturating.is_some() => {
            iter.next();
            saturating.unwrap()
        }
        _ => single,
    };
    let end = iter.peek().map(|(i, _)| *i).unwrap_or(src_len);
    (kind, end)
}

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
    let mut result = lex("fn main() { @ }");
    assert!(result.is_err(), "expected lexing to fail on '@'");

    let mut diags = result.unwrap_err();
    assert_eq!(
        diags.len(),
        1,
        "expected exactly one diagnostic, got {diags:?}"
    );

    let diag = &diags[0];
    assert_eq!(diag.span, Span { start: 12, end: 13 }); // '@' is at byte 12
    assert_eq!(
        diags,
        vec![Diag {
            code: DiagCode("E0001"),
            span: Span { start: 12, end: 13 },
            message: "Unexpected token '@'".to_string(),
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
    let tokens =
        lex("== != <= >= && || << >>").expect("lexer should accept all valid two-char operators");

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