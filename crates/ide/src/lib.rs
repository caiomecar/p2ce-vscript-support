mod arena;
mod collector;
mod db;
// mod doc;
mod symbol;

use la_arena::Idx;
use rustc_hash::{FxHashMap, FxHashSet};
use sq_3_parser::{TextRange, TextSize};

use crate::{
    arena::{ClassId, Container, EnumId, SourceArena, SymbolId, TableData, TableId},
    collector::Collector,
    db::Db,
};

pub use arena::ArenaId;
pub use db::{Database, File, line_index, parse, source_symbol};
pub use symbol::{Symbol, SymbolKind, SymbolTable, Type};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpressionKind {
    // L value
    Literal(Type),
    // R value
    Symbol(SymbolId),
}

pub type NullableExprKind = Option<ExpressionKind>;

macro_rules! non_container_members {
    ($func:ident => $ty:ident) => {
        fn $func(db: &dyn Db) -> SymbolTable {
            if let Some(builtins) = db.builtins().map(|b| &b.$ty) {
                builtins.clone()
            } else {
                SymbolTable::default()
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
non_container_members!(instance_members => instance);
non_container_members!(builtin_table_members => table);
non_container_members!(builtin_class_members => class);

pub fn builtin_type_members(db: &dyn Db, typ: Type) -> SymbolTable {
    match typ {
        Type::Integer(_) => integer_members(db),
        Type::Float(_) => float_members(db),
        Type::String(_) => string_members(db),
        Type::Boolean(_) => boolean_members(db),
        Type::Instance(_) => instance_members(db),
        Type::Array(_) => array_members(db),
        Type::Table(_) => builtin_table_members(db),
        Type::Class(_) => builtin_class_members(db),
        Type::Enum(_) => SymbolTable::default(),
        Type::Function(_) => function_members(db),
        Type::Generator(_) => generator_members(db),
        Type::Thread(_) => thread_members(db),
        Type::Weakref => weakref_members(db),
        Type::Unknown => SymbolTable::default(),
        Type::Null => SymbolTable::default(),
    }
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

pub enum FindSymbol {
    Any,
    OnlyBefore(TextSize),
    BeforeIfInExecutionRange(TextSize),
}

#[derive(Clone, Copy)]
pub enum FileState<'db> {
    // We cannot use .source_symbol on a file that wasn't
    // finished otherwise we get cycle
    InProcess(&'db Collector<'db>),
    Finished(&'db dyn Db, File),
}

impl<'db> FileState<'db> {
    pub fn get<T>(&self, id: T) -> &'db T::Data
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
            FileState::InProcess(collector) => collector.file(),
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
            FileState::InProcess(collector) => collector.db(),
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

    fn local_members(&self, offset: TextSize) -> SymbolTable {
        let mut scope = Some(match self {
            FileState::InProcess(collector) => collector.scope(),
            FileState::Finished(db, file) => source_symbol(*db, *file).arena().scope_at(offset),
        });

        let mut result = SymbolTable::default();
        while let Some(current_scope) = scope {
            let new = self.arena()[current_scope].locals.clone();
            for (k, v) in new {
                if self.get(v).range.end() >= offset {
                    continue;
                };
                result.entry(k).or_insert(v);
            }
            scope = self.arena()[current_scope].parent;
        }

        result
    }

    fn root_table(&self) -> TableId {
        match self {
            FileState::InProcess(collector) => {
                TableId::new(collector.file(), collector.root_table())
            }
            FileState::Finished(db, file) => {
                let source_symbol = source_symbol(*db, *file);
                TableId::new(*file, source_symbol.root_table())
            }
        }
    }

    fn const_table(&self) -> TableId {
        match self {
            FileState::InProcess(collector) => {
                TableId::new(collector.file(), collector.const_table())
            }
            FileState::Finished(db, file) => {
                let source_symbol = source_symbol(*db, *file);
                TableId::new(*file, source_symbol.const_table())
            }
        }
    }

    fn table_members(&self, table: TableId) -> SymbolTable {
        let table = self.get(table);
        let mut result = table.members.clone();

        if let Some(delegate) = table.delegate {
            for (k, v) in &self.get(delegate).members {
                result.entry(k.clone()).or_insert(*v);
            }
        }

        result
    }

    fn enum_members(&self, enum_: EnumId) -> SymbolTable {
        let enum_ = self.get(enum_);
        enum_.members.clone()
    }

    fn class_members(&self, class: ClassId) -> SymbolTable {
        let class = self.get(class);
        class.members.clone()
    }

    fn import_members(&self, settings: GetMembers) -> SymbolTable {
        let mut already_included = FxHashSet::default();
        already_included.insert(self.file());
        let mut result = SymbolTable::default();
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
    ) -> SymbolTable {
        let mut result = match settings {
            GetMembersInner::Container(container) => {
                self.members_of_container(container, FindSymbol::Any, None)
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

    pub fn symbols_at(&self, offset: TextSize) -> Vec<SymbolId> {
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

    pub fn members_of_type(&self, typ: Type, settings: FindSymbol) -> SymbolTable {
        match typ {
            Type::Table(id) => self.members_of_table(
                id,
                settings,
                Some(GetMembers::Container(Container::Table(id))),
            ),
            Type::Class(id) => self.members_of_class(
                id,
                settings,
                Some(GetMembers::Container(Container::Class(id))),
            ),
            Type::Instance(id) => self.members_of_class(
                id,
                settings,
                Some(GetMembers::Container(Container::Class(id))),
            ),
            Type::Enum(id) => self.enum_members(id),
            _ => builtin_type_members(self.db(), typ),
        }
    }

    fn members_of_container(
        &self,
        container: Container,
        settings: FindSymbol,
        imports: Option<GetMembers>,
    ) -> SymbolTable {
        match container {
            Container::Table(id) => self.members_of_table(id, settings, imports),
            Container::Class(id) => self.members_of_class(id, settings, imports),
            Container::Enum(id) => self.enum_members(id),
        }
    }

    fn members_of_table(
        &self,
        table: TableId,
        settings: FindSymbol,
        imports: Option<GetMembers>,
    ) -> SymbolTable {
        let mut members = builtin_table_members(self.db());

        let additional = match settings {
            FindSymbol::Any => self.table_members(table),
            FindSymbol::OnlyBefore(offset) => self
                .table_members(table)
                .into_iter()
                .filter(|(_, id)| {
                    let symbol = self.get(*id);
                    symbol.range.end() < offset
                })
                .collect(),
            FindSymbol::BeforeIfInExecutionRange(offset) => {
                let scope = match self {
                    FileState::InProcess(collector) => collector.scope(),
                    FileState::Finished(db, file) => {
                        source_symbol(*db, *file).arena().scope_at(offset)
                    }
                };
                let execution_range = self.arena()[scope].execution_range;
                self.table_members(table)
                    .into_iter()
                    .filter(|(_, id)| {
                        let symbol = self.get(*id);
                        !execution_range.contains_range(symbol.range) || symbol.range.end() < offset
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
    ) -> SymbolTable {
        let mut members = builtin_table_members(self.db());

        let additional = match settings {
            FindSymbol::Any => self.class_members(class),
            FindSymbol::OnlyBefore(offset) => self
                .class_members(class)
                .into_iter()
                .filter(|(_, id)| {
                    let symbol = self.get(*id);
                    symbol.range.end() < offset
                })
                .collect(),
            FindSymbol::BeforeIfInExecutionRange(offset) => {
                let scope = match self {
                    FileState::InProcess(collector) => collector.scope(),
                    FileState::Finished(db, file) => {
                        source_symbol(*db, *file).arena().scope_at(offset)
                    }
                };
                let execution_range = self.arena()[scope].execution_range;
                self.class_members(class)
                    .into_iter()
                    .filter(|(_, id)| {
                        let symbol = self.get(*id);
                        !execution_range.contains_range(symbol.range) || symbol.range.end() < offset
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

    pub fn expr_to_type(&self, expr: NullableExprKind) -> Type {
        match expr {
            Some(ExpressionKind::Literal(typ)) => typ,
            Some(ExpressionKind::Symbol(symbol)) => self.get(symbol).typ,
            None => Type::Unknown,
        }
    }

    pub fn expr_kinds(&self) -> &FxHashMap<TextRange, ExpressionKind> {
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

    pub fn name_kinds(&self) -> &FxHashMap<TextRange, SymbolId> {
        match self {
            FileState::InProcess(collector) => collector.name_kinds(),
            FileState::Finished(db, file) => {
                let source = source_symbol(*db, *file);
                source.name_kinds()
            }
        }
    }

    pub fn symbol_at(&self, range: TextRange) -> Option<SymbolId> {
        self.name_kinds().get(&range).cloned()
    }

    pub fn type_at(&self, text_range: TextRange) -> Type {
        self.expr_to_type(self.expr_kind_at(text_range))
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

pub trait SourceSymbolic {
    fn imports(&self) -> &FxHashMap<Container, Vec<File>>;

    fn arena(&self) -> &SourceArena;

    fn const_table(&self) -> Idx<TableData>;
    fn root_table(&self) -> Idx<TableData>;

    fn expr_kinds(&self) -> &FxHashMap<TextRange, ExpressionKind>;
    fn name_kinds(&self) -> &FxHashMap<TextRange, SymbolId>;
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
