//! The RdxApp struct is the main struct that holds all the plugins and their state.
#![allow(clippy::arc_with_non_send_sync)]

mod layer;

use layer::LayerPlugin;
use rdx::{
    layer::{Instantiator, Value},
    PluginDeets,
};

#[cfg(not(target_arch = "wasm32"))]
mod state;

#[cfg(not(target_arch = "wasm32"))]
use state::State;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
use web::state::State;

use crate::app::platform::{self, create_peerpiper};

use std::{collections::HashMap, sync::Arc};
use std::{ops::Deref as _, sync::Mutex};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::Mutex as AsyncMutex;

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

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

        let (wallet_name, wallet_bytes) = builtins
            // find position and remove it into wallet_plugin var
            .iter()
            // starts with `wallet_plugin`
            .position(|(name, _)| *name == "wallet_plugin.wasm")
            .map(|i| builtins.remove(i))
            .unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        let peerpiper = Arc::new(AsyncMutex::new(None));
        #[cfg(target_arch = "wasm32")]
        let peerpiper = Rc::new(RefCell::new(None));

        let peerpiper_clone = peerpiper.clone();
        #[cfg(target_arch = "wasm32")]
        let (sender, receiver) = futures::channel::oneshot::channel::<()>();

        platform::spawn(async move {
            let peerpiper = create_peerpiper().await.unwrap_or_else(|e| {
                panic!("Failed to create PeerPiper: {:?}", e);
            });
            #[cfg(not(target_arch = "wasm32"))]
            peerpiper_clone.lock().await.replace(peerpiper);
            #[cfg(target_arch = "wasm32")]
            peerpiper_clone.borrow_mut().replace(peerpiper);

            log::info!("PeerPiper created");
            #[cfg(target_arch = "wasm32")]
            sender.send(()).unwrap();
        });

        // Since the browser is so much slower than native,
        // We need to gather up the arc_wallet and all the arc_plugins
        // and call them after we've received the signal that peerpiper is ready.
        #[cfg(target_arch = "wasm32")]
        let arc_collection = Arc::new(Mutex::new(HashMap::new()));

        // Instantiate the wallet_plugin
        let mut wallet_layer = LayerPlugin::new(
            wallet_bytes,
            State::new(wallet_name.to_string(), ctx.clone(), peerpiper.clone()),
            None,
        );
        let rdx_source = wallet_layer.call("load", &[]).unwrap();
        let Some(Value::String(rdx_source)) = rdx_source else {
            panic!("RDX Source should be a string");
        };

        let arc_wallet = Arc::new(Mutex::new(wallet_layer));

        // add to arc_collection
        #[cfg(target_arch = "wasm32")]
        {
            arc_collection
                .lock()
                .unwrap()
                .insert(wallet_name.to_string(), arc_wallet.clone());
        }

        let mut wallet_deets =
            PluginDeets::new(wallet_name.to_string(), arc_wallet, rdx_source.to_string());

        let arc_wallet = wallet_deets.plugin.clone();

        let plugin_clone = send_wrapper::SendWrapper::new(arc_wallet.clone());
        wallet_deets.engine.register_fn("unlocked", move || {
            let res = {
                let plugin_clone = plugin_clone.deref();
                let mut lock = plugin_clone.lock().unwrap();
                lock.call("unlocked", &[]).unwrap()
            };
            // if res is Some, unwrap and return it. If none, return false.
            res.map(|v| match v {
                Value::Bool(b) => b,
                _ => false,
            })
            .unwrap_or(false)
        });

        plugins.insert(wallet_name.to_string(), wallet_deets);

        // the rest of the plugins
        for (name, wasm_bytes) in builtins {
            log::info!("Loading plugin: {:?}", name);
            let mut plugin = LayerPlugin::new(
                wasm_bytes,
                State::new(name.to_string(), ctx.clone(), peerpiper.clone()),
                Some(arc_wallet.clone()),
            );
            let rdx_source = plugin.call("load", &[]).unwrap();
            let Some(Value::String(rdx_source)) = rdx_source else {
                panic!("RDX Source should be a string");
            };

            let arc_plugin = Arc::new(Mutex::new(plugin));

            // add to arc_collection
            #[cfg(target_arch = "wasm32")]
            {
                arc_collection
                    .lock()
                    .unwrap()
                    .insert(name.to_string(), arc_plugin.clone());
            }

            plugins.insert(
                name.to_string(),
                PluginDeets::new(name.to_string(), arc_plugin, rdx_source.to_string()),
            );
        }

        // clone the arc_collection, pass it into the spawned task,
        // wait for the peerpiper receiver to be ready, then iterate over the arc_collection
        // then call state.init() on each plugin: layer_plugin.store().data()

        #[cfg(target_arch = "wasm32")]
        let arc_collection_clone = arc_collection.clone();

        // We only do this for the wasm32 target because the tokio is fast enough to not need it.
        // The browser on the other hand is slow and needs to wait for the PeerPiper to be ready.
        #[cfg(target_arch = "wasm32")]
        platform::spawn(async move {
            tracing::info!("Waiting for PeerPiper to be ready");
            receiver.await.unwrap();
            tracing::info!("PeerPiper is ready");
            let lock = arc_collection_clone.lock().unwrap();
            for (name, arc_plugin) in lock.iter() {
                tracing::info!("Initializing plugin: {:?}", name);
                log::info!("Initializing plugin: {:?}", name);
                let plugin = arc_plugin.lock().unwrap();
                plugin.store().data().init();
                //drop(plugin);
                tracing::info!("Initialized plugin: {:?}", name);
                log::info!("Initialized plugin: {:?}", name);
            }
        });

        Self { plugins }
    }
}
