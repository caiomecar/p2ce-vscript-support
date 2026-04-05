use crate::cst::{SquirrelLanguage, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::ast::{AstChildren, AstNode, support};

macro_rules! ast_node {
    ($name:ident, $kind:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            type Language = SquirrelLanguage;

            fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::$kind
            }

            fn cast(node: SyntaxNode) -> Option<Self> {
                if node.kind() == SyntaxKind::$kind {
                    Some(Self(node))
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

macro_rules! ast_enum {
    ($name:ident { $($variant:ident($inner:ident)),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant($inner),)*
        }

        impl AstNode for $name {
            type Language = SquirrelLanguage;

            fn can_cast(kind: SyntaxKind) -> bool {
                matches!(kind, $(SyntaxKind::$inner)|*)
            }

            fn cast(node: SyntaxNode) -> Option<Self> {
                Some(match node.kind() {
                    $(SyntaxKind::$inner => $name::$variant($inner(node)),)*
                    _ => return None,
                })
            }

            fn syntax(&self) -> &SyntaxNode {
                match self {
                    $($name::$variant(n) => n.syntax(),)*
                }
            }
        }
    };
}

macro_rules! ast_token_enum {
    ($name:ident { $($kind:ident => $variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub fn from_kind(kind: SyntaxKind) -> Option<Self> {
                match kind {
                    $(SyntaxKind::$kind => Some(Self::$variant),)*
                    _ => None,
                }
            }

            pub fn token(node: &SyntaxNode) -> Option<(Self, SyntaxToken)> {
                node.children_with_tokens()
                    .filter_map(|it| it.into_token())
                    .find_map(|tok| Some((Self::from_kind(tok.kind())?, tok)))
            }
        }
    };
}

// Searching `Expr` can be ambigious if a node has multiple expressions as direct children.
// Example: for (;abc;) | for (;;abc)
// Searching for the first expression here will either give us the condition or the increment
// Therefore expression wrappers are used that are nodes which just contain 1 expression
// But make it possible to distinct different parts of a node by having another SyntaxKind
// Otherwise the tree should be as flattened as possible so if a node contains only a single
// expression the wrapper is not created
pub trait ExpressionWrapper: AstNode<Language = SquirrelLanguage> {
    fn expression(&self) -> Option<Expr> {
        support::child(self.syntax())
    }
}

// Doesn't actually account for all the names, only the flat ones
pub trait HasName: AstNode<Language = SquirrelLanguage> {
    fn name(&self) -> Option<Name> {
        support::child(self.syntax())
    }
}

pub trait HasDoc: AstNode<Language = SquirrelLanguage> {
    fn doc(&self) -> Option<SyntaxToken> {
        support::token(self.syntax(), SyntaxKind::DocComment)
    }
}

pub trait HasOperand: AstNode<Language = SquirrelLanguage> {
    fn operand(&self) -> Option<Expr> {
        support::child(self.syntax())
    }
}

// Doesn't account for case/default bodies and lambda expression
pub trait HasBody: AstNode<Language = SquirrelLanguage> {
    fn body(&self) -> Option<Stmt> {
        support::child(self.syntax())
    }
}

pub trait IsFunction: AstNode<Language = SquirrelLanguage> {
    fn environment(&self) -> Option<Environment> {
        support::child(self.syntax())
    }

    fn parameter_list(&self) -> Option<ParameterList> {
        support::child(self.syntax())
    }

    fn body(&self) -> Option<FunctionBody> {
        support::child(self.syntax()).map(FunctionBody::Stmt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FunctionBody {
    Stmt(Stmt),
    Expr(Expr),
}

pub trait IsClass: AstNode<Language = SquirrelLanguage> {
    fn extends(&self) -> Option<Extends> {
        support::child(self.syntax())
    }

    fn members(&self) -> AstChildren<Member> {
        support::children(self.syntax())
    }
}

pub trait IsClassMember: AstNode<Language = SquirrelLanguage> {
    fn attributes(&self) -> Option<Attributes> {
        support::child(self.syntax())
    }

    fn static_keyword(&self) -> Option<SyntaxToken> {
        support::token(self.syntax(), SyntaxKind::StaticKeyword)
    }
}

ast_node!(Name, Name);
impl Name {
    pub fn identifier(&self) -> Option<SyntaxToken> {
        support::token(&self.0, SyntaxKind::Identifier)
    }

    pub fn text(&self) -> Option<String> {
        self.identifier().map(|t| t.text().to_owned())
    }
}

ast_node!(QualifiedName, QualifiedName);
impl QualifiedName {
    pub fn names(&self) -> AstChildren<Name> {
        support::children(&self.0)
    }
}

ast_token_enum!(LiteralExpressionKind {
    NullKeyword => Null,
    TrueKeyword => True,
    FalseKeyword => False,
    String => String,
    VerbatimString => VerbatimString,
    DecimalInteger => DecimalInteger,
    OctalInteger => OctalInteger,
    HexInteger => HexInteger,
    Character => Character,
    Float => Float,
});

ast_node!(LiteralExpression, LiteralExpression);
impl LiteralExpression {
    pub fn token(&self) -> Option<(LiteralExpressionKind, SyntaxToken)> {
        LiteralExpressionKind::token(&self.0)
    }
}
ast_token_enum!(BinaryOperator {
    Comma => Comma,

    Plus => Add,
    Minus => Subtract,
    Asterisk => Multiply,
    Slash => Divide,
    Percent => Modulo,

    Ampersand => BitwiseAnd,
    Bar => BitwiseOr,
    Caret => BitwiseXor,
    LessThanLessThan => LeftShift,
    GreaterThanGreaterThan => RightShift,
    GreaterThanGreaterThanGreaterThan => UnsignedRightShift,

    EqualsEquals => Equals,
    ExclamationEquals => NotEquals,

    LessThan => Less,
    LessThanEquals => LessEqual,
    GreaterThan => Greater,
    GreaterThanEquals => GreaterEqual,
    LessThanEqualsGreaterThan => ThreeWay,

    InstanceOfKeyword => InstanceOf,
    InKeyword => In,

    AmpersandAmpersand => LogicalAnd,
    BarBar => LogicalOr,

    Equals => Assign,
    LessThanMinus => NewSlot,
    PlusEquals => AddAssign,
    MinusEquals => SubtractAssign,
    AsteriskEquals => MultiplyAssign,
    SlashEquals => DivideAssign,
    PercentEquals => ModuloAssign,
});

ast_node!(BinaryExpression, BinaryExpression);
impl HasDoc for BinaryExpression {}

impl BinaryExpression {
    pub fn lhs(&self) -> Option<Expr> {
        support::children(&self.0).next()
    }

    pub fn operator(&self) -> Option<(BinaryOperator, SyntaxToken)> {
        BinaryOperator::token(&self.0)
    }
    // It's impossible to have rhs without lhs with the current
    // algorithm therefore expression wrapper is not needed so
    // .nth works this may need to change in the future if
    // recovery of `local a = * 2` is actually present
    pub fn rhs(&self) -> Option<Expr> {
        support::children(&self.0).nth(1)
    }
}

ast_node!(ThenBranch, ThenBranch);
impl ExpressionWrapper for ThenBranch {}

ast_node!(ElseBranch, ElseBranch);
impl ExpressionWrapper for ElseBranch {}

ast_node!(ConditionalExpression, ConditionalExpression);
impl ConditionalExpression {
    pub fn condition(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn then_branch(&self) -> Option<ThenBranch> {
        support::child(&self.0)
    }

    pub fn else_branch(&self) -> Option<ElseBranch> {
        support::child(&self.0)
    }
}

ast_token_enum!(PrefixUnaryOperator {
    Minus => Negation,
    Tilde => BitwiseNot,
    Exclamation => LogicalNot,
});
ast_node!(PrefixUnaryExpression, PrefixUnaryExpression);
impl HasOperand for PrefixUnaryExpression {}
impl PrefixUnaryExpression {
    pub fn operator(&self) -> Option<(PrefixUnaryOperator, SyntaxToken)> {
        PrefixUnaryOperator::token(&self.0)
    }
}

ast_token_enum!(PrefixUpdateOperator {
    PlusPlus => Increment,
    MinusMinus => Decrement,
});
ast_node!(PrefixUpdateExpression, PrefixUpdateExpression);
impl HasOperand for PrefixUpdateExpression {}
impl PrefixUpdateExpression {
    pub fn operator(&self) -> Option<(PrefixUpdateOperator, SyntaxToken)> {
        PrefixUpdateOperator::token(&self.0)
    }
}

ast_token_enum!(PostfixUpdateOperator {
    PlusPlus => Increment,
    MinusMinus => Decrement,
});
ast_node!(PostfixUpdateExpression, PostfixUpdateExpression);
impl HasOperand for PostfixUpdateExpression {}
impl PostfixUpdateExpression {
    pub fn operator(&self) -> Option<(PostfixUpdateOperator, SyntaxToken)> {
        PostfixUpdateOperator::token(&self.0)
    }
}

ast_node!(DeleteExpression, DeleteExpression);
impl HasOperand for DeleteExpression {}

ast_node!(TypeOfExpression, TypeOfExpression);
impl HasOperand for TypeOfExpression {}

ast_node!(CloneExpression, CloneExpression);
impl HasOperand for CloneExpression {}

ast_node!(ResumeExpression, ResumeExpression);
impl HasOperand for ResumeExpression {}

ast_node!(RawCallExpression, RawCallExpression);
impl RawCallExpression {
    pub fn arguments(&self) -> AstChildren<Expr> {
        support::children(&self.0)
    }
}

ast_node!(MemberPart, MemberPart);
impl HasName for MemberPart {}

ast_node!(MemberAccessExpression, MemberAccessExpression);

impl MemberAccessExpression {
    pub fn object(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn member_part(&self) -> Option<MemberPart> {
        support::child(&self.0)
    }
}

ast_node!(Index, Index);
impl ExpressionWrapper for Index {}

ast_node!(ElementAccessExpression, ElementAccessExpression);
impl ElementAccessExpression {
    pub fn object(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn index(&self) -> Option<Index> {
        support::child(&self.0)
    }
}

ast_node!(Callee, Callee);
impl ExpressionWrapper for Callee {}
ast_node!(CallExpression, CallExpression);

impl CallExpression {
    pub fn callee(&self) -> Option<Callee> {
        support::child(&self.0)
    }

    pub fn arguments(&self) -> impl Iterator<Item = Expr> + '_ {
        support::children(&self.0)
    }

    pub fn post_call_initialiser(&self) -> Option<PostCallInitialiser> {
        support::child(&self.0)
    }
}

ast_node!(RootAccessExpression, RootAccessExpression);
impl HasName for RootAccessExpression {}

ast_node!(ThisExpression, ThisExpression);
ast_node!(BaseExpression, BaseExpression);
ast_node!(FileExpression, FileExpression);
ast_node!(LineExpression, LineExpression);

ast_node!(ParenthesisedExpression, ParenthesisedExpression);

impl ParenthesisedExpression {
    pub fn inner(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(ArrayLiteralExpression, ArrayLiteralExpression);

impl ArrayLiteralExpression {
    pub fn elements(&self) -> AstChildren<Expr> {
        support::children(&self.0)
    }
}

ast_node!(TableLiteralExpression, TableLiteralExpression);

impl TableLiteralExpression {
    pub fn members(&self) -> AstChildren<Member> {
        support::children(&self.0)
    }
}

ast_node!(FunctionExpression, FunctionExpression);
impl IsFunction for FunctionExpression {}

ast_node!(LambdaExpression, LambdaExpression);
impl IsFunction for LambdaExpression {
    fn body(&self) -> Option<FunctionBody> {
        support::child(self.syntax()).map(FunctionBody::Expr)
    }
}

ast_node!(ClassExpression, ClassExpression);
impl IsClass for ClassExpression {}

ast_enum!(Expr {
    Literal(LiteralExpression),
    Name(Name),
    Binary(BinaryExpression),
    Conditional(ConditionalExpression),
    PrefixUnary(PrefixUnaryExpression),
    PrefixUpdate(PrefixUpdateExpression),
    PostfixUpdate(PostfixUpdateExpression),
    Delete(DeleteExpression),
    TypeOf(TypeOfExpression),
    Clone(CloneExpression),
    Resume(ResumeExpression),
    RawCall(RawCallExpression),
    MemberAccess(MemberAccessExpression),
    ElementAccess(ElementAccessExpression),
    Call(CallExpression),
    RootAccess(RootAccessExpression),
    This(ThisExpression),
    Base(BaseExpression),
    File(FileExpression),
    Line(LineExpression),
    Parenthesised(ParenthesisedExpression),
    ArrayLiteral(ArrayLiteralExpression),
    TableLiteral(TableLiteralExpression),
    Function(FunctionExpression),
    Lambda(LambdaExpression),
    Class(ClassExpression),
});

ast_node!(SourceFile, SourceFile);
impl HasDoc for SourceFile {}

impl SourceFile {
    pub fn statements(&self) -> AstChildren<Stmt> {
        support::children(&self.0)
    }
}

ast_node!(EmptyStatement, EmptyStatement);

ast_node!(BlockStatement, BlockStatement);

impl BlockStatement {
    pub fn statements(&self) -> AstChildren<Stmt> {
        support::children(&self.0)
    }
}

ast_node!(IfStatement, IfStatement);
ast_node!(IfElseBranch, IfElseBranch);

impl IfStatement {
    pub fn condition(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn statement(&self) -> Option<Stmt> {
        support::child(&self.0)
    }

    pub fn else_branch(&self) -> Option<IfElseBranch> {
        support::child(&self.0)
    }
}

impl IfElseBranch {
    pub fn statement(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

ast_node!(WhileStatement, WhileStatement);
impl HasBody for WhileStatement {}

impl WhileStatement {
    pub fn condition(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(DoWhileStatement, DoWhileStatement);
impl HasBody for DoWhileStatement {}

impl DoWhileStatement {
    pub fn condition(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(ForInitialiser, ForInitialiser);
pub enum ForInitialiserKind {
    LocalVariableDeclaration(LocalVariableDeclaration),
    LocalFunctionDeclaration(LocalFunctionDeclaration),
    Expression(Expr),
}
impl ForInitialiser {
    pub fn kind(&self) -> Option<ForInitialiserKind> {
        self.syntax().children().find_map(|node| match node.kind() {
            SyntaxKind::LocalVariableDeclaration => Some(
                ForInitialiserKind::LocalVariableDeclaration(LocalVariableDeclaration(node)),
            ),
            SyntaxKind::LocalFunctionDeclaration => Some(
                ForInitialiserKind::LocalFunctionDeclaration(LocalFunctionDeclaration(node)),
            ),
            _ if Expr::can_cast(node.kind()) => {
                Some(ForInitialiserKind::Expression(Expr::cast(node)?))
            }
            _ => None,
        })
    }
}

ast_node!(ForCondition, ForCondition);
impl ExpressionWrapper for ForCondition {}

ast_node!(ForIncrement, ForIncrement);
impl ExpressionWrapper for ForIncrement {}

ast_node!(ForStatement, ForStatement);
impl HasBody for ForStatement {}

impl ForStatement {
    pub fn initialiser(&self) -> Option<ForInitialiser> {
        support::child(&self.0)
    }

    pub fn condition(&self) -> Option<ForCondition> {
        support::child(&self.0)
    }

    pub fn increment(&self) -> Option<ForIncrement> {
        support::child(&self.0)
    }
}

ast_node!(ForEachKey, ForeachKey);
impl HasName for ForEachKey {}

ast_node!(ForEachValue, ForeachValue);
impl HasName for ForEachValue {}

ast_node!(ForEachStatement, ForEachStatement);
impl HasBody for ForEachStatement {}

impl ForEachStatement {
    pub fn key(&self) -> Option<ForEachKey> {
        support::child(&self.0)
    }

    pub fn value(&self) -> Option<ForEachValue> {
        support::child(&self.0)
    }

    pub fn iterable(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(CaseClause, CaseClause);

impl CaseClause {
    pub fn test(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn body(&self) -> AstChildren<Stmt> {
        support::children(&self.0)
    }
}

ast_node!(DefaultClause, DefaultClause);

impl DefaultClause {
    pub fn body(&self) -> AstChildren<Stmt> {
        support::children(&self.0)
    }
}

ast_enum!(SwitchClause {
    Case(CaseClause),
    Default(DefaultClause)
});

ast_node!(SwitchStatement, SwitchStatement);

impl SwitchStatement {
    pub fn discriminant(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn clauses(&self) -> AstChildren<SwitchClause> {
        support::children(&self.0)
    }
}

ast_node!(ConstStatement, ConstStatement);
impl HasDoc for ConstStatement {}
impl HasName for ConstStatement {}

impl ConstStatement {
    pub fn value(&self) -> Option<Initialiser> {
        support::child(&self.0)
    }
}

ast_node!(LocalVariableDeclaration, LocalVariableDeclaration);
impl HasDoc for LocalVariableDeclaration {}

impl LocalVariableDeclaration {
    pub fn declarations(&self) -> AstChildren<VariableDeclaration> {
        support::children(&self.0)
    }
}

ast_node!(LocalFunctionDeclaration, LocalFunctionDeclaration);
impl HasDoc for LocalFunctionDeclaration {}
impl HasName for LocalFunctionDeclaration {}
impl IsFunction for LocalFunctionDeclaration {}

ast_node!(ReturnStatement, ReturnStatement);

impl ReturnStatement {
    pub fn value(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(YieldStatement, YieldStatement);

impl YieldStatement {
    pub fn value(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(ContinueStatement, ContinueStatement);
ast_node!(BreakStatement, BreakStatement);

ast_node!(FunctionStatement, FunctionStatement);
impl HasDoc for FunctionStatement {}
impl IsFunction for FunctionStatement {}

impl FunctionStatement {
    pub fn name(&self) -> Option<QualifiedName> {
        support::child(&self.0)
    }
}

ast_node!(ClassStatement, ClassStatement);
impl HasDoc for ClassStatement {}
impl IsClass for ClassStatement {}

impl ClassStatement {
    pub fn name(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(EnumStatement, EnumStatement);
impl HasDoc for EnumStatement {}
impl HasName for EnumStatement {}

impl EnumStatement {
    pub fn members(&self) -> AstChildren<Property> {
        support::children(&self.0)
    }
}

ast_node!(TryStatement, TryStatement);
impl HasBody for TryStatement {}

impl TryStatement {
    pub fn catch_clause(&self) -> Option<CatchClause> {
        support::child(&self.0)
    }
}

ast_node!(CatchClause, CatchClause);
impl HasBody for CatchClause {}

impl CatchClause {
    pub fn binding(&self) -> Option<VariableDeclaration> {
        support::child(&self.0)
    }
}

ast_node!(ThrowStatement, ThrowStatement);

impl ThrowStatement {
    pub fn value(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(ExpressionStatement, ExpressionStatement);
impl ExpressionWrapper for ExpressionStatement {}

ast_enum!(Stmt {
    Empty(EmptyStatement),
    Block(BlockStatement),
    If(IfStatement),
    While(WhileStatement),
    DoWhile(DoWhileStatement),
    For(ForStatement),
    ForEach(ForEachStatement),
    Switch(SwitchStatement),
    Const(ConstStatement),
    LocalVariable(LocalVariableDeclaration),
    LocalFunction(LocalFunctionDeclaration),
    Return(ReturnStatement),
    Yield(YieldStatement),
    Continue(ContinueStatement),
    Break(BreakStatement),
    Function(FunctionStatement),
    Class(ClassStatement),
    Enum(EnumStatement),
    Try(TryStatement),
    Throw(ThrowStatement),
    Expression(ExpressionStatement),
});

ast_node!(Initialiser, Initialiser);
impl ExpressionWrapper for Initialiser {}

ast_node!(VariableDeclaration, VariableDeclaration);
impl HasDoc for VariableDeclaration {}
impl HasName for VariableDeclaration {}

impl VariableDeclaration {
    pub fn initialiser(&self) -> Option<Initialiser> {
        support::child(&self.0)
    }
}

ast_enum!(Parameter {
    Variable(VariableDeclaration),
    Ellipsis(VariedArgs),
});

ast_node!(ParameterList, ParameterList);

impl ParameterList {
    pub fn parameters(&self) -> AstChildren<Parameter> {
        support::children(&self.0)
    }
}

ast_node!(Environment, Environment);
impl ExpressionWrapper for Environment {}

ast_node!(VariedArgs, VariedArgs);

ast_node!(Extends, Extends);
impl ExpressionWrapper for Extends {}

ast_node!(Attributes, Attributes);

impl Attributes {
    pub fn members(&self) -> AstChildren<Property> {
        support::children(&self.0)
    }
}

ast_node!(PostCallInitialiser, PostCallInitialiser);

impl PostCallInitialiser {
    pub fn members(&self) -> AstChildren<Property> {
        support::children(&self.0)
    }
}

ast_node!(Property, Property);
impl HasDoc for Property {}
impl IsClassMember for Property {}

impl Property {
    pub fn name(&self) -> Option<MemberName> {
        support::child(&self.0)
    }

    pub fn value(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(SimpleName, SimpleName);
impl SimpleName {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }
}

ast_token_enum!(StringNameKind {
    String => Normal,
    VerbatimString => Verbatim,
});

ast_node!(StringName, StringName);
impl StringName {
    pub fn token(&self) -> Option<(StringNameKind, SyntaxToken)> {
        StringNameKind::token(&self.0)
    }
}

ast_node!(ComputedName, ComputedName);
impl ExpressionWrapper for ComputedName {}

ast_enum!(MemberName {
    Identifier(SimpleName),
    String(StringName),
    Computed(ComputedName),
});

ast_node!(Constructor, Constructor);
impl HasDoc for Constructor {}
impl IsFunction for Constructor {}
impl IsClassMember for Constructor {}
impl Constructor {
    pub fn constructor_keyword(&self) -> Option<SyntaxToken> {
        support::token(&self.0, SyntaxKind::ConstructorKeyword)
    }
}

ast_node!(Method, Method);
impl HasDoc for Method {}
impl HasName for Method {}
impl IsFunction for Method {}
impl IsClassMember for Method {}

ast_enum!(Member {
    Property(Property),
    Constructor(Constructor),
    Method(Method),
});
