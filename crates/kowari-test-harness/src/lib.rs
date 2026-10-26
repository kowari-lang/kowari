use std::fs;
use std::path::{Path, PathBuf};

/// `<repo>/tests/golden`. `CARGO_MANIFEST_DIR` is the declaring package
/// (`crates/kowari-test-harness`), so two `..` climbs reach the repo root.
pub fn golden_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden")
}

/// `<repo>/tests/golden/<phase>`, e.g. `golden_dir("lexer")`.
pub fn golden_dir(phase: &str) -> PathBuf {
    golden_root().join(phase)
}

/// Run every `dir/*.kowari` through `render` and compare against
/// `<name>.expected` in the same directory. `UPDATE=1` rewrites instead.
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
        let src = fs::read_to_string(path).unwrap();
        let actual = render(&src);
        let expected_path = dir.join(format!("{name}.expected"));

        if update {
            fs::write(&expected_path, actual).unwrap();
        } else {
            let expected = fs::read_to_string(&expected_path).unwrap_or_else(|_| {
                panic!("missing {}; rerun with UPDATE=1", expected_path.display())
            });
            assert_eq!(actual, expected, "golden mismatch for {name}");
        }
    }

    assert!(!paths.is_empty(), "no .kowari fixtures in {}", dir.display());
}