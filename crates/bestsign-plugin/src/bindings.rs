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
            /// Event wherte there is a list of strings
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
        pub mod host {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Event = super::super::super::host::component::types::Event;
            pub type KeyArgs = super::super::super::host::component::types::KeyArgs;
            pub type ProveArgs = super::super::super::host::component::types::ProveArgs;
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
                            (0i32, ptr1.cast_mut(), len1, ptr2.cast_mut(), len2)
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
                            (1i32, ptr4.cast_mut(), len4, ptr5.cast_mut(), len5)
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
                            (2i32, ptr7.cast_mut(), len7, result9, len9)
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
                pub unsafe fn _export_create_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                ) -> i32 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let len1 = arg3;
                    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
                    let result2 = T::create(
                        _rt::string_lift(bytes0),
                        _rt::string_lift(bytes1),
                    );
                    match result2 {
                        true => 1,
                        false => 0,
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_getmk_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::getmk();
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result0 {
                        Some(e) => {
                            *ptr1.add(0).cast::<u8>() = (1i32) as u8;
                            let vec2 = (e).into_boxed_slice();
                            let ptr2 = vec2.as_ptr().cast::<u8>();
                            let len2 = vec2.len();
                            ::core::mem::forget(vec2);
                            *ptr1.add(8).cast::<usize>() = len2;
                            *ptr1.add(4).cast::<*mut u8>() = ptr2.cast_mut();
                        }
                        None => {
                            *ptr1.add(0).cast::<u8>() = (0i32) as u8;
                        }
                    };
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_getmk<T: Guest>(arg0: *mut u8) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {}
                        _ => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            let base3 = l1;
                            let len3 = l2;
                            _rt::cabi_dealloc(base3, len3 * 1, 1);
                        }
                    }
                }
                pub trait Guest {
                    /// from ./deps/host.wit
                    /// use host:component/types.{event, key-args};
                    /// loads just the XML like markdown
                    fn load() -> _rt::String;
                    /// Creates a data provenance log, returns the serialized log.
                    fn create(lock: _rt::String, unlock: _rt::String) -> bool;
                    /// Re-export get-mk, so that the rhai script can check to see if we have an available Multikey to use
                    fn getmk() -> Option<_rt::Vec<u8>>;
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
                        "component:plugin/run#create"] unsafe extern "C" fn
                        export_create(arg0 : * mut u8, arg1 : usize, arg2 : * mut u8,
                        arg3 : usize,) -> i32 { $($path_to_types)*::
                        _export_create_cabi::<$ty > (arg0, arg1, arg2, arg3) }
                        #[export_name = "component:plugin/run#getmk"] unsafe extern "C"
                        fn export_getmk() -> * mut u8 { $($path_to_types)*::
                        _export_getmk_cabi::<$ty > () } #[export_name =
                        "cabi_post_component:plugin/run#getmk"] unsafe extern "C" fn
                        _post_return_getmk(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_getmk::<$ty > (arg0) } };
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
macro_rules! __export_plugin_world_impl {
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
pub(crate) use __export_plugin_world_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.35.0:component:plugin:plugin-world:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 849] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xce\x05\x01A\x02\x01\
A\x09\x01B\x0e\x01r\x02\x04names\x05values\x04\0\x0cstring-event\x03\0\0\x01p}\x01\
r\x02\x04names\x05value\x02\x04\0\x0bbytes-event\x03\0\x03\x01ps\x01r\x02\x04nam\
es\x05value\x05\x04\0\x11string-list-event\x03\0\x06\x01q\x03\x04text\x01\x01\0\x05\
bytes\x01\x04\0\x0bstring-list\x01\x07\0\x04\0\x05event\x03\0\x08\x01r\x04\x03ke\
ys\x05codecs\x09threshold}\x05limit}\x04\0\x08key-args\x03\0\x0a\x01r\x02\x02mk\x02\
\x04data\x02\x04\0\x0aprove-args\x03\0\x0c\x03\0\x14host:component/types\x05\0\x02\
\x03\0\0\x05event\x02\x03\0\0\x08key-args\x02\x03\0\0\x0aprove-args\x01B\x14\x02\
\x03\x02\x01\x01\x04\0\x05event\x03\0\0\x02\x03\x02\x01\x02\x04\0\x08key-args\x03\
\0\x02\x02\x03\x02\x01\x03\x04\0\x0aprove-args\x03\0\x04\x01q\x04\x0dinvalid-cod\
ec\x01s\0\x14wallet-uninitialized\0\0\x0emultikey-error\x01s\0\x0dkey-not-found\x01\
s\0\x04\0\x08mk-error\x03\0\x06\x01@\x01\x03evt\x01\x01\0\x04\0\x04emit\x01\x08\x01\
@\x01\x03msgs\x01\0\x04\0\x03log\x01\x09\x01@\0\0}\x04\0\x0brandom-byte\x01\x0a\x01\
p}\x01j\x01\x0b\x01\x07\x01@\x01\x04args\x03\0\x0c\x04\0\x06get-mk\x01\x0d\x01@\x01\
\x04args\x05\0\x0c\x04\0\x05prove\x01\x0e\x03\0\x13host:component/host\x05\x04\x01\
B\x08\x01@\0\0s\x04\0\x04load\x01\0\x01@\x02\x04locks\x06unlocks\0\x7f\x04\0\x06\
create\x01\x01\x01p}\x01k\x02\x01@\0\0\x03\x04\0\x05getmk\x01\x04\x04\0\x14compo\
nent:plugin/run\x05\x05\x04\0\x1dcomponent:plugin/plugin-world\x04\0\x0b\x12\x01\
\0\x0cplugin-world\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-compon\
ent\x070.220.0\x10wit-bindgen-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
