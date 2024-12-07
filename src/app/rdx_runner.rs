//! The RdxApp struct is the main struct that holds all the plugins and their state.
#![allow(clippy::arc_with_non_send_sync)]

mod layer;
mod state;

use layer::LayerPlugin;
use rdx::{
    layer::{Instantiator, Value},
    PluginDeets,
};
use state::State;

use crate::app::platform::{self, create_peerpiper};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use std::{ops::Deref as _, sync::mpsc::channel};
use tokio::sync::Mutex as AsyncMutex;

/// The RdxRunner struct is the main struct that holds all the plugins and their state.
pub struct RdxRunner {
    /// plugins is a hashmap of all the plugins that are loaded.
    ///
    /// Plugins > [PluginDeets] > [LayerPlugin] > [State] > [rdx::layer::rhai::Scope]
    pub(crate) plugins: HashMap<String, PluginDeets<State>>,
}

impl Default for RdxRunner {
    fn default() -> Self {
        tracing::warn!("No context provided, creating empty RdxRunner");
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

        // TODO: get each layer state from previous, or default()
        let (tx, rx) = channel(); // Channel to receive the peerpiper
        platform::spawn(async move {
            let peerpiper = create_peerpiper().await.unwrap_or_else(|e| {
                panic!("Failed to create PeerPiper: {:?}", e);
            });
            tx.send(peerpiper).unwrap();
        });

        let peerpiper = Arc::new(AsyncMutex::new(rx.recv().unwrap()));

        // Instantiate the wallet_plugin
        let mut wallet_layer = LayerPlugin::new(
            wallet_bytes,
            State::new(
                wallet_name.to_string(),
                ctx.clone(),
                Some(peerpiper.clone()),
            ),
            None,
        );
        let rdx_source = wallet_layer.call("load", &[]).unwrap();
        let Some(Value::String(rdx_source)) = rdx_source else {
            panic!("RDX Source should be a string");
        };

        let arc_wallet = Arc::new(Mutex::new(
            Box::new(wallet_layer) as Box<dyn Instantiator<_>>
        ));

        let mut deets = PluginDeets::new(
            wallet_name.to_string(),
            arc_wallet.clone(),
            rdx_source.to_string(),
        );

        let plugin_clone = send_wrapper::SendWrapper::new(arc_wallet.clone());
        deets.engine.register_fn("unlocked", move || {
            let plugin_clone = plugin_clone.deref();
            let mut lock = plugin_clone.lock().unwrap();
            let res = lock.call("unlocked", &[]).unwrap();
            tracing::info!("Locked response: {:?}", res);
            // if res is Some, unwrap and return it. If none, return false.
            res.map(|v| match v {
                Value::Bool(b) => b,
                _ => false,
            })
            .unwrap_or(false)
        });

        plugins.insert(wallet_name.to_string(), deets);

        // the rest of the plugins
        for (name, wasm_bytes) in builtins {
            let mut plugin = LayerPlugin::new(
                wasm_bytes,
                State::new(name.to_string(), ctx.clone(), Some(peerpiper.clone())),
                Some(arc_wallet.clone()),
            );
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
