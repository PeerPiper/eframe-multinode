#[allow(warnings)]
mod bindings;

use bestsign_core::{Base, EncodedVlad};
use bindings::component::plugin::host::{
    emit, get_mk, log, prove, random_byte, Event, KeyArgs, ProveArgs,
};
use bindings::exports::component::plugin::run::Guest;

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
        let lock_str = r#"
            check_signature("/recoverykey", "/entry/") ||
            check_signature("/pubkey", "/entry/") ||
            check_preimage("/hash")
        "#;

        let unlock_str = r#"
                push("/entry/");
                push("/entry/proof");
            "#;

        emit(&Event {
            name: "lock".to_string(),
            value: lock_str.to_string(),
        });

        emit(&Event {
            name: "unlock".to_string(),
            value: unlock_str.to_string(),
        });

        r#"

        if !is_def_var("vlad") {
            render(`
                <Vertical>
                    <Text>{{lock}}</Text>
                    <Text>{{unlock}}</Text>
                    <Button on_click=create(lock, unlock)>Create Plog</Button>
                </Vertical>
            `)
        } else {
            render(`
                <Vertical>
                    <Label>{{vlad}}</Label>
                </Vertical>
            `)
        }
        "#
        .to_string()
    }

    fn create(lock: String, unlock: String) -> bool {
        create_plog(lock, unlock).is_ok()
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

    let log = create(&config, &mut key_manager).map_err(|e| {
        log(&format!("Failed to create Plog: {:?}", e));
        Error::Plog
    })?;

    let encoded = EncodedVlad::new(Base::Base36Lower, log.vlad.clone()).to_string();

    emit(&Event {
        name: "vlad".to_string(),
        value: encoded,
    });

    Ok(log)
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
            key: key.to_string().into(),
            codec: codec.to_string().into(),
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
            mk: mk_bytes.into(),
            data: data.into(),
        })
        .map_err(|e| bestsign_core::Error::Generic(format!("Error: {:?}", e)))?;

        let multisig = Multisig::try_from(sig.as_slice());
        multisig.map_err(|e| bestsign_core::Error::Generic(format!("Error: {:?}", e)))
    }
}
