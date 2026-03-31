use la_arena::{Arena, Idx};

use crate::{
    File,
    db::{Db, source_symbol},
    symbol::{Symbol, SymbolTable, Type},
};

pub trait ArenaId {
    type Data;
    fn file(&self) -> File;
    fn idx(&self) -> Idx<Self::Data>;
    fn get_data<'a>(&self, db: &'a dyn Db) -> &'a Self::Data;
}

macro_rules! arena_id {
    ($name:ident => $data:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name {
            file: File,
            idx: Idx<$data>,
        }

        impl $name {
            pub fn new(file: File, idx: Idx<$data>) -> $name {
                Self { file, idx }
            }
        }

        impl ArenaId for $name {
            type Data = $data;

            fn file(&self) -> File {
                self.file
            }

            fn idx(&self) -> Idx<$data> {
                self.idx
            }

            fn get_data<'a>(&self, db: &'a dyn Db) -> &'a Self::Data {
                let source = source_symbol(db, self.file);
                &source.arena[self.idx]
            }
        }
    };
}

arena_id!(SymbolId => Symbol);
arena_id!(TableId => TableData);
arena_id!(ClassId => ClassData);
arena_id!(EnumId => EnumData);
arena_id!(FunctionId => FunctionData);
arena_id!(ArrayId => ArrayData);
arena_id!(StringId => Box<str>);

// Containers are used for and stored on the scope stack
// They cannot be shared between files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Container {
    Table(Idx<TableData>),
    Class(Idx<ClassData>),
    Enum(Idx<EnumData>),
}

impl Container {
    pub fn to_type(self, file: File) -> Type {
        match self {
            Container::Table(idx) => Type::Table(TableId::new(file, idx)),
            Container::Class(idx) => Type::Class(ClassId::new(file, idx)),
            Container::Enum(idx) => Type::Enum(EnumId::new(file, idx)),
        }
    }
}

impl TryFrom<Type> for Container {
    type Error = ();
    fn try_from(value: Type) -> Result<Self, Self::Error> {
        Ok(match value {
            Type::Table(id) => Container::Table(id.idx()),
            Type::Class(id) => Container::Class(id.idx()),
            Type::Instance(id) => Container::Class(id.idx()),
            Type::Enum(id) => Container::Enum(id.idx()),
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
    pub fn get_members(&self) -> Vec<SymbolId> {
        let result: Vec<SymbolId> = self.members.values().copied().collect();
        // if let Some(delegate) = self.delegate {
        //     let taken_names: FxHashSet<&str> = self.members.keys().map(|s| s.as_str()).collect();
        //     result.extend(
        //         arenas[delegate.0]
        //             .get_members(arenas)
        //             .into_iter()
        //             .filter(|id| !taken_names.contains(arenas[*id].name.as_str())),
        //     );
        // }

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
    pub typ: Type,
}

pub trait ArenaAlloc<T> {
    fn alloc(&mut self, value: T) -> Idx<T>;
}

macro_rules! impl_source_arena {
    ($($field:ident: $data:ty),* $(,)?) => {
        #[derive(Debug, Default, PartialEq, Eq)]
        pub struct SourceArena {
            $($field: Arena<$data>,)*
        }

        $(
            impl std::ops::Index<Idx<$data>> for SourceArena {
                type Output = $data;
                fn index(&self, id: Idx<$data>) -> &$data {
                    &self.$field[id]
                }
            }

            impl std::ops::IndexMut<Idx<$data>> for SourceArena {
                fn index_mut(&mut self, id: Idx<$data>) -> &mut $data {
                    &mut self.$field[id]
                }
            }

            impl ArenaAlloc<$data> for SourceArena {
                fn alloc(&mut self, value: $data) -> Idx<$data> {
                    self.$field.alloc(value)
                }
            }
        )*
    };
}

impl_source_arena! {
    symbols:   Symbol,
    tables:    TableData,
    classes:   ClassData,
    enums:     EnumData,
    functions: FunctionData,
    arrays:    ArrayData,
    strings:   Box<str>,
}

// impl SourceArena {
//     pub fn add_container_member(&mut self, container: Container, name: String, member: SymbolId) {
//         match container {
//             Container::Table(id) => self[id].add_member(name, member),
//             Container::Class(id) => self[id].add_member(name, member),
//             Container::Enum(id) => self[id].add_member(name, member),
//         }
//     }

//     pub fn get_container_members(&self, container: Container) -> Vec<SymbolId> {
//         match container {
//             Container::Table(id) => self[id].get_members(self),
//             Container::Class(id) => self[id].get_members(),
//             Container::Enum(id) => self[id].get_members(),
//         }
//     }

//     pub fn get_type_members(&self, typ: Type) -> Vec<SymbolId> {
//         match typ {
//             Type::Table(id) => self[id].get_members(self),
//             Type::Class(id) => self[id].get_members(),
//             Type::Instance(id) => self[id].get_members(),
//             Type::Enum(id) => self[id].get_members(),
//             _ => Vec::new(),
//         }
//     }

//     pub fn clone_type(&mut self, kind: Type) -> Type {
//         match kind {
//             Type::Table(id) => Type::Table(self.alloc(self[id].clone())),
//             Type::Class(id) => Type::Class(self.alloc(self[id].clone())),
//             _ => kind,
//         }
//     }

//     pub fn clone_members(&mut self, superclass: ClassId) -> SymbolTable {
//         let symbol = &self.classes[superclass];
//         let mut members = symbol.members.clone();

//         for value in members.values_mut() {
//             let symbol = self[*value].clone();
//             *value = self.alloc(symbol);
//         }

//         members
//     }
// }
