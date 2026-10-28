use kowari_ast::item::ReturnType;
use crate::{ParseError, Parser};
use kowari_ast::ty::{Type, TypeKind};
use kowari_lexer::TokenKind;

impl<'src> Parser<'src> {
    pub(crate) fn parse_type(&mut self) -> Result<Type<'src>, ParseError<'src>> {
        if self.check(TokenKind::Ampersand) {
            let amp_span = self.advance().unwrap().span;

            let mutable = if self.check(TokenKind::MutToken) {
                self.advance();
                true
            } else {
                false
            };

            let inner = self.parse_type()?;
            let inner_span = inner.span;
            let span = amp_span.merge(inner_span);

            return Ok(Type {
                kind: TypeKind::Reference {
                    mutable,
                    inner: Box::new(inner),
                },
                span,
            });
        }

        let ident = self.expect_ident()?;
        let span = self.prev_span();
        Ok(Type {
            kind: TypeKind::Name { name: ident },
            span,
        })
    }

    pub(crate) fn parse_optional_return_type(
        &mut self,
    ) -> Result<Option<ReturnType<'src>>, ParseError<'src>> {
        if !self.check(TokenKind::SmallArrow) {
            return Ok(None);
        }
        // Consume the `->` token
        // check() returned true → peek() was Some → advance() yields Some.
        let span = self
            .advance()
            .expect("check() guarantees a token is present")
            .span;

        let ty = self.parse_type()?;

        Ok(Some(ReturnType { arrow: span, ty }))
    }
}
