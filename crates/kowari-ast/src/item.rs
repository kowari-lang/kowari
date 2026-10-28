use crate::Visibility;
use crate::stmt::Stmt;
use crate::ty::Type;
use kowari_diagnostics::Span;

#[derive(Debug, Clone)]
pub struct Item<'src> {
    pub kind: ItemKind<'src>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ItemKind<'src> {
    Function(Function<'src>),
    Struct(StructDefinition<'src>),
    Enum(EnumDefinition<'src>),
    Trait(TraitDefinition<'src>),
    Impl(ImplDefinition<'src>),
}

#[derive(Debug, Clone)]
pub struct Function<'src> {
    pub name: &'src str,
    pub name_span: Span,
    pub visibility: Visibility,
    pub params: Vec<Param<'src>>,
    pub ret: Option<ReturnType<'src>>,
    pub body: Block<'src>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param<'src> {
    pub name: &'src str,
    pub name_span: Span,
    pub ty: Option<Type<'src>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SourceFile<'src> {
    pub items: Vec<Item<'src>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block<'src> {
    pub stmts: Vec<Stmt<'src>>, // may be an empty stub for now
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ReturnType<'src> {
    pub ty: Type<'src>,
    pub arrow: Span,
}

#[derive(Clone, Debug)]
pub struct StructDefinition<'src> {
    pub name: &'src str,
    pub name_span: Span,
    pub fields: Vec<Item<'src>>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumDefinition<'src> {
    pub name: &'src str,
    pub name_span: Span,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TraitDefinition<'src> {
    pub name: &'src str,
    pub name_span: Span,
    pub items: Vec<Item<'src>>,
    pub visibility: Visibility,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct ImplDefinition<'src> {
    pub name: &'src str,
    pub name_span: Span,
    pub fields: Vec<Item<'src>>,
    pub visibility: Visibility,
    pub span: Span,
}
