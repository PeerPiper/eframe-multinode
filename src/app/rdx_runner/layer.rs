//! multinode specific layers
use core::task::{Context, Poll};
use std::{
    any::Any,
    collections::HashMap,
    future::Future,
    pin::{pin, Pin},
    sync::{Arc, Mutex},
};

use peerpiper::core::{events::AllCommands, ReturnValues};
use peerpiper::core::{events::SystemCommand, Cid};
use rdx::layer::{
    noop_waker,
    poll::{MakeFuture, PollableFuture},
    rhai::Dynamic,
    Component, Engine, Error, Func, FuncType, Inner, Instance, Linker, List, ListType, Pollable,
    RecordType, Resource, ResourceTable, ResourceType, Store, SystemTime, Value, ValueType,
};
use rdx::wasm_component_layer::{ResultType, VariantCase, VariantType};
use rdx::{layer::*, wasm_component_layer::ResultValue};

#[cfg(target_arch = "wasm32")]
use send_wrapper::SendWrapper;
#[cfg(target_arch = "wasm32")]
use std::ops::Deref;
#[cfg(target_arch = "wasm32")]
use std::ops::DerefMut;

//#[cfg(not(target_arch = "wasm32"))]
//use tokio::sync::Mutex as AsyncMutex;

//use crate::app::platform::peerpiper::PeerPiper;

use crate::app::platform;

use super::PeerPiperWired;

/// Use wasm_component_layer to intanitate a plugin and some state data
pub struct LayerPlugin<T: Inner + Send + Sync> {
    #[cfg(target_arch = "wasm32")]
    pub(crate) store: SendWrapper<Store<T, runtime_layer::Engine>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) store: Store<T, runtime_layer::Engine>,
    raw_instance: Instance,
}

impl<T: Inner + Clone + Send + Sync + 'static> LayerPlugin<T> {
    /// Creates a new with the given wallet layer as a dependency
    pub fn new(
        bytes: &[u8],
        data: T,
        wallet_layer: Option<Arc<Mutex<dyn Instantiator<T>>>>,
        commander: Option<PeerPiperWired>,
    ) -> Self {
        let (instance, store) = instantiate_instance(bytes, data, wallet_layer, commander);

        Self {
            #[cfg(target_arch = "wasm32")]
            store: SendWrapper::new(store),
            #[cfg(not(target_arch = "wasm32"))]
            store,
            raw_instance: instance,
        }
    }
}

impl<T: Inner + Send + Sync + 'static> Instantiator<T> for LayerPlugin<T> {
    fn store(&self) -> &Store<T, runtime_layer::Engine> {
        &self.store
    }

    fn store_mut(&mut self) -> &mut Store<T, runtime_layer::Engine> {
        &mut self.store
    }

    fn call(&mut self, name: &str, arguments: &[Value]) -> Result<Option<Value>, Error> {
        tracing::trace!("Calling function: {}", name);
        let export_instance = self
            .raw_instance
            .exports()
            .instance(&"component:plugin/run".try_into()?)
            .ok_or(Error::InstanceNotFound)?;

        let func = export_instance
            .func(name)
            .ok_or_else(|| Error::FuncNotFound(name.to_string()))?;

        let func_result_len = func.ty().results().len();
        let binding = func.ty();
        let func_result_ty = binding.results().first().unwrap_or(&ValueType::Bool);
        //let mut results = vec![Value::Bool(false); func_result_len];
        // generate mut results from func_result_ty
        let mut results: Vec<Value> = match func_result_ty {
            ValueType::Result(_) => {
                vec![
                    Value::Result(ResultValue::new(
                        ResultType::new(None, Some(ValueType::String)),
                        Ok(None)
                    )?);
                    func_result_len
                ]
            }
            ValueType::Bool => vec![Value::Bool(false); func_result_len],
            ValueType::U8 => vec![Value::U8(0); func_result_len],
            ValueType::List(_) => {
                vec![Value::List(List::new(ListType::new(ValueType::U8), vec![])?); func_result_len]
            }
            _ => vec![Value::Bool(false); func_result_len],
        };

        #[cfg(target_arch = "wasm32")]
        func.call(self.store.deref_mut(), arguments, &mut results)
            .map_err(|e| {
                tracing::error!("Error calling function: {:?}", e);
                e
            })?;

        #[cfg(not(target_arch = "wasm32"))]
        func.call(&mut self.store, arguments, &mut results)
            .map_err(|e| {
                tracing::error!("Error calling function: {:?}", e);
                e
            })?;

        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results.remove(0)))
        }
    }
}

