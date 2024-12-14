//! multinode specific layers
use core::task::{Context, Poll};
use std::{
    any::Any,
    collections::HashMap,
    future::Future,
    pin::{pin, Pin},
    sync::{Arc, Mutex},
};

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
use std::ops::DerefMut;

//#[cfg(not(target_arch = "wasm32"))]
//use tokio::sync::Mutex as AsyncMutex;

//use crate::app::platform::peerpiper::PeerPiper;

use super::CommanderCounter;

/// Use wasm_component_layer to intanitate a plugin and some state data
pub struct LayerPlugin<T: Inner + Send + Sync> {
    #[cfg(target_arch = "wasm32")]
    pub(crate) store: SendWrapper<Store<T, runtime_layer::Engine>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) store: Store<T, runtime_layer::Engine>,
    raw_instance: Instance,
}

impl<T: Inner + Send + Sync + 'static> LayerPlugin<T> {
    /// Creates a new with the given wallet layer as a dependency
    pub fn new(
        bytes: &[u8],
        data: T,
        wallet_layer: Option<Arc<Mutex<dyn Instantiator<T>>>>,
        commander: Option<CommanderCounter>,
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

pub fn instantiate_instance<T: Inner + Send + Sync + 'static>(
    bytes: &[u8],
    data: T,
    wallet_layer: Option<Arc<Mutex<dyn Instantiator<T>>>>,
    commander: Option<CommanderCounter>,
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

    // "log" function using tracing
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
    //  text(string-event),
    //  bytes(bytes-event),
    //  string-list(string-list-event)
    //}

    let text_variant = VariantCase::new(
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
                vec![
                    ("name", ValueType::String),
                    ("value", ValueType::List(ListType::new(ValueType::U8))),
                ],
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

    let event =
        VariantType::new(None, vec![text_variant, bytes_variant, string_list_variant]).unwrap();

    host_interface
        .define_func(
            "emit",
            Func::new(
                &mut store,
                FuncType::new([ValueType::Variant(event)], []),
                move |mut store, params, _results| {
                    if let Value::Variant(variant) = &params[0] {
                        tracing::debug!("Emitting event: {:?}", variant);
                        if let Some(Value::Record(record)) = variant.value() {
                            // record is either: string-event, bytes-event, string-list-event
                            match record.field("value") {
                                Some(Value::String(value)) => {
                                    if let Some(Value::String(name)) = record.field("name") {
                                        store.data_mut().update(&name, &*value);
                                    }
                                }
                                Some(Value::List(list)) => {
                                    tracing::debug!("Emitting list: {:?}", list);
                                    let vals = list
                                        .iter()
                                        .map(|v| {
                                            tracing::debug!("Val v is: {:?}", v);
                                            match v {
                                                Value::U8(u) => Dynamic::from(u),
                                                Value::String(s) => {
                                                    let s: String = s.to_string();
                                                    tracing::debug!("String s is: {:?}", s);
                                                    Dynamic::from(s)
                                                }
                                                Value::List(lis) => lis
                                                    .iter()
                                                    .map(|v| match v {
                                                        Value::U8(u) => Dynamic::from(u),
                                                        Value::String(s) => {
                                                            Dynamic::from(s.to_string())
                                                        }
                                                        _ => Dynamic::from("Unsupported type"),
                                                    })
                                                    .collect::<Vec<_>>()
                                                    .into(),
                                                _ => Dynamic::from("Unsupported type"),
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    tracing::debug!("Emitting list: {:?}", vals);
                                    if let Some(Value::String(name)) = record.field("name") {
                                        tracing::debug!(
                                            "Emitting name value pair: {:?} {:?}",
                                            name,
                                            vals
                                        );
                                        store.data_mut().update(&name, &*vals);
                                    }
                                }
                                _ => {}
                            }
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
        let ok = ValueType::List(ListType::new(ValueType::U8));
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
                ("mk", ValueType::List(ListType::new(ValueType::U8))),
                ("data", ValueType::List(ListType::new(ValueType::U8))),
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
    }

    // if peerpiper_layer is Some, we can also add the peerpiper functions
    // We can wire peerpiper AllCommands as host functions.
    // For now, let's do get/put for Blockstore actions
    if let Some(_cmdr) = commander {
        // Let's provide both get and put functions for the blockstore
        // these orders are async, but we get a root CID as a result,
        // which we always save to the plugin's state StringStore

        // actually you just need to save the key:value bytes to the rhai Scope,
        // and the periodic save() will take care of the rest.

        // put: func(data: list<u8>) -> string;
        //let put_args = FuncType::new(
        //    [ValueType::List(ListType::new(ValueType::U8))],
        //    [ValueType::String],
        //);
    }

    (linker.instantiate(&mut store, &component).unwrap(), store)
}
