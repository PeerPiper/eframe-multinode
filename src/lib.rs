#![warn(clippy::all, rust_2018_idioms)]
#![allow(static_mut_refs)] // dirs crate has warnings that break the CI build.

mod app;
pub use app::MultinodeApp;