pub fn instantiate_instance<T: Inner + Clone + Send + Sync + 'static>(
    bytes: &[u8],
    data: T,
    wallet_layer: Option<Arc<Mutex<dyn Instantiator<T>>>>,
    peerpiper: Option<PeerPiperWired>,
) -> (Instance, Store<T, runtime_layer::Engine>) {
    let table = Arc::new(Mutex::new(ResourceTable::new()));

    // Create a new engine for instantiating a component.
    let engine = Engine::new(runtime_layer::Engine::default());

    // Create a store for managing WASM data and any custom user-defined state.
    let mut store = Store::new(&engine, data);

    // Parse the component bytes and load its imports and exports.
    let component = Component::new(&engine, bytes).unwrap();
    // Create a linker that will be used to resolve the component's imports, if any.
    let mut linker = Linker::default();

    // Pollable resource type
    let resource_pollable_ty = ResourceType::new::<Resource<Pollable>>(None);

    let list_data = ValueType::List(ListType::new(ValueType::U8));

    // pollable is wasi:io/poll
    let poll_interface = linker
        .define_instance("wasi:io/poll@0.2.2".try_into().unwrap())
        .unwrap();

    poll_interface
        .define_resource("pollable", resource_pollable_ty.clone())
        .unwrap();

    // ready and block are methods on the pollable resource, "[method]pollable.ready" and "[method]pollable.block"
    //ready: func() -> bool;
    let table_clone = table.clone();
    poll_interface
        .define_func(
            "[method]pollable.ready",
            Func::new(
                &mut store,
                FuncType::new(
                    [ValueType::Borrow(resource_pollable_ty.clone())],
                    [ValueType::Bool],
                ),
                move |store, params, results| {
                    tracing::info!("[method]pollable.ready");

                    let Value::Borrow(pollable_resource) = &params[0] else {
                        panic!("Incorrect input type, found {:?}", params[0]);
                    };

                    tracing::info!("Got borrow param pollable {:?}", pollable_resource);

                    let binding = store.as_context();
                    let res_pollable: &Resource<Pollable> =
                        pollable_resource.rep(&binding).map_err(|e| {
                            tracing::error!("Error getting pollable resource: {:?}", e);
                            e
                        })?;

                    tracing::info!("Got pollable resource");

                    // get pollable from table
                    // get inner table
                    let table: &mut ResourceTable = &mut table_clone.lock().unwrap();

                    let pollable = table.get(res_pollable)?;

                    let ready = (pollable.make_future)(table.get_any_mut(pollable.index)?);

                    tracing::info!("Got ready");

                    let mut fut = pin!(ready);
                    let waker = noop_waker();
                    let mut cx = Context::from_waker(&waker);

                    // Poll the future once
                    let poll_result = fut.as_mut().poll(&mut cx);

                    // Check the result
                    let ready = matches!(poll_result, Poll::Ready(()));

                    tracing::info!("[ready] Poll result: {:?}", ready);

                    // if not ready, save the future to the table

                    results[0] = Value::Bool(ready);
                    Ok(())
                },
            ),
        )
        .unwrap();

    poll_interface
        .define_func(
            "[method]pollable.block",
            Func::new(
                &mut store,
                FuncType::new([], []),
                move |_store, _params, _results| {
                    tracing::info!("[method]pollable.block");
                    //todo!();
                    Ok(())
                },
            ),
        )
        .unwrap();

    // poll: func(in: list<borrow<pollable>>) -> list<u32>;
    let table_clone = table.clone();
    poll_interface
        .define_func(
            "poll",
            Func::new(
                &mut store,
                FuncType::new(
                    [ValueType::List(ListType::new(ValueType::Borrow(
                        resource_pollable_ty.clone(),
                    )))],
                    [ValueType::List(ListType::new(ValueType::U32))],
                ),
                move |mut store, params, results| {
                    tracing::info!("[method]pollable.poll");

                    type ReadylistIndex = u32;

                    tracing::debug!("[poll]: convert list to pollables");

                    let pollables = match &params[0] {
                        Value::List(pollables) => pollables,
                        _ => bail!("Incorrect input type"),
                    };

                    tracing::debug!("[poll]: check if pollables is empty");

                    if pollables.is_empty() {
                        bail!("Empty pollables list");
                    }

                    tracing::debug!("[poll]: create table futures");

                    let mut table_futures: HashMap<u32, (MakeFuture, Vec<ReadylistIndex>)> =
                        HashMap::new();

                    for (ix, p) in pollables.iter().enumerate() {
                        let ix: u32 = ix.try_into()?;

                        tracing::debug!("[poll]: get pollable resource");

                        let Value::Borrow(pollable_resource) = p else {
                            bail!("Incorrect input type, found {:?}", p);
                        };

                        let mut binding = store.as_context_mut();
                        let p: &mut Resource<Pollable> = pollable_resource.rep_mut(&mut binding)?;

                        let binding = table_clone.lock().unwrap();
                        let pollable = binding.get(p)?;
                        let (_, list) = table_futures
                            .entry(pollable.index)
                            .or_insert((pollable.make_future, Vec::new()));
                        list.push(ix);
                    }

                    let mut futures: Vec<(PollableFuture<'_>, Vec<ReadylistIndex>)> = Vec::new();

                    let mut binding = table_clone.lock().unwrap();

                    let it = table_futures.into_iter().map(move |(k, v)| {
                        let item = binding
                            .occupied_mut(k)
                            .map(|e| Box::as_mut(&mut e.entry))
                            // Safety: extending the lifetime of the mutable reference.
                            .map(|item| unsafe { &mut *(item as *mut dyn Any) });
                        (item, v)
                    });

                    for (entry, (make_future, readylist_indices)) in it {
                        let entry = entry?;
                        futures.push((make_future(entry), readylist_indices));
                    }

                    struct PollList<'a> {
                        futures: Vec<(PollableFuture<'a>, Vec<ReadylistIndex>)>,
                    }

                    impl Future for PollList<'_> {
                        type Output = Vec<u32>;

                        fn poll(
                            mut self: Pin<&mut Self>,
                            cx: &mut Context<'_>,
                        ) -> Poll<Self::Output> {
                            let mut any_ready = false;
                            let mut results = Vec::new();
                            for (fut, readylist_indicies) in self.futures.iter_mut() {
                                match fut.as_mut().poll(cx) {
                                    Poll::Ready(()) => {
                                        results.extend_from_slice(readylist_indicies);
                                        any_ready = true;
                                    }
                                    Poll::Pending => {}
                                }
                            }
                            if any_ready {
                                Poll::Ready(results)
                            } else {
                                Poll::Pending
                            }
                        }
                    }

                    tracing::debug!("[poll]: return poll list");

                    // We set results[0] to be the sync equivalent to: PollList { futures }.await
                    results[0] = Value::List(List::new(
                        ListType::new(ValueType::U32),
                        futures
                            .into_iter()
                            // only add to the returned list if the future is ready, otherwise skip
                            // the future until next time
                            .filter_map(|(mut fut, readylist_indices)| {
                                let waker = noop_waker();
                                let mut cx = Context::from_waker(&waker);
                                match fut.as_mut().poll(&mut cx) {
                                    Poll::Ready(()) => Some(readylist_indices),
                                    Poll::Pending => None,
                                }
                            })
                            .flatten()
                            .map(Value::U32)
                            .collect::<Vec<_>>(),
                    )?);

                    Ok(())
                },
            ),
        )
        .unwrap();

    let host_interface = linker
        .define_instance("host:component/host".try_into().unwrap())
        .unwrap();

    host_interface
        .define_func(
            "log",
            Func::new(
                &mut store,
                FuncType::new([ValueType::String], []),
                move |_store, params, _results| {
                    if let Value::String(s) = &params[0] {
                        tracing::info!("{}", s);
                    }
                    Ok(())
                },
            ),
        )
        .unwrap();

    ///// Event type where value is a string.
    //record string-event {
    //  /// The variable name
    //  name: string,
    //  value: string
    //}
    //
    ///// Event type where value is a list<u8>.
    //record bytes-event {
    //  /// The variable name
    //  name: string,
    //  value: list<u8>
    //}
    //
    ///// Event wherte there is a list of strings
    //record string-list-event {
    //  /// The variable name
    //  name: string,
    //  value: list<string>
    //}
    //
    ///// Event is a variant of string and bytes events.
    //variant event {
    //  save,
    //  text(string-event),
    //  bytes(bytes-event),
    //  string-list(string-list-event)
    //}

    let save_variant_case = VariantCase::new("save", None);

    let text_variant_case = VariantCase::new(
        "text",
        Some(ValueType::Record(
            RecordType::new(
                None,
                vec![("name", ValueType::String), ("value", ValueType::String)],
            )
            .unwrap(),
        )),
    );

    let bytes_variant = VariantCase::new(
        "bytes",
        Some(ValueType::Record(
            RecordType::new(
                None,
                vec![("name", ValueType::String), ("value", list_data.clone())],
            )
            .unwrap(),
        )),
    );

    let string_list_variant = VariantCase::new(
        "string-list",
        Some(ValueType::Record(
            RecordType::new(
                None,
                vec![
                    ("name", ValueType::String),
                    ("value", ValueType::List(ListType::new(ValueType::String))),
                ],
            )
            .unwrap(),
        )),
    );

    let event = VariantType::new(
        None,
        vec![
            save_variant_case,
            text_variant_case,
            bytes_variant,
            string_list_variant,
        ],
    )
    .unwrap();

    host_interface
        .define_func(
            "emit",
            Func::new(
                &mut store,
                FuncType::new([ValueType::Variant(event)], []),
                move |mut store, params, _results| {
                    if let Value::Variant(variant) = &params[0] {
                        match variant.ty().cases()[variant.discriminant()].name() {
                            "save" => {
                                tracing::info!("Save event");
                                store.data().save();
                            }
                            "text" => {
                                tracing::info!("Text event {:?}", variant.value());
                                if let Some(Value::Record(record)) = variant.value() {
                                    if let Some(Value::String(name)) = record.field("name") {
                                        if let Some(Value::String(value)) = record.field("value") {
                                            tracing::info!(
                                                "[layer]: Name: {}, Value: {}",
                                                name,
                                                value
                                            );
                                            store.data_mut().update(&name, &*value);
                                        }
                                    }
                                }
                            }
                            "bytes" => {
                                if let Some(Value::Record(record)) = variant.value() {
                                    if let Some(Value::String(name)) = record.field("name") {
                                        if let Some(Value::List(list)) = record.field("value") {
                                            let vals = list
                                                .iter()
                                                .map(|v| match v {
                                                    Value::U8(u) => Dynamic::from(u),
                                                    Value::String(s) => {
                                                        let s: String = s.to_string();
                                                        Dynamic::from(s)
                                                    }
                                                    _ => Dynamic::from("Unsupported type"),
                                                })
                                                .collect::<Vec<_>>();
                                            store.data_mut().update(&name, &*vals);
                                        }
                                    }
                                }
                            }
                            "string-list" => {
                                if let Some(Value::Record(record)) = variant.value() {
                                    if let Some(Value::String(name)) = record.field("name") {
                                        if let Some(Value::List(list)) = record.field("value") {
                                            let vals = list
                                                .iter()
                                                .map(|v| match v {
                                                    Value::String(s) => {
                                                        let s: String = s.to_string();
                                                        Dynamic::from(s)
                                                    }
                                                    _ => Dynamic::from("Unsupported type"),
                                                })
                                                .collect::<Vec<_>>();
                                            store.data_mut().update(&name, &*vals);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    Ok(())
                },
            ),
        )
        .unwrap();

    // add func get_random
    host_interface
        .define_func(
            "random-byte",
            Func::new(
                &mut store,
                FuncType::new([], [ValueType::U8]),
                move |_store, _params, results| {
                    let random = rand::random::<u8>();
                    results[0] = Value::U8(random);
                    Ok(())
                },
            ),
        )
        .unwrap();

    // now function
    host_interface
        .define_func(
            "now",
            Func::new(
                &mut store,
                FuncType::new([], [ValueType::S64]),
                move |_store, _params, results| {
                    let unix_timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    results[0] = Value::S64(unix_timestamp);
                    Ok(())
                },
            ),
        )
        .unwrap();

    if let Some(wallet_layer) = wallet_layer {
        // add get-mk and prove as host functions.
        // These will be bound to the exports of the wallet plugin.
        // Which means we need to call the wallet plugin explicitly to use these functions.
        let key_args_record = RecordType::new(
            None,
            vec![
                ("key", ValueType::String),
                ("codec", ValueType::String),
                ("threshold", ValueType::U8),
                ("limit", ValueType::U8),
            ],
        )
        .unwrap();

        let wallet_clone = wallet_layer.clone();

        // return type is:  result<list<u8>, variant<plog: string, wallet-uninitialized, multikey-error: string, config: string>>
        let ok = list_data.clone();
        let err = ValueType::Variant(
            VariantType::new(
                None,
                vec![
                    VariantCase::new("invalid-codec", Some(ValueType::String)),
                    VariantCase::new("wallet-uninitialized", None),
                    VariantCase::new("multikey-error", Some(ValueType::String)),
                    VariantCase::new("key-not-found", Some(ValueType::String)),
                ],
            )
            .unwrap(),
        );

        let return_type = ResultType::new(Some(ok.clone()), Some(err.clone()));

        //
        host_interface
            .define_func(
                "get-mk",
                Func::new(
                    &mut store,
                    FuncType::new(
                        [ValueType::Record(key_args_record)],
                        // result is: result<list<u8>, variant<plog: string, wallet-uninitialized, multikey-error: string, config: string>>
                        [ValueType::Result(return_type)],
                    ),
                    move |_store, params, results| {
                        let Value::Record(key_args_record) = &params[0] else {
                            panic!("Incorrect input type, found {:?}", params[0]);
                        };
                        let mk = wallet_clone
                            .lock()
                            .unwrap()
                            .call("get-mk", &[Value::Record(key_args_record.clone())])?;
                        results[0] = mk.unwrap();
                        Ok(())
                    },
                ),
            )
            .unwrap();

        // prove
        let prove_args_record = RecordType::new(
            None,
            vec![
                // multikey is a list of u8
                ("mk", list_data.clone()),
                ("data", list_data.clone()),
            ],
        )
        .unwrap();

        let wallet_clone = wallet_layer.clone();

        host_interface
            .define_func(
                "prove",
                Func::new(
                    &mut store,
                    FuncType::new(
                        [ValueType::Record(prove_args_record)],
                        [ValueType::Result(ResultType::new(Some(ok), Some(err)))],
                    ),
                    move |_store, params, results| {
                        let Value::Record(prove_args_record) = &params[0] else {
                            panic!("Incorrect input type, found {:?}", params[0]);
                        };
                        let proof = wallet_clone
                            .lock()
                            .unwrap()
                            .call("prove", &[Value::Record(prove_args_record.clone())])?;
                        results[0] = proof.unwrap();
                        Ok(())
                    },
                ),
            )
            .unwrap();

        // returns the rhai scope
        host_interface
            .define_func(
                "get-scope",
                Func::new(
                    &mut store,
                    FuncType::new([], [ValueType::String]),
                    move |store, _params, results| {
                        let scope = store.data().clone().into_scope();
                        let serde_json_str = serde_json::to_string(&scope).unwrap();
                        results[0] = Value::String(serde_json_str.into());
                        Ok(())
                    },
                ),
            )
            .unwrap();
    }

    // if peerpiper_layer is Some, we can also add the peerpiper functions
    // We can wire peerpiper AllCommands as host functions.
    // For now, let's do get/put for Blockstore actions
    if let Some(cmdr) = peerpiper {
        // Wrap cmdr in SendWrapper in wasm32, as Func needs Send
        #[cfg(target_arch = "wasm32")]
        let cmdr = SendWrapper::new(cmdr);

        // Let's provide `commander.order()` function to the plugins
        // these orders are async, but don't always return something
        // for put/get, we get a root CID as a result,
        // which we always save to the plugin's state StringStore

        // actually you just need to save the key:value bytes to the rhai Scope,
        // and the periodic save() will take care of the rest.

        //interface peerpiper {
        //
        //  /// Publsih data to a topic
        //  record publish {
        //    /// The topic
        //    topic: string,
        //    /// The data
        //    data: list<u8>
        //  }
        //
        //  record keyed-bytes {
        //    key: list<u8>,
        //    value: list<u8>
        //  }
        //  variant system-command {
        //    /// Put bytes on the local disk
        //    put(list<u8>),
        //    /// Puts keyed bytes on the local disk
        //    put-keyed(keyed-bytes),
        //    /// Get bytes from the local disk
        //    get(list<u8>),
        //  }
        //
        //  /// Make a Rwquest from a Peer
        //  /// The request is encoded as a list of bytes
        //  record peer-request {
        //    /// The request
        //    request: list<u8>,
        //    /// The peer id
        //    peer-id: string
        //  }
        //
        //  /// Put bytes in the DHT
        //  record put-record {
        //    /// The key
        //    key: list<u8>,
        //    /// The value
        //    value: list<u8>
        //  }
        //
        //  /// Get Record that has this key
        //  record get-record {
        //    /// The key
        //    key: list<u8>
        //  }
        //
        //  variant all-commands {
        //    /// Publish data to a topic
        //    publish(publish),
        //    /// Subscribe to a topic
        //    subscribe(string),
        //    /// Unsubscribe from a topic
        //    unsubscribe(string),
        //    /// System commands are a subset of [AllCommands] that do not go to the network, but come
        //    /// from componets to direct the system to do something, like save bytes to a file.
        //    system(system-command),
        //    /// Please peer, do something with this data and give me a response
        //    peer-request(peer-request),
        //    /// Puts a Record on the DHT, and optionally provides the data for Pinning
        //    put-record(put-record),
        //    /// Gets the Providers of a Record on the DHT
        //    get-providers(list<u8>),
        //    /// Start Providing a Record on the DHT
        //    start-providing(list<u8>),
        //  }
        //
        //  variant return-values {
        //    /// The data
        //    data(list<u8>),
        //    /// The ID
        //    id(string),
        //    /// The providers
        //    providers(list<string>),
        //    /// None
        //    none,
        //  }
        //
        //}
        //
        // order: func(order: all-commands) -> return-values;

        let peer_request_record = RecordType::new(
            None,
            vec![
                ("request", list_data.clone()),
                ("peer-id", ValueType::String),
            ],
        )
        .unwrap();

        let put_record = RecordType::new(
            None,
            vec![("key", list_data.clone()), ("value", list_data.clone())],
        )
        .unwrap();

        let system_command_variant = VariantType::new(
            None, // host:component/peerpiper.system-command
            vec![
                VariantCase::new("put", Some(list_data.clone())),
                VariantCase::new("put-keyed", Some(ValueType::Record(put_record.clone()))),
                VariantCase::new("get", Some(list_data.clone())),
            ],
        )
        .unwrap();

        let _return_value_ty = VariantType::new(
            None,
            vec![
                VariantCase::new("data", Some(list_data.clone())),
                VariantCase::new("id", Some(ValueType::String)),
                VariantCase::new(
                    "providers",
                    Some(ValueType::List(ListType::new(ValueType::String))),
                ),
                VariantCase::new("none", None),
            ],
        )
        .unwrap();

        host_interface
            .define_func(
                "order",
                Func::new(
                    &mut store,
                    FuncType::new(
                        // params are all-commands variant
                        [ValueType::Variant(
                            VariantType::new(
                                None,
                                vec![
                                    VariantCase::new(
                                        "publish",
                                        Some(ValueType::Record(
                                            RecordType::new(
                                                None,
                                                vec![
                                                    ("topic", ValueType::String),
                                                    (
                                                        "data",
                                                        ValueType::List(ListType::new(
                                                            ValueType::U8,
                                                        )),
                                                    ),
                                                ],
                                            )
                                            .unwrap(),
                                        )),
                                    ),
                                    VariantCase::new("subscribe", Some(ValueType::String)),
                                    VariantCase::new("unsubscribe", Some(ValueType::String)),
                                    VariantCase::new(
                                        "system",
                                        Some(ValueType::Variant(system_command_variant.clone())),
                                    ),
                                    VariantCase::new(
                                        "peer-request",
                                        Some(ValueType::Record(peer_request_record.clone())),
                                    ),
                                    VariantCase::new(
                                        "put-record",
                                        Some(ValueType::Record(put_record.clone())),
                                    ),
                                    VariantCase::new("get-record", Some(list_data.clone())),
                                    VariantCase::new("get-providers", Some(list_data.clone())),
                                    VariantCase::new("start-providing", Some(list_data.clone())),
                                ],
                            )
                            .unwrap(),
                        )],
                        // results are return-values variant
                        //[ValueType::Variant(return_value_ty.clone())],
                        [],
                    ),
                    move |mut store, params, _results| {
                        let mut command = None;
                        let mut key = None;
                        let mut key_variant = None;

                        if let Value::Variant(variant) = &params[0] {
                            //tracing::info!("Got variant : {:?}", variant);
                            match variant.ty().cases()[variant.discriminant()].name() {
                                // wit variable are kebab-case
                                "get-record" => {
                                    if let Some(Value::List(data)) = variant.value() {
                                        let k = data
                                            .iter()
                                            .map(|v| match v {
                                                Value::U8(u) => u,
                                                _ => 0,
                                            })
                                            .collect::<Vec<u8>>();

                                        command = Some(AllCommands::GetRecord { key: k.clone() });

                                        // update the key
                                        key = Some(k);
                                        // key_variant becomes the rhai variable, so it must be snake_case
                                        key_variant = Some("get_record");

                                        // save to rhai scope so it status can be displayed
                                        // we first get the existing value for "get-record",
                                        // then update it with the new value.
                                        // The value is a map of key values, so we can show
                                        // what is pending and what's resolved.
                                        // This also gives us a place to put the results, as
                                        // we can simply look up the matching command and key,
                                        // and update the value with the result.
                                        //
                                        // We will use a rhai map to map the key to the
                                        // Option<value>, where value is the return value
                                        //
                                        // Get the current store scope state
                                        let mut get_record = store
                                            .data_mut()
                                            .scope_mut()
                                            .get_value::<HashMap<String, String>>(
                                                key_variant.unwrap(),
                                            )
                                            .unwrap_or_default();

                                        // update the key with None, as it's pending
                                        get_record.insert(
                                            format!(
                                                "[{}]",
                                                key.clone()
                                                    .unwrap()
                                                    .iter()
                                                    .map(|b| b.to_string())
                                                    .collect::<Vec<String>>()
                                                    .join(", ")
                                            ),
                                            // TODO: Is null good? Should it be "pending"? Or the
                                            // UI just infers pending from null?
                                            "null".to_string(),
                                        );

                                        // update the scope
                                        store.data_mut().update(key_variant.unwrap(), get_record);
                                    }
                                }
                                "get-providers" => {
                                    if let Some(Value::List(data)) = variant.value() {
                                        command = Some(AllCommands::GetProviders {
                                            key: data
                                                .iter()
                                                .map(|v| match v {
                                                    Value::U8(u) => u,
                                                    _ => 0,
                                                })
                                                .collect::<Vec<u8>>(),
                                        });
                                    }
                                }
                                "start-providing" => {
                                    if let Some(Value::List(data)) = variant.value() {
                                        command = Some(AllCommands::StartProviding {
                                            key: data
                                                .iter()
                                                .map(|v| match v {
                                                    Value::U8(u) => u,
                                                    _ => 0,
                                                })
                                                .collect::<Vec<u8>>(),
                                        });
                                    }
                                }
                                _ => {}
                            }
                            match variant.value() {
                                Some(Value::Record(record)) => {
                                    // record is either: string-event, bytes-event, string-list-event
                                    if record.ty() == peer_request_record.clone() {
                                        if let Some(Value::List(request)) = record.field("request")
                                        {
                                            if let Some(Value::String(peer_id)) =
                                                record.field("peer-id")
                                            {
                                                command = Some(AllCommands::PeerRequest {
                                                    request: request
                                                        .iter()
                                                        .map(|v| match v {
                                                            Value::U8(u) => u,
                                                            _ => 0,
                                                        })
                                                        .collect::<Vec<u8>>(),
                                                    peer_id: peer_id.to_string(),
                                                });
                                            }
                                        }
                                    }
                                    if record.ty() == put_record.clone() {
                                        if let Some(Value::List(k)) = record.field("key") {
                                            if let Some(Value::List(value)) = record.field("value")
                                            {
                                                let k = k
                                                    .iter()
                                                    .map(|v| match v {
                                                        Value::U8(u) => u,
                                                        _ => 0,
                                                    })
                                                    .collect::<Vec<u8>>();
                                                key = Some(k.clone());
                                                key_variant = Some("put_record");
                                                command = Some(AllCommands::PutRecord {
                                                    key: k,
                                                    value: value
                                                        .iter()
                                                        .map(|v| match v {
                                                            Value::U8(u) => u,
                                                            _ => 0,
                                                        })
                                                        .collect::<Vec<u8>>(),
                                                });
                                                tracing::info!("Gen'd put-record command");
                                            } else {
                                                tracing::error!("No value found");
                                            }
                                        } else {
                                            tracing::error!("No put_record key found");
                                        }
                                    }
                                }
                                Some(Value::Variant(variant)) => {
                                    //tracing::info!("Got variant value: {:?}", variant.value());
                                    match variant.ty().cases()[variant.discriminant()].name() {
                                        "put" => {
                                            if let Some(Value::List(data)) = variant.value() {
                                                command =
                                                    Some(AllCommands::System(SystemCommand::Put {
                                                        bytes: data
                                                            .iter()
                                                            .map(|v| match v {
                                                                Value::U8(u) => u,
                                                                _ => 0,
                                                            })
                                                            .collect::<Vec<u8>>(),
                                                    }));
                                            }
                                        }
                                        "put-keyed" => {
                                            if let Some(Value::Record(record)) = variant.value() {
                                                if let Some(Value::List(key)) = record.field("key")
                                                {
                                                    if let Some(Value::List(value)) =
                                                        record.field("value")
                                                    {
                                                        command = Some(AllCommands::System(
                                                            SystemCommand::PutKeyed {
                                                                key: key
                                                                    .iter()
                                                                    .map(|v| match v {
                                                                        Value::U8(u) => u,
                                                                        _ => 0,
                                                                    })
                                                                    .collect::<Vec<u8>>(),
                                                                bytes: value
                                                                    .iter()
                                                                    .map(|v| match v {
                                                                        Value::U8(u) => u,
                                                                        _ => 0,
                                                                    })
                                                                    .collect::<Vec<u8>>(),
                                                            },
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                        "get" => {
                                            if let Some(Value::List(data)) = variant.value() {
                                                command =
                                                    Some(AllCommands::System(SystemCommand::Get {
                                                        key: data
                                                            .iter()
                                                            .map(|v| match v {
                                                                Value::U8(u) => u,
                                                                _ => 0,
                                                            })
                                                            .collect::<Vec<u8>>(),
                                                    }));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Some(Value::String(_name)) => {}
                                Some(Value::List(_list)) => {}
                                _ => {}
                            }
                        } else {
                            tracing::error!("Incorrect input type, found {:?}", params[0]);
                        }

                        // take or return early if no command
                        let Some(command) = command else {
                            tracing::error!("No command found");
                            return Ok(());
                        };

                        let commander = {
                            #[cfg(target_arch = "wasm32")]
                            {
                                // unwrap the SendWrapper to get the inner commander
                                let unwrapped = cmdr.deref().clone();
                                // get the inner commander from Rc<RefCell<Option<PeerPiper>>>,
                                // return early if None
                                // commander is peerpiper.commander
                                let lock = unwrapped.borrow();
                                let maybe_peerpiper = lock.as_ref();
                                let Some(peerpiper) = maybe_peerpiper else {
                                    tracing::error!("No peerpiper found yet");
                                    return Ok(());
                                };
                                peerpiper.commander.clone()
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            cmdr.clone()
                        };

                        // a clone of the Scope for this command
                        let command_record = store.data().scope().clone();
                        // Since store.data_mut() and scope_mut() are &mut T, we can't just pass
                        // them to the async block, as they are borrowed mutably.
                        // We need to clone them, and then pass the clone to the async block, or,
                        // we can use a callback channel to send the response to the main thread.

                        platform::spawn(async move {
                            #[cfg(not(target_arch = "wasm32"))]
                            let commander = commander.lock().await;

                            tracing::info!("Ordering command: {:?}", command);

                            match commander.order(command.clone()).await {
                                Ok(return_values) => {
                                    tracing::info!(
                                        "Command {:?} order returned: {:?}",
                                        command,
                                        return_values
                                    );

                                    // same for key
                                    let Some(key) = key else {
                                        tracing::error!(
                                            "No key found, returning early for {:?}",
                                            // show only the first 100 chars of the string
                                            Truncated((command, 100))
                                        );
                                        return;
                                    };

                                    match return_values {
                                        ReturnValues::Data(data) => {
                                            if let Ok(cid) = Cid::try_from(data) {
                                                tracing::info!("Data is cid");
                                                {
                                                    // try to get key_variant from scope if it
                                                    // exists.
                                                    // It is doesn't exist, return early as we have
                                                    // no reference key.
                                                    // For example, this value map could hold
                                                    // all the return values for key_variant "get-record"
                                                    // which could be (bytes, cid string).
                                                    #[cfg(not(target_arch = "wasm32"))]
                                                    {
                                                        let mut scope_binding =
                                                            command_record.lock().unwrap();

                                                        let value_map = match scope_binding
                                                            .get_value_mut::<Dynamic>(
                                                                key_variant.unwrap(),
                                                            ) {
                                                            Some(value_map) => value_map,
                                                            None => {
                                                                // new string string map
                                                                let map: HashMap<String, String> =
                                                                    HashMap::new();

                                                                let rhai_map: Dynamic = map.into();
                                                                scope_binding
                                                                    .set_value(
                                                                        key_variant.unwrap(),
                                                                        rhai_map,
                                                                    )
                                                                    .get_value_mut::<Dynamic>(
                                                                        key_variant.unwrap(),
                                                                    )
                                                                    .unwrap()
                                                            }
                                                        };

                                                        // set the value
                                                        if let Ok(mut map) = value_map.as_map_mut()
                                                        {
                                                            map.insert(
                                                                format!(
                                                                    "[{}]",
                                                                    key.iter()
                                                                        .map(|b| b.to_string())
                                                                        .collect::<Vec<String>>()
                                                                        .join(", ")
                                                                )
                                                                .into(),
                                                                cid.to_string().into(),
                                                            );
                                                        };
                                                    }

                                                    #[cfg(target_arch = "wasm32")]
                                                    {
                                                        let mut cmd_rec =
                                                            command_record.borrow_mut();
                                                        let value_map = match cmd_rec
                                                            .get_value_mut::<Dynamic>(
                                                                key_variant.unwrap(),
                                                            ) {
                                                            Some(value_map) => value_map,
                                                            None => {
                                                                // new string string map
                                                                let map: HashMap<String, String> =
                                                                    HashMap::new();

                                                                let rhai_map: Dynamic = map.into();
                                                                cmd_rec
                                                                    .set_value(
                                                                        key_variant.unwrap(),
                                                                        rhai_map,
                                                                    )
                                                                    .get_value_mut::<Dynamic>(
                                                                        key_variant.unwrap(),
                                                                    )
                                                                    .unwrap()
                                                            }
                                                        };

                                                        // set the value
                                                        if let Ok(mut map) = value_map.as_map_mut()
                                                        {
                                                            map.insert(
                                                                format!(
                                                                    "[{}]",
                                                                    key.iter()
                                                                        .map(|b| b.to_string())
                                                                        .collect::<Vec<String>>()
                                                                        .join(", ")
                                                                )
                                                                .into(),
                                                                cid.to_string().into(),
                                                            );
                                                        };
                                                    }
                                                }

                                                // eagerly load the data from the given cid?
                                                match commander
                                                    .order(AllCommands::System(
                                                        SystemCommand::Get {
                                                            key: cid.to_bytes(),
                                                        },
                                                    ))
                                                    .await
                                                {
                                                    Ok(ReturnValues::Data(data)) => {
                                                        // save the data to the plugin's state
                                                        tracing::info!("Got cid data: {:?}", data);
                                                        // save the data to the plugin's state
                                                        // since it's data, we'll use serde_json to
                                                        // convert to string. rhai can deserde it
                                                        // as red'q
                                                        #[cfg(not(target_arch = "wasm32"))]
                                                        {
                                                            let mut scope_binding =
                                                                command_record.lock().unwrap();

                                                            let value_map = match scope_binding
                                                                .get_value_mut::<Dynamic>(
                                                                "get",
                                                            ) {
                                                                Some(value_map) => value_map,
                                                                None => {
                                                                    // new string string map
                                                                    let map: HashMap<
                                                                        String,
                                                                        String,
                                                                    > = HashMap::new();

                                                                    let rhai_map: Dynamic =
                                                                        map.into();
                                                                    scope_binding
                                                                        .set_value("get", rhai_map)
                                                                        .get_value_mut::<Dynamic>(
                                                                            "get",
                                                                        )
                                                                        .unwrap()
                                                                }
                                                            };

                                                            // TODO: set the value? This could be huge if
                                                            // it's a file...
                                                            if let Ok(mut map) =
                                                                value_map.as_map_mut()
                                                            {
                                                                map.insert(
                                                                    cid.to_string().into(),
                                                                    serde_json::to_string(&data)
                                                                        .unwrap()
                                                                        .into(),
                                                                );
                                                            };
                                                        }

                                                        #[cfg(target_arch = "wasm32")]
                                                        {
                                                            // wasm32 version of above:
                                                            let mut cmd_rec =
                                                                command_record.borrow_mut();
                                                            let value_map = match cmd_rec
                                                                .get_value_mut::<Dynamic>("get")
                                                            {
                                                                Some(value_map) => value_map,
                                                                None => {
                                                                    // new string string map
                                                                    let map: HashMap<
                                                                        String,
                                                                        String,
                                                                    > = HashMap::new();

                                                                    let rhai_map: Dynamic =
                                                                        map.into();
                                                                    cmd_rec
                                                                        .set_value("get", rhai_map)
                                                                        .get_value_mut::<Dynamic>(
                                                                            "get",
                                                                        )
                                                                        .unwrap()
                                                                }
                                                            };

                                                            // set the value? This could be huge if
                                                            // it's a file...
                                                            if let Ok(mut map) =
                                                                value_map.as_map_mut()
                                                            {
                                                                map.insert(
                                                                    cid.to_string().into(),
                                                                    serde_json::to_string(&data)
                                                                        .unwrap()
                                                                        .into(),
                                                                );
                                                            };
                                                        }
                                                    }
                                                    _ => {
                                                        tracing::error!(
                                                            "Error getting data: {:?}",
                                                            cid
                                                        );
                                                    }
                                                }
                                            } else {
                                                tracing::info!("Data is not a cid");
                                            }
                                        }
                                        ReturnValues::ID(cid) => {
                                            tracing::info!("Put cid: {:?}", cid);
                                        }
                                        _ => {}
                                    }

                                    //results[0] = Value::Variant(VariantValue::new(
                                    //    "data",
                                    //    Some(Value::List(List::new(
                                    //        ListType::new(ValueType::U8),
                                    //        data.iter().map(|u| Value::U8(*u)).collect(),
                                    //    )?)),
                                    //));
                                }
                                Err(e) => {
                                    tracing::error!("Error ordering command: {:?}", e);
                                }
                            }
                            //results[0] = result;
                            //Ok(())
                        });

                        // todo: figure out what to return, as it's async
                        // for now, return Value for VariantCase::new("none", None)
                        //results[0] = Value::Bool(false);
                        Ok(())
                    },
                ),
            )
            .unwrap();
    }

    (linker.instantiate(&mut store, &component).unwrap(), store)
}

struct Truncated<T>((T, usize));

impl<T: std::fmt::Debug> std::fmt::Debug for Truncated<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug_str = format!("{:?}", self.0 .0);
        if debug_str.len() > self.0 .1 {
            let limit = (self.0 .1) - 3;
            write!(f, "{}...", &debug_str[..limit])
        } else {
            write!(f, "{}", debug_str)
        }
    }
}

#[cfg(test)]
mod tests {

    // test to verify that a HashMap turns into a rhai Dynamic map
    #[test]
    fn test_map() {
        //use rdx::layer::rhai::Dynamic;
        use std::collections::HashMap;

        let mut scope = rdx::layer::rhai::Scope::new();

        let mut map = HashMap::new();
        map.insert("key1", None);
        map.insert("key2", Some("value2"));

        scope.set_value("get_record", map);

        let map = scope
            .get_value_ref::<HashMap<&str, Option<&str>>>("get_record")
            .unwrap();

        eprintln!("Map: {:?}", map);

        // update the scope key1 to have a value
        let mut map = map.clone();
        map.insert("key1", Some("value1"));

        assert_eq!(*map.get("key1").unwrap(), Some("value1"));
    }
}
