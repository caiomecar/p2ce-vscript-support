use ::line_index::LineIndex;
use salsa::Setter;
use sq_3_parser::Parse;

use crate::{SourceSymbol, collector::Collector};

#[salsa::input]
#[derive(Debug)]
pub struct File {
    #[returns(ref)]
    pub text: String,
}

#[salsa::db]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
    pub stdlibs: Vec<File>,
}

#[salsa::db]
pub trait Db: salsa::Database {
    fn stdlibs(&self) -> &[File];
}

#[salsa::db]
impl salsa::Database for Database {}
#[salsa::db]
impl Db for Database {
    fn stdlibs(&self) -> &[File] {
        &self.stdlibs
    }
}

impl Database {
    pub fn init_stdlibs(&mut self, texts: Vec<String>) {
        for text in texts {
            let lib = File::new(self, text);
            lib.set_text(self).with_durability(salsa::Durability::HIGH);
            self.stdlibs.push(lib);
        }
    }
}

#[salsa::tracked(returns(ref))]
pub fn line_index(db: &dyn salsa::Database, file: File) -> LineIndex {
    LineIndex::new(file.text(db))
}

#[salsa::tracked(returns(ref))]
pub fn parse(db: &dyn salsa::Database, file: File) -> Parse {
    Parse::new(file.text(db))
}

#[salsa::tracked(returns(ref))]
pub fn source_symbol(db: &dyn Db, file: File) -> SourceSymbol {
    let parse = parse(db, file);
    Collector::symbol_from_source_file(db, file, parse.source_file())
}
