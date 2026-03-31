mod arena;
mod collector;
mod db;
mod symbol;

use crate::{
    arena::{ArenaId, Container, SourceArena, SymbolId, TableData, TableId},
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

enum FilterSymbols {
    OnlyBefore(TextSize),
    BeforeIfInExecutionRange(TextSize, TextRange),
}

#[derive(Debug, Clone, Copy)]
enum GetMembers {
    Container(Container),
    Source,
    Const,
    Root,
}

impl SourceSymbol {
    fn get_container_members(&self, container: Container) -> Vec<SymbolId> {
        match container {
            Container::Table(idx) => self.arena[idx].get_members(),
            Container::Class(idx) => self.arena[idx].get_members(),
            Container::Enum(idx) => self.arena[idx].get_members(),
        }
    }

    pub(crate) fn get_members_inner(
        &self,
        db: &dyn Db,
        request_for: File,
        settings: GetMembers,
    ) -> Vec<SymbolId> {
        let mut already_included = FxHashSet::default();
        already_included.insert(request_for);
        self.get_members(db, request_for, &mut already_included, settings)
    }

    pub(crate) fn get_members(
        &self,
        db: &dyn Db,
        for_file: File,
        already_included: &mut FxHashSet<File>,
        settings: GetMembers,
    ) -> Vec<SymbolId> {
        let mut result = match settings {
            GetMembers::Container(container) => self.get_container_members(container),
            GetMembers::Source => self.arena[self.source_table].get_members(),
            GetMembers::Const => self.arena[self.const_table].get_members(),
            GetMembers::Root => self.arena[self.root_table].get_members(),
        };

        let container = match settings {
            GetMembers::Container(container) => container,
            GetMembers::Source => Container::Table(self.source_table),
            GetMembers::Const | GetMembers::Root => {
                for imports in self.imports.values() {
                    for import in imports {
                        if !already_included.insert(*import) {
                            continue;
                        }

                        let source_symbol = source_symbol(db, *import);
                        result.extend(source_symbol.get_members(
                            db,
                            for_file,
                            already_included,
                            settings,
                        ));
                    }
                }
                return result;
            }
        };

        let Some(imports) = self.imports.get(&container) else {
            return result;
        };

        for import in imports {
            if already_included.contains(import) {
                continue;
            }

            already_included.insert(*import);
            let source_symbol = source_symbol(db, *import);
            result.extend(source_symbol.get_members(db, for_file, already_included, settings));
        }

        result
    }

    pub fn symbols_at(&self, db: &dyn Db, offset: TextSize) -> Vec<Symbol> {
        let stack = self.source_scope.stack_at(offset);
        let mut taken_names = FxHashSet::default();
        let mut items = Vec::new();

        {
            let mut filter = |id: SymbolId| {
                let symbol = id.get_data(db);

                if id.file() == self.file
                    && (symbol.range.end() >= offset || !taken_names.insert(symbol.name.clone()))
                {
                    return None;
                }

                Some(symbol.clone())
            };

            // Locals first
            items.extend(
                stack
                    .iter()
                    .rev()
                    .flat_map(|scope| scope.locals.values().copied())
                    .filter_map(&mut filter),
            );

            // Consts second
            items.extend(
                self.get_members_inner(db, self.file, GetMembers::Const)
                    .into_iter()
                    .filter_map(&mut filter),
            );
        }

        {
            let last_scope = stack.last().unwrap();
            // We ignore member positions if they're declared outside of the current function
            let enclosing_function = last_scope.execution_range;

            let mut filter_function = |id: SymbolId| {
                let symbol = id.get_data(db);

                if id.file() == self.file
                    && (enclosing_function.contains_range(symbol.range)
                        && symbol.range.end() >= offset
                        || !taken_names.insert(symbol.name.clone()))
                {
                    return None;
                }

                Some(symbol.clone())
            };

            // Members third
            items.extend(
                self.get_members_inner(db, self.file, GetMembers::Container(last_scope.container))
                    .into_iter()
                    .filter_map(&mut filter_function),
            );

            // Root fourth
            items.extend(
                self.get_members_inner(db, self.file, GetMembers::Root)
                    .into_iter()
                    .filter_map(&mut filter_function),
            );
        }

        items
    }

    fn expr_to_type(&self, db: &dyn Db, expr: NullableExprKind) -> Type {
        match expr {
            Some(ExpressionKind::Literal(kind)) => kind,
            Some(ExpressionKind::Symbol(symbol)) => symbol.get_data(db).typ,
            None => Type::Unknown,
        }
    }

    pub fn expr_kind_at(&self, text_range: TextRange) -> Option<ExpressionKind> {
        self.expr_kinds.get(&text_range).cloned()
    }

    pub fn type_at(&self, db: &dyn Db, text_range: TextRange) -> Type {
        self.expr_to_type(db, self.expr_kind_at(text_range))
    }

    fn get_type_members(&self, db: &dyn Db, typ: Type) -> Vec<SymbolId> {
        match typ {
            Type::Table(id) => id.get_data(db).get_members(),
            Type::Class(id) => id.get_data(db).get_members(),
            Type::Enum(id) => id.get_data(db).get_members(),
            Type::Instance(id) => id.get_data(db).get_members(),
            _ => Vec::new(),
        }
    }

    pub fn members_of_type(&self, db: &dyn Db, typ: Type) -> Vec<Symbol> {
        let members = self.get_type_members(db, typ);
        members
            .into_iter()
            .map(|id| id.get_data(db).clone())
            .collect()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
