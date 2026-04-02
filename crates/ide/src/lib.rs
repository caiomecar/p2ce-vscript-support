mod arena;
mod collector;
mod db;
// mod doc;
mod symbol;

use crate::{
    arena::{
        ArenaId, ClassId, Container, EnumId, Scope, SourceArena, SymbolId, TableData, TableId,
    },
    collector::{Collector, ExpressionKind, NullableExprKind, RangeExprKindMap},
    db::Db,
};
pub use db::{Database, File, line_index, parse, source_symbol};
use la_arena::Idx;
use rustc_hash::{FxHashMap, FxHashSet};
use sq_3_parser::{TextRange, TextSize};
pub use symbol::{Symbol, SymbolKind, Type};

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

macro_rules! non_container_members {
    ($func:ident => $ty:ident) => {
        fn $func(db: &dyn Db) -> Vec<SymbolId> {
            if let Some(builtins) = db.builtins().map(|b| &b.$ty) {
                builtins.values().cloned().collect()
            } else {
                Vec::new()
            }
        }
    };
}

non_container_members!(integer_members => integer);
non_container_members!(float_members => float);
non_container_members!(boolean_members => boolean);
non_container_members!(string_members => string);
non_container_members!(array_members => array);
non_container_members!(function_members => function);
non_container_members!(generator_members => generator);
non_container_members!(thread_members => thread);
non_container_members!(weakref_members => weakref);

fn table_members(file_state: FileState, table: TableId) -> Vec<SymbolId> {
    let table = file_state.get(table);
    let mut result = table.members.clone();

    if let Some(delegate) = table.delegate {
        for (k, v) in &file_state.get(delegate).members {
            result.entry(k.clone()).or_insert(*v);
        }
    }

    if let Some(builtins) = file_state.db().builtins().map(|b| &b.table) {
        for (k, v) in builtins {
            result.entry(k.clone()).or_insert(*v);
        }
    }

    result.values().cloned().collect()
}

fn enum_members(file_state: FileState, enum_: EnumId) -> Vec<SymbolId> {
    let enum_ = file_state.get(enum_);
    enum_.members.values().cloned().collect()
}

