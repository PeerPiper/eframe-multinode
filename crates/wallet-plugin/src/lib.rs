#[allow(warnings)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod bindings;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use bindings::exports::component::plugin::run::{Guest, KeyArgs, MkError, ProveArgs};
use bindings::host::component::host::{emit, log, random_byte};
use bindings::host::component::types::{Event, StringEvent};

use bestsign_core::ops::config::defaults::{DEFAULT_ENTRYKEY, DEFAULT_VLAD_KEY};
use bestsign_core::{mk, Codec, EncodedMultikey, Key, Multikey, Views as _};
use seed_keeper_core::credentials::{Credentials, MinString, Wallet};

static WALLET: LazyLock<Mutex<Option<Wallet>>> = LazyLock::new(|| Mutex::new(None));

/// Encoded Public Key (epk) map
///
/// A static HashMap to map the epk to the (key, mk) tuple.
static EPK_MAP: LazyLock<Mutex<HashMap<EncodedMultikey, (Key, Multikey)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
    fn load() -> String {
        // There are 3 options:
        // 1) No encrypted_seed, and is not unlocked. Create a seed and unlock.
        // 2) encrypted_seed, and is not unlocked. Unlock the seed.
        // 3) encrypted_seed, and is unlocked. Show the encrypted seed.
        r#"
        if !is_def_var("encrypted_seed") && !unlocked() {
            render(`
                <div>
                    <label>Create a new wallet</label>
                    <input value="{{username}}" />
                    <input value="{{password}}" password="true" />
                    <button data-on-click="create(username, password)">Login</button>
                </div>
            `)
        } else if is_def_var("encrypted_seed") && !unlocked() {
            render(`
                <div>
                    <label>Unlock your wallet</label>
                    <input value="{{encrypted_seed}}" />
                    <input value="{{username}}" />
                    <input value="{{password}}" password="true" />
                    <button data-on-click="unlock(username, password, encrypted_seed)">Unlock</button>
                </div>
            `)
        } else {
            render(`
                <div>
                    <label>Encrypted Seed:</label>
                    <label>{{encrypted_seed}}</label>
                </div>
            `)
        }
        "#
        .to_string()
    }

    /// Returns true if the wallet is unlocked (is some)
    fn unlocked() -> bool {
        let w = WALLET.lock().unwrap();
        w.is_some()
    }

    fn unlock(username: String, password: String, encrypted_seed: String) {
        log("Unlock button clicked");

        // return early if Err Result
        let Ok(username_min) = MinString::<8>::new(&username) else {
            log(&format!("Failed to create MinString from {}", username));
            return;
        };

        let Ok(password_min) = MinString::<8>::new(&password) else {
            log(&format!("Failed to create MinString from {}", password));
            return;
        };

        let encrypted_seed = encrypted_seed
            .trim_start_matches("[")
            .trim_end_matches("]")
            .split(",")
            .map(|s| s.parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        let credentials = Credentials {
            username: username_min,
            password: password_min,
            encrypted_seed: Some(encrypted_seed),
        };

        process_creds(credentials);
    }

    fn create(username: String, password: String) {
        log("Login button clicked");

        // return early if Err Result
        let Ok(username_min) = MinString::<8>::new(&username) else {
            log(&format!("Failed to create MinString from {}", username));
            return;
        };

        let Ok(password_min) = MinString::<8>::new(&password) else {
            log(&format!("Failed to create MinString from {}", password));
            return;
        };

        let credentials = Credentials {
            username: username_min,
            password: password_min,
            encrypted_seed: None,
        };

        process_creds(credentials);
    }

    /// Returns the Public Multikey associated with the given key args.
    fn get_mk(args: KeyArgs) -> Result<Vec<u8>, MkError> {
        let codec = Codec::try_from(args.codec.as_str())
            .map_err(|e| MkError::InvalidCodec(format!("Invalid codec, found: {}", e)))?;

        // if args.key.as_str() is DEFAULT_ENTRYKEY or DEFAULT_VLAD_KEY builder is mk::Builder::new_from_random_bytes(codec, &mut rng)
        // otherwise mk::Builder::new_from_seed(codec, seed)
        let mk_buildr =
            if args.key.as_str() == DEFAULT_ENTRYKEY || args.key.as_str() == DEFAULT_VLAD_KEY {
                log("Creating Multikey from random bytes");
                let mut rng = rand::thread_rng();
                mk::Builder::new_from_random_bytes(codec, &mut rng)
                    .map_err(|e| MkError::InvalidCodec(format!("Invalid codec, found: {}", e)))?
            } else {
                // get seed from wallet.seed
                //log("Creating Multikey from seed");
                let wallet_lock = WALLET.lock().unwrap();
                let Some(wallet) = wallet_lock.as_ref() else {
                    return Err(MkError::WalletUninitialized);
                };

                mk::Builder::new_from_seed(codec, wallet.seed())
                    .map_err(|e| MkError::InvalidCodec(format!("Invalid codec, found: {}", e)))?
            };

        let mk = mk_buildr.try_build().unwrap();

        let public_key = mk
            .conv_view()
            .map_err(multikey_error)?
            .to_public_key()
            .map_err(multikey_error)?;

        let key = Key::try_from(args.key).map_err(multikey_error)?;
        let epk = EncodedMultikey::from(public_key.clone());

        // add to key map
        EPK_MAP
            .lock()
            .unwrap()
            .insert(epk, (key.clone(), mk.clone()));

        Ok(public_key.into())
    }

    /// Proves the data for the given Multikey.
    // prove: func(mk: list<u8>, data: list<u8>) -> list<u8>;
    fn prove(args: ProveArgs) -> Result<Vec<u8>, MkError> {
        log("Proving some data");

        let mk = Multikey::try_from(args.mk.as_slice()).map_err(multikey_error)?;
        let attr = mk.attr_view().map_err(multikey_error)?;

        // Hanle both public and private keys
        let pk = if attr.is_secret_key() {
            mk.conv_view()
                .map_err(multikey_error)?
                .to_public_key()
                .map_err(multikey_error)?
        } else {
            mk.clone()
        };

        let epk = EncodedMultikey::from(pk.clone());

        let mut map_lock = EPK_MAP.lock().unwrap();

        let (key, mk) = map_lock
            .get(&epk)
            .ok_or(MkError::KeyNotFound(epk.to_string()))?;

        let signature = mk
            .sign_view()
            .map_err(multikey_error)?
            .sign(&args.data, false, None)
            .map_err(multikey_error)?;

        // remove the key if it is DEFAULT_ENTRYKEY or DEFAULT_VLAD_KEY
        if key.as_str() == DEFAULT_ENTRYKEY || key.as_str() == DEFAULT_VLAD_KEY {
            map_lock.remove(&epk);
        }

        Ok(signature.into())
    }
}

