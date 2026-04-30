use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use db::{BaseDatabase, DashMap, File, Url};
use rustc_hash::FxHashMap;
use salsa::Setter;
use sq_3_parser::Parse;

use crate::{
    FinishedFile, Source, SourceSymbol, SymbolId,
    arena::{ArenaId, ClassId, FunctionId},
    resolver::Resolver,
    symbol::{FlatSymbolTable, to_flat_symbol_table},
};

// ---------------------------------------------------------------------------
// Concrete database
// ---------------------------------------------------------------------------

#[salsa::db]
#[derive(Default, Clone)]
pub struct Database {
    storage: salsa::Storage<Self>,
    files: DashMap<Url, File>,
    urls: DashMap<File, Url>,
    builtins: Option<Arc<Builtins>>,
    tf2_root: Option<PathBuf>,
    squirrel_lib: Option<File>,
    vscript_lib: Option<File>,
    base_entity_class: Option<ClassId>,
    native_functions: FxHashMap<FunctionId, NativeFunction>,
}

impl salsa::Database for Database {}
impl std::panic::RefUnwindSafe for Database {}

#[salsa::db]
impl BaseDatabase for Database {
    fn get_files(&self) -> &DashMap<Url, File> {
        &self.files
    }

    fn get_urls(&self) -> &DashMap<File, Url> {
        &self.urls
    }
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Builtin {
    pub symbol: SymbolId,
    pub members: FlatSymbolTable,
}

#[derive(Debug, Clone)]
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
    ArrayReturnItem,
    IncludeScript,
    DoIncludeScript,
}

// ---------------------------------------------------------------------------
// VScriptDatabase trait
// ---------------------------------------------------------------------------

#[salsa::db]
pub trait VScriptDatabase: BaseDatabase {
    fn builtins(&self) -> Option<&Builtins>;
    fn squirrel_lib(&self) -> Option<File>;
    fn vscript_lib(&self) -> Option<File>;
    fn base_entity_class(&self) -> Option<ClassId>;
    fn check_native(&self, id: FunctionId) -> Option<NativeFunction>;

    fn update_tf2_root(&mut self, path: Option<PathBuf>);

    /// # Errors
    /// If the script path is absent, has a bad extension, or can't be opened.
    fn get_script(&self, path: PathBuf) -> Result<File, String>;
    fn script_literals(&self) -> Vec<String>;
}

