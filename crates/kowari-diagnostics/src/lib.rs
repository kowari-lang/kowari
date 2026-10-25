//! Diagnostic codes, spans, and rendering.

mod codes;
pub use codes::ErrorCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagCode(pub &'static str); // e.g. "E0142"

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    pub code: DiagCode,
    pub span: Span,
    pub message: String,
}
