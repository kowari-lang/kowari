use kowari_diagnostics::{Diag, DiagCode, ErrorCode, Span};
use kowari_lexer::TokenKind;

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
}

impl<'src> From<ParseError<'src>> for Diag {
    fn from(e: ParseError) -> Self {
        use ParseError::*;
        match e {
            UnexpectedToken {
                expected,
                got,
                span,
            } => Diag {
                code: DiagCode::from(ErrorCode::ExpectedToken),
                span,
                message: format!("expected `{expected:?}`, found `{got:?}`"),
            },
            UnexpectedEof { ref expected, span } => Diag {
                code: DiagCode::from(ErrorCode::UnexpectedEof),
                span,
                message: format!("unexpected end of input; expected `{expected:?}`"),
            },
        }
    }
}
