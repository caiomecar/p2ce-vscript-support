use la_arena::Idx;
use rustc_hash::FxHashMap;
use sq_3_parser::{
    AstNode, SyntaxToken, TextRange, TextSize,
    ast::{self, *},
};
use std::{mem::discriminant, path::PathBuf};

use crate::{
    Diagnostic, DiagnosticSeverity, ExpressionKind, File, FindSymbol, ImportMembers,
    NullableExprKind, Source, SourceSymbol,
    arena::{
        ArenaAlloc, ArenaId, ArrayData, ArrayId, ClassData, ClassId, Container, EnumData, EnumId,
        FunctionData, FunctionId, ImportTarget, ParamsState, Scope, ScopeId, SourceArena,
        StringData, StringId, SymbolId, TableData, TableId,
    },
    db::{Db, ScriptResolutionError, SpecialFunction},
    symbol::{Static, Symbol, SymbolKind, SymbolTable, Type, insert_symbol},
};

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
        name_range: TextRange,
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

struct DeferredFunctionTrace {
    node: Box<dyn IsFunction>,
    scope: ScopeId,
}

pub struct Collector<'db> {
    db: &'db dyn Db,
    file: File,

    imports: FxHashMap<ImportTarget, Vec<File>>,

    arena: SourceArena,
    source_table: Idx<TableData>,
    const_table: Idx<TableData>,
    root_table: Idx<TableData>,

    scope: ScopeId,

    /// The container new members will be added to. Note that this is different from
    /// container that we take symbols from. That one is stored on the scope and can
    /// be acquired via .execution_container()
    container: Container,

    can_break: bool,
    can_continue: bool,
    dead_code: bool,

    function: Option<Idx<FunctionData>>,
    deferred_functions: FxHashMap<Idx<FunctionData>, DeferredFunctionTrace>,

    range_to_expr: FxHashMap<TextRange, ExpressionKind>,
    range_to_symbol: FxHashMap<TextRange, SymbolId>,
    symbol_to_ranges: FxHashMap<SymbolId, Vec<TextRange>>,
    diagnostics: Vec<Diagnostic>,
}

impl<'db> Source for Collector<'db> {
    fn file(&self) -> File {
        self.file
    }

    fn db(&self) -> &dyn Db {
        self.db
    }

    fn arena(&self) -> &SourceArena {
        &self.arena
    }

    fn imports(&self) -> &FxHashMap<ImportTarget, Vec<File>> {
        &self.imports
    }

    fn scope(&self, _offset: TextSize) -> ScopeId {
        self.scope
    }

    fn source_table(&self) -> TableId {
        TableId::new(self.file, self.source_table)
    }

    fn root_table(&self) -> TableId {
        TableId::new(self.file, self.root_table)
    }

    fn const_table(&self) -> TableId {
        TableId::new(self.file, self.const_table)
    }

    fn range_to_expr(&self) -> &FxHashMap<TextRange, ExpressionKind> {
        &self.range_to_expr
    }

    fn range_to_symbol(&self) -> &FxHashMap<TextRange, SymbolId> {
        &self.range_to_symbol
    }

    fn symbol_to_ranges(&self) -> &FxHashMap<SymbolId, Vec<TextRange>> {
        &self.symbol_to_ranges
    }

    fn get<T>(&self, id: T) -> &T::Data
    where
        T: ArenaId,
        SourceArena: std::ops::Index<Idx<T::Data>, Output = T::Data>,
    {
        // To avoid cycle, get the data from the current file from here
        if id.file() != self.file {
            return id.get_data(self.db);
        }

        &self.arena[id.idx()]
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
            range: node.syntax().text_range(),
            ..Default::default()
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
            imports.insert(ImportTarget::Table(TableId::new(file, root_table)), libs);
        }

        let mut collector = Self {
            db,
            file,
            imports,
            scope,
            container,
            can_break: false,
            can_continue: false,
            dead_code: false,
            arena,
            source_table,
            const_table,
            root_table,
            function: None,
            deferred_functions: FxHashMap::default(),
            range_to_expr: FxHashMap::default(),
            range_to_symbol: FxHashMap::default(),
            symbol_to_ranges: FxHashMap::default(),
            diagnostics: Vec::new(),
        };

        for stmt in node.statements() {
            collector.collect_stmt(&stmt);
        }

        assert_eq!(collector.arena[collector.scope].parent, None);

        // Resolve remaining functions
        while let Some(idx) = collector.deferred_functions.keys().next().cloned() {
            let trace = collector.deferred_functions.remove(&idx).unwrap();
            collector.resolve_function_idx(idx, trace);
        }

        collector.unused_variables_diagnostics();

        SourceSymbol {
            imports: collector.imports,
            arena: collector.arena,
            const_table,
            root_table,
            source_table,
            range_to_expr: collector.range_to_expr,
            range_to_symbol: collector.range_to_symbol,
            symbol_to_ranges: collector.symbol_to_ranges,
            diagnostics: collector.diagnostics,
        }
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

    fn new_reference(&mut self, range: TextRange, id: SymbolId) {
        self.range_to_symbol.insert(range, id);
        self.symbol_to_ranges
            .entry(id)
            .and_modify(|list| list.push(range))
            .or_insert_with(|| vec![range]);
    }

