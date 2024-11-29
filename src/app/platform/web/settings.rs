//! Web speicifc settings
use multiaddr::Multiaddr;

/// Settings specific to the native platform
#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct Settings {
    /// Whether the settings window is open
    open: bool,
}

impl Settings {
    /// Show the current settings
    pub fn show(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui, _addr: &Multiaddr) {
        egui::Window::new("Browser Options")
            .open(&mut self.open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Browser Settings");
            });
    }
}
