//! Lexer: source text → tokens.
use kowari_diagnostics::{Diag, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'src> {
    kind: TokenKind<'src>,
    span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind<'src> {
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

pub fn lex<'src>(src: &'src str) -> Result<Vec<Token>, Vec<Diag>> {
    let tokens: Vec<Token> = Vec::new();

    for char in src.chars() {

    }
    unimplemented!("lexer not implemented yet")
}

#[test]
fn lexes_fn_main() {
    let tokens = lex("fn main() { 1 + 2 }").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![
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
    ]);
    // Spot-check a span
    assert_eq!(tokens[1].span, Span { start: 3, end: 7 }); // "main"
}