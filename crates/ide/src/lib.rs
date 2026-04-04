mod arena;
mod collector;
mod db;
// mod doc;
mod symbol;

use std::collections::hash_map::Entry;

use la_arena::Idx;
use rustc_hash::{FxHashMap, FxHashSet};
use sq_3_parser::{TextRange, TextSize};

use crate::{
    arena::{
        ClassId, Container, EnumId, FunctionId, ScopeId, SourceArena, SymbolId, TableData, TableId,
    },
    db::Db,
    symbol::{FlatSymbolTable, Static, to_flat_symbol_table},
};

pub use arena::{ArenaId, FunctionData, ParamsState};
pub use db::{Database, File, line_index, parse, source_symbol};
pub use symbol::{Symbol, SymbolKind, SymbolTable, Type};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub range: TextRange,
    pub severity: DiagnosticSeverity,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    #[default]
    Error,
    Warning,
    Unnecessary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpressionKind {
    // L value
    Literal(Type),
    // R value
    Symbol(SymbolId),
}

pub type NullableExprKind = Option<ExpressionKind>;

macro_rules! builtin_members {
    ($func:ident => $ty:ident) => {
        fn $func(db: &dyn Db) -> FlatSymbolTable {
            if let Some(builtins) = db.builtins().map(|b| &b.$ty) {
                builtins.clone()
            } else {
                FlatSymbolTable::default()
            }
        }
    };
}

builtin_members!(integer_members => integer);
builtin_members!(float_members => float);
builtin_members!(boolean_members => boolean);
builtin_members!(string_members => string);
builtin_members!(array_members => array);
builtin_members!(function_members => function);
builtin_members!(generator_members => generator);
builtin_members!(thread_members => thread);
builtin_members!(weakref_members => weakref);
builtin_members!(instance_members => instance);
builtin_members!(builtin_table_members => table);
builtin_members!(builtin_class_members => class);

#[derive(Debug, Clone, Copy)]
pub enum GetMembers {
    Container(Container),
    Const,
    Root,
}

#[derive(Debug, Clone, Copy)]
pub enum GetMembersInner {
    Container(Container),
    RootAsSource,
    Root,
    ConstAsSource,
    Const,
}

pub enum FindSymbol {
    Any,
    OnlyBefore(TextSize),
    BeforeIfInExecutionRange(TextSize),
}

#[derive(Debug)]
pub enum FunctionIdResolution {
    Function(FunctionId),
    DefaultConstructor,
}

/// This trait exists to share behaviour between Collector and SourceFile
/// Which might (or might not) use the same functions. Note that
/// SourceSymbol is immutable once constructed so there's no mutable methods
pub trait Source {
    fn file(&self) -> File;
    fn db(&self) -> &dyn Db;
    fn arena(&self) -> &SourceArena;
    fn imports(&self) -> &FxHashMap<Container, Vec<File>>;
    fn scope(&self, offset: TextSize) -> ScopeId;
    fn root_table(&self) -> TableId;
    fn const_table(&self) -> TableId;
    fn expr_kinds(&self) -> &FxHashMap<TextRange, ExpressionKind>;
    fn name_kinds(&self) -> &FxHashMap<TextRange, SymbolId>;
    fn diagnostics(&self) -> &[Diagnostic];

    fn get<T>(&self, id: T) -> &T::Data
    where
        T: ArenaId,
        SourceArena: std::ops::Index<Idx<T::Data>, Output = T::Data>;

    fn all_symbols(&self) -> impl Iterator<Item = (Idx<Symbol>, &Symbol)> {
        self.arena().all_symbols()
    }

    fn to_function_id(&self, typ: Type, offset: TextSize) -> Option<FunctionIdResolution> {
        match typ {
            Type::Class(id) => {
                let class = self.get(id);
                let Some(member) = self.get_symbol(&class.members, "constructor", offset) else {
                    return Some(FunctionIdResolution::DefaultConstructor);
                };
                self.to_function_id(member.typ, offset)
            }
            Type::Function(id) => Some(FunctionIdResolution::Function(id)),
            Type::Table(id) => {
                let table = self.get(id);
                let Some(delegate_idx) = table.delegate else {
                    return None;
                };

                let members = &self.get(delegate_idx).members;
                let Some(member) = self.get_symbol(members, "_call", offset) else {
                    return None;
                };

                self.to_function_id(member.typ, offset)
            }
            Type::Instance(id) => {
                let class = self.get(id);
                let Some(member) = self.get_symbol(&class.members, "_call", offset) else {
                    return None;
                };

                self.to_function_id(member.typ, offset)
            }
            _ => None,
        }
    }

