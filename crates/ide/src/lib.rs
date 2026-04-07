mod arena;
mod collector;
mod db;
mod doc;
mod symbol;

use std::collections::hash_map::Entry;

use la_arena::Idx;
use rustc_hash::{FxHashMap, FxHashSet};
use sq_3_parser::{SyntaxKind, SyntaxToken, TextRange, TextSize};

use crate::{
    arena::{ClassId, Container, EnumId, FunctionId, ScopeId, SourceArena, TableData, TableId},
    db::Db,
    symbol::{FlatSymbolTable, Static, to_flat_symbol_table},
};

pub use arena::{ArenaId, FunctionData, ParamsState, SymbolId};
pub use db::{Database, DbConfig, File, line_index, parse, source_symbol};
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
    Information,
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
pub enum ImportMembers {
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
    OnlyBefore(TextSize),
    BeforeIfInExecutionRange(TextSize),
}

#[derive(Debug)]
pub enum FunctionIdResolution {
    Function(FunctionId),
    DefaultConstructor,
}

#[derive(Debug)]
pub enum FileTableType {
    Root,
    Const,
    Source,
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
    fn source_table(&self) -> TableId;
    fn root_table(&self) -> TableId;
    fn const_table(&self) -> TableId;
    fn range_to_expr(&self) -> &FxHashMap<TextRange, ExpressionKind>;
    fn range_to_symbol(&self) -> &FxHashMap<TextRange, SymbolId>;
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

