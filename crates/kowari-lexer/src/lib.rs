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
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LParen,
    RParen,
    LBrace,
    RBrace,

    // Words
    Function,
    Identifier(&'src str),

    // Literals
    Integer(&'src str),

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
            '+' => Some((TokenKind::Plus, start + c.len_utf8())),
            '-' => Some((TokenKind::Minus, start + c.len_utf8())),
            '*' => Some((TokenKind::Star, start + c.len_utf8())),
            '/' => Some((TokenKind::Slash, start + c.len_utf8())),
            '%' => Some((TokenKind::Percent, start + c.len_utf8())),
            '(' => Some((TokenKind::LParen, start + c.len_utf8())),
            ')' => Some((TokenKind::RParen, start + c.len_utf8())),
            '{' => Some((TokenKind::LBrace, start + c.len_utf8())),
            '}' => Some((TokenKind::RBrace, start + c.len_utf8())),
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
        return Err(errors)
    }

    Ok(tokens)
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
