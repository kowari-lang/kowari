use crate::Parser;
use crate::errors::ParseError;
use kowari_ast::expr::{BinaryOp, ExprKind, UnaryOp};
use kowari_ast::item::{ItemKind, ReturnType, SourceFile};
use kowari_ast::stmt::{Stmt, StmtKind};
use kowari_ast::ty::Type;
use kowari_diagnostics::Span;
use kowari_lexer::{Token, TokenKind, lex};

fn try_parse(src: &str) -> Result<SourceFile<'static>, Vec<kowari_diagnostics::Diag>> {
    let src: &'static str = Box::leak(src.to_string().into_boxed_str());
    let tokens: &'static [Token<'static>] = Box::leak(lex(src)?.into_boxed_slice());
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
        Stmt {
            kind: StmtKind::Let { init: Some(e), .. },
            ..
        } => e,
        other => panic!("expected `let` with init, got {other:?}"),
    }
}

/// Runs only `parse_optional_return_type` on `src`.
/// Adjust the Lexer/Parser construction to match your setup.
fn parse_optional_ret(src: &str) -> Result<Option<ReturnType<'static>>, ParseError<'static>> {
    let src: &'static str = Box::leak(src.to_string().into_boxed_str());
    let tokens: &'static [Token<'static>] =
        Box::leak(lex(src).expect("lex failed").into_boxed_slice());

    Parser::new(tokens).parse_optional_return_type()
}

fn parse_err_item(src: &str) -> ParseError {
    todo!()
}