fn class_members(file_state: FileState, class: ClassId) -> Vec<SymbolId> {
    let class = file_state.get(class);
    let mut result = class.members.clone();
    if let Some(builtins) = file_state.db().builtins().map(|b| &b.class) {
        for (k, v) in builtins {
            result.entry(k.clone()).or_insert(*v);
        }
    }
    result.values().cloned().collect()
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GetMembers {
    Container(Container),
    Const,
    Root,
}

#[derive(Debug, Clone, Copy)]
enum GetMembersInner {
    Container(Container),
    RootAsSource,
    Root,
    ConstAsSource,
    Const,
}

enum FindSymbol {
    Any,
    OnlyBefore(TextSize),
    BeforeIfInExecutionRange(TextSize, TextRange),
}

#[derive(Clone, Copy)]
pub enum FileState<'db> {
    // We cannot use .source_symbol on a file that wasn't
    // finished otherwise we get cycle
    InProcess(&'db Collector<'db>),
    Finished(&'db dyn Db, File),
}

impl<'db> FileState<'db> {
    fn get<T>(&self, id: T) -> &'db T::Data
    where
        T: ArenaId,
        SourceArena: std::ops::Index<Idx<T::Data>, Output = T::Data>,
    {
        match self {
            FileState::InProcess(collector) => collector.get(id),
            FileState::Finished(db, _) => id.get_data(*db),
        }
    }

    fn file(&self) -> File {
        match self {
            FileState::InProcess(collector) => collector.file,
            FileState::Finished(_, file) => *file,
        }
    }

    fn arena(&self) -> &SourceArena {
        match self {
            FileState::InProcess(collector) => collector.arena(),
            FileState::Finished(db, file) => source_symbol(*db, *file).arena(),
        }
    }

    fn db(&self) -> &dyn Db {
        match self {
            FileState::InProcess(collector) => collector.db,
            FileState::Finished(db, _) => *db,
        }
    }

    fn imports(&self) -> &FxHashMap<Container, Vec<File>> {
        match self {
            FileState::InProcess(collector) => collector.imports(),
            FileState::Finished(db, file) => {
                let source_symbol = source_symbol(*db, *file);
                source_symbol.imports()
            }
        }
    }

    fn local_members(&self, mut scope: Idx<Scope>) -> Vec<SymbolId> {
        let mut result = self.arena()[scope].locals.clone();
        while let Some(parent) = self.arena()[scope].parent {
            for (k, v) in &self.arena()[parent].locals {
                result.entry(k.clone()).or_insert(*v);
            }
            scope = parent;
        }
        result.values().cloned().collect()
    }

    fn root_table_of(&self, file: File) -> TableId {
        match self {
            FileState::InProcess(collector) if collector.file == file => {
                TableId::new(collector.file, collector.root_table())
            }
            FileState::InProcess(collector) => {
                let source_symbol = source_symbol(collector.db, file);
                TableId::new(file, source_symbol.root_table())
            }
            FileState::Finished(db, _) => {
                let source_symbol = source_symbol(*db, file);
                TableId::new(file, source_symbol.root_table())
            }
        }
    }

    fn root_members_of(&self, file: File) -> Vec<SymbolId> {
        table_members(*self, self.root_table_of(file))
    }

    fn root_table(&self) -> TableId {
        match self {
            FileState::InProcess(collector) => TableId::new(collector.file, collector.root_table()),
            FileState::Finished(db, file) => {
                let source_symbol = source_symbol(*db, *file);
                TableId::new(*file, source_symbol.root_table())
            }
        }
    }

    fn root_members(&self) -> Vec<SymbolId> {
        table_members(*self, self.root_table())
    }

    fn const_table_of(&self, file: File) -> TableId {
        match self {
            FileState::InProcess(collector) if collector.file == file => {
                TableId::new(collector.file, collector.const_table())
            }
            FileState::InProcess(collector) => {
                let source_symbol = source_symbol(collector.db, file);
                TableId::new(file, source_symbol.const_table())
            }
            FileState::Finished(db, _) => {
                let source_symbol = source_symbol(*db, file);
                TableId::new(file, source_symbol.const_table())
            }
        }
    }

    fn const_members_of(&self, file: File) -> Vec<SymbolId> {
        table_members(*self, self.const_table_of(file))
    }

    fn const_table(&self) -> TableId {
        match self {
            FileState::InProcess(collector) => {
                TableId::new(collector.file, collector.const_table())
            }
            FileState::Finished(db, file) => {
                let source_symbol = source_symbol(*db, *file);
                TableId::new(*file, source_symbol.const_table())
            }
        }
    }

    fn const_members(&self) -> Vec<SymbolId> {
        table_members(*self, self.const_table())
    }

    fn source_table_of(&self, file: File) -> TableId {
        match self {
            FileState::InProcess(collector) if collector.file == file => {
                TableId::new(collector.file, collector.source_table())
            }
            FileState::InProcess(collector) => {
                let source_symbol = source_symbol(collector.db, file);
                TableId::new(file, source_symbol.source_table())
            }
            FileState::Finished(db, _) => {
                let source_symbol = source_symbol(*db, file);
                TableId::new(file, source_symbol.source_table())
            }
        }
    }

    fn source_members_of(&self, file: File) -> Vec<SymbolId> {
        table_members(*self, self.source_table_of(file))
    }

    fn container_members(&self, container: Container) -> Vec<SymbolId> {
        match container {
            Container::Table(id) => table_members(*self, id),
            Container::Class(id) => class_members(*self, id),
            Container::Enum(id) => enum_members(*self, id),
        }
    }

    fn import_members(&self, settings: GetMembers) -> Vec<SymbolId> {
        let mut already_included = FxHashSet::default();
        already_included.insert(self.file());
        let mut result = Vec::new();
        // If we ask for root symbols but imports contains symbols for root container
        // how to not iterate the files twice?
        // include source_table + root_table where it's the import goes in the root table
        // include just root_table where it's not put in the root table?
        // First iterate through possi
        let (first_settings, second_settings, container) = match settings {
            GetMembers::Container(container) => {
                (GetMembersInner::Container(container), None, container)
            }
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
    ) -> Vec<SymbolId> {
        let mut result = match settings {
            GetMembersInner::Container(container) => self.container_members(container),
            GetMembersInner::ConstAsSource => self
                .const_members_of(import)
                .into_iter()
                .chain(self.source_members_of(import))
                .collect(),
            GetMembersInner::Const => self.const_members_of(import),
            GetMembersInner::RootAsSource => self
                .root_members_of(import)
                .into_iter()
                .chain(self.root_members_of(import))
                .collect(),
            GetMembersInner::Root => self.root_members_of(import),
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

    fn filter(
        &self,
        iter: impl IntoIterator<Item = SymbolId>,
        taken_names: &mut FxHashSet<String>,
        settings: FindSymbol,
    ) -> Vec<Symbol> {
        match settings {
            FindSymbol::Any => iter
                .into_iter()
                .filter_map(|symbol| {
                    let symbol = self.get(symbol);
                    if !taken_names.insert(symbol.name.clone()) {
                        return None;
                    }
                    Some(symbol.clone())
                })
                .collect(),
            FindSymbol::OnlyBefore(offset) => iter
                .into_iter()
                .filter_map(|symbol| {
                    let symbol = self.get(symbol);
                    if symbol.range.end() >= offset || !taken_names.insert(symbol.name.clone()) {
                        return None;
                    }
                    Some(symbol.clone())
                })
                .collect(),
            FindSymbol::BeforeIfInExecutionRange(offset, execution_range) => iter
                .into_iter()
                .filter_map(|symbol| {
                    let symbol = self.get(symbol);
                    if execution_range.contains_range(symbol.range) && symbol.range.end() >= offset
                        || !taken_names.insert(symbol.name.clone())
                    {
                        return None;
                    }
                    Some(symbol.clone())
                })
                .collect(),
        }
    }

    pub fn symbols_at(&self, offset: TextSize) -> Vec<Symbol> {
        let mut taken_names = FxHashSet::default();
        let mut items = Vec::new();

        let scope = self.arena().scope_at(offset);

        items.extend(self.filter(
            self.local_members(scope),
            &mut taken_names,
            FindSymbol::OnlyBefore(offset),
        ));

        items.extend(self.filter(
            self.const_members(),
            &mut taken_names,
            FindSymbol::OnlyBefore(offset),
        ));

        items.extend(self.filter(
            self.import_members(GetMembers::Const),
            &mut taken_names,
            FindSymbol::Any,
        ));

        let execution_range = self.arena()[scope].execution_range;
        let container = self.arena()[scope].container;

        items.extend(self.filter(
            self.container_members(container),
            &mut taken_names,
            FindSymbol::BeforeIfInExecutionRange(offset, execution_range),
        ));

        items.extend(self.filter(
            self.import_members(GetMembers::Container(container)),
            &mut taken_names,
            FindSymbol::Any,
        ));

        items.extend(self.filter(
            self.root_members(),
            &mut taken_names,
            FindSymbol::BeforeIfInExecutionRange(offset, execution_range),
        ));

        items.extend(self.filter(
            self.import_members(GetMembers::Root),
            &mut taken_names,
            FindSymbol::Any,
        ));

        items
    }

    pub fn type_members(&self, typ: Type) -> Vec<SymbolId> {
        match typ {
            Type::Integer => integer_members(self.db()),
            Type::Float => float_members(self.db()),
            Type::String(_) => string_members(self.db()),
            Type::Boolean => boolean_members(self.db()),
            Type::Instance(id) => class_members(*self, id),
            Type::Array(_) => array_members(self.db()),
            Type::Table(id) => table_members(*self, id),
            Type::Class(id) => class_members(*self, id),
            Type::Enum(id) => enum_members(*self, id),
            Type::Function(_) => function_members(self.db()),
            Type::Generator(_) => generator_members(self.db()),
            Type::Thread(_) => thread_members(self.db()),
            Type::Weakref => weakref_members(self.db()),
            Type::Unknown => Vec::new(),
            Type::Null => Vec::new(),
        }
    }

    pub fn members_of_type(&self, typ: Type) -> Vec<Symbol> {
        let members = self.type_members(typ);
        members.into_iter().map(|id| self.get(id).clone()).collect()
    }

    pub fn expr_to_type(&self, expr: NullableExprKind) -> Type {
        match expr {
            Some(ExpressionKind::Literal(typ)) => typ,
            Some(ExpressionKind::Symbol(symbol)) => self.get(symbol).typ,
            None => Type::Unknown,
        }
    }

    pub fn expr_kinds(&self) -> &RangeExprKindMap {
        match self {
            FileState::InProcess(collector) => collector.expr_kinds(),
            FileState::Finished(db, file) => {
                let source = source_symbol(*db, *file);
                source.expr_kinds()
            }
        }
    }

    pub fn expr_kind_at(&self, range: TextRange) -> NullableExprKind {
        self.expr_kinds().get(&range).cloned()
    }

    pub fn type_at(&self, text_range: TextRange) -> Type {
        self.expr_to_type(self.expr_kind_at(text_range))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSymbol {
    imports: FxHashMap<Container, Vec<File>>,

    arena: SourceArena,
    const_table: Idx<TableData>,
    root_table: Idx<TableData>,
    source_table: Idx<TableData>,

    expr_kinds: RangeExprKindMap,
    diagnostics: Vec<Diagnostic>,
}

pub trait SourceSymbolic {
    fn imports(&self) -> &FxHashMap<Container, Vec<File>>;

    fn arena(&self) -> &SourceArena;

    fn const_table(&self) -> Idx<TableData>;
    fn root_table(&self) -> Idx<TableData>;
    fn source_table(&self) -> Idx<TableData>;

    fn expr_kinds(&self) -> &RangeExprKindMap;
    fn diagnostics(&self) -> &[Diagnostic];
}

impl SourceSymbolic for SourceSymbol {
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

    fn source_table(&self) -> Idx<TableData> {
        self.source_table
    }

    fn expr_kinds(&self) -> &RangeExprKindMap {
        &self.expr_kinds
    }

    fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
