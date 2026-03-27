use rowan::Language;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u16)]
pub enum SyntaxKind {
    Unknown = 0,
    Eof,

    Whitespace,
    LineFeed,

    BlockComment,
    DocComment,
    LineComment,

    AsteriskEquals,
    Equals,
    MinusEquals,
    PercentEquals,
    PlusEquals,
    SlashEquals,
    LessThanMinus,

    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Ampersand,
    Bar,
    Caret,
    AmpersandAmpersand,
    BarBar,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    EqualsEquals,
    ExclamationEquals,
    LessThanLessThan,
    GreaterThanGreaterThan,
    GreaterThanGreaterThanGreaterThan,
    LessThanEqualsGreaterThan,

    Exclamation,
    Tilde,
    PlusPlus,
    MinusMinus,

    At,
    Colon,
    ColonColon,
    Comma,
    Semicolon,
    Dot,
    DotDotDot,
    Question,

    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenParenthesis,
    CloseParenthesis,
    LessThanSlash,
    SlashGreaterThan,

    Identifier,
    Float,
    Integer,
    String,
    Character,
    VerbatimString,

    BaseKeyword,
    BreakKeyword,
    CaseKeyword,
    CatchKeyword,
    ClassKeyword,
    CloneKeyword,
    ConstKeyword,
    ConstructorKeyword,
    ContinueKeyword,
    DefaultKeyword,
    DeleteKeyword,
    DoKeyword,
    ElseKeyword,
    EnumKeyword,
    ExtendsKeyword,
    FalseKeyword,
    ForEachKeyword,
    ForKeyword,
    FunctionKeyword,
    IfKeyword,
    InKeyword,
    InstanceOfKeyword,
    LocalKeyword,
    NullKeyword,
    RawCallKeyword,
    ResumeKeyword,
    ReturnKeyword,
    StaticKeyword,
    SwitchKeyword,
    ThisKeyword,
    ThrowKeyword,
    TrueKeyword,
    TryKeyword,
    TypeOfKeyword,
    WhileKeyword,
    YieldKeyword,

    FileKeyword,
    LineKeyword,

    __LastToken,

    SourceFile,
    Error,

    Name,
    QualifiedName,

    Index,
    MemberPart,

    VariableDeclarationList,
    VariableDeclaration,
    Initialiser,

    CatchClause,

    ForInitialiser,
    ForCondition,
    ForIncrement,

    ForeachKey,
    ForeachValue,

    Environment,
    ParameterList,
    VariedArgs,

    PostCallInitialiser,

    Extends,
    Members,
    Property,
    SimpleName,
    StringName,
    ComputedName,

    Constructor,
    Method,
    Attributes,

    // For ternary
    ThenBranch,
    ElseBranch,

    BinaryExpression,
    ConditionalExpression,
    PrefixUnaryExpression,
    PrefixUpdateExpression,
    DeleteExpression,
    TypeOfExpression,
    CloneExpression,
    ResumeExpression,
    RawCallExpression,
    PostfixUpdateExpression,
    MemberAccessExpression,
    ElementAccessExpression,
    CallExpression,
    LiteralExpression,
    RootAccessExpression,
    ThisExpression,
    BaseExpression,
    FileExpression,
    LineExpression,
    ParenthesisedExpression,
    ArrayLiteralExpression,
    TableLiteralExpression,
    FunctionExpression,
    LambdaExpression,
    ClassExpression,

    EmptyStatement,
    BlockStatement,
    IfStatement,
    WhileStatement,
    DoWhileStatement,
    ForStatement,
    ForEachStatement,
    SwitchStatement,
    ConstStatement,
    // These 2 can also appear inside for loop, so they're not necessarily statements
    LocalFunctionDeclaration,
    LocalVariableDeclaration,
    ReturnStatement,
    YieldStatement,
    ContinueStatement,
    BreakStatement,
    FunctionStatement,
    ClassStatement,
    EnumStatement,
    TryStatement,
    ThrowStatement,
    ExpressionStatement,

    CaseClause,
    DefaultClause,

