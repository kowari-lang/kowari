#[test]
fn parses_empty_function() {
    let tokens = kowari_lexer::lex("fn main() {}").expect("Failed to lex");
    let mut parser = crate::Parser::new(&tokens);
    let ast = parser.parse().expect("Failed to parse");
    assert_eq!(ast.items.len(), 1);
    // assert the item is a fn named "main" with empty body
}
