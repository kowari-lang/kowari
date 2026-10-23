//! Golden-test runner. Discovers `tests/compile_pass/` and `tests/compile_fail/`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HarnessError {
    /// Placeholder — replaced when the harness is implemented (see PLANNING.md §5, M0).
    #[error("test harness not yet implemented")]
    NotImplemented,
}

pub fn run_all() -> Result<(), HarnessError> {
    // TODO: M0 — discover .kowari files, run them, report.
    //       Currently a stub; the real implementation is the last piece of M0.
    Ok(())
}
