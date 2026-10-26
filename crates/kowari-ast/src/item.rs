use crate::expr::Block;
use crate::ty::Type;
use kowari_diagnostics::Span;

#[derive(Debug, Clone)]
pub enum Item<'src> {
    Function(Function<'src>),
    // Struct, Enum, … later
}

#[derive(Debug, Clone)]
pub struct Function<'src> {
    pub name: &'src str,
    pub params: Vec<Param<'src>>,
    pub ret: Option<Type<'src>>,
    pub body: Block<'src>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param<'src> {
    pub name: &'src str,
    pub ty: Option<Type<'src>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SourceFile<'src> {
    pub items: Vec<Item<'src>>,
    pub span: Span,
}
