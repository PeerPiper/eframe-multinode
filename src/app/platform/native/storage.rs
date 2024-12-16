//! Impl Storage interface for native platform
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

/// Storage interface for native platform
#[derive(Debug, Default, Clone)]
pub struct StringStore {
    dir: PathBuf,
}

impl StringStore {
    // use dirs to get the default storage directory
    pub fn new() -> Self {
        let dir = dirs::data_dir().unwrap().join("multinode/cids/");
        fs::create_dir_all(&dir).unwrap();
        Self { dir }
    }
    /// Get the value from disk
    pub fn get_string(&self, key: &str) -> Option<String> {
        let path = PathBuf::from(key);
        let path = self.dir.join(path);
        fs::read_to_string(path).ok()
    }

    /// Save the value to disk
    pub fn set_string(&self, filename: &str, value: String) -> std::io::Result<()> {
        // create the file if it hasn't been created,
        // otherwise, open the file and write the value
        let path = self.dir.join(filename);
        let mut file = File::create(path.clone())?;
        tracing::info!("Saving to {:?}", path);
        file.write_all(value.as_bytes())
    }

    //pub fn flush(&mut self) {
    //    // rm the directory contents
    //    fs::remove_dir_all(&self.dir).unwrap();
    //}
}
