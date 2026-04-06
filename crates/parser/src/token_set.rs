use crate::cst::SyntaxKind;

pub struct TokenSet(u128);

impl TokenSet {
    const fn new(kinds: &[SyntaxKind]) -> TokenSet {
        let mut bitset = 0u128;
        let mut i = 0;
        while i < kinds.len() {
            bitset |= mask(kinds[i]);
            i += 1;
        }
        TokenSet(bitset)
    }

    const fn besides(kinds: &[SyntaxKind]) -> TokenSet {
        let mut bitset = u128::MAX;
        let mut i = 0;
        while i < kinds.len() {
            bitset &= !mask(kinds[i]);
            i += 1;
        }
        TokenSet(bitset)
    }

    const fn union(&self, other: TokenSet) -> TokenSet {
        TokenSet(self.0 | other.0)
    }

    const fn difference(&self, other: TokenSet) -> TokenSet {
        TokenSet(self.0 & !other.0)
    }

    pub const fn contains(&self, kind: SyntaxKind) -> bool {
        self.0 & mask(kind) != 0
    }
}

const fn mask(kind: SyntaxKind) -> u128 {
    assert!(
        (kind as u16) < (SyntaxKind::__LastToken as u16),
        "Provided SyntaxKind is not a valid token kind"
    );
    1u128 << (kind as u16)
}

pub const ALWAYS_RECOVER: TokenSet = TokenSet::new(&[
    SyntaxKind::Eof,
    SyntaxKind::OpenBrace,
    SyntaxKind::CloseBrace,
]);

pub const END_OF_BLOCK: TokenSet = TokenSet::new(&[SyntaxKind::Eof, SyntaxKind::CloseBrace]);

pub const END_OF_STATEMENT: TokenSet = TokenSet::new(&[
    SyntaxKind::Eof,
    SyntaxKind::CloseBrace,
    SyntaxKind::Semicolon,
]);

pub const END_OF_CASE_CLAUSE: TokenSet = TokenSet::new(&[
    SyntaxKind::Eof,
    SyntaxKind::CloseBrace,
    SyntaxKind::CaseKeyword,
    SyntaxKind::DefaultKeyword,
]);

pub const NAME: TokenSet = TokenSet::new(&[SyntaxKind::Identifier, SyntaxKind::ConstructorKeyword]);

pub const ASSIGNMENT_OPERATORS: TokenSet = TokenSet::new(&[
    SyntaxKind::Equals,
    SyntaxKind::PlusEquals,
    SyntaxKind::MinusEquals,
    SyntaxKind::AsteriskEquals,
    SyntaxKind::SlashEquals,
    SyntaxKind::PercentEquals,
    SyntaxKind::LessThanMinus,
]);

pub const BINARY_OPERATORS: TokenSet = TokenSet::new(&[
    SyntaxKind::BarBar,
    SyntaxKind::AmpersandAmpersand,
    SyntaxKind::Bar,
    SyntaxKind::Caret,
    SyntaxKind::Ampersand,
    SyntaxKind::EqualsEquals,
    SyntaxKind::ExclamationEquals,
    SyntaxKind::LessThanEqualsGreaterThan,
    SyntaxKind::LessThan,
    SyntaxKind::GreaterThan,
    SyntaxKind::LessThanEquals,
    SyntaxKind::GreaterThanEquals,
    SyntaxKind::InstanceOfKeyword,
    SyntaxKind::InKeyword,
    SyntaxKind::LessThanLessThan,
    SyntaxKind::GreaterThanGreaterThan,
    SyntaxKind::GreaterThanGreaterThanGreaterThan,
    SyntaxKind::Plus,
    SyntaxKind::Minus,
    SyntaxKind::Asterisk,
    SyntaxKind::Slash,
    SyntaxKind::Percent,
]);

pub const PREFIX_UNARY_OPERATORS: TokenSet = TokenSet::new(&[
    SyntaxKind::Minus,
    SyntaxKind::Exclamation,
    SyntaxKind::Tilde,
]);

