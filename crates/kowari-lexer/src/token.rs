use kowari_diagnostics::Span;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind<'src> {
    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LSquareBracket, // [
    RSquareBracket, // ]
    Pipe,           // |
    Ampersand,      // &
    Caret,          // ^
    HashTag,        // #
    Underscore,     // _
    AtSign,         // @
    GreaterThan,    // >
    LessThan,       // <
    Exclamation,    // !
    Colon,          // :
    Semicolon,      // ;
    Dot,            // .
    QuestionMark,   // ?
    Tilde,          // ~
    Comma,          // ,

    // Multi-Operators
    LShift,             // <<
    RShift,             // >>
    LShiftEquals,       // <<=
    RShiftEquals,       // >>=
    Equals,             // =
    DoubleEquals,       // ==
    NotEquals,          // !=
    GreaterOrEquals,    // >=
    LessOrEquals,       // <=
    DoubleAnd,          // &&
    DoublePipe,         // ||
    PlusEquals,         // +=
    MinusEquals,        // -=
    StarEquals,         // *=
    SlashEquals,        // /=
    PercentEquals,      // %=
    DoubleColon,        // ::
    DoubleDot,          // ..
    RangeInclusive,     // ..=
    SmallArrow,         // ->
    BigArrow,           // =>
    DoubleQuestionMark, // ??
    BitwiseAndEquals,   // &=
    BitwiseOrEquals,    // |=
    BitwiseXorEquals,   // ^=

    // Overflow-policy operators (see Spec §11.2)

    // Wrapping (explicit wrap-around, modulo 2^N)
    PlusWrapping,   // +%
    MinusWrapping,  // -%
    StarWrapping,   // *%
    SlashWrapping,  // /%
    LShiftWrapping, // <<%
    RShiftWrapping, // >>%

    // Checked (returns Option<T>, None on overflow)
    PlusChecked,   // +?
    MinusChecked,  // -?
    StarChecked,   // *?
    SlashChecked,  // /?
    LShiftChecked, // <<?
    RShiftChecked, // >>?

    // Saturating (clamps to min/max of the type)
    PlusSaturating,  // +|
    MinusSaturating, // -|
    StarSaturating,  // *|

    // Keywords
    Function,
    Identifier(&'src str),
    LetToken,
    MutToken,
    IfToken,
    ElseToken,
    MatchToken,
    ForToken,
    WhileToken,
    LoopToken,
    ReturnToken,
    Defer,
    ErrDefer,
    StructToken,
    EnumToken,
    TraitToken,
    ImplToken,
    PubToken,
    UnsafeToken,
    ExternToken,
    InToken,
    RangeToken,
    AsToken,
    BreakToken,
    ContinueToken,
    UseToken,
    CrateToken,
    SelfValueToken,
    SelfTypeToken,
    ConstToken,
    StaticToken,
    TypeDefToken,
    WhereToken,
    ModuleToken,
    DynToken,
    UnionToken,

    // Literals
    Integer(&'src str),
    Float(&'src str),
    String(&'src str),
    Char(&'src str),
    TrueToken,
    FalseToken,

    // End Of File
    Eof,
}

/// Type suffixes valid on numeric literals. Keep in sync with the primitive
/// types in the Spec (§3).
pub const NUMERIC_SUFFIXES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64",
];

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}..{}", self.kind, self.span.start, self.span.end)
    }
}

