//! Diagnostic codes, spans, and rendering.

mod codes;

pub use codes::ErrorCode;
use std::cmp::{max, min};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn merge(&self, new_span: Span) -> Span {
        Span::new(min(self.start, new_span.start), max(self.end, new_span.end))
    }
    pub fn at(index: usize) -> Span {
        Span::new(index, index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagCode(pub &'static str); // e.g. "E0142"

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    pub code: DiagCode,
    pub span: Span,
    pub message: String,
}

impl Span {
    pub fn length(&self) -> usize {
        self.end - self.start
    }

    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }
}

impl fmt::Display for Diag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} @ {}..{}: {}",
            self.code, self.span.start, self.span.end, self.message
        )
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
