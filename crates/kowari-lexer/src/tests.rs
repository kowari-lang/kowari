#[cfg(test)]
use crate::lex;
#[cfg(test)]
use crate::token::TokenKind;
#[cfg(test)]
use kowari_diagnostics::{Diag, ErrorCode, Span};

#[test]
fn lexes_fn_main() {
    let tokens = lex("fn main() { 1 + 2 }").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Function,
            TokenKind::Identifier("main"),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Integer("1"),
            TokenKind::Plus,
            TokenKind::Integer("2"),
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
    // Spot-check a span
    assert_eq!(tokens[1].span, Span { start: 3, end: 7 }); // "main"
}

#[test]
fn lex_unknown_token_errors() {
    let mut result = lex("fn main() { ♥ }");
    assert!(result.is_err(), "expected lexing to fail on '♥'");

    let mut diags = result.unwrap_err();
    assert_eq!(
        diags.len(),
        1,
        "expected exactly one diagnostic, got {diags:?}"
    );

    let diag = &diags[0];
    assert_eq!(diag.span, Span { start: 12, end: 15 });
    assert_eq!(
        diags,
        vec![Diag {
            code: ErrorCode::UnexpectedToken.into(),
            span: Span { start: 12, end: 15 },
            message: "Unexpected token '♥'".to_string(),
        }]
    );

    result = lex("fn main() { ♥☻○ }");
    diags = result.unwrap_err();
    assert_eq!(
        diags.len(),
        3,
        "expected exactly 3 diagnostic, got {diags:?}"
    );
}
#[test]
fn lexes_operators_without_whitespace() {
    let kinds: Vec<_> = lex("1+2").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1"),
            TokenKind::Plus,
            TokenKind::Integer("2"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_multi_char_operators() {
    let tokens =
        lex("== != <= >= && || << >>").expect("lexer should accept all valid two-char operators");

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::DoubleEquals,
            TokenKind::NotEquals,
            TokenKind::LessOrEquals,
            TokenKind::GreaterOrEquals,
            TokenKind::DoubleAnd,
            TokenKind::DoublePipe,
            TokenKind::LShift,
            TokenKind::RShift,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_shift_assign_disambiguation() {
    // `<<=` must not lex as `<<`, `=`
    let kinds: Vec<_> = lex("<<= >>= <<").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::LShiftEquals,
            TokenKind::RShiftEquals,
            TokenKind::LShift,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn span_of_two_char_operator() {
    let tokens = lex("==").unwrap();
    assert_eq!(tokens[0].span, Span { start: 0, end: 2 });
}

#[test]
fn lexes_single_char_operator_at_eof() {
    let tokens = lex("+").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[0].span, Span { start: 0, end: 1 });
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

// ----- Wrapping ------------------------------------------------------------

#[test]
fn lexes_wrapping_operators() {
    let kinds: Vec<_> = lex("+% -% *% /% <<% >>%")
        .expect("wrapping operators should lex")
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::PlusWrapping,
            TokenKind::MinusWrapping,
            TokenKind::StarWrapping,
            TokenKind::SlashWrapping,
            TokenKind::LShiftWrapping,
            TokenKind::RShiftWrapping,
            TokenKind::Eof,
        ]
    );
}

// ----- Checked -------------------------------------------------------------

#[test]
fn lexes_checked_operators() {
    let kinds: Vec<_> = lex("+? -? *? /? <<? >>?")
        .expect("checked operators should lex")
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::PlusChecked,
            TokenKind::MinusChecked,
            TokenKind::StarChecked,
            TokenKind::SlashChecked,
            TokenKind::LShiftChecked,
            TokenKind::RShiftChecked,
            TokenKind::Eof,
        ]
    );
}

// ----- Saturating ----------------------------------------------------------

#[test]
fn lexes_saturating_operators() {
    let kinds: Vec<_> = lex("+| -| *|")
        .expect("saturating operators should lex")
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::PlusSaturating,
            TokenKind::MinusSaturating,
            TokenKind::StarSaturating,
            TokenKind::Eof,
        ]
    );
}

// ----- Disambiguation ------------------------------------------------------

