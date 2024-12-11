//! Web platform speciic code.
//!
//! TODO.
//!
//! This module contains the web specific code for the platform.
//! Instead of spinning up a native node, this code would connect to a remote node
//! using peerpiper-browser.
pub mod peerpiper;
mod settings;
mod storage;
mod web_error;
mod widget;

//pub use peerpiper_browser::opfs::OPFSBlockstore as Blockstore;
pub use peerpiper::OPFSWrapped as Blockstore;
pub(crate) use settings::Settings;
pub use storage::StringStore;
pub use web_error::WebError as Error;

use crate::app::platform;
use crate::app::platform::peerpiper::PeerPiper;
use multiaddr::Multiaddr;
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

#[derive(Clone)]
// allow unused code
#[allow(dead_code)]
pub struct Platform {
    /// The Context
    ctx: Rc<RefCell<ContextSet>>,
    /// The node multiaddr to which we are connected
    node_multiaddr: String,
    /// PeerPiper instance. It is generated from a spawned task.
    peerpiper: Rc<RefCell<Option<PeerPiper>>>,
}

impl Default for Platform {
    fn default() -> Self {
        let peerpiper = Rc::new(RefCell::new(None));
        let peerpiper_clone = peerpiper.clone();
        // new PeerPiper with built-in Commander and BrowserBlockStore
        platform::spawn(async move {
            let Ok(piper) = peerpiper::create_peerpiper().await else {
                log::error!("Error creating PeerPiper BrowserBlockStore instance");
                return;
            };
            peerpiper_clone.borrow_mut().replace(piper);
        });

        Self {
            ctx: Rc::new(RefCell::new(ContextSet::new())),
            node_multiaddr: "/dnsaddr/peerpiper.io".to_string(),
            peerpiper,
        }
    }
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
        widget::fetch(ctx, ui, &mut self.node_multiaddr);

        // TODO: use peerpiper.connect(libp2p_endpoints, on_event) to connect to the network
    }

    /// Loads the plugin (TODO)
    pub fn load_plugin(&self, _name: String, _bytes: Vec<u8>) {
        // TODO
    }

    ///// Pass along PeerPiper comamnds to the PeerPiper instance
    //pub fn command(&self, command: commander::PeerPiperCommand) {
    //    if let Some(piper) = self.peerpiper.borrow_mut().as_ref() {
    //        piper.order(command);
    //    }
    //}
}