pub const UPDATE_OPERATORS: TokenSet =
    TokenSet::new(&[SyntaxKind::PlusPlus, SyntaxKind::MinusMinus]);

pub const INIT_OPERATORS: TokenSet = TokenSet::new(&[
    SyntaxKind::Equals,
    SyntaxKind::Colon,
    SyntaxKind::LessThanMinus,
]);

pub const SEPARATORS: TokenSet = TokenSet::new(&[SyntaxKind::Comma, SyntaxKind::Semicolon]);

pub const EXPRESSIONS: TokenSet = TokenSet::new(&[
    SyntaxKind::Minus,
    SyntaxKind::Tilde,
    SyntaxKind::Exclamation,
    SyntaxKind::At,
    SyntaxKind::MinusMinus,
    SyntaxKind::PlusPlus,
    SyntaxKind::ColonColon,
    SyntaxKind::CloneKeyword,
    SyntaxKind::DeleteKeyword,
    SyntaxKind::TypeOfKeyword,
    SyntaxKind::ResumeKeyword,
    SyntaxKind::ThisKeyword,
    SyntaxKind::BaseKeyword,
    SyntaxKind::FileKeyword,
    SyntaxKind::LineKeyword,
    SyntaxKind::DecimalInteger,
    SyntaxKind::Character,
    SyntaxKind::Float,
    SyntaxKind::TrueKeyword,
    SyntaxKind::FalseKeyword,
    SyntaxKind::NullKeyword,
    SyntaxKind::String,
    SyntaxKind::VerbatimString,
    SyntaxKind::OpenBrace,
    SyntaxKind::OpenParenthesis,
    SyntaxKind::ClassKeyword,
    SyntaxKind::RawCallKeyword,
    SyntaxKind::Identifier,
    SyntaxKind::ConstructorKeyword,
    SyntaxKind::FunctionKeyword,
    SyntaxKind::OpenBracket,
]);

pub const MEMBER_FIRST: TokenSet = TokenSet::new(&[
    SyntaxKind::Identifier,
    SyntaxKind::ConstructorKeyword,
    SyntaxKind::FunctionKeyword,
    SyntaxKind::OpenBracket,
    SyntaxKind::StaticKeyword,
]);

pub const NON_MEMBER_FIRST_EXPRESSIONS: TokenSet = EXPRESSIONS.difference(MEMBER_FIRST);

pub const MEMBER_RECOVERY: TokenSet = NON_MEMBER_FIRST_EXPRESSIONS.union(INIT_OPERATORS);

// Other expressions make no sense when used as statements
pub const COMMON_EXPRESSION_STATEMENTS: TokenSet = TokenSet::new(&[
    SyntaxKind::Identifier,
    SyntaxKind::ConstructorKeyword,
    SyntaxKind::ColonColon,
    SyntaxKind::MinusMinus,
    SyntaxKind::PlusPlus,
    SyntaxKind::ResumeKeyword,
    SyntaxKind::DeleteKeyword,
    SyntaxKind::ThisKeyword,
    SyntaxKind::BaseKeyword,
    SyntaxKind::RawCallKeyword,
]);

pub const STATEMENT: TokenSet = TokenSet::new(&[
    SyntaxKind::Semicolon,
    SyntaxKind::OpenBrace,
    SyntaxKind::IfKeyword,
    SyntaxKind::WhileKeyword,
    SyntaxKind::DoKeyword,
    SyntaxKind::ForKeyword,
    SyntaxKind::ForEachKeyword,
    SyntaxKind::SwitchKeyword,
    SyntaxKind::LocalKeyword,
    SyntaxKind::ConstKeyword,
    SyntaxKind::ReturnKeyword,
    SyntaxKind::YieldKeyword,
    SyntaxKind::ContinueKeyword,
    SyntaxKind::BreakKeyword,
    SyntaxKind::FunctionKeyword,
    SyntaxKind::ClassKeyword,
    SyntaxKind::EnumKeyword,
    SyntaxKind::TryKeyword,
    SyntaxKind::ThrowKeyword,
]);

