use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    time::Instant,
};

use ::line_index::LineIndex;
use rustc_hash::FxHashMap;
use salsa::Setter;
use sq_3_parser::Parse;

use crate::{
    FinishedFile, Source, SourceSymbol, SymbolId, Type,
    arena::{ArenaId, FunctionId},
    collector::Collector,
    symbol::{FlatSymbolTable, SymbolTable, to_flat_symbol_table},
};

#[salsa::input]
#[derive(Debug)]
pub struct File {
    #[returns(ref)]
    pub text: String,
}

#[derive(Default)]
pub struct DbConfig {
    pub tf2_root_path: Option<PathBuf>,
    pub builtins_path: Option<PathBuf>,
    pub squirrel_lib_path: Option<PathBuf>,
    pub vscript_lib_path: Option<PathBuf>,
}

#[salsa::db]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
    path_to_file: RefCell<FxHashMap<PathBuf, File>>,
    file_to_path: RefCell<FxHashMap<File, PathBuf>>,
    builtins: Option<Builtins>,
    tf2_root: Option<PathBuf>,
    squirrel_lib: Option<File>,
    vscript_lib: Option<File>,
    special_functions: FxHashMap<FunctionId, SpecialFunction>,
}

pub struct Builtin {
    pub symbol: SymbolId,
    pub members: FlatSymbolTable,
}

pub struct Builtins {
    pub integer: Builtin,
    pub float: Builtin,
    pub boolean: Builtin,
    pub string: Builtin,
    pub array: Builtin,
    pub table: Builtin,
    pub function: Builtin,
    pub class: Builtin,
    pub instance: Builtin,
    pub generator: Builtin,
    pub thread: Builtin,
    pub weakref: Builtin,
}

#[derive(Debug, Clone, Copy)]
pub enum SpecialFunction {
    GetRootTable,
    GetConstTable,
    IncludeScript,
    DoIncludeScript,
}

#[derive(Debug)]
pub enum ScriptResolutionError {
    WrongExtension,
    AbsolutePath,
    NoRootAssigned,
    DoesntExist,
}

#[salsa::db]
pub trait Db: salsa::Database {
    fn builtins(&self) -> Option<&Builtins>;
    fn get_script(&self, path: PathBuf) -> Result<File, ScriptResolutionError>;
    fn squirrel_lib(&self) -> Option<File>;
    fn vscript_lib(&self) -> Option<File>;
    fn check_special(&self, id: FunctionId) -> Option<SpecialFunction>;
}

impl salsa::Database for Database {}
#[salsa::db]
impl Db for Database {
    fn get_script(&self, mut path: PathBuf) -> Result<File, ScriptResolutionError> {
        if path.extension().is_none() {
            path.set_extension("nut");
        } else {
            // Validate extension
            if path.extension().and_then(|e| e.to_str()) != Some("nut") {
                return Err(ScriptResolutionError::WrongExtension);
            }
        }

        if path.is_absolute() {
            return Err(ScriptResolutionError::AbsolutePath);
        }

        let Some(root) = &self.tf2_root else {
            return Err(ScriptResolutionError::NoRootAssigned);
        };

        let scripts = root.join(PathBuf::from("tf/scripts/vscripts"));

        let full_path = scripts.join(&path);

        if !full_path.exists() {
            return Err(ScriptResolutionError::DoesntExist);
        }

        if let Some(file) = self.get_file(&full_path) {
            Ok(file)
        } else {
            self.open_file(full_path)
                .ok_or(ScriptResolutionError::DoesntExist)
        }
    }

    fn builtins(&self) -> Option<&Builtins> {
        self.builtins.as_ref()
    }

    fn squirrel_lib(&self) -> Option<File> {
        self.squirrel_lib
    }

    fn vscript_lib(&self) -> Option<File> {
        self.vscript_lib
    }

    fn check_special(&self, id: FunctionId) -> Option<SpecialFunction> {
        self.special_functions.get(&id).cloned()
    }
}

impl Database {
    pub fn new(config: DbConfig) -> Self {
        let mut this = Self::default();
        if let Some(builtins_path) = config.builtins_path {
            this.init_builtins(builtins_path);
        }
        if let Some(squirrel_lib_path) = config.squirrel_lib_path {
            this.init_squirrel_lib(squirrel_lib_path);
        }
        if let Some(vscript_lib_path) = config.vscript_lib_path {
            this.init_vscript_lib(vscript_lib_path);
        }
        this.tf2_root = config.tf2_root_path;
        this.load_all_scripts();

        this
    }

    pub fn open_file(&self, path: PathBuf) -> Option<File> {
        let text = std::fs::read_to_string(path.clone()).ok()?;
        Some(self.open_file_with_text(path, text))
    }

    pub fn open_file_with_text(&self, path: PathBuf, text: String) -> File {
        let file = File::new(self, text);
        self.path_to_file.borrow_mut().insert(path.clone(), file);
        self.file_to_path.borrow_mut().insert(file, path);
        file
    }