bindings::export!(Component with_types_in bindings);

// Process the credentials, create and set the WALLET, emit the values
fn process_creds(credentials: Credentials) {
    let username = credentials.username.to_string();
    let password = credentials.password.to_string();

    let Ok(wallet) = Wallet::new(credentials) else {
        log("Failed to create Wallet from credentials");
        return;
    };

    let encrypted_seed = wallet.encrypted_seed().unwrap();

    // set the static
    WALLET.lock().unwrap().replace(wallet);

    // unlock evt
    emit(&Event::Text(StringEvent {
        name: "unlocked".to_string(),
        value: "true".to_string(),
    }));

    // emit username and password so they can be persisted for easy login
    emit(&Event::Text(StringEvent {
        name: "username".to_string(),
        value: username,
    }));

    emit(&Event::Text(StringEvent {
        name: "password".to_string(),
        value: password,
    }));

    emit(&Event::Text(StringEvent {
        name: "encrypted_seed".to_string(),
        value: format!(
            "[{}]",
            encrypted_seed
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<String>>()
                .join(","),
        ),
    }));
}

/// Helper fn which does `|e| Error::MultikeyError(&e.to_string())`
fn multikey_error(e: impl std::fmt::Display) -> MkError {
    MkError::MultikeyError(e.to_string())
}
