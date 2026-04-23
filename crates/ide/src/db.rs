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
    arena::{ArenaId, ClassId, FunctionId},
    collector::Collector,
    symbol::{FlatSymbolTable, to_flat_symbol_table},
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
    base_entity_class: Option<ClassId>,
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
    SetDelegate,
    Bindenv,
    GetRootTable,
    GetConstTable,
    NewThread,
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
    fn base_entity_class(&self) -> Option<ClassId>;
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

        self.get_file(&full_path).map_or_else(
            || {
                self.open_file(full_path)
                    .ok_or(ScriptResolutionError::DoesntExist)
            },
            Ok,
        )
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

    fn base_entity_class(&self) -> Option<ClassId> {
        self.base_entity_class
    }

    fn check_special(&self, id: FunctionId) -> Option<SpecialFunction> {
        self.special_functions.get(&id).copied()
    }
}

impl Database {
    #[must_use]
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
        self.path_to_file.borrow().get(path).copied()
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
            .filter_map(std::result::Result::ok)
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

        for (special_function, path) in [
            (SpecialFunction::SetDelegate, ["table", "setdelegate"]),
            (SpecialFunction::Bindenv, ["function_", "bindenv"]),
        ] {
            let Some(symbol) = self.find_symbol(builtins, &path) else {
                continue;
            };

            let Type::Function(Some(function_id)) =
                source_symbol(self, builtins).arena[symbol.idx()].typ
            else {
                eprintln!(
                    "Standard library symbol '{path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.special_functions.insert(function_id, special_function);
        }

        self.builtins = Some(Builtins {
            integer: self.init_builtin(builtins, "integer"),
            float: self.init_builtin(builtins, "float"),
            boolean: self.init_builtin(builtins, "bool"),
            string: self.init_builtin(builtins, "string"),
            array: self.init_builtin(builtins, "array"),
            table: self.init_builtin(builtins, "table"),
            function: self.init_builtin(builtins, "function_"),
            class: self.init_builtin(builtins, "class_"),
            instance: self.init_builtin(builtins, "instance"),
            generator: self.init_builtin(builtins, "generator"),
            thread: self.init_builtin(builtins, "thread"),
            weakref: self.init_builtin(builtins, "weakref"),
        });
    }

    fn init_builtin(&self, file: File, name: &'static str) -> Builtin {
        let symbol = self
            .find_symbol(file, &[name])
            .expect("Builtins should always exist");

        let source = source_symbol(self, file);

        let Type::Class(Some(id)) = source.arena[symbol.idx()].typ else {
            panic!("'{name}' member is not of type 'class'");
        };

        Builtin {
            symbol,
            members: to_flat_symbol_table(id.get_data(self).members.clone()),
        }
    }

    fn init_squirrel_lib(&mut self, path: PathBuf) {
        let Some(lib) = self.open_file(path) else {
            return;
        };

        lib.set_text(self).with_durability(salsa::Durability::HIGH);
        for (special_function, path) in [
            (SpecialFunction::NewThread, ["newthread"]),
            (SpecialFunction::GetRootTable, ["getroottable"]),
            (SpecialFunction::GetConstTable, ["getconsttable"]),
        ] {
            let Some(symbol) = self.find_symbol(lib, &path) else {
                continue;
            };

            let Type::Function(Some(function_id)) =
                source_symbol(self, lib).arena[symbol.idx()].typ
            else {
                eprintln!(
                    "Standard library symbol '{path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.special_functions.insert(function_id, special_function);
        }

        self.squirrel_lib = Some(lib);
    }

    fn init_vscript_lib(&mut self, path: PathBuf) {
        let Some(lib) = self.open_file(path) else {
            return;
        };

        lib.set_text(self).with_durability(salsa::Durability::HIGH);

        for (special_function, path) in [
            (SpecialFunction::IncludeScript, ["IncludeScript"]),
            (SpecialFunction::DoIncludeScript, ["DoIncludeScript"]),
        ] {
            let Some(symbol) = self.find_symbol(lib, &path) else {
                continue;
            };

            let Type::Function(Some(function_id)) =
                source_symbol(self, lib).arena[symbol.idx()].typ
            else {
                eprintln!(
                    "Standard library symbol '{path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.special_functions.insert(function_id, special_function);
        }

        if let Some(symbol) = self.find_symbol(lib, &["CBaseEntity"]) {
            if let Type::Class(maybe_id) = source_symbol(self, lib).arena[symbol.idx()].typ {
                self.base_entity_class = maybe_id;
            } else {
                eprintln!(
                    "Standard library symbol 'CBaseEntity' has a wrong type. (Expected 'class')",
                );
            }
        }

        self.vscript_lib = Some(lib);
    }

    fn find_symbol(&self, file: File, path: &[&'static str]) -> Option<SymbolId> {
        let source = source_symbol(self, file);

        let mut last: Option<SymbolId> = None;
        'inner: for part in path {
            let members = match last {
                Some(id) => {
                    if let Type::Class(Some(id)) = source.arena[id.idx()].typ {
                        if id.file() != file {
                            eprintln!(
                                "Standard library symbol in '{path:?}' is defined externally"
                            );
                            return None;
                        }
                        &source.arena[id.idx()].members
                    } else {
                        eprintln!(
                            "Symbol '{}' in '{path:?}' is not of type class",
                            source.arena[id.idx()].name
                        );
                        return None;
                    }
                }
                None => &source.arena[source.source_table].members,
            };

            for (name, ids) in members {
                if name.as_ref() != *part {
                    continue;
                }

                if ids.len() > 1 {
                    eprintln!("Multiple definitions for the standard library symbol '{name}'");
                }

                let id = ids
                    .last()
                    .expect("SymbolTable vector contains at least 1 symbol");

                if id.file() != file {
                    eprintln!("Standard library symbol '{name}' is defined externally");
                    return None;
                }

                last = Some(*id);
                continue 'inner;
            }

            eprintln!("Couldn't find '{part}' in '{path:?}'");
            return None;
        }

        last
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
    let source = Collector::symbol_from_source_file(db, file, &parse.source_file());
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