#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
#[salsa::db]
impl VScriptDatabase for Database {
    fn builtins(&self) -> Option<&Builtins> {
        self.builtins.as_deref()
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

    fn update_tf2_root(&mut self, path: Option<PathBuf>) {
        let is_some = path.is_some();
        self.tf2_root = path;
        if is_some {
            self.load_all_scripts();
        }
    }

    fn get_script(&self, mut path: PathBuf) -> Result<File, String> {
        let scripts = self.scripts_dir()?;

        if path.extension().is_none() {
            path.set_extension("nut");
        } else if path.extension().and_then(|e| e.to_str()) != Some("nut") {
            return Err("Script path must either have no or '.nut' extension".to_owned());
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

        let url = Url::from_file_path(&full_path)
            .map_err(|()| format!("Couldn't convert '{}' to URL", full_path.display()))?;

        if let Some(file) = self.get_file(&url) {
            return Ok(file);
        }

        let text =
            std::fs::read_to_string(&full_path).map_err(|_| "Couldn't read file".to_owned())?;

        Ok(self.open_file(&url, text))
    }

    fn script_literals(&self) -> Vec<String> {
        let Ok(scripts) = self.scripts_dir() else {
            return Vec::new();
        };

        self.get_files()
            .iter()
            .filter_map(|entry| {
                // Convert URL back to path only for the prefix stripping
                let path = entry.key().to_file_path().ok()?;
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
}

pub struct VScriptDbConfig {
    pub tf2_root_path: Option<PathBuf>,
    pub builtins_path: Option<PathBuf>,
    pub squirrel_lib_path: Option<PathBuf>,
    pub vscript_lib_path: Option<PathBuf>,
}

impl Database {
    #[must_use]
    pub fn new(config: VScriptDbConfig) -> Self {
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

        this.update_tf2_root(config.tf2_root_path);

        this
    }

    fn scripts_dir(&self) -> Result<PathBuf, String> {
        let Some(root) = &self.tf2_root else {
            return Err("Couldn't resolve path: TF2 installation path is not set".to_owned());
        };

        let scripts = root.join("tf/scripts/vscripts");
        scripts.canonicalize().map_err(|_| {
            "Couldn't resolve path: TF2 installation path contains no 'tf/scripts/vscripts'"
                .to_owned()
        })
    }

    fn load_all_scripts(&self) {
        let Ok(scripts) = self.scripts_dir() else {
            return;
        };

        for entry in walkdir::WalkDir::new(&scripts)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().and_then(|e| e.to_str()) == Some("nut"))
        {
            let path = entry.into_path();
            let Ok(url) = Url::from_file_path(&path) else {
                continue;
            };
            if self.get_file(&url).is_some() {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            self.open_file(&url, text);
        }
    }

    /// Converts a `&Path` to a URL and opens the file.
    /// Used only during init where we receive `PathBuf` from config.
    fn open_file_from_path(&self, path: &Path) -> Option<File> {
        let path = path.canonicalize().ok()?;
        let url = Url::from_file_path(&path).ok()?;

        if let Some(file) = self.get_file(&url) {
            return Some(file);
        }

        let text = std::fs::read_to_string(&path).ok()?;
        Some(self.open_file(&url, text))
    }

    fn init_builtins(&mut self, path: &Path) {
        let Some(builtins) = self.open_file_from_path(path) else {
            return;
        };

        builtins
            .set_text(self)
            .with_durability(salsa::Durability::HIGH);

        for (native_function, sym_path) in [
            (
                NativeFunction::SetDelegate,
                ["table", "setdelegate"].as_slice(),
            ),
            (NativeFunction::Bindenv, ["function_", "bindenv"].as_slice()),
            (NativeFunction::ArrayExtend, ["array", "extend"].as_slice()),
            (NativeFunction::ArrayReturnItem, ["array", "pop"].as_slice()),
            (
                NativeFunction::ArrayReturnItem,
                ["array", "remove"].as_slice(),
            ),
            (NativeFunction::CopySelf, ["array", "filter"].as_slice()),
            (NativeFunction::CopySelf, ["array", "map"].as_slice()),
            (NativeFunction::CopySelf, ["array", "slice"].as_slice()),
        ] {
            let Some(symbol) = self.find_symbol(builtins, sym_path) else {
                continue;
            };

            let Ok(function_id) = source_symbol(self, builtins).arena[symbol.idx()]
                .typ
                .to_function()
            else {
                eprintln!(
                    "Standard library symbol '{sym_path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.native_functions.insert(function_id, native_function);
        }

        self.builtins = Some(Arc::new(Builtins {
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
        }));
    }

    fn init_builtin(&self, file: File, name: &'static str) -> Builtin {
        let symbol = self
            .find_symbol(file, &[name])
            .unwrap_or_else(|| panic!("Builtin '{name}' not found"));

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
        let Some(lib) = self.open_file_from_path(path) else {
            return;
        };

        lib.set_text(self).with_durability(salsa::Durability::HIGH);

        for (native_function, sym_path) in [
            (NativeFunction::Array, ["array"].as_slice()),
            (NativeFunction::NewThread, ["newthread"].as_slice()),
            (NativeFunction::GetRootTable, ["getroottable"].as_slice()),
            (NativeFunction::GetConstTable, ["getconsttable"].as_slice()),
        ] {
            let Some(symbol) = self.find_symbol(lib, sym_path) else {
                continue;
            };

            let Ok(id) = source_symbol(self, lib).arena[symbol.idx()]
                .typ
                .to_function()
            else {
                eprintln!(
                    "Standard library symbol '{sym_path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.native_functions.insert(id, native_function);
        }

        self.squirrel_lib = Some(lib);
    }

    fn init_vscript_lib(&mut self, path: &Path) {
        let Some(lib) = self.open_file_from_path(path) else {
            return;
        };

        lib.set_text(self).with_durability(salsa::Durability::HIGH);

        for (native_function, sym_path) in [
            (NativeFunction::IncludeScript, ["IncludeScript"].as_slice()),
            (
                NativeFunction::DoIncludeScript,
                ["DoIncludeScript"].as_slice(),
            ),
        ] {
            let Some(symbol) = self.find_symbol(lib, sym_path) else {
                continue;
            };

            let Ok(id) = source_symbol(self, lib).arena[symbol.idx()]
                .typ
                .to_function()
            else {
                eprintln!(
                    "Standard library symbol '{sym_path:?}' has a wrong type. (Expected 'function')"
                );
                continue;
            };

            self.native_functions.insert(id, native_function);
        }

        if let Some(symbol) = self.find_symbol(lib, &["CBaseEntity"]) {
            match source_symbol(self, lib).arena[symbol.idx()].typ.to_class() {
                Ok(id) => self.base_entity_class = Some(id),
                Err(_) => eprintln!(
                    "Standard library symbol 'CBaseEntity' has a wrong type. (Expected 'class')"
                ),
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
                    let Ok(class_id) = source.arena[id.idx()].typ.to_class() else {
                        eprintln!(
                            "Symbol '{}' in '{path:?}' is not of type 'class'",
                            source.arena[id.idx()].name
                        );
                        return None;
                    };
                    if class_id.file() != file {
                        eprintln!("Standard library symbol in '{path:?}' is defined externally");
                        return None;
                    }
                    &source.arena[class_id.idx()].members
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
pub fn parse(db: &dyn BaseDatabase, file: File) -> Parse {
    let now = Instant::now();
    let result = Parse::new(file.text(db));
    eprintln!("Parsing took {:?}", now.elapsed());
    result
}

#[salsa::tracked(returns(ref))]
pub fn source_symbol(db: &dyn VScriptDatabase, file: File) -> SourceSymbol {
    let now = Instant::now();
    let p = parse(db, file);
    let source = Resolver::symbol_from_source_file(db, file, &p.source_file());
    eprintln!("Source symbol took {:?}", now.elapsed());
    source
}

#[salsa::tracked]
pub fn top_root_members(db: &dyn VScriptDatabase, file: File) -> FlatSymbolTable {
    let finished_file = FinishedFile::new(db, file);
    to_flat_symbol_table(finished_file.additional_table_members(finished_file.root_table()))
}

#[salsa::tracked]
pub fn top_source_members(db: &dyn VScriptDatabase, file: File) -> FlatSymbolTable {
    let finished_file = FinishedFile::new(db, file);
    to_flat_symbol_table(finished_file.additional_table_members(finished_file.source_table()))
}

#[salsa::tracked]
pub fn top_const_members(db: &dyn VScriptDatabase, file: File) -> FlatSymbolTable {
    let finished_file = FinishedFile::new(db, file);
    to_flat_symbol_table(finished_file.additional_table_members(finished_file.const_table()))
}

#[salsa::tracked]
pub fn top_source_and_root_members(db: &dyn VScriptDatabase, file: File) -> FlatSymbolTable {
    top_source_members(db, file)
        .into_iter()
        .chain(top_root_members(db, file))
        .collect()
}

#[salsa::tracked]
pub fn top_source_and_const_members(db: &dyn VScriptDatabase, file: File) -> FlatSymbolTable {
    top_source_members(db, file)
        .into_iter()
        .chain(top_const_members(db, file))
        .collect()
}
