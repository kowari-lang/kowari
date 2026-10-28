use kowari_ast::ty::{Type, TypeKind};
use kowari_lexer::TokenKind;
use crate::{ParseError, Parser};

impl<'src> Parser<'src> {
    pub(crate) fn parse_type(&mut self) -> Result<Type<'src>, ParseError<'src>> {
        // Reference type: `&` [`mut`] Type
        if self.check(TokenKind::Ampersand) {
            let amp = self.advance().unwrap();
            let mutable = if self.check(TokenKind::MutToken) {
                self.advance();
                true
            } else {
                false
            };
            let inner = self.parse_type()?;
            return Ok(Type {
                kind: TypeKind::Reference {
                    mutable,
                    // inner: Box::new(inner),
                },
                span: amp.span.merge(inner.span),
            });
        }

        // Named type — existing behaviour
        let ident = self.expect_ident()?;
        Ok(Type {
            span: ident,
            kind: TypeKind::Name { name: ident}
        })
    }
}