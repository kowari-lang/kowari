use crate::{ParseError, Parser};
use kowari_ast::pat::{Pattern, PatternKind};
use kowari_lexer::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_pattern(&mut self) -> Result<Pattern<'src>, ParseError<'src>> {
        let span = self.current_span();

        match self.peek_kind() {
            Some(TokenKind::Underscore) => {
                // Consume the wildcard
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Wildcard,
                    span,
                })
            }
            Some(TokenKind::Identifier(name)) => {
                // Consume the identifier
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Identifier(name),
                    span,
                })
            }
            _ => Err(ParseError::IdentifierExpected {
                found: self.peek_kind().unwrap_or(TokenKind::Eof),
                span,
            }),
        }
    }
}
