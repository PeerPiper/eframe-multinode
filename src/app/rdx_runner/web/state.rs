//! Custom State for the RDX Plugins, Implements [rdx::layer::Inner] and custom Serialize/Deserialize
//! so that the [rdx::layer::rhai::Scope] can be serialized and deserialized.
use crate::app::platform;
use crate::app::platform::peerpiper::PeerPiper;
use crate::app::platform::StringStore;
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
    peerpiper: Rc<RefCell<Option<PeerPiper>>>,
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
        peerpiper: Rc<RefCell<Option<PeerPiper>>>,
    ) -> Self {
        let scope = Rc::new(RefCell::new(Scope::new()));

        let cid_map = StringStore::new();

        // Load the CID of the state from the platform storage
        // filesystem, localstorage, etc.
        // and load the state from the CID
        // and set the scope to the loaded state
        // if the CID is not found, create a new scope
        // and set the scope to the new scope
        //
        // Technically it's possible that the PeerPiper is not set yet, so we need to handle THAT
        // case as well, but it also seems to always be ready by the time we get here.
        //let scope_clone = scope.clone();
        //if let Some(key) = cid_map.get_string(name.as_ref()) {
        //    if let Ok(cid) = Cid::try_from(key.clone()) {
        //        let peerpiper_clone = peerpiper.clone();
        //
        //        platform::spawn(async move {
        //            let pp = {
        //                log::info!("Borrow mut in new");
        //                let mut binding = peerpiper_clone.borrow_mut();
        //                let Some(ref mut peerpiper) = binding.as_mut() else {
        //                    log::warn!("INIT: Commander is not set yet");
        //                    return;
        //                };
        //                let command = AllCommands::System(SystemCommand::Get { key: cid.into() });
        //                peerpiper.order(command).await
        //            };
        //
        //            let Ok(ReturnValues::Data(bytes)) = pp else {
        //                log::warn!("Failed to get state from CID: {}", key);
        //                return;
        //            };
        //
        //            let Ok(scope): Result<Scope, cbor4ii::serde::DecodeError<_>> =
        //                cbor4ii::serde::from_slice(&bytes)
        //            else {
        //                log::warn!("Failed to decode state scope from CID: {}", key);
        //                return;
        //            };
        //
        //            *scope_clone.borrow_mut() = scope;
        //            log::info!("*** State loaded from PeerPiper: {:?}", scope_clone);
        //        });
        //    }
        //}

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
    pub fn init(&self) {
        log::debug!("State::init() called");
        let scope_clone = self.inner.scope.clone();
        if let Some(key) = self.inner.cid_map.get_string(&self.inner.name) {
            if let Ok(cid) = Cid::try_from(key.clone()) {
                let peerpiper_clone = self.inner.peerpiper.clone();

                platform::spawn(async move {
                    let pp = {
                        log::info!("Borrow mut in init");
                        let binding = peerpiper_clone.borrow();
                        let Some(ref mut peerpiper) = binding.as_ref() else {
                            log::warn!("INIT: Commander is not set yet");
                            return;
                        };
                        let command = AllCommands::System(SystemCommand::Get { key: cid.into() });
                        peerpiper.order(command).await
                    };

                    let Ok(ReturnValues::Data(bytes)) = pp else {
                        log::warn!("Failed to get state from CID: {}", key);
                        return;
                    };

                    let Ok(scope): Result<Scope, cbor4ii::serde::DecodeError<_>> =
                        cbor4ii::serde::from_slice(&bytes)
                    else {
                        log::warn!("Failed to decode state scope from CID: {}", key);
                        return;
                    };

                    *scope_clone.borrow_mut() = scope;
                    log::info!("*** State loaded from PeerPiper: {:?}", scope_clone);
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
        let bytes = cbor4ii::serde::to_vec(Vec::new(), &self.inner.scope)?;
        // Save the serialized state to disk, independent of the platform
        // for this we can use peerpiper SystemCommandHanlder to put the bytes into the local system
        log::info!("borrow_mut in state.save()");
        let binding = self.inner.peerpiper.borrow();
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
mod tests {

    use super::*;

    #[test]
    fn test_state_scope_serialization() {
        // Create a State with a Scope
        let mut state = State::default();

        // Add some values to the scope
        let x = 42i64;
        state.scope.set_value("x", x);
        state.scope.push_constant("name", "John");
        state.scope.set_value("is_active", true);

        // Serialize to JSON
        let serialized = cbor4ii::serde::to_vec(Vec::new(), &state.scope).unwrap();
        println!("Serialized: {:?}", serialized);

        // Deserialize back to State
        let deserialized_scope: Scope<'_> = cbor4ii::serde::from_slice(&serialized).unwrap();
        eprintln!("Deserialized: {:#?}", deserialized_scope);

        // Verify scope values
        assert_eq!(deserialized_scope.get_value::<i64>("x").unwrap(), x);

        assert_eq!(
            deserialized_scope.get_value::<String>("name").unwrap(),
            "John"
        );

        assert!(deserialized_scope.get_value::<bool>("is_active").unwrap());
    }
}
