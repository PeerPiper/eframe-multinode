//! Impl Storage interface for native platform
use std::fs;
use std::path::PathBuf;

/// Storage interface for native platform
#[derive(Debug, Default, Clone)]
pub struct StringStore {
    dir: PathBuf,
}

impl StringStore {
    // use dirs to get the default storage directory
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dirs::data_dir()
            .unwrap()
            .join("multinode/cids/")
            .join(dir.into());
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
    pub fn set_string(&self, key: &str, value: String) {
        let path = PathBuf::from(key);
        let path = self.dir.join(path);
        tracing::debug!("Writing {:?} to {:?}", key, path);
        fs::write(path, value).unwrap();
    }

    pub fn flush(&mut self) {
        // rm the directory contents
        fs::remove_dir_all(&self.dir).unwrap();
    }
}
