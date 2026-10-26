use kowari_test_harness::{golden_dir, run_dir};

#[test]
fn golden_pass() {
    run_dir(&golden_dir("ast/pass"), |src| {
        let tokens = kowari_lexer::lex(src).expect("lex failed");
        let ast = kowari_parser::parse(tokens).expect("parse failed");
        format!("{ast:#?}")
    });
}
