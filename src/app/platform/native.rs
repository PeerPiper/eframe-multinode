//! Platform code, specific to the native platform.
//!
//! For example, a native node will only be available here. Whereas the browser needs to connect
//! to a remote node, which is handled in the `web` module.
mod cloudflare;
mod settings;

use multiaddr::Multiaddr;
pub(crate) use settings::Settings;

use peerpiper_plugins::tokio::{ExternalEvents, PluggablePiper, PluginLoader};
use std::sync::{Arc, Mutex};

// use peerpiper_plugins::{PluggablePiper};

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
pub(crate) struct Platform {
    log: Arc<Mutex<Vec<String>>>,

    /// Clone of the [egui::Context] so that the platform can trigger repaints
    ctx: Arc<Mutex<ContextSet>>,

    loader: PluginLoader,

    addr: Arc<Mutex<Option<Multiaddr>>>,
}

impl Default for Platform {
    fn default() -> Self {
        let log = Arc::new(Mutex::new(Vec::new()));
        let ctx: Arc<Mutex<ContextSet>> = Arc::new(Mutex::new(ContextSet::new()));
        let addr = Arc::new(Mutex::new(None));

        let (mut pluggable, command_receiver, loader, mut plugin_evts) = PluggablePiper::new();

        let log_clone = log.clone();
        let ctx_clone = ctx.clone();
        let addr_clone = addr.clone();

        // task for listening on plugin events and updating the log accoringly
        tokio::task::spawn(async move {
            while let Some(event) = plugin_evts.recv().await {
                let msg = format!("{:?}", event);
                tracing::debug!("Received event: {:?}", msg);
                log_clone.lock().unwrap().push(msg);
                ctx_clone.lock().unwrap().request_repaint();

                if let ExternalEvents::Address(addr) = event {
                    tracing::debug!("Received Node Address: {:?}", addr);
                    let mut lock = addr_clone.lock().unwrap();
                    *lock = Some(addr);
                }
            }
        });

        // Execute the runtime in its own thread.
        tokio::task::spawn(async move {
            pluggable.run(command_receiver).await.unwrap_or_else(|e| {
                tracing::error!("Failed to run PluggablePiper: {:?}", e);
            });
        });

        Self {
            log,
            ctx,
            loader,
            addr,
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
    /// Load a plugin into the Platform
    pub fn load_plugin(&self, name: String, wasm: Vec<u8>) {
        // call self.loader.load_plugin(name, wasm).await from this sync function using tokio
        let mut loader = self.loader.clone();
        tokio::task::spawn(async move {
            if let Err(e) = loader.load_plugin(name, &wasm).await {
                tracing::error!("Failed to load plugin: {:?}", e);
            }
        });
    }

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
