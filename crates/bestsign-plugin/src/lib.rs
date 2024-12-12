#[allow(warnings)]
mod bindings;

use bestsign_core::Codec;
use bindings::exports::component::plugin::run::Guest;
use bindings::host::component::host::{
    emit, get_mk, log, prove, random_byte, Event, KeyArgs, ProveArgs,
};
use bindings::host::component::types::{StringEvent, StringListEvent};

use bestsign_core::{
    ops::{
        config::{defaults::DEFAULT_PUBKEY, LockScript, UnlockScript},
        create,
        open::config::NewLogBuilder,
        CryptoManager,
    },
    Key, Log, Multikey, Multisig, Script,
};

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

bindings::export!(Component with_types_in bindings);

struct Component;

impl Guest for Component {
    fn load() -> String {
        let lock_str = r#"check_signature("/recoverykey", "/entry/") ||
check_signature("/pubkey", "/entry/") ||
check_preimage("/hash")
        "#;

        let unlock_str = r#"push("/entry/");
push("/entry/proof");
            "#;

        emit(&Event::Text(StringEvent {
            name: "lock".to_string(),
            value: lock_str.to_string(),
        }));

        emit(&Event::Text(StringEvent {
            name: "unlock".to_string(),
            value: unlock_str.to_string(),
        }));

        r#"
        // Get the public(multi)key from the wallet, if unlocked.
        let mk = getmk();

        if !is_def_var("vlad") && type_of(mk) != "array" {
            render(`
                <Vertical>
                    <Text>Unlock your wallet to see options</Text>
                </Vertical>
            `)
        } else {
    
            // If "plog" is defined, display the details & CRUD ops
            
            render(`
                <Vertical>
                ${if !is_def_var("pretty_plog") {
                    `
                    <Text>{{lock}}</Text>
                    <Text>{{unlock}}</Text>
                    <Button on_click=create(lock, unlock)>Create Plog</Button>
                    `
                } else {
                    `
                    <Label>pub multikey: ` + mk + `</Label>
                    ` 
                    + 
                    pretty_plog.map(|p| `<Label>${p}</Label>`).reduce(|acc, s| acc + s, "")
                }}
                </Vertical>
            `)
        }
        "#
        .to_string()
    }

    fn create(lock: String, unlock: String) -> bool {
        create_plog(lock, unlock).is_ok()
    }

    //fn update(lock: String, unlock: String) -> bool {
    //    update_plog(lock, unlock).is_ok()
    //}

    /// re-export get_mk so that rhai Script can call it
    fn getmk() -> Option<Vec<u8>> {
        let codec = Codec::Ed25519Priv.to_string();
        let args = KeyArgs {
            key: "/blah_blah".to_string(), // doesn't matter, because we only have one ed25519 key from seed and it
            // doesn't matter what we calll it
            codec, // only thing that matters is the codec
            threshold: 1,
            limit: 1,
        };
        let pk = get_mk(&args);
        log(&format!("getmk results pk: {:?}", pk));
        pk.ok()
    }
}

enum Error {
    Config,
    Plog,
}

fn create_plog(lock: String, unlock: String) -> Result<Log, Error> {
    log("Creating Plog");

    let lock = Script::Code(Key::default(), lock.to_string());
    let unlock = Script::Code(Key::default(), unlock.to_string());
    let config = NewLogBuilder::new(LockScript(lock), UnlockScript(unlock))
        .try_build()
        .map_err(|e| {
            log(&format!("Failed to build NewLogBuilder: {:?}", e));
            Error::Config
        })?;

    let mut key_manager = KeyManager::default();

    let plog = create(&config, &mut key_manager).map_err(|e| {
        log(&format!("Failed to create Plog: {:?}", e));
        Error::Plog
    })?;

    // use to_value to skip the is_human_readable check, so we keep everything
    let plog_value = serde_json::to_value(&plog).map_err(|e| {
        log(&format!("Failed to serialize Log: {:?}", e));
        Error::Plog
    })?;

    let plog_bytes = serde_json::to_string_pretty(&plog_value).map_err(|e| {
        log(&format!("Failed to serialize Log to bytes: {:?}", e));
        Error::Plog
    })?;

    emit(&Event::Text(StringEvent {
        name: "plog".to_string(),
        value: plog_bytes,
    }));

    //let encoded = EncodedVlad::new(Base::Base36Lower, plog.vlad.clone()).to_string();

    let display_data = bestsign_core::utils::get_display_data(&plog).map_err(|e| {
        log(&format!("Failed to get display data: {:?}", e));
        Error::Plog
    })?;

    if let bestsign_core::utils::DisplayData::ReturnValue { vlad, kvp_data, .. } = display_data {
        let encoded = format!("Encoded Vlad: {}", vlad.encoded);

        let pretty_kvp = kvp_data
            .iter()
            // filter: only show Multikey, tr, and Cid types of display data
            .filter_map(|data| match data {
                bestsign_core::utils::DisplayData::Multikey {
                    key_path,
                    fingerprint,
                    ..
                } => Some(format!(
                    "Multikey: {} fingerprint: {}",
                    key_path, fingerprint
                )),
                bestsign_core::utils::DisplayData::Cid {
                    key_path, encoded, ..
                } => Some(format!("Cid: {} {}", key_path, encoded)),
                _ => None,
            })
            .collect::<Vec<String>>();

        // concat encoded vlad and pretty_kvp vecs
        let mut pretty_plog = vec![encoded];
        pretty_plog.extend(pretty_kvp);

        emit(&Event::StringList(StringListEvent {
            name: "pretty_plog".to_string(),
            value: pretty_plog,
        }));
    }

    Ok(plog)
}

#[derive(Debug, Default, Clone)]
struct KeyManager {
    key: Option<Multikey>,
}

impl CryptoManager for KeyManager {
    fn get_mk(
        &mut self,
        key: &bestsign_core::Key,
        codec: bestsign_core::Codec,
        threshold: usize,
        limit: usize,
    ) -> Result<bestsign_core::Multikey, bestsign_core::Error> {
        let args = KeyArgs {
            key: key.to_string(),
            codec: codec.to_string(),
            threshold: threshold.try_into().unwrap(),
            limit: limit.try_into().unwrap(),
        };
        let mk =
            get_mk(&args).map_err(|e| bestsign_core::Error::Generic(format!("Error: {:?}", e)))?;

        let maybe_mk = Multikey::try_from(mk.as_slice());

        // if Key is "/pubkey" then set the key
        if maybe_mk.is_ok() {
            let mk = maybe_mk.unwrap();
            if key.to_string() == DEFAULT_PUBKEY {
                self.key = Some(mk.clone());
            }
            Ok(mk)
        } else {
            Err(bestsign_core::Error::Generic(
                "Failed to get Multikey from KeyManager".to_string(),
            ))
        }
    }

    fn prove(
        &self,
        mk: &bestsign_core::Multikey,
        data: &[u8],
    ) -> Result<bestsign_core::Multisig, bestsign_core::Error> {
        let mk_bytes: Vec<u8> = mk.clone().into();

        let sig = prove(&ProveArgs {
            mk: mk_bytes,
            data: data.into(),
        })
        .map_err(|e| bestsign_core::Error::Generic(format!("Error: {:?}", e)))?;

        let multisig = Multisig::try_from(sig.as_slice());
        multisig.map_err(|e| bestsign_core::Error::Generic(format!("Error: {:?}", e)))
    }
}
