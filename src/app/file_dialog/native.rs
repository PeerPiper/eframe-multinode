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
        on_load_callback: impl Fn(String, Vec<u8>) + 'static,
        //    platform: &mut crate::app::Platform
    ) -> Result<(), crate::app::error::Error> {
        ui.label("Drag-and-drop files onto the window!");

        if ui.button("Open file…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.picked_path = Some(path.file_stem().unwrap().to_string_lossy().to_string());
                // load the file bytes
                let bytes = std::fs::read(&path)
                    .unwrap_or_else(|err| panic!("Failed to read file: {}", err));
                self.dropped_files.push(egui::DroppedFile {
                    bytes: None,
                    path: None, // Some(path.display().to_string().into()),
                    last_modified: None,
                    name: self.picked_path.clone().unwrap(),
                    mime: "".to_owned(),
                });

                // call platform load plugin
                // platform.load_plugin(self.picked_path.clone().unwrap(), bytes);
                on_load_callback(self.picked_path.clone().unwrap(), bytes);
            }
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
