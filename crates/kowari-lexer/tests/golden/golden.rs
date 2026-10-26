use kowari_diagnostics::Diag;
use kowari_lexer::{dump_tokens, lex};
use std::fs;
use std::path::Path;

fn render(src: &str) -> String {
    match lex(src) {
        Ok(_) => dump_tokens(src).unwrap(),
        Err(diags) => {
            let mut out = String::new();
            for d in diags {
                use std::fmt::Write;
                writeln!(out, "{d}").unwrap();
            }
            out
        }
    }
}

fn run_dir(dir: &Path) {
    let update = std::env::var("UPDATE").is_ok();
    let mut any = false;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("kowari") {
            continue;
        }
        any = true;
        let expected_path = path.with_extension("expected");
        let src = fs::read_to_string(&path).unwrap();
        let actual = render(&src);
        if update {
            fs::write(&expected_path, &actual).unwrap();
        } else {
            let expected = fs::read_to_string(&expected_path).unwrap_or_default();
            assert_eq!(actual, expected, "golden mismatch in {}", path.display());
        }
    }
    assert!(any, "no .kowari files found in {}", dir.display());
}

#[test]
fn golden_pass() {
    run_dir(Path::new("tests/golden/pass"));
}

#[test]
fn golden_fail() {
    run_dir(Path::new("tests/golden/fail"));
}