use crate::expr::Expr;
use crate::ty::Type;
use kowari_diagnostics::Span;

#[derive(Debug, Clone)]
pub enum Stmt<'src> {
    Let {
        mutable: bool,
        name: &'src str,
        ty: Option<Type<'src>>,
        init: Option<Expr<'src>>,
        span: Span,
    },
    Expr(Expr<'src>),
    // Return, Assign, etc. come later.
}