#[test]
fn compound_and_wrapping_are_distinct() {
    // "+=" (compound assign) must not be confused with "+%" (wrapping).
    let kinds: Vec<_> = lex("+= +%").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::PlusEquals,
            TokenKind::PlusWrapping,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn saturating_vs_pipe_and_double_pipe() {
    // "+|" must not lex as "+", "|".
    // Standalone "|" and "||" must still lex correctly.
    let kinds: Vec<_> = lex("+| | ||").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::PlusSaturating,
            TokenKind::Pipe,
            TokenKind::DoublePipe,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn saturating_requires_adjacency() {
    // With a space: "+ |" is Plus then Pipe (bitwise-or).
    let kinds: Vec<_> = lex("a + | b").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a"),
            TokenKind::Plus,
            TokenKind::Pipe,
            TokenKind::Identifier("b"),
            TokenKind::Eof,
        ]
    );

    // Without a space: "+|" is one token.
    let kinds: Vec<_> = lex("a+|b").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a"),
            TokenKind::PlusSaturating,
            TokenKind::Identifier("b"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn checked_is_a_single_token() {
    // "a +? b" — "+?" must be one token.
    let tokens = lex("a+?b").unwrap();
    assert_eq!(tokens[1].kind, TokenKind::PlusChecked);
    // "+?" is 2 bytes at offset 1..3
    assert_eq!(tokens[1].span, Span { start: 1, end: 3 });
}

// ----- Spans ---------------------------------------------------------------

#[test]
fn spans_of_overflow_policy_operators() {
    for (src, offset, expected_kind, expected_len) in [
        ("a+%b", 1, TokenKind::PlusWrapping, 2),
        ("a+?b", 1, TokenKind::PlusChecked, 2),
        ("a+|b", 1, TokenKind::PlusSaturating, 2),
        ("a-%b", 1, TokenKind::MinusWrapping, 2),
        ("a-?b", 1, TokenKind::MinusChecked, 2),
        ("a-|b", 1, TokenKind::MinusSaturating, 2),
        ("a*%b", 1, TokenKind::StarWrapping, 2),
        ("a*?b", 1, TokenKind::StarChecked, 2),
        ("a*|b", 1, TokenKind::StarSaturating, 2),
        ("a/%b", 1, TokenKind::SlashWrapping, 2),
        ("a/?b", 1, TokenKind::SlashChecked, 2),
        ("a<<%b", 1, TokenKind::LShiftWrapping, 3),
        ("a<<?b", 1, TokenKind::LShiftChecked, 3),
        ("a>>%b", 1, TokenKind::RShiftWrapping, 3),
        ("a>>?b", 1, TokenKind::RShiftChecked, 3),
    ] {
        let tokens = lex(src).unwrap();
        assert_eq!(tokens[1].kind, expected_kind, "input: {src:?}");
        assert_eq!(
            tokens[1].span,
            Span {
                start: offset,
                end: offset + expected_len
            },
            "span mismatch for {src:?}"
        );
    }
}

// ----- Edge cases ----------------------------------------------------------

#[test]
fn overflow_policy_operators_at_eof() {
    for (src, expected_kind) in [
        ("+%", TokenKind::PlusWrapping),
        ("+?", TokenKind::PlusChecked),
        ("+|", TokenKind::PlusSaturating),
        ("<<%", TokenKind::LShiftWrapping),
        (">>?", TokenKind::RShiftChecked),
    ] {
        let tokens = lex(src).unwrap();
        assert_eq!(tokens[0].kind, expected_kind, "input: {src:?}");
        assert_eq!(
            tokens[0].span,
            Span {
                start: 0,
                end: src.len()
            },
            "input: {src:?}"
        );
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }
}

#[test]
fn percent_compound_assignment_is_one_token() {
    // Regression: the original lexer passed Percent for both kinds,
    // silently swallowing the `=`.
    let kinds: Vec<_> = lex("%= %").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::PercentEquals, TokenKind::Percent, TokenKind::Eof,]
    );
}

// ----- End-to-end demo -----------------------------------------------------

