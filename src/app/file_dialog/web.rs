use std::cell::RefCell;
use std::rc::Rc;

use eframe::wasm_bindgen::prelude::Closure;
use eframe::wasm_bindgen::{JsCast, JsValue};
use eframe::web_sys;
use eframe::web_sys::{js_sys::Uint8Array, Event, FileReader};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct FileDialog {
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
}

impl FileDialog {
    pub fn file_dialog(
        &mut self,
        ui: &mut egui::Ui,
        on_load_callback: impl FnMut(String, Vec<u8>) + 'static,
    ) -> Result<(), JsValue> {
        ui.label("Drag-and-drop files onto the window!");

        if ui.button("Open file…").clicked() {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let input = document.create_element("input").unwrap();
            input.set_attribute("type", "file").unwrap();

            // Wrap the callback in Rc<RefCell<...>> to allow multiple uses
            let callback = Rc::new(RefCell::new(on_load_callback));

            let input: web_sys::HtmlInputElement = input.dyn_into().unwrap();
            input.set_attribute("multiple", "false").unwrap();
            input.set_attribute("accept", ".wasm").unwrap();
            let on_loaded = {
                let callback = callback.clone();
                Closure::wrap(Box::new(move |event: Event| {
                    let target = event.target().expect("Event should have a target");
                    let file_reader = target
                        .dyn_ref::<FileReader>()
                        .expect("Target should be a FileReader");

                    if let Ok(result) = file_reader.result() {
                        let array = Uint8Array::new(&result);
                        let bytes = array.to_vec();

                        // Now you have the file bytes in the `bytes` vector
                        tracing::info!("File loaded, size: {} bytes", bytes.len());

                        // Here you can process the bytes as needed
                        // For example, you could pass them to another function:
                        // process_file_bytes(&bytes);

                        // Call the provided callback with the loaded bytes
                        callback.borrow_mut()("file.wasm".to_owned(), bytes);
                    }
                }) as Box<dyn FnMut(Event)>)
            };

            let file_reader = FileReader::new()?;
            file_reader.set_onload(Some(on_loaded.as_ref().unchecked_ref()));
            on_loaded.forget(); // Prevent the closure from being dropped

            let onchange = Closure::wrap(Box::new(move |event: Event| {
                let target = event.target().expect("Event should have a target");
                let input: web_sys::HtmlInputElement = target
                    .dyn_into()
                    .expect("Target should be an HtmlInputElement");

                if let Some(file) = input.files().and_then(|files| files.get(0)) {
                    file_reader
                        .read_as_array_buffer(&file)
                        .expect("Failed to read file");
                }
            }) as Box<dyn FnMut(Event)>);

            input.add_event_listener_with_callback("change", onchange.as_ref().unchecked_ref())?;
            onchange.forget(); // Prevent the closure from being dropped

            // Append the input to the document body
            document.body().unwrap().append_child(&input)?;

            return Ok(());
        }

        if let Some(picked_path) = &self.picked_path {
            ui.vertical(|ui| {
                ui.label("Picked file:");
                ui.monospace(picked_path);
            });
        }

        // Show dropped files (if any):
        if !self.dropped_files.is_empty() {
            ui.group(|ui| {
                ui.label("Plugins:");

                for file in &self.dropped_files {
                    let mut info = if let Some(path) = &file.path {
                        path.display().to_string()
                    } else if !file.name.is_empty() {
                        file.name.clone()
                    } else {
                        "???".to_owned()
                    };

                    let mut additional_info = vec![];
                    if !file.mime.is_empty() {
                        additional_info.push(format!("type: {}", file.mime));
                    }
                    if let Some(bytes) = &file.bytes {
                        additional_info.push(format!("{} bytes", bytes.len()));
                    }
                    if !additional_info.is_empty() {
                        info += &format!(" ({})", additional_info.join(", "));
                    }

                    ui.label(info);
                }
            });
        }

        // Collect dropped files:
        ui.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });
        Ok(())
    }
}
