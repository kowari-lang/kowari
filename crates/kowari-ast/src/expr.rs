use crate::item::Param;
use crate::literal::Literal;
use crate::stmt::Stmt;
use crate::ty::Type;
use kowari_diagnostics::Span;

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Identifier {
        name: &'src str,
        span: Span,
    },
    Literal {
        value: Literal<'src>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr<'src>>,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr<'src>>,
        rhs: Box<Expr<'src>>,
        span: Span,
    },
    Call {
        callee: Box<Expr<'src>>,
        args: Vec<Expr<'src>>,
        span: Span,
    },
    If {
        cond: Box<Expr<'src>>,
        then_branch: Block<'src>,
        else_branch: Option<Box<Expr<'src>>>,
        span: Span,
    },
    Closure {
        params: Vec<Param<'src>>,
        ret: Option<Type<'src>>,
        body: Block<'src>,
        span: Span,
    },
    Block(Block<'src>),
}

#[derive(Debug, Clone)]
pub struct Block<'src> {
    pub stmts: Vec<Stmt<'src>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Remainder,
    Equal,
    NotEqual,
    GreaterThan,
    SmallerThan,
    GreaterThanEquals,
    SmallerThanEquals,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    BitLeftShift,
    BitRightShift,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negative,
    Not,
    Dereference,
    Reference,
}

impl Expr<'_> {
    pub fn span(&self) -> Span {
        match self {
            Expr::Identifier { span, .. } => *span,
            Expr::Literal { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Block(block) => block.span,
            Expr::Closure { span, .. } => *span,
        }
    }
}
