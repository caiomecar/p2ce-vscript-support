mod arena;
mod collector;
mod db;
mod symbol;

use crate::{
    arena::{Arenas, SymbolId, TableId},
    collector::{ExpressionKind, RangeKindMap, Scope},
};
pub use db::{Database, File, line_index, parse, source_symbol};
use rustc_hash::FxHashSet;
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
    diagnostics: Vec<Diagnostic>,
    scope: Scope,
    arenas: Arenas,
    const_table: TableId,
    root_table: TableId,
    source_table: TableId,
    range_kind: RangeKindMap,
}

impl SourceSymbol {
    pub(crate) fn new(
        diagnostics: Vec<Diagnostic>,
        scope: Scope,
        arenas: Arenas,
        const_table: TableId,
        root_table: TableId,
        source_table: TableId,
        range_kind: RangeKindMap,
    ) -> SourceSymbol {
        dbg!(&range_kind);
        Self {
            diagnostics,
            scope,
            arenas,
            const_table,
            root_table,
            source_table,
            range_kind,
        }
    }

    pub fn symbols_at(&self, offset: TextSize) -> Vec<&Symbol> {
        let stack = self.scope.stack_at(offset);
        let mut taken_names = FxHashSet::default();
        let mut items = Vec::new();

        {
            let mut filter = |idx: SymbolId| {
                let symbol = &self.arenas[idx];

                if symbol.range.end() >= offset || !taken_names.insert(symbol.name.clone()) {
                    return None;
                }

                Some(symbol)
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
                self.arenas[self.const_table]
                    .get_members(&self.arenas)
                    .into_iter()
                    .filter_map(&mut filter),
            );
        }

        {
            let last_scope = stack.last().unwrap();
            // We ignore member positions if they're declared outside of the current function
            let enclosing_function = last_scope.execution_range;

            let mut filter_function = |idx: SymbolId| {
                let symbol = &self.arenas[idx];

                if enclosing_function.contains_range(symbol.range) && symbol.range.end() >= offset
                    || !taken_names.insert(symbol.name.clone())
                {
                    return None;
                }

                Some(symbol)
            };

            // Members third
            items.extend(
                self.arenas
                    .get_container_members(last_scope.container)
                    .into_iter()
                    .filter_map(&mut filter_function),
            );

            // Root fourth
            items.extend(
                self.arenas[self.root_table]
                    .get_members(&self.arenas)
                    .into_iter()
                    .filter_map(&mut filter_function),
            );
        }

        items
    }

    pub fn expr_kind_at(&self, text_range: TextRange) -> Option<ExpressionKind> {
        self.range_kind.get(&text_range).cloned()
    }

    pub fn type_at(&self, text_range: TextRange) -> Type {
        self.arenas
            .expr_to_type(self.range_kind.get(&text_range).cloned())
    }

    pub fn members_of_type(&self, kind: Type) -> Vec<&Symbol> {
        let members = self.arenas.get_type_members(kind);
        members.into_iter().map(|idx| &self.arenas[idx]).collect()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
