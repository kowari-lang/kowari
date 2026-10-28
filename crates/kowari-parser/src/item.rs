use crate::errors::ParseError;
use crate::parser::Parser;
use kowari_ast::item::{Function, Item, ItemKind, Param};
use kowari_diagnostics::Span;
use kowari_lexer::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_item(&mut self) -> Result<Item<'src>, ParseError<'src>> {
        let start = self.current_span();
        let kind = match self.peek().copied().map(|t| t.kind) {
            Some(TokenKind::Function) => ItemKind::Function(self.parse_function()?),
            Some(found) => {
                return Err(ParseError::ItemExpected { found, span: start });
            }
            None => {
                return Err(ParseError::UnexpectedEof {
                    expected: TokenKind::Function,
                    span: start,
                });
            }
        };
        let end = self.prev_span();
        Ok(Item {
            kind,
            span: start.merge(end),
        })
    }

    fn parse_function(&mut self) -> Result<Function<'src>, ParseError<'src>> {
        let start = self.current_span();
        // consume `fn` token
        self.advance();
        let (name, name_span) = self.expect_ident()?;
        let params = self.parse_param_list()?;
        let ret = self.parse_optional_return_type()?;
        let body = self.parse_block()?;
        // `}` of the body
        let end = self.prev_span();
        Ok(Function {
            name,
            name_span,
            params,
            body,
            ret,
            span: start.merge(end),
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param<'src>>, ParseError<'src>> {
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if matches!(self.peek().map(|t| t.kind), Some(TokenKind::RParen)) {
            self.advance();
            return Ok(params);
        }
        loop {
            params.push(self.parse_param()?);
            match self.advance().map(|t| t.kind) {
                Some(TokenKind::Comma) => continue,
                Some(TokenKind::RParen) => break,
                Some(got) => {
                    let span = self.prev_span();
                    return Err(ParseError::UnexpectedToken {
                        expected: TokenKind::Comma,
                        got,
                        span,
                    });
                }
                None => {
                    return Err(ParseError::UnexpectedEof {
                        expected: TokenKind::Comma,
                        span: self.eof_span(),
                    });
                }
            }
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param<'src>, ParseError<'src>> {
        let start = self.peek().map(|t| t.span.start).unwrap_or(0);
        let (name, name_span) = self.expect_ident()?;
        // Only if the fixture has typed params:
        let ty = if matches!(self.peek().map(|t| t.kind), Some(TokenKind::Colon)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        Ok(Param {
            name,
            name_span,
            ty,
            span: Span::new(start, self.prev_span().end),
        })
    }
}
