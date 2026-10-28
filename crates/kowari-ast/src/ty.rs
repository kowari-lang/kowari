use kowari_diagnostics::Span;

#[derive(Debug, Clone)]
pub enum Type<'src> {
    Name { name: &'src str, span: Span },
    
}