impl fmt::Display for TokenKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // --- simple (no payload): just the name ---
            TokenKind::Plus => f.write_str("Plus"),
            TokenKind::Minus => f.write_str("Minus"),
            TokenKind::Star => f.write_str("Star"),
            TokenKind::Slash => f.write_str("Slash"),
            TokenKind::Percent => f.write_str("Percent"),
            TokenKind::LParen => f.write_str("LParen"),
            TokenKind::RParen => f.write_str("RParen"),
            TokenKind::LBrace => f.write_str("LBrace"),
            TokenKind::RBrace => f.write_str("RBrace"),
            TokenKind::LSquareBracket => f.write_str("LSquareBracket"),
            TokenKind::RSquareBracket => f.write_str("RSquareBracket"),
            TokenKind::Pipe => f.write_str("Pipe"),
            TokenKind::Ampersand => f.write_str("Ampersand"),
            TokenKind::Caret => f.write_str("Caret"),
            TokenKind::HashTag => f.write_str("HashTag"),
            TokenKind::Underscore => f.write_str("Underscore"),
            TokenKind::AtSign => f.write_str("AtSign"),
            TokenKind::GreaterThan => f.write_str("GreaterThan"),
            TokenKind::LessThan => f.write_str("LessThan"),
            TokenKind::Exclamation => f.write_str("Exclamation"),
            TokenKind::Colon => f.write_str("Colon"),
            TokenKind::Semicolon => f.write_str("Semicolon"),
            TokenKind::Dot => f.write_str("Dot"),
            TokenKind::QuestionMark => f.write_str("QuestionMark"),
            TokenKind::Tilde => f.write_str("Tilde"),
            TokenKind::Comma => f.write_str("Comma"),
            TokenKind::LShift => f.write_str("LShift"),
            TokenKind::RShift => f.write_str("RShift"),
            TokenKind::LShiftEquals => f.write_str("LShiftEquals"),
            TokenKind::RShiftEquals => f.write_str("RShiftEquals"),
            TokenKind::Equals => f.write_str("Equals"),
            TokenKind::DoubleEquals => f.write_str("DoubleEquals"),
            TokenKind::NotEquals => f.write_str("NotEquals"),
            TokenKind::GreaterOrEquals => f.write_str("GreaterOrEquals"),
            TokenKind::LessOrEquals => f.write_str("LessOrEquals"),
            TokenKind::DoubleAnd => f.write_str("DoubleAnd"),
            TokenKind::DoublePipe => f.write_str("DoublePipe"),
            TokenKind::PlusEquals => f.write_str("PlusEquals"),
            TokenKind::MinusEquals => f.write_str("MinusEquals"),
            TokenKind::StarEquals => f.write_str("StarEquals"),
            TokenKind::SlashEquals => f.write_str("SlashEquals"),
            TokenKind::PercentEquals => f.write_str("PercentEquals"),
            TokenKind::DoubleColon => f.write_str("DoubleColon"),
            TokenKind::DoubleDot => f.write_str("DoubleDot"),
            TokenKind::RangeInclusive => f.write_str("RangeInclusive"),
            TokenKind::SmallArrow => f.write_str("SmallArrow"),
            TokenKind::BigArrow => f.write_str("BigArrow"),
            TokenKind::DoubleQuestionMark => f.write_str("DoubleQuestionMark"),
            TokenKind::BitwiseAndEquals => f.write_str("BitwiseAndEquals"),
            TokenKind::BitwiseOrEquals => f.write_str("BitwiseOrEquals"),
            TokenKind::BitwiseXorEquals => f.write_str("BitwiseXorEquals"),
            TokenKind::PlusWrapping => f.write_str("PlusWrapping"),
            TokenKind::MinusWrapping => f.write_str("MinusWrapping"),
            TokenKind::StarWrapping => f.write_str("StarWrapping"),
            TokenKind::SlashWrapping => f.write_str("SlashWrapping"),
            TokenKind::LShiftWrapping => f.write_str("LShiftWrapping"),
            TokenKind::RShiftWrapping => f.write_str("RShiftWrapping"),
            TokenKind::PlusChecked => f.write_str("PlusChecked"),
            TokenKind::MinusChecked => f.write_str("MinusChecked"),
            TokenKind::StarChecked => f.write_str("StarChecked"),
            TokenKind::SlashChecked => f.write_str("SlashChecked"),
            TokenKind::LShiftChecked => f.write_str("LShiftChecked"),
            TokenKind::RShiftChecked => f.write_str("RShiftChecked"),
            TokenKind::PlusSaturating => f.write_str("PlusSaturating"),
            TokenKind::MinusSaturating => f.write_str("MinusSaturating"),
            TokenKind::StarSaturating => f.write_str("StarSaturating"),
            TokenKind::Function => f.write_str("Function"),
            TokenKind::LetToken => f.write_str("Let"),
            TokenKind::MutToken => f.write_str("Mut"),
            TokenKind::IfToken => f.write_str("If"),
            TokenKind::ElseToken => f.write_str("Else"),
            TokenKind::MatchToken => f.write_str("Match"),
            TokenKind::ForToken => f.write_str("For"),
            TokenKind::WhileToken => f.write_str("While"),
            TokenKind::LoopToken => f.write_str("Loop"),
            TokenKind::ReturnToken => f.write_str("Return"),
            TokenKind::Defer => f.write_str("Defer"),
            TokenKind::ErrDefer => f.write_str("ErrDefer"),
            TokenKind::StructToken => f.write_str("Struct"),
            TokenKind::EnumToken => f.write_str("Enum"),
            TokenKind::TraitToken => f.write_str("Trait"),
            TokenKind::ImplToken => f.write_str("Impl"),
            TokenKind::PubToken => f.write_str("Pub"),
            TokenKind::UnsafeToken => f.write_str("Unsafe"),
            TokenKind::ExternToken => f.write_str("Extern"),
            TokenKind::InToken => f.write_str("In"),
            TokenKind::RangeToken => f.write_str("Range"),
            TokenKind::AsToken => f.write_str("As"),
            TokenKind::BreakToken => f.write_str("Break"),
            TokenKind::ContinueToken => f.write_str("Continue"),
            TokenKind::UseToken => f.write_str("Use"),
            TokenKind::CrateToken => f.write_str("Crate"),
            TokenKind::SelfValueToken => f.write_str("SelfValue"),
            TokenKind::SelfTypeToken => f.write_str("SelfType"),
            TokenKind::ConstToken => f.write_str("Const"),
            TokenKind::StaticToken => f.write_str("Static"),
            TokenKind::TypeDefToken => f.write_str("Type"),
            TokenKind::WhereToken => f.write_str("Where"),
            TokenKind::ModuleToken => f.write_str("Mod"),
            TokenKind::DynToken => f.write_str("Dyn"),
            TokenKind::UnionToken => f.write_str("Union"),
            TokenKind::TrueToken => f.write_str("True"),
            TokenKind::FalseToken => f.write_str("False"),
            TokenKind::Eof => f.write_str("Eof"),

            // --- payload: Debug the value so escapes can't break a line ---
            TokenKind::Identifier(s) => write!(f, "Identifier({s:?})"),
            TokenKind::Integer(s) => write!(f, "Integer({s:?})"),
            TokenKind::Float(s) => write!(f, "Float({s:?})"),
            TokenKind::String(s) => write!(f, "String({s:?})"),
            TokenKind::Char(s) => write!(f, "Char({s:?})"),
        }
    }
}
