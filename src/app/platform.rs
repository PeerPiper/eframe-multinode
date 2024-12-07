//! Platform can be either Web or Native.
//!
//! This module provides a unified interface for both platforms.

/// Native Platform Module
#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native as platform;

/// Web Platform Module
#[cfg(target_arch = "wasm32")]
pub mod web;

/// Web Platform Module
#[cfg(target_arch = "wasm32")]
pub use web as platform;

pub(crate) use platform::{spawn, Platform, Settings};

//pub trait System {
///// Put bytes into the local system
//fn put(&self, bytes: Vec<u8>) -> Cid;
//}

pub mod peerpiper;

pub use platform::peerpiper::create_peerpiper;
pub use platform::StringStore;
