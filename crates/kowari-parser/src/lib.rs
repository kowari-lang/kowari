mod errors;
mod expr_parsing;
#[cfg(test)]
mod tests;

use crate::errors::ParseError;
use kowari_ast::item::{Block, Item, SourceFile};
use kowari_ast::item::{Function, Param};
use kowari_ast::stmt::Stmt;
use kowari_ast::ty::Type;
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
        let start = self.expect(TokenKind::Function)?.start;
        let name = self.expect_ident()?;
        let params = self.parse_param_list()?;
        let ret = if matches!(self.peek().map(|t| t.kind), Some(TokenKind::SmallArrow)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        let span = Span::new(start, body.span.end);
        Ok(Function {
            name,
            params,
            ret,
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
        let start = self.current_span();
        let kind = match self.peek().copied().map(|t| t.kind) {
            Some(TokenKind::Function) => self.parse_function()?,
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

    fn parse_block(&mut self) -> Result<Block<'src>, ParseError<'src>> {
        let span = self.current_span();

        self.expect(TokenKind::LBrace)?;

        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }

        if self.is_at_end() {
            return Err(ParseError::UnexpectedEof {
                expected: TokenKind::RBrace,
                span,
            });
        }
        let end = self.expect(TokenKind::RBrace)?;

        Ok(Block {
            stmts,
            span: span.merge(end),
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

    fn parse_type(&mut self) -> Result<Type<'src>, ParseError<'src>> {
        // Minimal: a bare identifier as a type name (e.g. `i32`, `Foo`)
        let span = self.peek().map(|t| t.span).unwrap_or_default();
        match self.advance().map(|t| t.kind) {
            Some(TokenKind::Identifier(name)) => Ok(Type::Name { name, span }),
            Some(got) => Err(ParseError::UnexpectedToken {
                expected: TokenKind::Identifier(""),
                got,
                span,
            }),
            None => Err(ParseError::UnexpectedEof {
                expected: TokenKind::Identifier(""),
                span: self.eof_span(),
            }),
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        match self.peek().map(|t| t.kind) {
            Some(TokenKind::LetToken) => self.parse_let_stmt(),
            Some(TokenKind::ReturnToken) => self.parse_return_stmt(),
            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::Expr(expr)) // expression statement
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
            Some(Type::Name {
                name: id,
                span: Span::new(0, 0),
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

        Ok(Stmt::Let {
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

    fn current_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else {
            // EOF: zero-width span at the end of the source
            Span::at(self.tokens.last().map(|t| t.span.end).unwrap_or(0))
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.tokens[self.pos].kind == kind
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        // Consume the return
        self.advance();

        let value = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };

        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(value))
    }

    fn parse_function(&mut self) -> Result<Function<'src>, ParseError<'src>> {
        let start = self.current_span();
        // consume `fn` token
        self.advance();
        let name = self.expect_ident()?;
        let params = self.parse_param_list()?;
        let ret = self.parse_optional_return_type()?;
        let body = self.parse_block()?;
        // `}` of the body
        let end = self.prev_span();
        Ok(Function {
            name,
            params,
            body,
            ret,
            span: start.merge(end),
        })
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::at(0)
        }
    }

    fn parse_optional_return_type(&self) -> Result<Option<Type>, ParseError<'src>> {
        todo!()
    }
}
