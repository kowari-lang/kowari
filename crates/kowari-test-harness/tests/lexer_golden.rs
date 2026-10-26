use kowari_lexer::{dump_tokens, lex};
use kowari_test_harness::{golden_dir, run_dir};
use std::fs;
use std::path::{Path, PathBuf};

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

fn normalize_source(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn regenerate(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("kowari") {
            continue;
        }
        let expected_path = path.with_extension("expected");
        let src = normalize_source(&fs::read_to_string(&path).unwrap());
        fs::write(&expected_path, normalize(&render(&src))).unwrap();
    }
}

#[test]
#[ignore = "regenerates .expected files; run with: cargo test -- --ignored"]
fn update_golden() {
    regenerate(&fixture_root().join("pass"));
    regenerate(&fixture_root().join("fail"));
}

#[test]
fn golden_pass() {
    run_dir(&golden_dir("lexer/pass"), |src| {
        lex(src).expect("expected lex success");
        dump_tokens(src).expect("dump_tokens failed")
    });
}

#[test]
fn golden_fail() {
    run_dir(&golden_dir("lexer/fail"), |src| {
        let errs = lex(src).expect_err("expected lex failure");
        errs.iter()
            .map(|e| format!("{e}"))
            .collect::<Vec<_>>()
            .join("\n")
    });
}
