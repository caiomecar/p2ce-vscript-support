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

// Searching `Expr` can be ambigious if a node has multiple expressions as direct children.
// Example: for (;abc;) | for (;;abc)
// Searching for the first expression here will either give us the condition or the increment
// Therefore expression wrappers are used that are nodes which just contain 1 expression
// But make it possible to distinct different parts of a node by having another SyntaxKind
// Otherwise the tree should be as flattened as possible so if a node contains only a single
// expression the wrapper is not created
macro_rules! expr_wrapper {
    ($name:ident) => {
        impl $name {
            pub fn expression(&self) -> Option<Expr> {
                support::child(&self.0)
            }
        }
    };
}

fn first_token(node: &SyntaxNode) -> Option<SyntaxToken> {
    node.children_with_tokens()
        .filter_map(|it| it.into_token())
        .next()
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

ast_node!(Operator, Operator);

impl Operator {
    pub fn token(&self) -> Option<SyntaxToken> {
        first_token(&self.0)
    }
}

ast_node!(LiteralExpression, LiteralExpression);
impl LiteralExpression {
    pub fn token(&self) -> Option<SyntaxToken> {
        first_token(&self.0)
    }
}

ast_node!(BinaryExpression, BinaryExpression);
impl BinaryExpression {
    pub fn lhs(&self) -> Option<Expr> {
        support::children(&self.0).next()
    }

    pub fn operator(&self) -> Option<Operator> {
        support::child(&self.0)
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
expr_wrapper!(ThenBranch);

ast_node!(ElseBranch, ElseBranch);
expr_wrapper!(ElseBranch);

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

ast_node!(PrefixUnaryExpression, PrefixUnaryExpression);

impl PrefixUnaryExpression {
    pub fn operator(&self) -> Option<Operator> {
        support::child(&self.0)
    }

    pub fn operand(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(PrefixUpdateExpression, PrefixUpdateExpression);

impl PrefixUpdateExpression {
    pub fn operator(&self) -> Option<Operator> {
        support::child(&self.0)
    }

    pub fn operand(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(PostfixUpdateExpression, PostfixUpdateExpression);

impl PostfixUpdateExpression {
    pub fn operand(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn operator(&self) -> Option<Operator> {
        support::child(&self.0)
    }
}

ast_node!(DeleteExpression, DeleteExpression);

impl DeleteExpression {
    pub fn operand(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(TypeOfExpression, TypeOfExpression);

impl TypeOfExpression {
    pub fn operand(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(CloneExpression, CloneExpression);

impl CloneExpression {
    pub fn operand(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(ResumeExpression, ResumeExpression);

impl ResumeExpression {
    pub fn operand(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(RawCallExpression, RawCallExpression);

impl RawCallExpression {
    pub fn arguments(&self) -> AstChildren<Expr> {
        support::children(&self.0)
    }
}

ast_node!(Member, Member);
impl Member {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }
}

ast_node!(MemberAccessExpression, MemberAccessExpression);

impl MemberAccessExpression {
    pub fn object(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn member(&self) -> Option<Member> {
        support::child(&self.0)
    }
}

ast_node!(Index, Index);
expr_wrapper!(Index);

ast_node!(ElementAccessExpression, ElementAccessExpression);
impl ElementAccessExpression {
    pub fn object(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn index(&self) -> Option<Index> {
        support::child(&self.0)
    }
}

ast_node!(CallExpression, CallExpression);

impl CallExpression {
    pub fn callee(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn arguments(&self) -> impl Iterator<Item = Expr> + '_ {
        support::children(&self.0).skip(1)
    }

    pub fn post_call_initialiser(&self) -> Option<PostCallInitialiser> {
        support::child(&self.0)
    }
}

ast_node!(RootAccessExpression, RootAccessExpression);

impl RootAccessExpression {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }
}

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
    pub fn members(&self) -> AstChildren<Property> {
        support::children(&self.0)
    }
}

ast_node!(FunctionExpression, FunctionExpression);

impl FunctionExpression {
    pub fn parameter_list(&self) -> Option<ParameterList> {
        support::child(&self.0)
    }

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

ast_node!(LambdaExpression, LambdaExpression);

impl LambdaExpression {
    pub fn parameter_list(&self) -> Option<ParameterList> {
        support::child(&self.0)
    }

    pub fn body(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(ClassExpression, ClassExpression);

impl ClassExpression {
    pub fn extends(&self) -> Option<Extends> {
        support::child(&self.0)
    }

    pub fn members(&self) -> AstChildren<ClassMember> {
        support::children(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
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
}

impl AstNode for Expr {
    type Language = SquirrelLanguage;

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::LiteralExpression
                | SyntaxKind::Name
                | SyntaxKind::BinaryExpression
                | SyntaxKind::ConditionalExpression
                | SyntaxKind::PrefixUnaryExpression
                | SyntaxKind::PrefixUpdateExpression
                | SyntaxKind::PostfixUpdateExpression
                | SyntaxKind::DeleteExpression
                | SyntaxKind::TypeOfExpression
                | SyntaxKind::CloneExpression
                | SyntaxKind::ResumeExpression
                | SyntaxKind::RawCallExpression
                | SyntaxKind::MemberAccessExpression
                | SyntaxKind::ElementAccessExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::RootAccessExpression
                | SyntaxKind::ThisExpression
                | SyntaxKind::BaseExpression
                | SyntaxKind::FileExpression
                | SyntaxKind::LineExpression
                | SyntaxKind::ParenthesisedExpression
                | SyntaxKind::ArrayLiteralExpression
                | SyntaxKind::TableLiteralExpression
                | SyntaxKind::FunctionExpression
                | SyntaxKind::LambdaExpression
                | SyntaxKind::ClassExpression
        )
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        Some(match node.kind() {
            SyntaxKind::LiteralExpression => Expr::Literal(LiteralExpression(node)),
            SyntaxKind::Name => Expr::Name(Name(node)),
            SyntaxKind::BinaryExpression => Expr::Binary(BinaryExpression(node)),
            SyntaxKind::ConditionalExpression => Expr::Conditional(ConditionalExpression(node)),
            SyntaxKind::PrefixUnaryExpression => Expr::PrefixUnary(PrefixUnaryExpression(node)),
            SyntaxKind::PrefixUpdateExpression => Expr::PrefixUpdate(PrefixUpdateExpression(node)),
            SyntaxKind::PostfixUpdateExpression => {
                Expr::PostfixUpdate(PostfixUpdateExpression(node))
            }
            SyntaxKind::DeleteExpression => Expr::Delete(DeleteExpression(node)),
            SyntaxKind::TypeOfExpression => Expr::TypeOf(TypeOfExpression(node)),
            SyntaxKind::CloneExpression => Expr::Clone(CloneExpression(node)),
            SyntaxKind::ResumeExpression => Expr::Resume(ResumeExpression(node)),
            SyntaxKind::RawCallExpression => Expr::RawCall(RawCallExpression(node)),
            SyntaxKind::MemberAccessExpression => Expr::MemberAccess(MemberAccessExpression(node)),
            SyntaxKind::ElementAccessExpression => {
                Expr::ElementAccess(ElementAccessExpression(node))
            }
            SyntaxKind::CallExpression => Expr::Call(CallExpression(node)),
            SyntaxKind::RootAccessExpression => Expr::RootAccess(RootAccessExpression(node)),
            SyntaxKind::ThisExpression => Expr::This(ThisExpression(node)),
            SyntaxKind::BaseExpression => Expr::Base(BaseExpression(node)),
            SyntaxKind::FileExpression => Expr::File(FileExpression(node)),
            SyntaxKind::LineExpression => Expr::Line(LineExpression(node)),
            SyntaxKind::ParenthesisedExpression => {
                Expr::Parenthesised(ParenthesisedExpression(node))
            }
            SyntaxKind::ArrayLiteralExpression => Expr::ArrayLiteral(ArrayLiteralExpression(node)),
            SyntaxKind::TableLiteralExpression => Expr::TableLiteral(TableLiteralExpression(node)),
            SyntaxKind::FunctionExpression => Expr::Function(FunctionExpression(node)),
            SyntaxKind::LambdaExpression => Expr::Lambda(LambdaExpression(node)),
            SyntaxKind::ClassExpression => Expr::Class(ClassExpression(node)),
            _ => return None,
        })
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Expr::Literal(n) => n.syntax(),
            Expr::Name(n) => n.syntax(),
            Expr::Binary(n) => n.syntax(),
            Expr::Conditional(n) => n.syntax(),
            Expr::PrefixUnary(n) => n.syntax(),
            Expr::PrefixUpdate(n) => n.syntax(),
            Expr::PostfixUpdate(n) => n.syntax(),
            Expr::Delete(n) => n.syntax(),
            Expr::TypeOf(n) => n.syntax(),
            Expr::Clone(n) => n.syntax(),
            Expr::Resume(n) => n.syntax(),
            Expr::RawCall(n) => n.syntax(),
            Expr::MemberAccess(n) => n.syntax(),
            Expr::ElementAccess(n) => n.syntax(),
            Expr::Call(n) => n.syntax(),
            Expr::RootAccess(n) => n.syntax(),
            Expr::This(n) => n.syntax(),
            Expr::Base(n) => n.syntax(),
            Expr::File(n) => n.syntax(),
            Expr::Line(n) => n.syntax(),
            Expr::Parenthesised(n) => n.syntax(),
            Expr::ArrayLiteral(n) => n.syntax(),
            Expr::TableLiteral(n) => n.syntax(),
            Expr::Function(n) => n.syntax(),
            Expr::Lambda(n) => n.syntax(),
            Expr::Class(n) => n.syntax(),
        }
    }
}

ast_node!(SourceFile, SourceFile);

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

impl IfStatement {
    pub fn condition(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn then_branch(&self) -> Option<Stmt> {
        support::children(&self.0).next()
    }

    // It's impossible to have else branch but not have then branch
    // Therefore it's possible to use .nth here instead of using a wrapper
    pub fn else_branch(&self) -> Option<Stmt> {
        support::children(&self.0).nth(1)
    }
}

ast_node!(WhileStatement, WhileStatement);

impl WhileStatement {
    pub fn condition(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

ast_node!(DoWhileStatement, DoWhileStatement);

impl DoWhileStatement {
    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }

    pub fn condition(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForInitialiser {
    LocalVariableDeclaration(LocalVariableDeclaration),
    LocalFunctionDeclaration(LocalFunctionDeclaration),
    Expression(Expr),
}

impl AstNode for ForInitialiser {
    type Language = SquirrelLanguage;

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::LocalVariableDeclaration | SyntaxKind::LocalFunctionDeclaration
        ) || Expr::can_cast(kind)
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        Some(match node.kind() {
            SyntaxKind::LocalVariableDeclaration => {
                ForInitialiser::LocalVariableDeclaration(LocalVariableDeclaration(node))
            }
            SyntaxKind::LocalFunctionDeclaration => {
                ForInitialiser::LocalFunctionDeclaration(LocalFunctionDeclaration(node))
            }
            _ => ForInitialiser::Expression(Expr::cast(node)?),
        })
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            ForInitialiser::LocalVariableDeclaration(n) => n.syntax(),
            ForInitialiser::LocalFunctionDeclaration(n) => n.syntax(),
            ForInitialiser::Expression(n) => n.syntax(),
        }
    }
}

ast_node!(ForCondition, ForCondition);
expr_wrapper!(ForCondition);

ast_node!(ForIncrement, ForIncrement);
expr_wrapper!(ForIncrement);

ast_node!(ForStatement, ForStatement);

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

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

ast_node!(ForEachKey, ForeachKey);
impl ForEachKey {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }
}

ast_node!(ForEachValue, ForeachValue);
impl ForEachValue {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }
}

ast_node!(ForEachStatement, ForEachStatement);

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

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

ast_node!(SwitchStatement, SwitchStatement);

impl SwitchStatement {
    pub fn discriminant(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn case_clauses(&self) -> AstChildren<CaseClause> {
        support::children(&self.0)
    }

    pub fn default_clause(&self) -> Option<DefaultClause> {
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

ast_node!(ConstStatement, ConstStatement);

impl ConstStatement {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }

    pub fn value(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(LocalVariableDeclaration, LocalVariableDeclaration);

impl LocalVariableDeclaration {
    pub fn declarations(&self) -> AstChildren<VariableDeclaration> {
        support::children(&self.0)
    }
}

ast_node!(LocalFunctionDeclaration, LocalFunctionDeclaration);

impl LocalFunctionDeclaration {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }

    pub fn parameter_list(&self) -> Option<ParameterList> {
        support::child(&self.0)
    }

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

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

impl FunctionStatement {
    pub fn name(&self) -> FunctionName {
        if let Some(qn) = support::child::<QualifiedName>(&self.0) {
            FunctionName::Qualified(qn)
        } else {
            FunctionName::Simple(
                support::child(&self.0).expect("FunctionStatement must have a name"),
            )
        }
    }

    pub fn parameter_list(&self) -> Option<ParameterList> {
        support::child(&self.0)
    }

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

pub enum FunctionName {
    Simple(Name),
    Qualified(QualifiedName),
}

ast_node!(ClassStatement, ClassStatement);

impl ClassStatement {
    pub fn name(&self) -> Option<Expr> {
        support::child(&self.0)
    }

    pub fn extends(&self) -> Option<Extends> {
        support::child(&self.0)
    }

    pub fn members(&self) -> AstChildren<ClassMember> {
        support::children(&self.0)
    }
}

ast_node!(EnumStatement, EnumStatement);

impl EnumStatement {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }

    pub fn members(&self) -> AstChildren<Property> {
        support::children(&self.0)
    }
}

ast_node!(TryStatement, TryStatement);

impl TryStatement {
    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }

    pub fn catch_clause(&self) -> Option<CatchClause> {
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
expr_wrapper!(ExpressionStatement);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
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
}

impl AstNode for Stmt {
    type Language = SquirrelLanguage;

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::EmptyStatement
                | SyntaxKind::BlockStatement
                | SyntaxKind::IfStatement
                | SyntaxKind::WhileStatement
                | SyntaxKind::DoWhileStatement
                | SyntaxKind::ForStatement
                | SyntaxKind::ForEachStatement
                | SyntaxKind::SwitchStatement
                | SyntaxKind::ConstStatement
                | SyntaxKind::LocalVariableDeclaration
                | SyntaxKind::LocalFunctionDeclaration
                | SyntaxKind::ReturnStatement
                | SyntaxKind::YieldStatement
                | SyntaxKind::ContinueStatement
                | SyntaxKind::BreakStatement
                | SyntaxKind::FunctionStatement
                | SyntaxKind::ClassStatement
                | SyntaxKind::EnumStatement
                | SyntaxKind::TryStatement
                | SyntaxKind::ThrowStatement
                | SyntaxKind::ExpressionStatement
        )
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        Some(match node.kind() {
            SyntaxKind::EmptyStatement => Stmt::Empty(EmptyStatement(node)),
            SyntaxKind::BlockStatement => Stmt::Block(BlockStatement(node)),
            SyntaxKind::IfStatement => Stmt::If(IfStatement(node)),
            SyntaxKind::WhileStatement => Stmt::While(WhileStatement(node)),
            SyntaxKind::DoWhileStatement => Stmt::DoWhile(DoWhileStatement(node)),
            SyntaxKind::ForStatement => Stmt::For(ForStatement(node)),
            SyntaxKind::ForEachStatement => Stmt::ForEach(ForEachStatement(node)),
            SyntaxKind::SwitchStatement => Stmt::Switch(SwitchStatement(node)),
            SyntaxKind::ConstStatement => Stmt::Const(ConstStatement(node)),
            SyntaxKind::LocalVariableDeclaration => {
                Stmt::LocalVariable(LocalVariableDeclaration(node))
            }
            SyntaxKind::LocalFunctionDeclaration => {
                Stmt::LocalFunction(LocalFunctionDeclaration(node))
            }
            SyntaxKind::ReturnStatement => Stmt::Return(ReturnStatement(node)),
            SyntaxKind::YieldStatement => Stmt::Yield(YieldStatement(node)),
            SyntaxKind::ContinueStatement => Stmt::Continue(ContinueStatement(node)),
            SyntaxKind::BreakStatement => Stmt::Break(BreakStatement(node)),
            SyntaxKind::FunctionStatement => Stmt::Function(FunctionStatement(node)),
            SyntaxKind::ClassStatement => Stmt::Class(ClassStatement(node)),
            SyntaxKind::EnumStatement => Stmt::Enum(EnumStatement(node)),
            SyntaxKind::TryStatement => Stmt::Try(TryStatement(node)),
            SyntaxKind::ThrowStatement => Stmt::Throw(ThrowStatement(node)),
            SyntaxKind::ExpressionStatement => Stmt::Expression(ExpressionStatement(node)),
            _ => return None,
        })
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Stmt::Empty(n) => n.syntax(),
            Stmt::Block(n) => n.syntax(),
            Stmt::If(n) => n.syntax(),
            Stmt::While(n) => n.syntax(),
            Stmt::DoWhile(n) => n.syntax(),
            Stmt::For(n) => n.syntax(),
            Stmt::ForEach(n) => n.syntax(),
            Stmt::Switch(n) => n.syntax(),
            Stmt::Const(n) => n.syntax(),
            Stmt::LocalVariable(n) => n.syntax(),
            Stmt::LocalFunction(n) => n.syntax(),
            Stmt::Return(n) => n.syntax(),
            Stmt::Yield(n) => n.syntax(),
            Stmt::Continue(n) => n.syntax(),
            Stmt::Break(n) => n.syntax(),
            Stmt::Function(n) => n.syntax(),
            Stmt::Class(n) => n.syntax(),
            Stmt::Enum(n) => n.syntax(),
            Stmt::Try(n) => n.syntax(),
            Stmt::Throw(n) => n.syntax(),
            Stmt::Expression(n) => n.syntax(),
        }
    }
}

ast_node!(Initialiser, Initialiser);
expr_wrapper!(Initialiser);

ast_node!(VariableDeclaration, VariableDeclaration);

impl VariableDeclaration {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }

    pub fn initialiser(&self) -> Option<Initialiser> {
        support::child(&self.0)
    }
}

ast_node!(ParameterList, ParameterList);

impl ParameterList {
    pub fn environment(&self) -> Option<Environment> {
        support::child(&self.0)
    }

    pub fn parameters(&self) -> AstChildren<VariableDeclaration> {
        support::children(&self.0)
    }

    pub fn is_variadic(&self) -> bool {
        support::child::<VariedArgs>(&self.0).is_some()
    }
}

ast_node!(Environment, Environment);
expr_wrapper!(Environment);

ast_node!(VariedArgs, VariedArgs);

ast_node!(CatchClause, CatchClause);

impl CatchClause {
    pub fn binding(&self) -> Option<VariableDeclaration> {
        support::child(&self.0)
    }

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

ast_node!(Extends, Extends);
expr_wrapper!(Extends);

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

impl Property {
    pub fn name(&self) -> Option<MemberName> {
        if let Some(n) = support::child::<Name>(&self.0) {
            return Some(MemberName::Identifier(n));
        }
        if let Some(n) = support::child::<StringName>(&self.0) {
            return Some(MemberName::String(n));
        }
        if let Some(n) = support::child::<ComputedName>(&self.0) {
            return Some(MemberName::Computed(n));
        }
        None
    }

    pub fn value(&self) -> Option<Expr> {
        support::child(&self.0)
    }
}

ast_node!(StringName, StringName);

impl StringName {
    pub fn token(&self) -> Option<SyntaxToken> {
        first_token(&self.0)
    }
}

ast_node!(ComputedName, ComputedName);
expr_wrapper!(ComputedName);

pub enum MemberName {
    Identifier(Name),
    String(StringName),
    Computed(ComputedName),
}

ast_node!(Constructor, Constructor);

impl Constructor {
    pub fn parameter_list(&self) -> Option<ParameterList> {
        support::child(&self.0)
    }

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

ast_node!(Method, Method);

impl Method {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.0)
    }

    pub fn parameter_list(&self) -> Option<ParameterList> {
        support::child(&self.0)
    }

    pub fn body(&self) -> Option<Stmt> {
        support::child(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClassMember {
    Property(Property),
    Constructor(Constructor),
    Method(Method),
}

impl AstNode for ClassMember {
    type Language = SquirrelLanguage;

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::Property | SyntaxKind::Constructor | SyntaxKind::Method
        )
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        Some(match node.kind() {
            SyntaxKind::Property => ClassMember::Property(Property(node)),
            SyntaxKind::Constructor => ClassMember::Constructor(Constructor(node)),
            SyntaxKind::Method => ClassMember::Method(Method(node)),
            _ => return None,
        })
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            ClassMember::Property(n) => n.syntax(),
            ClassMember::Constructor(n) => n.syntax(),
            ClassMember::Method(n) => n.syntax(),
        }
    }
}
