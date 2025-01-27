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

#[cfg(not(target_arch = "wasm32"))]
type PeerPiperWired = Arc<AsyncMutex<PeerPiper>>;
#[cfg(target_arch = "wasm32")]
type PeerPiperWired = Rc<RefCell<Option<PeerPiper>>>;

/// The RdxRunner struct is the main struct that holds all the plugins and their state.
pub struct RdxRunner {
    /// The PeerPiper pointer
    peerpiper: PeerPiperWired,
    /// plugins is a hashmap of all the plugins that are loaded.
    ///
    /// Plugins > [PluginDeets] > [LayerPlugin] > [State] > [rdx::layer::rhai::Scope]
    pub(crate) plugins: HashMap<String, PluginDeets<State>>,
    /// Reference counter for the wallet plugin
    pub(crate) arc_wallet: Option<Arc<Mutex<dyn Instantiator<State>>>>,
    /// egui Context option, so a change in State can request a repaint
    ctx: Option<egui::Context>,
    /// For wasm32, we need to wait for the receiver to be ready before we can use PeerPiper
    #[cfg(target_arch = "wasm32")]
    receiver: Option<futures::channel::oneshot::Receiver<PeerPiper>>,
}

impl RdxRunner {
    pub fn new(
        peerpiper: PeerPiperWired,
        ctx: Option<egui::Context>,
        #[cfg(target_arch = "wasm32")] receiver: futures::channel::oneshot::Receiver<PeerPiper>,
    ) -> Self {
        let all_plugin_deets = HashMap::new();

        Self {
            peerpiper,
            plugins: all_plugin_deets,
            arc_wallet: None,
            ctx,
            #[cfg(target_arch = "wasm32")]
            receiver: Some(receiver),
        }
    }

    /// Loads and initializes a plugin into the RdxRunner.
    /// Always load the wallet plugin first, so that you can pass the returned arc_wallet to the other plugins,
    /// in order to give them access to the wallet functions like `getmk` and `prove`
    pub fn load(&mut self, name: &str, wasm_bytes: &[u8]) -> Arc<Mutex<LayerPlugin<State>>> {
        tracing::info!("Loading plugin: {:?}", name);

        // If a plugin has access to the wallet,
        // then it has access to peerpiper as well.
        // If the wallet is None, then the plugin will not have access to peerpiper.
        // This is because we typically don't want to give the wallet itself access to peerpiper,
        // as we don't want our wallet o have network access.
        //
        // Wallet plugins can still emit data, but it is the RDX runner who decides how this data
        // is handled. Whereas direct peerpiper access from the plugin, the plugin can make
        // whatever command orders it wants.
        let commander: Option<PeerPiperWired> = match self.arc_wallet {
            Some(_) => Some(self.peerpiper.clone()),
            None => None,
        };

        let mut plugin = LayerPlugin::new(
            wasm_bytes,
            State::new(name.to_string(), self.ctx.clone(), self.peerpiper.clone()),
            self.arc_wallet.clone(),
            commander,
        );
        let rdx_source = plugin.call("load", &[]).unwrap();

        // If this is NOT wasm32 target_arch, we will have the init scope loaded
        // with State::new() already thus we can call the init() function here
        // so that the scope can be loaded into the plugin
        //
        // Contrary with wasm32, the init() will have to happen after the commander is ready
        #[cfg(not(target_arch = "wasm32"))]
        if let Err(e) = plugin.call("init", &[]) {
            // it's ok not to have an init function
            // the plugin just won't be initialized with any loaded scope
            tracing::warn!("Failed to call init on plugin: {:?}", e);
        }

        let Some(Value::String(rdx_source)) = rdx_source else {
            tracing::error!("RDX Source must be a string");
            // this will never happen, the WIT interface for `load` will always return a string
            return Arc::new(Mutex::new(plugin));
        };

        let arc_plugin = Arc::new(Mutex::new(plugin));

        // PeerPiper uses the plugins to handle network calls & validate data, messages, etc.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let piper_clone = self.peerpiper.clone();
            let name = name.to_string();
            let arc_plugin_clone = arc_plugin.clone();
            platform::spawn(async move {
                let binding = piper_clone.lock().await;
                let mut hash_map = binding.plugins.lock().unwrap();
                hash_map.insert(name, arc_plugin_clone);
            });
        }