    fn import_members(&self, settings: ImportMembers) -> FlatSymbolTable {
        let mut already_included = FxHashSet::default();
        already_included.insert(self.file());
        let mut result = FlatSymbolTable::default();
        // If we ask for root symbols but imports contains symbols for root container
        // how to not iterate the files twice?
        // include source_table + root_table where it's the import goes in the root table
        // include just root_table where it's not put in the root table?
        // First iterate through possi
        let (first_settings, second_settings, container) = match settings {
            ImportMembers::Container(container) => (
                GetMembersInner::Container(container),
                None,
                // Fix later
                match container {
                    Container::Instance(id) => Container::Class(id),
                    _ => container,
                },
            ),
            ImportMembers::Const => (
                GetMembersInner::ConstAsSource,
                Some(GetMembersInner::Const),
                Container::Table(self.const_table()),
            ),
            ImportMembers::Root => (
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
        let imp = FinishedFile::new(self.db(), import);
        let mut result = match settings {
            GetMembersInner::Container(_) => imp.top_level_members(FileTableType::Source),
            GetMembersInner::ConstAsSource => imp
                .top_level_members(FileTableType::Const)
                .into_iter()
                .chain(imp.top_level_members(FileTableType::Source))
                .collect(),
            GetMembersInner::Const => imp.top_level_members(FileTableType::Const),
            GetMembersInner::RootAsSource => imp
                .top_level_members(FileTableType::Root)
                .into_iter()
                .chain(imp.top_level_members(FileTableType::Source))
                .collect(),
            GetMembersInner::Root => imp.top_level_members(FileTableType::Root),
        };

        if let GetMembersInner::Container(container) = settings {
            let Some(imports) = imp.imports().get(&container) else {
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

        for imports in imp.imports().values() {
            for import in imports {
                if !already_included.insert(*import) {
                    continue;
                }

                result.extend(self.import_members_inner(*import, already_included, settings));
            }
        }

        result
    }

    fn get_scope_execution_range(&self, scope: ScopeId) -> TextRange {
        if let Some(idx) = self.arena()[scope].function {
            let function = &self.arena()[idx];
            function.range
        } else {
            // Possibly slow?
            let parse = parse(self.db(), self.file());
            parse.syntax().text_range()
        }
    }

    fn get_scope_container(&self, scope: ScopeId) -> Container {
        if let Some(idx) = self.arena()[scope].function {
            let function = &self.arena()[idx];
            function.bindenv.unwrap_or(function.container)
        } else {
            Container::Table(self.source_table())
        }
    }

    fn symbols_at(&self, offset: TextSize, filter_by_static: bool) -> FlatSymbolTable {
        let mut items = FlatSymbolTable::default();

        let scope = self.scope(offset);

        for (name, id) in self.local_members(offset) {
            items.entry(name).or_insert(id);
        }

        for (name, id) in self.members_of_table(
            self.const_table(),
            FindSymbol::OnlyBefore(offset),
            ImportMembers::Const,
        ) {
            items.entry(name).or_insert(id);
        }

        let container = self.get_scope_container(scope);
        dbg!(container);

        for (name, id) in self.members_of_container(
            container,
            FindSymbol::BeforeIfInExecutionRange(offset),
            filter_by_static,
        ) {
            items.entry(name).or_insert(id);
        }

        for (name, id) in self.members_of_table(
            self.root_table(),
            FindSymbol::BeforeIfInExecutionRange(offset),
            ImportMembers::Root,
        ) {
            items.entry(name).or_insert(id);
        }

        items
    }

    fn members_of_type(
        &self,
        typ: Type,
        settings: FindSymbol,
        filter_by_static: bool,
    ) -> FlatSymbolTable {
        match typ {
            Type::Table(id) => {
                self.members_of_table(id, settings, ImportMembers::Container(Container::Table(id)))
            }
            Type::Class(id) => self.members_of_class(
                id,
                settings,
                ImportMembers::Container(Container::Class(id)),
                false,
                filter_by_static,
            ),
            Type::Instance(id) => self.members_of_class(
                id,
                settings,
                ImportMembers::Container(Container::Class(id)),
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
        filter_by_static: bool,
    ) -> FlatSymbolTable {
        match container {
            Container::Table(id) => {
                self.members_of_table(id, settings, ImportMembers::Container(container))
            }
            Container::Class(id) => self.members_of_class(
                id,
                settings,
                ImportMembers::Container(container),
                false,
                filter_by_static,
            ),
            Container::Instance(id) => self.members_of_class(
                id,
                settings,
                ImportMembers::Container(container),
                true,
                filter_by_static,
            ),
            Container::Enum(id) => self.enum_members(id),
        }
    }

    fn members_of_table(
        &self,
        table: TableId,
        settings: FindSymbol,
        imports: ImportMembers,
    ) -> FlatSymbolTable {
        let mut members = match imports {
            ImportMembers::Const => {
                // This incorrectly calls table builtin methods on const instead of the root when in name
                // expression, e.g. `keys()`. It also breaks redefinition of builtin methods, e.g.
                // `class keys {}` which should normally work
                FlatSymbolTable::default()
            }
            _ => builtin_table_members(self.db()),
        };

        let additional: FlatSymbolTable = match settings {
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
                let execution_range = self.get_scope_execution_range(scope);
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

        let imports = self.import_members(imports);
        for (k, v) in imports {
            members.insert(k, v);
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
        imports: ImportMembers,
        for_instance: bool,
        filter_by_static: bool,
    ) -> FlatSymbolTable {
        let mut members = if for_instance {
            instance_members(self.db())
        } else {
            builtin_class_members(self.db())
        };

        let additional: FlatSymbolTable = match settings {
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
                let execution_range = self.get_scope_execution_range(scope);
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

        let imports = self.import_members(imports);
        for (k, v) in imports {
            members.insert(k, v);
        }

        if filter_by_static {
            let avoid_static = if for_instance {
                Static::Yes
            } else {
                Static::No
            };

            // Just not overwriting the 'members' with symbols that don't pass the filter is not enough
            // Internally the name will be overwritten, but if we don't overwrite it on our side we will
            // show a misleading symbol. Instead we can avoid showing these names to the user by removing
            // them from the table
            for (name, id) in additional {
                let symbol = self.get(id);
                if let SymbolKind::Property(statik) = symbol.kind
                    && statik == avoid_static
                {
                    members.remove(&name);
                } else {
                    members.insert(name, id);
                }
            }

            members.remove("constructor");
        } else {
            for (name, id) in additional {
                members.insert(name, id);
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
        let execution_range = self.get_scope_execution_range(self.scope(offset));
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

    fn expr_to_type(&self, expr: NullableExprKind) -> Type {
        match expr {
            Some(ExpressionKind::Literal(typ)) => typ,
            Some(ExpressionKind::Symbol(symbol)) => self.get(symbol).typ,
            None => Type::Unknown,
        }
    }

    fn expr_at(&self, range: TextRange) -> NullableExprKind {
        self.range_to_expr().get(&range).cloned()
    }

    fn symbol_at(&self, token: &SyntaxToken) -> Option<SymbolId> {
        let token_range = token.text_range();
        let range = match token.kind() {
            SyntaxKind::String => {
                let text = token.text();
                let left = if text.starts_with('"') { 1 } else { 0 };
                let right = if text.ends_with('"') { 1 } else { 0 };

                TextRange::new(
                    token_range.start() + TextSize::new(left),
                    token_range.end() - TextSize::new(right),
                )
            }
            SyntaxKind::VerbatimString => {
                let text = token.text();
                let left = if text.starts_with("@\"") { 2 } else { 0 };
                let right = if text.ends_with('"') { 1 } else { 0 };

                TextRange::new(
                    token_range.start() + TextSize::new(left),
                    token_range.end() - TextSize::new(right),
                )
            }
            _ => token_range,
        };
        self.range_to_symbol().get(&range).cloned()
    }

    fn type_at(&self, text_range: TextRange) -> Type {
        self.expr_to_type(self.expr_at(text_range))
    }
}

pub struct FinishedFile<'db>(&'db dyn Db, File);

impl<'db> FinishedFile<'db> {
    pub fn new(db: &'db dyn Db, file: File) -> FinishedFile<'db> {
        Self(db, file)
    }

    fn top_level_members(&self, from: FileTableType) -> FlatSymbolTable {
        let source = self.source();
        let table = match from {
            FileTableType::Root => source.root_table,
            FileTableType::Const => source.const_table,
            FileTableType::Source => source.source_table,
        };

        to_flat_symbol_table(self.table_members(TableId::new(self.1, table)))
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

    fn source_table(&self) -> TableId {
        TableId::new(self.1, self.source().source_table)
    }

    fn root_table(&self) -> TableId {
        TableId::new(self.1, self.source().root_table)
    }

    fn const_table(&self) -> TableId {
        TableId::new(self.1, self.source().const_table)
    }

    fn range_to_expr(&self) -> &FxHashMap<TextRange, ExpressionKind> {
        &self.source().range_to_expr
    }

    fn range_to_symbol(&self) -> &FxHashMap<TextRange, SymbolId> {
        &self.source().range_to_symbol
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

    range_to_expr: FxHashMap<TextRange, ExpressionKind>,
    range_to_symbol: FxHashMap<TextRange, SymbolId>,
    diagnostics: Vec<Diagnostic>,
}
