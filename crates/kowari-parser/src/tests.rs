use crate::{Parser, SourceFile};
use kowari_ast::item::ItemKind;
use kowari_ast::expr::{BinaryOp, ExprKind, UnaryOp};
use kowari_ast::stmt::Stmt;
use kowari_lexer::Token;

fn try_parse(src: &str) -> Result<SourceFile<'static>, Vec<kowari_diagnostics::Diag>> {
    let src: &'static str = Box::leak(src.to_string().into_boxed_str());
    let tokens: &'static [Token<'static>] =
        Box::leak(kowari_lexer::lex(src)?.into_boxed_slice());
    Parser::new(tokens).parse()
}

fn parse(src: &str) -> SourceFile<'static> {
    try_parse(src).unwrap_or_else(|diags| panic!("parse failed: {diags:#?}"))
}

/// Parses `fn f() { <body> }` and returns the block's statements.
fn parse_stmts(body: &str) -> &'static [Stmt<'static>] {
    let ast = Box::leak(Box::new(parse(&format!("fn f() {{ {body} }}"))));
    match &ast.items[0].kind {
        ItemKind::Function(f) => &f.body.stmts,
        other => panic!("expected fn, got {other:?}"),
    }
}

/// Parses `fn f() { let x = <expr>; }` and returns the initializer.
fn parse_expr(expr: &str) -> &'static kowari_ast::expr::Expr<'static> {
    match &parse_stmts(&format!("let x = {expr};"))[0] {
        Stmt::Let { init: Some(e), .. } => e,
        other => panic!("expected `let` with init, got {other:?}"),
    }
}

#[test]
#[allow(irrefutable_let_patterns)] // Item currently only has Function so it's always true
fn parses_empty_function() {
    let text = "fn main() {}";
    let tokens = kowari_lexer::lex(text).expect("lex");
    let mut parser = crate::Parser::new(&tokens);
    let ast = parser.parse().expect("parse");

    assert_eq!(ast.items.len(), 1);
    let ItemKind::Function(f) = &ast.items[0] else {
        panic!("expected a function, got {:?}", ast.items[0]);
    };
    assert_eq!(f.name, "main");
    assert!(f.params.is_empty());
    assert!(f.ret.is_none());
    assert!(f.body.stmts.is_empty());
    assert_eq!(f.span.start, 0);
    assert_eq!(f.span.end, text.len());
}

#[test]
fn parse_simple_add_function() {
    let text = "fn main(a: i32, b: i32) -> i32 { let sum = a + b; return sum; }";
    let tokens = kowari_lexer::lex(text).expect("lex failure");
    let mut parser = crate::Parser::new(&tokens);
    let ast = parser.parse().expect("parse failure");
    assert_eq!(ast.items.len(), 1);
    let ItemKind::Function(f) = &ast.items[0];
    assert_eq!(f.name, "main");
    assert!(!f.params.is_empty());
    assert!(f.ret.is_some());
    assert!(!f.body.stmts.is_empty(), "Body is empty: {:?}", f.body);
    assert_eq!(f.span.start, 0);
    assert_eq!(f.span.end, text.len());
}

#[test]
fn precedence_mul_binds_tighter_than_add() {
    let expr = parse_expr("a + b * c");
    let ExprKind::Binary { op: BinaryOp::Add, lhs, rhs } = &expr.kind else {
        panic!("expected top-level `+`, got {expr:#?}");
    };
    assert!(matches!(lhs.kind, ExprKind::Path("a")));
    let ExprKind::Binary { op: BinaryOp::Mul, .. } = &rhs.kind else {
        panic!("expected `*` on RHS, got {rhs:#?}");
    };
}

#[test]
fn precedence_add_over_comparison() {
    // `a + b < c + d`  →  `(a + b) < (c + d)`
    let expr = parse_expr("a + b < c + d");
    let ExprKind::Binary { op: BinaryOp::LessThan, lhs, rhs } = &expr.kind else {
        panic!("expected `<` at root, got {expr:#?}");
    };
    assert!(matches!(lhs.kind, ExprKind::Binary { op: BinaryOp::Add, .. }));
    assert!(matches!(rhs.kind, ExprKind::Binary { op: BinaryOp::Add, .. }));
}

#[test]
fn subtraction_is_left_associative() {
    // `a - b - c`  →  `(a - b) - c`, not `a - (b - c)`
    let expr = parse_expr("a - b - c");
    let ExprKind::Binary { op: BinaryOp::Sub, lhs, rhs } = &expr.kind else {
        panic!("expected `-` at root");
    };
    assert!(matches!(lhs.kind, ExprKind::Binary { op: BinaryOp::Sub, .. }));
    assert!(matches!(rhs.kind, ExprKind::Path("c")));
}

#[test]
fn parentheses_override_precedence() {
    // `(a + b) * c`  →  `Mul(Group(Add(a,b)), c)`
    let expr = parse_expr("(a + b) * c");
    let ExprKind::Binary { op: BinaryOp::Mul, lhs, .. } = &expr.kind else {
        panic!("expected `*` at root");
    };
    assert!(matches!(lhs.kind, ExprKind::Group(_)));
}

#[test]
fn unary_negation_binds_tighter_than_binary() {
    // `-a + b`  →  `Add(Neg(a), b)`
    let expr = parse_expr("-a + b");
    let ExprKind::Binary { op: BinaryOp::Add, lhs, .. } = &expr.kind else {
        panic!("expected `+` at root");
    };
    assert!(matches!(lhs.kind, ExprKind::Unary { op: UnaryOp::Negative, .. }));
}

#[test]
fn multiple_unary_operators_stack() {
    // `--a`  →  `Neg(Neg(a))`
    let expr = parse_expr("--a");
    let ExprKind::Unary { op: UnaryOp::Negative, operand } = &expr.kind else {
        panic!("expected outer `Neg`");
    };
    assert!(matches!(operand.kind, ExprKind::Unary { op: UnaryOp::Negative, .. }));
}

#[test]
fn let_without_init_or_type() {
    let stmts = parse_stmts("let x;");
    let Stmt::Let { mutable: false, name: "x", ty: None, init: None, .. } = &stmts[0] else {
        panic!("unexpected: {stmts:#?}");
    };
}

#[test]
fn let_mut_sets_flag() {
    let stmts = parse_stmts("let mut x = 1;");
    assert!(matches!(&stmts[0], Stmt::Let { mutable: true, name: "x", .. }));
}

#[test]
fn let_with_type_and_init() {
    let stmts = parse_stmts("let x: i32 = 5;");
    let Stmt::Let { ty: Some(_), init: Some(e), .. } = &stmts[0] else {
        panic!("missing ty/init: {stmts:#?}");
    };
    assert!(matches!(e.kind, ExprKind::Literal {
        value: kowari_ast::literal::Literal::Integer(_),
    }));
}

#[test]
fn return_with_and_without_value() {
    let stmts = parse_stmts("return 1; return;");
    assert!(matches!(&stmts[0], Stmt::Return(Some(_))));
    assert!(matches!(&stmts[1], Stmt::Return(None)));
}

#[test]
fn expression_statement() {
    let stmts = parse_stmts("a + b;");
    assert!(matches!(&stmts[0], Stmt::Expr(_)));
}

#[test]
fn function_parses_name_params_and_return_type() {
    let ast = parse("fn add(a: i32, b: i32) -> i32 { a + b }");
    let ItemKind::Function(f) = &ast.items[0].kind else { panic!() };
    assert_eq!(f.name, "add");
    assert_eq!(f.params.len(), 2);
    assert!(f.ret.is_some());
    assert_eq!(f.body.stmts.len(), 1);
}