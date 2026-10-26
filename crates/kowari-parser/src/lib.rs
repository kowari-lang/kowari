use kowari_ast::item::SourceFile;
use kowari_diagnostics::{Diag, Span};
use kowari_lexer::Token;

pub fn parse<'src>(_tokens: &[Token]) -> Result<SourceFile<'src>, Vec<Diag>> {
    Ok(SourceFile {
        items: Vec::new(),
        span: Span::default(),
    })
}