    fn local_members(&self, offset: TextSize) -> FlatSymbolTable {
        let mut scope = Some(self.scope(offset));
        let mut result = FlatSymbolTable::default();

        while let Some(current_scope) = scope {
            let new = self.arena()[current_scope].locals.clone();
            for (name, ids) in new {
                let entry = result.entry(name.clone());
                let Entry::Vacant(entry) = entry else {
                    continue;
                };

                let mut last = None;
                for id in ids {
                    let symbol = self.get(id);
                    if symbol.range.end() >= offset {
                        break;
                    };

                    last = Some(id);
                }

                if let Some(value) = last {
                    entry.insert(value);
                }
            }
            scope = self.arena()[current_scope].parent;
        }

        result
    }

    fn enum_members(&self, enum_: EnumId) -> FlatSymbolTable {
        let enum_ = self.get(enum_);
        // Enum does not support adding new slots nor defining methods, so there can
        // be only 1 symbol under a certain name for the duration of the whole program
        to_flat_symbol_table(enum_.members.clone())
    }

    fn table_members(&self, table: TableId) -> SymbolTable {
        let table = self.get(table);
        let mut result = table.members.clone();

        if let Some(delegate) = table.delegate {
            for (k, v) in &self.get(delegate).members {
                result.entry(k.clone()).or_insert(v.clone());
            }
        }

        result
    }

    fn class_members(&self, class: ClassId) -> SymbolTable {
        let class = self.get(class);
        class.members.clone()
    }

    fn import_members(&self, settings: GetMembers) -> FlatSymbolTable {
        let mut already_included = FxHashSet::default();
        already_included.insert(self.file());
        let mut result = FlatSymbolTable::default();
        // If we ask for root symbols but imports contains symbols for root container
        // how to not iterate the files twice?
        // include source_table + root_table where it's the import goes in the root table
        // include just root_table where it's not put in the root table?
        // First iterate through possi
        let (first_settings, second_settings, container) = match settings {
            GetMembers::Container(container) => (
                GetMembersInner::Container(container),
                None,
                match container {
                    Container::Instance(id) => Container::Class(id),
                    _ => container,
                },
            ),
            GetMembers::Const => (
                GetMembersInner::ConstAsSource,
                Some(GetMembersInner::Const),
                Container::Table(self.const_table()),
            ),
            GetMembers::Root => (
                GetMembersInner::RootAsSource,
                Some(GetMembersInner::Root),
                Container::Table(self.root_table()),
            ),
        };

        let imports = self.imports();

        if let Some(imports) = imports.get(&container) {
            for import in imports {
                if !already_included.insert(*import) {
                    continue;
                }

                result.extend(self.import_members_inner(
                    *import,
                    &mut already_included,
                    first_settings,
                ));
            }
        }

        let Some(second_settings) = second_settings else {
            return result;
        };

        for import_list in imports.values() {
            for import in import_list {
                if !already_included.insert(*import) {
                    continue;
                }

                result.extend(self.import_members_inner(
                    *import,
                    &mut already_included,
                    second_settings,
                ));
            }
        }

        result
    }

