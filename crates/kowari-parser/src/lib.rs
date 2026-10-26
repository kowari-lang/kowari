mod errors;
#[cfg(test)]
mod tests;

use crate::errors::ParseError;
use kowari_ast::item::Function;
use kowari_ast::item::{Block, Item, SourceFile};
use kowari_diagnostics::{Diag, Span};
use kowari_lexer::{Token, TokenKind};

pub struct Parser<'src> {
    tokens: &'src [Token<'src>], // the full token stream from the lexer
    pos: usize,                  // index of the NEXT token to look at,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: &'src [Token<'src>]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<SourceFile<'src>, Vec<Diag>> {
        match self.parse_source_file() {
            // -> Result<_, ParseError>
            Ok(file) => Ok(file),
            Err(e) => Err(vec![e.into()]), // From<ParseError> for Diag
        }
    }

    // Look at the next token WITHOUT consuming it.
    pub fn peek(&self) -> Option<&'src Token<'src>> {
        self._peek_at_n(self.pos)
    }

    fn _peek_at_n(&self, n: usize) -> Option<&'src Token<'src>> {
        self.tokens.get(n)
    }

    // Look ahead N tokens without consuming (useful for disambiguation).
    pub fn peek_nth(&self, n: usize) -> Option<&'src Token<'src>> {
        self._peek_at_n(self.pos + n)
    }

    // Consume and return the next token.
    pub fn advance(&mut self) -> Option<&'src Token<'src>> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    // Consume the next token, erroring if it's not what we expected.
    fn expect(&mut self, expected: TokenKind<'src>) -> Result<Span, ParseError<'src>> {
        match self.advance() {
            Some(token) if token.kind == expected => Ok(token.span),
            Some(token) => Err(ParseError::UnexpectedToken {
                expected,
                got: token.kind,
                span: token.span,
            }),
            None => Err(ParseError::UnexpectedEof {
                expected,
                span: self.eof_span(),
            }),
        }
    }

    fn eof_span(&self) -> Span {
        let end = self.tokens.last().map_or(0, |t| t.span.end);
        Span::new(end, end)
    }
    fn parse_fn_item(&mut self) -> Result<Function<'src>, ParseError<'src>> {
        let start = self.expect(TokenKind::Function)?.start; // expect returns Span
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;
        self.expect(TokenKind::RParen)?;
        let body = self.parse_block()?;
        let span = Span::new(start, body.span.end);
        Ok(Function {
            name,
            params: Vec::new(), // no params yet
            ret: None,          // no return type yet
            body,
            span,
        })
    }

    fn parse_source_file(&mut self) -> Result<SourceFile<'src>, ParseError<'src>> {
        let mut items = Vec::new();
        let start = self.peek().map(|t| t.span.start).unwrap_or(0);
        while !matches!(
            self.peek().map(|token| token.kind),
            Some(TokenKind::Eof) | None
        ) {
            items.push(self.parse_item()?);
        }
        let end = self.eof_span().end;

        Ok(SourceFile {
            items,
            span: Span::new(start, end),
        })
    }

    fn parse_item(&mut self) -> Result<Item<'src>, ParseError<'src>> {
        match self.peek().map(|t| t.kind) {
            Some(TokenKind::Function) => self.parse_fn_item().map(Item::Function),
            Some(other) => Err(ParseError::UnexpectedToken {
                expected: TokenKind::Function,
                got: other,
                span: self.peek().unwrap().span,
            }),
            None => Err(ParseError::UnexpectedEof {
                expected: TokenKind::Function,
                span: self.eof_span(),
            }),
        }
    }

    fn parse_block(&mut self) -> Result<Block<'src>, ParseError<'src>> {
        let start = self.expect(TokenKind::LBrace)?.start;
        // TODO(step 3): loop { parse_stmt() } until RBrace
        let end = self.expect(TokenKind::RBrace)?.end;
        Ok(Block {
            stmts: Vec::new(),
            span: Span::new(start, end),
        })
    }

    fn expect_ident(&mut self) -> Result<&'src str, ParseError<'src>> {
        match self.advance() {
            Some(token) => match token.kind {
                TokenKind::Identifier(name) => Ok(name),
                got => Err(ParseError::UnexpectedToken {
                    expected: TokenKind::Identifier(""),
                    got,
                    span: token.span,
                }),
            },
            None => Err(ParseError::UnexpectedEof {
                expected: TokenKind::Identifier(""),
                span: self.eof_span(),
            }),
        }
    }
}