        // for wasm32, we need to wait for the receiver to be ready before we can use PeerPiper
        // so we clone the arc_plugin and pass it into the spawned task
        #[cfg(target_arch = "wasm32")]
        {
            let peerpiper_clone = self.peerpiper.clone();
            let uninitilaized = { peerpiper_clone.borrow().is_none() && self.receiver.is_some() };
            if uninitilaized {
                // Take and Await the receiver to get the PeerPiper if it's not ready yet
                // if there's None receiver, then we're in trouble
                let receiver = self.receiver.take().unwrap();
                let name_clone = name.to_string();
                let arc_plugin_clone = arc_plugin.clone();

                platform::spawn(async move {
                    let piper = receiver.await.unwrap();
                    {
                        // Update the Rc<PeerPiper> with the new PeerPiper
                        peerpiper_clone.borrow_mut().replace(piper);
                        tracing::info!(
                            "PeerPiper replaced with received value by {:?}",
                            &name_clone
                        );
                        peerpiper_clone
                            .borrow_mut()
                            .as_ref()
                            .unwrap()
                            .plugins
                            .lock()
                            .unwrap()
                            .insert(name_clone.clone(), arc_plugin_clone.clone());

                        tracing::info!("Initializing plugin: {:?}", &name_clone);
                        let state = {
                            let plugin = arc_plugin_clone.lock().unwrap();
                            plugin.store().data().clone()
                        };
                        state.init().await;
                        tracing::info!("Initialized plugin: {:?}", name_clone);

                        // once the scope is loaded, we should call the init() function
                        // to intiiate the plugin with the given state
                        // wasm32 happens here, whereas native happens in the loop above after
                        // State::new() is called
                        let mut plugin = arc_plugin_clone.lock().unwrap();
                        if let Err(e) = plugin.call("init", &[]) {
                            // it's ok not to have an init function
                            // the plugin just won't be initialized with any loaded scope
                            tracing::warn!("Failed to call init on plugin: {:?}", e);
                        }
                    }
                });
            } else {
                let name_clone = name.to_string();
                let arc_plugin_clone = arc_plugin.clone();
                platform::spawn(async move {
                    // This is a bit of a hack because of the way async work in the browser.
                    // We need to ensure that the peerpiper is ready before we can use it.
                    // But we can't use async message passing in wasm32.
                    // So instead we sleep for 100ms in a loop until peerpiper is confirmed  `is_some()`
                    while peerpiper_clone.borrow().is_none() {
                        gloo_timers::future::TimeoutFuture::new(
                            web_time::Duration::from_millis(200).as_millis() as u32,
                        )
                        .await;
                    }

                    let binding = peerpiper_clone.borrow();
                    let piper = binding.as_ref().unwrap();
                    piper
                        .plugins
                        .lock()
                        .unwrap()
                        .insert(name_clone.clone(), arc_plugin_clone.clone());
                    let state = {
                        let plugin = arc_plugin_clone.lock().unwrap();
                        plugin.store().data().clone()
                    };
                    tracing::info!("Initializing plugin: {:?}", &name_clone);
                    state.init().await;
                    tracing::info!("Initialized plugin: {:?}", name_clone);

                    // once the scope is loaded, we should call the init() function
                    // to intiiate the plugin with the given state
                    // wasm32 happens here, whereas native happens in the loop above after
                    // State::new() is called
                    let mut plugin = arc_plugin_clone.lock().unwrap();
                    if let Err(e) = plugin.call("init", &[]) {
                        // it's ok not to have an init function
                        // the plugin just won't be initialized with any loaded scope
                        tracing::warn!("Failed to call init on plugin: {:?}", e);
                    }
                });
            }
        }

        let mut plugin_deets =
            PluginDeets::new(name.to_string(), arc_plugin.clone(), rdx_source.to_string());

        // register get_mk with rhai, so we can bind it inthe plugin and call it from rhai scripts
        register(&mut plugin_deets, "getmk".to_string(), vec![]);

        // Next we call "register" on the plugin to get any plugin-specific functions
        // that need to be registered with the rhai engine.
        // These functions can be called from Rhai scripts, and also called from RDX since they
        // are valid wasm function names.
        // This can fail, as not all plugins have functions to register.
        match arc_plugin.lock().unwrap().call("register", &[]) {
            // If Ok and a List of Strings, then iterate over these strings and rgister them
            Ok(Some(Value::List(list))) => {
                for fn_name in &list {
                    if let Value::String(fn_name) = fn_name {
                        tracing::info!(
                            "Registering function: {:?} from plugin: {:?}",
                            fn_name,
                            name
                        );
                        register(&mut plugin_deets, fn_name.to_string(), vec![]);
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("Failed to call register on plugin: {:?}", e);
            }
        }

        self.plugins.insert(name.to_string(), plugin_deets);

        // return the arc_plugin, because we need a handle to the arc_wallet plugin
        // to pass to the other plugins which call wallet functions.
        arc_plugin
    }
}

/// a function that enables us to register a function by name
fn register(deets: &mut PluginDeets<State>, fn_name: String, arguments: Vec<Value>) {
    let plugin_clone = deets.plugin.clone();
    deets
        .engine
        .borrow_mut()
        .register_fn(fn_name.clone(), move || {
            let res = {
                //let plugin_clone = plugin_clone.deref();
                let mut lock = plugin_clone.lock().unwrap();
                lock.call(&fn_name, &arguments).unwrap()
            };

            // a recurive function that converts List type into Dynamic type
            fn value_to_dynamic(v: Value) -> Dynamic {
                match v {
                    Value::Bool(b) => Dynamic::from(b),
                    Value::Option(ov) => match ov.deref().clone() {
                        Some(v) => value_to_dynamic(v),
                        None => false.into(),
                    },
                    Value::String(s) => Dynamic::from(s.to_string()),
                    Value::U8(u) => Dynamic::from(u),
                    Value::List(list) => {
                        let list = list.into_iter().map(value_to_dynamic).collect::<Vec<_>>();
                        Dynamic::from(list)
                    }
                    Value::Tuple(t) => {
                        let t = t.into_iter().map(value_to_dynamic).collect::<Vec<_>>();
                        Dynamic::from(t)
                    }
                    Value::F32(f) => Dynamic::from(f),
                    Value::F64(f) => Dynamic::from(f),
                    Value::U32(u) => Dynamic::from(u),
                    Value::U64(u) => Dynamic::from(u),
                    _ => false.into(),
                }
            }

            // convert the returned result into Dynamic type
            res.map(value_to_dynamic).unwrap_or(false.into())
        });
}
