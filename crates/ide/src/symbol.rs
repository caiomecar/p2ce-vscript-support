use rustc_hash::FxHashMap;
use sq_3_parser::TextRange;

use crate::arena::{ArrayId, ClassId, EnumId, FunctionId, StringId, SymbolId, TableId};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Symbol {
    pub name: String,
    pub typ: Type,
    pub kind: SymbolKind,
    pub name_range: TextRange,
    pub range: TextRange,
}

/// To represent multiple symbols with the same name
/// we use a vector instead of 1 to 1 mapping
/// this complicated API quite a bit since we need to
/// pass current execution range and offset whenever
/// we want a specific symbol but properly represents
/// what is actually happening in the source file
pub type SymbolTable = FxHashMap<String, Vec<SymbolId>>;

// This is used in "members_of_type" where it's possible to flatten the table
pub type FlatSymbolTable = FxHashMap<String, SymbolId>;

pub fn insert_symbol(table: &mut SymbolTable, name: String, value: SymbolId) {
    table
        .entry(name)
        .and_modify(|entry| entry.push(value))
        .or_insert_with(|| vec![value]);
}

pub fn to_flat_symbol_table(table: SymbolTable) -> FlatSymbolTable {
    table
        .into_iter()
        .map(|(name, ids)| (name, *ids.last().unwrap()))
        .collect()
}

/// options here basically mean: we know it would be this type
/// but we don't really know what is the exact data behind it
/// for instances tables with different keys, or
/// instances coming from different classes.
/// It's better to use None here rather than to turn everything
/// into Unknown since we at least can get partial completions
/// and operations
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Type {
    #[default]
    Unknown,
    Any,
    Integer(Option<i32>),
    Float(Option<f32>),
    String(Option<StringId>),
    Boolean(Option<bool>),
    Null,
    Instance(Option<ClassId>),
    Array(Option<ArrayId>),
    Table(Option<TableId>),
    Class(Option<ClassId>),
    Enum(EnumId),
    Function(Option<FunctionId>),
    Generator(Option<FunctionId>),
    Thread(Option<FunctionId>),
    Weakref,
}

impl Type {
    pub fn try_substitute_with(&mut self, other: Type) {
        match (&self, other) {
            // We want to replace null with unknown to not error out
            (Type::Null, Type::Unknown) => *self = other,
            (Type::Null | Type::Unknown, _) => *self = other,
            _ => {}
        }
    }

    pub fn merge(&self, other: Type) -> Type {
        match (*self, other) {
            (Type::Any | Type::Unknown | Type::Null, _) => other,
            (_, Type::Any | Type::Unknown | Type::Null) => *self,

            (Type::Integer(_), Type::Integer(_)) => Type::Integer(None),
            (Type::Integer(_) | Type::Float(_), Type::Integer(_) | Type::Float(_)) => {
                Type::Float(None)
            }
            (Type::String(_), Type::String(_)) => Type::String(None),
            (Type::Boolean(_), Type::Boolean(_)) => Type::Boolean(None),
            (Type::Instance(Some(self_id)), Type::Instance(Some(other_id)))
                if self_id == other_id =>
            {
                other
            }
            (Type::Instance(_), Type::Instance(_)) => Type::Instance(None),
            (Type::Table(_), Type::Table(_)) => Type::Table(None),
            (Type::Class(_), Type::Class(_)) => Type::Class(None),
            (Type::Array(_), Type::Array(_)) => Type::Array(None),
            (Type::Function(_), Type::Function(_)) => Type::Function(None),
            (Type::Generator(_), Type::Generator(_)) => Type::Generator(None),
            (Type::Thread(_), Type::Thread(_)) => Type::Thread(None),
            (Type::Weakref, Type::Weakref) => Type::Weakref,
            (_, _) => Type::Any,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Local(LocalKind),
    Constant,
    Enum,
    EnumMember,
    Property(PropertyKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalKind {
    Variable,
    Function,
    Parameter,
    Exception,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum PropertyKind {
    #[default]
    NoSupport,
    NewSlot,
    No,
    Yes,
}

impl Default for SymbolKind {
    fn default() -> Self {
        Self::Property(PropertyKind::default())
    }
}

impl SymbolKind {
    pub fn is_modifiable(self) -> bool {
        match self {
            SymbolKind::Local(_) | SymbolKind::Property(_) => true,
            _ => false,
        }
    }
}
