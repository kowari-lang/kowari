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
    format!("{ast:#?}")
}

fn normalize_source(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
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
        let src = normalize_source(&fs::read_to_string(&path).unwrap());
        let actual = render(&src);

        if update {
            fs::write(&expected_path, normalize(&actual)).unwrap();
            continue;
        }

        let expected = normalize(&fs::read_to_string(&expected_path).unwrap_or_default());
        assert_eq!(actual, expected, "golden mismatch in {}", path.display());
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
        let src = normalize_source(&fs::read_to_string(&path).unwrap());
        fs::write(&expected_path, normalize(&render(&src))).unwrap();
    }
}

fn check(name: &str, actual: &str) {
    let path = Path::new("").join(format!("{name}.expected.ast"));
    if std::env::var_os("UPDATE_EXPECT").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).ok();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing {path:?}; rerun with UPDATE_EXPECT=1"));
    assert_eq!(actual, expected, "AST golden mismatch for {name}");
}

#[test]
fn golden_pass() {
    run_dir(&fixture_root().join("pass"));
}

#[test]
fn golden_fail() {
    run_dir(&fixture_root().join("fail"));
}

#[test]
#[ignore = "regenerates .expected files; run with: cargo test -- --ignored"]
fn update_golden() {
    regenerate(&fixture_root().join("pass"));
    regenerate(&fixture_root().join("fail"));
}
