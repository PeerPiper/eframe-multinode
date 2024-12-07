#[allow(dead_code)]
pub mod component {
    #[allow(dead_code)]
    pub mod plugin {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            /// The Event type.
            #[derive(Clone)]
            pub struct Event {
                /// The variable name
                pub name: _rt::String,
                pub value: _rt::String,
            }
            impl ::core::fmt::Debug for Event {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Event")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
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
        }
        #[allow(dead_code, clippy::all)]
        pub mod host {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            pub type Event = super::super::super::component::plugin::types::Event;
            #[allow(unused_unsafe, clippy::all)]
            /// emit an event.
            pub fn emit(evt: &Event) {
                unsafe {
                    let super::super::super::component::plugin::types::Event {
                        name: name0,
                        value: value0,
                    } = evt;
                    let vec1 = name0;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    let vec2 = value0;
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "component:plugin/host")]
                    extern "C" {
                        #[link_name = "emit"]
                        fn wit_import(_: *mut u8, _: usize, _: *mut u8, _: usize);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8, _: usize, _: *mut u8, _: usize) {
                        unreachable!()
                    }
                    wit_import(ptr1.cast_mut(), len1, ptr2.cast_mut(), len2);
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
                    #[link(wasm_import_module = "component:plugin/host")]
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
                    #[link(wasm_import_module = "component:plugin/host")]
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
                pub type KeyArgs = super::super::super::super::component::plugin::types::KeyArgs;
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
                ) {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let len1 = arg3;
                    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
                    T::create(_rt::string_lift(bytes0), _rt::string_lift(bytes1));
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_unlock_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                    arg4: *mut u8,
                    arg5: usize,
                ) {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let len1 = arg3;
                    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
                    let len2 = arg5;
                    let bytes2 = _rt::Vec::from_raw_parts(arg4.cast(), len2, len2);
                    T::unlock(
                        _rt::string_lift(bytes0),
                        _rt::string_lift(bytes1),
                        _rt::string_lift(bytes2),
                    );
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_get_mk_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                    arg4: i32,
                    arg5: i32,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let len1 = arg3;
                    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
                    let result2 = T::get_mk(super::super::super::super::component::plugin::types::KeyArgs {
                        key: _rt::string_lift(bytes0),
                        codec: _rt::string_lift(bytes1),
                        threshold: arg4 as u8,
                        limit: arg5 as u8,
                    });
                    let ptr3 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result2 {
                        Ok(e) => {
                            *ptr3.add(0).cast::<u8>() = (0i32) as u8;
                            let vec4 = (e).into_boxed_slice();
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            ::core::mem::forget(vec4);
                            *ptr3.add(8).cast::<usize>() = len4;
                            *ptr3.add(4).cast::<*mut u8>() = ptr4.cast_mut();
                        }
                        Err(e) => {
                            *ptr3.add(0).cast::<u8>() = (1i32) as u8;
                            match e {
                                MkError::InvalidCodec(e) => {
                                    *ptr3.add(4).cast::<u8>() = (0i32) as u8;
                                    let vec5 = (e.into_bytes()).into_boxed_slice();
                                    let ptr5 = vec5.as_ptr().cast::<u8>();
                                    let len5 = vec5.len();
                                    ::core::mem::forget(vec5);
                                    *ptr3.add(12).cast::<usize>() = len5;
                                    *ptr3.add(8).cast::<*mut u8>() = ptr5.cast_mut();
                                }
                                MkError::WalletUninitialized => {
                                    *ptr3.add(4).cast::<u8>() = (1i32) as u8;
                                }
                                MkError::MultikeyError(e) => {
                                    *ptr3.add(4).cast::<u8>() = (2i32) as u8;
                                    let vec6 = (e.into_bytes()).into_boxed_slice();
                                    let ptr6 = vec6.as_ptr().cast::<u8>();
                                    let len6 = vec6.len();
                                    ::core::mem::forget(vec6);
                                    *ptr3.add(12).cast::<usize>() = len6;
                                    *ptr3.add(8).cast::<*mut u8>() = ptr6.cast_mut();
                                }
                                MkError::KeyNotFound(e) => {
                                    *ptr3.add(4).cast::<u8>() = (3i32) as u8;
                                    let vec7 = (e.into_bytes()).into_boxed_slice();
                                    let ptr7 = vec7.as_ptr().cast::<u8>();
                                    let len7 = vec7.len();
                                    ::core::mem::forget(vec7);
                                    *ptr3.add(12).cast::<usize>() = len7;
                                    *ptr3.add(8).cast::<*mut u8>() = ptr7.cast_mut();
                                }
                            }
                        }
                    };
                    ptr3
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_get_mk<T: Guest>(arg0: *mut u8) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            let base3 = l1;
                            let len3 = l2;
                            _rt::cabi_dealloc(base3, len3 * 1, 1);
                        }
                        _ => {
                            let l4 = i32::from(*arg0.add(4).cast::<u8>());
                            match l4 {
                                0 => {
                                    let l5 = *arg0.add(8).cast::<*mut u8>();
                                    let l6 = *arg0.add(12).cast::<usize>();
                                    _rt::cabi_dealloc(l5, l6, 1);
                                }
                                1 => {}
                                2 => {
                                    let l7 = *arg0.add(8).cast::<*mut u8>();
                                    let l8 = *arg0.add(12).cast::<usize>();
                                    _rt::cabi_dealloc(l7, l8, 1);
                                }
                                _ => {
                                    let l9 = *arg0.add(8).cast::<*mut u8>();
                                    let l10 = *arg0.add(12).cast::<usize>();
                                    _rt::cabi_dealloc(l9, l10, 1);
                                }
                            }
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_prove_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let len1 = arg3;
                    let result2 = T::prove(ProveArgs {
                        mk: _rt::Vec::from_raw_parts(arg0.cast(), len0, len0),
                        data: _rt::Vec::from_raw_parts(arg2.cast(), len1, len1),
                    });
                    let ptr3 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result2 {
                        Ok(e) => {
                            *ptr3.add(0).cast::<u8>() = (0i32) as u8;
                            let vec4 = (e).into_boxed_slice();
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            ::core::mem::forget(vec4);
                            *ptr3.add(8).cast::<usize>() = len4;
                            *ptr3.add(4).cast::<*mut u8>() = ptr4.cast_mut();
                        }
                        Err(e) => {
                            *ptr3.add(0).cast::<u8>() = (1i32) as u8;
                            match e {
                                MkError::InvalidCodec(e) => {
                                    *ptr3.add(4).cast::<u8>() = (0i32) as u8;
                                    let vec5 = (e.into_bytes()).into_boxed_slice();
                                    let ptr5 = vec5.as_ptr().cast::<u8>();
                                    let len5 = vec5.len();
                                    ::core::mem::forget(vec5);
                                    *ptr3.add(12).cast::<usize>() = len5;
                                    *ptr3.add(8).cast::<*mut u8>() = ptr5.cast_mut();
                                }
                                MkError::WalletUninitialized => {
                                    *ptr3.add(4).cast::<u8>() = (1i32) as u8;
                                }
                                MkError::MultikeyError(e) => {
                                    *ptr3.add(4).cast::<u8>() = (2i32) as u8;
                                    let vec6 = (e.into_bytes()).into_boxed_slice();
                                    let ptr6 = vec6.as_ptr().cast::<u8>();
                                    let len6 = vec6.len();
                                    ::core::mem::forget(vec6);
                                    *ptr3.add(12).cast::<usize>() = len6;
                                    *ptr3.add(8).cast::<*mut u8>() = ptr6.cast_mut();
                                }
                                MkError::KeyNotFound(e) => {
                                    *ptr3.add(4).cast::<u8>() = (3i32) as u8;
                                    let vec7 = (e.into_bytes()).into_boxed_slice();
                                    let ptr7 = vec7.as_ptr().cast::<u8>();
                                    let len7 = vec7.len();
                                    ::core::mem::forget(vec7);
                                    *ptr3.add(12).cast::<usize>() = len7;
                                    *ptr3.add(8).cast::<*mut u8>() = ptr7.cast_mut();
                                }
                            }
                        }
                    };
                    ptr3
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_prove<T: Guest>(arg0: *mut u8) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            let base3 = l1;
                            let len3 = l2;
                            _rt::cabi_dealloc(base3, len3 * 1, 1);
                        }
                        _ => {
                            let l4 = i32::from(*arg0.add(4).cast::<u8>());
                            match l4 {
                                0 => {
                                    let l5 = *arg0.add(8).cast::<*mut u8>();
                                    let l6 = *arg0.add(12).cast::<usize>();
                                    _rt::cabi_dealloc(l5, l6, 1);
                                }
                                1 => {}
                                2 => {
                                    let l7 = *arg0.add(8).cast::<*mut u8>();
                                    let l8 = *arg0.add(12).cast::<usize>();
                                    _rt::cabi_dealloc(l7, l8, 1);
                                }
                                _ => {
                                    let l9 = *arg0.add(8).cast::<*mut u8>();
                                    let l10 = *arg0.add(12).cast::<usize>();
                                    _rt::cabi_dealloc(l9, l10, 1);
                                }
                            }
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_unlocked_cabi<T: Guest>() -> i32 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::unlocked();
                    match result0 {
                        true => 1,
                        false => 0,
                    }
                }
                pub trait Guest {
                    /// loads just the XML like markdown
                    fn load() -> _rt::String;
                    /// create a seed and lock it
                    fn create(username: _rt::String, password: _rt::String);
                    /// Unlock an existing encrypted seed
                    fn unlock(
                        username: _rt::String,
                        password: _rt::String,
                        encrypted_seed: _rt::String,
                    );
                    /// Gets the Multikey
                    fn get_mk(args: KeyArgs) -> Result<_rt::Vec<u8>, MkError>;
                    /// Proves the data for the given Multikey.
                    fn prove(args: ProveArgs) -> Result<_rt::Vec<u8>, MkError>;
                    /// Returns whether the wallet is unlocked or not
                    fn unlocked() -> bool;
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
                        arg3 : usize,) { $($path_to_types)*:: _export_create_cabi::<$ty >
                        (arg0, arg1, arg2, arg3) } #[export_name =
                        "component:plugin/run#unlock"] unsafe extern "C" fn
                        export_unlock(arg0 : * mut u8, arg1 : usize, arg2 : * mut u8,
                        arg3 : usize, arg4 : * mut u8, arg5 : usize,) {
                        $($path_to_types)*:: _export_unlock_cabi::<$ty > (arg0, arg1,
                        arg2, arg3, arg4, arg5) } #[export_name =
                        "component:plugin/run#get-mk"] unsafe extern "C" fn
                        export_get_mk(arg0 : * mut u8, arg1 : usize, arg2 : * mut u8,
                        arg3 : usize, arg4 : i32, arg5 : i32,) -> * mut u8 {
                        $($path_to_types)*:: _export_get_mk_cabi::<$ty > (arg0, arg1,
                        arg2, arg3, arg4, arg5) } #[export_name =
                        "cabi_post_component:plugin/run#get-mk"] unsafe extern "C" fn
                        _post_return_get_mk(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_get_mk::<$ty > (arg0) } #[export_name =
                        "component:plugin/run#prove"] unsafe extern "C" fn
                        export_prove(arg0 : * mut u8, arg1 : usize, arg2 : * mut u8, arg3
                        : usize,) -> * mut u8 { $($path_to_types)*::
                        _export_prove_cabi::<$ty > (arg0, arg1, arg2, arg3) }
                        #[export_name = "cabi_post_component:plugin/run#prove"] unsafe
                        extern "C" fn _post_return_prove(arg0 : * mut u8,) {
                        $($path_to_types)*:: __post_return_prove::<$ty > (arg0) }
                        #[export_name = "component:plugin/run#unlocked"] unsafe extern
                        "C" fn export_unlocked() -> i32 { $($path_to_types)*::
                        _export_unlocked_cabi::<$ty > () } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_component_plugin_run_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 16]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 16],
                );
            }
        }
    }
}
mod _rt {
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
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
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    extern crate alloc as alloc_crate;
    pub use alloc_crate::alloc;
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
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 787] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x90\x05\x01A\x02\x01\
A\x09\x01B\x04\x01r\x02\x04names\x05values\x04\0\x05event\x03\0\0\x01r\x04\x03ke\
ys\x05codecs\x09threshold}\x05limit}\x04\0\x08key-args\x03\0\x02\x03\0\x16compon\
ent:plugin/types\x05\0\x02\x03\0\0\x05event\x03\0\x05event\x03\0\x01\x02\x03\0\0\
\x08key-args\x01B\x0a\x02\x03\x02\x01\x01\x04\0\x05event\x03\0\0\x02\x03\x02\x01\
\x03\x04\0\x08key-args\x03\0\x02\x01@\x01\x03evt\x01\x01\0\x04\0\x04emit\x01\x04\
\x01@\x01\x03msgs\x01\0\x04\0\x03log\x01\x05\x01@\0\0}\x04\0\x0brandom-byte\x01\x06\
\x03\0\x15component:plugin/host\x05\x04\x01B\x16\x02\x03\x02\x01\x01\x04\0\x05ev\
ent\x03\0\0\x02\x03\x02\x01\x03\x04\0\x08key-args\x03\0\x02\x01q\x04\x0dinvalid-\
codec\x01s\0\x14wallet-uninitialized\0\0\x0emultikey-error\x01s\0\x0dkey-not-fou\
nd\x01s\0\x04\0\x08mk-error\x03\0\x04\x01p}\x01r\x02\x02mk\x06\x04data\x06\x04\0\
\x0aprove-args\x03\0\x07\x01@\0\0s\x04\0\x04load\x01\x09\x01@\x02\x08usernames\x08\
passwords\x01\0\x04\0\x06create\x01\x0a\x01@\x03\x08usernames\x08passwords\x0een\
crypted-seeds\x01\0\x04\0\x06unlock\x01\x0b\x01j\x01\x06\x01\x05\x01@\x01\x04arg\
s\x03\0\x0c\x04\0\x06get-mk\x01\x0d\x01@\x01\x04args\x08\0\x0c\x04\0\x05prove\x01\
\x0e\x01@\0\0\x7f\x04\0\x08unlocked\x01\x0f\x04\0\x14component:plugin/run\x05\x05\
\x04\0\x1dcomponent:plugin/plugin-world\x04\0\x0b\x12\x01\0\x0cplugin-world\x03\0\
\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x070.220.0\x10wit-bi\
ndgen-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
