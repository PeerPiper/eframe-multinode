//! The RdxApp struct is the main struct that holds all the plugins and their state.
#![allow(clippy::arc_with_non_send_sync)]

#[cfg(not(target_arch = "wasm32"))]
mod debouncer; // debouncer for tokio only

mod layer;

use crate::app::platform;

pub use layer::LayerPlugin;
use rdx::{
    layer::{rhai::Dynamic, Instantiator, Value},
    PluginDeets,
};
use std::{collections::HashMap, sync::Arc};
use std::{ops::Deref, sync::Mutex};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::Mutex as AsyncMutex;

#[cfg(not(target_arch = "wasm32"))]
mod state;
#[cfg(not(target_arch = "wasm32"))]
pub use state::State;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
pub use web::state::State;

use super::platform::piper::PeerPiper;

/// The RdxRunner struct is the main struct that holds all the plugins and their state.
pub struct RdxRunner {
    /// plugins is a hashmap of all the plugins that are loaded.
    ///
    /// Plugins > [PluginDeets] > [LayerPlugin] > [State] > [rdx::layer::rhai::Scope]
    pub(crate) plugins: HashMap<String, PluginDeets<State>>,
}

#[cfg(not(target_arch = "wasm32"))]
type PeerPiperWired = Arc<AsyncMutex<PeerPiper>>;
#[cfg(target_arch = "wasm32")]
type PeerPiperWired = Rc<RefCell<Option<PeerPiper>>>;

