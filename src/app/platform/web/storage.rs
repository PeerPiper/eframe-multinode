//! String storage for web platform
//! uses wasm_bindgen for local storage to store key value pairs
use eframe::web_sys::Storage;
use send_wrapper::SendWrapper;

/// String storage for web platform
#[derive(Debug, Clone)]
pub struct StringStore {
    storage: SendWrapper<Storage>,
}

impl Default for StringStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StringStore {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let storage = window
            .local_storage()
            .expect("failed to get local storage")
            .expect("no local storage found");
        StringStore {
            storage: SendWrapper::new(storage),
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.storage.get_item(key).ok().flatten()
    }

    pub fn set_string(&self, key: &str, value: String) {
        self.storage
            .set_item(key, &value)
            .expect("failed to set item in local storage");
    }
}
