#[allow(dead_code)]
pub mod host {
    #[allow(dead_code)]
    pub mod component {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            /// Event type where value is a string.
            #[derive(Clone)]
            pub struct StringEvent {
                /// The variable name
                pub name: _rt::String,
                pub value: _rt::String,
            }
            impl ::core::fmt::Debug for StringEvent {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("StringEvent")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
            /// Event type where value is a list<u8>.
            #[derive(Clone)]
            pub struct BytesEvent {
                /// The variable name
                pub name: _rt::String,
                pub value: _rt::Vec<u8>,
            }
            impl ::core::fmt::Debug for BytesEvent {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("BytesEvent")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
            /// Event where there is a list of strings
            #[derive(Clone)]
            pub struct StringListEvent {
                /// The variable name
                pub name: _rt::String,
                pub value: _rt::Vec<_rt::String>,
            }
            impl ::core::fmt::Debug for StringListEvent {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("StringListEvent")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
            /// Event is a variant of string and bytes events.
            #[derive(Clone)]
            pub enum Event {
                /// Save all the rhai Scope to disk.
                Save,
                /// Add this string key value pair to the scope.
                Text(StringEvent),
                Bytes(BytesEvent),
                StringList(StringListEvent),
            }
            impl ::core::fmt::Debug for Event {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        Event::Save => f.debug_tuple("Event::Save").finish(),
                        Event::Text(e) => f.debug_tuple("Event::Text").field(e).finish(),
                        Event::Bytes(e) => {
                            f.debug_tuple("Event::Bytes").field(e).finish()
                        }
                        Event::StringList(e) => {
                            f.debug_tuple("Event::StringList").field(e).finish()
                        }
                    }
                }
            }
            /// Key arguments for getting a Multikey
            #[derive(Clone)]
            pub struct KeyArgs {
                /// The key
                pub key: _rt::String,
                /// The codec
                pub codec: _rt::String,
                /// THreshold
                pub threshold: u8,
                /// Limit
                pub limit: u8,
            }
            impl ::core::fmt::Debug for KeyArgs {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("KeyArgs")
                        .field("key", &self.key)
                        .field("codec", &self.codec)
                        .field("threshold", &self.threshold)
                        .field("limit", &self.limit)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub struct ProveArgs {
                /// The Multikey
                pub mk: _rt::Vec<u8>,
                /// The data
                pub data: _rt::Vec<u8>,
            }
            impl ::core::fmt::Debug for ProveArgs {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("ProveArgs")
                        .field("mk", &self.mk)
                        .field("data", &self.data)
                        .finish()
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod peerpiper {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            /// Publsih data to a topic
            #[derive(Clone)]
            pub struct Publish {
                /// The topic
                pub topic: _rt::String,
                /// The data
                pub data: _rt::Vec<u8>,
            }
            impl ::core::fmt::Debug for Publish {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Publish")
                        .field("topic", &self.topic)
                        .field("data", &self.data)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub struct PutKeyed {
                /// The key
                pub key: _rt::Vec<u8>,
                /// The value
                pub value: _rt::Vec<u8>,
            }
            impl ::core::fmt::Debug for PutKeyed {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("PutKeyed")
                        .field("key", &self.key)
                        .field("value", &self.value)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub enum SystemCommand {
                /// Put bytes on the local disk
                Put(_rt::Vec<u8>),
                /// Puts Keyed bytes into the local disk
                PutKeyed(PutKeyed),
                /// Get bytes from the local disk
                Get(_rt::Vec<u8>),
            }
            impl ::core::fmt::Debug for SystemCommand {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        SystemCommand::Put(e) => {
                            f.debug_tuple("SystemCommand::Put").field(e).finish()
                        }
                        SystemCommand::PutKeyed(e) => {
                            f.debug_tuple("SystemCommand::PutKeyed").field(e).finish()
                        }
                        SystemCommand::Get(e) => {
                            f.debug_tuple("SystemCommand::Get").field(e).finish()
                        }
                    }
                }
            }
            /// Make a Rwquest from a Peer
            /// The request is encoded as a list of bytes
            #[derive(Clone)]
            pub struct PeerRequest {
                /// The request
                pub request: _rt::Vec<u8>,
                /// The peer id
                pub peer_id: _rt::String,
            }
            impl ::core::fmt::Debug for PeerRequest {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("PeerRequest")
                        .field("request", &self.request)
                        .field("peer-id", &self.peer_id)
                        .finish()
                }
            }
            /// Put bytes in the DHT
            #[derive(Clone)]
            pub struct PutRecord {
                /// The key
                pub key: _rt::Vec<u8>,
                /// The value
                pub value: _rt::Vec<u8>,
            }
            impl ::core::fmt::Debug for PutRecord {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("PutRecord")
                        .field("key", &self.key)
                        .field("value", &self.value)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub enum AllCommands {
                /// Publish data to a topic
                Publish(Publish),
                /// Subscribe to a topic
                Subscribe(_rt::String),
                /// Unsubscribe from a topic
                Unsubscribe(_rt::String),
                /// System commands are a subset of [AllCommands] that do not go to the network, but come
                /// from componets to direct the system to do something, like save bytes to a file.
                System(SystemCommand),
                /// Please peer, do something with this data and give me a response
                PeerRequest(PeerRequest),
                /// Puts a Record on the DHT, and optionally provides the data for Pinning
                PutRecord(PutRecord),
                /// Gets a Record from the DHT
                GetRecord(_rt::Vec<u8>),
                /// Gets the Providers of a Record on the DHT
                GetProviders(_rt::Vec<u8>),
                /// Start Providing a Record on the DHT
                StartProviding(_rt::Vec<u8>),
            }
            impl ::core::fmt::Debug for AllCommands {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        AllCommands::Publish(e) => {
                            f.debug_tuple("AllCommands::Publish").field(e).finish()
                        }
                        AllCommands::Subscribe(e) => {
                            f.debug_tuple("AllCommands::Subscribe").field(e).finish()
                        }
                        AllCommands::Unsubscribe(e) => {
                            f.debug_tuple("AllCommands::Unsubscribe").field(e).finish()
                        }
                        AllCommands::System(e) => {
                            f.debug_tuple("AllCommands::System").field(e).finish()
                        }
                        AllCommands::PeerRequest(e) => {
                            f.debug_tuple("AllCommands::PeerRequest").field(e).finish()
                        }
                        AllCommands::PutRecord(e) => {
                            f.debug_tuple("AllCommands::PutRecord").field(e).finish()
                        }
                        AllCommands::GetRecord(e) => {
                            f.debug_tuple("AllCommands::GetRecord").field(e).finish()
                        }
                        AllCommands::GetProviders(e) => {
                            f.debug_tuple("AllCommands::GetProviders").field(e).finish()
                        }
                        AllCommands::StartProviding(e) => {
                            f.debug_tuple("AllCommands::StartProviding")
                                .field(e)
                                .finish()
                        }
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod host {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Event = super::super::super::host::component::types::Event;
            pub type KeyArgs = super::super::super::host::component::types::KeyArgs;
            pub type ProveArgs = super::super::super::host::component::types::ProveArgs;
            pub type AllCommands = super::super::super::host::component::peerpiper::AllCommands;
            /// get-mk Error type
            #[derive(Clone)]
            pub enum MkError {
                /// The error message
                InvalidCodec(_rt::String),
                /// Wallet uninitialized
                WalletUninitialized,
                /// Mulitkey Error
                MultikeyError(_rt::String),
                /// Key not found
                KeyNotFound(_rt::String),
            }
            impl ::core::fmt::Debug for MkError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        MkError::InvalidCodec(e) => {
                            f.debug_tuple("MkError::InvalidCodec").field(e).finish()
                        }
                        MkError::WalletUninitialized => {
                            f.debug_tuple("MkError::WalletUninitialized").finish()
                        }
                        MkError::MultikeyError(e) => {
                            f.debug_tuple("MkError::MultikeyError").field(e).finish()
                        }
                        MkError::KeyNotFound(e) => {
                            f.debug_tuple("MkError::KeyNotFound").field(e).finish()
                        }
                    }
                }
            }
            impl ::core::fmt::Display for MkError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl std::error::Error for MkError {}
            #[allow(unused_unsafe, clippy::all)]
            /// emit an event.
            pub fn emit(evt: &Event) {
                unsafe {
                    let mut cleanup_list = _rt::Vec::new();
                    use super::super::super::host::component::types::Event as V10;
                    let (result11_0, result11_1, result11_2, result11_3, result11_4) = match evt {
                        V10::Save => {
                            (
                                0i32,
                                ::core::ptr::null_mut(),
                                0usize,
                                ::core::ptr::null_mut(),
                                0usize,
                            )
                        }
                        V10::Text(e) => {
                            let super::super::super::host::component::types::StringEvent {
                                name: name0,
                                value: value0,
                            } = e;
                            let vec1 = name0;
                            let ptr1 = vec1.as_ptr().cast::<u8>();
                            let len1 = vec1.len();
                            let vec2 = value0;
                            let ptr2 = vec2.as_ptr().cast::<u8>();
                            let len2 = vec2.len();
                            (1i32, ptr1.cast_mut(), len1, ptr2.cast_mut(), len2)
                        }
                        V10::Bytes(e) => {
                            let super::super::super::host::component::types::BytesEvent {
                                name: name3,
                                value: value3,
                            } = e;
                            let vec4 = name3;
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            let vec5 = value3;
                            let ptr5 = vec5.as_ptr().cast::<u8>();
                            let len5 = vec5.len();
                            (2i32, ptr4.cast_mut(), len4, ptr5.cast_mut(), len5)
                        }
                        V10::StringList(e) => {
                            let super::super::super::host::component::types::StringListEvent {
                                name: name6,
                                value: value6,
                            } = e;
                            let vec7 = name6;
                            let ptr7 = vec7.as_ptr().cast::<u8>();
                            let len7 = vec7.len();
                            let vec9 = value6;
                            let len9 = vec9.len();
                            let layout9 = _rt::alloc::Layout::from_size_align_unchecked(
                                vec9.len() * 8,
                                4,
                            );
                            let result9 = if layout9.size() != 0 {
                                let ptr = _rt::alloc::alloc(layout9).cast::<u8>();
                                if ptr.is_null() {
                                    _rt::alloc::handle_alloc_error(layout9);
                                }
                                ptr
                            } else {
                                ::core::ptr::null_mut()
                            };
                            for (i, e) in vec9.into_iter().enumerate() {
                                let base = result9.add(i * 8);
                                {
                                    let vec8 = e;
                                    let ptr8 = vec8.as_ptr().cast::<u8>();
                                    let len8 = vec8.len();
                                    *base.add(4).cast::<usize>() = len8;
                                    *base.add(0).cast::<*mut u8>() = ptr8.cast_mut();
                                }
                            }
                            cleanup_list.extend_from_slice(&[(result9, layout9)]);
                            (3i32, ptr7.cast_mut(), len7, result9, len9)
                        }
                    };
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "host:component/host")]
                    extern "C" {
                        #[link_name = "emit"]
                        fn wit_import(
                            _: i32,
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8, _: usize) {
                        unreachable!()
                    }
                    wit_import(
                        result11_0,
                        result11_1,
                        result11_2,
                        result11_3,
                        result11_4,
                    );
                    for (ptr, layout) in cleanup_list {
                        if layout.size() != 0 {
                            _rt::alloc::dealloc(ptr.cast(), layout);
                        }
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// log a message.
            pub fn log(msg: &str) {
                unsafe {
                    let vec0 = msg;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "host:component/host")]
                    extern "C" {
                        #[link_name = "log"]
                        fn wit_import(_: *mut u8, _: usize);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8, _: usize) {
                        unreachable!()
                    }
                    wit_import(ptr0.cast_mut(), len0);
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// get a random byte
            pub fn random_byte() -> u8 {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "host:component/host")]
                    extern "C" {
                        #[link_name = "random-byte"]
                        fn wit_import() -> i32;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i32 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    ret as u8
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Gets the Multikey
            pub fn get_mk(args: &KeyArgs) -> Result<_rt::Vec<u8>, MkError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let super::super::super::host::component::types::KeyArgs {
                        key: key0,
                        codec: codec0,
                        threshold: threshold0,
                        limit: limit0,
                    } = args;
                    let vec1 = key0;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    let vec2 = codec0;
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    let ptr3 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "host:component/host")]
                    extern "C" {
                        #[link_name = "get-mk"]
                        fn wit_import(
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: i32,
                            _: i32,
                            _: *mut u8,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                        _: usize,
                        _: i32,
                        _: i32,
                        _: *mut u8,
                    ) {
                        unreachable!()
                    }
                    wit_import(
                        ptr1.cast_mut(),
                        len1,
                        ptr2.cast_mut(),
                        len2,
                        _rt::as_i32(threshold0),
                        _rt::as_i32(limit0),
                        ptr3,
                    );
                    let l4 = i32::from(*ptr3.add(0).cast::<u8>());
                    match l4 {
                        0 => {
                            let e = {
                                let l5 = *ptr3.add(4).cast::<*mut u8>();
                                let l6 = *ptr3.add(8).cast::<usize>();
                                let len7 = l6;
                                _rt::Vec::from_raw_parts(l5.cast(), len7, len7)
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l8 = i32::from(*ptr3.add(4).cast::<u8>());
                                let v18 = match l8 {
                                    0 => {
                                        let e18 = {
                                            let l9 = *ptr3.add(8).cast::<*mut u8>();
                                            let l10 = *ptr3.add(12).cast::<usize>();
                                            let len11 = l10;
                                            let bytes11 = _rt::Vec::from_raw_parts(
                                                l9.cast(),
                                                len11,
                                                len11,
                                            );
                                            _rt::string_lift(bytes11)
                                        };
                                        MkError::InvalidCodec(e18)
                                    }
                                    1 => MkError::WalletUninitialized,
                                    2 => {
                                        let e18 = {
                                            let l12 = *ptr3.add(8).cast::<*mut u8>();
                                            let l13 = *ptr3.add(12).cast::<usize>();
                                            let len14 = l13;
                                            let bytes14 = _rt::Vec::from_raw_parts(
                                                l12.cast(),
                                                len14,
                                                len14,
                                            );
                                            _rt::string_lift(bytes14)
                                        };
                                        MkError::MultikeyError(e18)
                                    }
                                    n => {
                                        debug_assert_eq!(n, 3, "invalid enum discriminant");
                                        let e18 = {
                                            let l15 = *ptr3.add(8).cast::<*mut u8>();
                                            let l16 = *ptr3.add(12).cast::<usize>();
                                            let len17 = l16;
                                            let bytes17 = _rt::Vec::from_raw_parts(
                                                l15.cast(),
                                                len17,
                                                len17,
                                            );
                                            _rt::string_lift(bytes17)
                                        };
                                        MkError::KeyNotFound(e18)
                                    }
                                };
                                v18
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Proves the data for the given Multikey.
            pub fn prove(args: &ProveArgs) -> Result<_rt::Vec<u8>, MkError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let super::super::super::host::component::types::ProveArgs {
                        mk: mk0,
                        data: data0,
                    } = args;
                    let vec1 = mk0;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    let vec2 = data0;
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    let ptr3 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "host:component/host")]
                    extern "C" {
                        #[link_name = "prove"]
                        fn wit_import(
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                    ) {
                        unreachable!()
                    }
                    wit_import(ptr1.cast_mut(), len1, ptr2.cast_mut(), len2, ptr3);
                    let l4 = i32::from(*ptr3.add(0).cast::<u8>());
                    match l4 {
                        0 => {
                            let e = {
                                let l5 = *ptr3.add(4).cast::<*mut u8>();
                                let l6 = *ptr3.add(8).cast::<usize>();
                                let len7 = l6;
                                _rt::Vec::from_raw_parts(l5.cast(), len7, len7)
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l8 = i32::from(*ptr3.add(4).cast::<u8>());
                                let v18 = match l8 {
                                    0 => {
                                        let e18 = {
                                            let l9 = *ptr3.add(8).cast::<*mut u8>();
                                            let l10 = *ptr3.add(12).cast::<usize>();
                                            let len11 = l10;
                                            let bytes11 = _rt::Vec::from_raw_parts(
                                                l9.cast(),
                                                len11,
                                                len11,
                                            );
                                            _rt::string_lift(bytes11)
                                        };
                                        MkError::InvalidCodec(e18)
                                    }
                                    1 => MkError::WalletUninitialized,
                                    2 => {
                                        let e18 = {
                                            let l12 = *ptr3.add(8).cast::<*mut u8>();
                                            let l13 = *ptr3.add(12).cast::<usize>();
                                            let len14 = l13;
                                            let bytes14 = _rt::Vec::from_raw_parts(
                                                l12.cast(),
                                                len14,
                                                len14,
                                            );
                                            _rt::string_lift(bytes14)
                                        };
                                        MkError::MultikeyError(e18)
                                    }
                                    n => {
                                        debug_assert_eq!(n, 3, "invalid enum discriminant");
                                        let e18 = {
                                            let l15 = *ptr3.add(8).cast::<*mut u8>();
                                            let l16 = *ptr3.add(12).cast::<usize>();
                                            let len17 = l16;
                                            let bytes17 = _rt::Vec::from_raw_parts(
                                                l15.cast(),
                                                len17,
                                                len17,
                                            );
                                            _rt::string_lift(bytes17)
                                        };
                                        MkError::KeyNotFound(e18)
                                    }
                                };
                                v18
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Order PeerPiper to do something.
            pub fn order(order: &AllCommands) {
                unsafe {
                    use super::super::super::host::component::peerpiper::AllCommands as V21;
                    let (
                        result22_0,
                        result22_1,
                        result22_2,
                        result22_3,
                        result22_4,
                        result22_5,
                    ) = match order {
                        V21::Publish(e) => {
                            let super::super::super::host::component::peerpiper::Publish {
                                topic: topic0,
                                data: data0,
                            } = e;
                            let vec1 = topic0;
                            let ptr1 = vec1.as_ptr().cast::<u8>();
                            let len1 = vec1.len();
                            let vec2 = data0;
                            let ptr2 = vec2.as_ptr().cast::<u8>();
                            let len2 = vec2.len();
                            (
                                0i32,
                                ptr1.cast_mut(),
                                len1 as *mut u8,
                                ptr2.cast_mut(),
                                len2 as *mut u8,
                                0usize,
                            )
                        }
                        V21::Subscribe(e) => {
                            let vec3 = e;
                            let ptr3 = vec3.as_ptr().cast::<u8>();
                            let len3 = vec3.len();
                            (
                                1i32,
                                ptr3.cast_mut(),
                                len3 as *mut u8,
                                ::core::ptr::null_mut(),
                                ::core::ptr::null_mut(),
                                0usize,
                            )
                        }
                        V21::Unsubscribe(e) => {
                            let vec4 = e;
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            (
                                2i32,
                                ptr4.cast_mut(),
                                len4 as *mut u8,
                                ::core::ptr::null_mut(),
                                ::core::ptr::null_mut(),
                                0usize,
                            )
                        }
                        V21::System(e) => {
                            use super::super::super::host::component::peerpiper::SystemCommand as V10;
                            let (
                                result11_0,
                                result11_1,
                                result11_2,
                                result11_3,
                                result11_4,
                            ) = match e {
                                V10::Put(e) => {
                                    let vec5 = e;
                                    let ptr5 = vec5.as_ptr().cast::<u8>();
                                    let len5 = vec5.len();
                                    (
                                        0i32,
                                        ptr5.cast_mut(),
                                        len5,
                                        ::core::ptr::null_mut(),
                                        0usize,
                                    )
                                }
                                V10::PutKeyed(e) => {
                                    let super::super::super::host::component::peerpiper::PutKeyed {
                                        key: key6,
                                        value: value6,
                                    } = e;
                                    let vec7 = key6;
                                    let ptr7 = vec7.as_ptr().cast::<u8>();
                                    let len7 = vec7.len();
                                    let vec8 = value6;
                                    let ptr8 = vec8.as_ptr().cast::<u8>();
                                    let len8 = vec8.len();
                                    (1i32, ptr7.cast_mut(), len7, ptr8.cast_mut(), len8)
                                }
                                V10::Get(e) => {
                                    let vec9 = e;
                                    let ptr9 = vec9.as_ptr().cast::<u8>();
                                    let len9 = vec9.len();
                                    (
                                        2i32,
                                        ptr9.cast_mut(),
                                        len9,
                                        ::core::ptr::null_mut(),
                                        0usize,
                                    )
                                }
                            };
                            (
                                3i32,
                                result11_0 as *mut u8,
                                result11_1,
                                result11_2 as *mut u8,
                                result11_3,
                                result11_4,
                            )
                        }
                        V21::PeerRequest(e) => {
                            let super::super::super::host::component::peerpiper::PeerRequest {
                                request: request12,
                                peer_id: peer_id12,
                            } = e;
                            let vec13 = request12;
                            let ptr13 = vec13.as_ptr().cast::<u8>();
                            let len13 = vec13.len();
                            let vec14 = peer_id12;
                            let ptr14 = vec14.as_ptr().cast::<u8>();
                            let len14 = vec14.len();
                            (
                                4i32,
                                ptr13.cast_mut(),
                                len13 as *mut u8,
                                ptr14.cast_mut(),
                                len14 as *mut u8,
                                0usize,
                            )
                        }
                        V21::PutRecord(e) => {
                            let super::super::super::host::component::peerpiper::PutRecord {
                                key: key15,
                                value: value15,
                            } = e;
                            let vec16 = key15;
                            let ptr16 = vec16.as_ptr().cast::<u8>();
                            let len16 = vec16.len();
                            let vec17 = value15;
                            let ptr17 = vec17.as_ptr().cast::<u8>();
                            let len17 = vec17.len();
                            (
                                5i32,
                                ptr16.cast_mut(),
                                len16 as *mut u8,
                                ptr17.cast_mut(),
                                len17 as *mut u8,
                                0usize,
                            )
                        }
                        V21::GetRecord(e) => {
                            let vec18 = e;
                            let ptr18 = vec18.as_ptr().cast::<u8>();
                            let len18 = vec18.len();
                            (
                                6i32,
                                ptr18.cast_mut(),
                                len18 as *mut u8,
                                ::core::ptr::null_mut(),
                                ::core::ptr::null_mut(),
                                0usize,
                            )
                        }
                        V21::GetProviders(e) => {
                            let vec19 = e;
                            let ptr19 = vec19.as_ptr().cast::<u8>();
                            let len19 = vec19.len();
                            (
                                7i32,
                                ptr19.cast_mut(),
                                len19 as *mut u8,
                                ::core::ptr::null_mut(),
                                ::core::ptr::null_mut(),
                                0usize,
                            )
                        }
                        V21::StartProviding(e) => {
                            let vec20 = e;
                            let ptr20 = vec20.as_ptr().cast::<u8>();
                            let len20 = vec20.len();
                            (
                                8i32,
                                ptr20.cast_mut(),
                                len20 as *mut u8,
                                ::core::ptr::null_mut(),
                                ::core::ptr::null_mut(),
                                0usize,
                            )
                        }
                    };
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "host:component/host")]
                    extern "C" {
                        #[link_name = "order"]
                        fn wit_import(
                            _: i32,
                            _: *mut u8,
                            _: *mut u8,
                            _: *mut u8,
                            _: *mut u8,
                            _: usize,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(
                        _: i32,
                        _: *mut u8,
                        _: *mut u8,
                        _: *mut u8,
                        _: *mut u8,
                        _: usize,
                    ) {
                        unreachable!()
                    }
                    wit_import(
                        result22_0,
                        result22_1,
                        result22_2,
                        result22_3,
                        result22_4,
                        result22_5,
                    );
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// -> return-values;
            /// Gets the current rhai scope from the host, if available.
            pub fn get_scope() -> _rt::String {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "host:component/host")]
                    extern "C" {
                        #[link_name = "get-scope"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<*mut u8>();
                    let l2 = *ptr0.add(4).cast::<usize>();
                    let len3 = l2;
                    let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
                    _rt::string_lift(bytes3)
                }
            }
        }
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod component {
        #[allow(dead_code)]
        pub mod plugin {
            #[allow(dead_code, clippy::all)]
            pub mod run {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_load_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::load();
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec2 = (result0.into_bytes()).into_boxed_slice();
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    ::core::mem::forget(vec2);
                    *ptr1.add(4).cast::<usize>() = len2;
                    *ptr1.add(0).cast::<*mut u8>() = ptr2.cast_mut();
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_load<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    _rt::cabi_dealloc(l0, l1, 1);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_init_cabi<T: Guest>() {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    T::init();
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_register_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::register();
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec3 = result0;
                    let len3 = vec3.len();
                    let layout3 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec3.len() * 8,
                        4,
                    );
                    let result3 = if layout3.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout3).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout3);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec3.into_iter().enumerate() {
                        let base = result3.add(i * 8);
                        {
                            let vec2 = (e.into_bytes()).into_boxed_slice();
                            let ptr2 = vec2.as_ptr().cast::<u8>();
                            let len2 = vec2.len();
                            ::core::mem::forget(vec2);
                            *base.add(4).cast::<usize>() = len2;
                            *base.add(0).cast::<*mut u8>() = ptr2.cast_mut();
                        }
                    }
                    *ptr1.add(4).cast::<usize>() = len3;
                    *ptr1.add(0).cast::<*mut u8>() = result3;
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_register<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    let base4 = l0;
                    let len4 = l1;
                    for i in 0..len4 {
                        let base = base4.add(i * 8);
                        {
                            let l2 = *base.add(0).cast::<*mut u8>();
                            let l3 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l2, l3, 1);
                        }
                    }
                    _rt::cabi_dealloc(base4, len4 * 8, 4);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_search_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let result1 = T::search(_rt::string_lift(bytes0));
                    let ptr2 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result1 {
                        Ok(_) => {
                            *ptr2.add(0).cast::<u8>() = (0i32) as u8;
                        }
                        Err(e) => {
                            *ptr2.add(0).cast::<u8>() = (1i32) as u8;
                            let vec3 = (e.into_bytes()).into_boxed_slice();
                            let ptr3 = vec3.as_ptr().cast::<u8>();
                            let len3 = vec3.len();
                            ::core::mem::forget(vec3);
                            *ptr2.add(8).cast::<usize>() = len3;
                            *ptr2.add(4).cast::<*mut u8>() = ptr3.cast_mut();
                        }
                    };
                    ptr2
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_search<T: Guest>(arg0: *mut u8) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {}
                        _ => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l1, l2, 1);
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_add_to_contacts_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let len1 = arg3;
                    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
                    let result2 = T::add_to_contacts(
                        _rt::string_lift(bytes0),
                        _rt::string_lift(bytes1),
                    );
                    let ptr3 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result2 {
                        Ok(_) => {
                            *ptr3.add(0).cast::<u8>() = (0i32) as u8;
                        }
                        Err(e) => {
                            *ptr3.add(0).cast::<u8>() = (1i32) as u8;
                            let vec4 = (e.into_bytes()).into_boxed_slice();
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            ::core::mem::forget(vec4);
                            *ptr3.add(8).cast::<usize>() = len4;
                            *ptr3.add(4).cast::<*mut u8>() = ptr4.cast_mut();
                        }
                    };
                    ptr3
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_add_to_contacts<T: Guest>(arg0: *mut u8) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {}
                        _ => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l1, l2, 1);
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_contacts_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::contacts();
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec4 = result0;
                    let len4 = vec4.len();
                    let layout4 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec4.len() * 8,
                        4,
                    );
                    let result4 = if layout4.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout4).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout4);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec4.into_iter().enumerate() {
                        let base = result4.add(i * 8);
                        {
                            let vec3 = e;
                            let len3 = vec3.len();
                            let layout3 = _rt::alloc::Layout::from_size_align_unchecked(
                                vec3.len() * 8,
                                4,
                            );
                            let result3 = if layout3.size() != 0 {
                                let ptr = _rt::alloc::alloc(layout3).cast::<u8>();
                                if ptr.is_null() {
                                    _rt::alloc::handle_alloc_error(layout3);
                                }
                                ptr
                            } else {
                                ::core::ptr::null_mut()
                            };
                            for (i, e) in vec3.into_iter().enumerate() {
                                let base = result3.add(i * 8);
                                {
                                    let vec2 = (e.into_bytes()).into_boxed_slice();
                                    let ptr2 = vec2.as_ptr().cast::<u8>();
                                    let len2 = vec2.len();
                                    ::core::mem::forget(vec2);
                                    *base.add(4).cast::<usize>() = len2;
                                    *base.add(0).cast::<*mut u8>() = ptr2.cast_mut();
                                }
                            }
                            *base.add(4).cast::<usize>() = len3;
                            *base.add(0).cast::<*mut u8>() = result3;
                        }
                    }
                    *ptr1.add(4).cast::<usize>() = len4;
                    *ptr1.add(0).cast::<*mut u8>() = result4;
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_contacts<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    let base7 = l0;
                    let len7 = l1;
                    for i in 0..len7 {
                        let base = base7.add(i * 8);
                        {
                            let l2 = *base.add(0).cast::<*mut u8>();
                            let l3 = *base.add(4).cast::<usize>();
                            let base6 = l2;
                            let len6 = l3;
                            for i in 0..len6 {
                                let base = base6.add(i * 8);
                                {
                                    let l4 = *base.add(0).cast::<*mut u8>();
                                    let l5 = *base.add(4).cast::<usize>();
                                    _rt::cabi_dealloc(l4, l5, 1);
                                }
                            }
                            _rt::cabi_dealloc(base6, len6 * 8, 4);
                        }
                    }
                    _rt::cabi_dealloc(base7, len7 * 8, 4);
                }
                pub trait Guest {
                    /// loads just the RDX for rendering
                    fn load() -> _rt::String;
                    /// initialize the state of the component
                    fn init();
                    /// Register wasm functions to be bound to Rhai
                    /// Returns a list of func names that are to be bound
                    fn register() -> _rt::Vec<_rt::String>;
                    /// search for a peer by VLAD
                    fn search(vlad: _rt::String) -> Result<(), _rt::String>;
                    /// Adds this vlad's nickname to contacts
                    fn add_to_contacts(
                        vlad: _rt::String,
                        nickname: _rt::String,
                    ) -> Result<(), _rt::String>;
                    /// Contacts that are in our Book.
                    /// [vlad, nickname, notes]
                    fn contacts() -> _rt::Vec<_rt::Vec<_rt::String>>;
                }
                #[doc(hidden)]
                macro_rules! __export_component_plugin_run_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name = "component:plugin/run#load"]
                        unsafe extern "C" fn export_load() -> * mut u8 {
                        $($path_to_types)*:: _export_load_cabi::<$ty > () } #[export_name
                        = "cabi_post_component:plugin/run#load"] unsafe extern "C" fn
                        _post_return_load(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_load::<$ty > (arg0) } #[export_name =
                        "component:plugin/run#init"] unsafe extern "C" fn export_init() {
                        $($path_to_types)*:: _export_init_cabi::<$ty > () } #[export_name
                        = "component:plugin/run#register"] unsafe extern "C" fn
                        export_register() -> * mut u8 { $($path_to_types)*::
                        _export_register_cabi::<$ty > () } #[export_name =
                        "cabi_post_component:plugin/run#register"] unsafe extern "C" fn
                        _post_return_register(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_register::<$ty > (arg0) } #[export_name =
                        "component:plugin/run#search"] unsafe extern "C" fn
                        export_search(arg0 : * mut u8, arg1 : usize,) -> * mut u8 {
                        $($path_to_types)*:: _export_search_cabi::<$ty > (arg0, arg1) }
                        #[export_name = "cabi_post_component:plugin/run#search"] unsafe
                        extern "C" fn _post_return_search(arg0 : * mut u8,) {
                        $($path_to_types)*:: __post_return_search::<$ty > (arg0) }
                        #[export_name = "component:plugin/run#add-to-contacts"] unsafe
                        extern "C" fn export_add_to_contacts(arg0 : * mut u8, arg1 :
                        usize, arg2 : * mut u8, arg3 : usize,) -> * mut u8 {
                        $($path_to_types)*:: _export_add_to_contacts_cabi::<$ty > (arg0,
                        arg1, arg2, arg3) } #[export_name =
                        "cabi_post_component:plugin/run#add-to-contacts"] unsafe extern
                        "C" fn _post_return_add_to_contacts(arg0 : * mut u8,) {
                        $($path_to_types)*:: __post_return_add_to_contacts::<$ty > (arg0)
                        } #[export_name = "component:plugin/run#contacts"] unsafe extern
                        "C" fn export_contacts() -> * mut u8 { $($path_to_types)*::
                        _export_contacts_cabi::<$ty > () } #[export_name =
                        "cabi_post_component:plugin/run#contacts"] unsafe extern "C" fn
                        _post_return_contacts(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_contacts::<$ty > (arg0) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_component_plugin_run_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 12]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 12],
                );
            }
        }
    }
}
mod _rt {
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    pub use alloc_crate::alloc;
    pub fn as_i32<T: AsI32>(t: T) -> i32 {
        t.as_i32()
    }
    pub trait AsI32 {
        fn as_i32(self) -> i32;
    }
    impl<'a, T: Copy + AsI32> AsI32 for &'a T {
        fn as_i32(self) -> i32 {
            (*self).as_i32()
        }
    }
    impl AsI32 for i32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for char {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for usize {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    extern crate alloc as alloc_crate;
}
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_example_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::component::plugin::run::__export_component_plugin_run_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::component::plugin::run);
    };
}
#[doc(inline)]
pub(crate) use __export_example_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.35.0:component:plugin:example:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 1465] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xbb\x0a\x01A\x02\x01\
A\x0d\x01B\x0e\x01r\x02\x04names\x05values\x04\0\x0cstring-event\x03\0\0\x01p}\x01\
r\x02\x04names\x05value\x02\x04\0\x0bbytes-event\x03\0\x03\x01ps\x01r\x02\x04nam\
es\x05value\x05\x04\0\x11string-list-event\x03\0\x06\x01q\x04\x04save\0\0\x04tex\
t\x01\x01\0\x05bytes\x01\x04\0\x0bstring-list\x01\x07\0\x04\0\x05event\x03\0\x08\
\x01r\x04\x03keys\x05codecs\x09threshold}\x05limit}\x04\0\x08key-args\x03\0\x0a\x01\
r\x02\x02mk\x02\x04data\x02\x04\0\x0aprove-args\x03\0\x0c\x03\0\x14host:componen\
t/types\x05\0\x01B\x10\x01p}\x01r\x02\x05topics\x04data\0\x04\0\x07publish\x03\0\
\x01\x01r\x02\x03key\0\x05value\0\x04\0\x09put-keyed\x03\0\x03\x01q\x03\x03put\x01\
\0\0\x09put-keyed\x01\x04\0\x03get\x01\0\0\x04\0\x0esystem-command\x03\0\x05\x01\
r\x02\x07request\0\x07peer-ids\x04\0\x0cpeer-request\x03\0\x07\x01r\x02\x03key\0\
\x05value\0\x04\0\x0aput-record\x03\0\x09\x01q\x09\x07publish\x01\x02\0\x09subsc\
ribe\x01s\0\x0bunsubscribe\x01s\0\x06system\x01\x06\0\x0cpeer-request\x01\x08\0\x0a\
put-record\x01\x0a\0\x0aget-record\x01\0\0\x0dget-providers\x01\0\0\x0fstart-pro\
viding\x01\0\0\x04\0\x0call-commands\x03\0\x0b\x01ps\x01q\x04\x04data\x01\0\0\x02\
id\x01s\0\x09providers\x01\x0d\0\x04none\0\0\x04\0\x0dreturn-values\x03\0\x0e\x03\
\0\x18host:component/peerpiper\x05\x01\x02\x03\0\0\x05event\x02\x03\0\0\x08key-a\
rgs\x02\x03\0\0\x0aprove-args\x02\x03\0\x01\x0call-commands\x02\x03\0\x01\x0dret\
urn-values\x01B\x1c\x02\x03\x02\x01\x02\x04\0\x05event\x03\0\0\x02\x03\x02\x01\x03\
\x04\0\x08key-args\x03\0\x02\x02\x03\x02\x01\x04\x04\0\x0aprove-args\x03\0\x04\x02\
\x03\x02\x01\x05\x04\0\x0call-commands\x03\0\x06\x02\x03\x02\x01\x06\x04\0\x0dre\
turn-values\x03\0\x08\x01q\x04\x0dinvalid-codec\x01s\0\x14wallet-uninitialized\0\
\0\x0emultikey-error\x01s\0\x0dkey-not-found\x01s\0\x04\0\x08mk-error\x03\0\x0a\x01\
@\x01\x03evt\x01\x01\0\x04\0\x04emit\x01\x0c\x01@\x01\x03msgs\x01\0\x04\0\x03log\
\x01\x0d\x01@\0\0}\x04\0\x0brandom-byte\x01\x0e\x01p}\x01j\x01\x0f\x01\x0b\x01@\x01\
\x04args\x03\0\x10\x04\0\x06get-mk\x01\x11\x01@\x01\x04args\x05\0\x10\x04\0\x05p\
rove\x01\x12\x01@\x01\x05order\x07\x01\0\x04\0\x05order\x01\x13\x01@\0\0s\x04\0\x09\
get-scope\x01\x14\x03\0\x13host:component/host\x05\x07\x01B\x0f\x01@\0\0s\x04\0\x04\
load\x01\0\x01@\0\x01\0\x04\0\x04init\x01\x01\x01ps\x01@\0\0\x02\x04\0\x08regist\
er\x01\x03\x01j\0\x01s\x01@\x01\x04vlads\0\x04\x04\0\x06search\x01\x05\x01@\x02\x04\
vlads\x08nicknames\0\x04\x04\0\x0fadd-to-contacts\x01\x06\x01p\x02\x01@\0\0\x07\x04\
\0\x08contacts\x01\x08\x04\0\x14component:plugin/run\x05\x08\x04\0\x18component:\
plugin/example\x04\0\x0b\x0d\x01\0\x07example\x03\0\0\0G\x09producers\x01\x0cpro\
cessed-by\x02\x0dwit-component\x070.220.0\x10wit-bindgen-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
