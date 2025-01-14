//! Build code for ./build.rs process.
//!
//! This could be it's own app in ot's own crate for code sharing and whatnot,
//! but for now we'll just keep it together with the plugin code.
use html_egui_bindgen::{Button, Division, Input, Label, Paragraph};
use html_to_egui::{Action, DivSelectors, Handler};

//  Below is the equivalent for this:
//    r#"
//render(`
//    <Vertical>
//        <Label>Peer Book</Label>
//        <TextEdit>{{vlad}}</TextEdit>
//        <Button on_click=search(vlad)>Search</Button>
//        ${if is_def_var("get_record") {
//            // loop over get_record and display the key and optional result if it exists
//            get_record.values().map(|val| {
//                `
//                <Horizontal>
//                    <Label>${val}</Label>
//                </Horizontal>
//                `
//            })
//            .reduce(|acc, x| acc + x)
//
//        } else {
//            `<Label>Enter a VLAD to search.</Label>`
//        }}
//    </Vertical>
//`)
//"#
pub(crate) fn gen_script() -> String {
    let peer_book_label = Label::builder().text("Peer Book").build();

    let vlad_input = Input::new_with_func(
        Action::OnChange,
        Handler::builder()
            .named("set_vlad".to_string())
            .args(vec!["vlad".to_string()])
            .build(),
    )
    .value("{{vlad}}")
    .build();

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

    let has_get_record = Division::builder()
        .push(
            Division::builder()
                .push(Label::builder().text("${val}").build())
                .class(DivSelectors::FlexRow)
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
        .text("${ ")
        .text(inner_logic)
        .text(" } ")
        .build();

    format!(
        r#"
// peer-book.rhai 
// This is rhai script, controlling the logic flow of what html fragments to render.
// The rhai will check for the presence of get_record via is_def_var("get_record")
// 
// Next, it depends on whether get_record is defined or not
// To inject the rhai logic in the middle of this html, we use dollar sign plus brakets
render(`
<!-- First we show the search vlad section -->
{search_div}

<!-- Next we show the results of the search -->
{rhai_control_section}

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
