use crate::DiagCode;

/// Every diagnostic the compiler can emit. Adding a variant forces you to
/// fill in `.code()` and `.description()` (exhaustive `match`), so a new
/// code can never sneak in undocumented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // --- Lexer ---
    UnexpectedToken,     // E0001
    UnterminatedString,  // E0002
    UnterminatedChar,    // E0003
    EmptyChar,           // E0004
    MultiChar,           // E0005
    BadNumericSeparator, // E0006
    RadixWithoutDigits,  // E0007
                         // --- add parser / typeck / borrow-check codes as they land ---
}

impl ErrorCode {
    /// The stable wire code. Never change this once shipped — user tooling
    /// and `--explain` links depend on it.
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnexpectedToken => "E0001",
            Self::UnterminatedString => "E0002",
            Self::UnterminatedChar => "E0003",
            Self::EmptyChar => "E0004",
            Self::MultiChar => "E0005",
            Self::BadNumericSeparator => "E0006",
            Self::RadixWithoutDigits => "E0007",
        }
    }

    /// Short human description, used by `kowaric --explain E0001` and docs.
    pub const fn description(self) -> &'static str {
        match self {
            Self::UnexpectedToken => {
                "The lexer encountered a character that does not start any token."
            }
            Self::UnterminatedString => {
                "A string literal was opened with `\"` but never closed before end of file."
            }
            Self::UnterminatedChar => "A char literal was opened with `'` but never closed.",
            Self::EmptyChar => "A char literal `''` contains no character.",
            Self::MultiChar => "A char literal contains more than one character.",
            Self::BadNumericSeparator => {
                "A numeric literal has a trailing or doubled `_` separator."
            }
            Self::RadixWithoutDigits => {
                "A radix prefix (`0x`, `0b`, `0o`) was not followed by any digits."
            }
        }
    }

    /// Enumerate everything. Drives `--explain` with no code duplication.
    pub const ALL: &'static [Self] = &[
        Self::UnexpectedToken,
        Self::UnterminatedString,
        Self::UnterminatedChar,
        Self::EmptyChar,
        Self::MultiChar,
        Self::BadNumericSeparator,
        Self::RadixWithoutDigits,
    ];

    pub fn from_code(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.code() == s)
    }
}

impl From<ErrorCode> for DiagCode {
    fn from(c: ErrorCode) -> Self {
        DiagCode(c.code())
    }
}
