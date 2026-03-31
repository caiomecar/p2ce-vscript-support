use la_arena::Idx;
use rustc_hash::{FxHashMap, FxHashSet};
use sq_3_parser::{AstNode, TextRange, TextSize, ast::*};
use std::{collections::VecDeque, mem::discriminant};

use crate::{
    Diagnostic, DiagnosticSeverity, File, GetMembers, SourceSymbol,
    arena::{
        ArenaAlloc, ArenaId, ArrayData, ArrayId, ClassData, ClassId, Container, EnumData, EnumId,
        FunctionData, FunctionId, ParamsState, SourceArena, StringId, SymbolId, TableData, TableId,
    },
    db::Db,
    source_symbol,
    symbol::{Symbol, SymbolKind, SymbolTable, Type},
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
            locals: FxHashMap::default(),
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

// This is needed to accommodate for the fact that symbols inside function
// body can be defined after the function body itself, execution_range
// only solves this for when we call symbols_at, at which point the whole
// file is already process, however during the execution of the function
// body only the symbols defined before it are visible to mirror this
// behaviour in the actual analysis we save all state of the collector in
// this struct and put this struct in the queue, once the direct source
// file statements have been collected we run a loop over this queue
// where we copy the state onto collector and run collect_stmt on the
// function body.
#[derive(Debug)]
struct DeferredFunction {
    idx: Idx<FunctionData>,
    body: FunctionBody,
    execution_range: TextRange,
    scope_stack: Vec<Scope>,
}

#[derive(Debug, Clone)]
enum AssignmentLeftHandSide {
    CanCreate {
        parent: Type,
        range: TextRange,
        new_key: Box<str>,
    },
    // Parent doesn't exist for locals
    Exists {
        parent: Option<Type>,
        // For saving the new type in case it is updated
        range: TextRange,
        symbol: SymbolId,
    },
    NonStringKey {
        parent: Type,
        range: TextRange,
        key: NullableExprKind,
    },
    Invalid(NullableExprKind),
}

impl From<AssignmentLeftHandSide> for NullableExprKind {
    fn from(value: AssignmentLeftHandSide) -> Self {
        match value {
            AssignmentLeftHandSide::CanCreate { .. } => None,
            AssignmentLeftHandSide::Exists { symbol, .. } => Some(ExpressionKind::Symbol(symbol)),
            AssignmentLeftHandSide::NonStringKey { .. } => None,
            AssignmentLeftHandSide::Invalid(kind) => kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    // L value
    Literal(Type),
    // R value
    Symbol(SymbolId),
}

enum FindSymbol {
    Any,
    OnlyBefore(TextSize),
    BeforeIfInExecutionRange(TextSize),
}

pub type NullableExprKind = Option<ExpressionKind>;

enum MetamethodErrors {
    No,
    Yes { keyword: &'static str },
    YesBinary { keyword: &'static str, right: Type },
}

pub type RangeExprKindMap = FxHashMap<TextRange, ExpressionKind>;

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

pub struct Collector<'db> {
    db: &'db dyn Db,
    file: File,
    imports: FxHashMap<Container, Vec<File>>,

    arena: SourceArena,
    const_table: Idx<TableData>,
    root_table: Idx<TableData>,

    scope_stack: Vec<Scope>,
    container: Container,
    execution_range: TextRange,
    function: Option<Idx<FunctionData>>,
    deferred_functions: VecDeque<DeferredFunction>,

    expr_kinds: RangeExprKindMap,
    diagnostics: Vec<Diagnostic>,
}

impl<'db> Collector<'db> {
    pub fn symbol_from_source_file(db: &'db dyn Db, file: File, node: SourceFile) -> SourceSymbol {
        let mut arenas = SourceArena::default();
        // Source table is not always the root table, it depends on which entity
        // was the script executed. script_execute and non-edict entities execute stuff
        // in the root while edict entities with 'vscripts' keyvalue will have their
        // script scope as the execution context
        // This should also drive whether 'self' is present in the scope
        // TODO: Get source file's jsdoc and determine
        let source_table = arenas.alloc(TableData::default());
        let container = Container::Table(source_table);
        let root_table = arenas.alloc(TableData::default());
        let const_table = arenas.alloc(TableData::default());
        let mut imports = FxHashMap::default();
        imports.insert(container, db.stdlibs().to_vec());

        let mut collector = Self {
            db,
            file,
            imports,
            container,
            arena: arenas,
            const_table,
            root_table,
            scope_stack: Vec::new(),
            execution_range: node.syntax().text_range(),
            function: None,
            deferred_functions: VecDeque::new(),
            expr_kinds: FxHashMap::default(),
            diagnostics: Vec::new(),
        };

        collector.enter_scope(collector.execution_range);

        for stmt in node.statements() {
            collector.collect_stmt(&stmt);
        }

        assert_eq!(collector.scope_stack.len(), 1);
        let source_scope = collector.exit_scope().unwrap();

        while let Some(func) = collector.deferred_functions.pop_front() {
            collector.execution_range = func.execution_range;
            collector.function = Some(func.idx);
            collector.container = func.scope_stack.last().unwrap().container;
            collector.scope_stack = func.scope_stack;
            match func.body {
                FunctionBody::Expr(expr) => {
                    let typ = collector.expr_type(&expr);
                    collector.arena[func.idx].ret = typ;
                }
                FunctionBody::Stmt(stmt) => {
                    collector.collect_stmt(&stmt);
                }
            }
        }

        SourceSymbol {
            imports: collector.imports,
            file,
            arena: collector.arena,
            const_table,
            root_table,
            source_table,
            source_scope,
            expr_kinds: collector.expr_kinds,
            diagnostics: collector.diagnostics,
        }
    }

    fn get<T>(&self, id: T) -> &T::Data
    where
        T: ArenaId,
        SourceArena: std::ops::Index<Idx<T::Data>, Output = T::Data>,
    {
        if id.file() != self.file {
            return id.get_data(self.db);
        }

        &self.arena[id.idx()]
    }

    fn get_mut<T>(&mut self, id: T) -> Option<&mut T::Data>
    where
        T: ArenaId,
        SourceArena: std::ops::IndexMut<Idx<T::Data>, Output = T::Data>,
    {
        if id.file() != self.file {
            return None;
        }

        Some(&mut self.arena[id.idx()])
    }

    fn symbol(&mut self, symbol: Symbol) -> SymbolId {
        SymbolId::new(self.file, self.arena.alloc(symbol))
    }

    fn function(&mut self) -> FunctionId {
        FunctionId::new(self.file, self.arena.alloc(FunctionData::default()))
    }

    fn class(&mut self, class: &impl IsClass) -> ClassId {
        let expr = class.extends().and_then(|e| e.expression());

        let inherits = if let Some(expr) = expr {
            match self.expr_type(&expr) {
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
            Some(id) => self.clone_members(id),
            None => SymbolTable::default(),
        };

        ClassId::new(self.file, self.arena.alloc(ClassData { inherits, members }))
    }

    fn array(&mut self, array: ArrayData) -> ArrayId {
        ArrayId::new(self.file, self.arena.alloc(array))
    }

    fn clone_members(&self, superclass: ClassId) -> SymbolTable {
        let symbol = self.get(superclass);
        let members = symbol.members.clone();

        members
    }

    fn clone_type(&mut self, typ: Type) -> Type {
        match typ {
            Type::Table(id) => {
                let new = TableId::new(self.file, self.arena.alloc(self.get(id).clone()));
                Type::Table(new)
            }
            Type::Class(id) => {
                let new = ClassId::new(self.file, self.arena.alloc(self.get(id).clone()));
                Type::Class(new)
            }
            _ => typ,
        }
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

    fn get_local_members(&self) -> Vec<SymbolId> {
        self.scope_stack
            .iter()
            .rev()
            .flat_map(|scope| scope.locals.values().copied())
            .collect()
    }

    fn get_const_members(&self) -> Vec<SymbolId> {
        self.arena[self.const_table].get_members()
    }

    fn get_root_members(&self) -> Vec<SymbolId> {
        self.arena[self.root_table].get_members()
    }

    fn get_current_container_members(&self) -> Vec<SymbolId> {
        self.get_container_members(self.container)
    }

    fn get_container_members(&self, container: Container) -> Vec<SymbolId> {
        match container {
            Container::Table(idx) => self.arena[idx].get_members(),
            Container::Class(idx) => self.arena[idx].get_members(),
            Container::Enum(idx) => self.arena[idx].get_members(),
        }
    }

    fn get_type_members(&self, typ: Type) -> Vec<SymbolId> {
        match typ {
            Type::Table(id) => self.get(id).get_members(),
            Type::Class(id) => self.get(id).get_members(),
            Type::Enum(id) => self.get(id).get_members(),
            Type::Instance(id) => self.get(id).get_members(),
            _ => Vec::new(),
        }
    }

    fn add_current_container_member(&mut self, name: String, symbol: SymbolId) {
        self.add_container_member(self.container, name, symbol);
    }

    fn add_container_member(&mut self, container: Container, name: String, symbol: SymbolId) {
        match container {
            Container::Table(idx) => self.arena[idx].add_member(name, symbol),
            Container::Class(idx) => self.arena[idx].add_member(name, symbol),
            Container::Enum(idx) => self.arena[idx].add_member(name, symbol),
        }
    }

    fn expr_to_type(&self, expr: NullableExprKind) -> Type {
        match expr {
            Some(ExpressionKind::Literal(kind)) => kind,
            Some(ExpressionKind::Symbol(symbol)) => self.get(symbol).typ,
            None => Type::Unknown,
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

                        let symbol = self.symbol(Symbol {
                            name: name.clone(),
                            typ: Type::Null,
                            kind: SymbolKind::Local,
                            range: var.syntax().text_range(),
                        });

                        self.current_scope().locals.insert(name, symbol);
                        self.arena[function].params.push(symbol);
                        continue;
                    };

                    let typ = self.expr_type(&expr);

                    let symbol = self.symbol(Symbol {
                        name: name.clone(),
                        typ,
                        kind: SymbolKind::Local,
                        range: var.syntax().text_range(),
                    });

                    self.current_scope().locals.insert(name, symbol);
                    self.arena[function].params.push(symbol);
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

        self.arena[function].params_state = params_state;
    }

    fn call_metamethod(
        &mut self,
        typ: Type,
        metamethod: &str,
        arguments: &[NullableExprKind],
        error_range: TextRange,
        errors: MetamethodErrors,
    ) -> Option<Type> {
        match typ {
            Type::Table(id) => {
                let table = self.get(id);
                let Some(delegate_idx) = table.delegate else {
                    match errors {
                        MetamethodErrors::Yes { keyword }
                        | MetamethodErrors::YesBinary { keyword, .. } => {
                            self.diagnostics.push(Diagnostic {
                                message: format!(
                                    "'table' does not support {keyword}: no delegate assigned"
                                ),
                                range: error_range,
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        MetamethodErrors::No => {}
                    }
                    return None;
                };

                let members = &self.get(delegate_idx).members;
                let Some(&member) = members.get(metamethod) else {
                    match errors {
                        MetamethodErrors::Yes { keyword }
                        | MetamethodErrors::YesBinary { keyword, .. } => {
                            self.diagnostics.push(Diagnostic {
                                message: format!("'table' does not support {keyword}: delegate has no '{metamethod}' metamethod"),
                                range: error_range,
                                severity: DiagnosticSeverity::Error
                            });
                        }
                        MetamethodErrors::No => {}
                    }
                    return None;
                };

                Some(self.call_type(self.get(member).typ, arguments, error_range)?)
            }
            Type::Instance(id) => {
                let class = self.get(id);
                let Some(member) = class.get_member(metamethod) else {
                    match errors {
                        MetamethodErrors::Yes { keyword }
                        | MetamethodErrors::YesBinary { keyword, .. } => {
                            self.diagnostics.push(Diagnostic {
                                message: format!("'instance' does not support {keyword}: class has no '{metamethod}' metamethod"),
                                range: error_range,
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        MetamethodErrors::No => {}
                    }
                    return None;
                };

                Some(self.call_type(self.get(member).typ, arguments, error_range)?)
            }
            Type::Unknown => None,
            _ => {
                match errors {
                    MetamethodErrors::Yes { keyword } => {
                        self.diagnostics.push(Diagnostic {
                            message: format!("'{typ}' does not support {keyword}"),
                            range: error_range,
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                    MetamethodErrors::YesBinary {
                        keyword,
                        right: left,
                    } => {
                        self.diagnostics.push(Diagnostic {
                            message: format!("'{typ}' does not support {keyword} with '{left}'"),
                            range: error_range,
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                    MetamethodErrors::No => {}
                }
                None
            }
        }
    }

    fn call_new_slot(
        &mut self,
        typ: Type,
        arguments: &[NullableExprKind],
        error_range: TextRange,
    ) -> Option<Container> {
        match typ {
            Type::Class(id) => Some(Container::Class(id.idx())),
            Type::Table(id) => {
                self.call_metamethod(
                    typ,
                    "_newslot",
                    arguments,
                    error_range,
                    MetamethodErrors::No,
                );
                Some(Container::Table(id.idx()))
            }
            _ => {
                self.call_metamethod(
                    typ,
                    "_newslot",
                    arguments,
                    error_range,
                    MetamethodErrors::Yes {
                        keyword: "new slot operator",
                    },
                )?;
                Some(Container::try_from(typ).unwrap())
            }
        }
    }

    fn call_delete(&mut self, typ: Type, index: NullableExprKind, error_range: TextRange) -> bool {
        match typ {
            Type::Class(_) => true,
            Type::Table(_) => {
                self.call_metamethod(
                    typ,
                    "_delslot",
                    &vec![index],
                    error_range,
                    MetamethodErrors::No,
                );
                true
            }
            _ => self
                .call_metamethod(
                    typ,
                    "_delslot",
                    &vec![index],
                    error_range,
                    MetamethodErrors::Yes {
                        keyword: "delete operator",
                    },
                )
                .is_some(),
        }
    }

    fn call_set(
        &mut self,
        typ: Type,
        arguments: &[NullableExprKind],
        error_range: TextRange,
    ) -> bool {
        match typ {
            Type::Table(_) => {
                self.call_metamethod(typ, "_set", arguments, error_range, MetamethodErrors::No);
                true
            }
            Type::Array(_) | Type::Class(_) => true,
            Type::Instance(_) => {
                self.call_metamethod(typ, "_set", arguments, error_range, MetamethodErrors::No);
                true
            }
            _ => self
                .call_metamethod(
                    typ,
                    "_set",
                    arguments,
                    error_range,
                    MetamethodErrors::Yes {
                        keyword: "equals operator",
                    },
                )
                .is_some(),
        }
    }

    fn call_arithmetic(
        &mut self,
        left: NullableExprKind,
        right: NullableExprKind,
        operator: BinaryOperator,
        error_range: TextRange,
    ) -> Option<Type> {
        let left_type = self.expr_to_type(left);
        let right_type = self.expr_to_type(right);
        let (metamethod, keyword) = match operator {
            BinaryOperator::Add | BinaryOperator::AddAssign => ("_add", "adding"),
            BinaryOperator::Subtract | BinaryOperator::SubtractAssign => ("_sub", "subtracting"),
            BinaryOperator::Multiply | BinaryOperator::MultiplyAssign => ("_mul", "multiplying"),
            BinaryOperator::Divide | BinaryOperator::DivideAssign => ("_div", "dividing"),
            BinaryOperator::Modulo | BinaryOperator::ModuloAssign => ("_modulo", "modulo"),
            _ => unreachable!(),
        };

        match (left_type, right_type) {
            // Add is special: strings
            _ if operator == BinaryOperator::Add || operator == BinaryOperator::AddAssign => {
                match (left_type, right_type) {
                    (_, Type::String(_)) | (Type::String(_), _) => return Some(Type::String(None)),
                    (Type::Integer, Type::Integer) => return Some(Type::Integer),
                    (Type::Float | Type::Integer, Type::Float | Type::Integer) => {
                        return Some(Type::Float);
                    }
                    _ => {}
                }
            }
            (Type::Integer, Type::Integer) => return Some(Type::Integer),
            (Type::Float | Type::Integer, Type::Float | Type::Integer) => return Some(Type::Float),
            _ => {}
        }

        let arguments = vec![right];
        self.call_metamethod(
            left_type,
            metamethod,
            &arguments,
            error_range,
            MetamethodErrors::YesBinary {
                keyword,
                right: right_type,
            },
        )
    }

    fn collect_property(&mut self, property: &Property, default_value: Option<usize>) {
        let kind = match property.value() {
            Some(expr) => self.expr_type(&expr),
            None if default_value.is_some() => Type::Integer,
            None => Type::Unknown,
        };

        let Some(name) = (match property.name() {
            Some(MemberName::Identifier(name)) => name.name().and_then(|n| n.text()),
            Some(MemberName::String(name)) => name
                .token()
                .map(|(_kind, token)| unquote_string(token.text())),
            Some(MemberName::Computed(name)) => {
                let Some(expr) = name.expression() else {
                    return;
                };

                let kind = self.expr_type(&expr);

                match kind {
                    Type::String(id) => {
                        let Some(id) = id else {
                            return;
                        };

                        Some(self.get(id).to_string())
                    }
                    _ => None,
                }
            }
            _ => None,
        }) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: name.clone(),
            typ: kind,
            kind: if default_value.is_some() {
                SymbolKind::EnumMember
            } else {
                SymbolKind::Property
            },
            range: property.syntax().text_range(),
        });

        self.add_current_container_member(name, symbol);
    }

    // Lambdas are processed differently
    fn collect_function<T>(&mut self, idx: Idx<FunctionData>, node: &T)
    where
        T: IsFunction,
    {
        let save_function = self.function;
        self.function = Some(idx);
        let save_execution = self.execution_range;
        self.execution_range = match node.body() {
            Some(FunctionBody::Expr(body)) => body.syntax().text_range(),
            Some(FunctionBody::Stmt(body)) => body.syntax().text_range(),
            None => TextRange::empty(node.syntax().text_range().end()),
        };
        self.enter_scope(self.execution_range);

        if let Some(param_list) = node.parameter_list() {
            self.collect_params(param_list.parameters());
        }

        if let Some(body) = node.body() {
            self.deferred_functions.push_back(DeferredFunction {
                idx,
                body,
                execution_range: self.execution_range,
                scope_stack: self.scope_stack.clone(),
            });
        }

        self.exit_scope();
        self.function = save_function;
        self.execution_range = save_execution;
    }

    fn collect_member(&mut self, member: &Member) {
        match member {
            Member::Property(property) => self.collect_property(property, None),
            Member::Method(method) => {
                let function = self.function();
                if let Some(name) = method.name().and_then(|n| n.text()) {
                    let symbol = self.symbol(Symbol {
                        name: name.clone(),
                        typ: Type::Function(function),
                        kind: SymbolKind::Property,
                        range: method.syntax().text_range(),
                    });
                    self.add_current_container_member(name, symbol);
                }
                self.collect_function(function.idx(), method);
            }
            Member::Constructor(constructor) => {
                let function = self.function();

                let symbol = self.symbol(Symbol {
                    name: "constructor".to_owned(),
                    typ: Type::Function(function),
                    kind: SymbolKind::Property,
                    range: constructor.syntax().text_range(),
                });

                self.add_current_container_member("constructor".to_owned(), symbol);

                self.collect_function(function.idx(), constructor);
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
                let id = self.symbol(Symbol {
                    name: name.clone(),
                    typ: Type::Null,
                    kind: SymbolKind::Local,
                    range: var.syntax().text_range(),
                });

                self.current_scope().locals.insert(name, id);
                continue;
            };

            let typ = self.expr_type(&expr);
            let id = self.symbol(Symbol {
                name: name.clone(),
                typ,
                kind: SymbolKind::Local,
                range: var.syntax().text_range(),
            });

            self.current_scope().locals.insert(name, id);
        }
    }

    fn local_function(&mut self, decl: &LocalFunctionDeclaration) {
        let id = self.function();
        if let Some(name) = decl.name().and_then(|n| n.text()) {
            let symbol = self.symbol(Symbol {
                name: name.clone(),
                typ: Type::Function(id),
                kind: SymbolKind::Local,
                range: decl.syntax().text_range(),
            });

            self.current_scope().locals.insert(name, symbol);
        }
        self.collect_function(id.idx(), decl);
    }
    fn block_statement(&mut self, stmt: &BlockStatement) {
        self.enter_scope(stmt.syntax().text_range());
        for stmt in stmt.statements() {
            self.collect_stmt(&stmt);
        }
        self.exit_scope();
    }

    fn const_statement(&mut self, stmt: &ConstStatement) {
        let typ = match stmt.value() {
            Some(expr) => self.expr_type(&expr),
            None => Type::Unknown,
        };

        let Some(name) = stmt.name().and_then(|n| n.text()) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: name.clone(),
            typ,
            kind: SymbolKind::Constant,
            range: stmt.syntax().text_range(),
        });
        self.arena[self.const_table].add_member(name, symbol);
    }

    fn for_each_statement(&mut self, stmt: &ForEachStatement) {
        if let Some(body) = stmt.body() {
            self.enter_scope(body.syntax().text_range());
        } else {
            self.enter_scope(TextRange::empty(stmt.syntax().text_range().end()));
        }

        let iterable = match stmt.iterable() {
            Some(i) => self.expr_type(&i),
            None => Type::Unknown,
        };

        let (key_type, value_type) = match iterable {
            Type::Array(id) => {
                let array = self.get(id);
                (Type::Integer, array.typ)
            }
            _ => (Type::String(None), Type::Unknown),
        };

        if let Some(key) = stmt.key() {
            if let Some(name) = key.name().and_then(|n| n.text()) {
                let symbol = self.symbol(Symbol {
                    name: name.clone(),
                    typ: key_type,
                    kind: SymbolKind::Local,
                    range: key.syntax().text_range(),
                });

                self.current_scope().locals.insert(name, symbol);
            }
        }

        if let Some(value) = stmt.value() {
            if let Some(name) = value.name().and_then(|n| n.text()) {
                let symbol = self.symbol(Symbol {
                    name: name.clone(),
                    typ: value_type,
                    kind: SymbolKind::Local,
                    range: value.syntax().text_range(),
                });

                self.current_scope().locals.insert(name, symbol);
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
        let class = self.class(stmt);

        let name = stmt.name().and_then(|n| self.assignment_lhs(&n));
        self.do_new_slot(
            name,
            Some(ExpressionKind::Literal(Type::Class(class))),
            stmt.syntax().text_range(),
        );

        let save_symbol = self.container;
        self.container = Container::Class(class.idx());
        for member in stmt.members() {
            self.collect_member(&member);
        }
        self.container = save_symbol;
    }

    fn function_statement(&mut self, stmt: &FunctionStatement) {
        let function = self.function();

        let Some(qualified_name) = stmt.name() else {
            self.collect_function(function.idx(), stmt);
            return;
        };

        let mut names: Vec<Name> = qualified_name.names().collect();

        let Some(final_name) = names.pop().and_then(|n| n.text()) else {
            self.collect_function(function.idx(), stmt);
            return;
        };

        if names.is_empty() {
            // Plain `function abc()`: declare in current container
            let symbol = self.symbol(Symbol {
                name: final_name.clone(),
                typ: Type::Function(function),
                kind: SymbolKind::Property,
                range: stmt.syntax().text_range(),
            });
            self.add_current_container_member(final_name, symbol);

            self.collect_function(function.idx(), stmt);
            return;
        }
        let Some(text) = names[0].text() else {
            self.collect_function(function.idx(), stmt);
            return;
        };

        let offset = qualified_name.syntax().text_range().end();

        let members = self.find_symbol(
            self.get_current_container_members(),
            FindSymbol::BeforeIfInExecutionRange(offset),
            &text,
        );

        let root = || {
            self.find_symbol(
                self.get_root_members(),
                FindSymbol::BeforeIfInExecutionRange(offset),
                &text,
            )
        };

        let Some(expr_kind) = members.or_else(root).map(|id| ExpressionKind::Symbol(id)) else {
            self.collect_function(function.idx(), stmt);
            return;
        };

        let mut typ = self.expr_to_type(Some(expr_kind));
        let key = names[0].syntax().text_range();
        self.expr_kinds.insert(key, expr_kind);

        for segment in &names[1..] {
            let Some(text) = segment.text() else {
                self.collect_function(function.idx(), stmt);
                return;
            };

            let arguments = vec![
                Some(ExpressionKind::Literal(Type::String(None))),
                Some(ExpressionKind::Literal(Type::Unknown)),
            ];

            let Some(container) = self.call_new_slot(typ, &arguments, stmt.syntax().text_range())
            else {
                self.collect_function(function.idx(), stmt);
                return;
            };

            let Some(expr_kind) = self
                .find_symbol(
                    self.get_container_members(container),
                    FindSymbol::BeforeIfInExecutionRange(offset),
                    &text,
                )
                .map(|id| ExpressionKind::Symbol(id))
            else {
                self.collect_function(function.idx(), stmt);
                return;
            };

            typ = self.expr_to_type(Some(expr_kind));
            let key = segment.syntax().text_range();
            self.expr_kinds.insert(key, expr_kind);
        }

        let arguments = vec![
            Some(ExpressionKind::Literal(Type::String(None))),
            Some(ExpressionKind::Literal(Type::Function(function))),
        ];

        let Some(container) = self.call_new_slot(typ, &arguments, stmt.syntax().text_range())
        else {
            self.collect_function(function.idx(), stmt);
            return;
        };

        let symbol = self.symbol(Symbol {
            name: final_name.clone(),
            typ: Type::Function(function),
            kind: SymbolKind::Property,
            range: stmt.syntax().text_range(),
        });

        self.add_container_member(container, final_name, symbol);

        self.collect_function(function.idx(), stmt);
    }

    fn enum_statement(&mut self, stmt: &EnumStatement) {
        let enum_ = EnumId::new(self.file, self.arena.alloc(EnumData::default()));

        if let Some(name) = stmt.name().and_then(|n| n.text()) {
            let symbol = self.symbol(Symbol {
                name: name.clone(),
                typ: Type::Enum(enum_),
                kind: SymbolKind::Enum,
                range: stmt.syntax().text_range(),
            });

            self.arena[self.const_table].add_member(name, symbol);
        }

        let save_symbol = self.container;
        self.container = Container::Enum(enum_.idx());
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
            self.expr_type(&discriminant)
        } else {
            Type::Unknown
        };

        for clause in stmt.clauses() {
            match clause {
                SwitchClause::Case(case) => {
                    if let Some(test) = case.test() {
                        let case_kind = self.expr_type(&test);
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
            Some(self.expr_type(&value))
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

        if !matches!(self.arena[function].ret, Type::Unknown | Type::Null) {
            return;
        }

        match kind {
            None | Some(Type::Unknown) => {}
            Some(kind) => {
                self.arena[function].ret = kind;
            }
        }
    }

    fn yield_statement(&mut self, stmt: &YieldStatement) {
        let kind = if let Some(value) = stmt.value() {
            self.expr_type(&value)
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

        if self.arena[function].yielding.is_none() {
            self.arena[function].yielding = Some(kind);
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
                let symbol = self.symbol(Symbol {
                    typ: Type::String(None),
                    name: name.clone(),
                    kind: SymbolKind::Local,
                    range: binding.syntax().text_range(),
                });

                self.current_scope().locals.insert(name, symbol);
            };
        }

        if let Some(body) = catch.body() {
            self.collect_stmt(&body);
        }
    }

    fn throw_statement(&mut self, stmt: &ThrowStatement) {
        // mark current function as exception throwing
        let kind = if let Some(value) = stmt.value() {
            self.expr_type(&value)
        } else {
            Type::Unknown
        };

        let Some(function) = self.function else {
            return;
        };

        if self.arena[function].throwing.is_none() {
            self.arena[function].throwing = Some(kind);
        }
    }

    fn expr_type(&mut self, expr: &Expr) -> Type {
        let kind = self.collect_expr(expr);
        self.expr_to_type(kind)
    }

    fn collect_expr(&mut self, expr: &Expr) -> NullableExprKind {
        let kind = match expr {
            Expr::Literal(expr) => self.literal_expression(expr),
            Expr::TableLiteral(expr) => Some(self.table_literal_expression(expr)),
            Expr::Class(expr) => Some(self.class_expression(expr)),
            Expr::ArrayLiteral(expr) => Some(self.array_literal_expression(expr)),
            Expr::Name(expr) => self.name_expression(expr),
            Expr::This(expr) => Some(self.this_expression(expr)),
            Expr::RootAccess(expr) => self.root_access_expression(expr),
            Expr::Base(expr) => Some(self.base_expression(expr)),
            Expr::MemberAccess(expr) => self.member_access_expression(expr),
            Expr::ElementAccess(expr) => self.element_access_expression(expr),
            Expr::Call(expr) => self.call_expression(expr),
            Expr::Clone(expr) => self.clone_expression(expr),
            Expr::Binary(expr) => self.binary_expression(expr),
            Expr::Conditional(expr) => Some(self.conditional_expression(expr)),
            Expr::PrefixUnary(expr) => self.prefix_unary_expression(expr),
            Expr::PrefixUpdate(expr) => self.prefix_update_expression(expr),
            Expr::PostfixUpdate(expr) => self.postfix_update_expression(expr),
            Expr::Delete(expr) => self.delete_expression(expr),
            Expr::TypeOf(expr) => Some(self.type_of_expression(expr)),
            Expr::Resume(expr) => self.resume_expression(expr),
            Expr::RawCall(expr) => self.raw_call_expression(expr),
            Expr::File(_) => Some(ExpressionKind::Literal(Type::String(None))),
            Expr::Line(_) => Some(ExpressionKind::Literal(Type::Integer)),
            Expr::Parenthesised(expr) => self.parenthesised_expression(expr),
            Expr::Function(expr) => Some(self.function_expression(expr)),
            Expr::Lambda(expr) => Some(self.lambda_expression(expr)),
        };
        if let Some(kind) = kind {
            let key = expr.syntax().text_range();
            self.expr_kinds.insert(key, kind);
        }
        kind
    }

    fn literal_expression(&mut self, expr: &LiteralExpression) -> NullableExprKind {
        let (kind, token) = expr.token()?;

        Some(match kind {
            LiteralExpressionKind::Integer => ExpressionKind::Literal(Type::Integer),
            LiteralExpressionKind::Character => ExpressionKind::Literal(Type::Integer),
            LiteralExpressionKind::Float => ExpressionKind::Literal(Type::Float),
            LiteralExpressionKind::String | LiteralExpressionKind::VerbatimString => {
                let string = StringId::new(
                    self.file,
                    self.arena
                        .alloc(unquote_string(token.text()).into_boxed_str()),
                );

                ExpressionKind::Literal(Type::String(Some(string)))
            }
            LiteralExpressionKind::Null => ExpressionKind::Literal(Type::Null),
            LiteralExpressionKind::True => ExpressionKind::Literal(Type::Boolean),
            LiteralExpressionKind::False => ExpressionKind::Literal(Type::Boolean),
        })
    }

    fn table_literal_expression(&mut self, expr: &TableLiteralExpression) -> ExpressionKind {
        let table = TableId::new(self.file, self.arena.alloc(TableData::default()));
        let save_symbol = self.container;
        self.container = Container::Table(table.idx());
        for member in expr.members() {
            self.collect_member(&member);
        }
        self.container = save_symbol;

        ExpressionKind::Literal(Type::Table(table))
    }

    fn class_expression(&mut self, expr: &ClassExpression) -> ExpressionKind {
        let class = self.class(expr);

        let save_symbol = self.container;
        self.container = Container::Class(class.idx());
        for member in expr.members() {
            self.collect_member(&member);
        }
        self.container = save_symbol;

        ExpressionKind::Literal(Type::Class(class))
    }

    fn array_literal_expression(&mut self, expr: &ArrayLiteralExpression) -> ExpressionKind {
        let mut types = expr.elements().map(|element| self.expr_type(&element));

        let Some(typ) = types.next() else {
            return ExpressionKind::Literal(Type::Array(
                self.array(ArrayData { typ: Type::Unknown }),
            ));
        };

        ExpressionKind::Literal(
            if types.all(|t| std::mem::discriminant(&t) == std::mem::discriminant(&typ)) {
                Type::Array(self.array(ArrayData { typ }))
            } else {
                Type::Array(self.array(ArrayData { typ: Type::Unknown }))
            },
        )
    }

    fn find_symbol(
        &self,
        iter: impl IntoIterator<Item = SymbolId>,
        settings: FindSymbol,
        text: &str,
    ) -> Option<SymbolId> {
        match settings {
            FindSymbol::Any => iter.into_iter().find(|symbol| {
                let symbol = self.get(*symbol);
                text == symbol.name
            }),
            FindSymbol::OnlyBefore(offset) => iter.into_iter().find(|symbol| {
                let symbol = self.get(*symbol);
                symbol.range.end() < offset && text == symbol.name
            }),
            FindSymbol::BeforeIfInExecutionRange(offset) => iter.into_iter().find(|symbol| {
                let symbol = self.get(*symbol);
                (!self.execution_range.contains_range(symbol.range) || symbol.range.end() < offset)
                    && text == symbol.name
            }),
        }
    }

    fn name_expression(&mut self, expr: &Name) -> NullableExprKind {
        let text = expr.text()?;
        let offset = expr.syntax().text_range().end();
        if let Some(symbol) = self.find_symbol(
            self.get_local_members(),
            FindSymbol::OnlyBefore(offset),
            &text,
        ) {
            return Some(ExpressionKind::Symbol(symbol));
        }

        if let Some(symbol) = self.find_symbol(
            self.get_const_members(),
            FindSymbol::OnlyBefore(offset),
            &text,
        ) {
            return Some(ExpressionKind::Symbol(symbol));
        }

        let mut already_included = FxHashSet::default();
        already_included.insert(self.file);
        for imports in self.imports.values() {
            for import in imports {
                let source_symbol = source_symbol(self.db, *import);
                let Some(symbol) = self.find_symbol(
                    source_symbol.get_members(
                        self.db,
                        self.file,
                        &mut already_included,
                        GetMembers::Const,
                    ),
                    FindSymbol::Any,
                    &text,
                ) else {
                    continue;
                };

                return Some(ExpressionKind::Symbol(symbol));
            }
        }

        if let Some(symbol) = self.find_symbol(
            self.get_current_container_members(),
            FindSymbol::BeforeIfInExecutionRange(offset),
            &text,
        ) {
            return Some(ExpressionKind::Symbol(symbol));
        }

        let mut already_included = FxHashSet::default();
        already_included.insert(self.file);
        if let Some(imports) = self.imports.get(&self.container) {
            for import in imports {
                let source_symbol = source_symbol(self.db, *import);
                let Some(symbol) = self.find_symbol(
                    source_symbol.get_members(
                        self.db,
                        self.file,
                        &mut already_included,
                        GetMembers::Source,
                    ),
                    FindSymbol::Any,
                    &text,
                ) else {
                    continue;
                };

                return Some(ExpressionKind::Symbol(symbol));
            }
        }

        if let Some(symbol) = self.find_symbol(
            self.get_root_members(),
            FindSymbol::BeforeIfInExecutionRange(offset),
            &text,
        ) {
            return Some(ExpressionKind::Symbol(symbol));
        }

        let mut already_included = FxHashSet::default();
        already_included.insert(self.file);
        for imports in self.imports.values() {
            for import in imports {
                let source_symbol = source_symbol(self.db, *import);

                let Some(symbol) = self.find_symbol(
                    source_symbol.get_members(
                        self.db,
                        self.file,
                        &mut already_included,
                        GetMembers::Root,
                    ),
                    FindSymbol::Any,
                    &text,
                ) else {
                    continue;
                };

                return Some(ExpressionKind::Symbol(symbol));
            }
        }

        None
    }

    fn this_expression(&self, _expr: &ThisExpression) -> ExpressionKind {
        ExpressionKind::Literal(match self.container {
            Container::Class(idx) => Type::Class(ClassId::new(self.file, idx)),
            Container::Table(idx) => Type::Table(TableId::new(self.file, idx)),
            Container::Enum(idx) => Type::Enum(EnumId::new(self.file, idx)),
        })
    }

    fn root_access_expression(&mut self, expr: &RootAccessExpression) -> NullableExprKind {
        let text = expr.name()?.text()?;
        let offset = expr.syntax().text_range().end();

        if let Some(symbol) = self.find_symbol(
            self.get_root_members(),
            FindSymbol::BeforeIfInExecutionRange(offset),
            &text,
        ) {
            return Some(ExpressionKind::Symbol(symbol));
        }

        let mut already_included = FxHashSet::default();
        already_included.insert(self.file);
        for imports in self.imports.values() {
            for import in imports {
                let source_symbol = source_symbol(self.db, *import);
                let Some(symbol) = self.find_symbol(
                    source_symbol.get_members(
                        self.db,
                        self.file,
                        &mut already_included,
                        GetMembers::Root,
                    ),
                    FindSymbol::Any,
                    &text,
                ) else {
                    continue;
                };

                return Some(ExpressionKind::Symbol(symbol));
            }
        }

        None
    }

    fn base_expression(&mut self, expr: &BaseExpression) -> ExpressionKind {
        match self.container {
            Container::Class(id) => {
                let class = &self.arena[id];
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

    fn member_access_expression(&mut self, expr: &MemberAccessExpression) -> NullableExprKind {
        let obj = self.expr_type(&expr.object()?);
        let text = expr.member_part()?.name()?.text()?;

        let members = self.get_type_members(obj);

        let offset = expr.syntax().text_range().end();
        if let Some(symbol) =
            self.find_symbol(members, FindSymbol::BeforeIfInExecutionRange(offset), &text)
        {
            return Some(ExpressionKind::Symbol(symbol));
        };

        Container::try_from(obj).ok().and_then(|c| {
            let mut already_included = FxHashSet::default();
            already_included.insert(self.file);
            let imports = self.imports.get(&c)?;
            for import in imports {
                let source_symbol = source_symbol(self.db, *import);
                let Some(symbol) = self.find_symbol(
                    source_symbol.get_members(
                        self.db,
                        self.file,
                        &mut already_included,
                        GetMembers::Source,
                    ),
                    FindSymbol::Any,
                    &text,
                ) else {
                    continue;
                };

                return Some(ExpressionKind::Symbol(symbol));
            }
            None
        })
    }

    fn element_access_expression(&mut self, expr: &ElementAccessExpression) -> NullableExprKind {
        let obj = self.expr_type(&expr.object()?);
        let index = expr.index()?.expression()?;
        let Type::String(Some(id)) = self.expr_type(&index) else {
            return None;
        };

        let text = self.get(id).as_ref();
        let members = self.get_type_members(obj);

        let offset = expr.syntax().text_range().end();
        if let Some(symbol) =
            self.find_symbol(members, FindSymbol::BeforeIfInExecutionRange(offset), &text)
        {
            return Some(ExpressionKind::Symbol(symbol));
        };

        Container::try_from(obj).ok().and_then(|c| {
            let mut already_included = FxHashSet::default();
            already_included.insert(self.file);
            let imports = self.imports.get(&c)?;
            for import in imports {
                let source_symbol = source_symbol(self.db, *import);
                let Some(symbol) = self.find_symbol(
                    source_symbol.get_members(
                        self.db,
                        self.file,
                        &mut already_included,
                        GetMembers::Source,
                    ),
                    FindSymbol::Any,
                    &text,
                ) else {
                    continue;
                };

                return Some(ExpressionKind::Symbol(symbol));
            }
            None
        })
    }

    fn call_type(
        &mut self,
        callable: Type,
        arguments: &[NullableExprKind],
        error_range: TextRange,
    ) -> Option<Type> {
        match callable {
            Type::Function(id) => {
                let is_variadic = matches!(self.get(id).params_state, ParamsState::VarArgs(_));
                for (count, argument_kind) in arguments.iter().cloned().enumerate() {
                    let Some(&param) = self.get(id).params.get(count) else {
                        if !is_variadic {
                            self.diagnostics.push(Diagnostic {
                                message: "Passing more parameters than possible".to_owned(),
                                range: error_range,
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        continue;
                    };

                    match (self.expr_to_type(argument_kind), self.get(param).typ) {
                        (Type::Unknown | Type::Null, required_kind) => {
                            // If passed in parameter has type of unknown
                            // we can coerce it to be the type of a required parameter
                            if let Some(ExpressionKind::Symbol(id)) = argument_kind
                                && !required_kind.should_substitute()
                            {
                                if let Some(symbol) = self.get_mut(id) {
                                    symbol.typ = required_kind;
                                }
                            }
                        }
                        (passed, Type::Unknown | Type::Null) => {
                            if let Some(symbol) = self.get_mut(param) {
                                symbol.typ = passed;
                            }
                        }

                        (Type::Integer | Type::Float, Type::Integer | Type::Float) => {}

                        (passed, required) => {
                            if discriminant(&passed) != discriminant(&required) {
                                self.diagnostics.push(Diagnostic {
                                    message: format!("Expected parameter of type '{required}', but got '{passed}'"),
                                    range: error_range,
                                    severity: DiagnosticSeverity::Warning,
                                });
                            }
                        }
                    }
                }

                let enough_parameters = match self.get(id).params_state {
                    ParamsState::NoDefault => arguments.len() == self.get(id).params.len(),
                    ParamsState::Default(from) | ParamsState::VarArgs(from) => {
                        arguments.len() >= from
                    }
                };

                if !enough_parameters {
                    self.diagnostics.push(Diagnostic {
                        message: "Insufficient number of parameters passed".to_owned(),
                        range: error_range,
                        severity: DiagnosticSeverity::Error,
                    });
                }

                Some(if self.get(id).yielding.is_some() {
                    Type::Generator(id)
                } else {
                    self.clone_type(self.get(id).ret)
                })
            }
            Type::Class(id) => {
                let class = self.get(id);
                if let Some(symbol) = class.get_member("constructor") {
                    self.call_type(self.get(symbol).typ, arguments, error_range);
                } else if arguments.len() != 0 {
                    self.diagnostics.push(Diagnostic {
                        message: "Default constructor should have no parameters".to_owned(),
                        range: error_range,
                        severity: DiagnosticSeverity::Error,
                    });
                }

                Some(Type::Instance(id))
            }
            _ => self.call_metamethod(
                callable,
                "_call",
                arguments,
                error_range,
                MetamethodErrors::Yes { keyword: "calling" },
            ),
        }
    }

    fn call_expression(&mut self, expr: &CallExpression) -> NullableExprKind {
        let obj = match expr.callee() {
            Some(expr) => self.expr_type(&expr),
            None => Type::Unknown,
        };

        let arguments: Vec<_> = expr
            .arguments()
            .map(|arg| self.collect_expr(&arg))
            .collect();

        Some(ExpressionKind::Literal(self.call_type(
            obj,
            &arguments,
            expr.syntax().text_range(),
        )?))
    }

    fn clone_expression(&mut self, expr: &CloneExpression) -> NullableExprKind {
        let operand = expr.operand()?;
        let kind = self.expr_type(&operand);
        Some(ExpressionKind::Literal(self.clone_type(kind)))
    }

    fn extract_lhs_and_rhs(
        &mut self,
        expr: &BinaryExpression,
    ) -> (NullableExprKind, NullableExprKind) {
        let left = expr.lhs().and_then(|l| self.collect_expr(&l));
        let right = expr.rhs().and_then(|r| self.collect_expr(&r));
        (left, right)
    }

    fn assignment_lhs(&mut self, expr: &Expr) -> Option<AssignmentLeftHandSide> {
        match expr {
            Expr::Name(expr) => {
                let text = expr.text()?;
                let offset = expr.syntax().text_range().end();

                let locals = self.find_symbol(
                    self.get_local_members(),
                    FindSymbol::OnlyBefore(offset),
                    &text,
                );

                if let Some(symbol) = locals {
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: None,
                        range: expr.syntax().text_range(),
                        symbol,
                    });
                }

                let consts = self.find_symbol(
                    self.get_const_members(),
                    FindSymbol::OnlyBefore(offset),
                    &text,
                );

                if let Some(symbol) = consts {
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(Type::Table(TableId::new(self.file, self.const_table))),
                        range: expr.syntax().text_range(),
                        symbol,
                    });
                }

                let members = self.find_symbol(
                    self.get_current_container_members(),
                    FindSymbol::BeforeIfInExecutionRange(offset),
                    &text,
                );

                if let Some(symbol) = members {
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(self.container.to_type(self.file)),
                        range: expr.syntax().text_range(),
                        symbol,
                    });
                }

                let root = self.find_symbol(
                    self.get_root_members(),
                    FindSymbol::BeforeIfInExecutionRange(offset),
                    &text,
                );

                if let Some(symbol) = root {
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(Type::Table(TableId::new(self.file, self.root_table))),
                        range: expr.syntax().text_range(),
                        symbol,
                    });
                }

                Some(AssignmentLeftHandSide::CanCreate {
                    parent: self.container.to_type(self.file),
                    range: expr.syntax().text_range(),
                    new_key: text.into_boxed_str(),
                })
            }
            Expr::MemberAccess(expr) => {
                let obj = self.expr_type(&expr.object()?);
                let text = expr.member_part()?.name()?.text()?;

                let members = self.get_type_members(obj);

                let offset = expr.syntax().text_range().end();
                Some(
                    self.find_symbol(members, FindSymbol::BeforeIfInExecutionRange(offset), &text)
                        .map_or_else(
                            || AssignmentLeftHandSide::CanCreate {
                                parent: obj,
                                range: expr.syntax().text_range(),
                                new_key: text.into_boxed_str(),
                            },
                            |id| AssignmentLeftHandSide::Exists {
                                parent: Some(obj),
                                range: expr.syntax().text_range(),
                                symbol: id,
                            },
                        ),
                )
            }
            Expr::ElementAccess(expr) => {
                let obj = self.expr_type(&expr.object()?);
                let index = expr.index()?.expression()?;
                let index_kind = self.collect_expr(&index);
                let Type::String(Some(id)) = self.expr_to_type(index_kind) else {
                    return Some(AssignmentLeftHandSide::NonStringKey {
                        parent: obj,
                        range: expr.syntax().text_range(),
                        key: index_kind,
                    });
                };

                let text = self.get(id).as_ref();
                let members = self.get_type_members(obj);

                let offset = expr.syntax().text_range().end();
                Some(
                    self.find_symbol(members, FindSymbol::BeforeIfInExecutionRange(offset), &text)
                        .map_or_else(
                            || AssignmentLeftHandSide::CanCreate {
                                parent: obj,
                                range: expr.syntax().text_range(),
                                new_key: text.into(),
                            },
                            |id| AssignmentLeftHandSide::Exists {
                                parent: Some(obj),
                                range: expr.syntax().text_range(),
                                symbol: id,
                            },
                        ),
                )
            }
            Expr::RootAccess(expr) => {
                let text = expr.name()?.text()?;
                let offset = expr.syntax().text_range().end();
                Some(
                    self.find_symbol(
                        self.get_root_members(),
                        FindSymbol::BeforeIfInExecutionRange(offset),
                        &text,
                    )
                    .map_or_else(
                        || AssignmentLeftHandSide::CanCreate {
                            parent: Type::Table(TableId::new(self.file, self.root_table)),
                            range: expr.syntax().text_range(),
                            new_key: text.into_boxed_str(),
                        },
                        |id| AssignmentLeftHandSide::Exists {
                            parent: None,
                            range: expr.syntax().text_range(),
                            symbol: id,
                        },
                    ),
                )
            }
            _ => Some(AssignmentLeftHandSide::Invalid(self.collect_expr(expr))),
        }
    }

    fn binary_expression(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let (operator, _) = expr.operator()?;
        match operator {
            BinaryOperator::NewSlot => self.new_slot_operator(expr),
            BinaryOperator::Assign => self.assign_operator(expr),
            BinaryOperator::Comma => self.comma_operator(expr),
            BinaryOperator::In => self.in_operator(expr),
            BinaryOperator::InstanceOf => Some(self.instance_of_operator(expr)),
            BinaryOperator::Equals | BinaryOperator::NotEquals => {
                Some(self.equality_operator(expr))
            }
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::ThreeWay => {
                Some(self.comparison_operator(expr, operator == BinaryOperator::ThreeWay))
            }
            BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
            | BinaryOperator::LeftShift
            | BinaryOperator::RightShift
            | BinaryOperator::UnsignedRightShift => Some(self.bitwise_operator(expr)),

            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                Some(self.logical_operator(expr))
            }

            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => self.arithmetic_operator(expr, operator),

            BinaryOperator::AddAssign
            | BinaryOperator::SubtractAssign
            | BinaryOperator::MultiplyAssign
            | BinaryOperator::DivideAssign
            | BinaryOperator::ModuloAssign => {
                let left_kind = expr.lhs().and_then(|l| self.assignment_lhs(&l));
                let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
                self.arithmetic_assign_operator(left_kind, right_kind, operator)
            }
        }
    }

    // Also used by class statement
    fn do_new_slot(
        &mut self,
        left_kind: Option<AssignmentLeftHandSide>,
        right_kind: NullableExprKind,
        expr_range: TextRange,
    ) -> NullableExprKind {
        match left_kind {
            Some(AssignmentLeftHandSide::CanCreate {
                parent,
                new_key,
                range,
            }) => {
                let arguments = vec![
                    Some(ExpressionKind::Literal(Type::String(None))),
                    right_kind,
                ];

                let Some(container) = self.call_new_slot(parent, &arguments, range) else {
                    return right_kind;
                };

                let symbol = self.symbol(Symbol {
                    typ: self.expr_to_type(right_kind),
                    name: new_key.to_string(),
                    kind: SymbolKind::Property,
                    range: expr_range,
                });

                self.add_container_member(container, new_key.into_string(), symbol);

                if let Some(right_kind) = right_kind {
                    self.expr_kinds.insert(range, right_kind);
                }
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                range,
            }) => {
                if let Some(parent) = parent {
                    let arguments = vec![
                        Some(ExpressionKind::Literal(Type::String(None))),
                        right_kind,
                    ];
                    if self.call_new_slot(parent, &arguments, range).is_none() {
                        return right_kind;
                    }
                }

                if matches!(
                    self.get(symbol).kind,
                    SymbolKind::Local | SymbolKind::Constant | SymbolKind::Enum
                ) {
                    // ```
                    // local a = 2
                    // a <- 1
                    // ```
                    // is illegal
                    self.diagnostics.push(Diagnostic {
                        message: "Cannot create a new slot with the same name as a local or constant due to the resolution precedence. Prepend variable name with `this.` if you wish to do that".to_owned(),
                        range,
                        severity: DiagnosticSeverity::Error,
                    });
                    return right_kind;
                }

                let typ = self.expr_to_type(right_kind);

                let Some(symbol) = self.get_mut(symbol) else {
                    return right_kind;
                };

                // Update symbol kind if it's null or unknown
                if matches!(symbol.typ, Type::Unknown | Type::Null)
                    && !matches!(typ, Type::Unknown | Type::Null)
                {
                    symbol.typ = typ;

                    self.expr_kinds.insert(range, right_kind.unwrap());
                }
            }
            Some(AssignmentLeftHandSide::NonStringKey { parent, key, range }) => {
                let arguments = vec![key, right_kind];
                self.call_new_slot(parent, &arguments, range);
            }
            _ => {}
        }
        right_kind
    }

    fn new_slot_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let left_kind = expr.lhs().and_then(|l| self.assignment_lhs(&l));
        let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
        self.do_new_slot(left_kind, right_kind, expr.syntax().text_range())
    }

    fn assign_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let left_kind = expr.lhs().and_then(|l| self.assignment_lhs(&l));
        let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));

        match left_kind {
            Some(AssignmentLeftHandSide::CanCreate { parent, range, .. }) => {
                let arguments = vec![
                    Some(ExpressionKind::Literal(Type::String(None))),
                    right_kind,
                ];
                self.call_set(parent, &arguments, range);
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                range,
            }) => {
                if !self.get(symbol).kind.is_modifiable() {
                    let name = &self.get(symbol).name;
                    self.diagnostics.push(Diagnostic {
                        message: format!("Symbol '{name}' is not modifiable"),
                        range,
                        severity: DiagnosticSeverity::Error,
                    });
                    return right_kind;
                }

                let arguments = vec![
                    Some(ExpressionKind::Literal(Type::String(None))),
                    right_kind,
                ];
                if let Some(parent) = parent
                    && !self.call_set(parent, &arguments, range)
                {
                    return right_kind;
                }

                let typ = self.expr_to_type(right_kind);
                let Some(symbol) = self.get_mut(symbol) else {
                    return right_kind;
                };

                if symbol.typ.should_substitute() && !typ.should_substitute() {
                    symbol.typ = typ;
                    self.expr_kinds.insert(range, right_kind.unwrap());
                }
            }
            Some(AssignmentLeftHandSide::NonStringKey { parent, key, range }) => {
                let arguments = vec![key, right_kind];
                self.call_set(parent, &arguments, range);
            }

            _ => {}
        }
        right_kind
    }

    fn comma_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let (_left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        right_kind
    }

    fn in_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let right = self.expr_to_type(right_kind);

        match right {
            Type::Array(_) => {
                let left = self.expr_to_type(left_kind);
                if !matches!(left, Type::Unknown | Type::Integer) {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Trying to index into an array using '{left}' (only integers are applicable)"),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Warning
                    });
                }
            }
            Type::Table(_) | Type::Class(_) | Type::Instance(_) | Type::Unknown => {}
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: format!("Indexing into '{right}' will always return false"),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                });
            }
        }
        Some(ExpressionKind::Literal(Type::Boolean))
    }

    fn instance_of_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let left = self.expr_to_type(left_kind);
        let right = self.expr_to_type(right_kind);

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
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Error,
                });
            }
        }

        ExpressionKind::Literal(Type::Boolean)
    }

    fn equality_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (_left_kind, _right_kind) = self.extract_lhs_and_rhs(expr);
        ExpressionKind::Literal(Type::Boolean)
    }

    fn comparison_operator(
        &mut self,
        expr: &BinaryExpression,
        is_three_way: bool,
    ) -> ExpressionKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let left = self.expr_to_type(left_kind);
        let right = self.expr_to_type(right_kind);

        match (left, right) {
            (Type::Unknown | Type::Null, _)
            | (_, Type::Unknown | Type::Null)
            | (Type::Integer | Type::Float, Type::Integer | Type::Float)
            | (Type::String(_), Type::String(_))
            | (Type::Boolean, Type::Boolean) => {}
            (Type::Table(_), Type::Table(_)) | (Type::Instance(_), Type::Instance(_)) => {
                let arguments = vec![right_kind];
                match self.call_metamethod(
                    left,
                    "_cmp",
                    &arguments,
                    expr.syntax().text_range(),
                    MetamethodErrors::No,
                ) {
                    Some(Type::Integer) | Some(Type::Unknown) => {}
                    Some(_) => self.diagnostics.push(Diagnostic {
                        message: "'_cmp' must return an integer".to_owned(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Error,
                    }),
                    None => {
                        self.diagnostics.push(Diagnostic {
                            message: if matches!(left, Type::Table(_)) {
                                "Comparing classes with no '_cmp' delegate metamethod defined. The result is undetermenistic".to_owned()
                            } else {
                                "Comparing instances with no '_cmp' class metamethod defined. The result is undetermenistic".to_owned()
                            },
                            range: expr.syntax().text_range(),
                            severity: DiagnosticSeverity::Warning
                        });
                    }
                }
            }
            _ => self.diagnostics.push(Diagnostic {
                message: format!("'{left}' does not support comparison with '{right}'"),
                range: expr.syntax().text_range(),
                severity: DiagnosticSeverity::Error,
            }),
        }

        ExpressionKind::Literal(if is_three_way {
            Type::Integer
        } else {
            Type::Boolean
        })
    }

    fn bitwise_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let left = self.expr_to_type(left_kind);
        let right = self.expr_to_type(right_kind);

        match (left, right) {
            (Type::Integer, Type::Integer) => {}
            (Type::Unknown | Type::Null, Type::Unknown | Type::Null) => {}
            (Type::Integer, Type::Unknown | Type::Null) => {
                if let Some(ExpressionKind::Symbol(symbol)) = right_kind {
                    if let Some(symbol) = self.get_mut(symbol) {
                        symbol.typ = Type::Integer;
                    }
                }
            }
            (Type::Unknown | Type::Null, Type::Integer) => {
                if let Some(ExpressionKind::Symbol(symbol)) = left_kind {
                    if let Some(symbol) = self.get_mut(symbol) {
                        symbol.typ = Type::Integer;
                    }
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: format!("'{left}' does not support bitwise operator with '{right}'"),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Error,
                });
            }
        }

        ExpressionKind::Literal(Type::Integer)
    }

    fn logical_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let left = self.expr_to_type(left_kind);
        let right = self.expr_to_type(right_kind);

        ExpressionKind::Literal(if left.should_substitute() {
            right
        } else {
            left
        })
    }

    fn arithmetic_operator(
        &mut self,
        expr: &BinaryExpression,
        operator: BinaryOperator,
    ) -> Option<ExpressionKind> {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let result =
            self.call_arithmetic(left_kind, right_kind, operator, expr.syntax().text_range())?;
        Some(ExpressionKind::Literal(result))
    }

    // This signature is so weird because it is also used by increment / decrement operators
    fn arithmetic_assign_operator(
        &mut self,
        left_kind: Option<AssignmentLeftHandSide>,
        right_kind: NullableExprKind,
        operator: BinaryOperator,
    ) -> Option<ExpressionKind> {
        match left_kind {
            Some(AssignmentLeftHandSide::CanCreate { parent, range, .. }) => {
                let Some(typ) = self.call_arithmetic(
                    Some(ExpressionKind::Literal(Type::String(None))),
                    right_kind,
                    operator,
                    range,
                ) else {
                    return right_kind;
                };

                let arguments = vec![
                    Some(ExpressionKind::Literal(Type::String(None))),
                    Some(ExpressionKind::Literal(typ)),
                ];
                self.call_set(parent, &arguments, range);
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                range,
            }) => {
                let Some(typ) = self.call_arithmetic(
                    Some(ExpressionKind::Symbol(symbol)),
                    right_kind,
                    operator,
                    range,
                ) else {
                    return right_kind;
                };

                if !self.get(symbol).kind.is_modifiable() {
                    let name = &self.get(symbol).name;
                    self.diagnostics.push(Diagnostic {
                        message: format!("Symbol '{name}' is not modifiable"),
                        range,
                        severity: DiagnosticSeverity::Error,
                    });
                    return right_kind;
                }

                let arguments = vec![
                    Some(ExpressionKind::Literal(Type::String(None))),
                    Some(ExpressionKind::Literal(typ)),
                ];

                if let Some(parent) = parent
                    && !self.call_set(parent, &arguments, range)
                {
                    return right_kind;
                }

                let typ = self.expr_to_type(right_kind);
                let Some(symbol) = self.get_mut(symbol) else {
                    return right_kind;
                };

                if symbol.typ.should_substitute() && !typ.should_substitute() {
                    symbol.typ = typ;

                    self.expr_kinds.insert(range, right_kind.unwrap());
                }
            }
            Some(AssignmentLeftHandSide::NonStringKey { parent, key, range }) => {
                let Some(typ) = self.call_arithmetic(key, right_kind, operator, range) else {
                    return right_kind;
                };

                let arguments = vec![key, Some(ExpressionKind::Literal(typ))];
                self.call_set(parent, &arguments, range);
            }

            _ => {}
        }
        right_kind
    }

    fn conditional_expression(&mut self, expr: &ConditionalExpression) -> ExpressionKind {
        if let Some(expr) = expr.condition() {
            self.collect_expr(&expr);
        };

        let then_kind = if let Some(expr) = expr.then_branch().and_then(|b| b.expression()) {
            self.expr_type(&expr)
        } else {
            Type::Unknown
        };

        let else_kind = if let Some(expr) = expr.else_branch().and_then(|b| b.expression()) {
            self.expr_type(&expr)
        } else {
            Type::Unknown
        };

        ExpressionKind::Literal(if then_kind != Type::Unknown {
            then_kind
        } else {
            else_kind
        })
    }

    fn prefix_unary_expression(&mut self, expr: &PrefixUnaryExpression) -> NullableExprKind {
        let (operator, _) = expr.operator()?;
        match operator {
            PrefixUnaryOperator::Negation => self.negation_operator(expr),
            PrefixUnaryOperator::BitwiseNot => Some(self.bitwise_not_operator(expr)),
            PrefixUnaryOperator::LogicalNot => Some(self.logical_not_operator(expr)),
        }
    }

    fn negation_operator(&mut self, expr: &PrefixUnaryExpression) -> NullableExprKind {
        let typ = self.expr_type(&expr.operand()?);

        Some(ExpressionKind::Literal(match typ {
            Type::Integer | Type::Float => typ,
            _ => self.call_metamethod(
                typ,
                "_unm",
                &Vec::new(),
                expr.syntax().text_range(),
                MetamethodErrors::Yes {
                    keyword: "negation",
                },
            )?,
        }))
    }

    fn bitwise_not_operator(&mut self, expr: &PrefixUnaryExpression) -> ExpressionKind {
        let typ = if let Some(operand) = expr.operand() {
            self.expr_type(&operand)
        } else {
            Type::Unknown
        };

        match typ {
            Type::Integer | Type::Unknown => {}
            _ => self.diagnostics.push(Diagnostic {
                message: format!("'{typ}' does not support bitwise not operator"),
                range: expr.syntax().text_range(),
                severity: DiagnosticSeverity::Error,
            }),
        }

        ExpressionKind::Literal(Type::Integer)
    }

    fn logical_not_operator(&mut self, expr: &PrefixUnaryExpression) -> ExpressionKind {
        if let Some(operand) = expr.operand() {
            self.collect_expr(&operand);
        }

        ExpressionKind::Literal(Type::Boolean)
    }

    fn prefix_update_expression(&mut self, expr: &PrefixUpdateExpression) -> NullableExprKind {
        let (operator, _) = expr.operator()?;
        match operator {
            PrefixUpdateOperator::Increment => self.prefix_increment_operator(expr),
            PrefixUpdateOperator::Decrement => self.prefix_decrement_operator(expr),
        }
    }

    fn prefix_increment_operator(&mut self, expr: &PrefixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let increment = Some(ExpressionKind::Literal(Type::Integer));
        self.arithmetic_assign_operator(operand, increment, BinaryOperator::AddAssign)
    }

    fn prefix_decrement_operator(&mut self, expr: &PrefixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let increment = Some(ExpressionKind::Literal(Type::Integer));
        self.arithmetic_assign_operator(operand, increment, BinaryOperator::SubtractAssign)
    }

    fn postfix_update_expression(&mut self, expr: &PostfixUpdateExpression) -> NullableExprKind {
        let (operator, _) = expr.operator()?;
        match operator {
            PostfixUpdateOperator::Increment => self.postfix_increment_operator(expr),
            PostfixUpdateOperator::Decrement => self.postfix_decrement_operator(expr),
        }
    }

    fn postfix_increment_operator(&mut self, expr: &PostfixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let increment = Some(ExpressionKind::Literal(Type::Integer));
        self.arithmetic_assign_operator(operand.clone(), increment, BinaryOperator::AddAssign);
        operand?.into()
    }

    fn postfix_decrement_operator(&mut self, expr: &PostfixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let increment = Some(ExpressionKind::Literal(Type::Integer));
        self.arithmetic_assign_operator(operand.clone(), increment, BinaryOperator::SubtractAssign);
        operand?.into()
    }

    fn delete_expression(&mut self, expr: &DeleteExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        match operand {
            Some(AssignmentLeftHandSide::CanCreate { parent, range, .. })
            | Some(AssignmentLeftHandSide::NonStringKey { parent, range, .. }) => {
                self.call_delete(parent, None, range);
                return operand?.into();
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                range,
            }) => {
                if let Some(parent) = parent {
                    self.call_delete(
                        parent,
                        Some(ExpressionKind::Literal(Type::String(None))),
                        range,
                    );

                    return Some(ExpressionKind::Literal(self.get(symbol).typ));
                }

                if matches!(
                    self.get(symbol).kind,
                    SymbolKind::Local | SymbolKind::Constant | SymbolKind::Enum
                ) {
                    // ```
                    // local a = 2
                    // delete a
                    // ```
                    // is illegal
                    self.diagnostics.push(Diagnostic {
                        message: "Cannot delete a variable with the same name as a local or constant due to the resolution precedence. Prepend variable name with `this.` if you wish to do that".to_owned(),
                        range,
                        severity: DiagnosticSeverity::Error,
                    });
                }

                Some(ExpressionKind::Literal(self.get(symbol).typ))
            }
            _ => None,
        }
    }

    fn type_of_expression(&mut self, expr: &TypeOfExpression) -> ExpressionKind {
        let Some(operand) = expr.operand().map(|o| self.expr_type(&o)) else {
            return ExpressionKind::Literal(Type::String(None));
        };

        ExpressionKind::Literal(
            self.call_metamethod(
                operand,
                "_typeof",
                &Vec::new(),
                expr.syntax().text_range(),
                MetamethodErrors::No,
            )
            .unwrap_or(Type::String(None)),
        )
    }

    fn resume_expression(&mut self, expr: &ResumeExpression) -> NullableExprKind {
        let typ = self.expr_type(&expr.operand()?);

        match typ {
            Type::Unknown => None,
            Type::Generator(id) => Some(ExpressionKind::Literal(self.get(id).yielding?)),
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: "Only generators can be resumed".to_owned(),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Error,
                });
                None
            }
        }
    }

    fn raw_call_expression(&mut self, expr: &RawCallExpression) -> NullableExprKind {
        let mut arguments: Vec<_> = expr
            .arguments()
            .map(|arg| self.collect_expr(&arg))
            .collect();

        if arguments.len() < 2 {
            self.diagnostics.push(Diagnostic {
                message: "'rawcall' requires at least 2 parameters: function to call and context"
                    .to_owned(),
                range: expr.syntax().text_range(),
                severity: DiagnosticSeverity::Error,
            });
            return None;
        }

        let function = arguments.remove(0);
        let _context = arguments.remove(0);

        let obj = self.expr_to_type(function);
        Some(ExpressionKind::Literal(self.call_type(
            obj,
            &arguments,
            expr.syntax().text_range(),
        )?))
    }

    fn parenthesised_expression(&mut self, expr: &ParenthesisedExpression) -> NullableExprKind {
        let expr = expr.inner()?;
        self.collect_expr(&expr)
    }

    fn function_expression(&mut self, expr: &FunctionExpression) -> ExpressionKind {
        let function = self.function();
        self.collect_function(function.idx(), expr);
        ExpressionKind::Literal(Type::Function(function))
    }

    fn lambda_expression(&mut self, expr: &LambdaExpression) -> ExpressionKind {
        let function = self.function();
        self.collect_function(function.idx(), expr);
        ExpressionKind::Literal(Type::Function(function))
    }
}
