use kowari_diagnostics::{Diag, DiagCode, ErrorCode, Span};
use kowari_lexer::TokenKind;

#[derive(Debug)]
pub enum ParseError<'src> {
    UnexpectedToken {
        expected: TokenKind<'src>,
        got: TokenKind<'src>,
        span: Span,
    },
    UnexpectedEof {
        expected: TokenKind<'src>,
        span: Span,
    },
    UnclosedBlock {
        span: Span,
    },
    InvalidStatement {
        span: Span,
    },
    IdentifierExpected {
        span: Span,
        found: TokenKind<'src>,
    },
    ExpressionExpected {
        span: Span,
        found: TokenKind<'src>,
    },
    ItemExpected {
        found: TokenKind<'src>,
        span: Span,
    },
    NonAssociativeOperator {
        op: &'static str,
        span: Span,
    },
    ExpectedBlockOrIf {
        found: TokenKind<'src>,
        span: Span,
    },
}

impl<'src> ParseError<'src> {
    #[cfg(test)]
    pub(crate) fn span(&self) -> &Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => span,
            ParseError::UnexpectedEof { span, .. } => span,
            ParseError::UnclosedBlock { span } => span,
            ParseError::InvalidStatement { span } => span,
            ParseError::IdentifierExpected { span, .. } => span,
            ParseError::ExpressionExpected { span, .. } => span,
            ParseError::ItemExpected { span, .. } => span,
            ParseError::NonAssociativeOperator { span, .. } => span,
            ParseError::ExpectedBlockOrIf { span, .. } => span,
        }
    }
}

impl<'src> From<ParseError<'src>> for Diag {
    fn from(e: ParseError<'src>) -> Self {
        use ParseError::*;
        match e {
            UnexpectedToken {
                expected,
                got,
                span,
            } => Diag {
                code: DiagCode::from(ErrorCode::ExpectedToken),
                message: format!("expected `{expected:?}`, found `{got:?}`"),
                span,
            },
            UnexpectedEof { ref expected, span } => Diag {
                code: DiagCode::from(ErrorCode::UnexpectedEof),
                message: format!("unexpected end of input; expected `{expected:?}`"),
                span,
            },
            UnclosedBlock { span } => Diag {
                code: DiagCode::from(ErrorCode::UnterminatedBlock),
                message: "Unclosed block, expected a closing '}'".to_string(),
                span,
            },
            InvalidStatement { span } => Diag {
                code: DiagCode::from(ErrorCode::InvalidStatement),
                message: format!("invalid statement `{span}`"),
                span,
            },
            IdentifierExpected { span, found } => Diag {
                code: DiagCode::from(ErrorCode::ExpectedToken),
                message: format!("Expected an Identifier but got `{found}`"),
                span,
            },
            ExpressionExpected { span, found } => Diag {
                code: DiagCode::from(ErrorCode::ExpectedToken),
                message: format!("Expected an Expression but got `{found}`"),
                span,
            },
            ItemExpected { span, found } => Diag {
                code: DiagCode::from(ErrorCode::ExpectedToken),
                message: format!("Expected an Item but got `{found}`"),
                span,
            },
            NonAssociativeOperator { op, span } => Diag {
                code: DiagCode::from(ErrorCode::NonAssociativeOperator),
                message: format!("`{op}` cannot be chained; add parentheses to disambiguate"),
                span,
            },
            ExpectedBlockOrIf { found, span } => Diag {
                code: DiagCode::from(ErrorCode::ExpectedToken),
                message: format!("Expected a Block or If but got `{found}`"),
                span,
            },
        }
    }
}
