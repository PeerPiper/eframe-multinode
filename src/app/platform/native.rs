//! Platform code, specific to the native platform.
//!
//! For example, a native node will only be available here. Whereas the browser needs to connect
//! to a remote node, which is handled in the `web` module.
mod cloudflare;
mod error;
mod settings;
mod storage;

pub use error::Error;
pub use peerpiper_native::NativeBlockstore as Blockstore;
use peerpiper_native::NativeBlockstoreBuilder;
pub(crate) use settings::Settings;
pub use storage::StringStore;
use tokio::sync::Mutex as AsyncMutex;

use multiaddr::Multiaddr;
pub use peerpiper::core::events::PublicEvent;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use crate::app::rdx_runner::RdxRunner;

use super::piper::PeerPiper;

pub fn spawn(f: impl Future<Output = ()> + Send + 'static) {
    tracing::debug!("Spawning tokio task");
    tokio::spawn(f);
}

/// Track whether the Context has been set
#[derive(Debug, Default)]
pub(crate) struct ContextSet {
    /// Whether the Context has been set
    pub(crate) set: bool,

    /// The Context
    pub(crate) ctx: egui::Context,
}

impl ContextSet {
    /// Create a new ContextSet
    pub(crate) fn new() -> Self {
        Self {
            set: false,
            ..Default::default()
        }
    }

    /// Requests repaint. Successful only if the Context has been set.
    pub(crate) fn request_repaint(&self) {
        if self.set {
            self.ctx.request_repaint();
        }
    }
}

#[derive(Clone)]
pub(crate) struct Loader;

impl Loader {
    /// Load a plugin into the Platform
    pub fn load_plugin(&self, _name: String, _wasmm: Vec<u8>) {
        // call self.loader.load_plugin(name, wasm).await from this sync function using tokio
        //let mut loader = self.0.clone();
        //tokio::task::spawn(async move {
        //    if let Err(e) = loader.load_plugin(name, &wasm).await {
        //        tracing::error!("Failed to load plugin: {:?}", e);
        //    }
        //});
    }
}

pub(crate) struct Platform {
    log: Arc<Mutex<Vec<String>>>,

    /// Clone of the [egui::Context] so that the platform can trigger repaints
    ctx: Arc<Mutex<ContextSet>>,

    /// Gives the Platform the ability to load plugins
    pub loader: Loader,

    /// The address of the node
    addr: Arc<Mutex<Option<Multiaddr>>>,

    pub rdx_runner: RdxRunner,
}

