use la_arena::{Arena, Idx};
use rustc_hash::FxHashMap;

use crate::{ExpressionKind, Symbol, SymbolKind};

pub type SymbolTable = FxHashMap<String, SymbolId>;
pub type SymbolId = Idx<Symbol>;
pub type TableId = Idx<TableData>;
pub type ClassId = Idx<ClassData>;
pub type EnumId = Idx<EnumData>;
pub type FunctionId = Idx<FunctionData>;
pub type ArrayId = Idx<ArrayData>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    Table(TableId),
    Class(ClassId),
    Enum(EnumId),
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TableData {
    pub delegate: Option<TableId>,
    pub members: SymbolTable,
}

impl TableData {
    pub fn get_members(&self, arenas: &Arenas) -> Vec<SymbolId> {
        let mut result: Vec<SymbolId> = self.members.values().copied().collect();
        if let Some(delegate) = self.delegate {
            result.extend(arenas.tables[delegate].get_members(arenas));
        }

        result
    }

    pub fn add_member(&mut self, name: String, symbol: SymbolId) {
        self.members.insert(name, symbol);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ClassData {
    pub inherits: Option<ClassId>,
    pub members: SymbolTable,
}

impl ClassData {
    pub fn get_members(&self) -> Vec<SymbolId> {
        self.members.values().copied().collect()
    }

    pub fn add_member(&mut self, name: String, symbol: SymbolId) {
        self.members.insert(name, symbol);
    }

    pub fn get_member(&self, name: &str) -> Option<SymbolId> {
        Some(*self.members.get(name)?)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EnumData {
    pub members: SymbolTable,
}

impl EnumData {
    pub fn get_members(&self) -> Vec<SymbolId> {
        self.members.values().copied().collect()
    }

    pub fn add_member(&mut self, name: String, symbol: SymbolId) {
        self.members.insert(name, symbol);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FunctionData {
    pub ret: SymbolKind,
    pub params: Vec<SymbolId>,
    pub params_state: ParamsState,
    pub yielding: Option<SymbolKind>,
    pub throwing: Option<SymbolKind>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ParamsState {
    #[default]
    NoDefault,
    Default(usize),
    VarArgs(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArrayData {
    pub kind: SymbolKind,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Arenas {
    pub symbols: Arena<Symbol>,
    pub tables: Arena<TableData>,
    pub classes: Arena<ClassData>,
    pub enums: Arena<EnumData>,
    pub functions: Arena<FunctionData>,
    pub arrays: Arena<ArrayData>,
}

impl Arenas {
    pub fn add_container_member(&mut self, container: Container, name: String, member: SymbolId) {
        match container {
            Container::Table(id) => self.tables[id].add_member(name, member),
            Container::Class(id) => self.classes[id].add_member(name, member),
            Container::Enum(id) => self.enums[id].add_member(name, member),
        }
    }

    pub fn get_container_members(&self, container: Container) -> Vec<SymbolId> {
        match container {
            Container::Table(id) => self.tables[id].get_members(self),
            Container::Class(id) => self.classes[id].get_members(),
            Container::Enum(id) => self.enums[id].get_members(),
        }
    }

    pub fn get_kind_members(&self, kind: SymbolKind) -> Option<Vec<SymbolId>> {
        Some(match kind {
            SymbolKind::Table(id) => self.tables[id].get_members(self),
            SymbolKind::Class(id) => self.classes[id].get_members(),
            SymbolKind::Instance(id) => self.classes[id].get_members(),
            SymbolKind::Enum(id) => self.enums[id].get_members(),
            _ => return None,
        })
    }

    pub fn clone_kind(&mut self, kind: SymbolKind) -> SymbolKind {
        match kind {
            SymbolKind::Table(id) => SymbolKind::Table(self.tables.alloc(self.tables[id].clone())),
            SymbolKind::Class(id) => {
                SymbolKind::Class(self.classes.alloc(self.classes[id].clone()))
            }
            _ => kind,
        }
    }

    pub fn clone_members(&mut self, superclass: ClassId) -> SymbolTable {
        let symbol = &self.classes[superclass];
        let mut members = symbol.members.clone();

        for value in members.values_mut() {
            let symbol = self.symbols[*value].clone();
            *value = self.symbols.alloc(symbol);
        }

        members
    }

    pub fn expr_to_symbol_kind(&self, expr: &ExpressionKind) -> SymbolKind {
        match *expr {
            ExpressionKind::Literal(kind) => kind,
            ExpressionKind::Symbol(symbol) => self.symbols[symbol].kind,
            ExpressionKind::Parent(_, _) | ExpressionKind::Unknown => SymbolKind::Unknown,
        }
    }
}