    fn import_members_inner(
        &self,
        import: File,
        already_included: &mut FxHashSet<File>,
        settings: GetMembersInner,
    ) -> FlatSymbolTable {
        let mut result = match settings {
            GetMembersInner::Container(container) => {
                self.members_of_container(container, FindSymbol::Any, None, false)
            }
            GetMembersInner::ConstAsSource => {
                let source = source_symbol(self.db(), import);
                self.members_of_table(
                    TableId::new(import, source.const_table),
                    FindSymbol::Any,
                    None,
                )
                .into_iter()
                .chain(self.members_of_table(
                    TableId::new(import, source.source_table),
                    FindSymbol::Any,
                    None,
                ))
                .collect()
            }
            GetMembersInner::Const => {
                let source = source_symbol(self.db(), import);
                self.members_of_table(
                    TableId::new(import, source.const_table),
                    FindSymbol::Any,
                    None,
                )
            }
            GetMembersInner::RootAsSource => {
                let source = source_symbol(self.db(), import);
                self.members_of_table(
                    TableId::new(import, source.root_table),
                    FindSymbol::Any,
                    None,
                )
                .into_iter()
                .chain(self.members_of_table(
                    TableId::new(import, source.source_table),
                    FindSymbol::Any,
                    None,
                ))
                .collect()
            }
            GetMembersInner::Root => {
                let source = source_symbol(self.db(), import);
                self.members_of_table(
                    TableId::new(import, source.root_table),
                    FindSymbol::Any,
                    None,
                )
            }
        };

        if let GetMembersInner::Container(container) = settings {
            let Some(imports) = self.imports().get(&container) else {
                return result;
            };

            for import in imports {
                if already_included.contains(import) {
                    continue;
                }

                already_included.insert(*import);
                result.extend(self.import_members_inner(*import, already_included, settings));
            }
            return result;
        }

        for imports in self.imports().values() {
            for import in imports {
                if !already_included.insert(*import) {
                    continue;
                }

                result.extend(self.import_members_inner(*import, already_included, settings));
            }
        }

        result
    }

    fn symbols_at(&self, offset: TextSize, filter_by_static: bool) -> Vec<SymbolId> {
        let mut taken_names = FxHashSet::default();
        let mut items = Vec::new();

        let mut filter = |(name, id)| {
            if taken_names.insert(name) {
                Some(id)
            } else {
                None
            }
        };

        let scope = self.arena().scope_at(offset);

        items.extend(
            self.local_members(offset)
                .into_iter()
                .filter_map(&mut filter),
        );

        items.extend(
            self.members_of_table(
                self.const_table(),
                FindSymbol::OnlyBefore(offset),
                Some(GetMembers::Const),
            )
            .into_iter()
            .filter_map(&mut filter),
        );

        let container = self.arena()[scope].container;

        items.extend(
            self.members_of_container(
                container,
                FindSymbol::BeforeIfInExecutionRange(offset),
                Some(GetMembers::Container(container)),
                filter_by_static,
            )
            .into_iter()
            .filter_map(&mut filter),
        );

        items.extend(
            self.members_of_table(
                self.root_table(),
                FindSymbol::BeforeIfInExecutionRange(offset),
                Some(GetMembers::Root),
            )
            .into_iter()
            .filter_map(&mut filter),
        );

        items
    }

    fn members_of_type(
        &self,
        typ: Type,
        settings: FindSymbol,
        allow_imports: bool,
        filter_by_static: bool,
    ) -> FlatSymbolTable {
        match typ {
            Type::Table(id) => self.members_of_table(
                id,
                settings,
                if allow_imports {
                    Some(GetMembers::Container(Container::Table(id)))
                } else {
                    None
                },
            ),
            Type::Class(id) => self.members_of_class(
                id,
                settings,
                if allow_imports {
                    Some(GetMembers::Container(Container::Class(id)))
                } else {
                    None
                },
                false,
                filter_by_static,
            ),
            Type::Instance(id) => self.members_of_class(
                id,
                settings,
                if allow_imports {
                    Some(GetMembers::Container(Container::Class(id)))
                } else {
                    None
                },
                true,
                filter_by_static,
            ),
            Type::Enum(id) => self.enum_members(id),
            Type::Integer(_) => integer_members(self.db()),
            Type::Float(_) => float_members(self.db()),
            Type::String(_) => string_members(self.db()),
            Type::Boolean(_) => boolean_members(self.db()),
            Type::Array(_) => array_members(self.db()),
            Type::Function(_) => function_members(self.db()),
            Type::Generator(_) => generator_members(self.db()),
            Type::Thread(_) => thread_members(self.db()),
            Type::Weakref => weakref_members(self.db()),
            Type::Unknown => FlatSymbolTable::default(),
            Type::Null => FlatSymbolTable::default(),
        }
    }

    fn members_of_container(
        &self,
        container: Container,
        settings: FindSymbol,
        imports: Option<GetMembers>,
        filter_by_static: bool,
    ) -> FlatSymbolTable {
        match container {
            Container::Table(id) => self.members_of_table(id, settings, imports),
            Container::Class(id) => {
                self.members_of_class(id, settings, imports, false, filter_by_static)
            }
            Container::Instance(id) => {
                self.members_of_class(id, settings, imports, true, filter_by_static)
            }
            Container::Enum(id) => self.enum_members(id),
        }
    }