    __Last,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl SyntaxKind {
    pub fn text(&self) -> &'static str {
        match self {
            SyntaxKind::Eof => "end of file",
            SyntaxKind::AsteriskEquals => "'*='",
            SyntaxKind::Equals => "'='",
            SyntaxKind::MinusEquals => "'-='",
            SyntaxKind::PercentEquals => "'%='",
            SyntaxKind::PlusEquals => "'+='",
            SyntaxKind::SlashEquals => "'/='",
            SyntaxKind::LessThanMinus => "'<-'",

            SyntaxKind::Plus => "'+'",
            SyntaxKind::Minus => "'-'",
            SyntaxKind::Asterisk => "'*'",
            SyntaxKind::Slash => "'/'",
            SyntaxKind::Percent => "'%'",
            SyntaxKind::Ampersand => "'&'",
            SyntaxKind::Bar => "'|'",
            SyntaxKind::Caret => "'^'",
            SyntaxKind::AmpersandAmpersand => "'&&'",
            SyntaxKind::BarBar => "'||'",
            SyntaxKind::LessThan => "'<'",
            SyntaxKind::GreaterThan => "'>'",
            SyntaxKind::LessThanEquals => "'<='",
            SyntaxKind::GreaterThanEquals => "'>='",
            SyntaxKind::EqualsEquals => "'=='",
            SyntaxKind::ExclamationEquals => "'!='",
            SyntaxKind::LessThanLessThan => "'<<'",
            SyntaxKind::GreaterThanGreaterThan => "'>>'",
            SyntaxKind::GreaterThanGreaterThanGreaterThan => "'>>>'",
            SyntaxKind::LessThanEqualsGreaterThan => "'<=>'",

            SyntaxKind::Exclamation => "'!'",
            SyntaxKind::Tilde => "'~'",
            SyntaxKind::PlusPlus => "'++'",
            SyntaxKind::MinusMinus => "'--'",

            SyntaxKind::At => "'@'",
            SyntaxKind::Colon => "':'",
            SyntaxKind::ColonColon => "'::'",
            SyntaxKind::Comma => "','",
            SyntaxKind::Semicolon => "';'",
            SyntaxKind::Dot => "'.'",
            SyntaxKind::DotDotDot => "'...'",
            SyntaxKind::Question => "'?'",

            SyntaxKind::OpenBrace => "'{'",
            SyntaxKind::CloseBrace => "'}'",
            SyntaxKind::OpenBracket => "'['",
            SyntaxKind::CloseBracket => "']'",
            SyntaxKind::OpenParenthesis => "'('",
            SyntaxKind::CloseParenthesis => "')'",
            SyntaxKind::LessThanSlash => "'</'",
            SyntaxKind::SlashGreaterThan => "'/>'",

            SyntaxKind::Identifier => "'identifier'",
            SyntaxKind::Float => "float",
            SyntaxKind::Integer => "integer",
            SyntaxKind::String => "string",
            SyntaxKind::Character => "character",
            SyntaxKind::VerbatimString => "verbatim string",

            SyntaxKind::BaseKeyword => "'base'",
            SyntaxKind::BreakKeyword => "'break'",
            SyntaxKind::CaseKeyword => "'case'",
            SyntaxKind::CatchKeyword => "'catch'",
            SyntaxKind::ClassKeyword => "'class'",
            SyntaxKind::CloneKeyword => "'clone'",
            SyntaxKind::ConstKeyword => "'const'",
            SyntaxKind::ConstructorKeyword => "'constructor'",
            SyntaxKind::ContinueKeyword => "'continue'",
            SyntaxKind::DefaultKeyword => "'default'",
            SyntaxKind::DeleteKeyword => "'delete'",
            SyntaxKind::DoKeyword => "'do'",
            SyntaxKind::ElseKeyword => "'else'",
            SyntaxKind::EnumKeyword => "'enum'",
            SyntaxKind::ExtendsKeyword => "'extends'",
            SyntaxKind::FalseKeyword => "'false'",
            SyntaxKind::ForEachKeyword => "'foreach'",
            SyntaxKind::ForKeyword => "'for'",
            SyntaxKind::FunctionKeyword => "'function'",
            SyntaxKind::IfKeyword => "'if'",
            SyntaxKind::InKeyword => "'in'",
            SyntaxKind::InstanceOfKeyword => "'instanceof'",
            SyntaxKind::LocalKeyword => "'local'",
            SyntaxKind::NullKeyword => "'null'",
            SyntaxKind::RawCallKeyword => "'rawcall'",
            SyntaxKind::ResumeKeyword => "'resume'",
            SyntaxKind::ReturnKeyword => "'return'",
            SyntaxKind::StaticKeyword => "'static'",
            SyntaxKind::SwitchKeyword => "'switch'",
            SyntaxKind::ThisKeyword => "'this'",
            SyntaxKind::ThrowKeyword => "'throw'",
            SyntaxKind::TrueKeyword => "'true'",
            SyntaxKind::TryKeyword => "'try'",
            SyntaxKind::TypeOfKeyword => "'typeof'",
            SyntaxKind::WhileKeyword => "'while'",
            SyntaxKind::YieldKeyword => "'yield'",

            SyntaxKind::FileKeyword => "'__FILE__'",
            SyntaxKind::LineKeyword => "'__LINE__'",

            _ => panic!("SyntaxKind::{:?} has no fixed text representation", self),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SquirrelLanguage {}

impl Language for SquirrelLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 < SyntaxKind::__Last as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<SquirrelLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<SquirrelLanguage>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;