impl RdxRunner {
    pub fn new(
        peerpiper: PeerPiperWired,
        ctx: Option<egui::Context>,
        #[cfg(target_arch = "wasm32")] receiver: futures::channel::oneshot::Receiver<PeerPiper>,
    ) -> Self {
        let mut all_plugin_deets = HashMap::new();

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

        //#[cfg(not(target_arch = "wasm32"))]
        //let peerpiper = Arc::new(AsyncMutex::new(None));
        //#[cfg(target_arch = "wasm32")]
        //let peerpiper = Rc::new(RefCell::new(None));
        //
        //let peerpiper_clone = peerpiper.clone();
        //#[cfg(target_arch = "wasm32")]
        //let (sender, receiver) = futures::channel::oneshot::channel::<()>();
        //
        //platform::spawn(async move {
        //    let peerpiper = PeerPiper::new(commander);
        //    //let peerpiper = create_peerpiper().await.unwrap_or_else(|e| {
        //    //    panic!("Failed to create PeerPiper: {:?}", e);
        //    //});
        //    #[cfg(not(target_arch = "wasm32"))]
        //    peerpiper_clone.lock().await.replace(peerpiper);
        //    #[cfg(target_arch = "wasm32")]
        //    peerpiper_clone.borrow_mut().replace(peerpiper);
        //
        //    tracing::info!("PeerPiper created");
        //    #[cfg(target_arch = "wasm32")]
        //    sender.send(()).unwrap();
        //});

        // Since the browser is so much slower than native,
        // We need to gather up the arc_wallet and all the arc_plugins
        // and call them after we've received the signal that peerpiper is ready.

        // Instantiate the wallet_plugin
        let mut wallet_layer = LayerPlugin::new(
            wallet_bytes,
            State::new(wallet_name.to_string(), ctx.clone(), peerpiper.clone()),
            None,
            None,
        );
        let rdx_source = wallet_layer.call("load", &[]).unwrap();
        let Some(Value::String(rdx_source)) = rdx_source else {
            panic!("RDX Source should be a string");
        };

        let arc_wallet = Arc::new(Mutex::new(wallet_layer));

        let arc_collection = Arc::new(Mutex::new(HashMap::new()));

        arc_collection
            .lock()
            .unwrap()
            .insert(wallet_name.to_string(), arc_wallet.clone());

        let mut wallet_deets = PluginDeets::new(
            wallet_name.to_string(),
            arc_wallet.clone(),
            rdx_source.to_string(),
        );

        // a closure that enables us to register a function by name with zero arguments
        let register = |deets: &mut PluginDeets<State>, fn_name, args| {
            let plugin_clone = deets.plugin.clone();
            deets.engine.register_fn(fn_name, move || {
                let res = {
                    //let plugin_clone = plugin_clone.deref();
                    let mut lock = plugin_clone.lock().unwrap();
                    lock.call(fn_name, args).unwrap()
                };

                res.map(|v| match v {
                    Value::Bool(b) => Dynamic::from(b),
                    Value::Option(ov) => match ov.deref().clone() {
                        Some(Value::Bool(b)) => Dynamic::from(b),
                        Some(Value::List(list)) => {
                            let list = list
                                .into_iter()
                                .map(|v| match v {
                                    Value::String(s) => Dynamic::from(s),
                                    Value::U8(u) => Dynamic::from(u),
                                    Value::Bool(b) => Dynamic::from(b),
                                    _ => Dynamic::from("Unsupported type"),
                                })
                                .collect::<Vec<_>>();
                            Dynamic::from(list)
                        }
                        _ => false.into(),
                    },
                    _ => false.into(),
                })
                .unwrap_or(false.into())
            });
        };

        register(&mut wallet_deets, "unlocked", &[]);

        all_plugin_deets.insert(wallet_name.to_string(), wallet_deets);

        // the rest of the plugins
        for (name, wasm_bytes) in builtins {
            tracing::info!("Loading plugin: {:?}", name);
            let mut plugin = LayerPlugin::new(
                wasm_bytes,
                State::new(name.to_string(), ctx.clone(), peerpiper.clone()),
                Some(arc_wallet.clone()),
                Some(peerpiper.clone()),
            );
            let rdx_source = plugin.call("load", &[]).unwrap();

            // if this is not wasm32 target_arch, we will have the init scope loaded
            // with State::new() and we can call the init() function here
            // so that the scope can be loaded into the plugin
            //
            // With wasm32, this init() will have to happen after the commander is ready
            #[cfg(not(target_arch = "wasm32"))]
            if let Err(e) = plugin.call("init", &[]) {
                // it's ok not to have an init function
                // the plugin just won't be initialized with any loaded scope
                tracing::warn!("Failed to call init on plugin: {:?}", e);
            }

            let Some(Value::String(rdx_source)) = rdx_source else {
                panic!("RDX Source should be a string");
            };

            let arc_plugin = Arc::new(Mutex::new(plugin));

            arc_collection
                .lock()
                .unwrap()
                .insert(name.to_string(), arc_plugin.clone());

            let mut plugin_deets =
                PluginDeets::new(name.to_string(), arc_plugin, rdx_source.to_string());

            // register get_mk with rhai, so we can bind it inthe plugin and call it from rhai scripts
            register(&mut plugin_deets, "getmk", &[]);

            all_plugin_deets.insert(name.to_string(), plugin_deets);
        }

        // For native arch, we set the peerpiper.plugins to the arc_collection
        #[cfg(not(target_arch = "wasm32"))]
        {
            let piper_clone = peerpiper.clone();
            platform::spawn(async move {
                let binding = piper_clone.lock().await;
                let mut hash_map = binding.plugins.lock().unwrap();
                *hash_map = arc_collection.lock().unwrap().clone();
            });
        }

        // clone the arc_collection, pass it into the spawned task,
        // wait for the commander receiver to be ready, then iterate over the arc_collection
        // then call state.init() on each plugin: layer_plugin.store().data()

        #[cfg(target_arch = "wasm32")]
        let arc_collection_clone = arc_collection.clone();

        // We only do this for the wasm32 target because the tokio is fast enough to not need it.
        // The browser on the other hand is slow and needs to wait for the commander to be ready.
        #[cfg(target_arch = "wasm32")]
        platform::spawn(async move {
            tracing::info!("Waiting for peerpiper commander to be ready");

            // grab the PeerPiper from the receiver
            let piper = receiver.await.unwrap();

            // set the plugins in the PeerPiper to the arc_collection
            let mut piper_plugins_lock = piper.plugins.lock().unwrap();
            *piper_plugins_lock = arc_collection_clone.lock().unwrap().clone();

            drop(piper_plugins_lock);

            // Update the Rc
            {
                peerpiper.borrow_mut().replace(piper);
            }

            // Initialize the plugins
            {
                for (name, arc_plugin) in peerpiper
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .plugins
                    .lock()
                    .unwrap()
                    .iter()
                {
                    tracing::info!("Initializing plugin: {:?}", name);
                    let mut plugin = arc_plugin.lock().unwrap();
                    plugin.store().data().init();
                    tracing::info!("Initialized plugin: {:?}", name);

                    // once the scope is loaded, we should call the init() function
                    // to intiiate the plugin with the given state
                    // wasm32 happens here, whereas native happens in the loop above after
                    // State::new() is called
                    if let Err(e) = plugin.call("init", &[]) {
                        // it's ok not to have an init function
                        // the plugin just won't be initialized with any loaded scope
                        tracing::warn!("Failed to call init on plugin: {:?}", e);
                    }
                }
            }
        });

        Self {
            plugins: all_plugin_deets,
        }
    }
}
