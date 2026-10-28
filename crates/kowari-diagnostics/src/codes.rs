use crate::DiagCode;

/// Every diagnostic the compiler can emit. Adding a variant forces you to
/// fill in `.code()` and `.description()` (exhaustive `match`), so a new
/// code can never sneak in undocumented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // --- Lexer ---
    UnexpectedToken,          // E0001
    UnterminatedString,       // E0002
    UnterminatedChar,         // E0003
    EmptyChar,                // E0004
    MultiChar,                // E0005
    BadNumericSeparator,      // E0006
    RadixWithoutDigits,       // E0007
    UnterminatedBlockComment, // E0008
    ExpectedToken,            // E0009  "expected `X`, found `Y`"
    ExpectedIdent,            // E0010  "expected an identifier, found `Y`"
    UnexpectedEof,            // E0011  "unexpected end of input; expected `X`"
    UnexpectedItem,           // E0012  "expected an item, found `Y`"
    UnterminatedBlock,        // E0013  "No closing RightBrace:
    InvalidStatement,         // E0014
    NonAssociativeOperator,   // E0015
                              // --- add parser / typeck / borrow-check codes as they land ---
}

impl ErrorCode {
    /// The stable wire code. Never change this once shipped — user tooling
    /// and `--explain` links depend on it.
    pub const fn code(self) -> &'static str {
        use ErrorCode::*;
        match self {
            UnexpectedToken => "E0001",
            UnterminatedString => "E0002",
            UnterminatedChar => "E0003",
            EmptyChar => "E0004",
            MultiChar => "E0005",
            BadNumericSeparator => "E0006",
            RadixWithoutDigits => "E0007",
            UnterminatedBlockComment => "E0008",
            ExpectedToken => "E0009",
            ExpectedIdent => "E0010",
            UnexpectedEof => "E0011",
            UnexpectedItem => "E0012",
            UnterminatedBlock => "E0013",
            InvalidStatement => "E0014",
            NonAssociativeOperator => "E0015",
        }
    }

    /// Short human description, used by `kowaric --explain E0001` and docs.
    pub const fn description(self) -> &'static str {
        use ErrorCode::*;
        match self {
            UnexpectedToken => "The lexer encountered a character that does not start any token.",
            UnterminatedString => {
                "A string literal was opened with `\"` but never closed before end of file."
            }
            UnterminatedChar => "A char literal was opened with `'` but never closed.",
            EmptyChar => "A char literal `''` contains no character.",
            MultiChar => "A char literal contains more than one character.",
            BadNumericSeparator => "A numeric literal has a trailing or doubled `_` separator.",
            RadixWithoutDigits => {
                "A radix prefix (`0x`, `0b`, `0o`) was not followed by any digits."
            }

            UnterminatedBlockComment => {
                "A block comment was opened with `/*` but never closed with `*/`."
            }

            ExpectedToken => "expected `X`, found `Y`",
            ExpectedIdent => "expected an identifier, found `Y`",
            UnexpectedEof => "unexpected end of input; expected `X`",
            UnexpectedItem => "expected an item, found `Y`",
            UnterminatedBlock => "Unterminated block, expected a closing '}'",
            InvalidStatement => "invalid statement",
            NonAssociativeOperator => {
                "A non-associative operator was chained, e.g. `a..b..c` or `a == b == c`."
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
        Self::UnterminatedBlockComment,
        Self::ExpectedToken,
        Self::ExpectedIdent,
        Self::UnexpectedEof,
        Self::UnexpectedItem,
        Self::UnterminatedBlock,
        Self::InvalidStatement,
        Self::NonAssociativeOperator,
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

#[cfg(test)]
mod tests {
    use super::ErrorCode;
    use std::collections::HashSet;

    #[test]
    fn every_code_is_unique() {
        let mut seen = HashSet::new();
        for c in ErrorCode::ALL {
            assert!(seen.insert(c.code()), "duplicate code {}", c.code());
        }
    }

    #[test]
    fn every_code_has_a_description() {
        for c in ErrorCode::ALL {
            assert!(
                !c.description().is_empty(),
                "{} has no description",
                c.code()
            );
        }
    }

    #[test]
    /// Checks if an error code is correctly formed, starting with E, then 4x a number
    fn code_format_malformed_error_codes() {
        for c in ErrorCode::ALL {
            let s = c.code();
            assert!(
                s.len() == 5 && s.starts_with('E') && s[1..].chars().all(|x| x.is_ascii_digit()),
                "malformed code {s}"
            );
        }
    }

    #[test]
    fn round_trip() {
        for c in ErrorCode::ALL {
            assert_eq!(ErrorCode::from_code(c.code()), Some(*c));
        }
    }
}
