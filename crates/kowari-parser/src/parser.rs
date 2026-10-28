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

    pub fn parse(&mut self) -> Result<SourceFile<'src>, Vec<Diag>> {
        match self.parse_source_file() {
            // -> Result<_, ParseError>
            Ok(file) => Ok(file),
            Err(e) => Err(vec![e.into()]), // From<ParseError> for Diag
        }
    }

    pub(crate) fn peek(&self) -> Option<&Token<'src>> {
        self.tokens.get(self.pos)
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

    pub(crate) fn expect(&mut self, kind: TokenKind<'src>) -> Result<&'src Token<'src>, ParseError<'src>> {
        if self.check(kind) {
            Ok(self.advance().unwrap())
        } else {
            match self.peek() {
                Some(t) => Err(ParseError::UnexpectedToken { expected: kind, got: t.kind, span: t.span }),
                None => Err(ParseError::UnexpectedEof { expected: kind, span: self.eof_span() }),
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

    pub fn parse_file(&mut self) -> Result<SourceFile<'src>, ParseError<'src>> {
        let mut items = Vec::new();
        while self.peek().is_some() {
            items.push(self.parse_item()?);
        }
        Ok(SourceFile { items, span: Default::default() })
    }
}