use la_arena::{Arena, Idx};

use crate::{
    collector::ExpressionKind,
    symbol::{Symbol, SymbolTable, Type},
};

pub type SymbolId = Idx<Symbol>;
pub type TableId = Idx<TableData>;
pub type ClassId = Idx<ClassData>;
pub type EnumId = Idx<EnumData>;
pub type FunctionId = Idx<FunctionData>;
pub type ArrayId = Idx<ArrayData>;
pub type StringId = Idx<Box<str>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    Table(TableId),
    Class(ClassId),
    Enum(EnumId),
}

impl From<Container> for Type {
    fn from(value: Container) -> Self {
        match value {
            Container::Table(idx) => Type::Table(idx),
            Container::Class(idx) => Type::Class(idx),
            Container::Enum(idx) => Type::Enum(idx),
        }
    }
}

impl TryFrom<Type> for Container {
    type Error = ();
    fn try_from(value: Type) -> Result<Self, Self::Error> {
        Ok(match value {
            Type::Table(idx) => Container::Table(idx),
            Type::Class(idx) => Container::Class(idx),
            Type::Enum(idx) => Container::Enum(idx),
            _ => return Err(()),
        })
    }
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
    pub ret: Type,
    pub params: Vec<SymbolId>,
    pub params_state: ParamsState,
    pub yielding: Option<Type>,
    pub throwing: Option<Type>,
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
    pub kind: Type,
}

pub trait ArenaAlloc<T> {
    fn alloc(&mut self, value: T) -> Idx<T>;
}

macro_rules! impl_arenas {
    ($($field:ident: $data:ty),* $(,)?) => {
        #[derive(Debug, Default, PartialEq, Eq)]
        pub struct Arenas {
            $($field: Arena<$data>,)*
        }

        $(
            impl std::ops::Index<Idx<$data>> for Arenas {
                type Output = $data;
                fn index(&self, id: Idx<$data>) -> &$data {
                    &self.$field[id]
                }
            }

            impl std::ops::IndexMut<Idx<$data>> for Arenas {
                fn index_mut(&mut self, id: Idx<$data>) -> &mut $data {
                    &mut self.$field[id]
                }
            }

            impl ArenaAlloc<$data> for Arenas {
                fn alloc(&mut self, value: $data) -> Idx<$data> {
                    self.$field.alloc(value)
                }
            }
        )*
    };
}

impl_arenas! {
    symbols:   Symbol,
    tables:    TableData,
    classes:   ClassData,
    enums:     EnumData,
    functions: FunctionData,
    arrays:    ArrayData,
    strings:   Box<str>,
}

impl Arenas {
    pub fn add_container_member(&mut self, container: Container, name: String, member: SymbolId) {
        match container {
            Container::Table(id) => self[id].add_member(name, member),
            Container::Class(id) => self[id].add_member(name, member),
            Container::Enum(id) => self[id].add_member(name, member),
        }
    }

    pub fn get_container_members(&self, container: Container) -> Vec<SymbolId> {
        match container {
            Container::Table(id) => self[id].get_members(self),
            Container::Class(id) => self[id].get_members(),
            Container::Enum(id) => self[id].get_members(),
        }
    }

    pub fn get_type_members(&self, typ: Type) -> Vec<SymbolId> {
        match typ {
            Type::Table(id) => self[id].get_members(self),
            Type::Class(id) => self[id].get_members(),
            Type::Instance(id) => self[id].get_members(),
            Type::Enum(id) => self[id].get_members(),
            _ => Vec::new(),
        }
    }

    pub fn clone_type(&mut self, kind: Type) -> Type {
        match kind {
            Type::Table(id) => Type::Table(self.alloc(self[id].clone())),
            Type::Class(id) => Type::Class(self.alloc(self[id].clone())),
            _ => kind,
        }
    }

    pub fn clone_members(&mut self, superclass: ClassId) -> SymbolTable {
        let symbol = &self.classes[superclass];
        let mut members = symbol.members.clone();

        for value in members.values_mut() {
            let symbol = self[*value].clone();
            *value = self.alloc(symbol);
        }

        members
    }

    pub fn expr_to_type(&self, expr: Option<ExpressionKind>) -> Type {
        match expr {
            Some(ExpressionKind::Literal(kind)) => kind,
            Some(ExpressionKind::Symbol(symbol)) => self[symbol].typ,
            None => Type::Unknown,
        }
    }
}
