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
    pub description: Option<String>,
    pub flags: SymbolFlags,
}

bitflags::bitflags! {
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct SymbolFlags: u8 {
        const CONST = 1 << 0;
        const HIDE = 1 << 1;
        const DEPRECATED = 1 << 2;
        const PRIVATE = 1 << 3;
    }
}

impl Symbol {
    #[must_use]
    pub const fn is_modifiable(&self) -> bool {
        match self.kind {
            SymbolKind::Local(_) | SymbolKind::Property(_) => {
                !self.flags.contains(SymbolFlags::CONST)
            }
            _ => false,
        }
    }
}

/// Maps name to multiple symbols
///
/// To represent multiple symbols with the same name
/// we use a vector instead of 1 to 1 mapping
/// this complicated API quite a bit since we need to
/// pass current execution range and offset whenever
/// we want a specific symbol but properly represents
/// what is actually happening in the source file
pub type SymbolTable = FxHashMap<String, Vec<SymbolId>>;

/// Maps name to a single symbol
///
/// This is used in `members_of_type` where it's possible to flatten the table
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
        .map(|(name, ids)| {
            (
                name,
                *ids.last().expect(
                    "Symbol table vector is guaranteed to contain at least a single symbol",
                ),
            )
        })
        .collect()
}

/// Symbol's type
///
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

impl From<Type> for AnnotatedType {
    fn from(val: Type) -> Self {
        Self(val, false)
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

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub const fn are_both_numbers(first: Self, second: Self) -> bool {
        Self::NUMBER.contains(first) && Self::NUMBER.contains(second)
    }

    pub const EMPTY: Self = Self::new(&[]);
    pub const ANY: Self = Self::new(&[TypeKind::Unknown, TypeKind::Any]);
    pub const INTEGER: Self = Self::from_kind(TypeKind::Integer);
    pub const NUMBER: Self = Self::new(&[TypeKind::Float, TypeKind::Integer]);
    pub const NUMBER_OR_ANY: Self = Self::NUMBER.union(Self::ANY);
    pub const STRING: Self = Self::from_kind(TypeKind::String);
    pub const NULL: Self = Self::from_kind(TypeKind::Null);
    pub const TABLE: Self = Self::from_kind(TypeKind::Table);
    pub const INSTANCE: Self = Self::from_kind(TypeKind::Instance);

    pub const TABLE_OR_INSTANCE: Self = Self::new(&[TypeKind::Table, TypeKind::Instance]);

    pub const VALID_IN_LHS: Self =
        Self::new(&[TypeKind::Array, TypeKind::Table, TypeKind::Class]).union(Self::ANY);
    pub const VALID_INSTANCE_OF_LHS: Self = Self::new(&[TypeKind::Instance]).union(Self::ANY);
    pub const VALID_INSTANCE_OF_RHS: Self = Self::new(&[TypeKind::Class]).union(Self::ANY);
    pub const VALID_SWITCH_DISCRIMINANT: Self = Self::new(&[
        TypeKind::Null,
        TypeKind::Float,
        TypeKind::Integer,
        TypeKind::Boolean,
        TypeKind::String,
    ])
    .union(Self::ANY);
    pub const CAN_COMPARE: Self = Self::new(&[
        TypeKind::Null,
        TypeKind::Float,
        TypeKind::Integer,
        TypeKind::Boolean,
        TypeKind::String,
        TypeKind::Table,
        TypeKind::Instance,
    ])
    .union(Self::ANY);
    pub const CAN_HAVE_UNKNOWN_MEMBERS: Self =
        Self::new(&[TypeKind::Table, TypeKind::Class, TypeKind::Instance]).union(Self::ANY);
}

impl From<Type> for TypeKind {
    fn from(val: Type) -> Self {
        match val {
            Type::Unknown => Self::Unknown,
            Type::Any => Self::Any,
            Type::Integer(_) => Self::Integer,
            Type::Float(_) => Self::Float,
            Type::String(_) => Self::String,
            Type::Boolean(_) => Self::Boolean,
            Type::Null => Self::Null,
            Type::Instance(_) => Self::Instance,
            Type::Array(_) => Self::Array,
            Type::Table(_) => Self::Table,
            Type::Class(_) => Self::Class,
            Type::Enum(_) => Self::Enum,
            Type::Function(_) => Self::Function,
            Type::Generator(_) => Self::Generator,
            Type::Thread(_) => Self::Thread,
            Type::Weakref => Self::Weakref,
            Type::Union(_) => unreachable!(), // handled separately
        }
    }
}
