mod errors;
mod expr;
mod item;
mod parser;
mod stmt;
mod ty;

pub use errors::ParseError;
use kowari_ast::item::{Param, ReturnType, SourceFile};
use kowari_ast::stmt::{Stmt, StmtKind};
use kowari_ast::ty::TypeKind;
use kowari_diagnostics::Span;
use kowari_lexer::TokenKind;
pub use parser::Parser;

#[cfg(test)]
mod tests;

impl<'src> Parser<'src> {

    fn eof_span(&self) -> Span {
        let end = self.tokens.last().map_or(0, |t| t.span.end);
        Span::new(end, end)
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

    fn expect_ident(&mut self) -> Result<&'src str, ParseError<'src>> {
        match self.advance() {
            Some(token) => match token.kind {
                TokenKind::Identifier(name) => Ok(name),
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
                    let span = self.tokens[self.pos - 1].span;
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
        let name = self.expect_ident()?;
        // Only if the fixture has typed params:
        let ty = if matches!(self.peek().map(|t| t.kind), Some(TokenKind::Colon)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let end = self.tokens[self.pos - 1].span.end;
        Ok(Param {
            name,
            ty,
            span: Span::new(start, end),
        })
    }

    fn parse_stmt(&mut self) -> Result<StmtKind<'src>, ParseError<'src>> {
        match self.peek().map(|t| t.kind) {
            Some(TokenKind::LetToken) => self.parse_let_stmt(),
            Some(TokenKind::ReturnToken) => self.parse_return_stmt(),
            _ => {
                let expr = self.parse_expr()?;
                Ok(StmtKind::Expr(expr)) // expression statement
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        let start = self.current_span();
        // Consume the 'let' token
        self.advance();

        let mutable = self.check(TokenKind::MutToken);
        if mutable {
            // Consume the mut token if it exists
            self.advance();
        };

        let name = self.expect_ident()?;
        //let pat = Pattern::Identifier(name);
        //let pat_span = start.merge(self.prev_span());

        let ty = if self.check(TokenKind::Colon) {
            // Consume the ':' then add the type
            self.advance();
            let id = self.expect_ident()?;
            Some(TypeKind::Name {
                name: id,
            })
        } else {
            None
        };

        // Parse the expression
        // (optional) `= Expr`
        let init = if self.check(TokenKind::Equals) {
            // consume '='
            self.advance();
            let expr = self.parse_expr()?;
            Some(expr)
        } else {
            None
        };

        // Expect a semicolon at the end of the statement
        let semi = self.expect(TokenKind::Semicolon)?;

        Ok(StmtKind::Let {
            mutable,
            name,
            ty,
            init,
            span: start.merge(semi),
        })
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn parse_return_stmt(&mut self) -> Result<StmtKind<'src>, ParseError<'src>> {
        // Consume the return
        self.advance();

        let value = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };

        self.expect(TokenKind::Semicolon)?;
        Ok(StmtKind::Return(value))
    }

    fn parse_optional_return_type(&mut self) -> Result<Option<ReturnType<'src>>, ParseError<'src>> {
        if !self.check(TokenKind::SmallArrow) {
            return Ok(None);
        }
        // Consume the `->` token
        // check() returned true → peek() was Some → advance() yields Some.
        let span = self
            .advance()
            .expect("check() guarantees a token is present")
            .span;

        let ty = self.parse_type()?;

        Ok(Some(ReturnType { span, ty }))
    }
}
