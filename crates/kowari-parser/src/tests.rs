use kowari_ast::item::Item;

#[test]
#[allow(irrefutable_let_patterns)] // Item currently only has Function so it's always true
fn parses_empty_function() {
    let text = "fn main() {}";
    let tokens = kowari_lexer::lex(text).expect("lex");
    let mut parser = crate::Parser::new(&tokens);
    let ast = parser.parse().expect("parse");

    assert_eq!(ast.items.len(), 1);
    let Item::Function(f) = &ast.items[0] else {
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
    let Item::Function(f) = &ast.items[0];
    assert_eq!(f.name, "main");
    assert!(!f.params.is_empty());
    assert!(f.ret.is_some());
    assert!(!f.body.stmts.is_empty(), "Body is empty: {:?}", f.body);
    assert_eq!(f.span.start, 0);
    assert_eq!(f.span.end, text.len());
}
