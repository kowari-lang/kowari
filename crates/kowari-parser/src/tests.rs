use kowari_ast::item::Item;

#[test]
#[allow(irrefutable_let_patterns)] // Item currently only has Function so it's always true
fn parses_empty_function() {
    let tokens = kowari_lexer::lex("fn main() {}").expect("lex");
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
    assert_eq!(f.span.end, "fn main() {}".len());
}
