use kowari_lexer::{dump_tokens, lex};
use std::fs;
use std::path::Path;

/// Canonicalize text so editor/git line-ending changes can't fail the diff.
/// `dump_tokens` always emits LF and always ends with a newline, so match that.
fn normalize(s: &str) -> String {
    let mut s = s.replace("\r\n", "\n");
    if !s.is_empty() && !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

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
        let expected = normalize(&fs::read_to_string(&expected_path).unwrap_or_default());
        let src = fs::read_to_string(&path).unwrap();
        let actual = render(&src);
        assert_eq!(actual, expected, "golden mismatch in {}", path.display());
        if update {
            fs::write(&expected_path, normalize(&actual)).unwrap();
            continue;
        } else {
            let expected = fs::read_to_string(&expected_path).unwrap_or_default();
            assert_eq!(actual, expected, "golden mismatch in {}", path.display());
        }
    }
    if !any {
        eprintln!("warning: no .kowari files in {}", dir.display());
    }
}

fn regenerate(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("kowari") {
            continue;
        }
        let expected_path = path.with_extension("expected");
        let src = fs::read_to_string(&path).unwrap();
        fs::write(&expected_path, normalize(&render(&src))).unwrap();
    }
}

#[test]
fn golden_pass() {
    run_dir(Path::new("tests/golden/pass"));
}

#[test]
fn golden_fail() {
    run_dir(Path::new("tests/golden/fail"));
}

#[test]
#[ignore = "regenerates .expected files; run with: cargo test -- --ignored"]
fn update_golden() {
    regenerate(Path::new("tests/golden/pass"));
    regenerate(Path::new("tests/golden/fail"));
}
