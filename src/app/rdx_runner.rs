//! The RdxApp struct is the main struct that holds all the plugins and their state.
#![allow(clippy::arc_with_non_send_sync)]

mod layer;

use layer::LayerPlugin;
use rdx::{
    layer::{Instantiator, Value},
    PluginDeets, State,
};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// The RdxRunner struct is the main struct that holds all the plugins and their state.
pub struct RdxRunner {
    pub(crate) plugins: HashMap<String, PluginDeets<State>>,
}

impl Default for RdxRunner {
    fn default() -> Self {
        Self::new(None)
    }
}

impl RdxRunner {
    pub fn new(ctx: Option<egui::Context>) -> Self {
        let mut plugins = HashMap::new();

        // the wallet_plugin needs to be separate from the other plugins
        // because it's exports (get-mk, prove) become the imports for
        // the other plugins.
        // Getthe single `wallet_plugin` from `BUILTIN_PLUGINS`,
        // and put the remaining in `rest` array.
        let mut builtins = crate::BUILTIN_PLUGINS.to_vec();

        // show builtins names (first in tuple) only
        tracing::info!(
            "Builtin plugins: {:?}",
            builtins.iter().map(|(name, _)| name).collect::<Vec<_>>()
        );

        let (wallet_name, wallet_bytes) = builtins
            // find position and remove it into wallet_plugin var
            .iter()
            // starts with `wallet_plugin`
            .position(|(name, _)| *name == "wallet_plugin.wasm")
            .map(|i| builtins.remove(i))
            .unwrap();

        // Instantiate the wallet_plugin
        let mut wallet_layer = LayerPlugin::new(wallet_bytes, State::new(ctx.clone()), None);
        tracing::info!("Wallet plugin instantiated");
        let rdx_source = wallet_layer.call("load", &[]).unwrap();
        let Some(Value::String(rdx_source)) = rdx_source else {
            panic!("RDX Source should be a string");
        };

        let arc_wallet = Arc::new(Mutex::new(
            Box::new(wallet_layer) as Box<dyn Instantiator<_>>
        ));

        plugins.insert(
            wallet_name.to_string(),
            PluginDeets::new(
                wallet_name.to_string(),
                arc_wallet.clone(),
                rdx_source.to_string(),
            ),
        );

        // the rest of the plugins
        for (name, wasm_bytes) in builtins {
            let mut plugin = LayerPlugin::new(
                wasm_bytes,
                State::new(ctx.clone()),
                Some(arc_wallet.clone()),
            );
            tracing::info!("Plugin {} instantiated", name);
            let rdx_source = plugin.call("load", &[]).unwrap();
            let Some(Value::String(rdx_source)) = rdx_source else {
                panic!("RDX Source should be a string");
            };
            plugins.insert(
                name.to_string(),
                PluginDeets::new(
                    name.to_string(),
                    Arc::new(Mutex::new(Box::new(plugin) as Box<dyn Instantiator<_>>)),
                    rdx_source.to_string(),
                ),
            );
        }

        Self { plugins }
    }
}
