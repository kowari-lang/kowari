//! Diagnostic codes, spans, and rendering.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagCode(pub &'static str); // e.g. "E0142"

#[derive(Debug, Clone)]
pub struct Diag {
    pub code: DiagCode,
    pub span: Span,
    pub message: String,
}