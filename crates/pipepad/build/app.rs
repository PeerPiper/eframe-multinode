//! Build code for ./build.rs process.
//!
//! This could be it's own app in ot's own crate for code sharing and whatnot,
//! but for now we'll just keep it together with the plugin code.
use html_egui_bindgen::{Division, Label, TextArea};

pub(crate) fn gen_script() -> String {
    let title = Label::builder()
        .text("Pipe Pad: AI Networked Notes")
        .build();

    // {{this}} will create a Rhai Scope variable of the same name for us
    // to save the value of the text into
    let textarea = TextArea::builder().placeholder("{{pipepad}}").build();

    let pipepad = Division::builder()
        .push(title)
        .push(textarea)
        .build()
        .to_string();

    format!(
        r#"
// pipepad.rhai 
// This is rhai script, controlling the logic flow of what html fragments to render.

// Here, we call the function "render" which is 'registered' witht the rhai engine,
// so it will call some Rust code to show the rendered html.
render(`
<!-- First we show the search vlad section -->
{pipepad}
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
