use rustc_hash::FxHashMap;
use sq_3_parser::TextRange;

use crate::{
    FileState,
    arena::{ArrayId, ClassId, EnumId, FunctionId, StringId, SymbolId, TableId},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Symbol {
    pub name: String,
    pub typ: Type,
    pub kind: SymbolKind,
    pub name_range: TextRange,
    pub range: TextRange,
}

pub type SymbolTable = FxHashMap<String, SymbolId>;

// For option: if not None the value is known at compile time
// otherwise it's not (primarily used for consts features)
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Type {
    #[default]
    Unknown,
    Integer(Option<i32>),
    Float(Option<f32>),
    String(Option<StringId>),
    Boolean(Option<bool>),
    Null,
    Instance(ClassId),
    Array(ArrayId),
    Table(TableId),
    Class(ClassId),
    Enum(EnumId),
    Function(FunctionId),
    Generator(FunctionId),
    Thread(FunctionId),
    Weakref,
}

impl Type {
    pub fn should_substitute(&self) -> bool {
        matches!(self, Type::Unknown | Type::Null)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Local,
    Constant,
    Enum,
    EnumMember,
    Property,
}

impl SymbolKind {
    pub fn is_modifiable(self) -> bool {
        match self {
            SymbolKind::Local | SymbolKind::Property => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unknown => write!(f, "unknown"),
            Type::Integer(_) => write!(f, "integer"),
            Type::Float(_) => write!(f, "float"),
            Type::String(_) => write!(f, "string"),
            Type::Boolean(_) => write!(f, "bool"),
            Type::Null => write!(f, "null"),
            Type::Instance(_) => write!(f, "instance"),
            Type::Array(_) => write!(f, "array"),
            Type::Table(_) => write!(f, "table"),
            Type::Class(_) => write!(f, "class"),
            Type::Enum(_) => write!(f, "enum"),
            Type::Function(_) => write!(f, "function"),
            Type::Generator(_) => write!(f, "generator"),
            Type::Thread(_) => write!(f, "thread"),
            Type::Weakref => write!(f, "weakref"),
        }
    }
}

impl Symbol {
    pub fn display<'a>(&'a self, file_state: &'a FileState<'a>) -> SymbolDisplay<'a> {
        SymbolDisplay {
            symbol: self,
            file_state,
        }
    }
}

pub struct SymbolDisplay<'a> {
    symbol: &'a Symbol,
    file_state: &'a FileState<'a>,
}

impl std::fmt::Display for SymbolDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.symbol;
        match s.kind {
            SymbolKind::Local => write!(f, "local ")?,
            SymbolKind::Property => {}
            SymbolKind::Enum => return write!(f, "enum {}", s.name),
            SymbolKind::Constant | SymbolKind::EnumMember => {
                let type_text = match s.typ {
                    Type::Integer(Some(value)) => value.to_string(),
                    Type::Float(Some(value)) => value.to_string(),
                    Type::Boolean(Some(value)) => value.to_string(),
                    Type::String(Some(id)) => {
                        format!("\"{}\"", self.file_state.get(id))
                    }
                    _ => return write!(f, "const {}", s.name),
                };
                return write!(f, "const {}: {}", s.name, type_text);
            }
        };

        match s.typ {
            Type::Function(id) => {
                let func = self.file_state.get(id);
                write!(f, "function {}(", s.name)?;
                for (i, &param) in func.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    let param = self.file_state.get(param);
                    if param.typ != Type::Unknown {
                        write!(f, "{}: {}", param.name, param.typ)?;
                    } else {
                        write!(f, "{}", param.name)?;
                    }
                }
                if func.ret != Type::Unknown {
                    if func.throwing.is_some() {
                        write!(f, ") -> !{}", func.ret)
                    } else {
                        write!(f, ") -> {}", func.ret)
                    }
                } else if func.throwing.is_some() {
                    write!(f, ") -> !")
                } else {
                    write!(f, ")")
                }
            }
            Type::Class(_) => write!(f, "class {}", s.name),
            _ => write!(f, "{}: {}", s.name, s.typ),
        }
    }
}
