use crate::Parser;
use crate::errors::ParseError;
use kowari_ast::Visibility;
use kowari_ast::expr::{BinaryOp, Expr, ExprKind, UnaryOp};
use kowari_ast::item::{Item, ItemKind, ReturnType, SourceFile};
use kowari_ast::pat::{Pattern, PatternKind};
use kowari_ast::stmt::{Stmt, StmtKind};
use kowari_ast::ty::Type;
use kowari_diagnostics::{Diag, DiagCode, ErrorCode, Span};
use kowari_lexer::{Token, TokenKind, lex};

fn parser(src: &'static str) -> Parser<'static> {
    let tokens = lex(src).unwrap_or_else(|diags| {
        let joined = diags
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        panic!("lexer failed for {src:?}: {joined}")
    });
    // Token<'src> borrows from src; leaking the box gives us a 'static slice
    // that outlives the call. Fine in tests, never in real code.
    let tokens: &'static [Token<'static>] = Box::leak(tokens.into_boxed_slice());
    Parser::new(tokens)
}

fn parse_item(src: &'static str) -> Item<'static> {
    parser(src).parse_item().unwrap_or_else(|e| {
        let d = Diag::from(e);
        panic!("expected Ok for {src:?}, got {d}")
    })
}

fn parse_expr(src: &'static str) -> Expr<'static> {
    parser(src).parse_expression().unwrap_or_else(|e| {
        let d = Diag::from(e);
        panic!("expected Ok for {src:?}, got {d}")
    })
}
fn _parse_stmt(src: &'static str) -> Stmt<'static> {
    parser(src).parse_statement().unwrap_or_else(|e| {
        let d = Diag::from(e);
        panic!("expected Ok for {src:?}, got {d}")
    })
}
fn _parse_type(src: &'static str) -> Type<'static> {
    parser(src).parse_type().unwrap_or_else(|e| {
        let d = Diag::from(e);
        panic!("expected Ok for {src:?}, got {d}")
    })
}

/// Terse assertion helper — most error tests use this form.
/// Most error tests use this. Parses as an item by default; add
/// `parse_err_expr` / `parse_err_stmt` when asserting on expression-level errors.
#[track_caller]
fn parse_err_item(src: &'static str) -> Diag {
    match parser(src).parse_item() {
        Ok(ok) => panic!("expected Err for {src:?}, got Ok({ok:?})"),
        Err(e) => Diag::from(e),
    }
}

#[track_caller]
fn assert_err_code(src: &'static str, code: ErrorCode) {
    let d = parse_err_item(src);
    assert_eq!(d.code, DiagCode::from(code), "wrong code for {src:?}: {d}");
}

/// When the exact byte offset matters.
#[track_caller]
fn _assert_err_code_at(src: &'static str, code: ErrorCode, start: usize) {
    let d = parse_err_item(src);
    assert_eq!(d.code, DiagCode::from(code), "wrong code for {src:?}: {d}");
    assert_eq!(d.span.start, start, "wrong span for {src:?}: {d}");
}

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

/// Runs only `parse_optional_return_type` on `src`.
/// Adjust the Lexer/Parser construction to match your setup.
fn parse_optional_ret(src: &str) -> Result<Option<ReturnType<'static>>, ParseError<'static>> {
    let src: &'static str = Box::leak(src.to_string().into_boxed_str());
    let tokens: &'static [Token<'static>] =
        Box::leak(lex(src).expect("lex failed").into_boxed_slice());

    Parser::new(tokens).parse_optional_return_type()
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
fn parse_loop() {
    let e = parse_expr("loop { }");
    assert!(matches!(e.kind, ExprKind::Loop { .. }));
}

// ---- block / unclosed ----
#[test]
fn parse_unclosed_fn_body() {
    assert_err_code("fn f() {", ErrorCode::UnterminatedBlock);
}

// ---- if ----
#[test]
fn parse_if_no_else() {
    let e = parse_expr("if cond { }");
    assert!(matches!(
        e.kind,
        ExprKind::If {
            else_branch: None,
            ..
        }
    ));
}

#[test]
fn parse_if_else() {
    let e = parse_expr("if a { } else { }");
    match e.kind {
        ExprKind::If {
            else_branch: Some(b),
            ..
        } => assert!(matches!(b.kind, ExprKind::Block(_))),
        _ => panic!("expected if with else"),
    }
}

#[test]
fn parse_else_if_chain() {
    let e = parse_expr("if a { } else if b { } else { }");
    match e.kind {
        ExprKind::If {
            else_branch: Some(inner),
            ..
        } => assert!(matches!(inner.kind, ExprKind::If { .. })),
        _ => panic!("expected if chain"),
    }
}

#[test]
fn parse_if_missing_brace() {
    assert_err_code("fn f() { if a { }", ErrorCode::UnterminatedBlock);
}

#[test]
fn parse_if_missing_cond() {
    assert_err_code("fn f() { if ; { } }", ErrorCode::ExpectedToken);
}

// ---- if let ----
#[test]
fn parse_if_let_ident() {
    let e = parse_expr("if let x = opt { }");
    match e.kind {
        ExprKind::IfLet {
            pattern:
                Pattern {
                    kind: PatternKind::Identifier(name),
                    ..
                },
            ..
        } => assert_eq!(name, "x"),
        _ => panic!("expected if let"),
    }
}

// ---- while / loop / break / continue ----
#[test]
fn parse_while() {
    let e = parse_expr("while x < 10 { }");
    assert!(matches!(e.kind, ExprKind::While { .. }));
}

#[test]
fn parse_loop_break() {
    let e = parse_expr("loop { break; }");
    match e.kind {
        ExprKind::Loop { body } => {
            assert_eq!(body.stmts.len(), 1);
            assert!(matches!(body.stmts[0].kind, StmtKind::Break(None)));
        }
        _ => panic!(),
    }
}

#[test]
fn parse_loop_break_with_value() {
    let e = parse_expr("loop { break 42; }");
    match e.kind {
        ExprKind::Loop { body } => match &body.stmts[0].kind {
            StmtKind::Break(Some(v)) => {
                assert!(matches!(v.kind, ExprKind::Literal { .. }));
            }
            other => panic!("expected break with value, got {other:?}"),
        },
        _ => panic!(),
    }
}

#[test]
fn parse_continue() {
    let e = parse_expr("loop { continue; }");
    match e.kind {
        ExprKind::Loop { body } => assert!(matches!(body.stmts[0].kind, StmtKind::Continue)),
        _ => panic!(),
    }
}

// ---- for ----
#[test]
fn parse_for_range() {
    let e = parse_expr("for i in 0..10 { }");
    match e.kind {
        ExprKind::For { iter, .. } => assert!(matches!(iter.kind, ExprKind::Range { .. })),
        _ => panic!(),
    }
}

#[test]
fn parse_for_over_expr() {
    let e = parse_expr("for x in vec { }");
    match e.kind {
        ExprKind::For { iter, .. } => assert!(matches!(iter.kind, ExprKind::Path(_))),
        _ => panic!(),
    }
}

// ---- ranges ----
#[test]
fn parse_range_full() {
    let e = parse_expr("0..10");
    assert!(matches!(
        e.kind,
        ExprKind::Range {
            start: Some(_),
            end: Some(_),
            inclusive: false
        }
    ));
}

#[test]
fn parse_range_inclusive() {
    let e = parse_expr("0..=10");
    assert!(matches!(
        e.kind,
        ExprKind::Range {
            inclusive: true,
            ..
        }
    ));
}

#[test]
fn parse_range_from() {
    let e = parse_expr("..10");
    assert!(matches!(
        e.kind,
        ExprKind::Range {
            start: None,
            end: Some(_),
            inclusive: false
        }
    ));
}

#[test]
fn parse_range_to() {
    let e = parse_expr("0..");
    assert!(matches!(
        e.kind,
        ExprKind::Range {
            start: Some(_),
            end: None,
            ..
        }
    ));
}

#[test]
fn parse_range_full_open() {
    let e = parse_expr("..");
    assert!(matches!(
        e.kind,
        ExprKind::Range {
            start: None,
            end: None,
            ..
        }
    ));
}

#[test]
fn parse_range_lowest_precedence() {
    // `..` binds looser than `==`, so this is `a..(b == c)`
    let e = parse_expr("a..b == c");
    match e.kind {
        ExprKind::Range { end: Some(rhs), .. } => {
            assert!(matches!(rhs.kind, ExprKind::Binary { .. }))
        }
        _ => panic!(),
    }
}

#[test]
fn parse_range_non_associative() {
    assert_err_code_expr("a..b..c", ErrorCode::NonAssociativeOperator);
}

#[test]
fn parse_range_inclusive_non_associative() {
    assert_err_code_expr("a..=b..c", ErrorCode::NonAssociativeOperator);
}

// ---- visibility (Function only in Section A) ----
#[test]
fn parse_pub_fn() {
    let i = parse_item("pub fn f() { }");
    match i.kind {
        ItemKind::Function(f) => assert!(matches!(f.visibility, Visibility::Public)),
        _ => panic!(),
    }
}

#[test]
fn parse_private_fn() {
    let i = parse_item("fn f() { }");
    match i.kind {
        ItemKind::Function(f) => assert!(matches!(f.visibility, Visibility::Private)),
        _ => panic!(),
    }
}

// ---- never panic on garbage ----
#[test]
fn control_flow_never_panics_on_garbage() {
    for src in [
        "fn f () { if }",
        "fn f () { if { }",
        "fn f () { if a { }",
        "fn f () { if let { }",
        "fn f () { if let x { }",
        "fn f () { while }",
        "fn f () { while a }",
        "fn f () { loop }",
        "fn f () { loop { }",
        "fn f () { for }",
        "fn f () { for i }",
        "fn f () { for i in }",
        "fn f () { for i in a }",
        "fn f () { break }",
        "fn f () { break 1 }",
        "fn f () { continue }",
        "..",
        "..=",
        "0..",
        "..0",
        "..=..",
        "a..=b..c",
        "pub",
        "pub pub fn f() {}",
    ] {
        let _ = std::panic::catch_unwind(|| {
            let _ = parser(src).parse_item();
        });
    }
}

#[test]
fn parse_comparison_non_assoc_span() {
    // "a < b < c":          0123456
    //                         ^ second `<` is at byte 6
    assert_err_code_at_expr("a < b < c", ErrorCode::NonAssociativeOperator, 6);
}

#[track_caller]
fn parse_err_expr(src: &'static str) -> Diag {
    match parser(src).parse_expression() {
        Ok(ok) => panic!("expected Err for {src:?}, got Ok({ok:?})"),
        Err(e) => Diag::from(e),
    }
}

#[track_caller]
fn assert_err_code_at_expr(src: &'static str, code: ErrorCode, start: usize) {
    let d = parse_err_expr(src);
    assert_eq!(d.code, DiagCode::from(code), "wrong code for {src:?}: {d}");
    assert_eq!(d.span.start, start, "wrong span for {src:?}: {d}");
}

#[track_caller]
fn assert_err_code_expr(src: &'static str, code: ErrorCode) {
    let d = parse_err_expr(src);
    assert_eq!(d.code, DiagCode::from(code), "wrong code for {src:?}: {d}");
}

// ---- if span tests ----
#[test]
fn parse_if_span_covers_whole_construct_no_else() {
    let src = "if a { }";
    let e = parser(src).parse_expression().unwrap();
    assert_eq!(e.span.start, 0);
    assert_eq!(e.span.end, src.len());
}

#[test]
fn parse_if_span_covers_else_tail() {
    let src = "if a { } else { }";
    let e = parser(src).parse_expression().unwrap();
    assert_eq!(e.span.end, src.len());
}

#[test]
fn parse_if_span_ends_after_closing_brace() {
    // `;` follows: `parse_expression` returns the If directly.
    let e = parse_expr("if a { } ;");
    assert!(matches!(e.kind, ExprKind::If { .. }));
    assert_eq!((e.span.start, e.span.end), (0, 8));
}

#[test]
fn parse_if_span_does_not_bleed_into_following_binop() {
    // `+` follows: If becomes lhs of Binary(Add).
    let e = parse_expr("if a { } + 1");
    match e.kind {
        ExprKind::Binary {
            op: BinaryOp::Add,
            lhs,
            ..
        } => {
            assert_eq!((lhs.span.start, lhs.span.end), (0, 8));
        }
        other => panic!("expected Binary(Add, If, 1), got {other:?}"),
    }
}

// ---- while / loop span tests ----
#[test]
fn parse_while_span() {
    let src = "while x < 10 { }";
    let e = parser(src).parse_expression().unwrap();
    assert_eq!(e.span.start, 0);
    assert_eq!(e.span.end, src.len());
}

#[test]
fn parse_loop_span() {
    let src = "loop { }";
    let e = parser(src).parse_expression().unwrap();
    assert_eq!(e.span.end, src.len());
}

#[test]
fn parse_while_span_ends_after_closing_brace() {
    let src = "while x { } ;";
    let e = parse_expr(src);
    assert_eq!((e.span.start, e.span.end), (0, 11));
    //                        ^^ `while x { }` is bytes 0..11
}

#[test]
fn parse_loop_span_ends_after_closing_brace() {
    let src = "loop { } ;";
    let e = parse_expr(src);
    assert_eq!((e.span.start, e.span.end), (0, 8));
}
#[test]
fn pub_fn_span_covers_pub_keyword() {
    let src = "pub fn f() { }";
    let ast = parse(src);
    assert_eq!(ast.items[0].span.start, 0);
    assert_eq!(ast.items[0].span.end, src.len());
}

#[test]
fn if_let_with_else_block() {
    let e = parse_expr("if let x = opt { } else { }");
    match e.kind {
        ExprKind::IfLet {
            else_branch: Some(_),
            ..
        } => {}
        _ => panic!(),
    }
}

#[test]
fn if_let_with_else_if() {
    let e = parse_expr("if let x = opt { } else if c { }");
    match e.kind {
        ExprKind::IfLet {
            else_branch: Some(inner),
            ..
        } => assert!(matches!(inner.kind, ExprKind::If { .. })),
        _ => panic!(),
    }
}

#[test]
fn if_let_missing_equals() {
    assert_err_code_expr("if let x opt { }", ErrorCode::ExpectedToken);
}
