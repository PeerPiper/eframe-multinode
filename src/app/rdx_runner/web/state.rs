//! Custom State for the RDX Plugins, Implements [rdx::layer::Inner] and custom Serialize/Deserialize
//! so that the [rdx::layer::rhai::Scope] can be serialized and deserialized.
use crate::app::platform;
use crate::app::platform::StringStore;
use crate::app::rdx_runner::PeerPiperWired;
use gloo_timers::callback::Timeout;
use peerpiper::core::events::AllCommands;
use peerpiper::core::events::SystemCommand;
use peerpiper::core::Cid;
use peerpiper::core::ReturnValues;
use rdx::layer::{
    rhai::{Dynamic, Scope},
    Inner, ScopeRef, ScopeRefMut,
};
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::rc::Rc;

/// Serializable [State] struct that holds the [Scope] and [egui::Context]
#[derive(Clone)]
//#[serde(default)]
pub struct State {
    inner: SendWrapper<InnerState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            inner: SendWrapper::new(InnerState::default()),
        }
    }
}

#[derive(Clone, Default)]
struct InnerState {
    /// The unique name of this plugin, typically the filename
    name: String,
    /// The reference counted [Scope] that holds the state of the plugin.
    ///
    /// Scope needs to be Rc<RefCell<Scope>> because we need to be able to clone the state
    /// and pass it into an async task in order to be initialized.
    scope: Rc<RefCell<Scope<'static>>>,
    /// The [egui::Context] that holds the UI state. Used to request repaints
    egui_ctx: Option<egui::Context>,
    /// Handler to PeerPiper SystemCommander
    peerpiper: PeerPiperWired,
    /// String Store map the name of the plugin to the CID of the state
    cid_map: StringStore,
    /// Debouncing mechanism to save the state but workaround many requests at once
    timer: Rc<RefCell<Option<Timeout>>>,
}

//pub fn sleep(dur: web_time::Duration) -> impl futures::Future<Output = ()> {
//    gloo_timers::future::TimeoutFuture::new(dur.as_millis() as u32)
//}

impl State {
    pub fn new(
        name: impl AsRef<str> + Clone,
        ctx: Option<egui::Context>,
        peerpiper: PeerPiperWired,
    ) -> Self {
        let scope = Rc::new(RefCell::new(Scope::new()));

        let cid_map = StringStore::new();

        Self {
            inner: SendWrapper::new(InnerState {
                scope,
                egui_ctx: ctx,
                name: name.clone().as_ref().to_string(),
                peerpiper,
                cid_map,
                timer: Rc::new(RefCell::new(None)),
            }),
        }
    }

    /// Initialize the state with a scope from storage, if it exists.
    /// Same steps as in new(), but using self.* instead.
    pub async fn init(&self) {
        tracing::info!("State::init() called for {:?}", self.inner.name);
        if let Some(key) = self.inner.cid_map.get_string(&self.inner.name) {
            if let Ok(cid) = Cid::try_from(key.clone()) {
                if self.inner.peerpiper.borrow().is_none() {
                    tracing::warn!("Failed to get PeerPiper from State");
                    return;
                };

                let name: String = self.inner.name.clone();
                let pp = {
                    tracing::info!("Borrow mut in init");
                    let command = AllCommands::System(SystemCommand::Get { key: cid.into() });
                    self.inner
                        .peerpiper
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .order(command)
                        .await
                };

                let Ok(ReturnValues::Data(bytes)) = pp else {
                    tracing::warn!("Failed to get state from CID: {}", key);
                    return;
                };

                let str = String::from_utf8_lossy(&bytes);

                let Ok(scope) = serde_json::from_str(&str) else {
                    tracing::warn!("Failed to decode state scope from CID: {}", key);
                    return;
                };

                // set the plugin scope to the loaded scope,
                // this is how we load the state from disk into the plugin
                *self.inner.scope.borrow_mut() = scope;
                tracing::info!("*** State loaded for {:?} ", name,);
            } else {
                tracing::warn!("Failed to parse CID from string: {}", key);
            }
        } else {
            tracing::warn!("No state found for {:?}", self.inner.name);
        }
    }

    /// Persist the [rhai::Scope] state on disk
    ///
    /// Should work in both browser and native environments
    pub async fn async_save(&self) -> anyhow::Result<String> {
        // save the state scope to disk.
        // Use put/get and get CID values, which you then map to plugin names and save THAT too.
        // Advantage of Option B is we can content address share plugin state scope.
        // Disadvantage is that we need to keep a mapping of plugin names to CIDs (a-la IPNS) when
        // data changes.
        let str = serde_json::to_string_pretty(&self.inner.scope.borrow().clone())?;
        let bytes = str.as_bytes().to_vec();

        // Save the serialized state to disk, independent of the platform
        // for this we can use peerpiper SystemCommandHanlder to put the bytes into the local system
        let binding = self.inner.peerpiper.borrow();
        let Some(pipr) = binding.as_ref() else {
            tracing::warn!("Save: PeerPiper is not ready yet");
            return Err(anyhow::anyhow!(
                "Anyhow Save: PeerPiper Commander is not set yet"
            ))?;
        };

        let command = AllCommands::System(SystemCommand::Put { bytes });
        let Ok(ReturnValues::ID(cid)) = pipr.order(command).await else {
            return Err(anyhow::anyhow!("Failed to order command: Put"))?;
        };

        drop(binding);

        // Save name:cid mapping to platform storage
        // filesystem, localstorage, etc.
        self.inner
            .cid_map
            .set_string(&self.inner.name, cid.to_string());

        Ok(cid.to_string())
    }
}

impl Inner for State {
    /// Saves the state to disk
    fn save(&self) {
        tracing::info!("[web.save] Saving state for {:?}", self.inner.name);
        // Debounce the save operation by pushing the save time into the save_after field
        // and then spawning a task to save the state after the debounce time
        let self_clone = self.clone(); // our save() function lives here
        let timer = self.inner.timer.clone();
        let delay = 750u32; // ms

        if let Some(t) = timer.borrow_mut().take() {
            t.cancel();
        }

        let new_timer = Timeout::new(delay, || {
            platform::spawn(async move {
                tracing::info!("Saving state after gloo_timer");
                let cid = self_clone.async_save().await;
                tracing::info!("State saved to CID: {:?}", cid);
            });
        });

        *timer.borrow_mut() = Some(new_timer);
    }

    /// Updates the scope variable to the given value
    fn update(&mut self, key: &str, value: impl Into<Dynamic> + Clone) {
        tracing::info!("[web.update] Updating scope with key: {}", key);
        self.inner.scope.borrow_mut().set_value(key, value.into());

        if let Some(egui_ctx) = &self.inner.egui_ctx {
            tracing::info!("Requesting repaint");
            egui_ctx.request_repaint();
            // now that the rhai scope has been updated, we need to re-run
        } else {
            tracing::warn!("Egui context is not set");
        }

        self.save();
    }

    fn scope(&self) -> ScopeRef {
        ScopeRef::Refcell(self.inner.scope.clone())
    }

    fn scope_mut(&mut self) -> ScopeRefMut {
        ScopeRefMut::Refcell(self.inner.scope.borrow_mut())
    }

    // into_scope with 'static lifetime
    fn into_scope(self) -> Scope<'static> {
        // return a copy of the scope
        self.inner.scope.borrow().clone() // creates a completely new copy of the inner data
    }
}

// Example usage and tests
#[cfg(test)]
mod tests {}
