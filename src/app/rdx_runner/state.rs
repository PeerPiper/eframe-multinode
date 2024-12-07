//! Custom State for the RDX Plugins, Implements [rdx::layer::Inner] and custom Serialize/Deserialize
//! so that the [rdx::layer::rhai::Scope] can be serialized and deserialized.
use crate::app::platform;
use crate::app::platform::peerpiper::PeerPiper;
use crate::app::platform::StringStore;
use peerpiper::core::events::AllCommands;
use peerpiper::core::events::SystemCommand;
use peerpiper::core::Cid;
use peerpiper::core::ReturnValues;
use rdx::layer::{
    rhai::{Dynamic, Scope},
    Inner,
};
use std::sync::Arc;
//use std::sync::Mutex;
use tokio::sync::Mutex;

/// Serializable [State] struct that holds the [Scope] and [egui::Context]
#[derive(Debug, Clone, Default)]
//#[serde(default)]
pub struct State {
    /// The unique name of this plugin, typically the filename
    name: String,
    /// The [Scope] that holds the state of the plugin
    scope: Scope<'static>,
    /// The [egui::Context] that holds the UI state. Used to request repaints
    egui_ctx: Option<egui::Context>,
    /// Handler to PeerPiper SystemCommander
    peerpiper: Option<Arc<Mutex<PeerPiper>>>,
    /// String Store map the name of the plugin to the CID of the state
    cid_map: StringStore,
}

impl State {
    pub fn new(
        name: impl AsRef<str> + Clone,
        ctx: Option<egui::Context>,
        peerpiper: Option<Arc<Mutex<PeerPiper>>>,
    ) -> Self {
        let mut scope = Scope::new();

        let cid_map = StringStore::new(name.as_ref());

        // Load the CID of the state from the platform storage
        // filesystem, localstorage, etc.
        // and load the state from the CID
        // and set the scope to the loaded state
        // if the CID is not found, create a new scope
        // and set the scope to the new scope
        if let Some(key) = cid_map.get_string(name.as_ref()) {
            if let Ok(cid) = Cid::try_from(key.clone()) {
                if let Some(ref peerpiper) = peerpiper {
                    let (tx, rx) = std::sync::mpsc::channel();
                    let peerpiper = peerpiper.clone();
                    platform::spawn(async move {
                        let command = AllCommands::System(SystemCommand::Get { key: cid.into() });
                        let pp = {
                            let mut peerpiper = peerpiper.lock().await;
                            peerpiper.order(command).await
                        };

                        let Ok(ReturnValues::Data(bytes)) = pp else {
                            tracing::warn!("Failed to get state from CID: {}", key);
                            tx.send(None).unwrap();
                            return;
                        };

                        let Ok(scope): Result<Scope, cbor4ii::serde::DecodeError<_>> =
                            cbor4ii::serde::from_slice(&bytes)
                        else {
                            tracing::warn!("Failed to decode state scope from CID: {}", key);
                            tx.send(None).unwrap();
                            return;
                        };
                        tx.send(Some(scope)).unwrap();
                    });

                    if let Some(sco) = rx.recv().unwrap() {
                        scope = sco;
                    }
                }
            }
        }
        tracing::info!("*** State loaded: {:?}", scope);

        Self {
            scope,
            egui_ctx: ctx,
            name: name.clone().as_ref().to_string(),
            peerpiper,
            cid_map,
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
        let bytes = cbor4ii::serde::to_vec(Vec::new(), &self.scope)?;
        // Save the serialized state to disk, independent of the platform
        // for this we can use peerpiper SystemCommandHanlder to put the bytes into the local system
        let Some(cmdr) = &self.peerpiper else {
            tracing::warn!("Commander is not set yet");
            return Err(anyhow::anyhow!("Commander is not set yet"))?;
        };

        let mut cmdr = cmdr.lock().await;
        let command = AllCommands::System(SystemCommand::Put { bytes });
        let Ok(ReturnValues::ID(cid)) = cmdr.order(command).await else {
            return Err(anyhow::anyhow!("Failed to order command: Put"))?;
        };

        // Save name:cid mapping to platform storage
        // filesystem, localstorage, etc.
        self.cid_map.set_string(&self.name, cid.to_string());

        Ok(cid.to_string())
    }
}

impl Inner for State {
    /// Updates the scope variable to the given value
    fn update(&mut self, key: &str, value: impl Into<Dynamic> + Copy) {
        self.scope.set_or_push(key, value.into());

        let clone = self.clone();
        platform::spawn(async move {
            tracing::info!("Saving state");
            match clone.save().await {
                Ok(cid) => {
                    tracing::info!("State saved to CID: {:?}", cid);
                }
                Err(e) => {
                    tracing::error!("Error saving state: {:?}", e);
                }
            }
        });

        if let Some(egui_ctx) = &self.egui_ctx {
            tracing::info!("Requesting repaint");
            egui_ctx.request_repaint();
            // now that the rhai scope has been updated, we need to re-run
        } else {
            tracing::warn!("Egui context is not set");
        }
    }

    fn scope(&self) -> &Scope {
        &self.scope
    }

    fn scope_mut(&mut self) -> &mut Scope<'static> {
        &mut self.scope
    }

    // into_scope with 'static lifetime'
    fn into_scope(self) -> Scope<'static> {
        self.scope
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
