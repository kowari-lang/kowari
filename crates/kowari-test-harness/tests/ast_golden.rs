use kowari_test_harness::{golden_dir, run_dir};

#[test]
#[ignore = "waiting for kowari_parser::parse (M1a)"]
fn golden_pass() {
    run_dir(&golden_dir("ast/pass"), |src| {
        let tokens = kowari_lexer::lex(src).expect("lex failed");
        let ast = kowari_parser::parse(tokens).expect("parse failed");
        format!("{ast:#?}")
    });
}
