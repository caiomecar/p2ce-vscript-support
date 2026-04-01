mod arena;
mod collector;
mod db;
mod symbol;

use crate::{
    arena::{ArenaId, Container, SourceArena, SymbolId, TableData},
    collector::{ExpressionKind, NullableExprKind, RangeExprKindMap, Scope},
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

#[derive(Debug, PartialEq, Eq)]
pub struct SourceSymbol {
    imports: FxHashMap<Container, Vec<File>>,
    file: File,

    arena: SourceArena,
    const_table: Idx<TableData>,
    root_table: Idx<TableData>,
    source_table: Idx<TableData>,
    source_scope: Scope,

    expr_kinds: RangeExprKindMap,
    diagnostics: Vec<Diagnostic>,
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
fn get_import_members(
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
            Container::Table(idx),
        ),
        GetMembers::Root(idx) => (
            GetMembersInner::RootAsSource,
            Some(GetMembersInner::Root),
            Container::Table(idx),
        ),
    };

    if let Some(imports) = imports.get(&container) {
        for import in imports {
            if !already_included.insert(*import) {
                continue;
            }

            result.extend(get_import_members_inner(
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

            result.extend(get_import_members_inner(
                db,
                *import,
                &mut already_included,
                second_settings,
            ));
        }
    }
    result
}

fn get_import_members_inner(
    db: &dyn Db,
    file: File,
    already_included: &mut FxHashSet<File>,
    settings: GetMembersInner,
) -> Vec<SymbolId> {
    let source = source_symbol(db, file);

    let mut result = match settings {
        GetMembersInner::Container(container) => source.get_container_members(container),
        GetMembersInner::ConstAsSource => {
            let mut members = source.arena[source.const_table].get_members();
            members.extend(source.arena[source.source_table].get_members());
            members
        }
        GetMembersInner::Const => source.arena[source.const_table].get_members(),
        GetMembersInner::RootAsSource => {
            let mut members = source.arena[source.root_table].get_members();
            members.extend(source.arena[source.source_table].get_members());
            members
        }
        GetMembersInner::Root => source.arena[source.root_table].get_members(),
    };

    if let GetMembersInner::Container(container) = settings {
        let Some(imports) = source.imports.get(&container) else {
            return result;
        };

        for import in imports {
            if already_included.contains(import) {
                continue;
            }

            already_included.insert(*import);
            result.extend(get_import_members_inner(
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

            result.extend(get_import_members_inner(
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

pub fn get_symbols_at(db: &dyn Db, file: File, offset: TextSize) -> Vec<Symbol> {
    let source = source_symbol(db, file);
    let mut taken_names = FxHashSet::default();
    let mut items = Vec::new();

    let stack = source.source_scope.stack_at(offset);

    items.extend(filter(
        db,
        stack
            .iter()
            .rev()
            .flat_map(|scope| scope.locals.values().copied()),
        &mut taken_names,
        FindSymbol::OnlyBefore(offset),
    ));

    items.extend(filter(
        db,
        source.get_const_members(),
        &mut taken_names,
        FindSymbol::OnlyBefore(offset),
    ));

    items.extend(filter(
        db,
        get_import_members(
            db,
            file,
            &source.imports,
            GetMembers::Const(source.const_table),
        ),
        &mut taken_names,
        FindSymbol::Any,
    ));

    let scope = stack.last().unwrap();
    let execution_range = scope.execution_range;
    let container = scope.container;

    items.extend(filter(
        db,
        source.get_container_members(container),
        &mut taken_names,
        FindSymbol::BeforeIfInExecutionRange(offset, execution_range),
    ));

    items.extend(filter(
        db,
        get_import_members(db, file, &source.imports, GetMembers::Container(container)),
        &mut taken_names,
        FindSymbol::Any,
    ));

    items.extend(filter(
        db,
        source.get_root_members(),
        &mut taken_names,
        FindSymbol::BeforeIfInExecutionRange(offset, execution_range),
    ));

    items.extend(filter(
        db,
        get_import_members(
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

pub fn get_type_members(db: &dyn Db, typ: Type) -> Vec<SymbolId> {
    match typ {
        Type::Table(id) => id.get_data(db).get_members(),
        Type::Class(id) => id.get_data(db).get_members(),
        Type::Enum(id) => id.get_data(db).get_members(),
        Type::Instance(id) => id.get_data(db).get_members(),
        _ => Vec::new(),
    }
}

pub fn members_of_type(db: &dyn Db, typ: Type) -> Vec<Symbol> {
    let members = get_type_members(db, typ);
    members
        .into_iter()
        .map(|id| id.get_data(db).clone())
        .collect()
}

pub fn expr_to_type(db: &dyn Db, expr: NullableExprKind) -> Type {
    match expr {
        Some(ExpressionKind::Literal(kind)) => kind,
        Some(ExpressionKind::Symbol(symbol)) => symbol.get_data(db).typ,
        None => Type::Unknown,
    }
}

pub fn type_at(db: &dyn Db, file: File, text_range: TextRange) -> Type {
    let source = source_symbol(db, file);
    expr_to_type(db, source.expr_kind_at(text_range))
}

impl SourceSymbol {
    fn get_const_members(&self) -> Vec<SymbolId> {
        self.arena[self.const_table].get_members()
    }

    fn get_root_members(&self) -> Vec<SymbolId> {
        self.arena[self.root_table].get_members()
    }

    fn get_container_members(&self, container: Container) -> Vec<SymbolId> {
        match container {
            Container::Table(idx) => self.arena[idx].get_members(),
            Container::Class(idx) => self.arena[idx].get_members(),
            Container::Enum(idx) => self.arena[idx].get_members(),
        }
    }

    pub fn expr_kind_at(&self, text_range: TextRange) -> Option<ExpressionKind> {
        self.expr_kinds.get(&text_range).cloned()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
