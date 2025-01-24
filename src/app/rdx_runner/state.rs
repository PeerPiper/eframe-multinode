//! Custom State for the RDX Plugins, Implements [rdx::layer::Inner] and custom Serialize/Deserialize
//! so that the [rdx::layer::rhai::Scope] can be serialized and deserialized.
use crate::app::platform;
use crate::app::platform::piper::PeerPiper;
use crate::app::platform::StringStore;
use peerpiper::core::events::AllCommands;
use peerpiper::core::events::SystemCommand;
use peerpiper::core::Cid;
use peerpiper::core::ReturnValues;
use rdx::layer::ScopeRef;
//use rdx::layer::ScopeRef;
use rdx::layer::ScopeRefMut;
use rdx::layer::{
    rhai::{Dynamic, Scope},
    Inner,
};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use tokio::sync::Mutex as AsyncMutex;

use super::debouncer::Debouncer;

/// Serializable [State] struct that holds the [Scope] and [egui::Context]
#[derive(Clone)]
//#[serde(default)]
pub struct State {
    /// The unique name of this plugin, typically the filename
    name: String,
    /// The [Scope] that holds the state of the plugin.
    ///
    /// It's Arc Mutex so we can update it as a result of an async task.
    scope: Arc<Mutex<Scope<'static>>>,
    /// The [egui::Context] that holds the UI state. Used to request repaints
    egui_ctx: Option<egui::Context>,
    /// Handler to PeerPiper SystemCommander
    peerpiper: Arc<AsyncMutex<PeerPiper>>,
    /// String Store map the name of the plugin to the CID of the state
    cid_map: StringStore,
    /// Canceller for deboucning saving state
    cancel_save: Arc<AsyncMutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl State {
    pub fn new(
        name: impl AsRef<str> + Clone,
        ctx: Option<egui::Context>,
        peerpiper: Arc<AsyncMutex<PeerPiper>>,
    ) -> Self {
        let mut scope = Scope::new();

        let cid_map = StringStore::new();

        // Load the CID of the state from the platform storage
        // filesystem, localstorage, etc.
        // and load the state from the CID
        // and set the scope to the loaded state
        // if the CID is not found, create a new scope
        // and set the scope to the new scope
        if let Some(key) = cid_map.get_string(name.as_ref()) {
            if let Ok(cid) = Cid::try_from(key.clone()) {
                let (tx, rx) = std::sync::mpsc::channel();

                let piper_clone = peerpiper.clone();

                platform::spawn(async move {
                    let command = AllCommands::System(SystemCommand::Get { key: cid.into() });
                    let pp = { piper_clone.lock().await.commander.order(command).await };

                    let Ok(ReturnValues::Data(bytes)) = pp else {
                        tracing::warn!("Failed to get state from CID: {}", key);
                        tx.send(None).unwrap();
                        return;
                    };

                    // bytes to string, lossy is fine here
                    let str = String::from_utf8_lossy(&bytes);

                    let Ok(scope) = serde_json::from_str(&str) else {
                        tracing::warn!("Failed to decode state scope from CID: {}", key);
                        tx.send(None).unwrap();
                        return;
                    };

                    tracing::info!("*** State loaded from disk.");
                    tx.send(Some(scope)).unwrap();
                });

                if let Some(sco) = rx.recv().unwrap() {
                    scope = sco;
                }
            }
        }

        Self {
            scope: Arc::new(Mutex::new(scope)),
            egui_ctx: ctx,
            name: name.clone().as_ref().to_string(),
            peerpiper,
            cid_map,
            cancel_save: Arc::new(AsyncMutex::new(None)),
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

        // get a clone of the scope
        let scope = self.scope.lock().unwrap().clone();

        tracing::info!("Saving state: {:?}", scope);

        let str = serde_json::to_string(&scope)?;

        tracing::info!("State serialized: {:?}", str);

        let bytes = str.as_bytes().to_vec();

        let command = AllCommands::System(SystemCommand::Put { bytes });
        let Ok(ReturnValues::ID(cid)) = self.peerpiper.lock().await.commander.order(command).await
        else {
            return Err(anyhow::anyhow!("Failed to order command: Put"))?;
        };

        // Save name:cid mapping to platform storage
        // filesystem, localstorage, etc.
        if let Err(e) = self.cid_map.set_string(&self.name, cid.to_string()) {
            tracing::error!("Error saving state: {:?}", e);
        }

        Ok(cid.to_string())
    }
}

impl Inner for State {
    /// Saves the plugin rhai Scope to disk.
    fn save(&self) {
        let state_clone = self.clone();
        let callback = move || {
            let state = state_clone.clone();
            platform::spawn(async move {
                tracing::info!("Saving state");
                match state.async_save().await {
                    Ok(cid) => {
                        tracing::info!("State saved to CID: {:?}", cid);
                    }
                    Err(e) => {
                        tracing::error!("Error saving state: {:?}", e);
                    }
                }
            });
        };

        let cancel_token = self.cancel_save.clone();
        let callback = Arc::new(callback);

        let debouncer = Debouncer {
            cancel_token,
            callback,
            delay: Duration::from_millis(400),
        };

        // spawn a debouncer.debounce().await; task
        platform::spawn(async move {
            debouncer.debounce().await;
        });
    }

    /// Updates the scope variable to the given value
    fn update(&mut self, key: &str, value: impl Into<Dynamic> + Clone) {
        tracing::info!("Updating state: {} = {:?}", key, value.clone().into());
        self.scope
            .lock()
            .unwrap()
            .set_value(key, value.clone().into());

        tracing::info!("State updated: {} = {:?}", key, value.into());

        if let Some(egui_ctx) = &self.egui_ctx {
            tracing::info!("Requesting repaint");
            egui_ctx.request_repaint();
            // now that the rhai scope has been updated, we need to re-run
        } else {
            tracing::warn!("Egui context is not set");
        }

        self.save();
    }

    fn scope(&self) -> ScopeRef {
        ScopeRef::Borrowed(self.scope.clone())
    }

    fn scope_mut(&mut self) -> ScopeRefMut {
        ScopeRefMut::Borrowed(self.scope.lock().unwrap())
    }

    // into_scope with 'static lifetime'
    fn into_scope(self) -> Scope<'static> {
        self.scope.lock().unwrap().clone() // creates a completely new copy of the inner data
    }
}

// Example usage and tests
#[cfg(test)]
mod tests {}