    fn symbol(&mut self, symbol: Symbol) -> SymbolId {
        let name_range = symbol.name_range;
        let id = SymbolId::new(self.file, self.arena.alloc(symbol));
        self.new_reference(name_range, id);
        id
    }

    fn class(&mut self, class: &impl IsClass) -> ClassId {
        let expr = class.extends().and_then(|e| e.expression());

        let inherits = if let Some(expr) = expr {
            match self.expr_type(&expr) {
                Type::Class(id) => Some(id),
                Type::Unknown => None,
                typ => {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Trying to inherit from {typ}"),
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

    fn string(&mut self, token_result: (StringNameKind, SyntaxToken)) -> StringId {
        let (left_offset, right_offset, text) = match token_result.0 {
            StringNameKind::Normal => {
                let input = token_result.1.text();
                let (s, left) = input.strip_prefix('"').map_or((input, 0u32), |s| (s, 1));
                let (s, right) = s.strip_suffix('"').map_or((s, 0u32), |s| (s, 1));

                (left, right, s.to_owned())
            }
            StringNameKind::Verbatim => {
                let input = token_result.1.text();
                let (s, left) = input.strip_prefix("@\"").map_or((input, 0u32), |s| (s, 2));
                let (s, right) = s.strip_suffix('"').map_or((s, 0u32), |s| (s, 1));

                (
                    left,
                    right,
                    s.replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace("\"\"", "\\\""),
                )
            }
        };

        let range = token_result.1.text_range();

        let range = TextRange::new(
            range.start() + TextSize::new(left_offset),
            range.end() - TextSize::new(right_offset),
        );

        StringId::new(
            self.file,
            self.arena.alloc(StringData {
                text: text.into_boxed_str(),
                unquoted_range: range,
            }),
        )
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
            parent: Some(self.scope),
            locals: SymbolTable::default(),
            range,
            function: self.function,
        });
    }

    fn exit_scope(&mut self) {
        self.dead_code = false;
        self.scope = self.arena[self.scope].parent.unwrap();
    }

    fn execution_container(&self) -> Container {
        if let Some(id) = self.function {
            let function = &self.arena[id];
            function.bindenv.unwrap_or(function.container)
        } else {
            Container::Table(self.source_table())
        }
    }

    fn add_current_container_member(&mut self, name: String, symbol: SymbolId) {
        self.add_container_member(self.container, name, symbol);
    }

    fn add_container_member(&mut self, container: Container, name: String, symbol: SymbolId) {
        match container {
            Container::Table(id) => {
                if let Some(t) = self.get_mut(id) {
                    insert_symbol(&mut t.members, name, symbol);
                }
            }
            Container::Class(id) => {
                if let Some(c) = self.get_mut(id) {
                    insert_symbol(&mut c.members, name, symbol);
                }
            }
            Container::Instance(id) => {
                if let Some(c) = self.get_mut(id) {
                    insert_symbol(&mut c.members, name, symbol);
                }
            }
            Container::Enum(id) => {
                if let Some(e) = self.get_mut(id) {
                    insert_symbol(&mut e.members, name, symbol);
                }
            }
        }
    }

    /// This is only a speculation, you can actually execute static member as an instance
    /// and the other way around. The best approximation is this though
    fn try_swap_to_instance(
        &mut self,
        member: &impl IsClassMember,
        method_id: Option<FunctionId>,
    ) -> Static {
        if let Container::Class(id) = self.container
            && member.static_keyword().is_none()
        {
            if let Some(func) = method_id.and_then(|id| self.get_mut(id)) {
                func.container = Container::Instance(id);
            }

            Static::No
        } else {
            Static::Yes
        }
    }

    fn collect_params(
        &mut self,
        idx: Idx<FunctionData>,
        parameters: impl Iterator<Item = Parameter>,
    ) {
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
                            ..Default::default()
                        });

                        insert_symbol(&mut self.current_scope().locals, text, symbol);
                        self.arena[idx].params.push(symbol);
                        continue;
                    };

                    let typ = self.expr_type(&expr);

                    let symbol = self.symbol(Symbol {
                        name: text.clone(),
                        typ,
                        kind: SymbolKind::Local,
                        name_range: name.syntax().text_range(),
                        range: var.syntax().text_range(),
                        ..Default::default()
                    });

                    insert_symbol(&mut self.current_scope().locals, text, symbol);
                    self.arena[idx].params.push(symbol);
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

