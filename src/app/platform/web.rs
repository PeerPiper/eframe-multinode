//! Web platform speciic code.
//!
//! TODO.
//!
//! This module contains the web specific code for the platform.
//! Instead of spinning up a native node, this code would connect to a remote node
//! using peerpiper-browser.
pub mod piper;
mod settings;
mod storage;
mod web_error;
mod widget;

//pub use peerpiper_browser::opfs::OPFSBlockstore as Blockstore;
pub use piper::OPFSWrapped as Blockstore;
pub(crate) use settings::Settings;
pub use storage::StringStore;
pub use web_error::WebError as Error;

use crate::app::platform;
use crate::app::platform::piper::PeerPiper;
use crate::app::platform::web::piper::OPFSWrapped;
use crate::app::RdxRunner;
use chrono::TimeZone;
use multiaddr::Multiaddr;
use peerpiper::core::events::PublicEvent;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub fn spawn(f: impl Future<Output = ()> + 'static) {
    tracing::debug!("Spawning wasm_bingen future");
    wasm_bindgen_futures::spawn_local(f);
}

/// Reference counted [egui::Context] with a flag to indicate whether it has been set
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
    #[allow(dead_code)]
    pub(crate) fn request_repaint(&self) {
        if self.set {
            self.ctx.request_repaint();
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct Loader {
    /// Name of the plugin
    pub(crate) name: String,
    /// Plugin bytes
    pub(crate) bytes: Vec<u8>,
}

impl Loader {}

// allow unused code
#[allow(dead_code)]
pub struct Platform {
    /// Clone of the [egui::Context] so that the platform can trigger repaints
    ctx: Rc<RefCell<ContextSet>>,

    /// The node multiaddr to which we are connected
    node_multiaddr: String,

    /// Plugin bytes Loader
    pub loader: Arc<Mutex<Option<Loader>>>,

    /// RDX Runner
    pub rdx_runner: RdxRunner,

    /// PeerPiper gives us access to the netowrk, storage, and plugins
    pub peerpiper: Rc<RefCell<Option<PeerPiper>>>,
}

impl Default for Platform {
    fn default() -> Self {
        let arc_collection_plugins = Arc::new(Mutex::new(HashMap::new()));

        let peerpiper = Rc::new(RefCell::new(None));

        let (sender, receiver) = futures::channel::oneshot::channel::<PeerPiper>();

        platform::spawn(async move {
            let Ok(blockstore) = OPFSWrapped::new().await else {
                tracing::error!("Error creating OPFSWrapped instance");
                return;
            };
            let peerpiper = PeerPiper::new(blockstore, arc_collection_plugins.clone());

            // signal to the rdx_runner that the peerpiper is ready
            if let Err(_) = sender.send(peerpiper) {
                tracing::error!("Error sending ready signal to rdx_runner");
            }
        });

        let mut rdx_runner = RdxRunner::new(peerpiper.clone(), None, receiver);

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
            ctx: Rc::new(RefCell::new(ContextSet::new())),
            node_multiaddr: "/dnsaddr/peerpiper.io".to_string(),
            rdx_runner,
            loader: Default::default(),
            peerpiper,
        }
    }
}

#[derive(Default, Clone)]
struct ConnectState {
    response: Vec<String>,
    is_loading: bool,
    error: Option<String>,
    marker: std::marker::PhantomData<()>,
}
impl Platform {
    // pub fn close(&mut self) {}

    /// Address of the node. This will eventually be the relay address through
    /// a server node since this is the Browser side of things.
    pub fn addr(&self) -> Option<Multiaddr> {
        // TODO: Switch to relay address once connected to server node.
        Multiaddr::try_from(self.node_multiaddr.clone()).ok()
    }

    /// Returns whether the ctx is set or not
    pub fn egui_ctx(&self) -> bool {
        self.ctx.borrow().set
    }

    /// Sets the egui context
    pub fn set_egui_ctx(&mut self, ctx: egui::Context) {
        self.ctx.borrow_mut().ctx = ctx;
        self.ctx.borrow_mut().set = true;
    }

    /// Show the GUI for this platform
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Connect to the node_multiaddr, internally using fetch if it's a dnsaddr.
        let ready = {
            let piper_borrow = self.peerpiper.borrow();
            piper_borrow.is_some()
        };

        if ready {
            self.dial(ctx, ui);
        }
        //widget::fetch(ctx, ui, &mut self.node_multiaddr);

        // TODO: use peerpiper.connect(libp2p_endpoints, on_event) to connect to the network
    }

    /// Load a plugin with the given name and bytes
    pub(crate) fn load_plugin(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Check self.loader for the plugin details
        // If it's there, take the bytes out and load the plugin:
        if let Some(loader) = self.loader.lock().unwrap().take() {
            self.rdx_runner.load(&loader.name, &loader.bytes);
            ctx.request_repaint();
        }

        if ui.button("Pick plugin file…").clicked() {
            let task = rfd::AsyncFileDialog::new().pick_file();
            let loader_clone = Arc::clone(&self.loader);
            platform::spawn(async move {
                if let Some(file) = task.await {
                    let name = file.file_name();
                    let bytes = file.read().await;

                    let mut loader = loader_clone.lock().unwrap();
                    *loader = Some(Loader { name, bytes });
                }
            });
        }
    }
    /// Connect to node multiaddr and show the state.
    /// Similar to fetch above, uses some of the ame code and logic,
    /// but additionally calls peerpiper commander connect() to actually make the connection.
    pub fn dial(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        use crate::app::platform::web::widget::fetch_dns_query;

        let url = &mut self.node_multiaddr;

        // Generate an id for the state
        let state_id = ui.id().with("connection_state_libp2p");

        // Retrieve shared fetch state
        let mut connect_state =
            ctx.data_mut(|data| data.get_temp::<ConnectState>(state_id).unwrap_or_default());

        ui.label("Addr:");
        ui.add_sized([300.0, 20.0], egui::TextEdit::singleline(url));

        if ui.button("Dial").clicked() {
            // Update fetch state
            connect_state.response = Vec::with_capacity(4);
            connect_state.error = Default::default();
            connect_state.is_loading = true;

            let ctx_clone = ctx.clone();

            // show our loading spinner now
            ctx.data_mut(|data| {
                data.insert_temp(state_id, connect_state.clone());
            });

            let mut connect_state_clone = connect_state.clone();
            let pp_clone = self.peerpiper.clone();
            let (on_event, mut rx_evts) = tokio::sync::mpsc::channel(16);

            let maddr = self.addr();

            platform::spawn(async move {
                // if addr is a dnsaddr, fecth dns first to get the libp2p endpoints
                let libp2p_endpoints = if let Some(addr) = maddr {
                    if addr
                        .iter()
                        .any(|protocol| matches!(protocol, multiaddr::Protocol::Dns(_)))
                    {
                        fetch_dns_query(addr.to_string())
                            .await
                            .map_err(|e| {
                                connect_state_clone.error =
                                    Some(format!("Could not fetch endpoints. Error: {:?}", e));
                                connect_state_clone.is_loading = false;
                                connect_state_clone.response = vec!["Endpoints Error".to_string()];
                                ctx_clone.data_mut(|data| {
                                    data.insert_temp(state_id, connect_state_clone);
                                });
                            })
                            .unwrap_or_default()
                    } else {
                        vec![addr.to_string()]
                    }
                } else {
                    connect_state_clone.error = Some("Invalid multiaddress".to_string());
                    connect_state_clone.is_loading = false;
                    connect_state_clone.response = vec!["Invalid multiaddress".to_string()];
                    ctx_clone.data_mut(|data| {
                        data.insert_temp(state_id, connect_state_clone);
                    });
                    return;
                };

                let listen = {
                    match pp_clone
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        .connect(libp2p_endpoints)
                        .await
                    {
                        Ok(listen) => listen,
                        Err(e) => {
                            tracing::error!("Failed to connect to the network: {:?}", e);
                            return;
                        }
                    }
                };

                listen(on_event);
            });

            let mut connect_state_clone = connect_state.clone();
            let ctx_clone = ctx.clone();
            platform::spawn(async move {
                let timeout = gloo_timers::future::TimeoutFuture::new(10000);
                tokio::pin!(timeout);

                let connect_state = connect_state_clone.clone();
                tokio::select! {
                    _ = &mut timeout => {
                        connect_state_clone.error = Some("Connection timed out".to_string());
                        connect_state_clone.is_loading = false;
                        connect_state_clone.response = vec!["Connect Timeout".to_string()];
                        ctx_clone.data_mut(|data| {
                            data.insert_temp(state_id, connect_state_clone);
                        });
                        return;
                    }
                    // Wait for 5 seconds for a Events::Outer(PublicEvent::NewConnection { peer }) event
                    // await rx_evts
                    event = rx_evts.recv() => {
                        if let Some(PublicEvent::NewConnection { peer }) = event {

                            connect_state_clone.response = vec![format!("Connected to {}", peer)];
                            connect_state_clone.is_loading = false;
                            ctx_clone.data_mut(|data| {
                                data.insert_temp(state_id, connect_state_clone);
                            });
                            ctx_clone.request_repaint();
                        }
                    }
                }

                while let Some(event) = rx_evts.recv().await {
                    let mut connect_state_clone = ctx_clone.data_mut(|data| {
                        data.get_temp::<ConnectState>(state_id)
                            .unwrap_or(connect_state.clone())
                    });

                    let unix_timestamp = rdx::layer::SystemTime::now()
                        .duration_since(rdx::layer::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    let datetime: chrono::DateTime<chrono::Utc> =
                        chrono::Utc.timestamp_opt(unix_timestamp, 0).unwrap();

                    // format into date, hours,mins, seconds
                    let formatted_date = datetime.format("%Y-%m-%d %H:%M:%S");

                    // TODO: Wire up these events to the plugins
                    tracing::debug!("{formatted_date} Received event: {:?}", &event);
                    // put the event on to the front of .response
                    connect_state_clone
                        .response
                        .push(format!("{} {:?}", formatted_date, event));

                    ctx_clone.data_mut(|data| {
                        data.insert_temp(state_id, connect_state_clone);
                    });
                }
            });
        }

        // Loading indicator
        if connect_state.is_loading {
            ui.spinner();
        }

        // Error display
        if let Some(error) = &connect_state.error {
            ui.colored_label(egui::Color32::RED, error);
        }

        // Response display
        egui::ScrollArea::vertical().show(ui, |ui| {
            //ui.push_id("connect_response");
            for line in connect_state.response.into_iter().rev() {
                // print out len of response
                ui.label(line);
            }
        });
    }
}