    fn members_of_table(
        &self,
        table: TableId,
        settings: FindSymbol,
        imports: Option<GetMembers>,
    ) -> FlatSymbolTable {
        let mut members = match imports {
            None | Some(GetMembers::Const) => {
                // This incorrectly calls table builtin methods on const instead of the root when in name
                // expression, e.g. `keys()`. It also breaks redefinition of builtin methods, e.g.
                // `class keys {}` which should normally work
                FlatSymbolTable::default()
            }
            _ => builtin_table_members(self.db()),
        };

        let additional: FlatSymbolTable = match settings {
            FindSymbol::Any => to_flat_symbol_table(self.table_members(table)),
            FindSymbol::OnlyBefore(offset) => self
                .table_members(table)
                .into_iter()
                .filter_map(|(name, ids)| {
                    let mut last = None;
                    for id in ids {
                        let symbol = self.get(id);
                        if symbol.range.end() >= offset {
                            break;
                        }

                        last = Some(id)
                    }
                    Some((name, last?))
                })
                .collect(),
            FindSymbol::BeforeIfInExecutionRange(offset) => {
                let scope = self.scope(offset);
                let execution_range = self.arena()[scope].execution_range;
                self.table_members(table)
                    .into_iter()
                    .filter_map(|(name, ids)| {
                        let mut last = None;
                        for id in ids {
                            let symbol = self.get(id);
                            if execution_range.contains_range(symbol.range)
                                && symbol.range.end() >= offset
                            {
                                break;
                            }

                            last = Some(id)
                        }

                        Some((name, last?))
                    })
                    .collect()
            }
        };

        if let Some(imports) = imports {
            let imports = self.import_members(imports);
            for (k, v) in imports {
                members.insert(k, v);
            }
        }

        for (k, v) in additional {
            members.insert(k, v);
        }

        members
    }

