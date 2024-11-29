//! The PeerPiper Command sender for the Browser
#![allow(dead_code)]

use super::web_error::WebError;
use crate::app::platform;
use futures::SinkExt;
use futures::{
    channel::{
        mpsc::{self, Sender},
        oneshot,
    },
    StreamExt,
};
use peerpiper_browser::opfs::OPFSBlockstore;
//use peerpiper_core::events::PeerPiperCommand;
use peerpiper_core::events::PublicEvent;
use peerpiper_core::Commander;

pub struct PeerPiper {
    // / Make interior mutability possible for the Commander struct with [RefCell]
    // / This way we can keep the idiomatic Rust way of borrowing and mutating with &self
    commander: Commander<OPFSBlockstore>,
}

impl PeerPiper {
    pub async fn new() -> Result<PeerPiper, WebError> {
        let blockstore = OPFSBlockstore::new()
            .await
            .map_err(|e| WebError::OPFSBlockstore(e.as_string().unwrap_or_default()))?;
        let commander = Commander::new(blockstore);
        Ok(Self { commander })
    }

    /// Try to connect to the list of endpoints.
    /// Send the `on_event` callback to the Commander to be called when an event is received.
    pub async fn connect(
        &mut self,
        libp2p_endpoints: Vec<String>,
        mut on_event: Sender<PublicEvent>,
    ) -> Result<(), WebError> {
        // 16 is arbitrary, but should be enough for now
        let (tx_evts, mut rx_evts) = mpsc::channel(16);

        // client sync oneshot
        let (tx_client, rx_client) = oneshot::channel();

        // command_sender will be used by other wasm_bindgen functions to send commands to the network
        // so we will need to wrap it in a Mutex or something to make it thread safe.
        let (command_sender, command_receiver) = mpsc::channel(8);

        platform::spawn(async move {
            peerpiper_browser::start(tx_evts, command_receiver, tx_client, libp2p_endpoints)
                .await
                .expect("never end")
        });

        // wait on rx_client to get the client handle
        let client_handle = rx_client.await?;

        self.commander
            .with_network(command_sender)
            .with_client(client_handle);

        while let Some(event) = rx_evts.next().await {
            match event {
                peerpiper_core::events::Events::Outer(event) => {
                    log::debug!("[Browser] Received event: {:?}", &event);
                    on_event.send(event).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
