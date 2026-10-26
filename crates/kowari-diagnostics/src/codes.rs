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
            Self::UnterminatedBlockComment => "E0008",
            Self::ExpectedToken => "E0009",
            Self::ExpectedIdent => "E0010",
            Self::UnexpectedEof => "E0011",
            Self::UnexpectedItem => "E0012",
            Self::UnterminatedBlock => "E0013",
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

            Self::UnterminatedBlockComment => {
                "A block comment was opened with `/*` but never closed with `*/`."
            }

            Self::ExpectedToken => "expected `X`, found `Y`",
            Self::ExpectedIdent => "expected an identifier, found `Y`",
            Self::UnexpectedEof => "unexpected end of input; expected `X`",
            Self::UnexpectedItem => "expected an item, found `Y`",
            Self::UnterminatedBlock => "Unterminated block, expected a closing '}'",
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