    fn members_of_class(
        &self,
        class: ClassId,
        settings: FindSymbol,
        imports: Option<GetMembers>,
        for_instance: bool,
        filter_by_static: bool,
    ) -> FlatSymbolTable {
        let mut members = if for_instance {
            instance_members(self.db())
        } else {
            builtin_class_members(self.db())
        };

        let additional: FlatSymbolTable = match settings {
            FindSymbol::Any => to_flat_symbol_table(self.class_members(class)),
            FindSymbol::OnlyBefore(offset) => self
                .class_members(class)
                .into_iter()
                .filter_map(|(name, ids)| {
                    let mut last = None;
                    for id in ids {
                        let symbol = self.get(id);
                        if symbol.range.end() >= offset {
                            break;
                        }

                        last = Some(id)
                    }
                    Some((name, last?))
                })
                .collect(),
            FindSymbol::BeforeIfInExecutionRange(offset) => {
                let scope = self.scope(offset);
                let execution_range = self.arena()[scope].execution_range;
                self.class_members(class)
                    .into_iter()
                    .filter_map(|(name, ids)| {
                        let mut last = None;
                        for id in ids {
                            let symbol = self.get(id);
                            if execution_range.contains_range(symbol.range)
                                && symbol.range.end() >= offset
                            {
                                break;
                            }

                            last = Some(id)
                        }

                        Some((name, last?))
                    })
                    .collect()
            }
        };

        if let Some(imports) = imports {
            let imports = self.import_members(imports);
            for (k, v) in imports {
                members.insert(k, v);
            }
        }

        if filter_by_static {
            let statics: Vec<_> = if for_instance {
                additional
                    .iter()
                    .filter_map(|(name, id)| {
                        let symbol = self.get(*id);
                        if matches!(symbol.statik, Static::Yes) {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                additional
                    .iter()
                    .filter_map(|(name, id)| {
                        let symbol = self.get(*id);
                        if matches!(symbol.statik, Static::No) {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            // Just not overwriting the 'members' with symbols that don't pass the filter is not enough
            // Internally the name will be overwritten, however we can avoid showing this name to the user
            // instead of misleadingly showing the default member
            for (k, v) in additional {
                members.insert(k, v);
            }

            for name in statics {
                members.remove(&name);
            }
        } else {
            for (k, v) in additional {
                members.insert(k, v);
            }
        }

        members
    }

    fn clone_members(&self, superclass: ClassId) -> SymbolTable {
        let symbol = self.get(superclass);
        let members = symbol.members.clone();

        members
    }

    /// The vector is in order of symbols being added, therefore the last symbol that passes the condition
    /// must be the symbol we're looking for
    fn get_symbol(&self, table: &SymbolTable, name: &str, offset: TextSize) -> Option<&Symbol> {
        let symbols = table.get(name)?;
        let execution_range = self.arena()[self.scope(offset)].execution_range;
        let mut last = None;
        for id in symbols {
            let symbol = self.get(*id);
            if execution_range.contains_range(symbol.range) && symbol.range.end() > offset {
                break;
            }

            last = Some(symbol)
        }
        last
    }

    fn to_flat_symbol_table_context_aware(
        &self,
        table: SymbolTable,
        offset: TextSize,
    ) -> FlatSymbolTable {
        let execution_range = self.arena()[self.scope(offset)].execution_range;
        let mut result = FxHashMap::default();

        for (name, symbols) in table.into_iter() {
            let mut last = None;

            for id in symbols {
                let symbol = self.get(id);

                // same filtering logic
                if execution_range.contains_range(symbol.range) && symbol.range.end() > offset {
                    break;
                }

                last = Some(id);
            }

            if let Some(id) = last {
                result.insert(name, id);
            }
        }

        result
    }

    fn expr_to_type(&self, expr: NullableExprKind) -> Type {
        match expr {
            Some(ExpressionKind::Literal(typ)) => typ,
            Some(ExpressionKind::Symbol(symbol)) => self.get(symbol).typ,
            None => Type::Unknown,
        }
    }

    fn expr_kind_at(&self, range: TextRange) -> NullableExprKind {
        self.expr_kinds().get(&range).cloned()
    }

    fn symbol_at(&self, range: TextRange) -> Option<SymbolId> {
        self.name_kinds().get(&range).cloned()
    }

    fn type_at(&self, text_range: TextRange) -> Type {
        self.expr_to_type(self.expr_kind_at(text_range))
    }
}

pub struct FinishedFile<'db>(&'db dyn Db, File);

impl<'db> FinishedFile<'db> {
    pub fn new(db: &'db dyn Db, file: File) -> FinishedFile<'db> {
        Self(db, file)
    }

    fn source(&self) -> &SourceSymbol {
        source_symbol(self.0, self.1)
    }
}

impl<'db> Source for FinishedFile<'db> {
    fn arena(&self) -> &SourceArena {
        &self.source().arena
    }

    fn get<T>(&self, id: T) -> &T::Data
    where
        T: ArenaId,
        SourceArena: std::ops::Index<Idx<T::Data>, Output = T::Data>,
    {
        id.get_data(self.0)
    }

    fn file(&self) -> File {
        self.1
    }

    fn db(&self) -> &dyn Db {
        self.0
    }

    fn imports(&self) -> &FxHashMap<Container, Vec<File>> {
        &self.source().imports
    }

    fn scope(&self, offset: TextSize) -> ScopeId {
        self.source().arena.scope_at(offset)
    }

    fn root_table(&self) -> TableId {
        TableId::new(self.1, self.source().root_table)
    }

    fn const_table(&self) -> TableId {
        TableId::new(self.1, self.source().const_table)
    }

    fn expr_kinds(&self) -> &FxHashMap<TextRange, ExpressionKind> {
        &self.source().expr_kinds
    }

    fn name_kinds(&self) -> &FxHashMap<TextRange, SymbolId> {
        &self.source().name_kinds
    }

    fn diagnostics(&self) -> &[Diagnostic] {
        &self.source().diagnostics
    }
}

#[derive(Debug, PartialEq)]
pub struct SourceSymbol {
    imports: FxHashMap<Container, Vec<File>>,

    arena: SourceArena,
    const_table: Idx<TableData>,
    root_table: Idx<TableData>,
    source_table: Idx<TableData>,

    expr_kinds: FxHashMap<TextRange, ExpressionKind>,
    name_kinds: FxHashMap<TextRange, SymbolId>,
    diagnostics: Vec<Diagnostic>,
}
