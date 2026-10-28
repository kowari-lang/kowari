use crate::Parser;
use crate::errors::ParseError;
use kowari_ast::expr::{BinaryOp, Expr, ExprKind, UnaryOp};
use kowari_lexer::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Expr<'src>, ParseError<'src>> {
        let mut lhs = self.parse_unary_expr()?;

        while let Some(token) = self.peek().copied() {
            let Some((op, prec)) = Self::binary_op(&token.kind) else {
                break;
            };
            if prec < min_prec {
                break;
            }

            // Consume the left token
            self.advance();
            let rhs = self.parse_binary_expr(prec + 1)?;
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

    fn binary_op(kind: &TokenKind) -> Option<(BinaryOp, u8)> {
        use TokenKind::*;
        Some(match kind {
            DoublePipe => (BinaryOp::Or, 1),
            DoubleAnd => (BinaryOp::And, 2),
            DoubleEquals => (BinaryOp::Equal, 3),
            NotEquals => (BinaryOp::NotEqual, 3),
            LessThan => (BinaryOp::LessThan, 4),
            LessOrEquals => (BinaryOp::LessThanEquals, 4),
            GreaterThan => (BinaryOp::GreaterThan, 4),
            GreaterOrEquals => (BinaryOp::GreaterThanEquals, 4),
            Plus => (BinaryOp::Add, 5),
            Minus => (BinaryOp::Sub, 5),
            Star => (BinaryOp::Mul, 6),
            Slash => (BinaryOp::Div, 6),
            Percent => (BinaryOp::Remainder, 6),
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
                let inner = self.parse_expr()?;
                let end = self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    kind: ExprKind::Group(Box::new(inner)),
                    span: start.merge(end),
                })
            }
            _ => Err(ParseError::ExpressionExpected {
                found: token.kind,
                span: start,
            }),
        }
    }
}
