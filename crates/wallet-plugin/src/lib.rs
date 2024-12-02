#[allow(warnings)]
mod bindings;

use bindings::component::plugin::host::{emit, log, random_byte};
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::Guest;

use seed_keeper_core::credentials::{Credentials, MinString, Wallet};
use seed_keeper_core::error;

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
                    <Label>{{username}}</Label>
                    <Label>{{password}}</Label>
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
            log("Failed to create MinString from username");
            return;
        };

        let Ok(password) = MinString::<8>::new(&password) else {
            log("Failed to create MinString from password");
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

        let evt = Event {
            name: "encrypted_seed".to_string(),
            value: encrypted_seed.iter().map(|b| b.to_string()).collect(),
        };
        emit(&evt);
    }
}

bindings::export!(Component with_types_in bindings);
