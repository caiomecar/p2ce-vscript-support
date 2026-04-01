mod arena;
mod collector;
mod db;
// mod doc;
mod symbol;

use crate::{
    arena::{
        ArenaId, ClassId, Container, EnumId, Scope, SourceArena, SymbolId, TableData, TableId,
    },
    collector::{ExpressionKind, NullableExprKind, RangeExprKindMap},
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
non_container_members!(array_members => string);
non_container_members!(function_members => function);
non_container_members!(generator_members => generator);
non_container_members!(thread_members => generator);
non_container_members!(weakref_members => weakref);

fn table_members(db: &dyn Db, table: TableId) -> Vec<SymbolId> {
    let table = table.get_data(db);

    let mut result = table.members.clone();

    if let Some(delegate) = table.delegate {
        for (k, v) in &delegate.get_data(db).members {
            result.entry(k.clone()).or_insert(*v);
        }
    }

    if let Some(builtins) = db.builtins().map(|b| &b.table) {
        for (k, v) in builtins {
            result.entry(k.clone()).or_insert(*v);
        }
    }

    result.values().cloned().collect()
}

fn enum_members(db: &dyn Db, enum_: EnumId) -> Vec<SymbolId> {
    let enum_ = enum_.get_data(db);
    enum_.members.values().cloned().collect()
}

fn class_members(db: &dyn Db, class: ClassId) -> Vec<SymbolId> {
    let class = class.get_data(db);
    let mut result = class.members.clone();
    if let Some(builtins) = db.builtins().map(|b| &b.class) {
        for (k, v) in builtins {
            result.entry(k.clone()).or_insert(*v);
        }
    }
    result.values().cloned().collect()
}

#[derive(Debug, Clone, Copy)]
enum GetMembers {
    Container(Container),
    Const(Idx<TableData>),
    Root(Idx<TableData>),
}

#[derive(Debug, Clone, Copy)]
enum GetMembersInner {
    Container(Container),
    RootAsSource,
    Root,
    ConstAsSource,
    Const,
}

fn import_members(
    db: &dyn Db,
    file: File,
    imports: &FxHashMap<Container, Vec<File>>,
    settings: GetMembers,
) -> Vec<SymbolId> {
    let mut already_included = FxHashSet::default();
    already_included.insert(file);
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
        GetMembers::Const(idx) => (
            GetMembersInner::ConstAsSource,
            Some(GetMembersInner::Const),
            Container::Table(TableId::new(file, idx)),
        ),
        GetMembers::Root(idx) => (
            GetMembersInner::RootAsSource,
            Some(GetMembersInner::Root),
            Container::Table(TableId::new(file, idx)),
        ),
    };

    if let Some(imports) = imports.get(&container) {
        for import in imports {
            if !already_included.insert(*import) {
                continue;
            }

            result.extend(import_members_inner(
                db,
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

            result.extend(import_members_inner(
                db,
                *import,
                &mut already_included,
                second_settings,
            ));
        }
    }
    result
}

fn import_members_inner(
    db: &dyn Db,
    file: File,
    already_included: &mut FxHashSet<File>,
    settings: GetMembersInner,
) -> Vec<SymbolId> {
    let mut result = match settings {
        GetMembersInner::Container(container) => container_members(db, container),
        GetMembersInner::ConstAsSource => const_members(db, file)
            .into_iter()
            .chain(source_members(db, file))
            .collect(),
        GetMembersInner::Const => const_members(db, file),
        GetMembersInner::RootAsSource => root_members(db, file)
            .into_iter()
            .chain(source_members(db, file))
            .collect(),
        GetMembersInner::Root => root_members(db, file),
    };

    let source = source_symbol(db, file);
    if let GetMembersInner::Container(container) = settings {
        let Some(imports) = source.imports.get(&container) else {
            return result;
        };

        for import in imports {
            if already_included.contains(import) {
                continue;
            }

            already_included.insert(*import);
            result.extend(import_members_inner(
                db,
                *import,
                already_included,
                settings,
            ));
        }
        return result;
    }

    for imports in source.imports.values() {
        for import in imports {
            if !already_included.insert(*import) {
                continue;
            }

            result.extend(import_members_inner(
                db,
                *import,
                already_included,
                settings,
            ));
        }
    }

    result
}

enum FindSymbol {
    Any,
    OnlyBefore(TextSize),
    BeforeIfInExecutionRange(TextSize, TextRange),
}

fn filter(
    db: &dyn Db,
    iter: impl IntoIterator<Item = SymbolId>,
    taken_names: &mut FxHashSet<String>,
    settings: FindSymbol,
) -> Vec<Symbol> {
    match settings {
        FindSymbol::Any => iter
            .into_iter()
            .filter_map(|symbol| {
                let symbol = symbol.get_data(db);
                if !taken_names.insert(symbol.name.clone()) {
                    return None;
                }
                Some(symbol.clone())
            })
            .collect(),
        FindSymbol::OnlyBefore(offset) => iter
            .into_iter()
            .filter_map(|symbol| {
                let symbol = symbol.get_data(db);
                if symbol.range.end() >= offset || !taken_names.insert(symbol.name.clone()) {
                    return None;
                }
                Some(symbol.clone())
            })
            .collect(),
        FindSymbol::BeforeIfInExecutionRange(offset, execution_range) => iter
            .into_iter()
            .filter_map(|symbol| {
                let symbol = symbol.get_data(db);
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

pub fn symbols_at(db: &dyn Db, file: File, offset: TextSize) -> Vec<Symbol> {
    let source = source_symbol(db, file);
    let mut taken_names = FxHashSet::default();
    let mut items = Vec::new();

    let scope = source.arena.scope_at(offset);

    items.extend(filter(
        db,
        source.local_members(scope),
        &mut taken_names,
        FindSymbol::OnlyBefore(offset),
    ));

    items.extend(filter(
        db,
        const_members(db, file),
        &mut taken_names,
        FindSymbol::OnlyBefore(offset),
    ));

    items.extend(filter(
        db,
        import_members(
            db,
            file,
            &source.imports,
            GetMembers::Const(source.const_table),
        ),
        &mut taken_names,
        FindSymbol::Any,
    ));

    let execution_range = source.arena[scope].execution_range;
    let container = source.arena[scope].container;

    items.extend(filter(
        db,
        container_members(db, container),
        &mut taken_names,
        FindSymbol::BeforeIfInExecutionRange(offset, execution_range),
    ));

    items.extend(filter(
        db,
        import_members(db, file, &source.imports, GetMembers::Container(container)),
        &mut taken_names,
        FindSymbol::Any,
    ));

    items.extend(filter(
        db,
        root_members(db, file),
        &mut taken_names,
        FindSymbol::BeforeIfInExecutionRange(offset, execution_range),
    ));

    items.extend(filter(
        db,
        import_members(
            db,
            file,
            &source.imports,
            GetMembers::Root(source.root_table),
        ),
        &mut taken_names,
        FindSymbol::Any,
    ));

    items
}

pub fn type_members(db: &dyn Db, typ: Type) -> Vec<SymbolId> {
    match typ {
        Type::Integer => integer_members(db),
        Type::Float => float_members(db),
        Type::String(_) => string_members(db),
        Type::Boolean => boolean_members(db),
        Type::Instance(id) => class_members(db, id),
        Type::Array(_) => array_members(db),
        Type::Table(id) => table_members(db, id),
        Type::Class(id) => class_members(db, id),
        Type::Enum(id) => enum_members(db, id),
        Type::Function(_) => function_members(db),
        Type::Generator(_) => generator_members(db),
        Type::Thread(_) => thread_members(db),
        Type::Weakref => weakref_members(db),
        Type::Unknown => Vec::new(),
        Type::Null => Vec::new(),
    }
}

pub fn members_of_type(db: &dyn Db, typ: Type) -> Vec<Symbol> {
    let members = type_members(db, typ);
    members
        .into_iter()
        .map(|id| id.get_data(db).clone())
        .collect()
}

pub fn expr_to_type(db: &dyn Db, expr: NullableExprKind) -> Type {
    match expr {
        Some(ExpressionKind::Literal(typ)) => typ,
        Some(ExpressionKind::Symbol(symbol)) => symbol.get_data(db).typ,
        None => Type::Unknown,
    }
}

pub fn type_at(db: &dyn Db, file: File, text_range: TextRange) -> Type {
    let source = source_symbol(db, file);
    expr_to_type(db, source.expr_kind_at(text_range))
}

pub fn container_members(db: &dyn Db, container: Container) -> Vec<SymbolId> {
    match container {
        Container::Table(id) => table_members(db, id),
        Container::Class(id) => class_members(db, id),
        Container::Enum(id) => enum_members(db, id),
    }
}

pub fn root_members(db: &dyn Db, file: File) -> Vec<SymbolId> {
    let source = source_symbol(db, file);
    table_members(db, TableId::new(file, source.root_table))
}

pub fn const_members(db: &dyn Db, file: File) -> Vec<SymbolId> {
    let source = source_symbol(db, file);
    table_members(db, TableId::new(file, source.const_table))
}

pub fn source_members(db: &dyn Db, file: File) -> Vec<SymbolId> {
    let source = source_symbol(db, file);
    table_members(db, TableId::new(file, source.source_table))
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSymbol {
    imports: FxHashMap<Container, Vec<File>>,
    file: File,

    arena: SourceArena,
    const_table: Idx<TableData>,
    root_table: Idx<TableData>,
    source_table: Idx<TableData>,

    expr_kinds: RangeExprKindMap,
    diagnostics: Vec<Diagnostic>,
}

impl SourceSymbol {
    pub fn local_members(&self, mut scope: Idx<Scope>) -> Vec<SymbolId> {
        let mut result = self.arena[scope].locals.clone();
        while let Some(parent) = self.arena[scope].parent {
            for (k, v) in &self.arena[parent].locals {
                result.entry(k.clone()).or_insert(*v);
            }
            scope = parent;
        }
        result.values().cloned().collect()
    }

    pub fn expr_kind_at(&self, text_range: TextRange) -> Option<ExpressionKind> {
        self.expr_kinds.get(&text_range).cloned()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
