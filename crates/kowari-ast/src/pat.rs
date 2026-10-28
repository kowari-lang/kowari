use kowari_diagnostics::Span;

#[derive(Clone, Debug)]
pub struct Pattern<'src> {
    pub kind: PatternKind<'src>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum PatternKind<'src> {
    Identifier(&'src str),
    Literal(&'src str),
    Wildcard,
    Tuple(Vec<Pattern<'src>>),
}
