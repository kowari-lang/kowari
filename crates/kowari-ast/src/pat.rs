use kowari_diagnostics::Span;

pub struct Pattern<'src> {
    pub kind: PatternKind<'src>,
    pub span: Span,
}

pub enum PatternKind<'src> {
    Identifier(&'src str),
    Literal(&'src str),
    Tuple(Vec<Pattern<'src>>),
}