#[test]
fn lexes_spec_overflow_examples() {
    // The three overflow-policy examples from Spec §11.2.
    let kinds: Vec<_> = lex("250 +% 10  250 +? 10  250 +| 10")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("250"),
            TokenKind::PlusWrapping,
            TokenKind::Integer("10"),
            TokenKind::Integer("250"),
            TokenKind::PlusChecked,
            TokenKind::Integer("10"),
            TokenKind::Integer("250"),
            TokenKind::PlusSaturating,
            TokenKind::Integer("10"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn dot_and_range() {
    let kinds: Vec<_> = lex(". .. ..= in range")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Dot,
            TokenKind::DoubleDot,
            TokenKind::RangeInclusive,
            TokenKind::InToken,
            TokenKind::RangeToken,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn integer_separators_are_preserved() {
    let kinds: Vec<_> = lex("1_000_000").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![TokenKind::Integer("1_000_000"), TokenKind::Eof]);
}

#[test]
fn underscore_leading_identifiers_are_not_integers() {
    // A leading '_' must produce an Identifier, not an Integer.
    let kinds: Vec<_> = lex("_foo _ _1").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("_foo"),
            TokenKind::Identifier("_"),
            TokenKind::Identifier("_1"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn trailing_underscore_is_rejected() {
    // "1_" — a trailing separator is not valid in any reasonable grammar.
    let result = lex("1_");
    assert!(result.is_err(), "trailing '_' should be a diagnostic");
    let diags = result.unwrap_err();
    assert_eq!(diags[0].code, ErrorCode::BadNumericSeparator.into(),);
    assert_eq!(diags[0].span, Span { start: 1, end: 2 });
}

#[test]
fn double_underscore_is_rejected() {
    let result = lex("1__2");
    assert!(result.is_err(), "double '_' should be a diagnostic");
    let diags = result.unwrap_err();
    assert_eq!(diags[0].code, ErrorCode::BadNumericSeparator.into(),);
}

#[test]
fn lexes_simple_char() {
    let tokens = lex("'a'").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Char("a"));
    assert_eq!(tokens[0].span, Span { start: 0, end: 3 });
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn lexes_escaped_char() {
    let tokens = lex(r"'\n'").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Char(r"\n"));
    assert_eq!(tokens[0].span, Span { start: 0, end: 4 });
}

#[test]
fn lexes_escaped_quote_char() {
    // '\'' — escape is \' , closing quote is the final '
    let tokens = lex(r"'\''").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Char(r"\'"));
    assert_eq!(tokens[0].span, Span { start: 0, end: 4 });
}

#[test]
fn empty_char_is_an_error() {
    let diags = lex("''").unwrap_err();
    assert_eq!(diags.len(), 1, "got: {diags:?}");
    assert_eq!(diags[0].code, ErrorCode::EmptyChar.into(),);
    assert_eq!(diags[0].span, Span { start: 0, end: 2 });
}

#[test]
fn multi_char_is_an_error_and_recovers() {
    // 'ab' — should consume through the closing quote and produce one diag.
    let diags = lex("'ab' + 1").unwrap_err();
    assert_eq!(diags.len(), 1, "got: {diags:?}");
    assert_eq!(diags[0].code, ErrorCode::MultiChar.into());
    assert_eq!(diags[0].span, Span { start: 0, end: 4 });
}

#[test]
fn unterminated_char_is_an_error() {
    let diags = lex("'a").unwrap_err();
    assert_eq!(diags.len(), 1, "got: {diags:?}");
    assert_eq!(diags[0].code, ErrorCode::UnterminatedChar.into());
}

#[test]
fn bare_quote_at_eof_is_an_error() {
    let diags = lex("'").unwrap_err();
    assert_eq!(diags.len(), 1, "got: {diags:?}");
    assert_eq!(diags[0].code, ErrorCode::UnterminatedChar.into());
}

#[test]
fn unterminated_escape_is_an_error() {
    let diags = lex(r"'\").unwrap_err();
    assert_eq!(diags.len(), 1, "got: {diags:?}");
    assert_eq!(diags[0].code, ErrorCode::UnterminatedChar.into());
}

#[test]
fn string_content_is_correct_with_multibyte_chars() {
    // é is 2 bytes; the slice must not land mid-codepoint.
    let tokens = lex("\"héllo\"").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::String("héllo"));
    assert_eq!(tokens[0].span, Span { start: 0, end: 8 });

    let tokens = lex("\"é\"").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::String("é"));
    assert_eq!(tokens[0].span, Span { start: 0, end: 4 });
}

#[test]
fn string_span_includes_both_quotes() {
    let tokens = lex("\"abc\"").unwrap();
    assert_eq!(tokens[0].span, Span { start: 0, end: 5 });
    assert_eq!(tokens[0].kind, TokenKind::String("abc"));
}

#[test]
fn lexes_arrows() {
    let kinds: Vec<_> = lex("-> =>").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::SmallArrow, TokenKind::BigArrow, TokenKind::Eof]
    );
}

