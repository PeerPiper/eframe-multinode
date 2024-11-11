#[cfg(not(target_arch = "wasm32"))]
mod native;

// TODO: Web
// #[cfg(target_arch = "wasm32")]
// pub use multinode_web::file_dialog::FileDialog;

// use as file_dialog
#[cfg(not(target_arch = "wasm32"))]
pub use native::FileDialog;
