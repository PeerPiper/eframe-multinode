//! Web platform speciic code.
//!
//! TODO.
//!
//! This module contains the web specific code for the platform.
//! Instead of spinning up a native node, this code would connect to a remote node
//! using peerpiper-browser.
mod widget;

use std::cell::RefCell;
use std::rc::Rc;

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
pub struct Platform {
    /// The Context
    ctx: Rc<RefCell<ContextSet>>,

    /// The node multiaddr to which we are connected
    node_multiaddr: String,
}

impl Default for Platform {
    fn default() -> Self {
        Self {
            ctx: Rc::new(RefCell::new(ContextSet::new())),
            node_multiaddr: "/dnsaddr/peerpiper.io".to_string(),
        }
    }
}

impl Platform {
    // pub fn close(&mut self) {}

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
    }

    /// Loads the plugin (TODO)
    pub fn load_plugin(&self, _name: String, _bytes: Vec<u8>) {
        // TODO
    }
}
