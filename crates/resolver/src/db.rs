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
    FinishedFile, Source, SourceSymbol, SymbolId,
    arena::{ArenaId, ClassId, FunctionId},
    resolver::Resolver,
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
    native_functions: FxHashMap<FunctionId, NativeFunction>,
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
    pub null: Builtin,
}

#[derive(Debug, Clone, Copy)]
pub enum NativeFunction {
    GetRootTable,
    GetConstTable,

    SetDelegate,
    Bindenv,

    NewThread,

    CopySelf,

    Array,
    ArrayExtend,
    // Need to find a way to check whether the type is explicit or not
    // ArrayFind,
    // ArrayAppend,
    // ArrayInsert,
    ArrayReturnItem,
    // ArrayPush,
    // ArrayResize,
    IncludeScript,
    DoIncludeScript,
}

#[salsa::db]
pub trait Db: salsa::Database {
    fn builtins(&self) -> Option<&Builtins>;
    fn get_script(&self, path: PathBuf) -> Result<File, String>;
    fn script_literals(&self) -> Vec<String>;
    fn squirrel_lib(&self) -> Option<File>;
    fn vscript_lib(&self) -> Option<File>;
    fn base_entity_class(&self) -> Option<ClassId>;
    fn check_native(&self, id: FunctionId) -> Option<NativeFunction>;
}

impl salsa::Database for Database {}

#[salsa::db]
impl Db for Database {
    fn get_script(&self, mut path: PathBuf) -> Result<File, String> {
        let scripts = self.scripts_dir()?;

        if path.extension().is_none() {
            path.set_extension("nut");
        } else {
            // Validate extension
            if path.extension().and_then(|e| e.to_str()) != Some("nut") {
                return Err("Script path must either have no or '.nut' extension".to_owned());
            }
        }

        if path.is_absolute() {
            return Err(format!(
                "Script path must be relative to '{}'",
                scripts.display()
            ));
        }

        let full_path = scripts.join(&path);

        if !full_path.exists() {
            return Err("Couldn't resolve path".to_owned());
        }

        self.get_file(&full_path).map_or_else(
            || {
                self.open_file(&full_path)
                    .ok_or_else(|| "Couldn't open file".to_owned())
            },
            Ok,
        )
    }

    fn script_literals(&self) -> Vec<String> {
        let Ok(scripts) = self.scripts_dir() else {
            return Vec::new();
        };

        self.all_files()
            .into_iter()
            .filter_map(|(_, path)| {
                let rel_path = path.strip_prefix(&scripts).ok()?;

                if rel_path.extension().and_then(|e| e.to_str()) != Some("nut") {
                    return None;
                }

                let forward_slash_path = rel_path
                    .with_extension("")
                    .components()
                    .filter_map(|c| c.as_os_str().to_str())
                    .collect::<Vec<_>>()
                    .join("/");
                Some(forward_slash_path)
            })
            .collect()
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

    fn check_native(&self, id: FunctionId) -> Option<NativeFunction> {
        self.native_functions.get(&id).copied()
    }
}

impl Database {
    #[must_use]
    pub fn new(config: DbConfig) -> Self {
        let mut this = Self::default();
        if let Some(builtins_path) = config.builtins_path {
            this.init_builtins(&builtins_path);
        }
        if let Some(squirrel_lib_path) = config.squirrel_lib_path {
            this.init_squirrel_lib(&squirrel_lib_path);
        }
        if let Some(vscript_lib_path) = config.vscript_lib_path {
            this.init_vscript_lib(&vscript_lib_path);
        }
        this.tf2_root = config.tf2_root_path;
        this.load_all_scripts();

        this
    }

    pub fn update_tf2_root(&mut self, path: Option<PathBuf>) {
        self.tf2_root = path;
        self.load_all_scripts();
    }

    pub fn open_file(&self, path: &Path) -> Option<File> {
        let path = path.canonicalize().ok()?;
        if let Some(file) = self.path_to_file.borrow().get(&path) {
            return Some(*file);
        }

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

    fn scripts_dir(&self) -> Result<PathBuf, &'static str> {
        let Some(root) = &self.tf2_root else {
            return Err("Couldn't resolve path: TF2 installation path is not set");
        };

        let scripts = root.join("tf/scripts/vscripts");
        let Ok(scripts) = scripts.canonicalize() else {
            return Err(
                "Couldn't resolve path: TF2 installation path contains no 'tf/scripts/vscripts'",
            );
        };

        Ok(scripts)
    }

    fn load_all_scripts(&self) {
        let Ok(scripts) = self.scripts_dir() else {
            return;
        };

        for entry in walkdir::WalkDir::new(scripts)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().and_then(|e| e.to_str()) == Some("nut"))
        {
            self.open_file(&entry.into_path());
        }
    }

