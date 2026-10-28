use kowari_diagnostics::Span;

#[derive(Debug, Clone)]
pub struct Type<'src> {
    pub kind: TypeKind<'src>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeKind<'src> {
    Name {
        name: &'src str,
    },
    Reference {
        mutable: bool,
        inner: Box<Type<'src>>,
    },
}
