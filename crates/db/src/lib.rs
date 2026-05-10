pub use dashmap::DashMap;
pub use lsp_types::Url;

#[salsa::input]
#[derive(Debug)]
pub struct File {
    #[returns(ref)]
    pub text: String,
}
#[salsa::db]
pub trait BaseDatabase: salsa::Database {
    fn get_files(&self) -> &DashMap<Url, File>;
    fn get_urls(&self) -> &DashMap<File, Url>;

    fn get_file(&self, url: &Url) -> Option<File> {
        self.get_files().get(url).map(|file| *file)
    }

    fn get_url(&self, file: &File) -> Option<Url> {
        self.get_urls().get(file).map(|url| url.clone())
    }

    fn open_file(&self, url: &Url, text: String) -> File {
        let file = File::new(self, text);
        self.get_files().insert(url.clone(), file);
        self.get_urls().insert(file, url.clone());
        file
    }
}
