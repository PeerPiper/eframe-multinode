#![allow(dead_code)]
#[allow(warnings)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod bindings;

use bindings::exports::component::plugin::run::Guest;
use bindings::host::component::host::{get_scope, log, random_byte};

use std::sync::{LazyLock, Mutex};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct PipePad(String);

/// The global static PIPEPAD itself
static PIPEPAD: LazyLock<Mutex<PipePad>> = LazyLock::new(|| Mutex::new(PipePad::default()));

/// Constant for saving the contact book to rhai::Scope
const PIPEPAD_KEY: &str = "pipepad";

/// Custom function to use the import for random byte generation.
///
/// We do this is because "js" feature is incompatible with the component model
/// if you ever got the __wbindgen_placeholder__ error when trying to use the `js` feature
/// of getrandom,
fn imported_random(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    // iterate over the length of the destination buffer and fill it with random bytes
    (0..dest.len()).for_each(|i| {
        dest[i] = random_byte();
    });

    Ok(())
}

getrandom::register_custom_getrandom!(imported_random);

struct Component;

impl Guest for Component {
    /// Say hello!
    fn load() -> String {
        include_str!(concat!(env!("OUT_DIR"), "/pipepad.rhai")).to_string()
    }

    /// Now that the rhai Scope is ready, let's call it and load the PipePad with contacts.
    fn init() {
        log("Initializing pipepad");
        // get the scope from the host system.
        // This constructor only should be called once the host state scope is ready.
        let scope = get_scope();

        // turn scope string back into json
        let _value: rhai::Scope = serde_json::from_str(&scope).unwrap();

        //// try to get contact book from scope, falling back to empty contact book upon failure
        //// There are two ways we could save rhai scope: emit from wasm, or set to var in rhai.
        //let contact_book: PipePad = value
        //    .get_value::<String>(PIPEPAD_KEY)
        //    .and_then(|contact_book_string| {
        //        serde_json::from_str::<Vec<u8>>(&contact_book_string)
        //            .map(|bytes| AutoCommit::load(&bytes).unwrap_or_default())
        //            .map(|doc| hydrate(&doc).unwrap_or_default())
        //            .ok()
        //    })
        //    .unwrap_or_default();
        //
        //log(&format!(
        //    "Loaded contact book with {:?} contacts",
        //    contact_book
        //));
        //
        //// save the contact book to the global static PIPEPAD
        //let mut lock = PIPEPAD.lock().unwrap();
        //*lock = contact_book;
        //
        //log("TEST to see if contacts is also a key in scope");
        //
        //match value.get_value::<String>("contacts") {
        //    Some(contacts) => {
        //        log(&format!("Contacts key found in scope: {}", contacts));
        //    }
        //    None => {
        //        log("Contacts key not found in scope");
        //    }
        //}
    }

    fn register() -> Vec<String> {
        // This function will be available to us in Rhai.
        // It can only return Option, Bool, Array, String, or numbers.
        // No records (structs), enums (variants) as they are named,
        // and the host has no idea what your names are.
        // TODO: Grab names from the *.wasm wit to overcome this?
        vec![]
    }
}

bindings::export!(Component with_types_in bindings);