#[test]
#[allow(irrefutable_let_patterns)] // Item currently only has Function so it's always true
fn parses_empty_function() {
    let text = "fn main() {}";
    let tokens = kowari_lexer::lex(text).expect("lex");
    let mut parser = crate::Parser::new(&tokens);
    let ast = parser.parse().expect("parse");

    assert_eq!(ast.items.len(), 1);
    let ItemKind::Function(f) = &ast.items[0].kind else {
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
    let ItemKind::Function(f) = &ast.items[0].kind else {
        panic!("expected a function");
    };
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
    let ExprKind::Binary {
        op: BinaryOp::Add,
        lhs,
        rhs,
    } = &expr.kind
    else {
        panic!("expected top-level `+`, got {expr:#?}");
    };
    assert!(matches!(lhs.kind, ExprKind::Path("a")));
    let ExprKind::Binary {
        op: BinaryOp::Mul, ..
    } = &rhs.kind
    else {
        panic!("expected `*` on RHS, got {rhs:#?}");
    };
}

#[test]
fn precedence_add_over_comparison() {
    // `a + b < c + d`  →  `(a + b) < (c + d)`
    let expr = parse_expr("a + b < c + d");
    let ExprKind::Binary {
        op: BinaryOp::LessThan,
        lhs,
        rhs,
    } = &expr.kind
    else {
        panic!("expected `<` at root, got {expr:#?}");
    };
    assert!(matches!(
        lhs.kind,
        ExprKind::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
    assert!(matches!(
        rhs.kind,
        ExprKind::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
}

#[test]
fn subtraction_is_left_associative() {
    // `a - b - c`  →  `(a - b) - c`, not `a - (b - c)`
    let expr = parse_expr("a - b - c");
    let ExprKind::Binary {
        op: BinaryOp::Sub,
        lhs,
        rhs,
    } = &expr.kind
    else {
        panic!("expected `-` at root");
    };
    assert!(matches!(
        lhs.kind,
        ExprKind::Binary {
            op: BinaryOp::Sub,
            ..
        }
    ));
    assert!(matches!(rhs.kind, ExprKind::Path("c")));
}

#[test]
fn parentheses_override_precedence() {
    // `(a + b) * c`  →  `Mul(Group(Add(a,b)), c)`
    let expr = parse_expr("(a + b) * c");
    let ExprKind::Binary {
        op: BinaryOp::Mul,
        lhs,
        ..
    } = &expr.kind
    else {
        panic!("expected `*` at root");
    };
    assert!(matches!(lhs.kind, ExprKind::Group(_)));
}

#[test]
fn unary_negation_binds_tighter_than_binary() {
    // `-a + b`  →  `Add(Neg(a), b)`
    let expr = parse_expr("-a + b");
    let ExprKind::Binary {
        op: BinaryOp::Add,
        lhs,
        ..
    } = &expr.kind
    else {
        panic!("expected `+` at root");
    };
    assert!(matches!(
        lhs.kind,
        ExprKind::Unary {
            op: UnaryOp::Negative,
            ..
        }
    ));
}

#[test]
fn multiple_unary_operators_stack() {
    // `--a`  →  `Neg(Neg(a))`
    let expr = parse_expr("--a");
    let ExprKind::Unary {
        op: UnaryOp::Negative,
        operand,
    } = &expr.kind
    else {
        panic!("expected outer `Neg`");
    };
    assert!(matches!(
        operand.kind,
        ExprKind::Unary {
            op: UnaryOp::Negative,
            ..
        }
    ));
}

#[test]
fn let_without_init_or_type() {
    let stmts = parse_stmts("let x;");
    let StmtKind::Let {
        mutable: false,
        name: "x",
        ty: None,
        init: None,
        ..
    } = &stmts[0].kind
    else {
        panic!("unexpected: {stmts:#?}");
    };
}

#[test]
fn let_mut_sets_flag() {
    let stmts = parse_stmts("let mut x = 1;");
    assert!(matches!(
        &stmts[0].kind,
        StmtKind::Let {
            mutable: true,
            name: "x",
            ..
        }
    ));
}

#[test]
fn let_with_type_and_init() {
    let stmts = parse_stmts("let x: i32 = 5;");
    let StmtKind::Let {
        ty: Some(_),
        init: Some(e),
        ..
    } = &stmts[0].kind
    else {
        panic!("missing ty/init: {stmts:#?}");
    };
    assert!(matches!(
        e.kind,
        ExprKind::Literal {
            value: kowari_ast::literal::Literal::Integer(_),
        }
    ));
}

#[test]
fn return_with_and_without_value() {
    let stmts = parse_stmts("return 1; return;");
    assert!(matches!(&stmts[0].kind, StmtKind::Return(Some(_))));
    assert!(matches!(&stmts[1].kind, StmtKind::Return(None)));
}

#[test]
fn expression_statement() {
    let stmts = parse_stmts("a + b;");
    assert!(matches!(&stmts[0].kind, StmtKind::Expr(_)));
}

#[test]
fn function_parses_name_params_and_return_type() {
    let ast = parse("fn add(a: i32, b: i32) -> i32 { a + b }");
    let ItemKind::Function(f) = &ast.items[0].kind else {
        panic!()
    };
    assert_eq!(f.name, "add");
    assert_eq!(f.params.len(), 2);
    assert!(f.ret.is_some());
    assert_eq!(f.body.stmts.len(), 1);
}

#[test]
fn function_item_span_covers_whole_function() {
    let src = "fn f() { }";
    let ast = parse(src);
    assert_eq!(ast.items[0].span.start, 0);
    assert_eq!(ast.items[0].span.end, src.len());
}

#[test]
fn returns_none_without_arrow() {
    let src: &'static str = "i32";
    let tokens: &'static [Token<'static>] =
        Box::leak(lex(src).expect("lex failed").into_boxed_slice());

    let mut parser = Parser::new(tokens);
    let before = parser.pos;
    let result = parser.parse_optional_return_type().unwrap();

    assert!(result.is_none());
    assert_eq!(parser.pos, before, "input must not be consumed");
}

#[test]
fn returns_none_on_empty_input() {
    // EOF: check() is false → Ok(None), not an error.
    let result = parse_optional_ret("");
    assert!(matches!(result, Ok(None)));
}

#[test]
fn parses_simple_return_type() {
    let rt = parse_optional_ret("-> i32").unwrap().unwrap();
    assert_eq!((rt.arrow.start, rt.arrow.end), (0, 2));
}

#[test]
fn arrow_span_points_at_small_arrow_only() {
    let rt = parse_optional_ret("->  bool").unwrap().unwrap();
    assert_eq!((rt.arrow.start, rt.arrow.end), (0, 2));
}

#[test]
fn parses_nested_return_type() {
    // Whatever your type grammar supports — pointer, reference,
    // generic, etc. Use one representative case.
    assert!(parse_optional_ret("-> &i32").is_ok());
}

#[test]
fn parses_bare_return_type() {
    assert!(parse_optional_ret("-> Foo").is_ok());
}

#[test]
fn errors_on_eof_after_arrow() {
    // `-> ` with nothing after must be an Err from parse_type().
    assert!(parse_optional_ret("->").is_err());
}

#[test]
fn errors_on_garbage_after_arrow() {
    // Second `->` is not a valid type start.
    assert!(parse_optional_ret("-> -> i32").is_err());
}

#[test]
fn error_span_after_arrow_points_past_the_arrow() {
    // The `->` was consumed, so the error should point at the
    // offending token, not at the arrow itself.
    let err = parse_optional_ret("-> -> i32").unwrap_err();
    let span = err.span(); // adjust to your accessor
    assert!(span.start >= 3, "error should not point at the arrow");
}

#[test]
fn does_not_consume_leading_non_arrow_tokens() {
    let src: &'static str = "(";
    let tokens: &'static [Token<'static>] =
        Box::leak(lex(src).expect("lex failed").into_boxed_slice());

    let mut parser = Parser::new(tokens);
    assert!(parser.parse_optional_return_type().unwrap().is_none());
    assert!(parser.check(TokenKind::LParen));
}

#[test]
fn optional_return_none_when_no_arrow() {
    // A type at the cursor must NOT be consumed.
    let src: &'static str = "i32";
    let tokens: &'static [Token<'static>] = Box::leak(lex(src).unwrap().into_boxed_slice());
    let mut parser = Parser::new(tokens);

    let before = parser.pos; // adjust if `pos` is private
    let result = parser.parse_optional_return_type().unwrap();
    assert!(result.is_none());
    assert_eq!(parser.pos, before);
}

#[test]
fn optional_return_parses_simple_type() {
    let rt = parse_optional_ret("-> i32").unwrap().unwrap();
    assert_eq!(rt.arrow, Span::new(0, 2));
    // The `Type` variant is whatever your `Type::Name` is.
    matches!(rt.ty, Type { .. });
}

#[test]
fn optional_return_errors_on_eof() {
    assert!(parse_optional_ret("->").is_err());
}

#[test]
fn optional_return_errors_on_garbage() {
    assert!(parse_optional_ret("-> -> i32").is_err());
}

#[test]
fn parse_if_no_else() {
    let e = parse_expr("if cond { }");
    assert!(matches!(e.kind, ExprKind::If {
        cond: _,
        then_branch: _,
        else_branch: None,
    }));
}

#[test]
fn parse_if_else() {
    let e = parse_expr("if a { } else { }");
    match e.kind.clone() {
        ExprKind::If { else_branch: Some(b), .. } => {
            assert!(matches!(b.kind, ExprKind::Block(_)));
        }
        _ => panic!("expected if with else"),
    }
}

#[test]
fn parse_else_if_chain() {
    let e = parse_expr("if a { } else if b { } else { }");
    // outer If has else_branch = Some(Block/Expr containing another If)
    // assert the inner If is reachable by descending into the else branch
}

#[test]
fn parse_while() {
    let e = parse_expr("while x < 10 { }");
    assert!(matches!(e.kind, ExprKind::While { .. }));
}

#[test]
fn parse_loop() {
    let e = parse_expr("loop { break; }");
    assert!(matches!(e.kind, ExprKind::Loop { .. }));
}

#[test]
fn parse_for_range() {
    // If you use `a..b` as a Range expression, test that too:
    let e = parse_expr("for i in 0..10 { }");
    assert!(matches!(e.kind, ExprKind::For { .. }));
}

#[test]
fn parse_if_error_missing_brace() {
    let err = parse_err_item("fn f() { if a { }");
    // should be UnclosedBlock (or whatever your error is)
}

#[test]
fn parse_if_error_missing_cond() {
    let err = parse_err_item("fn f() { if { } }");
    // should be ExpressionExpected
}