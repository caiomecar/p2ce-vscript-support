use std::mem::discriminant;

use rustc_hash::FxHashMap;
use sq_3_parser::{AstNode, SyntaxKind, SyntaxToken, TextRange, TextSize, ast::*};

use crate::{
    Diagnostic, DiagnosticSeverity, SourceSymbol,
    arena::{
        Arenas, ArrayData, ClassData, ClassId, Container, EnumData, FunctionData, FunctionId,
        ParamsState, SymbolId, TableData, TableId,
    },
    symbol::{Symbol, SymbolTable, Type},
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

    pub fn stack_at(&self, offset: TextSize) -> Vec<&Scope> {
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

#[derive(Debug)]
pub enum TableDelegateError {
    NoDelegate,
    NoMember,
}

#[derive(Debug)]
pub enum NewSlotApplicable {
    Table(TableId),
    Class(ClassId),
    Instance(ClassId),
}

impl From<NewSlotApplicable> for Container {
    fn from(value: NewSlotApplicable) -> Self {
        match value {
            NewSlotApplicable::Table(id) => Container::Table(id),
            NewSlotApplicable::Class(id) | NewSlotApplicable::Instance(id) => Container::Class(id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionKind {
    // L value
    Literal(Type),
    // R value with optional parent
    Symbol(Option<Type>, SymbolId),
    // abc.b
    // abc was found, but not b
    // "b" is returned as .1
    //
    // This is used by <- operator to create a
    // new symbol if it's possible
    Parent(Type, Box<str>),
    Unknown,
}

pub type RangeKindMap = FxHashMap<TextRange, ExpressionKind>;

fn unquote_string(input: &str) -> String {
    if let Some(stripped) = input.strip_prefix("@\"") {
        // Verbatim string
        let inner = stripped.strip_suffix('"').unwrap_or(stripped);
        inner.replace("\"\"", "\\\"")
    } else if let Some(stripped) = input.strip_prefix('"') {
        // Normal string
        stripped.strip_suffix('"').unwrap_or(stripped).to_string()
    } else {
        // Not quoted
        input.to_string()
    }
}
pub struct Collector {
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
    pub fn symbol_from_source_file(file: SourceFile) -> SourceSymbol {
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
                                    message: "Non-default parameter cannot be preceded by a default parameter".to_owned(),
                                    range: var.syntax().text_range(),
                                    severity: DiagnosticSeverity::Error,
                                });
                                params_state = ParamsState::NoDefault;
                            }
                            ParamsState::VarArgs(_) => {
                                self.diagnostics.push(Diagnostic {
                                    message: "Parameters cannot be preceded by varied arguments"
                                        .to_owned(),
                                    range: var.syntax().text_range(),
                                    severity: DiagnosticSeverity::Error,
                                });
                                params_state = ParamsState::NoDefault;
                            }
                            ParamsState::NoDefault => {}
                        }

                        let index = self.arenas.symbols.alloc(Symbol {
                            name: name.clone(),
                            kind: Type::Null,
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
                                message: "Parameters cannot be preceded by varied arguments"
                                    .to_owned(),
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
                            message:
                                "Function with varied arguments cannot have default parameters"
                                    .to_owned(),
                            range: var_args.syntax().text_range(),
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                    ParamsState::VarArgs(_) => {
                        self.diagnostics.push(Diagnostic {
                            message: "There can't be 2 varied arguments in a function signature"
                                .to_owned(),
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
                Type::Class(id) => Some(id),
                Type::Unknown => None,
                kind => {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Trying to inherit from {kind}"),
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
            None if default_value.is_some() => Type::Integer,
            None => Type::Unknown,
        };

        let Some(name) = (match property.name() {
            Some(MemberName::Identifier(name)) => name.name().and_then(|n| n.text()),
            Some(MemberName::String(name)) => {
                name.token().and_then(|t| Some(unquote_string(t.text())))
            }
            Some(MemberName::Computed(name)) => {
                let Some(expr) = name.expression() else {
                    return;
                };

                let kind = self.expr_symbol_kind(&expr);

                match kind {
                    Type::String(id) => {
                        let Some(id) = id else {
                            return;
                        };

                        Some(self.arenas.strings[id].to_string())
                    }
                    _ => None,
                }
            }
            _ => None,
        }) else {
            return;
        };

        let index = self.arenas.symbols.alloc(Symbol {
            name: name.clone(),
            kind,
            range: property.syntax().text_range(),
        });
        self.arenas
            .add_container_member(self.container, name, index);
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
                        kind: Type::Function(idx),
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
                    kind: Type::Function(idx),
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
                    kind: Type::Null,
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
                kind: Type::Function(idx),
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
            None => Type::Unknown,
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
            None => Type::Unknown,
        };

        let (key_kind, value_kind) = match iterable {
            Type::Array(id) => {
                let array = &self.arenas.arrays[id];
                (Type::Integer, array.kind)
            }
            _ => (Type::String(None), Type::Unknown),
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
                kind: Type::Class(id),
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

        let Some(qualified_name) = stmt.name() else {
            self.collect_function(idx, stmt);
            return;
        };

        let mut names: Vec<Name> = qualified_name.names().collect();

        let Some(final_name) = names.pop().and_then(|n| n.text()) else {
            self.collect_function(idx, stmt);
            return;
        };

        if names.is_empty() {
            // Plain `function abc()`: declare in current container
            let symbol = self.arenas.symbols.alloc(Symbol {
                name: final_name.clone(),
                kind: Type::Function(idx),
                range: stmt.syntax().text_range(),
            });
            self.arenas
                .add_container_member(self.container, final_name, symbol);
        } else {
            let mut expr_kind = self.name_expression(&names[0], true);
            let mut kind = self.arenas.expr_to_type(&expr_kind);
            let key = names[0].syntax().text_range();
            self.expr_kinds.insert(key, expr_kind.clone());

            let mut container = match self.to_new_slot_applicable(kind, key) {
                Some(applicable) => applicable.into(),
                _ => {
                    self.collect_function(idx, stmt);
                    return;
                }
            };

            let offset = qualified_name.syntax().text_range().end();
            for segment in &names[1..] {
                let Some(text) = segment.text() else {
                    self.collect_function(idx, stmt);
                    return;
                };

                expr_kind = self
                    .arenas
                    .get_container_members(container)
                    .into_iter()
                    .find_map(|idx| {
                        let symbol = &self.arenas.symbols[idx];
                        if self.execution_range.contains_range(symbol.range)
                            && symbol.range.end() >= offset
                            || text != symbol.name
                        {
                            return None;
                        }
                        Some(idx)
                    })
                    .map_or_else(
                        || ExpressionKind::Parent(kind, text.into_boxed_str()),
                        |symbol| ExpressionKind::Symbol(Some(kind), symbol),
                    );

                kind = self.arenas.expr_to_type(&expr_kind);
                let key = segment.syntax().text_range();
                self.expr_kinds.insert(key, expr_kind.clone());

                container = match self.to_new_slot_applicable(kind, key) {
                    Some(applicable) => applicable.into(),
                    _ => {
                        self.collect_function(idx, stmt);
                        return;
                    }
                };
            }

            let symbol = self.arenas.symbols.alloc(Symbol {
                name: final_name.clone(),
                kind: Type::Function(idx),
                range: stmt.syntax().text_range(),
            });

            self.arenas
                .add_container_member(container, final_name, symbol);
        }

        self.collect_function(idx, stmt);
    }

    fn enum_statement(&mut self, stmt: &EnumStatement) {
        let idx = self.arenas.enums.alloc(EnumData::default());

        if let Some(name) = stmt.name().and_then(|n| n.text()) {
            let symbol = self.arenas.symbols.alloc(Symbol {
                name: name.clone(),
                kind: Type::Enum(idx),
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
            Type::Unknown
        };

        for clause in stmt.clauses() {
            match clause {
                SwitchClause::Case(case) => {
                    if let Some(test) = case.test() {
                        let case_kind = self.expr_symbol_kind(&test);
                        if !matches!(
                            (kind, case_kind),
                            (Type::Null, _)
                                | (_, Type::Null)
                                | (Type::Unknown, _)
                                | (_, Type::Unknown)
                                | (Type::Integer | Type::Float, Type::Integer | Type::Float,)
                                | (Type::String(_), Type::String(_))
                                | (Type::Boolean, Type::Boolean)
                        ) {
                            self.diagnostics.push(Diagnostic {
                                message: format!("Case of type '{case_kind}' is incompitable with discriminant of type '{kind}'"),
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
                    message: "Value returned by the source file execution scope cannot be received in any way".to_owned(),
                    range: stmt.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                });
            }
            return;
        };

        if !matches!(
            self.arenas.functions[function].ret,
            Type::Unknown | Type::Null
        ) {
            return;
        }

        match kind {
            None | Some(Type::Unknown) => {}
            Some(kind) => {
                self.arenas.functions[function].ret = kind;
            }
        }
    }

    fn yield_statement(&mut self, stmt: &YieldStatement) {
        let kind = if let Some(value) = stmt.value() {
            self.expr_symbol_kind(&value)
        } else {
            Type::Null
        };

        let Some(function) = self.function else {
            self.diagnostics.push(Diagnostic {
                message: "Yielding in the source file execution scope".to_owned(),
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
                    kind: Type::String(None),
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
            Type::Unknown
        };

        let Some(function) = self.function else {
            return;
        };

        if self.arenas.functions[function].throwing.is_none() {
            self.arenas.functions[function].throwing = Some(kind);
        }
    }

    fn expr_symbol_kind(&mut self, expr: &Expr) -> Type {
        let kind = self.collect_expr(expr);
        self.arenas.expr_to_type(&kind)
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
            Expr::Name(expr) => self.name_expression(expr, false),
            Expr::This(expr) => self.this_expression(expr),
            Expr::RootAccess(expr) => self.root_access_expression(expr),
            Expr::Base(expr) => self.base_expression(expr),
            Expr::MemberAccess(expr) => self.member_access_expression(expr),
            Expr::ElementAccess(expr) => self.element_access_expression(expr),
            Expr::Call(expr) => self.call_expression(expr),
            Expr::Clone(expr) => self.clone_expression(expr),
            Expr::Binary(expr) => self.binary_expression(expr),
            Expr::Conditional(expr) => self.conditional_expression(expr),
            Expr::PrefixUnary(expr) => todo!(),
            Expr::PrefixUpdate(expr) => todo!(),
            Expr::PostfixUpdate(expr) => todo!(),
            Expr::Delete(expr) => todo!(),
            Expr::TypeOf(expr) => todo!(),
            Expr::Resume(expr) => todo!(),
            Expr::RawCall(expr) => todo!(),
            Expr::File(_) => ExpressionKind::Literal(Type::String(None)),
            Expr::Line(_) => ExpressionKind::Literal(Type::Integer),
            Expr::Parenthesised(expr) => self.parenthesised_expression(expr),
            Expr::Function(expr) => self.function_expression(expr),
            Expr::Lambda(expr) => self.lambda_expression(expr),
        }
    }

    fn literal_expression(&mut self, expr: &LiteralExpression) -> ExpressionKind {
        let token = expr.token().unwrap();
        match token.kind() {
            SyntaxKind::Integer => ExpressionKind::Literal(Type::Integer),
            SyntaxKind::Character => ExpressionKind::Literal(Type::Integer),
            SyntaxKind::Float => ExpressionKind::Literal(Type::Float),
            SyntaxKind::String | SyntaxKind::VerbatimString => {
                let id = self
                    .arenas
                    .strings
                    .alloc(unquote_string(token.text()).into_boxed_str());
                ExpressionKind::Literal(Type::String(Some(id)))
            }
            SyntaxKind::NullKeyword => ExpressionKind::Literal(Type::Null),
            SyntaxKind::TrueKeyword => ExpressionKind::Literal(Type::Boolean),
            SyntaxKind::FalseKeyword => ExpressionKind::Literal(Type::Boolean),
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

        ExpressionKind::Literal(Type::Table(id))
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

        ExpressionKind::Literal(Type::Class(id))
    }

    fn array_literal_expression(&mut self, expr: &ArrayLiteralExpression) -> ExpressionKind {
        let mut kinds = expr
            .elements()
            .map(|element| self.expr_symbol_kind(&element));

        let Some(kind) = kinds.next() else {
            return ExpressionKind::Literal(Type::Array(self.arenas.arrays.alloc(ArrayData {
                kind: Type::Unknown,
            })));
        };

        ExpressionKind::Literal(
            if kinds.all(|k| std::mem::discriminant(&k) == std::mem::discriminant(&kind)) {
                Type::Array(self.arenas.arrays.alloc(ArrayData { kind }))
            } else {
                Type::Array(self.arenas.arrays.alloc(ArrayData {
                    kind: Type::Unknown,
                }))
            },
        )
    }

    fn name_expression(&mut self, expr: &Name, only_members: bool) -> ExpressionKind {
        let Some(text) = expr.text() else {
            return ExpressionKind::Unknown;
        };

        let offset = expr.syntax().text_range().end();

        let find_function = |idx: SymbolId| {
            let symbol = &self.arenas.symbols[idx];
            if self.execution_range.contains_range(symbol.range) && symbol.range.end() >= offset
                || text != symbol.name
            {
                return None;
            }
            Some(idx)
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

        if only_members {
            return members().or_else(root).map_or_else(
                || ExpressionKind::Parent(self.container.into(), text.into_boxed_str()),
                |id| ExpressionKind::Symbol(None, id),
            );
        }

        let find = |idx: SymbolId| {
            let symbol = &self.arenas.symbols[idx];
            if symbol.range.end() >= offset || text != symbol.name {
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

        return locals
            .or_else(consts)
            .or_else(members)
            .or_else(root)
            .map_or_else(
                || ExpressionKind::Parent(self.container.into(), text.into_boxed_str()),
                |id| ExpressionKind::Symbol(None, id),
            );
    }

    fn this_expression(&self, _expr: &ThisExpression) -> ExpressionKind {
        ExpressionKind::Literal(match self.container {
            Container::Class(id) => Type::Class(id),
            Container::Table(id) => Type::Table(id),
            Container::Enum(id) => Type::Enum(id),
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
                || ExpressionKind::Parent(Type::Table(self.root_table), text.into_boxed_str()),
                |symbol| ExpressionKind::Symbol(None, symbol),
            )
    }

    fn base_expression(&mut self, expr: &BaseExpression) -> ExpressionKind {
        match self.container {
            Container::Class(id) => {
                let class = &self.arenas.classes[id];
                if let Some(inherits) = class.inherits {
                    ExpressionKind::Literal(Type::Class(inherits))
                } else {
                    self.diagnostics.push(Diagnostic {
                        message: "Accessing 'base' in a class that doesn't have a superclass"
                            .to_owned(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Warning,
                    });
                    ExpressionKind::Literal(Type::Null)
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: "Accessing 'base' inside non-class execution scope".to_owned(),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                });
                ExpressionKind::Literal(Type::Null)
            }
        }
    }

    fn member_access_expression(&mut self, expr: &MemberAccessExpression) -> ExpressionKind {
        let obj = match expr.object() {
            Some(expr) => self.expr_symbol_kind(&expr),
            None => Type::Unknown,
        };

        let Some(text) = expr
            .member_part()
            .and_then(|m| m.name())
            .and_then(|n| n.text())
        else {
            return ExpressionKind::Unknown;
        };

        let Some(members) = self.arenas.get_kind_members(obj) else {
            return ExpressionKind::Parent(obj, text.into_boxed_str());
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
                || ExpressionKind::Parent(obj, text.into_boxed_str()),
                |symbol| ExpressionKind::Symbol(Some(obj), symbol),
            )
    }

    fn element_access_expression(&mut self, expr: &ElementAccessExpression) -> ExpressionKind {
        let obj = match expr.object() {
            Some(expr) => self.expr_symbol_kind(&expr),
            None => Type::Unknown,
        };

        let Some(index) = expr.index().and_then(|i| i.expression()) else {
            return ExpressionKind::Unknown;
        };

        let index_kind = self.expr_symbol_kind(&index);
        match index_kind {
            Type::String(id) => {
                let Some(id) = id else {
                    return ExpressionKind::Unknown;
                };

                let text = self.arenas.strings[id].as_ref();
                let Some(members) = self.arenas.get_kind_members(obj) else {
                    return ExpressionKind::Unknown;
                };

                let offset = expr.syntax().text_range().end();
                members
                    .into_iter()
                    .find_map(|idx| {
                        let symbol = &self.arenas.symbols[idx];
                        if self.execution_range.contains_range(symbol.range)
                            && symbol.range.end() >= offset
                            || text != symbol.name
                        {
                            return None;
                        }
                        Some(idx)
                    })
                    .map_or_else(
                        || ExpressionKind::Parent(obj, text.into()),
                        |symbol| ExpressionKind::Symbol(Some(obj), symbol),
                    )
            }
            _ => ExpressionKind::Unknown,
        }
    }

    fn call_symbol_kind(&mut self, expr: &CallExpression, kind: Type) -> ExpressionKind {
        match kind {
            Type::Function(id) => {
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
                            match (self.arenas.expr_to_type(&argument_kind), symbol.kind) {
                                (Type::Unknown, required_kind) => {
                                    // If passed in parameter has type of unknown
                                    // we can coerce it to be the type of a required parameter
                                    if let ExpressionKind::Symbol(_, id) = &argument_kind {
                                        if self.arenas.expr_to_type(&argument_kind) == Type::Unknown
                                            && required_kind != Type::Unknown
                                        {
                                            self.arenas.symbols[*id].kind = required_kind;
                                        }
                                    }
                                }

                                (Type::Null, _)
                                | (_, Type::Null)
                                | (_, Type::Unknown)
                                | (Type::Integer | Type::Float, Type::Integer | Type::Float) => {}

                                (passed, required) => {
                                    if discriminant(&passed) != discriminant(&required) {
                                        self.diagnostics.push(Diagnostic {
                                            message: format!("Expected parameter of type '{required}', but got '{passed}'"),
                                            range: expr.syntax().text_range(),
                                            severity: DiagnosticSeverity::Warning,
                                        });
                                    }
                                }
                            }
                        }
                        None if !is_variadic => self.diagnostics.push(Diagnostic {
                            message: "Passing more parameters than possible".to_owned(),
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
                        message: "Insufficient number of parameters passed".to_owned(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }

                return ExpressionKind::Literal(if self.arenas.functions[id].yielding.is_some() {
                    Type::Generator(id)
                } else {
                    self.arenas.clone_type(self.arenas.functions[id].ret)
                });
            }
            Type::Class(id) => {
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
                            message: "Default constructor should have no parameters".to_owned(),
                            range: expr.syntax().text_range(),
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                }
                return ExpressionKind::Literal(Type::Instance(id));
            }
            Type::Table(id) => match self.get_delegate_member(id, "_call") {
                Ok(id) => {
                    let kind = self.arenas.symbols[id].kind;
                    return self.call_symbol_kind(expr, kind);
                }
                Err(TableDelegateError::NoDelegate) => {
                    self.diagnostics.push(Diagnostic {
                        message: "'table' does not support calling: no delegate assigned"
                            .to_owned(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }
                Err(TableDelegateError::NoMember) => {
                    self.diagnostics.push(Diagnostic {
                        message:
                            "'table' does not support calling: delegate has no '_call' metamethod"
                                .to_owned(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }
            },
            Type::Instance(id) => {
                let class = &self.arenas.classes[id];
                if let Some(member) = class.get_member("_call") {
                    return self.call_symbol_kind(expr, self.arenas.symbols[member].kind);
                } else {
                    self.diagnostics.push(Diagnostic {
                        message:
                            "'instance' does not support calling: class does not define a '_call' metamethod"
                                .to_owned(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }
            }
            Type::Unknown => {}
            kind => {
                self.diagnostics.push(Diagnostic {
                    message: format!("'{kind}' is not callable"),
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
            None => Type::Unknown,
        };

        self.call_symbol_kind(expr, obj)
    }

    fn clone_expression(&mut self, expr: &CloneExpression) -> ExpressionKind {
        let Some(operand) = expr.operand() else {
            return ExpressionKind::Unknown;
        };

        let kind = self.expr_symbol_kind(&operand);
        ExpressionKind::Literal(self.arenas.clone_type(kind))
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
            SyntaxKind::InKeyword => self.in_operator(left_kind, right_kind, operator),
            SyntaxKind::InstanceOfKeyword => {
                self.instance_of_operator(left_kind, right_kind, operator)
            }
            SyntaxKind::EqualsEquals | SyntaxKind::ExclamationEquals => {
                ExpressionKind::Literal(Type::Boolean)
            }
            SyntaxKind::LessThan
            | SyntaxKind::LessThanEquals
            | SyntaxKind::GreaterThan
            | SyntaxKind::GreaterThanEquals
            | SyntaxKind::LessThanEqualsGreaterThan => {
                self.comparison_operator(left_kind, right_kind, operator)
            }
            SyntaxKind::Ampersand
            | SyntaxKind::Bar
            | SyntaxKind::Caret
            | SyntaxKind::LessThanLessThan
            | SyntaxKind::GreaterThanGreaterThan
            | SyntaxKind::GreaterThanGreaterThanGreaterThan => {
                self.bitwise_operator(left_kind, right_kind, operator)
            }
            SyntaxKind::AmpersandAmpersand | SyntaxKind::BarBar => {
                self.logical_operator(left_kind, right_kind, operator)
            }
            SyntaxKind::PlusEquals | SyntaxKind::Plus => todo!(),
            SyntaxKind::MinusEquals | SyntaxKind::Minus => todo!(),
            SyntaxKind::AsteriskEquals | SyntaxKind::Asterisk => todo!(),
            SyntaxKind::SlashEquals | SyntaxKind::Slash => todo!(),
            SyntaxKind::PercentEquals | SyntaxKind::Percent => todo!(),
            _ => unreachable!(),
        }
    }

    fn to_new_slot_applicable(
        &mut self,
        kind: Type,
        error_range: TextRange,
    ) -> Option<NewSlotApplicable> {
        Some(match kind {
            Type::Table(id) => NewSlotApplicable::Table(id),
            Type::Class(id) => NewSlotApplicable::Class(id),
            Type::Instance(id) => {
                let class = &self.arenas.classes[id];
                if class.get_member("_newslot").is_none() {
                    self.diagnostics.push(Diagnostic {
                        message: "'instance' does not support a new slot operator: class has no '_newslot' metamethod".to_owned(),
                        range: error_range,
                        severity: DiagnosticSeverity::Error,
                    });
                    return None;
                }
                NewSlotApplicable::Instance(id)
            }
            Type::Unknown => return None,
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: format!("'{kind}' does not support a new slot operator"),
                    range: error_range,
                    severity: DiagnosticSeverity::Error,
                });
                return None;
            }
        })
    }

    fn equals_operator(
        &mut self,
        left_kind: ExpressionKind,
        right_kind: ExpressionKind,
        operator: SyntaxToken,
        range: TextRange,
    ) -> ExpressionKind {
        match left_kind {
            ExpressionKind::Parent(parent, member)
                if operator.kind() == SyntaxKind::LessThanMinus =>
            {
                match self.to_new_slot_applicable(parent, operator.text_range()) {
                    Some(applicable) => {
                        let symbol = self.arenas.symbols.alloc(Symbol {
                            kind: self.arenas.expr_to_type(&right_kind),
                            name: member.to_string(),
                            range,
                        });

                        self.arenas.add_container_member(
                            applicable.into(),
                            member.into_string(),
                            symbol,
                        );
                    }
                    _ => {}
                }
            }
            ExpressionKind::Parent(parent, _) => match parent {
                Type::Array(_) | Type::Class(_) | Type::Table(_) | Type::Instance(_) => {}
                kind => {
                    self.diagnostics.push(Diagnostic {
                        message: format!("'{kind}' does not support an equals operator"),
                        range: operator.text_range(),
                        severity: DiagnosticSeverity::Error,
                    });
                }
            },
            ExpressionKind::Symbol(obj, symbol) if operator.kind() == SyntaxKind::LessThanMinus => {
                if let Some(obj) = obj {
                    if self
                        .to_new_slot_applicable(obj, operator.text_range())
                        .is_none()
                    {
                        return right_kind;
                    }
                }

                let kind = self.arenas.expr_to_type(&right_kind);

                let symbol = &mut self.arenas.symbols[symbol];

                // Update symbol kind if it's null or unknown
                if matches!(symbol.kind, Type::Unknown | Type::Null)
                    && !matches!(kind, Type::Unknown | Type::Null)
                {
                    symbol.kind = kind;
                }
            }

            ExpressionKind::Symbol(obj, symbol) => {
                match obj {
                    Some(Type::Array(_)) => {}
                    None
                    | Some(Type::Class(_))
                    | Some(Type::Table(_))
                    | Some(Type::Instance(_)) => {
                        let kind = self.arenas.expr_to_type(&right_kind);

                        let symbol = &mut self.arenas.symbols[symbol];

                        // Update symbol kind if it's null or unknown
                        if matches!(symbol.kind, Type::Unknown | Type::Null)
                            && !matches!(kind, Type::Unknown | Type::Null)
                        {
                            symbol.kind = kind;
                        }
                    }

                    Some(kind) => {
                        self.diagnostics.push(Diagnostic {
                            message: format!("'{kind}' does not support an equals operator"),
                            range: operator.text_range(),
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                }
            }

            _ => {}
        }
        right_kind
    }

    fn in_operator(
        &mut self,
        left_kind: ExpressionKind,
        right_kind: ExpressionKind,
        operator: SyntaxToken,
    ) -> ExpressionKind {
        let right = self.arenas.expr_to_type(&right_kind);
        match right {
            Type::Array(_) => {
                let left = self.arenas.expr_to_type(&left_kind);
                if !matches!(left, Type::Unknown | Type::Integer) {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Trying to index into an array using '{left}' (only integers are applicable)"),
                        range: operator.text_range(),
                        severity: DiagnosticSeverity::Warning
                    });
                }
            }
            Type::Table(_) | Type::Class(_) | Type::Instance(_) | Type::Unknown => {}
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: format!("Indexing into '{right}' will always return false"),
                    range: operator.text_range(),
                    severity: DiagnosticSeverity::Warning,
                });
            }
        }
        ExpressionKind::Literal(Type::Boolean)
    }

    fn instance_of_operator(
        &mut self,
        left_kind: ExpressionKind,
        right_kind: ExpressionKind,
        operator: SyntaxToken,
    ) -> ExpressionKind {
        let left = self.arenas.expr_to_type(&left_kind);
        let right = self.arenas.expr_to_type(&right_kind);

        match (left, right) {
            (Type::Unknown, Type::Class(_))
            | (Type::Instance(_), Type::Unknown)
            | (Type::Unknown, Type::Unknown)
            | (Type::Instance(_), Type::Class(_)) => {}
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: format!(
                        "'instanceof' operator between '{left}' and '{right}' is not supported"
                    ),
                    range: operator.text_range(),
                    severity: DiagnosticSeverity::Error,
                });
            }
        }

        ExpressionKind::Literal(Type::Boolean)
    }

    fn comparison_operator(
        &mut self,
        left_kind: ExpressionKind,
        right_kind: ExpressionKind,
        operator: SyntaxToken,
    ) -> ExpressionKind {
        let left = self.arenas.expr_to_type(&left_kind);
        let right = self.arenas.expr_to_type(&right_kind);

        match (left, right) {
            (Type::Null, _)
            | (_, Type::Null)
            | (Type::Unknown, _)
            | (_, Type::Unknown)
            | (Type::Integer | Type::Float, Type::Integer | Type::Float)
            | (Type::String(_), Type::String(_))
            | (Type::Boolean, Type::Boolean) => {}
            // (SymbolKind::Table(id), _) => match self.get_delegate_member(id, "_cmp") {
            //     Ok(id) => {
            //         self.call_symbol_kind(expr, self.arenas.symbols[id].kind)
            //         //
            //     }
            //     Err(TableDelegateError::NoDelegate) => {
            //         self.diagnostics.push(Diagnostic {
            //             message: "Table is uncomparable: no delegate assigned".to_owned(),
            //             range: operator.text_range(),
            //             severity: DiagnosticSeverity::Error,
            //         });
            //     }
            //     Err(TableDelegateError::NoMember) => {
            //         self.diagnostics.push(Diagnostic {
            //             message: "Table is uncomparable: delegate has no '_cmp' metamethod".to_owned(),
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
            //                     .to_owned(),
            //             range: operator.text_range(),
            //             severity: DiagnosticSeverity::Error,
            //         });
            //     }
            //     Err(ClassMethodError::NotAMethod) => {
            //         self.diagnostics.push(Diagnostic {
            //             message: "Instance is uncomparable: class's '_call' member is not a method"
            //                 .to_owned(),
            //             range: operator.text_range(),
            //             severity: DiagnosticSeverity::Error,
            //         });
            //     }
            // },
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: format!("Cannot compare '{left}' to '{right}'"),
                    range: operator.text_range(),
                    severity: DiagnosticSeverity::Error,
                });
            }
        }

        ExpressionKind::Literal(
            if operator.kind() == SyntaxKind::GreaterThanGreaterThanGreaterThan {
                Type::Integer
            } else {
                Type::Boolean
            },
        )
    }

    fn bitwise_operator(
        &mut self,
        left_kind: ExpressionKind,
        right_kind: ExpressionKind,
        operator: SyntaxToken,
    ) -> ExpressionKind {
        let left = self.arenas.expr_to_type(&left_kind);
        let right = self.arenas.expr_to_type(&right_kind);

        match (left, right) {
            (Type::Integer, Type::Integer) => {
                return ExpressionKind::Literal(Type::Integer);
            }
            (_, Type::Unknown) => {
                if left == Type::Integer {
                    match right_kind {
                        ExpressionKind::Symbol(_, symbol) => {
                            self.arenas.symbols[symbol].kind = Type::Integer;
                        }
                        _ => {}
                    }
                    return ExpressionKind::Literal(Type::Integer);
                }
                if left == Type::Unknown {
                    return ExpressionKind::Literal(Type::Integer);
                }
            }
            (Type::Unknown, _) => {
                if right == Type::Integer {
                    match left_kind {
                        ExpressionKind::Symbol(_, symbol) => {
                            self.arenas.symbols[symbol].kind = Type::Integer;
                        }
                        _ => {}
                    }
                    return ExpressionKind::Literal(Type::Integer);
                }
                if right == Type::Unknown {
                    return ExpressionKind::Literal(Type::Integer);
                }
            }
            _ => {}
        }

        self.diagnostics.push(Diagnostic {
            message: format!("Bitwise operator between '{left}' and '{right}' is not supported"),
            range: operator.text_range(),
            severity: DiagnosticSeverity::Error,
        });

        ExpressionKind::Literal(Type::Integer)
    }

    fn logical_operator(
        &mut self,
        left_kind: ExpressionKind,
        right_kind: ExpressionKind,
        _operator: SyntaxToken,
    ) -> ExpressionKind {
        let left = self.arenas.expr_to_type(&left_kind);
        let right = self.arenas.expr_to_type(&right_kind);

        ExpressionKind::Literal(if left == Type::Unknown { right } else { left })
    }

    fn conditional_expression(&mut self, expr: &ConditionalExpression) -> ExpressionKind {
        if let Some(expr) = expr.condition() {
            self.collect_expr(&expr);
        };

        let then_kind = if let Some(expr) = expr.then_branch().and_then(|b| b.expression()) {
            self.expr_symbol_kind(&expr)
        } else {
            Type::Unknown
        };

        let else_kind = if let Some(expr) = expr.else_branch().and_then(|b| b.expression()) {
            self.expr_symbol_kind(&expr)
        } else {
            Type::Unknown
        };

        ExpressionKind::Literal(if then_kind != Type::Unknown {
            then_kind
        } else {
            else_kind
        })
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
        ExpressionKind::Literal(Type::Function(idx))
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
            Type::Unknown
        };

        self.arenas.functions[idx].ret = ret;

        self.exit_scope();
        self.function = save_function;
        self.execution_range = save_execution;

        ExpressionKind::Literal(Type::Function(idx))
    }
}