impl Default for Platform {
    fn default() -> Self {
        // Collection of the Plugins
        let arc_collection_plugins = Arc::new(Mutex::new(HashMap::new()));

        let log = Arc::new(Mutex::new(Vec::new()));
        let ctx: Arc<Mutex<ContextSet>> = Arc::new(Mutex::new(ContextSet::new()));
        let addr = Arc::new(Mutex::new(None));

        // setup oneshot channel to pass blockstore from task back to this thread
        // the receiver must not be async
        let (tx, rx) = std::sync::mpsc::channel();

        super::spawn(async move {
            // 1. First we need a NativeBlockstore from NativeBlockstoreBuilder
            let blockstore = NativeBlockstoreBuilder::default().open().await.unwrap();
            tx.send(blockstore).unwrap();
        });

        let blockstore = rx.recv().unwrap();

        let peerpiper = Arc::new(AsyncMutex::new(PeerPiper::new(
            blockstore,
            arc_collection_plugins.clone(),
        )));

        let log_clone = log.clone();
        let ctx_clone = ctx.clone();
        let addr_clone = addr.clone();

        //let (on_event, mut plugin_evts) = mpsc::channel(16);
        let (on_event, mut rx_evts) = tokio::sync::mpsc::channel(32);

        // task for listening on plugin events and updating the log accoringly
        let peerpiper_clone = peerpiper.clone();
        super::spawn(async move {
            let libp2p_endpoints = vec![];
            let listen = match peerpiper_clone.lock().await.connect(libp2p_endpoints).await {
                Ok(listen) => listen,
                Err(e) => {
                    tracing::error!("Failed to connect to the network: {:?}", e);
                    return;
                }
            };

            listen(on_event);
        });

        tokio::task::spawn(async move {
            while let Some(event) = rx_evts.recv().await {
                let msg = format!("{:?}", event);
                tracing::debug!("Received event: {:?}", msg);

                let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");

                match event {
                    PublicEvent::ListenAddr { address: addr, .. } => {
                        tracing::debug!("Node Address: {}", &addr.to_string());
                        // Update rules: only update if this is an/ip6/ AND there isn't already an
                        // /ip6/ address in the addr

                        // first, check if empty
                        let mut lock = addr_clone.lock().unwrap();
                        if lock.is_none() {
                            *lock = Some(addr);
                        } else {
                            // not empty, check if it's an ip6 address repalcing an ip4 address
                            // if so, update it
                            let is_ip4 = lock.as_ref().unwrap().to_string().starts_with("/ip4/");
                            if is_ip4 && addr.to_string().starts_with("/ip6/") {
                                *lock = Some(addr);
                            }
                        }

                        log_clone.lock().unwrap().push(msg);
                    }
                    PublicEvent::Message { topic, data, peer } => {
                        // check if data decodes into a utf8 string, if not skip it
                        if let Ok(maybe_str) = std::str::from_utf8(&data) {
                            let msg = format!(
                                "[{}] 📬 Message from {}: {}: {}",
                                timestamp, peer, topic, maybe_str
                            );
                            tracing::debug!("Received Message: {:?}", msg);
                            log_clone.lock().unwrap().push(msg);
                        }
                    }
                    PublicEvent::Pong { peer, rtt } => {
                        let msg = format!("[{}] 🏓 Pong {}ms from {}", timestamp, rtt, peer);
                        log_clone.lock().unwrap().push(msg);
                    }
                    _ => {
                        log_clone.lock().unwrap().push(msg);
                    }
                }

                ctx_clone.lock().unwrap().request_repaint();
            }
        });

        let rdx_runner = RdxRunner::new(peerpiper, None);

        Self {
            log,
            ctx,
            addr,
            loader: Loader,
            rdx_runner,
        }
    }
}

impl Drop for Platform {
    fn drop(&mut self) {
        // Kill the server process using thread_handle
        self.close();
    }
}

impl Platform {
    ///// Load a plugin into the Platform put it in arc_collection of plugins
    //pub fn load_plugin(&self, name: String, wasm: Vec<u8>) {
    //    // call self.loader.load_plugin(name, wasm).await from this sync function using tokio
    //    let mut loader = self.loader.clone();
    //    tokio::task::spawn(async move {
    //        if let Err(e) = loader.load_plugin(name, &wasm).await {
    //            tracing::error!("Failed to load plugin: {:?}", e);
    //        }
    //    });
    //}

    /// Returns whether the ctx is set or not
    pub(crate) fn egui_ctx(&self) -> bool {
        self.ctx.lock().unwrap().set
    }

    /// Sets the ctx, so that the platform can trigger repaints on events.
    pub(crate) fn set_egui_ctx(&mut self, ctx: egui::Context) {
        self.ctx.lock().unwrap().ctx = ctx;
        self.ctx.lock().unwrap().set = true;
    }

    // This is where you would put platform-specific methods
    pub(crate) fn close(&mut self) {}

    /// Platform specific UI to show
    pub(crate) fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        // Bottom Up inner panel
        egui::TopBottomPanel::bottom("log")
            .resizable(true)
            .show_inside(ui, |ui| {
                ui.collapsing("Node Log", |ui| {
                    // SCROLLABLE SECTION for the log
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.vertical(|ui| {
                            for line in self.log.lock().unwrap().iter().rev() {
                                ui.label(line);
                            }
                        });
                    });
                });
            });
    }

    pub(crate) fn addr(&self) -> Option<Multiaddr> {
        self.addr.lock().unwrap().clone()
    }
}
