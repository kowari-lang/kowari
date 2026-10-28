use crate::expr::Expr;
use crate::ty::Type;
use kowari_diagnostics::Span;

#[derive(Debug, Clone)]
pub struct Stmt<'src> {
    pub kind: StmtKind<'src>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StmtKind<'src> {
    Empty,
    Let {
        mutable: bool,
        name: &'src str,
        name_span: Span,
        ty: Option<Type<'src>>,
        init: Option<Expr<'src>>,
    },
    Expr(Expr<'src>),
    Defer(Expr<'src>),
    ErrDefer(Expr<'src>),
    Return(Option<Expr<'src>>),
    // Return, Assign, etc. come later.
}
