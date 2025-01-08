//! The PeerPiper Command sender for the Browser
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::platform::Error;
use crate::app::platform;
use crate::app::platform::platform::Blockstore;
use crate::app::rdx_runner::{LayerPlugin, State};

use futures::{
    channel::{
        mpsc::{self},
        oneshot,
    },
    StreamExt,
};

pub use peerpiper::core::events::AllCommands;
use peerpiper::core::events::Events;
use peerpiper::core::events::PublicEvent;
use peerpiper::core::libp2p::api::Libp2pEvent;
pub use peerpiper::core::Commander;
pub use peerpiper::core::ReturnValues;
use rdx::layer::{Instantiator as _, List, ListType, RecordType, Value, ValueType};
use tokio::sync::mpsc::Sender;

/// Simplify the plugins signature type with alias
pub type Plugins = Arc<Mutex<HashMap<String, Arc<Mutex<LayerPlugin<State>>>>>>;

#[derive(Clone)]
pub struct PeerPiper {
    pub commander: Commander<Blockstore>,
    /// The collection of plugins
    pub plugins: Plugins,
}

impl PeerPiper {
    /// Crate a new PeerPiper instanc ewith the given struct which impls both
    /// [peerpiper::core::Blockstore] and
    pub fn new(blockstore: Blockstore, arc_collection: Plugins) -> PeerPiper {
        let commander = Commander::new(blockstore);
        Self {
            commander,
            plugins: arc_collection,
        }
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
    ) -> Result<impl FnOnce(Sender<PublicEvent>), Error> {
        // 16 is arbitrary, but should be enough for now
        let (tx_evts, mut rx_evts) = mpsc::channel(16);

        // client sync oneshot
        let (tx_client, rx_client) = oneshot::channel();

        // command_sender will be used by other wasm_bindgen functions to send commands to the network
        // so we will need to wrap it in a Mutex or something to make it thread safe.
        let (network_command_sender, network_command_receiver) = tokio::sync::mpsc::channel(8);

        let bstore = self.commander.blockstore.clone();

        platform::spawn(async move {
            peerpiper::start(
                tx_evts,
                network_command_receiver,
                tx_client,
                libp2p_endpoints,
                bstore,
            )
            .await
            .expect("never end")
        });

        // wait on rx_client to get the client handle
        let client_handle = rx_client.await?;

        self.commander
            .with_network(network_command_sender)
            .with_client(client_handle);

        // enable caller to Start listening for events from the network and handle them.
        // Any [Libp2pEvent] received will be handled by the plugins.
        // Any [PublicEvent] received will be sent to the `on_event` callback.
        let plugins = self.plugins.clone();
        let listen = |on_event: Sender<PublicEvent>| {
            platform::spawn(async move {
                while let Some(event) = rx_evts.next().await {
                    match event {
                        // Outter/Public events are not handled by plugins
                        Events::Outer(public_event) => {
                            tracing::debug!("Received event: {:?}", &public_event);
                            on_event.send(public_event).await.unwrap();
                        }
                        // Inner events are events that can be handled by plugins
                        Events::Inner(libp2p_evt) => {
                            tracing::debug!("Received inner libp2p event: {:?}", &libp2p_evt);
                            match libp2p_evt {
                                Libp2pEvent::InboundRequest {
                                    request,
                                    channel: _,
                                } => {
                                    tracing::debug!("Received inbound request: {:?}", &request);
                                }
                                Libp2pEvent::DhtProviderRequest { key: _, channel: _ } => todo!(),
                                Libp2pEvent::PutRecordRequest { source, record } => {
                                    tracing::info!("Received PutRecordRequest from: {:?}", &source);
                                    // need a connection to the list of plugins, which
                                    // are located in RDXRunner.
                                    let list_data = ValueType::List(ListType::new(ValueType::U8));

                                    let key = Value::List(
                                        List::new(
                                            ListType::new(ValueType::U8),
                                            record
                                                .key
                                                .to_vec()
                                                .iter()
                                                .map(|u| Value::U8(*u))
                                                .collect::<Vec<_>>(),
                                        )
                                        .unwrap(),
                                    );

                                    let value = Value::List(
                                        List::new(
                                            ListType::new(ValueType::U8),
                                            record
                                                .value
                                                .iter()
                                                .map(|u| Value::U8(*u))
                                                .collect::<Vec<_>>(),
                                        )
                                        .unwrap(),
                                    );

                                    let peer = Value::List(
                                        List::new(
                                            ListType::new(ValueType::U8),
                                            source
                                                .to_bytes()
                                                .to_vec()
                                                .iter()
                                                .map(|u| Value::U8(*u))
                                                .collect::<Vec<_>>(),
                                        )
                                        .unwrap(),
                                    );

                                    let values =
                                        vec![("key", key), ("value", value), ("peer", peer)];

                                    let kad_record = rdx::layer::Value::Record(
                                        rdx::wasm_component_layer::Record::new(
                                            RecordType::new(
                                                None,
                                                vec![
                                                    ("key", list_data.clone()),
                                                    ("value", list_data.clone()),
                                                    ("peer", list_data.clone()),
                                                ],
                                            )
                                            .unwrap(),
                                            values,
                                        )
                                        .unwrap(),
                                    );
                                    plugins.lock().unwrap().iter().for_each(|(name, plugin)| {
                                        tracing::debug!(
                                            "Calling handle-put-record-request with plugin: {:?}",
                                            name
                                        );
                                        let mut plugin_deets = plugin.lock().unwrap();
                                        if let Err(e) = plugin_deets.call(
                                            "handle-put-record-request",
                                            &[kad_record.clone()],
                                        ) {
                                            tracing::error!("Error calling plugin: {:?}", e);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            });
        };

        Ok(listen)
    }
}
