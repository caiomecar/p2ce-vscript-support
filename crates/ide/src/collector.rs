use la_arena::Idx;
use rustc_hash::FxHashMap;
use sq_3_parser::{
    AstNode, TextRange, TextSize,
    ast::{self, *},
};
use std::{collections::VecDeque, mem::discriminant};

use crate::{
    Diagnostic, DiagnosticSeverity, ExpressionKind, File, FileState, FindSymbol, GetMembers,
    NullableExprKind, SourceSymbol, SourceSymbolic,
    arena::{
        ArenaAlloc, ArenaId, ArrayData, ArrayId, ClassData, ClassId, Container, EnumData, EnumId,
        FunctionData, FunctionId, ParamsState, Scope, ScopeId, SourceArena, StringId, SymbolId,
        TableData, TableId,
    },
    db::Db,
    symbol::{Symbol, SymbolKind, SymbolTable, Type},
};

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
    scope: ScopeId,
}

#[derive(Debug, Clone)]
enum AssignmentLeftHandSide {
    CanCreate {
        parent: Type,
        new_key: Box<str>,
        name_range: TextRange,
        range: TextRange,
    },
    // Parent doesn't exist for locals
    Exists {
        parent: Option<Type>,
        symbol: SymbolId,
        range: TextRange,
    },
    NonStringKey {
        parent: Type,
        key: NullableExprKind,
        range: TextRange,
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

fn lhs_container(lhs: Option<&AssignmentLeftHandSide>) -> Option<Container> {
    let parent = match lhs {
        Some(AssignmentLeftHandSide::CanCreate { parent, .. }) => parent,
        Some(AssignmentLeftHandSide::Exists { parent, .. }) => parent.as_ref()?,
        Some(AssignmentLeftHandSide::NonStringKey { parent, .. }) => parent,
        Some(AssignmentLeftHandSide::Invalid(_)) => return None,
        None => return None,
    };

    Container::try_from(parent.clone()).ok()
}

fn get_name<T>(node: &T) -> Option<(Name, String)>
where
    T: HasName,
{
    let name = node.name()?;
    let text = name.text()?;
    Some((name, text))
}

impl TryFrom<AssignmentLeftHandSide> for Type {
    type Error = ();
    fn try_from(value: AssignmentLeftHandSide) -> Result<Self, Self::Error> {
        match value {
            AssignmentLeftHandSide::CanCreate { parent, .. } => Ok(parent),
            AssignmentLeftHandSide::Exists { parent, .. } => parent.ok_or(()),
            AssignmentLeftHandSide::NonStringKey { parent, .. } => Ok(parent),
            AssignmentLeftHandSide::Invalid(_) => Err(()),
        }
    }
}

impl TryFrom<AssignmentLeftHandSide> for Container {
    type Error = ();

    fn try_from(value: AssignmentLeftHandSide) -> Result<Self, Self::Error> {
        let typ = Type::try_from(value)?;
        Self::try_from(typ)
    }
}

enum MetamethodErrors {
    No,
    Yes { keyword: &'static str },
    YesBinary { keyword: &'static str, right: Type },
}

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

    scope: ScopeId,

    /// The container new members will be added to. Note that this is different from
    /// container that we take symbols from. That one is stored on the scope and can
    /// be acquired via .execution_container()
    container: Container,
    /// If a function overwrites the current container, either via static environment
    /// binding function a[whatever](){} or by trying to reside outside of the current
    /// container, e.g. a.b.c <- function(){need to show completions inside 'b' in here}
    /// function a::b::c(){same deal here}
    /// while the second reason is not perfect since it can be overwritten, which would
    /// lead to errors, it's still the best approximation we can possibly use
    special_container: Option<Container>,

    can_break: bool,
    can_continue: bool,
    dead_code: bool,

    execution_range: TextRange,
    function: Option<Idx<FunctionData>>,
    deferred_functions: VecDeque<DeferredFunction>,

    expr_kinds: FxHashMap<TextRange, ExpressionKind>,
    name_kinds: FxHashMap<TextRange, SymbolId>,
    diagnostics: Vec<Diagnostic>,
}

impl<'db> SourceSymbolic for Collector<'db> {
    fn imports(&self) -> &FxHashMap<Container, Vec<File>> {
        &self.imports
    }

    fn arena(&self) -> &SourceArena {
        &self.arena
    }

    fn const_table(&self) -> Idx<TableData> {
        self.const_table
    }

    fn root_table(&self) -> Idx<TableData> {
        self.root_table
    }

    fn expr_kinds(&self) -> &FxHashMap<TextRange, ExpressionKind> {
        &self.expr_kinds
    }

    fn name_kinds(&self) -> &FxHashMap<TextRange, SymbolId> {
        &self.name_kinds
    }

    fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl<'db> Collector<'db> {
    pub fn symbol_from_source_file(db: &'db dyn Db, file: File, node: SourceFile) -> SourceSymbol {
        let mut arena = SourceArena::default();
        // Source table is not always the root table, it depends on which entity
        // was the script executed. script_execute and non-edict entities execute stuff
        // in the root while edict entities with 'vscripts' keyvalue will have their
        // script scope as the execution context
        // This should also drive whether 'self' is present in the scope
        // TODO: Get source file's jsdoc and determine
        let source_table = arena.alloc(TableData::default());
        let container = Container::Table(TableId::new(file, source_table));
        let root_table = arena.alloc(TableData::default());
        let const_table = arena.alloc(TableData::default());
        let scope = arena.alloc(Scope {
            locals: SymbolTable::default(),
            parent: None,
            container,
            execution_range: node.syntax().text_range(),
            range: node.syntax().text_range(),
        });

        let mut imports = FxHashMap::default();
        let mut libs = Vec::new();
        if let Some(squirrel_lib) = db.squirrel_lib() {
            if squirrel_lib != file {
                libs.push(squirrel_lib);
            }
        }
        if let Some(vscript_lib) = db.vscript_lib() {
            if vscript_lib != file {
                libs.push(vscript_lib);
            }
        }
        if !libs.is_empty() {
            imports.insert(Container::Table(TableId::new(file, root_table)), libs);
        }

        let mut collector = Self {
            db,
            file,
            imports,
            scope,
            container,
            special_container: None,
            can_break: false,
            can_continue: false,
            dead_code: false,
            arena,
            const_table,
            root_table,
            execution_range: node.syntax().text_range(),
            function: None,
            deferred_functions: VecDeque::new(),
            expr_kinds: FxHashMap::default(),
            name_kinds: FxHashMap::default(),
            diagnostics: Vec::new(),
        };

        for stmt in node.statements() {
            collector.collect_stmt(&stmt);
        }

        assert_eq!(collector.arena[collector.scope].parent, None);

        while let Some(func) = collector.deferred_functions.pop_front() {
            collector.execution_range = func.execution_range;
            collector.function = Some(func.idx);
            collector.scope = func.scope;
            collector.container = collector.arena[func.idx]
                .container
                .unwrap_or(collector.arena[func.scope].container);
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

        collector.unused_variables_diagnostics();

        SourceSymbol {
            imports: collector.imports,
            arena: collector.arena,
            const_table,
            root_table,
            source_table,
            expr_kinds: collector.expr_kinds,
            name_kinds: collector.name_kinds,
            diagnostics: collector.diagnostics,
        }
    }

    pub fn get<T>(&self, id: T) -> &T::Data
    where
        T: ArenaId,
        SourceArena: std::ops::Index<Idx<T::Data>, Output = T::Data>,
    {
        if id.file() != self.file {
            return id.get_data(self.db);
        }

        &self.arena[id.idx()]
    }

    pub fn get_mut<T>(&mut self, id: T) -> Option<&mut T::Data>
    where
        T: ArenaId,
        SourceArena: std::ops::IndexMut<Idx<T::Data>, Output = T::Data>,
    {
        if id.file() != self.file {
            return None;
        }

        Some(&mut self.arena[id.idx()])
    }

    pub fn db(&self) -> &dyn Db {
        self.db
    }

    pub fn file(&self) -> File {
        self.file
    }

    pub fn scope(&self) -> ScopeId {
        self.scope
    }

    fn symbol(&mut self, symbol: Symbol) -> SymbolId {
        let name_range = symbol.name_range;
        let id = SymbolId::new(self.file, self.arena.alloc(symbol));
        self.name_kinds.insert(name_range, id);
        id
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
                        ..Default::default()
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
        &mut self.arena[self.scope]
    }

    fn enter_scope(&mut self, range: TextRange) {
        self.scope = self.arena.alloc(Scope {
            container: self.container,
            parent: Some(self.scope),
            locals: SymbolTable::default(),
            range,
            execution_range: self.execution_range,
        });
    }

    fn exit_scope(&mut self) {
        self.dead_code = false;
        self.scope = self.arena[self.scope].parent.unwrap();
    }

    fn execution_container(&self) -> Container {
        self.arena[self.scope].container
    }

    fn local_members(&self, offset: TextSize) -> SymbolTable {
        self.state().local_members(offset)
    }

    fn state(&self) -> FileState<'_> {
        FileState::InProcess(self)
    }

    fn add_current_container_member(&mut self, name: String, symbol: SymbolId) {
        self.add_container_member(self.container, name, symbol);
    }

    fn add_container_member(&mut self, container: Container, name: String, symbol: SymbolId) {
        match container {
            Container::Table(id) => {
                if let Some(t) = self.get_mut(id) {
                    t.members.insert(name, symbol);
                }
            }
            Container::Class(id) => {
                if let Some(c) = self.get_mut(id) {
                    c.members.insert(name, symbol);
                }
            }
            Container::Enum(id) => {
                if let Some(e) = self.get_mut(id) {
                    e.members.insert(name, symbol);
                }
            }
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
                    let Some((name, text)) = get_name(&var) else {
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
                                    ..Default::default()
                                });
                                params_state = ParamsState::NoDefault;
                            }
                            ParamsState::VarArgs(_) => {
                                self.diagnostics.push(Diagnostic {
                                    message: "Parameters cannot be preceded by varied arguments"
                                        .to_owned(),
                                    range: var.syntax().text_range(),
                                    ..Default::default()
                                });
                                params_state = ParamsState::NoDefault;
                            }
                            ParamsState::NoDefault => {}
                        }

                        let symbol = self.symbol(Symbol {
                            name: text.clone(),
                            typ: Type::Unknown,
                            kind: SymbolKind::Local,
                            name_range: name.syntax().text_range(),
                            range: var.syntax().text_range(),
                        });

                        self.current_scope().locals.insert(text, symbol);
                        self.arena[function].params.push(symbol);
                        continue;
                    };

                    let typ = self.expr_type(&expr);

                    let symbol = self.symbol(Symbol {
                        name: text.clone(),
                        typ,
                        kind: SymbolKind::Local,
                        name_range: name.syntax().text_range(),
                        range: var.syntax().text_range(),
                    });

                    self.current_scope().locals.insert(text, symbol);
                    self.arena[function].params.push(symbol);
                    match params_state {
                        ParamsState::NoDefault => {
                            params_state = ParamsState::Default(count as u32);
                        }
                        ParamsState::Default(_) => {}
                        ParamsState::VarArgs(var_args_at) => {
                            self.diagnostics.push(Diagnostic {
                                message: "Parameters cannot be preceded by varied arguments"
                                    .to_owned(),
                                range: var.syntax().text_range(),
                                ..Default::default()
                            });
                            params_state = ParamsState::Default(var_args_at);
                        }
                    }
                }
                Parameter::Ellipsis(var_args) => match params_state {
                    ParamsState::NoDefault => params_state = ParamsState::VarArgs(count as u32),
                    ParamsState::Default(_) => {
                        self.diagnostics.push(Diagnostic {
                            message:
                                "Function with varied arguments cannot have default parameters"
                                    .to_owned(),
                            range: var_args.syntax().text_range(),
                            ..Default::default()
                        });
                    }
                    ParamsState::VarArgs(_) => {
                        self.diagnostics.push(Diagnostic {
                            message: "There can't be 2 varied arguments in a function signature"
                                .to_owned(),
                            range: var_args.syntax().text_range(),
                            ..Default::default()
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
                                ..Default::default()
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
                                ..Default::default()
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
                let Some(&member) = class.members.get(metamethod) else {
                    match errors {
                        MetamethodErrors::Yes { keyword }
                        | MetamethodErrors::YesBinary { keyword, .. } => {
                            self.diagnostics.push(Diagnostic {
                                message: format!("'instance' does not support {keyword}: class has no '{metamethod}' metamethod"),
                                range: error_range,
                                ..Default::default()
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
                            ..Default::default()
                        });
                    }
                    MetamethodErrors::YesBinary {
                        keyword,
                        right: left,
                    } => {
                        if left != Type::Unknown {
                            self.diagnostics.push(Diagnostic {
                                message: format!(
                                    "'{typ}' does not support {keyword} with '{left}'"
                                ),
                                range: error_range,
                                ..Default::default()
                            });
                        }
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
            Type::Class(id) => Some(Container::Class(id)),
            Type::Table(id) => {
                self.call_metamethod(
                    typ,
                    "_newslot",
                    arguments,
                    error_range,
                    MetamethodErrors::No,
                );
                Some(Container::Table(id))
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
        let left_type = self.state().expr_to_type(left);
        let right_type = self.state().expr_to_type(right);
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
                    (Type::Integer(_), Type::Integer(_)) => return Some(Type::Integer(None)),
                    (Type::Float(_) | Type::Integer(_), Type::Float(_) | Type::Integer(_)) => {
                        return Some(Type::Float(None));
                    }
                    _ => {}
                }
            }
            (Type::Integer(_), Type::Integer(_)) => return Some(Type::Integer(None)),
            (Type::Float(_) | Type::Integer(_), Type::Float(_) | Type::Integer(_)) => {
                return Some(Type::Float(None));
            }
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

    fn call_iter(&mut self, iterable: Type, error_range: TextRange) -> Option<(Type, Type)> {
        match iterable {
            Type::Table(_) => {
                let arguments = vec![Some(ExpressionKind::Literal(Type::Null))];
                self.call_metamethod(
                    iterable,
                    "_nexti",
                    &arguments,
                    error_range,
                    MetamethodErrors::No,
                );
                Some((Type::Unknown, Type::Unknown))
            }
            Type::Array(id) => {
                let typ = self.get(id).typ;
                Some((Type::Integer(None), typ))
            }
            Type::Generator(id) => {
                let typ = self.get(id).yielding.unwrap_or(Type::Unknown);
                Some((Type::Integer(None), typ))
            }
            Type::Class(_) => Some((Type::Unknown, Type::Unknown)),
            _ => {
                let arguments = vec![Some(ExpressionKind::Literal(Type::Null))];
                match self.call_metamethod(
                    iterable,
                    "_nexti",
                    &arguments,
                    error_range,
                    MetamethodErrors::Yes {
                        keyword: "iterating",
                    },
                ) {
                    // Shouldn't be Unknown, instead look in the function signature? But who cares about those niche cases
                    Some(typ) => Some((Type::Unknown, typ)),
                    None => None,
                }
            }
        }
    }

    fn check_constant(&mut self, value: NullableExprKind, error_range: TextRange) {
        match value {
            Some(ExpressionKind::Literal(Type::Integer(Some(_)))) => {}
            Some(ExpressionKind::Literal(Type::Float(Some(_)))) => {}
            Some(ExpressionKind::Literal(Type::String(Some(_)))) => {}
            Some(ExpressionKind::Literal(Type::Boolean(Some(_)))) => {}
            _ => {
                self.diagnostics.push(Diagnostic {
                    message:
                        "Constant can only hold value of 'integer', 'float', 'string' or 'bool'"
                            .to_owned(),
                    range: error_range,
                    ..Default::default()
                });
            }
        }
    }

    fn collect_property(&mut self, property: &Property, default_value: Option<i32>) {
        let is_enum = default_value.is_some();
        let value = match property.value() {
            Some(expr) => {
                let value = self.collect_expr(&expr);
                if is_enum {
                    self.check_constant(value, expr.syntax().text_range());
                }
                value
            }
            None => {
                if let Some(value) = default_value {
                    Some(ExpressionKind::Literal(Type::Integer(Some(value))))
                } else {
                    None
                }
            }
        };

        let Some((name_range, text)) = (match &property.name() {
            Some(MemberName::Identifier(name)) => {
                if let Some(text) = name.name().and_then(|n| n.text()) {
                    Some((name.syntax().text_range(), text))
                } else {
                    None
                }
            }
            Some(MemberName::String(name)) => {
                if let Some(token) = name
                    .token()
                    .map(|(_kind, token)| unquote_string(token.text()))
                {
                    Some((name.syntax().text_range(), token))
                } else {
                    None
                }
            }
            Some(MemberName::Computed(name)) => {
                if let Some(expr) = name.expression() {
                    let kind = self.expr_type(&expr);
                    if let Type::String(Some(id)) = kind {
                        Some((expr.syntax().text_range(), self.get(id).to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        }) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ: self.state().expr_to_type(value),
            kind: if is_enum {
                SymbolKind::EnumMember
            } else {
                SymbolKind::Property
            },
            name_range,
            range: property.syntax().text_range(),
        });

        self.add_current_container_member(text, symbol);
    }

    fn try_setting_container<T>(&mut self, idx: Idx<FunctionData>, node: &T)
    where
        T: IsFunction,
    {
        if let Some(env) = node.environment().and_then(|e| e.expression()) {
            let typ = self.expr_type(&env);
            if let Ok(container) = Container::try_from(typ) {
                self.arena[idx].container = Some(container);
                return;
            } else if typ != Type::Unknown {
                self.diagnostics.push(Diagnostic {
                    message: format!("Trying to use '{typ}' as function's environment"),
                    range: env.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                    ..Default::default()
                });
            }
        }

        if let Some(container) = self.special_container {
            self.arena[idx].container = Some(container);
        }
    }
    // Lambdas are processed differently
    fn collect_function<T>(&mut self, idx: Idx<FunctionData>, node: &T)
    where
        T: IsFunction,
    {
        self.try_setting_container(idx, node);

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
                scope: self.scope,
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
                let Some((name, text)) = get_name(method) else {
                    self.collect_function(function.idx(), method);
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: text.clone(),
                    typ: Type::Function(function),
                    kind: SymbolKind::Property,
                    name_range: name.syntax().text_range(),
                    range: method.syntax().text_range(),
                });
                self.add_current_container_member(text, symbol);

                self.collect_function(function.idx(), method);
            }
            Member::Constructor(constructor) => {
                let function = self.function();
                let Some(keyword) = constructor.constructor_keyword() else {
                    self.collect_function(function.idx(), constructor);
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: "constructor".to_owned(),
                    typ: Type::Function(function),
                    kind: SymbolKind::Property,
                    name_range: keyword.text_range(),
                    range: constructor.syntax().text_range(),
                });

                self.add_current_container_member("constructor".to_owned(), symbol);

                self.collect_function(function.idx(), constructor);
            }
        }
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        if self.dead_code {
            self.diagnostics.push(Diagnostic {
                message: "Unreachable statement".to_owned(),
                range: stmt.syntax().text_range(),
                severity: DiagnosticSeverity::Unnecessary,
            });
        }

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
            Stmt::Continue(stmt) => self.continue_statement(stmt),
            Stmt::Break(stmt) => self.break_statement(stmt),
            Stmt::Try(stmt) => self.try_statement(stmt),
            Stmt::Throw(stmt) => self.throw_statement(stmt),
        }
    }

    fn local_variable(&mut self, decl: &LocalVariableDeclaration) {
        for var in decl.declarations() {
            let Some((name, text)) = get_name(&var) else {
                let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                    continue;
                };

                self.collect_expr(&expr);
                continue;
            };

            let Some(expr) = var.initialiser().and_then(|i| i.expression()) else {
                let id = self.symbol(Symbol {
                    name: text.clone(),
                    typ: Type::Null,
                    kind: SymbolKind::Local,
                    name_range: name.syntax().text_range(),
                    range: var.syntax().text_range(),
                });

                self.current_scope().locals.insert(text, id);
                continue;
            };

            let typ = self.expr_type(&expr);
            let id = self.symbol(Symbol {
                name: text.clone(),
                typ,
                kind: SymbolKind::Local,
                name_range: name.syntax().text_range(),
                range: var.syntax().text_range(),
            });

            self.current_scope().locals.insert(text, id);
        }
    }

    fn local_function(&mut self, decl: &LocalFunctionDeclaration) {
        let id = self.function();
        if let Some((name, text)) = get_name(decl) {
            let symbol = self.symbol(Symbol {
                name: text.clone(),
                typ: Type::Function(id),
                kind: SymbolKind::Local,
                name_range: name.syntax().text_range(),
                range: decl.syntax().text_range(),
            });

            self.current_scope().locals.insert(text, symbol);
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
        let value = match stmt.value().and_then(|v| v.expression()) {
            Some(expr) => {
                let value = self.collect_expr(&expr);
                self.check_constant(value, expr.syntax().text_range());
                value
            }
            None => None,
        };

        let Some((name, text)) = get_name(stmt) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ: self.state().expr_to_type(value),
            kind: SymbolKind::Constant,
            name_range: name.syntax().text_range(),
            range: stmt.syntax().text_range(),
        });
        self.arena[self.const_table].members.insert(text, symbol);
    }

    fn for_each_statement(&mut self, stmt: &ForEachStatement) {
        let save_break_continue = (self.can_break, self.can_continue);
        self.can_break = true;
        self.can_continue = true;
        if let Some(body) = stmt.body() {
            self.enter_scope(body.syntax().text_range());
        } else {
            self.enter_scope(TextRange::empty(stmt.syntax().text_range().end()));
        }

        let (key_type, value_type) = match stmt.iterable() {
            Some(iterable) => {
                let typ = self.expr_type(&iterable);
                self.call_iter(typ, iterable.syntax().text_range())
                    .unwrap_or((Type::Unknown, Type::Unknown))
            }
            None => (Type::Unknown, Type::Unknown),
        };

        if let Some(key) = stmt.key() {
            if let Some((name, text)) = get_name(&key) {
                let symbol = self.symbol(Symbol {
                    name: text.clone(),
                    typ: key_type,
                    kind: SymbolKind::Local,
                    name_range: name.syntax().text_range(),
                    range: key.syntax().text_range(),
                });

                self.current_scope().locals.insert(text, symbol);
            }
        }

        if let Some(value) = stmt.value() {
            if let Some((name, text)) = get_name(&value) {
                let symbol = self.symbol(Symbol {
                    name: text.clone(),
                    typ: value_type,
                    kind: SymbolKind::Local,
                    name_range: name.syntax().text_range(),
                    range: value.syntax().text_range(),
                });

                self.current_scope().locals.insert(text, symbol);
            }
        }

        if let Some(body) = stmt.body() {
            self.collect_stmt(&body);
        }

        self.exit_scope();
        (self.can_break, self.can_continue) = save_break_continue;
    }

    fn for_statement(&mut self, stmt: &ForStatement) {
        let save_break_continue = (self.can_break, self.can_continue);
        self.can_break = true;
        self.can_continue = true;
        self.enter_scope(stmt.syntax().text_range());
        match stmt.initialiser().and_then(|i| i.kind()) {
            Some(ForInitialiserKind::LocalVariableDeclaration(decl)) => self.local_variable(&decl),
            Some(ForInitialiserKind::LocalFunctionDeclaration(decl)) => self.local_function(&decl),
            Some(ForInitialiserKind::Expression(expr)) => {
                self.collect_expr(&expr);
            }
            None => {}
        }
        if let Some(condition) = stmt.condition().and_then(|c| c.expression()) {
            self.collect_expr(&condition);
        }
        if let Some(increment) = stmt.condition().and_then(|i| i.expression()) {
            self.collect_expr(&increment);
        }
        if let Some(body) = stmt.body() {
            self.collect_stmt(&body);
        }
        self.exit_scope();
        (self.can_break, self.can_continue) = save_break_continue;
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
        self.container = Container::Class(class);
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

        let mut names: Vec<_> = qualified_name.names().collect();

        let Some(final_name) = names.pop() else {
            self.collect_function(function.idx(), stmt);
            return;
        };

        let Some(final_text) = final_name.text() else {
            self.collect_function(function.idx(), stmt);
            return;
        };

        if names.is_empty() {
            // Plain `function abc()`: declare in current container
            let symbol = self.symbol(Symbol {
                name: final_text.clone(),
                typ: Type::Function(function),
                kind: SymbolKind::Property,
                name_range: final_name.syntax().text_range(),
                range: stmt.syntax().text_range(),
            });
            self.add_current_container_member(final_text, symbol);

            self.collect_function(function.idx(), stmt);
            return;
        }

        let Some(text) = names[0].text() else {
            self.collect_function(function.idx(), stmt);
            return;
        };

        let offset = qualified_name.syntax().text_range().end();
        let state = self.state();

        let members = state
            .members_of_container(
                self.execution_container(),
                FindSymbol::BeforeIfInExecutionRange(offset),
                Some(GetMembers::Container(self.execution_container())),
            )
            .into_iter()
            .find_map(|(name, id)| if text == name { Some(id) } else { None });

        let root = || {
            state
                .members_of_table(
                    state.root_table(),
                    FindSymbol::BeforeIfInExecutionRange(offset),
                    Some(GetMembers::Container(self.execution_container())),
                )
                .into_iter()
                .find_map(|(name, id)| if text == name { Some(id) } else { None })
        };

        let Some(id) = members.or_else(root) else {
            if state
                .local_members(offset)
                .into_iter()
                .find(|(name, _)| &text == name)
                .is_some()
            {
                self.diagnostics.push(Diagnostic {
                    message: "Function statement does not lookup locals. Initial symbol not found"
                        .to_owned(),
                    range: names[0].syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                    ..Default::default()
                });
            }
            self.collect_function(function.idx(), stmt);
            return;
        };

        let mut typ = self.get(id).typ;
        let key = names[0].syntax().text_range();
        self.name_kinds.insert(key, id);

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

            let state = self.state();

            let Some(id) = state
                .members_of_container(
                    container,
                    FindSymbol::BeforeIfInExecutionRange(offset),
                    Some(GetMembers::Container(container)),
                )
                .into_iter()
                .find_map(|(name, id)| if text == name { Some(id) } else { None })
            else {
                self.collect_function(function.idx(), stmt);
                return;
            };

            typ = self.get(id).typ;
            let key = segment.syntax().text_range();
            self.name_kinds.insert(key, id);
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
            name: final_text.clone(),
            typ: Type::Function(function),
            kind: SymbolKind::Property,
            name_range: final_name.syntax().text_range(),
            range: stmt.syntax().text_range(),
        });

        self.add_container_member(container, final_text, symbol);

        let save_container = self.special_container;
        self.special_container = Some(container);
        self.collect_function(function.idx(), stmt);
        self.special_container = save_container;
    }

    fn enum_statement(&mut self, stmt: &EnumStatement) {
        let enum_ = EnumId::new(self.file, self.arena.alloc(EnumData::default()));

        if let Some((name, text)) = get_name(stmt) {
            let symbol = self.symbol(Symbol {
                name: text.clone(),
                typ: Type::Enum(enum_),
                kind: SymbolKind::Enum,
                name_range: name.syntax().text_range(),
                range: stmt.syntax().text_range(),
            });

            self.arena[self.const_table].members.insert(text, symbol);
        }

        let save_symbol = self.container;
        self.container = Container::Enum(enum_);
        for (value, property) in stmt.members().enumerate() {
            self.collect_property(&property, Some(value as i32));
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
            let save_break_continue = (self.can_break, self.can_continue);
            self.can_break = true;
            self.can_continue = true;
            self.enter_scope(body.syntax().text_range());
            self.collect_stmt(&body);
            self.exit_scope();
            (self.can_break, self.can_continue) = save_break_continue;
        }
    }

    fn do_while_statement(&mut self, stmt: &DoWhileStatement) {
        if let Some(body) = stmt.body() {
            let save_break_continue = (self.can_break, self.can_continue);
            self.can_break = true;
            self.can_continue = true;
            self.enter_scope(body.syntax().text_range());
            self.collect_stmt(&body);
            self.exit_scope();
            (self.can_break, self.can_continue) = save_break_continue;
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

        let save_break = self.can_break;
        self.can_break = true;
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
                                | (
                                    Type::Integer(_) | Type::Float(_),
                                    Type::Integer(_) | Type::Float(_),
                                )
                                | (Type::String(_), Type::String(_))
                                | (Type::Boolean(_), Type::Boolean(_))
                        ) {
                            self.diagnostics.push(Diagnostic {
                                message: format!("Case of type '{case_kind}' is incompitable with discriminant of type '{kind}'"),
                                range: test.syntax().text_range(),
                                severity: DiagnosticSeverity::Warning,
                                ..Default::default()
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
        self.can_break = save_break;
    }

    fn return_statement(&mut self, stmt: &ReturnStatement) {
        let kind = if let Some(value) = stmt.value() {
            Some(self.expr_type(&value))
        } else {
            None
        };

        self.dead_code = true;

        let Some(function) = self.function else {
            if kind.is_some() {
                self.diagnostics.push(Diagnostic {
                    message: "Value returned by the source file execution scope cannot be received in any way".to_owned(),
                    range: stmt.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                    ..Default::default()
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
                ..Default::default()
            });
            return;
        };

        if self.arena[function].yielding.is_none() {
            self.arena[function].yielding = Some(kind);
        }
    }

    fn continue_statement(&mut self, stmt: &ContinueStatement) {
        if !self.can_continue {
            self.diagnostics.push(Diagnostic {
                message: "'continue' has to be in a loop block".to_owned(),
                range: stmt.syntax().text_range(),
                ..Default::default()
            })
        }
        self.dead_code = true;
    }

    fn break_statement(&mut self, stmt: &BreakStatement) {
        if !self.can_break {
            self.diagnostics.push(Diagnostic {
                message: "'break' has to be in a loop or 'switch' block".to_owned(),
                range: stmt.syntax().text_range(),
                ..Default::default()
            })
        }
        self.dead_code = true;
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
            if let Some((name, text)) = get_name(&binding) {
                let symbol = self.symbol(Symbol {
                    typ: Type::String(None),
                    name: text.clone(),
                    kind: SymbolKind::Local,
                    name_range: name.syntax().text_range(),
                    range: binding.syntax().text_range(),
                });

                self.current_scope().locals.insert(text, symbol);
            };
        }

        if let Some(body) = catch.body() {
            self.collect_stmt(&body);
        }

        self.exit_scope();
    }

    fn throw_statement(&mut self, stmt: &ThrowStatement) {
        // mark current function as exception throwing
        let kind = if let Some(value) = stmt.value() {
            self.expr_type(&value)
        } else {
            Type::Unknown
        };

        self.dead_code = true;
        let Some(function) = self.function else {
            return;
        };

        if self.arena[function].throwing.is_none() {
            self.arena[function].throwing = Some(kind);
        }
    }

    fn expr_type(&mut self, expr: &Expr) -> Type {
        let kind = self.collect_expr(expr);
        self.state().expr_to_type(kind)
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
            Expr::Line(_) => Some(ExpressionKind::Literal(Type::Integer(None))),
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
            LiteralExpressionKind::DecimalInteger => {
                let text = token.text();

                if text.starts_with('0') && text.len() > 1 {
                    self.diagnostics.push(Diagnostic {
                        message: "Leading '0' can be removed".to_owned(),
                        range: token.text_range(),
                        severity: DiagnosticSeverity::Warning,
                        ..Default::default()
                    });
                }
                // Default values are provided to signify that the user has tried
                // to write a literal but the literal was malformed
                // This is to not error out
                let value = text.parse::<i32>().unwrap_or(0);

                ExpressionKind::Literal(Type::Integer(Some(value)))
            }
            LiteralExpressionKind::OctalInteger => {
                let text = token.text();
                // 0321321
                let value = i32::from_str_radix(&text[1..], 8).unwrap_or(0);

                ExpressionKind::Literal(Type::Integer(Some(value)))
            }
            LiteralExpressionKind::HexInteger => {
                let text = token.text();
                //0x12312312
                let value = i32::from_str_radix(&text[2..], 16).unwrap_or(0);

                ExpressionKind::Literal(Type::Integer(Some(value)))
            }
            LiteralExpressionKind::Character => {
                // let text = token.text();
                // let inner = &text[1..];

                // let value = if !inner.starts_with('\\') {
                //     inner.chars().next().map(|c| c as i32)
                // } else {
                //     match inner.chars().nth(1) {
                //         Some('n') => Some('\n' as i32),
                //         Some('t') => Some('\t' as i32),
                //         Some('r') => Some('\r' as i32),
                //         Some('\\') => Some('\\' as i32),
                //         Some('\'') => Some('\'' as i32),

                //         Some('x') => {
                //             let hex = &inner[2..];
                //             u8::from_str_radix(hex, 16).ok().map(|c| c as i32)
                //         }

                //         Some(other) => panic!("unknown escape: {}", other),
                //         None => None,
                //     }
                // }
                // .unwrap_or(0);

                ExpressionKind::Literal(Type::Integer(Some(0)))
            }
            LiteralExpressionKind::Float => {
                let text = token.text();
                let value = text.parse::<f32>().unwrap_or(0.0);

                ExpressionKind::Literal(Type::Float(Some(value)))
            }
            LiteralExpressionKind::String | LiteralExpressionKind::VerbatimString => {
                let string = StringId::new(
                    self.file,
                    self.arena
                        .alloc(unquote_string(token.text()).into_boxed_str()),
                );

                ExpressionKind::Literal(Type::String(Some(string)))
            }
            LiteralExpressionKind::Null => ExpressionKind::Literal(Type::Null),
            LiteralExpressionKind::True => ExpressionKind::Literal(Type::Boolean(Some(true))),
            LiteralExpressionKind::False => ExpressionKind::Literal(Type::Boolean(Some(false))),
        })
    }

    fn table_literal_expression(&mut self, expr: &TableLiteralExpression) -> ExpressionKind {
        let table = TableId::new(self.file, self.arena.alloc(TableData::default()));
        let save_symbol = self.container;
        self.container = Container::Table(table);
        for member in expr.members() {
            self.collect_member(&member);
        }
        self.container = save_symbol;

        ExpressionKind::Literal(Type::Table(table))
    }

    fn class_expression(&mut self, expr: &ClassExpression) -> ExpressionKind {
        let class = self.class(expr);

        let save_symbol = self.container;
        self.container = Container::Class(class);
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

    fn name_expression(&mut self, expr: &Name) -> NullableExprKind {
        let text = expr.text()?;
        let offset = expr.syntax().text_range().end();
        let filter = |(name, id)| {
            if name == text { Some(id) } else { None }
        };

        let locals = self.local_members(offset).into_iter().find_map(filter);

        let state = self.state();

        let consts = || {
            state
                .members_of_table(
                    state.const_table(),
                    FindSymbol::OnlyBefore(offset),
                    Some(GetMembers::Const),
                )
                .into_iter()
                .find_map(filter)
        };

        let members = || {
            state
                .members_of_container(
                    self.execution_container(),
                    FindSymbol::BeforeIfInExecutionRange(offset),
                    Some(GetMembers::Container(self.execution_container())),
                )
                .into_iter()
                .find_map(filter)
        };

        let root = || {
            state
                .members_of_table(
                    state.root_table(),
                    FindSymbol::BeforeIfInExecutionRange(offset),
                    Some(GetMembers::Root),
                )
                .into_iter()
                .find_map(filter)
        };

        locals
            .or_else(consts)
            .or_else(members)
            .or_else(root)
            .map(|id| {
                self.name_kinds.insert(expr.syntax().text_range(), id);
                if matches!(self.get(id).typ, Type::Enum(_))
                    && expr
                        .syntax()
                        .parent()
                        .map(|p| !ast::MemberAccessExpression::can_cast(p.kind()))
                        .unwrap_or(false)
                {
                    self.diagnostics.push(Diagnostic {
                        message: "'enum' can only appear in property access expression".to_owned(),
                        range: expr.syntax().text_range(),
                        ..Default::default()
                    })
                }
                ExpressionKind::Symbol(id)
            })
    }

    fn this_expression(&self, _expr: &ThisExpression) -> ExpressionKind {
        ExpressionKind::Literal(self.execution_container().into())
    }

    fn root_access_expression(&mut self, expr: &RootAccessExpression) -> NullableExprKind {
        let (name_node, text) = get_name(expr)?;
        let offset = expr.syntax().text_range().end();
        let state = self.state();

        state
            .members_of_table(
                state.root_table(),
                FindSymbol::BeforeIfInExecutionRange(offset),
                Some(GetMembers::Root),
            )
            .into_iter()
            .find_map(|(name, id)| {
                if name == text {
                    self.name_kinds.insert(name_node.syntax().text_range(), id);
                    Some(ExpressionKind::Symbol(id))
                } else {
                    None
                }
            })
    }

    fn base_expression(&mut self, expr: &BaseExpression) -> ExpressionKind {
        match self.execution_container() {
            Container::Class(id) => {
                let class = self.get(id);
                if let Some(inherits) = class.inherits {
                    ExpressionKind::Literal(Type::Class(inherits))
                } else {
                    self.diagnostics.push(Diagnostic {
                        message: "Accessing 'base' in a class that doesn't have a superclass"
                            .to_owned(),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Warning,
                        ..Default::default()
                    });
                    ExpressionKind::Literal(Type::Null)
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: "Accessing 'base' inside non-class execution scope".to_owned(),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                    ..Default::default()
                });
                ExpressionKind::Literal(Type::Null)
            }
        }
    }

    fn member_access_expression(&mut self, expr: &MemberAccessExpression) -> NullableExprKind {
        let obj = self.expr_type(&expr.object()?);
        let (name_node, text) = get_name(&expr.member_part()?)?;

        let offset = expr.syntax().text_range().end();

        let result = self
            .state()
            .members_of_type(obj, FindSymbol::BeforeIfInExecutionRange(offset))
            .into_iter()
            .find_map(|(name, id)| {
                if name == text {
                    self.name_kinds.insert(name_node.syntax().text_range(), id);
                    Some(ExpressionKind::Symbol(id))
                } else {
                    None
                }
            });

        if result.is_none()
            && !matches!(
                obj,
                Type::Table(_) | Type::Class(_) | Type::Instance(_) | Type::Unknown
            )
        {
            self.diagnostics.push(Diagnostic {
                message: format!("'{obj}' has no member named '{text}'"),
                range: expr.syntax().text_range(),
                ..Default::default()
            });
        }
        result
    }

    fn element_access_expression(&mut self, expr: &ElementAccessExpression) -> NullableExprKind {
        let obj = self.expr_type(&expr.object()?);
        let index = expr.index()?.expression()?;
        let Type::String(Some(id)) = self.expr_type(&index) else {
            return None;
        };

        let text = self.get(id).to_string();
        let offset = expr.syntax().text_range().end();

        let result = self
            .state()
            .members_of_type(obj, FindSymbol::BeforeIfInExecutionRange(offset))
            .into_iter()
            .find_map(|(name, id)| {
                if name == text {
                    self.name_kinds.insert(index.syntax().text_range(), id);
                    Some(ExpressionKind::Symbol(id))
                } else {
                    None
                }
            });

        if result.is_none() && !matches!(obj, Type::Table(_) | Type::Class(_) | Type::Instance(_)) {
            self.diagnostics.push(Diagnostic {
                message: format!("'{obj}' has no member named '{text}'"),
                range: expr.syntax().text_range(),
                ..Default::default()
            });
        }

        result
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
                                ..Default::default()
                            });
                        }
                        continue;
                    };

                    match (
                        self.state().expr_to_type(argument_kind),
                        self.get(param).typ,
                    ) {
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

                        (Type::Integer(_) | Type::Float(_), Type::Integer(_) | Type::Float(_)) => {}

                        (passed, required) => {
                            if discriminant(&passed) != discriminant(&required) {
                                self.diagnostics.push(Diagnostic {
                                    message: format!("Expected parameter of type '{required}', but got '{passed}'"),
                                    range: error_range,
                                    severity: DiagnosticSeverity::Warning,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }

                let enough_parameters = match self.get(id).params_state {
                    ParamsState::NoDefault => arguments.len() == self.get(id).params.len(),
                    ParamsState::Default(from) | ParamsState::VarArgs(from) => {
                        arguments.len() as u32 >= from
                    }
                };

                if !enough_parameters {
                    self.diagnostics.push(Diagnostic {
                        message: "Insufficient number of parameters passed".to_owned(),
                        range: error_range,
                        ..Default::default()
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
                if let Some(&symbol) = class.members.get("constructor") {
                    self.call_type(self.get(symbol).typ, arguments, error_range);
                } else if arguments.len() != 0 {
                    self.diagnostics.push(Diagnostic {
                        message: "Default constructor should have no parameters".to_owned(),
                        range: error_range,
                        ..Default::default()
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
        let obj = match expr.callee().and_then(|c| c.expression()) {
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

                let filter = |(name, id)| {
                    if name == text { Some(id) } else { None }
                };

                let locals = self.local_members(offset).into_iter().find_map(filter);

                if let Some(symbol) = locals {
                    self.name_kinds.insert(expr.syntax().text_range(), symbol);
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: None,
                        symbol,
                        range: expr.syntax().text_range(),
                    });
                }

                let consts = self
                    .state()
                    .members_of_table(
                        self.state().const_table(),
                        FindSymbol::OnlyBefore(offset),
                        Some(GetMembers::Const),
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = consts {
                    self.name_kinds.insert(expr.syntax().text_range(), symbol);
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(Type::Table(self.state().const_table())),
                        symbol,
                        range: expr.syntax().text_range(),
                    });
                }

                let members = self
                    .state()
                    .members_of_container(
                        self.execution_container(),
                        FindSymbol::BeforeIfInExecutionRange(offset),
                        Some(GetMembers::Container(self.execution_container())),
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = members {
                    self.name_kinds.insert(expr.syntax().text_range(), symbol);
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(self.arena[self.scope].container.into()),
                        symbol,
                        range: expr.syntax().text_range(),
                    });
                }

                let root = self
                    .state()
                    .members_of_table(
                        self.state().root_table(),
                        FindSymbol::BeforeIfInExecutionRange(offset),
                        Some(GetMembers::Root),
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = root {
                    self.name_kinds.insert(expr.syntax().text_range(), symbol);
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(Type::Table(self.state().root_table())),
                        symbol,
                        range: expr.syntax().text_range(),
                    });
                }

                Some(AssignmentLeftHandSide::CanCreate {
                    parent: self.container.into(),
                    new_key: text.into_boxed_str(),
                    name_range: expr.syntax().text_range(),
                    range: expr.syntax().text_range(),
                })
            }
            Expr::MemberAccess(expr) => {
                let obj = self.expr_type(&expr.object()?);
                let member_part = expr.member_part()?;

                let (name, text) = get_name(&member_part)?;
                let offset = expr.syntax().text_range().end();

                Some(
                    self.state()
                        .members_of_type(obj, FindSymbol::BeforeIfInExecutionRange(offset))
                        .into_iter()
                        .find_map(|(name, id)| if name == text { Some(id) } else { None })
                        .map_or_else(
                            || AssignmentLeftHandSide::CanCreate {
                                parent: obj,
                                new_key: text.into_boxed_str(),
                                name_range: name.syntax().text_range(),
                                range: expr.syntax().text_range(),
                            },
                            |id| {
                                self.name_kinds.insert(name.syntax().text_range(), id);
                                AssignmentLeftHandSide::Exists {
                                    parent: Some(obj),
                                    symbol: id,
                                    range: expr.syntax().text_range(),
                                }
                            },
                        ),
                )
            }
            Expr::ElementAccess(expr) => {
                let obj = self.expr_type(&expr.object()?);
                let index = expr.index()?.expression()?;
                let index_kind = self.collect_expr(&index);
                let Type::String(Some(id)) = self.state().expr_to_type(index_kind) else {
                    return Some(AssignmentLeftHandSide::NonStringKey {
                        parent: obj,
                        range: expr.syntax().text_range(),
                        key: index_kind,
                    });
                };

                let text = self.get(id).to_string();
                let offset = expr.syntax().text_range().end();

                Some(
                    self.state()
                        .members_of_type(obj, FindSymbol::BeforeIfInExecutionRange(offset))
                        .into_iter()
                        .find_map(|(name, id)| if name == text { Some(id) } else { None })
                        .map_or_else(
                            || AssignmentLeftHandSide::CanCreate {
                                parent: obj,
                                new_key: text.into_boxed_str(),
                                name_range: index.syntax().text_range(),
                                range: expr.syntax().text_range(),
                            },
                            |id| {
                                self.name_kinds.insert(index.syntax().text_range(), id);
                                AssignmentLeftHandSide::Exists {
                                    parent: Some(obj),
                                    symbol: id,
                                    range: expr.syntax().text_range(),
                                }
                            },
                        ),
                )
            }
            Expr::RootAccess(expr) => {
                let (name, text) = get_name(expr)?;
                let offset = expr.syntax().text_range().end();
                let root_table = self.state().root_table();
                Some(
                    self.state()
                        .members_of_table(
                            root_table,
                            FindSymbol::BeforeIfInExecutionRange(offset),
                            Some(GetMembers::Root),
                        )
                        .into_iter()
                        .find_map(|(name, id)| if name == text { Some(id) } else { None })
                        .map_or_else(
                            || AssignmentLeftHandSide::CanCreate {
                                parent: Type::Table(root_table),
                                new_key: text.into_boxed_str(),
                                name_range: name.syntax().text_range(),
                                range: expr.syntax().text_range(),
                            },
                            |id| {
                                self.name_kinds.insert(name.syntax().text_range(), id);
                                AssignmentLeftHandSide::Exists {
                                    parent: Some(Type::Table(root_table)),
                                    symbol: id,
                                    range: expr.syntax().text_range(),
                                }
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

                let save_container = self.special_container;
                if let Some(container) = lhs_container(left_kind.as_ref()) {
                    self.special_container = Some(container);
                }
                let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
                self.special_container = save_container;

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
                range,
                name_range,
                new_key,
            }) => {
                let arguments = vec![
                    Some(ExpressionKind::Literal(Type::String(None))),
                    right_kind,
                ];

                let Some(container) = self.call_new_slot(parent, &arguments, range) else {
                    return right_kind;
                };

                let symbol = self.symbol(Symbol {
                    name: new_key.to_string(),
                    typ: self.state().expr_to_type(right_kind),
                    kind: SymbolKind::Property,
                    name_range,
                    range: expr_range,
                });

                self.add_container_member(container, new_key.into_string(), symbol);
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
                        ..Default::default()
                    });
                    return right_kind;
                }

                let typ = self.state().expr_to_type(right_kind);

                let Some(symbol) = self.get_mut(symbol) else {
                    return right_kind;
                };

                // Update symbol kind if it's null or unknown
                if matches!(symbol.typ, Type::Unknown | Type::Null)
                    && !matches!(typ, Type::Unknown | Type::Null)
                {
                    symbol.typ = typ;
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
        let save_container = self.special_container;
        if let Some(container) = lhs_container(left_kind.as_ref()) {
            self.special_container = Some(container);
        }
        let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
        self.special_container = save_container;

        self.do_new_slot(left_kind, right_kind, expr.syntax().text_range())
    }

    fn assign_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let left_kind = expr.lhs().and_then(|l| self.assignment_lhs(&l));

        let save_container = self.special_container;
        if let Some(container) = lhs_container(left_kind.as_ref()) {
            self.special_container = Some(container);
        }
        let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
        self.special_container = save_container;

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
                        ..Default::default()
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

                let typ = self.state().expr_to_type(right_kind);
                let Some(symbol) = self.get_mut(symbol) else {
                    return right_kind;
                };

                if symbol.typ.should_substitute() && !typ.should_substitute() {
                    symbol.typ = typ;
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
        let right = self.state().expr_to_type(right_kind);

        match right {
            Type::Array(_) => {
                let left = self.state().expr_to_type(left_kind);
                if !matches!(left, Type::Unknown | Type::Integer(_)) {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Trying to index into an array using '{left}' (only integers are applicable)"),
                        range: expr.syntax().text_range(),
                        severity: DiagnosticSeverity::Warning,
                        ..Default::default()
                    });
                }
            }
            Type::Table(_) | Type::Class(_) | Type::Instance(_) | Type::Unknown => {}
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: format!("Indexing into '{right}' will always return false"),
                    range: expr.syntax().text_range(),
                    severity: DiagnosticSeverity::Warning,
                    ..Default::default()
                });
            }
        }
        Some(ExpressionKind::Literal(Type::Boolean(None)))
    }

    fn instance_of_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let left = self.state().expr_to_type(left_kind);
        let right = self.state().expr_to_type(right_kind);

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
                    ..Default::default()
                });
            }
        }

        ExpressionKind::Literal(Type::Boolean(None))
    }

    fn equality_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (_left_kind, _right_kind) = self.extract_lhs_and_rhs(expr);
        ExpressionKind::Literal(Type::Boolean(None))
    }

    fn comparison_operator(
        &mut self,
        expr: &BinaryExpression,
        is_three_way: bool,
    ) -> ExpressionKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let left = self.state().expr_to_type(left_kind);
        let right = self.state().expr_to_type(right_kind);

        match (left, right) {
            (Type::Unknown | Type::Null, _)
            | (_, Type::Unknown | Type::Null)
            | (Type::Integer(_) | Type::Float(_), Type::Integer(_) | Type::Float(_))
            | (Type::String(_), Type::String(_))
            | (Type::Boolean(_), Type::Boolean(_)) => {}
            (Type::Table(_), Type::Table(_)) | (Type::Instance(_), Type::Instance(_)) => {
                let arguments = vec![right_kind];
                match self.call_metamethod(
                    left,
                    "_cmp",
                    &arguments,
                    expr.syntax().text_range(),
                    MetamethodErrors::No,
                ) {
                    Some(Type::Integer(_)) | Some(Type::Unknown) => {}
                    Some(_) => self.diagnostics.push(Diagnostic {
                        message: "'_cmp' must return an integer".to_owned(),
                        range: expr.syntax().text_range(),
                        ..Default::default()
                    }),
                    None => {
                        self.diagnostics.push(Diagnostic {
                            message: if matches!(left, Type::Table(_)) {
                                "Comparing classes with no '_cmp' delegate metamethod defined. The result is undetermenistic".to_owned()
                            } else {
                                "Comparing instances with no '_cmp' class metamethod defined. The result is undetermenistic".to_owned()
                            },
                            range: expr.syntax().text_range(),
                            severity: DiagnosticSeverity::Warning,
                            ..Default::default()
                        });
                    }
                }
            }
            _ => self.diagnostics.push(Diagnostic {
                message: format!("'{left}' does not support comparison with '{right}'"),
                range: expr.syntax().text_range(),
                ..Default::default()
            }),
        }

        ExpressionKind::Literal(if is_three_way {
            Type::Integer(None)
        } else {
            Type::Boolean(None)
        })
    }

    fn bitwise_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let left = self.state().expr_to_type(left_kind);
        let right = self.state().expr_to_type(right_kind);

        match (left, right) {
            (Type::Integer(_), Type::Integer(_)) => {}
            (Type::Unknown | Type::Null, Type::Unknown | Type::Null) => {}
            (Type::Integer(_), Type::Unknown | Type::Null) => {
                if let Some(ExpressionKind::Symbol(symbol)) = right_kind {
                    if let Some(symbol) = self.get_mut(symbol) {
                        symbol.typ = Type::Integer(None);
                    }
                }
            }
            (Type::Unknown | Type::Null, Type::Integer(_)) => {
                if let Some(ExpressionKind::Symbol(symbol)) = left_kind {
                    if let Some(symbol) = self.get_mut(symbol) {
                        symbol.typ = Type::Integer(None);
                    }
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic {
                    message: format!("'{left}' does not support bitwise operator with '{right}'"),
                    range: expr.syntax().text_range(),
                    ..Default::default()
                });
            }
        }

        ExpressionKind::Literal(Type::Integer(None))
    }

    fn logical_operator(&mut self, expr: &BinaryExpression) -> ExpressionKind {
        let (left_kind, right_kind) = self.extract_lhs_and_rhs(expr);
        let left = self.state().expr_to_type(left_kind);
        let right = self.state().expr_to_type(right_kind);

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
                        ..Default::default()
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

                let typ = self.state().expr_to_type(right_kind);
                let Some(symbol) = self.get_mut(symbol) else {
                    return right_kind;
                };

                if symbol.typ.should_substitute() && !typ.should_substitute() {
                    symbol.typ = typ;
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
            Type::Integer(Some(value)) => Type::Integer(Some(-value)),
            Type::Float(Some(value)) => Type::Float(Some(-value)),
            Type::Integer(_) | Type::Float(_) => typ,
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
            Type::Integer(_) | Type::Unknown => {}
            _ => self.diagnostics.push(Diagnostic {
                message: format!("'{typ}' does not support bitwise not operator"),
                range: expr.syntax().text_range(),
                ..Default::default()
            }),
        }

        ExpressionKind::Literal(Type::Integer(None))
    }

    fn logical_not_operator(&mut self, expr: &PrefixUnaryExpression) -> ExpressionKind {
        if let Some(operand) = expr.operand() {
            self.collect_expr(&operand);
        }

        ExpressionKind::Literal(Type::Boolean(None))
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
        let increment = Some(ExpressionKind::Literal(Type::Integer(Some(1))));
        self.arithmetic_assign_operator(operand, increment, BinaryOperator::AddAssign)
    }

    fn prefix_decrement_operator(&mut self, expr: &PrefixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let decrement = Some(ExpressionKind::Literal(Type::Integer(Some(1))));
        self.arithmetic_assign_operator(operand, decrement, BinaryOperator::SubtractAssign)
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
        let increment = Some(ExpressionKind::Literal(Type::Integer(Some(1))));
        self.arithmetic_assign_operator(operand.clone(), increment, BinaryOperator::AddAssign);
        operand?.into()
    }

    fn postfix_decrement_operator(&mut self, expr: &PostfixUpdateExpression) -> NullableExprKind {
        let operand = self.assignment_lhs(&expr.operand()?);
        let increment = Some(ExpressionKind::Literal(Type::Integer(Some(1))));
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
                ..
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
                        ..Default::default()
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
                    ..Default::default()
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
                ..Default::default()
            });
            return None;
        }

        let function = arguments.remove(0);
        let _context = arguments.remove(0);

        let obj = self.state().expr_to_type(function);
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

    fn unused_variables_diagnostics(&mut self) {
        let mut seen: FxHashMap<SymbolId, bool> = FxHashMap::default();
        dbg!(&self.name_kinds);

        for id in self.name_kinds().values() {
            seen.entry(*id)
                .and_modify(|used| *used = true)
                .or_insert(false);
        }

        for (id, _) in seen.into_iter().filter(|(_, seen)| !seen) {
            let symbol = self.get(id);
            if symbol.kind != SymbolKind::Local {
                continue;
            }

            if symbol.name.starts_with("_") {
                continue;
            }

            self.diagnostics.push(Diagnostic {
                message: format!("Unused local variable '{}'", symbol.name),
                range: symbol.name_range,
                severity: DiagnosticSeverity::Unnecessary,
            });
        }
    }
}
