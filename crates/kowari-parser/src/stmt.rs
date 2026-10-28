use kowari_ast::item::Block;
use kowari_ast::stmt::{Stmt, StmtKind};
use kowari_lexer::TokenKind;
use crate::{ParseError, Parser};

impl<'src> Parser<'src> {
    pub(crate) fn parse_statement(&mut self) -> Result<Stmt<'src>, ParseError<'src>> {
        // Empty statement: bare `;`
        if self.check(TokenKind::Semicolon) {
            let tok = self.advance().unwrap();
            return Ok(Stmt { kind: StmtKind::Empty, span: tok.span });
        }

        let expr = self.parse_expression()?;

        // Consume the terminating `;` when present.
        if self.check(TokenKind::Semicolon) {
            let semi = self.advance().unwrap();
            return Ok(Stmt {
                kind: StmtKind::Expr(expr),
                span: expr.span.merge(semi.span),
            });
        }

        // No `;` — tail expression of the block.
        Ok(Stmt { span: expr.span, kind: StmtKind::Expr(expr) })
    }

    pub(crate) fn parse_block(&mut self) -> Result<Block<'src>, ParseError<'src>> {
        let open = self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && self.peek().is_some() {
            stmts.push(self.parse_statement()?);
        }
        let close = self.expect(TokenKind::RBrace)?;
        Ok(Block { stmts, span: open.span.merge(close.span) })
    }
}