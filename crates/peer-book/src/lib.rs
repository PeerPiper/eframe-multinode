#[allow(warnings)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod bindings;
mod contact_book;

use std::sync::{LazyLock, Mutex};

use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile};
use bindings::exports::component::plugin::run::Guest;
use bindings::host::component::host::{
    emit, get_scope, log, order, random_byte, AllCommands, Event,
};
//{
//    emit, get_mk, log, order, prove, random_byte, AllCommands, Event, KeyArgs, ProveArgs,
//};

use bindings::host::component::types::StringEvent;
use multicid::EncodedVlad;

use contact_book::{Contact, ContactBook, VladId};

/// Constant for saving the contact book to rhai::Scope
// TODO: Migrate this to Guest Resource as time permits. It will change the layer API though.
const CONTACT_BOOK_KEY: &str = "contact_book";

/// The global static CONTACT_BOOK itself
static CONTACT_BOOK: LazyLock<Mutex<ContactBook>> =
    LazyLock::new(|| Mutex::new(ContactBook::default()));

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
        include_str!(concat!(env!("OUT_DIR"), "/peer-book.rhai")).to_string()
    }

    /// Now that the rhai Scope is ready, let's call it and load the ContactBook with contacts.
    fn init() {
        log("Initializing contact book");
        // get the scope from the host system.
        // This constructor only should be called once the host state scope is ready.
        let scope = get_scope();

        // turn scope string back into json
        let value: rhai::Scope = serde_json::from_str(&scope).unwrap();

        // try to get contact book from scope, falling back to empty contact book upon failure
        // There are two ways we could save rhai scope: emit from wasm, or set to var in rhai.
        let contact_book: ContactBook = value
            .get_value::<String>(CONTACT_BOOK_KEY)
            .and_then(|contact_book_string| {
                serde_json::from_str::<Vec<u8>>(&contact_book_string)
                    .map(|bytes| AutoCommit::load(&bytes).unwrap_or_default())
                    .map(|doc| hydrate(&doc).unwrap_or_default())
                    .ok()
            })
            .unwrap_or_default();

        log(&format!(
            "Loaded contact book with {:?} contacts",
            contact_book
        ));

        // save the contact book to the global static CONTACT_BOOK
        let mut lock = CONTACT_BOOK.lock().unwrap();
        *lock = contact_book;

        log("TEST to see if contacts is also a key in scope");

        match value.get_value::<String>("contacts") {
            Some(contacts) => {
                log(&format!("Contacts key found in scope: {}", contacts));
            }
            None => {
                log("Contacts key not found in scope");
            }
        }
    }

    fn register() -> Vec<String> {
        // This function will be available to us in Rhai.
        // It can only return Option, Bool, Array, String, or numbers.
        // No records (structs), enums (variants) as they are named,
        // and the host has no idea what your names are.
        // TODO: Grab names from the *.wasm wit to overcome this?
        vec!["contacts".to_string()]
    }

    // The contacts function which returns a list of contacts,
    // each of which is a list of [vald, nickname] strings.
    fn contacts() -> Vec<Vec<String>> {
        let lock = CONTACT_BOOK.lock().unwrap();
        lock.contacts()
            .iter()
            .map(|contact| contact.clone().into())
            .collect()
    }

    fn search(vlad: String) -> Result<(), String> {
        log(&format!("Searching for {}", vlad));

        // The VLAD gives us a key that we can query the DHT with.
        // First we need to convert the string to a proper typed VLAD
        let encoded_vlad = EncodedVlad::try_from(vlad.as_str())
            .map_err(|e| format!("Failed to parse VLAD: {}", e))?;
        let decoded_vlad = encoded_vlad.to_inner();

        let vlad_bytes: Vec<u8> = decoded_vlad.into();

        // send Get command to the system, so it looks up the Vlad and its DHT record
        // then the system will store the response in the rhai Scope
        // order(..) will put an entry in the rhai Scope for "get_record" (command in snake_case)
        // as well as (key, option<result>) for the response.
        // So our plugin must check the key for a result, and optionally display it
        order(&AllCommands::GetRecord(vlad_bytes.clone()));

        // we can emit (key, value): (vlad-bytes: Vec<u8>, vlad-encoded: String) so that it's saved to the rhai Scope
        // and available to decode when the response comes in. Because the key is "bytes"
        // but we sent an encoded string, we'll need to decode once we get a response.
        emit(&Event::Text(StringEvent {
            // turn Vec<u8> into a string like "[1, 2, 3, 4]"
            // rhai scope does best with string, bytes arrays, not so good at it.
            name: format!(
                "[{}]",
                vlad_bytes
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            value: vlad,
        }));

        // now we will be able to look up the bytes we get and map them to the vlad string that was
        // entered.
        Ok(())
    }

    fn add_to_contacts(vlad: String, nickname: String) -> Result<(), String> {
        log(&format!(
            "Adding {} to contacts with nickname {}",
            vlad, nickname
        ));

        let vlad_id = VladId::try_from(vlad.as_str())
            .map_err(|e| format!("Invalid Vlad, failed to parse: {}", e))?;

        // add to local contact book
        let contact = Contact::builder()
            .id(vlad_id)
            .name(nickname.clone())
            .notes("NB:".into())
            .build();

        // add to the global contact book
        let mut lock = CONTACT_BOOK.lock().unwrap();
        lock.add(contact);

        let mut doc = AutoCommit::new();
        reconcile(&mut doc, &*lock).map_err(|e| format!("Failed to reconcile: {}", e))?;
        let saved = doc.save();

        // to save the contacts book to our Rhai Scope (memory), we emit it.
        // the Rhai scope is saved to the disk after debouncing.
        emit(&Event::Text(StringEvent {
            name: CONTACT_BOOK_KEY.to_string(),
            // saved with serde_json
            value: serde_json::to_string(&saved).unwrap(),
        }));
        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::AutoCommit;
    use autosurgeon::{hydrate, reconcile};
    use bestsign_core::Codec;
    use multihash::mh;
    use multikey::nonce;

    // test roundtrip ContactBook
    #[test]
    fn test_roundtrip_contact_book() {
        // build a nonce
        let mut rng = rand::rngs::OsRng;
        let nonce = nonce::Builder::new_from_random_bytes(32, &mut rng)
            .try_build()
            .unwrap();

        // build a cid
        let cid = multicid::cid::Builder::new(Codec::Cidv1)
            .with_target_codec(Codec::DagCbor)
            .with_hash(
                &mh::Builder::new_from_bytes(Codec::Sha2256, b"for great justice, move every zig!")
                    .unwrap()
                    .try_build()
                    .unwrap(),
            )
            .try_build()
            .unwrap();

        let vlad = multicid::vlad::Builder::default()
            .with_nonce(&nonce)
            .with_cid(&cid)
            .try_build(|cid| {
                // sign those bytes
                let v: Vec<u8> = cid.clone().into();
                Ok(v)
            })
            .unwrap();

        let mut contact_book = ContactBook::default();
        let contact = Contact::builder()
            .id(VladId::new(vlad))
            .name("John Doe".to_string())
            .notes("Some notes about John Doe".to_string())
            .build();

        contact_book.add(contact.clone());

        let mut doc = AutoCommit::new();
        reconcile(&mut doc, &contact_book).unwrap();
        let saved = doc.save();

        let saved_string = format!(
            "[{}]",
            saved
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let contact_book: ContactBook = serde_json::from_str::<Vec<u8>>(&saved_string)
            .map(|bytes| AutoCommit::load(&bytes).unwrap_or_default())
            .map(|doc| hydrate(&doc).unwrap_or_default())
            .unwrap();

        assert_eq!(contact_book.contacts().len(), 1);
        assert_eq!(contact_book.contacts()[0], contact);
    }
}
