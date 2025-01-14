#[allow(warnings)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod bindings;

use bindings::exports::component::plugin::run::Guest;
use bindings::host::component::host::{emit, log, order, random_byte, AllCommands, Event};
//{
//    emit, get_mk, log, order, prove, random_byte, AllCommands, Event, KeyArgs, ProveArgs,
//};

use bindings::host::component::types::StringEvent;
use multicid::EncodedVlad;

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

///// The Rhai script that will be executed when the component is loaded.
///// This is separate here so we can test the Rhai script in isolation.
//const SCRIPT: &str = validate_rhai!(
//    r#"
//        render(`
//            <Vertical>
//                <Label>Peer Book</Label>
//                <TextEdit>{{vlad}}</TextEdit>
//                <Button on_click=search(vlad)>Search</Button>
//                ${if is_def_var("get_record") {
//                    `
//                    <Horizontal>
//                        <Label>VLAD</Label>
//                        <Label>Result</Label>
//                    </Horizontal>
//                    `
//                    +
//                    // loop over get_record and display the key and optional result if it exists
//                    get_record.values().map(|val| {
//                        `
//                        <Horizontal>
//                            <Label>${val}</Label>
//                        </Horizontal>
//                        `
//                    })
//                    .reduce(|acc, x| acc + x)
//
//                } else {
//                    `<Label>Enter a VLAD to search.</Label>`
//                }}
//            </Vertical>
//        `)
//        "#
//);

impl Guest for Component {
    /// Say hello!
    fn load() -> String {
        include_str!(concat!(env!("OUT_DIR"), "/peer-book.rhai")).to_string()
    }

    fn search(vlad: String) -> Result<String, String> {
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

        // Send a `Get` command so the system tries to look up the Vlad and its details
        Ok("Plog would go here".to_string())
    }
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {}
