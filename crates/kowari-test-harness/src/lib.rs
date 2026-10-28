use std::fs;
use std::path::{Path, PathBuf};

/// `<repo>/tests/golden`. `CARGO_MANIFEST_DIR` is the declaring package
/// (`crates/kowari-test-harness`), so two `..` climbs reach the repo root.
pub fn golden_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent() // crates/
        .and_then(Path::parent) // repo root
        .expect("CARGO_MANIFEST_DIR has at least two ancestors")
        .join("tests/golden")
}

/// `<repo>/tests/golden/<phase>`, e.g. `golden_dir("lexer")`.
pub fn golden_dir(phase: &str) -> PathBuf {
    golden_root().join(phase)
}

pub fn run_dir(dir: &Path, mut render: impl FnMut(&str) -> String) {
    let update = std::env::var_os("UPDATE").is_some();
    let mut paths: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("kowari"))
        .collect();
    paths.sort();

    for path in &paths {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let src = normalize_source(&fs::read_to_string(path).unwrap()); // ← added
        let actual = normalize_output(&render(&src)); // ← added
        let expected_path = dir.join(format!("{name}.expected"));

        if update {
            fs::write(&expected_path, &actual).unwrap();
        } else {
            let expected =
                normalize_output(&fs::read_to_string(&expected_path).unwrap_or_else(|_| {
                    panic!("missing {}; rerun with UPDATE=1", expected_path.display())
                })); // ← added
            assert_eq!(actual, expected, "golden mismatch for {name}");
        }
    }

    assert!(
        !paths.is_empty(),
        "no .kowari fixtures in {}",
        dir.display()
    );
}

/// Strip CRLF from fixture source before handing it to the phase.
/// Otherwise, spans computed on Windows differ from spans computed on CI.
fn normalize_source(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Canonicalize rendered output: LF only, always trailing newline.
fn normalize_output(s: &str) -> String {
    let mut s = s.replace("\r\n", "\n");
    if !s.is_empty() && !s.ends_with('\n') {
        s.push('\n');
    }
    s
}
