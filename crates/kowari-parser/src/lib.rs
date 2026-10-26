mod errors;
mod tests;

use crate::errors::ParseError;
use kowari_ast::item::{Block, Function, Item, SourceFile};
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

    // internal: structured, cheap
    fn parse_fn_item(&mut self) -> Result<Function<'src>, ParseError<'src>> {
        self.expect(TokenKind::Function)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;
        self.expect(TokenKind::RParen)?;
        let body = self.parse_block()?;

        todo!()
    }

    fn parse_source_file(&mut self) -> Result<SourceFile<'src>, ParseError<'src>> {
        unimplemented!("step 2: loop {{ items.push(self.parse_item()?) }}")
    }

    fn parse_item(&mut self) -> Result<Item<'src>, ParseError<'src>> {
        unimplemented!("step 2: dispatch on TokenKind::Fn")
    }

    fn parse_block(&mut self) -> Result<Block<'src>, ParseError<'src>> {
        unimplemented!("step 2: expect LeftBrace, then RightBrace")
    }

    fn expect_ident(&mut self) -> Result<&'src str, ParseError<'src>> {
        unimplemented!("step 2: consume TokenKind::Ident")
    }
}
