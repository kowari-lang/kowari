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

    // Multi-Operators
    LShift,          // <<
    RShift,          // >>
    LShiftEquals,    // <<=
    RShiftEquals,    // >>=
    Equals,          // =
    DoubleEquals,    // ==
    NotEquals,       // !=
    BiggerOrEquals,  // >=
    SmallerOrEquals, // <=
    DoubleAnd,       // &&
    DoublePipe,      // ||
    PlusEquals,      // +=
    MinusEquals,     // -=
    StarEquals,      // *=
    SlashEquals,     // /=
    PercentEquals,   // %=

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

    // Literals
    Integer(&'src str),
    Float(&'src str),
    String(&'src str),
    Str(&'src str),
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
            '+' => Some(lex_optional_suffix(&mut iter, src.len(), '=', TokenKind::Plus, TokenKind::PlusEquals)),
            '-' => Some(lex_optional_suffix(&mut iter, src.len(), '=', TokenKind::Minus, TokenKind::MinusEquals)),
            '*' => Some(lex_optional_suffix(&mut iter, src.len(), '=', TokenKind::Star, TokenKind::StarEquals)),
            '/' => Some(lex_optional_suffix(&mut iter, src.len(), '=', TokenKind::Slash, TokenKind::SlashEquals)),
            '%' => Some(lex_optional_suffix(&mut iter, src.len(), '=', TokenKind::Percent, TokenKind::PercentEquals)),
            '!' => Some(lex_optional_suffix(&mut iter, src.len(), '=', TokenKind::Exclamation, TokenKind::NotEquals)),
            '(' => Some((TokenKind::LParen, start + c.len_utf8())),
            ')' => Some((TokenKind::RParen, start + c.len_utf8())),
            '{' => Some((TokenKind::LBrace, start + c.len_utf8())),
            '}' => Some((TokenKind::RBrace, start + c.len_utf8())),
            '=' => Some(lex_optional_suffix(&mut iter, src.len(), '=', TokenKind::Equals, TokenKind::DoubleEquals)),
            '<' => {
                if try_consume(&mut iter, '<') {
                    // We've got `<<`; is it `<<=`?
                    if try_consume(&mut iter, '=') {
                        Some((TokenKind::LShiftEquals, start + 3))
                    } else {
                        Some((TokenKind::LShift, start + 2))
                    }
                } else if try_consume(&mut iter, '=') {
                    Some((TokenKind::SmallerOrEquals, start + 2))
                } else {
                    Some((TokenKind::SmallerThen, start + 1))
                }
            },
            '>' => {
                if try_consume(&mut iter, '>') {
                    if try_consume(&mut iter, '=') {
                        Some((TokenKind::RShiftEquals, start + 3))
                    } else {
                        Some((TokenKind::RShift, start + 2))
                    }
                } else if try_consume(&mut iter, '=') {
                    Some((TokenKind::BiggerOrEquals, start + 2))
                } else {
                    Some((TokenKind::BiggerThen, start + 1))
                }
            },
            '&' => Some(lex_optional_suffix(&mut iter, src.len(), '&', TokenKind::Ampersand, TokenKind::DoubleAnd)),
            '|' => Some(lex_optional_suffix(&mut iter, src.len(), '|', TokenKind::Pipe, TokenKind::DoublePipe)),
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
    let kind = if try_consume(iter, suffix) { combined } else { single };
    // Peek *after* consuming: `end` must sit past the suffix when present.
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

    result = lex("fn main() { @~♥ }");
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