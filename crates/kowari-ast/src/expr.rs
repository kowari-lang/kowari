use crate::item::Param;
use crate::literal::Literal;
use crate::stmt::Stmt;
use crate::ty::Type;
use kowari_diagnostics::Span;

#[derive(Debug, Clone)]
pub struct Expr<'src> {
    pub kind: ExprKind<'src>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind<'src> {
    Identifier {
        name: &'src str,
    },
    Literal {
        value: Literal<'src>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr<'src>>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr<'src>>,
        rhs: Box<Expr<'src>>,
    },
    Call {
        callee: Box<Expr<'src>>,
        args: Vec<Expr<'src>>,
    },
    If {
        cond: Box<Expr<'src>>,
        then_branch: Block<'src>,
        else_branch: Option<Box<Expr<'src>>>,
    },
    Closure {
        params: Vec<Param<'src>>,
        ret: Option<Type<'src>>,
        body: Block<'src>,
    },
    Block(Block<'src>),
    Group(Box<Expr<'src>>),
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
    LessThan,
    GreaterThanEquals,
    LessThanEquals,
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
