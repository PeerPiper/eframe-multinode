#![warn(clippy::all, rust_2018_idioms)]
#![allow(static_mut_refs)] // dirs crate has warnings that break the CI build.

// The build file results, builtin components, are included in the build script
// Build files are for the demos, so the wasm binaries can be included by default in the build
include!(concat!(env!("OUT_DIR"), "/builtin_components.rs"));

mod app;
pub use app::MultinodeApp;