    fn init_builtins(&mut self, path: &Path) {
        let Some(builtins) = self.open_file(path) else {
            return;
        };

        builtins
            .set_text(self)
            .with_durability(salsa::Durability::HIGH);

        for (native_function, path) in [
            (
                NativeFunction::SetDelegate,
                ["table", "setdelegate"].as_slice(),
            ),
            (NativeFunction::Bindenv, ["function_", "bindenv"].as_slice()),
            // (NativeFunction::ArrayAppend, ["array", "append"].as_slice()),
            (NativeFunction::ArrayExtend, ["array", "extend"].as_slice()),
            // (NativeFunction::ArrayFind, ["array", "find"].as_slice()),
            // (NativeFunction::ArrayInsert, ["array", "insert"].as_slice()),
            (NativeFunction::ArrayReturnItem, ["array", "pop"].as_slice()),
            // (NativeFunction::ArrayPush, ["array", "push"].as_slice()),
            (
                NativeFunction::ArrayReturnItem,
                ["array", "remove"].as_slice(),
            ),
            // (NativeFunction::ArrayResize, ["array", "resize"].as_slice()),
            (NativeFunction::CopySelf, ["array", "filter"].as_slice()),
            (NativeFunction::CopySelf, ["array", "map"].as_slice()),
            (NativeFunction::CopySelf, ["array", "slice"].as_slice()),
        ] {
            let Some(symbol) = self.find_symbol(builtins, path) else {
                continue;
            };

            let Ok(function_id) = source_symbol(self, builtins).arena[symbol.idx()]
                .typ
                .to_function()
            else {
                eprintln!(
                    "Standard library symbol '{path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.native_functions.insert(function_id, native_function);
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
            null: self.init_builtin(builtins, "null_"),
        });
    }

    fn init_builtin(&self, file: File, name: &'static str) -> Builtin {
        let symbol = self
            .find_symbol(file, &[name])
            .expect("Builtins should always exist");

        let source = source_symbol(self, file);

        let Ok(id) = source.arena[symbol.idx()].typ.to_class() else {
            panic!("'{name}' member is not of type 'class'");
        };

        Builtin {
            symbol,
            members: to_flat_symbol_table(id.get_data(self).members.clone()),
        }
    }

    fn init_squirrel_lib(&mut self, path: &Path) {
        let Some(lib) = self.open_file(path) else {
            return;
        };

        lib.set_text(self).with_durability(salsa::Durability::HIGH);
        for (native_function, path) in [
            (NativeFunction::Array, ["array"].as_slice()),
            (NativeFunction::NewThread, ["newthread"].as_slice()),
            (NativeFunction::GetRootTable, ["getroottable"].as_slice()),
            (NativeFunction::GetConstTable, ["getconsttable"].as_slice()),
        ] {
            let Some(symbol) = self.find_symbol(lib, path) else {
                continue;
            };

            let Ok(id) = source_symbol(self, lib).arena[symbol.idx()]
                .typ
                .to_function()
            else {
                eprintln!(
                    "Standard library symbol '{path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.native_functions.insert(id, native_function);
        }

        self.squirrel_lib = Some(lib);
    }

    fn init_vscript_lib(&mut self, path: &Path) {
        let Some(lib) = self.open_file(path) else {
            return;
        };

        lib.set_text(self).with_durability(salsa::Durability::HIGH);

        for (native_function, path) in [
            (NativeFunction::IncludeScript, ["IncludeScript"].as_slice()),
            (
                NativeFunction::DoIncludeScript,
                ["DoIncludeScript"].as_slice(),
            ),
        ] {
            let Some(symbol) = self.find_symbol(lib, path) else {
                continue;
            };

            let Ok(id) = source_symbol(self, lib).arena[symbol.idx()]
                .typ
                .to_function()
            else {
                eprintln!(
                    "Standard library symbol '{path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.native_functions.insert(id, native_function);
        }

        if let Some(symbol) = self.find_symbol(lib, &["CBaseEntity"]) {
            if let Ok(id) = source_symbol(self, lib).arena[symbol.idx()].typ.to_class() {
                self.base_entity_class = Some(id);
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
                    if let Ok(id) = source.arena[id.idx()].typ.to_class() {
                        if id.file() != file {
                            eprintln!(
                                "Standard library symbol in '{path:?}' is defined externally"
                            );
                            return None;
                        }
                        &source.arena[id.idx()].members
                    } else {
                        eprintln!(
                            "Symbol '{}' in '{path:?}' is not of type 'class'",
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
    let source = Resolver::symbol_from_source_file(db, file, &parse.source_file());
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
