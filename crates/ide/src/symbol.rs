use rustc_hash::FxHashMap;
use sq_3_parser::TextRange;

use crate::arena::{ArrayId, ClassId, EnumId, FunctionId, StringId, SymbolId, TableId, UnionId};

/// The type itself and whether it's annotated explicitly
/// or not. (Need to use doc comment to annotate)
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct AnnotatedType(pub Type, pub bool);
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Symbol {
    pub name: String,
    pub typ: AnnotatedType,
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
    Union(UnionId),
}

impl Into<AnnotatedType> for Type {
    fn into(self) -> AnnotatedType {
        AnnotatedType(self, false)
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
    VariedArgs,
    Exception,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum PropertyKind {
    #[default]
    NoSupport,
    NewSlot,
    No,
    Yes,
    Embedded,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TypeKind {
    Unknown = 1 << 0,
    Any = 1 << 1,
    Integer = 1 << 2,
    Float = 1 << 3,
    String = 1 << 4,
    Boolean = 1 << 5,
    Null = 1 << 6,
    Instance = 1 << 7,
    Array = 1 << 8,
    Table = 1 << 9,
    Class = 1 << 10,
    Enum = 1 << 11,
    Function = 1 << 12,
    Generator = 1 << 13,
    Thread = 1 << 14,
    Weakref = 1 << 15,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TypeSet(u32);

impl TypeSet {
    pub const fn new(kinds: &[TypeKind]) -> Self {
        let mut bitset = 0u32;
        let mut i = 0;
        while i < kinds.len() {
            bitset |= kinds[i] as u32;
            i += 1;
        }
        Self(bitset)
    }

    pub const fn from_kind(kind: TypeKind) -> Self {
        Self(kind as u32)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn contains(self, other: TypeSet) -> bool {
        (self.0 & other.0) != 0
    }

    pub const fn are_both_numbers(first: TypeSet, second: TypeSet) -> bool {
        TypeSet::NUMBER.contains(first) && TypeSet::NUMBER.contains(second)
    }

    pub const EMPTY: TypeSet = TypeSet::new(&[]);
    pub const ANY: TypeSet = TypeSet::new(&[TypeKind::Unknown, TypeKind::Any]);
    pub const INTEGER: TypeSet = TypeSet::from_kind(TypeKind::Integer);
    pub const NUMBER: TypeSet = TypeSet::new(&[TypeKind::Float, TypeKind::Integer]);
    pub const NUMBER_OR_ANY: TypeSet = TypeSet::NUMBER.union(TypeSet::ANY);
    pub const STRING: TypeSet = TypeSet::from_kind(TypeKind::String);
    pub const NULL: TypeSet = TypeSet::from_kind(TypeKind::Null);
    pub const TABLE: TypeSet = TypeSet::from_kind(TypeKind::Table);
    pub const INSTANCE: TypeSet = TypeSet::from_kind(TypeKind::Instance);

    pub const TABLE_OR_INSTANCE: TypeSet = TypeSet::new(&[TypeKind::Table, TypeKind::Instance]);

    pub const VALID_IN_LHS: TypeSet =
        TypeSet::new(&[TypeKind::Array, TypeKind::Table, TypeKind::Class]).union(TypeSet::ANY);
    pub const VALID_INSTANCE_OF_LHS: TypeSet =
        TypeSet::new(&[TypeKind::Instance]).union(TypeSet::ANY);
    pub const VALID_INSTANCE_OF_RHS: TypeSet = TypeSet::new(&[TypeKind::Class]).union(TypeSet::ANY);
    pub const VALID_SWITCH_DISCRIMINANT: TypeSet = TypeSet::new(&[
        TypeKind::Null,
        TypeKind::Float,
        TypeKind::Integer,
        TypeKind::Boolean,
        TypeKind::String,
    ])
    .union(TypeSet::ANY);
    pub const CAN_COMPARE: TypeSet = TypeSet::new(&[
        TypeKind::Null,
        TypeKind::Float,
        TypeKind::Integer,
        TypeKind::Boolean,
        TypeKind::String,
        TypeKind::Table,
        TypeKind::Instance,
    ])
    .union(TypeSet::ANY);
    pub const CAN_HAVE_UNKNOWN_MEMBERS: TypeSet =
        TypeSet::new(&[TypeKind::Table, TypeKind::Class, TypeKind::Instance]).union(TypeSet::ANY);
}

impl Into<TypeKind> for Type {
    fn into(self) -> TypeKind {
        match self {
            Type::Unknown => TypeKind::Unknown,
            Type::Any => TypeKind::Any,
            Type::Integer(_) => TypeKind::Integer,
            Type::Float(_) => TypeKind::Float,
            Type::String(_) => TypeKind::String,
            Type::Boolean(_) => TypeKind::Boolean,
            Type::Null => TypeKind::Null,
            Type::Instance(_) => TypeKind::Instance,
            Type::Array(_) => TypeKind::Array,
            Type::Table(_) => TypeKind::Table,
            Type::Class(_) => TypeKind::Class,
            Type::Enum(_) => TypeKind::Enum,
            Type::Function(_) => TypeKind::Function,
            Type::Generator(_) => TypeKind::Generator,
            Type::Thread(_) => TypeKind::Thread,
            Type::Weakref => TypeKind::Weakref,
            Type::Union(_) => unreachable!(), // handled separately
        }
    }
}
