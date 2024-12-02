//! The RdxApp struct is the main struct that holds all the plugins and their state.
use rdx::{LayerPlugin, PluginDeets, State, Value};

use std::collections::HashMap;

/// The RdxRunner struct is the main struct that holds all the plugins and their state.
pub struct RdxRunner {
    pub(crate) plugins: HashMap<String, PluginDeets>,
}

impl Default for RdxRunner {
    fn default() -> Self {
        Self::new(None)
    }
}

impl RdxRunner {
    pub fn new(ctx: Option<egui::Context>) -> Self {
        let mut plugins = HashMap::new();
        for (name, wasm_bytes) in crate::BUILTIN_PLUGINS.iter() {
            let mut plugin = LayerPlugin::new(wasm_bytes, State::new(ctx.clone()));
            let rdx_source = plugin.call("load", &[]).unwrap();
            let Some(Value::String(rdx_source)) = rdx_source else {
                panic!("RDX Source should be a string");
            };
            plugins.insert(
                name.to_string(),
                PluginDeets::new(name.to_string(), plugin, rdx_source.to_string()),
            );
        }

        Self { plugins }
    }
}
