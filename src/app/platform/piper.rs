//! The PeerPiper Command sender for the Browser
#![allow(dead_code)]

use super::platform::Error;
use crate::app::platform;
use crate::app::platform::platform::Blockstore;
use futures::SinkExt;
use futures::{
    channel::{
        mpsc::{self, Sender},
        oneshot,
    },
    StreamExt,
};
pub use peerpiper::core::events::AllCommands;
use peerpiper::core::events::PublicEvent;
pub use peerpiper::core::Commander;
pub use peerpiper::core::ReturnValues;

#[derive(Debug, Clone)]
pub struct PeerPiper {
    // / Make interior mutability possible for the Commander struct with [RefCell]
    // / This way we can keep the idiomatic Rust way of borrowing and mutating with &self
    commander: Commander<Blockstore>,
}

impl PeerPiper {
    /// Crate a new PeerPiper instanc ewith the given struct which impls both
    /// [peerpiper::core::wnfs_common::blockstore::BlockStore] and [SystemCommandHandler]
    pub fn new(handler: Blockstore) -> PeerPiper {
        let commander = Commander::new(handler);
        Self { commander }
    }

    /// Send Commands to PeerPiper whether connected or not.
    ///
    /// Throws an error if network command are sent before connecting to the network.
    ///
    /// put and get can store and retrieve data locally without network connection.
    pub async fn order(&self, command: AllCommands) -> Result<ReturnValues, Error> {
        Ok(self.commander.order(command).await?)
    }

    /// Try to connect to the list of endpoints.
    /// Send the `on_event` callback to the Commander to be called when an event is received.
    pub async fn connect(
        &mut self,
        libp2p_endpoints: Vec<String>,
        mut on_event: Sender<PublicEvent>,
    ) -> Result<(), Error> {
        // 16 is arbitrary, but should be enough for now
        let (tx_evts, mut rx_evts) = mpsc::channel(16);

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

        self.commander
            .with_network(network_command_sender)
            .with_client(client_handle);

        while let Some(event) = rx_evts.next().await {
            if let peerpiper::core::events::Events::Outer(event) = event {
                log::debug!("[Browser] Received event: {:?}", &event);
                on_event.send(event).await?;
            }
        }

        Ok(())
    }
}
