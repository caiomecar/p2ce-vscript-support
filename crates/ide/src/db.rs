use ::line_index::LineIndex;
use sq_3_parser::Parse;

use crate::{Collector, SourceSymbol};

#[salsa::input]
pub struct File {
    #[returns(ref)]
    pub text: String,
}

#[salsa::db]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for Database {}

#[salsa::tracked(returns(ref))]
pub fn line_index(db: &dyn salsa::Database, file: File) -> LineIndex {
    let text = file.text(db);
    LineIndex::new(text)
}

#[salsa::tracked(returns(ref))]
pub fn parse(db: &dyn salsa::Database, file: File) -> Parse {
    let text = file.text(db);
    Parse::new(text)
}

#[salsa::tracked(returns(ref))]
pub fn source_symbol(db: &dyn salsa::Database, file: File) -> SourceSymbol {
    let parse = parse(db, file).clone();
    Collector::symbol_from_source_file(parse.source_file())
}
