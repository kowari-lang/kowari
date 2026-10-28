use crate::{ParseError, Parser};
use kowari_ast::expr::{BinaryOp, Expr, ExprKind, UnaryOp};
use kowari_lexer::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_expression(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        self.parse_bin_expr(0)
    }

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
            let rhs = self.parse_bin_expr(prec + 1)?; // left-assoc
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
        Some(match kind {
            TokenKind::DoublePipe => (BinaryOp::Or, 1),
            TokenKind::DoubleAnd => (BinaryOp::And, 2),
            TokenKind::DoubleEquals => (BinaryOp::Equal, 3),
            TokenKind::NotEquals => (BinaryOp::NotEqual, 3),
            TokenKind::LessThan => (BinaryOp::LessThan, 4),
            TokenKind::LessOrEquals => (BinaryOp::LessThanEquals, 4),
            TokenKind::GreaterThan => (BinaryOp::GreaterThan, 4),
            TokenKind::GreaterOrEquals => (BinaryOp::GreaterThanEquals, 4),
            TokenKind::Plus => (BinaryOp::Add, 5),
            TokenKind::Minus => (BinaryOp::Sub, 5),
            TokenKind::Star => (BinaryOp::Mul, 6),
            TokenKind::Slash => (BinaryOp::Div, 6),
            TokenKind::Percent => (BinaryOp::Remainder, 6),
            _ => return None,
        })
    }

    fn parse_unary_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let op = match self.peek().copied().map(|t| t.kind) {
            Some(TokenKind::Minus) => Some(UnaryOp::Negative),
            Some(TokenKind::Exclamation) => Some(UnaryOp::Not),
            Some(TokenKind::Ampersand) => Some(UnaryOp::Reference),
            Some(TokenKind::Star) => Some(UnaryOp::Dereference),
            _ => None,
        };
        if let Some(op) = op {
            let start = self.current_span();
            self.advance();
            let operand = self.parse_unary_expr()?; // recurse: `--x`, `!!x`, `&&x`
            let span = start.merge(operand.span);
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
                span,
            });
        }
        self.parse_primary_expr()
    }

    fn parse_primary_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let start = self.current_span();

        let Some(token) = self.peek() else {
            return Err(ParseError::ExpressionExpected {
                found: TokenKind::Eof,
                span: start,
            });
        };

        match token.kind {
            TokenKind::Integer(v) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal {
                        value: kowari_ast::literal::Literal::Integer(v),
                    },
                    span: start,
                })
            }
            TokenKind::Float(v) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal {
                        value: kowari_ast::literal::Literal::Float(v),
                    },
                    span: start,
                })
            }
            TokenKind::String(v) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal {
                        value: kowari_ast::literal::Literal::String(v),
                    },
                    span: start,
                })
            }
            TokenKind::Char(v) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal {
                        value: kowari_ast::literal::Literal::Char(v),
                    },
                    span: start,
                })
            }
            TokenKind::TrueToken => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal {
                        value: kowari_ast::literal::Literal::Boolean(true),
                    },
                    span: start,
                })
            }
            TokenKind::FalseToken => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal {
                        value: kowari_ast::literal::Literal::Boolean(false),
                    },
                    span: start,
                })
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal {
                        value: kowari_ast::literal::Literal::Path(name),
                    },
                    span: start,
                })
            }
            TokenKind::LParen => {
                self.advance();
                let inner = self.parse_expression()?;
                let end = self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    kind: ExprKind::Group(Box::new(inner)),
                    span: start.merge(end.span),
                })
            }
            _ => Err(ParseError::ExpressionExpected {
                found: token.kind,
                span: start,
            }),
        }
    }
}
