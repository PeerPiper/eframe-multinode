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
    tracing::trace!("Spawning tokio task");
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

pub(crate) struct Platform {
    log: Arc<Mutex<Vec<String>>>,

    /// Clone of the [egui::Context] so that the platform can trigger repaints
    ctx: Arc<Mutex<ContextSet>>,

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
            let blockstore = NativeBlockstoreBuilder::default();
            let blockstore = blockstore
                .open()
                .await
                .map_err(|e| {
                    tracing::error!("Failed to open blockstore: {:?}", e);
                })
                .unwrap();
            if let Err(e) = tx.send(blockstore) {
                tracing::error!("Failed to send blockstore: {:?}", e);
            }
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

        let mut rdx_runner = RdxRunner::new(peerpiper, None);

        let mut builtins = crate::BUILTIN_PLUGINS.to_vec();

        // the wallet_plugin needs to be separate from the other plugins
        // because it's exports (get-mk, prove) become the imports for
        // the other plugins.
        // Getthe single `wallet_plugin` from `BUILTIN_PLUGINS`,
        // and put the remaining in `rest` array.
        let (wallet_name, wallet_bytes) = builtins
            .iter()
            .position(|(name, _)| *name == "wallet_plugin.wasm")
            .map(|i| builtins.remove(i))
            .unwrap();

        let arc_wallet = rdx_runner.load(wallet_name, wallet_bytes);

        // set self.arc_wallet to arc_wallet
        rdx_runner.arc_wallet = Some(arc_wallet);

        // now the rest of the plugins
        for (name, bytes) in builtins {
            let _ = rdx_runner.load(name, bytes);
        }

        Self {
            log,
            ctx,
            addr,
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

    /// Load a plugin with the given name and bytes
    pub(crate) fn load_plugin(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.button("Pick plugin file…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                // load the file bytes
                let bytes = std::fs::read(&path)
                    .unwrap_or_else(|err| panic!("Failed to read file: {}", err));

                self.rdx_runner.load(&file_name, &bytes);
                ctx.request_repaint();
            }
        }
    }
}
