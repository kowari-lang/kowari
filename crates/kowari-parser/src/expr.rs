use crate::{ParseError, Parser};
use kowari_ast::expr::{BinaryOp, Expr, ExprKind, UnaryOp};
use kowari_lexer::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_expression(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        self.parse_bin_expr(0)
    }

    /// Precedence-climbing. `min_prec` is the minimum binding power the
    /// operator must have for us to consume it in this call.
    fn parse_bin_expr(&mut self, min_prec: u8) -> Result<Expr<'src>, ParseError<'src>> {
        let mut lhs = self.parse_unary_expr()?;

        loop {
            let Some((op, prec)) = self.peek_binop() else {
                break;
            };
            if prec < min_prec {
                break;
            }
            self.advance();
            // `prec + 1` forces LEFT associativity. Use `prec` instead for
            // right-associative operators (e.g. `=`).
            let rhs = self.parse_bin_expr(prec + 1)?;

            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }

    fn peek_binop(&self) -> Option<(BinaryOp, u8)> {
        let kind = self.peek()?.kind;
        let op_prec = match kind {
            TokenKind::DoublePipe => (BinaryOp::Or, 1),
            TokenKind::DoubleAnd => (BinaryOp::And, 2),
            TokenKind::DoubleEquals => (BinaryOp::Equal, 3),
            TokenKind::NotEquals => (BinaryOp::NotEqual, 3),
            TokenKind::LessThan => (BinaryOp::LessThan, 3),
            TokenKind::LessOrEquals => (BinaryOp::LessThanEquals, 3),
            TokenKind::GreaterThan => (BinaryOp::GreaterThan, 3),
            TokenKind::GreaterOrEquals => (BinaryOp::GreaterThanEquals, 3),
            TokenKind::Plus => (BinaryOp::Add, 4),
            TokenKind::Minus => (BinaryOp::Sub, 4),
            TokenKind::Star => (BinaryOp::Mul, 5),
            TokenKind::Slash => (BinaryOp::Div, 5),
            TokenKind::Percent => (BinaryOp::Remainder, 5),
            _ => return None,
        };
        Some(op_prec)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let is_unary = matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Exclamation | TokenKind::Minus),
        );
        if !is_unary {
            return self.parse_primary_expr();
        }
        let op_tok = self.advance().unwrap();
        let op = match op_tok.kind {
            TokenKind::Exclamation => UnaryOp::Not,
            TokenKind::Minus => UnaryOp::Negative,
            _ => unreachable!(),
        };
        let operand = self.parse_unary_expr()?;
        let span = op_tok.span.merge(operand.span);
        Ok(Expr {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
            span,
        })
    }

    fn parse_primary_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        // Literals, identifiers (→ ExprKind::Path), parenthesized exprs.
        // Anything your existing `parse_primary` already handled.
    }
}
