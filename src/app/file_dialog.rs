/// Native File Loader
#[cfg(not(target_arch = "wasm32"))]
mod native;

/// Web File Loader
#[cfg(target_arch = "wasm32")]
mod web;

/// Web File Loader
#[cfg(target_arch = "wasm32")]
pub use web::FileDialog;

/// use as file_dialog
#[cfg(not(target_arch = "wasm32"))]
pub use native::FileDialog;
