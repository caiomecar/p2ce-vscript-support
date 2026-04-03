use std::time::Instant;

use ::line_index::LineIndex;
use salsa::Setter;
use sq_3_parser::Parse;

use crate::{SourceSymbol, Type, arena::ArenaId, collector::Collector, symbol::SymbolTable};

#[salsa::input]
#[derive(Debug)]
pub struct File {
    #[returns(ref)]
    pub text: String,
}

pub struct Builtins {
    pub integer: SymbolTable,
    pub float: SymbolTable,
    pub boolean: SymbolTable,
    pub string: SymbolTable,
    pub array: SymbolTable,
    pub table: SymbolTable,
    pub function: SymbolTable,
    pub class: SymbolTable,
    pub instance: SymbolTable,
    pub generator: SymbolTable,
    pub thread: SymbolTable,
    pub weakref: SymbolTable,
}

#[salsa::db]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
    builtins: Option<Builtins>,
    squirrel_lib: Option<File>,
    vscript_lib: Option<File>,
}

#[salsa::db]
pub trait Db: salsa::Database {
    fn builtins(&self) -> Option<&Builtins>;
    fn squirrel_lib(&self) -> Option<File>;
    fn vscript_lib(&self) -> Option<File>;
}

impl salsa::Database for Database {}
#[salsa::db]
impl Db for Database {
    fn builtins(&self) -> Option<&Builtins> {
        self.builtins.as_ref()
    }

    fn squirrel_lib(&self) -> Option<File> {
        self.squirrel_lib
    }

    fn vscript_lib(&self) -> Option<File> {
        self.vscript_lib
    }
}

impl Database {
    pub fn init_builtins(&mut self, text: String) -> File {
        let builtins = File::new(self, text);
        builtins
            .set_text(self)
            .with_durability(salsa::Durability::HIGH);

        let source = source_symbol(self, builtins);
        let source_members = source.arena[source.source_table].members.clone();

        self.builtins = Some(Builtins {
            integer: self.init_builtin(&source_members, "integer"),
            float: self.init_builtin(&source_members, "float"),
            boolean: self.init_builtin(&source_members, "bool"),
            string: self.init_builtin(&source_members, "string"),
            array: self.init_builtin(&source_members, "array"),
            table: self.init_builtin(&source_members, "table"),
            function: self.init_builtin(&source_members, "function_"),
            class: self.init_builtin(&source_members, "class_"),
            instance: self.init_builtin(&source_members, "instance"),
            generator: self.init_builtin(&source_members, "generator"),
            thread: self.init_builtin(&source_members, "thread"),
            weakref: self.init_builtin(&source_members, "weakref"),
        });
        builtins
    }

    fn init_builtin(&self, source_members: &SymbolTable, name: &str) -> SymbolTable {
        let Some(typ) = source_members.iter().find_map(|(symbol_name, id)| {
            if symbol_name != name {
                return None;
            }
            Some(id.get_data(self).typ)
        }) else {
            panic!("'{name}' is not contained inside builtins members");
        };

        let Type::Class(id) = typ else {
            panic!("'{name}' member is not of type 'class'");
        };

        id.get_data(self).members.clone()
    }

    pub fn init_squirrel_lib(&mut self, text: String) -> File {
        let lib = File::new(self, text);
        lib.set_text(self).with_durability(salsa::Durability::HIGH);
        source_symbol(self, lib);
        self.squirrel_lib = Some(lib);
        lib
    }

    pub fn init_vscript_lib(&mut self, text: String) -> File {
        let lib = File::new(self, text);
        lib.set_text(self).with_durability(salsa::Durability::HIGH);
        source_symbol(self, lib);
        self.vscript_lib = Some(lib);
        lib
    }
}

#[salsa::tracked(returns(ref))]
pub fn line_index(db: &dyn Db, file: File) -> LineIndex {
    LineIndex::new(file.text(db))
}

#[salsa::tracked(returns(ref))]
pub fn parse(db: &dyn Db, file: File) -> Parse {
    let now = Instant::now();
    let parse = Parse::new(file.text(db));
    eprintln!("Parsing took {:?}", now.elapsed());
    parse
}

#[salsa::tracked(returns(ref))]
pub fn source_symbol(db: &dyn Db, file: File) -> SourceSymbol {
    let now = Instant::now();
    let parse = parse(db, file);
    let source = Collector::symbol_from_source_file(db, file, parse.source_file());
    eprintln!("Source symbol took {:?}", now.elapsed());
    source
}
