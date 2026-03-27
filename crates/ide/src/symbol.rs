use rustc_hash::FxHashMap;
use sq_3_parser::TextRange;

use crate::arena::{ArrayId, ClassId, EnumId, FunctionId, StringId, SymbolId, TableId};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Symbol {
    pub kind: Type,
    pub name: String,
    pub range: TextRange,
}

pub type SymbolTable = FxHashMap<String, SymbolId>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[default]
    Unknown,
    Integer,
    Float,
    String(Option<StringId>),
    Boolean,
    Null,
    Instance(ClassId),
    Array(ArrayId),
    Table(TableId),
    Class(ClassId),
    Enum(EnumId),
    Function(FunctionId),
    Generator(FunctionId),
    Thread(FunctionId),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unknown => write!(f, "unknown"),
            Type::Integer => write!(f, "integer"),
            Type::Float => write!(f, "float"),
            Type::String(_) => write!(f, "string"),
            Type::Boolean => write!(f, "bool"),
            Type::Null => write!(f, "null"),
            Type::Instance(_) => write!(f, "instance"),
            Type::Array(_) => write!(f, "array"),
            Type::Table(_) => write!(f, "table"),
            Type::Class(_) => write!(f, "class"),
            Type::Enum(_) => write!(f, "enum"),
            Type::Function(_) => write!(f, "function"),
            Type::Generator(_) => write!(f, "generator"),
            Type::Thread(_) => write!(f, "thread"),
        }
    }
}