        self.arena[idx].params_state = params_state;
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
                // possibly change error_range.start() to the real offset parameter?
                let Some(member) = self.get_symbol(members, metamethod, error_range.start()) else {
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

                Some(self.call_type(member.typ, arguments, error_range)?)
            }
            Type::Instance(id) => {
                let class = self.get(id);
                let Some(member) = self.get_symbol(&class.members, metamethod, error_range.start())
                else {
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

                Some(self.call_type(member.typ, arguments, error_range)?)
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
                    MetamethodErrors::YesBinary { keyword, right } => {
                        if right != Type::Unknown {
                            self.diagnostics.push(Diagnostic {
                                message: format!(
                                    "'{typ}' does not support {keyword} with '{right}'"
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
                let typ = self.get(id).yields.unwrap_or(Type::Unknown);
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

    fn collect_function<T>(&mut self, node: &T) -> FunctionId
    where
        T: IsFunction + Clone + 'static,
    {
        let bindenv = node
            .environment()
            .and_then(|e| e.expression())
            .map(|env| (env.syntax().text_range(), self.expr_type(&env)))
            .and_then(|(range, typ)| {
                if let Ok(container) = Container::try_from(typ) {
                    Some(container)
                } else {
                    if typ != Type::Unknown {
                        self.diagnostics.push(Diagnostic {
                            message: format!("Trying to use '{typ}' as function's environment"),
                            range,
                            severity: DiagnosticSeverity::Warning,
                            ..Default::default()
                        });
                    }
                    None
                }
            });

        let range = match node.body() {
            Some(FunctionBody::Expr(body)) => body.syntax().text_range(),
            Some(FunctionBody::Stmt(body)) => body.syntax().text_range(),
            None => TextRange::empty(node.syntax().text_range().end()),
        };

        let id = FunctionId::new(
            self.file,
            self.arena.alloc(FunctionData {
                range,
                container: self.container,
                bindenv,
                params: Vec::new(),
                params_state: ParamsState::NoDefault,
                ret: Type::Null,
                throws: None,
                yields: None,
            }),
        );

        self.enter_scope(range);

        if let Some(param_list) = node.parameter_list() {
            self.collect_params(id.idx(), param_list.parameters());
        }

        self.deferred_functions.insert(
            id.idx(),
            DeferredFunctionTrace {
                node: Box::new(node.clone()),
                scope: self.scope,
            },
        );

        self.exit_scope();

        id
    }

    fn resolve_function_idx(&mut self, idx: Idx<FunctionData>, trace: DeferredFunctionTrace) {
        // No reason to change stuff if function has no body
        let Some(body) = trace.node.body() else {
            return;
        };

        let function = &self.arena[idx];

        let save_container = self.container;
        self.container = function.bindenv.unwrap_or(function.container);
        let save_scope = self.scope;
        self.scope = trace.scope;
        let save_function = self.function;
        self.function = Some(idx);

        match body {
            FunctionBody::Expr(expr) => {
                self.collect_expr(&expr);
            }
            FunctionBody::Stmt(stmt) => self.collect_stmt(&stmt),
        };

        self.container = save_container;
        self.scope = save_scope;
        self.function = save_function;
    }

    fn resolve_function(&mut self, id: FunctionId) {
        // If function is external it is already resolved
        if id.file() != self.file {
            return;
        }

        let idx = id.idx();
        // If function is not in deferred_functions it is already resolved
        let Some(trace) = self.deferred_functions.remove(&idx) else {
            return;
        };

        self.resolve_function_idx(idx, trace)
    }

    fn get_member_name(&mut self, name: MemberName) -> Option<(TextRange, String)> {
        match name {
            MemberName::Identifier(n) => {
                let text = n.name().and_then(|n| n.text())?;
                Some((n.syntax().text_range(), text))
            }
            MemberName::String(n) => {
                let id = n.token().map(|r| self.string(r))?;
                let s = self.get(id);
                Some((s.unquoted_range, s.text.to_string()))
            }
            MemberName::Computed(n) => {
                let typ = self.expr_type(&n.expression()?);
                let Type::String(Some(id)) = typ else {
                    return None;
                };
                let s = self.get(id);
                Some((s.unquoted_range, s.text.to_string()))
            }
        }
    }

    fn collect_table_member(&mut self, member: &Member) {
        match member {
            Member::Property(property) => self.collect_table_property(property),
            Member::Method(method) => {
                let id = self.collect_function(method);

                let Some((name, text)) = get_name(method) else {
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: text.clone(),
                    typ: Type::Function(id),
                    name_range: name.syntax().text_range(),
                    range: method.syntax().text_range(),
                    ..Default::default()
                });
                self.add_current_container_member(text, symbol);
            }
            Member::Constructor(constructor) => {
                let id = self.collect_function(constructor);

                let Some(keyword) = constructor.constructor_keyword() else {
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: "constructor".to_owned(),
                    typ: Type::Function(id),
                    name_range: keyword.text_range(),
                    range: constructor.syntax().text_range(),
                    ..Default::default()
                });

                self.add_current_container_member("constructor".to_owned(), symbol);
            }
        }
    }

    fn collect_class_member(&mut self, member: &Member) {
        match member {
            Member::Property(property) => self.collect_class_property(property),
            Member::Method(method) => {
                let id = self.collect_function(method);
                let statik = self.try_swap_to_instance(method, Some(id));

                let Some((name, text)) = get_name(method) else {
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: text.clone(),
                    typ: Type::Function(id),
                    kind: SymbolKind::Property(statik),
                    name_range: name.syntax().text_range(),
                    range: method.syntax().text_range(),
                });
                self.add_current_container_member(text, symbol);
            }
            Member::Constructor(constructor) => {
                let id = self.collect_function(constructor);
                let statik = self.try_swap_to_instance(constructor, Some(id));

                let Some(keyword) = constructor.constructor_keyword() else {
                    return;
                };

                let symbol = self.symbol(Symbol {
                    name: "constructor".to_owned(),
                    typ: Type::Function(id),
                    kind: SymbolKind::Property(statik),
                    name_range: keyword.text_range(),
                    range: constructor.syntax().text_range(),
                });

                self.add_current_container_member("constructor".to_owned(), symbol);
            }
        }
    }

    fn collect_table_property(&mut self, property: &Property) {
        let value = property.value().and_then(|v| self.collect_expr(&v));

        let Some(name) = property.name() else {
            return;
        };

        let Some((name_range, text)) = self.get_member_name(name) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ: self.expr_to_type(value),
            name_range,
            range: property.syntax().text_range(),
            ..Default::default()
        });

        self.add_current_container_member(text, symbol);
    }

    fn collect_class_property(&mut self, property: &Property) {
        let value = property
            .value()
            .map_or(Type::Unknown, |v| self.expr_type(&v));

        let statik = self.try_swap_to_instance(
            property,
            match value {
                Type::Function(id) => Some(id),
                _ => None,
            },
        );

        let Some(name) = property.name() else {
            return;
        };

        let Some((name_range, text)) = self.get_member_name(name) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ: value,
            kind: SymbolKind::Property(statik),
            name_range,
            range: property.syntax().text_range(),
        });

        self.add_current_container_member(text, symbol);
    }

