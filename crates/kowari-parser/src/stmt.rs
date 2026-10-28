use crate::{ParseError, Parser};
use kowari_ast::item::Block;
use kowari_ast::stmt::{Stmt, StmtKind};
use kowari_lexer::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_statement(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        // bare `;`
        if self.check(TokenKind::Semicolon) {
            let tok = self.advance().unwrap();
            return Ok(Stmt {
                kind: StmtKind::Empty,
                span: tok.span,
            });
        }

        match self.peek().map(|t| t.kind) {
            Some(TokenKind::LetToken) => self.parse_let_statement(),
            Some(TokenKind::ReturnToken) => self.parse_return_statement(),
            Some(TokenKind::BreakToken) => self.parse_break_stmt(),
            Some(TokenKind::ContinueToken) => self.parse_continue_stmt(),
            _ => {
                let expr = self.parse_expression()?;
                if self.check(TokenKind::Semicolon) {
                    let semi = self.advance().unwrap();
                    return Ok(Stmt {
                        span: expr.span.merge(semi.span),
                        kind: StmtKind::Expr(expr),
                    });
                }
                Ok(Stmt {
                    span: expr.span,
                    kind: StmtKind::Expr(expr),
                })
            }
        }
    }

    pub(crate) fn parse_block(&mut self) -> Result<Block<'src>, ParseError<'src>> {
        let open = self.expect(TokenKind::LBrace)?; // assume it's checked first
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            stmts.push(self.parse_statement()?);
        }
        // stmt.rs — parse_block
        if !self.check(TokenKind::RBrace) {
            return Err(ParseError::UnclosedBlock { span: open.span });
        }
        let close = self.advance().unwrap(); // safe: check() just passed
        Ok(Block {
            stmts,
            span: open.span.merge(close.span),
        })
    }

    fn parse_let_statement(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        // Consumes the 'let' token
        let start = self.expect(TokenKind::LetToken)?.span;

        let mutable = if self.check(TokenKind::MutToken) {
            // Consume the mut token if it exists
            self.advance();
            true
        } else {
            false
        };

        let (name, name_span) = self.expect_ident()?;
        //let pat = Pattern::Identifier(name);
        //let pat_span = start.merge(self.prev_span());

        // Type annotation: `: Type`
        let ty = if self.check(TokenKind::Colon) {
            // Consume the ':' then add the type
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // `= Expr`
        let init = if self.check(TokenKind::Equals) {
            // consume '='
            self.advance();
            let expr = self.parse_expression()?;
            Some(expr)
        } else {
            None
        };

        // Expect a semicolon at the end of the statement
        let semi = self.expect(TokenKind::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Let {
                mutable,
                name,
                name_span,
                ty,
                init,
            },
            span: start.merge(semi.span),
        })
    }

    fn parse_return_statement(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        let start = self.expect(TokenKind::ReturnToken)?.span;

        // `return;` or `return }` → no value
        let value = if self.check(TokenKind::Semicolon) || self.check(TokenKind::RBrace) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        let end = if self.check(TokenKind::Semicolon) {
            self.advance().unwrap().span
        } else {
            value.as_ref().map(|e| e.span).unwrap_or(start)
        };

        Ok(Stmt {
            kind: StmtKind::Return(value),
            span: start.merge(end),
        })
    }

    fn parse_break_stmt(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        let start = self.current_span();
        // Consume `break` token
        self.advance();

        let value = if self.can_start_expr() {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        let semi = self.expect(TokenKind::Semicolon)?;
        Ok(Stmt {
            kind: StmtKind::Break(value),
            span: start.merge(semi.span),
        })
    }

    fn parse_continue_stmt(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        let start = self.current_span();
        // Consume `span` token
        self.advance();

        let semi = self.expect(TokenKind::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Continue,
            span: start.merge(semi.span),
        })
    }
}
