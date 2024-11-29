#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(target_os = "linux", allow(static_mut_refs))] // dirs crate has warnings that break the CI build.

mod app;
pub use app::MultinodeApp;
