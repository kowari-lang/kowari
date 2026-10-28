use kowari_ast::item::SourceFile;
use kowari_diagnostics::{Diag, Span};
use kowari_lexer::{Token, TokenKind};

use crate::errors::ParseError;

pub struct Parser<'src> {
    tokens: &'src [Token<'src>],
    pub(crate) pos: usize,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: &'src [Token<'src>]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_file(&mut self) -> Result<SourceFile<'src>, ParseError<'src>> {
        let mut items = Vec::new();
        let start = self.peek().map(|t| t.span.start).unwrap_or(0);
        while !matches!(self.peek_kind(), Some(TokenKind::Eof) | None) {
            items.push(self.parse_item()?);
        }
        let end = self.eof_span().end;
        Ok(SourceFile {
            items,
            span: Span::new(start, end),
        })
    }

    pub fn parse(&mut self) -> Result<SourceFile<'src>, Vec<Diag>> {
        self.parse_file().map_err(|e| vec![e.into()])
    }

    pub(crate) fn peek(&self) -> Option<&Token<'src>> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn peek_kind(&self) -> Option<TokenKind<'src>> {
        self.peek().map(|t| t.kind)
    }

    fn _peek_at_n(&self, n: usize) -> Option<&'src Token<'src>> {
        self.tokens.get(n)
    }

    // Look ahead N tokens without consuming (useful for disambiguation).
    pub fn peek_nth(&self, n: usize) -> Option<&'src Token<'src>> {
        self._peek_at_n(self.pos + n)
    }

    pub(crate) fn check(&self, kind: TokenKind<'src>) -> bool {
        self.peek().is_some_and(|t| t.kind == kind)
    }

    pub(crate) fn advance(&mut self) -> Option<&'src Token<'src>> {
        let tok = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(tok)
    }

    pub(crate) fn expect(
        &mut self,
        kind: TokenKind<'src>,
    ) -> Result<&'src Token<'src>, ParseError<'src>> {
        if self.check(kind) {
            Ok(self.advance().unwrap())
        } else {
            match self.peek() {
                Some(t) => Err(ParseError::UnexpectedToken {
                    expected: kind,
                    got: t.kind,
                    span: t.span,
                }),
                None => Err(ParseError::UnexpectedEof {
                    expected: kind,
                    span: self.eof_span(),
                }),
            }
        }
    }

    pub(crate) fn current_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else {
            // EOF: zero-width span at the end of the source
            Span::at(self.tokens.last().map(|t| t.span.end).unwrap_or(0))
        }
    }
    pub(crate) fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::at(0)
        }
    }

    pub(crate) fn eof_span(&self) -> Span {
        let end = self.tokens.last().map_or(0, |t| t.span.end);
        Span::new(end, end)
    }

    pub(crate) fn expect_ident(&mut self) -> Result<(&'src str, Span), ParseError<'src>> {
        match self.advance() {
            Some(token) => match token.kind {
                TokenKind::Identifier(name) => Ok((name, self.prev_span())),
                found => Err(ParseError::IdentifierExpected {
                    found,
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
