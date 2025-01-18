//! Build code for ./build.rs process.
//!
//! This could be it's own app in ot's own crate for code sharing and whatnot,
//! but for now we'll just keep it together with the plugin code.
use html_egui_bindgen::{Button, Division, Input, Label, Paragraph, Span};
use html_to_egui::{Action, Handler, Selectors};

pub(crate) fn gen_script() -> String {
    let peer_book_label = Label::builder().text("Peer Book").build();

    let vlad_input = Input::builder().value("{{vlad}}").build();

    let search_button = Button::new_with_func(
        Action::OnClick,
        Handler::builder()
            .named("search".to_string())
            .args(vec!["vlad".to_string()])
            .build(),
    )
    .text("Search")
    .build();

    let search_div = Division::builder()
        .push(peer_book_label)
        .push(vlad_input)
        .push(search_button)
        .build()
        .to_string();

    let no_get_record = Paragraph::builder().text("Enter a VLAD to search.").build();

    // Has record should show:
    // "Found. Add to contacts?" with an input field for the nickname,
    // and a button to add to contacts.
    let has_get_record = Division::builder()
        .push(
            Division::builder()
                .push(Label::builder().text("${val}").build())
                .class(Selectors::FlexRow)
                .build(),
        )
        // push a child div with 1) input field for nickname, 2) button to add to contacts
        .push(
            Division::builder()
                .push(
                    Label::builder()
                        .text("Found Vlad! Add nickname to contacts?")
                        .build(),
                )
                .push(Input::builder().value("{{nickname}}").build())
                .push(
                    Button::new_with_func(
                        Action::OnClick,
                        Handler::builder()
                            // We will create a wit interface caled add-to-contacts, and a
                            // wasm function in our Rust lib.rs called add_to_contacts
                            .named("add-to-contacts".to_string())
                            .args(vec!["vlad".to_string(), "nickname".to_string()])
                            .build(),
                    )
                    .text("Add to contacts")
                    .build(),
                )
                .build(),
        )
        .build()
        .to_string();

    let inner_logic = format!(
        r#"
    if is_def_var("get_record") {{
        // loop over get_record and display the key and optional result if it exists
        get_record.values().map(|val| {{
            `{has_get_record}`
        }})
        .reduce(|acc, x| acc + x)

    }} else {{
        `{no_get_record}`
    }}
"#
    );

    let rhai_control_section = Division::builder()
        // We could also do this: instead of 3 text()s, we could do .with_rhai(inner_logic)
        //.with_rhai(inner_logic)
        .text("${ ")
        .text(inner_logic)
        .text(" } ")
        .build();

    // Lastly, we want to list out every vlad & nickname we have added.
    // We have registered a function called `contacts: func() -> list>list<string>>`
    // which gives us a list of contacts' vladid, nickname and notes.
    // So in this section, we call that function, then if not false,
    // iterate over the results and display them.
    // let nickname = "nickname";
    // let notes = "notes";
    // let vlad = "vlad";
    let show_contact = Division::builder()
        .push(Span::builder().text("${vlad}").build())
        .push(Span::builder().text("${nickname}").build())
        .push(Span::builder().text("${notes}").build())
        .class(Selectors::FlexRow)
        .build()
        .to_string();

    let show_contacts_block = Division::builder()
        .with_rhai(format!(
            r#"
    // Save the loaded contacts from wasm into rhai Scope.
    // This should persist them for the next time we load the plugin.

    // loop over contacts and display the vlad and nickname and notes 
    if contacts().len() > 0 {{
        contacts().map(|contact| {{
            let vlad = contact[0];
            let nickname = contact[1];
            let notes = contact[2];

            `{show_contact}`

        }})
        .reduce(|acc, s| acc + s, "")
    }} else {{
        "<p>No contacts found.</p>"
        }}
"#
        ))
        .build();

    let contacts = Division::builder()
        .push(
            Paragraph::builder()
                .text("List Vlad nicknames from our PeerBook")
                .build(),
        )
        .push(show_contacts_block)
        .build()
        .to_string();

    format!(
        r#"
// peer-book.rhai 
// This is rhai script, controlling the logic flow of what html fragments to render.

// Here, we call the function "render" which is 'registered' witht the rhai engine,
// so it will call some Rust code to show the rendered html.
render(`
<!-- First we show the search vlad section -->
{search_div}

<!-- Next we show the results of the search -->
{rhai_control_section}

<!-- List Vlad & Nicknames in our PeerBook -->
{contacts}
`)

"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::gen_script;

    #[test]
    fn test_gen_script() {
        let script = super::gen_script();
        println!("{}", script);
        assert!(script.contains("render"));
    }
}
