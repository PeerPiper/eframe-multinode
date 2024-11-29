//! Platform Module

use std::future::Future;

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

pub(crate) use platform::{Platform, Settings};

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(f: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(f);
}

#[cfg(target_arch = "wasm32")]
pub fn spawn(f: impl Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(f);
}
