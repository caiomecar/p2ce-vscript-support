mod arena;
mod db;

use std::mem::discriminant;

pub use db::{Database, File, line_index, parse, source_symbol};

use rustc_hash::{FxHashMap, FxHashSet};
use sq_3_parser::{AstNode, SyntaxKind, SyntaxToken, TextRange, TextSize, ast::*};

use crate::arena::{
    Arenas, ArrayData, ArrayId, ClassData, ClassId, Container, EnumData, EnumId, FunctionData,
    FunctionId, ParamsState, SymbolId, SymbolTable, TableData, TableId,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Scope {
    pub range: TextRange,
    pub locals: SymbolTable,
    pub children: Vec<Scope>,
    pub container: Container,
    pub execution_range: TextRange,
}

impl Scope {
    fn new(range: TextRange, container: Container, execution_range: TextRange) -> Scope {
        Self {
            range,
            container,
            locals: SymbolTable::default(),
            children: Vec::new(),
            execution_range,
        }
    }

    fn stack_at(&self, offset: TextSize) -> Vec<&Scope> {
        if !self.range.contains_inclusive(offset) {
            return Vec::new();
        }

        for child in &self.children {
            if !child.range.contains_inclusive(offset) {
                continue;
            }

            let mut result = vec![self];
            result.extend(child.stack_at(offset));
            return result;
        }

        return vec![self];
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    #[default]
    Unknown,
    Integer,
    Float,
    String,
    Boolean,
    Null,
    Instance(ClassId),
    Array(ArrayId),
    Table(TableId),
    Class(ClassId),
    Enum(EnumId),
    Function(FunctionId),
    Generator(FunctionId),
    Thread(FunctionId),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub range: TextRange,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug)]
pub enum TableDelegateError {
    NoDelegate,
    NoMember,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionKind {
    // L value
    Literal(SymbolKind),
    // R value
    Symbol(SymbolId),
    // abc.b
    // abc was found, but not b
    // "b" is returned as .1
    //
    // This is used by <- operator to create a
    // new symbol if it's possible
    Parent(Container, Box<str>),
    Unknown,
}

pub type RangeKindMap = FxHashMap<TextRange, ExpressionKind>;

struct Collector {
    arenas: Arenas,
    container: Container,
    const_table: TableId,
    root_table: TableId,
    scope_stack: Vec<Scope>,
    execution_range: TextRange,
    function: Option<FunctionId>,
    expr_kinds: RangeKindMap,
    diagnostics: Vec<Diagnostic>,
}

impl Collector {
    fn symbol_from_source_file(file: SourceFile) -> SourceSymbol {
        let mut arenas = Arenas::default();
        let container = Container::Table(arenas.tables.alloc(TableData::default()));
        let root_table = arenas.tables.alloc(TableData::default());
        let const_table = arenas.tables.alloc(TableData::default());

        let mut collector = Self {
            arenas,
            container,
            const_table,
            root_table,
            scope_stack: Vec::new(),
            execution_range: file.syntax().text_range(),
            function: None,
            expr_kinds: FxHashMap::default(),
            diagnostics: Vec::new(),
        };

        collector.enter_scope(collector.execution_range);

        for stmt in file.statements() {
            collector.collect_stmt(&stmt);
        }

        let scope = collector.exit_scope().unwrap();

        SourceSymbol::new(
            collector.diagnostics,
            scope,
            collector.arenas,
            collector.const_table,
            collector.root_table,
            match collector.container {
                Container::Table(id) => id,
                _ => unreachable!(),
            },
            collector.expr_kinds,
        )
    }

    fn current_scope(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().unwrap()
    }

    fn enter_scope(&mut self, range: TextRange) {
        self.scope_stack.push(Scope::new(
            range,
            self.container.clone(),
            self.execution_range,
        ));
    }

    fn exit_scope(&mut self) -> Option<Scope> {
        let finished = self.scope_stack.pop().unwrap();
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.children.push(finished);
            None
        } else {
            Some(finished)
        }
    }

    fn collect_params(&mut self, parameters: impl Iterator<Item = Parameter>) {
        let function = self
            .function
            .expect("We should be in the function context if we're collecting params");
        let mut params_state = ParamsState::NoDefault;

        for (count, param) in parameters.enumerate() {
            match param {
                Parameter::Variable(var) => {
                    let Some(name) = var.name().and_then(|n| n.text()) else {
                        let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                            continue;
                        };

                        self.collect_expr(&expr);
                        continue;
                    };

                    let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                        match params_state {
                            ParamsState::Default(_) => {
                                self.diagnostics.push(Diagnostic {
                                    message: "Non-default parameter cannot be preceded by a default parameter".into(),
                                    range: var.syntax().text_range(),
                                    severity: DiagnosticSeverity::Error,
                                });
                                params_state = ParamsState::NoDefault;
                            }
                            ParamsState::VarArgs(_) => {
                                self.diagnostics.push(Diagnostic {
                                    message: "Parameters cannot be preceded by varied args".into(),
                                    range: var.syntax().text_range(),
                                    severity: DiagnosticSeverity::Error,
                                });
                                params_state = ParamsState::NoDefault;
                            }
                            ParamsState::NoDefault => {}
                        }

                        let index = self.arenas.symbols.alloc(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Null,
                            range: var.syntax().text_range(),
                        });

                        self.current_scope().locals.insert(name, index);
                        self.arenas.functions[function].params.push(index);
                        continue;
                    };

                    let kind = self.expr_symbol_kind(&expr);

                    let index = self.arenas.symbols.alloc(Symbol {
                        name: name.clone(),
                        kind,
                        range: var.syntax().text_range(),
                    });

                    self.current_scope().locals.insert(name, index);
                    self.arenas.functions[function].params.push(index);
                    match params_state {
                        ParamsState::NoDefault => {
                            params_state = ParamsState::Default(count);
                        }
                        ParamsState::Default(_) => {}
                        ParamsState::VarArgs(var_args_at) => {
                            self.diagnostics.push(Diagnostic {
                                message: "Parameters cannot be preceded by varied args".into(),
                                range: var.syntax().text_range(),
                                severity: DiagnosticSeverity::Error,
                            });
                            params_state = ParamsState::Default(var_args_at);
                        }
                    }
                }
                Parameter::Ellipsis(var_args) => match params_state {
                    ParamsState::NoDefault => params_state = ParamsState::VarArgs(count),
                    ParamsState::Default(_) => {
                        self.diagnostics.push(Diagnostic {
                            message: "Function with varied args cannot have default parameters"
                                .into(),
                            range: var_args.syntax().text_range(),
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                    ParamsState::VarArgs(_) => {
                        self.diagnostics.push(Diagnostic {
                            message: "There can't be 2 varied args in a function signature.".into(),
                            range: var_args.syntax().text_range(),
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                },
            };
        }

        self.arenas.functions[function].params_state = params_state;
    }

    fn class_data(&mut self, class: &impl IsClass) -> ClassData {
        let expr = class.extends().and_then(|e| e.expression());

        let inherits = if let Some(expr) = expr {
            match self.expr_symbol_kind(&expr) {
                SymbolKind::Class(id) => Some(id),
                SymbolKind::Unknown => None,
                _ => {
                    self.diagnostics.push(Diagnostic {
                        message: "Trying to inherit non-class type".into(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                    None
                }
            }
        } else {
            None
        };

        let members = match inherits {
            Some(id) => self.arenas.clone_members(id),
            None => SymbolTable::default(),
        };

        ClassData { inherits, members }
    }

    fn get_delegate_member(&self, id: TableId, text: &str) -> Result<SymbolId, TableDelegateError> {
        let table = &self.arenas.tables[id];
        let Some(delegate_idx) = table.delegate else {
            return Err(TableDelegateError::NoDelegate);
        };

        let members = &self.arenas.tables[delegate_idx].members;
        let Some(member) = members.get(text) else {
            return Err(TableDelegateError::NoMember);
        };

        Ok(*member)
    }

    fn collect_property(&mut self, property: &Property, default_value: Option<usize>) {
        let kind = match property.value() {
            Some(expr) => self.expr_symbol_kind(&expr),
            None if default_value.is_some() => SymbolKind::Integer,
            None => SymbolKind::Unknown,
        };

        let name = match property.name() {
            Some(MemberName::Identifier(name)) => name.name().and_then(|n| n.text()),
            Some(MemberName::String(name)) => name.token().and_then(|t| Some(t.text().to_owned())),
            _ => None,
        };

        if let Some(name) = name {
            let index = self.arenas.symbols.alloc(Symbol {
                name: name.clone(),
                kind,
                range: property.syntax().text_range(),
            });
            self.arenas
                .add_container_member(self.container, name, index);
        }
    }

    // Lambdas are processed differently
    fn collect_function<T>(&mut self, idx: FunctionId, node: &T)
    where
        T: IsFunction + HasBody,
    {
        let save_function = self.function;
        self.function = Some(idx);
        let save_execution = self.execution_range;
        self.execution_range = if let Some(body) = node.body() {
            body.syntax().text_range()
        } else {
            TextRange::empty(node.syntax().text_range().end())
        };
        self.enter_scope(self.execution_range);

        if let Some(param_list) = node.parameter_list() {
            self.collect_params(param_list.parameters());
        }

        if let Some(body) = node.body() {
            self.collect_stmt(&body);
        }

        self.exit_scope();
        self.function = save_function;
        self.execution_range = save_execution;
    }

    fn collect_member(&mut self, member: &Member) {
        match member {
            Member::Property(property) => self.collect_property(property, None),
            Member::Method(method) => {
                let idx = self.arenas.functions.alloc(FunctionData::default());
                if let Some(name) = method.name().and_then(|n| n.text()) {
                    let symbol = self.arenas.symbols.alloc(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Function(idx),
                        range: method.syntax().text_range(),
                    });
                    self.arenas
                        .add_container_member(self.container, name, symbol);
                }
                self.collect_function(idx, method);
            }
            Member::Constructor(constructor) => {
                let idx = self.arenas.functions.alloc(FunctionData::default());

                let symbol = self.arenas.symbols.alloc(Symbol {
                    name: "constructor".to_owned(),
                    kind: SymbolKind::Function(idx),
                    range: constructor.syntax().text_range(),
                });

                self.arenas
                    .add_container_member(self.container, "constructor".to_owned(), symbol);

                self.collect_function(idx, constructor);
            }
        }
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::LocalVariable(stmt) => self.local_variable(stmt),
            Stmt::LocalFunction(stmt) => self.local_function(stmt),
            Stmt::Block(stmt) => self.block_statement(stmt),
            Stmt::Const(stmt) => self.const_statement(stmt),
            Stmt::ForEach(stmt) => self.for_each_statement(stmt),
            Stmt::For(stmt) => self.for_statement(stmt),
            Stmt::Class(stmt) => self.class_statement(stmt),
            Stmt::Function(stmt) => self.function_statement(stmt),
            Stmt::Enum(stmt) => self.enum_statement(stmt),
            Stmt::Expression(stmt) => self.expression_statement(stmt),
            Stmt::Empty(_) => (),
            Stmt::If(stmt) => self.if_statement(stmt),
            Stmt::While(stmt) => self.while_statement(stmt),
            Stmt::DoWhile(stmt) => self.do_while_statement(stmt),
            Stmt::Switch(stmt) => self.switch_statement(stmt),
            Stmt::Return(stmt) => self.return_statement(stmt),
            Stmt::Yield(stmt) => self.yield_statement(stmt),
            Stmt::Continue(_) => (),
            Stmt::Break(_) => (),
            Stmt::Try(stmt) => self.try_statement(stmt),
            Stmt::Throw(stmt) => self.throw_statement(stmt),
        }
    }

    fn local_variable(&mut self, decl: &LocalVariableDeclaration) {
        for var in decl.declarations() {
            let Some(name) = var.name().and_then(|n| n.text()) else {
                let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                    continue;
                };

                self.collect_expr(&expr);
                continue;
            };

            let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                let index = self.arenas.symbols.alloc(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Null,
                    range: var.syntax().text_range(),
                });

                self.current_scope().locals.insert(name, index);
                continue;
            };

            let kind = self.expr_symbol_kind(&expr);

            let index = self.arenas.symbols.alloc(Symbol {
                name: name.clone(),
                kind,
                range: var.syntax().text_range(),
            });

            self.current_scope().locals.insert(name, index);
        }
    }

    fn local_function(&mut self, decl: &LocalFunctionDeclaration) {
        let idx = self.arenas.functions.alloc(FunctionData::default());
        if let Some(name) = decl.name().and_then(|n| n.text()) {
            let symbol = self.arenas.symbols.alloc(Symbol {
                name: name.clone(),
                kind: SymbolKind::Function(idx),
                range: decl.syntax().text_range(),
            });

            self.current_scope().locals.insert(name, symbol);
        }
        self.collect_function(idx, decl);
    }
    fn block_statement(&mut self, stmt: &BlockStatement) {
        self.enter_scope(stmt.syntax().text_range());
        for stmt in stmt.statements() {
            self.collect_stmt(&stmt);
        }
        self.exit_scope();
    }

    fn const_statement(&mut self, stmt: &ConstStatement) {
        let kind = match stmt.value() {
            Some(expr) => self.expr_symbol_kind(&expr),
            None => SymbolKind::Unknown,
        };

        let Some(name) = stmt.name().and_then(|n| n.text()) else {
            return;
        };

        let index = self.arenas.symbols.alloc(Symbol {
            name: name.clone(),
            kind,
            range: stmt.syntax().text_range(),
        });
        self.arenas.tables[self.const_table].add_member(name, index);
    }

    fn for_each_statement(&mut self, stmt: &ForEachStatement) {
        if let Some(body) = stmt.body() {
            self.enter_scope(body.syntax().text_range());
        } else {
            self.enter_scope(TextRange::empty(stmt.syntax().text_range().end()));
        }

        let iterable = match stmt.iterable() {
            Some(i) => self.expr_symbol_kind(&i),
            None => SymbolKind::Unknown,
        };

        let (key_kind, value_kind) = match iterable {
            SymbolKind::Array(id) => {
                let array = &self.arenas.arrays[id];
                (SymbolKind::Integer, array.kind)
            }
            _ => (SymbolKind::String, SymbolKind::Unknown),
        };

        if let Some(key) = stmt.key() {
            if let Some(name) = key.name().and_then(|n| n.text()) {
                let index = self.arenas.symbols.alloc(Symbol {
                    name: name.clone(),
                    kind: key_kind,
                    range: key.syntax().text_range(),
                });

                self.current_scope().locals.insert(name, index);
            }
        }

        if let Some(value) = stmt.value() {
            if let Some(name) = value.name().and_then(|n| n.text()) {
                let index = self.arenas.symbols.alloc(Symbol {
                    name: name.clone(),
                    kind: value_kind,
                    range: value.syntax().text_range(),
                });

                self.current_scope().locals.insert(name, index);
            }
        }

        if let Some(body) = stmt.body() {
            self.collect_stmt(&body);
        }

        self.exit_scope();
    }

    fn for_statement(&mut self, stmt: &ForStatement) {
        self.enter_scope(stmt.syntax().text_range());
        match stmt.initialiser().and_then(|i| i.kind()) {
            Some(ForInitialiserKind::LocalVariableDeclaration(decl)) => self.local_variable(&decl),
            Some(ForInitialiserKind::LocalFunctionDeclaration(decl)) => self.local_function(&decl),
            Some(ForInitialiserKind::Expression(expr)) => {
                self.collect_expr(&expr);
            }
            None => {}
        }
        self.exit_scope();
    }

    fn class_statement(&mut self, stmt: &ClassStatement) {
        let class_data = self.class_data(stmt);
        let id = self.arenas.classes.alloc(class_data);

        let name = match stmt.name() {
            Some(Expr::Name(name)) => name.text(),
            _ => None,
        };

        if let Some(name) = name {
            let symbol = self.arenas.symbols.alloc(Symbol {
                kind: SymbolKind::Class(id),
                name: name.clone(),
                range: stmt.syntax().text_range(),
            });
            self.arenas
                .add_container_member(self.container, name, symbol);
        }

        let save_symbol = self.container;
        self.container = Container::Class(id);
        for member in stmt.members() {
            self.collect_member(&member);
        }
        self.container = save_symbol;
    }

    fn function_statement(&mut self, stmt: &FunctionStatement) {
        let idx = self.arenas.functions.alloc(FunctionData::default());

        if let Some(name) = stmt
            .name()
            .and_then(|n| n.names().last().and_then(|n| n.text()))
        {
            let symbol = self.arenas.symbols.alloc(Symbol {
                name: name.clone(),
                kind: SymbolKind::Function(idx),
                range: stmt.syntax().text_range(),
            });

            self.arenas
                .add_container_member(self.container, name, symbol);
        }

        self.collect_function(idx, stmt);
    }

    fn enum_statement(&mut self, stmt: &EnumStatement) {
        let idx = self.arenas.enums.alloc(EnumData::default());

        if let Some(name) = stmt.name().and_then(|n| n.text()) {
            let symbol = self.arenas.symbols.alloc(Symbol {
                name: name.clone(),
                kind: SymbolKind::Enum(idx),
                range: stmt.syntax().text_range(),
            });
            self.arenas
                .add_container_member(self.container, name, symbol);
        }

        let save_symbol = self.container;
        self.container = Container::Enum(idx);
        for (value, property) in stmt.members().enumerate() {
            self.collect_property(&property, Some(value));
        }
        self.container = save_symbol;
    }

    fn expression_statement(&mut self, stmt: &ExpressionStatement) {
        let Some(expr) = stmt.expression() else {
            return;
        };

        self.collect_expr(&expr);
    }

    fn if_statement(&mut self, stmt: &IfStatement) {
        if let Some(condition) = stmt.condition() {
            self.collect_expr(&condition);
        }

        if let Some(then_branch) = stmt.then_branch() {
            self.enter_scope(then_branch.syntax().text_range());
            self.collect_stmt(&then_branch);
            self.exit_scope();
        }

        if let Some(else_branch) = stmt.else_branch() {
            self.enter_scope(else_branch.syntax().text_range());
            self.collect_stmt(&else_branch);
            self.exit_scope();
        }
    }

    fn while_statement(&mut self, stmt: &WhileStatement) {
        if let Some(condition) = stmt.condition() {
            self.collect_expr(&condition);
        }

        if let Some(body) = stmt.body() {
            self.enter_scope(body.syntax().text_range());
            self.collect_stmt(&body);
            self.exit_scope();
        }
    }

    fn do_while_statement(&mut self, stmt: &DoWhileStatement) {
        if let Some(body) = stmt.body() {
            self.enter_scope(body.syntax().text_range());
            self.collect_stmt(&body);
            self.exit_scope();
        }

        if let Some(condition) = stmt.condition() {
            self.collect_expr(&condition);
        }
    }

    fn switch_statement(&mut self, stmt: &SwitchStatement) {
        let kind = if let Some(discriminant) = stmt.discriminant() {
            self.expr_symbol_kind(&discriminant)
        } else {
            SymbolKind::Unknown
        };

        for clause in stmt.clauses() {
            match clause {
                SwitchClause::Case(case) => {
                    if let Some(test) = case.test() {
                        let case_kind = self.expr_symbol_kind(&test);
                        if !matches!(
                            (kind, case_kind),
                            (SymbolKind::Null, _)
                                | (_, SymbolKind::Null)
                                | (SymbolKind::Unknown, _)
                                | (_, SymbolKind::Unknown)
                                | (
                                    SymbolKind::Integer | SymbolKind::Float,
                                    SymbolKind::Integer | SymbolKind::Float,
                                )
                                | (SymbolKind::String, SymbolKind::String)
                                | (SymbolKind::Boolean, SymbolKind::Boolean)
                        ) {
                            self.diagnostics.push(Diagnostic {
                                message: "Case is incompitable with discriminant type".into(),
                                range: test.syntax().text_range(),
                                severity: DiagnosticSeverity::Warning,
                            });
                        }
                    }

                    self.enter_scope(case.syntax().text_range());
                    for stmt in case.body() {
                        self.collect_stmt(&stmt);
                    }
                    self.exit_scope();
                }
                SwitchClause::Default(default) => {
                    self.enter_scope(default.syntax().text_range());
                    for stmt in default.body() {
                        self.collect_stmt(&stmt);
                    }
                    self.exit_scope();
                }
            }
        }
    }

    fn return_statement(&mut self, stmt: &ReturnStatement) {
        let kind = if let Some(value) = stmt.value() {
            Some(self.expr_symbol_kind(&value))
        } else {
            None
        };

        let Some(function) = self.function else {
            if kind.is_some() {
                self.diagnostics.push(Diagnostic {
                    message: "Returning a value in the source file execution scope".into(),
                    range: stmt.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                });
            }
            return;
        };

        if !matches!(
            self.arenas.functions[function].ret,
            SymbolKind::Unknown | SymbolKind::Null
        ) {
            return;
        }

        match kind {
            None | Some(SymbolKind::Unknown) => {}
            Some(kind) => {
                self.arenas.functions[function].ret = kind;
            }
        }
    }

    fn yield_statement(&mut self, stmt: &YieldStatement) {
        let kind = if let Some(value) = stmt.value() {
            self.expr_symbol_kind(&value)
        } else {
            SymbolKind::Null
        };

        let Some(function) = self.function else {
            self.diagnostics.push(Diagnostic {
                message: "Yielding in the source file execution scope".into(),
                range: stmt.syntax().text_range(),
                severity: DiagnosticSeverity::Warning,
            });
            return;
        };

        if self.arenas.functions[function].yielding.is_none() {
            self.arenas.functions[function].yielding = Some(kind);
        }
    }

    fn try_statement(&mut self, stmt: &TryStatement) {
        if let Some(body) = stmt.body() {
            self.enter_scope(body.syntax().text_range());
            self.collect_stmt(&body);
            self.exit_scope();
        }

        let Some(catch) = stmt.catch_clause() else {
            return;
        };

        self.enter_scope(if let Some(body) = catch.body() {
            body.syntax().text_range()
        } else {
            TextRange::empty(catch.syntax().text_range().end())
        });

        if let Some(binding) = catch.binding() {
            if let Some(name) = binding.name().and_then(|n| n.text()) {
                let index = self.arenas.symbols.alloc(Symbol {
                    kind: SymbolKind::String,
                    name: name.clone(),
                    range: binding.syntax().text_range(),
                });

                self.current_scope().locals.insert(name, index);
            };
        }

        if let Some(body) = catch.body() {
            self.collect_stmt(&body);
        }
    }

    fn throw_statement(&mut self, stmt: &ThrowStatement) {
        // mark current function as exception throwing
        let kind = if let Some(value) = stmt.value() {
            self.expr_symbol_kind(&value)
        } else {
            SymbolKind::Unknown
        };

        let Some(function) = self.function else {
            return;
        };

        if self.arenas.functions[function].throwing.is_none() {
            self.arenas.functions[function].throwing = Some(kind);
        }
    }

    fn expr_symbol_kind(&mut self, expr: &Expr) -> SymbolKind {
        let kind = self.collect_expr(expr);
        self.arenas.expr_to_symbol_kind(&kind)
    }

    fn collect_expr(&mut self, expr: &Expr) -> ExpressionKind {
        let key = expr.syntax().text_range();
        let kind = self.expr_kind(expr);
        self.expr_kinds.insert(key, kind.clone());
        kind
    }

    fn expr_kind(&mut self, expr: &Expr) -> ExpressionKind {
        match expr {
            Expr::Literal(expr) => self.literal_expression(expr),
            Expr::TableLiteral(expr) => self.table_literal_expression(expr),
            Expr::Class(expr) => self.class_expression(expr),
            Expr::ArrayLiteral(expr) => self.array_literal_expression(expr),
            Expr::Name(expr) => self.name_expression(expr),
            Expr::This(expr) => self.this_expression(expr),
            Expr::RootAccess(expr) => self.root_access_expression(expr),
            Expr::Base(expr) => self.base_expression(expr),
            Expr::MemberAccess(expr) => self.member_access_expression(expr),
            Expr::ElementAccess(expr) => self.element_access_expression(expr),
            Expr::Call(expr) => self.call_expression(expr),
            Expr::Clone(expr) => self.clone_expression(expr),
            Expr::Binary(expr) => self.binary_expression(expr),
            Expr::Conditional(conditional_expression) => todo!(),
            Expr::PrefixUnary(prefix_unary_expression) => todo!(),
            Expr::PrefixUpdate(prefix_update_expression) => todo!(),
            Expr::PostfixUpdate(postfix_update_expression) => todo!(),
            Expr::Delete(delete_expression) => todo!(),
            Expr::TypeOf(type_of_expression) => todo!(),
            Expr::Resume(resume_expression) => todo!(),
            Expr::RawCall(raw_call_expression) => todo!(),
            Expr::File(_) => ExpressionKind::Literal(SymbolKind::String),
            Expr::Line(_) => ExpressionKind::Literal(SymbolKind::Integer),
            Expr::Parenthesised(expr) => self.parenthesised_expression(expr),
            Expr::Function(expr) => self.function_expression(expr),
            Expr::Lambda(expr) => self.lambda_expression(expr),
        }
    }

    fn literal_expression(&mut self, expr: &LiteralExpression) -> ExpressionKind {
        match expr.token().unwrap().kind() {
            SyntaxKind::Integer => ExpressionKind::Literal(SymbolKind::Integer),
            SyntaxKind::Character => ExpressionKind::Literal(SymbolKind::Integer),
            SyntaxKind::Float => ExpressionKind::Literal(SymbolKind::Float),
            SyntaxKind::String => ExpressionKind::Literal(SymbolKind::String),
            SyntaxKind::VerbatimString => ExpressionKind::Literal(SymbolKind::String),
            SyntaxKind::NullKeyword => ExpressionKind::Literal(SymbolKind::Null),
            SyntaxKind::TrueKeyword => ExpressionKind::Literal(SymbolKind::Boolean),
            SyntaxKind::FalseKeyword => ExpressionKind::Literal(SymbolKind::Boolean),
            _ => unreachable!(),
        }
    }

    fn table_literal_expression(&mut self, expr: &TableLiteralExpression) -> ExpressionKind {
        let id = self.arenas.tables.alloc(TableData::default());
        let save_symbol = self.container;
        self.container = Container::Table(id);
        for member in expr.members() {
            self.collect_member(&member);
        }
        self.container = save_symbol;

        ExpressionKind::Literal(SymbolKind::Table(id))
    }

    fn class_expression(&mut self, expr: &ClassExpression) -> ExpressionKind {
        let class_data = self.class_data(expr);
        let id = self.arenas.classes.alloc(class_data);

        let save_symbol = self.container;
        self.container = Container::Class(id);
        for member in expr.members() {
            self.collect_member(&member);
        }
        self.container = save_symbol;

        ExpressionKind::Literal(SymbolKind::Class(id))
    }

    fn array_literal_expression(&mut self, expr: &ArrayLiteralExpression) -> ExpressionKind {
        let mut kinds = expr
            .elements()
            .map(|element| self.expr_symbol_kind(&element));

        let Some(kind) = kinds.next() else {
            return ExpressionKind::Literal(SymbolKind::Array(self.arenas.arrays.alloc(
                ArrayData {
                    kind: SymbolKind::Unknown,
                },
            )));
        };

        ExpressionKind::Literal(
            if kinds.all(|k| std::mem::discriminant(&k) == std::mem::discriminant(&kind)) {
                SymbolKind::Array(self.arenas.arrays.alloc(ArrayData { kind }))
            } else {
                SymbolKind::Array(self.arenas.arrays.alloc(ArrayData {
                    kind: SymbolKind::Unknown,
                }))
            },
        )
    }

    fn name_expression(&mut self, expr: &Name) -> ExpressionKind {
        let Some(text) = expr.text() else {
            return ExpressionKind::Unknown;
        };

        let offset = expr.syntax().text_range().end();
        let find = |idx: SymbolId| {
            let symbol = &self.arenas.symbols[idx];
            if symbol.range.end() >= offset || text != symbol.name {
                return None;
            }
            Some(idx)
        };

        let find_function = |idx: SymbolId| {
            let symbol = &self.arenas.symbols[idx];
            if self.execution_range.contains_range(symbol.range) && symbol.range.end() >= offset
                || text != symbol.name
            {
                return None;
            }
            Some(idx)
        };

        let locals = self
            .scope_stack
            .iter()
            .rev()
            .flat_map(|scope| scope.locals.values().copied())
            .find_map(find);

        let consts = || {
            self.arenas.tables[self.const_table]
                .get_members(&self.arenas)
                .into_iter()
                .find_map(find)
        };

        let members = || {
            self.arenas
                .get_container_members(self.container)
                .into_iter()
                .find_map(find_function)
        };

        let root = || {
            self.arenas.tables[self.root_table]
                .get_members(&self.arenas)
                .into_iter()
                .find_map(find_function)
        };

        return locals
            .or_else(consts)
            .or_else(members)
            .or_else(root)
            .map_or_else(
                || ExpressionKind::Parent(self.container, text.into_boxed_str()),
                |id| ExpressionKind::Symbol(id),
            );
    }

    fn this_expression(&self, _expr: &ThisExpression) -> ExpressionKind {
        ExpressionKind::Literal(match self.container {
            Container::Class(id) => SymbolKind::Class(id),
            Container::Table(id) => SymbolKind::Table(id),
            Container::Enum(id) => SymbolKind::Enum(id),
        })
    }

    fn root_access_expression(&mut self, expr: &RootAccessExpression) -> ExpressionKind {
        let Some(text) = expr.name().and_then(|n| n.text()) else {
            return ExpressionKind::Unknown;
        };

        let offset = expr.syntax().text_range().end();
        self.arenas.tables[self.root_table]
            .get_members(&self.arenas)
            .into_iter()
            .find_map(|idx| {
                let symbol = &self.arenas.symbols[idx];
                if self.execution_range.contains_range(symbol.range) && symbol.range.end() >= offset
                    || text != symbol.name
                {
                    return None;
                }
                Some(idx)
            })
            .map_or_else(
                || ExpressionKind::Parent(Container::Table(self.root_table), text.into_boxed_str()),
                |symbol| ExpressionKind::Symbol(symbol),
            )
    }

    fn base_expression(&mut self, expr: &BaseExpression) -> ExpressionKind {
        match self.container {
            Container::Class(id) => {
                let class = &self.arenas.classes[id];
                if let Some(inherits) = class.inherits {
                    ExpressionKind::Literal(SymbolKind::Class(inherits))
                } else {
                    self.diagnostics.push(Diagnostic {
                        message: "Class doesn't have a superclass".into(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Warning,
                    });
                    ExpressionKind::Literal(SymbolKind::Null)
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: "Trying to access base inside non-class context".into(),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                });
                ExpressionKind::Literal(SymbolKind::Null)
            }
        }
    }

    fn member_access_expression(&mut self, expr: &MemberAccessExpression) -> ExpressionKind {
        let obj = match expr.object() {
            Some(expr) => self.expr_symbol_kind(&expr),
            None => SymbolKind::Unknown,
        };

        let Some(text) = expr
            .member_part()
            .and_then(|m| m.name())
            .and_then(|n| n.text())
        else {
            return ExpressionKind::Unknown;
        };

        let Some(members) = self.arenas.get_kind_members(obj) else {
            return ExpressionKind::Unknown;
        };

        let offset = expr.syntax().text_range().end();
        members
            .into_iter()
            .find_map(|idx| {
                let symbol = &self.arenas.symbols[idx];
                if self.execution_range.contains_range(symbol.range) && symbol.range.end() >= offset
                    || text != symbol.name
                {
                    return None;
                }
                Some(idx)
            })
            .map_or_else(
                || match obj {
                    SymbolKind::Table(id) => {
                        ExpressionKind::Parent(Container::Table(id), text.into_boxed_str())
                    }
                    SymbolKind::Class(id) => {
                        ExpressionKind::Parent(Container::Class(id), text.into_boxed_str())
                    }
                    SymbolKind::Enum(id) => {
                        ExpressionKind::Parent(Container::Enum(id), text.into_boxed_str())
                    }
                    _ => ExpressionKind::Unknown,
                },
                |symbol| ExpressionKind::Symbol(symbol),
            )
    }

    fn element_access_expression(&mut self, expr: &ElementAccessExpression) -> ExpressionKind {
        let obj = match expr.object() {
            Some(expr) => self.expr_symbol_kind(&expr),
            None => SymbolKind::Unknown,
        };

        let Some(members) = self.arenas.get_kind_members(obj) else {
            return ExpressionKind::Unknown;
        };

        ExpressionKind::Unknown
    }

    fn call_symbol_kind(&mut self, expr: &CallExpression, kind: SymbolKind) -> ExpressionKind {
        match kind {
            SymbolKind::Function(id) => {
                let mut last_count = 0;
                let is_variadic = matches!(
                    &self.arenas.functions[id].params_state,
                    ParamsState::VarArgs(_)
                );
                for (count, expr) in expr.arguments().enumerate() {
                    let argument_kind = self.collect_expr(&expr);

                    let param = self.arenas.functions[id].params.get(count);
                    match param {
                        Some(param) => {
                            let symbol = &self.arenas.symbols[*param];
                            match (self.arenas.expr_to_symbol_kind(&argument_kind), symbol.kind) {
                                (SymbolKind::Unknown, required_kind) => {
                                    // If passed in parameter has type of unknown
                                    // we can coerce it to be the type of a required parameter
                                    if let ExpressionKind::Symbol(id) = &argument_kind {
                                        if self.arenas.expr_to_symbol_kind(&argument_kind)
                                            == SymbolKind::Unknown
                                            && required_kind != SymbolKind::Unknown
                                        {
                                            self.arenas.symbols[*id].kind = required_kind;
                                        }
                                    }
                                }

                                (SymbolKind::Null, _)
                                | (_, SymbolKind::Null)
                                | (_, SymbolKind::Unknown)
                                | (
                                    SymbolKind::Integer | SymbolKind::Float,
                                    SymbolKind::Integer | SymbolKind::Float,
                                ) => {}

                                (passed, required) => {
                                    if discriminant(&passed) != discriminant(&required) {
                                        self.diagnostics.push(Diagnostic {
                                            message: "Wrong parameter type is passed in".into(),
                                            range: expr.syntax().text_range(),
                                            severity: DiagnosticSeverity::Warning,
                                        });
                                    }
                                }
                            }
                        }
                        None if !is_variadic => self.diagnostics.push(Diagnostic {
                            message: "Passing more parameters than possible".into(),
                            range: expr.syntax().text_range(),
                            severity: DiagnosticSeverity::Error,
                        }),
                        None => {}
                    }
                    last_count = count;
                }

                let enough_parameters = match self.arenas.functions[id].params_state {
                    ParamsState::NoDefault => last_count == self.arenas.functions[id].params.len(),
                    ParamsState::Default(from) | ParamsState::VarArgs(from) => last_count >= from,
                };

                if !enough_parameters {
                    self.diagnostics.push(Diagnostic {
                        message: "Insufficient number of parameters passed".into(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }

                return ExpressionKind::Literal(if self.arenas.functions[id].yielding.is_some() {
                    SymbolKind::Generator(id)
                } else {
                    self.arenas.clone_kind(self.arenas.functions[id].ret)
                });
            }
            SymbolKind::Class(id) => {
                let class = &self.arenas.classes[id];
                if let Some(symbol) = class.get_member("constructor") {
                    self.call_symbol_kind(expr, self.arenas.symbols[symbol].kind);
                } else {
                    let mut has_parameters = false;
                    for expr in expr.arguments() {
                        self.collect_expr(&expr);
                        has_parameters = true;
                    }

                    if has_parameters {
                        self.diagnostics.push(Diagnostic {
                            message: "Default constructor should have no parameters".into(),
                            range: expr.syntax().text_range(),
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                }
                return ExpressionKind::Literal(SymbolKind::Instance(id));
            }
            SymbolKind::Table(id) => match self.get_delegate_member(id, "_call") {
                Ok(id) => {
                    let kind = self.arenas.symbols[id].kind;
                    return self.call_symbol_kind(expr, kind);
                }
                Err(TableDelegateError::NoDelegate) => {
                    self.diagnostics.push(Diagnostic {
                        message: "Table is uncallable: no delegate assigned".into(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }
                Err(TableDelegateError::NoMember) => {
                    self.diagnostics.push(Diagnostic {
                        message: "Table is uncallable: delegate has no '_call' metamethod".into(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }
            },
            SymbolKind::Instance(id) => {
                let class = &self.arenas.classes[id];
                if let Some(member) = class.get_member("_call") {
                    return self.call_symbol_kind(expr, self.arenas.symbols[member].kind);
                } else {
                    self.diagnostics.push(Diagnostic {
                        message:
                            "Instance is uncallable: class does not define a '_call' metamethod"
                                .into(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }
            }
            SymbolKind::Unknown => {}
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: "Type is not callable".into(),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Error,
                });
            }
        }
        for expr in expr.arguments() {
            self.collect_expr(&expr);
        }
        ExpressionKind::Unknown
    }

    fn call_expression(&mut self, expr: &CallExpression) -> ExpressionKind {
        let obj = match expr.callee() {
            Some(expr) => self.expr_symbol_kind(&expr),
            None => SymbolKind::Unknown,
        };

        self.call_symbol_kind(expr, obj)
    }

    fn clone_expression(&mut self, expr: &CloneExpression) -> ExpressionKind {
        let Some(operand) = expr.operand() else {
            return ExpressionKind::Unknown;
        };

        let kind = self.expr_symbol_kind(&operand);
        ExpressionKind::Literal(self.arenas.clone_kind(kind))
    }

    fn binary_expression(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let Some(left) = expr.lhs() else {
            let Some(right) = expr.rhs() else {
                return ExpressionKind::Unknown;
            };

            return self.collect_expr(&right);
        };

        let left_kind = self.collect_expr(&left);
        let right_kind = if let Some(right) = expr.rhs() {
            self.collect_expr(&right)
        } else {
            ExpressionKind::Unknown
        };

        let Some(operator) = expr.operator().and_then(|o| o.token()) else {
            return ExpressionKind::Unknown;
        };

        match operator.kind() {
            SyntaxKind::Equals | SyntaxKind::LessThanMinus => {
                self.equals_operator(left_kind, right_kind, operator, expr.syntax().text_range())
            }
            SyntaxKind::Comma => right_kind,
            SyntaxKind::EqualsEquals | SyntaxKind::ExclamationEquals => {
                ExpressionKind::Literal(SymbolKind::Boolean)
            }
            SyntaxKind::LessThan
            | SyntaxKind::LessThanEquals
            | SyntaxKind::GreaterThan
            | SyntaxKind::GreaterThanEquals
            | SyntaxKind::GreaterThanGreaterThanGreaterThan => {
                self.comparison_operator(left_kind, right_kind, operator)
            }
            SyntaxKind::PlusEquals | SyntaxKind::Plus => ExpressionKind::Unknown,
            _ => ExpressionKind::Unknown,
        }
    }

    fn equals_operator(
        &mut self,
        left_kind: ExpressionKind,
        right_kind: ExpressionKind,
        operator: SyntaxToken,
        range: TextRange,
    ) -> ExpressionKind {
        match left_kind {
            ExpressionKind::Parent(parent, member) => {
                if operator.kind() == SyntaxKind::LessThanMinus {
                    dbg!(&right_kind);
                    let symbol = self.arenas.symbols.alloc(Symbol {
                        kind: self.arenas.expr_to_symbol_kind(&right_kind),
                        name: member.to_string(),
                        range,
                    });

                    self.arenas
                        .add_container_member(parent, member.into_string(), symbol);
                }

                right_kind
            }
            ExpressionKind::Symbol(symbol) => {
                let kind = self.arenas.expr_to_symbol_kind(&right_kind);

                let symbol = &mut self.arenas.symbols[symbol];

                // Update symbol kind if it's null or unknown
                if matches!(symbol.kind, SymbolKind::Unknown | SymbolKind::Null)
                    && !matches!(kind, SymbolKind::Unknown | SymbolKind::Null)
                {
                    symbol.kind = kind;
                }

                right_kind
            }
            ExpressionKind::Literal(_) | ExpressionKind::Unknown => right_kind,
        }
    }

    fn comparison_operator(
        &mut self,
        left_kind: ExpressionKind,
        right_kind: ExpressionKind,
        operator: SyntaxToken,
    ) -> ExpressionKind {
        let left = self.arenas.expr_to_symbol_kind(&left_kind);
        let right = self.arenas.expr_to_symbol_kind(&right_kind);

        match (left, right) {
            (SymbolKind::Null, _)
            | (_, SymbolKind::Null)
            | (SymbolKind::Unknown, _)
            | (_, SymbolKind::Unknown)
            | (SymbolKind::Integer | SymbolKind::Float, SymbolKind::Integer | SymbolKind::Float)
            | (SymbolKind::String, SymbolKind::String)
            | (SymbolKind::Boolean, SymbolKind::Boolean) => {}
            // (SymbolKind::Table(id), _) => match self.get_delegate_member(id, "_cmp") {
            //     Ok(id) => {
            //         self.call_symbol_kind(expr, self.arenas.symbols[id].kind)
            //         //
            //     }
            //     Err(TableDelegateError::NoDelegate) => {
            //         self.diagnostics.push(Diagnostic {
            //             message: "Table is uncomparable: no delegate assigned".into(),
            //             range: operator.text_range(),
            //             severity: DiagnosticSeverity::Error,
            //         });
            //     }
            //     Err(TableDelegateError::NoMember) => {
            //         self.diagnostics.push(Diagnostic {
            //             message: "Table is uncomparable: delegate has no '_cmp' metamethod".into(),
            //             range: operator.text_range(),
            //             severity: DiagnosticSeverity::Error,
            //         });
            //     }
            // },

            // (SymbolKind::Instance(id), _) => match self.get_class_method(id, "_cmp") {
            //     Ok(id) => {
            //         let func = &self.arenas.functions[id];
            //     }
            //     Err(ClassMethodError::NoMember) => {
            //         self.diagnostics.push(Diagnostic {
            //             message:
            //                 "Instance is uncomparable: class does not define a '_cmp' metamethod"
            //                     .into(),
            //             range: operator.text_range(),
            //             severity: DiagnosticSeverity::Error,
            //         });
            //     }
            //     Err(ClassMethodError::NotAMethod) => {
            //         self.diagnostics.push(Diagnostic {
            //             message: "Instance is uncomparable: class's '_call' member is not a method"
            //                 .into(),
            //             range: operator.text_range(),
            //             severity: DiagnosticSeverity::Error,
            //         });
            //     }
            // },
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: "Cannot compare these types".into(),
                    range: operator.text_range(),
                    severity: DiagnosticSeverity::Error,
                });
            }
        }

        ExpressionKind::Literal(
            if operator.kind() == SyntaxKind::GreaterThanGreaterThanGreaterThan {
                SymbolKind::Integer
            } else {
                SymbolKind::Boolean
            },
        )
    }

    fn parenthesised_expression(&mut self, expr: &ParenthesisedExpression) -> ExpressionKind {
        let Some(expr) = expr.inner() else {
            return ExpressionKind::Unknown;
        };

        self.collect_expr(&expr)
    }

    fn function_expression(&mut self, expr: &FunctionExpression) -> ExpressionKind {
        let idx = self.arenas.functions.alloc(FunctionData::default());
        self.collect_function(idx, expr);
        ExpressionKind::Literal(SymbolKind::Function(idx))
    }

    fn lambda_expression(&mut self, expr: &LambdaExpression) -> ExpressionKind {
        let idx = self.arenas.functions.alloc(FunctionData::default());

        let save_function = self.function;
        self.function = Some(idx);
        let save_execution = self.execution_range;
        self.execution_range = if let Some(body) = expr.body() {
            body.syntax().text_range()
        } else {
            TextRange::empty(expr.syntax().text_range().end())
        };
        self.enter_scope(self.execution_range);

        if let Some(param_list) = expr.parameter_list() {
            self.collect_params(param_list.parameters());
        }

        let ret = if let Some(body) = expr.body() {
            self.expr_symbol_kind(&body)
        } else {
            SymbolKind::Unknown
        };

        self.arenas.functions[idx].ret = ret;

        self.exit_scope();
        self.function = save_function;
        self.execution_range = save_execution;

        ExpressionKind::Literal(SymbolKind::Function(idx))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSymbol {
    diagnostics: Vec<Diagnostic>,
    scope: Scope,
    arenas: Arenas,
    const_table: TableId,
    root_table: TableId,
    source_table: TableId,
    range_kind: RangeKindMap,
}

impl SourceSymbol {
    pub fn new(
        diagnostics: Vec<Diagnostic>,
        scope: Scope,
        arenas: Arenas,
        const_table: TableId,
        root_table: TableId,
        source_table: TableId,
        range_kind: RangeKindMap,
    ) -> SourceSymbol {
        dbg!(&range_kind);
        Self {
            diagnostics,
            scope,
            arenas,
            const_table,
            root_table,
            source_table,
            range_kind,
        }
    }

    pub fn symbols_at(&self, offset: TextSize) -> Vec<&Symbol> {
        let stack = self.scope.stack_at(offset);
        let mut taken_names = FxHashSet::default();
        let mut items = Vec::new();

        {
            let mut filter = |idx: SymbolId| {
                let symbol = &self.arenas.symbols[idx];

                if symbol.range.end() >= offset || !taken_names.insert(symbol.name.clone()) {
                    return None;
                }

                Some(symbol)
            };

            // Locals first
            items.extend(
                stack
                    .iter()
                    .rev()
                    .flat_map(|scope| scope.locals.values().copied())
                    .filter_map(&mut filter),
            );

            // Consts second
            items.extend(
                self.arenas.tables[self.const_table]
                    .get_members(&self.arenas)
                    .into_iter()
                    .filter_map(&mut filter),
            );
        }

        {
            let last_scope = stack.last().unwrap();
            // We ignore member positions if they're declared outside of the current function
            let enclosing_function = last_scope.execution_range;

            let mut filter_function = |idx: SymbolId| {
                let symbol = &self.arenas.symbols[idx];

                if enclosing_function.contains_range(symbol.range) && symbol.range.end() >= offset
                    || !taken_names.insert(symbol.name.clone())
                {
                    return None;
                }

                Some(symbol)
            };

            // Members third
            items.extend(
                self.arenas
                    .get_container_members(last_scope.container)
                    .into_iter()
                    .filter_map(&mut filter_function),
            );

            // Root fourth
            items.extend(
                self.arenas.tables[self.root_table]
                    .get_members(&self.arenas)
                    .into_iter()
                    .filter_map(&mut filter_function),
            );
        }

        items
    }

    pub fn expr_kind_at(&self, text_range: TextRange) -> Option<ExpressionKind> {
        self.range_kind.get(&text_range).cloned()
    }

    pub fn symbol_kind_at(&self, text_range: TextRange) -> Option<SymbolKind> {
        Some(
            self.arenas
                .expr_to_symbol_kind(self.range_kind.get(&text_range)?),
        )
    }

    pub fn members_of_kind(&self, kind: SymbolKind) -> Option<Vec<&Symbol>> {
        let members = match kind {
            SymbolKind::Table(id) => self.arenas.tables[id].get_members(&self.arenas),
            SymbolKind::Class(id) => self.arenas.classes[id].get_members(),
            SymbolKind::Instance(id) => self.arenas.classes[id].get_members(),
            SymbolKind::Enum(id) => self.arenas.enums[id].get_members(),
            _ => return None,
        };

        Some(
            members
                .into_iter()
                .map(|idx| &self.arenas.symbols[idx])
                .collect(),
        )
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