pub const STATEMENT_OR_EXPRESSION: TokenSet = EXPRESSIONS.union(STATEMENT);
pub const COMMON_STATEMENT_OR_EXPRESSION: TokenSet = COMMON_EXPRESSION_STATEMENTS.union(STATEMENT);

pub const VARIABLE_RECOVERY: TokenSet = COMMON_STATEMENT_OR_EXPRESSION.union(INIT_OPERATORS);

pub const PARAMETER_RECOVERY: TokenSet = VARIABLE_RECOVERY.union(TokenSet::new(&[
    SyntaxKind::CloseParenthesis,
    SyntaxKind::DotDotDot,
]));

pub const CATCH_RECOVERY: TokenSet =
    VARIABLE_RECOVERY.union(TokenSet::new(&[SyntaxKind::CloseParenthesis]));

pub const NAME_QUALIFIER: TokenSet = TokenSet::new(&[SyntaxKind::Dot, SyntaxKind::ColonColon]);

// if we see an equals sign we can parse it as nameless param
pub const FUNCTION_NAME_RECOVERY: TokenSet = PARAMETER_RECOVERY.union(TokenSet::new(&[
    SyntaxKind::OpenParenthesis,
    SyntaxKind::OpenBracket,
    SyntaxKind::CloseBracket,
    SyntaxKind::Dot,
]));

pub const KEYWORDS: TokenSet = TokenSet::new(&[
    SyntaxKind::BaseKeyword,
    SyntaxKind::BreakKeyword,
    SyntaxKind::CaseKeyword,
    SyntaxKind::CatchKeyword,
    SyntaxKind::ClassKeyword,
    SyntaxKind::CloneKeyword,
    SyntaxKind::ConstKeyword,
    SyntaxKind::ConstructorKeyword,
    SyntaxKind::ContinueKeyword,
    SyntaxKind::DefaultKeyword,
    SyntaxKind::DeleteKeyword,
    SyntaxKind::DoKeyword,
    SyntaxKind::ElseKeyword,
    SyntaxKind::EnumKeyword,
    SyntaxKind::ExtendsKeyword,
    SyntaxKind::FalseKeyword,
    SyntaxKind::ForEachKeyword,
    SyntaxKind::ForKeyword,
    SyntaxKind::FunctionKeyword,
    SyntaxKind::IfKeyword,
    SyntaxKind::InKeyword,
    SyntaxKind::InstanceOfKeyword,
    SyntaxKind::LocalKeyword,
    SyntaxKind::NullKeyword,
    SyntaxKind::RawCallKeyword,
    SyntaxKind::ResumeKeyword,
    SyntaxKind::ReturnKeyword,
    SyntaxKind::StaticKeyword,
    SyntaxKind::SwitchKeyword,
    SyntaxKind::ThisKeyword,
    SyntaxKind::ThrowKeyword,
    SyntaxKind::TrueKeyword,
    SyntaxKind::TryKeyword,
    SyntaxKind::TypeOfKeyword,
    SyntaxKind::WhileKeyword,
    SyntaxKind::YieldKeyword,
    SyntaxKind::FileKeyword,
    SyntaxKind::LineKeyword,
]);

pub const SWITCH_RECOVERY: TokenSet = STATEMENT_OR_EXPRESSION.union(TokenSet::new(&[
    SyntaxKind::Eof,
    SyntaxKind::CloseBrace,
    SyntaxKind::CaseKeyword,
    SyntaxKind::DefaultKeyword,
]));

pub const CALL_ARGUMENTS_STOP: TokenSet = STATEMENT.union(TokenSet::new(&[
    SyntaxKind::Eof,
    SyntaxKind::CloseParenthesis,
    SyntaxKind::CloseBrace,
]));

pub const EXPRESSION_RECOVERY: TokenSet = TokenSet::besides(&[
    SyntaxKind::Colon,
    SyntaxKind::DotDotDot,
    SyntaxKind::LessThanSlash,
    SyntaxKind::SlashGreaterThan,
]);