    fn collect_enum_property(&mut self, property: &Property, default_value: i32) {
        let value = match property.value() {
            Some(expr) => {
                let value = self.collect_expr(&expr);
                self.check_constant(value, expr.syntax().text_range());
                value
            }
            None => Some(ExpressionKind::Literal(Type::Integer(Some(default_value)))),
        };

        let Some(name) = property.name() else {
            return;
        };

        let Some((name_range, text)) = self.get_member_name(name) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ: self.expr_to_type(value),
            kind: SymbolKind::EnumMember,
            name_range,
            range: property.syntax().text_range(),
            ..Default::default()
        });

        self.add_current_container_member(text, symbol);
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        if self.dead_code && !matches!(stmt, Stmt::Empty(_)) {
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
                    ..Default::default()
                });

                insert_symbol(&mut self.current_scope().locals, text, id);
                continue;
            };

            let typ = self.expr_type(&expr);
            let id = self.symbol(Symbol {
                name: text.clone(),
                typ,
                kind: SymbolKind::Local,
                name_range: name.syntax().text_range(),
                range: var.syntax().text_range(),
                ..Default::default()
            });

            insert_symbol(&mut self.current_scope().locals, text, id);
        }
    }

    fn local_function(&mut self, decl: &LocalFunctionDeclaration) {
        let id = self.collect_function(decl);
        let Some((name, text)) = get_name(decl) else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: text.clone(),
            typ: Type::Function(id),
            kind: SymbolKind::Local,
            name_range: name.syntax().text_range(),
            range: decl.syntax().text_range(),
            ..Default::default()
        });

        insert_symbol(&mut self.current_scope().locals, text, symbol);
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
            typ: self.expr_to_type(value),
            kind: SymbolKind::Constant,
            name_range: name.syntax().text_range(),
            range: stmt.syntax().text_range(),
            ..Default::default()
        });

        insert_symbol(&mut self.arena[self.const_table].members, text, symbol);
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
                    ..Default::default()
                });

                insert_symbol(&mut self.current_scope().locals, text, symbol);
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
                    ..Default::default()
                });

                insert_symbol(&mut self.current_scope().locals, text, symbol);
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
            self.collect_class_member(&member);
        }
        self.container = save_symbol;
    }

    fn function_statement(&mut self, stmt: &FunctionStatement) {
        let id = self.collect_function(stmt);

        let Some(qualified_name) = stmt.name() else {
            return;
        };

        let mut names: Vec<_> = qualified_name.names().collect();

        let Some(final_name) = names.pop() else {
            return;
        };

        let Some(final_text) = final_name.text() else {
            return;
        };

        if names.is_empty() {
            // Plain `function abc()`: declare in current container
            let symbol = self.symbol(Symbol {
                name: final_text.clone(),
                typ: Type::Function(id),
                name_range: final_name.syntax().text_range(),
                range: stmt.syntax().text_range(),
                ..Default::default()
            });

            self.add_current_container_member(final_text, symbol);
            return;
        }

        let Some(text) = names[0].text() else {
            return;
        };

        let offset = qualified_name.syntax().text_range().end();

        let members = self
            .members_of_container(
                self.execution_container(),
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                false,
            )
            .into_iter()
            .find_map(|(name, id)| if text == name { Some(id) } else { None });

        let root = || {
            self.members_of_table(
                self.root_table(),
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                ImportMembers::Root,
            )
            .into_iter()
            .find_map(|(name, id)| if text == name { Some(id) } else { None })
        };

        let Some(symbol_id) = members.or_else(root) else {
            if self
                .local_members(offset)
                .into_iter()
                .find(|(name, _)| &text == name)
                .is_some()
            {
                self.diagnostics.push(Diagnostic {
                    message: "Function statement does not lookup locals. Initial symbol not found"
                        .to_owned(),
                    range: names[0].syntax().text_range(),
                    severity: DiagnosticSeverity::Information,
                    ..Default::default()
                });
            }
            return;
        };

        let mut typ = self.get(symbol_id).typ;
        let name_range = names[0].syntax().text_range();
        self.new_reference(name_range, symbol_id);

        for segment in &names[1..] {
            let Some(text) = segment.text() else {
                return;
            };

            let arguments = vec![
                Some(ExpressionKind::Literal(Type::String(None))),
                Some(ExpressionKind::Literal(Type::Unknown)),
            ];

            let Some(container) = self.call_new_slot(typ, &arguments, stmt.syntax().text_range())
            else {
                return;
            };

            let Some(id) = self
                .members_of_container(
                    container,
                    FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                    false,
                )
                .into_iter()
                .find_map(|(name, id)| if text == name { Some(id) } else { None })
            else {
                return;
            };

            typ = self.get(id).typ;

            let name_range = segment.syntax().text_range();
            self.new_reference(name_range, id);
        }

        let arguments = vec![
            Some(ExpressionKind::Literal(Type::String(None))),
            Some(ExpressionKind::Literal(Type::Function(id))),
        ];

        let Some(container) = self.call_new_slot(typ, &arguments, stmt.syntax().text_range())
        else {
            return;
        };

        let symbol = self.symbol(Symbol {
            name: final_text.clone(),
            typ: Type::Function(id),
            name_range: final_name.syntax().text_range(),
            range: stmt.syntax().text_range(),
            ..Default::default()
        });

        self.add_container_member(container, final_text, symbol);

        if let Some(function) = self.get_mut(id) {
            function.container = container;
        }
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
                ..Default::default()
            });

            insert_symbol(&mut self.arena[self.const_table].members, text, symbol);
        }

        let save_symbol = self.container;
        self.container = Container::Enum(enum_);
        for (value, property) in stmt.members().enumerate() {
            self.collect_enum_property(&property, value as i32);
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

        if let Some(then_stmt) = stmt.statement() {
            self.enter_scope(then_stmt.syntax().text_range());
            self.collect_stmt(&then_stmt);
            self.exit_scope();
        }

        if let Some(else_stmt) = stmt.else_branch().and_then(|e| e.statement()) {
            self.enter_scope(else_stmt.syntax().text_range());
            self.collect_stmt(&else_stmt);
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
        let typ = if let Some(discriminant) = stmt.discriminant() {
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
                        let case_type = self.expr_type(&test);
                        if !matches!(
                            (typ, case_type),
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
                                message: format!("Case of type '{case_type}' is incompitable with discriminant of type '{typ}'"),
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
        let typ = if let Some(value) = stmt.value() {
            Some(self.expr_type(&value))
        } else {
            None
        };

        self.dead_code = true;

        let Some(function) = self.function else {
            if typ.is_some() {
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

        match typ {
            None | Some(Type::Unknown) => {}
            Some(typ) => {
                self.arena[function].ret = typ;
            }
        }
    }

    fn yield_statement(&mut self, stmt: &YieldStatement) {
        let typ = if let Some(value) = stmt.value() {
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

        if self.arena[function].yields.is_none() {
            self.arena[function].yields = Some(typ);
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
                    ..Default::default()
                });

                insert_symbol(&mut self.current_scope().locals, text, symbol);
            };
        }

        if let Some(body) = catch.body() {
            self.collect_stmt(&body);
        }

        self.exit_scope();
    }

    fn throw_statement(&mut self, stmt: &ThrowStatement) {
        // mark current function as exception throwing
        let typ = if let Some(value) = stmt.value() {
            self.expr_type(&value)
        } else {
            Type::Unknown
        };

        self.dead_code = true;
        let Some(function) = self.function else {
            return;
        };

        if self.arena[function].throws.is_none() {
            self.arena[function].throws = Some(typ);
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
            Expr::Line(_) => Some(ExpressionKind::Literal(Type::Integer(None))),
            Expr::Parenthesised(expr) => self.parenthesised_expression(expr),
            Expr::Function(expr) => Some(self.function_expression(expr)),
            Expr::Lambda(expr) => Some(self.lambda_expression(expr)),
        };
        if let Some(kind) = kind {
            let key = expr.syntax().text_range();
            self.range_to_expr.insert(key, kind);
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
            LiteralExpressionKind::String => {
                let string = self.string((StringNameKind::Normal, token));

                ExpressionKind::Literal(Type::String(Some(string)))
            }
            LiteralExpressionKind::VerbatimString => {
                let string = self.string((StringNameKind::Verbatim, token));

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
            self.collect_table_member(&member);
        }
        self.container = save_symbol;

        ExpressionKind::Literal(Type::Table(table))
    }

    fn class_expression(&mut self, expr: &ClassExpression) -> ExpressionKind {
        let class = self.class(expr);

        let save_symbol = self.container;
        self.container = Container::Class(class);
        for member in expr.members() {
            self.collect_class_member(&member);
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

        let consts = || {
            self.members_of_table(
                self.const_table(),
                FindSymbol::OnlyBefore(offset),
                ImportMembers::Const,
            )
            .into_iter()
            .find_map(filter)
        };

        let members = || {
            self.members_of_container(
                self.execution_container(),
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                false,
            )
            .into_iter()
            .find_map(filter)
        };

        let root = || {
            self.members_of_table(
                self.root_table(),
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                ImportMembers::Root,
            )
            .into_iter()
            .find_map(filter)
        };

        locals
            .or_else(consts)
            .or_else(members)
            .or_else(root)
            .map(|id| {
                self.new_reference(expr.syntax().text_range(), id);
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

        self.members_of_table(
            self.root_table(),
            FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
            ImportMembers::Root,
        )
        .into_iter()
        .find_map(|(name, id)| {
            if name == text {
                self.new_reference(name_node.syntax().text_range(), id);
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
            .members_of_type(
                obj,
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                false,
            )
            .into_iter()
            .find_map(|(name, id)| {
                if name == text {
                    self.new_reference(name_node.syntax().text_range(), id);
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

        let string = self.get(id);
        let text = string.text.to_string();
        let name_range = string.unquoted_range;
        let offset = expr.syntax().text_range().end();

        let result = self
            .members_of_type(
                obj,
                FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                false,
            )
            .into_iter()
            .find_map(|(name, id)| {
                if name == text {
                    self.new_reference(name_range, id);
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

    fn include_script(&mut self, arguments: &[NullableExprKind], error_range: TextRange) {
        let Some(path) = arguments.get(0) else {
            // Case with no path will be handled in call_type
            return;
        };

        let Type::String(str) = self.expr_to_type(*path) else {
            // Same as above
            return;
        };

        let Some(id) = str else {
            self.diagnostics.push(Diagnostic {
                message: format!(
                    "Could not resolve the path statically, symbols will not be included"
                ),
                range: error_range,
                severity: DiagnosticSeverity::Information,
            });
            return;
        };

        let path = PathBuf::from(self.get(id).text.to_string());

        let file = match self.db.get_script(path) {
            Ok(file) => file,
            Err(ScriptResolutionError::AbsolutePath) => {
                self.diagnostics.push(Diagnostic {
                    message: "The path must be relative".to_owned(),
                    range: error_range,
                    severity: DiagnosticSeverity::Warning,
                });
                return;
            }
            Err(ScriptResolutionError::DoesntExist) => {
                self.diagnostics.push(Diagnostic {
                    message: "Could not find the path".to_owned(),
                    range: error_range,
                    severity: DiagnosticSeverity::Warning,
                });
                return;
            }
            Err(ScriptResolutionError::NoRootAssigned) => {
                self.diagnostics.push(Diagnostic {
                    message: "TF2 Installation folder is not set, the symbols will not be included"
                        .to_owned(),
                    range: error_range,
                    severity: DiagnosticSeverity::Information,
                });
                return;
            }
            Err(ScriptResolutionError::WrongExtension) => {
                self.diagnostics.push(Diagnostic {
                    message: "The path must have none or '.nut' extension".to_owned(),
                    range: error_range,
                    severity: DiagnosticSeverity::Warning,
                });
                return;
            }
        };

        let target = match arguments.get(1) {
            Some(Some(expr)) => {
                let typ = self.expr_to_type(Some(*expr));
                let Ok(target) = ImportTarget::try_from(typ) else {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Type '{typ}' cannot receive new members"),
                        range: error_range,
                        severity: DiagnosticSeverity::Warning,
                    });
                    return;
                };
                target
            }
            Some(None) | None => match self.execution_container() {
                Container::Table(id) => ImportTarget::Table(id),
                Container::Class(id) => ImportTarget::Class(id),
                Container::Instance(id) => ImportTarget::Class(id),
                Container::Enum(_) => {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Type 'enum' cannot receive new members"),
                        range: error_range,
                        severity: DiagnosticSeverity::Warning,
                    });
                    return;
                }
            },
        };

        self.imports
            .entry(target)
            .and_modify(|e| e.push(file))
            .or_insert_with(|| vec![file]);
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
                if !is_variadic && arguments.len() > self.get(id).params.len() {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "Passing {} parameters when only {} is possible ",
                            arguments.len(),
                            self.get(id).params.len()
                        ),
                        range: error_range,
                        ..Default::default()
                    });
                }

                for (count, argument_kind) in arguments.iter().cloned().enumerate() {
                    let Some(&param) = self.get(id).params.get(count) else {
                        continue;
                    };

                    match (self.expr_to_type(argument_kind), self.get(param).typ) {
                        (Type::Unknown | Type::Null, required_kind) => {
                            // If passed in parameter has type of unknown
                            // we can coerce it to be the type of a required parameter
                            if let Some(ExpressionKind::Symbol(id)) = argument_kind {
                                if let Some(symbol) = self.get_mut(id)
                                    && symbol.typ.should_substitute_with(required_kind)
                                {
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

                let least_params_required = match self.get(id).params_state {
                    ParamsState::NoDefault => self.get(id).params.len(),
                    ParamsState::Default(from) | ParamsState::VarArgs(from) => from as usize,
                };

                if arguments.len() < least_params_required {
                    self.diagnostics.push(Diagnostic {
                        message: format!(
                            "Passing {} parameters when at least {} is required",
                            arguments.len(),
                            least_params_required
                        ),
                        range: error_range,
                        ..Default::default()
                    });
                }

                // We resolve the params first so we can get param type substitution before we run the body
                // Resolve the deferred functions on the first call site, then reuse the result
                self.resolve_function(id);

                match self.db.check_special(id) {
                    Some(SpecialFunction::IncludeScript) => {
                        self.include_script(arguments, error_range);
                    }
                    Some(SpecialFunction::DoIncludeScript) => {
                        self.include_script(arguments, error_range);
                    }
                    Some(SpecialFunction::GetRootTable) => {
                        // Overrides return
                        return Some(Type::Table(self.root_table()));
                    }
                    Some(SpecialFunction::GetConstTable) => {
                        // Overrides return
                        return Some(Type::Table(self.const_table()));
                    }
                    None => {}
                };

                Some(if self.get(id).yields.is_some() {
                    Type::Generator(id)
                } else {
                    self.clone_type(self.get(id).ret)
                })
            }
            Type::Class(id) => {
                let class = self.get(id);
                if let Some(symbol) =
                    self.get_symbol(&class.members, "constructor", error_range.start())
                {
                    self.call_type(symbol.typ, arguments, error_range);
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
        let typ = self.expr_type(&operand);
        Some(ExpressionKind::Literal(self.clone_type(typ)))
    }

    fn extract_lhs_and_rhs(
        &mut self,
        expr: &BinaryExpression,
    ) -> (NullableExprKind, NullableExprKind) {
        let right = expr.rhs().and_then(|r| self.collect_expr(&r));
        let left = expr.lhs().and_then(|l| self.collect_expr(&l));
        (left, right)
    }

    fn assignment_lhs(&mut self, expr: &Expr) -> Option<AssignmentLeftHandSide> {
        // We explicitly don't allow imports anywhere so we can do a new addition to our container
        // Otherwise we find a symbol that is outside of our scope which is unmodifable
        match expr {
            Expr::Name(expr) => {
                let text = expr.text()?;
                let range = expr.syntax().text_range();
                let offset = range.end();

                let filter = |(name, id)| {
                    if name == text { Some(id) } else { None }
                };

                let locals = self.local_members(offset).into_iter().find_map(filter);

                if let Some(symbol) = locals {
                    self.new_reference(range, symbol);
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: None,
                        symbol,
                        name_range: range,
                        range,
                    });
                }

                let consts = self
                    .members_of_table(
                        self.const_table(),
                        FindSymbol::OnlyBefore(offset),
                        ImportMembers::Const,
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = consts {
                    self.new_reference(range, symbol);
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: None,
                        symbol,
                        name_range: range,
                        range,
                    });
                }

                let members = self
                    .members_of_container(
                        self.execution_container(),
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        false,
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = members {
                    self.new_reference(range, symbol);
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(self.execution_container().into()),
                        symbol,
                        name_range: range,
                        range,
                    });
                }

                let root = self
                    .members_of_table(
                        self.root_table(),
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        ImportMembers::Root,
                    )
                    .into_iter()
                    .find_map(filter);

                if let Some(symbol) = root {
                    self.new_reference(range, symbol);
                    return Some(AssignmentLeftHandSide::Exists {
                        parent: Some(Type::Table(self.root_table())),
                        symbol,
                        name_range: range,
                        range,
                    });
                }

                Some(AssignmentLeftHandSide::CanCreate {
                    parent: self.container.into(),
                    new_key: text.into_boxed_str(),
                    name_range: range,
                    range,
                })
            }
            Expr::MemberAccess(expr) => {
                let obj = self.expr_type(&expr.object()?);
                let member_part = expr.member_part()?;

                let (name, text) = get_name(&member_part)?;
                let range = expr.syntax().text_range();
                let name_range = name.syntax().text_range();
                let offset = range.end();

                Some(
                    self.members_of_type(
                        obj,
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        false,
                    )
                    .into_iter()
                    .find_map(|(name, id)| if name == text { Some(id) } else { None })
                    .map_or_else(
                        || AssignmentLeftHandSide::CanCreate {
                            parent: obj,
                            new_key: text.into_boxed_str(),
                            name_range,
                            range,
                        },
                        |id| {
                            self.new_reference(name_range, id);
                            AssignmentLeftHandSide::Exists {
                                parent: Some(obj),
                                symbol: id,
                                name_range,
                                range,
                            }
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

                let string = self.get(id);
                let text = string.text.to_string();
                let name_range = string.unquoted_range;
                let range = expr.syntax().text_range();
                let offset = range.end();

                Some(
                    self.members_of_type(
                        obj,
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        false,
                    )
                    .into_iter()
                    .find_map(|(name, id)| if name == text { Some(id) } else { None })
                    .map_or_else(
                        || AssignmentLeftHandSide::CanCreate {
                            parent: obj,
                            new_key: text.into_boxed_str(),
                            name_range,
                            range,
                        },
                        |id| {
                            self.new_reference(name_range, id);
                            AssignmentLeftHandSide::Exists {
                                parent: Some(obj),
                                symbol: id,
                                name_range,
                                range,
                            }
                        },
                    ),
                )
            }
            Expr::RootAccess(expr) => {
                let (name, text) = get_name(expr)?;
                let range = expr.syntax().text_range();
                let name_range = name.syntax().text_range();
                let offset = range.end();

                let root = self.root_table();
                Some(
                    self.members_of_table(
                        root,
                        FindSymbol::BeforeIfInExecutionRange(offset, self.scope),
                        ImportMembers::Root,
                    )
                    .into_iter()
                    .find_map(|(name, id)| if name == text { Some(id) } else { None })
                    .map_or_else(
                        || AssignmentLeftHandSide::CanCreate {
                            parent: Type::Table(root),
                            new_key: text.into_boxed_str(),
                            name_range,
                            range,
                        },
                        |id| {
                            self.new_reference(name_range, id);
                            AssignmentLeftHandSide::Exists {
                                parent: Some(Type::Table(root)),
                                symbol: id,
                                name_range,
                                range,
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
                let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
                let left_kind = expr.lhs().and_then(|l| self.assignment_lhs(&l));

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
                    typ: self.expr_to_type(right_kind),
                    name_range,
                    range: expr_range,
                    ..Default::default()
                });

                self.add_container_member(container, new_key.into_string(), symbol);
            }
            Some(AssignmentLeftHandSide::Exists {
                parent,
                symbol,
                name_range,
                range,
            }) => {
                if let Some(parent) = parent {
                    // Problematic: when we have something like
                    // ::a <- 1
                    // a <- 1
                    // The `parent` for the second assignment becomes the root which means that the code
                    // below will add the symbol to the root table instead of adding it to the current
                    // `this`, we also can't just map the root to `this` since it doesn't consider
                    // ::a <- 1
                    // ::a <- 1
                    // Where both symbols should go to the root
                    // to solve this we check if name_range == range which distinguishes plain name
                    // expressions from other expressions
                    let parent = if name_range == range {
                        self.execution_container().into()
                    } else {
                        parent
                    };

                    let arguments = vec![
                        Some(ExpressionKind::Literal(Type::String(None))),
                        right_kind,
                    ];

                    let Some(container) = self.call_new_slot(parent, &arguments, range) else {
                        return right_kind;
                    };

                    let name = self.get(symbol).name.clone();

                    let symbol = self.symbol(Symbol {
                        name: name.clone(),
                        typ: self.expr_to_type(right_kind),
                        name_range,
                        range: expr_range,
                        ..Default::default()
                    });

                    self.add_container_member(container, name, symbol);
                    // Parent is only None for locals and consts
                } else {
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

                let typ = self.expr_to_type(right_kind);

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
        let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
        let left_kind = expr.lhs().and_then(|l| self.assignment_lhs(&l));

        if let Some(container) = lhs_container(left_kind.as_ref())
            && let Type::Function(id) = self.expr_to_type(right_kind)
            && let Some(function) = self.get_mut(id)
        {
            function.container = container;
        }

        self.do_new_slot(left_kind, right_kind, expr.syntax().text_range())
    }

    fn assign_operator(&mut self, expr: &BinaryExpression) -> NullableExprKind {
        let right_kind = expr.rhs().and_then(|r| self.collect_expr(&r));
        let left_kind = expr.lhs().and_then(|l| self.assignment_lhs(&l));

        if let Some(container) = lhs_container(left_kind.as_ref())
            && let Type::Function(id) = self.expr_to_type(right_kind)
            && let Some(function) = self.get_mut(id)
        {
            function.container = container;
        }

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
                ..
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

                let typ = self.expr_to_type(right_kind);
                let Some(symbol) = self.get_mut(symbol) else {
                    return right_kind;
                };

                if symbol.typ.should_substitute_with(typ) {
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
        let right = self.expr_to_type(right_kind);

        match right {
            Type::Array(_) => {
                let left = self.expr_to_type(left_kind);
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
        let left = self.expr_to_type(left_kind);
        let right = self.expr_to_type(right_kind);

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
        let left = self.expr_to_type(left_kind);
        let right = self.expr_to_type(right_kind);

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
        let left = self.expr_to_type(left_kind);
        let right = self.expr_to_type(right_kind);

        ExpressionKind::Literal(if left == Type::Unknown { right } else { left })
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
                ..
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

                let typ = self.expr_to_type(right_kind);
                let Some(symbol) = self.get_mut(symbol) else {
                    return right_kind;
                };

                if symbol.typ.should_substitute_with(typ) {
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
                } else {
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
            Type::Generator(id) => Some(ExpressionKind::Literal(self.get(id).yields?)),
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
        let id = self.collect_function(expr);
        ExpressionKind::Literal(Type::Function(id))
    }

    fn lambda_expression(&mut self, expr: &LambdaExpression) -> ExpressionKind {
        let id = self.collect_function(expr);
        ExpressionKind::Literal(Type::Function(id))
    }

    fn unused_variables_diagnostics(&mut self) {
        for (id, references) in self.symbol_to_ranges.iter() {
            if references.len() > 1 {
                continue;
            }

            let symbol = self.get(*id);
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
