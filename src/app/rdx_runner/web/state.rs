//! Custom State for the RDX Plugins, Implements [rdx::layer::Inner] and custom Serialize/Deserialize
//! so that the [rdx::layer::rhai::Scope] can be serialized and deserialized.
use crate::app::platform;
use crate::app::platform::StringStore;
use crate::app::rdx_runner::CommanderCounter;
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
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Default)]
struct InnerState {
    /// The unique name of this plugin, typically the filename
    name: String,
    /// The [Scope] that holds the state of the plugin
    scope: Rc<RefCell<Scope<'static>>>,
    /// The [egui::Context] that holds the UI state. Used to request repaints
    egui_ctx: Option<egui::Context>,
    /// Handler to PeerPiper SystemCommander
    commander: CommanderCounter,
    /// String Store map the name of the plugin to the CID of the state
    cid_map: StringStore,
    /// Debouncing mechanism to save the state but workaround many requests at once
    timer: Rc<RefCell<Option<Timeout>>>,
}

//pub fn sleep(dur: Duration) -> impl Future<Output = ()> {
//    gloo_timers::future::TimeoutFuture::new(dur.as_millis() as u32)
//}

impl State {
    pub fn new(
        name: impl AsRef<str> + Clone,
        ctx: Option<egui::Context>,
        commander: CommanderCounter,
    ) -> Self {
        let scope = Rc::new(RefCell::new(Scope::new()));

        let cid_map = StringStore::new();

        Self {
            inner: SendWrapper::new(InnerState {
                scope,
                egui_ctx: ctx,
                name: name.clone().as_ref().to_string(),
                commander,
                cid_map,
                timer: Rc::new(RefCell::new(None)),
            }),
        }
    }

    /// Initialize the state with a scope from storage, if it exists.
    /// Same steps as in new(), but using self.* instead.
    pub fn init(&self) {
        log::debug!("State::init() called");
        let scope_clone = self.inner.scope.clone();
        if let Some(key) = self.inner.cid_map.get_string(&self.inner.name) {
            if let Ok(cid) = Cid::try_from(key.clone()) {
                let commander_clone = self.inner.commander.clone();

                platform::spawn(async move {
                    let pp = {
                        log::info!("Borrow mut in init");
                        let binding = commander_clone.borrow();
                        let Some(ref mut commander) = binding.as_ref() else {
                            log::warn!("INIT: Commander is not set yet");
                            return;
                        };
                        let command = AllCommands::System(SystemCommand::Get { key: cid.into() });
                        commander.order(command).await
                    };

                    let Ok(ReturnValues::Data(bytes)) = pp else {
                        log::warn!("Failed to get state from CID: {}", key);
                        return;
                    };

                    let str = String::from_utf8_lossy(&bytes);

                    let Ok(scope) = serde_json::from_str(&str) else {
                        tracing::warn!("Failed to decode state scope from CID: {}", key);
                        return;
                    };

                    *scope_clone.borrow_mut() = scope;
                    log::info!("*** State loaded from commander: {:?}", scope_clone);
                });
            }
        }
    }

    /// Persist the [rhai::Scope] state on disk
    ///
    /// Should work in both browser and native environments
    pub async fn save(&self) -> anyhow::Result<String> {
        // save the state scope to disk.
        // Use put/get and get CID values, which you then map to plugin names and save THAT too.
        // Advantage of Option B is we can content address share plugin state scope.
        // Disadvantage is that we need to keep a mapping of plugin names to CIDs (a-la IPNS) when
        // data changes.
        let str = serde_json::to_string_pretty(&self.inner.scope.borrow().clone())?;
        let bytes = str.as_bytes().to_vec();

        // Save the serialized state to disk, independent of the platform
        // for this we can use peerpiper SystemCommandHanlder to put the bytes into the local system
        log::info!("borrow_mut in state.save()");
        let binding = self.inner.commander.borrow();
        let Some(cmdr) = binding.as_ref() else {
            log::warn!("Save: Commander is not set yet");
            return Err(anyhow::anyhow!("Anyhow Save: Commander is not set yet"))?;
        };

        let command = AllCommands::System(SystemCommand::Put { bytes });
        let Ok(ReturnValues::ID(cid)) = cmdr.order(command).await else {
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
    /// Updates the scope variable to the given value
    fn update(&mut self, key: &str, value: impl Into<Dynamic> + Copy) {
        self.inner.scope.borrow_mut().set_or_push(key, value.into());

        if let Some(egui_ctx) = &self.inner.egui_ctx {
            log::info!("Requesting repaint");
            egui_ctx.request_repaint();
            // now that the rhai scope has been updated, we need to re-run
        } else {
            log::warn!("Egui context is not set");
        }

        // Debounce the save operation by pushing the save time into the save_after field
        // and then spawning a task to save the state after the debounce time
        let self_clone = self.clone(); // our save() function lives here
        let timer = self.inner.timer.clone();
        let delay = 5u32;

        if let Some(t) = timer.borrow_mut().take() {
            t.cancel();
        }

        let new_timer = Timeout::new(delay, || {
            platform::spawn(async move {
                log::info!("Saving state after gloo_timer");
                let cid = self_clone.save().await;
                log::info!("State saved to CID: {:?}", cid);
            });
        });

        *timer.borrow_mut() = Some(new_timer);
    }

    fn scope(&self) -> ScopeRef {
        ScopeRef::Refcell(self.inner.scope.borrow())
    }

    fn scope_mut(&mut self) -> ScopeRefMut {
        ScopeRefMut::Refcell(self.inner.scope.borrow_mut())
    }

    // into_scope with 'static lifetime'
    fn into_scope(self) -> Scope<'static> {
        // return a copy of the scope
        self.inner.scope.borrow().clone()
    }
}

// Example usage and tests
#[cfg(test)]
mod tests {}
