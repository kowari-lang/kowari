use kowari_parser::Parser;
use kowari_test_harness::{golden_dir, run_dir};

#[test]
fn golden_pass() {
    run_dir(&golden_dir("ast/pass"), |src| {
        let tokens = kowari_lexer::lex(src).expect("lex failed");
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().expect("parse failed");
        format!("{ast:#?}")
    });
}