#[test]
fn arrow_spans() {
    let tokens = lex("->").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::SmallArrow);
    assert_eq!(tokens[0].span, Span { start: 0, end: 2 });

    let tokens = lex("=>").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::BigArrow);
    assert_eq!(tokens[0].span, Span { start: 0, end: 2 });
}

#[test]
fn arrow_is_not_minus_then_greater() {
    // Regression: without the '>' suffix on '-', this used to lex as
    // [Minus, BiggerThen].
    let kinds: Vec<_> = lex("->").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::SmallArrow, TokenKind::Eof],
        "-> must be one token, not Minus + BiggerThen"
    );
}

#[test]
fn fat_arrow_is_not_equals_then_greater() {
    let kinds: Vec<_> = lex("=>").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::BigArrow, TokenKind::Eof],
        "=> must be one token, not Equals + BiggerThen"
    );
}

#[test]
fn minus_arrow_does_not_break_arithmetic() {
    // The '-' arm now has five suffixes; make sure none shadow each other.
    let kinds: Vec<_> = lex("-= -% -? -| ->")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::MinusEquals,
            TokenKind::MinusWrapping,
            TokenKind::MinusChecked,
            TokenKind::MinusSaturating,
            TokenKind::SmallArrow,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn equals_arrow_does_not_break_equality() {
    let kinds: Vec<_> = lex("= == =>").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Equals,
            TokenKind::DoubleEquals,
            TokenKind::BigArrow,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn fn_signature_with_return_arrow() {
    // End-to-end: the exact shape every parser test will need.
    let kinds: Vec<_> = lex("fn f() -> i32")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Function,
            TokenKind::Identifier("f"),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::SmallArrow,
            TokenKind::Identifier("i32"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn match_arm_with_fat_arrow() {
    let kinds: Vec<_> = lex("1 => 2").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1"),
            TokenKind::BigArrow,
            TokenKind::Integer("2"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_floats() {
    let kinds: Vec<_> = lex("3.14 1e10 1.5e-3 2E+8 0.0")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Float("3.14"),
            TokenKind::Float("1e10"),
            TokenKind::Float("1.5e-3"),
            TokenKind::Float("2E+8"),
            TokenKind::Float("0.0"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn range_is_not_a_float() {
    let kinds: Vec<_> = lex("1..2").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1"),
            TokenKind::DoubleDot,
            TokenKind::Integer("2"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn float_then_dot_recovers() {
    let kinds: Vec<_> = lex("1.5.6").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Float("1.5"),
            TokenKind::Dot,
            TokenKind::Integer("6"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn radix_literals() {
    let kinds: Vec<_> = lex("0xFF 0b1010 0o777 0xFF_FF")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("0xFF"),
            TokenKind::Integer("0b1010"),
            TokenKind::Integer("0o777"),
            TokenKind::Integer("0xFF_FF"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn radix_without_digits_is_an_error() {
    let diags = lex("0x").unwrap_err();
    assert_eq!(diags[0].code, ErrorCode::RadixWithoutDigits.into());
}

// ----- Comments ----------------------------------------------------------

#[test]
fn line_comment_skips_to_eol() {
    let kinds: Vec<_> = lex("1 // c\n2").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1"),
            TokenKind::Integer("2"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn line_comment_at_eof() {
    let kinds: Vec<_> = lex("1 // x").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![TokenKind::Integer("1"), TokenKind::Eof]);
}

#[test]
fn block_comment_inline() {
    let kinds: Vec<_> = lex("1 /* x */ 2").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1"),
            TokenKind::Integer("2"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn nested_block_comment() {
    let kinds: Vec<_> = lex("1 /* a /* b */ c */ 2")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1"),
            TokenKind::Integer("2"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn adjacent_block_comments() {
    let kinds: Vec<_> = lex("/*a*//*b*/1").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![TokenKind::Integer("1"), TokenKind::Eof]);
}

#[test]
fn unterminated_block_comment_is_error() {
    let diags = lex("1 /* never ends").unwrap_err();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, ErrorCode::UnterminatedBlockComment.into());
    assert_eq!(diags[0].span, Span { start: 2, end: 15 });
}

#[test]
fn division_still_works_next_to_comments() {
    // `/` not followed by `/` or `*` must still be a Slash token.
    let kinds: Vec<_> = lex("a / b // c").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a"),
            TokenKind::Slash,
            TokenKind::Identifier("b"),
            TokenKind::Eof,
        ]
    );
}

// ----- Numeric suffixes --------------------------------------------------

#[test]
fn integer_suffixes() {
    let kinds: Vec<_> = lex("1u32 42i64 0xFFu8 0b1010usize")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1u32"),
            TokenKind::Integer("42i64"),
            TokenKind::Integer("0xFFu8"),
            TokenKind::Integer("0b1010usize"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn float_suffixes() {
    let kinds: Vec<_> = lex("1.5f32 3.14f64 1e10f32 2E+8f64")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Float("1.5f32"),
            TokenKind::Float("3.14f64"),
            TokenKind::Float("1e10f32"),
            TokenKind::Float("2E+8f64"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn suffix_separated_by_underscore() {
    let kinds: Vec<_> = lex("1_u32 1_000_u32")
        .unwrap()
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1_u32"),
            TokenKind::Integer("1_000_u32"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn incomplete_suffix_is_not_absorbed() {
    // `u32x` is not a suffix — number must end at `1`, ident starts at `u`.
    let kinds: Vec<_> = lex("1u32x").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1"),
            TokenKind::Identifier("u32x"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn unknown_suffix_is_not_absorbed() {
    let kinds: Vec<_> = lex("1foo").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer("1"),
            TokenKind::Identifier("foo"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn trailing_underscore_still_errors_with_suffix_following() {
    // `1__u32` — the `__` is caught before the suffix is considered.
    let diags = lex("1__u32").unwrap_err();
    assert_eq!(diags[0].code, ErrorCode::BadNumericSeparator.into());
}

#[test]
fn empty_radix_is_still_an_error_even_with_underscore() {
    // Regression: `0x_` used to slip past the "no digits" check because
    // the span was non-empty (it contained the `_`).
    let diags = lex("0x_").unwrap_err();
    assert_eq!(diags[0].code, ErrorCode::RadixWithoutDigits.into());
}

#[test]
fn unterminated_char_on_line_stops_at_newline() {
    // `'a` followed by more source: the recovery loop must stop at the
    // newline, not swallow the rest of the file looking for a quote.
    let diags = lex("'a\nlet x = 1;").unwrap_err();
    assert_eq!(diags.len(), 1, "got: {diags:?}");
    assert_eq!(diags[0].code, ErrorCode::UnterminatedChar.into());
    assert_eq!(diags[0].span, Span { start: 0, end: 2 });
}

// ----- Additional coverage (M0 review) -------------------------------------

#[test]
fn empty_input_produces_only_eof() {
    let tokens = lex("").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
    assert_eq!(tokens[0].span, Span { start: 0, end: 0 });
}

#[test]
fn whitespace_only_input_produces_only_eof() {
    let tokens = lex("   ").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
    assert_eq!(tokens[0].span, Span { start: 3, end: 3 });
}

#[test]
fn crlf_is_treated_as_whitespace() {
    let kinds: Vec<_> = lex("fn\r\nmain").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Function,
            TokenKind::Identifier("main"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn unicode_identifiers_are_accepted() {
    // Provisional M0 behaviour: identifiers use is_alphabetic() /
    // is_alphanumeric(), so Unicode letters are accepted. If the spec
    // later restricts identifiers, update this test to expect diags.
    let kinds: Vec<_> = lex("café λ _x1").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("café"),
            TokenKind::Identifier("λ"),
            TokenKind::Identifier("_x1"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn string_with_escaped_quote() {
    // Source text: "a\"b"  — content must include the escaped quote.
    let tokens = lex(r#""a\"b""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::String(r#"a\"b"#));
    assert_eq!(tokens[0].span, Span { start: 0, end: 6 });
}

#[test]
fn string_with_actual_newline_is_accepted() {
    // Provisional M0 behaviour: escapes are not validated, so a raw
    // newline inside a string is currently accepted. Pin it so any
    // future tightening is deliberate.
    let src = "\"a\nb\"";
    let tokens = lex(src).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::String("a\nb"));
    assert_eq!(tokens[0].span, Span { start: 0, end: 5 });
}

#[test]
fn unterminated_string_span_covers_input() {
    let diags = lex("\"abc").unwrap_err();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, ErrorCode::UnterminatedString.into());
    assert_eq!(diags[0].span, Span { start: 0, end: 4 });
}

#[test]
fn bare_double_quote_at_eof_is_an_error() {
    let diags = lex("\"").unwrap_err();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, ErrorCode::UnterminatedString.into());
    assert_eq!(diags[0].span, Span { start: 0, end: 1 });
}

#[test]
fn multi_char_at_eof_is_unterminated() {
    // `'ab` — no closing quote, no newline; recovery hits EOF.
    let diags = lex("'ab").unwrap_err();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, ErrorCode::UnterminatedChar.into());
    assert_eq!(diags[0].span, Span { start: 0, end: 3 });
}

#[test]
fn exponent_with_underscore() {
    let kinds: Vec<_> = lex("1e1_0").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![TokenKind::Float("1e1_0"), TokenKind::Eof]);
}

#[test]
fn double_minus_lexes_as_two_tokens() {
    // The spec has no `--` token; document that it is two Minus tokens.
    let kinds: Vec<_> = lex("--").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::Minus, TokenKind::Minus, TokenKind::Eof]
    );
}

#[test]
fn triple_ampersand_is_double_and_then_single() {
    let kinds: Vec<_> = lex("&&&").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::DoubleAnd, TokenKind::Ampersand, TokenKind::Eof,]
    );
}

#[test]
fn double_question_mark_and_trailing_question() {
    let kinds: Vec<_> = lex("?? ???").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::DoubleQuestionMark,
            TokenKind::DoubleQuestionMark,
            TokenKind::QuestionMark,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn range_inclusive_requires_adjacency() {
    let kinds: Vec<_> = lex("..= .. =").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::RangeInclusive,
            TokenKind::DoubleDot,
            TokenKind::Equals,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn block_comment_slash_star_slash_is_unterminated() {
    // `/*/` — the initial `/*` opens a comment; the trailing `/` is not
    // a closing `*/`, so this is unterminated.
    let diags = lex("/*/").unwrap_err();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, ErrorCode::UnterminatedBlockComment.into());
    assert_eq!(diags[0].span, Span { start: 0, end: 3 });
}

#[test]
fn block_comment_star_star_slash_is_empty() {
    // `/**/` — an empty block comment; no off-by-one depth error.
    let kinds: Vec<_> = lex("/**/ 1").unwrap().iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![TokenKind::Integer("1"), TokenKind::Eof]);
}

#[test]
fn multiple_errors_on_one_line() {
    // Two unexpected tokens with a valid `+` between them; the lexer
    // must report both rather than stopping at the first.
    let diags = lex("♥ + ♥").unwrap_err();
    assert_eq!(diags.len(), 2, "got: {diags:?}");
    assert_eq!(diags[0].code, ErrorCode::UnexpectedToken.into());
    assert_eq!(diags[1].code, ErrorCode::UnexpectedToken.into());
    // ♥ is 3 bytes; the second one starts after "♥ + " (6 bytes).
    assert_eq!(diags[0].span, Span { start: 0, end: 3 });
    assert_eq!(diags[1].span, Span { start: 6, end: 9 });
}

#[test]
fn single_char_operators_at_eof() {
    for (src, expected_kind) in [
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("%", TokenKind::Percent),
        ("<", TokenKind::LessThan),
        (">", TokenKind::GreaterThan),
        ("=", TokenKind::Equals),
        ("!", TokenKind::Exclamation),
        ("&", TokenKind::Ampersand),
        ("|", TokenKind::Pipe),
        (":", TokenKind::Colon),
        ("^", TokenKind::Caret),
        ("?", TokenKind::QuestionMark),
        (".", TokenKind::Dot),
    ] {
        let tokens = lex(src).unwrap();
        assert_eq!(tokens[0].kind, expected_kind, "input: {src:?}");
        assert_eq!(
            tokens[0].span,
            Span {
                start: 0,
                end: src.len()
            },
            "input: {src:?}"
        );
        assert_eq!(tokens[1].kind, TokenKind::Eof, "input: {src:?}");
    }
}
