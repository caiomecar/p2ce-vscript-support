use std::cmp::Reverse;

use la_arena::{Arena, Idx};
use line_index::{TextRange, TextSize};

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
arena_id!(StringId => StringData);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Container {
    Table(TableId),
    Enum(EnumId),
    // Both classes and instances take their members
    // from the same spot (outside of builtins). And
    // squirrel is bad enough to allow accessing non-static
    // members as a class and static members as an instance.
    // Solution: we normally resolve ignoring static and
    // non-static properties, however for completions we don't
    // show non-static members when accessing class
    // and vise versa (the constructor is always avoided)
    Class(ClassId),
    Instance(ClassId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportTarget {
    Table(TableId),
    Class(ClassId),
}

#[derive(Debug)]
pub enum TypeConversionError {
    WrongType,
    NotSpecific,
}

impl From<Container> for Type {
    fn from(value: Container) -> Self {
        match value {
            Container::Table(idx) => Type::Table(Some(idx)),
            Container::Class(idx) => Type::Class(Some(idx)),
            Container::Instance(idx) => Type::Instance(Some(idx)),
            Container::Enum(idx) => Type::Enum(idx),
        }
    }
}

impl TryFrom<Type> for Container {
    type Error = TypeConversionError;
    fn try_from(value: Type) -> Result<Self, Self::Error> {
        Ok(match value {
            Type::Table(id) => {
                let Some(id) = id else {
                    return Err(TypeConversionError::NotSpecific);
                };
                Container::Table(id)
            }
            Type::Class(id) => {
                let Some(id) = id else {
                    return Err(TypeConversionError::NotSpecific);
                };
                Container::Class(id)
            }
            Type::Instance(id) => {
                let Some(id) = id else {
                    return Err(TypeConversionError::NotSpecific);
                };
                Container::Class(id)
            }
            Type::Enum(id) => Container::Enum(id),
            _ => return Err(TypeConversionError::WrongType),
        })
    }
}

impl TryFrom<Container> for ImportTarget {
    type Error = ();
    fn try_from(value: Container) -> Result<Self, Self::Error> {
        Ok(match value {
            Container::Table(id) => ImportTarget::Table(id),
            Container::Class(id) => ImportTarget::Class(id),
            Container::Instance(id) => ImportTarget::Class(id),
            Container::Enum(_) => return Err(()),
        })
    }
}

impl TryFrom<Type> for ImportTarget {
    type Error = TypeConversionError;
    fn try_from(value: Type) -> Result<Self, Self::Error> {
        Ok(match value {
            Type::Table(id) => {
                let Some(id) = id else {
                    return Err(TypeConversionError::NotSpecific);
                };
                ImportTarget::Table(id)
            }
            Type::Instance(id) | Type::Class(id) => {
                let Some(id) = id else {
                    return Err(TypeConversionError::NotSpecific);
                };
                ImportTarget::Class(id)
            }
            _ => return Err(TypeConversionError::WrongType),
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TableData {
    pub delegate: Option<TableId>,
    pub members: SymbolTable,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ClassData {
    pub symbol: Option<SymbolId>,
    pub inherits: Option<ClassId>,
    pub members: SymbolTable,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EnumData {
    pub members: SymbolTable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionData {
    pub range: TextRange,
    pub ret: Type,
    pub container: Container,
    pub bindenv: Option<Container>,
    pub params: Vec<SymbolId>,
    pub params_state: ParamsState,
    pub yields: Option<Type>,
    pub throws: Option<Type>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum ParamsState {
    #[default]
    NoDefault,
    Default(u32),
    VarArgs(u32, SymbolId),
}

#[derive(Debug, PartialEq)]
pub struct ArrayData {
    pub typ: Type,
}

#[derive(Debug, PartialEq)]
pub struct StringData {
    pub text: Box<str>,
    pub unquoted_range: TextRange,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Scope {
    pub range: TextRange,
    pub locals: SymbolTable,
    pub parent: Option<Idx<Scope>>,
    pub function: Option<Idx<FunctionData>>,
}

pub type ScopeId = Idx<Scope>;

pub trait ArenaAlloc<T> {
    fn alloc(&mut self, value: T) -> Idx<T>;
}

macro_rules! impl_source_arena {
    ($($field:ident: $data:ty),* $(,)?) => {
        #[derive(Debug, Default, PartialEq)]
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
    scopes:    Scope,
    tables:    TableData,
    classes:   ClassData,
    enums:     EnumData,
    functions: FunctionData,
    arrays:    ArrayData,
    strings:   StringData,
}

impl SourceArena {
    pub fn scope_at(&self, offset: TextSize) -> Idx<Scope> {
        self.scopes
            .iter()
            .filter(|(_, s)| s.range.contains_inclusive(offset))
            // Since the range can be equal (e.g. when we have a function that creates a scope with the size of its body)
            // the higher value of the index would serve as the tiebreaker since if range is equal then scope created later
            // is guaranteed to be deeper
            .min_by_key(|(i, s)| (s.range.len(), Reverse(*i)))
            .map(|(i, _)| i)
            .unwrap()
        //.unwrap_or_default(Idx::from_raw(RawIdx::from(0 as u32)))
    }

    pub fn all_symbols(&self) -> impl Iterator<Item = (Idx<Symbol>, &Symbol)> {
        self.symbols.iter().map(|entry| entry)
    }
}
