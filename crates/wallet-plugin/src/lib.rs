#[allow(warnings)]
mod bindings;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use bindings::component::plugin::host::{emit, log, random_byte};
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::{Guest, KeyArgs, MkError, ProveArgs};

use bestsign_core::ops::config::defaults::{DEFAULT_ENTRYKEY, DEFAULT_VLAD_KEY};
use bestsign_core::{mk, Codec, EncodedMultikey, Key, Multikey, Views as _};
use seed_keeper_core::credentials::{Credentials, MinString, Wallet};

static WALLET: LazyLock<Mutex<Option<Wallet>>> = LazyLock::new(|| Mutex::new(None));

/// static HashMap to map the epk to the (key, mk) tuple.
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
        r#"
        if !is_def_var("encrypted_seed") {
            render(`
                <Vertical>
                    // TextEdit automatically adds the template literal value to Rhai scope
                    // when text is changed, so it can be accessed in the login function
                    <TextEdit>{{username}}</TextEdit>
                    <TextEdit>{{password}}</TextEdit>
                    <Button on_click=login(username, password)>Login</Button>
                </Vertical>
            `)

        } else {
            render(`
                <Vertical>
                    <Label>{{encrypted_seed}}</Label>
                </Vertical>
            `)
        }
        "#
        .to_string()
    }

    fn login(username: String, password: String) {
        log("Login button clicked");

        // return early if Err Result
        let Ok(username) = MinString::<8>::new(&username) else {
            log(&format!("Failed to create MinString from {}", username));
            return;
        };

        let Ok(password) = MinString::<8>::new(&password) else {
            log(&format!("Failed to create MinString from {}", password));
            return;
        };

        let credentials = Credentials {
            username,
            password,
            encrypted_seed: None,
        };

        let Ok(wallet) = Wallet::new(credentials) else {
            log("Failed to create Wallet from credentials");
            return;
        };

        // safe to unwrap because we know the seed is corect AES length
        let encrypted_seed = wallet.encrypted_seed().unwrap();

        // set the static
        WALLET.lock().unwrap().replace(wallet);

        let evt = Event {
            name: "encrypted_seed".to_string(),
            value: format!(
                "[{}]",
                encrypted_seed
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            ),
        };
        emit(&evt);
    }

    /// Gets the Multikey
    // get-mk: func(args: key-args) -> list<u8>;
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
                log("Creating Multikey from seed");
                let wallet_lock = WALLET.lock().unwrap();
                let Some(wallet) = wallet_lock.as_ref() else {
                    return Err(MkError::WalletUninitialized);
                };

                mk::Builder::new_from_seed(codec, wallet.seed())
                    .map_err(|e| MkError::InvalidCodec(format!("Invalid codec, found: {}", e)))?
            };

        let mk = mk_buildr.try_build().unwrap();

        log("Getting PK for Multikey");

        let pk = mk
            .conv_view()
            .map_err(multikey_error)?
            .to_public_key()
            .map_err(multikey_error)?;

        log(&format!("Got PK: {:?}", pk));

        let key = Key::try_from(args.key).map_err(multikey_error)?;
        let epk = EncodedMultikey::from(pk.clone());

        // add to key map
        EPK_MAP
            .lock()
            .unwrap()
            .insert(epk, (key.clone(), mk.clone()));

        Ok(pk.into())
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

        let is_secret = mk.attr_view().map_err(multikey_error)?.is_secret_key();

        log(&format!("Is secret key: {}", is_secret));

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

/// Helper fn which does `|e| Error::MultikeyError(&e.to_string())`
fn multikey_error(e: impl std::fmt::Display) -> MkError {
    MkError::MultikeyError(e.to_string())
}
