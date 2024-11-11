//! Platform Module

/// Native Platform Module
#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
use native as platform;

/// Web Platform Module
#[cfg(target_arch = "wasm32")]
mod web;

/// Web Platform Module
#[cfg(target_arch = "wasm32")]
use web as platform;

pub(crate) use platform::Platform;
