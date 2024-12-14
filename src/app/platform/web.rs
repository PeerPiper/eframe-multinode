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
use chrono::TimeZone;
use futures::{
    channel::{
        mpsc::{self},
        oneshot,
    },
    StreamExt,
};
//use crate::app::platform::piper::AllCommands;
use crate::app::platform::web::piper::OPFSWrapped;
use crate::app::RdxRunner;
use multiaddr::Multiaddr;
use peerpiper::core::events::Events;
use peerpiper::core::Commander;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

pub fn spawn(f: impl Future<Output = ()> + 'static) {
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
pub(crate) struct Loader;

impl Loader {
    /// Load a plugin into the Platform
    pub fn load_plugin(&self, _name: String, _wasm: Vec<u8>) {
        // TODO: Web plugins
    }
}

// allow unused code
#[allow(dead_code)]
pub struct Platform {
    /// Clone of the [egui::Context] so that the platform can trigger repaints
    ctx: Rc<RefCell<ContextSet>>,

    /// The node multiaddr to which we are connected
    node_multiaddr: String,

    /// PeerPiper commander instance. It is generated from a spawned task.
    commander: Rc<RefCell<Option<Commander<Blockstore>>>>,

    /// Plugin Loader
    pub loader: Loader,

    /// RDX Runner
    pub rdx_runner: RdxRunner,
}

impl Default for Platform {
    fn default() -> Self {
        let commander = Rc::new(RefCell::new(None));
        let commander_clone = commander.clone();
        // new PeerPiper with built-in Commander and BrowserBlockStore

        let (sender, receiver) = futures::channel::oneshot::channel::<()>();

        platform::spawn(async move {
            let Ok(handler) = OPFSWrapped::new().await else {
                log::error!("Error creating OPFSWrapped instance");
                return;
            };
            let c = Commander::new(handler);
            commander_clone.borrow_mut().replace(c);

            // signal to the rdx_runner that the commander is ready
            if let Err(e) = sender.send(()) {
                log::error!("Error sending ready signal to rdx_runner: {:?}", e);
            }
        });

        let rdx_runner = RdxRunner::new(commander.clone(), None, receiver);

        Self {
            ctx: Rc::new(RefCell::new(ContextSet::new())),
            node_multiaddr: "/dnsaddr/peerpiper.io".to_string(),
            rdx_runner,
            loader: Loader,
            commander,
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

        self.dial(ctx, ui);
        //widget::fetch(ctx, ui, &mut self.node_multiaddr);

        // TODO: use peerpiper.connect(libp2p_endpoints, on_event) to connect to the network
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

            // Clone URL for async operation
            let url = url.clone();
            let ctx_clone = ctx.clone();

            let mut connect_state_clone = connect_state.clone();

            let cdr_clone = self.commander.clone();

            platform::spawn(async move {
                // Fetch data
                match fetch_dns_query(url).await {
                    Ok(libp2p_endpoints) => {
                        match connect(cdr_clone, libp2p_endpoints).await {
                            Ok(mut rx_evts) => {
                                // Wait for 5 seconds for a Events::Outer(PublicEvent::NewConnection { peer }) event,
                                // then start processing events. Otherwise, dials have failed.
                                // Select on either 5 seconds passing or if let Some(Events::Outer(PublicEvent::NewConnection { peer })) = rx_evts.next().await

                                let timeout = gloo_timers::future::TimeoutFuture::new(5000);
                                tokio::pin!(timeout);

                                let mut connect_state = connect_state_clone.clone();

                                tokio::select! {
                                    _ = &mut timeout => {
                                        connect_state.error = Some("Connection timed out".to_string());
                                        connect_state.is_loading = false;
                                        connect_state.response = vec!["Connect Timeout".to_string()];
                                        ctx_clone.data_mut(|data| {
                                            data.insert_temp(state_id, connect_state);
                                        });
                                        return;
                                    }
                                    // Wait for 5 seconds for a Events::Outer(PublicEvent::NewConnection { peer }) event
                                    event = rx_evts.next() => {
                                        if let Some(Events::Outer(peerpiper::core::events::PublicEvent::NewConnection { peer })) = event {

                                            connect_state.response = vec![format!("Connected to {}", peer)];
                                            connect_state.is_loading = false;
                                            ctx_clone.data_mut(|data| {
                                                data.insert_temp(state_id, connect_state);
                                            });
                                            ctx_clone.request_repaint();
                                        }
                                    }
                                }

                                let connect_state = connect_state_clone.clone();
                                while let Some(event) = rx_evts.next().await {
                                    let mut connect_state_clone = connect_state.clone();

                                    let unix_timestamp = rdx::layer::SystemTime::now()
                                        .duration_since(rdx::layer::SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs()
                                        as i64;

                                    let datetime: chrono::DateTime<chrono::Utc> =
                                        chrono::Utc.timestamp_opt(unix_timestamp, 0).unwrap();

                                    // format into date, hours,mins, seconds
                                    let formatted_date = datetime.format("%Y-%m-%d %H:%M:%S");

                                    // TODO: Wire up these events to the plugins
                                    log::debug!("{formatted_date} Received event: {:?}", &event);
                                    // put the event on to the front of .response
                                    connect_state_clone
                                        .response
                                        .push(format!("{} {:?}", formatted_date, event));

                                    log::debug!("Responses: {:?}", &connect_state_clone.response);

                                    ctx_clone.data_mut(|data| {
                                        data.insert_temp(state_id, connect_state_clone);
                                    });
                                }
                            }
                            Err(e) => {
                                connect_state_clone.error =
                                    Some(format!("Could not connect. Error: {:?}", e));
                                connect_state_clone.is_loading = false;
                                connect_state_clone.response = vec!["Connect Error".to_string()];
                                ctx_clone.data_mut(|data| {
                                    data.insert_temp(state_id, connect_state_clone);
                                });
                                return;
                            }
                        };
                    }
                    Err(e) => {
                        connect_state_clone.error =
                            Some(format!("Could not fetch endpoints. Error: {:?}", e));
                        connect_state_clone.is_loading = false;
                        connect_state_clone.response = vec!["Endpoints Error".to_string()];
                        ctx_clone.data_mut(|data| {
                            data.insert_temp(state_id, connect_state_clone);
                        });
                    }
                };
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
            for line in &connect_state.response {
                // print out len of response
                ui.label(format!("Responses: {}", &connect_state.response.len()));
                ui.label(line);
            }
        });
    }
    ///// Pass Platform commands along as PeerPiper comamnds to the PeerPiper commander instance
    //pub async fn command(&self, command: AllCommands) -> Result<ReturnValue, Error> {
    //    let Some(piper) = self.commander.borrow().as_ref() else {
    //        return Err(Error::CommanderNotReady);
    //    };
    //
    //    Ok(piper.order(command).await?)
    //}
}

/// Try to connect to the list of endpoints.
/// Send the `on_event` callback to the Commander to be called when an event is received.
pub async fn connect(
    commander: Rc<RefCell<Option<Commander<Blockstore>>>>,
    libp2p_endpoints: Vec<String>,
) -> Result<mpsc::Receiver<Events>, Error> {
    // 16 is arbitrary, but should be enough for now
    let (tx_evts, rx_evts) = mpsc::channel(16);

    // client sync oneshot
    let (tx_client, rx_client) = oneshot::channel();

    // command_sender will be used by other wasm_bindgen functions to send commands to the network
    // so we will need to wrap it in a Mutex or something to make it thread safe.
    let (network_command_sender, network_command_receiver) = tokio::sync::mpsc::channel(8);

    platform::spawn(async move {
        peerpiper::start(
            tx_evts,
            network_command_receiver,
            tx_client,
            libp2p_endpoints,
        )
        .await
        .expect("never end")
    });

    // wait on rx_client to get the client handle
    let client_handle = rx_client.await?;

    commander
        .borrow_mut()
        .as_mut()
        .unwrap()
        .with_network(network_command_sender)
        .with_client(client_handle);

    //while let Some(event) = rx_evts.next().await {
    //    if let peerpiper::core::events::Events::Outer(event) = event {
    //        log::debug!("[Browser] Received event: {:?}", &event);
    //        on_event.send(event).await?;
    //    }
    //}

    Ok(rx_evts)
}
