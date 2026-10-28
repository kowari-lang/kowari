use crate::{ParseError, Parser};
use kowari_ast::expr::{BinaryOp, Expr, ExprKind, UnaryOp};
use kowari_lexer::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_expression(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        self.parse_range_expr()
    }

    fn parse_unary_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let op = match self.peek_kind() {
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
                    kind: ExprKind::Path(name),
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

    fn can_start_expr(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(
                TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::Char(_)
                | TokenKind::TrueToken
                | TokenKind::FalseToken
                | TokenKind::Identifier(_)
                | TokenKind::LParen
                | TokenKind::Minus       // unary -
                | TokenKind::Exclamation // unary !
                | TokenKind::Ampersand   // unary &
                | TokenKind::Star        // unary *
                | TokenKind::DoubleDot   // nested range prefix (parse will reject with E0015)
                | TokenKind::RangeInclusive
            )
        )
    }

    fn parse_range_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        // Prefix form: `..e`, `..`
        if self.check(TokenKind::DoubleDot) || self.check(TokenKind::RangeInclusive) {
            let op = self.advance().unwrap();
            let inclusive = matches!(op.kind, TokenKind::RangeInclusive);
            let end = if self.can_start_expr() {
                Some(Box::new(self.parse_logical_or_expr()?))
            } else {
                None
            };

            if self.check(TokenKind::DoubleDot) || self.check(TokenKind::RangeInclusive) {
                return Err(ParseError::NonAssociativeOperator {
                    op: if inclusive { "..=" } else { ".." },
                    span: self.current_span(),
                });
            }

            return Ok(Expr {
                span: op.span.merge(end.as_ref().map_or(op.span, |e| e.span)),
                kind: ExprKind::Range {
                    start: None,
                    end,
                    inclusive,
                },
            });
        }

        let lhs = self.parse_logical_or_expr()?;

        if self.check(TokenKind::DoubleDot) || self.check(TokenKind::RangeInclusive) {
            let op = self.advance().unwrap();
            let inclusive = matches!(op.kind, TokenKind::RangeInclusive);
            let end = if self.can_start_expr() {
                Some(Box::new(self.parse_logical_or_expr()?))
            } else {
                None
            };
            if self.check(TokenKind::DoubleDot) || self.check(TokenKind::RangeInclusive) {
                return Err(ParseError::NonAssociativeOperator {
                    op: if inclusive { "..=" } else { ".." },
                    span: self.current_span(),
                });
            }
            return Ok(Expr {
                span: lhs.span.merge(end.as_ref().map_or(op.span, |e| e.span)),
                kind: ExprKind::Range {
                    start: Some(Box::new(lhs)),
                    end,
                    inclusive,
                },
            });
        }

        Ok(lhs)
    }

    fn parse_logical_or_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let mut lhs = self.parse_logical_and_expr()?;
        while self.check(TokenKind::DoublePipe) {
            self.advance();
            let rhs = self.parse_logical_and_expr()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                span,
                kind: ExprKind::Binary {
                    op: BinaryOp::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            }
        }
        Ok(lhs)
    }

    fn parse_logical_and_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let mut lhs = self.parse_comparison_expr()?;
        while self.check(TokenKind::DoubleAnd) {
            self.advance();
            let rhs = self.parse_comparison_expr()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                span,
                kind: ExprKind::Binary {
                    op: BinaryOp::And,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            };
        }
        Ok(lhs)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let lhs = self.parse_additive_expr()?;

        let op = match self.peek_kind() {
            Some(TokenKind::DoubleEquals) => BinaryOp::Equal,
            Some(TokenKind::NotEquals) => BinaryOp::NotEqual,
            Some(TokenKind::LessThan) => BinaryOp::LessThan,
            Some(TokenKind::LessOrEquals) => BinaryOp::LessThanEquals,
            Some(TokenKind::GreaterThan) => BinaryOp::GreaterThan,
            Some(TokenKind::GreaterOrEquals) => BinaryOp::GreaterThanEquals,
            _ => return Ok(lhs),
        };

        self.advance();
        let rhs = self.parse_additive_expr()?;

        // Non-associativity: `a < b < c` is an error.
        if matches!(
            self.peek_kind(),
            Some(
                TokenKind::DoubleEquals
                    | TokenKind::NotEquals
                    | TokenKind::LessThan
                    | TokenKind::LessOrEquals
                    | TokenKind::GreaterThan
                    | TokenKind::GreaterOrEquals
            )
        ) {
            return Err(ParseError::NonAssociativeOperator {
                op: "comparison",
                span: lhs.span,
            });
        }

        let span = lhs.span.merge(rhs.span);
        Ok(Expr {
            span,
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
        })
    }

    fn parse_additive_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let mut lhs = self.parse_multiplicative_expr()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Plus) => BinaryOp::Add,
                Some(TokenKind::Minus) => BinaryOp::Sub,
                _ => return Ok(lhs),
            };
            self.advance();
            let rhs = self.parse_multiplicative_expr()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                span,
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            };
        }
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr<'src>, ParseError<'src>> {
        let mut lhs = self.parse_unary_expr()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Star) => BinaryOp::Mul,
                Some(TokenKind::Slash) => BinaryOp::Div,
                Some(TokenKind::Percent) => BinaryOp::Remainder,
                _ => return Ok(lhs),
            };
            self.advance();
            let rhs = self.parse_unary_expr()?;
            let span = lhs.span.merge(rhs.span);
            lhs = Expr {
                span,
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            };
        }
    }
}