    pub fn all_files(&self) -> Vec<(File, PathBuf)> {
        self.file_to_path
            .borrow()
            .iter()
            .map(|(file, path)| (*file, path.clone()))
            .collect()
    }

    pub fn get_file(&self, path: &Path) -> Option<File> {
        self.path_to_file.borrow().get(path).cloned()
    }

    // Can't return a reference because of the ref cell
    pub fn get_path(&self, file: File) -> Option<PathBuf> {
        self.file_to_path.borrow().get(&file).cloned()
    }

    fn load_all_scripts(&self) {
        let Some(root) = &self.tf2_root else { return };
        let scripts = root.join("tf/scripts/vscripts");

        for entry in walkdir::WalkDir::new(scripts)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|e| e.to_str()) == Some("nut"))
        {
            self.open_file(entry.into_path());
        }
    }

    fn init_builtins(&mut self, path: PathBuf) {
        let Some(builtins) = self.open_file(path) else {
            return;
        };

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
    }

    fn init_builtin(&self, source_members: &SymbolTable, name: &str) -> Builtin {
        let Some(symbol_id) = source_members.iter().find_map(|(symbol_name, ids)| {
            if symbol_name != name {
                return None;
            }

            if ids.len() > 1 {
                eprintln!("Multiple definitions for the symbol '{name}'")
            }

            Some(ids[0])
        }) else {
            panic!("'{name}' is not contained inside builtins members");
        };

        let typ = symbol_id.get_data(self).typ;

        let Type::Class(id) = typ else {
            panic!("'{name}' member is not of type 'class'");
        };

        Builtin {
            symbol: symbol_id,
            members: to_flat_symbol_table(id.get_data(self).members.clone()),
        }
    }

    fn init_squirrel_lib(&mut self, path: PathBuf) {
        let Some(lib) = self.open_file(path) else {
            return;
        };

        lib.set_text(self).with_durability(salsa::Durability::HIGH);
        let special_functions = self.find_special_functions(lib);
        for (key, value) in special_functions {
            self.special_functions.insert(key, value);
        }

        self.squirrel_lib = Some(lib);
    }

    fn init_vscript_lib(&mut self, path: PathBuf) {
        let Some(lib) = self.open_file(path) else {
            return;
        };

        lib.set_text(self).with_durability(salsa::Durability::HIGH);

        let special_functions = self.find_special_functions(lib);
        for (key, value) in special_functions {
            self.special_functions.insert(key, value);
        }

        self.vscript_lib = Some(lib);
    }

    fn find_special_functions(&self, file: File) -> Vec<(FunctionId, SpecialFunction)> {
        let source = source_symbol(self, file);
        let source_members = &source.arena[source.source_table].members;
        source_members.into_iter().filter_map(|(name, ids)| {
            // Both squirrel and vscript files use the same match, this works fine since
            // there's no name clashing for special functions but it might not be the best
            // practice
            let kind = match name.as_str() {
                "getroottable" => SpecialFunction::GetRootTable,
                "getconsttable" => SpecialFunction::GetConstTable,
                "IncludeScript" => SpecialFunction::IncludeScript,
                "DoIncludeScript" => SpecialFunction::DoIncludeScript,
                _ => return None,
            };

            if ids.len() > 1 {
                eprintln!("Multiple definitions for the standard library symbol '{name}'");
            }

            let id = ids.last().unwrap();

            if id.file() != file {
                eprintln!("Standard library symbol '{name}' is defined externally");
                return None;
            }

            let Type::Function(function_id) = source.arena[id.idx()].typ else {
                eprintln!(
                    "Standard library symbol '{name}' has the type of '{}'. (Expected 'function')",
                    source.arena[id.idx()].typ
                );
                return None;
            };

            Some((function_id, kind))
        }).collect()
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

#[salsa::tracked]
pub fn top_root_members(db: &dyn Db, file: File) -> FlatSymbolTable {
    let finished_file = FinishedFile::new(db, file);
    to_flat_symbol_table(finished_file.additional_table_members(finished_file.root_table()))
}

#[salsa::tracked]
pub fn top_source_members(db: &dyn Db, file: File) -> FlatSymbolTable {
    let finished_file = FinishedFile::new(db, file);
    to_flat_symbol_table(finished_file.additional_table_members(finished_file.source_table()))
}

#[salsa::tracked]
pub fn top_const_members(db: &dyn Db, file: File) -> FlatSymbolTable {
    let finished_file = FinishedFile::new(db, file);
    to_flat_symbol_table(finished_file.additional_table_members(finished_file.const_table()))
}

#[salsa::tracked]
pub fn top_source_and_root_members(db: &dyn Db, file: File) -> FlatSymbolTable {
    top_source_members(db, file)
        .into_iter()
        .chain(top_root_members(db, file))
        .collect()
}

#[salsa::tracked]
pub fn top_source_and_const_members(db: &dyn Db, file: File) -> FlatSymbolTable {
    top_source_members(db, file)
        .into_iter()
        .chain(top_const_members(db, file))
        .collect()
}
