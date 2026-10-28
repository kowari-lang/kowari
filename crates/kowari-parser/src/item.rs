use kowari_ast::item::{Function, Item, ItemKind};
use kowari_lexer::TokenKind;
use crate::errors::ParseError;
use crate::parser::Parser;

impl<'src> Parser<'src> {
    pub(crate) fn parse_item(&mut self) -> Result<Item<'src>, ParseError<'src>> {
        let start = self.current_span();
        let kind = match self.peek().copied().map(|t| t.kind) {
            Some(TokenKind::Function) => ItemKind::Function(self.parse_function()?),
            Some(found) => {
                return Err(ParseError::ItemExpected { found, span: start });
            }
            None => {
                return Err(ParseError::UnexpectedEof {
                    expected: TokenKind::Function,
                    span: start,
                });
            }
        };
        let end = self.prev_span();
        Ok(Item {
            kind,
            span: start.merge(end),
        })
    }

    fn parse_function(&mut self) -> Result<Function<'src>, ParseError<'src>> {
        let start = self.current_span();
        // consume `fn` token
        self.advance();
        let name = self.expect_ident()?;
        let params = self.parse_param_list()?;
        let ret = self.parse_optional_return_type()?;
        let body = self.parse_block()?;
        // `}` of the body
        let end = self.prev_span();
        Ok(Function {
            name,
            params,
            body,
            ret,
            span: start.merge(end),
        })
    }
}